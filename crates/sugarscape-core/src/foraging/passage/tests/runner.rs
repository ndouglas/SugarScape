use super::*;
use crate::foraging::passage::runner::{append_snapshot, validate_options};
use rand::RngCore;

fn options(ticks: u32, sample_every: u32, snapshots: bool) -> RunOptions {
    RunOptions {
        ticks,
        sample_every,
        snapshots,
    }
}
fn food_setup() -> Setup {
    let mut s = setup();
    s.resources.push(Resource {
        id: u64::MAX,
        pos: pos(3, 0),
    });
    s
}
#[test]
fn run_and_repeated_step_share_summary_and_private_knowledge() {
    let s = food_setup();
    let e = run(s.clone(), 12, options(20, 7, true)).unwrap();
    let mut w = World::new(s, 12).unwrap();
    for _ in 0..20 {
        w.step().unwrap();
    }
    assert_eq!(e.summary, w.summary().unwrap());
    assert_eq!(
        e.snapshots
            .iter()
            .map(|s| s.summary.completed_ticks)
            .collect::<Vec<_>>(),
        vec![0, 7, 14, 20]
    );
    assert_eq!(e.summary.work.opportunities, 20);
    assert_eq!(e.summary.compute.observations, 21);
    assert_eq!(e.summary.inventory.initial, 1);
}
#[test]
fn normalized_episode_preserves_worker_identity_and_original_food_cells() {
    let mut s = food_setup();
    s.open.reverse();
    s.nest.reverse();
    s.workers = vec![pos(1, 0), pos(0, 0)];
    s.resources.push(Resource {
        id: 0,
        pos: pos(3, 1),
    });
    let e = run(s.clone(), 0, options(1, 1, true)).unwrap();
    assert_eq!(e.setup.workers, s.workers);
    assert_eq!(e.setup.nest, vec![pos(0, 0), pos(1, 0)]);
    assert!(e.setup.open.windows(2).all(|pair| pair[0] < pair[1]));
    assert_eq!(
        e.setup.resources,
        vec![
            Resource {
                id: 0,
                pos: pos(3, 1)
            },
            Resource {
                id: u64::MAX,
                pos: pos(3, 0)
            }
        ]
    );
    assert_eq!(e.seed, 0);
    assert_eq!(
        e.snapshots
            .iter()
            .map(|s| s.summary.completed_ticks)
            .collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        run(e.setup.clone(), e.seed, options(1, 1, true)).unwrap(),
        e
    );
}
#[test]
fn sampling_and_observers_leave_state_counters_and_draws_unchanged() {
    let s = food_setup();
    let recorded = run(s.clone(), u64::MAX, options(40, 1, true)).unwrap();
    for cadence in [3, 40, 100] {
        assert_eq!(
            run(s.clone(), u64::MAX, options(40, cadence, true))
                .unwrap()
                .summary,
            recorded.summary
        );
    }
    let disabled = run(s.clone(), u64::MAX, options(40, 1, false)).unwrap();
    assert_eq!(disabled.summary, recorded.summary);
    assert!(disabled.snapshots.is_empty());
    assert_eq!(disabled.snapshot_bytes, 0);
    let mut observed = World::new(s, u64::MAX).unwrap();
    let mut untouched = observed.clone();
    for _ in 0..40 {
        let before = observed.state.clone();
        observed.knowledge(0).unwrap();
        observed.summary().unwrap();
        observed.snapshot().unwrap();
        assert_eq!(observed.state, before);
        observed.step().unwrap();
        untouched.step().unwrap();
    }
    assert_eq!(
        observed.knowledge(0).unwrap(),
        untouched.knowledge(0).unwrap()
    );
    assert_eq!(observed.state, untouched.state);
    assert_eq!(observed.rng.next_u64(), untouched.rng.next_u64());
}
#[test]
fn knowledge_is_sorted_classifications_only_and_invalid_id_is_contextual() {
    let w = World::new(food_setup(), 12).unwrap();
    let view = w.knowledge(0).unwrap();
    assert_eq!(
        view.cells,
        vec![
            KnownCell {
                pos: pos(0, 0),
                kind: CellKnowledge::KnownOpen
            },
            KnownCell {
                pos: pos(0, 1),
                kind: CellKnowledge::KnownSolid
            },
            KnownCell {
                pos: pos(1, 0),
                kind: CellKnowledge::KnownOpen
            }
        ]
    );
    let json = serde_json::to_value(view).unwrap();
    assert_eq!(json["cells"][0].as_object().unwrap().len(), 2);
    assert!(w.knowledge(1).unwrap_err()[0].field.contains("agent"));
    assert!(w.knowledge(u32::MAX).is_err());
    let snapshot = w.snapshot().unwrap();
    assert_eq!(
        (
            snapshot.agents[0].known_open,
            snapshot.agents[0].known_solid
        ),
        (2, 1)
    );
    assert_eq!(snapshot.summary.work.opportunities, 0);
    assert_eq!(snapshot.summary.compute.observations, 1);
}
#[test]
fn summary_uses_checked_additive_totals_and_maximum_queue_peak() {
    let mut s = setup();
    s.workers.push(pos(1, 0));
    let mut w = World::new(s, 0).unwrap();
    w.state.agents[0].compute.peak_queue = 4;
    w.state.agents[1].compute.peak_queue = 7;
    let summary = w.summary().unwrap();
    assert_eq!(summary.compute.peak_queue, 7);
    assert_eq!(summary.compute.observations, 2);
    assert_eq!(summary.per_agent_compute.len(), 2);
    assert_eq!(summary.per_agent_work.len(), 2);
    w.state.agents[0].compute.observations = u64::MAX;
    assert!(w.summary().unwrap_err()[0]
        .field
        .contains("compute.observations"));
}
#[test]
fn snapshot_budget_exact_limit_commits_and_one_byte_short_rolls_back() {
    let frame = World::new(food_setup(), 12).unwrap().snapshot().unwrap();
    let size = serde_json::to_vec(&frame).unwrap().len() as u64;
    let mut frames = vec![];
    let mut bytes = 0;
    append_snapshot(&mut frames, &mut bytes, frame.clone(), size).unwrap();
    assert_eq!(bytes, size);
    assert_eq!(frames, vec![frame.clone()]);
    let before = frames.clone();
    let errors = append_snapshot(&mut frames, &mut bytes, frame.clone(), 2 * size - 1).unwrap_err();
    assert_eq!(errors[0].field, "snapshot_bytes");
    assert_eq!(frames, before);
    assert_eq!(bytes, size);
    let mut empty = vec![];
    let mut zero = 0;
    assert!(append_snapshot(&mut empty, &mut zero, frame, size - 1).is_err());
    assert!(empty.is_empty());
    assert_eq!(zero, 0);
}
#[test]
fn serialized_snapshot_sum_includes_geometry_and_all_frame_fields() {
    let e = run(food_setup(), 12, options(20, 5, true)).unwrap();
    let bytes = e
        .snapshots
        .iter()
        .map(|s| serde_json::to_vec(s).unwrap().len() as u64)
        .sum::<u64>();
    assert_eq!(e.snapshot_bytes, bytes);
    assert!(e
        .snapshots
        .iter()
        .all(|s| s.open == e.setup.open && s.nest == e.setup.nest));
}
#[test]
fn option_limits_and_aggregate_errors_are_checked_before_execution() {
    assert!(validate_options(&setup(), &options(7200, 1, false)).is_ok());
    for ticks in [0, 7201] {
        assert!(run(setup(), 0, options(ticks, 1, false)).is_err());
    }
    assert!(run(setup(), 0, options(1, 0, false)).is_err());
    let mut invalid = setup();
    invalid.width = 0;
    let errors = run(invalid, 0, options(0, 0, false)).unwrap_err();
    assert!(errors.iter().any(|e| e.field == "width"));
    assert!(errors.iter().any(|e| e.field == "ticks"));
    assert!(errors.iter().any(|e| e.field == "sample_every"));
}
fn full_chamber() -> Setup {
    let mut s = setup();
    s.width = 16;
    s.height = 8;
    s.nest = (0..16)
        .flat_map(|x| (0..8).map(move |y| pos(x, y)))
        .collect();
    s.open = s.nest.clone();
    s.workers = s.nest.iter().flat_map(|&p| [p, p]).collect();
    s
}
#[test]
fn population_budget_boundary_and_single_step_exhaustion_are_atomic() {
    let s = full_chamber();
    s.validate().unwrap();
    assert!(validate_options(&s, &options(3906, 1, false)).is_ok());
    assert!(validate_options(&s, &options(3907, 1, false))
        .unwrap_err()
        .iter()
        .any(|e| e.field == "opportunities"));
    let mut w = World::new(s, 0).unwrap();
    w.state.tick = 3905;
    w.step().unwrap();
    let before = w.state.clone();
    let mut rng = w.rng.clone();
    assert!(w.step().is_err());
    assert_eq!(w.state, before);
    assert_eq!(w.rng.next_u64(), rng.next_u64());
    let mut w = World::new(setup(), 0).unwrap();
    w.state.tick = 7199;
    w.step().unwrap();
    assert_eq!(w.summary().unwrap().completed_ticks, 7200);
    let before = w.state.clone();
    assert!(w.step().is_err());
    assert_eq!(w.state, before);
}
#[test]
fn empty_and_disconnected_worlds_run_full_horizon_with_absent_milestones() {
    let empty = run(setup(), 0, options(40, 100, true)).unwrap();
    assert_eq!(empty.summary.completed_ticks, 40);
    assert_eq!(
        (
            empty.summary.first_pickup_tick,
            empty.summary.first_delivery_tick,
            empty.summary.all_delivered_tick
        ),
        (None, None, None)
    );
    let mut s = setup();
    s.open.push(pos(4, 4));
    s.resources.push(Resource {
        id: 7,
        pos: pos(4, 4),
    });
    let e = run(s, 12, options(40, 100, true)).unwrap();
    assert_eq!(
        e.summary.inventory,
        Inventory {
            initial: 1,
            available: 1,
            carried: 0,
            delivered: 0
        }
    );
    assert_eq!(e.summary.work.opportunities, 40);
    assert_eq!(
        (
            e.summary.first_pickup_tick,
            e.summary.first_delivery_tick,
            e.summary.all_delivered_tick
        ),
        (None, None, None)
    );
}

