use glam::{Quat, Vec3};
use std::collections::VecDeque;

pub struct NavigationTarget {
    pub target_position: Vec3,
    pub target_orientation: Option<Quat>,
    pub arrival_threshold: f32,
}

impl NavigationTarget {
    pub fn new(position: Vec3, orientation: Option<Quat>, threshold: f32) -> Self {
        Self {
            target_position: position,
            target_orientation: orientation,
            arrival_threshold: threshold,
        }
    }

    pub fn position_only(position: Vec3, threshold: f32) -> Self {
        Self::new(position, None, threshold)
    }
}

pub struct NavigationQueue {
    pub waypoints: VecDeque<NavigationTarget>,
}

impl NavigationQueue {
    pub fn new() -> Self {
        Self {
            waypoints: VecDeque::new(),
        }
    }

    pub fn add_waypoint(&mut self, waypoint: NavigationTarget) {
        self.waypoints.push_back(waypoint);
    }

    pub fn clear(&mut self) {
        self.waypoints.clear();
    }
}
