use serde::{Deserialize, Serialize};
use crate::engine::{Agent, Grid};
use crate::engine::agent::AgentDecision;
use crate::engine::sim_engine::SimParams;
use crate::spatial::SpatialGrid;
use crate::rng::Mulberry32;

mod reactive;
mod bounded;
mod bdi;
pub mod prospect_theory;
pub mod thompson_sampling;
pub mod social_learning;
pub mod pbdi;

pub use reactive::ReactiveRules;
pub use bounded::BoundedRationalityRules;
pub use bdi::BdiRules;
pub use prospect_theory::ProspectTheoryParams;
pub use thompson_sampling::{ThompsonAntibiotic, AntibioticBandit};
pub use social_learning::{SocialLearningParams, social_learning_step, social_learning_from_mean};
pub use pbdi::{LikelihoodTable, UtilityParams, PhysicianPBDI};

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
        agent:   &Agent,
        agents:  &[Agent],
        grid:    &Grid<f32>,
        spatial: &SpatialGrid,
        params:  &SimParams,
        rng:     &mut Mulberry32,
    ) -> AgentDecision;

    fn name(&self) -> &'static str;
    fn pseudocode(&self) -> &'static str;
}

pub fn make_rule_set(id: &RuleSetId) -> Box<dyn DecisionRule> {
    match id {
        RuleSetId::Reactive => Box::new(ReactiveRules),
        RuleSetId::Bounded  => Box::new(BoundedRationalityRules),
        RuleSetId::Bdi      => Box::new(BdiRules::new()),
    }
}
