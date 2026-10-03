//! Checked construction and prepared-source registration; no phase scheduling.
use super::{controller::perceived_exposure, state::*};
use crate::{
    agent::{in_slot_0, Agent, AgentId},
    config::*,
    geometry::Pos,
    world::World,
};

fn walls() -> Vec<Wall> {
    [(0, 0, 9, 1), (0, 8, 9, 1), (0, 1, 1, 7), (8, 1, 1, 7)]
        .into_iter()
        .map(|(x, y, width, height)| Wall {
            x,
            y,
            width,
            height,
            opaque: true,
        })
        .collect()
}
pub fn rig_config(lab: LabConfig) -> Config {
    let mut c = Config {
        width: 9,
        height: 9,
        population: 2,
        vision: URange::new(2, 2),
        walls: walls(),
        goods: vec![Good {
            map: Map::Flat { capacity: 0.0 },
            ..Good::sugar()
        }],
        movement: Movement {
            mode: MoveMode::Walk,
            speed: 1,
        },
        ..Config::default()
    };
    c.growback.rate = 0.0;
    c.goap.horizon = 4;
    c.caching.capacity = 128;
    c.caching.dig_below = DigBelow::Reserve;
    // Fixed role traits are supplied before insertion, rather than dealt by id.
    c.watching.watchers = 0.0;
    c.watching.on = true;
    c.watching.span = lab.observer_span;
    c.watching.raid_if = RaidIf::Always;
    c.watching.value = SeenValue::Room;
    c.theft.find = lab.discovery;
    c.theft.loot = Loot::Keep;
    c.protection_lab = Some(lab);
    c
}

pub(crate) fn validation_errors(c: &Config) -> Vec<FieldError> {
    let Some(lab) = &c.protection_lab else {
        return vec![];
    };
    let mut errors = Vec::new();
    let mut check = |ok: bool, field: &str| {
        if !ok {
            errors.push(FieldError::new(
                field,
                "unsupported protection lab configuration",
            ));
        }
    };
    for (ok, field) in [
        (c.width == 9, "width"),
        (c.height == 9, "height"),
        (c.population == 2, "population"),
        (c.goods.len() == 1, "goods"),
        (c.walls == walls(), "walls"),
        (
            c.goods
                .first()
                .is_some_and(|g| matches!(g.map,Map::Flat{capacity} if capacity==0.0)),
            "goods.0.map",
        ),
        (c.decision.rule == DecisionRule::Book, "decision.rule"),
        (c.movement.mode == MoveMode::Walk, "movement.mode"),
        (c.movement.speed == 1, "movement.speed"),
        (!c.spatial_hoarding.enabled, "spatial_hoarding.enabled"),
        (c.lab.is_none(), "lab"),
        (!c.central.enabled, "central.enabled"),
        (c.schedule.is_empty(), "schedule"),
        (c.memory.span == 0, "memory.span"),
        (c.truffles.share == 0.0, "truffles.share"),
        (!c.sex.enabled, "sex.enabled"),
        (!c.replacement.enabled, "replacement.enabled"),
        (!c.lifespan.enabled, "lifespan.enabled"),
        (!c.seasons.enabled, "seasons.enabled"),
        (!c.pollution.enabled, "pollution.enabled"),
        (!c.diffusion.enabled, "diffusion.enabled"),
        (!c.combat.enabled, "combat.enabled"),
        (!c.trade.enabled, "trade.enabled"),
        (!c.credit.enabled, "credit.enabled"),
        (!c.disease.enabled, "disease.enabled"),
        (c.disease.outbreaks.is_empty(), "disease.outbreaks"),
        (!c.inheritance.enabled, "inheritance.enabled"),
        (!c.culture.enabled, "culture.enabled"),
        (!c.foresight.enabled, "foresight.enabled"),
        (c.growback.rate == 0.0 && !c.growback.instant, "growback"),
        (c.goap.horizon == 4, "goap.horizon"),
        (c.caching.capacity == 128, "caching.capacity"),
        (
            c.caching.dig_below == DigBelow::Reserve,
            "caching.dig_below",
        ),
        (!c.caching.mixed, "caching.mixed"),
        (c.caching.rule == CachingRule::None, "caching.rule"),
        (c.caching.bury_cost == 0.0, "caching.bury_cost"),
        (c.watching.on, "watching.on"),
        (c.watching.watchers == 0.0, "watching.watchers"),
        (c.watching.who == Who::Share, "watching.who"),
        (c.theft.cheaters == 0.0, "theft.cheaters"),
        (c.watching.span == lab.observer_span, "watching.span"),
        (
            c.watching.raid_when == RaidWhen::Always,
            "watching.raid_when",
        ),
        (c.watching.raid_if == RaidIf::Always, "watching.raid_if"),
        (c.watching.value == SeenValue::Room, "watching.value"),
        (
            c.watching.scrounge == Scrounge::Harvest,
            "watching.scrounge",
        ),
        (c.theft.owner_memory, "theft.owner_memory"),
        (c.theft.loot == Loot::Keep, "theft.loot"),
        (c.theft.find == lab.discovery, "theft.find"),
        (
            lab.reburial_cost.is_finite() && lab.reburial_cost >= 0.0,
            "protection_lab.reburial_cost",
        ),
        (
            (0.0..=1.0).contains(&lab.discovery),
            "protection_lab.discovery",
        ),
        (lab.exposure_span > 0, "protection_lab.exposure_span"),
        (lab.observer_span > 0, "protection_lab.observer_span"),
    ] {
        check(ok, field);
    }
    errors
}

