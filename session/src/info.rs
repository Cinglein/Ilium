use crate::msg::Message;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(bound = "I: Message")]
pub enum StateInfo<I: 'static + Message> {
    Closed,
    Queue,
    Lobby,
    Session(I),
}

/// A wrapper for data that is only sometimes visible.
#[derive(Clone, Serialize, Deserialize)]
#[serde(bound = "T: Message")]
pub enum Hidden<T: Message> {
    Unseen,
    Seen(T),
    LastSeen(T),
}

impl<T: Message> Hidden<T> {
    pub fn update(self, rhs: Self) -> Self {
        match (&self, &rhs) {
            (Self::LastSeen(_), Self::Unseen) => self,
            _ => rhs,
        }
    }
}
