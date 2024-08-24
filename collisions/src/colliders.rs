use bevy::prelude::*;
use common::FRect;
use common::Position;

use super::{
    shape_tests::*,
    shapes::{ColliderShape, ColliderShapeType},
    store::ALL_LAYERS,
    ColliderId, CollisionResultRef, RaycastHit,
};

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct ColliderData {
    /// The underlying `ColliderShape` of the `Collider`.
    pub shape_type: ColliderShapeType,
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
}

impl Default for ColliderData {
    fn default() -> Self {
        Self {
            shape_type: ColliderShapeType::None,
            is_trigger: false,
            local_offset: Vec2::ZERO,
            physics_layer: 1 << 0,
            collides_with_layers: ALL_LAYERS,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Collider {
    pub id: ColliderId,
    pub entity: Option<Entity>,
    pub data: ColliderData,
    pub shape: ColliderShape,
    pub(crate) is_registered: bool,
}

impl Collider {
    pub fn new(data: ColliderData, entity: Option<Entity>) -> Self {
        let bounds = match data.shape_type {
            ColliderShapeType::Circle { radius } => {
                FRect::new(0.0, 0.0, radius * 2.0, radius * 2.0)
            }
            ColliderShapeType::Box { width, height } => FRect::new(0.0, 0.0, width, height),
            ColliderShapeType::None => FRect::new(0.0, 0.0, 0.0, 0.0),
        };

        let mut shape = ColliderShape::new(data.shape_type);
        shape.bounds = bounds;

        Self {
            id: ColliderId(0),
            entity,
            data,
            shape,
            is_registered: false,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.shape.position
    }

    pub fn absolute_position(&self) -> Vec2 {
        self.shape.position + self.data.local_offset
    }

    pub fn bounds(&self) -> FRect {
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
                    position + self.data.local_offset,
                    r1,
                    other.position() + other.data.local_offset,
                    r2,
                ),
                ColliderShapeType::Box { width, height } => rect_to_circle(
                    other.position().x + other.data.local_offset.x,
                    other.position().y + other.data.local_offset.y,
                    width,
                    height,
                    self.position() + self.data.local_offset,
                    r1,
                ),
                ColliderShapeType::None => false,
            },
            ColliderShapeType::Box {
                width: w1,
                height: h1,
            } => match other.shape.shape_type {
                ColliderShapeType::Circle { radius } => rect_to_circle(
                    position.x + self.data.local_offset.x,
                    position.y + self.data.local_offset.y,
                    w1,
                    h1,
                    other.position() + other.data.local_offset,
                    radius,
                ),
                ColliderShapeType::Box {
                    width: w2,
                    height: h2,
                } => rect_to_rect(
                    position + self.data.local_offset,
                    Vec2::new(w1, h1),
                    other.position() + other.data.local_offset,
                    Vec2::new(w2, h2),
                ),
                ColliderShapeType::None => false,
            },
            ColliderShapeType::None => false,
        }
    }

    /// Checks if this Collider collides with collider. If it does,
    /// true will be returned and result will be populated with collision data.
    pub fn collides_with<'a>(&self, other: &'a Collider) -> Option<CollisionResultRef<'a>> {
        if self.data.is_trigger || other.data.is_trigger {
            return None;
        }

        let res = match self.data.shape_type {
            ColliderShapeType::Circle { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_circle(
                    &self.shape,
                    &other.shape,
                    self.data.local_offset,
                    other.data.local_offset,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::circle_to_box(
                    &self.shape,
                    &other.shape,
                    self.data.local_offset,
                    other.data.local_offset,
                ),
                ColliderShapeType::None => None,
            },
            ColliderShapeType::Box { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_box(
                    &other.shape,
                    &self.shape,
                    other.data.local_offset,
                    self.data.local_offset,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::box_to_box(
                    &self.shape,
                    &other.shape,
                    self.data.local_offset,
                    other.data.local_offset,
                ),
                ColliderShapeType::None => None,
            },
            ColliderShapeType::None => None,
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
        if self.data.is_trigger || other.data.is_trigger {
            return None;
        }

        let res = match self.shape.shape_type {
            ColliderShapeType::Circle { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_circle(
                    &self.shape,
                    &other.shape,
                    self.data.local_offset + motion,
                    other.data.local_offset,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::circle_to_box(
                    &self.shape,
                    &other.shape,
                    self.data.local_offset + motion,
                    other.data.local_offset,
                ),
                ColliderShapeType::None => None,
            },
            ColliderShapeType::Box { .. } => match other.shape.shape_type {
                ColliderShapeType::Circle { .. } => super::shapes::collisions::circle_to_box(
                    &other.shape,
                    &self.shape,
                    other.data.local_offset,
                    self.data.local_offset + motion,
                ),
                ColliderShapeType::Box { .. } => super::shapes::collisions::box_to_box(
                    &self.shape,
                    &other.shape,
                    self.data.local_offset + motion,
                    other.data.local_offset,
                ),
                ColliderShapeType::None => None,
            },
            ColliderShapeType::None => None,
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
                self.shape.bounds.x = self.shape.center.x + self.data.local_offset.x - radius;
                self.shape.bounds.y = self.shape.center.y + self.data.local_offset.y - radius;
                self.shape.bounds.width = radius * 2.0;
                self.shape.bounds.height = radius * 2.0;
            }
            ColliderShapeType::Box { width, height } => {
                let hw = width / 2.0;
                let hh = height / 2.0;
                self.shape.bounds.x = self.shape.position.x + self.data.local_offset.x - hw;
                self.shape.bounds.y = self.shape.position.y + self.data.local_offset.y - hh;
                self.shape.bounds.width = width;
                self.shape.bounds.height = height;
            }
            ColliderShapeType::None => {}
        };
    }

    pub fn collides_with_line(&self, start: Vec2, end: Vec2) -> Option<RaycastHit> {
        match self.shape.shape_type {
            ColliderShapeType::Circle { .. } => {
                super::shapes::collisions::line_to_circle(start, end, &self.shape)
            }
            ColliderShapeType::Box { .. } => todo!(),
            ColliderShapeType::None => None,
        }
    }

    pub fn contains_point(&self, point: Vec2) -> bool {
        match self.shape.shape_type {
            ColliderShapeType::Circle { radius } => {
                (point - self.shape.position).length_squared() <= radius * radius
            }
            ColliderShapeType::Box { .. } => self.bounds().contains(point),
            ColliderShapeType::None => false,
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
