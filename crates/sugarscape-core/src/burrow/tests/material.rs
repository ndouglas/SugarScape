use super::super::state::UnitLocation;
use super::super::*;

fn p(x: u32, y: u32) -> Pos {
    Pos { x, y }
}

fn config() -> LabConfig {
    LabConfig {
        fixture: Fixture::Corridor {
            length: 7,
            workers: 1,
        },
        ..Default::default()
    }
}

fn setup() -> Setup {
    fixtures::corridor_setup(7, 1).unwrap()
}

fn world(s: Setup) -> World {
    World::from_setup(config(), s, 7).unwrap()
}

fn blocked(w: &mut World, worker: u32, action: Action, reason: &str) {
    let inventory = w.inventory();
    let open = w.open.clone();
    let positions: Vec<_> = w
        .workers
        .iter()
        .map(|w| (w.pos, w.carried, w.loaded_moves))
        .collect();
    let event = w.apply(worker, action);
    assert!(
        matches!(event.outcome, Outcome::Blocked { reason: ref r } if r.contains(reason)),
        "{event:?}"
    );
    assert_eq!(w.inventory(), inventory);
    assert_eq!(w.open, open);
    assert_eq!(
        w.workers
            .iter()
            .map(|w| (w.pos, w.carried, w.loaded_moves))
            .collect::<Vec<_>>(),
        positions
    );
    w.check_invariants().unwrap();
}

#[test]
fn dig_creates_one_unit_and_full_hands_cannot_dig_again() {
    let mut s = setup();
    s.diggable.push(p(6, 0));
    let mut w = world(s);
    let event = w.apply(0, Action::Dig(p(7, 1)));
    assert_eq!(event.outcome, Outcome::Success);
    assert_eq!(
        (event.from, event.to, event.material),
        (p(6, 1), p(6, 1), Some(0))
    );
    assert_eq!(
        w.inventory(),
        Inventory {
            initial: 0,
            excavated: 1,
            carried: 1,
            loose: 0,
            disposed: 0
        }
    );
    assert!(w.open[w.index(p(7, 1)).unwrap()]);
    blocked(&mut w, 0, Action::Dig(p(6, 0)), "full hands");
}

#[test]
fn drop_then_other_worker_pickup_preserves_identity_and_birth() {
    let mut s = setup();
    s.start_tick = 8;
    s.workers.push(p(6, 1));
    let mut w = world(s);
    assert_eq!(w.apply(0, Action::Dig(p(7, 1))).material, Some(0));
    w.tick = 13;
    assert_eq!(w.apply(0, Action::Drop).material, Some(0));
    assert_eq!(w.units[&0].born, 8);
    assert_eq!(w.apply(1, Action::Pickup).material, Some(0));
    assert_eq!(w.units[&0].location, UnitLocation::Carried(1));
    assert_eq!(w.units[&0].born, 8);
    w.check_invariants().unwrap();
}

#[test]
fn pickup_selects_smallest_loose_id_and_rejects_full_hands() {
    let mut s = setup();
    s.spoil = vec![
        InitialSpoil {
            pos: p(6, 1),
            born: 0,
        },
        InitialSpoil {
            pos: p(6, 1),
            born: 0,
        },
    ];
    let mut w = world(s);
    assert_eq!(w.apply(0, Action::Pickup).material, Some(0));
    blocked(&mut w, 0, Action::Pickup, "full hands");
    assert_eq!(w.inventory().loose, 1);
}

#[test]
fn successive_pickups_compete_for_one_unit() {
    let mut s = setup();
    s.workers.push(p(6, 1));
    s.spoil.push(InitialSpoil {
        pos: p(6, 1),
        born: 0,
    });
    let mut w = world(s);
    assert_eq!(w.apply(0, Action::Pickup).outcome, Outcome::Success);
    blocked(&mut w, 1, Action::Pickup, "no loose spoil");
}

#[test]
fn disposal_requires_exit_and_retains_disposed_unit() {
    let mut w = world(setup());
    w.apply(0, Action::Dig(p(7, 1)));
    blocked(&mut w, 0, Action::Dispose, "exit");
    for x in (0..6).rev() {
        assert_eq!(w.apply(0, Action::Move(p(x, 1))).outcome, Outcome::Success);
    }
    assert_eq!(w.workers[0].loaded_moves, 6);
    assert_eq!(w.apply(0, Action::Dispose).material, Some(0));
    assert_eq!(w.units[&0].location, UnitLocation::Disposed);
    assert_eq!(
        w.inventory(),
        Inventory {
            initial: 0,
            excavated: 1,
            carried: 0,
            loose: 0,
            disposed: 1
        }
    );
    w.check_invariants().unwrap();
}

#[test]
fn non_neighbors_and_solid_moves_are_rejected() {
    let mut w = world(setup());
    blocked(&mut w, 0, Action::Move(p(4, 1)), "neighbor");
    blocked(&mut w, 0, Action::Dig(p(7, 0)), "neighbor");
    blocked(&mut w, 0, Action::Move(p(7, 1)), "solid");
    blocked(&mut w, 0, Action::Dig(p(5, 1)), "open");
    blocked(&mut w, 0, Action::Dig(p(6, 0)), "non-diggable");
}

