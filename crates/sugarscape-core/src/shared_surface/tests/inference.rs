use super::super::*;
use std::collections::BTreeMap;

fn protocol(rounds: u8, roles: [Knowledge; 2]) -> Protocol {
    Protocol {
        calibration_rounds: rounds,
        pair: Pair { roles },
        prior_mode: PriorMode::Treatment,
        ids: Ids {
            surface: "field".into(),
            agents: ["first".into(), "second".into()],
        },
    }
}
fn p(n: u64, d: u64) -> Probability {
    Probability::new(n, d).unwrap()
}
fn fixture_live_a_view(weights: [u64; 4], total: u64, private_bit: Option<bool>) -> LocalView {
    assert_eq!(weights.iter().sum::<u64>(), total);
    let bit = private_bit.expect("live fixture requires current own private bit");
    LocalView {
        prefix: LocalPrefix {
            role: Role::A,
            ids: protocol(0, [Knowledge::Unknown; 2]).ids,
            checkpoint: Checkpoint::Action(Position {
                phase: Phase::Live { trial: 0 },
                round: 1,
                slot: 1,
            }),
            own_prior: OwnPrior::Uniform,
            entries: vec![
                LocalEntry::Reset {
                    phase: Phase::Live { trial: 0 },
                },
                LocalEntry::PrivateBit { trial: 0, bit },
            ],
        },
        credits: 48,
        private_bit,
        belief: Belief {
            models: weights.map(|w| p(w, total)),
            target: Some(p(1, 2)),
        },
    }
}

#[test]
fn half_probability_inspects_instead_of_sending() {
    assert_eq!(
        decide(
            &fixture_live_a_view([1, 0, 1, 0], 2, Some(false)),
            Knowledge::Unknown
        )
        .unwrap()
        .action,
        Action::InspectOwnTarget
    );
}
#[test]
fn target_ties_predict_zero_and_prediction_requires_terminal_checkpoint() {
    let mut view = fixture_live_a_view([1; 4], 4, Some(false));
    assert!(matches!(predict(&view), Err(Error::InvalidAction(_))));
    view.prefix.checkpoint = Checkpoint::Prediction { trial: 0 };
    assert!(!predict(&view).unwrap());
    view.belief.target = Some(p(2, 3));
    assert!(predict(&view).unwrap());
}
#[test]
fn b_rechecks_after_read_and_action_requires_before_slot() {
    let mut view = fixture_live_a_view([3, 1, 0, 0], 4, Some(false));
    view.prefix.role = Role::B;
    let position = Position {
        phase: Phase::Live { trial: 0 },
        round: 2,
        slot: 2,
    };
    view.prefix.checkpoint = Checkpoint::Action(position);
    assert_eq!(
        decide(&view, Knowledge::Unknown).unwrap().action,
        Action::Read
    );
    view.prefix.entries.push(LocalEntry::Action(Event {
        position,
        action: Action::Read,
        outcome: Outcome::Read(Symbol::Blank),
        credits_after: 47,
    }));
    view.credits = 47;
    view.prefix.checkpoint = Checkpoint::AfterSlot(position);
    assert!(matches!(
        decide(&view, Knowledge::Unknown),
        Err(Error::InvalidAction(_))
    ));
    view.prefix.checkpoint = Checkpoint::Action(Position {
        slot: 3,
        ..position
    });
    view.belief.models = [p(1, 4), p(3, 4), p(0, 1), p(0, 1)];
    assert_eq!(
        decide(&view, Knowledge::Unknown).unwrap().action,
        Action::InspectOwnTarget
    );
}
#[test]
fn model_mass_and_current_private_bit_are_validated() {
    let mut view = fixture_live_a_view([1; 4], 4, Some(false));
    view.private_bit = None;
    assert!(decide(&view, Knowledge::Unknown).is_err());
    view.private_bit = Some(false);
    view.belief.models = [p(0, 1); 4];
    assert!(decide(&view, Knowledge::Unknown).is_err());
}
#[test]
fn bounded_family_and_explicit_time_zero_priors() {
    let protocol = protocol(0, [Knowledge::Known; 2]);
    let ensemble = Ensemble::build(&protocol).unwrap();
    assert_eq!(ensemble.candidate_count(), 1024);
    for model in Mechanism::ALL {
        let prefix = LocalPrefix {
            role: Role::A,
            ids: protocol.ids.clone(),
            checkpoint: Checkpoint::EpisodeStart,
            own_prior: OwnPrior::PointMass(model),
            entries: vec![],
        };
        assert_eq!(
            ensemble.infer(&prefix).unwrap(),
            Belief {
                models: Mechanism::ALL.map(|m| p(u64::from(m == model), 1)),
                target: None
            }
        );
    }
}

