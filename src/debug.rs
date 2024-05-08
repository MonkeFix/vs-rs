use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::collisions::colliders::Collider;
use crate::enemy::Enemy;
use crate::movement::SteeringHost;
use crate::player::Player;

#[cfg(debug_assertions)]
pub struct DebugPlugin;

#[cfg(debug_assertions)]
impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SteeringHost>();
        app.register_type::<Collider>();
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_systems(Update, bevy::window::close_on_esc);

        app.add_systems(Startup, (add_enemy_count,));
        app.add_systems(FixedUpdate, (update_enemy_count, update_fps));
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
