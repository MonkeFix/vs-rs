use bevy::{log, prelude::*, utils::HashMap};

use super::{
    circle_to_circle, rect_to_circle, rect_to_rect,
    shapes::{ColliderShape, ColliderShapeType},
    CollisionResultRef,
};

#[derive(Component, Debug, Clone, Reflect)]
pub struct Collider {
    pub shape: ColliderShape,
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
            shape: ColliderShape {
                shape_type,
                position: Vec2::ZERO,
                center: Vec2::ZERO,
                bounds,
            },
        }
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

    pub(crate) fn update_from_transform(&mut self, transform: &Transform) {
        self.shape.position.x = transform.translation.x;
        self.shape.position.y = transform.translation.y;
        self.shape.center = self.shape.position;

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
                self.shape.bounds.x = transform.translation.x - hw;
                self.shape.bounds.y = transform.translation.y - hh;
                self.shape.bounds.width = width;
                self.shape.bounds.height = height;
            }
        }
    }

    pub(crate) fn needs_update(&self, transform: &Transform) -> bool {
        self.shape.position.x != transform.translation.x
            || self.shape.position.y != transform.translation.y
            || self.shape.center.x != transform.translation.x
            || self.shape.center.y != transform.translation.y
    }
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub collider: Collider,
}

#[derive(Resource, Default)]
pub struct ColliderSet {
    pub map: HashMap<u32, Collider>,
}

impl ColliderSet {
    pub fn register(&mut self, collider: Collider, entity: Entity) {
        log::info!("Entity {entity:?} | added collider: {collider:?}");

        match self.map.try_insert(entity.index(), collider) {
            Ok(_res) => {}
            Err(_err) => {
                // log::error!("Error while registering collider: {err:?}");
            }
        }
    }

    pub fn deregister(&mut self, entity: Entity) {
        self.map.remove(&entity.index());
    }

    pub fn update(&mut self, entity: Entity, transform: &Transform) {
        let col = self.map.get_mut(&entity.index());
        if let Some(col) = col {
            col.update_from_transform(transform);
        }
    }
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColliderSet::default()).add_systems(
            FixedUpdate,
            (
                register_collider,
                update_positions,
                collider_changed_transform,
            ),
        );
    }
}

fn update_positions(mut colliders: Query<(&mut Collider, &Transform)>) {
    for (mut collider, transform) in &mut colliders {
        if collider.needs_update(transform) {
            collider.update_from_transform(transform);
        }
    }
}

fn collider_changed_transform(
    mut collider_set: ResMut<ColliderSet>,
    changed_query: Query<(&Transform, Entity), Changed<Collider>>,
) {
    for (transform, entity) in &changed_query {
        collider_set.update(entity, transform);
    }
}

fn register_collider(
    mut collider_set: ResMut<ColliderSet>,
    colliders: Query<(&Collider, &Transform, Entity), Added<Collider>>,
) {
    for (col, transform, entity) in &colliders {
        let mut col = col.clone();
        col.update_from_transform(transform);
        collider_set.register(col, entity);
    }
}
