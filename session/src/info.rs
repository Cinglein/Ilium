use crate::{msg::Message, state::*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::hash::Hash;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "I: Serialize + DeserializeOwned")]
pub enum StateInfo<I: 'static + Message> {
    Closed,
    Queue,
    Lobby,
    Session(I),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Info<U: UserState, S: SharedState, I: Sized + Eq + PartialEq + Hash> {
    pub users: bevy::utils::hashbrown::HashMap<I, U::Info>,
    pub shared: S::Info,
}
