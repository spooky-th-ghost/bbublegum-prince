use bevy::prelude::*;
use paste::paste;
use std::time::Duration;

pub mod components;
pub use components::*;

pub mod locomotion;
pub use locomotion::*;

pub mod jumping;
pub use jumping::*;

pub struct PlayerMovementPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum PlayerPhysicsSet {
    SetForces,
    ApplyForces,
    Cleanup,
}

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(apply_momentum.in_set(PlayerPhysicsSet::ApplyForces))
            .add_system(handle_self_removing_components.in_set(PlayerPhysicsSet::Cleanup))
            .add_systems(
                (
                    set_player_direction,
                    handle_player_speed,
                    rotate_to_direction,
                )
                    .chain()
                    .in_set(PlayerPhysicsSet::SetForces),
            )
            .add_systems((buffer_jump, handle_jumping).chain())
            .add_systems(
                (
                    handle_grounded,
                    detect_walls,
                    detect_ledges,
                    handle_wall_jumping,
                    aerial_drift,
                    handle_ledge_grab,
                    reset_jumps_after_landing,
                    add_friction_when_landing,
                    handle_jump_buffer,
                    handle_long_jump,
                )
                    .in_set(PlayerPhysicsSet::SetForces),
            );
    }
}

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
