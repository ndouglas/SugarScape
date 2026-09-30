//! Minds 6: finding and pilfering caches on arrival.
//!
//! - **When.** Only under `theft.find` f > 0, and only on arrival
//!   (`movement::go_and_gather`), after the agent has moved and instead of
//!   harvesting the site. With f = 0 nothing here runs: no draw, no index.
//! - **The dig wins.** With `owner_memory` on, an agent whose own cache at
//!   the site would be dug (it's hungry and the cache is at least the site's
//!   welfare) digs it as in Minds 5 and makes no draw. Otherwise it draws.
//! - **The draws.** One `world.rng` draw (`gen_bool(f)`) per cache at the
//!   site that the agent doesn't know about, in owner-id order: every other
//!   agent's cache there, and under `owner_memory: off` its own as well.
//!   Every such cache gets its draw, whatever the earlier ones gave, so the
//!   number of draws depends only on the caches present. The first success
//!   is the one taken; at most one cache is taken a tick.
//! - **A pilfer.** The thief takes min(cache, room under the carrying
//!   limit) (the whole cache with no limit) from the owner's cache, the
//!   same under either loot rule; the rest stays the owner's, and an
//!   emptied cache is removed with its age. The owner's fate records close
//!   as `Pilfered { by }` (`fates`). It replaces the tick's harvest, as a
//!   dig does: the site and any truffle stay as they are. The take is
//!   `Harvest::pilfered`, never `gathered` (it was gathered once already:
//!   no pollution, not income, not the marginal-value rule's intake).
//!   A success with no room takes nothing, and the agent harvests the site
//!   as usual (taking nothing either, being full).
//! - **Loot.** `keep`: the take goes into the thief's holdings, where its
//!   burial rule may bury it again that tick. `eat`: it's eaten on the spot,
//!   counted as eaten in `events.loot_eaten` (as `bury_cost` is) and never
//!   reaching holdings.
//! - **An owner's own find** (`owner_memory: off`): a success on its own
//!   cache is a dig (`caching::dig`: `dug`, `digs`, a `Dug` fate, into its
//!   holdings under either loot rule), counted in `owner_finds`, not in
//!   `pilfered` or `pilfers`. Under `owner_memory: off` an owner never digs
//!   any other way, so with f = 0 its caches are never recovered.
//! - **The index.** Caches live in their owners' maps, so the agents whose
//!   caches lie at a site are found through `World.cache_sites` ([`CacheSites`],
//!   site → owners, iterated in id order). It's built from every agent's
//!   caches the first time an arrival needs it (so a `find` turned on live
//!   sees every cache already buried), then kept by bury, dig, pilfer and
//!   removal ([`note`]), and dropped by the first of those after f goes back
//!   to 0.
//!
//! Sugar is conserved: a pilfer moves the same f64 from a cache to holdings
//! (or to eaten), and an owner's find is a dig.

use std::collections::{BTreeMap, BTreeSet};

use rand::Rng;

use crate::agent::AgentId;
use crate::config::Loot;
use crate::rules::Harvest;
use crate::world::World;

/// Site index → the ids of the agents with a cache there.
pub(crate) type CacheSites = BTreeMap<u32, BTreeSet<AgentId>>;

/// Records that `owner` now has (`present`) or no longer has a cache at
/// `site`, when the index exists; drops the index once `find` is 0.
pub(crate) fn note(world: &mut World, owner: AgentId, site: u32, present: bool) {
    if world.cache_sites.is_none() {
        return;
    }
    if world.config.theft.find <= 0.0 {
        world.cache_sites = None;
        return;
    }
    let index = world.cache_sites.as_mut().expect("checked");
    if present {
        index.entry(site).or_default().insert(owner);
    } else if let Some(owners) = index.get_mut(&site) {
        owners.remove(&owner);
        if owners.is_empty() {
            index.remove(&site);
        }
    }
}

