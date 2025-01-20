use bevy::prelude::*;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Default, Debug, Resource)]
pub struct AccountMap(pub bevy::utils::hashbrown::HashMap<Account, bevy::prelude::Entity>);

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, Archive, Serialize, Deserialize, Component)]
pub enum Account {
    Guest { ip: std::net::SocketAddr },
}
