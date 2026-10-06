use super::super::*;
use crate::deduction::strategic_reporting::{
    Config, DecisionAction, Policy, Probability, UtilityTable,
};
use crate::deduction::strategy_inference::canonical_bits;
use serde_json::Value;
use std::collections::BTreeMap;

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
fn reference() -> Value {
    serde_json::from_str(include_str!("attack-reference.json")).unwrap()
}

#[test]
fn passive_best_response_uses_smallest_canonical_encoding() {
    let actions = FrozenActions::freeze(&config(4), ControllerKind::Passive).unwrap();
    let response = exact_best_response(&AttackBasis::build(&actions).unwrap()).unwrap();
    assert_eq!(response.policy.bits, 0);
    assert_eq!(response.score.utility_numerator, 0);
}

#[test]
fn every_fitness_and_optimum_matches_independent_full_world_reference() {
    let fixture = reference();
    let policies = canonical_policies();
    let bits: Vec<_> = policies.iter().map(|p| p.bits).collect();
    assert_eq!(
        serde_json::to_value(&bits).unwrap(),
        fixture["canonical_bits"]
    );
    for env in fixture["environments"].as_array().unwrap() {
        let q = env["q"][0].as_u64().unwrap() as u16;
        for controller in controllers() {
            let name = serde_json::to_value(controller).unwrap();
            let name = name.as_str().unwrap();
            let actions = FrozenActions::freeze(&config(q), controller).unwrap();
            let basis = AttackBasis::build(&actions).unwrap();
            let table = fitness_table(&basis).unwrap();
            assert_eq!(table.len(), 1024);
            for (i, policy) in policies.iter().enumerate() {
                let direct = score(&actions, policy).unwrap();
                let expected = env["fitness_utility_numerators"][name][i].as_i64().unwrap();
                assert_eq!(table[i].canonical_bits, policy.bits);
                assert_eq!(
                    i128::from(table[i].utility_numerator) * 10000,
                    i128::from(expected) * i128::from(basis.denominator()),
                    "q={q}, {name}, bits={}",
                    policy.bits
                );
                assert_eq!(direct.utility_numerator, table[i].utility_numerator);
                assert_eq!(direct.payoff_numerator, -direct.utility_numerator);
                assert_eq!(direct.rules, config(q));
                direct.validate().unwrap();
            }
            let optimum = exact_best_response(&basis).unwrap();
            let expected = &env["optima"][name];
            assert_eq!(
                optimum.policy.bits,
                expected["canonical_bits"].as_u64().unwrap() as u32
            );
            assert_eq!(
                i128::from(optimum.score.utility_numerator)
                    * expected["utility"][1].as_i64().unwrap() as i128,
                expected["utility"][0].as_i64().unwrap() as i128
                    * i128::from(optimum.score.denominator)
            );
            assert_eq!(
                table
                    .iter()
                    .filter(|r| r.utility_numerator == optimum.score.utility_numerator)
                    .count(),
                expected["canonical_tie_count"].as_u64().unwrap() as usize
            );
            assert_eq!(score(&actions, &optimum.policy).unwrap(), optimum.score);
        }
    }
}

#[test]
fn every_raw_alias_has_identical_fitness_and_256_fold_multiplicity() {
    let policies = canonical_policies();
    let mut counts: BTreeMap<u32, usize> = policies.iter().map(|p| (p.bits, 0)).collect();
    let bases: Vec<_> = [4, 3]
        .into_iter()
        .flat_map(|q| {
            controllers().into_iter().map(move |kind| {
                AttackBasis::build(&FrozenActions::freeze(&config(q), kind).unwrap()).unwrap()
            })
        })
        .collect();
    for bits in 0..1 << 18 {
        let raw = Policy::new(bits).unwrap();
        let canonical = Policy::new(canonical_bits(&raw).unwrap()).unwrap();
        *counts.get_mut(&canonical.bits).unwrap() += 1;
        for basis in &bases {
            assert_eq!(
                basis.fitness(&raw).unwrap(),
                basis.fitness(&canonical).unwrap()
            );
        }
    }
    assert_eq!(counts.len(), 1024);
    assert!(counts.values().all(|count| *count == 256));
}

