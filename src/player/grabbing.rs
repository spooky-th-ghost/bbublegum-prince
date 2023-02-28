use crate::{
    HeavyItem, Item, ItemId, LightItem, MediumItem, Player, PlayerAction, PlayerGrabSensor, Weight,
};
use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use std::convert::From;

pub struct PlayerGrabbingPlugin;

impl Plugin for PlayerGrabbingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ItemsInRange::default())
            .add_system(detect_items)
            .add_system(grab_item.after(detect_items))
            .add_system(throw_item)
            .add_system(handle_thrown_momentum.after(throw_item));
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
pub struct HeldItem {
    pub item: ItemId,
    pub entity: Entity,
}

impl HeldItem {
    pub fn new(item: ItemId, entity: Entity) -> Self {
        HeldItem { item, entity }
    }
}

enum ItemDetectionStatus {
    Hit(Entity),
    NoHit,
}

#[derive(Component)]
pub struct ThrownItem {
    pub throw_velocity: Vec3,
    pub throw_position: Vec3,
}

impl ThrownItem {
    pub fn new(throw_velocity: Vec3, throw_position: Vec3) -> Self {
        ThrownItem {
            throw_velocity,
            throw_position,
        }
    }
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
    mut item_query: Query<(Entity, &mut Transform, &Item, Option<&RigidBody>), With<Item>>,
) {
    if !items_in_range.is_empty() {
        let Ok((player_entity, player_action)) = player_query.get_single() else {println!("No Player with an action state found in grab item, skipping"); return;};

        if player_action.just_pressed(PlayerAction::Grab) {
            if let Some((item_entity, item_weight)) = items_in_range.get_closest() {
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

                if let Ok((_, mut item_transform, item, item_rigidbody)) =
                    item_query.get_mut(item_entity)
                {
                    commands
                        .entity(player_entity)
                        .add_child(item_entity)
                        .insert(HeldItem::new(item.item_id, item_entity));
                    item_transform.rotation = Quat::default();
                    item_transform.translation = Vec3::new(0.0, 1.00, -1.00);
                    if item_rigidbody.is_some() {
                        commands
                            .entity(item_entity)
                            .remove::<RigidBody>()
                            .remove::<Collider>();
                        // .insert(LockedAxes::TRANSLATION_LOCKED | LockedAxes::ROTATION_LOCKED)
                        // .insert(Sensor);
                    }
                } else {
                    println!("Something went wrong while holding an item");
                };
            }
        }
    }
}

pub fn throw_item(
    mut commands: Commands,
    player_query: Query<(Entity, &HeldItem, &Transform, &ActionState<PlayerAction>), With<Player>>,
) {
    for (player_entity, held_item, player_transform, player_action) in &player_query {
        if player_action.just_pressed(PlayerAction::Grab) {
            let HeldItem {
                entity: item_entity,
                item: item_id,
            } = held_item;
            let player_forward = player_transform.forward().normalize_or_zero();
            let throw_velocity = (player_forward * 15.0) + (Vec3::Y * 10.0);
            let throw_position = player_transform.translation + (player_forward * 1.2);

            commands
                .entity(*item_entity)
                .remove_parent()
                .insert(ThrownItem::new(throw_velocity, throw_position))
                .insert(RigidBody::Dynamic)
                //This line should be based on the item
                .insert(item_id.into_collider());

            commands.entity(player_entity).remove::<HeldItem>();
        }
    }
}

pub fn handle_thrown_momentum(
    mut commands: Commands,
    mut item_query: Query<(Entity, &ThrownItem, &mut Velocity, &mut Transform), (With<RigidBody>)>,
) {
    for (item_entity, thrown_item, mut item_velocity, mut item_transform) in &mut item_query {
        commands.entity(item_entity).remove::<ThrownItem>();

        println!("Item Velo: {:?}", thrown_item.throw_velocity);
        item_velocity.linvel = thrown_item.throw_velocity;
        println!("Item Position: {:?}", thrown_item.throw_position);
        item_transform.translation = thrown_item.throw_position;
    }
}
