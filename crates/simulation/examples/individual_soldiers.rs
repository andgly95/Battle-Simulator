//! Individual soldier simulation - 4,800 separate entities!
//!
//! Each soldier:
//! - Has their own entity
//! - Moves independently
//! - Shoots individually  
//! - Can be hit specifically
//! - Actively reforms into position
//!
//! Run with: cargo run --example individual_soldiers

use bevy_ecs::prelude::*;
use battle_sim_core::spatial::SpatialIndex;
use battle_sim_simulation::components::*;
use battle_sim_simulation::*;
use battle_sim_renderer::{PixelRenderer, colors};
use winit::{
    event::{WindowEvent, ElementState, MouseButton, MouseScrollDelta, KeyEvent},
    event_loop::{EventLoop, ControlFlow, ActiveEventLoop},
    dpi::{PhysicalPosition, LogicalSize},
    application::ApplicationHandler,
    keyboard::{PhysicalKey, KeyCode},
};
use glam::{vec2, Vec2};
use std::time::{Instant, Duration};
use rand::Rng;

/// Individual soldier component
#[derive(Component, Debug, Clone)]
struct Soldier {
    battalion: Entity,
    rank: u32,  // Row in formation (0 = front)
    file: u32,  // Column in formation
    target_pos: Vec2,
    in_front_rank: bool,
}

/// Individual soldier weapon
#[derive(Component, Debug, Clone)]
struct SoldierWeapon {
    loaded: bool,
    reload_time: f32,
    time_since_fire: f32,
    ammunition: u32,
}

impl SoldierWeapon {
    fn new() -> Self {
        Self {
            loaded: true,
            reload_time: 15.0,
            time_since_fire: 15.0,
            ammunition: 60,
        }
    }
    
    fn can_fire(&self) -> bool {
        self.loaded && self.ammunition > 0
    }
    
    fn fire(&mut self) {
        self.loaded = false;
        self.ammunition -= 1;
        self.time_since_fire = 0.0;
    }
}

/// Projectile in flight
#[derive(Component, Debug, Clone)]
struct Projectile {
    velocity: Vec2,
    max_range: f32,
    distance: f32,
    team: Side,
}

