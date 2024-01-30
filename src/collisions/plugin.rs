use super::{shapes::ColliderShapeType, store::ColliderStore, ColliderId};
use bevy::{ecs::system::EntityCommands, prelude::*};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect, Hash)]
pub struct ColliderComponent {
    pub id: ColliderId,
}

impl ColliderComponent {
    pub fn new(collider_set: &mut ColliderStore, shape_type: ColliderShapeType) -> Self {
        collider_set.create_and_register(shape_type)
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
            .add_systems(FixedUpdate, (update_positions, on_collider_added));
    }
}

fn update_positions(
    mut collider_set: ResMut<ColliderStore>,
    colliders: Query<(&ColliderComponent, &Transform), Changed<Transform>>,
) {
    for (collider, transform) in &colliders {
        collider_set.update_single(collider.id, transform);
    }
}

fn on_collider_added(
    mut collider_set: ResMut<ColliderStore>,
    colliders: Query<(&ColliderComponent, &Transform), Added<ColliderComponent>>,
) {
    for (col, transform) in &colliders {
        collider_set.added_with_transform(col.id, transform);
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

impl<'w, 's, 'a> ColliderDespawnable for EntityCommands<'w, 's, 'a> {
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
