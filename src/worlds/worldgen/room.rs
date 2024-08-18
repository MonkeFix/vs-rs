use crate::{assets::rooms::MapAsset, collisions::Rect};
use bevy::prelude::*;

pub struct WorldRoom {
    pub map_asset: MapAsset,
    pub rect: Rect,
}
