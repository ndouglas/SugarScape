//! Signed stock ledgers, source damage matrix, and literal PRA.
use super::{
    analysis::Ledger,
    config::*,
    decision::FrontKey,
    territory,
    world::{discount, draw, PolarityWorld},
};
use std::collections::{BTreeMap, BTreeSet};
pub fn ratio_overflows(a: f64, b: f64) -> bool {
    b != 0.0 && !(a / b).is_finite()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chapter4_dc_damages_only_cooperator_and_signed_damage_creates() {
        assert_eq!(damage([true, false], [20.0, 10.0], 0.05, true), [0.0, 1.0]);
        assert_eq!(damage([true, true], [-20.0, 10.0], 0.05, true), [0.5, -1.0]);
        assert_eq!(damage([true, false], [20.0, 10.0], 0.05, false), [0.0, 0.0]);
    }
    #[test]
    fn pra_passive_fronts_are_conditional_and_not_normalized() {
        assert_eq!(
            pra(100.0, &[20.0, 30.0, 10.0], &[true, true, false]),
            vec![40.0, 60.0, 100.0 / 6.0]
        );
        assert_eq!(pra(100.0, &[0.0, 0.0], &[false, false]), vec![100.0, 100.0]);
        assert_eq!(pra(100.0, &[0.0, 0.0], &[true, true]), vec![50.0, 50.0]);
    }
    #[test]
    fn ratios_keep_signed_zero_semantics() {
        assert_eq!(ratio(0.0, 0.0), 0.0);
        assert_eq!(ratio(-2.0, 0.0), f64::NEG_INFINITY);
        assert_eq!(ratio(2.0, 0.0), f64::INFINITY);
        assert_eq!(ratio(-4.0, 2.0), -2.0);
    }
}
pub fn damage(actions: [bool; 2], commitments: [f64; 2], rate: f64, asymmetric: bool) -> [f64; 2] {
    match actions {
        [false, false] => [0.0, 0.0],
        [true, true] => [rate * commitments[1], rate * commitments[0]],
        [true, false] if asymmetric => [0.0, rate * commitments[0]],
        [false, true] if asymmetric => [rate * commitments[1], 0.0],
        _ => [0.0, 0.0],
    }
}
pub fn pra(stock: f64, opposing: &[f64], active: &[bool]) -> Vec<f64> {
    let sum: f64 = opposing
        .iter()
        .zip(active)
        .filter(|(_, a)| **a)
        .map(|(r, _)| *r)
        .sum();
    let count = active.iter().filter(|a| **a).count();
    opposing
        .iter()
        .zip(active)
        .map(|(r, a)| {
            let denominator = if *a { sum } else { sum + *r };
            if denominator == 0.0 {
                if *a {
                    stock / count as f64
                } else if count == 0 {
                    stock
                } else {
                    0.0
                }
            } else {
                stock * *r / denominator
            }
        })
        .collect()
}
pub fn ratio(a: f64, b: f64) -> f64 {
    if b == 0.0 {
        if a > 0.0 {
            f64::INFINITY
        } else if a < 0.0 {
            f64::NEG_INFINITY
        } else {
            0.0
        }
    } else {
        a / b
    }
}
#[cfg(test)]
mod matrix_motifs {
    use super::*;
    #[test]
    fn cc_has_no_damage_even_with_negative_allocations() {
        assert_eq!(
            damage([false, false], [-20.0, -10.0], 0.05, true),
            [0.0, 0.0]
        );
    }
    #[test]
    fn chapter4_cd_is_dc_mirror() {
        assert_eq!(damage([false, true], [20.0, 10.0], 0.05, true), [0.5, 0.0]);
    }
    #[test]
    fn chapter5_dd_keeps_both_losses() {
        assert_eq!(damage([true, true], [20.0, 10.0], 0.05, false), [0.5, 1.0]);
    }
    #[test]
    fn signed_pra_denominator_is_not_absolute_value() {
        assert_eq!(pra(100.0, &[-20.0, -30.0], &[true, true]), vec![40.0, 60.0]);
    }
    #[test]
    fn zero_active_sum_splits_only_active_fronts() {
        assert_eq!(
            pra(90.0, &[0.0, 0.0, 10.0], &[true, true, false]),
            vec![45.0, 45.0, 90.0]
        );
    }
}

