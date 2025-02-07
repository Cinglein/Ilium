use crate::{account::AccountMap, queries::*, send::*};
use bevy::prelude::*;
use session::queue::QueueComponent;

pub fn update_client<QC: QueueComponent>(sessions: Sessions<QC>, users: InSession<QC>) {
    for user in users.into_iter() {
        todo!()
    }
}

pub fn process_actions<QC: QueueComponent>(
    mut commands: Commands,
    accounts: ResMut<AccountMap>,
    actions: ResMut<Receiver<ActionSignal<QC>>>,
    mut sessions: Sessions<QC>,
    mut users: InSession<QC>,
) {
    let accounts = &mut accounts.into_inner().0;
    while let Ok(Some(msg)) = actions.try_recv() {
        if let Some(user) = accounts
            .get(&msg.account)
            .and_then(|e| users.get_mut(*e).ok())
            && let Ok(session) = sessions.get_mut(user.session.0)
        {
            let lobby = session.lobby;
        }
    }
}
