export const MAX_AGENTS       = 5000;
export const FIELDS_PER_AGENT = 10;
export const HEADER_INT32S    = 8;
export const HEADER_BYTES     = HEADER_INT32S * 4;
export const AGENT_BUFFER_BYTES = MAX_AGENTS * FIELDS_PER_AGENT * 4;
export const TOTAL_SAB_BYTES  = HEADER_BYTES + AGENT_BUFFER_BYTES;

/** Field index in SoA layout (offset = FIELD.X * count + agentIndex) */
export const FIELD = {
  POS_X:      0,
  POS_Y:      1,
  ENERGY:     2,
  AGENT_TYPE: 3,
  ACTION:     4,
  AGE:        5,
  INTENTION:  6,
  KILLS:      7,
  OFFSPRING:  8,
  ID:         9,
};

/** Header Int32Array indices */
export const HEADER = {
  AGENT_COUNT: 0,
  TICK:        1,
  WRITE_LOCK:  2,
};

export const ACTION_CODES = {
  WANDER: 0, FORAGE: 1, FLEE: 2, HUNT: 3, REST: 4, PATROL: 5, SOCIALIZE: 6,
};

export const INTENTION_CODES = {
  WANDER: 0, EAT: 1, SURVIVE: 2, REPRODUCE: 3, HUNT: 4, PATROL: 5, REST: 6,
};
