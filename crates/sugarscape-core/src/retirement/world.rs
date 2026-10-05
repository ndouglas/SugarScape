//! The Timing of Retirement world. Each period, in activation order, each
//! agent ages a year (the pseudo-code: "select an agent … increment its
//! age"); one past its death age dies and a 20-year-old takes its place, and
//! every other agent who may retire decides: rationals at once, randoms by
//! chance, imitators once enough of their network has. Those not yet
//! activated this period are still a year younger.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{
    Counts, InitialDeaths, Order, Renewal, RetirementConfig, COHORTS, OLDEST, YOUNGEST,
};
use super::stats::{RetirementSnapshot, AGES_WINDOW};
use super::view::{
    population, row, scale, AGES_W, BAR, EMPTY, GAP, GROUP_A, GROUP_B, HIGH, IMITATOR, LINE, LOW,
    MARK, RANDOM, RATIONAL, RETIRED, ROW, SHOWN, TALL, TIME_W,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Thresholds are kept to six decimals and compared in integers.
const SCALE: u64 = 1_000_000;

/// An agent's type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Rational,
    Random,
    Imitator,
}

#[derive(Clone, Debug)]
pub struct Agent {
    /// The period of its birth at 20 (the initial agents: 0 − (age − 20)).
    pub born: i64,
    /// The last period it was activated (and aged).
    pub aged: u64,
    pub death: f64,
    pub kind: Kind,
    /// τ × 1 000 000.
    pub threshold: u64,
    pub extent: u32,
    pub group: u8,
    pub retired: bool,
    pub retired_at: Option<u32>,
    pub network: Vec<u32>,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RetirementMode {
    Status,
    Type,
    Threshold,
    Group,
}

impl std::str::FromStr for RetirementMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "status" => Self::Status,
            "type" => Self::Type,
            "threshold" => Self::Threshold,
            "group" => Self::Group,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// One neighbor at the actual activation-local decision time.
#[derive(Clone, Debug, Serialize)]
pub struct DecisionNeighbor {
    pub id: u32,
    pub born: i64,
    pub age: u32,
    pub eligible: bool,
    pub retired: bool,
    pub counted: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RetirementDecision {
    pub tick: u64,
    pub id: u32,
    pub born: i64,
    pub age: u32,
    pub eligibility: u32,
    pub counts: Counts,
    pub threshold_units: u64,
    pub threshold_scale: u64,
    pub neighbors: Vec<DecisionNeighbor>,
    pub counted: u64,
    pub retired_counted: u64,
    pub retired_before: bool,
    pub retired_after: bool,
}

/// Compact measurements; age-array index zero is age 20, last is age 100.
#[derive(Clone, Debug, Serialize)]
pub struct RetirementPeriod {
    pub tick: u64,
    pub eligibility: u32,
    pub decision_eligibility: u32,
    pub policy_switched: bool,
    pub retirements_by_age: Vec<u32>,
    pub working_exposure_by_age: Vec<u32>,
    pub retired: f64,
    pub retired_a: f64,
    pub retired_b: f64,
    pub imitator_retirements: u32,
    pub decision: Option<RetirementDecision>,
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RetirementInspection {
    pub site: RetirementCell,
    /// `population`, `ages` or `time`; null between panels.
    pub panel: Option<&'static str>,
    /// The age of a population row or an ages bar.
    pub age: Option<u32>,
    /// Ages: retirements and those who could have retired, over the last 10 periods.
    pub retirements: Option<u32>,
    pub exposed: Option<u32>,
    /// Time: the period and its share retired.
    pub period: Option<u64>,
    pub retired: Option<f64>,
    /// The agent at a population cell.
    pub member: Option<AgentView>,
    /// Always null: cells are read where they are.
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct RetirementCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub age: u32,
    pub kind: Kind,
    pub threshold: f64,
    pub death_age: f64,
    pub group: u8,
    pub network: u32,
    /// Members now eligible, and retired.
    pub eligible: u32,
    pub retired_members: u32,
    pub retired: bool,
    pub retired_at: Option<u32>,
}

#[derive(Clone)]
pub struct RetirementWorld {
    pub config: RetirementConfig,
    /// Completed periods.
    pub tick: u64,
    rng: SimRng,
    agents: Vec<Agent>,
    /// Slots by birth period, in order of birth.
    cohorts: BTreeMap<i64, Vec<u32>>,
    /// Each cohort's size at birth.
    born_size: BTreeMap<i64, u32>,
    /// Who holds each slot in its network.
    known_by: Vec<Vec<u32>>,
    eligibility: u32,
    transition: Option<u64>,
    switched_at: Option<u64>,
    transition_new: Option<u64>,
    /// The period each group reached the norm.
    transition_group: [Option<u64>; 2],
    /// The last periods' share retired, and whether the policy switched there.
    recent: VecDeque<(f64, bool)>,
    /// The last periods' retirements and exposures by age (index age − 20).
    ages: VecDeque<(Vec<u32>, Vec<u32>)>,
    pub stats: Stats<RetirementSnapshot>,
}

impl RetirementWorld {
    pub fn new(config: RetirementConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let c = config.per_cohort;
        let mut world = RetirementWorld {
            eligibility: config.eligibility,
            config,
            tick: 0,
            rng: rng::seeded(seed),
            agents: Vec::new(),
            cohorts: BTreeMap::new(),
            born_size: BTreeMap::new(),
            known_by: Vec::new(),
            transition: None,
            switched_at: None,
            transition_new: None,
            transition_group: [None, None],
            recent: VecDeque::new(),
            ages: VecDeque::new(),
            stats: Stats::default(),
        };
        for k in 0..COHORTS {
            let born = -i64::from(k);
            for m in 0..c {
                let age = YOUNGEST + k;
                let group = if world.config.groups.enabled {
                    (m % 2) as u8
                } else {
                    0
                };
                let from = match world.config.initial_deaths {
                    InitialDeaths::Literal => 60.0,
                    InitialDeaths::Survivors => f64::from(age.max(60)),
                };
                let agent = world.newborn(born, group, from);
                let slot = world.agents.len() as u32;
                world.agents.push(agent);
                world.cohorts.entry(born).or_default().push(slot);
            }
            world.born_size.insert(born, c);
        }
        world.known_by = vec![Vec::new(); world.agents.len()];
        for i in 0..world.agents.len() {
            world.draw_network(i);
        }
        world.remember(false);
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Agent `i`'s age now (a year less until it is activated this period).
    pub fn age(&self, i: usize) -> u32 {
        let a = &self.agents[i];
        (i64::from(YOUNGEST) + a.aged as i64 - a.born) as u32
    }

    pub fn eligibility(&self) -> u32 {
        self.eligibility
    }

    pub fn transition(&self) -> Option<u64> {
        self.transition
    }

    pub fn transition_new(&self) -> Option<u64> {
        self.transition_new
    }

    pub fn is_finished(&self) -> bool {
        let c = &self.config;
        (c.stop_at > 0 && self.tick >= u64::from(c.stop_at))
            || (c.stop_at_norm
                && if c.policy.enabled {
                    self.transition_new.is_some()
                } else {
                    self.transition.is_some()
                })
    }

    /// A new agent born at `born` in `group`, its death age drawn from
    /// U[`from`, 100].
    fn newborn(&mut self, born: i64, group: u8, from: f64) -> Agent {
        let c = &self.config;
        let death = from + (f64::from(OLDEST) - from) * self.rng.gen::<f64>();
        let u: f64 = self.rng.gen();
        let mut kind = if u < c.rational {
            Kind::Rational
        } else if u < c.rational + c.random {
            Kind::Random
        } else {
            Kind::Imitator
        };
        if c.groups.enabled && group == 0 && kind == Kind::Rational {
            // AE: "The 50 agents on the left do not include any rational agents".
            kind = Kind::Imitator;
        }
        let half = 3f64.sqrt() * c.spread;
        let tau = (c.threshold + half * (2.0 * self.rng.gen::<f64>() - 1.0)).clamp(0.0, 1.0);
        let extent = self.rng.gen_range(0..=c.extent);
        Agent {
            born,
            aged: born.max(0) as u64,
            death,
            kind,
            threshold: (tau * SCALE as f64).round() as u64,
            extent,
            group,
            retired: false,
            retired_at: None,
            network: Vec::new(),
        }
    }

    /// Slots within `extent` cohorts of `born`, in `group` (or the other
    /// group), excluding `me`.
    fn pool(&self, born: i64, extent: u32, me: usize, same: Option<(u8, bool)>) -> Vec<u32> {
        let e = i64::from(extent);
        self.cohorts
            .range(born - e..=born + e)
            .flat_map(|(_, v)| v.iter().copied())
            .filter(|&j| {
                j as usize != me
                    && same.is_none_or(|(g, own)| (self.agents[j as usize].group == g) == own)
            })
            .collect()
    }

    /// Draws agent `i`'s network: S distinct members within its extent (with
    /// groups, each from the other group with probability `coupling`).
    fn draw_network(&mut self, i: usize) {
        let (lo, hi) = (self.config.size.min, self.config.size.max);
        let s = self.rng.gen_range(lo..=hi) as usize;
        let (born, extent, group) = (
            self.agents[i].born,
            self.agents[i].extent,
            self.agents[i].group,
        );
        let groups = self.config.groups.enabled;
        let mut own = self.pool(born, extent, i, groups.then_some((group, true)));
        let mut other = if groups {
            self.pool(born, extent, i, Some((group, false)))
        } else {
            Vec::new()
        };
        let mut net = Vec::with_capacity(s);
        for _ in 0..s {
            let cross = groups && self.rng.gen::<f64>() < self.config.groups.coupling;
            let from = if (cross && !other.is_empty()) || own.is_empty() {
                &mut other
            } else {
                &mut own
            };
            if from.is_empty() {
                break;
            }
            let k = self.rng.gen_range(0..from.len() as u32) as usize;
            net.push(from.swap_remove(k));
        }
        for &m in &net {
            self.known_by[m as usize].push(i as u32);
        }
        self.agents[i].network = net;
    }

    /// Agent `i` dies; a 20-year-old takes its slot.
    fn die(&mut self, i: usize) {
        let old = self.agents[i].born;
        if let Some(v) = self.cohorts.get_mut(&old) {
            v.retain(|&j| j as usize != i);
            if v.is_empty() {
                self.cohorts.remove(&old);
            }
        }
        for m in std::mem::take(&mut self.agents[i].network) {
            self.known_by[m as usize].retain(|&h| h as usize != i);
        }
        let group = self.agents[i].group;
        let born = self.tick as i64;
        self.agents[i] = self.newborn(born, group, 60.0);
        self.cohorts.entry(born).or_default().push(i as u32);
        *self.born_size.entry(born).or_default() += 1;
        if self.config.renewal == Renewal::Replace {
            // Replace within each holder's extent, preserving the friend's group category.
            for h in std::mem::take(&mut self.known_by[i]) {
                let h = h as usize;
                let (hb, he, hg) = (
                    self.agents[h].born,
                    self.agents[h].extent,
                    self.agents[h].group,
                );
                let same = self.config.groups.enabled.then_some((hg, group == hg));
                let pool: Vec<u32> = self
                    .pool(hb, he, h, same)
                    .into_iter()
                    .filter(|&k| k as usize != i && !self.agents[h].network.contains(&k))
                    .collect();
                let pick = if pool.is_empty() {
                    None
                } else {
                    Some(pool[self.rng.gen_range(0..pool.len() as u32) as usize])
                };
                let net = &mut self.agents[h].network;
                if let Some(k) = pick {
                    for m in net.iter_mut().filter(|m| **m as usize == i) {
                        *m = k;
                    }
                    self.known_by[k as usize].push(h as u32);
                } else {
                    net.retain(|&m| m as usize != i);
                }
            }
        }
        self.draw_network(i);
    }

    fn retire(&mut self, i: usize, age: u32, log: &mut [u32]) {
        self.agents[i].retired = true;
        self.agents[i].retired_at = Some(age);
        log[(age - YOUNGEST) as usize] += 1;
    }

    /// Whether imitator `i` sees enough of its network retired.
    fn imitates(&self, i: usize) -> bool {
        let (mut counted, mut retired) = (0u64, 0u64);
        for &m in &self.agents[i].network {
            let m = m as usize;
            if self.config.counts == Counts::Eligible && self.age(m) < self.eligibility {
                continue;
            }
            counted += 1;
            if self.agents[m].retired {
                retired += 1;
            }
        }
        counted > 0 && retired * SCALE >= self.agents[i].threshold * counted
    }

    pub fn step(&mut self) {
        self.advance(false);
    }

    pub fn step_recorded(&mut self, teaching: bool) -> RetirementPeriod {
        self.advance(teaching)
    }

    pub fn initial_period(&self) -> RetirementPeriod {
        let latest = self.stats.latest().expect("initial statistics");
        RetirementPeriod {
            tick: self.tick,
            eligibility: self.eligibility,
            decision_eligibility: self.eligibility,
            policy_switched: false,
            retirements_by_age: vec![0; COHORTS as usize],
            working_exposure_by_age: vec![0; COHORTS as usize],
            retired: latest.retired,
            retired_a: latest.retired_a,
            retired_b: latest.retired_b,
            imitator_retirements: 0,
            decision: None,
        }
    }

    fn decision(&self, i: usize) -> RetirementDecision {
        let a = &self.agents[i];
        let neighbors: Vec<_> = a
            .network
            .iter()
            .map(|&id| {
                let m = &self.agents[id as usize];
                let age = self.age(id as usize);
                let eligible = age >= self.eligibility;
                DecisionNeighbor {
                    id,
                    born: m.born,
                    age,
                    eligible,
                    retired: m.retired,
                    counted: self.config.counts == Counts::All || eligible,
                }
            })
            .collect();
        RetirementDecision {
            tick: self.tick,
            id: i as u32,
            born: a.born,
            age: self.age(i),
            eligibility: self.eligibility,
            counts: self.config.counts,
            threshold_units: a.threshold,
            threshold_scale: SCALE,
            counted: neighbors.iter().filter(|m| m.counted).count() as u64,
            retired_counted: neighbors.iter().filter(|m| m.counted && m.retired).count() as u64,
            neighbors,
            retired_before: a.retired,
            retired_after: a.retired,
        }
    }

    fn advance(&mut self, teaching: bool) -> RetirementPeriod {
        let decision_eligibility = self.eligibility;
        let mut decision: Option<RetirementDecision> = None;
        let mut imitator_retirements = 0;
        self.tick += 1;
        let n = self.agents.len();
        let mut order: Vec<u32> = (0..n as u32).collect();
        for i in (1..n).rev() {
            let j = self.rng.gen_range(0..=i as u32) as usize;
            order.swap(i, j);
        }
        if self.config.order == Order::ByCohort {
            // Oldest cohorts first; the shuffle's order kept within each.
            let born: Vec<i64> = self.agents.iter().map(|a| a.born).collect();
            order.sort_by_key(|&i| born[i as usize]);
        }
        let mut retirements = vec![0u32; COHORTS as usize];
        let mut exposed = vec![0u32; COHORTS as usize];
        for i in order {
            let i = i as usize;
            self.agents[i].aged = self.tick;
            let age = self.age(i);
            if f64::from(age) >= self.agents[i].death || age > OLDEST {
                self.die(i);
                continue;
            }
            if self.agents[i].retired {
                continue;
            }
            let c = &self.config;
            if c.mandatory > 0 && age >= c.mandatory {
                exposed[(age - YOUNGEST) as usize] += 1;
                if self.agents[i].kind == Kind::Imitator {
                    imitator_retirements += 1;
                }
                self.retire(i, age, &mut retirements);
                continue;
            }
            if age < self.eligibility {
                continue;
            }
            exposed[(age - YOUNGEST) as usize] += 1;
            let capture = teaching
                && self.agents[i].kind == Kind::Imitator
                && decision.as_ref().is_none_or(|d| d.counted == 0);
            if capture
                && (decision.is_none()
                    || self.agents[i].network.iter().any(|&m| {
                        self.config.counts == Counts::All
                            || self.age(m as usize) >= self.eligibility
                    }))
            {
                decision = Some(self.decision(i));
            }
            let go = match self.agents[i].kind {
                Kind::Rational => true,
                Kind::Random => self.rng.gen::<f64>() < self.config.p,
                Kind::Imitator => self.imitates(i),
            };
            if go {
                if self.agents[i].kind == Kind::Imitator {
                    imitator_retirements += 1;
                }
                self.retire(i, age, &mut retirements);
            }
            if let Some(d) = decision.as_mut().filter(|d| d.id == i as u32) {
                d.retired_after = self.agents[i].retired;
            }
        }
        if self.ages.len() == AGES_WINDOW {
            self.ages.pop_front();
        }
        self.ages.push_back((retirements.clone(), exposed.clone()));
        let share = self.share(None);
        if self.config.groups.enabled && self.switched_at.is_none() {
            for g in 0..2u8 {
                if self.transition_group[g as usize].is_none()
                    && self.share(Some(g)) >= self.config.norm
                {
                    self.transition_group[g as usize] = Some(self.tick);
                }
            }
        }
        let mut switched = false;
        if self.transition.is_none() {
            if share >= self.config.norm {
                self.transition = Some(self.tick);
                if self.config.policy.enabled {
                    self.eligibility = self.config.policy.to;
                    self.switched_at = Some(self.tick);
                    switched = true;
                }
            }
        } else if self.config.policy.enabled && self.switched_at.is_none() {
            // The policy turned on after the norm: it switches now.
            self.eligibility = self.config.policy.to;
            self.switched_at = Some(self.tick);
            switched = true;
        } else if let (Some(at), None) = (self.switched_at, self.transition_new) {
            if share >= self.config.norm {
                self.transition_new = Some(self.tick - at);
            }
        }
        self.remember(switched);
        self.record();
        let latest = self.stats.latest().unwrap();
        RetirementPeriod {
            tick: self.tick,
            eligibility: self.eligibility,
            decision_eligibility,
            policy_switched: switched,
            retirements_by_age: retirements,
            working_exposure_by_age: exposed,
            retired: latest.retired,
            retired_a: latest.retired_a,
            retired_b: latest.retired_b,
            imitator_retirements,
            decision,
        }
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// The share of eligible agents retired (in `group`, if given).
    fn share(&self, group: Option<u8>) -> f64 {
        let (mut eligible, mut retired) = (0u32, 0u32);
        for i in 0..self.agents.len() {
            if group.is_some_and(|g| self.agents[i].group != g) || self.age(i) < self.eligibility {
                continue;
            }
            eligible += 1;
            if self.agents[i].retired {
                retired += 1;
            }
        }
        if eligible == 0 {
            0.0
        } else {
            f64::from(retired) / f64::from(eligible)
        }
    }

    fn remember(&mut self, switched: bool) {
        if self.recent.len() == SHOWN {
            self.recent.pop_front();
        }
        self.recent.push_back((self.share(None), switched));
    }

    fn record(&mut self) {
        let nan_or = |x: Option<u64>| x.map_or(f64::NAN, |v| v as f64);
        let mut counts = vec![0u32; COHORTS as usize];
        for (r, _) in &self.ages {
            for (k, v) in r.iter().enumerate() {
                counts[k] += v;
            }
        }
        let total: u32 = counts.iter().sum();
        let (modal, mean) = if total == 0 {
            (f64::NAN, f64::NAN)
        } else {
            let top = (0..counts.len())
                .max_by_key(|&k| (counts[k], std::cmp::Reverse(k)))
                .unwrap();
            let sum: u64 = counts
                .iter()
                .enumerate()
                .map(|(k, &v)| u64::from(v) * u64::from(YOUNGEST + k as u32))
                .sum();
            (
                f64::from(YOUNGEST + top as u32),
                sum as f64 / f64::from(total),
            )
        };
        let all = self.share(None);
        let groups = self.config.groups.enabled;
        let rationals = self
            .agents
            .iter()
            .filter(|a| a.kind == Kind::Rational)
            .count();
        let s = RetirementSnapshot {
            tick: self.tick,
            retired: all,
            retired_a: if groups { self.share(Some(0)) } else { all },
            retired_b: if groups { self.share(Some(1)) } else { all },
            transition: nan_or(self.transition),
            transition_new: nan_or(self.transition_new),
            transition_a: nan_or(if groups {
                self.transition_group[0]
            } else {
                self.transition
            }),
            transition_b: nan_or(if groups {
                self.transition_group[1]
            } else {
                self.transition
            }),
            modal_age: modal,
            mean_age: mean,
            rational_share: rationals as f64 / self.agents.len() as f64,
            eligibility: self.eligibility,
        };
        self.stats.push(s);
    }

    fn view(&self, i: usize) -> AgentView {
        let a = &self.agents[i];
        let (mut eligible, mut retired) = (0, 0);
        for &m in &a.network {
            if self.age(m as usize) >= self.eligibility {
                eligible += 1;
            }
            if self.agents[m as usize].retired {
                retired += 1;
            }
        }
        AgentView {
            id: i as u64 + 1,
            age: self.age(i),
            kind: a.kind,
            threshold: a.threshold as f64 / SCALE as f64,
            death_age: a.death,
            group: a.group,
            network: a.network.len() as u32,
            eligible,
            retired_members: retired,
            retired: a.retired,
            retired_at: a.retired_at,
        }
    }

    fn color(&self, mode: RetirementMode, i: usize) -> [u8; 3] {
        let a = &self.agents[i];
        let kind = match a.kind {
            Kind::Rational => RATIONAL,
            Kind::Imitator => IMITATOR,
            Kind::Random => RANDOM,
        };
        match mode {
            RetirementMode::Status => {
                if a.retired {
                    RETIRED
                } else {
                    kind
                }
            }
            RetirementMode::Type => kind,
            RetirementMode::Threshold => scale(a.threshold as f64 / SCALE as f64, LOW, HIGH),
            RetirementMode::Group => {
                if a.group == 0 {
                    GROUP_A
                } else {
                    GROUP_B
                }
            }
        }
    }

    /// The population panel's width, and where the other two panels start.
    fn layout(&self) -> (usize, usize, usize) {
        let (cols, cell) = population(self.config.per_cohort);
        let pop = cols * cell;
        let ages_x = pop + GAP;
        (pop, ages_x, ages_x + AGES_W + GAP)
    }

    /// The hazard of retiring at each age over the last `AGES_WINDOW` periods.
    fn hazards(&self) -> (Vec<u32>, Vec<u32>) {
        let mut r = vec![0u32; COHORTS as usize];
        let mut e = vec![0u32; COHORTS as usize];
        for (rs, es) in &self.ages {
            for k in 0..COHORTS as usize {
                r[k] += rs[k];
                e[k] += es[k];
            }
        }
        (r, e)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<RetirementInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let (pop, ages_x, time_x) = self.layout();
        let age = YOUNGEST + (cy / ROW) as u32;
        let mut out = RetirementInspection {
            site: RetirementCell { x, y },
            panel: None,
            age: None,
            retirements: None,
            exposed: None,
            period: None,
            retired: None,
            member: None,
            agent: None,
        };
        if cx < pop {
            out.panel = Some("population");
            out.age = Some(age);
            let (_, cell) = population(self.config.per_cohort);
            let born = self.tick as i64 - i64::from(age - YOUNGEST);
            if let Some(&slot) = self.cohorts.get(&born).and_then(|v| v.get(cx / cell)) {
                out.member = Some(self.view(slot as usize));
            }
        } else if (ages_x..ages_x + AGES_W).contains(&cx) {
            let (r, e) = self.hazards();
            let k = (age - YOUNGEST) as usize;
            out.panel = Some("ages");
            out.age = Some(age);
            out.retirements = Some(r[k]);
            out.exposed = Some(e[k]);
        } else if cx >= time_x {
            if let Some(&(share, _)) = self.recent.get(cx - time_x) {
                out.panel = Some("time");
                out.period = Some(self.tick + 1 + (cx - time_x) as u64 - self.recent.len() as u64);
                out.retired = Some(share);
            }
        }
        Ok(out)
    }
}

impl Model for RetirementWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Retirement(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        RetirementWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick, the eligibility and every agent's state.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.eligibility.to_le_bytes());
        for a in &self.agents {
            eat(&a.born.to_le_bytes());
            eat(&a.death.to_bits().to_le_bytes());
            eat(&a.threshold.to_le_bytes());
            eat(&[a.kind as u8, a.group, u8::from(a.retired), a.extent as u8]);
            for m in &a.network {
                eat(&m.to_le_bytes());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (_, _, time_x) = self.layout();
        ((time_x + TIME_W) as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: RetirementMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let (pop, ages_x, time_x) = self.layout();
        let (cols, cell) = population(self.config.per_cohort);
        // The population: one row per age, agents in order of birth; empty
        // places (the cohort's dead) light.
        for k in 0..COHORTS {
            let born = self.tick as i64 - i64::from(k);
            let members = self.cohorts.get(&born).map_or(&[][..], Vec::as_slice);
            let places = (*self.born_size.get(&born).unwrap_or(&0) as usize).min(cols);
            for col in 0..places.max(members.len().min(cols)) {
                let color = members
                    .get(col)
                    .map_or(EMPTY, |&i| self.color(mode, i as usize));
                for dy in 0..ROW {
                    for dx in 0..cell {
                        c.put(col * cell + dx, k as usize * ROW + dy, color);
                    }
                }
            }
        }
        let _ = pop;
        // Retirement by age: the hazard over the last periods; the
        // eligibility and mandatory ages marked.
        let (r, e) = self.hazards();
        for k in 0..COHORTS as usize {
            let hz = if e[k] == 0 {
                0.0
            } else {
                f64::from(r[k]) / f64::from(e[k])
            };
            let len = (hz * (AGES_W - 1) as f64).round() as usize;
            for x in 0..len {
                for dy in 0..ROW {
                    c.put(ages_x + x, k * ROW + dy, BAR);
                }
            }
        }
        for age in [self.eligibility, self.config.mandatory] {
            if (YOUNGEST..=OLDEST).contains(&age) {
                let y = (age - YOUNGEST) as usize * ROW;
                for x in 0..AGES_W {
                    c.put(ages_x + x, y, MARK);
                }
            }
        }
        // The share retired over time; the norm and the policy switch marked.
        let norm = row(self.config.norm);
        for x in 0..TIME_W {
            c.put(time_x + x, norm, MARK);
        }
        for (k, &(share, switched)) in self.recent.iter().enumerate() {
            if switched {
                c.column(time_x + k, 0, TALL - 1, MARK);
            }
            let y = row(share);
            let to = self.recent.get(k + 1).map_or(y, |next| row(next.0));
            c.column(time_x + k, y, to, LINE);
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        super::SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out =
            String::from("id,age,kind,threshold,death_age,group,network,eligible,retired_members,retired,retired_at\n");
        for i in 0..self.agents.len() {
            let v = self.view(i);
            writeln!(
                out,
                "{},{},{:?},{},{},{},{},{},{},{},{}",
                v.id,
                v.age,
                v.kind,
                v.threshold,
                v.death_age,
                v.group,
                v.network,
                v.eligible,
                v.retired_members,
                u8::from(v.retired),
                v.retired_at.map_or(String::new(), |a| a.to_string())
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Nothing to follow: Inspect reads a cell.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Retirement(next) = next else {
            return Err(wrong_model(ModelKind::Retirement, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        if self.switched_at.is_none() {
            self.eligibility = next.eligibility;
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped at the norm or at `stop_at`: a sweep reads its last period.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::retirement::config::{Groups, Policy, Size};

    fn config(edit: impl FnOnce(&mut RetirementConfig)) -> RetirementConfig {
        let mut c = RetirementConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut RetirementConfig)) -> RetirementWorld {
        RetirementWorld::new(config(edit), 1).unwrap()
    }

    /// The cohort and reverse indexes match the agents.
    fn consistent(w: &RetirementWorld) {
        let mut seen = 0;
        for (&born, v) in &w.cohorts {
            for &i in v {
                assert_eq!(w.agents[i as usize].born, born);
                seen += 1;
            }
        }
        assert_eq!(seen, w.agents.len());
        for (i, a) in w.agents.iter().enumerate() {
            let mut sorted = a.network.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), a.network.len(), "distinct members of {i}");
            assert!(!a.network.contains(&(i as u32)));
            for &m in &a.network {
                assert!(
                    w.known_by[m as usize].contains(&(i as u32)),
                    "{m} known by {i}"
                );
            }
        }
        for (m, holders) in w.known_by.iter().enumerate() {
            for &h in holders {
                assert!(w.agents[h as usize].network.contains(&(m as u32)));
            }
        }
    }

    #[test]
    fn the_population_is_81_cohorts_of_c_with_networks_in_range() {
        let w = world(|c| c.per_cohort = 20);
        assert_eq!(w.agents.len(), 81 * 20);
        assert_eq!((w.age(0), w.age(81 * 20 - 1)), (20, 100));
        for (i, a) in w.agents.iter().enumerate() {
            assert!(a.extent <= 5 && a.death >= 60.0 && a.death <= 100.0);
            assert!(a.network.len() <= 25);
            for &m in &a.network {
                assert!((w.agents[m as usize].born - a.born).unsigned_abs() <= u64::from(a.extent));
            }
            assert!(!a.retired, "{i} starts working");
        }
        consistent(&w);
        let kinds = |k: Kind| w.agents.iter().filter(|a| a.kind == k).count() as f64 / 1620.0;
        assert!((kinds(Kind::Rational) - 0.10).abs() < 0.03);
        assert!((kinds(Kind::Random) - 0.05).abs() < 0.02);
    }

    #[test]
    fn deaths_bring_20_year_olds_into_the_slot() {
        let mut w = world(|c| c.per_cohort = 20);
        w.step();
        // Everyone past a literal death age died in period 1.
        let newborns: Vec<usize> = (0..w.agents.len()).filter(|&i| w.age(i) == 20).collect();
        assert!(newborns.len() > 300, "{}", newborns.len());
        for &i in &newborns {
            assert_eq!(w.agents[i].born, 1);
            assert!(!w.agents[i].retired);
        }
        consistent(&w);
        let mut s = world(|c| {
            c.per_cohort = 20;
            c.initial_deaths = InitialDeaths::Survivors;
        });
        s.step();
        let fewer = (0..s.agents.len()).filter(|&i| s.age(i) == 20).count();
        assert!(fewer < newborns.len() / 3, "{fewer}");
    }

    #[test]
    fn each_type_follows_its_rule() {
        let mut all_rational = world(|c| {
            c.per_cohort = 10;
            c.rational = 1.0;
            c.random = 0.0;
        });
        all_rational.step();
        for i in 0..all_rational.agents.len() {
            let a = &all_rational.agents[i];
            assert_eq!(a.retired, all_rational.age(i) >= 65 && a.born < 1, "{i}");
        }
        let mut randoms = world(|c| {
            c.per_cohort = 100;
            c.rational = 0.0;
            c.random = 1.0;
        });
        randoms.step();
        let s = randoms.stats.latest().unwrap().retired;
        assert!((s - 0.5).abs() < 0.05, "{s}");
    }

    #[test]
    fn imitators_compare_retired_members_with_their_threshold_exactly() {
        let mut w = world(|c| {
            c.per_cohort = 10;
            c.rational = 0.0;
            c.random = 0.0;
        });
        // Agent 800 (age 100) with a hand network: 3 of 10 eligible members retired.
        let me = 80 * 10;
        let members: Vec<u32> = (70 * 10..70 * 10 + 10).collect();
        w.agents[me].network = members.clone();
        for (k, &m) in members.iter().enumerate() {
            w.agents[m as usize].retired = k < 3;
        }
        w.agents[me].threshold = 300_000;
        assert!(w.imitates(me), "3 of 10 reaches 0.3 exactly");
        w.agents[me].threshold = 300_001;
        assert!(!w.imitates(me));
        // Counting all members: two young members join the count.
        w.agents[me].network.extend([0, 1]);
        w.agents[me].threshold = 300_000;
        assert!(w.imitates(me), "eligible only: still 3 of 10");
        w.config.counts = Counts::All;
        assert!(!w.imitates(me), "all members: 3 of 12");
        w.agents[me].network.clear();
        assert!(!w.imitates(me), "nobody counted: no retirement");
    }

    #[test]
    fn the_base_case_reaches_the_norm_and_counting_all_members_never_does() {
        let mut w = world(|_| {});
        w.run(200);
        let t = w.transition().expect("the 65 norm");
        assert!((5..120).contains(&t), "{t}");
        let mut all = world(|c| c.counts = Counts::All);
        all.run(300);
        assert!(
            all.transition().is_none(),
            "footnote 5's alternative never sets in"
        );
    }

    #[test]
    fn replace_renewal_keeps_networks_full_and_indexes_right() {
        let mut w = world(|c| {
            c.per_cohort = 20;
            c.renewal = Renewal::Replace;
        });
        let sizes: Vec<usize> = w.agents.iter().map(|a| a.network.len()).collect();
        w.run(30);
        consistent(&w);
        // Nobody in a network is a newborn from a slot someone died in, unless
        // it was drawn there anew within the holder's extent.
        for a in &w.agents {
            for &m in &a.network {
                assert!((w.agents[m as usize].born - a.born).unsigned_abs() <= u64::from(a.extent));
            }
        }
        let _ = sizes;
        let mut s = world(|c| c.per_cohort = 20);
        s.run(30);
        consistent(&s);
        let stale = s
            .agents
            .iter()
            .flat_map(|a| a.network.iter().map(move |&m| (a, m)))
            .filter(|(a, m)| {
                (s.agents[*m as usize].born - a.born).unsigned_abs() > u64::from(a.extent)
            })
            .count();
        assert!(stale > 0, "slot renewal leaves newborns in old networks");
    }

    // Four hand-placed agents in one cohort; only holder 0 knows deceased 1.
    fn group_replacement_fixture(groups: [u8; 4], network: &[u32]) -> RetirementWorld {
        let mut w = world(|c| {
            c.per_cohort = 2;
            c.groups.enabled = true;
            c.renewal = Renewal::Replace;
        });
        w.agents.truncate(4);
        w.cohorts.clear();
        w.cohorts.insert(-10, vec![0, 1, 2, 3]);
        w.born_size.clear();
        w.born_size.insert(-10, 4);
        w.known_by = vec![Vec::new(); 4];
        for (i, a) in w.agents.iter_mut().enumerate() {
            a.born = -10;
            a.extent = 0;
            a.group = groups[i];
            a.network.clear();
        }
        w.agents[0].network = network.to_vec();
        for &m in network {
            w.known_by[m as usize].push(0);
        }
        consistent(&w);
        w
    }

    #[test]
    fn replace_renewal_preserves_opposite_group_friendship() {
        let mut w = group_replacement_fixture([0, 1, 1, 0], &[1]);
        w.die(1);
        assert_eq!(w.agents[0].network, vec![2]);
        consistent(&w);
    }

    #[test]
    fn replace_renewal_preserves_same_group_friendship() {
        let mut w = group_replacement_fixture([0, 0, 0, 1], &[1]);
        w.die(1);
        assert_eq!(w.agents[0].network, vec![2]);
        consistent(&w);
    }

    #[test]
    fn replace_renewal_removes_cross_group_edge_when_only_duplicate_matches() {
        let mut w = group_replacement_fixture([0, 1, 1, 0], &[1, 2]);
        w.die(1);
        assert_eq!(w.agents[0].network, vec![2]);
        consistent(&w);
    }

    #[test]
    fn replace_renewal_removes_same_group_edge_when_only_holder_matches() {
        let mut w = group_replacement_fixture([0, 0, 1, 1], &[1]);
        w.die(1);
        assert!(w.agents[0].network.is_empty());
        consistent(&w);
    }

    #[test]
    fn mandatory_retirement_and_the_policy_switch() {
        let mut w = world(|c| {
            c.rational = 0.05;
            c.random = 0.05;
            c.mandatory = 70;
            c.policy = Policy {
                enabled: true,
                to: 62,
            };
            c.stop_at_norm = true;
        });
        w.run(500);
        assert!(w.is_finished());
        let t = w.transition().unwrap();
        assert_eq!(w.eligibility(), 62);
        let t2 = w.transition_new().unwrap();
        assert!(
            t2 < 30,
            "the new norm in {t2} periods after the switch at {t}"
        );
        for i in 0..w.agents.len() {
            if w.age(i) >= 70 {
                assert!(w.agents[i].retired, "{i}");
            }
        }
    }

    #[test]
    fn mandatory_retirees_count_as_exposed_at_their_age() {
        let mut w = world(|c| c.mandatory = 70);
        w.run(3);
        let (r, e) = w.hazards();
        let k = (70 - YOUNGEST) as usize;
        assert!(r[k] > 0);
        assert!(e[k] >= r[k], "{} retired of {} exposed at 70", r[k], e[k]);
    }

    #[test]
    fn turning_the_policy_on_after_the_norm_switches_next_period() {
        let mut w = world(|c| c.stop_at_norm = true);
        w.run(500);
        let t = w.transition().unwrap();
        assert!(w.is_finished());
        let mut next = w.config.clone();
        next.policy.enabled = true;
        Model::set_config(&mut w, ModelConfig::Retirement(next)).unwrap();
        assert!(!w.is_finished());
        w.run(500);
        assert!(w.is_finished());
        assert_eq!(w.eligibility(), 62);
        assert!(w.transition_new().is_some());
        assert_eq!(w.transition(), Some(t));
    }

    #[test]
    fn groups_split_every_cohort_and_rationals_only_in_the_second() {
        let w = world(|c| {
            c.per_cohort = 40;
            c.groups = Groups {
                enabled: true,
                coupling: 0.2,
            };
        });
        assert!(w
            .agents
            .iter()
            .filter(|a| a.group == 0)
            .all(|a| a.kind != Kind::Rational));
        assert!(w
            .agents
            .iter()
            .any(|a| a.group == 1 && a.kind == Kind::Rational));
        let (mut cross, mut total) = (0, 0);
        for a in &w.agents {
            for &m in &a.network {
                total += 1;
                if w.agents[m as usize].group != a.group {
                    cross += 1;
                }
            }
        }
        let share = f64::from(cross) / f64::from(total);
        assert!((share - 0.2).abs() < 0.03, "{share}");
        consistent(&w);
        let mut w = w;
        w.run(300);
        let s = w.stats.latest().unwrap();
        assert!(
            s.transition_a.is_finite() && s.transition_b.is_finite(),
            "{s:?}"
        );
        assert!(
            s.transition_b <= s.transition_a,
            "the group with rationals first"
        );
    }

    #[test]
    fn statistics_track_shares_ages_and_the_transition() {
        let mut w = world(|c| c.rational = 0.2);
        w.run(40);
        let s = w.stats.latest().unwrap().clone();
        assert!(s.retired > 0.9);
        assert_eq!(s.transition, w.transition().unwrap() as f64);
        assert!(s.transition_new.is_nan());
        assert!(s.modal_age >= 65.0 && s.mean_age >= 65.0, "{s:?}");
        assert!((s.retired_a - s.retired).abs() < 1e-12);
        assert!(Model::latest_json(&w).contains("\"transition_new\":null"));
    }

    #[test]
    fn activation_order_is_by_cohort_or_shuffled() {
        let a = world(|c| c.per_cohort = 20);
        let b = world(|c| {
            c.per_cohort = 20;
            c.order = Order::Shuffled;
        });
        let (mut a, mut b) = (a, b);
        a.run(20);
        b.run(20);
        assert_ne!(Model::fingerprint(&a), Model::fingerprint(&b));
    }

    #[test]
    fn the_frame_draws_the_population_ages_and_time() {
        let mut w = world(|_| {});
        w.run(10);
        let mut buf = Vec::new();
        w.render("status", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw, fh), ((400 + 8 + 101 + 8 + 301) as u32, 162));
        for mode in ["type", "threshold", "group"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
    }

    #[test]
    fn inspect_reads_agents_ages_and_periods() {
        let mut w = world(|_| {});
        w.run(10);
        let a = w.inspect(0, (45 * ROW) as u32).unwrap();
        assert_eq!((a.panel, a.age), (Some("population"), Some(65)));
        let m = a.member.unwrap();
        assert_eq!(m.age, 65);
        assert!(m.network >= 10 || m.network == 0);
        let g = w.inspect(409, (45 * ROW) as u32).unwrap();
        assert_eq!(g.panel, Some("ages"));
        assert!(g.exposed.unwrap() > 0);
        let t = w.inspect(517 + 10, 0).unwrap();
        assert_eq!((t.panel, t.period), (Some("time"), Some(10)));
        assert_eq!(Model::locate(&w, 1), None);
    }

    #[test]
    fn keyframes_restore_the_world_and_its_view() {
        let c = config(|c| {
            c.per_cohort = 20;
            c.renewal = Renewal::Replace;
        });
        let mut any = crate::model::ModelWorld::new(ModelConfig::Retirement(c.clone()), 6).unwrap();
        any.model_mut().run(10);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("status", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("status", "", &mut after).unwrap();
        assert_eq!(before, after);
        any.model_mut().run(10);
        let mut fresh = crate::model::ModelWorld::new(ModelConfig::Retirement(c), 6).unwrap();
        fresh.model_mut().run(20);
        assert_eq!(
            any.model().fingerprint(),
            fresh.model().fingerprint(),
            "replays the same"
        );
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|c| c.per_cohort = 20);
        let next = config(|c| {
            c.per_cohort = 20;
            c.counts = Counts::All;
            c.mandatory = 70;
            c.stop_at = 5;
        });
        Model::set_config(&mut w, ModelConfig::Retirement(next)).unwrap();
        w.run(100);
        assert_eq!(w.tick, 5);
        for (field, edit) in [
            ("per_cohort", config(|c| c.per_cohort = 30)),
            (
                "renewal",
                config(|c| {
                    c.per_cohort = 20;
                    c.renewal = Renewal::Replace;
                }),
            ),
            (
                "size",
                config(|c| {
                    c.per_cohort = 20;
                    c.size = Size { min: 5, max: 6 };
                }),
            ),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Retirement(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.per_cohort = 1),
            config(|c| {
                c.per_cohort = 5;
                c.size = Size { min: 0, max: 0 };
            }),
            config(|c| {
                c.per_cohort = 5;
                c.extent = 0;
                c.renewal = Renewal::Replace;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.rational = 0.0;
                c.random = 0.0;
                c.spread = 0.5;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.groups = Groups {
                    enabled: true,
                    coupling: 1.0,
                };
                c.renewal = Renewal::Replace;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.eligibility = 20;
                c.mandatory = 20;
            }),
            config(|c| {
                c.per_cohort = 5;
                c.size = Size { min: 200, max: 200 };
            }),
        ] {
            let mut w = RetirementWorld::new(c.clone(), 1).unwrap();
            w.run(30);
            consistent(&w);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.retired), "{c:?}");
            let mut buf = Vec::new();
            w.render("status", "", &mut buf).unwrap();
        }
    }
    #[test]
    fn recorded_decision_uses_activation_local_ages_and_exact_equality() {
        let mut w = world(|c| {
            c.per_cohort = 1;
            c.rational = 0.0;
            c.random = 0.0;
        });
        // Oldest surviving slot activates first; its younger neighbors have not aged.
        for a in &mut w.agents {
            a.death = 101.0;
            a.network.clear();
        }
        w.known_by.iter_mut().for_each(Vec::clear);
        w.agents[79].network = vec![45, 46];
        w.agents[79].threshold = 500_000;
        w.agents[45].retired = true;
        let p = w.step_recorded(true);
        assert_eq!(
            (
                p.retirements_by_age[80],
                p.working_exposure_by_age[80],
                p.imitator_retirements
            ),
            (1, 1, 1)
        );
        let d = p.decision.unwrap();
        assert_eq!((d.id, d.age, d.counted, d.retired_counted), (79, 100, 2, 1));
        assert_eq!(
            d.neighbors.iter().map(|m| m.age).collect::<Vec<_>>(),
            vec![65, 66]
        );
        assert!(!d.retired_before && d.retired_after);
    }

    #[test]
    fn recorded_empty_denominator_never_retires_and_capture_does_not_change_rng() {
        let mut w = world(|c| {
            c.per_cohort = 1;
            c.rational = 0.0;
            c.random = 0.0;
        });
        for a in &mut w.agents {
            a.network.clear();
            a.threshold = 0;
        }
        let mut plain = w.clone();
        let p = w.step_recorded(true);
        plain.step();
        let d = p.decision.unwrap();
        assert_eq!(d.counted, 0);
        assert!(!d.retired_after);
        for _ in 0..20 {
            w.step_recorded(true);
            plain.step();
        }
        assert_eq!(w.fingerprint(), plain.fingerprint());
        assert_eq!(
            serde_json::to_value(w.stats.history()).unwrap(),
            serde_json::to_value(plain.stats.history()).unwrap()
        );
    }
    #[test]
    fn policy_export_timing_matches_audit_live_toggle_without_resetting_dynamics() {
        let c = config(|c| {
            c.per_cohort = 1;
            c.mandatory = 70;
        });
        let mut live = RetirementWorld::new(c.clone(), 1).unwrap();
        let mut audit = live.clone();
        for _ in 0..100 {
            live.step();
            audit.step();
        }
        let mut next = live.config.clone();
        next.policy.enabled = true;
        live.set_config(ModelConfig::Retirement(next)).unwrap();
        audit.config.policy.enabled = true;
        for _ in 0..101 {
            live.step_recorded(true);
            audit.step();
        }
        assert_eq!(live.fingerprint(), audit.fingerprint());
        assert_eq!(live.latest_json(), audit.latest_json());
    }

    #[test]
    fn teaching_without_any_eligible_imitator_is_null() {
        let mut w = world(|c| {
            c.per_cohort = 1;
            c.rational = 1.0;
            c.random = 0.0;
        });
        assert!(w.step_recorded(true).decision.is_none());
    }
}
