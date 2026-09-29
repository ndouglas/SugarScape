//! Minds 5: the labs — Raby et al.'s (2007) "planning for breakfast" and
//! Amodio et al.'s (2021) Experiment 2, run as a world mode.
//!
//! With `config.lab` set, the world applies the protocol's schedule itself
//! at the start of every tick ([`apply`], called from `World::step` before
//! anyone moves): the agents' placement each morning, each compartment's
//! food, the doorways, and the test evening's burying. It draws nothing:
//! the only draws in a lab world are the world's own (placement at build,
//! the turn order, rule M's tie-breaks) and, in [`run_population`], each
//! agent's (share, λ) through the world's RNG. [`run_lab`] and
//! [`run_population`] build the config ([`rig_config`]), step the world
//! through the schedule and read the caches, so the tests, the survey and
//! the page's presets run one code path.
//!
//! **The rig** (13 × 8, walled all round, so the torus never wraps):
//!
//! ```text
//!      x 0         1
//!        0123456789012
//!   y 0  #############
//!     1  #...#...#...#   K1 = x 1–3, K2 = x 5–7, K3 = x 9–11, rows 1–3
//!     2  #...#...#...#
//!     3  #.T.#.T.#.T.#   T: each compartment's tray, (2, 3), (6, 3), (10, 3)
//!     4  ##D###D###D##   D: the doorways (2, 4), (6, 4), (10, 4)
//!     5  #...........#   the hall's corridor
//!     6  #PPPPPPPPPPP#   the hall's perches, (1 + k, 6) for the k-th agent
//!     7  #############
//! ```
//!
//! All walls are opaque. The doorways are walls in the config (shut) and
//! the lab opens them on the test evening with `World::open_wall`: all
//! three for Amodio, K1's and K3's for Raby (K2 stays shut; Raby's birds
//! had caching trays only in A and C). Sugar is the one good, on a flat
//! zero map; the lab agents burn nothing (metabolism 0: the housed birds'
//! maintenance diet is outside the protocol), see 1, walk, and never age.
//! At most [`MAX_AGENTS`] agents (a compartment's 9 sites).
//!
//! **A day** is [`DAY`] = 4 ticks: a morning of [`MORNING`] = 2 ticks
//! (ticks 4d and 4d + 1), then an evening of 2. Day d (0-based) of the
//! [`training_days`] (6 for Raby, 9 for Amodio):
//!
//! - **Morning** (start of tick 4d): every compartment's sugar is cleared;
//!   if the day has food, each site of the day's compartment gets 1 unit
//!   (capacity 1); the k-th agent (in id order) is carried to the
//!   compartment's k-th site, row-major, its walk plan cleared; each agent
//!   records the day's episode (place = compartment 0, 1, 2 for K1, K2, K3;
//!   food = false). Standing on food with nothing better in sight, rule M
//!   keeps it and harvests; [`found`] (from the caching step, after the
//!   harvest) marks the episode's food true when it gathers any sugar.
//! - **Evening** (start of tick 4d + 2): the compartment is cleared; under
//!   `compensate`, the day's compartment's weight updates from the
//!   episode, w ← w × (1 − λ) if it found food (the weights are keyed by
//!   **compartment id**, 0–2, in the lab, never by site); the agents are
//!   carried back to their perches.
//! - **Schedules.** Raby: K1 on even d, K3 on odd d; breakfast on K1's
//!   mornings when `food_first` ("breakfast first") and on K3's otherwise,
//!   so `food_first` picks both which compartment has breakfast and whether
//!   the first morning had it. Amodio: K1, K2, K3, K1, …; food on odd days
//!   (d even) for the Food-First group, on even days for Empty-First.
//! - **The test evening** (start of tick 4N, N the training days): the
//!   doorways open; each agent is given [`F`] = 30 sugar and computes its
//!   allocation of exactly F over the protocol's caching compartments
//!   ([`allocation`], by `rules::allocate_*` with `whole`: remainders to
//!   K1 < K2 < K3). The reserve is 0 in the lab, so nothing is dug. One
//!   agent at a time (in id order, as the birds were tested alone), it
//!   walks from its perch one step a tick (A* over the free sites) to each
//!   compartment's tray in K order, buries that compartment's share there,
//!   and walks back to its perch; then the next agent goes. Nothing else
//!   moves: with no sugar in sight rule M keeps every agent where it is.
//!
//! Rule `plan` predicts with the cycle finder (`Episodes::predict`) for
//! days 1..=`caching.lookahead` past the last training day, and splits F
//! equally over the caching compartments it predicts will lack food (each
//! counted once); predicting nothing, it buries nothing.

