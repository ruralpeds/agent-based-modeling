use super::super::agents::types::{AntibioticClass, N_ANTIBIOTIC_CLASSES};
/// Thompson Sampling for antibiotic selection (Beta-Bernoulli bandit).
///
/// Each antibiotic arm maintains a Beta(α, β) posterior over its
/// efficacy (probability of clinical success). At each selection step,
/// one sample is drawn from each arm's posterior and the arm with the
/// highest sample is chosen (Thompson, 1933).
///
/// After observing an outcome, the winning arm is updated:
///   success: α += 1
///   failure: β += 1
use crate::rng::Mulberry32;
use crate::stochastic::distributions::beta_sample;

// ─── ThompsonAntibiotic ───────────────────────────────────────────────────────

/// Beta posterior for one antibiotic arm.
#[derive(Debug, Clone, Copy)]
pub struct ThompsonAntibiotic {
    pub antibiotic: AntibioticClass,
    /// Pseudo-successes (α > 0).
    pub alpha: f32,
    /// Pseudo-failures  (β > 0).
    pub beta: f32,
}

impl ThompsonAntibiotic {
    pub fn new(antibiotic: AntibioticClass) -> Self {
        // Uninformative Beta(1,1) = Uniform[0,1]
        Self {
            antibiotic,
            alpha: 1.0,
            beta: 1.0,
        }
    }

    /// Posterior mean efficacy estimate.
    pub fn mean_efficacy(&self) -> f32 {
        self.alpha / (self.alpha + self.beta)
    }

    /// Draw one sample from Beta(α, β) via ratio-of-gammas.
    pub fn sample_efficacy(&self, rng: &mut Mulberry32) -> f32 {
        beta_sample(self.alpha, self.beta, rng)
    }

    /// Update posterior after observing treatment outcome.
    pub fn update(&mut self, success: bool) {
        if success {
            self.alpha += 1.0;
        } else {
            self.beta += 1.0;
        }
    }
}

// ─── AntibioticBandit ─────────────────────────────────────────────────────────

/// Multi-armed bandit over all antibiotic classes using Thompson Sampling.
#[derive(Debug, Clone)]
pub struct AntibioticBandit {
    pub arms: [ThompsonAntibiotic; N_ANTIBIOTIC_CLASSES],
}

impl AntibioticBandit {
    /// Initialise all arms with Uniform[0,1] priors.
    pub fn new() -> Self {
        Self {
            arms: [
                ThompsonAntibiotic::new(AntibioticClass::BetaLactam),
                ThompsonAntibiotic::new(AntibioticClass::Vancomycin),
                ThompsonAntibiotic::new(AntibioticClass::Linezolid),
                ThompsonAntibiotic::new(AntibioticClass::Daptomycin),
                ThompsonAntibiotic::new(AntibioticClass::Cephalosporin),
            ],
        }
    }

    /// Thompson Sampling selection: draw from each Beta posterior, pick argmax.
    pub fn select(&mut self, rng: &mut Mulberry32) -> AntibioticClass {
        let mut best_class = self.arms[0].antibiotic;
        let mut best_sample = self.arms[0].sample_efficacy(rng);
        for arm in &self.arms[1..] {
            let s = arm.sample_efficacy(rng);
            if s > best_sample {
                best_sample = s;
                best_class = arm.antibiotic;
            }
        }
        best_class
    }

    /// Update the arm for `antibiotic` after observing `success`.
    pub fn update(&mut self, antibiotic: AntibioticClass, success: bool) {
        for arm in &mut self.arms {
            if arm.antibiotic == antibiotic {
                arm.update(success);
                return;
            }
        }
    }

    /// Posterior mean for a specific antibiotic.
    pub fn mean_efficacy(&self, antibiotic: AntibioticClass) -> f32 {
        for arm in &self.arms {
            if arm.antibiotic == antibiotic {
                return arm.mean_efficacy();
            }
        }
        0.0
    }
}

impl Default for AntibioticBandit {
    fn default() -> Self {
        Self::new()
    }
}
