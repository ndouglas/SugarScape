//! Geometry uses private classifications; only the next step uses fresh occupancy.
//! Task 4 owns retained frontier/face destinations and validates/learns each view.
use super::{
    draws::{choose, DrawSource},
    knowledge::{CellKnowledge, Knowledge},
    metrics::ComputeCounts,
    observation::Observation,
    setup::neighbors,
    Checked, Pos,
};
use crate::config::FieldError;
use std::collections::{BTreeSet, VecDeque};

// Staged Task 4 controller route outcomes and retained destinations.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Navigation {
    AtGoal,
    Move(Pos),
    Blocked,
    Unreachable,
}
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Frontier {
    pub(super) pos: Pos,
    pub(super) unknown_neighbors: Vec<Pos>,
    pub(super) distance: u32,
}
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Face {
    pub(super) pos: Pos,
    pub(super) approaches: Vec<Pos>,
    pub(super) distance: u32,
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
fn manhattan(a: Pos, b: Pos) -> u32 {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

/// Each distance array and queue contains at most width * height entries.
/// Visits count dequeues; the peak includes the initial unique seed queue.
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
        for next in neighbors(pos, width, height) {
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

/// Scans count reachable KnownOpen cells whose neighbors are examined.
/// Candidate and unknown-neighbor order is Pos, independently of BFS order.
// Staged Task 4 food/outlet exploration and destination validity.
#[allow(dead_code)]
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
        for next in neighbors(pos, width, height) {
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

fn reachable_approaches(knowledge: &Knowledge, face: Pos, distance: &[Option<u32>]) -> Vec<Pos> {
    let (width, height) = knowledge.dimensions();
    let mut approaches: Vec<_> = neighbors(face, width, height)
        .into_iter()
        .filter(|&p| distance[index(p, width)].is_some())
        .collect();
    approaches.sort_unstable();
    approaches
}

/// Scans count every remembered solid entry examined, including protected walls
/// and diggable walls without a reachable privately open approach.
// Staged Task 4 exploration-first excavation and retained-face validity.
#[allow(dead_code)]
pub(super) fn faces(knowledge: &Knowledge, origin: Pos) -> Checked<(Vec<Face>, ComputeCounts)> {
    check_origin(knowledge, origin)?;
    let (width, _) = knowledge.dimensions();
    let (distance, mut counts) = distances(knowledge, &[origin])?;
    let mut found = Vec::new();
    for cell in knowledge.known() {
        let CellKnowledge::KnownSolid { diggable } = cell.kind else {
            continue;
        };
        counts.face_scans += 1;
        if !diggable {
            continue;
        }
        let approaches = reachable_approaches(knowledge, cell.pos, &distance);
        let nearest = approaches
            .iter()
            .filter_map(|&p| distance[index(p, width)])
            .min();
        if let Some(distance) = nearest {
            found.push(Face {
                pos: cell.pos,
                approaches,
                distance,
            });
        }
    }
    // Knowledge::known is coordinate ordered, so eligible faces stay ordered.
    Ok((found, counts))
}

/// Returns all reachable known-open approaches in Pos order. In-bounds faces
/// that are unknown, open, protected or unreachable return an empty list.
// Staged Task 4 travel to a retained eligible dig face.
#[allow(dead_code)]
pub(super) fn face_approaches(
    knowledge: &Knowledge,
    origin: Pos,
    face: Pos,
) -> Checked<(Vec<Pos>, ComputeCounts)> {
    check_origin(knowledge, origin)?;
    if knowledge.kind(face)? != (CellKnowledge::KnownSolid { diggable: true }) {
        return Ok((Vec::new(), ComputeCounts::default()));
    }
    let (distance, counts) = distances(knowledge, &[origin])?;
    Ok((reachable_approaches(knowledge, face, &distance), counts))
}

// Staged Task 4 food exploration and unknown-outlet exploration use one rule.
#[allow(dead_code)]
pub(super) fn select_frontier(
    knowledge: &Knowledge,
    origin: Pos,
    site: Option<Pos>,
    draws: &mut impl DrawSource,
) -> Checked<(Option<Pos>, ComputeCounts)> {
    if let Some(site) = site {
        knowledge.kind(site)?; // Coordinate label supplies no route or classification.
    }
    let (frontiers, counts) = frontiers(knowledge, origin)?;
    let candidates = if let Some(site) = site {
        let rank = |f: &Frontier| {
            (
                f.unknown_neighbors
                    .iter()
                    .map(|&p| manhattan(p, site))
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

// Staged Task 4 informed or uninformed excavation after open exploration.
#[allow(dead_code)]
pub(super) fn select_face(
    knowledge: &Knowledge,
    origin: Pos,
    site: Option<Pos>,
    draws: &mut impl DrawSource,
) -> Checked<(Option<Pos>, ComputeCounts)> {
    if let Some(site) = site {
        knowledge.kind(site)?;
    }
    let (faces, counts) = faces(knowledge, origin)?;
    let candidates = if let Some(site) = site {
        let rank = |f: &Face| (manhattan(f.pos, site), f.distance);
        let best = faces.iter().map(rank).min();
        faces
            .iter()
            .filter(|f| Some(rank(f)) == best)
            .map(|f| f.pos)
            .collect::<Vec<_>>()
    } else {
        faces.iter().map(|f| f.pos).collect::<Vec<_>>()
    };
    Ok((choose(&candidates, draws)?, counts))
}

/// Current sensing may reveal a stale diggable wall opened; this check neither
/// revises knowledge nor lets that cell enter the remembered route graph.
fn check_observation(knowledge: &Knowledge, origin: Pos, observation: &Observation) -> Checked<()> {
    let (width, height) = knowledge.dimensions();
    observation.validate(width, height)?;
    if observation.origin != origin {
        return Err(error(
            "navigation.observation",
            "must be fresh at the supplied origin",
        ));
    }
    for cell in &observation.cells {
        let conflict = match knowledge.kind(cell.pos)? {
            CellKnowledge::Unknown => false,
            CellKnowledge::KnownOpen => !cell.open,
            CellKnowledge::KnownSolid { diggable } => {
                if cell.open {
                    !diggable
                } else {
                    diggable != cell.diggable
                }
            }
        };
        if conflict {
            return Err(error(
                "navigation.observation",
                "illegal local terrain revision",
            ));
        }
    }
    Ok(())
}

/// Reverse multi-source private BFS keeps nearest goals fixed. Capacity filters
/// only decreasing-distance immediate steps; blocked routes never take detours.
// Staged Task 4 food, spoil, frontier and face-approach movement.
#[allow(dead_code)]
pub(super) fn route_step(
    knowledge: &Knowledge,
    origin: Pos,
    goals: &[Pos],
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<(Navigation, ComputeCounts)> {
    check_origin(knowledge, origin)?;
    check_observation(knowledge, origin, observation)?;
    for &goal in goals {
        knowledge.kind(goal)?;
    }
    if goals.contains(&origin) {
        return Ok((Navigation::AtGoal, ComputeCounts::default()));
    }
    let (width, height) = knowledge.dimensions();
    let (distance, counts) = distances(knowledge, goals)?;
    let Some(current) = distance[index(origin, width)] else {
        return Ok((Navigation::Unreachable, counts));
    };
    let candidates: Vec<_> = neighbors(origin, width, height)
        .into_iter()
        .filter(|&next| {
            distance[index(next, width)] == Some(current - 1)
                && observation
                    .cells
                    .iter()
                    .any(|c| c.pos == next && c.open && c.occupants < 2)
        })
        .collect();
    Ok((
        choose(&candidates, draws)?.map_or(Navigation::Blocked, Navigation::Move),
        counts,
    ))
}

/// Requires a complete view validated against actual dimensions by Task 4
/// before dispatch. Without dimensions this helper checks only the locally
/// enforceable envelope, identity, origin openness and classification payloads.
// Staged Task 4 empty-search fallback movement.
#[allow(dead_code)]
pub(super) fn wander(
    origin: Pos,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Navigation> {
    if origin.x >= 125 || origin.y >= 125 || observation.origin != origin {
        return Err(error(
            "navigation.observation",
            "requires a valid matching origin",
        ));
    }
    let adjacent = neighbors(origin, 125, 125);
    let mut seen = BTreeSet::new();
    for cell in &observation.cells {
        if (cell.pos != origin && !adjacent.contains(&cell.pos)) || !seen.insert(cell.pos) {
            return Err(error(
                "navigation.observation",
                "must contain distinct current/cardinal cells",
            ));
        }
        if (cell.open && cell.diggable)
            || cell.occupants > 2
            || (!cell.open && (cell.occupants != 0 || cell.food))
        {
            return Err(error(
                "navigation.observation",
                "invalid occupancy or food for cell classification",
            ));
        }
    }
    if !observation.cells.iter().any(|c| c.pos == origin && c.open) {
        return Err(error(
            "navigation.observation",
            "origin must be observed open",
        ));
    }
    let candidates: Vec<_> = adjacent
        .into_iter()
        .filter(|&next| {
            observation
                .cells
                .iter()
                .any(|c| c.pos == next && c.open && c.occupants < 2)
        })
        .collect();
    Ok(choose(&candidates, draws)?.map_or(Navigation::Blocked, Navigation::Move))
}
