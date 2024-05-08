use crate::collisions::plugin::{ColliderBundle, ColliderComponent};
use crate::collisions::shapes::ColliderShapeType;
use crate::collisions::store::{ColliderIdResolver, ColliderStore};
use crate::enemy::Enemy;
use crate::input::PlayerControls;
use crate::movement::behaviors::SteerSeek;
use crate::movement::{PhysicsParams, Position, SteeringBundle, SteeringHost};
use crate::stats::*;
use bevy::log;
use bevy::{input::gamepad::GamepadSettings, prelude::*};
use std::time::Duration;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(FixedUpdate, (movement, check_enemy_collision, check_health))
            .add_systems(Update, handle_input);
    }
}

#[derive(Component)]
pub struct Player {
    steer_seek: SteerSeek,
}

#[derive(Component)]
struct PlTimer(Timer);

#[derive(Component)]
struct Direction(Vec2);

#[derive(Event)]
struct TimerCallbackEvent(());

#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    health: Health,
    inv_timer: PlTimer,
    direction: Direction,
}

impl PlayerBundle {
    fn new() -> Self {
        Self {
            player: Player {
                steer_seek: SteerSeek,
            },
            health: Health(100),
            inv_timer: PlTimer(Timer::new(Duration::from_millis(500), TimerMode::Repeating)),
            direction: Direction(Vec2::ZERO),
        }
    }
}

fn spawn(
    mut collider_set: ResMut<ColliderStore>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let texture_handle: Handle<Image> = asset_server.load("player.png");
    commands.spawn((
        PlayerBundle::new(),
        SpriteBundle {
            texture: texture_handle,
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..default()
        },
        SteeringBundle { ..default() },
        Name::new("player"),
        ColliderBundle {
            collider: ColliderComponent::new(
                &mut collider_set,
                ColliderShapeType::Circle { radius: 16.0 },
            ),
        },
    ));
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_settings: Res<GamepadSettings>,
    gamepads: Res<Gamepads>,
    controls: Res<PlayerControls>,
    mut query: Query<&mut Direction, With<Player>>,
) {
    if let Ok(mut dir) = query.get_single_mut() {
        let mut direction = Vec2::ZERO;

        let dead_upper = gamepad_settings.default_axis_settings.deadzone_upperbound();
        let dead_lower = gamepad_settings.default_axis_settings.deadzone_lowerbound();

        for gamepad in gamepads.iter() {
            let x1 = gamepad_axes
                .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_x))
                .unwrap();
            let y1 = gamepad_axes
                .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_y))
                .unwrap();

            let x2 = gamepad_axes
                .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_x_2))
                .unwrap();
            let y2 = gamepad_axes
                .get(GamepadAxis::new(gamepad, controls.gamepad.move_axis_y_2))
                .unwrap();

            let x = (x1 + x2) / 2.0;
            let y = (y1 + y2) / 2.0;

            if x > dead_lower || x < dead_upper {
                direction.x = x;
            }
            if y > dead_lower || y < dead_upper {
                direction.y = y;
            }
        }

        if keyboard_input.pressed(controls.keyboard.move_left)
            || keyboard_input.pressed(controls.keyboard.move_left_2)
        {
            direction -= Vec2::new(1.0, 0.0);
        }
        if keyboard_input.pressed(controls.keyboard.move_right)
            || keyboard_input.pressed(controls.keyboard.move_right_2)
        {
            direction += Vec2::new(1.0, 0.0);
        }
        if keyboard_input.pressed(controls.keyboard.move_up)
            || keyboard_input.pressed(controls.keyboard.move_up_2)
        {
            direction += Vec2::new(0.0, 1.0);
        }
        if keyboard_input.pressed(controls.keyboard.move_down)
            || keyboard_input.pressed(controls.keyboard.move_down_2)
        {
            direction -= Vec2::new(0.0, 1.0);
        }

        let direction = direction.normalize_or_zero();
        dir.0 = direction;
    }
}

fn movement(
    mut steering_host: Query<
        (
            &mut SteeringHost,
            &Position,
            &PhysicsParams,
            &Direction,
            &Player,
        ),
        With<Player>,
    >,
) {
    if let Ok((mut host, pos, params, dir, player)) = steering_host.get_single_mut() {
        let target = pos.0 + dir.0;

        let steering = player.steer_seek.steer(pos, &host, params, &target);

        host.steer(steering);
    }
}

fn check_enemy_collision(
    time: Res<Time>,
    mut player_collider: Query<(&ColliderComponent, &mut Health, &mut PlTimer), With<Player>>,
    enemies: Query<&Damage, (With<Enemy>, Without<Player>)>,
    collider_store: Res<ColliderStore>,
) {
    if let Ok((id, mut hp, mut timer)) = player_collider.get_single_mut() {
        let player_collider = collider_store.get(id.id).unwrap();

        let rect = player_collider.bounds();
        let neighbors = collider_store.aabb_broadphase_excluding_self(id.id, rect, None);
        for neighbor in neighbors {
            let collider = collider_store.get(neighbor).unwrap();
            if player_collider.collides_with(collider).is_some() {
                timer.0.tick(time.delta());

                if timer.0.finished() {
                    if let Some(entity) = collider.entity {
                        let dmg = enemies.get_component::<Damage>(entity);
                        if let Ok(dmg) = dmg {
                            hp.0 = hp.0.saturating_sub(dmg.0);
                        }
                    }
                }
            }
        }
    }
}

//fn timer_callback() {}

fn check_health(mut player: Query<&mut Health, With<Player>>) {
    if let Ok(h) = player.get_single_mut() {
        if h.0 <= 0 {
            log::error!("you lost. please close the game");
        }
    }
}
