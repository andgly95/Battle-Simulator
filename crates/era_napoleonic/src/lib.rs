//! Napoleonic Wars era implementation (1803-1815)
//!
//! This crate provides:
//! - Napoleonic unit definitions
//! - Era-specific combat mechanics (musket, cannon, cavalry)
//! - Formation rules (line, column, square)
//! - Morale system for Napoleonic warfare
//! - Historical scenarios

pub mod units;
pub mod combat;
pub mod formations;
pub mod weapons;
pub mod scenarios;

use battle_sim_core::*;
use battle_sim_simulation::traits::Era;

/// Napoleonic Wars era (1803-1815)
pub struct NapoleonicEra {
    unit_database: units::UnitDatabase,
    weapon_database: weapons::WeaponDatabase,
}

impl NapoleonicEra {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            unit_database: units::UnitDatabase::load()?,
            weapon_database: weapons::WeaponDatabase::load()?,
        })
    }
}

impl Default for NapoleonicEra {
    fn default() -> Self {
        Self::new().expect("Failed to load Napoleonic era data")
    }
}

// Era trait implementation will be added once the trait is defined in simulation crate

/// Napoleonic-specific constants
pub mod constants {
    /// Average musket effective range in meters
    pub const MUSKET_EFFECTIVE_RANGE: f32 = 50.0;

    /// Average musket maximum range in meters
    pub const MUSKET_MAX_RANGE: f32 = 200.0;

    /// Average musket reload time in seconds
    pub const MUSKET_RELOAD_TIME: f32 = 15.0;

    /// Cannon effective range in meters
    pub const CANNON_EFFECTIVE_RANGE: f32 = 800.0;

    /// Infantry marching speed in m/s
    pub const INFANTRY_MARCH_SPEED: f32 = 1.2;

    /// Infantry running speed in m/s
    pub const INFANTRY_RUN_SPEED: f32 = 2.5;

    /// Cavalry trot speed in m/s
    pub const CAVALRY_TROT_SPEED: f32 = 4.0;

    /// Cavalry charge speed in m/s
    pub const CAVALRY_CHARGE_SPEED: f32 = 12.0;

    /// Morale threshold for routing
    pub const ROUT_THRESHOLD: f32 = 20.0;

    /// Morale threshold for wavering
    pub const WAVER_THRESHOLD: f32 = 40.0;
}
