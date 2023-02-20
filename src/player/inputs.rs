use bevy::prelude::*;
use leafwing_input_manager::{prelude::*, *};

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Default)]
pub enum PlayerAction {
    #[default]
    Up,
    Down,
    Left,
    Right,
    Jump,
    CameraLeft,
    CameraRight,
    ClimbLedge,
    DropLedge,
}

#[derive(Bundle)]
pub struct InputListenerBundle {
    #[bundle]
    input_manager: InputManagerBundle<PlayerAction>,
}

impl InputListenerBundle {
    pub fn input_map() -> InputListenerBundle {
        use PlayerAction::*;

        let input_map = input_map::InputMap::new([
            (KeyCode::W, Up),
            (KeyCode::S, Down),
            (KeyCode::A, Left),
            (KeyCode::D, Right),
            (KeyCode::Space, Jump),
            (KeyCode::Q, CameraLeft),
            (KeyCode::E, CameraRight),
            (KeyCode::Space, ClimbLedge),
            (KeyCode::X, DropLedge),
        ])
        .build();

        InputListenerBundle {
            input_manager: InputManagerBundle {
                input_map,
                ..Default::default()
            },
        }
    }
}
