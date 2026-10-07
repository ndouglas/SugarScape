use super::super::*;
use std::cmp::Ordering;

fn ids() -> Ids {
    Ids {
        surface: "s".into(),
        agents: ["a".into(), "b".into()],
    }
}

fn calibration(round: u8, slot: u8) -> Position {
    Position {
        phase: Phase::Calibration,
        round,
        slot,
    }
}

fn live(trial: u8, round: u8, slot: u8) -> Position {
    Position {
        phase: Phase::Live { trial },
        round,
        slot,
    }
}

fn protocol(rounds: u8) -> Protocol {
    Protocol {
        calibration_rounds: rounds,
        pair: Pair {
            roles: [Knowledge::Unknown; 2],
        },
        prior_mode: PriorMode::Treatment,
        ids: ids(),
    }
}

#[test]
fn inert_acceptance_is_not_delivery() {
    let mut world = World::new(Environment::InFamily(Mechanism::Inert), ids());
    let event = world
        .apply(
            Role::A,
            calibration(1, 1),
            &Action::Write(Symbol::Probe0),
            None,
            48,
        )
        .unwrap();
    assert_eq!(event.outcome, Outcome::Accepted);
    assert_eq!(
        world
            .apply(Role::B, calibration(1, 2), &Action::Read, None, 48)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Blank)
    );
    assert!(world.read_lineage(Role::B).is_none());
}

#[test]
fn resetting_matches_same_round_visibility_but_loses_delayed_message() {
    for (mechanism, delayed) in [
        (Mechanism::SharedPersistent, Symbol::Probe0),
        (Mechanism::SharedResetting, Symbol::Blank),
    ] {
        let mut world = World::new(Environment::InFamily(mechanism), ids());
        world
            .apply(
                Role::A,
                calibration(1, 1),
                &Action::Write(Symbol::Probe0),
                None,
                48,
            )
            .unwrap();
        assert_eq!(
            world
                .apply(Role::B, calibration(1, 2), &Action::Read, None, 48)
                .unwrap()
                .outcome,
            Outcome::Read(Symbol::Probe0)
        );
        world.finish_round();
        assert_eq!(
            world
                .apply(Role::B, calibration(2, 2), &Action::Read, None, 47)
                .unwrap()
                .outcome,
            Outcome::Read(delayed)
        );
    }
}

#[test]
fn private_field_retains_only_writer_visible_symbol() {
    let mut world = World::new(Environment::InFamily(Mechanism::PrivatePersistent), ids());
    world
        .apply(
            Role::A,
            calibration(1, 1),
            &Action::Write(Symbol::Probe0),
            None,
            48,
        )
        .unwrap();
    assert_eq!(
        world
            .apply(Role::B, calibration(1, 2), &Action::Read, None, 48)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Blank)
    );
    world.finish_round();
    assert_eq!(
        world
            .apply(Role::A, calibration(2, 1), &Action::Read, None, 47)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Probe0)
    );
}

#[test]
fn overwrite_replaces_lineage_and_keeps_raw_symbols() {
    let mut world = World::new(Environment::InFamily(Mechanism::SharedPersistent), ids());
    world
        .apply(
            Role::A,
            calibration(1, 1),
            &Action::Write(Symbol::Probe0),
            None,
            48,
        )
        .unwrap();
    world
        .apply(
            Role::B,
            calibration(1, 3),
            &Action::Write(Symbol::Ack0),
            None,
            48,
        )
        .unwrap();
    assert_eq!(
        world.read_lineage(Role::A),
        Some(Lineage {
            writer: Role::B,
            write_position: calibration(1, 3),
            symbol: Symbol::Ack0,
            task_trial: None,
        })
    );
    assert_eq!(
        world
            .apply(Role::A, calibration(1, 4), &Action::Read, None, 47)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Ack0)
    );
}

