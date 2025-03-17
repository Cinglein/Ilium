use bevy::{prelude::Component, utils::Duration};

pub trait AsStopwatch: Component {
    fn pause(&mut self);
    fn unpause(&mut self);
    fn reset(&mut self);
    fn tick(&mut self, delta: Duration);
}
