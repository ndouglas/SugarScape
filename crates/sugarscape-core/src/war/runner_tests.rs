use super::{
    config::{EngagementConfig, Geometry, StudyInput},
    records::{Capture, CapturedFrame, ObservedFrame, RunPayload, RunRecord},
    runner::run_to,
    EndReason, Engagement,
};
use crate::{config::Config, stats::Snapshot, world::World};

fn graph() -> EngagementConfig {
    EngagementConfig {
        blue: 12,
        red: 10,
        blue_rate: 0.2,
        red_rate: 0.3,
        dt: 0.1,
        max_steps: 8,
        geometry: Geometry::DuelContact,
    }
}

fn collect(input: &StudyInput, steps: Vec<u64>) -> (super::records::RunSummary, Vec<RunRecord>) {
    let mut records = Vec::new();
    let summary = run_to(
        input,
        &Capture {
            retain_steps: steps,
        },
        &mut |r| {
            records.push(r.clone());
            Ok(())
        },
    )
    .unwrap();
    (summary, records)
}

// Catches replacing literal World execution with an approximate control or
// observing before settlement / after the next tick's event reset.
#[test]
fn book_frames_match_direct_world_at_every_tick_for_combat_controls() {
    for preset in [
        "iii-9-combat",
        "iii-11-combat-fixed",
        "iii-14-combat-culture",
    ] {
        for seed in [7, 8] {
            let config = crate::presets::by_id(preset).unwrap().config;
            let input = StudyInput::BookC {
                config: config.clone(),
                seed,
                max_steps: 32,
            };
            let (summary, records) = collect(&input, vec![0, 1, 32]);
            let mut direct = World::new(config, seed).unwrap();
            let books: Vec<_> = records
                .iter()
                .filter_map(|r| match &r.payload {
                    RunPayload::Observed {
                        frame: ObservedFrame::Book { frame },
                    } => Some(frame),
                    _ => None,
                })
                .collect();
            assert_eq!(books.len(), 32, "{preset}, seed {seed}");
            for book in books {
                direct.step();
                assert_eq!(book.tick, direct.tick);
                assert_eq!(book.snapshot, Snapshot::of(&direct));
                assert_eq!(book.fingerprint, format!("0x{:016x}", direct.fingerprint()));
                assert_eq!(book.rng_state, crate::rng::state_json(&direct.rng));
                assert_eq!(book.combat_enabled, direct.config.combat.enabled);
                let deaths: Vec<_> = book
                    .deaths
                    .iter()
                    .map(|d| (d.id, d.tribe.clone(), d.cause.clone()))
                    .collect();
                let direct_deaths: Vec<_> = direct
                    .events()
                    .deaths
                    .iter()
                    .map(|d| {
                        let tribe = match d.tribe {
                            crate::agent::Tribe::Blue => "blue",
                            crate::agent::Tribe::Red => "red",
                        };
                        let cause = match d.cause {
                            crate::world::DeathCause::Starvation => "starvation",
                            crate::world::DeathCause::OldAge => "old_age",
                            crate::world::DeathCause::Combat => "combat",
                        };
                        (d.id, tribe.to_string(), cause.to_string())
                    })
                    .collect();
                assert_eq!(deaths, direct_deaths);
                assert_eq!(
                    book.kills
                        .iter()
                        .map(|k| (k.attacker, k.victim, k.loot))
                        .collect::<Vec<_>>(),
                    direct
                        .events()
                        .kills
                        .iter()
                        .map(|k| (k.attacker, k.victim, k.loot))
                        .collect::<Vec<_>>()
                );
                for good in 0..direct.config.goods.len() {
                    assert_eq!(
                        book.agent_stores[good],
                        direct.agents().map(|a| a.holdings[good]).sum::<f64>()
                    );
                    assert_eq!(
                        book.site_stores[good],
                        direct.sites.iter().map(|s| s.resource[good]).sum::<f64>()
                    );
                }
            }
            assert_eq!(summary.completed_steps, 32);
            assert_eq!(summary.capture.len(), 3);
        }
    }
}

