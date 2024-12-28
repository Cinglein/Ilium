use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, bevy::prelude::Resource)]
pub struct AccountMap(pub bevy::utils::hashbrown::HashMap<Account, bevy::prelude::Entity>);

#[derive(Eq, PartialEq, Hash, Clone, Copy, Debug, Serialize, Deserialize, Component)]
pub enum Account {
    Guest { ip: std::net::SocketAddr },
}
