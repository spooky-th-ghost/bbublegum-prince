use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::{
    apply_momentum, get_direction_in_camera_space, Coyote, Drift, Grounded, Jump, Landing, Ledge,
    LedgeGrab, MainCamera, Momentum, Player, PlayerAction, PlayerLedgeSensor, PlayerWallSensor,
    Wall, Walljump,
};

pub struct PlayerJumpingPlugin;

impl Plugin for PlayerJumpingPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Jump>()
            .add_system(handle_grounded)
            .add_system(buffer_jump)
            .add_system(handle_jumping.after(buffer_jump))
            .add_system(detect_walls)
            .add_system(detect_ledges)
            .add_system(handle_wall_jumping.before(apply_momentum))
            .add_system(aerial_drift.before(apply_momentum))
            .add_system(handle_ledge_grab.before(apply_momentum))
            .add_system(reset_jumps_after_landing)
            .add_system(add_friction_when_landing)
            .add_system(handle_jump_buffer);
    }
}

pub fn handle_jump_buffer(time: Res<Time>, mut query: Query<&mut Jump>) {
    for mut jump in &mut query {
        jump.update(time.delta());
    }
}

pub fn aerial_drift(
    time: Res<Time>,
    mut query: Query<
        (&mut Drift, &ActionState<PlayerAction>),
        (With<Player>, Without<Grounded>, Without<LedgeGrab>),
    >,

    camera_query: Query<&Transform, With<MainCamera>>,
) {
    let camera_transform = camera_query.single();

    for (mut drift, action) in &mut query {
        drift.add(
            get_direction_in_camera_space(camera_transform, action) * (10.0 * time.delta_seconds()),
        );
    }
}

pub fn handle_grounded(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform, &mut Drift, Option<&Grounded>), With<Player>>,
    rapier_context: Res<RapierContext>,
) {
    for (entity, transform, mut drift, grounded) in &mut query {
        let is_grounded = grounded.is_some();
        let ray_pos = transform.translation;
        let ray_dir = Vec3::Y * -1.0;
        let max_distance = 1.1;
        let solid = true;
        let filter = QueryFilter::exclude_dynamic().exclude_sensors();

        if let Some((_entity, _intersection)) =
            rapier_context.cast_ray(ray_pos, ray_dir, max_distance, solid, filter)
        {
            if !is_grounded {
                drift.reset();
                commands
                    .entity(entity)
                    .insert(Grounded)
                    .insert(Landing::new());
            }
        } else {
            if is_grounded {
                commands
                    .entity(entity)
                    .insert(Coyote::new())
                    .remove::<Grounded>();
            }
        }
    }
}

pub fn buffer_jump(mut query: Query<(&mut Jump, &ActionState<PlayerAction>), With<Player>>) {
    for (mut jump, action) in &mut query {
        if action.just_pressed(PlayerAction::Jump) {
            jump.buffer_jump();
        }
    }
}

pub fn handle_jumping(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Velocity,
            &mut Jump,
            Option<&Grounded>,
            Option<&Coyote>,
        ),
        With<Player>,
    >,
) {
    for (entity, mut velocity, mut jump, grounded, coyote) in &mut query {
        if grounded.is_some() || coyote.is_some() {
            if let Some(force) = jump.get_jump_force() {
                velocity.linvel.y = force;

                if grounded.is_some() {
                    commands.entity(entity).remove::<Grounded>();
                }
                if coyote.is_some() {
                    commands.entity(entity).remove::<Coyote>();
                }
            }
        }
    }
}

pub fn reset_jumps_after_landing(
    mut query: Query<&mut Jump, (With<Player>, With<Grounded>, Without<Landing>)>,
) {
    for mut jump in &mut query {
        jump.reset_jump_stage();
    }
}

enum WallDetectionStatus {
    Hit(Entity),
    NoHit,
}

pub fn detect_walls(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    rapier_context: Res<RapierContext>,
    mut player_query: Query<
        (Entity, &Transform, &mut Friction, Option<&Walljump>),
        (
            With<Player>,
            Without<Grounded>,
            Without<PlayerWallSensor>,
            Without<Wall>,
        ),
    >,
    wall_sensor_query: Query<Entity, (With<PlayerWallSensor>, Without<Player>, Without<Wall>)>,
    wall_query: Query<(Entity, &Transform), With<Wall>>,
) {
    let sensor_entity = wall_sensor_query.single();
    for (player_entity, player_transform, mut friction, walljump) in &mut player_query {
        for collision_event in collision_events.iter() {
            match collision_event {
                CollisionEvent::Started(e1, e2, _) => {
                    let wall_detection_status = if *e1 == sensor_entity
                        && wall_query.contains(*e2)
                        && walljump.is_none()
                    {
                        WallDetectionStatus::Hit(*e2)
                    } else if *e2 == sensor_entity && wall_query.contains(*e1) && walljump.is_none()
                    {
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
                            commands
                                .entity(player_entity)
                                .insert(Walljump(intersection.normal));
                            friction.coefficient = 0.0;
                        }
                    }
                }

                CollisionEvent::Stopped(e1, e2, _) => {
                    if (*e1 == sensor_entity && wall_query.contains(*e2))
                        || (*e2 == sensor_entity && wall_query.contains(*e1))
                    {
                        friction.coefficient = 1.0;
                        if walljump.is_some() {
                            commands.entity(player_entity).remove::<Walljump>();
                        }
                    };
                }
            }
        }
    }
}

