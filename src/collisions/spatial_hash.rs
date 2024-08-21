#![allow(dead_code)]

use bevy::{
    log,
    math::Vec2,
    prelude::{default, Circle},
    utils::{hashbrown::HashSet, HashMap},
};

use crate::math::{self, approach};

use super::{
    colliders::{Collider, ColliderData},
    rect_to_circle,
    shapes::ColliderShape,
    ColliderId, Ray2D, RaycastHit, Rect,
};

#[derive(Debug)]
pub struct SpatialHash {
    cell_size: i32,
    inverse_cell_size: f32,
    cell_map: IntIntMap,
    pub grid_bounds: Rect,
    tmp_hashset: HashSet<ColliderId>,
}

impl SpatialHash {
    pub fn new(cell_size: i32) -> Self {
        Self {
            cell_size,
            inverse_cell_size: 1.0 / cell_size as f32,
            cell_map: IntIntMap::default(),
            grid_bounds: Rect::new(0.0, 0.0, 0.0, 0.0),
            tmp_hashset: HashSet::new(),
        }
    }

    pub fn register(&mut self, collider: &Collider) {
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
                    c.push(collider.id);
                } else {
                    let c = vec![collider.id];
                    self.cell_map.insert(x, y, c);
                }
            }
        }
    }

    pub fn remove(&mut self, collider: &Collider) {
        let bounds = collider.bounds();

        let p1 = self.cell_coords(bounds.x, bounds.y);
        let p2 = self.cell_coords(bounds.right(), bounds.bottom());

        for x in (p1.x as i32)..=(p2.x as i32) {
            for y in (p1.y as i32)..=(p2.y as i32) {
                if let Some(c) = self.get_cell_mut(x, y) {
                    c.retain(|&x| x != collider.id);
                } else {
                    log::error!(
                        "removing collider {:?} from a cell that is is not present in",
                        collider.id
                    );
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.cell_map.clear();
    }

    pub fn aabb_broadphase<'a, F>(
        &'a self,
        bounds: &Rect,
        exclude_collider: Option<ColliderId>,
        layer_mask: i32,
        collider_finder: F,
    ) -> HashSet<ColliderId>
    where
        F: Fn(&ColliderId) -> Option<&'a Collider>,
    {
        let mut tmp_hashset = HashSet::new();

        let p1 = self.cell_coords(bounds.x, bounds.y);
        let p2 = self.cell_coords(bounds.right(), bounds.bottom());

        for x in (p1.x as i32)..=(p2.x as i32) {
            for y in (p1.y as i32)..=(p2.y as i32) {
                let cell = self.get_cell(x, y);

                match cell {
                    Some(cell) => {
                        for collider_id in cell {
                            let collder = collider_finder(collider_id).unwrap();

                            if exclude_collider.is_some_and(|excl| *collider_id == excl)
                                || !is_flag_set(layer_mask, collder.data.physics_layer)
                            {
                                continue;
                            }

                            if bounds.intersects(collder.bounds()) {
                                tmp_hashset.insert(*collider_id);
                            }
                        }
                    }
                    None => continue,
                }
            }
        }

        tmp_hashset
    }

    pub fn linecast<'a, F>(
        &self,
        start: Vec2,
        end: Vec2,
        //hits: &'a mut [RaycastHit<'a>],
        layer_mask: i32,
        find_collider: F,
    ) -> (i32, Vec<RaycastHit>)
    where
        F: Fn(&ColliderId) -> Option<&'a Collider>,
    {
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
            if parser.check_ray_intersection(cell, &find_collider, &mut res) {
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
                if parser.check_ray_intersection(cell, &find_collider, &mut res) {
                    parser.reset();
                    return (parser.hit_counter, res);
                }
            }
        }

        parser.reset();
        (parser.hit_counter, res)
    }

    pub fn overlap_rectangle<'a, F>(
        &'a self,
        rect: &Rect,
        exclude_collider: Option<ColliderId>,
        results: &mut Vec<ColliderId>,
        layer_mask: i32,
        collider_finder: F,
    ) -> i32
    where
        F: Fn(&ColliderId) -> Option<&'a Collider>,
    {
        let potentials = self.aabb_broadphase(rect, exclude_collider, layer_mask, &collider_finder);
        for collider_id in potentials {
            let collider = collider_finder(&collider_id).unwrap();
            match collider.shape.shape_type {
                super::shapes::ColliderShapeType::Circle { radius } => {
                    if rect_to_circle(
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                        collider.center(),
                        radius,
                    ) {
                        results.push(collider_id);
                    }
                }
                super::shapes::ColliderShapeType::Box { .. } => {
                    results.push(collider_id);
                }
                super::shapes::ColliderShapeType::None => {}
            }
        }

        results.len() as i32
    }

    pub fn overlap_circle<'a, F>(
        &'a self,
        circle_center: Vec2,
        radius: f32,
        exclude_collider: Option<ColliderId>,
        results: &mut Vec<ColliderId>,
        layer_mask: i32,
        collider_finder: F,
    ) -> i32
    where
        F: Fn(&ColliderId) -> Option<&'a Collider>,
    {
        let bounds = Rect::new(
            circle_center.x - radius,
            circle_center.y - radius,
            radius * 2.0,
            radius * 2.0,
        );

        let mut test_circle = Collider::new(
            ColliderData {
                shape_type: super::shapes::ColliderShapeType::Circle { radius },
                ..default()
            },
            None,
        );

        test_circle.shape.position = circle_center;
        test_circle.shape.center = circle_center;

        let potentials =
            self.aabb_broadphase(&bounds, exclude_collider, layer_mask, &collider_finder);
        for collider_id in potentials {
            let collider = collider_finder(&collider_id).unwrap();
            match collider.shape.shape_type {
                super::shapes::ColliderShapeType::Circle { .. } => {
                    if collider.overlaps(&test_circle) {
                        results.push(collider_id);
                    }
                }
                super::shapes::ColliderShapeType::Box { .. } => {
                    if collider.overlaps(&test_circle) {
                        results.push(collider_id);
                    }
                }
                super::shapes::ColliderShapeType::None => {}
            }
        }

        results.len() as i32
    }

    fn cell_coords(&self, x: f32, y: f32) -> Vec2 {
        Vec2::new(
            math::floor_to_int(x * self.inverse_cell_size) as f32,
            math::floor_to_int(y * self.inverse_cell_size) as f32,
        )
    }

    fn get_cell(&self, x: i32, y: i32) -> Option<&Vec<ColliderId>> {
        if let Some(collider) = self.cell_map.get(x, y) {
            return Some(collider);
        }

        None
    }

    fn get_cell_mut(&mut self, x: i32, y: i32) -> Option<&mut Vec<ColliderId>> {
        if let Some(collider) = self.cell_map.get_mut(x, y) {
            return Some(collider);
        }

        None
    }
}

