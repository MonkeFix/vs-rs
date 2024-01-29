use bevy::prelude::*;

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
        app.add_systems(Update, (steer, update_position, update_translation).chain());
    }
}

fn update_translation(mut host: Query<(&mut Transform, &SteeringHost)>) {
    for (mut transform, host) in &mut host {
        transform.translation = host.position.extend(1.0);
    }
}

fn update_position(time: Res<Time>, mut host: Query<(&mut SteeringHost, Entity)>) {
    for (mut host, _entity) in &mut host {
        let movement = host.cur_velocity * time.delta_seconds();
        //calc_movement(&mut movement, &mut colliders, entity, &collider_set);

        host.position += movement;

        let friction = host.friction;
        host.cur_velocity *= friction;
    }
}

fn steer(mut host: Query<&mut SteeringHost>) {
    for mut host in &mut host {
        let mass = host.mass;

        host.steering = crate::math::truncate_vec2(host.steering, host.max_force);
        host.steering /= mass;

        let steering = host.steering;
        host.cur_velocity =
            crate::math::truncate_vec2(host.cur_velocity + steering, host.max_velocity);
    }
}
/*
fn calc_movement(
    motion: &mut Vec2,
    colliders: &mut Query<&mut Collider>,
    entity: Entity,
    collider_set: &Res<ColliderSet>,
) -> Option<CollisionResult> {
    let mut res = None;

    for mut collider in colliders {
        let mut bounds = collider.shape.bounds;
        bounds.x += motion.x;
        bounds.y += motion.y;
        let neighbors = get_neighbors(entity, &collider_set);

        for neighbor in neighbors {
            if let Some(collision) = collider.collides_with_motion(*motion, &neighbor) {
                *motion -= collision.min_translation;
                let col: CollisionResult = collision.clone();
                res = Some(col);
            }
        }
    }

    res
}

fn get_neighbors(entity: Entity, collider_set: &Res<ColliderSet>) -> Vec<Collider> {
    let mut res = vec![];

    for (index, collider) in &collider_set.map {
        if entity.index() == *index {
            continue;
        }

        res.push(collider.clone());
    }

    res
}
 */
