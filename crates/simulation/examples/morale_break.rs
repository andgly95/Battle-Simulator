//! Extended firefight to demonstrate morale breaking and routing
//!
//! This scenario runs a prolonged engagement to show the full combat cycle:
//! - Sustained casualties over time
//! - Morale degradation under fire
//! - Units reaching breaking point and routing
//! - Army collapse when 2+ units rout
//!
//! Run with: cargo run --example morale_break

use bevy_ecs::prelude::*;
use battle_sim_core::spatial::SpatialIndex;
use battle_sim_simulation::components::*;
use battle_sim_simulation::*;

fn main() {
    // Initialize logging with INFO level to see major events
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("==============================================");
    tracing::info!("  BATTLE SIMULATOR - MORALE BREAK SCENARIO");
    tracing::info!("==============================================");
    tracing::info!("");

    // Create ECS world
    let mut world = World::new();

    // Insert resources
    world.insert_resource(SpatialIndex::new(100.0));
    world.insert_resource(SimulationTime::new());
    world.insert_resource(BattleStatistics::default());

    // Create schedule
    let mut schedule = Schedule::default();

    // Add systems in order
    schedule.add_systems((
        // Weapon reload
        weapon_reload_system,
        // Movement phase
        destination_system,
        movement_system,
        fatigue_system,
        // Spatial update
        spatial_index_update_system,
        // Combat phase
        ranged_combat_system,
        melee_combat_system,
        // Morale phase
        morale_check_system,
        routing_behavior_system,
        // Statistics
        statistics_system,
        status_report_system,
    ).chain());

    // Spawn French forces - 4 battalions in LINE formation
    // Starting at y=300, facing north
    // Lower starting morale (70) to see breaking point faster
    tracing::info!("FRENCH FORCES (L'Armée):");
    spawn_unit(&mut world, "French 1st Line Infantry", 50.0, 300.0, Side::Allied, FormationType::Line, 70.0);
    spawn_unit(&mut world, "French 2nd Line Infantry", 150.0, 300.0, Side::Allied, FormationType::Line, 70.0);
    spawn_unit(&mut world, "French 3rd Line Infantry", 250.0, 300.0, Side::Allied, FormationType::Line, 70.0);
    spawn_unit(&mut world, "French 4th Line Infantry", 350.0, 300.0, Side::Allied, FormationType::Line, 70.0);
    tracing::info!("  • 4 battalions (~3,200 men)");
    tracing::info!("  • LINE formation (maximum firepower)");
    tracing::info!("  • Position: y=300m");
    tracing::info!("  • Morale: 70 (Trained troops)");
    tracing::info!("");

    // Spawn British forces - 4 battalions in LINE formation
    // Starting at y=375 (75m apart = brutal range!)
    tracing::info!("BRITISH FORCES (The Red Coats):");
    spawn_unit(&mut world, "British 1st Regiment", 50.0, 375.0, Side::Enemy, FormationType::Line, 70.0);
    spawn_unit(&mut world, "British 2nd Regiment", 150.0, 375.0, Side::Enemy, FormationType::Line, 70.0);
    spawn_unit(&mut world, "British 3rd Regiment", 250.0, 375.0, Side::Enemy, FormationType::Line, 70.0);
    spawn_unit(&mut world, "British 4th Regiment", 350.0, 375.0, Side::Enemy, FormationType::Line, 70.0);
    tracing::info!("  • 4 battalions (~3,200 men)");
    tracing::info!("  • LINE formation (maximum firepower)");
    tracing::info!("  • Position: y=375m");
    tracing::info!("  • Morale: 70 (Regular troops)");
    tracing::info!("");

    tracing::info!("INITIAL SEPARATION: 75 meters");
    tracing::info!("  • POINT BLANK RANGE!");
    tracing::info!("  • Within optimal effective range (50m)");
    tracing::info!("  • Expect catastrophic casualties!");
    tracing::info!("  • First side to break will rout...");
    tracing::info!("");

    // Give orders: Both sides HOLD and fire
    let mut query = world.query::<(&mut AIState,)>();
    for (mut ai_state,) in query.iter_mut(&mut world) {
        ai_state.current_order = Order::Hold;
    }

    tracing::info!("==============================================");
    tracing::info!("              BATTLE COMMENCES!");
    tracing::info!("==============================================");
    tracing::info!("");

    // Run simulation - 10 MINUTES of combat
    let dt = 1.0 / 30.0; // 30 ticks per second
    let max_ticks = 18000; // 600 seconds = 10 minutes

    for tick in 0..max_ticks {
        // Update time
        let mut time = world.resource_mut::<SimulationTime>();
        time.tick(dt);
        let current_time = time.elapsed();
        drop(time);

        // Run all systems
        schedule.run(&mut world);

        // Check for battle end conditions
        let stats = world.resource::<BattleStatistics>();

        // Battle ends if 2+ units routing (army breaking)
        if stats.routing_units >= 2 {
            tracing::warn!("");
            tracing::warn!("==============================================");
            tracing::warn!("    BATTLE ENDED - ARMY HAS BROKEN!");
            tracing::warn!("==============================================");
            tracing::warn!("Time: {:.1} seconds ({} ticks)", current_time, tick);
            break;
        }

        // Or if casualties exceed 50%
        if stats.total_casualties > 3200 {
            tracing::warn!("");
            tracing::warn!("==============================================");
            tracing::warn!("   BATTLE ENDED - CATASTROPHIC CASUALTIES!");
            tracing::warn!("==============================================");
            tracing::warn!("Time: {:.1} seconds ({} ticks)", current_time, tick);
            break;
        }

        // Check if all units on one side are destroyed/routing
        let mut french_active = 0;
        let mut british_active = 0;
        let mut query = world.query::<(&UnitIdentity, &Squad, &AIState)>();
        for (identity, squad, ai_state) in query.iter(&world) {
            if squad.size > 0 && ai_state.state != BehaviorState::Routing {
                if identity.nation == "French" {
                    french_active += 1;
                } else {
                    british_active += 1;
                }
            }
        }

        if french_active == 0 || british_active == 0 {
            tracing::warn!("");
            tracing::warn!("==============================================");
            tracing::warn!("    BATTLE ENDED - TOTAL VICTORY!");
            tracing::warn!("==============================================");
            tracing::warn!("Time: {:.1} seconds ({} ticks)", current_time, tick);
            break;
        }
    }

    // Final statistics
    let stats = world.resource::<BattleStatistics>();
    let time = world.resource::<SimulationTime>();

    tracing::info!("");
    tracing::info!("==============================================");
    tracing::info!("         FINAL BATTLE REPORT");
    tracing::info!("==============================================");
    tracing::info!("Duration: {:.1} seconds", time.elapsed());
    tracing::info!("Total casualties: {} ({:.1}%)",
        stats.total_casualties,
        (stats.total_casualties as f32 / 6400.0) * 100.0
    );
    tracing::info!("Routing units: {} / {}", stats.routing_units, stats.total_units);
    tracing::info!("");

    // Determine victor
    let mut french_strength = 0u32;
    let mut british_strength = 0u32;
    let mut french_routing = 0;
    let mut british_routing = 0;

    let mut query = world.query::<(&UnitIdentity, &Squad, &AIState)>();
    for (identity, squad, ai_state) in query.iter(&world) {
        if identity.nation == "French" {
            french_strength += squad.size;
            if ai_state.state == BehaviorState::Routing {
                french_routing += 1;
            }
        } else {
            british_strength += squad.size;
            if ai_state.state == BehaviorState::Routing {
                british_routing += 1;
            }
        }
    }

    tracing::info!("FRENCH ARMY:");
    tracing::info!("  Remaining strength: {} men ({:.1}%)",
        french_strength,
        (french_strength as f32 / 3200.0) * 100.0
    );
    tracing::info!("  Routing battalions: {}", french_routing);
    tracing::info!("");

    tracing::info!("BRITISH ARMY:");
    tracing::info!("  Remaining strength: {} men ({:.1}%)",
        british_strength,
        (british_strength as f32 / 3200.0) * 100.0
    );
    tracing::info!("  Routing battalions: {}", british_routing);
    tracing::info!("");

    // Declare victor
    if french_routing >= 2 {
        tracing::info!("🇬🇧 BRITISH VICTORY! The French army has broken!");
    } else if british_routing >= 2 {
        tracing::info!("🇫🇷 FRENCH VICTORY! The British line has shattered!");
    } else if french_strength > british_strength * 2 {
        tracing::info!("🇫🇷 FRENCH DECISIVE VICTORY!");
    } else if british_strength > french_strength * 2 {
        tracing::info!("🇬🇧 BRITISH DECISIVE VICTORY!");
    } else if french_strength > british_strength {
        tracing::info!("🇫🇷 FRENCH TACTICAL VICTORY (higher casualties inflicted)");
    } else if british_strength > french_strength {
        tracing::info!("🇬🇧 BRITISH TACTICAL VICTORY (higher casualties inflicted)");
    } else {
        tracing::info!("⚔️  DRAW - Both armies fought to a standstill!");
    }

    tracing::info!("");
    tracing::info!("==============================================");
    tracing::info!("           UNIT-BY-UNIT BREAKDOWN");
    tracing::info!("==============================================");

    // Detailed unit report
    let mut query = world.query::<(&UnitIdentity, &Squad, &Morale, &AIState, &Position)>();
    for (identity, squad, morale, ai_state, pos) in query.iter(&world) {
        let strength_pct = (squad.size as f32 / squad.max_size as f32) * 100.0;
        let status = match ai_state.state {
            BehaviorState::Routing => "🏃 ROUTING",
            BehaviorState::Idle => "✓ STEADY",
            _ => "⚔ ACTIVE",
        };

        tracing::info!(
            "{}: {} men ({:.0}%), morale: {:.1}, status: {}, casualties: {}, pos: ({:.0}, {:.0})",
            identity.name,
            squad.size,
            strength_pct,
            morale.current,
            status,
            squad.casualties,
            pos.x,
            pos.y
        );
    }

    tracing::info!("");
    tracing::info!("==============================================");
    tracing::info!("         BATTLE SIMULATION COMPLETE");
    tracing::info!("==============================================");
}

