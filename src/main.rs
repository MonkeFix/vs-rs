use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;

mod tilemap;
mod movement;

use tilemap::TileMapPlugin;
use movement::MovementPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TileMapPlugin)
        .add_plugins(MovementPlugin)
        .add_systems(Startup, (spawn_camera, spawn_player))
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
