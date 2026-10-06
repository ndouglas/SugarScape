use super::super::{canonical_bits, CalibrationView, Catalog, Error, Model, WeightedPolicy};
use crate::deduction::strategic_reporting::{Config, Policy, Probability, UtilityTable};
fn config(numerator: u16, denominator: u16) -> Config {
    Config::standard(
        Probability {
            numerator,
            denominator,
        },
        Probability {
            numerator: 3,
            denominator: 4,
        },
        UtilityTable::opposed(),
    )
}
#[test]
fn calibration_preserves_copy_vs_live_invert_prior_odds() {
    for q in [4, 3] {
        let rules = config(q, 5);
        for (catalog, odds) in [
            (Catalog::uniform(), 1),
            (Catalog::optimization_informed(), 16),
        ] {
            let model = Model::new(&rules, &catalog).unwrap();
            for i in 0..8 {
                let belief = model
                    .calibration(&CalibrationView {
                        rules: rules.clone(),
                        calibration_truth: i & 4 != 0,
                        calibration_reports: [i & 2 != 0, i & 1 != 0],
                    })
                    .unwrap();
                let copy = belief
                    .policies
                    .iter()
                    .find(|p| p.canonical_bits == canonical_bits(&Policy::copy()).unwrap())
                    .unwrap();
                let invert = belief
                    .policies
                    .iter()
                    .find(|p| {
                        p.canonical_bits
                            == canonical_bits(&Policy::calibration_copy_live_invert()).unwrap()
                    })
                    .unwrap();
                assert!(copy.probability.numerator > 0);
                assert_eq!(
                    u128::from(invert.probability.numerator)
                        * u128::from(copy.probability.denominator),
                    u128::from(copy.probability.numerator)
                        * u128::from(invert.probability.denominator)
                        * odds
                );
            }
        }
    }
}
#[test]
fn singleton_copy_rejects_impossible_calibration() {
    let config = Config::standard(
        Probability {
            numerator: 1,
            denominator: 1,
        },
        Probability {
            numerator: 1,
            denominator: 1,
        },
        UtilityTable::opposed(),
    );
    let catalog = Catalog::new(vec![WeightedPolicy {
        policy: Policy::copy(),
        weight: 1,
    }])
    .unwrap();
    let model = Model::new(&config, &catalog).unwrap();
    let view = CalibrationView {
        rules: config,
        calibration_truth: true,
        calibration_reports: [false, true],
    };
    assert!(matches!(model.calibration(&view), Err(Error::ZeroEvidence)));
}

use super::super::{
    CalibrationBelief, InferenceDecision, LivePrediction, PolicyProbability, Ratio, SignedRatio,
};
use crate::deduction::strategic_reporting::{
    enumerate, histories, DecisionAction, DecisionObservation,
};
use serde::Deserialize;
use serde_json::{json, Value};

