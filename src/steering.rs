use bevy::prelude::*;
use crate::stats::{Mass, MaxForce, MaxVelocity};

pub trait SteeringTarget {
    /// Returns target's position.
    fn position(&self) -> Vec2;
    /// Returns target's velocity. Defaults to `Vec2::ZERO`.
    fn velocity(&self) -> Vec2 {
        Vec2::ZERO
    }
}

impl SteeringTarget for Vec2 {
    fn position(&self) -> Vec2 {
        *self
    }
}

impl SteeringTarget for SteeringHost {
    fn position(&self) -> Vec2 {
        self.position
    }

    fn velocity(&self) -> Vec2 {
        self.cur_velocity
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct SteerResult {
    pub steering_vec: Vec2,
    pub desired_velocity: Vec2,
}

pub trait SteeringBehavior {
    fn steer(&mut self, host: &SteeringHost, target: &impl SteeringTarget) -> SteerResult;
    /// If the behavior is additive, steering vector adds on top of the result of `steer()` method.
    /// Otherwise, the vector is directly assigned the value.
    fn is_additive(&self) -> bool {
        false
    }
}

/// Seeks the target, directly moving towards it.
#[derive(Debug, Default)]
pub struct SteerSeek;

impl SteeringBehavior for SteerSeek {
    fn steer(&mut self, host: &SteeringHost, target: &impl SteeringTarget) -> SteerResult {
        let dv = target.position() - host.position;
        let dv = dv.normalize_or_zero();

        SteerResult {
            steering_vec: dv * host.max_velocity.0 - host.cur_velocity,
            desired_velocity: dv,
        }
    }
}

/// Flees from the target, moving away from it.
#[derive(Debug, Default)]
pub struct SteerFlee;

impl SteeringBehavior for SteerFlee {
    fn steer(&mut self, host: &SteeringHost, target: &impl SteeringTarget) -> SteerResult {
        let dv = target.position() - host.position;
        let dv = dv.normalize_or_zero() * host.max_velocity.0;

        SteerResult {
            steering_vec: dv - host.cur_velocity,
            desired_velocity: -dv,
        }
    }
}

/// Calculates future position of the target and moves towards it.
#[derive(Debug, Default)]
pub struct SteerPursuit {
    pub seek: SteerSeek,
}

impl SteerPursuit {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SteeringBehavior for SteerPursuit {
    fn steer(&mut self, host: &SteeringHost, target: &impl SteeringTarget) -> SteerResult {
        let distance = (target.position() - host.position).length();
        let updates_ahead = distance / host.max_velocity.0;
        let future_pos = target.position() + target.velocity() * updates_ahead;

        self.seek.steer(host, &future_pos)
    }
}

/// Seeks the target. The closer the target, the slower the entity.
#[derive(Debug, Default)]
pub struct SteerArrival {
    pub slowing_radius: f32,
}

impl SteerArrival {
    pub fn new() -> Self {
        Self {
            slowing_radius: 0.0,
        }
    }
}

impl SteeringBehavior for SteerArrival {
    fn steer(&mut self, host: &SteeringHost, target: &impl SteeringTarget) -> SteerResult {
        let mut dv = target.position() - host.position;
        let distance = dv.length();
        dv = dv.normalize_or_zero();

        let steering = if distance < self.slowing_radius {
            host.max_velocity.0 * (distance / self.slowing_radius)
        } else {
            host.max_velocity.0
        };

        SteerResult {
            steering_vec: dv * steering,
            desired_velocity: dv,
        }
    }
}

#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect)]
#[reflect(Component, Default, PartialEq)]
pub struct SteeringHost {
    pub position: Vec2,

    pub desired_velocity: Vec2,
    pub cur_velocity: Vec2,
    pub steering: Vec2,

    /// The highest speed entity can get to.
    pub max_velocity: MaxVelocity,
    pub max_force: MaxForce,
    pub mass: Mass,
    pub friction: f32,
}

impl Default for SteeringHost {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,

            desired_velocity: Vec2::ZERO,
            cur_velocity: Vec2::ZERO,
            steering: Vec2::ZERO,

            max_velocity: MaxVelocity(250.0),
            max_force: MaxForce(150.0),
            mass: Mass(4.0),
            friction: 0.98,
        }
    }
}

impl SteeringHost {
    pub fn steer(
        &mut self,
        mut steering_behavior: impl SteeringBehavior,
        target: &impl SteeringTarget,
    ) {
        let res = steering_behavior.steer(self, target);
        self.desired_velocity = res.desired_velocity;

        if steering_behavior.is_additive() {
            self.steering += res.steering_vec;
        } else {
            self.steering = res.steering_vec;
        }
    }
}

#[derive(Bundle, Reflect)]
pub struct SteeringBundle {
    pub host: SteeringHost,
}

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (steer, update_position, update_translation).chain());
    }
}

fn update_translation(mut host: Query<(&mut Transform, &SteeringHost)>) {
    for (mut transform, host) in &mut host {
        transform.translation = host.position.extend(1.0);
    }
}

fn update_position(time: Res<Time>, mut host: Query<&mut SteeringHost>) {
    for mut host in &mut host {
        let dt = host.cur_velocity * time.delta_seconds();
        host.position += dt;

        let friction = host.friction;
        host.cur_velocity *= friction;
    }
}

fn steer(mut host: Query<&mut SteeringHost>) {
    for mut host in &mut host {
        let mass = host.mass.0;

        host.steering = crate::math::truncate_vec2(host.steering, host.max_force.0);
        host.steering /= mass;

        let steering = host.steering;
        host.cur_velocity =
            crate::math::truncate_vec2(host.cur_velocity + steering, host.max_velocity.0);
    }
}
