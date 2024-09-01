use super::steering::{
    PhysicalParams, SteeringHost, SteeringTarget, SteeringTargetEntity, SteeringTargetVec2,
};
use bevy::prelude::*;
use common::math::rng_f32;

/// Seeks the specified target moving directly towards it.
#[derive(Component, Default)]
pub struct SteerSeek;

pub(crate) fn steer_seek(
    mut hosts: Query<(
        &SteerSeek,
        &SteeringTargetEntity,
        &mut SteeringHost,
        &Transform,
        &PhysicalParams,
    )>,
    targets: Query<&Transform>,
) {
    for (seek, target_entity, mut host, transform, params) in hosts.iter_mut() {
        if let Ok(target) = targets.get(target_entity.0) {
            let steering = seek.steer(transform, &host, params, target);
            host.steer(steering);
        }
    }
}

pub(crate) fn steer_seek_vec2(
    mut hosts: Query<(
        &SteerSeek,
        &SteeringTargetVec2,
        &mut SteeringHost,
        &Transform,
        &PhysicalParams,
    )>,
) {
    for (seek, target, mut host, transform, params) in hosts.iter_mut() {
        if target.0 != Vec2::ZERO {
            let steering = seek.steer(transform, &host, params, &target.0);
            host.steer(steering);
        } 
    }
}

impl SteerSeek {
    pub fn steer(
        &self,
        position: &Transform,
        host: &SteeringHost,
        params: &PhysicalParams,
        target: &impl SteeringTarget,
    ) -> Vec2 {
        let dv = target.position() - position.translation.xy();
        let dv = dv.normalize_or_zero();

        dv * params.max_velocity - host.velocity
    }
}

/// Flees from the specified target moving away from it.
/// Works the same way as `SteerSeek` but the result vector is inverted.
pub struct SteerFlee;

impl SteerFlee {
    pub fn steer(
        &self,
        position: &Transform,
        host: &SteeringHost,
        params: &PhysicalParams,
        target: &impl SteeringTarget,
    ) -> Vec2 {
        let dv = target.position() - position.translation.xy();
        let dv = dv.normalize_or_zero();

        -dv * params.max_velocity - host.velocity
    }
}

/// Moves towards the specified target slowing down gradually as the host is
/// getting closer. The slowing starts when the host is within circle with radius
/// `slowing_radius`.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct SteerArrival {
    pub slowing_radius: f32,
}

impl Default for SteerArrival {
    fn default() -> Self {
        Self {
            slowing_radius: 16.0,
        }
    }
}

impl SteerArrival {
    pub fn steer(
        &self,
        position: &Transform,
        host: &SteeringHost,
        params: &PhysicalParams,
        target: &impl SteeringTarget,
    ) -> Vec2 {
        let dv = target.position() - position.translation.xy();
        let distance = dv.length();
        let dv = if distance < self.slowing_radius {
            dv.normalize_or_zero() * params.max_velocity * (distance / self.slowing_radius)
        } else {
            dv.normalize_or_zero() * params.max_velocity
        };

        dv - host.velocity
    }
}

/// Moves away from the target with prediction of the target's future position.
pub struct SteerEvade;

impl SteerEvade {
    pub fn steer(
        &self,
        position: &Transform,
        host: &SteeringHost,
        params: &PhysicalParams,
        target: &impl SteeringTarget,
    ) -> Vec2 {
        let distance = (target.position() - position.translation.xy()).length();
        let updates_ahead = distance / params.max_velocity;

        let future_pos = target.position() + target.velocity() * updates_ahead;

        SteerFlee.steer(position, host, params, &future_pos)
    }
}

/// Moves towards future position of the target, predicting it.
pub struct SteerPursuit;

impl SteerPursuit {
    pub fn steer(
        &self,
        position: &Transform,
        host: &SteeringHost,
        params: &PhysicalParams,
        target: &impl SteeringTarget,
    ) -> Vec2 {
        let distance = (target.position() - position.translation.xy()).length();
        let updates_ahead = distance / params.max_velocity;

        let future_pos = target.position() + target.velocity() * updates_ahead;

        SteerSeek.steer(position, host, params, &future_pos)
    }
}

