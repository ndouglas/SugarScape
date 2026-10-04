use super::super::{ledger::Ledger, *};
use rand::RngCore;

fn options(ticks: u32, sample_every: u32) -> RunOptions {
    RunOptions {
        ticks,
        sample_every,
    }
}
fn p(x: u32, y: u32) -> Pos {
    Pos { x, y }
}
fn corridor() -> LabConfig {
    LabConfig {
        fixture: Fixture::Corridor {
            length: 7,
            workers: 1,
        },
        ..Default::default()
    }
}
#[test]
fn identical_seed_and_config_reproduce_the_entire_record() {
    let opts = options(16, 4);
    let a = run_episode(LabConfig::default(), 7, opts.clone()).unwrap();
    let b = run_episode(LabConfig::default(), 7, opts).unwrap();
    assert_eq!(
        serde_json::to_string(&a).unwrap(),
        serde_json::to_string(&b).unwrap()
    );
}
#[test]
fn zero_ticks_still_exports_the_starting_frame() {
    let e = run_episode(LabConfig::default(), 7, options(0, 4)).unwrap();
    assert_eq!(e.final_summary.opportunities, 0);
    assert_eq!(e.frames.len(), 1);
    assert_eq!(e.rates.excavation, None);
    assert_eq!(e.rates.disposal, None);
}
#[test]
fn completed_growing_ticks_give_each_worker_one_opportunity() {
    let e = run_episode(LabConfig::default(), 7, options(16, 4)).unwrap();
    assert_eq!(e.completed_ticks, 16);
    assert_eq!(e.final_summary.opportunities, 128);
    for tick in 0..16 {
        let mut ids: Vec<_> = e
            .events
            .iter()
            .filter(|e| e.tick == tick)
            .map(|e| e.worker)
            .collect();
        ids.sort_unstable();
        assert_eq!(ids, (0..8).collect::<Vec<_>>());
    }
}
#[test]
fn sampling_cadence_cannot_change_events_or_terminal_state() {
    let a = run_episode(LabConfig::default(), 7, options(17, 1)).unwrap();
    let b = run_episode(LabConfig::default(), 7, options(17, 4)).unwrap();
    assert_eq!(a.events, b.events);
    assert_eq!(a.choices, b.choices);
    assert_eq!(a.frames.last(), b.frames.last());
    assert_eq!(a.final_summary, b.final_summary);
    assert_eq!(
        b.frames.iter().map(|f| f.tick).collect::<Vec<_>>(),
        [0, 4, 8, 12, 16, 17]
    );
}
#[test]
fn choice_selection_stops_before_any_action_or_opportunity() {
    let c = LabConfig {
        fixture: Fixture::Choice {
            side: Side::Left,
            pile: Pile::OldAccumulation,
        },
        ..Default::default()
    };
    let e = run_episode(c, 7, options(16, 4)).unwrap();
    assert_eq!(e.choices.len(), 1);
    assert!(e.events.is_empty());
    assert_eq!(e.completed_ticks, 0);
    assert_eq!(e.final_summary.opportunities, 0);
    assert_eq!(e.stop_reason, "choice_selected");
    assert_eq!(e.deliveries.len(), 4);
    assert!(e
        .deliveries
        .iter()
        .all(|d| d.born == 0 && d.waiting_ticks == 0));
    assert_ne!(
        e.frames.first().unwrap().fingerprint,
        e.frames.last().unwrap().fingerprint
    );
}
#[test]
fn sampling_and_replay_budget_are_checked() {
    assert!(run_episode(LabConfig::default(), 7, options(1, 0))
        .unwrap_err()
        .iter()
        .any(|e| e.field == "sample_every"));
    assert!(run_episode(LabConfig::default(), 7, options(125_001, 1))
        .unwrap_err()
        .iter()
        .any(|e| e.field == "ticks"));
    let c = LabConfig {
        fixture: Fixture::Choice {
            side: Side::Left,
            pile: Pile::SingleFresh,
        },
        freshness_window: (u64::MAX - 1) / 2,
        ..Default::default()
    };
    assert!(run_episode(c, 7, options(2, 1))
        .unwrap_err()
        .iter()
        .any(|e| e.field == "ticks"));
}
#[test]
fn decimal_seed_and_diagnostics_survive_serialization() {
    let e = run_episode(corridor(), u64::MAX, options(10, 3)).unwrap();
    assert_eq!(e.seed, "18446744073709551615");
    assert_eq!(
        serde_json::from_str::<Episode>(&serde_json::to_string(&e).unwrap()).unwrap(),
        e
    );
    assert_eq!(
        e.spatial_work,
        vec![SpatialWork {
            pos: p(7, 1),
            digs: 1,
            workers: vec![0]
        }]
    );
    assert_eq!(
        e.dig_distances,
        vec![DigDistance {
            tick: 0,
            worker: 0,
            pos: p(7, 1),
            distance: 7
        }]
    );
    assert_eq!(e.travel.loaded, 6);
    assert!(e.labels.sources.iter().any(|s| s.contains("supplied")));
    assert!(e.labels.assumptions.iter().any(|s| s.contains("history")));
    assert_eq!(e.storage.events, 10);
    assert_eq!(e.storage.retained_records, 27);
    assert_eq!(e.travel.unloaded, 2);
    assert_eq!(e.rates.excavation, Some(0.1));
    assert_eq!(e.rates.disposal, Some(0.1));
    assert_eq!(e.storage.deliveries, 1);
    assert_eq!(e.storage.peak_exit_field_cells, 27);
}
#[test]
fn exit_field_is_rebuilt_only_after_geometry_changes() {
    let e = run_episode(corridor(), 7, options(7, 1)).unwrap();
    assert_eq!(e.final_summary.exit_bfs_calls, 2);
    assert_eq!(e.final_summary.exit_bfs_visits, 15);
    assert_eq!(e.final_summary.connected_open, 8);
    assert_eq!(e.final_summary.observation_bfs_calls, 7);
    assert_eq!(e.final_summary.controller_bfs_calls, 1);
}
fn apply(w: &mut World, ledger: &mut Ledger, tick: u64, worker: u32, action: Action) {
    w.tick = tick;
    let event = w.apply(worker, action);
    assert_eq!(event.outcome, Outcome::Success);
    ledger.record(&event, w);
    w.check_invariants().unwrap();
}
#[test]
fn ledger_tracks_two_carriers_loaded_moves_loose_intervals_and_disposal() {
    let mut s = fixtures::corridor_setup(7, 1).unwrap();
    s.workers.push(p(6, 1));
    let mut w = World::from_setup(corridor(), s, 7).unwrap();
    let mut ledger = Ledger::new(&w);
    apply(&mut w, &mut ledger, 0, 0, Action::Dig(p(7, 1)));
    apply(&mut w, &mut ledger, 1, 0, Action::Move(p(5, 1)));
    apply(&mut w, &mut ledger, 2, 0, Action::Drop);
    apply(&mut w, &mut ledger, 4, 1, Action::Move(p(5, 1)));
    apply(&mut w, &mut ledger, 5, 1, Action::Pickup);
    apply(&mut w, &mut ledger, 6, 1, Action::Drop);
    apply(&mut w, &mut ledger, 8, 1, Action::Pickup);
    for x in (0..5).rev() {
        apply(
            &mut w,
            &mut ledger,
            9 + u64::from(4 - x),
            1,
            Action::Move(p(x, 1)),
        );
    }
    apply(&mut w, &mut ledger, 14, 1, Action::Dispose);
    assert_eq!(
        ledger.finish(&w),
        vec![Delivery {
            material: 0,
            born: 0,
            disposed_at: Some(14),
            carriers: vec![0, 1],
            carried_moves: 6,
            waiting_ticks: 5
        }]
    );
}
#[test]
fn ledger_retains_carrying_and_loose_censored_units() {
    let mut s = fixtures::corridor_setup(7, 1).unwrap();
    s.spoil.push(InitialSpoil {
        pos: p(5, 1),
        born: 0,
    });
    s.workers.push(p(6, 1));
    s.diggable.push(p(6, 0));
    let mut w = World::from_setup(corridor(), s, 7).unwrap();
    let mut ledger = Ledger::new(&w);
    apply(&mut w, &mut ledger, 2, 0, Action::Dig(p(7, 1)));
    apply(&mut w, &mut ledger, 4, 0, Action::Drop);
    apply(&mut w, &mut ledger, 5, 1, Action::Dig(p(6, 0)));
    w.tick = 9;
    assert_eq!(
        ledger.finish(&w),
        vec![
            Delivery {
                material: 0,
                born: 0,
                disposed_at: None,
                carriers: vec![],
                carried_moves: 0,
                waiting_ticks: 9
            },
            Delivery {
                material: 1,
                born: 2,
                disposed_at: None,
                carriers: vec![0],
                carried_moves: 0,
                waiting_ticks: 5
            },
            Delivery {
                material: 2,
                born: 5,
                disposed_at: None,
                carriers: vec![1],
                carried_moves: 0,
                waiting_ticks: 0
            }
        ]
    );
}
#[test]
fn fixture_old_birth_is_sensory_history_not_observed_loose_wait() {
    let mut s = fixtures::corridor_setup(7, 1).unwrap();
    s.start_tick = 64;
    s.spoil.push(InitialSpoil {
        pos: p(6, 1),
        born: 0,
    });
    let mut w = World::from_setup(corridor(), s, 7).unwrap();
    let mut ledger = Ledger::new(&w);
    apply(&mut w, &mut ledger, 67, 0, Action::Pickup);
    w.tick = 70;
    assert_eq!(
        ledger.finish(&w),
        vec![Delivery {
            material: 0,
            born: 0,
            disposed_at: None,
            carriers: vec![0],
            carried_moves: 0,
            waiting_ticks: 3
        }]
    );
}
#[test]
fn blocked_actions_count_once_and_never_add_loaded_travel() {
    let mut w = World::new(corridor(), 7).unwrap();
    let mut r = std::mem::take(&mut w.recording);
    let event = w.apply(0, Action::Move(p(7, 1)));
    r.record(&event, &w, None);
    let event = w.apply(0, Action::Wait);
    r.record(&event, &w, None);
    w.recording = r;
    let summary = w.snapshot();
    assert_eq!(
        (
            summary.opportunities,
            summary.blocked,
            summary.waits,
            summary.successful_moves
        ),
        (2, 1, 1, 0)
    );
    assert_eq!(w.recording.travel.loaded, 0);
}
#[test]
fn fingerprint_covers_rng_continuation_targets_clock_and_configuration() {
    let w = World::new(corridor(), 7).unwrap();
    for change in 0..4 {
        let mut other = w.clone();
        match change {
            0 => {
                other.rng.next_u64();
            }
            1 => other.workers[0].target = Some(p(7, 1)),
            2 => other.tick += 1,
            _ => other.config.relay_distance += 1,
        }
        assert_ne!(w.fingerprint(), other.fingerprint());
    }
    let mut cloned_rng = w.rng.clone();
    let expected = cloned_rng.next_u64();
    w.fingerprint();
    let mut cloned_rng = w.rng.clone();
    assert_eq!(cloned_rng.next_u64(), expected);
}
#[test]
fn ascii_worker_overlay_exposes_loaded_state_and_preserves_exit() {
    let mut w = World::new(corridor(), 7).unwrap();
    assert!(super::super::view::ascii(&w).contains("E.....w##"));
    w.apply(0, Action::Dig(p(7, 1)));
    assert!(super::super::view::ascii(&w).contains("E.....W.#"));
}

