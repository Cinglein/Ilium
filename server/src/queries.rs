use crate::{account::Account, data::UserData, matchmaking::Accepted, send::SendFrame, time::Ping};
use bevy::{ecs::query::QueryData, prelude::*};
use session::{action::Action, queue::QueueComponent};

pub type InQueue<'a, 'b, Q, U> = Query<'a, 'b, AccountQuery<U>, (With<Q>, Without<EntityId>)>;
pub type InLobby<'a, 'b, QC> = Query<
    'a,
    'b,
    LobbyQuery,
    (
        Without<<<QC as QueueComponent>::Action as Action>::User>,
        Without<Accepted>,
    ),
>;
pub type InLobbyAccepted<'a, 'b, QC> = Query<
    'a,
    'b,
    LobbyQuery,
    (
        Without<<<QC as QueueComponent>::Action as Action>::User>,
        With<Accepted>,
    ),
>;
pub type InSession<'a, 'b, QC> = Query<'a, 'b, UserQuery<QC>>;
pub type Sessions<'a, 'b, QC> = Query<'a, 'b, SessionQuery<QC>>;

#[derive(Clone, Copy, Debug, Component)]
pub struct EntityId(pub Entity);

#[derive(QueryData)]
#[query_data(mutable)]
pub struct AccountQuery<U: UserData> {
    pub entity: Entity,
    pub account: &'static mut Account,
    pub user_data: &'static mut U,
    pub send_frame: &'static mut SendFrame,
    pub ping: &'static Ping,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct LobbyQuery {
    pub entity: Entity,
    pub session: &'static mut EntityId,
    pub account: &'static mut Account,
    pub send_frame: &'static mut SendFrame,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct UserQuery<QC: QueueComponent> {
    pub entity: Entity,
    pub session: &'static mut EntityId,
    pub account: &'static mut Account,
    pub state: &'static mut <<QC as QueueComponent>::Action as Action>::User,
    pub send_frame: &'static mut SendFrame,
    pub ping: &'static mut Ping,
}

#[derive(QueryData)]
#[query_data(mutable)]
pub struct SessionQuery<QC: QueueComponent> {
    pub entity: Entity,
    pub lobby: &'static mut <QC as QueueComponent>::Lobby,
    pub state: &'static mut <<QC as QueueComponent>::Action as Action>::Shared,
}
