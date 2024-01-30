use std::sync::atomic::{AtomicU32, Ordering};

use bevy::{
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
};

use super::{
    circle_to_circle, rect_to_circle, rect_to_rect,
    shapes::{ColliderShape, ColliderShapeType},
    spatial_hash::SpatialHash,
    ColliderId, CollisionResultRef, RaycastHit,
};

pub const ALL_LAYERS: i32 = -1;

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

pub trait ColliderIdResolver {
    fn get(&self, id: ColliderId) -> Option<&Collider>;
    fn get_mut(&mut self, id: ColliderId) -> Option<&mut Collider>;
}

static COLLIDER_ID_GEN: AtomicU32 = AtomicU32::new(0);

#[derive(Resource)]
pub struct ColliderStore {
    pub colliders: HashMap<ColliderId, Collider>,
    spatial_hash: SpatialHash,
}

impl Default for ColliderStore {
    fn default() -> Self {
        Self {
            colliders: HashMap::new(),
            spatial_hash: SpatialHash::new(100),
        }
    }
}

impl ColliderStore {
    pub fn new(cell_size: i32) -> Self {
        Self {
            spatial_hash: SpatialHash::new(cell_size),
            ..default()
        }
    }

    pub fn create_and_register(&mut self, shape_type: ColliderShapeType) -> ColliderComponent {
        let collider = Collider::new(shape_type);
        let id = self.register(collider);

        ColliderComponent { id }
    }

    pub fn register(&mut self, mut collider: Collider) -> ColliderId {
        let id = COLLIDER_ID_GEN.fetch_add(1, Ordering::SeqCst);
        let id = ColliderId(id);
        collider.id = id;

        self.colliders.insert(id, collider);

        id
    }

    pub fn from_component(&self, component: &ColliderComponent) -> Option<&Collider> {
        self.colliders.get(&component.id)
    }

    pub fn from_component_mut(&mut self, component: &ColliderComponent) -> Option<&mut Collider> {
        self.colliders.get_mut(&component.id)
    }

    pub fn remove(&mut self, component: ColliderComponent) -> Option<Collider> {
        let col = self.colliders.get_mut(&component.id);
        if col.is_none() {
            return None;
        }

        let col: &mut Collider = col.unwrap();
        col.is_registered = false;

        self.spatial_hash.remove(&col);
        self.colliders.remove(&component.id)
    }

    pub fn get_as_components(&self) -> Vec<ColliderComponent> {
        let mut res = vec![];

        for (id, _v) in &self.colliders {
            res.push(ColliderComponent { id: *id });
        }

        res
    }

    pub fn aabb_broadphase(
        &self,
        rect: super::Rect,
        layer_mask: Option<i32>,
    ) -> HashSet<ColliderId> {
        let layer_mask = match layer_mask {
            Some(val) => val,
            None => ALL_LAYERS,
        };

        self.spatial_hash
            .aabb_broadphase(&rect, None, layer_mask, |id| self.colliders.get(id))
    }

    pub fn aabb_broadphase_excluding_self(
        &self,
        self_collider: ColliderId,
        rect: super::Rect,
        layer_mask: Option<i32>,
    ) -> HashSet<ColliderId> {
        let layer_mask = match layer_mask {
            Some(val) => val,
            None => ALL_LAYERS,
        };

        self.spatial_hash
            .aabb_broadphase(&rect, Some(self_collider), layer_mask, |id| {
                self.colliders.get(id)
            })
    }

    pub fn linecast(
        &self,
        start: Vec2,
        end: Vec2,
        layer_mask: Option<i32>,
    ) -> (i32, Vec<RaycastHit>) {
        let layer_mask = match layer_mask {
            Some(val) => val,
            None => ALL_LAYERS,
        };

        self.spatial_hash
            .linecast(start, end, layer_mask, |id| self.colliders.get(id))
    }

    pub fn overlap_circle(&self, circle_center: Vec2, radius: f32, layer_mask: Option<i32>) -> i32 {
        let layer_mask = match layer_mask {
            Some(val) => val,
            None => ALL_LAYERS,
        };

        let mut results = vec![ColliderId(0); 1];

        self.spatial_hash
            .overlap_circle(circle_center, radius, &mut results, layer_mask, |id| {
                self.colliders.get(id)
            })
    }

    pub fn overlap_rectangle(&self, rect: super::Rect, layer_mask: Option<i32>) -> i32 {
        let layer_mask = match layer_mask {
            Some(val) => val,
            None => ALL_LAYERS,
        };

        let mut results = vec![ColliderId(0); 1];

        self.spatial_hash
            .overlap_rectangle(&rect, &mut results, layer_mask, |id| self.colliders.get(id))
    }

    pub(crate) fn clear_hash(&mut self) {
        self.spatial_hash.clear();
    }

    pub(crate) fn update_single(&mut self, component: &ColliderComponent, transform: &Transform) {
        if let Some(col) = self.colliders.get(&component.id) {
            if col.is_registered {
                self.spatial_hash.remove(col);
            }
        }

        if let Some(col) = self.get_mut(component.id) {
            col.is_registered = true;
            col.update_from_transform(transform);
        }

        if let Some(col) = self.colliders.get(&component.id) {
            self.spatial_hash.register(col);
        }
    }

    pub(crate) fn added_with_transform(
        &mut self,
        component: &ColliderComponent,
        transform: &Transform,
    ) {
        if let Some(col) = self.get_mut(component.id) {
            col.is_registered = true;
            col.update_from_transform(transform);
        }

        if let Some(col) = self.colliders.get(&component.id) {
            self.spatial_hash.register(col);
        }
    }
}

impl ColliderIdResolver for ColliderStore {
    fn get(&self, id: ColliderId) -> Option<&Collider> {
        self.colliders.get(&id)
    }

    fn get_mut(&mut self, id: ColliderId) -> Option<&mut Collider> {
        self.colliders.get_mut(&id)
    }
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
