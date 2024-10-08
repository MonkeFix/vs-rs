#[cfg(debug_assertions)]
use crate::debug::DebugSettings;
use crate::player::*;
use crate::prelude::*;
use crate::stats::*;
use crate::AppState;
use behaviors::SteerArrival;
use behaviors::SteerCollisionAvoidance;
use behaviors::SteerSeek;
use bevy::time::TimerMode::Repeating;
use colliders::Collider;
use shapes::Shape;
use std::collections::HashMap;
use std::time::Duration;
use steering::PhysicalParams;
use steering::SteeringBundle;
use steering::SteeringHost;
use steering::SteeringTargetEntity;
use vs_assets::enemies::EnemyConfig;
use vs_assets::plugin::Configs;
use vs_assets::plugin::GameAssets;

pub struct EnemyPlugin;

impl EnemyBundle {
    fn new(h: Health, d: Damage, r: Rewards) -> Self {
        Self {
            enemy: Enemy,
            health: h,
            damage: d,
            rewards: r,
        }
    }
}
/*
todolist
Walk towards player [V]
Damage player when almost in the player (atm dying).
Proper spawn of multiple enemies [V]
Die. [V]
 */

// TODO: Move to config?
const WAVE_DURATION_SEC: u64 = 30;
const GLOBAL_TIME_TICKER_SEC: u64 = 1;
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentWave {
            num: 1,
            timer: Timer::new(Duration::from_secs(WAVE_DURATION_SEC), Repeating),
            need_wave_spawn: true,
        })
        .insert_resource(GlobalTimeTickerResource(Timer::new(
            Duration::from_secs(GLOBAL_TIME_TICKER_SEC),
            TimerMode::Repeating,
        )))
        .add_systems(OnEnter(AppState::Finished), (enemy_factory,))
        .add_systems(FixedUpdate, (update_timers,));

        #[cfg(debug_assertions)]
        app.add_systems(
            FixedUpdate,
            (
                spawn,
                movement,
                check_health,
                change_wave,
                global_timer_tick,
            )
                .run_if(in_state(AppState::Finished))
                .run_if(enemy_spawns_enabled),
        );

        #[cfg(not(debug_assertions))]
        app.add_systems(
            FixedUpdate,
            (
                spawn,
                movement,
                check_health,
                change_wave,
                global_timer_tick,
            )
                .run_if(in_state(AppState::Finished)),
        );
    }
}

struct MinMaxStruct<T> {
    min: T,
    max: T,
}
const SPAWN_DISTANCE: MinMaxStruct<f32> = MinMaxStruct {
    min: 500.0,
    max: 800.0,
};

const ENEMY_BATCH_SIZE: MinMaxStruct<i64> = MinMaxStruct { min: 15, max: 20 };
#[derive(Resource)]
struct GlobalTimeTickerResource(Timer);

#[derive(Component, Clone, Debug)]
struct Rewards {
    _exp: u64,
    _items: &'static str, // TODO: When <Item> class is implemented, remove this mock up
}

#[derive(Component, Clone, Debug)]
pub struct Enemy;

#[derive(Event)]
pub struct EnemyDieEvent {
    pub position: Vec2,
    pub exp: u32,
}

#[derive(Bundle)]
struct EnemyBundle {
    enemy: Enemy,
    health: Health,
    damage: Damage,
    rewards: Rewards,
}

#[derive(Component, Clone, Debug)]
struct EnemySpawnComponent {
    name: String,
    _enemy: Enemy,
    health: Health,
    damage: Damage,
    max_velocity: f32,
    max_force: f32,
    mass: f32,
    texture: Handle<Image>,
    rewards: Rewards,
    is_elite: Option<bool>,
    timer: Timer,
    exp_drop: u32,
}

#[derive(Resource, Debug, Clone)]
struct EnemySpawners(HashMap<u16, Vec<EnemySpawnComponent>>);
#[derive(Resource, Default)]
struct CurrentWave {
    num: u16,
    timer: Timer,
    need_wave_spawn: bool,
}

