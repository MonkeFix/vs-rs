use bevy::{
    input::gamepad::{AxisSettings, GamepadSettings},
    prelude::*,
};

#[derive(Resource, Default, Debug, Reflect)]
pub struct PlayerControls {
    pub keyboard: PlayerControlsKeyboard,
    pub gamepad: PlayerControlsGamepad,
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct PlayerControlsKeyboard {
    // Primary
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    // Secondary
    pub move_up_2: KeyCode,
    pub move_down_2: KeyCode,
    pub move_left_2: KeyCode,
    pub move_right_2: KeyCode,
}

impl Default for PlayerControlsKeyboard {
    fn default() -> Self {
        Self {
            // Primary
            move_up: KeyCode::Up,
            move_down: KeyCode::Down,
            move_left: KeyCode::Left,
            move_right: KeyCode::Right,
            // Secondary
            move_up_2: KeyCode::W,
            move_down_2: KeyCode::S,
            move_left_2: KeyCode::A,
            move_right_2: KeyCode::D,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct PlayerControlsGamepad {
    // Primary
    pub move_axis_x: GamepadAxisType,
    pub move_axis_y: GamepadAxisType,
    // Secondary
    pub move_axis_x_2: GamepadAxisType,
    pub move_axis_y_2: GamepadAxisType,
}

impl Default for PlayerControlsGamepad {
    fn default() -> Self {
        Self {
            move_axis_x: GamepadAxisType::LeftStickX,
            move_axis_y: GamepadAxisType::LeftStickY,
            move_axis_x_2: GamepadAxisType::RightStickX,
            move_axis_y_2: GamepadAxisType::RightStickY,
        }
    }
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerControls::default());
        app.add_systems(
            Startup,
            (setup_gamepad_axis_settings, setup_player_controls),
        );
    }
}

fn setup_gamepad_axis_settings(mut gamepad_settings: ResMut<GamepadSettings>) {
    let settings = AxisSettings::new(-1.0, -0.15, 0.15, 1.0, 0.1);
    let settings = settings.unwrap();

    gamepad_settings.default_axis_settings = settings.clone();
}

fn setup_player_controls(mut controls: ResMut<PlayerControls>) {
    controls.gamepad = PlayerControlsGamepad::default();
    controls.keyboard = PlayerControlsKeyboard::default();
}
