use std::time::Instant;

use bevy::{prelude::*, utils::info};
use pathfinding::directed::astar;
use rand::{thread_rng, Rng};
use room::WorldRoom;

use crate::{
    assets::rooms::RoomStore, collisions::Rect, math::choose_random,
    worlds::world::IntermediateWorld,
};

use self::{
    prim::PrimEdge,
    settings::WorldGeneratorSettings,
    stages::{
        WorldGenStage, WorldGenStage1GenRects, WorldGenStage2Triangulate,
        WorldGenStage3MinSpanningTree, WorldGenStage4PlaceTiles, WorldGenStage5AStar,
        WorldGenStageCalcBitmapAndBitmask, WorldGenStageCreateWalls,
    },
};

use super::world::{CellType, World};

pub mod delaunay2d;
pub mod prim;
pub mod room;
pub mod settings;
mod stages;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct GraphPos(pub i32, pub i32);

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
struct GridGraphPos(pub i32, pub i32);

impl GridGraphPos {
    fn distance(&self, other: &GridGraphPos) -> u32 {
        (self.0.abs_diff(other.0) + self.1.abs_diff(other.1)) as u32
    }

    fn successors(
        &self,
        world: &IntermediateWorld,
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

    fn get_cost(
        &self,
        world: &IntermediateWorld,
        setting: &WorldGeneratorSettings,
        pos: GridGraphPos,
    ) -> u32 {
        if !self.is_in_bounds(&pos, world.width, world.height) {
            return u32::MAX;
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

fn gen_room(world: &IntermediateWorld, room_store: &RoomStore) -> WorldRoom {
    let all_rooms = room_store.get_rooms(world.settings.room_id);
    let room = choose_random(all_rooms);
    let size = UVec2::new(room.map.width, room.map.height);

    // genering a point that will not touch the world's border
    let pos = gen_point(
        1,
        1,
        world.settings.world_width - size.x - 1,
        world.settings.world_height - size.y - 1,
    );

    let rect = Rect {
        x: pos.x as f32,
        y: pos.y as f32,
        width: size.x as f32,
        height: size.y as f32,
    };

    WorldRoom {
        map_asset: room.clone(),
        rect,
    }
}

fn is_rect_oob(world: &IntermediateWorld, rect: &Rect) -> bool {
    rect.x < 0.
        || rect.x >= world.settings.world_width as f32 - 1.
        || rect.y < 0.
        || rect.y >= world.settings.world_height as f32 - 1.
}

fn min_area_constraint(world: &IntermediateWorld) -> bool {
    if world.rooms.len() >= world.settings.max_rooms as usize {
        return true;
    }

    let mut cur = 0;

    for room in &world.rooms {
        cur += (room.rect.width * room.rect.height) as u32;
    }

    cur >= world.settings.min_used_area
}

fn intersects_any(world: &IntermediateWorld, mut rect: Rect, offset: Vec2) -> bool {
    rect.inflate(offset.x, offset.y);

    for room in &world.rooms {
        if room.rect.intersects(rect) {
            return true;
        }
    }

    false
}

fn get_border_points(rect: &Rect) -> Vec<(i32, i32)> {
    let mut res = vec![];

    // --------
    // .      .
    // .      .
    // --------
    for x in rect.left() as i32..=rect.right() as i32 {
        // top edge
        res.push((x, rect.top() as i32));
        // bottom edge
        res.push((x, rect.bottom() as i32));
    }

    // ........
    // |      |
    // |      |
    // ........
    for y in rect.top() as i32 + 1..=rect.bottom() as i32 - 1 {
        // left edge
        res.push((rect.left() as i32, y));
        // right edge
        res.push((rect.right() as i32, y));
    }

    res
}

pub struct WorldGenerator {
    // TODO: Use double linked list to allow manipulation of stage ordering
    pub stages: Vec<Box<dyn WorldGenStage>>,
}

impl Default for WorldGenerator {
    fn default() -> Self {
        let stages: Vec<Box<dyn WorldGenStage>> = vec![
            Box::new(WorldGenStage1GenRects {}),
            Box::new(WorldGenStage2Triangulate {}),
            Box::new(WorldGenStage3MinSpanningTree {}),
            Box::new(WorldGenStage4PlaceTiles {}),
            Box::new(WorldGenStageCreateWalls {}),
            Box::new(WorldGenStage5AStar {}),
            Box::new(WorldGenStageCalcBitmapAndBitmask {}),
        ];

        Self { stages }
    }
}

impl WorldGenerator {
    pub fn generate(&mut self, settings: WorldGeneratorSettings, room_store: &RoomStore) -> World {
        let w = settings.world_width as usize;
        let h = settings.world_height as usize;

        info!(
            "Started generating a new world, width: {}, height: {}",
            w, h
        );

        let mut world = IntermediateWorld {
            settings,
            grid: vec![vec![CellType::None; w]; h],
            rooms: vec![],
            width: w,
            height: h,
            triangulation_graph: None,
            edges: vec![],
            edges_extra: vec![],
            bitmap: vec![],
            bitmask: vec![],
        };

        let now = Instant::now();

        for stage in self.stages.iter_mut() {
            let desc = stage.get_description();
            info!("{}", desc);
            stage.execute(&mut world, room_store);
        }

        let elapsed = now.elapsed();
        info!("Finished generating. Elapsed: {:.2?}", elapsed);

        world.to_world()
    }
}
