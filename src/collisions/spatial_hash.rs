use bevy::{
    ecs::component::{Component, SparseStorage},
    math::{Mat2, Mat3, Rect, Vec2},
    utils::{HashMap, HashSet},
};

use crate::math;

use super::colliders::Collider;

pub struct SpatialHash {
    cell_size: i32,
    inverse_cell_size: f32,
    cell_map: IntIntMap,
    pub grid_bounds: Rect,
}

impl SpatialHash {
    pub fn new(cell_size: i32) -> Self {
        Self {
            cell_size,
            inverse_cell_size: 1.0 / cell_size as f32,
            cell_map: IntIntMap::default(),
            grid_bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn register(&mut self, mut collider: Collider) {
        let bounds = collider.bounds;
        collider.registered_physics_bounds = bounds;
        let p1 = self.cell_coords(bounds.min.x, bounds.min.y);
        let p2 = self.cell_coords(bounds.max.x, bounds.max.y);
    }

    fn cell_coords(&self, x: f32, y: f32) -> (i32, i32) {
        (
            math::floor_to_int(x * self.inverse_cell_size),
            math::floor_to_int(y * self.inverse_cell_size),
        )
    }

    fn get_cell(&self, x: i32, y: i32) -> Option<&Vec<Collider>> {
        if let Some(collider) = self.cell_map.get(x, y) {
            return Some(collider);
        }

        None
    }
}

type ColliderList = Vec<Collider>;
#[derive(Default)]
struct IntIntMap {
    store: HashMap<i64, ColliderList>,
}

fn get_key(x: i32, y: i32) -> i64 {
    let shl = (x as i64).overflowing_shl(32);
    shl.0 | ((y as u32) as i64)
}

impl IntIntMap {
    pub fn insert(&mut self, x: i32, y: i32, colliders: ColliderList) {
        self.store.insert(get_key(x, y), colliders);
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&ColliderList> {
        self.store.get(&get_key(x, y))
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut ColliderList> {
        self.store.get_mut(&get_key(x, y))
    }

    pub fn clear(&mut self) {
        self.store.clear();
    }
}
