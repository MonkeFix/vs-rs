use bevy::{
    input::gamepad::{AxisSettings, GamepadSettings},
    prelude::*,
};

mod camera;
mod input;
mod math;
mod player;
mod steering;
mod tilemap;

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::CameraMovementPlugin;
use player::PlayerPlugin;
use steering::{SteeringHost, SteeringPlugin};
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
    .add_plugins(input::InputPlugin)
    .add_plugins(TileMapPlugin)
    .add_plugins(CameraMovementPlugin)
    .add_plugins(PlayerPlugin)
    .add_plugins(SteeringPlugin)
    .add_systems(Startup, (spawn_camera, setup_gamepad))
    .register_type::<SteeringHost>();

    #[cfg(debug_assertions)]
    app.add_plugins(WorldInspectorPlugin::new());
    #[cfg(debug_assertions)]
    app.add_systems(Update, bevy::window::close_on_esc);

    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_gamepad(mut gamepad_settings: ResMut<GamepadSettings>) {
    let settings = AxisSettings::new(-1.0, -0.15, 0.15, 1.0, 0.1);
    let settings = settings.unwrap();

    gamepad_settings.default_axis_settings = settings.clone();
}