#[test]
fn all_live_row_flips_match_direct_scores_and_independent_deltas() {
    let fixture = reference();
    for env in fixture["environments"].as_array().unwrap() {
        let q = env["q"][0].as_u64().unwrap() as u16;
        for kind in controllers() {
            let name = serde_json::to_value(kind).unwrap();
            let name = name.as_str().unwrap();
            let actions = FrozenActions::freeze(&config(q), kind).unwrap();
            let basis = AttackBasis::build(&actions).unwrap();
            for calibration in 0..4u32 {
                let row = &basis.rows()[calibration as usize];
                let expected = &env["basis"][name][calibration as usize];
                assert_eq!(row.calibration, calibration as u8);
                assert_eq!(serde_json::to_value(row).unwrap(), *expected);
                for live_row in 0..16 {
                    let flipped = Policy::new(calibration | 1 << (2 + live_row)).unwrap();
                    let direct = score(&actions, &flipped).unwrap();
                    assert_eq!(direct.utility_numerator, basis.fitness(&flipped).unwrap());
                    let delta = direct.utility_numerator - row.base_utility_numerator;
                    assert_eq!(row.deltas[live_row].unwrap_or(0), delta);
                    // Distinguish structurally unreachable rows from reachable zero deltas.
                    let private_calibration_signal = live_row & 8 != 0;
                    let own_calibration_report = live_row & 4 != 0;
                    assert_eq!(
                        row.deltas[live_row].is_some(),
                        own_calibration_report
                            == (calibration >> u32::from(private_calibration_signal) & 1 != 0)
                    );
                }
            }
        }
    }
}

#[test]
fn nonconstant_multiple_live_bits_recombine_without_cross_terms() {
    let actions =
        FrozenActions::freeze(&config(4), ControllerKind::StrategyOptimizationInformed).unwrap();
    let basis = AttackBasis::build(&actions).unwrap();
    // Calibration identity: private signal false -> report false; true -> report true.
    // Rows 1, 3, 13, 15 use both verified truths and both live-signal positions.
    let policy = Policy::new(2 | (1 << 3) | (1 << 5) | (1 << 15) | (1 << 17)).unwrap();
    assert_eq!(
        score(&actions, &policy).unwrap().utility_numerator,
        basis.rows()[2].base_utility_numerator
            + [1, 3, 13, 15]
                .into_iter()
                .map(|r| basis.rows()[2].deltas[r].unwrap())
                .sum::<i64>()
    );
    assert_eq!(
        basis.fitness(&policy).unwrap(),
        score(&actions, &policy).unwrap().utility_numerator
    );
}

#[test]
fn zero_delta_and_calibration_ties_leave_bits_false() {
    for kind in [ControllerKind::FixedOnly, ControllerKind::Passive] {
        let basis = AttackBasis::build(&FrozenActions::freeze(&config(4), kind).unwrap()).unwrap();
        assert!(basis
            .rows()
            .iter()
            .flat_map(|r| r.deltas)
            .flatten()
            .all(|d| d == 0));
        assert_eq!(exact_best_response(&basis).unwrap().policy.bits, 0);
    }
}

#[test]
fn maximum_probability_denominators_preserve_exact_scores() {
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
    for kind in controllers() {
        let actions = FrozenActions::freeze(&c, kind).unwrap();
        let basis = AttackBasis::build(&actions).unwrap();
        assert_eq!(basis.denominator(), 4_194_304);
        let best = exact_best_response(&basis).unwrap();
        assert_eq!(best.score, score(&actions, &best.policy).unwrap());
        best.score.validate().unwrap();
    }
}

#[test]
fn invalid_policy_encodings_fail_before_fitness_or_history_scoring() {
    let actions = FrozenActions::freeze(&config(4), ControllerKind::Passive).unwrap();
    let basis = AttackBasis::build(&actions).unwrap();
    for bits in [1 << 18, u32::MAX] {
        assert!(score(&actions, &Policy { bits }).is_err());
        assert!(basis.fitness(&Policy { bits }).is_err());
    }
}

#[test]
fn score_validation_rejects_forged_rules_bounds_and_extreme_numerators() {
    let valid = Score {
        rules: config(4),
        denominator: 1 << 30,
        payoff_numerator: 1 << 29,
        utility_numerator: -(1 << 29),
    };
    valid.validate().unwrap();
    let mut negative = valid.clone();
    negative.payoff_numerator = -(1 << 29);
    negative.utility_numerator = 1 << 29;
    negative.validate().unwrap();
    for (payoff, utility) in [
        (1 << 29 | 1, -(1 << 29 | 1)),
        (1, 1),
        (i64::MIN, i64::MAX),
        (i64::MAX, i64::MIN),
    ] {
        let mut forged = valid.clone();
        forged.payoff_numerator = payoff;
        forged.utility_numerator = utility;
        assert!(forged.validate().is_err());
    }
    for denominator in [0, (1 << 30) + 1, u64::MAX] {
        let mut forged = valid.clone();
        forged.denominator = denominator;
        assert!(forged.validate().is_err());
    }
    let mut forged = valid.clone();
    forged.rules.strategic_utility = UtilityTable::aligned();
    assert!(forged.validate().is_err());
    let mut forged = valid.clone();
    forged.rules.version += 1;
    assert!(forged.validate().is_err());
}

