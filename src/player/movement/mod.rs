use bevy::prelude::*;
use paste::paste;
use std::time::Duration;

pub mod components;
pub use components::*;

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

#[derive(Component)]
pub struct Landing(Timer);

impl Landing {
    pub fn new() -> Self {
        Landing(Timer::from_seconds(0.15, TimerMode::Once))
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
            .add_system(handle_self_removing_components);
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

pub fn handle_landing(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Landing)>,
) {
    for (entity, mut landing) in &mut query {
        landing.tick(time.delta());
        if landing.finished() {
            commands.entity(entity).remove::<Landing>();
        }
    }
}

macro_rules! SelfRemoving {
    ($time:ident, $commands:ident, for $($t:ty, $q:tt),+) => {
        paste! {
            $(for (entity, mut [<$t:lower>]) in &mut $q {
                [<$t:lower>].tick($time.delta());
                if [<$t:lower>].finished() {
                    $commands.entity(entity).remove::<$t>();
                }
            })*
        }
    };
}

pub fn handle_self_removing_components(
    mut commands: Commands,
    time: Res<Time>,
    mut busy_query: Query<(Entity, &mut Busy)>,
    mut landing_query: Query<(Entity, &mut Landing)>,
    mut coyote_query: Query<(Entity, &mut Coyote)>,
) {
    SelfRemoving!(time, commands, for Busy, busy_query, Landing, landing_query, Coyote, coyote_query);
}
