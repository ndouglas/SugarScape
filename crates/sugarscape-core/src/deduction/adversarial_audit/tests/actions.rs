use super::super::*;
use crate::deduction::strategic_reporting::{Config, DecisionAction, Probability, UtilityTable};

fn config(q: u16, denominator: u16, rho: u16, rho_denominator: u16) -> Config {
    Config::standard(
        Probability {
            numerator: q,
            denominator,
        },
        Probability {
            numerator: rho,
            denominator: rho_denominator,
        },
        UtilityTable::opposed(),
    )
}

#[test]
fn changed_named_actions_are_not_a_valid_freeze() {
    let c = config(4, 5, 3, 4);
    let mut snapshot = FrozenActions::freeze(&c, ControllerKind::Passive)
        .unwrap()
        .snapshot();
    snapshot.rows[0].decision.action = DecisionAction::Intervene;
    assert!(FrozenActions::from_snapshot(&snapshot).is_err());
}

#[test]
fn fixed_view_rejects_private_fields() {
    let view = FixedView {
        rules: config(4, 5, 3, 4),
        calibration_truth: false,
        calibration_report: false,
        live_report: false,
    };
    let base = serde_json::to_value(view).unwrap();
    for field in [
        "live_truth",
        "signal",
        "calibration_signal",
        "actual_policy",
        "world",
        "seed",
        "archive",
        "evaluator_score",
    ] {
        let mut wire = base.clone();
        wire[field] = serde_json::json!(false);
        assert!(
            serde_json::from_value::<FixedView>(wire).is_err(),
            "accepted {field}"
        );
    }
}

fn controllers() -> [ControllerKind; 4] {
    [
        ControllerKind::StrategyUniform,
        ControllerKind::StrategyOptimizationInformed,
        ControllerKind::FixedOnly,
        ControllerKind::Passive,
    ]
}
fn fixed_view(rules: &Config, index: usize) -> FixedView {
    FixedView {
        rules: rules.clone(),
        calibration_truth: index & 4 != 0,
        calibration_report: index & 2 != 0,
        live_report: index & 1 != 0,
    }
}
fn fixed_index(observation: &crate::deduction::strategic_reporting::DecisionObservation) -> usize {
    (usize::from(observation.calibration_truth) << 2)
        | (usize::from(observation.calibration_reports[1]) << 1)
        | usize::from(observation.live_reports[1])
}

#[test]
fn every_named_freeze_has_exact_order_and_reconstructs() {
    for q in [3, 4] {
        let rules = config(q, 5, 3, 4);
        for controller in controllers() {
            let frozen = FrozenActions::freeze(&rules, controller).unwrap();
            assert_eq!(frozen.config(), &rules);
            assert_eq!(frozen.controller(), &controller);
            let snapshot = frozen.snapshot();
            assert_eq!(snapshot.version, ACTION_VERSION);
            assert_eq!(snapshot.rows.len(), 32);
            for (index, row) in frozen.rows().iter().enumerate() {
                assert_eq!(usize::from(row.index), index);
                assert_eq!(history_index(&row.observation), index);
                assert_eq!(row.observation, history_view(&rules, index as u8).unwrap());
            }
            let decoded: ActionSnapshot =
                serde_json::from_value(serde_json::to_value(&snapshot).unwrap()).unwrap();
            assert_eq!(
                FrozenActions::from_snapshot(&decoded).unwrap().snapshot(),
                snapshot
            );
        }
    }
}

#[test]
fn invalid_history_indices_and_rules_fail() {
    let rules = config(4, 5, 3, 4);
    for index in 32..=u8::MAX {
        assert!(history_view(&rules, index).is_err());
    }
    let mut invalid = rules;
    invalid.accuracy.denominator = 0;
    assert!(history_view(&invalid, 0).is_err());
}

