use bevy::{log, prelude::*};

#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect)]
#[reflect(Component, Default, PartialEq)]
pub struct SteeringHost {
    pub position: Vec2,

    pub desired_velocity: Vec2,
    pub cur_velocity: Vec2,
    pub steering: Vec2,

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

pub fn seek(host: &mut SteeringHost, target: Vec2) {
    let dv = target - host.position;
    let dv = dv.normalize_or_zero();

    host.desired_velocity = dv;
    let steering = host.desired_velocity * host.max_velocity - host.cur_velocity;
    host.steering = steering;
}

#[derive(Bundle)]
pub struct SteeringBundle {
    pub host: SteeringHost,
}

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (steer, update));
    }
}

fn update(time: Res<Time>, mut host: Query<(&mut Transform, &mut SteeringHost)>) {
    if let Ok((mut transform, mut host)) = host.get_single_mut() {
        // TODO: Calculate collisions here

        transform.translation.x += host.cur_velocity.x * time.delta_seconds();
        transform.translation.y += host.cur_velocity.y * time.delta_seconds();

        host.position = Vec2::new(transform.translation.x, transform.translation.y);

        let friction = host.friction;

        host.cur_velocity *= friction;
    }
}

fn steer(mut host: Query<&mut SteeringHost>) {
    if let Ok(mut host) = host.get_single_mut() {
        let mass = host.mass;

        host.steering = crate::math::truncate(host.steering, host.max_force);
        host.steering /= mass;

        let steering = host.steering;
        host.cur_velocity = crate::math::truncate(host.cur_velocity + steering, host.max_velocity);
    }
}
