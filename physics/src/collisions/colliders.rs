use bevy::prelude::*;
use common::FRect;

use super::{
    shapes::{Shape, ShapeType},
    CollisionResultRef, RaycastHit, ALL_LAYERS,
};

/// Represents a collider with specified `Shape`, local offset vector and physics layer.
/// Also can be marked as trigger which skips all the collision resolution calculations
/// and just sends an event of type `InvokeTriggerEvent`.
/// You also can specify with which layers this collider collides with by setting
/// `collided_with_layers` field.
///
/// NOTE: `physics_layer` and `collides_with_layers` are bitmasks which allow to
/// specify multiple layers simultaneously. Example: 0b0010 (layer 2), 1 | 2 | 4 (layers 1, 2 and 4).
#[derive(Debug, Component, Clone, Reflect)]
pub struct Collider {
    /// Shape of collider. Defaults to `Shape::None`.
    pub shape: Shape,
    /// Whether collider is a trigger or not.
    pub is_trigger: bool,
    /// Local offset of collider. Can be used to store multiple colliders on the same entity.
    pub local_offset: Vec2,
    /// Bitmask of collider's physics layer. Defaults to `1`.
    pub physics_layer: i32,
    /// Bitmask of layers collider collides with. Ignores other layers for collision resolution,
    /// however still triggers `CollideEvent`. Defaults to `ALL_LAYERS`.
    pub collides_with_layers: i32,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            shape: Shape::new(super::shapes::ShapeType::None),
            is_trigger: false,
            local_offset: Vec2::ZERO,
            physics_layer: 1 << 0,
            collides_with_layers: ALL_LAYERS,
        }
    }
}

#[inline]
fn calc_bounds(shape: &Shape, local_offset: Vec2) -> FRect {
    match shape.shape_type {
        ShapeType::None => FRect::new(0.0, 0.0, 0.0, 0.0),
        ShapeType::Circle { radius } => FRect::new(
            shape.position.x + local_offset.x - radius,
            shape.position.y + local_offset.y - radius,
            radius * 2.0,
            radius * 2.0,
        ),
        ShapeType::Box { width, height } => FRect::new(
            shape.position.x + local_offset.x - width / 2.0,
            shape.position.y + local_offset.y - height / 2.0,
            width,
            height,
        ),
    }
}

#[inline]
fn get_center(shape: &Shape, local_offset: Vec2) -> Vec2 {
    match shape.shape_type {
        ShapeType::None => shape.position,
        ShapeType::Circle { radius } => Vec2::new(
            shape.position.x + local_offset.x + radius,
            shape.position.y + local_offset.y + radius,
        ),
        ShapeType::Box { width, height } => Vec2::new(
            shape.position.x + local_offset.x + width / 2.0,
            shape.position.y + local_offset.y + height / 2.0,
        ),
    }
}

impl Collider {
    /// Creates a new `Collider` with specified `ShapeType`.
    /// On creation bounds and center are calculated and set immediately.
    pub fn new(shape_type: ShapeType) -> Self {
        let mut shape = Shape::new(shape_type);
        let bounds = calc_bounds(&shape, Vec2::ZERO);
        let center = get_center(&shape, Vec2::ZERO);
        shape.bounds = bounds;
        shape.center = center;
        Self { shape, ..default() }
    }

    /// Gets current bounds rectangle.
    pub fn bounds(&self) -> FRect {
        self.shape.bounds
    }

    /// Gets `Collider`'s center.
    pub fn center(&self) -> Vec2 {
        self.shape.center
    }

    /// Gets `Collider`'s internal position without taking local offset into account.
    pub fn position(&self) -> Vec2 {
        self.shape.position
    }

    /// Gets `Collider`'s internal position with taking local offset into account.
    pub fn absolute_position(&self) -> Vec2 {
        self.shape.position + self.local_offset
    }

