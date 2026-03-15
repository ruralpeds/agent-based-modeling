use super::DecisionRule;
use crate::engine::agent::{Action, AgentDecision, Intention};
use crate::engine::sim_engine::SimParams;
use crate::engine::{Agent, AgentType, Grid};
use crate::rng::Mulberry32;
use crate::spatial::SpatialGrid;

/// Bounded rationality: satisficing — pick first option above aspiration threshold
pub struct BoundedRationalityRules;

impl DecisionRule for BoundedRationalityRules {
    fn decide(
        &self,
        agent: &Agent,
        agents: &[Agent],
        grid: &Grid<f32>,
        spatial: &SpatialGrid,
        params: &SimParams,
        rng: &mut Mulberry32,
    ) -> AgentDecision {
        if agent.agent_type == AgentType::Prey {
            bounded_prey(agent, agents, grid, spatial, params, rng)
        } else {
            bounded_predator(agent, agents, grid, spatial, params, rng)
        }
    }

    fn name(&self) -> &'static str {
        "Bounded"
    }
    fn pseudocode(&self) -> &'static str {
        "aspiration = f(energy)\nFOR each option in [flee, forage, rest, wander]:\n  IF utility(option) >= aspiration THEN do option"
    }
}

fn bounded_prey(
    agent: &Agent,
    agents: &[Agent],
    grid: &Grid<f32>,
    spatial: &SpatialGrid,
    params: &SimParams,
    rng: &mut Mulberry32,
) -> AgentDecision {
    let speed = params.prey_speed;
    let aspiration = 1.0 - (agent.energy / params.max_energy);

    let pred = spatial.nearest_of_type(
        agent.x,
        agent.y,
        params.prey_vision,
        AgentType::Predator,
        agents,
    );
    if let Some((px, py, _)) = pred {
        let threat = 1.0 / ((agent.x - px).hypot(agent.y - py) / params.prey_vision).max(0.1);
        if threat >= aspiration * 0.6 {
            let dx = agent.x - px;
            let dy = agent.y - py;
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            return AgentDecision {
                dx: (dx / d) * speed,
                dy: (dy / d) * speed,
                action: Action::Flee,
                intention: Intention::Survive,
                target_id: None,
            };
        }
    }

    let grass = grid.get_at(agent.x, agent.y);
    let hunger = 1.0 - (agent.energy / params.max_energy);
    if grass * hunger >= aspiration * 0.4 {
        return AgentDecision {
            dx: (rng.next_f32() - 0.5) * speed * 0.4,
            dy: (rng.next_f32() - 0.5) * speed * 0.4,
            action: Action::Forage,
            intention: Intention::Eat,
            target_id: None,
        };
    }

    if agent.energy > params.prey_reproduce_energy * 0.8 {
        return AgentDecision {
            dx: 0.0,
            dy: 0.0,
            action: Action::Rest,
            intention: Intention::Rest,
            target_id: None,
        };
    }

    let angle = rng.next_f32() * std::f32::consts::TAU;
    AgentDecision {
        dx: angle.cos() * speed * 0.7,
        dy: angle.sin() * speed * 0.7,
        action: Action::Wander,
        intention: Intention::Wander,
        target_id: None,
    }
}

fn bounded_predator(
    agent: &Agent,
    agents: &[Agent],
    _grid: &Grid<f32>,
    spatial: &SpatialGrid,
    params: &SimParams,
    rng: &mut Mulberry32,
) -> AgentDecision {
    let speed = params.predator_speed;
    let aspiration = 1.0 - (agent.energy / params.max_energy);

    let prey = spatial.nearest_of_type(
        agent.x,
        agent.y,
        params.predator_vision,
        AgentType::Prey,
        agents,
    );
    if let Some((tx, ty, tid)) = prey {
        let utility = 1.0 - ((agent.x - tx).hypot(agent.y - ty) / params.predator_vision).min(1.0);
        if utility >= aspiration * 0.3 {
            let dx = tx - agent.x;
            let dy = ty - agent.y;
            let d = (dx * dx + dy * dy).sqrt().max(0.001);
            return AgentDecision {
                dx: (dx / d) * speed,
                dy: (dy / d) * speed,
                action: Action::Hunt,
                intention: Intention::Hunt,
                target_id: Some(tid),
            };
        }
    }

    if agent.energy > params.predator_reproduce_energy * 0.85 {
        return AgentDecision {
            dx: 0.0,
            dy: 0.0,
            action: Action::Rest,
            intention: Intention::Rest,
            target_id: None,
        };
    }

    let angle = rng.next_f32() * std::f32::consts::TAU;
    AgentDecision {
        dx: angle.cos() * speed * 0.8,
        dy: angle.sin() * speed * 0.8,
        action: Action::Patrol,
        intention: Intention::Patrol,
        target_id: None,
    }
}
