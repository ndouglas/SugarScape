//! Minds 3: memory. A rememberer keeps what it has seen of a site after it
//! leaves sight, so it can walk back to what it recalls rather than only
//! what's currently in view.

use std::collections::BTreeMap;

use crate::agent::AgentId;
use crate::config::{Belief, MemoryPrior, MAX_GOODS};
use crate::geometry::Pos;
use crate::world::World;

/// Most sites one Flump remembers at once. When `observe` would push its
/// memory past this, the entries seen longest ago go first (on a tie, the
/// lowest site index). Before the cap, a big flat world with a long span let
/// memory grow without bound: the final review measured 1.29 M entries and
/// about 220 MB on a 500×500 world with span 10 000 at tick 1 500, before
/// keyframes. The standard presets hold far fewer per Flump than this, so the
/// cap never binds on them.
pub const MEMORY_CAP: usize = 4096;

/// A remembered site (Minds 3): the levels seen, the most ever seen there, when it was last seen, and
/// what the Flump knows of a truffle spot there.
///
/// Levels and most are kept only for the configured goods (one boxed slice,
/// levels then most, `n` each), not for all `MAX_GOODS`: the values are the
/// same `f64`s as ever, just without the unused slots.
#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    vals: Box<[f64]>,
    pub tick: u64,
    pub truffle: Option<TruffleSeen>,
}

impl Seen {
    /// A site seen at `tick` with these levels and most (the same length:
    /// one per configured good), and no known truffle spot.
    pub fn new(levels: &[f64], most: &[f64], tick: u64) -> Self {
        assert_eq!(levels.len(), most.len(), "one level and one most per good");
        let vals = levels.iter().chain(most).copied().collect();
        Self {
            vals,
            tick,
            truffle: None,
        }
    }

    /// The levels last seen, one per configured good.
    pub fn levels(&self) -> &[f64] {
        &self.vals[..self.vals.len() / 2]
    }

    /// The most ever seen, one per configured good.
    pub fn most(&self) -> &[f64] {
        &self.vals[self.vals.len() / 2..]
    }

    /// Records `levels` seen at `tick`, raising `most` where they're higher.
    fn update(&mut self, levels: &[f64], tick: u64) {
        let n = self.vals.len() / 2;
        let (seen, most) = self.vals.split_at_mut(n);
        seen.copy_from_slice(levels);
        for (m, &l) in most.iter_mut().zip(levels) {
            *m = m.max(l);
        }
        self.tick = tick;
    }
}

/// A known truffle spot: whether it was ripe when last seen, and when that was (a harvest counts as seen unripe).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TruffleSeen {
    pub ripe: bool,
    pub tick: u64,
}

/// A Flump's remembered sites, by site index (deterministic order), at most
/// [`MEMORY_CAP`] of them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory {
    pub sites: BTreeMap<u32, Seen>,
}

/// After the move: a rememberer records every site in its sight and its own
/// site. Builds the observed list first (immutable borrow of `world`), then
/// writes it into the agent's memory. Runs `forget` every 16 ticks per agent;
/// Task 4's candidates also forget lazily. Then, if the memory holds more
/// than [`MEMORY_CAP`] sites, drops the ones seen longest ago (lowest site
/// index first on a tie) until it holds exactly the cap.
pub(crate) fn observe(world: &mut World, id: AgentId) {
    let agent = world.agent(id).expect("live agent");
    if !agent.remembers {
        return;
    }
    let pos = agent.pos;
    let vision = agent.vision;
    let n = world.config.goods.len();
    let now = world.tick;
    let span = world.config.memory.span;

    let mut positions: Vec<Pos> = world
        .sight(pos, vision)
        .into_iter()
        .map(|(p, _)| p)
        .collect();
    positions.push(pos);

    let observed: Vec<(u32, [f64; MAX_GOODS])> = positions
        .into_iter()
        .map(|p| {
            let idx = world.torus.index(p) as u32;
            let mut levels = [0.0; MAX_GOODS];
            levels[..n].copy_from_slice(&world.site(p).resource[..n]);
            (idx, levels)
        })
        .collect();

    // The Flump's own site's truffle spot, if it has one (a harvest this
    // turn already left it unripe by now, so this sees that).
    let own_idx = world.torus.index(pos) as u32;
    let own_truffle = world
        .truffle(pos)
        .map(|ripe| TruffleSeen { ripe, tick: now });

    let agent = world.agent_mut(id).expect("live agent");
    let zeros = [0.0; MAX_GOODS];
    for (idx, levels) in observed {
        let entry = agent
            .memory
            .sites
            .entry(idx)
            .or_insert_with(|| Seen::new(&zeros[..n], &zeros[..n], now));
        entry.update(&levels[..n], now);
        if idx == own_idx {
            if let Some(t) = own_truffle {
                entry.truffle = Some(t);
            }
        }
    }
    if now.is_multiple_of(16) {
        forget(&mut agent.memory, now, span);
    }
    cap(&mut agent.memory, MEMORY_CAP);
}

