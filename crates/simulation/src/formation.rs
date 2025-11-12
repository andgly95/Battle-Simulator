//! Formation coherence and reformation system

use bevy_ecs::prelude::*;
use crate::components::*;
use crate::movement::SimulationTime;

/// Formation reformation system - units restore cohesion when not under fire
pub fn formation_reformation_system(
    mut query: Query<(&mut Squad, &AIState, &Position)>,
    time: Res<SimulationTime>,
) {
    let dt = time.delta();
    
    for (mut squad, ai_state, _pos) in query.iter_mut() {
        // Only reform when not routing or engaging in melee
        let can_reform = match ai_state.state {
            BehaviorState::Idle | BehaviorState::Moving | BehaviorState::Reforming => true,
            BehaviorState::Routing | BehaviorState::Engaging => false,
        };
        
        if can_reform && squad.cohesion < 1.0 {
            // Reformation rate: ~10 seconds to go from 0.5 to 1.0 cohesion
            // NCOs and officers shout orders, men close ranks
            let reformation_rate = 0.05; // 5% per second
            squad.cohesion = (squad.cohesion + reformation_rate * dt).min(1.0);
        }
        
        // Cohesion degrades slightly while moving (maintaining formation is hard)
        if ai_state.state == BehaviorState::Moving {
            let drift_rate = 0.01; // 1% per second
            squad.cohesion = (squad.cohesion - drift_rate * dt).max(0.3);
        }
    }
}
