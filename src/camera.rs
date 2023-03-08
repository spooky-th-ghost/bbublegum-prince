use crate::{Momentum, Movement, Player, PlayerAction};
use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct IdeaUi;

pub enum CameraMode {
    Normal,
    Fixed { position: Vec3, look_target: Vec3 },
}
#[derive(Component)]
pub struct CameraController {
    pub z_distance: f32,
    pub y_distance: f32,
    pub angle: f32,
    pub easing: f32,
    pub target_position: Vec3,
    pub player_position: Vec3,
    pub mode: CameraMode,
    pub blocked_by_a_wall: bool,
}

impl CameraController {
    pub fn desired_y_height(&self, momentum: f32) -> f32 {
        if momentum < 5.0 {
            self.y_distance / 2.0
        } else {
            self.y_distance
        }
    }

    pub fn desired_z_distance(&self, momentum: f32) -> f32 {
        if momentum < 10.0 {
            self.z_distance
        } else {
            self.z_distance * 1.5
        }
    }

    pub fn desired_easing_speed(&self) -> f32 {
        match self.mode {
            CameraMode::Normal => {
                if self.blocked_by_a_wall {
                    self.easing * 2.5
                } else {
                    self.easing
                }
            }
            CameraMode::Fixed {
                position: _,
                look_target: _,
            } => self.easing * 5.0,
        }
    }
}

impl Default for CameraController {
    fn default() -> Self {
        CameraController {
            z_distance: 10.0,
            y_distance: 7.0,
            angle: 0.0,
            easing: 4.0,
            target_position: Vec3::ZERO,
            player_position: Vec3::ZERO,
            // mode: CameraMode::Normal,
            mode: CameraMode::Fixed {
                position: Vec3::new(0.0, 40.0, -23.0),
                look_target: Vec3::ZERO,
            },
            blocked_by_a_wall: false,
        }
    }
}
pub fn circle_distribution(initial_index: usize, distance: f32, total: f32) -> Transform {
    let i = initial_index as f32;
    let theta = 2.0 * (std::f32::consts::PI / total) * i;
    Transform::from_xyz(theta.cos() * distance, theta.sin() * distance, 0.0)
}

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_main_camera)
            .add_startup_system(spawn_ui_camera)
            .add_system(update_camera_target_position)
            .add_system(lerp_to_camera_position.after(update_camera_target_position))
            .add_system(rotate_camera)
            .add_system(debug_change_camera_mode);
    }
}
fn spawn_main_camera(mut commands: Commands) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::splat(10.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(CameraController::default())
        .insert(MainCamera);
}

pub fn spawn_ui_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(Camera3dBundle {
            camera_3d: Camera3d {
                clear_color: ClearColorConfig::None,
                ..default()
            },
            camera: Camera {
                priority: 1,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 500.0, 0.0)),
            ..default()
        })
        .insert(UiCamera);

    commands
        .spawn(SpatialBundle {
            transform: Transform::from_xyz(0.0, 500.0, -3.0),
            ..default()
        })
        .insert(IdeaUi)
        .with_children(|parent| {
            for i in 0..10 {
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.25,
                        subdivisions: 2,
                    })),
                    material: materials.add(
                        Color::Rgba {
                            red: 1.0,
                            green: 0.0,
                            blue: 0.0,
                            alpha: 0.5,
                        }
                        .into(),
                    ),
                    transform: circle_distribution(i, 0.85, 10.0),
                    ..default()
                });
            }
        });
}

fn debug_change_camera_mode(
    mut camera_query: Query<&mut CameraController>,
    player_query: Query<&ActionState<PlayerAction>>,
) {
    let mut camera = camera_query.single_mut();
    let Ok(player_action) = player_query.get_single() else {println!("No Player to set camera mode"); return;};
    if player_action.just_pressed(PlayerAction::CameraMode) {
        if let CameraMode::Normal = camera.mode {
            camera.mode = CameraMode::Fixed {
                position: Vec3::new(0.0, 30.0, -20.0),
                look_target: Vec3::ZERO,
            };
        } else {
            camera.mode = CameraMode::Normal;
        }
    }
}
fn update_camera_target_position(
    rapier_context: Res<RapierContext>,
    mut camera_query: Query<&mut CameraController>,
    player_query: Query<(Entity, &Transform, &Momentum), With<Player>>,
) {
    let mut camera = camera_query.single_mut();
    let (player_entity, player_transform, player_momentum) = player_query.single();

    let mut starting_transform = player_transform.clone();
    starting_transform.rotation = Quat::default();
    starting_transform.rotate_y(camera.angle.to_radians());
    let dir = starting_transform.forward().normalize();
    camera.player_position = player_transform.translation;
    let mut desired_position = starting_transform.translation
        + (dir * camera.desired_z_distance(player_momentum.get()))
        + (Vec3::Y * camera.desired_y_height(player_momentum.get()));

    let ray_pos = player_transform.translation;
    let ray_dir = (desired_position - player_transform.translation).normalize_or_zero();
    let max_distance = ray_pos.distance(desired_position) * 1.0;
    let solid = true;
    let filter = QueryFilter::new()
        .exclude_sensors()
        .exclude_collider(player_entity);

    if let Some((_, intersection)) =
        rapier_context.cast_ray_and_get_normal(ray_pos, ray_dir, max_distance, solid, filter)
    {
        desired_position = intersection.point;
    }

    camera.target_position = desired_position;
}

fn lerp_to_camera_position(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &CameraController)>,
) {
    for (mut transform, camera) in &mut camera_query {
        match camera.mode {
            CameraMode::Normal => {
                let lerped_position = transform.translation.lerp(
                    camera.target_position,
                    time.delta_seconds() * camera.desired_easing_speed(),
                );
                transform.translation = lerped_position;
                transform.look_at(camera.player_position, Vec3::Y);
            }
            CameraMode::Fixed {
                position,
                look_target,
            } => {
                let lerped_position = transform.translation.lerp(
                    position,
                    time.delta_seconds() * camera.desired_easing_speed(),
                );

                transform.translation = lerped_position;
                transform.look_at(look_target, Vec3::Y);
            }
        }
    }
}

fn rotate_camera(
    mut camera_query: Query<&mut CameraController>,
    player_query: Query<&ActionState<PlayerAction>>,
) {
    let mut camera = camera_query.single_mut();
    let Ok(player_action) = player_query.get_single() else {println!("No Player to rotate the camera"); return;};
    if player_action.just_pressed(PlayerAction::CameraLeft) {
        camera.angle -= 45.0;
    }
    if player_action.just_pressed(PlayerAction::CameraRight) {
        camera.angle += 45.0;
    }

    if camera.angle > 360.0 {
        camera.angle -= 360.0;
    }

    if camera.angle < -360.0 {
        camera.angle += 360.0;
    }
}
