use super::{
    forage::{act_routine, act_task_goap, observe},
    fsm::act_fsm,
    policy::{expire_failed, note_failure, select_target},
    runtime::Status,
    state::{Controller, Profile, Settings, TaskState},
    WorkCounters,
};
use crate::{
    config::{DecisionRule, MoveMode, Movement},
    geometry::Pos,
    testkit::*,
};

fn policy_world() -> (crate::world::World, u64) {
    let mut c = blank_config(11, 11);
    c.movement = Movement {
        mode: MoveMode::Walk,
        speed: 1,
    };
    c.decision.travel = 1.0;
    c.decision.crowding = 0.0;
    let mut w = crate::world::World::new(c, 7).unwrap();
    let id = spawn(&mut w, 2, 5);
    w.agent_mut(id).unwrap().holdings[0] = 16.0;
    w.agent_mut(id).unwrap().metabolism[0] = 1;
    set_sugar(&mut w, 3, 5, 20.0);
    (w, id)
}

#[test]
fn behavior_tree_actual_harvest_completes_quota_on_one_step() {
    let (mut w, id) = policy_world();
    let mut s = TaskState::new(20);
    let t = act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!(
        (t.harvest.gathered[0], s.gross, s.first_completion),
        (20.0, 20.0, Some(1))
    );
}

#[test]
fn behavior_tree_failed_sites_expire_independently_on_t_plus_three() {
    let mut s = TaskState::new(20);
    note_failure(&mut s, 60, 3).unwrap();
    note_failure(&mut s, 61, 4).unwrap();
    expire_failed(&mut s, 6);
    assert_eq!(s.failed_until, std::collections::BTreeMap::from([(61, 7)]));
}

#[test]
fn behavior_tree_hidden_stock_does_not_override_remembered_choice() {
    use crate::minds::memory::Seen;
    let (mut full, id) = policy_world();
    full.config.memory.span = 128;
    full.config.growback.rate = 0.0;
    set_sugar(&mut full, 3, 5, 0.0);
    set_sugar(&mut full, 7, 5, 24.0);
    let site = full.torus.index(Pos::new(7, 5)) as u32;
    let a = full.agent_mut(id).unwrap();
    a.vision = 1;
    a.remembers = true;
    a.memory.sites.insert(site, Seen::new(&[24.0], &[24.0], 0));
    let mut empty = full.clone();
    empty.site_mut(Pos::new(7, 5)).resource[0] = 0.0;
    let mut left = TaskState::new(20);
    let mut right = TaskState::new(20);
    let a = act_routine(&mut full, id, &mut left, true, 64).unwrap();
    let b = act_routine(&mut empty, id, &mut right, true, 64).unwrap();
    assert_eq!((a.receipt, full.rng), (b.receipt, empty.rng));
}

#[test]
fn behavior_tree_failed_positive_targets_are_excluded_for_two_turns() {
    let (w, id) = policy_world();
    let mut s = TaskState::new(20);
    let o = observe(&w, id, &s).unwrap();
    note_failure(&mut s, 58, 1).unwrap();
    for t in [2, 3] {
        expire_failed(&mut s, t);
        assert_eq!(select_target(&o, &s, &mut w.rng.clone()), None);
    }
    expire_failed(&mut s, 4);
    assert_eq!(select_target(&o, &s, &mut w.rng.clone()), Some(58));
}

#[test]
fn behavior_tree_failure_on_retry_renews_only_its_deadline() {
    let mut s = TaskState::new(20);
    note_failure(&mut s, 60, 1).unwrap();
    note_failure(&mut s, 61, 2).unwrap();
    expire_failed(&mut s, 4);
    note_failure(&mut s, 60, 4).unwrap();
    assert_eq!(
        s.failed_until,
        std::collections::BTreeMap::from([(60, 7), (61, 5)])
    );
}

#[test]
fn behavior_tree_expiry_overflow_is_checked_before_mutation() {
    let mut s = TaskState::new(20);
    s.target = Some(60);
    let before = s.clone();
    assert!(note_failure(&mut s, 60, u64::MAX - 2).is_err());
    assert_eq!(s, before);
}

