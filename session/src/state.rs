use crate::msg::Message;
use bevy::prelude::Component;

pub trait UserState: Component + Default {
    type Info: Message;
    type Shared: SharedState;
    fn info(index: usize, users: &[Self], shared: &Self::Shared) -> Vec<Self::Info>;
}

pub trait SharedState: Component + Default {
    type Info: Message;
    type User: UserState;
    fn info(index: usize, users: &[Self::User], shared: &Self) -> Self::Info;
}
