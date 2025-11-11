//! World War 1 era implementation (1914-1918)
//!
//! This crate provides:
//! - WW1 unit definitions
//! - Trench warfare mechanics
//! - Machine gun and artillery systems
//! - Gas warfare
//! - Tank mechanics
//! - Historical WW1 scenarios

pub mod units;
pub mod combat;
pub mod trenches;
pub mod weapons;
pub mod scenarios;

use battle_sim_core::*;
use battle_sim_simulation::traits::Era;

/// World War 1 era (1914-1918)
pub struct WW1Era {
    unit_database: units::UnitDatabase,
    weapon_database: weapons::WeaponDatabase,
}

impl WW1Era {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            unit_database: units::UnitDatabase::load()?,
            weapon_database: weapons::WeaponDatabase::load()?,
        })
    }
}

impl Default for WW1Era {
    fn default() -> Self {
        Self::new().expect("Failed to load WW1 era data")
    }
}

/// WW1-specific constants
pub mod constants {
    /// Rifle effective range in meters
    pub const RIFLE_EFFECTIVE_RANGE: f32 = 300.0;

    /// Machine gun effective range in meters
    pub const MG_EFFECTIVE_RANGE: f32 = 1000.0;

    /// Artillery effective range in meters
    pub const ARTILLERY_EFFECTIVE_RANGE: f32 = 5000.0;

    /// Infantry walking speed in m/s
    pub const INFANTRY_WALK_SPEED: f32 = 1.5;

    /// Infantry charge speed in m/s
    pub const INFANTRY_CHARGE_SPEED: f32 = 3.5;

    /// Suppression threshold for taking cover
    pub const SUPPRESSION_THRESHOLD: f32 = 60.0;

    /// Trench bonus to defense
    pub const TRENCH_DEFENSE_MULTIPLIER: f32 = 3.0;
}
