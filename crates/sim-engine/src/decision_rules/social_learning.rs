use crate::agents::types::ProviderSocialState;
/// Fermi imitation social learning for hand-hygiene compliance.
///
/// P(imitate peer j) = σ(k · (compliance_j − compliance_i))
/// where σ is the logistic function and k is the selection intensity.
///
/// Applied each social-learning period: a provider samples one peer from its
/// reference group and may adopt the peer's compliance level if the gap is
/// favourable and the Bernoulli draw succeeds.
///
/// Audit-and-feedback boost: if the provider received audit feedback this
/// period, compliance is nudged toward the target by a fraction `audit_boost`.
use crate::rng::Mulberry32;

// ─── SocialLearningParams ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct SocialLearningParams {
    /// Fermi selection intensity k > 0.  Higher → sharper imitation threshold.
    pub selection_intensity: f32,
    /// Fraction of compliance gap covered per audit-feedback event [0,1].
    pub audit_boost: f32,
    /// Target compliance after audit (e.g. 0.85 for WHO-5 Gold Standard).
    pub audit_target: f32,
}

impl Default for SocialLearningParams {
    fn default() -> Self {
        Self {
            selection_intensity: 5.0,
            audit_boost: 0.30,
            audit_target: 0.85,
        }
    }
}

// ─── Core update ─────────────────────────────────────────────────────────────

/// Run one social-learning step for `self_state` given a sampled peer state.
///
/// Mutates `self_state.current_compliance` in place.
/// Also applies audit-and-feedback boost if `received_feedback_this_period`,
/// then resets the flag.
pub fn social_learning_step(
    self_state: &mut ProviderSocialState,
    peer_state: &ProviderSocialState,
    params: &SocialLearningParams,
    rng: &mut Mulberry32,
) {
    // ── Fermi imitation ───────────────────────────────────────────────────────
    let gap = peer_state.current_compliance - self_state.current_compliance;
    let p_imitate = logistic(params.selection_intensity * gap);
    if rng.next_f32() < p_imitate {
        // Adopt peer's compliance
        self_state.current_compliance = peer_state.current_compliance;
    }

    // ── Audit-and-feedback boost ──────────────────────────────────────────────
    if self_state.received_feedback_this_period {
        let boost =
            params.audit_boost * (params.audit_target - self_state.current_compliance).max(0.0);
        self_state.current_compliance = (self_state.current_compliance + boost).clamp(0.0, 1.0);
        self_state.received_feedback_this_period = false;
    }
}

/// Logistic function σ(x) = 1 / (1 + e^{-x}).
#[inline]
fn logistic(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

// ─── Convenience: population update ──────────────────────────────────────────

/// Update a single provider's compliance given the population mean.
/// Uses the population mean as the "sampled peer" — useful when individual
/// peer tracking is not needed.
pub fn social_learning_from_mean(
    self_state: &mut ProviderSocialState,
    population_mean: f32,
    params: &SocialLearningParams,
    rng: &mut Mulberry32,
) {
    let peer = ProviderSocialState {
        id: u32::MAX,
        current_compliance: population_mean,
        received_feedback_this_period: false,
    };
    social_learning_step(self_state, &peer, params, rng);
}
