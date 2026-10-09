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
