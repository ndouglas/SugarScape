//! Portable probabilistic encounters and mutually exclusive claims.
use super::combat;
use super::resources::ratio;
use super::{
    analysis::{Episode, Ledger},
    config::*,
    decision::FrontKey,
    resources,
    territory::{self, Claim},
    world::{shuffle, PolarityWorld},
};
use rand::Rng;
use std::collections::BTreeSet;
pub fn probability(r: f64, threshold: f64, exponent: f64) -> f64 {
    if r <= 0.0 {
        return 0.0;
    }
    if r == f64::INFINITY {
        return 1.0;
    }
    // Normalize subnormals before the portable logarithm.
    fn log(x: f64) -> f64 {
        if x.is_subnormal() {
            crate::portable::ln(x * 4503599627370496.0) - 52.0 * std::f64::consts::LN_2
        } else {
            crate::portable::ln(x)
        }
    }
    let z = exponent * (log(r) - log(threshold));
    if z >= 0.0 {
        1.0 / (1.0 + crate::portable::exp_neg(-z))
    } else {
        let e = crate::portable::exp_neg(z);
        e / (1.0 + e)
    }
}
pub fn single_draw(a: f64, b: f64, draw: f64) -> Option<usize> {
    if draw < a {
        Some(0)
    } else if draw < a + b {
        Some(1)
    } else {
        None
    }
}
pub fn deterministic(commitments: [f64; 2], threshold: f64) -> Option<usize> {
    if ratio(commitments[0], commitments[1]) > threshold {
        Some(0)
    } else if ratio(commitments[1], commitments[0]) > threshold {
        Some(1)
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn threshold_equality_is_not_victory() {
        assert_eq!(deterministic([20.0, 10.0], 2.0), None);
        assert_eq!(deterministic([21.0, 10.0], 2.0), Some(0));
        assert_eq!(deterministic([10.0, 21.0], 2.0), Some(1));
    }
    #[test]
    fn logistic_endpoints_and_midpoint() {
        assert_eq!(probability(0.0, 3.0, 5.0), 0.0);
        assert_eq!(probability(f64::INFINITY, 3.0, 5.0), 1.0);
        assert_eq!(probability(3.0, 3.0, 5.0), 0.5);
    }
    #[test]
    fn single_draw_competing_victories_have_adjacent_intervals() {
        assert_eq!(single_draw(0.3, 0.2, 0.25), Some(0));
        assert_eq!(single_draw(0.3, 0.2, 0.4), Some(1));
        assert_eq!(single_draw(0.3, 0.2, 0.8), None);
    }
}
#[cfg(test)]
mod numeric_motifs {
    use super::*;
    #[test]
    fn nonpositive_ratios_have_zero_stochastic_success() {
        for r in [-100.0, -0.1, 0.0] {
            assert_eq!(probability(r, 3.0, 5.0), 0.0);
        }
    }
    #[test]
    fn equal_power_does_not_imply_equal_half_chance() {
        assert!((probability(1.0, 3.0, 5.0) - 1.0 / 244.0).abs() < 1e-16);
    }
    #[test]
    fn reverse_probabilities_leave_stalemate_mass() {
        assert!(probability(2.0, 3.0, 5.0) + probability(0.5, 3.0, 5.0) < 1.0);
    }
    #[test]
    fn subnormal_positive_ratio_does_not_panic_portable_log() {
        assert_eq!(probability(f64::from_bits(1), 3.0, 5.0), 0.0);
    }
    #[test]
    fn zero_stock_both_sides_stalemate() {
        assert_eq!(deterministic([0.0, 0.0], 2.0), None);
    }
    #[test]
    fn defender_can_win_instead_of_forced_stalemate() {
        assert_eq!(deterministic([10.0, 30.0], 2.0), Some(1));
    }
}

impl PolarityWorld {
    pub(super) fn victory(
        &mut self,
        key: FrontKey,
        values: [f64; 2],
        e: &mut Ledger,
    ) -> Option<usize> {
        if values.iter().any(|v| !v.is_finite()) {
            self.finish(
                "invalid",
                Some(format!(
                    "period {} front {:?}: nonfinite victory allocation; config {:?}",
                    self.period, key, self.config
                )),
            );
            return None;
        }
        if resources::ratio_overflows(values[0], values[1])
            || resources::ratio_overflows(values[1], values[0])
        {
            self.finish(
                "invalid",
                Some(format!(
                    "period {} front {:?}: nonfinite ratio with nonzero denominator; config {:?}",
                    self.period, key, self.config
                )),
            );
            return None;
        }
        if self.config.variant != Variant::Overextension {
            return combat::deterministic(
                values,
                if key.domestic {
                    2.0
                } else {
                    self.config.victory
                },
            );
        }
        let threshold = if key.domestic {
            2.0
        } else {
            self.config.stochastic_threshold
        };
        let a = combat::probability(
            resources::ratio(values[0], values[1]),
            threshold,
            self.config.stochastic_exponent,
        );
        let b = combat::probability(
            resources::ratio(values[1], values[0]),
            threshold,
            self.config.stochastic_exponent,
        );
        match self.config.stochastic_resolution {
            StochasticResolution::SingleDraw => combat::single_draw(a, b, self.rng.gen::<f64>()),
            StochasticResolution::IndependentDraws => {
                let av = self.rng.gen::<f64>() < a;
                let bv = self.rng.gen::<f64>() < b;
                match (av, bv) {
                    (true, false) => Some(0),
                    (false, true) => Some(1),
                    (true, true) => {
                        e.double_successes += 1;
                        None
                    }
                    _ => None,
                }
            }
        }
    }
    pub(super) fn episode_period(&mut self, key: FrontKey, losses: [f64; 2]) {
        let actions = self.fronts[&key].actions;
        if !actions[0] && !actions[1] {
            self.end_episode(key, "cooperation", None);
            return;
        }
        if self.fronts[&key].episode.is_none() {
            let f = &self.fronts[&key];
            let initiator = if f.initiated[1] && !f.initiated[0] {
                key.b
            } else if actions[0] {
                key.a
            } else {
                key.b
            };
            let ep = Episode {
                id: self.next_episode,
                domestic: key.domestic,
                capitals: [key.a, key.b],
                initiator,
                start: self.period,
                end: None,
                duration: 0,
                path: f.path.unwrap(),
                actions,
                initial_sizes: [
                    territory::members(&self.cells, key.a).len(),
                    if key.domestic {
                        1
                    } else {
                        territory::members(&self.cells, key.b).len()
                    },
                ],
                positive_loss: 0.0,
                signed_creation: 0.0,
                end_cause: None,
                winner: None,
                censored: false,
            };
            self.next_episode += 1;
            self.fronts.get_mut(&key).unwrap().episode = Some(ep);
        }
        let ep = self.fronts.get_mut(&key).unwrap().episode.as_mut().unwrap();
        ep.duration += 1;
        ep.actions = actions;
        for loss in losses {
            if loss > 0.0 {
                ep.positive_loss += loss;
            } else {
                ep.signed_creation -= loss;
            }
        }
    }
    pub(super) fn end_episode(&mut self, key: FrontKey, cause: &str, winner: Option<usize>) {
        if let Some(f) = self.fronts.get_mut(&key) {
            if let Some(mut ep) = f.episode.take() {
                ep.end = Some(self.period);
                ep.end_cause = Some(cause.into());
                ep.winner = winner;
                self.episodes.push(ep);
            }
            if cause == "victory" {
                f.actions = [false, false];
                f.previous = [false, false];
                f.path = None;
            }
        }
    }
    pub(super) fn claim(&self, key: FrontKey, side: usize) -> Option<Claim> {
        let path = self.fronts[&key].path?;
        if key.domestic {
            if side == 1 {
                Some(Claim {
                    winner: key.b,
                    loser: key.a,
                    target: key.b,
                    domestic: true,
                    source: None,
                })
            } else {
                None
            }
        } else {
            Some(Claim {
                winner: if side == 0 { key.a } else { key.b },
                loser: if side == 0 { key.b } else { key.a },
                target: path[1 - side],
                domestic: false,
                source: Some(path[side]),
            })
        }
    }
    pub(super) fn structural(&mut self, claims: &[Claim], e: &mut Ledger) {
        let change = territory::apply_locked(
            &self.config,
            &mut self.cells,
            claims,
            &mut self.period_locks,
        );
        e.transfers += change.transfer_volume;
        for claim in &change.applied {
            if !claim.domestic {
                self.aggression.insert(claim.winner);
                self.dyadic_aggression.insert((claim.winner, claim.loser));
            }
        }
        e.conquests += change.conquests;
        e.capital_collapses += change.collapses;
        e.disconnections += change.disconnections;
        e.stale_claims += change.stale;
        e.locked_claims += change.locked;
        if !change.cells.is_empty() {
            self.log("structure", change.cells, None);
            self.rebuild();
        }
    }
    pub(super) fn sequential(&mut self, mut caps: Vec<usize>, e: &mut Ledger) {
        shuffle(&mut caps, &mut self.rng);
        let mut visited = BTreeSet::new();
        for actor in caps {
            if self.cells[actor].capital != actor {
                continue;
            }
            self.rebuild();
            let stocks = self.cells.iter().map(|c| c.stock).collect::<Vec<_>>();
            let values = self.allocate(&stocks);
            for (k, v) in values {
                if !visited.contains(&k) {
                    self.fronts.get_mut(&k).unwrap().commitments = v;
                }
            }
            self.decide_actor(actor, e);
            if self.outcome.is_some() {
                return;
            }
            self.obligations();
            if self.outcome.is_some() {
                return;
            }
            self.paths(e);
            if self.outcome.is_some() {
                return;
            }
            let keys: Vec<FrontKey> = self
                .fronts
                .keys()
                .filter(|k| k.a == actor || (!k.domestic && k.b == actor))
                .copied()
                .collect();
            for key in keys {
                if !visited.insert(key) {
                    continue;
                }
                self.resolved_fronts.insert(key);
                let f = &self.fronts[&key];
                if f.commitments.iter().any(|v| !v.is_finite()) {
                    self.finish(
                        "invalid",
                        Some(format!(
                            "period {} front {:?}: nonfinite sequential commitment",
                            self.period, key
                        )),
                    );
                    return;
                }
                let losses = resources::damage(
                    f.actions,
                    f.commitments,
                    self.config.damage_rate,
                    self.config.asymmetric_losses(),
                );
                let old = f.commitments;
                if f.actions == [true, true] {
                    e.dd_encounters += 1;
                }
                self.episode_period(key, losses);
                let active = self.fronts[&key].actions.iter().any(|x| *x);
                let early = if active && self.config.victory_timing == VictoryTiming::BeforeDamage {
                    self.victory(key, old, e)
                } else {
                    None
                };
                if self.outcome.is_some() {
                    return;
                }
                self.cells[key.a].stock -= losses[0];
                self.cells[key.b].stock -= losses[1];
                for loss in losses {
                    if loss > 0.0 {
                        e.destruction += loss;
                    } else {
                        e.signed_creation -= loss;
                    }
                }
                if !self.intermediate_valid() {
                    return;
                }
                let winner = if active && self.config.victory_timing != VictoryTiming::BeforeDamage
                {
                    let values =
                        self.allocate(&self.cells.iter().map(|c| c.stock).collect::<Vec<_>>());
                    self.victory(key, values[&key], e)
                } else {
                    early
                };
                if self.outcome.is_some() {
                    return;
                }
                self.note_aggression();
                if let Some(side) = winner {
                    let claim = self.claim(key, side);
                    let winner = if side == 0 { key.a } else { key.b };
                    self.end_episode(key, "victory", Some(winner));
                    if let Some(claim) = claim {
                        self.structural(&[claim], e);
                    }
                }
            }
        }
        self.harvest(e);
    }
}
