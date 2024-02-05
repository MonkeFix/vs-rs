use bevy::prelude::*;

use crate::collisions::{
    plugin::ColliderComponent,
    store::{ColliderIdResolver, ColliderStore},
    ColliderId, CollisionResult,
};
use crate::movement::{PhysicsParams, Position, SteeringHost};

use super::SteeringHostQuery;

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

impl<'w> SteeringTarget for SteeringHostQuery<'w> {
    fn position(&self) -> Vec2 {
        self.position.0
    }
    fn velocity(&self) -> Vec2 {
        self.host.velocity
    }
}

pub fn steer_seek(
    position: &Position,
    host: &SteeringHost,
    physics_params: &PhysicsParams,
    target: Vec2,
) -> Vec2 {
    let dv = target - position.0;
    let dv = dv.normalize_or_zero();

    dv * physics_params.max_velocity - host.velocity
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

// Useful for debugging
/*fn host_added(mut gizmos: Gizmos, query: Query<(&SteeringHost)>) {
    for (host) in &query {
        gizmos.circle_2d(host.position, 16.0, Color::BLUE);
        // gizmos.ray_2d(host.position, host.cur_velocity.normalize_or_zero() * 100.0, Color::RED);
        // gizmos.ray_2d(host.position, host.desired_velocity.normalize_or_zero() * 1000.0, Color::BLUE);
        // gizmos.ray_2d(host.position, host.steering.normalize_or_zero() * 80.0, Color::YELLOW);
    }
}*/

fn update_translation(mut host: Query<(&mut Transform, &Position)>) {
    for (mut transform, pos) in &mut host {
        transform.translation = pos.0.extend(1.0);
    }
}

fn steer(mut host: Query<(&mut SteeringHost, &PhysicsParams)>) {
    for (mut host, params) in &mut host {
        host.steering = crate::math::truncate_vec2(host.steering, params.max_force);
        host.steering /= params.mass;

        let steering = host.steering;
        host.velocity = crate::math::truncate_vec2(host.velocity + steering, params.max_velocity);
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
