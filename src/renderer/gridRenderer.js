import { PALETTE } from '../constants.js';

/**
 * Draws grass grid as colored rectangles.
 * For performance, uses ImageData for dense grids.
 */
export function drawGrid(ctx, grassData, gridW, gridH, canvasW, canvasH, showGrid) {
  if (!grassData || grassData.length === 0) return;

  const cellW = canvasW / gridW;
  const cellH = canvasH / gridH;

  // Use simple fillRect approach for reasonable grid sizes
  for (let y = 0; y < gridH; y++) {
    for (let x = 0; x < gridW; x++) {
      const v = grassData[y * gridW + x] ?? 0;
      const g = Math.round(40 + v * 80);
      ctx.fillStyle = `rgb(10,${g},20)`;
      ctx.fillRect(x * cellW, y * cellH, Math.ceil(cellW), Math.ceil(cellH));
    }
  }

  if (showGrid) {
    ctx.strokeStyle = 'rgba(255,255,255,0.04)';
    ctx.lineWidth = 0.5;
    for (let x = 0; x <= gridW; x++) {
      ctx.beginPath();
      ctx.moveTo(x * cellW, 0);
      ctx.lineTo(x * cellW, canvasH);
      ctx.stroke();
    }
    for (let y = 0; y <= gridH; y++) {
      ctx.beginPath();
      ctx.moveTo(0, y * cellH);
      ctx.lineTo(canvasW, y * cellH);
      ctx.stroke();
    }
  }
}
