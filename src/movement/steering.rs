use bevy::prelude::*;

use crate::collisions::{
    colliders::Collider,
    plugin::ColliderComponent,
    store::{ColliderIdResolver, ColliderStore},
    ColliderId, CollisionResult, CollisionResultRef,
};

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
            steering_vec: dv * host.max_velocity - host.cur_velocity,
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
        let dv = dv.normalize_or_zero() * host.max_velocity;

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
        let updates_ahead = distance / host.max_velocity;
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
            host.max_velocity * (distance / self.slowing_radius)
        } else {
            host.max_velocity
        };

        SteerResult {
            steering_vec: dv * steering,
            desired_velocity: dv,
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct SteeringObstaclceCircle {
    pub center: Vec2,
    pub radius: f32,
}

#[derive(Debug)]
pub struct SteerCollisionAvoidance {
    pub max_see_ahead: f32,
    pub avoid_force: f32,
    pub obstacle_list: Vec<SteeringObstaclceCircle>,
    ahead: Vec2,
    avoidance: Vec2,
}

impl Default for SteerCollisionAvoidance {
    fn default() -> Self {
        Self {
            max_see_ahead: 16.0,
            avoid_force: 150.0,
            obstacle_list: Vec::default(),
            ahead: Vec2::default(),
            avoidance: Vec2::default(),
        }
    }
}

impl SteerCollisionAvoidance {
    pub fn new(obstacles: Vec<SteeringObstaclceCircle>) -> Self {
        Self {
            obstacle_list: obstacles,
            ..default()
        }
    }

    fn most_threatening(&self, pos: Vec2, ahead: Vec2) -> Option<SteeringObstaclceCircle> {
        let mut res = None;

        for obs in &self.obstacle_list {
            let collides =
                crate::collisions::circle_to_line(obs.center, obs.radius, pos, pos + ahead);

            if collides {
                match res {
                    None => res = Some(*obs),
                    Some(other) => {
                        if (obs.center - pos).length() < (other.center - pos).length() {
                            res = Some(*obs);
                        }
                    }
                }
            }
        }

        res
    }
}

impl SteeringBehavior for SteerCollisionAvoidance {
    fn steer(&mut self, host: &SteeringHost, _target: &impl SteeringTarget) -> SteerResult {
        let dv = host.cur_velocity.normalize_or_zero();
        let dv = dv * self.max_see_ahead * host.cur_velocity.length() / host.max_velocity;

        self.ahead = host.position + dv;

        if let Some(obs) = self.most_threatening(host.position, self.ahead) {
            let avoidance = self.ahead - obs.center;
            self.avoidance = avoidance.normalize_or_zero();
            self.avoidance *= self.avoid_force;
        }

        SteerResult {
            desired_velocity: dv,
            steering_vec: self.avoidance,
        }
    }

    fn is_additive(&self) -> bool {
        true
    }
}

/// Returns ahead vector
pub fn avoid_collisions(
    collider_store: &ColliderStore,
    collider: &Collider,
    host: &SteeringHost,
    max_see_ahead: f32,
    avoid_force: f32,
    avoidance: &mut Vec2,
    layer_mask: Option<i32>,
) -> Vec2 {
    let dv = host.cur_velocity.normalize_or_zero() * max_see_ahead * host.cur_velocity.length()
        / host.max_velocity;

    let ahead = host.position + dv;

    let mut bounds = collider.bounds();
    bounds.x += dv.x;
    bounds.y += dv.y;

    let mut min_dist = f32::MAX;
    let mut closest_col = None;

    let neighbors = collider_store.aabb_broadphase_excluding_self(collider.id, bounds, layer_mask);
    // find the closest
    /* if neighbors.len() > 0 {
        bevy::log::info!("found {} neighbors", neighbors.len());
    } */
    for neighbor_id in neighbors {
        let neighbor = collider_store.get(neighbor_id).unwrap();
        let distance = (neighbor.position() - host.position).length();

        if distance < min_dist {
            min_dist = distance;
            closest_col = Some(neighbor);
        }
    }

    if let Some(collider) = closest_col {
        *avoidance = (ahead - collider.position()).normalize_or_zero();
        *avoidance *= avoid_force;
    } else {
        *avoidance *= 0.0;
    }

    ahead
}

#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect)]
#[reflect(Component, Default, PartialEq)]
pub struct SteeringHost {
    pub position: Vec2,

    pub desired_velocity: Vec2,
    pub cur_velocity: Vec2,
    pub steering: Vec2,

    /// The highest speed entity can get to.
    pub max_velocity: f32,
    pub max_force: f32,
    pub mass: f32,
    pub friction: f32,
}

impl Default for SteeringHost {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,

            desired_velocity: Vec2::ZERO,
            cur_velocity: Vec2::ZERO,
            steering: Vec2::ZERO,

            max_velocity: 250.0,
            max_force: 150.0,
            mass: 4.0,
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
        app.add_systems(Update, (host_added,));
        app.add_systems(FixedUpdate, (steer, update_translation));
    }
}

fn host_added(mut gizmos: Gizmos, query: Query<(&SteeringHost)>) {
    for (host) in &query {
        gizmos.circle_2d(host.position, 16.0, Color::BLUE);
        // gizmos.ray_2d(host.position, host.cur_velocity.normalize_or_zero() * 100.0, Color::RED);
        // gizmos.ray_2d(host.position, host.desired_velocity.normalize_or_zero() * 1000.0, Color::BLUE);
        // gizmos.ray_2d(host.position, host.steering.normalize_or_zero() * 80.0, Color::YELLOW);
    }
}

fn update_translation(mut host: Query<(&mut Transform, &SteeringHost)>) {
    for (mut transform, host) in &mut host {
        transform.translation = host.position.extend(1.0);
    }
}

fn steer(
    collider_store: Res<ColliderStore>,
    time: Res<Time>,
    mut host: Query<(&mut SteeringHost, &ColliderComponent)>,
) {
    for (mut host, collider_id) in &mut host {
        let mass = host.mass;

        host.steering = crate::math::truncate_vec2(host.steering, host.max_force);
        host.steering /= mass;

        let steering = host.steering;
        host.cur_velocity =
            crate::math::truncate_vec2(host.cur_velocity + steering, host.max_velocity);

        let mut movement = host.cur_velocity * time.delta_seconds();
        calc_movement(&mut movement, collider_id.id, &collider_store);

        host.position += movement;

        let friction = host.friction;
        host.cur_velocity *= friction;
    }
}

fn calc_movement(
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

        if let Some(collision) = collider.collides_with_motion(&neighbor, *motion) {
            *motion -= collision.min_translation;

            result = Some(CollisionResult::from_ref(&collision));
        }
    }

    result
}
