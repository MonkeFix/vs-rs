use super::WorldPoint;

pub struct WorldGeneratorSettings {
    pub max_rooms: u32,
    pub min_used_area: u32,

    /// In tiles
    pub world_width: u32,
    /// In tiles
    pub world_height: u32,
    pub max_room_iterations: u32,

    pub room_spacing: WorldPoint,

    pub cost_empty_space: u32,
    pub cost_room: u32,
    pub cost_hallway: u32,
    pub cost_wall: u32,

    pub map_id: u32,
}

impl Default for WorldGeneratorSettings {
    fn default() -> Self {
        Self {
            max_rooms: 35,
            min_used_area: 50_000, // max: 65_536 (256*256)

            world_width: 256,
            world_height: 256,

            max_room_iterations: 400_000,
            room_spacing: WorldPoint { x: 8, y: 8 },

            cost_empty_space: 2,
            cost_room: 1,
            cost_hallway: 3,
            cost_wall: 20,

            map_id: 1,
        }
    }
}
