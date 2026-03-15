import { select, create } from 'd3-selection';

/**
 * Build the full application DOM using D3.
 * Returns references to key containers.
 */
export function buildLayout() {
  const app = select('#app');

  // Top stats bar
  const statsBar = app.append('div')
    .attr('class', 'stats-bar')
    .attr('id', 'stats-bar');

  // Main content row
  const main = app.append('div')
    .attr('class', 'main-row');

  // Left sidebar — controls
  const sidebar = main.append('div')
    .attr('class', 'sidebar')
    .attr('id', 'sidebar');

  // Viewport — canvas
  const viewport = main.append('div')
    .attr('class', 'viewport')
    .attr('id', 'viewport');

  const canvas = viewport.append('canvas')
    .attr('id', 'sim-canvas')
    .attr('class', 'sim-canvas');

  // Right panel — charts
  const chartPanel = main.append('div')
    .attr('class', 'chart-panel')
    .attr('id', 'chart-panel');

  return {
    statsBar:   statsBar.node(),
    sidebar:    sidebar.node(),
    viewport:   viewport.node(),
    canvas:     canvas.node(),
    chartPanel: chartPanel.node(),
  };
}
