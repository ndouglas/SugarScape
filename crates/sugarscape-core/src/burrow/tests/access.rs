use super::super::{
    access::{install_goal_state, validate_task, AccessObserver, GoalGuidance},
    controller::{decide_measured, decide_with_goal_measured, goal_factor, select_ticket},
    observation::observe,
    runner::run_world,
    *,
};
use crate::rng;
use rand::{Rng, RngCore};

fn p(x: u32, y: u32) -> Pos {
    Pos { x, y }
}

fn task(goal: Pos) -> AccessTask {
    AccessTask {
        goal,
        objective: AccessObjective::KnownGoal,
        goal_weight: 3,
    }
}

fn three_cells() -> World {
    World::from_setup(
        LabConfig::default(),
        Setup {
            width: 3,
            height: 3,
            exit: p(0, 1),
            start_tick: 0,
            open: vec![p(0, 1), p(1, 1)],
            diggable: vec![p(2, 1)],
            workers: vec![p(1, 1)],
            spoil: vec![],
        },
        7,
    )
    .unwrap()
}

#[test]
fn access_task_requires_explicit_fields_and_rejects_unknown_fields() {
    for json in [
        r#"{"lab":{},"task":{"goal":{"x":2,"y":1,"z":0},"objective":"explore","goal_weight":3}}"#,
        r#"{"lab":{},"task":{"objective":"explore","goal_weight":3}}"#,
        r#"{"lab":{},"task":{"goal":{"x":2,"y":1},"goal_weight":3}}"#,
        r#"{"lab":{},"task":{"goal":{"x":2,"y":1},"objective":"explore"}}"#,
        r#"{"lab":{},"task":{"goal":{"x":2,"y":1},"objective":"explore","goal_weight":3,"extra":0}}"#,
        r#"{"lab":{},"task":{"goal":{"x":2,"y":1},"objective":"explore","goal_weight":3},"extra":0}"#,
    ] {
        assert!(serde_json::from_str::<AccessConfig>(json).is_err());
    }
    // Strict task coordinates preserve the shared legacy position schema.
    assert_eq!(
        serde_json::from_str::<Pos>(r#"{"x":2,"y":1,"z":0}"#).unwrap(),
        p(2, 1)
    );
    let json =
        r#"{"lab":{},"task":{"goal":{"x":2,"y":1},"objective":"known_goal","goal_weight":3}}"#;
    let config: AccessConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.task, task(p(2, 1)));
    assert_eq!(
        serde_json::from_str::<AccessConfig>(&serde_json::to_string(&config).unwrap()).unwrap(),
        config
    );
}

#[test]
fn access_validation_rejects_invalid_goals_before_installing_state() {
    for (goal, field) in [
        (p(u32::MAX, 1), "task.goal"),
        (p(2, u32::MAX), "task.goal"),
        (p(0, 1), "task.goal"),
        (p(2, 0), "task.goal"),
    ] {
        let mut world = three_cells();
        let before = world.fingerprint();
        let errors = install_goal_state(&mut world, &task(goal)).unwrap_err();
        assert!(errors.iter().any(|error| error.field == field));
        assert!(world.goal_state.is_none());
        assert_eq!(world.fingerprint(), before);
    }
}

#[test]
fn access_validation_checks_positive_weight_and_cue_product() {
    let world = three_cells();
    let mut invalid = task(p(2, 1));
    invalid.goal_weight = 0;
    assert!(validate_task(&invalid, &world)
        .unwrap_err()
        .iter()
        .any(|e| e.field == "task.goal_weight"));
    invalid.goal_weight = u32::MAX;
    assert!(validate_task(&invalid, &world)
        .unwrap_err()
        .iter()
        .any(|e| e.field == "task.goal_weight"));
    invalid.goal_weight = u32::MAX / world.config.response_weight;
    assert!(validate_task(&invalid, &world).is_ok());
}

#[test]
fn explore_installs_no_private_state_or_fingerprint_bytes() {
    let mut world = three_cells();
    let before = world.fingerprint();
    let mut explore = task(p(2, 1));
    explore.objective = AccessObjective::Explore;
    install_goal_state(&mut world, &explore).unwrap();
    assert!(world.goal_state.is_none());
    assert_eq!(world.fingerprint(), before);
}

