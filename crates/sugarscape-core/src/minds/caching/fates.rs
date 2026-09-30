//! Minds 6: every cache's fate, in a world-level log the survey reads.
//!
//! - **Records.** Each burial event (a positive `bury`) opens one
//!   [`CacheRecord`] holding the amount buried then, its owner, its site and
//!   the tick. A record ends in exactly one [`Fate`]: dug by its owner,
//!   pilfered (by whom), or lost with its dead owner. A record with no fate
//!   is still buried.
//! - **FIFO.** A dig or a pilfer of `amount` at an owner's site closes that
//!   owner's open records there oldest first. When only part of a record is
//!   taken, it splits: a new, closed record for the part taken (same owner,
//!   site and burial tick) is appended, and the open record keeps the rest.
//!   So the log isn't in burial order; each record carries its own tick.
//! - **Rounding.** A cache is a running f64 sum of its burials, and its
//!   records hold the same terms one by one, so they agree to rounding. A
//!   take that empties the cache (the owner no longer has one at the site)
//!   closes every open record there, whatever their sum; a take that
//!   doesn't closes whole records while they fit and splits the next.
//! - **Death.** `World::remove` closes all of a dead owner's open records as
//!   `Lost`.
//! - **Theft only.** Records open only while `theft.is_on()` (`find` or
//!   `cheaters` above 0): every other world, Minds 5's caching worlds
//!   included, leaves the log and its index empty and unallocated. Records
//!   already open keep closing if theft is turned off.
//! - **Backfill.** `find` is live, so a cache can hold sugar the log never
//!   saw (buried while theft was off). While theft is on, every touch of a
//!   cache (a burial, a dig, a pilfer, its owner's death) first compares the
//!   cache as it stood before the touch with its open records; if the cache
//!   holds more (by over 1e-9 of it, relatively), a backfill record for the
//!   difference is pushed, buried at the cache's `cache_since`, before the
//!   touch is logged. Unlogged sugar is always newer than the open records
//!   (it was buried after them) or the only sugar there, so it goes at the
//!   back of the queue. The one thing outside the log is sugar buried while
//!   theft was off in a cache nobody has touched since theft came on.
//! - **What is exact.** From the tick theft comes on (and stays on), the
//!   closed `Dug` records sum to the dug sugar (`events.dug`), the `Lost`
//!   records to the lost sugar (`events.cache_lost`), and each (owner,
//!   site) with open records holds a cache equal to their sum, all to
//!   rounding; a cache with no open records was last touched before theft
//!   came on.
//! - **Cap.** The log holds at most [`LOG_CAP`] records. A touch that might
//!   pass it (a burial, a take, which may push a backfill and a split, or a
//!   death, which may push a backfill per cache) sets `World.cache_log_full`
//!   before changing anything and freezes the log: nothing opens, closes or
//!   splits after that, so records still open then read as buried whatever
//!   happened to them. The survey checks the flag and skips a full log.
//!   The log is never hashed and draws nothing.

use std::collections::{BTreeMap, VecDeque};

use crate::agent::AgentId;
use crate::world::World;

/// Most records the log holds before it freezes (`World.cache_log_full`).
pub const LOG_CAP: usize = 1_000_000;

/// A cache holding more than its open records by more than this share of
/// itself (or of 1, for a cache under 1) has unlogged sugar to backfill.
const BACKFILL_EPS: f64 = 1e-9;

/// How a cache (or part of one) ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// Dug by its owner.
    Dug,
    /// Pilfered by another agent.
    Pilfered { by: AgentId },
    /// Its owner died with it buried.
    Lost,
}

/// One burial event (or the part of one that a take split off, or a
/// backfill of sugar buried while theft was off).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CacheRecord {
    pub owner: AgentId,
    /// Site index.
    pub site: u32,
    pub amount: f64,
    /// The tick it was buried (for a backfill, the cache's `cache_since`).
    pub buried: u64,
    /// The tick it ended and how; `None` while it's still buried.
    pub fate: Option<(u64, Fate)>,
}