/// Wanders around randomly changing host's angle.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct SteerWander {
    pub circle_distance: f32,
    pub circle_radius: f32,
    pub wander_angle: f32,
    pub angle_change: f32,
}

impl Default for SteerWander {
    fn default() -> Self {
        Self {
            circle_distance: 16.0,
            circle_radius: 8.0,
            wander_angle: std::f32::consts::FRAC_PI_2,
            angle_change: 0.1,
        }
    }
}

impl SteerWander {
    fn set_angle(&self, mut vec: Vec2, value: f32) -> Vec2 {
        let length = vec.length();
        vec.x = value.cos() * length;
        vec.y = value.sin() * length;
        vec
    }
}

impl SteerWander {
    pub fn steer(&mut self, host: &SteeringHost, params: &PhysicalParams) -> Vec2 {
        let circle_center = host.velocity.normalize_or_zero() * self.circle_distance;

        let displacement = Vec2::new(0.0, -1.0) * self.circle_radius;
        let displacement = self.set_angle(displacement, self.wander_angle);

        let next = rng_f32(-self.angle_change, self.angle_change);
        self.wander_angle += next;

        let wander_force = circle_center + displacement;

        wander_force.normalize_or_zero() * params.max_velocity - host.velocity
    }
}
/*
/// Tries to avoid all collisions by checking the closest threatening collider on its way.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct SteerCollisionAvoidance {
    pub max_see_ahead: f32,
    pub avoid_force: f32,
    ahead: Vec2,
    avoidance: Vec2,
}

impl Default for SteerCollisionAvoidance {
    fn default() -> Self {
        Self {
            max_see_ahead: 16.0,
            avoid_force: 75.0,
            ..default()
        }
    }
}

impl SteerCollisionAvoidance {
    pub fn steer(
        &mut self,
        position: &Transform,
        host: &SteeringHost,
        params: &PhysicalParams,
        collider: &Collider,
        hash: &SpatialHash,
        layer_mask: Option<i32>,
    ) -> Vec2 {
        let dv = host.velocity.normalize_or_zero()
            * (self.max_see_ahead * host.velocity.length() / params.max_velocity);

        self.ahead = position.translation.xy() + dv;

        let mut rect = collider.bounds();
        rect.x += self.ahead.x;
        rect.y += self.ahead.y;

        let neighbors =
            hash.aabb_broadphase(collider.id, rect, layer_mask);

        let mut distance = f32::MAX;
        let mut closest = None;

        for neighbor_id in neighbors {
            let neighbor = collider_store.get(neighbor_id).unwrap();

            let d = (neighbor.position() - collider.position()).length();
            if d < distance {
                distance = d;
                closest = Some(neighbor);
            }
        }

        if let Some(closest) = closest {
            self.avoidance =
                (self.ahead - closest.position()).normalize_or_zero() * self.avoid_force;
        } else {
            self.avoidance *= 0.0;
        }

        self.avoidance
    }
}

/// Tries to separate from other colliders.
/// Separation radius is defined by the `radius` field.
#[derive(Debug, Reflect)]
pub struct SteerSeparation {
    pub radius: f32,
    pub max_force: f32,
}

impl Default for SteerSeparation {
    fn default() -> Self {
        Self {
            radius: 32.0,
            max_force: 75.0,
        }
    }
}

impl SteerSeparation {
    pub fn steer(
        &self,
        position: &Transform,
        collider: &Collider,
        layer_mask: Option<i32>,
    ) -> Vec2 {
        let mut force = Vec2::ZERO;

        let mut rect = collider_store.get(collider.id).unwrap().bounds();
        rect.inflate(self.radius, self.radius);

        // TODO: Check if this method works, if not, use aabb_broadphase()
        let neighbors =
            collider_store.overlap_circle(position.translation.xy(), self.radius, Some(collider.id), layer_mask);
        let neighbor_count = neighbors.len();

        for neighbor_id in neighbors {
            let neighbor = collider_store.get(neighbor_id).unwrap();
            force += neighbor.position() - position.translation.xy();
        }

        if neighbor_count != 0 {
            force /= neighbor_count as f32;
            force *= -1.0;
        }

        force.normalize_or_zero() * self.max_force
    }
}

#[derive(Debug, Clone, Copy, Reflect)]
pub struct SteerQueue {
    pub max_radius: f32,
    pub max_ahead: f32,
    pub brake_coef: f32,
    pub velocity_mult: f32,
}

impl Default for SteerQueue {
    fn default() -> Self {
        Self {
            max_radius: 16.0,
            max_ahead: 16.0,
            brake_coef: 0.8,
            velocity_mult: 0.3,
        }
    }
}

/// Imitates a smooth queue of steering hosts by slowing down if another host
/// is ahead of it.
pub struct SteerQueueResult {
    pub steering: Vec2,
    pub velocity_multiplier: f32,
}

impl SteerQueue {
    pub fn steer(
        &self,
        position: &Transform,
        host: &SteeringHost,
        collider: &Collider,
        layer_mask: Option<i32>,
    ) -> SteerQueueResult {
        let mut velocity = host.velocity;
        let mut brake = Vec2::ZERO;
        let mut velocity_multiplier = 1.0;

        let neighbor =
            self.get_neighbor_ahead(position, host, collider, collider_store, layer_mask);
        if let Some(neighbor) = neighbor {
            brake = -host.steering * self.brake_coef;
            velocity *= -1.0;
            brake += velocity;

            if (neighbor.position() - position.translation.xy()).length() <= self.max_radius {
                velocity_multiplier = self.velocity_mult;
            }
        }

        SteerQueueResult {
            steering: brake,
            velocity_multiplier,
        }
    }

    fn get_neighbor_ahead(
        &self,
        position: &Transform,
        host: &SteeringHost,
        collider: &Collider,
        layer_mask: Option<i32>,
    ) -> Option<&Collider> {
        let qa = host.velocity.normalize_or_zero() * self.max_ahead;

        let ahead = position.translation.xy() + qa;

        let collider = collider_store.get(collider.id).unwrap();
        // TODO: Check if this method works, if not, use aabb_broadphase()
        let neighbors = collider_store.overlap_circle(
            position.translation.xy(),
            self.max_radius,
            Some(collider.id),
            layer_mask,
        );

        let mut closest = None;
        let mut distance = f32::MAX;

        for neighbor_id in neighbors {
            let neighbor = collider_store.get(neighbor_id).unwrap();
            let d = (neighbor.position() - ahead).length();
            if d < distance {
                distance = d;
                closest = Some(neighbor);
            }
        }

        closest
    }
}

/// Follows a specified "leader" entity. The host will try to stay behind the leader
/// by moving away from leader's field of view.
///
/// `steer` method updates `ahead` and `behind` vectors. You can check if your
/// steering host is on leader's sight by calling the `is_on_leader_sight()` method.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct SteerLeaderFollowing {
    pub leader_behind_dist: f32,
    pub leader_sight_radius: f32,
    pub ahead: Vec2,
    pub behind: Vec2,
}

impl Default for SteerLeaderFollowing {
    fn default() -> Self {
        Self {
            leader_behind_dist: 32.0,
            leader_sight_radius: 32.0,
            ..default()
        }
    }
}

impl SteerLeaderFollowing {
    pub fn steer(&mut self, position: &Transform, host: &SteeringHost) -> Vec2 {
        let mut dv = host.velocity.normalize_or_zero() * self.leader_behind_dist;

        self.ahead = position.translation.xy() + dv;
        dv *= -1.0;
        self.behind = position.translation.xy() + dv;

        self.behind
    }

    pub fn get_leader_ahead(&self, position: &Transform, host: &SteeringHost) -> Vec2 {
        let dv = host.velocity.normalize_or_zero() * self.leader_behind_dist;
        position.translation.xy() + dv
    }

    pub fn is_on_leader_sight(&self, leader_position: &Transform, position: &Transform) -> bool {
        (self.ahead - position.translation.xy()).length() <= self.leader_sight_radius
            || (leader_position.translation.xy() - position.translation.xy()).length() <= self.leader_sight_radius
    }
}
 */
