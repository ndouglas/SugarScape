//! Minds 5: central-place foraging (`central.enabled`; Stephens & Krebs
//! 1986 §3.5, after Orians & Pearson 1979): an agent forages in round trips
//! from a home and carries its loads back to a larder there.
//!
//! - **Home and larder.** An agent's home is where it was placed or born
//!   (`Agent.home`, set by `World::insert_agent`); its larder is its cache
//!   at home (`minds::caching`).
//! - **Delivery** happens at home, before the agent's move on the tick it
//!   leaves again: it buries its trip's whole load (`Agent.load_trip`,
//!   good 0 gathered since it last left) above R, one tick's need, into the
//!   larder: min(load, holdings − R). A positive burial is a delivery
//!   (`events.deliveries`, `events.delivered`: the load brought home).
//!   Sugar it held before the trip (its endowment) isn't a load and stays
//!   in hand.
//! - **Provisions.** Then, if it holds less than P = R × (2D + 2), with D
//!   the lattice distance to where it's going next (food for the walk out
//!   and back and a tick to spare), it digs the larder for the difference
//!   (it costs no tick, like burying). Provisions come out of the larder
//!   after the delivery is counted, so a far trip's load counts in full,
//!   not net of the longer walk the next trip needs. Without provisions an
//!   agent that delivered everything above R would starve on its first
//!   step out (holdings ≤ 0 kill); with metabolism 0, P = 0 and the
//!   delivery is the whole load.
//! - **The carrying limit** caps the trip's load, not load plus provisions:
//!   a harvest takes at most C − `load_trip` (`movement::go_and_gather`).
//! - **Reserve.** In a central-place world the caching reserve is one
//!   tick's need (`caching::reserve`), so an agent just back from a
//!   delivery, holding its provisions, isn't hungry for its larder and
//!   doesn't dig its own load straight back. An agent holding less than one
//!   tick's need digs the larder on arrival home, up to that need
//!   (`movement::go_and_gather`).
//!
//! **The rule under `mvt`.** The marginal-value rule over round trips.
//!
//! - **ρ is the delivery rate**, `Agent.delivery_rate`: every tick, ρ ← ρ +
//!   α(delivered this tick − ρ), α = `mvt.alpha`, so travel ticks and ticks
//!   in a patch count 0 delivered and only the tick of a delivery counts
//!   its load. A new agent's ρ starts at its good-0 metabolism, as in
//!   Minds 4. This is not Minds 4's ρ (`Agent.rate`, left untouched here),
//!   which smooths the sugar *gathered* each tick: a forager that eats as
//!   it goes earns its intake where it harvests, but a central-place
//!   forager's currency is what reaches home, so the rate it should beat is
//!   delivered sugar per tick of the whole round trip, travel included.
//! - **Leaving home.** At home (after delivering) it commits to the best
//!   site it knows of away from home and worth something (in sight or
//!   remembered; ties nearer, then the lower site index, no draw), and
//!   walks there without reconsidering, as Minds 4 commits to a leave. With
//!   nothing known it stays home.
//! - **In a patch.** Away from home and not committed, it harvests the best
//!   site within distance 1 (its own site included, home excluded; rule M's
//!   tie rule) while that site yields ≥ ρ and something (> 0: an empty site
//!   isn't a yield, or ρ = 0 would hold it on bare ground forever), and its
//!   load is under the carrying limit less one tick's need (`full` is
//!   `load_trip` + R ≥ C: a load within a tick's need of the limit has too
//!   little room for another harvest to be worth the tick). Otherwise
//!   it commits to going home. It also goes home when what it holds would only just get it
//!   there (holdings ≤ R × (distance home + 1)).
//! - **The comparison.** "The best site within distance 1 yields ≥ ρ" sets
//!   one site's per-tick yield against a long-run rate. That is the
//!   marginal value theorem's leaving rule (leave when the marginal gain
//!   falls to the long-run rate) only where the agent's position doesn't
//!   change the cost of the walk home. On a lattice it does: a step from
//!   distance d to d + 1 costs two ticks (the harvest and a longer walk
//!   home), a step back from d + 1 to d costs none net, and the rule can't
//!   see either. With ρ held at the true optimal rate, the comparison
//!   gives loads 45, 60 and 63 on the analytic test's staircase at d = 2,
//!   5 and 10, where the optima are 51, 60 and 65, and 45 and 63 are
//!   dominated even loads (`fixed_rho_misses_the_parity`). The learned ρ,
//!   a pulse average, has decayed below the long-run rate by the time of
//!   the in-patch decisions (≈ 3.21 against 3.75 at d = 5, ≈ 1.88 against
//!   2.32 at d = 10), and that underestimate cancels the parity error: the
//!   loads the rule reaches at d = 5 and 10 are optimal for that reason,
//!   not because the comparison is the theorem's.
//! - **Guard bias.** In populated worlds (metabolism > 0, a binding
//!   capacity), `full` and `low` end trips early and push loads short, and
//!   provisions cap what's delivered; the analytic test turns all of these
//!   off (metabolism 0, capacity 2000).
//!
//! **Under `goap`** (report only): the goal is "deliver G" (G = good-0
//! metabolism × `goap.horizon`, Minds 4's goal): Minds 4's foraging plan
//! with one more action, going home, allowed once the plan has gathered G;
//! a plan is complete only at home, so its last step is home and its cost
//! counts the walk back. At home the agent delivers and plans; away, it
//! follows its plan, and goes home when a step is taken by another agent
//! or walled off.
//!
//! Nothing here draws, except rule M's tie draw (`choose`) in a patch.

use crate::agent::{AgentId, GoapPlan};
use crate::config::DecisionRule;
use crate::geometry::{Pos, Torus};
use crate::rules::movement::{arrive, choose, lattice_distance, record_choice};
use crate::rules::Harvest;
use crate::world::World;

