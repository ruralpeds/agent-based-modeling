/// SoA (Structure-of-Arrays) buffer writer for hospital patient agents.
///
/// Layout (12 × f32 per patient, MAX_PATIENTS = 500):
///
/// Offset stripe  Field              Encoding
/// ──────────────────────────────────────────────────────────
///  0 × N         room_id            u16 as f32
///  1 × N         unit               HospitalUnit as u8 as f32
///  2 × N         disease_state      DiseaseState as usize as f32
///  3 × N         age                u8 as f32
///  4 × N         wbc                f32
///  5 × N         temperature        f32
///  6 × N         drug_conc          f32
///  7 × N         resistance         ResistanceProfile as u8 as f32
///  8 × N         antibiotic_days    u16 as f32
///  9 × N         admission_severity f32
/// 10 × N         treatment_action   TreatmentAction as u8 as f32
/// 11 × N         patient_id         u32 as f32
///
/// Header (first 8 bytes of the SharedArrayBuffer, Int32Array):
///   [0] patient_count  (Atomics.load)
///   [1] tick           (Atomics.load)
use crate::agents::patient_agent::PatientAgent;
use super::hospital_params::FIELDS_PER_PATIENT;

/// Write all patients in SoA order to `buf`.
/// `buf` must be at least `patients.len() × FIELDS_PER_PATIENT` floats long.
/// Returns the number of f32 values written.
pub fn write_patient_soa(patients: &[PatientAgent], buf: &mut [f32]) -> usize {
    let n = patients.len();
    if n == 0 || buf.len() < n * FIELDS_PER_PATIENT {
        return 0;
    }
    for (i, p) in patients.iter().enumerate() {
        buf[i]              = p.room_id             as f32;
        buf[n      + i]     = p.unit                as u8 as f32;
        buf[2  * n + i]     = p.disease_state       as usize as f32;
        buf[3  * n + i]     = p.age                 as f32;
        buf[4  * n + i]     = p.wbc;
        buf[5  * n + i]     = p.temperature;
        buf[6  * n + i]     = p.drug_conc;
        buf[7  * n + i]     = p.resistance_profile  as u8 as f32;
        buf[8  * n + i]     = p.antibiotic_days      as f32;
        buf[9  * n + i]     = p.admission_severity;
        buf[10 * n + i]     = p.treatment_action()  as u8 as f32;
        buf[11 * n + i]     = p.id                  as f32;
    }
    n * FIELDS_PER_PATIENT
}

/// Nurse-facing summary buffer: 4 × f32 per nurse.
/// [compliance, workload, fatigue, n_patients]
pub fn write_nurse_summary(nurses: &[crate::agents::nurse_agent::NurseAgent], buf: &mut [f32]) -> usize {
    let n = nurses.len();
    if n == 0 || buf.len() < n * 4 { return 0; }
    for (i, nurse) in nurses.iter().enumerate() {
        buf[i]         = nurse.base_compliance;
        buf[n     + i] = nurse.workload;
        buf[2 * n + i] = nurse.fatigue;
        buf[3 * n + i] = nurse.n_patients as f32;
    }
    n * 4
}

/// Physician-facing summary buffer: 3 × f32 per physician.
/// [compliance, map_disease_state, n_patients]
pub fn write_physician_summary(
    physicians: &[crate::agents::physician_agent::PhysicianAgent],
    buf: &mut [f32],
) -> usize {
    let n = physicians.len();
    if n == 0 || buf.len() < n * 3 { return 0; }
    for (i, phys) in physicians.iter().enumerate() {
        buf[i]         = phys.social.current_compliance;
        buf[n     + i] = phys.pbdi.map_state() as usize as f32;
        buf[2 * n + i] = phys.n_patients as f32;
    }
    n * 3
}
