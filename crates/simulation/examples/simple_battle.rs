//! Simple battle simulation example
//!
//! Run with: cargo run --example simple_battle

use bevy_ecs::prelude::*;
use battle_sim_core::spatial::SpatialIndex;
use battle_sim_simulation::components::*;
use battle_sim_simulation::movement::{SimulationTime, movement_system, destination_system, fatigue_system};
use battle_sim_simulation::combat::{ranged_combat_system, melee_combat_system};
use battle_sim_simulation::morale::{morale_check_system, routing_behavior_system};
use battle_sim_simulation::systems::*;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting simple battle simulation");

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

    // Spawn French units (Blue team)
    spawn_unit(&mut world, "French 1st Battalion", 100.0, 200.0, Side::Allied);
    spawn_unit(&mut world, "French 2nd Battalion", 150.0, 200.0, Side::Allied);
    spawn_unit(&mut world, "French 3rd Battalion", 200.0, 200.0, Side::Allied);

    // Spawn British units (Red team)
    spawn_unit(&mut world, "British 1st Battalion", 100.0, 600.0, Side::Enemy);
    spawn_unit(&mut world, "British 2nd Battalion", 150.0, 600.0, Side::Enemy);
    spawn_unit(&mut world, "British 3rd Battalion", 200.0, 600.0, Side::Enemy);

    // Give orders: French advance, British hold
    let mut query = world.query::<(&UnitIdentity, &mut Destination, &mut AIState)>();
    for (identity, mut dest, mut ai_state) in query.iter_mut(&mut world) {
        if identity.nation == "French" {
            // French advance north
            dest.x = 150.0;
            dest.y = 400.0;
            ai_state.current_order = Order::Advance;
        } else {
            // British hold position
            ai_state.current_order = Order::Hold;
        }
    }

    tracing::info!("=== Battle Start ===");
    tracing::info!("French (3 battalions, ~2400 men) advancing north");
    tracing::info!("British (3 battalions, ~2400 men) holding position");
    tracing::info!("");

    // Run simulation
    let dt = 1.0 / 30.0; // 30 ticks per second
    let max_ticks = 1800; // 60 seconds

    for tick in 0..max_ticks {
        // Update time
        let mut time = world.resource_mut::<SimulationTime>();
        time.tick(dt);
        drop(time);

        // Run all systems
        schedule.run(&mut world);

        // Check for battle end
        let stats = world.resource::<BattleStatistics>();
        if stats.routing_units >= 3 {
            tracing::info!("=== Battle Ended: One side has broken! ===");
            break;
        }
    }

    // Final statistics
    let stats = world.resource::<BattleStatistics>();
    let time = world.resource::<SimulationTime>();

    tracing::info!("");
    tracing::info!("=== Final Battle Report ===");
    tracing::info!("Duration: {:.1} seconds", time.elapsed());
    tracing::info!("Total units: {}", stats.total_units);
    tracing::info!("Routing units: {}", stats.routing_units);
    tracing::info!("Total casualties: {}", stats.total_casualties);

    // Unit-by-unit report
    let mut query = world.query::<(&UnitIdentity, &Squad, &Morale, &AIState, &Position)>();
    for (identity, squad, morale, ai_state, pos) in query.iter(&world) {
        tracing::info!(
            "{}: {} men remaining ({:.0}%), morale: {:.1}, state: {:?}, position: ({:.0}, {:.0})",
            identity.name,
            squad.size,
            (squad.size as f32 / squad.max_size as f32) * 100.0,
            morale.current,
            ai_state.state,
            pos.x,
            pos.y
        );
    }
}

/// Helper function to spawn a unit
fn spawn_unit(world: &mut World, name: &str, x: f32, y: f32, side: Side) -> Entity {
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
        Formation::new(FormationType::Line, 800),
        Weapon {
            weapon_id: "musket".to_string(),
            weapon_type: WeaponType::Musket,
            range: 200.0,
            effective_range: 50.0,
            accuracy: 0.4,
            reload_time: 15.0,
            time_since_fire: 0.0,
            ammunition: 60,
            max_ammunition: 60,
        },
        CombatStats::new(70.0, 65.0, 60.0),
        Morale::new(75.0),
        Fatigue::new(),
        Experience::new(ExperienceLevel::Trained),
        AIState::new(),
        Destination::new(x, y),
        Team::new(0, side),
    )).id()
}
