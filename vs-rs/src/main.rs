use assets::{
    rooms::{MapAsset, MapAssetLoader},
    tilesheets::{TsxTilesetAsset, TsxTilesetAssetLoader},
};
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::gamepad::{AxisSettings, GamepadSettings},
    prelude::*,
};
use bevy_simple_tilemap::plugin::SimpleTileMapPlugin;
use collisions::plugin::CollisionPlugin;
use worlds::WorldPlugin;

mod camera;
mod collisions;
mod debug;
mod input;
mod math;
mod movement;
mod player;

mod assets;
mod enemy;
mod stats;
mod worlds;

use crate::assets::GameAssetsPlugin;
use crate::enemy::EnemyPlugin;
use camera::CameraMovementPlugin;
#[cfg(debug_assertions)]
use debug::DebugPlugin;
use movement::steering::SteeringPlugin;
use player::PlayerPlugin;

pub const FRAMERATE: f64 = 60.0;
pub const FIXED_TIMESTEP: f64 = 1.0 / FRAMERATE;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AppState {
    #[default]
    LoadAssets,
    SetupAssets,
    WorldGen,
    Finished,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "VS-RS".into(),
                    resolution: (1600., 900.).into(),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    .init_state::<AppState>()
    .add_plugins(FrameTimeDiagnosticsPlugin)
    .add_plugins(SimpleTileMapPlugin)
    .add_plugins(input::InputPlugin)
    .add_plugins(CameraMovementPlugin)
    .add_plugins(PlayerPlugin)
    .add_plugins(SteeringPlugin)
    .add_plugins(EnemyPlugin)
    .add_plugins(CollisionPlugin)
    .add_systems(Startup, (spawn_camera, setup_gamepad))
    .add_plugins(GameAssetsPlugin)
    .add_plugins(WorldPlugin)
    .insert_resource(Time::<Fixed>::from_seconds(FIXED_TIMESTEP))
    .insert_resource(Msaa::Off)
    .init_asset::<MapAsset>()
    .init_asset::<TsxTilesetAsset>()
    .init_asset_loader::<MapAssetLoader>()
    .init_asset_loader::<TsxTilesetAssetLoader>();

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
