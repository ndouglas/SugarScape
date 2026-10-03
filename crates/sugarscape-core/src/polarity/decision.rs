//! Foreign and domestic fronts have separate namespaces and stable paths.
use super::decision;
use super::{analysis::Episode, config::*};
use super::{
    analysis::Ledger,
    combat, resources, territory,
    world::{index, PolarityWorld},
};
use rand::Rng;
use serde::Serialize;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct FrontKey {
    pub domestic: bool,
    pub a: usize,
    pub b: usize,
}
impl FrontKey {
    pub fn foreign(a: usize, b: usize) -> Self {
        Self {
            domestic: false,
            a: a.min(b),
            b: a.max(b),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Front {
    pub key: FrontKey,
    pub previous: [bool; 2],
    pub actions: [bool; 2],
    pub old_commitments: [f64; 2],
    pub commitments: [f64; 2],
    pub path: Option<[usize; 2]>,
    pub initiated: [bool; 2],
    pub episode: Option<Episode>,
}
pub fn provisional(memory: ActionMemory, prior: [bool; 2], open: bool) -> [bool; 2] {
    if memory == ActionMemory::WarUntilVictory {
        [prior[1] || prior[0] || open, prior[0] || prior[1] || open]
    } else {
        [prior[1], prior[0]]
    }
}
pub fn may_initiate(
    gate: SchlieffenGate,
    own: &[bool],
    previous: &[[bool; 2]],
    open: bool,
) -> bool {
    match gate {
        SchlieffenGate::OwnCurrentDefections => !own.iter().any(|x| *x),
        SchlieffenGate::PreviousHostilities => !previous.iter().any(|p| p[0] || p[1]),
        SchlieffenGate::UnresolvedWar => !open,
    }
}

impl PolarityWorld {
    pub(super) fn valid_path(&self, key: FrontKey, p: [usize; 2]) -> bool {
        if key.domestic {
            return p == [key.a, key.b] && self.cells[key.b].capital == key.a;
        }
        self.cells[p[0]].capital == key.a
            && self.cells[p[1]].capital == key.b
            && territory::adjacent(&self.config, p[0]).contains(&p[1])
    }
    pub(super) fn sample_path(&mut self, key: FrontKey, side: usize) -> [usize; 2] {
        if key.domestic {
            return [key.a, key.b];
        }
        let (a, b) = if side == 0 {
            (key.a, key.b)
        } else {
            (key.b, key.a)
        };
        let candidates: Vec<usize> = territory::members(&self.cells, a)
            .into_iter()
            .filter(|&i| {
                territory::adjacent(&self.config, i)
                    .iter()
                    .any(|&j| self.cells[j].capital == b)
            })
            .collect();
        let i = candidates[index(&mut self.rng, candidates.len())];
        let targets: Vec<usize> = territory::adjacent(&self.config, i)
            .into_iter()
            .filter(|&j| self.cells[j].capital == b)
            .collect();
        let j = targets[index(&mut self.rng, targets.len())];
        if side == 0 {
            [i, j]
        } else {
            [j, i]
        }
    }
    pub(super) fn prepare(&mut self) {
        for f in self.fronts.values_mut() {
            f.actions =
                decision::provisional(self.config.action_memory, f.previous, f.episode.is_some());
            f.initiated = [false, false];
        }
        for &(a, b) in &self.pending {
            let k = FrontKey::foreign(a, b);
            if let Some(f) = self.fronts.get_mut(&k) {
                f.actions[usize::from(k.b == a)] = true;
            }
        }
        self.pending.clear();
        let stocks: Vec<f64> = self.cells.iter().map(|c| c.stock).collect();
        let allocated = self.allocate(&stocks);
        for (k, v) in allocated {
            self.fronts.get_mut(&k).unwrap().commitments = v;
        }
    }
    pub(super) fn decide_actor(&mut self, actor: usize, e: &mut Ledger) {
        // Provinces can initiate before their center evaluates its domestic guard.
        if self.config.provincial() {
            let domestic: Vec<FrontKey> = self
                .fronts
                .keys()
                .filter(|k| k.domestic && k.a == actor)
                .copied()
                .collect();
            for key in domestic {
                let f = &self.fronts[&key];
                if !f.actions[1] && f.episode.is_none() {
                    if resources::ratio_overflows(self.cells[key.b].stock, f.commitments[0]) {
                        self.finish("invalid",Some(format!("period {} domestic front {:?}: nonfinite revolt ratio; config {:?}",self.period,key,self.config)));
                        return;
                    }
                    let r = resources::ratio(self.cells[key.b].stock, f.commitments[0]);
                    let yes = if self.config.variant == Variant::Overextension {
                        self.rng.gen::<f64>()
                            < combat::probability(r, 2.0, self.config.stochastic_exponent)
                    } else {
                        r > 2.0
                    };
                    if yes {
                        let f = self.fronts.get_mut(&key).unwrap();
                        f.actions[1] = true;
                        f.initiated[1] = true;
                        e.revolts += 1;
                    }
                }
            }
        }
        if !self.cells[actor].predator {
            return;
        }
        let related: Vec<&Front> = self
            .fronts
            .values()
            .filter(|f| f.key.a == actor || (!f.key.domestic && f.key.b == actor))
            .collect();
        let own: Vec<bool> = related
            .iter()
            .map(|f| f.actions[usize::from(f.key.b == actor && !f.key.domestic)])
            .collect();
        let prior: Vec<[bool; 2]> = related.iter().map(|f| f.previous).collect();
        let open = related.iter().any(|f| f.episode.is_some());
        if !decision::may_initiate(self.config.schlieffen_gate, &own, &prior, open)
            || related
                .iter()
                .any(|f| f.key.domestic && (f.actions[0] || f.actions[1]))
        {
            return;
        }
        let candidates: Vec<(usize, f64)> = related
            .iter()
            .filter(|f| {
                !f.key.domestic
                    && !(self.config.update == Update::Sequential
                        && self.resolved_fronts.contains(&f.key))
            })
            .map(|f| {
                let side = usize::from(f.key.b == actor);
                let other = if side == 0 { f.key.b } else { f.key.a };
                let score = if self.config.allocation == Allocation::Equal {
                    self.cells[other].stock
                } else {
                    resources::ratio(f.commitments[side], f.commitments[1 - side])
                };
                (other, score)
            })
            .collect();
        if self.config.allocation == Allocation::Pra {
            for &(other, _) in &candidates {
                let key = FrontKey::foreign(actor, other);
                let f = &self.fronts[&key];
                let side = usize::from(key.b == actor);
                if resources::ratio_overflows(f.commitments[side], f.commitments[1 - side]) {
                    self.finish(
                        "invalid",
                        Some(format!(
                            "period {} front {:?}: nonfinite target ratio; config {:?}",
                            self.period, key, self.config
                        )),
                    );
                    return;
                }
            }
        }
        let maximize = self.config.allocation == Allocation::Pra
            && self.config.pra_attack_rule == PraAttackRule::Diagram;
        let mut best: Option<f64> = None;
        let mut tied = Vec::new();
        for (other, score) in candidates {
            let improves = best.is_none_or(|v| if maximize { score > v } else { score < v });
            if improves {
                best = Some(score);
                tied.clear();
                tied.push(other);
            } else if best == Some(score) {
                tied.push(other);
            }
        }
        if tied.is_empty() {
            return;
        }
        let victim = if self.config.tie_break == TieBreak::Random {
            tied[index(&mut self.rng, tied.len())]
        } else {
            tied[0]
        };
        let key = FrontKey::foreign(actor, victim);
        let side = usize::from(key.b == actor);
        let support = self.supported(actor, victim);
        let comparison = if self.config.allocation == Allocation::Pra {
            self.deterrence_commitments(actor, victim)
        } else {
            support
        };
        if comparison.iter().any(|v| !v.is_finite())
            || resources::ratio_overflows(comparison[0], comparison[1])
        {
            self.finish(
                "invalid",
                Some(format!(
                    "period {} front {:?}: nonfinite initiation ratio/support; config {:?}",
                    self.period, key, self.config
                )),
            );
            return;
        }
        let r = resources::ratio(comparison[0], comparison[1]);
        let initiate = if self.config.variant == Variant::Overextension {
            self.rng.gen::<f64>()
                < combat::probability(
                    r,
                    self.config.stochastic_threshold,
                    self.config.stochastic_exponent,
                )
        } else if self.config.allocation == Allocation::Pra
            && self.config.pra_attack_rule == PraAttackRule::LiteralProse
        {
            r < self.config.superiority
        } else {
            r > self.config.superiority
        };
        if initiate {
            let f = self.fronts.get_mut(&key).unwrap();
            f.actions[side] = true;
            f.initiated[side] = true;
            e.attacks += 1;
            self.log("initiation", vec![], Some([actor, victim]));
        }
    }
    pub(super) fn paths(&mut self, e: &mut Ledger) {
        let keys: Vec<FrontKey> = self.fronts.keys().copied().collect();
        for key in keys {
            let f = &self.fronts[&key];
            if !f.actions[0] && !f.actions[1] {
                continue;
            }
            let initiated = f.initiated;
            let redraw =
                self.config.combat_path == CombatPath::RedrawEachPeriod || f.path.is_none();
            if !redraw {
                continue;
            }
            let first = if (initiated[1] && !initiated[0]) || !f.actions[0] {
                1
            } else {
                0
            };
            let mut path = self.sample_path(key, first);
            if initiated == [true, true] {
                let other = self.sample_path(key, 1);
                e.path_collisions += 1;
                if self.config.path_collision == PathCollision::Random && self.rng.gen::<bool>() {
                    path = other;
                }
            }
            self.fronts.get_mut(&key).unwrap().path = Some(path);
        }
    }
    pub(super) fn front_count(&self, actor: usize, _domestic: bool, _unused: bool) -> usize {
        territory::neighbors(&self.config, &self.cells, actor).len()
            + if self.config.provincial() {
                territory::members(&self.cells, actor).len() - 1
            } else {
                0
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tft_echo_allows_predator_reinitiation_to_mutual_defection() {
        assert_eq!(
            provisional(ActionMemory::PreviousAction, [true, false], true),
            [false, true]
        );
        assert!(may_initiate(
            SchlieffenGate::OwnCurrentDefections,
            &[false],
            &[[true, false]],
            true
        ));
        assert!(!may_initiate(
            SchlieffenGate::PreviousHostilities,
            &[false],
            &[[true, false]],
            true
        ));
        assert_eq!(
            provisional(ActionMemory::WarUntilVictory, [true, false], true),
            [true, true]
        );
    }
}
