use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build (&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, follow_camera);
    }
}

#[derive(Component)]
struct Player;

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

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let texture_handle: Handle<Image> = asset_server.load("player.png");
    commands.spawn((
        PlayerBundle::new(),
        SpriteBundle{
            texture: texture_handle,
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..default()
        }
    ));
}

fn follow_camera(
    mut player: Query<&mut Transform, With<Player>>,
    camera: Query<&Transform, (With<Camera>, Without<Player>)>,
){
    if let Ok(camera_transform) = camera.get_single() {
        if let Ok(mut player_transform) = player.get_single_mut() {
            player_transform.translation = camera_transform.translation;
            player_transform.translation.z = 1.;
        }
    }
}
