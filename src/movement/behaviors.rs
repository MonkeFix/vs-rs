use std::collections::VecDeque;

use bevy::prelude::*;

use crate::{
    collisions::{
        colliders::Collider,
        store::{ColliderIdResolver, ColliderStore},
    },
    math::rng_f32,
};

use super::{steering::SteeringTarget, SteeringHostQuery};

/// Seeks the specified target moving directly towards it.
pub struct SteerSeek;

impl SteerSeek {
    pub fn steer(&self, query: &SteeringHostQuery, target: &impl SteeringTarget) -> Vec2 {
        let dv = target.position() - query.position.0;
        let dv = dv.normalize_or_zero();

        dv * query.params.max_velocity - query.host.velocity
    }
}

/// Flees from the specified target moving away from it.
/// Works the same way as `SteerSeek` but the result vector is inverted.
pub struct SteerFlee;

impl SteerFlee {
    pub fn steer(&self, query: &SteeringHostQuery, target: &impl SteeringTarget) -> Vec2 {
        let dv = target.position() - query.position.0;
        let dv = dv.normalize_or_zero();

        -dv * query.params.max_velocity - query.host.velocity
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
    pub fn steer(&self, query: &SteeringHostQuery, target: &impl SteeringTarget) -> Vec2 {
        let dv = target.position() - query.position.0;
        let distance = dv.length();
        let dv = if distance < self.slowing_radius {
            dv.normalize_or_zero() * query.params.max_velocity * (distance / self.slowing_radius)
        } else {
            dv.normalize_or_zero() * query.params.max_velocity
        };

        dv - query.host.velocity
    }
}

/// Moves away from the target with prediction of the target's future position.
pub struct SteerEvade;

impl SteerEvade {
    pub fn steer(&self, query: &SteeringHostQuery, target: &impl SteeringTarget) -> Vec2 {
        let distance = (target.position() - query.position.0).length();
        let updates_ahead = distance / query.params.max_velocity;

        let future_pos = target.position() + target.velocity() * updates_ahead;

        SteerFlee.steer(query, &future_pos)
    }
}

/// Moves towards future position of the target, predicting it.
pub struct SteerPursuit;

impl SteerPursuit {
    pub fn steer(&self, query: &SteeringHostQuery, target: &impl SteeringTarget) -> Vec2 {
        let distance = (target.position() - query.position.0).length();
        let updates_ahead = distance / query.params.max_velocity;

        let future_pos = target.position() + target.velocity() * updates_ahead;

        SteerSeek.steer(query, &future_pos)
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
    pub fn steer(&mut self, query: &SteeringHostQuery) -> Vec2 {
        let circle_center = query.host.velocity.normalize_or_zero() * self.circle_distance;

        let displacement = Vec2::new(0.0, -1.0) * self.circle_radius;
        let displacement = self.set_angle(displacement, self.wander_angle);

        let next = rng_f32(-self.angle_change, self.angle_change);
        self.wander_angle += next;

        let wander_force = circle_center + displacement;

        wander_force.normalize_or_zero() * query.params.max_velocity - query.host.velocity
    }
}

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
        query: &SteeringHostQuery,
        collider_store: &ColliderStore,
        layer_mask: Option<i32>,
    ) -> Vec2 {
        let dv = query.host.velocity.normalize_or_zero()
            * (self.max_see_ahead * query.host.velocity.length() / query.params.max_velocity);

        self.ahead = query.position.0 + dv;

        let collider = collider_store.get(query.collider.id).unwrap();
        let mut rect = collider.bounds();
        rect.x += self.ahead.x;
        rect.y += self.ahead.y;

        let neighbors =
            collider_store.aabb_broadphase_excluding_self(query.collider.id, rect, layer_mask);

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
        query: &SteeringHostQuery,
        collider_store: &ColliderStore,
        layer_mask: Option<i32>,
    ) -> Vec2 {
        let mut force = Vec2::ZERO;

        let mut rect = collider_store.get(query.collider.id).unwrap().bounds();
        rect.inflate(self.radius, self.radius);

        // TODO: Check if this method works, if not, use aabb_broadphase()
        let neighbors = collider_store.overlap_circle(
            query.position.0,
            self.radius,
            Some(query.collider.id),
            layer_mask,
        );
        let neighbor_count = neighbors.len();

        for neighbor_id in neighbors {
            let neighbor = collider_store.get(neighbor_id).unwrap();
            force += neighbor.position() - query.position.0;
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
        query: &SteeringHostQuery,
        collider_store: &ColliderStore,
        layer_mask: Option<i32>,
    ) -> SteerQueueResult {
        let mut velocity = query.host.velocity;
        let mut brake = Vec2::ZERO;
        let mut velocity_multiplier = 1.0;

        let neighbor = self.get_neighbor_ahead(query, collider_store, layer_mask);
        if let Some(neighbor) = neighbor {
            brake = -query.host.steering * self.brake_coef;
            velocity *= -1.0;
            brake += velocity;

            if (neighbor.position() - query.position.0).length() <= self.max_radius {
                velocity_multiplier = self.velocity_mult;
            }
        }

        SteerQueueResult {
            steering: brake,
            velocity_multiplier,
        }
    }

    fn get_neighbor_ahead<'a>(
        &self,
        query: &SteeringHostQuery,
        collider_store: &'a ColliderStore,
        layer_mask: Option<i32>,
    ) -> Option<&'a Collider> {
        let qa = query.host.velocity.normalize_or_zero() * self.max_ahead;

        let ahead = query.position.0 + qa;

        let collider = collider_store.get(query.collider.id).unwrap();
        // TODO: Check if this method works, if not, use aabb_broadphase()
        let neighbors = collider_store.overlap_circle(
            query.position.0,
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
    pub fn steer(&mut self, leader_query: &SteeringHostQuery) -> Vec2 {
        let mut dv = leader_query.host.velocity.normalize_or_zero() * self.leader_behind_dist;

        self.ahead = leader_query.position.0 + dv;
        dv *= -1.0;
        self.behind = leader_query.position.0 + dv;

        self.behind
    }

    pub fn get_leader_ahead(&self, leader_query: &SteeringHostQuery) -> Vec2 {
        let dv = leader_query.host.velocity.normalize_or_zero() * self.leader_behind_dist;
        leader_query.position.0 + dv
    }

    pub fn is_on_leader_sight(
        &self,
        query: &SteeringHostQuery,
        leader_query: &SteeringHostQuery,
    ) -> bool {
        (self.ahead - query.position.0).length() <= self.leader_sight_radius
            || (leader_query.position.0 - query.position.0).length() <= self.leader_sight_radius
    }
}
