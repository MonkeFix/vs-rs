use bevy::{asset::AssetLoader, prelude::*, sprite::TextureAtlasLayout, utils::HashMap};
use rand::{thread_rng, Rng};
use thiserror::Error;
use tiled::Tileset;

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

    pub fn create_layout(
        tilesheet: &Tileset,
        tilesheet_texture: Handle<Image>,
        layouts: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
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

        let img = tilesheet.image.as_ref().expect("Image must not be empty");

        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(tilesheet.tile_width, tilesheet.tile_height),
            tilesheet.columns,
            img.height as u32 / tilesheet.tile_height,
            None,
            None,
        );
        let layout_handle = layouts.add(layout);

        Self {
            name: tilesheet.name.clone(),
            layout: layout_handle,
            image: tilesheet_texture,
            named_tiles: Some(named_tiles),
        }
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct TsxTilesetAsset {
    pub tileset: tiled::Tileset,
    pub image_handle: Option<Handle<Image>>,
}
#[derive(Default)]
pub struct TsxTilesetAssetLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TsxTilesetAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load TSX tileset: {0}")]
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

        info!("Loading tileset: {:?}", path);
        let tileset = loader.load_tsx_tileset(path)?;

        let image_handle = if let Some(img) = &tileset.image {
            let tileset_texture = load_context.load::<Image>(img.source.clone());
            Some(tileset_texture)
        } else {
            None
        };

        Ok(TsxTilesetAsset {
            tileset,
            image_handle,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tsx"]
    }
}
