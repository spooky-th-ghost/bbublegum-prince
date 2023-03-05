use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn spawn_chain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0))),
            material: materials.add(Color::WHITE.into()),
            transform: Transform::from_xyz(3.0, 5.0, 3.0),
            ..default()
        })
        .with_children(|parent| {
            let mesh_shape = shape::Capsule {
                radius: 0.1,
                rings: 0,
                depth: 0.2,
                latitudes: 6,
                longitudes: 12,
                uv_profile: shape::CapsuleUvProfile::Aspect,
            };

            let mut previous_entity = parent
                .spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(mesh_shape)),
                    material: materials.add(Color::WHITE.into()),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0),
                    ..default()
                })
                .insert(RigidBody::Dynamic)
                .insert(Collider::capsule_y(0.1, 0.1))
                .id();

            for i in 1..10 {
                let i_float = i as f32;
                let current_entity = parent
                    .spawn(PbrBundle {
                        mesh: meshes.add(Mesh::from(mesh_shape)),
                        material: materials.add(Color::WHITE.into()),
                        transform: Transform::from_xyz(0.0, i_float * -0.2, 0.0),
                        ..default()
                    })
                    .insert(RigidBody::Dynamic)
                    .insert(Collider::capsule_y(0.1, 0.1))
                    .id();

                let anchor1 = Vec3::Y * ((i_float - 1.0) * -0.2);
                let anchor2 = Vec3::Y * (i_float * -0.2);
                let joint = SphericalJointBuilder::new()
                    .local_anchor1(anchor1)
                    .local_anchor2(anchor2);

                parent
                    .spawn(RigidBody::Dynamic)
                    .insert(ImpulseJoint::new(previous_entity, joint));

                previous_entity = current_entity;
            }
        });
}
