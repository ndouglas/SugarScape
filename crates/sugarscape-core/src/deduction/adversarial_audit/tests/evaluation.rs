use super::super::*;
use crate::deduction::strategic_reporting::{
    self, Config, DecisionAction, Policy, Probability, UtilityTable,
};
use crate::deduction::strategy_inference::{self, Catalog, WeightedPolicy};
use serde_json::Value;

fn config(q: u16) -> Config {
    Config::standard(
        Probability {
            numerator: q,
            denominator: 5,
        },
        Probability {
            numerator: 3,
            denominator: 4,
        },
        UtilityTable::opposed(),
    )
}
fn controllers() -> [ControllerKind; 4] {
    [
        ControllerKind::StrategyUniform,
        ControllerKind::StrategyOptimizationInformed,
        ControllerKind::FixedOnly,
        ControllerKind::Passive,
    ]
}
fn fixture() -> Value {
    serde_json::from_str(include_str!("evaluation-reference.json")).unwrap()
}
fn score_of(actions: &FrozenActions, e: &AuditEvaluation) -> Score {
    Score {
        rules: actions.config().clone(),
        denominator: e.denominator,
        payoff_numerator: e.payoff_numerator,
        utility_numerator: e.utility_numerator,
    }
}
fn assert_ratio(n: i128, d: u64, expected: &Value) {
    assert_eq!(
        n * expected[1].as_i64().unwrap() as i128,
        expected[0].as_i64().unwrap() as i128 * i128::from(d)
    );
}
fn action_value(action: Option<DecisionAction>) -> Value {
    action
        .map(|a| Value::Bool(a == DecisionAction::Intervene))
        .unwrap_or(Value::Null)
}
fn assert_evaluation(e: &AuditEvaluation, row: &Value, names: &Value) {
    let serialized = serde_json::to_value(e).unwrap();
    let denominator = row["metrics"][3].as_u64().unwrap();
    for (name, expected) in names
        .as_array()
        .unwrap()
        .iter()
        .zip(row["metrics"].as_array().unwrap())
    {
        let actual = serialized[name.as_str().unwrap()].as_i64().unwrap();
        assert_eq!(
            i128::from(actual) * i128::from(denominator),
            expected.as_i64().unwrap() as i128 * i128::from(e.denominator),
            "metric {name}, controller {}, group {}",
            row["controller"],
            row["group"]
        );
    }
    assert_eq!(e.histories.len(), 32);
    for (i, (h, expected)) in e
        .histories
        .iter()
        .zip(row["histories"].as_array().unwrap())
        .enumerate()
    {
        assert_eq!(i, expected[0].as_u64().unwrap() as usize);
        assert_eq!(history_index(&h.observation), i);
        assert_eq!(
            serde_json::to_value([
                u8::from(h.observation.calibration_truth),
                u8::from(h.observation.calibration_reports[0]),
                u8::from(h.observation.calibration_reports[1]),
                u8::from(h.observation.live_reports[0]),
                u8::from(h.observation.live_reports[1])
            ])
            .unwrap(),
            expected[1]
        );
        for (actual, index) in [
            (h.total_mass as i64, 2),
            (h.true_mass as i64, 3),
            (h.regret_numerator, 7),
            (h.reference_regret_numerator, 8),
        ] {
            assert_eq!(
                i128::from(actual) * i128::from(denominator),
                expected[index].as_i64().unwrap() as i128 * i128::from(e.denominator),
                "history {i}"
            );
        }
        if let Some(p) = &h.reference_posterior {
            assert_ratio(i128::from(p.numerator), p.denominator, &expected[4]);
        } else {
            assert!(expected[4].is_null());
        }
        assert_eq!(action_value(h.reference_action), expected[5]);
        assert_eq!(action_value(h.listener_action), expected[6]);
    }
}

#[test]
fn fixed_only_has_the_published_constant_policy_benchmark() {
    // 9/50 was published in the approved spec before this implementation.
    let actions = FrozenActions::freeze(&config(4), ControllerKind::FixedOnly).unwrap();
    for policy in canonical_policies() {
        let s = score(&actions, &policy).unwrap();
        assert_eq!(s.payoff_numerator * 50, 9 * s.denominator as i64);
    }
}