#[test]
fn array_edges_never_wrap_or_panic() {
    let mut s = setup();
    s.workers[0] = p(0, 1);
    let mut w = world(s);
    blocked(&mut w, 0, Action::Move(p(u32::MAX, 1)), "bounds");
    blocked(&mut w, 0, Action::Dig(p(9, 1)), "bounds");
    blocked(&mut w, 0, Action::Move(p(8, 1)), "neighbor");
}

#[test]
fn full_destination_blocks_move() {
    let mut s = setup();
    s.workers.extend([p(5, 1), p(5, 1)]);
    blocked(&mut world(s), 0, Action::Move(p(5, 1)), "capacity");
}

#[test]
fn unknown_worker_is_a_contextual_record() {
    let mut w = world(setup());
    let e = w.apply(99, Action::Wait);
    assert_eq!(e.worker, 99);
    assert!(matches!(e.outcome, Outcome::Blocked { reason } if reason.contains("worker 99")));
    w.check_invariants().unwrap();
}

#[test]
fn empty_hands_handling_and_wait_have_no_material_effect() {
    let mut w = world(setup());
    blocked(&mut w, 0, Action::Drop, "empty hands");
    blocked(&mut w, 0, Action::Dispose, "empty hands");
    blocked(&mut w, 0, Action::Pickup, "no loose spoil");
    assert_eq!(w.apply(0, Action::Wait).outcome, Outcome::Success);
    assert_eq!(w.inventory().excavated, 0);
}

#[test]
fn counters_overflow_before_any_mutation() {
    let mut w = world(setup());
    w.next_material = u64::MAX;
    blocked(&mut w, 0, Action::Dig(p(7, 1)), "overflow");
    w.next_material = 0;
    w.excavated = u64::MAX;
    let open = w.open.clone();
    assert!(
        matches!(w.apply(0, Action::Dig(p(7, 1))).outcome, Outcome::Blocked { reason } if reason.contains("overflow"))
    );
    assert_eq!(w.open, open);
    assert!(w.units.is_empty());
}

fn errors(s: Setup) -> Vec<crate::config::FieldError> {
    World::from_setup(config(), s, 7).expect_err("invalid setup accepted")
}

#[test]
fn disconnected_spawns_and_open_cells_are_rejected() {
    let mut s = setup();
    s.open.push(p(8, 2));
    s.workers[0] = p(8, 2);
    assert!(errors(s).iter().any(|e| e.message.contains("connected")));
}

#[test]
fn duplicate_open_and_diggable_cells_are_rejected() {
    for field in ["open", "diggable"] {
        let mut s = setup();
        if field == "open" {
            s.open.push(p(0, 1));
        } else {
            s.diggable.push(p(7, 1));
        }
        assert!(errors(s)
            .iter()
            .any(|e| e.field.contains(field) && e.message.contains("duplicate")));
    }
}

#[test]
fn two_workers_per_cell_are_legal_but_three_are_rejected() {
    let mut s = setup();
    s.workers.push(p(6, 1));
    world(s.clone()).check_invariants().unwrap();
    s.workers.push(p(6, 1));
    assert!(errors(s).iter().any(|e| e.message.contains("capacity")));
}

#[test]
fn non_open_or_out_of_bounds_setup_positions_are_rejected() {
    for field in ["exit", "workers", "open", "diggable", "spoil"] {
        let mut s = setup();
        match field {
            "exit" => s.exit = p(9, 1),
            "workers" => s.workers[0] = p(9, 1),
            "open" => s.open.push(p(9, 1)),
            "diggable" => s.diggable.push(p(9, 1)),
            _ => s.spoil.push(InitialSpoil {
                pos: p(9, 1),
                born: 0,
            }),
        }
        assert!(errors(s).iter().any(|e| e.field.contains(field)));
    }
    let mut s = setup();
    s.exit = p(7, 1);
    assert!(errors(s).iter().any(|e| e.field == "setup.exit"));
    let mut s = setup();
    s.workers[0] = p(7, 1);
    assert!(errors(s).iter().any(|e| e.field.contains("workers")));
    let mut s = setup();
    s.spoil.push(InitialSpoil {
        pos: p(7, 1),
        born: 0,
    });
    assert!(errors(s).iter().any(|e| e.field.contains("spoil")));
}

#[test]
fn future_born_spoil_and_unadvanceable_clock_are_rejected() {
    let mut s = setup();
    s.spoil.push(InitialSpoil {
        pos: p(6, 1),
        born: 1,
    });
    assert!(errors(s).iter().any(|e| e.message.contains("future")));
    let mut s = setup();
    s.start_tick = u64::MAX;
    assert!(errors(s).iter().any(|e| e.message.contains("overflow")));
}

#[test]
fn empty_and_oversized_dimensions_are_checked_before_allocation() {
    for (width, height) in [(0, 3), (9, 0), (u32::MAX, u32::MAX), (1024, 1024)] {
        let mut s = setup();
        s.width = width;
        s.height = height;
        assert!(!errors(s).is_empty());
    }
}

