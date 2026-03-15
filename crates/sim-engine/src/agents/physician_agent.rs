/// Physician agent: integrates pBDI deliberation, Thompson Sampling for
/// antibiotic selection, Prospect Theory for diagnostic test ordering,
/// and Fermi-imitation social learning for compliance.
use crate::rng::Mulberry32;
use crate::stochastic::bayesian::ClinicalObservation;
use super::types::{TreatmentAction, ProviderSocialState};
use crate::decision_rules::pbdi::PhysicianPBDI;
use crate::decision_rules::thompson_sampling::AntibioticBandit;
use crate::decision_rules::prospect_theory::ProspectTheoryParams;
use super::super::agents::types::AntibioticClass;

// ─── PhysicianAgent ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PhysicianAgent {
    pub id:              u32,
    pub unit_id:         u16,
    /// pBDI belief + deliberation engine.
    pub pbdi:            PhysicianPBDI,
    /// Multi-armed bandit for antibiotic selection.
    pub bandit:          AntibioticBandit,
    /// Prospect Theory parameters for diagnostic test ordering.
    pub prospect:        ProspectTheoryParams,
    /// Portable social-learning compliance state.
    pub social:          ProviderSocialState,
    /// Currently assigned patient IDs (up to 12 for ward attending).
    pub patient_ids:     [u32; 12],
    pub n_patients:      u8,
    /// Ticks since last clinical round.
    pub ticks_since_round: u32,
}

impl PhysicianAgent {
    pub fn new(id: u32, unit_id: u16, base_compliance: f32) -> Self {
        Self {
            id,
            unit_id,
            pbdi:             PhysicianPBDI::icu_default(),
            bandit:           AntibioticBandit::new(),
            prospect:         ProspectTheoryParams::default(),
            social:           ProviderSocialState::new(id, base_compliance),
            patient_ids:      [0; 12],
            n_patients:       0,
            ticks_since_round: 0,
        }
    }

    // ── Patient assignment ────────────────────────────────────────────────────

    /// Returns false if already at 12-patient capacity.
    pub fn assign_patient(&mut self, patient_id: u32) -> bool {
        if self.n_patients < 12 {
            self.patient_ids[self.n_patients as usize] = patient_id;
            self.n_patients += 1;
            true
        } else {
            false
        }
    }

    // ── pBDI belief update + deliberation ─────────────────────────────────────

    /// Update physician's disease-state belief given a clinical observation.
    pub fn observe_sign(&mut self, sign_idx: usize, present: bool) {
        self.pbdi.update_belief(sign_idx, present);
    }

    /// Convenience wrapper: update on a ClinicalObservation enum.
    pub fn observe_clinical(&mut self, obs: ClinicalObservation, present: bool) {
        let sign_idx = obs as usize;
        self.pbdi.update_belief(sign_idx, present);
    }

    /// Deliberate and return the recommended TreatmentAction.
    pub fn deliberate(&mut self) -> TreatmentAction {
        self.pbdi.deliberate()
    }

    // ── Antibiotic selection (Thompson Sampling) ──────────────────────────────

    pub fn select_antibiotic(&mut self, rng: &mut Mulberry32) -> AntibioticClass {
        self.bandit.select(rng)
    }

    pub fn update_antibiotic_outcome(&mut self, antibiotic: AntibioticClass, success: bool) {
        self.bandit.update(antibiotic, success);
    }

    // ── Diagnostic test ordering (Prospect Theory) ────────────────────────────

    /// Should this physician order a diagnostic test for the current patient?
    pub fn should_order_test(
        &self,
        obs: ClinicalObservation,
        p_colonized: f32,
    ) -> bool {
        self.prospect.should_order_observation(obs, p_colonized)
    }

    // ── Round clock ───────────────────────────────────────────────────────────

    pub fn tick_round_clock(&mut self) {
        self.ticks_since_round = self.ticks_since_round.saturating_add(1);
    }

    /// Reset round counter after completing a clinical round.
    pub fn complete_round(&mut self) {
        self.ticks_since_round = 0;
    }

    // ── Audit feedback ────────────────────────────────────────────────────────

    pub fn receive_audit_feedback(&mut self, target: f32) {
        self.social.received_feedback_this_period = true;
        let boost = 0.30 * (target - self.social.current_compliance).max(0.0);
        self.social.current_compliance =
            (self.social.current_compliance + boost).clamp(0.0, 1.0);
    }
}
