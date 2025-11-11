//! Morale and psychology systems

use bevy_ecs::prelude::*;
use rand::Rng;
use crate::components::*;
use battle_sim_core::spatial::SpatialIndex;

/// Morale event types
#[derive(Debug, Clone)]
pub enum MoraleEvent {
    TakingCasualties { percentage_lost: f32 },
    FriendlyUnitRouted { distance: f32 },
    EnemyUnitRouted { distance: f32 },
    EnemyCharge { charging_unit_size: u32 },
    FlankAttack,
    RearAttack,
    CommanderKilled,
    CommanderPresent,
    VictoryInMelee,
    DefeatInMelee,
    RalliedByOfficer,
    UnderArtilleryFire,
}

/// Calculate morale change from an event
pub fn calculate_morale_change(
    event: &MoraleEvent,
    experience: &Experience,
) -> f32 {
    let base_change = match event {
        MoraleEvent::TakingCasualties { percentage_lost } => {
            -percentage_lost * 30.0
        },
        MoraleEvent::FriendlyUnitRouted { distance } => {
            let proximity = (500.0 - distance).max(0.0) / 500.0;
            -15.0 * proximity
        },
        MoraleEvent::EnemyUnitRouted { distance } => {
            let proximity = (500.0 - distance).max(0.0) / 500.0;
            15.0 * proximity
        },
        MoraleEvent::EnemyCharge { charging_unit_size } => {
            -10.0 * (*charging_unit_size as f32 / 800.0)
        },
        MoraleEvent::FlankAttack => -20.0,
        MoraleEvent::RearAttack => -30.0,
        MoraleEvent::CommanderKilled => -25.0,
        MoraleEvent::CommanderPresent => 5.0,
        MoraleEvent::VictoryInMelee => 15.0,
        MoraleEvent::DefeatInMelee => -25.0,
        MoraleEvent::RalliedByOfficer => 10.0,
        MoraleEvent::UnderArtilleryFire => -5.0,
    };

    // Apply experience modifier
    let experience_modifier = match experience.level {
        ExperienceLevel::Raw => 1.5,      // More affected
        ExperienceLevel::Trained => 1.0,   // Normal
        ExperienceLevel::Veteran => 0.7,   // More resilient
        ExperienceLevel::Elite => 0.5,     // Very resilient
    };

    base_change * experience_modifier
}

/// Morale check system - handles routing and rallying
pub fn morale_check_system(
    mut query: Query<(
        Entity,
        &mut Morale,
        &mut AIState,
        &Squad,
        &Experience,
        &Position,
    )>,
    spatial: Res<SpatialIndex>,
) {
    let mut rng = rand::thread_rng();

    for (entity, mut morale, mut ai_state, squad, experience, pos) in query.iter_mut() {
        // Check for routing
        if morale.is_routing() && ai_state.state != BehaviorState::Routing {
            let break_chance = 1.0 - (morale.current / 20.0).powi(2);

            if rng.gen::<f32>() < break_chance {
                ai_state.state = BehaviorState::Routing;
                tracing::warn!(
                    "Unit {:?} has broken and is routing! (morale: {:.1})",
                    entity,
                    morale.current
                );
            }
        }

        // Check for wavering
        if morale.is_wavering() && ai_state.state == BehaviorState::Idle {
            ai_state.state = BehaviorState::Idle; // Wavering doesn't change state but affects combat
        }

        // Recovery from wavering
        if ai_state.state == BehaviorState::Idle && morale.current > 50.0 {
            // Recovered
        }

        // Rally check for routing units
        if ai_state.state == BehaviorState::Routing {
            // Check distance to nearest enemy
            let nearby_enemies = spatial.grid.query_radius(pos.x, pos.y, 300.0);

            if nearby_enemies.is_empty() || morale.current > 30.0 {
                // Chance to rally
                let rally_chance = (morale.current - 20.0) / 80.0;

                if rng.gen::<f32>() < rally_chance * 0.05 { // 5% check per tick
                    ai_state.state = BehaviorState::Reforming;
                    tracing::info!(
                        "Unit {:?} is rallying! (morale: {:.1})",
                        entity,
                        morale.current
                    );
                }
            }
        }

        // Slow morale recovery when out of combat
        // Only recover when truly out of combat (at 30 ticks/sec, 0.001 = 0.03 morale/sec)
        if ai_state.state == BehaviorState::Idle && morale.current < morale.base {
            morale.modify(0.001); // Very slow recovery - takes ~30 seconds to recover 1 point
        }

        // Check for unit destruction
        if squad.size == 0 {
            ai_state.state = BehaviorState::Routing;
            morale.current = 0.0;
        }
    }
}

/// Routing behavior system - units flee from enemy
pub fn routing_behavior_system(
    mut query: Query<(
        &AIState,
        &Position,
        &mut Velocity,
        &mut Destination,
        &Team,
    )>,
    all_units: Query<(&Position, &Team)>,
) {
    for (ai_state, pos, mut vel, mut dest, team) in query.iter_mut() {
        if ai_state.state == BehaviorState::Routing {
            // Find nearest enemy
            let mut nearest_enemy_pos: Option<(f32, f32)> = None;
            let mut nearest_dist = f32::MAX;

            for (enemy_pos, enemy_team) in all_units.iter() {
                if team.is_enemy(enemy_team) {
                    let dist = pos.distance(enemy_pos);
                    if dist < nearest_dist {
                        nearest_dist = dist;
                        nearest_enemy_pos = Some((enemy_pos.x, enemy_pos.y));
                    }
                }
            }

            // Flee away from enemy
            if let Some((enemy_x, enemy_y)) = nearest_enemy_pos {
                // Run in opposite direction
                let dx = pos.x - enemy_x;
                let dy = pos.y - enemy_y;
                let distance = (dx * dx + dy * dy).sqrt();

                if distance > 0.0 {
                    // Set destination far away
                    dest.x = pos.x + (dx / distance) * 500.0;
                    dest.y = pos.y + (dy / distance) * 500.0;

                    // Run fast
                    vel.dx = dx / distance;
                    vel.dy = dy / distance;
                    vel.speed = 2.5; // Running speed
                }
            }
        }
    }
}
