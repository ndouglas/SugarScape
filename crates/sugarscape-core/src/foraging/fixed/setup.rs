use crate::config::FieldError;
use crate::foraging::CpfaParameters;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Pos {
    pub x: u32,
    pub y: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Resource {
    pub id: u64,
    pub pos: Pos,
}
/// Explicit fixed geometry, population and shared CPFA parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Setup {
    pub width: u32,
    pub height: u32,
    pub nest: Pos,
    pub agents: u32,
    pub resources: Vec<Resource>,
    pub parameters: CpfaParameters,
}
impl Setup {
    /// Aggregate errors on original inputs without allocating a dense grid.
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        for (field, value) in [("width", self.width), ("height", self.height)] {
            if !(3..=125).contains(&value) {
                errors.push(FieldError::new(field, "must be in [3,125]"));
            }
        }
        if !self.contains(self.nest) {
            errors.push(FieldError::new("nest", "must be inside the grid"));
        }
        if !(1..=256).contains(&self.agents) {
            errors.push(FieldError::new("agents", "must be in [1,256]"));
        }
        if self.resources.len() > 256 {
            errors.push(FieldError::new(
                "resources",
                "must contain at most 256 resources",
            ));
        }
        let mut ids = BTreeSet::new();
        let mut cells = BTreeSet::new();
        for (index, resource) in self.resources.iter().enumerate() {
            if !ids.insert(resource.id) {
                errors.push(FieldError::new(
                    format!("resources[{index}].id"),
                    "duplicate resource identity",
                ));
            }
            if !cells.insert(resource.pos) {
                errors.push(FieldError::new(
                    format!("resources[{index}].pos"),
                    "duplicate resource cell",
                ));
            }
            if !self.contains(resource.pos) {
                errors.push(FieldError::new(
                    format!("resources[{index}].pos"),
                    "must be inside the grid",
                ));
            }
            if resource.pos == self.nest {
                errors.push(FieldError::new(
                    format!("resources[{index}].pos"),
                    "must differ from the nest",
                ));
            }
        }
        if let Err(parameter_errors) = self.parameters.validate() {
            errors.extend(
                parameter_errors
                    .into_iter()
                    .map(|e| FieldError::new(format!("parameters.{}", e.field), e.message)),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    pub(super) fn contains(&self, pos: Pos) -> bool {
        pos.x < self.width && pos.y < self.height
    }
    pub(super) fn site(&self, pos: Pos) -> u64 {
        u64::from(pos.y) * u64::from(self.width) + u64::from(pos.x)
    }
    pub(super) fn position(&self, site: u64) -> Result<Pos, Vec<FieldError>> {
        if !(3..=125).contains(&self.width) || !(3..=125).contains(&self.height) {
            return Err(vec![FieldError::new(
                "site",
                "requires valid grid dimensions",
            )]);
        }
        if site >= u64::from(self.width) * u64::from(self.height) {
            return Err(vec![FieldError::new(
                "site",
                "must encode a cell inside the grid",
            )]);
        }
        Ok(Pos {
            x: (site % u64::from(self.width)) as u32,
            y: (site / u64::from(self.width)) as u32,
        })
    }
}
