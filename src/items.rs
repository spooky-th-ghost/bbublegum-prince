pub use bevy::prelude::*;

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
