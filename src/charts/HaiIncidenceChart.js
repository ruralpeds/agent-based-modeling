/**
 * Dual-series chart: bar (new HAI per tick) + line (cumulative HAI).
 * Listens on hospital:tick and hospital:reset.
 */
import { ChartBase, MARGIN } from './ChartBase.js';
import { scaleLinear }       from 'd3-scale';
import { line }              from 'd3-shape';
import { axisBottom, axisLeft, axisRight } from 'd3-axis';
import { ABMDispatch }       from '../stores/dispatch.js';
import { HOSPITAL_PALETTE }  from '../constants.js';

const MAX_HISTORY = 300;
const BAR_COLOR   = '#f97316';
const LINE_COLOR  = '#fb923c';

export class HaiIncidenceChart extends ChartBase {
  #newHai  = [];  // [{tick, v}]  — new HAI per tick
  #cumHai  = [];  // [{tick, v}]  — cumulative HAI

  constructor(container) {
    super(container);

    ABMDispatch.on('hospital:tick.haichart', (d) => {
      if (!d.stats) return;
      const s = typeof d.stats === 'string' ? JSON.parse(d.stats) : d.stats;
      if (s == null) return;
      this.#newHai.push({ tick: s.tick, v: s.newHaiThisTick ?? 0 });
      this.#cumHai.push({ tick: s.tick, v: s.cumulativeHai ?? 0 });
      if (this.#newHai.length > MAX_HISTORY) { this.#newHai.shift(); this.#cumHai.shift(); }
      this._draw();
    });

    ABMDispatch.on('hospital:reset.haichart', () => {
      this.#newHai = [];
      this.#cumHai = [];
      this._draw();
    });
  }

  _draw() {
    const { g, w, h } = this;
    if (this.#newHai.length < 2) return;

    const ticks  = this.#newHai.map(d => d.tick);
    const maxNew = Math.max(1, ...this.#newHai.map(d => d.v));
    const maxCum = Math.max(1, ...this.#cumHai.map(d => d.v));

    const x  = scaleLinear().domain([ticks[0], ticks[ticks.length - 1]]).range([0, w]);
    const yL = scaleLinear().domain([0, maxNew * 1.2]).range([h, 0]);   // new HAI (left)
    const yR = scaleLinear().domain([0, maxCum * 1.1]).range([h, 0]);   // cumulative (right)

    // Bars — new HAI per tick
    const barW = Math.max(1, w / this.#newHai.length - 1);
    g.selectAll('.hai-bar').data(this.#newHai).join('rect')
      .attr('class', 'hai-bar')
      .attr('x',    d => x(d.tick) - barW / 2)
      .attr('y',    d => yL(d.v))
      .attr('width',  barW)
      .attr('height', d => h - yL(d.v))
      .attr('fill', BAR_COLOR)
      .attr('opacity', 0.6);

    // Line — cumulative HAI
    const lineGen = line().x(d => x(d.tick)).y(d => yR(d.v));
    g.selectAll('.cum-line').data([this.#cumHai]).join('path')
      .attr('class', 'cum-line')
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

    g.selectAll('.y-axis-l').data([0]).join('g')
      .attr('class', 'y-axis-l')
      .call(axisLeft(yL).ticks(3))
      .call(g => g.select('.domain').attr('stroke', HOSPITAL_PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', BAR_COLOR).attr('font-size', 9));

    g.selectAll('.y-axis-r').data([0]).join('g')
      .attr('class', 'y-axis-r')
      .attr('transform', `translate(${w},0)`)
      .call(axisRight(yR).ticks(3))
      .call(g => g.select('.domain').attr('stroke', HOSPITAL_PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', LINE_COLOR).attr('font-size', 9));

    // Title
    g.selectAll('.chart-title').data([0]).join('text')
      .attr('class', 'chart-title')
      .attr('x', w / 2).attr('y', -4)
      .attr('text-anchor', 'middle')
      .attr('fill', HOSPITAL_PALETTE.textMuted)
      .attr('font-size', 10)
      .text('HAI Incidence (bars) & Cumulative (line)');
  }
}
