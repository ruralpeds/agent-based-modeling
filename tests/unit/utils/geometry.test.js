import { describe, test, expect } from 'vitest';
import { dist2, clamp, lerp } from '@/utils/geometry.js';

describe('dist2', () => {
  test('distance from origin to (3, 4) is 25', () => {
    // Reference: 3² + 4² = 9 + 16 = 25
    expect(dist2(0, 0, 3, 4)).toBe(25);
  });

  test('distance from point to itself is 0', () => {
    expect(dist2(5, 7, 5, 7)).toBe(0);
    expect(dist2(0, 0, 0, 0)).toBe(0);
  });

  test('distance is symmetric', () => {
    expect(dist2(1, 2, 4, 6)).toBe(dist2(4, 6, 1, 2));
  });

  test('distance between (1,1) and (4,5) is 25', () => {
    // dx=3, dy=4 → 9+16=25
    expect(dist2(1, 1, 4, 5)).toBe(25);
  });

  test('handles negative coordinates', () => {
    // (-1,0) to (2,0) → dx=3 → 9
    expect(dist2(-1, 0, 2, 0)).toBe(9);
  });

  test('handles floating-point coordinates', () => {
    // (0,0) to (1,1) → 2
    expect(dist2(0, 0, 1, 1)).toBeCloseTo(2, 10);
  });
});

describe('clamp', () => {
  test('value within range is unchanged', () => {
    expect(clamp(5, 0, 10)).toBe(5);
    expect(clamp(0, 0, 10)).toBe(0);
    expect(clamp(10, 0, 10)).toBe(10);
  });

  test('value below minimum is clamped to minimum', () => {
    expect(clamp(-5, 0, 10)).toBe(0);
    expect(clamp(-100, -50, 50)).toBe(-50);
  });

  test('value above maximum is clamped to maximum', () => {
    expect(clamp(15, 0, 10)).toBe(10);
    expect(clamp(200, 0, 100)).toBe(100);
  });

  test('clamps agent energy to [0, MAX_ENERGY]', () => {
    // Simulates energy clamping used in the renderer
    expect(clamp(-1, 0, 200)).toBe(0);
    expect(clamp(250, 0, 200)).toBe(200);
    expect(clamp(100, 0, 200)).toBe(100);
  });

  test('handles equal lo and hi', () => {
    expect(clamp(5, 3, 3)).toBe(3);
    expect(clamp(3, 3, 3)).toBe(3);
  });

  test('handles floating-point bounds', () => {
    expect(clamp(0.75, 0.0, 1.0)).toBe(0.75);
    expect(clamp(1.5, 0.0, 1.0)).toBe(1.0);
    expect(clamp(-0.1, 0.0, 1.0)).toBe(0.0);
  });
});

describe('lerp', () => {
  test('t=0 returns a', () => {
    expect(lerp(0, 100, 0)).toBe(0);
    expect(lerp(5, 20, 0)).toBe(5);
  });

  test('t=1 returns b', () => {
    expect(lerp(0, 100, 1)).toBe(100);
    expect(lerp(5, 20, 1)).toBe(20);
  });

  test('t=0.5 returns midpoint', () => {
    expect(lerp(0, 100, 0.5)).toBe(50);
    expect(lerp(10, 20, 0.5)).toBe(15);
  });

  test('interpolates linearly between values', () => {
    expect(lerp(0, 10, 0.25)).toBeCloseTo(2.5, 10);
    expect(lerp(0, 10, 0.75)).toBeCloseTo(7.5, 10);
  });

  test('works with negative range', () => {
    expect(lerp(-10, 10, 0.5)).toBe(0);
    expect(lerp(-10, 10, 0)).toBe(-10);
    expect(lerp(-10, 10, 1)).toBe(10);
  });

  test('t > 1 extrapolates', () => {
    expect(lerp(0, 10, 2)).toBe(20);
  });

  test('t < 0 extrapolates backward', () => {
    expect(lerp(0, 10, -1)).toBe(-10);
  });
});
