import { describe, test, expect } from 'vitest';
import {
  MAX_AGENTS,
  FIELDS_PER_AGENT,
  HEADER_INT32S,
  HEADER_BYTES,
  AGENT_BUFFER_BYTES,
  TOTAL_SAB_BYTES,
  FIELD,
  HEADER,
  ACTION_CODES,
  INTENTION_CODES,
} from '@/bridge/bufferLayout.js';

describe('SAB size constants', () => {
  test('HEADER_BYTES equals HEADER_INT32S * 4', () => {
    expect(HEADER_BYTES).toBe(HEADER_INT32S * 4);
  });

  test('HEADER_BYTES is 32 (8 Int32s × 4 bytes)', () => {
    expect(HEADER_BYTES).toBe(32);
  });

  test('AGENT_BUFFER_BYTES equals MAX_AGENTS * FIELDS_PER_AGENT * 4', () => {
    expect(AGENT_BUFFER_BYTES).toBe(MAX_AGENTS * FIELDS_PER_AGENT * 4);
  });

  test('AGENT_BUFFER_BYTES is 200000 bytes for 5000 agents × 10 fields', () => {
    expect(AGENT_BUFFER_BYTES).toBe(200_000);
  });

  test('TOTAL_SAB_BYTES equals HEADER_BYTES + AGENT_BUFFER_BYTES', () => {
    expect(TOTAL_SAB_BYTES).toBe(HEADER_BYTES + AGENT_BUFFER_BYTES);
  });

  test('TOTAL_SAB_BYTES is 200032', () => {
    // 32 header + 200000 agent data
    expect(TOTAL_SAB_BYTES).toBe(200_032);
  });

  test('TOTAL_SAB_BYTES is divisible by 4 (Float32Array alignment)', () => {
    expect(TOTAL_SAB_BYTES % 4).toBe(0);
  });
});

describe('FIELD SoA indices match CLAUDE.md spec', () => {
  // Layout doc: Offset = FIELD.X * N + agentIndex
  test('POS_X is 0', () => expect(FIELD.POS_X).toBe(0));
  test('POS_Y is 1', () => expect(FIELD.POS_Y).toBe(1));
  test('ENERGY is 2', () => expect(FIELD.ENERGY).toBe(2));
  test('AGENT_TYPE is 3', () => expect(FIELD.AGENT_TYPE).toBe(3));
  test('ACTION is 4', () => expect(FIELD.ACTION).toBe(4));
  test('AGE is 5', () => expect(FIELD.AGE).toBe(5));
  test('INTENTION is 6', () => expect(FIELD.INTENTION).toBe(6));
  test('KILLS is 7', () => expect(FIELD.KILLS).toBe(7));
  test('OFFSPRING is 8', () => expect(FIELD.OFFSPRING).toBe(8));
  test('ID is 9', () => expect(FIELD.ID).toBe(9));

  test('exactly 10 fields defined (matches FIELDS_PER_AGENT)', () => {
    expect(Object.keys(FIELD).length).toBe(FIELDS_PER_AGENT);
  });

  test('all field indices are unique', () => {
    const values = Object.values(FIELD);
    expect(new Set(values).size).toBe(values.length);
  });

  test('field indices span 0..9 with no gaps', () => {
    const values = Object.values(FIELD).sort((a, b) => a - b);
    values.forEach((v, i) => expect(v).toBe(i));
  });
});

describe('HEADER Int32Array indices', () => {
  test('AGENT_COUNT is 0', () => expect(HEADER.AGENT_COUNT).toBe(0));
  test('TICK is 1', () => expect(HEADER.TICK).toBe(1));
  test('WRITE_LOCK is 2', () => expect(HEADER.WRITE_LOCK).toBe(2));

  test('all header indices are unique', () => {
    const values = Object.values(HEADER);
    expect(new Set(values).size).toBe(values.length);
  });
});

describe('ACTION_CODES', () => {
  test('WANDER is 0', () => expect(ACTION_CODES.WANDER).toBe(0));
  test('FORAGE is 1', () => expect(ACTION_CODES.FORAGE).toBe(1));
  test('FLEE is 2', () => expect(ACTION_CODES.FLEE).toBe(2));
  test('HUNT is 3', () => expect(ACTION_CODES.HUNT).toBe(3));
  test('REST is 4', () => expect(ACTION_CODES.REST).toBe(4));
  test('PATROL is 5', () => expect(ACTION_CODES.PATROL).toBe(5));
  test('SOCIALIZE is 6', () => expect(ACTION_CODES.SOCIALIZE).toBe(6));

  test('exactly 7 action codes (matches action_dist array size)', () => {
    expect(Object.keys(ACTION_CODES).length).toBe(7);
  });

  test('all action codes are unique', () => {
    const values = Object.values(ACTION_CODES);
    expect(new Set(values).size).toBe(values.length);
  });
});

describe('INTENTION_CODES', () => {
  test('WANDER is 0', () => expect(INTENTION_CODES.WANDER).toBe(0));
  test('EAT is 1', () => expect(INTENTION_CODES.EAT).toBe(1));
  test('SURVIVE is 2', () => expect(INTENTION_CODES.SURVIVE).toBe(2));
  test('REPRODUCE is 3', () => expect(INTENTION_CODES.REPRODUCE).toBe(3));
  test('HUNT is 4', () => expect(INTENTION_CODES.HUNT).toBe(4));
  test('PATROL is 5', () => expect(INTENTION_CODES.PATROL).toBe(5));
  test('REST is 6', () => expect(INTENTION_CODES.REST).toBe(6));

  test('all intention codes are unique', () => {
    const values = Object.values(INTENTION_CODES);
    expect(new Set(values).size).toBe(values.length);
  });
});

describe('JS/Rust layout contract', () => {
  test('SoA offset formula: field_offset = FIELD.X * MAX_AGENTS + agentIndex', () => {
    // At MAX_AGENTS=5000, agent 0 energy offset in Float32Array:
    const energyOffset = FIELD.ENERGY * MAX_AGENTS + 0;
    expect(energyOffset).toBe(10_000);
  });

  test('agent 0 pos_x starts at Float32Array index 0', () => {
    const offset = FIELD.POS_X * MAX_AGENTS + 0;
    expect(offset).toBe(0);
  });

  test('agent 4999 id is the last element in the buffer', () => {
    const lastOffset = FIELD.ID * MAX_AGENTS + (MAX_AGENTS - 1);
    expect(lastOffset).toBe(MAX_AGENTS * FIELDS_PER_AGENT - 1);
  });
});
