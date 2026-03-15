import { describe, test, expect } from 'vitest';
import { energyColor } from '@/utils/colorScales.js';
import { MAX_ENERGY } from '@/constants.js';

// energyColor(energy, isPrey):
//   ratio = clamp(energy / MAX_ENERGY, 0, 1)
//   prey:      rgb(20, round(80 + ratio*100), 40)
//   predator:  rgb(round(120 + ratio*120), 30, 30)

describe('energyColor — prey', () => {
  test('zero energy produces darkest green', () => {
    // ratio=0, g = round(80 + 0) = 80
    expect(energyColor(0, true)).toBe('rgb(20,80,40)');
  });

  test('full energy produces brightest green', () => {
    // ratio=1, g = round(80 + 100) = 180
    expect(energyColor(MAX_ENERGY, true)).toBe('rgb(20,180,40)');
  });

  test('half energy produces mid-range green', () => {
    // ratio=0.5, g = round(80 + 50) = 130
    expect(energyColor(MAX_ENERGY / 2, true)).toBe('rgb(20,130,40)');
  });

  test('negative energy clamps to zero energy', () => {
    expect(energyColor(-10, true)).toBe(energyColor(0, true));
  });

  test('energy above max clamps to full energy', () => {
    expect(energyColor(MAX_ENERGY + 50, true)).toBe(energyColor(MAX_ENERGY, true));
  });

  test('always returns rgb(...) format', () => {
    expect(energyColor(50, true)).toMatch(/^rgb\(\d+,\d+,\d+\)$/);
  });

  test('red channel is always 20 for prey', () => {
    expect(energyColor(0, true)).toMatch(/^rgb\(20,/);
    expect(energyColor(100, true)).toMatch(/^rgb\(20,/);
    expect(energyColor(200, true)).toMatch(/^rgb\(20,/);
  });

  test('blue channel is always 40 for prey', () => {
    expect(energyColor(0, true)).toMatch(/,40\)$/);
    expect(energyColor(100, true)).toMatch(/,40\)$/);
  });
});

describe('energyColor — predator', () => {
  test('zero energy produces darkest red', () => {
    // ratio=0, r = round(120 + 0) = 120
    expect(energyColor(0, false)).toBe('rgb(120,30,30)');
  });

  test('full energy produces brightest red', () => {
    // ratio=1, r = round(120 + 120) = 240
    expect(energyColor(MAX_ENERGY, false)).toBe('rgb(240,30,30)');
  });

  test('half energy produces mid-range red', () => {
    // ratio=0.5, r = round(120 + 60) = 180
    expect(energyColor(MAX_ENERGY / 2, false)).toBe('rgb(180,30,30)');
  });

  test('negative energy clamps to zero energy', () => {
    expect(energyColor(-1, false)).toBe(energyColor(0, false));
  });

  test('energy above max clamps to full energy', () => {
    expect(energyColor(999, false)).toBe(energyColor(MAX_ENERGY, false));
  });

  test('always returns rgb(...) format', () => {
    expect(energyColor(100, false)).toMatch(/^rgb\(\d+,\d+,\d+\)$/);
  });

  test('green and blue channels are always 30 for predator', () => {
    expect(energyColor(0, false)).toMatch(/,30,30\)$/);
    expect(energyColor(100, false)).toMatch(/,30,30\)$/);
    expect(energyColor(200, false)).toMatch(/,30,30\)$/);
  });
});

describe('energyColor — type distinction', () => {
  test('prey and predator return different colors at same energy', () => {
    expect(energyColor(100, true)).not.toBe(energyColor(100, false));
    expect(energyColor(0, true)).not.toBe(energyColor(0, false));
  });
});
