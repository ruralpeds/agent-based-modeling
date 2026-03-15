/// Probabilistic BDI (pBDI) architecture for the physician agent.
///
/// Belief state: discrete distribution over DiseaseState (7 states).
///   Updated by Bayesian likelihood: P(s|obs) ∝ P(obs|s)·P(s)
///
/// Desire / utility: each (state, action) pair has a utility outcome
///   specified in UtilityParams.
///
/// Intention: argmax over TreatmentAction of expected utility, subject to
///   a commitment threshold — the physician keeps the current intention
///   unless EU gain exceeds `commitment_threshold`.
use crate::stochastic::dtmc::DiseaseState;
use super::super::agents::types::{TreatmentAction, N_TREATMENT_ACTIONS};

// ─── LikelihoodTable ─────────────────────────────────────────────────────────

/// 7 × 6 table of P(clinical_sign | disease_state).
/// Rows = DiseaseState (0=Susceptible … 6=Dead).
/// Cols = clinical signs (Fever, PosCulture, AbnWbc, PurulentDrain, Erythema, VentSupp).
/// Values calibrated from published ICU epidemiology (Vincent et al.; Levy et al.).
#[derive(Debug, Clone)]
pub struct LikelihoodTable {
    /// Stored row-major: `data[state * N_SIGNS + sign]`
    pub data: [f32; 7 * 6],
}

pub const N_SIGNS: usize = 6;

impl LikelihoodTable {
    /// Default ICU-calibrated likelihoods.
    pub fn icu_default() -> Self {
        #[rustfmt::skip]
        let data: [f32; 7 * 6] = [
            // Fever  PosCult AbnWbc PurDrn Eryt VentSupp   ← signs
            0.10,  0.00,   0.10,  0.00,  0.00, 0.02,  // Susceptible
            0.25,  0.05,   0.20,  0.05,  0.05, 0.05,  // Exposed
            0.40,  0.30,   0.35,  0.15,  0.10, 0.10,  // Colonized
            0.70,  0.80,   0.65,  0.40,  0.30, 0.30,  // Infected
            0.35,  0.30,   0.30,  0.20,  0.15, 0.15,  // Treated
            0.10,  0.05,   0.10,  0.05,  0.05, 0.05,  // Recovered
            0.05,  0.10,   0.10,  0.05,  0.05, 0.50,  // Dead
        ];
        Self { data }
    }

    /// P(sign `sign_idx` | disease state `state`).
    pub fn p_sign_given_state(&self, state: DiseaseState, sign_idx: usize) -> f32 {
        let row = state as usize;
        self.data[row * N_SIGNS + sign_idx]
    }
}

// ─── UtilityParams ────────────────────────────────────────────────────────────

/// Utility matrix: rows = DiseaseState (7), cols = TreatmentAction (6).
/// Values represent expected patient health outcome on a [−10, 10] scale.
#[derive(Debug, Clone)]
pub struct UtilityParams {
    /// Row-major: `data[state * N_TREATMENT_ACTIONS + action]`
    pub data: [f32; 7 * N_TREATMENT_ACTIONS],
}

impl UtilityParams {
    pub fn icu_default() -> Self {
        #[rustfmt::skip]
        let data: [f32; 7 * N_TREATMENT_ACTIONS] = [
            // Observe  OrderCult  BroadSpec  NarrowSpec  Isolate  Discharge
            8.0,      6.0,       3.0,       5.0,        4.0,    10.0,  // Susceptible
            5.0,      7.0,       4.0,       5.0,        6.0,     3.0,  // Exposed
            3.0,      8.0,       5.0,       6.0,        7.0,     0.0,  // Colonized
           -2.0,      5.0,       7.0,       8.0,        9.0,    -5.0,  // Infected
            4.0,      5.0,       3.0,       7.0,        5.0,     4.0,  // Treated
            7.0,      4.0,       2.0,       3.0,        3.0,     9.0,  // Recovered
           -8.0,     -5.0,      -3.0,      -3.0,       -2.0,    -9.0,  // Dead
        ];
        Self { data }
    }

    pub fn utility(&self, state: DiseaseState, action: TreatmentAction) -> f32 {
        self.data[state as usize * N_TREATMENT_ACTIONS + action.as_index()]
    }
}

