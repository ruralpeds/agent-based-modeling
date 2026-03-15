use crate::engine::agent::AgentDecision;
use crate::engine::sim_engine::SimParams;
use crate::engine::{Agent, Grid};
use crate::rng::Mulberry32;
use crate::spatial::SpatialGrid;
use serde::{Deserialize, Serialize};

mod bdi;
mod bounded;
pub mod pbdi;
pub mod prospect_theory;
mod reactive;
pub mod social_learning;
pub mod thompson_sampling;

pub use bdi::BdiRules;
pub use bounded::BoundedRationalityRules;
pub use pbdi::{LikelihoodTable, PhysicianPBDI, UtilityParams};
pub use prospect_theory::ProspectTheoryParams;
pub use reactive::ReactiveRules;
pub use social_learning::{social_learning_from_mean, social_learning_step, SocialLearningParams};
pub use thompson_sampling::{AntibioticBandit, ThompsonAntibiotic};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleSetId {
    Reactive,
    Bounded,
    Bdi,
}

/// Object-safe trait — all rule sets implement this
pub trait DecisionRule: Send + Sync {
    fn decide(
        &self,
        agent: &Agent,
        agents: &[Agent],
        grid: &Grid<f32>,
        spatial: &SpatialGrid,
        params: &SimParams,
        rng: &mut Mulberry32,
    ) -> AgentDecision;

    fn name(&self) -> &'static str;
    fn pseudocode(&self) -> &'static str;
}

pub fn make_rule_set(id: &RuleSetId) -> Box<dyn DecisionRule> {
    match id {
        RuleSetId::Reactive => Box::new(ReactiveRules),
        RuleSetId::Bounded => Box::new(BoundedRationalityRules),
        RuleSetId::Bdi => Box::new(BdiRules::new()),
    }
}