use super::caching::{bury, dig, reserve};
use super::goap::forage::{reachable_candidates, PLAN_LIMIT};
use super::goap::{plan, Domain};
use super::mvt::updated_rate;

/// One central-place step under the configured decision rule (validated to
/// `mvt` or `goap`).
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    match world.config.decision.rule {
        DecisionRule::Goap => act_goap(world, id),
        _ => act_mvt(world, id),
    }
}

/// `id`'s home, set to where it stands if it has none (an agent inserted
/// before `central.enabled` was set, in tests).
fn home_of(world: &mut World, id: AgentId) -> Pos {
    let a = world.agent_mut(id).expect("live agent");
    *a.home.get_or_insert(a.pos)
}

/// P = R × (2D + 2): provisions for a trip to a site `dist` away.
fn provisions(world: &World, id: AgentId, dist: u32) -> f64 {
    reserve(world, id) * f64::from(2 * dist + 2)
}

/// At home: buries the trip's whole load above R (one tick's need) into the
/// larder and counts that as the delivery, then takes the next trip's
/// provisions, `keep`, back from the larder when it holds less than that.
/// Resets the trip's load and returns what was delivered (0 for no
/// delivery).
///
/// `delivered` is the load brought home: min(`load_trip`, holdings − R),
/// the sugar gathered on the trip less anything it had to eat of it on the
/// way (when it came home holding less than its load plus R). Provisions
/// are drawn from the larder afterwards, so they don't shrink a delivery:
/// a far trip's load counts in full, not net of the longer walk it funds.
/// ρ is measured in the same unit.
fn at_home(world: &mut World, id: AgentId, home: Pos, keep: f64) -> f64 {
    let r = reserve(world, id);
    let a = world.agent(id).expect("live agent");
    let (held, load) = (a.holdings[0], a.load_trip);
    let q = load.min(held - r);
    let mut delivered = 0.0;
    if q > 0.0 {
        // With a bury cost, less than q may fit.
        let q = bury(world, id, q);
        let e = &mut world.events;
        e.deliveries += 1;
        e.delivered += q;
        delivered = q;
        world.agent_mut(id).expect("live agent").last_load = q;
    }
    let held = world.agent(id).expect("live agent").holdings[0];
    if held < keep {
        let site = world.torus.index(home) as u32;
        let took = dig(world, id, site, keep - held);
        world.agent_mut(id).expect("live agent").holdings[0] += took;
    }
    world.agent_mut(id).expect("live agent").load_trip = 0.0;
    delivered
}

/// The best candidate away from `home` worth anything: highest value, then
/// nearer, then the lower site index. Draws nothing.
fn best_away(torus: Torus, candidates: &[(Pos, u32, f64)], home: Pos) -> Option<(Pos, u32, f64)> {
    candidates
        .iter()
        .filter(|c| c.0 != home && c.2 > 0.0)
        .min_by(|a, b| {
            b.2.total_cmp(&a.2)
                .then(a.1.cmp(&b.1))
                .then(torus.index(a.0).cmp(&torus.index(b.0)))
        })
        .copied()
}

/// Heads for `target` (counted as Minds 3 counts a choice).
fn go(
    world: &mut World,
    id: AgentId,
    candidates: &[(Pos, u32, f64)],
    start: usize,
    target: Pos,
) -> Harvest {
    record_choice(world, id, candidates, start, target);
    arrive(world, id, target)
}

/// After the move: clears a commitment reached, adds the harvest to the
/// trip's load and updates ρ from what was delivered this tick.
fn finish(world: &mut World, id: AgentId, harvest: Harvest, delivered: f64) -> Harvest {
    let alpha = world.config.mvt.alpha;
    let a = world.agent_mut(id).expect("live agent");
    if a.leaving == Some(a.pos) {
        a.leaving = None;
    }
    a.load_trip += harvest.gathered[0];
    a.delivery_rate = updated_rate(a.delivery_rate, alpha, delivered);
    harvest
}

/// The marginal-value rule over round trips (see the module doc).
fn act_mvt(world: &mut World, id: AgentId) -> Harvest {
    let home = home_of(world, id);
    let (candidates, start) = reachable_candidates(world, id);
    let torus = world.torus;
    let r = reserve(world, id);
    let capacity = f64::from(world.config.caching.capacity);
    let a = world.agent(id).expect("live agent");
    let (pos, committed, rho, held) = (a.pos, a.leaving, a.delivery_rate, a.holdings[0]);
    let mut delivered = 0.0;
    let target = match committed {
        Some(t) if t == home && pos != home => home,
        Some(t) if t != pos && t != home && candidates.iter().any(|c| c.0 == t) => t,
        _ => {
            world.agent_mut(id).expect("live agent").leaving = None;
            if pos == home {
                let best = best_away(torus, &candidates, home);
                let keep = provisions(world, id, best.map_or(0, |c| c.1));
                delivered = at_home(world, id, home, keep);
                match best {
                    Some(c) => {
                        world.agent_mut(id).expect("live agent").leaving = Some(c.0);
                        world.events.leaves += 1;
                        c.0
                    }
                    None => home,
                }
            } else {
                let local: Vec<(Pos, u32, f64)> = candidates
                    .iter()
                    .copied()
                    .filter(|c| c.1 <= 1 && c.0 != home)
                    .collect();
                let best_local = local.iter().map(|c| c.2).fold(f64::NEG_INFINITY, f64::max);
                // Full: its load is within a tick's need of the limit (the
                // limit caps the load, not load plus provisions).
                let load = world.agent(id).expect("live agent").load_trip;
                let full = capacity > 0.0 && load + r >= capacity;
                let low = r > 0.0 && held <= r * f64::from(lattice_distance(torus, pos, home) + 1);
                if !full && !low && best_local > 0.0 && best_local >= rho {
                    choose(&local, &mut world.rng)
                } else {
                    world.agent_mut(id).expect("live agent").leaving = Some(home);
                    home
                }
            }
        }
    };
    let harvest = go(world, id, &candidates, start, target);
    finish(world, id, harvest, delivered)
}

