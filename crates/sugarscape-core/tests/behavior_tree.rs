use sugarscape_core::minds::behavior_tree::{lab, records::RunOptions, runner, state::*};
use sugarscape_core::{geometry::Pos, world::World};

fn stable(controller: Controller, quota: u32) -> LabConfig {
    LabConfig {
        controller,
        scenario: Scenario::Stable,
        quota,
        mirrored: false,
    }
}
#[test]
fn behavior_tree_initial_prior_contains_levels_not_c_capacity() {
    let w = World::new(lab::rig_config(stable(Controller::GuardedTree, 20)), 7).unwrap();
    let site = w.torus.index(Pos::new(4, 9)) as u32;
    assert_eq!(w.agent(1).unwrap().memory.sites[&site].levels()[0], 24.0);
}
#[test]
fn behavior_tree_complete_episode_has_all_sixty_five_frames() {
    let r = runner::run_episode(
        stable(Controller::GuardedTree, 20),
        7,
        RunOptions {
            diagnostics: true,
            controller_timing: false,
        },
    )
    .unwrap();
    assert_eq!(
        r.frames.iter().map(|f| f.tick).collect::<Vec<_>>(),
        (0..=64).collect::<Vec<_>>()
    );
}
#[test]
fn behavior_tree_strict_rig_rejects_unregistered_quota() {
    let mut c = lab::rig_config(stable(Controller::GuardedTree, 20));
    c.behavior_tree_lab.as_mut().unwrap().quota = 21;
    assert!(c
        .validate()
        .unwrap_err()
        .iter()
        .any(|e| e.field == "behavior_tree_lab.quota"));
}
#[test]
fn behavior_tree_sink_failure_retains_completed_frame() {
    let mut seen = vec![];
    let failure = runner::run_episode_to(
        stable(Controller::GuardedTree, 20),
        7,
        RunOptions {
            diagnostics: true,
            controller_timing: false,
        },
        |f| {
            seen.push(f.tick);
            if f.tick == 3 {
                Err("sink unavailable".into())
            } else {
                Ok(())
            }
        },
    )
    .unwrap_err();
    let partial = failure.partial.unwrap();
    assert_eq!(partial.completed_ticks, 3);
    assert_eq!(
        partial.frames.iter().map(|f| f.tick).collect::<Vec<_>>(),
        seen
    );
}
#[test]
fn behavior_tree_all_construction_conditions_and_matched_pairs() {
    let conditions = lab::conditions();
    assert_eq!(conditions.len(), 96);
    let ids = conditions.iter().map(lab::condition_id).collect::<Vec<_>>();
    assert!(ids.windows(2).all(|w| w[0] < w[1]));
    // Opt-in engineering evidence uses create_new so an earlier source record
    // can never be silently replaced. These are exclusively construction seeds.
    let evidence = std::env::var_os("SUGARSCAPE_BT_CONSTRUCTION_DIR").map(std::path::PathBuf::from);
    if let Some(dir) = &evidence {
        std::fs::create_dir_all(dir).unwrap();
    }
    let mut pairs = vec![];
    for seed in [7, 8] {
        for condition in &conditions {
            let r = runner::run_episode(
                condition.clone(),
                seed,
                RunOptions {
                    diagnostics: true,
                    controller_timing: false,
                },
            )
            .unwrap();
            assert_eq!(r.frames.len(), 65);
            if let Some(dir) = &evidence {
                let file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(dir.join(format!("{}-seed{seed}.json", lab::condition_id(condition))))
                    .unwrap();
                serde_json::to_writer(file, &r).unwrap();
            }
            assert!(
                r.errors.is_empty(),
                "{} {seed}: {:?}",
                lab::condition_id(condition),
                r.errors
            );
            for f in &r.frames {
                let rng: sugarscape_core::rng::SimRng =
                    serde_json::from_str(&f.rng_state_json).unwrap();
                assert_eq!(sugarscape_core::rng::state_json(&rng), f.rng_state_json);
                assert_eq!(
                    serde_json::from_str::<sugarscape_core::minds::behavior_tree::records::Frame>(
                        &serde_json::to_string(f).unwrap()
                    )
                    .unwrap(),
                    *f
                );
            }
            if condition.controller == Controller::GuardedTree {
                let mut matched = condition.clone();
                matched.controller = Controller::MatchedFsm;
                let other = runner::run_episode(
                    matched,
                    seed,
                    RunOptions {
                        diagnostics: true,
                        controller_timing: false,
                    },
                )
                .unwrap();
                for (a, b) in r.frames.iter().zip(&other.frames) {
                    assert_eq!(
                        runner::physical_projection(a),
                        runner::physical_projection(b),
                        "{} {seed} tick {}",
                        lab::condition_id(condition),
                        a.tick
                    );
                    assert_eq!(a.rng_state_json, b.rng_state_json);
                }
                pairs.push(serde_json::json!({"guarded":lab::condition_id(condition),"seed":seed,"frames":65,"physical_task_equal":true,"canonical_rng_equal":true}));
            }
        }
    }
    assert_eq!(pairs.len(), 32);
    if let Some(dir) = &evidence {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(dir.join("matched-pairs.json"))
            .unwrap();
        serde_json::to_writer_pretty(file, &pairs).unwrap();
    }
}
#[test]
fn behavior_tree_default_profile_cannot_erase_required_rig_setting() {
    let mut c = lab::rig_config(stable(Controller::GuardedTree, 20));
    c.behavior_tree = Settings::default();
    assert!(
        c.validate().is_err(),
        "omitted default settings must still differ from guarded_rate"
    );
}
#[test]
fn behavior_tree_task_profile_requires_checked_lab() {
    let mut c = sugarscape_core::config::Config::default();
    c.behavior_tree.profile = Profile::GuardedRate;
    assert!(c.validate().is_err());
}

