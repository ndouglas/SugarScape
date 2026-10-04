//! Snapshot commitments and decisions; no same-phase voluntary-action leakage.
use super::*;
use super::{
    territory::{key, side},
    types::bump,
    world::Engine,
};
impl Engine {
    pub(crate) fn allocate(&mut self, c: &DemocraticPeaceConfig) -> Result<(), String> {
        let mut next = Vec::new();
        for (&id, s) in &self.states {
            let neighbors: Vec<_> = self
                .neighbors(id)
                .into_iter()
                .filter(|n| {
                    s.regime != Regime::Democratic || self.states[n].regime != Regime::Democratic
                })
                .collect();
            let n = neighbors.len();
            let enemy: f64 = neighbors
                .iter()
                .filter(|&&neighbor| {
                    c.enemy_total == EnemyTotal::AllFronts
                        || self.fronts[&key(id, neighbor)].previous.iter().any(|a| *a)
                })
                .map(|&neighbor| {
                    let k = key(id, neighbor);
                    self.fronts[&k].old_commitments[1 - side(k, id)]
                })
                .sum();
            let first = neighbors
                .iter()
                .filter(|&&neighbor| !self.fronts[&key(id, neighbor)].previous.iter().any(|a| *a))
                .map(|&neighbor| {
                    let k = key(id, neighbor);
                    self.fronts[&k].old_commitments[1 - side(k, id)]
                })
                .next()
                .unwrap_or(0.);
            for neighbor in self.neighbors(id) {
                let k = key(id, neighbor);
                let i = side(k, id);
                if !neighbors.contains(&neighbor) {
                    next.push((k, i, 0., Allocation::default()));
                    continue;
                }
                let f = &self.fronts[&k];
                let old_opposing = f.old_commitments[1 - i];
                let active = f.previous.iter().any(|a| *a);
                let term = if c.inactive_commitment == InactiveCommitment::PerFrontOpponent {
                    old_opposing
                } else {
                    first
                };
                let v = super::resources::commitment(
                    s.resources,
                    c.mobile_share,
                    n,
                    old_opposing,
                    enemy,
                    active,
                    term,
                )?;
                next.push((
                    k,
                    i,
                    v,
                    Allocation {
                        fixed: (1. - c.mobile_share) * s.resources / n as f64,
                        mobile_pool: c.mobile_share * s.resources,
                        eligible_fronts: n,
                        old_opposing,
                        enemy_total: enemy,
                        inactive_term: term,
                        active,
                    },
                ));
            }
        }
        for (k, i, v, a) in next {
            let f = self.fronts.get_mut(&k).unwrap();
            f.commitments[i] = v;
            f.allocations[i] = a;
        }
        Ok(())
    }
    pub(crate) fn decide(&mut self, c: &DemocraticPeaceConfig) -> Result<(), String> {
        for f in self.fronts.values_mut() {
            f.actions = [f.previous.iter().any(|a| *a); 2];
            f.initiations = [false; 2];
            f.obligations = Default::default();
            f.attack_probabilities = [None; 2];
            f.attack_ratios = Default::default();
            f.victory_probabilities = [None; 2];
            f.victory_ratios = Default::default();
            f.claims = [false; 2];
        }
        if c.obligation_observation == ObligationObservation::PriorActions {
            self.obligations(c, false);
        }
        let actors: Vec<_> = self.states.keys().copied().collect();
        let mut proposals = Vec::new();
        for id in actors {
            let neighbors = self.neighbors(id);
            if neighbors.iter().any(|&n| {
                let k = key(id, n);
                self.fronts[&k].actions[side(k, id)]
            }) {
                continue;
            }
            let eligible: Vec<_> = neighbors
                .into_iter()
                .filter(|n| {
                    self.states[&id].regime != Regime::Democratic
                        || self.states[n].regime != Regime::Democratic
                })
                .collect();
            let Some(target) = super::world::pick(&eligible, &mut self.rng) else {
                continue;
            };
            let k = key(id, target);
            let i = side(k, id);
            let n = self.fronts[&k].commitments[i];
            let d = self
                .alliances
                .iter()
                .find(|a| a.threat_id == id && a.members.contains(&target))
                .map(|a| a.pooled_resources)
                .unwrap_or(self.fronts[&k].commitments[1 - i]);
            let p = super::resources::probability(
                n,
                d,
                c.superiority_threshold,
                c.superiority_exponent,
                c.probability_direction,
                c.zero_ratio,
            )?;
            let f = self.fronts.get_mut(&k).unwrap();
            f.attack_probabilities[i] = Some(p);
            f.attack_ratios[i] = Some(Ratio::new(n, d));
            if super::world::chance(p, &mut self.rng) {
                let path = self.path(id, target)?;
                proposals.push((k, i, id, path));
            }
        }
        for (k, i, id, path) in proposals {
            let f = self.fronts.get_mut(&k).unwrap();
            let already_initiated = f.initiations.iter().any(|v| *v);
            f.actions[i] = true;
            f.initiations[i] = true;
            if f.path.is_none() {
                f.path = Some(path);
                f.path_proposer = Some(id);
            }
            if !already_initiated {
                bump(&mut self.counters.initiated_fronts)?;
            }
        }
        if c.obligation_observation == ObligationObservation::CurrentPlansOnce {
            self.obligations(c, true);
        }
        Ok(())
    }
}
