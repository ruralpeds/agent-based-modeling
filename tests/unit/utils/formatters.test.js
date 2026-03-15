import { describe, test, expect } from 'vitest';
import { fmtTick, fmtEnergy, fmtPct, fmtCount } from '@/utils/formatters.js';

describe('fmtTick', () => {
  test('returns a string', () => {
    expect(typeof fmtTick(0)).toBe('string');
    expect(typeof fmtTick(1000)).toBe('string');
  });

  test('formats zero', () => {
    expect(fmtTick(0)).toBe((0).toLocaleString());
  });

  test('formats small integers', () => {
    expect(fmtTick(42)).toBe((42).toLocaleString());
    expect(fmtTick(999)).toBe((999).toLocaleString());
  });

  test('formats large tick counts', () => {
    const result = fmtTick(1_000_000);
    expect(typeof result).toBe('string');
    expect(result.length).toBeGreaterThan(0);
  });
});

describe('fmtEnergy', () => {
  test('formats integer energy to one decimal place', () => {
    expect(fmtEnergy(100)).toBe('100.0');
    expect(fmtEnergy(0)).toBe('0.0');
    expect(fmtEnergy(200)).toBe('200.0');
  });

  test('rounds to one decimal', () => {
    // Note: 50.15 → '50.1' due to IEEE 754 (50.15 is stored as 50.149999...)
    expect(fmtEnergy(99.95)).toBe('100.0');
    expect(fmtEnergy(50.14)).toBe('50.1');
    expect(fmtEnergy(50.25)).toBe('50.3');
  });

  test('handles negative energy (edge case)', () => {
    expect(fmtEnergy(-1.5)).toBe('-1.5');
  });

  test('handles very small values', () => {
    expect(fmtEnergy(0.001)).toBe('0.0');
    expect(fmtEnergy(0.05)).toBe('0.1');
  });
});

describe('fmtPct', () => {
  test('formats 0 as 0.0%', () => {
    expect(fmtPct(0)).toBe('0.0%');
  });

  test('formats 1 as 100.0%', () => {
    expect(fmtPct(1)).toBe('100.0%');
  });

  test('formats 0.5 as 50.0%', () => {
    expect(fmtPct(0.5)).toBe('50.0%');
  });

  test('formats fractional percentages to one decimal', () => {
    expect(fmtPct(0.333)).toBe('33.3%');
    expect(fmtPct(0.666)).toBe('66.6%');
    expect(fmtPct(0.1234)).toBe('12.3%');
  });

  test('always appends % sign', () => {
    expect(fmtPct(0.75)).toMatch(/%$/);
  });
});

describe('fmtCount', () => {
  test('converts number to string', () => {
    expect(fmtCount(0)).toBe('0');
    expect(fmtCount(42)).toBe('42');
    expect(fmtCount(5000)).toBe('5000');
  });

  test('returns a string type', () => {
    expect(typeof fmtCount(0)).toBe('string');
    expect(typeof fmtCount(999)).toBe('string');
  });

  test('handles large counts', () => {
    expect(fmtCount(1_000_000)).toBe('1000000');
  });
});
