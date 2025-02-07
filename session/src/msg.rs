use crate::{queue::Queue, token::ClientToken};
use bytecheck::CheckBytes;
use rkyv::{
    api::high::*, de::Pool, rancor::*, ser::allocator::ArenaHandle, util::*, Archive, Deserialize,
    Serialize,
};
use std::fmt::Debug;

pub trait Message = 'static + Clone + Send + Sync + Archive + Debug
where
    <Self as Archive>::Archived:
        for<'a> CheckBytes<HighValidator<'a, Error>> + Deserialize<Self, Strategy<Pool, Error>>,
    Self: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, Error>>;

#[derive(Clone, Copy, Debug, Archive, Serialize, Deserialize)]
pub struct Msg<Q: Queue> {
    pub token: ClientToken,
    pub queue: Q,
    pub msg_type: MsgType<Q>,
}

impl<Q: Queue> Msg<Q> {
    pub fn leave(token: ClientToken, queue: Q) -> Self {
        let msg_type = MsgType::Reconnect;
        Self {
            token,
            queue,
            msg_type,
        }
    }
}

#[derive(Clone, Copy, Debug, Archive, Serialize, Deserialize)]
pub enum MsgType<Q: Queue> {
    Join,
    Reconnect,
    Accept,
    Leave,
    Action(Q::Action),
}
