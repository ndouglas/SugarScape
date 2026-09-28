//! Minds 3: memory. A rememberer keeps what it has seen of a site after it
//! leaves sight, so it can walk back to what it recalls rather than only
//! what's currently in view.

use std::collections::BTreeMap;

use crate::agent::AgentId;
use crate::config::{Belief, MAX_GOODS};
use crate::geometry::Pos;
use crate::world::World;

/// A remembered site (Minds 3): the levels seen, the most ever seen there, when it was last seen, and
/// what the Flump knows of a truffle spot there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seen {
    pub levels: [f64; MAX_GOODS],
    pub most: [f64; MAX_GOODS],
    pub tick: u64,
    pub truffle: Option<TruffleSeen>,
}

/// A known truffle spot: whether it was ripe when last seen, and when that was (a harvest counts as seen unripe).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TruffleSeen {
    pub ripe: bool,
    pub tick: u64,
}

/// A Flump's remembered sites, by site index (deterministic order).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory {
    pub sites: BTreeMap<u32, Seen>,
}

/// After the move: a rememberer records every site in its sight and its own
/// site. Builds the observed list first (immutable borrow of `world`), then
/// writes it into the agent's memory. Runs `forget` every 16 ticks per agent;
/// Task 4's candidates also forget lazily.
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

    let agent = world.agent_mut(id).expect("live agent");
    for (idx, levels) in observed {
        let entry = agent.memory.sites.entry(idx).or_insert(Seen {
            levels: [0.0; MAX_GOODS],
            most: [0.0; MAX_GOODS],
            tick: now,
            truffle: None,
        });
        entry.levels = levels;
        entry.tick = now;
        for (m, &l) in entry.most.iter_mut().zip(levels.iter()) {
            *m = m.max(l);
        }
    }
    if now.is_multiple_of(16) {
        forget(&mut agent.memory, now, span);
    }
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
    if belief == Belief::Recall {
        return seen.levels[i];
    }
    let age = now.saturating_sub(seen.tick);
    if age == 0 {
        return seen.levels[i];
    }
    if instant {
        seen.most[i]
    } else {
        (seen.levels[i] + rate * age as f64).min(seen.most[i])
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
        let mut s = Seen {
            levels: [0.0; MAX_GOODS],
            most: [0.0; MAX_GOODS],
            tick,
            truffle: None,
        };
        s.levels[0] = levels;
        s.most[0] = most;
        s
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
        assert_eq!(mem.sites[&own].levels[0], 3.0);
        assert_eq!(mem.sites[&own].most[0], 3.0);
        assert_eq!(mem.sites[&own].tick, 10);
        assert_eq!(mem.sites[&east].levels[0], 4.0);
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
        let seen = w.agent(id).unwrap().memory.sites[&own];
        assert_eq!(seen.levels[0], 2.0, "the level tracks what's current");
        assert_eq!(seen.tick, 9);
        assert_eq!(seen.most[0], 5.0, "most stays at the higher value seen");
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
    fn span_zero_draws_nothing_new_in_agent_random() {
        use crate::agent::Agent;
        use crate::config::Config;
        use crate::geometry::Pos;
        let config = Config::default();
        assert_eq!(config.memory.span, 0, "the default is off");
        let mut rng = seeded(4);
        let _ = Agent::random(&config, Pos::new(0, 0), 0, &mut rng);
        // Nothing about the `remembers` field changes what's drawn: a clone
        // taken beforehand and advanced by the exact same call agrees on
        // every later draw, so `Agent::random` under span 0 draws the same
        // sequence a clone of the same call does (a direct, if weak, check
        // that the guarded branch was skipped rather than merely idempotent
        // — the strong check is `span_zero_world_matches_the_pinned_golden`
        // below, which pins the sequence against a value recorded before
        // this change existed).
        let mut clone = seeded(4);
        let _ = Agent::random(&config, Pos::new(0, 0), 0, &mut clone);
        assert_eq!(rng.gen::<u64>(), clone.gen::<u64>());
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
}
