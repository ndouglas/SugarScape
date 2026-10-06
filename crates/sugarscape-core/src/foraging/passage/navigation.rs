//! Geometry uses private classifications; only the first step uses fresh occupancy.
use super::{
    draws::{choose, DrawSource},
    knowledge::{CellKnowledge, Knowledge},
    metrics::ComputeCounts,
    observation::Observation,
    Checked, Pos,
};
use crate::config::FieldError;
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Frontier {
    pub(super) pos: Pos,
    pub(super) unknown_neighbors: Vec<Pos>,
    pub(super) distance: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Navigation {
    AtGoal,
    Move(Pos),
    Blocked,
    Unreachable,
}
fn error(field: &str, message: &str) -> Vec<FieldError> {
    vec![FieldError::new(field, message)]
}
fn check_origin(knowledge: &Knowledge, origin: Pos) -> Checked<()> {
    if knowledge.kind(origin)? != CellKnowledge::KnownOpen {
        return Err(error("navigation.origin", "must be privately known open"));
    }
    Ok(())
}
fn index(pos: Pos, width: u32) -> usize {
    (pos.y * width + pos.x) as usize
}

/// Each distance array and queue contains at most width * height entries.
/// Visits count dequeued cells; the peak includes the initial unique seed queue.
fn distances(knowledge: &Knowledge, seeds: &[Pos]) -> Checked<(Vec<Option<u32>>, ComputeCounts)> {
    let (width, height) = knowledge.dimensions();
    let mut distance = vec![None; (width * height) as usize];
    let mut queue = VecDeque::new();
    for &seed in seeds {
        if knowledge.kind(seed)? == CellKnowledge::KnownOpen
            && distance[index(seed, width)].is_none()
        {
            distance[index(seed, width)] = Some(0);
            queue.push_back(seed);
        }
    }
    let mut counts = ComputeCounts {
        route_calls: 1,
        peak_queue: queue.len() as u64,
        ..ComputeCounts::default()
    };
    while let Some(pos) = queue.pop_front() {
        counts.route_visits += 1;
        let next_distance = distance[index(pos, width)].expect("queued cell has a distance") + 1;
        for next in pos.neighbors(width, height) {
            if knowledge.kind(next)? == CellKnowledge::KnownOpen
                && distance[index(next, width)].is_none()
            {
                distance[index(next, width)] = Some(next_distance);
                queue.push_back(next);
            }
        }
        counts.peak_queue = counts.peak_queue.max(queue.len() as u64);
    }
    Ok((distance, counts))
}

/// Frontier scans count reachable KnownOpen cells whose neighbors are examined.
/// Neither array enumeration nor solid/unknown classifications count as a scan.
pub(super) fn frontiers(
    knowledge: &Knowledge,
    origin: Pos,
) -> Checked<(Vec<Frontier>, ComputeCounts)> {
    check_origin(knowledge, origin)?;
    let (width, height) = knowledge.dimensions();
    let (distances, mut counts) = distances(knowledge, &[origin])?;
    let mut found = Vec::new();
    for (i, distance) in distances.into_iter().enumerate() {
        let Some(distance) = distance else { continue };
        let pos = Pos {
            x: i as u32 % width,
            y: i as u32 / width,
        };
        counts.frontier_scans += 1;
        let mut unknown_neighbors = Vec::new();
        for next in pos.neighbors(width, height) {
            if knowledge.kind(next)? == CellKnowledge::Unknown {
                unknown_neighbors.push(next);
            }
        }
        if !unknown_neighbors.is_empty() {
            unknown_neighbors.sort_unstable();
            found.push(Frontier {
                pos,
                unknown_neighbors,
                distance,
            });
        }
    }
    found.sort_by_key(|f| f.pos);
    Ok((found, counts))
}

pub(super) fn select_frontier(
    knowledge: &Knowledge,
    origin: Pos,
    informed: Option<Pos>,
    draws: &mut impl DrawSource,
) -> Checked<(Option<Pos>, ComputeCounts)> {
    if let Some(site) = informed {
        knowledge.kind(site)?; // Coordinate validity does not reveal or require topology.
    }
    let (frontiers, counts) = frontiers(knowledge, origin)?;
    let candidates = if let Some(site) = informed {
        let rank = |f: &Frontier| {
            (
                f.unknown_neighbors
                    .iter()
                    .map(|p| p.x.abs_diff(site.x) + p.y.abs_diff(site.y))
                    .min()
                    .expect("a frontier has unknown neighbors"),
                f.distance,
            )
        };
        let best = frontiers.iter().map(rank).min();
        frontiers
            .iter()
            .filter(|f| Some(rank(f)) == best)
            .map(|f| f.pos)
            .collect::<Vec<_>>()
    } else {
        frontiers.iter().map(|f| f.pos).collect::<Vec<_>>()
    };
    Ok((choose(&candidates, draws)?, counts))
}

fn check_observation(
    origin: Pos,
    observation: &Observation,
    width: u32,
    height: u32,
) -> Checked<()> {
    observation.validate(width, height)?;
    if observation.origin != origin {
        return Err(error(
            "navigation.observation",
            "must be fresh at the supplied origin",
        ));
    }
    Ok(())
}

pub(super) fn route_step(
    knowledge: &Knowledge,
    origin: Pos,
    goals: &[Pos],
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<(Navigation, ComputeCounts)> {
    check_origin(knowledge, origin)?;
    let (width, height) = knowledge.dimensions();
    check_observation(origin, observation, width, height)?;
    if observation.cells.len() != origin.neighbors(width, height).len() + 1 {
        return Err(error(
            "navigation.observation",
            "must include every in-bounds local cell",
        ));
    }
    for cell in &observation.cells {
        let kind = knowledge.kind(cell.pos)?;
        if (kind == CellKnowledge::KnownOpen && !cell.open)
            || (kind == CellKnowledge::KnownSolid && cell.open)
        {
            return Err(error(
                "navigation.observation",
                "conflicts with private fixed topology",
            ));
        }
    }
    for &goal in goals {
        knowledge.kind(goal)?;
    }
    if goals.contains(&origin) {
        return Ok((Navigation::AtGoal, ComputeCounts::default()));
    }
    // Reverse multi-source BFS includes every privately open goal. Occupancy
    // cannot change the nearest distance or admit a longer route to another goal.
    let (distance, counts) = distances(knowledge, goals)?;
    let Some(current) = distance[index(origin, width)] else {
        return Ok((Navigation::Unreachable, counts));
    };
    let candidates = origin
        .neighbors(width, height)
        .into_iter()
        .filter(|&next| {
            distance[index(next, width)] == Some(current - 1)
                && observation
                    .cells
                    .iter()
                    .any(|cell| cell.pos == next && cell.open && cell.occupants < 2)
        })
        .collect::<Vec<_>>();
    let step = choose(&candidates, draws)?.map_or(Navigation::Blocked, Navigation::Move);
    Ok((step, counts))
}

pub(super) fn wander(
    origin: Pos,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Navigation> {
    // No private map is needed. Validate local structure against the maximum
    // admitted world dimensions; smaller-world boundaries are in the observation.
    check_observation(origin, observation, 125, 125)?;
    let candidates = origin
        .neighbors(125, 125)
        .into_iter()
        .filter(|&next| {
            observation
                .cells
                .iter()
                .any(|cell| cell.pos == next && cell.open && cell.occupants < 2)
        })
        .collect::<Vec<_>>();
    Ok(choose(&candidates, draws)?.map_or(Navigation::Blocked, Navigation::Move))
}
