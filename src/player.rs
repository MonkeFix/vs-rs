use bevy::prelude::*;

use crate::steering::{SteeringBundle, SteeringHost};

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
) {
    let mut direction = Vec2::ZERO;
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
        host.seek(target);
    }
}
