use super::super::*;

fn fixture_protocol(calibration_rounds: u8, roles: [Knowledge; 2]) -> Protocol {
    Protocol {
        calibration_rounds,
        pair: Pair { roles },
        prior_mode: PriorMode::Treatment,
        ids: Ids {
            surface: "field".into(),
            agents: ["first".into(), "second".into()],
        },
    }
}
fn score(n: i64) -> ScoreFraction {
    ScoreFraction::new(n, 1).unwrap()
}
fn prob(n: u64, d: u64) -> Probability {
    Probability::new(n, d).unwrap()
}
#[test]
fn fully_inspecting_baseline_accounts_for_waits_and_calibration() {
    let panel = evaluate_panel(
        &fixture_protocol(3, [Knowledge::NoCommunication; 2]),
        Environment::InFamily(Mechanism::Inert),
    )
    .unwrap();
    assert_eq!(panel.episode_count(), 256);
    assert_eq!(panel.agent_expected_credits(Role::A), score(42));
    assert_eq!(panel.agent_expected_accuracy(Role::A), Some(prob(1, 1)));
    assert_eq!(panel.agent_expected_correct_count(Role::A), Some(score(4)));
    assert_eq!(panel.agent_expected_net(Role::A), Some(score(6)));
    assert_eq!(panel.aggregate.agents[0].benefit_gross, Some(score(0)));
    assert_eq!(panel.aggregate.expected_group_net, Some(score(12)));
}
#[test]
fn delayed_shared_data_saves_cost_without_increasing_gross_reward() {
    let protocol = fixture_protocol(3, [Knowledge::Unknown; 2]);
    let episode = run_episode(
        &protocol,
        Environment::InFamily(Mechanism::SharedPersistent),
        &EpisodeBits::from_index(0).unwrap(),
        &Ensemble::build(&protocol).unwrap(),
    )
    .unwrap();
    for m in &episode.metrics {
        assert_eq!(m.spent, 30);
        assert_eq!(m.attempted_sends, 4);
        assert_eq!(m.other_agent_task_lineage_reads, 4);
        assert_eq!(m.inspections, 0);
        assert_eq!(m.terminal.as_ref().unwrap().gross_reward, 48);
    }
    assert_eq!(episode.group_net_utility, Some(36));
}
#[test]
fn unsupported_stale_read_stops_before_later_action_and_preserves_last_belief() {
    let mut protocol = fixture_protocol(0, [Knowledge::Unknown; 2]);
    protocol.prior_mode = PriorMode::Stale(Mechanism::SharedPersistent);
    let episode = run_episode(
        &protocol,
        Environment::InFamily(Mechanism::SharedResetting),
        &EpisodeBits::from_index(0).unwrap(),
        &Ensemble::build(&protocol).unwrap(),
    )
    .unwrap();
    let f = episode.failure.as_ref().unwrap();
    assert_eq!(f.role, Role::B);
    assert_eq!(f.spent, [3, 3]);
    assert_eq!(
        f.checkpoint,
        Checkpoint::AfterSlot(Position {
            phase: Phase::Live { trial: 0 },
            round: 2,
            slot: 2
        })
    );
    assert_eq!(episode.privileged.len(), 6);
    assert!(episode.metrics.iter().all(|m| m.terminal.is_none()));
    assert_eq!(f.last_supported_belief.models[0], prob(1, 1));
}
#[test]
fn dataflip_can_be_supported_confident_and_wrong_without_catalog_true_score() {
    let protocol = fixture_protocol(3, [Knowledge::Known; 2]);
    let episode = run_episode(
        &protocol,
        Environment::DataFlip,
        &EpisodeBits::from_index(0).unwrap(),
        &Ensemble::build(&protocol).unwrap(),
    )
    .unwrap();
    assert!(episode.failure.is_none());
    for m in &episode.metrics {
        assert_eq!(m.terminal.as_ref().unwrap().correct, [false; 4]);
        assert_eq!(m.true_model_probability, None);
        assert_eq!(m.unique_true_identification, None);
    }
}
#[test]
fn negative_controls_never_claim_other_role_task_delivery() {
    for mechanism in [
        Mechanism::SharedResetting,
        Mechanism::PrivatePersistent,
        Mechanism::Inert,
    ] {
        let protocol = fixture_protocol(3, [Knowledge::Unknown; 2]);
        let episode = run_episode(
            &protocol,
            Environment::InFamily(mechanism),
            &EpisodeBits::from_index(255).unwrap(),
            &Ensemble::build(&protocol).unwrap(),
        )
        .unwrap();
        for m in &episode.metrics {
            assert_eq!(m.other_agent_task_lineage_reads, 0);
            assert_eq!(m.inspections, 4);
        }
    }
}
fn mechanism(s: &str) -> Mechanism {
    match s {
        "SP" => Mechanism::SharedPersistent,
        "SR" => Mechanism::SharedResetting,
        "PP" => Mechanism::PrivatePersistent,
        "IN" => Mechanism::Inert,
        _ => panic!("fixture mechanism"),
    }
}
fn knowledge(s: &str) -> Knowledge {
    match s {
        "unknown" => Knowledge::Unknown,
        "known" => Knowledge::Known,
        "none" => Knowledge::NoCommunication,
        _ => panic!("fixture knowledge {s}"),
    }
}
#[test]
fn bounded_independent_references_match_terminal_counts_and_paired_sensitivity() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/evaluation-reference.json")).unwrap();
    for row in fixture["panels"].as_array().unwrap() {
        let pair = row["pair"].as_array().unwrap();
        let mut protocol = fixture_protocol(
            row["calibration_rounds"].as_u64().unwrap() as u8,
            [
                knowledge(pair[0].as_str().unwrap()),
                knowledge(pair[1].as_str().unwrap()),
            ],
        );
        if row["kind"] == "stale" {
            protocol.prior_mode =
                PriorMode::Stale(mechanism(row["old_mechanism"].as_str().unwrap()));
        }
        let environment = if row["environment"] == "DataFlip" {
            Environment::DataFlip
        } else {
            Environment::InFamily(mechanism(row["environment"].as_str().unwrap()))
        };
        let panel = evaluate_panel(&protocol, environment).unwrap();
        let actual = serde_json::to_value(&panel.aggregate).unwrap();
        let expected = &row["aggregate"];
        for key in [
            "total",
            "valid",
            "failed",
            "failure_mass",
            "expected_group_net",
        ] {
            assert_eq!(actual[key], expected[key], "{row}: {key}");
        }
        for role in 0..2 {
            for key in [
                "expected_credits",
                "expected_accuracy",
                "expected_correct_count",
                "expected_gross",
                "expected_net",
                "benefit_gross",
                "benefit_net",
            ] {
                assert_eq!(
                    actual["agents"][role][key], expected["agents"][role][key],
                    "{row}: role {role} {key}"
                );
            }
            let counts = &expected["agents"][role]["expected_counts"];
            for key in [
                "attempted_sends",
                "calibration_reads",
                "live_reads",
                "read",
                "write",
                "wait",
                "inspect",
                "decoded_data",
                "causal_transfers",
            ] {
                let sum: i64 = panel
                    .episodes
                    .iter()
                    .map(|e| {
                        let m = &e.metrics[role];
                        i64::from(match key {
                            "attempted_sends" => m.attempted_sends,
                            "calibration_reads" => m.calibration_reads,
                            "live_reads" => m.live_reads,
                            "read" => m.reads,
                            "write" => m.calibration_writes + m.attempted_sends,
                            "wait" => m.waits,
                            "inspect" => m.inspections,
                            "decoded_data" => m.decoded_data,
                            "causal_transfers" => m.other_agent_task_lineage_reads,
                            _ => unreachable!(),
                        })
                    })
                    .sum();
                let want = if counts[key].is_null() {
                    serde_json::to_value(score(0)).unwrap()
                } else {
                    counts[key].clone()
                };
                assert_eq!(
                    serde_json::to_value(ScoreFraction::new(sum, 256).unwrap()).unwrap(),
                    want,
                    "role {role} {key} setting {row}"
                );
            }
            for (property, name) in [
                (Property::Visibility, "visibility"),
                (Property::Retention, "retention"),
                (Property::UsefulChannel, "useful_channel"),
                (Property::TrueMechanism, "true_mechanism"),
            ] {
                let want = &expected["agents"][role]["discoveries"][name][0]["value"];
                for e in &panel.episodes {
                    let certainty = &e.metrics[role]
                        .discoveries
                        .iter()
                        .find(|d| d.property == property)
                        .unwrap()
                        .certainty;
                    match certainty {
                        Certainty::Censored => assert_eq!(want, "censored", "{row} {role} {name}"),
                        Certainty::Supplied => {
                            assert_eq!(want["supplied"], true, "{row} {role} {name}")
                        }
                        Certainty::Acquired {
                            checkpoint,
                            credits,
                        } => {
                            let Checkpoint::AfterSlot(p) = checkpoint else {
                                panic!("discovery must follow slot")
                            };
                            assert_eq!(want["credits"], *credits);
                            assert_eq!(want["checkpoint"][2], p.round);
                            assert_eq!(want["checkpoint"][3], p.slot);
                        }
                    }
                }
            }
            if !expected["agents"][role]["final_posterior"].is_null() {
                assert_eq!(
                    serde_json::to_value(
                        panel.episodes[0].metrics[role]
                            .final_belief
                            .as_ref()
                            .unwrap()
                            .models
                    )
                    .unwrap(),
                    expected["agents"][role]["final_posterior"]
                );
            }
        }
        for want in expected["paired_sensitivity"].as_array().unwrap() {
            let sender = if want["sender"] == 0 {
                Role::A
            } else {
                Role::B
            };
            let trial = want["trial"].as_u64().unwrap() as u8;
            for name in ["read", "posterior", "prediction"] {
                let mut counts = std::collections::BTreeMap::<String, u64>::new();
                for s in panel
                    .sensitivity
                    .iter()
                    .filter(|s| s.sender == sender && s.trial == trial)
                {
                    let value = match name {
                        "read" => s.read_changed,
                        "posterior" => s.posterior_changed,
                        _ => s.prediction_changed,
                    };
                    *counts
                        .entry(
                            match value {
                                Some(true) => "true",
                                Some(false) => "false",
                                None => "unavailable",
                            }
                            .into(),
                        )
                        .or_default() += 1;
                }
                assert_eq!(
                    serde_json::to_value(counts).unwrap(),
                    want[name],
                    "{row} {sender:?} {trial} {name}"
                );
            }
        }
    }
}

