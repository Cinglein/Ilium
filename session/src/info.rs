use crate::*;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type AsInfo<Q> =
    Info<<<Q as AsQueue>::Action as Action>::User, <<Q as AsQueue>::Action as Action>::Shared>;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "I: Serialize + DeserializeOwned")]
pub enum StateInfo<I: 'static + Message> {
    Closed,
    Queue,
    Lobby,
    Session(I),
}

/// Trait for info serialized to the client
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Info<U: UserState, S: SharedState> {
    pub users: hashbrown::HashMap<u64, U::Info>,
    pub shared: S::Info,
    pub index: u64,
}
