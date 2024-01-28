use crate::collisions::colliders::{Collider, ColliderBundle};
use crate::collisions::shapes::ColliderShapeType;
use crate::enemy::Enemy;
use crate::stats::*;
use crate::{
    input::PlayerControls,
    steering::{SteerSeek, SteeringBundle, SteeringHost},
};
use bevy::log;
use bevy::{input::gamepad::GamepadSettings, prelude::*};
use std::time::Duration;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn).add_systems(
            Update,
            (movement, check_enemy_collision, check_health).chain(),
        );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct PlTimer(Timer);
#[derive(Bundle)]
struct PlayerBundle {
    player: Player,
    health: Health,
    inv_timer: PlTimer,
}

impl PlayerBundle {
    fn new() -> Self {
        Self {
            player: Player,
            health: Health(100),
            inv_timer: PlTimer(Timer::new(Duration::from_millis(500), TimerMode::Repeating)),
        }
    }
}

fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_handle: Handle<Image> = asset_server.load("player.png");
    commands.spawn((
        PlayerBundle::new(),
        SpriteBundle {
            texture: texture_handle,
            transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
            ..default()
        },
        SteeringBundle {
            host: SteeringHost::default(),
        },
        Name::new("player"),
        ColliderBundle {
            collider: Collider::new(ColliderShapeType::Circle { radius: 16.0 }),
        },
    ));
}

fn movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut steering_host: Query<&mut SteeringHost, With<Player>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
    gamepad_settings: Res<GamepadSettings>,
    gamepads: Res<Gamepads>,
    controls: Res<PlayerControls>,
    player_collider: Query<&Collider, With<Player>>,
    other_colliders: Query<&Collider, Without<Player>>,
) {
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

    if let Ok(mut host) = steering_host.get_single_mut() {
        let target = host.position + direction;
        host.steer(SteerSeek, &target);

        if let Ok(player_collider) = player_collider.get_single() {
            for collider in &other_colliders {
                let col = player_collider.collides_with(collider);
                if let Some(mut col) = col {
                    col.invert();
                    let target = host.position + col.min_translation;
                    host.steer(SteerSeek, &target);
                }
            }
        }
    }
}

fn check_enemy_collision(
    mut player: Query<(&mut Health, &mut PlTimer, &SteeringHost), With<Player>>,
    mut enemies: Query<(&mut Damage, &SteeringHost), (With<Enemy>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok((mut pl, mut timer, sh)) = player.get_single_mut() {
        for (e_d, e_sh) in enemies.iter_mut() {
            if (sh.position.x.abs() - e_sh.position.x.abs()).abs() <= 10.0
                && (sh.position.y.abs() - e_sh.position.y.abs()).abs() <= 10.0
            {
                timer.0.tick(time.delta());

                if timer.0.finished() {
                    pl.0 = pl.0.saturating_sub(e_d.0);
                }
            }
        }
    }
}

fn check_health(mut player: Query<&mut Health, With<Player>>) {
    if let Ok(h) = player.get_single_mut() {
        if h.0 <= 0 {
            log::error!("you lost. please close the game");
        }
    }
}
