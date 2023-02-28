use bevy::{prelude::*, utils::HashMap};
use bevy_rapier3d::prelude::*;

#[derive(PartialEq, Clone, Copy, Default)]
pub enum ItemId {
    #[default]
    WoodenCrate,
}

impl ItemId {
    pub fn get_weight(&self) -> Weight {
        use ItemId::*;
        use Weight::*;
        match self {
            WoodenCrate => Medium,
        }
    }

    pub fn into_collider(&self) -> Collider {
        match self {
            ItemId::WoodenCrate => Collider::cuboid(0.5, 0.5, 0.5),
        }
    }
}

pub enum Weight {
    Light,
    Medium,
    Heavy,
}

#[derive(Component, Clone, Copy, Default)]
pub struct Item {
    pub item_id: ItemId,
}

#[derive(Component)]
pub struct HeavyItem;
#[derive(Component)]
pub struct MediumItem;
#[derive(Component)]
pub struct LightItem;

// pub struct ItemInfo {
//     pub mesh: Handle<Mesh>,
//     pub color: Color,
//     pub weight: Weight,
//     pub shape: ColliderShape,
// }

// #[derive(Resource)]
// pub struct ItemCache(HashMap<ItemId, ItemInfo>);

// impl ItemCache {}
