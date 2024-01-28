use bevy::prelude::*;

use super::{
    circle_to_circle, rect_to_circle, rect_to_rect,
    shapes::{ColliderShape, ColliderShapeType},
    CollisionResult,
};

#[derive(Component, Debug, Clone)]
pub struct Collider {
    pub shape: ColliderShape,
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub collider: Collider,
}

impl Collider {
    pub fn new(shape: ColliderShape) -> Self {
        Self { shape }
    }

    /// Checks if this shape overlaps any other `Collider`.
    pub fn overlaps(&self, other: &Collider) -> bool {
        let position = self.position();
        match self.shape.shape_type {
            ColliderShapeType::Circle { radius: r1 } => match other.shape.shape_type {
                ColliderShapeType::Circle { radius: r2 } => {
                    circle_to_circle(position, r1, other.position(), r2)
                }
                ColliderShapeType::Box { width, height } => rect_to_circle(
                    other.position().x,
                    other.position().y,
                    width,
                    height,
                    self.position(),
                    r1,
                ),
            },
            ColliderShapeType::Box {
                width: w1,
                height: h1,
            } => match other.shape.shape_type {
                ColliderShapeType::Circle { radius } => {
                    rect_to_circle(position.x, position.y, w1, h1, other.position(), radius)
                }
                ColliderShapeType::Box {
                    width: w2,
                    height: h2,
                } => rect_to_rect(
                    position.x,
                    position.y,
                    w1,
                    h1,
                    other.position().x,
                    other.position().y,
                    w2,
                    h2,
                ),
            },
        }
    }

    /// Checks if this Collider collides with collider. If it does, 
    /// true will be returned and result will be populated with collision data.
    pub fn collides_with(&self, other: &Collider) -> Option<CollisionResult> {
        let res = match self.shape.shape_type {
            ColliderShapeType::Circle { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => {
                    super::shapes::collisions::circle_to_circle(&self.shape, &other.shape)
                }
                ColliderShapeType::Box { .. } => {
                    super::shapes::collisions::circle_to_box(&self.shape, &other.shape)
                }
            },
            ColliderShapeType::Box { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => {
                    super::shapes::collisions::circle_to_box(&other.shape, &self.shape)
                }
                ColliderShapeType::Box { .. } => {
                    super::shapes::collisions::circle_to_circle(&other.shape, &self.shape)
                }
            },
        };

        if let Some(mut res) = res {
            res.collider = Some(other.clone());
            return Some(res);
        }

        None
    }

    /// Checks if this Collider with motion applied (delta movement vector) collides 
    /// with collider. If it does, true will be returned and result will be populated
    ///  with collision data.
    pub fn collides_with_motion(
        &mut self,
        motion: Vec2,
        other: &Collider,
    ) -> Option<CollisionResult> {
        // alter the shapes position so that it is in the place it would be after movement
        // so we can check for overlaps
        let old_pos = self.position();
        self.shape.position += motion;

        let res = self.collides_with(other);

        // return the shapes position to where it was before the check
        self.shape.position = old_pos;

        res
    }

    fn position(&self) -> Vec2 {
        self.shape.position
    }
}
