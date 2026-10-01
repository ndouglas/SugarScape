//! Agent movement rule M (Chapter II): multicommodity M over n goods, with
//! the pollution-modified welfare s / (1 + p) when pollution is on.

use rand::seq::SliceRandom;

use crate::agent::{Agent, AgentId, Plan};
use crate::config::{Config, MoveMode, MAX_GOODS};
use crate::geometry::{Pos, Torus};
use crate::landscape::Site;
use crate::minds::astar::astar;
use crate::minds::grid::TorusGrid;
use crate::minds::memory::{believed_level, believed_ripe};
use crate::rng::SimRng;
use crate::rules::Harvest;
use crate::social::Seen;
use crate::world::World;

/// Most sites A* may expand for a walking agent; past it the target counts
/// as unreachable (Minds 2).
pub const WALK_LIMIT: usize = 4096;

/// Picks among `(site, distance, value)` candidates: highest value, then
/// nearest, then uniformly at random.
pub(crate) fn choose(candidates: &[(Pos, u32, f64)], rng: &mut SimRng) -> Pos {
    let best_value = candidates
        .iter()
        .map(|c| c.2)
        .fold(f64::NEG_INFINITY, f64::max);
    let nearest = candidates
        .iter()
        .filter(|c| c.2 == best_value)
        .map(|c| c.1)
        .min()
        .expect("at least one candidate");
    let ties: Vec<Pos> = candidates
        .iter()
        .filter(|c| c.2 == best_value && c.1 == nearest)
        .map(|c| c.0)
        .collect();
    *ties.choose(rng).expect("non-empty ties")
}

/// Σ pₖ, in pollutant order from 0.0, over the pollutants that devalue
/// `good`; `None` when pollution is off or none does (the good then counts
/// undiscounted).
pub(crate) fn devaluation(config: &Config, site: &Site, good: usize) -> Option<f64> {
    if !config.pollution.enabled {
        return None;
    }
    let mut total = None;
    for (k, p) in config.pollution.pollutants.iter().enumerate() {
        if p.devalues[good] {
            total = Some(total.unwrap_or(0.0) + site.pollution[k]);
        }
    }
    total
}

/// Rule M: look along the four lattice directions as far as vision permits,
/// go to the nearest unoccupied site of maximum welfare and collect its sugar
/// (every good, with n ≥ 2). The agent's current site competes at distance 0,
/// so it stays put when nothing visible is better. Returns the harvest.
/// A rememberer (Minds 3) also weighs the sites it remembers out of sight.
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let (candidates, start) = candidates_with_memory(world, id);
    let target = choose(&candidates, &mut world.rng);
    record_choice(world, id, &candidates, start, target);
    arrive(world, id, target)
}

/// Rule M's welfare for one agent, computed once per decision so each
/// candidate only supplies its levels.
pub(crate) struct Welfare {
    n: usize,
    phi: u32,
    held: [f64; MAX_GOODS],
    mets: [f64; MAX_GOODS],
}

impl Welfare {
    pub(crate) fn new(world: &World, agent: &Agent) -> Self {
        let n = world.config.goods.len();
        let (phi, held) = (agent.foresight, agent.holdings);
        let mets = if n >= 2 {
            agent.effective_metabolisms(n, world.config.disease.active_fee())
        } else {
            [0.0; MAX_GOODS]
        };
        Self { n, phi, held, mets }
    }

    /// Rule M's welfare of gathering `levels` (as counted: any pollution
    /// discount already applied). One good: the level itself. n ≥ 2 goods:
    /// foresight welfare of the holdings after gathering.
    pub(crate) fn of(&self, levels: &[f64]) -> f64 {
        let n = self.n;
        if n >= 2 {
            let after: [f64; MAX_GOODS] = std::array::from_fn(|i| {
                if i >= n {
                    0.0
                } else {
                    self.held[i] + levels[i]
                }
            });
            crate::econ::foresight_welfare_n(&after[..n], &self.mets[..n], self.phi)
        } else {
            levels[0]
        }
    }
}

/// Rule M's welfare for `agent` of gathering `levels` (already counted).
pub(crate) fn welfare_of(world: &World, agent: &Agent, levels: &[f64]) -> f64 {
    Welfare::new(world, agent).of(levels)
}

/// The levels of `site` as rule M counts them: each good discounted by the
/// pollutants that devalue it (one good: s / (1 + Σ pₖ); n ≥ 2 goods:
/// s · 1/(1 + Σ pₖ), the arithmetic each path has always used).
fn counted_levels(config: &Config, site: &Site) -> [f64; MAX_GOODS] {
    let n = config.goods.len();
    std::array::from_fn(|i| {
        if i >= n {
            return 0.0;
        }
        match devaluation(config, site, i) {
            Some(d) if n >= 2 => site.resource[i] * (1.0 / (1.0 + d)),
            Some(d) => site.resource[i] / (1.0 + d),
            None => site.resource[i],
        }
    })
}

/// Rule M's candidates for `id`: its current site at distance 0, then the
/// unoccupied sites in sight in `torus.sight` order, each with rule M's
/// welfare. One good: sugar, discounted by 1/(1 + Σ pₖ) under pollution.
/// n ≥ 2 goods: foresight welfare after gathering, each good discounted by
/// the pollutants that devalue it. A rememberer's remembered sites follow
/// (`candidates_with_memory`, which rule M and the utility mind both call;
/// this list alone is for tests).
#[cfg(test)]
pub(crate) fn candidates(world: &World, id: AgentId) -> Vec<(Pos, u32, f64)> {
    candidates_with_memory(world, id).0
}

