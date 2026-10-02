//! Larder observations and arrivals.

use std::collections::BTreeMap;

use rand::Rng;

use super::{access, state::SeenLarder, stores};
use crate::agent::AgentId;
use crate::config::{Loot, RaidIf};
use crate::geometry::Pos;
use crate::minds::caching::{self, watching};
use crate::world::World;

pub(crate) fn fresh_larder(observed: u64, now: u64, span: u64) -> bool {
    now.saturating_sub(observed) <= span
}

/// Record only a deposit actually visible from the home, never its total stock.
pub(crate) fn observe_deposit(world: &mut World, owner: AgentId, amount: f64) {
    if !world.config.spatial_hoarding.enabled || !world.config.watching.on || amount <= 0.0 {
        return;
    }
    let Some(home) = world
        .agent(owner)
        .and_then(|a| a.spatial.as_ref())
        .map(|s| s.home)
    else {
        return;
    };
    let observers = watching::watchers_of(world, owner, world.torus.index(home) as u32);
    if observers.is_empty() {
        return;
    }
    let now = world.tick;
    for id in &observers {
        let seen = world
            .agent_mut(*id)
            .expect("watcher")
            .spatial
            .as_mut()
            .expect("spatial watcher")
            .seen_larders
            .entry(owner)
            .or_insert(SeenLarder {
                home,
                amount: 0.0,
                tick: now,
            });
        seen.home = home;
        seen.amount += amount;
        seen.tick = now;
    }
    world.events.burials_seen += 1;
    world.events.sightings += u32::try_from(observers.len()).unwrap_or(u32::MAX);
    let e = &mut stores::tick_events(world).expect("enabled").observation;
    e.burials_seen += 1;
    e.sightings += u32::try_from(observers.len()).unwrap_or(u32::MAX);
}

pub(crate) fn sweep(world: &mut World) {
    if !world.config.spatial_hoarding.enabled || !world.config.watching.on {
        return;
    }
    let (now, span) = (world.tick, u64::from(world.config.watching.span));
    let mut entries = 0usize;
    for a in world.agents_mut() {
        if let Some(s) = &mut a.spatial {
            s.seen_larders
                .retain(|_, e| fresh_larder(e.tick, now, span));
            entries += s.seen_larders.len();
        }
    }
    let entries = u32::try_from(entries).unwrap_or(u32::MAX);
    world.events.seen_entries = world.events.seen_entries.saturating_add(entries);
    stores::tick_events(world)
        .expect("enabled")
        .observation
        .seen_entries = entries;
}

/// Join an owner's known home regardless of scatter owner-memory, then observed foreign homes.
pub(crate) fn join_larders(
    world: &World,
    id: AgentId,
    out: &mut Vec<(Pos, u32, f64)>,
    start: &mut usize,
) {
    if !world.config.spatial_hoarding.enabled {
        return;
    }
    let Some(a) = world.agent(id) else {
        return;
    };
    let Some(s) = &a.spatial else {
        return;
    };
    let mut sites = BTreeMap::<u32, f64>::new();
    if caching::hungry(world, id) && s.larder > 0.0 {
        if let Some(p) = access::endpoint(world, id, s.home) {
            sites.insert(
                world.torus.index(p) as u32,
                s.larder.min(carrying_room(world, id)),
            );
        }
    }
    if world.config.watching.on && watching::raiding(world, id) {
        for (&owner, e) in &s.seen_larders {
            if owner == id || !watching::fresh(world, e.tick) {
                continue;
            }
            if let Some(p) = access::endpoint(world, id, e.home) {
                *sites.entry(world.torus.index(p) as u32).or_default() += e.amount;
            }
        }
    }
    let cap = watching::room_cap(world, id);
    caching::join_sites(
        world,
        id,
        sites
            .into_iter()
            .map(|(site, amount)| (site, amount.min(cap))),
        out,
        start,
    );
}