// Test-only actual rollout. It reveals only the current own bit and local outcome.
// Returning every public checkpoint lets tests compare hidden-world equivalence.
fn rollout(
    protocol: &Protocol,
    ensemble: &Ensemble,
    model: Mechanism,
    sequence: u16,
) -> Result<Vec<[LocalView; 2]>, Error> {
    let bits = EpisodeBits::from_index(sequence)?;
    let mut world = World::new(Environment::InFamily(model), protocol.ids.clone());
    let empty = Belief {
        models: [p(1, 4); 4],
        target: None,
    };
    let mut views = std::array::from_fn(|i| LocalView {
        prefix: LocalPrefix {
            role: if i == 0 { Role::A } else { Role::B },
            ids: protocol.ids.clone(),
            checkpoint: Checkpoint::EpisodeStart,
            own_prior: match protocol.prior_mode {
                PriorMode::Stale(old) => OwnPrior::PointMass(old),
                _ if protocol.pair.roles[i] == Knowledge::Known => OwnPrior::PointMass(model),
                _ => OwnPrior::Uniform,
            },
            entries: vec![],
        },
        credits: 48,
        private_bit: None,
        belief: empty.clone(),
    });
    let mut result = vec![];
    for view in &mut views {
        view.belief = ensemble.infer(&view.prefix)?;
    }
    result.push(views.clone());
    let mut phase = None;
    for (role, position) in protocol.schedule()? {
        if phase != Some(position.phase) {
            world.reset(position.phase);
            phase = Some(position.phase);
            for view in &mut views {
                view.prefix.entries.push(LocalEntry::Reset {
                    phase: position.phase,
                });
                view.private_bit = match position.phase {
                    Phase::Calibration => None,
                    Phase::Live { trial } => {
                        let (x, y) = bits.0[usize::from(trial)];
                        let bit = if view.prefix.role == Role::A { x } else { y };
                        view.prefix
                            .entries
                            .push(LocalEntry::PrivateBit { trial, bit });
                        Some(bit)
                    }
                };
            }
        }
        for view in &mut views {
            view.prefix.checkpoint = Checkpoint::Action(position);
            view.belief = ensemble.infer(&view.prefix)?;
        }
        result.push(views.clone());
        let view = &mut views[role.index()];
        let action = decide(view, protocol.pair.roles[role.index()])?.action;
        let target = match position.phase {
            Phase::Calibration => None,
            Phase::Live { trial } => {
                let (x, y) = bits.0[usize::from(trial)];
                Some(if role == Role::A { y } else { x })
            }
        };
        let event = world.apply(role, position, &action, target, view.credits)?;
        view.credits = event.credits_after;
        view.prefix.entries.push(LocalEntry::Action(event));
        for view in &mut views {
            view.prefix.checkpoint = Checkpoint::AfterSlot(position);
            view.belief = ensemble.infer(&view.prefix)?;
        }
        result.push(views.clone());
        if position.slot == 4 {
            world.finish_round();
        }
        if let Phase::Live { trial } = position.phase {
            if position.round == 3 && position.slot == 4 {
                for view in &mut views {
                    view.prefix.checkpoint = Checkpoint::Prediction { trial };
                    view.belief = ensemble.infer(&view.prefix)?;
                }
                result.push(views.clone());
            }
        }
    }
    Ok(result)
}

