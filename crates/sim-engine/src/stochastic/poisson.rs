/// Non-Homogeneous Poisson Process (NHPP) with Lewis-Shedler thinning.
/// Calibrated to NHAMCS emergency department arrival patterns.
use crate::rng::Mulberry32;
use crate::stochastic::distributions::exponential_sample;

// ─── NHPP arrival process ─────────────────────────────────────────────────────

/// NHPP with multiplicative time-of-day, day-of-week, and seasonal factors.
pub struct NhppArrivalProcess {
    /// Baseline arrival rate in patients per hour (at factor = 1.0).
    pub base_rate_per_hour: f32,
    /// Multiplicative factors for hours 0–23 (midnight = 0).
    pub hourly_factors: [f32; 24],
    /// Multiplicative factors for days 0–6 (0 = Monday).
    pub day_factors: [f32; 7],
    /// Seasonal multiplicative factor (e.g. 1.25 for winter influenza peak).
    pub season_factor: f32,
    /// Supremum of λ(t) used in thinning acceptance step (computed at construction).
    lambda_max: f32,
}

impl NhppArrivalProcess {
    /// Construct from physical rate and calibration factors.
    /// `lambda_max` is computed automatically.
    pub fn new(
        base_rate_per_hour: f32,
        hourly_factors: [f32; 24],
        day_factors: [f32; 7],
        season_factor: f32,
    ) -> Self {
        let max_h = hourly_factors.iter().cloned().fold(0.0_f32, f32::max);
        let max_d = day_factors.iter().cloned().fold(0.0_f32, f32::max);
        let lambda_max = (base_rate_per_hour * max_h * max_d * season_factor).max(1e-6);
        Self {
            base_rate_per_hour,
            hourly_factors,
            day_factors,
            season_factor,
            lambda_max,
        }
    }

    /// Emergency department default profile calibrated to NHAMCS data.
    /// Base rate ≈ 3.2 patients/hour; afternoon peak × 1.35.
    pub fn ed_default() -> Self {
        let hourly: [f32; 24] = [
            0.35, 0.30, 0.28, 0.25, 0.25, 0.30, // 00–05  overnight trough
            0.70, 0.85, 0.95, 1.05, 1.15, 1.20, // 06–11  morning ramp
            1.25, 1.30, 1.35, 1.35, 1.30, 1.25, // 12–17  afternoon peak
            1.15, 1.05, 0.95, 0.85, 0.70, 0.50, // 18–23  evening decline
        ];
        let day: [f32; 7] = [1.10, 1.05, 1.00, 0.98, 0.95, 0.85, 0.90]; // Mon–Sun
        Self::new(3.2, hourly, day, 1.0)
    }

    /// Instantaneous arrival rate (patients/hour) at simulation tick `tick`.
    /// `ticks_per_hour` converts tick index to time-of-day and day-of-week.
    pub fn rate_at(&self, tick: u64, ticks_per_hour: u32) -> f32 {
        let tph = ticks_per_hour as u64;
        let hour_of_day  = (tick / tph % 24) as usize;
        let day_of_week  = (tick / (tph * 24) % 7) as usize;
        self.base_rate_per_hour
            * self.hourly_factors[hour_of_day]
            * self.day_factors[day_of_week]
            * self.season_factor
    }

    /// Lewis-Shedler thinning: returns the number of ticks until the next
    /// accepted arrival event (always ≥ 1).
    ///
    /// Algorithm:
    ///   1. Draw candidate gap from Poisson at λ_max (homogeneous).
    ///   2. Accept with probability λ(t_candidate) / λ_max.
    ///   3. Repeat until accepted.
    pub fn next_arrival_delta(
        &self,
        current_tick:   u64,
        ticks_per_hour: u32,
        rng:            &mut Mulberry32,
    ) -> u64 {
        let lambda_max_per_tick = self.lambda_max / ticks_per_hour as f32;
        let mut t = current_tick;
        loop {
            // Inter-arrival from homogeneous Poisson(λ_max) — converted to ticks.
            let frac = exponential_sample(lambda_max_per_tick, rng);
            let delta = (frac.ceil() as u64).max(1);
            t += delta;
            let lambda_t = self.rate_at(t, ticks_per_hour) / ticks_per_hour as f32;
            // Accept with probability λ(t) / λ_max.
            if rng.next_f32() < lambda_t / lambda_max_per_tick {
                return t - current_tick;
            }
        }
    }
}
