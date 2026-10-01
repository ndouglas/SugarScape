//! Minds 8: watchers who see a burial, remember the cache and raid it.
//!
//! - **Who watches.** `Agent.watches`: founders dealt by id under
//!   `watching.watchers` (`Watching::founder_watches`, the rule of
//!   `Theft::founder_cheats`), with no draw. An agent born later doesn't
//!   watch.
//! - **Seeing a burial** ([`see`], called by `caching::bury` after a
//!   positive burial). Only under `watching.on`. Every living watcher other
//!   than the owner whose sight covers the burial site remembers the cache:
//!   the site is in `World::sight(w.pos, w.vision)`, on one of the four
//!   lattice lines from the watcher, within its vision and not behind an
//!   opaque wall (a fence doesn't block). The owner never knows it was seen.
//! - **The walk.** Sight is symmetric along a lattice line: the sites
//!   between a watcher and the burial are the same seen from either end.
//!   So rather than asking every agent's sight, `see` walks the four lines
//!   out from the burial site, up to the largest vision any watcher has,
//!   stopping each at an opaque wall as `Torus::sight_until` does. Each
//!   watcher met at distance d ≤ its own vision sees the burial. On a torus
//!   small enough that the lines wrap, a watcher may be met more than once;
//!   it sees once. Watchers are then visited in id order.
//! - **The memory.** `Agent.seen`, keyed (site index, owner): the amount
//!   seen buried and the tick last seen. A second burial by the same owner
//!   at the same site, seen again, adds its amount and refreshes the tick;
//!   a burial not seen leaves the entry as it was.
//! - **Forgetting.** An entry is remembered while now − tick ≤ `span`
//!   ([`fresh`]). The tick-start [`sweep`] (only while `watching.on`) drops
//!   the rest from every agent, so an entry kept while watching was off, or
//!   under a longer `span`, is forgotten as soon as it's read again. A
//!   watcher's entries die with it. Entries about an owner who has died
//!   stay until read. An arrival on a site forgets every entry there
//!   ([`forget`]), whatever the agent did there (a dig, a raid or a
//!   harvest), except a fed arrival under `raid_when: hungry` that doesn't
//!   dig: it doesn't raid, so it keeps them (see below). A walk that finds
//!   no path to its target forgets every entry at the target ([`give_up`]).
//! - **Going to a seen cache** ([`join_seen`], called after
//!   `caching::join_caches` wherever rule M's candidates are built). Each
//!   site with fresh entries joins the candidates after the agent's own
//!   caches and before Minds 3's remembered sites, valued at the amounts
//!   remembered there summed over owners: what it believes, so a cache dug
//!   or taken since still looks full until it arrives. The same sites are
//!   skipped and merged as for its own caches (`caching::join_sites`).
//!   Under `raid_when: hungry` they join only while it holds less than
//!   R / 2 (with or without caches of its own). Minds 3's true value of a
//!   candidate counts them, under the same gates, at what is truly left of
//!   them ([`seen_truth`]), so a choice of an emptied one reads as stale.
//! - **Giving up** ([`give_up`], called by `movement::arrive` where a walk
//!   finds no path, the one place that outcome is decided). A seen cache
//!   in a pocket other agents seal off, or walled apart, can't be reached;
//!   `join_sites` skips only the last failed target, so without this an
//!   agent would chain from one unreachable seen cache to the next. The
//!   walker forgets its entries at the target (every owner's) and counts
//!   nothing. Its own caches keep Minds 5's rule.
//! - **A raid** ([`raid`], called by `movement::go_and_gather` on every
//!   arrival under `watching.on`, after the dig and before Minds 6's
//!   stumble). The dig wins: an owner digging its own cache there doesn't
//!   raid. Under `raid_when: hungry` the raid has the joining gate too, on
//!   the owners' terms: an agent at or above R / 2 doesn't raid, forgets
//!   nothing, counts nothing, and stumbles and harvests as if it had no
//!   entries (under `always` every arrival may raid). Otherwise the agent
//!   takes from the first owner, in id order, of a fresh entry at the site
//!   whose cache is still there, with no draw: a pilfer by Minds 6's rules
//!   (`theft::loot`: min(cache, room) kept, or the whole cache eaten;
//!   `pilfered`, `pilfers`, `caches_pilfered` and a `Pilfered { by }`
//!   fate). A raid that took something replaces the
//!   tick's harvest, and nothing is drawn for the other caches there: at
//!   most one take an arrival. If none of the remembered caches is still
//!   there (dug, taken, or lost with a dead owner, who is never looked up),
//!   the raid is wasted and the agent goes on to stumble and harvest. A
//!   watcher with no room under `keep` takes nothing (the cache stays its
//!   owner's) and harvests as usual; that is neither a raid nor wasted.
//!   Under the survey probe `World::probe_raid_harvests` (not config) a
//!   raid that took something also harvests the site, under the carrying
//!   limit with kept loot counted against it, and still draws no stumble.
//! - **Counts.** `burials_seen` counts burials with at least one watcher;
//!   `sightings` counts (watcher, burial) pairs; `seen_entries` is the
//!   entries held, summed over agents, after the sweep. `seen_arrivals`
//!   counts arrivals that may raid with a fresh entry at the site, `raids`
//!   and `raided` the takes (also in `pilfers` and `pilfered`), and
//!   `raids_wasted` the arrivals whose remembered caches were all gone.
//!   Minds 6's pilfering bookkeeping (candidates, the fate log, the theft
//!   stats) runs under watching as under theft (`Config::pilfering_on`).
//!
//! Nothing here draws. Sugar is conserved: a raid is a pilfer, moving the
//! same f64 from a cache to holdings or a stomach. With `watching.on` false
//! nothing runs and nothing is allocated.

use std::collections::{BTreeMap, BTreeSet};

use crate::agent::AgentId;
use crate::config::RaidWhen;
use crate::geometry::{Pos, DIRECTIONS};
use crate::rules::Harvest;
use crate::world::World;

/// A cache a watcher saw buried: the amount it saw and the tick it last saw
/// a burial there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SeenCache {
    pub amount: f64,
    pub tick: u64,
}

/// `owner` has just buried `q` at site `site` (its own position).
pub(crate) fn see(world: &mut World, owner: AgentId, site: u32, q: f64) {
    if !world.config.watching.on {
        return;
    }
    let watchers = watchers_of(world, owner, site);
    if watchers.is_empty() {
        return;
    }
    let now = world.tick;
    for &w in &watchers {
        let e = world
            .agent_mut(w)
            .expect("live agent")
            .seen
            .entry((site, owner))
            .or_insert(SeenCache {
                amount: 0.0,
                tick: now,
            });
        e.amount += q;
        e.tick = now;
    }
    world.events.burials_seen += 1;
    world.events.sightings += u32::try_from(watchers.len()).unwrap_or(u32::MAX);
}

/// The watchers other than `owner` whose sight covers site index `site`:
/// the four lattice lines walked out from the site, each stopped at an
/// opaque wall, up to the largest vision any watcher has.
fn watchers_of(world: &World, owner: AgentId, site: u32) -> BTreeSet<AgentId> {
    let mut out = BTreeSet::new();
    let reach = world
        .agents()
        .filter(|a| a.watches && a.id != owner)
        .map(|a| a.vision)
        .max()
        .unwrap_or(0);
    if reach == 0 {
        return out;
    }
    let torus = world.torus;
    let from = torus.pos(site as usize);
    let walls = world.has_walls();
    for (dx, dy) in DIRECTIONS {
        for d in 1..=reach {
            let p = torus.offset(from, dx * d as i32, dy * d as i32);
            if walls && world.is_opaque(p) {
                break;
            }
            let Some(a) = world.agent_at(p) else {
                continue;
            };
            if a.watches && a.id != owner && d <= a.vision {
                out.insert(a.id);
            }
        }
    }
    out
}

