use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{MainCamera, Movement, OutsideForce, Wall};

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

pub enum JumpStage {
    Single,
    Double,
    Triple,
}

#[derive(Component)]
pub struct Jump {
    pub input_timer: Timer,
    pub jump_stage: JumpStage,
    pub jump_buffered: bool,
}

impl Jump {
    fn reset_input(&mut self) {
        self.jump_buffered = false;
        self.input_timer.reset();
    }

    pub fn update(&mut self, time: Res<Time>) {
        if self.jump_buffered {
            self.input_timer.tick(time.delta());
            if self.input_timer.finished() {
                self.jump_buffered = false;
            }
        }
    }

    pub fn get_jump_force(&mut self) -> Option<f32> {
        if self.jump_buffered {
            self.reset_input();
            match self.jump_stage {
                JumpStage::Single => {
                    self.jump_stage = JumpStage::Double;
                    Some(10.0)
                }
                JumpStage::Double => {
                    self.jump_stage = JumpStage::Triple;
                    Some(15.0)
                }
                JumpStage::Triple => {
                    self.jump_stage = JumpStage::Single;
                    Some(20.0)
                }
            }
        } else {
            None
        }
    }

    pub fn get_wall_jump_force(&mut self) -> f32 {
        self.reset_input();
        15.0
    }

    pub fn buffer_jump(&mut self) {
        self.jump_buffered = true;
        self.input_timer.reset();
    }
}

impl Default for Jump {
    fn default() -> Self {
        Jump {
            input_timer: Timer::from_seconds(0.4, TimerMode::Once),
            jump_stage: JumpStage::Single,
            jump_buffered: false,
        }
    }
}

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

#[derive(PartialEq, Reflect)]
pub enum GroundedState {
    Grounded,
    Coyote,
    Rising,
    Falling,
    WallSliding(Vec3),
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

    pub fn is_wall_sliding(&self) -> bool {
        match self.state {
            GroundedState::WallSliding(_) => true,
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

#[derive(Component)]
pub struct PlayerWallSensor;

const PLAYER_ROTATION_SPEED: f32 = 10.0;

pub struct PlayerMovementPlugin;

impl Plugin for PlayerMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(set_player_direction)
            .add_system(handle_player_acceleration.after(set_player_direction))
            .add_system(handle_grounded)
            .add_system(buffer_jump)
            .add_system(handle_jumping.after(buffer_jump))
            .add_system(rotate_to_direction.after(set_player_direction))
            .add_system(move_player_from_rotation.after(rotate_to_direction))
            .add_system(handle_busy)
            .add_system(handle_wall_sliding)
            .add_system(detect_walls)
            .add_system(handle_wall_jumping);
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

pub fn aerial_drift(mut query: Query<(&mut Velocity, &Movement, &Grounded), With<Player>>) {
    let (mut velocity, movement, grounded) = query.single_mut();
    // To enable this to truly work, we probably need to start storing forward momentum
    // seperate from velocity and then apply it at the end of the physics loop, we don't want
    // to cancel out our x/z velo when inputting a direction in the air, we want to maintain it
    // and allow the player to influence it
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
            grounded.land();
        }
    } else {
        if is_grounded {
            grounded.walk_off();
        }
    }
}

pub fn buffer_jump(input: Res<Input<KeyCode>>, mut query: Query<&mut Jump, With<Player>>) {
    let mut jump = query.single_mut();

    if input.just_pressed(KeyCode::Space) {
        jump.buffer_jump();
    }
}

pub fn handle_jumping(mut query: Query<(&mut Velocity, &mut Grounded, &mut Jump), With<Player>>) {
    let (mut velocity, mut grounded, mut jump) = query.single_mut();

    if grounded.can_jump() {
        if let Some(force) = jump.get_jump_force() {
            grounded.jump();
            velocity.linvel.y = force;
        }
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

enum WallDetectionStatus {
    Hit(Entity),
    NoHit,
}

pub fn detect_walls(
    mut collision_events: EventReader<CollisionEvent>,
    rapier_context: Res<RapierContext>,
    mut player_query: Query<
        (Entity, &Transform, &mut Grounded, &mut Velocity),
        (With<Player>, Without<PlayerWallSensor>, Without<Wall>),
    >,
    wall_sensor_query: Query<Entity, (With<PlayerWallSensor>, Without<Player>, Without<Wall>)>,
    wall_query: Query<(Entity, &Transform), With<Wall>>,
) {
    let (player_entity, player_transform, mut grounded, mut velocity) = player_query.single_mut();
    let sensor_entity = wall_sensor_query.single();

    let can_wallslide = grounded.is_airborne() && !grounded.is_wall_sliding();

    for collision_event in collision_events.iter() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                let wall_detection_status =
                    if *e1 == sensor_entity && wall_query.contains(*e2) && can_wallslide {
                        WallDetectionStatus::Hit(*e2)
                    } else if *e2 == sensor_entity && wall_query.contains(*e1) && can_wallslide {
                        WallDetectionStatus::Hit(*e1)
                    } else {
                        WallDetectionStatus::NoHit
                    };

                if let WallDetectionStatus::Hit(wall) = wall_detection_status {
                    println!("Found a wall");
                    let (_, wall_transform) = wall_query.get(wall).unwrap();
                    let ray_pos = player_transform.translation;
                    let ray_dir = (wall_transform.translation - player_transform.translation)
                        .normalize_or_zero();
                    let max_distance = ray_pos.distance(wall_transform.translation);
                    let solid = true;
                    let filter = QueryFilter::new()
                        .exclude_sensors()
                        .exclude_collider(player_entity);

                    if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(
                        ray_pos,
                        ray_dir,
                        max_distance,
                        solid,
                        filter,
                    ) {
                        velocity.linvel.x = 0.0;
                        velocity.linvel.z = 0.0;
                        grounded.state = GroundedState::WallSliding(intersection.normal);
                    }
                }
            }

            CollisionEvent::Stopped(e1, e2, _) => {
                if (*e1 == sensor_entity && wall_query.contains(*e2) && grounded.is_wall_sliding())
                    || (*e2 == sensor_entity
                        && wall_query.contains(*e1)
                        && grounded.is_wall_sliding())
                {
                    grounded.state = GroundedState::Grounded;
                };
            }
        }
    }
}

pub fn handle_wall_sliding(mut query: Query<(&mut Velocity, &Grounded), With<Player>>) {
    let (mut velocity, grounded) = query.single_mut();

    match grounded.state {
        GroundedState::WallSliding(_normal) => {
            velocity.linvel.y = -2.0;
        }
        _ => (),
    }
}

pub fn handle_wall_jumping(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Grounded, &mut Jump), With<Player>>,
) {
    let (mut transform, mut velocity, mut grounded, mut jump) = query.single_mut();

    if let GroundedState::WallSliding(wall_normal) = grounded.state {
        if input.just_pressed(KeyCode::Space) {
            let position = transform.translation;
            transform.look_at(position + wall_normal, Vec3::Y);
            velocity.linvel = (Vec3::Y + wall_normal) * jump.get_wall_jump_force();
            grounded.state = GroundedState::Rising;
        }
    }
}
