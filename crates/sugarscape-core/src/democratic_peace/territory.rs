//! Bounded cardinal topology and source distance choices.
use super::world::Engine;
use super::*;
use std::collections::{BTreeSet, VecDeque};
impl Engine {
    pub(crate) fn adjacent(&self, id: usize) -> Vec<usize> {
        let w = self.width as usize;
        let n = self.cells.len();
        let mut v = Vec::with_capacity(4);
        if !id.is_multiple_of(w) {
            v.push(id - 1);
        }
        if id % w + 1 < w {
            v.push(id + 1);
        }
        if id >= w {
            v.push(id - w);
        }
        if id + w < n {
            v.push(id + w);
        }
        v.sort_unstable();
        v
    }
    pub(crate) fn neighbors(&self, id: StateId) -> Vec<StateId> {
        self.fronts
            .keys()
            .filter_map(|k| {
                if k[0] == id {
                    Some(k[1])
                } else if k[1] == id {
                    Some(k[0])
                } else {
                    None
                }
            })
            .collect()
    }
    pub(crate) fn member_cells(&self, id: StateId) -> Vec<usize> {
        self.cells
            .iter()
            .filter(|c| c.owner == id)
            .map(|c| c.id)
            .collect()
    }
    pub(crate) fn reached(&self, id: StateId, excluded: Option<usize>) -> BTreeSet<usize> {
        let mut reached = BTreeSet::new();
        let mut todo = vec![id.capital_cell];
        while let Some(cell) = todo.pop() {
            if Some(cell) == excluded || self.cells[cell].owner != id || !reached.insert(cell) {
                continue;
            }
            for next in self.adjacent(cell) {
                if Some(next) != excluded && self.cells[next].owner == id {
                    todo.push(next);
                }
            }
        }
        reached
    }
    pub(crate) fn rebuild(&mut self) -> Result<(), String> {
        for s in self.states.values_mut() {
            s.members.clear();
        }
        for c in &self.cells {
            self.states
                .get_mut(&c.owner)
                .ok_or("cell owner has no state")?
                .members
                .push(c.id);
        }
        let mut keys = BTreeSet::new();
        for c in &self.cells {
            for next in self.adjacent(c.id) {
                let other = self.cells[next].owner;
                if other != c.owner {
                    keys.insert(key(c.owner, other));
                }
            }
        }
        let mut old = std::mem::take(&mut self.fronts);
        self.fronts = keys
            .into_iter()
            .map(|k| (k, old.remove(&k).unwrap_or_else(|| Front::new(k))))
            .collect();
        Ok(())
    }
    pub(crate) fn distance(
        &self,
        c: &DemocraticPeaceConfig,
        id: StateId,
        cell: usize,
    ) -> Result<f64, String> {
        let w = self.width as usize;
        let dx = (id.capital_cell % w).abs_diff(cell % w);
        let dy = (id.capital_cell / w).abs_diff(cell / w);
        match c.distance_metric {
            DistanceMetric::Euclidean => Ok(libm::sqrt((dx * dx + dy * dy) as f64)),
            DistanceMetric::Manhattan => Ok((dx + dy) as f64),
            DistanceMetric::TerritorialPath => {
                let mut todo = VecDeque::from([(id.capital_cell, 0)]);
                let mut seen = BTreeSet::new();
                while let Some((p, d)) = todo.pop_front() {
                    if !seen.insert(p) {
                        continue;
                    }
                    if p == cell {
                        return Ok(d as f64);
                    }
                    for q in self.adjacent(p) {
                        if self.cells[q].owner == id {
                            todo.push_back((q, d + 1));
                        }
                    }
                }
                Err("unreachable territorial distance".into())
            }
        }
    }
    pub(crate) fn path(
        &mut self,
        attacker: StateId,
        target: StateId,
    ) -> Result<[usize; 2], String> {
        let targets: Vec<_> = self
            .cells
            .iter()
            .filter(|c| {
                c.owner == target
                    && self
                        .adjacent(c.id)
                        .iter()
                        .any(|&q| self.cells[q].owner == attacker)
            })
            .map(|c| c.id)
            .collect();
        let victim =
            super::world::pick(&targets, &mut self.rng).ok_or("no bordering target cell")?;
        let agents: Vec<_> = self
            .adjacent(victim)
            .into_iter()
            .filter(|&q| self.cells[q].owner == attacker)
            .collect();
        let agent = super::world::pick(&agents, &mut self.rng).ok_or("no bordering agent cell")?;
        Ok(if attacker < target {
            [agent, victim]
        } else {
            [victim, agent]
        })
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        for (id, s) in &self.states {
            if s.members.is_empty()
                || self.cells[id.capital_cell].owner != *id
                || s.regime != self.cells[id.capital_cell].latent_regime
            {
                return Err("invalid capital/member/regime invariant".into());
            }
            if !s.resources.is_finite() || s.resources < 0. {
                return Err("invalid sovereign resources".into());
            }
            if self.reached(*id, None).len() != s.members.len() {
                return Err("disconnected sovereign territory".into());
            }
        }
        for f in self.fronts.values() {
            if f.commitments
                .iter()
                .chain(f.old_commitments.iter())
                .any(|r| !r.is_finite() || *r < 0.)
            {
                return Err("invalid front commitment".into());
            }
        }
        let mut membership = BTreeSet::new();
        for a in &self.alliances {
            if a.members.len() < 2
                || !self.states.contains_key(&a.threat_id)
                || !a.pooled_resources.is_finite()
            {
                return Err("invalid alliance".into());
            }
            for m in &a.members {
                if !membership.insert(*m) || !self.fronts.contains_key(&key(*m, a.threat_id)) {
                    return Err("invalid alliance member".into());
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn key(a: StateId, b: StateId) -> [StateId; 2] {
    if a < b {
        [a, b]
    } else {
        [b, a]
    }
}
pub(crate) fn side(k: [StateId; 2], id: StateId) -> usize {
    usize::from(k[1] == id)
}