fn observation(rules: &Config, index: usize) -> DecisionObservation {
    DecisionObservation {
        rules: rules.clone(),
        calibration_truth: index & 16 != 0,
        calibration_reports: [index & 8 != 0, index & 4 != 0],
        live_reports: [index & 2 != 0, index & 1 != 0],
    }
}
fn calibration(rules: &Config, index: usize) -> CalibrationView {
    CalibrationView {
        rules: rules.clone(),
        calibration_truth: index & 4 != 0,
        calibration_reports: [index & 2 != 0, index & 1 != 0],
    }
}
fn equal_ratio(actual: &Ratio, expected: [u64; 2]) {
    assert!(actual.denominator > 0);
    assert_eq!(
        u128::from(actual.numerator) * u128::from(expected[1]),
        u128::from(expected[0]) * u128::from(actual.denominator)
    );
}
#[derive(Deserialize)]
struct Reference {
    cases: Vec<ReferenceCase>,
}
// Compact fixture rows follow lexicographic [C, own-cal, fixed-cal, own-live, fixed-live].
type ExpectedPolicies = Vec<[u64; 2]>;
type ExpectedHistory = (Option<[u64; 2]>, Option<String>, Option<ExpectedPolicies>);
type ExpectedCalibration = (Option<ExpectedPolicies>, Option<Vec<[[u64; 2]; 2]>>);
#[derive(Deserialize)]
struct ReferenceCase {
    id: String,
    accuracy: [u16; 2],
    fixed_copy_prior: [u16; 2],
    entries: Vec<(u32, u8)>,
    rows: Vec<ExpectedHistory>,
    calibrations: Vec<ExpectedCalibration>,
}
fn reference() -> Reference {
    serde_json::from_str(include_str!("inference-reference.json")).unwrap()
}
fn reference_model(case: &ReferenceCase) -> (Config, Catalog, Model) {
    let rules = Config::standard(
        Probability {
            numerator: case.accuracy[0],
            denominator: case.accuracy[1],
        },
        Probability {
            numerator: case.fixed_copy_prior[0],
            denominator: case.fixed_copy_prior[1],
        },
        UtilityTable::opposed(),
    );
    let catalog = Catalog::new(
        case.entries
            .iter()
            .map(|&(bits, weight)| WeightedPolicy {
                policy: Policy::new(bits).unwrap(),
                weight,
            })
            .collect(),
    )
    .unwrap();
    let model = Model::new(&rules, &catalog).unwrap();
    (rules, catalog, model)
}
#[test]
fn all_reference_histories_and_calibrations_match_independent_full_joint_oracle() {
    for case in reference().cases {
        let (rules, catalog, model) = reference_model(&case);
        assert_eq!(case.rows.len(), 32, "{}", case.id);
        assert_eq!(case.calibrations.len(), 8, "{}", case.id);
        for (i, (posterior, action, policies)) in case.rows.iter().enumerate() {
            let result = model.decide(&observation(&rules, i));
            if let Some(posterior) = posterior {
                let decision = result.unwrap_or_else(|e| panic!("{} row {i}: {e}", case.id));
                equal_ratio(&decision.posterior_true, *posterior);
                assert_eq!(
                    decision.action,
                    if action.as_deref() == Some("intervene") {
                        DecisionAction::Intervene
                    } else {
                        DecisionAction::Abstain
                    },
                    "{} row {i}",
                    case.id
                );
                for ((actual, expected), entry) in decision
                    .policies
                    .iter()
                    .zip(policies.as_ref().unwrap())
                    .zip(catalog.entries())
                {
                    assert_eq!(actual.canonical_bits, entry.policy.bits);
                    equal_ratio(&actual.probability, *expected);
                }
                assert_eq!(decision.policies.len(), catalog.entries().len());
            } else {
                assert!(
                    matches!(result, Err(Error::ZeroEvidence)),
                    "{} row {i}",
                    case.id
                );
            }
        }
        for (i, (policies, live)) in case.calibrations.iter().enumerate() {
            let result = model.calibration(&calibration(&rules, i));
            if let Some(policies) = policies {
                let belief = result.unwrap_or_else(|e| panic!("{} calibration {i}: {e}", case.id));
                assert_eq!(belief.policies.len(), catalog.entries().len());
                assert_eq!(belief.live.len(), 4);
                for ((actual, expected), entry) in
                    belief.policies.iter().zip(policies).zip(catalog.entries())
                {
                    assert_eq!(actual.canonical_bits, entry.policy.bits);
                    equal_ratio(&actual.probability, *expected);
                }
                for (j, (actual, expected)) in
                    belief.live.iter().zip(live.as_ref().unwrap()).enumerate()
                {
                    assert_eq!(actual.live_reports, [j & 2 != 0, j & 1 != 0]);
                    equal_ratio(&actual.probability, expected[0]);
                    equal_ratio(&actual.truth_and_reports_probability, expected[1]);
                }
            } else {
                assert!(
                    matches!(result, Err(Error::ZeroEvidence)),
                    "{} calibration {i}",
                    case.id
                );
            }
        }
    }
}
#[test]
fn half_accuracy_abstains_on_every_supported_history_including_both_calibration_truths() {
    for rho in [0, 1] {
        let rules = Config::standard(
            Probability {
                numerator: 1,
                denominator: 2,
            },
            Probability {
                numerator: rho,
                denominator: 1,
            },
            UtilityTable::opposed(),
        );
        for catalog in [Catalog::uniform(), Catalog::optimization_informed()] {
            let model = Model::new(&rules, &catalog).unwrap();
            for i in 0..32 {
                let decision = model.decide(&observation(&rules, i)).unwrap();
                equal_ratio(&decision.posterior_true, [1, 2]);
                assert_eq!(decision.action, DecisionAction::Abstain);
            }
        }
    }
}
#[test]
fn singleton_inference_equals_actual_policy_reference_for_all_supported_histories() {
    for q in [(0, 1), (1, 2), (1, 1), (3, 5), (4, 5)] {
        for rho in [(0, 1), (3, 4), (1, 1)] {
            let rules = Config::standard(
                Probability {
                    numerator: q.0,
                    denominator: q.1,
                },
                Probability {
                    numerator: rho.0,
                    denominator: rho.1,
                },
                UtilityTable::opposed(),
            );
            let distribution = enumerate(&rules).unwrap();
            for policy in [
                Policy::copy(),
                Policy::invert(),
                Policy::positive(),
                Policy::negative(),
                Policy::calibration_copy_live_invert(),
                Policy::new(98342).unwrap(),
            ] {
                let catalog = Catalog::new(vec![WeightedPolicy {
                    policy: policy.clone(),
                    weight: 32,
                }])
                .unwrap();
                let model = Model::new(&rules, &catalog).unwrap();
                for row in histories(&distribution, &policy).unwrap() {
                    let decision = model.decide(&row.observation);
                    if row.total_mass == 0 {
                        assert!(matches!(decision, Err(Error::ZeroEvidence)));
                    } else {
                        let decision = decision.unwrap();
                        equal_ratio(&decision.posterior_true, [row.true_mass, row.total_mass]);
                        equal_ratio(&decision.policies[0].probability, [1, 1]);
                        assert_eq!(
                            decision.action,
                            if row.true_mass > row.total_mass - row.true_mass {
                                DecisionAction::Intervene
                            } else {
                                DecisionAction::Abstain
                            }
                        );
                    }
                }
            }
        }
    }
}

