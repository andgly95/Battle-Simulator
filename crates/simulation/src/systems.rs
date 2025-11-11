//! ECS systems for simulation

use bevy_ecs::prelude::*;
use battle_sim_core::spatial::SpatialIndex;
use crate::components::*;
use crate::movement::{SimulationTime, movement_system, destination_system, fatigue_system};
use crate::combat::{ranged_combat_system, melee_combat_system};
use crate::morale::{morale_check_system, routing_behavior_system};

/// Spatial index update system - rebuilds spatial index each frame
pub fn spatial_index_update_system(
    mut spatial: ResMut<SpatialIndex>,
    query: Query<(Entity, &Position)>,
) {
    // Clear the spatial index
    spatial.grid.clear();

    // Reinsert all entities
    for (entity, pos) in query.iter() {
        spatial.grid.insert(entity, pos.x, pos.y);
    }
}

/// Statistics tracking resource
#[derive(Resource, Default)]
pub struct BattleStatistics {
    pub tick: u64,
    pub total_units: usize,
    pub routing_units: usize,
    pub total_casualties: u32,
}

/// Statistics update system
pub fn statistics_system(
    mut stats: ResMut<BattleStatistics>,
    query: Query<(&Squad, &AIState)>,
) {
    stats.tick += 1;
    stats.total_units = query.iter().count();
    stats.routing_units = query.iter().filter(|(_, ai)| ai.state == BehaviorState::Routing).count();
    stats.total_casualties = query.iter().map(|(squad, _)| squad.casualties).sum();
}

/// Print battle status
pub fn status_report_system(
    stats: Res<BattleStatistics>,
    time: Res<SimulationTime>,
) {
    if stats.tick % 300 == 0 { // Every 10 seconds (at 30 ticks/sec)
        tracing::info!(
            "Battle status at t={:.1}s: {} units, {} routing, {} total casualties",
            time.elapsed(),
            stats.total_units,
            stats.routing_units,
            stats.total_casualties
        );
    }
}

/// Schedule for running all simulation systems
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimulationSystemSet {
    Input,
    AI,
    Movement,
    Combat,
    Morale,
    SpatialUpdate,
    Statistics,
}
