//! Common types used throughout the simulator

use serde::{Deserialize, Serialize};

/// Position in 2D space (meters)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// Direction/facing (radians)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Facing {
    pub angle: f32,
}