#[test]
fn behavior_tree_every_task_semantic_field_changes_fingerprint_individually() {
    use serde_json::{json, Value};
    use sugarscape_core::minds::behavior_tree::runtime::Status;
    let mut base = World::new(lab::rig_config(stable(Controller::GuardedTree, 40)), 7).unwrap();
    let task = &mut base.behavior_tree_lab.as_mut().unwrap().task;
    task.first_completion = Some(9);
    task.target = Some(62);
    task.failed_until.extend([(61, 5), (62, 6)]);
    task.tree.cursors.insert(0, 1);
    task.tree.statuses.insert(1, Status::Running);
    task.tree.running_leaves.insert(4);
    task.tree.deferred = Some(3);
    task.task_plan = Some(TaskPlan {
        goal: 20.0,
        steps: vec![(Pos::new(4, 9), 24.0), (Pos::new(7, 5), 24.0)],
    });
    let value = serde_json::to_value(task).unwrap();
    let changes: &[(&str, Value)] = &[
        ("/quota", json!(20)),
        ("/gross", json!(1.0)),
        ("/first_completion", json!(10)),
        ("/first_completion", Value::Null),
        ("/target", json!(61)),
        ("/target", Value::Null),
        ("/failed_until", json!({})),
        ("/failed_until", json!({"60":5,"62":6})),
        ("/failed_until/61", json!(7)),
        ("/failed_until/62", json!(7)),
        ("/tree/cursors", json!({})),
        ("/tree/cursors", json!({"1":1})),
        ("/tree/cursors/0", json!(2)),
        ("/tree/statuses", json!({})),
        ("/tree/statuses", json!({"2":"running"})),
        ("/tree/statuses/1", json!("success")),
        ("/tree/running_leaves", json!([])),
        ("/tree/running_leaves/0", json!(5)),
        ("/tree/deferred", Value::Null),
        ("/tree/deferred", json!(4)),
        ("/fsm/phase", json!("moving")),
        ("/task_plan", Value::Null),
        ("/task_plan/goal", json!(21.0)),
        ("/task_plan/steps", json!([])),
        ("/task_plan/steps/0/0/x", json!(5)),
        ("/task_plan/steps/0/0/y", json!(8)),
        ("/task_plan/steps/0/1", json!(25.0)),
        (
            "/task_plan/steps",
            json!([[{"x":7,"y":5},24.0],[{"x":4,"y":9},24.0]]),
        ),
    ];
    for (pointer, next) in changes {
        let mut changed = value.clone();
        *changed.pointer_mut(pointer).unwrap() = next.clone();
        let mut altered = base.clone();
        altered.behavior_tree_lab.as_mut().unwrap().task = serde_json::from_value(changed).unwrap();
        assert_ne!(
            base.fingerprint(),
            altered.fingerprint(),
            "{pointer} -> {next}"
        );
    }
}

