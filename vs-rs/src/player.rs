use crate::enemy::Enemy;
use crate::input::PlayerControls;
use crate::stats::*;
use crate::AppState;
use behaviors::SteerSeek;
use bevy::sprite::Anchor;
use bevy::{input::gamepad::GamepadSettings, prelude::*};
use colliders::Collider;
use physics::prelude::*;
use physics::CollideEvent;
use shapes::Shape;
use std::time::Duration;
use steering::SteeringBundle;
use steering::SteeringTargetVec2;
use vs_assets::plugin::GameAssets;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Finished), spawn)
            .add_systems(
                FixedUpdate,
                (movement, check_health)
                    .run_if(in_state(AppState::Finished)),
            )
            .add_systems(
                Update,
                (update_timer, handle_input, update_sprite, update_atlas)
                    .run_if(in_state(AppState::Finished)),
            );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
struct PlTimer(Timer);

#[derive(Component)]
struct Direction(Vec2);

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
            player: Player,
            health: Health(100),
            inv_timer: PlTimer(Timer::new(Duration::from_millis(500), TimerMode::Once)),
            direction: Direction(Vec2::ZERO),
        }
    }
}

fn spawn(mut commands: Commands, assets: Res<GameAssets>) {
    let player_tileset = &assets.player_tilesheet;

    commands
        .spawn((
            PlayerBundle::new(),
            TextureAtlas {
                layout: player_tileset.layout.clone(),
                index: 0,
            },
            SpriteBundle {
                sprite: Sprite::default(),
                texture: player_tileset.image.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 1.0),
                ..default()
            },
            SteeringBundle { ..default() },
            Name::new("player"),
            Collider {
                shape: Shape::new(shapes::ShapeType::Circle { radius: 10.0 }),
                local_offset: Vec2::new(0.0, -16.0),
                physics_layer: 2,
                collides_with_layers: 1 | 2,
                ..default()
            },
            SteerSeek,
            SteeringTargetVec2::default(),
        ))
        .with_children(|c| {
            c.spawn((
                TextureAtlas {
                    layout: player_tileset.layout.clone(),
                    index: 3,
                },
                SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::Center,
                        color: Color::srgba(1.0, 1.0, 1.0, 0.3),
                        ..default()
                    },
                    texture: player_tileset.image.clone(),
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, -1.0),
                        ..default()
                    },
                    ..default()
                },
            ));
        })
        .observe(on_collision);
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
    mut steering_host: Query<(&mut SteeringTargetVec2, &Transform, &Direction), With<Player>>,
) {
    if let Ok((mut target, pos, dir)) = steering_host.get_single_mut() {
        target.0 = if dir.0 == Vec2::ZERO {
            Vec2::ZERO
        } else {
            pos.translation.xy() + dir.0 * 10.0
        };
    }
}

fn update_atlas(mut query: Query<(&Direction, &mut TextureAtlas), With<Player>>) {
    if let Ok((dir, mut atlas)) = query.get_single_mut() {
        if dir.0.y > 0.0 {
            atlas.index = 1;
        } else if dir.0.x != 0.0 {
            atlas.index = 2;
        } else {
            atlas.index = 0;
        }
    }
}

fn update_sprite(mut query: Query<(&Direction, &mut Sprite), With<Player>>) {
    if let Ok((dir, mut sprite)) = query.get_single_mut() {
        sprite.flip_x = dir.0.x > 0.;
    }
}

fn on_collision(
    trigger: Trigger<CollideEvent>,
    damage_query: Query<&Damage, (With<Enemy>, Without<Player>)>,
    mut player: Query<(&mut PlTimer, &mut Health), With<Player>>,
) {
    let e = trigger.event();

    if let Ok(dmg) = damage_query.get(e.collided_with) {
        if let Ok((mut timer, mut health)) = player.get_mut(e.entity_main) {
            if timer.0.finished() {
                info!("damaging!");
                health.0 = health.0.saturating_sub(dmg.0);
                timer.0.reset();
            }
        }
    }
}

fn update_timer(time: Res<Time>, mut player: Query<&mut PlTimer, With<Player>>) {
    if let Ok(mut timer) = player.get_single_mut() {
        timer.0.tick(time.delta());
    }
}

fn check_health(mut player: Query<&mut Health, With<Player>>) {
    if let Ok(mut h) = player.get_single_mut() {
        if h.0 <= 0 {
            error!("you lost. please close the game");
            h.0 = 100;
        }
    }
}
