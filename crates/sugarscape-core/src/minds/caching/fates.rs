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
//! - **Cap.** The log holds at most [`LOG_CAP`] records. The first record
//!   that would pass it sets `World.cache_log_full` and freezes the log: no
//!   record opens, closes or splits after that, so records still open then
//!   read as buried whatever happened to them. The survey checks the flag
//!   and doesn't read a full log.
//! - **Cost.** Nothing is logged unless something is buried: with caching
//!   off the log and its index stay empty and never allocate. The log is
//!   never hashed and draws nothing.

use std::collections::{BTreeMap, VecDeque};

use crate::agent::AgentId;
use crate::world::World;

/// Most records the log holds before it freezes (`World.cache_log_full`).
pub const LOG_CAP: usize = 1_000_000;

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

/// One burial event (or the part of one that a take split off).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CacheRecord {
    pub owner: AgentId,
    /// Site index.
    pub site: u32,
    pub amount: f64,
    /// The tick it was buried.
    pub buried: u64,
    /// The tick it ended and how; `None` while it's still buried.
    pub fate: Option<(u64, Fate)>,
}

/// Open records per (owner, site), oldest first, as indices into
/// `World.cache_log`.
pub(crate) type OpenRecords = BTreeMap<(AgentId, u32), VecDeque<usize>>;

fn freeze(world: &mut World) {
    world.cache_log_full = true;
    world.cache_open = BTreeMap::new();
}

/// Opens a record for `amount` just buried by `owner` at `site`.
pub(crate) fn open(world: &mut World, owner: AgentId, site: u32, amount: f64) {
    if world.cache_log_full {
        return;
    }
    if world.cache_log.len() >= LOG_CAP {
        freeze(world);
        return;
    }
    let i = world.cache_log.len();
    world.cache_log.push(CacheRecord {
        owner,
        site,
        amount,
        buried: world.tick,
        fate: None,
    });
    world
        .cache_open
        .entry((owner, site))
        .or_default()
        .push_back(i);
}

/// Closes `amount` of `owner`'s records at `site`, oldest first, as `fate`.
/// Call it after the cache itself has been reduced: if the owner no longer
/// has a cache there, every open record there closes.
fn close(world: &mut World, owner: AgentId, site: u32, amount: f64, fate: Fate) {
    if world.cache_log_full {
        return;
    }
    let now = world.tick;
    let emptied = world
        .agent(owner)
        .is_none_or(|a| !a.caches.contains_key(&site));
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
        if world.cache_log.len() >= LOG_CAP {
            freeze(world);
            return;
        }
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
/// caller takes the amount from the cache first.
#[allow(dead_code)] // until the pilfer calls it
pub(crate) fn close_pilfered(
    world: &mut World,
    owner: AgentId,
    site: u32,
    amount: f64,
    by: AgentId,
) {
    close(world, owner, site, amount, Fate::Pilfered { by });
}

/// Closes every open record of `owner` (removed from the world) as lost.
pub(crate) fn close_lost(world: &mut World, owner: AgentId) {
    if world.cache_open.is_empty() {
        return;
    }
    let now = world.tick;
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
        let mut w = blank_world(11, 11);
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
        let mut w = blank_world(11, 11);
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
        let mut w = blank_world(11, 11);
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
        let mut w = blank_world(11, 11);
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
    fn nothing_buried_nothing_logged() {
        let mut w = blank_world(11, 11);
        let id = hoarder(&mut w, 5, 5, 0.0);
        bury(&mut w, id, 3.0);
        w.kill(id, crate::world::DeathCause::OldAge);
        assert_eq!(w.cache_log.capacity(), 0);
        assert!(w.cache_open.is_empty());
    }
}
