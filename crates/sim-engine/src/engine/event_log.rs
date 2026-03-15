#[derive(Debug, Clone)]
pub enum SimEvent {
    Birth {
        id: u32,
        parent_id: u32,
        tick: u64,
    },
    Death {
        id: u32,
        cause: DeathCause,
        tick: u64,
    },
    Kill {
        predator_id: u32,
        prey_id: u32,
        tick: u64,
    },
}

#[derive(Debug, Clone)]
pub enum DeathCause {
    Starvation,
    Predation,
    OldAge,
}

pub struct EventLog {
    events: Vec<SimEvent>,
    max_len: usize,
}

impl EventLog {
    pub fn new(max_len: usize) -> Self {
        Self {
            events: Vec::with_capacity(max_len),
            max_len,
        }
    }

    pub fn push(&mut self, event: SimEvent) {
        if self.events.len() >= self.max_len {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    pub fn recent(&self, n: usize) -> &[SimEvent] {
        let start = self.events.len().saturating_sub(n);
        &self.events[start..]
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}