#[test]
fn access_goal_ticket_keeps_nonimproving_face_eligible() {
    let guidance = GoalGuidance {
        goal: p(0, 1),
        weight: 3,
        seen_open: false,
    };
    let weights = [
        goal_factor(Some(guidance), p(2, 1), p(1, 1)),
        goal_factor(Some(guidance), p(2, 1), p(3, 1)),
    ];
    assert_eq!(weights, [3, 1]);
    assert_eq!(
        (0..4)
            .map(|ticket| select_ticket(&weights, ticket))
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 1]
    );
    assert_eq!(
        goal_factor(
            Some(GoalGuidance {
                seen_open: true,
                ..guidance
            }),
            p(2, 1),
            p(1, 1)
        ),
        1
    );
    assert_eq!(goal_factor(None, p(2, 1), p(1, 1)), 1);
    assert_eq!(goal_factor(Some(guidance), p(2, 1), p(2, 0)), 1);
    assert_eq!(
        goal_factor(
            Some(GoalGuidance {
                goal: p(u32::MAX, u32::MAX),
                ..guidance
            }),
            p(0, 0),
            p(1, 0)
        ),
        3
    );
}

#[test]
fn fresh_selection_multiplies_cue_weights_with_one_integer_draw() {
    let observation = Observation {
        origin: p(2, 1),
        open: vec![ObservedCell {
            pos: p(2, 1),
            occupants: 1,
            loose: 0,
            recent: 2,
        }],
        frontier: vec![p(1, 1), p(3, 1)],
    };
    let worker = WorkerView {
        id: 0,
        pos: p(2, 1),
        carrying: false,
        target: None,
        loaded_moves: 0,
    };
    let config = LabConfig {
        cue: Cue::Responsive,
        ..Default::default()
    };
    let guidance = GoalGuidance {
        goal: p(0, 1),
        weight: 3,
        seen_open: false,
    };
    for seed in 0..32 {
        let mut reference = rng::seeded(seed);
        let ticket = reference.gen_range(0..12u64);
        let mut actual = rng::seeded(seed);
        let (decision, _, evaluated) = decide_with_goal_measured(
            &observation,
            &worker,
            &config,
            false,
            &[],
            Some(guidance),
            &mut actual,
        );
        assert_eq!(
            decision.selected,
            Some(if ticket < 9 { p(1, 1) } else { p(3, 1) })
        );
        assert_eq!(evaluated, 2);
        assert_eq!(actual.next_u64(), reference.next_u64());
    }
}

#[test]
fn guidance_keeps_retained_target_and_waits_for_congested_route() {
    for congested in [false, true] {
        let mut setup = fixtures::setup(&LabConfig {
            fixture: Fixture::Choice {
                side: Side::Left,
                pile: Pile::OldAccumulation,
            },
            ..Default::default()
        })
        .unwrap();
        if congested {
            setup.workers.extend([p(3, 3), p(3, 3)]);
        }
        let mut world = World::from_setup(LabConfig::default(), setup, 7).unwrap();
        world.workers[0].target = Some(p(1, 3));
        let mut actual = rng::seeded(7);
        let (decision, _, evaluated) = decide_with_goal_measured(
            &observe(&world, 0),
            &WorkerView::from(&world.workers[0]),
            &world.config,
            true,
            &[],
            Some(GoalGuidance {
                goal: p(7, 3),
                weight: 3,
                seen_open: false,
            }),
            &mut actual,
        );
        assert_eq!(
            decision.action,
            if congested {
                Action::Wait
            } else {
                Action::Move(p(3, 3))
            }
        );
        assert_eq!(decision.target, Some(p(1, 3)));
        assert_eq!(decision.selected, None);
        assert_eq!(evaluated, 0);
        assert_eq!(actual.next_u64(), rng::seeded(7).next_u64());
    }
}

#[test]
fn weight_one_preserves_legacy_decisions_stats_and_rng_continuation() {
    let world = World::new(
        LabConfig {
            fixture: Fixture::Choice {
                side: Side::Left,
                pile: Pile::OldAccumulation,
            },
            ..Default::default()
        },
        7,
    )
    .unwrap();
    for seed in 0..16 {
        let mut plain_rng = rng::seeded(seed);
        let mut goal_rng = rng::seeded(seed);
        let observation = observe(&world, 0);
        let worker = WorkerView::from(&world.workers[0]);
        let plain = decide_measured(
            &observation,
            &worker,
            &world.config,
            false,
            &[],
            &mut plain_rng,
        );
        let goal = decide_with_goal_measured(
            &observation,
            &worker,
            &world.config,
            false,
            &[],
            Some(GoalGuidance {
                goal: p(7, 3),
                weight: 1,
                seen_open: false,
            }),
            &mut goal_rng,
        );
        assert_eq!(plain, (goal.0, goal.1));
        assert_eq!(plain_rng.next_u64(), goal_rng.next_u64());
    }
}

