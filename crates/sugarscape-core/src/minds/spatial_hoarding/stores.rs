//! Conserved larder transfers; callers must not credit the returned amount again.

use super::{access::in_contact, state::StoreKind};
use crate::agent::AgentId;
use crate::config::Loot;
use crate::minds::caching::fates::{self, Fate};
use crate::world::World;

/// Extension-only per-kind tick accounting. `None` on ordinary worlds.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StoreEvents {
    pub scatter: StoreFlow,
    pub larder: StoreFlow,
}

/// Amounts are food units; counts describe positive takes and tick-start stock exposure.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StoreFlow {
    pub buried: f64,
    pub dug: f64,
    pub pilfered: f64,
    pub lost: f64,
    pub bury_cost: f64,
    pub loot_eaten: f64,
    pub digs: u32,
    pub pilfers: u32,
    pub pilfer_candidates: u32,
    pub caches_pilfered: u32,
}

pub(crate) fn events(world: &mut World, kind: StoreKind) -> Option<&mut StoreFlow> {
    if !world.config.spatial_hoarding.enabled {
        return None;
    }
    let e = world
        .events
        .spatial_stores
        .get_or_insert_with(StoreEvents::default);
    Some(match kind {
        StoreKind::Scatter => &mut e.scatter,
        StoreKind::Larder => &mut e.larder,
    })
}

fn valid(world: &World, id: AgentId, amount: f64) -> bool {
    world.config.spatial_hoarding.enabled
        && amount.is_finite()
        && amount > 0.0
        && world.agent(id).and_then(|a| a.spatial.as_ref()).is_some()
}

/// Task 3 calls this after a completed return; no remote deposit is accepted.
#[allow(dead_code)]
pub(crate) fn deposit(world: &mut World, owner: AgentId, amount: f64) -> f64 {
    if !valid(world, owner, amount) {
        return 0.0;
    }
    let a = world.agent(owner).expect("checked");
    let s = a.spatial.as_ref().expect("checked");
    if !in_contact(world, a.pos, s.home) {
        return 0.0;
    }
    let site = world.torus.index(s.home) as u32;
    let cost = world.config.caching.bury_cost;
    let q = amount.min(a.holdings[0] / (1.0 + cost)).max(0.0);
    if q <= 0.0 {
        return 0.0;
    }
    let before = (s.larder, s.larder_since.unwrap_or(world.tick));
    let now = world.tick;
    let a = world.agent_mut(owner).expect("checked");
    a.holdings[0] -= q;
    let paid = (q * cost).min(a.holdings[0]);
    a.holdings[0] -= paid;
    let s = a.spatial.as_mut().expect("checked");
    s.larder += q;
    s.larder_since.get_or_insert(now);
    world.events.buried += q;
    world.events.bury_cost += paid;
    let e = events(world, StoreKind::Larder).expect("enabled");
    e.buried += q;
    e.bury_cost += paid;
    fates::open_store(world, owner, site, StoreKind::Larder, q, before);
    q
}

/// Task 3 applies the hunger threshold before calling; this enforces geometry and room.
#[allow(dead_code)]
pub(crate) fn recover(world: &mut World, owner: AgentId, amount: f64) -> f64 {
    if !valid(world, owner, amount) {
        return 0.0;
    }
    let a = world.agent(owner).expect("checked");
    let s = a.spatial.as_ref().expect("checked");
    if !in_contact(world, a.pos, s.home) {
        return 0.0;
    }
    let room = (f64::from(world.config.caching.capacity) - a.holdings[0]).max(0.0);
    let q = amount.min(s.larder).min(room);
    if q <= 0.0 {
        return 0.0;
    }
    let since = s.larder_since.unwrap_or(world.tick);
    reduce(world, owner, q, Fate::Dug);
    world.agent_mut(owner).expect("checked").holdings[0] += q;
    world.events.dug += q;
    world.events.digs += 1;
    world.events.dig_ages_sum += world.tick.saturating_sub(since);
    let e = events(world, StoreKind::Larder).expect("enabled");
    e.dug += q;
    e.digs += 1;
    q
}

