use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use common::{
    math::{approach, floor_to_int, is_flag_set, sign},
    FRect, Ray2D,
};

use super::{colliders::Collider, tests::*, RaycastHit, ALL_LAYERS};

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

    /// Extended version of [`get_nearby_bounds`] which checks collisions
    /// with the reference of the colliders.
    pub fn aabb_broadphase(
        &self,
        query: &Query<&Collider>,
        bounds: FRect,
        exclude_collider: Option<Entity>,
        layer_mask: Option<i32>,
    ) -> HashSet<Entity> {
        let mut tmp_hashset = HashSet::new();

        let layer_mask = layer_mask.unwrap_or(ALL_LAYERS);

        let p1 = self.cell_coords(bounds.x, bounds.y);
        let p2 = self.cell_coords(bounds.right(), bounds.bottom());

        for x in (p1.x as i32)..=(p2.x as i32) {
            for y in (p1.y as i32)..=(p2.y as i32) {
                let cell = self.get_cell(x, y);

                match cell {
                    Some(cell) => {
                        for entity in cell {
                            let collder = query.get(*entity).unwrap();

                            if exclude_collider.is_some_and(|excl| *entity == excl)
                                || !is_flag_set(layer_mask, collder.physics_layer)
                            {
                                continue;
                            }

                            if bounds.intersects(collder.bounds()) {
                                tmp_hashset.insert(*entity);
                            }
                        }
                    }
                    None => continue,
                }
            }
        }

        tmp_hashset
    }

    pub fn overlap_rectangle(
        &self,
        query: &Query<&Collider>,
        rect: FRect,
        exclude_collider: Option<Entity>,
        mut results: Option<&mut Vec<Entity>>,
        layer_mask: Option<i32>,
    ) -> i32 {
        let mut total = 0;
        let potentials = self.aabb_broadphase(query, rect, exclude_collider, layer_mask);
        for entity in potentials {
            let collider = query.get(entity).unwrap();
            match collider.shape.shape_type {
                super::shapes::ShapeType::Circle { radius } => {
                    if rect_to_circle(
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                        collider.center(),
                        radius,
                    ) {
                        if let Some(results) = results.as_mut() {
                            results.push(entity);
                        }
                        total += 1;
                    }
                }
                super::shapes::ShapeType::Box { .. } => {
                    if let Some(results) = results.as_mut() {
                        results.push(entity);
                    }
                    total += 1;
                }
                super::shapes::ShapeType::None => {}
            }
        }

        total
    }

    pub fn overlap_circle(
        &self,
        query: &Query<&Collider>,
        circle_center: Vec2,
        radius: f32,
        exclude_collider: Option<Entity>,
        mut results: Option<&mut Vec<Entity>>,
        layer_mask: Option<i32>,
    ) -> i32 {
        let bounds = FRect::new(
            circle_center.x - radius,
            circle_center.y - radius,
            radius * 2.0,
            radius * 2.0,
        );

        let mut total = 0;

        let mut test_circle = Collider::new(super::shapes::ShapeType::Circle { radius });
        test_circle.set_position(circle_center);

        let potentials = self.aabb_broadphase(query, bounds, exclude_collider, layer_mask);
        for entity in potentials {
            let collider = query.get(entity).unwrap();
            match collider.shape.shape_type {
                super::shapes::ShapeType::Circle { .. } => {
                    if collider.overlaps(&test_circle) {
                        if let Some(results) = results.as_mut() {
                            results.push(entity);
                        }
                        total += 1;
                    }
                }
                super::shapes::ShapeType::Box { .. } => {
                    if collider.overlaps(&test_circle) {
                        if let Some(results) = results.as_mut() {
                            results.push(entity);
                        }
                        total += 1;
                    }
                }
                super::shapes::ShapeType::None => {}
            }
        }

        total
    }

    pub fn linecast(
        &self,
        query: &Query<&Collider>,
        start: Vec2,
        end: Vec2,
        layer_mask: i32,
    ) -> (i32, Vec<RaycastHit>) {
        let mut res = Vec::new();
        let ray = Ray2D::new(start, end);
        let mut parser = RaycastResultParser::default();
        parser.start(ray, layer_mask);

        let mut cur_cell = self.cell_coords(start.x, start.y);
        let last_cell = self.cell_coords(end.x, end.y);

        let mut step_x = sign(ray.direction.x);
        let mut step_y = sign(ray.direction.y);

        if cur_cell.x == last_cell.x {
            step_x = 0;
        }
        if cur_cell.y == last_cell.y {
            step_y = 0;
        }

        let x_step = if (step_x as f32) < 0.0 {
            0.0
        } else {
            step_x as f32
        };
        let y_step = if (step_y as f32) < 0.0 {
            0.0
        } else {
            step_y as f32
        };
        let next_boundary_x = (cur_cell.x + x_step) * self.cell_size as f32;
        let next_boundary_y = (cur_cell.y + y_step) * self.cell_size as f32;

        let mut max_x = if ray.direction.x != 0.0 {
            (next_boundary_x - ray.start.x) / ray.direction.x
        } else {
            f32::MAX
        };
        let mut max_y = if ray.direction.y != 0.0 {
            (next_boundary_y - ray.start.y) / ray.direction.y
        } else {
            f32::MAX
        };

        let dt_x = if ray.direction.x != 0.0 {
            self.cell_size as f32 / (ray.direction.x * step_x as f32)
        } else {
            f32::MAX
        };
        let dt_y = if ray.direction.y != 0.0 {
            self.cell_size as f32 / (ray.direction.y * step_y as f32)
        } else {
            f32::MAX
        };

        if let Some(cell) = self.get_cell(cur_cell.x as i32, cur_cell.y as i32) {
            if parser.check_ray_intersection(query, cell, &mut res) {
                parser.reset();
                return (parser.hit_counter, res);
            }
        }

        let mut cell;

        while cur_cell.x != last_cell.x || cur_cell.y != last_cell.y {
            if max_x < max_y {
                cur_cell.x = (approach(cur_cell.x, last_cell.x, step_x.abs() as f32) as i32) as f32;
                max_x += dt_x;
            } else {
                cur_cell.y = (approach(cur_cell.y, last_cell.y, step_y.abs() as f32) as i32) as f32;
                max_y += dt_y;
            }

            cell = self.get_cell(cur_cell.x as i32, cur_cell.y as i32);
            if let Some(cell) = cell {
                if parser.check_ray_intersection(query, cell, &mut res) {
                    parser.reset();
                    return (parser.hit_counter, res);
                }
            }
        }

        parser.reset();
        (parser.hit_counter, res)
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

#[derive(Default)]
struct RaycastResultParser {
    pub hit_counter: i32,
    //hits: Option<&'a mut [RaycastHit<'a>]>,
    tmp_hit: Option<RaycastHit>,
    checked_colliders: Vec<Entity>,
    cell_hits: Vec<RaycastHit>,
    ray: Option<Ray2D>,
    layer_mask: i32,
}

impl RaycastResultParser {
    pub fn start(&mut self, ray: Ray2D, layer_mask: i32) {
        self.ray = Some(ray);
        self.layer_mask = layer_mask;
        self.hit_counter = 0;
    }

    pub fn check_ray_intersection(
        &mut self,
        query: &Query<&Collider>,
        cell: &HashSet<Entity>,
        raycasts: &mut Vec<RaycastHit>,
    ) -> bool {
        let ray = self.ray.unwrap();

        for potential in cell {
            if self.checked_colliders.contains(potential) {
                continue;
            }

            let potential_collider = query.get(*potential).unwrap();

            if potential_collider.is_trigger {
                continue;
            }

            if !is_flag_set(self.layer_mask, potential_collider.physics_layer) {
                continue;
            }

            let collider_bounds = &potential_collider.bounds();

            if let Some(fraction) = collider_bounds.ray_intersects(&ray) {
                if fraction <= 1.0 {
                    if let Some(mut tmp_hit) =
                        potential_collider.collides_with_line(ray.start, ray.end)
                    {
                        /* if potential.contains_point(ray.start) {
                            bevy::log::info!("contains point");
                            continue;
                        } */

                        tmp_hit.collider = Some(*potential);
                        self.cell_hits.push(tmp_hit);

                        self.tmp_hit = Some(tmp_hit);
                    }
                }
            }
        }

        if self.cell_hits.is_empty() {
            return false;
        }

        self.cell_hits.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for i in 0..self.cell_hits.len() {
            //self.hits.as_mut().unwrap()[self.hit_counter as usize] = self.cell_hits[i];
            raycasts.push(self.cell_hits[i]);

            self.hit_counter += 1;
            if self.hit_counter as usize == raycasts.len() {
                return true;
            }
        }

        false
    }

    pub fn reset(&mut self) {
        self.checked_colliders.clear();
        self.cell_hits.clear();
    }
}
