use bevy::prelude::*;

#[derive(Clone, Debug, Component)]
pub struct Ping(pub tokio::sync::watch::Receiver<Option<u128>>);

impl Ping {
    pub fn get(&self) -> Option<u128> {
        *self.0.borrow()
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct Stopwatch {
    pub stopwatch: bevy::time::Stopwatch,
}

impl Stopwatch {
    pub fn elapsed_secs(&self) -> f32 {
        self.stopwatch.elapsed_secs()
    }
    pub fn pause(&mut self) {
        self.stopwatch.pause();
    }
    pub fn unpause(&mut self) {
        self.stopwatch.unpause();
    }
    pub fn is_paused(&self) -> bool {
        self.stopwatch.is_paused()
    }
    pub fn reset(&mut self) {
        self.stopwatch.reset();
    }
}

pub fn tick(time: Res<Time>, mut stopwatches: Query<&mut Stopwatch>) {
    for mut sw in stopwatches.iter_mut() {
        sw.stopwatch.tick(time.delta());
    }
}
