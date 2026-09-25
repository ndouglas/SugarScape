//! The civil violence world: agents and cops on a torus, acting once a tick
//! in one random order (Epstein 2002's rules M, A and C), jail, and in
//! Model II killing, cloning and death by age.

use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{CivilConfig, Variant};
use super::math::exp_neg;
use super::stats::{CivilSnapshot, Outbursts, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::geometry::{Pos, Torus};
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, LENDER, RED};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

/// Cops: the paper's black, lightened for the dark grid.
pub const COP: Rgb = [0xe6, 0xe6, 0xe6];
/// Model II's second group.
pub const GREEN: Rgb = LENDER;

/// What is left of a jail term.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Term {
    Ticks(u32),
    Life,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Citizen {
    pub id: u64,
    /// Where it stands; while jailed, where it was arrested.
    pub pos: Pos,
    /// Perceived hardship H and risk aversion R, each U(0,1).
    pub hardship: f64,
    pub risk_aversion: f64,
    pub active: bool,
    pub jail: Option<Term>,
    /// Model II: Green (else Blue). Always false in Model I.
    pub green: bool,
    /// Model II: ticks lived and the age at which it dies (0 in Model I).
    pub age: u32,
    pub death_age: u32,
    alive: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cop {
    pub id: u64,
    pub pos: Pos,
}

/// Who stands on a site: an index into the agents or the cops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Occupant {
    Agent(u32),
    Cop(u32),
}

/// The color modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CivilMode {
    /// The paper's left screen: quiet blue (Model II: the group's color),
    /// active red.
    Action,
    /// The right screen: red by grievance.
    Grievance,
    /// Model II's groups.
    Group,
}

impl std::str::FromStr for CivilMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "action" => Self::Action,
            "grievance" => Self::Grievance,
            "group" => Self::Group,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// The sites within a Euclidean radius, as offsets, excluding the center:
/// `dx² + dy² ≤ radius²`, row by row (`dy`, then `dx`, ascending).
pub fn sight(radius: f64) -> Vec<(i32, i32)> {
    let r = radius.floor() as i32;
    let r2 = radius * radius;
    let mut out = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            if (dx, dy) != (0, 0) && f64::from(dx * dx + dy * dy) <= r2 {
                out.push((dx, dy));
            }
        }
    }
    out
}

/// What each site sees at one vision radius: its own index, then the sites
/// at `sight(radius)`'s offsets, for every site (computed once per radius).
#[derive(Clone)]
struct View {
    radius: f64,
    stride: usize,
    idx: Vec<u32>,
}

impl View {
    fn new(torus: Torus, radius: f64) -> Self {
        let offsets = sight(radius);
        let stride = offsets.len() + 1;
        let mut idx = Vec::with_capacity(torus.len() * stride);
        for i in 0..torus.len() {
            let p = torus.pos(i);
            idx.push(i as u32);
            for &(dx, dy) in &offsets {
                idx.push(torus.index(torus.offset(p, dx, dy)) as u32);
            }
        }
        View {
            radius,
            stride,
            idx,
        }
    }