use super::episodes::{Episode, Episodes};
use super::rules::{allocate_compensate, allocate_even, allocate_plan, rule_of};
use crate::agent::{AgentId, Plan};
use crate::config::{
    CachingRule, Config, Good, Lab, LabProtocol, Map, MoveMode, Movement, Placement, URange, Wall,
};
use crate::geometry::Pos;
use crate::minds::astar::astar;
use crate::minds::grid::TorusGrid;
use crate::rules::Harvest;
use crate::world::World;
use rand::Rng;

/// The rig's width and height.
pub const RIG_WIDTH: u32 = 13;
pub const RIG_HEIGHT: u32 = 8;
/// Ticks in a lab day, and in its morning.
pub const DAY: u64 = 4;
pub const MORNING: u64 = 2;
/// The sugar each agent is given on the test evening.
pub const F: f64 = 30.0;
/// Most agents a lab holds (a compartment's sites).
pub const MAX_AGENTS: u32 = 9;
/// Sugar on each site of a compartment with food.
const FOOD: f64 = 1.0;

/// The agent's own caching parameters for [`run_lab`]: `share` (unused by
/// the lab's allocations, which cache all of F; kept for the field),
/// `lambda` (compensate's weight decay) and `lookahead` (plan's days).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabParams {
    pub share: f64,
    pub lambda: f64,
    pub lookahead: u32,
}

impl Default for LabParams {
    fn default() -> Self {
        Self {
            share: 0.5,
            lambda: 0.5,
            lookahead: 1,
        }
    }
}

/// Units one agent cached in K1, K2 and K3 on the test evening.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LabResult {
    pub caches: [u32; 3],
}

/// Compartment `k`'s left column (K1 = 0, K2 = 1, K3 = 2).
fn left(k: u32) -> u32 {
    1 + 4 * k
}

/// Compartment `k`'s tray: the site just inside its doorway.
pub fn tray(k: u32) -> Pos {
    Pos::new(left(k) + 1, 3)
}

/// Compartment `k`'s doorway.
pub fn doorway(k: u32) -> Pos {
    Pos::new(left(k) + 1, 4)
}

/// Compartment `k`'s `i`-th site, row-major (i < 9).
fn site(k: u32, i: u32) -> Pos {
    Pos::new(left(k) + i % 3, 1 + i / 3)
}

/// The `i`-th agent's perch.
pub fn perch(i: u32) -> Pos {
    Pos::new(1 + i, 6)
}

/// The rig's walls: the border, the compartments' dividers, and the row
/// between the compartments and the hall, doorways included (shut).
pub fn rig_walls() -> Vec<Wall> {
    let wall = |x, y, width, height| Wall {
        x,
        y,
        width,
        height,
        opaque: true,
    };
    vec![
        wall(0, 0, RIG_WIDTH, 1),
        wall(0, RIG_HEIGHT - 1, RIG_WIDTH, 1),
        wall(0, 1, 1, RIG_HEIGHT - 2),
        wall(RIG_WIDTH - 1, 1, 1, RIG_HEIGHT - 2),
        wall(4, 1, 1, 3),
        wall(8, 1, 1, 3),
        wall(1, 4, RIG_WIDTH - 2, 1),
    ]
}

