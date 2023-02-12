use bevy::prelude::*;

pub mod movement;
pub use movement::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerMovementPlugin);
    }
}
