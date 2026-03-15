import { describe, test, expect, beforeEach } from 'vitest';
import { uiStore } from '@/stores/uiStore.js';

// Reset relevant state before each test to keep tests independent
beforeEach(() => {
  uiStore.set('selectedAgentId', null);
  uiStore.set('isRunning', false);
  uiStore.set('speed', 1);
  uiStore.set('activeChart', 'population');
  uiStore.set('zoom', 1);
  uiStore.set('showTrails', false);
  uiStore.set('showVision', false);
  uiStore.set('showGrid', false);
});

// ── get / set ────────────────────────────────────────────────────────────────

describe('uiStore.get / set', () => {
  test('initial isRunning is false', () => {
    expect(uiStore.get('isRunning')).toBe(false);
  });

  test('initial selectedAgentId is null', () => {
    expect(uiStore.get('selectedAgentId')).toBeNull();
  });

  test('initial speed is 1', () => {
    expect(uiStore.get('speed')).toBe(1);
  });

  test('set stores a new value', () => {
    uiStore.set('zoom', 2.5);
    expect(uiStore.get('zoom')).toBe(2.5);
  });

  test('set overwrites a previous value', () => {
    uiStore.set('activeChart', 'population');
    uiStore.set('activeChart', 'energy');
    expect(uiStore.get('activeChart')).toBe('energy');
  });

  test('boolean flags can be toggled', () => {
    uiStore.set('showGrid', true);
    expect(uiStore.get('showGrid')).toBe(true);
    uiStore.set('showGrid', false);
    expect(uiStore.get('showGrid')).toBe(false);
  });
});

// ── setRunning ────────────────────────────────────────────────────────────────

describe('uiStore.setRunning', () => {
  test('sets isRunning to true', () => {
    uiStore.setRunning(true);
    expect(uiStore.get('isRunning')).toBe(true);
  });

  test('sets isRunning to false', () => {
    uiStore.setRunning(true);
    uiStore.setRunning(false);
    expect(uiStore.get('isRunning')).toBe(false);
  });
});

// ── setSpeed ──────────────────────────────────────────────────────────────────

describe('uiStore.setSpeed', () => {
  test('updates speed value', () => {
    uiStore.setSpeed(4);
    expect(uiStore.get('speed')).toBe(4);
  });

  test('fractional speed is stored exactly', () => {
    uiStore.setSpeed(0.25);
    expect(uiStore.get('speed')).toBe(0.25);
  });
});

// ── selectAgent ───────────────────────────────────────────────────────────────

describe('uiStore.selectAgent', () => {
  test('sets selectedAgentId to the given id', () => {
    uiStore.selectAgent(42);
    expect(uiStore.get('selectedAgentId')).toBe(42);
  });

  test('passing null deselects agent', () => {
    uiStore.selectAgent(7);
    uiStore.selectAgent(null);
    expect(uiStore.get('selectedAgentId')).toBeNull();
  });

  test('sequential selections overwrite', () => {
    uiStore.selectAgent(1);
    uiStore.selectAgent(2);
    expect(uiStore.get('selectedAgentId')).toBe(2);
  });
});
