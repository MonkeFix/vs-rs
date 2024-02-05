use bevy::ecs::query::WorldQuery;
use bevy::prelude::*;

use crate::collisions::plugin::ColliderComponent;

pub mod behaviors;
pub mod paths;
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

#[derive(Component, Debug, Default, Clone, PartialEq, Reflect)]
pub struct SteeringHost {
    pub velocity: Vec2,
    pub steering: Vec2,
    pub desired_velocity: Vec2,
}

impl SteeringHost {
    pub fn steer(&mut self, steering_vec: Vec2) {
        self.steering += steering_vec;
    }
}

#[derive(Bundle, Default)]
pub struct SteeringBundle {
    pub position: Position,
    pub steering: SteeringHost,
    pub physics_params: PhysicsParams,
}
