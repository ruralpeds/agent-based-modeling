import { select }     from 'd3-selection';
import { HospitalBridge } from '../bridge/hospitalBridge.js';
import { ABMDispatch }    from '../stores/dispatch.js';
import {
  DEFAULT_HOSPITAL_PARAMS,
  HOSPITAL_PARAM_LIMITS,
  HOSPITAL_PALETTE,
} from '../constants.js';

export function buildHospitalControlPanel(container) {
  const panel = select(container);
  panel.selectAll('*').remove();

  // Title
  panel.append('div').attr('class', 'panel-title').text('Hospital Sim');

  // Playback controls
  const pb = panel.append('div').attr('class', 'playback-row');

  const btnRun   = pb.append('button').attr('class', 'btn btn-primary').attr('id', 'hbtn-run').text('▶ Run');
  const btnStep  = pb.append('button').attr('class', 'btn').attr('id', 'hbtn-step').text('⏭ Step');
  const btnReset = pb.append('button').attr('class', 'btn btn-danger').attr('id', 'hbtn-reset').text('↺ Reset');

  let running = false;
  btnRun.on('click', () => {
    running = !running;
    if (running) {
      HospitalBridge.start();
      btnRun.text('⏸ Pause').attr('class', 'btn btn-warning');
    } else {
      HospitalBridge.pause();
      btnRun.text('▶ Run').attr('class', 'btn btn-primary');
    }
  });
  btnStep.on('click', () => HospitalBridge.step());
  btnReset.on('click', () => {
    running = false;
    btnRun.text('▶ Run').attr('class', 'btn btn-primary');
    HospitalBridge.reset();
  });

  // Speed slider
  const speedRow = panel.append('div').attr('class', 'param-row');
  speedRow.append('label').attr('class', 'param-label').text('Speed');
  const speedVal = speedRow.append('span').attr('class', 'param-value').text('1×');
  speedRow.append('input')
    .attr('type', 'range').attr('min', 0.25).attr('max', 20).attr('step', 0.25).attr('value', 1)
    .on('input', function () {
      const v = +this.value;
      speedVal.text(v + '×');
      HospitalBridge.setSpeed(v);
    });

  // ── Parameters ──────────────────────────────────────────────────────────────
  panel.append('div').attr('class', 'section-title').text('Parameters');

  // Live params object — copy defaults
  const params = structuredClone(DEFAULT_HOSPITAL_PARAMS);

  const sliderDefs = [
    {
      key: 'baseArrivalRate',
      label: 'Arrival rate (pt/h)',
      fmt: v => v.toFixed(1),
      get: () => params.baseArrivalRate,
      set: v => { params.baseArrivalRate = v; },
    },
    {
      key: 'admissionColonizationRate',
      label: 'Admission colonization',
      fmt: v => (v * 100).toFixed(0) + '%',
      get: () => params.admissionColonizationRate,
      set: v => { params.admissionColonizationRate = v; },
    },
    {
      key: 'dtmcBeta',
      label: 'DTMC β (C→I)',
      fmt: v => v.toFixed(2),
      get: () => params.dtmcBeta,
      set: v => { params.dtmcBeta = v; },
    },
    {
      key: 'dtmcGamma',
      label: 'DTMC γ (C→T)',
      fmt: v => v.toFixed(2),
      get: () => params.dtmcGamma,
      set: v => { params.dtmcGamma = v; },
    },
    {
      key: 'dtmcPsi',
      label: 'DTMC ψ (T→R)',
      fmt: v => v.toFixed(2),
      get: () => params.dtmcPsi,
      set: v => { params.dtmcPsi = v; },
    },
    {
      key: 'meanDispenserDist',
      label: 'Dispenser dist (m)',
      fmt: v => v.toFixed(1),
      get: () => params.meanDispenserDist,
      set: v => { params.meanDispenserDist = v; },
    },
    {
      key: 'nNurses',
      label: 'Nurses',
      fmt: v => Math.round(v),
      get: () => params.nNurses,
      set: v => { params.nNurses = Math.round(v); },
    },
  ];

  sliderDefs.forEach(({ key, label, fmt, get, set }) => {
    const limits = HOSPITAL_PARAM_LIMITS[key];
    if (!limits) return;
    const row = panel.append('div').attr('class', 'param-row');
    row.append('label').attr('class', 'param-label').text(label);
    const valSpan = row.append('span').attr('class', 'param-value').text(fmt(get()));
    row.append('input')
      .attr('type', 'range')
      .attr('min', limits.min)
      .attr('max', limits.max)
      .attr('step', limits.step)
      .attr('value', get())
      .on('change', function () {
        set(+this.value);
        valSpan.text(fmt(get()));
        HospitalBridge.setParams(params);
      });
  });

  // ── Audit button ─────────────────────────────────────────────────────────────
  panel.append('div').attr('class', 'section-title').text('Interventions');

  panel.append('button')
    .attr('class', 'btn')
    .attr('id', 'hbtn-audit')
    .style('width', '100%')
    .style('margin-top', '4px')
    .text('📋 Trigger HH Audit')
    .on('click', () => {
      // Boost compliance via params temporarily — raise dispenser proximity,
      // which the nurse model translates to higher compliance.
      const auditParams = structuredClone(params);
      auditParams.meanDispenserDist = Math.max(0.5, params.meanDispenserDist * 0.5);
      HospitalBridge.setParams(auditParams);
    });

  // Sync running state from dispatch
  ABMDispatch.on('hospital:start.hcp', () => {
    running = true;
    btnRun.text('⏸ Pause').attr('class', 'btn btn-warning');
  });
  ABMDispatch.on('hospital:pause.hcp', () => {
    running = false;
    btnRun.text('▶ Run').attr('class', 'btn btn-primary');
  });
  ABMDispatch.on('hospital:reset.hcp', () => {
    running = false;
    btnRun.text('▶ Run').attr('class', 'btn btn-primary');
  });

  // Outbreak alert
  ABMDispatch.on('hospital:outbreak.hcp', (d) => {
    const banner = panel.select('#h-outbreak-banner');
    if (banner.empty()) {
      panel.insert('div', '.panel-title + *')
        .attr('id', 'h-outbreak-banner')
        .style('background', '#dc2626')
        .style('color', '#fff')
        .style('padding', '4px 8px')
        .style('border-radius', '4px')
        .style('font-size', '11px')
        .style('margin-bottom', '6px')
        .text('⚠ OUTBREAK DETECTED — tick ' + (d.tick ?? '?'));
    } else {
      banner.text('⚠ OUTBREAK DETECTED — tick ' + (d.tick ?? '?'));
    }
    // Auto-dismiss after 5 s
    setTimeout(() => panel.select('#h-outbreak-banner').remove(), 5000);
  });
}
