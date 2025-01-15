use crate::{
    account::AccountMap,
    data::UserData,
    queries::*,
    send::{QueueSignal, Receiver, ReconnectSignal},
};
use bevy::{prelude::*, utils::hashbrown::HashSet};
use session::{action::Action, info::StateInfo, queue::*};

#[derive(Component)]
pub struct Accepted;

pub fn process_queue<QC: QueueComponent, U: UserData>(
    mut commands: Commands,
    receiver: ResMut<Receiver<QueueSignal<QC::Queue, U>>>,
    accounts: ResMut<AccountMap>,
    in_queue: InQueue<QC, U>,
    in_lobby: InLobby<QC>,
) {
    let accounts = &mut accounts.into_inner().0;
    let receiver = &receiver;
    while let Ok(Some(msg)) = receiver.try_recv() {
        match msg {
            QueueSignal::Join {
                queue,
                send_frame,
                ping,
                user_data,
                account,
            } => {
                if !accounts.contains_key(&account) {
                    send_frame.send(&StateInfo::Queue::<QC::Info>);
                    let mut ec = commands.spawn((account, ping, user_data, send_frame));
                    queue.insert(&mut ec);
                    let entity = ec.id();
                    accounts.insert(account, entity);
                }
            }
            QueueSignal::Accept { account } => {
                if let Some(player) = accounts.get(&account).and_then(|e| in_lobby.get(*e).ok()) {
                    if let Some(mut ec) = commands.get_entity(player.entity) {
                        ec.insert(Accepted);
                    }
                }
            }
            QueueSignal::Leave { account } => {
                if let Some(entity) = accounts.get(&account) {
                    if in_queue.contains(*entity) || in_lobby.contains(*entity) {
                        if let Some(mut ec) = commands.get_entity(*entity) {
                            ec.despawn();
                            accounts.remove(&account);
                        }
                    }
                }
            }
        }
    }
}

pub fn reconnect<QC: QueueComponent>(
    receiver: ResMut<Receiver<ReconnectSignal>>,
    accounts: ResMut<AccountMap>,
    mut in_session: InSession<QC>,
) {
    let accounts = &mut accounts.into_inner().0;
    let receiver = &receiver;
    while let Ok(Some(ReconnectSignal {
        send_frame,
        ping,
        account,
    })) = receiver.try_recv()
    {
        if let Some(entity) = accounts.get(&account).copied() {
            if let Ok(mut ec) = in_session.get_mut(entity) {
                *ec.send_frame = send_frame;
                *ec.ping = ping;
            }
        }
    }
}

/// Given a queue, matchmake users and initiate a new session
pub fn matchmake<QC: QueueComponent, U: UserData>(
    mut commands: Commands,
    in_queue: InQueue<QC, U>,
) {
    let mut users: Vec<_> = in_queue
        .iter()
        .map(|user| (user.entity, user.user_data.clone()))
        .collect();
    users.sort_unstable_by_key(|(_, u)| u.matchmake_priority());
    let mut taken = HashSet::new();
    for (_entity, user) in users.iter() {
        let valid: Vec<_> = users
            .iter()
            .filter_map(|(e, u)| (!taken.contains(e) && user.matchmake_valid(u)).then_some(*e))
            .collect();
        let lobby = QC::Lobby::try_from(&valid).ok();
        if let Some(lobby) = lobby {
            valid.into_iter().for_each(|e| {
                taken.insert(e);
            });
            for entity in lobby.entities() {
                if let Ok(user) = in_queue.get(entity) {
                    let shared_state = <QC::Action as Action>::Shared::default();
                    let session_id = EntityId(commands.spawn(shared_state).id());
                    commands.entity(entity).insert(session_id);
                    user.send_frame.send(&StateInfo::<QC::Info>::Lobby);
                }
            }
        }
    }
}
