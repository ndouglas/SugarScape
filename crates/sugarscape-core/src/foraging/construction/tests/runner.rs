use super::*;
use crate::foraging::construction::runner::{append_snapshot, validate_options};
use crate::foraging::passage as f3;
use rand::RngCore;

fn options(ticks: u32, sample_every: u32, snapshots: bool) -> RunOptions {
    RunOptions {
        ticks,
        sample_every,
        snapshots,
    }
}
fn continuation(w: &World) -> [u64; 4] {
    let mut rng = w.rng.clone();
    std::array::from_fn(|_| rng.next_u64())
}
#[test]
fn run_matches_steps_and_replays_complete_normalized_episode() {
    let mut s = setup();
    s.open.reverse();
    s.nest.reverse();
    s.diggable.reverse();
    s.workers = vec![pos(1, 0), pos(0, 0)];
    s.food.push(Resource {
        id: 0,
        pos: pos(4, 2),
    });
    let e = run(s.clone(), 12, options(40, 7, true)).unwrap();
    let mut w = World::new(s.clone(), 12).unwrap();
    for _ in 0..40 {
        w.step().unwrap();
    }
    assert_eq!(e.summary, w.summary().unwrap());
    assert_eq!(e.setup, s.normalized().unwrap());
    assert_eq!(e.options, options(40, 7, true));
    assert_eq!(run(e.setup.clone(), e.seed, e.options.clone()).unwrap(), e);
    assert_eq!(
        e.snapshots
            .iter()
            .map(|f| f.summary.completed_ticks)
            .collect::<Vec<_>>(),
        vec![0, 7, 14, 21, 28, 35, 40]
    );
}
#[test]
fn cadence_and_views_preserve_full_state_and_rng_including_access_cache() {
    let s = setup();
    let recorded = run(s.clone(), 12, options(40, 1, true)).unwrap();
    for (cadence, enabled) in [(7, true), (40, true), (100, true), (1, false)] {
        let e = run(s.clone(), 12, options(40, cadence, enabled)).unwrap();
        assert_eq!(e.summary, recorded.summary);
        if !enabled {
            assert!(e.snapshots.is_empty());
            assert_eq!(e.snapshot_bytes, 0);
        }
    }
    let mut observed = World::new(s, 12).unwrap();
    let mut untouched = observed.clone();
    let mut cadence_seven = observed.clone();
    for completed in 1..=40 {
        let before = observed.state.clone();
        let draws = continuation(&observed);
        observed.summary().unwrap();
        observed.snapshot().unwrap();
        observed.knowledge(0).unwrap();
        assert_eq!(observed.state, before);
        assert_eq!(continuation(&observed), draws);
        observed.step().unwrap();
        untouched.step().unwrap();
        cadence_seven.step().unwrap();
        if completed % 7 == 0 || completed == 40 {
            cadence_seven.snapshot().unwrap();
            cadence_seven.summary().unwrap();
            cadence_seven.knowledge(0).unwrap();
        }
        assert_eq!(observed.state, cadence_seven.state);
        assert_eq!(continuation(&observed), continuation(&cadence_seven));
        assert_eq!(observed.state, untouched.state);
        assert_eq!(continuation(&observed), continuation(&untouched));
    }
    assert_eq!(
        observed.knowledge(0).unwrap(),
        untouched.knowledge(0).unwrap()
    );
    assert_eq!(
        observed.knowledge(0).unwrap(),
        cadence_seven.knowledge(0).unwrap()
    );
}
#[test]
fn knowledge_is_private_sorted_classifications_and_invalid_identity_is_contextual() {
    let mut s = setup();
    s.workers = vec![pos(1, 0)];
    let w = World::new(s, 12).unwrap();
    let view = w.knowledge(0).unwrap();
    assert!(view.cells.windows(2).all(|p| p[0].pos < p[1].pos));
    assert!(view
        .cells
        .iter()
        .any(|c| c.kind == CellKnowledge::KnownSolid { diggable: false }));
    assert_eq!(
        serde_json::to_value(view).unwrap()["cells"][0]
            .as_object()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        w.knowledge(u32::MAX).unwrap_err()[0].field,
        "knowledge.agent[4294967295]"
    );
    let frame = w.snapshot().unwrap();
    assert_eq!(frame.summary.work.opportunities, 0);
    assert_eq!(frame.summary.compute.observations, 1);
    assert_eq!(frame.summary.access.compute.calls, 1);
    assert_eq!(frame.summary.access.compute.visits, 4);
    assert_eq!(frame.agents[0].mode, Mode::Departing);
}
#[test]
fn summary_checks_sums_and_uses_maximum_peak() {
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
fn exact_snapshot_limit_commits_and_short_limit_is_transactional() {
    let frame = run(setup(), 12, options(40, 40, true))
        .unwrap()
        .snapshots
        .pop()
        .unwrap();
    let size = serde_json::to_vec(&frame).unwrap().len() as u64;
    let mut frames = vec![];
    let mut bytes = 0;
    append_snapshot(&mut frames, &mut bytes, frame.clone(), size).unwrap();
    assert_eq!(bytes, size);
    assert_eq!(frames, vec![frame.clone()]);
    let before = frames.clone();
    let errors = append_snapshot(&mut frames, &mut bytes, frame.clone(), 2 * size - 1).unwrap_err();
    assert_eq!(errors[0].field, "snapshot_bytes");
    assert!(errors[0].message.contains("completed tick 40"));
    assert_eq!(frames, before);
    assert_eq!(bytes, size);
    let mut empty = vec![];
    let mut zero = 0;
    assert!(append_snapshot(&mut empty, &mut zero, frame, size - 1).is_err());
    assert!(empty.is_empty());
    assert_eq!(zero, 0);
}
#[test]
fn snapshot_bytes_include_all_fields_and_exclude_episode_wrapper() {
    let e = run(setup(), 12, options(40, 5, true)).unwrap();
    let sum = e
        .snapshots
        .iter()
        .map(|f| serde_json::to_vec(f).unwrap().len() as u64)
        .sum::<u64>();
    assert_eq!(e.snapshot_bytes, sum);
    assert!(serde_json::to_vec(&e).unwrap().len() as u64 > sum);
    for f in &e.snapshots {
        let j = serde_json::to_value(f).unwrap();
        for key in [
            "summary",
            "open",
            "nest",
            "waste",
            "agents",
            "food",
            "spoil",
            "waypoints",
        ] {
            assert!(j.get(key).is_some());
        }
        assert_eq!(f.open.len() as u32, f.summary.terrain.open);
        assert_eq!(f.nest, e.setup.nest);
        assert_eq!(f.waste, e.setup.waste);
    }
}
#[test]
fn options_aggregate_setup_and_run_errors_before_execution() {
    assert!(validate_options(&setup(), &options(7200, 1, false)).is_ok());
    for ticks in [0, 7201] {
        assert!(run(setup(), 0, options(ticks, 1, false)).is_err());
    }
    assert!(run(setup(), 0, options(1, 0, false)).is_err());
    let mut invalid = setup();
    invalid.width = 0;
    let errors = run(invalid, 0, options(0, 0, false)).unwrap_err();
    for field in ["width", "ticks", "sample_every"] {
        assert!(errors.iter().any(|e| e.field == field));
    }
}
fn full_chamber() -> Setup {
    let mut s = setup();
    s.width = 16;
    s.height = 9;
    s.nest = (0..16)
        .flat_map(|x| (0..8).map(move |y| pos(x, y)))
        .collect();
    s.open = s.nest.clone();
    s.waste = pos(0, 8);
    s.open.push(s.waste);
    s.workers = s.nest.iter().flat_map(|&p| [p, p]).collect();
    s.food.clear();
    s.diggable.clear();
    s
}
#[test]
fn opportunity_and_tick_boundaries_use_helper_states_and_roll_back() {
    let s = full_chamber();
    s.validate().unwrap();
    assert_eq!(s.workers.len(), 256);
    assert!(validate_options(&s, &options(3906, 1, false)).is_ok());
    assert!(validate_options(&s, &options(3907, 1, false))
        .unwrap_err()
        .iter()
        .any(|e| e.field == "opportunities"));
    let mut w = World::new(s, 0).unwrap();
    w.state.tick = 3905;
    w.step().unwrap();
    assert_eq!(w.state.tick, 3906);
    let state = w.state.clone();
    let rng = continuation(&w);
    assert!(w.step().is_err());
    assert_eq!(w.state, state);
    assert_eq!(continuation(&w), rng);
    let mut w = World::new(setup(), 0).unwrap();
    w.state.tick = 7199;
    w.step().unwrap();
    assert_eq!(w.summary().unwrap().completed_ticks, 7200);
    let state = w.state.clone();
    let rng = continuation(&w);
    assert!(w.step().is_err());
    assert_eq!(w.state, state);
    assert_eq!(continuation(&w), rng);
}
#[test]
fn maximum_grid_maps_and_spoil_capacity_exclude_initially_open_mask_cells() {
    let mut s = full_chamber();
    s.width = 125;
    s.height = 125;
    s.diggable = (0..125)
        .flat_map(|x| (0..125).map(move |y| pos(x, y)))
        .collect();
    s.validate().unwrap();
    let terrain = Terrain::new(&s).unwrap();
    assert_eq!(terrain.capacity(), 15625 - s.open.len() as u32);
    let map = Knowledge::new(125, 125).unwrap();
    assert_eq!(map.dimensions(), (125, 125));
    assert_eq!(
        u64::from(s.width) * u64::from(s.height) * s.workers.len() as u64,
        4_000_000
    );
    assert!(Knowledge::new(126, 125).is_err());
    assert!(super::super::spoil::SpoilLedger::new(15626).is_err());
}
#[test]
fn empty_food_and_censored_materials_continue_full_horizon() {
    let mut empty = setup();
    empty.food.clear();
    empty.diggable.clear();
    let e = run(empty, 0, options(40, 100, true)).unwrap();
    assert_eq!(e.summary.completed_ticks, 40);
    assert_eq!(e.summary.work.opportunities, 40);
    assert_eq!(e.summary.milestones, Milestones::default());
    let mut protected = setup();
    protected.diggable.clear();
    let e = run(protected, 12, options(40, 100, true)).unwrap();
    assert_eq!(e.summary.food.hidden, 1);
    assert_eq!(e.summary.food.delivered, 0);
    assert_eq!(e.summary.milestones, Milestones::default());
    // Inspect each cutoff: carried spoil/food remains conserved without fabricated completion.
    let mut w = World::new(setup(), 12).unwrap();
    let mut saw_spoil = false;
    let mut saw_food = false;
    for _ in 0..40 {
        w.step().unwrap();
        let f = w.snapshot().unwrap();
        if f.summary.spoil.carried > 0 {
            saw_spoil = true;
            assert!(f
                .spoil
                .iter()
                .any(|s| matches!(s.state, SpoilState::Carried { .. })));
        }
        if f.summary.food.carried > 0 {
            saw_food = true;
            assert!(f.summary.milestones.all_food_delivered_tick.is_none());
        }
    }
    assert!(saw_spoil);
    assert!(saw_food);
}
#[test]
fn weak_waypoints_and_frozen_find_are_observational() {
    let mut s = setup();
    s.food[0].pos = pos(2, 0);
    s.diggable.clear();
    s.parameters.lambda_waypoint = 10.0_f64.ln();
    let mut w = World::new(s, 12).unwrap();
    let mut saw_find = false;
    for _ in 0..40 {
        w.step().unwrap();
        let f = w.snapshot().unwrap();
        if let Some(find) = &f.agents[0].find {
            saw_find = true;
            assert_eq!(
                find,
                &FindView {
                    site: pos(2, 0),
                    count: 1
                }
            );
        }
        if !f.waypoints.is_empty() && f.waypoints[0].strength < 0.001 {
            let state = w.state.clone();
            let rng = continuation(&w);
            assert_eq!(f.summary.expired_records, 0);
            w.summary().unwrap();
            w.knowledge(0).unwrap();
            assert_eq!(w.snapshot().unwrap(), f);
            assert_eq!(w.state, state);
            assert_eq!(continuation(&w), rng);
            assert!(saw_find);
            return;
        }
    }
    panic!("fixture must retain decayed advice before another food arrival");
}
fn food_work(w: &WorkCounts) -> f3::WorkCounts {
    f3::WorkCounts {
        opportunities: w.opportunities,
        moves: w.moves,
        departure_moves: w.departure_moves,
        search_moves: w.search_moves,
        empty_return_moves: w.empty_return_moves,
        loaded_return_moves: w.food_moves,
        pickups: w.pickups,
        deposits: w.deposits,
        waits: w.waits,
        transition_waits: w.transition_waits,
        congestion_waits: w.empty_congestion_waits + w.food_congestion_waits,
        no_neighbor_waits: w.no_neighbor_waits,
        empty_arrival_waits: w.empty_arrival_waits,
        search_entries: w.search_entries,
        empty_returns: w.empty_returns,
        fidelity_departures: w.fidelity_departures,
        recruited_departures: w.recruited_departures,
        uninformed_departures: w.uninformed_departures,
        publications: w.publications,
        abandoned_targets: w.abandoned_targets,
    }
}
#[test]
fn no_dig_reduction_matches_every_step_and_pcg_continuation() {
    for seed in [0, 12, u64::MAX] {
        for mask in [vec![], vec![pos(2, 0)]] {
            let mut s = setup();
            s.food[0].pos = pos(2, 0);
            s.diggable = mask;
            let baseline = f3::Setup {
                width: s.width,
                height: s.height,
                open: s.open.clone(),
                nest: s.nest.clone(),
                workers: s.workers.clone(),
                resources: s.food.clone(),
                parameters: s.parameters.clone(),
            };
            let mut a = World::new(s, seed).unwrap();
            let mut b = f3::World::new(baseline, seed).unwrap();
            for tick in 0..=40 {
                let x = a.snapshot().unwrap();
                let y = b.snapshot().unwrap();
                assert_eq!(x.summary.completed_ticks, y.summary.completed_ticks);
                assert_eq!(
                    x.food
                        .iter()
                        .map(|f| (
                            f.resource.clone(),
                            match f.state {
                                FoodState::Available => f3::ResourceState::Available,
                                FoodState::Carried { agent } =>
                                    f3::ResourceState::Carried { agent },
                                FoodState::Delivered => f3::ResourceState::Delivered,
                                FoodState::Hidden => panic!("no hidden baseline food"),
                            }
                        ))
                        .collect::<Vec<_>>(),
                    y.resources
                        .iter()
                        .map(|f| (f.resource.clone(), f.state.clone()))
                        .collect::<Vec<_>>(),
                    "seed {seed} tick {tick}"
                );
                for (x, y) in x.agents.iter().zip(&y.agents) {
                    let phase = match x.phase {
                        FoodPhase::Departing => f3::Phase::Departing,
                        FoodPhase::Searching => f3::Phase::Searching,
                        FoodPhase::Returning => f3::Phase::Returning,
                    };
                    let cargo = match x.cargo {
                        None => None,
                        Some(Cargo::Food(id)) => Some(id),
                        Some(Cargo::Spoil(_)) => panic!("open mask cannot be dug"),
                    };
                    assert_eq!(
                        (x.id, x.pos, phase, cargo, x.site, x.frontier),
                        (y.id, y.pos, y.phase, y.cargo, y.site, y.frontier),
                        "seed {seed} tick {tick}"
                    );
                    assert_eq!(
                        x.find.as_ref().map(|f| (f.site, f.count)),
                        y.find.as_ref().map(|f| (f.site, f.count))
                    );
                    assert_eq!(food_work(&x.work), y.work);
                }
                assert_eq!(food_work(&x.summary.work), y.summary.work);
                assert_eq!(
                    (
                        x.summary.work.digs,
                        x.summary.work.disposals,
                        x.summary.spoil.excavated
                    ),
                    (0, 0, 0)
                );
                assert_eq!(
                    continuation(&a),
                    f3::construction_rng_probe(&b),
                    "seed {seed} tick {tick}"
                );
                if tick < 40 {
                    a.step().unwrap();
                    b.step().unwrap();
                }
            }
        }
    }
}

// Supplemental acceptance checks added after the initial 12-test GREEN.
#[test]
fn sixty_four_mib_boundary_counts_cumulative_bytes_before_append() {
    let frame = World::new(setup(), 12).unwrap().snapshot().unwrap();
    let size = serde_json::to_vec(&frame).unwrap().len() as u64;
    let limit = 64 * 1024 * 1024;
    // Model previously serialized frames by their counted bytes, without allocating 64 MiB.
    let mut frames = vec![];
    let mut bytes = limit - size;
    append_snapshot(&mut frames, &mut bytes, frame.clone(), limit).unwrap();
    assert_eq!(bytes, limit);
    let before = frames.clone();
    assert!(append_snapshot(&mut frames, &mut bytes, frame.clone(), limit).is_err());
    assert_eq!(frames, before);
    assert_eq!(bytes, limit);
    let mut frames = vec![];
    let mut bytes = limit - size + 1;
    let before = bytes;
    assert!(append_snapshot(&mut frames, &mut bytes, frame, limit).is_err());
    assert!(frames.is_empty());
    assert_eq!(bytes, before);
}

fn fixture_action(w: &mut World, i: usize, action: super::super::decision::Action) {
    use super::super::{actions::apply, controller::occupancy, decision::Decision};
    let observation = observe(
        &w.state.terrain,
        w.state.agents[i].pos,
        &occupancy(&w.state),
        &w.state.food.available(),
    )
    .unwrap();
    let mut agent = w.state.agents[i].clone();
    let delta = agent.map.learn(&observation).unwrap();
    agent
        .compute
        .checked_include(&ComputeCounts {
            observations: 1,
            cells_inspected: observation.cells.len() as u64,
            cells_learned: delta.first,
            observed_revisions: delta.observed_revisions,
            ..Default::default()
        })
        .unwrap();
    apply(
        &w.setup,
        &mut w.state,
        i,
        Decision { agent, action },
        &observation,
        &mut Scripted::new(&[]),
    )
    .unwrap();
}
fn fixture_tick(w: &mut World, i: usize, action: super::super::decision::Action) {
    use super::super::decision::{Action, WaitReason};
    // Paid real moves and waits preserve complete-tick event numbering.
    for j in 0..w.state.agents.len() {
        let action = if i == j {
            action
        } else {
            Action::Wait(WaitReason::NoNeighbor)
        };
        fixture_action(w, j, action);
    }
    w.state.tick += 1;
}
#[test]
fn knowledge_view_keeps_stale_wall_while_snapshot_shows_excavation_and_frozen_access() {
    use super::super::decision::Action;
    let mut s = setup();
    s.workers = vec![pos(1, 0), pos(0, 0)];
    let mut w = World::new(s, 12).unwrap();
    fixture_tick(&mut w, 0, Action::Move(pos(2, 0)));
    fixture_tick(&mut w, 1, Action::Move(pos(1, 0)));
    fixture_tick(&mut w, 1, Action::Move(pos(2, 0)));
    fixture_tick(&mut w, 1, Action::Move(pos(1, 0)));
    fixture_tick(&mut w, 1, Action::Move(pos(0, 0)));
    fixture_tick(&mut w, 0, Action::Dig(pos(3, 0)));
    let state = w.state.clone();
    let rng = continuation(&w);
    let knowledge = w.knowledge(1).unwrap();
    assert!(knowledge.cells.contains(&KnownCell {
        pos: pos(3, 0),
        kind: CellKnowledge::KnownSolid { diggable: true }
    }));
    let frame = w.snapshot().unwrap();
    assert!(frame.open.contains(&pos(3, 0)));
    assert_eq!(frame.agents[1].known_diggable, 1);
    assert_eq!(frame.summary.access.compute.calls, 2);
    let event = frame.summary.access.records[0]
        .first_access
        .clone()
        .unwrap();
    assert_eq!(event.nest_distance, 2);
    assert_eq!(event.context.opportunity, 11);
    assert_eq!(
        w.summary().unwrap().access.records[0].first_access,
        Some(event)
    );
    assert_eq!(w.state, state);
    assert_eq!(continuation(&w), rng);
}
