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
    use crate::collisions::{CollisionResultRef, RaycastHitRef, Rect};
    use bevy::math::Vec2;

    pub fn box_to_box<'a>(first: &ColliderShape, second: &ColliderShape) -> Option<CollisionResultRef<'a>> {
        let mut res = CollisionResultRef::default();

        let diff = minkowski_diff(first, second);
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
    ) -> Option<CollisionResultRef<'a>> {
        match first.shape_type {
            ColliderShapeType::Circle { radius: r1 } => match second.shape_type {
                ColliderShapeType::Circle { radius: r2 } => {
                    let mut res = CollisionResultRef::default();

                    let dist_sqr = Vec2::distance_squared(first.position, second.position);
                    let sum_of_radii = r1 + r2;
                    let collided = dist_sqr < sum_of_radii * sum_of_radii;
                    if collided {
                        let normal = first.position - second.position;
                        res.normal = normal.normalize_or_zero();
                        let depth = sum_of_radii - dist_sqr.sqrt();
                        res.min_translation = -depth * res.normal;
                        res.point = second.position + res.normal * r2;

                        return Some(res);
                    }

                    None
                }
                ColliderShapeType::Box { .. } => panic!("second: expected circle, bot box"),
            },
            ColliderShapeType::Box { .. } => panic!("first: expected circle, got box"),
        }
    }

    pub fn circle_to_box<'a>(circle: &ColliderShape, bx: &ColliderShape) -> Option<CollisionResultRef<'a>> {
        match circle.shape_type {
            ColliderShapeType::Circle { radius } => {
                let mut res = CollisionResultRef::default();

                let (closest_point, normal) = bx.bounds.closest_point_on_border(circle.position);
                res.normal = normal;

                if bx.bounds.contains(circle.position) {
                    res.point = closest_point;
                    let safe_place = closest_point + res.normal * radius;
                    res.min_translation = circle.position - safe_place;

                    return Some(res);
                }

                let sqr_dist = closest_point.distance_squared(circle.position);
                if sqr_dist == 0.0 {
                    res.min_translation = res.normal * radius;
                } else if sqr_dist <= radius * radius {
                    res.normal = circle.position - closest_point;
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

    pub fn line_to_circle<'a>(start: Vec2, end: Vec2, s: &ColliderShape) -> Option<RaycastHitRef<'a>> {
        match s.shape_type {
            ColliderShapeType::Circle { radius } => {
                let mut hit = RaycastHitRef::default();

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

    fn minkowski_diff(first: &ColliderShape, second: &ColliderShape) -> Rect {
        let pos_offset = first.position - (first.bounds.location() + first.bounds.size() / 2.0);
        let top_left = first.bounds.location() + pos_offset - second.bounds.max();
        let full_size = first.bounds.size() + second.bounds.size();

        Rect::new(top_left.x, top_left.y, full_size.x, full_size.y)
    }
}
