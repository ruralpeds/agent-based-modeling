/**
 * Stacked area chart showing all 7 disease-state counts over simulation time.
 * Listens on hospital:tick and hospital:reset.
 */
import { ChartBase, MARGIN } from './ChartBase.js';
import { select }            from 'd3-selection';
import { scaleLinear }       from 'd3-scale';
import { area, stack }       from 'd3-shape';
import { axisBottom, axisLeft } from 'd3-axis';
import { ABMDispatch }       from '../stores/dispatch.js';
import { DISEASE_STATE_COLORS, DISEASE_STATE_NAMES, HOSPITAL_PALETTE } from '../constants.js';

const MAX_HISTORY = 300;

export class DiseaseStateChart extends ChartBase {
  #history = [];  // [{tick, counts:[u32×7]}]

  constructor(container) {
    super(container);

    ABMDispatch.on('hospital:tick.dschart', (d) => {
      if (!d.stats) return;
      const s = typeof d.stats === 'string' ? JSON.parse(d.stats) : d.stats;
      if (!s?.diseaseStateCounts) return;
      this.#history.push({ tick: s.tick, counts: s.diseaseStateCounts });
      if (this.#history.length > MAX_HISTORY) this.#history.shift();
      this._draw();
    });

    ABMDispatch.on('hospital:reset.dschart', () => {
      this.#history = [];
      this._draw();
    });
  }

  _draw() {
    const { g, w, h } = this;
    if (this.#history.length < 2) return;

    const keys = DISEASE_STATE_NAMES.map((_, i) => i);

    // Convert to [{tick, 0:n, 1:n, ...}]
    const data = this.#history.map(d => {
      const row = { tick: d.tick };
      keys.forEach(k => { row[k] = d.counts[k] ?? 0; });
      return row;
    });

    const stackGen = stack().keys(keys);
    const series   = stackGen(data);

    const ticks   = data.map(d => d.tick);
    const maxVal  = Math.max(10, ...data.map(d => keys.reduce((s, k) => s + (d[k] ?? 0), 0)));

    const x = scaleLinear().domain([ticks[0], ticks[ticks.length - 1]]).range([0, w]);
    const y = scaleLinear().domain([0, maxVal * 1.05]).range([h, 0]);

    const areaGen = area()
      .x(d => x(d.data.tick))
      .y0(d => y(d[0]))
      .y1(d => y(d[1]));

    g.selectAll('.ds-area').data(series, d => d.key).join('path')
      .attr('class', 'ds-area')
      .attr('d', areaGen)
      .attr('fill', d => DISEASE_STATE_COLORS[d.key])
      .attr('opacity', 0.75);

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

    // Chart title
    g.selectAll('.chart-title').data([0]).join('text')
      .attr('class', 'chart-title')
      .attr('x', w / 2).attr('y', -4)
      .attr('text-anchor', 'middle')
      .attr('fill', HOSPITAL_PALETTE.textMuted)
      .attr('font-size', 10)
      .text('Disease State Counts');
  }
}
