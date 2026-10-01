//! Minds 8: watchers who see a burial and remember the cache.
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
//!   stay until read.
//! - **Counts.** `burials_seen` counts burials with at least one watcher;
//!   `sightings` counts (watcher, burial) pairs; `seen_entries` is the
//!   entries held, summed over agents, after the sweep.
//!
//! Nothing here draws or moves sugar: with `watching.on` false nothing runs
//! and nothing is allocated.

use std::collections::BTreeSet;

use crate::agent::AgentId;
use crate::geometry::DIRECTIONS;
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
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "read when entries are read by raids")
)]
pub(crate) fn fresh(world: &World, tick: u64) -> bool {
    world.tick.saturating_sub(tick) <= u64::from(world.config.watching.span)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Wall};
    use crate::geometry::Pos;
    use crate::minds::caching::bury;
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

    /// Watching draws nothing and moves no sugar: a caching world steps the
    /// same with it on as off. With it off nothing is seen or allocated.
    #[test]
    fn watching_changes_no_trajectory_and_off_allocates_nothing() {
        let c = crate::presets::by_id("cache-winter-mixed").unwrap().config;
        let mut off = World::new(c.clone(), 3).unwrap();
        let mut on = c;
        on.watching.on = true;
        let mut on = World::new(on, 3).unwrap();
        let (mut buried, mut sightings) = (0.0, 0);
        for _ in 0..150 {
            off.step();
            on.step();
            assert_eq!(off.fingerprint(), on.fingerprint());
            assert_eq!(
                (
                    off.events.burials_seen,
                    off.events.sightings,
                    off.events.seen_entries
                ),
                (0, 0, 0)
            );
            assert!(off.agents().all(|a| a.seen.is_empty()));
            buried += off.events.buried;
            sightings += on.events.sightings;
        }
        assert!(buried > 0.0 && sightings > 0, "{buried} {sightings}");
        assert!(on.agents().any(|a| !a.seen.is_empty()));
    }
}
