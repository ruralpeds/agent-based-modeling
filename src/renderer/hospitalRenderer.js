import { ABMDispatch }      from '../stores/dispatch.js';
import { HOSPITAL_PALETTE, DISEASE_STATE_COLORS, DEFAULT_HOSPITAL_PARAMS } from '../constants.js';
import { PATIENT_FIELD, MAX_PATIENTS } from '../bridge/hospitalBufferLayout.js';

// ─── Floor plan constants ─────────────────────────────────────────────────────

const ICU_COLS   = 5;
const WARD_COLS  = 10;
const BED_PAD    = 4;   // px between beds
const BED_LABEL_H = 14; // px for unit label row

// ─── HospitalRenderer ─────────────────────────────────────────────────────────

export class HospitalRenderer {
  #canvas  = null;
  #ctx     = null;
  #lastBuf = null;
  #lastN   = 0;
  #params  = DEFAULT_HOSPITAL_PARAMS;

  constructor(canvasEl) {
    this.#canvas = canvasEl;
    this.#ctx    = canvasEl.getContext('2d');
    this.resize();

    ABMDispatch.on('hospital:tick.hrenderer', (d) => {
      this.#lastBuf = d.patientBuf;
      this.#lastN   = d.count;
      if (d.stats) this.#params = d.stats;
      this.render(d.patientBuf, d.count);
    });

    ABMDispatch.on('hospital:reset.hrenderer', () => {
      this.#lastBuf = null;
      this.#lastN   = 0;
      this._clear();
    });

    window.addEventListener('resize', () => this.resize());
  }

  resize() {
    const parent = this.#canvas.parentElement;
    if (!parent) return;
    const { width, height } = parent.getBoundingClientRect();
    this.#canvas.width  = width;
    this.#canvas.height = height;
    if (this.#lastBuf) {
      this.render(this.#lastBuf, this.#lastN);
    } else {
      this._clear();
    }
  }

  _clear() {
    const ctx = this.#ctx;
    ctx.fillStyle = HOSPITAL_PALETTE.background;
    ctx.fillRect(0, 0, this.#canvas.width, this.#canvas.height);
  }

  /**
   * Main render pass.
   * @param {Float32Array} buf  – SoA patient buffer (FIELDS_PER_PATIENT × MAX_PATIENTS)
   * @param {number} count      – active patient census
   */
  render(buf, count) {
    const ctx  = this.#ctx;
    const W    = this.#canvas.width;
    const H    = this.#canvas.height;

    // Background
    ctx.fillStyle = HOSPITAL_PALETTE.background;
    ctx.fillRect(0, 0, W, H);

    const icuBeds  = DEFAULT_HOSPITAL_PARAMS.icuBeds;
    const wardBeds = DEFAULT_HOSPITAL_PARAMS.wardBeds;

    // Build a map room_id → disease state (last patient wins if multiple share room)
    const roomState = new Map();
    for (let i = 0; i < count; i++) {
      const roomId = Math.round(buf[PATIENT_FIELD.ROOM_ID * MAX_PATIENTS + i]);
      const ds     = Math.round(buf[PATIENT_FIELD.DISEASE_STATE * MAX_PATIENTS + i]);
      roomState.set(roomId, ds);
    }

    // Compute bed geometry
    const icuRows  = Math.ceil(icuBeds  / ICU_COLS);
    const wardRows = Math.ceil(wardBeds / WARD_COLS);

    const totalRows = icuRows + wardRows + 2; // +2 for labels
    const bedH = Math.max(12, (H - BED_PAD * (totalRows + 1) - BED_LABEL_H * 2) / totalRows);
    const icuW  = Math.max(12, (W - BED_PAD * (ICU_COLS  + 1)) / ICU_COLS);
    const wardW = Math.max(12, (W - BED_PAD * (WARD_COLS + 1)) / WARD_COLS);

    let yOff = BED_PAD;

    // ── ICU section ──────────────────────────────────────────────────────────
    this._sectionLabel(ctx, 'ICU', W / 2, yOff + 10, '#60a5fa');
    yOff += BED_LABEL_H + BED_PAD;

    for (let r = 0; r < icuRows; r++) {
      for (let c = 0; c < ICU_COLS; c++) {
        const bedIdx = r * ICU_COLS + c;
        if (bedIdx >= icuBeds) break;
        const x  = BED_PAD + c * (icuW + BED_PAD);
        const y  = yOff + r * (bedH + BED_PAD);
        const ds = roomState.get(bedIdx) ?? -1;
        this._drawBed(ctx, x, y, icuW, bedH, ds, true);
      }
    }
    yOff += icuRows * (bedH + BED_PAD) + BED_PAD * 2;

    // ── Ward section ─────────────────────────────────────────────────────────
    this._sectionLabel(ctx, 'General Ward', W / 2, yOff + 10, '#4ade80');
    yOff += BED_LABEL_H + BED_PAD;

    for (let r = 0; r < wardRows; r++) {
      for (let c = 0; c < WARD_COLS; c++) {
        const bedIdx = icuBeds + r * WARD_COLS + c;
        if (bedIdx >= icuBeds + wardBeds) break;
        const x  = BED_PAD + c * (wardW + BED_PAD);
        const y  = yOff + r * (bedH + BED_PAD);
        const ds = roomState.get(bedIdx) ?? -1;
        this._drawBed(ctx, x, y, wardW, bedH, ds, false);
      }
    }

    // ── Legend ───────────────────────────────────────────────────────────────
    this._drawLegend(ctx, W, H);
  }

  _drawBed(ctx, x, y, w, h, diseaseState, isIcu) {
    const empty = diseaseState < 0;
    const fill  = empty
      ? (isIcu ? HOSPITAL_PALETTE.icuBed : HOSPITAL_PALETTE.wardBed)
      : DISEASE_STATE_COLORS[diseaseState] ?? '#475569';

    ctx.fillStyle   = fill;
    ctx.strokeStyle = HOSPITAL_PALETTE.bedBorder;
    ctx.lineWidth   = 1;

    ctx.beginPath();
    ctx.roundRect(x, y, w, h, 2);
    ctx.fill();
    if (!empty) ctx.stroke();
  }

  _sectionLabel(ctx, text, x, y, color) {
    ctx.fillStyle  = color;
    ctx.font       = 'bold 11px monospace';
    ctx.textAlign  = 'center';
    ctx.fillText(text, x, y);
  }

  _drawLegend(ctx, W, H) {
    const labels = ['Susceptible','Exposed','Colonized','Infected','Treated','Recovered','Deceased'];
    const swSize = 10;
    const pad    = 6;
    const cols   = 4;
    const rowH   = 16;
    const totalW = cols * 110;
    const startX = (W - totalW) / 2;
    const startY = H - Math.ceil(labels.length / cols) * rowH - pad;

    ctx.font      = '10px monospace';
    ctx.textAlign = 'left';

    labels.forEach((lbl, i) => {
      const col = i % cols;
      const row = Math.floor(i / cols);
      const x   = startX + col * 110;
      const y   = startY + row * rowH;

      ctx.fillStyle = DISEASE_STATE_COLORS[i];
      ctx.fillRect(x, y, swSize, swSize);
      ctx.fillStyle = HOSPITAL_PALETTE.text;
      ctx.fillText(lbl, x + swSize + 3, y + swSize - 1);
    });
  }

  get canvas() { return this.#canvas; }
}
