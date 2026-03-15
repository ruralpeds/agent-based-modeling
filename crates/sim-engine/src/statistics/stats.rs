use serde::{Serialize, Deserialize};
use crate::engine::{Agent, AgentType};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimStats {
    pub tick:           u64,
    pub prey_count:     usize,
    pub pred_count:     usize,
    pub mean_prey_energy:   f32,
    pub mean_pred_energy:   f32,
    pub gini_energy:        f32,
    pub action_dist:        [f32; 7],
}

pub fn compute_stats(agents: &[Agent], tick: u64) -> SimStats {
    let prey: Vec<&Agent>  = agents.iter().filter(|a| a.agent_type == AgentType::Prey).collect();
    let preds: Vec<&Agent> = agents.iter().filter(|a| a.agent_type == AgentType::Predator).collect();

    let mean_prey_energy = mean_energy(&prey);
    let mean_pred_energy = mean_energy(&preds);
    let gini_energy      = gini(agents);
    let action_dist      = action_distribution(agents);

    SimStats {
        tick,
        prey_count:      prey.len(),
        pred_count:      preds.len(),
        mean_prey_energy,
        mean_pred_energy,
        gini_energy,
        action_dist,
    }
}

fn mean_energy(agents: &[&Agent]) -> f32 {
    if agents.is_empty() { return 0.0; }
    let sum: f32 = agents.iter().map(|a| a.energy).sum();
    sum / agents.len() as f32
}

fn gini(agents: &[Agent]) -> f32 {
    if agents.is_empty() { return 0.0; }
    let n = agents.len() as f32;
    let mut energies: Vec<f32> = agents.iter().map(|a| a.energy).collect();
    energies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut sum = 0.0f32;
    for (i, &e) in energies.iter().enumerate() {
        sum += (2.0 * (i as f32 + 1.0) - n - 1.0) * e;
    }
    let total: f32 = energies.iter().sum();
    if total < 0.001 { return 0.0; }
    sum / (n * total)
}

fn action_distribution(agents: &[Agent]) -> [f32; 7] {
    let mut counts = [0u32; 7];
    for a in agents {
        let idx = a.action as u8 as usize;
        if idx < 7 { counts[idx] += 1; }
    }
    let total = agents.len().max(1) as f32;
    let mut dist = [0.0f32; 7];
    for (i, &c) in counts.iter().enumerate() {
        dist[i] = c as f32 / total;
    }
    dist
}
