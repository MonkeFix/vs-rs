use bevy::{
    asset::{io::Reader, Asset, AssetLoader, LoadContext}, log::info, prelude::*, reflect::TypePath, utils::{HashMap, HashSet}
};
use thiserror::Error;

#[derive(Asset, TypePath, Debug, Clone)]
pub struct MapAsset {
    pub name: String,
    pub map_id: u32,
    pub map: tiled::Map,
}

#[derive(Default, Resource)]
pub struct RoomStore {
    pub maps: HashMap<u32, Vec<MapAsset>>,
}

impl RoomStore {
    pub fn get_room_sizes(&self, map_id: u32) -> HashSet<UVec2> {
        let rooms = self.maps.get(&map_id).expect("Invalid map id");
        rooms
            .iter()
            .map(|m| UVec2::new(m.map.width, m.map.height))
            .collect()
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
        let path = format!("assets/{}", load_context.path().to_str().unwrap());
        info!("Loading map: {:?}", path);

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

        let map = loader.load_tmx_map(path)?;
        Ok(MapAsset { name, map_id, map })
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}
