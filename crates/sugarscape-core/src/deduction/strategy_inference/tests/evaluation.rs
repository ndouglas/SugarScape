use super::super::{
    evaluate_fixed, evaluate_mixture, Catalog, Error, EvaluatedListener, Evaluation, Model, Ratio,
    SignedRatio, WeightedPolicy,
};
use crate::deduction::strategic_reporting::{
    Config, DecisionAction, FrozenListener, Listener, Policy, Probability, UtilityTable,
};

fn config(q: [u16; 2], rho: [u16; 2]) -> Config {
    Config::standard(
        Probability {
            numerator: q[0],
            denominator: q[1],
        },
        Probability {
            numerator: rho[0],
            denominator: rho[1],
        },
        UtilityTable::opposed(),
    )
}
fn singleton(policy: Policy) -> Catalog {
    Catalog::new(vec![WeightedPolicy { policy, weight: 1 }]).unwrap()
}
fn strategy(rules: &Config, catalog: &Catalog) -> EvaluatedListener {
    EvaluatedListener::Strategy(Model::new(rules, catalog).unwrap())
}
#[test]
fn invert_against_perfect_copy_is_all_unsupported_with_no_unconditional_metrics() {
    let rules = config([1, 1], [1, 1]);
    let listener = strategy(&rules, &singleton(Policy::copy()));
    let evaluation = evaluate_fixed(&rules, &Policy::invert(), &listener).unwrap();
    assert_eq!(evaluation.unsupported_mass, evaluation.denominator);
    assert_eq!(evaluation.supported_mass, 0);
    assert_eq!(evaluation.payoff, None);
    assert_eq!(evaluation.decision_regret, None);
    assert_eq!(evaluation.reporter_utility, None);
    assert_eq!(evaluation.maximum_belief_error, None);
    assert_eq!(evaluation.histories.len(), 32);
    for row in &evaluation.histories {
        assert_eq!(row.unsupported, row.actual_mass > 0);
        assert_eq!(row.action, None);
        assert_eq!(row.listener_posterior, None);
    }
}
#[test]
fn passive_has_zero_payoff_without_listener_probabilities() {
    let rules = config([4, 5], [3, 4]);
    let listener = EvaluatedListener::Legacy(FrozenListener {
        algorithm: Listener::Passive,
        assumed_copy_prior: Probability {
            numerator: 3,
            denominator: 4,
        },
    });
    let evaluation = evaluate_fixed(&rules, &Policy::copy(), &listener).unwrap();
    assert_eq!(evaluation.payoff.unwrap().numerator, 0);
    assert_eq!(evaluation.unsupported_mass, 0);
    assert_eq!(evaluation.maximum_belief_error, None);
    for row in evaluation.histories {
        assert_eq!(row.listener_posterior, None);
        assert_eq!(row.belief_error, None);
        if row.actual_mass > 0 {
            assert_eq!(row.action, Some(DecisionAction::Abstain));
        }
    }
}

