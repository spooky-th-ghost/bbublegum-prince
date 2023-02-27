use bevy::prelude::*;

pub mod movement;
pub use movement::*;
pub mod inputs;
pub use inputs::*;
pub mod grabbing;
pub use grabbing::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayerMovementPlugin)
            .add_plugin(PlayerGrabbingPlugin);
    }
}