/// `candidates`, extended for a rememberer (Minds 3), and the index where
/// its remembered entries start (the list's length when there are none).
///
/// - A site in sight (or its own) carries its true welfare, with
///   `truffles.value` sugar added to good 0 when the agent knows a spot
///   there (a memory with `truffle: Some`) that it believes ripe.
/// - After the sight list come the remembered sites out of sight, in site
///   index order: not its own site, not in sight, not a wall, and not
///   forgotten (last seen at most `span` ticks ago). Each carries the
///   welfare of its believed levels plus any believed-ripe truffle, at its
///   torus Manhattan distance. True occupancy doesn't filter them (the agent
///   can't see who's there), and they carry no pollution discount (memory
///   keeps no pollution: the agent doesn't see it out of sight).
///
/// A non-rememberer's list is rule M's exactly, value for value.
///
/// Minds 5: a hungry agent's own caches join after the sight list, before
/// the remembered entries (`caching::join_caches`: a cache already listed is
/// merged by the larger value and moved there if it was remembered); the
/// list is untouched for an agent without caches.
pub(crate) fn candidates_with_memory(world: &World, id: AgentId) -> (Vec<(Pos, u32, f64)>, usize) {
    let a = world.agent(id).expect("live agent");
    let (pos, vision) = (a.pos, a.vision);
    let welfare = Welfare::new(world, a);
    let config = &world.config;
    let now = world.tick;
    let mem = &config.memory;
    let known = a.remembers && mem.span > 0;
    let span = u64::from(mem.span);
    let truffle_value = config.truffles.value;
    let regrow = config.truffles.regrow;
    // Whether the agent knows a spot at site index `i` that it believes ripe
    // (and hasn't forgotten).
    let believes_ripe = |i: u32| -> bool {
        a.memory.sites.get(&i).is_some_and(|seen| {
            now.saturating_sub(seen.tick) <= span
                && seen
                    .truffle
                    .is_some_and(|t| believed_ripe(&t, now, regrow, mem.belief))
        })
    };
    let value = |p: Pos| {
        let mut levels = counted_levels(config, world.site(p));
        if known && believes_ripe(world.torus.index(p) as u32) {
            levels[0] += truffle_value;
        }
        welfare.of(&levels)
    };
    let sight = world.sight(pos, vision);
    let mut out = vec![(pos, 0, value(pos))];
    for &(q, d) in &sight {
        if !world.is_occupied(q) {
            out.push((q, d, value(q)));
        }
    }
    let mut start = out.len();
    if !known {
        crate::minds::caching::join_caches(world, id, &mut out, &mut start);
        crate::minds::caching::watching::join_seen(world, id, &mut out, &mut start);
        return (out, start);
    }
    let torus = world.torus;
    let own = torus.index(pos) as u32;
    let mut in_sight: Vec<u32> = sight.iter().map(|&(q, _)| torus.index(q) as u32).collect();
    in_sight.sort_unstable();
    let instant = config.growback.instant;
    let n = config.goods.len();
    for (&i, seen) in &a.memory.sites {
        let q = torus.pos(i as usize);
        if i == own
            || now.saturating_sub(seen.tick) > span
            || in_sight.binary_search(&i).is_ok()
            || world.is_wall(q)
        {
            continue;
        }
        let rate = crate::rules::growback::rate_at(config, now, q.y);
        let mut levels = [0.0; MAX_GOODS];
        for (g, level) in levels.iter_mut().enumerate().take(n) {
            *level = believed_level(seen, g, now, rate, instant, mem.belief);
        }
        if seen
            .truffle
            .is_some_and(|t| believed_ripe(&t, now, regrow, mem.belief))
        {
            levels[0] += truffle_value;
        }
        out.push((q, lattice_distance(torus, pos, q), welfare.of(&levels)));
    }
    crate::minds::caching::join_caches(world, id, &mut out, &mut start);
    crate::minds::caching::watching::join_seen(world, id, &mut out, &mut start);
    (out, start)
}

/// Steps between `a` and `b` along the 4-way torus (Manhattan, wrapping).
pub(crate) fn lattice_distance(torus: Torus, a: Pos, b: Pos) -> u32 {
    let dx = a.x.abs_diff(b.x);
    let dy = a.y.abs_diff(b.y);
    dx.min(torus.width - dx) + dy.min(torus.height - dy)
}

/// Rule M's welfare of `p` for `id` as it truly is now (Minds 3's
/// diagnostics): its counted levels, plus `truffles.value` sugar if it has a
/// spot that's ripe now, known or not.
fn true_value(world: &World, id: AgentId, p: Pos) -> f64 {
    let value = site_value(world, id, p);
    // Minds 5: a hungry agent's own cache counts as its candidate did.
    // Minds 8: so do the caches it saw buried there, at what is truly
    // left of them now (not what it believes), so an emptied one is stale.
    let value = match crate::minds::caching::cache_value(world, id, p) {
        Some(cache) => value.max(cache),
        None => value,
    };
    match crate::minds::caching::watching::seen_truth(world, id, p) {
        Some(seen) => value.max(seen),
        None => value,
    }
}

/// `true_value` without any cache: rule M's welfare of `p`'s counted levels
/// (pollution-discounted), plus `truffles.value` sugar if its spot is ripe.
fn site_value(world: &World, id: AgentId, p: Pos) -> f64 {
    let a = world.agent(id).expect("live agent");
    let mut levels = counted_levels(&world.config, world.site(p));
    if world.truffle(p) == Some(true) {
        levels[0] += world.config.truffles.value;
    }
    welfare_of(world, a, &levels)
}

/// Minds 3's diagnostics for a rememberer's choice of `target` among
/// `candidates` (unscored welfare; remembered entries from `start`): every
/// such choice counts a move, and a remembered target also counts a
/// remembered move and adds |believed − true| to the belief error, as
/// believed against true value now, at choosing, not on arrival; a target
/// truly worth less than believed is a stale choice. Draws nothing.
pub(crate) fn record_choice(
    world: &mut World,
    id: AgentId,
    candidates: &[(Pos, u32, f64)],
    start: usize,
    target: Pos,
) {
    if !world.agent(id).expect("live agent").remembers || world.config.memory.span == 0 {
        return;
    }
    world.events.moves += 1;
    let Some(believed) = candidates[start..]
        .iter()
        .find(|c| c.0 == target)
        .map(|c| c.2)
    else {
        return;
    };
    let truth = true_value(world, id, target);
    let e = &mut world.events;
    e.remembered_moves += 1;
    e.belief_error_sum += (believed - truth).abs();
    if truth < believed {
        e.stale_choices += 1;
    }
}

