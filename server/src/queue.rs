use bevy::{ecs::component::*, prelude::*};
use core::hash::Hash;
use session::*;

pub trait Lobby: 'static + Clone + Send + Sync + for<'a> TryFrom<&'a [Entity]> {
    fn len(&self) -> usize;
    fn entities(&self) -> impl Iterator<Item = Entity>;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait Queue: AsQueue + Message {
    fn insert(&self, ec: &mut bevy::ecs::system::EntityCommands);
}

pub trait QueueComponent: Component + Default + std::fmt::Debug {
    type Queue: Queue;
    type Lobby: Lobby + Component<Mutability = Mutable>;
    type Action: Action;
    type Shared: SharedState + Component<Mutability = Mutable>;
    type User: UserState + Component<Mutability = Mutable>;
    fn info<S: AsState<Shared = Self::Shared, User = Self::User>>(
        index: S::Index,
        state: &S,
    ) -> Info<S::User, S::Shared>;
}

pub trait AsIndex: Sized + Copy + Clone + Eq + PartialEq + Hash {
    fn from_index(i: u64) -> Option<Self>;
    fn to_index(&self) -> u64;
}

impl AsIndex for bevy::prelude::Entity {
    fn from_index(i: u64) -> Option<Self> {
        Self::try_from_bits(i).ok()
    }
    fn to_index(&self) -> u64 {
        self.to_bits()
    }
}
