use bevy::prelude::*;
use bevy::text::*;
use bevy::time::Stopwatch;
use std::time::Duration;
use bevy::sprite::Anchor;

pub struct GameTimerPlugin;

impl Plugin for GameTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
           .insert_resource(GameTimer(Stopwatch::default()))
           .add_systems(Update, update_timer);
    }
}

#[derive(Resource)]
struct GameTimer(Stopwatch);

#[derive(Component)]
struct GameTimerText;

fn spawn (mut commands: Commands) {
    commands.spawn(NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            justify_self: JustifySelf::Center,
            ..Default::default()
        },
        z_index: ZIndex::Global(2),
        ..Default::default()
    }).with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                "00:00",
                TextStyle {
                    font_size: 50.0,
                    color: Color::BLACK,
                    ..default()
                },
            ).with_text_justify(JustifyText::Center),
            GameTimerText,
        ));
    });
}

fn update_timer(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    mut query: Query<&mut Text, With<GameTimerText>>,
) {
    timer.0.tick(time.delta());

    let elapsed_secs = timer.0.elapsed_secs();
    let minutes = (elapsed_secs / 60.0).floor() as u32;
    let seconds = (elapsed_secs % 60.0).floor() as u32;

    for mut text in query.iter_mut() {
        text.sections[0].value = format!("{:02}:{:02}", minutes, seconds);
    }
}
