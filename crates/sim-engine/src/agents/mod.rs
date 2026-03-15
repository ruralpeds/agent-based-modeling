/// Healthcare agent modules.
pub mod types;
pub mod patient_agent;
pub mod nurse_agent;
pub mod bed_agent;
pub mod pathogen;
pub mod infection_control;
pub mod physician_agent;

pub use types::{
    Sex, HospitalUnit, ResistanceProfile, AntibioticClass, N_ANTIBIOTIC_CLASSES,
    NurseActivity, TreatmentAction, N_TREATMENT_ACTIONS, PhysicianIntention,
    ContactRoute, ProviderSocialState,
};
pub use patient_agent::{PatientAgent, PatientParams};
pub use nurse_agent::{NurseAgent, NurseParams};
pub use bed_agent::BedAgent;
pub use pathogen::{
    PathogenParams, transmission_probability, transmission_event,
    resistance_mutation_event, escalate_resistance, antibiotic_efficacy,
};
pub use infection_control::{InfectionControlAgent, SprtDecision, SprtMonitor};
pub use physician_agent::PhysicianAgent;
