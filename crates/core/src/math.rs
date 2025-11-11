//! Math utilities and helper functions

pub use glam::*;

/// Calculate distance between two 2D points
pub fn distance_2d(a: Vec2, b: Vec2) -> f32 {
    a.distance(b)
}

/// Calculate angle between two 2D points
pub fn angle_between(from: Vec2, to: Vec2) -> f32 {
    (to - from).y.atan2((to - from).x)
}
