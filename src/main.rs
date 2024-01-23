use bevy::prelude::*;

mod movement;
mod player;
mod tilemap;

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;
use tilemap::TileMapPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(TileMapPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(PlayerPlugin)
        .add_systems(Startup, spawn_camera);

    #[cfg(debug_assertions)]
    app.add_plugins(WorldInspectorPlugin::new());

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
