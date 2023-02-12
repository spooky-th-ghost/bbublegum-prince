use crate::{Movement, Player};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct CameraController {
    pub z_distance: f32,
    pub y_distance: f32,
    pub angle: f32,
    pub easing: f32,
    pub target_position: Vec3,
    pub player_position: Vec3,
}

impl CameraController {
    pub fn desired_y_height(&self, velocity_magnitude: f32) -> f32 {
        if velocity_magnitude < 3.0 {
            self.y_distance / 2.0
        } else {
            self.y_distance
        }
    }

    pub fn desired_z_distance(&self, velocity_magnitude: f32) -> f32 {
        if velocity_magnitude < 55.0 {
            self.z_distance
        } else {
            self.z_distance * 1.5
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
        }
    }
}

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(update_camera_target_position)
            .add_system(lerp_to_camera_position.after(update_camera_target_position))
            .add_system(rotate_camera);
    }
}
fn update_camera_target_position(
    mut camera_query: Query<&mut CameraController>,
    player_query: Query<(&Transform, &Velocity), With<Player>>,
) {
    let mut camera = camera_query.single_mut();
    let (player_transform, player_velocity) = player_query.single();

    let mut starting_transform = player_transform.clone();
    starting_transform.rotation = Quat::default();
    starting_transform.rotate_y(camera.angle.to_radians());
    let dir = starting_transform.forward().normalize();
    camera.target_position = starting_transform.translation
        + (dir * camera.desired_z_distance(player_velocity.linvel.length_squared()))
        + (Vec3::Y * camera.desired_y_height(player_velocity.linvel.length_squared()));
    camera.player_position = player_transform.translation;
}

fn lerp_to_camera_position(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &CameraController)>,
) {
    for (mut transform, camera_controller) in &mut camera_query {
        let lerped_position = transform.translation.lerp(
            camera_controller.target_position,
            time.delta_seconds() * camera_controller.easing,
        );
        transform.translation = lerped_position;
        transform.look_at(camera_controller.player_position, Vec3::Y);
    }
}

fn rotate_camera(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    player_query: Query<&Movement>,
    mut camera_query: Query<&mut CameraController>,
) {
    let mut camera = camera_query.single_mut();
    let movement = player_query.single();

    if movement.0.x != 0.0 {
        camera.angle += movement.0.x * 10.0 * time.delta_seconds();
    }

    if keyboard.pressed(KeyCode::Q) {
        camera.angle -= 45.0 * time.delta_seconds();
    }
    if keyboard.pressed(KeyCode::E) {
        camera.angle += 45.0 * time.delta_seconds();
    }

    if camera.angle > 360.0 {
        camera.angle -= 360.0;
    }

    if camera.angle < -360.0 {
        camera.angle += 360.0;
    }
}