#[test]
fn all_104_complete_evaluations_match_independent_fraction_reference() {
    let f = fixture();
    let mut count = 0;
    for env in f["environments"].as_array().unwrap() {
        let c = config(env["q"][0].as_u64().unwrap() as u16);
        let fixed = FrozenActions::freeze(&c, ControllerKind::FixedOnly).unwrap();
        let fixed_score = score(&fixed, &Policy::negative()).unwrap();
        assert_ratio(
            i128::from(fixed_score.payoff_numerator),
            fixed_score.denominator,
            &env["fixed_only_F"],
        );
        for row in env["evaluations"].as_array().unwrap() {
            let kind: ControllerKind = serde_json::from_value(row["controller"].clone()).unwrap();
            let actions = FrozenActions::freeze(&c, kind).unwrap();
            let e = if let Some(bits) = row["policy_bits"].as_u64() {
                evaluate_fixed(&actions, &Policy::new(bits as u32).unwrap()).unwrap()
            } else {
                let catalog = if row["mixture"] == "strategy_uniform" {
                    Catalog::uniform()
                } else {
                    Catalog::optimization_informed()
                };
                evaluate_mixture(&actions, &catalog).unwrap()
            };
            assert_evaluation(&e, row, &f["metric_names"]);
            if !row["guarantee_shortfall"].is_null() {
                let r = guarantee_shortfall(&fixed_score, &score_of(&actions, &e)).unwrap();
                assert_ratio(
                    i128::from(r.numerator),
                    r.denominator,
                    &row["guarantee_shortfall"],
                );
            }
            if !row["nominal_minus_fixed_only"].is_null() {
                let r = nominal_difference(&score_of(&actions, &e), &fixed_score).unwrap();
                assert_ratio(
                    i128::from(r.numerator),
                    r.denominator,
                    &row["nominal_minus_fixed_only"],
                );
            }
            count += 1;
        }
    }
    assert_eq!(count, 104);
}

#[test]
fn mixture_conditions_pooled_evidence_before_reference_maximization() {
    let actions = FrozenActions::freeze(&config(4), ControllerKind::StrategyUniform).unwrap();
    let catalog = Catalog::uniform();
    let e = evaluate_mixture(&actions, &catalog).unwrap();
    assert_eq!(e.optimal_numerator * 5000, 951 * e.denominator as i64);
    let policy_aware: i64 = catalog
        .entries()
        .iter()
        .map(|p| {
            evaluate_fixed(&actions, &p.policy)
                .unwrap()
                .optimal_numerator
                * i64::from(p.weight)
        })
        .sum();
    assert_eq!(policy_aware * 250, 63 * e.denominator as i64);
    assert!(e.optimal_numerator < policy_aware);
}

#[test]
fn constant_witnesses_and_fixed_only_invariance_prove_expected_minimax_bound() {
    for env in fixture()["environments"].as_array().unwrap() {
        let c = config(env["q"][0].as_u64().unwrap() as u16);
        let fixed = FrozenActions::freeze(&c, ControllerKind::FixedOnly).unwrap();
        let f = score(&fixed, &Policy::negative()).unwrap();
        assert_ratio(
            i128::from(f.payoff_numerator),
            f.denominator,
            &env["fixed_only_F"],
        );
        for kind in controllers() {
            let actions = FrozenActions::freeze(&c, kind).unwrap();
            for p in [Policy::positive(), Policy::negative()] {
                let e = evaluate_fixed(&actions, &p).unwrap();
                assert_eq!(e.optimal_numerator, f.payoff_numerator);
                assert!(e.payoff_numerator <= f.payoff_numerator);
            }
        }
        let passive = FrozenActions::freeze(&c, ControllerKind::Passive).unwrap();
        for p in canonical_policies() {
            assert_eq!(score(&fixed, &p).unwrap(), f);
            assert_eq!(score(&passive, &p).unwrap().payoff_numerator, 0);
        }
    }
}

