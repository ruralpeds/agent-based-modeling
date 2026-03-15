/// P1-13: Survival distribution mean and variance within 5% of theoretical.
/// Tests Exponential, Weibull, and LogNormal samplers against their analytic moments.
/// Also verifies cox_hazard_ratio produces the expected ordering.
#[cfg(test)]
mod tests {
    use crate::rng::Mulberry32;
    use crate::stochastic::survival::{
        SurvivalDistribution, CoxCovariates, CoxParams, cox_hazard_ratio,
    };

    fn moments(dist: &SurvivalDistribution, n: usize, seed: u32) -> (f32, f32) {
        let mut rng = Mulberry32::new(seed);
        let samples: Vec<f32> = (0..n).map(|_| dist.sample(&mut rng)).collect();
        let mean = samples.iter().sum::<f32>() / n as f32;
        let var  = samples.iter().map(|&x| (x - mean) * (x - mean)).sum::<f32>() / n as f32;
        (mean, var)
    }

    // ── Exponential ───────────────────────────────────────────────────────────

    #[test]
    fn exponential_mean_within_5pct() {
        let dist = SurvivalDistribution::Exponential { lambda: 0.5 };
        let (mean, _) = moments(&dist, 50_000, 42);
        let expected  = dist.mean();
        let rel_err   = (mean - expected).abs() / expected;
        assert!(rel_err < 0.05, "Exp mean {mean:.3} vs theoretical {expected:.3}");
    }

    // ── Weibull ───────────────────────────────────────────────────────────────

    #[test]
    fn weibull_icu_los_mean_within_5pct() {
        let dist = SurvivalDistribution::icu_los(); // Weibull(0.85, 120.0)
        let (mean, var) = moments(&dist, 50_000, 7);
        let exp_mean = dist.mean();
        let exp_var  = dist.variance();

        let mean_err = (mean - exp_mean).abs() / exp_mean;
        let var_err  = (var  - exp_var ).abs() / exp_var;

        assert!(mean_err < 0.05, "Weibull mean {mean:.2} vs theoretical {exp_mean:.2}");
        assert!(var_err  < 0.10, "Weibull var  {var:.2}  vs theoretical {exp_var:.2}");
    }

    #[test]
    fn weibull_samples_positive() {
        let dist = SurvivalDistribution::Weibull { shape: 0.7, scale: 50.0 };
        let mut rng = Mulberry32::new(99);
        for _ in 0..1000 {
            let s = dist.sample(&mut rng);
            assert!(s > 0.0, "Weibull produced non-positive sample: {s}");
        }
    }

    // ── LogNormal ─────────────────────────────────────────────────────────────

    #[test]
    fn lognormal_ward_los_mean_within_5pct() {
        let dist = SurvivalDistribution::ward_los(); // LN(2.8, 0.9)
        let (mean, _) = moments(&dist, 50_000, 13);
        let exp_mean  = dist.mean();
        let rel_err   = (mean - exp_mean).abs() / exp_mean;
        assert!(rel_err < 0.05, "LogNormal mean {mean:.3} vs theoretical {exp_mean:.3}");
    }

    #[test]
    fn lognormal_ed_los_mean_within_5pct() {
        let dist = SurvivalDistribution::ed_los_esi3();
        let (mean, _) = moments(&dist, 50_000, 31);
        let exp_mean  = dist.mean();
        let rel_err   = (mean - exp_mean).abs() / exp_mean;
        assert!(rel_err < 0.05, "ED LOS mean {mean:.3} vs theoretical {exp_mean:.3}");
    }

    // ── Cox hazard ratio ──────────────────────────────────────────────────────

    #[test]
    fn cox_hr_increases_with_comorbidity() {
        let params = CoxParams::default();
        let low_risk = CoxCovariates {
            age: 45.0, charlson_index: 0.0,
            admission_severity: 0.0, ventilated: false, central_line: false,
        };
        let high_risk = CoxCovariates {
            age: 80.0, charlson_index: 6.0,
            admission_severity: 12.0, ventilated: true, central_line: true,
        };
        let hr_low  = cox_hazard_ratio(&low_risk,  &params);
        let hr_high = cox_hazard_ratio(&high_risk, &params);
        assert!(hr_high > hr_low, "High-risk HR {hr_high:.3} should exceed low-risk HR {hr_low:.3}");
        assert!(hr_low  > 0.0, "HR must be positive");
    }

    #[test]
    fn cox_adjusted_sample_shorter_for_high_hr() {
        // High HR patients should have shorter expected LOS on average.
        let dist   = SurvivalDistribution::icu_los();
        let params = CoxParams::default();
        let low_cov  = CoxCovariates { age: 45.0, ..CoxCovariates::default() };
        let high_cov = CoxCovariates {
            age: 80.0, charlson_index: 8.0, admission_severity: 15.0,
            ventilated: true, central_line: true,
        };
        let hr_low  = cox_hazard_ratio(&low_cov,  &params);
        let hr_high = cox_hazard_ratio(&high_cov, &params);

        let n = 10_000usize;
        let mut rng = Mulberry32::new(5);
        let mean_low:  f32 = (0..n).map(|_| dist.sample_adjusted(hr_low,  &mut rng)).sum::<f32>() / n as f32;
        let mut rng2 = Mulberry32::new(5);
        let mean_high: f32 = (0..n).map(|_| dist.sample_adjusted(hr_high, &mut rng2)).sum::<f32>() / n as f32;

        assert!(mean_high < mean_low, "High-risk mean LOS {mean_high:.2} should be < low-risk {mean_low:.2}");
    }
}
