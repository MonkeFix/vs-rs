use bevy::prelude::*;
use colliders::Collider;

pub mod colliders;
pub mod shapes;
pub mod spatial_hash;
pub mod tests;

/// Represents a collision result of a `Collider` with another `Collider`.
#[derive(Debug, Default, Clone, Copy)]
pub struct CollisionResultRef<'a> {
    /// Another `Collider` the main one collided with.
    pub collider: Option<&'a Collider>,
    /// A normal vector of the collision.
    pub normal: Vec2,
    /// Min translation required to correctly resolve the collision.
    pub min_translation: Vec2,
    pub point: Vec2,
}

impl<'a> CollisionResultRef<'a> {
    pub fn invert(&mut self) {
        self.normal.x = -self.normal.x;
        self.normal.y = -self.normal.y;

        self.min_translation.x = -self.min_translation.x;
        self.min_translation.y = -self.min_translation.y;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RaycastHit {
    pub collider: Option<Entity>,
    pub fraction: f32,
    pub distance: f32,
    pub point: Vec2,
    pub normal: Vec2,
    pub centroid: Vec2,
}

pub const ALL_LAYERS: i32 = -1;