impl PolarityWorld {
    pub(super) fn allocate(&self, stocks: &[f64]) -> BTreeMap<FrontKey, [f64; 2]> {
        let mut out: BTreeMap<FrontKey, [f64; 2]> =
            self.fronts.keys().map(|&k| (k, [0.0, 0.0])).collect();
        for actor in territory::capitals(&self.cells) {
            let fronts: Vec<(FrontKey, usize)> = self
                .fronts
                .keys()
                .filter_map(|&k| {
                    if k.a == actor {
                        Some((k, 0))
                    } else if !k.domestic && k.b == actor {
                        Some((k, 1))
                    } else {
                        None
                    }
                })
                .collect();
            if fronts.is_empty() {
                continue;
            }
            let values = if self.config.allocation == Allocation::Equal {
                vec![stocks[actor] / fronts.len() as f64; fronts.len()]
            } else {
                let opposing: Vec<f64> = fronts
                    .iter()
                    .map(|(k, side)| self.fronts[k].old_commitments[1 - side])
                    .collect();
                let active: Vec<bool> = fronts
                    .iter()
                    .map(|(k, _)| {
                        let p = self.fronts[k].previous;
                        match self.config.pra_active {
                            PraActive::EitherDefection => p[0] || p[1],
                            PraActive::MutualDefection => p[0] && p[1],
                        }
                    })
                    .collect();
                pra(stocks[actor], &opposing, &active)
            };
            for ((k, side), v) in fronts.into_iter().zip(values) {
                out.get_mut(&k).unwrap()[side] = v;
            }
        }
        for (&key, value) in &mut out {
            if key.domestic {
                value[1] = stocks[key.b];
            }
        }
        out
    }
    pub(super) fn harvest(&mut self, e: &mut Ledger) {
        let mut changes = vec![0.0; self.cells.len()];
        for c in &self.cells {
            let h = draw(
                &self.config,
                &mut self.rng,
                self.config.harvest_mean,
                self.config.harvest_sd,
            );
            e.harvest += h;
            if !self.config.provincial() {
                changes[c.capital] += h;
            } else if c.capital == c.id {
                changes[c.id] += h;
            } else {
                let rate = if self.config.variant == Variant::Overextension {
                    self.config.tax_rate
                        * discount(self.config.tax_discount, self.distance(c.capital, c.id))
                } else {
                    self.config.tax_rate
                };
                let tax = if c.stock == 0.0 { 0.0 } else { h * rate };
                changes[c.capital] += tax;
                changes[c.id] += h - tax;
                e.taxes += tax;
            }
        }
        for (c, change) in self.cells.iter_mut().zip(changes) {
            c.stock += change;
        }
    }
    pub(super) fn distance(&self, a: usize, b: usize) -> u32 {
        if self.config.tax_distance == TaxDistance::Manhattan {
            let w = self.config.width as usize;
            ((a % w).abs_diff(b % w) + (a / w).abs_diff(b / w)) as u32
        } else {
            let mut queue = std::collections::VecDeque::from([(a, 0)]);
            let mut seen = BTreeSet::from([a]);
            while let Some((i, d)) = queue.pop_front() {
                if i == b {
                    return d;
                }
                for j in territory::adjacent(&self.config, i) {
                    if self.cells[j].capital == a && seen.insert(j) {
                        queue.push_back((j, d + 1));
                    }
                }
            }
            0
        }
    }
    pub(super) fn policy(&mut self, e: &mut Ledger) {
        if !self.intermediate_valid() {
            return;
        }
        let bad = self
            .cells
            .iter()
            .find(|c| !c.stock.is_finite())
            .map(|c| c.id);
        if let Some(i) = bad {
            self.finish(
                "invalid",
                Some(format!(
                    "period {} cell {i}: nonfinite stock; config {:?}",
                    self.period, self.config
                )),
            );
            return;
        }
        match self.config.resource_policy {
            ResourcePolicy::Signed => {}
            ResourcePolicy::FloorZero => {
                for c in &mut self.cells {
                    if c.stock < 0.0 {
                        e.clipping -= c.stock;
                        c.stock = 0.0;
                    }
                }
            }
            ResourcePolicy::RejectNonpositive => {
                if let Some(c) = self
                    .cells
                    .iter()
                    .find(|c| c.stock <= 0.0 && (self.config.provincial() || c.id == c.capital))
                {
                    let message = format!(
                        "period {} cell {}: nonpositive stock {} under reject_nonpositive",
                        self.period, c.id, c.stock
                    );
                    self.finish("invalid", Some(message));
                }
            }
        }
    }
    pub(super) fn intermediate_valid(&mut self) -> bool {
        let stock_sum = self.cells.iter().map(|c| c.stock).sum::<f64>();
        let bad = self.cells.iter().find(|c| {
            !c.stock.is_finite()
                || (self.config.resource_policy == ResourcePolicy::RejectNonpositive
                    && c.stock <= 0.0
                    && (self.config.provincial() || c.id == c.capital))
        });
        if let Some(c) = bad {
            self.finish(
                "invalid",
                Some(format!(
                    "period {} cell {}: invalid stock {} under {:?}",
                    self.period, c.id, c.stock, self.config.resource_policy
                )),
            );
            false
        } else if !stock_sum.is_finite() {
            self.finish(
                "invalid",
                Some(format!(
                    "period {}: nonfinite aggregate stock; config {:?}",
                    self.period, self.config
                )),
            );
            false
        } else {
            true
        }
    }
}
