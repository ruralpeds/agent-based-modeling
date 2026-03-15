import { describe, test, expect } from 'vitest';
import {
  DISEASE_STATE_COLORS,
  DISEASE_STATE_NAMES,
  DEFAULT_HOSPITAL_PARAMS,
  HOSPITAL_PARAM_LIMITS,
  HOSPITAL_PALETTE,
  DEFAULT_PARAMS,
  PARAM_LIMITS,
} from '@/constants.js';

// ── DISEASE_STATE_COLORS ──────────────────────────────────────────────────────

describe('DISEASE_STATE_COLORS', () => {
  test('has exactly 7 entries (one per DiseaseState)', () => {
    expect(DISEASE_STATE_COLORS).toHaveLength(7);
  });

  test('every entry is a valid CSS hex colour', () => {
    DISEASE_STATE_COLORS.forEach(c => {
      expect(c).toMatch(/^#[0-9a-fA-F]{6}$/);
    });
  });

  test('all colours are unique (no accidental duplicates)', () => {
    expect(new Set(DISEASE_STATE_COLORS).size).toBe(7);
  });
});

// ── DISEASE_STATE_NAMES ───────────────────────────────────────────────────────

describe('DISEASE_STATE_NAMES', () => {
  test('has exactly 7 entries matching DISEASE_STATE_COLORS', () => {
    expect(DISEASE_STATE_NAMES).toHaveLength(DISEASE_STATE_COLORS.length);
  });

  test('all entries are non-empty strings', () => {
    DISEASE_STATE_NAMES.forEach(n => {
      expect(typeof n).toBe('string');
      expect(n.length).toBeGreaterThan(0);
    });
  });

  test('contains expected state names', () => {
    expect(DISEASE_STATE_NAMES).toContain('Susceptible');
    expect(DISEASE_STATE_NAMES).toContain('Infected');
    expect(DISEASE_STATE_NAMES).toContain('Recovered');
    expect(DISEASE_STATE_NAMES).toContain('Deceased');
  });
});

// ── DEFAULT_HOSPITAL_PARAMS ───────────────────────────────────────────────────

describe('DEFAULT_HOSPITAL_PARAMS — shape', () => {
  test('has icuBeds (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.icuBeds).toBe('number'));
  test('has wardBeds (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.wardBeds).toBe('number'));
  test('has nNurses (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.nNurses).toBe('number'));
  test('has nPhysicians (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.nPhysicians).toBe('number'));
  test('has ticksPerHour (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.ticksPerHour).toBe('number'));
  test('has baseArrivalRate (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.baseArrivalRate).toBe('number'));
  test('has admissionColonizationRate (number)', () => {
    expect(typeof DEFAULT_HOSPITAL_PARAMS.admissionColonizationRate).toBe('number');
  });
  test('has pathogen object with dose_response_r', () => {
    expect(typeof DEFAULT_HOSPITAL_PARAMS.pathogen).toBe('object');
    expect(typeof DEFAULT_HOSPITAL_PARAMS.pathogen.dose_response_r).toBe('number');
  });
  test('has dtmcBeta (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.dtmcBeta).toBe('number'));
  test('has dtmcGamma (number)', () => expect(typeof DEFAULT_HOSPITAL_PARAMS.dtmcGamma).toBe('number'));
  test('has nurse object with hh_intercept', () => {
    expect(typeof DEFAULT_HOSPITAL_PARAMS.nurse).toBe('object');
    expect(typeof DEFAULT_HOSPITAL_PARAMS.nurse.hh_intercept).toBe('number');
  });
  test('has cox object with age_coef', () => {
    expect(typeof DEFAULT_HOSPITAL_PARAMS.cox).toBe('object');
    expect(typeof DEFAULT_HOSPITAL_PARAMS.cox.age_coef).toBe('number');
  });
  test('has meanDispenserDist (number)', () => {
    expect(typeof DEFAULT_HOSPITAL_PARAMS.meanDispenserDist).toBe('number');
  });
  test('has socialLearningPeriod (number)', () => {
    expect(typeof DEFAULT_HOSPITAL_PARAMS.socialLearningPeriod).toBe('number');
  });
});

describe('DEFAULT_HOSPITAL_PARAMS — value sanity', () => {
  test('ticksPerHour is 12 (5-minute ticks)', () => {
    expect(DEFAULT_HOSPITAL_PARAMS.ticksPerHour).toBe(12);
  });

  test('admissionColonizationRate is in [0, 1]', () => {
    const r = DEFAULT_HOSPITAL_PARAMS.admissionColonizationRate;
    expect(r).toBeGreaterThanOrEqual(0);
    expect(r).toBeLessThanOrEqual(1);
  });

  test('baseArrivalRate is positive', () => {
    expect(DEFAULT_HOSPITAL_PARAMS.baseArrivalRate).toBeGreaterThan(0);
  });

  test('DTMC rates are all in [0, 1]', () => {
    const rates = ['dtmcLambda','dtmcAlpha','dtmcDeltaE','dtmcBeta','dtmcGamma',
                   'dtmcEta','dtmcPsi','dtmcRho','dtmcPhi','dtmcTheta'];
    rates.forEach(k => {
      const v = DEFAULT_HOSPITAL_PARAMS[k];
      expect(v, `${k} should be ≥ 0`).toBeGreaterThanOrEqual(0);
      expect(v, `${k} should be ≤ 1`).toBeLessThanOrEqual(1);
    });
  });

  test('icuBeds + wardBeds > 0', () => {
    expect(DEFAULT_HOSPITAL_PARAMS.icuBeds + DEFAULT_HOSPITAL_PARAMS.wardBeds).toBeGreaterThan(0);
  });
});

// ── HOSPITAL_PARAM_LIMITS ─────────────────────────────────────────────────────

describe('HOSPITAL_PARAM_LIMITS', () => {
  test('each entry has min, max, step', () => {
    Object.entries(HOSPITAL_PARAM_LIMITS).forEach(([key, limits]) => {
      expect(limits, key).toHaveProperty('min');
      expect(limits, key).toHaveProperty('max');
      expect(limits, key).toHaveProperty('step');
    });
  });

  test('min < max for every limit entry', () => {
    Object.entries(HOSPITAL_PARAM_LIMITS).forEach(([key, limits]) => {
      expect(limits.min, `${key}.min < max`).toBeLessThan(limits.max);
    });
  });

  test('step > 0 for every limit entry', () => {
    Object.entries(HOSPITAL_PARAM_LIMITS).forEach(([key, limits]) => {
      expect(limits.step, `${key}.step > 0`).toBeGreaterThan(0);
    });
  });

  test('baseArrivalRate is among the defined limits', () => {
    expect(HOSPITAL_PARAM_LIMITS).toHaveProperty('baseArrivalRate');
  });
});

// ── HOSPITAL_PALETTE ──────────────────────────────────────────────────────────

describe('HOSPITAL_PALETTE', () => {
  test('has a background colour', () => {
    expect(typeof HOSPITAL_PALETTE.background).toBe('string');
  });

  test('has outbreak and safe colours for status indication', () => {
    expect(typeof HOSPITAL_PALETTE.outbreak).toBe('string');
    expect(typeof HOSPITAL_PALETTE.safe).toBe('string');
  });
});

// ── ABM constants still intact ────────────────────────────────────────────────

describe('ABM DEFAULT_PARAMS still intact after hospital additions', () => {
  test('preyCount is defined', () => expect(typeof DEFAULT_PARAMS.preyCount).toBe('number'));
  test('predatorCount is defined', () => expect(typeof DEFAULT_PARAMS.predatorCount).toBe('number'));
  test('ruleSetId is defined', () => expect(typeof DEFAULT_PARAMS.ruleSetId).toBe('string'));
  test('PARAM_LIMITS has preyCount', () => expect(PARAM_LIMITS).toHaveProperty('preyCount'));
});
