use bevy::prelude::*;
use bevy::utils::*;

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build (&self, app: &mut App) {
        app.insert_resource(ChunkManager::default())
            .add_systems(
                Update,
                (
                    spawn_chunks_around_camera,
                    despawn_outofrange_chunks,
                )
            );
    }
}

#[derive(Default, Debug, Resource)]
struct ChunkManager {
    spawned_chunks: HashSet<IVec2>,
}

#[derive(Component)]
struct TileSize { x: f32, y: f32 }

const TILE_SIZE: TileSize = TileSize { x: 128.0, y: 128.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };

fn spawn_chunk(commands: &mut Commands, asset_server: &AssetServer, chunk_pos: IVec2) {
    let texture_handle: Handle<Image> = asset_server.load("tile.png");
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let transform = Transform::from_translation(Vec3::new(
                chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x + x as f32 * TILE_SIZE.x,
                chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y + y as f32 * TILE_SIZE.y,
                0.0,
            ));
            commands.spawn(SpriteBundle{
                texture: texture_handle.clone(),
                transform,
                ..default()
            });
        }
    }

    // Next lines of code are about my try into grouping up tiles for easier debugging
    //
    // let texture_handle: Handle<Image> = asset_server.load("tile.png");
    //
    // commands.spawn(SpatialBundle::default())
    //     .with_children(|entity|{
    //         for x in 0..CHUNK_SIZE.x {
    //             for y in 0..CHUNK_SIZE.y {
    //                 let transform = Transform::from_translation(Vec3::new(
    //                     chunk_pos.x as f32 * CHUNK_SIZE.x as f32 * TILE_SIZE.x + x as f32 * TILE_SIZE.x,
    //                     chunk_pos.y as f32 * CHUNK_SIZE.y as f32 * TILE_SIZE.y + y as f32 * TILE_SIZE.y,
    //                     0.0,
    //                 ));
    //                 entity.spawn(SpriteBundle{
    //                     texture: texture_handle.clone(),
    //                     transform,
    //                     ..default()
    //                 });
    //             }
    //         }
    //     });
}

fn camera_pos_to_chunk_pos(camera_pos: &Vec2) -> IVec2 {
    let camera_pos = camera_pos.as_ivec2();
    let chunk_size: IVec2 = IVec2::new(CHUNK_SIZE.x as i32, CHUNK_SIZE.y as i32);
    let tile_size: IVec2 = IVec2::new(TILE_SIZE.x as i32, TILE_SIZE.y as i32);
    camera_pos / (chunk_size * tile_size)
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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
                    spawn_chunk(&mut commands, &asset_server, IVec2::new(x, y));
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
    for camera_transform in camera_query.iter() {
        for (entity, chunk_transform) in chunks_query.iter() {
            let chunk_pos = chunk_transform.translation.xy();
            let distance = camera_transform.translation.xy().distance(chunk_pos);
            if distance > 3200.0 { // TODO: set the distance threshold to some reasonable value
                let x = (chunk_pos.x / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32;
                let y = (chunk_pos.y / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32;
                chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
