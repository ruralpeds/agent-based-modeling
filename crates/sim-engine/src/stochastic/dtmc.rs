/// Discrete-Time Markov Chain for patient disease state progression.
/// The transition matrix is built externally from agent covariates (Phase 2)
/// so this module stays free of PatientAgent dependencies.
use crate::rng::Mulberry32;
use crate::stochastic::distributions::categorical_draw;

// ─── Disease states ───────────────────────────────────────────────────────────

/// Seven-state HAI patient disease progression model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum DiseaseState {
    Susceptible = 0,
    Exposed     = 1,
    Colonized   = 2,
    Infected    = 3,
    Treated     = 4,
    Recovered   = 5,
    Dead        = 6,
}

pub const N_DISEASE_STATES: usize = 7;

impl DiseaseState {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Susceptible,
            1 => Self::Exposed,
            2 => Self::Colonized,
            3 => Self::Infected,
            4 => Self::Treated,
            5 => Self::Recovered,
            _ => Self::Dead,
        }
    }

    pub fn as_index(self) -> usize {
        self as usize
    }

    /// Dead is the only absorbing state in the HAI model.
    pub fn is_absorbing(self) -> bool {
        matches!(self, Self::Dead)
    }
}

// ─── Generic DTMC sampler ─────────────────────────────────────────────────────

/// Generic N-state DTMC.  The transition matrix is kept external so the caller
/// can rebuild it from agent covariates each tick without allocating here.
pub struct PatientDTMC {
    pub n: usize,
}

impl PatientDTMC {
    pub fn new(n: usize) -> Self {
        Self { n }
    }

    /// Draw the next state index given a pre-computed row-major n×n matrix.
    /// `current` must be in 0..n.
    pub fn sample_next_state(
        &self,
        current: usize,
        matrix: &[f32],
        rng: &mut Mulberry32,
    ) -> usize {
        debug_assert!(current < self.n, "current state {} out of bounds (n={})", current, self.n);
        debug_assert_eq!(matrix.len(), self.n * self.n, "matrix size mismatch");
        let row_start = current * self.n;
        categorical_draw(&matrix[row_start..row_start + self.n], rng)
    }

    /// Convenience: sample using DiseaseState enum.
    pub fn sample_next_disease_state(
        &self,
        current: DiseaseState,
        matrix: &[f32],
        rng: &mut Mulberry32,
    ) -> DiseaseState {
        DiseaseState::from_index(self.sample_next_state(current.as_index(), matrix, rng))
    }
}

// ─── HAI DTMC matrix builder ──────────────────────────────────────────────────

/// Build a flat 7×7 row-major transition matrix for the HAI patient DTMC.
///
/// Rates (per tick, typically one 24-hour period):
/// - `lambda`   — force of infection   S → E
/// - `alpha`    — colonization rate    E → Colonized
/// - `delta_e`  — death/discharge      E → Dead
/// - `beta`     — progression          Colonized → Infected
/// - `gamma`    — tx initiation        Colonized → Treated
/// - `eta`      — tx initiation        Infected  → Treated
/// - `psi`      — mortality            Infected  → Dead
/// - `rho`      — recovery             Treated   → Recovered
/// - `phi`      — mortality            Treated   → Dead
/// - `theta`    — late mortality       Recovered → Dead
///
/// Rows are renormalized to sum to 1 to absorb floating-point drift.
#[allow(clippy::too_many_arguments)]
pub fn build_hai_dtmc_matrix(
    lambda:  f32,
    alpha:   f32,
    delta_e: f32,
    beta:    f32,
    gamma:   f32,
    eta:     f32,
    psi:     f32,
    rho:     f32,
    phi:     f32,
    theta:   f32,
) -> Vec<f32> {
    use DiseaseState::*;
    let n = N_DISEASE_STATES;
    let mut p = vec![0.0f32; n * n];

    let row = |s: DiseaseState| s.as_index() * n;

    // Row 0: Susceptible
    p[row(Susceptible) + Exposed as usize]     = lambda.min(1.0);
    p[row(Susceptible) + Susceptible as usize] = (1.0 - lambda).max(0.0);

    // Row 1: Exposed
    let leave_e = (alpha + delta_e).min(1.0);
    p[row(Exposed) + Colonized as usize] = alpha.min(leave_e);
    p[row(Exposed) + Dead as usize]      = delta_e.min(leave_e);
    p[row(Exposed) + Exposed as usize]   = (1.0 - leave_e).max(0.0);

    // Row 2: Colonized
    let leave_col = (beta + gamma).min(1.0);
    p[row(Colonized) + Infected as usize]   = beta.min(leave_col);
    p[row(Colonized) + Treated as usize]    = gamma.min(leave_col);
    p[row(Colonized) + Colonized as usize]  = (1.0 - leave_col).max(0.0);

    // Row 3: Infected
    let leave_inf = (eta + psi).min(1.0);
    p[row(Infected) + Treated as usize]  = eta.min(leave_inf);
    p[row(Infected) + Dead as usize]     = psi.min(leave_inf);
    p[row(Infected) + Infected as usize] = (1.0 - leave_inf).max(0.0);

    // Row 4: Treated
    let leave_tx = (rho + phi).min(1.0);
    p[row(Treated) + Recovered as usize] = rho.min(leave_tx);
    p[row(Treated) + Dead as usize]      = phi.min(leave_tx);
    p[row(Treated) + Treated as usize]   = (1.0 - leave_tx).max(0.0);

    // Row 5: Recovered
    p[row(Recovered) + Dead as usize]      = theta.min(1.0);
    p[row(Recovered) + Recovered as usize] = (1.0 - theta).max(0.0);

    // Row 6: Dead (absorbing)
    p[row(Dead) + Dead as usize] = 1.0;

    // Renormalize each row to correct floating-point drift.
    for r in 0..n {
        let s: f32 = p[r * n..r * n + n].iter().sum();
        if s > 0.0 {
            for c in 0..n {
                p[r * n + c] /= s;
            }
        }
    }

    p
}

/// Verify that every row of a matrix sums to 1 within tolerance.
/// Returns the max absolute row-sum error (should be < 1e-5).
pub fn max_row_sum_error(matrix: &[f32], n: usize) -> f32 {
    let mut max_err = 0.0f32;
    for r in 0..n {
        let s: f32 = matrix[r * n..r * n + n].iter().sum();
        max_err = max_err.max((s - 1.0).abs());
    }
    max_err
}
