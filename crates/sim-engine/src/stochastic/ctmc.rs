/// Continuous-Time Markov Chain sampler using the Gillespie direct method.
/// Holding times are Exp(|q_ii|); destination chosen proportional to off-diagonal rates.
use crate::rng::Mulberry32;
use crate::stochastic::distributions::exponential_sample;

// ─── Transition record ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct CtmcTransition {
    pub from: usize,
    pub to: usize,
    pub rate: f32, // q_{ij}, must be ≥ 0
}

// ─── Gillespie sampler ────────────────────────────────────────────────────────

/// Simulates one event from a CTMC at `current_state` using Gillespie's
/// direct method.
pub struct GillespieSampler {
    pub transitions: Vec<CtmcTransition>,
}

impl GillespieSampler {
    pub fn new(transitions: Vec<CtmcTransition>) -> Self {
        Self { transitions }
    }

    /// Returns `Some((time_to_event, next_state))`, or `None` if `current_state`
    /// has no enabled outgoing transitions (absorbing state).
    ///
    /// Algorithm:
    ///   1. Sum enabled rates: R = Σ q_{current → j}
    ///   2. τ ~ Exp(R)
    ///   3. destination chosen with probability q_{current → j} / R
    pub fn next_event(&self, current_state: usize, rng: &mut Mulberry32) -> Option<(f32, usize)> {
        // Collect enabled transitions out of current state.
        // Iterate twice to avoid allocating a Vec in the hot path.
        let total_rate: f32 = self
            .transitions
            .iter()
            .filter(|t| t.from == current_state && t.rate > 0.0)
            .map(|t| t.rate)
            .sum();

        if total_rate <= 0.0 {
            return None; // absorbing state
        }

        let tau = exponential_sample(total_rate, rng);

        // Select destination proportional to rates.
        let u = rng.next_f32() * total_rate;
        let mut cumsum = 0.0f32;
        let mut last_to = current_state; // fallback (should not be reached)
        for t in self
            .transitions
            .iter()
            .filter(|t| t.from == current_state && t.rate > 0.0)
        {
            cumsum += t.rate;
            last_to = t.to;
            if u < cumsum {
                return Some((tau, t.to));
            }
        }
        Some((tau, last_to))
    }

    /// Set (or insert) the rate for a specific (from, to) pair.
    pub fn set_rate(&mut self, from: usize, to: usize, rate: f32) {
        for t in &mut self.transitions {
            if t.from == from && t.to == to {
                t.rate = rate;
                return;
            }
        }
        self.transitions.push(CtmcTransition { from, to, rate });
    }

    /// Total outflow rate |q_ii| for `state`.
    pub fn total_rate_from(&self, state: usize) -> f32 {
        self.transitions
            .iter()
            .filter(|t| t.from == state && t.rate > 0.0)
            .map(|t| t.rate)
            .sum()
    }

    /// Run the CTMC forward for `time_horizon` units, returning the state at
    /// that time.  Useful for embedding a CTMC within a DTMC tick.
    pub fn run_to_time(
        &self,
        start_state: usize,
        time_horizon: f32,
        rng: &mut Mulberry32,
    ) -> usize {
        let mut state = start_state;
        let mut elapsed = 0.0f32;
        loop {
            match self.next_event(state, rng) {
                None => break,
                Some((tau, next)) => {
                    if elapsed + tau > time_horizon {
                        break;
                    }
                    elapsed += tau;
                    state = next;
                }
            }
        }
        state
    }
}

// ─── Convenience constructors ─────────────────────────────────────────────────

/// Two-state colonization/clearance CTMC.
/// State 0 = Susceptible, State 1 = Colonized.
pub fn colonization_ctmc(colonization_rate: f32, clearance_rate: f32) -> GillespieSampler {
    GillespieSampler::new(vec![
        CtmcTransition {
            from: 0,
            to: 1,
            rate: colonization_rate,
        },
        CtmcTransition {
            from: 1,
            to: 0,
            rate: clearance_rate,
        },
    ])
}
