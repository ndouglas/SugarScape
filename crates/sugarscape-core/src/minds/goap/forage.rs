//! Minds 4's foraging domain and the GOAP decision rule
//! (`decision.rule: goap`): a Flump plans the fastest way to gather enough.
//!
//! - **Candidates.** Rule M's list (`candidates_with_memory`: its own site,
//!   the free sites in sight and, for a rememberer, remembered sites at
//!   believed values). The best `goap.k` other than its own site, by value
//!   descending, then distance ascending, then site index, plus its own site
//!   (slot 0; the others are slots 1..=K).
//! - **State.** `(slot the Flump is at, mask of slots harvested)`. The sugar
//!   gathered is derived from the mask, never stored, so there are at most
//!   (K + 1)·2^(K + 1) states.
//! - **`Harvest(i)`.** Walk to unharvested slot i and harvest it: the torus
//!   Manhattan distance + 1 ticks (1 in place). Regrowth during a plan, and
//!   the sugar picked up on sites walked over, are ignored (stated).
//! - **Goal.** Gathered ≥ G = metabolism × `goap.horizon`. Heuristic
//!   ⌈(G − gathered) / v_max⌉ over the unharvested values: every action
//!   costs at least 1 tick and gains at most v_max, so it's admissible, and
//!   it's consistent (after an action gaining v ≤ v_max it drops by at most
//!   1). Infinite when nothing unharvested has value.
//! - **Ties.** Actions are generated best value first (then nearest, then
//!   site index), and among equal-cost plans A*'s deterministic tie order
//!   decides (the controller's ruling: no search per first action, and the
//!   planner draws nothing). `choose` breaks ties only in the fallback.

use crate::agent::{AgentId, GoapPlan};
use crate::geometry::{Pos, Torus};
use crate::rules::movement::{
    arrive, candidates_with_memory, choose, lattice_distance, record_choice,
};
use crate::rules::Harvest;
use crate::world::World;

use super::{plan, Domain};

/// Most states one plan may expand; past it the Flump takes the fallback.
/// The planner counts a goal state's step to its sentinel as an expansion,
/// so this is 4 095 foraging states plus that step.
pub const PLAN_LIMIT: usize = 4096;

/// The foraging domain over one decision's slots (0 the Flump's own site).
pub(crate) struct Forage {
    values: Vec<f64>,
    /// Travel ticks between slots, row-major `n × n`.
    dist: Vec<u32>,
    /// Slots in action order: value descending, then distance from slot 0
    /// ascending, then site index.
    order: Vec<u8>,
    goal: f64,
}

