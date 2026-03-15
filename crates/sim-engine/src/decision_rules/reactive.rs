use super::DecisionRule;
use crate::engine::agent::{Action, AgentDecision, Intention};
use crate::engine::sim_engine::SimParams;
use crate::engine::{Agent, AgentType, Grid};
use crate::rng::Mulberry32;
use crate::spatial::SpatialGrid;

pub struct ReactiveRules;

impl DecisionRule for ReactiveRules {
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
            decide_prey_reactive(agent, agents, grid, spatial, params, rng)
        } else {
            decide_predator_reactive(agent, agents, grid, spatial, params, rng)
        }
    }

    fn name(&self) -> &'static str {
        "Reactive"
    }
    fn pseudocode(&self) -> &'static str {
        "IF predator_near THEN flee\nELSE IF hungry AND grass_near THEN forage\nELSE wander"
    }
}

fn decide_prey_reactive(
    agent: &Agent,
    agents: &[Agent],
    grid: &Grid<f32>,
    spatial: &SpatialGrid,
    params: &SimParams,
    rng: &mut Mulberry32,
) -> AgentDecision {
    let vision = params.prey_vision;
    let speed = params.prey_speed;

    // Check for nearby predators
    let nearest_pred =
        spatial.nearest_of_type(agent.x, agent.y, vision, AgentType::Predator, agents);

    if let Some((px, py, _)) = nearest_pred {
        // Flee away from predator
        let dx = agent.x - px;
        let dy = agent.y - py;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        return AgentDecision {
            dx: (dx / dist) * speed,
            dy: (dy / dist) * speed,
            action: Action::Flee,
            intention: Intention::Survive,
            target_id: None,
        };
    }

    // Forage if hungry and grass nearby
    let grass = grid.get_at(agent.x, agent.y);
    if agent.energy < params.prey_reproduce_energy * 0.6 && grass > 0.2 {
        return AgentDecision {
            dx: (rng.next_f32() - 0.5) * speed * 0.5,
            dy: (rng.next_f32() - 0.5) * speed * 0.5,
            action: Action::Forage,
            intention: Intention::Eat,
            target_id: None,
        };
    }

    // Wander
    let angle = rng.next_f32() * std::f32::consts::TAU;
    AgentDecision {
        dx: angle.cos() * speed,
        dy: angle.sin() * speed,
        action: Action::Wander,
        intention: Intention::Wander,
        target_id: None,
    }
}

fn decide_predator_reactive(
    agent: &Agent,
    agents: &[Agent],
    _grid: &Grid<f32>,
    spatial: &SpatialGrid,
    params: &SimParams,
    rng: &mut Mulberry32,
) -> AgentDecision {
    let vision = params.predator_vision;
    let speed = params.predator_speed;

    // Hunt nearest prey
    let nearest_prey = spatial.nearest_of_type(agent.x, agent.y, vision, AgentType::Prey, agents);

    if let Some((tx, ty, tid)) = nearest_prey {
        let dx = tx - agent.x;
        let dy = ty - agent.y;
        let dist = (dx * dx + dy * dy).sqrt().max(0.001);
        return AgentDecision {
            dx: (dx / dist) * speed,
            dy: (dy / dist) * speed,
            action: Action::Hunt,
            intention: Intention::Hunt,
            target_id: Some(tid),
        };
    }

    // Patrol / wander
    let angle = rng.next_f32() * std::f32::consts::TAU;
    AgentDecision {
        dx: angle.cos() * speed,
        dy: angle.sin() * speed,
        action: Action::Patrol,
        intention: Intention::Patrol,
        target_id: None,
    }
}
