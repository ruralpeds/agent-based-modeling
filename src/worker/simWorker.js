// Runs entirely in a Web Worker — WASM lives here
// This file is loaded as a module worker by simBridge.js

import { timer } from 'd3-timer';
import { DEFAULT_PARAMS } from '../constants.js';
import { HEADER } from '../bridge/bufferLayout.js';

let engine    = null;
let loopTimer = null;
let sab       = null;
let headerView = null;   // Int32Array over SAB header
let sabAgents  = null;   // Float32Array over SAB agent data
let speed     = 1;
let wasmModule = null;

const HEADER_BYTES = 32;

// First message must be INIT with the SharedArrayBuffer
self.addEventListener('message', async (event) => {
  if (event.data.type !== 'INIT') return;
  sab        = event.data.sab;
  headerView = new Int32Array(sab, 0, 8);
  sabAgents  = new Float32Array(sab, HEADER_BYTES);

  try {
    // Dynamic import of the wasm-pack generated module.
    // Uses import.meta.env.BASE_URL so the path is correct on both
    // localhost (base '/') and GitHub Pages (base '/repo-name/').
    wasmModule = await import(/* @vite-ignore */ import.meta.env.BASE_URL + 'pkg/sim_engine.js');
    await wasmModule.default();  // initialize WASM

    engine = new wasmModule.WasmSimEngine(JSON.stringify(DEFAULT_PARAMS), 42);

    // Register ongoing command handler
    self.addEventListener('message', handleCommand);
    self.postMessage({ type: 'READY' });
  } catch (err) {
    self.postMessage({ type: 'ERROR', data: { error: err.toString() } });
  }
}, { once: true });

function handleCommand({ data }) {
  switch (data.type) {
    case 'START':
      startLoop(); break;
    case 'PAUSE':
      stopLoop(); break;
    case 'STEP':
      stopLoop(); tick(); break;
    case 'RESET':
      stopLoop();
      engine.reset(data.seed ?? Math.floor(Math.random() * 0xFFFFFFFF));
      tick(); break;
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
  }
}

function tick() {
  if (!engine) return;
  try {
    const stepsPerFrame = Math.max(1, Math.round(speed));
    let lastCount = 0;
    for (let i = 0; i < stepsPerFrame; i++) {
      lastCount = engine.step();
    }

    // Copy agent data from WASM buffer into SAB
    const agentBuf = engine.get_agent_buffer();
    sabAgents.set(agentBuf.subarray(0, lastCount * 10));

    // Update header atomically
    Atomics.store(headerView, HEADER.AGENT_COUNT, lastCount);
    Atomics.store(headerView, HEADER.TICK, engine.get_tick());

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
