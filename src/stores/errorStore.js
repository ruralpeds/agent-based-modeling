import { ABMDispatch } from './dispatch.js';
import { logger } from '../utils/logger.js';

const _errors = [];

ABMDispatch.on('engine:error', (d) => {
  _errors.push({ message: d.error?.toString() ?? 'Unknown error', ts: Date.now() });
  logger.error('engine', d.error);
});

export const errorStore = {
  getLatest() { return _errors[_errors.length - 1] ?? null; },
  clear()     { _errors.length = 0; },
  hasErrors() { return _errors.length > 0; },
};
