//! Prospective complete claim footprints and single-cell sovereign release.
use super::*;
use super::{
    types::{bump, Claim},
    world::Engine,
};
use std::collections::BTreeSet;
impl Engine {
    fn release(&mut self, cell: usize) -> Result<StateId, String> {
        self.cells[cell].next_generation = self.cells[cell]
            .next_generation
            .checked_add(1)
            .ok_or("sovereignty generation overflow")?;
        let id = StateId {
            capital_cell: cell,
            sovereignty_generation: self.cells[cell].next_generation,
        };
        self.cells[cell].owner = id;
        self.states.insert(
            id,
            State {
                id,
                regime: self.cells[cell].latent_regime,
                resources: 10.,
                members: vec![cell],
            },
        );
        bump(&mut self.counters.released_states)?;
        Ok(id)
    }
    pub(crate) fn apply_claims(
        &mut self,
        c: &DemocraticPeaceConfig,
        claims: Vec<Claim>,
    ) -> Result<(), String> {
        if claims.is_empty() {
            return Ok(());
        }
        let mut actors: Vec<_> = self.states.keys().copied().collect();
        super::world::shuffle(&mut actors, &mut self.rng);
        let mut locked_cells = BTreeSet::new();
        let mut locked_states = BTreeSet::new();
        for actor in actors {
            let mut own: Vec<_> = claims
                .iter()
                .filter(|q| q.states[q.side] == actor)
                .collect();
            own.sort_by_key(|q| q.states);
            for q in own {
                let winner = q.states[q.side];
                let loser = q.states[1 - q.side];
                let agent = q.path[q.side];
                let target = q.path[1 - q.side];
                if !self.states.contains_key(&winner)
                    || !self.states.contains_key(&loser)
                    || self.cells[agent].owner != winner
                    || self.cells[target].owner != loser
                    || !self.adjacent(agent).contains(&target)
                {
                    bump(&mut self.counters.stale_claims)?;
                    continue;
                }
                let members = self.member_cells(loser);
                let capital = target == loser.capital_cell;
                let releases: Vec<usize> = if capital && members.len() > 1 {
                    members
                        .iter()
                        .copied()
                        .filter(|&cell| {
                            c.capital_capture == CapitalCapture::CollapseOnly || cell != target
                        })
                        .collect()
                } else if !capital {
                    let reached = self.reached(loser, Some(target));
                    members
                        .iter()
                        .copied()
                        .filter(|&cell| cell != target && !reached.contains(&cell))
                        .collect()
                } else {
                    Vec::new()
                };
                let footprint: BTreeSet<_> = std::iter::once(target)
                    .chain(releases.iter().copied())
                    .collect();
                let blocked = match c.claim_locking {
                    ClaimLocking::AffectedCells => {
                        locked_cells.contains(&agent)
                            || footprint.iter().any(|cell| locked_cells.contains(cell))
                    }
                    ClaimLocking::AffectedStates => {
                        locked_states.contains(&winner) || locked_states.contains(&loser)
                    }
                };
                if blocked {
                    bump(&mut self.counters.locked_claims)?;
                    continue;
                }
                let retired = capital;
                if retired {
                    self.states.remove(&loser);
                    bump(&mut self.counters.retired_states)?;
                }
                let absorb = !(capital
                    && members.len() > 1
                    && c.capital_capture == CapitalCapture::CollapseOnly);
                if absorb {
                    self.cells[target].owner = winner;
                    if c.latent_regime == LatentRegime::OverwriteOnConquest {
                        self.cells[target].latent_regime = self.states[&winner].regime;
                    }
                }
                let mut new_ids = Vec::new();
                for cell in releases {
                    new_ids.push(self.release(cell)?);
                }
                if c.claim_locking == ClaimLocking::AffectedCells {
                    locked_cells.extend(footprint.iter().copied());
                    locked_cells.insert(agent);
                } else {
                    locked_states.extend([winner, loser]);
                    locked_states.extend(new_ids);
                }
                bump(&mut self.counters.successful_claims)?;
                let event = Event {
                    period: self.period,
                    kind: if capital && members.len() > 1 {
                        "capital_collapse"
                    } else {
                        "conquest"
                    }
                    .into(),
                    states: vec![winner, loser],
                    cells: footprint.into_iter().collect(),
                };
                self.last_structural_event = Some(event.clone());
                self.record(c, event)?;
            }
        }
        self.rebuild()?;
        let alive: BTreeSet<_> = self.states.keys().copied().collect();
        let fronts = &self.fronts;
        for a in &mut self.alliances {
            a.members.retain(|m| {
                alive.contains(m) && fronts.contains_key(&super::territory::key(*m, a.threat_id))
            });
        }
        self.alliances
            .retain(|a| alive.contains(&a.threat_id) && a.members.len() >= 2);
        self.pariahs.retain(|p, sources| {
            sources.retain(|s| alive.contains(s));
            alive.contains(p) && !sources.is_empty()
        });
        Ok(())
    }
}
