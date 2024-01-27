use bevy::math::Vec2;
use num_enum::FromPrimitive;

pub mod colliders;
pub mod shapes;
pub mod spatial_hash;

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
