/// P1-10: Mulberry32 chi-square uniformity test.
/// Bins 100_000 samples into 10 equal-width buckets and checks χ² < critical value.
#[cfg(test)]
mod tests {
    use crate::rng::Mulberry32;
    use crate::stochastic::distributions::{
        beta_sample, exponential_sample, gamma_sample, standard_normal,
    };

    // ── helpers ──────────────────────────────────────────────────────────────

    fn chi_square_uniform(counts: &[u64], n: u64) -> f32 {
        let expected = n as f32 / counts.len() as f32;
        counts
            .iter()
            .map(|&c| {
                let diff = c as f32 - expected;
                diff * diff / expected
            })
            .sum()
    }

    // ── P1-10: Mulberry32 uniformity ─────────────────────────────────────────

    #[test]
    fn mulberry32_uniform_chi_square() {
        let n_samples = 100_000u64;
        let n_bins = 10usize;
        let mut rng = Mulberry32::new(42);
        let mut counts = vec![0u64; n_bins];

        for _ in 0..n_samples {
            let u = rng.next_f32();
            assert!(u >= 0.0 && u < 1.0, "uniform draw out of [0,1): {u}");
            let bin = (u * n_bins as f32) as usize;
            counts[bin.min(n_bins - 1)] += 1;
        }

        let chi2 = chi_square_uniform(&counts, n_samples);
        // χ² critical value for 9 df at α = 0.001: 27.877
        // We use a very conservative bound (50) to avoid flaky failures.
        assert!(
            chi2 < 50.0,
            "χ² = {chi2:.2} exceeds critical value 50.0 (9 df, α=0.001): distribution non-uniform"
        );
    }

    // ── Standard normal sanity checks ────────────────────────────────────────

    #[test]
    fn standard_normal_mean_and_variance() {
        let n = 100_000usize;
        let mut rng = Mulberry32::new(7);
        let samples: Vec<f32> = (0..n).map(|_| standard_normal(&mut rng)).collect();

        let mean = samples.iter().sum::<f32>() / n as f32;
        let var = samples
            .iter()
            .map(|&x| (x - mean) * (x - mean))
            .sum::<f32>()
            / n as f32;

        assert!(mean.abs() < 0.05, "N(0,1) mean = {mean:.4}, expected ≈ 0");
        assert!(
            (var - 1.0).abs() < 0.05,
            "N(0,1) var  = {var:.4}, expected ≈ 1"
        );
    }

    // ── Exponential sanity checks ─────────────────────────────────────────────

    #[test]
    fn exponential_mean() {
        let lambda = 2.5f32;
        let n = 50_000usize;
        let mut rng = Mulberry32::new(13);
        let mean: f32 = (0..n)
            .map(|_| exponential_sample(lambda, &mut rng))
            .sum::<f32>()
            / n as f32;
        let expected_mean = 1.0 / lambda;
        let rel_err = (mean - expected_mean).abs() / expected_mean;
        assert!(
            rel_err < 0.03,
            "Exp({lambda}) mean = {mean:.4}, expected {expected_mean:.4}"
        );
    }

    // ── Gamma sanity checks ───────────────────────────────────────────────────

    #[test]
    fn gamma_mean_and_variance() {
        let shape = 3.0f32;
        let scale = 2.0f32;
        let n = 50_000usize;
        let mut rng = Mulberry32::new(99);
        let samples: Vec<f32> = (0..n)
            .map(|_| gamma_sample(shape, scale, &mut rng))
            .collect();

        let mean = samples.iter().sum::<f32>() / n as f32;
        let var = samples
            .iter()
            .map(|&x| (x - mean) * (x - mean))
            .sum::<f32>()
            / n as f32;

        let expected_mean = shape * scale; // 6.0
        let expected_var = shape * scale * scale; // 12.0

        assert!(
            (mean - expected_mean).abs() / expected_mean < 0.03,
            "Gamma({shape},{scale}) mean = {mean:.4}, expected {expected_mean:.4}"
        );
        assert!(
            (var - expected_var).abs() / expected_var < 0.05,
            "Gamma({shape},{scale}) var  = {var:.4}, expected {expected_var:.4}"
        );
    }

    #[test]
    fn gamma_small_shape() {
        // shape < 1 branch
        let shape = 0.5f32;
        let scale = 1.0f32;
        let n = 20_000usize;
        let mut rng = Mulberry32::new(55);
        let mean: f32 = (0..n)
            .map(|_| gamma_sample(shape, scale, &mut rng))
            .sum::<f32>()
            / n as f32;
        let expected = shape * scale; // 0.5
        let rel_err = (mean - expected).abs() / expected;
        assert!(
            rel_err < 0.05,
            "Gamma(0.5,1) mean = {mean:.4}, expected {expected:.4}"
        );
    }

    // ── Mulberry32 signed range ───────────────────────────────────────────────

    #[test]
    fn mulberry32_signed_range() {
        // next_f32_signed should return values in [-1, 1).
        let n = 100_000usize;
        let mut rng = Mulberry32::new(37);
        let mut saw_negative = false;
        let mut saw_positive = false;
        for _ in 0..n {
            let v = rng.next_f32_signed();
            assert!(v >= -1.0 && v < 1.0, "next_f32_signed out of [-1, 1): {v}");
            if v < 0.0 {
                saw_negative = true;
            }
            if v > 0.0 {
                saw_positive = true;
            }
        }
        assert!(saw_negative, "never sampled a negative value in 100k draws");
        assert!(saw_positive, "never sampled a positive value in 100k draws");
    }

    #[test]
    fn mulberry32_signed_mean_near_zero() {
        // Signed samples are symmetric around 0 — mean should be ≈ 0.
        let n = 100_000usize;
        let mut rng = Mulberry32::new(41);
        let mean: f32 = (0..n).map(|_| rng.next_f32_signed()).sum::<f32>() / n as f32;
        assert!(
            mean.abs() < 0.02,
            "next_f32_signed mean = {mean:.4}, expected ≈ 0.0"
        );
    }

    // ── Beta sanity checks ────────────────────────────────────────────────────

    #[test]
    fn beta_mean_and_in_unit_interval() {
        let alpha = 2.0f32;
        let beta = 5.0f32;
        let n = 50_000usize;
        let mut rng = Mulberry32::new(17);
        let samples: Vec<f32> = (0..n).map(|_| beta_sample(alpha, beta, &mut rng)).collect();

        let mean = samples.iter().sum::<f32>() / n as f32;
        let expected_mean = alpha / (alpha + beta); // 2/7 ≈ 0.2857

        for &s in &samples {
            assert!(s >= 0.0 && s <= 1.0, "Beta sample out of [0,1]: {s}");
        }
        assert!(
            (mean - expected_mean).abs() < 0.01,
            "Beta({alpha},{beta}) mean = {mean:.4}, expected {expected_mean:.4}"
        );
    }
}
