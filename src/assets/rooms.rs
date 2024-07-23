use bevy::{asset::{io::Reader, Asset, AssetLoader, LoadContext}, log::info, reflect::TypePath};
use thiserror::Error;

#[derive(Asset, TypePath, Debug)]
pub struct MapAsset {
    pub map: tiled::Map,
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

        let map = loader.load_tmx_map(path)?;
        Ok(MapAsset { map })
    }

    fn extensions(&self) -> &[&str] {
        &["tmx"]
    }
}