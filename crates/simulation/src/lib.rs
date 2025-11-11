//! Battle simulation engine
//!
//! This crate provides:
//! - Combat resolution systems
//! - Movement and pathfinding
//! - Morale and psychology
//! - AI decision making
//! - Order and command systems

pub mod combat;
pub mod movement;
pub mod morale;
pub mod ai;
pub mod orders;
pub mod formations;

use battle_sim_core::*;

/// Core components used throughout the simulation
pub mod components;

/// Systems that process entity updates
pub mod systems;

/// Traits for era-specific behavior
pub mod traits;
