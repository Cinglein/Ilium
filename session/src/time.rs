use bevy::{prelude::Component, utils::Duration};

pub trait AsStopwatch: Component {
    fn elapsed_secs(&self) -> f32;
    fn pause(&mut self);
    fn unpause(&mut self);
    fn is_paused(&self) -> bool;
    fn reset(&mut self);
    fn tick(&mut self, delta: Duration);
}

pub trait AsTimer: AsStopwatch + Component {
    fn remaining_secs(&self) -> f32;
    fn finished(&self) -> bool;
}
