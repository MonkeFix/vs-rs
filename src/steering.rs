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

impl SteeringHost {
    pub fn seek(&mut self, target: Vec2) {
        let dv = target - self.position;
        let dv = dv.normalize_or_zero();

        self.desired_velocity = dv;
        let steering = self.desired_velocity * self.max_velocity - self.cur_velocity;
        self.steering = steering;
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
        let friction = host.friction;

        host.cur_velocity *= friction;

        let dt = host.cur_velocity * time.delta_seconds();
        host.position += dt;
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
