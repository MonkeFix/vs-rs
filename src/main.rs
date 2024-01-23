use bevy::prelude::*;

mod movement;
mod player;
mod tilemap;

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use movement::MovementPlugin;
use player::PlayerPlugin;
use tilemap::TileMapPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "VS-RS".into(),
            resolution: (1600., 900.).into(),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
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
