import { FIELD, ACTION_CODES } from '../bridge/bufferLayout.js';
import { PREY_RADIUS, PRED_RADIUS, PALETTE } from '../constants.js';
import { energyColor } from '../utils/colorScales.js';

// Pre-built Path2D objects — reused every frame
const PREY_PATH = (() => {
  const p = new Path2D();
  p.arc(0, 0, PREY_RADIUS, 0, Math.PI * 2);
  return p;
})();

const PRED_PATH = (() => {
  const p = new Path2D();
  p.moveTo(0, -PRED_RADIUS * 1.4);
  p.lineTo( PRED_RADIUS * 1.2,  PRED_RADIUS);
  p.lineTo(-PRED_RADIUS * 1.2,  PRED_RADIUS);
  p.closePath();
  return p;
})();

/**
 * Draw all agents from SoA Float32Array buffer.
 * @param {CanvasRenderingContext2D} ctx
 * @param {Float32Array} agentBuf - SoA buffer from WASM
 * @param {number} count - agent count
 * @param {number} CW - canvas pixel width
 * @param {number} CH - canvas pixel height
 * @param {number|null} selectedId - highlighted agent id
 * @param {number} gridW - simulation grid width
 * @param {number} gridH - simulation grid height
 */
export function drawAgents(ctx, agentBuf, count, CW, CH, selectedId, gridW, gridH) {
  if (count === 0) return;

  const scaleX = CW / gridW;
  const scaleY = CH / gridH;

  const base_x   = FIELD.POS_X      * count;
  const base_y   = FIELD.POS_Y      * count;
  const base_e   = FIELD.ENERGY     * count;
  const base_t   = FIELD.AGENT_TYPE * count;
  const base_id  = FIELD.ID         * count;
  const base_act = FIELD.ACTION     * count;

  for (let i = 0; i < count; i++) {
    const px     = agentBuf[base_x + i]   * scaleX;
    const py     = agentBuf[base_y + i]   * scaleY;
    const energy = agentBuf[base_e + i];
    const isPrey = agentBuf[base_t + i] < 0.5;
    const id     = agentBuf[base_id + i];
    const action = agentBuf[base_act + i];

    ctx.save();
    ctx.translate(px, py);

    // Selection ring
    if (id === selectedId) {
      const r = (isPrey ? PREY_RADIUS : PRED_RADIUS) + 6;
      ctx.beginPath();
      ctx.arc(0, 0, r, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(251,191,36,0.35)';
      ctx.fill();
      ctx.strokeStyle = '#fbbf24';
      ctx.lineWidth = 1.5;
      ctx.stroke();
    }

    ctx.fillStyle   = energyColor(energy, isPrey);
    ctx.strokeStyle = isPrey ? PALETTE.preyStroke : PALETTE.predStroke;
    ctx.lineWidth   = 1;

    const path = isPrey ? PREY_PATH : PRED_PATH;
    ctx.fill(path);
    ctx.stroke(path);

    // Action indicator dot (flee = yellow, hunt = orange)
    if (action === ACTION_CODES.FLEE || action === ACTION_CODES.HUNT) {
      const dotR = isPrey ? PREY_RADIUS : PRED_RADIUS;
      ctx.beginPath();
      ctx.arc(dotR, -dotR, 2.5, 0, Math.PI * 2);
      ctx.fillStyle = action === ACTION_CODES.FLEE ? '#fbbf24' : '#f97316';
      ctx.fill();
    }

    ctx.restore();
  }
}
