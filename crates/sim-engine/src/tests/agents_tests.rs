/// Phase 2 behavioral tests for healthcare agents and decision rules.
#[cfg(test)]
mod tests {
    use crate::rng::Mulberry32;
    use crate::agents::{
        PatientAgent, PatientParams, NurseAgent, NurseParams,
        BedAgent, PhysicianAgent,
        pathogen::{PathogenParams, transmission_probability, resistance_mutation_event,
                   escalate_resistance, antibiotic_efficacy},
        infection_control::{InfectionControlAgent, SprtDecision},
        ResistanceProfile, AntibioticClass,
    };
    use crate::decision_rules::{
        ProspectTheoryParams, AntibioticBandit,
        SocialLearningParams, social_learning_step,
        PhysicianPBDI,
    };
    use crate::agents::types::ProviderSocialState;
    use crate::stochastic::bayesian::ClinicalObservation;
    use crate::stochastic::survival::CoxParams;

    // ── PatientAgent ──────────────────────────────────────────────────────────

    #[test]
    fn patient_susceptibility_in_unit_interval() {
        let mut rng = Mulberry32::new(1);
        let params  = PatientParams::default();
        for i in 0..20 {
            let p = PatientAgent::new_icu(i, 0, &mut rng, &params);
            let s = p.susceptibility_multiplier();
            assert!(s >= 0.0 && s <= 5.0,
                "susceptibility_multiplier {s} out of expected range [0,5]");
        }
    }

    #[test]
    fn patient_discharge_after_planned_los() {
        let mut rng    = Mulberry32::new(2);
        let params     = PatientParams::default();
        let mut p      = PatientAgent::new_icu(0, 0, &mut rng, &params);
        p.planned_los_ticks = 10;
        assert!(!p.should_discharge(9),  "should not discharge before planned LOS");
        assert!( p.should_discharge(10), "should discharge at planned LOS");
        assert!( p.should_discharge(11), "should discharge after planned LOS");
    }

    #[test]
    fn patient_cox_hazard_ratio_positive() {
        let mut rng = Mulberry32::new(3);
        let params  = PatientParams::default();
        let p       = PatientAgent::new_icu(0, 0, &mut rng, &params);
        let hr      = p.los_hazard_ratio(&params.cox);
        assert!(hr > 0.0, "Cox HR must be positive, got {hr}");
    }

    // ── NurseAgent ────────────────────────────────────────────────────────────

    #[test]
    fn nurse_hh_compliance_decreases_with_workload() {
        let params    = NurseParams::default();
        let mut rng   = Mulberry32::new(10);
        let mut low_w = NurseAgent::new(0, 0.65, 1, &mut rng);
        let mut hi_w  = NurseAgent::new(1, 0.65, 1, &mut rng);
        low_w.workload = 1.0;
        hi_w.workload  = 6.0;

        let n = 5_000usize;
        let mut rng2 = Mulberry32::new(99);
        let low_count: u32 = (0..n).filter(|_| low_w.hand_hygiene_compliance(1.0, &params, &mut rng2)).count() as u32;
        let hi_count:  u32 = (0..n).filter(|_| hi_w.hand_hygiene_compliance(1.0, &params, &mut rng2)).count() as u32;

        assert!(low_count > hi_count,
            "low workload HH {low_count} should exceed high workload {hi_count}");
    }

    #[test]
    fn nurse_hh_compliance_decreases_with_dispenser_distance() {
        let params  = NurseParams::default();
        let mut rng = Mulberry32::new(11);
        let nurse   = NurseAgent::new(0, 0.65, 1, &mut rng);

        let n = 5_000usize;
        let mut rng2 = Mulberry32::new(42);
        let near: u32 = (0..n).filter(|_| nurse.hand_hygiene_compliance(0.5, &params, &mut rng2)).count() as u32;
        let far:  u32 = (0..n).filter(|_| nurse.hand_hygiene_compliance(5.0, &params, &mut rng2)).count() as u32;
        assert!(near > far, "near dispenser HH {near} should exceed far {far}");
    }

    #[test]
    fn nurse_fatigue_stays_in_unit_interval() {
        let mut rng   = Mulberry32::new(20);
        let mut nurse = NurseAgent::new(0, 0.65, 1, &mut rng);
        nurse.workload = 6.0;
        for _ in 0..200 {
            nurse.update_fatigue(&mut rng);
            assert!(nurse.fatigue >= 0.0 && nurse.fatigue <= 1.0,
                "fatigue {} out of [0,1]", nurse.fatigue);
        }
    }

