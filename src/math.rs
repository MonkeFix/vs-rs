use bevy::prelude::*;
use rand::{thread_rng, Rng};

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

pub fn rng_f32(min: f32, max: f32) -> f32 {
    let mut rand = thread_rng();
    rand.next_f32() * (max - min) + min
}

pub fn rng_vec2(min_length: f32, max_length: f32) -> Vec2 {
    let mut rand = thread_rng();
    let theta: f64 = rand.next_f64() * 2.0 * std::f64::consts::PI;
    let length: f32 = rng_f32(min_length, max_length);

    Vec2::new(length * theta.cos() as f32, length * theta.sin() as f32)
}
