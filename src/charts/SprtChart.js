/**
 * SPRT log-likelihood ratio chart with upper/lower decision boundaries and
 * outbreak signal markers.
 *
 * SPRT boundaries:
 *   Upper (H1): ln((1 − β) / α) ≈  ln(19)  ≈  2.94  (α=0.05, β=0.05)
 *   Lower (H0): ln(β / (1 − α)) ≈  ln(0.053) ≈ −2.94
 */
import { ChartBase, MARGIN } from './ChartBase.js';
import { scaleLinear }       from 'd3-scale';
import { line }              from 'd3-shape';
import { axisBottom, axisLeft } from 'd3-axis';
import { ABMDispatch }       from '../stores/dispatch.js';
import { HOSPITAL_PALETTE }  from '../constants.js';

const MAX_HISTORY     = 300;
const UPPER_BOUNDARY  =  2.94;
const LOWER_BOUNDARY  = -2.94;
const LINE_COLOR      = '#a78bfa';
const UPPER_COLOR     = '#ef4444';
const LOWER_COLOR     = '#22c55e';

export class SprtChart extends ChartBase {
  #data     = [];  // [{tick, logLr, signal}]

  constructor(container) {
    super(container);

    ABMDispatch.on('hospital:tick.sprtchart', (d) => {
      if (!d.stats) return;
      const s = typeof d.stats === 'string' ? JSON.parse(d.stats) : d.stats;
      if (s == null) return;
      this.#data.push({
        tick:   s.tick,
        logLr:  s.sprtLogLr   ?? 0,
        signal: s.outbreakSignal ?? false,
      });
      if (this.#data.length > MAX_HISTORY) this.#data.shift();
      this._draw();
    });

    ABMDispatch.on('hospital:reset.sprtchart', () => {
      this.#data = [];
      this._draw();
    });
  }

  _draw() {
    const { g, w, h } = this;
    if (this.#data.length < 2) return;

    const ticks  = this.#data.map(d => d.tick);
    const allLr  = this.#data.map(d => d.logLr);
    const minLr  = Math.min(LOWER_BOUNDARY * 1.2, ...allLr);
    const maxLr  = Math.max(UPPER_BOUNDARY * 1.2, ...allLr);

    const x = scaleLinear().domain([ticks[0], ticks[ticks.length - 1]]).range([0, w]);
    const y = scaleLinear().domain([minLr, maxLr]).range([h, 0]);

    // Zero reference
    g.selectAll('.zero-line').data([0]).join('line')
      .attr('class', 'zero-line')
      .attr('x1', 0).attr('x2', w)
      .attr('y1', y(0)).attr('y2', y(0))
      .attr('stroke', HOSPITAL_PALETTE.textMuted)
      .attr('stroke-width', 0.5)
      .attr('stroke-dasharray', '2 3');

    // Upper boundary (H1 — outbreak)
    g.selectAll('.upper-bound').data([0]).join('line')
      .attr('class', 'upper-bound')
      .attr('x1', 0).attr('x2', w)
      .attr('y1', y(UPPER_BOUNDARY)).attr('y2', y(UPPER_BOUNDARY))
      .attr('stroke', UPPER_COLOR)
      .attr('stroke-width', 1)
      .attr('stroke-dasharray', '4 2');

    g.selectAll('.upper-label').data([0]).join('text')
      .attr('class', 'upper-label')
      .attr('x', 2).attr('y', y(UPPER_BOUNDARY) - 3)
      .attr('fill', UPPER_COLOR).attr('font-size', 8)
      .text('H₁ outbreak');

    // Lower boundary (H0 — no outbreak)
    g.selectAll('.lower-bound').data([0]).join('line')
      .attr('class', 'lower-bound')
      .attr('x1', 0).attr('x2', w)
      .attr('y1', y(LOWER_BOUNDARY)).attr('y2', y(LOWER_BOUNDARY))
      .attr('stroke', LOWER_COLOR)
      .attr('stroke-width', 1)
      .attr('stroke-dasharray', '4 2');

    g.selectAll('.lower-label').data([0]).join('text')
      .attr('class', 'lower-label')
      .attr('x', 2).attr('y', y(LOWER_BOUNDARY) + 10)
      .attr('fill', LOWER_COLOR).attr('font-size', 8)
      .text('H₀ no outbreak');

    // Log-LR line
    const lineGen = line().x(d => x(d.tick)).y(d => y(d.logLr));
    g.selectAll('.sprt-line').data([this.#data]).join('path')
      .attr('class', 'sprt-line')
      .attr('d', lineGen)
      .attr('fill', 'none')
      .attr('stroke', LINE_COLOR)
      .attr('stroke-width', 1.5);

    // Outbreak signal markers (circles at signal ticks)
    const signals = this.#data.filter(d => d.signal);
    g.selectAll('.signal-dot').data(signals, d => d.tick).join('circle')
      .attr('class', 'signal-dot')
      .attr('cx', d => x(d.tick))
      .attr('cy', d => y(d.logLr))
      .attr('r', 4)
      .attr('fill', UPPER_COLOR)
      .attr('stroke', '#fff')
      .attr('stroke-width', 1);

    // Axes
    g.selectAll('.x-axis').data([0]).join('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${h})`)
      .call(axisBottom(x).ticks(4))
      .call(g => g.select('.domain').attr('stroke', HOSPITAL_PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', HOSPITAL_PALETTE.textMuted).attr('font-size', 9));

    g.selectAll('.y-axis').data([0]).join('g')
      .attr('class', 'y-axis')
      .call(axisLeft(y).ticks(4))
      .call(g => g.select('.domain').attr('stroke', HOSPITAL_PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', HOSPITAL_PALETTE.textMuted).attr('font-size', 9));

    // Title
    g.selectAll('.chart-title').data([0]).join('text')
      .attr('class', 'chart-title')
      .attr('x', w / 2).attr('y', -4)
      .attr('text-anchor', 'middle')
      .attr('fill', HOSPITAL_PALETTE.textMuted)
      .attr('font-size', 10)
      .text('SPRT Log-Likelihood Ratio');
  }
}
