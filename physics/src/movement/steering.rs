use bevy::prelude::*;

pub trait SteeringTarget {
    fn position(&self) -> Vec2;
    fn velocity(&self) -> Vec2 {
        Vec2::ZERO
    }
}

impl SteeringTarget for Vec2 {
    fn position(&self) -> Vec2 {
        *self
    }
}

impl SteeringTarget for Transform {
    fn position(&self) -> Vec2 {
        self.translation.xy()
    }
}

/// A component that stores only the position part of the `SteeringTarget` trait.
#[derive(Component, Debug, Default)]
pub struct SteeringTargetVec2(pub Vec2);

/// A component that stores both `position` and `velocity` of the `SteeringTarget` trait.
#[derive(Component, Debug, Default)]
pub struct SteeringTargetFull {
    pub position: Vec2,
    pub velocity: Vec2,
}

impl SteeringTarget for SteeringTargetFull {
    fn position(&self) -> Vec2 {
        self.position
    }

    fn velocity(&self) -> Vec2 {
        self.velocity
    }
}

/// A component that stores an `Entity` that is served as a target.
/// Implements the `SteeringTarget` trait, but `position()` and `velocity()` always return `Vec2::ZERO`.
/// For actual values you'll need to query them: `Transform` for position and `SteeringHost` for velocity.
#[derive(Component, Debug)]
pub struct SteeringTargetEntity(pub Entity);

impl SteeringTarget for SteeringTargetEntity {
    fn position(&self) -> Vec2 {
        Vec2::ZERO
    }
}

impl SteeringTarget for SteeringTargetVec2 {
    fn position(&self) -> Vec2 {
        self.0
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Reflect)]
pub struct PhysicalParams {
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

impl Default for PhysicalParams {
    fn default() -> Self {
        Self {
            max_velocity: 250.0,
            max_force: 150.0,
            mass: 4.0,
            friction: 0.9,
        }
    }
}

/// Represents a `Component` which stores movement data.
#[derive(Component, Debug, Default, Clone, PartialEq, Reflect)]
pub struct SteeringHost {
    /// Current velocity.
    pub velocity: Vec2,
    /// Indicates the steering vector which is the sum of all steering behaviors.
    pub steering: Vec2,
    /// Indicates where the host is wanting to move to.
    pub desired_velocity: Vec2,
    /// An actual movement the host is going to perform.
    /// Is directly applied to current position by summing these two values together.
    pub movement: Vec2,
}

impl SteeringHost {
    /// Applies a steering vector to the host.
    pub fn steer(&mut self, steering_vec: Vec2) {
        self.steering += steering_vec;
    }
}

#[derive(Bundle, Default)]
pub struct SteeringBundle {
    pub steering: SteeringHost,
    pub physics_params: PhysicalParams,
}
