import { select } from 'd3-selection';

export const MARGIN = { top: 12, right: 16, bottom: 30, left: 40 };

export class ChartBase {
  constructor(container) {
    this.container = container;
    this.svg = null;
    this.g   = null;
    this.w   = 0;
    this.h   = 0;
    this._init();
    const ro = new ResizeObserver(() => this._resize());
    ro.observe(container);
  }

  _init() {
    this.svg = select(this.container).append('svg')
      .attr('width',  '100%')
      .attr('height', '100%')
      .style('overflow', 'visible');
    this.g = this.svg.append('g')
      .attr('transform', `translate(${MARGIN.left},${MARGIN.top})`);
    this._resize();
  }

  _resize() {
    const rect = this.container.getBoundingClientRect();
    this.w = Math.max(10, rect.width  - MARGIN.left - MARGIN.right);
    this.h = Math.max(10, rect.height - MARGIN.top  - MARGIN.bottom);
    this._draw();
  }

  _draw() { /* Override in subclass */ }
}
