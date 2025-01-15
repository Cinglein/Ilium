use crate::{msg::Message, state::*};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "I: Message")]
pub enum StateInfo<I: 'static + Message> {
    Closed,
    Queue,
    Lobby,
    Session(I),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Info<U: UserState, S: SharedState> {
    pub users: Vec<U::Info>,
    pub shared: S::Info,
}

/// A wrapper for data that is only sometimes visible.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
