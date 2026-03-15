#[cfg(test)]
mod tests {
    use crate::engine::{Agent, AgentType};
    use crate::statistics::stats::{compute_stats, SimStats};

    // ── helpers ──────────────────────────────────────────────────────────────

    fn prey(energy: f32) -> Agent {
        Agent::new_prey(0, 0.0, 0.0, energy)
    }

    fn pred(energy: f32) -> Agent {
        Agent::new_predator(1, 1.0, 1.0, energy)
    }

    fn prey_with_action(id: u32, energy: f32, action: crate::engine::agent::Action) -> Agent {
        let mut a = Agent::new_prey(id, 0.0, 0.0, energy);
        a.action = action;
        a
    }

    // ── compute_stats ────────────────────────────────────────────────────────

    #[test]
    fn stats_empty_agents() {
        let s = compute_stats(&[], 42);
        assert_eq!(s.tick, 42);
        assert_eq!(s.prey_count, 0);
        assert_eq!(s.pred_count, 0);
        assert!((s.mean_prey_energy - 0.0).abs() < 1e-5);
        assert!((s.mean_pred_energy - 0.0).abs() < 1e-5);
        assert!((s.gini_energy - 0.0).abs() < 1e-5);
        assert_eq!(s.action_dist, [0.0f32; 7]);
    }

    #[test]
    fn stats_counts_correct() {
        let agents = vec![prey(100.0), prey(50.0), pred(80.0)];
        let s = compute_stats(&agents, 10);
        assert_eq!(s.prey_count, 2);
        assert_eq!(s.pred_count, 1);
    }

    #[test]
    fn stats_tick_propagated() {
        let agents = vec![prey(100.0)];
        let s = compute_stats(&agents, 9999);
        assert_eq!(s.tick, 9999);
    }

    // ── mean_energy ──────────────────────────────────────────────────────────

    #[test]
    fn mean_prey_energy_single_agent() {
        let agents = vec![prey(75.0)];
        let s = compute_stats(&agents, 0);
        assert!((s.mean_prey_energy - 75.0).abs() < 1e-4);
    }

    #[test]
    fn mean_prey_energy_multiple_agents() {
        // mean of [100, 50, 150] = 300/3 = 100
        let agents = vec![prey(100.0), prey(50.0), prey(150.0)];
        let s = compute_stats(&agents, 0);
        assert!(
            (s.mean_prey_energy - 100.0).abs() < 1e-4,
            "mean_prey_energy = {}, expected 100.0",
            s.mean_prey_energy
        );
    }

    #[test]
    fn mean_pred_energy_zero_when_no_predators() {
        let agents = vec![prey(80.0), prey(120.0)];
        let s = compute_stats(&agents, 0);
        assert!((s.mean_pred_energy - 0.0).abs() < 1e-5);
    }

    #[test]
    fn mean_energy_uses_correct_type_partition() {
        // prey energy = 100, pred energy = 50 — must not be mixed
        let agents = vec![prey(100.0), pred(50.0)];
        let s = compute_stats(&agents, 0);
        assert!((s.mean_prey_energy - 100.0).abs() < 1e-4);
        assert!((s.mean_pred_energy - 50.0).abs() < 1e-4);
    }

    // ── gini coefficient ─────────────────────────────────────────────────────

    #[test]
    fn gini_equal_energy_is_zero() {
        // All same energy → perfect equality → gini = 0
        let agents = vec![prey(100.0), prey(100.0), prey(100.0)];
        let s = compute_stats(&agents, 0);
        assert!(
            (s.gini_energy - 0.0).abs() < 1e-5,
            "gini = {}, expected 0.0 for equal energies",
            s.gini_energy
        );
    }

    #[test]
    fn gini_two_equal_agents_is_zero() {
        let agents = vec![prey(50.0), prey(50.0)];
        let s = compute_stats(&agents, 0);
        assert!((s.gini_energy - 0.0).abs() < 1e-5);
    }

    #[test]
    fn gini_two_agents_one_zero_is_half() {
        // Reference: n=2, sorted=[0,100]
        // sum = (2*2 - 2 - 1)*100 = 100; total=100; gini = 100/(2*100) = 0.5
        let agents = vec![prey(0.0), prey(100.0)];
        let s = compute_stats(&agents, 0);
        assert!(
            (s.gini_energy - 0.5).abs() < 1e-5,
            "gini = {}, expected 0.5 for [0, 100]",
            s.gini_energy
        );
    }

