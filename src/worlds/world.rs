use bevy::prelude::*;
use bevy_simple_tilemap::TileMap;

use super::worldgen::CellType;

pub struct World {
    pub width: usize,
    pub height: usize,
    pub grid: Vec<Vec<CellType>>,
}