/// Battalion commander - manages formation
#[derive(Component)]
struct Battalion {
    formation_type: FormationType,
    formation_center: Vec2,
}

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
        let mut world = World::new();
        
        world.insert_resource(SpatialIndex::new(100.0));
        world.insert_resource(SimulationTime::new());
        
        let mut schedule = Schedule::default();
        schedule.add_systems((
            assign_formation_positions,
            soldier_movement_system,
            soldier_weapon_reload_system,
            soldier_shooting_system,
            projectile_movement_system,
            projectile_hit_detection_system,
        ).chain());
        
        tracing::info!("==============================================");
        tracing::info!("  INDIVIDUAL SOLDIER SIMULATION");
        tracing::info!("  4,800 soldiers, each with their own AI!");
        tracing::info!("==============================================");

        // Spawn British line - 3 battalions × 800 soldiers = 2,400 individual entities
        tracing::info!("Spawning 2,400 British soldiers in LINE formation...");
        spawn_battalion(&mut world, vec2(-200.0, 300.0), Side::Enemy, FormationType::Line, 800);
        spawn_battalion(&mut world, vec2(200.0, 300.0), Side::Enemy, FormationType::Line, 800);
        spawn_battalion(&mut world, vec2(600.0, 300.0), Side::Enemy, FormationType::Line, 800);

        // Spawn French columns - 3 battalions × 800 soldiers = 2,400 individual entities
        tracing::info!("Spawning 2,400 French soldiers in COLUMN formation...");
        spawn_battalion(&mut world, vec2(-200.0, 150.0), Side::Allied, FormationType::Column, 800);
        spawn_battalion(&mut world, vec2(200.0, 150.0), Side::Allied, FormationType::Column, 800);
        spawn_battalion(&mut world, vec2(600.0, 150.0), Side::Allied, FormationType::Column, 800);

        tracing::info!("Total: 4,800 individual soldier entities created!");
        tracing::info!("");
        
        Self {
            window: None,
            renderer: None,
            world,
            schedule,
            last_sim_tick: Instant::now(),
            sim_rate: Duration::from_millis(33),
            paused: false,
            mouse_pressed: false,
            last_mouse_pos: None,
            frame_count: 0,
            fps_timer: Instant::now(),
        }
    }
    
    fn render(&mut self) {
        if let Some(ref mut renderer) = self.renderer {
            // Run simulation
            if !self.paused {
                while self.last_sim_tick.elapsed() >= self.sim_rate {
                    let dt = self.sim_rate.as_secs_f32();
                    let mut time = self.world.resource_mut::<SimulationTime>();
                    time.tick(dt);
                    drop(time);
                    
                    self.schedule.run(&mut self.world);
                    self.last_sim_tick += self.sim_rate;
                }
            }
            
            renderer.begin_frame();
            renderer.draw_grid(100.0);
            
            // Draw individual soldiers
            let visible = renderer.visible_bounds();
            let mut soldier_query = self.world.query::<(&Position, &Team)>();

            for (pos, team) in soldier_query.iter(&self.world) {
                if !visible.contains(pos.x, pos.y) {
                    continue;
                }
                
                let color = match team.side {
                    Side::Allied => colors::BLUE_ACTIVE,
                    Side::Enemy => colors::RED_ACTIVE,
                    Side::Neutral => colors::TEXT,
                };
                
                renderer.draw_unit(pos.x, pos.y, color, 2);
            }
            
            // Draw projectiles
            let mut proj_query = self.world.query::<(&Position, &Projectile)>();
            for (pos, proj) in proj_query.iter(&self.world) {
                let color = match proj.team {
                    Side::Allied => [100, 150, 255, 255],
                    Side::Enemy => [255, 150, 100, 255],
                    Side::Neutral => [200, 200, 200, 255],
                };
                renderer.draw_unit(pos.x, pos.y, color, 1);
            }
            
            renderer.end_frame().unwrap();
            
            self.frame_count += 1;
            if self.fps_timer.elapsed() >= Duration::from_secs(1) {
                let soldier_count = self.world.query::<&Soldier>().iter(&self.world).count();
                let proj_count = self.world.query::<&Projectile>().iter(&self.world).count();
                tracing::info!(
                    "FPS: {} | Soldiers: {} | Projectiles: {}",
                    self.frame_count,
                    soldier_count,
                    proj_count
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
            let attrs = winit::window::Window::default_attributes()
                .with_title("Individual Soldier Simulation - 4,800 entities")
                .with_inner_size(LogicalSize::new(1280, 720));
            let window = event_loop.create_window(attrs).unwrap();

            let mut renderer = unsafe {
                PixelRenderer::new(&*(&window as *const _)).unwrap()
            };

            // Position camera to see the battlefield
            // Center on battlefield: x spans -200 to 600, y spans 150 to 300
            renderer.camera_mut().position = vec2(200.0, 225.0);
            renderer.camera_mut().zoom = 1.0;

            self.renderer = Some(renderer);
            self.window = Some(window);
        }
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: winit::window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render(),

            WindowEvent::MouseInput { state, button, .. } => {
                if button == MouseButton::Left {
                    self.mouse_pressed = state == ElementState::Pressed;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    if let (Some(last_pos), Some(ref mut renderer)) = (self.last_mouse_pos, &mut self.renderer) {
                        let dx = position.x - last_pos.x;
                        let dy = position.y - last_pos.y;

                        let camera = renderer.camera_mut();
                        camera.position.x -= dx as f32 / camera.zoom;
                        camera.position.y += dy as f32 / camera.zoom;
                    }
                }
                self.last_mouse_pos = Some(position);
            }

            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(ref mut renderer) = self.renderer {
                    let zoom_factor = match delta {
                        MouseScrollDelta::LineDelta(_, y) => {
                            if y > 0.0 { 1.1 } else { 0.9 }
                        }
                        MouseScrollDelta::PixelDelta(pos) => {
                            if pos.y > 0.0 { 1.1 } else { 0.9 }
                        }
                    };

                    let camera = renderer.camera_mut();
                    camera.zoom = (camera.zoom * zoom_factor).clamp(0.1, 100.0);
                }
            }

            WindowEvent::KeyboardInput { event: KeyEvent { physical_key, state: ElementState::Pressed, .. }, .. } => {
                if let PhysicalKey::Code(KeyCode::Space) = physical_key {
                    self.paused = !self.paused;
                    tracing::info!("Simulation {}", if self.paused { "PAUSED" } else { "RESUMED" });
                }
            }

            _ => {}
        }
    }
    
    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }
}

