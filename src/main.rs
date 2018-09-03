extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use std::cell::RefCell;
use std::env;
use std::str;
use std::time::Duration;

use futures::{Future, Stream};
use telegram_bot::{Api, DeleteMessage, UpdateKind};
use tokio_core::reactor::{Core, Interval};

use storage::Storage;

mod storage;

/// Default chat message lifetime (seconds).
/// Please [note][1] that message can only be deleted if it was sent less than 48 hours ago.
/// [1]: https://core.telegram.org/bots/api#deletemessage
const MESSAGE_LIFETIME_DEFAULT: u64 = 42 * 60 * 60;
/// Default deletion period (seconds).
const DELETION_PERIOD_DEFAULT: u64 = 5 * 60;

fn env_var<T: str::FromStr>(name: &str) -> Option<T> {
    env::var(name).ok().and_then(|env_var| env_var.parse().ok())
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let token = env::var("TELEGRAM_BOT_TOKEN")
        .expect("Please specify TELEGRAM_BOT_TOKEN environment variable");
    let api = Api::configure(token).build(&handle).unwrap();

    let message_lifetime = env_var("MESSAGE_LIFETIME").unwrap_or(MESSAGE_LIFETIME_DEFAULT);
    let storage = RefCell::new(Storage::new(message_lifetime));

    let fetching = api.stream().for_each(|update| {
        if let UpdateKind::Message(message) = update.kind {
            storage.borrow_mut().add(message);
        }

        Ok(())
    });

    let deletion_period = env_var("DELETION_PERIOD").unwrap_or(DELETION_PERIOD_DEFAULT);
    let deletion = Interval::new(Duration::from_secs(deletion_period), &handle)
        .unwrap()
        .for_each(|_| {
            let mut storage = storage.borrow_mut();

            for (chat_id, message_id) in storage.clean() {
                api.spawn(DeleteMessage::new(chat_id, message_id));
            }

            Ok(())
        })
        .map_err(From::from);

    core.run(fetching.join(deletion)).unwrap();
}
