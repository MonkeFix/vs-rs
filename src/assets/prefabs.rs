use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TiledPrefab {
    pub layers: Vec<TiledPrefabLayer>,
    pub tileheight: u32,
    pub tilewidth: u32,
    pub width: u32,
    pub height: u32,
}

impl TiledPrefab {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[derive(Debug, Deserialize)]
pub struct TiledPrefabLayer {
    pub id: u32,
    pub name: String,
    pub visible: bool,
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub opacity: f32,
    pub data: Vec<u32>,
}