#[test]
fn guarantee_shortfall_and_actual_policy_regret_are_distinct() {
    let c = config(4);
    let actions = FrozenActions::freeze(&c, ControllerKind::StrategyUniform).unwrap();
    let p = Policy::new(8321).unwrap();
    let e = evaluate_fixed(&actions, &p).unwrap();
    let fixed = score(
        &FrozenActions::freeze(&c, ControllerKind::FixedOnly).unwrap(),
        &p,
    )
    .unwrap();
    let r = guarantee_shortfall(&fixed, &score(&actions, &p).unwrap()).unwrap();
    assert_eq!(r.numerator * 125, 12 * r.denominator);
    assert_eq!(e.regret_numerator * 125, 24 * e.denominator as i64);
}

#[test]
fn comparisons_validate_rules_and_forged_scores_before_arithmetic() {
    let valid = Score {
        rules: config(4),
        denominator: 1 << 30,
        payoff_numerator: 1 << 29,
        utility_numerator: -(1 << 29),
    };
    let mut negative = valid.clone();
    negative.payoff_numerator = -(1 << 29);
    negative.utility_numerator = 1 << 29;
    let r = guarantee_shortfall(&valid, &negative).unwrap();
    assert_eq!(r.numerator, r.denominator);
    let r = nominal_difference(&negative, &valid).unwrap();
    assert_eq!(r.numerator, -(r.denominator as i64));
    assert!(guarantee_shortfall(&negative, &valid).is_err());
    let mut bad = valid.clone();
    bad.rules = config(3);
    assert!(guarantee_shortfall(&valid, &bad).is_err());
    assert!(nominal_difference(&valid, &bad).is_err());
    for (d, p, u) in [
        (0, 0, 0),
        ((1 << 30) + 1, 0, 0),
        (u64::MAX, 0, 0),
        (2, 2, -2),
        (2, 1, 1),
        (2, i64::MIN, i64::MAX),
        (2, i64::MAX, i64::MIN),
    ] {
        let bad = Score {
            rules: config(4),
            denominator: d,
            payoff_numerator: p,
            utility_numerator: u,
        };
        for (a, b) in [(&bad, &valid), (&valid, &bad)] {
            assert!(guarantee_shortfall(a, b).is_err());
            assert!(nominal_difference(a, b).is_err());
        }
    }
}

