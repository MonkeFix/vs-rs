use bevy::math::Vec2;

pub mod behaviors;
pub mod paths;
pub mod plugin;
pub mod prelude;

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