/// Drops the entries seen longest ago (lowest site index first on a tie)
/// until `memory` holds at most `limit` sites.
pub(crate) fn cap(memory: &mut Memory, limit: usize) {
    let excess = memory.sites.len().saturating_sub(limit);
    if excess == 0 {
        return;
    }
    let mut order: Vec<(u64, u32)> = memory
        .sites
        .iter()
        .map(|(&i, seen)| (seen.tick, i))
        .collect();
    order.select_nth_unstable(excess - 1);
    for &(_, i) in &order[..excess] {
        memory.sites.remove(&i);
    }
}

/// Minds 4: `memory.prior: map`. At world creation, after founders are
/// placed, gives every founder that remembers a memory of every non-wall
/// site as the world starts: its opening level of each good recorded as
/// both `levels` and `most`, at tick 0, with no truffle knowledge (knowing
/// the map isn't knowing where the truffles are). A no-op when `prior`
/// isn't `Map`, or for an agent that doesn't remember. Draws nothing from
/// any RNG. Children and replacements never call this, so they still start
/// empty, as in Minds 3.
///
/// Respects [`MEMORY_CAP`] (Review Focus 4): on a world with more non-wall
/// sites than the cap, keeps the [`MEMORY_CAP`] sites with the highest
/// starting level of good 0, ties going to the lower site index — see
/// [`keep_richest`].
pub(crate) fn know_the_map(world: &mut World) {
    if world.config.memory.prior != MemoryPrior::Map {
        return;
    }
    let n = world.config.goods.len();
    let entries: Vec<(f64, u32)> = (0..world.sites.len())
        .filter(|&i| world.walls[i] == 0)
        .map(|i| (world.sites[i].resource[0], i as u32))
        .collect();
    let kept = keep_richest(entries, MEMORY_CAP);
    let prior: BTreeMap<u32, Seen> = kept
        .into_iter()
        .map(|(_, i)| {
            let levels = &world.sites[i as usize].resource[..n];
            (i, Seen::new(levels, levels, 0))
        })
        .collect();
    let founders: Vec<AgentId> = world
        .agents()
        .filter(|a| a.remembers)
        .map(|a| a.id)
        .collect();
    for id in founders {
        world.agent_mut(id).expect("live founder").memory.sites = prior.clone();
    }
}

/// Keeps the `cap` best of `entries` (a site's good-0 level and its index),
/// highest level first, ties going to the lower site index. A single
/// `select_nth_unstable_by` partition, not a full sort, and a no-op once
/// `entries` is at or under `cap`.
fn keep_richest(mut entries: Vec<(f64, u32)>, cap: usize) -> Vec<(f64, u32)> {
    let excess = entries.len().saturating_sub(cap);
    if excess > 0 {
        entries.select_nth_unstable_by(excess - 1, |a, b| {
            a.0.total_cmp(&b.0).then_with(|| b.1.cmp(&a.1))
        });
        entries.drain(..excess);
    }
    entries
}

/// Drops entries last seen more than `span` ticks ago (an entry at age
/// exactly `span` stays).
pub(crate) fn forget(memory: &mut Memory, now: u64, span: u32) {
    let span = u64::from(span);
    memory
        .sites
        .retain(|_, seen| now.saturating_sub(seen.tick) <= span);
}

