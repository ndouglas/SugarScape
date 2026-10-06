use super::{Checked, Pos, Setup};
use crate::config::FieldError;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ObservedCell {
    pub(super) pos: Pos,
    pub(super) open: bool,
    pub(super) occupants: u32,
    pub(super) food: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Observation {
    pub(super) origin: Pos,
    pub(super) cells: Vec<ObservedCell>,
}
impl Observation {
    pub(super) fn validate(&self, width: u32, height: u32) -> Checked<()> {
        super::setup::dimensions(width, height)?;
        let mut allowed: BTreeSet<_> = self.origin.neighbors(width, height).into_iter().collect();
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
            if c.occupants > 2 || (!c.open && (c.occupants != 0 || c.food)) {
                errors.push(FieldError::new(
                    format!("observation.cells[{i}]"),
                    "invalid occupancy or food for cell classification",
                ));
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
pub(super) fn observe(
    setup: &Setup,
    origin: Pos,
    occupants: &BTreeMap<Pos, u32>,
    food: &BTreeSet<Pos>,
) -> Checked<Observation> {
    super::setup::dimensions(setup.width, setup.height)?;
    let cells = std::iter::once(origin)
        .chain(origin.neighbors(setup.width, setup.height))
        .map(|pos| ObservedCell {
            pos,
            open: setup.open.contains(&pos),
            occupants: occupants.get(&pos).copied().unwrap_or(0),
            food: food.contains(&pos),
        })
        .collect();
    let observation = Observation { origin, cells };
    observation.validate(setup.width, setup.height)?;
    Ok(observation)
}