/// Drops every entry older than `span` (now − tick > span), from every agent.
pub(crate) fn sweep(world: &mut World) {
    let (now, span) = (world.tick, u64::from(world.config.watching.span));
    let mut held = 0usize;
    for a in world.agents_mut() {
        if !a.seen.is_empty() {
            a.seen.retain(|_, e| now.saturating_sub(e.tick) <= span);
            held += a.seen.len();
        }
    }
    world.events.seen_entries = u32::try_from(held).unwrap_or(u32::MAX);
}

/// Whether an entry seen at `tick` is still remembered now.
pub(crate) fn fresh(world: &World, tick: u64) -> bool {
    world.tick.saturating_sub(tick) <= u64::from(world.config.watching.span)
}

/// Adds `id`'s remembered caches to rule M's candidates (after its own
/// caches, before Minds 3's remembered sites): each site with a fresh
/// entry, in site index order, valued at the amounts remembered there
/// summed over owners, by `caching::join_sites`'s rules (the same sites
/// skipped and merged as for the agent's own caches). Only under
/// `watching.on`, and under `raid_when: hungry` only below R / 2 ([`raiding`]).
/// No entries: the list is untouched and nothing allocates.
pub(crate) fn join_seen(
    world: &World,
    id: AgentId,
    out: &mut Vec<(Pos, u32, f64)>,
    start: &mut usize,
) {
    if !world.config.watching.on {
        return;
    }
    let a = world.agent(id).expect("live agent");
    if a.seen.is_empty() || !raiding(world, id) {
        return;
    }
    let mut sites: Vec<(u32, f64)> = Vec::new();
    for (&(site, _), e) in &a.seen {
        if !fresh(world, e.tick) {
            continue;
        }
        match sites.last_mut() {
            Some((last, v)) if *last == site => *v += e.amount,
            _ => sites.push((site, e.amount)),
        }
    }
    if !sites.is_empty() {
        super::join_sites(world, id, sites.into_iter(), out, start);
    }
}

/// What `id` believes is buried at `p` by others it saw (summed over
/// owners), when seen caches are candidates now. `None` with no fresh entry
/// there, with watching off, or under `raid_when: hungry` at or above R / 2.
/// The value `join_seen` lists; tests read it (the diagnostics read
/// [`seen_truth`] instead).
#[cfg(test)]
pub(crate) fn seen_value(world: &World, id: AgentId, p: Pos) -> Option<f64> {
    if !world.config.watching.on {
        return None;
    }
    let a = world.agent(id).expect("live agent");
    if a.seen.is_empty() || !raiding(world, id) {
        return None;
    }
    let site = world.torus.index(p) as u32;
    at_site(&a.seen, site)
        .filter(|(_, e)| fresh(world, e.tick))
        .map(|(_, e)| e.amount)
        .reduce(|x, y| x + y)
}

/// Whether `id`'s seen caches are places to go, and to raid, now: always,
/// or under `raid_when: hungry` while it holds less than R / 2 (Minds 5's
/// threshold, without `hungry`'s requirement that it has caches of its
/// own). It stays R / 2 under the survey probe `probe_dig_at_reserve`, by
/// design: the spec's threshold.
fn raiding(world: &World, id: AgentId) -> bool {
    match world.config.watching.raid_when {
        RaidWhen::Always => true,
        RaidWhen::Hungry => {
            let held = world.agent(id).expect("live agent").holdings[0];
            held < super::reserve(world, id) / 2.0
        }
    }
}

/// What is truly at `p` of the caches `id` remembers there (Minds 3's true
/// value of a candidate): the sum, over its fresh entries there, of each
/// owner's cache still at the site (0 if gone or the owner is dead). Under
/// the gates of [`seen_value`]; `None` where that is `None`.
pub(crate) fn seen_truth(world: &World, id: AgentId, p: Pos) -> Option<f64> {
    if !world.config.watching.on {
        return None;
    }
    let a = world.agent(id).expect("live agent");
    if a.seen.is_empty() || !raiding(world, id) {
        return None;
    }
    let site = world.torus.index(p) as u32;
    at_site(&a.seen, site)
        .filter(|(_, e)| fresh(world, e.tick))
        .map(|(owner, _)| {
            world
                .agent(owner)
                .and_then(|o| o.caches.get(&site).copied())
                .unwrap_or(0.0)
        })
        .reduce(|x, y| x + y)
}

/// `id`'s walk found no path to site index `site` (`movement::arrive`):
/// forgets its entries there, every owner's, fresh or not, counting
/// nothing. Its own caches keep Minds 5's rule (`caching::join_sites` skips
/// only the last failed target).
pub(crate) fn give_up(world: &mut World, id: AgentId, site: u32) {
    let a = world.agent_mut(id).expect("live agent");
    if !a.seen.is_empty() {
        a.seen.retain(|&(s, _), _| s != site);
    }
}

/// The entries at site index `site`, in owner-id order.
fn at_site(
    seen: &BTreeMap<(u32, AgentId), SeenCache>,
    site: u32,
) -> impl Iterator<Item = (AgentId, &SeenCache)> {
    seen.range((site, AgentId::MIN)..=(site, AgentId::MAX))
        .map(|(&(_, owner), e)| (owner, e))
}

/// `id` has arrived on site index `site`: forgets its entries there,
/// counting `seen_arrivals` once if any was fresh. Returns whether one was.
pub(crate) fn forget(world: &mut World, id: AgentId, site: u32) -> bool {
    let (any, any_fresh) = {
        let a = world.agent(id).expect("live agent");
        let mut entries = at_site(&a.seen, site).peekable();
        let any = entries.peek().is_some();
        (any, entries.any(|(_, e)| fresh(world, e.tick)))
    };
    if !any {
        return false;
    }
    let a = world.agent_mut(id).expect("live agent");
    a.seen.retain(|&(s, _), _| s != site);
    if any_fresh {
        world.events.seen_arrivals += 1;
    }
    any_fresh
}

