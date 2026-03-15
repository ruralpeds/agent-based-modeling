import { DEFAULT_PARAMS, PARAM_LIMITS } from '../constants.js';
import { ABMDispatch } from './dispatch.js';

let _params = { ...DEFAULT_PARAMS };

export const paramStore = {
  get()         { return { ..._params }; },
  getKey(k)     { return _params[k]; },

  set(key, value) {
    const limits = PARAM_LIMITS[key];
    if (limits) {
      value = Math.max(limits.min, Math.min(limits.max, value));
    }
    _params[key] = value;
    ABMDispatch.call('param:change', this, { key, value });
    return _params[key];
  },

  setAll(params) {
    _params = { ..._params, ...params };
    ABMDispatch.call('param:change', this, { key: null, value: null });
  },

  reset() { _params = { ...DEFAULT_PARAMS }; },

  toJSON() { return JSON.stringify(_params); },
};