#[test]
fn data_flip_preserves_calibration_and_raw_write_origin() {
    let mut world = World::new(Environment::DataFlip, ids());
    world
        .apply(
            Role::A,
            calibration(1, 1),
            &Action::Write(Symbol::Probe0),
            None,
            48,
        )
        .unwrap();
    assert_eq!(
        world
            .apply(Role::B, calibration(1, 2), &Action::Read, None, 48)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Probe0)
    );
    world.reset(Phase::Live { trial: 0 });
    world
        .apply(
            Role::A,
            live(0, 1, 1),
            &Action::Write(Symbol::Data0),
            None,
            47,
        )
        .unwrap();
    world.finish_round();
    assert_eq!(
        world
            .apply(Role::B, live(0, 2, 2), &Action::Read, None, 47)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Data1)
    );
    assert_eq!(world.read_lineage(Role::B).unwrap().symbol, Symbol::Data0);
    assert_eq!(world.read_lineage(Role::B).unwrap().task_trial, Some(0));
}

#[test]
fn blank_write_rejected_before_field_or_lineage_changes() {
    let mut world = World::new(Environment::InFamily(Mechanism::SharedPersistent), ids());
    world
        .apply(
            Role::A,
            calibration(1, 1),
            &Action::Write(Symbol::Probe0),
            None,
            48,
        )
        .unwrap();
    let before = world.read_lineage(Role::B);
    assert!(matches!(
        world.apply(
            Role::B,
            calibration(1, 3),
            &Action::Write(Symbol::Blank),
            None,
            48
        ),
        Err(Error::InvalidAction(_))
    ));
    assert_eq!(world.read_lineage(Role::B), before);
    assert_eq!(
        world
            .apply(Role::B, calibration(1, 2), &Action::Read, None, 48)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Probe0)
    );
}

#[test]
fn calibration_inspection_rejected_even_with_target() {
    let mut world = World::new(Environment::InFamily(Mechanism::SharedPersistent), ids());
    assert!(matches!(
        world.apply(
            Role::A,
            calibration(1, 1),
            &Action::InspectOwnTarget,
            Some(true),
            48
        ),
        Err(Error::InvalidAction(_))
    ));
}

#[test]
fn inspection_requires_truth_and_four_credits() {
    let mut world = World::new(Environment::InFamily(Mechanism::SharedPersistent), ids());
    assert!(world
        .apply(Role::A, live(0, 1, 1), &Action::InspectOwnTarget, None, 48)
        .is_err());
    assert!(world
        .apply(
            Role::A,
            live(0, 1, 1),
            &Action::InspectOwnTarget,
            Some(true),
            3
        )
        .is_err());
    assert_eq!(
        world
            .apply(
                Role::A,
                live(0, 1, 1),
                &Action::InspectOwnTarget,
                Some(true),
                4
            )
            .unwrap(),
        Event {
            position: live(0, 1, 1),
            action: Action::InspectOwnTarget,
            outcome: Outcome::Inspected(true),
            credits_after: 0,
        }
    );
}

#[test]
fn depleted_write_budget_does_not_mutate_surface() {
    let mut world = World::new(Environment::InFamily(Mechanism::SharedPersistent), ids());
    assert!(world
        .apply(
            Role::A,
            calibration(1, 1),
            &Action::Write(Symbol::Probe0),
            None,
            0
        )
        .is_err());
    assert_eq!(
        world
            .apply(Role::B, calibration(1, 2), &Action::Read, None, 48)
            .unwrap()
            .outcome,
        Outcome::Read(Symbol::Blank)
    );
}

#[test]
fn read_write_wait_each_cost_one_credit() {
    for action in [Action::Read, Action::Write(Symbol::Probe0), Action::Wait] {
        let mut world = World::new(Environment::InFamily(Mechanism::Inert), ids());
        assert_eq!(
            world
                .apply(Role::A, calibration(1, 1), &action, None, 1)
                .unwrap()
                .credits_after,
            0
        );
    }
}

#[test]
fn all_public_slots_enforce_owner_before_mutation() {
    let schedule = protocol(3).schedule().unwrap();
    assert_eq!(schedule.len(), 60);
    for (owner, position) in schedule {
        let mut world = World::new(Environment::InFamily(Mechanism::SharedPersistent), ids());
        let other = if owner == Role::A { Role::B } else { Role::A };
        assert!(world
            .apply(other, position, &Action::Write(Symbol::Probe0), None, 48)
            .is_err());
        assert!(world.read_lineage(owner).is_none());
        assert_eq!(
            world
                .apply(owner, position, &Action::Wait, None, 48)
                .unwrap()
                .outcome,
            Outcome::Waited
        );
    }
}

