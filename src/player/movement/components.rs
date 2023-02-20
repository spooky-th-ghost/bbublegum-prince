use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Jump {
    pub input_timer: Timer,
    pub jump_stage: JumpStage,
    pub jump_buffered: bool,
}

impl Jump {
    fn reset(&mut self) {
        self.reset_jump_stage();
        self.reset_input();
    }

    pub fn reset_jump_stage(&mut self) {
        self.jump_stage = JumpStage::Single;
    }

    pub fn reset_input(&mut self) {
        self.jump_buffered = false;
        self.input_timer.reset();
    }

    pub fn update(&mut self, delta: std::time::Duration) {
        if self.jump_buffered {
            self.input_timer.tick(delta);
            if self.input_timer.finished() {
                self.reset();
            }
        }
    }

    pub fn get_jump_force(&mut self) -> Option<f32> {
        if self.jump_buffered {
            self.reset_input();
            match self.jump_stage {
                JumpStage::Single => {
                    self.jump_stage = JumpStage::Double;
                    Some(10.0)
                }
                JumpStage::Double => {
                    self.jump_stage = JumpStage::Triple;
                    Some(15.0)
                }
                JumpStage::Triple => {
                    self.jump_stage = JumpStage::Single;
                    Some(20.0)
                }
            }
        } else {
            None
        }
    }

    pub fn get_wall_jump_force(&mut self) -> f32 {
        self.reset_input();
        15.0
    }

    pub fn buffer_jump(&mut self) {
        self.jump_buffered = true;
        self.input_timer.reset();
    }
}

impl Default for Jump {
    fn default() -> Self {
        Jump {
            input_timer: Timer::from_seconds(0.2, TimerMode::Once),
            jump_stage: JumpStage::Single,
            jump_buffered: false,
        }
    }
}

#[derive(Default, Reflect)]
pub enum JumpStage {
    #[default]
    Single,
    Double,
    Triple,
}

#[derive(Component)]
pub struct Coyote(Timer);

impl Coyote {
    pub fn new() -> Self {
        Coyote(Timer::from_seconds(0.2, TimerMode::Once))
    }
    pub fn tick(&mut self, delta: std::time::Duration) {
        self.0.tick(delta);
    }

    pub fn finished(&self) -> bool {
        self.0.finished()
    }
}

#[derive(Component, Default)]
pub struct Grounded;

#[derive(Component, Default)]
pub struct Walljump(pub Vec3);

#[derive(Component, Default)]
pub struct LedgeGrab(pub Vec3);

#[derive(Component)]
pub struct PlayerWallSensor;

#[derive(Component)]
pub struct PlayerLedgeSensor;

#[derive(Component, Default)]
pub struct Drift(pub Vec3);

impl Drift {
    pub fn has_drift(&self) -> bool {
        self.0 != Vec3::ZERO
    }

    pub fn reset(&mut self) {
        self.0 = Vec3::ZERO;
    }

    pub fn set(&mut self, drift: Vec3) {
        self.0 = drift;
    }

    pub fn add(&mut self, drift: Vec3) {
        self.0 += drift;
    }
}
