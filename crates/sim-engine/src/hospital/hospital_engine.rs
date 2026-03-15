/// HospitalSimEngine — tick-driven HAI simulation.
///
/// Tick loop (one 5-minute step at 12 ticks/h):
///
///  1. Patient arrivals via NHPP (Lewis-Shedler thinning).
///  2. Nurse activity advance → direct-contact transmission events.
///  3. Bed contamination decay + indirect-contact exposure.
///  4. Nurse fatigue update (OU process).
///  5. Patient biomarker step (OU WBC/temp, PK drug_conc).
///  6. DTMC disease-state transition for each patient.
///  7. Physician deliberation (pBDI) → update patient treatment action.
///  8. Apply treatment: start antibiotics, isolate, discharge.
///  9. Resistance mutation check for antibiotic-exposed patients.
/// 10. Discharge patients at planned LOS or explicit discharge action.
/// 11. SPRT outbreak detection update.
/// 12. Periodic social learning for providers.
/// 13. Write SoA patient buffer.
use crate::rng::Mulberry32;
use crate::stochastic::dtmc::{PatientDTMC, DiseaseState, build_hai_dtmc_matrix};
use crate::stochastic::sde::{OrnsteinUhlenbeck, OneCompartmentPk};
use crate::stochastic::poisson::NhppArrivalProcess;
use crate::stochastic::bayesian::ClinicalObservation;
use crate::agents::patient_agent::{PatientAgent, PatientParams};
use crate::agents::nurse_agent::NurseAgent;
use crate::agents::bed_agent::BedAgent;
use crate::agents::physician_agent::PhysicianAgent;
use crate::agents::infection_control::{InfectionControlAgent, SprtDecision};
use crate::agents::pathogen::{
    transmission_event, resistance_mutation_event, escalate_resistance,
};
use crate::agents::types::TreatmentAction;
use crate::decision_rules::social_learning::{SocialLearningParams, social_learning_from_mean};
use super::hospital_params::{HospitalParams, HospitalStats, MAX_PATIENTS, FIELDS_PER_PATIENT};
use super::hospital_buffers::write_patient_soa;

// ─── HospitalSimEngine ────────────────────────────────────────────────────────

pub struct HospitalSimEngine {
    pub patients:   Vec<PatientAgent>,
    pub nurses:     Vec<NurseAgent>,
    pub beds:       Vec<BedAgent>,
    pub physicians: Vec<PhysicianAgent>,
    pub ica:        InfectionControlAgent,
    pub rng:        Mulberry32,
    pub tick:       u64,
    pub params:     HospitalParams,

    // ── Internal derived state ─────────────────────────────────────────────────
    dtmc:              PatientDTMC,
    dtmc_matrix:       Vec<f32>,
    arriv_process:     NhppArrivalProcess,
    #[allow(dead_code)]
    wbc_ou:            OrnsteinUhlenbeck,
    #[allow(dead_code)]
    temp_ou:           OrnsteinUhlenbeck,
    pk:                OneCompartmentPk,
    social_params:     SocialLearningParams,

    /// Ticks until the next scheduled patient arrival.
    next_arrival_in:   u64,
    /// Running patient ID counter.
    next_patient_id:   u32,
    /// Cumulative HAI count.
    pub cumulative_hai: u32,
    /// HAI in the current tick.
    new_hai_this_tick:  u32,
    /// HH compliance events this tick.
    hh_yes:            u32,
    hh_total:          u32,
    /// Whether outbreak was signalled this tick.
    pub outbreak_signal: bool,

    /// Flat SoA patient buffer (MAX_PATIENTS × FIELDS_PER_PATIENT f32s).
    patient_buf: Vec<f32>,
}