#[test]
fn behavior_tree_all_failed_positive_candidates_idle_without_rng_draw() {
    let (mut w, id) = policy_world();
    let mut s = TaskState::new(20);
    note_failure(&mut s, 58, 0).unwrap();
    let rng = w.rng.clone();
    let t = act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!(
        (t.harvest.gathered[0], w.agent(id).unwrap().pos, w.rng),
        (0.0, Pos::new(2, 5), rng)
    );
}

#[test]
fn behavior_tree_settles_completion_on_last_visit_and_stops_gathering() {
    let (mut w, id) = policy_world();
    let mut s = TaskState::new(20);
    let t = act_routine(&mut w, id, &mut s, true, 6).unwrap();
    assert_eq!(
        (t.status, s.gross, s.first_completion, s.target),
        (Status::Success, 20.0, Some(1), None)
    );
    set_sugar(&mut w, 3, 5, 4.0);
    let t = act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!(
        (t.harvest.gathered[0], w.site(Pos::new(3, 5)).resource[0]),
        (0.0, 4.0)
    );
}

#[test]
fn behavior_tree_tiny_budget_preserves_selected_target_and_rng() {
    let (mut w, id) = policy_world();
    w.bt_work = Some(WorkCounters::default());
    let mut s = TaskState::new(20);
    let t = act_routine(&mut w, id, &mut s, true, 5).unwrap();
    assert!(t.receipt.is_none());
    let rng = w.rng.clone();
    let selected = s.target;
    let t = act_routine(&mut w, id, &mut s, true, 5).unwrap();
    assert_eq!(
        (t.harvest.gathered[0], w.rng, selected),
        (20.0, rng, Some(58))
    );
    assert_eq!(w.bt_work.unwrap().target_selections, 0);
}

#[test]
fn behavior_tree_independent_fsm_matches_physical_task_and_rng_projection() {
    for drained in [false, true] {
        let (mut tree, id) = policy_world();
        tree.agent_mut(id).unwrap().vision = 8;
        set_sugar(&mut tree, 3, 5, 4.0);
        set_sugar(&mut tree, 7, 5, 24.0);
        let mut fsm = tree.clone();
        let mut ts = TaskState::new(40);
        let mut fs = TaskState::new(40);
        for tick in 0..12 {
            tree.tick = tick;
            fsm.tick = tick;
            if drained && tick == 2 {
                set_sugar(&mut tree, 7, 5, 0.0);
                set_sugar(&mut fsm, 7, 5, 0.0);
            }
            let a = act_routine(&mut tree, id, &mut ts, true, 64).unwrap();
            let b = act_fsm(&mut fsm, id, &mut fs).unwrap();
            assert_eq!(
                (
                    a.harvest,
                    a.receipt,
                    ts.gross,
                    ts.first_completion,
                    ts.target,
                    &ts.failed_until,
                    &tree.rng
                ),
                (
                    b.harvest,
                    b.receipt,
                    fs.gross,
                    fs.first_completion,
                    fs.target,
                    &fs.failed_until,
                    &fsm.rng
                ),
                "tick {tick}"
            );
        }
    }
}

#[test]
fn behavior_tree_task_goap_uses_remaining_quota_without_changing_legacy_goal() {
    let (mut task, id) = policy_world();
    task.agent_mut(id).unwrap().vision = 8;
    set_sugar(&mut task, 3, 5, 4.0);
    set_sugar(&mut task, 7, 5, 24.0);
    task.config.goap.horizon = 6;
    let mut legacy = task.clone();
    let mut s = TaskState::new(40);
    s.gross = 20.0;
    act_task_goap(&mut task, id, &mut s).unwrap();
    crate::minds::goap::forage::act(&mut legacy, id);
    assert_eq!(
        (
            s.task_plan.unwrap().goal,
            legacy.agent(id).unwrap().goap_plan.as_ref().unwrap().goal
        ),
        (20.0, 6.0)
    );
}