#[test]
fn ascii_budget_rejects_conservative_request_before_running() {
    let c = LabConfig {
        fixture: Fixture::Corridor {
            length: 87_379,
            workers: 1,
        },
        ..Default::default()
    };
    assert!(run_episode(c.clone(), 7, options(256, 1))
        .unwrap_err()
        .iter()
        .any(|e| e.field == "sample_every" && e.message.contains("64 MiB")));
    let e = run_episode(c, 7, options(0, 1)).unwrap();
    assert_eq!(e.frames.len(), 1);
    assert_eq!(e.storage.ascii_bytes, e.frames[0].ascii.len() as u64);
}
#[test]
fn ascii_budget_handles_zero_ticks_cadence_and_checked_overflow() {
    use super::super::runner::requested_ascii_bytes;
    assert_eq!(requested_ascii_bytes(&options(0, 4), 10), Some(10));
    assert_eq!(requested_ascii_bytes(&options(16, 4), 10), Some(60));
    assert_eq!(requested_ascii_bytes(&options(17, 4), 10), Some(60));
    assert_eq!(requested_ascii_bytes(&options(1, 1), u64::MAX), None);
    assert_eq!(requested_ascii_bytes(&options(1, 0), 10), None);
}
#[test]
fn recorded_dig_distance_survives_a_later_shortcut() {
    let mut s = fixtures::corridor_setup(7, 1).unwrap();
    // U-shaped open route: the first dig at (2,1) is distance five,
    // then opening (1,1) shortens that cell's current distance to three.
    s.width = 4;
    s.height = 4;
    s.exit = p(0, 0);
    s.open = vec![p(0, 0), p(0, 1), p(0, 2), p(1, 2), p(2, 2)];
    s.diggable = vec![p(2, 1), p(1, 1)];
    s.workers = vec![p(2, 2)];
    let mut w = World::from_setup(corridor(), s, 7).unwrap();
    w.workers[0].target = Some(p(2, 1));
    w.step();
    w.check_invariants().unwrap();
    for action in [Action::Drop, Action::Move(p(2, 1))] {
        let event = w.apply(0, action);
        assert_eq!(event.outcome, Outcome::Success);
        let mut recording = std::mem::take(&mut w.recording);
        recording.record(&event, &w, None);
        w.recording = recording;
        w.check_invariants().unwrap();
    }
    w.workers[0].target = Some(p(1, 1));
    w.step();
    w.check_invariants().unwrap();
    let field = super::super::observation::exit_distances(&w);
    assert_eq!(field[w.index(p(2, 1)).unwrap()], Some(3));
    assert_eq!(w.recording.dig_distances[0].distance, 5);
}
#[test]
fn public_step_and_replay_have_identical_state_without_sampling_side_effects() {
    let mut w = World::new(corridor(), 7).unwrap();
    for _ in 0..10 {
        w.step();
        w.check_invariants().unwrap();
    }
    let e = run_episode(corridor(), 7, options(10, 3)).unwrap();
    assert_eq!(
        e.frames.last().unwrap().fingerprint,
        format!("{:016x}", w.fingerprint())
    );
    assert_eq!(e.events, w.recording.events);
    assert_eq!(e.final_summary, w.snapshot());
}

#[test]
fn legacy_seed_seven_fingerprints_remain_unchanged() {
    let cases = [
        (
            LabConfig::default(),
            vec![
                "ffd1c042eeec3490",
                "262dc74c70c7697b",
                "1669e2a3d6d86c01",
                "1c3417eab5b56e8c",
                "29dae4f7d2f50b0d",
            ],
        ),
        (
            corridor(),
            vec![
                "2b5b1e03fe6f4044",
                "da06e0422beace6e",
                "26181f66e34f4a35",
                "3c7a8ceeb44e6652",
                "a2054777a5ebad24",
            ],
        ),
        (
            LabConfig {
                fixture: Fixture::Choice {
                    side: Side::Left,
                    pile: Pile::OldAccumulation,
                },
                ..Default::default()
            },
            vec!["0617e012fdbb8e5d", "7e4551a4c30d9f93"],
        ),
    ];
    for (config, expected) in cases {
        let e = run_episode(config, 7, options(16, 4)).unwrap();
        assert_eq!(
            e.frames
                .iter()
                .map(|f| f.fingerprint.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        validate_episode(&e, &options(16, 4)).unwrap();
    }
}
