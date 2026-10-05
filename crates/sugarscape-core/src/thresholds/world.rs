//! The Threshold Models world. An episode is one crowd (or network) and one
//! trigger: each step, actors act when the others acting reach their
//! threshold of the group they watch, until nothing changes. Without ceilings
//! or clusters nobody stops (Granovetter's model without "removal"); with
//! them every state is recomputed each step, and the crowd may never settle.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{Distribution, Network, Population, ThresholdsConfig, Trigger, Update, Zero};
use super::crowd::{self, Th};
use super::stats::{ThresholdsSnapshot, RECENT};
use super::theory;
use super::view::{
    col, grid, row, scale, share_at, ACTING, BAR, CROWDS, FEW, GRID_X, HIGH, IDLE, LINE, LOW, MANY,
    MARK, MID_W, MID_X, PATH, SEED, SHOWN, TALL, TIME_W,
};
use crate::config::FieldError;
use crate::export;
use crate::graph;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThresholdsMode {
    State,
    Threshold,
    Degree,
    Crowd,
}

impl std::str::FromStr for ThresholdsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "state" => Self::State,
            "threshold" => Self::Threshold,
            "degree" => Self::Degree,
            "crowd" => Self::Crowd,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ThresholdsInspection {
    pub site: ThresholdsCell,
    /// `time`, `figure` (Granovetter's Fig. 1), `histogram` or `actors`;
    /// null between panels.
    pub panel: Option<&'static str>,
    /// The step of a time-panel column.
    pub step: Option<u64>,
    /// Each crowd's share acting at that step.
    pub crowds: Option<Vec<f64>>,
    /// The share a Figure 1 column or histogram row stands for.
    pub share: Option<f64>,
    /// Figure 1: the share whose threshold is at most that share.
    pub cdf: Option<f64>,
    /// Histogram: the episodes that ended there.
    pub count: Option<u32>,
    /// The actor at a grid cell.
    pub member: Option<ActorView>,
    /// Always null: cells are read where they are.
    pub agent: Option<ActorView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ThresholdsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ActorView {
    pub id: u64,
    /// Its threshold, as a fraction (null: never acts).
    pub threshold: Option<f64>,
    /// The level above which it stops, if it has a ceiling.
    pub ceiling: Option<f64>,
    /// Whom it watches: its neighbors or friends (the whole crowd: null).
    pub degree: Option<u32>,
    /// The others acting it sees (weighted by friendship), and of how many.
    pub sees: u64,
    pub of: u64,
    pub acting: bool,
    /// Switched on to start the episode.
    pub seed: bool,
    /// Its crowd, from 1.
    pub crowd: u32,
}

/// One step of the time panel: whether an episode began there, and each
/// crowd's share acting.
#[derive(Clone, Debug, PartialEq)]
struct Column {
    start: bool,
    shares: Vec<f64>,
}

#[derive(Clone)]
pub struct ThresholdsWorld {
    pub config: ThresholdsConfig,
    /// Completed steps.
    pub tick: u64,
    rng: SimRng,
    th: Vec<Th>,
    ceiling: Vec<bool>,
    /// Whom each actor watches (friends or neighbors; empty under
    /// `everyone` without friends), and, when ties are one-way, who watches it.
    sees: Arc<Vec<Vec<u32>>>,
    watched_by: Option<Arc<Vec<Vec<u32>>>>,
    crowd_of: Vec<u32>,
    crowd_size: Vec<u32>,
    crowd_acting: Vec<u32>,
    acting: Vec<bool>,
    seed: Vec<bool>,
    /// For each actor, how many of those it watches act.
    seen_acting: Vec<u32>,
    total: u32,
    step: u32,
    settled: bool,
    /// This episode's acting count after each step (Figure 1's staircase).
    path: Vec<u32>,
    recent: VecDeque<Column>,
    window: VecDeque<u32>,
    /// Finished episodes by final share (percent).
    sizes: [u32; 101],
    episodes: u32,
    size_sum: f64,
    globals: u32,
    last_size: f64,
    theory: f64,
    pub stats: Stats<ThresholdsSnapshot>,
}

/// Granovetter's continuous equilibrium share, when the crowd is a normal
/// one seen whole.
fn theory_share(c: &ThresholdsConfig) -> f64 {
    if c.distribution == Distribution::Normal
        && c.network == Network::Everyone
        && !c.friends.enabled
        && c.population == Population::Fixed
        && !c.reversible()
    {
        theory::granovetter(c.actors, c.mean, c.sd) / f64::from(c.actors)
    } else {
        f64::NAN
    }
}

impl ThresholdsWorld {
    pub fn new(config: ThresholdsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let n = config.population_size() as usize;
        let mut world = ThresholdsWorld {
            theory: theory_share(&config),
            config,
            tick: 0,
            rng: rng::seeded(seed),
            th: Vec::new(),
            ceiling: vec![false; n],
            sees: Arc::new(Vec::new()),
            watched_by: None,
            crowd_of: vec![0; n],
            crowd_size: Vec::new(),
            crowd_acting: Vec::new(),
            acting: vec![false; n],
            seed: vec![false; n],
            seen_acting: vec![0; n],
            total: 0,
            step: 0,
            settled: false,
            path: Vec::new(),
            recent: VecDeque::new(),
            window: VecDeque::new(),
            sizes: [0; 101],
            episodes: 0,
            size_sum: 0.0,
            globals: 0,
            last_size: 0.0,
            stats: Stats::default(),
        };
        world.begin();
        world.remember(true);
        world.record();
        Ok(world)
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    /// Actors in the world.
    pub fn size(&self) -> usize {
        self.acting.len()
    }

    pub fn thresholds(&self) -> &[Th] {
        &self.th
    }

    pub fn acting(&self) -> &[bool] {
        &self.acting
    }

    /// Whom each actor watches (empty lists under `everyone` without friends).
    pub fn watches(&self) -> &[Vec<u32>] {
        &self.sees
    }

    /// Finished episodes by final share, in percent.
    pub fn sizes(&self) -> &[u32; 101] {
        &self.sizes
    }

    /// Starts an episode: draws the crowd (and its friends or network),
    /// the ceilings and the seed.
    fn begin(&mut self) {
        let c = self.config.clone();
        let n = c.population_size();
        let nu = n as usize;
        self.th = match c.population {
            Population::City => crowd::city(n, &mut self.rng),
            Population::Fixed => crowd::draw(&c, &mut self.rng),
        };
        self.ceiling = vec![false; nu];
        let q = (c.ceilings.share * f64::from(n)).round() as usize;
        if q > 0 {
            let mut order: Vec<u32> = (0..n).collect();
            for i in 0..q.min(nu) {
                let j = i + self.rng.gen_range(0..(nu - i) as u32) as usize;
                order.swap(i, j);
                self.ceiling[order[i] as usize] = true;
            }
        }
        self.watched_by = None;
        self.sees = Arc::new(match c.network {
            Network::Everyone if c.friends.enabled => {
                let a = c.friends.acquaintance;
                if c.friends.symmetric {
                    graph::gnp_sparse(nu, a, &mut self.rng)
                } else {
                    let mut sees = vec![Vec::new(); nu];
                    let mut by = vec![Vec::new(); nu];
                    for (i, row) in sees.iter_mut().enumerate() {
                        for (j, watchers) in by.iter_mut().enumerate() {
                            if i != j && self.rng.gen::<f64>() < a {
                                row.push(j as u32);
                                watchers.push(i as u32);
                            }
                        }
                    }
                    self.watched_by = Some(Arc::new(by));
                    sees
                }
            }
            Network::Everyone => Vec::new(),
            Network::Random => graph::gnp_sparse(nu, c.degree / f64::from(n - 1), &mut self.rng),
            Network::PowerLaw => {
                let p = crowd::power_law(c.degree, (n - 1).min(10_000));
                let d = crowd::degrees(&p, n, &mut self.rng);
                graph::configuration(&d, &mut self.rng)
            }
        });
        let k = if c.clusters.enabled {
            c.clusters.count
        } else {
            1
        };
        self.crowd_of = (0..n).map(|i| i / c.actors).collect();
        self.crowd_size = vec![c.actors; k as usize];
        self.crowd_acting = vec![0; k as usize];
        self.acting = vec![false; nu];
        self.seed = vec![false; nu];
        self.seen_acting = vec![0; nu];
        self.total = 0;
        let start = match c.trigger {
            Trigger::Instigators => None,
            Trigger::Random => Some(self.rng.gen_range(0..n) as usize),
            Trigger::Hub => (0..nu).rev().max_by_key(|&i| self.sees[i].len()),
        };
        if let Some(s) = start {
            self.seed[s] = true;
            self.set(s, true);
        }
        self.step = 0;
        self.settled = false;
        self.path = vec![self.total];
    }

    fn set(&mut self, i: usize, on: bool) {
        if self.acting[i] == on {
            return;
        }
        self.acting[i] = on;
        let d: i64 = if on { 1 } else { -1 };
        self.total = (i64::from(self.total) + d) as u32;
        let k = self.crowd_of[i] as usize;
        self.crowd_acting[k] = (i64::from(self.crowd_acting[k]) + d) as u32;
        let watchers = match &self.watched_by {
            Some(by) => Arc::clone(by),
            None => Arc::clone(&self.sees),
        };
        if !watchers.is_empty() {
            for &j in &watchers[i] {
                let j = j as usize;
                self.seen_acting[j] = (i64::from(self.seen_acting[j]) + d) as u32;
            }
        }
    }

    /// What actor `i` perceives: the others acting (A) and the group (G),
    /// weighted by friendship, as integers.
    fn perceived(&self, i: usize) -> (u64, u64) {
        let c = &self.config;
        let me = u64::from(self.acting[i]);
        match c.network {
            Network::Random | Network::PowerLaw => {
                (u64::from(self.seen_acting[i]), self.sees[i].len() as u64)
            }
            Network::Everyone => {
                let k = self.crowd_of[i] as usize;
                let others = u64::from(self.crowd_acting[k]) - me;
                let size = u64::from(self.crowd_size[k]);
                let group = if c.counts_self { size } else { size - 1 };
                if c.friends.enabled {
                    let w = u64::from(c.friends.weight);
                    let fr = u64::from(self.seen_acting[i]);
                    let nf = self.sees[i].len() as u64;
                    (w * fr + (others - fr), w * nf + (group - nf))
                } else {
                    (others, group)
                }
            }
        }
    }

    /// Whether actor `i` would act now.
    fn decide(&self, i: usize) -> bool {
        let t = self.th[i];
        let (a, g) = self.perceived(i);
        let zero = t.num == 0;
        let on = if g == 0 {
            zero && self.config.zero == Zero::Acts
        } else if zero {
            self.config.zero == Zero::Acts || a >= 1
        } else {
            t.reached(a, g)
        };
        if on && self.ceiling[i] && g > 0 {
            !Th::from_fraction(self.config.ceilings.at).exceeded(a, g)
        } else {
            on
        }
    }

    /// The state actor `i` takes this step.
    fn next(&self, i: usize) -> bool {
        if self.seed[i] {
            true
        } else if self.config.reversible() {
            self.decide(i)
        } else {
            self.acting[i] || self.decide(i)
        }
    }

    /// Clusters: each actor moves to a random other crowd with probability m.
    fn move_about(&mut self) {
        let k = self.config.clusters.count;
        let m = self.config.clusters.movement;
        if m <= 0.0 {
            return;
        }
        for i in 0..self.acting.len() {
            if self.rng.gen::<f64>() >= m {
                continue;
            }
            let from = self.crowd_of[i];
            let mut to = self.rng.gen_range(0..k - 1);
            if to >= from {
                to += 1;
            }
            self.crowd_size[from as usize] -= 1;
            self.crowd_size[to as usize] += 1;
            if self.acting[i] {
                self.crowd_acting[from as usize] -= 1;
                self.crowd_acting[to as usize] += 1;
            }
            self.crowd_of[i] = to;
        }
    }

    /// The episode is over: record its size.
    fn finish(&mut self) {
        self.settled = true;
        let n = self.acting.len() as f64;
        let share = f64::from(self.total) / n;
        self.last_size = share;
        self.size_sum += share;
        self.episodes += 1;
        if share >= self.config.global {
            self.globals += 1;
        }
        self.sizes[(share * 100.0).round() as usize] += 1;
    }

    pub fn step(&mut self) {
        let mut start = false;
        if self.settled {
            if self.config.repeat {
                self.begin();
                start = true;
            }
        } else {
            if self.config.clusters.enabled {
                self.move_about();
            }
            let n = self.acting.len();
            let mut changed = false;
            match self.config.update {
                Update::Synchronous => {
                    let next: Vec<bool> = (0..n).map(|i| self.next(i)).collect();
                    for (i, on) in next.into_iter().enumerate() {
                        if on != self.acting[i] {
                            self.set(i, on);
                            changed = true;
                        }
                    }
                }
                Update::Asynchronous => {
                    let mut order: Vec<u32> = (0..n as u32).collect();
                    for i in (1..n).rev() {
                        let j = self.rng.gen_range(0..=i as u32) as usize;
                        order.swap(i, j);
                    }
                    for i in order {
                        let i = i as usize;
                        let on = self.next(i);
                        if on != self.acting[i] {
                            self.set(i, on);
                            changed = true;
                        }
                    }
                }
            }
            self.step += 1;
            if self.path.len() < 10_000 {
                self.path.push(self.total);
            }
            if !self.config.clusters.enabled && (!changed || self.step >= self.config.max_steps) {
                self.finish();
            }
        }
        self.tick += 1;
        self.remember(start);
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

    fn remember(&mut self, start: bool) {
        if self.recent.len() == SHOWN {
            self.recent.pop_front();
        }
        let shares = self
            .crowd_acting
            .iter()
            .zip(&self.crowd_size)
            .map(|(&a, &s)| {
                if s == 0 {
                    0.0
                } else {
                    f64::from(a) / f64::from(s)
                }
            })
            .collect();
        self.recent.push_back(Column { start, shares });
        if self.window.len() == RECENT {
            self.window.pop_front();
        }
        self.window.push_back(self.total);
    }

    fn record(&mut self) {
        let n = self.acting.len() as f64;
        let (lo, hi) = self
            .window
            .iter()
            .fold((u32::MAX, 0), |(lo, hi), &v| (lo.min(v), hi.max(v)));
        let per = |x: u32| f64::from(x) / n;
        let e = f64::from(self.episodes);
        let s = ThresholdsSnapshot {
            tick: self.tick,
            acting: per(self.total),
            step: self.step,
            episodes: self.episodes,
            last_size: self.last_size,
            mean_size: if self.episodes == 0 {
                0.0
            } else {
                self.size_sum / e
            },
            global_share: if self.episodes == 0 {
                0.0
            } else {
                f64::from(self.globals) / e
            },
            theory: self.theory,
            recent_mean: self.window.iter().map(|&v| per(v)).sum::<f64>()
                / self.window.len() as f64,
            swing: per(hi - lo),
        };
        self.stats.push(s);
    }

    /// Whether the middle panel draws Granovetter's Figure 1: one fixed
    /// crowd, seen whole.
    fn figure_one(&self) -> bool {
        let c = &self.config;
        c.network == Network::Everyone
            && !c.friends.enabled
            && !c.clusters.enabled
            && c.population == Population::Fixed
    }

    /// The share of actors whose threshold is at most `x`.
    fn cdf(&self, x: f64) -> f64 {
        let n = self.th.len();
        let (num, den) = ((x * 1e6).round() as u64, 1_000_000u64);
        let below = self
            .th
            .iter()
            .filter(|t| u128::from(t.num) * u128::from(den) <= u128::from(num) * u128::from(t.den))
            .count();
        below as f64 / n as f64
    }

    /// Current state of the actor at the given zero-based index.
    pub fn view(&self, i: usize) -> ActorView {
        let (a, g) = self.perceived(i);
        let t = self.th[i];
        ActorView {
            id: i as u64 + 1,
            threshold: (t.num <= t.den).then(|| t.value()),
            ceiling: self.ceiling[i].then_some(self.config.ceilings.at),
            degree: (!self.sees.is_empty()).then(|| self.sees[i].len() as u32),
            sees: a,
            of: g,
            acting: self.acting[i],
            seed: self.seed[i],
            crowd: self.crowd_of[i] + 1,
        }
    }

    fn color(&self, mode: ThresholdsMode, i: usize, most: usize) -> [u8; 3] {
        match mode {
            ThresholdsMode::State => {
                if self.seed[i] {
                    SEED
                } else if self.acting[i] {
                    ACTING
                } else {
                    IDLE
                }
            }
            ThresholdsMode::Threshold => {
                let t = self.th[i];
                if t.num > t.den {
                    HIGH
                } else {
                    scale(t.value(), LOW, HIGH)
                }
            }
            ThresholdsMode::Degree => {
                let d = if self.sees.is_empty() {
                    0
                } else {
                    self.sees[i].len()
                };
                scale(
                    if most > 0 {
                        d as f64 / most as f64
                    } else {
                        0.0
                    },
                    FEW,
                    MANY,
                )
            }
            ThresholdsMode::Crowd => CROWDS[self.crowd_of[i] as usize % CROWDS.len()],
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<ThresholdsInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let mut out = ThresholdsInspection {
            site: ThresholdsCell { x, y },
            panel: None,
            step: None,
            crowds: None,
            share: None,
            cdf: None,
            count: None,
            member: None,
            agent: None,
        };
        if cx < TIME_W {
            if let Some(column) = self.recent.get(cx) {
                out.panel = Some("time");
                out.step = Some(self.tick + 1 - self.recent.len() as u64 + cx as u64);
                out.crowds = Some(column.shares.clone());
            }
        } else if (MID_X..MID_X + MID_W).contains(&cx) {
            if self.figure_one() {
                let share = (cx - MID_X) as f64 / (MID_W - 1) as f64;
                out.panel = Some("figure");
                out.share = Some(share);
                out.cdf = Some(self.cdf(share));
            } else {
                let share = share_at(cy);
                out.panel = Some("histogram");
                out.share = Some(share);
                out.count = Some(
                    (0..=100)
                        .filter(|&p| row(p as f64 / 100.0) == cy)
                        .map(|p| self.sizes[p])
                        .sum(),
                );
            }
        } else if cx >= GRID_X {
            let n = self.acting.len();
            let (side, cell) = grid(n as u32);
            let (gx, gy) = ((cx - GRID_X) / cell, cy / cell);
            let i = gy * side + gx;
            if gx < side && i < n {
                out.panel = Some("actors");
                out.member = Some(self.view(i));
            }
        }
        Ok(out)
    }
}

impl Model for ThresholdsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Thresholds(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        ThresholdsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.acting.len()
    }

    /// FNV-1a over the tick, every actor's threshold, crowd and state, and the
    /// episodes' record.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.step.to_le_bytes());
        for i in 0..self.acting.len() {
            eat(&self.th[i].num.to_le_bytes());
            eat(&self.th[i].den.to_le_bytes());
            eat(&self.crowd_of[i].to_le_bytes());
            eat(&[
                u8::from(self.acting[i]),
                u8::from(self.seed[i]),
                u8::from(self.ceiling[i]),
            ]);
        }
        for s in self.sees.iter() {
            eat(&(s.len() as u32).to_le_bytes());
        }
        eat(&self.episodes.to_le_bytes());
        eat(&self.globals.to_le_bytes());
        eat(&self.size_sum.to_bits().to_le_bytes());
        h
    }

    fn size(&self) -> (u32, u32) {
        let (side, cell) = grid(self.acting.len() as u32);
        ((GRID_X + side * cell) as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ThresholdsMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        // Time: each crowd's share acting; episode starts marked by a tick at the top.
        let crowds = self.crowd_size.len();
        for (k, column) in self.recent.iter().enumerate() {
            if column.start && k > 0 {
                c.column(k, 0, 6, MARK);
            }
        }
        for s in (0..crowds).rev() {
            let color = if crowds == 1 {
                LINE
            } else {
                CROWDS[s % CROWDS.len()]
            };
            for (k, column) in self.recent.iter().enumerate() {
                let y = row(column.shares[s]);
                let to = self
                    .recent
                    .get(k + 1)
                    .filter(|next| !next.start)
                    .map_or(y, |next| row(next.shares[s]));
                c.column(k, y, to, color);
            }
        }
        if self.figure_one() {
            // Granovetter's Figure 1: F against the 45° line, and the staircase.
            for x in 0..MID_W {
                let share = x as f64 / (MID_W - 1) as f64;
                c.put(MID_X + x, row(share), MARK);
                c.put(MID_X + x, row(self.cdf(share)), BAR);
            }
            let n = self.acting.len() as f64;
            for w in self.path.windows(2) {
                let (a, b) = (f64::from(w[0]) / n, f64::from(w[1]) / n);
                c.column(MID_X + col(a), row(a), row(b), PATH);
            }
        } else {
            let mut per_row = vec![0u32; TALL];
            for (p, &k) in self.sizes.iter().enumerate() {
                per_row[row(p as f64 / 100.0)] += k;
            }
            let top = per_row.iter().copied().max().unwrap_or(0).max(1);
            for (y, &k) in per_row.iter().enumerate() {
                let len = (f64::from(k) / f64::from(top) * (MID_W - 1) as f64).round() as usize;
                for x in 0..len {
                    c.put(MID_X + x, y, BAR);
                }
            }
            let g = row(self.config.global);
            for x in 0..MID_W {
                c.put(MID_X + x, g, MARK);
            }
        }
        let n = self.acting.len();
        let (side, cell) = grid(n as u32);
        let most = if self.sees.is_empty() {
            0
        } else {
            self.sees.iter().map(Vec::len).max().unwrap_or(0)
        };
        for i in 0..n {
            let color = self.color(mode, i, most);
            let (gx, gy) = (i % side, i / side);
            for dy in 0..cell {
                for dx in 0..cell {
                    c.put(GRID_X + gx * cell + dx, gy * cell + dy, color);
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
        let mut out = String::from("id,threshold,ceiling,degree,sees,of,acting,seed,crowd\n");
        for i in 0..self.acting.len() {
            let v = self.view(i);
            let opt = |x: Option<f64>| x.map_or(String::new(), |x| x.to_string());
            writeln!(
                out,
                "{},{},{},{},{},{},{},{},{}",
                v.id,
                opt(v.threshold),
                opt(v.ceiling),
                v.degree.map_or(String::new(), |d| d.to_string()),
                v.sees,
                v.of,
                u8::from(v.acting),
                u8::from(v.seed),
                v.crowd
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
        let ModelConfig::Thresholds(next) = next else {
            return Err(wrong_model(ModelKind::Thresholds, &next));
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

    /// Stopped at `stop_at`: a sweep reads the run at its last step.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::thresholds::config::{Ceilings, Clusters, Crowd, Friends, Rounding};

    fn config(edit: impl FnOnce(&mut ThresholdsConfig)) -> ThresholdsConfig {
        let mut c = ThresholdsConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut ThresholdsConfig)) -> ThresholdsWorld {
        ThresholdsWorld::new(config(edit), 1).unwrap()
    }

    /// Runs to the end of the first episode.
    fn settle(w: &mut ThresholdsWorld) -> u32 {
        for _ in 0..100_000 {
            if w.settled {
                break;
            }
            w.step();
        }
        w.total
    }

    /// The incrementally kept counts match a recount.
    fn consistent(w: &ThresholdsWorld) {
        let total = w.acting.iter().filter(|&&a| a).count() as u32;
        assert_eq!(w.total, total);
        for k in 0..w.crowd_size.len() {
            let members = (0..w.acting.len()).filter(|&i| w.crowd_of[i] as usize == k);
            assert_eq!(w.crowd_size[k] as usize, members.clone().count());
            assert_eq!(
                w.crowd_acting[k] as usize,
                members.filter(|&i| w.acting[i]).count()
            );
        }
        if !w.sees.is_empty() {
            for i in 0..w.acting.len() {
                let seen = w.sees[i].iter().filter(|&&j| w.acting[j as usize]).count();
                assert_eq!(w.seen_acting[i] as usize, seen, "actor {i}");
            }
        }
    }

    #[test]
    fn the_uniform_crowd_riots_to_the_last_person_and_the_perturbed_stops_at_one() {
        for n in [100, 37, 1000] {
            let mut w = world(|c| c.actors = n);
            assert_eq!(settle(&mut w), n, "N {n}: exact comparisons all the way up");
        }
        let mut p = world(|c| c.distribution = Distribution::Perturbed);
        assert_eq!(settle(&mut p), 1);
        // One person per step: the bandwagon.
        let mut u = world(|_| {});
        u.run(5);
        assert_eq!(u.total, 5);
    }

    #[test]
    fn granovetter_s_friend_example_perceives_63_of_120() {
        // 48 of 100 act; our actor knows 20, 15 of them acting, friends count 2.
        let mut w = world(|c| {
            c.friends = Friends {
                enabled: true,
                acquaintance: 0.0,
                ..Friends::default()
            }
        });
        let me = 99;
        let friends: Vec<u32> = (0..20).collect();
        let mut sees = vec![Vec::new(); 100];
        sees[me] = friends.clone();
        for &f in &friends {
            sees[f as usize].push(me as u32);
        }
        w.sees = Arc::new(sees);
        for i in 0..100 {
            w.acting[i] = false;
        }
        w.total = 0;
        w.crowd_acting = vec![0];
        w.seen_acting = vec![0; 100];
        for i in (0..15).chain(20..53) {
            w.set(i, true);
        }
        assert_eq!(w.total, 48);
        assert_eq!(w.perceived(me), (63, 120));
        w.th[me] = Th::people(50, 100);
        assert!(w.decide(me), "0.525 is above his 50 %");
    }

    #[test]
    fn thresholds_count_oneself_or_not() {
        // With 49 of 100 others... an actor at 50 % among 99 others needs 50
        // of 100 counting himself, 50 of 99 not.
        let mut w = world(|_| {});
        for i in 0..100 {
            w.acting[i] = false;
        }
        w.total = 0;
        w.crowd_acting = vec![0];
        for i in 0..49 {
            w.set(i, true);
        }
        w.th[99] = Th::people(49, 100);
        assert!(w.decide(99), "49 of 100");
        w.th[99] = Th::from_fraction(0.4945);
        assert!(!w.decide(99), "49 of 100 is below 49.45 %");
        w.config.counts_self = false;
        assert!(w.decide(99), "49 of 99 is above 49.45 %");
    }

    #[test]
    fn normal_crowds_tip_where_the_rounding_puts_them() {
        let tip = |sd: f64, rounding| {
            let mut w = world(|c| {
                c.distribution = Distribution::Normal;
                c.sd = sd;
                c.rounding = rounding;
            });
            settle(&mut w)
        };
        assert!(tip(0.122, Rounding::Nearest) < 10);
        assert!(tip(0.123, Rounding::Nearest) > 90);
        assert!(
            tip(0.12, Rounding::Floor) > 90,
            "rounding down tips earlier"
        );
        assert!(
            tip(0.124, Rounding::Exact) < 10,
            "exact fractions tip later"
        );
        let w = world(|c| {
            c.distribution = Distribution::Normal;
            c.sd = 0.122;
        });
        let s = w.stats.latest().unwrap();
        assert!((s.theory * 100.0 - 5.5).abs() < 0.5, "{}", s.theory);
    }

    #[test]
    fn city_crowds_differ_each_episode() {
        let mut w = world(|c| {
            c.population = Population::City;
            c.repeat = true;
        });
        w.run(3000);
        assert!(w.episodes > 100);
        let small = w.sizes[0] + w.sizes[1];
        let share = f64::from(small) / f64::from(w.episodes);
        assert!((0.35..0.65).contains(&share), "{share}");
        consistent(&w);
    }

    #[test]
    fn friends_symmetric_or_one_way_keep_counts() {
        for symmetric in [true, false] {
            let mut w = world(|c| {
                c.friends = Friends {
                    enabled: true,
                    symmetric,
                    ..Friends::default()
                };
                c.repeat = true;
            });
            w.run(300);
            consistent(&w);
            assert_eq!(w.watched_by.is_some(), !symmetric);
        }
    }

    #[test]
    fn watts_s_networks_and_triggers() {
        let mut w = world(|c| {
            c.actors = 2000;
            c.distribution = Distribution::Fixed;
            c.mean = 0.18;
            c.network = Network::Random;
            c.degree = 3.0;
            c.trigger = Trigger::Random;
            c.update = Update::Asynchronous;
            c.repeat = true;
        });
        let links = w.sees.iter().map(Vec::len).sum::<usize>() as f64 / 2000.0;
        assert!((links - 3.0).abs() < 0.2, "{links}");
        assert_eq!(w.seed.iter().filter(|&&s| s).count(), 1);
        w.run(400);
        consistent(&w);
        assert!(w.episodes > 5);
        let hub = world(|c| {
            c.actors = 500;
            c.network = Network::Random;
            c.trigger = Trigger::Hub;
        });
        let s = (0..500).find(|&i| hub.seed[i]).unwrap();
        let most = hub.sees.iter().map(Vec::len).max().unwrap();
        assert_eq!(hub.sees[s].len(), most);
        let p = world(|c| {
            c.actors = 3000;
            c.network = Network::PowerLaw;
            c.degree = 1.5;
        });
        let mean = p.sees.iter().map(Vec::len).sum::<usize>() as f64 / 3000.0;
        assert!((mean - 1.5).abs() < 0.2, "{mean}");
    }

    #[test]
    fn zero_thresholds_act_at_once_or_when_reached() {
        let isolated = |zero| {
            let mut w = world(|c| {
                c.actors = 50;
                c.distribution = Distribution::Fixed;
                c.mean = 0.0;
                c.network = Network::Random;
                c.degree = 0.0;
                c.zero = zero;
            });
            settle(&mut w)
        };
        assert_eq!(isolated(Zero::Acts), 50, "0 ≥ 0 read literally");
        assert_eq!(isolated(Zero::WhenReached), 0, "nobody to reach them");
    }

    #[test]
    fn synchronous_and_asynchronous_reach_the_same_equilibrium() {
        for d in [Distribution::Uniform, Distribution::Perturbed] {
            let mut a = world(|c| c.distribution = d);
            let mut b = world(|c| {
                c.distribution = d;
                c.update = Update::Asynchronous;
            });
            assert_eq!(settle(&mut a), settle(&mut b));
        }
    }

    #[test]
    fn ceilings_make_most_riots_pulse_under_synchronous_updating() {
        // Whether a crowd pulses depends on who holds the ceilings: with 10 %
        // leaving above 90 %, most of 20 crowds never settle; those that do
        // rest at 91.
        let mut pulsing = 0;
        for seed in 1..=20 {
            let mut w = ThresholdsWorld::new(
                config(|c| {
                    c.ceilings = Ceilings {
                        share: 0.1,
                        at: 0.9,
                    }
                }),
                seed,
            )
            .unwrap();
            w.run(600);
            if w.settled {
                assert_eq!(w.total, 91, "seed {seed}");
            } else {
                pulsing += 1;
                // It swings by about the share holding ceilings (83 to 92).
                assert!(w.stats.latest().unwrap().swing > 0.05, "seed {seed}");
            }
        }
        assert!(pulsing >= 12, "{pulsing} of 20");
        // Asynchronous updating hovers near the ceiling instead.
        let mut a = world(|c| {
            c.ceilings = Ceilings {
                share: 0.1,
                at: 0.9,
            };
            c.update = Update::Asynchronous;
        });
        a.run(300);
        assert!(a.stats.latest().unwrap().recent_mean > 0.85);
    }

    #[test]
    fn clusters_move_and_rioters_can_stop() {
        let mut w = world(|c| {
            c.population = Population::City;
            c.clusters = Clusters {
                enabled: true,
                count: 10,
                movement: 0.05,
            };
        });
        let before = w.crowd_of.clone();
        w.run(50);
        consistent(&w);
        assert_ne!(before, w.crowd_of);
        assert!(!w.settled, "clusters are one long episode");
        assert_eq!(w.crowd_size.iter().sum::<u32>(), 1000);
    }

    #[test]
    fn episodes_record_sizes_and_global_shares() {
        let mut w = world(|c| {
            c.population = Population::City;
            c.repeat = true;
            c.global = 0.5;
        });
        w.run(2000);
        let s = w.stats.latest().unwrap().clone();
        assert_eq!(s.episodes, w.episodes);
        assert_eq!(w.sizes.iter().sum::<u32>(), w.episodes);
        let big: u32 = w.sizes[50..].iter().sum();
        assert!((s.global_share - f64::from(big) / f64::from(w.episodes)).abs() < 0.02);
        assert!(s.mean_size > 0.0 && s.mean_size < 0.5);
        assert!(s.theory.is_nan());
        assert!(Model::latest_json(&w).contains("\"theory\":null"));
        let mut once = world(|_| {});
        once.run(300);
        assert_eq!(
            (once.episodes, once.total),
            (1, 100),
            "without repeat the world rests"
        );
    }

    #[test]
    fn the_frame_draws_time_figure_or_histogram_and_actors() {
        let mut w = world(|_| {});
        w.run(30);
        let mut buf = Vec::new();
        w.render("state", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw as usize, fh as usize), (GRID_X + 200, TALL));
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * fw as usize + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        assert_eq!(px(&buf, 30, row(0.30)), LINE);
        assert_eq!(px(&buf, GRID_X, 0), ACTING);
        for mode in ["threshold", "degree", "crowd"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let mut h = world(|c| {
            c.population = Population::City;
            c.repeat = true;
        });
        h.run(100);
        h.render("state", "", &mut buf).unwrap();
    }

    #[test]
    fn inspect_reads_steps_the_figure_bins_and_actors() {
        let mut w = world(|_| {});
        w.run(5);
        let v = w.inspect(4, 0).unwrap();
        assert_eq!((v.panel, v.step), (Some("time"), Some(4)));
        let f = w.inspect((MID_X + 50) as u32, 0).unwrap();
        assert_eq!(f.panel, Some("figure"));
        assert!(
            (f.cdf.unwrap() - 0.51).abs() < 1e-9,
            "51 of 100 at or below 50 %"
        );
        let a = w.inspect(GRID_X as u32, 0).unwrap().member.unwrap();
        assert_eq!((a.id, a.threshold, a.acting), (1, Some(0.0), true));
        let mut h = world(|c| {
            c.population = Population::City;
            c.repeat = true;
        });
        h.run(200);
        let b = h.inspect(MID_X as u32, 200).unwrap();
        assert_eq!(b.panel, Some("histogram"));
        assert!(
            b.count.unwrap() > 0,
            "episodes that ended with no one acting"
        );
        assert_eq!(Model::locate(&h, 1), None);
    }

    #[test]
    fn keyframes_restore_the_world_and_its_view() {
        let c = config(|c| {
            c.population = Population::City;
            c.friends.enabled = true;
            c.repeat = true;
        });
        let mut any = crate::model::ModelWorld::new(ModelConfig::Thresholds(c.clone()), 6).unwrap();
        any.model_mut().run(20);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("state", "", &mut before).unwrap();
        any.model_mut().run(30);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("state", "", &mut after).unwrap();
        assert_eq!(before, after);
        any.model_mut().run(30);
        let mut fresh = crate::model::ModelWorld::new(ModelConfig::Thresholds(c), 6).unwrap();
        fresh.model_mut().run(50);
        assert_eq!(
            any.model().fingerprint(),
            fresh.model().fingerprint(),
            "replays the same"
        );
    }

    #[test]
    fn live_edits_apply_and_the_crowd_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.update = Update::Asynchronous;
            c.repeat = true;
            c.stop_at = 10;
        });
        Model::set_config(&mut w, ModelConfig::Thresholds(next)).unwrap();
        w.run(100);
        assert_eq!(w.tick, 10);
        for (field, edit) in [
            ("actors", config(|c| c.actors = 101)),
            (
                "distribution",
                config(|c| c.distribution = Distribution::Perturbed),
            ),
            ("network", config(|c| c.network = Network::Random)),
            ("crowd", config(|c| c.crowd = Crowd::Sampled)),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Thresholds(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.actors = 2),
            config(|c| {
                c.distribution = Distribution::Normal;
                c.sd = 0.0;
            }),
            config(|c| {
                c.distribution = Distribution::Normal;
                c.sd = 1.0;
                c.crowd = Crowd::Sampled;
                c.repeat = true;
            }),
            config(|c| {
                c.friends = Friends {
                    enabled: true,
                    acquaintance: 1.0,
                    weight: 20,
                    symmetric: false,
                };
            }),
            config(|c| c.counts_self = false),
            config(|c| {
                c.network = Network::Random;
                c.degree = 0.0;
                c.trigger = Trigger::Random;
            }),
            config(|c| {
                c.network = Network::PowerLaw;
                c.degree = 1.9;
                c.trigger = Trigger::Hub;
                c.repeat = true;
            }),
            config(|c| {
                c.ceilings = Ceilings {
                    share: 1.0,
                    at: 0.0,
                };
                c.update = Update::Asynchronous;
            }),
            config(|c| {
                c.population = Population::City;
                c.clusters = Clusters {
                    enabled: true,
                    count: 2,
                    movement: 1.0,
                };
            }),
            config(|c| c.max_steps = 1),
        ] {
            let mut w = ThresholdsWorld::new(c.clone(), 1).unwrap();
            w.run(60);
            consistent(&w);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.acting), "{c:?}");
            assert!(s.swing.is_finite() && s.recent_mean.is_finite(), "{c:?}");
            let mut buf = Vec::new();
            w.render("degree", "", &mut buf).unwrap();
        }
    }
}
