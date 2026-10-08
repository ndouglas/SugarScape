use super::super::*;
use crate::shared_surface::Outcome;

fn state(role: Role, policy: PolicyKind, model: Mechanism) -> EpisodeState {
    let p = Protocol::new(role, policy);
    let mut priors = [OwnPrior::PointMass(model); 2];
    priors[role.index()] = if policy == PolicyKind::Known {
        OwnPrior::PointMass(model)
    } else {
        OwnPrior::Uniform
    };
    EpisodeState::new(
        &p,
        Environment::InFamily(model),
        EpisodeBits::from_index(0).unwrap(),
        priors,
    )
    .unwrap()
}
fn belief() -> Belief {
    Belief {
        models: [Probability::new(1, 4).unwrap(); 4],
        target: None,
    }
}
fn physical(prefix: &Prefix) -> Vec<&LocalEntry> {
    prefix
        .entries
        .iter()
        .filter_map(|e| match e {
            Entry::Physical(e) => Some(e),
            _ => None,
        })
        .collect()
}

// Catches omitting either Agent's paid calibration opportunities.
#[test]
fn one_probe_round_charges_both_agents_two_credits() {
    for role in [Role::A, Role::B] {
        let mut s = state(role, PolicyKind::Adaptive, Mechanism::SharedPersistent);
        let step = apply_choice(&mut s, Choice::ContinueProbe).unwrap();
        assert_eq!(step.spent_delta, [2, 2]);
        assert_eq!(step.next, Some(Boundary::ProbeChoice { completed: 1 }));
    }
}

// Catches virtual calibration resets, waits, target inspection, and early bits.
#[test]
fn stop_at_zero_has_only_live_reset_and_issued_own_bit() {
    let mut s = state(Role::A, PolicyKind::Adaptive, Mechanism::Inert);
    assert!(physical(s.prefix(Role::A)).is_empty());
    assert_eq!(
        View::from_prefix(s.prefix(Role::A).clone(), belief())
            .unwrap()
            .private_bit,
        None
    );
    let step = apply_choice(&mut s, Choice::StopProbing).unwrap();
    assert_eq!(step.spent_delta, [0, 0]);
    assert_eq!(step.next, Some(Boundary::TrialChoice { trial: 0 }));
    for role in [Role::A, Role::B] {
        assert_eq!(
            physical(s.prefix(role)),
            vec![
                &LocalEntry::Reset {
                    phase: Phase::Live { trial: 0 }
                },
                &LocalEntry::PrivateBit {
                    trial: 0,
                    bit: false
                }
            ]
        );
        assert!(s
            .prefix(role)
            .entries
            .contains(&Entry::PublicProbeStop { completed: 0 }));
    }
    assert_eq!(
        s.prefix(Role::B)
            .entries
            .iter()
            .filter(|e| matches!(e, Entry::OwnChoice { .. }))
            .count(),
        0
    );
}

// Catches swapping labels or revealing responder A's first action to B.
#[test]
fn b_first_trial_boundary_follows_unobserved_a_slot() {
    let mut s = state(Role::B, PolicyKind::Adaptive, Mechanism::SharedPersistent);
    let step = apply_choice(&mut s, Choice::StopProbing).unwrap();
    assert_eq!(step.spent_delta, [1, 0]);
    assert_eq!(step.events.len(), 1);
    assert_eq!(step.events[0].0, Role::A);
    assert_eq!(
        step.events[0].1.position,
        Position {
            phase: Phase::Live { trial: 0 },
            round: 1,
            slot: 1
        }
    );
    assert_eq!(step.events[0].1.action, Action::Write(Symbol::Data0));
    assert!(!physical(s.prefix(Role::B))
        .iter()
        .any(|e| matches!(e, LocalEntry::Action(_))));
    assert_eq!(
        s.prefix(Role::B).checkpoint,
        Checkpoint::Boundary(Boundary::TrialChoice { trial: 0 })
    );
}

