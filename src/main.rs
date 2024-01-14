use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ChunkManager::default())
        .add_systems(Startup, (spawn_camera, spawn_player))
        .add_systems(
            Update,
            (
                // project_positions,
                movement,
                spawn_chunks_around_camera,
                // handle_player_input,
                move_player.after(handle_player_input),
                despawn_outofrange_chunks,
            )
        )
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_empty()
        .insert(Camera2dBundle::default());
}

const PLAYER_SIZE: f32 = 5.;
const PLAYER_SPEED: f32 = 1.;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Health(u16);

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    health: Health,
    position: Position,
    velocity: Velocity,
}

impl PlayerBundle {
    fn new() -> Self {
        Self {
            player: Player,
            health: Health(100),
            position: Position(Vec2::new(0., 0.)),
            velocity: Velocity(Vec2::new(0., 0.)),
        }
    }
}

#[derive(Component)]
struct Position(Vec2);

#[derive(Component)]
struct Velocity(Vec2);

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_handle = meshes.add(Mesh::from(shape::Circle::new(PLAYER_SIZE)));
    let material_handle = materials.add(ColorMaterial::from(Color::rgb(1., 0., 0.)));

    commands.spawn((
        PlayerBundle::new(),
        MaterialMesh2dBundle {
            mesh: mesh_handle.into(),
            material: material_handle,
            ..default()
        },
    ));
}

fn axis(negative: bool, positive: bool) -> f32 {
    ((positive as i8) - (negative as i8)) as f32
}

fn handle_player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<&mut Velocity, With<Player>>,
) {
    if let Ok(mut velocity) = player.get_single_mut() {
        let is_pressed = |key| keyboard_input.pressed(key);
        let x = axis(is_pressed(KeyCode::Left), is_pressed(KeyCode::Right));
        let y = axis(is_pressed(KeyCode::Down), is_pressed(KeyCode::Up));
        let magnitude = (x.powi(2) + y.powi(2)).sqrt();
        velocity.0.x = x / magnitude;
        velocity.0.y = y / magnitude;
    }
}

fn project_positions (mut obj: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut obj {
        transform.translation = position.0.extend(0.);
    }
}

fn move_player(
    mut paddle: Query<(&mut Position, &Velocity), With<Player>>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        for (mut position, velocity) in &mut paddle {
            let new_position = position.0 + velocity.0 * PLAYER_SPEED;
            let inside_window =
                new_position.y.abs() < window.resolution.height() / 2. - PLAYER_SIZE  &&
                new_position.x.abs() < window.resolution.width() / 2. - PLAYER_SIZE;
            if inside_window {
                position.0 = position.0 + velocity.0 * PLAYER_SPEED;
            }
        }
    }
}

const TILE_SIZE: TileSize = TileSize { x: 128.0, y: 128.0 };
const CHUNK_SIZE: UVec2 = UVec2 { x: 4, y: 4 };

#[derive(Component)]
struct TileSize { x: f32, y: f32 }

#[derive(Component)]
struct TilemapId(Entity);

#[derive(Component)]
struct TilemapSize(UVec2);

#[derive(Bundle)]
struct TilemapBundle {
    // grid_size: TileSize,
    size: TilemapSize,
    texture: Handle<Image>,
    tile_size: TileSize,
    transform: Transform,
}

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
            if distance > 3200.0 {
                let x = (chunk_pos.x / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32;
                let y = (chunk_pos.y / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32;
                chunk_manager.spawned_chunks.remove(&IVec2::new(x, y));
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    for (mut transform, mut ortho) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::Left) {
            direction -= Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += Vec3::new(1.0, 0.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Up) {
            direction += Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Down) {
            direction -= Vec3::new(0.0, 1.0, 0.0);
        }

        if keyboard_input.pressed(KeyCode::Z) {
            ortho.scale += 0.1;
        }

        if keyboard_input.pressed(KeyCode::X) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }

        let z = transform.translation.z;
        transform.translation += time.delta_seconds() * direction * 500.;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}

#[derive(Default, Debug, Resource)]
struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>,
}