#[test]
fn corrupt_named_snapshots_never_certify() {
    let rules = config(4, 5, 3, 4);
    for controller in controllers() {
        let base = FrozenActions::freeze(&rules, controller)
            .unwrap()
            .snapshot();
        for corruption in 0..11 {
            let mut changed = base.clone();
            match corruption {
                0 => changed.version += 1,
                1 => {
                    changed.rows.pop();
                }
                2 => changed.rows.push(changed.rows[0].clone()),
                3 => changed.rows.swap(0, 1),
                4 => changed.rows[0].index = 31,
                5 => changed.rows[0].observation.calibration_truth = true,
                6 => changed.rows[0].observation.rules.accuracy.numerator = 3,
                7 => changed.rules.accuracy.numerator = 3,
                8 => {
                    changed.rows[0].decision.posterior_true =
                        Some(crate::deduction::strategy_inference::Ratio {
                            numerator: 1,
                            denominator: 2,
                        })
                }
                9 => {
                    changed.rows[0].decision.action = match changed.rows[0].decision.action {
                        DecisionAction::Abstain => DecisionAction::Intervene,
                        DecisionAction::Intervene => DecisionAction::Abstain,
                    }
                }
                _ => {
                    changed.controller = if controller == ControllerKind::Passive {
                        ControllerKind::FixedOnly
                    } else {
                        ControllerKind::Passive
                    }
                }
            }
            assert!(
                FrozenActions::from_snapshot(&changed).is_err(),
                "accepted {controller:?} corruption {corruption}"
            );
        }
    }
}

#[test]
fn snapshot_unknown_fields_fail_at_every_nesting_level() {
    let frozen = FrozenActions::freeze(&config(4, 5, 3, 4), ControllerKind::FixedOnly).unwrap();
    let base = serde_json::to_value(frozen.snapshot()).unwrap();
    for pointer in [
        "",
        "/rules",
        "/rules/accuracy",
        "/rules/fixed_copy_prior",
        "/rules/strategic_utility",
        "/rules/permissions/0",
        "/rows/0",
        "/rows/0/observation",
        "/rows/0/observation/rules",
        "/rows/0/decision",
        "/rows/0/decision/posterior_true",
    ] {
        let mut wire = base.clone();
        wire.pointer_mut(pointer).unwrap()["extra"] = serde_json::json!(true);
        assert!(
            serde_json::from_value::<ActionSnapshot>(wire).is_err(),
            "accepted {pointer}"
        );
    }
    for posterior in [
        serde_json::json!({"numerator":0,"denominator":0}),
        serde_json::json!({"numerator":3,"denominator":2}),
    ] {
        let mut wire = base.clone();
        wire["rows"][0]["decision"]["posterior_true"] = posterior;
        assert!(serde_json::from_value::<ActionSnapshot>(wire).is_err());
    }
    let mut wire = base;
    wire["controller"] = serde_json::json!("actual_policy_or_oracle");
    assert!(serde_json::from_value::<ActionSnapshot>(wire).is_err());
}

#[test]
fn named_controller_wire_strings_are_stable() {
    for (controller, label) in controllers().into_iter().zip([
        "strategy_uniform",
        "strategy_optimization_informed",
        "fixed_only",
        "passive",
    ]) {
        assert_eq!(
            serde_json::to_value(controller).unwrap(),
            serde_json::json!(label)
        );
        assert_eq!(
            serde_json::from_value::<ControllerKind>(serde_json::json!(label)).unwrap(),
            controller
        );
    }
}

#[test]
fn strategy_freezes_match_unchanged_named_catalogs() {
    use crate::deduction::strategy_inference::{Catalog, Model};
    for q in [3, 4] {
        let rules = config(q, 5, 3, 4);
        for (controller, catalog) in [
            (ControllerKind::StrategyUniform, Catalog::uniform()),
            (
                ControllerKind::StrategyOptimizationInformed,
                Catalog::optimization_informed(),
            ),
        ] {
            let frozen = FrozenActions::freeze(&rules, controller).unwrap();
            let model = Model::new(&rules, &catalog).unwrap();
            for row in frozen.rows() {
                let expected = model.decide(&row.observation).unwrap();
                assert_eq!(
                    row.decision.posterior_true.as_ref(),
                    Some(&expected.posterior_true)
                );
                assert_eq!(row.decision.action, expected.action);
            }
        }
    }
}

#[test]
fn fixed_only_ignores_both_strategic_bits_on_all_histories() {
    for q in [3, 4] {
        let rules = config(q, 5, 3, 4);
        let frozen = FrozenActions::freeze(&rules, ControllerKind::FixedOnly).unwrap();
        let model = FixedModel::new(&rules).unwrap();
        for row in frozen.rows() {
            let view = fixed_view(&rules, fixed_index(&row.observation));
            assert_eq!(row.decision, model.decide(&view).unwrap());
            for strategic_bits in [0, 2, 8, 10] {
                let index = usize::from(row.index) & !10 | strategic_bits;
                assert_eq!(row.decision, frozen.rows()[index].decision);
            }
        }
    }
}

