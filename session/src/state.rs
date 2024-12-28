use crate::msg::Message;
use bevy::prelude::Component;

pub trait State: Component + Default {
    type Info: Message;
    fn info(&self) -> Self::Info;
}
