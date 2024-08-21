#![allow(dead_code)]

use super::{shapes::ColliderShapeType, store::ColliderStore, ColliderId};
use crate::movement::Position;
use bevy::{ecs::system::EntityCommands, prelude::*};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash)]
pub struct ColliderComponent {
    pub id: ColliderId,
}

impl From<ColliderComponent> for ColliderId {
    fn from(value: ColliderComponent) -> Self {
        value.id
    }
}

impl ColliderComponent {
    pub fn new(collider_set: &mut ColliderStore, shape_type: ColliderShapeType) -> Self {
        collider_set.create_and_register(shape_type, None)
    }
}

#[derive(Bundle)]
pub struct ColliderBundle {
    pub collider: ColliderComponent,
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ColliderStore::default())
            .add_systems(FixedUpdate, (update_positions,));
        app.observe(on_collider_added);
        app.observe(on_collider_removed);
    }
}

fn update_positions(
    mut collider_set: ResMut<ColliderStore>,
    colliders: Query<(&ColliderComponent, &Position), Changed<Position>>,
) {
    for (collider, position) in &colliders {
        collider_set.update_single(collider.id, position);
    }
}

fn on_collider_added(
    trigger: Trigger<OnAdd, ColliderComponent>,
    mut collider_set: ResMut<ColliderStore>,
    colliders: Query<(&ColliderComponent, &Position)>,
) {
    let ent = colliders.get(trigger.entity());
    if let Ok((collider_id, pos)) = ent {
        collider_set.added_with_position(collider_id.id, pos);
    }
}

fn on_collider_removed(
    trigger: Trigger<OnRemove, ColliderComponent>,
    mut collider_set: ResMut<ColliderStore>,
    query: Query<&ColliderComponent>,
) {
    let ent = query.get(trigger.entity());
    if let Ok(collider_id) = ent {
        collider_set.remove(collider_id.id);
    }
}

pub trait ColliderDespawnable {
    fn despawn_and_unregister(
        &mut self,
        collider_store: &mut ColliderStore,
        collider_id: ColliderId,
    );
    fn despawn_recursive_and_unregister(
        self,
        collider_store: &mut ColliderStore,
        collider_id: ColliderId,
    );
}

impl<'a> ColliderDespawnable for EntityCommands<'a> {
    fn despawn_and_unregister(
        &mut self,
        collider_store: &mut ColliderStore,
        collider_id: ColliderId,
    ) {
        self.despawn();
        collider_store.remove(collider_id);
    }

    fn despawn_recursive_and_unregister(
        self,
        collider_store: &mut ColliderStore,
        collider_id: ColliderId,
    ) {
        collider_store.remove(collider_id);
        self.despawn_recursive();
    }
}