// A separate rational calculation for sequential Bayes; no Model caches are accessed.
#[derive(Clone, Copy)]
struct Fraction {
    n: i128,
    d: i128,
}
impl Fraction {
    fn new(n: i128, d: i128) -> Self {
        assert!(d > 0);
        let mut a = n.abs();
        let mut b = d;
        while b != 0 {
            let rem = a % b;
            a = b;
            b = rem;
        }
        Self { n: n / a, d: d / a }
    }
    fn add(self, other: Self) -> Self {
        Self::new(self.n * other.d + other.n * self.d, self.d * other.d)
    }
    fn mul(self, other: Self) -> Self {
        Self::new(self.n * other.n, self.d * other.d)
    }
    fn div(self, other: Self) -> Self {
        Self::new(self.n * other.d, self.d * other.n)
    }
}
#[test]
fn independent_sequential_conditioning_and_predictive_conditioning_equal_direct() {
    for q in [3, 4] {
        let rules = config(q, 5);
        let d = enumerate(&rules).unwrap();
        for catalog in [Catalog::uniform(), Catalog::optimization_informed()] {
            let model = Model::new(&rules, &catalog).unwrap();
            let policy_rows: Vec<_> = catalog
                .entries()
                .iter()
                .map(|entry| histories(&d, &entry.policy).unwrap())
                .collect();
            for i in 0..32 {
                let belief = model.calibration(&calibration(&rules, i >> 2)).unwrap();
                let mut total = Fraction::new(0, 1);
                let mut true_total = total;
                let mut sequential_policies = Vec::new();
                for (rows, probability) in policy_rows.iter().zip(&belief.policies) {
                    let cal_mass: u64 = rows[i & !3..(i & !3) + 4]
                        .iter()
                        .map(|row| row.total_mass)
                        .sum();
                    let p = Fraction::new(
                        i128::from(probability.probability.numerator),
                        i128::from(probability.probability.denominator),
                    );
                    let joint = if cal_mass == 0 {
                        Fraction::new(0, 1)
                    } else {
                        p.mul(Fraction::new(
                            i128::from(rows[i].total_mass),
                            i128::from(cal_mass),
                        ))
                    };
                    sequential_policies.push(joint);
                    total = total.add(joint);
                    if cal_mass > 0 {
                        true_total = true_total.add(p.mul(Fraction::new(
                            i128::from(rows[i].true_mass),
                            i128::from(cal_mass),
                        )));
                    }
                }
                let direct = model.decide(&observation(&rules, i)).unwrap();
                let sequential = true_total.div(total);
                equal_ratio(
                    &direct.posterior_true,
                    [sequential.n as u64, sequential.d as u64],
                );
                for (actual, joint) in direct.policies.iter().zip(sequential_policies) {
                    let posterior = joint.div(total);
                    equal_ratio(
                        &actual.probability,
                        [posterior.n as u64, posterior.d as u64],
                    );
                }
                let prediction = &belief.live[i & 3];
                let predictive = Fraction::new(
                    i128::from(prediction.truth_and_reports_probability.numerator),
                    i128::from(prediction.truth_and_reports_probability.denominator),
                )
                .div(Fraction::new(
                    i128::from(prediction.probability.numerator),
                    i128::from(prediction.probability.denominator),
                ));
                equal_ratio(
                    &direct.posterior_true,
                    [predictive.n as u64, predictive.d as u64],
                );
                equal_ratio(&prediction.probability, [total.n as u64, total.d as u64]);
            }
        }
    }
}
#[test]
fn calibration_is_not_counted_twice() {
    let rules = config(4, 5);
    let model = Model::new(&rules, &Catalog::uniform()).unwrap();
    let posterior = model
        .decide(&observation(&rules, 0))
        .unwrap()
        .posterior_true;
    equal_ratio(&posterior, [8, 31]);
    assert_ne!(
        u128::from(posterior.numerator) * 6719,
        1672 * u128::from(posterior.denominator)
    );
}
#[test]
fn self_mixture_decisions_have_zero_regret() {
    for q in [3, 4] {
        let rules = config(q, 5);
        let d = enumerate(&rules).unwrap();
        for catalog in [Catalog::uniform(), Catalog::optimization_informed()] {
            let model = Model::new(&rules, &catalog).unwrap();
            let mut totals = [0u64; 32];
            let mut truths = totals;
            for entry in catalog.entries() {
                for (i, row) in histories(&d, &entry.policy).unwrap().iter().enumerate() {
                    totals[i] += row.total_mass * u64::from(entry.weight);
                    truths[i] += row.true_mass * u64::from(entry.weight);
                }
            }
            let mut regret = 0i128;
            for i in 0..32 {
                let action = model.decide(&observation(&rules, i)).unwrap().action;
                let benefit = 2 * i128::from(truths[i]) - i128::from(totals[i]);
                regret += benefit.max(0)
                    - if action == DecisionAction::Intervene {
                        benefit
                    } else {
                        0
                    };
            }
            assert_eq!(regret, 0);
        }
    }
}
#[test]
fn catalog_permutation_and_agent_renaming_preserve_inference() {
    let rules = config(4, 5);
    let catalog = Catalog::optimization_informed();
    let reordered = Catalog::new(catalog.entries().iter().rev().cloned().collect()).unwrap();
    let mut renamed = rules.clone();
    renamed.strategic = 13;
    renamed.fixed = 42;
    renamed.decider = 99;
    for permissions in &mut renamed.permissions {
        permissions.agent = match permissions.agent {
            0 => 13,
            1 => 42,
            2 => 99,
            _ => unreachable!(),
        };
    }
    let model = Model::new(&rules, &catalog).unwrap();
    let permutation = Model::new(&rules, &reordered).unwrap();
    let renamed_model = Model::new(&renamed, &catalog).unwrap();
    for i in 0..32 {
        assert_eq!(
            model.decide(&observation(&rules, i)).unwrap(),
            permutation.decide(&observation(&rules, i)).unwrap()
        );
        assert_eq!(
            model.decide(&observation(&rules, i)).unwrap(),
            renamed_model.decide(&observation(&renamed, i)).unwrap()
        );
    }
    for i in 0..8 {
        assert_eq!(
            model.calibration(&calibration(&rules, i)).unwrap(),
            renamed_model
                .calibration(&calibration(&renamed, i))
                .unwrap()
        );
    }
}
#[test]
fn invalid_or_mismatched_rules_fail_without_changing_model() {
    let rules = config(4, 5);
    let catalog = Catalog::uniform();
    let model = Model::new(&rules, &catalog).unwrap();
    let before = model.calibration(&calibration(&rules, 0)).unwrap();
    for failure in 0..3 {
        let mut invalid = rules.clone();
        match failure {
            0 => invalid.accuracy.denominator = 0,
            1 => invalid.version = 0,
            _ => invalid.strategic_utility.intervene_true = 2,
        };
        assert!(Model::new(&invalid, &catalog).is_err());
        let mut view = calibration(&rules, 0);
        view.rules = invalid.clone();
        assert!(model.calibration(&view).is_err());
        let mut view = observation(&rules, 0);
        view.rules = invalid;
        assert!(model.decide(&view).is_err());
    }
    for change in 0..3 {
        let mut different = rules.clone();
        match change {
            0 => different.accuracy.numerator = 3,
            1 => different.fixed_copy_prior.numerator = 2,
            _ => different.strategic_utility = UtilityTable::aligned(),
        };
        let mut view = calibration(&rules, 0);
        view.rules = different.clone();
        assert!(matches!(
            model.calibration(&view),
            Err(Error::InvalidObservation(_))
        ));
        let mut view = observation(&rules, 0);
        view.rules = different;
        assert!(matches!(
            model.decide(&view),
            Err(Error::InvalidObservation(_))
        ));
    }
    assert_eq!(model.calibration(&calibration(&rules, 0)).unwrap(), before);
    assert_eq!(model.config(), &rules);
    assert_eq!(model.catalog(), &catalog);
}
#[test]
fn maximum_valid_configuration_and_catalog_fit_mass_bound() {
    let rules = Config::standard(
        Probability {
            numerator: 7,
            denominator: 16,
        },
        Probability {
            numerator: 9,
            denominator: 16,
        },
        UtilityTable::opposed(),
    );
    let catalog = Catalog::new(
        (0..8)
            .map(|bits| WeightedPolicy {
                policy: Policy::new(bits << 2).unwrap(),
                weight: 32,
            })
            .collect(),
    )
    .unwrap();
    let model = Model::new(&rules, &catalog).unwrap();
    assert_eq!(model.denominator(), 1_073_741_824);
    for i in 0..32 {
        match model.decide(&observation(&rules, i)) {
            Ok(decision) => {
                assert!(decision.posterior_true.numerator <= decision.posterior_true.denominator);
                assert!(decision.posterior_true.denominator <= model.denominator());
            }
            Err(Error::ZeroEvidence) => {}
            other => panic!("unexpected {other:?}"),
        }
    }
}
#[test]
fn public_calibration_view_rejects_private_or_live_fields_and_malformed_rules() {
    let base = serde_json::to_value(calibration(&config(4, 5), 0)).unwrap();
    assert_eq!(base.as_object().unwrap().len(), 3);
    for field in [
        "live_reports",
        "live_truth",
        "actual_policy",
        "calibration_signal",
        "seed",
    ] {
        let mut wire = base.clone();
        wire[field] = json!(false);
        assert!(serde_json::from_value::<CalibrationView>(wire).is_err());
    }
    for wire in [
        json!({"calibration_truth":false,"calibration_reports":[false,true]}),
        json!({"rules":base["rules"],"calibration_truth":false,"calibration_reports":[false]}),
        json!({"rules":base["rules"],"calibration_truth":null,"calibration_reports":[false,true]}),
    ] {
        assert!(serde_json::from_value::<CalibrationView>(wire).is_err());
    }
    let mut unknown = base.clone();
    unknown["rules"]["extra"] = json!(true);
    assert!(serde_json::from_value::<CalibrationView>(unknown).is_err());
    let mut invalid = base;
    invalid["rules"]["accuracy"]["denominator"] = json!(0);
    let view = serde_json::from_value::<CalibrationView>(invalid).unwrap();
    let model = Model::new(&config(4, 5), &Catalog::uniform()).unwrap();
    assert!(model.calibration(&view).is_err());
}
#[test]
fn ratio_wire_preserves_unreduced_pairs_and_rejects_invalid_values() {
    let ratio: Ratio = serde_json::from_value(json!({"numerator":2,"denominator":4})).unwrap();
    assert_eq!(
        serde_json::to_value(ratio).unwrap(),
        json!({"numerator":2,"denominator":4})
    );
    let signed: SignedRatio =
        serde_json::from_value(json!({"numerator":-2,"denominator":4})).unwrap();
    assert_eq!(
        serde_json::to_value(signed).unwrap(),
        json!({"numerator":-2,"denominator":4})
    );
    for wire in [
        json!({"numerator":0,"denominator":0}),
        json!({"numerator":3,"denominator":2}),
        json!({"numerator":1,"denominator":2,"extra":0}),
    ] {
        assert!(serde_json::from_value::<Ratio>(wire).is_err());
    }
    for wire in [
        json!({"numerator":0,"denominator":0}),
        json!({"numerator":-3,"denominator":2}),
        json!({"numerator":1,"denominator":2,"extra":0}),
    ] {
        assert!(serde_json::from_value::<SignedRatio>(wire).is_err());
    }
    assert!(serde_json::from_value::<SignedRatio>(
        json!({"numerator":i64::MIN,"denominator":i64::MIN.unsigned_abs()})
    )
    .is_ok());
}
#[test]
fn public_belief_wire_rejects_unknown_fields_at_every_nesting_level() {
    let ratio = Ratio {
        numerator: 0,
        denominator: 2,
    };
    let policy = PolicyProbability {
        canonical_bits: 0,
        probability: ratio.clone(),
    };
    let live = LivePrediction {
        live_reports: [false, false],
        probability: ratio.clone(),
        truth_and_reports_probability: ratio.clone(),
    };
    let belief = CalibrationBelief {
        policies: vec![policy.clone()],
        live: vec![live],
    };
    let base = serde_json::to_value(belief).unwrap();
    for path in [
        vec![],
        vec!["policies", "0"],
        vec!["policies", "0", "probability"],
        vec!["live", "0"],
        vec!["live", "0", "probability"],
        vec!["live", "0", "truth_and_reports_probability"],
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
        target["extra"] = Value::Bool(true);
        assert!(serde_json::from_value::<CalibrationBelief>(wire).is_err());
    }
    let decision = InferenceDecision {
        policies: vec![policy],
        posterior_true: ratio,
        action: DecisionAction::Abstain,
    };
    let mut wire = serde_json::to_value(decision).unwrap();
    wire["extra"] = json!(true);
    assert!(serde_json::from_value::<InferenceDecision>(wire).is_err());
}
