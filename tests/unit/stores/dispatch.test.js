import { describe, test, expect, vi } from 'vitest';
import { ABMDispatch } from '@/stores/dispatch.js';

// ── Event registration ────────────────────────────────────────────────────────

const EXPECTED_EVENTS = [
  // ABM predator-prey events
  'sim:tick',
  'sim:reset',
  'sim:start',
  'sim:pause',
  'param:change',
  'agent:select',
  'engine:error',
  'ui:resize',
  // Hospital simulation events
  'hospital:tick',
  'hospital:reset',
  'hospital:start',
  'hospital:pause',
  'hospital:outbreak',
];

describe('ABMDispatch — registered events', () => {
  EXPECTED_EVENTS.forEach(event => {
    test(`"${event}" is a registered dispatch event`, () => {
      // d3-dispatch throws if you call .on() with an unregistered type;
      // we verify by registering and then removing a no-op handler.
      expect(() => {
        ABMDispatch.on(`${event}._test_probe`, () => {});
        ABMDispatch.on(`${event}._test_probe`, null); // deregister
      }).not.toThrow();
    });
  });

  test('all 13 expected events are present', () => {
    expect(EXPECTED_EVENTS.length).toBe(13);
  });
});

// ── Pub/sub round-trip ───────────────────────────────────────────────────────

describe('ABMDispatch — call / on round-trip', () => {
  test('sim:tick listener receives the dispatched data', () => {
    const handler = vi.fn();
    ABMDispatch.on('sim:tick._dispatch_test', handler);
    ABMDispatch.call('sim:tick', {}, { tick: 42, count: 5 });
    expect(handler).toHaveBeenCalledOnce();
    expect(handler.mock.calls[0][0]).toMatchObject({ tick: 42, count: 5 });
    ABMDispatch.on('sim:tick._dispatch_test', null);
  });

  test('hospital:outbreak listener receives outbreak data', () => {
    const handler = vi.fn();
    ABMDispatch.on('hospital:outbreak._dispatch_test', handler);
    ABMDispatch.call('hospital:outbreak', {}, { tick: 100 });
    expect(handler).toHaveBeenCalledOnce();
    expect(handler.mock.calls[0][0]).toMatchObject({ tick: 100 });
    ABMDispatch.on('hospital:outbreak._dispatch_test', null);
  });

  test('deregistered listener is not called', () => {
    const handler = vi.fn();
    ABMDispatch.on('engine:error._dispatch_test', handler);
    ABMDispatch.on('engine:error._dispatch_test', null); // deregister
    ABMDispatch.call('engine:error', {}, { error: 'test' });
    expect(handler).not.toHaveBeenCalled();
  });

  test('multiple listeners on the same event are all called', () => {
    const h1 = vi.fn();
    const h2 = vi.fn();
    ABMDispatch.on('sim:reset._dispatch_test1', h1);
    ABMDispatch.on('sim:reset._dispatch_test2', h2);
    ABMDispatch.call('sim:reset', {}, { seed: 1 });
    expect(h1).toHaveBeenCalledOnce();
    expect(h2).toHaveBeenCalledOnce();
    ABMDispatch.on('sim:reset._dispatch_test1', null);
    ABMDispatch.on('sim:reset._dispatch_test2', null);
  });
});
