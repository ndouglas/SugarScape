use super::Checked;
use crate::{config::FieldError, foraging::CpfaParameters};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}
impl Pos {
    /// In-bounds cardinal neighbors in N,S,E,W order.
    pub(super) fn neighbors(self, width: u32, height: u32) -> Vec<Pos> {
        if self.x >= width || self.y >= height {
            return vec![];
        }
        [(0, -1), (0, 1), (1, 0), (-1, 0)]
            .into_iter()
            .filter_map(|(dx, dy)| {
                let x = i64::from(self.x) + dx;
                let y = i64::from(self.y) + dy;
                (x >= 0 && y >= 0 && x < i64::from(width) && y < i64::from(height)).then_some(Pos {
                    x: x as u32,
                    y: y as u32,
                })
            })
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Resource {
    pub id: u64,
    pub pos: Pos,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Parameters {
    pub p_search: f64,
    pub p_return: f64,
    pub lambda_fidelity: f64,
    pub lambda_publish: f64,
    pub lambda_waypoint: f64,
}
impl Parameters {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        self.information().validate()
    }
    pub(super) fn information(&self) -> CpfaParameters {
        CpfaParameters {
            p_search: self.p_search,
            p_return: self.p_return,
            omega: 0.0,
            lambda_informed: 0.0,
            lambda_fidelity: self.lambda_fidelity,
            lambda_publish: self.lambda_publish,
            lambda_waypoint: self.lambda_waypoint,
        }
    }
}
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Setup {
    pub width: u32,
    pub height: u32,
    pub open: Vec<Pos>,
    pub nest: Vec<Pos>,
    pub workers: Vec<Pos>,
    pub resources: Vec<Resource>,
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
                for next in p.neighbors(self.width, self.height) {
                    if nest.contains(&next) && reached.insert(next) {
                        pending.push(next);
                    }
                }
            }
            if reached.len() != nest.len() {
                errors.push(FieldError::new("nest", "must be four-neighbor connected"));
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
        if self.resources.len() > 256 {
            errors.push(FieldError::new(
                "resources",
                "must contain at most 256 resources",
            ));
        }
        let mut ids = BTreeSet::new();
        let mut cells = BTreeSet::new();
        for (i, r) in self.resources.iter().enumerate() {
            if !ids.insert(r.id) {
                errors.push(FieldError::new(
                    format!("resources[{i}].id"),
                    "duplicate resource identity",
                ));
            }
            if !cells.insert(r.pos) {
                errors.push(FieldError::new(
                    format!("resources[{i}].pos"),
                    "duplicate resource cell",
                ));
            }
            if !self.contains(r.pos) || !open.contains(&r.pos) || nest.contains(&r.pos) {
                errors.push(FieldError::new(
                    format!("resources[{i}].pos"),
                    "must be an in-bounds open cell outside the nest",
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
        self.resources.sort_by_key(|r| r.id);
        Ok(self)
    }
    pub(super) fn contains(&self, p: Pos) -> bool {
        p.x < self.width && p.y < self.height
    }
    // Tasks 3–4 encode/decode information sites with these checked conversions.
    #[allow(dead_code)]
    pub(super) fn site(&self, p: Pos) -> Checked<u64> {
        dimensions(self.width, self.height)?;
        if !self.contains(p) {
            return Err(vec![FieldError::new("site", "must be inside the grid")]);
        }
        Ok(u64::from(p.y) * u64::from(self.width) + u64::from(p.x))
    }
    #[allow(dead_code)]
    pub(super) fn position(&self, site: u64) -> Checked<Pos> {
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
