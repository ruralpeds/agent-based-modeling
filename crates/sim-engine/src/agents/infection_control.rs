/// Infection Control Agent: SPRT-based outbreak detection and per-patient
/// Bayesian colonization belief tracking.
///
/// SPRT (Wald 1947): compares H₁ (outbreak rate λ₁) vs H₀ (baseline λ₀).
///   Λₙ = Σ log(P(xᵢ|H₁)/P(xᵢ|H₀))
///   Stop if Λₙ ≥ log((1-β)/α)  → signal outbreak
///           Λₙ ≤ log(β/(1-α))  → accept baseline
use std::collections::HashMap;
use crate::stochastic::bayesian::{BayesianBeliefState, ClinicalObservation};

// ─── SPRT ─────────────────────────────────────────────────────────────────────

/// Decision returned by each SPRT update.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SprtDecision {
    /// Continue monitoring — no boundary crossed.
    Continue,
    /// Upper boundary crossed — signal outbreak.
    SignalOutbreak,
    /// Lower boundary crossed — accept baseline (rare in one-sided test).
    AcceptBaseline,
}

/// One-sided Poisson SPRT for HAI outbreak detection.
///
/// H₀: cases arrive at baseline rate λ₀ (per patient-day).
/// H₁: cases arrive at elevated rate λ₁ = outbreak_multiplier × λ₀.
#[derive(Debug, Clone)]
pub struct SprtMonitor {
    /// Baseline expected case rate (per patient-day).
    pub lambda_0:           f32,
    /// Outbreak rate multiplier (H₁ = multiplier × λ₀).
    pub outbreak_multiplier: f32,
    /// Type-I error target (false-positive rate).
    pub alpha:              f32,
    /// Type-II error target (false-negative rate).
    pub beta:               f32,
    /// Running log-likelihood ratio.
    pub log_lr:             f32,
    /// Cumulative cases observed in this monitoring window.
    pub case_count:         u32,
    /// Cumulative patient-days observed in this window.
    pub patient_days:       f32,
}

impl SprtMonitor {
    pub fn new(lambda_0: f32, outbreak_multiplier: f32, alpha: f32, beta: f32) -> Self {
        assert!(lambda_0 > 0.0,             "lambda_0 must be positive");
        assert!(outbreak_multiplier > 1.0,  "multiplier must exceed 1");
        assert!(alpha > 0.0 && alpha < 1.0, "alpha must be in (0,1)");
        assert!(beta  > 0.0 && beta  < 1.0, "beta must be in (0,1)");
        Self {
            lambda_0, outbreak_multiplier, alpha, beta,
            log_lr:       0.0,
            case_count:   0,
            patient_days: 0.0,
        }
    }

    /// Default ICU monitor: baseline 0.02 cases/patient-day, 2× outbreak.
    pub fn icu_default() -> Self {
        Self::new(0.02, 2.0, 0.05, 0.10)
    }

    /// Upper boundary: log((1-β)/α).
    pub fn upper_boundary(&self) -> f32 {
        ((1.0 - self.beta) / self.alpha).ln()
    }

    /// Lower boundary: log(β/(1-α)).
    pub fn lower_boundary(&self) -> f32 {
        (self.beta / (1.0 - self.alpha)).ln()
    }

    /// Observe one patient-day interval with `new_cases` HAI events.
    /// Returns the SPRT decision after updating the log-likelihood ratio.
    pub fn observe_case(&mut self, new_cases: u32, patient_days_delta: f32) -> SprtDecision {
        let lambda_1 = self.lambda_0 * self.outbreak_multiplier;
        let expected_0 = self.lambda_0 * patient_days_delta;
        let expected_1 = lambda_1     * patient_days_delta;

        // Poisson log-likelihood ratio increment: n·log(λ₁/λ₀) − (λ₁−λ₀)·t
        let increment = new_cases as f32 * (lambda_1 / self.lambda_0).ln()
                        - (expected_1 - expected_0);
        self.log_lr      += increment;
        self.case_count  += new_cases;
        self.patient_days += patient_days_delta;

        if self.log_lr >= self.upper_boundary() {
            SprtDecision::SignalOutbreak
        } else if self.log_lr <= self.lower_boundary() {
            SprtDecision::AcceptBaseline
        } else {
            SprtDecision::Continue
        }
    }

