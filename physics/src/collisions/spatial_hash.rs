use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use common::{math::floor_to_int, FRect};

use super::colliders::Collider;

pub type ColliderSet = HashSet<Entity>;

#[derive(Debug, Resource)]
pub struct SpatialHash {
    cell_size: i32,
    inverse_cell_size: f32,
    cell_map: IntIntMap,
    pub grid_bounds: FRect,
}

impl SpatialHash {
    pub fn new(cell_size: i32) -> Self {
        Self {
            cell_size,
            inverse_cell_size: 1.0 / cell_size as f32,
            cell_map: IntIntMap::default(),
            grid_bounds: FRect::new(0.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn register(&mut self, collider: &Collider, entity: Entity) {
        let bounds = collider.bounds();

        let p1 = self.cell_coords(bounds.x, bounds.y);
        let p2 = self.cell_coords(bounds.right(), bounds.bottom());

        if !self.grid_bounds.contains(p1) {
            self.grid_bounds = self.grid_bounds.union_vec2(&p1);
        }
        if !self.grid_bounds.contains(p2) {
            self.grid_bounds = self.grid_bounds.union_vec2(&p2);
        }

        for x in (p1.x as i32)..=(p2.x as i32) {
            for y in (p1.y as i32)..=(p2.y as i32) {
                if let Some(c) = self.get_cell_mut(x, y) {
                    c.insert(entity);
                } else {
                    let mut c = HashSet::new();
                    c.insert(entity);
                    self.cell_map.insert(x, y, c);
                }
            }
        }
    }

    pub fn remove(&mut self, collider: &Collider, entity: Entity) {
        let bounds = collider.bounds();

        let p1 = self.cell_coords(bounds.x, bounds.y);
        let p2 = self.cell_coords(bounds.right(), bounds.bottom());

        for x in (p1.x as i32)..=(p2.x as i32) {
            for y in (p1.y as i32)..=(p2.y as i32) {
                if let Some(c) = self.get_cell_mut(x, y) {
                    c.retain(|&x| x != entity);
                } else {
                    error!(
                        "removing collider {:?} from a cell that is is not present in",
                        collider
                    );
                }
            }
        }
    }

    pub fn get_nearby_pos(&self, pos: Vec2) -> HashSet<Entity> {
        let mut result = HashSet::new();

        let pos = self.cell_coords(pos.x, pos.y);
        for y in -1..2 {
            for x in -1..2 {
                if let Some(cell) = self.get_cell(pos.x as i32 + x, pos.y as i32 + y) {
                    result.extend(cell);
                }
            }
        }

        result
    }

    pub fn get_nearby_bounds(&self, bounds: FRect) -> HashSet<Entity> {
        let mut result = HashSet::new();

        let p1 = self.cell_coords(bounds.x, bounds.y);
        let p2 = self.cell_coords(bounds.right(), bounds.bottom());

        for x in (p1.x as i32)..=(p2.x as i32) {
            for y in (p1.y as i32)..=(p2.y as i32) {
                if let Some(cell) = self.get_cell(x, y) {
                    result.extend(cell);
                }
            }
        }

        result
    }

    pub fn clear(&mut self) {
        self.cell_map.clear();
    }

    pub fn cell_size(&self) -> i32 {
        self.cell_size
    }

    pub fn inverse_cell_size(&self) -> f32 {
        self.inverse_cell_size
    }

    pub fn get_all(&self) -> HashSet<Entity> {
        let mut result = HashSet::new();

        for (_hash, cell) in &self.cell_map.store {
            result.extend(cell);
        }

        result
    }

    fn cell_coords(&self, x: f32, y: f32) -> Vec2 {
        Vec2::new(
            floor_to_int(x * self.inverse_cell_size) as f32,
            floor_to_int(y * self.inverse_cell_size) as f32,
        )
    }

    fn get_cell(&self, x: i32, y: i32) -> Option<&ColliderSet> {
        if let Some(collider) = self.cell_map.get(x, y) {
            return Some(collider);
        }

        None
    }

    fn get_cell_mut(&mut self, x: i32, y: i32) -> Option<&mut ColliderSet> {
        if let Some(collider) = self.cell_map.get_mut(x, y) {
            return Some(collider);
        }

        None
    }
}

#[derive(Debug, Default)]
struct IntIntMap {
    pub store: HashMap<i64, ColliderSet>,
}

fn get_key(x: i32, y: i32) -> i64 {
    let shl = (x as i64).overflowing_shl(32);
    shl.0 | ((y as u32) as i64)
}

impl IntIntMap {
    pub fn insert(&mut self, x: i32, y: i32, colliders: ColliderSet) {
        self.store.insert(get_key(x, y), colliders);
    }

    pub fn get(&self, x: i32, y: i32) -> Option<&ColliderSet> {
        self.store.get(&get_key(x, y))
    }

    pub fn get_mut(&mut self, x: i32, y: i32) -> Option<&mut ColliderSet> {
        self.store.get_mut(&get_key(x, y))
    }

    pub fn clear(&mut self) {
        self.store.clear();
    }
}
