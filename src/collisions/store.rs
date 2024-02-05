#![allow(dead_code)]

use std::sync::atomic::{AtomicU32, Ordering};

use crate::movement::Position;
use bevy::{
    prelude::*,
    utils::{hashbrown::HashSet, HashMap},
};

use super::{
    colliders::Collider, plugin::ColliderComponent, shapes::ColliderShapeType,
    spatial_hash::SpatialHash, ColliderId, RaycastHit,
};

pub const ALL_LAYERS: i32 = -1;

pub trait ColliderIdResolver {
    fn get(&self, id: impl Into<ColliderId>) -> Option<&Collider>;
    fn get_mut(&mut self, id: impl Into<ColliderId>) -> Option<&mut Collider>;
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
        let collider = Collider::new(shape_type, None);
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

    pub fn remove(&mut self, id: impl Into<ColliderId>) -> Option<Collider> {
        let id = id.into();
        let col = self.colliders.get_mut(&id);
        col.as_ref()?;

        let col: &mut Collider = col.unwrap();
        col.is_registered = false;

        self.spatial_hash.remove(col);
        self.colliders.remove(&id)
    }

    pub fn aabb_broadphase(
        &self,
        rect: super::Rect,
        layer_mask: Option<i32>,
    ) -> HashSet<ColliderId> {
        let layer_mask = layer_mask.unwrap_or(ALL_LAYERS);

        self.spatial_hash
            .aabb_broadphase(&rect, None, layer_mask, |id| self.colliders.get(id))
    }

    pub fn aabb_broadphase_excluding_self(
        &self,
        self_collider: ColliderId,
        rect: super::Rect,
        layer_mask: Option<i32>,
    ) -> HashSet<ColliderId> {
        let layer_mask = layer_mask.unwrap_or(ALL_LAYERS);

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
        let layer_mask = layer_mask.unwrap_or(ALL_LAYERS);

        self.spatial_hash
            .linecast(start, end, layer_mask, |id| self.colliders.get(id))
    }

    pub fn overlap_circle(
        &self,
        circle_center: Vec2,
        radius: f32,
        excluding_collider: Option<ColliderId>,
        layer_mask: Option<i32>,
    ) -> Vec<ColliderId> {
        let layer_mask = layer_mask.unwrap_or(ALL_LAYERS);

        let mut results = vec![];

        let _count = self.spatial_hash.overlap_circle(
            circle_center,
            radius,
            excluding_collider,
            &mut results,
            layer_mask,
            |id| self.colliders.get(id),
        );

        results
    }

    pub fn overlap_rectangle(
        &self,
        rect: super::Rect,
        excluding_collider: Option<ColliderId>,
        layer_mask: Option<i32>,
    ) -> Vec<ColliderId> {
        let layer_mask = layer_mask.unwrap_or(ALL_LAYERS);

        let mut results = vec![];

        let _count = self.spatial_hash.overlap_rectangle(
            &rect,
            excluding_collider,
            &mut results,
            layer_mask,
            |id| self.colliders.get(id),
        );

        results
    }

    pub(crate) fn clear_hash(&mut self) {
        self.spatial_hash.clear();
    }

    pub(crate) fn update_single(&mut self, id: ColliderId, position: &Position) {
        if let Some(col) = self.colliders.get(&id) {
            if col.is_registered {
                self.spatial_hash.remove(col);
            }
        }

        if let Some(col) = self.get_mut(id) {
            col.is_registered = true;
            col.update_from_position(position);
        }

        if let Some(col) = self.colliders.get(&id) {
            self.spatial_hash.register(col);
        }
    }

    pub(crate) fn added_with_position(&mut self, id: ColliderId, position: &Position) {
        if let Some(col) = self.get_mut(id) {
            col.is_registered = true;
            col.update_from_position(position);
        }

        if let Some(col) = self.colliders.get(&id) {
            self.spatial_hash.register(col);
        }
    }

    pub(crate) fn set_entity(&mut self, id: ColliderId, entity: Entity) {
        if let Some(col) = self.get_mut(id) {
            col.entity = Some(entity);
        }
    }
}

impl ColliderIdResolver for ColliderStore {
    fn get(&self, id: impl Into<ColliderId>) -> Option<&Collider> {
        self.colliders.get(&id.into())
    }

    fn get_mut(&mut self, id: impl Into<ColliderId>) -> Option<&mut Collider> {
        self.colliders.get_mut(&id.into())
    }
}