/// Moves `id` to `target` (its own site to stay), records its neighbors, and
/// gathers every good there, plus a truffle spot's value (Minds 3) if it's
/// ripe: into good 0's harvest and the agent's holdings, like any other
/// gathered sugar, and the spot goes unripe until `tick + regrow`.
///
/// Minds 5:
/// - **Carrying limit.** With `caching.capacity` C > 0, good 0 gathered is
///   min(level, C − holdings) (≥ 0) and the rest stays on the site. In a
///   central-place world the limit caps the trip's load instead: C −
///   `load_trip` (provisions in hand don't count against it). A ripe
///   truffle's value is capped by what room is left; the spot is picked all
///   the same, so the unpicked excess is lost with its ripeness (as a
///   truffle's sugar was never on the site).
/// - **Dig.** At a site holding its own cache, while it's hungry (holdings
///   below half the reserve; the reserve itself in a central-place world,
///   `caching::hungry`), the agent takes the larger of the two: it digs when the cache
///   is at least the site's welfare (its level discounted by pollution, as
///   the candidate list counts it, plus a ripe truffle's value), and
///   otherwise harvests the site as usual.
///   So a candidate valued at max(site, cache) delivers that on arrival. A
///   dig takes min(cache, room) (room unlimited with C = 0) into its
///   holdings and leaves the site and any truffle as they are. It is
///   `Harvest::dug`, not `gathered`: it was gathered once already, so it
///   forms no pollution. In a central-place world a dig takes no more than
///   the reserve less holdings (`minds::central`).
///
/// Minds 6 (`minds::caching::theft`), under `theft.find` > 0:
/// - **Stumbling on caches.** An agent that doesn't dig here draws once per
///   cache on the site it doesn't know about, in owner-id order, and takes
///   the first found (min(cache, room)) instead of harvesting the site: a
///   pilfer (`Harvest::pilfered`, kept or eaten by `theft.loot`), or under
///   `owner_memory: off` its own cache found, which is a dig.
/// - **The dig wins.** If its own cache here would be dug, it digs and draws
///   nothing. Under `owner_memory: off` it never digs this way (it doesn't
///   know where its caches are).
///
/// Minds 8 (`minds::caching::watching`), under `watching.on`:
/// - **Raids.** An agent that doesn't dig here and remembers caches it saw
///   buried here takes from the first still there, in owner-id order, with
///   no draw, as a pilfer; only if it took nothing does it go on to
///   stumbling. Either way it forgets what it saw buried here (a dig too).
///   Under `raid_when: hungry` a fed agent (at or above R / 2) doesn't
///   raid, and keeps what it saw here. Under the survey probe
///   `World::probe_raid_harvests` a raid that took something also harvests
///   the site as below, and draws no stumble.
pub(crate) fn go_and_gather(world: &mut World, id: AgentId, target: Pos) -> Harvest {
    let n = world.config.goods.len();
    let a = world.agent(id).expect("live agent");
    let (tags, mut social, remembers) = (a.tags, a.social, a.remembers);
    let capacity = world.config.caching.capacity;
    let site_index = world.torus.index(target) as u32;
    // Cache against the site's welfare (pollution-discounted, as the
    // candidate list valued it), not its raw level.
    let digs = world.config.theft.owner_memory
        && a.caches.get(&site_index).is_some_and(|&cache| {
            cache >= site_value(world, id, target) && crate::minds::caching::hungry(world, id)
        });
    // Room under the carrying limit; infinite with no limit.
    let room = |held: f64| {
        if capacity > 0 {
            (f64::from(capacity) - held).max(0.0)
        } else {
            f64::INFINITY
        }
    };
    let held = a.holdings[0];
    // What counts against the limit: holdings, or in a central-place world
    // the trip's load alone (the limit caps a load, not load plus
    // provisions).
    let central = world.config.central.enabled;
    let mut used = if central { a.load_trip } else { held };
    world.move_agent(id, target);
    social.moved(world, Seen::at(world, target), tags);
    let mut harvest = Harvest::default();
    if digs {
        // A central-place forager digs only up to its reserve (one tick's
        // need), so what it digs is eaten, not delivered again.
        let room = if central {
            let short = crate::minds::caching::reserve(world, id) - held;
            room(used).min(short.max(0.0))
        } else {
            room(used)
        };
        harvest.dug = crate::minds::caching::dig(world, id, site_index, room);
        // Minds 8: an arrival forgets what it saw buried here, dig or not.
        if world.config.watching.on {
            crate::minds::caching::watching::forget(world, id, site_index);
        }
        let a = world.agent_mut(id).expect("live agent");
        a.holdings[0] += harvest.dug;
        a.social = social;
        return harvest;
    }
    if world.config.watching.on {
        if let Some(taken) =
            crate::minds::caching::watching::raid(world, id, site_index, room(used))
        {
            if !world.probe_raid_harvests {
                world.agent_mut(id).expect("live agent").social = social;
                return taken;
            }
            // The survey probe: the raid doesn't replace the harvest. Kept
            // loot now counts against the limit (watching never runs in a
            // central-place world, so that is the holdings).
            harvest.pilfered = taken.pilfered;
            used = world.agent(id).expect("live agent").holdings[0];
        }
    }
    if world.config.theft.find > 0.0 && harvest.pilfered == 0.0 {
        if let Some(taken) =
            crate::minds::caching::theft::stumble(world, id, site_index, room(used))
        {
            world.agent_mut(id).expect("live agent").social = social;
            return taken;
        }
    }
    let site = world.site_mut(target);
    for (got, level) in harvest
        .gathered
        .iter_mut()
        .zip(site.resource.iter_mut())
        .take(n)
    {
        *got = *level;
        *level = 0.0;
    }
    if capacity > 0 {
        // Caching runs with exactly one good (validated): only good 0 is
        // limited. What doesn't fit goes back on the site.
        let take = harvest.gathered[0].min(room(used));
        site.resource[0] = harvest.gathered[0] - take;
        harvest.gathered[0] = take;
    }
    let now = world.tick;
    let regrow = u64::from(world.config.truffles.regrow);
    let value = world.config.truffles.value;
    let mut picked = false;
    if let Some(ripe_at) = world.truffle_ripe_at(target) {
        if *ripe_at <= now {
            *ripe_at = now + regrow;
            picked = true;
        }
    }
    if picked {
        harvest.gathered[0] += if capacity > 0 {
            value.min(room(used + harvest.gathered[0]))
        } else {
            value
        };
        world.events.truffles_found += 1;
        if remembers {
            world.events.truffles_by_rememberers += 1;
        }
    }
    let a = world.agent_mut(id).expect("live agent");
    for (have, got) in a.holdings.iter_mut().zip(&harvest.gathered).take(n) {
        *have += got;
    }
    a.social = social;
    harvest
}