/// Task 4's observed raids and discoveries share this guard/contact/loot enforcement.
#[allow(dead_code)]
pub(crate) fn take(world: &mut World, owner: AgentId, taker: AgentId, amount: f64) -> f64 {
    if owner == taker || !valid(world, owner, amount) || !valid(world, taker, amount) {
        return 0.0;
    }
    let a = world.agent(owner).expect("checked");
    let s = a.spatial.as_ref().expect("checked");
    let t = world.agent(taker).expect("checked");
    if s.guarding || !in_contact(world, t.pos, s.home) {
        return 0.0;
    }
    let site = world.torus.index(s.home) as u32;
    let at_start = s.larder_since.is_some_and(|since| since < world.tick);
    let eat = world.config.theft.loot == Loot::Eat;
    let room = if eat {
        f64::INFINITY
    } else {
        (f64::from(world.config.caching.capacity) - t.holdings[0]).max(0.0)
    };
    let q = amount.min(s.larder).min(room);
    if q <= 0.0 {
        return 0.0;
    }
    reduce(world, owner, q, Fate::Pilfered { by: taker });
    world.agent_mut(owner).expect("checked").stolen_from_me += q;
    let t = world.agent_mut(taker).expect("checked");
    t.stolen_by_me += q;
    if eat {
        t.fed += q;
    } else {
        t.holdings[0] += q;
    }
    world.events.pilfered += q;
    world.events.pilfers += 1;
    if eat {
        world.events.loot_eaten += q;
    }
    let distinct = at_start
        && world
            .events
            .pilfered_caches
            .insert((owner, site, StoreKind::Larder));
    if distinct {
        world.events.caches_pilfered += 1;
    }
    let e = events(world, StoreKind::Larder).expect("enabled");
    e.pilfered += q;
    e.pilfers += 1;
    e.caches_pilfered += u32::from(distinct);
    if eat {
        e.loot_eaten += q;
    }
    q
}

fn reduce(world: &mut World, owner: AgentId, amount: f64, fate: Fate) {
    let torus = world.torus;
    let now = world.tick;
    let s = world
        .agent_mut(owner)
        .expect("checked")
        .spatial
        .as_mut()
        .expect("checked");
    s.larder = (s.larder - amount).max(0.0);
    let site = torus.index(s.home) as u32;
    let after = (s.larder, s.larder_since.unwrap_or(now));
    fates::close_store(world, owner, site, StoreKind::Larder, amount, fate, after);
    if after.0 == 0.0 {
        world
            .agent_mut(owner)
            .expect("checked")
            .spatial
            .as_mut()
            .expect("checked")
            .larder_since = None;
    }
}

#[cfg(test)]
mod tests {
    use super::super::state::{FounderTraits, SpatialState, StoreKind};
    use super::*;
    use crate::config::{CachingRule, Loot, MoveMode};
    use crate::geometry::Pos;
    use crate::minds::caching::{
        bury,
        fates::{Fate, LOG_CAP},
    };
    use crate::testkit::*;
    use crate::world::World;

    fn fixture() -> (World, crate::agent::AgentId, crate::agent::AgentId) {
        let mut w = blank_world(9, 9);
        w.config.spatial_hoarding.enabled = true;
        w.config.caching.rule = CachingRule::Even;
        w.config.caching.capacity = 20;
        w.config.movement.mode = MoveMode::Walk;
        w.record_fates = true;
        let owner = spawn(&mut w, 4, 4);
        let taker = spawn(&mut w, 4, 3);
        for id in [owner, taker] {
            let a = w.agent_mut(id).unwrap();
            a.holdings[0] = 20.0;
            a.spatial = Some(SpatialState::new(
                a.pos,
                FounderTraits {
                    larder: 1.0,
                    defense: 0.5,
                    cheater: false,
                    watches: false,
                },
            ));
        }
        (w, owner, taker)
    }
    fn stock(w: &World, id: crate::agent::AgentId) -> f64 {
        w.agent(id).unwrap().spatial.as_ref().unwrap().larder
    }
    fn conservation(w: &World, kind: StoreKind, buried: f64) {
        let records: Vec<_> = w.cache_log.iter().filter(|r| r.kind == kind).collect();
        let open: f64 = records
            .iter()
            .filter(|r| r.fate.is_none())
            .map(|r| r.amount)
            .sum();
        let closed: f64 = records
            .iter()
            .filter(|r| r.fate.is_some())
            .map(|r| r.amount)
            .sum();
        let stock: f64 = w
            .agents()
            .map(|a| match kind {
                StoreKind::Scatter => a.caches.values().sum(),
                StoreKind::Larder => a.spatial.as_ref().map_or(0.0, |s| s.larder),
            })
            .sum();
        let tol = 1e-9 * buried.max(1.0);
        assert!((open - stock).abs() <= tol);
        assert!((open + closed - buried).abs() <= tol);
    }

