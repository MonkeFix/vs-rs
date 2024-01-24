use bevy::{input::gamepad::GamepadSettings, prelude::*};

use crate::{
    input::PlayerControls,
    steering::{SteerSeek, SteeringBundle, SteeringHost},
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, movement);
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct Health(u16);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    health: Health,
}

impl PlayerBundle {
    fn new() -> Self {
        Self {
            player: Player,
            health: Health(100),
        }
    }
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("player.png");
    commands.spawn((
        PlayerBundle::new(),
        SpriteBundle {
            texture: texture_handle,
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..default()
        },
        SteeringBundle {
            host: SteeringHost::default(),
        },
        Name::new("player"),
    ));
}

fn movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut steering_host: Query<&mut SteeringHost, With<Player>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_settings: Res<GamepadSettings>,
    gamepads: Res<Gamepads>,
    controls: Res<PlayerControls>,
) {
    let mut direction = Vec2::ZERO;

    let dead_upper = gamepad_settings.default_axis_settings.deadzone_upperbound();
    let dead_lower = gamepad_settings.default_axis_settings.deadzone_lowerbound();

    for gamepad in gamepads.iter() {
        let x1 = gamepad_axes
            .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_x))
            .unwrap();
        let y1 = gamepad_axes
            .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_y))
            .unwrap();

        let x2 = gamepad_axes
            .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_x_2))
            .unwrap();
        let y2 = gamepad_axes
            .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_y_2))
            .unwrap();

        let x = (x1 + x2) / 2.0;
        let y = (y1 + y2) / 2.0;

        if x > dead_lower || x < dead_upper {
            direction.x = x;
        }
        if y > dead_lower || y < dead_upper {
            direction.y = y;
        }
    }

    if keyboard_input.pressed(controls.keyboard.move_left)
        || keyboard_input.pressed(controls.keyboard.move_left_2)
    {
        direction -= Vec2::new(1.0, 0.0);
    }
    if keyboard_input.pressed(controls.keyboard.move_right)
        || keyboard_input.pressed(controls.keyboard.move_right_2)
    {
        direction += Vec2::new(1.0, 0.0);
    }
    if keyboard_input.pressed(controls.keyboard.move_up)
        || keyboard_input.pressed(controls.keyboard.move_up_2)
    {
        direction += Vec2::new(0.0, 1.0);
    }
    if keyboard_input.pressed(controls.keyboard.move_down)
        || keyboard_input.pressed(controls.keyboard.move_down_2)
    {
        direction -= Vec2::new(0.0, 1.0);
    }

    let direction = direction.normalize_or_zero();

    if let Ok(mut host) = steering_host.get_single_mut() {
        let target = host.position + direction;
        host.steer(SteerSeek, &target);
    }
}
