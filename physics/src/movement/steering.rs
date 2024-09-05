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

#[derive(Component, Debug, Default)]
pub struct SteeringTargetComponent;

#[derive(Component, Debug, Default)]
pub struct SteeringTargetVec2(pub Vec2);

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

#[derive(Component, Debug, Default, Clone, PartialEq, Reflect)]
pub struct SteeringHost {
    pub velocity: Vec2,
    pub steering: Vec2,
    pub desired_velocity: Vec2,
    pub movement: Vec2,
}

impl SteeringHost {
    pub fn steer(&mut self, steering_vec: Vec2) {
        self.steering += steering_vec;
    }
}

#[derive(Bundle, Default)]
pub struct SteeringBundle {
    pub steering: SteeringHost,
    pub physics_params: PhysicalParams,
}
