use crate::{action::Action, msg::Message, queue::Queue, state::*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub type AsInfo<Q> =
    Info<<<Q as Queue>::Action as Action>::User, <<Q as Queue>::Action as Action>::Shared>;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(bound = "I: Serialize + DeserializeOwned")]
pub enum StateInfo<I: 'static + Message> {
    Closed,
    Queue,
    Lobby,
    Session(I),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Info<U: UserState, S: SharedState> {
    pub users: bevy::utils::hashbrown::HashMap<u64, U::Info>,
    pub shared: S::Info,
    pub index: u64,
}
