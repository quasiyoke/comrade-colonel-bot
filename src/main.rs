#[macro_use]
extern crate log;

use std::cell::RefCell;

use anyhow::{Context, Result};
use futures::{stream, Future, Stream};
use telegram_bot::{Api, DeleteMessage, UpdateKind};
use tokio_core::reactor::{Core, Interval};

use self::{config::Config, storage::Storage};

mod config;
mod storage;

fn from_tg_error(e: telegram_bot::Error) -> anyhow::Error {
    anyhow::anyhow!("{}", e)
}

fn main() -> Result<()> {
    env_logger::init();
    let mut core = Core::new()?;
    let handle = core.handle();

    let config = envy::from_env::<Config>().context("failed to build the config from env")?;

    info!("starting with config: {:?}", config);

    let api = Api::configure(&*config.telegram_bot_token)
        .build(&handle)
        .map_err(from_tg_error)
        .context("failed to configure tg api")?;

    let storage = Storage::new(&config.storage_path, config.message_lifetime)
        .context("failed to open the storage")?;
    let storage = RefCell::new(storage);

    let fetching = api
        .stream()
        .map_err(|err| from_tg_error(err).context("failed to fetch an update"))
        .for_each(|update| {
            if let UpdateKind::Message(message) = update.kind {
                storage
                    .borrow_mut()
                    .add(message)
                    .context("failed to add to the storage")?;
            }

            Ok(())
        });

    let deletion = Interval::new(config.deletion_period, &handle)?
        .map_err(anyhow::Error::from)
        .and_then(|_| {
            let mut storage = storage.borrow_mut();
            let messages = storage.clean().context("failed to clean the storage")?;
            Ok(stream::iter_ok::<_, anyhow::Error>(messages))
        })
        .flatten()
        .for_each(|message| {
            info!("deleting message {:?}", message);

            api.spawn(DeleteMessage::new(
                message.chat_telegram_id,
                message.telegram_id,
            ));

            Ok(())
        })
        .map_err(From::from);

    core.run(fetching.join(deletion))?;
    Ok(())
}