// Catches graph endings leaking into World controls, or disabled combat being
// mislabeled / enabled by the dispatcher.
#[test]
fn disabled_combat_empty_world_runs_its_full_horizon() {
    let config = Config {
        population: 0,
        ..Config::default()
    };
    let (summary, records) = collect(
        &StudyInput::BookC {
            config,
            seed: 7,
            max_steps: 3,
        },
        vec![3],
    );
    assert_eq!(summary.completed_steps, 3);
    assert!(summary.ending.is_none());
    let books: Vec<_> = records
        .iter()
        .filter_map(|r| match &r.payload {
            RunPayload::Observed {
                frame: ObservedFrame::Book { frame },
            } => Some(frame),
            _ => None,
        })
        .collect();
    assert_eq!(
        books
            .iter()
            .map(|b| (b.tick, b.combat_enabled, b.snapshot.population))
            .collect::<Vec<_>>(),
        vec![(1, false, 0), (2, false, 0), (3, false, 0)]
    );
}

// Catches clocks advancing only after a successful sink write: the actual model
// settlement already completed, and its checkpoint must be retained.
#[test]
fn graph_sink_failure_retains_completed_settlement_and_distinguishes_emitted_steps() {
    for fail_step in [1, 3] {
        let config = graph();
        let mut direct = Engagement::new(config.clone(), 7).unwrap();
        for _ in 0..fail_step {
            direct.step().unwrap();
        }
        let failure = run_to(&StudyInput::ReciprocalGraph { config, seed: 7 }, &Capture { retain_steps: vec![fail_step] }, &mut |r| {
            if matches!(&r.payload, RunPayload::Observed { frame: ObservedFrame::Graph { frame, .. } } if frame.step == fail_step) { Err("sink full".into()) } else { Ok(()) }
        }).unwrap_err();
        assert_eq!(failure.kind, "output");
        assert_eq!(failure.detail, "sink full");
        assert_eq!(failure.attempted_step, Some(fail_step));
        assert_eq!(failure.completed_steps, fail_step);
        assert_eq!(failure.emitted_steps, fail_step - 1);
        assert_eq!(failure.checkpoint.unwrap(), direct.checkpoint());
    }
}

#[test]
fn book_sink_failure_reports_completed_and_emitted_ticks() {
    let input = StudyInput::BookC {
        config: Config::default(),
        seed: 7,
        max_steps: 3,
    };
    let failure = run_to(&input, &Capture { retain_steps: vec![] }, &mut |r| {
        if matches!(&r.payload, RunPayload::Observed { frame: ObservedFrame::Book { frame } } if frame.tick == 2) { Err("write failed".into()) } else { Ok(()) }
    }).unwrap_err();
    assert_eq!(
        (
            failure.completed_steps,
            failure.emitted_steps,
            failure.attempted_step
        ),
        (2, 1, Some(2))
    );
    assert!(failure.checkpoint.is_none());
}

// Catches treating an unchanged World tick as a completed requested tick.
#[test]
fn halted_world_reports_only_ticks_that_actually_completed() {
    use crate::minds::behavior_tree::{
        lab,
        state::{Controller, LabConfig, Scenario},
    };
    let config = lab::rig_config(LabConfig {
        controller: Controller::ReactiveUtility,
        scenario: Scenario::Stable,
        quota: 20,
        mirrored: false,
    });
    let mut direct = World::new(config.clone(), 7).unwrap();
    for _ in 0..65 {
        direct.step();
    }
    assert_eq!(direct.tick, 64);
    let failure = run_to(
        &StudyInput::BookC {
            config,
            seed: 7,
            max_steps: 65,
        },
        &Capture {
            retain_steps: vec![],
        },
        &mut |_| Ok(()),
    )
    .unwrap_err();
    assert_eq!(failure.kind, "world_halted");
    assert_eq!(
        (
            failure.completed_steps,
            failure.emitted_steps,
            failure.attempted_step
        ),
        (64, 64, Some(65))
    );
    assert!(failure.checkpoint.is_none());
}