pub fn handle_wall_jumping(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut Velocity,
            &mut Momentum,
            &mut Jump,
            &Walljump,
            &ActionState<PlayerAction>,
        ),
        With<Player>,
    >,
) {
    for (entity, mut transform, mut velocity, mut momentum, mut jump, walljump, action) in
        &mut query
    {
        if action.just_pressed(PlayerAction::Jump) {
            let position = transform.translation;
            transform.look_at(position + walljump.0, Vec3::Y);
            momentum.set(jump.get_wall_jump_force());
            velocity.linvel = Vec3::Y * jump.get_wall_jump_force();
            commands.entity(entity).remove::<Walljump>();
        }
    }
}

enum LedgeDetectionStatus {
    Hit(Entity),
    NoHit,
}

pub fn detect_ledges(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    rapier_context: Res<RapierContext>,
    mut player_query: Query<
        (
            Entity,
            &Transform,
            &mut Velocity,
            &mut GravityScale,
            Option<&LedgeGrab>,
            Option<&Walljump>,
        ),
        (
            With<Player>,
            Without<Grounded>,
            Without<PlayerLedgeSensor>,
            Without<Wall>,
        ),
    >,
    ledge_sensor_query: Query<Entity, (With<PlayerLedgeSensor>, Without<Player>, Without<Wall>)>,
    ledge_query: Query<(Entity, &Transform), With<Ledge>>,
) {
    let sensor_entity = ledge_sensor_query.single();
    for (
        player_entity,
        player_transform,
        mut player_velocity,
        mut player_gravity,
        ledgegrab,
        walljump,
    ) in &mut player_query
    {
        for collision_event in collision_events.iter() {
            match collision_event {
                CollisionEvent::Started(e1, e2, _) => {
                    let ledge_detection_status =
                        if *e1 == sensor_entity && ledge_query.contains(*e2) && ledgegrab.is_none()
                        {
                            LedgeDetectionStatus::Hit(*e2)
                        } else if *e2 == sensor_entity
                            && ledge_query.contains(*e1)
                            && ledgegrab.is_none()
                        {
                            LedgeDetectionStatus::Hit(*e1)
                        } else {
                            LedgeDetectionStatus::NoHit
                        };

                    if let LedgeDetectionStatus::Hit(ledge) = ledge_detection_status {
                        println!("Hit a ledge");
                        let (_, ledge_transform) = ledge_query.get(ledge).unwrap();
                        let mut ray_pos = player_transform.translation;
                        ray_pos.y = ledge_transform.translation.y;
                        let ray_dir =
                            (ledge_transform.translation - ray_pos.clone()).normalize_or_zero();
                        println!("Ray Origin: {:?}\nRay Direction: {:?}", ray_pos, ray_dir);
                        let max_distance = ray_pos.distance(ledge_transform.translation);
                        let solid = true;
                        let filter = QueryFilter::new().exclude_collider(player_entity);

                        if let Some((_, intersection)) = rapier_context.cast_ray_and_get_normal(
                            ray_pos,
                            ray_dir,
                            max_distance,
                            solid,
                            filter,
                        ) {
                            println!("Found the ledge");
                            player_velocity.linvel = Vec3::ZERO;
                            player_gravity.0 = 0.0;
                            commands
                                .entity(player_entity)
                                .insert(LedgeGrab(intersection.normal * -1.0));

                            if walljump.is_some() {
                                commands.entity(player_entity).remove::<Walljump>();
                            }
                        }
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn handle_ledge_grab(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &mut GravityScale,
            &ActionState<PlayerAction>,
            &LedgeGrab,
        ),
        With<Player>,
    >,
) {
    for (entity, mut transform, mut gravity_scale, action, ledgegrab) in &mut query {
        if action.just_pressed(PlayerAction::Grab) {
            println!("Dropping from ledge");
        }

        if action.just_pressed(PlayerAction::Jump) {
            println!("Climbing a ledge");
            let new_position = transform.translation + (ledgegrab.0 * 1.5) + (Vec3::Y * 1.8);
            transform.translation = new_position;
        }

        if action.just_pressed(PlayerAction::Grab) || action.just_pressed(PlayerAction::Jump) {
            commands.entity(entity).remove::<LedgeGrab>();
            gravity_scale.0 = 1.0;
        }
    }
}

pub fn add_friction_when_landing(
    mut player_query: Query<&mut Friction, (With<Player>, Added<Grounded>)>,
) {
    for mut friction in &mut player_query {
        friction.coefficient = 1.0;
    }
}
