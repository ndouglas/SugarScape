use super::{lab, state::*};
use crate::world::World;
#[test]
fn behavior_tree_rng_state_is_semantic_only_in_lab() {
    use rand::RngCore;
    let mut w = World::new(
        lab::rig_config(LabConfig {
            controller: Controller::GuardedTree,
            scenario: Scenario::Stable,
            quota: 20,
            mirrored: false,
        }),
        7,
    )
    .unwrap();
    let hash = w.fingerprint();
    w.rng.next_u64();
    assert_ne!(w.fingerprint(), hash);
}
#[test]
fn behavior_tree_checkpoint_retains_future_and_diagnostics_do_not_change_it() {
    use super::{records::RunOptions, runner};
    for condition in lab::conditions() {
        let mut w = World::new(lab::rig_config(condition.clone()), 8).unwrap();
        w.run(2);
        let mut checkpoint = w.clone();
        w.run(62);
        checkpoint.run(62);
        assert_eq!(w.fingerprint(), checkpoint.fingerprint());
        assert_eq!(
            crate::rng::state_json(&w.rng),
            crate::rng::state_json(&checkpoint.rng)
        );
        let a = runner::run_episode(
            condition.clone(),
            8,
            RunOptions {
                diagnostics: true,
                controller_timing: false,
            },
        )
        .unwrap();
        let b = runner::run_episode(
            condition,
            8,
            RunOptions {
                diagnostics: false,
                controller_timing: true,
            },
        )
        .unwrap();
        for (a, b) in a.frames.iter().zip(&b.frames) {
            assert_eq!(a.fingerprint, b.fingerprint);
            assert_eq!(a.rng_state_json, b.rng_state_json);
        }
    }
}
#[test]
fn behavior_tree_semantic_motion_and_legacy_plan_affect_only_enabled_hashes() {
    use crate::{agent::GoapPlan, geometry::Pos};
    for condition in lab::conditions() {
        let w = World::new(lab::rig_config(condition), 7).unwrap();
        let mut a = w.clone();
        a.agent_mut(1).unwrap().plan.target = Some(Pos::new(4, 9));
        assert_ne!(w.fingerprint(), a.fingerprint());
        let mut a = w.clone();
        a.agent_mut(1).unwrap().goap_plan = Some(GoapPlan {
            steps: vec![(Pos::new(4, 9), 24.0)],
            goal: 20.0,
            gathers: 24.0,
        });
        assert_ne!(w.fingerprint(), a.fingerprint());
        let mut a = w.clone();
        a.behavior_tree_lab.as_mut().unwrap().fatal_error = Some("blocked".into());
        assert_ne!(w.fingerprint(), a.fingerprint());
    }
}
#[test]
fn behavior_tree_wire_records_reject_unknown_outer_fields() {
    use super::{records::*, runner};
    let r = runner::run_episode(
        lab::conditions()[0].clone(),
        7,
        RunOptions {
            diagnostics: true,
            controller_timing: false,
        },
    )
    .unwrap();
    let mut v = serde_json::to_value(&r.frames[0]).unwrap();
    v["unexpected"] = true.into();
    assert!(serde_json::from_value::<Frame>(v).is_err());
    let mut v = serde_json::to_value(&r).unwrap();
    v["unexpected"] = true.into();
    assert!(serde_json::from_value::<EpisodeRecord>(v).is_err());
}
#[test]
fn behavior_tree_controller_work_resets_and_hold_and_death_have_no_stale_diagnostics() {
    use super::{records::RunOptions, runner};
    let r = runner::run_episode(
        LabConfig {
            controller: Controller::GuardedTree,
            scenario: Scenario::Stable,
            quota: 20,
            mirrored: false,
        },
        7,
        RunOptions {
            diagnostics: true,
            controller_timing: true,
        },
    )
    .unwrap();
    let first = &r.frames[1];
    let second = &r.frames[2];
    assert_eq!(
        first.work.as_ref().unwrap().candidate_evaluations,
        first.observation.as_ref().unwrap().candidates.len() as u64 + 3
    );
    assert_eq!(first.work.as_ref().unwrap().target_selections, 1);
    assert_eq!(
        second.work.as_ref().unwrap().candidate_evaluations,
        second.observation.as_ref().unwrap().candidates.len() as u64
    );
    assert_eq!(second.work.as_ref().unwrap().target_selections, 0);
    let completion = r.first_completion.unwrap();
    for f in r.frames.iter().skip(1) {
        if f.tick <= completion {
            #[cfg(not(target_arch = "wasm32"))]
            assert!(f
                .controller_seconds
                .is_some_and(|s| s.is_finite() && s >= 0.0));
        } else {
            assert!(f.controller_seconds.is_none());
            assert!(f.observation.is_none());
            if let Some(receipt) = &f.receipt {
                assert_eq!(receipt.gathered, 0.0);
                assert_eq!(f.work, Some(super::WorkCounters::default()));
            } else {
                assert!(f.work.is_none());
            }
        }
    }
    assert!(r.frames.last().unwrap().actor.is_none());
    assert_eq!(r.frames.last().unwrap().consumed, 16.0 + r.gross_gathered);
}

