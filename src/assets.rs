#![allow(dead_code)]

use crate::AppState;
use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use bevy::render::texture::ImageSampler;
use bevy::utils::HashMap;

// TODO: maybe https://bevy-cheatbook.github.io/assets/ready.html will be useful later
pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Setup), (load,));
        app.add_systems(Update, check_textures.run_if(in_state(AppState::Setup)));
        //app.add_systems(OnEnter(AppState::Finished), (spawn_test,));
    }
}

pub struct GameAssetTileset {
    pub name: String,
    pub layout: Handle<TextureAtlasLayout>,
    pub image: Handle<Image>,
}

#[derive(Default, Resource)]
pub struct GameAssets {
    pub tilesets: HashMap<String, GameAssetTileset>,
    pub tiles_folder: Handle<LoadedFolder>,
    pub player_texture: Handle<Image>,
    pub test_tile_texture: Handle<Image>,
    pub capybara_texture: Handle<Image>,
    pub capybara_elite_texture: Handle<Image>,
}

fn check_textures(
    mut next_state: ResMut<NextState<AppState>>,
    game_assets: Res<GameAssets>,
    mut events: EventReader<AssetEvent<LoadedFolder>>,
) {
    for event in events.read() {
        if event.is_loaded_with_dependencies(&game_assets.tiles_folder) {
            next_state.set(AppState::Finished);
            info!("Loaded game assets");
        }
    }
}

fn load(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    info!("Loading game assets");

    let folder_handle = asset_server.load_folder(".");

    let player_handle = asset_server.load("player.png");
    let tile_handle = asset_server.load("tile.png");
    let capybara_handle = asset_server.load("map_1/capybara.png");
    let capybara_elite_handle = asset_server.load("map_1/capybara_elite.png");

    let mut game_assets = GameAssets {
        tiles_folder: folder_handle,
        tilesets: HashMap::new(),
        player_texture: player_handle,
        test_tile_texture: tile_handle,
        capybara_texture: capybara_handle,
        capybara_elite_texture: capybara_elite_handle,
    };

    // grass
    load_tileset(
        "grass.png",
        &asset_server,
        &mut atlases,
        &mut game_assets,
        Vec2::new(32., 32.),
        8,
        8,
    );
    // stone ground
    load_tileset(
        "stone_ground.png",
        &asset_server,
        &mut atlases,
        &mut game_assets,
        Vec2::new(32., 32.),
        8,
        8,
    );
    // walls
    load_tileset(
        "walls.png",
        &asset_server,
        &mut atlases,
        &mut game_assets,
        Vec2::new(32., 32.),
        9,
        8,
    );
    // structs
    load_tileset(
        "structs.png",
        &asset_server,
        &mut atlases,
        &mut game_assets,
        Vec2::new(32., 32.),
        9,
        6,
    );
    // props
    load_tileset(
        "props.png",
        &asset_server,
        &mut atlases,
        &mut game_assets,
        Vec2::new(32., 32.),
        11,
        11,
    );
    // plants
    load_tileset(
        "plants.png",
        &asset_server,
        &mut atlases,
        &mut game_assets,
        Vec2::new(32., 32.),
        11,
        7,
    );

    // player
    load_tileset(
        "player.png",
        &asset_server,
        &mut atlases,
        &mut game_assets,
        Vec2::new(32., 64.),
        4,
        2,
    );

    commands.insert_resource(game_assets);
}

fn load_tileset(
    name: &str,
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    game_assets: &mut GameAssets,
    tile_size: Vec2,
    columns: usize,
    rows: usize,
) {
    info!("Loading tileset {}", name);
    let texture_handle = asset_server.load(format!("tiles/{}", name));

    let layout = TextureAtlasLayout::from_grid(tile_size, columns, rows, None, None);
    let layout_handle = layouts.add(layout);

    let set = GameAssetTileset {
        name: name.to_string(),
        layout: layout_handle,
        image: texture_handle,
    };

    game_assets.tilesets.insert(name.to_string(), set);
}

/*fn spawn_test(mut commands: Commands, game_assets: Res<GameAssets>) {
    commands.spawn(
        (SpriteSheetBundle {
            sprite: Sprite::default(),
            atlas: TextureAtlas {
                layout: game_assets.layout.clone(),
                index: 10,
            },
            texture: game_assets.image.clone(),
            ..default()
        }),
    );
}
*/
