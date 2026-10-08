//! Hand actions only; no registered seed or episode schedule execution.
use super::{
    controller::{perform_bout, BoutKind, BoutResult},
    lab::rig_config,
    state::*,
};
use crate::{
    geometry::Pos,
    minds::{
        caching,
        protection::ledger::{Ledger, Outflow},
    },
    rules::{self, lifecycle, movement, Harvest},
    world::{DeathCause, World},
};

fn rig() -> World {
    World::new(rig_config(LabConfig::default()), 7).unwrap()
}

#[test]
fn sham_spends_effort_without_creating_food() {
    let mut w = rig();
    let caches = w.agent(1).unwrap().caches.clone();
    let cohorts = w
        .deception
        .as_ref()
        .unwrap()
        .ledger
        .as_ref()
        .unwrap()
        .cohorts
        .len();
    assert_eq!(
        perform_bout(&mut w, 1, BoutKind::Sham, 3.0).unwrap(),
        BoutResult::Completed
    );
    assert_eq!(w.agent(1).unwrap().holdings[0], 41.0);
    assert_eq!(w.agent(1).unwrap().caches, caches);
    assert_eq!(
        w.deception
            .as_ref()
            .unwrap()
            .ledger
            .as_ref()
            .unwrap()
            .cohorts
            .len(),
        cohorts
    );
    assert_eq!(
        (w.events.buried, w.events.bury_cost, w.events.burials_seen),
        (0.0, 0.0, 0)
    );
    assert_eq!(w.agent(2).unwrap().seen[&(30, 1)].amount, 12.0);
    assert_eq!(w.agent(2).unwrap().seen.len(), 1);
    assert_eq!(w.deception.as_ref().unwrap().sham_sightings, 1);
    assert!(perform_bout(&mut w, 1, BoutKind::Sham, 3.0).is_err());
    assert_eq!(w.agent(1).unwrap().holdings[0], 41.0);
    assert_eq!(w.agent(2).unwrap().seen[&(30, 1)].amount, 12.0);
}
#[test]
fn neutral_spends_identical_effort_without_cue() {
    let mut w = rig();
    assert_eq!(
        perform_bout(&mut w, 1, BoutKind::Neutral, 3.0).unwrap(),
        BoutResult::Completed
    );
    assert_eq!(w.agent(1).unwrap().holdings[0], 41.0);
    assert!(w.agent(2).unwrap().seen.is_empty());
}
#[test]
fn clear_or_unseen_sham_has_no_positive_cue() {
    for clear in [false, true] {
        let mut w = rig();
        if clear {
            w.config.deception_lab.as_mut().unwrap().view = View::Clear;
        } else {
            w.move_agent(2, Pos::new(4, 6));
        }
        perform_bout(&mut w, 1, BoutKind::Sham, 3.0).unwrap();
        assert_eq!(w.agent(1).unwrap().holdings[0], 41.0);
        assert!(w.agent(2).unwrap().seen.is_empty());
    }
}
#[test]
fn unaffordable_attempt_is_final_and_has_no_cue_or_debit() {
    let mut w = rig();
    w.agent_mut(1).unwrap().holdings[0] = 2.0;
    assert_eq!(
        perform_bout(&mut w, 1, BoutKind::Sham, 3.0).unwrap(),
        BoutResult::Unaffordable
    );
    assert!(w.agent(1).unwrap().deception.as_ref().unwrap().attempted);
    assert!(perform_bout(&mut w, 1, BoutKind::Sham, 0.0).is_err());
    assert_eq!(w.agent(1).unwrap().holdings[0], 2.0);
    assert!(w.agent(2).unwrap().seen.is_empty());
    assert_eq!(w.deception.as_ref().unwrap().bouts.len(), 1);
}
#[test]
fn invalid_effort_or_receiver_cannot_spend_or_emit() {
    for effort in [-1.0, f64::NAN, f64::INFINITY] {
        let mut w = rig();
        assert!(perform_bout(&mut w, 1, BoutKind::Sham, effort).is_err());
        assert_eq!(w.agent(1).unwrap().holdings[0], 44.0);
        assert!(w.agent(2).unwrap().seen.is_empty());
    }
    let mut w = rig();
    assert!(perform_bout(&mut w, 2, BoutKind::Sham, 3.0).is_err());
    w.remove(1);
    assert!(perform_bout(&mut w, 1, BoutKind::Sham, 3.0).is_err());
}
#[test]
fn real_burial_legacy_hook_hides_quantity_and_never_double_adds() {
    for amount in [3.0, 9.0] {
        let mut w = rig();
        w.tick = 13;
        assert_eq!(caching::bury(&mut w, 1, amount), amount);
        assert_eq!(w.agent(2).unwrap().seen[&(30, 1)].amount, 12.0);
        assert_eq!((w.events.burials_seen, w.events.sightings), (1, 1));
        assert_eq!(w.events.buried, amount);
        assert!(w
            .deception
            .as_ref()
            .unwrap()
            .ledger
            .as_ref()
            .unwrap()
            .cohorts
            .is_empty());
    }
}
#[test]
fn initial_deposit_is_clear_for_both_views_and_only_original_is_labelled() {
    for view in [View::Clear, View::Ambiguous] {
        let mut w = rig();
        w.config.deception_lab.as_mut().unwrap().view = view;
        caching::bury(&mut w, 1, 12.0);
        assert!(w.deception.as_ref().unwrap().prepared);
        assert_eq!(w.agent(2).unwrap().seen[&(30, 1)].amount, 12.0);
        let ledger = w.deception.as_ref().unwrap().ledger.as_ref().unwrap();
        assert_eq!(ledger.cohorts[&30].initial, 12.0);
        assert_eq!(ledger.unlabelled_carried, 32.0);
        caching::bury(&mut w, 1, 3.0);
        assert_eq!(
            w.deception
                .as_ref()
                .unwrap()
                .ledger
                .as_ref()
                .unwrap()
                .cohorts
                .len(),
            1
        );
    }
}
#[test]
fn clear_zero_and_withdrawal_preserve_historical_positive_evidence() {
    let mut w = rig();
    caching::bury(&mut w, 1, 12.0);
    let take = caching::dig(&mut w, 1, 30, 12.0);
    w.agent_mut(1).unwrap().holdings[0] += take;
    super::accounting::reconcile_world(&mut w);
    w.config.deception_lab.as_mut().unwrap().view = View::Clear;
    w.tick = 13;
    perform_bout(&mut w, 1, BoutKind::Sham, 3.0).unwrap();
    assert_eq!(
        w.agent(2).unwrap().seen[&(30, 1)],
        caching::watching::SeenCache {
            amount: 12.0,
            tick: 0
        }
    );
    assert!(w.agent(1).unwrap().caches.is_empty());
    assert!(w.deception.as_ref().unwrap().ledger_errors.is_empty());
}
#[test]
fn complete_sender_call_path_is_private_to_own_state() {
    let base = rig();
    for view in [View::Clear, View::Ambiguous] {
        let mut a = base.clone();
        let mut b = base.clone();
        b.config.deception_lab.as_mut().unwrap().view = view;
        b.agent_mut(2).unwrap().seen.insert(
            (30, 1),
            caching::watching::SeenCache {
                amount: 999.0,
                tick: 0,
            },
        );
        assert_eq!(
            perform_bout(&mut a, 1, BoutKind::Sham, 3.0),
            perform_bout(&mut b, 1, BoutKind::Sham, 3.0)
        );
        assert_eq!(a.agent(1).unwrap().holdings, b.agent(1).unwrap().holdings);
        assert_eq!(a.agent(1).unwrap().pos, b.agent(1).unwrap().pos);
        assert_eq!(a.agent(1).unwrap().deception, b.agent(1).unwrap().deception);
    }
}
#[test]
fn display_turn_spends_action_then_normal_metabolism_without_harvest() {
    for (holdings, sender) in [
        (44.0, SenderPolicy::Sham),
        (2.0, SenderPolicy::Sham),
        (44.0, SenderPolicy::MatchedNeutral),
        (2.0, SenderPolicy::MatchedNeutral),
    ] {
        let mut w = rig();
        w.config.deception_lab.as_mut().unwrap().sender = sender;
        w.config.deception_lab.as_mut().unwrap().effort_cost = 3.0;
        w.agent_mut(1).unwrap().deception.as_mut().unwrap().stage = Stage::Display;
        w.agent_mut(1).unwrap().holdings[0] = holdings;
        w.site_mut(Pos::new(3, 3)).resource[0] = 4.0;
        rules::agent_turn(&mut w, 1);
        assert_eq!(
            w.agent(1).unwrap().holdings[0],
            if holdings == 44.0 { 40.0 } else { 1.0 }
        );
        assert_eq!(w.site(Pos::new(3, 3)).resource[0], 4.0);
        assert_eq!(
            w.agent(2).unwrap().seen.is_empty(),
            holdings == 2.0 || sender == SenderPolicy::MatchedNeutral
        );
    }
}
#[test]
fn effort_debits_labelled_food_proportionally() {
    let mut l = Ledger::new(1, 18.0);
    l.prepare(30, 12.0).unwrap();
    l.withdraw(30, 12.0).unwrap();
    l.outflow(3.0, Outflow::ActionCost).unwrap();
    assert_eq!(l.cohorts[&30].cost, 2.0);
    assert_eq!(l.cohorts[&30].carried, 10.0);
    assert_eq!(l.unlabelled_carried, 5.0);
}
#[test]
fn diagnostic_switch_does_not_change_hand_action_biology() {
    let mut on = rig();
    let mut off = on.clone();
    off.deception.as_mut().unwrap().diagnostics = false;
    for w in [&mut on, &mut off] {
        caching::bury(w, 1, 12.0);
        w.tick = 13;
        perform_bout(w, 1, BoutKind::Sham, 3.0).unwrap();
        lifecycle::metabolize(w, 1, Harvest::default());
        movement::arrive(w, 1, Pos::new(2, 3));
        lifecycle::metabolize(w, 1, Harvest::default());
        super::accounting::reconcile_world(w);
    }
    assert_eq!(on.fingerprint(), off.fingerprint());
    assert_eq!(
        on.agent(1).unwrap().holdings,
        off.agent(1).unwrap().holdings
    );
    assert!(on.deception.as_ref().unwrap().ledger_errors.is_empty());
    assert!(off.deception.as_ref().unwrap().bouts.is_empty());
}
#[test]
fn owner_death_accounts_terminal_original_food() {
    let mut w = rig();
    caching::bury(&mut w, 1, 12.0);
    let take = caching::dig(&mut w, 1, 30, 6.0);
    w.agent_mut(1).unwrap().holdings[0] += take;
    w.kill(1, DeathCause::Starvation);
    let c = &w
        .deception
        .as_ref()
        .unwrap()
        .ledger
        .as_ref()
        .unwrap()
        .cohorts[&30];
    assert_eq!((c.lost_carried, c.lost_cached[&30]), (6.0, 6.0));
    assert_eq!((c.carried, c.cached.len()), (0.0, 0));
    assert!(w.deception.as_ref().unwrap().ledger_errors.is_empty());
}
#[test]
fn diagnostic_failure_is_absorbing_without_stopping_paid_action() {
    let mut w = rig();
    super::accounting::update(&mut w, 1, |l| l.outflow(100.0, Outflow::Consumption));
    assert!(!w.deception.as_ref().unwrap().diagnostics);
    assert!(w.deception.as_ref().unwrap().ledger.is_none());
    perform_bout(&mut w, 1, BoutKind::Sham, 3.0).unwrap();
    assert_eq!(w.agent(1).unwrap().holdings[0], 41.0);
    assert_eq!(w.agent(2).unwrap().seen[&(30, 1)].amount, 12.0);
    assert_eq!(w.deception.as_ref().unwrap().ledger_errors.len(), 1);
}
#[test]
fn p4_pilfer_and_environment_harvest_reconcile_actual_stocks() {
    let mut w = rig();
    caching::bury(&mut w, 1, 12.0);
    w.move_agent(1, Pos::new(2, 3));
    movement::arrive(&mut w, 2, Pos::new(3, 5));
    movement::arrive(&mut w, 2, Pos::new(3, 4));
    movement::arrive(&mut w, 2, Pos::new(3, 3));
    assert_eq!(
        w.deception
            .as_ref()
            .unwrap()
            .ledger
            .as_ref()
            .unwrap()
            .cohorts[&30]
            .transferred,
        12.0
    );
    movement::gather_site(&mut w, 1, Pos::new(2, 3), 32.0, false);
    assert_eq!(
        w.deception
            .as_ref()
            .unwrap()
            .ledger
            .as_ref()
            .unwrap()
            .unlabelled_carried,
        36.0
    );
    assert!(w.deception.as_ref().unwrap().ledger_errors.is_empty());
}