#[derive(Component)]
struct EnemyDamageTimer(Timer);

fn enemy_factory(
    mut commands: Commands,
    assets: Res<GameAssets>,
    configs: Res<Configs>,
    config_assets: Res<Assets<EnemyConfig>>,
) {
    let data = config_assets.get(configs.enemy_config.id()).unwrap();
    let data = &data.param_list;

    let mut spawn_map: HashMap<u16, Vec<EnemySpawnComponent>> = HashMap::new();

    for enemy_conf in data {
        // TODO: use file from the config
        let texture_handle: Handle<Image> = assets.capybara_texture.clone();
        let mut enemy = EnemySpawnComponent {
            name: enemy_conf.name.clone(),
            _enemy: Enemy,
            health: Health(enemy_conf.hp),
            damage: Damage(enemy_conf.dmg),
            max_velocity: enemy_conf.max_velocity,
            max_force: enemy_conf.max_force,
            mass: enemy_conf.mass,
            texture: texture_handle,
            is_elite: enemy_conf.is_elite,
            rewards: Rewards {
                _exp: 1,
                _items: "Orange",
            }, // TODO: Make them drop gems)
            timer: Default::default(),
            exp_drop: enemy_conf.exp_drop,
        };

        for waves in &enemy_conf.spawn_waves {
            for n in waves.from..waves.to + 1 {
                if let Some(components) = spawn_map.get_mut(&n) {
                    enemy.timer =
                        Timer::new(Duration::from_secs(waves.spawn_time), TimerMode::Repeating);
                    components.push(enemy.clone());
                } else {
                    let mut spawn_vec = Vec::new();
                    enemy.timer =
                        Timer::new(Duration::from_secs(waves.spawn_time), TimerMode::Repeating);
                    spawn_vec.push(enemy.clone());
                    spawn_map.insert(n, spawn_vec);
                }
            }
        }
    }

    let spawners = EnemySpawners(spawn_map);
    commands.insert_resource(spawners.clone());
}

#[cfg(debug_assertions)]
fn enemy_spawns_enabled(debug_settings: Res<DebugSettings>) -> bool {
    !debug_settings.disable_enemy_spawns
}

