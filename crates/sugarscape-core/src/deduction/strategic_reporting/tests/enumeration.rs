use super::super::*;
fn valid_config() -> Config {
    Config::standard(
        Probability {
            numerator: 4,
            denominator: 5,
        },
        Probability {
            numerator: 3,
            denominator: 4,
        },
        UtilityTable::opposed(),
    )
}
fn listener(algorithm: Listener) -> FrozenListener {
    FrozenListener {
        algorithm,
        assumed_copy_prior: Probability {
            numerator: 3,
            denominator: 4,
        },
    }
}
#[test]
fn normalization_and_endpoints() {
    for q in [0, 1, 2] {
        for rho in [0, 1, 2] {
            let mut c = valid_config();
            c.accuracy = Probability {
                numerator: q,
                denominator: 2,
            };
            c.fixed_copy_prior = Probability {
                numerator: rho,
                denominator: 2,
            };
            let d = enumerate(&c).unwrap();
            for p in [
                Policy::copy(),
                Policy::invert(),
                Policy::positive(),
                Policy::negative(),
                Policy::calibration_copy_live_invert(),
            ] {
                let h = histories(&d, &p).unwrap();
                assert_eq!(h.iter().map(|h| h.total_mass).sum::<u64>(), d.denominator());
                assert_eq!(
                    h.iter().map(|h| h.true_mass).sum::<u64>(),
                    d.denominator() / 2
                );
                let e = evaluate(&d, &p, &listener(Listener::Passive)).unwrap();
                assert_eq!(e.payoff_numerator, 0);
                assert_eq!(e.utility_numerator, 0);
                assert_eq!(e.reference_regret_numerator, 0);
            }
        }
    }
    let mut c = valid_config();
    c.accuracy = Probability {
        numerator: 1,
        denominator: 1,
    };
    c.fixed_copy_prior = c.accuracy.clone();
    let d = enumerate(&c).unwrap();
    let e = evaluate(&d, &Policy::copy(), &listener(Listener::Credulous)).unwrap();
    assert_eq!(2 * e.payoff_numerator, e.denominator as i64);
    assert_eq!(e.utility_numerator, -e.payoff_numerator);
    assert!(evaluate(&d, &Policy::invert(), &listener(Listener::Credulous)).is_err());
    assert!(TrainingPanel::new(&c, vec![listener(Listener::Credulous)]).is_err());
}
#[test]
fn invalid_configs_and_policies_fail_before_evaluation() {
    let mut c = valid_config();
    c.accuracy.denominator = 0;
    assert!(enumerate(&c).is_err());
    let d = enumerate(&valid_config()).unwrap();
    assert!(evaluate(&d, &Policy { bits: 1 << 18 }, &listener(Listener::Passive)).is_err());
}
#[test]
fn exact_half_accuracy_ties_abstain() {
    let mut c = valid_config();
    c.accuracy = Probability {
        numerator: 1,
        denominator: 2,
    };
    let d = enumerate(&c).unwrap();
    for p in [Policy::copy(), Policy::invert(), Policy::positive()] {
        for l in [Listener::Bayesian, Listener::Credulous] {
            let e = evaluate(&d, &p, &listener(l)).unwrap();
            assert_eq!(e.payoff_numerator, 0);
            assert_eq!(e.optimal_numerator, 0);
            for h in e.histories.iter().filter(|h| h.total_mass > 0) {
                assert_eq!(h.reference_action, Some(DecisionAction::Abstain));
                assert_eq!(h.listener_action, Some(DecisionAction::Abstain));
            }
        }
    }
}
// Independently generated Python Fraction fixtures; script SHA-256 dac061c8ce428b998a4db9d6a190ab09139d99282a56a28938d92e55f5127006.
#[test]
fn independent_actual_policy_history_and_payoff_fixtures() {
    let d = enumerate(&valid_config()).unwrap();
    {
        // always_negative
        let policy = Policy::new(0).unwrap();
        let expected: [(u64, u64); 32] = [
            (1625, 400),
            (1625, 1225),
            (0, 0),
            (0, 0),
            (875, 475),
            (875, 400),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (875, 475),
            (875, 400),
            (0, 0),
            (0, 0),
            (1625, 400),
            (1625, 1225),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
        ];
        let h = histories(&d, &policy).unwrap();
        for (actual, (total, true_mass)) in h.iter().zip(expected) {
            assert_eq!((actual.total_mass, actual.true_mass), (total, true_mass));
        }
        let e = evaluate(&d, &policy, &listener(Listener::Bayesian)).unwrap();
        assert_eq!(e.payoff_numerator, 900);
        assert_eq!(e.regret_numerator, 900);
        assert_eq!(e.intervention_mass, 2500);
        assert_eq!(e.false_intervention_mass, 800);
        assert_eq!(e.missed_beneficial_intervention_mass, 3300);
        assert_eq!(e.optimal_numerator, 1800);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
        let e = evaluate(&d, &policy, &listener(Listener::Credulous)).unwrap();
        assert_eq!(e.payoff_numerator, 0);
        assert_eq!(e.regret_numerator, 1800);
        assert_eq!(e.intervention_mass, 0);
        assert_eq!(e.false_intervention_mass, 0);
        assert_eq!(e.missed_beneficial_intervention_mass, 5000);
        assert_eq!(e.optimal_numerator, 1800);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
    }
    {
        // always_positive
        let policy = Policy::new(262143).unwrap();
        let expected: [(u64, u64); 32] = [
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (1625, 400),
            (1625, 1225),
            (0, 0),
            (0, 0),
            (875, 475),
            (875, 400),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (0, 0),
            (875, 475),
            (875, 400),
            (0, 0),
            (0, 0),
            (1625, 400),
            (1625, 1225),
        ];
        let h = histories(&d, &policy).unwrap();
        for (actual, (total, true_mass)) in h.iter().zip(expected) {
            assert_eq!((actual.total_mass, actual.true_mass), (total, true_mass));
        }
        let e = evaluate(&d, &policy, &listener(Listener::Bayesian)).unwrap();
        assert_eq!(e.payoff_numerator, 1650);
        assert_eq!(e.regret_numerator, 150);
        assert_eq!(e.intervention_mass, 5000);
        assert_eq!(e.false_intervention_mass, 1675);
        assert_eq!(e.missed_beneficial_intervention_mass, 1675);
        assert_eq!(e.optimal_numerator, 1800);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
        let e = evaluate(&d, &policy, &listener(Listener::Credulous)).unwrap();
        assert_eq!(e.payoff_numerator, 1500);
        assert_eq!(e.regret_numerator, 300);
        assert_eq!(e.intervention_mass, 5000);
        assert_eq!(e.false_intervention_mass, 1750);
        assert_eq!(e.missed_beneficial_intervention_mass, 1750);
        assert_eq!(e.optimal_numerator, 1800);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
    }
    {
        // canonical_optimum
        let policy = Policy::new(81942).unwrap();
        let expected: [(u64, u64); 32] = [
            (452, 256),
            (848, 784),
            (848, 64),
            (452, 196),
            (368, 304),
            (332, 256),
            (332, 76),
            (368, 64),
            (113, 64),
            (212, 196),
            (212, 16),
            (113, 49),
            (92, 76),
            (83, 64),
            (83, 19),
            (92, 16),
            (92, 76),
            (83, 64),
            (83, 19),
            (92, 16),
            (113, 64),
            (212, 196),
            (212, 16),
            (113, 49),
            (368, 304),
            (332, 256),
            (332, 76),
            (368, 64),
            (452, 256),
            (848, 784),
            (848, 64),
            (452, 196),
        ];
        let h = histories(&d, &policy).unwrap();
        for (actual, (total, true_mass)) in h.iter().zip(expected) {
            assert_eq!((actual.total_mass, actual.true_mass), (total, true_mass));
        }
        let e = evaluate(&d, &policy, &listener(Listener::Bayesian)).unwrap();
        assert_eq!(e.payoff_numerator, -510);
        assert_eq!(e.regret_numerator, 3510);
        assert_eq!(e.intervention_mass, 3138);
        assert_eq!(e.false_intervention_mass, 1824);
        assert_eq!(e.missed_beneficial_intervention_mass, 3686);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
        let e = evaluate(&d, &policy, &listener(Listener::Credulous)).unwrap();
        assert_eq!(e.payoff_numerator, -750);
        assert_eq!(e.regret_numerator, 3750);
        assert_eq!(e.intervention_mass, 2050);
        assert_eq!(e.false_intervention_mass, 1400);
        assert_eq!(e.missed_beneficial_intervention_mass, 4350);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
    }
    {
        // copy
        let policy = Policy::new(174762).unwrap();
        let expected: [(u64, u64); 32] = [
            (848, 64),
            (452, 196),
            (452, 256),
            (848, 784),
            (332, 76),
            (368, 64),
            (368, 304),
            (332, 256),
            (212, 16),
            (113, 49),
            (113, 64),
            (212, 196),
            (83, 19),
            (92, 16),
            (92, 76),
            (83, 64),
            (83, 19),
            (92, 16),
            (92, 76),
            (83, 64),
            (212, 16),
            (113, 49),
            (113, 64),
            (212, 196),
            (332, 76),
            (368, 64),
            (368, 304),
            (332, 256),
            (848, 64),
            (452, 196),
            (452, 256),
            (848, 784),
        ];
        let h = histories(&d, &policy).unwrap();
        for (actual, (total, true_mass)) in h.iter().zip(expected) {
            assert_eq!((actual.total_mass, actual.true_mass), (total, true_mass));
        }
        let e = evaluate(&d, &policy, &listener(Listener::Bayesian)).unwrap();
        assert_eq!(e.payoff_numerator, 2520);
        assert_eq!(e.regret_numerator, 480);
        assert_eq!(e.intervention_mass, 3912);
        assert_eq!(e.false_intervention_mass, 696);
        assert_eq!(e.missed_beneficial_intervention_mass, 1784);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
        let e = evaluate(&d, &policy, &listener(Listener::Credulous)).unwrap();
        assert_eq!(e.payoff_numerator, 2250);
        assert_eq!(e.regret_numerator, 750);
        assert_eq!(e.intervention_mass, 2950);
        assert_eq!(e.false_intervention_mass, 350);
        assert_eq!(e.missed_beneficial_intervention_mass, 2400);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
    }
    {
        // copy_calibration_invert_live
        let policy = Policy::new(87382).unwrap();
        let expected: [(u64, u64); 32] = [
            (452, 256),
            (848, 784),
            (848, 64),
            (452, 196),
            (368, 304),
            (332, 256),
            (332, 76),
            (368, 64),
            (113, 64),
            (212, 196),
            (212, 16),
            (113, 49),
            (92, 76),
            (83, 64),
            (83, 19),
            (92, 16),
            (92, 76),
            (83, 64),
            (83, 19),
            (92, 16),
            (113, 64),
            (212, 196),
            (212, 16),
            (113, 49),
            (368, 304),
            (332, 256),
            (332, 76),
            (368, 64),
            (452, 256),
            (848, 784),
            (848, 64),
            (452, 196),
        ];
        let h = histories(&d, &policy).unwrap();
        for (actual, (total, true_mass)) in h.iter().zip(expected) {
            assert_eq!((actual.total_mass, actual.true_mass), (total, true_mass));
        }
        let e = evaluate(&d, &policy, &listener(Listener::Bayesian)).unwrap();
        assert_eq!(e.payoff_numerator, -510);
        assert_eq!(e.regret_numerator, 3510);
        assert_eq!(e.intervention_mass, 3138);
        assert_eq!(e.false_intervention_mass, 1824);
        assert_eq!(e.missed_beneficial_intervention_mass, 3686);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
        let e = evaluate(&d, &policy, &listener(Listener::Credulous)).unwrap();
        assert_eq!(e.payoff_numerator, -750);
        assert_eq!(e.regret_numerator, 3750);
        assert_eq!(e.intervention_mass, 2050);
        assert_eq!(e.false_intervention_mass, 1400);
        assert_eq!(e.missed_beneficial_intervention_mass, 4350);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
    }
    {
        // invert
        let policy = Policy::new(87381).unwrap();
        let expected: [(u64, u64); 32] = [
            (113, 64),
            (212, 196),
            (212, 16),
            (113, 49),
            (92, 76),
            (83, 64),
            (83, 19),
            (92, 16),
            (452, 256),
            (848, 784),
            (848, 64),
            (452, 196),
            (368, 304),
            (332, 256),
            (332, 76),
            (368, 64),
            (368, 304),
            (332, 256),
            (332, 76),
            (368, 64),
            (452, 256),
            (848, 784),
            (848, 64),
            (452, 196),
            (92, 76),
            (83, 64),
            (83, 19),
            (92, 16),
            (113, 64),
            (212, 196),
            (212, 16),
            (113, 49),
        ];
        let h = histories(&d, &policy).unwrap();
        for (actual, (total, true_mass)) in h.iter().zip(expected) {
            assert_eq!((actual.total_mass, actual.true_mass), (total, true_mass));
        }
        let e = evaluate(&d, &policy, &listener(Listener::Bayesian)).unwrap();
        assert_eq!(e.payoff_numerator, 1560);
        assert_eq!(e.regret_numerator, 1440);
        assert_eq!(e.intervention_mass, 3912);
        assert_eq!(e.false_intervention_mass, 1176);
        assert_eq!(e.missed_beneficial_intervention_mass, 2264);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
        let e = evaluate(&d, &policy, &listener(Listener::Credulous)).unwrap();
        assert_eq!(e.payoff_numerator, -750);
        assert_eq!(e.regret_numerator, 3750);
        assert_eq!(e.intervention_mass, 2050);
        assert_eq!(e.false_intervention_mass, 1400);
        assert_eq!(e.missed_beneficial_intervention_mass, 4350);
        assert_eq!(e.optimal_numerator, 3000);
        assert_eq!(e.reference_regret_numerator, 0);
        for h in &e.histories {
            if h.total_mass == 0 {
                assert!(h.reference_posterior.is_none());
                assert!(h.reference_action.is_none());
                assert!(h.listener_action.is_none());
            } else {
                let p = h.reference_posterior.as_ref().unwrap();
                assert_eq!(p.numerator * h.total_mass, h.true_mass * p.denominator);
                assert_eq!(h.reference_regret_numerator, 0);
            }
        }
    }
}
#[test]
fn independent_assumed_model_action_fixtures() {
    let c = valid_config();
    let expected = [
        false, false, false, true, false, false, true, true, false, true, false, true, true, false,
        false, false, true, false, false, false, false, true, false, true, false, false, true,
        true, false, false, false, true,
    ];
    for (index, intervene) in expected.into_iter().enumerate() {
        let view = DecisionObservation {
            rules: c.clone(),
            calibration_truth: index & 16 != 0,
            calibration_reports: [index & 8 != 0, index & 4 != 0],
            live_reports: [index & 2 != 0, index & 1 != 0],
        };
        assert_eq!(
            listener(Listener::Bayesian).decide(&view).unwrap() == DecisionAction::Intervene,
            intervene
        );
    }
    let expected = [
        false, false, false, true, false, false, false, true, false, false, false, true, false,
        false, false, true, false, false, false, true, false, false, false, true, false, false,
        false, true, false, false, false, true,
    ];
    for (index, intervene) in expected.into_iter().enumerate() {
        let view = DecisionObservation {
            rules: c.clone(),
            calibration_truth: index & 16 != 0,
            calibration_reports: [index & 8 != 0, index & 4 != 0],
            live_reports: [index & 2 != 0, index & 1 != 0],
        };
        assert_eq!(
            listener(Listener::Credulous).decide(&view).unwrap() == DecisionAction::Intervene,
            intervene
        );
    }
}
