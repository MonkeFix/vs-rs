use bevy::{input::gamepad::GamepadSettings, prelude::*};

use crate::steering::{SteerSeek, SteeringBundle, SteeringHost};

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
) {
    let mut direction = Vec2::ZERO;

    let dead_upper = gamepad_settings.default_axis_settings.deadzone_upperbound();
    let dead_lower = gamepad_settings.default_axis_settings.deadzone_lowerbound();

    for gamepad in gamepads.iter() {
        let x = gamepad_axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();
        let y = gamepad_axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickY))
            .unwrap();

        if x > dead_lower || x < dead_upper {
            direction.x = x;
        }
        if y > dead_lower || y < dead_upper {
            direction.y = y;
        }
    }

    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        direction -= Vec2::new(1.0, 0.0);
    }
    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        direction += Vec2::new(1.0, 0.0);
    }
    if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W) {
        direction += Vec2::new(0.0, 1.0);
    }
    if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S) {
        direction -= Vec2::new(0.0, 1.0);
    }

    let direction = direction.normalize_or_zero();

    if let Ok(mut host) = steering_host.get_single_mut() {
        let target = host.position + direction;
        host.steer(SteerSeek, &target);
    }
}
