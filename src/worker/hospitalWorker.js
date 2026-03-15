// Hospital simulation Web Worker — WasmHospitalEngine lives here.
// Loaded as a module worker by hospitalBridge.js.

import { timer } from 'd3-timer';
import { DEFAULT_HOSPITAL_PARAMS } from '../constants.js';
import { HOSPITAL_HEADER, HOSPITAL_HEADER_BYTES, FIELDS_PER_PATIENT } from '../bridge/hospitalBufferLayout.js';

let engine     = null;
let loopTimer  = null;
let sab        = null;
let headerView = null;   // Int32Array over SAB header
let sabPatients = null;  // Float32Array over SAB patient data
let speed      = 1;
let wasmModule = null;

// First message must be INIT with the SharedArrayBuffer
self.addEventListener('message', async (event) => {
  if (event.data.type !== 'INIT') return;
  sab         = event.data.sab;
  headerView  = new Int32Array(sab, 0, 8);
  sabPatients = new Float32Array(sab, HOSPITAL_HEADER_BYTES);

  try {
    wasmModule = await import(/* @vite-ignore */ import.meta.env.BASE_URL + 'pkg/sim_engine.js');
    await wasmModule.default();  // initialise WASM

    engine = new wasmModule.WasmHospitalEngine(
      JSON.stringify(DEFAULT_HOSPITAL_PARAMS),
      42,
    );

    self.addEventListener('message', handleCommand);
    self.postMessage({ type: 'READY' });
  } catch (err) {
    self.postMessage({ type: 'ERROR', data: { error: err.toString() } });
  }
}, { once: true });

function handleCommand({ data }) {
  switch (data.type) {
    case 'START':
      startLoop();
      break;
    case 'PAUSE':
      stopLoop();
      break;
    case 'STEP':
      stopLoop();
      tick();
      break;
    case 'RESET':
      stopLoop();
      engine.reset(data.seed ?? Math.floor(Math.random() * 0xFFFFFFFF));
      tick();
      break;
    case 'SET_PARAMS':
      try {
        engine.set_params(JSON.stringify(data.params));
      } catch (e) {
        self.postMessage({ type: 'ERROR', data: { error: e.toString() } });
      }
      break;
    case 'SET_SPEED':
      speed = data.multiplier ?? 1;
      break;
    default:
      break;
  }
}

function tick() {
  if (!engine) return;
  try {
    const stepsPerFrame = Math.max(1, Math.round(speed));
    let patientCount = 0;
    for (let i = 0; i < stepsPerFrame; i++) {
      patientCount = engine.step();
    }

    // Zero-copy view of WASM patient buffer → copy into SAB
    const patientBuf = engine.get_patient_buffer();
    // SoA: each field stripe is MAX_PATIENTS wide (not patientCount).
    // The Rust writer fills all FIELDS_PER_PATIENT stripes × MAX_PATIENTS slots.
    const STRIPE = 500; // MAX_PATIENTS
    sabPatients.set(patientBuf.subarray(0, FIELDS_PER_PATIENT * STRIPE));

    // Update header atomically
    Atomics.store(headerView, HOSPITAL_HEADER.PATIENT_COUNT, patientCount);
    Atomics.store(headerView, HOSPITAL_HEADER.TICK, engine.get_tick());

    const stats = engine.get_stats_json();
    self.postMessage({ type: 'TICK', data: { stats } });
  } catch (err) {
    stopLoop();
    self.postMessage({ type: 'ERROR', data: { error: err.toString() } });
  }
}

function startLoop() {
  stopLoop();
  loopTimer = timer(tick);
}

function stopLoop() {
  if (loopTimer) { loopTimer.stop(); loopTimer = null; }
}
