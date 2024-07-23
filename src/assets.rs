#![allow(dead_code)]

use crate::AppState;
use bevy::{asset::LoadedFolder, prelude::*, utils::HashMap};
use serde::Deserialize;
use tilesheets::AssetTileSheet;

pub mod rooms;
pub mod tilesheets;

// TODO: maybe https://bevy-cheatbook.github.io/assets/ready.html will be useful later
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Setup), (start_loading,));
        app.add_systems(
            Update,
            check_asset_folders.run_if(in_state(AppState::Setup)),
        );
        //app.add_systems(OnEnter(AppState::Finished), (spawn_test,));
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
    commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut game_assets: ResMut<GameAssetFolders>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
    asset_server: Res<AssetServer>,
    layouts: ResMut<Assets<TextureAtlasLayout>>,
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
        setup_game_assets(commands, asset_server, layouts);
        next_state.set(AppState::Finished);
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
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    info!("Loading game assets");

    let capybara_handle = asset_server
        .get_handle::<Image>("textures/capybara.png")
        .unwrap();
    let capybara_elite_handle = asset_server
        .get_handle::<Image>("textures/capybara_elite.png")
        .unwrap();

    let player_tilesheet = load_player(&asset_server, &mut layouts);

    let tilesheet_main = load_tsx_tileset("tilesheet", &asset_server, &mut layouts);

    let game_assets = GameAssets {
        player_tilesheet,
        tilesheet_main,
        capybara_texture: capybara_handle,
        capybara_elite_texture: capybara_elite_handle,
    };

    commands.insert_resource(game_assets);

    let mut loader = tiled::Loader::new();
    let obj_bench = loader.load_tmx_map("assets/map_example.tmx").unwrap();

    for layer in obj_bench.layers() {
        let ltype = layer.layer_type();
        match ltype {
            tiled::LayerType::Tiles(tiles) => {}
            tiled::LayerType::Objects(objects) => {}
            tiled::LayerType::Image(images) => {}
            tiled::LayerType::Group(group) => {}
        }
    }
    /* let obj_bench = obj_bench.get_layer(0).unwrap().as_tile_layer().unwrap();

    let w = obj_bench.width().unwrap() as i32;
    let h = obj_bench.height().unwrap() as i32;

    for y in 0..h {
        for x in 0..w {
            let tile = obj_bench.get_tile(x, y).unwrap();
            dbg!(&tile.id());
        }
    } */

    //let json = std::fs::read_to_string("assets/obj_bench.json").unwrap();
    //let obj_bench = serde_json::from_str::<TiledPrefab>(&json).unwrap();
    //dbg!(&obj_bench);
}

#[derive(Debug, Deserialize)]
struct TiledPrefab {
    pub layers: Vec<TiledPrefabLayer>,
    pub tileheight: u32,
    pub tilewidth: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
struct TiledPrefabLayer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub opacity: f32,
    pub data: Vec<u32>,
}

fn load_tsx_tileset(
    name: &str,
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> AssetTileSheet {
    info!("Loading tilesheet '{}.tsx'", name);

    let mut loader = tiled::Loader::new();
    let tilesheet = loader
        .load_tsx_tileset(format!("assets/{}.tsx", name))
        .unwrap_or_else(|_| panic!("could not read file '{}'.tsx", name));

    // Setting up named tiles (tiles with non-empty type described in the tile sheet)
    let mut named_tiles: HashMap<String, Vec<u32>> = HashMap::new();

    for (i, tile) in tilesheet.tiles() {
        if let Some(ut) = &tile.user_type {
            if named_tiles.contains_key(ut) {
                named_tiles.get_mut(ut).unwrap().push(i);
            } else {
                named_tiles.insert(ut.to_string(), vec![i]);
            }
        }
    }

    //dbg!(&named_tiles);

    let img = tilesheet.image.expect("Image must not be empty");

    // tilesheet name and texture name must match, and we're not just taking img.source
    // because tsx loader fucks up the path from being 'assets/textures/a.png'
    // to 'assets/assets/textures/a.png'
    let texture_handle = asset_server.load(format!("textures/{}.png", name));

    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(tilesheet.tile_width, tilesheet.tile_height),
        tilesheet.columns,
        img.height as u32 / tilesheet.tile_height,
        None,
        None,
    );
    let layout_handle = layouts.add(layout);

    AssetTileSheet {
        name: name.to_string(),
        layout: layout_handle,
        image: texture_handle,
        named_tiles: Some(named_tiles),
    }
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