#[test]
fn solid_goal_does_not_latch_until_next_local_open_observation() {
    let mut world = three_cells();
    install_goal_state(&mut world, &task(p(2, 1))).unwrap();
    world.step();
    let state = world.goal_state.as_ref().unwrap();
    assert_eq!(state.seen_open, [false]);
    assert_eq!(state.diagnostics.local_completion_checks, 1);
    assert_eq!(state.diagnostics.goal_weight_evaluations, 1);
    assert!(state.diagnostics.completion_observations.is_empty());
    assert_eq!(world.recording.events[0].action, Action::Dig(p(2, 1)));
    world.step();
    let state = world.goal_state.as_ref().unwrap();
    assert_eq!(state.seen_open, [true]);
    assert_eq!(
        state.diagnostics.completion_observations,
        [GoalObservation {
            tick: 1,
            opportunities_before: 1,
            worker: 0
        }]
    );
    assert_eq!(world.recording.events[1].action, Action::Move(p(0, 1)));
    world.step();
    assert_eq!(world.recording.events[2].action, Action::Dispose);
    let state = world.goal_state.as_ref().unwrap();
    assert_eq!(state.diagnostics.local_completion_checks, 3);
    assert_eq!(state.diagnostics.completion_observations.len(), 1);
}

#[test]
fn loaded_first_observation_latches_before_disposal() {
    let mut world = three_cells();
    install_goal_state(&mut world, &task(p(2, 1))).unwrap();
    assert_eq!(
        world.apply(0, Action::Dig(p(2, 1))).outcome,
        Outcome::Success
    );
    assert_eq!(
        world.apply(0, Action::Move(p(0, 1))).outcome,
        Outcome::Success
    );
    world.step();
    assert_eq!(world.recording.events[0].action, Action::Dispose);
    let state = world.goal_state.as_ref().unwrap();
    assert_eq!(state.seen_open, [true]);
    assert_eq!(
        state.diagnostics.completion_observations,
        [GoalObservation {
            tick: 0,
            opportunities_before: 0,
            worker: 0
        }]
    );
    assert_eq!(state.diagnostics.goal_weight_evaluations, 0);
}

#[test]
fn opened_goal_does_not_broadcast_to_remote_worker() {
    let mut setup = fixtures::corridor_setup(7, 1).unwrap();
    setup.workers.push(p(0, 1));
    let mut world = World::from_setup(LabConfig::default(), setup, 7).unwrap();
    install_goal_state(&mut world, &task(p(7, 1))).unwrap();
    world.step();
    assert_eq!(world.goal_state.as_ref().unwrap().seen_open, [false, false]);
    world.step();
    let state = world.goal_state.as_ref().unwrap();
    assert_eq!(state.seen_open, [true, false]);
    assert_eq!(state.diagnostics.local_completion_checks, 4);
    assert_eq!(state.diagnostics.completion_observations.len(), 1);
    assert_eq!(state.diagnostics.completion_observations[0].worker, 0);
    for _ in 0..3 {
        world.step();
    }
    assert!(!observe(&world, 0)
        .open
        .iter()
        .any(|cell| cell.pos == p(7, 1)));
    let state = world.goal_state.as_ref().unwrap();
    assert!(state.seen_open[0]);
    assert_eq!(state.diagnostics.completion_observations.len(), 1);
}

#[test]
fn goal_identity_and_private_latches_change_continuation_fingerprint() {
    let mut world = three_cells();
    let legacy = world.fingerprint();
    install_goal_state(&mut world, &task(p(2, 1))).unwrap();
    assert_ne!(world.fingerprint(), legacy);
    for change in 0..4 {
        let mut other = world.clone();
        let state = other.goal_state.as_mut().unwrap();
        match change {
            0 => state.task.goal = p(2, 0),
            1 => state.task.goal_weight += 1,
            2 => state.seen_open[0] = true,
            _ => state.seen_open.push(false),
        }
        assert_ne!(world.fingerprint(), other.fingerprint());
    }
    let mut setup = world.setup.clone();
    setup.workers.push(p(0, 1));
    let mut two_workers = World::from_setup(LabConfig::default(), setup, 7).unwrap();
    install_goal_state(&mut two_workers, &task(p(2, 1))).unwrap();
    let mut worker_zero = two_workers.clone();
    worker_zero.goal_state.as_mut().unwrap().seen_open[0] = true;
    let mut worker_one = two_workers.clone();
    worker_one.goal_state.as_mut().unwrap().seen_open[1] = true;
    assert_ne!(two_workers.fingerprint(), worker_zero.fingerprint());
    assert_ne!(two_workers.fingerprint(), worker_one.fingerprint());
    assert_ne!(worker_zero.fingerprint(), worker_one.fingerprint());
    let mut other = world.clone();
    let diagnostics = &mut other.goal_state.as_mut().unwrap().diagnostics;
    diagnostics.local_completion_checks += 1;
    diagnostics.goal_weight_evaluations += 1;
    diagnostics.completion_observations.push(GoalObservation {
        tick: 0,
        opportunities_before: 0,
        worker: 0,
    });
    assert_eq!(world.fingerprint(), other.fingerprint());
}

