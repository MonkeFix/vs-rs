use crate::prelude::*;
use bevy_simple_tilemap::{prelude::TileMapBundle, Tile, TileMap};
use worldgen::{
    generation::{settings::WorldGeneratorSettings, WorldGenerator},
    world::World,
};

use crate::AppState;

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

fn spawn_world(
    mut commands: Commands,
    assets: Res<GameAssets>,
    room_store: Res<RoomStore>,
    map_assets: Res<Assets<MapAsset>>,
    mut collider_store: ResMut<ColliderStore>,
) {
    let mut world_gen = WorldGenerator::new(&room_store);

    let settings = WorldGeneratorSettings {
        world_width: 256,
        world_height: 256,
        ..default()
    };

    let world_comp = WorldComponent {
        world: world_gen.generate(settings),
    };

    let lower_tilemap = world_to_tilemap(&world_comp.world, &assets.tilesheet_main, &map_assets);

    let x = -(((world_comp.world.width * 32) / 2) as f32);
    let y = -(((world_comp.world.height * 32) / 2) as f32);

    world_comp.world.add_colliders(
        &map_assets,
        "collision",
        "collision_fine",
        &mut collider_store,
        Vec2::new(x, y),
    );

    let mut upper_tilemap = TileMap::default();

    world_comp
        .world
        .fill_tilemap(&mut upper_tilemap, &map_assets, "upper_group");

    commands
        .spawn((
            world_comp,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
        ))
        .with_children(|c| {
            c.spawn(create_tile_map_bundle(
                &assets.tilesheet_main,
                lower_tilemap,
                Vec3::new(x, y, 0.0),
            ));
            c.spawn(create_tile_map_bundle(
                &assets.tilesheet_main,
                upper_tilemap,
                Vec3::new(x, y, 10.0),
            ));
        });
}

fn create_tile_map_bundle(
    grass_asset: &AssetTileSheet,
    tilemap: TileMap,
    translation: Vec3,
) -> TileMapBundle {
    TileMapBundle {
        texture: grass_asset.image.clone(),
        tilemap,
        atlas: TextureAtlas {
            layout: grass_asset.layout.clone(),
            ..default()
        },
        transform: Transform {
            scale: Vec3::splat(1.0),
            translation,
            ..default()
        },
        ..default()
    }
}

fn world_to_tilemap(
    world: &World,
    tile_sheet: &AssetTileSheet,
    assets: &Assets<MapAsset>,
) -> TileMap {
    let mut tilemap = TileMap::default();

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

    world.fill_tilemap(&mut tilemap, assets, "lower_group");

    tilemap
}
