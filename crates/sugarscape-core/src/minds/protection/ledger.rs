//! Diagnostic original-food lineage; never consulted by biological decisions.
use crate::agent::AgentId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
pub type CohortId = u32;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Outflow {
    Consumption,
    BurialCost,
    Deposit { site: u32 },
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CohortBalance {
    pub initial: f64,
    pub carried: f64,
    pub cached: BTreeMap<u32, f64>,
    pub consumed: f64,
    pub transferred: f64,
    pub cost: f64,
    pub lost_carried: f64,
    pub lost_cached: BTreeMap<u32, f64>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ledger {
    pub owner: AgentId,
    pub cohorts: BTreeMap<CohortId, CohortBalance>,
    pub unlabelled_carried: f64,
    pub unlabelled_cached: BTreeMap<u32, f64>,
}
fn valid(amount: f64) -> Result<(), String> {
    if !amount.is_finite() || amount < 0.0 {
        return Err("invalid protection ledger amount".into());
    }
    Ok(())
}
pub(crate) fn proportional_outflow(balances: &[f64], amount: f64) -> Result<Vec<f64>, String> {
    if !amount.is_finite() || amount < 0.0 || balances.iter().any(|b| !b.is_finite() || *b <= 0.0) {
        return Err("invalid protection ledger outflow".into());
    }
    let total: f64 = balances.iter().sum();
    let tolerance = 1e-9 * total.max(1.0);
    if !total.is_finite() || amount > total + tolerance {
        return Err("protection ledger outflow exceeds available food".into());
    }
    if amount == 0.0 {
        return Ok(vec![0.0; balances.len()]);
    }
    let debit = amount.min(total);
    let mut shares = Vec::with_capacity(balances.len());
    let mut left = debit;
    for (i, balance) in balances.iter().enumerate() {
        let share = if i + 1 == balances.len() {
            left
        } else {
            (debit * (balance / total)).min(left)
        };
        if share > balance + 1e-9 * balance.max(1.0) {
            return Err("protection ledger share exceeds balance".into());
        }
        shares.push(share);
        left -= share;
    }
    Ok(shares)
}
impl Ledger {
    pub fn new(owner: AgentId, holdings: f64) -> Self {
        Self {
            owner,
            cohorts: BTreeMap::new(),
            unlabelled_carried: holdings,
            unlabelled_cached: BTreeMap::new(),
        }
    }
    pub fn prepare(&mut self, source: u32, amount: f64) -> Result<(), String> {
        valid(amount)?;
        if self.cohorts.contains_key(&source) {
            return Err("duplicate protection cohort".into());
        }
        let debit = proportional_outflow(&self.positive_unlabelled(), amount)?
            .first()
            .copied()
            .unwrap_or(0.0);
        self.unlabelled_carried -= debit;
        self.cohorts.insert(
            source,
            CohortBalance {
                initial: amount,
                cached: BTreeMap::from([(source, debit)]),
                ..Default::default()
            },
        );
        self.reconcile()
    }
    fn positive_unlabelled(&self) -> Vec<f64> {
        if self.unlabelled_carried > 0.0 {
            vec![self.unlabelled_carried]
        } else {
            vec![]
        }
    }
    fn shares(
        &self,
        site: Option<u32>,
        amount: f64,
    ) -> Result<Vec<(Option<CohortId>, f64)>, String> {
        let mut entries: Vec<_> = self
            .cohorts
            .iter()
            .filter_map(|(&id, c)| {
                let balance = site.map_or(c.carried, |s| c.cached.get(&s).copied().unwrap_or(0.0));
                (balance > 0.0).then_some((Some(id), balance))
            })
            .collect();
        let unlabelled = site.map_or(self.unlabelled_carried, |s| {
            self.unlabelled_cached.get(&s).copied().unwrap_or(0.0)
        });
        if unlabelled > 0.0 {
            entries.push((None, unlabelled));
        }
        let shares =
            proportional_outflow(&entries.iter().map(|e| e.1).collect::<Vec<_>>(), amount)?;
        Ok(entries
            .into_iter()
            .zip(shares)
            .map(|((id, _), q)| (id, q))
            .collect())
    }
    pub fn outflow(&mut self, amount: f64, kind: Outflow) -> Result<(), String> {
        for (id, q) in self.shares(None, amount)? {
            if let Some(id) = id {
                let c = self.cohorts.get_mut(&id).unwrap();
                c.carried -= q;
                match kind {
                    Outflow::Consumption => c.consumed += q,
                    Outflow::BurialCost => c.cost += q,
                    Outflow::Deposit { site } => *c.cached.entry(site).or_default() += q,
                }
            } else {
                self.unlabelled_carried -= q;
                if let Outflow::Deposit { site } = kind {
                    *self.unlabelled_cached.entry(site).or_default() += q;
                }
            }
        }
        self.reconcile()
    }
    fn cache_outflow(&mut self, site: u32, amount: f64, pilfer: bool) -> Result<(), String> {
        for (id, q) in self.shares(Some(site), amount)? {
            if let Some(id) = id {
                let c = self.cohorts.get_mut(&id).unwrap();
                *c.cached.get_mut(&site).unwrap() -= q;
                if pilfer {
                    c.transferred += q;
                } else {
                    c.carried += q;
                }
            } else {
                *self.unlabelled_cached.get_mut(&site).unwrap() -= q;
                if !pilfer {
                    self.unlabelled_carried += q;
                }
            }
        }
        self.reconcile()
    }
    pub fn withdraw(&mut self, site: u32, amount: f64) -> Result<(), String> {
        self.cache_outflow(site, amount, false)
    }
    pub fn pilfer(&mut self, site: u32, amount: f64) -> Result<(), String> {
        self.cache_outflow(site, amount, true)
    }
    pub fn harvest(&mut self, amount: f64) -> Result<(), String> {
        valid(amount)?;
        self.unlabelled_carried += amount;
        self.reconcile()
    }
    pub fn lose_owner(&mut self) {
        for c in self.cohorts.values_mut() {
            c.lost_carried += c.carried;
            c.carried = 0.0;
            for (site, q) in std::mem::take(&mut c.cached) {
                *c.lost_cached.entry(site).or_default() += q;
            }
        }
        self.unlabelled_carried = 0.0;
        self.unlabelled_cached.clear();
    }
    pub fn reconcile(&self) -> Result<(), String> {
        for q in
            std::iter::once(self.unlabelled_carried).chain(self.unlabelled_cached.values().copied())
        {
            if !q.is_finite() || q < -1e-9 * q.abs().max(1.0) {
                return Err("invalid protection unlabelled balance".into());
            }
        }
        for (&id, c) in &self.cohorts {
            let tolerance = 1e-9 * c.initial.max(1.0);
            for q in [
                c.initial,
                c.carried,
                c.consumed,
                c.transferred,
                c.cost,
                c.lost_carried,
            ]
            .into_iter()
            .chain(c.cached.values().copied())
            .chain(c.lost_cached.values().copied())
            {
                if !q.is_finite() || q < -tolerance {
                    return Err(format!("invalid protection cohort {id} balance"));
                }
            }
            let accounted = c.carried
                + c.cached.values().sum::<f64>()
                + c.consumed
                + c.transferred
                + c.cost
                + c.lost_carried
                + c.lost_cached.values().sum::<f64>();
            if (c.initial - accounted).abs() > tolerance {
                return Err(format!("protection cohort {id} does not reconcile"));
            }
        }
        Ok(())
    }
    pub(crate) fn reconcile_physical(
        &self,
        holdings: f64,
        caches: &BTreeMap<u32, f64>,
    ) -> Result<(), String> {
        self.reconcile()?;
        if !holdings.is_finite() || caches.values().any(|q| !q.is_finite() || *q < 0.0) {
            return Err("invalid engine protection food balance".into());
        }
        let carried =
            self.unlabelled_carried + self.cohorts.values().map(|c| c.carried).sum::<f64>();
        if (carried - holdings.max(0.0)).abs() > 1e-9 * holdings.abs().max(1.0) {
            return Err("protection carried food disagrees with engine".into());
        }
        let mut totals = self.unlabelled_cached.clone();
        for c in self.cohorts.values() {
            for (&site, &q) in &c.cached {
                *totals.entry(site).or_default() += q;
            }
        }
        for site in totals.keys().chain(caches.keys()) {
            let actual = caches.get(site).copied().unwrap_or(0.0);
            if (totals.get(site).copied().unwrap_or(0.0) - actual).abs() > 1e-9 * actual.max(1.0) {
                return Err(format!("protection cache {site} disagrees with engine"));
            }
        }
        Ok(())
    }
}
/// Diagnostic failure is absorbing: retain its reason, disable lineage updates.
pub(crate) fn update(
    world: &mut crate::world::World,
    id: AgentId,
    f: impl FnOnce(&mut Ledger) -> Result<(), String>,
) {
    if world
        .protection_ledger
        .as_ref()
        .is_none_or(|l| l.owner != id)
    {
        return;
    }
    let result = f(world.protection_ledger.as_mut().unwrap());
    if let Err(error) = result {
        world
            .protection_ledger_errors
            .push(format!("tick {} agent {id}: {error}", world.tick));
        world.protection_ledger = None;
    }
}
pub(crate) fn reconcile_world(world: &mut crate::world::World) {
    let Some(ledger) = &world.protection_ledger else {
        return;
    };
    let result = if let Some(a) = world.agent(ledger.owner) {
        ledger.reconcile_physical(a.holdings[0], &a.caches)
    } else {
        ledger.reconcile()
    };
    if let Err(error) = result {
        world
            .protection_ledger_errors
            .push(format!("tick {}: {error}", world.tick));
        world.protection_ledger = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protection_ledger_preserves_original_food_through_reburial() {
        let mut l = Ledger::new(1, 20.0);
        l.prepare(30, 10.0).unwrap();
        l.withdraw(30, 10.0).unwrap();
        l.outflow(2.0, Outflow::Consumption).unwrap();
        l.outflow(8.0, Outflow::Deposit { site: 21 }).unwrap();
        l.pilfer(21, 2.0).unwrap();
        let c = &l.cohorts[&30];
        assert_eq!(
            (c.carried, c.cached[&21], c.consumed, c.transferred),
            (5.0, 3.0, 1.0, 1.0)
        );
        l.reconcile().unwrap();
    }
    #[test]
    fn kernel_handles_zero_empty_partial_full_and_rounding() {
        assert_eq!(proportional_outflow(&[], 0.0).unwrap(), Vec::<f64>::new());
        for amount in [0.0, 1.0, 3.0] {
            let shares = proportional_outflow(&[1.0, 2.0], amount).unwrap();
            assert_eq!(shares.iter().sum::<f64>(), amount);
        }
        assert_eq!(
            proportional_outflow(&[1.0, 2.0], 3.0 + 1e-10).unwrap(),
            vec![1.0, 2.0]
        );
        let shares = proportional_outflow(&[0.1, 0.2, 0.3], 0.4).unwrap();
        assert_eq!(shares[2], 0.4 - shares[0] - shares[1]);
        for amount in [4.0, f64::NAN, f64::INFINITY, -1.0] {
            assert!(proportional_outflow(&[1.0, 2.0], amount).is_err());
        }
        assert!(proportional_outflow(&[0.0], 0.0).is_err());
    }
    #[test]
    fn mixed_cohorts_harvest_cost_and_death_conserve_food() {
        let mut l = Ledger::new(1, 12.0);
        l.prepare(1, 4.0).unwrap();
        l.prepare(2, 4.0).unwrap();
        l.withdraw(1, 4.0).unwrap();
        l.withdraw(2, 4.0).unwrap();
        l.harvest(4.0).unwrap();
        l.outflow(4.0, Outflow::BurialCost).unwrap();
        l.outflow(6.0, Outflow::Deposit { site: 3 }).unwrap();
        l.pilfer(3, 3.0).unwrap();
        let transferred = l.cohorts.values().map(|c| c.transferred).sum::<f64>();
        l.lose_owner();
        l.reconcile().unwrap();
        assert_eq!(transferred, 1.5);
        assert!(l
            .cohorts
            .values()
            .all(|c| c.carried == 0.0 && c.cached.is_empty()));
    }
    #[test]
    fn full_mixed_outflow_accepts_only_tiny_rounding_remainders() {
        let mut l = Ledger::new(1, 0.3);
        for (source, amount) in [(1, 0.1), (2, 0.2)] {
            l.cohorts.insert(
                source,
                CohortBalance {
                    initial: amount,
                    carried: amount,
                    ..Default::default()
                },
            );
        }
        let amount = l.unlabelled_carried + l.cohorts.values().map(|c| c.carried).sum::<f64>();
        l.outflow(amount, Outflow::Consumption).unwrap();
        l.reconcile_physical(0.0, &BTreeMap::new()).unwrap();
        l.unlabelled_carried = -1e-7;
        assert!(l.reconcile().is_err());
    }
    #[test]
    fn physical_reconciliation_rejects_nonfinite_and_inconsistent_engine_balances() {
        let mut l = Ledger::new(1, 10.0);
        l.prepare(30, 5.0).unwrap();
        let caches = BTreeMap::from([(30, 5.0)]);
        assert!(l.reconcile_physical(f64::NAN, &caches).is_err());
        assert!(l
            .reconcile_physical(5.0, &BTreeMap::from([(30, f64::NAN)]))
            .is_err());
        assert!(l
            .reconcile_physical(5.0, &BTreeMap::from([(30, -1.0)]))
            .is_err());
        assert!(l.reconcile_physical(4.0, &caches).is_err());
        assert!(l.reconcile_physical(5.0 + 1e-10, &caches).is_ok());
        assert!(l
            .reconcile_physical(5.0, &BTreeMap::from([(30, 5.0 + 1e-10)]))
            .is_ok());
    }
    #[test]
    fn rejected_transfers_do_not_mutate_balances() {
        let mut l = Ledger::new(1, 10.0);
        l.prepare(1, 5.0).unwrap();
        let before = l.clone();
        assert!(l.withdraw(1, 6.0).is_err());
        assert_eq!(l, before);
        assert!(l.outflow(6.0, Outflow::Consumption).is_err());
        assert_eq!(l, before);
        assert!(l.prepare(1, 1.0).is_err());
        assert_eq!(l, before);
        assert!(l.harvest(f64::NAN).is_err());
        assert_eq!(l, before);
    }
}
