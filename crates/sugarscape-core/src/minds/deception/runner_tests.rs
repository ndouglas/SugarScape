use super::{
    lab::{condition_id, conditions, rig_config, transform},
    runner::*,
    state::*,
};
use crate::{geometry::Pos, world::World};
use std::collections::BTreeSet;

#[test]
fn fixed_conditions_have_complete_unique_identities() {
    let configs = conditions();
    let ids: BTreeSet<_> = configs.iter().map(condition_id).collect();
    assert_eq!(ids.len(), 96);
    let strata: BTreeSet<_> = configs
        .iter()
        .map(|c| {
            let mut c = c.clone();
            c.sender = SenderPolicy::Ordinary;
            condition_id(&c)
        })
        .collect();
    assert_eq!(strata.len(), 32);
    for c in configs {
        assert!(rig_config(c).validate().is_ok());
    }
}
#[test]
fn reflection_is_pure_and_involutive_for_every_declared_waypoint() {
    let c = LabConfig {
        mirrored: true,
        ..LabConfig::default()
    };
    for p in [
        Pos::new(3, 3),
        Pos::new(3, 6),
        Pos::new(2, 2),
        Pos::new(2, 3),
        Pos::new(3, 2),
    ]
    .into_iter()
    .chain(owner_route(Layout::OnRoute, true))
    .chain(owner_route(Layout::OnRoute, false))
    .chain(owner_route(Layout::OffRoute, true))
    .chain(owner_route(Layout::OffRoute, false))
    .chain(observer_route(true))
    .chain(observer_route(false))
    {
        assert_eq!(transform(&c, p), Pos::new(8 - p.x, p.y));
        assert_eq!(transform(&c, transform(&c, p)), p);
    }
}
fn physical(e: &super::records::EpisodeRecord, through: usize) -> serde_json::Value {
    serde_json::json!(e.frames.iter().take(through).map(|f| serde_json::json!({"tick":f.tick,"roles":f.roles.iter().map(|r| serde_json::json!({"id":r.id,"pos":r.pos,"holdings":r.holdings,"caches":r.caches})).collect::<Vec<_>>(),"actions":f.actions.iter().map(|a| serde_json::json!({"actor":a.actor,"target":a.target,"harvest":a.harvest,"dug":a.dug,"buried":a.buried,"effort":a.effort,"demand":a.metabolic_demand,"consumed":a.metabolic_consumed})).collect::<Vec<_>>(),"deaths":f.deaths,"stocks":f.stocks})).collect::<Vec<_>>())
}
#[test]
fn neutral_and_sham_are_physically_matched_until_receiver_release() {
    for c in conditions()
        .into_iter()
        .filter(|c| !c.mirrored && c.sender == SenderPolicy::MatchedNeutral)
    {
        for seed in [7, 8] {
            let n = run_episode(c.clone(), seed, true).unwrap();
            let mut c = c.clone();
            c.sender = SenderPolicy::Sham;
            let s = run_episode(c, seed, true).unwrap();
            assert_eq!(physical(&n, 33), physical(&s, 33));
        }
    }
}
#[test]
fn ordinary_effort_labels_are_effective_aliases() {
    for c in conditions()
        .into_iter()
        .filter(|c| !c.mirrored && c.sender == SenderPolicy::Ordinary && c.effort_cost == 0.0)
    {
        let a = run_episode(c.clone(), 7, true).unwrap();
        let mut c = c;
        c.effort_cost = 3.0;
        let b = run_episode(c, 7, true).unwrap();
        assert_eq!(physical(&a, 65), physical(&b, 65));
    }
}
#[test]
fn routes_and_clear_preparation_have_actual_opportunity() {
    for layout in [Layout::OnRoute, Layout::OffRoute] {
        for seen in [true, false] {
            let c = LabConfig {
                sender: SenderPolicy::Sham,
                layout,
                display_seen: seen,
                ..LabConfig::default()
            };
            let e = run_episode(c, 7, true).unwrap();
            let prep = &e.frames[1];
            assert_eq!(prep.observations.len(), 1);
            assert_eq!(prep.observations[0].actual_transfer, 12.0);
            assert!(matches!(
                prep.observations[0].public.signal,
                super::observation::Signal::VisibleTransfer { amount: 12.0 }
            ));
            let display = &e.frames[14];
            assert_eq!(display.observations.len(), usize::from(seen));
            assert_eq!(
                display.roles.iter().find(|r| r.id == 1).unwrap().pos,
                if layout == Layout::OnRoute {
                    Pos::new(3, 5)
                } else {
                    Pos::new(5, 6)
                }
            );
            let release = &e.frames[32];
            assert_eq!(
                release.roles.iter().find(|r| r.id == 2).unwrap().pos,
                Pos::new(3, 6)
            );
            assert!(release.roles.iter().all(|r| r.pos != Pos::new(3, 3)
                && r.pos != Pos::new(3, 5)
                && r.pos != Pos::new(5, 6)));
            assert!(e.fixture_errors.is_empty(), "{:?}", e.fixture_errors);
        }
    }
}
#[test]
fn preparation_and_observer_walks_gather_zero_and_pay_metabolism() {
    let e = run_episode(
        LabConfig {
            sender: SenderPolicy::Sham,
            display_seen: false,
            ..LabConfig::default()
        },
        7,
        true,
    )
    .unwrap();
    for f in e.frames.iter().skip(1).take(32) {
        for a in &f.actions {
            if a.actor == 2 || f.tick <= 8 {
                assert_eq!(a.harvest, 0.0);
                assert_eq!(a.dug, 0.0);
                assert_eq!(a.metabolic_demand, 1.0);
                assert_eq!(a.metabolic_consumed, 1.0);
            }
        }
    }
    assert_eq!(
        e.frames[1]
            .roles
            .iter()
            .find(|r| r.id == 1)
            .unwrap()
            .holdings,
        31.0
    );
}
#[test]
fn diagnostics_and_sink_cannot_change_biology() {
    let c = LabConfig {
        sender: SenderPolicy::Sham,
        effort_cost: 3.0,
        ..LabConfig::default()
    };
    let a = run_episode(c.clone(), 7, true).unwrap();
    let b = run_episode_with_sink(c, 7, false, |_| Ok(())).unwrap();
    assert_eq!(
        a.frames.iter().map(|f| &f.fingerprint).collect::<Vec<_>>(),
        b.frames.iter().map(|f| &f.fingerprint).collect::<Vec<_>>()
    );
    assert_eq!(physical(&a, 65), physical(&b, 65));
    assert!(b.cohorts.is_none());
    assert!(b
        .lineage_unavailable_reason
        .as_ref()
        .is_some_and(|s| !s.trim().is_empty()));
}
#[test]
fn sink_failure_preserves_initial_and_completed_frames() {
    let mut calls = 0;
    let err = run_episode_with_sink(LabConfig::default(), 7, true, |_| {
        calls += 1;
        if calls == 3 {
            Err("test sink".into())
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    assert!(err.message.contains("test sink"));
    let e = err.partial.unwrap();
    assert_eq!(e.completed_ticks, 2);
    assert_eq!(e.frames.len(), 3);
    let err = run_episode_with_sink(LabConfig::default(), 7, true, |_| Err("initial".into()))
        .unwrap_err();
    assert_eq!(err.partial.unwrap().frames.len(), 1);
}
#[test]
fn common_departure_follows_successful_source_recovery() {
    for sender in [
        SenderPolicy::Ordinary,
        SenderPolicy::MatchedNeutral,
        SenderPolicy::Sham,
    ] {
        let mut w = World::new(
            rig_config(LabConfig {
                sender,
                ..LabConfig::default()
            }),
            7,
        )
        .unwrap();
        crate::minds::caching::bury(&mut w, 1, 12.0);
        w.tick = 32;
        w.agent_mut(1).unwrap().holdings[0] = 3.0;
        w.deception.as_mut().unwrap().diagnostics = false;
        w.deception.as_mut().unwrap().ledger = None;
        crate::rules::agent_turn(&mut w, 1);
        assert_eq!(
            w.deception.as_ref().unwrap().actions.last().unwrap().dug,
            12.0
        );
        w.tick = 33;
        crate::rules::agent_turn(&mut w, 1);
        let r = w.deception.as_ref().unwrap();
        let a = r.actions.last().unwrap();
        assert_eq!(a.action, "departure");
        assert_eq!(a.target, Some(Pos::new(3, 2)));
        assert_eq!(a.walk_outcome.as_deref(), Some("arrived"));
        assert_eq!(w.agent(1).unwrap().pos, Pos::new(3, 2));
    }
}
#[test]
fn owner_removal_cancels_the_unattempted_scheduled_bout_in_research_record() {
    let mut w = World::new(
        rig_config(LabConfig {
            sender: SenderPolicy::Sham,
            ..LabConfig::default()
        }),
        7,
    )
    .unwrap();
    w.remove(1);
    let r = w.deception.as_ref().unwrap();
    assert_eq!(r.bouts.len(), 1);
    assert_eq!(r.bouts[0].result, super::controller::BoutResult::OwnerDied);
    assert_eq!(
        r.actions.last().unwrap().cancellation,
        Some(super::controller::BoutResult::OwnerDied)
    );
    assert_eq!(r.deaths.len(), 1);
    w.remove(1);
    assert_eq!(w.deception.as_ref().unwrap().bouts.len(), 1);
}
#[test]
fn occupied_waypoint_cancels_without_display_effort_or_emission() {
    let mut w = World::new(
        rig_config(LabConfig {
            sender: SenderPolicy::Sham,
            layout: Layout::OnRoute,
            effort_cost: 3.0,
            ..LabConfig::default()
        }),
        7,
    )
    .unwrap();
    w.tick = 8;
    w.move_agent(2, Pos::new(3, 4));
    w.step();
    let r = w.deception.as_ref().unwrap();
    let a = r.actions.iter().find(|a| a.actor == 1).unwrap();
    assert_eq!(
        a.cancellation,
        Some(super::controller::BoutResult::Occupied)
    );
    assert_eq!(a.effort, 0.0);
    assert!(r.observations.is_empty());
    assert_eq!(w.agent(1).unwrap().pos, Pos::new(3, 3));
}
#[test]
fn empty_and_truthful_inspections_capture_actual_choice_and_raid() {
    for actual in [0.0, 12.0] {
        let mut w = World::new(rig_config(LabConfig::default()), 7).unwrap();
        crate::minds::caching::bury(&mut w, 1, 12.0);
        if actual == 0.0 {
            let site = w.deception.as_ref().unwrap().source;
            crate::minds::caching::dig(&mut w, 1, site, 12.0);
        }
        w.tick = 32;
        w.move_agent(1, Pos::new(2, 3));
        w.move_agent(2, Pos::new(3, 4));
        w.deception.as_mut().unwrap().diagnostics = false;
        w.deception.as_mut().unwrap().ledger = None;
        crate::rules::agent_turn(&mut w, 2);
        let c = w.deception.as_ref().unwrap().choices.last().unwrap();
        assert_eq!(c.target, Pos::new(3, 3));
        assert_eq!(c.remembered_value, 12.0);
        assert_eq!(c.actual_value, actual);
        assert_eq!(c.inspected_stock, Some(actual));
        assert!(c.arrived);
        assert_eq!(c.raid_amount, actual);
        assert_eq!(c.wasted, actual == 0.0);
    }
}
#[test]
fn expired_evidence_does_not_select_a_remote_cache() {
    let mut w = World::new(rig_config(LabConfig::default()), 7).unwrap();
    crate::minds::caching::bury(&mut w, 1, 12.0);
    w.tick = 65;
    w.step();
    assert!(w.agent(2).unwrap().seen.is_empty());
    let c = w
        .deception
        .as_ref()
        .unwrap()
        .choices
        .iter()
        .find(|c| c.actor == 2)
        .unwrap();
    assert_ne!(c.target, Pos::new(3, 3));
}
#[test]
fn all_base_construction_cells_have_valid_fixed_fixtures() {
    let mut evidence = Vec::new();
    for c in conditions().into_iter().filter(|c| !c.mirrored) {
        for seed in [7, 8] {
            let e = run_episode(c.clone(), seed, true).unwrap();
            assert!(
                e.fixture_errors.is_empty(),
                "{} seed {seed}: {:?}",
                condition_id(&c),
                e.fixture_errors
            );
            assert!(e.ledger_errors.is_empty(), "{:?}", e.ledger_errors);
            let initial = &e.frames[0];
            assert_eq!(initial.roles[0].holdings, 44.0);
            assert_eq!(initial.roles[1].holdings, 96.0);
            assert_eq!(initial.stocks.iter().map(|s| s.amount).sum::<f64>(), 8.0);
            assert_eq!(e.frames[1].roles[0].holdings, 31.0);
            assert_eq!(e.frames[1].roles[1].holdings, 95.0);
            assert_eq!(e.frames[1].observations.len(), 1);
            assert_eq!(e.frames[1].observations[0].actual_stock, 12.0);
            let display = if c.layout == Layout::OnRoute {
                Pos::new(3, 5)
            } else {
                Pos::new(5, 6)
            };
            assert!(e.frames[32]
                .roles
                .iter()
                .all(|r| r.pos != Pos::new(3, 3) && r.pos != display));
            assert_eq!(e.frames[32].roles[1].pos, Pos::new(3, 6));
            if c.sender != SenderPolicy::Ordinary {
                let a = e.frames[14].actions.iter().find(|a| a.actor == 1).unwrap();
                assert_eq!(a.effort, c.effort_cost);
                assert_eq!(a.pos, Some(display));
                assert_eq!(
                    e.frames[14].observations.len(),
                    usize::from(c.sender == SenderPolicy::Sham && c.display_seen)
                );
            }
            evidence.push(serde_json::json!({"condition":condition_id(&c),"seed":seed,"initial":e.frames[0],"preparation":e.frames[1],"display":e.frames[14],"release":e.frames[32],"route_frames":e.frames[9..=20],"first_receiver_turn":e.frames[33],"fixture_errors":e.fixture_errors,"ledger_errors":e.ledger_errors}));
        }
    }
    assert_eq!(evidence.len(), 96);
    if let Ok(path) = std::env::var("SUGARSCAPE_P4_CONSTRUCTION_EVIDENCE") {
        std::fs::write(path, serde_json::to_vec_pretty(&evidence).unwrap()).unwrap();
    }
}

#[test]
fn restrictions_count_every_supplied_turn_including_walks_and_display() {
    for sender in [
        SenderPolicy::Ordinary,
        SenderPolicy::MatchedNeutral,
        SenderPolicy::Sham,
    ] {
        let e = run_episode(
            LabConfig {
                sender,
                display_seen: false,
                ..LabConfig::default()
            },
            7,
            true,
        )
        .unwrap();
        assert_eq!(e.frames[32].restrictions.get(&2), Some(&32));
        assert_eq!(
            e.frames[32].restrictions.get(&1),
            Some(&if sender == SenderPolicy::Ordinary {
                8
            } else {
                20
            })
        );
    }
}
#[test]
fn unreachable_waypoint_retains_a_cancellation_receipt() {
    let mut w = World::new(
        rig_config(LabConfig {
            sender: SenderPolicy::Sham,
            layout: Layout::OnRoute,
            ..LabConfig::default()
        }),
        7,
    )
    .unwrap();
    let i = w.torus.index(Pos::new(3, 4));
    w.walls[i] = 2;
    w.regions[i] = u32::MAX;
    w.tick = 8;
    w.step();
    let r = w.deception.as_ref().unwrap();
    let a = r.actions.iter().find(|a| a.actor == 1).unwrap();
    assert_eq!(
        a.cancellation,
        Some(super::controller::BoutResult::Unreachable)
    );
    assert_eq!(a.effort, 0.0);
    assert_eq!(a.walk_outcome.as_deref(), Some("unreachable"));
    assert!(r.observations.is_empty());
}
#[test]
fn blocked_departure_records_occupancy_after_actual_recovery_and_pays_metabolism() {
    for sender in [
        SenderPolicy::Ordinary,
        SenderPolicy::MatchedNeutral,
        SenderPolicy::Sham,
    ] {
        let mut w = World::new(
            rig_config(LabConfig {
                sender,
                ..LabConfig::default()
            }),
            7,
        )
        .unwrap();
        crate::minds::caching::bury(&mut w, 1, 12.0);
        w.tick = 32;
        w.agent_mut(1).unwrap().holdings[0] = 3.0;
        w.deception.as_mut().unwrap().diagnostics = false;
        w.deception.as_mut().unwrap().ledger = None;
        crate::rules::agent_turn(&mut w, 1);
        w.move_agent(2, Pos::new(3, 2));
        let held = w.agent(1).unwrap().holdings[0];
        w.tick = 33;
        crate::rules::agent_turn(&mut w, 1);
        let a = w.deception.as_ref().unwrap().actions.last().unwrap();
        assert_eq!(a.action, "departure");
        assert_eq!(a.walk_outcome.as_deref(), Some("blocked"));
        assert_eq!(a.target_occupant, Some(2));
        assert!(a.source_recovered);
        assert_eq!(w.agent(1).unwrap().holdings[0], held - 1.0);
        assert!(
            !w.agent(1)
                .unwrap()
                .deception
                .as_ref()
                .unwrap()
                .pending_departure
        );
    }
}
#[test]
fn on_route_decoy_need_not_add_a_receiver_movement_step() {
    let c = LabConfig {
        sender: SenderPolicy::MatchedNeutral,
        layout: Layout::OnRoute,
        ..LabConfig::default()
    };
    let n = run_episode(c.clone(), 7, true).unwrap();
    let mut c = c;
    c.sender = SenderPolicy::Sham;
    let s = run_episode(c, 7, true).unwrap();
    assert_eq!(
        n.frames[33].roles.iter().find(|r| r.id == 2).unwrap().pos,
        Pos::new(3, 5)
    );
    assert_eq!(
        s.frames[33].roles.iter().find(|r| r.id == 2).unwrap().pos,
        Pos::new(3, 5)
    );
}

#[test]
fn completed_display_survives_same_turn_starvation_as_completed_evidence() {
    let mut w = World::new(
        rig_config(LabConfig {
            sender: SenderPolicy::Sham,
            layout: Layout::OnRoute,
            effort_cost: 3.0,
            ..LabConfig::default()
        }),
        7,
    )
    .unwrap();
    w.move_agent(1, Pos::new(3, 5));
    w.tick = 13;
    w.agent_mut(1).unwrap().holdings[0] = 3.0;
    w.agent_mut(1).unwrap().deception.as_mut().unwrap().stage = Stage::Display;
    w.deception.as_mut().unwrap().ledger =
        Some(crate::minds::protection::ledger::Ledger::new(1, 3.0));
    crate::rules::agent_turn(&mut w, 1);
    let r = w.deception.as_ref().unwrap();
    assert!(w.agent(1).is_none());
    assert_eq!(r.bouts.len(), 1);
    assert_eq!(r.bouts[0].result, super::controller::BoutResult::Completed);
    assert_eq!(r.actions.last().unwrap().action, "sham");
    assert_eq!(r.actions.last().unwrap().effort, 3.0);
    assert_eq!(r.actions.last().unwrap().cancellation, None);
    assert_eq!(r.observations.len(), 1);
    assert_eq!(r.deaths.len(), 1);
    assert_eq!(r.deaths[0].cause, "starvation");
}
