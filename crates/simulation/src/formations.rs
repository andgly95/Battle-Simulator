//! Formation management and maintenance

use bevy_ecs::prelude::*;
use crate::components::*;
use crate::movement::SimulationTime;

/// Formation reformation system - units restore cohesion when not under fire
///
/// This system simulates NCOs and officers reforming the ranks:
/// - Cohesion increases when not in combat (men close up gaps)
/// - Cohesion decreases slightly while moving (hard to maintain formation on the march)
/// - Routing units can't reform (they've broken completely)
pub fn formation_reformation_system(
    mut query: Query<(&mut Squad, &AIState)>,
    time: Res<SimulationTime>,
) {
    let dt = time.delta();

    for (mut squad, ai_state) in query.iter_mut() {
        // Only reform when not routing or in melee
        let can_reform = match ai_state.state {
            BehaviorState::Idle | BehaviorState::Moving | BehaviorState::Reforming | BehaviorState::Rallying => true,
            BehaviorState::Routing | BehaviorState::Engaging => false,
        };

        if can_reform && squad.cohesion < 1.0 {
            // Reformation rate: ~10 seconds to go from 0.5 to 1.0
            // NCOs shout orders, men close ranks
            let reformation_rate = 0.05; // 5% per second
            squad.cohesion = (squad.cohesion + reformation_rate * dt).min(1.0);
        }

        // Formation drifts slightly while moving
        if ai_state.state == BehaviorState::Moving {
            let drift_rate = 0.01; // 1% per second
            squad.cohesion = (squad.cohesion - drift_rate * dt).max(0.3);
        }

        // Under fire, cohesion degrades faster (men flinch, duck, break ranks)
        // This happens naturally via apply_casualties(), but we could add ambient degradation
    }
}
