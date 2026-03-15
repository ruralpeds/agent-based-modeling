use crate::engine::{Agent, AgentType};

pub struct SpatialGrid {
    buckets:     Vec<Vec<usize>>,
    bucket_size: f32,
    cols:        usize,
    rows:        usize,
}

impl SpatialGrid {
    pub fn new(world_w: f32, world_h: f32, bucket_size: f32) -> Self {
        let cols = (world_w / bucket_size).ceil() as usize + 1;
        let rows = (world_h / bucket_size).ceil() as usize + 1;
        Self {
            buckets: vec![Vec::with_capacity(16); cols * rows],
            bucket_size,
            cols,
            rows,
        }
    }

    pub fn build(&mut self, agents: &[Agent]) {
        for b in &mut self.buckets { b.clear(); }
        for (i, agent) in agents.iter().enumerate() {
            let bx = ((agent.x / self.bucket_size) as usize).min(self.cols - 1);
            let by = ((agent.y / self.bucket_size) as usize).min(self.rows - 1);
            self.buckets[by * self.cols + bx].push(i);
        }
    }

    /// Returns (x, y, id) of nearest agent of given type within radius
    pub fn nearest_of_type(
        &self,
        cx: f32, cy: f32, radius: f32,
        agent_type: AgentType,
        agents: &[Agent],
    ) -> Option<(f32, f32, u32)> {
        let r2 = radius * radius;
        let bx0 = ((cx - radius) / self.bucket_size).floor().max(0.0) as usize;
        let bx1 = (((cx + radius) / self.bucket_size).ceil() as usize).min(self.cols - 1);
        let by0 = ((cy - radius) / self.bucket_size).floor().max(0.0) as usize;
        let by1 = (((cy + radius) / self.bucket_size).ceil() as usize).min(self.rows - 1);

        let mut best_d2 = r2 + 1.0;
        let mut best:   Option<(f32, f32, u32)> = None;

        for by in by0..=by1 {
            for bx in bx0..=bx1 {
                for &idx in &self.buckets[by * self.cols + bx] {
                    let a = &agents[idx];
                    if a.agent_type != agent_type { continue; }
                    let dx = a.x - cx;
                    let dy = a.y - cy;
                    let d2 = dx * dx + dy * dy;
                    if d2 < best_d2 {
                        best_d2 = d2;
                        best    = Some((a.x, a.y, a.id));
                    }
                }
            }
        }
        best
    }

    /// Returns all agents within radius as (x, y, id, type)
    pub fn query_radius(
        &self,
        cx: f32, cy: f32, radius: f32,
        agents: &[Agent],
    ) -> Vec<(f32, f32, u32, AgentType)> {
        let r2 = radius * radius;
        let bx0 = ((cx - radius) / self.bucket_size).floor().max(0.0) as usize;
        let bx1 = (((cx + radius) / self.bucket_size).ceil() as usize).min(self.cols - 1);
        let by0 = ((cy - radius) / self.bucket_size).floor().max(0.0) as usize;
        let by1 = (((cy + radius) / self.bucket_size).ceil() as usize).min(self.rows - 1);

        let mut result = Vec::new();
        for by in by0..=by1 {
            for bx in bx0..=bx1 {
                for &idx in &self.buckets[by * self.cols + bx] {
                    let a = &agents[idx];
                    let dx = a.x - cx;
                    let dy = a.y - cy;
                    if dx * dx + dy * dy <= r2 {
                        result.push((a.x, a.y, a.id, a.agent_type));
                    }
                }
            }
        }
        result
    }
}