#[test]
fn behavior_tree_work_hooks_observe_without_changing_rng_or_actions() {
    let (mut measured, id) = policy_world();
    measured.bt_work = Some(WorkCounters::default());
    let mut plain = measured.clone();
    plain.bt_work = None;
    let a = crate::minds::utility::act(&mut measured, id);
    let b = crate::minds::utility::act(&mut plain, id);
    assert_eq!(
        (a, measured.agent(id).unwrap().pos, &measured.rng),
        (b, plain.agent(id).unwrap().pos, &plain.rng)
    );
    let c = measured.bt_work.unwrap();
    assert!(c.candidate_evaluations > 0);
    assert_eq!((c.target_selections, c.path_queries), (1, 1));
}

#[test]
fn behavior_tree_closed_profiles_validate_defaults_budget_combat_and_schedule() {
    let mut c = blank_config(11, 11);
    assert_eq!(
        c.behavior_tree,
        Settings {
            profile: Profile::BookLeaf,
            visits: 64
        }
    );
    c.decision.rule = DecisionRule::BehaviorTree;
    c.combat.enabled = true;
    assert!(c.validate().is_ok());
    c.behavior_tree.profile = Profile::UtilityLeaf;
    assert!(c.validate().is_err());
    c.combat.enabled = false;
    c.behavior_tree.visits = 0;
    assert!(c.validate().is_err());
    c.behavior_tree.visits = 65;
    assert!(c.validate().is_err());
    c.behavior_tree.visits = 64;
    for path in [
        "behavior_tree",
        "behavior_tree.profile",
        "behavior_tree.visits",
    ] {
        let value = match path {
            "behavior_tree" => serde_json::json!({"profile":"book_leaf","visits":64}),
            "behavior_tree.profile" => serde_json::json!("book_leaf"),
            _ => serde_json::json!(64),
        };
        c.schedule = vec![crate::config::ScheduledChange {
            tick: 1,
            set: std::collections::BTreeMap::from([(path.to_string(), value)]),
        }];
        assert!(c.validate().is_err(), "{path}");
    }
}

#[test]
fn behavior_tree_every_book_golden_profile_reduces_with_same_rng() {
    for p in crate::presets::all() {
        if p.config.decision.rule != DecisionRule::Book {
            continue;
        }
        let mut c = p.config.clone();
        c.decision.rule = DecisionRule::BehaviorTree;
        let mut book = crate::world::World::new(p.config, 7).unwrap();
        let mut leaf = crate::world::World::new(c, 7).unwrap();
        book.run(200);
        leaf.run(200);
        assert_eq!(
            (leaf.fingerprint(), leaf.rng),
            (book.fingerprint(), book.rng),
            "{}",
            p.id
        );
    }
}

#[test]
fn behavior_tree_utility_leaf_reduces_with_unchanged_utility_config() {
    for p in crate::presets::all() {
        if p.config.decision.rule != DecisionRule::Utility {
            continue;
        }
        let mut c = p.config.clone();
        c.decision.rule = DecisionRule::BehaviorTree;
        c.behavior_tree.profile = Profile::UtilityLeaf;
        let mut ordinary = crate::world::World::new(p.config, 7).unwrap();
        let mut leaf = crate::world::World::new(c, 7).unwrap();
        ordinary.run(200);
        leaf.run(200);
        assert_eq!(
            (leaf.fingerprint(), leaf.rng),
            (ordinary.fingerprint(), ordinary.rng),
            "{}",
            p.id
        );
    }
}

#[test]
fn behavior_tree_controller_graph_is_only_for_tree_controllers() {
    for c in [Controller::GuardedTree, Controller::UnguardedTree] {
        assert!(super::tree_for_controller(c).is_some());
    }
    for c in [
        Controller::ReactiveUtility,
        Controller::MatchedFsm,
        Controller::TaskGoap,
        Controller::LegacyGoap,
    ] {
        assert!(super::tree_for_controller(c).is_none());
    }
}

#[test]
fn behavior_tree_invalid_actor_is_rejected_before_effects() {
    let (mut w, _) = policy_world();
    let rng = w.rng.clone();
    let mut s = TaskState::new(20);
    let before = s.clone();
    assert!(act_routine(&mut w, 999, &mut s, true, 64).is_err());
    assert_eq!((w.rng, s), (rng, before));
}