/// On arrival: raid a remembered cache at `site`. `None` = nothing taken,
/// so go on to stumbling.
///
/// Under `raid_when: hungry` an agent at or above R / 2 ([`raiding`])
/// doesn't raid: it forgets nothing, counts nothing and returns `None`, as
/// with no entries. Otherwise it takes from the first owner, in id order,
/// of a fresh entry at the site whose cache is still there, through
/// `theft::loot` (min(cache, `room`) kept, or the whole cache eaten),
/// counting `raids` and `raided`; the entries at the site are forgotten
/// either way. None still there (dug, pilfered, or lost with a dead owner):
/// a wasted raid. A take of 0 (no room under `keep`) is neither.
pub(crate) fn raid(world: &mut World, id: AgentId, site: u32, room: f64) -> Option<Harvest> {
    let a = world.agent(id).expect("live agent");
    if a.seen.is_empty() || !raiding(world, id) {
        return None;
    }
    let target = {
        let mut owners = at_site(&a.seen, site)
            .filter(|(_, e)| fresh(world, e.tick))
            .map(|(owner, _)| owner);
        owners.find(|&o| world.agent(o).is_some_and(|x| x.caches.contains_key(&site)))
    };
    if !forget(world, id, site) {
        return None;
    }
    let Some(owner) = target else {
        world.events.raids_wasted += 1;
        return None;
    };
    let take = super::theft::loot(world, owner, id, site, room);
    if take <= 0.0 {
        return None;
    }
    world.events.raids += 1;
    world.events.raided += take;
    Some(Harvest {
        pilfered: take,
        ..Harvest::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Wall};
    use crate::geometry::Pos;
    use crate::minds::caching::bury;
    use crate::rules::movement::{candidates, candidates_with_memory, go_and_gather};
    use crate::testkit::*;
    use rand::{Rng, SeedableRng};

    fn at(w: &World, x: u32, y: u32) -> u32 {
        w.torus.index(Pos::new(x, y)) as u32
    }

    /// A world from `c` with watching on (set after validation, which
    /// would want caching on; these tests bury by hand).
    fn watching(c: Config) -> World {
        let mut w = World::new(c, 7).unwrap();
        w.config.watching.on = true;
        w
    }

    /// An agent at (x, y) with `vision`, who watches or not.
    fn put(w: &mut World, x: u32, y: u32, vision: u32, watches: bool) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.vision = vision;
        a.watches = watches;
        a.holdings[0] = 100.0;
        id
    }

    fn entry(w: &World, watcher: AgentId, site: u32, owner: AgentId) -> Option<SeenCache> {
        w.agent(watcher).unwrap().seen.get(&(site, owner)).copied()
    }

    fn wall(x: u32, y: u32, opaque: bool) -> Wall {
        Wall {
            x,
            y,
            width: 1,
            height: 1,
            opaque,
        }
    }

    #[test]
    fn watchers_on_each_lattice_line_see_a_burial() {
        let mut w = watching(blank_config(11, 11));
        let owner = put(&mut w, 5, 5, 1, false);
        let north = put(&mut w, 5, 2, 3, true);
        let east = put(&mut w, 7, 5, 3, true);
        let south = put(&mut w, 5, 6, 3, true);
        let west = put(&mut w, 1, 5, 4, true);
        let diagonal = put(&mut w, 6, 6, 5, true);
        assert_eq!(bury(&mut w, owner, 4.0), 4.0);
        let s = at(&w, 5, 5);
        for id in [north, east, south, west] {
            assert_eq!(
                entry(&w, id, s, owner),
                Some(SeenCache {
                    amount: 4.0,
                    tick: 0
                }),
                "{id}"
            );
            assert_eq!(w.agent(id).unwrap().seen.len(), 1);
        }
        assert!(w.agent(diagonal).unwrap().seen.is_empty());
        assert_eq!((w.events.burials_seen, w.events.sightings), (1, 4));
    }

    #[test]
    fn a_watcher_sees_at_exactly_its_vision_and_not_one_beyond() {
        let mut w = watching(blank_config(15, 15));
        let owner = put(&mut w, 7, 7, 1, false);
        let edge = put(&mut w, 7, 4, 3, true);
        let beyond = put(&mut w, 11, 7, 3, true);
        bury(&mut w, owner, 2.0);
        assert!(entry(&w, edge, at(&w, 7, 7), owner).is_some());
        assert!(w.agent(beyond).unwrap().seen.is_empty());
        assert_eq!((w.events.burials_seen, w.events.sightings), (1, 1));
    }

    #[test]
    fn an_opaque_wall_blocks_the_sighting_and_a_fence_does_not() {
        let mut c = blank_config(11, 11);
        c.walls = vec![wall(5, 3, true), wall(7, 5, false)];
        let mut w = watching(c);
        let owner = put(&mut w, 5, 5, 1, false);
        let behind_wall = put(&mut w, 5, 2, 4, true);
        let behind_fence = put(&mut w, 8, 5, 4, true);
        bury(&mut w, owner, 2.0);
        assert!(w.agent(behind_wall).unwrap().seen.is_empty());
        assert!(entry(&w, behind_fence, at(&w, 5, 5), owner).is_some());
    }

    #[test]
    fn an_owner_never_sees_its_own_burial() {
        let mut w = watching(blank_config(11, 11));
        let owner = put(&mut w, 5, 5, 3, true);
        bury(&mut w, owner, 2.0);
        assert!(w.agent(owner).unwrap().seen.is_empty());
        assert_eq!((w.events.burials_seen, w.events.sightings), (0, 0));
    }

    #[test]
    fn non_watchers_never_see() {
        let mut w = watching(blank_config(11, 11));
        let owner = put(&mut w, 5, 5, 1, false);
        let other = put(&mut w, 5, 4, 3, false);
        bury(&mut w, owner, 2.0);
        assert!(w.agent(other).unwrap().seen.is_empty());
        assert_eq!((w.events.burials_seen, w.events.sightings), (0, 0));
    }

    #[test]
    fn nothing_is_seen_with_watching_off() {
        let mut w = World::new(blank_config(11, 11), 7).unwrap();
        let owner = put(&mut w, 5, 5, 1, false);
        let watcher = put(&mut w, 5, 4, 3, true);
        bury(&mut w, owner, 2.0);
        assert!(w.agent(watcher).unwrap().seen.is_empty());
        assert_eq!((w.events.burials_seen, w.events.sightings), (0, 0));
    }

    #[test]
    fn a_second_seen_burial_adds_and_refreshes_and_an_unseen_one_does_not() {
        let mut w = watching(blank_config(11, 11));
        let owner = put(&mut w, 5, 5, 1, false);
        let watcher = put(&mut w, 5, 4, 3, true);
        let s = at(&w, 5, 5);
        bury(&mut w, owner, 4.0);
        w.tick = 3;
        bury(&mut w, owner, 2.0);
        assert_eq!(
            entry(&w, watcher, s, owner),
            Some(SeenCache {
                amount: 6.0,
                tick: 3
            })
        );
        // Out of sight now: a third burial leaves the entry as it was.
        w.move_agent(watcher, Pos::new(1, 1));
        w.tick = 5;
        bury(&mut w, owner, 1.0);
        assert_eq!(
            entry(&w, watcher, s, owner),
            Some(SeenCache {
                amount: 6.0,
                tick: 3
            })
        );
        assert_eq!(w.agent(owner).unwrap().caches[&s], 7.0);
    }

    #[test]
    fn an_entry_is_kept_at_exactly_span_and_gone_at_span_plus_1() {
        let mut w = watching(blank_config(11, 11));
        w.config.watching.span = 7;
        let owner = put(&mut w, 5, 5, 1, false);
        let watcher = put(&mut w, 5, 4, 3, true);
        bury(&mut w, owner, 4.0);
        w.tick = 7;
        assert!(fresh(&w, 0));
        sweep(&mut w);
        assert_eq!(w.agent(watcher).unwrap().seen.len(), 1);
        assert_eq!(w.events.seen_entries, 1);
        w.tick = 8;
        assert!(!fresh(&w, 0));
        sweep(&mut w);
        assert!(w.agent(watcher).unwrap().seen.is_empty());
        assert_eq!(w.events.seen_entries, 0);
    }

    #[test]
    fn the_tick_start_sweep_counts_entries_and_forgets_by_age() {
        let mut w = watching(blank_config(11, 11));
        w.config.watching.span = 2;
        let owner = put(&mut w, 5, 5, 1, false);
        let watcher = put(&mut w, 5, 4, 3, true);
        bury(&mut w, owner, 4.0);
        // Steps 1 and 2 start at ticks 0 and 1 (ages 0, 1): kept.
        w.step();
        w.step();
        assert_eq!(w.events.seen_entries, 1);
        w.step(); // starts at tick 2: age 2 = span, kept
        assert_eq!(w.events.seen_entries, 1);
        w.step(); // tick 3: gone
        assert_eq!(w.events.seen_entries, 0);
        assert!(w.agent(watcher).unwrap().seen.is_empty());
    }

    #[test]
    fn entries_kept_while_off_are_forgotten_once_on_and_a_shorter_span_forgets_live() {
        let mut w = watching(blank_config(11, 11));
        let owner = put(&mut w, 5, 5, 1, false);
        let watcher = put(&mut w, 5, 4, 3, true);
        bury(&mut w, owner, 4.0);
        w.config.watching.on = false;
        for _ in 0..20 {
            w.step();
            assert_eq!(w.events.seen_entries, 0);
        }
        // No sweep while off: the old entry is still held.
        assert_eq!(w.agent(watcher).unwrap().seen.len(), 1);
        w.config.watching.on = true;
        w.step();
        assert!(w.agent(watcher).unwrap().seen.is_empty());

        // Shortening `span` live forgets the older entries on their next read.
        let s = w.agent(owner).unwrap().pos;
        w.move_agent(watcher, Pos::new(s.x, (s.y + 10) % 11));
        let t0 = w.tick;
        bury(&mut w, owner, 1.0);
        assert_eq!(w.agent(watcher).unwrap().seen.len(), 1);
        w.tick = t0 + 3;
        w.config.watching.span = 7;
        w.step();
        assert_eq!(w.events.seen_entries, 1);
        w.config.watching.span = 2;
        w.step();
        assert_eq!(w.events.seen_entries, 0);
        assert!(w.agent(watcher).unwrap().seen.is_empty());
    }

    #[test]
    fn a_dead_watchers_entries_go_with_it_and_a_dead_owners_stay() {
        let mut w = watching(blank_config(11, 11));
        let owner = put(&mut w, 5, 5, 1, false);
        let a = put(&mut w, 5, 4, 3, true);
        let b = put(&mut w, 4, 5, 3, true);
        bury(&mut w, owner, 4.0);
        w.remove(a);
        sweep(&mut w);
        assert_eq!(w.events.seen_entries, 1);
        w.remove(owner);
        sweep(&mut w);
        assert_eq!(w.events.seen_entries, 1);
        assert!(entry(&w, b, at(&w, 5, 5), owner).is_some());
    }

    /// The walk out from the site gives exactly the watchers whose
    /// `World::sight` covers it, on random small walled tori, including ones
    /// small enough (and visions long enough) that the lines wrap.
    #[test]
    fn the_walk_matches_world_sight_on_random_walled_tori() {
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(8);
        let mut checked = 0;
        for trial in 0..400 {
            let (wd, ht) = (rng.gen_range(2..=8u32), rng.gen_range(2..=8u32));
            let mut c = blank_config(wd, ht);
            for _ in 0..rng.gen_range(0..=(wd * ht / 3)) {
                let (x, y) = (rng.gen_range(0..wd), rng.gen_range(0..ht));
                c.walls.push(wall(x, y, rng.gen_bool(0.5)));
            }
            let Ok(mut w) = World::new(c, trial) else {
                continue;
            };
            for _ in 0..rng.gen_range(1..=(wd * ht)) {
                let p = Pos::new(rng.gen_range(0..wd), rng.gen_range(0..ht));
                if w.is_wall(p) || w.occupant(p).is_some() {
                    continue;
                }
                let v = rng.gen_range(0..=(wd.max(ht) + 2));
                put(&mut w, p.x, p.y, v, rng.gen_bool(0.7));
            }
            let ids: Vec<AgentId> = w.agents().map(|a| a.id).collect();
            for &owner in &ids {
                let pos = w.agent(owner).unwrap().pos;
                let site = w.torus.index(pos) as u32;
                let expected: BTreeSet<AgentId> = w
                    .agents()
                    .filter(|a| a.watches && a.id != owner)
                    .filter(|a| w.sight(a.pos, a.vision).iter().any(|&(q, _)| q == pos))
                    .map(|a| a.id)
                    .collect();
                assert_eq!(watchers_of(&w, owner, site), expected, "trial {trial}");
                checked += expected.len();
            }
        }
        assert!(checked > 100, "{checked}");
    }

    /// Seeing draws nothing and moves no sugar: a caching world steps the
    /// same with watching on as off until some agent remembers a cache
    /// (from then on its candidates and raids may differ). With it off
    /// nothing is seen or allocated.
    #[test]
    fn watching_changes_nothing_until_a_cache_is_remembered_and_off_allocates_nothing() {
        let c = crate::presets::by_id("cache-winter-mixed").unwrap().config;
        let mut off = World::new(c.clone(), 3).unwrap();
        let mut on = c;
        on.watching.on = true;
        let mut on = World::new(on, 3).unwrap();
        let (mut buried, mut sightings, mut raids, mut same) = (0.0, 0, 0, 0);
        for _ in 0..150 {
            let remembered = on.agents().any(|a| !a.seen.is_empty());
            off.step();
            on.step();
            if !remembered {
                assert_eq!(off.fingerprint(), on.fingerprint());
                same += 1;
            }
            assert_eq!(
                (
                    off.events.burials_seen,
                    off.events.sightings,
                    off.events.seen_entries,
                    off.events.seen_arrivals,
                    off.events.raids_wasted,
                ),
                (0, 0, 0, 0, 0)
            );
            assert_eq!((off.events.raids, off.events.raided), (0, 0.0));
            assert!(off.agents().all(|a| a.seen.is_empty()));
            buried += off.events.buried;
            sightings += on.events.sightings;
            raids += on.events.raids;
        }
        assert!(buried > 0.0 && sightings > 0, "{buried} {sightings}");
        assert!(same > 0 && raids > 0, "{same} {raids}");
    }

    // Raids.

    fn total(w: &World) -> f64 {
        let sites: f64 = w.sites.iter().map(|s| s.resource[0]).sum();
        let held: f64 = w.agents().map(|a| a.holdings[0]).sum();
        let cached: f64 = w.agents().flat_map(|a| a.caches.values()).sum();
        let fed: f64 = w.agents().map(|a| a.fed).sum();
        sites + held + cached + fed
    }

    /// An agent at (x, y) with metabolism 1 (R = 10), `held` sugar and
    /// `vision`, who watches or not.
    fn walker(w: &mut World, x: u32, y: u32, held: f64, vision: u32, watches: bool) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.holdings[0] = held;
        a.vision = vision;
        a.watches = watches;
        id
    }

    /// An 11×11 world under watching (`find`, carrying limit `capacity`),
    /// recording fates: an owner buried `cached` at (5, 6) while a watcher
    /// at (5, 4) holding `held` saw it, then walked off to (1, 1). The site
    /// holds sugar 2.
    fn seen_world(find: f64, capacity: u32, cached: f64, held: f64) -> (World, AgentId, AgentId) {
        let mut w = watching(blank_config(11, 11));
        w.record_fates = true;
        w.config.theft.find = find;
        w.config.caching.capacity = capacity;
        let owner = walker(&mut w, 5, 6, cached.max(100.0), 1, false);
        let watcher = walker(&mut w, 5, 4, held, 3, true);
        bury(&mut w, owner, cached);
        assert_eq!(w.agent(watcher).unwrap().seen.len(), 1);
        w.move_agent(owner, Pos::new(1, 1));
        set_sugar(&mut w, 5, 6, 2.0);
        (w, owner, watcher)
    }

    /// A second owner's cache of `q` at (5, 6), buried out of the watcher's
    /// sight, the owner then at (2, 2).
    fn unseen_cache(w: &mut World, watcher: AgentId, q: f64) -> AgentId {
        let back = w.agent(watcher).unwrap().pos;
        w.move_agent(watcher, Pos::new(0, 9));
        let other = walker(w, 5, 6, 50.0, 1, false);
        bury(w, other, q);
        w.move_agent(other, Pos::new(2, 2));
        w.move_agent(watcher, back);
        assert_eq!(w.agent(watcher).unwrap().seen.len(), 1, "unseen");
        other
    }

    fn raid_counts(w: &World) -> (u32, f64, u32, u32) {
        let e = &w.events;
        (e.raids, e.raided, e.raids_wasted, e.seen_arrivals)
    }

    fn pilfered_by(w: &World, by: AgentId) -> f64 {
        use crate::minds::caching::fates::Fate;
        w.cache_log
            .iter()
            .filter(|r| r.fate.is_some_and(|(_, f)| f == Fate::Pilfered { by }))
            .map(|r| r.amount)
            .sum()
    }

    #[test]
    fn a_seen_cache_still_there_is_always_taken() {
        for find in [0.0, 1e-12] {
            let (mut w, owner, watcher) = seen_world(find, 0, 6.0, 3.0);
            let here = at(&w, 5, 6);
            let (rng, before) = (w.rng.clone(), total(&w));
            let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
            assert_eq!(
                (h.gathered[0], h.dug, h.pilfered),
                (0.0, 0.0, 6.0),
                "{find}"
            );
            assert_eq!(w.agent(watcher).unwrap().holdings[0], 9.0, "kept");
            assert_eq!(w.site(Pos::new(5, 6)).resource[0], 2.0, "site untouched");
            assert!(!w.agent(owner).unwrap().caches.contains_key(&here));
            assert_eq!(raid_counts(&w), (1, 6.0, 0, 1));
            assert_eq!((w.events.pilfers, w.events.pilfered), (1, 6.0));
            assert_eq!(w.events.pilfer_draws, 0, "no stumble after a take");
            assert!(w.agent(watcher).unwrap().seen.is_empty(), "forgotten");
            assert_eq!(rng, w.rng, "no draw");
            assert_eq!(total(&w), before);
            assert_eq!(w.agent(watcher).unwrap().stolen_by_me, 6.0);
            assert_eq!(w.agent(owner).unwrap().stolen_from_me, 6.0);
        }
    }

    #[test]
    fn eaten_raid_loot_goes_into_the_stomach() {
        let (mut w, _, watcher) = seen_world(0.0, 10, 6.0, 9.0);
        w.config.theft.loot = crate::config::Loot::Eat;
        let before = total(&w);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0, "the whole cache, though room is 1");
        let t = w.agent(watcher).unwrap();
        assert_eq!((t.holdings[0], t.fed), (9.0, 6.0));
        assert_eq!(w.events.loot_eaten, 6.0);
        assert_eq!(total(&w), before);
    }

    #[test]
    fn an_unseen_cache_is_taken_only_through_find() {
        let (mut w, owner, _) = seen_world(0.0, 0, 6.0, 3.0);
        let here = at(&w, 5, 6);
        let blind = walker(&mut w, 9, 9, 3.0, 1, true);
        let h = go_and_gather(&mut w, blind, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
        assert_eq!(w.agent(owner).unwrap().caches[&here], 6.0);
        assert_eq!(raid_counts(&w), (0, 0.0, 0, 0));
        assert_eq!(w.events.pilfers, 0);
        // With find 1 it stumbles on it: a pilfer but no raid.
        w.move_agent(blind, Pos::new(9, 9));
        w.config.theft.find = 1.0;
        let h = go_and_gather(&mut w, blind, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0);
        assert_eq!(raid_counts(&w), (0, 0.0, 0, 0));
        assert_eq!((w.events.pilfers, w.events.pilfer_draws), (1, 1));
    }

    #[test]
    fn the_dig_wins_over_a_raid() {
        let (mut w, owner, watcher) = seen_world(0.0, 0, 5.0, 10.0);
        let here = at(&w, 5, 6);
        // The watcher's own cache of 8 at the same site; holdings 2 < R / 2.
        w.move_agent(watcher, Pos::new(5, 6));
        bury(&mut w, watcher, 8.0);
        w.move_agent(watcher, Pos::new(5, 5));
        let before = total(&w);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!((h.dug, h.pilfered, h.gathered[0]), (8.0, 0.0, 0.0));
        assert_eq!(w.agent(owner).unwrap().caches[&here], 5.0, "not raided");
        assert_eq!(raid_counts(&w), (0, 0.0, 0, 1));
        assert!(
            w.agent(watcher).unwrap().seen.is_empty(),
            "arrived: forgotten"
        );
        assert_eq!(total(&w), before);
    }

    #[test]
    fn one_take_per_arrival_with_a_raid_and_a_stumble_possible() {
        let (mut w, owner, watcher) = seen_world(1.0, 0, 6.0, 3.0);
        let here = at(&w, 5, 6);
        let other = unseen_cache(&mut w, watcher, 4.0);
        let rng = w.rng.clone();
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0, "the seen cache");
        assert!(!w.agent(owner).unwrap().caches.contains_key(&here));
        assert_eq!(w.agent(other).unwrap().caches[&here], 4.0);
        assert_eq!((w.events.pilfers, w.events.pilfer_draws), (1, 0));
        assert_eq!(rng, w.rng, "no find draw");
    }

    #[test]
    fn a_raid_takes_the_first_remembered_owner_still_there() {
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        let here = at(&w, 5, 6);
        // A second owner, seen too, with a larger id.
        let second = walker(&mut w, 5, 6, 50.0, 1, false);
        bury(&mut w, second, 4.0);
        w.move_agent(second, Pos::new(2, 2));
        assert_eq!(w.agent(watcher).unwrap().seen.len(), 2);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0, "the lower id first");
        assert!(!w.agent(owner).unwrap().caches.contains_key(&here));
        assert_eq!(w.agent(second).unwrap().caches[&here], 4.0);
        assert!(w.agent(watcher).unwrap().seen.is_empty(), "all forgotten");
        assert_eq!(raid_counts(&w), (1, 6.0, 0, 1));
        // The first gone, the next remembered owner's is taken.
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        let second = walker(&mut w, 5, 6, 50.0, 1, false);
        bury(&mut w, second, 4.0);
        w.move_agent(second, Pos::new(2, 2));
        crate::minds::caching::dig(&mut w, owner, here, f64::INFINITY);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!(h.pilfered, 4.0);
        assert_eq!(raid_counts(&w), (1, 4.0, 0, 1));
    }

    #[test]
    fn a_wasted_raid_falls_through_to_stumbling() {
        let (mut w, owner, watcher) = seen_world(1.0, 0, 6.0, 3.0);
        let here = at(&w, 5, 6);
        let other = unseen_cache(&mut w, watcher, 4.0);
        crate::minds::caching::dig(&mut w, owner, here, f64::INFINITY);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!(h.pilfered, 4.0, "found by stumbling");
        assert!(w.agent(other).unwrap().caches.is_empty());
        assert_eq!(raid_counts(&w), (0, 0.0, 1, 1));
        assert_eq!((w.events.pilfers, w.events.pilfer_draws), (1, 1));
        assert!(w.agent(watcher).unwrap().seen.is_empty());
        // With find 0 a wasted raid harvests the site.
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        crate::minds::caching::dig(&mut w, owner, here, f64::INFINITY);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
        assert_eq!(raid_counts(&w), (0, 0.0, 1, 1));
    }

    // Review Focus 1.
    #[test]
    fn a_watcher_with_no_room_takes_nothing_and_harvests_as_usual() {
        let (mut w, owner, watcher) = seen_world(0.0, 10, 6.0, 10.0);
        let here = at(&w, 5, 6);
        let before = total(&w);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (0.0, 0.0));
        assert_eq!(
            w.agent(owner).unwrap().caches[&here],
            6.0,
            "still the owner's"
        );
        assert_eq!(w.site(Pos::new(5, 6)).resource[0], 2.0, "full: no room");
        assert!(w.agent(watcher).unwrap().seen.is_empty(), "forgotten");
        assert_eq!(raid_counts(&w), (0, 0.0, 0, 1), "neither a raid nor wasted");
        assert_eq!(w.events.pilfers, 0);
        assert_eq!(total(&w), before);
        // With room 1 it takes 1, and the rest stays the owner's.
        let (mut w, owner, watcher) = seen_world(0.0, 10, 6.0, 9.0);
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!(h.pilfered, 1.0);
        assert_eq!(w.agent(owner).unwrap().caches[&here], 5.0);
        assert_eq!(raid_counts(&w), (1, 1.0, 0, 1));
    }

    // Review Focus 2.
    #[test]
    fn an_owner_and_a_watcher_on_one_cache_in_one_tick() {
        // The owner digs it all first: the watcher's raid is wasted.
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        w.agent_mut(owner).unwrap().holdings[0] = 2.0;
        w.move_agent(owner, Pos::new(5, 7));
        let before = total(&w);
        assert_eq!(go_and_gather(&mut w, owner, Pos::new(5, 6)).dug, 6.0);
        w.move_agent(owner, Pos::new(5, 7));
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
        assert_eq!(raid_counts(&w), (0, 0.0, 1, 1));
        assert_eq!(total(&w), before);
        // A partial dig (room 4 of 10) leaves the rest to the raid.
        let (mut w, owner, watcher) = seen_world(0.0, 6, 10.0, 0.0);
        w.tick = 1;
        w.agent_mut(owner).unwrap().holdings[0] = 2.0;
        w.move_agent(owner, Pos::new(5, 7));
        let before = total(&w);
        assert_eq!(go_and_gather(&mut w, owner, Pos::new(5, 6)).dug, 4.0);
        w.move_agent(owner, Pos::new(5, 7));
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!(h.pilfered, 6.0);
        assert!(w.agent(owner).unwrap().caches.is_empty());
        assert_eq!(raid_counts(&w), (1, 6.0, 0, 1));
        assert_eq!(
            (w.events.digs, w.events.pilfers, w.events.caches_pilfered),
            (1, 1, 1)
        );
        assert_eq!(total(&w), before);
        // The fate log runs under watching alone (find 0, no cheaters).
        assert_eq!(pilfered_by(&w, watcher), 6.0);
        assert!(w.cache_open.is_empty());
    }

    // Review Focus 3.
    #[test]
    fn a_seen_cache_of_a_dead_owner_gives_a_wasted_raid() {
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        w.kill(owner, crate::world::DeathCause::OldAge);
        assert_eq!(w.agent(watcher).unwrap().seen.len(), 1, "kept until read");
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
        assert_eq!(raid_counts(&w), (0, 0.0, 1, 1));
        assert!(w.agent(watcher).unwrap().seen.is_empty());
        // A dead watcher takes its entries with it: none is held after.
        let (mut w, _, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        sweep(&mut w);
        assert_eq!(w.events.seen_entries, 1);
        w.kill(watcher, crate::world::DeathCause::OldAge);
        sweep(&mut w);
        assert_eq!(w.events.seen_entries, 0);
        assert!(w.agents().all(|a| a.seen.is_empty()));
    }

    #[test]
    fn a_stale_entry_is_no_candidate_and_no_raid() {
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        w.move_agent(watcher, Pos::new(0, 0));
        w.tick = 8;
        assert_eq!(seen_value(&w, watcher, Pos::new(5, 6)), None);
        assert!(!candidates(&w, watcher)
            .iter()
            .any(|c| c.0 == Pos::new(5, 6)));
        let h = go_and_gather(&mut w, watcher, Pos::new(5, 6));
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
        assert_eq!(raid_counts(&w), (0, 0.0, 0, 0));
        assert!(
            w.agent(watcher).unwrap().seen.is_empty(),
            "forgotten all the same"
        );
        assert_eq!(w.agent(owner).unwrap().caches[&at(&w, 5, 6)], 6.0);
    }

    #[test]
    fn a_seen_cache_joins_after_own_caches_at_its_remembered_amount() {
        let (mut w, _, watcher) = seen_world(0.0, 0, 6.0, 8.0);
        let target = Pos::new(5, 6);
        // Out of sight of (5, 6), with a second owner's seen cache there.
        let second = walker(&mut w, 5, 6, 50.0, 1, false);
        bury(&mut w, second, 4.0);
        w.move_agent(second, Pos::new(2, 2));
        w.move_agent(watcher, Pos::new(0, 0));
        assert_eq!(seen_value(&w, watcher, target), Some(10.0), "summed");
        let (list, start) = candidates_with_memory(&w, watcher);
        let d = crate::rules::movement::lattice_distance(w.torus, Pos::new(0, 0), target);
        assert_eq!(list[start - 1], (target, d, 10.0));
        assert_eq!(list.iter().filter(|c| c.0 == target).count(), 1);
        // Its own (hungry) cache elsewhere comes first.
        w.move_agent(watcher, Pos::new(8, 8));
        bury(&mut w, watcher, 4.0);
        w.move_agent(watcher, Pos::new(0, 0));
        assert!(crate::minds::caching::hungry(&w, watcher));
        let (list, start) = candidates_with_memory(&w, watcher);
        assert_eq!(list[start - 2].0, Pos::new(8, 8));
        assert_eq!(list[start - 1].0, target);
        // An occupied site isn't a place to go.
        w.move_agent(second, target);
        assert!(!candidates(&w, watcher).iter().any(|c| c.0 == target));
        // Watching off: no candidate, no value.
        w.move_agent(second, Pos::new(2, 2));
        w.config.watching.on = false;
        assert!(!candidates(&w, watcher).iter().any(|c| c.0 == target));
        assert_eq!(seen_value(&w, watcher, target), None);
    }

    /// Minds 3's diagnostics value a chosen seen-cache site at what is
    /// truly left there, not the watcher's belief: a choice of a cache
    /// emptied since is stale, a still-full one isn't.
    #[test]
    fn the_diagnostics_value_a_seen_cache_at_what_is_truly_left() {
        let target = Pos::new(5, 6);
        for emptied in [false, true] {
            let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
            w.config.memory.span = 5;
            w.agent_mut(watcher).unwrap().remembers = true;
            w.move_agent(watcher, Pos::new(0, 0));
            if emptied {
                let here = at(&w, 5, 6);
                crate::minds::caching::dig(&mut w, owner, here, f64::INFINITY);
            }
            let truth = if emptied { 0.0 } else { 6.0 };
            assert_eq!(seen_truth(&w, watcher, target), Some(truth));
            assert_eq!(seen_value(&w, watcher, target), Some(6.0), "believed");
            // Chosen as a remembered entry believed worth 6.
            let list = [(target, 5, 6.0)];
            crate::rules::movement::record_choice(&mut w, watcher, &list, 0, target);
            let e = &w.events;
            assert_eq!(e.remembered_moves, 1);
            if emptied {
                // The site's sugar 2 is all that's left: off by 4.
                assert_eq!((e.stale_choices, e.belief_error_sum), (1, 4.0));
            } else {
                assert_eq!((e.stale_choices, e.belief_error_sum), (0, 0.0));
            }
        }
        // A dead owner's cache is truly gone too; watching off, no value.
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        w.kill(owner, crate::world::DeathCause::OldAge);
        assert_eq!(seen_truth(&w, watcher, target), Some(0.0));
        w.config.watching.on = false;
        assert_eq!(seen_truth(&w, watcher, target), None);
    }

    #[test]
    fn raid_when_hungry_lists_seen_caches_only_below_half_the_reserve() {
        let (mut w, _, watcher) = seen_world(0.0, 0, 6.0, 8.0);
        let target = Pos::new(5, 6);
        w.move_agent(watcher, Pos::new(0, 0));
        assert!(w.agent(watcher).unwrap().caches.is_empty());
        let listed = |w: &World| candidates(w, watcher).iter().any(|c| c.0 == target);
        assert!(listed(&w), "always: fed, listed");
        w.config.watching.raid_when = crate::config::RaidWhen::Hungry;
        assert!(!listed(&w), "hungry: fed (8 ≥ R / 2 = 5), not listed");
        assert_eq!(seen_value(&w, watcher, target), None);
        w.agent_mut(watcher).unwrap().holdings[0] = 4.0;
        assert!(listed(&w), "below R / 2, with no caches of its own");
        assert_eq!(seen_value(&w, watcher, target), Some(6.0));
    }

    /// Controller ruling (Task 5b): under `raid_when: hungry` the raid on
    /// arrival has the joining gate too. A fed watcher arriving takes
    /// nothing, keeps its entry, counts nothing and harvests (and stumbles)
    /// as if it had none; a hungry one takes; under `always` a fed one takes.
    #[test]
    fn raid_when_hungry_gates_the_raid_too() {
        use crate::config::RaidWhen;
        let target = Pos::new(5, 6);
        // Fed (8 ≥ R / 2 = 5) under hungry, with an unseen cache to stumble on.
        let (mut w, owner, watcher) = seen_world(1e-12, 0, 6.0, 8.0);
        let other = unseen_cache(&mut w, watcher, 3.0);
        w.config.watching.raid_when = RaidWhen::Hungry;
        let here = at(&w, 5, 6);
        let (before, seen) = (total(&w), entry(&w, watcher, here, owner));
        let h = go_and_gather(&mut w, watcher, target);
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0), "harvests");
        assert_eq!(w.agent(owner).unwrap().caches.get(&here), Some(&6.0));
        assert_eq!(w.agent(other).unwrap().caches.get(&here), Some(&3.0));
        assert_eq!(entry(&w, watcher, here, owner), seen, "kept");
        assert_eq!(raid_counts(&w), (0, 0.0, 0, 0), "counts nothing");
        assert_eq!(w.events.pilfer_draws, 2, "stumbles as with no entries");
        assert_eq!(total(&w), before);
        // Hungry (4 < 5): takes it.
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 4.0);
        w.config.watching.raid_when = RaidWhen::Hungry;
        let h = go_and_gather(&mut w, watcher, target);
        assert_eq!((h.gathered[0], h.pilfered), (0.0, 6.0));
        assert!(!w.agent(owner).unwrap().caches.contains_key(&here));
        assert_eq!(raid_counts(&w), (1, 6.0, 0, 1));
        assert!(w.agent(watcher).unwrap().seen.is_empty());
        // Always: a fed one takes it.
        let (mut w, _, watcher) = seen_world(0.0, 0, 6.0, 8.0);
        assert_eq!(w.config.watching.raid_when, RaidWhen::Always);
        assert_eq!(go_and_gather(&mut w, watcher, target).pilfered, 6.0);
        assert_eq!(raid_counts(&w), (1, 6.0, 0, 1));
    }

    /// A walking watcher at (5, 1) (vision 1) remembering a cache of 6 at
    /// (5, 6), which four other agents seal off, and one of 4 at (8, 1),
    /// reachable. No sugar anywhere.
    fn sealed_world() -> (World, AgentId, AgentId) {
        use crate::config::{MoveMode, Movement};
        let mut w = watching(blank_config(11, 11));
        w.config.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        let owner = walker(&mut w, 0, 0, 100.0, 1, false);
        for (x, y, q) in [(5, 6, 6.0), (8, 1, 4.0)] {
            let site = at(&w, x, y);
            w.agent_mut(owner).unwrap().caches.insert(site, q);
        }
        let watcher = walker(&mut w, 5, 1, 3.0, 1, true);
        for (x, y, q) in [(5, 6, 6.0), (8, 1, 4.0)] {
            let site = at(&w, x, y);
            let e = SeenCache { amount: q, tick: 0 };
            w.agent_mut(watcher).unwrap().seen.insert((site, owner), e);
        }
        for (x, y) in [(5, 5), (5, 7), (4, 6), (6, 6)] {
            walker(&mut w, x, y, 100.0, 1, false);
        }
        (w, owner, watcher)
    }

    fn lists(w: &World, id: AgentId, p: Pos) -> bool {
        candidates(w, id).iter().any(|c| c.0 == p)
    }

    /// Controller ruling (Task 5b): a walk that finds no path to a seen
    /// cache gives it up, so the watcher doesn't target it again.
    #[test]
    fn a_seen_cache_sealed_off_is_given_up_after_a_failed_walk() {
        use crate::agent::Plan;
        use crate::rules::movement::arrive;
        let (mut w, owner, watcher) = sealed_world();
        let (sealed, open) = (Pos::new(5, 6), Pos::new(8, 1));
        assert!(lists(&w, watcher, sealed) && lists(&w, watcher, open));
        let h = arrive(&mut w, watcher, sealed);
        assert_eq!(w.agent(watcher).unwrap().pos, Pos::new(5, 1), "stays");
        assert_eq!(h.pilfered, 0.0);
        let s = at(&w, 5, 6);
        assert_eq!(entry(&w, watcher, s, owner), None, "given up");
        assert!(entry(&w, watcher, at(&w, 8, 1), owner).is_some());
        assert_eq!(raid_counts(&w), (0, 0.0, 0, 0), "counts nothing");
        // Not a candidate on later ticks, once the failed target is no
        // longer the last one.
        w.agent_mut(watcher).unwrap().plan = Plan::default();
        assert!(!lists(&w, watcher, sealed));
        assert!(lists(&w, watcher, open), "the reachable one is still there");
        for _ in 0..3 {
            w.tick += 1;
            crate::rules::movement::act(&mut w, watcher);
            assert_ne!(w.agent(watcher).unwrap().plan.target, Some(sealed));
        }
    }

    #[test]
    fn a_walk_toward_a_reachable_seen_cache_keeps_it() {
        use crate::rules::movement::arrive;
        let (mut w, owner, watcher) = sealed_world();
        arrive(&mut w, watcher, Pos::new(8, 1));
        assert_eq!(w.agent(watcher).unwrap().pos, Pos::new(6, 1), "one step");
        assert!(entry(&w, watcher, at(&w, 8, 1), owner).is_some());
        assert!(entry(&w, watcher, at(&w, 5, 6), owner).is_some());
    }

    #[test]
    fn a_seen_cache_walled_apart_is_given_up_and_never_targeted() {
        use crate::config::{MoveMode, Movement};
        use crate::rules::movement::arrive;
        let mut c = blank_config(11, 11);
        c.walls = vec![
            Wall {
                x: 3,
                y: 0,
                width: 1,
                height: 11,
                opaque: false,
            },
            Wall {
                x: 8,
                y: 0,
                width: 1,
                height: 11,
                opaque: false,
            },
        ];
        let mut w = watching(c);
        w.config.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        let owner = walker(&mut w, 5, 0, 100.0, 1, false);
        let watcher = walker(&mut w, 1, 5, 3.0, 1, true);
        let (target, s) = (Pos::new(5, 5), at(&w, 5, 5));
        w.agent_mut(owner).unwrap().caches.insert(s, 6.0);
        let e = SeenCache {
            amount: 6.0,
            tick: 0,
        };
        w.agent_mut(watcher).unwrap().seen.insert((s, owner), e);
        assert!(w.walled_apart(Pos::new(1, 5), target));
        assert!(!lists(&w, watcher, target), "never a candidate");
        arrive(&mut w, watcher, target);
        assert_eq!(entry(&w, watcher, s, owner), None, "given up");
    }

    /// Survey probe `World::probe_raid_harvests`: a raid that took
    /// something also harvests the site that tick, under the carrying limit.
    #[test]
    fn under_the_probe_a_raid_also_harvests_the_site() {
        let target = Pos::new(5, 6);
        // Off (the default): the raid replaces the harvest.
        let (mut w, _, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        assert!(!w.probe_raid_harvests);
        let h = go_and_gather(&mut w, watcher, target);
        assert_eq!((h.gathered[0], h.pilfered), (0.0, 6.0));
        assert_eq!(w.site(target).resource[0], 2.0);
        // On: it gathers the site too, and no stumble follows.
        let (mut w, _, watcher) = seen_world(1e-12, 0, 6.0, 3.0);
        unseen_cache(&mut w, watcher, 3.0);
        w.probe_raid_harvests = true;
        let (rng, before) = (w.rng.clone(), total(&w));
        let h = go_and_gather(&mut w, watcher, target);
        assert_eq!((h.gathered[0], h.dug, h.pilfered), (2.0, 0.0, 6.0));
        assert_eq!(w.agent(watcher).unwrap().holdings[0], 11.0);
        assert_eq!(w.site(target).resource[0], 0.0);
        assert_eq!(raid_counts(&w), (1, 6.0, 0, 1));
        assert_eq!((w.events.pilfers, w.events.pilfer_draws), (1, 0));
        assert_eq!(rng, w.rng, "no draw");
        assert_eq!(total(&w), before);
        // Under the limit C = 10: holds 3, takes 6, so room 1 for the site's
        // 2; the other 1 stays on it.
        let (mut w, _, watcher) = seen_world(0.0, 10, 6.0, 3.0);
        w.probe_raid_harvests = true;
        let before = total(&w);
        let h = go_and_gather(&mut w, watcher, target);
        assert_eq!((h.gathered[0], h.pilfered), (1.0, 6.0));
        assert_eq!(w.agent(watcher).unwrap().holdings[0], 10.0);
        assert_eq!(w.site(target).resource[0], 1.0);
        assert_eq!(total(&w), before);
        // Loot eaten doesn't count against the limit: room 7 for the 2.
        let (mut w, _, watcher) = seen_world(0.0, 10, 6.0, 3.0);
        w.config.theft.loot = crate::config::Loot::Eat;
        w.probe_raid_harvests = true;
        let h = go_and_gather(&mut w, watcher, target);
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 6.0));
        assert_eq!(w.agent(watcher).unwrap().holdings[0], 5.0);
        // A wasted raid harvests as usual, probe or not.
        let (mut w, owner, watcher) = seen_world(0.0, 0, 6.0, 3.0);
        w.agent_mut(owner).unwrap().caches.clear();
        w.probe_raid_harvests = true;
        let h = go_and_gather(&mut w, watcher, target);
        assert_eq!((h.gathered[0], h.pilfered), (2.0, 0.0));
    }

    /// Σ sites + holdings + caches + stomachs + eaten + what left with the
    /// dead = start + growback, over 300 ticks of five watching walkers
    /// burying by script, one killed at tick 150.
    fn conserved_through_raids(loot: crate::config::Loot, find: f64) {
        conserved_through_raids_probe(loot, find, false);
    }

    fn conserved_through_raids_probe(loot: crate::config::Loot, find: f64, harvests: bool) {
        use crate::config::{Loot, MoveMode, Movement};
        let mut c = blank_config(12, 12);
        c.theft.find = find;
        c.theft.loot = loot;
        c.caching.bury_cost = 0.1;
        c.growback.rate = 0.3;
        c.goap.horizon = 6;
        c.caching.capacity = 8;
        c.watching.on = true;
        c.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        let mut w = World::new(c, 11).unwrap();
        w.record_fates = true;
        w.probe_raid_harvests = harvests;
        for y in 0..12 {
            for x in 0..12 {
                set_sugar(&mut w, x, y, if (x + y) % 3 == 0 { 4.0 } else { 1.5 });
            }
        }
        let mut ids = Vec::new();
        for (x, y) in [(1, 1), (6, 2), (3, 8), (9, 9), (10, 4)] {
            ids.push(walker(&mut w, x, y, 5.0, 3, true));
        }
        let start = total(&w);
        let (mut eaten, mut grown, mut left) = (0.0, 0.0, 0.0);
        let (mut pilfered, mut pilfers, mut raids, mut raided) = (0.0, 0, 0, 0.0);
        let doomed = ids[0];
        for _ in 0..300 {
            w.events = crate::world::TickEvents::default();
            if w.tick == 150 {
                left += w.agent(doomed).unwrap().holdings[0];
                w.kill(doomed, crate::world::DeathCause::OldAge);
            }
            let mut order = w.agent_ids();
            use rand::seq::SliceRandom;
            order.shuffle(&mut w.rng);
            for id in order {
                eaten += f64::from(w.agent(id).unwrap().metabolism[0]);
                crate::rules::agent_turn(&mut w, id);
                assert!(w.agent(id).is_some(), "the landscape is rich");
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
            left += w.events.cache_lost + w.events.fed_lost;
            eaten += w.events.bury_cost;
            let e = &w.events;
            pilfered += e.pilfered;
            pilfers += e.pilfers;
            raids += e.raids;
            raided += e.raided;
            assert!(e.raids <= e.pilfers && e.raided <= e.pilfered + 1e-12);
            if find == 0.0 {
                assert_eq!((e.raids, e.raided), (e.pilfers, e.pilfered));
            }
            match loot {
                Loot::Eat => assert_eq!(e.loot_eaten, e.pilfered),
                Loot::Keep => assert_eq!(e.loot_eaten, 0.0),
            }
            assert!(e.pilfers <= 5, "at most one each");
            let (lhs, rhs) = (total(&w) + eaten + left, start + grown);
            assert!((lhs - rhs).abs() <= 1e-9 * rhs, "{lhs} vs {rhs}");
        }
        assert_eq!(w.population(), 4);
        assert!(raids > 0 && raided > 0.0, "{raids} {raided}");
        assert!(pilfers >= raids && pilfered >= raided);
        if find > 0.0 {
            assert!(pilfers > raids, "stumbles too: {pilfers} {raids}");
        }
        let logged: f64 = w
            .cache_log
            .iter()
            .filter(|r| {
                matches!(
                    r.fate,
                    Some((_, crate::minds::caching::fates::Fate::Pilfered { .. }))
                )
            })
            .map(|r| r.amount)
            .sum();
        assert!((logged - pilfered).abs() <= 1e-9 * (1.0 + pilfered));
    }

    #[test]
    fn sugar_is_conserved_through_raids_kept_without_find() {
        conserved_through_raids(crate::config::Loot::Keep, 0.0);
    }

    #[test]
    fn sugar_is_conserved_through_raids_eaten_without_find() {
        conserved_through_raids(crate::config::Loot::Eat, 0.0);
    }

    #[test]
    fn sugar_is_conserved_through_raids_kept_with_find() {
        conserved_through_raids(crate::config::Loot::Keep, 0.25);
    }

    #[test]
    fn sugar_is_conserved_through_raids_eaten_with_find() {
        conserved_through_raids(crate::config::Loot::Eat, 0.25);
    }

    #[test]
    fn sugar_is_conserved_through_raids_that_also_harvest() {
        for loot in [crate::config::Loot::Keep, crate::config::Loot::Eat] {
            for find in [0.0, 0.25] {
                conserved_through_raids_probe(loot, find, true);
            }
        }
    }

    /// Watching on with no watchers is the run without watching, bit for
    /// bit, with and without theft.
    #[test]
    fn watching_with_no_watchers_is_the_run_without_watching() {
        for find in [0.0, 0.25] {
            let mut c = crate::presets::by_id("cache-winter-mixed").unwrap().config;
            c.theft.find = find;
            let mut off = World::new(c.clone(), 3).unwrap();
            c.watching.on = true;
            c.watching.watchers = 0.0;
            let mut on = World::new(c, 3).unwrap();
            for _ in 0..300 {
                off.step();
                on.step();
                assert_eq!(off.fingerprint(), on.fingerprint(), "{find}");
                assert_eq!(raid_counts(&on), (0, 0.0, 0, 0));
            }
        }
    }

    /// Controller ruling R1: Minds 6's pilfering bookkeeping runs under
    /// watching alone (find 0, no cheaters), so raids give a pilferage rate.
    #[test]
    fn a_watching_only_world_counts_its_raids_as_pilferage() {
        let mut c = crate::presets::by_id("cache-winter-mixed").unwrap().config;
        assert_eq!((c.theft.find, c.theft.cheaters), (0.0, 0.0));
        c.watching.on = true;
        assert!(!c.theft.is_on());
        assert!(crate::stats::series_names(&c)
            .iter()
            .any(|n| n == "pilferage_rate"));
        let mut w = World::new(c, 3).unwrap();
        let (mut raids, mut pilfered_caches, mut candidates_seen) = (0, 0, 0);
        for _ in 0..300 {
            let caches: usize = w.agents().map(|a| a.caches.len()).sum();
            w.step();
            let e = &w.events;
            assert_eq!(e.pilfer_candidates as usize, caches);
            assert_eq!((e.raids, e.raided), (e.pilfers, e.pilfered));
            assert!(e.caches_pilfered <= e.raids);
            raids += e.raids;
            pilfered_caches += e.caches_pilfered;
            candidates_seen += e.pilfer_candidates;
            assert!(w.stats.latest().unwrap().theft.is_some());
        }
        assert!(
            raids > 0 && pilfered_caches > 0,
            "{raids} {pilfered_caches}"
        );
        assert!(candidates_seen > pilfered_caches);
        let id = w.agents().next().unwrap().id;
        let p = w.agent(id).unwrap().pos;
        assert!(w.inspect(p.x, p.y).unwrap().agent.unwrap().theft.is_some());
    }
}