#[test]
fn old_full_calibration_records_acquired_point_mass_and_separate_costs() {
    for old in Mechanism::ALL {
        let ids = fixture_protocol(0, [Knowledge::Unknown; 2]).ids;
        let origin = acquire_transfer_origin(old, &ids).unwrap();
        assert_eq!(origin.spent, [6, 6]);
        for role in 0..2 {
            assert_eq!(origin.posterior[role].models[old.index()], prob(1, 1));
            assert_eq!(origin.acquisition[role].own_prior, OwnPrior::Uniform);
            assert!(origin.acquisition[role]
                .steps
                .iter()
                .flat_map(|s| &s.prefix.entries)
                .all(|e| !matches!(e, LocalEntry::PrivateBit { .. })));
        }
    }
}
#[test]
fn every_old_new_transfer_restarts_uniform_or_retains_old_prior_with_fresh_budget() {
    for old in Mechanism::ALL {
        for new in Mechanism::ALL {
            for rounds in 0..=3 {
                let mut protocol = fixture_protocol(rounds, [Knowledge::Unknown; 2]);
                protocol.prior_mode = PriorMode::RestartUniform;
                let episode = run_episode(
                    &protocol,
                    Environment::InFamily(new),
                    &EpisodeBits::from_index(0).unwrap(),
                    &Ensemble::build(&protocol).unwrap(),
                )
                .unwrap();
                assert!(episode.failure.is_none());
                for role in 0..2 {
                    assert_eq!(episode.local[role].steps[0].credits, 48);
                    assert_eq!(
                        episode.local[role].steps[0].belief.as_ref().unwrap().models,
                        [prob(1, 4); 4]
                    );
                    assert!(episode.local[role].steps[0].prefix.entries.is_empty());
                }
                protocol.prior_mode = PriorMode::Stale(old);
                let stale = run_episode(
                    &protocol,
                    Environment::InFamily(new),
                    &EpisodeBits::from_index(0).unwrap(),
                    &Ensemble::build(&protocol).unwrap(),
                )
                .unwrap();
                for role in 0..2 {
                    assert_eq!(stale.local[role].steps[0].credits, 48);
                    assert_eq!(
                        stale.local[role].steps[0].belief.as_ref().unwrap().models[old.index()],
                        prob(1, 1)
                    );
                    assert_eq!(
                        stale.metrics[role].prior_origin,
                        PriorOrigin::RetainedOldAcquisition { old_mechanism: old }
                    );
                    assert!(stale.metrics[role].spent <= 42);
                }
                if old == new {
                    assert!(stale.failure.is_none());
                }
            }
        }
    }
}
#[test]
fn renamed_ids_preserve_posterior_actions_costs_and_utility_for_bounded_panels() {
    for mechanism in Mechanism::ALL {
        for knowledge in [
            Knowledge::Unknown,
            Knowledge::Known,
            Knowledge::NoCommunication,
        ] {
            let protocol = fixture_protocol(3, [knowledge; 2]);
            let mut renamed = protocol.clone();
            renamed.ids = Ids {
                surface: "z-surface-17".into(),
                agents: ["z-agent-91".into(), "z-agent-32".into()],
            };
            let a = evaluate_panel(&protocol, Environment::InFamily(mechanism)).unwrap();
            let mut b = evaluate_panel(&renamed, Environment::InFamily(mechanism)).unwrap();
            b.protocol.ids = protocol.ids.clone();
            for e in &mut b.episodes {
                for trace in &mut e.local {
                    for step in &mut trace.steps {
                        step.prefix.ids = protocol.ids.clone();
                    }
                }
                if let Some(f) = &mut e.failure {
                    f.prefix.ids = protocol.ids.clone();
                }
            }
            assert_eq!(a, b);
        }
    }
}

