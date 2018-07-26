extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

use std::cell::RefCell;
use std::env;
use std::time::Duration;

use futures::{Future, Stream};
use telegram_bot::{Api, DeleteMessage, UpdateKind};
use tokio_core::reactor::{Core, Interval};

use storage::Storage;

mod storage;

const LIFETIME: u64 = 42 * 60 * 60;
const PERIOD: Duration = Duration::from_secs(5 * 60);

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
    let api = Api::configure(token).build(&handle).unwrap();

    let storage = RefCell::new(Storage::new(LIFETIME));

    let fetching = api.stream().for_each(|update| {
        if let UpdateKind::Message(mut message) = update.kind {
            use std::time::{SystemTime, UNIX_EPOCH};

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            message.date += 20;

            println!("ADD {} {} {}", message.id, message.date, now);
            storage.borrow_mut().add(message);
        }

        Ok(())
    });

    let deleting = Interval::new(PERIOD, &handle)
        .unwrap()
        .for_each(|_| {
            let mut storage = storage.borrow_mut();

            for (chat_id, message_id) in storage.clean() {
                println!("RM {}", message_id);
                api.spawn(DeleteMessage::new(chat_id, message_id));
            }

            Ok(())
        })
        .map_err(From::from);

    core.run(fetching.join(deleting)).unwrap();
}