// Catches skipped costs and permitting a fourth probe.
#[test]
fn every_stop_path_preserves_actual_probe_count_and_cost() {
    for role in [Role::A, Role::B] {
        for completed in 0..=3 {
            let mut s = state(role, PolicyKind::Adaptive, Mechanism::Inert);
            for _ in 0..completed {
                apply_choice(&mut s, Choice::ContinueProbe).unwrap();
            }
            if completed == 3 {
                let before = s.prefix(role).clone();
                assert!(select_choice(&mut s, Choice::ContinueProbe).is_err());
                assert_eq!(s.prefix(role), &before);
            }
            let step = apply_choice(&mut s, Choice::StopProbing).unwrap();
            assert_eq!(step.next, Some(Boundary::TrialChoice { trial: 0 }));
            assert_eq!(s.credits(role), 48 - 2 * completed);
            assert!(s
                .prefix(role)
                .entries
                .contains(&Entry::PublicProbeStop { completed }));
            assert_eq!(physical(s.prefix(role)).iter().filter(|e| matches!(e, LocalEntry::Action(event) if event.position.phase == Phase::Calibration)).count(), usize::from(2 * completed));
        }
    }
}

// Catches charging inspect once per slot or omitting the scheduled waits.
#[test]
fn full_trials_charge_inspect_nine_and_attempt_six_for_both_roles() {
    for role in [Role::A, Role::B] {
        for (choice, cost) in [(Choice::Inspect, 9), (Choice::AttemptCommunication, 6)] {
            let mut s = state(role, PolicyKind::Adaptive, Mechanism::SharedPersistent);
            apply_choice(&mut s, Choice::StopProbing).unwrap();
            for trial in 0..4 {
                let before = s.credits(role);
                let step = apply_choice(&mut s, choice).unwrap();
                assert_eq!(before - s.credits(role), cost);
                assert_eq!(
                    step.next,
                    if trial == 3 {
                        None
                    } else {
                        Some(Boundary::TrialChoice { trial: trial + 1 })
                    }
                );
                assert_eq!(s.prefix(role).entries.iter().filter(|e| matches!(e, Entry::OwnChoice { boundary: Boundary::TrialChoice { trial: t }, .. } if *t == trial)).count(), 1);
            }
            assert_eq!(s.prefix(role).checkpoint, Checkpoint::Finished);
            assert!(select_choice(&mut s, Choice::Inspect).is_err());
            assert!(advance_one(&mut s).is_err());
        }
    }
}

// Catches resetting or disclosing the next bit in the same step as a paid slot.
#[test]
fn paid_steps_return_after_slot_before_next_trial_bit_or_reset() {
    let mut s = state(Role::A, PolicyKind::Adaptive, Mechanism::SharedResetting);
    apply_choice(&mut s, Choice::StopProbing).unwrap();
    select_choice(&mut s, Choice::AttemptCommunication).unwrap();
    assert_eq!(s.credits(Role::A), 48);
    let first = advance_one(&mut s).unwrap();
    assert_eq!(first.events.len(), 1);
    assert_eq!(first.spent_delta, [1, 0]);
    assert_eq!(
        s.prefix(Role::A).checkpoint,
        Checkpoint::AfterSlot(Position {
            phase: Phase::Live { trial: 0 },
            round: 1,
            slot: 1
        })
    );
    // A host may halt here; no other paid event was executed.
    assert_eq!(s.credits(Role::B), 48);
    loop {
        let step = advance_one(&mut s).unwrap();
        if let Some((_, event)) = step.events.first() {
            assert_eq!(step.events.len(), 1);
            assert_eq!(
                s.prefix(Role::A).checkpoint,
                Checkpoint::AfterSlot(event.position)
            );
            if event.position.round == 3 && event.position.slot == 4 {
                break;
            }
        }
    }
    for role in [Role::A, Role::B] {
        assert!(!physical(s.prefix(role)).iter().any(|e| matches!(
            e,
            LocalEntry::PrivateBit { trial: 1, .. }
                | LocalEntry::Reset {
                    phase: Phase::Live { trial: 1 }
                }
        )));
    }
    advance_one(&mut s).unwrap();
    assert_eq!(
        s.prefix(Role::A).checkpoint,
        Checkpoint::Prediction { trial: 0 }
    );
    advance_one(&mut s).unwrap();
    assert!(
        physical(s.prefix(Role::A)).contains(&&LocalEntry::PrivateBit {
            trial: 1,
            bit: false
        })
    );
}