#[test]
fn public_schedule_matches_role_order_and_trial_boundaries() {
    let schedule = protocol(0).schedule().unwrap();
    assert_eq!(schedule.len(), 48);
    assert_eq!(
        &schedule[..4],
        &[
            (Role::A, live(0, 1, 1)),
            (Role::B, live(0, 1, 2)),
            (Role::B, live(0, 1, 3)),
            (Role::A, live(0, 1, 4))
        ]
    );
    assert_eq!(schedule[8], (Role::A, live(0, 3, 1)));
    assert_eq!(schedule[12], (Role::A, live(1, 1, 1)));
    assert_eq!(schedule[47], (Role::A, live(3, 3, 4)));
}

#[test]
fn explicit_reset_clears_every_field_and_lineage() {
    for mechanism in Mechanism::ALL {
        let mut world = World::new(Environment::InFamily(mechanism), ids());
        world
            .apply(
                Role::A,
                calibration(1, 1),
                &Action::Write(Symbol::Probe0),
                None,
                48,
            )
            .unwrap();
        world
            .apply(
                Role::B,
                calibration(1, 3),
                &Action::Write(Symbol::Ack0),
                None,
                48,
            )
            .unwrap();
        let remembered = [LocalEntry::Action(
            world
                .apply(Role::A, calibration(1, 4), &Action::Read, None, 47)
                .unwrap(),
        )];
        world.reset(Phase::Live { trial: 0 });
        assert_eq!(
            world
                .apply(Role::A, live(0, 1, 1), &Action::Read, None, 46)
                .unwrap()
                .outcome,
            Outcome::Read(Symbol::Blank)
        );
        assert_eq!(
            world
                .apply(Role::B, live(0, 1, 2), &Action::Read, None, 47)
                .unwrap()
                .outcome,
            Outcome::Read(Symbol::Blank)
        );
        assert!(world.read_lineage(Role::A).is_none());
        assert_eq!(remembered.len(), 1);
    }
}

#[test]
fn malformed_identifiers_rejected_at_protocol_boundary() {
    for bad_ids in [
        Ids {
            surface: "".into(),
            agents: ["a".into(), "b".into()],
        },
        Ids {
            surface: "s".into(),
            agents: ["a".into(), "a".into()],
        },
        Ids {
            surface: "s".into(),
            agents: [" ".into(), "b".into()],
        },
    ] {
        let mut protocol = protocol(1);
        protocol.ids = bad_ids;
        assert!(matches!(
            protocol.validate(),
            Err(Error::InvalidProtocol(_))
        ));
    }
}