/// Why `config` can't run its lab, if it can't: the lab runs only on its
/// rig (size, walls, a flat zero map) with 1 to [`MAX_AGENTS`] agents.
pub(crate) fn rig_problem(config: &Config) -> Option<String> {
    config.lab?;
    let flat_zero = matches!(config.goods[0].map, Map::Flat { capacity } if capacity == 0.0);
    if config.width != RIG_WIDTH
        || config.height != RIG_HEIGHT
        || config.walls != rig_walls()
        || !flat_zero
    {
        return Some(format!(
            "a lab runs on its rig: {RIG_WIDTH} × {RIG_HEIGHT}, its walls and a flat zero map"
        ));
    }
    if !(1..=MAX_AGENTS).contains(&config.population) {
        return Some(format!("a lab holds 1 to {MAX_AGENTS} agents"));
    }
    None
}

/// Training days before the test evening: Raby's six mornings, Amodio's
/// nine.
pub fn training_days(protocol: LabProtocol) -> u64 {
    match protocol {
        LabProtocol::Raby => 6,
        LabProtocol::Amodio => 9,
    }
}

/// The compartments an agent may cache in on the test evening.
pub fn caching_places(protocol: LabProtocol) -> &'static [u32] {
    match protocol {
        LabProtocol::Raby => &[0, 2],
        LabProtocol::Amodio => &[0, 1, 2],
    }
}

/// Day `d`'s compartment (0-based day).
pub fn place(protocol: LabProtocol, d: u64) -> u32 {
    match protocol {
        LabProtocol::Raby => {
            if d.is_multiple_of(2) {
                0
            } else {
                2
            }
        }
        LabProtocol::Amodio => (d % 3) as u32,
    }
}

/// Whether day `d` (0-based) has food: on the first day and every other
/// day after it when `food_first`, on the others otherwise.
pub fn has_food(lab: Lab, d: u64) -> bool {
    lab.food_first == d.is_multiple_of(2)
}

/// The config of a lab world: the rig, `population` agents under `rule`
/// with `params`.
pub fn rig_config(lab: Lab, rule: CachingRule, params: LabParams, population: u32) -> Config {
    let mut c = Config {
        width: RIG_WIDTH,
        height: RIG_HEIGHT,
        population,
        placement: Placement::Block {
            x: 1,
            y: 6,
            width: RIG_WIDTH - 2,
            height: 1,
        },
        vision: URange::new(1, 1),
        goods: vec![Good {
            map: Map::Flat { capacity: 0.0 },
            metabolism: URange::new(0, 0),
            endowment: URange::new(10, 10),
            ..Good::sugar()
        }],
        movement: Movement {
            mode: MoveMode::Walk,
            speed: 1,
        },
        walls: rig_walls(),
        lab: Some(lab),
        ..Config::default()
    };
    c.lifespan.enabled = false;
    c.caching.rule = rule;
    c.caching.share = params.share;
    c.caching.lambda = params.lambda;
    c.caching.lookahead = params.lookahead;
    c
}

/// The lab's agents, in id order.
fn roster(world: &World) -> Vec<AgentId> {
    world.agents().map(|a| a.id).collect()
}

/// Empties every compartment's sites.
fn clear_food(world: &mut World) {
    for k in 0..3 {
        for i in 0..9 {
            let s = world.site_mut(site(k, i));
            s.resource[0] = 0.0;
            s.capacity[0] = 0.0;
        }
    }
}

/// Carries `id` to `to` (free, or its own site), clearing its walk plan.
fn carry(world: &mut World, id: AgentId, to: Pos) {
    world.move_agent(id, to);
    world.agent_mut(id).expect("live agent").plan = Plan::default();
}

/// The lab's schedule for this tick (see the module docs). Called by
/// `World::step` before anyone moves, only in a lab world.
pub(crate) fn apply(world: &mut World) {
    let Some(lab) = world.config.lab else {
        return;
    };
    let n = training_days(lab.protocol);
    let (day, phase) = (world.tick / DAY, world.tick % DAY);
    if day < n {
        if phase == 0 {
            morning(world, lab, day);
        } else if phase == MORNING {
            evening(world, lab, day);
        }
        return;
    }
    if world.tick == n * DAY {
        test_evening(world, lab);
    }
    walk(world);
}

