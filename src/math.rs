use bevy::prelude::*;

pub fn truncate_vec2(vec2: Vec2, max: f32) -> Vec2 {
    if vec2.length() > max {
        let vec2 = vec2.normalize_or_zero();
        return vec2 * max;
    }

    vec2
}
