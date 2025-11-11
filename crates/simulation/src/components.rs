//! ECS components for simulation

use serde::{Deserialize, Serialize};
use bevy_ecs::prelude::*;

// ============================================================================
// Position & Movement Components
// ============================================================================

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y, z: 0.0 }
    }

    pub fn distance(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
    pub speed: f32,
}

impl Velocity {
    pub fn new(dx: f32, dy: f32, speed: f32) -> Self {
        Self { dx, dy, speed }
    }

    pub fn zero() -> Self {
        Self { dx: 0.0, dy: 0.0, speed: 0.0 }
    }
}

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Facing {
    /// Direction in radians (0 = east, π/2 = north)
    pub angle: f32,
}

impl Facing {
    pub fn new(angle: f32) -> Self {
        Self { angle }
    }
}

// ============================================================================
// Unit Identity & Organization
// ============================================================================

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct UnitIdentity {
    pub id: String,
    pub name: String,
    pub nation: String,
    pub unit_type: UnitType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitType {
    Infantry,
    Cavalry,
    Artillery,
    Commander,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Squad {
    pub size: u32,
    pub max_size: u32,
    pub casualties: u32,
    pub cohesion: f32,  // 0.0 to 1.0
}

impl Squad {
    pub fn new(size: u32) -> Self {
        Self {
            size,
            max_size: size,
            casualties: 0,
            cohesion: 1.0,
        }
    }

    pub fn strength_percentage(&self) -> f32 {
        self.size as f32 / self.max_size as f32
    }

    pub fn apply_casualties(&mut self, killed: u32) {
        let actual = killed.min(self.size);
        self.size -= actual;
        self.casualties += actual;

        // Cohesion drops with casualties
        let loss_rate = actual as f32 / self.max_size as f32;
        self.cohesion = (self.cohesion - loss_rate * 0.3).max(0.0);
    }
}

// ============================================================================
// Combat Components
// ============================================================================

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self {
            current: max,
            maximum: max,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn percentage(&self) -> f32 {
        self.current / self.maximum
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Weapon {
    pub weapon_id: String,
    pub weapon_type: WeaponType,
    pub range: f32,
    pub effective_range: f32,
    pub accuracy: f32,
    pub reload_time: f32,
    pub time_since_fire: f32,
    pub ammunition: u32,
    pub max_ammunition: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeaponType {
    Musket,
    Rifle,
    Cannon,
    MachineGun,
    Bayonet,
}

impl Weapon {
    pub fn can_fire(&self) -> bool {
        self.ammunition > 0 && self.time_since_fire >= self.reload_time
    }

    pub fn fire(&mut self) {
        if self.ammunition > 0 {
            self.ammunition -= 1;
        }
        self.time_since_fire = 0.0;
    }

    pub fn update_reload(&mut self, dt: f32) {
        self.time_since_fire += dt;
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub melee_skill: f32,      // 0-100
    pub ranged_skill: f32,     // 0-100
    pub defense: f32,          // 0-100
}

impl CombatStats {
    pub fn new(melee: f32, ranged: f32, defense: f32) -> Self {
        Self {
            melee_skill: melee,
            ranged_skill: ranged,
            defense,
        }
    }
}

// ============================================================================
// Morale & Psychology
// ============================================================================

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Morale {
    pub current: f32,   // 0-100
    pub base: f32,      // 0-100
}

impl Morale {
    pub fn new(base: f32) -> Self {
        Self {
            current: base,
            base,
        }
    }

    pub fn modify(&mut self, delta: f32) {
        self.current = (self.current + delta).clamp(0.0, 100.0);
    }

    pub fn is_routing(&self) -> bool {
        self.current < 20.0
    }

    pub fn is_wavering(&self) -> bool {
        self.current < 40.0 && self.current >= 20.0
    }

    pub fn is_steady(&self) -> bool {
        self.current >= 40.0
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Fatigue {
    pub current: f32,   // 0-100 (0 = fresh, 100 = exhausted)
    pub max: f32,
    pub recovery_rate: f32,
}

impl Fatigue {
    pub fn new() -> Self {
        Self {
            current: 0.0,
            max: 100.0,
            recovery_rate: 1.5,
        }
    }

    pub fn add(&mut self, amount: f32) {
        self.current = (self.current + amount).min(self.max);
    }

    pub fn recover(&mut self, dt: f32) {
        self.current = (self.current - self.recovery_rate * dt).max(0.0);
    }

    pub fn modifier(&self) -> f32 {
        1.0 - (self.current / 100.0) * 0.3
    }
}

impl Default for Fatigue {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Experience {
    pub level: ExperienceLevel,
    pub battles: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Raw,
    Trained,
    Veteran,
    Elite,
}

impl Experience {
    pub fn new(level: ExperienceLevel) -> Self {
        Self { level, battles: 0 }
    }

    pub fn morale_modifier(&self) -> f32 {
        match self.level {
            ExperienceLevel::Raw => 0.7,
            ExperienceLevel::Trained => 1.0,
            ExperienceLevel::Veteran => 1.2,
            ExperienceLevel::Elite => 1.4,
        }
    }

    pub fn combat_modifier(&self) -> f32 {
        match self.level {
            ExperienceLevel::Raw => 0.8,
            ExperienceLevel::Trained => 1.0,
            ExperienceLevel::Veteran => 1.15,
            ExperienceLevel::Elite => 1.3,
        }
    }
}

// ============================================================================
// Formation Components
// ============================================================================

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Formation {
    pub formation_type: FormationType,
    pub width: f32,
    pub depth: f32,
    pub spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormationType {
    Line,
    Column,
    Square,
    Skirmish,
}

impl Formation {
    pub fn new(formation_type: FormationType, size: u32) -> Self {
        let (width, depth, spacing) = match formation_type {
            FormationType::Line => {
                let men_per_rank = size / 2;
                (men_per_rank as f32 * 0.6, 2.0, 0.6)
            }
            FormationType::Column => {
                let men_per_rank = size / 12;
                (men_per_rank as f32 * 0.5, 12.0, 0.5)
            }
            FormationType::Square => {
                let side = (size as f32 / 4.0).sqrt();
                (side * 0.6, side * 0.6, 0.6)
            }
            FormationType::Skirmish => {
                let side = (size as f32).sqrt();
                (side * 5.0, side * 5.0, 5.0)
            }
        };

        Self {
            formation_type,
            width,
            depth,
            spacing,
        }
    }

    pub fn firepower_modifier(&self) -> f32 {
        match self.formation_type {
            FormationType::Line => 1.2,
            FormationType::Column => 0.4,
            FormationType::Square => 0.7,
            FormationType::Skirmish => 0.6,
        }
    }

    pub fn target_profile_modifier(&self) -> f32 {
        match self.formation_type {
            FormationType::Line => 1.2,
            FormationType::Column => 1.5,
            FormationType::Square => 0.9,
            FormationType::Skirmish => 0.4,
        }
    }

    pub fn speed_modifier(&self) -> f32 {
        match self.formation_type {
            FormationType::Line => 0.9,
            FormationType::Column => 1.1,
            FormationType::Square => 0.3,
            FormationType::Skirmish => 1.2,
        }
    }

    pub fn melee_attack_modifier(&self) -> f32 {
        match self.formation_type {
            FormationType::Line => 1.0,
            FormationType::Column => 1.3,
            FormationType::Square => 0.8,
            FormationType::Skirmish => 0.5,
        }
    }

    pub fn melee_defense_modifier(&self) -> f32 {
        match self.formation_type {
            FormationType::Line => 0.9,
            FormationType::Column => 0.8,
            FormationType::Square => 1.5,
            FormationType::Skirmish => 0.5,
        }
    }
}

// ============================================================================
// AI & Orders
// ============================================================================

#[derive(Component, Debug, Clone)]
pub struct AIState {
    pub current_order: Order,
    pub target: Option<Entity>,
    pub state: BehaviorState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Hold,
    Advance,
    Retreat,
    Attack(Entity),
    Defend,
    ChangeFormation(FormationType),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BehaviorState {
    Idle,
    Moving,
    Engaging,
    Routing,
    Rallying,
    Reforming,
}

impl AIState {
    pub fn new() -> Self {
        Self {
            current_order: Order::Hold,
            target: None,
            state: BehaviorState::Idle,
        }
    }
}

impl Default for AIState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Destination {
    pub x: f32,
    pub y: f32,
    pub tolerance: f32,
}

impl Destination {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            tolerance: 5.0,
        }
    }

    pub fn reached(&self, pos: &Position) -> bool {
        let dx = pos.x - self.x;
        let dy = pos.y - self.y;
        (dx * dx + dy * dy).sqrt() < self.tolerance
    }
}

// ============================================================================
// Team/Side
// ============================================================================

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Team {
    pub id: u32,
    pub side: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Allied,
    Enemy,
    Neutral,
}

impl Team {
    pub fn new(id: u32, side: Side) -> Self {
        Self { id, side }
    }

    pub fn is_enemy(&self, other: &Team) -> bool {
        self.side != other.side && other.side != Side::Neutral && self.side != Side::Neutral
    }
}