/// "Deliver G": Minds 4's foraging domain plus going home. Slots are sites
/// (0 the agent's own); the state is (slot it's at, mask harvested, home).
struct Deliver {
    values: Vec<f64>,
    /// Travel ticks between slots, row-major `n × n`.
    dist: Vec<u32>,
    /// Travel ticks from each slot to home.
    home: Vec<u32>,
    /// Slots in action order: value descending, then distance from slot 0,
    /// then site index.
    order: Vec<u8>,
    goal: f64,
}

/// The go-home action.
const HOME: u8 = u8::MAX;

impl Deliver {
    fn gathered(&self, mask: u16) -> f64 {
        (0..self.values.len())
            .filter(|&i| mask & (1 << i) != 0)
            .map(|i| self.values[i])
            .sum()
    }
}

impl Domain for Deliver {
    type State = (u8, u16, bool);
    type Action = u8;

    fn actions(
        &self,
        &(at, mask, done): &(u8, u16, bool),
        out: &mut Vec<(u8, f64, (u8, u16, bool))>,
    ) {
        if done {
            return;
        }
        let at_ = usize::from(at);
        if self.gathered(mask) >= self.goal {
            // Going home, then delivering: its walk plus a tick.
            out.push((HOME, f64::from(self.home[at_] + 1), (at, mask, true)));
            return;
        }
        let n = self.values.len();
        for &i in &self.order {
            if mask & (1 << i) == 0 {
                let cost = self.dist[at_ * n + usize::from(i)] + 1;
                out.push((i, f64::from(cost), (i, mask | 1 << i, false)));
            }
        }
    }

    fn is_goal(&self, s: &(u8, u16, bool)) -> bool {
        s.2
    }

    /// Done: 0. Gathered G: the walk home and the delivery tick, exactly.
    /// Short of G: Minds 4's ⌈(G − gathered) / v_max⌉ harvests plus the
    /// delivery tick (every path home costs at least that tick), which keeps
    /// it admissible and consistent.
    fn heuristic(&self, &(at, mask, done): &(u8, u16, bool)) -> f64 {
        if done {
            return 0.0;
        }
        let g = self.gathered(mask);
        if g >= self.goal {
            return f64::from(self.home[usize::from(at)] + 1);
        }
        let v_max = (0..self.values.len())
            .filter(|&i| mask & (1 << i) == 0)
            .map(|i| self.values[i])
            .fold(0.0, f64::max);
        if v_max > 0.0 {
            ((self.goal - g) / v_max).ceil() + 1.0
        } else {
            f64::INFINITY
        }
    }
}

/// A found "deliver G" plan: its steps (the last one home), its cost in
/// ticks and the goal G.
struct Trip {
    steps: Vec<(Pos, f64)>,
    cost: f64,
    goal: f64,
}

/// A "deliver G" plan from home over the agent's candidates: its steps (the
/// last one home) and its cost in ticks, or `None` when what it knows falls
/// short of G or the search passes `PLAN_LIMIT`.
fn plan_trip(
    world: &World,
    id: AgentId,
    candidates: &[(Pos, u32, f64)],
    home: Pos,
) -> Option<Trip> {
    let a = world.agent(id).expect("live agent");
    let burn = a.effective_metabolism(0, world.config.disease.active_fee());
    let goal = burn * f64::from(world.config.goap.horizon);
    let torus = world.torus;
    let key = |c: &(Pos, u32, f64)| match world.config.goap.shortlist {
        crate::config::Shortlist::Rate => c.2 / (f64::from(c.1) + 1.0),
        crate::config::Shortlist::Value => c.2,
    };
    let mut others: Vec<usize> = (1..candidates.len())
        .filter(|&i| candidates[i].0 != home)
        .collect();
    others.sort_by(|&i, &j| {
        let (a, b) = (&candidates[i], &candidates[j]);
        key(b)
            .total_cmp(&key(a))
            .then(a.1.cmp(&b.1))
            .then(torus.index(a.0).cmp(&torus.index(b.0)))
    });
    others.truncate(world.config.goap.k as usize);
    let sites: Vec<(Pos, f64)> = std::iter::once(0)
        .chain(others)
        .map(|i| (candidates[i].0, candidates[i].2))
        .collect();
    let n = sites.len();
    let raw = |i: usize, j: usize| lattice_distance(torus, sites[i].0, sites[j].0);
    let mut dist = vec![0; n * n];
    for i in 0..n {
        for j in 0..n {
            dist[i * n + j] = raw(i, j);
        }
    }
    let mut order: Vec<u8> = (0..n as u8).collect();
    order.sort_by(|&a, &b| {
        let (a, b) = (usize::from(a), usize::from(b));
        sites[b]
            .1
            .total_cmp(&sites[a].1)
            .then(raw(0, a).cmp(&raw(0, b)))
            .then(torus.index(sites[a].0).cmp(&torus.index(sites[b].0)))
    });
    let domain = Deliver {
        values: sites.iter().map(|s| s.1).collect(),
        dist,
        home: sites
            .iter()
            .map(|s| lattice_distance(torus, s.0, home))
            .collect(),
        order,
        goal,
    };
    let p = plan(&domain, (0, 0, false), PLAN_LIMIT)?;
    let steps = p
        .actions
        .iter()
        .map(|&s| {
            if s == HOME {
                (home, 0.0)
            } else {
                sites[usize::from(s)]
            }
        })
        .collect();
    Some(Trip {
        steps,
        cost: p.cost,
        goal,
    })
}

