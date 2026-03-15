pub mod bayesian;
pub mod ctmc;
pub mod distributions;
pub mod dtmc;
pub mod monte_carlo;
pub mod poisson;
pub mod sde;
pub mod survival;

// Convenience re-exports used across Phase 2 agent modules.
pub use bayesian::{BayesianBeliefState, ClinicalObservation, ObservationLikelihoods};
pub use ctmc::{colonization_ctmc, CtmcTransition, GillespieSampler};
pub use distributions::{
    beta_sample, categorical_draw, exponential_sample, gamma_fn, gamma_sample, log_normal_sample,
    standard_normal,
};
pub use dtmc::{build_hai_dtmc_matrix, DiseaseState, PatientDTMC, N_DISEASE_STATES};
pub use monte_carlo::{antithetic_mean, antithetic_pair, monte_carlo_run, CrnManager};
pub use poisson::NhppArrivalProcess;
pub use sde::{OneCompartmentPk, OrnsteinUhlenbeck};
pub use survival::{cox_hazard_ratio, CoxCovariates, CoxParams, SurvivalDistribution};
