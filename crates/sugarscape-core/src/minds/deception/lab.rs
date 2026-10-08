//! Checked finite rig construction; physical actions and schedules are separate.
use super::state::*;
use crate::{
    agent::{in_slot_0, Agent},
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
    c.memory.span = OBSERVER_SPAN;
    c.watching.on = true;
    c.watching.watchers = 0.0;
    c.watching.span = OBSERVER_SPAN;
    c.watching.raid_if = RaidIf::Always;
    c.watching.value = SeenValue::Room;
    c.deception_lab = Some(lab);
    c
}

pub(crate) fn validation_errors(c: &Config) -> Vec<FieldError> {
    let Some(lab) = &c.deception_lab else {
        return vec![];
    };
    let mut errors = Vec::new();
    if !lab.effort_cost.is_finite() || ![0.0, 3.0].contains(&lab.effort_cost) {
        errors.push(FieldError::new(
            "deception_lab.effort_cost",
            "supported effort costs are 0 and 3",
        ));
    }
    // Every configuration field is fixed except the explicitly typed treatment.
    // Compare the full rig rather than accepting an unlisted feature or knob.
    let expected = serde_json::to_value(rig_config(lab.clone())).expect("serializable rig");
    let actual = serde_json::to_value(c).expect("serializable configuration");
    for (field, value) in actual.as_object().expect("configuration object") {
        if field != "deception_lab" && expected.get(field) != Some(value) {
            errors.push(FieldError::new(
                field,
                "unsupported deception lab configuration",
            ));
        }
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

pub(crate) fn initialize(w: &mut World) {
    let lab = w.config.deception_lab.clone().expect("enabled lab");
    for pos in [Pos::new(2, 2), Pos::new(2, 3)] {
        let index = w.torus.index(transform(&lab, pos));
        w.sites[index].resource[0] = 4.0;
        w.sites[index].capacity[0] = 4.0;
    }
    for (role, pos, held, vision) in [(1, Pos::new(3, 3), 44.0, 2), (2, Pos::new(3, 6), 96.0, 6)] {
        let mut a = Agent::random(&w.config, transform(&lab, pos), 0, &mut w.rng);
        a.holdings = in_slot_0(held);
        a.initial = in_slot_0(held);
        a.metabolism = in_slot_0(1);
        a.vision = vision;
        a.rate = 1.0;
        a.delivery_rate = 1.0;
        a.cheater = role == 2;
        a.watches = role == 2;
        // P4 receivers use only bounded seen-cache evidence, not map beliefs.
        a.remembers = false;
        a.deception = (role == 1).then(SenderState::default);
        w.insert_agent(a).expect("fixed valid role placement");
    }
    let display = match lab.layout {
        Layout::OnRoute => Pos::new(3, 5),
        Layout::OffRoute => Pos::new(5, 6),
    };
    w.deception = Some(Runtime {
        source: w.torus.index(transform(&lab, Pos::new(3, 3))) as u32,
        display: w.torus.index(transform(&lab, display)) as u32,
        prepared: false,
        diagnostics: true,
        ledger: Some(crate::minds::protection::ledger::Ledger::new(
            1,
            w.agent(1).expect("constructed owner").holdings[0],
        )),
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
}

/// Canonical axis order is fixed before constructing any worlds.
pub fn conditions() -> Vec<LabConfig> {
    let mut out = Vec::new();
    for sender in [
        SenderPolicy::Ordinary,
        SenderPolicy::MatchedNeutral,
        SenderPolicy::Sham,
    ] {
        for view in [View::Ambiguous, View::Clear] {
            for display_seen in [false, true] {
                for layout in [Layout::OnRoute, Layout::OffRoute] {
                    for effort_cost in [0.0, 3.0] {
                        for mirrored in [false, true] {
                            out.push(LabConfig {
                                sender,
                                view,
                                display_seen,
                                layout,
                                effort_cost,
                                mirrored,
                            });
                        }
                    }
                }
            }
        }
    }
    out.sort_by_key(condition_id);
    out
}
pub fn condition_id(c: &LabConfig) -> String {
    format!(
        "{}-{}-{}-{}-cost{}-{}",
        match c.sender {
            SenderPolicy::Ordinary => "ordinary",
            SenderPolicy::MatchedNeutral => "matched_neutral",
            SenderPolicy::Sham => "sham",
        },
        match c.view {
            View::Ambiguous => "ambiguous",
            View::Clear => "clear",
        },
        if c.display_seen { "seen" } else { "unseen" },
        match c.layout {
            Layout::OnRoute => "on_route",
            Layout::OffRoute => "off_route",
        },
        c.effort_cost,
        if c.mirrored { "reflected" } else { "base" }
    )
}
/// Declared owner waypoints; no RNG or world construction occurs here.
pub(crate) fn owner_route(layout: Layout, outward: bool) -> Vec<Pos> {
    let points: &[(u32, u32)] = match (layout, outward) {
        (Layout::OnRoute, true) => &[(3, 4), (3, 5)],
        (Layout::OffRoute, true) => &[(4, 3), (5, 3), (5, 4), (5, 5), (5, 6)],
        (Layout::OnRoute, false) => &[(2, 5), (2, 4), (2, 3)],
        (Layout::OffRoute, false) => &[(5, 5), (4, 5), (3, 5), (2, 5), (2, 4), (2, 3)],
    };
    points.iter().map(|&(x, y)| Pos::new(x, y)).collect()
}
pub(crate) fn observer_route(outward: bool) -> Vec<Pos> {
    let points: &[(u32, u32)] = if outward {
        &[(4, 6), (5, 6), (6, 6), (7, 6), (7, 7)]
    } else {
        &[(7, 6), (6, 6), (5, 6), (4, 6), (3, 6)]
    };
    points.iter().map(|&(x, y)| Pos::new(x, y)).collect()
}
