/// Parametric survival distributions for healthcare length-of-stay and
/// time-to-event outcomes.  Calibrated to HCUP NIS and SCCM data.
use crate::rng::Mulberry32;
use crate::stochastic::distributions::{standard_normal, exponential_sample, gamma_fn};

// ─── Cox PH covariates ────────────────────────────────────────────────────────

/// Patient covariates for the Cox Proportional Hazards LOS model.
#[derive(Debug, Clone, Copy, Default)]
pub struct CoxCovariates {
    pub age:                  f32,  // years
    pub charlson_index:       f32,  // 0–37
    pub admission_severity:   f32,  // e.g. SOFA 0–24
    pub ventilated:           bool,
    pub central_line:         bool,
}

/// Coefficients for the Cox PH model (calibrated from HCUP NIS data).
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct CoxParams {
    pub age_coef:       f32,  // per year above 65
    pub cci_coef:       f32,  // per CCI point
    pub severity_coef:  f32,  // per SOFA point
    pub vent_coef:      f32,  // ventilated indicator
    pub cline_coef:     f32,  // central-line indicator
}

impl Default for CoxParams {
    fn default() -> Self {
        Self {
            age_coef:      0.02,
            cci_coef:      0.08,
            severity_coef: 0.10,
            vent_coef:     0.45,
            cline_coef:    0.25,
        }
    }
}

/// Compute the Cox PH hazard ratio for a patient.
/// HR > 1 → event sooner (shorter LOS); HR < 1 → event later.
pub fn cox_hazard_ratio(cov: &CoxCovariates, params: &CoxParams) -> f32 {
    let lp = params.age_coef      * (cov.age - 65.0)
           + params.cci_coef      * cov.charlson_index
           + params.severity_coef * cov.admission_severity
           + params.vent_coef     * cov.ventilated as u8 as f32
           + params.cline_coef    * cov.central_line as u8 as f32;
    lp.exp()
}

// ─── Survival distributions ───────────────────────────────────────────────────

/// Parametric survival distributions for LOS and time-to-event modeling.
#[derive(Debug, Clone, Copy)]
pub enum SurvivalDistribution {
    /// Constant hazard: T ~ Exp(lambda).  Mean = 1/lambda.
    Exponential { lambda: f32 },

    /// Monotone hazard: T ~ Weibull(shape, scale).
    /// k < 1 → decreasing hazard; k > 1 → increasing hazard.
    /// Mean = scale × Γ(1 + 1/k).
    Weibull { shape: f32, scale: f32 },

    /// Non-monotone (hump-shaped) hazard: T ~ LogNormal(mu, sigma).
    /// Mean = exp(mu + σ²/2).
    LogNormal { mu: f32, sigma: f32 },
}

impl SurvivalDistribution {
    // ── Pre-calibrated constructors ──────────────────────────────────────────

    /// ICU LOS: Weibull(k=0.85, λ=120h).  Mean ≈ 128 h.  Source: SCCM.
    pub fn icu_los() -> Self {
        Self::Weibull { shape: 0.85, scale: 120.0 }
    }

    /// General ward LOS: LogNormal(μ=2.8, σ=0.9).  Mean ≈ 23 h.
    pub fn ward_los() -> Self {
        Self::LogNormal { mu: 2.8, sigma: 0.9 }
    }

    /// ED LOS for ESI-3 patients (NHAMCS): LogNormal(μ=0.90, σ=0.55).
    /// Mean ≈ 2.7 h.
    pub fn ed_los_esi3() -> Self {
        Self::LogNormal { mu: 0.90, sigma: 0.55 }
    }

    // ── Sampling ─────────────────────────────────────────────────────────────

    /// Draw one time-to-event sample via inverse CDF.
    /// Result clamped to [1e-3, 1e6] for numerical stability.
    pub fn sample(&self, rng: &mut Mulberry32) -> f32 {
        let raw = match *self {
            Self::Exponential { lambda } => exponential_sample(lambda, rng),

            Self::Weibull { shape, scale } => {
                // F⁻¹(u) = scale × (-ln(1-u))^(1/shape) ≡ scale × (-ln(u))^(1/shape)
                let u = rng.next_f32().max(1e-7);
                scale * (-u.ln()).powf(1.0 / shape)
            }

            Self::LogNormal { mu, sigma } => {
                let z = standard_normal(rng);
                (mu + sigma * z).exp()
            }
        };
        raw.clamp(1e-3, 1e6)
    }

    /// Sample with Cox PH covariate adjustment.
    /// Divides the baseline sample by the hazard ratio (proportional hazards ⟹
    /// accelerated failure time is exact for Weibull; approximate for others).
    pub fn sample_adjusted(&self, hr: f32, rng: &mut Mulberry32) -> f32 {
        (self.sample(rng) / hr.max(1e-2)).clamp(1e-3, 1e6)
    }

    // ── Moments ───────────────────────────────────────────────────────────────

    /// Theoretical mean.
    pub fn mean(&self) -> f32 {
        match *self {
            Self::Exponential { lambda } => 1.0 / lambda,
            Self::Weibull { shape, scale } => scale * gamma_fn(1.0 + 1.0 / shape),
            Self::LogNormal { mu, sigma } => (mu + 0.5 * sigma * sigma).exp(),
        }
    }

    /// Theoretical variance.
    pub fn variance(&self) -> f32 {
        match *self {
            Self::Exponential { lambda } => 1.0 / (lambda * lambda),
            Self::Weibull { shape, scale } => {
                let g1 = gamma_fn(1.0 + 1.0 / shape);
                let g2 = gamma_fn(1.0 + 2.0 / shape);
                scale * scale * (g2 - g1 * g1)
            }
            Self::LogNormal { mu, sigma } => {
                let s2 = sigma * sigma;
                let e2m = (2.0 * mu + s2).exp();
                e2m * (s2.exp() - 1.0)
            }
        }
    }
}
