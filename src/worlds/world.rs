use bevy::{prelude::*, utils::HashMap};
use bevy_simple_tilemap::TileMap;

use crate::collisions::Rect;

use super::{
    bitmasking::{calc_bitmask, create_bitmap_from},
    worldgen::{delaunay2d::Delaunay2D, prim::PrimEdge, settings::WorldGeneratorSettings},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellType {
    None,
    Room,
    Hallway,
    Wall,
}

pub struct WorldLayer {
    pub id: usize,
    /// Layer's width in tiles
    pub width: usize,
    /// Layer's height in tiles
    pub height: usize,
    /// Layer's offset in pixels
    pub offset: Vec2,
    /// 2D matrix which describes layer's grid. Specific tile can be accessed via `data[y][x]`
    pub data: Vec<Vec<CellType>>,
}

pub struct World {
    /// World's width in tiles
    pub width: usize,
    /// World's height in tiles
    pub height: usize,
    /// All world's layers in a hash map, key is the layer id
    pub layers: HashMap<usize, WorldLayer>,
}

impl World {
    pub fn from_intermediate(iw: IntermediateWorld) -> World {
        iw.to_world()
    }

    pub fn fill_tilemap(&self, tilemap: &mut TileMap) {}
}

pub struct IntermediateWorld {
    pub width: usize,
    pub height: usize,
    pub settings: WorldGeneratorSettings,
    pub grid: Vec<Vec<CellType>>,
    pub room_rects: Vec<Rect>,
    pub triangulation_graph: Option<Delaunay2D>,
    pub edges: Vec<PrimEdge>,
    pub edges_extra: Vec<PrimEdge>,
    pub bitmap: Vec<Vec<bool>>,
    pub bitmask: Vec<Vec<u32>>
}

impl IntermediateWorld {
    pub fn to_world(self) -> World {

        let mut data = HashMap::new();
        data.insert(
            0,
            WorldLayer {
                id: 0,
                offset: Vec2::new(0.0, 0.0),
                width: self.width,
                height: self.height,
                data: self.grid,
            },
        );

        World {
            width: self.width,
            height: self.height,
            layers: data,
        }
    }
}