fn spawn_battalion(world: &mut World, center: Vec2, side: Side, formation: FormationType, count: u32) -> Entity {
    // Create battalion commander
    let battalion = world.spawn((
        Battalion {
            formation_type: formation,
            formation_center: center,
        },
        Team::new(0, side),
    )).id();
    
    // Spawn individual soldiers
    match formation {
        FormationType::Line => {
            // 800 men in a single line
            let spacing = 2.0;
            let start_x = center.x - (count as f32 * spacing) / 2.0;
            
            for file in 0..count {
                let x = start_x + file as f32 * spacing;
                spawn_soldier(world, battalion, vec2(x, center.y), 0, file, side, true);
            }
        }
        
        FormationType::Column => {
            // 20 wide, 40 deep
            let width = 20;
            let spacing = 1.5;
            let ranks = (count as f32 / width as f32).ceil() as u32;
            
            let start_x = center.x - (width as f32 * spacing) / 2.0;
            let start_y = center.y - (ranks as f32 * spacing) / 2.0;
            
            for rank in 0..ranks {
                let men_in_rank = width.min(count - rank * width);
                for file in 0..men_in_rank {
                    let x = start_x + file as f32 * spacing;
                    let y = start_y + rank as f32 * spacing;
                    spawn_soldier(world, battalion, vec2(x, y), rank, file, side, rank == 0);
                }
            }
        }
        
        _ => {}
    }
    
    battalion
}

fn spawn_soldier(world: &mut World, battalion: Entity, pos: Vec2, rank: u32, file: u32, side: Side, front_rank: bool) {
    world.spawn((
        Soldier {
            battalion,
            rank,
            file,
            target_pos: pos,
            in_front_rank: front_rank,
        },
        Position::new(pos.x, pos.y),
        Velocity::zero(),
        Team::new(0, side),
        SoldierWeapon::new(),
    ));
}

// SYSTEMS

fn assign_formation_positions(
    battalions: Query<(Entity, &Battalion)>,
    mut soldiers: Query<(&mut Soldier, &Team)>,
) {
    for (batt_entity, battalion) in battalions.iter() {
        // Find all soldiers in this battalion and reassign positions
        // This handles gaps from casualties - survivors close ranks

        let mut soldier_list: Vec<(u32, u32)> = Vec::new(); // (rank, file)

        for (soldier, _) in soldiers.iter_mut() {
            if soldier.battalion == batt_entity {
                soldier_list.push((soldier.rank, soldier.file));
            }
        }

        // Sort by rank, then file
        soldier_list.sort();

        // Reassign formation positions to fill gaps
        let mut new_index = 0u32;
        for (mut soldier, _) in soldiers.iter_mut() {
            if soldier.battalion == batt_entity {
                match battalion.formation_type {
                    FormationType::Line => {
                        // Reassign as single line
                        let spacing = 2.0;
                        let total_width = soldier_list.len() as f32 * spacing;
                        let start_x = battalion.formation_center.x - total_width / 2.0;
                        let x = start_x + new_index as f32 * spacing;
                        soldier.target_pos = vec2(x, battalion.formation_center.y);
                        soldier.file = new_index;
                        soldier.rank = 0;
                        soldier.in_front_rank = true;
                    }

                    FormationType::Column => {
                        // Reassign as column (20 wide)
                        let width = 20;
                        let rank = new_index / width;
                        let file = new_index % width;

                        let spacing = 1.5;
                        let ranks_total = (soldier_list.len() as f32 / width as f32).ceil();

                        let start_x = battalion.formation_center.x - (width as f32 * spacing) / 2.0;
                        let start_y = battalion.formation_center.y - (ranks_total * spacing) / 2.0;

                        soldier.target_pos = vec2(
                            start_x + file as f32 * spacing,
                            start_y + rank as f32 * spacing,
                        );
                        soldier.rank = rank;
                        soldier.file = file;
                        soldier.in_front_rank = rank == 0;
                    }

                    _ => {}
                }

                new_index += 1;
            }
        }
    }
}

