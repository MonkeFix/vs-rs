use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypePath,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpawnWave {
    pub from: u16,
    pub to: u16,
    pub spawn_time: u64,
}

#[derive(Debug, Asset, TypePath, Clone, Serialize, Deserialize)]
pub struct EnemyConfig {
    pub param_list: Vec<EnemyParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyParams {
    pub name: String,
    pub dmg: i64,
    pub hp: i64,
    pub max_velocity: f32,
    pub max_force: f32,
    pub mass: f32,
    pub asset_path: String,
    pub is_elite: Option<bool>,
    pub spawn_waves: Vec<SpawnWave>,
    pub exp_drop: u32,
}

#[derive(Default)]
pub struct EnemyConfigLoader;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EnemyConfigLoaderError {
    #[error("Could not load config: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not parse JSON: {0}")]
    JsonError(#[from] serde_json::Error),
}

impl AssetLoader for EnemyConfigLoader {
    type Asset = EnemyConfig;
    type Settings = ();
    type Error = EnemyConfigLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut json_str = String::new();

        reader.read_to_string(&mut json_str).await?;

        let obj = serde_json::from_str::<Vec<EnemyParams>>(&json_str)?;

        Ok(EnemyConfig { param_list: obj })
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }
}
