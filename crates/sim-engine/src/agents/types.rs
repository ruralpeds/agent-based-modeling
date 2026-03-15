/// Shared enum/struct types used across all healthcare agent modules.
/// No imports from this crate — pure primitive types only.

// ─── Demographics ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Sex {
    Male = 0,
    Female = 1,
    Other = 2,
}

// ─── Hospital layout ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HospitalUnit {
    Icu = 0,
    Ward = 1,
    Ed = 2,
    Ltc = 3,
}

// ─── Pathogen resistance ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum ResistanceProfile {
    FullySusceptible = 0,
    MethicillinR = 1,           // MRSA
    VancomycinIntermediate = 2, // VISA
    Pandrug = 3,                // PDR — no effective agents
}

impl ResistanceProfile {
    /// Whether the pathogen is still susceptible to this antibiotic.
    pub fn is_susceptible_to(self, ab: AntibioticClass) -> bool {
        match (self, ab) {
            (Self::FullySusceptible, _) => true,
            (Self::MethicillinR, AntibioticClass::BetaLactam) => false,
            (Self::MethicillinR, _) => true,
            (Self::VancomycinIntermediate, AntibioticClass::Vancomycin) => false,
            (Self::VancomycinIntermediate, _) => true,
            (Self::Pandrug, _) => false,
        }
    }
}

// ─── Antibiotic classes ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AntibioticClass {
    BetaLactam = 0,
    Vancomycin = 1,
    Linezolid = 2,
    Daptomycin = 3,
    Cephalosporin = 4,
}

pub const N_ANTIBIOTIC_CLASSES: usize = 5;

// ─── Nurse activities ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NurseActivity {
    BedsideCare = 0,
    MedicationAdmin = 1,
    Assessment = 2,
    Documentation = 3,
    Walking = 4,
}

// ─── Physician treatment actions ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TreatmentAction {
    Observe = 0,
    OrderCulture = 1,
    StartBroadSpectrum = 2,
    StartNarrowSpectrum = 3,
    IsolatePrecautions = 4,
    Discharge = 5,
}

pub const N_TREATMENT_ACTIONS: usize = 6;

impl TreatmentAction {
    pub fn as_index(self) -> usize {
        self as usize
    }
}

// ─── Physician intentions ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhysicianIntention {
    Watchful,
    Treat(TreatmentAction),
    Discharge,
    Escalate,
}

// ─── Transmission route ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContactRoute {
    DirectContact,   // hands-on care
    IndirectContact, // via bed / equipment / fomite
    Droplet,         // respiratory / proximity
}

// ─── Provider social-learning state ──────────────────────────────────────────

/// Embeddable compliance state used by NurseAgent and PhysicianAgent for
/// Fermi-imitation social learning and audit-and-feedback updates.
#[derive(Debug, Clone)]
pub struct ProviderSocialState {
    pub id: u32,
    pub current_compliance: f32, // [0, 1]
    pub received_feedback_this_period: bool,
}

impl ProviderSocialState {
    pub fn new(id: u32, initial_compliance: f32) -> Self {
        Self {
            id,
            current_compliance: initial_compliance.clamp(0.0, 1.0),
            received_feedback_this_period: false,
        }
    }
}