/// Builds the index from every agent's caches if it isn't built.
fn ensure_index(world: &mut World) {
    if world.cache_sites.is_some() {
        return;
    }
    let mut index = CacheSites::new();
    for a in world.agents() {
        for &site in a.caches.keys() {
            index.entry(site).or_default().insert(a.id);
        }
    }
    world.cache_sites = Some(index);
}

/// Counts the caches in the world into `events.pilfer_candidates` (called
/// at the tick's start under theft).
pub(crate) fn count_candidates(world: &mut World) {
    let n: usize = world.agents().map(|a| a.caches.len()).sum();
    world.events.pilfer_candidates = u32::try_from(n).unwrap_or(u32::MAX);
}

/// `id` has just arrived on site index `site`, with `room` left under the
/// carrying limit, and isn't digging its own cache there: draws for each
/// cache there it doesn't know about and takes the first found. Returns the
/// tick's harvest when it took something (a pilfer, or its own cache found
/// under `owner_memory: off`), or `None` to harvest the site as usual.
pub(crate) fn stumble(world: &mut World, id: AgentId, site: u32, room: f64) -> Option<Harvest> {
    let theft = world.config.theft;
    if theft.find <= 0.0 {
        return None;
    }
    ensure_index(world);
    let owners: Vec<AgentId> = world
        .cache_sites
        .as_ref()
        .and_then(|x| x.get(&site))?
        .iter()
        .copied()
        .filter(|&o| o != id || !theft.owner_memory)
        .collect();
    let mut found = None;
    for &o in &owners {
        let hit = world.rng.gen_bool(theft.find);
        if hit && found.is_none() {
            found = Some(o);
        }
    }
    let owner = found?;
    let mut harvest = Harvest::default();
    if owner == id {
        let take = super::dig(world, id, site, room);
        if take <= 0.0 {
            return None;
        }
        world.events.owner_finds += 1;
        world.agent_mut(id).expect("live agent").holdings[0] += take;
        harvest.dug = take;
        return Some(harvest);
    }
    let take = pilfer(world, owner, id, site, room);
    if take <= 0.0 {
        return None;
    }
    match theft.loot {
        Loot::Keep => world.agent_mut(id).expect("live agent").holdings[0] += take,
        Loot::Eat => world.events.loot_eaten += take,
    }
    harvest.pilfered = take;
    Some(harvest)
}

