use bevy::prelude::*;
use bevy_simple_tilemap::TileMap;

use crate::collisions::Rect;

use super::worldgen::{delaunay2d::Delaunay2D, prim::PrimEdge, CellType, GraphPos};

pub struct World {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<CellType>>,
    pub room_rects: Vec<Rect>,
    pub triangulation_graph: Option<Delaunay2D>,
    pub edges: Vec<PrimEdge>,
    pub edges_extra: Vec<PrimEdge>,
}
