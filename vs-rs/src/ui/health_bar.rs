use bevy::prelude::*;
use bevy::sprite::Anchor;
use crate::stats::*;
use crate::player::*;
use vs_assets::plugin::UiAssets;

pub struct HealthBarPlugin;

impl Plugin for HealthBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

const WIDTH: f32 = 75.;
const HEIGHT: f32 = 10.;

#[derive(Component)]
struct HealthBar;

pub fn spawn_health_bar(
    child_builder: &mut ChildBuilder,
    assets: Res<UiAssets>,
) {
    let bar_texture: Handle<Image> = assets.health_bar.clone();
    let bar_outline_texture: Handle<Image> = assets.health_bar_outline.clone();

    let sprite = Sprite {
        custom_size: Some(Vec2::new(WIDTH, HEIGHT)),
        anchor: Anchor::CenterLeft,
        color: Color::rgb(0.8, 0.1, 0.1),
        ..Default::default()
    };

    let transform = Transform::from_translation(Vec3::new(-WIDTH / 2., -35., 100.));

    let health_bar_bundle = SpriteBundle {
        texture: bar_texture,
        sprite: sprite.clone(),
        transform: transform.clone(),
        ..Default::default()
    };
    child_builder.spawn((health_bar_bundle, HealthBar));

    let health_bar_outline_bundle = SpriteBundle {
        texture: bar_outline_texture,
        transform,
        sprite,
        ..Default::default()
    };
    child_builder.spawn(health_bar_outline_bundle);
}

fn update(
    mut hp_bar: Query<&mut Sprite, With<HealthBar>>,
    health: Query<(&Health, &MaxHealth), With<Player>>,
) {
    if let Ok(mut sprite) = hp_bar.get_single_mut() {
        if let Ok((health, max_health)) = health.get_single() {
            let percent = f32::clamp(health.0 as f32 / max_health.0 as f32, 0., 1.);
            sprite.custom_size = Some(Vec2::new(WIDTH * percent, HEIGHT));
        }
    }
}
