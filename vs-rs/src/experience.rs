use bevy::prelude::*;
use physics::{
    prelude::{
        colliders::Collider,
        shapes::{Shape, ShapeType},
    },
    InvokeTriggerEvent,
};
use vs_assets::plugin::GameAssets;

use crate::{
    enemy::{Enemy, EnemyDieEvent},
    player::Player,
    stats::{Experience, ExperienceDrop},
    AppState,
};

#[derive(Component, Debug, Default)]
pub struct ExperienceGem {
    pub value: u32,
}

#[derive(Bundle, Default)]
pub struct ExperienceGemBundle {
    pub gem: ExperienceGem,
    pub image: Handle<Image>,
}

#[derive(Default)]
pub struct ExperiencePlugin;

impl Plugin for ExperiencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (on_enemy_die, on_player_collided).run_if(in_state(AppState::Finished)),
        );
    }
}

fn on_enemy_die(
    mut events: EventReader<EnemyDieEvent>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
) {
    for evt in events.read() {
        commands.spawn((
            ExperienceGem { value: evt.exp },
            SpriteBundle {
                transform: Transform::from_translation(evt.position.extend(10.0)),
                texture: game_assets.exp_gem_texture.clone(),
                ..default()
            },
            Collider {
                shape: Shape::new(ShapeType::Circle { radius: 16.0 }),
                physics_layer: 1,
                is_trigger: true,
                ..default()
            },
        ));
    }
}

fn on_player_collided(
    mut events: EventReader<InvokeTriggerEvent>,
    exp: Query<(&ExperienceGem, Entity)>,
    mut player: Query<(&mut Experience, Entity), With<Player>>,
    mut commands: Commands,
) {
    if let Ok((mut player_exp, player_entity)) = player.get_single_mut() {
        for evt in events.read() {
            if evt.entity_main != player_entity {
                return;
            }
            if let Ok(gem) = exp.get(evt.entity_trigger) {
                player_exp.0 += gem.0.value;
                info!("Adding {} exp! Cur exp: {}", gem.0.value, player_exp.0);

                commands.entity(evt.entity_trigger).despawn();
            }
        }
    }
}
