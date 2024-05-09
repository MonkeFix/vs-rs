use crate::assets::GameAssets;
use crate::AppState;
use bevy::prelude::*;
use bevy::utils::*;
use bevy_simple_tilemap::prelude::*;
use rand::thread_rng;
use rand::Rng;

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        //app.insert_resource(ChunkManager::default())
        //    .add_systems(Update, (spawn_chunk).run_if(in_state(AppState::Finished)));
        //app.add_plugins(SimpleTileMapPlugin);
        //app.add_systems(OnEnter(AppState::Finished), (spawn_chunk,));
    }
}

#[derive(Default, Debug, Resource)]
struct ChunkManager {
    spawned_chunks: HashSet<IVec2>,
}

#[derive(Component)]
struct TileSize {
    x: f32,
    y: f32,
}

const TILE_SIZE: TileSize = TileSize { x: 32.0, y: 32.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 16, y: 16 };

fn spawn_chunk(mut commands: Commands, assets: Res<GameAssets>) {
    let grass_asset = assets.tilesets.get("grass.png").unwrap();
    let mut rng = thread_rng();

    let mut tilemap = TileMap::default();

    let mut tiles = vec![];

    for x in 0..32 {
        for y in 0..32 {
            tiles.push((
                IVec3::new(x as i32, y as i32, 0),
                Some(Tile {
                    sprite_index: rng.gen_range(0..64),
                    ..default()
                }),
            ));
        }
    }

    tilemap.set_tiles(tiles);

    let tilemap_bundle = TileMapBundle {
        texture: grass_asset.image.clone(),
        tilemap,
        atlas: TextureAtlas {
            layout: grass_asset.layout.clone(),
            ..default()
        },
        transform: Transform {
            scale: Vec3::splat(2.0),
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..default()
        },
        ..default()
    };
    commands.spawn(tilemap_bundle);
}

/* fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    assets: Res<GameAssets>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = camera_pos_to_chunk_pos(&transform.translation.xy());
        // TODO: set xs and ys to some reasonable values
        for y in (camera_chunk_pos.y - 3)..(camera_chunk_pos.y + 3) {
            for x in (camera_chunk_pos.x - 3)..(camera_chunk_pos.x + 3) {
                if !chunk_manager.spawned_chunks.contains(&IVec2::new(x, y)) {
                    chunk_manager.spawned_chunks.insert(IVec2::new(x, y));
                    spawn_chunk(&mut commands, &assets, IVec2::new(x, y));
                }
            }
        }
    }
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    chunks_query: Query<(Entity, &Transform)>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    /* for camera_transform in camera_query.iter() {
        for (entity, chunk_transform) in chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.xy();
            let distance = camera_transform.translation.xy().distance(chunk_pos);
            if distance > 3200.0 {
                // TODO: set the distance threshold to some reasonable value
                let x = (chunk_pos.x / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32;
                let y = (chunk_pos.y / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32;
                chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                commands.entity(entity).despawn_recursive();
            }
        }
    } */
}
 */
