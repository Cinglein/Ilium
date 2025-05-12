use bevy::prelude::*;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Resource)]
pub struct AccountMap(pub bevy::utils::hashbrown::HashMap<Account, bevy::prelude::Entity>);

impl AccountMap {
    pub fn get(&self, account: &Account) -> Option<bevy::prelude::Entity> {
        self.0.get(account).copied()
    }
}

#[derive(
    Eq, PartialEq, Hash, Clone, Copy, Debug, Serialize, Deserialize, Decode, Encode, Component,
)]
pub enum Account {
    Guest { ip: std::net::SocketAddr },
}
