/// P1-12: CTMC Gillespie holding-time K-S test.
/// Samples 5_000 holding times from a 2-state CTMC with rate λ=2.0 and
/// tests that the empirical CDF is within the Kolmogorov-Smirnov critical
/// value for Exp(λ=2.0).
#[cfg(test)]
mod tests {
    use crate::rng::Mulberry32;
    use crate::stochastic::ctmc::{GillespieSampler, CtmcTransition, colonization_ctmc};

    // ── KS test against Exp(lambda) ───────────────────────────────────────────

    /// One-sample Kolmogorov-Smirnov statistic against Exp(lambda) CDF.
    /// Returns D = max_t |F_empirical(t) - F_theoretical(t)|.
    fn ks_stat_vs_exp(mut samples: Vec<f32>, lambda: f32) -> f32 {
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = samples.len() as f32;
        let mut d_max = 0.0f32;
        for (i, &t) in samples.iter().enumerate() {
            let f_emp_hi = (i + 1) as f32 / n;
            let f_emp_lo = i as f32 / n;
            let f_theo   = 1.0 - (-lambda * t).exp();
            d_max = d_max.max((f_emp_hi - f_theo).abs());
            d_max = d_max.max((f_emp_lo - f_theo).abs());
        }
        d_max
    }

    // ── P1-12: holding time distribution ─────────────────────────────────────

    #[test]
    fn ctmc_holding_time_ks_test() {
        let lambda = 2.0f32;
        // State 0 → State 1 with rate lambda; absorb at state 1.
        let sampler = GillespieSampler::new(vec![
            CtmcTransition { from: 0, to: 1, rate: lambda },
        ]);
        let mut rng = Mulberry32::new(42);
        let n = 5_000usize;
        let times: Vec<f32> = (0..n)
            .filter_map(|_| sampler.next_event(0, &mut rng).map(|(t, _)| t))
            .collect();

        assert_eq!(times.len(), n, "All events from state 0 must fire");

        let ks = ks_stat_vs_exp(times, lambda);
        // KS critical value at α=0.05: 1.36/√n ≈ 0.0192 for n=5000.
        // Use 0.03 as a safe bound accounting for f32 discretisation.
        assert!(
            ks < 0.03,
            "KS statistic {ks:.4} exceeds 0.03 — holding times not Exp({lambda})"
        );
    }

    #[test]
    fn absorbing_state_returns_none() {
        let sampler = GillespieSampler::new(vec![]); // no transitions
        let mut rng = Mulberry32::new(1);
        assert!(sampler.next_event(0, &mut rng).is_none());
    }

    #[test]
    fn total_rate_from_correct() {
        let sampler = GillespieSampler::new(vec![
            CtmcTransition { from: 0, to: 1, rate: 1.5 },
            CtmcTransition { from: 0, to: 2, rate: 2.5 },
            CtmcTransition { from: 1, to: 0, rate: 0.8 },
        ]);
        let rel_err = (sampler.total_rate_from(0) - 4.0).abs();
        assert!(rel_err < 1e-5, "total_rate_from(0) should be 4.0, got {}", sampler.total_rate_from(0));
    }

    #[test]
    fn ctmc_destination_proportional_to_rate() {
        // Two competing transitions: 0→1 at rate 3, 0→2 at rate 1.
        // Expected P(→1) = 0.75, P(→2) = 0.25.
        let sampler = GillespieSampler::new(vec![
            CtmcTransition { from: 0, to: 1, rate: 3.0 },
            CtmcTransition { from: 0, to: 2, rate: 1.0 },
        ]);
        let mut rng = Mulberry32::new(9);
        let n = 20_000usize;
        let mut to_one = 0u32;
        for _ in 0..n {
            if let Some((_, dest)) = sampler.next_event(0, &mut rng) {
                if dest == 1 { to_one += 1; }
            }
        }
        let p_one = to_one as f32 / n as f32;
        assert!(
            (p_one - 0.75).abs() < 0.02,
            "P(→1) = {p_one:.4}, expected 0.75 ± 0.02"
        );
    }

    #[test]
    fn colonization_ctmc_runs_without_panic() {
        let mut ctmc = colonization_ctmc(0.10, 0.05);
        let mut rng  = Mulberry32::new(3);
        let state = ctmc.run_to_time(0, 100.0, &mut rng);
        assert!(state == 0 || state == 1, "unexpected state {state}");
        // Set a custom rate
        ctmc.set_rate(0, 1, 0.20);
        assert!((ctmc.total_rate_from(0) - 0.20).abs() < 1e-6);
    }
}