    #[test]
    fn same_site_kinds_partial_recovery_and_death_are_conserved() {
        let (mut w, owner, _) = fixture();
        assert_eq!(bury(&mut w, owner, 4.0), 4.0);
        assert_eq!(deposit(&mut w, owner, 6.0), 6.0);
        assert_eq!(recover(&mut w, owner, 2.0), 2.0);
        assert_eq!(stock(&w, owner), 4.0);
        assert_eq!(w.agent(owner).unwrap().caches.values().sum::<f64>(), 4.0);
        assert_eq!(w.agent(owner).unwrap().holdings[0], 12.0);
        conservation(&w, StoreKind::Scatter, 4.0);
        conservation(&w, StoreKind::Larder, 6.0);
        w.remove(owner);
        conservation(&w, StoreKind::Scatter, 4.0);
        conservation(&w, StoreKind::Larder, 6.0);
        assert_eq!(w.events.cache_lost, 8.0);
        assert_eq!(
            w.cache_log
                .iter()
                .filter(|r| r.kind == StoreKind::Larder && r.fate == Some((0, Fate::Lost)))
                .map(|r| r.amount)
                .sum::<f64>(),
            4.0
        );
        assert!(w.cache_open.is_empty());
    }

    #[test]
    fn foreign_keep_respects_room_and_eat_feeds_without_holdings() {
        for loot in [Loot::Keep, Loot::Eat] {
            let (mut w, owner, taker) = fixture();
            w.config.theft.loot = loot;
            deposit(&mut w, owner, 6.0);
            w.agent_mut(taker).unwrap().holdings[0] = 19.0;
            let expected = if loot == Loot::Keep { 1.0 } else { 6.0 };
            assert_eq!(take(&mut w, owner, taker, 6.0), expected);
            let a = w.agent(taker).unwrap();
            assert_eq!(a.holdings[0], if loot == Loot::Keep { 20.0 } else { 19.0 });
            assert_eq!(a.fed, if loot == Loot::Keep { 0.0 } else { 6.0 });
            assert_eq!(w.events.pilfered, expected);
            conservation(&w, StoreKind::Larder, 6.0);
        }
    }

    #[test]
    fn invalid_remote_guarded_and_full_transfers_do_nothing() {
        let (mut w, owner, taker) = fixture();
        for q in [f64::NAN, f64::INFINITY, -1.0, 0.0] {
            assert_eq!(deposit(&mut w, owner, q), 0.0);
            assert_eq!(recover(&mut w, owner, q), 0.0);
            assert_eq!(take(&mut w, owner, taker, q), 0.0);
        }
        deposit(&mut w, owner, 6.0);
        assert_eq!(take(&mut w, owner, taker, 1.0), 0.0);
        w.agent_mut(taker).unwrap().holdings[0] = 0.0;
        w.agent_mut(owner)
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .guarding = true;
        assert_eq!(take(&mut w, owner, taker, 1.0), 0.0);
        w.agent_mut(owner)
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .guarding = false;
        w.move_agent(owner, Pos::new(0, 0));
        assert_eq!(deposit(&mut w, owner, 1.0), 0.0);
        assert_eq!(recover(&mut w, owner, 1.0), 0.0);
        assert_eq!(take(&mut w, u64::MAX, taker, 1.0), 0.0);
        assert_eq!(take(&mut w, owner, owner, 1.0), 0.0);
        assert_eq!(stock(&w, owner), 6.0);
    }

    #[test]
    fn burial_cost_is_charged_at_deposit_without_negative_holdings() {
        let (mut w, owner, _) = fixture();
        w.config.caching.bury_cost = 1.0;
        assert_eq!(deposit(&mut w, owner, 20.0), 10.0);
        assert_eq!(w.agent(owner).unwrap().holdings[0], 0.0);
        assert_eq!(w.events.bury_cost, 10.0);
        assert_eq!(stock(&w, owner) + w.events.bury_cost, 20.0);
    }

    #[test]
    fn death_backfills_unlogged_larder_and_saturated_ledger_stays_explicit() {
        let (mut w, owner, _) = fixture();
        w.record_fates = false;
        deposit(&mut w, owner, 6.0);
        w.record_fates = true;
        w.remove(owner);
        conservation(&w, StoreKind::Larder, 6.0);
        let (mut w, owner, _) = fixture();
        deposit(&mut w, owner, 6.0);
        w.cache_log.resize(LOG_CAP, w.cache_log[0]);
        assert_eq!(recover(&mut w, owner, 2.0), 2.0);
        assert!(w.cache_log_full);
        assert!(w.cache_open.is_empty());
        assert_eq!(stock(&w, owner), 4.0);
    }
    #[test]
    fn empty_larder_death_does_not_freeze_nearly_saturated_scatter_ledger() {
        let (mut w, owner, _) = fixture();
        bury(&mut w, owner, 2.0);
        w.cache_log.resize(LOG_CAP - 1, w.cache_log[0]);
        w.remove(owner);
        assert!(!w.cache_log_full);
    }

