use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags};
use telegram_bot::{ChatId, Message as TelegramMessage, MessageId};
use time::Timespec;

#[derive(Debug)]
pub struct Message {
    id: u32,
    pub telegram_id: MessageId,
    pub chat_telegram_id: ChatId,
    date: Timespec,
}

impl From<TelegramMessage> for Message {
    fn from(message: TelegramMessage) -> Message {
        Message {
            id: 0,
            telegram_id: message.id,
            chat_telegram_id: message.chat.id(),
            date: Timespec::new(message.date, 0),
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct Storage {
    connection: Connection,
    lifetime: u64,
}

impl Storage {
    pub fn new(path: &String, lifetime: u64) -> Storage {
        debug!("Looking for storage at `{}`", path);
        let connection = Connection::open_with_flags(
            Path::new(path),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        ).unwrap();
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS message (
                    id                  INTEGER PRIMARY KEY,
                    telegram_id         INTEGER NOT NULL,
                    chat_telegram_id    INTEGER NOT NULL,
                    date                TEXT NOT NULL
                )",
                &[],
            )
            .expect("Troubles during table creation");
        Storage {
            connection,
            lifetime,
        }
    }

    pub fn add(&self, message: TelegramMessage) {
        let message: Message = message.into();
        debug!("Inserting message {:?}", message);
        let telegram_id: i64 = message.telegram_id.into();
        let chat_telegram_id: i64 = message.chat_telegram_id.into();
        self.connection
            .execute(
                "INSERT INTO message (telegram_id, chat_telegram_id, date)
                    VALUES (?1, ?2, ?3)",
                &[&telegram_id, &chat_telegram_id, &message.date],
            )
            .expect("Troubles during message insertion");
    }

    pub fn clean(&self) -> Vec<Message> {
        let threshold_date = self.threshold_date();
        debug!("Looking for messages before {:?} to delete", threshold_date);
        let mut statement = self
            .connection
            .prepare(
                "SELECT id, telegram_id, chat_telegram_id, date
                FROM message
                WHERE date < ?1",
            )
            .unwrap();
        let messages_iterator = statement
            .query_map(&[&threshold_date], |row| {
                let telegram_id: i64 = row.get(1);
                let chat_telegram_id: i64 = row.get(2);
                Message {
                    id: row.get(0),
                    telegram_id: telegram_id.into(),
                    chat_telegram_id: chat_telegram_id.into(),
                    date: row.get(3),
                }
            })
            .expect("Binding parameters to the query for messages to delete was failed")
            .filter_map(|message_result| message_result.ok());
        let messages = messages_iterator.collect();
        self.connection
            .execute("DELETE FROM message WHERE date < ?1", &[&threshold_date])
            .unwrap();
        messages
    }

    fn threshold_date(&self) -> Timespec {
        let threshold_timestamp = now() - self.lifetime;
        Timespec::new(threshold_timestamp as i64, 0)
    }
}