#[test]
fn fixed_conditional_agrees_after_marginalizing_representative_hypothetical_policies() {
    use crate::deduction::strategic_reporting::{enumerate, histories, Policy};
    for q in [3, 4] {
        let rules = config(q, 5, 3, 4);
        let model = FixedModel::new(&rules).unwrap();
        let distribution = enumerate(&rules).unwrap();
        for policy in [
            Policy::copy(),
            Policy::invert(),
            Policy::positive(),
            Policy::negative(),
            Policy::calibration_copy_live_invert(),
            Policy::new(98342).unwrap(),
            Policy::new(262143).unwrap(),
        ] {
            let mut totals = [0u64; 8];
            let mut truths = [0u64; 8];
            for row in histories(&distribution, &policy).unwrap() {
                let i = fixed_index(&row.observation);
                totals[i] += row.total_mass;
                truths[i] += row.true_mass;
            }
            for i in 0..8 {
                let decision = model.decide(&fixed_view(&rules, i)).unwrap();
                let posterior = decision.posterior_true.unwrap();
                assert_eq!(
                    (posterior.numerator, posterior.denominator),
                    (truths[i], totals[i])
                );
            }
        }
    }
}

// Independent latent enumeration: only C, T, the fixed channel's profile and
// its two signals. Strategic signals/reports and production enumeration are absent.
fn independent_fixed_masses(rules: &Config) -> ([u64; 8], [u64; 8]) {
    let mut totals = [0u64; 8];
    let mut truths = [0u64; 8];
    for c in [false, true] {
        for t in [false, true] {
            for copy in [false, true] {
                for calibration_signal in [false, true] {
                    for live_signal in [false, true] {
                        let q = &rules.accuracy;
                        let rho = &rules.fixed_copy_prior;
                        let profile_mass = if copy {
                            rho.numerator
                        } else {
                            rho.denominator - rho.numerator
                        };
                        let calibration_mass = if calibration_signal == c {
                            q.numerator
                        } else {
                            q.denominator - q.numerator
                        };
                        let live_mass = if live_signal == t {
                            q.numerator
                        } else {
                            q.denominator - q.numerator
                        };
                        // Summing the absent strategic signal channels contributes q.denominator².
                        let mass = u64::from(profile_mass)
                            * u64::from(calibration_mass)
                            * u64::from(live_mass)
                            * u64::from(q.denominator).pow(2);
                        let calibration_report = if copy {
                            calibration_signal
                        } else {
                            !calibration_signal
                        };
                        let live_report = if copy { live_signal } else { !live_signal };
                        let i = 4 * usize::from(c)
                            + 2 * usize::from(calibration_report)
                            + usize::from(live_report);
                        totals[i] += mass;
                        if t {
                            truths[i] += mass;
                        }
                    }
                }
            }
        }
    }
    (totals, truths)
}

