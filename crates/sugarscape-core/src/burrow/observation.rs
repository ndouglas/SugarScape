//! Two-hop local sensing and the explicitly supplied global exit scaffold.

use super::{state::UnitLocation, Pos, World};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedCell {
    pub pos: Pos,
    pub occupants: u32,
    pub loose: u32,
    pub recent: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub origin: Pos,
    pub open: Vec<ObservedCell>,
    pub frontier: Vec<Pos>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExitNeighbor {
    pub pos: Pos,
    pub distance: u32,
    pub occupants: u32,
}

/// Deterministic search work, separate from in-world action costs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct BfsStats {
    pub calls: u64,
    pub visits: u64,
    pub peak_queue: u64,
}

impl BfsStats {
    pub(super) fn include(&mut self, other: Self) {
        self.calls += other.calls;
        self.visits += other.visits;
        self.peak_queue = self.peak_queue.max(other.peak_queue);
    }

    pub(super) fn queued(&mut self, count: usize) {
        self.peak_queue = self.peak_queue.max(count as u64);
    }
}

pub(super) fn is_recent(now: u64, born: u64, window: u64) -> bool {
    now.checked_sub(born).is_some_and(|age| age < window)
}

// Tests retain the specified plain API; production consumes measured wrappers.
#[cfg(test)]
pub(crate) fn observe(world: &World, worker: u32) -> Observation {
    observe_measured(world, worker).0
}

pub(crate) fn observe_measured(world: &World, worker: u32) -> (Observation, BfsStats) {
    let origin = world.workers[worker as usize].pos;
    let mut reached = BTreeMap::from([(origin, 0)]);
    let mut queue = VecDeque::from([origin]);
    let mut frontier = BTreeSet::new();
    let mut stats = BfsStats {
        calls: 1,
        peak_queue: 1,
        ..Default::default()
    };
    while let Some(pos) = queue.pop_front() {
        stats.visits += 1;
        let depth = reached[&pos];
        for next in pos.neighbors(world.setup.width, world.setup.height) {
            let index = world.index(next).unwrap();
            if world.open[index] {
                if depth < 2 && !reached.contains_key(&next) {
                    reached.insert(next, depth + 1);
                    queue.push_back(next);
                    stats.queued(queue.len());
                }
            } else if world.diggable[index] {
                frontier.insert(next);
            }
        }
    }
    let mut cells: BTreeMap<_, _> = reached
        .keys()
        .map(|&pos| {
            (
                pos,
                ObservedCell {
                    pos,
                    occupants: 0,
                    loose: 0,
                    recent: 0,
                },
            )
        })
        .collect();
    for worker in &world.workers {
        if let Some(cell) = cells.get_mut(&worker.pos) {
            cell.occupants += 1;
        }
    }
    for unit in world.units.values() {
        if let UnitLocation::Loose(pos) = unit.location {
            if let Some(cell) = cells.get_mut(&pos) {
                cell.loose = cell
                    .loose
                    .checked_add(1)
                    .expect("observed loose count overflow");
                if is_recent(world.tick, unit.born, world.config.freshness_window) {
                    cell.recent = cell
                        .recent
                        .checked_add(1)
                        .expect("observed recent count overflow");
                }
            }
        }
    }
    (
        Observation {
            origin,
            open: cells.into_values().collect(),
            frontier: frontier.into_iter().collect(),
        },
        stats,
    )
}

// Tests retain the specified plain API; production consumes measured wrappers.
#[cfg(test)]
pub(crate) fn exit_distances(world: &World) -> Vec<Option<u32>> {
    exit_distances_measured(world).0
}

pub(crate) fn exit_distances_measured(world: &World) -> (Vec<Option<u32>>, BfsStats) {
    let mut distances = vec![None; world.open.len()];
    distances[world.index(world.setup.exit).unwrap()] = Some(0);
    let mut queue = VecDeque::from([world.setup.exit]);
    let mut stats = BfsStats {
        calls: 1,
        peak_queue: 1,
        ..Default::default()
    };
    while let Some(pos) = queue.pop_front() {
        stats.visits += 1;
        let distance = distances[world.index(pos).unwrap()].unwrap();
        for next in pos.neighbors(world.setup.width, world.setup.height) {
            let index = world.index(next).unwrap();
            if world.open[index] && distances[index].is_none() {
                distances[index] = Some(distance + 1);
                queue.push_back(next);
                stats.queued(queue.len());
            }
        }
    }
    (distances, stats)
}
