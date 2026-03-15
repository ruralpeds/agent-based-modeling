use super::types::{NurseActivity, ProviderSocialState};
/// Nurse agent with logistic compliance model, OU fatigue, and activity
/// state machine calibrated to WHO-5 hand hygiene audit data.
use crate::rng::Mulberry32;
use crate::stochastic::distributions::{exponential_sample, log_normal_sample};
use crate::stochastic::sde::OrnsteinUhlenbeck;

// ─── Logistic hand-hygiene parameters ────────────────────────────────────────

/// Logistic regression coefficients for the hand-hygiene compliance model.
/// p = σ(β₀ + β₁·workload + β₂·fatigue + β₃·dispenser_dist + β₄·ticks_since_audit)
///
/// Calibrated to WHO-5 ICU audit baseline ≈ 65% with workload/fatigue sensitivity
/// from published observational studies (Pittet et al.; Huis et al.).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct NurseParams {
    pub hh_intercept: f32,        // β₀: base log-odds → logit(0.65) ≈ 0.62
    pub hh_workload_coef: f32,    // β₁ < 0: higher workload → lower compliance
    pub hh_fatigue_coef: f32,     // β₂ < 0
    pub hh_distance_coef: f32,    // β₃ < 0: dispenser further away → lower
    pub hh_audit_decay_coef: f32, // β₄ < 0: compliance wanes since last audit
}

impl Default for NurseParams {
    fn default() -> Self {
        Self {
            hh_intercept: 0.62, // logit(0.65)
            hh_workload_coef: -0.35,
            hh_fatigue_coef: -0.50,
            hh_distance_coef: -0.15,
            hh_audit_decay_coef: -0.002,
        }
    }
}

// ─── NurseAgent ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct NurseAgent {
    pub id: u32,
    /// Patient load (0–6 ICU, 0–10 ward) — primary workload driver.
    pub workload: f32,
    /// Shift fatigue [0, 1]; updated via OU process each tick.
    pub fatigue: f32,
    /// Individual hand-hygiene tendency [0, 1] (drives hh_intercept offset).
    pub base_compliance: f32,
    /// Ticks since last audit-and-feedback event.
    pub ticks_since_audit: u32,
    /// Current activity (state machine).
    pub current_activity: NurseActivity,
    /// Remaining ticks in current activity.
    pub activity_ticks_left: u32,
    /// Assigned patient IDs (up to 6 for ICU).
    pub patient_ids: [u32; 6],
    pub n_patients: u8,
    /// Room assignment for proximity-based contact events.
    pub room_id: u16,
    /// Portable social-learning compliance state.
    pub social: ProviderSocialState,
}

impl NurseAgent {
    pub fn new(id: u32, base_compliance: f32, room_id: u16, rng: &mut Mulberry32) -> Self {
        Self {
            id,
            workload: 3.0 + rng.next_f32() * 2.0, // 3–5 patients on admission
            fatigue: 0.0,
            base_compliance: base_compliance.clamp(0.0, 1.0),
            ticks_since_audit: 0,
            current_activity: NurseActivity::BedsideCare,
            activity_ticks_left: 1,
            patient_ids: [0; 6],
            n_patients: 0,
            room_id,
            social: ProviderSocialState::new(id, base_compliance),
        }
    }

    // ── Hand hygiene compliance ───────────────────────────────────────────────

    /// Bernoulli draw for one hand-hygiene opportunity.
    ///
    /// p = σ(β₀ + β₁·workload + β₂·fatigue + β₃·dist + β₄·ticks_since_audit)
    pub fn hand_hygiene_compliance(
        &self,
        dispenser_dist: f32,
        params: &NurseParams,
        rng: &mut Mulberry32,
    ) -> bool {
        let lp = params.hh_intercept
            + params.hh_workload_coef * self.workload
            + params.hh_fatigue_coef * self.fatigue
            + params.hh_distance_coef * dispenser_dist
            + params.hh_audit_decay_coef * self.ticks_since_audit as f32;
        let prob = 1.0 / (1.0 + (-lp).exp());
        rng.next_f32() < prob
    }

