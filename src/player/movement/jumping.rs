use std::fmt::Formatter;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    apply_momentum, get_direction_in_camera_space, Landing, MainCamera, Momentum, Player,
    PlayerAction, Wall,
};

pub struct PlayerJumpingPlugin;

impl Plugin for PlayerJumpingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Jump>()
            .add_system(handle_grounded)
            .add_system(buffer_jump)
            .add_system(handle_jumping.after(buffer_jump))
            .add_system(detect_walls)
            .add_system(handle_wall_jumping.before(apply_momentum))
            .add_system(aerial_drift)
            .add_system(reset_jumps_after_landing)
            .add_system(handle_jump_buffer);
    }
}

#[derive(Default, Reflect)]
pub enum JumpStage {
    #[default]
    Single,
    Double,
    Triple,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Jump {
    pub input_timer: Timer,
    pub jump_stage: JumpStage,
    pub jump_buffered: bool,
}

impl Jump {
    fn reset(&mut self) {
        self.reset_jump_stage();
        self.reset_input();
    }

    fn reset_jump_stage(&mut self) {
        self.jump_stage = JumpStage::Single;
    }

    fn reset_input(&mut self) {
        self.jump_buffered = false;
        self.input_timer.reset();
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        if self.jump_buffered {
            self.input_timer.tick(delta);
            if self.input_timer.finished() {
                self.reset();
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
            input_timer: Timer::from_seconds(0.2, TimerMode::Once),
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

impl std::fmt::Display for GroundedState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use GroundedState::*;
        match self {
            Grounded => write!(f, "Grounded")?,
            Coyote => write!(f, "Coyote")?,
            Rising => write!(f, "Rising")?,
            Falling => write!(f, "Falling")?,
            WallSliding(_) => write!(f, "WallSliding")?,
        }
        Ok(())
    }
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

#[derive(Component, Default)]
pub struct Drift(pub Vec3);

impl Drift {
    pub fn has_drift(&self) -> bool {
        self.0 != Vec3::ZERO
    }

    pub fn reset(&mut self) {
        self.0 = Vec3::ZERO;
    }

    pub fn set(&mut self, drift: Vec3) {
        self.0 = drift;
    }

    pub fn add(&mut self, drift: Vec3) {
        self.0 += drift;
    }
}

pub fn handle_jump_buffer(time: Res<Time>, mut query: Query<&mut Jump>) {
    for mut jump in &mut query {
        jump.update(time.delta());
    }
}
pub fn aerial_drift(
    time: Res<Time>,
    mut query: Query<(&mut Drift, &Grounded, &ActionState<PlayerAction>), With<Player>>,

    camera_query: Query<&Transform, With<MainCamera>>,
) {
    let (mut drift, grounded, action) = query.single_mut();
    let camera_transform = camera_query.single();

    if grounded.is_airborne() {
        drift.add(
            get_direction_in_camera_space(camera_transform, action) * (10.0 * time.delta_seconds()),
        );
    }
}

pub fn handle_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Grounded, &mut Drift), With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    let (entity, transform, mut grounded, mut drift) = query.single_mut();

    let is_grounded = grounded.is_grounded();
    let ray_pos = transform.translation;
    let ray_dir = Vec3::Y * -1.0;
    let max_distance = 1.1;
    let solid = true;
    let filter = QueryFilter::exclude_dynamic().exclude_sensors();

    if let Some((_entity, _intersection)) =
        rapier_context.cast_ray(ray_pos, ray_dir, max_distance, solid, filter)
    {
        if !is_grounded {
            grounded.land();
            drift.reset();
            commands.entity(entity).insert(Landing::new());
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

pub fn reset_jumps_after_landing(
    mut query: Query<(&Grounded, &mut Jump), (With<Player>, Without<Landing>)>,
) {
    let Ok((grounded, mut jump)) = query.get_single_mut() else {return;};

    if grounded.is_grounded() {
        jump.reset_jump_stage();
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
        (
            Entity,
            &Transform,
            &mut Grounded,
            &mut Velocity,
            &mut Friction,
        ),
        (With<Player>, Without<PlayerWallSensor>, Without<Wall>),
    >,
    wall_sensor_query: Query<Entity, (With<PlayerWallSensor>, Without<Player>, Without<Wall>)>,
    wall_query: Query<(Entity, &Transform), With<Wall>>,
) {
    let (player_entity, player_transform, mut grounded, mut velocity, mut friction) =
        player_query.single_mut();
    let sensor_entity = wall_sensor_query.single();
    let can_wallslide = grounded.is_airborne() && !grounded.is_wall_sliding(); // && velocity.linvel.y < 0.0;

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
                        friction.coefficient = 0.0;
                    }
                }
            }

            CollisionEvent::Stopped(e1, e2, _) => {
                if (*e1 == sensor_entity && wall_query.contains(*e2))
                    || (*e2 == sensor_entity && wall_query.contains(*e1))
                {
                    grounded.state = GroundedState::Grounded;
                    friction.coefficient = 1.0;
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
    mut query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut Grounded,
            &mut Momentum,
            &mut Jump,
        ),
        With<Player>,
    >,
) {
    let (mut transform, mut velocity, mut grounded, mut momentum, mut jump) = query.single_mut();

    if let GroundedState::WallSliding(wall_normal) = grounded.state {
        if input.just_pressed(KeyCode::Space) {
            let position = transform.translation;
            transform.look_at(position + wall_normal, Vec3::Y);
            momentum.set(jump.get_wall_jump_force());
            velocity.linvel = Vec3::Y * jump.get_wall_jump_force();
            grounded.state = GroundedState::Rising;
        }
    }
}