#[test]
fn behavior_tree_stale_positive_empty_arrival_clears_target_without_cooldown() {
    use crate::minds::memory::Seen;
    let (mut w, id) = policy_world();
    w.config.memory.span = 128;
    set_sugar(&mut w, 3, 5, 0.0);
    let site = w.torus.index(Pos::new(7, 5)) as u32;
    let a = w.agent_mut(id).unwrap();
    a.remembers = true;
    a.vision = 1;
    a.memory.sites.insert(site, Seen::new(&[24.0], &[24.0], 0));
    let mut s = TaskState::new(20);
    s.target = Some(site);
    w.move_agent(id, Pos::new(6, 5));
    // Arrival ends the commitment; only physical route failure earns a cooldown.
    // Unguarded continuation deliberately arrives on an empty, now visible target.
    let t = act_routine(&mut w, id, &mut s, false, 64).unwrap();
    let r = t.receipt.unwrap();
    assert_eq!(
        (
            t.status,
            r.destination,
            r.gathered,
            r.route_failed,
            s.failed_until.get(&site).copied()
        ),
        (Status::Failure, Pos::new(7, 5), 0.0, false, None)
    );
}

#[test]
fn behavior_tree_physical_failed_route_has_its_own_receipt_and_cooldown() {
    use crate::minds::memory::Seen;
    let (mut w, id) = policy_world();
    w.config.memory.span = 128;
    set_sugar(&mut w, 3, 5, 0.0);
    let site = w.torus.index(Pos::new(7, 5)) as u32;
    let a = w.agent_mut(id).unwrap();
    a.remembers = true;
    a.vision = 1;
    a.memory.sites.insert(site, Seen::new(&[24.0], &[24.0], 0));
    for (x, y) in [(1, 5), (3, 5), (2, 4), (2, 6)] {
        spawn(&mut w, x, y);
    }
    let mut s = TaskState::new(20);
    let t = act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!(
        (
            t.status,
            t.receipt.unwrap().route_failed,
            s.failed_until.get(&site).copied()
        ),
        (Status::Failure, true, Some(4))
    );
}

#[test]
fn behavior_tree_guarded_invalid_target_normalizes_without_a_second_physical_action() {
    let (mut w, id) = policy_world();
    w.agent_mut(id).unwrap().vision = 8;
    set_sugar(&mut w, 3, 5, 0.0);
    set_sugar(&mut w, 7, 5, 24.0);
    let mut s = TaskState::new(40);
    act_routine(&mut w, id, &mut s, true, 64).unwrap();
    set_sugar(&mut w, 7, 5, 0.0);
    set_sugar(&mut w, 4, 5, 20.0);
    w.tick = 1;
    let t = act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!(
        (t.receipt.unwrap().target, t.harvest.gathered[0], s.gross),
        (Pos::new(4, 5), 20.0, 20.0)
    );
}

