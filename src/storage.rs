use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags, Result};
use telegram_bot::{ChatId, Message as TelegramMessage, MessageId};

#[derive(Debug)]
pub struct Message {
    id: u32,
    pub telegram_id: MessageId,
    pub chat_telegram_id: ChatId,
    date: u64,
}

impl From<TelegramMessage> for Message {
    fn from(message: TelegramMessage) -> Message {
        Message {
            id: 0,
            telegram_id: message.id,
            chat_telegram_id: message.chat.id(),
            date: message.date as u64,
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
    pub fn new(path: &str, lifetime: u64) -> Storage {
        debug!("Looking for storage at `{}`", path);
        let connection = Connection::open_with_flags(
            Path::new(&String::from(path)),
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        ).unwrap();
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS message (
                    id                  INTEGER PRIMARY KEY,
                    telegram_id         INTEGER NOT NULL,
                    chat_telegram_id    INTEGER NOT NULL,
                    date                INTEGER NOT NULL
                )",
                &[],
            ).expect("Troubles during table creation");
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
                &[&telegram_id, &chat_telegram_id, &(message.date as i64)],
            ).expect("Troubles during message insertion");
    }

    pub fn clean(&mut self) -> Result<Vec<Message>> {
        let transaction = self.connection.transaction()?;
        let obsolete_messages = delete_obsolete_messages(&transaction, now() - self.lifetime)?;
        transaction.commit()?;
        Ok(obsolete_messages)
    }
}

fn delete_obsolete_messages(connection: &Connection, threshold_date: u64) -> Result<Vec<Message>> {
    debug!("Looking for obsolete messages before {}", threshold_date);

    let mut statement = connection.prepare(
        "SELECT id, telegram_id, chat_telegram_id, date
            FROM message
            WHERE date < ?1",
    )?;
    let messages_iterator = statement
        .query_map(&[&(threshold_date as i64)], |row| {
            let telegram_id: i64 = row.get(1);
            let chat_telegram_id: i64 = row.get(2);
            let date: i64 = row.get(3);
            Message {
                id: row.get(0),
                telegram_id: telegram_id.into(),
                chat_telegram_id: chat_telegram_id.into(),
                date: date as u64,
            }
        })?.filter_map(|message_result| message_result.ok());
    let messages: Vec<Message> = messages_iterator.collect();
    debug!("Obsolete messages found: {}", messages.len());

    connection.execute(
        "DELETE FROM message
            WHERE date < ?1",
        &[&(threshold_date as i64)],
    )?;

    Ok(messages)
}
