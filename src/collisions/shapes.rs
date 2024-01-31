use bevy::{math::Vec2, reflect::Reflect};

use super::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub enum ColliderShapeType {
    Circle { radius: f32 },
    Box { width: f32, height: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Reflect)]
pub struct ColliderShape {
    pub shape_type: ColliderShapeType,
    pub(crate) position: Vec2,
    pub(crate) center: Vec2,
    pub(crate) bounds: Rect,
}

pub mod collisions {
    use super::{ColliderShape, ColliderShapeType};
    use crate::collisions::{CollisionResultRef, RaycastHit, Rect};
    use bevy::math::Vec2;

    pub fn box_to_box<'a>(
        first: &ColliderShape,
        second: &ColliderShape,
        first_movement: Option<Vec2>,
        second_movement: Option<Vec2>,
    ) -> Option<CollisionResultRef<'a>> {
        let mut res = CollisionResultRef::default();

        let diff = minkowski_diff(first, second, first_movement, second_movement);
        if diff.contains(Vec2::ZERO) {
            res.min_translation = diff.closest_point_to_origin();

            if res.min_translation == Vec2::ZERO {
                return None;
            }

            let normal = -res.min_translation;
            res.normal = normal.normalize_or_zero();

            return Some(res);
        }

        None
    }

    pub fn circle_to_circle<'a>(
        first: &ColliderShape,
        second: &ColliderShape,
        first_movement: Option<Vec2>,
        second_movement: Option<Vec2>,
    ) -> Option<CollisionResultRef<'a>> {
        match first.shape_type {
            ColliderShapeType::Circle { radius: r1 } => match second.shape_type {
                ColliderShapeType::Circle { radius: r2 } => {
                    let mut res = CollisionResultRef::default();

                    let first_pos = if let Some(dt) = first_movement {
                        first.position + dt
                    } else {
                        first.position
                    };
                    let second_pos = if let Some(dt) = second_movement {
                        second.position + dt
                    } else {
                        second.position
                    };

                    let dist_sqr = Vec2::distance_squared(first_pos, second_pos);
                    let sum_of_radii = r1 + r2;
                    let collided = dist_sqr < sum_of_radii * sum_of_radii;
                    if collided {
                        let normal = first_pos - second_pos;
                        res.normal = normal.normalize_or_zero();
                        let depth = sum_of_radii - dist_sqr.sqrt();
                        res.min_translation = -depth * res.normal;
                        res.point = second_pos + res.normal * r2;

                        return Some(res);
                    }

                    None
                }
                ColliderShapeType::Box { .. } => panic!("second: expected circle, bot box"),
            },
            ColliderShapeType::Box { .. } => panic!("first: expected circle, got box"),
        }
    }

    pub fn circle_to_box<'a>(
        circle: &ColliderShape,
        bx: &ColliderShape,
        circle_movement: Option<Vec2>,
        bx_movement: Option<Vec2>,
    ) -> Option<CollisionResultRef<'a>> {
        match circle.shape_type {
            ColliderShapeType::Circle { radius } => {
                let mut res = CollisionResultRef::default();

                let circle_pos = if let Some(dt) = circle_movement {
                    circle.position + dt
                } else {
                    circle.position
                };

                let bx_pos = if let Some(dt) = bx_movement {
                    bx.position + dt
                } else {
                    bx.position
                };

                let mut bx_bounds = bx.bounds;
                bx_bounds.x = bx_pos.x;
                bx_bounds.y = bx_pos.y;

                let (closest_point, normal) = bx_bounds.closest_point_on_border(circle_pos);
                res.normal = normal;

                if bx_bounds.contains(circle_pos) {
                    res.point = closest_point;
                    let safe_place = closest_point + res.normal * radius;
                    res.min_translation = circle_pos - safe_place;

                    return Some(res);
                }

                let sqr_dist = closest_point.distance_squared(circle_pos);
                if sqr_dist == 0.0 {
                    res.min_translation = res.normal * radius;
                } else if sqr_dist <= radius * radius {
                    res.normal = circle_pos - closest_point;
                    let depth = res.normal.length() - radius;

                    res.point = closest_point;
                    let normal = res.normal.normalize_or_zero();
                    res.normal = normal;
                    res.min_translation = depth * normal;

                    return Some(res);
                }

                None
            }
            ColliderShapeType::Box { .. } => panic!("circle: expected circle, got box"),
        }
    }

    pub fn line_to_circle<'a>(start: Vec2, end: Vec2, s: &ColliderShape) -> Option<RaycastHit> {
        match s.shape_type {
            ColliderShapeType::Circle { radius } => {
                let mut hit = RaycastHit::default();

                let length = start.distance(end);
                let d = (end - start) / length;
                let m = start - s.position;
                let b = m.dot(d);
                let c = m.dot(m) - radius * radius;

                if c > 0.0 && b > 0.0 {
                    return None;
                }

                let discr = b * b - c;

                if discr < 0.0 {
                    return None;
                }

                hit.fraction = -b - discr.sqrt();

                if hit.fraction < 0.0 {
                    hit.fraction = 0.0;
                }

                hit.point = start + hit.fraction * d;
                let dist = start.distance(hit.point);
                hit.distance = dist;
                hit.normal = (hit.point - s.position).normalize_or_zero();
                hit.fraction = hit.distance / length;

                Some(hit)
            }
            ColliderShapeType::Box { .. } => panic!("s: expected circle, got box"),
        }
    }

    fn minkowski_diff(
        first: &ColliderShape,
        second: &ColliderShape,
        first_movement: Option<Vec2>,
        second_movement: Option<Vec2>,
    ) -> Rect {
        let ((pos1, b1), (_, b2)) =
            super::collisions::parse_movements(first, second, first_movement, second_movement);

        let pos_offset = pos1 - (b1 + first.bounds.size() / 2.0);
        let top_left =
            b1 + pos_offset - Vec2::new(b2.x + second.bounds.width, b2.y + second.bounds.height);
        let full_size = first.bounds.size() + second.bounds.size();

        Rect::new(top_left.x, top_left.y, full_size.x, full_size.y)
    }

    fn parse_movements(
        first: &ColliderShape,
        second: &ColliderShape,
        first_movement: Option<Vec2>,
        second_movement: Option<Vec2>,
    ) -> ((Vec2, Vec2), (Vec2, Vec2)) {
        let first_pos = if let Some(dt) = first_movement {
            (first.position + dt, first.bounds.location() + dt)
        } else {
            (first.position, first.bounds.location())
        };

        let second_pos = if let Some(dt) = second_movement {
            (second.position + dt, second.bounds.location() + dt)
        } else {
            (second.position, second.bounds.location())
        };

        (first_pos, second_pos)
    }
}