#[test]
fn wire_sequence_and_trial_indices_are_bounded() {
    let value = serde_json::json!({"sequence":256,"paired_sequence":0,"trial":0,"sender":"A","read_changed":null,"posterior_changed":null,"prediction_changed":null});
    assert!(serde_json::from_value::<Sensitivity>(value).is_err());
    let value = serde_json::json!({"sequence":0,"paired_sequence":0,"trial":4,"sender":"A","read_changed":null,"posterior_changed":null,"prediction_changed":null});
    assert!(serde_json::from_value::<Sensitivity>(value).is_err());
}

#[test]
fn no_communication_keeps_unexplored_uniform_belief_and_censored_discovery() {
    let protocol = fixture_protocol(3, [Knowledge::NoCommunication; 2]);
    let episode = run_episode(
        &protocol,
        Environment::InFamily(Mechanism::SharedPersistent),
        &EpisodeBits::from_index(0).unwrap(),
        &Ensemble::build(&protocol).unwrap(),
    )
    .unwrap();
    for m in &episode.metrics {
        assert_eq!(m.final_belief.as_ref().unwrap().models, [prob(1, 4); 4]);
        assert_eq!(m.true_model_probability, None);
        assert_eq!(m.unique_true_identification, None);
        assert!(m
            .discoveries
            .iter()
            .all(|d| d.certainty == Certainty::Censored));
    }
}