#[test]
fn snapshot_converts_frozen_find_and_retains_weak_advice_without_expiry() {
    let mut s = food_setup();
    s.parameters.lambda_waypoint = 10.0_f64.ln();
    let mut w = World::new(s, 12).unwrap();
    w.state.ledger.claim(pos(3, 0), 0).unwrap();
    w.state.agents[0].cargo = Some(u64::MAX);
    w.state.agents[0].phase = Phase::Returning;
    w.state.agents[0].find = Some(crate::foraging::FindRecord { site: 3, count: 1 });
    let loaded = w.snapshot().unwrap();
    assert_eq!(
        loaded.agents[0].find,
        Some(FindView {
            site: pos(3, 0),
            count: 1
        })
    );
    assert_eq!(loaded.resources[0].resource.pos, pos(3, 0));
    assert_eq!(
        loaded.resources[0].state,
        ResourceState::Carried { agent: 0 }
    );
    w.state.ledger.deposit(u64::MAX, 0).unwrap();
    w.state
        .server
        .arrive(&w.setup.parameters, 0, 1, w.state.agents[0].find, [0.0; 3])
        .unwrap();
    w.state.agents[0].cargo = None;
    w.state.agents[0].find = None;
    w.state.agents[0].work.publications = 1;
    w.state.tick = 4;
    let before = w.state.clone();
    let mut rng = w.rng.clone();
    let frame = w.snapshot().unwrap();
    assert_eq!(frame.waypoints.len(), 1);
    assert_eq!(
        (
            frame.waypoints[0].id,
            frame.waypoints[0].site,
            frame.waypoints[0].created_tick
        ),
        (0, pos(3, 0), 0)
    );
    assert!(frame.waypoints[0].strength < 0.001);
    assert_eq!(frame.summary.expired_records, 0);
    assert_eq!(w.state, before);
    assert_eq!(w.rng.next_u64(), rng.next_u64());
}