fn morning(world: &mut World, lab: Lab, d: u64) {
    let k = place(lab.protocol, d);
    clear_food(world);
    if has_food(lab, d) {
        for i in 0..9 {
            let s = world.site_mut(site(k, i));
            s.resource[0] = FOOD;
            s.capacity[0] = FOOD;
        }
    }
    for (i, id) in roster(world).into_iter().enumerate() {
        if i >= MAX_AGENTS as usize {
            break;
        }
        carry(world, id, site(k, i as u32));
        let a = world.agent_mut(id).expect("live agent");
        a.episodes
            .get_or_insert_with(Episodes::default)
            .push(Episode {
                place: k,
                food: false,
            });
    }
}

/// Marks today's episode as having found food when `harvest` gathered any
/// sugar during a training morning. Called by the caching step in a lab
/// world, in place of the field's burying.
pub(crate) fn found(world: &mut World, id: AgentId, harvest: &Harvest) {
    let Some(lab) = world.config.lab else {
        return;
    };
    let morning = world.tick / DAY < training_days(lab.protocol) && world.tick % DAY < MORNING;
    if !morning || harvest.gathered[0] <= 0.0 {
        return;
    }
    let a = world.agent_mut(id).expect("live agent");
    if let Some(e) = a.episodes.as_mut().and_then(|e| e.days.back_mut()) {
        e.food = true;
    }
}

fn evening(world: &mut World, lab: Lab, d: u64) {
    let k = place(lab.protocol, d);
    clear_food(world);
    let config_lambda = world.config.caching.lambda;
    for (i, id) in roster(world).into_iter().enumerate() {
        if i >= MAX_AGENTS as usize {
            break;
        }
        let compensate = rule_of(world, id) == CachingRule::Compensate;
        let a = world.agent_mut(id).expect("live agent");
        if compensate {
            let lambda = a.cache_params.map_or(config_lambda, |p| p.1);
            let food = a
                .episodes
                .as_ref()
                .and_then(|e| e.days.back())
                .is_some_and(|e| e.food);
            a.weights.harvested(k, food, lambda);
        }
        carry(world, id, perch(i as u32));
    }
}

fn test_evening(world: &mut World, lab: Lab) {
    clear_food(world);
    for &k in caching_places(lab.protocol) {
        world.open_wall(doorway(k));
    }
    for id in roster(world) {
        world.agent_mut(id).expect("live agent").holdings[0] += F;
    }
}

/// `id`'s test-evening allocation of [`F`] over the protocol's caching
/// compartments, in K order (whole units, remainders to K1 < K2 < K3).
pub fn allocation(world: &World, id: AgentId) -> Vec<(u32, f64)> {
    let Some(lab) = world.config.lab else {
        return Vec::new();
    };
    let places = caching_places(lab.protocol);
    let a = world.agent(id).expect("live agent");
    match rule_of(world, id) {
        CachingRule::None => Vec::new(),
        CachingRule::Even => allocate_even(F, places, true),
        CachingRule::Compensate => allocate_compensate(F, places, a.weights.map(), true),
        CachingRule::Plan => {
            let mut needy = Vec::new();
            if let Some(episodes) = &a.episodes {
                for k in 1..=world.config.caching.lookahead as usize {
                    if let Some(e) = episodes.predict(k) {
                        if !e.food && places.contains(&e.place) {
                            needy.push(e.place);
                        }
                    }
                }
            }
            allocate_plan(F, &needy, true)
        }
    }
}

/// The next compartment `id` still has to bury in, and how much.
fn pending(world: &World, id: AgentId) -> Option<(u32, f64)> {
    let a = world.agent(id).expect("live agent");
    allocation(world, id)
        .into_iter()
        .find(|&(k, q)| q > 0.0 && !a.caches.contains_key(&(world.torus.index(tray(k)) as u32)))
}

/// Whether `id` (the `i`-th agent) is done with its test evening: nothing
/// left to bury, and home on its perch.
fn done(world: &World, id: AgentId, i: u32) -> bool {
    pending(world, id).is_none() && world.agent(id).expect("live agent").pos == perch(i)
}