#[test]
fn phase_and_mass_identities_preserve_absent_zero_histories() {
    for q in [4, 3] {
        for kind in controllers() {
            let actions = FrozenActions::freeze(&config(q), kind).unwrap();
            for p in [
                Policy::copy(),
                Policy::invert(),
                Policy::negative(),
                Policy::positive(),
                Policy::calibration_copy_live_invert(),
                Policy::new(8321).unwrap(),
            ] {
                let e = evaluate_fixed(&actions, &p).unwrap();
                assert_eq!(e.payoff_numerator, -e.utility_numerator);
                assert_eq!(e.regret_numerator, e.optimal_numerator - e.payoff_numerator);
                assert_eq!(
                    e.intervention_mass,
                    e.correct_intervention_mass + e.false_intervention_mass
                );
                assert_eq!(
                    e.correct_intervention_mass + e.missed_beneficial_intervention_mass,
                    e.denominator / 2
                );
                assert_eq!(
                    e.histories.iter().map(|h| h.total_mass).sum::<u64>(),
                    e.denominator
                );
                assert_eq!(
                    e.histories.iter().map(|h| h.regret_numerator).sum::<i64>(),
                    e.regret_numerator
                );
                assert_eq!(e.error_mass, 0);
                assert_eq!(e.reference_regret_numerator, 0);
                let public = strategic_reporting::evaluate(
                    &strategic_reporting::enumerate(&config(q)).unwrap(),
                    &p,
                    &strategic_reporting::FrozenListener {
                        algorithm: strategic_reporting::Listener::Passive,
                        assumed_copy_prior: config(q).fixed_copy_prior,
                    },
                )
                .unwrap();
                assert_eq!(
                    (
                        e.calibration_truth_agreement_mass,
                        e.live_truth_agreement_mass,
                        e.calibration_signal_opposition_mass,
                        e.live_signal_opposition_mass
                    ),
                    (
                        public.calibration_truth_agreement_mass,
                        public.live_truth_agreement_mass,
                        public.calibration_signal_opposition_mass,
                        public.live_signal_opposition_mass
                    )
                );
                for h in &e.histories {
                    if h.total_mass == 0 {
                        assert_eq!(
                            (
                                h.reference_posterior.clone(),
                                h.reference_action,
                                h.listener_action,
                                h.regret_numerator
                            ),
                            (None, None, None, 0)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn zero_weight_policies_do_not_change_mixture_or_invent_actions() {
    let actions = FrozenActions::freeze(&config(4), ControllerKind::FixedOnly).unwrap();
    let mix = Catalog::new(vec![
        WeightedPolicy {
            policy: Policy::copy(),
            weight: 1,
        },
        WeightedPolicy {
            policy: Policy::negative(),
            weight: 0,
        },
    ])
    .unwrap();
    assert_eq!(
        evaluate_mixture(&actions, &mix).unwrap(),
        evaluate_fixed(&actions, &Policy::copy()).unwrap()
    );
    assert!(evaluate_fixed(&actions, &Policy { bits: 1 << 18 }).is_err());
}

#[test]
fn renamed_ids_and_policy_aliases_preserve_evaluation() {
    let c = config(4);
    let a = FrozenActions::freeze(&c, ControllerKind::StrategyUniform).unwrap();
    let baseline = evaluate_fixed(&a, &Policy::negative()).unwrap();
    let mut renamed = c.clone();
    renamed.strategic = 71;
    renamed.fixed = 72;
    renamed.decider = 73;
    for permission in &mut renamed.permissions {
        permission.agent = match permission.agent {
            0 => 71,
            1 => 72,
            2 => 73,
            _ => panic!("unexpected standard Agent ID"),
        };
    }
    let b = FrozenActions::freeze(&renamed, ControllerKind::StrategyUniform).unwrap();
    let e = evaluate_fixed(&b, &Policy::negative()).unwrap();
    assert_eq!(baseline.payoff_numerator, e.payoff_numerator);
    assert_eq!(baseline.regret_numerator, e.regret_numerator);
    let alias = Policy::new(0b1111 << 6).unwrap();
    assert_eq!(evaluate_fixed(&a, &alias).unwrap(), baseline);
}

#[test]
fn strategy_self_mixture_matches_earlier_public_evaluation_as_provenance_regression() {
    for q in [4, 3] {
        for (kind, catalog) in [
            (ControllerKind::StrategyUniform, Catalog::uniform()),
            (
                ControllerKind::StrategyOptimizationInformed,
                Catalog::optimization_informed(),
            ),
        ] {
            let c = config(q);
            let actions = FrozenActions::freeze(&c, kind).unwrap();
            let e = evaluate_mixture(&actions, &catalog).unwrap();
            let model = strategy_inference::Model::new(&c, &catalog).unwrap();
            let old = strategy_inference::evaluate_mixture(
                &c,
                &catalog,
                &strategy_inference::EvaluatedListener::Strategy(model),
            )
            .unwrap();
            let payoff = old.payoff.unwrap();
            assert_eq!(
                i128::from(e.payoff_numerator) * i128::from(payoff.denominator),
                i128::from(payoff.numerator) * i128::from(e.denominator)
            );
            let regret = old.decision_regret.unwrap();
            assert_eq!(
                e.regret_numerator as u128 * regret.denominator as u128,
                regret.numerator as u128 * e.denominator as u128
            );
        }
    }
}

#[test]
fn production_evaluation_uses_public_mass_interfaces_without_private_world_access() {
    let source = include_str!("../evaluation.rs");
    for forbidden in [
        ".worlds",
        "struct World",
        "StrategicObservation",
        "fn enumerate(",
    ] {
        assert!(!source.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn partial_probability_endpoints_keep_zero_mass_absent_and_ties_abstain() {
    for q in [0, 1, 2] {
        let c = Config::standard(
            Probability {
                numerator: q,
                denominator: 2,
            },
            Probability {
                numerator: 1,
                denominator: 1,
            },
            UtilityTable::opposed(),
        );
        let actions = FrozenActions::freeze(&c, ControllerKind::Passive).unwrap();
        let e = evaluate_fixed(&actions, &Policy::negative()).unwrap();
        assert_eq!(e.payoff_numerator, 0);
        assert_eq!(
            e.histories.iter().map(|h| h.total_mass).sum::<u64>(),
            e.denominator
        );
        for h in &e.histories {
            if h.total_mass == 0 {
                assert_eq!(h.reference_action, None);
                assert_eq!(h.listener_action, None);
            } else if q == 1 {
                let posterior = h.reference_posterior.as_ref().unwrap();
                assert_eq!(posterior.numerator * 2, posterior.denominator);
                assert_eq!(h.reference_action, Some(DecisionAction::Abstain));
            }
        }
    }
}

#[test]
fn maximum_catalog_mass_and_unequal_denominator_comparisons_remain_exact() {
    let c = Config::standard(
        Probability {
            numerator: 7,
            denominator: 16,
        },
        Probability {
            numerator: 11,
            denominator: 16,
        },
        UtilityTable::opposed(),
    );
    let actions = FrozenActions::freeze(&c, ControllerKind::Passive).unwrap();
    let catalog = Catalog::new(
        canonical_policies()
            .into_iter()
            .take(8)
            .map(|policy| WeightedPolicy { policy, weight: 32 })
            .collect(),
    )
    .unwrap();
    let e = evaluate_mixture(&actions, &catalog).unwrap();
    assert_eq!(e.denominator, 1 << 30);
    assert_eq!(e.payoff_numerator, 0);
    assert_eq!(
        e.histories.iter().map(|h| h.total_mass).sum::<u64>(),
        1 << 30
    );
    let left = Score {
        rules: config(4),
        denominator: 50,
        payoff_numerator: 9,
        utility_numerator: -9,
    };
    let right = Score {
        rules: config(4),
        denominator: 250,
        payoff_numerator: 21,
        utility_numerator: -21,
    };
    let r = guarantee_shortfall(&left, &right).unwrap();
    assert_eq!(r.numerator * 125, 12 * r.denominator);
    let r = nominal_difference(&right, &left).unwrap();
    assert_eq!(r.numerator * 125, -12 * r.denominator as i64);
}

#[test]
fn named_policy_phase_agreement_and_opposition_are_complete() {
    for q in [4, 3] {
        let actions = FrozenActions::freeze(&config(q), ControllerKind::FixedOnly).unwrap();
        for (p, cq, tq, cop, top) in [
            (Policy::copy(), q, q, 0, 0),
            (Policy::invert(), 5 - q, 5 - q, 1, 1),
            (Policy::calibration_copy_live_invert(), q, 5 - q, 0, 1),
        ] {
            let e = evaluate_fixed(&actions, &p).unwrap();
            assert_eq!(
                e.calibration_truth_agreement_mass * 5,
                u64::from(cq) * e.denominator
            );
            assert_eq!(
                e.live_truth_agreement_mass * 5,
                u64::from(tq) * e.denominator
            );
            assert_eq!(e.calibration_signal_opposition_mass, cop * e.denominator);
            assert_eq!(e.live_signal_opposition_mass, top * e.denominator);
            let calibration_agreement: u64 = e
                .histories
                .iter()
                .filter(|h| h.observation.calibration_reports[0] == h.observation.calibration_truth)
                .map(|h| h.total_mass)
                .sum();
            let calibration_opposition: u64 = e
                .histories
                .iter()
                .filter(|h| h.observation.calibration_reports[0] != h.observation.calibration_truth)
                .map(|h| h.total_mass)
                .sum();
            let live_agreement: u64 = e
                .histories
                .iter()
                .map(|h| {
                    if h.observation.live_reports[0] {
                        h.true_mass
                    } else {
                        h.total_mass - h.true_mass
                    }
                })
                .sum();
            let live_opposition: u64 = e
                .histories
                .iter()
                .map(|h| {
                    if h.observation.live_reports[0] {
                        h.total_mass - h.true_mass
                    } else {
                        h.true_mass
                    }
                })
                .sum();
            assert_eq!(calibration_agreement, e.calibration_truth_agreement_mass);
            assert_eq!(live_agreement, e.live_truth_agreement_mass);
            assert_eq!(
                calibration_agreement + calibration_opposition,
                e.denominator
            );
            assert_eq!(live_agreement + live_opposition, e.denominator);
        }
    }
}
