use super::Point;

pub struct WorldGeneratorSettings {
    pub max_rooms: u32,
    pub min_used_area: u32,

    /// In tiles
    pub world_width: u32,
    /// In tiles
    pub world_height: u32,

    pub init_min_room_w: u32,
    pub init_min_room_h: u32,
    pub init_max_room_w: u32,
    pub init_max_room_h: u32,

    pub next_min_room_w: u32,
    pub next_min_room_h: u32,
    pub next_max_room_w: u32,
    pub next_max_room_h: u32,

    pub next_step_iterations: u32,
    pub max_room_iterations: u32,

    pub room_spacing: Point,

    pub random_edge_inclusion_chance: f32,

    pub cost_empty_space: u32,
    pub cost_room: u32,
    pub cost_hallway: u32,
}

impl Default for WorldGeneratorSettings {
    fn default() -> Self {
        Self {
            max_rooms: 100,
            min_used_area: 100_000_000,

            world_width: 256,
            world_height: 256,

            init_min_room_w: 16,
            init_min_room_h: 16,
            init_max_room_w: 32,
            init_max_room_h: 32,

            next_min_room_w: 8,
            next_min_room_h: 8,
            next_max_room_w: 32,
            next_max_room_h: 32,

            next_step_iterations: 4_000,
            max_room_iterations: 400_000,
            room_spacing: Point { x: 4, y: 4 },

            random_edge_inclusion_chance: 0.125,

            cost_empty_space: 5,
            cost_room: 1,
            cost_hallway: 2,
        }
    }
}
