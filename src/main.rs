use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, spawn_camera);
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty()
        .insert(Camera2dBundle::default());
}
