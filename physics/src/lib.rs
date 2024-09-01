use bevy::prelude::*;

pub mod collisions;
pub mod movement;
pub mod plugin;
pub mod prelude;

#[derive(Debug, Event)]
pub(crate) struct MovementCalculateEvent {
    entity: Entity,
    movement: Vec2,
}

#[derive(Debug, Event)]
pub(crate) struct PositionUpdateEvent {
    entity: Entity,
    movement: Vec2,
}

#[derive(Debug, Event)]
pub struct InvokeTriggerEvent {
    pub entity_main: Entity,
    pub entity_trigger: Entity,
}

#[derive(Debug, Event)]
pub struct CollideEvent {
    pub entity_main: Entity,
    pub collided_with: Entity,
}
