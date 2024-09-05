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

    pub fn load_by_name(
        name: &str,
        asset_server: &AssetServer,
        layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        info!("Loading tilesheet '{}.tsx'", name);

        let mut loader = tiled::Loader::new();
        let tilesheet = loader
            .load_tsx_tileset(format!("assets/{}.tsx", name))
            .unwrap_or_else(|_| panic!("could not read file '{}'.tsx", name));

        // Setting up named tiles (tiles with non-empty type described in the tile sheet)
        let mut named_tiles: HashMap<String, Vec<u32>> = HashMap::new();

        for (i, tile) in tilesheet.tiles() {
            if let Some(ut) = &tile.user_type {
                if named_tiles.contains_key(ut) {
                    named_tiles.get_mut(ut).unwrap().push(i);
                } else {
                    named_tiles.insert(ut.to_string(), vec![i]);
                }
            }
        }

        //dbg!(&named_tiles);

        let img = tilesheet.image.expect("Image must not be empty");

        // tilesheet name and texture name must match, and we're not just taking img.source
        // because tsx loader fucks up the path from being 'assets/textures/a.png'
        // to 'assets/assets/textures/a.png'
        let texture_handle = asset_server.load(format!("textures/{}.png", name));

        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(tilesheet.tile_width, tilesheet.tile_height),
            tilesheet.columns,
            img.height as u32 / tilesheet.tile_height,
            None,
            None,
        );
        let layout_handle = layouts.add(layout);

        AssetTileSheet {
            name: name.to_string(),
            layout: layout_handle,
            image: texture_handle,
            named_tiles: Some(named_tiles),
        }
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
