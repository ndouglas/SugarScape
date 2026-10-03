//! Fixed episode execution and researcher-only records.
use super::{lab, ledger::Ledger, state::*};
use crate::{agent::AgentId, geometry::Pos, world::World};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpisodeRecord {
    pub schema: String,
    pub lab: LabConfig,
    pub seed: u64,
    pub requested_ticks: u64,
    pub completed_ticks: u64,
    pub owner_ticks_alive: u64,
    pub owner_alive: bool,
    pub ledger_errors: Vec<String>,
    pub thief_transferred: Option<f64>,
    pub cohorts: Option<Ledger>,
    pub frames: Vec<FrameRecord>,
    pub fixture_errors: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FrameRecord {
    pub tick: u64,
    pub fingerprint: String,
    pub roles: Vec<RoleRecord>,
    pub relocation: RelocationEvents,
    pub deaths: Vec<DeathRecord>,
    pub actions: Vec<ActionRecord>,
    pub current_bury_cost: f64,
    pub restriction_ticks: BTreeMap<AgentId, u64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RoleRecord {
    pub id: AgentId,
    pub pos: Pos,
    pub holdings: f64,
    pub caches: BTreeMap<u32, f64>,
    pub protection: Option<ProtectionState>,
    pub seen: Vec<SeenRecord>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeenRecord {
    pub site: u32,
    pub owner: AgentId,
    pub amount: f64,
    pub tick: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DeathRecord {
    pub id: AgentId,
    pub pos: Pos,
    pub cause: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionRecord {
    pub id: AgentId,
    pub phase: String,
    pub action: String,
    pub harvest: f64,
    pub metabolic_demand: f64,
    pub metabolic_consumed: f64,
    pub gross_dug: f64,
    pub gross_buried: f64,
    pub burial_cost: f64,
    pub source: Option<u32>,
    pub target: Option<Pos>,
    pub perceived_exposure: Option<bool>,
    pub actual_watchers: Vec<AgentId>,
    pub raid_site: Option<u32>,
    pub raid_amount: f64,
    pub raid_wasted: bool,
    pub discovery_site: Option<u32>,
    pub discovery_amount: f64,
}
fn frame(world: &World) -> FrameRecord {
    FrameRecord {
        tick: world.tick,
        fingerprint: format!("{:016x}", world.fingerprint()),
        roles: world
            .agents()
            .map(|a| RoleRecord {
                id: a.id,
                pos: a.pos,
                holdings: a.holdings[0],
                caches: a.caches.clone(),
                protection: a.protection.clone(),
                seen: a
                    .seen
                    .iter()
                    .map(|(&(site, owner), e)| SeenRecord {
                        site,
                        owner,
                        amount: e.amount,
                        tick: e.tick,
                    })
                    .collect(),
            })
            .collect(),
        relocation: world.relocation_events.clone().unwrap_or_default(),
        deaths: world.protection_deaths.clone(),
        actions: world.protection_actions.clone(),
        current_bury_cost: world.config.caching.bury_cost,
        restriction_ticks: world.protection_restrictions.clone(),
    }
}
pub fn run_episode(
    lab: LabConfig,
    seed: u64,
    collect_ledger: bool,
) -> Result<EpisodeRecord, String> {
    let mut world = World::new(lab::rig_config(lab.clone()), seed).map_err(|e| format!("{e:?}"))?;
    if collect_ledger {
        world.protection_ledger = Some(Ledger::new(1, 44.0));
    }
    let mut frames = vec![frame(&world)];
    let mut owner_ticks_alive = 0;
    for _ in 0..64 {
        if world.population() == 0 {
            break;
        }
        owner_ticks_alive += u64::from(world.agent(1).is_some());
        world.step();
        frames.push(frame(&world));
    }
    let owner_alive = world.agent(1).is_some();
    check_fixture(&lab, &frames, &mut world.protection_fixture_errors);
    let cohorts = world.protection_ledger;
    let thief_transferred = cohorts
        .as_ref()
        .map(|l| l.cohorts.values().map(|c| c.transferred).sum());
    Ok(EpisodeRecord {
        schema: "minds-protection-measured-v1".into(),
        lab,
        seed,
        requested_ticks: 64,
        completed_ticks: world.tick,
        owner_ticks_alive,
        owner_alive,
        ledger_errors: world.protection_ledger_errors,
        thief_transferred,
        cohorts,
        frames,
        fixture_errors: world.protection_fixture_errors,
    })
}
fn check_fixture(lab: &LabConfig, frames: &[FrameRecord], errors: &mut Vec<String>) {
    let mut check = |ok: bool, description: &str| {
        if !ok {
            errors.push(format!("fixture: {description}"));
        }
    };
    let role = |tick: usize, id: AgentId| {
        frames
            .get(tick)
            .and_then(|f| f.roles.iter().find(|r| r.id == id))
    };
    if let Some(owner) = role(8, 1) {
        check(
            (owner.holdings - 24.0).abs() < 1e-9,
            "owner preparation holdings must be 24 at tick 8",
        );
        check(
            (owner.caches.values().sum::<f64>() - 12.0).abs() < 1e-9,
            "original preparation stock must be 12 at tick 8",
        );
        let mixed = matches!(lab.fixture, Fixture::Mixed { .. });
        let sources = owner.protection.as_ref().map(|s| &s.sources);
        check(
            sources.is_some_and(|s| s.len() == if mixed { 2 } else { 1 }),
            "only preparation burials register sources",
        );
        if let Some(state) = &owner.protection {
            for (&site, source) in &state.sources {
                let pos = Pos::new(
                    if mixed && site == world_site(lab, Pos::new(5, 3)) {
                        5
                    } else {
                        3
                    },
                    3,
                );
                let exposed = if mixed {
                    pos.x == 3
                } else {
                    !matches!(
                        lab.fixture,
                        Fixture::Single {
                            initial_observed: false,
                            ..
                        } | Fixture::Stumble {
                            initial_observed: false
                        } | Fixture::CueUnseenWatcher
                    )
                };
                check(
                    state
                        .exposure
                        .entries
                        .get(&site)
                        .is_some_and(|e| e.exposed == exposed),
                    "prepared source cue disagrees with supplied owner sight",
                );
                check(
                    source.initial_amount == if mixed { 6.0 } else { 12.0 },
                    "prepared source amount mismatch",
                );
            }
        }
    } else {
        check(false, "owner did not survive preparation");
    }
    for f in frames {
        for a in &f.actions {
            if a.action == "prepare_deposit" {
                let seen = !matches!(lab.fixture, Fixture::CueVisibleNonwatcher)
                    && (a.source == Some(world_site(lab, Pos::new(3, 3)))
                        && !matches!(
                            lab.fixture,
                            Fixture::Single {
                                initial_observed: false,
                                ..
                            } | Fixture::Stumble {
                                initial_observed: false
                            }
                        ));
                check(
                    a.actual_watchers.contains(&2) == seen,
                    "initial deposit actual sightings mismatch",
                );
            }
            if a.phase == "relocation"
                && a.gross_buried > 0.0
                && matches!(
                    lab.fixture,
                    Fixture::Single { .. }
                        | Fixture::CueVisibleNonwatcher
                        | Fixture::CueUnseenWatcher
                        | Fixture::Stumble { .. }
                )
            {
                let expected = matches!(
                    lab.fixture,
                    Fixture::Single {
                        redeposit_observed: true,
                        ..
                    }
                );
                check(
                    a.actual_watchers.contains(&2) == expected,
                    "single-cache redeposit actual sighting mismatch",
                );
                check(
                    a.perceived_exposure == Some(false),
                    "completed redeposit must have no perceived visible witness",
                );
                let release = if matches!(lab.fixture, Fixture::Stumble { .. }) {
                    18
                } else {
                    12
                };
                if let (Some(owner), Some(observer)) = (role(release, 1), role(release, 2)) {
                    if let Some(site) = a.target {
                        check(
                            owner.pos != site && observer.pos != site,
                            "owner/observer blocks redeposit at opportunity release",
                        );
                    }
                }
            }
        }
        if let Some(owner) = f.roles.iter().find(|r| r.id == 1) {
            if f.tick >= 8 {
                check(
                    owner.protection.as_ref().is_some_and(|s| {
                        s.sources.len()
                            == if matches!(lab.fixture, Fixture::Mixed { .. }) {
                                2
                            } else {
                                1
                            }
                    }),
                    "reburial registered a new source",
                );
            }
        }
    }
    let release = match lab.fixture {
        Fixture::Mixed { .. } => 20,
        Fixture::Stumble { .. } => 18,
        _ => 12,
    };
    check(
        role(release, 2).is_some(),
        "observer did not survive to scheduled opportunity",
    );
}
fn world_site(lab: &LabConfig, pos: Pos) -> u32 {
    let pos = lab::transform(lab, pos);
    pos.y * 9 + pos.x
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_preparation_and_diagnostic_collection_preserve_trajectory() {
        for fixture in [
            Fixture::Single {
                initial_observed: true,
                redeposit_observed: false,
            },
            Fixture::Mixed {
                observed_first: true,
            },
            Fixture::Mixed {
                observed_first: false,
            },
            Fixture::CueUnseenWatcher,
            Fixture::CueVisibleNonwatcher,
        ] {
            let lab = LabConfig {
                fixture,
                policy: Policy::Selective,
                ..Default::default()
            };
            let on = run_episode(lab.clone(), 7, true).unwrap();
            let off = run_episode(lab, 7, false).unwrap();
            assert_eq!(on.frames, off.frames);
            assert_eq!(on.frames.len(), 65);
            assert!(on.ledger_errors.is_empty(), "{:?}", on.ledger_errors);
            let prepared = &on.frames[8].roles.iter().find(|r| r.id == 1).unwrap();
            assert_eq!(prepared.holdings, 24.0);
            assert_eq!(prepared.caches.values().sum::<f64>(), 12.0);
            assert!(off.cohorts.is_none());
        }
    }
    #[test]
    fn ordinary_arrivals_record_actual_targets_and_operations() {
        let record = run_episode(LabConfig::default(), 7, true).unwrap();
        let owner_ordinary: Vec<_> = record
            .frames
            .iter()
            .flat_map(|f| &f.actions)
            .filter(|a| a.id == 1 && a.phase == "ordinary")
            .collect();
        assert!(!owner_ordinary.is_empty());
        assert!(owner_ordinary.iter().all(|a| a.target.is_some()));
        assert_eq!(owner_ordinary.iter().map(|a| a.harvest).sum::<f64>(), 8.0);
    }
    #[test]
    fn diagnostic_failure_is_absorbing_and_preserves_death_and_trajectory() {
        let mut recorded = World::new(lab::rig_config(LabConfig::default()), 7).unwrap();
        let mut plain = recorded.clone();
        recorded.protection_ledger = Some(Ledger::new(1, 43.0));
        for _ in 0..64 {
            recorded.step();
            plain.step();
            assert_eq!(recorded.fingerprint(), plain.fingerprint());
            assert_eq!(recorded.protection_actions, plain.protection_actions);
        }
        assert!(recorded.protection_ledger.is_none());
        assert_eq!(recorded.protection_ledger_errors.len(), 1);
        assert!(recorded.protection_ledger_errors[0].contains("engine"));
    }
    #[test]
    fn actual_partial_metabolism_and_death_terminate_remaining_original_food() {
        let mut world = World::new(lab::rig_config(LabConfig::default()), 7).unwrap();
        world.protection_ledger = Some(Ledger::new(1, 44.0));
        world.step();
        let held = world.agent(1).unwrap().holdings[0];
        let demand = held as u32 + 5;
        world.agent_mut(1).unwrap().metabolism[0] = demand;
        world.step();
        let action = world.protection_actions.iter().find(|a| a.id == 1).unwrap();
        assert_eq!(action.metabolic_consumed, held);
        assert_eq!(action.metabolic_demand, f64::from(demand));
        assert_eq!(
            world.protection_deaths,
            vec![DeathRecord {
                id: 1,
                pos: Pos::new(3, 3),
                cause: "starvation".into()
            }]
        );
        let cohort = &world.protection_ledger.as_ref().unwrap().cohorts[&30];
        assert_eq!(cohort.lost_cached[&30], 12.0);
        assert!(world.protection_ledger_errors.is_empty());
    }

    #[test]
    fn base_construction_covers_order_cues_redeposit_and_stumble_opportunities() {
        let fixtures = [
            Fixture::Single {
                initial_observed: true,
                redeposit_observed: true,
            },
            Fixture::Single {
                initial_observed: true,
                redeposit_observed: false,
            },
            Fixture::Single {
                initial_observed: false,
                redeposit_observed: true,
            },
            Fixture::Single {
                initial_observed: false,
                redeposit_observed: false,
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
                initial_observed: true,
            },
            Fixture::Stumble {
                initial_observed: false,
            },
        ];
        for fixture in fixtures {
            for policy in [
                Policy::Off,
                Policy::Selective,
                Policy::Indiscriminate,
                Policy::Erased,
            ] {
                for cost in [0.0, 0.25] {
                    for seed in [7, 8] {
                        let discoveries: &[f64] = if matches!(fixture, Fixture::Stumble { .. }) {
                            &[0.0, 0.25]
                        } else {
                            &[0.0]
                        };
                        for &discovery in discoveries {
                            let lab = LabConfig {
                                fixture: fixture.clone(),
                                policy: policy.clone(),
                                reburial_cost: cost,
                                discovery,
                                ..Default::default()
                            };
                            let r = run_episode(lab.clone(), seed, true).unwrap();
                            assert!(
                                r.fixture_errors.is_empty(),
                                "{lab:?} seed {seed}: {:?}",
                                r.fixture_errors
                            );
                            assert!(
                                r.ledger_errors.is_empty(),
                                "{lab:?} seed {seed}: {:?}",
                                r.ledger_errors
                            );
                            assert_eq!(r.frames.len(), 65);
                            assert!(
                                r.frames[0].actions.is_empty() && r.frames[0].deaths.is_empty()
                            );
                            assert_eq!(r.frames[0].relocation, RelocationEvents::default());
                            assert_eq!(r.frames[8].restriction_ticks[&1], 8);
                            for f in &r.frames[1..=8] {
                                assert_eq!(
                                    f.actions.iter().find(|a| a.id == 1).unwrap().phase,
                                    "preparation"
                                );
                            }
                            let mixed = matches!(fixture, Fixture::Mixed { .. });
                            let route: Vec<Pos> = match fixture {
                                Fixture::Mixed { .. } => vec![
                                    Pos::new(4, 5),
                                    Pos::new(5, 5),
                                    Pos::new(6, 5),
                                    Pos::new(7, 5),
                                    Pos::new(7, 6),
                                ],
                                Fixture::CueUnseenWatcher => vec![
                                    Pos::new(4, 6),
                                    Pos::new(5, 6),
                                    Pos::new(6, 6),
                                    Pos::new(7, 6),
                                ],
                                Fixture::Single {
                                    initial_observed: true,
                                    redeposit_observed: true,
                                } => vec![Pos::new(3, 6)],
                                Fixture::Single {
                                    initial_observed: false,
                                    redeposit_observed: true,
                                } => vec![
                                    Pos::new(6, 5),
                                    Pos::new(5, 5),
                                    Pos::new(4, 5),
                                    Pos::new(3, 5),
                                    Pos::new(3, 6),
                                ],
                                Fixture::Single {
                                    initial_observed: false,
                                    ..
                                }
                                | Fixture::Stumble {
                                    initial_observed: false,
                                } => vec![Pos::new(7, 6)],
                                _ => vec![
                                    Pos::new(4, 5),
                                    Pos::new(5, 5),
                                    Pos::new(6, 5),
                                    Pos::new(7, 5),
                                    Pos::new(7, 6),
                                ],
                            };
                            for (i, pos) in route.iter().enumerate() {
                                let f = &r.frames[if mixed { 4 } else { 2 } + i];
                                assert_eq!(f.roles.iter().find(|a| a.id == 2).unwrap().pos, *pos);
                                let a = f.actions.iter().find(|a| a.id == 2).unwrap();
                                assert_eq!(a.action, "walk");
                                assert_eq!(a.target, Some(*pos));
                                assert_eq!(a.harvest, 0.0);
                            }
                            if let Fixture::Mixed { observed_first } = fixture {
                                for (i, pos) in [
                                    Pos::new(if observed_first { 3 } else { 5 }, 3),
                                    Pos::new(4, 3),
                                    Pos::new(if observed_first { 5 } else { 3 }, 3),
                                    Pos::new(4, 3),
                                ]
                                .iter()
                                .enumerate()
                                {
                                    assert_eq!(
                                        r.frames[i + 1]
                                            .roles
                                            .iter()
                                            .find(|a| a.id == 1)
                                            .unwrap()
                                            .pos,
                                        *pos
                                    );
                                }
                                assert_eq!(
                                    r.frames[3]
                                        .actions
                                        .iter()
                                        .find(|a| a.id == 1)
                                        .unwrap()
                                        .gross_buried,
                                    6.0
                                );
                            }
                            let release = if mixed {
                                20
                            } else if matches!(fixture, Fixture::Stumble { .. }) {
                                26
                            } else {
                                12
                            };
                            assert_eq!(
                                r.frames[release].restriction_ticks[&2],
                                if matches!(fixture, Fixture::Stumble { .. }) {
                                    18
                                } else {
                                    release as u64
                                }
                            );
                            assert_eq!(
                                r.frames[release + 1]
                                    .actions
                                    .iter()
                                    .find(|a| a.id == 2)
                                    .unwrap()
                                    .phase,
                                "ordinary"
                            );
                            for a in r.frames.iter().flat_map(|f| &f.actions).filter(|a| {
                                a.phase == "relocation"
                                    && a.gross_buried > 0.0
                                    && a.actual_watchers.contains(&2)
                            }) {
                                let target = a.target.unwrap();
                                assert!(
                                    r.frames.iter().skip(release + 1).any(|f| f
                                        .roles
                                        .iter()
                                        .find(|r| r.id == 2)
                                        .is_some_and(|r| r.pos == target)),
                                    "observer must realize supplied access"
                                );
                            }
                            assert_eq!(r.frames[9].current_bury_cost, cost);
                            assert_eq!(
                                serde_json::from_str::<EpisodeRecord>(
                                    &serde_json::to_string(&r).unwrap()
                                )
                                .unwrap(),
                                r
                            );
                            if matches!(fixture, Fixture::Stumble { .. }) {
                                for (i, pos) in [
                                    Pos::new(6, 6),
                                    Pos::new(5, 6),
                                    Pos::new(4, 6),
                                    Pos::new(3, 6),
                                    Pos::new(3, 5),
                                    Pos::new(3, 4),
                                    Pos::new(3, 3),
                                    Pos::new(3, 2),
                                ]
                                .iter()
                                .enumerate()
                                {
                                    let frame = &r.frames[19 + i];
                                    assert_eq!(
                                        frame.roles.iter().find(|a| a.id == 2).unwrap().pos,
                                        *pos
                                    );
                                    let action = frame.actions.iter().find(|a| a.id == 2).unwrap();
                                    assert_eq!(action.phase, "encounter");
                                    assert_eq!(action.target, Some(*pos));
                                }
                                assert_eq!(r.frames[18].restriction_ticks[&2], 18);
                            }
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn legacy_fate_logging_changes_no_episode_frame() {
        let mut off = World::new(
            lab::rig_config(LabConfig {
                policy: Policy::Selective,
                ..Default::default()
            }),
            8,
        )
        .unwrap();
        let mut on = off.clone();
        on.record_fates = true;
        on.protection_ledger = Some(Ledger::new(1, 44.0));
        for _ in 0..64 {
            off.step();
            on.step();
            assert_eq!(frame(&off), frame(&on));
        }
        assert!(!on.cache_log.is_empty());
        assert!(off.cache_log.is_empty());
        assert!(on.protection_ledger_errors.is_empty());
    }
}