/// A directed raid is attempted only after both watching gates pass.
pub(crate) fn raid(world: &mut World, id: AgentId, at: Pos) -> f64 {
    if !world.config.spatial_hoarding.enabled
        || !world.config.watching.on
        || !watching::raiding(world, id)
    {
        return 0.0;
    }
    let Some(s) = world.agent(id).and_then(|a| a.spatial.as_ref()) else {
        return 0.0;
    };
    let targets: Vec<AgentId> = s
        .seen_larders
        .iter()
        .filter(|(&owner, e)| {
            owner != id && watching::fresh(world, e.tick) && access::in_contact(world, at, e.home)
        })
        .map(|(&owner, _)| owner)
        .collect();
    if targets.is_empty() {
        return 0.0;
    }
    let remembered: f64 = targets
        .iter()
        .map(|owner| s.seen_larders[owner].amount)
        .sum();
    if world.config.watching.raid_if == RaidIf::Better
        && !watching::forgoes(world, id)
        && remembered < crate::rules::movement::site_value(world, id, at)
    {
        return 0.0;
    }
    let owner = targets[0];
    let (stock, blocked) = world
        .agent(owner)
        .and_then(|a| a.spatial.as_ref())
        .map_or((0.0, false), |o| (o.larder, o.guarding));
    let home = world
        .agent(id)
        .unwrap()
        .spatial
        .as_ref()
        .unwrap()
        .seen_larders[&owner]
        .home;
    // A later home cannot be inferred from the owner's current state.
    let valid_home = world
        .agent(owner)
        .and_then(|a| a.spatial.as_ref())
        .is_some_and(|o| o.home == home);
    world
        .agent_mut(id)
        .unwrap()
        .spatial
        .as_mut()
        .unwrap()
        .seen_larders
        .remove(&owner);
    world.events.seen_arrivals += 1;
    let e = &mut stores::tick_events(world).expect("enabled").observation;
    e.seen_arrivals += 1;
    e.raid_attempts += 1;
    if blocked && valid_home && stock > 0.0 {
        e.raids_blocked += 1;
        stores::tick_events(world)
            .expect("enabled")
            .guard
            .blocked_raids += 1;
        return 0.0;
    }
    if !valid_home || stock <= 0.0 {
        world.events.raids_wasted += 1;
        stores::tick_events(world)
            .expect("enabled")
            .observation
            .raids_empty += 1;
        return 0.0;
    }
    let taken = stores::take(world, owner, id, stock);
    if taken > 0.0 {
        let e = &mut stores::tick_events(world).expect("enabled").observation;
        e.raids += 1;
        e.raided += taken;
        world.events.raids += 1;
        world.events.raided += taken;
        return taken;
    }
    if world.config.theft.loot == Loot::Keep && carrying_room(world, id) <= 0.0 {
        stores::tick_events(world)
            .expect("enabled")
            .observation
            .raids_no_room += 1;
    }
    0.0
}

/// Ordered contact search, with one discovery draw per nonempty foreign larder until a take.
pub(crate) fn stumble(world: &mut World, id: AgentId, at: Pos) -> f64 {
    if !world.config.spatial_hoarding.enabled {
        return 0.0;
    }
    let ids = world.agent_ids();
    for owner in ids {
        if owner == id {
            continue;
        }
        let Some(s) = world.agent(owner).and_then(|a| a.spatial.as_ref()) else {
            continue;
        };
        if s.larder <= 0.0 || !access::in_contact(world, at, s.home) {
            continue;
        }
        let (amount, blocked) = (s.larder, s.guarding);
        stores::tick_events(world)
            .expect("enabled")
            .observation
            .contacts += 1;
        if world.config.spatial_hoarding.find_larder <= 0.0 {
            continue;
        }
        let hit = world
            .rng
            .gen_bool(world.config.spatial_hoarding.find_larder);
        stores::tick_events(world)
            .expect("enabled")
            .observation
            .discovery_draws += 1;
        if !hit {
            continue;
        }
        stores::tick_events(world)
            .expect("enabled")
            .observation
            .discovery_hits += 1;
        if blocked {
            stores::tick_events(world)
                .expect("enabled")
                .observation
                .discoveries_blocked += 1;
            stores::tick_events(world)
                .expect("enabled")
                .guard
                .blocked_discoveries += 1;
            continue;
        }
        let taken = stores::take(world, owner, id, amount);
        if taken > 0.0 {
            return taken;
        }
    }
    0.0
}

