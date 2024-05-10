use std::time::Instant;

use bevy::prelude::*;
use pathfinding::directed::astar;
use rand::{thread_rng, Rng};

use crate::collisions::Rect;

use self::{delaunay2d::Delaunay2D, prim::PrimEdge, settings::WorldGeneratorSettings};

use super::world::World;

pub mod delaunay2d;
pub mod prim;
pub mod settings;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GraphPos(pub i32, pub i32);

impl GraphPos {
    fn distance(&self, other: &GraphPos) -> u32 {
        (self.0.abs_diff(other.0) + self.1.abs_diff(other.1)) as u32
    }

    fn successors(&self, nodes: &[PrimEdge]) -> Vec<(GraphPos, u32)> {
        let mut res = vec![];
        for node in nodes {
            let ux = node.u.x as i32;
            let uy = node.u.y as i32;
            let vx = node.v.x as i32;
            let vy = node.v.y as i32;
            if vx == self.0 && vy == self.1 {
                res.push((GraphPos(ux, uy), 1));
            } else if ux == self.0 && uy == self.1 {
                res.push((GraphPos(vx, vy), 1));
            }
        }

        res
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GridGraphPos(pub i32, pub i32);

impl GridGraphPos {
    fn distance(&self, other: &GridGraphPos) -> u32 {
        (self.0.abs_diff(other.0) + self.1.abs_diff(other.1)) as u32
    }

    fn successors(
        &self,
        world: &World,
        settings: &WorldGeneratorSettings,
    ) -> Vec<(GridGraphPos, u32)> {
        static DIRS: &[GridGraphPos] = &[
            GridGraphPos(1, 0),
            GridGraphPos(0, -1),
            GridGraphPos(-1, 0),
            GridGraphPos(0, 1),
        ];

        let mut res = vec![];
        for dir in DIRS {
            let next = GridGraphPos(self.0 + dir.0, self.1 + dir.1);
            if self.is_in_bounds(&next, world.width, world.height) && self.is_passable(&next) {
                res.push((next, self.get_cost(world, settings, next)));
            }
        }
        res
    }

    fn is_in_bounds(&self, node: &GridGraphPos, width: usize, height: usize) -> bool {
        node.0 >= 0 && node.0 < width as i32 && node.1 >= 0 && node.1 < height as i32
    }

    fn is_passable(&self, _node: &GridGraphPos) -> bool {
        true
    }

    fn get_cost(&self, world: &World, setting: &WorldGeneratorSettings, pos: GridGraphPos) -> u32 {
        if !self.is_in_bounds(&pos, world.width, world.height) {
            return 999999;
        }

        match world.grid[pos.1 as usize][pos.0 as usize] {
            CellType::None => setting.cost_empty_space,
            CellType::Room => setting.cost_room,
            CellType::Hallway => setting.cost_hallway,
            CellType::Wall => setting.cost_wall,
        }
    }
}

struct WorldPoint {
    pub x: u32,
    pub y: u32,
}

fn gen_point(min_w: u32, min_h: u32, max_w: u32, max_h: u32) -> WorldPoint {
    let mut rng = thread_rng();

    WorldPoint {
        x: rng.gen_range(min_w..max_w),
        y: rng.gen_range(min_h..max_h),
    }
}

#[derive(Debug, Clone, Copy)]
pub enum CellType {
    None,
    Room,
    Hallway,
    Wall,
}

pub struct WorldGenerator {
    pub settings: WorldGeneratorSettings,
}

impl WorldGenerator {
    pub fn new(settings: WorldGeneratorSettings) -> Self {
        Self { settings }
    }

    pub fn new_with_default(width: u32, height: u32) -> Self {
        let mut settings = WorldGeneratorSettings::default();
        settings.world_width = width;
        settings.world_height = height;

        WorldGenerator::new(settings)
    }

    pub fn generate(&mut self) -> World {
        let w = self.settings.world_width as usize;
        let h = self.settings.world_height as usize;

        info!(
            "Started generating a new world, width: {}, height: {}",
            w, h
        );

        let mut world = World {
            grid: vec![vec![CellType::None; w]; h],
            room_rects: vec![],
            width: w,
            height: h,
            triangulation_graph: None,
            edges: vec![],
            edges_extra: vec![],
        };

        let now = Instant::now();

        self.stage_1(&mut world);
        self.stage_2(&mut world);
        self.stage_3(&mut world);
        self.stage_4(&mut world);
        self.stage_5(&mut world);

        let elapsed = now.elapsed();
        info!("Finished generating. Elapsed: {:.2?}", elapsed);

        world
    }

    /// Generate room rectangles randomly without intersections
    fn stage_1(&self, world: &mut World) {
        let mut iter = 0;

        while !self.min_area_constraint(world) {
            let mut min_w = self.settings.init_min_room_w;
            let mut min_h = self.settings.init_min_room_h;
            let mut max_w = self.settings.init_max_room_w;
            let mut max_h = self.settings.init_max_room_h;

            if iter > self.settings.next_step_iterations {
                min_w = self.settings.next_min_room_w;
                min_h = self.settings.next_min_room_h;
                max_w = self.settings.next_max_room_w;
                max_h = self.settings.next_max_room_h;
            }

            let random_rect = self.gen_rect(min_w, min_h, max_w, max_h);

            let offset = &self.settings.room_spacing;
            if !self.intersects_any(
                world,
                random_rect,
                Vec2::new(offset.x as f32, offset.y as f32),
            ) || self.is_rect_oob(&random_rect)
            {
                world.room_rects.push(random_rect);
            }

            if iter >= self.settings.max_room_iterations {
                warn!(
                    "Could not create required amount of rooms in {} iterations. Total rooms: {}",
                    iter,
                    world.room_rects.len()
                );
                break;
            }

            iter += 1;
        }

        info!(
            "Stage 1 completed in {} iterations. Total rooms: {}",
            iter,
            world.room_rects.len()
        );
    }

    fn stage_2(&self, world: &mut World) {
        world.triangulation_graph = Some(Delaunay2D::triangulate_constraint(&world.room_rects));
        info!("Stage 2 completed");
    }

    fn stage_3(&self, world: &mut World) {
        // find a minimum spanning tree
        let graph = world
            .triangulation_graph
            .as_ref()
            .expect("No triangulation graph was generated");

        let start = graph.edges.first().expect("Triangulation graph is empty").u;
        let prim_edges = graph
            .edges
            .iter()
            .map(|x| PrimEdge::new(x.u, x.v))
            .collect::<Vec<PrimEdge>>();

        world.edges = prim::min_spanning_tree(&prim_edges, start);

        let mut rng = thread_rng();
        let mut extra_edges = 0;
        // add some random edges
        for edge in graph.edges.iter() {
            if rng.gen_ratio(1, 200) {
                world.edges_extra.push(PrimEdge::new(edge.u, edge.v));
                extra_edges += 1;
            }
        }

        info!("Stage 3 completed. Extra edges: {}", extra_edges);
    }

    fn stage_4(&self, world: &mut World) {
        /* let mut rng = thread_rng();*/

        /* for y in 0..self.settings.world_height {
            for x in 0..self.settings.world_width {
                world.grid[y as usize][x as usize] = CellType::Room;
            }
        } */

        // create tiles

        for rect in &world.room_rects {
            let left = rect.x as usize;
            let right = (rect.x + rect.width) as usize;
            let top = rect.y as usize;
            let bottom = (rect.y + rect.height) as usize;

            for y in top..=bottom {
                for x in left..=right {
                    world.grid[y][x] = CellType::Room;
                }
            }
        }

        info!("Stage 4 completed");
    }

    fn stage_5(&self, world: &mut World) {
        // Find paths between connected rooms using A* algorithm
        let mut nodes = vec![];
        for edge in &world.edges {
            nodes.push(*edge);
        }
        /* for edge in &world.edges_extra {
            nodes.push(*edge);
        } */

        let mut found_paths = 0;

        for node in &nodes {
            let start_pos = GridGraphPos(node.u.x as i32, node.u.y as i32);
            let end_pos = GridGraphPos(node.v.x as i32, node.v.y as i32);

            let path = astar::astar(
                &start_pos,
                |p| p.successors(&world, &self.settings),
                |p| p.distance(&end_pos),
                |p| *p == end_pos,
            );

            if let Some(path) = path {
                found_paths += 1;

                for point in &path.0 {
                    world.grid[point.1 as usize][point.0 as usize] = CellType::Hallway;
                }
            }
        }

        info!("found {} paths", found_paths);
    }

    fn min_area_constraint(&self, world: &World) -> bool {
        if world.room_rects.len() >= self.settings.max_rooms as usize {
            return true;
        }

        let mut cur = 0;

        for rect in &world.room_rects {
            cur += (rect.width * rect.height) as u32;
        }

        cur >= self.settings.min_used_area
    }

    fn intersects_any(&self, world: &World, mut rect: Rect, offset: Vec2) -> bool {
        rect.inflate(offset.x, offset.y);

        for rect2 in &world.room_rects {
            if rect2.intersects(rect) {
                return true;
            }
        }

        false
    }

    fn gen_rect(&self, min_w: u32, min_h: u32, max_w: u32, max_h: u32) -> Rect {
        let size = gen_point(min_w, min_h, max_w, max_h);

        let pos = gen_point(
            1,
            1,
            self.settings.world_width - size.x - 1,
            self.settings.world_height - size.y - 1,
        );

        Rect {
            x: pos.x as f32,
            y: pos.y as f32,
            width: size.x as f32,
            height: size.y as f32,
        }
    }

    fn is_rect_oob(&self, rect: &Rect) -> bool {
        rect.x < 0.
            || rect.x >= self.settings.world_width as f32 - 1.
            || rect.y < 0.
            || rect.y >= self.settings.world_height as f32 - 1.
    }
}