#[test]
fn behavior_tree_wire_settings_and_state_reject_unknown_fields() {
    let mut value = serde_json::to_value(TaskState::new(20)).unwrap();
    value["extra"] = serde_json::json!(1);
    assert!(serde_json::from_value::<TaskState>(value).is_err());
    assert!(
        serde_json::from_str::<Settings>(r#"{"profile":"book_leaf","visits":64,"extra":1}"#)
            .is_err()
    );
}

#[test]
fn behavior_tree_selection_uses_raw_food_observation_and_fixed_task_rate() {
    let (mut w, id) = policy_world();
    w.agent_mut(id).unwrap().vision = 8;
    w.config.decision.travel = 0.0;
    set_sugar(&mut w, 3, 5, 10.0);
    set_sugar(&mut w, 7, 5, 24.0);
    let s = TaskState::new(20);
    let o = observe(&w, id, &s).unwrap();
    assert_eq!(
        o.candidates.iter().find(|c| c.site == 62).unwrap().value,
        24.0
    );
    assert_eq!(select_target(&o, &s, &mut w.rng), Some(58));
}

#[test]
fn behavior_tree_error_restores_saved_traversal_before_effects() {
    let (mut w, id) = policy_world();
    let mut s = TaskState::new(20);
    s.tree.deferred = Some(99);
    let before = s.clone();
    let rng = w.rng.clone();
    assert!(act_routine(&mut w, id, &mut s, true, 64).is_err());
    assert_eq!((s, w.rng), (before, rng));
}

#[test]
fn behavior_tree_idle_collects_excluded_underfoot_food_without_clearing_deadline() {
    let (mut tree, id) = policy_world();
    set_sugar(&mut tree, 2, 5, 3.0);
    let mut fsm = tree.clone();
    let mut ts = TaskState::new(40);
    let mut fs = TaskState::new(40);
    for s in [&mut ts, &mut fs] {
        note_failure(s, 57, 0).unwrap();
        note_failure(s, 58, 0).unwrap();
    }
    let o = observe(&tree, id, &ts).unwrap();
    let rng = tree.rng.clone();
    assert_eq!(select_target(&o, &ts, &mut tree.rng), None);
    let a = act_routine(&mut tree, id, &mut ts, true, 64).unwrap();
    let b = act_fsm(&mut fsm, id, &mut fs).unwrap();
    assert_eq!(
        (
            a.harvest,
            a.receipt,
            ts.gross,
            ts.target,
            &ts.failed_until,
            &tree.rng
        ),
        (
            b.harvest,
            b.receipt,
            fs.gross,
            fs.target,
            &fs.failed_until,
            &fsm.rng
        )
    );
    assert_eq!(
        (ts.gross, ts.target, ts.failed_until, tree.rng),
        (
            3.0,
            None,
            std::collections::BTreeMap::from([(57, 3), (58, 3)]),
            rng
        )
    );
}

#[test]
fn behavior_tree_fsm_actually_gathers_and_finishes_quota() {
    let (mut w, id) = policy_world();
    let mut s = TaskState::new(20);
    let t = act_fsm(&mut w, id, &mut s).unwrap();
    assert_eq!(
        (
            t.harvest.gathered[0],
            s.gross,
            s.first_completion,
            s.fsm.phase
        ),
        (20.0, 20.0, Some(1), super::state::FsmPhase::Finished)
    );
}

#[test]
fn behavior_tree_pure_selection_keeps_rng_with_fixed_observation_under_hidden_changes() {
    let (mut w, id) = policy_world();
    let s = TaskState::new(20);
    let o = observe(&w, id, &s).unwrap();
    let mut left = w.rng.clone();
    let mut right = left.clone();
    let a = select_target(&o, &s, &mut left);
    // Neither hidden physical stock nor researcher scenario metadata enters selection.
    set_sugar(&mut w, 7, 5, 24.0);
    let metadata = super::state::LabConfig {
        controller: Controller::GuardedTree,
        scenario: super::state::Scenario::DepletedTarget,
        quota: 20,
        mirrored: true,
    };
    assert_eq!((a, left), (select_target(&o, &s, &mut right), right));
    assert!(metadata.mirrored);
}

#[test]
fn behavior_tree_multi_good_task_is_rejected_before_effects() {
    let (mut w, id) = policy_world();
    add_goods(&mut w.config, 2);
    let rng = w.rng.clone();
    let mut s = TaskState::new(20);
    let before = s.clone();
    assert!(act_fsm(&mut w, id, &mut s).is_err());
    assert_eq!((s, w.rng), (before, rng));
}

#[test]
fn behavior_tree_search_unavailability_is_observed_without_invented_counts() {
    let (mut w, _) = policy_world();
    w.bt_work = Some(WorkCounters::default());
    super::telemetry::note_search(&mut w, Some(9), None);
    assert_eq!(w.bt_work.as_ref().unwrap().search_expansions, Some(9));
    super::telemetry::note_search(
        &mut w,
        None,
        Some("failed search does not expose exact expansions"),
    );
    super::telemetry::note_search(&mut w, Some(3), None);
    assert_eq!(
        (
            w.bt_work.as_ref().unwrap().search_expansions,
            w.bt_work.unwrap().search_unavailable_reason
        ),
        (
            None,
            Some("failed search does not expose exact expansions".into())
        )
    );
}

#[test]
fn behavior_tree_work_counts_raw_welfare_and_enabled_utility_rescoring() {
    let (mut w, id) = policy_world();
    w.bt_work = Some(WorkCounters::default());
    crate::minds::utility::act(&mut w, id);
    assert_eq!(w.bt_work.unwrap().candidate_evaluations, 10);
}

#[test]
fn behavior_tree_work_counts_retained_observation_without_new_selection() {
    use crate::minds::memory::Seen;
    let (mut w, id) = policy_world();
    w.config.memory.span = 128;
    set_sugar(&mut w, 3, 5, 0.0);
    let site = w.torus.index(Pos::new(7, 5)) as u32;
    let a = w.agent_mut(id).unwrap();
    a.remembers = true;
    a.memory.sites.insert(site, Seen::new(&[24.0], &[24.0], 0));
    w.bt_work = Some(WorkCounters::default());
    let mut s = TaskState::new(40);
    act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!(
        (
            w.bt_work.as_ref().unwrap().candidate_evaluations,
            w.bt_work.as_ref().unwrap().target_selections
        ),
        (7, 1)
    );
    w.tick = 1;
    act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!(
        (
            w.bt_work.as_ref().unwrap().candidate_evaluations,
            w.bt_work.as_ref().unwrap().target_selections
        ),
        (6, 0)
    );
}

#[test]
fn behavior_tree_work_domain_observer_preserves_real_plan_and_exact_counts() {
    use crate::minds::goap::{forage::Forage, plan};
    let sites = [(Pos::new(2, 5), 0.0), (Pos::new(3, 5), 20.0)];
    let torus = crate::geometry::Torus::new(11, 11);
    let plain = Forage::new(torus, &sites, 20.0);
    let mut measured = Forage::new(torus, &sites, 20.0);
    measured.observe_candidates();
    let a = plan(&plain, (0, 0), 4096).unwrap();
    let b = plan(&measured, (0, 0), 4096).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        (
            b.actions,
            b.cost,
            b.expanded,
            measured.candidate_evaluations()
        ),
        (vec![1], 2.0, 2, Some(7))
    );
    let mut capped = Forage::new(torus, &sites, 20.0);
    capped.observe_candidates();
    assert!(plan(&capped, (0, 0), 0).is_none());
    assert_eq!(capped.candidate_evaluations(), Some(2));
}

