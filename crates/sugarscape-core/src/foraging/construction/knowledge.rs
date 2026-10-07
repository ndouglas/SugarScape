use super::{observation::Observation, Checked, Pos};
use crate::config::FieldError;
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum CellKnowledge {
    Unknown,
    KnownOpen,
    KnownSolid { diggable: bool },
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct KnownCell {
    pub pos: Pos,
    pub kind: CellKnowledge,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Knowledge {
    width: u32,
    height: u32,
    cells: Vec<CellKnowledge>,
}
impl Knowledge {
    pub(super) fn new(width: u32, height: u32) -> Checked<Self> {
        super::setup::dimensions(width, height)?;
        Ok(Self {
            width,
            height,
            cells: vec![CellKnowledge::Unknown; (width * height) as usize],
        })
    }
    pub(super) fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn index(&self, pos: Pos) -> Checked<usize> {
        if pos.x >= self.width || pos.y >= self.height {
            return Err(vec![FieldError::new(
                "knowledge.pos",
                "must be inside the grid",
            )]);
        }
        Ok((pos.y * self.width + pos.x) as usize)
    }
    pub(super) fn kind(&self, pos: Pos) -> Checked<CellKnowledge> {
        Ok(self.cells[self.index(pos)?])
    }
    pub(super) fn learn(&mut self, observation: &Observation) -> Checked<LearnDelta> {
        observation.validate(self.width, self.height)?;
        // Prepare every classification before mutation, so a late conflict rolls back all learning.
        let mut updates = vec![];
        let mut delta = LearnDelta::default();
        for cell in &observation.cells {
            let index = self.index(cell.pos)?;
            let observed = if cell.open {
                CellKnowledge::KnownOpen
            } else {
                CellKnowledge::KnownSolid {
                    diggable: cell.diggable,
                }
            };
            match (self.cells[index], observed) {
                (CellKnowledge::Unknown, kind) => {
                    updates.push((index, kind));
                    delta.first += 1;
                }
                (CellKnowledge::KnownSolid { diggable: true }, CellKnowledge::KnownOpen) => {
                    updates.push((index, CellKnowledge::KnownOpen));
                    delta.observed_revisions += 1;
                }
                (kind, same) if kind == same => {}
                _ => {
                    return Err(vec![FieldError::new(
                        "knowledge",
                        "illegal terrain observation revision",
                    )])
                }
            }
        }
        for (index, kind) in updates {
            self.cells[index] = kind;
        }
        Ok(delta)
    }
    pub(super) fn confirm_dig(&mut self, origin: Pos, target: Pos) -> Checked<LearnDelta> {
        if self.kind(origin)? != CellKnowledge::KnownOpen
            || !super::setup::neighbors(origin, self.width, self.height).contains(&target)
            || self.kind(target)? != (CellKnowledge::KnownSolid { diggable: true })
        {
            return Err(vec![FieldError::new(
                "knowledge.confirm_dig",
                "requires adjacent known-open origin and diggable-solid target",
            )]);
        }
        let index = self.index(target)?;
        self.cells[index] = CellKnowledge::KnownOpen;
        Ok(LearnDelta {
            dig_confirmations: 1,
            ..Default::default()
        })
    }
    pub(super) fn known(&self) -> Vec<KnownCell> {
        let mut known: Vec<_> = self
            .cells
            .iter()
            .enumerate()
            .filter_map(|(index, &kind)| {
                (kind != CellKnowledge::Unknown).then_some(KnownCell {
                    pos: Pos {
                        x: index as u32 % self.width,
                        y: index as u32 / self.width,
                    },
                    kind,
                })
            })
            .collect();
        known.sort_by_key(|c| c.pos);
        known
    }
    pub(super) fn counts(&self) -> (u32, u32, u32) {
        self.cells
            .iter()
            .fold((0, 0, 0), |(open, solid, diggable), kind| match kind {
                CellKnowledge::KnownOpen => (open + 1, solid, diggable),
                CellKnowledge::KnownSolid { diggable: d } => {
                    (open, solid + 1, diggable + u32::from(*d))
                }
                CellKnowledge::Unknown => (open, solid, diggable),
            })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct LearnDelta {
    pub(super) first: u64,
    pub(super) observed_revisions: u64,
    pub(super) dig_confirmations: u64,
}
