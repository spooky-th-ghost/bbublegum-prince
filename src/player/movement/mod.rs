use std::time::Duration;

use bevy::prelude::*;

pub mod locomotion;
pub use locomotion::*;

pub mod jumping;
pub use jumping::*;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Busy(Timer);

impl Busy {
    pub fn new(seconds: f32) -> Self {
        Busy(Timer::from_seconds(seconds, TimerMode::Once))
    }

    pub fn tick(&mut self, duration: Duration) {
        self.0.tick(duration);
    }

    pub fn finished(&self) -> bool {
        self.0.finished()
    }
}

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerLocomotionPlugin)
            .add_plugin(PlayerJumpingPlugin)
            .add_system(handle_busy);
    }
}

pub fn handle_busy(mut commands: Commands, time: Res<Time>, mut query: Query<(Entity, &mut Busy)>) {
    for (entity, mut busy) in &mut query {
        busy.tick(time.delta());
        if busy.finished() {
            commands.entity(entity).remove::<Busy>();
        }
    }
}
