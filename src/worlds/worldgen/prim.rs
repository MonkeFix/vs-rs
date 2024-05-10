use bevy::prelude::*;

use crate::math::almost_equal_f32;

#[derive(Default, Debug, Clone, Copy)]
pub struct PrimEdge {
    pub u: Vec2,
    pub v: Vec2,
    distance: f32,
}

impl PrimEdge {
    pub fn new(u: Vec2, v: Vec2) -> Self {
        Self {
            u,
            v,
            distance: u.distance(v),
        }
    }
}

impl PartialEq for PrimEdge {
    fn eq(&self, other: &Self) -> bool {
        almost_equal_f32(self.distance, other.distance)
    }
}

impl Eq for PrimEdge {}

pub fn min_spanning_tree(edges: &[PrimEdge], start: Vec2) -> Vec<PrimEdge> {
    let mut open_set = vec![];
    let mut closed_set = vec![];

    for edge in edges {
        open_set.push(edge.u);
        open_set.push(edge.v);
    }

    closed_set.push(start);

    let mut res = vec![];

    while open_set.len() > 0 {
        let mut chosen = false;
        let mut chosen_edge = None;
        let mut min_weight = f32::INFINITY;

        for edge in edges {
            let mut closed_vert = 0;
            if !closed_set.contains(&edge.u) {
                closed_vert += 1;
            }
            if !closed_set.contains(&edge.v) {
                closed_vert += 1;
            }
            if closed_vert != 1 {
                continue;
            }

            if edge.distance < min_weight {
                chosen_edge = Some(*edge);
                chosen = true;
                min_weight = edge.distance;
            }
        }

        if !chosen {
            break;
        }
        if let Some(chosen_edge) = chosen_edge {
            res.push(chosen_edge);
            let ui = open_set.iter().position(|x| *x == chosen_edge.u).unwrap();
            let vi = open_set.iter().position(|x| *x == chosen_edge.v).unwrap();

            open_set.remove(ui);
            open_set.remove(vi);

            closed_set.push(chosen_edge.u);
            closed_set.push(chosen_edge.v);
        }
    }

    res
}