fn spawn(
    mut commands: Commands,
    mut spawn_map: ResMut<EnemySpawners>,
    mut player: Query<(&mut Transform, Entity), With<Player>>,
    mut current_wave: ResMut<CurrentWave>,
    global_time_ticker: Res<GlobalTimeTickerResource>,
) {
    if global_time_ticker.0.finished() || current_wave.need_wave_spawn {
        if let Some(spawners) = spawn_map.0.get_mut(&current_wave.num) {
            for spawner in spawners {
                spawner.timer.tick(global_time_ticker.0.duration());

                if spawner.timer.finished() || current_wave.need_wave_spawn {
                    // spawn elite only one time on the wave
                    if Some(true) == spawner.is_elite && !current_wave.need_wave_spawn {
                        continue;
                    }
                    if let Ok((player_transform, player_entity)) = player.get_single_mut() {
                        let is_left = thread_rng().gen_range(0..2);
                        let is_up = thread_rng().gen_range(0..2);
                        let mut enemy_batch = Vec::new();

                        for i in
                            1..thread_rng().gen_range(ENEMY_BATCH_SIZE.min..ENEMY_BATCH_SIZE.max)
                        {
                            let (m_x, m_y, m_z): (f32, f32, f32);
                            if is_left == 1 {
                                m_x = thread_rng().gen_range(
                                    player_transform.translation.x - SPAWN_DISTANCE.max
                                        ..player_transform.translation.x - SPAWN_DISTANCE.min,
                                );
                            } else {
                                m_x = thread_rng().gen_range(
                                    player_transform.translation.x + SPAWN_DISTANCE.min
                                        ..player_transform.translation.x + SPAWN_DISTANCE.max,
                                );
                            }

                            if is_up == 1 {
                                m_y = thread_rng().gen_range(
                                    player_transform.translation.y - SPAWN_DISTANCE.max
                                        ..player_transform.translation.y - SPAWN_DISTANCE.min,
                                );
                            } else {
                                m_y = thread_rng().gen_range(
                                    player_transform.translation.y + SPAWN_DISTANCE.min
                                        ..player_transform.translation.y + SPAWN_DISTANCE.max,
                                );
                            }

                            m_z = player_transform.translation.z;
                            enemy_batch.push((
                                EnemyBundle::new(
                                    spawner.health.clone(),
                                    spawner.damage.clone(),
                                    spawner.rewards.clone(),
                                ),
                                SpriteBundle {
                                    transform: Transform::from_translation(Vec3::new(
                                        m_x, m_y, m_z,
                                    )),
                                    texture: spawner.texture.clone(),
                                    ..default()
                                },
                                SteeringBundle {
                                    physics_params: PhysicalParams {
                                        max_velocity: spawner.max_velocity,
                                        max_force: spawner.max_force,
                                        mass: spawner.mass,
                                        ..default()
                                    },
                                    ..default()
                                },
                                Name::new(spawner.name.clone() + &i.to_string()),
                                Collider {
                                    shape: Shape::new(shapes::ShapeType::Circle { radius: 16.0 }),
                                    physics_layer: 0b100,
                                    collides_with_layers: 0b101,
                                    ..default()
                                },
                                //SteerSeek,
                                SteerArrival {
                                    slowing_radius: 64.0,
                                },
                                //SteerCollisionAvoidance::default(),
                                SteeringTargetEntity(player_entity),
                                EnemyDamageTimer(Timer::new(
                                    Duration::from_secs(1),
                                    TimerMode::Repeating,
                                )),
                                ExperienceDrop(spawner.exp_drop),
                            ));
                            if let Some(true) = spawner.is_elite {
                                break;
                            }
                        }
                        commands.spawn_batch(enemy_batch)
                    }
                }
            }
            if current_wave.need_wave_spawn {
                current_wave.need_wave_spawn = false;
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn movement(
    mut enemies: Query<(&mut Transform, &SteeringHost), (With<Enemy>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Enemy>)>,
) {
    let player = player.get_single().unwrap();
    for (mut transform, _host) in &mut enemies {
        if transform.translation.x - player.translation.x < 0.0 {
            transform.scale.x = 1.0;
        } else {
            transform.scale.x = -1.0;
        }
        /* if host.velocity.x < 0.0 {
            transform.scale.x = -1.0;
        } else {
            transform.scale.x = 1.0
        } */
    }
}

#[allow(clippy::type_complexity)]
fn check_health(
    mut commands: Commands,
    enemies: Query<(&Health, &Transform, &ExperienceDrop, Entity), (With<Enemy>, Without<Player>)>,
    mut enemy_die: EventWriter<EnemyDieEvent>,
) {
    for (health, transform, exp, entity) in enemies.iter() {
        if health.0 <= 0 {
            enemy_die.send(EnemyDieEvent {
                position: transform.translation.xy(),
                exp: exp.0,
            });
            commands.entity(entity).despawn();
        }
    }
}

fn change_wave(
    global_time_ticker: Res<GlobalTimeTickerResource>,
    mut cur_wave: ResMut<CurrentWave>,
) {
    if global_time_ticker.0.finished() {
        cur_wave.timer.tick(global_time_ticker.0.duration());
        if cur_wave.timer.finished() {
            cur_wave.num += 1;
            cur_wave.need_wave_spawn = true;
        }
    }
}

fn global_timer_tick(mut global_time_ticker: ResMut<GlobalTimeTickerResource>, t: Res<Time>) {
    global_time_ticker.0.tick(t.delta());
}

fn update_timers(
    time: Res<Time>,
    mut query: Query<(&mut EnemyDamageTimer, &mut Health), With<Enemy>>,
) {
    for (mut timer, mut health) in query.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            health.0 -= 19;
        }
    }
}