#[test]
fn all_controller_actions_match_reference_history_coordinates() {
    let fixture = reference();
    for env in fixture["environments"].as_array().unwrap() {
        let q = env["q"][0].as_u64().unwrap() as u16;
        for kind in controllers() {
            let name = serde_json::to_value(kind).unwrap();
            let name = name.as_str().unwrap();
            let actions = FrozenActions::freeze(&config(q), kind).unwrap();
            for (index, row) in actions.rows().iter().enumerate() {
                let expected = &env["action_tables"][name][index];
                assert_eq!(
                    row.index as usize,
                    expected["index"].as_u64().unwrap() as usize
                );
                assert_eq!(
                    row.decision.action == DecisionAction::Intervene,
                    expected["intervene"].as_bool().unwrap()
                );
                if let Some(posterior) = &row.decision.posterior_true {
                    assert_eq!(
                        u128::from(posterior.numerator)
                            * expected["posterior_true"][1].as_u64().unwrap() as u128,
                        u128::from(posterior.denominator)
                            * expected["posterior_true"][0].as_u64().unwrap() as u128
                    );
                } else {
                    assert!(expected["posterior_true"].is_null());
                }
            }
            let mut forged = actions.snapshot();
            forged.rows[3].decision.action =
                if forged.rows[3].decision.action == DecisionAction::Intervene {
                    DecisionAction::Abstain
                } else {
                    DecisionAction::Intervene
                };
            assert!(FrozenActions::from_snapshot(&forged).is_err());
        }
    }
}

#[test]
fn public_history_scoring_rejects_malformed_masses_rules_and_order() {
    use super::super::scoring::score_histories;
    use crate::deduction::strategic_reporting::{enumerate, histories};
    let c = config(4);
    let actions = FrozenActions::freeze(&c, ControllerKind::Passive).unwrap();
    let distribution = enumerate(&c).unwrap();
    let rows = histories(&distribution, &Policy::copy()).unwrap();
    score_histories(&actions, distribution.denominator(), &rows).unwrap();
    let mut forged = rows.clone();
    forged[0].true_mass = forged[0].total_mass + 1;
    assert!(score_histories(&actions, distribution.denominator(), &forged).is_err());
    let mut forged = rows.clone();
    forged[0].total_mass += 1;
    assert!(score_histories(&actions, distribution.denominator(), &forged).is_err());
    let mut forged = rows.clone();
    forged[0].true_mass += 1;
    assert!(score_histories(&actions, distribution.denominator(), &forged).is_err());
    let mut forged = rows.clone();
    forged[0].observation.rules = config(3);
    assert!(score_histories(&actions, distribution.denominator(), &forged).is_err());
    let mut forged = rows.clone();
    forged.swap(0, 1);
    assert!(score_histories(&actions, distribution.denominator(), &forged).is_err());
    assert!(score_histories(&actions, distribution.denominator(), &rows[..31]).is_err());
    assert!(score_histories(&actions, 0, &rows).is_err());
    assert!(score_histories(&actions, (1 << 30) + 1, &rows).is_err());
}

#[test]
fn history_contributions_preserve_gain_ties_and_zero_evidence() {
    use super::super::scoring::history_contribution;
    use crate::deduction::strategic_reporting::HistoryMass;
    for (total, yes, gain) in [(10, 7, 4), (10, 3, -4), (10, 5, 0)] {
        let h = HistoryMass {
            observation: history_view(&config(4), 0).unwrap(),
            total_mass: total,
            true_mass: yes,
        };
        for action in [DecisionAction::Intervene, DecisionAction::Abstain] {
            let contribution = history_contribution(&h, action).unwrap();
            let payoff = if action == DecisionAction::Intervene {
                gain
            } else {
                0
            };
            assert_eq!(contribution.gain, gain);
            assert_eq!(contribution.payoff, payoff);
            assert_eq!(contribution.utility, -payoff);
            assert_eq!(contribution.optimal, gain.max(0));
            assert_eq!(contribution.regret, gain.max(0) - payoff);
        }
    }
    let mut h = HistoryMass {
        observation: history_view(&config(4), 0).unwrap(),
        total_mass: 0,
        true_mass: 0,
    };
    assert_eq!(
        history_contribution(&h, DecisionAction::Abstain).unwrap_err(),
        Error::ZeroEvidence
    );
    h.total_mass = 1;
    h.true_mass = 2;
    assert!(history_contribution(&h, DecisionAction::Abstain).is_err());
    h.total_mass = u64::MAX;
    h.true_mass = 0;
    assert!(history_contribution(&h, DecisionAction::Intervene).is_err());
}
