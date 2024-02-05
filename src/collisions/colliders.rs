#![allow(dead_code)]

use crate::movement::Position;
use bevy::prelude::*;

use super::{
    circle_to_circle, rect_to_circle, rect_to_rect,
    shapes::{ColliderShape, ColliderShapeType},
    store::ALL_LAYERS,
    ColliderId, CollisionResultRef, RaycastHit,
};

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Collider {
    pub id: ColliderId,
    pub entity: Option<Entity>,
    /// The underlying `ColliderShape` of the `Collider`.
    pub shape: ColliderShape,
    /// If this collider is a trigger it will not cause collisions but it will still trigger events.
    pub is_trigger: bool,
    /// `local_offset` is added to `shape.position` to get the final position for the collider
    /// geometry. This allows to add multiple Colliders to an Entity and position them separately
    /// and also lets you set the point of scale.
    pub local_offset: Vec2,
    /// `physics_layer` can be used as a filter when dealing with collisions. It is a bitmask.
    pub physics_layer: i32,
    /// Layer mask of all the layers this Collider should collide with.
    /// Default is all layers.
    pub collides_with_layers: i32,
    pub(crate) is_registered: bool,
}

impl Collider {
    pub fn new(shape_type: ColliderShapeType, entity: Option<Entity>) -> Self {
        let bounds = match shape_type {
            ColliderShapeType::Circle { radius } => {
                super::Rect::new(0.0, 0.0, radius * 2.0, radius * 2.0)
            }
            ColliderShapeType::Box { width, height } => super::Rect::new(0.0, 0.0, width, height),
        };

        Self {
            id: ColliderId(0),
            entity,
            shape: ColliderShape {
                shape_type,
                position: Vec2::ZERO,
                center: Vec2::ZERO,
                bounds,
            },
            is_trigger: false,
            local_offset: Vec2::ZERO,
            physics_layer: 1 << 0,
            collides_with_layers: ALL_LAYERS,
            is_registered: false,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.shape.position
    }

    pub fn absolute_position(&self) -> Vec2 {
        self.shape.position + self.local_offset
    }

    pub fn bounds(&self) -> super::Rect {
        self.shape.bounds
    }

    pub fn center(&self) -> Vec2 {
        self.shape.center
    }

    /// Checks if this shape overlaps any other `Collider`.
    pub fn overlaps(&self, other: &Collider) -> bool {
        let position = self.position();
        match self.shape.shape_type {
            ColliderShapeType::Circle { radius: r1 } => match other.shape.shape_type {
                ColliderShapeType::Circle { radius: r2 } => circle_to_circle(
                    position + self.local_offset,
                    r1,
                    other.position() + other.local_offset,
                    r2,
                ),
                ColliderShapeType::Box { width, height } => rect_to_circle(
                    other.position().x + other.local_offset.x,
                    other.position().y + other.local_offset.y,
                    width,
                    height,
                    self.position() + self.local_offset,
                    r1,
                ),
            },
            ColliderShapeType::Box {
                width: w1,
                height: h1,
            } => match other.shape.shape_type {
                ColliderShapeType::Circle { radius } => rect_to_circle(
                    position.x + self.local_offset.x,
                    position.y + self.local_offset.y,
                    w1,
                    h1,
                    other.position() + other.local_offset,
                    radius,
                ),
                ColliderShapeType::Box {
                    width: w2,
                    height: h2,
                } => rect_to_rect(
                    position + self.local_offset,
                    Vec2::new(w1, h1),
                    other.position() + other.local_offset,
                    Vec2::new(w2, h2),
                ),
            },
        }
    }

    /// Checks if this Collider collides with collider. If it does,
    /// true will be returned and result will be populated with collision data.
    pub fn collides_with<'a>(&self, other: &'a Collider) -> Option<CollisionResultRef<'a>> {
        if self.is_trigger || other.is_trigger {
            return None;
        }

        let res = match self.shape.shape_type {
            ColliderShapeType::Circle { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_circle(
                    &self.shape,
                    &other.shape,
                    self.local_offset,
                    other.local_offset,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::circle_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset,
                    other.local_offset,
                ),
            },
            ColliderShapeType::Box { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_box(
                    &other.shape,
                    &self.shape,
                    other.local_offset,
                    self.local_offset,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::box_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset,
                    other.local_offset,
                ),
            },
        };

        if let Some(mut res) = res {
            res.collider = Some(other);
            return Some(res);
        }

        None
    }

    /// Checks if this Collider with motion applied (delta movement vector) collides
    /// with collider. If it does, true will be returned and result will be populated
    ///  with collision data.
    pub fn collides_with_motion<'a>(
        &self,
        other: &'a Collider,
        motion: Vec2,
    ) -> Option<CollisionResultRef<'a>> {
        if self.is_trigger || other.is_trigger {
            return None;
        }

        let res = match self.shape.shape_type {
            ColliderShapeType::Circle { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_circle(
                    &self.shape,
                    &other.shape,
                    self.local_offset + motion,
                    other.local_offset,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::circle_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset + motion,
                    other.local_offset,
                ),
            },
            ColliderShapeType::Box { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_box(
                    &other.shape,
                    &self.shape,
                    other.local_offset,
                    self.local_offset + motion,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::box_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset + motion,
                    other.local_offset,
                ),
            },
        };

        if let Some(mut res) = res {
            res.collider = Some(other);
            return Some(res);
        }

        None
    }

    pub fn recalc_bounds(&mut self) {
        match self.shape.shape_type {
            ColliderShapeType::Circle { radius } => {
                self.shape.bounds.x = self.shape.center.x - radius;
                self.shape.bounds.y = self.shape.center.y - radius;
                self.shape.bounds.width = radius * 2.0;
                self.shape.bounds.height = radius * 2.0;
            }
            ColliderShapeType::Box { width, height } => {
                let hw = width / 2.0;
                let hh = height / 2.0;
                self.shape.bounds.x = self.shape.position.x - hw;
                self.shape.bounds.y = self.shape.position.y - hh;
                self.shape.bounds.width = width;
                self.shape.bounds.height = height;
            }
        };
    }

    pub fn collides_with_line(&self, start: Vec2, end: Vec2) -> Option<RaycastHit> {
        match self.shape.shape_type {
            ColliderShapeType::Circle { .. } => {
                super::shapes::collisions::line_to_circle(start, end, &self.shape)
            }
            ColliderShapeType::Box { .. } => todo!(),
        }
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        match self.shape.shape_type {
            ColliderShapeType::Circle { radius } => {
                (point - self.shape.position).length_squared() <= radius * radius
            }
            ColliderShapeType::Box { .. } => self.bounds().contains(point),
        }
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.shape.position = position;
        self.shape.center = self.shape.position;

        self.recalc_bounds();
    }

    pub(crate) fn update_from_position(&mut self, position: &Position) {
        if !self.needs_update(position) {
            return;
        }

        self.set_position(position.0);

        self.recalc_bounds();
    }

    fn needs_update(&self, position: &Position) -> bool {
        !self.is_registered || self.shape.position != position.0 || self.shape.center != position.0
    }
}
