import { ChartBase, MARGIN } from './ChartBase.js';
import { select } from 'd3-selection';
import { scaleLinear } from 'd3-scale';
import { line, area }  from 'd3-shape';
import { axisBottom, axisLeft } from 'd3-axis';
import { ABMDispatch } from '../stores/dispatch.js';
import { PALETTE }     from '../constants.js';

const MAX_HISTORY = 500;

export class PopulationChart extends ChartBase {
  #preyData = [];
  #predData = [];

  constructor(container) {
    super(container);
    ABMDispatch.on('sim:tick.popchart', (d) => {
      if (!d.stats) return;
      const s = JSON.parse(d.stats);
      this.#preyData.push({ tick: s.tick, v: s.preyCount });
      this.#predData.push({ tick: s.tick, v: s.predCount });
      if (this.#preyData.length > MAX_HISTORY) {
        this.#preyData.shift();
        this.#predData.shift();
      }
      this._draw();
    });
    ABMDispatch.on('sim:reset.popchart', () => {
      this.#preyData = [];
      this.#predData = [];
      this._draw();
    });
  }

  _draw() {
    const { g, w, h } = this;
    const allData = [...this.#preyData, ...this.#predData];
    if (allData.length < 2) return;

    const ticks = this.#preyData.map(d => d.tick);
    const xDom = [ticks[0], ticks[ticks.length - 1]];
    const maxPop = Math.max(10, ...allData.map(d => d.v));

    const x = scaleLinear().domain(xDom).range([0, w]);
    const y = scaleLinear().domain([0, maxPop * 1.1]).range([h, 0]);

    const lineGen = (data) => line().x(d => x(d.tick)).y(d => y(d.v))(data);

    g.selectAll('.prey-line').data([this.#preyData]).join('path')
      .attr('class', 'prey-line')
      .attr('d', lineGen)
      .attr('fill', 'none')
      .attr('stroke', PALETTE.prey)
      .attr('stroke-width', 1.5);

    g.selectAll('.pred-line').data([this.#predData]).join('path')
      .attr('class', 'pred-line')
      .attr('d', lineGen)
      .attr('fill', 'none')
      .attr('stroke', PALETTE.predator)
      .attr('stroke-width', 1.5);

    g.selectAll('.x-axis').data([0]).join('g')
      .attr('class', 'x-axis')
      .attr('transform', `translate(0,${h})`)
      .call(axisBottom(x).ticks(4).tickFormat(d => d))
      .call(g => g.select('.domain').attr('stroke', PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', PALETTE.textMuted).attr('font-size', 10));

    g.selectAll('.y-axis').data([0]).join('g')
      .attr('class', 'y-axis')
      .call(axisLeft(y).ticks(4))
      .call(g => g.select('.domain').attr('stroke', PALETTE.textMuted))
      .call(g => g.selectAll('text').attr('fill', PALETTE.textMuted).attr('font-size', 10));
  }
}
