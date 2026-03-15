import { ABMDispatch } from '../stores/dispatch.js';
import { logger }      from '../utils/logger.js';
import {
  TOTAL_SAB_BYTES, HEADER_BYTES, HEADER,
} from './bufferLayout.js';

class SimBridgeClass {
  #worker      = null;
  #sab         = null;
  #headerView  = null;
  #agentView   = null;
  #running     = false;
  #initPromise = null;

  init() {
    if (this.#initPromise) return this.#initPromise;

    this.#initPromise = new Promise((resolve, reject) => {
      if (!crossOriginIsolated) {
        reject(new Error(
          'SharedArrayBuffer unavailable: page not cross-origin isolated. ' +
          'Ensure COOP/COEP headers are set (see vite.config.js).'
        ));
        return;
      }

      this.#sab        = new SharedArrayBuffer(TOTAL_SAB_BYTES);
      this.#headerView = new Int32Array(this.#sab, 0, 8);
      this.#agentView  = new Float32Array(this.#sab, HEADER_BYTES);

      this.#worker = new Worker(
        new URL('../worker/simWorker.js', import.meta.url),
        { type: 'module' }
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
      const count    = Atomics.load(this.#headerView, HEADER.AGENT_COUNT);
      const tick     = Atomics.load(this.#headerView, HEADER.TICK);
      const agentBuf = this.#agentView;
      ABMDispatch.call('sim:tick', this, { agentBuf, count, tick, stats: data?.stats });
    } else if (type === 'ERROR') {
      ABMDispatch.call('engine:error', this, { error: data?.error });
    }
  }

  #onError(error) {
    logger.error('worker-uncaught', error);
    ABMDispatch.call('engine:error', this, { error });
  }

  start()  {
    this.#running = true;
    ABMDispatch.call('sim:start', this, {});
    this.#post('START');
  }
  pause()  {
    this.#running = false;
    ABMDispatch.call('sim:pause', this, {});
    this.#post('PAUSE');
  }
  step()   { this.#post('STEP'); }
  reset(seed = Math.floor(Math.random() * 0xFFFFFFFF)) {
    this.#running = false;
    ABMDispatch.call('sim:reset', this, { seed });
    this.#post('RESET', { seed });
  }
  setParams(params) { this.#post('SET_PARAMS', { params }); }
  setSpeed(multiplier) { this.#post('SET_SPEED', { multiplier }); }

  get isRunning() { return this.#running; }

  readField(index, field, count) {
    return this.#agentView[field * count + index];
  }

  #post(type, data = {}) {
    if (!this.#worker) { logger.warn('bridge', 'Worker not initialized'); return; }
    this.#worker.postMessage({ type, ...data });
  }
}

export const SimBridge = new SimBridgeClass();
