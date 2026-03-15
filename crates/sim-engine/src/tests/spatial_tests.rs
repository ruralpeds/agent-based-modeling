#[cfg(test)]
mod tests {
    use crate::engine::{Agent, AgentType};
    use crate::spatial::SpatialGrid;

    // ── helpers ──────────────────────────────────────────────────────────────

    fn make_grid() -> SpatialGrid {
        // 80×60 world, 8.0 bucket size (matches sim_engine defaults)
        SpatialGrid::new(80.0, 60.0, 8.0)
    }

    fn prey_at(id: u32, x: f32, y: f32) -> Agent {
        Agent::new_prey(id, x, y, 100.0)
    }

    fn pred_at(id: u32, x: f32, y: f32) -> Agent {
        Agent::new_predator(id, x, y, 100.0)
    }

    // ── build / basic structure ───────────────────────────────────────────────

    #[test]
    fn build_empty_agents_no_panic() {
        let mut grid = make_grid();
        grid.build(&[]);
        // Querying on empty grid should return None/empty
        let result = grid.nearest_of_type(40.0, 30.0, 10.0, AgentType::Prey, &[]);
        assert!(result.is_none());
    }

    #[test]
    fn build_single_agent_no_panic() {
        let agents = vec![prey_at(0, 10.0, 10.0)];
        let mut grid = make_grid();
        grid.build(&agents);
    }

    // ── nearest_of_type ──────────────────────────────────────────────────────

    #[test]
    fn nearest_finds_agent_within_radius() {
        let agents = vec![prey_at(0, 10.0, 10.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.nearest_of_type(10.0, 10.0, 5.0, AgentType::Prey, &agents);
        assert!(result.is_some(), "expected to find prey at same position");
        let (x, y, id) = result.unwrap();
        assert!((x - 10.0).abs() < 1e-5);
        assert!((y - 10.0).abs() < 1e-5);
        assert_eq!(id, 0);
    }

    #[test]
    fn nearest_returns_none_when_outside_radius() {
        let agents = vec![prey_at(0, 10.0, 10.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        // Agent is at (10,10), query from (20,20) with radius 5 — too far (dist=~14.1)
        let result = grid.nearest_of_type(20.0, 20.0, 5.0, AgentType::Prey, &agents);
        assert!(result.is_none(), "should not find prey outside radius");
    }

    #[test]
    fn nearest_filters_by_type() {
        // Only a predator at (10,10) — looking for prey should return None
        let agents = vec![pred_at(0, 10.0, 10.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.nearest_of_type(10.0, 10.0, 20.0, AgentType::Prey, &agents);
        assert!(result.is_none(), "predator must not be returned when querying for prey");
    }

    #[test]
    fn nearest_picks_closer_of_two_agents() {
        // Two prey: one close at (5,5), one far at (20,20)
        let agents = vec![prey_at(0, 5.0, 5.0), prey_at(1, 20.0, 20.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.nearest_of_type(4.0, 4.0, 30.0, AgentType::Prey, &agents);
        assert!(result.is_some());
        let (_, _, id) = result.unwrap();
        assert_eq!(id, 0, "expected closer prey (id=0), got id={}", id);
    }

    #[test]
    fn nearest_prey_not_confused_with_predator_at_same_position() {
        // Predator at (10,10), prey at (15,15)
        let agents = vec![pred_at(0, 10.0, 10.0), prey_at(1, 15.0, 15.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let prey_result = grid.nearest_of_type(10.0, 10.0, 30.0, AgentType::Prey, &agents);
        assert!(prey_result.is_some());
        let (_, _, id) = prey_result.unwrap();
        assert_eq!(id, 1, "should return prey id=1, not predator id=0");

        let pred_result = grid.nearest_of_type(10.0, 10.0, 30.0, AgentType::Predator, &agents);
        assert!(pred_result.is_some());
        let (_, _, id) = pred_result.unwrap();
        assert_eq!(id, 0, "should return predator id=0");
    }

    // ── query_radius ─────────────────────────────────────────────────────────

    #[test]
    fn query_radius_empty_grid_returns_empty() {
        let mut grid = make_grid();
        grid.build(&[]);
        let result = grid.query_radius(40.0, 30.0, 10.0, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn query_radius_finds_agent_at_origin() {
        let agents = vec![prey_at(0, 0.0, 0.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.query_radius(0.0, 0.0, 5.0, &agents);
        assert_eq!(result.len(), 1);
        let (x, y, id, t) = result[0];
        assert!((x - 0.0).abs() < 1e-5);
        assert!((y - 0.0).abs() < 1e-5);
        assert_eq!(id, 0);
        assert_eq!(t, AgentType::Prey);
    }

    #[test]
    fn query_radius_excludes_agents_outside_radius() {
        // Agent at (10,0), query from (0,0) with radius 5 — distance is 10 > 5
        let agents = vec![prey_at(0, 10.0, 0.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.query_radius(0.0, 0.0, 5.0, &agents);
        assert!(result.is_empty(), "agent at distance 10 should be outside radius 5");
    }

    #[test]
    fn query_radius_returns_all_agents_within() {
        let agents = vec![
            prey_at(0, 1.0, 0.0),
            prey_at(1, 0.0, 1.0),
            prey_at(2, 10.0, 10.0), // outside radius 3
        ];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.query_radius(0.0, 0.0, 3.0, &agents);
        assert_eq!(result.len(), 2, "expected 2 agents within radius 3, got {}", result.len());
        let ids: Vec<u32> = result.iter().map(|&(_, _, id, _)| id).collect();
        assert!(ids.contains(&0) && ids.contains(&1));
    }

    #[test]
    fn query_radius_returns_mixed_types() {
        let agents = vec![prey_at(0, 5.0, 5.0), pred_at(1, 5.5, 5.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.query_radius(5.0, 5.0, 3.0, &agents);
        assert_eq!(result.len(), 2);
        let types: Vec<AgentType> = result.iter().map(|&(_, _, _, t)| t).collect();
        assert!(types.contains(&AgentType::Prey));
        assert!(types.contains(&AgentType::Predator));
    }

    #[test]
    fn query_radius_agent_exactly_on_boundary_is_included() {
        // dist2(0,0,3,4)=25, radius=5 → dist=5.0 exactly on boundary
        let agents = vec![prey_at(0, 3.0, 4.0)];
        let mut grid = make_grid();
        grid.build(&agents);

        let result = grid.query_radius(0.0, 0.0, 5.0, &agents);
        // dx*dx+dy*dy = 9+16 = 25 ≤ r*r = 25, so included
        assert_eq!(result.len(), 1, "agent on exact boundary should be included (≤ r²)");
    }

    // ── edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn agents_at_world_boundary_do_not_panic() {
        let agents = vec![
            prey_at(0, 0.0, 0.0),
            prey_at(1, 79.9, 59.9),
        ];
        let mut grid = make_grid();
        grid.build(&agents);
        // Should not panic querying near boundaries
        let _ = grid.query_radius(0.0, 0.0, 5.0, &agents);
        let _ = grid.query_radius(79.9, 59.9, 5.0, &agents);
    }

    #[test]
    fn rebuild_clears_previous_state() {
        let mut grid = make_grid();
        let agents_v1 = vec![prey_at(0, 5.0, 5.0)];
        grid.build(&agents_v1);

        // Rebuild with empty — prior agents should not appear
        grid.build(&[]);
        let result = grid.query_radius(5.0, 5.0, 10.0, &[]);
        assert!(result.is_empty(), "stale agents from prior build should be cleared");
    }
}