/// The first step of the shortest walk from `from` to `to` over free sites
/// (A*, deterministic), or `None` when there's none.
fn step_toward(world: &World, from: Pos, to: Pos) -> Option<Pos> {
    let torus = world.torus;
    let grid = TorusGrid::new(torus, |q: Pos| {
        !world.is_wall(q) && (q == to || world.occupant(q).is_none())
    });
    let found = astar(&grid, torus.index(from), torus.index(to), torus.len())?;
    found.path.get(1).map(|&i| torus.pos(i))
}

/// The test evening's walking: the first agent not done takes one step
/// toward its next tray (burying there on arrival) or back to its perch.
fn walk(world: &mut World) {
    let ids = roster(world);
    let Some((i, id)) = ids
        .into_iter()
        .enumerate()
        .take(MAX_AGENTS as usize)
        .find(|&(i, id)| !done(world, id, i as u32))
    else {
        return;
    };
    let next = pending(world, id);
    let target = next.map_or(perch(i as u32), |(k, _)| tray(k));
    let pos = world.agent(id).expect("live agent").pos;
    if pos != target {
        if let Some(step) = step_toward(world, pos, target) {
            carry(world, id, step);
        }
    }
    if let Some((k, q)) = next {
        if world.agent(id).expect("live agent").pos == tray(k) {
            super::bury(world, id, q);
        }
    }
}

/// Whether the test evening has begun and every agent is done.
pub fn finished(world: &World) -> bool {
    let Some(lab) = world.config.lab else {
        return false;
    };
    world.tick > training_days(lab.protocol) * DAY
        && roster(world)
            .into_iter()
            .enumerate()
            .take(MAX_AGENTS as usize)
            .all(|(i, id)| done(world, id, i as u32))
}

/// Each agent's caches in K1, K2 and K3 (its caches at the trays), in id
/// order.
pub fn results(world: &World) -> Vec<LabResult> {
    world
        .agents()
        .map(|a| {
            let mut caches = [0u32; 3];
            for (k, slot) in caches.iter_mut().enumerate() {
                let i = world.torus.index(tray(k as u32)) as u32;
                *slot = a.caches.get(&i).map_or(0, |&q| q.round() as u32);
            }
            LabResult { caches }
        })
        .collect()
}

/// Draws each agent's own (share, λ) through the world's RNG, in id order
/// (share first): share uniform in [0.4, 0.6), λ in [0.3, 0.7). The lab's
/// allocations cache all of F, so only λ (under `compensate`) changes what
/// an agent does. `share` is drawn but unused in the lab, only to keep the
/// draw order stable.
pub fn draw_params(world: &mut World) {
    for id in roster(world) {
        let share = 0.4 + 0.2 * world.rng.gen::<f64>();
        let lambda = 0.3 + 0.4 * world.rng.gen::<f64>();
        world.agent_mut(id).expect("live agent").cache_params = Some((share, lambda));
    }
}

/// Steps a lab world until its test evening is done (with a bound, should
/// a walk ever find no path) and returns its results.
fn run(mut world: World) -> Vec<LabResult> {
    let lab = world.config.lab.expect("a lab world");
    let limit = training_days(lab.protocol) * DAY + 1 + 64 * u64::from(MAX_AGENTS);
    while !finished(&world) && world.tick < limit {
        world.step();
    }
    assert!(
        finished(&world),
        "the lab's test evening didn't finish within {limit} ticks"
    );
    results(&world)
}

/// One agent through `lab`'s protocol under `rule` with `params`; its
/// caches on the test evening.
pub fn run_lab(lab: Lab, rule: CachingRule, params: LabParams, seed: u64) -> LabResult {
    let config = rig_config(lab, rule, params, 1);
    let world = World::new(config, seed).expect("the rig validates");
    run(world)[0]
}

