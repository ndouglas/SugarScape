//! The demographic Prisoner's Dilemma's world: agents with a fixed strategy
//! (cooperate or defect), wealth and age on a von Neumann torus. In each
//! cycle every agent, in the list's order, moves to a random unoccupied site
//! within its vision, plays each neighbour, clones itself onto an empty
//! neighbouring site once rich enough, ages, and dies when its wealth goes
//! negative or it outlives the maximum age.

use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{
    DeathTiming, DpdConfig, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct, Pairing, Play,
    Removal, Shuffle, Updating,
};
use super::stats::{ratio, DpdSnapshot, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, COOL, HOT, RED};
use crate::rng::{self, SimRng};
use crate::spatial::{self, Geometry};
use crate::stats::Stats;

/// The Strategy view's colours (GSS: cooperators blue, defectors red).
pub const COOPERATOR: Rgb = BLUE;
pub const DEFECTOR: Rgb = RED;

/// No agent on a site.
const NONE: u32 = u32::MAX;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DpdMode {
    Strategy,
    Wealth,
    Age,
    Surrounded,
}

impl std::str::FromStr for DpdMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "strategy" => Self::Strategy,
            "wealth" => Self::Wealth,
            "age" => Self::Age,
            "surrounded" => Self::Surrounded,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// An agent in the call list.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: u64,
    pub cooperator: bool,
    pub wealth: f64,
    pub age: u32,
    pub site: u32,
    /// Dead this cycle: it leaves the list at the cycle's end (and its site
    /// at once or then, per `removal`).
    pub dead: bool,
    /// Payoffs received and games played this cycle.
    pub income: f64,
    pub games: u32,
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DpdInspection {
    pub site: Site,
    /// The agent on the site, or none.
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Site {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    /// "C" or "D".
    pub strategy: &'static str,
    pub wealth: f64,
    pub age: u32,
    /// The maximum age (0: none).
    pub max_age: u32,
    pub surrounded: bool,
    /// Payoffs received and games played this cycle.
    pub income: f64,
    pub games: u32,
    /// The occupied neighbours: up, left, right, down.
    pub neighbors: Vec<NeighborView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NeighborView {
    pub x: u32,
    pub y: u32,
    pub id: u64,
    pub strategy: &'static str,
    /// One game's payoff to the agent against this neighbour, and to the
    /// neighbour, under the current payoffs.
    pub payoff: f64,
    pub their_payoff: f64,
}

/// Why an agent died.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DpdDeath {
    /// Its wealth went negative.
    Broke,
    /// Past the maximum age.
    OldAge,
}

/// What the last cycle did, for the studio's frame dumps: each birth as
/// (offspring, parent), and each death, in the order they happened.
/// Recording draws nothing.
#[derive(Clone, Debug, Default)]
pub struct DpdEvents {
    pub births: Vec<(u64, u64)>,
    pub deaths: Vec<(u64, DpdDeath)>,
}

#[derive(Clone)]
pub struct DpdWorld {
    pub config: DpdConfig,
    /// The periodic von Neumann lattice (movement, play, birth), and the
    /// Moore one (the surrounded index); fixed once built, so keyframes
    /// share them.
    pub geometry: Arc<Geometry>,
    moore: Arc<Geometry>,
    /// The distinct displacements (dx, dy mod width) within `vision`.
    reach: Arc<Vec<(u32, u32)>>,
    /// Completed cycles.
    pub tick: u64,
    /// The call list.
    agents: Vec<Agent>,
    /// Each site's agent (an index into `agents`), or `NONE`.
    at: Vec<u32>,
    /// The unoccupied sites (in no particular order) and each site's index
    /// in it (`u32::MAX` when occupied). A dead agent awaiting removal
    /// occupies its site.
    empty: Vec<u32>,
    slot: Vec<u32>,
    next_id: u64,
    /// Agents in the list not yet dead.
    living: u32,
    births: u32,
    deaths: u32,
    events: DpdEvents,
    rng: SimRng,
    pub stats: Stats<DpdSnapshot>,
}

/// The distinct displacements within von Neumann distance `vision` on a
/// `width` torus, excluding none: ascending by (dy, dx), wrapped, the first
/// of any that coincide kept.
fn reach(width: u32, vision: u32) -> Vec<(u32, u32)> {
    let (w, v) = (i64::from(width), i64::from(vision));
    let mut out = Vec::new();
    for dy in -v..=v {
        let span = v - dy.abs();
        for dx in -span..=span {
            let d = (dx.rem_euclid(w) as u32, dy.rem_euclid(w) as u32);
            if d != (0, 0) && !out.contains(&d) {
                out.push(d);
            }
        }
    }
    out
}

impl DpdWorld {
    pub fn new(config: DpdConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let lattice = |neighborhood| spatial::SpatialConfig {
            width: config.width,
            height: config.width,
            neighborhood,
            boundary: spatial::Boundary::Periodic,
            ..Default::default()
        };
        // A square lattice draws nothing; the world's stream starts untouched.
        let geometry = Geometry::new(
            &lattice(spatial::Neighborhood::VonNeumann),
            &mut rng::seeded(0),
        );
        let moore = Geometry::new(&lattice(spatial::Neighborhood::Moore), &mut rng::seeded(0));
        let n = geometry.len();
        let reach = reach(config.width, config.vision);
        let mut w = DpdWorld {
            config,
            geometry: Arc::new(geometry),
            moore: Arc::new(moore),
            reach: Arc::new(reach),
            tick: 0,
            agents: Vec::new(),
            at: vec![NONE; n],
            empty: (0..n as u32).collect(),
            slot: (0..n as u32).collect(),
            next_id: 1,
            living: 0,
            births: 0,
            deaths: 0,
            events: DpdEvents::default(),
            rng: rng::seeded(seed),
            stats: Stats::default(),
        };
        for _ in 0..w.config.agents {
            let site = w.random_empty().expect("validation keeps agents ≤ sites");
            let cooperator = w.rng.gen::<f64>() < w.config.initial_cooperators;
            let age = w.random_age();
            w.add(site, cooperator, w.config.initial_wealth, age);
        }
        w.record();
        Ok(w)
    }

    /// The last cycle's births and deaths.
    pub fn events(&self) -> &DpdEvents {
        &self.events
    }

    pub fn sites(&self) -> usize {
        self.at.len()
    }

    /// The living agents, in call order.
    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.agents.iter().filter(|a| !a.dead)
    }

    /// The living agent on `site`, if any.
    pub fn agent_at(&self, site: usize) -> Option<&Agent> {
        self.living_at(site).map(|k| &self.agents[k])
    }

    pub fn population(&self) -> usize {
        self.living as usize
    }

    /// Whether the run has reached its last cycle.
    pub fn is_finished(&self) -> bool {
        self.config.end > 0 && self.tick >= u64::from(self.config.end)
    }

    /// A uniform age in 1 … `max_age`, or 0 with no maximum.
    fn random_age(&mut self) -> u32 {
        match self.config.max_age {
            0 => 0,
            m => self.rng.gen_range(0..m) + 1,
        }
    }

    /// Adds an agent to the end of the list on the empty `site`; its id.
    fn add(&mut self, site: usize, cooperator: bool, wealth: f64, age: u32) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let k = self.agents.len();
        self.agents.push(Agent {
            id,
            cooperator,
            wealth,
            age,
            site: site as u32,
            dead: false,
            income: 0.0,
            games: 0,
        });
        self.place(site, k);
        self.living += 1;
        id
    }

    fn place(&mut self, site: usize, k: usize) {
        debug_assert_eq!(self.at[site], NONE, "site {site} is occupied");
        let j = self.slot[site] as usize;
        let last = *self.empty.last().expect("an empty site");
        self.empty.swap_remove(j);
        if last as usize != site {
            self.slot[last as usize] = j as u32;
        }
        self.slot[site] = u32::MAX;
        self.at[site] = k as u32;
    }

    fn vacate(&mut self, site: usize) {
        if self.at[site] != NONE {
            self.at[site] = NONE;
            self.slot[site] = self.empty.len() as u32;
            self.empty.push(site as u32);
        }
    }

    /// A uniformly chosen unoccupied site, if any.
    fn random_empty(&mut self) -> Option<usize> {
        if self.empty.is_empty() {
            None
        } else {
            let k = self.rng.gen_range(0..self.empty.len() as u32) as usize;
            Some(self.empty[k] as usize)
        }
    }

    /// A uniformly chosen member of `from`, if any.
    fn pick(&mut self, from: &[usize]) -> Option<usize> {
        if from.is_empty() {
            None
        } else {
            Some(from[self.rng.gen_range(0..from.len() as u32) as usize])
        }
    }

    /// The index of the living agent on `site`, if any.
    fn living_at(&self, site: usize) -> Option<usize> {
        let k = self.at[site];
        (k != NONE && !self.agents[k as usize].dead).then_some(k as usize)
    }

    /// Agent `k` moves to the empty `site`.
    fn relocate(&mut self, k: usize, site: usize) {
        let from = self.agents[k].site as usize;
        self.place(site, k);
        self.vacate(from);
        self.agents[k].site = site as u32;
    }

    /// One cycle.
    pub fn step(&mut self) {
        self.apply_schedule();
        self.begin_cycle();
        let start = self.agents.len();
        match self.config.updating {
            Updating::Asynchronous => {
                let mut i = 0;
                while i < self.bound(start) {
                    self.turn(i);
                    i += 1;
                }
            }
            Updating::Synchronous => {
                let phases: [fn(&mut Self, usize); 4] = [
                    Self::move_agent,
                    Self::play,
                    Self::reproduce,
                    Self::age_and_die,
                ];
                for phase in phases {
                    let mut i = 0;
                    while i < self.bound(start) {
                        if !self.agents[i].dead {
                            phase(self, i);
                        }
                        i += 1;
                    }
                }
            }
        }
        self.end_cycle();
        self.tick += 1;
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// Applies schedule entries due at the tick about to run.
    fn apply_schedule(&mut self) {
        let t = self.tick;
        let due: Vec<_> = self
            .config
            .schedule
            .iter()
            .filter(|c| c.tick == t)
            .flat_map(|c| c.set.clone())
            .collect();
        for (path, value) in due {
            let next = ModelConfig::Dpd(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Dpd(next)) = next {
                self.config = next;
            }
        }
    }

    fn begin_cycle(&mut self) {
        self.births = 0;
        self.deaths = 0;
        self.events = DpdEvents::default();
        for a in &mut self.agents {
            a.income = 0.0;
            a.games = 0;
        }
    }

    /// How far down the list this cycle reaches: the agents present at its
    /// start, or (`this_cycle`) newborns too.
    fn bound(&self, start: usize) -> usize {
        match self.config.newborns_act {
            NewbornsAct::NextCycle => start,
            NewbornsAct::ThisCycle => self.agents.len(),
        }
    }

    /// Agent `i`'s turn (asynchronous): move, play, reproduce, age and die.
    fn turn(&mut self, i: usize) {
        if self.agents[i].dead {
            return;
        }
        self.move_agent(i);
        self.play(i);
        if self.agents[i].dead {
            return;
        }
        self.reproduce(i);
        self.age_and_die(i);
    }

    /// Rule 1: to a uniformly chosen unoccupied site within vision (soup:
    /// anywhere); none → stay.
    fn move_agent(&mut self, i: usize) {
        let target = match self.config.pairing {
            Pairing::Space => {
                let w = self.config.width;
                let s = self.agents[i].site;
                let (x, y) = (s % w, s / w);
                let free: Vec<usize> = self
                    .reach
                    .iter()
                    .map(|&(dx, dy)| ((x + dx) % w + (y + dy) % w * w) as usize)
                    .filter(|&j| self.at[j] == NONE)
                    .collect();
                self.pick(&free)
            }
            Pairing::Soup => self.random_empty(),
        };
        if let Some(site) = target {
            self.relocate(i, site);
        }
    }

    /// Rule 2: one game with each occupied neighbour in direction order (up,
    /// left, right, down), stopping if the mover dies; or with one random
    /// occupied neighbour; or (soup) with one random other living agent.
    fn play(&mut self, i: usize) {
        match (self.config.pairing, self.config.play) {
            (Pairing::Soup, _) => {
                if self.living < 2 {
                    return;
                }
                let n = self.agents.len() as u32;
                let k = loop {
                    let k = self.rng.gen_range(0..n) as usize;
                    if k != i && !self.agents[k].dead {
                        break k;
                    }
                };
                self.game(i, k);
            }
            (Pairing::Space, Play::EachNeighbor) => {
                let geometry = Arc::clone(&self.geometry);
                for &j in geometry.neighbors(self.agents[i].site as usize) {
                    if let Some(k) = self.living_at(j as usize) {
                        self.game(i, k);
                        if self.agents[i].dead {
                            break;
                        }
                    }
                }
            }
            (Pairing::Space, Play::RandomNeighbor) => {
                let occupied: Vec<usize> = self
                    .geometry
                    .neighbors(self.agents[i].site as usize)
                    .iter()
                    .filter_map(|&j| self.living_at(j as usize))
                    .collect();
                if let Some(k) = self.pick(&occupied) {
                    self.game(i, k);
                }
            }
        }
    }

    /// One game between `i` and `k`: both add their payoff (less the
    /// metabolism when charged per game); with immediate death, either whose
    /// wealth goes negative dies.
    fn game(&mut self, i: usize, k: usize) {
        let (a, b) = (self.agents[i].cooperator, self.agents[k].cooperator);
        let per_game = self.config.metabolism_per == MetabolismPer::Interaction;
        let m = self.config.metabolism;
        for (me, pay) in [(i, self.config.payoff(a, b)), (k, self.config.payoff(b, a))] {
            let agent = &mut self.agents[me];
            agent.wealth += pay;
            if per_game {
                agent.wealth -= m;
            }
            agent.income += pay;
            agent.games += 1;
        }
        if self.config.death_timing == DeathTiming::Immediate {
            for me in [i, k] {
                if self.agents[me].wealth < 0.0 {
                    self.die(me, DpdDeath::Broke);
                }
            }
        }
    }

    fn die(&mut self, k: usize, cause: DpdDeath) {
        self.events.deaths.push((self.agents[k].id, cause));
        self.agents[k].dead = true;
        self.living -= 1;
        self.deaths += 1;
        if self.config.removal == Removal::Immediate {
            self.vacate(self.agents[k].site as usize);
        }
    }

    /// The strategy of an offspring of a parent with strategy `parent`.
    fn newborn_strategy(&mut self, parent: bool) -> bool {
        let flip = self.rng.gen::<f64>() < self.config.mutation;
        parent != flip
    }

    /// Rule 3: with wealth ≥ `fission_wealth` and an unoccupied neighbouring
    /// site (soup: anywhere), an offspring there with the endowment.
    fn reproduce(&mut self, i: usize) {
        if self.agents[i].wealth < self.config.fission_wealth {
            return;
        }
        let target = match self.config.pairing {
            Pairing::Space => {
                let free: Vec<usize> = self
                    .geometry
                    .neighbors(self.agents[i].site as usize)
                    .iter()
                    .map(|&j| j as usize)
                    .filter(|&j| self.at[j] == NONE)
                    .collect();
                self.pick(&free)
            }
            Pairing::Soup => self.random_empty(),
        };
        let Some(site) = target else {
            return;
        };
        let endowment = self.config.endowment;
        if self.config.endowment_from == EndowmentFrom::Parent {
            self.agents[i].wealth -= endowment;
        }
        let cooperator = self.newborn_strategy(self.agents[i].cooperator);
        let age = match self.config.newborn_age {
            NewbornAge::Random => self.random_age(),
            NewbornAge::Zero => 0,
        };
        let child = self.add(site, cooperator, endowment, age);
        self.events.births.push((child, self.agents[i].id));
        self.births += 1;
    }

    /// Rule 4: age by one, pay the per-cycle metabolism, and die with
    /// negative wealth or past the maximum age.
    fn age_and_die(&mut self, i: usize) {
        let c = &self.config;
        let (per_cycle, m, max_age) = (
            c.metabolism_per == MetabolismPer::Cycle,
            c.metabolism,
            c.max_age,
        );
        let a = &mut self.agents[i];
        a.age = a.age.saturating_add(1);
        if per_cycle {
            a.wealth -= m;
        }
        if a.wealth < 0.0 {
            self.die(i, DpdDeath::Broke);
        } else if max_age > 0 && a.age > max_age {
            self.die(i, DpdDeath::OldAge);
        }
    }

    /// After the list: the dead leave their sites and the list, and the
    /// list is reordered.
    fn end_cycle(&mut self) {
        for k in 0..self.agents.len() {
            let site = self.agents[k].site as usize;
            if self.agents[k].dead && self.at[site] == k as u32 {
                self.vacate(site);
            }
        }
        self.agents.retain(|a| !a.dead);
        match self.config.shuffle {
            Shuffle::Swaps => {
                let n = self.agents.len() as u32;
                for _ in 0..n / 2 {
                    let a = self.rng.gen_range(0..n) as usize;
                    let b = self.rng.gen_range(0..n) as usize;
                    self.agents.swap(a, b);
                }
            }
            Shuffle::Full => self.agents.shuffle(&mut self.rng),
        }
        for (k, a) in self.agents.iter().enumerate() {
            self.at[a.site as usize] = k as u32;
        }
        debug_assert_eq!(self.living as usize, self.agents.len());
    }

    /// Whether the living agent on `site` is a cooperator all eight of whose
    /// Moore neighbours are living cooperators.
    pub fn surrounded(&self, site: usize) -> bool {
        self.living_at(site)
            .is_some_and(|k| self.agents[k].cooperator)
            && self.moore.neighbors(site).iter().all(|&j| {
                self.living_at(j as usize)
                    .is_some_and(|k| self.agents[k].cooperator)
            })
    }

    fn record(&mut self) {
        let (mut c, mut d, mut wc, mut wd, mut surrounded) = (0u32, 0u32, 0.0, 0.0, 0u32);
        for a in self.agents.iter().filter(|a| !a.dead) {
            if a.cooperator {
                c += 1;
                wc += a.wealth;
                surrounded += u32::from(self.surrounded(a.site as usize));
            } else {
                d += 1;
                wd += a.wealth;
            }
        }
        let s = DpdSnapshot {
            tick: self.tick,
            cooperators: c,
            defectors: d,
            population: c + d,
            cooperator_share: ratio(f64::from(c), c + d),
            surrounded,
            wealth_c: ratio(wc, c),
            wealth_d: ratio(wd, d),
            births: self.births,
            deaths: self.deaths,
        };
        self.stats.push(s);
    }

    fn color(&self, a: &Agent, mode: DpdMode) -> Rgb {
        let own = if a.cooperator { COOPERATOR } else { DEFECTOR };
        match mode {
            DpdMode::Strategy => own,
            DpdMode::Wealth => {
                let top = 2.0 * self.config.fission_wealth.max(1.0);
                lerp(COOL, HOT, a.wealth / top)
            }
            DpdMode::Age => {
                let top = match self.config.max_age {
                    0 => 1000.0,
                    m => f64::from(m),
                };
                lerp(COOL, HOT, f64::from(a.age) / top)
            }
            DpdMode::Surrounded => {
                if self.surrounded(a.site as usize) {
                    COOPERATOR
                } else {
                    lerp(BACKGROUND, own, 0.35)
                }
            }
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<DpdInspection, String> {
        let site = self
            .geometry
            .at(x, y, 0)
            .ok_or_else(|| format!("({x}, {y}) is outside the lattice"))?;
        let letter = |c: bool| if c { "C" } else { "D" };
        let agent = self.agent_at(site).map(|a| {
            let neighbors = self
                .geometry
                .neighbors(site)
                .iter()
                .filter_map(|&j| {
                    let b = self.agent_at(j as usize)?;
                    let (x, y, _) = self.geometry.xyz(j as usize);
                    Some(NeighborView {
                        x,
                        y,
                        id: b.id,
                        strategy: letter(b.cooperator),
                        payoff: self.config.payoff(a.cooperator, b.cooperator),
                        their_payoff: self.config.payoff(b.cooperator, a.cooperator),
                    })
                })
                .collect();
            AgentView {
                id: a.id,
                strategy: letter(a.cooperator),
                wealth: a.wealth,
                age: a.age,
                max_age: self.config.max_age,
                surrounded: self.surrounded(site),
                income: a.income,
                games: a.games,
                neighbors,
            }
        });
        Ok(DpdInspection {
            site: Site { x, y },
            agent,
        })
    }
}

impl Model for DpdWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Dpd(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        DpdWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        DpdWorld::population(self)
    }

    /// FNV-1a over the tick, the id counter and every agent in call order.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        eat(self.next_id);
        for a in &self.agents {
            eat(a.id);
            eat(u64::from(a.site) | u64::from(a.cooperator) << 32);
            eat(a.wealth.to_bits());
            eat(u64::from(a.age));
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.width)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: DpdMode = mode.parse()?;
        buf.clear();
        buf.resize(self.at.len() * 4, 0);
        for (s, px) in buf.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let rgb = self.agent_at(s).map_or(BACKGROUND, |a| self.color(a, mode));
            px.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,x,y,strategy,wealth,age,surrounded\n");
        for a in self.agents() {
            let (x, y, _) = self.geometry.xyz(a.site as usize);
            writeln!(
                out,
                "{},{x},{y},{},{},{},{}",
                a.id,
                if a.cooperator { "C" } else { "D" },
                a.wealth,
                a.age,
                self.surrounded(a.site as usize)
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let a = self.agents().find(|a| a.id == id)?;
        let (x, y, _) = self.geometry.xyz(a.site as usize);
        Some((x, y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Dpd(next) = next else {
            return Err(wrong_model(ModelKind::Dpd, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduledChange;
    use serde_json::json;

    /// An empty `w` × `w` world (no initial agents), after `edit`.
    fn world(w: u32, edit: impl FnOnce(&mut DpdConfig)) -> DpdWorld {
        let mut c = DpdConfig {
            width: w,
            agents: 0,
            ..Default::default()
        };
        edit(&mut c);
        DpdWorld::new(c, 1).unwrap()
    }

    /// Puts an agent aged 0 at (x, y) at the end of the list; returns its index.
    fn put(w: &mut DpdWorld, (x, y): (u32, u32), cooperator: bool, wealth: f64) -> usize {
        let s = w.geometry.at(x, y, 0).unwrap();
        w.add(s, cooperator, wealth, 0);
        w.agents.len() - 1
    }

    fn site(w: &DpdWorld, (x, y): (u32, u32)) -> usize {
        w.geometry.at(x, y, 0).unwrap()
    }

    fn xy(w: &DpdWorld, k: usize) -> (u32, u32) {
        let (x, y, _) = w.geometry.xyz(w.agents[k].site as usize);
        (x, y)
    }

    /// The empty-site list holds exactly the unoccupied sites, each at its
    /// slot, and every living agent's site points back at it.
    fn empty_list_is_consistent(w: &DpdWorld) {
        let unoccupied = w.at.iter().filter(|&&k| k == NONE).count();
        assert_eq!(w.empty.len(), unoccupied);
        for (k, &s) in w.empty.iter().enumerate() {
            assert_eq!(w.at[s as usize], NONE, "listed site {s} is occupied");
            assert_eq!(w.slot[s as usize], k as u32);
        }
        for (k, a) in w.agents.iter().enumerate() {
            if !a.dead {
                assert_eq!(w.at[a.site as usize], k as u32, "agent {}", a.id);
            }
        }
    }

    #[test]
    fn vision_reaches_every_distinct_site_within_its_von_neumann_distance() {
        assert_eq!(reach(30, 1), [(0, 29), (29, 0), (1, 0), (0, 1)]);
        assert_eq!(reach(30, 2).len(), 12);
        assert_eq!(reach(30, 10).len(), 220);
        // On a 3 × 3 torus distance 2 reaches all eight other sites once.
        assert_eq!(reach(3, 2).len(), 8);
    }

    #[test]
    fn movers_go_to_an_unoccupied_site_within_vision_or_stay() {
        let mut w = world(5, |_| {});
        let a = put(&mut w, (2, 2), true, 100.0);
        for p in [(2, 1), (1, 2), (3, 2)] {
            put(&mut w, p, true, 100.0);
        }
        w.move_agent(a);
        assert_eq!(xy(&w, a), (2, 3), "the one unoccupied neighbour");
        let mut w = world(5, |_| {});
        let a = put(&mut w, (2, 2), true, 100.0);
        for p in [(2, 1), (1, 2), (3, 2), (2, 3)] {
            put(&mut w, p, true, 100.0);
        }
        w.move_agent(a);
        assert_eq!(xy(&w, a), (2, 2), "blocked: stays");
        empty_list_is_consistent(&w);
        // Vision 2 with the four neighbours taken: one of the eight sites at
        // distance 2, each in turn.
        let mut w = world(7, |c| c.vision = 2);
        let a = put(&mut w, (3, 3), true, 100.0);
        for p in [(3, 2), (2, 3), (4, 3), (3, 4)] {
            put(&mut w, p, true, 100.0);
        }
        let home = site(&w, (3, 3));
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..400 {
            w.move_agent(a);
            let (x, y) = xy(&w, a);
            assert_eq!(x.abs_diff(3) + y.abs_diff(3), 2, "({x}, {y})");
            seen.insert((x, y));
            w.relocate(a, home);
        }
        assert_eq!(seen.len(), 8);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_mover_plays_each_neighbour_or_one_and_both_players_are_paid() {
        let mut w = world(5, |_| {});
        let me = put(&mut w, (2, 2), true, 100.0);
        let up = put(&mut w, (2, 1), true, 100.0);
        let left = put(&mut w, (1, 2), false, 100.0);
        let right = put(&mut w, (3, 2), false, 100.0);
        w.play(me);
        let wealth = |k: usize| w.agents[k].wealth;
        assert_eq!(wealth(me), 100.0 + 5.0 - 6.0 - 6.0);
        assert_eq!(
            (wealth(up), wealth(left), wealth(right)),
            (105.0, 106.0, 106.0)
        );
        assert_eq!((w.agents[me].games, w.agents[me].income), (3, -7.0));
        // The working paper: one random occupied neighbour.
        let mut w = world(5, |c| c.play = Play::RandomNeighbor);
        let me = put(&mut w, (2, 2), true, 100.0);
        for (p, c) in [((2, 1), true), ((1, 2), false), ((3, 2), false)] {
            put(&mut w, p, c, 100.0);
        }
        w.play(me);
        assert_eq!(w.agents[me].games, 1);
        assert_eq!(w.agents.iter().map(|a| a.games).sum::<u32>(), 2);
    }

    #[test]
    fn a_mover_that_dies_stops_playing() {
        let mut w = world(5, |_| {});
        let me = put(&mut w, (2, 2), true, 1.0);
        let left = put(&mut w, (1, 2), false, 0.0);
        let right = put(&mut w, (3, 2), false, 0.0);
        w.play(me);
        assert!(w.agents[me].dead);
        assert_eq!((w.agents[left].wealth, w.agents[right].wealth), (6.0, 0.0));
        assert!(w.agent_at(site(&w, (2, 2))).is_none());
    }

    #[test]
    fn eleven_clones_and_the_endowment_comes_from_the_parent_or_is_granted() {
        let mut w = world(5, |_| {});
        let p = put(&mut w, (2, 2), true, 10.5);
        w.reproduce(p);
        assert_eq!(w.population(), 1, "10.5 < 11");
        w.agents[p].wealth = 11.0;
        w.reproduce(p);
        assert_eq!((w.population(), w.births), (2, 1));
        let child = w.agents.last().unwrap();
        assert_eq!((child.wealth, child.cooperator), (6.0, true));
        let (cx, cy, _) = w.geometry.xyz(child.site as usize);
        assert_eq!(cx.abs_diff(2) + cy.abs_diff(2), 1, "a neighbouring site");
        assert_eq!(w.agents[p].wealth, 5.0);
        let mut g = world(5, |c| c.endowment_from = EndowmentFrom::Granted);
        let p = put(&mut g, (2, 2), false, 11.0);
        g.reproduce(p);
        assert_eq!(
            (g.population(), g.agents[p].wealth, g.agents[1].wealth),
            (2, 11.0, 6.0)
        );
        // No unoccupied neighbour: no offspring.
        let mut full = world(5, |_| {});
        let p = put(&mut full, (2, 2), true, 50.0);
        for q in [(2, 1), (1, 2), (3, 2), (2, 3)] {
            put(&mut full, q, true, 0.0);
        }
        full.reproduce(p);
        assert_eq!((full.population(), full.agents[p].wealth), (5, 50.0));
    }

    #[test]
    fn negative_wealth_kills_at_once_and_the_site_empties_at_once() {
        let mut w = world(5, |_| {});
        let d = put(&mut w, (2, 2), false, 100.0);
        let c = put(&mut w, (2, 1), true, 1.0);
        w.play(d);
        assert!(w.agents[c].dead);
        assert_eq!((w.population(), w.deaths), (1, 1));
        assert_eq!(w.at[site(&w, (2, 1))], NONE);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn end_of_cycle_removal_leaves_a_body_that_blocks_and_takes_no_part() {
        let mut w = world(5, |c| c.removal = Removal::EndOfCycle);
        let d = put(&mut w, (2, 2), false, 100.0);
        let c = put(&mut w, (2, 1), true, 1.0);
        let other = put(&mut w, (1, 1), false, 100.0);
        w.play(d);
        assert!(w.agents[c].dead);
        let body = site(&w, (2, 1));
        assert_eq!(w.at[body], c as u32, "the body still holds its site");
        assert!(w.agent_at(body).is_none());
        w.play(other);
        assert_eq!(w.agents[other].games, 0, "nobody plays the dead");
        w.end_cycle();
        assert_eq!(w.at[body], NONE);
        assert_eq!(w.agents.len(), 2);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn own_turn_death_waits_for_the_agents_turn_where_it_may_recover() {
        let mut w = world(5, |c| c.death_timing = DeathTiming::OwnTurn);
        // A cooperator at (2, 1) boxed in by three cooperators and a defector.
        let c = put(&mut w, (2, 1), true, 1.0);
        for p in [(2, 0), (1, 1), (3, 1)] {
            put(&mut w, p, true, 100.0);
        }
        let d = put(&mut w, (2, 2), false, 100.0);
        w.play(d);
        assert_eq!(w.agents[c].wealth, -5.0);
        assert!(!w.agents[c].dead, "not on its own turn");
        w.turn(c);
        // Blocked from moving, it plays its four neighbours: −5 + 3 × 5 − 6.
        assert_eq!(w.agents[c].wealth, 4.0);
        assert!(!w.agents[c].dead);
        // Still negative at the end of its turn: it dies.
        w.agents[c].wealth = -20.0;
        w.turn(c);
        assert!(w.agents[c].dead);
    }

    #[test]
    fn initial_and_newborn_ages_are_uniform_up_to_the_maximum() {
        let c = DpdConfig {
            agents: 900,
            max_age: 100,
            ..Default::default()
        };
        let w = DpdWorld::new(c, 3).unwrap();
        let ages: Vec<u32> = w.agents().map(|a| a.age).collect();
        assert!(ages.iter().all(|a| (1..=100).contains(a)));
        assert!(ages.contains(&1) && ages.contains(&100));
        let none = DpdWorld::new(DpdConfig::default(), 3).unwrap();
        assert!(none.agents().all(|a| a.age == 0));
        let mut w = world(5, |c| c.max_age = 10);
        let p = put(&mut w, (2, 2), true, 0.0);
        let mut born = std::collections::BTreeSet::new();
        for _ in 0..300 {
            w.agents[p].wealth = 11.0;
            w.reproduce(p);
            let child = w.agents.pop().unwrap();
            w.vacate(child.site as usize);
            w.living -= 1;
            assert!((1..=10).contains(&child.age), "{}", child.age);
            born.insert(child.age);
        }
        assert_eq!(born.len(), 10);
        w.config.newborn_age = NewbornAge::Zero;
        w.agents[p].wealth = 11.0;
        w.reproduce(p);
        assert_eq!(w.agents.last().unwrap().age, 0);
    }

    #[test]
    fn agents_age_each_turn_and_die_past_the_maximum_age() {
        let mut w = world(5, |c| c.max_age = 100);
        let old = put(&mut w, (0, 0), true, 10.0);
        let young = put(&mut w, (3, 3), true, 10.0);
        w.agents[old].age = 100;
        w.agents[young].age = 99;
        w.age_and_die(old);
        w.age_and_die(young);
        assert!(w.agents[old].dead && !w.agents[young].dead);
        assert_eq!(w.agents[young].age, 100);
        let mut w = world(5, |_| {});
        let a = put(&mut w, (0, 0), true, 10.0);
        w.agents[a].age = 1_000_000;
        w.age_and_die(a);
        assert!(!w.agents[a].dead, "no maximum: age only counts");
    }

    #[test]
    fn a_live_max_age_lowered_mid_run_kills_agents_already_past_it_on_their_next_turn() {
        // No maximum yet: ages have been counting freely.
        let mut w = world(5, |_| {});
        let old = put(&mut w, (0, 0), true, 10.0);
        let young = put(&mut w, (3, 3), true, 10.0);
        w.agents[old].age = 150;
        w.agents[young].age = 50;
        let (old_id, young_id) = (w.agents[old].id, w.agents[young].id);
        let mut next = w.config.clone();
        next.max_age = 100;
        w.set_config(ModelConfig::Dpd(next)).unwrap();
        assert!(!w.agents[old].dead, "the change alone does not kill anyone");
        w.step();
        // `end_cycle` drops the dead and reorders the list, so look up by id.
        let ids: Vec<u64> = w.agents.iter().map(|a| a.id).collect();
        assert!(!ids.contains(&old_id), "already past the new maximum");
        assert!(ids.contains(&young_id));
    }

    #[test]
    fn metabolism_is_charged_per_cycle_or_per_game() {
        let mut w = world(5, |c| c.metabolism = 3.0);
        let a = put(&mut w, (2, 2), true, 10.0);
        let b = put(&mut w, (2, 1), true, 10.0);
        w.play(a);
        assert_eq!((w.agents[a].wealth, w.agents[b].wealth), (15.0, 15.0));
        w.age_and_die(a);
        assert_eq!(w.agents[a].wealth, 12.0);
        let mut w = world(5, |c| {
            c.metabolism = 3.0;
            c.metabolism_per = MetabolismPer::Interaction;
        });
        let a = put(&mut w, (2, 2), true, 10.0);
        let b = put(&mut w, (2, 1), true, 10.0);
        w.play(a);
        assert_eq!((w.agents[a].wealth, w.agents[b].wealth), (12.0, 12.0));
        w.age_and_die(a);
        assert_eq!(w.agents[a].wealth, 12.0);
        // Metabolism kills: 1 − 3 < 0.
        w.agents[b].wealth = 1.0;
        w.config.metabolism_per = MetabolismPer::Cycle;
        w.age_and_die(b);
        assert!(w.agents[b].dead);
    }

    #[test]
    fn shifted_payoffs_less_a_per_game_metabolism_replay_the_negative_payoffs() {
        let neg = DpdConfig {
            max_age: 100,
            ..Default::default()
        };
        let shifted = DpdConfig {
            t: 12.0,
            r: 11.0,
            p: 1.0,
            s: 0.0,
            metabolism: 6.0,
            metabolism_per: MetabolismPer::Interaction,
            ..neg.clone()
        };
        let mut a = DpdWorld::new(neg, 5).unwrap();
        let mut b = DpdWorld::new(shifted.clone(), 5).unwrap();
        a.run(150);
        b.run(150);
        assert_eq!(a.fingerprint(), b.fingerprint());
        let per_cycle = DpdConfig {
            metabolism_per: MetabolismPer::Cycle,
            ..shifted
        };
        let mut c = DpdWorld::new(per_cycle, 5).unwrap();
        c.run(150);
        assert_ne!(
            a.fingerprint(),
            c.fingerprint(),
            "per cycle is another model"
        );
    }

    #[test]
    fn soup_pairs_random_agents_and_places_anywhere() {
        let mut w = world(9, |c| c.pairing = Pairing::Soup);
        let a = put(&mut w, (0, 0), true, 100.0);
        let b = put(&mut w, (4, 4), false, 100.0);
        w.play(a);
        assert_eq!(
            (w.agents[a].games, w.agents[b].games),
            (1, 1),
            "strangers play"
        );
        assert_eq!((w.agents[a].wealth, w.agents[b].wealth), (94.0, 106.0));
        let mut lone = world(9, |c| c.pairing = Pairing::Soup);
        let x = put(&mut lone, (0, 0), true, 100.0);
        lone.play(x);
        assert_eq!(lone.agents[x].games, 0, "nobody to play");
        // Boxed in, it still clones and moves: anywhere.
        let mut w = world(9, |c| c.pairing = Pairing::Soup);
        let p = put(&mut w, (4, 4), true, 50.0);
        for q in [(4, 3), (3, 4), (5, 4), (4, 5)] {
            put(&mut w, q, true, 0.0);
        }
        w.reproduce(p);
        assert_eq!(w.population(), 6);
        let far = |w: &DpdWorld, k: usize| {
            let (x, y) = xy(w, k);
            x.abs_diff(4) + y.abs_diff(4) > 1
        };
        assert!(far(&w, 5), "the offspring is not next door");
        let mut moved_far = false;
        for _ in 0..20 {
            w.move_agent(p);
            moved_far |= far(&w, p);
        }
        assert!(moved_far);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn mutation_flips_an_offspring_strategy_at_its_rate() {
        let mut w = world(5, |c| c.mutation = 1.0);
        assert!(!w.newborn_strategy(true) && w.newborn_strategy(false));
        w.config.mutation = 0.0;
        assert!(w.newborn_strategy(true) && !w.newborn_strategy(false));
        w.config.mutation = 0.5;
        let flips = (0..4000).filter(|_| !w.newborn_strategy(true)).count();
        assert!((1800..=2200).contains(&flips), "{flips}");
    }

    #[test]
    fn the_call_order_takes_n_over_2_swaps_or_a_full_shuffle() {
        let unmoved = |shuffle: Shuffle| {
            let mut w = world(30, |c| c.shuffle = shuffle);
            for k in 0..400u32 {
                put(&mut w, (k % 30, k / 30), true, 0.0);
            }
            let before: Vec<u64> = w.agents.iter().map(|a| a.id).collect();
            w.end_cycle();
            let after: Vec<u64> = w.agents.iter().map(|a| a.id).collect();
            let mut sorted = after.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, before, "a permutation");
            empty_list_is_consistent(&w);
            before.iter().zip(&after).filter(|(a, b)| a == b).count()
        };
        // 200 random swaps leave about 1/e of 400 in place; a full shuffle about one.
        let swaps = unmoved(Shuffle::Swaps);
        assert!((110..=190).contains(&swaps), "{swaps}");
        assert!(unmoved(Shuffle::Full) < 10);
        let mut one = world(5, |_| {});
        put(&mut one, (0, 0), true, 0.0);
        one.end_cycle();
        assert_eq!(one.agents.len(), 1);
    }

    #[test]
    fn run_1_with_a_full_shuffle_reaches_the_pinned_fingerprint() {
        // No preset uses `shuffle: full` (every golden entry shuffles by swaps), so this pins one
        // directly; `crates/sugarscape-wasm/tests/web.rs` checks WASM against the same value.
        let c = DpdConfig {
            shuffle: Shuffle::Full,
            ..DpdConfig::default()
        };
        let mut w = DpdWorld::new(c, 1).unwrap();
        w.run(200);
        assert_eq!(w.fingerprint(), 0x97c2_897b_93c3_c7e4);
    }

    #[test]
    fn a_cycle_runs_turns_in_list_order_or_each_phase_for_everyone() {
        let phases: [fn(&mut DpdWorld, usize); 4] = [
            DpdWorld::move_agent,
            DpdWorld::play,
            DpdWorld::reproduce,
            DpdWorld::age_and_die,
        ];
        for updating in [Updating::Asynchronous, Updating::Synchronous] {
            let c = DpdConfig {
                updating,
                ..Default::default()
            };
            let mut w = DpdWorld::new(c, 4).unwrap();
            w.run(20);
            let mut by_hand = w.clone();
            let before = w.clone();
            w.step();
            by_hand.begin_cycle();
            let n = by_hand.agents.len();
            match updating {
                Updating::Asynchronous => (0..n).for_each(|i| by_hand.turn(i)),
                Updating::Synchronous => {
                    for phase in phases {
                        for i in 0..n {
                            if !by_hand.agents[i].dead {
                                phase(&mut by_hand, i);
                            }
                        }
                    }
                }
            }
            by_hand.end_cycle();
            by_hand.tick += 1;
            assert_eq!(w.fingerprint(), by_hand.fingerprint(), "{updating:?}");
            let mut other = before;
            other.config.updating = match updating {
                Updating::Asynchronous => Updating::Synchronous,
                Updating::Synchronous => Updating::Asynchronous,
            };
            other.step();
            assert_ne!(w.fingerprint(), other.fingerprint());
        }
    }

    #[test]
    fn newborns_act_from_the_next_cycle_unless_set_to_this_one() {
        for (act, age) in [(NewbornsAct::NextCycle, 0), (NewbornsAct::ThisCycle, 1)] {
            let mut w = world(7, |c| c.newborns_act = act);
            put(&mut w, (3, 3), false, 20.0);
            w.step();
            assert_eq!(w.population(), 2);
            let child = w.agents().find(|a| a.id == 2).unwrap();
            assert_eq!(child.age, age, "{act:?}");
        }
    }

    #[test]
    fn a_surrounded_cooperator_has_eight_cooperating_moore_neighbours() {
        let mut w = world(8, |_| {});
        // A 4 × 3 block of cooperators has two surrounded (as Figure 9.10).
        for y in 1..4 {
            for x in 1..5 {
                put(&mut w, (x, y), true, 0.0);
            }
        }
        w.record();
        assert_eq!(w.stats.latest().unwrap().surrounded, 2);
        assert!(w.surrounded(site(&w, (2, 2))) && w.surrounded(site(&w, (3, 2))));
        let k = w.at[site(&w, (1, 1))] as usize;
        w.agents[k].cooperator = false;
        w.record();
        assert_eq!(w.stats.latest().unwrap().surrounded, 1);
        assert!(!w.surrounded(site(&w, (2, 2))));
    }

    #[test]
    fn a_world_that_empties_reports_nan_and_still_draws() {
        let mut w = world(10, |c| c.metabolism = 1.0);
        put(&mut w, (2, 2), true, 0.5);
        w.step();
        assert_eq!(w.population(), 0);
        let last = w.stats.latest().unwrap();
        assert!(last.cooperator_share.is_nan() && last.wealth_c.is_nan() && last.wealth_d.is_nan());
        assert_eq!((last.deaths, last.births, last.population), (1, 0, 0));
        assert!(w.latest_json().contains("\"cooperator_share\":null"));
        let mut buf = Vec::new();
        for mode in ["strategy", "wealth", "age", "surrounded"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 10 * 10 * 4);
        }
        assert!(w.inspect_json(2, 2).unwrap().contains("\"agent\":null"));
        w.run(3);
        assert_eq!(w.tick, 4);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn frames_draw_each_mode_and_inspect_names_neighbours_and_payoffs() {
        let mut w = world(4, |c| c.max_age = 100);
        put(&mut w, (1, 0), true, 11.0);
        put(&mut w, (2, 0), false, 0.0);
        let mut buf = Vec::new();
        w.render("strategy", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 4 * 4 * 4);
        assert_eq!(&buf[0..3], &BACKGROUND);
        assert_eq!(&buf[4..7], &COOPERATOR);
        assert_eq!(&buf[8..11], &DEFECTOR);
        w.render("wealth", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &lerp(COOL, HOT, 0.5));
        assert_eq!(&buf[8..11], &COOL);
        w.render("age", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &COOL);
        w.render("surrounded", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &lerp(BACKGROUND, COOPERATOR, 0.35));
        assert!(w.render("nope", "", &mut buf).is_err());
        let v = w.inspect(1, 0).unwrap().agent.unwrap();
        assert_eq!(
            (v.strategy, v.wealth, v.max_age, v.surrounded),
            ("C", 11.0, 100, false)
        );
        assert_eq!(v.neighbors.len(), 1);
        let n = &v.neighbors[0];
        assert_eq!(
            (n.x, n.y, n.strategy, n.payoff, n.their_payoff),
            (2, 0, "D", -6.0, 6.0)
        );
        assert!(w.inspect(0, 0).unwrap().agent.is_none());
        assert!(w.inspect(4, 0).is_err());
        let json: serde_json::Value = serde_json::from_str(&w.inspect_json(1, 0).unwrap()).unwrap();
        assert_eq!(json["agent"]["neighbors"][0]["strategy"], "D");
    }

    #[test]
    fn runs_stop_at_the_end_and_follow_their_seed() {
        let c = DpdConfig {
            end: 30,
            ..Default::default()
        };
        let mut a = DpdWorld::new(c.clone(), 7).unwrap();
        let mut b = DpdWorld::new(c.clone(), 7).unwrap();
        let mut other = DpdWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(40);
        }
        assert_eq!(a.tick, 30);
        assert!(a.is_finished() && Model::finished(&a));
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
        assert_eq!(a.stats.history().len(), 31);
        let first = a.agents().next().unwrap().id;
        assert!(a.locate(first).is_some() && a.locate(0).is_none());
        assert_eq!(a.agents_csv().lines().count(), a.population() + 1);
        let s = a.stats.latest().unwrap();
        assert_eq!(s.cooperators + s.defectors, s.population);
        assert_eq!(s.population as usize, a.population());
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(10, |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("r".to_string(), json!(1.0))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.r, 5.0);
        w.step();
        assert_eq!(w.config.r, 1.0);
    }

    #[test]
    fn the_lattice_stays_consistent_under_every_switch_and_keyframes_replay() {
        let c = DpdConfig {
            max_age: 50,
            mutation: 0.1,
            ..Default::default()
        };
        let mut w = DpdWorld::new(c, 2).unwrap();
        type Edit = fn(&mut DpdConfig);
        let edits: [Edit; 9] = [
            |c| c.removal = Removal::EndOfCycle,
            |c| c.death_timing = DeathTiming::OwnTurn,
            |c| c.endowment_from = EndowmentFrom::Granted,
            |c| c.newborn_age = NewbornAge::Zero,
            |c| c.updating = Updating::Synchronous,
            |c| c.shuffle = Shuffle::Full,
            |c| c.newborns_act = NewbornsAct::ThisCycle,
            |c| c.pairing = Pairing::Soup,
            |c| {
                c.pairing = Pairing::Space;
                c.play = Play::RandomNeighbor;
                c.metabolism = 1.0;
                c.metabolism_per = MetabolismPer::Interaction;
            },
        ];
        for edit in edits {
            edit(&mut w.config);
            w.run(40);
            empty_list_is_consistent(&w);
            assert_eq!(w.population(), w.agents.len());
        }
        let kept = w.clone();
        let mut back = kept.clone();
        back.run(50);
        let mut again = kept;
        again.run(50);
        assert_eq!(back.fingerprint(), again.fingerprint());
    }

    #[test]
    fn a_full_lattice_starts_and_runs_with_nowhere_to_move_or_clone() {
        let mut w = world(4, |c| c.agents = 16);
        assert_eq!(w.population(), 16);
        empty_list_is_consistent(&w);
        w.run(20);
        assert!(w.population() <= 16);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_lone_agent_plays_no_one_in_space_or_soup() {
        for pairing in [Pairing::Space, Pairing::Soup] {
            let mut w = world(5, |c| c.pairing = pairing);
            let a = put(&mut w, (2, 2), false, 5.0);
            w.run(3);
            assert_eq!(w.population(), 1, "{pairing:?}");
            assert_eq!(w.agents[a].wealth, 5.0, "{pairing:?}: nobody to play");
            empty_list_is_consistent(&w);
        }
    }

    #[test]
    fn an_endowment_above_the_parents_wealth_kills_the_parent() {
        // The rules as written: the endowment comes from the parent, so a
        // parent at the threshold that gives more than it has goes negative.
        let mut w = world(5, |c| c.endowment = 20.0);
        put(&mut w, (2, 2), true, 11.0);
        w.step();
        assert_eq!(w.population(), 1, "the offspring lives, the parent dies");
        assert_eq!(w.agents[0].wealth, 20.0);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_zero_fission_wealth_fills_the_lattice_without_breaking_it() {
        let mut w = world(6, |c| {
            c.agents = 3;
            c.fission_wealth = 0.0;
            c.endowment = 0.0;
        });
        w.run(30);
        assert!(w.population() <= 36);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_vision_wider_than_the_lattice_reaches_each_site_once() {
        let mut w = world(3, |c| c.vision = 10);
        let a = put(&mut w, (1, 1), true, 5.0);
        w.step();
        assert_ne!(xy(&w, a), (1, 1), "eight empty sites are within reach");
        empty_list_is_consistent(&w);
    }
}
