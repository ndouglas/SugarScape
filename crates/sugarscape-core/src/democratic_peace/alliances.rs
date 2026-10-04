//! Threat-driven defensive alignment and explicit one-pass obligations.
use super::*;
use super::{
    territory::{key, side},
    types::bump,
    world::Engine,
};
use std::collections::{BTreeMap, BTreeSet};
impl Engine {
    fn threatening(
        &self,
        c: &DemocraticPeaceConfig,
        actor: StateId,
        neighbor: StateId,
    ) -> Result<Option<f64>, String> {
        if self.states[&actor].regime == Regime::Democratic
            && self.states[&neighbor].regime == Regime::Democratic
        {
            return Ok(None);
        }
        let k = key(actor, neighbor);
        let i = side(k, actor);
        let f = &self.fronts[&k];
        let b =
            super::resources::threat_ratio(f.commitments[1 - i], f.commitments[i], c.zero_ratio)?;
        Ok((b > c.min_threat).then_some(b))
    }
    pub(crate) fn align(&mut self, c: &DemocraticPeaceConfig) -> Result<(), String> {
        self.pariahs.clear();
        if c.mechanism == Mechanism::Tagging {
            self.alliances.clear();
            return Ok(());
        }
        let old = std::mem::take(&mut self.alliances);
        let mut groups: BTreeMap<StateId, (u64, u64, Vec<StateId>)> = BTreeMap::new();
        let mut assigned = BTreeSet::new();
        if c.alliance_maintenance == AllianceMaintenance::PersistWhileThreatened {
            for a in old {
                if !self.states.contains_key(&a.threat_id) {
                    continue;
                }
                let mut members = Vec::new();
                for m in a.members {
                    if self.states.contains_key(&m)
                        && self.fronts.contains_key(&key(m, a.threat_id))
                        && self.threatening(c, m, a.threat_id)?.is_some()
                    {
                        members.push(m);
                    }
                }
                if members.len() >= 2 {
                    assigned.extend(members.iter().copied());
                    groups.insert(a.threat_id, (a.creation_period, a.serial, members));
                }
            }
        }
        let actors: Vec<_> = self.states.keys().copied().collect();
        let mut proposals: BTreeMap<StateId, Vec<StateId>> = BTreeMap::new();
        for actor in actors {
            if assigned.contains(&actor) {
                continue;
            }
            let mut best = None;
            let mut ties = Vec::new();
            for neighbor in self.neighbors(actor) {
                if let Some(b) = self.threatening(c, actor, neighbor)? {
                    if best.is_none_or(|v| b > v) {
                        best = Some(b);
                        ties.clear();
                        ties.push(neighbor);
                    } else if best == Some(b) {
                        ties.push(neighbor);
                    }
                }
            }
            if !ties.is_empty() {
                let target = if c.threat_ties == ThreatTies::RandomTie {
                    super::world::pick(&ties, &mut self.rng).unwrap()
                } else {
                    ties[0]
                };
                proposals.entry(target).or_default().push(actor);
            }
        }
        for (threat, mut members) in proposals {
            if let Some((_, _, existing)) = groups.get_mut(&threat) {
                existing.append(&mut members);
                existing.sort();
            } else if members.len() >= 2 {
                let serial = self.next_alliance;
                bump(&mut self.next_alliance)?;
                groups.insert(threat, (self.period, serial, members));
            }
        }
        for (threat, (creation_period, serial, members)) in groups {
            let pooled_resources = members
                .iter()
                .map(|m| {
                    let k = key(*m, threat);
                    self.fronts[&k].commitments[side(k, *m)]
                })
                .sum();
            if !f64::is_finite(pooled_resources) {
                return Err("nonfinite alliance pool".into());
            }
            self.alliances.push(Alliance {
                threat_id: threat,
                creation_period,
                serial,
                members,
                pooled_resources,
            });
        }
        Ok(())
    }
    pub(crate) fn obligations(&mut self, c: &DemocraticPeaceConfig, current: bool) {
        let actions: BTreeMap<_, _> = self
            .fronts
            .iter()
            .map(|(&k, f)| (k, if current { f.actions } else { f.previous }))
            .collect();
        self.pariahs.clear();
        if c.mechanism == Mechanism::CollectiveSecurity {
            for (k, a) in &actions {
                if a[0] && a[1] {
                    let regimes = [self.states[&k[0]].regime, self.states[&k[1]].regime];
                    if regimes[0] != regimes[1] {
                        let d = usize::from(regimes[1] == Regime::Democratic);
                        self.pariahs.entry(k[1 - d]).or_default().push(k[d]);
                    }
                }
            }
            for sources in self.pariahs.values_mut() {
                sources.sort();
                sources.dedup();
            }
        }
        let mut obligations: Vec<([StateId; 2], usize, String)> = Vec::new();
        for alliance in &self.alliances {
            let attacked = alliance.members.iter().any(|m| {
                let k = key(*m, alliance.threat_id);
                actions
                    .get(&k)
                    .is_some_and(|a| a[side(k, alliance.threat_id)])
            });
            if attacked {
                for m in &alliance.members {
                    let k = key(*m, alliance.threat_id);
                    if self.fronts.contains_key(&k) {
                        obligations.push((k, side(k, *m), "attacked_ally".into()));
                    }
                }
            }
        }
        for (&pariah, sources) in &self.pariahs {
            for (&actor, s) in &self.states {
                if s.regime != Regime::Democratic {
                    continue;
                }
                let permitted = c.security_scope == SecurityScope::AllDemocracies
                    || self.alliances.iter().any(|a| {
                        a.members.contains(&actor) && sources.iter().any(|d| a.members.contains(d))
                    });
                let k = key(actor, pariah);
                if permitted && self.fronts.contains_key(&k) {
                    obligations.push((k, side(k, actor), "pariah".into()));
                }
            }
        }
        for (k, i, reason) in obligations {
            let f = self.fronts.get_mut(&k).unwrap();
            f.actions[i] = true;
            f.obligations[i].push(reason);
        }
    }
}
