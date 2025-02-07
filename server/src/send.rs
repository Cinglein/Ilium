use crate::{account::Account, data::*, time::Ping};
use bevy::ecs::system::Resource;
use core::future::Future;
use rkyv::{
    api::high::HighSerializer, rancor::Source, ser::allocator::ArenaHandle, util::*, Serialize,
};
use session::{
    msg::Msg,
    queue::{Queue, QueueComponent},
};
use sqlx::*;

#[derive(Clone, Debug, bevy::prelude::Resource)]
pub struct Receiver<T>(kanal::Receiver<T>);

impl<T> Receiver<T> {
    pub fn new(r: kanal::Receiver<T>) -> Self {
        Self(r)
    }
    pub fn try_recv(&self) -> Result<Option<T>, kanal::ReceiveError> {
        self.0.try_recv()
    }
}

#[derive(Clone, Debug, bevy::prelude::Component)]
pub struct SendFrame(kanal::Sender<fastwebsockets::Frame<'static>>);

impl SendFrame {
    pub fn new(sender: kanal::Sender<fastwebsockets::Frame<'static>>) -> Self {
        Self(sender)
    }
    pub fn send_raw(&self, frame: fastwebsockets::Frame<'static>) {
        let _ = self.0.send(frame);
    }
    pub fn send<T, E>(&self, data: &T)
    where
        E: Source,
        T: for<'a> Serialize<HighSerializer<AlignedVec, ArenaHandle<'a>, E>>,
    {
        if let Ok(bytes) = rkyv::to_bytes(data) {
            let frame = fastwebsockets::Frame::new(
                true,
                fastwebsockets::OpCode::Binary,
                None,
                bytes.into_vec().into(),
            );
            let _ = self.0.send(frame);
        }
    }
}

pub enum QueueSignal<U: UserData> {
    Join {
        send_frame: SendFrame,
        ping: Ping,
        user_data: U,
        account: Account,
    },
    Accept {
        account: Account,
    },
    Leave {
        account: Account,
    },
}

pub struct ReconnectSignal {
    pub send_frame: SendFrame,
    pub ping: Ping,
    pub account: Account,
}

pub struct ActionSignal<QC: QueueComponent> {
    pub action: QC::Action,
    pub account: Account,
}

pub trait Sender: Send + Sync + Clone + Resource {
    type Receivers: Receivers;
    type Queue: Queue;
    type UserData: UserData;
    fn new(pool: Pool<<Self::UserData as UserData>::DB>) -> (Self, Self::Receivers);
    fn send(
        &self,
        msg: Msg<Self::Queue>,
        ip: std::net::SocketAddr,
        send_frame: SendFrame,
        ping: Ping,
    ) -> impl Future<Output = eyre::Result<()>> + Send + Sync;
}

pub trait Receivers {
    fn insert(self, app: &mut bevy::prelude::App);
}
