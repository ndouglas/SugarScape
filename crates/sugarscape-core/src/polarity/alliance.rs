//! Directed trust and defensive coalitions keyed by a common prime threat.
use super::config::PolarityConfig;
use super::{
    config::*,
    decision::FrontKey,
    territory,
    world::{index, PolarityWorld},
};
use std::collections::BTreeSet;
pub fn trust(config: &PolarityConfig, old: f64, aggression: bool) -> f64 {
    let (rate, target) = if aggression {
        (config.negative_trust_rate, -1000.0)
    } else {
        (config.positive_trust_rate, 1000.0)
    };
    (1.0 - rate) * old + rate * target
}

impl PolarityWorld {
    pub(super) fn update_trust(&mut self) {
        if !self.config.alliances {
            return;
        }
        for (&(a, b), value) in &mut self.trust {
            let aggression = match self.config.threat_observation {
                ThreatObservation::NeighborAggression => self.aggression.contains(&b),
                ThreatObservation::DyadicAggression => self.dyadic_aggression.contains(&(b, a)),
            };
            *value = super::alliance::trust(&self.config, *value, aggression);
        }
    }
    pub(super) fn form_coalitions(&mut self) {
        self.coalitions.clear();
        self.threats.clear();
        if !self.config.alliances {
            return;
        }
        for a in territory::capitals(&self.cells) {
            let mut least = self.config.threat_threshold;
            let mut candidates = Vec::new();
            for b in territory::neighbors(&self.config, &self.cells, a) {
                let value = self.trust[&(a, b)];
                if value < least {
                    least = value;
                    candidates.clear();
                    candidates.push(b);
                } else if value == least && value < self.config.threat_threshold {
                    candidates.push(b);
                }
            }
            if !candidates.is_empty() {
                let b = if self.config.tie_break == TieBreak::Random && candidates.len() > 1 {
                    candidates[index(&mut self.rng, candidates.len())]
                } else {
                    candidates[0]
                };
                self.threats.insert(a, b);
                self.coalitions.entry(b).or_default().push(a);
            }
        }
        self.coalitions.retain(|_, members| members.len() >= 2);
    }
    pub(super) fn supported(&self, a: usize, victim: usize) -> [f64; 2] {
        let mut attack = self.cells[a].stock;
        let mut defend = self.cells[victim].stock;
        for (&threat, members) in &self.coalitions {
            if threat == victim && members.contains(&a) {
                attack = members.iter().map(|&i| self.cells[i].stock).sum();
            }
            if members.contains(&victim) {
                defend = members.iter().map(|&i| self.cells[i].stock).sum();
            }
        }
        [attack, defend]
    }
    pub(super) fn deterrence_commitments(&self, a: usize, victim: usize) -> [f64; 2] {
        let front_value = |actor: usize, opponent: usize| {
            let key = FrontKey::foreign(actor, opponent);
            self.fronts
                .get(&key)
                .map(|f| f.commitments[usize::from(key.b == actor)])
                .unwrap_or(0.0)
        };
        let mut result = [front_value(a, victim), front_value(victim, a)];
        for (&threat, members) in &self.coalitions {
            if threat == victim && members.contains(&a) {
                result[0] += members
                    .iter()
                    .filter(|&&i| i != a)
                    .map(|&i| {
                        if self.config.pra_alliance_support == PraAllianceSupport::Stocks {
                            self.cells[i].stock
                        } else {
                            front_value(i, victim)
                        }
                    })
                    .sum::<f64>();
            }
            if members.contains(&victim) {
                result[1] += members
                    .iter()
                    .filter(|&&i| i != victim)
                    .map(|&i| {
                        if self.config.pra_alliance_support == PraAllianceSupport::Stocks {
                            self.cells[i].stock
                        } else {
                            front_value(i, a)
                        }
                    })
                    .sum::<f64>();
            }
        }
        result
    }
    pub(super) fn coalition_threat(&self, capital: usize) -> Option<usize> {
        self.coalitions
            .iter()
            .find(|(_, m)| m.contains(&capital))
            .map(|(&t, _)| t)
    }
    pub(super) fn obligations(&mut self) {
        if !self.config.alliances {
            return;
        }
        let initiations: Vec<(usize, usize)> = self
            .fronts
            .values()
            .filter(|f| !f.key.domestic)
            .flat_map(|f| {
                [
                    (f.key.a, f.key.b, f.initiated[0]),
                    (f.key.b, f.key.a, f.initiated[1]),
                ]
            })
            .filter(|(_, _, yes)| *yes)
            .map(|(a, b, _)| (a, b))
            .collect();
        for (a, b) in &initiations {
            for members in self.coalitions.values_mut() {
                if members.contains(a) && members.contains(b) {
                    members.retain(|x| x != a);
                }
            }
        }
        self.coalitions.retain(|_, m| m.len() >= 2);
        let mut obligations = BTreeSet::new();
        for (a, b) in initiations {
            for members in self.coalitions.values() {
                if members.contains(&b) {
                    for &member in members {
                        if member != a {
                            let key = FrontKey::foreign(member, a);
                            let unresolved = self.config.obligation_timing
                                == ObligationTiming::NextPeriod
                                || !self.resolved_fronts.contains(&key);
                            if unresolved && self.fronts.contains_key(&key) {
                                obligations.insert((member, a));
                            }
                        }
                    }
                }
            }
        }
        if self.config.obligation_timing == ObligationTiming::NextPeriod {
            self.pending = obligations;
        } else {
            for (a, b) in obligations {
                let k = FrontKey::foreign(a, b);
                self.fronts.get_mut(&k).unwrap().actions[usize::from(k.b == a)] = true;
            }
        }
    }
    pub(super) fn note_aggression(&mut self) {
        for f in self.fronts.values().filter(|f| !f.key.domestic) {
            if f.initiated[0] {
                self.aggression.insert(f.key.a);
                self.dyadic_aggression.insert((f.key.a, f.key.b));
            }
            if f.initiated[1] {
                self.aggression.insert(f.key.b);
                self.dyadic_aggression.insert((f.key.b, f.key.a));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_and_positive_trust_learning_follow_source_recurrence() {
        let c = PolarityConfig::default();
        assert_eq!(trust(&c, 100.0, true), -450.0);
        assert_eq!(trust(&c, -450.0, false), -435.5);
    }
}
