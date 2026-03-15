use crate::engine::Agent;

/// Write agents in SoA layout to a f32 slice.
/// Layout: [pos_x * N][pos_y * N][energy * N][type * N][action * N]
///         [age * N][intention * N][kills * N][offspring * N][id * N]
pub fn write_soa(agents: &[Agent], buf: &mut [f32]) {
    let n = agents.len();
    if n == 0 || buf.len() < n * 10 {
        return;
    }
    for (i, a) in agents.iter().enumerate() {
        buf[i] = a.x;
        buf[n + i] = a.y;
        buf[2 * n + i] = a.energy;
        buf[3 * n + i] = a.agent_type as u8 as f32;
        buf[4 * n + i] = a.action as u8 as f32;
        buf[5 * n + i] = a.age as f32;
        buf[6 * n + i] = a.intention as u8 as f32;
        buf[7 * n + i] = a.kills as f32;
        buf[8 * n + i] = a.offspring as f32;
        buf[9 * n + i] = a.id as f32;
    }
}