impl Forage {
    /// `sites[0]` is the Flump's own site; each entry is a site and its
    /// value. At most 16 slots.
    pub(crate) fn new(torus: Torus, sites: &[(Pos, f64)], goal: f64) -> Self {
        let n = sites.len();
        assert!((1..=16).contains(&n), "1 to 16 slots");
        let raw = |i: usize, j: usize| lattice_distance(torus, sites[i].0, sites[j].0);
        let mut dist = vec![0; n * n];
        for i in 0..n {
            for j in 0..n {
                dist[i * n + j] = travel(raw(i, j));
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
        Forage {
            values: sites.iter().map(|s| s.1).collect(),
            dist,
            order,
            goal,
        }
    }

    /// Sugar gathered by the slots in `mask`, summed in slot order.
    fn gathered(&self, mask: u16) -> f64 {
        (0..self.values.len())
            .filter(|&i| mask & (1 << i) != 0)
            .map(|i| self.values[i])
            .sum()
    }
}

impl Domain for Forage {
    type State = (u8, u16);
    type Action = u8;

    fn actions(&self, &(at, mask): &(u8, u16), out: &mut Vec<(u8, f64, (u8, u16))>) {
        let n = self.values.len();
        for &i in &self.order {
            if mask & (1 << i) == 0 {
                let cost = self.dist[usize::from(at) * n + usize::from(i)] + 1;
                out.push((i, f64::from(cost), (i, mask | 1 << i)));
            }
        }
    }

    fn is_goal(&self, &(_, mask): &(u8, u16)) -> bool {
        self.gathered(mask) >= self.goal
    }

    fn heuristic(&self, &(_, mask): &(u8, u16)) -> f64 {
        let g = self.gathered(mask);
        if g >= self.goal {
            return 0.0;
        }
        let v_max = (0..self.values.len())
            .filter(|&i| mask & (1 << i) == 0)
            .map(|i| self.values[i])
            .fold(0.0, f64::max);
        if v_max > 0.0 {
            ((self.goal - g) / v_max).ceil()
        } else {
            f64::INFINITY
        }
    }
}

/// G: the Flump's burn per tick (its effective sugar metabolism; with n ≥ 2
/// goods, the mean over the goods) × `goap.horizon`.
fn goal_of(world: &World, id: AgentId) -> f64 {
    let a = world.agent(id).expect("live agent");
    let n = world.config.goods.len();
    let mets = a.effective_metabolisms(n, world.config.disease.active_fee());
    let burn = mets[..n].iter().sum::<f64>() / n as f64;
    burn * f64::from(world.config.goap.horizon)
}

/// The next target of the Flump's plan if it still holds, dropping the
/// plan's steps otherwise. It holds while the target is still one of the
/// Flump's candidates (so, if it's in sight, it's unoccupied), is worth at
/// least half its planned value when in sight, isn't walled off, and isn't
/// where the last walk found no path.
fn next_target(
    world: &mut World,
    id: AgentId,
    candidates: &[(Pos, u32, f64)],
    start: usize,
) -> Option<Pos> {
    let a = world.agent(id).expect("live agent");
    let &(t, planned) = a.goap_plan.as_ref()?.steps.first()?;
    let holds = candidates.iter().position(|c| c.0 == t).is_some_and(|i| {
        (i >= start || candidates[i].2 >= planned / 2.0)
            && !world.walled_apart(a.pos, t)
            && !(a.plan.target == Some(t) && a.plan.path.is_empty() && a.pos != t)
    });
    if !holds {
        let a = world.agent_mut(id).expect("live agent");
        if let Some(g) = a.goap_plan.as_mut() {
            g.steps.clear();
        }
    }
    holds.then_some(t)
}

/// Heads for `target` (counted as Minds 3 counts a choice), and pops it
/// from the plan on arrival.
fn go(
    world: &mut World,
    id: AgentId,
    candidates: &[(Pos, u32, f64)],
    start: usize,
    target: Pos,
) -> Harvest {
    record_choice(world, id, candidates, start, target);
    let harvest = arrive(world, id, target);
    let a = world.agent_mut(id).expect("live agent");
    if a.pos == target {
        if let Some(g) = a.goap_plan.as_mut() {
            if g.steps.first().is_some_and(|s| s.0 == target) {
                g.steps.remove(0);
            }
        }
    }
    harvest
}

/// Rule M's step under GOAP: follow the plan while its next target holds;
/// otherwise plan (at most `PLAN_LIMIT` expansions). An empty plan (G = 0)
/// stays. With no plan (not enough known sugar for G, or over the limit)
/// the Flump takes the rate choice: the candidates of best value ÷
/// (distance + 1), passed to `choose` (rule M's tie rule and draw).
pub(crate) fn act(world: &mut World, id: AgentId) -> Harvest {
    let (candidates, start) = candidates_with_memory(world, id);
    if let Some(t) = next_target(world, id, &candidates, start) {
        return go(world, id, &candidates, start, t);
    }
    let goal = goal_of(world, id);
    let torus = world.torus;
    let mut others: Vec<usize> = (1..candidates.len()).collect();
    others.sort_by(|&i, &j| {
        let (a, b) = (&candidates[i], &candidates[j]);
        b.2.total_cmp(&a.2)
            .then(a.1.cmp(&b.1))
            .then(torus.index(a.0).cmp(&torus.index(b.0)))
    });
    others.truncate(world.config.goap.k as usize);
    let slots: Vec<usize> = std::iter::once(0).chain(others).collect();
    let sites: Vec<(Pos, f64)> = slots
        .iter()
        .map(|&i| (candidates[i].0, candidates[i].2))
        .collect();
    let domain = Forage::new(torus, &sites, goal);
    // Every slot harvested is the most any plan gathers: short of G, no
    // plan exists, so don't search.
    let found = if domain.is_goal(&(0, (1u16 << sites.len()) - 1)) {
        plan(&domain, (0, 0), PLAN_LIMIT)
    } else {
        None
    };
    match found {
        Some(p) => {
            let steps: Vec<(Pos, f64)> = p.actions.iter().map(|&s| sites[usize::from(s)]).collect();
            if !steps.is_empty() {
                let e = &mut world.events;
                e.plans += 1;
                e.plan_steps_sum += steps.len() as u32;
                if p.actions.iter().any(|&s| slots[usize::from(s)] >= start) {
                    e.plans_with_remembered += 1;
                }
            }
            let target = steps.first().map_or(candidates[0].0, |s| s.0);
            world.agent_mut(id).expect("live agent").goap_plan = Some(GoapPlan {
                gathers: steps.iter().map(|s| s.1).sum(),
                steps,
                goal,
            });
            go(world, id, &candidates, start, target)
        }
        None => {
            let rate = |c: &(Pos, u32, f64)| c.2 / (f64::from(c.1) + 1.0);
            let best = candidates
                .iter()
                .map(rate)
                .fold(f64::NEG_INFINITY, f64::max);
            let top: Vec<(Pos, u32, f64)> = candidates
                .iter()
                .filter(|c| rate(c) == best)
                .copied()
                .collect();
            let target = choose(&top, &mut world.rng);
            go(world, id, &candidates, start, target)
        }
    }
}

/// Travel ticks for a lattice distance: the distance itself, or 0 under the
/// test-only hook that zeroes travel (the reduction to rule M).
fn travel(d: u32) -> u32 {
    #[cfg(test)]
    if ZERO_TRAVEL.with(std::cell::Cell::get) {
        return 0;
    }
    d
}

#[cfg(test)]
thread_local! {
    static ZERO_TRAVEL: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::super::{plan, Domain, Plan};
    use super::*;
    use crate::config::{DecisionRule, Goap, MoveMode, Movement};
    use crate::rules::movement::candidates_with_memory;
    use crate::testkit::*;

    /// A walking GOAP world (speed 1) with K = `k` and H = `horizon`.
    fn goap_world(size: u32, k: u32, horizon: u32) -> World {
        let mut c = blank_config(size, size);
        c.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        c.decision.rule = DecisionRule::Goap;
        c.goap = Goap { k, horizon };
        World::new(c, 7).unwrap()
    }

    /// A Flump at (x, y) with `vision` and sugar metabolism `metabolism`.
    fn forager(w: &mut World, x: u32, y: u32, vision: u32, metabolism: u32) -> AgentId {
        let id = spawn(w, x, y);
        let a = w.agent_mut(id).unwrap();
        a.vision = vision;
        a.metabolism[0] = metabolism;
        id
    }

    fn steps(w: &World, id: AgentId) -> Vec<Pos> {
        w.agent(id)
            .unwrap()
            .goap_plan
            .as_ref()
            .map(|g| g.steps.iter().map(|s| s.0).collect())
            .unwrap_or_default()
    }

    fn target(w: &World, id: AgentId) -> Option<Pos> {
        w.agent(id).unwrap().plan.target
    }

    #[test]
    fn two_near_sites_beat_one_far_rich_site() {
        // G = 1 × 4. Two near sites of 2 cost 2 + 3 = 5 ticks; the far 6
        // costs 7. Rule M would head for the 6.
        let mut w = goap_world(21, 8, 4);
        let id = forager(&mut w, 5, 5, 6, 1);
        set_sugar(&mut w, 5, 6, 2.0);
        set_sugar(&mut w, 5, 4, 2.0);
        set_sugar(&mut w, 5, 11, 6.0);
        act(&mut w, id);
        // Equal value and distance: the lower site index, (5, 4), first.
        assert_eq!(steps(&w, id), vec![Pos::new(5, 6)], "(5, 4) reached");
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 4));
        let g = w.agent(id).unwrap().goap_plan.clone().unwrap();
        assert_eq!((g.gathers, g.goal), (4.0, 4.0));
        assert_eq!((w.events.plans, w.events.plan_steps_sum), (1, 2));
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
        assert_eq!(target(&w, id), Some(Pos::new(5, 6)));
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6));
        assert!(steps(&w, id).is_empty(), "finished");
        assert_eq!(w.events.plans, 1, "followed, not replanned");
    }

    #[test]
    fn a_far_rich_patch_beats_near_scraps_when_the_goal_is_large() {
        // G = 1 × 10. Four scraps of 1 next door can't reach it; the patch
        // 6 + 6 at distance 6 and 7 costs 7 + 2 = 9 ticks.
        let mut w = goap_world(21, 8, 10);
        let id = forager(&mut w, 5, 5, 7, 1);
        for (x, y) in [(5, 6), (5, 4), (4, 5), (6, 5)] {
            set_sugar(&mut w, x, y, 1.0);
        }
        set_sugar(&mut w, 5, 11, 6.0);
        set_sugar(&mut w, 5, 12, 6.0);
        act(&mut w, id);
        assert_eq!(target(&w, id), Some(Pos::new(5, 11)));
        assert_eq!(steps(&w, id), vec![Pos::new(5, 11), Pos::new(5, 12)]);
    }

    /// A Flump at (5, 5), G = 3, planning for (5, 9) (3, distance 4) over
    /// (5, 0) (3, distance 5), both in sight (vision 8); after one act it stands on (5, 6).
    fn heading_for_5_9() -> (World, AgentId) {
        let mut w = goap_world(21, 8, 3);
        let id = forager(&mut w, 5, 5, 8, 1);
        set_sugar(&mut w, 5, 9, 3.0);
        set_sugar(&mut w, 5, 0, 3.0);
        act(&mut w, id);
        assert_eq!(steps(&w, id), vec![Pos::new(5, 9)]);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 6));
        assert_eq!(w.events.plans, 1);
        (w, id)
    }

    #[test]
    fn a_taken_target_replans_that_tick() {
        let (mut w, id) = heading_for_5_9();
        spawn(&mut w, 5, 9);
        act(&mut w, id);
        assert_eq!(w.events.plans, 2, "replanned");
        assert_eq!(target(&w, id), Some(Pos::new(5, 0)));
        assert_eq!(steps(&w, id), vec![Pos::new(5, 0)]);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5), "no stale step");
    }

    #[test]
    fn an_emptied_target_replans_but_half_its_value_holds() {
        let (mut w, id) = heading_for_5_9();
        set_sugar(&mut w, 5, 9, 1.5); // exactly half: kept
        act(&mut w, id);
        assert_eq!((w.events.plans, target(&w, id)), (1, Some(Pos::new(5, 9))));
        set_sugar(&mut w, 5, 9, 1.0); // below half: dropped
        act(&mut w, id);
        assert_eq!(w.events.plans, 2);
        assert_eq!(target(&w, id), Some(Pos::new(5, 0)));
    }

    #[test]
    fn a_target_the_last_walk_couldnt_reach_replans() {
        let mut w = goap_world(21, 8, 3);
        let id = forager(&mut w, 5, 5, 6, 1);
        set_sugar(&mut w, 5, 9, 3.0);
        for (x, y) in [(5, 4), (5, 6), (4, 5), (6, 5)] {
            spawn(&mut w, x, y); // boxed in
        }
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5), "no path: stays");
        assert_eq!(w.events.plans, 1);
        act(&mut w, id);
        assert_eq!(w.events.plans, 2, "the failed walk invalidates the plan");
    }

    #[test]
    fn no_known_sugar_stays_without_planning_or_panicking() {
        let mut w = goap_world(21, 12, 100);
        let id = forager(&mut w, 5, 5, 6, 4);
        for _ in 0..5 {
            act(&mut w, id);
            assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
        }
        assert_eq!(w.events.plans, 0);
        // Some sugar, but never enough for G: the rate choice, not a plan.
        set_sugar(&mut w, 5, 8, 2.0); // rate 2 / 4
        set_sugar(&mut w, 6, 5, 1.0); // rate 1 / 2
        set_sugar(&mut w, 5, 4, 0.5); // rate 0.5 / 2
        act(&mut w, id);
        assert_eq!(w.events.plans, 0);
        let t = target(&w, id).unwrap();
        assert!(t == Pos::new(5, 8) || t == Pos::new(6, 5));
        assert_eq!(t, Pos::new(5, 8), "equal rates: the higher value");
    }

    #[test]
    fn metabolism_zero_is_already_at_its_goal_and_stays() {
        let mut w = goap_world(21, 8, 10);
        let id = forager(&mut w, 5, 5, 6, 0);
        set_sugar(&mut w, 5, 8, 4.0);
        act(&mut w, id);
        assert_eq!(w.agent(id).unwrap().pos, Pos::new(5, 5));
        assert_eq!(w.events.plans, 0, "an empty plan isn't counted");
        let g = w.agent(id).unwrap().goap_plan.clone().unwrap();
        assert_eq!((g.steps.len(), g.goal), (0, 0.0));
    }

    #[test]
    fn goap_plan_is_not_hashed() {
        let mut w = goap_world(9, 8, 10);
        let id = spawn(&mut w, 1, 1);
        let before = w.fingerprint();
        w.agent_mut(id).unwrap().goap_plan = Some(crate::agent::GoapPlan {
            steps: vec![(Pos::new(2, 2), 3.0)],
            gathers: 3.0,
            goal: 2.0,
        });
        assert_eq!(w.fingerprint(), before);
    }

    #[test]
    fn inspect_shows_the_plan_only_under_goap() {
        let mut w = goap_world(21, 8, 10);
        let id = forager(&mut w, 5, 5, 7, 1);
        set_sugar(&mut w, 5, 11, 6.0);
        set_sugar(&mut w, 5, 12, 6.0);
        act(&mut w, id);
        let pos = w.agent(id).unwrap().pos;
        let v = w
            .inspect(pos.x, pos.y)
            .unwrap()
            .agent
            .unwrap()
            .goap
            .unwrap();
        assert_eq!(v.steps, vec![[5, 11], [5, 12]]);
        assert_eq!((v.gathers, v.goal), (12.0, 10.0));
        w.config.decision.rule = DecisionRule::Book;
        assert!(w
            .inspect(pos.x, pos.y)
            .unwrap()
            .agent
            .unwrap()
            .goap
            .is_none());
    }

    #[test]
    fn a_plan_through_a_remembered_site_counts_as_using_memory() {
        let mut c = blank_config(21, 21);
        c.movement = Movement {
            mode: MoveMode::Walk,
            speed: 1,
        };
        c.decision.rule = DecisionRule::Goap;
        c.goap = Goap { k: 8, horizon: 4 };
        c.memory.span = 100;
        c.memory.share = 1.0;
        let mut w = World::new(c, 7).unwrap();
        let id = forager(&mut w, 5, 5, 1, 1);
        w.agent_mut(id).unwrap().remembers = true;
        set_sugar(&mut w, 5, 9, 4.0);
        let idx = w.torus.index(Pos::new(5, 9)) as u32;
        let seen = crate::minds::memory::Seen::new(&[4.0], &[4.0], 0);
        w.agent_mut(id).unwrap().memory.sites.insert(idx, seen);
        w.tick = 1;
        act(&mut w, id);
        assert_eq!(target(&w, id), Some(Pos::new(5, 9)));
        let e = &w.events;
        assert_eq!((e.plans, e.plans_with_remembered), (1, 1));
        assert_eq!((e.moves, e.remembered_moves), (1, 1));
    }

    /// The domain with no heuristic: A* becomes Dijkstra.
    struct Blind<'a>(&'a Forage);
    impl Domain for Blind<'_> {
        type State = (u8, u16);
        type Action = u8;
        fn actions(&self, s: &(u8, u16), out: &mut Vec<(u8, f64, (u8, u16))>) {
            self.0.actions(s, out)
        }
        fn is_goal(&self, s: &(u8, u16)) -> bool {
            self.0.is_goal(s)
        }
        fn heuristic(&self, _s: &(u8, u16)) -> f64 {
            0.0
        }
    }

    #[test]
    fn the_heuristic_is_admissible_against_an_uninformed_search() {
        let torus = Torus::new(15, 15);
        let mut found = 0;
        for seed in 0..400u64 {
            let mut rng = crate::rng::seeded(seed);
            let k = rng.gen_range(1..=6);
            let mut sites = vec![(Pos::new(7, 7), f64::from(rng.gen_range(0..=3u32)))];
            while sites.len() < k + 1 {
                let p = Pos::new(rng.gen_range(0..15), rng.gen_range(0..15));
                if sites.iter().all(|s| s.0 != p) {
                    sites.push((p, rng.gen_range(0.0..5.0)));
                }
            }
            let goal = rng.gen_range(0.0..15.0);
            let d = Forage::new(torus, &sites, goal);
            let a = plan(&d, (0, 0), usize::MAX);
            let b = plan(&Blind(&d), (0, 0), usize::MAX);
            assert_eq!(
                a.as_ref().map(|p| p.cost),
                b.as_ref().map(|p| p.cost),
                "seed {seed}"
            );
            found += usize::from(a.is_some());
        }
        assert!(found > 200, "most instances reach the goal: {found}");
    }

    #[test]
    fn harvesting_in_place_costs_one_and_moving_costs_the_distance_plus_one() {
        let torus = Torus::new(15, 15);
        let sites = [
            (Pos::new(7, 7), 1.0),
            (Pos::new(7, 10), 5.0),
            (Pos::new(14, 7), 2.0), // 7 across, not 8: the torus wraps
        ];
        let d = Forage::new(torus, &sites, 100.0);
        let mut out = Vec::new();
        d.actions(&(0, 0), &mut out);
        // Best value first: 5, 2, then 1.
        assert_eq!(
            out,
            vec![
                (1, 4.0, (1, 0b010)),
                (2, 8.0, (2, 0b100)),
                (0, 1.0, (0, 0b001))
            ]
        );
        let p: Plan<u8> = plan(&Forage::new(torus, &sites, 6.0), (0, 0), usize::MAX).unwrap();
        assert_eq!((p.actions, p.cost), (vec![0, 1], 5.0));
    }

    #[test]
    fn with_no_travel_cost_and_a_one_tick_goal_goap_chooses_as_rule_m_does() {
        let mut moved = 0;
        for seed in 0..200u64 {
            let mut rng = crate::rng::seeded(seed);
            let vision = rng.gen_range(1..=3);
            let mut w = goap_world(15, 12, 1);
            let id = forager(&mut w, 7, 7, vision, 1);
            set_sugar(&mut w, 7, 7, f64::from(rng.gen_range(0..=4u32)));
            for (q, _) in w.sight(Pos::new(7, 7), vision) {
                if rng.gen_bool(0.15) {
                    spawn(&mut w, q.x, q.y);
                } else {
                    set_sugar(&mut w, q.x, q.y, f64::from(rng.gen_range(0..=4u32)));
                }
            }
            let (c, _) = candidates_with_memory(&w, id);
            assert!(c.len() <= 13, "K covers every candidate");
            ZERO_TRAVEL.with(|z| z.set(true));
            act(&mut w, id);
            ZERO_TRAVEL.with(|z| z.set(false));
            let t = target(&w, id).unwrap();
            let best = c.iter().map(|e| e.2).fold(f64::NEG_INFINITY, f64::max);
            let nearest = c.iter().filter(|e| e.2 == best).map(|e| e.1).min();
            let chosen = c.iter().find(|e| e.0 == t).unwrap();
            assert_eq!((chosen.2, Some(chosen.1)), (best, nearest), "seed {seed}");
            moved += usize::from(t != Pos::new(7, 7));
        }
        assert!(moved > 100, "most neighborhoods send it somewhere: {moved}");
    }
}
