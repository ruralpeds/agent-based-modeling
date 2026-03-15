use crate::engine::{Agent, AgentType, Grid};
use crate::engine::agent::{AgentDecision, Action, Intention};
use crate::engine::sim_engine::SimParams;
use crate::spatial::SpatialGrid;
use crate::rng::Mulberry32;
use super::DecisionRule;

/// BDI (Beliefs-Desires-Intentions) architecture
/// Beliefs: what agent knows about environment
/// Desires: agent's goals ranked by priority
/// Intentions: committed plan based on beliefs + desires
#[derive(Default)]
pub struct BdiRules {
    // Per-instance state would go here (but for simplicity, stateless)
}

impl BdiRules {
    pub fn new() -> Self { Self::default() }
}

#[allow(dead_code)]
struct Beliefs {
    nearest_pred:   Option<(f32, f32, u32)>,
    nearest_prey:   Option<(f32, f32, u32)>,
    grass_level:    f32,
    energy_ratio:   f32,
    age_ratio:      f32,
    population_pressure: f32,
}

fn perceive(
    agent: &Agent,
    agents: &[Agent],
    grid: &Grid<f32>,
    spatial: &SpatialGrid,
    params: &SimParams,
) -> Beliefs {
    let vision = if agent.agent_type == AgentType::Prey { params.prey_vision } else { params.predator_vision };
    Beliefs {
        nearest_pred: spatial.nearest_of_type(agent.x, agent.y, vision, AgentType::Predator, agents),
        nearest_prey: spatial.nearest_of_type(agent.x, agent.y, vision, AgentType::Prey, agents),
        grass_level:  grid.get_at(agent.x, agent.y),
        energy_ratio: agent.energy / params.max_energy,
        age_ratio:    (agent.age as f32 / 2000.0).min(1.0),
        population_pressure: (agents.len() as f32 / 1000.0).min(1.0),
    }
}

impl DecisionRule for BdiRules {
    fn decide(
        &self,
        agent:   &Agent,
        agents:  &[Agent],
        grid:    &Grid<f32>,
        spatial: &SpatialGrid,
        params:  &SimParams,
        rng:     &mut Mulberry32,
    ) -> AgentDecision {
        let beliefs = perceive(agent, agents, grid, spatial, params);

        if agent.agent_type == AgentType::Prey {
            bdi_prey(agent, &beliefs, params, rng)
        } else {
            bdi_predator(agent, &beliefs, params, rng)
        }
    }

    fn name(&self) -> &'static str { "BDI" }
    fn pseudocode(&self) -> &'static str {
        "beliefs = perceive(environment)\ndesires = rank_goals(beliefs)\nintention = select(desires)\nexecute(intention)"
    }
}

fn bdi_prey(agent: &Agent, b: &Beliefs, params: &SimParams, rng: &mut Mulberry32) -> AgentDecision {
    let speed = params.prey_speed;

    // Desire: Survive (highest priority when threatened)
    if let Some((px, py, _)) = b.nearest_pred {
        let threat_dist = (agent.x - px).hypot(agent.y - py);
        let threat_urgency = 1.0 - (threat_dist / params.prey_vision).min(1.0);
        if threat_urgency > 0.3 || b.energy_ratio < 0.25 {
            let dx = agent.x - px;
            let dy = agent.y - py;
            let d  = (dx * dx + dy * dy).sqrt().max(0.001);
            // Add randomness to evasion
            let noise_x = (rng.next_f32() - 0.5) * 0.3;
            let noise_y = (rng.next_f32() - 0.5) * 0.3;
            return AgentDecision {
                dx: (dx / d + noise_x) * speed,
                dy: (dy / d + noise_y) * speed,
                action: Action::Flee,
                intention: Intention::Survive,
                target_id: None,
            };
        }
    }

    // Desire: Reproduce
    if b.energy_ratio > 0.75 && b.nearest_pred.is_none() {
        let angle = rng.next_f32() * std::f32::consts::TAU;
        return AgentDecision {
            dx: angle.cos() * speed * 0.8,
            dy: angle.sin() * speed * 0.8,
            action: Action::Socialize,
            intention: Intention::Reproduce,
            target_id: None,
        };
    }

    // Desire: Eat
    if b.energy_ratio < 0.65 && b.grass_level > 0.15 {
        let bias_x = (rng.next_f32() - 0.5) * 2.0;
        let bias_y = (rng.next_f32() - 0.5) * 2.0;
        return AgentDecision {
            dx: bias_x.signum() * speed * 0.6,
            dy: bias_y.signum() * speed * 0.6,
            action: Action::Forage,
            intention: Intention::Eat,
            target_id: None,
        };
    }

    // Default: Wander
    let angle = rng.next_f32() * std::f32::consts::TAU;
    AgentDecision {
        dx: angle.cos() * speed,
        dy: angle.sin() * speed,
        action: Action::Wander,
        intention: Intention::Wander,
        target_id: None,
    }
}

fn bdi_predator(agent: &Agent, b: &Beliefs, params: &SimParams, rng: &mut Mulberry32) -> AgentDecision {
    let speed = params.predator_speed;

    // Desire: Hunt (when hungry or prey available)
    if let Some((tx, ty, tid)) = b.nearest_prey {
        let hunt_drive = 1.0 - b.energy_ratio;
        if hunt_drive > 0.2 || b.energy_ratio < 0.5 {
            let dx = tx - agent.x;
            let dy = ty - agent.y;
            let d  = (dx * dx + dy * dy).sqrt().max(0.001);
            // Intercept prediction: move slightly ahead of prey
            return AgentDecision {
                dx: (dx / d) * speed,
                dy: (dy / d) * speed,
                action: Action::Hunt,
                intention: Intention::Hunt,
                target_id: Some(tid),
            };
        }
    }

    // Desire: Reproduce
    if b.energy_ratio > 0.8 {
        let angle = rng.next_f32() * std::f32::consts::TAU;
        return AgentDecision {
            dx: angle.cos() * speed * 0.5,
            dy: angle.sin() * speed * 0.5,
            action: Action::Socialize,
            intention: Intention::Reproduce,
            target_id: None,
        };
    }

    // Desire: Patrol territory
    if b.energy_ratio > 0.5 {
        let angle = rng.next_f32() * std::f32::consts::TAU;
        return AgentDecision {
            dx: angle.cos() * speed * 0.7,
            dy: angle.sin() * speed * 0.7,
            action: Action::Patrol,
            intention: Intention::Patrol,
            target_id: None,
        };
    }

    // Default: Wander searching for prey
    let angle = rng.next_f32() * std::f32::consts::TAU;
    AgentDecision {
        dx: angle.cos() * speed,
        dy: angle.sin() * speed,
        action: Action::Wander,
        intention: Intention::Wander,
        target_id: None,
    }
}
