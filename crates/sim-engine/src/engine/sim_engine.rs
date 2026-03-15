use crate::decision_rules::{make_rule_set, DecisionRule, RuleSetId};
use crate::engine::agent_buffers::write_soa;
use crate::engine::event_log::{DeathCause, EventLog, SimEvent};
use crate::engine::{Agent, AgentType, Grid};
use crate::rng::Mulberry32;
use crate::spatial::SpatialGrid;
use crate::statistics::{compute_stats, SimStats};
use serde::{Deserialize, Serialize};

pub const GRID_W: f32 = 80.0;
pub const GRID_H: f32 = 60.0;
pub const MAX_ENERGY: f32 = 200.0;
pub const MAX_AGENTS: usize = 5000;
pub const FIELDS_PER_AGENT: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimParams {
    pub prey_count: usize,
    pub predator_count: usize,
    pub prey_speed: f32,
    pub predator_speed: f32,
    pub prey_vision: f32,
    pub predator_vision: f32,
    pub prey_reproduce_energy: f32,
    pub predator_reproduce_energy: f32,
    pub prey_energy_gain: f32,
    pub predator_energy_gain: f32,
    pub prey_metabolism: f32,
    pub predator_metabolism: f32,
    pub max_energy: f32,
    pub grass_regrow_rate: f32,
    pub rule_set_id: RuleSetId,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            prey_count: 80,
            predator_count: 15,
            prey_speed: 0.35,
            predator_speed: 0.45,
            prey_vision: 5.0,
            predator_vision: 8.0,
            prey_reproduce_energy: 120.0,
            predator_reproduce_energy: 140.0,
            prey_energy_gain: 15.0,
            predator_energy_gain: 40.0,
            prey_metabolism: 0.5,
            predator_metabolism: 0.8,
            max_energy: MAX_ENERGY,
            grass_regrow_rate: 0.03,
            rule_set_id: RuleSetId::Reactive,
        }
    }
}

pub struct SimEngine {
    pub agents: Vec<Agent>,
    pub grid: Grid<f32>,
    pub spatial: SpatialGrid,
    pub rng: Mulberry32,
    pub tick: u64,
    pub params: SimParams,
    pub event_log: EventLog,
    pub rule_set: Box<dyn DecisionRule>,
    next_id: u32,
    agent_buf: Vec<f32>,
}

impl SimEngine {
    pub fn new(params: SimParams, seed: u32) -> Self {
        let mut rng = Mulberry32::new(seed);
        let grid = Grid::new_grass(GRID_W as usize, GRID_H as usize, &mut rng);
        let rule_set = make_rule_set(&params.rule_set_id);

        let mut next_id = 0u32;
        let mut agents = Vec::with_capacity(params.prey_count + params.predator_count);

        for _ in 0..params.prey_count {
            let x = rng.next_f32() * GRID_W;
            let y = rng.next_f32() * GRID_H;
            agents.push(Agent::new_prey(next_id, x, y, params.max_energy * 0.6));
            next_id += 1;
        }
        for _ in 0..params.predator_count {
            let x = rng.next_f32() * GRID_W;
            let y = rng.next_f32() * GRID_H;
            agents.push(Agent::new_predator(next_id, x, y, params.max_energy * 0.7));
            next_id += 1;
        }

        let spatial = SpatialGrid::new(GRID_W, GRID_H, 8.0);
        let agent_buf = vec![0.0f32; MAX_AGENTS * FIELDS_PER_AGENT];

        Self {
            agents,
            grid,
            spatial,
            rng,
            tick: 0,
            params,
            event_log: EventLog::new(500),
            rule_set,
            next_id,
            agent_buf,
        }
    }

