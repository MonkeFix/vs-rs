use crate::{
    prelude::*, CollideEvent, InvokeTriggerEvent, MovementCalculateEvent, PositionUpdateEvent,
};
use behaviors::{
    steer_collision_avoidance, steer_entity, steer_vec2, SteerArrival, SteerFlee, SteerSeek,
};
use bevy::{color::palettes::css::RED, prelude::*};
use colliders::Collider;
use common::math::{is_flag_set, truncate_vec2};
use spatial_hash::SpatialHash;
use steering::*;

#[derive(Component, Default)]
pub struct RigidBodyStatic;

/// The main plugin. Required for collisions and movement to work.
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialHash::new(40))
            .add_event::<MovementCalculateEvent>()
            .add_event::<PositionUpdateEvent>()
            .add_event::<InvokeTriggerEvent>()
            .add_systems(
                Update,
                (
                    steer_entity::<SteerSeek>,
                    steer_vec2::<SteerSeek>,
                    steer_entity::<SteerArrival>,
                    steer_vec2::<SteerArrival>,
                    steer_collision_avoidance,
                ),
            )
            .add_systems(FixedUpdate, (steer, calc_movement, update_position).chain())
            .observe(on_collider_added)
            .observe(on_collider_removed);
    }
}

fn debug_draw(
    mut gizmos: Gizmos,
    spatial_hash: Res<SpatialHash>,
    query: Query<&Collider, With<Transform>>,
) {
    let entities = spatial_hash.get_all();
    for entity in entities {
        let collider = query.get(entity);
        if let Ok(collider) = collider {
            match collider.shape.shape_type {
                shapes::ShapeType::None => {}
                shapes::ShapeType::Circle { radius } => {
                    gizmos.circle_2d(collider.absolute_position(), radius, RED);
                }
                shapes::ShapeType::Box { width, height } => gizmos.rect_2d(
                    collider.absolute_position(),
                    0.0,
                    Vec2::new(width, height),
                    RED,
                ),
            }
        }
    }
}

fn on_collider_added(
    trigger: Trigger<OnAdd, Collider>,
    mut spatial_hash: ResMut<SpatialHash>,
    mut query: Query<(&mut Collider, &Transform)>,
) {
    let mut collider = query.get_mut(trigger.entity()).unwrap();
    collider.0.update_from_transform(collider.1);
    spatial_hash.register(&collider.0, trigger.entity());
}

fn on_collider_removed(
    trigger: Trigger<OnRemove, Collider>,
    mut hash: ResMut<SpatialHash>,
    query: Query<&Collider, With<Transform>>,
) {
    let collider = query.get(trigger.entity()).unwrap();
    hash.remove(collider, trigger.entity());
}

fn steer(
    mut host: Query<(&mut SteeringHost, &PhysicalParams, Entity)>,
    time: Res<Time>,
    mut evt_movement_calc: EventWriter<MovementCalculateEvent>,
) {
    for (mut host, params, entity) in &mut host {
        host.steering = truncate_vec2(host.steering, params.max_force);
        host.steering /= params.mass;

        let steering = host.steering;
        host.velocity = truncate_vec2(host.velocity + steering, params.max_velocity);

        host.movement = host.velocity * time.delta_seconds();
        let evt = MovementCalculateEvent {
            entity,
            movement: host.movement,
        };
        evt_movement_calc.send(evt);

        host.velocity *= params.friction;
    }
}

fn calc_movement(
    mut evt_movement_calc: EventReader<MovementCalculateEvent>,
    mut evt_pos_update: EventWriter<PositionUpdateEvent>,
    mut evt_invoke_trigger: EventWriter<InvokeTriggerEvent>,
    mut commands: Commands,
    spatial_hash: Res<SpatialHash>,
    hosts: Query<&SteeringHost>,
    colliders: Query<&Collider>,
) {
    info_span!("calc_movement", name = "calc_movement");
    for evt in evt_movement_calc.read() {
        let host = hosts.get(evt.entity);
        let collider = colliders.get(evt.entity);

        let mut motion = evt.movement;

        let mut process_collider = |collider: &Collider| {
            // Host has a collider, calculating correct movement
            if collider.is_trigger {
                // Skipping the trigger
                send_pos_update(&mut evt_pos_update, evt);
                return;
            }

            let mut bounds = collider.bounds();
            bounds.x += evt.movement.x;
            bounds.y += evt.movement.y;

            let neighbors = spatial_hash.aabb_broadphase(
                &colliders,
                bounds,
                Some(evt.entity),
                None,
                //Some(collider.collides_with_layers),
            );

            for neighbor_entity in neighbors {
                let neighbor = colliders.get(neighbor_entity).ok().unwrap();

                if let Some(collision) = collider.collides_with_motion(neighbor, motion) {
                    if !neighbor.is_trigger {
                        if is_flag_set(collider.collides_with_layers, neighbor.physics_layer) {
                            motion -= collision.min_translation;
                        }
                        commands.trigger(CollideEvent {
                            entity_main: evt.entity,
                            collided_with: neighbor_entity,
                        });
                        commands.trigger_targets(
                            CollideEvent {
                                entity_main: evt.entity,
                                collided_with: neighbor_entity,
                            },
                            evt.entity,
                        );
                    } else {
                        evt_invoke_trigger.send(InvokeTriggerEvent {
                            entity_main: evt.entity,
                            entity_trigger: neighbor_entity,
                        });
                    }
                }
            }
        };

        if let Ok(_host) = host {
            match collider {
                Ok(collider) => {
                    process_collider(collider);
                }
                Err(_) => {
                    // Host has no colliders, just sending the event further
                    send_pos_update(&mut evt_pos_update, evt);
                    continue;
                }
            };

            evt_pos_update.send(PositionUpdateEvent {
                entity: evt.entity,
                movement: motion,
            });
        }
    }
}

fn update_position(
    mut evt_pos_update: EventReader<PositionUpdateEvent>,
    mut host: Query<&mut Transform, With<SteeringHost>>,
    mut spatial_hash: ResMut<SpatialHash>,
    mut colliders: Query<&mut Collider>,
) {
    for ev in evt_pos_update.read() {
        if let Ok(mut transform) = host.get_mut(ev.entity) {
            transform.translation.x += ev.movement.x;
            transform.translation.y += ev.movement.y;

            if ev.movement != Vec2::ZERO {
                let collider = colliders.get_mut(ev.entity);
                if let Ok(mut collider) = collider {
                    info_span!("update_position_hash", name = "update_position_hash");
                    spatial_hash.remove(&collider, ev.entity);
                    collider.update_from_transform(&transform);
                    spatial_hash.register(&collider, ev.entity);
                }
            }
        }
    }
}

fn send_pos_update(ew: &mut EventWriter<PositionUpdateEvent>, evt: &MovementCalculateEvent) {
    ew.send(PositionUpdateEvent {
        entity: evt.entity,
        movement: evt.movement,
    });
}