#[test]
fn calibration_at_every_slot_matches_independent_incremental_filter_and_fixture() {
    let protocol = protocol(3, [Knowledge::Unknown; 2]);
    let ensemble = Ensemble::build(&protocol).unwrap();
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/inference-reference.json")).unwrap();
    // Hand-derived per-role read outcomes in chronological order; independent of controller/inference.
    let a = [
        [Symbol::Ack0, Symbol::Ack0, Symbol::Probe1],
        [Symbol::Ack0, Symbol::Blank, Symbol::Probe1],
        [Symbol::Probe0, Symbol::Probe0, Symbol::Probe0],
        [Symbol::Blank, Symbol::Blank, Symbol::Blank],
    ];
    let b = [
        [Symbol::Probe0, Symbol::Ack0, Symbol::Ack1],
        [Symbol::Probe0, Symbol::Blank, Symbol::Ack1],
        [Symbol::Blank, Symbol::Blank, Symbol::Probe1],
        [Symbol::Blank, Symbol::Blank, Symbol::Blank],
    ];
    for model in Mechanism::ALL {
        let trace = rollout(&protocol, &ensemble, model, 0).unwrap();
        let mut survivors = [[true; 4]; 2];
        let mut reads = [0; 2];
        for views in trace.iter().skip(1) {
            let Checkpoint::AfterSlot(position) = views[0].prefix.checkpoint else {
                continue;
            };
            if position.phase != Phase::Calibration {
                continue;
            }
            let role = role_at(position).unwrap();
            let i = role.index();
            if let Some(LocalEntry::Action(Event {
                outcome: Outcome::Read(symbol),
                ..
            })) = views[i].prefix.entries.last()
            {
                let catalog = if i == 0 { a } else { b };
                for (m, supported) in survivors[i].iter_mut().enumerate() {
                    *supported &= catalog[m][reads[i]] == *symbol;
                }
                reads[i] += 1;
            }
            for i in 0..2 {
                let total = survivors[i].iter().filter(|v| **v).count() as u64;
                assert_eq!(
                    views[i].belief.models,
                    survivors[i].map(|v| p(u64::from(v), total))
                );
                assert_eq!(ensemble.infer(&views[i].prefix).unwrap(), views[i].belief);
            }
            if position.slot == 4 {
                let name = ["SP:3", "SR:3", "PP:3", "IN:3"][model.index()];
                for i in 0..2 {
                    let expected: [Probability; 4] = serde_json::from_value(
                        fixture["calibration_posteriors_by_round"][name]
                            [usize::from(position.round - 1)][i][0]
                            .clone(),
                    )
                    .unwrap();
                    assert_eq!(views[i].belief.models, expected);
                }
            }
        }
    }
}

#[test]
fn unsupported_read_preserves_charged_prefix_and_stale_prior() {
    let mut protocol = protocol(0, [Knowledge::Unknown; 2]);
    protocol.prior_mode = PriorMode::Stale(Mechanism::SharedPersistent);
    let ensemble = Ensemble::build(&protocol).unwrap();
    assert_eq!(ensemble.candidate_count(), 1024);
    let trace = rollout(&protocol, &ensemble, Mechanism::SharedPersistent, 0).unwrap();
    let mut prefix = trace
        .iter()
        .find(|v| {
            matches!(
                v[1].prefix.checkpoint,
                Checkpoint::AfterSlot(Position {
                    phase: Phase::Live { trial: 0 },
                    round: 2,
                    slot: 2
                })
            )
        })
        .unwrap()[1]
        .prefix
        .clone();
    if let Some(LocalEntry::Action(event)) = prefix.entries.last_mut() {
        event.outcome = Outcome::Read(Symbol::Blank);
        assert_eq!(event.credits_after, 45);
    }
    assert_eq!(
        ensemble.infer(&prefix),
        Err(Error::UnsupportedHistory {
            prefix: prefix.clone()
        })
    );
    prefix.own_prior = OwnPrior::Uniform;
    assert!(matches!(
        ensemble.infer(&prefix),
        Err(Error::InvalidProtocol(_))
    ));
}

#[test]
fn unknown_point_mass_wrong_ids_and_future_bits_are_operational_errors() {
    let protocol = protocol(0, [Knowledge::Unknown; 2]);
    let ensemble = Ensemble::build(&protocol).unwrap();
    let mut prefix = LocalPrefix {
        role: Role::A,
        ids: protocol.ids,
        checkpoint: Checkpoint::EpisodeStart,
        own_prior: OwnPrior::PointMass(Mechanism::SharedPersistent),
        entries: vec![],
    };
    assert!(matches!(
        ensemble.infer(&prefix),
        Err(Error::InvalidProtocol(_))
    ));
    prefix.own_prior = OwnPrior::Uniform;
    prefix.ids.surface = "another".into();
    assert!(matches!(
        ensemble.infer(&prefix),
        Err(Error::InvalidProtocol(_))
    ));
    prefix.ids.surface = "field".into();
    prefix.entries.push(LocalEntry::PrivateBit {
        trial: 3,
        bit: true,
    });
    assert!(matches!(
        ensemble.infer(&prefix),
        Err(Error::InvalidProtocol(_))
    ));
}

