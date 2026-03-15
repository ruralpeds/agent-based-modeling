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
