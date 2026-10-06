use super::{observation::Observation, Checked, Pos};
use crate::config::FieldError;
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum CellKnowledge {
    Unknown,
    KnownOpen,
    KnownSolid,
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
    pub(super) fn learn(&mut self, observation: &Observation) -> Checked<u64> {
        observation.validate(self.width, self.height)?;
        // Prepare every classification before mutation, so a late conflict rolls back all learning.
        let mut updates = vec![];
        for cell in &observation.cells {
            let index = self.index(cell.pos)?;
            let observed = if cell.open {
                CellKnowledge::KnownOpen
            } else {
                CellKnowledge::KnownSolid
            };
            match (self.cells[index], observed) {
                (CellKnowledge::Unknown, kind) => updates.push((index, kind)),
                (kind, same) if kind == same => {}
                _ => {
                    return Err(vec![FieldError::new(
                        "knowledge",
                        "fixed topology observation conflicts",
                    )])
                }
            }
        }
        let learned = updates.len() as u64;
        for (index, kind) in updates {
            self.cells[index] = kind;
        }
        Ok(learned)
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
    pub(super) fn counts(&self) -> (u32, u32) {
        self.cells
            .iter()
            .fold((0, 0), |(open, solid), kind| match kind {
                CellKnowledge::KnownOpen => (open + 1, solid),
                CellKnowledge::KnownSolid => (open, solid + 1),
                CellKnowledge::Unknown => (open, solid),
            })
    }
}