#[test]
fn behavior_tree_model_checkpoints_restore_exact_future_at_event_and_terminal_boundaries() {
    use crate::model::ModelWorld;
    fn world(m: &ModelWorld) -> &World {
        match m {
            ModelWorld::Sugarscape(w) => w,
            _ => unreachable!(),
        }
    }
    for condition in lab::conditions()
        .into_iter()
        .filter(|c| c.quota == 20 && !c.mirrored)
    {
        let mut straight = ModelWorld::Sugarscape(Box::new(
            World::new(lab::rig_config(condition.clone()), 8).unwrap(),
        ));
        for tick in [0, 2, 3, 6, 7, 20, 40, 64] {
            let advance = tick - straight.model().tick();
            straight.model_mut().run(advance as u32);
            let expected = world(&straight).clone();
            let checkpoint = straight.checkpoint().unwrap();
            straight.model_mut().run(5);
            straight.restore(&checkpoint).unwrap();
            assert_eq!(
                world(&straight).behavior_tree_lab,
                expected.behavior_tree_lab
            );
            let mut a = world(&straight).clone();
            let mut b = expected;
            for _ in tick..64 {
                a.step();
                b.step();
                assert_eq!(a.fingerprint(), b.fingerprint());
                assert_eq!(
                    crate::rng::state_json(&a.rng),
                    crate::rng::state_json(&b.rng)
                );
                assert_eq!(a.behavior_tree_lab, b.behavior_tree_lab);
            }
        }
    }
}
#[test]
fn behavior_tree_after_expiry_entries_match_standalone_adapters() {
    use super::{forage, fsm, policy};
    for controller in [
        Controller::GuardedTree,
        Controller::UnguardedTree,
        Controller::MatchedFsm,
        Controller::TaskGoap,
    ] {
        let mut a = World::new(
            lab::rig_config(LabConfig {
                controller,
                scenario: Scenario::Stable,
                quota: 20,
                mirrored: false,
            }),
            7,
        )
        .unwrap();
        a.tick = 3;
        a.bt_work = Some(super::WorkCounters::default());
        let mut b = a.clone();
        let mut sa = TaskState::new(20);
        sa.failed_until.insert(62, 4);
        sa.failed_until.insert(103, 5);
        let mut sb = sa.clone();
        policy::expire_failed(&mut sb, 4);
        let (ta, tb) = match controller {
            Controller::GuardedTree | Controller::UnguardedTree => (
                forage::act_routine(
                    &mut a,
                    1,
                    &mut sa,
                    controller == Controller::GuardedTree,
                    64,
                ),
                forage::act_routine_after_expiry(
                    &mut b,
                    1,
                    &mut sb,
                    controller == Controller::GuardedTree,
                    64,
                ),
            ),
            Controller::MatchedFsm => (
                fsm::act_fsm(&mut a, 1, &mut sa),
                fsm::act_fsm_after_expiry(&mut b, 1, &mut sb),
            ),
            Controller::TaskGoap => (
                forage::act_task_goap(&mut a, 1, &mut sa),
                forage::act_task_goap_after_expiry(&mut b, 1, &mut sb),
            ),
            _ => unreachable!(),
        };
        assert_eq!(ta, tb);
        assert_eq!(sa, sb);
        assert_eq!(a.bt_work, b.bt_work);
        assert_eq!(
            crate::rng::state_json(&a.rng),
            crate::rng::state_json(&b.rng)
        );
        assert_eq!(a.fingerprint(), b.fingerprint());
    }
}