fn options(ticks: u32, sample_every: u32) -> RunOptions {
    RunOptions {
        ticks,
        sample_every,
    }
}

fn config(objective: AccessObjective, goal_weight: u32) -> AccessConfig {
    AccessConfig {
        lab: LabConfig::default(),
        task: AccessTask {
            goal: p(7, 12),
            objective,
            goal_weight,
        },
    }
}

#[test]
fn access_explore_is_byte_identical_to_legacy_episode() {
    for seed in [7, u64::MAX] {
        let config = config(AccessObjective::Explore, 3);
        let ordinary = run_episode(config.lab.clone(), seed, options(32, 7)).unwrap();
        let access = run_access_episode(config.clone(), seed, options(32, 7)).unwrap();
        assert_eq!(access.episode, ordinary);
        assert_eq!(
            serde_json::to_vec(&access.episode).unwrap(),
            serde_json::to_vec(&ordinary).unwrap()
        );
        assert_eq!(access.task_diagnostics, AccessDiagnostics::default());
        assert_eq!(access.config.lab, access.episode.config);
        assert_eq!(
            serde_json::from_slice::<AccessEpisode>(&serde_json::to_vec(&access).unwrap()).unwrap(),
            access
        );
    }
}

#[test]
fn weight_one_access_preserves_complete_physical_episode() {
    for seed in [7, u64::MAX] {
        let explore =
            run_access_episode(config(AccessObjective::Explore, 1), seed, options(64, 7)).unwrap();
        let known = run_access_episode(config(AccessObjective::KnownGoal, 1), seed, options(64, 7))
            .unwrap();
        assert_eq!(known.episode.events, explore.episode.events);
        assert_eq!(known.episode.choices, explore.episode.choices);
        assert_eq!(known.access, explore.access);
        // Only task-dependent fingerprints differ in the nested physical record.
        let mut physical = known.episode.clone();
        for (frame, plain) in physical.frames.iter_mut().zip(&explore.episode.frames) {
            frame.fingerprint = plain.fingerprint.clone();
        }
        assert_eq!(physical, explore.episode);
    }
}

#[test]
fn exact_access_milestone_records_action_clock_and_spoil_only() {
    let mut world = three_cells();
    world.tick = 19;
    let (episode, access, diagnostics) = run_world(
        world,
        7,
        options(1, 7),
        Some(AccessObserver {
            goal: p(2, 1),
            first: None,
        }),
    )
    .unwrap();
    assert_eq!(
        access.unwrap(),
        AccessSummary {
            structurally_accessible: true,
            first_access: Some(AccessMilestone {
                tick: 19,
                opportunity: 1,
                worker: 0,
                goal: p(2, 1),
                digs: 1,
                disposed: 0,
                carried: 1,
                loose: 0,
                exit_distance: 2
            }),
            final_exit_distance: Some(2),
            observed_opportunities: 1,
            deadline_censored: false,
        }
    );
    assert_eq!(episode.events[0].action, Action::Dig(p(2, 1)));
    assert_eq!(episode.deliveries.len(), 1);
    assert_eq!(
        episode.final_summary.inventory,
        Inventory {
            initial: 0,
            excavated: 1,
            carried: 1,
            loose: 0,
            disposed: 0
        }
    );
    assert_eq!(diagnostics, AccessDiagnostics::default());
}

fn record_observed(
    world: &mut World,
    observer: &mut AccessObserver,
    action: Action,
    distance: Option<u32>,
) -> ActionEvent {
    let event = world.apply(0, action);
    let mut recording = std::mem::take(&mut world.recording);
    recording.record(&event, world, distance);
    recording.ensure_exit_field(world);
    observer.observe(world, &recording, &event);
    world.recording = recording;
    world.check_invariants().unwrap();
    event
}