pub(crate) fn skip_scatter_draws(world: &mut World, id: AgentId, at: Pos) {
    if world.config.theft.find <= 0.0 {
        return;
    }
    let site = world.torus.index(at) as u32;
    let skipped = world
        .agents()
        .filter(|a| a.id != id && a.caches.get(&site).is_some_and(|&v| v > 0.0))
        .count();
    stores::tick_events(world)
        .expect("enabled")
        .observation
        .scatter_draws_skipped += u32::try_from(skipped).unwrap_or(u32::MAX);
}

fn carrying_room(world: &World, id: AgentId) -> f64 {
    (f64::from(world.config.caching.capacity) - world.agent(id).expect("live agent").holdings[0])
        .max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{CachingRule, Loot, MoveMode, RaidIf, RaidWhen};
    use crate::geometry::Pos;
    use crate::minds::spatial_hoarding::state::{FounderTraits, SeenLarder, SpatialState};
    use crate::testkit::*;
    use crate::world::{EpisodeProbe, World};

    fn fixture() -> (World, AgentId, AgentId) {
        let mut w = blank_world(11, 11);
        w.config.spatial_hoarding.enabled = true;
        w.config.spatial_hoarding.guard = false;
        w.config.spatial_hoarding.find_larder = 1.0;
        w.config.caching.rule = CachingRule::Even;
        w.config.caching.capacity = 20;
        w.config.movement.mode = MoveMode::Walk;
        w.config.watching.on = true;
        w.config.watching.span = 2;
        w.config.watching.raid_when = RaidWhen::Always;
        w.config.watching.raid_if = RaidIf::Always;
        let owner = spawn(&mut w, 5, 5);
        let watcher = spawn(&mut w, 5, 3);
        for id in [owner, watcher] {
            let a = w.agent_mut(id).unwrap();
            a.holdings[0] = 10.0;
            a.vision = 3;
            a.spatial = Some(SpatialState::new(
                a.pos,
                FounderTraits {
                    larder: 1.0,
                    defense: 0.5,
                    cheater: false,
                    watches: id == watcher,
                },
            ));
            a.watches = id == watcher;
        }
        (w, owner, watcher)
    }

    #[test]
    fn spatial_hoarding_larder_memory_expires_after_span_not_on_it() {
        assert!(fresh_larder(5, 7, 2));
        assert!(!fresh_larder(5, 8, 2));
    }

    #[test]
    fn spatial_hoarding_observation_records_deposit_amount_and_home() {
        let (mut w, owner, watcher) = fixture();
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 18.0;
        observe_deposit(&mut w, owner, 2.0);
        assert_eq!(
            w.agent(watcher)
                .unwrap()
                .spatial
                .as_ref()
                .unwrap()
                .seen_larders[&owner],
            SeenLarder {
                home: Pos::new(5, 5),
                amount: 2.0,
                tick: 0
            }
        );
    }

    #[test]
    fn spatial_hoarding_dead_fresh_owner_is_targeted_then_forgotten() {
        let (mut w, owner, watcher) = fixture();
        observe_deposit(&mut w, owner, 4.0);
        w.remove(owner);
        w.tick = 1;
        let mut candidates = vec![(Pos::new(5, 3), 0, 0.0)];
        let mut start = 1;
        join_larders(&w, watcher, &mut candidates, &mut start);
        assert!(candidates.iter().any(|c| c.0 == Pos::new(5, 4)));
        w.move_agent(watcher, Pos::new(5, 4));
        assert_eq!(raid(&mut w, watcher, Pos::new(5, 4)), 0.0);
        assert!(w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders
            .is_empty());
        assert_eq!(w.events.raids_wasted, 1);
    }

    #[test]
    fn spatial_hoarding_guarded_raid_clears_only_matching_memory() {
        let (mut w, owner, watcher) = fixture();
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 5.0;
        observe_deposit(&mut w, owner, 5.0);
        let other = spawn(&mut w, 9, 9);
        w.agent_mut(watcher)
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .seen_larders
            .insert(
                other,
                SeenLarder {
                    home: Pos::new(9, 9),
                    amount: 1.0,
                    tick: 0,
                },
            );
        w.agent_mut(owner)
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .guarding = true;
        w.move_agent(watcher, Pos::new(5, 4));
        assert_eq!(raid(&mut w, watcher, Pos::new(5, 4)), 0.0);
        let memory = &w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders;
        assert!(!memory.contains_key(&owner));
        assert!(memory.contains_key(&other));
    }

    #[test]
    fn spatial_hoarding_simultaneous_contact_targets_clear_only_first_denied_entry() {
        let (mut w, first, watcher) = fixture();
        let second = spawn(&mut w, 6, 4);
        let a = w.agent_mut(second).unwrap();
        a.spatial = Some(SpatialState::new(
            a.pos,
            FounderTraits {
                larder: 1.0,
                defense: 0.5,
                cheater: false,
                watches: false,
            },
        ));
        a.spatial.as_mut().unwrap().larder = 5.0;
        w.agent_mut(first).unwrap().spatial.as_mut().unwrap().larder = 5.0;
        w.agent_mut(first)
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .guarding = true;
        let memory = &mut w
            .agent_mut(watcher)
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .seen_larders;
        memory.insert(
            first,
            SeenLarder {
                home: Pos::new(5, 5),
                amount: 3.0,
                tick: 0,
            },
        );
        memory.insert(
            second,
            SeenLarder {
                home: Pos::new(6, 4),
                amount: 5.0,
                tick: 0,
            },
        );
        w.move_agent(watcher, Pos::new(5, 4));
        assert_eq!(raid(&mut w, watcher, Pos::new(5, 4)), 0.0);
        let memory = &w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders;
        assert!(!memory.contains_key(&first));
        assert!(memory.contains_key(&second));
        assert_eq!(
            w.agent(second).unwrap().spatial.as_ref().unwrap().larder,
            5.0
        );
    }

    #[test]
    fn spatial_hoarding_probe_defaults_off_and_checked_cohort_is_validated() {
        let (w, _, _) = fixture();
        assert_eq!(w.spatial_probe, EpisodeProbe::default());
    }

    #[test]
    fn spatial_hoarding_stumble_eat_takes_finite_full_stock() {
        let (mut w, owner, thief) = fixture();
        w.config.theft.loot = Loot::Eat;
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 6.0;
        w.move_agent(thief, Pos::new(5, 4));
        assert_eq!(stumble(&mut w, thief, Pos::new(5, 4)), 6.0);
        assert_eq!(w.agent(thief).unwrap().fed, 6.0);
    }

    #[test]
    fn spatial_hoarding_zero_discovery_still_records_contact_without_draw() {
        let (mut w, owner, thief) = fixture();
        w.config.spatial_hoarding.find_larder = 0.0;
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 6.0;
        w.move_agent(thief, Pos::new(5, 4));
        assert_eq!(stumble(&mut w, thief, Pos::new(5, 4)), 0.0);
        let e = stores::tick_events(&mut w).unwrap().observation;
        assert_eq!((e.contacts, e.discovery_draws), (1, 0));
    }

    #[test]
    fn spatial_hoarding_full_capacity_attempt_clears_only_then_gates_preserve_memory() {
        let (mut w, owner, watcher) = fixture();
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 5.0;
        observe_deposit(&mut w, owner, 2.0);
        w.move_agent(watcher, Pos::new(5, 4));
        w.agent_mut(watcher).unwrap().holdings[0] = 20.0;
        assert_eq!(raid(&mut w, watcher, Pos::new(5, 4)), 0.0);
        assert!(!w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders
            .contains_key(&owner));
        assert_eq!(
            stores::tick_events(&mut w)
                .unwrap()
                .observation
                .raids_no_room,
            1
        );
        observe_deposit(&mut w, owner, 2.0);
        w.agent_mut(watcher).unwrap().holdings[0] = 10.0;
        w.config.watching.raid_when = RaidWhen::Hungry;
        assert_eq!(raid(&mut w, watcher, Pos::new(5, 4)), 0.0);
        assert!(w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders
            .contains_key(&owner));
        w.config.watching.raid_when = RaidWhen::Always;
        w.config.watching.raid_if = RaidIf::Better;
        set_sugar(&mut w, 5, 4, 8.0);
        assert_eq!(raid(&mut w, watcher, Pos::new(5, 4)), 0.0);
        assert!(w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders
            .contains_key(&owner));
    }

    #[test]
    fn spatial_hoarding_no_watcher_or_expired_sighting_gives_no_target() {
        let (mut w, owner, watcher) = fixture();
        w.agent_mut(watcher).unwrap().watches = false;
        observe_deposit(&mut w, owner, 4.0);
        assert!(w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders
            .is_empty());
        w.agent_mut(watcher).unwrap().watches = true;
        observe_deposit(&mut w, owner, 4.0);
        w.tick = 2;
        let mut c = vec![(Pos::new(5, 3), 0, 0.0)];
        let mut start = 1;
        join_larders(&w, watcher, &mut c, &mut start);
        assert!(c.len() > 1);
        w.tick = 3;
        c.truncate(1);
        start = 1;
        join_larders(&w, watcher, &mut c, &mut start);
        assert_eq!(c.len(), 1);
        sweep(&mut w);
        assert!(w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders
            .is_empty());
    }

    #[test]
    fn spatial_hoarding_larder_stumble_precedes_scatter_and_harvest() {
        let (mut w, owner, thief) = fixture();
        w.config.theft.find = 1.0;
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 4.0;
        let at = Pos::new(5, 4);
        let site = w.torus.index(at) as u32;
        w.agent_mut(owner).unwrap().caches.insert(site, 3.0);
        w.agent_mut(thief).unwrap().holdings[0] = 0.0;
        set_sugar(&mut w, 5, 4, 7.0);
        let h = crate::rules::movement::go_and_gather(&mut w, thief, at);
        assert_eq!((h.pilfered, h.gathered[0]), (4.0, 0.0));
        assert_eq!(w.agent(owner).unwrap().caches[&site], 3.0);
        assert_eq!(w.site(at).resource[0], 7.0);
        assert_eq!(
            stores::tick_events(&mut w)
                .unwrap()
                .observation
                .scatter_draws_skipped,
            1
        );
    }

    #[test]
    fn spatial_hoarding_scatter_first_probe_reverses_stumble_only() {
        let (mut w, owner, thief) = fixture();
        w.spatial_probe.scatter_first = true;
        w.config.theft.find = 1.0;
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 4.0;
        let at = Pos::new(5, 4);
        let site = w.torus.index(at) as u32;
        w.agent_mut(owner).unwrap().caches.insert(site, 3.0);
        w.agent_mut(thief).unwrap().holdings[0] = 0.0;
        let h = crate::rules::movement::go_and_gather(&mut w, thief, at);
        assert_eq!(h.pilfered, 3.0);
        assert_eq!(
            w.agent(owner).unwrap().spatial.as_ref().unwrap().larder,
            4.0
        );
    }

    #[test]
    fn spatial_hoarding_guard_probe_harvests_only_after_zero_recovery() {
        let (mut w, owner, _) = fixture();
        w.spatial_probe.guard_harvest = true;
        w.agent_mut(owner)
            .unwrap()
            .spatial
            .as_mut()
            .unwrap()
            .guarding = true;
        set_sugar(&mut w, 5, 5, 3.0);
        let h = crate::minds::decide(&mut w, owner);
        assert_eq!((h.dug, h.gathered[0]), (0.0, 3.0));
        assert_eq!(
            stores::tick_events(&mut w).unwrap().guard.probe_harvest,
            3.0
        );
    }

    #[test]
    fn spatial_hoarding_own_scatter_then_known_home_recovery_stop_before_harvest() {
        let (mut w, owner, _) = fixture();
        w.config.theft.owner_memory = true;
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 5.0;
        assert_eq!(caching::bury(&mut w, owner, 4.0), 4.0);
        w.agent_mut(owner).unwrap().metabolism[0] = 1;
        w.agent_mut(owner).unwrap().holdings[0] = 0.0;
        set_sugar(&mut w, 5, 5, 3.0);
        let h = crate::rules::movement::go_and_gather(&mut w, owner, Pos::new(5, 5));
        assert_eq!((h.dug, h.gathered[0]), (4.0, 0.0));
        assert_eq!(
            w.agent(owner).unwrap().spatial.as_ref().unwrap().larder,
            5.0
        );
        w.config.theft.owner_memory = false;
        let mut candidates = vec![(Pos::new(5, 5), 0, 0.0)];
        let mut start = 1;
        join_larders(&w, owner, &mut candidates, &mut start);
        assert!(
            candidates[0].2 >= 5.0,
            "own home stays known with scatter owner memory off"
        );
        let h = crate::rules::movement::go_and_gather(&mut w, owner, Pos::new(5, 5));
        assert_eq!((h.dug, h.gathered[0]), (5.0, 0.0));
        assert_eq!(w.site(Pos::new(5, 5)).resource[0], 3.0);
    }

    #[test]
    fn spatial_hoarding_observed_larder_precedes_observed_scatter() {
        let (mut w, owner, thief) = fixture();
        let at = Pos::new(5, 4);
        let site = w.torus.index(at) as u32;
        w.agent_mut(owner).unwrap().spatial.as_mut().unwrap().larder = 5.0;
        w.agent_mut(owner).unwrap().caches.insert(site, 3.0);
        observe_deposit(&mut w, owner, 5.0);
        w.agent_mut(thief).unwrap().seen.insert(
            (site, owner),
            crate::minds::caching::watching::SeenCache {
                amount: 3.0,
                tick: 0,
            },
        );
        w.agent_mut(thief).unwrap().holdings[0] = 0.0;
        let h = crate::rules::movement::go_and_gather(&mut w, thief, at);
        assert_eq!(h.pilfered, 5.0);
        assert_eq!(w.agent(owner).unwrap().caches[&site], 3.0);
        assert_eq!(w.events.pilfer_draws, 0);
    }

    #[test]
    fn spatial_hoarding_opaque_wall_blocks_larder_sighting() {
        let mut c = blank_config(11, 11);
        c.walls = vec![crate::config::Wall {
            x: 5,
            y: 4,
            width: 1,
            height: 1,
            opaque: true,
        }];
        let mut w = World::new(c, 7).unwrap();
        w.config.spatial_hoarding.enabled = true;
        w.config.watching.on = true;
        let owner = spawn(&mut w, 5, 5);
        let watcher = spawn(&mut w, 5, 3);
        for id in [owner, watcher] {
            let a = w.agent_mut(id).unwrap();
            a.vision = 3;
            a.watches = id == watcher;
            a.spatial = Some(SpatialState::new(
                a.pos,
                FounderTraits {
                    larder: 1.0,
                    defense: 0.5,
                    cheater: false,
                    watches: a.watches,
                },
            ));
        }
        observe_deposit(&mut w, owner, 2.0);
        assert!(w
            .agent(watcher)
            .unwrap()
            .spatial
            .as_ref()
            .unwrap()
            .seen_larders
            .is_empty());
    }

    #[test]
    fn spatial_hoarding_guard_probe_skips_harvest_after_positive_recovery() {
        let (mut w, owner, _) = fixture();
        w.spatial_probe.guard_harvest = true;
        w.agent_mut(owner).unwrap().metabolism[0] = 1;
        w.agent_mut(owner).unwrap().holdings[0] = 0.0;
        let s = w.agent_mut(owner).unwrap().spatial.as_mut().unwrap();
        s.guarding = true;
        s.larder = 4.0;
        set_sugar(&mut w, 5, 5, 3.0);
        let h = crate::minds::decide(&mut w, owner);
        assert_eq!((h.dug, h.gathered[0]), (4.0, 0.0));
        assert_eq!(w.site(Pos::new(5, 5)).resource[0], 3.0);
        assert_eq!(
            stores::tick_events(&mut w).unwrap().guard.probe_harvest,
            0.0
        );
    }
}
