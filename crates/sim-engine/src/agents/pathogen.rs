/// Pathogen module: dose-response transmission probability, resistance
/// mutation events, and antibiotic efficacy look-up.
///
/// Transmission model (exponential dose-response, Haas 1983):
///   P(infect | dose) = 1 − exp(−r · dose · (1 − hand_wash_factor))
///
/// Resistance escalation (Poisson per antibiotic-day):
///   P(mutation) = 1 − exp(−mu · antibiotic_days)
use serde::{Deserialize, Serialize};
use crate::rng::Mulberry32;
use super::types::{ResistanceProfile, AntibioticClass};

// ─── PathogenParams ───────────────────────────────────────────────────────────

/// Calibration constants for S. aureus / MRSA-like pathogen (ICU default).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PathogenParams {
    /// Exponential dose-response rate r (units: 1/CFU-equivalent).
    /// Default 0.05 → P≈0.40 at dose=10.
    pub dose_response_r: f32,
    /// Fraction of dose blocked when hand hygiene is performed [0, 1].
    pub hand_wash_factor: f32,
    /// Per-antibiotic-day probability of resistance mutation (Poisson rate).
    pub mutation_rate_per_ab_day: f32,
}

impl Default for PathogenParams {
    fn default() -> Self {
        Self {
            dose_response_r:          0.05,
            hand_wash_factor:         0.90,
            mutation_rate_per_ab_day: 0.002,
        }
    }
}

// ─── Transmission ────────────────────────────────────────────────────────────

/// P(transmission) using exponential dose-response (Haas 1983).
/// `dose` is the exposure inoculum (arbitrary units, ≥ 0).
/// `hand_washed` blocks `hand_wash_factor` fraction of the dose.
pub fn transmission_probability(
    dose: f32,
    hand_washed: bool,
    params: &PathogenParams,
) -> f32 {
    if dose <= 0.0 { return 0.0; }
    let effective_dose = if hand_washed {
        dose * (1.0 - params.hand_wash_factor)
    } else {
        dose
    };
    1.0 - (-params.dose_response_r * effective_dose).exp()
}

/// Bernoulli draw: did transmission occur?
pub fn transmission_event(
    dose: f32,
    hand_washed: bool,
    params: &PathogenParams,
    rng: &mut Mulberry32,
) -> bool {
    rng.next_f32() < transmission_probability(dose, hand_washed, params)
}

// ─── Resistance mutation ─────────────────────────────────────────────────────

/// Bernoulli draw for a resistance escalation event after `antibiotic_days`
/// cumulative days of antibiotic pressure.
/// P = 1 − exp(−mu · antibiotic_days).
pub fn resistance_mutation_event(
    antibiotic_days: u16,
    params: &PathogenParams,
    rng: &mut Mulberry32,
) -> bool {
    let prob = 1.0 - (-params.mutation_rate_per_ab_day * antibiotic_days as f32).exp();
    rng.next_f32() < prob
}

/// Attempt to escalate resistance by one tier; returns new profile.
/// FullySusceptible → MethicillinR → VancomycinIntermediate → Pandrug.
pub fn escalate_resistance(current: ResistanceProfile) -> ResistanceProfile {
    match current {
        ResistanceProfile::FullySusceptible       => ResistanceProfile::MethicillinR,
        ResistanceProfile::MethicillinR           => ResistanceProfile::VancomycinIntermediate,
        ResistanceProfile::VancomycinIntermediate => ResistanceProfile::Pandrug,
        ResistanceProfile::Pandrug                => ResistanceProfile::Pandrug,
    }
}

// ─── Antibiotic efficacy ─────────────────────────────────────────────────────

/// Effective kill-rate multiplier for `antibiotic` against `resistance`.
/// Returns 0.0 if resistant, 1.0 if fully susceptible, 0.5 for intermediate.
pub fn antibiotic_efficacy(resistance: ResistanceProfile, antibiotic: AntibioticClass) -> f32 {
    match (resistance, antibiotic) {
        (ResistanceProfile::FullySusceptible, _)  => 1.0,
        (ResistanceProfile::Pandrug, _)           => 0.0,
        _ => {
            if resistance.is_susceptible_to(antibiotic) { 1.0 } else { 0.0 }
        }
    }
}
