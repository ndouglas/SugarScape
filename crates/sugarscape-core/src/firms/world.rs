//! The Emergence of Firms world. Each tick is a period: a number of agent
//! activations, then production. An activated agent weighs staying in its
//! firm (re-choosing its effort), starting a firm alone, and joining each
//! firm its network shows it, and takes the best (A99 §3.1). At the
//! period's end every firm produces a·E + b·E^β and pays its members.

use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{
    Activation, AdjustScope, BasePay, CesSign, EffortSearch, FirmsConfig, Initial, Network,
    OthersEffort, Pay, Preferences, RandomBehavior, SeniorityOrder,
};
use super::effort::{best, Choice, Prefs, Search, Share, Tech};
use super::fit::{mu_mle, mu_ols, ols, Records};
use super::stats::FirmsSnapshot;
use super::view::{
    plot_at, scale, AXIS, CELL, COLS, FIRMS_H, FIRMS_W, FIT, FOUNDER, HIGH, LOW, MEMBER, PLOT,
    PLOT_X, POINT, ROWS, TALL, WIDE,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::portable::{exp_neg, ln};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// What an activated agent weighs: staying, a start-up (its choice, drawn
/// technology and hiring standard) and the firms it may join.
type Options = (Choice, Option<(Choice, Tech, f64)>, Vec<(usize, Choice)>);

/// No firm (an agent before its first production).
const NONE: u64 = u64::MAX;

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirmsMode {
    Founder,
    Theta,
    Effort,
    Income,
}

impl std::str::FromStr for FirmsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "founder" => Self::Founder,
            "theta" => Self::Theta,
            "effort" => Self::Effort,
            "income" => Self::Income,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub prefs: Prefs,
    pub effort: f64,
    /// The slot of the agent's firm.
    pub firm: usize,
    /// Effort and firm (its serial id) at the last production.
    pub last_effort: f64,
    pub last_firm: u64,
    pub friends: Vec<u32>,
    pub loyalty: u32,
    /// Times it wanted to move but stayed (loyalty).
    pub wants: u32,
    /// The period it joined its firm.
    pub joined: u64,
    /// Base pay (for `Pay::Base`).
    pub base: f64,
    pub income: f64,
    pub utility: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Firm {
    /// A serial number, never reused.
    pub id: u64,
    pub alive: bool,
    /// Members in order of joining; the first is the longest-serving.
    pub members: Vec<u32>,
    pub tech: Tech,
    pub hiring: f64,
    pub born: u64,
    /// Members' current efforts, summed; and at the last production.
    pub total: f64,
    pub last_total: f64,
    /// Size at the last production (0 if it had none).
    pub last_size: u32,
    /// The sum of its members' base pay.
    pub base_sum: f64,
    /// Whether it ever had a second member.
    pub team: bool,
    pub output: f64,
}

/// What Inspect shows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FirmsInspection {
    pub site: FirmsCell,
    pub panel: Option<&'static str>,
    pub firm: Option<FirmView>,
    /// The member at the cell (called `member` so no other model's view is
    /// mistaken for it); `agent` is always null.
    pub member: Option<AgentView>,
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct FirmsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FirmView {
    pub id: u64,
    pub size: u32,
    pub output: f64,
    pub age: u64,
    pub a: f64,
    pub b: f64,
    pub beta: f64,
    pub mean_theta: f64,
    pub mean_effort: f64,
    /// Members putting in no effort.
    pub free_riders: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u32,
    pub theta: f64,
    pub effort: f64,
    pub income: f64,
    pub utility: f64,
    pub tenure: u64,
    pub firm: u64,
}

#[derive(Clone)]
pub struct FirmsWorld {
    pub config: FirmsConfig,
    /// Periods run.
    pub tick: u64,
    rng: SimRng,
    agents: Vec<Agent>,
    firms: Vec<Firm>,
    free: Vec<usize>,
    /// Live firm slots, and each slot's position in it.
    live: Vec<usize>,
    pos: Vec<usize>,
    next_id: u64,
    births: u32,
    deaths: u32,
    records: Records,
    /// Count and sum of lifetimes recorded since the burn-in (the same
    /// deaths `records.lifetimes` sees), kept running so the `lifetime`
    /// series doesn't walk the whole histogram every period.
    lifetime_count: u64,
    lifetime_sum: u64,
    pub stats: Stats<FirmsSnapshot>,
}

/// A uniform draw between `lo` and `hi` when `hi` is above `lo`; else `lo`.
fn draw(rng: &mut SimRng, lo: f64, hi: f64) -> f64 {
    if hi > lo {
        lo + (hi - lo) * rng.gen::<f64>()
    } else {
        lo
    }
}

fn draw_u32(rng: &mut SimRng, lo: u32, hi: u32) -> u32 {
    if hi > lo {
        rng.gen_range(lo..=hi)
    } else {
        lo
    }
}

fn prefs_of(c: &FirmsConfig, rng: &mut SimRng) -> Prefs {
    let theta = match c.preferences {
        Preferences::Uniform => rng.gen::<f64>(),
        Preferences::Middle => 0.25 + 0.5 * rng.gen::<f64>(),
        Preferences::Triangular | Preferences::TriangularHigh => {
            let m = if c.preferences == Preferences::Triangular {
                0.5
            } else {
                0.75
            };
            let u: f64 = rng.gen();
            if u < m {
                (u * m).sqrt()
            } else {
                1.0 - ((1.0 - u) * (1.0 - m)).sqrt()
            }
        }
        Preferences::Normal => loop {
            // Rejection from U[0, 1] against the normal (0.5, variance ½).
            let (x, u): (f64, f64) = (rng.gen(), rng.gen());
            if u < exp_neg(-(x - 0.5) * (x - 0.5)) {
                break x;
            }
        },
        Preferences::Beta => 1.0 - (1.0 - rng.gen::<f64>()).sqrt(),
        Preferences::Fixed => c.theta,
        Preferences::Ces => {
            let delta = rng.gen::<f64>();
            let rho = draw(rng, c.rho, c.rho_max);
            return Prefs::Ces {
                delta,
                rho,
                minus: c.ces_sign == CesSign::Text,
            };
        }
    };
    Prefs::CobbDouglas { theta }
}

impl FirmsWorld {
    pub fn new(config: FirmsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.agents as usize;
        let base_tech = Tech {
            a: config.a,
            b: config.b,
            beta: config.beta,
        };
        let search = search_of(&config);
        let alone = |p: &Prefs| {
            best(
                p,
                &Choice {
                    tech: base_tech,
                    others: 0.0,
                    share: Share::Fraction(1.0),
                },
                0.0,
                1.0,
                search,
            )
        };
        let mut agents: Vec<Agent> = (0..n)
            .map(|_| {
                let prefs = prefs_of(&config, &mut rng);
                Agent {
                    prefs,
                    effort: 0.0,
                    firm: 0,
                    last_effort: 0.0,
                    last_firm: NONE,
                    friends: Vec::new(),
                    loyalty: 0,
                    wants: 0,
                    joined: 0,
                    base: 0.0,
                    income: 0.0,
                    utility: 0.0,
                }
            })
            .collect();
        for (i, agent) in agents.iter_mut().enumerate() {
            let k = draw_u32(&mut rng, config.neighbors, config.neighbors_max).min(n as u32 - 1);
            let mut friends = Vec::with_capacity(k as usize);
            while friends.len() < k as usize {
                let j = rng.gen_range(0..n as u32);
                if j as usize != i && !friends.contains(&j) {
                    friends.push(j);
                }
            }
            agent.friends = friends;
            agent.loyalty = draw_u32(&mut rng, config.loyalty, config.loyalty_max);
        }
        // Base pay: singleton income, own or the median or mean agent's.
        let singleton: Vec<f64> = agents
            .iter()
            .map(|a| base_tech.output(alone(&a.prefs).0))
            .collect();
        let median = base_tech.output(alone(&Prefs::CobbDouglas { theta: 0.5 }).0);
        let mean = singleton.iter().sum::<f64>() / n as f64;
        for (a, own) in agents.iter_mut().zip(&singleton) {
            a.base = config.base_share
                * match config.base_pay {
                    BasePay::Own => *own,
                    BasePay::Median => median,
                    BasePay::Mean => mean,
                };
        }
        let mut world = FirmsWorld {
            config,
            tick: 0,
            rng,
            agents,
            firms: Vec::new(),
            free: Vec::new(),
            live: Vec::new(),
            pos: Vec::new(),
            next_id: 0,
            births: 0,
            deaths: 0,
            records: Records::default(),
            lifetime_count: 0,
            lifetime_sum: 0,
            stats: Stats::default(),
        };
        // The starting firms.
        let mut order: Vec<u32> = (0..n as u32).collect();
        match world.config.initial {
            Initial::Alone => {
                for i in 0..n {
                    let f = world.found();
                    world.enter(i, f);
                }
            }
            Initial::OneFirm => {
                let f = world.found();
                for i in 0..n {
                    world.enter(i, f);
                }
            }
            Initial::RandomGroups => {
                shuffle(&mut world.rng, &mut order);
                let mut k = 0;
                while k < n {
                    let mut size = 1;
                    while world.rng.gen::<f64>() >= 0.25 {
                        size += 1;
                    }
                    let f = world.found();
                    for &i in order[k..(k + size).min(n)].iter() {
                        world.enter(i as usize, f);
                    }
                    k += size;
                }
            }
        }
        for i in 0..n {
            let e = alone(&world.agents[i].prefs).0;
            world.set_effort(i, e);
        }
        world.births = 0;
        world.produce();
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn records(&self) -> &Records {
        &self.records
    }

    /// Live firms, in order of founding.
    pub fn firms(&self) -> Vec<&Firm> {
        let mut v: Vec<&Firm> = self.live.iter().map(|&f| &self.firms[f]).collect();
        v.sort_by_key(|f| (f.born, f.id));
        v
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// A new, empty firm with its technology and hiring standard drawn.
    fn found(&mut self) -> usize {
        let c = &self.config;
        let tech = Tech {
            a: draw(&mut self.rng, c.a, c.a_max),
            b: draw(&mut self.rng, c.b, c.b_max),
            beta: draw(&mut self.rng, c.beta, c.beta_max),
        };
        let hiring = draw(&mut self.rng, c.hiring, c.hiring_max);
        self.open(tech, hiring)
    }

    fn open(&mut self, tech: Tech, hiring: f64) -> usize {
        let firm = Firm {
            id: self.next_id,
            alive: true,
            members: Vec::new(),
            tech,
            hiring,
            born: self.tick,
            total: 0.0,
            last_total: 0.0,
            last_size: 0,
            base_sum: 0.0,
            team: false,
            output: 0.0,
        };
        self.next_id += 1;
        self.births += 1;
        let slot = if let Some(s) = self.free.pop() {
            self.firms[s] = firm;
            s
        } else {
            self.firms.push(firm);
            self.pos.push(0);
            self.firms.len() - 1
        };
        self.pos[slot] = self.live.len();
        self.live.push(slot);
        slot
    }

    fn enter(&mut self, i: usize, f: usize) {
        let firm = &mut self.firms[f];
        firm.members.push(i as u32);
        firm.total += self.agents[i].effort;
        firm.base_sum += self.agents[i].base;
        if firm.members.len() >= 2 {
            firm.team = true;
        }
        self.agents[i].firm = f;
        self.agents[i].joined = self.tick;
    }

    fn leave(&mut self, i: usize) {
        let f = self.agents[i].firm;
        let firm = &mut self.firms[f];
        firm.members.retain(|&m| m as usize != i);
        firm.total -= self.agents[i].effort;
        firm.base_sum -= self.agents[i].base;
        if firm.members.is_empty() {
            firm.alive = false;
            firm.total = 0.0;
            let (lifetime, team) = (self.tick.saturating_sub(firm.born), firm.team);
            if self.tick > u64::from(self.config.burn_in) {
                self.records.died(lifetime, team);
                self.lifetime_count += 1;
                self.lifetime_sum += lifetime;
            }
            self.deaths += 1;
            // Out of the live list.
            let p = self.pos[f];
            let last = *self.live.last().unwrap();
            self.live.swap_remove(p);
            if last != f {
                self.pos[last] = p;
            }
            self.free.push(f);
        }
    }

    fn set_effort(&mut self, i: usize, e: f64) {
        let f = self.agents[i].firm;
        self.firms[f].total += e - self.agents[i].effort;
        self.agents[i].effort = e;
    }

    /// The others' effort an agent sees in firm `f` (its own excluded if a
    /// member).
    fn others(&self, i: usize, f: usize) -> f64 {
        let firm = &self.firms[f];
        let member = self.agents[i].firm == f;
        match self.config.others_effort {
            OthersEffort::Live => {
                if member {
                    (firm.total - self.agents[i].effort).max(0.0)
                } else {
                    firm.total.max(0.0)
                }
            }
            OthersEffort::LastPeriod => {
                let own = if member && self.agents[i].last_firm == firm.id {
                    self.agents[i].last_effort
                } else {
                    0.0
                };
                (firm.last_total - own).max(0.0)
            }
        }
    }

    /// Agent i's share in firm `f` of `n` members at seniority `rank` (1 is
    /// the longest-serving).
    fn share(&self, i: usize, n: usize, rank: usize, base_others: f64) -> Share {
        match self.config.pay {
            Pay::Equal => Share::Fraction(1.0 / n as f64),
            Pay::Seniority => {
                let p = self.config.seniority_base;
                let w = |r: usize| 1.0 / crate::firms::effort::powf(p, r as f64);
                let total: f64 = (1..=n).map(w).sum();
                let r = match self.config.seniority_order {
                    SeniorityOrder::SeniorFirst => rank,
                    SeniorityOrder::JuniorFirst => n + 1 - rank,
                };
                Share::Fraction(w(r) / total)
            }
            Pay::Base => Share::Base {
                own: self.agents[i].base,
                others: base_others,
                n: n as f64,
            },
        }
    }

    /// The options agent i weighs: its own firm, a start-up (technology and
    /// hiring standard drawn now), and the firms its network shows it that
    /// admit it. (None for the start-up if it is alone: starting again is
    /// staying.)
    fn options(&mut self, i: usize) -> Options {
        let own = self.agents[i].firm;
        let n_own = self.firms[own].members.len();
        let rank = self.firms[own]
            .members
            .iter()
            .position(|&m| m as usize == i)
            .unwrap()
            + 1;
        let stay = Choice {
            tech: self.firms[own].tech,
            others: self.others(i, own),
            share: self.share(
                i,
                n_own,
                rank,
                self.firms[own].base_sum - self.agents[i].base,
            ),
        };
        let start = if n_own > 1 {
            let c = &self.config;
            let tech = Tech {
                a: draw(&mut self.rng, c.a, c.a_max),
                b: draw(&mut self.rng, c.b, c.b_max),
                beta: draw(&mut self.rng, c.beta, c.beta_max),
            };
            let hiring = draw(&mut self.rng, c.hiring, c.hiring_max);
            Some((
                Choice {
                    tech,
                    others: 0.0,
                    share: self.share(i, 1, 1, 0.0),
                },
                tech,
                hiring,
            ))
        } else {
            None
        };
        let mut targets: Vec<usize> = Vec::new();
        match self.config.network {
            Network::Friends => {
                for &j in &self.agents[i].friends {
                    let f = self.agents[j as usize].firm;
                    if f != own && !targets.contains(&f) {
                        targets.push(f);
                    }
                }
            }
            Network::RandomFirms => {
                let want = self.agents[i].friends.len();
                let others = self.live.len() - 1;
                if others <= want {
                    targets.extend(self.live.iter().copied().filter(|&f| f != own));
                } else {
                    while targets.len() < want {
                        let f = self.live[self.rng.gen_range(0..self.live.len() as u32) as usize];
                        if f != own && !targets.contains(&f) {
                            targets.push(f);
                        }
                    }
                }
            }
        }
        let joins = targets
            .into_iter()
            .filter(|&f| self.admits(f, i))
            .map(|f| {
                let n = self.firms[f].members.len() + 1;
                (
                    f,
                    Choice {
                        tech: self.firms[f].tech,
                        others: self.others(i, f),
                        share: self.share(i, n, n, self.firms[f].base_sum),
                    },
                )
            })
            .collect();
        (stay, start, joins)
    }

    /// Whether firm `f` admits agent i: its θ at least φ times that of the
    /// firm's longest-serving member (A99 §4.9).
    pub fn admits(&self, f: usize, i: usize) -> bool {
        let firm = &self.firms[f];
        let senior = self.agents[firm.members[0] as usize].prefs.weight();
        self.agents[i].prefs.weight() >= firm.hiring * senior
    }

    fn activate(&mut self, i: usize) {
        let prefs = self.agents[i].prefs;
        let current = self.agents[i].effort;
        let search = search_of(&self.config);
        let (lo, hi) = if self.config.effort_window >= 1.0 {
            (0.0, 1.0)
        } else {
            let half = 0.5 * self.config.effort_window;
            (current - half, current + half)
        };
        // Sticky effort's window for moves too, or only at home.
        let (mlo, mhi) = if self.config.adjust_scope == AdjustScope::Everywhere {
            (lo, hi)
        } else {
            (0.0, 1.0)
        };
        let (stay, start, joins) = self.options(i);
        // The best of the options: 0 stay, 1 start, 2.. join.
        let mut pick: (usize, f64, f64);
        match self.config.random_behavior {
            RandomBehavior::Choices => {
                let which = self.rng.gen_range(0..3u32);
                pick = match which {
                    1 if start.is_some() => {
                        let (e, u) = best(&prefs, &start.unwrap().0, lo, hi, search);
                        (1, e, u)
                    }
                    2 if !joins.is_empty() => {
                        let k = self.rng.gen_range(0..joins.len() as u32) as usize;
                        let (e, u) = best(&prefs, &joins[k].1, lo, hi, search);
                        (2 + k, e, u)
                    }
                    _ => {
                        let (e, u) = best(&prefs, &stay, lo, hi, search);
                        (0, e, u)
                    }
                };
                self.apply(i, pick.0, pick.1, start, &joins, true);
                return;
            }
            RandomBehavior::Effort => {
                let e: f64 = self.rng.gen();
                pick = (0, e, stay.utility(&prefs, e));
                if let Some((c, _, _)) = &start {
                    let u = c.utility(&prefs, e);
                    if u > pick.2 {
                        pick = (1, e, u);
                    }
                }
                for (k, (_, c)) in joins.iter().enumerate() {
                    let u = c.utility(&prefs, e);
                    if u > pick.2 {
                        pick = (2 + k, e, u);
                    }
                }
            }
            RandomBehavior::None if self.config.groping => {
                // One random try at a new effort here; the other options at
                // the current effort (our reading of §4.6).
                let trial: f64 = self.rng.gen();
                let (uc, ut) = (stay.utility(&prefs, current), stay.utility(&prefs, trial));
                pick = if ut > uc {
                    (0, trial, ut)
                } else {
                    (0, current, uc)
                };
                // Elsewhere: at the current effort, or (own firm only) the best.
                let elsewhere = |c: &Choice| {
                    if self.config.adjust_scope == AdjustScope::Everywhere {
                        (current, c.utility(&prefs, current))
                    } else {
                        best(&prefs, c, 0.0, 1.0, search)
                    }
                };
                if let Some((c, _, _)) = &start {
                    let (e, u) = elsewhere(c);
                    if u > pick.2 {
                        pick = (1, e, u);
                    }
                }
                for (k, (_, c)) in joins.iter().enumerate() {
                    let (e, u) = elsewhere(c);
                    if u > pick.2 {
                        pick = (2 + k, e, u);
                    }
                }
            }
            RandomBehavior::None => {
                let (e, u) = best(&prefs, &stay, lo, hi, search);
                pick = (0, e, u);
                if let Some((c, _, _)) = &start {
                    let (e, u) = best(&prefs, c, mlo, mhi, search);
                    if u > pick.2 {
                        pick = (1, e, u);
                    }
                }
                for (k, (_, c)) in joins.iter().enumerate() {
                    let (e, u) = best(&prefs, c, mlo, mhi, search);
                    if u > pick.2 {
                        pick = (2 + k, e, u);
                    }
                }
            }
        }
        // Loyalty: stay (re-choosing effort here) until wanting to move more
        // than λ times.
        if pick.0 != 0 && self.agents[i].wants < self.agents[i].loyalty {
            self.agents[i].wants += 1;
            let (e, _) = if self.config.groping {
                (current, 0.0)
            } else {
                best(&prefs, &stay, lo, hi, search)
            };
            pick = (0, e, 0.0);
        }
        self.apply(i, pick.0, pick.1, start, &joins, false);
    }

    fn apply(
        &mut self,
        i: usize,
        which: usize,
        e: f64,
        start: Option<(Choice, Tech, f64)>,
        joins: &[(usize, Choice)],
        _random: bool,
    ) {
        match which {
            0 => self.set_effort(i, e),
            1 => {
                let (_, tech, hiring) = start.unwrap();
                self.leave(i);
                let f = self.open(tech, hiring);
                self.agents[i].effort = e;
                self.agents[i].wants = 0;
                self.enter(i, f);
            }
            k => {
                let f = joins[k - 2].0;
                self.leave(i);
                self.agents[i].effort = e;
                self.agents[i].wants = 0;
                self.enter(i, f);
            }
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.births = 0;
        self.deaths = 0;
        let n = self.agents.len();
        let count = ((self.config.activation_rate * n as f64).round() as usize).max(1);
        match self.config.activation {
            Activation::Random => {
                for _ in 0..count {
                    let i = self.rng.gen_range(0..n as u32) as usize;
                    self.activate(i);
                }
            }
            Activation::Uniform => {
                let mut order: Vec<u32> = (0..n as u32).collect();
                let mut done = 0;
                while done < count {
                    shuffle(&mut self.rng, &mut order);
                    for &i in order.iter().take(count - done) {
                        self.activate(i as usize);
                    }
                    done += n.min(count - done);
                }
            }
        }
        self.produce();
        self.record();
    }

    /// Every firm produces and pays; the records take the period.
    fn produce(&mut self) {
        let sampling = self.tick > u64::from(self.config.burn_in);
        let sample = sampling
            && self
                .tick
                .is_multiple_of(u64::from(self.config.sample_every));
        let mut sizes: Vec<(u32, f64)> = Vec::new();
        for k in 0..self.live.len() {
            let f = self.live[k];
            let members = self.firms[f].members.clone();
            let total: f64 = members
                .iter()
                .map(|&m| self.agents[m as usize].effort)
                .sum();
            let tech = self.firms[f].tech;
            let output = tech.output(total);
            let n = members.len();
            let id = self.firms[f].id;
            let base_sum = self.firms[f].base_sum;
            for (r, &m) in members.iter().enumerate() {
                let m = m as usize;
                let share = self.share(m, n, r + 1, base_sum - self.agents[m].base);
                let income = share.income(output);
                let a = &mut self.agents[m];
                a.income = income;
                a.utility = a.prefs.utility(income, 1.0 - a.effort);
                a.last_effort = a.effort;
                a.last_firm = id;
            }
            let firm = &mut self.firms[f];
            if sampling && firm.last_size > 0 {
                let before = firm.last_size;
                self.records.grow(before, n as u32);
            }
            let firm = &mut self.firms[f];
            firm.total = total;
            firm.last_total = total;
            firm.last_size = n as u32;
            firm.output = output;
            if sample {
                sizes.push((n as u32, output));
            }
        }
        if sample {
            self.records.sample(&sizes);
        }
    }

    fn record(&mut self) {
        let n = self.agents.len() as f64;
        let firms = self.live.len();
        let largest = self
            .live
            .iter()
            .map(|&f| self.firms[f].members.len())
            .max()
            .unwrap_or(0);
        let singletons = self
            .live
            .iter()
            .filter(|&&f| self.firms[f].members.len() == 1)
            .count();
        let output: f64 = self.live.iter().map(|&f| self.firms[f].output).sum();
        let top = self
            .live
            .iter()
            .map(|&f| &self.firms[f])
            .max_by_key(|f| (f.members.len(), std::cmp::Reverse(f.id)));
        let mean = if self.lifetime_count == 0 {
            f64::NAN
        } else {
            self.lifetime_sum as f64 / self.lifetime_count as f64
        };
        self.stats.push(FirmsSnapshot {
            tick: self.tick,
            firms: firms as u32,
            births: self.births,
            deaths: self.deaths,
            mean_size: n / firms.max(1) as f64,
            largest: largest as u32,
            singletons: singletons as f64 / firms.max(1) as f64,
            effort: self.agents.iter().map(|a| a.effort).sum::<f64>() / n,
            output,
            income: self.agents.iter().map(|a| a.income).sum::<f64>() / n,
            utility: self.agents.iter().map(|a| a.utility).sum::<f64>() / n,
            largest_output_share: top.map_or(f64::NAN, |f| {
                if output > 0.0 {
                    f.output / output
                } else {
                    f64::NAN
                }
            }),
            mu: mu_ols(&self.records.sizes),
            // The exact maximum likelihood is costlier: every 10 periods.
            mu_mle: if self.tick.is_multiple_of(10) {
                mu_mle(&self.records.sizes, 2)
            } else {
                self.stats.latest().map_or(f64::NAN, |s| s.mu_mle)
            },
            lifetime: mean,
            period: self.tick,
        });
    }

    fn firm_view(&self, f: usize) -> FirmView {
        let firm = &self.firms[f];
        let n = firm.members.len().max(1) as f64;
        FirmView {
            id: firm.id,
            size: firm.members.len() as u32,
            output: firm.output,
            age: self.tick - firm.born,
            a: firm.tech.a,
            b: firm.tech.b,
            beta: firm.tech.beta,
            mean_theta: firm
                .members
                .iter()
                .map(|&m| self.agents[m as usize].prefs.weight())
                .sum::<f64>()
                / n,
            mean_effort: firm
                .members
                .iter()
                .map(|&m| self.agents[m as usize].effort)
                .sum::<f64>()
                / n,
            free_riders: firm
                .members
                .iter()
                .filter(|&&m| self.agents[m as usize].effort <= 1e-9)
                .count() as u32,
        }
    }

    fn agent_view(&self, i: usize) -> AgentView {
        let a = &self.agents[i];
        AgentView {
            id: i as u32 + 1,
            theta: a.prefs.weight(),
            effort: a.effort,
            income: a.income,
            utility: a.utility,
            tenure: self.tick - a.joined,
            firm: self.firms[a.firm].id,
        }
    }

    /// The rows the frame shows: the `ROWS` largest firms, oldest first
    /// among equals.
    fn rows(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self.live.clone();
        v.sort_by_key(|&f| {
            (
                std::cmp::Reverse(self.firms[f].members.len()),
                self.firms[f].born,
                self.firms[f].id,
            )
        });
        v.truncate(ROWS);
        v
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<FirmsInspection, String> {
        if x as usize >= WIDE || y as usize >= TALL {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let mut out = FirmsInspection {
            site: FirmsCell { x, y },
            panel: None,
            firm: None,
            member: None,
            agent: None,
        };
        if (x as usize) < FIRMS_W && (y as usize) < FIRMS_H {
            out.panel = Some("firms");
            let rows = self.rows();
            if let Some(&f) = rows.get(y as usize / CELL) {
                out.firm = Some(self.firm_view(f));
                if let Some(&m) = self.firms[f].members.get(x as usize / CELL) {
                    out.member = Some(self.agent_view(m as usize));
                }
            }
        } else if x as usize >= PLOT_X {
            out.panel = Some("sizes");
        }
        Ok(out)
    }
}

fn search_of(c: &FirmsConfig) -> Search {
    match c.effort_search {
        EffortSearch::Exact => Search::Exact,
        EffortSearch::Grid => Search::Grid(c.grid_steps),
    }
}

/// Fisher–Yates with `u32` draws.
fn shuffle(rng: &mut SimRng, v: &mut [u32]) {
    for k in (1..v.len()).rev() {
        let j = rng.gen_range(0..=k as u32) as usize;
        v.swap(k, j);
    }
}

fn line(c: &mut Canvas, a: (usize, usize), b: (usize, usize), color: [u8; 3]) {
    let (mut x0, mut y0, x1, y1) = (a.0 as i64, a.1 as i64, b.0 as i64, b.1 as i64);
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    loop {
        if (0..WIDE as i64).contains(&x0) && (0..TALL as i64).contains(&y0) {
            c.put(x0 as usize, y0 as usize, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

impl Model for FirmsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Firms(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        FirmsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick and every agent's firm, effort and income.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        for a in &self.agents {
            eat(&self.firms[a.firm].id.to_le_bytes());
            eat(&a.effort.to_bits().to_le_bytes());
            eat(&a.income.to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (WIDE as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: FirmsMode = mode.parse()?;
        let mut c = Canvas { buf, wide: 0 };
        c.clear(WIDE, TALL);
        for (row, &f) in self.rows().iter().enumerate() {
            for (col, &m) in self.firms[f].members.iter().take(COLS).enumerate() {
                let a = &self.agents[m as usize];
                let color = match mode {
                    FirmsMode::Founder => {
                        if col == 0 {
                            FOUNDER
                        } else {
                            MEMBER
                        }
                    }
                    FirmsMode::Theta => scale(a.prefs.weight(), LOW, HIGH),
                    FirmsMode::Effort => scale(a.effort, LOW, HIGH),
                    FirmsMode::Income => scale(a.income / 4.0, LOW, HIGH),
                };
                for dy in 0..CELL - 1 {
                    for dx in 0..CELL - 1 {
                        c.put(col * CELL + dx, row * CELL + dy, color);
                    }
                }
            }
        }
        // The size distribution: axes, points and the OLS line.
        line(
            &mut c,
            (PLOT_X, PLOT - 1),
            (PLOT_X + PLOT - 1, PLOT - 1),
            AXIS,
        );
        line(&mut c, (PLOT_X, 0), (PLOT_X, PLOT - 1), AXIS);
        let sizes = &self.records.sizes;
        let total: u64 = sizes.iter().sum();
        if total > 0 {
            let mut fitted = Vec::new();
            for (s, &k) in sizes.iter().enumerate().skip(1) {
                if k == 0 {
                    continue;
                }
                let (x, y) = (ln(s as f64), ln(k as f64 / total as f64));
                if let Some((px, py)) = plot_at(x, y) {
                    c.put(px, py, POINT);
                    if py + 1 < PLOT {
                        c.put(px, py + 1, POINT);
                    }
                }
                if s >= 2 && k as f64 / total as f64 >= 1e-5 {
                    fitted.push((x, y));
                }
            }
            if let Some((slope, icept)) = ols(&fitted) {
                let ends: Vec<(usize, usize)> = [ln(2.0), fitted.last().map_or(0.0, |p| p.0)]
                    .iter()
                    .filter_map(|&x| plot_at(x, icept + slope * x))
                    .collect();
                if ends.len() == 2 {
                    line(&mut c, ends[0], ends[1], FIT);
                }
            }
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
        let mut out = String::from("id,theta,effort,income,utility,firm,firm_size,tenure\n");
        for (i, a) in self.agents.iter().enumerate() {
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                i + 1,
                a.prefs.weight(),
                a.effort,
                a.income,
                a.utility,
                self.firms[a.firm].id,
                self.firms[a.firm].members.len(),
                self.tick - a.joined
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// An agent's cell in the firm rows (None if its firm is off the frame).
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let i = usize::try_from(id.checked_sub(1)?).ok()?;
        let a = self.agents.get(i)?;
        let row = self.rows().iter().position(|&f| f == a.firm)?;
        let col = self.firms[a.firm]
            .members
            .iter()
            .position(|&m| m as usize == i)?;
        (col < COLS).then(|| ((col * CELL) as u32, (row * CELL) as u32))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Firms(next) = next else {
            return Err(wrong_model(ModelKind::Firms, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        // Base pay follows its settings at once.
        if (next.base_pay, next.base_share) != (self.config.base_pay, self.config.base_share) {
            let tech = Tech {
                a: next.a,
                b: next.b,
                beta: next.beta,
            };
            let search = search_of(&next);
            let single = |p: &Prefs| {
                tech.output(
                    best(
                        p,
                        &Choice {
                            tech,
                            others: 0.0,
                            share: Share::Fraction(1.0),
                        },
                        0.0,
                        1.0,
                        search,
                    )
                    .0,
                )
            };
            let own: Vec<f64> = self.agents.iter().map(|a| single(&a.prefs)).collect();
            let (median, mean) = (
                single(&Prefs::CobbDouglas { theta: 0.5 }),
                own.iter().sum::<f64>() / own.len() as f64,
            );
            for (a, o) in self.agents.iter_mut().zip(&own) {
                a.base = next.base_share
                    * match next.base_pay {
                        BasePay::Own => *o,
                        BasePay::Median => median,
                        BasePay::Mean => mean,
                    };
            }
            for &f in &self.live {
                self.firms[f].base_sum = self.firms[f]
                    .members
                    .iter()
                    .map(|&m| self.agents[m as usize].base)
                    .sum();
            }
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped after its last period: a sweep reads it there.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(edit: impl FnOnce(&mut FirmsConfig)) -> FirmsWorld {
        let mut c = FirmsConfig {
            agents: 200,
            burn_in: 20,
            stop_at: 200,
            ..FirmsConfig::default()
        };
        edit(&mut c);
        FirmsWorld::new(c, 1).unwrap()
    }

    fn check_invariants(w: &FirmsWorld) {
        let mut seen = vec![0; w.agents.len()];
        for &f in &w.live {
            let firm = &w.firms[f];
            assert!(firm.alive && !firm.members.is_empty());
            let total: f64 = firm
                .members
                .iter()
                .map(|&m| w.agents[m as usize].effort)
                .sum();
            assert!(
                (total - firm.total).abs() < 1e-9,
                "firm {} total drift",
                firm.id
            );
            for &m in &firm.members {
                assert_eq!(w.agents[m as usize].firm, f);
                seen[m as usize] += 1;
            }
        }
        assert!(
            seen.iter().all(|&k| k == 1),
            "every agent in exactly one live firm"
        );
    }

    #[test]
    fn everyone_starts_alone_at_their_optimum() {
        let w = world(|_| {});
        assert_eq!(w.live.len(), 200);
        check_invariants(&w);
        let s = w.stats.latest().unwrap();
        // A99's singletons: mean output per agent ≈ 0.934 over θ ~ U[0, 1].
        assert!((s.output / 200.0 - 0.934).abs() < 0.08, "{}", s.output);
        assert_eq!((s.firms, s.largest), (200, 1));
    }

    #[test]
    fn firms_form_grow_and_the_books_balance() {
        let mut w = world(|_| {});
        w.run(200);
        check_invariants(&w);
        let s = w.stats.latest().unwrap();
        assert!(s.largest >= 5 && s.firms < 200, "{s:?}");
        assert!(s.mu.is_finite() && s.lifetime.is_finite());
        // Births and deaths balance the firm count.
        let hist = w.stats.history();
        for pair in hist.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            assert_eq!(
                i64::from(b.firms),
                i64::from(a.firms) + i64::from(b.births) - i64::from(b.deaths)
            );
        }
    }

    #[test]
    fn under_constant_returns_even_identical_agents_join_myopically() {
        // A99 fn 19: under constant returns working together is never
        // individually rational — at equilibrium. A myopic joiner takes the
        // other's effort as fixed: alone at e = θ, joining at θ² gains the
        // factor (1 + θ)/2^θ > 1 (≈ 1.04 at θ = 0.75), so firms still form.
        let theta: f64 = 0.75;
        assert!((1.0 + theta) / 2f64.powf(theta) > 1.0);
        let mut w = world(|c| {
            c.b = 0.0;
            c.preferences = Preferences::Fixed;
            c.theta = theta;
        });
        w.run(20);
        check_invariants(&w);
        assert!(w.stats.latest().unwrap().largest >= 2);
    }

    #[test]
    fn others_effort_readings_differ() {
        let mut a = world(|_| {});
        let mut b = world(|c| c.others_effort = OthersEffort::Live);
        a.run(50);
        b.run(50);
        assert_ne!(Model::fingerprint(&a), Model::fingerprint(&b));
    }

    #[test]
    fn loyal_agents_stay_until_their_count_runs_out() {
        let mut w = world(|c| {
            c.loyalty = 1000;
        });
        w.run(10);
        // Nobody may move before wanting to 1000 times: no firm grows.
        assert_eq!(w.stats.latest().unwrap().largest, 1);
    }

    #[test]
    fn a_hiring_standard_admits_only_those_near_the_senior_member() {
        let w = world(|c| c.hiring = 0.5);
        let weight = |i: usize| w.agents[i].prefs.weight();
        // Everyone starts alone: firm slot i holds agent i, its founder.
        let (hi, lo) = (0..200).fold((0, 0), |(h, l), i| {
            (
                if weight(i) > weight(h) { i } else { h },
                if weight(i) < weight(l) { i } else { l },
            )
        });
        assert!(
            !w.admits(w.agents[hi].firm, lo),
            "θ {} under half of {}",
            weight(lo),
            weight(hi)
        );
        assert!(w.admits(w.agents[lo].firm, hi));
        let open = world(|_| {});
        assert!(open.admits(open.agents[hi].firm, lo));
    }

    #[test]
    fn sticky_effort_moves_at_most_half_its_window() {
        // Uniform activation: each agent at most once a period.
        let mut w = world(|c| {
            c.effort_window = 0.1;
            c.activation = Activation::Uniform;
        });
        let before: Vec<f64> = w.agents.iter().map(|a| a.effort).collect();
        w.step();
        for (a, e) in w.agents.iter().zip(before) {
            assert!((a.effort - e).abs() <= 0.05 + 1e-12, "{} → {}", e, a.effort);
        }
    }

    #[test]
    fn sticky_effort_in_the_own_firm_only_leaves_moves_free() {
        use super::super::config::AdjustScope;
        // Uniform activation, window ±0.05, but only for effort at home: an
        // agent that moved may change effort by more.
        let mut w = world(|c| {
            c.effort_window = 0.1;
            c.activation = Activation::Uniform;
            c.adjust_scope = AdjustScope::OwnFirm;
        });
        let mut free_move = false;
        for _ in 0..20 {
            let before: Vec<(f64, u64)> = w
                .agents
                .iter()
                .map(|a| (a.effort, w.firms[a.firm].id))
                .collect();
            w.step();
            for (a, (e, f)) in w.agents.iter().zip(before) {
                if w.firms[a.firm].id == f {
                    assert!((a.effort - e).abs() <= 0.05 + 1e-12);
                } else if (a.effort - e).abs() > 0.05 {
                    free_move = true;
                }
            }
        }
        assert!(
            free_move,
            "some mover changed effort by more than the window"
        );
    }

    #[test]
    fn junior_first_seniority_pays_the_newest_most() {
        use super::super::config::SeniorityOrder;
        let mut w = world(|c| {
            c.pay = Pay::Seniority;
            c.seniority_order = SeniorityOrder::JuniorFirst;
        });
        w.run(100);
        let f = w
            .live
            .iter()
            .copied()
            .find(|&f| w.firms[f].members.len() >= 3)
            .expect("a firm of three");
        let incomes: Vec<f64> = w.firms[f]
            .members
            .iter()
            .map(|&m| w.agents[m as usize].income)
            .collect();
        assert!(
            incomes.windows(2).all(|p| p[0] <= p[1] + 1e-12),
            "{incomes:?}"
        );
    }

    #[test]
    fn seniority_shares_sum_to_output_and_fall_with_rank() {
        let mut w = world(|c| {
            c.pay = Pay::Seniority;
            c.seniority_base = 2.0;
        });
        w.run(100);
        for &f in &w.live {
            let firm = &w.firms[f];
            let paid: f64 = firm
                .members
                .iter()
                .map(|&m| w.agents[m as usize].income)
                .sum();
            assert!((paid - firm.output).abs() < 1e-9 * firm.output.max(1.0));
            let incomes: Vec<f64> = firm
                .members
                .iter()
                .map(|&m| w.agents[m as usize].income)
                .collect();
            assert!(incomes.windows(2).all(|p| p[0] >= p[1] - 1e-12));
        }
    }

    #[test]
    fn base_pay_is_paid_even_when_output_falls_short() {
        let mut w = world(|c| {
            c.pay = Pay::Base;
            c.base_share = 0.8;
        });
        w.run(50);
        for a in &w.agents {
            assert!(a.income >= a.base - 1e-12);
        }
    }

    #[test]
    fn the_2013_parameterization_draws_per_firm() {
        let mut w = world(|c| {
            c.a = 0.0;
            c.a_max = 0.5;
            c.b = 0.75;
            c.b_max = 1.25;
            c.beta = 1.5;
            c.beta_max = 2.0;
            c.neighbors = 2;
            c.neighbors_max = 6;
            c.activation_rate = 0.04;
        });
        w.run(100);
        check_invariants(&w);
        let techs: Vec<Tech> = w.live.iter().map(|&f| w.firms[f].tech).collect();
        assert!(techs.iter().all(|t| (0.0..=0.5).contains(&t.a)
            && (0.75..=1.25).contains(&t.b)
            && (1.5..=2.0).contains(&t.beta)));
        assert!(techs.windows(2).any(|p| p[0] != p[1]));
    }

    #[test]
    fn random_firms_uniform_activation_and_initial_groups_run() {
        for edit in [
            (|c: &mut FirmsConfig| c.network = Network::RandomFirms) as fn(&mut FirmsConfig),
            |c| c.activation = Activation::Uniform,
            |c| c.initial = Initial::RandomGroups,
            |c| c.initial = Initial::OneFirm,
            |c| c.random_behavior = RandomBehavior::Choices,
            |c| c.random_behavior = RandomBehavior::Effort,
            |c| c.groping = true,
            |c| c.effort_search = EffortSearch::Grid,
            |c| c.preferences = Preferences::Ces,
            |c| c.preferences = Preferences::Normal,
        ] {
            let mut w = world(edit);
            w.run(30);
            check_invariants(&w);
        }
    }

    #[test]
    fn lifetimes_are_counted_after_the_burn_in() {
        let mut w = world(|c| c.burn_in = 50);
        w.run(50);
        assert_eq!(w.records.lifetimes.iter().sum::<u64>(), 0);
        w.run(50);
        assert!(w.records.lifetimes.iter().sum::<u64>() > 0);
    }

    #[test]
    fn the_lifetime_series_matches_fit_lifetimes_mean() {
        let mut w = world(|c| c.burn_in = 5);
        w.run(150);
        let (_, mean, _, _) = super::super::fit::lifetimes(&w.records.lifetimes);
        assert!(mean.is_finite(), "expected some deaths recorded by tick 150");
        assert_eq!(w.stats.latest().unwrap().lifetime, mean);
    }

    #[test]
    fn recording_a_period_stays_cheap_even_with_a_long_lived_firm_in_the_histogram() {
        // A death at a very large lifetime makes `records.lifetimes` a huge
        // histogram. record() must not walk it every period: it should stay
        // O(1), not O(histogram length) per call.
        let mut w = world(|_| {});
        w.records.died(200_000, false);
        let start = std::time::Instant::now();
        for _ in 0..5_000 {
            w.record();
        }
        let elapsed = start.elapsed();
        assert!(
            elapsed.as_millis() < 300,
            "5 000 record() calls took {elapsed:?} with a 200 000-bin lifetime \
             histogram; record() must track the running mean instead of \
             recomputing it from the whole histogram every period"
        );
    }

    #[test]
    fn the_view_and_inspect_read_a_firm_and_its_member() {
        let mut w = world(|_| {});
        w.run(60);
        let mut buf = Vec::new();
        for mode in ["founder", "theta", "effort", "income"] {
            Model::render(&w, mode, "", &mut buf).unwrap();
        }
        assert!(Model::render(&w, "wealth", "", &mut buf).is_err());
        let i = w.inspect(0, 0).unwrap();
        assert_eq!(i.panel, Some("firms"));
        let (f, m) = (i.firm.unwrap(), i.member.unwrap());
        // The first row is the largest firm.
        assert_eq!(f.size, w.stats.latest().unwrap().largest);
        assert_eq!(m.firm, f.id);
        let (x, y) = Model::locate(&w, u64::from(m.id)).unwrap();
        assert_eq!((x, y), (0, 0));
        assert_eq!(
            w.inspect(PLOT_X as u32 + 5, 5).unwrap().panel,
            Some("sizes")
        );
        assert!(w.inspect(WIDE as u32, 0).is_err());
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let mut next = w.config.clone();
        next.beta = 1.8;
        next.pay = Pay::Base;
        Model::set_config(&mut w, ModelConfig::Firms(next.clone())).unwrap();
        assert!(w.agents.iter().all(|a| a.base > 0.0));
        next.agents = 300;
        assert!(Model::set_config(&mut w, ModelConfig::Firms(next)).is_err());
    }
}
