use super::{Checked, Pos, Setup};
use crate::config::FieldError;
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct TerrainInventory {
    pub initial_open: u32,
    pub open: u32,
    pub excavated: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Terrain {
    width: u32,
    height: u32,
    initial_open: Vec<bool>,
    open: Vec<bool>,
    diggable: Vec<bool>,
    excavated: u32,
    capacity: u32,
}
impl Terrain {
    pub(super) fn new(setup: &Setup) -> Checked<Self> {
        setup.validate()?;
        let mut open = vec![false; (setup.width * setup.height) as usize];
        let mut diggable = open.clone();
        for p in &setup.open {
            open[(p.y * setup.width + p.x) as usize] = true;
        }
        for p in &setup.diggable {
            diggable[(p.y * setup.width + p.x) as usize] = true;
        }
        let capacity = open
            .iter()
            .zip(&diggable)
            .filter(|(o, d)| !**o && **d)
            .count() as u32;
        Ok(Self {
            width: setup.width,
            height: setup.height,
            initial_open: open.clone(),
            open,
            diggable,
            excavated: 0,
            capacity,
        })
    }
    fn index(&self, p: Pos) -> Checked<usize> {
        if p.x >= self.width || p.y >= self.height {
            return Err(vec![FieldError::new(
                "terrain.pos",
                "must be inside the grid",
            )]);
        }
        Ok((p.y * self.width + p.x) as usize)
    }
    pub(super) fn is_open(&self, p: Pos) -> Checked<bool> {
        Ok(self.open[self.index(p)?])
    }
    pub(super) fn is_diggable(&self, p: Pos) -> Checked<bool> {
        let i = self.index(p)?;
        Ok(!self.open[i] && self.diggable[i])
    }
    pub(super) fn was_excavated(&self, p: Pos) -> Checked<bool> {
        let i = self.index(p)?;
        Ok(self.open[i] && !self.initial_open[i])
    }
    pub(super) fn dig(&mut self, p: Pos) -> Checked<()> {
        let i = self.index(p)?;
        if self.open[i] || !self.diggable[i] {
            return Err(vec![FieldError::new(
                "terrain.dig",
                "requires a solid masked cell",
            )]);
        }
        let next = self
            .excavated
            .checked_add(1)
            .ok_or_else(|| vec![FieldError::new("terrain.excavated", "overflow")])?;
        self.open[i] = true;
        self.excavated = next;
        Ok(())
    }
    pub(super) fn counts(&self) -> TerrainInventory {
        TerrainInventory {
            initial_open: self.initial_open.iter().filter(|&&o| o).count() as u32,
            open: self.open.iter().filter(|&&o| o).count() as u32,
            excavated: self.excavated,
        }
    }
    pub(super) fn open_positions(&self) -> Vec<Pos> {
        let mut cells: Vec<_> = self
            .open
            .iter()
            .enumerate()
            .filter_map(|(i, &o)| {
                o.then_some(Pos {
                    x: i as u32 % self.width,
                    y: i as u32 / self.width,
                })
            })
            .collect();
        cells.sort();
        cells
    }
    pub(super) fn capacity(&self) -> u32 {
        self.capacity
    }
    pub(super) fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub(super) fn check(&self) -> Checked<()> {
        super::setup::dimensions(self.width, self.height)?;
        let len = (self.width * self.height) as usize;
        if self.open.len() != len || self.initial_open.len() != len || self.diggable.len() != len {
            return Err(vec![FieldError::new(
                "terrain",
                "array dimensions disagree",
            )]);
        }
        let mut excavated = 0;
        let mut capacity = 0;
        for i in 0..len {
            if self.initial_open[i] && !self.open[i]
                || (!self.initial_open[i] && self.open[i] && !self.diggable[i])
            {
                return Err(vec![FieldError::new(
                    "terrain",
                    "illegal topology revision",
                )]);
            }
            excavated += u32::from(!self.initial_open[i] && self.open[i]);
            capacity += u32::from(!self.initial_open[i] && self.diggable[i]);
        }
        if excavated != self.excavated || capacity != self.capacity {
            return Err(vec![FieldError::new(
                "terrain",
                "inventory disagrees with indexed cells",
            )]);
        }
        Ok(())
    }
}
