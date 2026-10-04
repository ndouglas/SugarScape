//! Validate each transaction completely before changing authoritative state.

use super::{
    state::{Unit, UnitLocation},
    Pos, World,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Move(Pos),
    Dig(Pos),
    Pickup,
    Drop,
    Dispose,
    Wait,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Blocked { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionEvent {
    pub tick: u64,
    pub worker: u32,
    pub action: Action,
    pub outcome: Outcome,
    pub material: Option<u64>,
    pub from: Pos,
    pub to: Pos,
}

impl World {
    // Stage three supplies the production caller; tests exercise these transactions now.
    #[allow(dead_code)]
    pub(crate) fn apply(&mut self, worker: u32, action: Action) -> ActionEvent {
        let from = self
            .workers
            .get(worker as usize)
            .map_or(self.setup.exit, |w| w.pos);
        let result = self.resolve(worker, &action);
        let to = self.workers.get(worker as usize).map_or(from, |w| w.pos);
        let (outcome, material) = match result {
            Ok(material) => (Outcome::Success, material),
            Err(reason) => (
                Outcome::Blocked {
                    reason: format!("worker {worker} at ({},{}): {reason}", from.x, from.y),
                },
                None,
            ),
        };
        ActionEvent {
            tick: self.tick,
            worker,
            action,
            outcome,
            material,
            from,
            to,
        }
    }

    fn resolve(&mut self, worker: u32, action: &Action) -> Result<Option<u64>, String> {
        let index = worker as usize;
        let w = self
            .workers
            .get(index)
            .ok_or_else(|| format!("unknown worker {worker}"))?;
        let pos = w.pos;
        let carried = w.carried;
        match *action {
            Action::Move(to) => {
                let destination = self.index(to).ok_or("move destination out of bounds")?;
                if !pos
                    .neighbors(self.setup.width, self.setup.height)
                    .contains(&to)
                {
                    return Err("move must reach a four-neighbor cell".into());
                }
                if !self.open[destination] {
                    return Err("move destination is solid".into());
                }
                if self.workers.iter().filter(|w| w.pos == to).count() >= 2 {
                    return Err("move destination is at capacity".into());
                }
                let loaded_moves = if carried.is_some() {
                    w.loaded_moves
                        .checked_add(1)
                        .ok_or("loaded move counter overflow")?
                } else {
                    0
                };
                self.workers[index].pos = to;
                self.workers[index].loaded_moves = loaded_moves;
                Ok(carried)
            }
            Action::Dig(to) => {
                if carried.is_some() {
                    return Err("cannot dig with full hands".into());
                }
                let destination = self.index(to).ok_or("dig destination out of bounds")?;
                if !pos
                    .neighbors(self.setup.width, self.setup.height)
                    .contains(&to)
                {
                    return Err("dig must reach a four-neighbor cell".into());
                }
                if self.open[destination] {
                    return Err("dig destination is already open".into());
                }
                if !self.diggable[destination] {
                    return Err("solid face is non-diggable".into());
                }
                let next = self
                    .next_material
                    .checked_add(1)
                    .ok_or("material ID overflow")?;
                let excavated = self
                    .excavated
                    .checked_add(1)
                    .ok_or("excavation counter overflow")?;
                let id = self.next_material;
                self.open[destination] = true;
                self.units.insert(
                    id,
                    Unit {
                        id,
                        born: self.tick,
                        location: UnitLocation::Carried(worker),
                    },
                );
                self.next_material = next;
                self.excavated = excavated;
                self.workers[index].carried = Some(id);
                self.workers[index].loaded_moves = 0;
                self.workers[index].target = None;
                Ok(Some(id))
            }
            Action::Pickup => {
                if carried.is_some() {
                    return Err("cannot pick up with full hands".into());
                }
                let id = self
                    .units
                    .iter()
                    .find_map(|(id, unit)| {
                        (unit.location == UnitLocation::Loose(pos)).then_some(*id)
                    })
                    .ok_or("no loose spoil on current cell")?;
                self.units.get_mut(&id).unwrap().location = UnitLocation::Carried(worker);
                self.workers[index].carried = Some(id);
                self.workers[index].loaded_moves = 0;
                self.workers[index].target = None;
                Ok(Some(id))
            }
            Action::Drop | Action::Dispose => {
                let id = carried.ok_or("cannot handle spoil with empty hands")?;
                if *action == Action::Dispose && pos != self.setup.exit {
                    return Err("disposal requires the exit".into());
                }
                let location = if *action == Action::Dispose {
                    UnitLocation::Disposed
                } else {
                    UnitLocation::Loose(pos)
                };
                self.units.get_mut(&id).unwrap().location = location;
                self.workers[index].carried = None;
                self.workers[index].loaded_moves = 0;
                Ok(Some(id))
            }
            Action::Wait => Ok(None),
        }
    }
}
