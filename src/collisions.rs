use bevy::{math::Vec2, reflect::Reflect};
use num_enum::FromPrimitive;

use self::colliders::Collider;

pub mod colliders;
pub mod plugin;
pub mod shapes;
pub mod spatial_hash;
pub mod store;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Reflect)]
pub struct ColliderId(pub u32);

#[derive(Debug, Default, Clone, Copy)]
pub struct CollisionResult {
    pub collider: Option<ColliderId>,
    pub normal: Vec2,
    pub min_translation: Vec2,
    pub point: Vec2,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CollisionResultRef<'a> {
    pub collider: Option<&'a Collider>,
    pub normal: Vec2,
    pub min_translation: Vec2,
    pub point: Vec2,
}

impl CollisionResult {
    pub fn invert(&mut self) {
        self.normal.x = -self.normal.x;
        self.normal.y = -self.normal.y;

        self.min_translation.x = -self.min_translation.x;
        self.min_translation.y = -self.min_translation.y;
    }

    pub fn from_ref(result: &CollisionResultRef) -> Self {
        let collider = result.collider.map(|col| col.id);

        Self {
            collider,
            normal: result.normal,
            min_translation: result.min_translation,
            point: result.point,
        }
    }
}

impl<'a> CollisionResultRef<'a> {
    pub fn invert(&mut self) {
        self.normal.x = -self.normal.x;
        self.normal.y = -self.normal.y;

        self.min_translation.x = -self.min_translation.x;
        self.min_translation.y = -self.min_translation.y;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Ray2D {
    pub start: Vec2,
    pub end: Vec2,
    pub direction: Vec2,
}

impl Ray2D {
    pub fn new(position: Vec2, end: Vec2) -> Self {
        Self {
            start: position,
            end,
            direction: end - position,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RaycastHit {
    pub collider: Option<ColliderId>,
    pub fraction: f32,
    pub distance: f32,
    pub point: Vec2,
    pub normal: Vec2,
    pub centroid: Vec2,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum PointSectors {
    #[num_enum(default)]
    Center = 0,
    Top = 1,
    Bottom = 2,
    TopLeft = 9,
    TopRight = 5,
    Left = 8,
    Right = 4,
    BottomLeft = 10,
    BottomRight = 6,
}

pub fn line_to_line(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let b = a2 - a1;
    let d = b2 - b1;
    let d_dot = b.x * d.y - b.y * d.x;

    // the lines are parallel
    if d_dot == 0.0 {
        return false;
    }

    let c = b1 - a1;
    let t = (c.x * d.y - c.y * d.x) / d_dot;
    if !(0.0..=1.0).contains(&t) {
        return false;
    }

    let u = (c.x * b.y - c.y * b.x) / d_dot;
    if !(0.0..=1.0).contains(&u) {
        return false;
    }

    true
}

pub fn closest_point_on_line(line_a: Vec2, line_b: Vec2, closest_to: Vec2) -> Vec2 {
    let v = line_b - line_a;
    let w = closest_to - line_a;
    let t = Vec2::dot(w, v) / Vec2::dot(v, v);
    let t = t.clamp(0.0, 1.0);

    line_a + v * t
}

pub fn circle_to_circle(
    circle_center_1: Vec2,
    circle_radius_1: f32,
    circle_center_2: Vec2,
    circle_radius_2: f32,
) -> bool {
    Vec2::distance_squared(circle_center_1, circle_center_2)
        < (circle_radius_1 + circle_radius_2) * (circle_radius_1 * circle_radius_2)
}

pub fn circle_to_line(circle_center: Vec2, radius: f32, line_from: Vec2, line_to: Vec2) -> bool {
    Vec2::distance_squared(
        circle_center,
        closest_point_on_line(line_from, line_to, circle_center),
    ) < radius * radius
}

pub fn circle_to_point(circle_center: Vec2, radius: f32, point: Vec2) -> bool {
    Vec2::distance_squared(circle_center, point) < radius * radius
}

pub fn rect_to_circle(
    rect_x: f32,
    rect_y: f32,
    rect_w: f32,
    rect_h: f32,
    circle_center: Vec2,
    radius: f32,
) -> bool {
    if rect_to_point(rect_x, rect_y, rect_w, rect_h, circle_center) {
        return true;
    }

    let mut edge_from;
    let mut edge_to;
    let sector = get_sector(rect_x, rect_y, rect_w, rect_h, circle_center) as u8;

    if (sector & PointSectors::Top as u8) != 0 {
        edge_from = Vec2::new(rect_x, rect_y);
        edge_to = Vec2::new(rect_x + rect_w, rect_y);
        if circle_to_line(circle_center, radius, edge_from, edge_to) {
            return true;
        }
    }
    if (sector & PointSectors::Bottom as u8) != 0 {
        edge_from = Vec2::new(rect_x, rect_y + rect_h);
        edge_to = Vec2::new(rect_x + rect_w, rect_y + rect_h);
        if circle_to_line(circle_center, radius, edge_from, edge_to) {
            return true;
        }
    }
    if (sector & PointSectors::Left as u8) != 0 {
        edge_from = Vec2::new(rect_x, rect_y);
        edge_to = Vec2::new(rect_x, rect_y + rect_h);
        if circle_to_line(circle_center, radius, edge_from, edge_to) {
            return true;
        }
    }
    if (sector & PointSectors::Right as u8) != 0 {
        edge_from = Vec2::new(rect_x + rect_w, rect_y);
        edge_to = Vec2::new(rect_x + rect_w, rect_y + rect_h);
        if circle_to_line(circle_center, radius, edge_from, edge_to) {
            return true;
        }
    }

    false
}

pub fn rect_to_point(x: f32, y: f32, w: f32, h: f32, point: Vec2) -> bool {
    point.x >= x && point.y >= y && point.x < x + w && point.y < y + h
}

pub fn rect_to_rect(pos1: Vec2, size1: Vec2, pos2: Vec2, size2: Vec2) -> bool {
    pos1.x + size1.x >= pos2.x
        && pos1.x <= pos2.x + size2.x
        && pos1.y + size1.y >= pos2.y
        && pos1.y <= pos2.y + size2.y
}

pub fn get_sector(x: f32, y: f32, w: f32, h: f32, point: Vec2) -> PointSectors {
    let mut sector = PointSectors::Center as u8;

    if point.x < x {
        sector |= PointSectors::Left as u8;
    } else if point.x >= x + w {
        sector |= PointSectors::Right as u8;
    }

    if point.y < y {
        sector |= PointSectors::Top as u8;
    } else if point.y >= y + h {
        sector |= PointSectors::Bottom as u8;
    }

    PointSectors::from_primitive(sector)
}

/// Describes a 2D-rectangle with {x,y} being the top-left corner of the rectangle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Reflect)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_min_max(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }

    pub fn left(&self) -> f32 {
        self.x
    }

    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn top(&self) -> f32 {
        self.y
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn max(&self) -> Vec2 {
        Vec2::new(self.right(), self.bottom())
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0.0 && self.height == 0.0 && self.x == 0.0 && self.y == 0.0
    }

    pub fn location(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    pub fn contains(&self, point: Vec2) -> bool {
        self.x <= point.x
            && point.x < (self.x + self.width)
            && self.y <= point.y
            && point.y < (self.y + self.height)
    }

    pub fn inflate(&mut self, horizontal: f32, vertical: f32) {
        self.x -= horizontal;
        self.y -= vertical;
        self.width += horizontal * 2.0;
        self.height += vertical * 2.0;
    }

    pub fn intersects(&self, other: Rect) -> bool {
        other.left() < self.right()
            && self.left() < other.right()
            && other.top() < self.bottom()
            && self.top() < other.bottom()
    }

    pub fn union(&self, other: &Rect) -> Rect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);

        Rect::new(
            x,
            y,
            self.right().max(other.right()) - x,
            self.bottom().max(other.bottom()) - y,
        )
    }

    pub fn union_vec2(&self, vec: &Vec2) -> Rect {
        let rect = Rect::new(vec.x, vec.y, 0.0, 0.0);
        self.union(&rect)
    }

    pub fn closest_point_to_origin(&self) -> Vec2 {
        let max = self.max();
        let mut min_dist = self.x.abs();
        let mut bounds_point = Vec2::new(self.x, 0.0);

        if max.x.abs() < min_dist {
            min_dist = max.x.abs();
            bounds_point.x = max.x;
            bounds_point.y = 0.0;
        }

        if max.y.abs() < min_dist {
            min_dist = max.y.abs();
            bounds_point.x = 0.0;
            bounds_point.y = max.y;
        }

        if self.y.abs() < min_dist {
            bounds_point.x = 0.0;
            bounds_point.y = self.y;
        }

        bounds_point
    }

    /// Returns (Closest, EdgeNormal)
    pub fn closest_point_on_border(&self, point: Vec2) -> (Vec2, Vec2) {
        let mut edge_normal = Vec2::ZERO;

        let mut res = Vec2::new(0.0, 0.0);
        res.x = point.x.clamp(self.left(), self.right());
        res.y = point.y.clamp(self.top(), self.bottom());

        if self.contains(res) {
            let dl = res.x - self.left();
            let dr = self.right() - res.x;
            let dt = res.y - self.top();
            let db = self.bottom() - res.y;

            let min = dl.min(dr.min(dt.min(db)));

            if min == dt {
                res.y = self.top();
                edge_normal.y = -1.0;
            } else if min == db {
                res.y = self.bottom();
                edge_normal.y = 1.0;
            } else if min == dl {
                res.x = self.left();
                edge_normal.x = -1.0;
            } else {
                res.x = self.right();
                edge_normal.x = 1.0;
            }

            return (res, edge_normal);
        }

        if res.x == self.left() {
            edge_normal.x = -1.0;
        }
        if res.x == self.right() {
            edge_normal.x = 1.0;
        }
        if res.y == self.top() {
            edge_normal.y = -1.0;
        }
        if res.y == self.bottom() {
            edge_normal.y = 1.0;
        }

        (res, edge_normal)
    }

    fn ray_intersects(&self, ray: &Ray2D) -> Option<f32> {
        let mut distance = 0.0;
        let mut max = f32::MAX;

        let mut check_axis =
            |dir_axis: f32, start_axis: f32, self_axis: f32, self_size: f32| -> bool {
                if dir_axis.abs() < 1e-06 {
                    if start_axis < self_axis || start_axis > self_axis + self_size {
                        return false;
                    }
                } else {
                    let inv_x = 1.0 / dir_axis;
                    let mut left = (self_axis - start_axis) * inv_x;
                    let mut right = (self_axis + self_size - start_axis) * inv_x;
                    if left > right {
                        std::mem::swap(&mut left, &mut right);
                    }

                    distance = left.max(distance);
                    max = right.min(max);
                    if distance > max {
                        return false;
                    }
                }

                true
            };

        if !check_axis(ray.direction.x, ray.start.x, self.x, self.width)
            || !check_axis(ray.direction.y, ray.start.y, self.y, self.height)
        {
            return None;
        }

        Some(distance)
    }
}
