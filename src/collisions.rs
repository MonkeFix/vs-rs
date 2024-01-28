use bevy::math::Vec2;
use num_enum::FromPrimitive;

use self::colliders::Collider;

pub mod colliders;
pub mod shapes;
pub mod spatial_hash;

#[derive(Debug, Default)]
pub struct CollisionResult {
    pub collider: Option<Collider>,
    pub normal: Vec2,
    pub min_translation: Vec2,
    pub point: Vec2,
}

#[derive(Debug, Default)]
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
}

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct RaycastHit {
    pub collider: Option<Collider>,
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
    if t < 0.0 || t > 1.0 {
        return false;
    }

    let u = (c.x * b.y - c.y * b.x) / d_dot;
    if u < 0.0 || u > 1.0 {
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

pub fn rect_to_rect(
    x1: f32,
    y1: f32,
    w1: f32,
    h1: f32,
    x2: f32,
    y2: f32,
    w2: f32,
    h2: f32,
) -> bool {
    x1 + w1 >= x2 && x1 <= x2 + w2 && y1 + h1 >= y2 && y1 <= y2 + h2
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
#[derive(Debug, Clone, Copy, Default, PartialEq)]
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
            // min_dist = self.y.abs();
            bounds_point.x = 0.0;
            bounds_point.y = self.y;
        }

        bounds_point
    }
}
