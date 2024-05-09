use bevy::prelude::*;
use bevy_simple_tilemap::{Tile, TileFlags, TileMap};
use rand::{thread_rng, Rng};

use crate::assets::GameAssets;

use self::settings::WorldGeneratorSettings;

use super::world::World;

pub mod settings;

pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub fn gen_point(min_w: i32, min_h: i32, max_w: i32, max_h: i32) -> Point {
    let mut rng = thread_rng();

    Point {
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

        let mut world = World {
            grid: vec![vec![CellType::None; w]; h],
            width: w,
            height: h,
        };

        self.gen_layout(&mut world);
        //self.place_tiles(&mut world);

        world
    }

    fn gen_layout(&self, world: &mut World) {
        let mut rng = thread_rng();

        for y in 0..self.settings.world_height {
            for x in 0..self.settings.world_width {
                if rng.gen_ratio(1, 3) {
                    world.grid[y as usize][x as usize] = CellType::Room;
                } else if rng.gen_ratio(1, 3) {
                    world.grid[y as usize][x as usize] = CellType::Hallway;
                } else {
                    world.grid[y as usize][x as usize] = CellType::Wall;
                }
            }
        }
    }
}
