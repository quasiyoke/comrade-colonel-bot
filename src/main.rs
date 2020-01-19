#[macro_use]
extern crate log;

use std::{cell::RefCell, error::Error};

use futures::{Future, Stream};
use telegram_bot::{Api, DeleteMessage, UpdateKind};
use tokio_core::reactor::{Core, Interval};

use self::{config::Config, storage::Storage};

mod config;
mod storage;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let config = envy::from_env::<Config>()?;

    info!("starting with config: {:?}", config);

    let api = Api::configure(&*config.telegram_bot_token)
        .build(&handle)
        .unwrap();
    let storage = RefCell::new(Storage::new(&config.storage_path, config.message_lifetime));

    let fetching = api.stream().for_each(|update| {
        if let UpdateKind::Message(message) = update.kind {
            storage.borrow_mut().add(message);
        }

        Ok(())
    });

    let deletion = Interval::new(config.deletion_period, &handle)
        .unwrap()
        .for_each(|_| {
            let mut storage = storage.borrow_mut();

            for message in storage.clean().unwrap() {
                info!("deleting message {:?}", message);
                api.spawn(DeleteMessage::new(
                    message.chat_telegram_id,
                    message.telegram_id,
                ));
            }

            Ok(())
        })
        .map_err(From::from);

    core.run(fetching.join(deletion))?;
    Ok(())
}
