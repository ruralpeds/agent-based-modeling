import { describe, test, expect, beforeEach } from 'vitest';
import { errorStore } from '@/stores/errorStore.js';
import { ABMDispatch } from '@/stores/dispatch.js';

beforeEach(() => {
  errorStore.clear();
});

// ── initial state ─────────────────────────────────────────────────────────────

describe('errorStore — initial state', () => {
  test('hasErrors is false when store is empty', () => {
    expect(errorStore.hasErrors()).toBe(false);
  });

  test('getLatest returns null when store is empty', () => {
    expect(errorStore.getLatest()).toBeNull();
  });
});

// ── dispatch-driven accumulation ─────────────────────────────────────────────

describe('errorStore — populated via engine:error dispatch', () => {
  test('hasErrors is true after one error event', () => {
    ABMDispatch.call('engine:error', {}, { error: new Error('boom') });
    expect(errorStore.hasErrors()).toBe(true);
  });

  test('getLatest returns the most recent error message', () => {
    ABMDispatch.call('engine:error', {}, { error: new Error('first error') });
    ABMDispatch.call('engine:error', {}, { error: new Error('second error') });
    const latest = errorStore.getLatest();
    expect(latest).not.toBeNull();
    expect(latest.message).toContain('second error');
  });

  test('getLatest contains a ts timestamp (number)', () => {
    ABMDispatch.call('engine:error', {}, { error: 'oops' });
    const latest = errorStore.getLatest();
    expect(typeof latest.ts).toBe('number');
    expect(latest.ts).toBeGreaterThan(0);
  });

  test('string error is stringified to message', () => {
    ABMDispatch.call('engine:error', {}, { error: 'string-only error' });
    expect(errorStore.getLatest().message).toContain('string-only error');
  });

  test('null error falls back to "Unknown error"', () => {
    ABMDispatch.call('engine:error', {}, { error: null });
    expect(errorStore.getLatest().message).toBe('Unknown error');
  });
});

// ── clear ─────────────────────────────────────────────────────────────────────

describe('errorStore.clear', () => {
  test('clear removes all accumulated errors', () => {
    ABMDispatch.call('engine:error', {}, { error: 'err1' });
    ABMDispatch.call('engine:error', {}, { error: 'err2' });
    errorStore.clear();
    expect(errorStore.hasErrors()).toBe(false);
    expect(errorStore.getLatest()).toBeNull();
  });

  test('clearing an already-empty store does not throw', () => {
    expect(() => errorStore.clear()).not.toThrow();
  });
});
