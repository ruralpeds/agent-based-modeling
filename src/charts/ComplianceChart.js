/**
 * Line chart for hand-hygiene compliance rate over time [0, 1].
 * Reference line at WHO-5 ICU baseline (≈ 0.65).
 */
import { ChartBase, MARGIN } from './ChartBase.js';
import { scaleLinear }       from 'd3-scale';
import { line }              from 'd3-shape';
import { axisBottom, axisLeft } from 'd3-axis';
import { ABMDispatch }       from '../stores/dispatch.js';
import { HOSPITAL_PALETTE }  from '../constants.js';

const MAX_HISTORY  = 300;
const LINE_COLOR   = '#34d399';
const BASELINE_REF = 0.65;

export class ComplianceChart extends ChartBase {
  #data = [];  // [{tick, v}]

  constructor(container) {
    super(container);

    ABMDispatch.on('hospital:tick.compchart', (d) => {
      if (!d.stats) return;
      const s = typeof d.stats === 'string' ? JSON.parse(d.stats) : d.stats;
      if (s == null) return;
      this.#data.push({ tick: s.tick, v: s.hhComplianceRate ?? 0 });
      if (this.#data.length > MAX_HISTORY) this.#data.shift();
      this._draw();
    });

    ABMDispatch.on('hospital:reset.compchart', () => {
      this.#data = [];
      this._draw();
    });
  }

  _draw() {
    const { g, w, h } = this;
    if (this.#data.length < 2) return;

    const ticks = this.#data.map(d => d.tick);
    const x = scaleLinear().domain([ticks[0], ticks[ticks.length - 1]]).range([0, w]);
    const y = scaleLinear().domain([0, 1]).range([h, 0]);

    // WHO-5 reference line
    g.selectAll('.ref-line').data([0]).join('line')
      .attr('class', 'ref-line')
      .attr('x1', 0).attr('x2', w)
      .attr('y1', y(BASELINE_REF)).attr('y2', y(BASELINE_REF))
      .attr('stroke', '#fbbf24')
      .attr('stroke-width', 1)
      .attr('stroke-dasharray', '4 3')
      .attr('opacity', 0.6);

    // Reference label
    g.selectAll('.ref-label').data([0]).join('text')
      .attr('class', 'ref-label')
      .attr('x', w - 2).attr('y', y(BASELINE_REF) - 3)
      .attr('text-anchor', 'end')
      .attr('fill', '#fbbf24')
      .attr('font-size', 8)
      .text('WHO-5 65%');

    // Compliance line
    const lineGen = line().x(d => x(d.tick)).y(d => y(d.v));
    g.selectAll('.comp-line').data([this.#data]).join('path')
      .attr('class', 'comp-line')
      .attr('d', lineGen)
      .attr('fill', 'none')
      .attr('stroke', LINE_COLOR)
      .attr('stroke-width', 1.5);

    // Axes
    g.selectAll('.x-axis').data([0]).join('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${h})`)
      .call(axisBottom(x).ticks(4))
      .call(g => g.select('.domain').attr('stroke', HOSPITAL_PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', HOSPITAL_PALETTE.textMuted).attr('font-size', 9));

    g.selectAll('.y-axis').data([0]).join('g')
      .attr('class', 'y-axis')
      .call(axisLeft(y).ticks(4).tickFormat(d => (d * 100).toFixed(0) + '%'))
      .call(g => g.select('.domain').attr('stroke', HOSPITAL_PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', HOSPITAL_PALETTE.textMuted).attr('font-size', 9));

    // Title
    g.selectAll('.chart-title').data([0]).join('text')
      .attr('class', 'chart-title')
      .attr('x', w / 2).attr('y', -4)
      .attr('text-anchor', 'middle')
      .attr('fill', HOSPITAL_PALETTE.textMuted)
      .attr('font-size', 10)
      .text('Hand Hygiene Compliance');
  }
}
