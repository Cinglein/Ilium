use bevy::prelude::*;
use session::time::*;

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
