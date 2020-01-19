#[macro_use]
extern crate log;

use std::cell::RefCell;

use anyhow::{Context, Result};
use futures::{stream, Future, Stream};
use telegram_bot::{Api, DeleteMessage, Message, MessageEntityKind, MessageKind, UpdateKind};
use tokio_core::reactor::{Core, Interval};

use self::{config::Config, storage::Storage};

mod config;
mod storage;

fn from_tg_error(err: telegram_bot::Error) -> anyhow::Error {
    anyhow::anyhow!("{}", err)
}

fn to_utf16(val: impl AsRef<str>) -> Vec<u16> {
    val.as_ref().encode_utf16().collect()
}

fn are_hashtags_present(message: &Message, hashtags: &[Vec<u16>]) -> bool {
    let (data, entities) = match &message.kind {
        MessageKind::Text { data, entities } => (data, entities),
        _ => return false,
    };

    let locations = entities
        .iter()
        .filter(|entity| entity.kind == MessageEntityKind::Hashtag)
        .filter(|entity| hashtags.iter().any(|h| h.len() as i64 == entity.length - 1))
        .map(|entity| (entity.offset + 1) as usize..(entity.offset + entity.length) as usize)
        .collect::<Vec<_>>();

    if locations.is_empty() {
        return false;
    }

    let data_utf16 = to_utf16(data);

    locations
        .into_iter()
        .filter_map(|loc| data_utf16.get(loc))
        .any(|slice| hashtags.iter().any(|h| slice == &h[..]))
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

    let nodelete_hashtags = config
        .nodelete_hashtags
        .iter()
        .map(to_utf16)
        .collect::<Vec<_>>();

    let fetching = api
        .stream()
        .map_err(|err| from_tg_error(err).context("failed to fetch an update"))
        .for_each(|update| {
            if let UpdateKind::Message(message) = update.kind {
                if are_hashtags_present(&message, &nodelete_hashtags) {
                    return Ok(());
                }

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
