import { select }                  from 'd3-selection';
import { buildStatsBar }           from './ui/statsBar.js';
import { buildControlPanel }       from './ui/controlPanel.js';
import { buildHospitalStatsBar }   from './ui/hospitalStatsBar.js';
import { buildHospitalControlPanel } from './ui/hospitalControlPanel.js';
import { CanvasRenderer }          from './renderer/canvasRenderer.js';
import { HospitalRenderer }        from './renderer/hospitalRenderer.js';
import { PopulationChart }         from './charts/PopulationChart.js';
import { DiseaseStateChart }       from './charts/DiseaseStateChart.js';
import { HaiIncidenceChart }       from './charts/HaiIncidenceChart.js';
import { ComplianceChart }         from './charts/ComplianceChart.js';
import { SprtChart }               from './charts/SprtChart.js';
import { SimBridge }               from './bridge/simBridge.js';
import { HospitalBridge }          from './bridge/hospitalBridge.js';
import { ABMDispatch }             from './stores/dispatch.js';
import { logger }                  from './utils/logger.js';

// ─── DOM scaffolding ──────────────────────────────────────────────────────────

function buildDualLayout() {
  const app = select('#app');

  // Mode toggle bar
  const modeBar = app.append('div').attr('class', 'mode-bar');
  modeBar.append('button')
    .attr('class', 'btn active').attr('id', 'mode-btn-abm')
    .text('🐺 Predator-Prey ABM');
  modeBar.append('button')
    .attr('class', 'btn').attr('id', 'mode-btn-hospital')
    .text('🏥 Hospital Simulation');

  // ── ABM layer ────────────────────────────────────────────────────────────────
  const abmStatsBar = app.append('div')
    .attr('class', 'stats-bar').attr('id', 'stats-bar-abm');

  const abmMain = app.append('div').attr('class', 'main-row').attr('id', 'abm-main');

  const abmSidebar = abmMain.append('div')
    .attr('class', 'sidebar').attr('id', 'sidebar-abm');

  const abmViewport = abmMain.append('div')
    .attr('class', 'viewport').attr('id', 'viewport-abm');
  const abmCanvas = abmViewport.append('canvas')
    .attr('id', 'sim-canvas').attr('class', 'sim-canvas');

  const abmChartPanel = abmMain.append('div')
    .attr('class', 'chart-panel').attr('id', 'chart-panel-abm');

  // ── Hospital layer ───────────────────────────────────────────────────────────
  const hospStatsBar = app.append('div')
    .attr('class', 'stats-bar').attr('id', 'stats-bar-hospital')
    .style('display', 'none');

  const hospMain = app.append('div')
    .attr('class', 'main-row').attr('id', 'hospital-main')
    .style('display', 'none');

  const hospSidebar = hospMain.append('div')
    .attr('class', 'sidebar').attr('id', 'sidebar-hospital');

  const hospViewport = hospMain.append('div')
    .attr('class', 'viewport').attr('id', 'viewport-hospital');
  const hospCanvas = hospViewport.append('canvas')
    .attr('id', 'hospital-canvas').attr('class', 'sim-canvas');

  // Hospital chart panel — 4 sub-panels stacked
  const hospChartPanel = hospMain.append('div')
    .attr('class', 'chart-panel hospital-chart-panel')
    .attr('id', 'chart-panel-hospital');

  const dsPanel   = hospChartPanel.append('div').attr('class', 'hospital-sub-chart').node();
  const haiPanel  = hospChartPanel.append('div').attr('class', 'hospital-sub-chart').node();
  const compPanel = hospChartPanel.append('div').attr('class', 'hospital-sub-chart').node();
  const sprtPanel = hospChartPanel.append('div').attr('class', 'hospital-sub-chart').node();

  return {
    abmStatsBar:    abmStatsBar.node(),
    abmSidebar:     abmSidebar.node(),
    abmCanvas:      abmCanvas.node(),
    abmChartPanel:  abmChartPanel.node(),
    hospStatsBar:   hospStatsBar.node(),
    hospSidebar:    hospSidebar.node(),
    hospCanvas:     hospCanvas.node(),
    hospChartPanel: hospChartPanel.node(),
    dsPanel, haiPanel, compPanel, sprtPanel,
  };
}

// ─── Main ─────────────────────────────────────────────────────────────────────

let _hospInitDone = false;

async function main() {
  const refs = buildDualLayout();

  // Stats bars + control panels
  buildStatsBar(refs.abmStatsBar);
  buildControlPanel(refs.abmSidebar);
  buildHospitalStatsBar(refs.hospStatsBar);
  buildHospitalControlPanel(refs.hospSidebar);

  // Renderers
  new CanvasRenderer(refs.abmCanvas);
  refs.abmCanvas.addEventListener('click', () => {
    ABMDispatch.call('agent:select', refs.abmCanvas, { id: null });
  });
  new HospitalRenderer(refs.hospCanvas);

  // Charts — ABM
  new PopulationChart(refs.abmChartPanel);

  // Charts — Hospital
  new DiseaseStateChart(refs.dsPanel);
  new HaiIncidenceChart(refs.haiPanel);
  new ComplianceChart(refs.compPanel);
  new SprtChart(refs.sprtPanel);

  // Mode toggle
  select('#mode-btn-abm').on('click', () => {
    select('#abm-main').style('display', null);
    select('#stats-bar-abm').style('display', null);
    select('#hospital-main').style('display', 'none');
    select('#stats-bar-hospital').style('display', 'none');
    select('#mode-btn-abm').classed('active', true);
    select('#mode-btn-hospital').classed('active', false);
  });

  select('#mode-btn-hospital').on('click', async () => {
    select('#abm-main').style('display', 'none');
    select('#stats-bar-abm').style('display', 'none');
    select('#hospital-main').style('display', null);
    select('#stats-bar-hospital').style('display', null);
    select('#mode-btn-abm').classed('active', false);
    select('#mode-btn-hospital').classed('active', true);

    // Lazy-init hospital engine on first mode switch
    if (!_hospInitDone) {
      await _initHospital();
    }
  });

  // Init ABM engine
  try {
    await SimBridge.init();
    logger.info('main', 'SimBridge initialized — WASM ready');
    SimBridge.step();
  } catch (err) {
    logger.error('main', 'SimBridge init failed:', err);
    _showError('Failed to initialize ABM engine: ' + err.message);
  }
}

async function _initHospital() {
  try {
    await HospitalBridge.init();
    logger.info('main', 'HospitalBridge initialized — WASM ready');
    HospitalBridge.step();
    _hospInitDone = true;
  } catch (err) {
    logger.error('main', 'HospitalBridge init failed:', err);
    _showError('Failed to initialize Hospital engine: ' + err.message);
  }
}

function _showError(msg) {
  const app = document.getElementById('app');
  const div = document.createElement('div');
  div.style.cssText = 'color:#ef4444;padding:2rem;font-family:monospace;position:fixed;top:0;left:0;z-index:999;background:#0f172a';
  div.textContent = msg;
  app?.prepend(div);
}

main().catch(err => {
  logger.error('main', 'Fatal unhandled error:', err);
});
