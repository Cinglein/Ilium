use crate::{account::Account, data::*, queue::*, time::Ping};
use bevy::ecs::prelude::Resource;
use core::future::Future;
use serde::Serialize;
use session::msg::Msg;
use sqlx::*;
use std::marker::PhantomData;

#[derive(Clone, Debug, bevy::prelude::Resource)]
pub struct Receiver<T>(pub kanal::Receiver<T>);

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
    pub fn send<T>(&self, data: &T)
    where
        T: Serialize,
    {
        if let Ok(bytes) = bitcode::serialize(data) {
            let frame = fastwebsockets::Frame::new(
                true,
                fastwebsockets::OpCode::Binary,
                None,
                bytes.into(),
            );
            let _ = self.0.send(frame);
        }
    }
}

#[derive(Debug)]
pub enum QueueSignal<QC: QueueComponent, U: UserData> {
    Join {
        send_frame: SendFrame,
        ping: Ping,
        user_data: U,
        account: Account,
        _phantom: PhantomData<QC>,
    },
    Accept {
        account: Account,
        _phantom: PhantomData<QC>,
    },
    Leave {
        account: Account,
        _phantom: PhantomData<QC>,
    },
}

pub struct ReconnectSignal<QC: QueueComponent> {
    pub send_frame: SendFrame,
    pub ping: Ping,
    pub account: Account,
    pub _phantom: PhantomData<QC>,
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
    ) -> impl Future<Output = eyre::Result<()>> + Send;
}

pub trait Receivers {
    fn insert(self, app: &mut bevy::prelude::App);
}
