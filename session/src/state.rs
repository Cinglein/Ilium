use crate::*;
use hashbrown::HashMap;
use std::{borrow::Borrow, fmt::Debug, hash::Hash};

pub trait UserState: 'static + Send + Sync + Clone + Debug {
    type Info: Message;
    type Shared: SharedState;
    fn info<S: AsState<User = Self>>(index: S::Index, state: &S) -> HashMap<u64, Self::Info>;
    fn init(shared: &mut Self::Shared, users: usize) -> Vec<Self>;
}

pub trait SharedState: 'static + Send + Sync + Clone + Debug {
    type Info: Message;
    type User: UserState;
    fn info<S: AsState<Shared = Self>>(index: S::Index, state: &S) -> Self::Info;
    fn init(seed: [u8; 32]) -> Self;
}

pub trait AsState {
    type Shared: SharedState;
    type User: UserState;
    type Index: Sized + Copy + Clone + Eq + PartialEq + Hash;
    fn index_matches(&self, i: u64, index: Self::Index) -> bool;
    fn user(&self, i: u64) -> Option<impl Borrow<Self::User>>;
    fn users(&self) -> impl Iterator<Item = (u64, impl Borrow<Self::User>)>;
    fn user_mut(&mut self, i: u64) -> Option<impl AsMut<Self::User>>;
    fn shared(&self) -> impl Borrow<Self::Shared>;
    fn shared_mut(&mut self) -> Option<impl AsMut<Self::Shared>>;
    fn indices(&self) -> impl Iterator<Item = u64>;
}