/// Open records per (owner, site), oldest first, as indices into
/// `World.cache_log`.
pub(crate) type OpenRecords = BTreeMap<(AgentId, u32), VecDeque<usize>>;

/// Whether the log would pass the cap with `more` records; if so it
/// freezes.
fn frozen_by(world: &mut World, more: usize) -> bool {
    if world.cache_log.len() + more > LOG_CAP {
        world.cache_log_full = true;
        world.cache_open = BTreeMap::new();
    }
    world.cache_log_full
}

/// Pushes an open record (the caller has checked the cap).
fn push(world: &mut World, owner: AgentId, site: u32, amount: f64, buried: u64) {
    let i = world.cache_log.len();
    world.cache_log.push(CacheRecord {
        owner,
        site,
        amount,
        buried,
        fate: None,
    });
    world
        .cache_open
        .entry((owner, site))
        .or_default()
        .push_back(i);
}

/// Pushes a backfill record when `cache` (the cache as it stood before this
/// touch) holds more than `owner`'s open records at `site`.
fn backfill(world: &mut World, owner: AgentId, site: u32, cache: f64, since: u64) {
    let logged: f64 = world
        .cache_open
        .get(&(owner, site))
        .map_or(0.0, |q| q.iter().map(|&i| world.cache_log[i].amount).sum());
    let gap = cache - logged;
    if gap > BACKFILL_EPS * cache.max(1.0) {
        push(world, owner, site, gap, since);
    }
}

/// Opens a record for `amount` just buried by `owner` at `site` (under
/// theft), after backfilling what the cache held before it.
pub(crate) fn open(world: &mut World, owner: AgentId, site: u32, amount: f64) {
    if world.cache_log_full || !world.config.theft.is_on() || frozen_by(world, 2) {
        return;
    }
    let now = world.tick;
    let a = world.agent(owner).expect("live agent");
    let before = a.caches.get(&site).map_or(0.0, |c| c - amount);
    let since = a.cache_since.get(&site).copied().unwrap_or(now);
    backfill(world, owner, site, before, since);
    push(world, owner, site, amount, now);
}

/// Closes `amount` of `owner`'s records at `site`, oldest first, as `fate`.
/// Call it after the cache itself has been reduced, but before an emptied
/// cache's `cache_since` is removed (a backfill reads it): if the owner no
/// longer has a cache there, every open record there closes.
fn close(world: &mut World, owner: AgentId, site: u32, amount: f64, fate: Fate) {
    if world.cache_log_full {
        return;
    }
    let theft = world.config.theft.is_on();
    if !theft && world.cache_open.is_empty() {
        return;
    }
    // A backfill and a split: at most two records.
    if frozen_by(world, 2) {
        return;
    }
    let now = world.tick;
    let a = world.agent(owner).expect("live agent");
    let left_in_cache = a.caches.get(&site).copied();
    let emptied = left_in_cache.is_none();
    if theft {
        let since = a.cache_since.get(&site).copied().unwrap_or(now);
        backfill(
            world,
            owner,
            site,
            left_in_cache.unwrap_or(0.0) + amount,
            since,
        );
    }
    let Some(queue) = world.cache_open.get_mut(&(owner, site)) else {
        return;
    };
    let mut left = amount;
    let mut split = None;
    while let Some(&i) = queue.front() {
        let r = &mut world.cache_log[i];
        if emptied || r.amount <= left {
            left -= r.amount;
            r.fate = Some((now, fate));
            queue.pop_front();
        } else {
            if left > 0.0 {
                split = Some(i);
            }
            break;
        }
    }
    if queue.is_empty() {
        world.cache_open.remove(&(owner, site));
    }
    if let Some(i) = split {
        let r = &mut world.cache_log[i];
        r.amount -= left;
        let part = CacheRecord {
            amount: left,
            fate: Some((now, fate)),
            ..*r
        };
        world.cache_log.push(part);
    }
}

/// Closes `amount` of `owner`'s records at `site` as dug.
pub(crate) fn close_dug(world: &mut World, owner: AgentId, site: u32, amount: f64) {
    close(world, owner, site, amount, Fate::Dug);
}