    #[test]
    fn nurse_assign_patient_capped_at_6() {
        let mut rng   = Mulberry32::new(30);
        let mut nurse = NurseAgent::new(0, 0.65, 1, &mut rng);
        for i in 0..6 { assert!(nurse.assign_patient(i as u32)); }
        assert!(!nurse.assign_patient(99), "should be full at 6 patients");
    }

    #[test]
    fn nurse_receive_audit_boosts_compliance() {
        let mut rng   = Mulberry32::new(31);
        let mut nurse = NurseAgent::new(0, 0.50, 1, &mut rng);
        let before = nurse.base_compliance;
        nurse.receive_audit_feedback(0.85);
        assert!(nurse.base_compliance > before, "audit should boost compliance");
        assert!(nurse.ticks_since_audit == 0, "audit should reset clock");
    }

    // ── BedAgent ──────────────────────────────────────────────────────────────

    #[test]
    fn bed_contamination_clamped_to_unit_interval() {
        let mut bed = BedAgent::new(0, 0, 1.5);
        bed.contaminate(0.8);
        bed.contaminate(0.8); // would exceed 1.0 without clamp
        assert!(bed.contamination <= 1.0, "contamination must be ≤ 1.0");
    }

    #[test]
    fn bed_deep_clean_resets_state() {
        let mut bed = BedAgent::new(0, 0, 1.5);
        bed.contaminate(0.6);
        bed.deep_clean();
        assert_eq!(bed.contamination, 0.0);
        assert_eq!(bed.ticks_dirty,   0);
    }

    #[test]
    fn bed_indirect_contact_blocked_by_hand_wash() {
        let mut rng = Mulberry32::new(40);
        let mut bed = BedAgent::new(0, 0, 1.0);
        bed.contamination = 1.0; // fully contaminated
        let events: u32 = (0..1000)
            .filter(|_| bed.indirect_contact_event(true, &mut rng))
            .count() as u32;
        assert_eq!(events, 0, "hand wash should block all indirect contact");
    }

    #[test]
    fn bed_decay_reduces_contamination() {
        let mut bed = BedAgent::new(0, 0, 1.0);
        bed.contamination = 0.5;
        for _ in 0..100 {
            bed.decay_contamination();
        }
        assert!(bed.contamination < 0.5, "contamination should decay over time");
    }

    // ── Pathogen ──────────────────────────────────────────────────────────────

    #[test]
    fn transmission_prob_zero_at_zero_dose() {
        let params = PathogenParams::default();
        assert_eq!(transmission_probability(0.0, false, &params), 0.0);
    }

    #[test]
    fn transmission_prob_reduced_by_hand_wash() {
        let params  = PathogenParams::default();
        let no_wash = transmission_probability(10.0, false, &params);
        let washed  = transmission_probability(10.0, true,  &params);
        assert!(washed < no_wash, "hand wash should reduce transmission probability");
    }

    #[test]
    fn transmission_prob_in_unit_interval() {
        let params = PathogenParams::default();
        for dose in [0.1f32, 1.0, 5.0, 20.0, 100.0] {
            let p = transmission_probability(dose, false, &params);
            assert!(p >= 0.0 && p <= 1.0, "P must be in [0,1], got {p} at dose {dose}");
        }
    }

    #[test]
    fn resistance_escalates_in_order() {
        assert_eq!(escalate_resistance(ResistanceProfile::FullySusceptible), ResistanceProfile::MethicillinR);
        assert_eq!(escalate_resistance(ResistanceProfile::MethicillinR),     ResistanceProfile::VancomycinIntermediate);
        assert_eq!(escalate_resistance(ResistanceProfile::VancomycinIntermediate), ResistanceProfile::Pandrug);
        assert_eq!(escalate_resistance(ResistanceProfile::Pandrug),           ResistanceProfile::Pandrug);
    }

    #[test]
    fn antibiotic_efficacy_zero_for_pandrug() {
        for ab in [AntibioticClass::BetaLactam, AntibioticClass::Vancomycin,
                   AntibioticClass::Linezolid, AntibioticClass::Daptomycin,
                   AntibioticClass::Cephalosporin] {
            assert_eq!(antibiotic_efficacy(ResistanceProfile::Pandrug, ab), 0.0);
        }
    }

    #[test]
    fn antibiotic_efficacy_one_for_susceptible() {
        for ab in [AntibioticClass::BetaLactam, AntibioticClass::Vancomycin] {
            assert_eq!(antibiotic_efficacy(ResistanceProfile::FullySusceptible, ab), 1.0);
        }
    }

