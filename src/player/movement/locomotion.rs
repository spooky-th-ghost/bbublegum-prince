use crate::{Grounded, MainCamera, Movement, OutsideForce, Player};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

const PLAYER_ROTATION_SPEED: f32 = 10.0;

#[derive(Resource)]
pub struct PlayerSpeed {
    accel_timer: Timer,
    base_speed: f32,
    current_speed: f32,
    top_speed: f32,
    min_speed: f32,
    acceleration: f32,
}

impl PlayerSpeed {
    pub fn reset(&mut self) {
        self.current_speed = self.base_speed;
        self.accel_timer.reset();
    }

    pub fn accelerate(&mut self, time: Res<Time>) {
        self.accel_timer.tick(time.delta());
        if self.accel_timer.finished() {
            if self.current_speed + 0.3 <= self.top_speed {
                self.current_speed = self.current_speed
                    + (self.top_speed - self.current_speed)
                        * (time.delta_seconds() * self.acceleration);
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
            top_speed: 15.0,
            min_speed: -20.0,
            acceleration: 2.0,
        }
    }
}

pub struct PlayerLocomotionPlugin;

impl Plugin for PlayerLocomotionPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_player_direction)
            .add_system(handle_player_acceleration.after(set_player_direction))
            .add_system(rotate_to_direction.after(set_player_direction))
            .add_system(move_player_from_rotation.after(rotate_to_direction));
    }
}

pub fn set_player_direction(
    keyboard: Res<Input<KeyCode>>,
    mut player_query: Query<&mut Movement, With<Player>>,
    camera_query: Query<&Transform, With<MainCamera>>,
) {
    let camera_transform = camera_query.single();
    let mut player_direction = player_query.single_mut();

    player_direction.0 = get_direction_in_camera_space(camera_transform, keyboard);
}

pub fn get_direction_in_camera_space(
    camera_transform: &Transform,
    keyboard: Res<Input<KeyCode>>,
) -> Vec3 {
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

    (right_vec + forward_vec).normalize_or_zero()
}

pub fn rotate_to_direction(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Movement, &Grounded), With<Player>>,
    mut rotation_target: Local<Transform>,
) {
    let (mut transform, direction, grounded) = query.single_mut();

    rotation_target.translation = transform.translation;
    let cur_position = rotation_target.translation;
    let flat_velo_direction = Vec3::new(direction.0.x, 0.0, direction.0.z).normalize_or_zero();
    if flat_velo_direction != Vec3::ZERO && grounded.is_grounded() {
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
    query: Query<&Movement, With<Player>>,
) {
    let movement = query.single();

    if movement.is_moving() {
        player_speed.accelerate(time);
    } else {
        player_speed.reset();
    }
}

pub fn move_player_from_rotation(
    player_speed: Res<PlayerSpeed>,
    mut query: Query<(&mut Velocity, &Transform, &Movement, Option<&OutsideForce>)>,
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
