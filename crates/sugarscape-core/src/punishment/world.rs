//! The Altruistic Punishment world. Each period: every agent cooperates or
//! defects and punishers fine the defectors; everyone imitates someone, from
//! its own group or (with probability m) another, with probability
//! Wⱼ/(Wⱼ + Wᵢ); groups meet in conflict and the losers are replaced by the
//! winners; and a few agents mutate.

use std::collections::VecDeque;
use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{
    Counted, Erring, Imitation, Pairing, Punishing, PunishmentConfig, Refill, Start, Structure,
    Traits, Victory,
};
use super::stats::PunishmentSnapshot;
use super::view::{
    row, scale, traits_color, Mosaic, CONTRIBUTOR, DEFECTOR, ERRED, HIGH, LEAST_WIDE, LOST, LOW,
    MARK, PUNISHER, STRIP, STRIP_GAP,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::portable::exp_neg;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// A discrete agent's type (continuous agents have none).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Contributor,
    Defector,
    Punisher,
}

/// The three types as (cooperate, punish).
const TYPES: [(f64, f64); 3] = [(1.0, 0.0), (0.0, 0.0), (1.0, 1.0)];

fn kind(cooperate: f64, punish: f64) -> usize {
    if cooperate < 0.5 {
        1
    } else if punish < 0.5 {
        0
    } else {
        2
    }
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunishmentMode {
    Type,
    Acts,
    Payoff,
    Group,
}

impl std::str::FromStr for PunishmentMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "type" => Self::Type,
            "acts" => Self::Acts,
            "payoff" => Self::Payoff,
            "group" => Self::Group,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PunishmentInspection {
    pub site: PunishmentCell,
    /// `groups` or `time`; null elsewhere.
    pub panel: Option<&'static str>,
    /// The group under the cell, and the agent in it.
    pub group: Option<GroupView>,
    pub agent: Option<AgentView>,
    /// Time: the period and its cooperation and punishment.
    pub period: Option<u64>,
    pub cooperation: Option<f64>,
    pub punishment: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct PunishmentCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub group: u32,
    /// Discrete traits: the type; continuous: null.
    pub kind: Option<Kind>,
    pub cooperate: f64,
    pub punish: f64,
    /// This period: whether it cooperated and punished, and its payoff.
    pub cooperated: bool,
    pub punished: bool,
    pub payoff: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct GroupView {
    pub index: u32,
    pub contributors: f64,
    pub punishers: f64,
    pub defectors: f64,
    /// The share who cooperated this period, and the mean payoff.
    pub acts: f64,
    pub payoff: f64,
    /// The last period it fought, and whether it lost then.
    pub last_conflict: Option<u64>,
    pub lost: bool,
}

#[derive(Clone)]
pub struct PunishmentWorld {
    pub config: PunishmentConfig,
    /// Completed periods.
    pub tick: u64,
    rng: SimRng,
    /// Each agent's traits, group by group (agent a is in group a / n).
    cooperate: Vec<f64>,
    punish: Vec<f64>,
    /// This period's acts and payoffs.
    cooperated: Vec<bool>,
    punished: Vec<bool>,
    payoff: Vec<f64>,
    /// Each group's last conflict, and whether it lost it.
    fought: Vec<Option<(u64, bool)>>,
    conflicts: u32,
    extinctions: u32,
    /// The last periods' cooperation and punishment, for the time strip.
    recent: VecDeque<(f64, f64)>,
    /// The long-run window's first period, and its sum of cooperation so far.
    window_from: u64,
    window_sum: f64,
    pub stats: Stats<PunishmentSnapshot>,
}

impl PunishmentWorld {
    pub fn new(config: PunishmentConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let (g, n) = (config.groups as usize, config.size as usize);
        let mut cooperate = vec![0.0; g * n];
        let mut punish = vec![0.0; g * n];
        if config.start == Start::OnePunisherGroup {
            for a in 0..n {
                cooperate[a] = 1.0;
                punish[a] = 1.0;
            }
        }
        let mut world = PunishmentWorld {
            config,
            tick: 0,
            rng: rng::seeded(seed),
            cooperate,
            punish,
            cooperated: vec![false; g * n],
            punished: vec![false; g * n],
            payoff: vec![0.0; g * n],
            fought: vec![None; g],
            conflicts: 0,
            extinctions: 0,
            recent: VecDeque::new(),
            window_from: 0,
            window_sum: 0.0,
            stats: Stats::default(),
        };
        world.window_from = world.window_start();
        world.record();
        Ok(world)
    }

    pub fn groups(&self) -> usize {
        self.config.groups as usize
    }

    pub fn size(&self) -> usize {
        self.config.size as usize
    }

    /// Agent `a`'s traits (cooperate, punish).
    pub fn traits(&self, a: usize) -> (f64, f64) {
        (self.cooperate[a], self.punish[a])
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

    pub fn step(&mut self) {
        self.tick += 1;
        self.act();
        let (mean_payoff, defectors) = self.payoffs();
        self.imitate();
        self.conflict(&mean_payoff, &defectors);
        self.mutate();
        self.record();
    }

    /// Who cooperates, and who punishes, this period.
    fn act(&mut self) {
        let e = self.config.error;
        for a in 0..self.cooperate.len() {
            let x = self.cooperate[a];
            let c = x > 0.0 && self.rng.gen::<f64>() < x * (1.0 - e);
            let y = self.punish[a];
            let p = y >= 1.0 || (y > 0.0 && self.rng.gen::<f64>() < y);
            self.cooperated[a] = c;
            self.punished[a] = p && (c || self.config.erring != Erring::None);
        }
    }

    /// Everyone's payoff this period; each group's mean payoff and d.
    fn payoffs(&mut self) -> (Vec<f64>, Vec<f64>) {
        let c = &self.config;
        let (g, n) = (self.groups(), self.size());
        let nf = n as f64;
        let itself = c.erring == Erring::Itself;
        let mut means = vec![0.0; g];
        let mut ds = vec![0.0; g];
        for gi in 0..g {
            let s = gi * n;
            let nc = (s..s + n).filter(|&a| self.cooperated[a]).count() as f64;
            let np = (s..s + n).filter(|&a| self.punished[a]).count() as f64;
            let nd = nf - nc;
            let mut total = 0.0;
            for a in s..s + n {
                let coop = self.cooperated[a];
                let pun = self.punished[a];
                let mut w = c.baseline + c.benefit * (nc - f64::from(u8::from(coop))) / nf;
                if coop {
                    w -= c.cost;
                } else {
                    let by = np - f64::from(u8::from(pun && !itself));
                    w -= c.fine * by / nf;
                }
                if pun {
                    match c.punishing {
                        Punishing::Variable => {
                            let of = nd - f64::from(u8::from(!coop && !itself));
                            w -= c.punish_cost * of / nf;
                        }
                        Punishing::Fixed => w -= c.fixed_cost,
                    }
                }
                let w = w.max(0.0);
                self.payoff[a] = w;
                total += w;
            }
            means[gi] = total / nf;
            ds[gi] = match c.counted {
                Counted::Types => (s..s + n).map(|a| 1.0 - self.cooperate[a]).sum::<f64>() / nf,
                Counted::Acts => nd / nf,
            };
        }
        (means, ds)
    }

    /// Payoff-biased imitation, within groups and (with probability m) across.
    fn imitate(&mut self) {
        let (g, n) = (self.groups(), self.size());
        let m = self.config.mixing;
        let ring = self.config.structure == Structure::Ring;
        let together = self.config.imitation == Imitation::Together;
        let (from_c, from_p) = if together {
            (self.cooperate.clone(), self.punish.clone())
        } else {
            (Vec::new(), Vec::new())
        };
        for a in 0..g * n {
            let gi = a / n;
            let j = if self.rng.gen::<f64>() >= m {
                let k = self.rng.gen_range(0..n as u32 - 1) as usize;
                let own = a - gi * n;
                gi * n + if k >= own { k + 1 } else { k }
            } else {
                let h = if ring {
                    if self.rng.gen::<f64>() < 0.5 {
                        (gi + 1) % g
                    } else {
                        (gi + g - 1) % g
                    }
                } else {
                    let k = self.rng.gen_range(0..g as u32 - 1) as usize;
                    if k >= gi {
                        k + 1
                    } else {
                        k
                    }
                };
                h * n + self.rng.gen_range(0..n as u32) as usize
            };
            let (wi, wj) = (self.payoff[a], self.payoff[j]);
            let chance = if wi + wj > 0.0 { wj / (wi + wj) } else { 0.5 };
            if self.rng.gen::<f64>() < chance {
                if together {
                    self.cooperate[a] = from_c[j];
                    self.punish[a] = from_p[j];
                } else {
                    self.cooperate[a] = self.cooperate[j];
                    self.punish[a] = self.punish[j];
                }
            }
        }
    }

    /// The chance group i beats group j.
    fn victory(&self, i: usize, j: usize, means: &[f64], ds: &[f64]) -> f64 {
        let c = &self.config;
        let p = match c.victory {
            Victory::Defectors => 0.5 * (1.0 + ds[j] - ds[i]),
            Victory::Payoff => {
                let range = payoff_range(c);
                if range > 0.0 {
                    0.5 + 0.5 * (means[i] - means[j]) / range
                } else {
                    0.5
                }
            }
            Victory::Tanh => 0.5 + 0.5 * tanh(c.sensitivity * (means[i] - means[j])),
        };
        p.clamp(0.0, 1.0)
    }

    /// Groups meet; the losers are replaced by the winners.
    fn conflict(&mut self, means: &[f64], ds: &[f64]) {
        self.conflicts = 0;
        self.extinctions = 0;
        if self.config.structure == Structure::Ring {
            return;
        }
        let g = self.groups();
        let eps = self.config.conflict;
        let mut order: Vec<usize> = (0..g).collect();
        for i in (1..g).rev() {
            let j = self.rng.gen_range(0..=i as u32) as usize;
            order.swap(i, j);
        }
        let mut pairs = Vec::new();
        match self.config.pairing {
            Pairing::Paired => {
                for pair in order.chunks_exact(2) {
                    if self.rng.gen::<f64>() < eps {
                        pairs.push((pair[0], pair[1]));
                    }
                }
            }
            Pairing::Either => {
                for pair in order.chunks_exact(2) {
                    let (a, b) = (self.rng.gen::<f64>(), self.rng.gen::<f64>());
                    if a < eps || b < eps {
                        pairs.push((pair[0], pair[1]));
                    }
                }
            }
            Pairing::Challenge => {
                let mut busy = vec![false; g];
                for &i in &order {
                    if busy[i] || self.rng.gen::<f64>() >= eps {
                        continue;
                    }
                    let free: Vec<usize> = (0..g).filter(|&h| h != i && !busy[h]).collect();
                    if free.is_empty() {
                        continue;
                    }
                    let h = free[self.rng.gen_range(0..free.len() as u32) as usize];
                    busy[i] = true;
                    busy[h] = true;
                    pairs.push((i, h));
                }
            }
        }
        for (i, j) in pairs {
            let win_i = self.rng.gen::<f64>() < self.victory(i, j, means, ds);
            let (win, lose) = if win_i { (i, j) } else { (j, i) };
            self.replace(win, lose);
            self.fought[win] = Some((self.tick, false));
            self.fought[lose] = Some((self.tick, true));
            self.conflicts += 1;
            self.extinctions += 1;
        }
    }

    fn replace(&mut self, win: usize, lose: usize) {
        let n = self.size();
        match self.config.refill {
            Refill::Copy => {
                for k in 0..n {
                    self.cooperate[lose * n + k] = self.cooperate[win * n + k];
                    self.punish[lose * n + k] = self.punish[win * n + k];
                }
            }
            Refill::Split => {
                let c: Vec<f64> = self.cooperate[win * n..win * n + n].to_vec();
                let p: Vec<f64> = self.punish[win * n..win * n + n].to_vec();
                for site in [win, lose] {
                    for k in 0..n {
                        let s = self.rng.gen_range(0..n as u32) as usize;
                        self.cooperate[site * n + k] = c[s];
                        self.punish[site * n + k] = p[s];
                    }
                }
            }
        }
    }

    fn mutate(&mut self) {
        let mu = self.config.mutation;
        let continuous = self.config.traits == Traits::Continuous;
        for a in 0..self.cooperate.len() {
            if self.rng.gen::<f64>() >= mu {
                continue;
            }
            if continuous {
                self.cooperate[a] = self.rng.gen::<f64>();
                self.punish[a] = self.rng.gen::<f64>();
            } else {
                let now = kind(self.cooperate[a], self.punish[a]);
                let next = (now + 1 + self.rng.gen_range(0..2u32) as usize) % 3;
                (self.cooperate[a], self.punish[a]) = TYPES[next];
            }
        }
    }

    /// The long-run window's first period.
    fn window_start(&self) -> u64 {
        let (stop, window) = (
            u64::from(self.config.stop_at),
            u64::from(self.config.window),
        );
        if stop > window {
            stop - window + 1
        } else {
            1
        }
    }

    fn record(&mut self) {
        let (g, n) = (self.groups(), self.size());
        let total = (g * n) as f64;
        let (mut coop, mut pun, mut contributors, mut punishers, mut defectors) =
            (0.0, 0.0, 0.0, 0.0, 0.0);
        for a in 0..g * n {
            let (x, y) = (self.cooperate[a], self.punish[a]);
            coop += x;
            pun += y;
            contributors += x * (1.0 - y);
            punishers += x * y;
            defectors += 1.0 - x;
        }
        let cooperation = coop / total;
        let group_coop: Vec<f64> = (0..g)
            .map(|gi| self.cooperate[gi * n..gi * n + n].iter().sum::<f64>() / n as f64)
            .collect();
        let mean = group_coop.iter().sum::<f64>() / g as f64;
        let spread = (group_coop
            .iter()
            .map(|v| (v - mean) * (v - mean))
            .sum::<f64>()
            / g as f64)
            .sqrt();
        // The long-run window moves when `stop_at` or `window` changes; its
        // sum is then rebuilt from the history.
        let from = self.window_start();
        if from != self.window_from {
            self.window_from = from;
            self.window_sum = self
                .stats
                .history()
                .iter()
                .filter(|s| s.tick >= from)
                .map(|s| s.cooperation)
                .sum();
        }
        let long_run = if self.tick >= from {
            self.window_sum += cooperation;
            self.window_sum / (self.tick - from + 1) as f64
        } else {
            f64::NAN
        };
        let acts = if self.tick == 0 {
            f64::NAN
        } else {
            self.cooperated.iter().filter(|&&c| c).count() as f64 / total
        };
        let payoff = if self.tick == 0 {
            f64::NAN
        } else {
            self.payoff.iter().sum::<f64>() / total
        };
        let wide = self.frame_wide();
        if self.recent.len() == wide {
            self.recent.pop_front();
        }
        self.recent.push_back((cooperation, pun / total));
        self.stats.push(PunishmentSnapshot {
            tick: self.tick,
            cooperation,
            contributors: contributors / total,
            punishers: punishers / total,
            defectors: defectors / total,
            punishment: pun / total,
            acts,
            payoff,
            conflicts: self.conflicts,
            extinctions: self.extinctions,
            spread,
            long_run,
        });
    }

    fn mosaic(&self) -> Mosaic {
        Mosaic::new(self.config.groups, self.config.size)
    }

    /// The frame's width: the mosaic, or the time strip if wider.
    fn frame_wide(&self) -> usize {
        self.mosaic().extent().0.max(LEAST_WIDE)
    }

    fn view(&self, a: usize) -> AgentView {
        let (x, y) = self.traits(a);
        AgentView {
            id: a as u64 + 1,
            group: (a / self.size()) as u32,
            kind: (self.config.traits == Traits::Discrete)
                .then(|| [Kind::Contributor, Kind::Defector, Kind::Punisher][kind(x, y)]),
            cooperate: x,
            punish: y,
            cooperated: self.cooperated[a],
            punished: self.punished[a],
            payoff: self.payoff[a],
        }
    }

    fn group_view(&self, gi: usize) -> GroupView {
        let n = self.size();
        let nf = n as f64;
        let range = gi * n..gi * n + n;
        let (mut c, mut p, mut d) = (0.0, 0.0, 0.0);
        for a in range.clone() {
            let (x, y) = self.traits(a);
            c += x * (1.0 - y);
            p += x * y;
            d += 1.0 - x;
        }
        GroupView {
            index: gi as u32,
            contributors: c / nf,
            punishers: p / nf,
            defectors: d / nf,
            acts: range.clone().filter(|&a| self.cooperated[a]).count() as f64 / nf,
            payoff: range.map(|a| self.payoff[a]).sum::<f64>() / nf,
            last_conflict: self.fought[gi].map(|f| f.0),
            lost: self.fought[gi].is_some_and(|f| f.1),
        }
    }

    fn color(&self, mode: PunishmentMode, a: usize, group_d: f64) -> [u8; 3] {
        let (x, y) = self.traits(a);
        match mode {
            PunishmentMode::Type => traits_color(x, y),
            PunishmentMode::Acts => match (self.cooperated[a], self.punished[a]) {
                (true, true) => PUNISHER,
                (true, false) => CONTRIBUTOR,
                (false, true) => ERRED,
                (false, false) => DEFECTOR,
            },
            PunishmentMode::Payoff => {
                let top = (self.config.baseline + self.config.benefit).max(f64::MIN_POSITIVE);
                scale(self.payoff[a] / top, LOW, HIGH)
            }
            PunishmentMode::Group => scale(group_d, CONTRIBUTOR, DEFECTOR),
        }
    }

    /// The agent at pixel (x, y) of the mosaic, if any.
    fn agent_at(&self, x: usize, y: usize) -> Option<usize> {
        let m = self.mosaic();
        let (bw, bh) = m.block();
        for gi in 0..self.groups() {
            let (ox, oy) = m.origin(gi);
            if (ox..ox + bw).contains(&x) && (oy..oy + bh).contains(&y) {
                let k = (y - oy) / m.cell * m.wide + (x - ox) / m.cell;
                return (k < self.size()).then_some(gi * self.size() + k);
            }
        }
        None
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<PunishmentInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let mut out = PunishmentInspection {
            site: PunishmentCell { x, y },
            panel: None,
            group: None,
            agent: None,
            period: None,
            cooperation: None,
            punishment: None,
        };
        let (cx, cy) = (x as usize, y as usize);
        let (_, mh) = self.mosaic().extent();
        if cy < mh {
            if let Some(a) = self.agent_at(cx, cy) {
                out.panel = Some("groups");
                out.agent = Some(self.view(a));
                out.group = Some(self.group_view(a / self.size()));
            }
        } else if cy >= mh + STRIP_GAP {
            if let Some(&(coop, pun)) = self.recent.get(cx) {
                out.panel = Some("time");
                out.period = Some(self.tick + 1 + cx as u64 - self.recent.len() as u64);
                out.cooperation = Some(coop);
                out.punishment = Some(pun);
            }
        }
        Ok(out)
    }
}

/// tanh from `+ − × ÷` and `exp_neg` (portable).
fn tanh(u: f64) -> f64 {
    let e = exp_neg(-2.0 * u.abs());
    ((1.0 - e) / (1.0 + e)).copysign(u)
}

/// The widest possible difference between two groups' expected mean payoffs
/// (Cooney's G_max − G_min). With y defectors and z punishers the expected
/// mean is baseline + (b − c)(1 − y) − a·y·z − q·z (a = p + k and q = 0 for
/// variable costs; a = p and q the fixed cost for fixed ones): linear in z,
/// so its extremes lie at the corners or on the edge z = 1 − y, a quadratic
/// in y with one stationary point.
pub fn payoff_range(c: &PunishmentConfig) -> f64 {
    let (a, q) = match c.punishing {
        Punishing::Variable => (c.fine + c.punish_cost, 0.0),
        Punishing::Fixed => (c.fine, c.fixed_cost),
    };
    let g = |y: f64, z: f64| (c.benefit - c.cost) * (1.0 - y) - a * y * z - q * z;
    let mut points = vec![g(0.0, 0.0), g(1.0, 0.0), g(0.0, 1.0)];
    if a > 0.0 {
        let y = (a + c.benefit - c.cost - q) / (2.0 * a);
        if (0.0..=1.0).contains(&y) {
            points.push(g(y, 1.0 - y));
        }
    }
    let hi = points.iter().copied().fold(f64::MIN, f64::max);
    let lo = points.iter().copied().fold(f64::MAX, f64::min);
    hi - lo
}

impl Model for PunishmentWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Punishment(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        PunishmentWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.cooperate.len()
    }

    /// FNV-1a over the tick and every agent's traits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        for a in 0..self.cooperate.len() {
            eat(&self.cooperate[a].to_bits().to_le_bytes());
            eat(&self.punish[a].to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (_, mh) = self.mosaic().extent();
        (self.frame_wide() as u32, (mh + STRIP_GAP + STRIP) as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: PunishmentMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let m = self.mosaic();
        let (bw, bh) = m.block();
        let n = self.size();
        for gi in 0..self.groups() {
            let (ox, oy) = m.origin(gi);
            let d = (gi * n..gi * n + n)
                .map(|a| 1.0 - self.cooperate[a])
                .sum::<f64>()
                / n as f64;
            if self.fought[gi] == Some((self.tick, true)) {
                for x in ox - 1..=ox + bw {
                    c.put(x, oy - 1, LOST);
                    c.put(x, oy + bh, LOST);
                }
                c.column(ox - 1, oy - 1, oy + bh, LOST);
                c.column(ox + bw, oy - 1, oy + bh, LOST);
            }
            for k in 0..n {
                let color = self.color(mode, gi * n + k, d);
                let (px, py) = (ox + (k % m.wide) * m.cell, oy + (k / m.wide) * m.cell);
                for dy in 0..m.cell {
                    for dx in 0..m.cell {
                        c.put(px + dx, py + dy, color);
                    }
                }
            }
        }
        // Cooperation (blue) and punishment (green) over the last periods.
        let top = m.extent().1 + STRIP_GAP;
        for x in 0..fw as usize {
            c.put(x, top + row(0.5), MARK);
        }
        for (k, &(coop, pun)) in self.recent.iter().enumerate() {
            let next = self.recent.get(k + 1).copied().unwrap_or((coop, pun));
            c.column(k, top + row(pun), top + row(next.1), PUNISHER);
            c.column(k, top + row(coop), top + row(next.0), CONTRIBUTOR);
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
        let mut out = String::from("id,group,kind,cooperate,punish,cooperated,punished,payoff\n");
        for a in 0..self.cooperate.len() {
            let v = self.view(a);
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                v.id,
                v.group,
                v.kind
                    .map_or(String::new(), |k| format!("{k:?}").to_lowercase()),
                v.cooperate,
                v.punish,
                u8::from(v.cooperated),
                u8::from(v.punished),
                v.payoff
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Agents stay where they are: agent `id`'s cell in its group's block.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let a = usize::try_from(id.checked_sub(1)?).ok()?;
        if a >= self.cooperate.len() {
            return None;
        }
        let m = self.mosaic();
        let (ox, oy) = m.origin(a / self.size());
        let k = a % self.size();
        Some((
            (ox + (k % m.wide) * m.cell) as u32,
            (oy + (k / m.wide) * m.cell) as u32,
        ))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Punishment(next) = next else {
            return Err(wrong_model(ModelKind::Punishment, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped at `stop_at`: a sweep reads its last period (the long-run average).
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(edit: impl FnOnce(&mut PunishmentConfig)) -> PunishmentConfig {
        let mut c = PunishmentConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut PunishmentConfig)) -> PunishmentWorld {
        PunishmentWorld::new(config(edit), 1).unwrap()
    }

    /// A world whose first group holds `types` (0 contributor, 1 defector,
    /// 2 punisher) and whose acts are set by hand.
    fn hand(
        edit: impl FnOnce(&mut PunishmentConfig),
        types: &[usize],
        acts: &[bool],
    ) -> PunishmentWorld {
        let mut w = world(|c| {
            c.groups = 2;
            c.size = types.len() as u32;
            edit(c);
        });
        for (a, &t) in types.iter().enumerate() {
            (w.cooperate[a], w.punish[a]) = TYPES[t];
            w.cooperated[a] = acts[a];
            let punisher = t == 2 && (acts[a] || w.config.erring != Erring::None);
            w.punished[a] = punisher;
        }
        w
    }

    #[test]
    fn it_starts_with_one_group_of_punishers() {
        let w = world(|_| {});
        let n = w.size();
        assert!((0..n).all(|a| w.traits(a) == (1.0, 1.0)));
        assert!((n..w.cooperate.len()).all(|a| w.traits(a) == (0.0, 0.0)));
        let s = w.stats.latest().unwrap();
        assert_eq!(s.cooperation, 1.0 / 128.0);
        assert!(s.long_run.is_nan() && s.acts.is_nan());
        let all = world(|c| c.start = Start::AllDefectors);
        assert_eq!(all.stats.latest().unwrap().cooperation, 0.0);
    }

    #[test]
    fn payoffs_follow_the_game() {
        // A contributor, a defector, two punishers; everyone acts to type.
        let mut w = hand(|_| {}, &[0, 1, 2, 2], &[true, false, true, true]);
        let (means, ds) = w.payoffs();
        let n = 4.0;
        let p = &w.payoff;
        assert!((p[0] - (1.0 - 0.2)).abs() < 1e-12);
        assert!((p[1] - (1.0 - 0.8 * 2.0 / n)).abs() < 1e-12);
        assert!((p[2] - (1.0 - 0.2 - 0.2 / n)).abs() < 1e-12);
        assert!((means[0] - p[..4].iter().sum::<f64>() / n).abs() < 1e-12);
        assert!((ds[0] - 0.25).abs() < 1e-12);
        // With a benefit, each cooperator gives b/n to every other member.
        let mut b = hand(
            |c| c.benefit = 0.4,
            &[0, 1, 2, 2],
            &[true, false, true, true],
        );
        b.payoffs();
        assert!((b.payoff[1] - (1.0 + 0.4 * 3.0 / n - 0.8 * 2.0 / n)).abs() < 1e-12);
        assert!((b.payoff[0] - (1.0 + 0.4 * 2.0 / n - 0.2)).abs() < 1e-12);
        // A fixed cost is paid whatever the group.
        let mut f = hand(
            |c| c.punishing = Punishing::Fixed,
            &[0, 0, 2, 2],
            &[true; 4],
        );
        f.payoffs();
        assert!((f.payoff[2] - (1.0 - 0.2 - 0.2)).abs() < 1e-12);
        // Payoffs never fall below 0.
        let mut z = hand(|c| c.fine = 10.0, &[1, 2, 2, 2], &[false, true, true, true]);
        z.payoffs();
        assert_eq!(z.payoff[0], 0.0);
    }

    #[test]
    fn a_punisher_who_errs_follows_its_rule() {
        // Two punishers; the first defects by mistake.
        let types = [2, 2, 1, 0];
        let acts = [false, true, false, true];
        let n = 4.0;
        let mut others = hand(|_| {}, &types, &acts);
        others.payoffs();
        // It punishes the other defector and is punished by the other punisher.
        assert!((others.payoff[0] - (1.0 - 0.8 / n - 0.2 / n)).abs() < 1e-12);
        assert!((others.payoff[2] - (1.0 - 0.8 * 2.0 / n)).abs() < 1e-12);
        let mut none = hand(|c| c.erring = Erring::None, &types, &acts);
        none.payoffs();
        assert!((none.payoff[0] - (1.0 - 0.8 / n)).abs() < 1e-12);
        assert!((none.payoff[2] - (1.0 - 0.8 / n)).abs() < 1e-12);
        let mut itself = hand(|c| c.erring = Erring::Itself, &types, &acts);
        itself.payoffs();
        assert!((itself.payoff[0] - (1.0 - 0.8 * 2.0 / n - 0.2 * 2.0 / n)).abs() < 1e-12);
    }

    #[test]
    fn imitation_copies_with_the_payoff_ratio() {
        // Two agents per group; agent 0 earns 1, agent 1 earns 3: agent 0
        // copies agent 1 three times in four.
        let mut copied = 0;
        for seed in 0..4000 {
            let mut w = PunishmentWorld::new(
                config(|c| {
                    c.groups = 2;
                    c.size = 2;
                    c.mixing = 0.0;
                }),
                seed,
            )
            .unwrap();
            (w.cooperate[0], w.punish[0]) = (0.0, 0.0);
            (w.cooperate[1], w.punish[1]) = (1.0, 1.0);
            w.payoff[..2].copy_from_slice(&[1.0, 3.0]);
            w.imitate();
            if w.cooperate[0] == 1.0 {
                copied += 1;
            }
        }
        assert!((2850..=3150).contains(&copied), "{copied} of 4000");
    }

    #[test]
    fn together_imitates_from_the_starting_traits_and_in_turn_does_not() {
        // A group of a defector and a punisher, earning the same: together,
        // both can copy the other's old type and swap; in turn, the second
        // copies the first's new type, so they never swap.
        let swaps = |imitation| {
            (0..400)
                .filter(|&seed| {
                    let mut w = PunishmentWorld::new(
                        config(|c| {
                            c.groups = 2;
                            c.size = 2;
                            c.mixing = 0.0;
                            c.imitation = imitation;
                        }),
                        seed,
                    )
                    .unwrap();
                    (w.cooperate[0], w.punish[0]) = TYPES[1];
                    (w.cooperate[1], w.punish[1]) = TYPES[2];
                    w.payoff[..2].copy_from_slice(&[1.0, 1.0]);
                    w.imitate();
                    kind(w.cooperate[0], w.punish[0]) == 2 && kind(w.cooperate[1], w.punish[1]) == 1
                })
                .count()
        };
        let together = swaps(Imitation::Together);
        assert!((70..=130).contains(&together), "{together} of 400");
        assert_eq!(swaps(Imitation::InTurn), 0);
    }

    #[test]
    fn migrants_come_from_other_groups_and_on_a_ring_from_neighbors() {
        for seed in 0..20 {
            // Group 0: a defector earning 0 beside a punisher earning 100;
            // every other group contributors earning 1. With m = 1 the
            // defector copies a contributor, never its own group's punisher.
            let mut w = PunishmentWorld::new(
                config(|c| {
                    c.groups = 6;
                    c.size = 2;
                    c.mixing = 1.0;
                }),
                seed,
            )
            .unwrap();
            for a in 0..12 {
                (w.cooperate[a], w.punish[a]) = TYPES[if a == 0 {
                    1
                } else if a == 1 {
                    2
                } else {
                    0
                }];
                w.payoff[a] = match a {
                    0 => 0.0,
                    1 => 100.0,
                    _ => 1.0,
                };
            }
            w.imitate();
            assert_eq!(kind(w.cooperate[0], w.punish[0]), 0);
            // On a ring, group 0's neighbors (1 and 5) are punishers and the
            // rest contributors: its defectors become punishers.
            let mut r = PunishmentWorld::new(
                config(|c| {
                    c.groups = 6;
                    c.size = 2;
                    c.mixing = 1.0;
                    c.structure = Structure::Ring;
                }),
                seed,
            )
            .unwrap();
            for a in 0..12 {
                let t = match a / 2 {
                    0 => 1,
                    1 | 5 => 2,
                    _ => 0,
                };
                (r.cooperate[a], r.punish[a]) = TYPES[t];
                r.payoff[a] = if a < 2 { 0.0 } else { 1.0 };
            }
            r.imitate();
            assert_eq!(kind(r.cooperate[0], r.punish[0]), 2);
            assert_eq!(kind(r.cooperate[1], r.punish[1]), 2);
        }
    }

    #[test]
    fn paired_groups_fight_at_half_the_rate_and_either_or_challenge_about_the_whole() {
        let rate = |pairing| {
            let mut w = world(|c| {
                c.pairing = pairing;
                c.conflict = 0.1;
                c.groups = 64;
                c.size = 2;
            });
            let (means, ds) = (vec![1.0; 64], vec![0.5; 64]);
            let mut fights = 0;
            for _ in 0..200 {
                w.conflict(&means, &ds);
                fights += w.conflicts;
            }
            f64::from(fights) / (200.0 * 64.0)
        };
        let paired = rate(Pairing::Paired);
        let either = rate(Pairing::Either);
        let challenge = rate(Pairing::Challenge);
        assert!((paired - 0.05).abs() < 0.01, "{paired}");
        assert!((either - 0.095).abs() < 0.012, "{either}");
        assert!((challenge - 0.09).abs() < 0.015, "{challenge}");
    }

    #[test]
    fn victory_follows_each_rule() {
        let w = world(|_| {});
        let (means, ds) = (vec![1.0, 0.5], vec![0.2, 0.6]);
        assert!((w.victory(0, 1, &means, &ds) - 0.7).abs() < 1e-12);
        let p = world(|c| c.victory = Victory::Payoff);
        let range = payoff_range(&p.config);
        assert!((range - 0.36).abs() < 1e-12);
        let close = [1.0, 0.95];
        assert!((p.victory(0, 1, &close, &ds) - (0.5 + 0.025 / range)).abs() < 1e-12);
        assert_eq!(p.victory(0, 1, &means, &ds), 1.0);
        let t = world(|c| {
            c.victory = Victory::Tanh;
            c.sensitivity = 2.0;
        });
        assert!((t.victory(0, 1, &means, &ds) - (0.5 + 0.5 * 1f64.tanh())).abs() < 1e-12);
        assert!((t.victory(1, 0, &means, &ds) - (0.5 - 0.5 * 1f64.tanh())).abs() < 1e-12);
        // No possible difference: a draw.
        let flat = world(|c| {
            c.victory = Victory::Payoff;
            c.cost = 0.0;
            c.fine = 0.0;
            c.punish_cost = 0.0;
        });
        assert_eq!(payoff_range(&flat.config), 0.0);
        assert_eq!(flat.victory(0, 1, &means, &ds), 0.5);
    }

    #[test]
    fn the_payoff_range_matches_a_grid() {
        for (b, p, k, fixed) in [
            (0.0, 0.8, 0.2, false),
            (0.4, 0.8, 0.2, false),
            (1.6, 0.4, 0.2, true),
            (0.8, 2.4, 0.05, false),
        ] {
            let c = config(|c| {
                c.benefit = b;
                c.fine = p;
                c.punish_cost = k;
                if fixed {
                    c.punishing = Punishing::Fixed;
                    c.fixed_cost = 0.1;
                }
            });
            let (a, q) = if fixed { (p, 0.1) } else { (p + k, 0.0) };
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for i in 0..=400 {
                for j in 0..=(400 - i) {
                    let (y, z) = (i as f64 / 400.0, j as f64 / 400.0);
                    let g = (b - 0.2) * (1.0 - y) - a * y * z - q * z;
                    lo = lo.min(g);
                    hi = hi.max(g);
                }
            }
            assert!((payoff_range(&c) - (hi - lo)).abs() < 1e-4, "{b} {p} {k}");
        }
    }

    #[test]
    fn losers_become_the_winners() {
        let mut w = world(|c| {
            c.groups = 4;
            c.size = 3;
        });
        for a in 0..3 {
            (w.cooperate[a], w.punish[a]) = TYPES[[0, 2, 2][a]];
        }
        w.replace(0, 2);
        assert_eq!(&w.cooperate[6..9], &w.cooperate[0..3]);
        assert_eq!(&w.punish[6..9], &w.punish[0..3]);
        let mut s = world(|c| {
            c.groups = 4;
            c.size = 3;
            c.refill = Refill::Split;
        });
        for a in 0..3 {
            (s.cooperate[a], s.punish[a]) = TYPES[[0, 2, 2][a]];
        }
        s.replace(0, 2);
        for a in (0..3).chain(6..9) {
            assert!(matches!(kind(s.cooperate[a], s.punish[a]), 0 | 2));
        }
    }

    #[test]
    fn mutants_switch_to_another_type_or_draw_new_traits() {
        let mut w = world(|c| {
            c.mutation = 1.0;
            c.groups = 2;
            c.size = 50;
            c.start = Start::AllDefectors;
        });
        w.mutate();
        let kinds: Vec<usize> = (0..100)
            .map(|a| kind(w.cooperate[a], w.punish[a]))
            .collect();
        assert!(kinds.iter().all(|&k| k != 1));
        assert!(kinds.contains(&0) && kinds.contains(&2));
        let mut c = world(|c| {
            c.mutation = 1.0;
            c.traits = Traits::Continuous;
        });
        c.mutate();
        assert!(c.cooperate.iter().all(|x| (0.0..1.0).contains(x)));
        assert!(c.cooperate.iter().any(|&x| x > 0.0 && x < 1.0));
    }

    #[test]
    fn the_long_run_average_covers_its_window() {
        let mut w = world(|c| {
            c.stop_at = 40;
            c.window = 10;
            c.groups = 8;
            c.size = 4;
        });
        w.run(100);
        assert_eq!(w.tick, 40);
        assert!(w.is_finished());
        let coop = w.series("cooperation").unwrap();
        let want = coop[31..=40].iter().sum::<f64>() / 10.0;
        assert!((w.latest_value("long_run").unwrap() - want).abs() < 1e-12);
        assert!(w.series("long_run").unwrap()[30].is_nan());
        // Moving the window rebuilds the average from the history.
        let mut next = w.config.clone();
        next.stop_at = 50;
        next.window = 30;
        Model::set_config(&mut w, ModelConfig::Punishment(next)).unwrap();
        w.run(10);
        let coop = w.series("cooperation").unwrap();
        let want = coop[21..=50].iter().sum::<f64>() / 30.0;
        assert!((w.latest_value("long_run").unwrap() - want).abs() < 1e-12);
    }

    #[test]
    fn punishment_sustains_cooperation_that_its_absence_loses() {
        let base = |c: &mut PunishmentConfig| {
            c.groups = 64;
            c.size = 16;
            c.stop_at = 1000;
        };
        let with = world(base);
        let without = world(|c| {
            base(c);
            c.fine = 0.0;
            c.punish_cost = 0.0;
        });
        let (mut a, mut b) = (with, without);
        a.run(1000);
        b.run(1000);
        let (x, y) = (
            a.latest_value("cooperation").unwrap(),
            b.latest_value("cooperation").unwrap(),
        );
        assert!(x > 0.5 && y < 0.3, "{x} against {y}");
    }

    #[test]
    fn stats_add_up() {
        let mut w = world(|c| {
            c.groups = 16;
            c.size = 8;
            c.traits = Traits::Continuous;
            c.mutation = 0.2;
        });
        w.run(20);
        let s = w.stats.latest().unwrap().clone();
        assert!((s.contributors + s.punishers + s.defectors - 1.0).abs() < 1e-9);
        assert!((s.contributors + s.punishers - s.cooperation).abs() < 1e-9);
        assert!(s.spread >= 0.0 && s.payoff > 0.0 && (0.0..=1.0).contains(&s.acts));
        assert_eq!(s.conflicts, s.extinctions);
    }

    #[test]
    fn the_view_and_inspect_read_agents_groups_and_periods() {
        let mut w = world(|_| {});
        w.run(5);
        let mut buf = Vec::new();
        for mode in ["type", "acts", "payoff", "group"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw, fh), (514, 258 + 6 + 60));
        let (x, y) = Model::locate(&w, 33).unwrap();
        assert_eq!((x, y), (34, 2));
        let i = w.inspect(x, y).unwrap();
        assert_eq!(i.panel, Some("groups"));
        assert_eq!(i.agent.as_ref().unwrap().id, 33);
        assert_eq!(i.group.as_ref().unwrap().index, 1);
        let t = w.inspect(3, 258 + 6 + 10).unwrap();
        assert_eq!((t.panel, t.period), (Some("time"), Some(3)));
        assert_eq!(w.inspect(0, 0).unwrap().panel, None);
        assert!(w.inspect(fw, 0).is_err());
        assert_eq!(Model::locate(&w, 0), None);
        assert_eq!(Model::locate(&w, 128 * 32 + 1), None);
    }

    #[test]
    fn tanh_is_portable_and_right() {
        for u in [-5.0, -1.0, -0.1, 0.0, 0.3, 2.0, 30.0] {
            assert!((tanh(u) - f64::tanh(u)).abs() < 1e-14, "{u}");
        }
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let mut next = w.config.clone();
        next.fine = 0.4;
        next.victory = Victory::Tanh;
        Model::set_config(&mut w, ModelConfig::Punishment(next.clone())).unwrap();
        assert_eq!(w.config.fine, 0.4);
        next.size = 16;
        assert!(Model::set_config(&mut w, ModelConfig::Punishment(next)).is_err());
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for edit in [
            (|c: &mut PunishmentConfig| {
                c.groups = 2;
                c.size = 2;
            }) as fn(&mut PunishmentConfig),
            |c| {
                c.error = 1.0;
                c.mixing = 1.0;
                c.mutation = 1.0;
                c.conflict = 1.0;
            },
            |c| {
                c.error = 0.0;
                c.mixing = 0.0;
                c.mutation = 0.0;
                c.conflict = 0.0;
            },
            |c| c.baseline = 0.0,
            |c| {
                c.fine = 0.0;
                c.cost = 0.0;
                c.punish_cost = 0.0;
                c.victory = Victory::Payoff;
            },
            |c| {
                c.structure = Structure::Ring;
                c.groups = 2;
            },
            |c| {
                c.pairing = Pairing::Challenge;
                c.conflict = 1.0;
                c.groups = 3;
                c.refill = Refill::Split;
            },
        ] {
            let mut w = world(|c| {
                c.groups = c.groups.min(16);
                c.size = c.size.min(8);
                edit(c);
            });
            w.run(30);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.cooperation));
        }
    }
}
