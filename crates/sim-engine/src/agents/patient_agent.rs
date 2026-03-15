/// Full patient agent state vector for HAI/ICU simulation.
/// `#[repr(C)]` ensures predictable field layout for SharedArrayBuffer writes.
use crate::rng::Mulberry32;
use crate::stochastic::dtmc::DiseaseState;
use crate::stochastic::survival::{SurvivalDistribution, CoxCovariates, CoxParams, cox_hazard_ratio};
use super::types::{Sex, HospitalUnit, ResistanceProfile, TreatmentAction};

// ─── PatientAgent ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
#[repr(C)]
pub struct PatientAgent {
    // Identity
    pub id:  u32,
    pub age: u8,   // years
    pub sex: Sex,

    // Location
    pub room_id: u16,
    pub bed_id:  u16,
    pub unit:    HospitalUnit,

    // Disease state (DTMC-driven)
    pub disease_state:       DiseaseState,
    pub admission_severity:  f32,   // SOFA/APACHE II 0–24
    pub charlson_index:      u8,    // Charlson Comorbidity Index 0–37
    pub antibiotic_days:     u16,
    pub ventilated:          bool,
    pub central_line:        bool,
    pub urinary_catheter:    bool,

    // Physiological state variables (Ornstein-Uhlenbeck SDE-driven)
    pub wbc:       f32,   // × 10⁹/L, homeostatic setpoint 7.5
    pub temperature: f32, // °C, homeostatic setpoint 37.0
    pub drug_conc:   f32, // normalized [0, 1]

    // Stochastic model parameters (personalised from admission covariates)
    pub immunity_score:    f32,   // [0, 1]
    pub pathogen_load:     f32,   // [0, 1]
    pub resistance_profile: ResistanceProfile,

    // Survival model
    pub arrival_tick:      u64,
    pub planned_los_ticks: u64,  // sampled from Weibull at admission
    pub mortality_prob:    f32,  // updated each tick

    // Assigned providers
    pub attending_id:     u32,
    pub primary_nurse_id: u32,

    /// Current physician-prescribed treatment action (updated by PhysicianAgent each round).
    pub current_treatment: TreatmentAction,
}

impl PatientAgent {
    /// Construct a new ICU admission patient.
    /// LOS is sampled from Weibull(0.85, 120 h) adjusted by Cox PH covariates.
    pub fn new_icu(id: u32, arrival_tick: u64, rng: &mut Mulberry32, params: &PatientParams) -> Self {
        let age      = 45u8 + (rng.next_f32() * 40.0) as u8;  // 45–85 years
        let charlson = (rng.next_f32() * 6.0) as u8;           // 0–5 typical ICU
        let severity = rng.next_f32() * 15.0 + 2.0;            // SOFA 2–17
        let ventilated    = rng.next_f32() < 0.45;
        let central_line  = rng.next_f32() < 0.60;
        let urinary_cath  = rng.next_f32() < 0.35;

        let cov = CoxCovariates {
            age: age as f32, charlson_index: charlson as f32,
            admission_severity: severity, ventilated, central_line,
        };
        let hr = cox_hazard_ratio(&cov, &params.cox);
        let los_h = SurvivalDistribution::icu_los().sample_adjusted(hr, rng);
        let planned_los_ticks = ((los_h * params.ticks_per_hour as f32) as u64).max(1);

        let immunity_score = (1.0
            - rng.next_f32() * 0.4
            - 0.1 * (charlson as f32 / 10.0)
            - if ventilated { 0.15 } else { 0.0 }
        ).clamp(0.0, 1.0);

        let disease_state = if rng.next_f32() < params.admission_colonization_rate {
            DiseaseState::Colonized
        } else {
            DiseaseState::Susceptible
        };

        Self {
            id, age,
            sex: if rng.next_f32() < 0.52 { Sex::Male } else { Sex::Female },
            room_id: (rng.next_u32() % 25) as u16,
            bed_id:  0,
            unit:    HospitalUnit::Icu,
            disease_state,
            admission_severity: severity,
            charlson_index:     charlson,
            antibiotic_days:    0,
            ventilated,
            central_line,
            urinary_catheter:   urinary_cath,
            wbc:         7.5 + rng.next_f32() * 3.0 - 1.5,
            temperature: 37.0 + rng.next_f32() * 0.6 - 0.3,
            drug_conc:   0.0,
            immunity_score,
            pathogen_load: if disease_state == DiseaseState::Colonized { 0.3 } else { 0.0 },
            resistance_profile: ResistanceProfile::FullySusceptible,
            arrival_tick,
            planned_los_ticks,
            mortality_prob: 0.0,
            attending_id:     0,
            primary_nurse_id: 0,
            current_treatment: TreatmentAction::Observe,
        }
    }

    // ── Covariate methods ────────────────────────────────────────────────────

    /// Multiplicative susceptibility factor for transmission probability.
    /// Combines immunity deficit with device-use amplifiers.
    pub fn susceptibility_multiplier(&self) -> f32 {
        let base = 1.0 - self.immunity_score;
        let amp = if self.central_line       { 1.4 } else { 1.0 }
                * if self.ventilated         { 1.6 } else { 1.0 }
                * if self.antibiotic_days > 5 { 1.3 } else { 1.0 }
                * (1.0 + 0.02 * self.charlson_index as f32);
        (base * amp).clamp(0.0, 1.0)
    }

    /// Immunosuppression factor ≥ 1 (used to scale DTMC transition rates).
    pub fn immunosuppression_factor(&self) -> f32 {
        (1.0
            + 0.15 * self.charlson_index as f32 / 10.0
            + if self.central_line { 0.30 } else { 0.0 }
            + if self.ventilated   { 0.40 } else { 0.0 }
        ).clamp(1.0, 3.0)
    }

    /// Cox PH hazard ratio for LOS (higher ⟹ shorter expected stay).
    pub fn los_hazard_ratio(&self, params: &CoxParams) -> f32 {
        let cov = CoxCovariates {
            age: self.age as f32,
            charlson_index:      self.charlson_index as f32,
            admission_severity:  self.admission_severity,
            ventilated:          self.ventilated,
            central_line:        self.central_line,
        };
        cox_hazard_ratio(&cov, params)
    }

    /// Mortality risk multiplier from age, comorbidity, and severity.
    pub fn mortality_multiplier(&self) -> f32 {
        (1.0
            + 0.05 * (self.age as f32 - 65.0).max(0.0)
            + 0.10 * self.charlson_index as f32
            + 0.08 * self.admission_severity
        ).max(0.0)
    }

    /// True when the patient's planned LOS has elapsed.
    pub fn should_discharge(&self, current_tick: u64) -> bool {
        current_tick >= self.arrival_tick + self.planned_los_ticks
    }

    /// Antibiotic days as f32 (used in logistic models).
    pub fn antibiotic_days_f32(&self) -> f32 { self.antibiotic_days as f32 }

    /// Current treatment action (for buffer serialisation).
    pub fn treatment_action(&self) -> TreatmentAction { self.current_treatment }
}

// ─── PatientParams ────────────────────────────────────────────────────────────

/// Construction-time parameters for new patient generation.
#[derive(Debug, Clone)]
pub struct PatientParams {
    pub ticks_per_hour:              u32,
    /// NHSN ICU admission colonization prevalence (~15%).
    pub admission_colonization_rate: f32,
    pub cox:                         CoxParams,
}

impl Default for PatientParams {
    fn default() -> Self {
        Self {
            ticks_per_hour:              12,
            admission_colonization_rate: 0.15,
            cox:                         CoxParams::default(),
        }
    }
}
