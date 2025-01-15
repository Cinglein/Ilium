use crate::{action::Action, msg::Message};
use bevy::{app::Update, prelude::*};

pub trait IliumApp {
    fn register<M>(&mut self, t: impl IntoSystemConfigs<M>);
}

impl IliumApp for App {
    fn register<M>(&mut self, system: impl IntoSystemConfigs<M>) {
        self.add_systems(Update, system);
    }
}

pub trait Lobby: Component + for<'a> TryFrom<&'a [Entity]> {
    fn entities(&self) -> impl Iterator<Item = Entity>;
}

pub trait Queue: Message {
    type Action: Action;
    fn register<A: IliumApp>(app: &mut A);
    fn insert(&self, ec: &mut bevy::ecs::system::EntityCommands);
}

pub trait QueueComponent: Component {
    type Queue: Queue;
    type Info: Message;
    type Lobby: Lobby;
    type Action: Action;
    fn info(
        index: usize,
        users: &[<Self::Action as Action>::User],
        shared: &<Self::Action as Action>::Shared,
    ) -> Self::Info;
}
