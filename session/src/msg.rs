use crate::{queue::Queue, token::ClientToken};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

pub trait Message = 'static + Clone + Copy + Send + Sync + Serialize + DeserializeOwned + Debug;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(bound = "Q: Serialize + DeserializeOwned")]
pub struct Msg<Q: Queue> {
    pub token: ClientToken,
    pub msg_type: MsgType<Q>,
}

impl<Q: Queue> Msg<Q> {
    pub fn leave(token: ClientToken) -> Self {
        let msg_type = MsgType::Reconnect;
        Self { token, msg_type }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(bound = "Q: Serialize + DeserializeOwned")]
pub enum MsgType<Q: Queue> {
    Join { queue: Q },
    Reconnect,
    Accept,
    Leave,
    Action(<Q as Queue>::Action),
}
