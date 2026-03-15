/// Monte Carlo variance-reduction utilities.
///
/// - `CrnManager`      — Common Random Numbers for paired scenario comparison
/// - `antithetic_pair` — Draw (U, 1-U) for antithetic variates
/// - `monte_carlo_run` — N-replication runner returning (mean, std_error)
use crate::rng::Mulberry32;

// ─── Common Random Numbers ────────────────────────────────────────────────────

/// Manages independent Mulberry32 streams for Common Random Numbers (CRN).
///
/// CRN ensures that the *same* random inputs drive both the baseline and
/// intervention scenarios in a paired comparison, dramatically reducing the
/// variance of the estimated treatment effect:
///   Var(Y_tx - Y_ctrl) = Var(Y_tx) + Var(Y_ctrl) - 2·Cov(Y_tx, Y_ctrl)
///
/// Usage: call `reset()` at the start of each replication to re-synchronize
/// all streams to their original seeds.
pub struct CrnManager {
    streams:   Vec<Mulberry32>,
    base_seed: u32,
}

impl CrnManager {
    /// Create `n_streams` independent streams derived from `base_seed`.
    /// Each stream i has seed = base_seed ⊕ (i × 0x9e3779b9) to ensure
    /// maximal separation in state space.
    pub fn new(base_seed: u32, n_streams: usize) -> Self {
        let streams = (0..n_streams)
            .map(|i| Mulberry32::new(base_seed.wrapping_add(i as u32 * 0x9e3779b9)))
            .collect();
        Self { streams, base_seed }
    }

    /// Borrow stream `i` for use in a stochastic call.
    pub fn stream(&mut self, i: usize) -> &mut Mulberry32 {
        &mut self.streams[i]
    }

    /// Reset all streams to their initial seeds.  Call before each replication.
    pub fn reset(&mut self) {
        for (i, s) in self.streams.iter_mut().enumerate() {
            *s = Mulberry32::new(self.base_seed.wrapping_add(i as u32 * 0x9e3779b9));
        }
    }

    /// Reset a single stream (e.g., after a scenario diverges from CRN).
    pub fn reset_stream(&mut self, i: usize) {
        self.streams[i] = Mulberry32::new(
            self.base_seed.wrapping_add(i as u32 * 0x9e3779b9),
        );
    }

    pub fn n_streams(&self) -> usize {
        self.streams.len()
    }
}

// ─── Antithetic variates ──────────────────────────────────────────────────────

/// Draw a uniform pair (U, 1-U) for antithetic variates.
/// The negative correlation between the pair halves variance of the average
/// when the estimator is monotone in U.
pub fn antithetic_pair(rng: &mut Mulberry32) -> (f32, f32) {
    let u = rng.next_f32();
    (u, 1.0 - u)
}

/// Evaluate `f` twice with antithetic variates and return the average.
/// `f` must be a monotone function of its Uniform(0,1) argument.
pub fn antithetic_mean<F: Fn(f32) -> f32>(f: F, rng: &mut Mulberry32) -> f32 {
    let (u, u_anti) = antithetic_pair(rng);
    (f(u) + f(u_anti)) * 0.5
}

// ─── Replicated Monte Carlo runner ───────────────────────────────────────────

/// Run `n_reps` independent replications of `sim` and return (mean, std_error).
///
/// Each replication receives a fresh Mulberry32 seeded as `base_seed + rep`,
/// guaranteeing reproducibility and independence.
///
/// The standard error = σ_Y / √n is the 1-σ margin for the estimated mean.
pub fn monte_carlo_run<F>(n_reps: u32, base_seed: u32, sim: F) -> (f32, f32)
where
    F: Fn(&mut Mulberry32) -> f32,
{
    let mut sum    = 0.0f32;
    let mut sum_sq = 0.0f32;

    for rep in 0..n_reps {
        let mut rng = Mulberry32::new(base_seed.wrapping_add(rep));
        let y = sim(&mut rng);
        sum    += y;
        sum_sq += y * y;
    }

    let n      = n_reps as f32;
    let mean   = sum / n;
    let var    = (sum_sq / n - mean * mean).max(0.0);
    let stderr = (var / n).sqrt();
    (mean, stderr)
}
