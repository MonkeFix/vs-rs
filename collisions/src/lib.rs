use bevy::prelude::*;
use colliders::Collider;

pub mod colliders;
pub mod plugin;
pub mod prelude;
pub mod shape_tests;
pub mod shapes;
mod spatial_hash;
pub mod store;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct ColliderId(pub u32);

#[derive(Debug, Default, Clone, Copy)]
pub struct CollisionResult {
    pub collider: Option<ColliderId>,
    pub normal: Vec2,
    pub min_translation: Vec2,
    pub point: Vec2,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CollisionResultRef<'a> {
    pub collider: Option<&'a Collider>,
    pub normal: Vec2,
    pub min_translation: Vec2,
    pub point: Vec2,
}

impl CollisionResult {
    pub fn invert(&mut self) {
        self.normal.x = -self.normal.x;
        self.normal.y = -self.normal.y;

        self.min_translation.x = -self.min_translation.x;
        self.min_translation.y = -self.min_translation.y;
    }

    pub fn from_ref(result: &CollisionResultRef) -> Self {
        let collider = result.collider.map(|col| col.id);

        Self {
            collider,
            normal: result.normal,
            min_translation: result.min_translation,
            point: result.point,
        }
    }
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
    pub collider: Option<ColliderId>,
    pub fraction: f32,
    pub distance: f32,
    pub point: Vec2,
    pub normal: Vec2,
    pub centroid: Vec2,
}