use crate::deduction::strategic_reporting::Genome;
use serde_json::{json, Value};
fn assert_fraction(numerator: i128, denominator: u64, expected: &Value) {
    assert!(denominator > 0);
    assert_eq!(
        numerator * i128::from(expected[1].as_u64().unwrap()),
        i128::from(expected[0].as_i64().unwrap()) * i128::from(denominator)
    );
}
fn optional_signed(actual: &Option<SignedRatio>, expected: &Value) {
    match actual {
        Some(ratio) => assert_fraction(i128::from(ratio.numerator), ratio.denominator, expected),
        None => assert!(expected.is_null()),
    }
}
fn optional_ratio(actual: &Option<Ratio>, expected: &Value) {
    match actual {
        Some(ratio) => assert_fraction(i128::from(ratio.numerator), ratio.denominator, expected),
        None => assert!(expected.is_null()),
    }
}
fn reference_listener(id: &str, rules: &Config) -> EvaluatedListener {
    match id {
        "strategy_uniform" => strategy(rules, &Catalog::uniform()),
        "strategy_optimization_informed" => strategy(rules, &Catalog::optimization_informed()),
        "singleton_copy" => strategy(rules, &singleton(Policy::copy())),
        id => EvaluatedListener::Legacy(FrozenListener {
            algorithm: match id {
                "bayesian" => Listener::Bayesian,
                "credulous" => Listener::Credulous,
                "skeptical" => Listener::Skeptical,
                "passive" => Listener::Passive,
                "evolved" => Listener::Evolved(Genome {
                    b: -3,
                    u: 0,
                    d: 2,
                    k: 0,
                }),
                _ => panic!("unknown reference listener {id}"),
            },
            assumed_copy_prior: rules.fixed_copy_prior.clone(),
        }),
    }
}
fn reference_evaluation(case: &Value) -> Evaluation {
    let pair = |key: &str| {
        [
            case[key][0].as_u64().unwrap() as u16,
            case[key][1].as_u64().unwrap() as u16,
        ]
    };
    let rules = config(pair("accuracy"), pair("fixed_copy_prior"));
    let listener = reference_listener(case["listener"].as_str().unwrap(), &rules);
    let entries: Vec<_> = case["actual"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| WeightedPolicy {
            policy: Policy::new(p[0].as_u64().unwrap() as u32).unwrap(),
            weight: p[1].as_u64().unwrap() as u8,
        })
        .collect();
    if case["fixed"].as_bool().unwrap() {
        evaluate_fixed(&rules, &entries[0].policy, &listener).unwrap()
    } else {
        evaluate_mixture(&rules, &Catalog::new(entries).unwrap(), &listener).unwrap()
    }
}
#[test]
fn all_fixed_mixture_support_and_endpoint_rows_match_independent_fraction_reference() {
    let fixture: Value = serde_json::from_str(include_str!("evaluation-reference.json")).unwrap();
    assert_eq!(fixture["cases"].as_array().unwrap().len(), 122);
    for (index, case) in fixture["cases"].as_array().unwrap().iter().enumerate() {
        let evaluation = reference_evaluation(case);
        let a = &case["aggregates"];
        assert_eq!(
            evaluation.supported_mass + evaluation.unsupported_mass,
            evaluation.denominator,
            "case {index}"
        );
        assert_fraction(
            i128::from(evaluation.supported_mass),
            evaluation.denominator,
            &a["supported_probability"],
        );
        assert_fraction(
            i128::from(evaluation.unsupported_mass),
            evaluation.denominator,
            &a["unsupported_probability"],
        );
        assert_fraction(
            i128::from(evaluation.supported_payoff_numerator),
            evaluation.denominator,
            &a["supported_payoff"],
        );
        assert_fraction(
            i128::from(evaluation.supported_regret_numerator),
            evaluation.denominator,
            &a["supported_regret"],
        );
        assert_fraction(
            i128::from(evaluation.supported_reporter_utility_numerator),
            evaluation.denominator,
            &a["supported_reporter_utility"],
        );
        optional_signed(&evaluation.payoff, &a["payoff"]);
        optional_ratio(&evaluation.decision_regret, &a["decision_regret"]);
        optional_signed(&evaluation.reporter_utility, &a["reporter_utility"]);
        let strategy = case["listener"].as_str().unwrap().starts_with("strategy_")
            || case["listener"] == "singleton_copy";
        if strategy {
            optional_ratio(&evaluation.maximum_belief_error, &a["maximum_belief_error"]);
        } else {
            assert_eq!(evaluation.maximum_belief_error, None);
        }
        assert_eq!(evaluation.histories.len(), 32);
        for (i, (actual, expected)) in evaluation
            .histories
            .iter()
            .zip(case["rows"].as_array().unwrap())
            .enumerate()
        {
            assert_eq!(actual.observation.calibration_truth, i & 16 != 0);
            assert_eq!(
                actual.observation.calibration_reports,
                [i & 8 != 0, i & 4 != 0]
            );
            assert_eq!(actual.observation.live_reports, [i & 2 != 0, i & 1 != 0]);
            assert_fraction(
                i128::from(actual.actual_mass),
                evaluation.denominator,
                &expected[0],
            );
            assert_fraction(
                i128::from(actual.actual_true_mass),
                evaluation.denominator,
                &expected[1],
            );
            optional_ratio(&actual.actual_posterior, &expected[2]);
            if actual.actual_mass == 0 {
                assert!(!actual.unsupported);
                assert_eq!(actual.action, None);
                assert_eq!(actual.listener_posterior, None);
                assert_eq!(actual.belief_error, None);
                continue;
            }
            assert_eq!(actual.unsupported, expected[6].as_bool().unwrap());
            assert_eq!(
                actual.action,
                match expected[5].as_str() {
                    Some("intervene") => Some(DecisionAction::Intervene),
                    Some("abstain") => Some(DecisionAction::Abstain),
                    None => None,
                    other => panic!("{other:?}"),
                }
            );
            if strategy {
                optional_ratio(&actual.listener_posterior, &expected[3]);
                optional_signed(&actual.belief_error, &expected[4]);
            } else {
                assert_eq!(actual.listener_posterior, None);
                assert_eq!(actual.belief_error, None);
            }
            if let Some(action) = actual.action {
                let gain = 2 * i128::from(actual.actual_true_mass) - i128::from(actual.actual_mass);
                let payoff = if action == DecisionAction::Intervene {
                    gain
                } else {
                    0
                };
                assert_fraction(payoff, evaluation.denominator, &expected[7]);
                assert_fraction(gain.max(0) - payoff, evaluation.denominator, &expected[8]);
            }
        }
    }
}

