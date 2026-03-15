use crate::agents::nurse_agent::NurseParams;
use crate::agents::pathogen::PathogenParams;
use crate::stochastic::survival::CoxParams;
/// Serialisable configuration and statistics for the hospital simulation.
use serde::{Deserialize, Serialize};

// ─── Buffer layout constants ──────────────────────────────────────────────────

/// Maximum simultaneous in-patient census.
pub const MAX_PATIENTS: usize = 500;
/// f32 fields written per patient into the SoA buffer.
pub const FIELDS_PER_PATIENT: usize = 12;

/// Field indices in the SoA patient buffer.
///
/// Buffer size = MAX_PATIENTS × FIELDS_PER_PATIENT × 4 bytes.
/// Each field occupies a contiguous stripe of MAX_PATIENTS f32 values.
pub mod patient_field {
    pub const ROOM_ID: usize = 0;
    pub const UNIT: usize = 1;
    pub const DISEASE_STATE: usize = 2;
    pub const AGE: usize = 3;
    pub const WBC: usize = 4;
    pub const TEMPERATURE: usize = 5;
    pub const DRUG_CONC: usize = 6;
    pub const RESISTANCE: usize = 7;
    pub const ANTIBIOTIC_DAYS: usize = 8;
    pub const ADMISSION_SEVERITY: usize = 9;
    pub const TREATMENT_ACTION: usize = 10;
    pub const PATIENT_ID: usize = 11;
}

// ─── HospitalParams ───────────────────────────────────────────────────────────

/// Top-level parameters for the hospital simulation (JSON-serialisable).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HospitalParams {
    // ── Census / layout ────────────────────────────────────────────────────────
    /// Number of ICU beds available.
    pub icu_beds: usize,
    /// Number of ward beds available.
    pub ward_beds: usize,
    /// Number of nursing staff.
    pub n_nurses: usize,
    /// Number of attending physicians.
    pub n_physicians: usize,

    // ── Simulation time ────────────────────────────────────────────────────────
    /// Ticks per simulated hour (default 12 → 5-min ticks).
    pub ticks_per_hour: u32,

    // ── Patient flow ───────────────────────────────────────────────────────────
    /// Base arrival rate (patients per hour, calibrated to NHAMCS).
    pub base_arrival_rate: f32,
    /// Fraction of arriving patients who are colonised on admission.
    pub admission_colonization_rate: f32,

    // ── Pathogen ───────────────────────────────────────────────────────────────
    pub pathogen: PathogenParams,

    // ── DTMC disease transition calibration ────────────────────────────────────
    /// λ  colonisation rate (per tick).
    pub dtmc_lambda: f32,
    /// α  exposed→colonised rate.
    pub dtmc_alpha: f32,
    /// δₑ exposed spontaneous clearance.
    pub dtmc_delta_e: f32,
    /// β  colonised→infected rate.
    pub dtmc_beta: f32,
    /// γ  infected→treated rate.
    pub dtmc_gamma: f32,
    /// η  infected→dead rate.
    pub dtmc_eta: f32,
    /// ψ  treated→recovered rate.
    pub dtmc_psi: f32,
    /// ρ  treated→dead rate.
    pub dtmc_rho: f32,
    /// φ  recovered→susceptible (waning immunity).
    pub dtmc_phi: f32,
    /// θ  colonised→susceptible (spontaneous clearance).
    pub dtmc_theta: f32,

    // ── Providers ─────────────────────────────────────────────────────────────
    pub nurse: NurseParams,

    // ── Survival / LOS ────────────────────────────────────────────────────────
    pub cox: CoxParams,

    // ── Hand-hygiene / dispenser layout ───────────────────────────────────────
    /// Mean distance (metres) to nearest HH dispenser.
    pub mean_dispenser_dist: f32,

    // ── Social learning ───────────────────────────────────────────────────────
    /// Period (ticks) between social-learning updates for providers.
    pub social_learning_period: u32,
}

impl Default for HospitalParams {
    fn default() -> Self {
        Self {
            icu_beds: 20,
            ward_beds: 80,
            n_nurses: 30,
            n_physicians: 10,
            ticks_per_hour: 12,
            base_arrival_rate: 3.2,
            admission_colonization_rate: 0.06,
            pathogen: PathogenParams::default(),
            dtmc_lambda: 0.002,
            dtmc_alpha: 0.10,
            dtmc_delta_e: 0.05,
            dtmc_beta: 0.08,
            dtmc_gamma: 0.30,
            dtmc_eta: 0.02,
            dtmc_psi: 0.40,
            dtmc_rho: 0.01,
            dtmc_phi: 0.001,
            dtmc_theta: 0.05,
            nurse: NurseParams::default(),
            cox: CoxParams::default(),
            mean_dispenser_dist: 2.0,
            social_learning_period: 12 * 8, // every 8 h
        }
    }
}

// ─── HospitalStats ────────────────────────────────────────────────────────────

/// Snapshot statistics emitted each tick (JSON-serialisable for JS charts).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HospitalStats {
    pub tick: u64,
    /// Current in-patient census.
    pub census: usize,
    pub icu_occupancy: usize,
    pub ward_occupancy: usize,
    /// Cumulative HAI events since simulation start.
    pub cumulative_hai: u32,
    /// New HAI events in this tick.
    pub new_hai_this_tick: u32,
    pub mean_wbc: f32,
    pub mean_temperature: f32,
    pub mean_drug_conc: f32,
    /// SPRT log-likelihood ratio (outbreak detection).
    pub sprt_log_lr: f32,
    /// True if SPRT signalled outbreak this tick.
    pub outbreak_signal: bool,
    /// Fraction of nurses who performed hand hygiene in the last tick.
    pub hh_compliance_rate: f32,
    /// Counts by disease state [Susceptible, Exposed, Colonised, Infected,
    ///   Treated, Recovered, Dead].
    pub disease_state_counts: [u32; 7],
}
