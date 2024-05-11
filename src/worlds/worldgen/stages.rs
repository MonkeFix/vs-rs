use crate::collisions::Rect;
use crate::worlds::world::{CellType, IntermediateWorld};
use crate::worlds::worldgen::{
    gen_rect, intersects_any, is_rect_oob, min_area_constraint, GridGraphPos,
};
use bevy::prelude::*;
use pathfinding::directed::astar;
use rand::{thread_rng, Rng};

use super::delaunay2d::Delaunay2D;
use super::prim::{self, PrimEdge};
use crate::worlds::bitmasking::{calc_bitmask, create_bitmap_from, BitMaskDirection};

pub trait WorldGenStage {
    fn get_description(&self) -> &'static str;
    fn execute(&mut self, world: &mut IntermediateWorld);
}

pub struct WorldGenStage1GenRects {}

impl WorldGenStage for WorldGenStage1GenRects {
    fn get_description(&self) -> &'static str {
        "Generating random rectangles"
    }

    fn execute(&mut self, world: &mut IntermediateWorld) {
        let mut iter = 0;

        while !min_area_constraint(world) {
            let mut min_w = world.settings.init_min_room_w;
            let mut min_h = world.settings.init_min_room_h;
            let mut max_w = world.settings.init_max_room_w;
            let mut max_h = world.settings.init_max_room_h;

            if iter > world.settings.next_step_iterations {
                min_w = world.settings.next_min_room_w;
                min_h = world.settings.next_min_room_h;
                max_w = world.settings.next_max_room_w;
                max_h = world.settings.next_max_room_h;
            }

            let random_rect = gen_rect(world, min_w, min_h, max_w, max_h);

            let offset = &world.settings.room_spacing;
            if !intersects_any(
                world,
                random_rect,
                Vec2::new(offset.x as f32, offset.y as f32),
            ) || is_rect_oob(world, &random_rect)
            {
                world.room_rects.push(random_rect);
            }

            if iter >= world.settings.max_room_iterations {
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
}

pub struct WorldGenStage2Triangulate {}

impl WorldGenStage for WorldGenStage2Triangulate {
    fn get_description(&self) -> &'static str {
        "Creating triangulation graph"
    }

    fn execute(&mut self, world: &mut IntermediateWorld) {
        world.triangulation_graph = Some(Delaunay2D::triangulate_constraint(&world.room_rects));
    }
}

pub struct WorldGenStage3MinSpanningTree {}

impl WorldGenStage for WorldGenStage3MinSpanningTree {
    fn get_description(&self) -> &'static str {
        "Finding a minimum spanning tree"
    }

    fn execute(&mut self, world: &mut IntermediateWorld) {
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

        info!("Extra edges added: {}", extra_edges);
    }
}

pub struct WorldGenStage4PlaceTIles {}

impl WorldGenStage for WorldGenStage4PlaceTIles {
    fn get_description(&self) -> &'static str {
        "Placing tiles"
    }

    fn execute(&mut self, world: &mut IntermediateWorld) {
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
    }
}

pub struct WorldGenStage5AStar {}

impl WorldGenStage for WorldGenStage5AStar {
    fn get_description(&self) -> &'static str {
        "Pathfinding connections between rooms"
    }

    fn execute(&mut self, world: &mut IntermediateWorld) {
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
                |p| p.successors(&world, &world.settings),
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
}

pub struct WorldGenStageCalcBitmapAndBitmask {}

impl WorldGenStage for WorldGenStageCalcBitmapAndBitmask {
    fn get_description(&self) -> &'static str {
        "Calculating bitmap and bitmask"
    }

    fn execute(&mut self, world: &mut IntermediateWorld) {
        let bitmap = create_bitmap_from(&world.grid, |p| *p == CellType::Wall);
        let bitmask = calc_bitmask(&bitmap, BitMaskDirection::Corners);

        world.bitmap = bitmap;
        world.bitmask = bitmask;
    }
}
