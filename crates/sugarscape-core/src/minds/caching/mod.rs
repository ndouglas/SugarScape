//! Minds 5's caching mechanics: the carrying limit, burying and digging.
//!
//! - **Caches.** Each agent owns `caches`, site index → sugar buried there
//!   (a `BTreeMap`, so every walk over them is in site index order). Nobody
//!   else sees or takes them, they don't decay, and they die with the agent
//!   (counted into `events.cache_lost` by `World::remove`).
//! - **Bury(q)** (`bury`) at the agent's current site: holdings −= q, cache
//!   += q. It costs no tick.
//! - **Dig** (`dig`, called by `movement::go_and_gather`): arriving at its own
//!   cache while holdings are below the reserve, the agent takes min(cache,
//!   room under the carrying limit) instead of harvesting the site.
//! - **Caches as candidates** (`join_caches`, called by
//!   `movement::candidates_with_memory`, which rule M, the utility mind,
//!   GOAP and the marginal-value rule all build on): while holdings are
//!   below the reserve, each cache joins the candidates at its amount.
//! - **Reserve.** R = good-0 effective metabolism × `goap.horizon` (GOAP's
//!   goal G: H ticks of food); 0 in the lab. Surplus is holdings above R.
//!
//! Sugar is conserved exactly across bury and dig: Σ sites + Σ holdings +
//! Σ caches + eaten. Nothing here draws.

use crate::agent::AgentId;
use crate::geometry::Pos;
use crate::rules::movement::lattice_distance;
use crate::world::World;

/// R: the sugar an agent keeps on hand, its good-0 burn per tick (effective
/// metabolism, disease fees included, as GOAP's G) × `goap.horizon`. In the
/// lab the reserve is 0.
pub(crate) fn reserve(world: &World, id: AgentId) -> f64 {
    if world.config.lab.is_some() {
        return 0.0;
    }
    let a = world.agent(id).expect("live agent");
    let burn = a.effective_metabolism(0, world.config.disease.active_fee());
    burn * f64::from(world.config.goap.horizon)
}

/// max(0, holdings − R): what an agent may bury.
#[allow(dead_code)] // The caching rules call it.
pub(crate) fn surplus(world: &World, id: AgentId) -> f64 {
    let held = world.agent(id).expect("live agent").holdings[0];
    (held - reserve(world, id)).max(0.0)
}

/// Whether `id` digs rather than forages: it has caches and holds less than
/// its reserve. False, without computing R, for an agent with no caches.
pub(crate) fn hungry(world: &World, id: AgentId) -> bool {
    let a = world.agent(id).expect("live agent");
    !a.caches.is_empty() && a.holdings[0] < reserve(world, id)
}

/// Buries `q` sugar (clamped to [0, holdings]) at the agent's current site.
/// Adds to `events.buried`; a cache begun on an empty site starts its age
/// now.
#[allow(dead_code)] // The caching rules call it; so do the tests.
pub(crate) fn bury(world: &mut World, id: AgentId, q: f64) {
    let now = world.tick;
    let torus = world.torus;
    let a = world.agent_mut(id).expect("live agent");
    let q = q.min(a.holdings[0]).max(0.0);
    if q <= 0.0 {
        return;
    }
    let site = torus.index(a.pos) as u32;
    a.holdings[0] -= q;
    *a.caches.entry(site).or_insert(0.0) += q;
    a.cache_since.entry(site).or_insert(now);
    world.events.buried += q;
}

/// Digs `id`'s cache at site index `site`, taking min(cache, `room`), and
/// returns what it took (0 when there's no cache there). An emptied cache is
/// removed. A positive dig counts `dug`, `digs` and the cache's age. The
/// caller adds the take to holdings, as the tick's harvest.
pub(crate) fn dig(world: &mut World, id: AgentId, site: u32, room: f64) -> f64 {
    let now = world.tick;
    let a = world.agent_mut(id).expect("live agent");
    let Some(cache) = a.caches.get_mut(&site) else {
        return 0.0;
    };
    let take = cache.min(room).max(0.0);
    if take <= 0.0 {
        return 0.0;
    }
    let since = a.cache_since.get(&site).copied().unwrap_or(now);
    if take >= *cache {
        a.caches.remove(&site);
        a.cache_since.remove(&site);
    } else {
        *cache -= take;
    }
    let e = &mut world.events;
    e.dug += take;
    e.digs += 1;
    e.dig_ages_sum += now.saturating_sub(since);
    take
}

