use bevy::prelude::*;

pub mod bitmasking;
pub mod delaunay2d;
pub mod math;
pub mod prim;

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

/// Describes a 2D-rectangle with {x,y} being the top-left corner of the rectangle.
#[derive(Debug, Clone, Copy, Default, PartialEq, Reflect)]
pub struct FRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl FRect {
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

    pub fn intersects(&self, other: FRect) -> bool {
        other.left() < self.right()
            && self.left() < other.right()
            && other.top() < self.bottom()
            && self.top() < other.bottom()
    }

    pub fn union(&self, other: &FRect) -> FRect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);

        FRect::new(
            x,
            y,
            self.right().max(other.right()) - x,
            self.bottom().max(other.bottom()) - y,
        )
    }

    pub fn union_vec2(&self, vec: &Vec2) -> FRect {
        let rect = FRect::new(vec.x, vec.y, 0.0, 0.0);
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

    pub fn ray_intersects(&self, ray: &Ray2D) -> Option<f32> {
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
