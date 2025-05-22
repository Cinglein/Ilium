use bevy::{ecs::component::*, prelude::*};
use core::time::Duration;

#[derive(Clone, Debug, Component)]
pub struct Ping(pub tokio::sync::watch::Receiver<Option<u128>>);

impl Ping {
    pub fn get(&self) -> Option<u128> {
        *self.0.borrow()
    }
}

pub fn tick<T: AsStopwatch>(time: Res<Time>, mut stopwatches: Query<&mut T>) {
    for mut sw in stopwatches.iter_mut() {
        sw.tick(time.delta());
    }
}

pub trait AsStopwatch: Component<Mutability = Mutable> {
    fn pause(&mut self);
    fn unpause(&mut self);
    fn reset(&mut self);
    fn tick(&mut self, delta: Duration);
}

impl<T: Component<Mutability = Mutable>> AsStopwatch for T {
    fn pause(&mut self) {}
    fn unpause(&mut self) {}
    fn reset(&mut self) {}
    fn tick(&mut self, _delta: Duration) {}
}
