import { ABMDispatch } from './dispatch.js';

const _state = {
  selectedAgentId: null,
  isRunning:       false,
  speed:           1,
  activeChart:     'population',
  zoom:            1,
  showTrails:      false,
  showVision:      false,
  showGrid:        false,
};

export const uiStore = {
  get(key)       { return _state[key]; },
  set(key, value) { _state[key] = value; },
  selectAgent(id) {
    _state.selectedAgentId = id;
    ABMDispatch.call('agent:select', this, { id });
  },
  setRunning(v)  { _state.isRunning = v; },
  setSpeed(v)    { _state.speed = v; },
};
