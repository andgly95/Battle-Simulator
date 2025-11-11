//! Combat resolution systems

use bevy_ecs::prelude::*;
use rand::Rng;
use rand_distr::{Distribution, Poisson};
use crate::components::*;
use battle_sim_core::spatial::SpatialIndex;

/// Calculate distance modifier for ranged combat
///
/// Based on the ratio of actual distance to effective range:
/// - 0-0.5x effective range: 100% effectiveness
/// - 0.5-1.0x effective range: Linear falloff to 70%
/// - 1.0-2.0x effective range: Exponential falloff to 16%
/// - 2.0x+ effective range: < 5% effectiveness
pub fn calculate_distance_modifier(distance: f32, effective_range: f32) -> f32 {
    let ratio = distance / effective_range;

    if ratio <= 0.5 {
        1.0
    } else if ratio <= 1.0 {
        1.0 - (ratio - 0.5) * 0.6  // 100% → 70%
    } else if ratio <= 2.0 {
        0.7 * (-1.5 * (ratio - 1.0)).exp()  // 70% → 16%
    } else {
        0.05 * (1.0 / ratio)  // < 5%
    }
}

/// Calculate morale modifier for combat
pub fn calculate_morale_modifier(morale: &Morale) -> f32 {
    // Square root scaling - diminishing returns
    (morale.current / 100.0).sqrt()
}

/// Weapon reload system - updates reload timers
pub fn weapon_reload_system(
    mut query: Query<&mut Weapon>,
    time: Res<crate::movement::SimulationTime>,
) {
    let dt = time.delta();
    for mut weapon in query.iter_mut() {
        weapon.update_reload(dt);
    }
}

/// Ranged combat system
pub fn ranged_combat_system(
    spatial: Res<SpatialIndex>,
    mut query: Query<(
        Entity,
        &Position,
        &mut Weapon,
        &Formation,
        &mut Morale,
        &Fatigue,
        &Experience,
        &Team,
        &mut Squad,
    )>,
) {
    let mut rng = rand::thread_rng();

    // Collect all potential shots to avoid borrow checker issues
    let mut shots = Vec::new();

    for (shooter_entity, shooter_pos, weapon, shooter_formation, shooter_morale,
         shooter_fatigue, shooter_exp, shooter_team, _squad) in query.iter() {

        // Check if can fire
        if !weapon.can_fire() {
            continue;
        }

        // Find nearby enemies
        let nearby = spatial.grid.query_radius(shooter_pos.x, shooter_pos.y, weapon.range);

        // Find best target
        let mut best_target: Option<(Entity, f32)> = None;
        let mut best_score = 0.0;

        for entity in nearby {
            // Skip self
            if entity == shooter_entity {
                continue;
            }

            if let Ok((_, target_pos, _, _, _, _, _, target_team, _)) = query.get(entity) {
                // Check if enemy
                if !shooter_team.is_enemy(target_team) {
                    continue;
                }

                let distance = shooter_pos.distance(target_pos);
                if distance <= weapon.range {
                    // Score based on distance (prefer closer targets)
                    let score = 1.0 - (distance / weapon.range);
                    if score > best_score {
                        best_score = score;
                        best_target = Some((entity, distance));
                    }
                }
            }
        }

        // Record shot if we have a target
        if let Some((target_entity, distance)) = best_target {
            // Calculate hit probability
            let base_accuracy = weapon.accuracy * shooter_exp.combat_modifier();
            let distance_mod = calculate_distance_modifier(distance, weapon.effective_range);
            let morale_mod = calculate_morale_modifier(shooter_morale);
            let fatigue_mod = shooter_fatigue.modifier();
            let firepower_mod = shooter_formation.firepower_modifier();

            shots.push((
                shooter_entity,
                target_entity,
                distance,
                base_accuracy * distance_mod * morale_mod * fatigue_mod * firepower_mod,
            ));
        }
    }

    // Now apply all the shots
    for (shooter_entity, target_entity, distance, hit_chance) in shots {
        // Get mutable access to both entities
        if let Ok([
            (_, _, mut shooter_weapon, _, _, _, _, _, _),
            (_, _, _, target_formation, mut target_morale, _, _, _, mut target_squad),
        ]) = query.get_many_mut([shooter_entity, target_entity]) {

            let formation_mod = target_formation.target_profile_modifier();
            let final_hit_chance = hit_chance * formation_mod;

            // Volley fire using Poisson distribution
            let shots_fired = target_squad.size as f32 * 0.5; // Assume half can fire
            let expected_hits = shots_fired * final_hit_chance;

            let hits = if expected_hits > 0.0 {
                // Use Poisson distribution for realistic variance
                let poisson = Poisson::new(expected_hits).unwrap();
                poisson.sample(&mut rng) as u32
            } else {
                0
            };

            if hits > 0 {
                // Apply casualties
                let casualties = (hits as f32 * 0.35) as u32; // 35% lethality
                target_squad.apply_casualties(casualties);

                // Morale impact
                let casualty_rate = casualties as f32 / target_squad.max_size as f32;
                target_morale.modify(-casualty_rate * 30.0);

                tracing::debug!(
                    "Ranged combat: {} shots, {} hits, {} casualties at {}m",
                    shots_fired,
                    hits,
                    casualties,
                    distance
                );
            }

            // Fire the weapon
            shooter_weapon.fire();
        }
    }
}

/// Melee combat resolution
pub struct MeleeResult {
    pub winner: Entity,
    pub attacker_casualties: u32,
    pub defender_casualties: u32,
}