#[test]
fn failed_digs_moves_and_waits_do_not_fabricate_access() {
    let mut world = three_cells();
    let mut observer = AccessObserver {
        goal: p(2, 1),
        first: None,
    };
    for action in [
        Action::Dig(p(2, 0)),
        Action::Dig(p(1, 0)),
        Action::Move(p(2, 1)),
        Action::Wait,
    ] {
        record_observed(&mut world, &mut observer, action, None);
        assert!(observer.first.is_none());
    }
    record_observed(&mut world, &mut observer, Action::Move(p(0, 1)), None);
    record_observed(&mut world, &mut observer, Action::Dig(p(2, 1)), None);
    assert!(observer.first.is_none());
    assert!(!observer.summary(&world).structurally_accessible);
    assert_eq!(observer.summary(&world).observed_opportunities, 6);
}

#[test]
fn full_hands_and_other_excavation_do_not_complete_goal() {
    let mut setup = three_cells().setup;
    setup.diggable.push(p(1, 0));
    let mut world = World::from_setup(LabConfig::default(), setup, 7).unwrap();
    let mut observer = AccessObserver {
        goal: p(2, 1),
        first: None,
    };
    assert_eq!(
        record_observed(&mut world, &mut observer, Action::Dig(p(1, 0)), Some(2)).outcome,
        Outcome::Success
    );
    assert!(matches!(
        record_observed(&mut world, &mut observer, Action::Dig(p(2, 1)), None).outcome,
        Outcome::Blocked { .. }
    ));
    assert_eq!(
        observer.summary(&world),
        AccessSummary {
            structurally_accessible: false,
            first_access: None,
            final_exit_distance: None,
            observed_opportunities: 2,
            deadline_censored: true
        }
    );
}

#[test]
fn access_first_distance_survives_later_shortcut() {
    let mut setup = three_cells().setup;
    setup.width = 4;
    setup.height = 4;
    setup.exit = p(0, 0);
    setup.open = vec![p(0, 0), p(0, 1), p(0, 2), p(1, 2), p(2, 2)];
    setup.diggable = vec![p(2, 1), p(1, 1)];
    setup.workers = vec![p(2, 2)];
    let mut world = World::from_setup(LabConfig::default(), setup, 7).unwrap();
    let mut observer = AccessObserver {
        goal: p(2, 1),
        first: None,
    };
    world.workers[0].target = Some(p(2, 1));
    world.step_observed(Some(&mut observer));
    let first = observer.first.clone().unwrap();
    assert_eq!(first.exit_distance, 5);
    record_observed(&mut world, &mut observer, Action::Drop, None);
    record_observed(&mut world, &mut observer, Action::Move(p(2, 1)), None);
    world.workers[0].target = Some(p(1, 1));
    world.step_observed(Some(&mut observer));
    let (_, summary, _) = run_world(world, 7, options(0, 7), Some(observer)).unwrap();
    let summary = summary.unwrap();
    assert_eq!(summary.first_access, Some(first));
    assert_eq!(summary.final_exit_distance, Some(3));
    assert_eq!(summary.observed_opportunities, 4);
}

#[test]
fn completion_does_not_stop_budget_and_last_action_is_success() {
    for objective in [AccessObjective::Explore, AccessObjective::KnownGoal] {
        for ticks in [1, 8] {
            let mut world = three_cells();
            let mut task = task(p(2, 1));
            task.objective = objective.clone();
            install_goal_state(&mut world, &task).unwrap();
            let (episode, summary, diagnostics) = run_world(
                world,
                7,
                options(ticks, 7),
                Some(AccessObserver {
                    goal: task.goal,
                    first: None,
                }),
            )
            .unwrap();
            let summary = summary.unwrap();
            assert!(summary.structurally_accessible);
            assert!(!summary.deadline_censored);
            assert_eq!(summary.first_access.unwrap().opportunity, 1);
            assert_eq!(summary.observed_opportunities, u64::from(ticks));
            assert_eq!(episode.completed_ticks, ticks);
            assert_eq!(episode.stop_reason, "tick_budget_exhausted");
            assert_eq!(episode.frames.last().unwrap().tick, u64::from(ticks));
            if objective == AccessObjective::KnownGoal && ticks > 1 {
                assert_eq!(
                    diagnostics.completion_observations,
                    [GoalObservation {
                        tick: 1,
                        opportunities_before: 1,
                        worker: 0
                    }]
                );
            }
        }
    }
}