// Catches a second routine choice replacing the committed choice mid-trial.
#[test]
fn wrong_phase_or_repeated_choices_leave_histories_and_credits_unchanged() {
    let mut s = state(Role::A, PolicyKind::Adaptive, Mechanism::Inert);
    let before = s.prefix(Role::A).clone();
    assert!(select_choice(&mut s, Choice::Inspect).is_err());
    assert_eq!(s.prefix(Role::A), &before);
    apply_choice(&mut s, Choice::StopProbing).unwrap();
    let before = s.prefix(Role::A).clone();
    assert!(select_choice(&mut s, Choice::ContinueProbe).is_err());
    assert_eq!(s.prefix(Role::A), &before);
    select_choice(&mut s, Choice::Inspect).unwrap();
    let before = s.prefix(Role::A).clone();
    assert!(select_choice(&mut s, Choice::AttemptCommunication).is_err());
    assert_eq!(s.prefix(Role::A), &before);
    assert_eq!(s.credits(Role::A), 48);
}

// Catches a missing credit bound over all supplied policies and mechanisms.
#[test]
fn complete_legal_paths_spend_at_most_forty_two() {
    for role in [Role::A, Role::B] {
        for model in Mechanism::ALL {
            for rounds in 0..=3 {
                for routine in [Choice::Inspect, Choice::AttemptCommunication] {
                    let mut s = state(role, PolicyKind::Adaptive, model);
                    for _ in 0..rounds {
                        apply_choice(&mut s, Choice::ContinueProbe).unwrap();
                    }
                    apply_choice(&mut s, Choice::StopProbing).unwrap();
                    for _ in 0..4 {
                        apply_choice(&mut s, routine).unwrap();
                    }
                    assert!(s.credits(Role::A) >= 6 && s.credits(Role::B) >= 6);
                    for r in [Role::A, Role::B] {
                        View::from_prefix(s.prefix(r).clone(), belief()).unwrap();
                    }
                }
            }
        }
    }
}

// Catches leaking peer bits or deriving the current bit from latent host state.
#[test]
fn views_extract_only_their_issued_own_bit_and_local_credits() {
    for role in [Role::A, Role::B] {
        let p = Protocol::new(role, PolicyKind::Adaptive);
        let mut priors = [OwnPrior::PointMass(Mechanism::SharedPersistent); 2];
        priors[role.index()] = OwnPrior::Uniform;
        let mut s = EpisodeState::new(
            &p,
            Environment::InFamily(Mechanism::SharedPersistent),
            EpisodeBits::from_index(1).unwrap(),
            priors,
        )
        .unwrap();
        apply_choice(&mut s, Choice::StopProbing).unwrap();
        for r in [Role::A, Role::B] {
            let view = View::from_prefix(s.prefix(r).clone(), belief()).unwrap();
            assert_eq!(view.private_bit, Some(r == Role::A));
            assert_eq!(
                view.credits,
                if role == Role::B && r == Role::A {
                    47
                } else {
                    48
                }
            );
            let json = serde_json::to_value(&view).unwrap();
            assert!(json.get("environment").is_none());
        }
    }
}

// Catches silently obtaining knowledge from a policy label or actual mechanism.
#[test]
fn protocol_prior_mismatches_are_rejected() {
    let model = Mechanism::Inert;
    let p = Protocol::new(Role::A, PolicyKind::Adaptive);
    assert!(EpisodeState::new(
        &p,
        Environment::InFamily(model),
        EpisodeBits::from_index(0).unwrap(),
        [OwnPrior::PointMass(model); 2]
    )
    .is_err());
    assert!(EpisodeState::new(
        &p,
        Environment::InFamily(model),
        EpisodeBits::from_index(0).unwrap(),
        [OwnPrior::Uniform; 2]
    )
    .is_err());
    let p = Protocol::new(Role::A, PolicyKind::Known);
    assert!(EpisodeState::new(
        &p,
        Environment::InFamily(model),
        EpisodeBits::from_index(0).unwrap(),
        [OwnPrior::Uniform, OwnPrior::PointMass(model)]
    )
    .is_err());
}

// Catches acknowledging private receipt or mechanism through generic acceptance.
#[test]
fn write_acceptance_is_generic_across_all_mechanisms() {
    for model in Mechanism::ALL {
        let mut s = state(Role::A, PolicyKind::Adaptive, model);
        let step = apply_choice(&mut s, Choice::ContinueProbe).unwrap();
        assert_eq!(step.events[0].1.outcome, Outcome::Accepted);
    }
}

