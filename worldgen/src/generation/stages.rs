use bevy::prelude::*;
use common::{
    bitmasking::{calc_bitmask, create_bitmap_from, BitMaskDirection},
    delaunay2d::Delaunay2D,
    prim::{min_spanning_tree, PrimEdge},
    FRect,
};
use pathfinding::directed::astar;
use rand::{thread_rng, Rng};
use vs_assets::rooms::RoomStore;

use crate::{
    generation::{gen_room, intersects_any, is_rect_oob, min_area_constraint, GridGraphPos},
    world::{CellType, IntermediateWorld},
};

use super::get_border_points;

pub trait WorldGenStage {
    fn get_description(&self) -> &'static str;
    fn execute(&mut self, world: &mut IntermediateWorld, room_store: &RoomStore);
}

pub struct WorldGenStage1GenRects {}

impl WorldGenStage for WorldGenStage1GenRects {
    fn get_description(&self) -> &'static str {
        "Generating random rectangles"
    }

    fn execute(&mut self, world: &mut IntermediateWorld, room_store: &RoomStore) {
        let mut iter = 0;

        while !min_area_constraint(world) {
            let room = gen_room(world, room_store);

            let offset = &world.settings.room_spacing;
            if !intersects_any(
                world,
                room.rect,
                Vec2::new(offset.x as f32, offset.y as f32),
            ) || is_rect_oob(world, &room.rect)
            {
                world.rooms.push(room);
            }

            if iter >= world.settings.max_room_iterations {
                warn!(
                    "Could not create required amount of rooms in {} iterations. Total rooms: {}",
                    iter,
                    world.rooms.len()
                );
                break;
            }

            iter += 1;
        }

        info!(
            "Stage 1 completed in {} iterations. Total rooms: {}",
            iter,
            world.rooms.len()
        );
    }
}

pub struct WorldGenStage2Triangulate {}

impl WorldGenStage for WorldGenStage2Triangulate {
    fn get_description(&self) -> &'static str {
        "Creating triangulation graph"
    }

    fn execute(&mut self, world: &mut IntermediateWorld, _room_store: &RoomStore) {
        let rooms = &world.rooms.iter().map(|x| x.rect).collect::<Vec<FRect>>();
        world.triangulation_graph = Some(Delaunay2D::triangulate_constraint(rooms));
    }
}

pub struct WorldGenStage3MinSpanningTree {}

impl WorldGenStage for WorldGenStage3MinSpanningTree {
    fn get_description(&self) -> &'static str {
        "Finding a minimum spanning tree"
    }

    fn execute(&mut self, world: &mut IntermediateWorld, _room_store: &RoomStore) {
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

        world.edges = min_spanning_tree(&prim_edges, start);

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

pub struct WorldGenStage4PlaceTiles {}

impl WorldGenStage for WorldGenStage4PlaceTiles {
    fn get_description(&self) -> &'static str {
        "Placing tiles"
    }

    fn execute(&mut self, world: &mut IntermediateWorld, _room_store: &RoomStore) {
        /* let mut rng = thread_rng();*/

        /* for y in 0..self.settings.world_height {
            for x in 0..self.settings.world_width {
                world.grid[y as usize][x as usize] = CellType::Room;
            }
        } */

        // create tiles

        for room in &world.rooms {
            let rect = room.rect;
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

    fn execute(&mut self, world: &mut IntermediateWorld, _room_store: &RoomStore) {
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
                |p| p.successors(world, &world.settings),
                |p| p.distance(&end_pos),
                |p| *p == end_pos,
            );

            if let Some(path) = path {
                found_paths += 1;

                for point in &path.0 {
                    world.grid[point.1 as usize][point.0 as usize] = CellType::Hallway;
                    world.grid[point.1 as usize + 1][point.0 as usize] = CellType::Hallway;
                    world.grid[point.1 as usize][point.0 as usize + 1] = CellType::Hallway;
                    world.grid[point.1 as usize + 1][point.0 as usize + 1] = CellType::Hallway;
                }
            }
        }

        info!("found {} paths", found_paths);
    }
}

pub struct WorldGenStageCreateWalls {}

impl WorldGenStage for WorldGenStageCreateWalls {
    fn get_description(&self) -> &'static str {
        "Creating walls around every room's borders"
    }

    fn execute(&mut self, world: &mut IntermediateWorld, _room_store: &RoomStore) {
        for room in &world.rooms {
            let rect = &room.rect;
            let border = get_border_points(rect);

            for point in &border {
                let tile = world.grid[point.1 as usize][point.0 as usize];
                if tile == CellType::Room || tile == CellType::None {
                    world.grid[point.1 as usize][point.0 as usize] = CellType::Wall;
                }
            }
        }
    }
}

pub struct WorldGenStageCalcBitmapAndBitmask {}

impl WorldGenStage for WorldGenStageCalcBitmapAndBitmask {
    fn get_description(&self) -> &'static str {
        "Calculating bitmap and bitmask"
    }

    fn execute(&mut self, world: &mut IntermediateWorld, _room_store: &RoomStore) {
        let bitmap = create_bitmap_from(&world.grid, |p| *p == CellType::Wall);
        let bitmask = calc_bitmask(&bitmap, BitMaskDirection::Corners);

        world.bitmap = bitmap;
        world.bitmask = bitmask;
    }
}
