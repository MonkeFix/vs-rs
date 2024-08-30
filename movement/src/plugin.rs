use bevy::prelude::*;
use collisions::{prelude::*, ColliderId};
use common::{math::truncate_vec2, Position};

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

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (steer,));
        app.add_systems(
            FixedUpdate,
            (update_positions, apply_friction, update_translation).chain(),
        );
    }
}

fn update_translation(mut host: Query<(&mut Transform, &Position)>) {
    for (mut transform, pos) in &mut host {
        transform.translation = pos.0.extend(1.0);
    }
}

fn steer(mut host: Query<(&mut SteeringHost, &PhysicsParams)>) {
    for (mut host, params) in &mut host {
        host.steering = truncate_vec2(host.steering, params.max_force);
        host.steering /= params.mass;

        let steering = host.steering;
        host.velocity = truncate_vec2(host.velocity + steering, params.max_velocity);
    }
}

fn update_positions(
    collider_store: Res<ColliderStore>,
    time: Res<Time>,
    mut host: Query<(&SteeringHost, &ColliderComponent, &mut Position)>,
) {
    for (host, collider_id, mut position) in &mut host {
        let mut movement = host.velocity * time.delta_seconds();
        calc_movement(&mut movement, collider_id.id, &collider_store);

        position.0 += movement;
    }
}

fn apply_friction(mut host: Query<(&mut SteeringHost, &PhysicsParams)>) {
    for (mut host, params) in &mut host {
        host.velocity *= params.friction;
    }
}

fn calc_movement(
    motion: &mut Vec2,
    collider_id: ColliderId,
    collider_store: &ColliderStore,
) -> Option<CollisionResult> {
    let mut result = None;

    let collider = collider_store.get(collider_id).unwrap();

    if collider.data.is_trigger {
        return None;
    }

    let mut bounds = collider.bounds();

    bounds.x += motion.x;
    bounds.y += motion.y;

    let neighbors = collider_store.aabb_broadphase_excluding_self(
        collider_id,
        bounds,
        Some(collider.data.collides_with_layers),
    );

    for id in neighbors {
        let neighbor = collider_store.get(id).unwrap();
        if neighbor.data.is_trigger {
            continue;
        }

        if let Some(collision) = collider.collides_with_motion(neighbor, *motion) {
            *motion -= collision.min_translation;

            result = Some(CollisionResult::from_ref(&collision));
        }
    }

    result
}
