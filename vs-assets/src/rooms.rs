use bevy::{
    asset::{io::Reader, Asset, AssetLoader, LoadContext},
    prelude::*,
    reflect::TypePath,
    utils::HashMap,
};
use common::FRect;
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct MapAsset {
    pub name: String,
    pub map_id: u32,
    pub map: tiled::Map,
}

impl MapAsset {
    pub fn get_collision_rects(&self, collision_layer: &str) -> Vec<FRect> {
        let mut checked_indexes = vec![false; (self.map.width * self.map.height) as usize];
        let mut rectangles = vec![];
        let mut start_col: i32 = -1;
        let mut index: i32;

        let layer = self
            .find_layer_by_name(collision_layer)
            .expect("No layer found");

        for y in 0..self.map.height {
            for x in 0..self.map.width {
                index = (y * self.map.width + x) as i32;
                let tile = match layer.layer_type() {
                    tiled::LayerType::Tiles(tile) => tile.get_tile(x as i32, y as i32),
                    _ => None,
                };

                if tile.is_some() && !checked_indexes[index as usize] {
                    if start_col < 0 {
                        start_col = x as i32;
                    }
                    checked_indexes[index as usize] = true;
                } else if (tile.is_none() || checked_indexes[index as usize]) && start_col >= 0 {
                    rectangles.push(self.find_bounds_rect(
                        collision_layer,
                        start_col,
                        x as i32,
                        y as i32,
                        &mut checked_indexes,
                    ));
                    start_col = -1;
                }
            }

            if start_col >= 0 {
                rectangles.push(self.find_bounds_rect(
                    collision_layer,
                    start_col,
                    self.map.width as i32,
                    y as i32,
                    &mut checked_indexes,
                ));
                start_col = -1;
            }
        }

        rectangles
    }

    pub fn find_bounds_rect(
        &self,
        collision_layer: &str,
        start_x: i32,
        end_x: i32,
        start_y: i32,
        checked_indexes: &mut [bool],
    ) -> FRect {
        let mut index;
        let layer = self
            .find_layer_by_name(collision_layer)
            .expect("No layer found");

        for y in (start_y + 1)..self.map.height as i32 {
            for x in start_x..end_x {
                index = y * self.map.width as i32 + x;
                let tile = match layer.layer_type() {
                    tiled::LayerType::Tiles(tile) => tile.get_tile(x, y),
                    // TODO: Check tiled::LayerType::Group
                    _ => None,
                };

                if tile.is_none() || checked_indexes[index as usize] {
                    for _x in start_x..x {
                        index = y * self.map.width as i32 + x;
                        checked_indexes[index as usize] = false;
                    }

                    return FRect {
                        x: (start_x * self.tile_width() as i32) as f32,
                        y: (start_y * self.tile_height() as i32) as f32,
                        width: ((end_x - start_x) * self.tile_width() as i32) as f32,
                        height: ((y - start_y) * self.tile_height() as i32) as f32,
                    };
                }

                checked_indexes[index as usize] = true;
            }
        }

        FRect {
            x: (start_x * self.tile_width() as i32) as f32,
            y: (start_y * self.tile_height() as i32) as f32,
            width: ((end_x - start_x) * self.tile_width() as i32) as f32,
            height: ((self.map.height as i32 - start_y) * self.tile_height() as i32) as f32,
        }
    }

    pub fn tile_width(&self) -> u32 {
        self.map.tile_width
    }

    pub fn tile_height(&self) -> u32 {
        self.map.tile_height
    }

    pub fn full_width(&self) -> u32 {
        self.map.width * self.tile_width()
    }

    pub fn full_height(&self) -> u32 {
        self.map.height * self.tile_height()
    }

    pub fn find_layer_by_name(&self, layer_name: &str) -> Option<tiled::Layer> {
        self.map.layers().find(|x| x.name == layer_name)
    }
}

#[derive(Default, Resource)]
pub struct RoomStore {
    rooms: HashMap<u32, Vec<(Handle<MapAsset>, UVec2)>>,
}

impl RoomStore {
    pub fn insert(&mut self, map_handle: Handle<MapAsset>, map: &MapAsset) {
        let size = UVec2::new(map.map.width, map.map.height);

        self.rooms
            .entry(map.map_id)
            .or_default()
            .push((map_handle, size));
    }

    pub fn get_rooms(&self, map_id: u32) -> &[(Handle<MapAsset>, UVec2)] {
        &self.rooms[&map_id]
    }

    pub fn get_room_by_index(&self, map_id: u32, index: usize) -> &(Handle<MapAsset>, UVec2) {
        &self.rooms[&map_id][index]
    }
}

#[derive(Default)]
pub struct MapAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum MapAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load TMX map: {0}")]
    TmxError(#[from] tiled::Error),
}

impl AssetLoader for MapAssetLoader {
    type Asset = MapAsset;
    type Settings = ();
    type Error = MapAssetLoaderError;

    async fn load<'a>(
        &'a self,
        _reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut loader = tiled::Loader::new();

        let map_id_str = load_context
            .path()
            .parent()
            .expect("Invalid room path (no parent)")
            .file_name()
            .expect("Invalid room path (no file name)")
            .to_str()
            .expect("Invalid room path (unable to convert to str)")
            .to_string()
            .replace("map_0", "");
        let map_id = map_id_str.parse::<u32>().unwrap();

        let name = load_context
            .path()
            .file_stem()
            .expect("Invalid room path (no file stem)")
            .to_str()
            .expect("Invalid room path (unable to convert to str)")
            .to_string();

        let cmd = std::env::var("CARGO_MANIFEST_DIR");

        let assets_dir = std::path::Path::new("assets");
        let path = match cmd {
            Ok(dir) => std::path::Path::new(&dir)
                .join(assets_dir)
                .join(load_context.path()),
            Err(_) => {
                let cur_exe = std::env::current_exe().unwrap();
                cur_exe
                    .as_path()
                    .parent()
                    .unwrap()
                    .join(assets_dir)
                    .join(load_context.path())
            }
        };

        let map = loader.load_tmx_map(path)?;

        Ok(MapAsset { name, map_id, map })
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}
