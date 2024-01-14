use bevy::prelude::*;

mod tilemap;
mod movement;
mod player;

use tilemap::TileMapPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TileMapPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty()
        .insert(Camera2dBundle::default());
}
