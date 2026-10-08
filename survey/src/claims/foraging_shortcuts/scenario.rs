//! Literal geometry and independent multi-source cardinal reachability.
use super::{Geometry, Panel, Regime};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use sugarscape_core::foraging::construction as core;

fn pos(x: u32, y: u32) -> core::Pos {
    core::Pos { x, y }
}

fn vertices(geometry: Geometry) -> Vec<core::Pos> {
    match geometry {
        Geometry::Straight => vec![pos(3, 20), pos(3, 5)],
        Geometry::Detour => vec![pos(4, 20), pos(21, 20), pos(21, 5), pos(5, 5)],
        Geometry::Twisting => vec![
            pos(4, 20),
            pos(21, 20),
            pos(21, 17),
            pos(18, 17),
            pos(18, 14),
            pos(21, 14),
            pos(21, 11),
            pos(18, 11),
            pos(18, 8),
            pos(21, 8),
            pos(21, 5),
            pos(5, 5),
        ],
    }
}

pub(super) fn segment(
    a: core::Pos,
    b: core::Pos,
    out: &mut BTreeSet<core::Pos>,
) -> Result<(), String> {
    if a.x != b.x && a.y != b.y {
        return Err("route segment must be cardinal".into());
    }
    for x in a.x.min(b.x)..=a.x.max(b.x) {
        for y in a.y.min(b.y)..=a.y.max(b.y) {
            out.insert(pos(x, y));
        }
    }
    Ok(())
}

fn route(geometry: Geometry, open: &mut BTreeSet<core::Pos>) -> Result<(), String> {
    for pair in vertices(geometry).windows(2) {
        segment(pair[0], pair[1], open)?;
    }
    Ok(())
}

#[allow(
    dead_code,
    reason = "build is consumed by the Task 2 wire validator and Task 4 collector"
)]
pub(super) fn build(
    panel: Panel,
    geometry: Geometry,
    regime: Regime,
) -> Result<core::Setup, String> {
    if panel == Panel::Access && regime == Regime::AlreadyOpen {
        return Err("access panel does not support already_open".into());
    }
    let nest: Vec<_> = (20..=22)
        .flat_map(|y| (2..=4).map(move |x| pos(x, y)))
        .collect();
    let waste = pos(1, 21);
    let food: Vec<_> = (2..=5)
        .flat_map(|y| (2..=5).map(move |x| pos(x, y)))
        .enumerate()
        .map(|(id, pos)| core::Resource { id: id as u64, pos })
        .collect();
    let mut open: BTreeSet<_> = nest
        .iter()
        .copied()
        .chain([waste])
        .chain(food.iter().map(|r| r.pos))
        .collect();
    route(geometry, &mut open)?;
    if panel == Panel::Access {
        let gate = match geometry {
            Geometry::Straight => pos(3, 6),
            Geometry::Detour | Geometry::Twisting => pos(6, 5),
        };
        if !open.remove(&gate) {
            return Err("declared access gate is absent".into());
        }
    }
    if regime == Regime::AlreadyOpen {
        route(Geometry::Straight, &mut open)?;
    }
    let diggable = if regime == Regime::Paid {
        (1..=23)
            .flat_map(|y| (1..=23).map(move |x| pos(x, y)))
            .collect()
    } else {
        vec![]
    };
    core::Setup {
        width: 25,
        height: 25,
        open: open.into_iter().collect(),
        diggable,
        waste,
        nest,
        workers: vec![
            pos(2, 21),
            pos(2, 21),
            pos(3, 21),
            pos(3, 21),
            pos(4, 21),
            pos(4, 21),
            pos(3, 20),
            pos(3, 20),
        ],
        food,
        parameters: core::Parameters {
            p_search: 0.05,
            p_return: 0.01,
            lambda_fidelity: 1.0,
            lambda_publish: 1.0,
            lambda_waypoint: 0.01,
        },
    }
    .normalized()
    .map_err(|errors| format!("shortcut scenario: {errors:?}"))
}

fn neighbors(p: core::Pos, width: u32, height: u32) -> impl Iterator<Item = core::Pos> {
    [
        p.x.checked_sub(1).map(|x| pos(x, p.y)),
        p.x.checked_add(1)
            .filter(|x| *x < width)
            .map(|x| pos(x, p.y)),
        p.y.checked_sub(1).map(|y| pos(p.x, y)),
        p.y.checked_add(1)
            .filter(|y| *y < height)
            .map(|y| pos(p.x, y)),
    ]
    .into_iter()
    .flatten()
}

#[allow(
    dead_code,
    reason = "patch_distances is consumed by the Task 2 validator and Task 5 analysis"
)]
pub(super) fn patch_distances(setup: &core::Setup) -> Result<Vec<(u64, Option<u32>)>, String> {
    setup
        .validate()
        .map_err(|errors| format!("patch distance setup: {errors:?}"))?;
    let open: BTreeSet<_> = setup.open.iter().copied().collect();
    let mut distances = BTreeMap::new();
    let mut queue = VecDeque::new();
    for &p in &setup.nest {
        distances.insert(p, 0);
        queue.push_back(p);
    }
    while let Some(p) = queue.pop_front() {
        let distance = distances[&p] + 1;
        for next in neighbors(p, setup.width, setup.height) {
            if open.contains(&next) && !distances.contains_key(&next) {
                distances.insert(next, distance);
                queue.push_back(next);
            }
        }
    }
    Ok(setup
        .food
        .iter()
        .map(|r| (r.id, distances.get(&r.pos).copied()))
        .collect())
}
