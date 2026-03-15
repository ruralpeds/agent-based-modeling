import { PALETTE, MAX_ENERGY } from '../constants.js';

/** Returns fill color for an agent based on energy and type */
export function energyColor(energy, isPrey) {
  const ratio = Math.max(0, Math.min(1, energy / MAX_ENERGY));
  if (isPrey) {
    // Green: low energy = dark, high energy = bright
    const g = Math.round(80 + ratio * 100);
    return `rgb(20,${g},40)`;
  } else {
    // Red
    const r = Math.round(120 + ratio * 120);
    return `rgb(${r},30,30)`;
  }
}
