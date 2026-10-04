use super::super::*;
fn record() -> (Episode, RunOptions) {
    let options = RunOptions {
        ticks: 8,
        sample_every: 3,
    };
    (
        run_episode(LabConfig::default(), 7, options.clone()).unwrap(),
        options,
    )
}
#[test]
fn validation_accepts_checked_construction_record() {
    let (record, options) = record();
    assert_eq!(validate_episode(&record, &options), Ok(()));
}
#[test]
fn validation_rejects_forged_final_inventory() {
    let (mut record, options) = record();
    record.final_summary.inventory.disposed += 1;
    assert!(validate_episode(&record, &options).is_err());
}
#[test]
fn validation_rejects_forged_material() {
    let (mut record, options) = record();
    record
        .events
        .iter_mut()
        .find(|e| matches!(e.action, Action::Dig(_)))
        .unwrap()
        .material = Some(999);
    assert!(validate_episode(&record, &options).is_err());
}
#[test]
fn validation_rejects_duplicate_worker() {
    let (mut record, options) = record();
    record.events[1].worker = record.events[0].worker;
    assert!(validate_episode(&record, &options).is_err());
}
#[test]
fn validation_rejects_bad_fingerprint_short_horizon_and_cadence() {
    for change in 0..3 {
        let (mut record, options) = record();
        match change {
            0 => record.frames[0].fingerprint = "ABC".into(),
            1 => record.completed_ticks -= 1,
            _ => record.frames[1].tick += 1,
        }
        assert!(validate_episode(&record, &options).is_err());
    }
}
#[test]
fn validation_accepts_old_material_choice() {
    let options = RunOptions {
        ticks: 1,
        sample_every: 1,
    };
    let config = LabConfig {
        fixture: Fixture::Choice {
            side: Side::Left,
            pile: Pile::OldAccumulation,
        },
        ..Default::default()
    };
    let record = run_episode(config, 7, options.clone()).unwrap();
    assert_eq!(validate_episode(&record, &options), Ok(()));
}
#[test]
fn validation_accepts_relay_handoffs() {
    let options = RunOptions {
        ticks: 512,
        sample_every: 16,
    };
    let record = run_episode(
        LabConfig {
            transport: Transport::Relay,
            cue: Cue::Responsive,
            ..Default::default()
        },
        7,
        options.clone(),
    )
    .unwrap();
    assert!(record.deliveries.iter().any(|d| d.carriers.len() > 1));
    assert_eq!(validate_episode(&record, &options), Ok(()));
}

#[test]
fn validation_rejects_forged_sample_history_storage_and_choice() {
    for change in 0..7 {
        let (mut record, options) = record();
        match change {
            0 => record.series[1].digs += 1,
            1 => record.frames[0].ascii.push('!'),
            2 => record.storage.retained_records += 1,
            3 => record.dig_distances[0].distance += 1,
            4 => record.seed = "07".into(),
            5 => record.events[0].worker = u32::MAX,
            _ => record.events[0].from.x = u32::MAX,
        }
        assert!(validate_episode(&record, &options).is_err());
    }
    let options = RunOptions {
        ticks: 1,
        sample_every: 1,
    };
    let config = LabConfig {
        fixture: Fixture::Choice {
            side: Side::Right,
            pile: Pile::SingleFresh,
        },
        ..Default::default()
    };
    let mut record = run_episode(config, 8, options.clone()).unwrap();
    record.choices[0].target = Pos { x: 0, y: 0 };
    assert!(validate_episode(&record, &options).is_err());
}

#[test]
fn validation_accepts_zero_ticks_and_corridor_construction() {
    for ticks in [0, 32] {
        let options = RunOptions {
            ticks,
            sample_every: 16,
        };
        let config = LabConfig {
            fixture: Fixture::Corridor {
                length: 8,
                workers: 4,
            },
            ..Default::default()
        };
        let record = run_episode(config, 8, options.clone()).unwrap();
        assert_eq!(validate_episode(&record, &options), Ok(()));
    }
}

#[test]
fn validation_accepts_legal_blocked_transaction_without_policy_replay() {
    use super::super::{observation::observe_measured, runner::Recording, view};
    let config = LabConfig {
        fixture: Fixture::Corridor {
            length: 8,
            workers: 1,
        },
        ..Default::default()
    };
    let mut record = run_episode(
        config.clone(),
        7,
        RunOptions {
            ticks: 0,
            sample_every: 1,
        },
    )
    .unwrap();
    let mut world = World::new(config, 7).unwrap();
    let mut recording = Recording::new(&world);
    recording.ensure_exit_field(&world);
    let (observation, stats) = observe_measured(&world, 0);
    recording.observation_stats.include(stats);
    recording.peak_observation_cells = observation.open.len() as u64;
    recording.peak_observation_frontier = observation.frontier.len() as u64;
    let event = world.apply(0, Action::Move(Pos { x: 8, y: 1 }));
    assert!(matches!(event.outcome, Outcome::Blocked { .. }));
    recording.record(&event, &world, None);
    world.tick = 1;
    record.requested_ticks = 1;
    record.completed_ticks = 1;
    record.events = recording.events.clone();
    record.worker_work = recording.worker_work.clone();
    record.final_summary = recording.snapshot(&world);
    record.series.push(record.final_summary.clone());
    record.frames.push(view::frame(&world));
    record.rates = Rates {
        excavation: Some(0.0),
        disposal: Some(0.0),
    };
    record.storage.events = 1;
    record.storage.peak_events = 1;
    record.storage.frames = 2;
    record.storage.snapshots = 2;
    record.storage.retained_records += 3;
    record.storage.ascii_bytes *= 2;
    record.storage.peak_ascii_bytes = record.storage.ascii_bytes;
    record.storage.peak_observation_cells = recording.peak_observation_cells;
    record.storage.peak_observation_frontier = recording.peak_observation_frontier;
    assert_eq!(
        validate_episode(
            &record,
            &RunOptions {
                ticks: 1,
                sample_every: 1
            }
        ),
        Ok(())
    );
}

#[test]
fn validation_rejects_impossible_controller_search_counts() {
    let (mut record, options) = record();
    record.final_summary.controller_bfs_calls = u64::MAX;
    record.final_summary.controller_bfs_visits = 0;
    record.final_summary.controller_bfs_peak_queue = 0;
    for sample in &mut record.series {
        sample.controller_bfs_calls = u64::MAX;
        sample.controller_bfs_visits = 0;
        sample.controller_bfs_peak_queue = 0;
    }
    assert!(validate_episode(&record, &options).is_err());
}