#[test]
fn behavior_tree_work_legacy_goap_observer_preserves_hash_physical_and_rng() {
    let (mut measured, id) = policy_world();
    measured.config.goap.horizon = 6;
    measured.bt_work = Some(WorkCounters::default());
    let mut plain = measured.clone();
    plain.bt_work = None;
    let a = crate::minds::goap::forage::act(&mut measured, id);
    let b = crate::minds::goap::forage::act(&mut plain, id);
    assert_eq!(
        (a, measured.fingerprint(), &measured.rng),
        (b, plain.fingerprint(), &plain.rng)
    );
    assert_eq!(
        (
            measured.bt_work.as_ref().unwrap().search_expansions,
            measured.bt_work.as_ref().unwrap().target_selections
        ),
        (Some(2), 1)
    );
}

#[test]
fn behavior_tree_retained_absent_candidate_normalizes_for_both_trees_and_fsm() {
    let (mut base, id) = policy_world();
    set_sugar(&mut base, 7, 5, 24.0);
    let mut tree = base.clone();
    let mut unguarded = base.clone();
    let mut fsm = base;
    let mut gs = TaskState::new(40);
    gs.target = Some(62);
    let mut us = gs.clone();
    let mut fs = gs.clone();
    let a = act_routine(&mut tree, id, &mut gs, true, 64).unwrap();
    let b = act_routine(&mut unguarded, id, &mut us, false, 64).unwrap();
    let c = act_fsm(&mut fsm, id, &mut fs).unwrap();
    assert_eq!(a.receipt.as_ref().unwrap().target, Pos::new(3, 5));
    assert_eq!((a.receipt, &tree.rng), (b.receipt, &unguarded.rng));
    assert_eq!(
        (a.harvest, gs.gross, gs.target, &tree.rng),
        (c.harvest, fs.gross, fs.target, &fsm.rng)
    );
}