    /// Site `i` itself, then the sites it sees.
    fn of(&self, i: usize) -> &[u32] {
        &self.idx[i * self.stride..(i + 1) * self.stride]
    }
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CivilInspection {
    pub site: SiteXy,
    pub agent: Option<CitizenView>,
    pub cop: Option<CopView>,
    /// Jailed agents arrested on this site (where they wait with
    /// `jailed_stay`, and where Inspect finds an agent it follows).
    pub jailed: Vec<CitizenView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SiteXy {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CopView {
    pub id: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CitizenView {
    pub id: u64,
    /// "quiet", "active" or "jailed".
    pub state: &'static str,
    pub hardship: f64,
    pub risk_aversion: f64,
    pub grievance: f64,
    /// The arrest probability P it would estimate here now, and N = R·P.
    pub arrest_probability: f64,
    pub net_risk: f64,
    /// Ticks of jail left, or null (free, or a term that never ends).
    pub jail_left: Option<u32>,
    pub jail_life: bool,
    /// Model II: "blue" or "green", its age and death age; null in Model I.
    pub group: Option<&'static str>,
    pub age: Option<u32>,
    pub death_age: Option<u32>,
}

/// Cloned for keyframes (`ModelWorld::checkpoint`).
#[derive(Clone)]
pub struct CivilWorld {
    pub config: CivilConfig,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    agents: Vec<Citizen>,
    cops: Vec<Cop>,
    /// Free agents and cops on each site (row-major); jailed agents are on
    /// no site.
    sites: Vec<Vec<Occupant>>,
    agent_view: View,
    cop_view: View,
    /// A reusable buffer for candidate sites.
    scratch: Vec<usize>,
    /// Each ramp's value when its start tick began (config order).
    ramp_base: Vec<Option<f64>>,
    rng: SimRng,
    next_id: u64,
    killed: u32,
    outbursts: Outbursts,
    extinct_at: Option<u64>,
    pub stats: Stats<CivilSnapshot>,
}

impl CivilWorld {
    pub fn new(config: CivilConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let torus = Torus::new(config.width, config.height);
        let mut w = CivilWorld {
            torus,
            tick: 0,
            agents: Vec::new(),
            cops: Vec::new(),
            sites: vec![Vec::new(); torus.len()],
            agent_view: View::new(torus, config.vision.agent),
            cop_view: View::new(torus, config.vision.cop),
            scratch: Vec::new(),
            ramp_base: vec![None; config.ramps.len()],
            rng: rng::seeded(seed),
            next_id: 1,
            killed: 0,
            outbursts: Outbursts::default(),
            extinct_at: None,
            stats: Stats::default(),
            config,
        };
        // Cops first, then agents, each on a random empty site: the first
        // sites of one shuffle.
        let mut cells: Vec<u32> = (0..torus.len() as u32).collect();
        cells.shuffle(&mut w.rng);
        let (nc, na) = (w.config.cops() as usize, w.config.agents() as usize);
        let cop_cells = cells[..nc].to_vec();
        for &i in &cells[nc..nc + na] {
            let a = w.newcomer(torus.pos(i as usize));
            w.agents.push(a);
        }
        for i in cop_cells {
            let id = w.take_id();
            w.cops.push(Cop {
                id,
                pos: torus.pos(i as usize),
            });
        }
        w.rebuild_sites();
        w.record();
        Ok(w)
    }

    fn take_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// A quiet, free agent at `pos` with H and R from U(0,1); in Model II
    /// Blue or Green with probability ½, age 0 and a death age uniform in
    /// 1..=max_age.
    fn newcomer(&mut self, pos: Pos) -> Citizen {
        let id = self.take_id();
        let hardship = self.rng.gen::<f64>();
        let risk_aversion = self.rng.gen::<f64>();
        let (green, death_age) = match self.config.variant {
            Variant::Rebellion => (false, 0),
            Variant::Ethnic => (
                self.rng.gen_bool(0.5),
                self.rng.gen_range(1..=self.config.max_age),
            ),
        };
        Citizen {
            id,
            pos,
            hardship,
            risk_aversion,
            active: false,
            jail: None,
            green,
            age: 0,
            death_age,
            alive: true,
        }
    }

    /// Model II: agent `i`'s clone at `pos`, keeping its group and H,
    /// drawing R and a death age.
    fn child(&mut self, i: usize, pos: Pos) -> Citizen {
        let id = self.take_id();
        let risk_aversion = self.rng.gen::<f64>();
        let death_age = self.rng.gen_range(1..=self.config.max_age);
        let parent = &self.agents[i];
        Citizen {
            id,
            pos,
            hardship: parent.hardship,
            risk_aversion,
            active: false,
            jail: None,
            green: parent.green,
            age: 0,
            death_age,
            alive: true,
        }
    }

    /// Files every free, living agent and every cop on its site, agents
    /// first, each in index order.
    fn rebuild_sites(&mut self) {
        for s in &mut self.sites {
            s.clear();
        }
        for (i, a) in self.agents.iter().enumerate() {
            if a.alive && a.jail.is_none() {
                self.sites[self.torus.index(a.pos)].push(Occupant::Agent(i as u32));
            }
        }
        for (j, c) in self.cops.iter().enumerate() {
            self.sites[self.torus.index(c.pos)].push(Occupant::Cop(j as u32));
        }
    }

    fn site(&self, pos: Pos) -> usize {
        self.torus.index(pos)
    }

    /// No free agent and no cop on it (jailed agents do not count).
    fn is_empty(&self, i: usize) -> bool {
        self.sites[i].is_empty()
    }

    fn leave(&mut self, i: usize, who: Occupant) {
        let list = &mut self.sites[i];
        let at = list.iter().position(|&o| o == who).expect("on its site");
        list.remove(at);
    }

    /// A uniformly random empty site among those site `i` sees with cop
    /// (`cop`) or agent vision, counting site `i` itself when `own`.
    fn empty_in_view(&mut self, i: usize, cop: bool, own: bool) -> Option<usize> {
        let mut buf = std::mem::take(&mut self.scratch);
        buf.clear();
        let view = if cop {
            &self.cop_view
        } else {
            &self.agent_view
        };
        let seen = view.of(i);
        let seen = if own { seen } else { &seen[1..] };
        buf.extend(
            seen.iter()
                .map(|&s| s as usize)
                .filter(|&s| self.sites[s].is_empty()),
        );
        let out = (!buf.is_empty()).then(|| buf[self.rng.gen_range(0..buf.len() as u32) as usize]);
        self.scratch = buf;
        out
    }

    /// A uniformly random empty site among `pos + offsets` (not `pos`).
    fn random_empty(&mut self, pos: Pos, offsets: &[(i32, i32)]) -> Option<usize> {
        let torus = self.torus;
        let empty: Vec<usize> = offsets
            .iter()
            .map(|&(dx, dy)| torus.index(torus.offset(pos, dx, dy)))
            .filter(|&i| self.is_empty(i))
            .collect();
        (!empty.is_empty()).then(|| empty[self.rng.gen_range(0..empty.len() as u32) as usize])
    }

    /// Rebuilds the vision tables whose radius changed.
    fn refresh_views(&mut self) {
        let v = self.config.vision;
        if v.agent != self.agent_view.radius {
            self.agent_view = View::new(self.torus, v.agent);
        }
        if v.cop != self.cop_view.radius {
            self.cop_view = View::new(self.torus, v.cop);
        }
    }

    /// A uniformly random empty site anywhere.
    fn random_empty_anywhere(&mut self) -> Option<usize> {
        let empty: Vec<usize> = (0..self.sites.len())
            .filter(|&i| self.is_empty(i))
            .collect();
        (!empty.is_empty()).then(|| empty[self.rng.gen_range(0..empty.len() as u32) as usize])
    }

    pub fn agents(&self) -> impl Iterator<Item = &Citizen> {
        self.agents.iter().filter(|a| a.alive)
    }

    pub fn cops(&self) -> &[Cop] {
        &self.cops
    }

    /// G = H(1 − L).
    pub fn grievance(&self, a: &Citizen) -> f64 {
        a.hardship * (1.0 - self.config.legitimacy)
    }

    /// The arrest probability agent `i` estimates where it stands:
    /// P = 1 − exp(−k·C/A), C the cops and A one plus the other active
    /// agents it sees (with `floor_ratio`, NetLogo's ⌊C/A⌋ with an active
    /// agent counted twice).
    fn arrest_probability(&self, i: usize) -> f64 {
        let a = &self.agents[i];
        let (mut cops, mut actives) = (0u32, 0u32);
        for &s in self.agent_view.of(self.site(a.pos)) {
            for &o in &self.sites[s as usize] {
                match o {
                    Occupant::Cop(_) => cops += 1,
                    Occupant::Agent(j) if j as usize != i && self.agents[j as usize].active => {
                        actives += 1
                    }
                    Occupant::Agent(_) => {}
                }
            }
        }
        let q = self.config.quirks;
        let mut a_count = 1 + actives;
        if q.active_counts_twice && a.active {
            a_count += 1;
        }
        let mut ratio = f64::from(cops) / f64::from(a_count);
        if q.floor_ratio {
            ratio = ratio.floor();
        }
        1.0 - exp_neg(-self.config.k * ratio)
    }

    /// Rule M for agent `i`.
    fn move_agent(&mut self, i: usize) {
        let from = self.agents[i].pos;
        if let Some(to) = self.empty_in_view(self.site(from), false, false) {
            let who = Occupant::Agent(i as u32);
            self.leave(self.site(from), who);
            self.sites[to].push(who);
            self.agents[i].pos = self.torus.pos(to);
        }
    }

    /// Rule M for cop `j`.
    fn move_cop(&mut self, j: usize) {
        let from = self.cops[j].pos;
        if let Some(to) = self.empty_in_view(self.site(from), true, false) {
            let who = Occupant::Cop(j as u32);
            self.leave(self.site(from), who);
            self.sites[to].push(who);
            self.cops[j].pos = self.torus.pos(to);
        }
    }

    /// Rule A for agent `i`: active iff G − R·P > T.
    fn decide(&mut self, i: usize) {
        let p = self.arrest_probability(i);
        let a = &self.agents[i];
        let active = self.grievance(a) - a.risk_aversion * p > self.config.threshold;
        self.agents[i].active = active;
    }

    /// Model II: an active agent kills one uniformly random free agent of
    /// the other group that it sees, if any.
    fn kill(&mut self, i: usize) {
        let a = &self.agents[i];
        let green = a.green;
        let targets: Vec<u32> = self
            .agent_view
            .of(self.site(a.pos))
            .iter()
            .flat_map(|&s| self.sites[s as usize].iter())
            .filter_map(|&o| match o {
                Occupant::Agent(j) if self.agents[j as usize].green != green => Some(j),
                _ => None,
            })
            .collect();
        if targets.is_empty() {
            return;
        }
        let victim = targets[self.rng.gen_range(0..targets.len() as u32) as usize] as usize;
        let at = self.site(self.agents[victim].pos);
        self.leave(at, Occupant::Agent(victim as u32));
        self.agents[victim].alive = false;
        self.killed += 1;
    }

    /// Rule C for cop `j`: arrest one uniformly random active agent it
    /// sees, if any.
    fn arrest(&mut self, j: usize) {
        let pos = self.cops[j].pos;
        let suspects: Vec<u32> = self
            .cop_view
            .of(self.site(pos))
            .iter()
            .flat_map(|&s| self.sites[s as usize].iter())
            .filter_map(|&o| match o {
                Occupant::Agent(i) if self.agents[i as usize].active => Some(i),
                _ => None,
            })
            .collect();
        if suspects.is_empty() {
            return;
        }
        let i = suspects[self.rng.gen_range(0..suspects.len() as u32) as usize] as usize;
        let term = self.jail_term();
        let at = self.agents[i].pos;
        self.leave(self.site(at), Occupant::Agent(i as u32));
        let a = &mut self.agents[i];
        a.active = false;
        a.jail = Some(term);
        if self.config.quirks.cop_moves_to_arrest {
            let who = Occupant::Cop(j as u32);
            self.leave(self.site(pos), who);
            let to = self.site(at);
            self.sites[to].push(who);
            self.cops[j].pos = at;
        }
    }

    /// ⌈U(0, J_max)⌉ ticks, i.e. uniform in 1..=J_max (NetLogo: 0..J_max),
    /// or a term that never ends.
    fn jail_term(&mut self) -> Term {
        let j = self.config.jail;
        if j.infinite {
            Term::Life
        } else if self.config.quirks.netlogo_jail_term {
            Term::Ticks(self.rng.gen_range(0..j.max))
        } else {
            Term::Ticks(self.rng.gen_range(1..=j.max))
        }
    }

    /// Every jailed agent's term counts down by one; those at 0 are
    /// released in index order — near where they were arrested, or where
    /// they stand with `jailed_stay`.
    ///
    /// This runs at the end of every tick, the arresting one included (as
    /// NetLogo's `go` does), so a term of n drawn at tick t frees the agent
    /// at the end of tick t + n − 1: a term of 1 releases it at the end of
    /// the tick it was arrested, before it acts again, and it is never
    /// counted as jailed (so terms 0 and 1 behave the same).
    fn serve_terms(&mut self) {
        for i in 0..self.agents.len() {
            let a = &mut self.agents[i];
            if !a.alive {
                continue;
            }
            let Some(Term::Ticks(left)) = a.jail else {
                continue;
            };
            // The arresting tick counts: a term of 1 ends here, the tick it began.
            let left = left.saturating_sub(1);
            a.jail = Some(Term::Ticks(left));
            if left == 0 {
                self.release(i);
            }
        }
    }

    /// Frees agent `i` (its term is served): with `jailed_stay` where it
    /// stands; otherwise on a random empty site among its arrest site and
    /// the sites it sees from there, else a random empty site anywhere, else
    /// it waits (still jailed, with 0 ticks left).
    fn release(&mut self, i: usize) {
        let pos = self.agents[i].pos;
        let to = if self.config.quirks.jailed_stay {
            Some(self.site(pos))
        } else {
            self.empty_in_view(self.site(pos), false, true)
                .or_else(|| self.random_empty_anywhere())
        };
        if let Some(to) = to {
            self.sites[to].push(Occupant::Agent(i as u32));
            let a = &mut self.agents[i];
            a.jail = None;
            a.pos = self.torus.pos(to);
        }
    }

    /// Model II: in a random order each free agent clones with probability
    /// p onto a random empty Moore neighbor; the child keeps its group and
    /// H, draws R and a death age. Then every agent but the newborn ages,
    /// and dies on reaching its death age.
    fn demography(&mut self) {
        let mut order: Vec<usize> = (0..self.agents.len())
            .filter(|&i| self.agents[i].alive && self.agents[i].jail.is_none())
            .collect();
        order.shuffle(&mut self.rng);
        let born_from = self.agents.len();
        const MOORE: [(i32, i32); 8] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        let p = self.config.clone_probability;
        for i in order {
            if !self.rng.gen_bool(p) {
                continue;
            }
            let Some(to) = self.random_empty(self.agents[i].pos, &MOORE) else {
                continue;
            };
            let child = self.child(i, self.torus.pos(to));
            self.sites[to].push(Occupant::Agent(self.agents.len() as u32));
            self.agents.push(child);
        }
        for a in &mut self.agents[..born_from] {
            if !a.alive {
                continue;
            }
            a.age += 1;
            if a.age >= a.death_age {
                a.alive = false;
            }
        }
    }

    /// Applies schedule entries due at the tick about to run, then ramps.
    fn apply_changes(&mut self) {
        let t = self.tick;
        let due: Vec<_> = self
            .config
            .schedule
            .iter()
            .filter(|c| c.tick == t)
            .flat_map(|c| c.set.clone())
            .collect();
        for (path, value) in due {
            // Validation checks each entry as if set, so this cannot fail.
            let next = ModelConfig::Civil(self.config.clone()).with_path(&path, &value);
            debug_assert!(
                matches!(next, Ok(ModelConfig::Civil(_))),
                "a validated schedule entry failed to apply: {path} = {value}"
            );
            if let Ok(ModelConfig::Civil(next)) = next {
                self.config = next;
            }
        }
        for k in 0..self.config.ramps.len() {
            let r = self.config.ramps[k].clone();
            if t == r.start {
                self.ramp_base[k] = Some(self.config.number(&r.path));
            }
            if let Some(base) = self.ramp_base[k] {
                if t > r.start && t <= r.end {
                    let f = (t - r.start) as f64 / (r.end - r.start) as f64;
                    self.config.set_number(&r.path, base + (r.to - base) * f);
                }
            }
        }
        self.refresh_views();
    }

    /// Adds cops on random empty sites or removes random cops until there
    /// are round(cop density × sites) (as many as fit).
    fn sync_cops(&mut self) {
        let target = self.config.cops() as usize;
        if self.cops.len() == target {
            return;
        }
        while self.cops.len() > target {
            let j = self.rng.gen_range(0..self.cops.len() as u32) as usize;
            self.cops.remove(j);
        }
        let mut empty: Vec<usize> = (0..self.sites.len())
            .filter(|&i| self.is_empty(i))
            .collect();
        while self.cops.len() < target && !empty.is_empty() {
            let k = self.rng.gen_range(0..empty.len() as u32) as usize;
            let i = empty.swap_remove(k);
            let id = self.take_id();
            self.cops.push(Cop {
                id,
                pos: self.torus.pos(i),
            });
            // Filed now so the next pick sees it taken.
            self.sites[i].push(Occupant::Cop(u32::MAX));
        }
        self.rebuild_sites();
    }

    pub fn is_finished(&self) -> bool {
        self.config.variant == Variant::Ethnic
            && self.config.stop_at_extinction
            && self.extinct_at.is_some()
    }

    /// One tick.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        self.apply_changes();
        self.sync_cops();
        self.killed = 0;
        #[derive(Clone, Copy)]
        enum Actor {
            Agent(usize),
            Cop(usize),
        }
        let mut order: Vec<Actor> = (0..self.agents.len())
            .filter(|&i| self.agents[i].jail.is_none())
            .map(Actor::Agent)
            .chain((0..self.cops.len()).map(Actor::Cop))
            .collect();
        order.shuffle(&mut self.rng);
        let ethnic = self.config.variant == Variant::Ethnic;
        for actor in order {
            match actor {
                Actor::Agent(i) => {
                    let a = &self.agents[i];
                    if !a.alive || a.jail.is_some() {
                        continue;
                    }
                    if self.config.movement {
                        self.move_agent(i);
                    }
                    self.decide(i);
                    if ethnic && self.agents[i].active {
                        self.kill(i);
                    }
                }
                Actor::Cop(j) => {
                    self.move_cop(j);
                    self.arrest(j);
                }
            }
        }
        self.serve_terms();
        if ethnic {
            self.demography();
        }
        self.agents.retain(|a| a.alive);
        self.rebuild_sites();
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

    fn record(&mut self) {
        let mut s = CivilSnapshot {
            tick: self.tick,
            cops: self.cops.len() as u32,
            legitimacy: self.config.legitimacy,
            killed: self.killed,
            ..Default::default()
        };
        let (mut g, mut r, mut free) = (0.0, 0.0, 0u32);
        for a in self.agents() {
            s.population += 1;
            if a.green {
                s.green += 1;
            } else {
                s.blue += 1;
            }
            if a.jail.is_some() {
                s.jailed += 1;
                continue;
            }
            free += 1;
            g += self.grievance(a);
            r += a.risk_aversion;
            if a.active {
                s.active += 1;
            } else {
                s.quiet += 1;
            }
        }
        if self.config.variant == Variant::Rebellion {
            (s.blue, s.green) = (0, 0);
        }
        if free > 0 {
            let n = f64::from(free);
            s.mean_grievance = g / n;
            let quiet_share = f64::from(s.quiet) / n;
            s.tension = if r > 0.0 {
                (g / n) * quiet_share / (r / n)
            } else {
                0.0
            };
        }
        self.outbursts
            .record(self.tick, s.active, self.config.outburst_threshold);
        s.outbursts = self.outbursts.ended;
        s.mean_wait = self.outbursts.mean_wait();
        s.mean_activation = self.outbursts.mean_activation();
        if self.config.variant == Variant::Ethnic
            && self.extinct_at.is_none()
            && (s.blue == 0 || s.green == 0)
        {
            self.extinct_at = Some(self.tick);
        }
        s.extinction = self.extinct_at.unwrap_or(self.tick);
        self.stats.push(s);
    }

    fn color(&self, a: &Citizen, mode: CivilMode) -> Rgb {
        let group = if a.green { GREEN } else { BLUE };
        match mode {
            CivilMode::Action if a.active => RED,
            CivilMode::Action | CivilMode::Group => group,
            CivilMode::Grievance => lerp(BACKGROUND, RED, 0.15 + 0.85 * self.grievance(a)),
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<CivilInspection, String> {
        if x >= self.torus.width || y >= self.torus.height {
            return Err(format!("({x}, {y}) is outside the grid"));
        }
        let pos = Pos::new(x, y);
        let here = &self.sites[self.site(pos)];
        let agent = here.iter().find_map(|&o| match o {
            Occupant::Agent(i) => Some(i as usize),
            Occupant::Cop(_) => None,
        });
        let cop = here.iter().find_map(|&o| match o {
            Occupant::Cop(j) => Some(CopView {
                id: self.cops[j as usize].id,
            }),
            Occupant::Agent(_) => None,
        });
        let jailed = (0..self.agents.len())
            .filter(|&i| {
                let a = &self.agents[i];
                a.alive && a.jail.is_some() && a.pos == pos
            })
            .map(|i| self.view(i))
            .collect();
        Ok(CivilInspection {
            site: SiteXy { x, y },
            agent: agent.map(|i| self.view(i)),
            cop,
            jailed,
        })
    }

    fn view(&self, i: usize) -> CitizenView {
        let a = &self.agents[i];
        let p = if a.jail.is_none() {
            self.arrest_probability(i)
        } else {
            0.0
        };
        let ethnic = self.config.variant == Variant::Ethnic;
        CitizenView {
            id: a.id,
            state: match (a.jail, a.active) {
                (Some(_), _) => "jailed",
                (None, true) => "active",
                (None, false) => "quiet",
            },
            hardship: a.hardship,
            risk_aversion: a.risk_aversion,
            grievance: self.grievance(a),
            arrest_probability: p,
            net_risk: a.risk_aversion * p,
            jail_left: match a.jail {
                Some(Term::Ticks(t)) => Some(t),
                _ => None,
            },
            jail_life: a.jail == Some(Term::Life),
            group: ethnic.then_some(if a.green { "green" } else { "blue" }),
            age: ethnic.then_some(a.age),
            death_age: ethnic.then_some(a.death_age),
        }
    }
}

impl Model for CivilWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Civil(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        CivilWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents().count()
    }

    /// FNV-1a over the tick, every agent (id, site, H, R, state, group, age
    /// and death age), every cop (id, site) and the ramps' starting values.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for a in self.agents() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(a.hardship.to_bits());
            eat(a.risk_aversion.to_bits());
            let jail = match a.jail {
                None => 0,
                Some(Term::Ticks(t)) => 1 + u64::from(t),
                Some(Term::Life) => u64::MAX,
            };
            eat(jail);
            eat(u64::from(a.active) | u64::from(a.green) << 1);
            eat((u64::from(a.age) << 32) | u64::from(a.death_age));
        }
        for c in &self.cops {
            eat(c.id);
            eat((u64::from(c.pos.x) << 32) | u64::from(c.pos.y));
        }
        for b in &self.ramp_base {
            eat(b.map_or(u64::MAX, f64::to_bits));
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.torus.width, self.torus.height)
    }

    /// Empty sites dark; cops light gray; agents by `mode`. `layer` is
    /// ignored.
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: CivilMode = mode.parse()?;
        buf.resize(self.sites.len() * 4, 0);
        for (i, here) in self.sites.iter().enumerate() {
            let rgb = if here.iter().any(|o| matches!(o, Occupant::Cop(_))) {
                COP
            } else {
                match here.first() {
                    Some(&Occupant::Agent(a)) => self.color(&self.agents[a as usize], mode),
                    _ => BACKGROUND,
                }
            };
            buf[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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
        let mut out = String::from(
            "id,x,y,state,jail_left,hardship,risk_aversion,grievance,group,age,death_age\n",
        );
        let ethnic = self.config.variant == Variant::Ethnic;
        for (i, a) in self.agents.iter().enumerate() {
            if !a.alive {
                continue;
            }
            let v = self.view(i);
            let jail = match a.jail {
                None => String::new(),
                Some(Term::Ticks(t)) => t.to_string(),
                Some(Term::Life) => "life".to_string(),
            };
            let (group, age, death) = if ethnic {
                (
                    v.group.unwrap_or_default().to_string(),
                    a.age.to_string(),
                    a.death_age.to_string(),
                )
            } else {
                Default::default()
            };
            writeln!(
                out,
                "{},{},{},{},{},{},{},{},{},{},{}",
                a.id,
                a.pos.x,
                a.pos.y,
                v.state,
                jail,
                a.hardship,
                a.risk_aversion,
                v.grievance,
                group,
                age,
                death
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// An agent's site (while jailed, its arrest site) or a cop's.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        self.agents()
            .find(|a| a.id == id)
            .map(|a| a.pos)
            .or_else(|| self.cops.iter().find(|c| c.id == id).map(|c| c.pos))
            .map(|p| (p.x, p.y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Civil(next) = next else {
            return Err(wrong_model(ModelKind::Civil, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        self.refresh_views();
        self.sync_cops();
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civil::config::{Ramp, Vision};
    use crate::config::ScheduledChange;
    use serde_json::json;

    /// A 10 × 10 world with nobody on it, vision 1.7 (the Moore
    /// neighborhood), after `edit`.
    fn blank(edit: impl FnOnce(&mut CivilConfig)) -> CivilWorld {
        let mut c = CivilConfig {
            width: 10,
            height: 10,
            agent_density: 0.0,
            cop_density: 0.0,
            vision: Vision {
                agent: 1.7,
                cop: 1.7,
            },
            ..Default::default()
        };
        edit(&mut c);
        CivilWorld::new(c, 1).unwrap()
    }

    /// Puts a quiet, free agent with hardship `h` and risk aversion `r` at
    /// (x, y); returns its index.
    fn put_agent(w: &mut CivilWorld, x: u32, y: u32, h: f64, r: f64) -> usize {
        let id = w.take_id();
        w.agents.push(Citizen {
            id,
            pos: Pos::new(x, y),
            hardship: h,
            risk_aversion: r,
            active: false,
            jail: None,
            green: false,
            age: 0,
            death_age: 1000,
            alive: true,
        });
        w.rebuild_sites();
        w.agents.len() - 1
    }

    /// Puts a cop at (x, y), keeping the cop density in step so a tick
    /// neither adds nor removes cops; returns its index.
    fn put_cop(w: &mut CivilWorld, x: u32, y: u32) -> usize {
        let id = w.take_id();
        w.cops.push(Cop {
            id,
            pos: Pos::new(x, y),
        });
        w.config.cop_density = w.cops.len() as f64 / w.torus.len() as f64;
        w.rebuild_sites();
        w.cops.len() - 1
    }

    fn at(w: &CivilWorld, x: u32, y: u32) -> usize {
        w.torus.index(Pos::new(x, y))
    }

    #[test]
    fn vision_is_a_euclidean_radius() {
        assert_eq!(sight(1.0).len(), 4);
        assert_eq!(sight(1.5).len(), 8, "√2 ≤ 1.5");
        assert_eq!(sight(1.7).len(), 8, "the paper's 1.7: the eight neighbors");
        assert_eq!(sight(7.0).len(), 148);
        assert!(!sight(7.0).contains(&(0, 0)));
    }

    #[test]
    fn views_wrap_around_the_torus() {
        let w = blank(|_| {});
        let seen = w.agent_view.of(at(&w, 0, 0));
        assert_eq!(seen[0] as usize, at(&w, 0, 0), "its own site first");
        assert!(seen.contains(&(at(&w, 9, 9) as u32)));
        assert!(seen.contains(&(at(&w, 1, 9) as u32)));
        assert_eq!(seen.len(), 9);
    }

    #[test]
    fn the_arrest_probability_counts_cops_and_other_actives_in_view() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        assert_eq!(w.arrest_probability(me), 0.0, "no cops, no risk");
        put_cop(&mut w, 5, 6);
        put_cop(&mut w, 5, 8); // out of view
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64).exp())).abs() < 1e-12,
            "C = A = 1: {p}"
        );
        assert!((p - 0.9).abs() < 0.001, "the paper's calibration");
        for x in [4, 6] {
            let a = put_agent(&mut w, x, 5, 0.5, 0.5);
            w.agents[a].active = true;
        }
        let quiet = put_agent(&mut w, 4, 4, 0.5, 0.5);
        assert!(!w.agents[quiet].active, "quiet agents are not counted");
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64 / 3.0).exp())).abs() < 1e-12,
            "C/A = 1/3: {p}"
        );
        w.config.quirks.floor_ratio = true;
        assert_eq!(w.arrest_probability(me), 0.0, "⌊1/3⌋ = 0");
        w.config.quirks.floor_ratio = false;
        w.config.quirks.active_counts_twice = true;
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64 / 3.0).exp())).abs() < 1e-12,
            "a quiet agent once"
        );
        w.agents[me].active = true;
        let p = w.arrest_probability(me);
        assert!(
            (p - (1.0 - (-2.3f64 / 4.0).exp())).abs() < 1e-12,
            "an active one twice"
        );
    }

    #[test]
    fn an_agent_is_active_only_when_g_minus_n_exceeds_t() {
        let mut w = blank(|c| {
            c.legitimacy = 0.0;
            c.threshold = 0.25;
        });
        let at_t = put_agent(&mut w, 1, 1, 0.25, 1.0);
        let above = put_agent(&mut w, 5, 5, 0.250_001, 1.0);
        w.decide(at_t);
        w.decide(above);
        assert!(!w.agents[at_t].active, "G − N = T is quiet");
        assert!(w.agents[above].active);
        put_cop(&mut w, 5, 6);
        w.decide(above);
        assert!(!w.agents[above].active, "R·P now outweighs G");
    }

    #[test]
    fn agents_move_only_to_empty_sites_in_view() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        for (x, y) in [(4, 4), (5, 4), (6, 4), (4, 5), (6, 5), (4, 6), (5, 6)] {
            put_cop(&mut w, x, y);
        }
        w.move_agent(me);
        assert_eq!(w.agents[me].pos, Pos::new(6, 6), "the one empty site");
        for (x, y) in [(5, 5), (7, 5), (7, 6), (5, 7), (6, 7), (7, 7)] {
            put_cop(&mut w, x, y);
        }
        w.move_agent(me);
        assert_eq!(w.agents[me].pos, Pos::new(6, 6), "boxed in: it stays");
        let here = &w.sites[at(&w, 6, 6)];
        assert_eq!(here, &[Occupant::Agent(me as u32)]);
    }

    #[test]
    fn without_movement_agents_stay_but_cops_still_move() {
        let mut w = blank(|c| c.movement = false);
        let me = put_agent(&mut w, 2, 2, 0.1, 0.9);
        let cop = put_cop(&mut w, 7, 7);
        w.step();
        assert_eq!(w.agents[me].pos, Pos::new(2, 2));
        assert_ne!(w.cops[cop].pos, Pos::new(7, 7));
    }

    #[test]
    fn cops_arrest_one_active_agent_they_see() {
        let mut w = blank(|c| c.jail.max = 5);
        let cop = put_cop(&mut w, 0, 0);
        let near = put_agent(&mut w, 1, 1, 0.5, 0.5);
        let far = put_agent(&mut w, 3, 3, 0.5, 0.5);
        let quiet = put_agent(&mut w, 9, 9, 0.5, 0.5);
        w.agents[near].active = true;
        w.agents[far].active = true;
        w.arrest(cop);
        assert!(matches!(w.agents[near].jail, Some(Term::Ticks(1..=5))));
        assert!(!w.agents[near].active, "jailed agents are quiet");
        assert!(w.sites[at(&w, 1, 1)].is_empty(), "off the lattice");
        assert!(w.agents[far].jail.is_none() && w.agents[quiet].jail.is_none());
        assert_eq!(w.cops[cop].pos, Pos::new(0, 0), "the paper's cop stays put");
    }

    #[test]
    fn jail_terms_follow_the_rule_in_force() {
        let mut w = blank(|c| c.jail.max = 3);
        let draws = |w: &mut CivilWorld| {
            let mut seen: Vec<Term> = (0..300).map(|_| w.jail_term()).collect();
            seen.sort_by_key(|t| match t {
                Term::Ticks(n) => *n,
                Term::Life => u32::MAX,
            });
            seen.dedup();
            seen
        };
        assert_eq!(
            draws(&mut w),
            [Term::Ticks(1), Term::Ticks(2), Term::Ticks(3)]
        );
        w.config.quirks.netlogo_jail_term = true;
        assert_eq!(
            draws(&mut w),
            [Term::Ticks(0), Term::Ticks(1), Term::Ticks(2)]
        );
        w.config.jail.infinite = true;
        assert_eq!(draws(&mut w), [Term::Life]);
    }

    #[test]
    fn terms_count_down_and_agents_are_released_near_their_arrest() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        let who = Occupant::Agent(me as u32);
        w.leave(at(&w, 5, 5), who);
        w.agents[me].jail = Some(Term::Ticks(2));
        w.serve_terms();
        assert_eq!(w.agents[me].jail, Some(Term::Ticks(1)));
        w.serve_terms();
        assert_eq!(w.agents[me].jail, None);
        let p = w.agents[me].pos;
        assert!(
            p.x.abs_diff(5) <= 1 && p.y.abs_diff(5) <= 1,
            "near (5, 5): {p:?}"
        );
        assert!(w.sites[w.torus.index(p)].contains(&who));
    }

    #[test]
    fn released_agents_go_anywhere_when_their_arrest_site_is_crowded_and_wait_when_full() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        w.leave(at(&w, 5, 5), Occupant::Agent(me as u32));
        w.agents[me].jail = Some(Term::Ticks(1));
        for y in 4..=6 {
            for x in 4..=6 {
                put_cop(&mut w, x, y);
            }
        }
        w.serve_terms();
        let p = w.agents[me].pos;
        assert!(w.agents[me].jail.is_none());
        assert!(p.x.abs_diff(5) > 1 || p.y.abs_diff(5) > 1, "{p:?}");
        let mut full = blank(|_| {});
        let me = put_agent(&mut full, 0, 0, 0.5, 0.5);
        full.leave(0, Occupant::Agent(me as u32));
        full.agents[me].jail = Some(Term::Ticks(1));
        for y in 0..10 {
            for x in 0..10 {
                put_cop(&mut full, x, y);
            }
        }
        full.serve_terms();
        assert_eq!(full.agents[me].jail, Some(Term::Ticks(0)), "it waits");
    }

    #[test]
    fn netlogo_jails_keep_their_site_and_the_cop_steps_onto_it() {
        let mut w = blank(|c| {
            c.quirks.jailed_stay = true;
            c.quirks.cop_moves_to_arrest = true;
            c.jail.max = 1;
        });
        let cop = put_cop(&mut w, 5, 6);
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        w.agents[me].active = true;
        w.arrest(cop);
        assert_eq!(w.cops[cop].pos, Pos::new(5, 5));
        assert_eq!(w.agents[me].pos, Pos::new(5, 5), "jailed where it stood");
        assert!(w.sites[at(&w, 5, 6)].is_empty());
        w.serve_terms();
        assert!(w.agents[me].jail.is_none());
        let here = &w.sites[at(&w, 5, 5)];
        assert!(here.contains(&Occupant::Cop(cop as u32)));
        assert!(
            here.contains(&Occupant::Agent(me as u32)),
            "sharing with the cop"
        );
    }

    #[test]
    fn a_live_cop_density_adds_and_removes_cops() {
        let mut w = blank(|_| {});
        put_agent(&mut w, 0, 0, 0.5, 0.5);
        let mut next = w.config.clone();
        next.cop_density = 0.05;
        w.set_config(ModelConfig::Civil(next.clone())).unwrap();
        assert_eq!(w.cops.len(), 5);
        let mut sites: Vec<Pos> = w.cops.iter().map(|c| c.pos).collect();
        sites.sort();
        sites.dedup();
        assert_eq!(sites.len(), 5, "on distinct sites");
        assert!(!sites.contains(&Pos::new(0, 0)), "on empty sites");
        next.cop_density = 0.02;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        assert_eq!(w.cops.len(), 2);
    }

    #[test]
    fn a_full_lattice_takes_no_more_cops_until_a_site_frees() {
        let mut w = blank(|_| {});
        for y in 0..10 {
            for x in 0..10 {
                put_agent(&mut w, x, y, 0.1, 0.9);
            }
        }
        let mut next = w.config.clone();
        next.cop_density = 0.05;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        assert!(w.cops.is_empty(), "no empty site to put one on");
        w.agents.pop();
        w.rebuild_sites();
        w.step();
        assert_eq!(w.cops.len(), 1, "the one freed site");
    }

    #[test]
    fn a_live_vision_change_rebuilds_what_agents_see() {
        let mut w = blank(|_| {});
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        put_cop(&mut w, 5, 7);
        assert_eq!(
            w.arrest_probability(me),
            0.0,
            "two sites away: out of sight"
        );
        let mut next = w.config.clone();
        next.vision.agent = 2.0;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        assert_eq!(w.agent_view.of(0).len(), 13, "itself and 12 sites");
        assert!(w.arrest_probability(me) > 0.8);
    }

    #[test]
    fn life_terms_outlast_a_change_of_the_jail_rule() {
        let mut w = blank(|c| c.jail.infinite = true);
        let me = put_agent(&mut w, 5, 5, 0.5, 0.5);
        w.leave(at(&w, 5, 5), Occupant::Agent(me as u32));
        w.agents[me].jail = Some(Term::Life);
        let mut next = w.config.clone();
        next.jail.infinite = false;
        w.set_config(ModelConfig::Civil(next)).unwrap();
        for _ in 0..40 {
            w.serve_terms();
        }
        assert_eq!(w.agents[me].jail, Some(Term::Life));
    }

    #[test]
    fn schedules_apply_before_ramps_and_ramps_move_linearly() {
        let mut w = blank(|c| {
            c.legitimacy = 0.9;
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("legitimacy".to_string(), json!(0.5))]
                    .into_iter()
                    .collect(),
            }];
            c.ramps = vec![Ramp {
                path: "legitimacy".into(),
                start: 2,
                end: 6,
                to: 0.1,
            }];
        });
        w.run(8);
        let l = w.stats.series("legitimacy").unwrap();
        let want = [0.9, 0.9, 0.9, 0.5, 0.4, 0.3, 0.2, 0.1, 0.1];
        for (t, (got, want)) in l.iter().zip(want).enumerate() {
            assert!((got - want).abs() < 1e-12, "tick {t}: {got} vs {want}");
        }
    }

    #[test]
    fn model_two_agents_kill_only_the_other_group_in_view() {
        let mut w = blank(|c| {
            c.variant = Variant::Ethnic;
            c.legitimacy = 0.0;
        });
        let killer = put_agent(&mut w, 5, 5, 1.0, 0.5);
        let friend = put_agent(&mut w, 4, 5, 0.0, 0.5);
        let victim = put_agent(&mut w, 5, 6, 0.0, 0.5);
        let far = put_agent(&mut w, 8, 8, 0.0, 0.5);
        for i in [victim, far] {
            w.agents[i].green = true;
        }
        w.decide(killer);
        assert!(w.agents[killer].active, "G = 1 > T with no cops");
        w.kill(killer);
        assert!(!w.agents[victim].alive);
        assert!(w.sites[at(&w, 5, 6)].is_empty());
        assert!(w.agents[friend].alive && w.agents[far].alive);
        assert_eq!(w.killed, 1);
    }

    #[test]
    fn model_two_agents_clone_onto_an_empty_neighbor_and_die_of_age() {
        let mut w = blank(|c| {
            c.variant = Variant::Ethnic;
            c.clone_probability = 1.0;
        });
        let parent = put_agent(&mut w, 5, 5, 0.3, 0.5);
        w.agents[parent].green = true;
        for (x, y) in [(5, 4), (6, 4), (4, 5), (6, 5), (4, 6), (5, 6), (6, 6)] {
            put_cop(&mut w, x, y);
        }
        let jailed = put_agent(&mut w, 0, 0, 0.5, 0.5);
        w.leave(0, Occupant::Agent(jailed as u32));
        w.agents[jailed].jail = Some(Term::Life);
        w.agents[jailed].death_age = 1;
        w.demography();
        let child = w.agents.last().unwrap();
        assert_eq!(child.pos, Pos::new(4, 4), "the one empty neighbor");
        assert!(child.green && child.hardship == 0.3 && child.age == 0);
        assert!(child.death_age >= 1 && child.death_age <= w.config.max_age);
        assert_eq!(w.agents[parent].age, 1);
        assert!(!w.agents[jailed].alive, "jailed agents age and die too");
    }

    #[test]
    fn extinction_is_recorded_and_can_stop_the_run() {
        let mut w = blank(|c| {
            c.variant = Variant::Ethnic;
            c.stop_at_extinction = true;
        });
        put_agent(&mut w, 1, 1, 0.1, 0.9);
        w.stats = Stats::default();
        w.record();
        assert_eq!(
            w.stats.latest().unwrap().extinction,
            0,
            "Green was never there"
        );
        assert!(w.is_finished());
        w.run(5);
        assert_eq!(w.tick, 0, "a finished world does not run");
        let mut one = blank(|_| {});
        put_agent(&mut one, 1, 1, 0.1, 0.9);
        one.run(3);
        assert!(!one.is_finished(), "Model I never finishes");
        assert_eq!(one.stats.latest().unwrap().extinction, 3);
    }

    #[test]
    fn model_one_keeps_its_population_and_worlds_follow_their_seed() {
        let c = CivilConfig::default();
        let mut a = CivilWorld::new(c.clone(), 7).unwrap();
        let mut b = CivilWorld::new(c.clone(), 7).unwrap();
        let mut other = CivilWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(40);
        }
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
        assert!(a
            .stats
            .series("population")
            .unwrap()
            .iter()
            .all(|&p| p == 1120.0));
        let s = a.stats.latest().unwrap();
        assert_eq!(s.active + s.quiet + s.jailed, 1120);
        assert_eq!(s.cops, 64);
    }

    #[test]
    fn frames_and_inspect_show_the_papers_left_screen() {
        let mut w = blank(|_| {});
        put_cop(&mut w, 1, 0);
        let me = put_agent(&mut w, 0, 0, 0.5, 0.25);
        w.agents[me].active = true;
        let mut buf = Vec::new();
        w.render("action", "", &mut buf).unwrap();
        assert_eq!(&buf[0..3], &RED);
        assert_eq!(&buf[4..7], &COP);
        assert_eq!(&buf[8..11], &BACKGROUND);
        w.render("grievance", "", &mut buf).unwrap();
        assert_eq!(&buf[0..3], &lerp(BACKGROUND, RED, 0.15 + 0.85 * 0.5 * 0.18));
        assert!(w.render("nope", "", &mut buf).is_err());
        let i = w.inspect(0, 0).unwrap();
        let a = i.agent.unwrap();
        assert_eq!((a.state, a.group, a.age), ("active", None, None));
        assert!((a.grievance - 0.09).abs() < 1e-12);
        assert!((a.net_risk - 0.25 * a.arrest_probability).abs() < 1e-12);
        assert!(i.cop.is_none());
        assert!(w.inspect(1, 0).unwrap().cop.is_some());
        assert!(w.inspect(10, 0).is_err());
        w.arrest(0);
        let i = w.inspect(0, 0).unwrap();
        assert!(i.agent.is_none(), "off the lattice");
        assert_eq!(i.jailed.len(), 1, "but listed where it was arrested");
        assert_eq!(i.jailed[0].state, "jailed");
        assert_eq!(
            w.locate(w.agents[me].id),
            Some((0, 0)),
            "Inspect keeps following it"
        );
    }
}
