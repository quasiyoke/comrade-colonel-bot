use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::{SystemTime, UNIX_EPOCH};

use telegram_bot::{ChatId, Message, MessageId};

#[derive(Eq)]
struct Item {
    chat_id: ChatId,
    message_id: MessageId,
    date: u64,
}

impl From<Message> for Item {
    fn from(message: Message) -> Item {
        Item {
            chat_id: message.chat.id(),
            message_id: message.id,
            date: message.date as u64,
        }
    }
}

impl Ord for Item {
    fn cmp(&self, other: &Item) -> Ordering {
        other.date.cmp(&self.date)
    }
}

impl PartialOrd for Item {
    fn partial_cmp(&self, other: &Item) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.date == other.date
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub struct Storage {
    heap: BinaryHeap<Item>,
    lifetime: u64,
}

impl Storage {
    pub fn new(lifetime: u64) -> Storage {
        Storage {
            heap: BinaryHeap::new(),
            lifetime,
        }
    }

    pub fn add(&mut self, message: Message) {
        self.heap.push(message.into())
    }

    pub fn clean(&mut self) -> OutdateIter {
        OutdateIter {
            heap: &mut self.heap,
            date: now() - self.lifetime,
        }
    }
}

pub struct OutdateIter<'a> {
    heap: &'a mut BinaryHeap<Item>,
    date: u64,
}

impl<'a> Iterator for OutdateIter<'a> {
    type Item = (ChatId, MessageId);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.heap.peek() {
            if item.date > self.date {
                return None;
            }
        }

        self.heap.pop().map(|item| (item.chat_id, item.message_id))
    }
}
