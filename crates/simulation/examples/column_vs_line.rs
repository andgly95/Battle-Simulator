//! Classic Napoleonic engagement: French columns charge British line
//!
//! This scenario demonstrates:
//! - Formation modifiers (Column vs Line)
//! - Movement and maneuver
//! - Transition from ranged to melee combat
//! - The tactical advantages of different formations
//!
//! Historical context:
//! - French columns: Better mobility, devastating melee charge
//! - British lines: Superior firepower, vulnerable to charges
//!
//! Run with: cargo run --example column_vs_line

use bevy_ecs::prelude::*;
use battle_sim_core::spatial::SpatialIndex;
use battle_sim_simulation::components::*;
use battle_sim_simulation::*;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("==============================================");
    tracing::info!("     COLUMN VS LINE - NAPOLEONIC TACTICS");
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
        weapon_reload_system,
        destination_system,
        movement_system,
        fatigue_system,
        spatial_index_update_system,
        ranged_combat_system,
        melee_combat_system,
        morale_check_system,
        routing_behavior_system,
        statistics_system,
        status_report_system,
    ).chain());

    // Spawn British forces - 3 battalions in LINE formation (defensive)
    // Starting at y=400, holding position
    tracing::info!("BRITISH FORCES (The Thin Red Line):");
    tracing::info!("  Deployed in LINE formation for maximum firepower");
    spawn_unit(&mut world, "British 95th Rifles", 50.0, 400.0, 50.0, 400.0,
                Side::Enemy, FormationType::Line, 80.0);
    spawn_unit(&mut world, "British 52nd Light Infantry", 150.0, 400.0, 150.0, 400.0,
                Side::Enemy, FormationType::Line, 80.0);
    spawn_unit(&mut world, "British 1st Foot Guards", 250.0, 400.0, 250.0, 400.0,
                Side::Enemy, FormationType::Line, 85.0);
    tracing::info!("  • 3 battalions (~2,400 men)");
    tracing::info!("  • Position: y=400m");
    tracing::info!("  • Tactic: HOLD and pour fire into advancing columns");
    tracing::info!("  • Formation advantages: +20% firepower, +20% target size");
    tracing::info!("");

    // Spawn French forces - 3 battalions in COLUMN formation (attacking)
    // Starting at y=200, will advance to y=400
    tracing::info!("FRENCH FORCES (L'Attaque à la Colonne):");
    tracing::info!("  Formed in COLUMN for assault");
    spawn_unit(&mut world, "French 9e Légère", 50.0, 200.0, 50.0, 395.0,
                Side::Allied, FormationType::Column, 75.0);
    spawn_unit(&mut world, "French 45e Ligne", 150.0, 200.0, 150.0, 395.0,
                Side::Allied, FormationType::Column, 75.0);
    spawn_unit(&mut world, "French 54e Ligne", 250.0, 200.0, 250.0, 395.0,
                Side::Allied, FormationType::Column, 75.0);
    tracing::info!("  • 3 battalions (~2,400 men)");
    tracing::info!("  • Position: y=200m");
    tracing::info!("  • Tactic: ADVANCE and charge with bayonets!");
    tracing::info!("  • Formation advantages: +10% speed, +30% melee attack");
    tracing::info!("");

    tracing::info!("INITIAL SEPARATION: 200 meters");
    tracing::info!("  • French must advance under fire");
    tracing::info!("  • British will fire volleys as they approach");
    tracing::info!("  • Columns vulnerable to musket fire (large target)");
    tracing::info!("  • If French reach melee, columns have advantage!");
    tracing::info!("");

    // French orders: Advance and attack
    let mut query = world.query::<(&UnitIdentity, &mut AIState, &mut Destination)>();
    for (identity, mut ai_state, mut dest) in query.iter_mut(&mut world) {
        if identity.nation == "French" {
            ai_state.current_order = Order::Advance;
            // Destinations already set in spawn_unit
        } else {
            ai_state.current_order = Order::Hold;
        }
    }

    tracing::info!("==============================================");
    tracing::info!("         THE ASSAULT BEGINS!");
    tracing::info!("==============================================");
    tracing::info!("");

    // Run simulation
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
            tracing::warn!("Time: {:.1} seconds", current_time);
            break;
        }

        // Or if casualties exceed 50%
        if stats.total_casualties > 2400 {
            tracing::warn!("");
            tracing::warn!("==============================================");
            tracing::warn!("   BATTLE ENDED - CATASTROPHIC CASUALTIES!");
            tracing::warn!("==============================================");
            tracing::warn!("Time: {:.1} seconds", current_time);
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
            tracing::warn!("Time: {:.1} seconds", current_time);
            break;
        }

        // Report when French columns get close
        if tick == 1500 { // ~50 seconds
            tracing::info!("");
            tracing::info!(">>> French columns advancing under fire...");
            tracing::info!("");
        }

        if tick == 3000 { // ~100 seconds
            tracing::info!("");
            tracing::info!(">>> Columns approaching musket range...");
            tracing::info!("");
        }

        if tick == 4500 { // ~150 seconds
            tracing::info!("");
            tracing::info!(">>> British volleys intensify! Columns taking heavy fire!");
            tracing::info!("");
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
        (stats.total_casualties as f32 / 4800.0) * 100.0
    );
    tracing::info!("Routing units: {} / {}", stats.routing_units, stats.total_units);
    tracing::info!("");

    // Analyze results
    let mut french_strength = 0u32;
    let mut british_strength = 0u32;
    let mut french_routing = 0;
    let mut british_routing = 0;
    let mut french_position_sum = 0.0;
    let mut french_count = 0;

    let mut query = world.query::<(&UnitIdentity, &Squad, &AIState, &Position)>();
    for (identity, squad, ai_state, pos) in query.iter(&world) {
        if identity.nation == "French" {
            french_strength += squad.size;
            french_position_sum += pos.y;
            french_count += 1;
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

    let avg_french_position = if french_count > 0 {
        french_position_sum / french_count as f32
    } else {
        0.0
    };

    let distance_advanced = avg_french_position - 200.0;

    tracing::info!("FRENCH ASSAULT:");
    tracing::info!("  Remaining strength: {} men ({:.1}%)",
        french_strength,
        (french_strength as f32 / 2400.0) * 100.0
    );
    tracing::info!("  Distance advanced: {:.0}m / 200m ({:.0}%)",
        distance_advanced,
        (distance_advanced / 200.0) * 100.0
    );
    tracing::info!("  Routing battalions: {}", french_routing);
    tracing::info!("");

    tracing::info!("BRITISH DEFENSE:");
    tracing::info!("  Remaining strength: {} men ({:.1}%)",
        british_strength,
        (british_strength as f32 / 2400.0) * 100.0
    );
    tracing::info!("  Routing battalions: {}", british_routing);
    tracing::info!("");

    // Determine victor and analyze tactics
    if french_routing >= 2 {
        tracing::info!("🇬🇧 BRITISH VICTORY! The French assault has been repulsed!");
        tracing::info!("   Analysis: British firepower broke the columns before they could close");
    } else if british_routing >= 2 {
        tracing::info!("🇫🇷 FRENCH VICTORY! The columns broke through!");
        tracing::info!("   Analysis: French columns reached melee and shattered the line");
    } else if distance_advanced >= 180.0 {
        if french_strength > british_strength {
            tracing::info!("🇫🇷 FRENCH TACTICAL VICTORY!");
            tracing::info!("   Analysis: Columns reached close combat and inflicted heavy casualties");
        } else {
            tracing::info!("⚔️  PYRRHIC VICTORY - Columns reached the line but at heavy cost");
        }
    } else if distance_advanced < 100.0 {
        tracing::info!("🇬🇧 DECISIVE BRITISH VICTORY!");
        tracing::info!("   Analysis: Devastating volleys stopped the assault cold");
    } else {
        if british_strength > french_strength {
            tracing::info!("🇬🇧 BRITISH TACTICAL VICTORY!");
            tracing::info!("   Analysis: Superior firepower won the day");
        } else {
            tracing::info!("⚔️  DRAW - Both sides fought valiantly");
        }
    }

    tracing::info!("");
    tracing::info!("==============================================");
    tracing::info!("           UNIT-BY-UNIT BREAKDOWN");
    tracing::info!("==============================================");

    // Detailed unit report
    let mut query = world.query::<(&UnitIdentity, &Squad, &Morale, &AIState, &Position, &Formation)>();
    for (identity, squad, morale, ai_state, pos, formation) in query.iter(&world) {
        let strength_pct = (squad.size as f32 / squad.max_size as f32) * 100.0;
        let status = match ai_state.state {
            BehaviorState::Routing => "🏃 ROUTING",
            BehaviorState::Moving => "⚔ ADVANCING",
            BehaviorState::Engaging => "⚔ ENGAGED",
            BehaviorState::Idle => "✓ STEADY",
            _ => "• ACTIVE",
        };

        let formation_name = match formation.formation_type {
            FormationType::Line => "LINE",
            FormationType::Column => "COLUMN",
            FormationType::Square => "SQUARE",
            FormationType::Skirmish => "SKIRMISH",
        };

        tracing::info!(
            "{} ({}): {} men ({:.0}%), morale: {:.1}, status: {}, position: {:.0}m, casualties: {}",
            identity.name,
            formation_name,
            squad.size,
            strength_pct,
            morale.current,
            status,
            pos.y,
            squad.casualties
        );
    }

    tracing::info!("");
    tracing::info!("==============================================");
    tracing::info!("     TACTICAL ANALYSIS");
    tracing::info!("==============================================");
    tracing::info!("");
    tracing::info!("Formation comparison:");
    tracing::info!("  British LINE formation:");
    tracing::info!("    • Firepower modifier: +20%");
    tracing::info!("    • Target size: +20% (easier to hit)");
    tracing::info!("    • Melee defense: -10%");
    tracing::info!("");
    tracing::info!("  French COLUMN formation:");
    tracing::info!("    • Firepower modifier: -60% (few men can fire)");
    tracing::info!("    • Target size: +50% (dense target)");
    tracing::info!("    • Melee attack: +30%");
    tracing::info!("    • Movement speed: +10%");
    tracing::info!("");
    tracing::info!("The classic tactical dilemma of Napoleonic warfare:");
    tracing::info!("  Can the columns close before the volleys break them?");
    tracing::info!("");
    tracing::info!("==============================================");
    tracing::info!("         SIMULATION COMPLETE");
    tracing::info!("==============================================");
}

/// Helper function to spawn a unit with specific parameters
fn spawn_unit(
    world: &mut World,
    name: &str,
    start_x: f32,
    start_y: f32,
    dest_x: f32,
    dest_y: f32,
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
        Position::new(start_x, start_y),
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
            time_since_fire: 15.0,
            ammunition: 60,
            max_ammunition: 60,
        },
        CombatStats::new(70.0, 65.0, 60.0),
        Morale::new(morale),
        Fatigue::new(),
        Experience::new(ExperienceLevel::Veteran),
        AIState::new(),
        Destination::new(dest_x, dest_y),
        Team::new(0, side),
    )).id()
}
