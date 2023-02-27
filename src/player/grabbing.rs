use crate::*;
use bevy::{prelude::*, utils::HashMap};
use leafwing_input_manager::prelude::ActionState;

pub struct PlayerGrabbingPlugin;

impl Plugin for PlayerGrabbingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ItemsInRange::default())
            .add_system(detect_items)
            .add_system(grab_item.after(detect_items));
    }
}

pub struct ItemRangeEntry {
    pub distance: f32,
}

#[derive(Resource, Default)]
pub struct ItemsInRange {
    items: HashMap<Entity, Weight>,
    closest_item: Option<(Entity, f32)>,
}

impl ItemsInRange {
    pub fn clear_closest(&mut self) {
        self.closest_item = None;
    }

    pub fn add(&mut self, entity: Entity, weight: Weight, distance: f32) {
        self.items.insert(entity, weight);
        if let Some((_, closest_distance)) = self.closest_item {
            if distance < closest_distance {
                self.closest_item = Some((entity, distance));
            }
        } else {
            self.closest_item = Some((entity, distance));
        }
    }

    pub fn remove(&mut self, entity: Entity) -> Option<Weight> {
        self.items.remove(&entity)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get_closest(&mut self) -> Option<(Entity, Weight)> {
        if let Some((entity, _)) = self.closest_item {
            self.items.remove_entry(&entity)
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct HeldItem;

enum ItemDetectionStatus {
    Hit(Entity),
    NoHit,
}

pub fn detect_items(
    mut items_in_range: ResMut<ItemsInRange>,
    mut collision_events: EventReader<CollisionEvent>,
    player_query: Query<&Transform, (With<Player>, Without<HeldItem>)>,
    grab_sensor_query: Query<Entity, (With<PlayerGrabSensor>, Without<Player>, Without<Item>)>,
    item_query: Query<(Entity, &Transform, Option<&HeavyItem>, Option<&MediumItem>), With<Item>>,
) {
    let sensor_entity = grab_sensor_query.single();
    for collision_event in collision_events.iter() {
        for player_transform in &player_query {
            match collision_event {
                CollisionEvent::Started(e1, e2, _) => {
                    let item_detection_status = if *e1 == sensor_entity && item_query.contains(*e2)
                    {
                        ItemDetectionStatus::Hit(*e2)
                    } else if *e2 == sensor_entity && item_query.contains(*e1) {
                        ItemDetectionStatus::Hit(*e1)
                    } else {
                        ItemDetectionStatus::NoHit
                    };

                    if let ItemDetectionStatus::Hit(item_entity) = item_detection_status {
                        println!("Found an Item");
                        let (_, item_transform, heavy, medium) =
                            item_query.get(item_entity).unwrap();

                        let item_weight = if heavy.is_some() {
                            Weight::Heavy
                        } else if medium.is_some() {
                            Weight::Medium
                        } else {
                            Weight::Light
                        };

                        let distance = player_transform
                            .translation
                            .distance(item_transform.translation);

                        items_in_range.add(item_entity, item_weight, distance);
                    }
                }
                CollisionEvent::Stopped(e1, e2, _) => {
                    let item_detection_status = if *e1 == sensor_entity && item_query.contains(*e2)
                    {
                        ItemDetectionStatus::Hit(*e2)
                    } else if *e2 == sensor_entity && item_query.contains(*e1) {
                        ItemDetectionStatus::Hit(*e1)
                    } else {
                        ItemDetectionStatus::NoHit
                    };

                    if let ItemDetectionStatus::Hit(item_entity) = item_detection_status {
                        items_in_range.remove(item_entity);
                    }
                }
            }
        }
    }
}

pub fn grab_item(
    mut commands: Commands,
    mut items_in_range: ResMut<ItemsInRange>,
    player_query: Query<(Entity, &ActionState<PlayerAction>), (With<Player>,)>,
    mut item_query: Query<(Entity, &mut Transform, Option<&RigidBody>), With<Item>>,
) {
    if !items_in_range.is_empty() {
        let Ok((player_entity, player_action)) = player_query.get_single() else {println!("No Player with an action state found in grab item, skipping"); return;};

        if player_action.just_pressed(PlayerAction::Grab) {
            if let Some((item_entity, item_weight)) = items_in_range.get_closest() {
                println!("We grabbing");
                commands.entity(item_entity).insert(Sensor);
                use Weight::*;
                match item_weight {
                    Heavy => {
                        commands.entity(player_entity).insert(HeavyItem);
                    }
                    Medium => {
                        commands.entity(player_entity).insert(MediumItem);
                    }
                    Light => {
                        commands.entity(player_entity).insert(LightItem);
                    }
                }

                if let Ok((_, mut item_transform, item_rigidbody)) = item_query.get_mut(item_entity)
                {
                    commands
                        .entity(player_entity)
                        .add_child(item_entity)
                        .insert(HeldItem);
                    item_transform.rotation = Quat::default();
                    item_transform.translation = Vec3::Y * 1.5;
                    if item_rigidbody.is_some() {
                        commands
                            .entity(item_entity)
                            .remove::<RigidBody>()
                            .remove::<Velocity>();
                    }
                } else {
                    println!("Something went wrong while holding an item");
                };
            }
        }
    }
}
