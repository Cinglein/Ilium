use crate::{account::Account, auth::auth, data::*, time::Ping};
use bevy::ecs::system::Resource;
use rkyv::{
    api::high::HighSerializer, rancor::Source, ser::allocator::ArenaHandle, util::*, Serialize,
};
use session::{
    msg::{Msg, MsgType},
    queue::Queue,
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

pub enum QueueSignal<Q: Queue, U: UserData> {
    Join {
        queue: Q,
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

pub struct Receivers<Q: Queue, U: UserData> {
    pub queue: Receiver<QueueSignal<Q, U>>,
    pub reconnect: Receiver<ReconnectSignal>,
    pub action: Receiver<<Q as Queue>::Action>,
}

impl<Q: Queue, U: UserData> Receivers<Q, U> {
    pub fn insert(self, app: &mut bevy::prelude::App) {
        let Receivers {
            queue,
            reconnect,
            action,
        } = self;
        app.insert_resource(queue);
        app.insert_resource(reconnect);
        app.insert_resource(action);
    }
}

#[derive(Debug, Resource)]
pub struct Sender<Q: Queue, U: UserData> {
    pub pool: Pool<U::DB>,
    pub queue: kanal::Sender<QueueSignal<Q, U>>,
    pub reconnect: kanal::Sender<ReconnectSignal>,
    pub action: kanal::Sender<<Q as Queue>::Action>,
}

impl<Q: Queue, U: UserData> Clone for Sender<Q, U> {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
            queue: self.queue.clone(),
            reconnect: self.reconnect.clone(),
            action: self.action.clone(),
        }
    }
}

impl<Q: Queue, U: UserData> Sender<Q, U> {
    pub fn new(pool: Pool<U::DB>) -> (Self, Receivers<Q, U>) {
        let (queue, receive_queue) = kanal::unbounded();
        let (reconnect, receive_reconnect) = kanal::unbounded();
        let (action, receive_action) = kanal::unbounded();
        let sender = Self {
            pool,
            queue,
            reconnect,
            action,
        };
        let receivers = Receivers {
            queue: Receiver(receive_queue),
            reconnect: Receiver(receive_reconnect),
            action: Receiver(receive_action),
        };
        (sender, receivers)
    }
    pub async fn send(
        &self,
        msg: Msg<Q>,
        ip: std::net::SocketAddr,
        send_frame: SendFrame,
        ping: Ping,
    ) -> eyre::Result<()> {
        let Msg { token, msg_type } = msg;
        let account = auth(token, ip);
        match msg_type {
            MsgType::Join { queue } => {
                let user_data = U::query(&self.pool, &account).await?;
                self.queue.send(QueueSignal::Join {
                    account,
                    queue,
                    send_frame,
                    user_data,
                    ping,
                })
            }
            MsgType::Reconnect => self.reconnect.send(ReconnectSignal {
                account,
                ping,
                send_frame,
            }),
            MsgType::Accept => self.queue.send(QueueSignal::Accept { account }),
            MsgType::Leave => self.queue.send(QueueSignal::Leave { account }),
            MsgType::Action(msg) => self.action.send(msg),
        }?;
        Ok(())
    }
}
