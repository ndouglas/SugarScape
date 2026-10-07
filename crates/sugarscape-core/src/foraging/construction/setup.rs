use super::Checked;
use super::{Parameters, Pos, Resource};
use crate::config::FieldError;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Setup {
    pub width: u32,
    pub height: u32,
    pub open: Vec<Pos>,
    pub diggable: Vec<Pos>,
    pub waste: Pos,
    pub nest: Vec<Pos>,
    pub workers: Vec<Pos>,
    pub food: Vec<Resource>,
    pub parameters: Parameters,
}
pub(super) fn dimensions(width: u32, height: u32) -> Checked<()> {
    let mut errors = vec![];
    for (field, value) in [("width", width), ("height", height)] {
        if !(3..=125).contains(&value) {
            errors.push(FieldError::new(field, "must be in [3,125]"));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
impl Setup {
    /// Aggregate errors against original list indices before dense allocation.
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = dimensions(self.width, self.height)
            .err()
            .unwrap_or_default();
        let mut open = BTreeSet::new();
        if self.open.is_empty() {
            errors.push(FieldError::new("open", "must not be empty"));
        }
        for (i, &p) in self.open.iter().enumerate() {
            if !open.insert(p) {
                errors.push(FieldError::new(format!("open[{i}]"), "duplicate open cell"));
            }
            if !self.contains(p) {
                errors.push(FieldError::new(
                    format!("open[{i}]"),
                    "must be inside the grid",
                ));
            }
        }
        let mut mask = BTreeSet::new();
        for (i, &p) in self.diggable.iter().enumerate() {
            if !mask.insert(p) {
                errors.push(FieldError::new(
                    format!("diggable[{i}]"),
                    "duplicate mask cell",
                ));
            }
            if !self.contains(p) {
                errors.push(FieldError::new(
                    format!("diggable[{i}]"),
                    "must be inside the grid",
                ));
            }
        }
        let mut nest = BTreeSet::new();
        for (i, &p) in self.nest.iter().enumerate() {
            if !nest.insert(p) {
                errors.push(FieldError::new(format!("nest[{i}]"), "duplicate nest cell"));
            }
            if !self.contains(p) || !open.contains(&p) {
                errors.push(FieldError::new(
                    format!("nest[{i}]"),
                    "must be an in-bounds open cell",
                ));
            }
        }
        if nest.len() < 2 {
            errors.push(FieldError::new(
                "nest",
                "must contain at least two distinct cells",
            ));
        }
        if let Some(&start) = nest.first() {
            let mut reached = BTreeSet::from([start]);
            let mut pending = vec![start];
            while let Some(p) = pending.pop() {
                for next in neighbors(p, self.width, self.height) {
                    if nest.contains(&next) && reached.insert(next) {
                        pending.push(next);
                    }
                }
            }
            if reached.len() != nest.len() {
                errors.push(FieldError::new("nest", "must be four-neighbor connected"));
            }
        }
        if !self.contains(self.waste) || !open.contains(&self.waste) || nest.contains(&self.waste) {
            errors.push(FieldError::new(
                "waste",
                "must be initially open outside the nest",
            ));
        }
        if let Some(&start) = nest.first() {
            let mut reached = BTreeSet::from([start]);
            let mut pending = vec![start];
            while let Some(p) = pending.pop() {
                for next in neighbors(p, self.width, self.height) {
                    if open.contains(&next) && reached.insert(next) {
                        pending.push(next);
                    }
                }
            }
            if !reached.contains(&self.waste) {
                errors.push(FieldError::new(
                    "waste",
                    "must connect to the nest through initial open cells",
                ));
            }
        }
        if !(1..=256).contains(&self.workers.len()) {
            errors.push(FieldError::new("workers", "must contain 1 to 256 spawns"));
        }
        let mut occupants = BTreeMap::new();
        for (i, &p) in self.workers.iter().enumerate() {
            if !self.contains(p) || !nest.contains(&p) {
                errors.push(FieldError::new(
                    format!("workers[{i}]"),
                    "must spawn in the nest",
                ));
            }
            let count = occupants.entry(p).or_insert(0usize);
            *count += 1;
            if *count > 2 {
                errors.push(FieldError::new(
                    format!("workers[{i}]"),
                    "cell capacity is two workers",
                ));
            }
        }
        if self.food.len() > 256 {
            errors.push(FieldError::new("food", "must contain at most 256 food"));
        }
        let mut ids = BTreeSet::new();
        let mut cells = BTreeSet::new();
        for (i, r) in self.food.iter().enumerate() {
            if !ids.insert(r.id) {
                errors.push(FieldError::new(
                    format!("food[{i}].id"),
                    "duplicate resource identity",
                ));
            }
            if !cells.insert(r.pos) {
                errors.push(FieldError::new(
                    format!("food[{i}].pos"),
                    "duplicate resource cell",
                ));
            }
            if !self.contains(r.pos) || nest.contains(&r.pos) || r.pos == self.waste {
                errors.push(FieldError::new(
                    format!("food[{i}].pos"),
                    "must be in bounds outside the nest and waste outlet",
                ));
            }
        }
        if let Err(es) = self.parameters.validate() {
            errors.extend(
                es.into_iter()
                    .map(|e| FieldError::new(format!("parameters.{}", e.field), e.message)),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    pub fn normalized(mut self) -> Result<Self, Vec<FieldError>> {
        self.validate()?;
        self.open.sort();
        self.nest.sort();
        self.diggable.sort();
        self.food.sort_by_key(|r| r.id);
        Ok(self)
    }
    pub(super) fn contains(&self, p: Pos) -> bool {
        p.x < self.width && p.y < self.height
    }
    pub fn site(&self, p: Pos) -> Checked<u64> {
        dimensions(self.width, self.height)?;
        if !self.contains(p) {
            return Err(vec![FieldError::new("site", "must be inside the grid")]);
        }
        Ok(u64::from(p.y) * u64::from(self.width) + u64::from(p.x))
    }
    pub fn position(&self, site: u64) -> Checked<Pos> {
        dimensions(self.width, self.height)?;
        if site >= u64::from(self.width) * u64::from(self.height) {
            return Err(vec![FieldError::new(
                "site",
                "must encode an in-bounds cell",
            )]);
        }
        Ok(Pos {
            x: (site % u64::from(self.width)) as u32,
            y: (site / u64::from(self.width)) as u32,
        })
    }
}

/// In-bounds cardinal neighbors in north, south, east, west order.
pub(super) fn neighbors(pos: Pos, width: u32, height: u32) -> Vec<Pos> {
    if pos.x >= width || pos.y >= height {
        return vec![];
    }
    [(0, -1), (0, 1), (1, 0), (-1, 0)]
        .into_iter()
        .filter_map(|(dx, dy)| {
            let x = i64::from(pos.x) + dx;
            let y = i64::from(pos.y) + dy;
            (x >= 0 && y >= 0 && x < i64::from(width) && y < i64::from(height)).then_some(Pos {
                x: x as u32,
                y: y as u32,
            })
        })
        .collect()
}
