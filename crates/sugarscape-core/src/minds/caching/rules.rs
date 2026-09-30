//! Minds 5: the caching rules — when an agent buries, how much, and where.
//! Digging is the same for every rule (`super::dig`); these only bury.
//!
//! - **Allocation** ([`allocate_even`], [`allocate_compensate`],
//!   [`allocate_plan`]): pure splits of an amount over places, shared by the
//!   field and the lab. Places come back in ascending order. With `whole`
//!   (the lab), shares are apportioned in integer arithmetic (see
//!   [`apportion_whole`]) and the remainder goes one unit at a time to the
//!   places in ascending order (K1 < K2 < K3); the field passes `whole =
//!   false`.
//! - **The field** ([`act`]), after each agent's move and harvest, before it
//!   eats:
//!   - `even` buries `share` × surplus at its current site, every tick.
//!   - `compensate` keeps a weight for each of its **known sites**: the
//!     sites it has harvested from (a tick where it moved and took its
//!     harvest from the site, not from a cache), at most [`MEMORY_CAP`] of
//!     them ([`Weights`]). Each tick it first updates the weight of the site
//!     it just harvested (w starts at 1 when the site becomes known; w ← w ×
//!     (1 − λ) when it found food, i.e. gathered good 0 > 0 from the site).
//!     A tick where it dug (`harvest.dug > 0`) harvested no site: no weight
//!     is added or changed. It then buries min(surplus, share × surplus × w
//!     / w̄) at its current site, w̄ the mean weight over its known sites
//!     (and w = 1 for a site it doesn't know). If every weight has decayed
//!     to exactly 0 (w̄ = 0), w / w̄ counts as 1: the sites are equal again.
//!   - `plan` caches only under `seasons.enabled` with `seasons.mode:
//!     global` (the calendar it plans by). With seasons off or in
//!     hemispheres mode it buries nothing: there's no winter everywhere to
//!     plan for. In winter it records its intake (good 0 gathered from sites
//!     only: dug sugar is its own store, not what winter yields), the ticks
//!     and the sites it stood on, a new record each winter. In summer the
//!     shortfall is effective metabolism × γ − forecast − Σ caches, the
//!     forecast being last winter's intake / ticks × γ, or 0 before its
//!     first winter (the worst case); it buries min(surplus, shortfall) when
//!     that's > 0, at its current site if the site is in last winter's
//!     sites, or anywhere before its first winter. Metabolism is effective
//!     metabolism (disease fees included), as the reserve's is.
//!
//! - **The lab** (`super::lab`) uses only the allocations, with `whole`;
//!   [`act`] buries nothing there.
//!
//! Nothing here draws.

use std::collections::{BTreeMap, BTreeSet};

use super::super::memory::MEMORY_CAP;
use super::{bury, surplus};
use crate::agent::AgentId;
use crate::config::{CachingRule, SeasonMode};
use crate::rules::growback::is_winter;
use crate::rules::Harvest;
use crate::world::World;

/// Rule `plan`'s record of one winter: what it gathered from sites, over
/// how many ticks, and the sites it occupied.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WinterRecord {
    pub intake: f64,
    pub ticks: u32,
    pub sites: BTreeSet<u32>,
}

impl WinterRecord {
    /// Forecast intake over a winter of `gamma` ticks: intake per tick ×
    /// γ, or 0 for an empty record.
    pub fn forecast(&self, gamma: u32) -> f64 {
        if self.ticks == 0 {
            return 0.0;
        }
        self.intake / f64::from(self.ticks) * f64::from(gamma)
    }
}

/// Rule `compensate`'s known sites: site index → weight, at most
/// [`MEMORY_CAP`] of them, with a running sum so the mean costs O(1).
///
/// - **Cap.** Adding a site to a full map first drops the known site with
///   the lowest site index other than the one being added (deterministic;
///   there's no recency to go by without another map).
/// - **Running sum.** Updated by each change's difference, and re-summed
///   from the map every `len` changes (amortized O(1)) so rounding drift
///   stays bounded; between re-sums the mean may differ from a fresh sum in
///   the last bits. A count of nonzero weights makes "every weight is 0"
///   exact, whatever the drift.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Weights {
    map: BTreeMap<u32, f64>,
    sum: f64,
    nonzero: usize,
    changes: usize,
}

impl Weights {
    /// The weights, by site index.
    pub fn map(&self) -> &BTreeMap<u32, f64> {
        &self.map
    }

