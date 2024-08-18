use bevy::prelude::*;
use bevy_simple_tilemap::{prelude::TileMapBundle, Tile, TileMap};

use crate::{
    assets::{rooms::RoomStore, tilesheets::AssetTileSheet, GameAssets},
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
    }
}

fn spawn_world(mut commands: Commands, assets: Res<GameAssets>, room_store: Res<RoomStore>) {
    let mut world_gen = WorldGenerator::default();

    let mut settings = WorldGeneratorSettings::default();
    settings.world_width = 256;
    settings.world_height = 256;

    let world_comp = WorldComponent {
        world: world_gen.generate(settings, &room_store),
    };

    let grass_asset = &assets.tilesheet_main;

    let tilemap = world_to_tilemap(&world_comp.world, grass_asset);

    let x = -(((world_comp.world.width * 32) / 2) as f32);
    let y = -(((world_comp.world.height * 32) / 2) as f32);

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

fn world_to_tilemap(world: &World, tile_sheet: &AssetTileSheet) -> TileMap {
    let mut tilemap = TileMap::default();

    world.fill_tilemap(&mut tilemap);

    for y in 0..world.height {
        for x in 0..world.width {
            let pos = IVec3::new(x as i32, y as i32, 0);

            for layer in &world.layers {
                match layer.1.data[y][x] {
                    CellType::None => {
                        tilemap.set_tile(
                            pos,
                            Some(Tile {
                                sprite_index: tile_sheet
                                    .get_random_tile_id("grass_decorated")
                                    .unwrap(),
                                ..default()
                            }),
                        );
                    }
                    CellType::Room => {
                        tilemap.set_tile(
                            pos,
                            Some(Tile {
                                sprite_index: tile_sheet
                                    .get_random_tile_id("grass_decorated")
                                    .unwrap(),
                                ..default()
                            }),
                        );
                    }
                    CellType::Hallway => {
                        tilemap.set_tile(
                            pos,
                            Some(Tile {
                                sprite_index: tile_sheet.get_random_tile_id("grass_road").unwrap(),
                                ..default()
                            }),
                        );
                    }
                    CellType::Wall => {
                        tilemap.set_tile(
                            pos,
                            Some(Tile {
                                sprite_index: 355, //tile_sheet.get_random_tile_id("grass_road").unwrap(),
                                ..default()
                            }),
                        );
                    }
                }
            }

            if x == 0 || y == 0 || x == world.width - 1 || y == world.height - 1 {
                tilemap.set_tile(
                    pos,
                    Some({
                        Tile {
                            sprite_index: 355,
                            ..default()
                        }
                    }),
                );
            }
        }
    }

    tilemap
}
