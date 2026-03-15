/**
 * Memory layout constants for the hospital simulation SoA patient buffer.
 * Must stay in sync with:
 *   crates/sim-engine/src/hospital/hospital_params.rs  (patient_field module)
 *   crates/sim-engine/src/hospital/hospital_buffers.rs (write_patient_soa)
 */

export const MAX_PATIENTS       = 500;
export const FIELDS_PER_PATIENT = 12;

export const HOSPITAL_HEADER_INT32S = 8;
export const HOSPITAL_HEADER_BYTES  = HOSPITAL_HEADER_INT32S * 4;
export const HOSPITAL_PATIENT_BYTES = MAX_PATIENTS * FIELDS_PER_PATIENT * 4;
export const HOSPITAL_TOTAL_SAB_BYTES = HOSPITAL_HEADER_BYTES + HOSPITAL_PATIENT_BYTES;

/**
 * Patient field indices in SoA layout.
 * Buffer offset for field F of patient i = PATIENT_FIELD[F] * N + i,
 * where N = current patient count.
 */
export const PATIENT_FIELD = {
  ROOM_ID:            0,
  UNIT:               1,   // 0 = ICU, 1 = ward
  DISEASE_STATE:      2,   // DiseaseState discriminant (0–6)
  AGE:                3,   // age in sim (ticks)
  WBC:                4,   // white blood cell count (× 10⁹/L)
  TEMPERATURE:        5,   // °C
  DRUG_CONC:          6,   // antibiotic plasma concentration (mg/L)
  RESISTANCE:         7,   // ResistanceProfile discriminant (0–3)
  ANTIBIOTIC_DAYS:    8,   // cumulative antibiotic days
  ADMISSION_SEVERITY: 9,   // SOFA-equivalent severity score
  TREATMENT_ACTION:   10,  // TreatmentAction discriminant (0–5)
  PATIENT_ID:         11,  // unique patient identifier
};

/** Disease state discriminants — matches DiseaseState enum order */
export const DISEASE_STATE = {
  SUSCEPTIBLE: 0,
  EXPOSED:     1,
  COLONIZED:   2,
  INFECTED:    3,
  TREATED:     4,
  RECOVERED:   5,
  DECEASED:    6,
};

/** Hospital SAB header Int32Array indices */
export const HOSPITAL_HEADER = {
  PATIENT_COUNT: 0,
  TICK:          1,
  WRITE_LOCK:    2,
};
