/// P1-14: Bayesian belief update numerical correctness.
/// Computes expected posteriors analytically and compares with model output.
#[cfg(test)]
mod tests {
    use crate::stochastic::bayesian::{
        BayesianBeliefState, ClinicalObservation, ObservationLikelihoods,
    };

    // ── Closed-form reference ─────────────────────────────────────────────────

    /// P(C | obs+) = sens*p / [sens*p + (1-spec)*(1-p)]
    fn posterior_positive(prior: f32, sens: f32, spec: f32) -> f32 {
        let num = sens * prior;
        let den = sens * prior + (1.0 - spec) * (1.0 - prior);
        num / den
    }

    /// P(C | obs-) = (1-sens)*p / [(1-sens)*p + spec*(1-p)]
    fn posterior_negative(prior: f32, sens: f32, spec: f32) -> f32 {
        let num = (1.0 - sens) * prior;
        let den = (1.0 - sens) * prior + spec * (1.0 - prior);
        num / den
    }

    // ── P1-14: positive update ────────────────────────────────────────────────

    #[test]
    fn positive_fever_update_matches_analytic() {
        let prior = 0.15f32;
        let mut belief = BayesianBeliefState::new(prior, 0.40);
        let lik = ObservationLikelihoods::for_observation(ClinicalObservation::Fever);

        belief.update_on_positive(ClinicalObservation::Fever);

        let expected = posterior_positive(prior, lik.sensitivity, lik.specificity);
        let err = (belief.p_colonized - expected).abs();
        assert!(
            err < 1e-4,
            "Fever positive update: got {:.6}, expected {:.6}",
            belief.p_colonized, expected
        );
    }

    #[test]
    fn positive_culture_update_matches_analytic() {
        let prior = 0.15f32;
        let mut belief = BayesianBeliefState::new(prior, 0.40);
        let lik = ObservationLikelihoods::for_observation(ClinicalObservation::PositiveCulture);

        belief.update_on_positive(ClinicalObservation::PositiveCulture);

        let expected = posterior_positive(prior, lik.sensitivity, lik.specificity);
        let err = (belief.p_colonized - expected).abs();
        assert!(err < 1e-4, "Culture positive update: got {:.6}, expected {:.6}", belief.p_colonized, expected);
    }

    // ── Negative update ───────────────────────────────────────────────────────

    #[test]
    fn negative_fever_update_matches_analytic() {
        let prior = 0.30f32;
        let mut belief = BayesianBeliefState::new(prior, 0.40);
        let lik = ObservationLikelihoods::for_observation(ClinicalObservation::Fever);

        belief.update_on_negative(ClinicalObservation::Fever);

        let expected = posterior_negative(prior, lik.sensitivity, lik.specificity);
        let err = (belief.p_colonized - expected).abs();
        assert!(err < 1e-4, "Fever negative update: got {:.6}, expected {:.6}", belief.p_colonized, expected);
    }

    // ── Sequential updates ────────────────────────────────────────────────────

    #[test]
    fn sequential_updates_monotone_increase() {
        let mut belief = BayesianBeliefState::new(0.10, 0.40);
        let p0 = belief.p_colonized;

        belief.update_on_positive(ClinicalObservation::Fever);
        let p1 = belief.p_colonized;

        belief.update_on_positive(ClinicalObservation::AbnormalWbc);
        let p2 = belief.p_colonized;

        belief.update_on_positive(ClinicalObservation::PositiveCulture);
        let p3 = belief.p_colonized;

        assert!(p1 > p0, "Fever should raise P(C)");
        assert!(p2 > p1, "AbnormalWbc should further raise P(C)");
        assert!(p3 > p2, "PositiveCulture should further raise P(C)");
        assert!(p3 <= 1.0, "Posterior must stay ≤ 1");
    }

    #[test]
    fn negative_updates_lower_probability() {
        let mut belief = BayesianBeliefState::new(0.50, 0.40);
        belief.update_on_negative(ClinicalObservation::Fever);
        belief.update_on_negative(ClinicalObservation::AbnormalWbc);
        assert!(belief.p_colonized < 0.50, "Negative observations should lower P(C)");
    }

    // ── Decision thresholds ───────────────────────────────────────────────────

    #[test]
    fn isolation_threshold_triggers_correctly() {
        let mut belief = BayesianBeliefState::new_icu_default();
        assert!(!belief.should_isolate(), "P=0.15 should not trigger isolation (threshold=0.40)");

        // Multiple positive observations should push over threshold.
        for _ in 0..5 {
            belief.update_on_positive(ClinicalObservation::Fever);
        }
        assert!(belief.should_isolate(), "High posterior should trigger isolation");
    }

    #[test]
    fn culture_threshold_lower_than_isolation() {
        let belief = BayesianBeliefState::new_icu_default();
        assert!(
            belief.culture_threshold < belief.isolation_threshold,
            "Culture threshold should be below isolation threshold"
        );
    }

    #[test]
    fn reset_restores_prior() {
        let mut belief = BayesianBeliefState::new(0.15, 0.40);
        belief.update_on_positive(ClinicalObservation::PositiveCulture);
        assert!(belief.p_colonized > 0.15);
        belief.reset(0.15);
        assert!((belief.p_colonized - 0.15).abs() < 1e-4);
    }
}