fn soldier_movement_system(
    mut query: Query<(&Soldier, &mut Position, &mut Velocity)>,
) {
    for (soldier, mut pos, mut vel) in query.iter_mut() {
        // Move towards target position
        let dx = soldier.target_pos.x - pos.x;
        let dy = soldier.target_pos.y - pos.y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist > 0.5 {
            // Move towards position
            vel.dx = dx / dist;
            vel.dy = dy / dist;
            vel.speed = 1.0;
            
            pos.x += vel.dx * vel.speed * 0.033;
            pos.y += vel.dy * vel.speed * 0.033;
        } else {
            vel.speed = 0.0;
        }
    }
}

fn soldier_weapon_reload_system(
    mut query: Query<&mut SoldierWeapon>,
    time: Res<SimulationTime>,
) {
    let dt = time.delta();
    for mut weapon in query.iter_mut() {
        weapon.time_since_fire += dt;
        if !weapon.loaded && weapon.time_since_fire >= weapon.reload_time {
            weapon.loaded = true;
        }
    }
}

fn soldier_shooting_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Soldier, &Position, &mut SoldierWeapon, &Team)>,
) {
    // Collect all soldiers first to avoid borrow issues
    let soldiers: Vec<_> = query.iter().map(|(e, s, p, w, t)| (e, s.clone(), *p, w.loaded, t.side)).collect();

    for (shooter_entity, shooter, shooter_pos, can_fire, shooter_team) in soldiers.iter() {
        // Only front rank can shoot
        if !shooter.in_front_rank || !can_fire {
            continue;
        }

        // Find nearest enemy in range
        let mut nearest_enemy: Option<(Vec2, f32)> = None;

        for (target_entity, _, target_pos, _, target_team) in soldiers.iter() {
            if *shooter_entity == *target_entity {
                continue;
            }

            if *shooter_team == *target_team {
                continue; // Same team
            }

            let dx = target_pos.x - shooter_pos.x;
            let dy = target_pos.y - shooter_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= 200.0 {
                if let Some((_, nearest_dist)) = nearest_enemy {
                    if dist < nearest_dist {
                        nearest_enemy = Some((vec2(target_pos.x, target_pos.y), dist));
                    }
                } else {
                    nearest_enemy = Some((vec2(target_pos.x, target_pos.y), dist));
                }
            }
        }

        if let Some((target, _)) = nearest_enemy {
            // FIRE!
            if let Ok((_, _, _, mut weapon, _)) = query.get_mut(*shooter_entity) {
                weapon.fire();

                // Spawn projectile
                let direction = (target - vec2(shooter_pos.x, shooter_pos.y)).normalize();
                commands.spawn((
                    Position::new(shooter_pos.x, shooter_pos.y),
                    Projectile {
                        velocity: direction * 400.0, // 400 m/s
                        max_range: 200.0,
                        distance: 0.0,
                        team: *shooter_team,
                    },
                ));
            }
        }
    }
}

fn projectile_movement_system(
    mut query: Query<(&mut Position, &mut Projectile)>,
    time: Res<SimulationTime>,
) {
    let dt = time.delta();
    for (mut pos, mut proj) in query.iter_mut() {
        pos.x += proj.velocity.x * dt;
        pos.y += proj.velocity.y * dt;
        proj.distance += proj.velocity.length() * dt;
    }
}

fn projectile_hit_detection_system(
    mut commands: Commands,
    projectiles: Query<(Entity, &Position, &Projectile)>,
    soldiers: Query<(Entity, &Position, &Team), With<Soldier>>,
) {
    for (proj_entity, proj_pos, proj) in projectiles.iter() {
        let mut hit = false;

        // Check if hit any soldier
        for (soldier_entity, soldier_pos, soldier_team) in soldiers.iter() {
            // Only hit enemies
            if proj.team == soldier_team.side {
                continue;
            }

            let dx = proj_pos.x - soldier_pos.x;
            let dy = proj_pos.y - soldier_pos.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 1.0 {
                // HIT!
                commands.entity(soldier_entity).despawn();
                commands.entity(proj_entity).despawn();
                hit = true;
                break;
            }
        }

        if hit {
            continue;
        }

        // Remove if out of range
        if proj.distance > proj.max_range {
            commands.entity(proj_entity).despawn();
        }
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    
    let mut app = BattleApp::new();
    event_loop.run_app(&mut app).unwrap();
}
