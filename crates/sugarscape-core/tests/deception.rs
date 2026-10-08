//! Checked P4 construction and authoritative persistence, construction seeds only.
use sugarscape_core::{
    config::{Config, ScheduledChange},
    geometry::Pos,
    minds::deception::{lab::rig_config, state::*},
    world::World,
};

#[test]
fn all_sender_stages_round_trip() {
    for stage in [
        Stage::Preparation,
        Stage::ToDisplay,
        Stage::Display,
        Stage::Return,
        Stage::Ordinary,
        Stage::Departure,
        Stage::Cancelled,
    ] {
        let state = SenderState {
            stage,
            attempted: true,
            pending_departure: true,
        };
        assert_eq!(
            serde_json::from_str::<SenderState>(&serde_json::to_string(&state).unwrap()).unwrap(),
            state
        );
    }
}
#[test]
fn lab_config_round_trips_and_rejects_unknown_fields() {
    let lab = LabConfig {
        sender: SenderPolicy::MatchedNeutral,
        view: View::Clear,
        display_seen: false,
        layout: Layout::OnRoute,
        effort_cost: 3.0,
        mirrored: true,
    };
    let value = serde_json::to_value(&lab).unwrap();
    assert_eq!(
        value,
        serde_json::json!({"sender":"matched_neutral","view":"clear","display_seen":false,"layout":"on_route","effort_cost":3.0,"mirrored":true})
    );
    assert_eq!(
        serde_json::from_value::<LabConfig>(value.clone()).unwrap(),
        lab
    );
    let mut extra = value;
    extra["private_stock"] = 12.into();
    assert!(serde_json::from_value::<LabConfig>(extra).is_err());
}
#[test]
fn runtime_round_trips() {
    let runtime = Runtime {
        source: 30,
        display: 59,
        prepared: false,
        diagnostics: true,
        ledger: None,
        ledger_errors: vec![],
        bouts: vec![],
        sham_bouts_seen: 0,
        sham_sightings: 0,
        source_recovered: false,
        actions: vec![],
        observations: vec![],
        choices: vec![],
        deaths: vec![],
        fixture_errors: vec![],
        restrictions: Default::default(),
    };
    assert_eq!(
        serde_json::from_str::<Runtime>(&serde_json::to_string(&runtime).unwrap()).unwrap(),
        runtime
    );
}
#[test]
fn default_off_json_and_fingerprint_remain_compatible() {
    let c = Config::default();
    let value = serde_json::to_value(&c).unwrap();
    assert!(value.get("deception_lab").is_none());
    let old: Config = serde_json::from_value(value).unwrap();
    assert!(old.deception_lab.is_none());
    let w = World::new(old, 7).unwrap();
    assert!(w.deception.is_none());
    assert!(w.agents().all(|a| a.deception.is_none()));
    let mut hidden = w.clone();
    hidden.agent_mut(1).unwrap().deception = Some(SenderState::default());
    hidden.deception = Some(Runtime {
        source: 30,
        display: 59,
        prepared: true,
        diagnostics: true,
        ledger: None,
        ledger_errors: vec![],
        bouts: vec![],
        sham_bouts_seen: 0,
        sham_sightings: 0,
        source_recovered: false,
        actions: vec![],
        observations: vec![],
        choices: vec![],
        deaths: vec![],
        fixture_errors: vec![],
        restrictions: Default::default(),
    });
    assert_eq!(w.fingerprint(), hidden.fingerprint());
}
#[test]
fn construction_has_fixed_roles_finite_patches_and_no_initial_deposit() {
    for (seed, mirrored) in [(7, false), (8, false), (7, true), (8, true)] {
        let w = World::new(
            rig_config(LabConfig {
                mirrored,
                ..Default::default()
            }),
            seed,
        )
        .unwrap();
        let owner = w.agent(1).unwrap();
        let observer = w.agent(2).unwrap();
        assert_eq!(owner.pos, Pos::new(if mirrored { 5 } else { 3 }, 3));
        assert_eq!(observer.pos, Pos::new(if mirrored { 5 } else { 3 }, 6));
        assert_eq!((owner.holdings[0], observer.holdings[0]), (44.0, 96.0));
        assert_eq!((owner.vision, observer.vision), (2, 6));
        assert_eq!((owner.metabolism[0], observer.metabolism[0]), (1, 1));
        assert!(!owner.cheater && !owner.watches && observer.cheater && observer.watches);
        assert!(!owner.remembers && !observer.remembers);
        assert_eq!((owner.rate, observer.rate), (1.0, 1.0));
        assert_eq!(owner.deception, Some(SenderState::default()));
        assert!(observer.deception.is_none());
        assert!(owner.caches.is_empty() && observer.caches.is_empty());
        assert!(observer.seen.is_empty());
        let patches: Vec<_> = w
            .sites
            .iter()
            .enumerate()
            .filter(|(_, s)| s.resource[0] > 0.0)
            .map(|(i, s)| (i as u32, s.resource[0], s.capacity[0]))
            .collect();
        assert_eq!(
            patches,
            if mirrored {
                vec![(24, 4.0, 4.0), (33, 4.0, 4.0)]
            } else {
                vec![(20, 4.0, 4.0), (29, 4.0, 4.0)]
            }
        );
        let runtime = w.deception.as_ref().unwrap();
        assert_eq!(runtime.source, if mirrored { 32 } else { 30 });
        assert_eq!(runtime.display, if mirrored { 57 } else { 59 });
        assert!(!runtime.prepared);
    }
}
#[test]
fn checked_rig_rejects_unsupported_costs_and_lab_combinations() {
    for effort_cost in [-1.0, 1.0, f64::NAN, f64::INFINITY] {
        assert!(rig_config(LabConfig {
            effort_cost,
            ..Default::default()
        })
        .validate()
        .is_err());
    }
    let mut c = rig_config(LabConfig::default());
    c.protection_lab = Some(Default::default());
    assert!(c.validate().is_err());
    c.protection_lab = None;
    c.lab = Some(sugarscape_core::config::Lab {
        protocol: sugarscape_core::config::LabProtocol::Raby,
        food_first: true,
    });
    assert!(c.validate().is_err());
}
#[test]
fn lab_option_is_reset_only_for_hot_and_scheduled_changes() {
    let c = rig_config(LabConfig::default());
    let mut next = c.clone();
    next.deception_lab.as_mut().unwrap().view = View::Clear;
    assert!(c
        .structural_changes(&next)
        .iter()
        .any(|e| e.field == "deception_lab"));
    let change = ScheduledChange {
        tick: 1,
        set: std::collections::BTreeMap::from([(
            "deception_lab.view".into(),
            serde_json::json!("clear"),
        )]),
    };
    assert!(c.apply_change(&change).is_err());
}
#[test]
fn fingerprint_covers_active_config_owner_memory_and_runtime() {
    let base = World::new(rig_config(LabConfig::default()), 7).unwrap();
    type Mutation = (&'static str, fn(&mut World));
    let changes: &[Mutation] = &[
        ("sender", |w| {
            w.config.deception_lab.as_mut().unwrap().sender = SenderPolicy::Sham
        }),
        ("view", |w| {
            w.config.deception_lab.as_mut().unwrap().view = View::Clear
        }),
        ("seen", |w| {
            w.config.deception_lab.as_mut().unwrap().display_seen = false
        }),
        ("layout", |w| {
            w.config.deception_lab.as_mut().unwrap().layout = Layout::OnRoute
        }),
        ("cost", |w| {
            w.config.deception_lab.as_mut().unwrap().effort_cost = 3.0
        }),
        ("mirror", |w| {
            w.config.deception_lab.as_mut().unwrap().mirrored = true
        }),
        ("stage", |w| {
            w.agent_mut(1).unwrap().deception.as_mut().unwrap().stage = Stage::Display
        }),
        ("attempted", |w| {
            w.agent_mut(1)
                .unwrap()
                .deception
                .as_mut()
                .unwrap()
                .attempted = true
        }),
        ("departure", |w| {
            w.agent_mut(1)
                .unwrap()
                .deception
                .as_mut()
                .unwrap()
                .pending_departure = true
        }),
        ("source", |w| w.deception.as_mut().unwrap().source += 1),
        ("display", |w| w.deception.as_mut().unwrap().display += 1),
        ("prepared", |w| {
            w.deception.as_mut().unwrap().prepared = true
        }),
        ("memory", |w| {
            w.agent_mut(2).unwrap().seen.insert(
                (30, 1),
                sugarscape_core::minds::caching::watching::SeenCache {
                    amount: 12.0,
                    tick: 0,
                },
            );
        }),
    ];
    for (name, change) in changes {
        let mut altered = base.clone();
        change(&mut altered);
        assert_ne!(base.fingerprint(), altered.fingerprint(), "{name}");
    }
    let mut diagnostic = base.clone();
    diagnostic.deception.as_mut().unwrap().diagnostics =
        !diagnostic.deception.as_ref().unwrap().diagnostics;
    assert_eq!(base.fingerprint(), diagnostic.fingerprint());
}

#[test]
fn checked_construction_rejects_changes_to_the_fixed_rig() {
    type Mutation = fn(&mut Config);
    let changes: &[Mutation] = &[
        |c| c.width = 10,
        |c| c.height = 10,
        |c| c.population = 3,
        |c| c.walls[0].opaque = false,
        |c| c.goods.push(c.goods[0].clone()),
        |c| c.growback.rate = 1.0,
        |c| c.growback.instant = true,
        |c| c.caching.capacity = 127,
        |c| c.goap.horizon = 3,
        |c| c.memory.span = 63,
        |c| c.watching.span = 63,
        |c| c.theft.find = 0.5,
        |c| c.theft.cheaters = 1.0,
        |c| c.caching.rule = sugarscape_core::config::CachingRule::Even,
        |c| c.sex.enabled = true,
        |c| c.replacement.enabled = true,
        |c| c.central.enabled = true,
        |c| c.spatial_hoarding.enabled = true,
        |c| c.movement.speed = 2,
    ];
    for change in changes {
        let mut c = rig_config(LabConfig::default());
        change(&mut c);
        assert!(c.validate().is_err());
    }
}
#[test]
fn active_config_json_restores_checked_construction() {
    let c = rig_config(LabConfig::default());
    let restored: Config = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
    assert_eq!(c, restored);
    assert_eq!(
        World::new(c, 7).unwrap().fingerprint(),
        World::new(restored, 7).unwrap().fingerprint()
    );
}

#[test]
fn layouts_reflect_every_runtime_site() {
    for (layout, mirrored, expected) in [
        (Layout::OnRoute, false, (30, 48)),
        (Layout::OnRoute, true, (32, 50)),
        (Layout::OffRoute, false, (30, 59)),
        (Layout::OffRoute, true, (32, 57)),
    ] {
        let w = World::new(
            rig_config(LabConfig {
                layout,
                mirrored,
                ..Default::default()
            }),
            7,
        )
        .unwrap();
        let runtime = w.deception.as_ref().unwrap();
        assert_eq!((runtime.source, runtime.display), expected);
    }
}
#[test]
fn fingerprint_covers_each_seen_and_cache_age_field() {
    let mut base = World::new(rig_config(LabConfig::default()), 7).unwrap();
    base.agent_mut(2).unwrap().seen.insert(
        (30, 1),
        sugarscape_core::minds::caching::watching::SeenCache {
            amount: 12.0,
            tick: 0,
        },
    );
    base.agent_mut(1).unwrap().cache_since.insert(30, 0);
    type Mutation = (&'static str, fn(&mut World));
    let changes: &[Mutation] = &[
        ("seen amount", |w| {
            w.agent_mut(2)
                .unwrap()
                .seen
                .get_mut(&(30, 1))
                .unwrap()
                .amount = 11.0
        }),
        ("seen tick", |w| {
            w.agent_mut(2).unwrap().seen.get_mut(&(30, 1)).unwrap().tick = 1
        }),
        ("seen site", |w| {
            let entry = w.agent_mut(2).unwrap().seen.remove(&(30, 1)).unwrap();
            w.agent_mut(2).unwrap().seen.insert((31, 1), entry);
        }),
        ("seen actor", |w| {
            let entry = w.agent_mut(2).unwrap().seen.remove(&(30, 1)).unwrap();
            w.agent_mut(2).unwrap().seen.insert((30, 2), entry);
        }),
        ("cache age", |w| {
            w.agent_mut(1).unwrap().cache_since.insert(30, 1);
        }),
        ("runtime absence", |w| w.deception = None),
        ("owner state absence", |w| {
            w.agent_mut(1).unwrap().deception = None
        }),
    ];
    for (name, change) in changes {
        let mut altered = base.clone();
        change(&mut altered);
        assert_ne!(base.fingerprint(), altered.fingerprint(), "{name}");
    }
}