#[test]
fn behavior_tree_retained_present_zero_value_is_only_unguarded_commitment() {
    let (mut base, id) = policy_world();
    base.agent_mut(id).unwrap().vision = 8;
    set_sugar(&mut base, 3, 5, 0.0);
    set_sugar(&mut base, 4, 5, 20.0);
    let mut tree = base.clone();
    let mut unguarded = base.clone();
    let mut fsm = base;
    let mut gs = TaskState::new(40);
    gs.target = Some(58);
    let mut us = gs.clone();
    let mut fs = gs.clone();
    let a = act_routine(&mut tree, id, &mut gs, true, 64).unwrap();
    let b = act_routine(&mut unguarded, id, &mut us, false, 64).unwrap();
    let c = act_fsm(&mut fsm, id, &mut fs).unwrap();
    assert_eq!(
        (
            a.receipt.as_ref().unwrap().target,
            b.receipt.as_ref().unwrap().target
        ),
        (Pos::new(4, 5), Pos::new(3, 5))
    );
    assert_eq!((a.receipt, &tree.rng), (c.receipt, &fsm.rng));
    assert_eq!(
        // Empty arrival clears commitment, but the route succeeded.
        (b.status, us.target, us.failed_until.get(&58).copied()),
        (Status::Failure, None, None)
    );
}

#[test]
fn behavior_tree_task_goap_retains_half_value_then_replans_below_half() {
    let (mut w, id) = policy_world();
    w.agent_mut(id).unwrap().vision = 8;
    set_sugar(&mut w, 3, 5, 4.0);
    set_sugar(&mut w, 7, 5, 24.0);
    w.bt_work = Some(WorkCounters::default());
    let mut s = TaskState::new(40);
    s.gross = 20.0;
    act_task_goap(&mut w, id, &mut s).unwrap();
    set_sugar(&mut w, 7, 5, 12.0);
    w.tick = 1;
    let half = act_task_goap(&mut w, id, &mut s).unwrap();
    assert_eq!(
        (
            half.receipt.unwrap().target,
            w.bt_work.as_ref().unwrap().target_selections
        ),
        (Pos::new(7, 5), 0)
    );
    set_sugar(&mut w, 7, 5, 11.0);
    set_sugar(&mut w, 4, 6, 20.0);
    w.tick = 2;
    let below = act_task_goap(&mut w, id, &mut s).unwrap();
    assert_eq!(
        (
            below.receipt.unwrap().target,
            w.bt_work.as_ref().unwrap().target_selections
        ),
        (Pos::new(4, 6), 1)
    );
}

#[test]
fn behavior_tree_task_goap_revalidates_a_physically_failed_route() {
    let (mut w, id) = policy_world();
    w.agent_mut(id).unwrap().vision = 8;
    set_sugar(&mut w, 3, 5, 0.0);
    set_sugar(&mut w, 7, 5, 24.0);
    for (x, y) in [(6, 5), (8, 5), (7, 4), (7, 6)] {
        spawn(&mut w, x, y);
    }
    let mut s = TaskState::new(20);
    let failed = act_task_goap(&mut w, id, &mut s).unwrap();
    assert!(failed.receipt.unwrap().route_failed);
    set_sugar(&mut w, 3, 5, 20.0);
    w.tick = 1;
    let retried = act_task_goap(&mut w, id, &mut s).unwrap();
    assert_eq!(
        (retried.receipt.unwrap().target, s.gross),
        (Pos::new(3, 5), 20.0)
    );
}

