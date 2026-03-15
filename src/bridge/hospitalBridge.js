import { ABMDispatch } from '../stores/dispatch.js';
import { logger }      from '../utils/logger.js';
import {
  HOSPITAL_TOTAL_SAB_BYTES,
  HOSPITAL_HEADER_BYTES,
  HOSPITAL_HEADER,
  MAX_PATIENTS,
  FIELDS_PER_PATIENT,
} from './hospitalBufferLayout.js';

class HospitalBridgeClass {
  #worker       = null;
  #sab          = null;
  #headerView   = null;
  #patientView  = null;
  #running      = false;
  #initPromise  = null;

  init() {
    if (this.#initPromise) return this.#initPromise;

    this.#initPromise = new Promise((resolve, reject) => {
      if (!crossOriginIsolated) {
        reject(new Error(
          'SharedArrayBuffer unavailable: page not cross-origin isolated. ' +
          'Ensure COOP/COEP headers are set (see vite.config.js).',
        ));
        return;
      }

      this.#sab         = new SharedArrayBuffer(HOSPITAL_TOTAL_SAB_BYTES);
      this.#headerView  = new Int32Array(this.#sab, 0, 8);
      this.#patientView = new Float32Array(this.#sab, HOSPITAL_HEADER_BYTES);

      this.#worker = new Worker(
        new URL('../worker/hospitalWorker.js', import.meta.url),
        { type: 'module' },
      );

      this.#worker.postMessage({ type: 'INIT', sab: this.#sab });

      const onReady = (e) => {
        if (e.data.type === 'READY') {
          this.#worker.removeEventListener('message', onReady);
          this.#worker.addEventListener('message', this.#onMessage.bind(this));
          this.#worker.addEventListener('error',   this.#onError.bind(this));
          resolve();
        } else if (e.data.type === 'ERROR') {
          reject(new Error(e.data.data?.error));
        }
      };
      this.#worker.addEventListener('message', onReady);
    });

    return this.#initPromise;
  }

  #onMessage(event) {
    const { type, data } = event.data;
    if (type === 'TICK') {
      const count      = Atomics.load(this.#headerView, HOSPITAL_HEADER.PATIENT_COUNT);
      const tick       = Atomics.load(this.#headerView, HOSPITAL_HEADER.TICK);
      const patientBuf = this.#patientView;

      // Parse stats and check for outbreak signal
      let stats = null;
      if (data?.stats) {
        try { stats = JSON.parse(data.stats); } catch { /* ignore */ }
      }

      ABMDispatch.call('hospital:tick', this, { patientBuf, count, tick, stats });

      if (stats?.outbreakSignal) {
        ABMDispatch.call('hospital:outbreak', this, { tick, stats });
      }
    } else if (type === 'ERROR') {
      ABMDispatch.call('engine:error', this, { error: data?.error });
    }
  }

  #onError(error) {
    logger.error('hospital-worker-uncaught', error);
    ABMDispatch.call('engine:error', this, { error });
  }

  start() {
    this.#running = true;
    ABMDispatch.call('hospital:start', this, {});
    this.#post('START');
  }

  pause() {
    this.#running = false;
    ABMDispatch.call('hospital:pause', this, {});
    this.#post('PAUSE');
  }

  step()  { this.#post('STEP'); }

  reset(seed = Math.floor(Math.random() * 0xFFFFFFFF)) {
    this.#running = false;
    ABMDispatch.call('hospital:reset', this, { seed });
    this.#post('RESET', { seed });
  }

  setParams(params) { this.#post('SET_PARAMS', { params }); }
  setSpeed(multiplier) { this.#post('SET_SPEED', { multiplier }); }

  get isRunning() { return this.#running; }

  /**
   * Read a single patient field value from the SAB patient view.
   * @param {number} patientIndex  – index within active census (0..count-1)
   * @param {number} field         – PATIENT_FIELD constant
   * @returns {number}
   */
  readField(patientIndex, field) {
    return this.#patientView[field * MAX_PATIENTS + patientIndex];
  }

  #post(type, data = {}) {
    if (!this.#worker) { logger.warn('hospital-bridge', 'Worker not initialised'); return; }
    this.#worker.postMessage({ type, ...data });
  }
}

export const HospitalBridge = new HospitalBridgeClass();
