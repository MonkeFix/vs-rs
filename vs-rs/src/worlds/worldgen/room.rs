use crate::assets::rooms::MapAsset;
use bevy::prelude::*;
use common::FRect;

pub struct WorldRoom {
    pub map_asset: Handle<MapAsset>,
    pub rect: FRect,
}
