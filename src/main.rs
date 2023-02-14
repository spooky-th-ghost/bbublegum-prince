use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub mod player;
use leafwing_input_manager::prelude::InputManagerPlugin;
pub use player::*;

pub mod camera;
pub use camera::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_editor_pls::prelude::EditorPlugin)
        .add_plugin(bevy_inspector_egui_rapier::InspectableRapierPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(InputManagerPlugin::<PlayerAction>::default())
        .add_plugin(PlayerPlugin)
        .add_plugin(CameraControlPlugin)
        .add_plugin(PhysiscsInteractablesPlugin)
        .register_type::<Grounded>()
        .insert_resource(RapierConfiguration {
            gravity: Vec3::Y * -30.0,
            ..default()
        })
        .insert_resource(PlayerSpeed::default())
        .add_startup_system(spawn_world)
        .add_system(rotate_block)
        .run();
}

#[derive(Component, Default)]
pub struct Movement(pub Vec3);

impl Movement {
    pub fn is_moving(&self) -> bool {
        self.0 != Vec3::ZERO
    }
}

#[derive(Component, Default)]
pub struct Momentum(f32);

impl Momentum {
    pub fn has_momentum(&self) -> bool {
        self.0 != 0.0
    }

    pub fn reset(&mut self) {
        self.0 = 0.0;
    }

    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn set(&mut self, momentum: f32) {
        self.0 = momentum;
    }

    pub fn add(&mut self, momentum: f32) {
        self.0 += momentum;
    }
}

#[derive(Component)]
pub struct Rot;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct WindZone(pub Vec3);

impl WindZone {
    pub fn get_force(&self) -> OutsideForce {
        OutsideForce(self.0)
    }
}

#[derive(Default, Component, Reflect)]
#[reflect(Component)]
pub struct OutsideForce(pub Vec3);

pub fn rotate_block(time: Res<Time>, mut query: Query<&mut Transform, With<Rot>>) {
    for mut transform in &mut query {
        transform.rotate_y(1.0 * time.delta_seconds());
    }
}

pub fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::splat(10.0))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(CameraController::default())
        .insert(MainCamera);

    // Player
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule::default())),
            material: materials.add(Color::TURQUOISE.into()),
            transform: Transform::from_xyz(-5.0, 30.0, -5.0),
            ..default()
        })
        .insert(RigidBody::Dynamic)
        .insert(Velocity::default())
        .insert(LockedAxes::ROTATION_LOCKED)
        .insert(Collider::capsule_y(0.5, 0.5))
        .insert(Movement::default())
        .insert(Damping {
            linear_damping: 0.2,
            angular_damping: 0.0,
        })
        .insert(Grounded::default())
        .insert(Jump::default())
        .insert(Drift::default())
        .insert(Momentum::default())
        .insert(InputListenerBundle::input_map())
        .insert(Friction {
            coefficient: 1.0,
            combine_rule: CoefficientCombineRule::Min,
        })
        .insert(Player)
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(0.5, 0.5, 0.5))),
                material: materials.add(Color::RED.into()),
                transform: Transform::from_xyz(0.0, 0.5, -0.5),
                ..default()
            });

            parent
                .spawn(TransformBundle::default())
                .insert(Collider::cylinder(0.1, 0.75))
                .insert(PlayerWallSensor)
                .insert(Sensor)
                .insert(ActiveEvents::COLLISION_EVENTS);
        });

    // Light
    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.45,
    });

    // Ground
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(50.0, 1.0, 50.0))),
            material: materials.add(Color::GREEN.into()),
            transform: Transform::from_xyz(0.0, -1.0, 0.0),
            ..default()
        })
        .insert(Collider::cuboid(25.0, 0.5, 25.0))
        .insert(RigidBody::Fixed);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(50.0, 50.0, 1.0))),
            material: materials.add(Color::PURPLE.into()),
            transform: Transform::from_xyz(0.0, 24.5, 25.0),
            ..default()
        })
        .insert(Collider::cuboid(25.0, 25.0, 0.5))
        .insert(Wall)
        .insert(RigidBody::Fixed);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(50.0, 50.0, 1.0))),
            material: materials.add(Color::PURPLE.into()),
            transform: Transform::from_xyz(0.0, 24.5, -25.0),
            ..default()
        })
        .insert(Collider::cuboid(25.0, 25.0, 0.5))
        .insert(Wall)
        .insert(RigidBody::Fixed);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 50.0, 50.0))),
            material: materials.add(Color::PURPLE.into()),
            transform: Transform::from_xyz(25.0, 24.5, 0.0),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 25.0, 25.0))
        .insert(Wall)
        .insert(RigidBody::Fixed);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 50.0, 50.0))),
            material: materials.add(Color::PURPLE.into()),
            transform: Transform::from_xyz(-25.0, 24.5, 0.0),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 25.0, 25.0))
        .insert(Wall)
        .insert(RigidBody::Fixed);
    // Block
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(5.0, 5.0, 5.0))),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(-5.0, 3.5, -5.0),
            ..default()
        })
        .insert(Collider::cuboid(2.5, 2.5, 2.5))
        .insert(Wall)
        .insert(RigidBody::Fixed);

    // Wall jump blocks
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 40.0, 5.0))),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(10.0, 20.0, 10.0),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 20.0, 2.5))
        .insert(Wall)
        .insert(RigidBody::Fixed);

    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 40.0, 5.0))),
            material: materials.add(Color::BLUE.into()),
            transform: Transform::from_xyz(15.0, 20.0, 10.0),
            ..default()
        })
        .insert(Collider::cuboid(0.5, 20.0, 2.5))
        .insert(Wall)
        .insert(RigidBody::Fixed);

    // // Wind Zone
    // commands
    //     .spawn(TransformBundle {
    //         local: Transform::from_xyz(-2.0, 1.5, -6.0),
    //         ..default()
    //     })
    //     .insert(Collider::cuboid(2.5, 2.5, 2.5))
    //     .insert(Sensor)
    //     .insert(ActiveEvents::COLLISION_EVENTS)
    //     .insert(WindZone(Vec3::new(4.0, 0.0, 4.0)))
    //     .insert(RigidBody::Fixed);
}

pub fn handle_entering_wind_zones(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    zone_query: Query<(Entity, &WindZone), With<Sensor>>,
    movable_query: Query<(Entity, Option<&OutsideForce>), (With<Movement>, With<Collider>)>,
) {
    for (zone_entity, windzone) in &zone_query {
        for (movable_entity, has_force) in &movable_query {
            for collision_event in collision_events.iter() {
                match collision_event {
                    CollisionEvent::Started(e1, e2, _flags) => {
                        if (*e1 == zone_entity && *e2 == movable_entity)
                            || (*e2 == zone_entity && *e1 == movable_entity)
                        {
                            if has_force.is_none() {
                                commands.entity(movable_entity).insert(windzone.get_force());
                            }
                        }
                    }
                    CollisionEvent::Stopped(e1, e2, _flags) => {
                        if (*e1 == zone_entity && *e2 == movable_entity)
                            || (*e2 == zone_entity && *e1 == movable_entity)
                        {
                            if has_force.is_some() {
                                commands.entity(movable_entity).remove::<OutsideForce>();
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct PhysiscsInteractablesPlugin;

impl Plugin for PhysiscsInteractablesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_entering_wind_zones);
    }
}
