#![allow(dead_code)]

use crate::AppState;
use bevy::{asset::LoadedFolder, prelude::*};
use rooms::{MapAsset, RoomStore};
use tilesheets::AssetTileSheet;

pub mod prefabs;
pub mod rooms;
pub mod tilesheets;

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(RoomStore::default());
        app.add_systems(OnEnter(AppState::LoadAssets), (start_loading,));
        app.add_systems(
            Update,
            check_asset_folders.run_if(in_state(AppState::LoadAssets)),
        );
        app.add_systems(OnEnter(AppState::SetupAssets), (setup_game_assets,));
    }
}

#[derive(Default, Resource)]
pub struct GameAssets {
    pub tilesheet_main: AssetTileSheet,
    pub player_tilesheet: AssetTileSheet,
    pub capybara_texture: Handle<Image>,
    pub capybara_elite_texture: Handle<Image>,
}

#[derive(Default, Resource)]
pub struct GameAssetFolders {
    pub tiles_folder: Handle<LoadedFolder>,
    pub rooms_folder: Handle<LoadedFolder>,
    pub tiles_loaded: bool,
    pub rooms_loaded: bool,
}

fn check_asset_folders(
    mut next_state: ResMut<NextState<AppState>>,
    mut game_assets: ResMut<GameAssetFolders>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&game_assets.tiles_folder) {
            // TODO: Change to WorldGen
            //next_state.set(AppState::Finished);
            info!("Loaded game assets");
            game_assets.tiles_loaded = true;
        }
        if event.is_loaded_with_dependencies(&game_assets.rooms_folder) {
            info!("Loaded rooms");
            game_assets.rooms_loaded = true;
        }
    }

    if game_assets.tiles_loaded && game_assets.rooms_loaded {
        info!("Finished loading");
        next_state.set(AppState::SetupAssets);
    }
}

fn start_loading(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Loading game asset folders");
    let tiles_folder_handle = asset_server.load_folder("textures");
    let rooms_folder_handle = asset_server.load_folder("rooms");

    let asset_folders = GameAssetFolders {
        tiles_folder: tiles_folder_handle,
        rooms_folder: rooms_folder_handle,
        ..default()
    };

    commands.insert_resource(asset_folders);

    info!("Finished loading game asset folders");
}

fn setup_game_assets(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    rooms: Res<Assets<MapAsset>>,
    mut room_store: ResMut<RoomStore>,
) {
    info!("Setting up game assets");

    let capybara_handle = asset_server
        .get_handle::<Image>("textures/capybara.png")
        .unwrap();
    let capybara_elite_handle = asset_server
        .get_handle::<Image>("textures/capybara_elite.png")
        .unwrap();

    let player_tilesheet = load_player(&asset_server, &mut layouts);

    let tilesheet_main = AssetTileSheet::load_by_name("tilesheet", &asset_server, &mut layouts);

    let game_assets = GameAssets {
        player_tilesheet,
        tilesheet_main,
        capybara_texture: capybara_handle,
        capybara_elite_texture: capybara_elite_handle,
    };

    commands.insert_resource(game_assets);

    for (id, map) in rooms.iter() {
        info!(
            "Map id {} ({}x{})",
            map.map_id, map.map.width, map.map.height
        );

        let handle = asset_server
            .get_id_handle(id)
            .unwrap_or_else(|| panic!("no handle for MapAsset {:?}", id));

        room_store.insert(handle, map);
    }

    info!("Finished setting up game assets");
    next_state.set(AppState::Finished);
}

fn load_player(
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> AssetTileSheet {
    info!("Loading player");
    let texture_handle = asset_server.load("textures/player.png");

    let layout = TextureAtlasLayout::from_grid(UVec2::new(32, 64), 4, 2, None, None);
    let layout_handle = layouts.add(layout);

    AssetTileSheet {
        name: "player".to_string(),
        layout: layout_handle,
        image: texture_handle,
        named_tiles: None,
    }
}
