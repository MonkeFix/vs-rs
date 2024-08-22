use bevy::{input::mouse::MouseWheel, prelude::*};

use crate::player::Player;

pub struct CameraMovementPlugin;

impl Plugin for CameraMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_follow);
        #[cfg(debug_assertions)]
        app.add_systems(Update, camera_zoom);
    }
}

#[cfg(debug_assertions)]
fn camera_zoom(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    if let Ok(mut ortho) = query.get_single_mut() {
        if keyboard_input.pressed(KeyCode::KeyZ) {
            ortho.scale += 0.1;
        }
        if keyboard_input.pressed(KeyCode::KeyX) {
            ortho.scale -= 0.1;
        }
        if keyboard_input.pressed(KeyCode::Backspace) {
            ortho.scale = 1.0;
        }

        for ev in scroll_evr.read() {
            ortho.scale -= ev.y / 10.0;
        }

        ortho.scale = ortho.scale.clamp(0.5, 4.0);
    }
}

fn camera_follow(
    player: Query<&Transform, With<Player>>,
    mut camera: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    if let Ok(player_transform) = player.get_single() {
        if let Ok(mut camera_transform) = camera.get_single_mut() {
            camera_transform.translation = player_transform.translation;
        }
    }
}
