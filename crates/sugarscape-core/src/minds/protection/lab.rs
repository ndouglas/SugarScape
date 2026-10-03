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
