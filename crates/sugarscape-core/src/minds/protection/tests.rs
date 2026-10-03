use super::{controller::perceived_exposure, state::*, tests_support::*};
use std::collections::BTreeMap;
#[test]
fn protection_exposure_expires_only_after_the_inclusive_span() {
    let mut m = ExposureMemory {
        entries: BTreeMap::new(),
    };
    m.remember(30, 0, true);
    m.sweep(2, 2);
    assert!(m.entries.contains_key(&30));
    m.sweep(3, 2);
    assert!(m.entries.is_empty());
}
#[test]
fn protection_visible_nonwatcher_is_a_possible_witness() {
    let mut w = rig(Policy::Selective);
    w.agent_mut(2).unwrap().watches = false;
    assert!(perceived_exposure(&w, 1));
}
#[test]
fn protection_cap_evicts_oldest_then_lowest_site_and_replacement_keeps_size() {
    let cap = crate::minds::memory::MEMORY_CAP;
    let mut m = ExposureMemory::default();
    for site in 0..cap as u32 {
        m.remember(site, 10, true);
    }
    m.remember(0, 11, false);
    assert_eq!(m.entries.len(), cap);
    m.remember(cap as u32, 10, true);
    assert!(!m.entries.contains_key(&1));
    m.remember(2, 0, true);
    m.remember(cap as u32 + 1, 20, true);
    assert!(!m.entries.contains_key(&2));
    assert_eq!(m.entries.len(), cap);
}
#[test]
fn protection_unseen_watcher_is_not_a_local_cue_but_really_sees_burial() {
    let c = super::lab::rig_config(LabConfig {
        fixture: Fixture::CueUnseenWatcher,
        ..LabConfig::default()
    });
    let mut w = crate::world::World::new(c, 7).unwrap();
    assert!(!perceived_exposure(&w, 1));
    crate::minds::caching::bury(&mut w, 1, 12.0);
    assert!(!w.agent(2).unwrap().seen.is_empty());
    assert!(
        !w.agent(1)
            .unwrap()
            .protection
            .as_ref()
            .unwrap()
            .exposure
            .entries[&source_site(&w)]
            .exposed
    );
}
#[test]
fn protection_cue_ignores_private_watcher_state() {
    let mut w = rig(Policy::Selective);
    let before = perceived_exposure(&w, 1);
    let observer = w.agent_mut(2).unwrap();
    observer.watches = false;
    observer.cheater = false;
    observer.vision = 1;
    observer.seen.clear();
    assert_eq!(perceived_exposure(&w, 1), before);
}
#[test]
fn protection_own_agent_is_excluded() {
    let mut w = rig(Policy::Selective);
    w.remove(2);
    assert!(!perceived_exposure(&w, 1));
}
#[test]
fn protection_opaque_wall_blocks_the_local_cue() {
    use crate::{config::Wall, geometry::Pos, testkit::*};
    let mut c = blank_config(9, 9);
    c.walls = vec![Wall {
        x: 3,
        y: 4,
        width: 1,
        height: 1,
        opaque: true,
    }];
    let mut w = crate::world::World::new(c, 7).unwrap();
    let owner = spawn(&mut w, 3, 3);
    w.agent_mut(owner).unwrap().vision = 2;
    spawn(&mut w, 3, 5);
    assert!(!perceived_exposure(&w, owner));
    assert_eq!(w.agent(owner).unwrap().pos, Pos::new(3, 3));
}
#[test]
fn protection_none_config_is_omitted_and_round_trips() {
    let c = crate::config::Config::default();
    let json = serde_json::to_value(&c).unwrap();
    assert!(json.get("protection_lab").is_none());
    assert_eq!(
        serde_json::from_value::<crate::config::Config>(json).unwrap(),
        c
    );
}
#[test]
fn protection_config_and_state_round_trip() {
    let mut w = rig(Policy::Selective);
    crate::minds::caching::bury(&mut w, 1, 12.0);
    let state = w.agent(1).unwrap().protection.as_ref().unwrap();
    assert_eq!(
        serde_json::from_value::<ProtectionState>(serde_json::to_value(state).unwrap()).unwrap(),
        *state
    );
    assert_eq!(
        serde_json::from_value::<crate::config::Config>(serde_json::to_value(&w.config).unwrap())
            .unwrap(),
        w.config
    );
}
#[test]
fn protection_roles_and_finite_resources_exist_before_tick_zero_snapshot() {
    use crate::geometry::Pos;
    for mirrored in [false, true] {
        for fixture in [
            Fixture::Single {
                initial_observed: true,
                redeposit_observed: false,
            },
            Fixture::Single {
                initial_observed: false,
                redeposit_observed: true,
            },
            Fixture::Mixed {
                observed_first: true,
            },
            Fixture::Mixed {
                observed_first: false,
            },
            Fixture::CueVisibleNonwatcher,
            Fixture::CueUnseenWatcher,
            Fixture::Stumble {
                initial_observed: false,
            },
        ] {
            let lab = LabConfig {
                fixture: fixture.clone(),
                mirrored,
                ..LabConfig::default()
            };
            let w = crate::world::World::new(super::lab::rig_config(lab.clone()), 7).unwrap();
            let a = w.agent(1).unwrap();
            let b = w.agent(2).unwrap();
            let start = if matches!(
                fixture,
                Fixture::Mixed {
                    observed_first: false
                }
            ) {
                Pos::new(5, 3)
            } else {
                Pos::new(3, 3)
            };
            let other = match fixture {
                Fixture::Single {
                    initial_observed: false,
                    ..
                }
                | Fixture::Stumble {
                    initial_observed: false,
                } => Pos::new(7, 5),
                Fixture::CueUnseenWatcher => Pos::new(3, 6),
                _ => Pos::new(3, 5),
            };
            assert_eq!(
                (a.pos, b.pos),
                (
                    super::lab::transform(&lab, start),
                    super::lab::transform(&lab, other)
                )
            );
            assert_eq!((a.holdings[0], b.holdings[0]), (44.0, 96.0));
            assert_eq!((a.initial[0], b.initial[0]), (44.0, 96.0));
            assert_eq!(
                (a.metabolism[0], b.metabolism[0], a.vision, b.vision),
                (1, 1, 2, 6)
            );
            assert!(!a.cheater && !a.watches && b.cheater);
            assert_eq!(b.watches, !matches!(fixture, Fixture::CueVisibleNonwatcher));
            assert!(a.protection.is_some() && b.protection.is_none());
            assert_eq!(w.stats.latest().unwrap().population, 2);
            assert_eq!(
                w.sites.iter().map(|s| s.resource[0]).sum::<f64>(),
                if matches!(fixture, Fixture::Mixed { .. }) {
                    0.0
                } else {
                    8.0
                }
            );
        }
    }
}
#[test]
fn protection_registers_only_positive_scheduled_owner_source_deposits() {
    let mut w = rig(Policy::Selective);
    let source = source_site(&w);
    crate::minds::caching::bury(&mut w, 1, 0.0);
    assert!(w
        .agent(1)
        .unwrap()
        .protection
        .as_ref()
        .unwrap()
        .sources
        .is_empty());
    crate::minds::caching::bury(&mut w, 2, 1.0);
    crate::minds::caching::bury(&mut w, 1, 12.0);
    w.tick = 8;
    crate::minds::caching::bury(&mut w, 1, 2.0);
    let state = w.agent(1).unwrap().protection.as_ref().unwrap();
    assert_eq!(state.sources.len(), 1);
    assert_eq!(state.sources[&source].initial_amount, 12.0);
    assert_eq!(state.sources[&source].tick, 0);
}
#[test]
fn protection_invalid_combinations_have_field_specific_errors() {
    use crate::config::*;
    type Change = (&'static str, fn(&mut Config));
    let changes: &[Change] = &[
        ("width", |c| c.width = 10),
        ("height", |c| c.height = 10),
        ("population", |c| c.population = 3),
        ("goods", |c| c.add_good(Good::spice())),
        ("spatial_hoarding.enabled", |c| {
            c.spatial_hoarding.enabled = true
        }),
        ("lab", |c| {
            c.lab = Some(Lab {
                protocol: LabProtocol::Raby,
                food_first: true,
            })
        }),
        ("central.enabled", |c| c.central.enabled = true),
        ("decision.rule", |c| c.decision.rule = DecisionRule::Utility),
        ("movement.mode", |c| c.movement.mode = MoveMode::Jump),
        ("movement.speed", |c| c.movement.speed = 2),
        ("sex.enabled", |c| c.sex.enabled = true),
        ("memory.span", |c| c.memory.span = 1),
        ("caching.mixed", |c| c.caching.mixed = true),
        ("schedule", |c| {
            c.schedule.push(ScheduledChange {
                tick: 1,
                set: BTreeMap::new(),
            })
        }),
        ("protection_lab.reburial_cost", |c| {
            c.protection_lab.as_mut().unwrap().reburial_cost = f64::NAN
        }),
        ("protection_lab.discovery", |c| {
            c.protection_lab.as_mut().unwrap().discovery = 1.1
        }),
        ("protection_lab.exposure_span", |c| {
            c.protection_lab.as_mut().unwrap().exposure_span = 0
        }),
        ("protection_lab.observer_span", |c| {
            c.protection_lab.as_mut().unwrap().observer_span = 0
        }),
    ];
    for (field, change) in changes {
        let mut c = super::lab::rig_config(LabConfig::default());
        change(&mut c);
        assert!(
            c.validate().unwrap_err().iter().any(|e| e.field == *field),
            "{field}"
        );
    }
}
#[test]
fn protection_short_spans_and_varied_cost_discovery_are_valid() {
    for cost in [0.0, 0.25, 2.0] {
        for discovery in [0.0, 0.25, 1.0] {
            super::lab::rig_config(LabConfig {
                exposure_span: 1,
                observer_span: 1,
                reburial_cost: cost,
                discovery,
                ..LabConfig::default()
            })
            .validate()
            .unwrap();
        }
    }
}
#[test]
fn protection_zero_growback_exception_is_bounded_to_the_lab() {
    let mut ordinary = crate::config::Config::default();
    ordinary.growback.rate = 0.0;
    assert!(ordinary
        .validate()
        .unwrap_err()
        .iter()
        .any(|e| e.field == "growback.rate"));
    let mut lab = super::lab::rig_config(LabConfig::default());
    lab.validate().unwrap();
    for rate in [-1.0, f64::NAN, f64::INFINITY] {
        lab.growback.rate = rate;
        assert!(lab
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "growback.rate"));
        ordinary.growback.rate = rate;
        assert!(ordinary
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "growback.rate"));
    }
}
#[test]
fn protection_registration_rejects_wrong_site_and_tracks_mixed_deposit_order() {
    use crate::geometry::Pos;
    for observed_first in [false, true] {
        for mirrored in [false, true] {
            let lab = LabConfig {
                fixture: Fixture::Mixed { observed_first },
                mirrored,
                ..LabConfig::default()
            };
            let mut w = crate::world::World::new(super::lab::rig_config(lab.clone()), 7).unwrap();
            let first = source_site(&w);
            let wrong = w.torus.index(super::lab::transform(&lab, Pos::new(4, 3))) as u32;
            super::lab::note_prepared_deposit(&mut w, 1, wrong, 6.0);
            assert!(w
                .agent(1)
                .unwrap()
                .protection
                .as_ref()
                .unwrap()
                .sources
                .is_empty());
            crate::minds::caching::bury(&mut w, 1, 6.0);
            w.tick = 2;
            let other =
                super::lab::transform(&lab, Pos::new(if observed_first { 5 } else { 3 }, 3));
            w.move_agent(1, other);
            crate::minds::caching::bury(&mut w, 1, 6.0);
            let state = w.agent(1).unwrap().protection.as_ref().unwrap();
            assert_eq!(state.sources.len(), 2);
            assert_eq!(state.sources[&first].tick, 0);
            assert_eq!(state.sources[&(w.torus.index(other) as u32)].tick, 2);
            assert_eq!(state.exposure.entries[&first].exposed, observed_first);
            assert_eq!(
                state.exposure.entries[&(w.torus.index(other) as u32)].exposed,
                !observed_first
            );
        }
    }
}
#[test]
fn protection_constructor_requires_zero_initial_burial_cost() {
    let mut c = super::lab::rig_config(LabConfig::default());
    c.caching.bury_cost = 0.25;
    assert!(c
        .validate()
        .unwrap_err()
        .iter()
        .any(|e| e.field == "caching.bury_cost"));
}