impl HospitalSimEngine {
    pub fn new(params: HospitalParams, seed: u32) -> Self {
        let mut rng = Mulberry32::new(seed);

        let dtmc = PatientDTMC { n: 7 };
        let dtmc_matrix = build_hai_dtmc_matrix(
            params.dtmc_lambda, params.dtmc_alpha, params.dtmc_delta_e,
            params.dtmc_beta,   params.dtmc_gamma, params.dtmc_eta,
            params.dtmc_psi,    params.dtmc_rho,   params.dtmc_phi,
            params.dtmc_theta,
        );

        // Arrival process using params base rate with NHAMCS-calibrated time factors.
        let arriv_process = NhppArrivalProcess::with_base_rate(params.base_arrival_rate);

        // Sample initial next-arrival gap.
        let next_arrival_in = arriv_process
            .next_arrival_delta(0, params.ticks_per_hour, &mut rng);

        // Nurse roster: spread across beds.
        let n_beds = params.icu_beds + params.ward_beds;
        let nurses: Vec<NurseAgent> = (0..params.n_nurses)
            .map(|i| {
                let room = (i as u16) % n_beds.max(1) as u16;
                let compliance = 0.55 + rng.next_f32() * 0.30; // 55–85%
                NurseAgent::new(i as u32, compliance, room, &mut rng)
            })
            .collect();

        // Bed roster.
        let beds: Vec<BedAgent> = (0..(params.icu_beds + params.ward_beds))
            .map(|i| {
                let dist = 1.0 + rng.next_f32() * 4.0;
                BedAgent::new(i as u16, (i as u16) / 4, dist)
            })
            .collect();

        // Physician roster.
        let physicians: Vec<PhysicianAgent> = (0..params.n_physicians)
            .map(|i| {
                let compliance = 0.60 + rng.next_f32() * 0.25;
                PhysicianAgent::new(i as u32, 0, compliance)
            })
            .collect();

        let ica = InfectionControlAgent::new(0, 0);

        Self {
            patients:          Vec::with_capacity(MAX_PATIENTS),
            nurses,
            beds,
            physicians,
            ica,
            rng,
            tick:              0,
            params,
            dtmc,
            dtmc_matrix,
            arriv_process,
            wbc_ou:            OrnsteinUhlenbeck::wbc(),
            temp_ou:           OrnsteinUhlenbeck::temperature(),
            pk:                OneCompartmentPk::antibiotic_iv(),
            social_params:     SocialLearningParams::default(),
            next_arrival_in,
            next_patient_id:   0,
            cumulative_hai:    0,
            new_hai_this_tick: 0,
            hh_yes:            0,
            hh_total:          0,
            outbreak_signal:   false,
            patient_buf:       vec![0.0f32; MAX_PATIENTS * FIELDS_PER_PATIENT],
        }
    }

    // ── Tick ─────────────────────────────────────────────────────────────────

    pub fn step(&mut self) {
        self.new_hai_this_tick = 0;
        self.hh_yes            = 0;
        self.hh_total          = 0;
        self.outbreak_signal   = false;

        // 1. Patient arrivals ─────────────────────────────────────────────────
        self.process_arrivals();

        // 2-4. Nurse contact loop ─────────────────────────────────────────────
        self.nurse_contact_loop();

        // 5. Patient biomarker step ────────────────────────────────────────────
        self.update_biomarkers();

        // 6. DTMC disease-state transitions ───────────────────────────────────
        self.update_disease_states();

        // 7-8. Physician deliberation + treatment actions ─────────────────────
        self.physician_round();

        // 9. Resistance mutations ──────────────────────────────────────────────
        self.check_resistance_mutations();

        // 10. Discharge eligible patients ─────────────────────────────────────
        self.process_discharges();

        // 11. SPRT outbreak detection ─────────────────────────────────────────
        self.update_sprt();

        // 12. Social learning ─────────────────────────────────────────────────
        if self.tick.is_multiple_of(self.params.social_learning_period as u64) {
            self.run_social_learning();
        }

        self.tick += 1;
    }

    // ── 1. Patient arrivals ───────────────────────────────────────────────────