    pub fn get(&self, site: u32) -> Option<f64> {
        self.map.get(&site).copied()
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Sets `site`'s weight to `w` (≥ 0), making it known (dropping the
    /// lowest other site first when full).
    pub fn set(&mut self, site: u32, w: f64) {
        let w = w.max(0.0);
        let old = match self.map.get(&site).copied() {
            Some(old) => old,
            None => {
                if self.map.len() >= MEMORY_CAP {
                    let (&drop, &dw) = self.map.iter().next().expect("full map");
                    self.map.remove(&drop);
                    self.sum -= dw;
                    self.nonzero -= usize::from(dw > 0.0);
                }
                0.0
            }
        };
        let known = self.map.insert(site, w).is_some();
        self.sum += w - old;
        self.nonzero = self.nonzero + usize::from(w > 0.0) - usize::from(known && old > 0.0);
        self.changes += 1;
        if self.changes >= self.map.len() {
            self.sum = self.map.values().sum();
            self.changes = 0;
        }
    }

    /// Makes `site` known at weight 1 if it isn't yet, then × (1 − λ) if
    /// the agent found food there.
    pub fn harvested(&mut self, site: u32, found: bool, lambda: f64) {
        let w = self.get(site).unwrap_or(1.0);
        self.set(site, if found { w * (1.0 - lambda) } else { w });
    }

    /// w̄: the mean weight over the known sites (1 when there are none; 0
    /// exactly when every weight is 0).
    pub fn mean(&self) -> f64 {
        if self.map.is_empty() {
            1.0
        } else if self.nonzero == 0 {
            0.0
        } else {
            self.sum.max(0.0) / self.map.len() as f64
        }
    }
}

/// Rule `plan`'s forecast shortfall for a winter of `gamma` ticks: `burn` ×
/// γ − forecast − `cached`, the forecast from `last` (0 before a first
/// winter: the worst case). May be negative (nothing to bury).
pub fn shortfall(burn: f64, gamma: u32, last: Option<&WinterRecord>, cached: f64) -> f64 {
    let forecast = last.map_or(0.0, |r| r.forecast(gamma));
    burn * f64::from(gamma) - forecast - cached
}

/// Rule `compensate`'s burial at `site`: min(`surplus`, share × surplus × w
/// / w̄), w the site's weight (1 if unknown) and w̄ the mean over `weights`
/// (1 when empty). When w̄ is 0 every weight is 0, and w / w̄ counts as 1.
pub fn compensate_amount(surplus: f64, share: f64, weights: &Weights, site: u32) -> f64 {
    let w = weights.get(site).unwrap_or(1.0);
    let mean = weights.mean();
    let ratio = if mean > 0.0 { w / mean } else { 1.0 };
    (share * surplus * ratio).min(surplus).max(0.0)
}

/// Splits `amount` over `places` (sorted ascending, duplicates dropped) by
/// `weight`, each place's share its weight over the total (equal shares if
/// the total isn't positive). With `whole`, see [`apportion_whole`].
///
/// One place without `whole` (the field's burial, every tick) takes a fast
/// path that allocates only its result. It keeps the general path's
/// arithmetic, amount × w / w, since that needn't round back to the amount.
fn apportion(
    amount: f64,
    places: &[u32],
    whole: bool,
    weight: impl Fn(u32) -> f64,
) -> Vec<(u32, f64)> {
    if let (&[p], false) = (places, whole) {
        let amount = amount.max(0.0);
        let w = sanitize(weight(p));
        let q = if w > 0.0 { amount * w / w } else { amount };
        return vec![(p, q)];
    }
    apportion_general(amount, places, whole, weight)
}

/// A weight as [`apportion`] counts it: finite and ≥ 0 (0 otherwise).
fn sanitize(w: f64) -> f64 {
    if w.is_finite() {
        w.max(0.0)
    } else {
        0.0
    }
}

/// [`apportion`] for any number of places.
fn apportion_general(
    amount: f64,
    places: &[u32],
    whole: bool,
    weight: impl Fn(u32) -> f64,
) -> Vec<(u32, f64)> {
    let mut places = places.to_vec();
    places.sort_unstable();
    places.dedup();
    let k = places.len();
    if k == 0 {
        return Vec::new();
    }
    let weights: Vec<f64> = places.iter().map(|&p| sanitize(weight(p))).collect();
    if whole {
        return places
            .into_iter()
            .zip(apportion_whole(amount, &weights))
            .map(|(p, q)| (p, q as f64))
            .collect();
    }
    let amount = amount.max(0.0);
    let total: f64 = weights.iter().sum();
    places
        .iter()
        .zip(&weights)
        .map(|(&p, &w)| {
            let q = if total > 0.0 {
                amount * w / total
            } else {
                amount / k as f64
            };
            (p, q)
        })
        .collect()
}

/// Scale for [`apportion_whole`]'s integer weights: the largest weight maps
/// to 2⁵².
const WHOLE_SCALE: f64 = 4_503_599_627_370_496.0;

/// Tolerance for [`apportion_whole`], in units: a share within 10⁻⁶ of the
/// next whole unit counts as reaching it.
const WHOLE_EPSILON_INV: u128 = 1_000_000;

/// Whole units of floor(`amount`) (clamped to [0, 2⁶³]) apportioned by
/// `weights` (finite and ≥ 0; equal if none is positive), in integer
/// arithmetic.
///
/// - Each weight is scaled to an integer nᵢ = round(wᵢ / max w × 2⁵²), and
///   each share is ⌊(total × nᵢ + ⌊N / 10⁶⌋) / N⌋ with N = Σ nᵢ, in u128:
///   the exact rational share of the scaled weights, reaching the next unit
///   when it's within 10⁻⁶ of it. The slack covers the scaling's rounding
///   (relative error ≤ 2⁻⁵³ per weight, far below 10⁻⁶ of a unit for any
///   amount the lab uses), so a share that is mathematically a whole number
///   — 1/3 of 30, or 0.1 : 0.2 : 0.3 of 60 — never loses a unit to it.
/// - The units left over go one at a time to the places in order (ascending
///   place index), cycling if need be. Should the slack ever over-give (it
///   can't with fewer than 10⁶ places), units come back from the last place
///   first.
fn apportion_whole(amount: f64, weights: &[f64]) -> Vec<u64> {
    let k = weights.len();
    let total = if amount.is_finite() {
        amount.floor().clamp(0.0, 9_223_372_036_854_775_808.0) as u64
    } else {
        0
    };
    let max = weights.iter().copied().fold(0.0_f64, f64::max);
    let scaled: Vec<u128> = if max > 0.0 {
        weights
            .iter()
            .map(|&w| (w / max * WHOLE_SCALE).round() as u128)
            .collect()
    } else {
        vec![1; k]
    };
    let n: u128 = scaled.iter().sum();
    let slack = n / WHOLE_EPSILON_INV;
    let t = u128::from(total);
    let mut out: Vec<u64> = scaled
        .iter()
        .map(|&s| ((t * s + slack) / n).min(t) as u64)
        .collect();
    let given: u64 = out.iter().sum();
    if given <= total {
        for i in 0..(total - given) as usize {
            out[i % k] += 1;
        }
    } else {
        let mut over = given - total;
        for q in out.iter_mut().rev() {
            let back = over.min(*q);
            *q -= back;
            over -= back;
        }
    }
    out
}

/// `even`: `amount` split equally over `places`, ascending.
pub fn allocate_even(amount: f64, places: &[u32], whole: bool) -> Vec<(u32, f64)> {
    apportion(amount, places, whole, |_| 1.0)
}

/// `compensate`: `amount` × w_p / Σw over `places` (a place without a
/// weight counts 1), ascending; equal shares if every weight is 0.
pub fn allocate_compensate(
    amount: f64,
    places: &[u32],
    weights: &BTreeMap<u32, f64>,
    whole: bool,
) -> Vec<(u32, f64)> {
    apportion(amount, places, whole, |p| {
        weights.get(&p).copied().unwrap_or(1.0)
    })
}

/// `plan`: `amount` split equally over the places it predicts will lack
/// food (`needy`), ascending; nothing when it predicts none.
pub fn allocate_plan(amount: f64, needy: &[u32], whole: bool) -> Vec<(u32, f64)> {
    apportion(amount, needy, whole, |_| 1.0)
}

/// Buries each allocation in the field, where the only place is the agent's
/// current site.
fn bury_all(world: &mut World, id: AgentId, allocation: Vec<(u32, f64)>) {
    for (place, q) in allocation {
        debug_assert_eq!(
            place as usize,
            world.torus.index(world.agent(id).expect("live agent").pos),
            "the field buries only where the agent stands"
        );
        bury(world, id, q);
    }
}

/// The caching rule `id` follows: `none` for a Minds 6 cheater, whatever
/// the config says; else its own (`Agent.caching_rule`) under
/// `caching.mixed`, otherwise `caching.rule`.
pub(crate) fn rule_of(world: &World, id: AgentId) -> CachingRule {
    let a = world.agent(id).expect("live agent");
    if a.cheater {
        CachingRule::None
    } else if world.config.caching.mixed {
        a.caching_rule
    } else {
        world.config.caching.rule
    }
}

/// The field's caching step for `id`, after its move and `harvest` and
/// before it eats. Does nothing under rule `none`. In a lab world nothing is
/// buried here: the lab's schedule buries on the test evening, and this
/// only notes whether the agent found food today (`lab::found`).
pub(crate) fn act(world: &mut World, id: AgentId, harvest: &Harvest) {
    if world.config.lab.is_some() {
        super::lab::found(world, id, harvest);
        return;
    }
    let caching = world.config.caching;
    let here = {
        let a = world.agent(id).expect("live agent");
        world.torus.index(a.pos) as u32
    };
    match rule_of(world, id) {
        CachingRule::None => {}
        CachingRule::Even => {
            let amount = caching.share * surplus(world, id);
            bury_all(world, id, allocate_even(amount, &[here], false));
        }
        CachingRule::Compensate => {
            // A dig harvested no site: the known sites stay as they were.
            if harvest.dug <= 0.0 {
                let found = harvest.gathered[0] > 0.0;
                let a = world.agent_mut(id).expect("live agent");
                a.weights.harvested(here, found, caching.lambda);
            }
            let s = surplus(world, id);
            let weights = &world.agent(id).expect("live agent").weights;
            let amount = compensate_amount(s, caching.share, weights, here);
            let allocation = allocate_compensate(amount, &[here], weights.map(), false);
            bury_all(world, id, allocation);
        }
        CachingRule::Plan => plan(world, id, here, harvest),
    }
}

/// Rule `plan` in the field (see the module docs).
fn plan(world: &mut World, id: AgentId, here: u32, harvest: &Harvest) {
    let seasons = world.config.seasons;
    if !seasons.enabled || seasons.mode != SeasonMode::Global {
        return;
    }
    let fee = world.config.disease.active_fee();
    if is_winter(&world.config, world.tick) {
        let a = world.agent_mut(id).expect("live agent");
        a.this_winter.intake += harvest.gathered[0];
        a.this_winter.ticks += 1;
        a.this_winter.sites.insert(here);
        return;
    }
    let a = world.agent_mut(id).expect("live agent");
    if a.this_winter.ticks > 0 {
        a.last_winter = Some(std::mem::take(&mut a.this_winter));
    }
    let placed = a
        .last_winter
        .as_ref()
        .is_none_or(|r| r.sites.contains(&here));
    if !placed {
        return;
    }
    let burn = a.effective_metabolism(0, fee);
    let cached: f64 = a.caches.values().sum();
    let short = shortfall(burn, seasons.period, a.last_winter.as_ref(), cached);
    let amount = surplus(world, id).min(short);
    if amount > 0.0 {
        bury_all(world, id, allocate_plan(amount, &[here], false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MoveMode, Movement};
    use crate::geometry::Pos;
    use crate::testkit::*;

    #[test]
    fn one_places_fast_path_equals_the_general_path() {
        let amounts = [0.0, -3.0, 0.1, 1.0 / 3.0, 7.3, 12.345_678_9, 1e12, f64::NAN];
        let weights = [1.0, 0.0, -2.0, 0.3, 0.49, 1.0 / 7.0, 3.0, f64::INFINITY];
        for &amount in &amounts {
            for &w in &weights {
                let fast = apportion(amount, &[42], false, |_| w);
                let general = apportion_general(amount, &[42], false, |_| w);
                assert_eq!(fast.len(), 1);
                assert_eq!(fast[0].0, general[0].0);
                assert_eq!(
                    fast[0].1.to_bits(),
                    general[0].1.to_bits(),
                    "amount {amount}, weight {w}"
                );
            }
        }
    }

    fn map(entries: &[(u32, f64)]) -> BTreeMap<u32, f64> {
        entries.iter().copied().collect()
    }

    #[test]
    fn even_splits_equally_and_whole_units_give_remainders_in_ascending_order() {
        assert_eq!(
            allocate_even(30.0, &[2, 0], false),
            vec![(0, 15.0), (2, 15.0)]
        );
        assert_eq!(
            allocate_even(30.0, &[2, 1, 0], true),
            vec![(0, 10.0), (1, 10.0), (2, 10.0)]
        );
        // 31 over three: 10 each, the one left over to K1.
        assert_eq!(
            allocate_even(31.0, &[2, 1, 0], true),
            vec![(0, 11.0), (1, 10.0), (2, 10.0)]
        );
        // 32: K1 and K2 get one each. A fractional amount floors first.
        assert_eq!(
            allocate_even(32.7, &[0, 1, 2], true),
            vec![(0, 11.0), (1, 11.0), (2, 10.0)]
        );
        assert_eq!(allocate_even(5.0, &[], true), vec![]);
        assert_eq!(allocate_even(-1.0, &[4], false), vec![(4, 0.0)]);
        assert_eq!(allocate_even(7.0, &[3, 3], false), vec![(3, 7.0)]);
    }

    #[test]
    fn compensate_splits_by_weight_over_the_total() {
        // Weights 1, 0.5, 0.5 (Σ 2): 30 → 15, 7.5, 7.5.
        let w = map(&[(0, 1.0), (1, 0.5), (2, 0.5)]);
        assert_eq!(
            allocate_compensate(30.0, &[0, 1, 2], &w, false),
            vec![(0, 15.0), (1, 7.5), (2, 7.5)]
        );
        // Whole: 15, 7, 7 and one left over to K1.
        assert_eq!(
            allocate_compensate(30.0, &[0, 1, 2], &w, true),
            vec![(0, 16.0), (1, 7.0), (2, 7.0)]
        );
        // Weights 0.25, 0.5, 1 (Σ 1.75), 30 whole: 4.29, 8.57, 17.14 floor
        // to 4, 8, 17 (29); the one left goes to K1.
        let w = map(&[(0, 0.25), (1, 0.5), (2, 1.0)]);
        assert_eq!(
            allocate_compensate(30.0, &[0, 1, 2], &w, true),
            vec![(0, 5.0), (1, 8.0), (2, 17.0)]
        );
        // An unweighted place counts 1; all-zero weights split equally.
        assert_eq!(
            allocate_compensate(9.0, &[7, 8], &map(&[(7, 0.5)]), false),
            vec![(7, 3.0), (8, 6.0)]
        );
        assert_eq!(
            allocate_compensate(9.0, &[7, 8], &map(&[(7, 0.0), (8, 0.0)]), false),
            vec![(7, 4.5), (8, 4.5)]
        );
    }

    #[test]
    fn plan_splits_over_the_needy_and_buries_nothing_without_a_prediction() {
        assert_eq!(allocate_plan(30.0, &[1], true), vec![(1, 30.0)]);
        assert_eq!(
            allocate_plan(30.0, &[2, 0], true),
            vec![(0, 15.0), (2, 15.0)]
        );
        assert_eq!(
            allocate_plan(31.0, &[2, 0], true),
            vec![(0, 16.0), (2, 15.0)]
        );
        assert_eq!(allocate_plan(30.0, &[], true), vec![]);
    }

    #[test]
    fn whole_allocations_always_sum_to_the_floored_amount() {
        let w = map(&[(0, 0.3), (1, 0.7), (2, 0.1), (3, 1.0)]);
        for amount in 0..200 {
            let a = f64::from(amount) + 0.5;
            let sum = |v: Vec<(u32, f64)>| v.iter().map(|e| e.1).sum::<f64>();
            assert_eq!(sum(allocate_even(a, &[0, 1, 2], true)), a.floor());
            assert_eq!(
                sum(allocate_compensate(a, &[0, 1, 2, 3], &w, true)),
                a.floor()
            );
        }
    }

    #[test]
    fn plans_shortfall_counts_the_forecast_and_the_caches() {
        // Before its first winter: the worst case, no winter intake.
        assert_eq!(shortfall(2.0, 10, None, 0.0), 20.0);
        assert_eq!(shortfall(2.0, 10, None, 5.0), 15.0);
        assert_eq!(shortfall(2.0, 10, None, 25.0), -5.0);
        // Last winter it gathered 6 over 4 ticks: 1.5 a tick, 15 over 10.
        let r = WinterRecord {
            intake: 6.0,
            ticks: 4,
            sites: BTreeSet::new(),
        };
        assert_eq!(r.forecast(10), 15.0);
        assert_eq!(shortfall(2.0, 10, Some(&r), 0.0), 5.0);
        assert_eq!(shortfall(2.0, 10, Some(&r), 3.0), 2.0);
        assert_eq!(WinterRecord::default().forecast(10), 0.0);
    }

    fn known(entries: &[(u32, f64)]) -> Weights {
        let mut w = Weights::default();
        for &(site, weight) in entries {
            w.set(site, weight);
        }
        w
    }

    #[test]
    fn compensates_amount_scales_the_share_by_w_over_the_mean() {
        // Weights 1 and 0.5: mean 0.75.
        let w = known(&[(0, 1.0), (1, 0.5)]);
        // Site 1: 0.5 × 12 × 0.5 / 0.75 = 4.
        assert_eq!(compensate_amount(12.0, 0.5, &w, 1), 4.0);
        // Site 0: 0.5 × 12 × 4/3 = 8.
        assert_eq!(compensate_amount(12.0, 0.5, &w, 0), 8.0);
        // Capped at the surplus: share 1 at site 0 would bury 16.
        assert_eq!(compensate_amount(12.0, 1.0, &w, 0), 12.0);
        // All weights 0: w / w̄ counts as 1.
        let z = known(&[(0, 0.0), (1, 0.0)]);
        assert_eq!(compensate_amount(12.0, 0.5, &z, 0), 6.0);
        // λ = 1 zeroes each site it finds food at; once all are 0, the
        // mean is exactly 0 however the running sum rounded.
        let mut z = known(&[(0, 0.3), (1, 0.7), (2, 0.1)]);
        for site in 0..3 {
            z.harvested(site, true, 1.0);
        }
        assert_eq!(z.mean(), 0.0);
        assert_eq!(compensate_amount(12.0, 0.5, &z, 0), 6.0);
        assert_eq!(compensate_amount(0.0, 0.5, &w, 0), 0.0);
    }

    /// A walker at (5, 5) with metabolism 1, vision 1 and `held` sugar, under
    /// `rule` (R = 10 under GOAP's default horizon).
    fn field(rule: CachingRule, held: f64) -> (World, AgentId) {
        let mut w = blank_world(11, 11);
        w.config.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        w.config.caching.rule = rule;
        let id = spawn(&mut w, 5, 5);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.holdings[0] = held;
        (w, id)
    }

    fn turn(w: &mut World, id: AgentId) {
        w.events = crate::world::TickEvents::default();
        crate::rules::agent_turn(w, id);
    }

    #[test]
    fn under_mixed_each_agent_buries_by_its_own_rule() {
        // Four founders, dealt none, even, compensate, plan by id, each on
        // its own site with 4 sugar and holding 30; each buries what a world
        // of its rule alone buries (plan: nothing, seasons being off).
        let mut w = blank_world(11, 11);
        w.config.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        w.config.caching.rule = CachingRule::Even;
        w.config.caching.mixed = true;
        let ids: Vec<AgentId> = (0..4).map(|k| spawn(&mut w, 1 + 3 * k, 5)).collect();
        for (k, &id) in ids.iter().enumerate() {
            set_sugar(&mut w, 1 + 3 * k as u32, 5, 4.0);
            let a = w.agent_mut(id).unwrap();
            a.metabolism[0] = 1;
            a.holdings[0] = 30.0;
            assert_eq!(a.caching_rule, crate::config::Caching::MIXED[k]);
        }
        for &id in &ids {
            let rule = w.agent(id).unwrap().caching_rule;
            turn(&mut w, id);
            let (mut alone, one) = field(rule, 30.0);
            set_sugar(&mut alone, 5, 5, 4.0);
            turn(&mut alone, one);
            assert_eq!(w.events.buried, alone.events.buried, "{rule:?}");
            assert_eq!(
                w.agent(id).unwrap().holdings[0],
                alone.agent(one).unwrap().holdings[0],
                "{rule:?}"
            );
        }
    }

    #[test]
    fn cheaters_are_dealt_by_id_in_exact_proportion() {
        for (s, every) in [(0.25, 4), (0.5, 2), (1.0 / 3.0, 3)] {
            let mut w = blank_world(11, 11);
            w.config.caching.rule = CachingRule::Even;
            w.config.theft.cheaters = s;
            let ids: Vec<AgentId> = (0..24).map(|k| spawn(&mut w, k % 11, k / 11)).collect();
            for (n, &id) in ids.iter().enumerate() {
                let a = w.agent(id).unwrap();
                assert_eq!(a.cheater, id % every == 0, "s = {s}, id {id}");
                let cheats = ids[..=n]
                    .iter()
                    .filter(|&&i| w.agent(i).unwrap().cheater)
                    .count();
                assert_eq!(cheats, ((n + 1) as f64 * s).floor() as usize, "s = {s}");
                let rule = if a.cheater {
                    CachingRule::None
                } else {
                    CachingRule::Even
                };
                assert_eq!(rule_of(&w, id), rule);
            }
            // Over many ids the count is ⌊n·s⌋ at every n.
            let t = w.config.theft;
            let mut count = 0;
            for i in 1..=10_000u64 {
                count += u64::from(t.founder_cheats(i));
                assert_eq!(count, (i as f64 * s).floor() as u64, "s = {s}, n = {i}");
            }
        }
        // No cheaters: nobody is marked.
        let mut w = blank_world(11, 11);
        let id = spawn(&mut w, 1, 1);
        assert!(!w.agent(id).unwrap().cheater);
    }

    // Review Focus 3.
    #[test]
    fn a_cheater_under_mixed_buries_nothing_whatever_the_round_robin_dealt_it() {
        // s = 1/3 makes ids 3 and 6 cheaters. The round robin deals ids 2
        // and 6 `even`: 2 buries as `even`, 6 buries nothing.
        let mut w = blank_world(11, 11);
        w.config.caching.mixed = true;
        w.config.theft.cheaters = 1.0 / 3.0;
        let ids: Vec<AgentId> = (0..6).map(|k| spawn(&mut w, 1 + k, 5)).collect();
        for &id in &ids {
            let a = w.agent_mut(id).unwrap();
            a.metabolism[0] = 1;
            a.holdings[0] = 30.0;
        }
        let (honest, cheater) = (ids[1], ids[5]);
        for id in [honest, cheater] {
            assert_eq!(w.agent(id).unwrap().caching_rule, CachingRule::Even);
        }
        assert!(w.agent(cheater).unwrap().cheater);
        assert!(!w.agent(honest).unwrap().cheater);
        assert_eq!(rule_of(&w, honest), CachingRule::Even);
        assert_eq!(rule_of(&w, cheater), CachingRule::None);
        assert_eq!(rule_of(&w, ids[2]), CachingRule::None, "dealt compensate");
        turn(&mut w, honest);
        assert!(w.events.buried > 0.0);
        turn(&mut w, cheater);
        assert_eq!(w.events.buried, 0.0);
        assert!(w.agent(cheater).unwrap().caches.is_empty());
        // A cheater still digs a cache it has (it never makes one itself).
        let here = w.torus.index(w.agent(cheater).unwrap().pos) as u32;
        let a = w.agent_mut(cheater).unwrap();
        a.holdings[0] = 0.0;
        a.caches.insert(here, 4.0);
        turn(&mut w, cheater);
        assert_eq!(w.events.dug, 4.0);
    }

    #[test]
    fn even_buries_its_share_of_the_surplus_where_it_stands() {
        let (mut w, id) = field(CachingRule::Even, 30.0);
        set_sugar(&mut w, 5, 5, 4.0);
        turn(&mut w, id);
        // Harvests 4 (34), surplus 24, buries 12, then eats 1.
        let here = w.torus.index(Pos::new(5, 5)) as u32;
        assert_eq!(w.events.buried, 12.0);
        assert_eq!(w.agent(id).unwrap().caches[&here], 12.0);
        assert_eq!(w.agent(id).unwrap().holdings[0], 21.0);
        // At the reserve it buries nothing.
        w.agent_mut(id).unwrap().holdings[0] = 10.0;
        turn(&mut w, id);
        assert_eq!(w.events.buried, 0.0);
    }

    #[test]
    fn compensates_weights_fall_where_it_finds_food_and_stay_where_it_doesnt() {
        let (mut w, id) = field(CachingRule::Compensate, 30.0);
        w.config.caching.lambda = 0.5;
        let here = w.torus.index(Pos::new(5, 5)) as u32;
        set_sugar(&mut w, 5, 5, 2.0);
        turn(&mut w, id);
        assert_eq!(
            w.agent(id).unwrap().weights.get(here),
            Some(0.5),
            "found food"
        );
        // Its only known site: w / w̄ = 1, so it buries share × surplus:
        // 0.5 × (32 − 10) = 11.
        assert_eq!(w.events.buried, 11.0);
        set_sugar(&mut w, 5, 5, 1.0);
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().weights.get(here), Some(0.25));
        // Nothing here now, and nothing in sight: no food, no decay.
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().weights.get(here), Some(0.25));
        // A second known site at weight 1 lowers this one's ratio: mean
        // 0.625, w / w̄ = 0.4.
        let other = w.torus.index(Pos::new(0, 0)) as u32;
        w.agent_mut(id).unwrap().weights.set(other, 1.0);
        w.agent_mut(id).unwrap().holdings[0] = 30.0;
        turn(&mut w, id);
        assert!((w.events.buried - 0.5 * 20.0 * 0.4).abs() < 1e-12);
    }

    #[test]
    fn a_dig_tick_harvests_no_site_and_leaves_the_known_sites_alone() {
        let (mut w, id) = field(CachingRule::Compensate, 2.0);
        let here = w.torus.index(Pos::new(5, 5)) as u32;
        set_sugar(&mut w, 5, 5, 1.0);
        w.agent_mut(id).unwrap().caches.insert(here, 5.0);
        turn(&mut w, id);
        assert_eq!(w.events.dug, 5.0);
        assert_eq!(
            w.agent(id).unwrap().weights.get(here),
            None,
            "a dig site doesn't become known"
        );
        // A known site it digs at keeps its weight too.
        w.agent_mut(id).unwrap().weights.set(here, 0.5);
        w.agent_mut(id).unwrap().holdings[0] = 2.0;
        w.agent_mut(id).unwrap().caches.insert(here, 5.0);
        turn(&mut w, id);
        assert_eq!(w.events.dug, 5.0);
        assert_eq!(w.agent(id).unwrap().weights.get(here), Some(0.5));
    }

    #[test]
    fn known_sites_are_capped_dropping_the_lowest_other_site() {
        let mut w = Weights::default();
        for site in 0..MEMORY_CAP as u32 {
            w.set(site + 10, 1.0);
        }
        assert_eq!(w.len(), MEMORY_CAP);
        // Updating a known site drops nothing.
        w.harvested(10, true, 0.5);
        assert_eq!((w.len(), w.get(10)), (MEMORY_CAP, Some(0.5)));
        // A new site drops the lowest index (10), even below the new one.
        w.harvested(3, false, 0.5);
        assert_eq!(w.len(), MEMORY_CAP);
        assert_eq!((w.get(10), w.get(3)), (None, Some(1.0)));
        // Then 3 is the lowest, dropped for the next newcomer.
        w.harvested(100_000, false, 0.5);
        assert_eq!((w.get(3), w.get(11)), (None, Some(1.0)));
        assert_eq!(w.mean(), 1.0);
    }

    #[test]
    fn the_running_sum_keeps_the_mean_of_the_map() {
        let mut w = Weights::default();
        for i in 0..5000u32 {
            let site = (i * 7919) % 300;
            w.harvested(site, i % 3 != 0, 0.3);
            let fresh = w.map().values().sum::<f64>() / w.len() as f64;
            assert!(
                (w.mean() - fresh).abs() <= 1e-12 * fresh.max(1e-300),
                "{i}: {} vs {fresh}",
                w.mean()
            );
        }
    }

    #[test]
    fn whole_units_never_lose_a_unit_to_non_dyadic_weights() {
        // 1/3 : 2/3 : 1 of 30 is exactly 5, 10, 15.
        let w = map(&[(0, 1.0 / 3.0), (1, 2.0 / 3.0), (2, 1.0)]);
        assert_eq!(
            allocate_compensate(30.0, &[0, 1, 2], &w, true),
            vec![(0, 5.0), (1, 10.0), (2, 15.0)]
        );
        // 0.1 : 0.2 : 0.3 of 60 is exactly 10, 20, 30 (0.1 + 0.2 ≠ 0.3 in
        // binary, but the shares are whole).
        let w = map(&[(0, 0.1), (1, 0.2), (2, 0.3)]);
        assert_eq!(
            allocate_compensate(60.0, &[0, 1, 2], &w, true),
            vec![(0, 10.0), (1, 20.0), (2, 30.0)]
        );
        // Thirds of 1/3 each: 10 apiece.
        let third = 1.0 / 3.0;
        let w = map(&[(0, third), (1, third), (2, third)]);
        assert_eq!(
            allocate_compensate(30.0, &[0, 1, 2], &w, true),
            vec![(0, 10.0), (1, 10.0), (2, 10.0)]
        );
        // (1 − 0.3)ⁿ weights, as compensate makes them: 0.7 : 0.49 of 119
        // is 70, 49.
        let w = map(&[(0, 0.7), (1, 0.7 * 0.7)]);
        assert_eq!(
            allocate_compensate(119.0, &[0, 1], &w, true),
            vec![(0, 70.0), (1, 49.0)]
        );
        // A true fraction still floors: 1 : 2 of 10 is 3.33, 6.67 → 3, 6,
        // and the unit left goes to the first place.
        let w = map(&[(0, 1.0), (1, 2.0)]);
        assert_eq!(
            allocate_compensate(10.0, &[0, 1], &w, true),
            vec![(0, 4.0), (1, 6.0)]
        );
    }
    fn global_winter(w: &mut World, period: u32) {
        w.config.seasons.enabled = true;
        w.config.seasons.mode = SeasonMode::Global;
        w.config.seasons.period = period;
        w.config.seasons.winter_divisor = 1000;
    }

    #[test]
    fn plan_buries_nothing_without_a_global_winter() {
        for (enabled, mode) in [(false, SeasonMode::Global), (true, SeasonMode::Hemispheres)] {
            let (mut w, id) = field(CachingRule::Plan, 40.0);
            w.config.seasons.enabled = enabled;
            w.config.seasons.mode = mode;
            for _ in 0..5 {
                turn(&mut w, id);
                assert_eq!(w.events.buried, 0.0);
                w.tick += 1;
            }
            assert!(w.agent(id).unwrap().caches.is_empty());
            assert_eq!(w.agent(id).unwrap().last_winter, None);
        }
    }

    // A small global-winter world: plan buries toward the worst case before
    // its first winter, stops once its caches cover the shortfall, records
    // the winter, and then plans from what that winter gave.
    #[test]
    fn plan_stops_burying_once_its_caches_cover_the_shortfall() {
        let (mut w, id) = field(CachingRule::Plan, 40.0);
        global_winter(&mut w, 10);
        let here = w.torus.index(Pos::new(5, 5)) as u32;
        // Tick 0, summer, no winter yet: shortfall 1 × 10 − 0 − 0 = 10.
        turn(&mut w, id);
        assert_eq!(w.events.buried, 10.0);
        assert_eq!(w.agent(id).unwrap().caches[&here], 10.0);
        // Covered: nothing more for the rest of summer.
        for t in 1..10 {
            w.tick = t;
            turn(&mut w, id);
            assert_eq!(w.events.buried, 0.0, "tick {t}");
        }
        assert_eq!(w.agent(id).unwrap().caches[&here], 10.0);
        // Winter (ticks 10–19): it records, buries nothing. Give it 2 sugar
        // underfoot on two winter ticks (holdings stay above R: no digging).
        w.agent_mut(id).unwrap().holdings[0] = 40.0;
        for t in 10..20 {
            w.tick = t;
            if t == 12 || t == 15 {
                set_sugar(&mut w, 5, 5, 2.0);
            }
            turn(&mut w, id);
            assert_eq!(w.events.buried, 0.0, "tick {t}");
        }
        let a = w.agent(id).unwrap();
        assert_eq!(a.last_winter, None, "still being recorded");
        assert_eq!(a.this_winter.ticks, 10);
        assert_eq!(a.this_winter.intake, 4.0);
        assert_eq!(a.this_winter.sites, BTreeSet::from([here]));
        // Summer again: last winter closes. Forecast 4 / 10 × 10 = 4, so the
        // shortfall is 10 − 4 − 10 < 0; empty the cache and it's 6.
        w.agent_mut(id).unwrap().caches.clear();
        w.tick = 20;
        turn(&mut w, id);
        let a = w.agent(id).unwrap();
        assert_eq!(a.last_winter.as_ref().unwrap().intake, 4.0);
        assert_eq!(a.this_winter, WinterRecord::default());
        assert_eq!(w.events.buried, 6.0);
        // Off its winter range it doesn't bury.
        w.agent_mut(id).unwrap().caches.clear();
        w.move_agent(id, Pos::new(1, 1));
        w.tick = 21;
        turn(&mut w, id);
        assert_eq!(w.events.buried, 0.0, "not a site it wintered on");
    }

    #[test]
    fn the_first_winter_worst_case_is_capped_by_the_surplus() {
        let (mut w, id) = field(CachingRule::Plan, 14.0);
        global_winter(&mut w, 10);
        turn(&mut w, id);
        // Shortfall 10, surplus 4: it buries 4 and keeps its reserve.
        assert_eq!(w.events.buried, 4.0);
        assert_eq!(w.agent(id).unwrap().holdings[0], 9.0, "10 − 1 eaten");
    }
}
