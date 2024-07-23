use bevy::{asset::AssetLoader, prelude::*, sprite::TextureAtlasLayout, utils::HashMap};
use rand::{thread_rng, Rng};
use thiserror::Error;

#[derive(Debug, Default, Clone, Asset, TypePath)]
pub struct AssetTileSheet {
    pub name: String,
    pub layout: Handle<TextureAtlasLayout>,
    pub image: Handle<Image>,
    pub named_tiles: Option<HashMap<String, Vec<u32>>>,
}

impl AssetTileSheet {
    pub fn as_texture_atlas(&self, index: usize) -> TextureAtlas {
        TextureAtlas {
            layout: self.layout.clone(),
            index,
        }
    }

    pub fn get_tile_ids(&self, tile_name: &str) -> Option<&Vec<u32>> {
        if let Some(tiles) = &self.named_tiles {
            return tiles.get(tile_name);
        }

        None
    }

    pub fn get_random_tile_id(&self, tile_name: &str) -> Option<u32> {
        let mut rng = thread_rng();

        if let Some(tiles) = &self.named_tiles {
            if let Some(tiles) = tiles.get(tile_name) {
                let num = rng.gen_range(0..tiles.len());
                return Some(tiles[num]);
            } else {
                return None;
            }
        }

        None
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct TsxTilesetAsset {
    pub tileset: tiled::Tileset,
}
#[derive(Default)]
pub struct TsxTilesetAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TsxTilesetAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load TMX map: {0}")]
    TmxError(#[from] tiled::Error),
}

impl AssetLoader for TsxTilesetAssetLoader {
    type Asset = TsxTilesetAsset;
    type Settings = ();
    type Error = TsxTilesetAssetLoaderError;

    async fn load<'a>(
        &'a self,
        _reader: &'a mut bevy::asset::io::Reader<'_>,
        _settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut loader = tiled::Loader::new();
        let path = format!("assets/{}", load_context.path().to_str().unwrap());
        info!("Loading tileset: {:?}", path);

        let tileset = loader.load_tsx_tileset(path)?;
        Ok(TsxTilesetAsset { tileset })
    }

    fn extensions(&self) -> &[&str] {
        &["tsx"]
    }
}
