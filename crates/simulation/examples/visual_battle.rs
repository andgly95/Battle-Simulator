//! Visual battle simulator - renders Column vs Line engagement
//!
//! Run with: cargo run --example visual_battle
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
    event::{WindowEvent, ElementState, MouseButton, MouseScrollDelta},
    event_loop::{EventLoop, ControlFlow, ActiveEventLoop},
    dpi::{PhysicalPosition, LogicalSize},
    application::ApplicationHandler,
};
use glam::vec2;
use std::time::{Instant, Duration};

struct BattleApp<'a> {
    window: Option<winit::window::Window>,
    renderer: Option<PixelRenderer<'a>>,
    world: World,
    schedule: Schedule,
    last_sim_tick: Instant,
    sim_rate: Duration,
    paused: bool,
    mouse_pressed: bool,
    last_mouse_pos: Option<PhysicalPosition<f64>>,
    frame_count: u64,
    fps_timer: Instant,
}

impl<'a> BattleApp<'a> {
    fn new() -> Self {
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
        Self::spawn_unit(&mut world, "British 95th Rifles", 50.0, 400.0, 50.0, 400.0,
                    Side::Enemy, FormationType::Line, 80.0);
        Self::spawn_unit(&mut world, "British 52nd Light Infantry", 150.0, 400.0, 150.0, 400.0,
                    Side::Enemy, FormationType::Line, 80.0);
        Self::spawn_unit(&mut world, "British 1st Foot Guards", 250.0, 400.0, 250.0, 400.0,
                    Side::Enemy, FormationType::Line, 85.0);

        // Spawn French forces in COLUMN formation (advancing)
        tracing::info!("Deploying French columns at y=200m...");
        Self::spawn_unit(&mut world, "French 9e Légère", 50.0, 200.0, 50.0, 395.0,
                    Side::Allied, FormationType::Column, 75.0);
        Self::spawn_unit(&mut world, "French 45e Ligne", 150.0, 200.0, 150.0, 395.0,
                    Side::Allied, FormationType::Column, 75.0);
        Self::spawn_unit(&mut world, "French 54e Ligne", 250.0, 200.0, 250.0, 395.0,
                    Side::Allied, FormationType::Column, 75.0);

        tracing::info!("");
        tracing::info!("Controls:");
        tracing::info!("  Mouse drag: Pan camera");
        tracing::info!("  Mouse wheel: Zoom");
        tracing::info!("  Space: Pause/Resume");
        tracing::info!("  R: Reset camera");
        tracing::info!("");

        Self {
            window: None,
            renderer: None,
            world,
            schedule,
            last_sim_tick: Instant::now(),
            sim_rate: Duration::from_millis(33), // 30 Hz simulation
            paused: false,
            mouse_pressed: false,
            last_mouse_pos: None,
            frame_count: 0,
            fps_timer: Instant::now(),
        }
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

    fn render(&mut self) {
        if let Some(ref mut renderer) = self.renderer {
            // Run simulation ticks if not paused
            if !self.paused {
                while self.last_sim_tick.elapsed() >= self.sim_rate {
                    // Update simulation time
                    let dt = self.sim_rate.as_secs_f32();
                    let mut time = self.world.resource_mut::<SimulationTime>();
                    time.tick(dt);
                    drop(time);

                    // Run all systems
                    self.schedule.run(&mut self.world);
                    self.last_sim_tick += self.sim_rate;
                }
            }

            // Render frame
            renderer.begin_frame();

            // Draw grid
            renderer.draw_grid(100.0);

            // Draw all units
            let visible = renderer.visible_bounds();
            let mut unit_query = self.world.query::<(&Position, &Team, &Squad, &AIState, &Morale)>();

            for (pos, team, squad, ai_state, morale) in unit_query.iter(&self.world) {
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
                    (Side::Neutral, _, _) => colors::TEXT, // Gray for neutral
                };

                // Size based on unit strength
                let size = (squad.size as f32 / squad.max_size as f32 * 8.0 + 2.0) as u32;

                renderer.draw_unit(pos.x, pos.y, color, size.max(2));
            }

            renderer.end_frame().unwrap();

            // FPS counter
            self.frame_count += 1;
            if self.fps_timer.elapsed() >= Duration::from_secs(1) {
                let stats = self.world.resource::<BattleStatistics>();
                let time = self.world.resource::<SimulationTime>();

                tracing::info!(
                    "FPS: {} | Sim time: {:.1}s | Casualties: {} | Routing: {}",
                    self.frame_count,
                    time.elapsed(),
                    stats.total_casualties,
                    stats.routing_units
                );

                self.frame_count = 0;
                self.fps_timer = Instant::now();
            }
        }
    }
}

impl<'a> ApplicationHandler for BattleApp<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window_attributes = winit::window::Window::default_attributes()
                .with_title("Battle Simulator - Column vs Line")
                .with_inner_size(LogicalSize::new(1280, 720));

            let window = event_loop.create_window(window_attributes).unwrap();

            // Create renderer - unsafe is needed to extend window lifetime
            // This is safe because window and renderer have the same lifetime in the struct
            let renderer = unsafe {
                let window_ptr = &window as *const winit::window::Window;
                PixelRenderer::new(&*window_ptr).unwrap()
            };

            self.renderer = Some(renderer);
            self.window = Some(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                tracing::info!("Window closed");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(ref mut renderer) = self.renderer {
                    renderer.resize(size.width, size.height);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    match event.physical_key {
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Space) => {
                            self.paused = !self.paused;
                            tracing::info!("Simulation {}", if self.paused { "PAUSED" } else { "RESUMED" });
                        }
                        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyR) => {
                            if let Some(ref mut renderer) = self.renderer {
                                renderer.camera_mut().look_at(vec2(150.0, 300.0));
                                renderer.camera_mut().zoom = 1.5;
                                tracing::info!("Camera reset");
                            }
                        }
                        _ => {}
                    }
                }
            }

            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => {
                self.mouse_pressed = state == ElementState::Pressed;
                if !self.mouse_pressed {
                    self.last_mouse_pos = None;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    if let Some(last_pos) = self.last_mouse_pos {
                        let delta = vec2(
                            (position.x - last_pos.x) as f32,
                            (position.y - last_pos.y) as f32,
                        );
                        if let Some(ref mut renderer) = self.renderer {
                            renderer.camera_mut().pan(delta);
                        }
                    }
                    self.last_mouse_pos = Some(position);
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
                if let Some(ref mut renderer) = self.renderer {
                    renderer.camera_mut().zoom_center(zoom_factor);
                }
            }

            WindowEvent::RedrawRequested => {
                self.render();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("==============================================");
    tracing::info!("    VISUAL BATTLE SIMULATOR");
    tracing::info!("      Column vs Line Engagement");
    tracing::info!("==============================================");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = BattleApp::new();

    event_loop.run_app(&mut app).unwrap();
}
