import { select } from 'd3-selection';
import { SimBridge }   from '../bridge/simBridge.js';
import { paramStore }  from '../stores/paramStore.js';
import { uiStore }     from '../stores/uiStore.js';
import { ABMDispatch } from '../stores/dispatch.js';
import { DEFAULT_PARAMS, PARAM_LIMITS } from '../constants.js';

export function buildControlPanel(container) {
  const panel = select(container);

  // Title
  panel.append('div').attr('class', 'panel-title').text('ABM Simulator');

  // Playback controls
  const pb = panel.append('div').attr('class', 'playback-row');

  const btnRun = pb.append('button').attr('class', 'btn btn-primary').attr('id', 'btn-run').text('▶ Run');
  const btnStep = pb.append('button').attr('class', 'btn').attr('id', 'btn-step').text('⏭ Step');
  const btnReset = pb.append('button').attr('class', 'btn btn-danger').attr('id', 'btn-reset').text('↺ Reset');

  let running = false;
  btnRun.on('click', () => {
    running = !running;
    if (running) { SimBridge.start(); btnRun.text('⏸ Pause').attr('class', 'btn btn-warning'); }
    else          { SimBridge.pause(); btnRun.text('▶ Run').attr('class', 'btn btn-primary'); }
    uiStore.setRunning(running);
  });
  btnStep.on('click', () => SimBridge.step());
  btnReset.on('click', () => {
    running = false;
    btnRun.text('▶ Run').attr('class', 'btn btn-primary');
    uiStore.setRunning(false);
    SimBridge.reset();
  });

  // Speed slider
  const speedRow = panel.append('div').attr('class', 'param-row');
  speedRow.append('label').attr('class', 'param-label').text('Speed');
  const speedVal = speedRow.append('span').attr('class', 'param-value').text('1×');
  speedRow.append('input')
    .attr('type', 'range').attr('min', 0.25).attr('max', 10).attr('step', 0.25).attr('value', 1)
    .on('input', function() {
      const v = +this.value;
      speedVal.text(v + '×');
      SimBridge.setSpeed(v);
      uiStore.setSpeed(v);
    });

  // Rule selector
  panel.append('div').attr('class', 'section-title').text('Decision Rules');
  const rules = ['reactive', 'bounded', 'bdi'];
  const ruleLabels = { reactive: 'Reactive', bounded: 'Bounded', bdi: 'BDI' };
  const ruleRow = panel.append('div').attr('class', 'rule-row');
  rules.forEach(id => {
    ruleRow.append('button')
      .attr('class', 'btn rule-btn' + (id === 'reactive' ? ' active' : ''))
      .attr('data-rule', id)
      .text(ruleLabels[id])
      .on('click', function() {
        select('.rule-btn.active').classed('active', false);
        select(this).classed('active', true);
        paramStore.set('ruleSetId', id);
        SimBridge.setParams(paramStore.get());
      });
  });

  // Parameter sliders
  panel.append('div').attr('class', 'section-title').text('Parameters');

  const sliderDefs = [
    { key: 'preyCount',      label: 'Prey count',     fmt: v => Math.round(v) },
    { key: 'predatorCount',  label: 'Predator count',  fmt: v => Math.round(v) },
    { key: 'preySpeed',      label: 'Prey speed',      fmt: v => v.toFixed(2) },
    { key: 'predatorSpeed',  label: 'Pred speed',      fmt: v => v.toFixed(2) },
    { key: 'preyVision',     label: 'Prey vision',     fmt: v => v.toFixed(1) },
    { key: 'predatorVision', label: 'Pred vision',     fmt: v => v.toFixed(1) },
    { key: 'grassRegrowRate',label: 'Grass regrow',    fmt: v => v.toFixed(3) },
    { key: 'preyMetabolism', label: 'Prey metabolism', fmt: v => v.toFixed(2) },
    { key: 'predatorMetabolism', label: 'Pred metabolism', fmt: v => v.toFixed(2) },
  ];

  sliderDefs.forEach(({ key, label, fmt }) => {
    const limits = PARAM_LIMITS[key];
    if (!limits) return;
    const row = panel.append('div').attr('class', 'param-row');
    row.append('label').attr('class', 'param-label').text(label);
    const valSpan = row.append('span').attr('class', 'param-value').text(fmt(DEFAULT_PARAMS[key]));
    row.append('input')
      .attr('type', 'range')
      .attr('min', limits.min)
      .attr('max', limits.max)
      .attr('step', limits.step)
      .attr('value', DEFAULT_PARAMS[key])
      .on('change', function() {
        const v = paramStore.set(key, +this.value);
        valSpan.text(fmt(v));
        SimBridge.setParams(paramStore.get());
      });
  });
}