type ColliderList = Vec<ColliderId>;
#[derive(Debug, Default)]
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

fn is_flag_set(bits: i32, flag: i32) -> bool {
    (bits & flag) != 0
}

fn sign(val: f32) -> i32 {
    if val == 0.0 {
        return 0;
    }
    if val < 0.0 {
        return -1;
    }
    1
}

#[derive(Default)]
struct RaycastResultParser {
    pub hit_counter: i32,
    //hits: Option<&'a mut [RaycastHit<'a>]>,
    tmp_hit: Option<RaycastHit>,
    checked_colliders: Vec<ColliderId>,
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

    pub fn check_ray_intersection<'a, F>(
        &mut self,
        cell: &[ColliderId],
        find_collider: F,
        raycasts: &mut Vec<RaycastHit>,
    ) -> bool
    where
        F: Fn(&ColliderId) -> Option<&'a Collider>,
    {
        let ray = self.ray.unwrap();

        for potential in cell {
            let potential = find_collider(potential).unwrap();

            if self.checked_colliders.contains(&potential.id) {
                continue;
            }

            if potential.data.is_trigger {
                continue;
            }

            if !is_flag_set(self.layer_mask, potential.data.physics_layer) {
                continue;
            }

            let collider_bounds = &potential.bounds();

            if let Some(fraction) = collider_bounds.ray_intersects(&ray) {
                if fraction <= 1.0 {
                    if let Some(mut tmp_hit) = potential.collides_with_line(ray.start, ray.end) {
                        /* if potential.contains_point(ray.start) {
                            bevy::log::info!("contains point");
                            continue;
                        } */

                        tmp_hit.collider = Some(potential.id);
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
