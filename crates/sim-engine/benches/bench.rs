use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use sim_engine::engine::sim_engine::{SimEngine, SimParams};

fn bench_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("sim_step");

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

criterion_group!(benches, bench_step);
criterion_main!(benches);
