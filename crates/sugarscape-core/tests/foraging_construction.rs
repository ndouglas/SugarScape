use sugarscape_core::foraging::construction::{
    run, Parameters, Pos, Resource, RunOptions, Setup, World,
};
fn pos(x: u32, y: u32) -> Pos {
    Pos { x, y }
}
fn setup() -> Setup {
    Setup {
        width: 5,
        height: 3,
        open: vec![pos(0, 0), pos(1, 0), pos(2, 0), pos(0, 1)],
        diggable: vec![pos(3, 0)],
        nest: vec![pos(0, 0), pos(1, 0)],
        waste: pos(0, 1),
        workers: vec![pos(0, 0)],
        food: vec![Resource {
            id: u64::MAX,
            pos: pos(3, 0),
        }],
        parameters: Parameters {
            p_search: 1.0,
            p_return: 0.0,
            lambda_fidelity: 0.0,
            lambda_publish: 0.0,
            lambda_waypoint: 0.0,
        },
    }
}
fn options(ticks: u32, sample_every: u32, snapshots: bool) -> RunOptions {
    RunOptions {
        ticks,
        sample_every,
        snapshots,
    }
}
#[test]
fn run_matches_repeated_steps_without_observer_side_effects() {
    let s = setup();
    let e = run(s.clone(), 12, options(40, 7, true)).unwrap();
    let mut w = World::new(s, 12).unwrap();
    for _ in 0..40 {
        w.step().unwrap();
    }
    assert_eq!(e.summary, w.summary().unwrap());
    assert_eq!(e.snapshots.first().unwrap().summary.completed_ticks, 0);
    assert_eq!(e.snapshots.last().unwrap().summary.completed_ticks, 40);
    assert_eq!(e.summary.work.opportunities, 40);
    for f in &e.snapshots {
        let s = &f.summary;
        assert_eq!(
            s.food.initial,
            s.food.hidden + s.food.available + s.food.carried + s.food.delivered
        );
        assert_eq!(s.spoil.excavated, s.spoil.carried + s.spoil.disposed);
        assert_eq!(s.terrain.open, s.terrain.initial_open + s.terrain.excavated);
        assert_eq!(
            s.work.opportunities,
            s.work.moves
                + s.work.digs
                + s.work.pickups
                + s.work.deposits
                + s.work.disposals
                + s.work.waits
        );
        assert_eq!(s.access.compute.calls, 1 + u64::from(s.terrain.excavated));
    }
}
#[test]
fn replay_and_sampling_are_observational() {
    let mut s = setup();
    s.open.reverse();
    s.food.push(Resource {
        id: 0,
        pos: pos(4, 2),
    });
    let e = run(s.clone(), 12, options(40, 7, true)).unwrap();
    assert_eq!(run(e.setup.clone(), e.seed, e.options.clone()).unwrap(), e);
    assert_eq!(e.setup.workers, s.workers);
    for (interval, enabled) in [(1, true), (7, true), (7, false)] {
        let r = run(s.clone(), 12, options(40, interval, enabled)).unwrap();
        assert_eq!(r.summary, e.summary);
        if !enabled {
            assert!(r.snapshots.is_empty());
            assert_eq!(r.snapshot_bytes, 0);
        }
    }
    assert_eq!(
        e.snapshot_bytes,
        e.snapshots
            .iter()
            .map(|f| serde_json::to_vec(f).unwrap().len() as u64)
            .sum::<u64>()
    );
}
#[test]
fn hidden_protected_and_exposed_disconnected_food_remain_censored() {
    let mut s = setup();
    s.diggable.clear();
    s.open.push(pos(4, 2));
    s.food.push(Resource {
        id: 0,
        pos: pos(4, 2),
    });
    let e = run(s, 12, options(40, 100, true)).unwrap();
    assert_eq!(
        (
            e.summary.food.hidden,
            e.summary.food.available,
            e.summary.food.delivered
        ),
        (1, 1, 0)
    );
    assert_eq!(
        (
            e.summary.access.initially_exposed,
            e.summary.access.initially_accessible,
            e.summary.access.accessible
        ),
        (1, 0, 0)
    );
    assert!(e.summary.milestones.first_exposure.is_none());
    assert!(e.summary.milestones.first_access.is_none());
    assert!(e.summary.milestones.all_food_delivered_tick.is_none());
    assert_eq!(e.summary.completed_ticks, 40);
}
#[test]
fn public_errors_are_aggregated_and_knowledge_is_contextual() {
    let mut s = setup();
    s.width = 0;
    let errors = run(s, 12, options(0, 0, false)).unwrap_err();
    for field in ["width", "ticks", "sample_every"] {
        assert!(errors.iter().any(|e| e.field == field));
    }
    let w = World::new(setup(), 12).unwrap();
    assert!(w.knowledge(1).unwrap_err()[0]
        .field
        .contains("knowledge.agent[1]"));
}
#[test]
fn initial_and_final_frames_are_unique_at_aligned_or_long_cadence() {
    for (ticks, cadence, expected) in [
        (1, 1, vec![0, 1]),
        (40, 40, vec![0, 40]),
        (40, 100, vec![0, 40]),
    ] {
        let e = run(setup(), 12, options(ticks, cadence, true)).unwrap();
        assert_eq!(
            e.snapshots
                .iter()
                .map(|f| f.summary.completed_ticks)
                .collect::<Vec<_>>(),
            expected
        );
    }
}

#[test]
fn legal_maximum_tick_run_continues_after_empty_food_exhaustion() {
    let mut s = setup();
    s.food.clear();
    s.diggable.clear();
    let e = run(s, 12, options(7200, 7200, false)).unwrap();
    assert_eq!(e.summary.completed_ticks, 7200);
    assert_eq!(e.summary.work.opportunities, 7200);
    assert!(e.summary.milestones.all_food_delivered_tick.is_none());
    assert!(e.snapshots.is_empty());
    assert_eq!(e.snapshot_bytes, 0);
}
