#[cfg(test)]
mod tests {
    use crate::engine::sim_engine::{SimEngine, SimParams, GRID_W, GRID_H};
    use crate::decision_rules::RuleSetId;

    fn default_params() -> SimParams {
        SimParams {
            prey_count: 80,
            predator_count: 12,
            prey_metabolism:     0.3,
            predator_metabolism: 0.5,
            prey_reproduce_energy:    80.0,
            predator_reproduce_energy: 100.0,
            ..SimParams::default()
        }
    }

    #[test]
    fn init_creates_correct_counts() {
        let p = SimParams { prey_count: 30, predator_count: 6, ..SimParams::default() };
        let engine = SimEngine::new(p, 42);
        assert_eq!(engine.prey_count(), 30);
        assert_eq!(engine.pred_count(), 6);
    }

    #[test]
    fn step_increments_tick() {
        let mut engine = SimEngine::new(default_params(), 42);
        engine.step();
        assert_eq!(engine.tick(), 1);
        engine.step();
        assert_eq!(engine.tick(), 2);
    }

    #[test]
    fn agent_positions_stay_in_bounds() {
        let mut engine = SimEngine::new(default_params(), 7);
        for _ in 0..100 {
            engine.step();
            for agent in engine.agents() {
                assert!(agent.x >= 0.0 && agent.x < GRID_W, "x={} out of bounds", agent.x);
                assert!(agent.y >= 0.0 && agent.y < GRID_H, "y={} out of bounds", agent.y);
            }
        }
    }

    #[test]
    fn energies_non_negative() {
        let mut engine = SimEngine::new(default_params(), 13);
        for _ in 0..200 {
            engine.step();
            for agent in engine.agents() {
                assert!(agent.energy >= 0.0, "negative energy: {}", agent.energy);
            }
        }
    }

    #[test]
    fn same_seed_deterministic() {
        let p = default_params();
        let mut e1 = SimEngine::new(p.clone(), 99);
        let mut e2 = SimEngine::new(p,          99);
        for _ in 0..30 { e1.step(); e2.step(); }
        assert_eq!(e1.agent_count(), e2.agent_count());
        for (a, b) in e1.agents().iter().zip(e2.agents().iter()) {
            assert!((a.x - b.x).abs() < 1e-5);
            assert!((a.energy - b.energy).abs() < 1e-5);
        }
    }

    #[test]
    fn reactive_survives_500_ticks() {
        let p = SimParams { rule_set_id: RuleSetId::Reactive, ..default_params() };
        let mut engine = SimEngine::new(p, 42);
        for _ in 0..500 { engine.step(); }
        assert!(engine.agent_count() > 0, "total extinction");
    }

    #[test]
    fn bounded_survives_500_ticks() {
        let p = SimParams { rule_set_id: RuleSetId::Bounded, ..default_params() };
        let mut engine = SimEngine::new(p, 42);
        for _ in 0..500 { engine.step(); }
        assert!(engine.agent_count() > 0, "total extinction");
    }

    #[test]
    fn bdi_survives_500_ticks() {
        let p = SimParams { rule_set_id: RuleSetId::Bdi, ..default_params() };
        let mut engine = SimEngine::new(p, 42);
        for _ in 0..500 { engine.step(); }
        assert!(engine.agent_count() > 0, "total extinction");
    }
}
