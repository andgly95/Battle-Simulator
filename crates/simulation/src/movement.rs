//! Movement and pathfinding systems

use bevy_ecs::prelude::*;
use crate::components::*;

/// Movement system - applies velocity to position
pub fn movement_system(
    mut query: Query<(&mut Position, &Velocity, &Formation, &Fatigue)>,
    time: Res<SimulationTime>,
) {
    let dt = time.delta();

    for (mut pos, vel, formation, fatigue) in query.iter_mut() {
        if vel.speed > 0.0 {
            // Apply formation and fatigue modifiers
            let speed_modifier = formation.speed_modifier() * fatigue.modifier();
            let effective_speed = vel.speed * speed_modifier;

            // Update position
            pos.x += vel.dx * effective_speed * dt;
            pos.y += vel.dy * effective_speed * dt;
        }
    }
}

/// Destination-seeking system - moves units toward their destination
pub fn destination_system(
    mut query: Query<(&Position, &mut Velocity, &Destination, &Formation, &AIState)>,
) {
    for (pos, mut vel, dest, formation, ai_state) in query.iter_mut() {
        // Don't move if routing or reforming
        if matches!(ai_state.state, BehaviorState::Routing | BehaviorState::Reforming) {
            continue;
        }

        if dest.reached(pos) {
            // Arrived - stop moving
            vel.dx = 0.0;
            vel.dy = 0.0;
            vel.speed = 0.0;
        } else {
            // Calculate direction to destination
            let dx = dest.x - pos.x;
            let dy = dest.y - pos.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance > 0.0 {
                // Normalize direction
                vel.dx = dx / distance;
                vel.dy = dy / distance;

                // Set speed based on order
                vel.speed = match ai_state.current_order {
                    Order::Advance => 1.2,    // Walk
                    Order::Attack(_) => 2.5,  // Run/charge
                    Order::Retreat => 2.0,    // Quick march
                    _ => 1.0,                  // Default walk
                };
            }
        }
    }
}

/// Fatigue accumulation from movement
pub fn fatigue_system(
    mut query: Query<(&Velocity, &mut Fatigue, &AIState)>,
    time: Res<SimulationTime>,
) {
    let dt = time.delta();

    for (vel, mut fatigue, ai_state) in query.iter_mut() {
        if vel.speed > 0.0 {
            // Accumulate fatigue based on movement speed
            let fatigue_rate = if vel.speed > 2.0 {
                0.05  // Running
            } else if vel.speed > 1.5 {
                0.02  // Quick march
            } else {
                0.01  // Walking
            };

            fatigue.add(fatigue_rate * dt);
        } else if matches!(ai_state.state, BehaviorState::Idle) {
            // Recover when idle
            fatigue.recover(dt);
        }
    }
}

/// Simulation time resource
#[derive(Resource)]
pub struct SimulationTime {
    elapsed: f32,
    delta: f32,
}

impl SimulationTime {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            delta: 0.0,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.delta = dt;
        self.elapsed += dt;
    }

    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }
}

impl Default for SimulationTime {
    fn default() -> Self {
        Self::new()
    }
}