pub(crate) fn transform(lab: &LabConfig, pos: Pos) -> Pos {
    if lab.mirrored {
        Pos::new(8 - pos.x, pos.y)
    } else {
        pos
    }
}
fn starts(lab: &LabConfig) -> (Pos, Pos) {
    let owner = match lab.fixture {
        Fixture::Mixed {
            observed_first: false,
        } => Pos::new(5, 3),
        _ => Pos::new(3, 3),
    };
    let observer = match lab.fixture {
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
    (transform(lab, owner), transform(lab, observer))
}
pub(crate) fn initialize(world: &mut World) {
    let lab = world.config.protection_lab.clone().expect("enabled lab");
    if !matches!(lab.fixture, Fixture::Mixed { .. }) {
        for pos in [Pos::new(2, 2), Pos::new(2, 3)] {
            let i = world.torus.index(transform(&lab, pos));
            world.sites[i].capacity[0] = 4.0;
            world.sites[i].resource[0] = 4.0;
        }
    }
    let (owner, observer) = starts(&lab);
    for (role, pos, held, vision) in [(1, owner, 44.0, 2), (2, observer, 96.0, 6)] {
        let mut a = Agent::random(&world.config, pos, 0, &mut world.rng);
        a.holdings = in_slot_0(held);
        a.initial = in_slot_0(held);
        a.metabolism = in_slot_0(1);
        a.vision = vision;
        a.rate = 1.0;
        a.delivery_rate = 1.0;
        a.cheater = role == 2;
        a.watches = role == 2 && !matches!(lab.fixture, Fixture::CueVisibleNonwatcher);
        a.protection = (role == 1).then(ProtectionState::default);
        world.insert_agent(a).expect("fixed valid role placement");
    }
}
pub(crate) fn note_prepared_deposit(world: &mut World, owner: AgentId, site: u32, amount: f64) {
    let Some(lab) = &world.config.protection_lab else {
        return;
    };
    if owner != 1 || amount <= 0.0 {
        return;
    }
    let pos = match lab.fixture {
        Fixture::Mixed { observed_first } => match world.tick {
            0 => Pos::new(if observed_first { 3 } else { 5 }, 3),
            2 => Pos::new(if observed_first { 5 } else { 3 }, 3),
            _ => return,
        },
        _ if world.tick == 0 => Pos::new(3, 3),
        _ => return,
    };
    if world.torus.index(transform(lab, pos)) as u32 != site {
        return;
    }
    let exposed = perceived_exposure(world, owner);
    let tick = world.tick;
    let state = world
        .agent_mut(owner)
        .expect("owner")
        .protection
        .as_mut()
        .expect("lab owner state");
    if state.sources.contains_key(&site) {
        return;
    }
    state.sources.insert(
        site,
        Source {
            tick,
            initial_amount: amount,
            attempted: false,
        },
    );
    state.exposure.remember(site, tick, exposed);
}

pub(crate) fn begin_tick(world: &mut World) {
    let Some(lab) = world.config.protection_lab.clone() else {
        return;
    };
    if world.tick == 8 {
        world.config.caching.bury_cost = lab.reburial_cost;
        if lab.policy == Policy::Erased {
            if let Some(state) = world.agent_mut(1).and_then(|a| a.protection.as_mut()) {
                state.exposure.entries.clear();
            }
        }
    }
}
fn observer_route(lab: &LabConfig) -> Vec<Pos> {
    let initial_observed = match lab.fixture {
        Fixture::Single {
            initial_observed, ..
        }
        | Fixture::Stumble { initial_observed } => initial_observed,
        _ => true,
    };
    let later_observed = matches!(
        lab.fixture,
        Fixture::Single {
            redeposit_observed: true,
            ..
        }
    );
    let xy: Vec<_> = match lab.fixture {
        Fixture::CueUnseenWatcher => vec![(4, 6), (5, 6), (6, 6), (7, 6)],
        _ => match (initial_observed, later_observed) {
            (true, true) => vec![(3, 6)],
            (true, false) => vec![(4, 5), (5, 5), (6, 5), (7, 5), (7, 6)],
            (false, true) => vec![(6, 5), (5, 5), (4, 5), (3, 5), (3, 6)],
            (false, false) => vec![(7, 6)],
        },
    };
    xy.into_iter()
        .map(|(x, y)| transform(lab, Pos::new(x, y)))
        .collect()
}
pub(crate) fn scripted_action(world: &mut World, id: AgentId) -> Option<crate::rules::Harvest> {
    use crate::rules::{movement, Harvest};
    let lab = world.config.protection_lab.clone()?;
    let tick = world.tick;
    let mut target = None;
    let mut deposit = 0.0;
    let mut encounter = false;
    if id == 1 {
        if tick >= 8 {
            return None;
        }
        match lab.fixture {
            Fixture::Mixed { observed_first } => match tick {
                0 => deposit = 6.0,
                1 | 3 => target = Some(transform(&lab, Pos::new(4, 3))),
                2 => {
                    target = Some(transform(
                        &lab,
                        Pos::new(if observed_first { 5 } else { 3 }, 3),
                    ));
                    deposit = 6.0;
                }
                _ => {}
            },
            _ if tick == 0 => deposit = 12.0,
            _ => {}
        }
    } else if id == 2 {
        match lab.fixture {
            Fixture::Mixed { .. } => {
                if tick >= 20 {
                    return None;
                }
                if (3..=7).contains(&tick) {
                    target = Some(transform(
                        &lab,
                        [
                            Pos::new(4, 5),
                            Pos::new(5, 5),
                            Pos::new(6, 5),
                            Pos::new(7, 5),
                            Pos::new(7, 6),
                        ][(tick - 3) as usize],
                    ));
                }
            }
            Fixture::Stumble { .. } => {
                if tick >= 26 {
                    return None;
                }
                if (18..=25).contains(&tick) {
                    encounter = true;
                    target = Some(transform(
                        &lab,
                        [
                            Pos::new(6, 6),
                            Pos::new(5, 6),
                            Pos::new(4, 6),
                            Pos::new(3, 6),
                            Pos::new(3, 5),
                            Pos::new(3, 4),
                            Pos::new(3, 3),
                            Pos::new(3, 2),
                        ][(tick - 18) as usize],
                    ));
                } else if tick > 0 {
                    target = observer_route(&lab).get((tick - 1) as usize).copied();
                }
            }
            _ => {
                if tick >= 12 {
                    return None;
                }
                if tick > 0 {
                    target = observer_route(&lab).get((tick - 1) as usize).copied();
                }
            }
        }
    } else {
        return None;
    }
    let phase = if encounter {
        "encounter"
    } else if tick < 8 {
        "preparation"
    } else {
        "observer_hold"
    };
    if let Some(a) = world.protection_actions.last_mut() {
        a.phase = phase.into();
        a.action = if deposit > 0.0 {
            "prepare_deposit"
        } else if target.is_some() {
            "walk"
        } else {
            "hold"
        }
        .into();
        a.target = target;
    }
    if !encounter {
        *world.protection_restrictions.entry(id).or_default() += 1;
    }
    let mut harvest = Harvest::default();
    if let Some(target) = target {
        if encounter {
            harvest = movement::arrive(world, id, target);
        } else {
            movement::walk_without_gather(world, id, target);
        }
        if world.agent(id).unwrap().pos != target {
            world.protection_fixture_errors.push(format!(
                "tick {tick} agent {id}: scripted target ({},{}) not reached",
                target.x, target.y
            ));
            deposit = 0.0;
        }
    }
    if deposit > 0.0 {
        crate::minds::caching::bury(world, id, deposit);
    }
    Some(harvest)
}

#[cfg(test)]
mod schedule_tests {
    use super::*;
    #[test]
    fn reflection_transforms_supplied_geometry_without_tuning_destinations() {
        for fixture in [
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
        ] {
            let base = LabConfig {
                fixture,
                ..Default::default()
            };
            let mirror = LabConfig {
                mirrored: true,
                ..base.clone()
            };
            let (a, b) = starts(&base);
            let (ma, mb) = starts(&mirror);
            assert_eq!((ma, mb), (Pos::new(8 - a.x, a.y), Pos::new(8 - b.x, b.y)));
            assert_eq!(
                observer_route(&mirror),
                observer_route(&base)
                    .iter()
                    .map(|p| Pos::new(8 - p.x, p.y))
                    .collect::<Vec<_>>()
            );
            let w = World::new(rig_config(mirror.clone()), 7).unwrap();
            for p in [Pos::new(2, 2), Pos::new(2, 3)] {
                assert_eq!(
                    w.site(transform(&mirror, p)).resource[0],
                    if matches!(base.fixture, Fixture::Mixed { .. }) {
                        0.0
                    } else {
                        4.0
                    }
                );
            }
            for p in [
                Pos::new(3, 3),
                Pos::new(5, 3),
                Pos::new(4, 3),
                Pos::new(3, 2),
                Pos::new(3, 4),
            ] {
                assert_eq!(transform(&mirror, p), Pos::new(8 - p.x, p.y));
            }
            for p in [
                Pos::new(6, 6),
                Pos::new(5, 6),
                Pos::new(4, 6),
                Pos::new(3, 6),
                Pos::new(3, 5),
                Pos::new(3, 4),
                Pos::new(3, 3),
                Pos::new(3, 2),
            ] {
                assert_eq!(transform(&mirror, p), Pos::new(8 - p.x, p.y));
            }
        }
    }
    #[test]
    fn blocked_script_is_preserved_as_fixture_failure_without_teleport_or_deposit() {
        let mut w = World::new(
            rig_config(LabConfig {
                fixture: Fixture::Mixed {
                    observed_first: true,
                },
                ..Default::default()
            }),
            7,
        )
        .unwrap();
        w.step();
        w.move_agent(2, Pos::new(4, 3));
        let before = w.agent(1).unwrap().pos;
        w.step();
        assert_eq!(w.agent(1).unwrap().pos, before);
        assert_eq!(w.protection_fixture_errors.len(), 1);
        assert!(w.protection_fixture_errors[0].contains("scripted target"));
    }
}
