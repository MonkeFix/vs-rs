use std::collections::HashMap;
use bevy::prelude::*;
use crate::player::*;
use std::time::Duration;
use bevy::asset::AssetContainer;
use bevy::time::TimerMode::Repeating;
use rand::{Rng, thread_rng};
use crate::steering::{SteeringBundle, SteeringHost, SteerSeek};
use crate::stats::*;
use serde_json;
use serde::{Deserialize, Serialize};

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
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CurrentWave{ num: 1, timer: Timer::new(Duration::from_secs(30, ),Repeating), need_wave_spawn: true })
            .insert_resource(GlobalTimeTickerResource(Timer::new(Duration::from_secs(5), TimerMode::Repeating)))
            .add_systems(Startup, enemy_factory)
            .add_systems(Update, (
                spawn,
                movement,
                change_wave,
                global_timer_tick
            ));
    }
}

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
    texture: Handle<Image>,
    rewards: Rewards,
    is_elite: Option<bool>,
    timer: Timer,
}

#[derive(Resource, Debug, Clone)]
struct EnemySpawners(HashMap<u16, Vec<EnemySpawnComponent>>);

impl EnemySpawnComponent {
    fn new(name: String, enemy: Enemy, health: Health, damage: Damage, texture: Handle<Image>,is_elite: Option<bool>, rewards: Rewards) -> Self {
        Self {
            name,
            enemy,
            health,
            damage,
            texture,
            rewards,
            is_elite,
            timer: Default::default(),
        }
    }
}

#[derive(Resource)]
struct CurrentWave{
    num: u16,
    timer: Timer,
    need_wave_spawn: bool,
}

fn enemy_factory(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    #[derive(Debug, Serialize, Deserialize)]
    struct EnemyConfig {
        name: String,
        dmg: i64,
        hp: i64,
        asset_path: String,
        is_elite: Option<bool>,
        spawn_waves: Vec<SpawnWave>,
    }

    let conf = std::fs::read_to_string("configs/enemies.json").unwrap();
    let data = serde_json::from_str::<Vec<EnemyConfig>>(&conf).unwrap();

    let mut spawn_map: HashMap<u16, Vec<EnemySpawnComponent>> = HashMap::new();

    for enemy_conf in data {
        let texture_handle: Handle<Image> = asset_server.load(enemy_conf.asset_path);
        let mut enemy = EnemySpawnComponent::new(
            enemy_conf.name,
            Enemy,
            Health(enemy_conf.hp),
            Damage(enemy_conf.dmg),
            texture_handle,
            enemy_conf.is_elite,
            Rewards { exp: 1, items: "Orange" }, // TODO: Make them drop gems)
        );

        for waves in enemy_conf.spawn_waves {
            for n in waves.from..waves.to + 1 {
                if let Some(mut components) = spawn_map.get_mut(&n) {
                    enemy.timer = Timer::new(Duration::from_secs(waves.spawn_time), TimerMode::Repeating);
                    components.push(enemy.clone());
                } else {
                    let mut spawn_vec = Vec::new();
                    enemy.timer = Timer::new(Duration::from_secs(waves.spawn_time), TimerMode::Repeating);
                    spawn_vec.push(enemy.clone());
                    spawn_map.insert(n, spawn_vec);
                }
            }
        }
    }

    let spawners = EnemySpawners(spawn_map);
    dbg!(spawners.clone());
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

                if spawner.timer.finished() || current_wave.need_wave_spawn == true {
                    // spawn elite only one time on the wave
                    if Some(true) == spawner.is_elite && current_wave.need_wave_spawn == false {
                        continue
                    }
                    if let Ok(p_t) = player.get_single_mut() {
                        let is_left = thread_rng().gen_range(0, 2);
                        let is_up = thread_rng().gen_range(0, 2);
                        let mut enemy_batch: Vec<(EnemyBundle, SpriteBundle, SteeringBundle, Name)> = Vec::new();

                        for i in 1..thread_rng().gen_range(10, 20) {
                            let (mut m_x, mut m_y, mut m_z): (f32, f32, f32);
                            if is_left == 1 {
                                m_x = thread_rng().gen_range(p_t.translation.x - 700.0, p_t.translation.x - 300.0);
                            } else {
                                m_x = thread_rng().gen_range(p_t.translation.x + 300.0, p_t.translation.x + 700.0);
                            }

                            if is_up == 1 {
                                m_y = thread_rng().gen_range(p_t.translation.y - 700.0, p_t.translation.y - 300.0);
                            } else {
                                m_y = thread_rng().gen_range(p_t.translation.y + 300.0, p_t.translation.y + 700.0);
                            }

                            m_z = p_t.translation.z;
                            enemy_batch.push(
                                (EnemyBundle::new(spawner.health.clone(), spawner.damage.clone(), spawner.rewards.clone()),
                                 SpriteBundle {
                                     transform: Transform::from_translation(Vec3::new(m_x, m_y, m_z)),
                                     texture: spawner.texture.clone(),
                                     ..default()
                                 },
                                 SteeringBundle {
                                     host: SteeringHost {
                                         position: Vec2::new(m_x, m_y),
                                         max_velocity: 100.0,
                                         max_force: 100.0,
                                         mass: 2.0,
                                         ..default()
                                     },
                                 },
                                 Name::new(spawner.name.clone() + &i.to_string())));
                            if let Some(true) = spawner.is_elite {
                               break;
                            }
                        }
                        commands.spawn_batch(enemy_batch)
                    }
                }
            }
            if current_wave.need_wave_spawn == true {
                current_wave.need_wave_spawn = false;
            }
        }
    }
}

fn spawn_with_new_wave() {

}
fn movement(
    player: Query<&SteeringHost, With<Player>>,
    mut enemies: Query<(&mut Transform, &mut SteeringHost), (With<Enemy>, Without<Player>)>,
) {
    if let Ok(pl) = player.get_single() {
        for (mut t, mut st) in &mut enemies {
            st.steer(SteerSeek, &pl.position);
            if st.cur_velocity.x < 0.0 {
                t.scale.x = -1.0;
            } else {
                t.scale.x = 1.0
            }
        }
    }
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

fn global_timer_tick(
    mut global_time_ticker: ResMut<GlobalTimeTickerResource>,
    t: Res<Time>
) {
    global_time_ticker.0.tick(t.delta());
}
