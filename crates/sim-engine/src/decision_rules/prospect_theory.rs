/// Prospect Theory decision model (Kahneman & Tversky, 1979; TK, 1992).
///
/// Subjective value of outcome x relative to reference point r:
///   v(x) = (x - r)^α              if x ≥ r  (gain)
///   v(x) = −λ · (r - x)^β        if x < r  (loss)
///
/// Probability weighting (Prelec, 1998 one-parameter form):
///   w(p) = exp(−(−ln p)^γ)
///
/// Applied to diagnostic test ordering: a physician evaluates expected
/// subjective utility of ordering vs. not ordering a test.
use crate::stochastic::bayesian::ClinicalObservation;

// ─── Parameters ───────────────────────────────────────────────────────────────

/// Prospect Theory parameters (calibrated to Kahneman-Tversky empirical data).
#[derive(Debug, Clone, Copy)]
pub struct ProspectTheoryParams {
    /// Gain curvature α ∈ (0,1).  Default 0.88.
    pub alpha: f32,
    /// Loss curvature β ∈ (0,1).  Default 0.88.
    pub beta: f32,
    /// Loss aversion λ > 1.        Default 2.25.
    pub lambda: f32,
    /// Probability weighting curvature γ ∈ (0,1). Default 0.65.
    pub gamma: f32,
    /// Reference outcome (0 = no change in patient health score).
    pub reference: f32,
}

impl Default for ProspectTheoryParams {
    fn default() -> Self {
        Self {
            alpha: 0.88,
            beta: 0.88,
            lambda: 2.25,
            gamma: 0.65,
            reference: 0.0,
        }
    }
}

// ─── Core functions ──────────────────────────────────────────────────────────

impl ProspectTheoryParams {
    /// Subjective value of outcome `x` relative to `self.reference`.
    pub fn value(&self, x: f32) -> f32 {
        let delta = x - self.reference;
        if delta >= 0.0 {
            delta.powf(self.alpha)
        } else {
            -self.lambda * (-delta).powf(self.beta)
        }
    }

    /// Prelec (1998) probability weighting function.
    /// w(0) = 0, w(1) = 1, overweights small probabilities.
    pub fn weight(&self, p: f32) -> f32 {
        if p <= 0.0 {
            return 0.0;
        }
        if p >= 1.0 {
            return 1.0;
        }
        (-((-p.ln()).powf(self.gamma))).exp()
    }

    /// Expected subjective utility of a two-outcome prospect:
    ///   EU = w(p_gain)·v(gain_outcome) + w(1−p_gain)·v(loss_outcome)
    pub fn expected_utility(&self, p_gain: f32, gain_outcome: f32, loss_outcome: f32) -> f32 {
        self.weight(p_gain) * self.value(gain_outcome)
            + self.weight(1.0 - p_gain) * self.value(loss_outcome)
    }

    // ── Clinical application: test-ordering decision ──────────────────────────

    /// Should the physician order a diagnostic test for this patient?
    ///
    /// Compares EU(order) vs EU(no_order).
    ///
    /// `p_colonized`  — current posterior P(HAI | evidence).
    /// `test_benefit` — health utility gained if test detects true positive.
    /// `test_cost`    — disutility of ordering (patient burden, false alarm).
    /// `miss_cost`    — disutility of missing true infection (negative → loss).
    pub fn should_order_test(
        &self,
        p_colonized: f32,
        test_benefit: f32,
        test_cost: f32,
        miss_cost: f32,
    ) -> bool {
        // EU of ordering: detect with p_colonized → gain test_benefit; FP otherwise
        let eu_order = self.expected_utility(p_colonized, test_benefit, -test_cost);
        // EU of not ordering: miss HAI if colonized → loss miss_cost; no cost if not
        let eu_skip = self.expected_utility(1.0 - p_colonized, 0.0, -miss_cost);
        eu_order > eu_skip
    }

    /// Should the physician order `obs` type test at a given colonization prior?
    /// Uses hard-coded outcome values calibrated to ICU harm avoidance context.
    pub fn should_order_observation(&self, obs: ClinicalObservation, p_colonized: f32) -> bool {
        let (benefit, cost, miss) = observation_outcome_values(obs);
        self.should_order_test(p_colonized, benefit, cost, miss)
    }
}

/// Calibrated (benefit, cost, miss_cost) for each observation type.
/// Values on a 0–10 health utility scale; miss_cost is negative (loss framing).
fn observation_outcome_values(obs: ClinicalObservation) -> (f32, f32, f32) {
    match obs {
        ClinicalObservation::Fever => (3.0, 0.5, 4.0),
        ClinicalObservation::PositiveCulture => (8.0, 1.0, 9.0),
        ClinicalObservation::AbnormalWbc => (4.0, 0.5, 5.0),
        ClinicalObservation::PurulentDrainage => (6.0, 0.5, 7.0),
        ClinicalObservation::Erythema => (3.5, 0.5, 4.0),
        ClinicalObservation::IncreasedVentSupport => (5.0, 1.0, 6.0),
    }
}
