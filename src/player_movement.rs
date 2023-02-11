use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{Direction, MainCamera, OutsideForce};

#[derive(Component)]
pub struct Player;

#[derive(Resource)]
pub struct PlayerSpeed {
    accel_timer: Timer,
    base_speed: f32,
    current_speed: f32,
    acceleration: f32,
    top_speed: f32,
    previous_direction: Vec3,
}

impl PlayerSpeed {
    pub fn reset(&mut self) {
        self.current_speed = self.base_speed;
        self.accel_timer.reset();
    }

    pub fn accelerate(&mut self, time: Res<Time>) {
        self.accel_timer.tick(time.delta());

        if self.accel_timer.finished() {
            if self.top_speed - self.current_speed < 0.2 {
                self.current_speed = self.current_speed
                    + (time.delta_seconds() * self.acceleration)
                        * (self.top_speed - self.current_speed);
            } else {
                self.current_speed = self.top_speed;
            }
        }
    }

    pub fn current(&self) -> f32 {
        self.current_speed
    }
}

impl Default for PlayerSpeed {
    fn default() -> Self {
        PlayerSpeed {
            accel_timer: Timer::from_seconds(1.5, TimerMode::Once),
            base_speed: 7.5,
            current_speed: 7.5,
            acceleration: 0.25,
            top_speed: 15.0,
            previous_direction: Vec3::ZERO,
        }
    }
}

#[derive(PartialEq, Reflect)]
pub enum GroundedState {
    Grounded,
    Coyote,
    Rising,
    Falling,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Grounded {
    coyote_timer: Timer,
    state: GroundedState,
}

impl Grounded {
    pub fn is_airborne(&self) -> bool {
        match self.state {
            GroundedState::Grounded => false,
            _ => true,
        }
    }

    pub fn is_grounded(&self) -> bool {
        match self.state {
            GroundedState::Grounded => true,
            _ => false,
        }
    }

    pub fn can_jump(&self) -> bool {
        match self.state {
            GroundedState::Grounded | GroundedState::Coyote => true,
            _ => false,
        }
    }

    pub fn walk_off(&mut self) {
        self.state = GroundedState::Falling;
    }

    pub fn jump(&mut self) {
        self.coyote_timer.reset();
        self.state = GroundedState::Rising;
    }

    pub fn land(&mut self) {
        self.state = GroundedState::Grounded;
    }

    pub fn coyote_tick(&mut self, time: Res<Time>) {
        if self.state == GroundedState::Coyote {
            self.coyote_timer.tick(time.delta());
            if self.coyote_timer.just_finished() {
                self.state = GroundedState::Falling
            }
        }
    }
}

impl Default for Grounded {
    fn default() -> Self {
        Grounded {
            state: GroundedState::Grounded,
            coyote_timer: Timer::from_seconds(0.2, TimerMode::Once),
        }
    }
}
const PLAYER_ROTATION_SPEED: f32 = 10.0;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_player_acceleration)
            .add_system(set_player_direction)
            .add_system(handle_grounded)
            .add_system(handle_jumping)
            .add_system(rotate_to_direction.after(set_player_direction))
            .add_system(move_player_from_rotation.after(rotate_to_direction));
    }
}

pub fn set_player_direction(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Direction, With<Player>>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    let camera_transform = camera_query.single();
    let mut player_direction = player_query.single_mut();

    let mut x = 0.0;
    let mut z = 0.0;

    let mut forward = camera_transform.forward();
    forward.y = 0.0;
    forward = forward.normalize();

    let mut right = camera_transform.right();
    right.y = 0.0;
    right = right.normalize();

    if keyboard.pressed(KeyCode::W) {
        z += 1.0;
    }

    if keyboard.pressed(KeyCode::S) {
        z -= 1.0;
    }

    if keyboard.pressed(KeyCode::D) {
        x += 1.0;
    }

    if keyboard.pressed(KeyCode::A) {
        x -= 1.0;
    }

    let right_vec: Vec3 = x * right;
    let forward_vec: Vec3 = z * forward;

    player_direction.0 = (right_vec + forward_vec).normalize_or_zero();
}

pub fn rotate_to_direction(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Direction), With<Player>>,
    mut rotation_target: Local<Transform>,
) {
    let (mut transform, direction) = query.single_mut();

    rotation_target.translation = transform.translation;
    let cur_position = rotation_target.translation;
    let flat_velo_direction = Vec3::new(direction.0.x, 0.0, direction.0.z).normalize_or_zero();
    if flat_velo_direction != Vec3::ZERO {
        rotation_target.look_at(cur_position + flat_velo_direction, Vec3::Y);
        transform.rotation = transform.rotation.slerp(
            rotation_target.rotation,
            time.delta_seconds() * PLAYER_ROTATION_SPEED,
        );
    }
}

pub fn handle_player_acceleration(
    time: Res<Time>,
    mut player_speed: ResMut<PlayerSpeed>,
    query: Query<&Direction, With<Player>>,
) {
    let direction = query.single();

    if direction.0 != Vec3::ZERO {
        if direction.0 == player_speed.previous_direction {
            player_speed.accelerate(time)
        }
    } else {
        player_speed.reset();
    }

    player_speed.previous_direction = direction.0;
}

pub fn move_player_from_rotation(
    player_speed: Res<PlayerSpeed>,
    mut query: Query<(&mut Velocity, &Transform, &Direction, Option<&OutsideForce>)>,
) {
    let (mut velocity, transform, direction, has_force) = query.single_mut();

    let mut speed_to_apply = Vec3::ZERO;
    let mut should_change_velocity: bool = false;

    if let Some(outside_force) = has_force {
        should_change_velocity = true;
        speed_to_apply.x += outside_force.0.x;
        speed_to_apply.z += outside_force.0.z;
    }

    if direction.is_moving() {
        should_change_velocity = true;
        let forward = transform.forward();
        speed_to_apply += forward * player_speed.current();
    }

    if should_change_velocity {
        velocity.linvel.x = speed_to_apply.x;
        velocity.linvel.z = speed_to_apply.z;
    }
}

pub fn handle_grounded(
    mut query: Query<(&Transform, &mut Grounded), With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    let (transform, mut grounded) = query.single_mut();

    let is_grounded = grounded.is_grounded();
    let ray_pos = transform.translation;
    let ray_dir = Vec3::Y * -1.0;
    let max_distance = 1.2;
    let solid = true;
    let filter = QueryFilter::exclude_dynamic().exclude_sensors();

    if let Some((_entity, _intersection)) =
        rapier_context.cast_ray(ray_pos, ray_dir, max_distance, solid, filter)
    {
        if !is_grounded {
            println!("Found some ground!");
            grounded.land();
        }
    } else {
        if is_grounded {
            grounded.walk_off();
        }
    }
}

pub fn handle_jumping(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Grounded), With<Player>>,
) {
    let (mut velocity, mut grounded) = query.single_mut();

    if grounded.can_jump() {
        if input.pressed(KeyCode::Space) {
            grounded.jump();
            velocity.linvel.y = 10.0;
        }
    }
}
