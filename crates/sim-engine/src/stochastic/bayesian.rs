/// Bayesian belief-state update for patient colonization probability.
/// Provider agents maintain a Beta posterior that is revised as clinical
/// observations arrive (Bayes' theorem with binary likelihood).
///
/// Evidence-based sensitivity/specificity values are from published HAI
/// diagnostic studies (referenced inline).
// ─── Clinical observations ────────────────────────────────────────────────────

/// Binary clinical signs that trigger a Bayesian belief revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClinicalObservation {
    /// Temperature > 38.3 °C or < 36.0 °C.
    Fever,
    /// Laboratory culture positive for target pathogen.
    PositiveCulture,
    /// WBC > 12 × 10⁹/L or < 4 × 10⁹/L.
    AbnormalWbc,
    /// Purulent drainage at catheter or wound site.
    PurulentDrainage,
    /// Erythema or warmth at device insertion site.
    Erythema,
    /// New or worsening ventilator requirements.
    IncreasedVentSupport,
}

// ─── Likelihood parameters ────────────────────────────────────────────────────

/// Sensitivity and specificity of a single clinical observation for
/// detecting HAI colonization/infection.
#[derive(Debug, Clone, Copy)]
pub struct ObservationLikelihoods {
    /// P(observation present | patient colonized/infected)
    pub sensitivity: f32,
    /// P(observation absent  | patient susceptible)
    pub specificity: f32,
}

impl ObservationLikelihoods {
    /// Evidence-based defaults.  Sources: Bates et al. (JAMA), NHSN criteria.
    pub fn for_observation(obs: ClinicalObservation) -> Self {
        match obs {
            ClinicalObservation::Fever =>
                Self { sensitivity: 0.60, specificity: 0.75 },
            ClinicalObservation::PositiveCulture =>
                Self { sensitivity: 0.85, specificity: 0.97 },
            ClinicalObservation::AbnormalWbc =>
                Self { sensitivity: 0.55, specificity: 0.68 },
            ClinicalObservation::PurulentDrainage =>
                Self { sensitivity: 0.45, specificity: 0.92 },
            ClinicalObservation::Erythema =>
                Self { sensitivity: 0.38, specificity: 0.88 },
            ClinicalObservation::IncreasedVentSupport =>
                Self { sensitivity: 0.50, specificity: 0.80 },
        }
    }
}

// ─── Belief state ─────────────────────────────────────────────────────────────

/// Scalar Bayesian belief state for a single patient's HAI colonization
/// probability.  Prior is updated sequentially as observations arrive.
///
/// Uses a point-mass representation (scalar p) rather than a full Beta
/// distribution; this is sufficient for the binary colonized/susceptible model
/// and avoids the cost of maintaining conjugate hyperparameters.
#[derive(Debug, Clone)]
pub struct BayesianBeliefState {
    /// Current posterior P(colonized).  Always in (0, 1).
    pub p_colonized: f32,
    /// Posterior threshold above which the provider triggers isolation.
    pub isolation_threshold: f32,
    /// Posterior threshold above which a diagnostic culture is ordered.
    /// Defaults to 60% of the isolation threshold.
    pub culture_threshold: f32,
}

impl BayesianBeliefState {
    /// Create with the NHSN ICU admission colonization prevalence (~15%).
    pub fn new_icu_default() -> Self {
        Self {
            p_colonized:         0.15,
            isolation_threshold: 0.40,
            culture_threshold:   0.24,
        }
    }

    /// Create with an explicit prior and isolation threshold.
    pub fn new(p_colonized: f32, isolation_threshold: f32) -> Self {
        Self {
            p_colonized:         p_colonized.clamp(1e-3, 1.0 - 1e-3),
            isolation_threshold,
            culture_threshold:   isolation_threshold * 0.60,
        }
    }

    // ── Positive observation update ──────────────────────────────────────────

    /// Update on a positive (present) observation.
    ///
    /// P(C | obs+) = sens × p / [sens × p + (1-spec) × (1-p)]
    pub fn update_on_positive(&mut self, obs: ClinicalObservation) {
        let lik = ObservationLikelihoods::for_observation(obs);
        self.apply_likelihood(lik.sensitivity, 1.0 - lik.specificity);
    }

    /// Update on a negative (absent) observation.
    ///
    /// P(C | obs-) = (1-sens) × p / [(1-sens) × p + spec × (1-p)]
    pub fn update_on_negative(&mut self, obs: ClinicalObservation) {
        let lik = ObservationLikelihoods::for_observation(obs);
        self.apply_likelihood(1.0 - lik.sensitivity, lik.specificity);
    }

    /// Update with explicit P(obs | C) and P(obs | ¬C).
    pub fn update_with_likelihood(&mut self, p_obs_given_c: f32, p_obs_given_nc: f32) {
        self.apply_likelihood(p_obs_given_c, p_obs_given_nc);
    }

    fn apply_likelihood(&mut self, p_obs_given_c: f32, p_obs_given_nc: f32) {
        let p  = self.p_colonized;
        let num = p_obs_given_c  * p;
        let den = p_obs_given_c  * p + p_obs_given_nc * (1.0 - p);
        if den > 1e-8 {
            self.p_colonized = (num / den).clamp(1e-4, 1.0 - 1e-4);
        }
    }

    // ── Decision thresholds ──────────────────────────────────────────────────

    pub fn should_isolate(&self) -> bool {
        self.p_colonized >= self.isolation_threshold
    }

    pub fn should_culture(&self) -> bool {
        self.p_colonized >= self.culture_threshold
    }

    /// Reset to a new prior (e.g., at patient admission).
    pub fn reset(&mut self, base_rate: f32) {
        self.p_colonized = base_rate.clamp(1e-3, 1.0 - 1e-3);
    }
}
