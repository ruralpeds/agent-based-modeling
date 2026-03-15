use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use sim_engine::engine::sim_engine::{SimEngine, SimParams};
use sim_engine::engine::Agent;
use sim_engine::hospital::{HospitalSimEngine, HospitalParams};
use sim_engine::spatial::spatial_grid::SpatialGrid;
use sim_engine::rng::Mulberry32;

// ── ABM step benchmarks ───────────────────────────────────────────────────────

fn bench_abm_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("abm_step");

    for &agent_count in &[100u32, 300, 500] {
        let params = SimParams {
            prey_count:     (agent_count * 8 / 10) as usize,
            predator_count: (agent_count * 2 / 10) as usize,
            ..SimParams::default()
        };
        let mut engine = SimEngine::new(params, 42);

        group.bench_with_input(
            BenchmarkId::from_parameter(agent_count),
            &agent_count,
            |b, _| b.iter(|| engine.step()),
        );
    }
    group.finish();
}

// ── Hospital sim step benchmarks ──────────────────────────────────────────────

fn bench_hospital_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("hospital_step");

    // Default arrival rate — warm up census first
    {
        let mut engine = HospitalSimEngine::new(HospitalParams::default(), 42);
        for _ in 0..500 { engine.step(); }
        group.bench_function("default_rate", |b| b.iter(|| engine.step()));
    }

    // High arrival rate (30 pt/h) — higher census → more work per tick
    {
        let params = HospitalParams { base_arrival_rate: 30.0, ..HospitalParams::default() };
        let mut engine = HospitalSimEngine::new(params, 7);
        for _ in 0..200 { engine.step(); }
        group.bench_function("high_rate_30", |b| b.iter(|| engine.step()));
    }

    group.finish();
}

// ── SpatialGrid build + query benchmarks ─────────────────────────────────────

fn make_agents(n: usize) -> Vec<Agent> {
    let mut rng = Mulberry32::new(99);
    (0..n)
        .map(|i| {
            let x = (rng.next_f32() * 80.0).clamp(0.0, 79.99);
            let y = (rng.next_f32() * 60.0).clamp(0.0, 59.99);
            Agent::new_prey(i as u32, x, y, 100.0)
        })
        .collect()
}

fn bench_spatial_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("spatial_grid");

    for &n in &[500usize, 1_000, 5_000] {
        let agents = make_agents(n);

        // Build benchmark
        group.bench_with_input(
            BenchmarkId::new("build", n),
            &agents,
            |b, ags| {
                b.iter(|| {
                    let mut g = SpatialGrid::new(80.0, 60.0, 5.0);
                    g.build(ags);
                    g
                })
            },
        );

        // Query benchmark (pre-built grid, fixed probe point)
        let mut grid = SpatialGrid::new(80.0, 60.0, 5.0);
        grid.build(&agents);
        group.bench_with_input(
            BenchmarkId::new("query_radius", n),
            &grid,
            |b, g| b.iter(|| g.query_radius(40.0, 30.0, 5.0)),
        );
    }

    group.finish();
}

criterion_group!(benches, bench_abm_step, bench_hospital_step, bench_spatial_grid);
criterion_main!(benches);