    fn process_arrivals(&mut self) {
        while self.next_arrival_in == 0 && self.patients.len() < MAX_PATIENTS {
            let patient_params = PatientParams {
                ticks_per_hour:              self.params.ticks_per_hour,
                admission_colonization_rate: self.params.admission_colonization_rate,
                cox:                         self.params.cox,
            };
            let mut p = PatientAgent::new_icu(
                self.next_patient_id,
                self.tick,
                &mut self.rng,
                &patient_params,
            );

            // Assign to least-loaded nurse (if any).
            let nurse_id = self.least_loaded_nurse_id();
            if let Some(nid) = nurse_id {
                p.primary_nurse_id = nid;
                if let Some(n) = self.nurses.iter_mut().find(|n| n.id == nid) {
                    n.assign_patient(p.id);
                }
            }
            // Assign to first free bed.
            if let Some(bed) = self.beds.iter_mut().find(|b| !b.is_occupied()) {
                p.bed_id  = bed.id;
                p.room_id = bed.room_id;
                bed.assign_patient(p.id);
            }
            // Assign to least-loaded physician.
            let phys_id = self.least_loaded_physician_id();
            if let Some(pid) = phys_id {
                p.attending_id = pid;
                if let Some(ph) = self.physicians.iter_mut().find(|ph| ph.id == pid) {
                    ph.assign_patient(p.id);
                }
            }
            // Register with infection control.
            self.ica.register_patient(p.id);

            self.next_patient_id += 1;
            self.patients.push(p);

            // Schedule next arrival.
            self.next_arrival_in = self.arriv_process
                .next_arrival_delta(self.tick, self.params.ticks_per_hour, &mut self.rng);
        }
        if self.next_arrival_in > 0 {
            self.next_arrival_in -= 1;
        }
    }

    // ── 2-4. Nurse contact + indirect exposure ────────────────────────────────

    fn nurse_contact_loop(&mut self) {
        // Iterate nurses; for each nurse performing bedside contact:
        //   a. HH compliance draw.
        //   b. Direct-contact transmission from infected/colonised patient.
        //   c. Contaminate bed surface.
        //   d. Indirect contact: susceptible patient exposed via contaminated bed.
        for nurse in self.nurses.iter_mut() {
            let contact_event = nurse.advance_activity(&mut self.rng);
            nurse.update_fatigue(&mut self.rng);
            nurse.tick_audit_clock();
            self.hh_total += 1;

            let hh = nurse.hand_hygiene_compliance(
                self.params.mean_dispenser_dist,
                &self.params.nurse,
                &mut self.rng,
            );
            if hh { self.hh_yes += 1; }

            if !contact_event { continue; }

            // Find the nurse's first patient (simplified: check patient_ids[0]).
            let patient_id = if nurse.n_patients > 0 { nurse.patient_ids[0] } else { continue };

            // Direct-contact: source patient → nurse hands → target patient.
            let source_idx = self.patients.iter().position(|p| p.id == patient_id);
            if let Some(si) = source_idx {
                let source_is_infectious = matches!(
                    self.patients[si].disease_state,
                    DiseaseState::Colonized | DiseaseState::Infected
                );
                if source_is_infectious && !hh {
                    let dose = self.patients[si].pathogen_load;
                    // Find another susceptible patient to potentially infect.
                    // (Round-robin: try the next patient in nurse's list.)
                    let target_id = if nurse.n_patients > 1 { nurse.patient_ids[1] } else { patient_id };
                    if let Some(ti) = self.patients.iter().position(|p| p.id == target_id) {
                        if ti != si && self.patients[ti].disease_state == DiseaseState::Susceptible {
                            let susc_mult = self.patients[ti].susceptibility_multiplier();
                            let effective_dose = dose * susc_mult;
                            if transmission_event(effective_dose, false, &self.params.pathogen, &mut self.rng) {
                                self.patients[ti].disease_state  = DiseaseState::Exposed;
                                self.patients[ti].pathogen_load  = 0.1;
                                self.new_hai_this_tick += 1;
                                self.cumulative_hai    += 1;
                            }
                        }
                    }
                }

                // Contaminate the patient's bed.
                let bed_id = self.patients[si].bed_id;
                if let Some(bed) = self.beds.iter_mut().find(|b| b.id == bed_id) {
                    if source_is_infectious {
                        bed.contaminate(0.05 * self.patients[si].pathogen_load);
                    }
                    // Indirect contact: patient on this bed exposed to surface contamination.
                    if bed.indirect_contact_event(hh, &mut self.rng) {
                        let susc_mult = self.patients[si].susceptibility_multiplier();
                        if self.patients[si].disease_state == DiseaseState::Susceptible
                            && self.rng.next_f32() < susc_mult * bed.contamination
                        {
                            self.patients[si].disease_state  = DiseaseState::Exposed;
                            self.patients[si].pathogen_load  = 0.05;
                            self.new_hai_this_tick += 1;
                            self.cumulative_hai    += 1;
                        }
                    }
                }
            }
        }

        // Passive bed contamination decay each tick.
        for bed in &mut self.beds {
            bed.decay_contamination();
        }
    }

