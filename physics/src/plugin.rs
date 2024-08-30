use crate::{prelude::*, MovementCalculateEvent, PositionUpdateEvent};
use bevy::{color::palettes::css::RED, prelude::*};
use colliders::Collider;
use common::math::{is_flag_set, truncate_vec2};
use spatial_hash::SpatialHash;
use steering::*;

// Order of actions:
// 1) SteeringHost::steer() call
// 2) steering::steer() system which updates SteeringHost's fields (velocity and steering)
// 3) steering::update_positions() system which calls calc_movement() function

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialHash::new(40))
            .add_systems(Update, debug_draw)
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
                    gizmos.circle_2d(collider.shape.position, radius, RED);
                }
                shapes::ShapeType::Box { width, height } => {
                    gizmos.rect_2d(collider.shape.position, 0.0, Vec2::new(width, height), RED)
                }
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

        evt_movement_calc.send(MovementCalculateEvent {
            entity,
            movement: host.movement,
        });

        host.velocity *= params.friction;
    }
}

fn calc_movement(
    mut evt_movement_calc: EventReader<MovementCalculateEvent>,
    mut evt_pos_update: EventWriter<PositionUpdateEvent>,
    spatial_hash: Res<SpatialHash>,
    hosts: Query<&SteeringHost>,
    colliders: Query<&Collider>,
) {
    for evt in evt_movement_calc.read() {
        let host = hosts.get(evt.entity);
        let collider = colliders.get(evt.entity);

        let mut motion = evt.movement;

        if let Ok(_host) = host {
            match collider {
                Ok(collider) => {
                    // Host has a collider, calculating correct movement
                    if collider.is_trigger {
                        // TODO: Invoke trigger
                        send_pos_update(&mut evt_pos_update, evt);
                        continue;
                    }

                    let mut bounds = collider.bounds();
                    bounds.x += evt.movement.x;
                    bounds.y += evt.movement.y;

                    let neighbors = spatial_hash.get_nearby_bounds(bounds);
                    for entity in neighbors {
                        // Skip self
                        if entity == evt.entity {
                            continue;
                        }
                        let neighbor = colliders.get(entity).ok().unwrap();

                        // Skip if collider doesn't collide with neighbor's physics layer
                        if !is_flag_set(collider.collides_with_layers, neighbor.physics_layer) {
                            continue;
                        }
                        if !bounds.intersects(collider.bounds()) {
                            continue;
                        }

                        if let Some(collision) =
                            collider.collides_with_motion(neighbor, evt.movement)
                        {
                            if !neighbor.is_trigger {
                                motion -= collision.min_translation;
                            } else {
                                // TODO: Invoke trigger
                            }
                        }
                    }
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
        let mut transform = host.get_mut(ev.entity).unwrap();
        transform.translation.x += ev.movement.x;
        transform.translation.y += ev.movement.y;

        let collider = colliders.get_mut(ev.entity);
        if let Ok(mut collider) = collider {
            spatial_hash.remove(&collider, ev.entity);
            collider.update_from_transform(&transform);
            spatial_hash.register(&collider, ev.entity);
        }
    }
}

fn send_pos_update(ew: &mut EventWriter<PositionUpdateEvent>, evt: &MovementCalculateEvent) {
    ew.send(PositionUpdateEvent {
        entity: evt.entity,
        movement: evt.movement,
    });
}