#[test]
fn independent_latent_fixed_evidence_matches_exact_private_mass_pairs() {
    for (q, d) in [(0, 1), (1, 2), (1, 1), (3, 5), (4, 5), (7, 16)] {
        for (rho, rd) in [(0, 1), (1, 1), (3, 4), (9, 16)] {
            let rules = config(q, d, rho, rd);
            let model = FixedModel::new(&rules).unwrap();
            let (totals, truths) = independent_fixed_masses(&rules);
            assert_eq!(
                totals.iter().sum::<u64>(),
                4 * u64::from(d).pow(4) * u64::from(rd)
            );
            for i in 0..8 {
                let decision = model.decide(&fixed_view(&rules, i));
                if totals[i] == 0 {
                    assert!(matches!(decision, Err(Error::ZeroEvidence)));
                } else {
                    let decision = decision.unwrap();
                    let posterior = decision.posterior_true.unwrap();
                    assert_eq!(
                        (posterior.numerator, posterior.denominator),
                        (truths[i], totals[i])
                    );
                    assert_eq!(
                        decision.action,
                        if truths[i] > totals[i] - truths[i] {
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

#[test]
fn exact_half_accuracy_ties_abstain_for_every_named_controller() {
    for rho in [(0, 1), (3, 4), (1, 1)] {
        let rules = config(1, 2, rho.0, rho.1);
        for controller in controllers() {
            let frozen = FrozenActions::freeze(&rules, controller).unwrap();
            for row in frozen.rows() {
                assert_eq!(row.decision.action, DecisionAction::Abstain);
                if controller == ControllerKind::Passive {
                    assert_eq!(row.decision.posterior_true, None);
                } else {
                    let posterior = row.decision.posterior_true.as_ref().unwrap();
                    assert_eq!(2 * posterior.numerator, posterior.denominator);
                }
            }
        }
    }
}

#[test]
fn unsupported_endpoint_freezes_error_without_fabricated_actions() {
    for q in [0, 1] {
        for rho in [0, 1] {
            let rules = config(q, 1, rho, 1);
            let fixed = FixedModel::new(&rules).unwrap();
            let (totals, _) = independent_fixed_masses(&rules);
            assert!(totals.contains(&0));
            for (i, total) in totals.into_iter().enumerate() {
                assert_eq!(
                    matches!(
                        fixed.decide(&fixed_view(&rules, i)),
                        Err(Error::ZeroEvidence)
                    ),
                    total == 0
                );
            }
            assert!(matches!(
                FrozenActions::freeze(&rules, ControllerKind::FixedOnly),
                Err(Error::ZeroEvidence)
            ));
            for controller in [
                ControllerKind::StrategyUniform,
                ControllerKind::StrategyOptimizationInformed,
            ] {
                assert!(matches!(
                    FrozenActions::freeze(&rules, controller),
                    Err(Error::ZeroEvidence)
                ));
            }
            assert!(FrozenActions::freeze(&rules, ControllerKind::Passive).is_ok());
        }
    }
}

#[test]
fn changed_rules_are_rejected_without_mutating_model() {
    let rules = config(4, 5, 3, 4);
    let fixed = FixedModel::new(&rules).unwrap();
    let view = fixed_view(&rules, 0);
    let before = fixed.decide(&view).unwrap();
    for change in 0..4 {
        let mut different = rules.clone();
        match change {
            0 => different.accuracy.numerator = 3,
            1 => different.fixed_copy_prior.numerator = 2,
            2 => different.accuracy.denominator = 0,
            _ => different.strategic_utility = UtilityTable::aligned(),
        }
        let mut changed = view.clone();
        changed.rules = different;
        assert!(fixed.decide(&changed).is_err());
    }
    assert_eq!(fixed.decide(&view).unwrap(), before);
}

#[test]
fn all_construction_rejects_invalid_or_nonopposed_rules() {
    for invalid_case in 0..5 {
        let mut rules = config(4, 5, 3, 4);
        match invalid_case {
            0 => rules.accuracy.denominator = 0,
            1 => rules.version = 0,
            2 => rules.fixed = rules.strategic,
            3 => rules.strategic_utility = UtilityTable::aligned(),
            _ => rules.strategic_utility.intervene_true = 2,
        }
        assert!(FixedModel::new(&rules).is_err());
        for controller in controllers() {
            assert!(FrozenActions::freeze(&rules, controller).is_err());
        }
    }
}

#[test]
fn agent_renaming_preserves_decisions_and_valid_named_reconstruction() {
    let rules = config(4, 5, 3, 4);
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
    for controller in controllers() {
        let original = FrozenActions::freeze(&rules, controller).unwrap();
        let alternate = FrozenActions::freeze(&renamed, controller).unwrap();
        for (first, second) in original.rows().iter().zip(alternate.rows()) {
            assert_eq!(first.decision, second.decision);
        }
        assert_eq!(
            FrozenActions::from_snapshot(&alternate.snapshot())
                .unwrap()
                .snapshot(),
            alternate.snapshot()
        );
        let mut forged = original.snapshot();
        forged.rules = renamed.clone();
        assert!(FrozenActions::from_snapshot(&forged).is_err());
    }
}

#[test]
fn fixed_view_wire_is_exactly_four_public_fields() {
    let base = serde_json::to_value(fixed_view(&config(4, 5, 3, 4), 0)).unwrap();
    assert_eq!(base.as_object().unwrap().len(), 4);
    for field in [
        "rules",
        "calibration_truth",
        "calibration_report",
        "live_report",
    ] {
        let mut wire = base.clone();
        wire.as_object_mut().unwrap().remove(field);
        assert!(serde_json::from_value::<FixedView>(wire).is_err());
    }
    let mut wire = base;
    wire["rules"]["accuracy"]["private_signal"] = serde_json::json!(false);
    assert!(serde_json::from_value::<FixedView>(wire).is_err());
}
