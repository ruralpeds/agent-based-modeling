pub mod distributions;
pub mod dtmc;
pub mod ctmc;
pub mod poisson;
pub mod survival;
pub mod bayesian;
pub mod sde;
pub mod monte_carlo;

// Convenience re-exports used across Phase 2 agent modules.
pub use distributions::{
    standard_normal, exponential_sample, log_normal_sample,
    gamma_sample, beta_sample, categorical_draw, gamma_fn,
};
pub use dtmc::{DiseaseState, PatientDTMC, N_DISEASE_STATES, build_hai_dtmc_matrix};
pub use ctmc::{CtmcTransition, GillespieSampler, colonization_ctmc};
pub use poisson::NhppArrivalProcess;
pub use survival::{SurvivalDistribution, CoxCovariates, CoxParams, cox_hazard_ratio};
pub use bayesian::{ClinicalObservation, ObservationLikelihoods, BayesianBeliefState};
pub use sde::{OrnsteinUhlenbeck, OneCompartmentPk};
pub use monte_carlo::{CrnManager, antithetic_pair, antithetic_mean, monte_carlo_run};
