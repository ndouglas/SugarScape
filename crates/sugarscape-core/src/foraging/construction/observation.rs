use super::{setup::neighbors, terrain::Terrain, Checked, Pos};
use crate::config::FieldError;
use std::collections::{BTreeMap, BTreeSet};
// Staged Task 2 navigation and Task 4 opportunity sensing.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ObservedCell {
    pub(super) pos: Pos,
    pub(super) open: bool,
    pub(super) diggable: bool,
    pub(super) occupants: u32,
    pub(super) food: bool,
}
// Staged Task 2 navigation and Task 4 opportunity sensing.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Observation {
    pub(super) origin: Pos,
    pub(super) cells: Vec<ObservedCell>,
}
#[allow(dead_code)]
impl Observation {
    pub(super) fn validate(&self, width: u32, height: u32) -> Checked<()> {
        super::setup::dimensions(width, height)?;
        let mut allowed: BTreeSet<_> = neighbors(self.origin, width, height).into_iter().collect();
        if self.origin.x >= width || self.origin.y >= height {
            return Err(vec![FieldError::new(
                "observation.origin",
                "must be inside the grid",
            )]);
        }
        allowed.insert(self.origin);
        let mut seen = BTreeSet::new();
        let mut errors = vec![];
        if !self.cells.iter().any(|c| c.pos == self.origin && c.open) {
            errors.push(FieldError::new(
                "observation.origin",
                "must be observed open",
            ));
        }
        for (i, c) in self.cells.iter().enumerate() {
            if !allowed.contains(&c.pos) || !seen.insert(c.pos) {
                errors.push(FieldError::new(
                    format!("observation.cells[{i}]"),
                    "must be a distinct local in-bounds cell",
                ));
            }
            if (c.open && c.diggable)
                || c.occupants > 2
                || (!c.open && (c.occupants != 0 || c.food))
            {
                errors.push(FieldError::new(
                    format!("observation.cells[{i}]"),
                    "invalid occupancy or food for cell classification",
                ));
            }
        }
        if seen != allowed {
            errors.push(FieldError::new(
                "observation.cells",
                "must include every current/cardinal cell",
            ));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
#[allow(dead_code)]
pub(super) fn observe(
    terrain: &Terrain,
    origin: Pos,
    occupants: &BTreeMap<Pos, u32>,
    food: &BTreeSet<Pos>,
) -> Checked<Observation> {
    let (width, height) = terrain.dimensions();
    let mut cells = vec![];
    for pos in std::iter::once(origin).chain(neighbors(origin, width, height)) {
        let open = terrain.is_open(pos)?;
        cells.push(ObservedCell {
            pos,
            open,
            diggable: terrain.is_diggable(pos)?,
            occupants: if open {
                occupants.get(&pos).copied().unwrap_or(0)
            } else {
                0
            },
            food: open && food.contains(&pos),
        });
    }
    let observation = Observation { origin, cells };
    observation.validate(width, height)?;
    Ok(observation)
}
