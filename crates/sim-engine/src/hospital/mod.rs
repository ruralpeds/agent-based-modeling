/// Hospital simulation: params, buffer layout, and tick engine.
pub mod hospital_params;
pub mod hospital_buffers;
pub mod hospital_engine;

pub use hospital_params::{
    HospitalParams, HospitalStats,
    MAX_PATIENTS, FIELDS_PER_PATIENT, patient_field,
};
pub use hospital_engine::HospitalSimEngine;
pub use hospital_buffers::{write_patient_soa, write_nurse_summary, write_physician_summary};