/// `n` agents through `lab`'s protocol under `rule`, each with its own
/// (share, λ) drawn by [`draw_params`] (`params.lookahead` for all); their
/// caches on the test evening, in id order.
pub fn run_population(
    lab: Lab,
    rule: CachingRule,
    params: LabParams,
    n: u32,
    seed: u64,
) -> Vec<LabResult> {
    let config = rig_config(lab, rule, params, n);
    let mut world = World::new(config, seed).expect("the rig validates");
    draw_params(&mut world);
    run(world)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raby(first: bool) -> Lab {
        Lab {
            protocol: LabProtocol::Raby,
            food_first: first,
        }
    }

    #[test]
    fn a_mixed_lab_gives_each_agent_its_own_rules_allocation() {
        // `caching.mixed`: agents 1..=n follow none, even, compensate, plan
        // round-robin, and each caches what that rule alone would (the
        // agents don't interact: they're carried, and tested one at a time).
        let amodio = Lab {
            protocol: LabProtocol::Amodio,
            food_first: true,
        };
        for (lab, n) in [(raby(true), 8), (raby(false), 8), (amodio, 6)] {
            let mut c = rig_config(lab, CachingRule::Even, LabParams::default(), n);
            c.caching.mixed = true;
            c.validate().unwrap();
            let w = World::new(c, 5).unwrap();
            let rules: Vec<CachingRule> = w.agents().map(|a| a.caching_rule).collect();
            assert_eq!(
                rules,
                (1..=u64::from(n))
                    .map(|id| w.config.caching.founder_rule(id))
                    .collect::<Vec<_>>()
            );
            let got = run(w);
            for (i, r) in got.iter().enumerate() {
                // A lab needs a caching rule, so `none` alone can't run: it
                // caches nothing.
                let alone = if rules[i] == CachingRule::None {
                    LabResult::default()
                } else {
                    run_lab(lab, rules[i], LabParams::default(), 5)
                };
                assert_eq!(*r, alone, "{lab:?} agent {i} under {:?}", rules[i]);
            }
            assert_ne!(got[1], LabResult::default(), "even caches all of F");
        }
    }

    #[test]
    fn the_rig_validates_and_has_49_free_sites() {
        let c = rig_config(raby(true), CachingRule::Even, LabParams::default(), 9);
        assert_eq!(c.validate(), Ok(()));
        assert_eq!(c.free_sites(), 27 + 22);
        let w = World::new(c, 1).unwrap();
        for k in 0..3 {
            assert!(w.is_wall(doorway(k)));
            assert!(!w.is_wall(tray(k)));
            assert!(w.walled_apart(tray(k), perch(0)));
        }
    }

    #[test]
    fn a_lab_off_its_rig_or_too_full_is_an_error() {
        let fields = |c: Config| match c.validate() {
            Ok(()) => Vec::new(),
            Err(e) => e.into_iter().map(|e| e.field).collect::<Vec<_>>(),
        };
        let base = || rig_config(raby(true), CachingRule::Even, LabParams::default(), 1);
        let mut c = base();
        c.width = 14;
        assert!(fields(c).contains(&"lab".to_string()));
        let mut c = base();
        c.walls.pop();
        assert!(fields(c).contains(&"lab".to_string()));
        let mut c = base();
        c.population = 10;
        assert!(fields(c).contains(&"lab".to_string()));
        let mut c = base();
        c.goods[0].map = Map::Flat { capacity: 2.0 };
        assert!(fields(c).contains(&"lab".to_string()));
    }

    #[test]
    fn schedules_follow_the_papers() {
        let places: Vec<u32> = (0..6).map(|d| place(LabProtocol::Raby, d)).collect();
        assert_eq!(places, [0, 2, 0, 2, 0, 2]);
        let places: Vec<u32> = (0..9).map(|d| place(LabProtocol::Amodio, d)).collect();
        assert_eq!(places, [0, 1, 2, 0, 1, 2, 0, 1, 2]);
        let ff = Lab {
            protocol: LabProtocol::Amodio,
            food_first: true,
        };
        let food: Vec<bool> = (0..9).map(|d| has_food(ff, d)).collect();
        assert_eq!(
            food,
            [true, false, true, false, true, false, true, false, true]
        );
    }

    #[test]
    fn a_training_day_shuts_the_agent_in_and_records_what_it_found() {
        let c = rig_config(raby(true), CachingRule::Plan, LabParams::default(), 1);
        let mut w = World::new(c, 3).unwrap();
        let id = roster(&w)[0];
        // Day 0: K1 with breakfast.
        w.step();
        let a = w.agent(id).unwrap();
        assert_eq!(a.pos, site(0, 0));
        assert_eq!(a.holdings[0], 11.0, "10 + the 1 it found on its first tick");
        let e = *a.episodes.as_ref().unwrap().days.back().unwrap();
        assert_eq!(
            e,
            Episode {
                place: 0,
                food: true
            }
        );
        w.step();
        w.step();
        assert_eq!(w.agent(id).unwrap().pos, perch(0), "home for the evening");
        // Day 1: K3, no breakfast.
        w.step();
        w.step();
        let a = w.agent(id).unwrap();
        assert_eq!(a.pos, site(2, 0));
        let e = *a.episodes.as_ref().unwrap().days.back().unwrap();
        assert_eq!(
            e,
            Episode {
                place: 2,
                food: false
            }
        );
        assert_eq!(a.caches.len(), 0, "nothing buried while training");
    }

    #[test]
    fn rabys_test_evening_opens_k1_and_k3_and_keeps_k2_shut() {
        let c = rig_config(raby(false), CachingRule::Even, LabParams::default(), 1);
        let mut w = World::new(c, 3).unwrap();
        for _ in 0..=24 {
            w.step();
        }
        assert!(!w.is_wall(doorway(0)));
        assert!(w.is_wall(doorway(1)), "K2 stays shut");
        assert!(!w.is_wall(doorway(2)));
        while !finished(&w) {
            w.step();
            assert!(w.is_wall(doorway(1)));
            assert!(w.tick < 200);
        }
        assert_eq!(results(&w)[0].caches, [15, 0, 15]);
    }

    #[test]
    fn a_lab_refuses_placing_and_removing_agents_and_steps_on() {
        let c = rig_config(raby(false), CachingRule::Even, LabParams::default(), 2);
        let mut w = World::new(c, 3).unwrap();
        let mut plain = w.clone();
        w.step();
        plain.step();
        let o = crate::edit::AgentOverrides::default();
        assert_eq!(
            w.place_agent(5, 5, &o).unwrap_err(),
            "the lab's roster is fixed"
        );
        let at = w.agents().next().unwrap().pos;
        assert_eq!(
            w.remove_agent(at.x, at.y).unwrap_err(),
            "the lab's roster is fixed"
        );
        assert_eq!(w.population(), 2);
        // Refused edits change nothing: it steps on as the unedited world.
        while !finished(&plain) {
            w.step();
            plain.step();
            assert_eq!(w.fingerprint(), plain.fingerprint());
            assert!(plain.tick < 300);
        }
        assert!(finished(&w));
        assert_eq!(
            results(&w),
            vec![
                LabResult {
                    caches: [15, 0, 15]
                };
                2
            ]
        );
    }

    #[test]
    fn on_the_test_evening_the_doors_open_and_it_walks_to_each_tray() {
        let c = rig_config(
            Lab {
                protocol: LabProtocol::Amodio,
                food_first: true,
            },
            CachingRule::Even,
            LabParams::default(),
            1,
        );
        let mut w = World::new(c, 3).unwrap();
        let id = roster(&w)[0];
        for _ in 0..36 {
            w.step();
        }
        assert!(w.is_wall(doorway(0)));
        let mut positions = Vec::new();
        while !finished(&w) {
            w.step();
            positions.push(w.agent(id).unwrap().pos);
            assert!(w.tick < 200);
        }
        assert!(!w.is_wall(doorway(0)));
        // One step a tick: every move is to a neighbor.
        for pair in positions.windows(2) {
            let d = pair[0].x.abs_diff(pair[1].x) + pair[0].y.abs_diff(pair[1].y);
            assert!(d <= 1, "{pair:?}");
        }
        for k in 0..3 {
            assert!(positions.contains(&tray(k)));
        }
        assert_eq!(*positions.last().unwrap(), perch(0));
        assert_eq!(results(&w)[0].caches, [10, 10, 10]);
        // 10 + 2 on each of five food mornings (the site regrows between
        // the morning's two ticks).
        assert_eq!(w.agent(id).unwrap().holdings[0], 20.0);
    }
}