    /// Checks if this shape overlaps any other `Collider`.
    pub fn overlaps(&self, other: &Collider) -> bool {
        let position = self.position();
        match self.shape.shape_type {
            ShapeType::Circle { radius: r1 } => match other.shape.shape_type {
                ShapeType::Circle { radius: r2 } => super::tests::circle_to_circle(
                    position + self.local_offset,
                    r1,
                    other.position() + other.local_offset,
                    r2,
                ),
                ShapeType::Box { width, height } => super::tests::rect_to_circle(
                    other.position().x + other.local_offset.x,
                    other.position().y + other.local_offset.y,
                    width,
                    height,
                    self.position() + self.local_offset,
                    r1,
                ),
                ShapeType::None => false,
            },
            ShapeType::Box {
                width: w1,
                height: h1,
            } => match other.shape.shape_type {
                ShapeType::Circle { radius } => super::tests::rect_to_circle(
                    position.x + self.local_offset.x,
                    position.y + self.local_offset.y,
                    w1,
                    h1,
                    other.position() + other.local_offset,
                    radius,
                ),
                ShapeType::Box {
                    width: w2,
                    height: h2,
                } => super::tests::rect_to_rect(
                    position + self.local_offset,
                    Vec2::new(w1, h1),
                    other.position() + other.local_offset,
                    Vec2::new(w2, h2),
                ),
                ShapeType::None => false,
            },
            ShapeType::None => false,
        }
    }

    /// Checks if this Collider collides with collider. If it does,
    /// true will be returned and result will be populated with collision data.
    pub fn collides_with<'a>(&self, other: &'a Collider) -> Option<CollisionResultRef<'a>> {
        if self.is_trigger || other.is_trigger {
            return None;
        }

        let res = match self.shape.shape_type {
            ShapeType::Circle { .. } => match other.shape.shape_type {
                ShapeType::Circle { .. } => super::shapes::collisions::circle_to_circle(
                    &self.shape,
                    &other.shape,
                    self.local_offset,
                    other.local_offset,
                ),
                ShapeType::Box { .. } => super::shapes::collisions::circle_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset,
                    other.local_offset,
                ),
                ShapeType::None => None,
            },
            ShapeType::Box { .. } => match other.shape.shape_type {
                ShapeType::Circle { .. } => super::shapes::collisions::circle_to_box(
                    &other.shape,
                    &self.shape,
                    other.local_offset,
                    self.local_offset,
                ),
                ShapeType::Box { .. } => super::shapes::collisions::box_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset,
                    other.local_offset,
                ),
                ShapeType::None => None,
            },
            ShapeType::None => None,
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
        if self.is_trigger {
            return None;
        }

        let res = match self.shape.shape_type {
            ShapeType::Circle { .. } => match other.shape.shape_type {
                ShapeType::Circle { .. } => super::shapes::collisions::circle_to_circle(
                    &self.shape,
                    &other.shape,
                    self.local_offset + motion,
                    other.local_offset,
                ),
                ShapeType::Box { .. } => super::shapes::collisions::circle_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset + motion,
                    other.local_offset,
                ),
                ShapeType::None => None,
            },
            ShapeType::Box { .. } => match other.shape.shape_type {
                ShapeType::Circle { .. } => super::shapes::collisions::circle_to_box(
                    &other.shape,
                    &self.shape,
                    other.local_offset,
                    self.local_offset + motion,
                ),
                ShapeType::Box { .. } => super::shapes::collisions::box_to_box(
                    &self.shape,
                    &other.shape,
                    self.local_offset + motion,
                    other.local_offset,
                ),
                ShapeType::None => None,
            },
            ShapeType::None => None,
        };

        if let Some(mut res) = res {
            res.collider = Some(other);
            return Some(res);
        }

        None
    }

    /// Checks if `Collider` collides with a line from start and to end vectors.
    /// If it does, returns `Some(RaycastHit)`.
    pub fn collides_with_line(&self, start: Vec2, end: Vec2) -> Option<RaycastHit> {
        match self.shape.shape_type {
            ShapeType::Circle { .. } => {
                super::shapes::collisions::line_to_circle(start, end, &self.shape)
            }
            ShapeType::Box { .. } => todo!(),
            ShapeType::None => None,
        }
    }

    /// Checks whether `Collider` contains a point withit itself or not.
    pub fn contains_point(&self, point: Vec2) -> bool {
        match self.shape.shape_type {
            ShapeType::Circle { radius } => {
                (point - self.shape.position).length_squared() <= radius * radius
            }
            ShapeType::Box { .. } => self.bounds().contains(point),
            ShapeType::None => false,
        }
    }

    pub(crate) fn set_position(&mut self, position: Vec2) {
        self.shape.position = position;

        let bounds = self.calc_bounds();
        let center = self.get_center();
        self.shape.bounds = bounds;
        self.shape.center = center;
    }

    pub(crate) fn update_from_transform(&mut self, transform: &Transform) {
        self.set_position(transform.translation.xy());
    }

    fn calc_bounds(&self) -> FRect {
        calc_bounds(&self.shape, self.local_offset)
    }

    fn get_center(&self) -> Vec2 {
        get_center(&self.shape, self.local_offset)
    }
}