    // ── 5. Biomarker updates ──────────────────────────────────────────────────

    fn update_biomarkers(&mut self) {
        const DT: f32 = 1.0; // one tick
        for p in &mut self.patients {
            let wbc_mu = match p.disease_state {
                DiseaseState::Infected => 14.0,
                DiseaseState::Colonized => 9.0,
                _ => 7.5,
            };
            let ou_wbc  = OrnsteinUhlenbeck { theta: 0.10, mu: wbc_mu,  sigma: 0.30 };
            let ou_temp = OrnsteinUhlenbeck { theta: 0.15, mu: match p.disease_state {
                DiseaseState::Infected => 38.5,
                _ => 37.0,
            }, sigma: 0.05 };

            p.wbc         = ou_wbc.step_clamped(p.wbc,         DT, 0.5,  25.0, &mut self.rng);
            p.temperature = ou_temp.step_clamped(p.temperature, DT, 35.0, 41.0, &mut self.rng);

            // PK step: antibiotic concentration.
            let dose = if matches!(p.current_treatment,
                TreatmentAction::StartBroadSpectrum | TreatmentAction::StartNarrowSpectrum)
            {
                let efficacy = crate::agents::pathogen::antibiotic_efficacy(
                    p.resistance_profile,
                    crate::agents::types::AntibioticClass::Vancomycin,
                );
                50.0 * efficacy
            } else {
                0.0
            };
            p.drug_conc = self.pk.step(p.drug_conc, DT, dose);

            // Track antibiotic exposure.
            if p.drug_conc > 0.1 {
                p.antibiotic_days = p.antibiotic_days.saturating_add(1);
            }
        }
    }

    // ── 6. DTMC disease-state transitions ─────────────────────────────────────

    fn update_disease_states(&mut self) {
        for p in &mut self.patients {
            let imm = p.immunosuppression_factor();
            // Scale transition matrix by immunosuppression inline.
            // (Simplified: use base matrix — full personalisation would rebuild per patient.)
            let _ = imm;
            let new_state = self.dtmc.sample_next_disease_state(
                p.disease_state,
                &self.dtmc_matrix,
                &mut self.rng,
            );
            // Track newly infected (Exposed → Infected or Colonized → Infected).
            if new_state == DiseaseState::Infected
                && p.disease_state != DiseaseState::Infected
                && p.disease_state != DiseaseState::Dead
            {
                // Newly progressed to clinical infection.
            }
            p.disease_state = new_state;

            // Update pathogen load based on disease state.
            p.pathogen_load = match p.disease_state {
                DiseaseState::Infected    => 0.8,
                DiseaseState::Colonized   => 0.3,
                DiseaseState::Treated     => (p.pathogen_load * 0.85).max(0.0),
                DiseaseState::Recovered   => 0.0,
                DiseaseState::Dead        => 0.0,
                _                         => p.pathogen_load * 0.95,
            };
        }
    }

    // ── 7-8. Physician deliberation ────────────────────────────────────────────