/// Reaches `target` by the configured movement, then gathers where the
/// agent stops. `jump` (rule M) goes there in one tick. `walk` takes `speed`
/// steps along an A* path on the 4-way torus (walls and occupied sites
/// impassable, except the target) and stays when there is none within
/// `WALK_LIMIT`. A remembered target (Minds 3) may be occupied: the walker
/// then stops one site short of it on the path, or stays when that's where
/// it stands. Records the agent's plan; draws nothing. Minds 8: a walker
/// that stays for want of a path forgets the caches it saw buried at the
/// target (`watching::give_up`, under `watching.on`).
pub(crate) fn arrive(world: &mut World, id: AgentId, target: Pos) -> Harvest {
    let pos = world.agent(id).expect("live agent").pos;
    let m = world.config.movement;
    if m.mode == MoveMode::Jump || target == pos {
        world.agent_mut(id).expect("live agent").plan = Plan {
            target: Some(target),
            path: Vec::new(),
            walked: m.mode == MoveMode::Walk,
        };
        return go_and_gather(world, id, target);
    }
    let torus = world.torus;
    // Walls split the non-wall sites into components labeled at build. A
    // target in another component can't be reached, so skip the search:
    // this only short-circuits searches A* would fail anyway, and the
    // walker stays exactly as on a `None` from A*.
    let found = if world.walled_apart(pos, target) {
        None
    } else {
        let grid = TorusGrid::new(torus, |q| q == target || !world.is_occupied(q));
        astar(&grid, torus.index(pos), torus.index(target), WALK_LIMIT)
    };
    let (stop, rest) = match found {
        Some(s) => {
            let mut steps = (m.speed as usize).min(s.path.len() - 1);
            // Only a remembered target can be occupied (every other site on
            // the path is passable only while free): stop on the site before
            // it, or stay when that's where the walker already is.
            if world
                .occupant(torus.pos(s.path[steps]))
                .is_some_and(|o| o != id)
            {
                steps -= 1;
            }
            let rest = s.path[steps + 1..].iter().map(|&i| torus.pos(i)).collect();
            (torus.pos(s.path[steps]), rest)
        }
        None => {
            // Minds 8: a seen cache it can't reach is given up.
            if world.config.watching.on {
                let site = torus.index(target) as u32;
                crate::minds::caching::watching::give_up(world, id, site);
            }
            (pos, Vec::new())
        }
    };
    world.agent_mut(id).expect("live agent").plan = Plan {
        target: Some(target),
        path: rest,
        walked: true,
    };
    go_and_gather(world, id, stop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Pollutant, Pollution};
    use crate::testkit::*;

    fn mover(w: &mut World, vision: u32) -> AgentId {
        let id = spawn(w, 5, 5);
        w.agent_mut(id).unwrap().vision = vision;
        id
    }

    fn walker(w: &mut World, vision: u32, speed: u32) -> AgentId {
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed,
        };
        mover(w, vision)
    }

    #[test]
    fn a_walker_takes_one_step_toward_the_target_and_gathers_only_there() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        set_sugar(&mut w, 5, 8, 3.0);
        set_sugar(&mut w, 5, 6, 1.0); // on the way: stepped onto, so gathered
        let h = act(&mut w, id);
        // Rule M picks (5, 8) (the most sugar); the walker steps to (5, 6).
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6));
        assert_eq!(h.gathered[0], 1.0);
        assert_eq!(w.site(Pos::new(5, 8)).resource[0], 3.0, "not reached yet");
        let plan = &w.agent(id).unwrap().plan;
        assert_eq!(plan.target, Some(Pos::new(5, 8)));
        assert_eq!(plan.path, vec![Pos::new(5, 7), Pos::new(5, 8)]);
    }

    #[test]
    fn a_walker_goes_around_an_occupant() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        spawn(&mut w, 5, 6);
        set_sugar(&mut w, 5, 7, 3.0);
        act(&mut w, id);
        let p = w.agent(id).unwrap().pos;
        assert!(
            p == Pos::new(4, 5) || p == Pos::new(6, 5),
            "a sidestep, got {p:?}"
        );
    }

    #[test]
    fn speed_three_stops_at_the_target() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 3);
        set_sugar(&mut w, 5, 7, 3.0);
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 7), 3.0)
        );
        assert!(w.agent(id).unwrap().plan.path.is_empty());
    }

    #[test]
    fn a_walker_with_no_path_stays_and_gathers_where_it_is() {
        let mut w = blank_world(11, 11);
        let id = walker(&mut w, 3, 1);
        set_sugar(&mut w, 5, 5, 0.5);
        set_sugar(&mut w, 5, 8, 3.0);
        for (x, y) in [(5, 4), (5, 6), (4, 5), (6, 5)] {
            spawn(&mut w, x, y); // boxed in
        }
        let before = w.rng.clone();
        let candidates_before = candidates(&w, id);
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 5), 0.5)
        );
        // No draws beyond `choose`: `w.rng` after `act` matches a clone
        // advanced by exactly one `choose` over the pre-`act` candidates.
        let mut expected = before;
        let _ = choose(&candidates_before, &mut expected);
        assert_eq!(w.rng, expected);
    }

    #[test]
    fn plan_is_not_hashed() {
        let mut w = blank_world(5, 5);
        let id = spawn(&mut w, 1, 1);
        let before = w.fingerprint();
        w.agent_mut(id).unwrap().plan = crate::agent::Plan {
            target: Some(Pos::new(2, 2)),
            path: vec![Pos::new(1, 2), Pos::new(2, 2)],
            walked: true,
        };
        assert_eq!(w.fingerprint(), before, "plan is observational, not hashed");
    }

    fn walled_world(walls: Vec<crate::config::Wall>) -> World {
        let mut c = blank_config(11, 11);
        c.walls = walls;
        World::new(c, 7).unwrap()
    }

    fn wall(x: u32, y: u32, width: u32, height: u32, opaque: bool) -> crate::config::Wall {
        crate::config::Wall {
            x,
            y,
            width,
            height,
            opaque,
        }
    }

    #[test]
    fn a_walker_boxed_in_by_walls_stays_and_gathers_where_it_is_without_panicking() {
        // Fences (not opaque), not occupants, on all four neighbors: they
        // block movement but not sight, so sugar beyond one is visible and
        // chosen as the target, yet the walker can't reach it.
        let mut w = walled_world(vec![
            wall(5, 4, 1, 1, false),
            wall(5, 6, 1, 1, false),
            wall(4, 5, 1, 1, false),
            wall(6, 5, 1, 1, false),
        ]);
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().vision = 3;
        set_sugar(&mut w, 5, 5, 0.5);
        set_sugar(&mut w, 5, 2, 3.0); // visible past the fence at (5, 4)
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 5), 0.5)
        );
    }

    #[test]
    fn a_walker_stays_put_when_the_target_is_a_pocket_sealed_by_fences() {
        // A ring of fences around (5, 5) leaves it visible (fences don't
        // stop sight) but unreachable (they do block movement) from any
        // side.
        let mut w = walled_world(vec![
            wall(4, 4, 3, 1, false),
            wall(4, 6, 3, 1, false),
            wall(4, 5, 1, 1, false),
            wall(6, 5, 1, 1, false),
        ]);
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        let id = spawn(&mut w, 5, 1);
        w.agent_mut(id).unwrap().vision = 4;
        set_sugar(&mut w, 5, 1, 0.2);
        set_sugar(&mut w, 5, 5, 5.0); // in the sealed pocket
        let h = act(&mut w, id);
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 1), 0.2)
        );
    }

    #[test]
    fn the_wall_component_precheck_changes_nothing_but_the_search() {
        // The sealed pocket again: the precheck skips A*, and the walker
        // stays and gathers exactly as when A* runs and finds no path.
        let mut w = walled_world(vec![
            wall(4, 4, 3, 1, false),
            wall(4, 6, 3, 1, false),
            wall(4, 5, 1, 1, false),
            wall(6, 5, 1, 1, false),
        ]);
        w.config.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        let id = spawn(&mut w, 5, 1);
        w.agent_mut(id).unwrap().vision = 4;
        set_sugar(&mut w, 5, 1, 0.2);
        set_sugar(&mut w, 5, 5, 5.0);
        assert!(w.walled_apart(Pos::new(5, 1), Pos::new(5, 5)));
        assert!(!w.walled_apart(Pos::new(5, 1), Pos::new(9, 9)));
        let mut searched = w.clone();
        searched.regions.clear(); // no precheck: A* runs and fails
        let (h, h_searched) = (act(&mut w, id), act(&mut searched, id));
        assert_eq!(
            (w.agent(id).unwrap().pos, h.gathered[0]),
            (Pos::new(5, 1), 0.2)
        );
        assert_eq!(h.gathered, h_searched.gathered);
        assert_eq!(w.agent(id).unwrap().plan, searched.agent(id).unwrap().plan);
        assert_eq!(w.rng, searched.rng);
        assert_eq!(w.fingerprint(), searched.fingerprint());
    }

    #[test]
    fn without_walls_no_components_are_labeled() {
        let w = blank_world(11, 11);
        assert!(w.regions.is_empty());
        assert!(!w.walled_apart(Pos::new(0, 0), Pos::new(5, 5)));
    }

    fn spicy(w: &mut World, vision: u32) -> AgentId {
        add_goods(&mut w.config, 2);
        let id = mover(w, vision);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.metabolism[1] = 1;
        id
    }

    #[test]
    fn moves_to_the_richest_visible_site_and_gathers_it() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 5, 8, 3.0);
        set_sugar(&mut w, 7, 5, 2.0);
        let gathered = act(&mut w, id);
        assert_eq!(gathered.gathered[0], 3.0);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 8));
        assert_eq!(w.agent(id).unwrap().holdings[0], 13.0);
        assert_eq!(w.site(Pos::new(5, 8)).resource[0], 0.0);
        assert_eq!(w.occupant(Pos::new(5, 5)), None);
    }

    #[test]
    fn prefers_the_nearest_of_equal_sites() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 5, 2, 2.0); // distance 3
        set_sugar(&mut w, 7, 5, 2.0); // distance 2
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(7, 5));
    }

    #[test]
    fn cannot_see_diagonally() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 6, 6, 4.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
    }

    #[test]
    fn skips_occupied_sites() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        spawn(&mut w, 5, 7);
        set_sugar(&mut w, 5, 7, 4.0);
        set_sugar(&mut w, 3, 5, 1.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(3, 5));
    }

    #[test]
    fn sees_across_the_wraparound_edge() {
        let mut w = blank_world(11, 11);
        let id = spawn(&mut w, 0, 5);
        set_sugar(&mut w, 10, 5, 1.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(10, 5));
    }

    #[test]
    fn pollution_devalues_sites_when_enabled() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 3);
        set_sugar(&mut w, 6, 5, 4.0);
        w.site_mut(Pos::new(6, 5)).pollution[0] = 3.0; // welfare 1
        set_sugar(&mut w, 5, 7, 2.0); // welfare 2
        w.config.pollution.enabled = true;
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7));
    }

    #[test]
    fn choose_breaks_ties_by_distance_then_randomly() {
        let mut rng = crate::rng::seeded(1);
        let a = (Pos::new(0, 0), 2, 3.0);
        let b = (Pos::new(1, 0), 1, 3.0);
        let c = (Pos::new(2, 0), 1, 3.0);
        let mut picks = std::collections::BTreeSet::new();
        for _ in 0..50 {
            picks.insert(choose(&[a, b, c], &mut rng));
        }
        assert_eq!(picks.into_iter().collect::<Vec<_>>(), vec![b.0, c.0]);
    }

    #[test]
    fn with_spice_agents_seek_the_good_they_lack() {
        let mut w = blank_world(11, 11);
        let id = spicy(&mut w, 3);
        w.agent_mut(id).unwrap().holdings[0] = 30.0;
        w.agent_mut(id).unwrap().holdings[1] = 2.0;
        set_sugar(&mut w, 5, 7, 4.0);
        w.site_mut(Pos::new(7, 5)).resource[1] = 2.0;
        let h = act(&mut w, id);
        assert_eq!(
            w.agent(id).unwrap().pos,
            Pos::new(7, 5),
            "spice-poor agent picks spice"
        );
        assert_eq!(h, crate::rules::Harvest::of(&[0.0, 2.0]));
        assert_eq!(w.agent(id).unwrap().holdings[1], 4.0);
    }

    #[test]
    fn with_spice_both_goods_are_gathered() {
        let mut w = blank_world(11, 11);
        let id = spicy(&mut w, 1);
        set_sugar(&mut w, 5, 6, 2.0);
        w.site_mut(Pos::new(5, 6)).resource[1] = 3.0;
        let h = act(&mut w, id);
        assert_eq!((h.gathered[0], h.gathered[1]), (2.0, 3.0));
        assert_eq!(w.site(Pos::new(5, 6)).resource[1], 0.0);
    }

    #[test]
    fn disease_fees_shift_the_two_good_welfare_weights() {
        // Holding 50 sugar and 5 spice with metabolisms (9, 1), welfare weights
        // are 0.9/0.1 and 5 more sugar beats 5 more spice. Four diseases at a
        // fee of 2 make the metabolisms (17, 9): weights 17/26 and 9/26, and
        // the spice site wins.
        let target = |sick: bool| {
            let mut w = blank_world(11, 11);
            let id = spicy(&mut w, 1);
            {
                let a = w.agent_mut(id).unwrap();
                (
                    a.holdings[0],
                    a.holdings[1],
                    a.metabolism[0],
                    a.metabolism[1],
                ) = (50.0, 5.0, 9, 1);
                if sick {
                    a.diseases = vec![0, 1, 2, 3];
                }
            }
            w.config.disease.enabled = true;
            w.config.disease.fee = 2.0;
            set_sugar(&mut w, 5, 6, 5.0);
            w.site_mut(Pos::new(6, 5)).resource[1] = 5.0;
            act(&mut w, id);
            w.agent(id).unwrap().pos
        };
        assert_eq!(target(false), Pos::new(5, 6));
        assert_eq!(target(true), Pos::new(6, 5));
    }

    #[test]
    fn with_three_goods_agents_seek_the_scarcest_good() {
        let mut w = blank_world(11, 11);
        add_goods(&mut w.config, 3);
        let id = mover(&mut w, 3);
        {
            let a = w.agent_mut(id).unwrap();
            a.metabolism[..3].copy_from_slice(&[1, 1, 1]);
            a.holdings[..3].copy_from_slice(&[30.0, 30.0, 2.0]);
        }
        set_resource(&mut w, 5, 7, 0, 4.0);
        set_resource(&mut w, 7, 5, 1, 4.0);
        set_resource(&mut w, 5, 3, 2, 2.0);
        let h = act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 3));
        assert_eq!(h.gathered[..3], [0.0, 0.0, 2.0]);
        assert_eq!(w.agent(id).unwrap().holdings[2], 4.0);
        assert_eq!(w.site(Pos::new(5, 3)).resource[2], 0.0);
    }

    #[test]
    fn candidates_list_the_current_site_first_then_sight_order_skipping_occupied() {
        let mut w = blank_world(11, 11);
        let id = mover(&mut w, 2);
        spawn(&mut w, 5, 6); // occupied: skipped
        set_sugar(&mut w, 5, 5, 1.0);
        set_sugar(&mut w, 7, 5, 3.0);
        let c = candidates(&w, id);
        assert_eq!(c[0], (Pos::new(5, 5), 0, 1.0));
        assert_eq!(c.len(), 1 + 8 - 1);
        assert!(c.iter().all(|x| x.0 != Pos::new(5, 6)));
        assert!(c.windows(2).all(|p| p[0].1 <= p[1].1), "distance order");
        assert!(c.contains(&(Pos::new(7, 5), 2, 3.0)));
    }

    #[test]
    fn a_pollutant_discounts_only_the_goods_it_devalues() {
        let mut w = blank_world(11, 11);
        add_goods(&mut w.config, 3);
        let id = mover(&mut w, 3);
        {
            let a = w.agent_mut(id).unwrap();
            a.metabolism[..3].copy_from_slice(&[1, 1, 1]);
            a.holdings[..3].copy_from_slice(&[30.0, 30.0, 2.0]);
        }
        let pollutant = |name: &str, devalues: Vec<bool>| Pollutant {
            name: name.into(),
            production: vec![0.0; 3],
            consumption: vec![0.0; 3],
            devalues,
        };
        w.config.pollution = Pollution {
            enabled: true,
            pollutants: vec![
                pollutant("smoke", vec![true, false, false]),
                pollutant("runoff", vec![false, false, true]),
            ],
        };
        // 2 of good 2 under runoff 3 counts as 2 · 1/4 = 0.5; 1 of good 2
        // under smoke (which spares good 2) counts as 1.
        set_resource(&mut w, 5, 3, 2, 2.0);
        w.site_mut(Pos::new(5, 3)).pollution[1] = 3.0;
        set_resource(&mut w, 5, 7, 2, 1.0);
        w.site_mut(Pos::new(5, 7)).pollution[0] = 9.0;
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7));
    }

    // --- Minds 3: truffles ---

    fn truffle_world(width: u32, height: u32, share: f64, value: f64, regrow: u32) -> World {
        let mut c = blank_config(width, height);
        c.truffles.share = share;
        c.truffles.value = value;
        c.truffles.regrow = regrow;
        World::new(c, 7).unwrap()
    }

    #[test]
    fn stopping_on_a_ripe_spot_gains_value_then_the_spot_regrows_after_regrow_ticks() {
        let mut w = truffle_world(11, 11, 1.0, 5.0, 3);
        let id = mover(&mut w, 1);
        let pos = w.agent(id).unwrap().pos;
        assert_eq!(w.truffle(pos), Some(true), "starts ripe");
        let before_holdings = w.agent(id).unwrap().holdings[0];

        w.tick = 10;
        let h = go_and_gather(&mut w, id, pos);
        assert_eq!(h.gathered[0], 5.0, "sugar (0) plus the truffle's value");
        assert_eq!(w.agent(id).unwrap().holdings[0], before_holdings + 5.0);
        assert_eq!(w.truffle(pos), Some(false), "picked: unripe now");
        assert_eq!(w.events.truffles_found, 1);

        // Stopping again before tick + regrow (13) gains nothing extra.
        w.tick = 12;
        let h2 = go_and_gather(&mut w, id, pos);
        assert_eq!(h2.gathered[0], 0.0);
        assert_eq!(w.events.truffles_found, 1, "not counted again");

        // At exactly tick + regrow it's ripe again.
        w.tick = 13;
        let h3 = go_and_gather(&mut w, id, pos);
        assert_eq!(h3.gathered[0], 5.0);
        assert_eq!(w.truffle(pos), Some(false));
        assert_eq!(w.events.truffles_found, 2);
    }

    #[test]
    fn truffles_found_counts_rememberers_separately() {
        let mut w = truffle_world(11, 11, 1.0, 5.0, 3);
        let id = mover(&mut w, 1);
        w.agent_mut(id).unwrap().remembers = true;
        let pos = w.agent(id).unwrap().pos;
        go_and_gather(&mut w, id, pos);
        assert_eq!(w.events.truffles_found, 1);
        assert_eq!(w.events.truffles_by_rememberers, 1);
    }

    #[test]
    fn truffles_found_by_a_non_rememberer_does_not_count_toward_by_rememberers() {
        let mut w = truffle_world(11, 11, 1.0, 5.0, 3);
        let id = mover(&mut w, 1); // remembers is false by default
        let pos = w.agent(id).unwrap().pos;
        go_and_gather(&mut w, id, pos);
        assert_eq!(w.events.truffles_found, 1);
        assert_eq!(w.events.truffles_by_rememberers, 0);
    }

    #[test]
    fn truffles_without_a_spot_gather_nothing_extra() {
        let mut w = truffle_world(11, 11, 0.0, 5.0, 3);
        let id = mover(&mut w, 1);
        let pos = w.agent(id).unwrap().pos;
        assert_eq!(w.truffle(pos), None);
        let h = go_and_gather(&mut w, id, pos);
        assert_eq!(h.gathered[0], 0.0);
        assert_eq!(w.events.truffles_found, 0);
    }

    #[test]
    fn truffle_spots_dont_change_rule_ms_candidate_values() {
        // An agent that hasn't harvested the spot doesn't know it's there, so
        // its value never leaks into the plain rule M valuation.
        let mut w = truffle_world(11, 11, 1.0, 100.0, 3);
        let id = mover(&mut w, 2);
        let c = candidates(&w, id);
        assert!(
            c.iter().all(|&(_, _, v)| v == 0.0),
            "every candidate is worth exactly its (zero) sugar, truffle or not"
        );
    }

    // --- Minds 3: deciding with memory ---

    /// A walking world with memory on (span 100, everyone remembering,
    /// belief `belief`) and growback 1 per tick.
    fn memory_world(belief: crate::config::Belief) -> World {
        let mut c = blank_config(21, 21);
        c.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        c.memory.span = 100;
        c.memory.share = 1.0;
        c.memory.belief = belief;
        c.growback.rate = 1.0;
        c.growback.instant = false;
        World::new(c, 7).unwrap()
    }

    /// A rememberer at (x, y) with vision 1.
    fn rememberer(w: &mut World, x: u32, y: u32) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.vision = 1;
        a.remembers = true;
        id
    }

    /// Plants a memory of sugar `level` (most `most`) seen at (x, y) at `tick`.
    fn remember(w: &mut World, id: AgentId, x: u32, y: u32, level: f64, most: f64, tick: u64) {
        let idx = w.torus.index(Pos::new(x, y)) as u32;
        let n = w.config.goods.len();
        let (mut levels, mut mosts) = (vec![0.0; n], vec![0.0; n]);
        (levels[0], mosts[0]) = (level, most);
        let seen = crate::minds::memory::Seen::new(&levels, &mosts, tick);
        w.agent_mut(id).unwrap().memory.sites.insert(idx, seen);
    }

    #[test]
    fn a_rememberer_chooses_a_remembered_site_out_of_sight_and_walks_toward_it() {
        let mut w = memory_world(crate::config::Belief::Recall);
        let id = rememberer(&mut w, 5, 5);
        set_sugar(&mut w, 5, 9, 4.0);
        remember(&mut w, id, 5, 9, 4.0, 4.0, 0);
        w.tick = 1;
        let (c, start) = candidates_with_memory(&w, id);
        assert_eq!(start, 5, "own site plus four in sight");
        assert_eq!(&c[start..], &[(Pos::new(5, 9), 4, 4.0)]);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6));
        assert_eq!(w.agent(id).unwrap().plan.target, Some(Pos::new(5, 9)));
    }

    #[test]
    fn project_grows_a_remembered_site_at_its_own_rows_seasonal_rate() {
        let mut w = memory_world(crate::config::Belief::Project);
        w.config.seasons.enabled = true;
        w.config.seasons.period = 50;
        w.config.seasons.winter_divisor = 8;
        let id = rememberer(&mut w, 5, 5);
        // Both seen harvested at tick 0, both able to hold 10.
        remember(&mut w, id, 5, 1, 0.0, 10.0, 0); // north: summer at tick 4
        remember(&mut w, id, 5, 15, 0.0, 10.0, 0); // south: winter at tick 4
        w.tick = 4;
        let (c, start) = candidates_with_memory(&w, id);
        assert_eq!(
            &c[start..],
            &[
                (Pos::new(5, 1), 4, 4.0),   // 0 + 4 ticks × 1
                (Pos::new(5, 15), 10, 0.5), // 0 + 4 ticks × 1/8
            ],
            "each site projects at its own row's rate, not the agent's"
        );
    }

    #[test]
    fn a_non_rememberer_ignores_whatever_is_in_its_memory() {
        let mut w = memory_world(crate::config::Belief::Recall);
        let id = rememberer(&mut w, 5, 5);
        set_sugar(&mut w, 5, 9, 4.0);
        remember(&mut w, id, 5, 9, 4.0, 4.0, 0);
        w.agent_mut(id).unwrap().remembers = false;
        w.tick = 1;
        let (c, start) = candidates_with_memory(&w, id);
        assert_eq!(start, c.len(), "no remembered entries");
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
    }

    #[test]
    fn stale_memories_past_span_are_not_candidates() {
        let mut w = memory_world(crate::config::Belief::Recall);
        let id = rememberer(&mut w, 5, 5);
        remember(&mut w, id, 5, 9, 4.0, 4.0, 0);
        w.tick = 100;
        assert_eq!(candidates_with_memory(&w, id).0.len(), 6, "age span stays");
        w.tick = 101;
        let (c, start) = candidates_with_memory(&w, id);
        assert_eq!(start, c.len(), "age span + 1 is forgotten");
    }

    #[test]
    fn recall_passes_over_a_harvested_memory_that_project_regrows() {
        let target = |belief| {
            let mut w = memory_world(belief);
            let id = rememberer(&mut w, 5, 5);
            set_sugar(&mut w, 5, 9, 4.0);
            remember(&mut w, id, 5, 9, 0.0, 4.0, 0); // seen harvested
            w.tick = 10;
            act(&mut w, id);
            w.agent(id).unwrap().plan.target
        };
        assert_eq!(
            target(crate::config::Belief::Recall),
            Some(Pos::new(5, 5)),
            "recall still believes 0 there"
        );
        assert_eq!(
            target(crate::config::Belief::Project),
            Some(Pos::new(5, 9)),
            "project believes it has grown back (min(0 + 10, 4))"
        );
    }

    #[test]
    fn a_walker_stops_one_step_short_of_an_occupied_remembered_target() {
        let mut w = memory_world(crate::config::Belief::Recall);
        w.config.movement.speed = 3;
        let id = rememberer(&mut w, 5, 5);
        set_sugar(&mut w, 5, 8, 4.0);
        remember(&mut w, id, 5, 8, 4.0, 4.0, 0);
        let other = spawn(&mut w, 5, 8); // out of sight, unknown to the walker
        w.tick = 1;
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 7));
        assert_eq!(w.agent(other).unwrap().pos, Pos::new(5, 8));
        assert_eq!(w.occupant(Pos::new(5, 8)), Some(other));
        assert_eq!(w.agent(id).unwrap().plan.path, vec![Pos::new(5, 8)]);
        assert_eq!(w.site(Pos::new(5, 8)).resource[0], 4.0, "not gathered");
    }

    #[test]
    fn remembered_sites_are_never_filtered_by_true_occupancy() {
        let mut w = memory_world(crate::config::Belief::Recall);
        let id = rememberer(&mut w, 5, 5);
        remember(&mut w, id, 5, 9, 4.0, 4.0, 0);
        spawn(&mut w, 5, 9);
        w.tick = 1;
        let (c, start) = candidates_with_memory(&w, id);
        assert_eq!(&c[start..], &[(Pos::new(5, 9), 4, 4.0)]);
    }

    #[test]
    fn remembered_candidates_skip_walls_the_own_site_and_sites_in_sight() {
        let mut c = blank_config(21, 21);
        c.movement = crate::config::Movement {
            mode: crate::config::MoveMode::Walk,
            speed: 1,
        };
        c.memory.span = 100;
        c.walls = vec![crate::config::Wall {
            x: 9,
            y: 9,
            width: 1,
            height: 1,
            opaque: false,
        }];
        let mut w = World::new(c, 7).unwrap();
        let id = rememberer(&mut w, 5, 5);
        remember(&mut w, id, 9, 9, 4.0, 4.0, 0); // a wall
        remember(&mut w, id, 5, 5, 4.0, 4.0, 0); // own site
        remember(&mut w, id, 6, 5, 4.0, 4.0, 0); // in sight
        remember(&mut w, id, 2, 2, 4.0, 4.0, 0); // out of sight
        w.tick = 1;
        let (c, start) = candidates_with_memory(&w, id);
        assert_eq!(&c[start..], &[(Pos::new(2, 2), 6, 4.0)]);
        assert_eq!(c[0], (Pos::new(5, 5), 0, 0.0), "own site at its true value");
    }

    #[test]
    fn remembered_distances_wrap_the_torus() {
        let mut w = memory_world(crate::config::Belief::Recall);
        let id = rememberer(&mut w, 1, 1);
        remember(&mut w, id, 19, 20, 4.0, 4.0, 0);
        w.tick = 1;
        let (c, start) = candidates_with_memory(&w, id);
        assert_eq!(
            &c[start..],
            &[(Pos::new(19, 20), 5, 4.0)],
            "3 + 2 across the edges"
        );
    }

    #[test]
    fn a_known_ripe_truffle_in_sight_adds_its_value_and_an_unknown_one_does_not() {
        let mut w = memory_world(crate::config::Belief::Recall);
        w.config.truffles.share = 1.0;
        w.config.truffles.value = 5.0;
        w.config.truffles.regrow = 30;
        // Rebuild so the spots are laid out.
        let config = w.config.clone();
        let mut w = World::new(config, 7).unwrap();
        let id = rememberer(&mut w, 5, 5);
        set_sugar(&mut w, 6, 5, 1.0);
        remember(&mut w, id, 6, 5, 1.0, 1.0, 0);
        let east = w.torus.index(Pos::new(6, 5)) as u32;
        w.agent_mut(id)
            .unwrap()
            .memory
            .sites
            .get_mut(&east)
            .unwrap()
            .truffle = Some(crate::minds::memory::TruffleSeen {
            ripe: true,
            tick: 0,
        });
        w.tick = 1;
        let c = candidates(&w, id);
        assert!(c.contains(&(Pos::new(6, 5), 1, 6.0)), "known: 1 + 5");
        assert!(c.contains(&(Pos::new(4, 5), 1, 0.0)), "unknown spot: 0");
    }

    #[test]
    fn choosing_a_remembered_target_worth_less_than_believed_counts_a_stale_choice() {
        let mut w = memory_world(crate::config::Belief::Recall);
        let id = rememberer(&mut w, 5, 5);
        set_sugar(&mut w, 5, 9, 1.0); // true value now
        remember(&mut w, id, 5, 9, 4.0, 4.0, 0); // believed 4
        w.tick = 1;
        act(&mut w, id);
        let e = w.events();
        assert_eq!(
            (
                e.moves,
                e.remembered_moves,
                e.stale_choices,
                e.belief_error_sum
            ),
            (1, 1, 1, 3.0)
        );
    }

    #[test]
    fn a_choice_in_sight_counts_a_move_but_nothing_remembered() {
        let mut w = memory_world(crate::config::Belief::Recall);
        let id = rememberer(&mut w, 5, 5);
        set_sugar(&mut w, 6, 5, 9.0);
        remember(&mut w, id, 5, 9, 4.0, 4.0, 0);
        w.tick = 1;
        act(&mut w, id);
        let e = w.events();
        assert_eq!(
            (
                e.moves,
                e.remembered_moves,
                e.stale_choices,
                e.belief_error_sum
            ),
            (1, 0, 0, 0.0)
        );
        let other = spawn(&mut w, 15, 15);
        act(&mut w, other);
        assert_eq!(
            w.events().moves,
            1,
            "a non-rememberer's choice isn't counted"
        );
    }

    #[test]
    fn the_diagnostics_true_value_counts_a_ripe_spot_there_now() {
        let mut c = memory_world(crate::config::Belief::Recall).config.clone();
        c.truffles.share = 1.0;
        c.truffles.value = 5.0;
        c.truffles.regrow = 30;
        let mut w = World::new(c, 7).unwrap();
        let id = rememberer(&mut w, 5, 5);
        set_sugar(&mut w, 5, 9, 1.0);
        remember(&mut w, id, 5, 9, 4.0, 4.0, 0); // believed 4, spot unknown
        w.tick = 1;
        act(&mut w, id);
        let e = w.events();
        // True: 1 sugar + a ripe 5-sugar truffle = 6 > believed 4.
        assert_eq!((e.stale_choices, e.belief_error_sum), (0, 2.0));
    }

    #[test]
    fn memory_on_with_nobody_remembering_lists_nothing_remembered() {
        let mut c = crate::presets::by_id("walk-capacity").unwrap().config;
        c.memory.span = 50;
        c.memory.share = 0.0;
        let mut w = World::new(c, 3).unwrap();
        for _ in 0..30 {
            w.step();
            assert_eq!(w.events().remembered_moves, 0);
            assert_eq!(w.events().moves, 0);
            let ids: Vec<AgentId> = w.agents().map(|a| a.id).collect();
            for id in ids {
                assert!(!w.agent(id).unwrap().remembers);
                let (c, start) = candidates_with_memory(&w, id);
                assert_eq!(start, c.len());
            }
        }
    }

    #[test]
    fn a_crowded_world_of_rememberers_runs_without_two_agents_on_a_site() {
        let mut c = crate::presets::by_id("walk-capacity").unwrap().config;
        c.memory.span = 100;
        c.memory.share = 1.0;
        let mut w = World::new(c, 3).unwrap();
        let mut remembered = 0;
        for _ in 0..150 {
            w.step();
            remembered += w.events().remembered_moves;
            let mut at: Vec<Pos> = w.agents().map(|a| a.pos).collect();
            let n = at.len();
            at.sort_by_key(|p| (p.y, p.x));
            at.dedup();
            assert_eq!(at.len(), n, "one agent per site");
            assert!(w.agents().all(|a| w.occupant(a.pos) == Some(a.id)));
        }
        assert!(remembered > 0, "someone chose a remembered site");
    }
}
