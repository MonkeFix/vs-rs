#![allow(dead_code)]

use std::collections::VecDeque;

use bevy::prelude::*;
use common::Position;

use super::{steering::SteeringTarget, SteeringHost};

#[derive(Debug, Clone, Copy, Reflect)]
pub struct SteerPathNode {
    /// Current position.
    pub position: Vec2,
    /// Determines when to slow down. Useful for `SteerArrival` behavior.
    pub arrival_radius: f32,
    /// Node's radius. When reached, the node is removed.
    pub target_radius: f32,
}

impl SteerPathNode {
    pub fn new(position: Vec2, arrival_radius: f32, target_radius: f32) -> Self {
        Self {
            position,
            arrival_radius,
            target_radius,
        }
    }
}

impl SteeringTarget for SteerPathNode {
    fn position(&self) -> Vec2 {
        self.position
    }
}

#[derive(Debug, Clone, Reflect)]
pub struct SteerPath {
    pub max_nodes: usize,
    nodes: VecDeque<SteerPathNode>,
    last_added: Option<SteerPathNode>,
}

impl Default for SteerPath {
    fn default() -> Self {
        let max_nodes = 32;
        Self {
            max_nodes,
            nodes: VecDeque::with_capacity(max_nodes),
            last_added: None,
        }
    }
}

impl SteerPath {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            max_nodes,
            nodes: VecDeque::with_capacity(max_nodes),
            last_added: None,
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    pub fn push(&mut self, node: SteerPathNode) -> Option<SteerPathNode> {
        let mut res = None;

        self.last_added = Some(node);

        if self.nodes.len() < self.max_nodes {
            self.nodes.push_front(node);
        } else {
            res = self.remove_target();
            self.push(node);
        }

        res
    }

    pub fn get_target(&self) -> Option<SteerPathNode> {
        if let Some(n) = self.nodes.front() {
            return Some(*n);
        }

        None
    }

    pub fn remove_target(&mut self) -> Option<SteerPathNode> {
        self.nodes.pop_front()
    }

    pub fn last_added(&self) -> Option<&SteerPathNode> {
        self.last_added.as_ref()
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<SteerPathNode> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<SteerPathNode> {
        self.nodes.iter_mut()
    }

    pub fn into_iter(self) -> std::collections::vec_deque::IntoIter<SteerPathNode> {
        self.nodes.into_iter()
    }

    pub fn index(&self, index: usize) -> &SteerPathNode {
        &self.nodes[index]
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
}

impl SteeringTarget for SteerPath {
    fn position(&self) -> Vec2 {
        if let Some(target) = self.get_target() {
            return target.position;
        }

        if let Some(last) = self.last_added() {
            return last.position;
        }

        Vec2::ZERO
    }
}

#[derive(Debug, Reflect)]
pub enum PathFollowingMode {
    OneWay,
    Looped,
    Patrol,
}

#[derive(Debug, Reflect)]
pub struct SteerPathFollowing {
    pub mode: PathFollowingMode,
    cur_node_index: usize,
    path_dir: i32,
}

impl Default for SteerPathFollowing {
    fn default() -> Self {
        Self {
            mode: PathFollowingMode::OneWay,
            cur_node_index: 0,
            path_dir: -1,
        }
    }
}

impl SteerPathFollowing {
    pub fn new(mode: PathFollowingMode) -> Self {
        Self {
            mode,
            cur_node_index: 0,
            path_dir: -1,
        }
    }

    pub fn steer<F>(
        &mut self,
        position: &Position,
        host: &SteeringHost,
        path: &mut SteerPath,
        steering_fn: F,
    ) -> Vec2
    where
        F: Fn(SteerPathNode) -> Vec2,
    {
        match self.mode {
            PathFollowingMode::OneWay => self.one_way(position, host, path, steering_fn),
            PathFollowingMode::Looped => self.looped(position, path, steering_fn),
            PathFollowingMode::Patrol => self.patrol(position, path, steering_fn),
        }
    }

    fn within_target(&self, position: Vec2, target: &SteerPathNode) -> bool {
        let dist = (position - target.position).length();
        dist <= target.target_radius
    }

    fn one_way<F>(
        &mut self,
        position: &Position,
        host: &SteeringHost,
        path: &mut SteerPath,
        steering_fn: F,
    ) -> Vec2
    where
        F: Fn(SteerPathNode) -> Vec2,
    {
        if let Some(target) = path.get_target() {
            if self.within_target(position.0, &target) {
                path.remove_target();
            }

            return steering_fn(target);
        }

        -host.velocity
    }

    fn patrol<F>(&mut self, position: &Position, path: &mut SteerPath, steering_fn: F) -> Vec2
    where
        F: Fn(SteerPathNode) -> Vec2,
    {
        let mut node = path.index(self.cur_node_index);
        let mut index: i32 = 0;

        if self.within_target(position.0, node) {
            index += self.path_dir;

            if index >= path.nodes.len() as i32 || index < 0 {
                self.path_dir *= -1;
                index += self.path_dir;
            }

            self.cur_node_index = index as usize;

            node = path.index(self.cur_node_index);
        }

        steering_fn(*node)
    }

    fn looped<F>(&mut self, position: &Position, path: &mut SteerPath, steering_fn: F) -> Vec2
    where
        F: Fn(SteerPathNode) -> Vec2,
    {
        let mut node = path.index(self.cur_node_index % path.len());

        if self.within_target(position.0, node) {
            self.cur_node_index += 1;
            node = path.index(self.cur_node_index % path.len());
        }

        steering_fn(*node)
    }
}
