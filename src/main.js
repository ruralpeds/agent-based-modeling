import { buildLayout }      from './ui/layout.js';
import { buildStatsBar }    from './ui/statsBar.js';
import { buildControlPanel } from './ui/controlPanel.js';
import { CanvasRenderer }   from './renderer/canvasRenderer.js';
import { PopulationChart }  from './charts/PopulationChart.js';
import { SimBridge }        from './bridge/simBridge.js';
import { ABMDispatch }      from './stores/dispatch.js';
import { logger }           from './utils/logger.js';

async function main() {
  // Build DOM structure via D3
  const { statsBar, sidebar, canvas, chartPanel } = buildLayout();

  // Initialize stats bar and control panel
  buildStatsBar(statsBar);
  buildControlPanel(sidebar);

  // Initialize canvas renderer
  const renderer = new CanvasRenderer(canvas);

  // Click to select agent
  canvas.addEventListener('click', (e) => {
    const rect = canvas.getBoundingClientRect();
    const px = e.clientX - rect.left;
    const py = e.clientY - rect.top;
    // Access last known buffer from renderer (via a stored reference)
    // For simplicity, dispatch deselect on canvas click (agent selection is a stretch goal)
    ABMDispatch.call('agent:select', canvas, { id: null });
  });

  // Initialize population chart
  new PopulationChart(chartPanel);

  // Initialize WASM + Worker
  try {
    await SimBridge.init();
    logger.info('main', 'SimBridge initialized — WASM ready');
    // Kick off one step so the canvas isn't blank
    SimBridge.step();
  } catch (err) {
    logger.error('main', 'SimBridge init failed:', err);
    // Show error in UI
    const app = document.getElementById('app');
    const errDiv = document.createElement('div');
    errDiv.style.cssText = 'color:#ef4444;padding:2rem;font-family:monospace';
    errDiv.textContent = 'Failed to initialize WASM engine: ' + err.message;
    app?.prepend(errDiv);
  }
}

main().catch(err => {
  console.error('[ABM] Fatal error:', err);
});
