import { select }      from 'd3-selection';
import { ABMDispatch } from '../stores/dispatch.js';
import { HOSPITAL_PALETTE } from '../constants.js';

export function buildHospitalStatsBar(container) {
  const bar = select(container);
  bar.selectAll('*').remove();

  bar.append('span').attr('class', 'stat-item')
    .html('<b>Tick</b> <span id="hval-tick">0</span>');
  bar.append('span').attr('class', 'stat-item')
    .html('<b>Census</b> <span id="hval-census">0</span>');
  bar.append('span').attr('class', 'stat-item')
    .html('<b>ICU</b> <span id="hval-icu">0</span>');
  bar.append('span').attr('class', 'stat-item')
    .html('<b>HAI</b> <span id="hval-hai">0</span>');
  bar.append('span').attr('class', 'stat-item')
    .html('<b>HH%</b> <span id="hval-hh">—</span>');
  bar.append('span').attr('class', 'stat-item')
    .html('<b>SPRT</b> <span id="hval-sprt">—</span>');

  const outbreakBadge = bar.append('span')
    .attr('class', 'stat-item')
    .attr('id', 'hval-outbreak')
    .style('display', 'none')
    .style('color', '#ef4444')
    .style('font-weight', 'bold')
    .text('⚠ OUTBREAK');

  let lastTs    = performance.now();
  let frameCount = 0;

  ABMDispatch.on('hospital:tick.hstatsbar', (d) => {
    const s = d.stats ? (typeof d.stats === 'string' ? safeJson(d.stats) : d.stats) : null;

    select('#hval-tick').text(d.tick ?? 0);
    if (s) {
      select('#hval-census').text(s.census ?? 0);
      select('#hval-icu').text(s.icuOccupancy ?? 0);
      select('#hval-hai').text(s.cumulativeHai ?? 0);
      const hhPct = s.hhComplianceRate != null
        ? (s.hhComplianceRate * 100).toFixed(1) + '%'
        : '—';
      select('#hval-hh').text(hhPct);
      select('#hval-sprt').text(
        s.sprtLogLr != null ? s.sprtLogLr.toFixed(2) : '—'
      );
      outbreakBadge.style('display', s.outbreakSignal ? null : 'none');
    }

    frameCount++;
    const now = performance.now();
    if (now - lastTs >= 1000) {
      frameCount = 0;
      lastTs = now;
    }
  });

  ABMDispatch.on('hospital:reset.hstatsbar', () => {
    ['hval-tick','hval-census','hval-icu','hval-hai'].forEach(id =>
      select('#' + id).text('0')
    );
    select('#hval-hh').text('—');
    select('#hval-sprt').text('—');
    outbreakBadge.style('display', 'none');
  });
}

function safeJson(str) {
  try { return JSON.parse(str); } catch { return null; }
}
