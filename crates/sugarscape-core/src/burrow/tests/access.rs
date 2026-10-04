use super::super::{
    access::{install_goal_state, validate_task, GoalGuidance},
    controller::{decide_measured, decide_with_goal_measured, goal_factor, select_ticket},
    observation::observe,
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
