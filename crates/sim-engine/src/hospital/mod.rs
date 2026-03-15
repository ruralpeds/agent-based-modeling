pub mod hospital_buffers;
pub mod hospital_engine;
/// Hospital simulation: params, buffer layout, and tick engine.
pub mod hospital_params;

pub use hospital_buffers::{write_nurse_summary, write_patient_soa, write_physician_summary};
pub use hospital_engine::HospitalSimEngine;
pub use hospital_params::{
    patient_field, HospitalParams, HospitalStats, FIELDS_PER_PATIENT, MAX_PATIENTS,
};
