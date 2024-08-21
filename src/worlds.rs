use bevy::prelude::*;
use bevy_simple_tilemap::{prelude::TileMapBundle, Tile, TileMap};

use crate::{
    assets::{
        rooms::{MapAsset, RoomStore},
        tilesheets::AssetTileSheet,
        GameAssets,
    },
    collisions::store::ColliderStore,
    AppState,
};

use self::{
    world::{CellType, World},
    worldgen::{settings::WorldGeneratorSettings, WorldGenerator},
};

pub mod bitmasking;
pub mod world;
pub mod worldgen;

#[derive(Component)]
pub struct WorldComponent {
    pub world: World,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        //app.add_systems(OnEnter(AppState::WorldGen), (spawn_world,));
        app.add_systems(OnEnter(AppState::Finished), (spawn_world,));
        app.add_systems(Update, (debug_draw,).run_if(in_state(AppState::Finished)));
    }
}

fn debug_draw(mut gizmos: Gizmos, collider_store: Res<ColliderStore>) {
    collider_store.debug_draw(&mut gizmos);
}

fn spawn_world(
    mut commands: Commands,
    assets: Res<GameAssets>,
    room_store: Res<RoomStore>,
    map_assets: Res<Assets<MapAsset>>,
    mut collider_store: ResMut<ColliderStore>,
    gizmos: Gizmos,
) {
    let mut world_gen = WorldGenerator::default();

    let settings = WorldGeneratorSettings {
        world_width: 256,
        world_height: 256,
        ..default()
    };

    let world_comp = WorldComponent {
        world: world_gen.generate(settings, &room_store),
    };

    let grass_asset = &assets.tilesheet_main;

    let tilemap = world_to_tilemap(&world_comp.world, grass_asset, &map_assets);

    let x = -(((world_comp.world.width * 32) / 2) as f32);
    let y = -(((world_comp.world.height * 32) / 2) as f32);

    world_comp.world.add_colliders(
        &map_assets,
        "collision",
        &mut collider_store,
        Vec2::new(x, y),
    );

    commands.spawn((
        world_comp,
        TileMapBundle {
            texture: grass_asset.image.clone(),
            tilemap,
            atlas: TextureAtlas {
                layout: grass_asset.layout.clone(),
                ..default()
            },
            transform: Transform {
                scale: Vec3::splat(1.0),
                translation: Vec3::new(x, y, 0.0),
                ..default()
            },
            ..default()
        },
    ));
}

fn world_to_tilemap(
    world: &World,
    tile_sheet: &AssetTileSheet,
    assets: &Assets<MapAsset>,
) -> TileMap {
    let mut tilemap = TileMap::default();

    // TODO: Spawn 3 separate tile maps for 3 different layers:
    // - lower level (characters are over tiles)
    // - upper level (characters are behind tiles)
    // - shadows

    for y in 0..world.height {
        for x in 0..world.width {
            let pos = IVec3::new(x as i32, y as i32, 0);

            tilemap.set_tile(
                pos,
                Some(Tile {
                    sprite_index: tile_sheet.get_random_tile_id("grass_decorated").unwrap(),
                    ..default()
                }),
            );
        }
    }

    world.fill_tilemap(&mut tilemap, assets);

    tilemap
}