#[test]
fn zero_ticks_and_unreached_deadline_remain_censored() {
    for objective in [AccessObjective::Explore, AccessObjective::KnownGoal] {
        for ticks in [0, 1] {
            let access =
                run_access_episode(config(objective.clone(), 3), 7, options(ticks, 7)).unwrap();
            assert_eq!(
                access.access,
                AccessSummary {
                    structurally_accessible: false,
                    first_access: None,
                    final_exit_distance: None,
                    observed_opportunities: u64::from(ticks) * 8,
                    deadline_censored: true
                }
            );
            if ticks == 0 {
                assert_eq!(access.episode.frames.len(), 1);
                assert_eq!(access.episode.series.len(), 1);
                assert_eq!(access.task_diagnostics, AccessDiagnostics::default());
            }
        }
    }
}

#[test]
fn task_sampling_preserves_actions_latches_and_final_fingerprints() {
    for objective in [AccessObjective::Explore, AccessObjective::KnownGoal] {
        let mut config = config(objective.clone(), 3);
        config.task.goal = p(3, 12);
        let dense = run_access_episode(config.clone(), 7, options(64, 1)).unwrap();
        let sparse = run_access_episode(config.clone(), 7, options(64, 7)).unwrap();
        assert_eq!(dense.episode.events, sparse.episode.events);
        assert_eq!(dense.episode.choices, sparse.episode.choices);
        assert_eq!(dense.episode.final_summary, sparse.episode.final_summary);
        assert_eq!(dense.episode.frames.last(), sparse.episode.frames.last());
        assert_eq!(dense.access, sparse.access);
        assert_eq!(dense.task_diagnostics, sparse.task_diagnostics);
        assert_eq!(sparse.episode.frames.last().unwrap().tick, 64);
        assert_eq!(sparse.episode.series.last().unwrap().tick, 64);
        assert_eq!(sparse.config.lab, sparse.episode.config);
        let diagnostics = &sparse.task_diagnostics;
        assert!(diagnostics.completion_observations.len() <= sparse.episode.worker_work.len());
        let mut workers = std::collections::BTreeSet::new();
        for observation in &diagnostics.completion_observations {
            assert!(workers.insert(observation.worker));
            let decision = &sparse.episode.events[observation.opportunities_before as usize];
            assert_eq!(
                (decision.tick, decision.worker),
                (observation.tick, observation.worker)
            );
        }
        if objective == AccessObjective::Explore {
            assert_eq!(*diagnostics, AccessDiagnostics::default());
        } else {
            assert!(!diagnostics.completion_observations.is_empty());
            assert_eq!(diagnostics.local_completion_checks, 512);
            assert!(diagnostics.goal_weight_evaluations > 0);
        }
    }
}

#[test]
fn public_access_rejects_non_growing_fixture_before_invalid_task() {
    for fixture in [
        Fixture::Corridor {
            length: 7,
            workers: 1,
        },
        Fixture::Choice {
            side: Side::Left,
            pile: Pile::OldAccumulation,
        },
    ] {
        let mut config = config(AccessObjective::KnownGoal, 0);
        config.lab.fixture = fixture;
        config.task.goal = p(u32::MAX, u32::MAX);
        let errors = run_access_episode(config, 7, options(0, 1)).unwrap_err();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "lab.fixture");
    }
}

#[test]
fn access_public_boundary_rejects_invalid_tasks_and_requested_budgets() {
    for (goal, weight) in [
        (p(u32::MAX, 12), 3),
        (p(0, 12), 3),
        (p(41, 12), 3),
        (p(7, 12), 0),
        (p(7, 12), u32::MAX),
    ] {
        let mut config = config(AccessObjective::KnownGoal, weight);
        config.task.goal = goal;
        assert!(run_access_episode(config, 7, options(0, 1)).is_err());
    }
    for (options, field) in [
        (options(0, 0), "sample_every"),
        (options(125_001, 1000), "ticks"),
        (options(100_000, 1), "sample_every"),
    ] {
        assert!(
            run_access_episode(config(AccessObjective::Explore, 3), 7, options)
                .unwrap_err()
                .iter()
                .any(|error| error.field == field)
        );
    }
    let mut world = three_cells();
    world.tick = u64::MAX - 1;
    assert!(run_world(
        world,
        7,
        options(2, 1),
        Some(AccessObserver {
            goal: p(2, 1),
            first: None
        })
    )
    .unwrap_err()
    .iter()
    .any(|error| error.field == "ticks"));
}
