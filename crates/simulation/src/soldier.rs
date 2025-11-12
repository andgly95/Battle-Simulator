//! Individual soldier simulation
//!
//! Each soldier is a separate entity with:
//! - Position in world space
//! - Assigned position in formation
//! - Individual AI to move to formation position
//! - Weapon state (loaded, firing, reloading)
//! - Health (alive/dead)

use bevy_ecs::prelude::*;
use crate::components::*;
use glam::Vec2;

/// Individual soldier component
#[derive(Component, Debug, Clone)]
pub struct Soldier {
    /// Which battalion this soldier belongs to
    pub battalion: Entity,
    
    /// This soldier's assigned position in the formation
    /// (e.g., rank 3, file 12 in a line formation)
    pub formation_rank: u32,
    pub formation_file: u32,
    
    /// Target position in world space (where formation says they should be)
    pub target_position: Vec2,
    
    /// Is this soldier in the front rank? (can shoot)
    pub in_front_rank: bool,
    
    /// Soldier's current state
    pub state: SoldierState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoldierState {
    /// Moving to formation position
    MovingToPosition,
    /// In position, ready to fight
    Ready,
    /// Loading weapon
    Reloading,
    /// Aiming at target
    Aiming,
    /// Firing
    Firing,
    /// Running away (routing)
    Routing,
    /// Dead
    Dead,
}

/// Individual soldier weapon state
#[derive(Component, Debug, Clone)]
pub struct SoldierWeapon {
    pub weapon_type: WeaponType,
    pub loaded: bool,
    pub reload_time: f32,
    pub time_since_fire: f32,
    pub ammunition: u32,
}

impl SoldierWeapon {
    pub fn new_musket() -> Self {
        Self {
            weapon_type: WeaponType::Musket,
            loaded: true,
            reload_time: 15.0,
            time_since_fire: 15.0,
            ammunition: 60,
        }
    }
    
    pub fn can_fire(&self) -> bool {
        self.loaded && self.ammunition > 0
    }
    
    pub fn fire(&mut self) {
        self.loaded = false;
        self.ammunition -= 1;
        self.time_since_fire = 0.0;
    }
    
    pub fn update(&mut self, dt: f32) {
        self.time_since_fire += dt;
        if !self.loaded && self.time_since_fire >= self.reload_time {
            self.loaded = true;
        }
    }
}

/// Projectile in flight
#[derive(Component, Debug, Clone)]
pub struct Projectile {
    pub velocity: Vec2,
    pub start_pos: Vec2,
    pub target_pos: Vec2,
    pub damage: f32,
    pub max_range: f32,
    pub distance_traveled: f32,
}