// Catches malformed bounded clocks, choices, identifiers, future bits, and budgets.
#[test]
fn checked_wire_rejects_malformed_local_histories_and_views() {
    for raw in [
        r#"{"ProbeChoice":{"completed":4}}"#,
        r#"{"TrialChoice":{"trial":4}}"#,
    ] {
        assert!(serde_json::from_str::<Boundary>(raw).is_err());
    }
    assert!(serde_json::from_str::<Checkpoint>(r#"{"Prediction":{"trial":4}}"#).is_err());
    assert!(serde_json::from_str::<Entry>(
        r#"{"OwnChoice":{"boundary":{"ProbeChoice":{"completed":0}},"choice":"Inspect"}}"#
    )
    .is_err());
    let mut s = state(Role::A, PolicyKind::Adaptive, Mechanism::SharedPersistent);
    apply_choice(&mut s, Choice::StopProbing).unwrap();
    let prefix = s.prefix(Role::A).clone();
    let view = View::from_prefix(prefix.clone(), belief()).unwrap();
    assert_eq!(
        serde_json::from_value::<View>(serde_json::to_value(&view).unwrap()).unwrap(),
        view
    );
    let mut wire = serde_json::to_value(&view).unwrap();
    wire["private_bit"] = serde_json::json!(true);
    assert!(serde_json::from_value::<View>(wire).is_err());
    let mut wire = serde_json::to_value(&view).unwrap();
    wire["credits"] = serde_json::json!(47);
    assert!(serde_json::from_value::<View>(wire).is_err());
    let mut wire = serde_json::to_value(&prefix).unwrap();
    wire["ids"]["agents"][1] = serde_json::json!("Agent-A");
    assert!(serde_json::from_value::<Prefix>(wire).is_err());
    let mut bad = prefix.clone();
    bad.entries.push(Entry::Physical(LocalEntry::PrivateBit {
        trial: 1,
        bit: true,
    }));
    assert!(View::from_prefix(bad, belief()).is_err());
    let mut bad = prefix.clone();
    bad.checkpoint = Checkpoint::Prediction { trial: 0 };
    assert!(View::from_prefix(bad, belief()).is_err());
    let mut bad = prefix;
    bad.entries.push(Entry::Physical(LocalEntry::Action(Event {
        position: Position {
            phase: Phase::Live { trial: 0 },
            round: 1,
            slot: 2,
        },
        action: Action::Wait,
        outcome: Outcome::Waited,
        credits_after: 47,
    })));
    assert!(View::from_prefix(bad, belief()).is_err());
    let mut bad = belief();
    bad.models[0] = Probability {
        numerator: 1,
        denominator: 0,
    };
    assert!(View::from_prefix(s.prefix(Role::A).clone(), bad).is_err());
}

// Catches a free Stop choice disclosing bits or consuming a paid opportunity.
#[test]
fn selecting_stop_records_public_boundary_without_bits_or_cost() {
    let mut s = state(Role::B, PolicyKind::Adaptive, Mechanism::Inert);
    select_choice(&mut s, Choice::StopProbing).unwrap();
    for role in [Role::A, Role::B] {
        assert_eq!(s.credits(role), 48);
        assert!(physical(s.prefix(role)).is_empty());
        assert!(s
            .prefix(role)
            .entries
            .contains(&Entry::PublicProbeStop { completed: 0 }));
        View::from_prefix(s.prefix(role).clone(), belief()).unwrap();
    }
}

// Catches parser rejection of legal intermediate checkpoints or skipped paid slots.
#[test]
fn every_one_step_snapshot_is_a_valid_local_view() {
    for experimenter in [Role::A, Role::B] {
        let protocol = Protocol::new(experimenter, PolicyKind::Adaptive);
        let mut s = state(
            experimenter,
            PolicyKind::Adaptive,
            Mechanism::SharedPersistent,
        );
        let mut count = 0;
        loop {
            match s.prefix(experimenter).checkpoint {
                Checkpoint::Boundary(Boundary::ProbeChoice { completed }) => {
                    select_choice(
                        &mut s,
                        if completed < 3 {
                            Choice::ContinueProbe
                        } else {
                            Choice::StopProbing
                        },
                    )
                    .unwrap();
                }
                Checkpoint::Boundary(Boundary::TrialChoice { .. }) => {
                    // Only newly exposed boundaries accept a choice; selection changes no paid state.
                    if !matches!(
                        s.prefix(experimenter).entries.last(),
                        Some(Entry::OwnChoice {
                            boundary: Boundary::TrialChoice { .. },
                            ..
                        })
                    ) {
                        select_choice(&mut s, Choice::Inspect).unwrap();
                    }
                }
                Checkpoint::Finished => break,
                _ => {}
            }
            for role in [Role::A, Role::B] {
                protocol.validate_prefix(s.prefix(role)).unwrap();
            }
            let step = advance_one(&mut s).unwrap();
            count += step.events.len();
            for role in [Role::A, Role::B] {
                View::from_prefix(s.prefix(role).clone(), belief()).unwrap();
                protocol.validate_prefix(s.prefix(role)).unwrap();
            }
        }
        assert_eq!(count, 60);
    }
}

// Catches requiring Continue only after the first paid outcome of a round.
#[test]
fn entered_probe_round_requires_own_continue_before_first_paid_slot() {
    for experimenter in [Role::A, Role::B] {
        let protocol = Protocol::new(experimenter, PolicyKind::Adaptive);
        for completed in 0..=2 {
            let mut s = state(experimenter, PolicyKind::Adaptive, Mechanism::Inert);
            for _ in 0..completed {
                apply_choice(&mut s, Choice::ContinueProbe).unwrap();
            }
            select_choice(&mut s, Choice::ContinueProbe).unwrap();
            advance_one(&mut s).unwrap();
            let mut prefix = s.prefix(experimenter).clone();
            prefix.entries.retain(|entry| {
                !matches!(entry, Entry::OwnChoice { boundary: Boundary::ProbeChoice { completed: chosen }, .. } if *chosen == completed)
            });
            assert!(protocol.validate_prefix(&prefix).is_err());
        }
    }
}

// Catches accepting the private half of an atomically emitted public Stop.
#[test]
fn queued_own_stop_requires_matching_public_stop() {
    for experimenter in [Role::A, Role::B] {
        let protocol = Protocol::new(experimenter, PolicyKind::Adaptive);
        let mut s = state(experimenter, PolicyKind::Adaptive, Mechanism::Inert);
        select_choice(&mut s, Choice::StopProbing).unwrap();
        let mut prefix = s.prefix(experimenter).clone();
        prefix
            .entries
            .retain(|entry| !matches!(entry, Entry::PublicProbeStop { .. }));
        assert!(protocol.validate_prefix(&prefix).is_err());
    }
}

// Catches giving a responder the experimenter's private live decision checkpoint.
#[test]
fn responder_cannot_have_a_trial_choice_checkpoint() {
    for experimenter in [Role::A, Role::B] {
        let protocol = Protocol::new(experimenter, PolicyKind::Adaptive);
        let mut s = state(
            experimenter,
            PolicyKind::Adaptive,
            Mechanism::SharedPersistent,
        );
        apply_choice(&mut s, Choice::StopProbing).unwrap();
        let responder = if experimenter == Role::A {
            Role::B
        } else {
            Role::A
        };
        let mut prefix = s.prefix(responder).clone();
        prefix.checkpoint = Checkpoint::Boundary(Boundary::TrialChoice { trial: 0 });
        assert!(protocol.validate_prefix(&prefix).is_err());
    }
}

// Catches accepting a forged experimenter history with its interventions removed.
#[test]
fn declared_experimenter_history_cannot_drop_own_choices() {
    let protocol = Protocol::new(Role::A, PolicyKind::Adaptive);
    let mut s = state(Role::A, PolicyKind::Adaptive, Mechanism::Inert);
    apply_choice(&mut s, Choice::StopProbing).unwrap();
    let mut prefix = s.prefix(Role::A).clone();
    prefix
        .entries
        .retain(|e| !matches!(e, Entry::OwnChoice { .. }));
    assert!(protocol.validate_prefix(&prefix).is_err());
}

// Catches disclosing a target belief before any task bit has been issued.
#[test]
fn probe_view_cannot_carry_a_target_posterior() {
    let s = state(Role::A, PolicyKind::Adaptive, Mechanism::Inert);
    let mut posterior = belief();
    posterior.target = Some(Probability::new(1, 2).unwrap());
    assert!(View::from_prefix(s.prefix(Role::A).clone(), posterior).is_err());
}
