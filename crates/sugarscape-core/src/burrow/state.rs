//! Bounded geometry, stable worker/material identity and checked construction.

use super::{
    config::{checked_cells, MAX_WORKERS},
    fixtures, LabConfig,
};
use crate::{
    config::FieldError,
    rng::{self, SimRng},
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}

impl Pos {
    /// Fixed north, south, east, west order; every result stays inside the array.
    pub(super) fn neighbors(self, width: u32, height: u32) -> Vec<Self> {
        let mut out = Vec::with_capacity(4);
        if self.y > 0 {
            out.push(Self {
                x: self.x,
                y: self.y - 1,
            });
        }
        if self.y + 1 < height {
            out.push(Self {
                x: self.x,
                y: self.y + 1,
            });
        }
        if self.x + 1 < width {
            out.push(Self {
                x: self.x + 1,
                y: self.y,
            });
        }
        if self.x > 0 {
            out.push(Self {
                x: self.x - 1,
                y: self.y,
            });
        }
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InitialSpoil {
    pub pos: Pos,
    pub born: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Setup {
    pub width: u32,
    pub height: u32,
    pub exit: Pos,
    pub start_tick: u64,
    pub open: Vec<Pos>,
    pub diggable: Vec<Pos>,
    pub workers: Vec<Pos>,
    pub spoil: Vec<InitialSpoil>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Inventory {
    pub initial: u64,
    pub excavated: u64,
    pub carried: u64,
    pub loose: u64,
    pub disposed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Worker {
    pub id: u32,
    pub pos: Pos,
    pub carried: Option<u64>,
    pub target: Option<Pos>,
    pub loaded_moves: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UnitLocation {
    Loose(Pos),
    Carried(u32),
    Disposed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Unit {
    pub id: u64,
    pub born: u64,
    pub location: UnitLocation,
}

#[derive(Clone, Debug)]
pub struct World {
    pub(super) config: LabConfig,
    pub(super) setup: Setup,
    pub(super) tick: u64,
    pub(super) open: Vec<bool>,
    pub(super) diggable: Vec<bool>,
    pub(super) workers: Vec<Worker>,
    pub(super) units: BTreeMap<u64, Unit>,
    pub(super) next_material: u64,
    pub(super) excavated: u64,
    pub(super) rng: SimRng,
    pub(super) recording: super::runner::Recording,
}

impl World {
    pub fn new(config: LabConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        let setup = fixtures::setup(&config)?;
        Self::from_setup(config, setup, seed)
    }

    pub(crate) fn from_setup(
        config: LabConfig,
        setup: Setup,
        seed: u64,
    ) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let cells =
            checked_cells(setup.width, setup.height, "setup.dimensions").map_err(|e| vec![e])?;
        let mut errors = Vec::new();
        if setup.workers.is_empty() || setup.workers.len() > MAX_WORKERS as usize {
            errors.push(FieldError::new(
                "setup.workers",
                "must contain between 1 and 4096 workers",
            ));
        }
        if config.response_weight.checked_mul(cells as u32).is_none() {
            errors.push(FieldError::new(
                "response_weight",
                "setup frontier weight sum overflow",
            ));
        }
        if setup.start_tick.checked_add(1).is_none() {
            errors.push(FieldError::new("setup.start_tick", "tick advance overflow"));
        }
        for (field, positions) in [
            ("setup.open", &setup.open),
            ("setup.diggable", &setup.diggable),
        ] {
            if positions.len() > cells {
                errors.push(FieldError::new(field, "cell list exceeds array capacity"));
            }
            let mut seen = BTreeSet::new();
            for pos in positions {
                if pos.x >= setup.width || pos.y >= setup.height {
                    errors.push(FieldError::new(
                        field,
                        format!("cell ({},{}) is out of bounds", pos.x, pos.y),
                    ));
                }
                if !seen.insert(*pos) {
                    errors.push(FieldError::new(
                        field,
                        format!("duplicate cell ({},{})", pos.x, pos.y),
                    ));
                }
            }
        }
        // No cell arrays or world records are allocated until dimensions and lists are checked.
        if !errors.is_empty() {
            return Err(errors);
        }
        let open_set: BTreeSet<_> = setup.open.iter().copied().collect();
        if !open_set.contains(&setup.exit) {
            errors.push(FieldError::new(
                "setup.exit",
                "exit must be an in-bounds open cell",
            ));
        }
        let mut occupancy = BTreeMap::new();
        for (id, pos) in setup.workers.iter().enumerate() {
            if !open_set.contains(pos) {
                errors.push(FieldError::new(
                    format!("setup.workers.{id}"),
                    "spawn must be an in-bounds open cell",
                ));
            }
            let count = occupancy.entry(*pos).or_insert(0);
            *count += 1;
            if *count > 2 {
                errors.push(FieldError::new(
                    format!("setup.workers.{id}"),
                    "spawn exceeds capacity of two workers per cell",
                ));
            }
        }
        for (id, unit) in setup.spoil.iter().enumerate() {
            if !open_set.contains(&unit.pos) {
                errors.push(FieldError::new(
                    format!("setup.spoil.{id}"),
                    "spoil must occupy an in-bounds open cell",
                ));
            }
            if unit.born > setup.start_tick {
                errors.push(FieldError::new(
                    format!("setup.spoil.{id}.born"),
                    "material birth is in the future",
                ));
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        let connected = connected_cells(setup.exit, setup.width, setup.height, |pos| {
            open_set.contains(&pos)
        });
        if connected.len() != open_set.len() {
            return Err(vec![FieldError::new(
                "setup.open",
                "all open cells and spawns must be connected to the exit",
            )]);
        }
        let next_material = u64::try_from(setup.spoil.len())
            .map_err(|_| vec![FieldError::new("setup.spoil", "material ID overflow")])?;
        if next_material.checked_add(cells as u64).is_none() {
            return Err(vec![FieldError::new(
                "setup.spoil",
                "material ID overflow after excavation",
            )]);
        }
        let mut open = vec![false; cells];
        let mut diggable = vec![false; cells];
        for pos in &setup.open {
            open[(pos.y * setup.width + pos.x) as usize] = true;
        }
        for pos in &setup.diggable {
            diggable[(pos.y * setup.width + pos.x) as usize] = true;
        }
        let workers = setup
            .workers
            .iter()
            .enumerate()
            .map(|(id, pos)| Worker {
                id: id as u32,
                pos: *pos,
                carried: None,
                target: None,
                loaded_moves: 0,
            })
            .collect();
        let units = setup
            .spoil
            .iter()
            .enumerate()
            .map(|(id, spoil)| {
                (
                    id as u64,
                    Unit {
                        id: id as u64,
                        born: spoil.born,
                        location: UnitLocation::Loose(spoil.pos),
                    },
                )
            })
            .collect();
        let mut world = Self {
            config,
            tick: setup.start_tick,
            setup,
            open,
            diggable,
            workers,
            units,
            next_material,
            excavated: 0,
            rng: rng::seeded(seed),
            recording: super::runner::Recording::default(),
        };
        world.recording = super::runner::Recording::new(&world);
        Ok(world)
    }

    pub(super) fn index(&self, pos: Pos) -> Option<usize> {
        (pos.x < self.setup.width && pos.y < self.setup.height)
            .then(|| (pos.y * self.setup.width + pos.x) as usize)
    }

    pub fn inventory(&self) -> Inventory {
        let mut inventory = Inventory {
            initial: self.setup.spoil.len() as u64,
            excavated: self.excavated,
            carried: 0,
            loose: 0,
            disposed: 0,
        };
        for unit in self.units.values() {
            match unit.location {
                UnitLocation::Loose(_) => inventory.loose += 1,
                UnitLocation::Carried(_) => inventory.carried += 1,
                UnitLocation::Disposed => inventory.disposed += 1,
            }
        }
        inventory
    }

    pub fn check_invariants(&self) -> Result<(), String> {
        let i = self.inventory();
        if i.initial.checked_add(i.excavated)
            != i.carried
                .checked_add(i.loose)
                .and_then(|n| n.checked_add(i.disposed))
        {
            return Err("burrow material conservation failed".into());
        }
        let open_count = self.open.iter().filter(|open| **open).count() as u64;
        if (self.setup.open.len() as u64).checked_add(self.excavated) != Some(open_count) {
            return Err("burrow geometry conservation failed".into());
        }
        let exit_index = self
            .index(self.setup.exit)
            .ok_or("burrow exit out of bounds")?;
        if !self.open[exit_index] {
            return Err("burrow exit is solid".into());
        }
        let connected =
            connected_cells(self.setup.exit, self.setup.width, self.setup.height, |p| {
                self.open[self.index(p).unwrap()]
            });
        if connected.len() as u64 != open_count {
            return Err("burrow open cells disconnected from exit".into());
        }
        let mut occupancy = BTreeMap::new();
        for (index, worker) in self.workers.iter().enumerate() {
            if worker.id as usize != index {
                return Err(format!("burrow worker {} has inconsistent ID", worker.id));
            }
            if !self.index(worker.pos).is_some_and(|index| self.open[index]) {
                return Err(format!(
                    "burrow worker {} occupies a non-open cell",
                    worker.id
                ));
            }
            let count = occupancy.entry(worker.pos).or_insert(0);
            *count += 1;
            if *count > 2 {
                return Err("burrow worker capacity exceeded".into());
            }
            if let Some(id) = worker.carried {
                if !self
                    .units
                    .get(&id)
                    .is_some_and(|unit| unit.location == UnitLocation::Carried(worker.id))
                {
                    return Err(format!(
                        "burrow worker {} has inconsistent material {id}",
                        worker.id
                    ));
                }
            }
        }
        for (id, unit) in &self.units {
            if *id != unit.id || *id >= self.next_material {
                return Err(format!("burrow material {id} has inconsistent ID"));
            }
            if unit.born > self.tick {
                return Err(format!("burrow material {id} born in the future"));
            }
            match unit.location {
                UnitLocation::Loose(pos)
                    if !self.index(pos).is_some_and(|index| self.open[index]) =>
                {
                    return Err(format!("burrow material {id} lies on a non-open cell"))
                }
                UnitLocation::Carried(worker)
                    if !self
                        .workers
                        .get(worker as usize)
                        .is_some_and(|w| w.carried == Some(*id)) =>
                {
                    return Err(format!(
                        "burrow material {id} has inconsistent carrier {worker}"
                    ))
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn connected_cells(
    exit: Pos,
    width: u32,
    height: u32,
    is_open: impl Fn(Pos) -> bool,
) -> BTreeSet<Pos> {
    let mut reached = BTreeSet::from([exit]);
    let mut queue = VecDeque::from([exit]);
    while let Some(pos) = queue.pop_front() {
        for next in pos.neighbors(width, height) {
            if is_open(next) && reached.insert(next) {
                queue.push_back(next);
            }
        }
    }
    reached
}
