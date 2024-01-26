use bevy::prelude::{Component};
use bevy::reflect::Reflect;

#[derive(Component, Clone, Debug)]
pub struct Health(pub i64);

#[derive(Component, Clone, Debug)]
pub struct Damage(pub i64);

#[derive(Component, Clone, Debug, Reflect, Copy, PartialEq)]
pub struct MaxForce(pub f32);
#[derive(Component, Clone, Debug, Reflect, Copy, PartialEq)]
pub struct MaxVelocity(pub f32);
#[derive(Component, Clone, Debug, Reflect, Copy, PartialEq)]
pub struct Mass(pub f32);
