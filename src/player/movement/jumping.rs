use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{Movement, Player, Wall};

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

pub struct PlayerJumpingPlugin;

impl Plugin for PlayerJumpingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_grounded)
            .add_system(buffer_jump)
            .add_system(handle_jumping.after(buffer_jump))
            .add_system(handle_wall_sliding)
            .add_system(detect_walls)
            .add_system(handle_wall_jumping);
    }
}

pub fn aerial_drift(mut query: Query<(&mut Velocity, &Movement, &Grounded), With<Player>>) {
    let (mut velocity, movement, grounded) = query.single_mut();
    // To enable this to truly work, we probably need to start storing forward momentum
    // seperate from velocity and then apply it at the end of the physics loop, we don't want
    // to cancel out our x/z velo when inputting a direction in the air, we want to maintain it
    // and allow the player to influence it
    //
    // play around with having a version of this drift apply on the ground too, not sure how it
    // will feel but it may make the roation based movement feel a little bit smoother, don't want
    // to go full sm64
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
