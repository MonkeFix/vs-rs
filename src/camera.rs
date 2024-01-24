use bevy::prelude::*;

use crate::player::Player;

pub struct CameraMovementPlugin;

impl Plugin for CameraMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_follow);
        #[cfg(debug_assertions)]
        app.add_systems(Update, camera_zoom);
    }
}

fn camera_zoom(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
) {
    if let Ok(mut ortho) = query.get_single_mut() {
        if keyboard_input.pressed(KeyCode::Z) {
            ortho.scale += 0.1;
        }
        if keyboard_input.pressed(KeyCode::X) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }
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