// Catches serde_json silently representing overflowing mandatory stock sums as
// null, despite every individual World site value being finite.
#[test]
fn overflowing_book_stock_observation_fails_after_the_completed_tick() {
    let mut config = Config {
        population: 0,
        ..Config::default()
    };
    config.goods[0].map = crate::config::Map::Flat { capacity: f64::MAX };
    config.validate().unwrap();
    let mut direct = World::new(config.clone(), 7).unwrap();
    direct.step();
    assert_eq!(direct.tick, 1);
    assert!(direct.sites.iter().all(|s| s.resource[0].is_finite()));
    let mut records = Vec::new();
    let failure = run_to(
        &StudyInput::BookC {
            config,
            seed: 7,
            max_steps: 2,
        },
        &Capture {
            retain_steps: vec![1],
        },
        &mut |r| {
            records.push(r.clone());
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(failure.kind, "invalid_observation");
    assert!(failure.detail.contains("site_stores[0]"));
    assert_eq!(
        (
            failure.completed_steps,
            failure.emitted_steps,
            failure.attempted_step
        ),
        (1, 0, Some(1))
    );
    assert!(failure.checkpoint.is_none());
    assert_eq!(records.len(), 2);
    serde_json::to_string(&failure).unwrap();
}

#[test]
fn header_initial_and_terminal_sink_failures_never_return_success() {
    for fail_kind in ["header", "initial", "terminal"] {
        let failure = run_to(
            &StudyInput::ReciprocalGraph {
                config: graph(),
                seed: 7,
            },
            &Capture {
                retain_steps: vec![],
            },
            &mut |r| {
                let kind = match &r.payload {
                    RunPayload::Header { .. } => "header",
                    RunPayload::Observed {
                        frame: ObservedFrame::Initial { .. },
                    } => "initial",
                    RunPayload::Terminal { .. } => "terminal",
                    _ => "settlement",
                };
                if kind == fail_kind {
                    Err(fail_kind.into())
                } else {
                    Ok(())
                }
            },
        )
        .unwrap_err();
        assert_eq!(failure.kind, "output");
        let want = if fail_kind == "terminal" { 8 } else { 0 };
        assert_eq!(
            (failure.completed_steps, failure.emitted_steps),
            (want, want)
        );
        assert_eq!(failure.checkpoint.unwrap().step, want);
    }
}

// Catches recursive terminal capture, dropped step zero, or fabricated future
// frames after a genuine initial extinction.
#[test]
fn terminal_at_zero_retains_initial_and_marks_future_capture_unavailable() {
    let config = EngagementConfig { blue: 0, ..graph() };
    let (summary, records) = collect(&StudyInput::ReciprocalGraph { config, seed: 7 }, vec![0, 1]);
    assert_eq!(summary.completed_steps, 0);
    assert_eq!(summary.ending.unwrap().reason, EndReason::OneSideExtinction);
    assert!(matches!(
        &summary.capture[0],
        CapturedFrame::Available {
            frame: ObservedFrame::Initial {
                counts: Some([0, 10])
            },
            ..
        }
    ));
    assert!(
        matches!(&summary.capture[1], CapturedFrame::Unavailable { step: 1, reason } if reason.contains("one_side_extinction"))
    );
    assert_eq!(records.len(), 3);
}

// Catches capture validation happening after World/Engagement construction or
// after any output, including bounds/overflow on untrusted requests.
#[test]
fn invalid_capture_requests_are_rejected_before_output() {
    for retain_steps in [vec![1, 1], vec![9], (0..1025).collect()] {
        let mut calls = 0;
        let failure = run_to(
            &StudyInput::ReciprocalGraph {
                config: graph(),
                seed: 7,
            },
            &Capture { retain_steps },
            &mut |_| {
                calls += 1;
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.kind, "invalid_input");
        assert_eq!(calls, 0);
        assert_eq!((failure.completed_steps, failure.emitted_steps), (0, 0));
        assert!(failure.checkpoint.is_none());
    }
}

#[test]
fn invalid_book_resource_bounds_and_config_are_rejected_before_output() {
    let oversized = Config {
        width: u32::MAX,
        height: u32::MAX,
        ..Config::default()
    };
    let too_many_sites = Config {
        width: 65,
        height: 65,
        ..Config::default()
    };
    let invalid = Config {
        population: u32::MAX,
        ..Config::default()
    };
    for (config, max_steps) in [
        (Config::default(), 0),
        (Config::default(), u64::MAX),
        (Config::default(), 25_601),
        (oversized, 1),
        (too_many_sites, 1),
        (invalid, 1),
    ] {
        let mut calls = 0;
        let failure = run_to(
            &StudyInput::BookC {
                config,
                seed: 7,
                max_steps,
            },
            &Capture {
                retain_steps: vec![],
            },
            &mut |_| {
                calls += 1;
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(failure.kind, "invalid_input");
        assert_eq!(calls, 0);
    }
}

#[test]
fn invalid_graph_config_is_rejected_before_output() {
    let mut calls = 0;
    let config = EngagementConfig {
        dt: f64::NAN,
        ..graph()
    };
    let failure = run_to(
        &StudyInput::ReciprocalGraph { config, seed: 7 },
        &Capture {
            retain_steps: vec![],
        },
        &mut |_| {
            calls += 1;
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(failure.kind, "invalid_input");
    assert_eq!(calls, 0);
}

// The tiny positive rate remains representable to the kernel at this dt but
// the independent reference's normalized imbalance is unresolved.
#[test]
fn unavailable_reference_preserves_successful_finite_graph_frame() {
    let config = EngagementConfig {
        blue: 1,
        red: 2,
        blue_rate: f64::from_bits(4.0_f64.to_bits() + 1),
        red_rate: 1.0,
        dt: 0.01,
        max_steps: 1,
        geometry: Geometry::AimedFire,
    };
    let mut direct = Engagement::new(config.clone(), 7).unwrap();
    let expected = direct.step().unwrap().unwrap();
    let (summary, records) = collect(&StudyInput::ReciprocalGraph { config, seed: 7 }, vec![1]);
    let found = records
        .iter()
        .find_map(|r| match &r.payload {
            RunPayload::Observed {
                frame:
                    ObservedFrame::Graph {
                        frame,
                        reference,
                        reference_error,
                        ..
                    },
            } => Some((frame, reference, reference_error)),
            _ => None,
        })
        .unwrap();
    assert_eq!(found.0, &expected);
    assert!(found.1.is_none());
    assert!(found.2.is_some());
    assert_eq!(summary.completed_steps, 1);
}

#[test]
fn invalid_numeric_attempt_keeps_last_checkpoint_and_completed_prefix() {
    let config = EngagementConfig {
        blue_rate: 1e-20,
        red_rate: 1e-20,
        ..graph()
    };
    let direct = Engagement::new(config.clone(), 7).unwrap();
    let mut records = Vec::new();
    let failure = run_to(
        &StudyInput::ReciprocalGraph { config, seed: 7 },
        &Capture {
            retain_steps: vec![],
        },
        &mut |r| {
            records.push(r.clone());
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(failure.kind, "invalid_numeric");
    assert_eq!(
        (
            failure.completed_steps,
            failure.emitted_steps,
            failure.attempted_step
        ),
        (0, 0, Some(1))
    );
    assert_eq!(failure.checkpoint.unwrap(), direct.checkpoint());
    assert_eq!(records.len(), 2);
}

#[test]
fn invalid_numeric_after_a_completed_prefix_preserves_both_rng_streams() {
    let config = EngagementConfig {
        blue: 2,
        red: 4,
        blue_rate: 3e-16,
        red_rate: 0.1,
        dt: 1.0,
        max_steps: 32,
        geometry: Geometry::AimedFire,
    };
    let mut direct = Engagement::new(config.clone(), 7).unwrap();
    let mut completed = 0;
    let expected_failure = loop {
        match direct.step() {
            Ok(Some(_)) => completed += 1,
            Err(failure) => break failure,
            Ok(None) => panic!("engineering fixture must fail after a completed prefix"),
        }
    };
    assert!(completed > 0);
    let mut records = Vec::new();
    let failure = run_to(
        &StudyInput::ReciprocalGraph { config, seed: 7 },
        &Capture {
            retain_steps: vec![1],
        },
        &mut |r| {
            records.push(r.clone());
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(failure.kind, "invalid_numeric");
    assert_eq!(
        (
            failure.completed_steps,
            failure.emitted_steps,
            failure.attempted_step
        ),
        (completed, completed, Some(expected_failure.attempted_step))
    );
    assert_eq!(failure.checkpoint.unwrap(), direct.checkpoint());
    assert_eq!(records.len(), completed as usize + 2);
}

// Catches observation consuming semantic RNG or captures adding extra steps.
#[test]
fn requested_captures_and_reference_observation_preserve_direct_graph_frames() {
    for geometry in [Geometry::AimedFire, Geometry::DuelContact] {
        let config = EngagementConfig {
            geometry,
            ..graph()
        };
        let input = StudyInput::ReciprocalGraph {
            config: config.clone(),
            seed: 8,
        };
        let (none, records) = collect(&input, vec![]);
        let (selected, captured_records) = collect(&input, vec![8, 0, 3]);
        assert_eq!(
            serde_json::to_value(&records[..records.len() - 1]).unwrap(),
            serde_json::to_value(&captured_records[..captured_records.len() - 1]).unwrap()
        );
        assert!(none.capture.is_empty());
        assert_eq!(selected.capture.len(), 3);
        let mut direct = Engagement::new(config, 8).unwrap();
        for record in &records {
            if let RunPayload::Observed {
                frame: ObservedFrame::Graph { frame, .. },
            } = &record.payload
            {
                assert_eq!(frame, &direct.step().unwrap().unwrap());
            }
        }
        for (capture, step) in selected.capture.iter().zip([8, 0, 3]) {
            let CapturedFrame::Available { frame, .. } = capture else {
                panic!("requested completed step missing")
            };
            match frame {
                ObservedFrame::Initial { .. } => assert_eq!(step, 0),
                ObservedFrame::Graph { frame, .. } => assert_eq!(frame.step, step),
                _ => panic!("wrong path"),
            }
        }
    }
}

#[test]
fn seed_construction_order_does_not_change_records_or_identity() {
    let input = |seed| StudyInput::ReciprocalGraph {
        config: graph(),
        seed,
    };
    let (_, first) = collect(&input(7), vec![]);
    let (_, other) = collect(&input(8), vec![]);
    let (_, repeated) = collect(&input(7), vec![]);
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&repeated).unwrap()
    );
    assert_ne!(first[0].input_identity, other[0].input_identity);
    assert!(first
        .iter()
        .all(|r| r.input_identity == first[0].input_identity));
    assert_eq!(first[0].input_identity, "war1-input-v1:0x7acc6027a402dbff");
}

#[test]
fn wire_records_are_tagged_and_unavailable_quantities_are_honest() {
    for input in [
        StudyInput::BookC {
            config: Config::default(),
            seed: 7,
            max_steps: 1,
        },
        StudyInput::ReciprocalGraph {
            config: graph(),
            seed: 7,
        },
    ] {
        let (_, records) = collect(&input, vec![1]);
        let json = serde_json::to_value(&records).unwrap();
        assert_eq!(json[0]["schema"], "war1-records-v1");
        assert_eq!(json[0]["payload"]["kind"], "header");
        assert_eq!(json[1]["payload"]["data"]["frame"]["kind"], "initial");
        let RunPayload::Header { header } = &records[0].payload else {
            panic!("header missing")
        };
        assert!(header.equality.resources.is_none());
        match input {
            StudyInput::BookC { .. } => {
                assert!(header.equality.counts.is_none());
                assert!(header.equality.rates.is_none());
                assert_eq!(header.clock_unit, "world_tick");
                let RunPayload::Observed {
                    frame: ObservedFrame::Book { frame },
                } = &records[2].payload
                else {
                    panic!("book missing")
                };
                for quantity in [
                    "harvest_flow",
                    "removed_wealth",
                    "death_site",
                    "active_battle_time",
                ] {
                    assert!(frame
                        .unavailable
                        .iter()
                        .any(|u| u.quantity == quantity && !u.reason.is_empty()));
                }
            }
            StudyInput::ReciprocalGraph { .. } => {
                assert_eq!(header.equality.counts, Some(false));
                assert_eq!(header.equality.rates, Some(false));
                assert_eq!(header.clock_unit, "dimensionless_model_time");
                for quantity in ["resources", "geography", "killer"] {
                    assert!(header
                        .unavailable
                        .iter()
                        .any(|u| u.quantity == quantity && !u.reason.is_empty()));
                }
            }
        }
        assert_eq!(
            json.as_array().unwrap().last().unwrap()["payload"]["kind"],
            "terminal"
        );
    }
}

// Catches accepted World snapshots becoming unlabeled JSON null observations.
#[test]
fn nonfinite_book_snapshot_fails_after_completed_tick_before_emission() {
    let mut config = Config::default();
    config.goods[0].map = crate::config::Map::Flat {
        capacity: 1.121e303,
    };
    config.validate().unwrap();
    let mut direct = World::new(config.clone(), 7).unwrap();
    direct.step();
    let snapshot = Snapshot::of(&direct);
    assert!(!snapshot.gini.is_finite());
    assert!(!snapshot.gini_total.is_finite());
    assert!(direct
        .agents()
        .map(|a| a.holdings[0])
        .sum::<f64>()
        .is_finite());
    assert!(direct
        .sites
        .iter()
        .map(|s| s.resource[0])
        .sum::<f64>()
        .is_finite());
    let mut records = Vec::new();
    let failure = run_to(
        &StudyInput::BookC {
            config,
            seed: 7,
            max_steps: 1,
        },
        &Capture {
            retain_steps: vec![],
        },
        &mut |r| {
            records.push(r.clone());
            Ok(())
        },
    )
    .unwrap_err();
    assert_eq!(failure.kind, "invalid_observation");
    assert!(
        failure.detail.contains("snapshot.gini"),
        "{}",
        failure.detail
    );
    assert!(failure.detail.contains("tick 1"));
    assert_eq!(
        (
            failure.completed_steps,
            failure.emitted_steps,
            failure.attempted_step
        ),
        (1, 0, Some(1))
    );
    assert_eq!(records.len(), 2);
}

// Catches loss of the explicit casualty interpretation in the wire envelope.
#[test]
fn graph_envelope_serializes_benchmark_exposure_cause() {
    let (_, records) = collect(
        &StudyInput::ReciprocalGraph {
            config: graph(),
            seed: 7,
        },
        vec![],
    );
    let json = serde_json::to_value(&records[2]).unwrap();
    assert_eq!(
        json["payload"]["data"]["frame"]["data"]["cause"],
        "benchmark_exposure"
    );
}

// Catches omitted guards for vectors/nested statistics and misclassified absence.
#[test]
fn snapshot_guard_checks_all_serialized_numbers_and_allows_absent_groups() {
    use crate::stats::{GoodStats, MemoryStats};
    let finite = Snapshot::default();
    super::runner::validate_snapshot(&finite).unwrap();
    let cases = [
        (
            Snapshot {
                gini_total: f64::NEG_INFINITY,
                ..finite.clone()
            },
            "snapshot.gini_total",
        ),
        (
            Snapshot {
                goods: vec![GoodStats {
                    traded: f64::NAN,
                    ..GoodStats::default()
                }],
                ..finite.clone()
            },
            "snapshot.goods[0].traded",
        ),
        (
            Snapshot {
                pollution: vec![f64::INFINITY],
                ..finite.clone()
            },
            "snapshot.pollution[0]",
        ),
        (
            Snapshot {
                groups: vec![f64::NAN],
                ..finite.clone()
            },
            "snapshot.groups[0]",
        ),
        (
            Snapshot {
                memory: Some(MemoryStats {
                    belief_error: f64::NAN,
                    ..MemoryStats::default()
                }),
                ..finite
            },
            "snapshot.memory.belief_error",
        ),
    ];
    for (snapshot, path) in cases {
        let error = super::runner::validate_snapshot(&snapshot).unwrap_err();
        assert!(error.contains(path), "{error}");
    }
}