#[test]
fn behavior_tree_semantic_actor_plans_and_runtime_fields_change_hash_individually() {
    use sugarscape_core::agent::GoapPlan;
    type Mutation = (&'static str, fn(&mut World));
    let mut base = World::new(lab::rig_config(stable(Controller::LegacyGoap, 40)), 7).unwrap();
    let a = base.agent_mut(1).unwrap();
    a.plan.target = Some(Pos::new(4, 9));
    a.plan.path = vec![Pos::new(3, 5), Pos::new(4, 5)];
    a.goap_plan = Some(GoapPlan {
        steps: vec![(Pos::new(4, 9), 24.0), (Pos::new(7, 5), 24.0)],
        goal: 40.0,
        gathers: 24.0,
    });
    let changes: &[Mutation] = &[
        ("actor presence", |w| {
            let p = w.agent(1).unwrap().pos;
            w.remove_agent(p.x, p.y).unwrap();
        }),
        ("actor id", |w| w.agent_mut(1).unwrap().id += 1),
        ("actor x", |w| w.agent_mut(1).unwrap().pos.x += 1),
        ("actor y", |w| w.agent_mut(1).unwrap().pos.y += 1),
        ("actor holdings", |w| {
            w.agent_mut(1).unwrap().holdings[0] += 1.0
        }),
        ("actor metabolism", |w| {
            w.agent_mut(1).unwrap().metabolism[0] += 1
        }),
        ("actor vision", |w| w.agent_mut(1).unwrap().vision += 1),
        ("actor remembers", |w| {
            w.agent_mut(1).unwrap().remembers = false
        }),
        ("actor age", |w| w.agent_mut(1).unwrap().age += 1),
        ("actor max age", |w| w.agent_mut(1).unwrap().max_age += 1),
        ("memory presence", |w| {
            w.agent_mut(1).unwrap().memory.sites.clear()
        }),
        ("memory site", |w| {
            let a = w.agent_mut(1).unwrap();
            let seen = a.memory.sites.remove(&103).unwrap();
            a.memory.sites.insert(102, seen);
        }),
        ("memory tick", |w| {
            w.agent_mut(1)
                .unwrap()
                .memory
                .sites
                .get_mut(&103)
                .unwrap()
                .tick += 1
        }),
        ("memory levels", |w| {
            w.agent_mut(1).unwrap().memory.sites.insert(
                103,
                sugarscape_core::minds::memory::Seen::new(&[25.0], &[24.0], 0),
            );
        }),
        ("memory most", |w| {
            w.agent_mut(1).unwrap().memory.sites.insert(
                103,
                sugarscape_core::minds::memory::Seen::new(&[24.0], &[25.0], 0),
            );
        }),
        ("cell food", |w| w.sites[103].resource[0] += 1.0),
        ("cell capacity", |w| w.sites[103].capacity[0] += 1.0),
        ("motion target presence", |w| {
            w.agent_mut(1).unwrap().plan.target = None
        }),
        ("motion target x", |w| {
            w.agent_mut(1).unwrap().plan.target.as_mut().unwrap().x += 1
        }),
        ("motion target y", |w| {
            w.agent_mut(1).unwrap().plan.target.as_mut().unwrap().y += 1
        }),
        ("motion path presence", |w| {
            w.agent_mut(1).unwrap().plan.path.clear()
        }),
        ("motion path x", |w| {
            w.agent_mut(1).unwrap().plan.path[0].x += 1
        }),
        ("motion path y", |w| {
            w.agent_mut(1).unwrap().plan.path[0].y += 1
        }),
        ("motion path order", |w| {
            w.agent_mut(1).unwrap().plan.path.reverse()
        }),
        ("motion walked", |w| {
            w.agent_mut(1).unwrap().plan.walked = true
        }),
        ("legacy presence", |w| {
            w.agent_mut(1).unwrap().goap_plan = None
        }),
        ("legacy goal", |w| {
            w.agent_mut(1).unwrap().goap_plan.as_mut().unwrap().goal += 1.0
        }),
        ("legacy gathers", |w| {
            w.agent_mut(1).unwrap().goap_plan.as_mut().unwrap().gathers += 1.0
        }),
        ("legacy steps presence", |w| {
            w.agent_mut(1)
                .unwrap()
                .goap_plan
                .as_mut()
                .unwrap()
                .steps
                .clear()
        }),
        ("legacy steps order", |w| {
            w.agent_mut(1)
                .unwrap()
                .goap_plan
                .as_mut()
                .unwrap()
                .steps
                .reverse()
        }),
        ("legacy x", |w| {
            w.agent_mut(1).unwrap().goap_plan.as_mut().unwrap().steps[0]
                .0
                .x += 1
        }),
        ("legacy y", |w| {
            w.agent_mut(1).unwrap().goap_plan.as_mut().unwrap().steps[0]
                .0
                .y += 1
        }),
        ("legacy value", |w| {
            w.agent_mut(1).unwrap().goap_plan.as_mut().unwrap().steps[0].1 += 1.0
        }),
        ("living ticks", |w| {
            w.behavior_tree_lab.as_mut().unwrap().living_ticks += 1
        }),
        ("external added", |w| {
            w.behavior_tree_lab.as_mut().unwrap().external_added += 1.0
        }),
        ("external removed", |w| {
            w.behavior_tree_lab.as_mut().unwrap().external_removed += 1.0
        }),
        ("consumed", |w| {
            w.behavior_tree_lab.as_mut().unwrap().consumed += 1.0
        }),
        ("death loss", |w| {
            w.behavior_tree_lab.as_mut().unwrap().death_loss += 1.0
        }),
        ("fatal error", |w| {
            w.behavior_tree_lab.as_mut().unwrap().fatal_error = Some("blocked".into())
        }),
        ("profile", |w| {
            w.config.behavior_tree.profile = Profile::UtilityLeaf
        }),
        ("visits", |w| w.config.behavior_tree.visits = 63),
        ("controller", |w| {
            w.config.behavior_tree_lab.as_mut().unwrap().controller = Controller::TaskGoap
        }),
        ("scenario", |w| {
            w.config.behavior_tree_lab.as_mut().unwrap().scenario = Scenario::BetterAlternative
        }),
        ("quota", |w| {
            w.config.behavior_tree_lab.as_mut().unwrap().quota = 20
        }),
        ("mirror", |w| {
            w.config.behavior_tree_lab.as_mut().unwrap().mirrored = true
        }),
    ];
    for (name, mutate) in changes {
        let mut altered = base.clone();
        mutate(&mut altered);
        assert_ne!(base.fingerprint(), altered.fingerprint(), "{name}");
    }
}

#[test]
fn behavior_tree_diagnostics_and_timing_are_neutral_at_every_tick() {
    for condition in lab::conditions() {
        for seed in [7, 8] {
            let plain = runner::run_episode(
                condition.clone(),
                seed,
                RunOptions {
                    diagnostics: true,
                    controller_timing: false,
                },
            )
            .unwrap();
            for (diagnostics, controller_timing) in [(false, false), (true, true), (false, true)] {
                let measured = runner::run_episode(
                    condition.clone(),
                    seed,
                    RunOptions {
                        diagnostics,
                        controller_timing,
                    },
                )
                .unwrap();
                for (a, b) in plain.frames.iter().zip(&measured.frames) {
                    assert_eq!(a.fingerprint, b.fingerprint);
                    assert_eq!(a.rng_state_json, b.rng_state_json);
                    // Receipts are diagnostics; physical and task state remain exact when disabled.
                    let mut aa = runner::physical_projection(a);
                    aa.as_object_mut().unwrap().remove("receipt");
                    let mut bb = runner::physical_projection(b);
                    bb.as_object_mut().unwrap().remove("receipt");
                    assert_eq!(aa, bb);
                    assert_eq!(a.task, b.task);
                    assert_eq!(a.legacy_plan, b.legacy_plan);
                }
            }
        }
    }
}

#[test]
fn behavior_tree_snapshot_is_checked_and_observational_only() {
    let w = World::new(lab::rig_config(stable(Controller::GuardedTree, 20)), 7).unwrap();
    let before = w.clone();
    let frame = runner::snapshot(&w).unwrap();
    assert_eq!(frame.tick, 0);
    assert_eq!(frame.actor.as_ref().unwrap().holdings, 16.0);
    assert_eq!(frame, runner::snapshot(&before).unwrap());
    assert_eq!(w.fingerprint(), before.fingerprint());
    let mut after = w.clone();
    let mut expected = before;
    after.run(64);
    expected.run(64);
    assert_eq!(
        runner::snapshot(&after).unwrap(),
        runner::snapshot(&expected).unwrap()
    );
    let default = World::new(sugarscape_core::config::Config::default(), 7).unwrap();
    assert_eq!(
        runner::snapshot(&default).unwrap_err(),
        "behavior-tree snapshot requires an enabled lab"
    );
}

#[test]
fn behavior_tree_native_world_and_episode_agree_on_every_field_each_tick() {
    for seed in [7, 8] {
        for condition in lab::conditions() {
            let record = runner::run_episode(
                condition.clone(),
                seed,
                RunOptions {
                    diagnostics: true,
                    controller_timing: false,
                },
            )
            .unwrap();
            let mut native = World::new(lab::rig_config(condition), seed).unwrap();
            native.behavior_tree_lab.as_mut().unwrap().diagnostics = true;
            for frame in &record.frames {
                assert_eq!(runner::snapshot(&native).unwrap(), *frame);
                native.step();
            }
        }
    }
}