#[test]
fn exhaustive_local_prefix_invariance_mixed_knowledge_and_no_double_conditioning() {
    for roles in [
        [Knowledge::Known, Knowledge::Unknown],
        [Knowledge::Unknown, Knowledge::Known],
    ] {
        let protocol = protocol(0, roles);
        let ensemble = Ensemble::build(&protocol).unwrap();
        let mut observed: BTreeMap<LocalPrefix, (Belief, Option<Action>)> = BTreeMap::new();
        for model in Mechanism::ALL {
            for sequence in 0..256 {
                for views in rollout(&protocol, &ensemble, model, sequence).unwrap() {
                    for view in views {
                        let action = match view.prefix.checkpoint {
                            Checkpoint::Action(pos)
                                if role_at(pos).unwrap() == view.prefix.role =>
                            {
                                Some(
                                    decide(&view, roles[view.prefix.role.index()])
                                        .unwrap()
                                        .action,
                                )
                            }
                            _ => None,
                        };
                        let item = (view.belief.clone(), action);
                        if let Some(previous) = observed.get(&view.prefix) {
                            assert_eq!(previous, &item);
                        } else {
                            observed.insert(view.prefix.clone(), item);
                        }
                        assert_eq!(ensemble.infer(&view.prefix).unwrap(), view.belief);
                        if view.prefix.checkpoint == Checkpoint::EpisodeStart
                            && roles[view.prefix.role.index()] == Knowledge::Unknown
                        {
                            assert_eq!(view.belief.models, [p(1, 4); 4]);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn retained_own_data_does_not_reveal_hidden_target_in_mixed_pair() {
    let protocol = protocol(0, [Knowledge::Known, Knowledge::Unknown]);
    let ensemble = Ensemble::build(&protocol).unwrap();
    let traces = [0, 2]
        .map(|bits| rollout(&protocol, &ensemble, Mechanism::SharedPersistent, bits).unwrap());
    let end = |trace: &Vec<[LocalView; 2]>| {
        trace
            .iter()
            .find(|v| v[0].prefix.checkpoint == Checkpoint::Prediction { trial: 0 })
            .unwrap()[0]
            .clone()
    };
    let left = end(&traces[0]);
    let right = end(&traces[1]);
    assert_eq!(left.prefix, right.prefix);
    assert_eq!(left.belief.target, Some(p(1, 2)));
    assert!(!predict(&left).unwrap());
}

#[test]
fn no_communication_inspects_without_model_learning_and_live_resets_clear_target_evidence() {
    let protocol = protocol(1, [Knowledge::NoCommunication; 2]);
    let ensemble = Ensemble::build(&protocol).unwrap();
    let trace = rollout(&protocol, &ensemble, Mechanism::SharedPersistent, 2).unwrap();
    for views in &trace {
        for view in views {
            assert_eq!(view.belief.models, [p(1, 4); 4]);
            if matches!(
                view.prefix.checkpoint,
                Checkpoint::Action(Position {
                    phase: Phase::Live { trial: 1 },
                    round: 1,
                    slot: 1
                })
            ) {
                assert_eq!(view.belief.target, Some(p(1, 2)));
            }
        }
    }
    let last = trace.last().unwrap();
    assert_eq!(last[0].credits, 10);
    assert_eq!(last[1].credits, 10);
    for views in trace
        .iter()
        .filter(|v| matches!(v[0].prefix.checkpoint, Checkpoint::Prediction { trial: 0 }))
    {
        assert!(predict(&views[0]).unwrap());
        assert!(!predict(&views[1]).unwrap());
    }
}

#[test]
fn a_reads_only_after_its_actual_write_not_after_inspection() {
    let mut view = fixture_live_a_view([1, 0, 0, 0], 1, Some(false));
    view.prefix.checkpoint = Checkpoint::Action(Position {
        phase: Phase::Live { trial: 0 },
        round: 3,
        slot: 1,
    });
    view.prefix.entries.push(LocalEntry::Action(Event {
        position: Position {
            phase: Phase::Live { trial: 0 },
            round: 1,
            slot: 1,
        },
        action: Action::InspectOwnTarget,
        outcome: Outcome::Inspected(true),
        credits_after: 44,
    }));
    view.credits = 44;
    assert_eq!(
        decide(&view, Knowledge::Unknown).unwrap().action,
        Action::Wait
    );
    if let Some(LocalEntry::Action(event)) = view.prefix.entries.last_mut() {
        event.action = Action::Write(Symbol::Data0);
        event.outcome = Outcome::Accepted;
        event.credits_after = 47;
    }
    view.credits = 47;
    assert_eq!(
        decide(&view, Knowledge::Unknown).unwrap().action,
        Action::Read
    );
}

#[test]
fn an_impossible_uniform_prior_read_is_unsupported_but_bad_cost_is_operational() {
    let protocol = protocol(1, [Knowledge::Unknown; 2]);
    let ensemble = Ensemble::build(&protocol).unwrap();
    let trace = rollout(&protocol, &ensemble, Mechanism::SharedPersistent, 0).unwrap();
    let mut prefix = trace
        .iter()
        .find(|v| {
            v[1].prefix.checkpoint
                == Checkpoint::AfterSlot(Position {
                    phase: Phase::Calibration,
                    round: 1,
                    slot: 2,
                })
        })
        .unwrap()[1]
        .prefix
        .clone();
    if let Some(LocalEntry::Action(event)) = prefix.entries.last_mut() {
        event.outcome = Outcome::Read(Symbol::Data1);
    }
    assert_eq!(
        ensemble.infer(&prefix),
        Err(Error::UnsupportedHistory {
            prefix: prefix.clone()
        })
    );
    if let Some(LocalEntry::Action(event)) = prefix.entries.last_mut() {
        event.credits_after = 48;
    }
    assert!(matches!(
        ensemble.infer(&prefix),
        Err(Error::InvalidProtocol(_))
    ));
}

#[test]
fn b_waits_at_half_and_prediction_without_target_evidence_is_fair() {
    let mut view = fixture_live_a_view([1, 1, 0, 0], 2, Some(true));
    view.prefix.role = Role::B;
    view.prefix.checkpoint = Checkpoint::Action(Position {
        phase: Phase::Live { trial: 0 },
        round: 2,
        slot: 2,
    });
    assert_eq!(
        decide(&view, Knowledge::Unknown).unwrap().action,
        Action::Wait
    );
    view.prefix.checkpoint = Checkpoint::Prediction { trial: 0 };
    view.belief.target = None;
    assert!(!predict(&view).unwrap());
}

#[test]
fn post_slot_wire_clock_validates_position_and_never_aliases_pre_slot() {
    let position = Position {
        phase: Phase::Calibration,
        round: 1,
        slot: 2,
    };
    let encoded = serde_json::to_string(&Checkpoint::AfterSlot(position)).unwrap();
    assert_eq!(
        serde_json::from_str::<Checkpoint>(&encoded).unwrap(),
        Checkpoint::AfterSlot(position)
    );
    assert!(serde_json::from_str::<Checkpoint>(
        r#"{"AfterSlot":{"phase":"Calibration","round":0,"slot":2}}"#
    )
    .is_err());
    assert_ne!(
        Checkpoint::AfterSlot(position),
        Checkpoint::Action(position)
    );
}

#[test]
fn delayed_live_data_and_inspection_posteriors_match_all_target_bits() {
    for (rounds, knowledge) in [
        (0, Knowledge::Known),
        (1, Knowledge::Unknown),
        (2, Knowledge::Unknown),
        (3, Knowledge::Unknown),
    ] {
        let protocol = protocol(rounds, [knowledge; 2]);
        let ensemble = Ensemble::build(&protocol).unwrap();
        for sequence in 0..256 {
            let bits = EpisodeBits::from_index(sequence).unwrap();
            let trace =
                rollout(&protocol, &ensemble, Mechanism::SharedPersistent, sequence).unwrap();
            for views in trace
                .iter()
                .filter(|v| matches!(v[0].prefix.checkpoint, Checkpoint::Prediction { .. }))
            {
                let Checkpoint::Prediction { trial } = views[0].prefix.checkpoint else {
                    unreachable!()
                };
                let (x, y) = bits.0[usize::from(trial)];
                assert_eq!(views[0].belief.target, Some(p(u64::from(y), 1)));
                assert_eq!(views[1].belief.target, Some(p(u64::from(x), 1)));
                assert_eq!(predict(&views[0]).unwrap(), y);
                assert_eq!(predict(&views[1]).unwrap(), x);
            }
            let spent = if rounds == 1 { 38 } else { 24 + 2 * rounds };
            assert_eq!(trace.last().unwrap()[0].credits, 48 - spent);
            assert_eq!(trace.last().unwrap()[1].credits, 48 - spent);
        }
    }
}
