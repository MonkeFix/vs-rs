use crate::collisions::colliders::Collider;
use crate::collisions::colliders::ColliderBundle;
use crate::collisions::colliders::ColliderSet;
use crate::collisions::shapes::ColliderShapeType;
use crate::player::*;
use crate::stats::*;
use crate::steering::{SteerSeek, SteeringBundle, SteeringHost};
use bevy::prelude::*;
use bevy::time::TimerMode::Repeating;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::time::Duration;

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
const GLOBAL_TIME_TICKER_SEC: u64 = 5;
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
        .add_systems(Startup, enemy_factory)
        .add_systems(
            Update,
            (
                spawn,
                movement,
                check_health,
                change_wave,
                global_timer_tick,
            ),
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

const ENEMY_BATCH_SIZE: MinMaxStruct<i64> = MinMaxStruct { min: 6, max: 20 };
#[derive(Resource)]
struct GlobalTimeTickerResource(Timer);

#[derive(Debug, Serialize, Deserialize)]
struct SpawnWave {
    from: u16,
    to: u16,
    spawn_time: u64,
}

#[derive(Component, Clone, Debug)]
struct Rewards {
    exp: u64,
    items: &'static str, // TODO: When <Item> class is implemented, remove this mock up
}

#[derive(Component, Clone, Debug)]
pub struct Enemy;

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
    enemy: Enemy,
    health: Health,
    damage: Damage,
    max_velocity: f32,
    max_force: f32,
    mass: f32,
    texture: Handle<Image>,
    rewards: Rewards,
    is_elite: Option<bool>,
    timer: Timer,
}

#[derive(Resource, Debug, Clone)]
struct EnemySpawners(HashMap<u16, Vec<EnemySpawnComponent>>);
#[derive(Resource, Default)]
struct CurrentWave {
    num: u16,
    timer: Timer,
    need_wave_spawn: bool,
}

fn enemy_factory(mut commands: Commands, asset_server: Res<AssetServer>) {
    #[derive(Debug, Serialize, Deserialize)]
    struct EnemyConfig {
        name: String,
        dmg: i64,
        hp: i64,
        max_velocity: f32,
        max_force: f32,
        mass: f32,
        asset_path: String,
        is_elite: Option<bool>,
        spawn_waves: Vec<SpawnWave>,
    }

    let conf = std::fs::read_to_string("configs/enemies.json").unwrap();
    let data = serde_json::from_str::<Vec<EnemyConfig>>(&conf).unwrap();

    let mut spawn_map: HashMap<u16, Vec<EnemySpawnComponent>> = HashMap::new();

    for enemy_conf in data {
        let texture_handle: Handle<Image> = asset_server.load(enemy_conf.asset_path);
        let mut enemy = EnemySpawnComponent {
            name: enemy_conf.name,
            enemy: Enemy,
            health: Health(enemy_conf.hp),
            damage: Damage(enemy_conf.dmg),
            max_velocity: enemy_conf.max_velocity,
            max_force: enemy_conf.max_force,
            mass: enemy_conf.mass,
            texture: texture_handle,
            is_elite: enemy_conf.is_elite,
            rewards: Rewards {
                exp: 1,
                items: "Orange",
            }, // TODO: Make them drop gems)
            timer: Default::default(),
        };

        for waves in enemy_conf.spawn_waves {
            for n in waves.from..waves.to + 1 {
                if let Some(mut components) = spawn_map.get_mut(&n) {
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
fn spawn(
    mut commands: Commands,
    mut spawn_map: ResMut<EnemySpawners>,
    mut player: Query<&mut Transform, With<Player>>,
    mut current_wave: ResMut<CurrentWave>,
    mut global_time_ticker: Res<GlobalTimeTickerResource>,
) {
    if global_time_ticker.0.finished() || current_wave.need_wave_spawn {
        if let Some(spawners) = spawn_map.0.get_mut(&current_wave.num) {
            for mut spawner in spawners {
                spawner.timer.tick(global_time_ticker.0.duration());

                if spawner.timer.finished() || current_wave.need_wave_spawn {
                    // spawn elite only one time on the wave
                    if Some(true) == spawner.is_elite && current_wave.need_wave_spawn == false {
                        continue;
                    }
                    if let Ok(p_t) = player.get_single_mut() {
                        let is_left = thread_rng().gen_range(0, 2);
                        let is_up = thread_rng().gen_range(0, 2);
                        let mut enemy_batch: Vec<(
                            EnemyBundle,
                            SpriteBundle,
                            SteeringBundle,
                            Name,
                            ColliderBundle,
                        )> = Vec::new();

                        for i in
                            1..thread_rng().gen_range(ENEMY_BATCH_SIZE.min, ENEMY_BATCH_SIZE.max)
                        {
                            let (mut m_x, mut m_y, mut m_z): (f32, f32, f32);
                            if is_left == 1 {
                                m_x = thread_rng().gen_range(
                                    p_t.translation.x - SPAWN_DISTANCE.max,
                                    p_t.translation.x - SPAWN_DISTANCE.min,
                                );
                            } else {
                                m_x = thread_rng().gen_range(
                                    p_t.translation.x + SPAWN_DISTANCE.min,
                                    p_t.translation.x + SPAWN_DISTANCE.max,
                                );
                            }

                            if is_up == 1 {
                                m_y = thread_rng().gen_range(
                                    p_t.translation.y - SPAWN_DISTANCE.max,
                                    p_t.translation.y - SPAWN_DISTANCE.min,
                                );
                            } else {
                                m_y = thread_rng().gen_range(
                                    p_t.translation.y + SPAWN_DISTANCE.min,
                                    p_t.translation.y + SPAWN_DISTANCE.max,
                                );
                            }

                            m_z = p_t.translation.z;
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
                                    host: SteeringHost {
                                        position: Vec2::new(m_x, m_y),
                                        max_velocity: spawner.max_velocity,
                                        max_force: spawner.max_force,
                                        mass: spawner.mass,
                                        ..default()
                                    },
                                },
                                Name::new(spawner.name.clone() + &i.to_string()),
                                ColliderBundle {
                                    collider: Collider::new(ColliderShapeType::Circle {
                                        radius: 16.0,
                                    }),
                                },
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
fn movement(
    player: Query<&SteeringHost, With<Player>>,
    mut enemies: Query<
        (&mut Transform, &mut SteeringHost, &Collider, Entity),
        (With<Enemy>, Without<Player>),
    >,
    collider_set: Res<ColliderSet>,
) {
    if let Ok(pl) = player.get_single() {
        for (mut t, mut st, collider, entity) in &mut enemies {
            st.steer(SteerSeek, &pl.position);

            if st.cur_velocity.x < 0.0 {
                t.scale.x = -1.0;
            } else {
                t.scale.x = 1.0
            }

            let neighbors = get_neighbors(entity, &collider_set);

            for n in neighbors {
                if let Some(res) = collider.collides_with(&n) {
                    let target = st.position - res.min_translation;
                    st.steer(SteerSeek, &target);
                }
            }
        }
    }
}

fn get_neighbors(entity: Entity, collider_set: &Res<ColliderSet>) -> Vec<Collider> {
    let mut res = vec![];

    for (index, collider) in &collider_set.map {
        if entity.index() == *index {
            continue;
        }

        res.push(collider.clone());
    }

    res
}

fn check_health(
    mut commands: Commands,
    enemies: Query<(&Health, Entity), (With<Enemy>, Without<Player>)>,
) {
    for (health, entity) in &enemies {
        if health.0 <= 0 {
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
