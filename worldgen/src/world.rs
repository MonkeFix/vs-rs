use bevy::prelude::*;
use bevy_simple_tilemap::{Tile, TileMap};
use colliders::Collider;
use common::{delaunay2d::Delaunay2D, prim::PrimEdge, FRect};
use physics::prelude::*;
use tiled::{Layer, TileLayer};
use vs_assets::rooms::MapAsset;

use crate::generation::{room::WorldRoom, settings::WorldGeneratorSettings};

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
        for room in &self.rooms {
            let offset_x = room.rect.x as i32;
            let offset_y = room.rect.y as i32;

            let map_asset = assets.get(room.map_asset.id()).expect("unknown map asset");
            let mut layer_z = 1;
            for layer in map_asset.map.layers().filter(|m| m.name == group_name) {
                self.process_layer(tilemap, layer, offset_x, offset_y, &mut layer_z);
            }
        }
    }

    pub fn add_colliders(
        &self,
        assets: &Assets<MapAsset>,
        collision_layer_name: &str,
        collision_fine_layer_name: &str,
        //collider_store: &mut ColliderStore,
        //spatial_hash: &SpatialHash,
        commands: &mut Commands,
        offset: Vec2,
    ) {
        for room in &self.rooms {
            // Add tilemap colliders
            let map_asset = assets.get(room.map_asset.id()).expect("unknown map asset");

            let tw = map_asset.map.tile_width as f32;
            let th = map_asset.map.tile_height as f32;

            let offset_x = room.rect.x * tw;
            let offset_y = room.rect.y * th;
            let room_height = room.rect.height * th;

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
                        let rect = FRect::new(
                            offset.x + offset_x + x as f32 * tw,
                            offset.y + offset_y + (room_height - y as f32 * th),
                            tw,
                            th,
                        );

                        /* collider_store.create_and_register(
                            ColliderData {
                                shape_type: ColliderShapeType::Box {
                                    width: rect.width,
                                    height: rect.height,
                                },
                                ..default()
                            },
                            Some(Vec2::new(rect.x, rect.y)),
                        ); */
                        commands.spawn((
                            SpatialBundle {
                                transform: Transform::from_xyz(rect.x, rect.y, 0.0),
                                ..default()
                            },
                            Collider::new(shapes::ShapeType::Box {
                                width: rect.width,
                                height: rect.height,
                            }),
                            RigidBodyStatic,
                        ));
                    }
                }
            }

            // Add object colliders
            let collision_layer = map_asset
                .find_layer_by_name(collision_fine_layer_name)
                .unwrap_or_else(|| {
                    panic!(
                        "invalid collision fine layer name: {}",
                        collision_fine_layer_name
                    )
                });

            let collision_layer = collision_layer
                .as_object_layer()
                .expect("expected an object layer");
            for obj in collision_layer.objects() {
                match obj.shape {
                    tiled::ObjectShape::Rect { width, height } => {
                        let pos = Vec2::new(
                            offset.x + offset_x + obj.x + width / 2.0 - 16.0,
                            offset.y + offset_y + (room_height - (obj.y + height / 2.0)) + 16.0,
                        );

                        /* collider_store.create_and_register(
                            ColliderData {
                                shape_type: ColliderShapeType::Box { width, height },
                                ..default()
                            },
                            Some(pos),
                        ); */
                        commands.spawn((
                            SpatialBundle {
                                transform: Transform::from_xyz(pos.x, pos.y, 0.0),
                                ..default()
                            },
                            Collider::new(shapes::ShapeType::Box { width, height }),
                            RigidBodyStatic,
                        ));
                    }
                    tiled::ObjectShape::Ellipse { .. } => {
                        unimplemented!("ellipse is not implemented")
                    }
                    tiled::ObjectShape::Polyline { .. } => {
                        unimplemented!("polyline is not implemented")
                    }
                    tiled::ObjectShape::Polygon { .. } => {
                        unimplemented!("polygon is not implemented")
                    }
                    tiled::ObjectShape::Point(_, _) => unimplemented!("point is not implemented"),
                    tiled::ObjectShape::Text { .. } => unimplemented!("text is not implemented"),
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
        if !layer.visible {
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
