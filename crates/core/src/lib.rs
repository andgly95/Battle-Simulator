//! Core engine for the Battle Simulator
//!
//! This crate provides:
//! - Entity Component System (ECS) wrapper
//! - Spatial partitioning structures (uniform grid, quadtree)
//! - Math utilities and types
//! - Core traits and interfaces

pub mod ecs;
pub mod spatial;
pub mod math;
pub mod types;

pub use glam::*;

/// Core result type for the battle simulator
pub type Result<T> = std::result::Result<T, Error>;

/// Core error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid position: {0}")]
    InvalidPosition(String),

    #[error("Entity not found: {0:?}")]
    EntityNotFound(bevy_ecs::entity::Entity),

    #[error("Spatial index error: {0}")]
    SpatialIndexError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Simulation time step in seconds
pub const FIXED_TIMESTEP: f32 = 1.0 / 30.0; // 30 ticks per second

/// Maximum simulation distance in meters (10km x 10km battlefield)
pub const MAX_BATTLEFIELD_SIZE: f32 = 10000.0;

/// Spatial grid cell size in meters
pub const SPATIAL_CELL_SIZE: f32 = 100.0;
