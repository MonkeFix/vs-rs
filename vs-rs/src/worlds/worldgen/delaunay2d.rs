#![allow(dead_code)]

use bevy::prelude::*;
use common::math::almost_equal_vec2;

use super::room::WorldRoom;

#[derive(Default, Clone, Copy)]
pub struct Edge {
    pub u: Vec2,
    pub v: Vec2,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        almost_equal_vec2(self.u, other.u) && almost_equal_vec2(self.v, other.v)
    }
}

impl Eq for Edge {}

#[derive(Default)]
pub struct Triangle {
    pub a: Vec2,
    pub b: Vec2,
    pub c: Vec2,
    pub is_bad: bool,
}

impl Triangle {
    pub fn new(a: Vec2, b: Vec2, c: Vec2) -> Self {
        Self {
            a,
            b,
            c,
            is_bad: false,
        }
    }

    pub fn contains(&self, v: Vec2) -> bool {
        Vec2::distance(v, self.a) < 0.01
            || Vec2::distance(v, self.b) < 0.01
            || Vec2::distance(v, self.c) < 0.01
    }

    pub fn circum_circle_contains(&self, v: &Vec2) -> bool {
        let ab = self.a.length_squared();
        let cd = self.b.length_squared();
        let ef = self.c.length_squared();

        let circum_x =
            (ab * (self.c.y - self.b.y) + cd * (self.a.y - self.c.y) + ef * (self.b.y - self.a.y))
                / (self.a.x * (self.c.y - self.b.y)
                    + self.b.x * (self.a.y - self.c.y)
                    + self.c.x * (self.b.y - self.a.y));
        let circum_y =
            (ab * (self.c.x - self.b.x) + cd * (self.a.x - self.c.x) + ef * (self.b.x - self.a.x))
                / (self.a.y * (self.c.x - self.b.x)
                    + self.b.y * (self.a.x - self.c.x)
                    + self.c.y * (self.b.x - self.a.x));

        let circum = Vec2::new(circum_x / 2., circum_y / 2.);
        let radius = (self.a - circum).length_squared();
        let dist = (*v - circum).length_squared();

        dist <= radius
    }
}

pub struct Delaunay2D {
    vertices: Vec<Vec2>,
    pub edges: Vec<Edge>,
    triangles: Vec<Triangle>,
}

impl Delaunay2D {
    pub fn triangulate(vertices: Vec<Vec2>) -> Delaunay2D {
        let mut res = Delaunay2D {
            edges: vec![],
            triangles: vec![],
            vertices,
        };
        res.triangulate_internal();
        res
    }

    pub fn triangulate_constraint(rooms: &[WorldRoom]) -> Delaunay2D {
        let mut res = Delaunay2D {
            edges: vec![],
            triangles: vec![],
            vertices: vec![],
        };

        for i in 0..rooms.len() {
            for j in (i + 1)..rooms.len() {
                let a = rooms[i].rect;
                let b = rooms[j].rect;

                res.edges.push(Edge {
                    u: a.center(),
                    v: b.center(),
                });
            }
        }

        res
    }

    fn triangulate_internal(&mut self) {
        let mut min_x = self.vertices[0].x;
        let mut min_y = self.vertices[0].y;
        let mut max_x = min_x;
        let mut max_y = min_y;

        for vertex in &self.vertices {
            if vertex.x < min_x {
                min_x = vertex.x;
            }
            if vertex.x > max_x {
                max_x = vertex.x;
            }
            if vertex.y < min_y {
                min_y = vertex.y;
            }
            if vertex.y > max_y {
                max_y = vertex.y;
            }
        }

        let dx = max_x - min_x;
        let dy = max_y - min_y;
        let dt_max = dx.max(dy) * 2.;

        let p1 = Vec2::new(min_x - 1., min_y - 1.);
        let p2 = Vec2::new(min_x - 1., max_y + dt_max);
        let p3 = Vec2::new(max_x + dt_max, min_y - 1.);

        self.triangles.push(Triangle::new(p1, p2, p3));

        for vertex in &self.vertices {
            let mut polygon = Vec::new();

            for t in self.triangles.iter_mut() {
                if t.circum_circle_contains(vertex) {
                    t.is_bad = true;
                    polygon.push(Edge { u: t.a, v: t.b });
                    polygon.push(Edge { u: t.b, v: t.c });
                    polygon.push(Edge { u: t.c, v: t.a });
                }
            }

            // remove bad rectangles
            self.triangles.retain(|x| !x.is_bad);

            for edge in polygon {
                self.triangles.push(Triangle::new(edge.u, edge.v, *vertex));
            }
        }

        self.triangles
            .retain(|x| !x.contains(p1) && !x.contains(p2) && !x.contains(p3));

        let mut added_edges = vec![];

        for t in &self.triangles {
            let ab = Edge { u: t.a, v: t.b };
            let bc = Edge { u: t.b, v: t.c };
            let ca = Edge { u: t.c, v: t.c };

            if !added_edges.contains(&ab) {
                added_edges.push(ab);
                self.edges.push(ab);
            }
            if !added_edges.contains(&bc) {
                added_edges.push(bc);
                self.edges.push(bc);
            }
            if !added_edges.contains(&ca) {
                added_edges.push(ca);
                self.edges.push(ca);
            }
        }
    }
}