/// The believed level of good `i` at `site` now, under `belief`. `recall`
/// always answers what was last seen; `project` grows it at `rate` per tick
/// since then, capped at the most ever seen, or jumps straight to the most
/// ever seen once `instant` growback has had a tick to act.
pub fn believed_level(
    seen: &Seen,
    i: usize,
    now: u64,
    rate: f64,
    instant: bool,
    belief: Belief,
) -> f64 {
    let (levels, most) = (seen.levels(), seen.most());
    if belief == Belief::Recall {
        return levels[i];
    }
    let age = now.saturating_sub(seen.tick);
    if age == 0 {
        return levels[i];
    }
    if instant {
        most[i]
    } else {
        (levels[i] + rate * age as f64).min(most[i])
    }
}

/// Whether a known truffle spot is believed ripe now, under `belief`.
/// `recall` answers exactly what was seen; `project` also believes it has
/// had time to regrow once `regrow` ticks have passed since it was last seen
/// unripe.
pub fn believed_ripe(t: &TruffleSeen, now: u64, regrow: u32, belief: Belief) -> bool {
    match belief {
        Belief::Recall => t.ripe,
        Belief::Project => t.ripe || now.saturating_sub(t.tick) >= u64::from(regrow),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MoveMode;
    use crate::rng::seeded;
    use crate::testkit::*;
    use rand::Rng;

    fn seen_at(levels: f64, most: f64, tick: u64) -> Seen {
        Seen::new(&[levels], &[most], tick)
    }

    fn memory_world(width: u32, height: u32, span: u32) -> World {
        let mut c = blank_config(width, height);
        c.movement.mode = MoveMode::Walk;
        c.memory.span = span;
        c.memory.share = 1.0;
        World::new(c, 7).expect("valid config")
    }

    // --- observe ---

    #[test]
    fn observe_records_own_site_and_sight_with_running_max() {
        let mut w = memory_world(10, 10, 50);
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().remembers = true;
        w.agent_mut(id).unwrap().vision = 2;
        set_sugar(&mut w, 5, 5, 3.0);
        set_sugar(&mut w, 6, 5, 4.0);
        w.tick = 10;
        observe(&mut w, id);
        let mem = &w.agent(id).unwrap().memory;
        // Own site plus the 8 sites in vision 2 on each of 4 axes.
        assert_eq!(mem.sites.len(), 9);
        let own = w.torus.index(Pos::new(5, 5)) as u32;
        let east = w.torus.index(Pos::new(6, 5)) as u32;
        assert_eq!(mem.sites[&own].levels()[0], 3.0);
        assert_eq!(mem.sites[&own].most()[0], 3.0);
        assert_eq!(mem.sites[&own].tick, 10);
        assert_eq!(mem.sites[&east].levels()[0], 4.0);
    }

    #[test]
    fn observe_does_nothing_for_a_non_rememberer() {
        let mut w = memory_world(10, 10, 50);
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().remembers = false;
        set_sugar(&mut w, 5, 5, 3.0);
        observe(&mut w, id);
        assert!(w.agent(id).unwrap().memory.sites.is_empty());
    }

    #[test]
    fn a_second_observe_updates_levels_and_tick_and_only_raises_most() {
        let mut w = memory_world(10, 10, 50);
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().remembers = true;
        set_sugar(&mut w, 5, 5, 5.0);
        w.tick = 3;
        observe(&mut w, id);
        set_sugar(&mut w, 5, 5, 2.0);
        w.tick = 9;
        observe(&mut w, id);
        let own = w.torus.index(Pos::new(5, 5)) as u32;
        let seen = &w.agent(id).unwrap().memory.sites[&own];
        assert_eq!(seen.levels()[0], 2.0, "the level tracks what's current");
        assert_eq!(seen.tick, 9);
        assert_eq!(seen.most()[0], 5.0, "most stays at the higher value seen");
    }

    // --- forget ---

    #[test]
    fn forget_drops_only_entries_older_than_span() {
        let mut m = Memory::default();
        m.sites.insert(1, seen_at(1.0, 1.0, 0));
        m.sites.insert(2, seen_at(1.0, 1.0, 9));
        forget(&mut m, 10, 10);
        assert!(m.sites.contains_key(&1), "age exactly span stays");
        assert!(m.sites.contains_key(&2));
        let mut m2 = Memory::default();
        m2.sites.insert(1, seen_at(1.0, 1.0, 0)); // age 11
        forget(&mut m2, 11, 10);
        assert!(!m2.sites.contains_key(&1), "age span + 1 is gone");
    }

    // --- believed_level ---

    #[test]
    fn believed_level_recall_returns_levels() {
        let seen = seen_at(2.0, 9.0, 0);
        assert_eq!(
            believed_level(&seen, 0, 20, 0.5, false, Belief::Recall),
            2.0
        );
    }

    #[test]
    fn believed_level_project_grows_at_rate_capped_at_most() {
        let seen = seen_at(2.0, 9.0, 0);
        // age 5, rate 1: 2 + 5 = 7, under the cap of 9.
        assert_eq!(
            believed_level(&seen, 0, 5, 1.0, false, Belief::Project),
            7.0
        );
        // age 20, rate 1: 2 + 20 = 22, capped at most (9).
        assert_eq!(
            believed_level(&seen, 0, 20, 1.0, false, Belief::Project),
            9.0
        );
    }

    #[test]
    fn believed_level_instant_gives_most() {
        let seen = seen_at(2.0, 9.0, 0);
        assert_eq!(believed_level(&seen, 0, 5, 1.0, true, Belief::Project), 9.0);
    }

    #[test]
    fn believed_level_at_age_zero_is_levels_under_either_belief() {
        let seen = seen_at(2.0, 9.0, 10);
        assert_eq!(
            believed_level(&seen, 0, 10, 1.0, true, Belief::Project),
            2.0
        );
        assert_eq!(
            believed_level(&seen, 0, 10, 1.0, false, Belief::Recall),
            2.0
        );
    }

    // --- believed_ripe ---

    #[test]
    fn believed_ripe_recall_returns_what_was_seen() {
        let ripe = TruffleSeen {
            ripe: true,
            tick: 0,
        };
        let unripe = TruffleSeen {
            ripe: false,
            tick: 0,
        };
        assert!(believed_ripe(&ripe, 100, 30, Belief::Recall));
        assert!(!believed_ripe(&unripe, 100, 30, Belief::Recall));
    }

    #[test]
    fn believed_ripe_project_believes_regrowth_after_enough_age() {
        let unripe = TruffleSeen {
            ripe: false,
            tick: 10,
        };
        assert!(!believed_ripe(&unripe, 39, 30, Belief::Project), "age 29");
        assert!(believed_ripe(&unripe, 40, 30, Belief::Project), "age 30");
        let ripe = TruffleSeen {
            ripe: true,
            tick: 10,
        };
        assert!(believed_ripe(&ripe, 10, 30, Belief::Project), "seen ripe");
    }

    // --- the `remembers` draw ---

    #[test]
    fn the_remembers_draw_is_one_extra_draw_after_everything_else() {
        use crate::agent::Agent;
        use crate::config::Config;
        let off = Config::default();
        assert_eq!(off.memory.span, 0, "the default is off");
        let mut on = off.clone();
        on.movement.mode = MoveMode::Walk;
        on.memory.span = 20;
        // Share 0.5, so the draw really consumes the stream (a share of 0 or
        // 1 answers without drawing).
        on.memory.share = 0.5;
        for seed in 0..32 {
            let mut rng_off = seeded(seed);
            let a = Agent::random(&off, Pos::new(0, 0), 0, &mut rng_off);
            let mut rng_on = seeded(seed);
            let b = Agent::random(&on, Pos::new(0, 0), 0, &mut rng_on);
            assert!(!a.remembers, "span 0 never remembers");
            let mut b_as_off = b.clone();
            b_as_off.remembers = false;
            assert_eq!(a, b_as_off, "seed {seed}: only `remembers` differs");
            // Exactly one more draw: span 0's end state advanced by one
            // `gen_bool(0.5)` is span > 0's end state, and that draw is the
            // one that decided `remembers`.
            let mut advanced = rng_off.clone();
            assert_eq!(advanced.gen_bool(0.5), b.remembers, "seed {seed}");
            assert!(advanced == rng_on, "seed {seed}: one extra draw");
        }
    }

    #[test]
    fn a_sex_childs_remembers_draw_is_one_extra_draw_after_everything_else() {
        let born = |span: u32| {
            let mut w = memory_world(10, 10, span.max(1));
            w.config.memory.span = span;
            w.config.memory.share = 0.5;
            w.config.sex.enabled = true;
            let mom = spawn(&mut w, 2, 2);
            let dad = spawn(&mut w, 3, 2);
            w.agent_mut(dad).unwrap().sex = crate::agent::Sex::Male;
            let rng_before = w.rng.clone();
            crate::rules::sex::act(&mut w, mom);
            let child = w.agents().find(|a| a.parents.is_some()).unwrap().clone();
            (child, rng_before, w.rng.clone())
        };
        let (a, before_off, rng_off) = born(0);
        let (b, before_on, rng_on) = born(20);
        assert!(before_off == before_on, "the same stream going in");
        assert!(!a.remembers, "span 0 never remembers");
        let mut b_as_off = b.clone();
        b_as_off.remembers = false;
        assert_eq!(a, b_as_off, "only `remembers` differs");
        let mut advanced = rng_off.clone();
        assert_eq!(advanced.gen_bool(0.5), b.remembers);
        assert!(advanced == rng_on, "one extra draw");
    }

    #[test]
    fn span_zero_world_matches_the_pinned_golden() {
        use crate::config::Config;
        // `ii-2-unit` is `Config::default()` (span 0). Its fingerprint was
        // pinned in `tests/golden.rs` before memory existed at all: if a
        // `remembers` draw ever slipped out from under the `span > 0`
        // guard, this world's whole random stream downstream would shift
        // and this fingerprint would no longer match.
        let mut w = World::new(Config::default(), 1).unwrap();
        w.run(200);
        assert_eq!(w.fingerprint(), 0x75b93943813545e4);
    }

    #[test]
    fn share_one_makes_every_flump_remember_and_share_zero_makes_none() {
        use crate::agent::Agent;
        let mut c = blank_config(10, 10);
        c.memory.span = 20;
        c.memory.share = 1.0;
        let a = Agent::random(&c, Pos::new(0, 0), 0, &mut seeded(1));
        assert!(a.remembers);
        c.memory.share = 0.0;
        let b = Agent::random(&c, Pos::new(0, 0), 0, &mut seeded(1));
        assert!(!b.remembers);
    }

    // --- observe and truffles ---

    #[test]
    fn observe_records_the_flumps_own_truffle_spot_ripe_then_unripe_after_a_harvest() {
        let mut c = blank_config(10, 10);
        c.movement.mode = MoveMode::Walk;
        c.memory.span = 50;
        c.memory.share = 1.0;
        c.truffles.share = 1.0;
        c.truffles.value = 5.0;
        c.truffles.regrow = 30;
        let mut w = World::new(c, 7).expect("valid config");
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().remembers = true;
        let own = w.torus.index(Pos::new(5, 5)) as u32;

        w.tick = 10;
        observe(&mut w, id);
        let seen = w.agent(id).unwrap().memory.sites[&own]
            .truffle
            .expect("the spot the Flump stands on is seen");
        assert!(seen.ripe, "not yet harvested");
        assert_eq!(seen.tick, 10);

        // Harvest it, then observe again at the same tick: it's now seen unripe.
        crate::rules::movement::go_and_gather(&mut w, id, Pos::new(5, 5));
        observe(&mut w, id);
        let seen2 = w.agent(id).unwrap().memory.sites[&own]
            .truffle
            .expect("still a spot, just picked");
        assert!(!seen2.ripe, "a Flump that just harvested sees it unripe");
        assert_eq!(seen2.tick, 10);
    }

    #[test]
    fn observe_leaves_truffle_none_when_the_own_site_has_no_spot() {
        let mut w = memory_world(10, 10, 50);
        // memory_world sets memory.share 1.0 but truffles default to share 0.
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().remembers = true;
        let own = w.torus.index(Pos::new(5, 5)) as u32;
        w.tick = 5;
        observe(&mut w, id);
        assert_eq!(w.agent(id).unwrap().memory.sites[&own].truffle, None);
    }

    #[test]
    fn a_child_born_under_memory_draws_its_own_remembers_and_starts_empty() {
        let mut w = memory_world(10, 10, 20);
        w.config.sex.enabled = true;
        w.config.memory.share = 1.0;
        let mom = spawn(&mut w, 2, 2);
        let dad = spawn(&mut w, 3, 2);
        {
            let d = w.agent_mut(dad).unwrap();
            d.sex = crate::agent::Sex::Male;
        }
        w.agent_mut(mom)
            .unwrap()
            .memory
            .sites
            .insert(1, seen_at(1.0, 1.0, 0));
        w.agent_mut(dad)
            .unwrap()
            .memory
            .sites
            .insert(2, seen_at(1.0, 1.0, 0));
        crate::rules::sex::act(&mut w, mom);
        let child = w.agents().find(|a| a.parents.is_some()).unwrap();
        assert!(
            child.memory.sites.is_empty(),
            "children don't inherit memories"
        );
        assert!(
            child.remembers,
            "the child draws its own remembers (share 1.0)"
        );
    }

    // --- the cap ---

    #[test]
    fn cap_drops_the_oldest_then_the_lowest_site_index() {
        let mut m = Memory::default();
        m.sites.insert(7, seen_at(1.0, 1.0, 5));
        m.sites.insert(3, seen_at(1.0, 1.0, 5));
        m.sites.insert(9, seen_at(1.0, 1.0, 2));
        m.sites.insert(1, seen_at(1.0, 1.0, 8));
        cap(&mut m, 4);
        assert_eq!(m.sites.len(), 4, "at the cap, nothing goes");
        cap(&mut m, 2);
        // 9 (tick 2) is oldest; 3 and 7 tie at tick 5, so 3 (lower) goes.
        assert_eq!(m.sites.keys().copied().collect::<Vec<_>>(), [1, 7]);
    }

    // --- `memory.prior: map` (Minds 4) ---

    #[test]
    fn keep_richest_keeps_the_highest_levels() {
        let entries = vec![(1.0, 7), (1.0, 3), (2.0, 9), (5.0, 1)];
        let mut idx: Vec<u32> = keep_richest(entries, 2)
            .into_iter()
            .map(|(_, i)| i)
            .collect();
        idx.sort_unstable();
        assert_eq!(idx, [1, 9], "5.0 and 2.0 beat 1.0");
    }

    #[test]
    fn keep_richest_breaks_ties_by_the_lower_site_index() {
        let entries = vec![(1.0, 7), (1.0, 3), (1.0, 9), (1.0, 1)];
        let mut idx: Vec<u32> = keep_richest(entries, 2)
            .into_iter()
            .map(|(_, i)| i)
            .collect();
        idx.sort_unstable();
        assert_eq!(idx, [1, 3], "every level ties, so the lowest indices win");
    }

    #[test]
    fn keep_richest_is_a_no_op_at_or_under_the_cap() {
        let entries = vec![(1.0, 7), (2.0, 3)];
        assert_eq!(keep_richest(entries.clone(), 2).len(), 2);
        assert_eq!(keep_richest(entries, 5).len(), 2);
    }

    #[test]
    fn map_prior_gives_every_founder_that_remembers_every_nonwall_site_at_tick_zero() {
        let mut c = blank_config(5, 5);
        c.movement.mode = MoveMode::Walk;
        c.memory.span = 20;
        c.memory.share = 1.0;
        c.memory.prior = MemoryPrior::Map;
        c.population = 3;
        let len = (c.width * c.height) as usize;
        let landscape: Vec<f64> = (0..len).map(|i| i as f64).collect();
        let w = World::with_landscapes(c, 7, &[Some(landscape.clone())]).expect("valid config");
        assert_eq!(w.population(), 3);
        for agent in w.agents() {
            assert!(agent.remembers, "share 1.0");
            assert_eq!(agent.memory.sites.len(), len, "every non-wall site");
            for (i, &level) in landscape.iter().enumerate() {
                let seen = &agent.memory.sites[&(i as u32)];
                assert_eq!(seen.levels()[0], level);
                assert_eq!(seen.most()[0], level);
                assert_eq!(seen.tick, 0);
                assert_eq!(seen.truffle, None, "no truffle knowledge");
            }
        }
    }

    #[test]
    fn map_prior_none_seeds_nothing() {
        let mut c = blank_config(5, 5);
        c.movement.mode = MoveMode::Walk;
        c.memory.span = 20;
        c.memory.share = 1.0;
        c.population = 3;
        // memory.prior defaults to None.
        let w = World::new(c, 7).expect("valid config");
        for agent in w.agents() {
            assert!(agent.remembers);
            assert!(
                agent.memory.sites.is_empty(),
                "prior none leaves founders exactly as Minds 3 did"
            );
        }
    }

    #[test]
    fn map_prior_gives_non_rememberers_nothing() {
        let mut c = blank_config(5, 5);
        c.movement.mode = MoveMode::Walk;
        c.memory.span = 20;
        c.memory.share = 0.0;
        c.memory.prior = MemoryPrior::Map;
        c.population = 3;
        let w = World::new(c, 7).expect("valid config");
        for agent in w.agents() {
            assert!(!agent.remembers, "share 0.0");
            assert!(agent.memory.sites.is_empty());
        }
    }

    #[test]
    fn map_prior_respects_the_cap_and_keeps_the_richest_sites() {
        let (width, height) = (80, 80);
        let len = (width * height) as usize;
        assert!(len > MEMORY_CAP, "the world has more sites than the cap");
        let mut c = blank_config(width, height);
        c.movement.mode = MoveMode::Walk;
        c.memory.span = 20;
        c.memory.share = 1.0;
        c.memory.prior = MemoryPrior::Map;
        c.population = 1;
        let landscape: Vec<f64> = (0..len).map(|i| i as f64).collect();
        let w = World::with_landscapes(c, 7, &[Some(landscape)]).expect("valid config");
        let agent = w.agents().next().expect("one founder");
        assert_eq!(agent.memory.sites.len(), MEMORY_CAP);
        let richest_start = (len - MEMORY_CAP) as u32;
        for &idx in agent.memory.sites.keys() {
            assert!(
                idx >= richest_start,
                "kept the {MEMORY_CAP} richest sites, not site {idx}"
            );
        }
    }

    #[test]
    fn memory_never_exceeds_the_cap_on_a_big_flat_world_with_a_long_span() {
        let (width, height) = (200, 200);
        let mut c = blank_config(width, height);
        c.goods[0].map = crate::config::Map::Flat { capacity: 4.0 };
        c.movement.mode = MoveMode::Walk;
        c.memory.span = 10_000;
        c.memory.share = 1.0;
        let mut w = World::new(c, 7).expect("valid config");
        assert!(
            (width * height) as usize > MEMORY_CAP,
            "the world has more sites than the cap"
        );
        let id = spawn(&mut w, 0, 0);
        w.agent_mut(id).unwrap().remembers = true;
        w.agent_mut(id).unwrap().vision = 10;
        let mut reached = false;
        // Hop 21 columns a tick (a fresh stripe of 41 sites each time) along
        // rows 21 apart, so memory keeps meeting sites it hasn't seen.
        for t in 1..=400u64 {
            let x = ((t * 21) % u64::from(width)) as u32;
            let y = ((t * 21 / u64::from(width)) * 21 % u64::from(height)) as u32;
            w.move_agent(id, Pos::new(x, y));
            w.tick = t;
            observe(&mut w, id);
            let m = &w.agent(id).unwrap().memory;
            assert!(m.sites.len() <= MEMORY_CAP, "tick {t}: {}", m.sites.len());
            reached |= m.sites.len() == MEMORY_CAP;
            // What was just seen is always kept: it's the newest.
            let own = w.torus.index(Pos::new(x, y)) as u32;
            assert_eq!(m.sites[&own].tick, t);
        }
        assert!(reached, "the walk fills memory to the cap");
    }
}
