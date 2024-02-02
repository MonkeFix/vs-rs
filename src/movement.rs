use crate::collisions::store::{ColliderIdResolver, ColliderStore};
use crate::collisions::{ColliderId, CollisionResult};
use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;

pub mod steering;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct Position(pub Vec2);

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
pub struct PhysicsParams {
    /// Determines how fast an object can move. This value is multiplied by delta time, so it
    /// probably should be higher than 100 for noticeable velocity.
    /// Defaults to `250.0`.
    pub max_velocity: f32,
    /// Stores the maximum impulse of a steering force applied to an object via a steering
    /// behavior. Set this value lower than `max_velocity` to achieve smooth acceleration.
    /// Defaults to `150.0`.
    pub max_force: f32,
    /// Determines how much inertia an object will have.
    /// Defaults to `4.0`.
    pub mass: f32,
    /// Determines how fast an object will decelerate. Lower values mean faster deceleration.
    /// Should be in range [0, 1] where 0 - instant stop, 1 - no deceleration at all.
    /// Defaults to `0.98`.
    pub friction: f32,
}

impl Default for PhysicsParams {
    fn default() -> Self {
        Self {
            max_velocity: 250.0,
            max_force: 150.0,
            mass: 4.0,
            friction: 0.98,
        }
    }
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Reflect)]
pub struct SteeringHost {
    pub velocity: Vec2,
    pub steering: Vec2,
    pub desired_velocity: Vec2,
}

#[derive(Bundle, Debug, Default)]
pub struct SteeringBundle {
    pub position: Position,
    pub steering: SteeringHost,
    pub physics_params: PhysicsParams,
}

#[derive(WorldQuery)]
#[world_query(mutable)]
pub struct SteeringHostQuery<'w> {
    position: &'w Position,
    host: &'w SteeringHost,
    params: &'w PhysicsParams,
}

pub fn calc_movement(
    motion: &mut Vec2,
    collider_id: ColliderId,
    collider_store: &ColliderStore,
) -> Option<CollisionResult> {
    let mut result = None;

    let collider = collider_store.get(collider_id).unwrap();

    if collider.is_trigger || !collider.is_registered {
        return None;
    }

    let mut bounds = collider.bounds();
    bounds.x += motion.x;
    bounds.y += motion.y;
    let neighbors = collider_store.aabb_broadphase_excluding_self(
        collider_id,
        bounds,
        Some(collider.collides_with_layers),
    );

    for id in neighbors {
        let neighbor = collider_store.get(id).unwrap();
        if neighbor.is_trigger {
            continue;
        }

        if let Some(collision) = collider.collides_with_motion(neighbor, *motion) {
            *motion -= collision.min_translation;

            result = Some(CollisionResult::from_ref(&collision));
        }
    }

    result
}
