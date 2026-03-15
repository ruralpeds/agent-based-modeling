#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum AgentType {
    Prey = 0,
    Predator = 1,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Action {
    Wander = 0,
    Forage = 1,
    Flee = 2,
    Hunt = 3,
    Rest = 4,
    Patrol = 5,
    Socialize = 6,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Intention {
    Wander = 0,
    Eat = 1,
    Survive = 2,
    Reproduce = 3,
    Hunt = 4,
    Patrol = 5,
    Rest = 6,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Agent {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub energy: f32,
    pub age: u32,
    pub kills: u32,
    pub offspring: u32,
    pub agent_type: AgentType,
    pub action: Action,
    pub intention: Intention,
    pub _padding: [u8; 2],
}

impl Agent {
    pub fn new_prey(id: u32, x: f32, y: f32, energy: f32) -> Self {
        Self {
            id,
            x,
            y,
            energy,
            age: 0,
            kills: 0,
            offspring: 0,
            agent_type: AgentType::Prey,
            action: Action::Wander,
            intention: Intention::Wander,
            _padding: [0; 2],
        }
    }

    pub fn new_predator(id: u32, x: f32, y: f32, energy: f32) -> Self {
        Self {
            id,
            x,
            y,
            energy,
            age: 0,
            kills: 0,
            offspring: 0,
            agent_type: AgentType::Predator,
            action: Action::Wander,
            intention: Intention::Wander,
            _padding: [0; 2],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentDecision {
    pub dx: f32,
    pub dy: f32,
    pub action: Action,
    pub intention: Intention,
    pub target_id: Option<u32>,
}

impl AgentDecision {
    pub fn wander(dx: f32, dy: f32) -> Self {
        Self {
            dx,
            dy,
            action: Action::Wander,
            intention: Intention::Wander,
            target_id: None,
        }
    }
}
