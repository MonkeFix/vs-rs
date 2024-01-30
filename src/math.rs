use bevy::prelude::*;

pub fn truncate_vec2(vec2: Vec2, max: f32) -> Vec2 {
    if vec2.length() > max {
        let vec2 = vec2.normalize_or_zero();
        return vec2 * max;
    }

    vec2
}

pub fn floor_to_int(f: f32) -> i32 {
    (f as f64).floor() as i32
}

pub fn approach(start: f32, end: f32, shift: f32) -> f32 {
    if start < end {
        return end.min(start + shift);
    }

    end.max(start - shift)
}
