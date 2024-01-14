use bevy::prelude::*;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build (&self, app: &mut App) {
        app.add_systems(Update, movement);
    }
}

fn axis(negative: bool, positive: bool) -> f32 {
    ((positive as i8) - (negative as i8)) as f32
}

fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
) {
    if let Ok((mut transform, mut ortho)) = query.get_single_mut() {
        let mut direction = Vec3::ZERO;

        let is_pressed = |key| keyboard_input.pressed(key);
        let x = axis(is_pressed(KeyCode::Left), is_pressed(KeyCode::Right));
        let y = axis(is_pressed(KeyCode::Down), is_pressed(KeyCode::Up));
        let magnitude = (x.powi(2) + y.powi(2)).sqrt();

        if magnitude != 0. {
            direction = Vec3::new(x / magnitude, y / magnitude, 0.);
        }

        #[cfg(debug_assertions)]
        if keyboard_input.pressed(KeyCode::Z) {
            ortho.scale += 0.1;
        }

        #[cfg(debug_assertions)]
        if keyboard_input.pressed(KeyCode::X) {
            ortho.scale -= 0.1;
        }

        if ortho.scale < 0.5 {
            ortho.scale = 0.5;
        }

        let z = transform.translation.z;
        transform.translation += time.delta_seconds() * direction * 500.;
        // Important! We need to restore the Z values when moving the camera around.
        // Bevy has a specific camera setup and this can mess with how our layers are shown.
        transform.translation.z = z;
    }
}