    fn physician_round(&mut self) {
        for phys in &mut self.physicians {
            phys.tick_round_clock();
        }

        for p in &mut self.patients {
            let phys_id = p.attending_id;
            if let Some(phys) = self.physicians.iter_mut().find(|ph| ph.id == phys_id) {
                // Feed clinical signals to pBDI.
                phys.observe_clinical(ClinicalObservation::Fever,
                    p.temperature > 38.3);
                phys.observe_clinical(ClinicalObservation::AbnormalWbc,
                    p.wbc < 4.0 || p.wbc > 11.0);
                phys.observe_clinical(ClinicalObservation::IncreasedVentSupport,
                    p.ventilated && p.disease_state == DiseaseState::Infected);

                let action = phys.deliberate();
                p.current_treatment = action;

                // Apply treatment actions.
                match action {
                    TreatmentAction::IsolatePrecautions => {
                        // Flag patient for contact precautions.
                        // (Physical isolation is modelled by reducing pathogen_load transmission.)
                        p.pathogen_load *= 0.5;
                    }
                    TreatmentAction::StartBroadSpectrum |
                    TreatmentAction::StartNarrowSpectrum => {
                        // Drug concentration handled in PK step.
                    }
                    TreatmentAction::OrderCulture => {
                        // Culture result informs pBDI next tick via PositiveCulture sign.
                        let culture_positive = p.disease_state == DiseaseState::Colonized
                            || p.disease_state == DiseaseState::Infected;
                        phys.observe_clinical(ClinicalObservation::PositiveCulture, culture_positive);
                        // Update infection control belief.
                        if culture_positive {
                            self.ica.update_positive(p.id, ClinicalObservation::PositiveCulture);
                        } else {
                            self.ica.update_negative(p.id, ClinicalObservation::PositiveCulture);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // ── 9. Resistance mutations ────────────────────────────────────────────────

    fn check_resistance_mutations(&mut self) {
        for p in &mut self.patients {
            if p.antibiotic_days > 0
                && resistance_mutation_event(p.antibiotic_days, &self.params.pathogen, &mut self.rng)
            {
                p.resistance_profile = escalate_resistance(p.resistance_profile);
            }
        }
    }

    // ── 10. Discharges ────────────────────────────────────────────────────────

    fn process_discharges(&mut self) {
        let tick = self.tick;
        let mut discharge_ids: Vec<u32> = Vec::new();

        // Minimum stay before a physician-ordered discharge is honoured:
        // at least 4 hours (ticks_per_hour × 4) to prevent flat-prior pBDI
        // from discharging newly-admitted patients immediately.
        let min_stay = (self.params.ticks_per_hour * 4) as u64;

        for p in &self.patients {
            let age_in_sim = tick.saturating_sub(p.arrival_tick);
            let explicit_discharge = p.current_treatment == TreatmentAction::Discharge
                && age_in_sim >= min_stay;
            if p.should_discharge(tick) || explicit_discharge
                || p.disease_state == DiseaseState::Dead
            {
                discharge_ids.push(p.id);
            }
        }

        for id in &discharge_ids {
            // Release bed.
            if let Some(bed) = self.beds.iter_mut().find(|b| b.patient_id == Some(*id)) {
                bed.release_patient();
                bed.routine_clean(0.85); // terminal clean on discharge.
            }
            // Remove from nurse assignment.
            for nurse in &mut self.nurses {
                for slot in nurse.patient_ids[..nurse.n_patients as usize].iter_mut() {
                    if *slot == *id {
                        *slot = u32::MAX;
                    }
                }
            }
            self.ica.remove_patient(*id);
            self.patients.retain(|p| p.id != *id);
        }
    }

    // ── 11. SPRT ──────────────────────────────────────────────────────────────

    fn update_sprt(&mut self) {
        let patient_days = self.patients.len() as f32 / self.params.ticks_per_hour as f32 / 24.0;
        let decision = self.ica.observe_cases(self.new_hai_this_tick, patient_days);
        if decision == SprtDecision::SignalOutbreak {
            self.outbreak_signal = true;
            self.ica.reset_sprt();   // restart after signalling
        }
        self.ica.tick_audit_clock();
        if self.ica.is_audit_due() {
            self.run_audit();
        }
    }

    // ── 12. Social learning ────────────────────────────────────────────────────

    fn run_social_learning(&mut self) {
        // Compute mean nurse compliance.
        let n = self.nurses.len();
        if n == 0 { return; }
        let mean_compliance: f32 = self.nurses.iter().map(|n| n.social.current_compliance).sum::<f32>() / n as f32;
        for nurse in &mut self.nurses {
            social_learning_from_mean(&mut nurse.social, mean_compliance, &self.social_params, &mut self.rng);
            nurse.base_compliance = nurse.social.current_compliance;
        }
    }

    // ── Audit helper ──────────────────────────────────────────────────────────

    fn run_audit(&mut self) {
        const TARGET: f32 = 0.85;
        for nurse in &mut self.nurses {
            nurse.receive_audit_feedback(TARGET);
        }
        for phys in &mut self.physicians {
            phys.receive_audit_feedback(TARGET);
        }
        self.ica.complete_audit();
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    pub fn write_patient_buffer(&mut self) -> &[f32] {
        let n = self.patients.len();
        write_patient_soa(&self.patients, &mut self.patient_buf[..n * FIELDS_PER_PATIENT]);
        &self.patient_buf[..n * FIELDS_PER_PATIENT]
    }

    pub fn stats(&self) -> HospitalStats {
        let n = self.patients.len();
        let (sum_wbc, sum_temp, sum_drug) = self.patients.iter().fold(
            (0.0f32, 0.0f32, 0.0f32),
            |(w, t, d), p| (w + p.wbc, t + p.temperature, d + p.drug_conc),
        );
        let inv_n = if n > 0 { 1.0 / n as f32 } else { 0.0 };

        let mut disease_state_counts = [0u32; 7];
        for p in &self.patients {
            disease_state_counts[p.disease_state as usize] += 1;
        }

        let icu_occ  = self.patients.iter().filter(|p| p.unit == crate::agents::types::HospitalUnit::Icu).count();
        let ward_occ = self.patients.iter().filter(|p| p.unit == crate::agents::types::HospitalUnit::Ward).count();

        let hh_rate = if self.hh_total > 0 {
            self.hh_yes as f32 / self.hh_total as f32
        } else { 0.0 };

        HospitalStats {
            tick:                 self.tick,
            census:               n,
            icu_occupancy:        icu_occ,
            ward_occupancy:       ward_occ,
            cumulative_hai:       self.cumulative_hai,
            new_hai_this_tick:    self.new_hai_this_tick,
            mean_wbc:             sum_wbc  * inv_n,
            mean_temperature:     sum_temp * inv_n,
            mean_drug_conc:       sum_drug * inv_n,
            sprt_log_lr:          self.ica.sprt.log_lr,
            outbreak_signal:      self.outbreak_signal,
            hh_compliance_rate:   hh_rate,
            disease_state_counts,
        }
    }

    pub fn reset(&mut self, seed: u32) {
        let params = self.params.clone();
        *self = Self::new(params, seed);
    }

    pub fn set_params(&mut self, params: HospitalParams) {
        self.dtmc_matrix = build_hai_dtmc_matrix(
            params.dtmc_lambda, params.dtmc_alpha, params.dtmc_delta_e,
            params.dtmc_beta,   params.dtmc_gamma, params.dtmc_eta,
            params.dtmc_psi,    params.dtmc_rho,   params.dtmc_phi,
            params.dtmc_theta,
        );
        self.arriv_process = NhppArrivalProcess::with_base_rate(params.base_arrival_rate);
        self.params = params;
    }

    pub fn patient_count(&self)      -> usize { self.patients.len()   }
    pub fn tick(&self)               -> u64   { self.tick             }
    pub fn cumulative_hai(&self)     -> u32   { self.cumulative_hai   }

    // ── Utilities ─────────────────────────────────────────────────────────────

    fn least_loaded_nurse_id(&self) -> Option<u32> {
        self.nurses.iter().min_by_key(|n| n.n_patients).map(|n| n.id)
    }

    fn least_loaded_physician_id(&self) -> Option<u32> {
        self.physicians.iter().min_by_key(|p| p.n_patients).map(|p| p.id)
    }
}
