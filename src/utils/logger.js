const LEVELS = { debug: 0, info: 1, warn: 2, error: 3 };
const MIN_LEVEL = import.meta.env?.DEV ? 0 : 2;

function log(level, tag, ...args) {
  if (LEVELS[level] < MIN_LEVEL) return;
  const prefix = `[ABM:${tag}]`;
  switch (level) {
    case 'debug': console.debug(prefix, ...args); break;
    case 'info':  console.info(prefix, ...args);  break;
    case 'warn':  console.warn(prefix, ...args);  break;
    case 'error': console.error(prefix, ...args); break;
  }
}

export const logger = {
  debug: (tag, ...a) => log('debug', tag, ...a),
  info:  (tag, ...a) => log('info',  tag, ...a),
  warn:  (tag, ...a) => log('warn',  tag, ...a),
  error: (tag, ...a) => log('error', tag, ...a),
};