/// Helper function to spawn a unit with specific parameters
fn spawn_unit(
    world: &mut World,
    name: &str,
    x: f32,
    y: f32,
    side: Side,
    formation: FormationType,
    morale: f32,
) -> Entity {
    let nation = if matches!(side, Side::Allied) { "French" } else { "British" };

    world.spawn((
        UnitIdentity {
            id: name.to_string(),
            name: name.to_string(),
            nation: nation.to_string(),
            unit_type: UnitType::Infantry,
        },
        Position::new(x, y),
        Velocity::zero(),
        Facing::new(0.0),
        Squad::new(800),
        Formation::new(formation, 800),
        Weapon {
            weapon_id: "musket".to_string(),
            weapon_type: WeaponType::Musket,
            range: 200.0,
            effective_range: 50.0,
            accuracy: 0.4,
            reload_time: 15.0,
            time_since_fire: 15.0,  // Ready to fire immediately!
            ammunition: 60,
            max_ammunition: 60,
        },
        CombatStats::new(70.0, 65.0, 60.0),
        Morale::new(morale),
        Fatigue::new(),
        Experience::new(ExperienceLevel::Veteran),  // Veteran troops
        AIState::new(),
        Destination::new(x, y),  // Hold position
        Team::new(0, side),
    )).id()
}
