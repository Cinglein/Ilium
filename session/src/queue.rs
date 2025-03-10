use crate::{action::Action, info::Info, msg::Message, state::AsState};
use bevy::prelude::*;

pub trait Lobby: Clone + Component + for<'a> TryFrom<&'a [Entity]> {
    fn len(&self) -> usize;
    fn entities(&self) -> impl Iterator<Item = Entity>;
}

pub trait Queue: Message {
    type Action: Action;
    fn insert(&self, ec: &mut bevy::ecs::system::EntityCommands);
}

pub trait QueueComponent: Component + Default {
    type Queue: Queue;
    type Lobby: Lobby;
    type Action: Action;
    fn info<
        S: AsState<Shared = <Self::Action as Action>::Shared, User = <Self::Action as Action>::User>,
    >(
        index: S::Index,
        state: &S,
    ) -> Info<S::User, S::Shared, S::Index>;
}
