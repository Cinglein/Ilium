use crate::{msg::Message, time::AsStopwatch};
use bevy::{prelude::Component, utils::hashbrown::HashMap};
use std::{borrow::Borrow, fmt::Debug, hash::Hash};

pub trait UserState: Component + Clone + Debug + AsStopwatch {
    type Info: Message;
    type Shared: SharedState;
    fn info<S: AsState<User = Self>>(index: S::Index, state: &S) -> HashMap<u64, Self::Info>;
    fn init(shared: &mut Self::Shared, users: usize) -> Vec<Self>;
}

pub trait SharedState: Component + Clone + Debug + AsStopwatch {
    type Info: Message;
    type User: UserState;
    fn info<S: AsState<Shared = Self>>(index: S::Index, state: &S) -> Self::Info;
    fn init(seed: [u8; 32]) -> Self;
}

pub trait AsState {
    type Shared: SharedState;
    type User: UserState;
    type Index: AsIndex;
    fn user(&self, i: Self::Index) -> Option<impl Borrow<Self::User>>;
    fn users(&self) -> impl Iterator<Item = (Self::Index, impl Borrow<Self::User>)>;
    fn user_mut(&mut self, i: Self::Index) -> Option<impl AsMut<Self::User>>;
    fn shared(&self) -> impl Borrow<Self::Shared>;
    fn shared_mut(&mut self) -> Option<impl AsMut<Self::Shared>>;
    fn indices(&self) -> impl Iterator<Item = Self::Index>;
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