/// Takes min(cache, `room`) from `owner`'s cache at `site` for `thief` and
/// returns it (0 when there's nothing to take). Removes an emptied cache
/// with its age, closes that much of the owner's fate records as pilfered,
/// keeps the index, and counts `pilfered` and `pilfers`. The caller places
/// the loot.
pub(crate) fn pilfer(
    world: &mut World,
    owner: AgentId,
    thief: AgentId,
    site: u32,
    room: f64,
) -> f64 {
    let a = world.agent_mut(owner).expect("a cache has a live owner");
    let Some(cache) = a.caches.get_mut(&site) else {
        debug_assert!(false, "the index lists a cache that isn't there");
        return 0.0;
    };
    let take = cache.min(room).max(0.0);
    if take <= 0.0 {
        return 0.0;
    }
    let emptied = take >= *cache;
    if emptied {
        a.caches.remove(&site);
    } else {
        *cache -= take;
    }
    world.events.pilfered += take;
    world.events.pilfers += 1;
    // The fate log reads `cache_since` (for a backfill), so it goes after.
    super::fates::close_pilfered(world, owner, site, take, thief);
    if emptied {
        let a = world.agent_mut(owner).expect("live agent");
        a.cache_since.remove(&site);
        note(world, owner, site, false);
    }
    take
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MoveMode, Movement};
    use crate::geometry::Pos;
    use crate::minds::caching::bury;
    use crate::minds::caching::fates::Fate;
    use crate::rules::movement::{candidates, go_and_gather};
    use crate::testkit::*;

    fn at(w: &World, x: u32, y: u32) -> u32 {
        w.torus.index(Pos::new(x, y)) as u32
    }

    fn total(w: &World) -> f64 {
        let sites: f64 = w.sites.iter().map(|s| s.resource[0]).sum();
        let held: f64 = w.agents().map(|a| a.holdings[0]).sum();
        let cached: f64 = w.agents().flat_map(|a| a.caches.values()).sum();
        sites + held + cached
    }

    /// An agent at (x, y) with metabolism 1 and `held` sugar.
    fn agent(w: &mut World, x: u32, y: u32, held: f64) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.holdings[0] = held;
        id
    }

    /// A world with `find` and carrying limit `capacity`, and an owner who
    /// buried `cached` at (5, 6) (with sugar 2 on the site) and walked off
    /// to (1, 1).
    fn cached_world(find: f64, capacity: u32, cached: f64) -> (World, AgentId) {
        let mut w = blank_world(11, 11);
        w.config.theft.find = find;
        w.config.caching.capacity = capacity;
        let owner = agent(&mut w, 5, 6, cached.max(100.0));
        bury(&mut w, owner, cached);
        w.move_agent(owner, Pos::new(1, 1));
        set_sugar(&mut w, 5, 6, 2.0);
        (w, owner)
    }

    fn pilfered_by(w: &World, by: AgentId) -> f64 {
        w.cache_log
            .iter()
            .filter(|r| r.fate.is_some_and(|(_, f)| f == Fate::Pilfered { by }))
            .map(|r| r.amount)
            .sum()
    }

    #[test]
    fn a_found_cache_is_pilfered_whole_instead_of_the_harvest() {
        let (mut w, owner) = cached_world(1.0, 0, 6.0);
        let thief = agent(&mut w, 5, 5, 3.0);
        let before = total(&w);
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.dug, h.pilfered), (0.0, 0.0, 6.0));
        assert_eq!(w.agent(thief).unwrap().holdings[0], 9.0, "kept");
        assert_eq!(w.site(Pos::new(5, 6)).resource[0], 2.0, "site untouched");
        let o = w.agent(owner).unwrap();
        assert!(o.caches.is_empty() && o.cache_since.is_empty());
        assert_eq!((w.events.pilfered, w.events.pilfers), (6.0, 1));
        assert_eq!(
            (w.events.dug, w.events.digs, w.events.loot_eaten),
            (0.0, 0, 0.0)
        );
        assert_eq!(pilfered_by(&w, thief), 6.0);
        assert!(w.cache_open.is_empty());
        assert_eq!(total(&w), before);
        assert!(w.cache_sites.as_ref().unwrap().is_empty(), "index kept");
    }

    // Review Focus 1.
    #[test]
    fn a_thief_at_the_carrying_limit_takes_only_its_room() {
        let (mut w, owner) = cached_world(1.0, 10, 6.0);
        let here = at(&w, 5, 6);
        let since = w.agent(owner).unwrap().cache_since[&here];
        let thief = agent(&mut w, 5, 5, 8.0);
        let before = total(&w);
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!(h.pilfered, 2.0, "room is 10 − 8");
        assert_eq!(w.agent(thief).unwrap().holdings[0], 10.0);
        let o = w.agent(owner).unwrap();
        assert_eq!(o.caches[&here], 4.0, "the rest stays the owner's");
        assert_eq!(o.cache_since[&here], since, "its age unchanged");
        assert_eq!(
            (o.holdings[0], o.pos),
            (94.0, Pos::new(1, 1)),
            "owner unchanged"
        );
        assert_eq!(pilfered_by(&w, thief), 2.0);
        let open: f64 = w.cache_open[&(owner, here)]
            .iter()
            .map(|&i| w.cache_log[i].amount)
            .sum();
        assert_eq!(open, 4.0, "the log splits the record");
        assert_eq!(total(&w), before);
        // Full: a success takes nothing and it harvests the site, taking
        // nothing either.
        w.move_agent(thief, Pos::new(5, 5));
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (0.0, 0.0));
        assert_eq!(w.agent(owner).unwrap().caches[&here], 4.0);
        assert_eq!(w.events.pilfers, 1);
        assert_eq!(total(&w), before);
    }

    #[test]
    fn eaten_loot_is_counted_as_eaten_and_never_held() {
        let (mut w, owner) = cached_world(1.0, 10, 6.0);
        w.config.theft.loot = Loot::Eat;
        let thief = agent(&mut w, 5, 5, 7.0);
        let before = total(&w);
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!(h.pilfered, 3.0, "capped by the room, 10 − 7");
        assert_eq!(w.agent(thief).unwrap().holdings[0], 7.0);
        assert_eq!(w.agent(owner).unwrap().caches[&at(&w, 5, 6)], 3.0);
        assert_eq!((w.events.pilfered, w.events.loot_eaten), (3.0, 3.0));
        assert_eq!(total(&w) + w.events.loot_eaten, before);
    }

    #[test]
    fn with_no_success_the_agent_harvests_as_usual() {
        let (mut w, owner) = cached_world(1e-12, 0, 6.0);
        let thief = agent(&mut w, 5, 5, 3.0);
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
        assert_eq!(w.agent(owner).unwrap().caches[&at(&w, 5, 6)], 6.0);
        assert_eq!(w.events.pilfers, 0);
    }

    #[test]
    fn one_draw_per_foreign_cache_in_owner_id_order_and_the_first_found_is_taken() {
        let mut w = blank_world(11, 11);
        w.config.theft.find = 1.0;
        let here = at(&w, 5, 6);
        let mut owners = Vec::new();
        for amount in [3.0, 4.0, 5.0] {
            let o = agent(&mut w, 5, 6, 50.0);
            bury(&mut w, o, amount);
            w.move_agent(o, Pos::new(owners.len() as u32, 0));
            owners.push(o);
        }
        let thief = agent(&mut w, 5, 5, 0.0);
        let mut rng = w.rng.clone();
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!(h.pilfered, 3.0, "the lowest id's cache");
        assert!(!w.agent(owners[0]).unwrap().caches.contains_key(&here));
        assert_eq!(w.agent(owners[1]).unwrap().caches[&here], 4.0);
        assert_eq!(w.agent(owners[2]).unwrap().caches[&here], 5.0);
        // Three draws were made, one per cache, though the first succeeded.
        for _ in 0..3 {
            rng.gen_bool(1.0);
        }
        assert_eq!(rng, w.rng);
        // A scripted order: with find 0.5, the draws decide which is taken.
        let mut w2 = blank_world(11, 11);
        w2.config.theft.find = 0.5;
        for amount in [3.0, 4.0, 5.0] {
            let o = agent(&mut w2, 5, 6, 50.0);
            bury(&mut w2, o, amount);
            w2.move_agent(o, Pos::new(amount as u32, 0));
        }
        let thief = agent(&mut w2, 5, 5, 0.0);
        let mut rng = w2.rng.clone();
        let hits: Vec<bool> = (0..3).map(|_| rng.gen_bool(0.5)).collect();
        let expect = hits
            .iter()
            .position(|&h| h)
            .map_or(0.0, |i| [3.0, 4.0, 5.0][i]);
        let h = go_and_gather(&mut w2, thief, Pos::new(5, 6));
        assert_eq!(h.pilfered, expect, "{hits:?}");
        assert_eq!(rng, w2.rng);
    }

    #[test]
    fn an_owner_that_would_dig_digs_and_draws_nothing() {
        let mut w = blank_world(11, 11);
        w.config.theft.find = 1.0;
        let other = agent(&mut w, 5, 6, 50.0);
        bury(&mut w, other, 5.0);
        w.move_agent(other, Pos::new(1, 1));
        let owner = agent(&mut w, 5, 6, 10.0);
        bury(&mut w, owner, 8.0);
        w.move_agent(owner, Pos::new(5, 5));
        // Holdings 2 < R / 2 = 5, and the cache beats the bare site.
        let rng = w.rng.clone();
        let h = go_and_gather(&mut w, owner, Pos::new(5, 6));
        assert_eq!((h.dug, h.pilfered), (8.0, 0.0));
        assert_eq!(rng, w.rng, "no draw");
        assert_eq!(w.agent(other).unwrap().caches[&at(&w, 5, 6)], 5.0);
        // Not hungry, it draws for the foreign cache only, and pilfers it.
        w.move_agent(owner, Pos::new(5, 5));
        bury(&mut w, owner, 1.0);
        w.move_agent(owner, Pos::new(5, 6));
        w.move_agent(owner, Pos::new(5, 5));
        let mut rng = w.rng.clone();
        let h = go_and_gather(&mut w, owner, Pos::new(5, 6));
        assert_eq!(h.pilfered, 5.0);
        rng.gen_bool(1.0);
        assert_eq!(rng, w.rng, "one draw: its own cache isn't drawn for");
        assert!(w.agent(other).unwrap().caches.is_empty());
    }

    // Review Focus 2.
    #[test]
    fn an_owner_and_a_thief_on_one_cache_in_one_tick_each_take_at_most_once() {
        // The owner first: it digs its room (4 of 10) and walks on; the
        // thief then pilfers the rest.
        let (mut w, owner) = cached_world(1.0, 6, 10.0);
        let here = at(&w, 5, 6);
        w.agent_mut(owner).unwrap().holdings[0] = 2.0;
        w.move_agent(owner, Pos::new(5, 7));
        let thief = agent(&mut w, 5, 5, 0.0);
        let before = total(&w);
        let h = go_and_gather(&mut w, owner, Pos::new(5, 6));
        assert_eq!((h.dug, h.pilfered), (4.0, 0.0));
        w.move_agent(owner, Pos::new(5, 7));
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0);
        assert!(!w.agent(owner).unwrap().caches.contains_key(&here));
        assert_eq!((w.events.digs, w.events.pilfers), (1, 1));
        assert_eq!(total(&w), before);
        // The thief first: it pilfers its room (6 of 10) and walks on; the
        // owner digs the rest.
        let (mut w, owner) = cached_world(1.0, 6, 10.0);
        w.agent_mut(owner).unwrap().holdings[0] = 2.0;
        w.move_agent(owner, Pos::new(5, 7));
        let thief = agent(&mut w, 5, 5, 0.0);
        let before = total(&w);
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0);
        w.move_agent(thief, Pos::new(5, 5));
        let h = go_and_gather(&mut w, owner, Pos::new(5, 6));
        assert_eq!((h.dug, h.pilfered), (4.0, 0.0));
        assert!(w.agent(owner).unwrap().caches.is_empty());
        assert_eq!((w.events.digs, w.events.pilfers), (1, 1));
        assert_eq!(total(&w), before);
        let dug: f64 = w
            .cache_log
            .iter()
            .filter(|r| r.fate.is_some_and(|(_, f)| f == Fate::Dug))
            .map(|r| r.amount)
            .sum();
        assert_eq!((dug, pilfered_by(&w, thief)), (4.0, 6.0));
        assert!(w.cache_open.is_empty());
    }

    // Review Focus 4.
    #[test]
    fn without_owner_memory_own_caches_arent_candidates_or_dug() {
        let mut w = blank_world(15, 15);
        w.config.theft.owner_memory = false;
        let id = agent(&mut w, 5, 5, 20.0);
        w.move_agent(id, Pos::new(5, 9));
        bury(&mut w, id, 20.0);
        w.move_agent(id, Pos::new(5, 5));
        set_sugar(&mut w, 5, 6, 1.0);
        let plain = {
            let mut w2 = w.clone();
            w2.agent_mut(id).unwrap().caches.clear();
            candidates(&w2, id)
        };
        assert!(crate::minds::caching::hungry(&w, id));
        assert_eq!(candidates(&w, id), plain, "the cache doesn't join");
        assert_eq!(
            crate::minds::caching::cache_value(&w, id, Pos::new(5, 9)),
            None
        );
        // Standing on it, hungry, with find 0, it never digs: it harvests.
        w.move_agent(id, Pos::new(5, 8));
        let h = go_and_gather(&mut w, id, Pos::new(5, 9));
        assert_eq!((h.dug, h.gathered[0]), (0.0, 0.0));
        assert_eq!(w.agent(id).unwrap().caches[&at(&w, 5, 9)], 20.0);
        assert_eq!(w.events.digs, 0);
        assert!(w.cache_sites.is_none(), "find 0: no index");
    }

    #[test]
    fn without_owner_memory_an_owner_finds_its_cache_at_rate_find_as_a_dig() {
        let mut w = blank_world(11, 11);
        w.config.theft.owner_memory = false;
        w.config.theft.find = 1.0;
        let id = agent(&mut w, 5, 6, 30.0);
        bury(&mut w, id, 30.0);
        w.move_agent(id, Pos::new(5, 5));
        let before = total(&w);
        let h = go_and_gather(&mut w, id, Pos::new(5, 6));
        assert_eq!((h.dug, h.pilfered), (30.0, 0.0));
        assert_eq!((w.events.owner_finds, w.events.digs), (1, 1));
        assert_eq!((w.events.pilfers, w.events.pilfered), (0, 0.0));
        assert!(w
            .cache_log
            .iter()
            .all(|r| r.fate.is_some_and(|(_, f)| f == Fate::Dug)));
        assert_eq!(total(&w), before);
        // Not hungry (holdings 29 ≥ R / 2), it finds it all the same: it
        // stumbles on it, not having chosen it.
        bury(&mut w, id, 1.0);
        assert!(!crate::minds::caching::hungry(&w, id));
        w.move_agent(id, Pos::new(5, 5));
        let h = go_and_gather(&mut w, id, Pos::new(5, 6));
        assert_eq!((h.dug, h.gathered[0]), (1.0, 0.0));
        assert_eq!(w.events.owner_finds, 2);
        // With find 0 it never finds it.
        bury(&mut w, id, 1.0);
        w.config.theft.find = 0.0;
        w.move_agent(id, Pos::new(5, 5));
        assert_eq!(go_and_gather(&mut w, id, Pos::new(5, 6)).dug, 0.0);
        w.agent_mut(id).unwrap().holdings[0] = 0.0;
        w.move_agent(id, Pos::new(5, 5));
        assert_eq!(go_and_gather(&mut w, id, Pos::new(5, 6)).dug, 0.0, "hungry");
        assert_eq!(w.events.owner_finds, 2);
    }

    #[test]
    fn without_owner_memory_and_find_0_caches_end_lost_or_buried() {
        let mut c = crate::presets::by_id("cache-winter-mixed").unwrap().config;
        c.theft.owner_memory = false;
        c.theft.cheaters = 0.2;
        let mut w = World::new(c, 3).unwrap();
        let (mut buried, mut lost) = (0.0, 0.0);
        for _ in 0..300 {
            w.step();
            assert_eq!((w.events.digs, w.events.dug), (0, 0.0));
            assert_eq!(w.events.pilfers, 0);
            buried += w.events.buried;
            lost += w.events.cache_lost;
        }
        assert!(buried > 0.0 && lost > 0.0, "{buried} {lost}");
        assert!(w
            .cache_log
            .iter()
            .all(|r| matches!(r.fate, None | Some((_, Fate::Lost)))));
        assert!(w.cache_sites.is_none());
    }

    #[test]
    fn the_pilfer_frequency_matches_find_over_10000_trials() {
        let find = 0.3;
        // A limit of 1 and an empty thief: each success takes 1 of 1e6.
        let (mut w, owner) = cached_world(find, 1, 1e6);
        let thief = agent(&mut w, 5, 5, 0.0);
        let n = 10_000;
        for _ in 0..n {
            w.agent_mut(thief).unwrap().holdings[0] = 0.0;
            go_and_gather(&mut w, thief, Pos::new(5, 6));
            w.move_agent(thief, Pos::new(5, 5));
        }
        let k = f64::from(w.events.pilfers);
        let sd = (f64::from(n) * find * (1.0 - find)).sqrt();
        let z = (k - f64::from(n) * find) / sd;
        assert!(z.abs() < 4.0, "{k} pilfers of {n}: z = {z}");
        assert!(w.agent(owner).unwrap().caches[&at(&w, 5, 6)] > 0.0);
    }

    #[test]
    fn find_turned_on_live_sees_caches_buried_before_and_off_drops_the_index() {
        let (mut w, owner) = cached_world(0.0, 0, 6.0);
        let thief = agent(&mut w, 5, 5, 0.0);
        let rng = w.rng.clone();
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
        assert!(w.cache_sites.is_none());
        assert_eq!(rng, w.rng, "find 0: no draw");
        w.move_agent(thief, Pos::new(5, 5));
        w.config.theft.find = 1.0;
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0, "the index was built with the old cache");
        assert!(w.agent(owner).unwrap().caches.is_empty());
        assert!(w.cache_sites.is_some());
        w.config.theft.find = 0.0;
        w.config.theft.cheaters = 0.5;
        bury(&mut w, thief, 1.0);
        assert!(w.cache_sites.is_none(), "dropped at the next touch");
    }

    #[test]
    fn a_dead_owners_caches_leave_the_index() {
        let (mut w, owner) = cached_world(1.0, 0, 6.0);
        let thief = agent(&mut w, 5, 5, 0.0);
        go_and_gather(&mut w, thief, Pos::new(5, 5));
        assert_eq!(w.cache_sites.as_ref().unwrap().len(), 1);
        w.kill(owner, crate::world::DeathCause::OldAge);
        assert!(w.cache_sites.as_ref().unwrap().is_empty());
        let h = go_and_gather(&mut w, thief, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
    }

    #[test]
    fn candidates_are_counted_at_the_tick_start_under_theft_only() {
        let mut c = crate::presets::by_id("cache-winter-mixed").unwrap().config;
        let mut w = World::new(c.clone(), 5).unwrap();
        for _ in 0..30 {
            w.step();
            assert_eq!(w.events.pilfer_candidates, 0);
        }
        c.theft.find = 0.2;
        let mut w = World::new(c, 5).unwrap();
        let (mut pilfers, mut candidates) = (0, 0);
        for _ in 0..300 {
            let caches: usize = w.agents().map(|a| a.caches.len()).sum();
            w.step();
            assert_eq!(w.events.pilfer_candidates as usize, caches);
            pilfers += w.events.pilfers;
            candidates += w.events.pilfer_candidates;
        }
        assert!(
            pilfers > 0 && candidates > pilfers,
            "{pilfers} {candidates}"
        );
    }

    /// Σ sites + holdings + caches + eaten (metabolism, bury cost and eaten
    /// loot) + what left with the dead = start + growback, over 300 ticks
    /// of five walkers burying by script under `find` 0.5, one killed at
    /// tick 150 with its caches.
    fn conserved_through_theft(loot: Loot) {
        let mut c = blank_config(12, 12);
        c.theft.find = 0.5;
        c.theft.loot = loot;
        c.caching.bury_cost = 0.1;
        c.growback.rate = 0.3;
        c.goap.horizon = 6;
        c.caching.capacity = 8;
        c.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        let mut w = World::new(c, 11).unwrap();
        for y in 0..12 {
            for x in 0..12 {
                set_sugar(&mut w, x, y, if (x + y) % 3 == 0 { 4.0 } else { 1.5 });
            }
        }
        let mut ids = Vec::new();
        for (x, y) in [(1, 1), (6, 2), (3, 8), (9, 9), (10, 4)] {
            let id = agent(&mut w, x, y, 5.0);
            w.agent_mut(id).unwrap().vision = 2;
            ids.push(id);
        }
        let start = total(&w);
        let (mut eaten, mut grown, mut left) = (0.0, 0.0, 0.0);
        let (mut pilfered, mut pilfers, mut dug) = (0.0, 0, 0.0);
        let doomed = ids[0];
        for _ in 0..300 {
            w.events = crate::world::TickEvents::default();
            if w.tick == 150 {
                let a = w.agent(doomed).unwrap();
                assert!(!a.caches.is_empty(), "it dies holding caches");
                left += a.holdings[0];
                w.kill(doomed, crate::world::DeathCause::OldAge);
            }
            let mut order = w.agent_ids();
            use rand::seq::SliceRandom;
            order.shuffle(&mut w.rng);
            for id in order {
                eaten += f64::from(w.agent(id).unwrap().metabolism[0]);
                crate::rules::agent_turn(&mut w, id);
                assert!(w.agent(id).is_some(), "the landscape is rich");
                // Every fifth tick it buries all but 2.5, below R / 2 = 3,
                // so it goes hungry and digs (and survives two ticks
                // without food, as eaten loot is).
                let q = if w.tick.is_multiple_of(5) {
                    w.agent(id).unwrap().holdings[0] - 2.5
                } else if w.tick.is_multiple_of(2) {
                    0.3 * crate::minds::caching::surplus(&w, id)
                } else {
                    0.0
                };
                bury(&mut w, id, q);
            }
            let before: f64 = w.sites.iter().map(|s| s.resource[0]).sum();
            crate::rules::growback::apply(&mut w);
            grown += w.sites.iter().map(|s| s.resource[0]).sum::<f64>() - before;
            w.tick += 1;
            left += w.events.cache_lost;
            eaten += w.events.bury_cost + w.events.loot_eaten;
            pilfered += w.events.pilfered;
            pilfers += w.events.pilfers;
            dug += w.events.dug;
            let e = &w.events;
            match loot {
                Loot::Eat => assert_eq!(e.loot_eaten, e.pilfered),
                Loot::Keep => assert_eq!(e.loot_eaten, 0.0),
            }
            assert!(e.pilfers <= 4, "at most one each");
            assert!(w.agents().all(|a| a.holdings[0] <= 8.0 + 1e-12));
            let (lhs, rhs) = (total(&w) + eaten + left, start + grown);
            assert!((lhs - rhs).abs() <= 1e-9 * rhs, "{lhs} vs {rhs}");
            // The index matches the caches.
            let mut index = CacheSites::new();
            for a in w.agents() {
                for &s in a.caches.keys() {
                    index.entry(s).or_default().insert(a.id);
                }
            }
            if let Some(kept) = &w.cache_sites {
                assert_eq!(kept, &index);
            }
        }
        assert_eq!(w.population(), 4);
        assert!(
            pilfered > 0.0 && pilfers > 0 && dug > 0.0,
            "{pilfered} {dug}"
        );
        // The log agrees: Σ pilfered records = Σ pilfered.
        let logged: f64 = w
            .cache_log
            .iter()
            .filter(|r| matches!(r.fate, Some((_, Fate::Pilfered { .. }))))
            .map(|r| r.amount)
            .sum();
        assert!((logged - pilfered).abs() <= 1e-9 * (1.0 + pilfered));
    }

    #[test]
    fn sugar_is_conserved_through_theft_when_the_loot_is_kept() {
        conserved_through_theft(Loot::Keep);
    }

    #[test]
    fn sugar_is_conserved_through_theft_when_the_loot_is_eaten() {
        conserved_through_theft(Loot::Eat);
    }

    #[test]
    fn a_pilfer_tick_harvests_no_site_for_compensate() {
        let (mut w, _) = cached_world(1.0, 0, 6.0);
        w.config.caching.rule = crate::config::CachingRule::Compensate;
        let thief = agent(&mut w, 5, 5, 0.0);
        crate::rules::agent_turn(&mut w, thief);
        assert_eq!(w.agent(thief).unwrap().pos, Pos::new(5, 6));
        assert_eq!(w.events.pilfers, 1);
        assert!(w.agent(thief).unwrap().weights.map().is_empty());
    }
}