    #[test]
    fn gini_three_agents_one_has_all_energy() {
        // Reference: n=3, sorted=[0,0,100]
        // sum = (-2)*0 + 0*0 + 2*100 = 200; total=100; gini = 200/300 ≈ 0.6667
        let agents = vec![prey(0.0), prey(0.0), prey(100.0)];
        let s = compute_stats(&agents, 0);
        let expected = 200.0f32 / 300.0f32;
        assert!(
            (s.gini_energy - expected).abs() < 1e-4,
            "gini = {}, expected {}",
            s.gini_energy,
            expected
        );
    }

    #[test]
    fn gini_is_in_zero_to_one_range() {
        let agents = vec![prey(10.0), prey(200.0), pred(0.0), pred(150.0)];
        let s = compute_stats(&agents, 0);
        assert!(
            s.gini_energy >= 0.0 && s.gini_energy <= 1.0,
            "gini = {} out of [0,1]",
            s.gini_energy
        );
    }

    #[test]
    fn gini_empty_agents_is_zero() {
        let s = compute_stats(&[], 0);
        assert!((s.gini_energy - 0.0).abs() < 1e-5);
    }

    #[test]
    fn gini_monotone_more_unequal_produces_higher_gini() {
        // Equal distribution → gini = 0
        let equal = vec![prey(50.0), prey(50.0), prey(50.0), prey(50.0)];
        let s_eq = compute_stats(&equal, 0);
        // Moderate inequality
        let moderate = vec![prey(25.0), prey(50.0), prey(75.0), prey(100.0)];
        let s_mod = compute_stats(&moderate, 0);
        // Maximal inequality: one agent has everything
        let unequal = vec![prey(0.0), prey(0.0), prey(0.0), prey(200.0)];
        let s_uneq = compute_stats(&unequal, 0);

        assert!(
            s_eq.gini_energy < s_mod.gini_energy,
            "equal ({:.4}) should have lower gini than moderate ({:.4})",
            s_eq.gini_energy,
            s_mod.gini_energy
        );
        assert!(
            s_mod.gini_energy < s_uneq.gini_energy,
            "moderate ({:.4}) should have lower gini than maximal ({:.4})",
            s_mod.gini_energy,
            s_uneq.gini_energy
        );
    }

    #[test]
    fn gini_single_agent_is_zero() {
        let agents = vec![prey(150.0)];
        let s = compute_stats(&agents, 0);
        assert!(
            (s.gini_energy - 0.0).abs() < 1e-5,
            "gini for single agent = {}",
            s.gini_energy
        );
    }

    // ── action_distribution ───────────────────────────────────────────────────

    #[test]
    fn action_dist_sums_to_one_nonempty() {
        use crate::engine::agent::Action;
        let agents = vec![
            prey_with_action(0, 100.0, Action::Wander),
            prey_with_action(1, 80.0, Action::Forage),
            prey_with_action(2, 60.0, Action::Flee),
        ];
        let s = compute_stats(&agents, 0);
        let total: f32 = s.action_dist.iter().sum();
        assert!(
            (total - 1.0).abs() < 1e-5,
            "action_dist sum = {}, expected 1.0",
            total
        );
    }

    #[test]
    fn action_dist_all_wander() {
        use crate::engine::agent::Action;
        let agents = vec![
            prey_with_action(0, 100.0, Action::Wander),
            prey_with_action(1, 100.0, Action::Wander),
        ];
        let s = compute_stats(&agents, 0);
        // Wander is index 0; all others must be 0.0
        assert!((s.action_dist[0] - 1.0).abs() < 1e-5);
        for i in 1..7 {
            assert!(
                (s.action_dist[i] - 0.0).abs() < 1e-5,
                "action_dist[{}] = {} expected 0.0",
                i,
                s.action_dist[i]
            );
        }
    }

    #[test]
    fn action_dist_empty_agents_all_zero() {
        let s = compute_stats(&[], 0);
        assert_eq!(s.action_dist, [0.0f32; 7]);
    }

    #[test]
    fn action_dist_even_split_two_actions() {
        use crate::engine::agent::Action;
        // 2 wander (0), 2 forage (1)
        let agents = vec![
            prey_with_action(0, 100.0, Action::Wander),
            prey_with_action(1, 100.0, Action::Wander),
            prey_with_action(2, 80.0, Action::Forage),
            prey_with_action(3, 80.0, Action::Forage),
        ];
        let s = compute_stats(&agents, 0);
        assert!((s.action_dist[0] - 0.5).abs() < 1e-5); // Wander
        assert!((s.action_dist[1] - 0.5).abs() < 1e-5); // Forage
        for i in 2..7 {
            assert!((s.action_dist[i] - 0.0).abs() < 1e-5);
        }
    }

    #[test]
    fn action_dist_array_length_is_seven() {
        let s = compute_stats(&[prey(100.0)], 0);
        assert_eq!(s.action_dist.len(), 7);
    }
}
