use crate::{
    account::AccountMap,
    data::UserData,
    queries::*,
    send::{QueueSignal, Receiver, ReconnectSignal},
    update::ActionStateInfo,
};
use bevy::{prelude::*, utils::hashbrown::HashSet};
use rand::{rngs::OsRng, TryRngCore};
use session::{action::Action, info::StateInfo, queue::*, state::*};

#[derive(Component)]
pub struct Accepted;

pub fn process_queue<QC: QueueComponent, U: UserData>(
    mut commands: Commands,
    receiver: ResMut<Receiver<QueueSignal<QC, U>>>,
    accounts: ResMut<AccountMap>,
    in_queue: InQueue<QC, U>,
    in_lobby: InLobby<QC>,
) {
    let accounts = &mut accounts.into_inner().0;
    let receiver = &receiver;
    while let Ok(Some(msg)) = receiver.try_recv() {
        match msg {
            QueueSignal::Join {
                send_frame,
                ping,
                user_data,
                account,
                ..
            } => {
                if !accounts.contains_key(&account) {
                    send_frame.send(&StateInfo::Queue::<ActionStateInfo<QC>>);
                    let mut ec = commands.spawn((account, ping, user_data, send_frame));
                    ec.insert(QC::default());
                    let entity = ec.id();
                    accounts.insert(account, entity);
                }
            }
            QueueSignal::Accept { account, .. } => {
                if let Some(player) = accounts.get(&account).and_then(|e| in_lobby.get(*e).ok()) {
                    if let Some(mut ec) = commands.get_entity(player.entity) {
                        ec.insert(Accepted);
                    }
                }
            }
            QueueSignal::Leave { account, .. } => {
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
    receiver: ResMut<Receiver<ReconnectSignal<QC>>>,
    accounts: ResMut<AccountMap>,
    mut in_session: InSession<QC>,
) {
    let accounts = &mut accounts.into_inner().0;
    let receiver = &receiver;
    while let Ok(Some(ReconnectSignal {
        send_frame,
        ping,
        account,
        ..
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

/// Given a queue, matchmake users into a lobby
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
            let mut seed = [0u8; 32];
            OsRng.try_fill_bytes(&mut seed).expect("OSRng Error");
            let shared_state = <<QC::Action as Action>::Shared as SharedState>::init(seed);
            let session_id = EntityId(commands.spawn((shared_state, lobby.clone())).id());
            for entity in lobby.entities() {
                if let Ok(user) = in_queue.get(entity) {
                    commands.entity(entity).insert(session_id);
                    user.send_frame
                        .send(&StateInfo::<ActionStateInfo<QC>>::Lobby);
                }
            }
        }
    }
}

///Initialize a new session from accepted lobbies
pub fn init_session<QC: QueueComponent>(
    mut commands: Commands,
    accepted: InLobbyAccepted<QC>,
    mut sessions: SessionsPending<QC>,
) where
    QC::Action: Action<Shared = <<QC::Action as Action>::User as UserState>::Shared>,
{
    for mut session in sessions.iter_mut() {
        if session
            .lobby
            .entities()
            .fold(true, |b, e| b && accepted.contains(e))
        {
            let user_states = <<QC::Action as Action>::User as UserState>::init(
                session.state.as_mut(),
                session.lobby.len(),
            );
            session
                .lobby
                .entities()
                .zip(user_states.into_iter())
                .for_each(|(e, state)| {
                    commands.entity(e).insert(state);
                });
            commands.entity(session.entity).insert(Accepted);
        }
    }
}
