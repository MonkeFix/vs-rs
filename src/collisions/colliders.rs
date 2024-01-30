use std::sync::atomic::{AtomicU32, Ordering};

use bevy::{
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
};

use super::{
    circle_to_circle, rect_to_circle, rect_to_rect, shapes::{ColliderShape, ColliderShapeType}, spatial_hash::SpatialHash, store::{ColliderStore, ALL_LAYERS}, ColliderId, CollisionResultRef, RaycastHit
};



#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct Collider {
    pub id: ColliderId,
    /// The underlying `ColliderShape` of the `Collider`.
    pub shape: ColliderShape,
    /// If this collider is a trigger it will not cause collisions but it will still trigger events.
    pub is_trigger: bool,
    /// TODO. `local_offset` is added to `shape.position` to get the final position for the collider
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
    pub fn new(shape_type: ColliderShapeType) -> Self {
        let bounds = match shape_type {
            ColliderShapeType::Circle { radius } => {
                super::Rect::new(0.0, 0.0, radius * 2.0, radius * 2.0)
            }
            ColliderShapeType::Box { width, height } => super::Rect::new(0.0, 0.0, width, height),
        };

        Self {
            id: ColliderId(0),
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
    pub fn collides_with<'a>(&self, other: &'a Collider) -> Option<CollisionResultRef<'a>> {
        if self.is_trigger || other.is_trigger {
            return None;
        }

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
            res.collider = Some(other);
            return Some(res);
        }

        None
    }

    /// Checks if this Collider with motion applied (delta movement vector) collides
    /// with collider. If it does, true will be returned and result will be populated
    ///  with collision data.
    pub fn collides_with_motion<'a>(
        &mut self,
        motion: Vec2,
        other: &'a Collider,
    ) -> Option<CollisionResultRef<'a>> {
        if self.is_trigger || other.is_trigger {
            return None;
        }

        // alter the shapes position so that it is in the place it would be after movement
        // so we can check for overlaps
        let old_pos = self.position();
        self.shape.position += motion;
        self.shape.bounds.x += motion.x;
        self.shape.bounds.y += motion.y;

        let res = self.collides_with(other);

        // return the shapes position to where it was before the check
        self.shape.position = old_pos;
        self.shape.bounds.x = old_pos.x;
        self.shape.bounds.y = old_pos.y;

        res
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

    pub(crate) fn update_from_transform(&mut self, transform: &Transform) {
        if !self.needs_update(transform) {
            return;
        }

        self.set_position(Vec2::new(transform.translation.x, transform.translation.y));

        self.recalc_bounds();
    }

    fn needs_update(&self, transform: &Transform) -> bool {
        !self.is_registered
            || self.shape.position.x != transform.translation.x
            || self.shape.position.y != transform.translation.y
            || self.shape.center.x != transform.translation.x
            || self.shape.center.y != transform.translation.y
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash)]
pub struct ColliderComponent {
    pub id: ColliderId,
}

impl ColliderComponent {
    pub fn new(collider_set: &mut ColliderStore, shape_type: ColliderShapeType) -> Self {
        collider_set.create_and_register(shape_type)
    }
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub collider: ColliderComponent,
}


pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColliderStore::default())
            .add_systems(FixedUpdate, (update_positions, on_collider_added));
    }
}

fn update_positions(
    mut collider_set: ResMut<ColliderStore>,
    colliders: Query<(&ColliderComponent, &Transform), Changed<Transform>>,
) {
    for (collider, transform) in &colliders {
        collider_set.update_single(collider, transform);
    }
}

fn on_collider_added(
    mut collider_set: ResMut<ColliderStore>,
    colliders: Query<(&ColliderComponent, &Transform), Added<ColliderComponent>>,
) {
    for (col, transform) in &colliders {
        collider_set.added_with_transform(col, transform);
    }
}
