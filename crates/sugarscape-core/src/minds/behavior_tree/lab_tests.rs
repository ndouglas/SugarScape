use super::{lab, state::*};
use crate::{geometry::Pos, world::World};
fn event_world(scenario: Scenario) -> World {
    World::new(
        lab::rig_config(LabConfig {
            controller: Controller::GuardedTree,
            scenario,
            quota: 20,
            mirrored: false,
        }),
        7,
    )
    .unwrap()
}
#[test]
fn behavior_tree_events_continue_after_actor_removal() {
    let mut w = event_world(Scenario::BetterAlternative);
    w.remove(1).unwrap();
    w.run(4);
    assert_eq!(
        (w.tick, w.behavior_tree_lab.as_ref().unwrap().external_added),
        (4, 12.0)
    );
}
#[test]
fn behavior_tree_occupied_event_stops_before_action_or_tick_increment() {
    let mut w = event_world(Scenario::TemporaryObstacle);
    w.run(2);
    w.move_agent(1, Pos::new(5, 5));
    let before = w.agent(1).unwrap().holdings[0];
    let rng = crate::rng::state_json(&w.rng);
    w.step();
    assert_eq!(crate::rng::state_json(&w.rng), rng);
    assert_eq!(w.agent(1).unwrap().pos, Pos::new(5, 5));
    assert!(!w.is_wall(Pos::new(5, 5)));
    assert_eq!((w.tick, w.agent(1).unwrap().holdings[0]), (2, before));
    assert!(w.behavior_tree_lab.as_ref().unwrap().fatal_error.is_some());
}
#[test]
fn behavior_tree_interventions_use_before_action_clock() {
    let mut w = event_world(Scenario::BetterAlternative);
    w.remove(1);
    w.run(2);
    assert_eq!(w.site(Pos::new(4, 9)).resource[0], 24.0);
    w.step();
    assert_eq!(w.site(Pos::new(4, 9)).resource[0], 36.0);
    let mut w = event_world(Scenario::TemporaryObstacle);
    w.remove(1);
    w.run(2);
    assert!(!w.is_wall(Pos::new(5, 5)));
    w.step();
    assert!(w.is_opaque(Pos::new(5, 5)));
    w.run(3);
    assert!(w.is_wall(Pos::new(5, 5)));
    w.step();
    assert!(!w.is_wall(Pos::new(5, 5)));
}
#[test]
fn behavior_tree_completion_holds_but_events_and_metabolism_continue() {
    let mut w = event_world(Scenario::TemporaryObstacle);
    // Software-only fixture: the extra 16 units are declared in the balance.
    w.site_mut(Pos::new(3, 5)).resource[0] = 20.0;
    w.site_mut(Pos::new(3, 5)).capacity[0] = 20.0;
    w.behavior_tree_lab.as_mut().unwrap().external_added = 16.0;
    w.step();
    assert_eq!(
        w.behavior_tree_lab.as_ref().unwrap().task.first_completion,
        Some(1)
    );
    let held = w.agent(1).unwrap().holdings[0];
    let plan = w.agent(1).unwrap().plan.clone();
    w.run(2);
    assert!(w.is_wall(Pos::new(5, 5)));
    w.run(4);
    assert!(!w.is_wall(Pos::new(5, 5)));
    assert_eq!(w.agent(1).unwrap().holdings[0], held - 6.0);
    assert_eq!(w.agent(1).unwrap().plan, plan);
    assert_eq!(w.behavior_tree_lab.as_ref().unwrap().task.gross, 20.0);
}
#[test]
fn behavior_tree_fatal_gate_cannot_resume() {
    let mut w = event_world(Scenario::TemporaryObstacle);
    w.run(2);
    w.move_agent(1, Pos::new(5, 5));
    w.step();
    let hash = w.fingerprint();
    let tick = w.tick;
    w.step();
    w.run(64);
    assert_eq!((w.tick, w.fingerprint()), (tick, hash));
    assert!(w.is_finished());
}
#[test]
fn behavior_tree_every_other_config_field_is_fixed() {
    let c = lab::rig_config(LabConfig {
        controller: Controller::GuardedTree,
        scenario: Scenario::Stable,
        quota: 20,
        mirrored: false,
    });
    let mut changed = c.clone();
    changed.caching.capacity = 1;
    assert!(changed.validate().is_err());
    let mut changed = c.clone();
    changed.growback.rate = 1.0;
    assert!(changed.validate().is_err());
    let mut changed = c.clone();
    changed.memory.prior = crate::config::MemoryPrior::None;
    assert!(changed.validate().is_err());
    let mut changed = c;
    changed.behavior_tree.visits = 63;
    assert!(changed.validate().is_err());
}
#[test]
fn behavior_tree_hidden_stock_does_not_rewrite_remembered_candidate() {
    let mut w = event_world(Scenario::Stable);
    w.move_agent(1, Pos::new(2, 2));
    w.site_mut(Pos::new(7, 5)).resource[0] = 0.0;
    let o = super::forage::observe(&w, 1, &w.behavior_tree_lab.as_ref().unwrap().task).unwrap();
    let b = o
        .candidates
        .iter()
        .find(|c| c.pos == Pos::new(7, 5))
        .unwrap();
    assert!(b.remembered);
    assert_eq!(b.value, 24.0);
}
#[test]
fn behavior_tree_all_condition_geometry_and_events_reflect_without_tuning() {
    for condition in lab::conditions() {
        let mut other = condition.clone();
        other.mirrored = !other.mirrored;
        let mut a = World::new(lab::rig_config(condition.clone()), 7).unwrap();
        let mut b = World::new(lab::rig_config(other), 7).unwrap();
        assert_eq!(
            a.agent(1).unwrap().pos,
            Pos::new(10 - b.agent(1).unwrap().pos.x, 5)
        );
        for i in 0..121 {
            let p = a.torus.pos(i);
            let mirror = Pos::new(10 - p.x, p.y);
            assert_eq!(a.site(p), b.site(mirror));
            assert_eq!(a.is_opaque(p), b.is_opaque(mirror));
            let ma = a.agent(1).unwrap().memory.sites.get(&(i as u32));
            let mb = b
                .agent(1)
                .unwrap()
                .memory
                .sites
                .get(&(b.torus.index(mirror) as u32));
            assert_eq!(ma, mb);
        }
        // Software-only removals expose scheduled geometry independently of policies.
        a.remove(1);
        b.remove(1);
        for _ in 0..7 {
            a.step();
            b.step();
            for i in 0..121 {
                let p = a.torus.pos(i);
                let mirror = Pos::new(10 - p.x, p.y);
                assert_eq!(a.site(p), b.site(mirror));
                assert_eq!(a.is_opaque(p), b.is_opaque(mirror));
            }
        }
    }
}
#[test]
fn behavior_tree_failed_attempt_uses_turn_and_keeps_independent_deadlines_through_checkpoint() {
    // Software-only sealed origin. The actual movement primitive reports failure.
    let mut w = event_world(Scenario::Stable);
    w.move_agent(1, Pos::new(2, 2));
    for p in [
        Pos::new(1, 2),
        Pos::new(3, 2),
        Pos::new(2, 1),
        Pos::new(2, 3),
    ] {
        w.close_behavior_tree_wall(p).unwrap();
    }
    let c = w.torus.index(Pos::new(4, 9)) as u32;
    w.step();
    let r = w.behavior_tree_lab.as_ref().unwrap();
    assert!(r.receipt.as_ref().unwrap().route_failed);
    assert_eq!(r.task.failed_until.get(&c), Some(&4));
    assert_eq!(r.task.target, None);
    w.step();
    let r = w.behavior_tree_lab.as_ref().unwrap();
    assert_eq!(r.task.failed_until.get(&c), Some(&4));
    assert_eq!(r.task.failed_until.len(), 2);
    assert_eq!((w.tick, w.agent(1).unwrap().holdings[0]), (2, 14.0));
    let mut saved = w.clone();
    for _ in 0..4 {
        w.step();
        saved.step();
        assert_eq!(w.fingerprint(), saved.fingerprint());
    }
}
#[test]
fn behavior_tree_dead_actor_deadlines_expire_without_actions_or_rng_and_prior_frames_remain() {
    let mut w = event_world(Scenario::BetterAlternative);
    w.move_agent(1, Pos::new(2, 2));
    for p in [
        Pos::new(1, 2),
        Pos::new(3, 2),
        Pos::new(2, 1),
        Pos::new(2, 3),
    ] {
        w.close_behavior_tree_wall(p).unwrap();
    }
    w.step();
    let prior = super::runner::frame(&w);
    let c = w.torus.index(Pos::new(4, 9)) as u32;
    assert_eq!(prior.task.failed_until.get(&c), Some(&4));
    w.remove(1).unwrap();
    let rng = crate::rng::state_json(&w.rng);
    w.run(2);
    assert_eq!(
        w.behavior_tree_lab
            .as_ref()
            .unwrap()
            .task
            .failed_until
            .get(&c),
        Some(&4)
    );
    w.step();
    let after = super::runner::frame(&w);
    assert_eq!(after.tick, 4);
    assert!(after.task.failed_until.is_empty());
    assert_eq!(prior.task.failed_until.get(&c), Some(&4));
    assert_eq!(after.rng_state_json, rng);
    assert_eq!((after.living_ticks, after.external_added), (1, 12.0));
    assert!(after.actor.is_none() && after.receipt.is_none() && after.work.is_none());
}