    // ── Activity durations ────────────────────────────────────────────────────

    /// Sample activity duration in ticks (1 tick ≈ 5 min at 12 ticks/hour).
    pub fn sample_activity_duration(activity: NurseActivity, rng: &mut Mulberry32) -> u32 {
        let raw = match activity {
            NurseActivity::BedsideCare => log_normal_sample(0.88, 0.40, rng), // ~2.4 ticks
            NurseActivity::MedicationAdmin => log_normal_sample(0.10, 0.30, rng), // ~1.1 ticks
            NurseActivity::Assessment => log_normal_sample(0.40, 0.50, rng),  // ~1.5 ticks
            NurseActivity::Documentation => log_normal_sample(0.70, 0.60, rng), // ~2.0 ticks
            NurseActivity::Walking => exponential_sample(2.0, rng),           // ~0.5 ticks
        };
        (raw.ceil() as u32).max(1)
    }

    // ── Fatigue model ─────────────────────────────────────────────────────────

    /// Advance shift fatigue by one tick via Ornstein-Uhlenbeck.
    /// Setpoint rises with workload; recovery toward zero only between shifts.
    pub fn update_fatigue(&mut self, rng: &mut Mulberry32) {
        let setpoint = (self.workload / 8.0).clamp(0.0, 1.0);
        let ou = OrnsteinUhlenbeck {
            theta: 0.005,
            mu: setpoint,
            sigma: 0.01,
        };
        self.fatigue = ou.step_clamped(self.fatigue, 1.0, 0.0, 1.0, rng);
    }

    // ── Activity state machine ────────────────────────────────────────────────

    /// Advance by one tick; returns `true` when the activity changes
    /// (= potential contact event with assigned patient).
    pub fn advance_activity(&mut self, rng: &mut Mulberry32) -> bool {
        if self.activity_ticks_left == 0 {
            self.current_activity = self.next_activity(rng);
            self.activity_ticks_left = Self::sample_activity_duration(self.current_activity, rng);
            true
        } else {
            self.activity_ticks_left -= 1;
            false
        }
    }

    fn next_activity(&self, rng: &mut Mulberry32) -> NurseActivity {
        const WEIGHTS: [f32; 5] = [0.35, 0.25, 0.15, 0.15, 0.10];
        const ACTS: [NurseActivity; 5] = [
            NurseActivity::BedsideCare,
            NurseActivity::MedicationAdmin,
            NurseActivity::Assessment,
            NurseActivity::Documentation,
            NurseActivity::Walking,
        ];
        let u = rng.next_f32();
        let mut cum = 0.0f32;
        for (i, &w) in WEIGHTS.iter().enumerate() {
            cum += w;
            if u < cum {
                return ACTS[i];
            }
        }
        NurseActivity::BedsideCare
    }

    // ── Patient assignment ────────────────────────────────────────────────────

    /// Returns false if the nurse is already at capacity (6 patients).
    pub fn assign_patient(&mut self, patient_id: u32) -> bool {
        if self.n_patients < 6 {
            self.patient_ids[self.n_patients as usize] = patient_id;
            self.n_patients += 1;
            self.workload = self.n_patients as f32;
            true
        } else {
            false
        }
    }

    // ── Audit clock ───────────────────────────────────────────────────────────

    pub fn tick_audit_clock(&mut self) {
        self.ticks_since_audit = self.ticks_since_audit.saturating_add(1);
    }

    /// Apply audit-and-feedback: reset clock and boost base compliance.
    pub fn receive_audit_feedback(&mut self, target: f32) {
        self.ticks_since_audit = 0;
        self.social.received_feedback_this_period = true;
        let boost = 0.30 * (target - self.base_compliance).max(0.0);
        self.base_compliance = (self.base_compliance + boost).clamp(0.0, 1.0);
        self.social.current_compliance = self.base_compliance;
    }
}
