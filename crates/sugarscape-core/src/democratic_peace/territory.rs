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
        if self.width == 0 || !self.cells.len().is_multiple_of(self.width as usize) {
            return Err(format!(
                "grid width {} does not divide {} cells",
                self.width,
                self.cells.len()
            ));
        }
        for (index, cell) in self.cells.iter().enumerate() {
            if cell.id != index {
                return Err(format!(
                    "cell at vector index {index} has mismatched cell id {}",
                    cell.id
                ));
            }
            if !self.states.contains_key(&cell.owner) {
                return Err(format!(
                    "cell {index} is owned by missing state {:?}",
                    cell.owner
                ));
            }
        }
        for (id, state) in &self.states {
            if state.id != *id {
                return Err(format!(
                    "state map key {:?} disagrees with stored state id {:?}",
                    id, state.id
                ));
            }
            if id.capital_cell >= self.cells.len() {
                return Err(format!(
                    "state {:?} capital cell {} is outside {} cells",
                    id,
                    id.capital_cell,
                    self.cells.len()
                ));
            }
            if self.cells[id.capital_cell].owner != *id {
                return Err(format!(
                    "state {:?} capital cell {} is owned by {:?}",
                    id, id.capital_cell, self.cells[id.capital_cell].owner
                ));
            }
            if self.cells[id.capital_cell].next_generation < id.sovereignty_generation {
                return Err(format!(
                    "state {:?} generation exceeds cell {} generation counter {}",
                    id, id.capital_cell, self.cells[id.capital_cell].next_generation
                ));
            }
            if state.regime != self.cells[id.capital_cell].latent_regime {
                return Err(format!(
                    "state {:?} regime disagrees with capital cell {} latent regime",
                    id, id.capital_cell
                ));
            }
            if !state.resources.is_finite() || state.resources < 0. {
                return Err(format!("state {:?} has invalid resources", id));
            }
            let owned: Vec<_> = self
                .cells
                .iter()
                .enumerate()
                .filter_map(|(index, cell)| (cell.owner == *id).then_some(index))
                .collect();
            if owned.is_empty() {
                return Err(format!("state {:?} owns no cells", id));
            }
            if state.members != owned {
                return Err(format!(
                    "state {:?} member list {:?} does not equal owned cells {:?}",
                    id, state.members, owned
                ));
            }
            let reached = self.reached(*id, None);
            if reached.iter().copied().collect::<Vec<_>>() != owned {
                return Err(format!(
                    "state {:?} owned cells {:?} are not connected from capital {}",
                    id, owned, id.capital_cell
                ));
            }
        }

        let mut expected_fronts = BTreeSet::new();
        for cell in 0..self.cells.len() {
            let owner = self.cells[cell].owner;
            for neighbor in self.adjacent(cell) {
                let other = self.cells[neighbor].owner;
                if owner != other {
                    expected_fronts.insert(key(owner, other));
                }
            }
        }
        for (front_key, front) in &self.fronts {
            if front.states != *front_key || key(front.states[0], front.states[1]) != *front_key {
                return Err(format!(
                    "front map key {:?} disagrees with front endpoint ids {:?}",
                    front_key, front.states
                ));
            }
            for state in front.states {
                if !self.states.contains_key(&state) {
                    return Err(format!(
                        "front {:?} references missing endpoint state {:?}",
                        front_key, state
                    ));
                }
            }
            if front
                .commitments
                .iter()
                .chain(front.old_commitments.iter())
                .any(|r| !r.is_finite() || *r < 0.)
            {
                return Err(format!("front {:?} has invalid commitment", front_key));
            }
        }
        let actual_fronts: BTreeSet<_> = self.fronts.keys().copied().collect();
        if let Some(missing) = expected_fronts.difference(&actual_fronts).next() {
            return Err(format!(
                "front topology is missing territorial front {:?}",
                missing
            ));
        }
        if let Some(extra) = actual_fronts.difference(&expected_fronts).next() {
            return Err(format!(
                "front topology has nonterritorial front {:?}",
                extra
            ));
        }

        let mut membership = BTreeSet::new();
        for alliance in &self.alliances {
            if alliance.members.len() < 2
                || !self.states.contains_key(&alliance.threat_id)
                || !alliance.pooled_resources.is_finite()
                || alliance.pooled_resources < 0.
            {
                return Err(format!(
                    "alliance against {:?} has invalid identity or pool",
                    alliance.threat_id
                ));
            }
            if alliance.members.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(format!(
                    "alliance against {:?} has unsorted or duplicate members {:?}",
                    alliance.threat_id, alliance.members
                ));
            }
            let mut expected_pool = 0.;
            for member in &alliance.members {
                if *member == alliance.threat_id
                    || !self.states.contains_key(member)
                    || !self.fronts.contains_key(&key(*member, alliance.threat_id))
                {
                    return Err(format!(
                        "alliance against {:?} has invalid member {:?}",
                        alliance.threat_id, member
                    ));
                }
                if !membership.insert(*member) {
                    return Err(format!("state {:?} belongs to multiple alliances", member));
                }
                let front_key = key(*member, alliance.threat_id);
                let member_side = side(front_key, *member);
                expected_pool += self.fronts[&front_key].commitments[member_side];
            }
            if !expected_pool.is_finite() || alliance.pooled_resources != expected_pool {
                return Err(format!(
                    "alliance against {:?} pool {} does not equal surviving member commitments {}",
                    alliance.threat_id, alliance.pooled_resources, expected_pool
                ));
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