/// Closes `amount` of `owner`'s records at `site` as pilfered by `by`. The
/// caller takes the amount from the cache first, and removes an emptied
/// cache's `cache_since` only after this call.
pub(crate) fn close_pilfered(
    world: &mut World,
    owner: AgentId,
    site: u32,
    amount: f64,
    by: AgentId,
) {
    close(world, owner, site, amount, Fate::Pilfered { by });
}

/// Closes every open record of `owner` (just removed from the world, with
/// these `caches` and `since`) as lost, after backfilling each cache.
pub(crate) fn close_lost(
    world: &mut World,
    owner: AgentId,
    caches: &BTreeMap<u32, f64>,
    since: &BTreeMap<u32, u64>,
) {
    if world.cache_log_full {
        return;
    }
    let now = world.tick;
    if world.config.theft.is_on() && !caches.is_empty() {
        if frozen_by(world, caches.len()) {
            return;
        }
        for (&site, &amount) in caches {
            let buried = since.get(&site).copied().unwrap_or(now);
            backfill(world, owner, site, amount, buried);
        }
    }
    if world.cache_open.is_empty() {
        return;
    }
    let keys: Vec<(AgentId, u32)> = world
        .cache_open
        .range((owner, 0)..=(owner, u32::MAX))
        .map(|(&k, _)| k)
        .collect();
    for k in keys {
        for i in world.cache_open.remove(&k).unwrap_or_default() {
            world.cache_log[i].fate = Some((now, Fate::Lost));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pos;
    use crate::minds::caching::{bury, dig};
    use crate::testkit::*;

    /// A blank world with theft on (so the log records).
    fn theft_world() -> World {
        let mut w = blank_world(11, 11);
        w.config.theft.find = 0.5;
        w
    }

    fn hoarder(w: &mut World, x: u32, y: u32, held: f64) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.holdings[0] = held;
        id
    }

    fn site(w: &World, x: u32, y: u32) -> u32 {
        w.torus.index(Pos::new(x, y)) as u32
    }

    /// Σ amounts of the records of `owner` at `site` with fate `f` (`None`:
    /// still buried).
    fn sum(w: &World, owner: AgentId, site: u32, f: Option<Fate>) -> f64 {
        w.cache_log
            .iter()
            .filter(|r| r.owner == owner && r.site == site && r.fate.map(|(_, x)| x) == f)
            .map(|r| r.amount)
            .sum()
    }

    #[test]
    fn a_burial_opens_one_record_and_digs_close_them_oldest_first_splitting() {
        let mut w = theft_world();
        let id = hoarder(&mut w, 5, 5, 20.0);
        let here = site(&w, 5, 5);
        w.tick = 2;
        bury(&mut w, id, 3.0);
        w.tick = 4;
        bury(&mut w, id, 5.0);
        assert_eq!(w.cache_log.len(), 2);
        assert_eq!(
            w.cache_log[0],
            CacheRecord {
                owner: id,
                site: here,
                amount: 3.0,
                buried: 2,
                fate: None
            }
        );
        // A dig of 4: the 3 closes whole, and 1 of the 5 splits off.
        w.tick = 7;
        assert_eq!(dig(&mut w, id, here, 4.0), 4.0);
        assert_eq!(w.cache_log.len(), 3);
        assert_eq!(w.cache_log[0].fate, Some((7, Fate::Dug)));
        assert_eq!((w.cache_log[1].amount, w.cache_log[1].fate), (4.0, None));
        assert_eq!(
            w.cache_log[2],
            CacheRecord {
                owner: id,
                site: here,
                amount: 1.0,
                buried: 4,
                fate: Some((7, Fate::Dug))
            }
        );
        // Emptying the cache closes the rest.
        w.tick = 9;
        assert_eq!(dig(&mut w, id, here, 100.0), 4.0);
        assert_eq!(w.cache_log[1].fate, Some((9, Fate::Dug)));
        assert!(w.cache_open.is_empty());
        assert_eq!(sum(&w, id, here, Some(Fate::Dug)), 8.0);
    }

    #[test]
    fn a_pilfer_closes_records_as_pilfered_by_the_thief() {
        let mut w = theft_world();
        let id = hoarder(&mut w, 5, 5, 20.0);
        let thief = hoarder(&mut w, 6, 6, 0.0);
        let here = site(&w, 5, 5);
        bury(&mut w, id, 2.0);
        bury(&mut w, id, 6.0);
        // The stub pilfer: take 5 from the cache, then close the records.
        *w.agent_mut(id).unwrap().caches.get_mut(&here).unwrap() -= 5.0;
        w.tick = 3;
        close_pilfered(&mut w, id, here, 5.0, thief);
        let by = Some(Fate::Pilfered { by: thief });
        assert_eq!(sum(&w, id, here, by), 5.0);
        assert_eq!(sum(&w, id, here, None), 3.0);
        assert_eq!(w.cache_log.len(), 3, "the 6 split into 3 taken and 3 left");
        // A pilfer that empties the cache closes the rest.
        w.agent_mut(id).unwrap().caches.remove(&here);
        close_pilfered(&mut w, id, here, 3.0, thief);
        assert_eq!(sum(&w, id, here, by), 8.0);
        assert!(w.cache_open.is_empty());
    }

    #[test]
    fn a_dead_owners_open_records_are_lost_and_every_record_has_one_fate() {
        let mut w = theft_world();
        let id = hoarder(&mut w, 5, 5, 30.0);
        let other = hoarder(&mut w, 8, 8, 30.0);
        let thief = hoarder(&mut w, 2, 2, 0.0);
        let here = site(&w, 5, 5);
        bury(&mut w, id, 4.0);
        bury(&mut w, other, 4.0);
        w.move_agent(id, Pos::new(5, 6));
        let there = site(&w, 5, 6);
        bury(&mut w, id, 3.0);
        w.move_agent(id, Pos::new(5, 7));
        bury(&mut w, id, 2.0);
        // Dig 1 at (5, 6), pilfer 1 at (5, 7), then the owner dies.
        dig(&mut w, id, there, 1.0);
        let far = site(&w, 5, 7);
        *w.agent_mut(id).unwrap().caches.get_mut(&far).unwrap() -= 1.0;
        close_pilfered(&mut w, id, far, 1.0, thief);
        w.tick = 12;
        w.kill(id, crate::world::DeathCause::OldAge);
        assert_eq!(w.events.cache_lost, 7.0);
        let lost: f64 = w
            .cache_log
            .iter()
            .filter(|r| r.owner == id && r.fate == Some((12, Fate::Lost)))
            .map(|r| r.amount)
            .sum();
        assert_eq!(lost, w.events.cache_lost, "4 + 2 + 1");
        assert_eq!(sum(&w, id, here, Some(Fate::Lost)), 4.0);
        assert!(w
            .cache_log
            .iter()
            .filter(|r| r.owner == id)
            .all(|r| r.fate.is_some()));
        // The other owner's cache is untouched: still buried.
        assert_eq!(sum(&w, other, site(&w, 8, 8), None), 4.0);
        assert_eq!(w.cache_open.len(), 1);
        // Per owner, fates and open records add up to what was buried.
        let total: f64 = w
            .cache_log
            .iter()
            .filter(|r| r.owner == id)
            .map(|r| r.amount)
            .sum();
        assert_eq!(total, 9.0);
    }

    #[test]
    fn a_full_log_freezes_and_sets_the_flag() {
        let mut w = theft_world();
        let id = hoarder(&mut w, 5, 5, 20.0);
        let here = site(&w, 5, 5);
        bury(&mut w, id, 1.0);
        // Pretend the log is at the cap.
        let filler = w.cache_log[0];
        w.cache_log.resize(LOG_CAP, filler);
        bury(&mut w, id, 1.0);
        assert!(w.cache_log_full);
        assert_eq!(w.cache_log.len(), LOG_CAP);
        assert!(w.cache_open.is_empty());
        assert_eq!(dig(&mut w, id, here, 2.0), 2.0, "the cache itself works");
        assert_eq!(w.cache_log[0].fate, None, "frozen");
    }

    #[test]
    fn sugar_buried_before_theft_is_backfilled_before_the_log_moves_on() {
        // A cache of 10 buried with theft off; theft comes on; 2 more are
        // buried; 3 are dug; a pilfer empties the other 9. The truth is 3
        // dug and 9 pilfered.
        let mut w = blank_world(11, 11);
        let id = hoarder(&mut w, 5, 5, 20.0);
        let thief = hoarder(&mut w, 6, 6, 0.0);
        let here = site(&w, 5, 5);
        w.tick = 1;
        bury(&mut w, id, 10.0);
        assert!(w.cache_log.is_empty(), "theft off: no record");
        w.config.theft.find = 0.5;
        w.tick = 5;
        bury(&mut w, id, 2.0);
        assert_eq!(w.cache_log.len(), 2);
        assert_eq!((w.cache_log[0].amount, w.cache_log[0].buried), (10.0, 1));
        assert_eq!((w.cache_log[1].amount, w.cache_log[1].buried), (2.0, 5));
        w.tick = 6;
        assert_eq!(dig(&mut w, id, here, 3.0), 3.0);
        w.agent_mut(id).unwrap().caches.remove(&here);
        w.tick = 7;
        close_pilfered(&mut w, id, here, 9.0, thief);
        w.agent_mut(id).unwrap().cache_since.remove(&here);
        assert_eq!(sum(&w, id, here, Some(Fate::Dug)), 3.0);
        assert_eq!(sum(&w, id, here, Some(Fate::Pilfered { by: thief })), 9.0);
        assert_eq!(sum(&w, id, here, None), 0.0);
        assert!(w.cache_open.is_empty());
        // The dug 3 came from the oldest sugar, buried at tick 1.
        assert!(w
            .cache_log
            .iter()
            .filter(|r| r.fate.is_some_and(|(_, f)| f == Fate::Dug))
            .all(|r| r.buried == 1));
    }

    #[test]
    fn a_dig_or_a_death_backfills_a_cache_untouched_since_theft_came_on() {
        let mut w = blank_world(11, 11);
        let id = hoarder(&mut w, 5, 5, 20.0);
        let here = site(&w, 5, 5);
        w.tick = 2;
        bury(&mut w, id, 6.0);
        w.move_agent(id, Pos::new(5, 6));
        let there = site(&w, 5, 6);
        bury(&mut w, id, 4.0);
        w.config.theft.cheaters = 0.25;
        w.tick = 8;
        assert_eq!(dig(&mut w, id, here, 1.0), 1.0);
        assert_eq!(sum(&w, id, here, Some(Fate::Dug)), 1.0);
        assert_eq!(sum(&w, id, here, None), 5.0);
        w.kill(id, crate::world::DeathCause::OldAge);
        assert_eq!(sum(&w, id, here, Some(Fate::Lost)), 5.0);
        assert_eq!(sum(&w, id, there, Some(Fate::Lost)), 4.0);
        assert!(w.cache_log.iter().all(|r| r.buried == 2));
    }

    /// Runs `cache-winter-mixed` (175 agents under `caching.mixed`, many
    /// dying in winter) for 300 ticks with `theft.cheaters` = `cheaters`
    /// and `theft.find` scheduled from 0 to 0.05 at tick 100, and checks the
    /// log's exact statements every tick from the first tick theft is on.
    fn the_log_matches_the_world(cheaters: f64) {
        let mut c = crate::presets::by_id("cache-winter-mixed").unwrap().config;
        c.theft.cheaters = cheaters;
        c.schedule.push(crate::config::ScheduledChange {
            tick: 100,
            set: [("theft.find".to_string(), serde_json::json!(0.05))]
                .into_iter()
                .collect(),
        });
        let mut w = World::new(c, 4).unwrap();
        let (mut dug, mut lost, mut buried, mut pilfered) = (0.0, 0.0, 0.0, 0.0);
        let mut on_since = None;
        for _ in 0..300 {
            w.step();
            if !w.config.theft.is_on() {
                assert!(w.cache_log.is_empty());
                continue;
            }
            // The step just taken is the first with theft on, or later.
            on_since.get_or_insert(w.tick - 1);
            dug += w.events.dug;
            lost += w.events.cache_lost;
            buried += w.events.buried;
            pilfered += w.events.pilfered;
            let tol = 1e-9 * (1.0 + buried + dug + lost + pilfered);
            let closed = |f: Fate| -> f64 {
                w.cache_log
                    .iter()
                    .filter(|r| r.fate.is_some_and(|(_, x)| x == f))
                    .map(|r| r.amount)
                    .sum()
            };
            let t = w.tick;
            assert!((closed(Fate::Dug) - dug).abs() <= tol, "{t}: dug");
            assert!((closed(Fate::Lost) - lost).abs() <= tol, "{t}: lost");
            let taken: f64 = w
                .cache_log
                .iter()
                .filter(|r| matches!(r.fate, Some((_, Fate::Pilfered { .. }))))
                .map(|r| r.amount)
                .sum();
            assert!((taken - pilfered).abs() <= tol, "{t}: pilfered");
            // Each (owner, site) with open records holds their sum.
            let (mut open, mut held) = (0.0, 0.0);
            for (&(owner, site), q) in &w.cache_open {
                let o: f64 = q.iter().map(|&i| w.cache_log[i].amount).sum();
                let cache = w.agent(owner).expect("live owner").caches[&site];
                assert!((o - cache).abs() <= tol, "{t}: ({owner}, {site})");
                open += o;
                held += cache;
            }
            assert!((open - held).abs() <= tol);
            // Every other cache was last touched before theft came on.
            let on = on_since.unwrap();
            for a in w.agents() {
                for (&site, &amount) in &a.caches {
                    if !w.cache_open.contains_key(&(a.id, site)) {
                        let since = a.cache_since[&site];
                        assert!(since < on || amount <= tol, "{t}: unlogged cache");
                    }
                }
            }
            assert!(w.cache_log.iter().all(|r| r.amount >= 0.0));
        }
        assert!(!w.cache_log_full);
        assert!(
            dug > 0.0 && lost > 0.0 && pilfered > 0.0,
            "{dug} {lost} {pilfered}"
        );
        assert_eq!(on_since, Some(if cheaters > 0.0 { 0 } else { 100 }));
        if cheaters == 0.0 {
            assert!(
                w.cache_log.iter().any(|r| r.buried < 100),
                "sugar from before theft was backfilled"
            );
        }
    }

    #[test]
    fn the_log_matches_the_world_with_theft_on_from_the_start() {
        the_log_matches_the_world(0.25);
    }

    #[test]
    fn the_log_matches_the_world_when_theft_comes_on_partway() {
        the_log_matches_the_world(0.0);
    }

    #[test]
    fn a_minds_5_caching_world_logs_nothing() {
        let mut c = blank_config(12, 12);
        c.caching.rule = crate::config::CachingRule::Even;
        c.caching.capacity = 20;
        c.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        let mut w = World::new(c, 3).unwrap();
        for y in 0..12 {
            for x in 0..12 {
                crate::testkit::set_sugar(&mut w, x, y, 4.0);
            }
        }
        let id = hoarder(&mut w, 5, 5, 30.0);
        let mut buried = 0.0;
        for _ in 0..20 {
            w.events = crate::world::TickEvents::default();
            crate::rules::agent_turn(&mut w, id);
            buried += w.events.buried;
            w.tick += 1;
        }
        assert!(buried > 0.0, "it buried");
        w.kill(id, crate::world::DeathCause::OldAge);
        assert_eq!(w.cache_log.capacity(), 0);
        assert!(w.cache_open.is_empty());
        assert!(!w.cache_log_full);
    }

    #[test]
    fn nothing_buried_nothing_logged() {
        let mut w = theft_world();
        let id = hoarder(&mut w, 5, 5, 0.0);
        bury(&mut w, id, 3.0);
        w.kill(id, crate::world::DeathCause::OldAge);
        assert_eq!(w.cache_log.capacity(), 0);
        assert!(w.cache_open.is_empty());
    }
}
