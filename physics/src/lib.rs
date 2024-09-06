//! Super-duper collision checking and resloving library.
//! Also contains some useful steering behaviors.
//!
//! # Examples
//!
//! In order to make colliders work you'll need to spawn a collider:
//!
//! ```
//! fn spawn_colliders(mut commands: Commands) {
//!     commands.spawn((
//!         SpatialBundle::default(),
//!         Collider {
//!             // Let the shape to be a circle with radius 10.0
//!             shape: Shape::new(shapes::ShapeType::Circle { radius: 10.0 }),
//!             // Set the collider's relative position to be a little lower than the entity's position
//!             local_offset: Vec2::new(0.0, -16.0),
//!             // Set the physics layer to be `2`
//!             physics_layer: 2,
//!             // This collider should collide only with layers 1 and 2 (equivalent to 0b11)
//!             collides_with_layers: 1 | 2,
//!             // This collider shouldn't be a trigger
//!             is_trigger: false
//!         }
//!     ))
//! }
//! ```
//!
//! You also can make the collider move by adding `SteerSeek` (or other behavior)
//! as well as `SteeringTargetVec2`, with an alternative to `SteeringTargetEntity(my_target_entity)`.
//!
//! You can mutate steering targets freely at any time.
//!
//! You can also observe collisions by calling `observe(on_collision)` after spawning an entity:
//! ```
//! fn on_collision(trigger: Trigger<CollideEvent>) {
//!     let event = trigger.event();
//!     info!("Collision occured! Event data: {:?}", event);
//! }
//! ```

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

/// An event that triggers when a collider collides with a trigger.
/// This event can be accessed through `EventReader<InvokeTriggerEvent>`.
#[derive(Debug, Event)]
pub struct InvokeTriggerEvent {
    pub entity_main: Entity,
    pub entity_trigger: Entity,
}

/// An event that triggers when a collider collides with another collider
/// whose `is_trigger` value is set to `false`.
/// This event can be observed through either a global observer
/// ```
/// commands.observe(my_observe_system);
/// ```
/// or a targeted observer
/// ```
/// commands.spawn(SpatialBundle::default()).observe(my_observe_system);
/// ```
#[derive(Debug, Event)]
pub struct CollideEvent {
    /// The main `Entity` which moved and thus collided another `Entity`.
    pub entity_main: Entity,
    /// The `Entity` main collider collided with.
    pub collided_with: Entity,
}
