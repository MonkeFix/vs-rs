#![allow(dead_code)]

use bevy::prelude::*;
use bevy_simple_tilemap::{Tile, TileMap};
use tiled::{Layer, TileLayer};

use crate::{
    assets::rooms::MapAsset,
    collisions::{colliders::ColliderData, shapes::ColliderShapeType, store::ColliderStore, Rect},
};

use super::worldgen::{
    delaunay2d::Delaunay2D, prim::PrimEdge, room::WorldRoom, settings::WorldGeneratorSettings,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CellType {
    None,
    Room,
    Hallway,
    Wall,
}

pub struct World {
    /// World's width in tiles
    pub width: usize,
    /// World's height in tiles
    pub height: usize,
    pub rooms: Vec<WorldRoom>,
    pub map_id: u32,
}

impl World {
    pub fn from_intermediate(iw: IntermediateWorld) -> World {
        iw.into_world()
    }

    pub fn fill_tilemap(&self, tilemap: &mut TileMap, assets: &Assets<MapAsset>, group_name: &str) {
        let mut i = 1;
        for room in &self.rooms {
            info!(" >> Filling room group {group_name} #{i}");

            let offset_x = room.rect.x as i32;
            let offset_y = room.rect.y as i32;

            let map_asset = assets.get(room.map_asset.id()).expect("unknown map asset");
            let mut layer_z = 1;
            for layer in map_asset.map.layers().filter(|m| m.name == group_name) {
                self.process_layer(tilemap, layer, offset_x, offset_y, &mut layer_z);
                //layer_z += 1;
            }

            i += 1;
        }
    }

    pub fn add_colliders(
        &self,
        assets: &Assets<MapAsset>,
        collision_layer_name: &str,
        collider_store: &mut ColliderStore,
        offset: Vec2,
    ) {
        for room in &self.rooms {
            let map_asset = assets.get(room.map_asset.id()).expect("unknown map asset");

            let tw = map_asset.map.tile_width as f32;
            let th = map_asset.map.tile_height as f32;

            let offset_x = room.rect.x * tw;
            let offset_y = room.rect.y * th;
            let height = room.rect.height * th;

            /* let collision_rects = map_asset.get_collision_rects(collision_layer_name);
            for rect in &collision_rects {
                collider_store.create_and_register(
                    ColliderShapeType::Box {
                        width: rect.width,
                        height: rect.height,
                    },
                    Some(Vec2::new(offset_x + rect.x, offset_y + (height - rect.y))),
                );
            } */

            let collision_layer = map_asset
                .find_layer_by_name(collision_layer_name)
                .unwrap_or_else(|| {
                    panic!("invalid collision layer name: {}", collision_layer_name)
                });

            let collision_layer = collision_layer
                .as_tile_layer()
                .expect("expected a tile layer");
            let w = collision_layer.width().expect("no width") as i32;
            let h = collision_layer.height().expect("no height") as i32;

            for y in 0..h {
                for x in 0..w {
                    let tile = collision_layer.get_tile(x, y);
                    if tile.is_some() {
                        let rect = Rect::new(
                            offset.x + offset_x + x as f32 * tw,
                            offset.y + offset_y + (height - y as f32 * th),
                            tw,
                            th,
                        );

                        collider_store.create_and_register(
                            ColliderData {
                                shape_type: ColliderShapeType::Box {
                                    width: rect.width,
                                    height: rect.height,
                                },
                                ..default()
                            },
                            Some(Vec2::new(rect.x, rect.y)),
                        );
                    }
                }
            }
        }
    }

    fn process_layer(
        &self,
        tilemap: &mut TileMap,
        layer: Layer,
        offset_x: i32,
        offset_y: i32,
        z: &mut i32,
    ) {
        info!("found layer \"{}\"", layer.name);
        if !layer.visible {
            info!("layer \"{}\" is not visible, skipping", layer.name);
            return;
        }
        let layer_type = layer.layer_type();
        match layer_type {
            tiled::LayerType::Tiles(tiles) => {
                self.fill_tile_layer(tilemap, tiles, offset_x, offset_y, *z);
                *z += 1;
            }
            tiled::LayerType::Group(group) => {
                for group_layer in group.layers() {
                    self.process_layer(tilemap, group_layer, offset_x, offset_y, z);
                }
            }
            _ => {}
        }
    }

    fn fill_tile_layer(
        &self,
        tilemap: &mut TileMap,
        tiles: TileLayer,
        offset_x: i32,
        offset_y: i32,
        z: i32,
    ) {
        let height = tiles.height().expect("no height wtf");

        info!(z);

        for y in 0..height {
            for x in 0..tiles.width().expect("no width wtf") {
                let tile = tiles.get_tile(x as i32, y as i32);

                if let Some(tile) = tile {
                    let tile = Tile {
                        sprite_index: tile.id(),
                        ..default()
                    };

                    tilemap.set_tile(
                        // offset_y + (height - y)
                        // is a fix for the Bevy's retarded 2D coordinate system
                        // where Y goes from bottom to up
                        IVec3::new(offset_x + x as i32, offset_y + (height - y) as i32, z),
                        Some(tile),
                    );
                }
            }
        }
    }
}

pub struct IntermediateWorld {
    pub width: usize,
    pub height: usize,
    pub settings: WorldGeneratorSettings,
    pub grid: Vec<Vec<CellType>>,
    pub rooms: Vec<WorldRoom>,
    pub triangulation_graph: Option<Delaunay2D>,
    pub edges: Vec<PrimEdge>,
    pub edges_extra: Vec<PrimEdge>,
    pub bitmap: Vec<Vec<bool>>,
    pub bitmask: Vec<Vec<u32>>,
}

impl IntermediateWorld {
    pub fn into_world(self) -> World {
        World {
            width: self.width,
            height: self.height,
            rooms: self.rooms,
            map_id: self.settings.map_id,
        }
    }
}
