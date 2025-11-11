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

/// Core components used throughout the simulation
pub mod components;

/// Systems that process entity updates
pub mod systems;

/// Traits for era-specific behavior
pub mod traits;

// Re-export commonly used items
pub use combat::{ranged_combat_system, melee_combat_system, weapon_reload_system};
pub use movement::{movement_system, destination_system, fatigue_system, SimulationTime};
pub use morale::{morale_check_system, routing_behavior_system};
pub use systems::{spatial_index_update_system, statistics_system, status_report_system, BattleStatistics};