#[test]
fn behavior_tree_legacy_failed_search_keeps_exact_expansions_unavailable() {
    let mut c = blank_config(21, 21);
    c.movement = Movement {
        mode: MoveMode::Walk,
        speed: 1,
    };
    c.goap.k = 12;
    c.goap.horizon = 1;
    let mut measured = crate::world::World::new(c, 7).unwrap();
    let id = spawn(&mut measured, 10, 10);
    let a = measured.agent_mut(id).unwrap();
    a.vision = 3;
    a.metabolism[0] = 13;
    set_sugar(&mut measured, 10, 10, 1.0);
    for (q, _) in measured.sight(Pos::new(10, 10), 3) {
        set_sugar(&mut measured, q.x, q.y, 1.0);
    }
    let mut plain = measured.clone();
    measured.bt_work = Some(WorkCounters::default());
    let a = crate::minds::goap::forage::act(&mut measured, id);
    let b = crate::minds::goap::forage::act(&mut plain, id);
    assert_eq!(
        (a, measured.fingerprint(), &measured.rng),
        (b, plain.fingerprint(), &plain.rng)
    );
    let c = measured.bt_work.unwrap();
    assert_eq!(
        (
            c.search_expansions,
            c.search_unavailable_reason,
            c.fallback_limit
        ),
        (
            None,
            Some("failed search does not expose exact expansions".into()),
            1
        )
    );
    assert!(c.candidate_evaluations > 0);
}

#[test]
fn behavior_tree_shared_settlement_empty_arrival_preserves_other_deadlines() {
    let mut state = TaskState::new(20);
    state.target = Some(62);
    state.failed_until.insert(60, 5);
    let receipt = super::state::PhysicalReceipt {
        action_tick: 1,
        actor: 1,
        origin: Pos::new(6, 5),
        target: Pos::new(7, 5),
        destination: Pos::new(7, 5),
        gathered: 0.0,
        route_failed: false,
    };
    super::forage::settle(&mut state, &receipt).unwrap();
    assert_eq!(
        (state.target, state.gross, state.failed_until),
        (None, 0.0, [(60, 5)].into())
    );
}

#[test]
fn behavior_tree_all_task_controllers_empty_arrival_has_no_cooldown_or_extra_action() {
    use super::state::TaskPlan;
    use crate::minds::memory::Seen;
    for controller in [
        Controller::GuardedTree,
        Controller::UnguardedTree,
        Controller::MatchedFsm,
        Controller::TaskGoap,
    ] {
        let (mut w, id) = policy_world();
        w.config.memory.span = 128;
        w.config.growback.rate = 0.0;
        set_sugar(&mut w, 3, 5, 0.0);
        w.move_agent(id, Pos::new(6, 5));
        let a = w.agent_mut(id).unwrap();
        a.remembers = true;
        // Software fixture: adjacent remembered target is outside this actor's sight.
        a.vision = 0;
        a.memory.sites.insert(62, Seen::new(&[24.0], &[24.0], 0));
        w.bt_work = Some(WorkCounters::default());
        let mut state = TaskState::new(20);
        state.target = Some(62);
        state.failed_until.insert(60, 5);
        if controller == Controller::TaskGoap {
            state.task_plan = Some(TaskPlan {
                goal: 20.0,
                steps: vec![(Pos::new(7, 5), 24.0)],
            });
        }
        let rng = w.rng.clone();
        let holdings = w.agent(id).unwrap().holdings;
        let turn = match controller {
            Controller::MatchedFsm => act_fsm(&mut w, id, &mut state),
            Controller::TaskGoap => act_task_goap(&mut w, id, &mut state),
            _ => act_routine(
                &mut w,
                id,
                &mut state,
                controller == Controller::GuardedTree,
                64,
            ),
        }
        .unwrap();
        let receipt = turn.receipt.unwrap();
        assert_eq!(
            (
                turn.status,
                receipt.destination,
                receipt.gathered,
                receipt.route_failed
            ),
            (Status::Failure, Pos::new(7, 5), 0.0, false),
            "{controller:?}"
        );
        assert_eq!(
            (state.target, state.gross, state.failed_until),
            (None, 0.0, [(60, 5)].into()),
            "{controller:?}"
        );
        assert_eq!((&w.rng, w.agent(id).unwrap().holdings), (&rng, holdings));
        assert_eq!(w.bt_work.unwrap().path_queries, 1);
    }
}