    #[test]
    fn tick_start_candidates_and_partial_pilfers_distinguish_same_site_kinds() {
        let (mut w, owner, taker) = fixture();
        bury(&mut w, owner, 4.0);
        deposit(&mut w, owner, 6.0);
        w.tick = 1;
        crate::minds::caching::theft::count_candidates(&mut w);
        assert_eq!(w.events.pilfer_candidates, 2);
        let site = w.torus.index(Pos::new(4, 4)) as u32;
        w.agent_mut(taker).unwrap().holdings[0] = 0.0;
        assert_eq!(take(&mut w, owner, taker, 1.0), 1.0);
        assert_eq!(take(&mut w, owner, taker, 1.0), 1.0);
        assert_eq!(
            crate::minds::caching::theft::loot(&mut w, owner, taker, site, 2.0),
            2.0
        );
        assert_eq!(w.events.caches_pilfered, 2);
        let e = w.events.spatial_stores.unwrap();
        assert_eq!(e.larder.pilfer_candidates, 1);
        assert_eq!(e.scatter.pilfer_candidates, 1);
        assert_eq!(e.larder.caches_pilfered, 1);
        assert_eq!(e.scatter.caches_pilfered, 1);
        conservation(&w, StoreKind::Scatter, 4.0);
        conservation(&w, StoreKind::Larder, 6.0);
    }

    #[test]
    fn off_worlds_do_not_allocate_extension_events_or_ledger() {
        let mut w = blank_world(9, 9);
        let id = spawn(&mut w, 4, 4);
        bury(&mut w, id, 4.0);
        assert_eq!(deposit(&mut w, id, 1.0), 0.0);
        w.remove(id);
        assert!(w.events.spatial_stores.is_none());
        assert_eq!(w.cache_log.capacity(), 0);
        assert!(w.cache_open.is_empty());
        assert!(w.cache_sites.is_none());
        assert_eq!(w.events.buried, 4.0);
        assert_eq!(w.events.cache_lost, 4.0);
    }

    #[test]
    fn multiple_larder_burials_close_fifo_and_backfill_when_discovery_turns_on() {
        let (mut w, owner, _) = fixture();
        w.config.spatial_hoarding.find_larder = 0.0;
        deposit(&mut w, owner, 3.0);
        assert!(w.cache_log.is_empty());
        w.config.spatial_hoarding.find_larder = 0.25;
        w.tick = 2;
        deposit(&mut w, owner, 3.0);
        assert_eq!(recover(&mut w, owner, 4.0), 4.0);
        assert_eq!(stock(&w, owner), 2.0);
        let recovered: Vec<_> = w
            .cache_log
            .iter()
            .filter(|r| r.fate == Some((2, Fate::Dug)))
            .map(|r| (r.amount, r.buried))
            .collect();
        assert_eq!(recovered, vec![(3.0, 0), (1.0, 2)]);
        conservation(&w, StoreKind::Larder, 6.0);
        w.config.spatial_hoarding.find_larder = 0.0;
        w.remove(owner);
        assert!(w.cache_open.is_empty());
    }
    #[test]
    fn aggregate_snapshots_include_larder_stocks_and_recovery() {
        let (mut w, owner, _) = fixture();
        bury(&mut w, owner, 4.0);
        deposit(&mut w, owner, 6.0);
        recover(&mut w, owner, 2.0);
        let snapshot = crate::stats::Snapshot::of(&w);
        let cached = snapshot.caching.unwrap();
        assert_eq!(cached.cached, 8.0);
        assert_eq!(cached.buried, 10.0);
        assert_eq!(cached.dug, 2.0);
        let theft = snapshot.theft.unwrap();
        assert_eq!(theft.fate_buried, 0.8);
        assert_eq!(theft.fate_dug, 0.2);
    }
    #[test]
    fn disabled_snapshot_preserves_original_scatter_summation_order() {
        let mut w = blank_world(9, 9);
        w.config.caching.rule = CachingRule::Even;
        w.config.theft.find = 0.25;
        let first = spawn(&mut w, 4, 4);
        let second = spawn(&mut w, 4, 3);
        w.agent_mut(first).unwrap().caches.insert(0, 1e16);
        w.agent_mut(second)
            .unwrap()
            .caches
            .extend([(1, 1.0), (2, 1.0)]);
        let old_sum: f64 = w.agents().flat_map(|a| a.caches.values()).sum();
        w.events.buried = old_sum;
        let snapshot = crate::stats::Snapshot::of(&w);
        assert_eq!(snapshot.caching.unwrap().cached, old_sum);
        assert_eq!(snapshot.theft.unwrap().fate_buried, 1.0);
    }
}