#[test]
fn zero_parameters_and_excessive_choice_weights_are_rejected() {
    for field in [
        "freshness_window",
        "relay_distance",
        "response_weight",
        "minimum_recent_units",
    ] {
        let mut c = config();
        match field {
            "freshness_window" => c.freshness_window = 0,
            "relay_distance" => c.relay_distance = 0,
            "response_weight" => c.response_weight = 0,
            _ => c.minimum_recent_units = 0,
        }
        assert!(c.validate().unwrap_err().iter().any(|e| e.field == field));
    }
    let mut c = config();
    c.response_weight = u32::MAX;
    assert!(c
        .validate()
        .unwrap_err()
        .iter()
        .any(|e| e.message.contains("overflow")));
}

#[test]
fn fixture_sizes_population_and_clock_overflow_are_rejected() {
    for fixture in [
        Fixture::Growing {
            width: 2,
            height: 14,
            workers: 1,
        },
        Fixture::Growing {
            width: 41,
            height: 25,
            workers: 29,
        },
        Fixture::Corridor {
            length: 0,
            workers: 1,
        },
        Fixture::Corridor {
            length: u32::MAX,
            workers: 1,
        },
        Fixture::Corridor {
            length: 7,
            workers: 0,
        },
        Fixture::Corridor {
            length: 7,
            workers: 15,
        },
        Fixture::Corridor {
            length: 3000,
            workers: 4097,
        },
    ] {
        let c = LabConfig {
            fixture,
            ..Default::default()
        };
        assert!(World::new(c, 7).is_err());
    }
    let c = LabConfig {
        fixture: Fixture::Choice {
            side: Side::Left,
            pile: Pile::OldAccumulation,
        },
        freshness_window: u64::MAX,
        ..Default::default()
    };
    assert!(World::new(c, 7).is_err());
}

#[test]
fn growing_spawns_distinct_cells_before_second_occupancy() {
    let c = LabConfig {
        fixture: Fixture::Growing {
            width: 41,
            height: 25,
            workers: 15,
        },
        ..Default::default()
    };
    let w = World::new(c, 7).unwrap();
    assert_eq!(w.workers[0].pos, p(0, 10));
    assert_eq!(w.workers[1].pos, p(1, 10));
    assert_eq!(w.workers[13].pos, p(2, 14));
    assert_eq!(w.workers[14].pos, p(0, 10));
    assert!(w.workers.iter().all(|a| a.pos != w.setup.exit));
    w.check_invariants().unwrap();
}

#[test]
fn corridor_is_bounded_and_spawns_from_deep_end() {
    let s = fixtures::corridor_setup(3, 4).unwrap();
    assert_eq!((s.width, s.height, s.exit), (5, 3, p(0, 1)));
    assert_eq!(s.open, vec![p(0, 1), p(1, 1), p(2, 1)]);
    assert_eq!(s.diggable, vec![p(3, 1)]);
    assert_eq!(s.workers, vec![p(2, 1), p(1, 1), p(0, 1), p(2, 1)]);
}

#[test]
fn choice_has_explicit_clock_and_mirrored_piles() {
    for (side, pos) in [(Side::Left, p(2, 3)), (Side::Right, p(6, 3))] {
        for (pile, count, born) in [
            (Pile::FreshAccumulation, 4, 64),
            (Pile::OldAccumulation, 4, 0),
            (Pile::SingleFresh, 1, 64),
        ] {
            let c = LabConfig {
                fixture: Fixture::Choice { side, pile },
                ..Default::default()
            };
            let w = World::new(c, 7).unwrap();
            assert_eq!((w.tick, w.setup.exit), (64, p(4, 3)));
            assert_eq!(w.inventory().initial, count);
            assert!(w
                .units
                .values()
                .all(|u| u.born == born && u.location == UnitLocation::Loose(pos)));
            w.check_invariants().unwrap();
        }
    }
}

#[test]
fn configuration_and_records_round_trip_and_reject_unknown_fields() {
    let c = LabConfig::default();
    assert_eq!(
        serde_json::from_str::<LabConfig>(&serde_json::to_string(&c).unwrap()).unwrap(),
        c
    );
    assert!(serde_json::from_str::<LabConfig>(r#"{"bogus":1}"#).is_err());
    assert!(serde_json::from_str::<LabConfig>(
        r#"{"fixture":{"corridor":{"length":7,"workers":1,"bogus":1}}}"#
    )
    .is_err());
    let mut w = world(setup());
    let e = w.apply(0, Action::Dig(p(7, 1)));
    assert_eq!(
        serde_json::from_str::<ActionEvent>(&serde_json::to_string(&e).unwrap()).unwrap(),
        e
    );
}

#[test]
fn invariant_checker_detects_geometry_and_material_corruption() {
    let mut w = world(setup());
    let index = w.index(p(7, 1)).unwrap();
    w.open[index] = true;
    assert!(w.check_invariants().unwrap_err().contains("geometry"));
    let mut w = world(setup());
    w.apply(0, Action::Dig(p(7, 1)));
    w.workers[0].carried = None;
    assert!(w.check_invariants().is_err());
}