    /// Reset SPRT after a decision or periodic restart.
    pub fn reset_sprt(&mut self) {
        self.log_lr       = 0.0;
        self.case_count   = 0;
        self.patient_days = 0.0;
    }
}

// ─── InfectionControlAgent ────────────────────────────────────────────────────

/// Infection Control Practitioner: runs SPRT for ward-level outbreak detection
/// and maintains a per-patient Bayesian belief over colonization status.
#[derive(Debug, Clone)]
pub struct InfectionControlAgent {
    pub id:             u32,
    pub unit_id:        u16,
    /// SPRT monitor for this ward/unit.
    pub sprt:           SprtMonitor,
    /// Per-patient colonization beliefs (patient_id → BayesianBeliefState).
    pub patient_beliefs: HashMap<u32, BayesianBeliefState>,
    /// Ticks since last audit round was completed.
    pub ticks_since_audit: u32,
    /// Scheduled audit interval in ticks (default 12×24×7 = 1 week at 12t/h).
    pub audit_interval_ticks: u32,
}

impl InfectionControlAgent {
    pub fn new(id: u32, unit_id: u16) -> Self {
        Self {
            id,
            unit_id,
            sprt:                SprtMonitor::icu_default(),
            patient_beliefs:     HashMap::new(),
            ticks_since_audit:   0,
            audit_interval_ticks: 12 * 24 * 7, // weekly
        }
    }

    // ── Patient belief management ─────────────────────────────────────────────

    /// Register a new patient with ICU default prior.
    pub fn register_patient(&mut self, patient_id: u32) {
        self.patient_beliefs
            .entry(patient_id)
            .or_insert_with(BayesianBeliefState::new_icu_default);
    }

    pub fn remove_patient(&mut self, patient_id: u32) {
        self.patient_beliefs.remove(&patient_id);
    }

    /// Update belief for a patient given a positive clinical observation.
    pub fn update_positive(&mut self, patient_id: u32, obs: ClinicalObservation) {
        if let Some(belief) = self.patient_beliefs.get_mut(&patient_id) {
            belief.update_on_positive(obs);
        }
    }

    /// Update belief for a patient given a negative clinical observation.
    pub fn update_negative(&mut self, patient_id: u32, obs: ClinicalObservation) {
        if let Some(belief) = self.patient_beliefs.get_mut(&patient_id) {
            belief.update_on_negative(obs);
        }
    }

    /// Returns true if the patient's posterior exceeds the isolation threshold.
    pub fn should_isolate(&self, patient_id: u32) -> bool {
        self.patient_beliefs
            .get(&patient_id)
            .map(|b| b.should_isolate())
            .unwrap_or(false)
    }

    /// Returns true if the patient's posterior exceeds the culture threshold.
    pub fn should_culture(&self, patient_id: u32) -> bool {
        self.patient_beliefs
            .get(&patient_id)
            .map(|b| b.should_culture())
            .unwrap_or(false)
    }

    // ── SPRT delegation ───────────────────────────────────────────────────────

    pub fn observe_cases(&mut self, new_cases: u32, patient_days: f32) -> SprtDecision {
        self.sprt.observe_case(new_cases, patient_days)
    }

    pub fn reset_sprt(&mut self) {
        self.sprt.reset_sprt();
    }

    // ── Audit clock ───────────────────────────────────────────────────────────

    pub fn tick_audit_clock(&mut self) {
        self.ticks_since_audit = self.ticks_since_audit.saturating_add(1);
    }

    /// Returns true if an audit is due this tick.
    pub fn is_audit_due(&self) -> bool {
        self.ticks_since_audit >= self.audit_interval_ticks
    }

    pub fn complete_audit(&mut self) {
        self.ticks_since_audit = 0;
    }
}
