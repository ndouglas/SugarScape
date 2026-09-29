//! Minds 5: the caching rules — when an agent buries, how much, and where.
//! Digging is the same for every rule (`super::dig`); these only bury.
//!
//! - **Allocation** ([`allocate_even`], [`allocate_compensate`],
//!   [`allocate_plan`]): pure splits of an amount over places, shared by the
//!   field and the lab. Places come back in ascending order. With `whole`
//!   (the lab), each share is floored to whole units and the remainder goes
//!   one unit at a time to the places in ascending order (K1 < K2 < K3); the
//!   field passes `whole = false`.
//! - **The field** ([`act`]), after each agent's move and harvest, before it
//!   eats:
//!   - `even` buries `share` × surplus at its current site, every tick.
//!   - `compensate` first updates the weight of the site it just harvested
//!     (w starts at 1; w ← w × (1 − λ) when it found food, i.e. gathered
//!     good 0 > 0 from the site — a dig isn't finding food), then buries
//!     min(surplus, share × surplus × w / w̄) there, w̄ the mean weight over
//!     the sites it has harvested. If every weight has decayed to exactly 0
//!     (w̄ = 0), w / w̄ counts as 1: the sites are equal again.
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
//! Nothing here draws.

use std::collections::{BTreeMap, BTreeSet};

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
pub fn compensate_amount(surplus: f64, share: f64, weights: &BTreeMap<u32, f64>, site: u32) -> f64 {
    let w = weights.get(&site).copied().unwrap_or(1.0);
    let mean = if weights.is_empty() {
        1.0
    } else {
        weights.values().sum::<f64>() / weights.len() as f64
    };
    let ratio = if mean > 0.0 { w / mean } else { 1.0 };
    (share * surplus * ratio).min(surplus).max(0.0)
}

/// Splits `amount` over `places` (sorted ascending, duplicates dropped) by
/// `weight`, each place's share its weight over the total (equal shares if
/// the total isn't positive). With `whole`, the amount is floored to whole
/// units, each share floored, and the remainder handed out one unit at a
/// time in ascending place order.
fn apportion(
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
    let amount = if whole { amount.floor() } else { amount }.max(0.0);
    let weights: Vec<f64> = places.iter().map(|&p| weight(p).max(0.0)).collect();
    let total: f64 = weights.iter().sum();
    let mut out: Vec<(u32, f64)> = places
        .iter()
        .zip(&weights)
        .map(|(&p, &w)| {
            let q = if total > 0.0 {
                amount * w / total
            } else {
                amount / k as f64
            };
            (p, if whole { q.floor() } else { q })
        })
        .collect();
    if whole {
        let given: f64 = out.iter().map(|e| e.1).sum();
        let left = (amount - given).max(0.0) as usize;
        for i in 0..left {
            out[i % k].1 += 1.0;
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

/// The field's caching step for `id`, after its move and `harvest` and
/// before it eats. Does nothing under rule `none`.
pub(crate) fn act(world: &mut World, id: AgentId, harvest: &Harvest) {
    let caching = world.config.caching;
    let here = {
        let a = world.agent(id).expect("live agent");
        world.torus.index(a.pos) as u32
    };
    match caching.rule {
        CachingRule::None => {}
        CachingRule::Even => {
            let amount = caching.share * surplus(world, id);
            bury_all(world, id, allocate_even(amount, &[here], false));
        }
        CachingRule::Compensate => {
            let found = harvest.gathered[0] > 0.0;
            let a = world.agent_mut(id).expect("live agent");
            let w = a.weights.entry(here).or_insert(1.0);
            if found {
                *w *= 1.0 - caching.lambda;
            }
            let s = surplus(world, id);
            let weights = &world.agent(id).expect("live agent").weights;
            let amount = compensate_amount(s, caching.share, weights, here);
            let allocation = allocate_compensate(amount, &[here], weights, false);
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

    #[test]
    fn compensates_amount_scales_the_share_by_w_over_the_mean() {
        // Weights 1 and 0.5: mean 0.75.
        let w = map(&[(0, 1.0), (1, 0.5)]);
        // Site 1: 0.5 × 12 × 0.5 / 0.75 = 4.
        assert_eq!(compensate_amount(12.0, 0.5, &w, 1), 4.0);
        // Site 0: 0.5 × 12 × 4/3 = 8.
        assert_eq!(compensate_amount(12.0, 0.5, &w, 0), 8.0);
        // Capped at the surplus: share 1 at site 0 would bury 16.
        assert_eq!(compensate_amount(12.0, 1.0, &w, 0), 12.0);
        // All weights 0: w / w̄ counts as 1.
        let z = map(&[(0, 0.0), (1, 0.0)]);
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
        assert_eq!(w.agent(id).unwrap().weights[&here], 0.5, "found food");
        // Its only known site: w / w̄ = 1, so it buries share × surplus:
        // 0.5 × (32 − 10) = 11.
        assert_eq!(w.events.buried, 11.0);
        set_sugar(&mut w, 5, 5, 1.0);
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().weights[&here], 0.25);
        // Nothing here now, and nothing in sight: no food, no decay.
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().weights[&here], 0.25);
        // A second known site at weight 1 lowers this one's ratio: mean
        // 0.625, w / w̄ = 0.4.
        let other = w.torus.index(Pos::new(0, 0)) as u32;
        w.agent_mut(id).unwrap().weights.insert(other, 1.0);
        w.agent_mut(id).unwrap().holdings[0] = 30.0;
        turn(&mut w, id);
        assert!((w.events.buried - 0.5 * 20.0 * 0.4).abs() < 1e-12);
    }

    #[test]
    fn a_dig_is_not_finding_food() {
        let (mut w, id) = field(CachingRule::Compensate, 2.0);
        let here = w.torus.index(Pos::new(5, 5)) as u32;
        w.agent_mut(id).unwrap().caches.insert(here, 5.0);
        turn(&mut w, id);
        assert_eq!(w.events.dug, 5.0);
        assert_eq!(w.agent(id).unwrap().weights[&here], 1.0);
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
