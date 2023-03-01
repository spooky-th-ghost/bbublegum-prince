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
    Grab,
    CameraLeft,
    CameraRight,
    CameraMode,
    Move,
    Crouch,
}

#[derive(Bundle)]
pub struct InputListenerBundle {
    #[bundle]
    input_manager: InputManagerBundle<PlayerAction>,
}

impl InputListenerBundle {
    pub fn input_map() -> InputListenerBundle {
        use PlayerAction::*;

        let mut input_map = input_map::InputMap::new([
            (KeyCode::W, Up),
            (KeyCode::S, Down),
            (KeyCode::A, Left),
            (KeyCode::D, Right),
            (KeyCode::Space, Jump),
            (KeyCode::Q, CameraLeft),
            (KeyCode::E, CameraRight),
            (KeyCode::Z, CameraMode),
            (KeyCode::X, Grab),
            (KeyCode::R, Crouch),
        ])
        //DEBUG THIS IS ALL DEBUG, DONT HARDCODE A GAMEPAD ID
        .set_gamepad(Gamepad { id: 1 })
        .build();

        input_map
            .insert_multiple([
                (GamepadButtonType::DPadUp, Up),
                (GamepadButtonType::DPadDown, Down),
                (GamepadButtonType::DPadLeft, Left),
                (GamepadButtonType::DPadRight, Right),
                (GamepadButtonType::South, Jump),
                (GamepadButtonType::West, Grab),
                (GamepadButtonType::RightTrigger, Crouch),
                (GamepadButtonType::RightTrigger2, CameraRight),
                (GamepadButtonType::LeftTrigger2, CameraLeft),
                (GamepadButtonType::Select, CameraMode),
            ])
            .insert(DualAxis::left_stick(), Move);

        InputListenerBundle {
            input_manager: InputManagerBundle {
                input_map,
                ..Default::default()
            },
        }
    }
}
