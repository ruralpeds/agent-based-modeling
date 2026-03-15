/// P1-15: NHPP thinning produces the correct time-average arrival rate.
/// Tests with a constant λ (all hourly/day factors = 1) so the process
/// degenerates to a homogeneous Poisson with mean inter-arrival = 1/λ.
/// Also verifies the ED default profile produces rates within expected bounds.
#[cfg(test)]
mod tests {
    use crate::rng::Mulberry32;
    use crate::stochastic::poisson::NhppArrivalProcess;

    // ── Homogeneous degenerate case ───────────────────────────────────────────

    #[test]
    fn constant_rate_mean_inter_arrival() {
        let base_rate = 4.0f32; // patients per hour
        let tph = 12u32; // 5-minute ticks
        let nhpp = NhppArrivalProcess::new(base_rate, [1.0f32; 24], [1.0f32; 7], 1.0);

        let mut rng = Mulberry32::new(42);
        let n = 5_000usize;
        let mut t = 0u64;
        let mut deltas: Vec<u64> = Vec::with_capacity(n);

        for _ in 0..n {
            let d = nhpp.next_arrival_delta(t, tph, &mut rng);
            deltas.push(d);
            t += d;
        }

        // The discrete-time mean inter-arrival accounts for ceil() discretization.
        // For Exp(λ_per_tick), E[ceil(X)] = 1 / (1 - exp(-λ_per_tick)).
        let lambda_per_tick = base_rate / tph as f32;
        let mean_delta = deltas.iter().sum::<u64>() as f32 / n as f32;
        let expected = 1.0 / (1.0 - (-lambda_per_tick).exp()); // ~3.53 for λ=1/3
        let rel_err = (mean_delta - expected).abs() / expected;
        assert!(
            rel_err < 0.05,
            "Homogeneous Poisson: mean inter-arrival {mean_delta:.3} ticks, expected {expected:.3}"
        );
    }

    // ── rate_at correctness ───────────────────────────────────────────────────

    #[test]
    fn rate_at_reflects_hourly_factor() {
        let nhpp = NhppArrivalProcess::ed_default();
        let tph = 12u32; // 5-minute ticks
                         // Hour 15 (15:00 afternoon peak) — factor = 1.35
                         // Hour 3  (03:00 trough)         — factor = 0.25
        let r15 = nhpp.rate_at(15 * tph as u64, tph);
        let r3 = nhpp.rate_at(3 * tph as u64, tph);
        assert!(
            r15 > r3,
            "Afternoon rate {r15:.3} should exceed overnight rate {r3:.3}"
        );
        assert!(r15 > 0.0);
        assert!(r3 > 0.0);
    }

    #[test]
    fn rate_at_day_of_week_effect() {
        let nhpp = NhppArrivalProcess::ed_default();
        let tph = 12u32;
        // Monday noon (tick 12*12 = 144) vs Saturday noon (tick 5*24*12 + 12*12 = 1584)
        let r_mon = nhpp.rate_at(12 * tph as u64, tph);
        let r_sat = nhpp.rate_at((5 * 24 + 12) * tph as u64, tph);
        // Monday factor 1.10 > Saturday factor 0.85 (both at noon)
        assert!(
            r_mon > r_sat,
            "Monday noon {r_mon:.3} should exceed Saturday noon {r_sat:.3}"
        );
    }

    // ── All deltas are positive ───────────────────────────────────────────────

    #[test]
    fn next_arrival_delta_always_positive() {
        let nhpp = NhppArrivalProcess::ed_default();
        let mut rng = Mulberry32::new(7);
        let tph = 12u32;
        let mut t = 0u64;
        for _ in 0..500 {
            let d = nhpp.next_arrival_delta(t, tph, &mut rng);
            assert!(d >= 1, "delta must be >= 1 tick, got {d}");
            t += d;
        }
    }

    // ── ED default arrives at plausible rate ──────────────────────────────────

    #[test]
    fn ed_default_peak_rate_in_range() {
        let nhpp = NhppArrivalProcess::ed_default();
        let tph = 12u32;
        // At peak (hour 14, Monday), rate should be approximately 3.2 * 1.30 * 1.10 ≈ 4.57
        let r = nhpp.rate_at(14 * tph as u64, tph);
        assert!(
            r > 3.0 && r < 6.0,
            "Peak ED rate {r:.3} outside plausible range [3, 6] patients/hr"
        );
    }
}
