use crate::{action::Action, msg::Message};
use bevy::{ecs::system::EntityCommands, prelude::*};

pub trait Lobby: Default + Component + TryFrom<Vec<Entity>> {
    fn iter(&self) -> impl Iterator<Item = Entity>;
    fn join(&mut self, entity: Entity) -> bool;
}

pub trait Queue: Message {
    type Action: Action;
    fn register(app: &mut App);
    fn insert(&self, ec: &mut EntityCommands);
}

pub trait QueueComponent: Component {
    const FIXED: bool;
    const LOBBY: usize;
    const PARTY: usize;
    type Queue: Queue;
    type Info: Message
        + for<'a> TryFrom<(
            &'a <Self::Action as Action>::Shared,
            &'a [<Self::Action as Action>::User],
        )>;
    type Lobby: Lobby;
    type Action: Action;
    fn info() -> Self::Info;
    fn lobby() -> Self::Lobby;
}
