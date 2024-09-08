use bevy::{asset::LoadedFolder, prelude::*};

use crate::{
    enemies::EnemyConfig,
    prelude::TsxTilesetAsset,
    rooms::{MapAsset, RoomStore},
    tilesheets::AssetTileSheet,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub enum AssetLoadingState {
    #[default]
    LoadAssets,
    SetupAssets,
    Finished,
}

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AssetLoadingState>();
        app.insert_resource(RoomStore::default());
        app.insert_resource(Configs::default());
        app.add_systems(OnEnter(AssetLoadingState::LoadAssets), (start_loading,));
        app.add_systems(
            Update,
            check_asset_folders.run_if(in_state(AssetLoadingState::LoadAssets)),
        );
        app.add_systems(
            OnEnter(AssetLoadingState::SetupAssets),
            (setup_game_assets,),
        );
    }
}

#[derive(Default, Resource)]
pub struct GameAssets {
    pub tilesheet_main: AssetTileSheet,
    pub player_tilesheet: AssetTileSheet,
    pub capybara_texture: Handle<Image>,
    pub capybara_elite_texture: Handle<Image>,
    pub exp_gem_texture: Handle<Image>,
    pub money_texture: Handle<Image>,
}

#[derive(Default, Resource)]
pub struct UiAssets {
    pub health_bar: Handle<Image>,
    pub health_bar_outline: Handle<Image>,
}

#[derive(Default, Resource)]
pub struct Configs {
    pub enemy_config: Handle<EnemyConfig>,
}

#[derive(Default, Resource)]
pub struct GameAssetFolders {
    pub tiles_folder: Handle<LoadedFolder>,
    pub rooms_folder: Handle<LoadedFolder>,
    pub ui_folder: Handle<LoadedFolder>,
    pub tileset_main: Handle<TsxTilesetAsset>,
    pub tiles_loaded: bool,
    pub rooms_loaded: bool,
}

fn check_asset_folders(
    mut next_state: ResMut<NextState<AssetLoadingState>>,
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
        next_state.set(AssetLoadingState::SetupAssets);
    }
}

fn start_loading(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut configs: ResMut<Configs>,
) {
    info!("Loading game asset folders");
    let tiles_folder_handle = asset_server.load_folder("textures");
    let rooms_folder_handle = asset_server.load_folder("rooms");
    let ui_folder_handle = asset_server.load_folder("ui");
    let tileset_main = asset_server.load("tilesheet.tsx");

    configs.enemy_config = asset_server.load("configs/enemies.json");

    let asset_folders = GameAssetFolders {
        tiles_folder: tiles_folder_handle,
        rooms_folder: rooms_folder_handle,
        ui_folder: ui_folder_handle,
        tileset_main,
        ..default()
    };

    commands.insert_resource(asset_folders);

    info!("Finished loading game asset folders");
}

#[allow(clippy::too_many_arguments)]
fn setup_game_assets(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AssetLoadingState>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    rooms: Res<Assets<MapAsset>>,
    mut room_store: ResMut<RoomStore>,
    tilesets: Res<Assets<TsxTilesetAsset>>,
    folders: Res<GameAssetFolders>,
) {
    info!("Setting up game assets");

    let capybara_handle = asset_server
        .get_handle::<Image>("textures/capybara.png")
        .unwrap();
    let capybara_elite_handle = asset_server
        .get_handle::<Image>("textures/capybara_elite.png")
        .unwrap();
    let exp_gem_handle = asset_server
        .get_handle::<Image>("textures/exp_gem.png")
        .unwrap();
    let money_handle = asset_server
        .get_handle::<Image>("textures/money.png")
        .unwrap();

    let player_tilesheet = load_player(&asset_server, &mut layouts);

    let tileset = tilesets.get(folders.tileset_main.id()).unwrap();
    let tilesheet_main = AssetTileSheet::create_layout(
        &tileset.tileset,
        tileset.image_handle.as_ref().unwrap().clone_weak(),
        &mut layouts,
    );

    let game_assets = GameAssets {
        player_tilesheet,
        tilesheet_main,
        capybara_texture: capybara_handle,
        capybara_elite_texture: capybara_elite_handle,
        exp_gem_texture: exp_gem_handle,
        money_texture: money_handle,
    };

    commands.insert_resource(game_assets);

    let health_bar = asset_server
        .get_handle::<Image>("ui/health_bar.png")
        .unwrap();
    let health_bar_outline = asset_server
        .get_handle::<Image>("ui/health_bar_outline.png")
        .unwrap();

    let ui_assets = UiAssets {
        health_bar,
        health_bar_outline,
    };

    commands.insert_resource(ui_assets);

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
    next_state.set(AssetLoadingState::Finished);
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