#[test]
fn disabled_p4_preserves_legacy_quantity_channel_and_ledger() {
    let mut w = rig();
    w.config.deception_lab = None;
    let before = w.deception.as_ref().unwrap().ledger.clone();
    assert!(perform_bout(&mut w, 1, BoutKind::Sham, 3.0).is_err());
    caching::bury(&mut w, 1, 3.0);
    assert_eq!(w.agent(2).unwrap().seen[&(30, 1)].amount, 3.0);
    assert_eq!(w.deception.as_ref().unwrap().ledger, before);
}
#[test]
fn physical_dispatch_keeps_receiver_memory_bounded() {
    let mut w = rig();
    for actor in 1000..5096 {
        w.agent_mut(2).unwrap().seen.insert(
            (0, actor),
            caching::watching::SeenCache {
                amount: 1.0,
                tick: 0,
            },
        );
    }
    w.tick = 13;
    perform_bout(&mut w, 1, BoutKind::Sham, 0.0).unwrap();
    let seen = &w.agent(2).unwrap().seen;
    assert_eq!(seen.len(), 4096);
    assert!(!seen.contains_key(&(0, 1000)));
    assert_eq!(seen[&(30, 1)].amount, 12.0);
}
#[test]
fn legacy_ledger_variants_keep_encodings_and_mixed_debits() {
    for (kind, value) in [
        (Outflow::Consumption, serde_json::json!("Consumption")),
        (Outflow::BurialCost, serde_json::json!("BurialCost")),
        (
            Outflow::Deposit { site: 21 },
            serde_json::json!({"Deposit":{"site":21}}),
        ),
    ] {
        assert_eq!(serde_json::to_value(&kind).unwrap(), value);
        let mut l = Ledger::new(1, 18.0);
        l.prepare(30, 12.0).unwrap();
        l.withdraw(30, 12.0).unwrap();
        l.outflow(3.0, kind.clone()).unwrap();
        let c = &l.cohorts[&30];
        assert_eq!(c.carried, 10.0);
        assert_eq!(l.unlabelled_carried, 5.0);
        match kind {
            Outflow::Consumption => assert_eq!(c.consumed, 2.0),
            Outflow::BurialCost => assert_eq!(c.cost, 2.0),
            Outflow::Deposit { .. } => {
                assert_eq!(c.cached[&21], 2.0);
                assert_eq!(l.unlabelled_cached[&21], 1.0);
            }
            Outflow::ActionCost => unreachable!(),
        }
    }
}
#[test]
fn preparation_rejects_wrong_time_site_or_quantity() {
    for (tick, pos, quantity) in [
        (1, Pos::new(3, 3), 12.0),
        (0, Pos::new(3, 4), 12.0),
        (0, Pos::new(3, 3), 3.0),
    ] {
        let mut w = rig();
        w.tick = tick;
        w.move_agent(1, pos);
        caching::bury(&mut w, 1, quantity);
        assert!(!w.deception.as_ref().unwrap().prepared);
        assert!(w
            .deception
            .as_ref()
            .unwrap()
            .ledger
            .as_ref()
            .unwrap()
            .cohorts
            .is_empty());
    }
}
#[test]
fn starvation_consumption_caps_at_nonnegative_available_food() {
    let mut w = rig();
    // Hand fixture with exactly one labelled carried unit and no unlabelled stock.
    w.agent_mut(1).unwrap().holdings[0] = 1.0;
    let mut l = Ledger::new(1, 1.0);
    l.prepare(30, 1.0).unwrap();
    l.withdraw(30, 1.0).unwrap();
    w.deception.as_mut().unwrap().ledger = Some(l);
    w.agent_mut(1).unwrap().metabolism[0] = 3;
    lifecycle::metabolize(&mut w, 1, Harvest::default());
    assert_eq!(w.agent(1).unwrap().holdings[0], -2.0);
    assert_eq!(
        w.deception
            .as_ref()
            .unwrap()
            .ledger
            .as_ref()
            .unwrap()
            .cohorts[&30]
            .consumed,
        1.0
    );
    assert!(w.deception.as_ref().unwrap().ledger_errors.is_empty());
    assert!(lifecycle::check_death(&mut w, 1));
}

#[test]
fn repeat_bout_after_owner_removal_errors_without_another_cue_or_receipt() {
    let mut w = rig();
    perform_bout(&mut w, 1, BoutKind::Sham, 3.0).unwrap();
    w.remove(1);
    let seen = w.agent(2).unwrap().seen.clone();
    let bouts = w.deception.as_ref().unwrap().bouts.clone();
    assert!(perform_bout(&mut w, 1, BoutKind::Sham, 3.0).is_err());
    assert_eq!(w.agent(2).unwrap().seen, seen);
    assert_eq!(w.deception.as_ref().unwrap().bouts, bouts);
    assert!(w.deception.as_ref().unwrap().ledger_errors.is_empty());
}
