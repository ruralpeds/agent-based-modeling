import { dispatch } from 'd3-dispatch';

/**
 * Global event bus. All modules communicate through this.
 * Events:
 *   sim:tick    — { agentBuf, count, tick, stats }
 *   sim:reset   — { seed }
 *   sim:start   — {}
 *   sim:pause   — {}
 *   param:change — { key, value }
 *   agent:select — { id }  (null id = deselect)
 *   engine:error — { error }
 *   ui:resize   — {}
 */
export const ABMDispatch = dispatch(
  'sim:tick',
  'sim:reset',
  'sim:start',
  'sim:pause',
  'param:change',
  'agent:select',
  'engine:error',
  'ui:resize',
);
