use bevy::prelude::*;
use common::FRect;
use vs_assets::rooms::MapAsset;

pub struct WorldRoom {
    pub map_asset: Handle<MapAsset>,
    pub rect: FRect,
}