#[test]
fn unknown_fields_and_symbols_rejected_on_wire() {
    assert!(
        serde_json::from_str::<Ids>(r#"{"surface":"s","agents":["a","b"],"mailbox":"x"}"#).is_err()
    );
    assert!(serde_json::from_str::<Symbol>(r#""recipient_b""#).is_err());
    assert!(serde_json::from_str::<Action>(r#"{"Write":"blank","to":"b"}"#).is_err());
}

#[test]
fn malformed_phase_position_checkpoint_and_bits_indices_rejected() {
    for position in [
        calibration(0, 1),
        calibration(4, 1),
        calibration(1, 0),
        calibration(1, 5),
        live(4, 1, 1),
    ] {
        assert!(role_at(position).is_err());
        let json = serde_json::to_string(&position).unwrap();
        assert!(serde_json::from_str::<Position>(&json).is_err());
    }
    assert!(serde_json::from_str::<Phase>(r#"{"Live":{"trial":4}}"#).is_err());
    assert!(serde_json::from_str::<Checkpoint>(r#"{"Prediction":{"trial":4}}"#).is_err());
    assert!(EpisodeBits::from_index(256).is_err());
    assert!(protocol(4).validate().is_err());
}

#[test]
fn sequence_bit_order_is_x_then_y_for_each_trial() {
    assert_eq!(
        EpisodeBits::from_index(1).unwrap().0,
        [
            (true, false),
            (false, false),
            (false, false),
            (false, false)
        ]
    );
    assert_eq!(
        EpisodeBits::from_index(128).unwrap().0,
        [
            (false, false),
            (false, false),
            (false, false),
            (false, true)
        ]
    );
    for index in 0..256 {
        assert_eq!(EpisodeBits::from_index(index).unwrap().index(), index);
    }
}

#[test]
fn lineage_wire_validates_optional_trial_indices() {
    let base = serde_json::json!({
        "writer": "A",
        "write_position": {"phase": {"Live": {"trial": 0}}, "round": 1, "slot": 1},
        "symbol": "Data0"
    });
    assert_eq!(
        serde_json::from_value::<Lineage>(base.clone())
            .unwrap()
            .task_trial,
        None
    );
    let cases = [
        (None, true),
        (Some(0), true),
        (Some(1), true),
        (Some(2), true),
        (Some(3), true),
        (Some(4), false),
        (Some(255), false),
    ];
    let accepted = cases.map(|(task_trial, _)| {
        let mut value = base.clone();
        value["task_trial"] = serde_json::json!(task_trial);
        serde_json::from_value::<Lineage>(value).is_ok()
    });
    assert_eq!(accepted, cases.map(|(_, expected)| expected));

    let mut unknown_field = base.clone();
    unknown_field["receipt"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Lineage>(unknown_field).is_err());
    for write_position in [live(4, 1, 1), live(0, 0, 1), live(0, 1, 5)] {
        let mut value = base.clone();
        value["write_position"] = serde_json::to_value(write_position).unwrap();
        assert!(serde_json::from_value::<Lineage>(value).is_err());
    }
}

#[test]
fn fraction_constructors_reject_invalid_values_and_reduce() {
    assert!(Probability::new(0, 0).is_err());
    assert!(Probability::new(3, 2).is_err());
    assert_eq!(
        Probability::new(2, 4).unwrap(),
        Probability {
            numerator: 1,
            denominator: 2
        }
    );
    assert_eq!(
        ScoreFraction::new(-84, 2).unwrap(),
        ScoreFraction {
            numerator: -42,
            denominator: 1
        }
    );
    assert!(ScoreFraction::new(12, 0).is_err());
    assert_eq!(ScoreFraction::new(i64::MIN, 1).unwrap().numerator, i64::MIN);
}

#[test]
fn wire_fractions_validate_and_canonicalize() {
    for json in [
        r#"{"numerator":1,"denominator":0}"#,
        r#"{"numerator":3,"denominator":2}"#,
        r#"{"numerator":0,"denominator":1,"float":0}"#,
    ] {
        assert!(serde_json::from_str::<Probability>(json).is_err());
    }
    assert_eq!(
        serde_json::from_str::<Probability>(r#"{"numerator":2,"denominator":4}"#).unwrap(),
        Probability::new(1, 2).unwrap()
    );
    assert_eq!(
        serde_json::from_str::<ScoreFraction>(r#"{"numerator":-84,"denominator":2}"#).unwrap(),
        ScoreFraction::new(-42, 1).unwrap()
    );
}

#[test]
fn exact_fraction_arithmetic_detects_overflow_and_probability_bounds() {
    let half = Probability::new(1, 2).unwrap();
    assert_eq!(
        half.checked_add(half).unwrap(),
        Probability::new(1, 1).unwrap()
    );
    assert_eq!(
        half.checked_mul(half).unwrap(),
        Probability::new(1, 4).unwrap()
    );
    assert!(half.checked_add(Probability::new(1, 1).unwrap()).is_err());
    assert_eq!(
        half.checked_cmp(Probability::new(1, 3).unwrap()).unwrap(),
        Ordering::Greater
    );
    assert!(matches!(
        Probability::new(u64::MAX - 1, u64::MAX)
            .unwrap()
            .checked_cmp(half),
        Err(Error::ArithmeticOverflow)
    ));
    assert!(matches!(
        ScoreFraction::new(i64::MAX, 1)
            .unwrap()
            .checked_add(ScoreFraction::new(1, 1).unwrap()),
        Err(Error::ArithmeticOverflow)
    ));
    assert_eq!(
        ScoreFraction::new(-42, 1)
            .unwrap()
            .checked_mul(ScoreFraction::new(1, 2).unwrap())
            .unwrap(),
        ScoreFraction::new(-21, 1).unwrap()
    );
}