/// While `id` is hungry (has caches and holds less than its reserve), adds
/// each cache, in site index order, to rule M's candidates at its amount.
///
/// - A cache already listed (its own site, a free site in sight, or a
///   remembered site) keeps its place, valued at the larger of the two.
/// - Otherwise it joins at its torus Manhattan distance, inserted at
///   `start` (after the sight list, before the remembered entries, so
///   Minds 3's diagnostics don't count it as a remembered choice), and
///   `start` moves past it.
/// - Skipped (the agent falls back to its rule's ordinary choice): a cache
///   walled apart from it, one another agent stands on (the agent knows
///   where its cache is taken, in sight or not, so no mode ever targets an
///   occupied site), and the target its last walk found no path to.
///
/// No caches, or not hungry: the list is untouched and nothing allocates.
pub(crate) fn join_caches(
    world: &World,
    id: AgentId,
    out: &mut Vec<(Pos, u32, f64)>,
    start: &mut usize,
) {
    if !hungry(world, id) {
        return;
    }
    let a = world.agent(id).expect("live agent");
    let torus = world.torus;
    let pos = a.pos;
    let failed = a
        .plan
        .target
        .filter(|&t| a.plan.path.is_empty() && t != pos);
    let mut extra = Vec::new();
    for (&i, &amount) in &a.caches {
        let q = torus.pos(i as usize);
        if let Some(c) = out.iter_mut().find(|c| c.0 == q) {
            c.2 = c.2.max(amount);
            continue;
        }
        if world.is_wall(q)
            || world.walled_apart(pos, q)
            || world.occupant(q).is_some_and(|o| o != id)
            || Some(q) == failed
        {
            continue;
        }
        extra.push((q, lattice_distance(torus, pos, q), amount));
    }
    let k = extra.len();
    out.splice(*start..*start, extra);
    *start += k;
}

