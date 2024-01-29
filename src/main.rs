use bevy::{
    input::gamepad::{AxisSettings, GamepadSettings},
    prelude::*,
};

mod camera;
mod collisions;
mod debug;
mod input;
mod math;
mod player;
mod steering;
mod tilemap;

mod enemy;
mod stats;

use crate::enemy::EnemyPlugin;
use camera::CameraMovementPlugin;
use collisions::colliders::CollisionPlugin;
#[cfg(debug_assertions)]
use debug::DebugPlugin;
use player::PlayerPlugin;
use steering::SteeringPlugin;
use tilemap::TileMapPlugin;

pub const FIXED_TIMESTEP: f64 = 1.0 / 60.0;

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
    .add_plugins(EnemyPlugin)
    .add_plugins(CollisionPlugin)
    .add_systems(Startup, (spawn_camera, setup_gamepad))
    .insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP));

    #[cfg(debug_assertions)]
    app.add_plugins(DebugPlugin);

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