    pub fn step(&mut self) {
        // 1. Rebuild spatial index
        self.spatial.build(&self.agents);

        // 2. Compute decisions (collect to avoid borrow conflicts)
        let decisions: Vec<_> = self
            .agents
            .iter()
            .map(|agent| {
                self.rule_set.decide(
                    agent,
                    &self.agents,
                    &self.grid,
                    &self.spatial,
                    &self.params,
                    &mut self.rng,
                )
            })
            .collect();

        // 3. Apply decisions
        let mut births: Vec<Agent> = Vec::new();
        let mut dead: Vec<usize> = Vec::new();
        let tick = self.tick;

        for (i, (agent, decision)) in self.agents.iter_mut().zip(decisions.iter()).enumerate() {
            // Move
            agent.x = (agent.x + decision.dx).rem_euclid(GRID_W);
            agent.y = (agent.y + decision.dy).rem_euclid(GRID_H);
            agent.action = decision.action;
            agent.intention = decision.intention;

            // Metabolize
            let metabolism = if agent.agent_type == AgentType::Prey {
                self.params.prey_metabolism
            } else {
                self.params.predator_metabolism
            };
            agent.energy -= metabolism;
            agent.age += 1;

            // Prey: forage
            if agent.agent_type == AgentType::Prey
                && decision.action == crate::engine::agent::Action::Forage
            {
                let gained = self
                    .grid
                    .eat_at(agent.x, agent.y, self.params.prey_energy_gain);
                agent.energy = (agent.energy + gained).min(self.params.max_energy);
            }

            // Reproduce
            let repro_threshold = if agent.agent_type == AgentType::Prey {
                self.params.prey_reproduce_energy
            } else {
                self.params.predator_reproduce_energy
            };
            if agent.energy >= repro_threshold && self.rng.next_f32() < 0.02 {
                let offspring_x = (agent.x + self.rng.next_f32() * 4.0 - 2.0).rem_euclid(GRID_W);
                let offspring_y = (agent.y + self.rng.next_f32() * 4.0 - 2.0).rem_euclid(GRID_H);
                let offspring_energy = agent.energy * 0.4;
                agent.energy *= 0.6;
                agent.offspring += 1;
                let id = self.next_id;
                self.next_id += 1;
                let offspring = if agent.agent_type == AgentType::Prey {
                    Agent::new_prey(id, offspring_x, offspring_y, offspring_energy)
                } else {
                    Agent::new_predator(id, offspring_x, offspring_y, offspring_energy)
                };
                self.event_log.push(SimEvent::Birth {
                    id,
                    parent_id: agent.id,
                    tick,
                });
                births.push(offspring);
            }

            // Die from starvation or old age
            if agent.energy <= 0.0 || agent.age > 2000 {
                let cause = if agent.energy <= 0.0 {
                    DeathCause::Starvation
                } else {
                    DeathCause::OldAge
                };
                self.event_log.push(SimEvent::Death {
                    id: agent.id,
                    cause,
                    tick,
                });
                dead.push(i);
            }
        }

        // Predator hunting — two-pass to satisfy borrow checker:
        // Pass 1 (read-only): collect (predator_idx, prey_idx) kill pairs
        let kill_range_sq = 2.0f32 * 2.0;
        let vision_sq = self.params.predator_vision * self.params.predator_vision;
        let mut kill_pairs: Vec<(usize, usize)> = Vec::new(); // (predator_idx, prey_idx)
        for (pi, predator) in self.agents.iter().enumerate() {
            if predator.agent_type != AgentType::Predator {
                continue;
            }
            let mut closest_d2 = vision_sq;
            let mut closest_idx: Option<usize> = None;
            for (j, prey) in self.agents.iter().enumerate() {
                if prey.agent_type != AgentType::Prey {
                    continue;
                }
                let dx = prey.x - predator.x;
                let dy = prey.y - predator.y;
                let d2 = dx * dx + dy * dy;
                if d2 < closest_d2 && d2 < kill_range_sq {
                    closest_d2 = d2;
                    closest_idx = Some(j);
                }
            }
            if let Some(j) = closest_idx {
                if !dead.contains(&j) && !kill_pairs.iter().any(|&(_, pj)| pj == j) {
                    kill_pairs.push((pi, j));
                }
            }
        }
        // Pass 2 (write): apply energy gains, record kills
        let mut extra_dead: Vec<usize> = Vec::new();
        for (pi, pj) in kill_pairs {
            let prey_id = self.agents[pj].id;
            let predator_id = self.agents[pi].id;
            self.agents[pi].energy = (self.agents[pi].energy + self.params.predator_energy_gain)
                .min(self.params.max_energy);
            self.agents[pi].kills += 1;
            self.event_log.push(SimEvent::Kill {
                predator_id,
                prey_id,
                tick,
            });
            extra_dead.push(pj);
        }
        dead.extend(extra_dead);

        // Remove dead (sort descending, dedup, swap_remove)
        dead.sort_unstable();
        dead.dedup();
        for i in dead.into_iter().rev() {
            if i < self.agents.len() {
                self.agents.swap_remove(i);
            }
        }

        // Add births
        let available = MAX_AGENTS.saturating_sub(self.agents.len());
        let to_add = births.len().min(available);
        self.agents.extend(births.into_iter().take(to_add));

        // Grass regrowth
        self.grid
            .regrow(self.params.grass_regrow_rate, &mut self.rng);

        self.tick += 1;
    }

    pub fn write_agent_buffer(&mut self) -> &[f32] {
        let n = self.agents.len();
        write_soa(&self.agents, &mut self.agent_buf[..n * FIELDS_PER_AGENT]);
        &self.agent_buf[..n * FIELDS_PER_AGENT]
    }

    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }
    pub fn prey_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|a| a.agent_type == AgentType::Prey)
            .count()
    }
    pub fn pred_count(&self) -> usize {
        self.agents
            .iter()
            .filter(|a| a.agent_type == AgentType::Predator)
            .count()
    }
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn stats(&self) -> SimStats {
        compute_stats(&self.agents, self.tick)
    }

    pub fn set_params(&mut self, params: SimParams) {
        if params.rule_set_id != self.params.rule_set_id {
            self.rule_set = make_rule_set(&params.rule_set_id);
        }
        self.params = params;
    }

    pub fn reset(&mut self, seed: u32) {
        let params = self.params.clone();
        *self = Self::new(params, seed);
    }
}