/// Calculate combat power for melee
pub fn calculate_combat_power(
    squad: &Squad,
    morale: &Morale,
    fatigue: &Fatigue,
    formation: &Formation,
    combat_stats: &CombatStats,
    velocity: &Velocity,
    experience: &Experience,
    is_attacker: bool,
) -> f32 {
    let mut power = combat_stats.melee_skill;

    // Size matters
    power *= (squad.size as f32).sqrt();

    // Morale is critical
    power *= morale.current / 100.0;

    // Fatigue heavily impacts melee
    power *= 1.0 - (fatigue.current / 100.0) * 0.5;

    // Formation modifier
    power *= if is_attacker {
        formation.melee_attack_modifier()
    } else {
        formation.melee_defense_modifier()
    };

    // Charge bonus for attacker
    if is_attacker {
        let speed_ratio = velocity.speed / 1.0;
        if speed_ratio > 2.0 {
            power *= 1.0 + (speed_ratio - 2.0) * 0.3;
        }
    }

    // Experience modifier
    power *= experience.combat_modifier();

    power
}

/// Melee combat system
pub fn melee_combat_system(
    spatial: Res<SpatialIndex>,
    mut query: Query<(
        Entity,
        &Position,
        &Velocity,
        &mut Squad,
        &Formation,
        &CombatStats,
        &mut Morale,
        &Fatigue,
        &Experience,
        &Team,
    )>,
) {
    let mut rng = rand::thread_rng();
    let melee_range = 5.0; // 5 meters

    // Collect all units that might be in melee
    let mut melee_pairs = Vec::new();

    for (entity, pos, _, _, _, _, _, _, _, team) in query.iter() {
        let nearby = spatial.grid.query_radius(pos.x, pos.y, melee_range);

        for other_entity in nearby {
            if entity == other_entity {
                continue;
            }

            if let Ok((_, other_pos, _, _, _, _, _, _, _, other_team)) = query.get(other_entity) {
                if team.is_enemy(other_team) {
                    let distance = pos.distance(other_pos);
                    if distance <= melee_range {
                        // Only add each pair once
                        if entity.index() < other_entity.index() {
                            melee_pairs.push((entity, other_entity, distance));
                        }
                    }
                }
            }
        }
    }

    // Resolve melee combats
    for (attacker_entity, defender_entity, _distance) in melee_pairs {
        if let Ok([
            (_, _, attacker_vel, mut attacker_squad, attacker_form, attacker_stats,
             mut attacker_morale, attacker_fatigue, attacker_exp, _),
            (_, _, defender_vel, mut defender_squad, defender_form, defender_stats,
             mut defender_morale, defender_fatigue, defender_exp, _),
        ]) = query.get_many_mut([attacker_entity, defender_entity]) {

            // Calculate combat powers
            let attacker_power = calculate_combat_power(
                &attacker_squad,
                &attacker_morale,
                attacker_fatigue,
                attacker_form,
                attacker_stats,
                attacker_vel,
                attacker_exp,
                true,
            );

            let defender_power = calculate_combat_power(
                &defender_squad,
                &defender_morale,
                defender_fatigue,
                defender_form,
                defender_stats,
                defender_vel,
                defender_exp,
                false,
            );

            // Opposed rolls
            let attacker_roll = rng.gen::<f32>() * attacker_power;
            let defender_roll = rng.gen::<f32>() * defender_power;

            let margin = attacker_roll - defender_roll;

            // Calculate casualties based on margin of victory
            let casualty_factor = (margin.abs() / attacker_power.max(defender_power)).min(0.5);

            if margin > 0.0 {
                // Attacker wins
                let defender_casualties = (defender_squad.size as f32 * casualty_factor * 0.1) as u32;
                let attacker_casualties = (attacker_squad.size as f32 * casualty_factor * 0.05) as u32;

                defender_squad.apply_casualties(defender_casualties);
                attacker_squad.apply_casualties(attacker_casualties);

                // Morale impacts
                defender_morale.modify(-25.0 * casualty_factor);
                attacker_morale.modify(15.0 * casualty_factor);

                tracing::debug!(
                    "Melee: Attacker wins (margin: {:.1}), casualties: A:{} D:{}",
                    margin,
                    attacker_casualties,
                    defender_casualties
                );
            } else {
                // Defender wins
                let attacker_casualties = (attacker_squad.size as f32 * casualty_factor * 0.1) as u32;
                let defender_casualties = (defender_squad.size as f32 * casualty_factor * 0.05) as u32;

                attacker_squad.apply_casualties(attacker_casualties);
                defender_squad.apply_casualties(defender_casualties);

                // Morale impacts
                attacker_morale.modify(-25.0 * casualty_factor);
                defender_morale.modify(15.0 * casualty_factor);

                tracing::debug!(
                    "Melee: Defender wins (margin: {:.1}), casualties: A:{} D:{}",
                    margin.abs(),
                    attacker_casualties,
                    defender_casualties
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_modifier() {
        let effective_range = 50.0;

        // Point blank
        assert!((calculate_distance_modifier(25.0, effective_range) - 1.0).abs() < 0.01);

        // Effective range
        assert!((calculate_distance_modifier(50.0, effective_range) - 0.7).abs() < 0.1);

        // Double range
        assert!((calculate_distance_modifier(100.0, effective_range) - 0.16).abs() < 0.05);

        // Very far
        assert!(calculate_distance_modifier(200.0, effective_range) < 0.05);
    }

    #[test]
    fn test_morale_modifier() {
        let high_morale = Morale::new(100.0);
        let med_morale = Morale::new(50.0);
        let low_morale = Morale::new(25.0);

        assert!((calculate_morale_modifier(&high_morale) - 1.0).abs() < 0.01);
        assert!((calculate_morale_modifier(&med_morale) - 0.707).abs() < 0.01);
        assert!((calculate_morale_modifier(&low_morale) - 0.5).abs() < 0.01);
    }
}
