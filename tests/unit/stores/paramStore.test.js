import { describe, test, expect, beforeEach } from 'vitest';
import { paramStore } from '@/stores/paramStore.js';
import { DEFAULT_PARAMS, PARAM_LIMITS } from '@/constants.js';

beforeEach(() => {
  paramStore.reset();
});

// ── get / getKey ──────────────────────────────────────────────────────────────

describe('paramStore.get', () => {
  test('returns a copy of current params', () => {
    const p = paramStore.get();
    expect(p).toMatchObject(DEFAULT_PARAMS);
  });

  test('returned object is a shallow copy (mutation does not affect store)', () => {
    const p = paramStore.get();
    p.preyCount = 999;
    expect(paramStore.getKey('preyCount')).toBe(DEFAULT_PARAMS.preyCount);
  });
});

describe('paramStore.getKey', () => {
  test('returns the current value of a key', () => {
    expect(paramStore.getKey('preyCount')).toBe(DEFAULT_PARAMS.preyCount);
  });
});

// ── set ───────────────────────────────────────────────────────────────────────

describe('paramStore.set — basic', () => {
  test('stores the new value', () => {
    paramStore.set('preyCount', 100);
    expect(paramStore.getKey('preyCount')).toBe(100);
  });

  test('returns the stored value', () => {
    const v = paramStore.set('preyCount', 150);
    expect(v).toBe(150);
  });
});

describe('paramStore.set — clamping against PARAM_LIMITS', () => {
  test('clamps to min when value is below min', () => {
    const min = PARAM_LIMITS.preyCount.min;
    paramStore.set('preyCount', min - 100);
    expect(paramStore.getKey('preyCount')).toBe(min);
  });

  test('clamps to max when value exceeds max', () => {
    const max = PARAM_LIMITS.preyCount.max;
    paramStore.set('preyCount', max + 1000);
    expect(paramStore.getKey('preyCount')).toBe(max);
  });

  test('value exactly at min is not clamped', () => {
    const min = PARAM_LIMITS.predatorCount.min;
    paramStore.set('predatorCount', min);
    expect(paramStore.getKey('predatorCount')).toBe(min);
  });

  test('value exactly at max is not clamped', () => {
    const max = PARAM_LIMITS.preySpeed.max;
    paramStore.set('preySpeed', max);
    expect(paramStore.getKey('preySpeed')).toBe(max);
  });

  test('unknown key with no PARAM_LIMITS entry is stored as-is', () => {
    // ruleSetId has no numeric limits — should be stored unchanged
    paramStore.set('ruleSetId', 'bdi');
    expect(paramStore.getKey('ruleSetId')).toBe('bdi');
  });
});

// ── setAll ────────────────────────────────────────────────────────────────────

describe('paramStore.setAll', () => {
  test('merges partial params without clearing other keys', () => {
    paramStore.setAll({ preyCount: 200 });
    expect(paramStore.getKey('preyCount')).toBe(200);
    // predatorCount should still have its default value
    expect(paramStore.getKey('predatorCount')).toBe(DEFAULT_PARAMS.predatorCount);
  });

  test('overwrites multiple keys at once', () => {
    paramStore.setAll({ preyCount: 50, predatorCount: 5 });
    expect(paramStore.getKey('preyCount')).toBe(50);
    expect(paramStore.getKey('predatorCount')).toBe(5);
  });
});

// ── reset ─────────────────────────────────────────────────────────────────────

describe('paramStore.reset', () => {
  test('restores all params to DEFAULT_PARAMS', () => {
    paramStore.set('preyCount', 999);
    paramStore.set('predatorCount', 0);
    paramStore.reset();
    expect(paramStore.get()).toMatchObject(DEFAULT_PARAMS);
  });
});

// ── toJSON ────────────────────────────────────────────────────────────────────

describe('paramStore.toJSON', () => {
  test('returns a valid JSON string', () => {
    const json = paramStore.toJSON();
    expect(() => JSON.parse(json)).not.toThrow();
  });

  test('JSON round-trips correctly', () => {
    paramStore.set('preyCount', 77);
    const parsed = JSON.parse(paramStore.toJSON());
    expect(parsed.preyCount).toBe(77);
  });
});
