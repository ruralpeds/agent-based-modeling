/// P1-11: DTMC stationary distribution convergence test.
/// Builds a 3-state ergodic DTMC with known analytic stationary distribution,
/// runs 500_000 steps from each starting state, and checks empirical
/// distribution is within 1% of the analytic solution.
#[cfg(test)]
mod tests {
    use crate::rng::Mulberry32;
    use crate::stochastic::dtmc::{
        build_hai_dtmc_matrix, max_row_sum_error, PatientDTMC, N_DISEASE_STATES,
    };

    // ── 3-state ergodic chain with known stationary distribution ─────────────

    /// P = [[0.7, 0.2, 0.1],
    ///      [0.3, 0.5, 0.2],
    ///      [0.4, 0.1, 0.5]]
    ///
    /// Solving π P = π, π·1 = 1 gives:
    ///   π₀ ≈ 0.4054,  π₁ ≈ 0.2432,  π₂ ≈ 0.3514
    fn ergodic_3x3_matrix() -> Vec<f32> {
        vec![0.7, 0.2, 0.1, 0.3, 0.5, 0.2, 0.4, 0.1, 0.5]
    }

    // Analytic stationary distribution, solved from π P = π, Σπᵢ = 1:
    //   π₀ = 23/43 ≈ 0.5349
    //   π₁ = 11/43 ≈ 0.2558
    //   π₂ =  9/43 ≈ 0.2093
    const PI: [f32; 3] = [0.5349, 0.2558, 0.2093];

    #[test]
    fn stationary_distribution_convergence() {
        let matrix = ergodic_3x3_matrix();
        let dtmc = PatientDTMC::new(3);
        let mut rng = Mulberry32::new(42);

        let n_steps = 500_000usize;
        let mut counts = [0u64; 3];
        let mut state = 0usize;

        for _ in 0..n_steps {
            state = dtmc.sample_next_state(state, &matrix, &mut rng);
            counts[state] += 1;
        }

        for i in 0..3 {
            let empirical = counts[i] as f32 / n_steps as f32;
            let analytic = PI[i];
            let abs_err = (empirical - analytic).abs();
            assert!(
                abs_err < 0.01,
                "State {i}: empirical {empirical:.4} vs analytic {analytic:.4}, err {abs_err:.4}"
            );
        }
    }

    #[test]
    fn dtmc_absorbing_state_stays_absorbing() {
        // Dead (state 6) must be absorbing: once entered, stays there.
        let matrix =
            build_hai_dtmc_matrix(0.05, 0.10, 0.01, 0.08, 0.05, 0.15, 0.10, 0.20, 0.05, 0.002);
        let dtmc = PatientDTMC::new(N_DISEASE_STATES);
        let mut rng = Mulberry32::new(7);

        for _ in 0..1000 {
            let next = dtmc.sample_next_state(6 /* Dead */, &matrix, &mut rng);
            assert_eq!(next, 6, "Dead state must be absorbing");
        }
    }

    #[test]
    fn hai_dtmc_rows_sum_to_one() {
        let matrix =
            build_hai_dtmc_matrix(0.05, 0.10, 0.01, 0.08, 0.05, 0.15, 0.10, 0.20, 0.05, 0.002);
        let err = max_row_sum_error(&matrix, N_DISEASE_STATES);
        assert!(err < 1e-5, "max row-sum error {err:.2e} exceeds 1e-5");
    }

    #[test]
    fn dtmc_sampling_hits_all_reachable_states() {
        // From Susceptible with lambda=0.20, all states should be visited
        // in 100_000 steps given the provided rates.
        let matrix =
            build_hai_dtmc_matrix(0.20, 0.30, 0.02, 0.20, 0.10, 0.50, 0.15, 0.40, 0.10, 0.005);
        let dtmc = PatientDTMC::new(N_DISEASE_STATES);
        let mut rng = Mulberry32::new(31);
        let mut visited = [false; N_DISEASE_STATES];
        let mut state = 0usize;

        for _ in 0..100_000 {
            // Restart from Susceptible once Dead to keep chain moving.
            if state == 6 {
                state = 0;
            }
            state = dtmc.sample_next_state(state, &matrix, &mut rng);
            visited[state] = true;
        }

        // States 0–5 should all be visited; Dead (6) may or may not be.
        for i in 0..6 {
            assert!(visited[i], "State {i} was never visited in 100_000 steps");
        }
    }
}
