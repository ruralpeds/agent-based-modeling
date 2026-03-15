/// Core probability distribution samplers.
/// All functions take `&mut Mulberry32` — never std::thread_rng().
use crate::rng::Mulberry32;

// ─── Normal ──────────────────────────────────────────────────────────────────

/// Sample Z ~ N(0, 1) via Box-Muller transform.
/// Consumes two uniform draws; returns the cosine variate (wastes the sine).
pub fn standard_normal(rng: &mut Mulberry32) -> f32 {
    loop {
        let u = rng.next_f32();
        if u > 0.0 {
            let v = rng.next_f32();
            let r = (-2.0 * u.ln()).sqrt();
            return r * (core::f32::consts::TAU * v).cos();
        }
    }
}

// ─── Exponential ─────────────────────────────────────────────────────────────

/// Sample X ~ Exp(lambda) via inverse CDF: X = -ln(U) / lambda.
pub fn exponential_sample(lambda: f32, rng: &mut Mulberry32) -> f32 {
    let u = rng.next_f32().max(1e-7);
    -u.ln() / lambda
}

// ─── Log-Normal ──────────────────────────────────────────────────────────────

/// Sample X ~ LogNormal(mu, sigma): X = exp(mu + sigma * Z), Z ~ N(0,1).
pub fn log_normal_sample(mu: f32, sigma: f32, rng: &mut Mulberry32) -> f32 {
    (mu + sigma * standard_normal(rng)).exp()
}

// ─── Gamma ───────────────────────────────────────────────────────────────────

/// Sample X ~ Gamma(shape, scale) via Marsaglia-Tsang (2000).
/// shape > 0, scale > 0.  Average ~1.5 loop iterations for shape >= 1.
pub fn gamma_sample(shape: f32, scale: f32, rng: &mut Mulberry32) -> f32 {
    if shape < 1.0 {
        // Boost shape to ≥1, correct by U^(1/shape) (Ahrens & Dieter 1982).
        let u = rng.next_f32().max(1e-7);
        return gamma_sample(shape + 1.0, scale, rng) * u.powf(1.0 / shape);
    }
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        let x = standard_normal(rng);
        let v_base = 1.0 + c * x;
        if v_base <= 0.0 {
            continue;
        }
        let v = v_base * v_base * v_base;
        let u = rng.next_f32().max(1e-7);
        let x2 = x * x;
        // Fast squeeze acceptance test (Marsaglia & Tsang eq. 4).
        if u < 1.0 - 0.0331 * x2 * x2 {
            return d * v * scale;
        }
        if u.ln() < 0.5 * x2 + d * (1.0 - v + v.ln()) {
            return d * v * scale;
        }
    }
}

// ─── Beta ────────────────────────────────────────────────────────────────────

/// Sample X ~ Beta(alpha, beta) via ratio-of-gammas.
/// alpha > 0, beta > 0.
pub fn beta_sample(alpha: f32, beta_param: f32, rng: &mut Mulberry32) -> f32 {
    let x = gamma_sample(alpha, 1.0, rng);
    let y = gamma_sample(beta_param, 1.0, rng);
    let s = x + y;
    if s < 1e-10 {
        return 0.5; // degenerate guard
    }
    (x / s).clamp(0.0, 1.0)
}

// ─── Categorical ─────────────────────────────────────────────────────────────

/// Draw index i with probability probs[i].
/// `probs` must be non-negative and sum to 1 (rows not validated for speed).
pub fn categorical_draw(probs: &[f32], rng: &mut Mulberry32) -> usize {
    let u = rng.next_f32();
    let mut cumsum = 0.0f32;
    for (i, &p) in probs.iter().enumerate() {
        cumsum += p;
        if u < cumsum {
            return i;
        }
    }
    probs.len() - 1 // floating-point edge fallback
}

// ─── Gamma function approximation ────────────────────────────────────────────

/// Compute Γ(x) for x > 0 using Stirling's series + recursion for small x.
/// Accurate to ~1e-6 relative error for x in (0, 20], sufficient for f32.
pub fn gamma_fn(x: f32) -> f32 {
    if x < 6.0 {
        // Recurse upward: Γ(x) = Γ(x+1) / x
        return gamma_fn(x + 1.0) / x;
    }
    // Stirling's approximation for x ≥ 6
    let x64 = x as f64;
    let ln_g =
        (x64 - 0.5) * x64.ln() - x64 + 0.5 * core::f64::consts::TAU.ln() + 1.0 / (12.0 * x64)
            - 1.0 / (360.0 * x64 * x64 * x64);
    ln_g.exp() as f32
}