// ─── PhysicianPBDI ────────────────────────────────────────────────────────────

/// Physician pBDI: holds a belief distribution over disease states and
/// deliberates to select the expected-utility-maximising treatment action.
#[derive(Debug, Clone)]
pub struct PhysicianPBDI {
    /// Normalised probability distribution over 7 disease states.
    pub belief_state:         [(DiseaseState, f32); 7],
    pub likelihood:           LikelihoodTable,
    pub utility:              UtilityParams,
    /// Current committed intention.
    pub current_intention:    TreatmentAction,
    /// Minimum EU gain required to switch intention (commitment inertia).
    pub commitment_threshold: f32,
}

impl PhysicianPBDI {
    pub fn new(likelihood: LikelihoodTable, utility: UtilityParams) -> Self {
        // Uniform prior over non-terminal states.
        let p = 1.0 / 6.0;
        Self {
            belief_state: [
                (DiseaseState::Susceptible, p),
                (DiseaseState::Exposed,     p),
                (DiseaseState::Colonized,   p),
                (DiseaseState::Infected,    p),
                (DiseaseState::Treated,     p),
                (DiseaseState::Recovered,   p),
                (DiseaseState::Dead,        0.0),
            ],
            likelihood,
            utility,
            current_intention:    TreatmentAction::Observe,
            commitment_threshold: 0.5,
        }
    }

    pub fn icu_default() -> Self {
        Self::new(LikelihoodTable::icu_default(), UtilityParams::icu_default())
    }

    // ── Belief update ─────────────────────────────────────────────────────────

    /// Bayesian update on clinical sign `sign_idx` being present (observed = true)
    /// or absent (observed = false).
    pub fn update_belief(&mut self, sign_idx: usize, observed: bool) {
        let mut weights = [0.0f32; 7];
        for (i, &(state, prior)) in self.belief_state.iter().enumerate() {
            let p_sign = self.likelihood.p_sign_given_state(state, sign_idx);
            let lik = if observed { p_sign } else { 1.0 - p_sign };
            weights[i] = prior * lik;
        }
        let sum: f32 = weights.iter().sum();
        if sum > 1e-8 {
            for (i, entry) in self.belief_state.iter_mut().enumerate() {
                entry.1 = weights[i] / sum;
            }
        }
    }

    // ── Deliberation / EU maximization ────────────────────────────────────────

    /// Compute expected utility of each TreatmentAction under current beliefs.
    fn expected_utilities(&self) -> [f32; N_TREATMENT_ACTIONS] {
        let mut eu = [0.0f32; N_TREATMENT_ACTIONS];
        for (action_idx, slot) in eu.iter_mut().enumerate() {
            let action = TreatmentAction::from_index(action_idx);
            *slot = self.belief_state.iter()
                .map(|&(state, prob)| prob * self.utility.utility(state, action))
                .sum();
        }
        eu
    }

    /// Deliberate: return best TreatmentAction by expected utility, subject to
    /// commitment inertia — only switch if EU gain > `commitment_threshold`.
    pub fn deliberate(&mut self) -> TreatmentAction {
        let eu = self.expected_utilities();
        let current_eu = eu[self.current_intention.as_index()];

        let (best_idx, best_eu) = eu.iter().enumerate()
            .fold((0, f32::NEG_INFINITY), |(bi, bv), (i, &v)| {
                if v > bv { (i, v) } else { (bi, bv) }
            });

        if best_eu - current_eu > self.commitment_threshold {
            self.current_intention = TreatmentAction::from_index(best_idx);
        }
        self.current_intention
    }

    /// Return dominant disease state (highest probability).
    pub fn map_state(&self) -> DiseaseState {
        self.belief_state.iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|&(s, _)| s)
            .unwrap_or(DiseaseState::Susceptible)
    }
}

// ─── TreatmentAction index helpers ───────────────────────────────────────────

impl TreatmentAction {
    pub fn from_index(i: usize) -> Self {
        match i {
            0 => Self::Observe,
            1 => Self::OrderCulture,
            2 => Self::StartBroadSpectrum,
            3 => Self::StartNarrowSpectrum,
            4 => Self::IsolatePrecautions,
            5 => Self::Discharge,
            _ => Self::Observe,
        }
    }
}
