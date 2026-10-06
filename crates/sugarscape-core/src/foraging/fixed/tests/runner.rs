use super::super::runner::append_snapshot;
use super::super::{run, RunOptions, World};
use super::setup;
fn options(ticks: u32, sample_every: u32, snapshots: bool) -> RunOptions {
    RunOptions {
        ticks,
        sample_every,
        snapshots,
    }
}
// Break caught: skipping invalid inactive options or constructing before aggregate validation.
#[test]
fn rejects_whole_run_budgets_and_aggregates_setup_errors() {
    for ticks in [0, 7201] {
        assert!(run(setup(), 0, options(ticks, 1, false)).is_err());
    }
    assert!(run(setup(), 0, options(1, 0, false)).is_err());
    let mut s = setup();
    s.agents = 256;
    assert!(run(s, 0, options(3907, 1, false)).is_err());
    let mut s = setup();
    s.width = 0;
    let errors = run(s, 0, options(0, 0, false)).unwrap_err();
    for field in ["width", "ticks", "sample_every"] {
        assert!(errors.iter().any(|e| e.field == field));
    }
}
// Break caught: omitting final frames or duplicating interval-aligned final frames.
#[test]
fn samples_initial_intervals_and_final_once() {
    for (ticks, want) in [(4, vec![0, 2, 4]), (5, vec![0, 2, 4, 5])] {
        let episode = run(setup(), 12, options(ticks, 2, true)).unwrap();
        assert_eq!(
            episode
                .snapshots
                .iter()
                .map(|s| s.completed_ticks)
                .collect::<Vec<_>>(),
            want
        );
        let exact: u64 = episode
            .snapshots
            .iter()
            .map(|s| u64::try_from(serde_json::to_vec(s).unwrap().len()).unwrap())
            .sum();
        assert_eq!(episode.snapshot_bytes, exact);
    }
}
// Break caught: observations consuming random draws or stopping empty worlds early.
#[test]
fn snapshot_sampling_is_an_observer() {
    let sparse = run(setup(), 12, options(20, 7, true)).unwrap();
    let dense = run(setup(), 12, options(20, 1, true)).unwrap();
    let silent = run(setup(), 12, options(20, 1, false)).unwrap();
    assert_eq!(sparse.summary, dense.summary);
    assert_eq!(sparse.summary, silent.summary);
    assert_eq!(sparse.snapshots.last(), dense.snapshots.last());
    assert!(silent.snapshots.is_empty());
    assert_eq!(silent.snapshot_bytes, 0);
    assert_eq!(silent.summary.completed_ticks, 20);
    assert_eq!(silent.summary.work.opportunities, 20);
    assert_eq!(silent.summary.first_delivery_tick, None);
    assert_eq!(silent.summary.first_pickup_tick, None);
    assert_eq!(silent.summary.all_delivered_tick, None);
}
// Break caught: counting Episode overhead or committing a frame after byte-budget failure.
#[test]
fn append_obeys_exact_remaining_bytes_atomically() {
    let snapshot = World::new(setup(), 0).unwrap().snapshot().unwrap();
    let size = u64::try_from(serde_json::to_vec(&snapshot).unwrap().len()).unwrap();
    let mut frames = vec![];
    let mut bytes = 7;
    assert!(append_snapshot(&mut frames, &mut bytes, snapshot.clone(), size + 6).is_err());
    assert_eq!(bytes, 7);
    assert!(frames.is_empty());
    append_snapshot(&mut frames, &mut bytes, snapshot.clone(), size + 7).unwrap();
    assert_eq!(bytes, size + 7);
    assert_eq!(frames, vec![snapshot]);
}
// Break caught: trusting an aggregate or failing to validate ledger in summaries.
#[test]
fn summary_recomputes_checked_work_and_validates_ledger() {
    let mut s = setup();
    s.agents = 2;
    let mut w = World::new(s, 0).unwrap();
    w.state.agents[0].work.waits = u64::MAX;
    w.state.agents[1].work.waits = 1;
    assert!(w.summary().is_err());
    w.state.agents[0].work.waits = 4;
    assert_eq!(w.summary().unwrap().work.waits, 5);
    w.state.agents[0].cargo = Some(99);
    assert!(w.summary().is_err());
}
