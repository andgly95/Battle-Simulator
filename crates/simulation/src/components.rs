//! ECS components for simulation

use serde::{Deserialize, Serialize};
use bevy_ecs::prelude::*;

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
    pub speed: f32,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}
