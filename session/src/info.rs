use crate::{msg::Message, state::*};
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
pub enum StateInfo<I: 'static + Message> {
    Closed,
    Queue,
    Lobby,
    Session(I),
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug)]
pub struct Info<U: UserState, S: SharedState> {
    pub users: Vec<U::Info>,
    pub shared: S::Info,
}

/// A wrapper for data that is only sometimes visible.
#[derive(Clone, Debug, Archive, Serialize, Deserialize)]
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