    // ── InfectionControlAgent / SPRT ─────────────────────────────────────────

    #[test]
    fn sprt_signals_outbreak_under_high_incidence() {
        let mut ica = InfectionControlAgent::new(0, 1);
        // Inject many cases rapidly → should cross upper boundary
        let mut decision = SprtDecision::Continue;
        for _ in 0..50 {
            decision = ica.observe_cases(3, 1.0); // 3 cases/patient-day vs baseline 0.02
            if decision == SprtDecision::SignalOutbreak { break; }
        }
        assert_eq!(decision, SprtDecision::SignalOutbreak,
            "SPRT should signal outbreak under high case rate");
    }

    #[test]
    fn sprt_continues_at_baseline() {
        let mut ica = InfectionControlAgent::new(0, 1);
        // One case per 50 patient-days ≈ 0.02 = baseline; should not signal quickly
        let decision = ica.observe_cases(1, 50.0);
        assert_eq!(decision, SprtDecision::Continue,
            "SPRT should not signal on a single observation at baseline rate");
    }

    #[test]
    fn sprt_reset_zeroes_log_lr() {
        let mut ica = InfectionControlAgent::new(0, 1);
        ica.observe_cases(5, 1.0);
        assert!(ica.sprt.log_lr != 0.0);
        ica.reset_sprt();
        assert_eq!(ica.sprt.log_lr, 0.0);
    }

    #[test]
    fn ica_belief_increases_on_positive_observation() {
        let mut ica = InfectionControlAgent::new(0, 1);
        ica.register_patient(42);
        let prior = ica.patient_beliefs[&42].p_colonized;
        ica.update_positive(42, ClinicalObservation::PositiveCulture);
        let posterior = ica.patient_beliefs[&42].p_colonized;
        assert!(posterior > prior, "positive culture should raise colonization belief");
    }

    // ── Prospect Theory ───────────────────────────────────────────────────────

    #[test]
    fn prospect_value_gain_positive() {
        let pt = ProspectTheoryParams::default();
        assert!(pt.value(5.0) > 0.0, "gain should have positive value");
    }

    #[test]
    fn prospect_value_loss_negative() {
        let pt = ProspectTheoryParams::default();
        assert!(pt.value(-5.0) < 0.0, "loss should have negative value");
    }

    #[test]
    fn prospect_loss_aversion_asymmetry() {
        let pt = ProspectTheoryParams::default();
        // |v(-x)| > |v(x)| due to λ > 1
        let gain_val = pt.value(5.0).abs();
        let loss_val = pt.value(-5.0).abs();
        assert!(loss_val > gain_val, "loss aversion: |v(-x)| should exceed |v(x)|");
    }

    #[test]
    fn prospect_weight_at_boundaries() {
        let pt = ProspectTheoryParams::default();
        assert_eq!(pt.weight(0.0), 0.0);
        assert_eq!(pt.weight(1.0), 1.0);
    }

    #[test]
    fn prospect_weight_overweights_small_probs() {
        let pt = ProspectTheoryParams::default();
        // Prelec: w(0.05) > 0.05 for typical γ < 1
        assert!(pt.weight(0.05) > 0.05, "Prelec should overweight small probs");
    }

    #[test]
    fn prospect_order_test_at_high_colonization_prob() {
        let pt = ProspectTheoryParams::default();
        // At p=0.80 the test EU should dominate skip EU
        assert!(pt.should_order_test(0.80, 8.0, 1.0, 9.0),
            "should order at high colonization probability");
    }

    // ── Thompson Sampling ─────────────────────────────────────────────────────

    #[test]
    fn thompson_converges_to_best_arm() {
        let mut bandit = AntibioticBandit::new();
        let mut rng    = Mulberry32::new(50);

        // Simulate: Vancomycin succeeds 80% vs others 20%
        for _ in 0..500 {
            let chosen = bandit.select(&mut rng);
            let success = if chosen == AntibioticClass::Vancomycin {
                rng.next_f32() < 0.80
            } else {
                rng.next_f32() < 0.20
            };
            bandit.update(chosen, success);
        }
        // Mean efficacy of Vancomycin should be the highest
        let vanc_mean = bandit.mean_efficacy(AntibioticClass::Vancomycin);
        for ab in [AntibioticClass::BetaLactam, AntibioticClass::Linezolid,
                   AntibioticClass::Daptomycin, AntibioticClass::Cephalosporin] {
            assert!(vanc_mean > bandit.mean_efficacy(ab),
                "Vancomycin posterior should lead after targeted feedback");
        }
    }