#[test]
fn self_mixtures_have_zero_regret_and_zero_belief_errors_for_each_prior_and_accuracy() {
    for q in [3, 4] {
        let rules = config([q, 5], [3, 4]);
        for catalog in [Catalog::uniform(), Catalog::optimization_informed()] {
            let evaluation =
                evaluate_mixture(&rules, &catalog, &strategy(&rules, &catalog)).unwrap();
            assert_eq!(evaluation.decision_regret.unwrap().numerator, 0);
            assert_eq!(evaluation.maximum_belief_error.unwrap().numerator, 0);
            for row in evaluation.histories {
                assert_eq!(row.belief_error.unwrap().numerator, 0);
            }
        }
    }
}
#[test]
fn fixed_policy_informed_reference_can_outperform_mixture_reference() {
    let rules = config([4, 5], [3, 4]);
    let catalog = Catalog::uniform();
    let listener = strategy(&rules, &catalog);
    let mixture = evaluate_mixture(&rules, &catalog, &listener).unwrap();
    let transfer = evaluate_fixed(&rules, &Policy::copy(), &listener).unwrap();
    let fixed_reference = evaluate_fixed(
        &rules,
        &Policy::copy(),
        &strategy(&rules, &singleton(Policy::copy())),
    )
    .unwrap();
    let fixed = fixed_reference.payoff.unwrap();
    let mix = mixture.payoff.unwrap();
    assert!(
        i128::from(fixed.numerator) * i128::from(mix.denominator)
            > i128::from(mix.numerator) * i128::from(fixed.denominator)
    );
    let actual = transfer.payoff.unwrap();
    let regret = transfer.decision_regret.unwrap();
    assert_eq!(
        i128::from(fixed.numerator) * i128::from(actual.denominator),
        (i128::from(actual.numerator) + i128::from(regret.numerator))
            * i128::from(fixed.denominator)
    );
    assert!(regret.numerator > 0);
}
#[test]
fn partial_support_preserves_original_denominator_and_reports_no_unconditional_scores() {
    let rules = config([1, 1], [1, 1]);
    let result = evaluate_fixed(
        &rules,
        &Policy::positive(),
        &strategy(&rules, &singleton(Policy::copy())),
    )
    .unwrap();
    assert_eq!(result.supported_mass * 4, result.denominator);
    assert_eq!(result.unsupported_mass * 4, result.denominator * 3);
    assert_eq!(
        result.supported_payoff_numerator * 4,
        result.denominator as i64
    );
    assert_eq!(
        result.supported_reporter_utility_numerator,
        -result.supported_payoff_numerator
    );
    assert_eq!(result.payoff, None);
    assert_eq!(result.decision_regret, None);
    assert_eq!(result.reporter_utility, None);
    assert_eq!(result.maximum_belief_error.unwrap().numerator, 0);
}
#[test]
fn zero_weight_actual_entries_do_not_change_scores_or_create_support() {
    let rules = config([1, 1], [1, 1]);
    let listener = strategy(&rules, &singleton(Policy::copy()));
    let actual = Catalog::new(vec![
        WeightedPolicy {
            policy: Policy::copy(),
            weight: 1,
        },
        WeightedPolicy {
            policy: Policy::invert(),
            weight: 0,
        },
    ])
    .unwrap();
    assert_eq!(
        evaluate_mixture(&rules, &actual, &listener).unwrap(),
        evaluate_fixed(&rules, &Policy::copy(), &listener).unwrap()
    );
}
#[test]
fn rare_history_sets_exact_unweighted_maximum_against_fraction_reference() {
    let fixture: Value = serde_json::from_str(include_str!("evaluation-reference.json")).unwrap();
    let case = &fixture["cases"][0]; // q=4/5, uniform listener against Copy.
    let result = reference_evaluation(case);
    let expected = &case["aggregates"]["maximum_belief_error"];
    optional_ratio(&result.maximum_belief_error, expected);
    let maximum_row = &result.histories[12];
    assert!(maximum_row.actual_mass * 100 < result.denominator);
    assert_fraction(
        i128::from(
            maximum_row
                .belief_error
                .as_ref()
                .unwrap()
                .numerator
                .unsigned_abs(),
        ),
        maximum_row.belief_error.as_ref().unwrap().denominator,
        expected,
    );
    // Independent Fraction fixture contains reduced signed errors and probabilities.
    // Check every exact error by a separate cross-product, then find the maximum.
    let mut largest = [0i128, 1i128];
    for row in case["rows"].as_array().unwrap() {
        if row[2].is_null() {
            continue;
        }
        let model = [
            i128::from(row[3][0].as_i64().unwrap()),
            i128::from(row[3][1].as_i64().unwrap()),
        ];
        let actual = [
            i128::from(row[2][0].as_i64().unwrap()),
            i128::from(row[2][1].as_i64().unwrap()),
        ];
        let n = (model[0] * actual[1] - actual[0] * model[1]).abs();
        let d = model[1] * actual[1];
        if n * largest[1] > largest[0] * d {
            largest = [n, d];
        }
    }
    assert_fraction(largest[0], largest[1] as u64, expected);
    let error = maximum_row.belief_error.as_ref().unwrap();
    let model = maximum_row.listener_posterior.as_ref().unwrap();
    let actual = maximum_row.actual_posterior.as_ref().unwrap();
    assert_eq!(error.denominator, model.denominator * actual.denominator);
    assert_eq!(
        i128::from(error.numerator),
        i128::from(model.numerator) * i128::from(actual.denominator)
            - i128::from(actual.numerator) * i128::from(model.denominator)
    );
}
#[test]
fn errors_propagate_and_leave_listener_model_unchanged() {
    let rules = config([4, 5], [3, 4]);
    let listener = strategy(&rules, &Catalog::uniform());
    let before = evaluate_fixed(&rules, &Policy::copy(), &listener).unwrap();
    let mut mismatched = rules.clone();
    mismatched.accuracy.numerator = 3;
    assert!(matches!(
        evaluate_fixed(&mismatched, &Policy::copy(), &listener),
        Err(Error::InvalidObservation(_))
    ));
    let mut invalid = rules.clone();
    invalid.strategic_utility.intervene_true = 2;
    assert!(matches!(
        evaluate_mixture(&invalid, &Catalog::uniform(), &listener),
        Err(Error::Existing(_))
    ));
    assert!(matches!(
        evaluate_fixed(&rules, &Policy { bits: 1 << 18 }, &listener),
        Err(Error::Existing(_))
    ));
    let invalid_legacy = EvaluatedListener::Legacy(FrozenListener {
        algorithm: Listener::Evolved(Genome {
            b: 17,
            u: 0,
            d: 0,
            k: 0,
        }),
        assumed_copy_prior: rules.fixed_copy_prior.clone(),
    });
    assert!(matches!(
        evaluate_fixed(&rules, &Policy::copy(), &invalid_legacy),
        Err(Error::Existing(
            crate::deduction::strategic_reporting::Error::LegacyListener(
                crate::deduction::testimony_game::Error::InvalidGenome
            )
        ))
    ));
    assert_eq!(
        evaluate_fixed(&rules, &Policy::copy(), &listener).unwrap(),
        before
    );
}
#[test]
fn wrapped_legacy_zero_evidence_is_unsupported_without_fabricated_probabilities() {
    let rules = config([1, 1], [1, 1]);
    let legacy = reference_listener("bayesian", &rules);
    let result = evaluate_fixed(&rules, &Policy::invert(), &legacy).unwrap();
    assert_eq!(result.unsupported_mass, result.denominator);
    assert_eq!(result.payoff, None);
    assert_eq!(result.maximum_belief_error, None);
}
#[test]
fn evaluation_preserves_catalog_permutations_and_role_renaming() {
    let rules = config([4, 5], [3, 4]);
    let catalog = Catalog::optimization_informed();
    let reordered = Catalog::new(catalog.entries().iter().rev().cloned().collect()).unwrap();
    let before = evaluate_mixture(&rules, &catalog, &strategy(&rules, &catalog)).unwrap();
    assert_eq!(
        evaluate_mixture(&rules, &reordered, &strategy(&rules, &reordered)).unwrap(),
        before
    );
    let mut renamed = rules.clone();
    renamed.strategic = 17;
    renamed.fixed = 29;
    renamed.decider = 103;
    for p in &mut renamed.permissions {
        p.agent = match p.agent {
            0 => 17,
            1 => 29,
            2 => 103,
            _ => unreachable!(),
        };
    }
    let mut after = evaluate_mixture(&renamed, &catalog, &strategy(&renamed, &catalog)).unwrap();
    for row in &mut after.histories {
        row.observation.rules = rules.clone();
    }
    assert_eq!(after, before);
}
#[test]
fn reporter_utility_uses_the_validated_table_for_both_truths_and_actions() {
    let mut rules = config([4, 5], [3, 4]);
    rules.strategic_utility = UtilityTable {
        intervene_true: 0,
        intervene_false: 1,
        abstain_true: -1,
        abstain_false: 1,
    };
    let listener = strategy(&rules, &Catalog::uniform());
    let result = evaluate_fixed(&rules, &Policy::copy(), &listener).unwrap();
    let mut expected = 0i64;
    for row in &result.histories {
        if let Some(action) = row.action {
            expected += (row.actual_mass - row.actual_true_mass) as i64;
            if action == DecisionAction::Abstain {
                expected -= row.actual_true_mass as i64;
            }
        }
    }
    assert_eq!(result.reporter_utility.unwrap().numerator, expected);
}
#[test]
fn maximum_valid_mass_inputs_fit_exact_error_and_comparison_bounds() {
    let rules = config([7, 16], [9, 16]);
    let catalog = Catalog::new(
        (0..8)
            .map(|bits| WeightedPolicy {
                policy: Policy::new(bits << 2).unwrap(),
                weight: 32,
            })
            .collect(),
    )
    .unwrap();
    let result = evaluate_mixture(
        &rules,
        &catalog,
        &strategy(&rules, &Catalog::optimization_informed()),
    )
    .unwrap();
    assert_eq!(result.denominator, 1 << 30);
    for row in result.histories {
        if let Some(error) = row.belief_error {
            assert!(error.denominator <= 1 << 60);
            assert!(error.numerator.unsigned_abs() <= error.denominator);
        }
    }
}
#[test]
fn evaluation_wire_rejects_unknown_fields_at_all_nesting_levels() {
    let rules = config([4, 5], [3, 4]);
    let result = evaluate_mixture(
        &rules,
        &Catalog::uniform(),
        &strategy(&rules, &Catalog::uniform()),
    )
    .unwrap();
    let base = serde_json::to_value(&result).unwrap();
    assert_eq!(
        serde_json::from_value::<Evaluation>(base.clone()).unwrap(),
        result
    );
    for path in [
        vec![],
        vec!["payoff"],
        vec!["decision_regret"],
        vec!["reporter_utility"],
        vec!["maximum_belief_error"],
        vec!["histories", "0"],
        vec!["histories", "0", "actual_posterior"],
        vec!["histories", "0", "listener_posterior"],
        vec!["histories", "0", "belief_error"],
        vec!["histories", "0", "observation"],
        vec!["histories", "0", "observation", "rules"],
    ] {
        let mut wire = base.clone();
        let mut target = &mut wire;
        for part in path {
            target = if part == "0" {
                &mut target[0]
            } else {
                &mut target[part]
            };
        }
        target["extra"] = json!(true);
        assert!(serde_json::from_value::<Evaluation>(wire).is_err());
    }
}
