/// Bed/room agent: tracks surface contamination, dispenser proximity,
/// and patient occupancy. Contamination drives indirect-contact transmission.
use crate::rng::Mulberry32;

// ─── BedAgent ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BedAgent {
    pub id:              u16,
    pub room_id:         u16,
    /// Probability that touching this bed/surface transmits pathogen [0, 1].
    pub contamination:   f32,
    /// Distance in metres to nearest hand-hygiene dispenser.
    pub dispenser_dist:  f32,
    /// Currently assigned patient (None = unoccupied).
    pub patient_id:      Option<u32>,
    /// Ticks since last terminal clean.
    pub ticks_dirty:     u32,
}

impl BedAgent {
    pub fn new(id: u16, room_id: u16, dispenser_dist: f32) -> Self {
        Self {
            id,
            room_id,
            contamination:  0.0,
            dispenser_dist: dispenser_dist.max(0.0),
            patient_id:     None,
            ticks_dirty:    0,
        }
    }

    // ── Occupancy ─────────────────────────────────────────────────────────────

    /// Returns false if the bed is already occupied.
    pub fn assign_patient(&mut self, patient_id: u32) -> bool {
        if self.patient_id.is_none() {
            self.patient_id = Some(patient_id);
            true
        } else {
            false
        }
    }

    pub fn release_patient(&mut self) {
        self.patient_id = None;
    }

    pub fn is_occupied(&self) -> bool {
        self.patient_id.is_some()
    }

    // ── Contamination dynamics ────────────────────────────────────────────────

    /// Deposit pathogen on bed surface after a contact event.
    /// `deposition` is the fraction of pathogen load transferred to surface.
    pub fn contaminate(&mut self, deposition: f32) {
        self.contamination = (self.contamination + deposition).clamp(0.0, 1.0);
        self.ticks_dirty   = self.ticks_dirty.saturating_add(1);
    }

    /// Passive decay each tick: contamination half-life ≈ 8 h at 12 ticks/h.
    /// k = ln(2) / 96 ticks ≈ 0.00722 per tick.
    pub fn decay_contamination(&mut self) {
        const DECAY: f32 = 0.007_22;
        self.contamination *= 1.0 - DECAY;
        if self.contamination < 1e-4 {
            self.contamination = 0.0;
        }
        self.ticks_dirty = self.ticks_dirty.saturating_add(1);
    }

    /// Terminal / deep clean: reset contamination and dirty counter.
    pub fn deep_clean(&mut self) {
        self.contamination = 0.0;
        self.ticks_dirty   = 0;
    }

    /// Routine wipe-down: reduce contamination by `efficacy` (e.g. 0.75).
    pub fn routine_clean(&mut self, efficacy: f32) {
        self.contamination *= 1.0 - efficacy.clamp(0.0, 1.0);
    }

    // ── Indirect-contact transmission ─────────────────────────────────────────

    /// Bernoulli draw: does touching this bed transmit to a touching agent?
    /// Base probability equals surface contamination; no wash → full exposure.
    pub fn indirect_contact_event(&self, hand_washed: bool, rng: &mut Mulberry32) -> bool {
        if hand_washed { return false; }
        rng.next_f32() < self.contamination
    }
}
