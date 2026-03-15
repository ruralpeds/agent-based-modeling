export const GRID_W  = 80;
export const GRID_H  = 60;
export const MAX_AGENTS = 5000;
export const FIELDS_PER_AGENT = 10;
export const MAX_ENERGY = 200;

export const PALETTE = {
  background:   '#0f172a',
  grass:        '#166534',
  grassDry:     '#78350f',
  prey:         '#22c55e',
  preyStroke:   '#15803d',
  predator:     '#ef4444',
  predStroke:   '#b91c1c',
  fleeing:      '#fbbf24',
  hunting:      '#f97316',
  text:         '#e2e8f0',
  textMuted:    '#94a3b8',
  border:       '#1e293b',
  panel:        '#0f172a',
  panelBorder:  '#1e293b',
  accent:       '#3b82f6',
};

export const PREY_RADIUS = 3.5;
export const PRED_RADIUS = 5;

/** Default simulation parameters — mirrors Rust SimParams (camelCase) */
export const DEFAULT_PARAMS = {
  preyCount:                80,
  predatorCount:            15,
  preySpeed:                0.35,
  predatorSpeed:            0.45,
  preyVision:               5.0,
  predatorVision:           8.0,
  preyReproduceEnergy:      120.0,
  predatorReproduceEnergy:  140.0,
  preyEnergyGain:           15.0,
  predatorEnergyGain:       40.0,
  preyMetabolism:           0.5,
  predatorMetabolism:       0.8,
  maxEnergy:                200.0,
  grassRegrowRate:          0.03,
  ruleSetId:                'reactive',
};

export const PARAM_LIMITS = {
  preyCount:               { min: 1,  max: 400, step: 1 },
  predatorCount:           { min: 1,  max: 100, step: 1 },
  preySpeed:               { min: 0.05, max: 2.0, step: 0.05 },
  predatorSpeed:           { min: 0.05, max: 2.0, step: 0.05 },
  preyVision:              { min: 1,  max: 20,  step: 0.5 },
  predatorVision:          { min: 1,  max: 20,  step: 0.5 },
  preyReproduceEnergy:     { min: 50, max: 190, step: 5 },
  predatorReproduceEnergy: { min: 50, max: 190, step: 5 },
  preyEnergyGain:          { min: 1,  max: 50,  step: 1 },
  predatorEnergyGain:      { min: 5,  max: 100, step: 5 },
  preyMetabolism:          { min: 0.1, max: 3.0, step: 0.1 },
  predatorMetabolism:      { min: 0.1, max: 3.0, step: 0.1 },
  grassRegrowRate:         { min: 0.001, max: 0.2, step: 0.001 },
};

export const ACTION_NAMES = ['Wander','Forage','Flee','Hunt','Rest','Patrol','Socialize'];
export const ACTION_COLORS = ['#64748b','#22c55e','#fbbf24','#ef4444','#3b82f6','#8b5cf6','#ec4899'];

// ─── Hospital simulation ──────────────────────────────────────────────────────

export const HOSPITAL_PALETTE = {
  background:   '#0f172a',
  icuBed:       '#1e3a5f',
  wardBed:      '#1a2f1a',
  bedBorder:    '#334155',
  bedOccupied:  '#1e293b',
  text:         '#e2e8f0',
  textMuted:    '#94a3b8',
  border:       '#1e293b',
  panel:        '#0f172a',
  panelBorder:  '#1e293b',
  accent:       '#3b82f6',
  outbreak:     '#dc2626',
  safe:         '#16a34a',
};

/** Colour per DiseaseState enum (index = discriminant from Rust) */
export const DISEASE_STATE_COLORS = [
  '#3b82f6',   // 0 Susceptible  — blue
  '#f59e0b',   // 1 Exposed      — amber
  '#f97316',   // 2 Colonized    — orange
  '#ef4444',   // 3 Infected     — red
  '#8b5cf6',   // 4 Treated      — violet
  '#22c55e',   // 5 Recovered    — green
  '#475569',   // 6 Deceased     — slate
];

export const DISEASE_STATE_NAMES = [
  'Susceptible', 'Exposed', 'Colonized', 'Infected', 'Treated', 'Recovered', 'Deceased',
];

/** Default params — mirrors HospitalParams::default() (serde camelCase) */
export const DEFAULT_HOSPITAL_PARAMS = {
  icuBeds:                  20,
  wardBeds:                 80,
  nNurses:                  30,
  nPhysicians:              10,
  ticksPerHour:             12,
  baseArrivalRate:          3.2,
  admissionColonizationRate: 0.06,
  pathogen: {
    dose_response_r:          0.05,
    hand_wash_factor:          0.90,
    mutation_rate_per_ab_day:  0.002,
  },
  dtmcLambda:  0.002,
  dtmcAlpha:   0.10,
  dtmcDeltaE:  0.05,
  dtmcBeta:    0.08,
  dtmcGamma:   0.30,
  dtmcEta:     0.02,
  dtmcPsi:     0.40,
  dtmcRho:     0.01,
  dtmcPhi:     0.001,
  dtmcTheta:   0.05,
  nurse: {
    hh_intercept:         0.62,
    hh_workload_coef:    -0.35,
    hh_fatigue_coef:     -0.50,
    hh_distance_coef:    -0.15,
    hh_audit_decay_coef: -0.002,
  },
  cox: {
    age_coef:      0.02,
    cci_coef:      0.08,
    severity_coef: 0.10,
    vent_coef:     0.45,
    cline_coef:    0.25,
  },
  meanDispenserDist:    2.0,
  socialLearningPeriod: 96,
};

export const HOSPITAL_PARAM_LIMITS = {
  baseArrivalRate:           { min: 0.5,  max: 30.0, step: 0.5  },
  admissionColonizationRate: { min: 0.0,  max: 1.0,  step: 0.01 },
  dtmcBeta:                  { min: 0.0,  max: 0.50, step: 0.01 },
  dtmcGamma:                 { min: 0.0,  max: 1.0,  step: 0.01 },
  dtmcPsi:                   { min: 0.0,  max: 1.0,  step: 0.01 },
  meanDispenserDist:         { min: 0.5,  max: 10.0, step: 0.5  },
  nNurses:                   { min: 5,    max: 60,   step: 1    },
};
