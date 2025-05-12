use crate::{queue::Queue, token::ClientToken};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

pub trait Message = 'static + Clone + Send + Sync + Serialize + DeserializeOwned + Debug;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(bound = "Q: Serialize + DeserializeOwned")]
pub struct Msg<Q: Queue> {
    pub token: ClientToken,
    pub queue: Q,
    pub msg_type: MsgType<Q>,
}

impl<Q: Queue> Msg<Q> {
    pub fn join(token: ClientToken, queue: Q) -> Self {
        let msg_type = MsgType::Join;
        Self {
            token,
            queue,
            msg_type,
        }
    }
    pub fn accept(token: ClientToken, queue: Q) -> Self {
        let msg_type = MsgType::Accept;
        Self {
            token,
            queue,
            msg_type,
        }
    }
    pub fn leave(token: ClientToken, queue: Q) -> Self {
        let msg_type = MsgType::Leave;
        Self {
            token,
            queue,
            msg_type,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum MsgType<Q: Queue> {
    Join,
    Reconnect,
    Accept,
    Leave,
    Action(Q::Action),
}