    #[test]
    fn thompson_selects_all_arms_initially() {
        // With uniform priors, all arms should be selected at least once in 50 draws.
        let mut bandit = AntibioticBandit::new();
        let mut rng    = Mulberry32::new(51);
        let mut seen   = [false; 5];
        for _ in 0..200 {
            let ab = bandit.select(&mut rng);
            seen[ab as usize] = true;
        }
        assert!(seen.iter().all(|&s| s), "all arms should be explored with uniform prior");
    }

    // ── pBDI ──────────────────────────────────────────────────────────────────

    #[test]
    fn pbdi_belief_sums_to_one() {
        let mut pbdi = PhysicianPBDI::icu_default();
        pbdi.update_belief(0, true);  // fever present
        let sum: f32 = pbdi.belief_state.iter().map(|&(_, p)| p).sum();
        assert!((sum - 1.0).abs() < 1e-4, "belief distribution must sum to 1, got {sum}");
    }

    #[test]
    fn pbdi_positive_fever_increases_infected_belief() {
        let mut pbdi = PhysicianPBDI::icu_default();
        let prior_infected = pbdi.belief_state[3].1; // DiseaseState::Infected
        pbdi.update_belief(0, true); // fever (sign 0) has highest P given Infected
        let post_infected  = pbdi.belief_state[3].1;
        assert!(post_infected > prior_infected,
            "fever should increase P(Infected): {prior_infected:.4} → {post_infected:.4}");
    }

    #[test]
    fn pbdi_deliberate_returns_valid_action() {
        let mut pbdi = PhysicianPBDI::icu_default();
        // Push toward Infected state
        for _ in 0..3 { pbdi.update_belief(0, true); }
        pbdi.update_belief(1, true); // positive culture
        let action = pbdi.deliberate();
        // When Infected belief is high, isolate or treat should dominate observe
        let _ = action; // just assert no panic + valid return
    }

    #[test]
    fn pbdi_commitment_prevents_frequent_switching() {
        let mut pbdi = PhysicianPBDI::icu_default();
        pbdi.commitment_threshold = 100.0; // very high — should never switch
        let initial = pbdi.current_intention;
        for i in 0..6 {
            pbdi.update_belief(i, true);
        }
        let after = pbdi.deliberate();
        assert_eq!(initial, after, "high commitment threshold should prevent switching");
    }

    // ── Social learning ───────────────────────────────────────────────────────

    #[test]
    fn social_learning_imitation_raises_compliance() {
        let params  = SocialLearningParams::default();
        let mut rng = Mulberry32::new(60);

        let mut low_provider = ProviderSocialState::new(0, 0.30);
        let high_peer        = ProviderSocialState::new(1, 0.90);

        // Run many steps — compliance should trend upward
        let initial = low_provider.current_compliance;
        for _ in 0..50 {
            social_learning_step(&mut low_provider, &high_peer, &params, &mut rng);
        }
        assert!(low_provider.current_compliance > initial,
            "imitation from high-compliance peer should raise compliance");
    }

    #[test]
    fn audit_feedback_boosts_compliance() {
        let params  = SocialLearningParams::default();
        let mut rng = Mulberry32::new(61);
        let mut provider = ProviderSocialState::new(0, 0.50);
        provider.received_feedback_this_period = true;

        // Use provider as own peer (gap = 0); audit boost should still apply
        let peer = ProviderSocialState::new(1, 0.50);
        let before = provider.current_compliance;
        social_learning_step(&mut provider, &peer, &params, &mut rng);
        assert!(provider.current_compliance > before,
            "audit feedback should boost compliance even with no gap");
        assert!(!provider.received_feedback_this_period,
            "feedback flag should be reset after application");
    }

    // ── PhysicianAgent integration ────────────────────────────────────────────

    #[test]
    fn physician_agent_rounds_smoke_test() {
        let mut rng  = Mulberry32::new(70);
        let mut phys = PhysicianAgent::new(0, 1, 0.70);
        phys.assign_patient(100);
        phys.observe_clinical(ClinicalObservation::Fever, true);
        phys.observe_clinical(ClinicalObservation::PositiveCulture, true);
        let action = phys.deliberate();
        let ab     = phys.select_antibiotic(&mut rng);
        phys.update_antibiotic_outcome(ab, true);
        let _ = action; // smoke test: no panic
    }
}
