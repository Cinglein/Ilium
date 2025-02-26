use crate::{account::AccountMap, queries::*, send::*};
use bevy::prelude::*;
use session::{action::Action, info::Info, queue::*, state::*};
use std::borrow::Borrow;

pub type ActionStateInfo<'a, QC: QueueComponent> = Info<
    <ActionState<'a, QC> as AsState>::User,
    <ActionState<'a, QC> as AsState>::Shared,
    <ActionState<'a, QC> as AsState>::Index,
>;

pub enum ActionState<'a, QC: QueueComponent> {
    Mutable {
        users: &'a mut InSession<'a, 'a, QC>,
        shared: Mut<'a, <QC::Action as Action>::Shared>,
        lobby: Mut<'a, QC::Lobby>,
    },
    Immutable {
        users: &'a InSession<'a, 'a, QC>,
        shared: &'a <QC::Action as Action>::Shared,
        lobby: &'a QC::Lobby,
    },
}

impl<'a, QC: QueueComponent> ActionState<'a, QC> {
    fn info(
        session_id: Entity,
        user_id: Entity,
        sessions: &'a Sessions<'a, 'a, QC>,
        users: &'a InSession<'a, 'a, QC>,
    ) -> Option<ActionStateInfo<'a, QC>> {
        let session = sessions.get(session_id).ok()?;
        let shared = session.state;
        let lobby = session.lobby;
        Some(QC::info(
            user_id,
            &Self::Immutable {
                users,
                shared,
                lobby,
            },
        ))
    }
    fn update(
        session_id: Entity,
        sessions: &'a mut Sessions<'a, 'a, QC>,
        users: &'a mut InSession<'a, 'a, QC>,
    ) -> Option<()> {
        let session = sessions.get_mut(session_id).ok()?;
        let shared = session.state;
        let lobby = session.lobby;
        QC::Action::update(Self::Mutable {
            users,
            shared,
            lobby,
        });
        Some(())
    }
    fn resolve(
        msg: ActionSignal<QC>,
        accounts: impl AsRef<AccountMap>,
        sessions: &'a mut Sessions<'a, 'a, QC>,
        users: &'a mut InSession<'a, 'a, QC>,
    ) -> Option<()> {
        let user_id = accounts.as_ref().get(&msg.account)?;
        let session_id = users.get(user_id).ok()?.session.0;
        let session = sessions.get_mut(session_id).ok()?;
        let shared = session.state;
        let lobby = session.lobby;
        msg.action.resolve(
            user_id,
            Self::Mutable {
                users,
                shared,
                lobby,
            },
        );
        Some(())
    }
}

impl<'a, QC: QueueComponent> AsState for ActionState<'a, QC> {
    type Shared = <QC::Action as Action>::Shared;
    type User = <QC::Action as Action>::User;
    type Index = Entity;
    fn user(&self, i: Self::Index) -> Option<impl Borrow<Self::User>> {
        match self {
            Self::Mutable { users, .. } => users.get(i).ok().map(|u| u.state),
            Self::Immutable { users, .. } => users.get(i).ok().map(|u| u.state),
        }
    }
    fn users(&self) -> impl Iterator<Item = (Self::Index, impl Borrow<Self::User>)> {
        self.indices().filter_map(|i| Some((i, self.user(i)?)))
    }
    fn user_mut(&mut self, i: Self::Index) -> Option<impl AsMut<Self::User>> {
        let Self::Mutable { users, .. } = self else {
            return None;
        };
        users.get_mut(i).ok().map(|u| u.state)
    }
    fn shared(&self) -> impl Borrow<Self::Shared> {
        match self {
            Self::Mutable { shared, .. } => shared.as_ref(),
            Self::Immutable { shared, .. } => shared,
        }
    }
    fn shared_mut(&mut self) -> Option<impl AsMut<Self::Shared>> {
        let Self::Mutable { shared, .. } = self else {
            return None;
        };
        Some(shared)
    }
    fn indices(&self) -> impl Iterator<Item = Self::Index> {
        match self {
            Self::Mutable { lobby, .. } => lobby.entities(),
            Self::Immutable { lobby, .. } => lobby.entities(),
        }
    }
}
pub fn update_client<QC: QueueComponent>(sessions: Sessions<QC>, users: InSession<QC>) {
    let s: Vec<_> = sessions
        .iter()
        .map(|s| (s.entity, s.lobby.clone()))
        .collect();
    for (session, lobby) in s.into_iter() {
        for user in lobby.entities() {
            if let Some(info) = ActionState::info(session, user, &sessions, &users)
                && let Ok(user) = users.get(user)
            {
                user.send_frame.send(&info);
            }
        }
    }
}

pub fn process_actions<QC: QueueComponent>(
    accounts: Res<AccountMap>,
    actions: ResMut<Receiver<ActionSignal<QC>>>,
    mut sessions: Sessions<QC>,
    mut users: InSession<QC>,
) {
    while let Ok(Some(msg)) = actions.try_recv() {
        ActionState::resolve(
            msg,
            &accounts,
            &mut sessions.reborrow(),
            &mut users.reborrow(),
        );
    }
    let s: Vec<_> = sessions.iter().map(|s| s.entity).collect();
    for session in s.into_iter() {
        ActionState::update(session, &mut sessions.reborrow(), &mut users.reborrow());
    }
}