/// `id`'s cache at `p`, when it's hungry (Minds 3's true value of a
/// candidate counts it, as the candidate did).
pub(crate) fn cache_value(world: &World, id: AgentId, p: Pos) -> Option<f64> {
    let a = world.agent(id).expect("live agent");
    if a.caches.is_empty() {
        return None;
    }
    let amount = *a.caches.get(&(world.torus.index(p) as u32))?;
    hungry(world, id).then_some(amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MoveMode, Movement};
    use crate::rules::movement::{candidates, go_and_gather};
    use crate::testkit::*;

    /// An agent at (5, 5) with metabolism 1, `held` sugar and vision 1, under
    /// capacity `capacity` and GOAP's default horizon (R = 10).
    fn caching_agent(w: &mut World, held: f64, capacity: u32) -> AgentId {
        w.config.caching.capacity = capacity;
        let id = spawn(w, 5, 5);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.holdings[0] = held;
        id
    }

    fn sites_sum(w: &World) -> f64 {
        w.sites.iter().map(|s| s.resource[0]).sum()
    }

    fn held_sum(w: &World) -> f64 {
        w.agents().map(|a| a.holdings[0]).sum()
    }

    fn cached_sum(w: &World) -> f64 {
        w.agents().flat_map(|a| a.caches.values()).sum()
    }

    fn total(w: &World) -> f64 {
        sites_sum(w) + held_sum(w) + cached_sum(w)
    }

    fn at(w: &World, x: u32, y: u32) -> u32 {
        w.torus.index(Pos::new(x, y)) as u32
    }

    #[test]
    fn reserve_is_metabolism_times_horizon_and_zero_in_the_lab() {
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 14.0, 0);
        w.config.goap.horizon = 4;
        w.agent_mut(id).unwrap().metabolism[0] = 3;
        assert_eq!(reserve(&w, id), 12.0);
        assert_eq!(surplus(&w, id), 2.0);
        w.agent_mut(id).unwrap().holdings[0] = 5.0;
        assert_eq!(surplus(&w, id), 0.0);
        w.config.lab = Some(crate::config::Lab {
            protocol: crate::config::LabProtocol::Raby,
            food_first: false,
        });
        assert_eq!(reserve(&w, id), 0.0);
        assert_eq!(surplus(&w, id), 5.0);
    }

    // Review Focus 1.
    #[test]
    fn at_the_carrying_limit_an_agent_takes_only_its_room_and_leaves_the_rest() {
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 7.0, 10);
        set_sugar(&mut w, 5, 6, 20.0);
        let before = total(&w);
        let h = go_and_gather(&mut w, id, Pos::new(5, 6));
        assert_eq!(h.gathered[0], 3.0, "room is 10 − 7");
        assert_eq!(w.agent(id).unwrap().holdings[0], 10.0);
        assert_eq!(w.site(Pos::new(5, 6)).resource[0], 17.0, "the rest stays");
        assert_eq!(total(&w), before, "nothing lost or created");
        // Full: it takes nothing, and the site keeps everything.
        let h = go_and_gather(&mut w, id, Pos::new(5, 6));
        assert_eq!(h.gathered[0], 0.0);
        assert_eq!(w.site(Pos::new(5, 6)).resource[0], 17.0);
        assert_eq!(total(&w), before);
        // Over the limit (it started there): room is 0, never negative.
        w.agent_mut(id).unwrap().holdings[0] = 12.0;
        let h = go_and_gather(&mut w, id, Pos::new(5, 6));
        assert_eq!(h.gathered[0], 0.0);
        assert_eq!(w.agent(id).unwrap().holdings[0], 12.0);
        assert_eq!(w.site(Pos::new(5, 6)).resource[0], 17.0);
    }

    #[test]
    fn a_site_that_fits_is_taken_whole_and_no_limit_takes_everything() {
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 7.0, 10);
        set_sugar(&mut w, 5, 6, 2.0);
        assert_eq!(go_and_gather(&mut w, id, Pos::new(5, 6)).gathered[0], 2.0);
        assert_eq!(w.site(Pos::new(5, 6)).resource[0], 0.0);
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 7.0, 0);
        set_sugar(&mut w, 5, 6, 20.0);
        assert_eq!(go_and_gather(&mut w, id, Pos::new(5, 6)).gathered[0], 20.0);
        assert_eq!(w.agent(id).unwrap().holdings[0], 27.0);
    }

    #[test]
    fn a_truffle_is_capped_by_the_room_left_and_its_excess_is_lost_with_the_spot() {
        let mut c = blank_config(11, 11);
        c.truffles.share = 1.0;
        c.truffles.value = 5.0;
        c.truffles.regrow = 3;
        let mut w = World::new(c, 7).unwrap();
        let id = caching_agent(&mut w, 7.0, 10);
        set_sugar(&mut w, 5, 6, 1.0);
        w.tick = 10;
        let h = go_and_gather(&mut w, id, Pos::new(5, 6));
        assert_eq!(h.gathered[0], 3.0, "1 of sugar, then 2 of the truffle's 5");
        assert_eq!(w.agent(id).unwrap().holdings[0], 10.0);
        assert_eq!(w.site(Pos::new(5, 6)).resource[0], 0.0);
        assert_eq!(
            w.truffle(Pos::new(5, 6)),
            Some(false),
            "picked all the same"
        );
    }

    #[test]
    fn bury_clamps_to_holdings_and_then_digging_conserves_exactly() {
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 14.0, 0);
        set_sugar(&mut w, 5, 5, 3.0);
        let before = total(&w);
        w.tick = 4;
        bury(&mut w, id, 6.0);
        bury(&mut w, id, -1.0);
        let here = at(&w, 5, 5);
        assert_eq!(w.agent(id).unwrap().caches[&here], 6.0);
        assert_eq!(w.agent(id).unwrap().cache_since[&here], 4);
        assert_eq!(w.agent(id).unwrap().holdings[0], 8.0);
        assert_eq!(w.events.buried, 6.0);
        assert_eq!(total(&w), before);
        // Burying more than it holds buries what it holds.
        w.tick = 6;
        w.agent_mut(id).unwrap().holdings[0] = 2.0;
        bury(&mut w, id, 50.0);
        assert_eq!(w.agent(id).unwrap().caches[&here], 8.0);
        assert_eq!(
            w.agent(id).unwrap().cache_since[&here],
            4,
            "not empty since"
        );
        assert_eq!(w.agent(id).unwrap().holdings[0], 0.0);
        let before = total(&w);
        // Holdings 0 < R = 10: it digs instead of harvesting the site.
        w.tick = 9;
        let h = go_and_gather(&mut w, id, Pos::new(5, 5));
        assert_eq!(h.gathered[0], 8.0);
        assert_eq!(w.agent(id).unwrap().holdings[0], 8.0);
        assert!(w.agent(id).unwrap().caches.is_empty());
        assert!(w.agent(id).unwrap().cache_since.is_empty());
        assert_eq!(
            w.site(Pos::new(5, 5)).resource[0],
            3.0,
            "the site untouched"
        );
        assert_eq!((w.events.dug, w.events.digs), (8.0, 1));
        assert_eq!(w.events.dig_ages_sum, 5);
        assert_eq!(total(&w), before);
    }

    #[test]
    fn a_dig_takes_only_the_room_and_a_fed_agent_harvests_instead() {
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 9.0, 12);
        let here = at(&w, 5, 5);
        set_sugar(&mut w, 5, 5, 1.0);
        w.agent_mut(id).unwrap().caches.insert(here, 5.0);
        w.agent_mut(id).unwrap().cache_since.insert(here, 0);
        let before = total(&w);
        let h = go_and_gather(&mut w, id, Pos::new(5, 5));
        assert_eq!(h.gathered[0], 3.0, "room is 12 − 9");
        assert_eq!(w.agent(id).unwrap().caches[&here], 2.0);
        assert_eq!(w.agent(id).unwrap().cache_since[&here], 0);
        assert_eq!(total(&w), before);
        // At 12 ≥ R = 10 it isn't hungry: it harvests the site (room 0).
        let h = go_and_gather(&mut w, id, Pos::new(5, 5));
        assert_eq!(h.gathered[0], 0.0);
        assert_eq!(w.agent(id).unwrap().caches[&here], 2.0);
        assert_eq!(w.events.digs, 1);
    }

    fn walker(w: &mut World) {
        w.config.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
    }

    #[test]
    fn a_hungry_agent_walks_to_its_cache_and_digs_it() {
        let mut w = blank_world(15, 15);
        walker(&mut w);
        let id = caching_agent(&mut w, 4.0, 0);
        let cache = at(&w, 9, 5);
        w.agent_mut(id).unwrap().caches.insert(cache, 7.0);
        w.agent_mut(id).unwrap().cache_since.insert(cache, 0);
        let c = candidates(&w, id);
        assert_eq!(*c.last().unwrap(), (Pos::new(9, 5), 4, 7.0));
        for _ in 0..4 {
            crate::minds::decide(&mut w, id);
        }
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(9, 5));
        assert_eq!(w.agent(id).unwrap().holdings[0], 11.0);
        assert!(w.agent(id).unwrap().caches.is_empty());
        assert_eq!(w.events.digs, 1);
    }

    #[test]
    fn a_fed_agent_ignores_its_caches_and_without_caches_the_list_is_rule_ms() {
        let mut w = blank_world(15, 15);
        let id = caching_agent(&mut w, 10.0, 0);
        let plain = candidates(&w, id);
        let cache = at(&w, 9, 5);
        w.agent_mut(id).unwrap().caches.insert(cache, 7.0);
        assert_eq!(candidates(&w, id), plain, "10 is not below R = 10");
        w.agent_mut(id).unwrap().holdings[0] = 9.0;
        assert_eq!(candidates(&w, id).len(), plain.len() + 1);
    }

    #[test]
    fn a_cache_in_sight_or_underfoot_merges_by_the_larger_value() {
        let mut w = blank_world(15, 15);
        let id = caching_agent(&mut w, 1.0, 0);
        set_sugar(&mut w, 5, 6, 4.0);
        set_sugar(&mut w, 6, 5, 1.0);
        let plain = candidates(&w, id);
        for (x, y, v) in [(5, 5, 2.0), (5, 6, 3.0), (6, 5, 3.0)] {
            let i = at(&w, x, y);
            w.agent_mut(id).unwrap().caches.insert(i, v);
        }
        let c = candidates(&w, id);
        assert_eq!(c.len(), plain.len(), "no new entries");
        let value = |x, y| c.iter().find(|e| e.0 == Pos::new(x, y)).unwrap().2;
        assert_eq!(value(5, 5), 2.0);
        assert_eq!(value(5, 6), 4.0, "the site's own 4 beats the cache's 3");
        assert_eq!(value(6, 5), 3.0);
    }

    // Review Focus 2.
    #[test]
    fn a_walled_off_or_occupied_cache_is_ignored_and_the_agent_forages_as_usual() {
        let mut c = blank_config(21, 21);
        c.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        c.walls = vec![crate::config::Wall {
            x: 10,
            y: 0,
            width: 1,
            height: 21,
            opaque: false,
        }];
        c.walls.push(crate::config::Wall {
            x: 0,
            y: 0,
            width: 1,
            height: 21,
            opaque: false,
        });
        let mut w = World::new(c, 7).unwrap();
        let id = caching_agent(&mut w, 2.0, 0);
        assert!(w.walled_apart(Pos::new(5, 5), Pos::new(15, 5)));
        let plain = candidates(&w, id);
        let walled = at(&w, 15, 5);
        let taken = at(&w, 5, 9);
        w.agent_mut(id).unwrap().caches.insert(walled, 50.0);
        w.agent_mut(id).unwrap().caches.insert(taken, 50.0);
        spawn(&mut w, 5, 9);
        assert_eq!(candidates(&w, id), plain);
        // It takes its ordinary choice, never stalling on the caches.
        set_sugar(&mut w, 5, 6, 1.0);
        for _ in 0..5 {
            crate::minds::decide(&mut w, id);
            let pos = w.agent(id).unwrap().pos;
            assert_ne!(pos, Pos::new(15, 5));
            assert_ne!(pos, Pos::new(5, 9));
        }
        assert_eq!(w.agent(id).unwrap().holdings[0], 3.0, "found the 1 nearby");
        assert_eq!(w.events.digs, 0);
        // A cache whose last walk found no path is skipped too.
        let far = at(&w, 5, 1);
        let mut w2 = w.clone();
        w2.agent_mut(id).unwrap().caches.insert(far, 50.0);
        w2.agent_mut(id).unwrap().plan = crate::agent::Plan {
            target: Some(Pos::new(5, 1)),
            path: Vec::new(),
            walked: true,
        };
        assert!(!candidates(&w2, id).iter().any(|c| c.0 == Pos::new(5, 1)));
    }

    // Review Focus 3.
    #[test]
    fn a_dying_agent_takes_its_caches_with_it_counted() {
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 6.0, 0);
        bury(&mut w, id, 4.0);
        let other = at(&w, 1, 1);
        w.agent_mut(id).unwrap().caches.insert(other, 1.5);
        let before = total(&w);
        w.agent_mut(id).unwrap().holdings[0] = 0.0;
        let held = 2.0;
        assert!(crate::rules::lifecycle::check_death(&mut w, id));
        assert!(w.agent(id).is_none());
        assert_eq!(w.events.cache_lost, 5.5);
        assert_eq!(total(&w) + w.events.cache_lost + held, before);
    }

    #[test]
    fn caches_are_hashed_only_when_there_are_any() {
        let mut w = blank_world(11, 11);
        let id = caching_agent(&mut w, 6.0, 0);
        let plain = w.fingerprint();
        let here = at(&w, 5, 5);
        w.agent_mut(id).unwrap().cache_since.insert(here, 0);
        assert_eq!(w.fingerprint(), plain, "cache_since alone isn't hashed");
        w.agent_mut(id).unwrap().caches.insert(here, 1.0);
        let one = w.fingerprint();
        assert_ne!(one, plain);
        w.agent_mut(id).unwrap().caches.insert(here, 2.0);
        assert_ne!(w.fingerprint(), one);
    }

    #[test]
    fn sugar_is_conserved_over_300_ticks_of_scripted_burying() {
        let mut c = blank_config(12, 12);
        c.growback.rate = 1.0;
        c.goap.horizon = 3;
        c.caching.capacity = 8;
        let mut w = World::new(c, 11).unwrap();
        for y in 0..12 {
            for x in 0..12 {
                set_sugar(&mut w, x, y, if (x + y) % 3 == 0 { 4.0 } else { 1.0 });
            }
        }
        let mut ids = Vec::new();
        for (x, y) in [(1, 1), (6, 2), (3, 8), (9, 9), (10, 4)] {
            let id = spawn(&mut w, x, y);
            let a = w.agent_mut(id).unwrap();
            a.metabolism[0] = 1;
            a.vision = 2;
            a.holdings[0] = 5.0;
            ids.push(id);
        }
        let start = total(&w);
        let (mut eaten, mut grown) = (0.0, 0.0);
        let (mut buried, mut dug, mut digs) = (0.0, 0.0, 0);
        for _ in 0..300 {
            w.events = crate::world::TickEvents::default();
            let mut order = w.agent_ids();
            use rand::seq::SliceRandom;
            order.shuffle(&mut w.rng);
            for id in order {
                eaten += f64::from(w.agent(id).unwrap().metabolism[0]);
                crate::rules::agent_turn(&mut w, id);
                assert!(w.agent(id).is_some(), "no deaths: the landscape is rich");
                // The script: bury all the surplus on even ticks, and every
                // fifth tick all but 2 (below R = 3, so it goes hungry and
                // digs).
                let q = if w.tick.is_multiple_of(5) {
                    w.agent(id).unwrap().holdings[0] - 2.0
                } else if w.tick.is_multiple_of(2) {
                    surplus(&w, id)
                } else {
                    0.0
                };
                bury(&mut w, id, q);
            }
            let before = sites_sum(&w);
            crate::rules::growback::apply(&mut w);
            grown += sites_sum(&w) - before;
            w.tick += 1;
            buried += w.events.buried;
            dug += w.events.dug;
            digs += w.events.digs;
            assert!(w.agents().all(|a| a.holdings[0] <= 8.0));
            assert_eq!(total(&w) + eaten, start + grown);
        }
        assert!(
            buried > 0.0 && dug > 0.0 && digs > 0,
            "{buried} {dug} {digs}"
        );
    }
}