/// "Deliver G" under GOAP (see the module doc).
fn act_goap(world: &mut World, id: AgentId) -> Harvest {
    let home = home_of(world, id);
    let (candidates, start) = reachable_candidates(world, id);
    let a = world.agent(id).expect("live agent");
    let pos = a.pos;
    let next = a
        .goap_plan
        .as_ref()
        .and_then(|p| p.steps.first().copied())
        .map(|s| s.0);
    let holds = |t: Pos| {
        t == home || (!world.occupant(t).is_some_and(|o| o != id) && !world.walled_apart(pos, t))
    };
    let mut delivered = 0.0;
    let target = match next {
        Some(t) if holds(t) => t,
        _ => {
            world.agent_mut(id).expect("live agent").goap_plan = None;
            if pos == home {
                match plan_trip(world, id, &candidates, home) {
                    Some(Trip { steps, cost, goal }) => {
                        // Provisions for the plan's ticks, the walk home
                        // included.
                        let keep = reserve(world, id) * (cost + 1.0);
                        delivered = at_home(world, id, home, keep);
                        let t = steps.first().map_or(home, |s| s.0);
                        world.events.plans += 1;
                        world.events.plan_steps_sum += steps.len() as u32;
                        world.agent_mut(id).expect("live agent").goap_plan = Some(GoapPlan {
                            gathers: steps.iter().map(|s| s.1).sum(),
                            steps,
                            goal,
                        });
                        t
                    }
                    None => {
                        world.events.fallback_short += 1;
                        let keep = provisions(world, id, 0);
                        delivered = at_home(world, id, home, keep);
                        home
                    }
                }
            } else {
                home
            }
        }
    };
    let harvest = go(world, id, &candidates, start, target);
    let a = world.agent_mut(id).expect("live agent");
    if let Some(g) = a.goap_plan.as_mut() {
        if g.steps.first().is_some_and(|s| s.0 == a.pos) {
            g.steps.remove(0);
        }
    }
    finish(world, id, harvest, delivered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{MoveMode, Movement};
    use crate::testkit::*;

    /// A walking central-place world under `rule`, capacity `capacity`.
    fn central_config(size: u32, rule: DecisionRule, capacity: u32) -> crate::config::Config {
        let mut c = blank_config(size, size);
        c.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        c.decision.rule = rule;
        c.caching.capacity = capacity;
        c.central.enabled = true;
        c
    }

    fn central_world(size: u32, rule: DecisionRule, capacity: u32) -> World {
        World::new(central_config(size, rule, capacity), 7).unwrap()
    }

    /// One tick of `id`'s turn (move, bury, eat, maybe die), with the tick's
    /// events fresh.
    fn turn(w: &mut World, id: AgentId) {
        w.events = crate::world::TickEvents::default();
        crate::rules::agent_turn(w, id);
        w.tick += 1;
    }

    fn at(w: &World, x: u32, y: u32) -> u32 {
        w.torus.index(Pos::new(x, y)) as u32
    }

    #[test]
    fn homes_are_where_agents_start_only_in_central_worlds() {
        let mut w = central_world(11, DecisionRule::Mvt, 20);
        let id = spawn(&mut w, 3, 4);
        assert_eq!(w.agent(id).unwrap().home, Some(Pos::new(3, 4)));
        let mut plain = blank_world(11, 11);
        let id = spawn(&mut plain, 3, 4);
        assert_eq!(plain.agent(id).unwrap().home, None);
        // Placement by the world itself sets homes too.
        let mut c = central_config(11, DecisionRule::Mvt, 20);
        c.population = 5;
        let w = World::new(c, 3).unwrap();
        assert!(w.agents().all(|a| a.home == Some(a.pos)));
        assert!(w
            .agents()
            .all(|a| a.delivery_rate == f64::from(a.metabolism[0])));
    }

    #[test]
    fn the_reserve_is_one_ticks_need_in_a_central_world() {
        let mut w = central_world(11, DecisionRule::Mvt, 20);
        let id = spawn(&mut w, 5, 5);
        w.agent_mut(id).unwrap().metabolism[0] = 3;
        w.config.goap.horizon = 10;
        assert_eq!(reserve(&w, id), 3.0);
        w.config.central.enabled = false;
        assert_eq!(reserve(&w, id), 30.0);
    }

    /// An agent at home (5, 5) with metabolism 1, vision 6, holding `held`,
    /// under capacity 40.
    fn forager(held: f64) -> (World, AgentId) {
        let mut w = central_world(21, DecisionRule::Mvt, 40);
        let id = spawn(&mut w, 5, 5);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.vision = 6;
        a.holdings[0] = held;
        a.delivery_rate = 1.0;
        (w, id)
    }

    // The controller's rulings: at home the agent buries its whole load
    // above R and counts that as the delivery, then takes its provisions
    // back from the larder; it leaves; away, it digs the larder only when
    // holdings fall below one tick's need.
    #[test]
    fn after_a_delivery_the_agent_leaves_and_digs_only_below_one_ticks_need() {
        let (mut w, id) = forager(20.0);
        w.agent_mut(id).unwrap().load_trip = 15.0;
        set_sugar(&mut w, 5, 8, 4.0); // 3 away
        let home = at(&w, 5, 5);
        turn(&mut w, id);
        // The whole load of 15 (≤ 20 − R) is delivered; then provisions for
        // 3 out and back and a tick, 1 × (2·3 + 2) = 8, come back from the
        // larder: 5 held + 3 dug.
        assert_eq!(w.events.deliveries, 1);
        assert_eq!(w.events.delivered, 15.0);
        assert_eq!((w.events.digs, w.events.dug), (1, 3.0), "provisions");
        assert_eq!(w.agent(id).unwrap().caches[&home], 12.0);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6), "it leaves");
        assert_eq!(w.agent(id).unwrap().leaving, Some(Pos::new(5, 8)));
        assert_eq!(w.agent(id).unwrap().holdings[0], 7.0, "8 − 1 eaten");
        assert_eq!(w.agent(id).unwrap().load_trip, 0.0);
        // ρ, in the unit of the load brought home: 1 + 0.05 × (15 − 1).
        assert!((w.agent(id).unwrap().delivery_rate - 1.7).abs() < 1e-12);
        // Its larder isn't a candidate while it holds a tick's need.
        let (c, _) = crate::rules::movement::candidates_with_memory(&w, id);
        let home_value = c.iter().find(|e| e.0 == Pos::new(5, 5)).unwrap().2;
        assert_eq!(home_value, 0.0, "home in sight, worth only its site");
        // Holding less than a tick's need, the larder joins its candidates
        // and, arriving home, it digs up to that need and no more.
        w.agent_mut(id).unwrap().holdings[0] = 0.25;
        w.agent_mut(id).unwrap().leaving = None;
        let (c, _) = crate::rules::movement::candidates_with_memory(&w, id);
        assert!(c.iter().any(|e| e.0 == Pos::new(5, 5) && e.2 == 12.0));
        let h = crate::rules::movement::go_and_gather(&mut w, id, Pos::new(5, 5));
        assert_eq!(h.dug, 0.75, "up to one tick's need");
        assert_eq!(w.agent(id).unwrap().caches[&home], 11.25);
    }

    #[test]
    fn short_of_provisions_at_home_it_digs_the_difference() {
        let (mut w, id) = forager(3.0);
        let home = at(&w, 5, 5);
        w.agent_mut(id).unwrap().caches.insert(home, 10.0);
        set_sugar(&mut w, 5, 7, 4.0); // 2 away: provisions 6
        turn(&mut w, id);
        assert_eq!(w.events.deliveries, 0);
        assert_eq!(w.events.dug, 3.0);
        assert_eq!(w.agent(id).unwrap().caches[&home], 7.0);
        assert_eq!(w.agent(id).unwrap().holdings[0], 5.0, "6 − 1 eaten");
    }

    #[test]
    fn an_endowment_is_not_a_load() {
        let (mut w, id) = forager(30.0);
        set_sugar(&mut w, 5, 7, 4.0);
        turn(&mut w, id);
        assert_eq!(w.events.deliveries, 0);
        assert!(w.agent(id).unwrap().caches.is_empty());
        assert_eq!(w.agent(id).unwrap().holdings[0], 29.0);
    }

    #[test]
    fn in_a_patch_it_harvests_while_the_best_neighbor_beats_rho_then_goes_home() {
        let (mut w, id) = forager(30.0);
        w.agent_mut(id).unwrap().pos = Pos::new(5, 5);
        // Walk it out to (5, 9) by hand: move it there directly.
        w.move_agent(id, Pos::new(5, 9));
        w.agent_mut(id).unwrap().delivery_rate = 2.0;
        set_sugar(&mut w, 5, 10, 3.0); // ≥ ρ
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 10));
        assert_eq!(w.agent(id).unwrap().load_trip, 3.0);
        assert_eq!(w.agent(id).unwrap().leaving, None);
        set_sugar(&mut w, 5, 11, 1.5); // < ρ (now 1.9)
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().leaving, Some(Pos::new(5, 5)));
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 9), "heading home");
        for _ in 0..4 {
            turn(&mut w, id);
        }
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
        assert_eq!(w.agent(id).unwrap().leaving, None);
        // Next tick it delivers its 3: provisions for the 1.5 it now sees
        // 6 away are 14, and it holds 30 + 3 − 6 eaten = 27.
        turn(&mut w, id);
        assert_eq!((w.events.deliveries, w.events.delivered), (1, 3.0));
    }

    #[test]
    fn a_full_load_goes_home_whatever_the_patch() {
        // The limit (40) caps the load, not load plus provisions: holding
        // 38 with a load of 36, it has room for 4.
        let (mut w, id) = forager(38.0);
        w.agent_mut(id).unwrap().load_trip = 36.0;
        w.move_agent(id, Pos::new(5, 9));
        w.agent_mut(id).unwrap().delivery_rate = 0.5;
        set_sugar(&mut w, 5, 9, 5.0);
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().holdings[0], 41.0, "38 + 4 room − 1");
        assert_eq!(w.agent(id).unwrap().load_trip, 40.0);
        assert_eq!(w.site(Pos::new(5, 9)).resource[0], 1.0);
        // A full load (40 + R ≥ 40): it goes home, however rich the next
        // site.
        set_sugar(&mut w, 5, 10, 9.0);
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().leaving, Some(Pos::new(5, 5)));
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 8));
    }

    #[test]
    fn nothing_known_away_from_home_stays_home_without_a_leave() {
        let (mut w, id) = forager(10.0);
        turn(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
        assert_eq!(w.agent(id).unwrap().leaving, None);
        assert_eq!(w.events.leaves, 0);
    }

    /// The staircase patch at distance `d` from home `h`: sites alternating
    /// between distance d and d + 1 around the diamond |x| + |y| = d,
    /// starting at (h.x + d, h.y) on home's row (in sight from home). Each is
    /// a lattice neighbor of the one before; a site at distance d borders
    /// only earlier and later patch sites and sites nearer home, so its best
    /// neighbor is always the next patch site.
    fn staircase(h: Pos, d: u32, len: usize) -> Vec<Pos> {
        let (hx, hy) = (i64::from(h.x), i64::from(h.y));
        let mut out = Vec::new();
        let (mut x, mut y) = (i64::from(d), 0i64);
        for k in 0..len {
            out.push(Pos::new((hx + x) as u32, (hy + y) as u32));
            // Counterclockwise: from a site on the diamond, a step out (to
            // d + 1); from one off it, a step back in (to d), turning at
            // each quadrant.
            let q = match (x, y) {
                (x, y) if x > 0 && y >= 0 => 0,
                (x, y) if x <= 0 && y > 0 => 1,
                (x, y) if x < 0 && y <= 0 => 2,
                _ => 3,
            };
            let (dx, dy) = if k % 2 == 0 {
                [(0, 1), (-1, 0), (0, -1), (1, 0)][q]
            } else {
                [(-1, 0), (0, -1), (1, 0), (0, 1)][q]
            };
            x += dx;
            y += dy;
        }
        out
    }

    /// The loading curve v₁ ≥ v₂ ≥ … (sites harvested nearest first, in
    /// staircase order), fixed before any run of the rule.
    const CURVE: [f64; 10] = [15.0, 12.0, 10.0, 8.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];

    /// Round-trip ticks for a load of the first n sites at distance d: d to
    /// walk out (the d-th tick harvests v₁), n − 1 ticks harvesting v₂…vₙ,
    /// then the walk home from sₙ — d when n is odd (sₙ is on the diamond),
    /// d + 1 when n is even — whose last tick is the arrival; the next
    /// tick's delivery is the first step out, so no tick is idle.
    fn round_trip(n: usize, d: u32) -> u32 {
        d + (n as u32 - 1) + if n % 2 == 1 { d } else { d + 1 }
    }

    /// The tangent construction, worked by maximizing G(n) / T(n) over n.
    fn optimum(d: u32) -> (usize, f64) {
        let mut best = (0, 0.0, f64::NEG_INFINITY);
        let mut g = 0.0;
        for n in 1..=CURVE.len() {
            g += CURVE[n - 1];
            let rate = g / f64::from(round_trip(n, d));
            if rate > best.2 {
                best = (n, g, rate);
            }
        }
        (best.0, best.1)
    }

    /// A staircase run's deliveries: each load and the tick it was made.
    struct Trips {
        loads: Vec<f64>,
        at: Vec<u64>,
        /// The agent's last GOAP plan's steps (under `goap`).
        last_plan: Option<Vec<(Pos, f64)>>,
        /// ρ at each decision to head home from the patch.
        rho_at_leave: Vec<f64>,
    }

    impl Trips {
        /// Mean load and delivered sugar per tick over deliveries `from..`.
        fn steady(&self, from: usize) -> (f64, f64) {
            let loads = &self.loads[from..];
            let sum: f64 = loads.iter().sum();
            let ticks = self.at[self.at.len() - 1] - self.at[from - 1];
            // The loads delivered after tick at[from − 1], over those ticks.
            (sum / loads.len() as f64, sum / ticks as f64)
        }
    }

    /// Runs one agent on the staircase at distance `d` for `trips`
    /// deliveries, restoring the patch whenever it's home (no regrowth
    /// during a visit). Under `mvt` it has metabolism 0, so provisions are 0
    /// and a load is all it brings back; under `goap`, metabolism 1 (G =
    /// `horizon`) and 1000 in hand, so provisions never touch the load.
    /// With `fixed_rho`, ρ is set to it before every tick (the textbook
    /// rule with the true long-run rate) instead of being learned.
    fn run_trips(
        d: u32,
        rule: DecisionRule,
        alpha: f64,
        horizon: u32,
        trips: usize,
        fixed_rho: Option<f64>,
    ) -> Trips {
        let size = 2 * d + 12;
        let mut w = central_world(size, rule, 2000);
        w.config.mvt.alpha = alpha;
        w.config.goap.horizon = horizon;
        let h = Pos::new(d + 4, d + 4);
        let id = spawn(&mut w, h.x, h.y);
        let a = w.agent_mut(id).unwrap();
        a.vision = d + 1;
        if rule == DecisionRule::Goap {
            a.metabolism[0] = 1;
            a.holdings[0] = 1000.0;
        }
        let patch = staircase(h, d, CURVE.len());
        for (i, p) in patch.iter().enumerate() {
            assert_eq!(lattice_distance(w.torus, h, *p), d + (i as u32 % 2));
        }
        let mut out = Trips {
            loads: Vec::new(),
            at: Vec::new(),
            last_plan: None,
            rho_at_leave: Vec::new(),
        };
        let mut ticks = 0;
        while out.loads.len() < trips {
            if w.agent(id).unwrap().pos == h {
                for (p, v) in patch.iter().zip(CURVE) {
                    set_sugar(&mut w, p.x, p.y, v);
                }
            }
            if let Some(rho) = fixed_rho {
                w.agent_mut(id).unwrap().delivery_rate = rho;
            }
            let a = w.agent(id).unwrap();
            let (rho, was) = (a.delivery_rate, a.leaving);
            turn(&mut w, id);
            if was != Some(h) && w.agent(id).unwrap().leaving == Some(h) {
                out.rho_at_leave.push(rho);
            }
            if w.events.deliveries > 0 {
                out.loads.push(w.events.delivered);
                out.at.push(w.tick);
                if let Some(g) = &w.agent(id).unwrap().goap_plan {
                    out.last_plan = Some(g.steps.clone());
                }
            }
            ticks += 1;
            assert!(ticks < 200 * trips, "no steady trips at d = {d}");
        }
        out
    }

    // The analytic test. Loading curve (stated, in harvest order, fixed
    // before any run of the rule): v = 15, 12, 10, 8, 6, 5, 4, 3, 2, 1, on a
    // staircase patch whose sites alternate between distance d and d + 1
    // from home (`staircase`). A load of the first n sites takes T(n) =
    // d + (n − 1) + r(n) ticks, r(n) = d for odd n and d + 1 for even n
    // (`round_trip`), so a round trip's time in the patch is T − 2d =
    // n − 1 + [n even]: an even n costs as much time as n + 1 and is never
    // optimal. The rate G(n) / T(n), G the cumulative gain:
    //
    //   n    G    d = 2          d = 5          d = 10
    //   1   15   15/4  = 3.750   15/10 = 1.500  15/20 = 0.750
    //   2   27   27/6  = 4.500   27/12 = 2.250  27/22 = 1.227
    //   3   37   37/6  = 6.167   37/12 = 3.083  37/22 = 1.682
    //   4   45   45/8  = 5.625   45/14 = 3.214  45/24 = 1.875
    //   5   51   51/8  = 6.375*  51/14 = 3.643  51/24 = 2.125
    //   6   56   56/10 = 5.600   56/16 = 3.500  56/26 = 2.154
    //   7   60   60/10 = 6.000   60/16 = 3.750* 60/26 = 2.308
    //   8   63   63/12 = 5.250   63/18 = 3.500  63/28 = 2.250
    //   9   65   65/12 = 5.417   65/18 = 3.611  65/28 = 2.321*
    //  10   66   66/14 = 4.714   66/20 = 3.300  66/30 = 2.200
    //
    // The optimal load (the tangent from −2d on the time axis to the
    // loading curve) is 51 (n = 5) at d = 2, 60 (n = 7) at d = 5 and 65
    // (n = 9) at d = 10: load rises with distance.
    //
    // The learned rule (α = 0.05, the default; mean load over deliveries
    // 100–399): 65 at d = 10 and 60 at d = 5, the optimum exactly; 49 at
    // d = 2, two short of 51, cycling 45, 51, 51 (it stops at n = 4 one
    // trip in three).
    //
    // Why: the comparison (one site's yield ≥ ρ) can't see that a site's
    // marginal cost depends on parity (d → d + 1 costs two ticks, d + 1 → d
    // none net). With ρ fixed at the optimal rate it gives 45, 60, 63
    // (`fixed_rho_misses_the_parity`). The learned ρ, an exponential
    // average of a pulse (the load on the delivery tick, 0 on every other),
    // has decayed below the long-run rate by the in-patch decisions, and
    // that underestimate cancels the parity error at d = 5 and 10. At d = 2
    // the stop at n = 4 is a dominated even load: v₅ = 6 would be free on
    // the way home, and the rule, comparing 6 against ρ ≈ 6, can't see it.
    //
    // The claim, asserted: within one site's value (vₙ₊₁ = 5, 3, 1) of the
    // optimum; load rising strictly with distance; and at d = 10 less than
    // the whole patch (66). The exact means are a regression pin, not the
    // claim.
    #[test]
    fn the_steady_load_against_the_tangent_construction() {
        assert_eq!(optimum(2), (5, 51.0));
        assert_eq!(optimum(5), (7, 60.0));
        assert_eq!(optimum(10), (9, 65.0));
        let mut means = Vec::new();
        for (d, measured) in [(2, 49.0), (5, 60.0), (10, 65.0)] {
            let trips = run_trips(d, DecisionRule::Mvt, 0.05, 10, 400, None);
            let (mean, rate) = trips.steady(100);
            let (n, best) = optimum(d);
            let best_rate = best / f64::from(round_trip(n, d));
            eprintln!(
                "d = {d}: optimum {best} (n = {n}, {best_rate:.3}/tick); \
                 rule's mean load {mean:.3} ({rate:.3}/tick), last {:?}",
                &trips.loads[394..]
            );
            assert!(
                (mean - best).abs() <= CURVE[n],
                "d = {d}: mean load {mean} vs optimum {best}"
            );
            assert!(rate <= best_rate + 1e-9, "nothing beats the optimum");
            // Regression pin (the measured means), not the claim.
            assert!((mean - measured).abs() < 1e-9, "d = {d}: {mean}");
            means.push(mean);
        }
        assert!(
            means[0] < means[1] && means[1] < means[2],
            "load rises with distance: {means:?}"
        );
        assert!(means[2] < 66.0, "not the whole patch at d = 10");
    }

    // The finding behind the analytic test: with ρ held at the true optimal
    // rate (the textbook leaving rule), the per-site comparison gives 45,
    // 60 and 63 at d = 2, 5 and 10, two of them dominated even loads; and
    // the learned ρ at the decision to leave sits below that rate. Printed
    // and asserted (`cargo test --release -p sugarscape-core --lib
    // fixed_rho -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn fixed_rho_misses_the_parity() {
        for (d, expected) in [(2, 45.0), (5, 60.0), (10, 63.0)] {
            let (n, best) = optimum(d);
            let best_rate = best / f64::from(round_trip(n, d));
            let fixed = run_trips(d, DecisionRule::Mvt, 0.05, 10, 50, Some(best_rate));
            let (mean, _) = fixed.steady(10);
            let learned = run_trips(d, DecisionRule::Mvt, 0.05, 10, 400, None);
            let rho = &learned.rho_at_leave[100..];
            let rho_mean = rho.iter().sum::<f64>() / rho.len() as f64;
            eprintln!(
                "d = {d}: optimum {best} at {best_rate:.3}/tick; fixed-ρ load {mean:.3}; \
                 learned ρ at leaving {rho_mean:.3}"
            );
            assert_eq!(mean, expected, "d = {d}");
            assert!(rho_mean < best_rate, "d = {d}: {rho_mean}");
        }
    }

    // The report's sensitivity to α: larger α makes ρ spikier. Printed, not
    // judged (`cargo test --release -p sugarscape-core --lib
    // alpha_sweep -- --ignored --nocapture`).
    #[test]
    #[ignore]
    fn alpha_sweep_for_the_report() {
        for alpha in [0.01, 0.05, 0.1, 0.2, 0.5, 1.0] {
            for d in [2, 5, 10] {
                let trips = run_trips(d, DecisionRule::Mvt, alpha, 10, 400, None);
                let (mean, rate) = trips.steady(100);
                let (_, best) = optimum(d);
                eprintln!(
                    "alpha {alpha} d {d}: optimum {best}, mean load {mean:.3}, rate {rate:.3}"
                );
            }
        }
    }

    // GOAP's "deliver G" (report only): the plan ends at home, and it
    // delivers. Its loads at each d are printed for the report.
    #[test]
    fn goap_plans_end_at_home_and_deliver() {
        for (horizon, d) in [(10, 2), (10, 5), (10, 10), (20, 2)] {
            let trips = run_trips(d, DecisionRule::Goap, 0.05, horizon, 30, None);
            let (mean, rate) = trips.steady(10);
            eprintln!(
                "goap G = {horizon}, d = {d}: mean load {mean:.3} ({rate:.3}/tick), last {:?}, plan {:?}",
                &trips.loads[25..],
                trips.last_plan
            );
            assert!(mean >= f64::from(horizon), "a delivery is at least G");
        }
        // G = 20 at d = 5: from home it sees only v₁ = 15 (the rest of the
        // patch is off its axes), so no plan reaches G and it stays home.
        let mut w = central_world(22, DecisionRule::Goap, 2000);
        w.config.goap.horizon = 20;
        let h = Pos::new(9, 9);
        let id = spawn(&mut w, h.x, h.y);
        let a = w.agent_mut(id).unwrap();
        (a.metabolism[0], a.vision) = (1, 6);
        for (p, v) in staircase(h, 5, CURVE.len()).iter().zip(CURVE) {
            set_sugar(&mut w, p.x, p.y, v);
        }
        turn(&mut w, id);
        assert_eq!(w.events.fallback_short, 1);
        assert_eq!(w.agent(id).unwrap().pos, h);
        // The plan's last step is home.
        let mut w = central_world(21, DecisionRule::Goap, 100);
        w.config.goap.horizon = 5;
        let id = spawn(&mut w, 5, 5);
        let a = w.agent_mut(id).unwrap();
        a.metabolism[0] = 1;
        a.vision = 6;
        set_sugar(&mut w, 5, 8, 3.0);
        set_sugar(&mut w, 8, 5, 4.0);
        turn(&mut w, id);
        let steps = &w.agent(id).unwrap().goap_plan.as_ref().unwrap().steps;
        assert_eq!(steps.last().unwrap().0, Pos::new(5, 5));
        assert_eq!(
            steps.len(),
            3,
            "one site stepped toward, one more, then home"
        );
    }

    /// Five agents with metabolism 1–2 foraging from homes on a regrowing
    /// landscape for 400 ticks: sites + holdings + caches + eaten equals the
    /// start plus growback exactly (to rounding in the running sums), nobody
    /// starves on a trip (provisions), and loads are delivered.
    #[test]
    fn central_foragers_conserve_sugar_and_survive_their_trips() {
        let mut c = central_config(16, DecisionRule::Mvt, 30);
        c.growback.rate = 1.0;
        let mut w = World::new(c, 5).unwrap();
        for y in 0..16 {
            for x in 0..16 {
                let v = if (x / 4 + y / 4) % 2 == 0 { 4.0 } else { 0.0 };
                w.sites[w.torus.index(Pos::new(x, y))].capacity[0] = v;
                set_sugar(&mut w, x, y, v);
            }
        }
        let mut ids = Vec::new();
        for (i, (x, y)) in [(1, 5), (6, 1), (9, 13), (13, 9), (5, 10)]
            .into_iter()
            .enumerate()
        {
            let id = spawn(&mut w, x, y);
            let a = w.agent_mut(id).unwrap();
            a.metabolism[0] = 1 + i as u32 % 2;
            a.vision = 3;
            a.holdings[0] = 8.0;
            a.delivery_rate = f64::from(a.metabolism[0]);
            ids.push(id);
        }
        let total = |w: &World| {
            let sites: f64 = w.sites.iter().map(|s| s.resource[0]).sum();
            let held: f64 = w.agents().map(|a| a.holdings[0]).sum();
            let cached: f64 = w.agents().flat_map(|a| a.caches.values()).sum();
            sites + held + cached
        };
        let start = total(&w);
        let (mut eaten, mut grown, mut deliveries) = (0.0, 0.0, 0);
        for _ in 0..400 {
            w.events = crate::world::TickEvents::default();
            for &id in &ids {
                eaten += f64::from(w.agent(id).unwrap().metabolism[0]);
                crate::rules::agent_turn(&mut w, id);
                assert!(w.agent(id).is_some(), "starved at tick {}", w.tick);
            }
            deliveries += w.events.deliveries;
            let before: f64 = w.sites.iter().map(|s| s.resource[0]).sum();
            crate::rules::growback::apply(&mut w);
            grown += w.sites.iter().map(|s| s.resource[0]).sum::<f64>() - before;
            w.tick += 1;
            let (lhs, rhs) = (total(&w) + eaten, start + grown);
            assert!((lhs - rhs).abs() <= 1e-9 * rhs, "{lhs} vs {rhs}");
            assert!(w.agents().all(|a| a.holdings[0] <= 30.0));
        }
        eprintln!("{deliveries} deliveries");
        assert!(deliveries > 20, "{deliveries} deliveries");
    }
}
