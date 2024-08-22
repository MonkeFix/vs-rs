#![cfg(debug_assertions)]
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use common::Position;

use crate::enemy::Enemy;
use crate::movement::SteeringHost;
use crate::player::Player;
use collisions::prelude::*;

#[cfg(debug_assertions)]
pub struct DebugPlugin;

#[cfg(debug_assertions)]
#[derive(Default, Resource)]
pub struct DebugSettings {
    pub collider_draw_enabled: bool,
    pub disable_enemy_spawns: bool,
}

#[cfg(debug_assertions)]
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SteeringHost>();
        app.register_type::<Collider>();
        app.register_type::<Position>();
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_systems(Update, close_on_esc);

        app.add_systems(Startup, (add_enemy_count,));
        app.add_systems(FixedUpdate, (update_enemy_count, update_fps));
        app.add_systems(Update, (handle_input, debug_draw));

        app.insert_resource(DebugSettings::default());
    }
}

fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

fn add_enemy_count(mut commands: Commands) {
    commands.spawn(
        TextBundle::from_section(
            "Capybaras: ",
            TextStyle {
                font_size: 40.0,
                color: Color::BLACK,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
    );

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "FPS: ",
                TextStyle {
                    font_size: 40.0,
                    color: Color::BLACK,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: 40.0,
                color: Color::BLACK,
                ..default()
            }),
        ]),
        FpsText,
    ));
}

fn update_enemy_count(
    enemies: Query<&Enemy, (With<Enemy>, Without<Player>)>,
    mut text: Query<&mut Text, Without<FpsText>>,
) {
    if let Ok(mut text) = text.get_single_mut() {
        text.sections[0].value = format!("Capybaras: {}", enemies.iter().count());
    }
}
#[cfg(debug_assertions)]
fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_settings: ResMut<DebugSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyG) {
        debug_settings.collider_draw_enabled = !debug_settings.collider_draw_enabled;
    }
    if keyboard_input.just_pressed(KeyCode::KeyH) {
        debug_settings.disable_enemy_spawns = !debug_settings.disable_enemy_spawns;
    }
}

#[cfg(debug_assertions)]
fn debug_draw(
    debug_settings: Res<DebugSettings>,
    collider_store: Res<ColliderStore>,
    mut gizmos: Gizmos,
) {
    if debug_settings.collider_draw_enabled {
        collider_store.debug_draw(&mut gizmos);
    }
}

#[derive(Component)]
struct FpsText;

fn update_fps(diagnostics: Res<DiagnosticsStore>, mut fps_text: Query<&mut Text, With<FpsText>>) {
    if let Ok(mut text) = fps_text.get_single_mut() {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(value) = fps.smoothed() {
                text.sections[1].value = format!("{value:.2}");
            }
        }
    }
}
