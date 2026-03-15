import { select } from 'd3-selection';
import { ABMDispatch } from '../stores/dispatch.js';
import { fmtTick, fmtCount } from '../utils/formatters.js';
import { PALETTE } from '../constants.js';

export function buildStatsBar(container) {
  const bar = select(container);

  const tick    = bar.append('span').attr('class', 'stat-item').html('<b>Tick</b> <span id="val-tick">0</span>');
  const prey    = bar.append('span').attr('class', 'stat-item').html('<b style="color:#22c55e">Prey</b> <span id="val-prey">0</span>');
  const pred    = bar.append('span').attr('class', 'stat-item').html('<b style="color:#ef4444">Pred</b> <span id="val-pred">0</span>');
  const fps     = bar.append('span').attr('class', 'stat-item').html('<b>FPS</b> <span id="val-fps">—</span>');

  let lastTs = performance.now();
  let frameCount = 0;

  ABMDispatch.on('sim:tick.statsbar', (d) => {
    let stats = null;
    if (d.stats) { try { stats = JSON.parse(d.stats); } catch { /* malformed — skip */ } }
    select('#val-tick').text(fmtTick(d.tick));
    if (stats) {
      select('#val-prey').text(fmtCount(stats.preyCount));
      select('#val-pred').text(fmtCount(stats.predCount));
    }
    frameCount++;
    const now = performance.now();
    if (now - lastTs >= 1000) {
      select('#val-fps').text(frameCount.toFixed(0));
      frameCount = 0;
      lastTs = now;
    }
  });
}
