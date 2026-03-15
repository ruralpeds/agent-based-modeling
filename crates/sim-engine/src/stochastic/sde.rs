/// Stochastic differential equation solvers for physiological state variables
/// and pharmacokinetic drug concentration models.
///
/// Uses the Euler-Maruyama scheme (first-order strong convergence) which is
/// sufficient for f32 precision over the short time steps used in agent ticks.
use crate::rng::Mulberry32;
use crate::stochastic::distributions::standard_normal;

// ─── Ornstein-Uhlenbeck process ───────────────────────────────────────────────

/// Mean-reverting stochastic process for physiological variables.
///
/// SDE:  dX = θ(μ - X) dt + σ dW
///
/// - θ  mean reversion speed (1/time)
/// - μ  long-run equilibrium / homeostatic setpoint
/// - σ  diffusion coefficient (noise intensity)
///
/// Euler-Maruyama step: X(t+dt) = X(t) + θ(μ - X(t))dt + σ√dt · Z,  Z ~ N(0,1)
#[derive(Debug, Clone, Copy)]
pub struct OrnsteinUhlenbeck {
    pub theta: f32,
    pub mu: f32,
    pub sigma: f32,
}

impl OrnsteinUhlenbeck {
    /// WBC count in 10⁹/L (setpoint 7.5, mild physiological noise).
    pub fn wbc() -> Self {
        Self {
            theta: 0.10,
            mu: 7.5,
            sigma: 0.30,
        }
    }

    /// Core body temperature in °C (setpoint 37.0 °C).
    pub fn temperature() -> Self {
        Self {
            theta: 0.15,
            mu: 37.0,
            sigma: 0.05,
        }
    }

    /// Drug plasma concentration normalized to [0, 1] (setpoint 0 = cleared).
    /// Without dosing, concentration decays to 0; noise reflects assay variability.
    pub fn drug_concentration() -> Self {
        Self {
            theta: 0.20,
            mu: 0.0,
            sigma: 0.02,
        }
    }

    /// Nurse shift fatigue (setpoint 0 = rested; slow reversion during shift).
    pub fn nurse_fatigue() -> Self {
        Self {
            theta: 0.005,
            mu: 0.0,
            sigma: 0.01,
        }
    }

    /// Euler-Maruyama step.  `dt` is in ticks (1 tick = 1 simulation time unit).
    pub fn step(&self, x: f32, dt: f32, rng: &mut Mulberry32) -> f32 {
        let drift = self.theta * (self.mu - x) * dt;
        let diffusion = self.sigma * dt.sqrt() * standard_normal(rng);
        x + drift + diffusion
    }

    /// Step with hard clamp to a physiologically plausible interval [lo, hi].
    pub fn step_clamped(&self, x: f32, dt: f32, lo: f32, hi: f32, rng: &mut Mulberry32) -> f32 {
        self.step(x, dt, rng).clamp(lo, hi)
    }

    /// Stationary variance of the OU process: σ² / (2θ).
    pub fn stationary_variance(&self) -> f32 {
        self.sigma * self.sigma / (2.0 * self.theta)
    }
}

// ─── One-compartment pharmacokinetic model ────────────────────────────────────

/// First-order one-compartment PK model with IV bolus dosing.
///
/// C(t+dt) = C(t) · exp(-ke · dt) + (dose_mg / V)
///
/// - ke = CL / V  (elimination rate constant, 1/hour)
/// - V            (volume of distribution, L)
#[derive(Debug, Clone, Copy)]
pub struct OneCompartmentPk {
    /// Elimination rate constant ke = CL/V  (1/hour).
    pub ke: f32,
    /// Volume of distribution (L).
    pub volume: f32,
}

impl OneCompartmentPk {
    /// Typical IV antibiotic (vancomycin-like) parameters.
    pub fn antibiotic_iv() -> Self {
        Self {
            ke: 0.35,
            volume: 10.0,
        }
    }

    /// Oral antibiotic with ~60% bioavailability (approx as lower effective ke).
    pub fn antibiotic_oral() -> Self {
        Self {
            ke: 0.18,
            volume: 15.0,
        }
    }

    /// Advance concentration by one time step `dt` (hours).
    /// `dose_mg` is the IV bolus delivered at the start of this tick (0.0 if no dose).
    pub fn step(&self, concentration: f32, dt: f32, dose_mg: f32) -> f32 {
        let decay = concentration * (-self.ke * dt).exp();
        let input = dose_mg / self.volume;
        (decay + input).max(0.0)
    }

    /// True if concentration is at or above the minimum inhibitory concentration.
    pub fn above_mic(&self, concentration: f32, mic: f32) -> bool {
        concentration >= mic
    }

    /// Steady-state concentration for a repeated dose `dose_mg` every `interval_h` hours.
    pub fn steady_state_trough(&self, dose_mg: f32, interval_h: f32) -> f32 {
        let factor = (-self.ke * interval_h).exp();
        (dose_mg / self.volume) * factor / (1.0 - factor)
    }
}
