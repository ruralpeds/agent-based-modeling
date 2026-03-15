import { describe, test, expect } from 'vitest';
import {
  MAX_PATIENTS,
  FIELDS_PER_PATIENT,
  HOSPITAL_HEADER_INT32S,
  HOSPITAL_HEADER_BYTES,
  HOSPITAL_PATIENT_BYTES,
  HOSPITAL_TOTAL_SAB_BYTES,
  PATIENT_FIELD,
  DISEASE_STATE,
  HOSPITAL_HEADER,
} from '@/bridge/hospitalBufferLayout.js';

// ── SAB size constants ────────────────────────────────────────────────────────

describe('Hospital SAB size constants', () => {
  test('MAX_PATIENTS is 500', () => expect(MAX_PATIENTS).toBe(500));
  test('FIELDS_PER_PATIENT is 12', () => expect(FIELDS_PER_PATIENT).toBe(12));

  test('HOSPITAL_HEADER_BYTES equals HOSPITAL_HEADER_INT32S * 4', () => {
    expect(HOSPITAL_HEADER_BYTES).toBe(HOSPITAL_HEADER_INT32S * 4);
  });

  test('HOSPITAL_HEADER_BYTES is 32', () => expect(HOSPITAL_HEADER_BYTES).toBe(32));

  test('HOSPITAL_PATIENT_BYTES equals MAX_PATIENTS * FIELDS_PER_PATIENT * 4', () => {
    expect(HOSPITAL_PATIENT_BYTES).toBe(MAX_PATIENTS * FIELDS_PER_PATIENT * 4);
  });

  test('HOSPITAL_PATIENT_BYTES is 24000 (500 × 12 × 4)', () => {
    expect(HOSPITAL_PATIENT_BYTES).toBe(24_000);
  });

  test('HOSPITAL_TOTAL_SAB_BYTES equals header + patient bytes', () => {
    expect(HOSPITAL_TOTAL_SAB_BYTES).toBe(HOSPITAL_HEADER_BYTES + HOSPITAL_PATIENT_BYTES);
  });

  test('HOSPITAL_TOTAL_SAB_BYTES is 24032', () => {
    expect(HOSPITAL_TOTAL_SAB_BYTES).toBe(24_032);
  });

  test('HOSPITAL_TOTAL_SAB_BYTES is divisible by 4 (Float32Array alignment)', () => {
    expect(HOSPITAL_TOTAL_SAB_BYTES % 4).toBe(0);
  });
});

// ── PATIENT_FIELD SoA indices ─────────────────────────────────────────────────

describe('PATIENT_FIELD SoA indices match Rust patient_field module', () => {
  test('ROOM_ID is 0',            () => expect(PATIENT_FIELD.ROOM_ID).toBe(0));
  test('UNIT is 1',               () => expect(PATIENT_FIELD.UNIT).toBe(1));
  test('DISEASE_STATE is 2',      () => expect(PATIENT_FIELD.DISEASE_STATE).toBe(2));
  test('AGE is 3',                () => expect(PATIENT_FIELD.AGE).toBe(3));
  test('WBC is 4',                () => expect(PATIENT_FIELD.WBC).toBe(4));
  test('TEMPERATURE is 5',        () => expect(PATIENT_FIELD.TEMPERATURE).toBe(5));
  test('DRUG_CONC is 6',          () => expect(PATIENT_FIELD.DRUG_CONC).toBe(6));
  test('RESISTANCE is 7',         () => expect(PATIENT_FIELD.RESISTANCE).toBe(7));
  test('ANTIBIOTIC_DAYS is 8',    () => expect(PATIENT_FIELD.ANTIBIOTIC_DAYS).toBe(8));
  test('ADMISSION_SEVERITY is 9', () => expect(PATIENT_FIELD.ADMISSION_SEVERITY).toBe(9));
  test('TREATMENT_ACTION is 10',  () => expect(PATIENT_FIELD.TREATMENT_ACTION).toBe(10));
  test('PATIENT_ID is 11',        () => expect(PATIENT_FIELD.PATIENT_ID).toBe(11));

  test('exactly 12 fields (matches FIELDS_PER_PATIENT)', () => {
    expect(Object.keys(PATIENT_FIELD).length).toBe(FIELDS_PER_PATIENT);
  });

  test('all field indices are unique', () => {
    const values = Object.values(PATIENT_FIELD);
    expect(new Set(values).size).toBe(values.length);
  });

  test('field indices span 0..11 with no gaps', () => {
    const values = Object.values(PATIENT_FIELD).sort((a, b) => a - b);
    values.forEach((v, i) => expect(v).toBe(i));
  });
});

// ── DISEASE_STATE discriminants ───────────────────────────────────────────────

describe('DISEASE_STATE discriminants match Rust DiseaseState enum', () => {
  test('SUSCEPTIBLE is 0', () => expect(DISEASE_STATE.SUSCEPTIBLE).toBe(0));
  test('EXPOSED is 1',     () => expect(DISEASE_STATE.EXPOSED).toBe(1));
  test('COLONIZED is 2',   () => expect(DISEASE_STATE.COLONIZED).toBe(2));
  test('INFECTED is 3',    () => expect(DISEASE_STATE.INFECTED).toBe(3));
  test('TREATED is 4',     () => expect(DISEASE_STATE.TREATED).toBe(4));
  test('RECOVERED is 5',   () => expect(DISEASE_STATE.RECOVERED).toBe(5));
  test('DECEASED is 6',    () => expect(DISEASE_STATE.DECEASED).toBe(6));

  test('exactly 7 states', () => {
    expect(Object.keys(DISEASE_STATE).length).toBe(7);
  });

  test('all discriminants are unique', () => {
    const values = Object.values(DISEASE_STATE);
    expect(new Set(values).size).toBe(values.length);
  });
});

// ── HOSPITAL_HEADER indices ───────────────────────────────────────────────────

describe('HOSPITAL_HEADER Int32Array indices', () => {
  test('PATIENT_COUNT is 0', () => expect(HOSPITAL_HEADER.PATIENT_COUNT).toBe(0));
  test('TICK is 1',          () => expect(HOSPITAL_HEADER.TICK).toBe(1));
  test('WRITE_LOCK is 2',    () => expect(HOSPITAL_HEADER.WRITE_LOCK).toBe(2));

  test('all header indices are unique', () => {
    const values = Object.values(HOSPITAL_HEADER);
    expect(new Set(values).size).toBe(values.length);
  });
});

// ── JS/Rust layout contract ───────────────────────────────────────────────────

describe('JS/Rust patient SoA layout contract', () => {
  test('patient 0 room_id starts at Float32Array index 0', () => {
    const offset = PATIENT_FIELD.ROOM_ID * MAX_PATIENTS + 0;
    expect(offset).toBe(0);
  });

  test('patient 0 disease_state offset is 2 * MAX_PATIENTS', () => {
    const offset = PATIENT_FIELD.DISEASE_STATE * MAX_PATIENTS + 0;
    expect(offset).toBe(1_000);
  });

  test('patient 499 patient_id is the last element in buffer', () => {
    const lastOffset = PATIENT_FIELD.PATIENT_ID * MAX_PATIENTS + (MAX_PATIENTS - 1);
    expect(lastOffset).toBe(MAX_PATIENTS * FIELDS_PER_PATIENT - 1);
  });

  test('SoA offset formula: stripe = PATIENT_FIELD.X * MAX_PATIENTS + patientIndex', () => {
    const wbcOffset = PATIENT_FIELD.WBC * MAX_PATIENTS + 5;
    expect(wbcOffset).toBe(4 * MAX_PATIENTS + 5);
  });
});
