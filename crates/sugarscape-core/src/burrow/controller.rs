//! Decisions use owned local observations, never world or material provenance.

use super::{
    observation::BfsStats, state::Worker, Action, Cue, ExitNeighbor, LabConfig, Observation,
    ObservedCell, Pos, Transport,
};
use crate::rng::SimRng;
use rand::Rng;
use std::collections::{BTreeMap, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerView {
    pub id: u32,
    pub pos: Pos,
    pub carrying: bool,
    pub target: Option<Pos>,
    pub loaded_moves: u32,
}

impl From<&Worker> for WorkerView {
    fn from(w: &Worker) -> Self {
        Self {
            id: w.id,
            pos: w.pos,
            carrying: w.carried.is_some(),
            target: w.target,
            loaded_moves: w.loaded_moves,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub action: Action,
    pub target: Option<Pos>,
    /// Newly chosen frontier face, including selections immediately dug.
    pub selected: Option<Pos>,
}

pub(crate) fn target_weight(c: &LabConfig, count: u32) -> u32 {
    if c.cue == Cue::Responsive && count >= c.minimum_recent_units {
        c.response_weight
    } else {
        1
    }
}

pub(crate) fn select_ticket(weights: &[u32], mut ticket: u64) -> usize {
    for (index, &weight) in weights.iter().enumerate() {
        if ticket < u64::from(weight) {
            return index;
        }
        ticket -= u64::from(weight);
    }
    panic!("frontier ticket exceeds total weight");
}

pub(super) fn adjacent(a: Pos, b: Pos) -> bool {
    u64::from(a.x.abs_diff(b.x)) + u64::from(a.y.abs_diff(b.y)) == 1
}

// Stage three uses the measured entry point; retain the specified plain API.
#[allow(dead_code)]
pub(crate) fn decide(
    o: &Observation,
    w: &WorkerView,
    c: &LabConfig,
    at_exit: bool,
    outward: &[ExitNeighbor],
    rng: &mut SimRng,
) -> Decision {
    decide_measured(o, w, c, at_exit, outward, rng).0
}

pub(crate) fn decide_measured(
    o: &Observation,
    w: &WorkerView,
    c: &LabConfig,
    at_exit: bool,
    outward: &[ExitNeighbor],
    rng: &mut SimRng,
) -> (Decision, BfsStats) {
    let mut stats = BfsStats::default();
    let finish = |action, target, selected| Decision {
        action,
        target,
        selected,
    };
    if w.carrying {
        let action = if at_exit {
            Action::Dispose
        } else if c.transport == Transport::Relay && w.loaded_moves >= c.relay_distance {
            Action::Drop
        } else {
            let available: Vec<_> = outward.iter().filter(|n| n.occupants < 2).collect();
            if available.is_empty() {
                Action::Wait
            } else {
                Action::Move(available[rng.gen_range(0..available.len() as u32) as usize].pos)
            }
        };
        return (finish(action, None, None), stats);
    }
    if o.open
        .iter()
        .any(|cell| cell.pos == w.pos && cell.loose > 0)
        && rng.gen_bool(0.5)
    {
        return (finish(Action::Pickup, w.target, None), stats);
    }
    let (reachable, search) = routes(o, w.pos, false);
    stats.include(search);
    let mut targets: Vec<_> = o
        .frontier
        .iter()
        .copied()
        .filter(|target| {
            !o.open.iter().any(|cell| cell.pos == *target)
                && reachable.keys().any(|&pos| adjacent(pos, *target))
        })
        .collect();
    targets.sort_unstable();
    targets.dedup();
    let mut selected = None;
    let target = w
        .target
        .filter(|target| targets.contains(target))
        .or_else(|| {
            if targets.is_empty() {
                return None;
            }
            let weights: Vec<_> = targets
                .iter()
                .map(|&target| {
                    // Every observed approach cell contributes once; shared approaches
                    // independently inform each target rather than multiplying units.
                    let count: u64 = o
                        .open
                        .iter()
                        .filter(|cell| adjacent(cell.pos, target))
                        .map(|cell| u64::from(cell.recent))
                        .sum();
                    target_weight(c, count.min(u64::from(u32::MAX)) as u32)
                })
                .collect();
            let total: u64 = weights.iter().map(|&weight| u64::from(weight)).sum();
            let target = targets[select_ticket(&weights, rng.gen_range(0..total))];
            selected = Some(target);
            Some(target)
        });
    if let Some(target) = target {
        if adjacent(w.pos, target) {
            return (finish(Action::Dig(target), None, selected), stats);
        }
        let (legal, search) = routes(o, w.pos, true);
        stats.include(search);
        let next = legal
            .iter()
            .filter(|(pos, _)| adjacent(**pos, target))
            .min_by_key(|(pos, route)| (route.distance, **pos))
            .and_then(|(_, route)| route.first);
        return (
            finish(
                next.map_or(Action::Wait, Action::Move),
                Some(target),
                selected,
            ),
            stats,
        );
    }
    let neighbors: Vec<_> = o
        .open
        .iter()
        .filter(|cell| adjacent(w.pos, cell.pos) && cell.occupants < 2)
        .collect();
    let action = if neighbors.is_empty() {
        Action::Wait
    } else {
        Action::Move(neighbors[rng.gen_range(0..neighbors.len() as u32) as usize].pos)
    };
    (finish(action, None, None), stats)
}

struct Route {
    distance: u32,
    first: Option<Pos>,
}

/// Geometry keeps targets valid; occupancy is checked separately for movement.
fn routes(o: &Observation, origin: Pos, legal: bool) -> (BTreeMap<Pos, Route>, BfsStats) {
    let cells: BTreeMap<Pos, &ObservedCell> = o.open.iter().map(|cell| (cell.pos, cell)).collect();
    let mut reached = BTreeMap::from([(
        origin,
        Route {
            distance: 0,
            first: None,
        },
    )]);
    let mut queue = VecDeque::from([origin]);
    let mut stats = BfsStats {
        calls: 1,
        peak_queue: 1,
        ..Default::default()
    };
    while let Some(pos) = queue.pop_front() {
        stats.visits += 1;
        let distance = reached[&pos].distance;
        let first = reached[&pos].first;
        for (&next, cell) in &cells {
            if adjacent(pos, next) && (!legal || cell.occupants < 2) && !reached.contains_key(&next)
            {
                reached.insert(
                    next,
                    Route {
                        distance: distance + 1,
                        first: first.or(Some(next)),
                    },
                );
                queue.push_back(next);
                stats.queued(queue.len());
            }
        }
    }
    (reached, stats)
}
