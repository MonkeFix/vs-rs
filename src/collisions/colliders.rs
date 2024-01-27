use bevy::{
    ecs::component::{Component, SparseStorage},
    math::{Rect, Vec2},
};

pub enum ColliderType {
    Circle { center: Vec2, radius: f32 },
    Box,
    Polygon,
}

pub struct Collider {
    pub bounds: Rect,
    pub(crate) registered_physics_bounds: Rect,
}
