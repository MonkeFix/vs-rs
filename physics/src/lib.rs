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
    entity_main: Entity,
    entity_trigger: Entity
}
