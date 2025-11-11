//! Traits for era-specific behavior

use bevy_ecs::world::World;

/// Trait for era-specific implementations
pub trait Era: Send + Sync {
    fn name(&self) -> &str;
    fn time_period(&self) -> (i32, i32);
}
