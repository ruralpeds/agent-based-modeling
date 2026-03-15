import { select } from 'd3-selection';
import { drawAgents } from './agentRenderer.js';
import { drawGrid }   from './gridRenderer.js';
import { PALETTE, GRID_W, GRID_H } from '../constants.js';
import { ABMDispatch } from '../stores/dispatch.js';
import { uiStore }     from '../stores/uiStore.js';

export class CanvasRenderer {
  #canvas = null;
  #ctx    = null;
  #lastBuf = null;
  #lastCount = 0;

  constructor(canvasEl) {
    this.#canvas = canvasEl;
    this.#ctx    = canvasEl.getContext('2d');
    this.resize();

    ABMDispatch.on('sim:tick.renderer', (d) => {
      this.#lastBuf   = d.agentBuf;
      this.#lastCount = d.count;
      this.render(d.agentBuf, d.count);
    });

    window.addEventListener('resize', () => this.resize());
  }

  resize() {
    const parent = this.#canvas.parentElement;
    if (!parent) return;
    const { width, height } = parent.getBoundingClientRect();
    this.#canvas.width  = width;
    this.#canvas.height = height;
    if (this.#lastBuf) {
      this.render(this.#lastBuf, this.#lastCount);
    }
  }

  render(agentBuf, count) {
    const ctx = this.#ctx;
    const w = this.#canvas.width;
    const h = this.#canvas.height;

    // Clear
    ctx.fillStyle = PALETTE.background;
    ctx.fillRect(0, 0, w, h);

    // Draw grass background (simple gradient for now — real grass from WASM grid)
    const grd = ctx.createLinearGradient(0, 0, 0, h);
    grd.addColorStop(0, '#0d2318');
    grd.addColorStop(1, '#0a1f14');
    ctx.fillStyle = grd;
    ctx.fillRect(0, 0, w, h);

    // Draw agents
    const selectedId = uiStore.get('selectedAgentId');
    drawAgents(ctx, agentBuf, count, w, h, selectedId, GRID_W, GRID_H);
  }

  /** Hit test: returns agent index (in buffer) at canvas pixel (px, py), or -1 */
  hitTest(px, py, agentBuf, count) {
    if (!agentBuf || count === 0) return -1;
    const w = this.#canvas.width;
    const h = this.#canvas.height;
    const scaleX = w / GRID_W;
    const scaleY = h / GRID_H;
    const base_x = 0 * count;
    const base_y = 1 * count;
    let best = -1, bestD = 10 * 10;
    for (let i = 0; i < count; i++) {
      const ax = agentBuf[base_x + i] * scaleX;
      const ay = agentBuf[base_y + i] * scaleY;
      const d  = (ax - px) ** 2 + (ay - py) ** 2;
      if (d < bestD) { bestD = d; best = i; }
    }
    return best;
  }

  get canvas() { return this.#canvas; }
}