#[test]
fn pp_own_calibration_write_is_visible_without_other_agent_delivery() {
    let protocol = fixture_protocol(1, [Knowledge::Unknown; 2]);
    let episode = run_episode(
        &protocol,
        Environment::InFamily(Mechanism::PrivatePersistent),
        &EpisodeBits::from_index(0).unwrap(),
        &Ensemble::build(&protocol).unwrap(),
    )
    .unwrap();
    let read = episode
        .privileged
        .iter()
        .find(|s| {
            s.event.position
                == Position {
                    phase: Phase::Calibration,
                    round: 1,
                    slot: 4,
                }
        })
        .unwrap();
    assert_eq!(read.event.outcome, Outcome::Read(Symbol::Probe0));
    assert_eq!(read.read_lineage.as_ref().unwrap().writer, Role::A);
    assert!(episode
        .metrics
        .iter()
        .all(|m| m.other_agent_task_lineage_reads == 0));
}
#[test]
fn inert_writes_are_paid_and_accepted_but_leave_blank_without_lineage() {
    let protocol = fixture_protocol(1, [Knowledge::Unknown; 2]);
    let episode = run_episode(
        &protocol,
        Environment::InFamily(Mechanism::Inert),
        &EpisodeBits::from_index(0).unwrap(),
        &Ensemble::build(&protocol).unwrap(),
    )
    .unwrap();
    assert!(episode.privileged.iter().any(
        |s| matches!(s.event.action, Action::Write(_)) && s.event.outcome == Outcome::Accepted
    ));
    for step in &episode.privileged {
        if step.event.action == Action::Read {
            assert_eq!(step.event.outcome, Outcome::Read(Symbol::Blank));
            assert!(step.read_lineage.is_none());
        }
    }
    assert_eq!(episode.metrics[0].spent, 38);
}
#[test]
fn resetting_surface_loses_handshake_lineage_before_delayed_round_two_read() {
    let protocol = fixture_protocol(2, [Knowledge::Unknown; 2]);
    let episode = run_episode(
        &protocol,
        Environment::InFamily(Mechanism::SharedResetting),
        &EpisodeBits::from_index(0).unwrap(),
        &Ensemble::build(&protocol).unwrap(),
    )
    .unwrap();
    let same_round = episode
        .privileged
        .iter()
        .find(|s| {
            s.event.position
                == Position {
                    phase: Phase::Calibration,
                    round: 1,
                    slot: 4,
                }
        })
        .unwrap();
    assert_eq!(same_round.event.outcome, Outcome::Read(Symbol::Ack0));
    assert_eq!(same_round.read_lineage.as_ref().unwrap().writer, Role::B);
    for slot in [1, 2] {
        let delayed = episode
            .privileged
            .iter()
            .find(|s| {
                s.event.position
                    == Position {
                        phase: Phase::Calibration,
                        round: 2,
                        slot,
                    }
            })
            .unwrap();
        assert_eq!(delayed.event.outcome, Outcome::Read(Symbol::Blank));
        assert!(delayed.read_lineage.is_none());
    }
}
