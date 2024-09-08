use bevy::prelude::Component;

#[derive(Component, Clone, Debug)]
pub struct Health(pub i64);

#[derive(Component, Clone, Debug)]
pub struct MaxHealth(pub i64);

#[derive(Component, Clone, Debug)]
pub struct Damage(pub i64);

#[derive(Component, Clone, Debug)]
pub struct Experience(pub u32);

#[derive(Component, Clone, Debug)]
pub struct Gold(pub u32);

#[derive(Component, Clone, Debug)]
pub struct ExperienceDrop(pub u32);
