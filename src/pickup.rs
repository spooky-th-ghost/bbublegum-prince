use bevy::{prelude::*, utils::HashMap};

#[derive(Component, Eq, PartialEq)]
pub enum Pickup {
    Coin(u8),
    Health(u8),
    Key { amount: u8, resource_name: String },
}

impl Pickup {
    pub fn get_resource_name(&self) -> String {
        match self {
            Pickup::Coin(_) => "Coins".to_string(),
            Pickup::Health(_) => "Health".to_string(),
            Pickup::Key {
                amount: _,
                resource_name,
            } => resource_name.clone(),
        }
    }

    pub fn get_amount(&self) -> u8 {
        match self {
            Pickup::Coin(amount) => *amount,
            Pickup::Health(amount) => *amount,
            Pickup::Key {
                amount,
                resource_name: _,
            } => *amount,
        }
    }
}

#[derive(Resource, Default)]
pub struct PickupsInventory(HashMap<String, u8>);

impl PickupsInventory {
    pub fn add(&mut self, pickup: Pickup) {}
}
