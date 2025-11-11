//! Visual battle simulator - renders Column vs Line engagement
//!
//! Run with: cargo run --example visual_battle --features="renderer"
//!
//! Controls:
//! - Mouse drag: Pan camera
//! - Mouse wheel: Zoom
//! - Space: Pause/Resume
//! - R: Reset camera

use bevy_ecs::prelude::*;
use battle_sim_core::spatial::SpatialIndex;
use battle_sim_simulation::components::*;
use battle_sim_simulation::*;
use battle_sim_renderer::{PixelRenderer, colors};
use winit::{
    event::{Event, WindowEvent, ElementState, MouseButton, MouseScrollDelta},
    event_loop::{EventLoop, ControlFlow},
    dpi::{PhysicalPosition, LogicalSize},
};
use glam::vec2;
use std::time::{Instant, Duration};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("==============================================");
    tracing::info!("    VISUAL BATTLE SIMULATOR");
    tracing::info!("      Column vs Line Engagement");
    tracing::info!("==============================================");

    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window_attributes = winit::window::Window::default_attributes()
        .with_title("Battle Simulator - Column vs Line")
        .with_inner_size(LogicalSize::new(1280, 720));
    let window = event_loop.create_window(window_attributes).unwrap();

    // Create renderer
    let mut renderer = PixelRenderer::new(&window).unwrap();

    // Create ECS world
    let mut world = World::new();

    // Insert resources
    world.insert_resource(SpatialIndex::new(100.0));
    world.insert_resource(SimulationTime::new());
    world.insert_resource(BattleStatistics::default());

    // Create schedule
    let mut schedule = Schedule::default();
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

    // Spawn British forces in LINE formation (holding)
    tracing::info!("Deploying British line at y=400m...");
    spawn_unit(&mut world, "British 95th Rifles", 50.0, 400.0, 50.0, 400.0,
                Side::Enemy, FormationType::Line, 80.0);
    spawn_unit(&mut world, "British 52nd Light Infantry", 150.0, 400.0, 150.0, 400.0,
                Side::Enemy, FormationType::Line, 80.0);
    spawn_unit(&mut world, "British 1st Foot Guards", 250.0, 400.0, 250.0, 400.0,
                Side::Enemy, FormationType::Line, 85.0);

    // Spawn French forces in COLUMN formation (advancing)
    tracing::info!("Deploying French columns at y=200m...");
    spawn_unit(&mut world, "French 9e Légère", 50.0, 200.0, 50.0, 395.0,
                Side::Allied, FormationType::Column, 75.0);
    spawn_unit(&mut world, "French 45e Ligne", 150.0, 200.0, 150.0, 395.0,
                Side::Allied, FormationType::Column, 75.0);
    spawn_unit(&mut world, "French 54e Ligne", 250.0, 200.0, 250.0, 395.0,
                Side::Allied, FormationType::Column, 75.0);

    tracing::info!("");
    tracing::info!("Controls:");
    tracing::info!("  Mouse drag: Pan camera");
    tracing::info!("  Mouse wheel: Zoom");
    tracing::info!("  Space: Pause/Resume");
    tracing::info!("  R: Reset camera");
    tracing::info!("");

    // Simulation state
    let mut last_sim_tick = Instant::now();
    let sim_rate = Duration::from_millis(33); // 30 Hz simulation
    let mut paused = false;

    // Mouse state for camera control
    let mut mouse_pressed = false;
    let mut last_mouse_pos: Option<PhysicalPosition<f64>> = None;

    // Main loop
    let mut frame_count = 0u64;
    let mut fps_timer = Instant::now();

    event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    tracing::info!("Window closed");
                    target.exit();
                }

                WindowEvent::Resized(size) => {
                    renderer.resize(size.width, size.height);
                }

                WindowEvent::KeyboardInput { event, .. } => {
                    if event.state == ElementState::Pressed {
                        match event.physical_key {
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space) => {
                                paused = !paused;
                                tracing::info!("Simulation {}", if paused { "PAUSED" } else { "RESUMED" });
                            }
                            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyR) => {
                                renderer.camera_mut().look_at(vec2(150.0, 300.0));
                                renderer.camera_mut().zoom = 1.5;
                                tracing::info!("Camera reset");
                            }
                            _ => {}
                        }
                    }
                }

                WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                    mouse_pressed = state == ElementState::Pressed;
                    if !mouse_pressed {
                        last_mouse_pos = None;
                    }
                }

                WindowEvent::CursorMoved { position, .. } => {
                    if mouse_pressed {
                        if let Some(last_pos) = last_mouse_pos {
                            let delta = vec2(
                                (position.x - last_pos.x) as f32,
                                (position.y - last_pos.y) as f32,
                            );
                            renderer.camera_mut().pan(delta);
                        }
                        last_mouse_pos = Some(position);
                    }
                }

                WindowEvent::MouseWheel { delta, .. } => {
                    let zoom_factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            if y > 0.0 { 1.1 } else { 0.9 }
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            if pos.y > 0.0 { 1.05 } else { 0.95 }
                        }
                    };
                    renderer.camera_mut().zoom_center(zoom_factor);
                }

                WindowEvent::RedrawRequested => {
                    // Run simulation ticks if not paused
                    if !paused {
                        while last_sim_tick.elapsed() >= sim_rate {
                            schedule.run(&mut world);
                            last_sim_tick += sim_rate;
                        }
                    }

                    // Render frame
                    renderer.begin_frame();

                    // Draw grid
                    renderer.draw_grid(100.0);

                    // Draw all units
                    let visible = renderer.visible_bounds();
                    let mut unit_query = world.query::<(&Position, &Team, &Squad, &AIState, &Morale)>();

                    for (pos, team, squad, ai_state, morale) in unit_query.iter(&world) {
                        // Cull units outside view
                        if !visible.contains(pos.x, pos.y) {
                            continue;
                        }

                        // Determine color based on team, state, and morale
                        let color = match (team.side, ai_state.state, morale.current) {
                            (Side::Allied, BehaviorState::Routing, _) => colors::BLUE_ROUTING,
                            (Side::Allied, _, m) if m < 30.0 => colors::WAVERING,
                            (Side::Allied, _, _) => colors::BLUE_ACTIVE,
                            (Side::Enemy, BehaviorState::Routing, _) => colors::RED_ROUTING,
                            (Side::Enemy, _, m) if m < 30.0 => colors::WAVERING,
                            (Side::Enemy, _, _) => colors::RED_ACTIVE,
                        };

                        // Size based on unit strength
                        let size = (squad.size as f32 / squad.max_size as f32 * 8.0 + 2.0) as u32;

                        renderer.draw_unit(pos.x, pos.y, color, size.max(2));
                    }

                    renderer.end_frame().unwrap();

                    // FPS counter
                    frame_count += 1;
                    if fps_timer.elapsed() >= Duration::from_secs(1) {
                        let stats = world.resource::<BattleStatistics>();
                        let time = world.resource::<SimulationTime>();

                        tracing::info!(
                            "FPS: {} | Sim time: {:.1}s | Casualties: {} | Routing: {}",
                            frame_count,
                            time.elapsed(),
                            stats.total_casualties,
                            stats.routing_units
                        );

                        frame_count = 0;
                        fps_timer = Instant::now();
                    }
                }

                _ => {}
            }

            Event::AboutToWait => {
                window.request_redraw();
            }

            _ => {}
        }
    }).unwrap();
}

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
