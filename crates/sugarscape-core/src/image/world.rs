//! Image scoring's world: g groups of n agents with fixed strategies. A tick
//! is a generation: the agents' scores and payoffs start afresh, each group
//! plays its rounds of one random donor and one random recipient, and the
//! next generation's parents are drawn in proportion to payoff, from the
//! offspring's own group or (LH01's island model) the whole population. The
//! world keeps the generation that last played, its scores, payoffs and
//! records, for the frame and Inspect; its offspring are drawn when the next
//! tick begins.

use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{ImageConfig, Initial, Offset, Records, RoundsKind};
use super::stats::{ratio, ImageSnapshot, SERIES};
use super::strategy::{Class, Situation, Strategy, Tallies};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{
    lerp, Rgb, BACKGROUND, BLUE, COOL, FEMALE, HOT, NEUTRAL, POLLUTION, RED, SUGAR,
};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

/// A mark's bits: the member believes the other in good standing; it has
/// seen the other act this generation.
const GOOD: u8 = 1;
const SEEN: u8 = 2;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageMode {
    Strategy,
    Score,
    Payoff,
}

impl std::str::FromStr for ImageMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "strategy" => Self::Strategy,
            "score" => Self::Score,
            "payoff" => Self::Payoff,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// One member of a group, with this generation's results.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: u64,
    pub strategy: Strategy,
    /// The true image score (its own record) and standing.
    pub score: i32,
    pub good: bool,
    pub payoff: f64,
    /// Helps given and received this generation.
    pub given: u32,
    pub received: u32,
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ImageInspection {
    pub cell: Cell,
    /// The group the cell's tile belongs to, or none (a gap).
    pub group: Option<u32>,
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub group: u32,
    /// "k = 0", "k = 0, h = 1 (AND)", "standing" …
    pub strategy: String,
    pub class: &'static str,
    /// Whether the strategy helps at a generation's start.
    pub cooperative: bool,
    pub score: i32,
    /// Good standing (as everyone would judge it without perception errors).
    pub standing: bool,
    /// With private records: the members (besides itself) who have seen it
    /// act this generation, and the mean of their records of its score;
    /// none with perfect information (everyone knows it).
    pub known: Option<u32>,
    pub mean_view: Option<f64>,
    pub payoff: f64,
    pub given: u32,
    pub received: u32,
}

#[derive(Clone)]
pub struct ImageWorld {
    pub config: ImageConfig,
    /// Generations played.
    pub tick: u64,
    /// Group-major: group g's members are `g·n .. (g + 1)·n`.
    agents: Vec<Agent>,
    /// The allowed strategies (fixed once built; keyframes share them).
    allowed: Arc<Vec<Strategy>>,
    /// Private records, kept only with observers or perception errors:
    /// `views[g·n² + j·n + i]` is member j's record of member i's score in
    /// group g, `marks` at the same index its belief in i's standing and
    /// whether it has seen i act.
    views: Vec<i16>,
    marks: Vec<u8>,
    /// A keyframe's stand-in for the records (which the next generation
    /// rebuilds anyway): each member's count of sightings and the sum of
    /// their records, all Inspect reads of them. Empty in a playing world.
    seen: Vec<(u32, f64)>,
    /// The q strategies' tallies, one per group (empty without them).
    tallies: Vec<Tallies>,
    /// Helps and rounds in the generation that last played.
    helps: u64,
    rounds_played: u64,
    next_id: u64,
    /// The last generation's meetings as (donor, recipient, helped), places
    /// in the agent list, when recording them (the studio's shots).
    meetings: Option<Vec<(u32, u32, bool)>>,
    rng: SimRng,
    pub stats: Stats<ImageSnapshot>,
}

/// The smallest s with s² ≥ v.
fn ceil_sqrt(v: u32) -> u32 {
    let mut s = 0;
    while s * s < v {
        s += 1;
    }
    s
}

/// The index of a draw `x` in `[0, total)` from cumulative weights `cum`
/// (the first entry above `x`), or `None` when every weight is 0.
fn roulette(cum: &[f64], x: f64) -> Option<usize> {
    let total = *cum.last()?;
    if total <= 0.0 {
        return None;
    }
    let i = cum.partition_point(|&v| v <= x);
    // x rounded up to the total: the last member with any weight.
    Some(if i < cum.len() {
        i
    } else {
        cum.partition_point(|&v| v < total)
    })
}

impl ImageWorld {
    pub fn new(config: ImageConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let allowed = config.allowed();
        let mut w = ImageWorld {
            config,
            tick: 0,
            agents: Vec::new(),
            allowed: Arc::new(allowed),
            views: Vec::new(),
            marks: Vec::new(),
            seen: Vec::new(),
            tallies: Vec::new(),
            helps: 0,
            rounds_played: 0,
            next_id: 1,
            meetings: None,
            rng: rng::seeded(seed),
            stats: Stats::default(),
        };
        let n = w.config.group_size as usize;
        for _ in 0..w.config.groups {
            for slot in 0..n {
                let strategy = match &w.config.initial {
                    Initial::Uniform => w.random_strategy(),
                    Initial::Seeded(s) => {
                        let invaders = (s.share * n as f64).round() as usize;
                        match s.invader {
                            Some(x) if slot < invaders => x,
                            _ => s.only,
                        }
                    }
                };
                w.add(strategy);
            }
        }
        w.reset_results();
        w.record();
        Ok(w)
    }

    fn random_strategy(&mut self) -> Strategy {
        let k = self.rng.gen_range(0..self.allowed.len() as u32) as usize;
        self.allowed[k]
    }

    fn add(&mut self, strategy: Strategy) {
        self.agents.push(Agent {
            id: self.next_id,
            strategy,
            score: 0,
            good: true,
            payoff: self.config.u0,
            given: 0,
            received: 0,
        });
        self.next_id += 1;
    }

    fn n(&self) -> usize {
        self.config.group_size as usize
    }

    /// Every agent, group by group.
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Starts or stops recording each generation's meetings (from the next
    /// generation played). Recording draws nothing.
    pub fn record_meetings(&mut self, on: bool) {
        self.meetings = on.then(Vec::new);
    }

    /// The last generation's meetings as (donor, recipient, helped), places
    /// in the agent list, if recording.
    pub fn meetings(&self) -> Option<&[(u32, u32, bool)]> {
        self.meetings.as_deref()
    }

    /// Group `g`'s members.
    pub fn group(&self, g: usize) -> &[Agent] {
        let n = self.n();
        &self.agents[g * n..(g + 1) * n]
    }

    pub fn population(&self) -> usize {
        self.agents.len()
    }

    /// The allowed strategies, in the order a mutation or uniform start draws them.
    pub fn allowed(&self) -> &[Strategy] {
        &self.allowed
    }

    /// Whether the run has reached its last generation.
    pub fn is_finished(&self) -> bool {
        self.config.end > 0 && self.tick >= u64::from(self.config.end)
    }

    /// Helps and rounds in the generation that last played.
    pub fn helps(&self) -> (u64, u64) {
        (self.helps, self.rounds_played)
    }

    /// Whether this generation keeps private records.
    pub fn private(&self) -> bool {
        !self.views.is_empty()
    }

    /// A copy of this world for a keyframe: everything but its statistics
    /// history and its private records, which take g·n²·3 bytes (60 MB at
    /// 80 groups of 500) and are rebuilt when the next generation starts.
    /// What Inspect reads of them is kept instead (g·n entries), so a
    /// restored world inspects as the live one did.
    pub fn keyframe(&mut self) -> ImageWorld {
        let seen = if self.private() {
            (0..self.agents.len()).map(|i| self.sightings(i)).collect()
        } else {
            self.seen.clone()
        };
        let stats = std::mem::take(&mut self.stats);
        let views = std::mem::take(&mut self.views);
        let marks = std::mem::take(&mut self.marks);
        let mut copy = self.clone();
        (self.stats, self.views, self.marks) = (stats, views, marks);
        copy.seen = seen;
        copy
    }

    /// One generation: offspring of the last (after the first), then play.
    pub fn step(&mut self) {
        self.apply_schedule();
        if self.tick > 0 {
            self.reproduce();
        }
        self.play();
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
            let next = ModelConfig::Image(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Image(next)) = next {
                self.config = next;
            }
        }
    }

    /// Rule 1: scores 0, standing good, payoffs u₀, records unknown, the q
    /// tallies at their prior.
    fn reset_results(&mut self) {
        let u0 = self.config.u0;
        for a in &mut self.agents {
            (a.score, a.good, a.payoff, a.given, a.received) = (0, true, u0, 0, 0);
        }
        let private = self.config.private();
        let (g, n) = (self.config.groups as usize, self.n());
        self.views.clear();
        self.marks.clear();
        self.seen.clear();
        if private {
            self.views.resize(g * n * n, 0);
            self.marks.resize(g * n * n, GOOD);
        }
        self.tallies.clear();
        if self.config.strategies.contains(&Class::Q) {
            let (lo, hi) = self.config.score_range();
            self.tallies.resize(g, Tallies::new(lo, hi));
        }
        self.helps = 0;
        self.rounds_played = 0;
        if let Some(m) = &mut self.meetings {
            m.clear();
        }
    }

    /// Rule 2: each group plays its rounds.
    fn play(&mut self) {
        self.reset_results();
        let n = self.n() as u32;
        let m = self.config.rounds;
        for g in 0..self.config.groups as usize {
            let mut played = 0;
            loop {
                let d = self.rng.gen_range(0..n);
                let mut r = self.rng.gen_range(0..n - 1);
                if r >= d {
                    r += 1;
                }
                self.interact(g, d as usize, r as usize);
                played += 1;
                let last = match self.config.rounds_kind {
                    RoundsKind::Fixed => played >= m,
                    RoundsKind::Random => self.rng.gen::<f64>() < 1.0 / f64::from(m),
                };
                if last {
                    break;
                }
            }
        }
    }

    /// One round in group `g`: member `d` may help member `r`.
    fn interact(&mut self, g: usize, d: usize, r: usize) {
        let n = self.n();
        let (di, ri) = (g * n + d, g * n + r);
        let row = g * n * n + d * n;
        let private = self.private();
        let (seen, own_good, their_good) = if private {
            (
                i32::from(self.views[row + r]),
                self.marks[row + d] & GOOD != 0,
                self.marks[row + r] & GOOD != 0,
            )
        } else {
            (
                self.agents[ri].score,
                self.agents[di].good,
                self.agents[ri].good,
            )
        };
        let situation = Situation {
            own: self.agents[di].score,
            seen,
            own_good,
            their_good,
            tallies: self.tallies.get(g),
        };
        let mut help = self.agents[di].strategy.helps(&situation);
        let e = self.config.execution_error;
        if e > 0.0 && self.rng.gen::<f64>() < e {
            help = !help;
        }
        let (b, c) = (self.config.b, self.config.c);
        let bonus = match self.config.offset {
            Offset::Both => c,
            Offset::None => 0.0,
        };
        let (old, r_score, r_good) = (
            self.agents[di].score,
            self.agents[ri].score,
            self.agents[ri].good,
        );
        let new = self.moved(old, help);
        let donor = &mut self.agents[di];
        if help {
            donor.payoff -= c;
            donor.given += 1;
        }
        donor.payoff += bonus;
        donor.score = new;
        if help {
            donor.good = true;
        } else if r_good {
            donor.good = false;
        }
        let recipient = &mut self.agents[ri];
        if help {
            recipient.payoff += b;
            recipient.received += 1;
        }
        recipient.payoff += bonus;
        if let Some(t) = self.tallies.get_mut(g) {
            t.record(r_score, help);
        }
        self.helps += u64::from(help);
        self.rounds_played += 1;
        if let Some(m) = &mut self.meetings {
            m.push((di as u32, ri as u32, help));
        }
        if private {
            self.observe(g, d, r, old, help);
        }
    }

    /// A score one up (helped) or down, within the range.
    fn moved(&self, score: i32, up: bool) -> i32 {
        let (lo, hi) = self.config.score_range();
        (score + if up { 1 } else { -1 }).clamp(lo, hi)
    }

    /// Updates the private records after `d` helped `r` or not: the donor
    /// judges its own standing by its own record of the recipient; the
    /// recipient always, and each other member with the watch probability,
    /// sees the action (the other one with probability ε) and records it.
    fn observe(&mut self, g: usize, d: usize, r: usize, old: i32, help: bool) {
        let n = self.n();
        let base = g * n * n;
        let judge = |marks: &[u8], row: usize, help: bool| {
            let mut mark = marks[row + d] | SEEN;
            if help {
                mark |= GOOD;
            } else if marks[row + r] & GOOD != 0 {
                mark &= !GOOD;
            }
            mark
        };
        let own = base + d * n;
        self.marks[own + d] = judge(&self.marks, own, help);
        let p = self.config.watch_probability();
        let eps = self.config.perception_error;
        for j in 0..n {
            if j == d || (j != r && p < 1.0 && self.rng.gen::<f64>() >= p) {
                continue;
            }
            let mut saw = help;
            if eps > 0.0 && self.rng.gen::<f64>() < eps {
                saw = !saw;
            }
            let row = base + j * n;
            let from = match self.config.records {
                Records::Score => old,
                Records::Tally => i32::from(self.views[row + d]),
            };
            // Saturated: unbounded scores with random rounds can pass i16.
            let next = self.moved(from, saw);
            self.views[row + d] = next.clamp(i16::MIN.into(), i16::MAX.into()) as i16;
            self.marks[row + d] = judge(&self.marks, row, saw);
        }
    }

    /// Rule 3: each new member's parent by payoff-proportional roulette from
    /// its own group (probability p) or the whole population, its strategy
    /// redrawn with probability ν. Negative payoffs (only possible without
    /// the offset) weigh 0; a pool with no weight at all is drawn uniformly.
    fn reproduce(&mut self) {
        let (g, n) = (self.config.groups as usize, self.n());
        let mut local = Vec::with_capacity(self.agents.len());
        let mut global = Vec::with_capacity(self.agents.len());
        let (mut in_group, mut all) = (0.0, 0.0);
        for (i, a) in self.agents.iter().enumerate() {
            if i % n == 0 {
                in_group = 0.0;
            }
            let w = a.payoff.max(0.0);
            in_group += w;
            all += w;
            local.push(in_group);
            global.push(all);
        }
        let p = self.config.local;
        let nu = self.config.mutation;
        let mut next = Vec::with_capacity(self.agents.len());
        for group in 0..g {
            for _ in 0..n {
                let home = g == 1 || p >= 1.0 || (p > 0.0 && self.rng.gen::<f64>() < p);
                let (pool, offset) = if home {
                    (&local[group * n..(group + 1) * n], group * n)
                } else {
                    (&global[..], 0)
                };
                let x = self.rng.gen::<f64>() * pool.last().copied().unwrap_or(0.0);
                let parent = match roulette(pool, x) {
                    Some(i) => i,
                    None => self.rng.gen_range(0..pool.len() as u32) as usize,
                } + offset;
                let mut strategy = self.agents[parent].strategy;
                if nu > 0.0 && self.rng.gen::<f64>() < nu {
                    strategy = self.random_strategy();
                }
                next.push(strategy);
            }
        }
        self.agents.clear();
        for s in next {
            self.add(s);
        }
    }

    fn record(&mut self) {
        let (mut coop, mut k_sum, mut k_count) = (0u64, 0.0, 0u64);
        let (mut payoff, mut score) = (0.0, 0.0);
        let mut class = [0u64; 8];
        let (mut k_coop, mut k_def) = (0u64, 0u64);
        let mut binary = [0u64; 3];
        for a in &self.agents {
            let s = a.strategy;
            coop += u64::from(s.cooperative());
            if let Some(k) = s.k() {
                k_sum += f64::from(k);
                k_count += 1;
            }
            payoff += a.payoff;
            score += f64::from(a.score);
            class[Class::ALL.iter().position(|&c| c == s.class()).unwrap()] += 1;
            match s {
                Strategy::K(k) if k <= 0 => k_coop += 1,
                Strategy::K(_) => k_def += 1,
                Strategy::Binary(k) => binary[(k + 1) as usize] += 1,
                _ => {}
            }
        }
        let total = self.agents.len() as u64;
        let share = |c: u64| ratio(c as f64, total);
        let s = ImageSnapshot {
            tick: self.tick,
            help_rate: ratio(self.helps as f64, self.rounds_played),
            mean_k: ratio(k_sum, k_count),
            cooperative: share(coop),
            mean_payoff: ratio(payoff, total),
            mean_score: ratio(score, total),
            k_cooperative: share(k_coop),
            k_defective: share(k_def),
            h: share(class[1]),
            and: share(class[2]),
            or: share(class[3]),
            own_only: share(class[4]),
            standing: share(class[6]),
            binary_c: share(binary[0]),
            binary_x: share(binary[1]),
            binary_d: share(binary[2]),
            q: share(class[7]),
            helps: self.helps,
        };
        self.stats.push(s);
    }

    /// The frame's tiles: each group a square of side `tile`, groups in
    /// `cols` columns with one-cell gaps.
    fn layout(&self) -> (u32, u32, u32) {
        let tile = ceil_sqrt(self.config.group_size);
        let cols = ceil_sqrt(self.config.groups);
        let rows = self.config.groups.div_ceil(cols);
        (tile, cols, rows)
    }

    /// Agent `i`'s cell.
    pub fn cell(&self, i: usize) -> (u32, u32) {
        let (tile, cols, _) = self.layout();
        let n = self.n();
        let (g, s) = ((i / n) as u32, (i % n) as u32);
        let (gx, gy) = (g % cols, g / cols);
        (gx * (tile + 1) + s % tile, gy * (tile + 1) + s / tile)
    }

    /// The group and agent (index) at a cell, if any.
    fn at(&self, x: u32, y: u32) -> (Option<u32>, Option<usize>) {
        let (tile, cols, rows) = self.layout();
        let (gx, ox, gy, oy) = (
            x / (tile + 1),
            x % (tile + 1),
            y / (tile + 1),
            y % (tile + 1),
        );
        if ox == tile || oy == tile || gx >= cols || gy >= rows {
            return (None, None);
        }
        let g = gy * cols + gx;
        if g >= self.config.groups {
            return (None, None);
        }
        let s = oy * tile + ox;
        let agent = (s < self.config.group_size).then(|| (g * self.config.group_size + s) as usize);
        (Some(g), agent)
    }

    fn color(&self, a: &Agent, mode: ImageMode, top: f64) -> Rgb {
        match mode {
            ImageMode::Strategy => match a.strategy {
                Strategy::Binary(k) => lerp(BLUE, RED, f64::from(k + 1) / 2.0),
                Strategy::H(_) => POLLUTION,
                Strategy::OwnOnly(_) => NEUTRAL,
                Strategy::Standing => SUGAR,
                Strategy::Q(_) => FEMALE,
                s => lerp(BLUE, RED, f64::from(s.k().unwrap_or(0) + 5) / 11.0),
            },
            ImageMode::Score => {
                let (lo, hi) = self.config.score_range();
                let (lo, hi) = (f64::from(lo.max(-5)), f64::from(hi.min(5)));
                let s = f64::from(a.score);
                if s >= 0.0 {
                    lerp(NEUTRAL, BLUE, if hi > 0.0 { s / hi } else { 0.0 })
                } else {
                    lerp(NEUTRAL, RED, s / lo)
                }
            }
            ImageMode::Payoff => lerp(COOL, HOT, if top > 0.0 { a.payoff / top } else { 0.0 }),
        }
    }

    /// How many of the others in member `i`'s group have seen it act, and
    /// the sum of their records of its score.
    fn sightings(&self, i: usize) -> (u32, f64) {
        let n = self.n();
        let (g, s) = (i / n, i % n);
        let (mut count, mut sum) = (0u32, 0.0);
        for j in (0..n).filter(|&j| j != s) {
            let at = g * n * n + j * n + s;
            if self.marks[at] & SEEN != 0 {
                count += 1;
                sum += f64::from(self.views[at]);
            }
        }
        (count, sum)
    }

    /// Member `i`'s record among the others in its group: how many have
    /// seen it act, and their mean record of its score; none without
    /// private records.
    fn known(&self, i: usize) -> Option<(u32, Option<f64>)> {
        let (count, sum) = if self.private() {
            self.sightings(i)
        } else {
            *self.seen.get(i)?
        };
        Some((count, (count > 0).then(|| sum / f64::from(count))))
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<ImageInspection, String> {
        let (w, h) = Model::size(self);
        if x >= w || y >= h {
            return Err(format!("({x}, {y}) is outside the frame"));
        }
        let (group, index) = self.at(x, y);
        let agent = index.map(|i| {
            let a = &self.agents[i];
            let (known, mean_view) = match self.known(i) {
                Some((k, m)) => (Some(k), m),
                None => (None, None),
            };
            AgentView {
                id: a.id,
                group: (i / self.n()) as u32,
                strategy: a.strategy.label(),
                class: a.strategy.class().as_str(),
                cooperative: a.strategy.cooperative(),
                score: a.score,
                standing: a.good,
                known,
                mean_view,
                payoff: a.payoff,
                given: a.given,
                received: a.received,
            }
        });
        Ok(ImageInspection {
            cell: Cell { x, y },
            group,
            agent,
        })
    }
}

/// A strategy's name in the agents CSV (no commas).
fn csv_name(s: Strategy) -> String {
    match s {
        Strategy::K(k) => format!("k={k}"),
        Strategy::H(h) => format!("h={h}"),
        Strategy::OwnOnly(h) => format!("own_only h={h}"),
        Strategy::And { k, h } => format!("and k={k} h={h}"),
        Strategy::Or { k, h } => format!("or k={k} h={h}"),
        Strategy::Binary(k) => format!("binary k={k}"),
        Strategy::Standing => "standing".to_string(),
        Strategy::Q(d) => format!("q {:.2}", f64::from(d) / 100.0),
    }
}

impl Model for ImageWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Image(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        ImageWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        ImageWorld::population(self)
    }

    /// FNV-1a over the tick, the id counter and every agent in order.
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
            eat(a.strategy.code());
            eat(i64::from(a.score) as u64 ^ u64::from(a.good) << 40);
            eat(a.payoff.to_bits());
            eat(u64::from(a.given) | u64::from(a.received) << 32);
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (tile, cols, rows) = self.layout();
        (cols * (tile + 1) - 1, rows * (tile + 1) - 1)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ImageMode = mode.parse()?;
        let (w, h) = Model::size(self);
        buf.clear();
        buf.resize((w * h * 4) as usize, 0);
        let px = buf.as_chunks_mut::<4>().0;
        for p in px.iter_mut() {
            p.copy_from_slice(&[BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]);
        }
        let top = self.agents.iter().map(|a| a.payoff).fold(0.0, f64::max);
        for (i, a) in self.agents.iter().enumerate() {
            let (x, y) = self.cell(i);
            let rgb = self.color(a, mode, top);
            px[(y * w + x) as usize].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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
        let mut out = String::from("id,group,x,y,strategy,score,standing,payoff,given,received\n");
        for (i, a) in self.agents.iter().enumerate() {
            let (x, y) = self.cell(i);
            writeln!(
                out,
                "{},{},{x},{y},{},{},{},{},{},{}",
                a.id,
                i / self.n(),
                csv_name(a.strategy),
                a.score,
                a.good,
                a.payoff,
                a.given,
                a.received
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
        let i = self.agents.iter().position(|a| a.id == id)?;
        Some(self.cell(i))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Image(next) = next else {
            return Err(wrong_model(ModelKind::Image, &next));
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
    use crate::image::config::{Information, Seeded};
    use serde_json::json;

    /// A world of `groups` × `n` all playing `only`, after `edit`, with its
    /// results reset (a generation about to play).
    fn world(
        groups: u32,
        n: u32,
        only: Strategy,
        edit: impl FnOnce(&mut ImageConfig),
    ) -> ImageWorld {
        let mut c = ImageConfig {
            groups,
            group_size: n,
            strategies: vec![only.class()],
            initial: Initial::Seeded(Seeded {
                only,
                invader: None,
                share: 0.0,
            }),
            ..Default::default()
        };
        edit(&mut c);
        let mut w = ImageWorld::new(c, 1).unwrap();
        w.reset_results();
        w
    }

    fn payoffs(w: &ImageWorld) -> Vec<f64> {
        w.agents.iter().map(|a| a.payoff).collect()
    }

    #[test]
    fn a_round_pays_b_and_c_with_or_without_the_offset() {
        // Helping: donor −c (+c offset), recipient +b (+c).
        let mut w = world(1, 3, Strategy::K(-5), |c| c.u0 = 2.0);
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [2.0, 3.1, 2.0]);
        assert_eq!((w.agents[0].given, w.agents[1].received), (1, 1));
        assert_eq!(w.helps(), (1, 1));
        let mut w = world(1, 3, Strategy::K(-5), |c| c.offset = Offset::None);
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [-0.1, 1.0, 0.0]);
        // Refusing: only the offset.
        let mut w = world(1, 3, Strategy::K(6), |_| {});
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [0.1, 0.1, 0.0]);
        assert_eq!(w.helps(), (0, 1));
        let mut w = world(1, 3, Strategy::K(6), |c| c.offset = Offset::None);
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn a_donors_score_moves_one_step_within_the_clamp_and_the_recipients_does_not() {
        let mut w = world(1, 2, Strategy::K(-5), |_| {});
        for i in 1..=8 {
            w.interact(0, 0, 1);
            assert_eq!(w.agents[0].score, i.min(5));
        }
        assert_eq!(w.agents[1].score, 0);
        let mut w = world(1, 2, Strategy::K(6), |c| c.clamp = 2);
        for _ in 0..5 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, -2);
        let mut w = world(1, 2, Strategy::K(-5), |c| c.clamp = 0);
        for _ in 0..30 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, 30, "unbounded");
        // Binary scorers live in {−1, 0}.
        let mut w = world(1, 2, Strategy::Binary(1), |_| {});
        for _ in 0..3 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, -1);
        w.agents[0].strategy = Strategy::Binary(-1);
        for _ in 0..3 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, 0);
    }

    #[test]
    fn each_class_decides_from_the_scores_it_reads() {
        // A k = 0 donor refuses a recipient at −1, helps one at 0.
        let mut w = world(1, 3, Strategy::K(0), |_| {});
        w.agents[1].score = -1;
        w.interact(0, 0, 1);
        w.interact(0, 0, 2);
        assert_eq!((w.agents[1].received, w.agents[2].received), (0, 1));
        // AND (k 0, h 1): not once its own score is 1.
        let mut w = world(1, 2, Strategy::And { k: 0, h: 1 }, |_| {});
        w.interact(0, 0, 1);
        w.interact(0, 0, 1);
        assert_eq!((w.agents[0].score, w.agents[0].given), (0, 1));
        // OR (k 1, h 1): helps a recipient at 0 only while its own score is below 1.
        let mut w = world(1, 2, Strategy::Or { k: 1, h: 1 }, |_| {});
        for _ in 0..3 {
            w.interact(0, 0, 1);
        }
        assert_eq!((w.agents[0].score, w.agents[0].given), (1, 2));
        // h = 1 keeps its score at 0 or 1 whatever the recipient's.
        let mut w = world(1, 2, Strategy::H(1), |_| {});
        w.agents[1].score = -5;
        for _ in 0..4 {
            w.interact(0, 0, 1);
        }
        assert_eq!((w.agents[0].score, w.agents[0].given), (0, 2));
    }

    #[test]
    fn standing_is_lost_by_refusing_the_good_and_regained_by_helping_anyone() {
        let mut w = world(1, 3, Strategy::Standing, |_| {});
        // A defector refusing a good recipient loses standing …
        w.agents[0].strategy = Strategy::Binary(1);
        w.interact(0, 0, 1);
        assert!(!w.agents[0].good);
        // … and a standing donor refuses it without losing its own.
        w.interact(0, 2, 0);
        assert_eq!(w.agents[0].received, 0);
        assert!(w.agents[2].good && w.agents[2].score == -1);
        // A standing agent in bad standing helps even a bad recipient, and is good again.
        w.agents[2].good = false;
        w.interact(0, 2, 0);
        assert_eq!(w.agents[0].received, 1);
        assert!(w.agents[2].good);
    }

    #[test]
    fn q_strategies_read_and_feed_their_groups_tallies() {
        let mut w = world(2, 3, Strategy::Q(50), |_| {});
        assert_eq!(w.tallies.len(), 2);
        // At score 0 the prior's gain is 1: help; at 1 it is 0: refuse.
        w.interact(0, 0, 1);
        w.interact(0, 0, 2);
        assert_eq!((w.agents[0].given, w.agents[0].score), (1, 0));
        // Group 0 tallied a help at 0 and a refusal at 0; group 1 nothing.
        let t = &w.tallies[0];
        assert_eq!((t.x[5], t.y[5]), (2, 1));
        assert_eq!(w.tallies[1], Tallies::new(-5, 5));
        w.reset_results();
        assert_eq!(w.tallies[0], Tallies::new(-5, 5));
    }

    #[test]
    fn execution_errors_flip_the_action_at_their_rate() {
        let mut w = world(1, 2, Strategy::K(-5), |c| c.execution_error = 0.25);
        for _ in 0..20_000 {
            w.interact(0, 0, 1);
        }
        let rate = w.helps as f64 / w.rounds_played as f64;
        assert!((rate - 0.75).abs() < 0.01, "{rate}");
    }

    /// The records of member `i` held by the others who have seen it act.
    fn records_of(w: &ImageWorld, i: usize) -> Vec<i16> {
        let n = w.n();
        (0..n)
            .filter(|&j| j != i && w.marks[j * n + i] & SEEN != 0)
            .map(|j| w.views[j * n + i])
            .collect()
    }

    #[test]
    fn perception_errors_flip_what_each_observer_records() {
        let mut w = world(1, 50, Strategy::K(-5), |c| c.perception_error = 0.2);
        assert!(w.private());
        let (mut up, mut down) = (0, 0);
        for _ in 0..400 {
            w.reset_results();
            w.interact(0, 0, 1);
            let r = records_of(&w, 0);
            assert_eq!(r.len(), 49, "perfect information: all see");
            up += r.iter().filter(|&&v| v == 1).count();
            down += r.iter().filter(|&&v| v == -1).count();
            assert_eq!(w.agents[0].score, 1, "the true score is untouched");
        }
        let wrong = down as f64 / (up + down) as f64;
        assert!((wrong - 0.2).abs() < 0.01, "{wrong}");
    }

    #[test]
    fn the_recipient_always_sees_and_others_see_observers_over_n_minus_2() {
        let mut w = world(1, 50, Strategy::K(-5), |c| {
            c.information = Information::Observers;
            c.observers = 10.0;
        });
        let mut seen = 0;
        for _ in 0..2000 {
            w.reset_results();
            w.interact(0, 0, 1);
            assert!(w.marks[w.n()] & SEEN != 0, "the recipient saw");
            seen += records_of(&w, 0).len() - 1;
        }
        let mean = seen as f64 / 2000.0;
        assert!((mean - 10.0).abs() < 0.2, "{mean}");
        // No observers: only the recipient.
        let mut w = world(1, 50, Strategy::K(-5), |c| {
            c.information = Information::Observers;
            c.observers = 0.0;
        });
        w.interact(0, 0, 1);
        assert_eq!(records_of(&w, 0).len(), 1);
    }

    #[test]
    fn records_saturate_at_the_ends_of_their_range() {
        let mut w = world(1, 2, Strategy::K(-5), |c| {
            c.information = Information::Observers;
            c.clamp = 0;
        });
        // 1's record of 0 at the top: 0 helps, and the record stays there.
        w.views[2] = i16::MAX;
        w.interact(0, 0, 1);
        assert_eq!(w.agents[1].received, 1);
        assert_eq!(w.views[2], i16::MAX);
        // 0's record of 1 at the bottom: 1 (reading 0's score as 0) refuses,
        // and the record stays there.
        w.agents[1].strategy = Strategy::K(6);
        w.views[2] = 0;
        w.views[1] = i16::MIN;
        w.interact(0, 1, 0);
        assert_eq!(w.agents[0].received, 0);
        assert_eq!(w.views[1], i16::MIN);
    }

    #[test]
    fn unknown_scores_read_as_zero_and_records_follow_the_switch() {
        let edit = |records| {
            move |c: &mut ImageConfig| {
                c.information = Information::Observers;
                c.observers = 0.0;
                c.records = records;
            }
        };
        let mut w = world(1, 4, Strategy::K(-5), edit(Records::Score));
        // 0 helps 1, 2 and 3 in turn: each recipient sees one act.
        for r in 1..=3 {
            w.interact(0, 0, r);
        }
        assert_eq!(w.agents[0].score, 3);
        // Score: each records the new score; tally: each its own record + 1.
        assert_eq!(records_of(&w, 0), [1, 2, 3]);
        let mut w = world(1, 4, Strategy::K(-5), edit(Records::Tally));
        for r in 1..=3 {
            w.interact(0, 0, r);
        }
        assert_eq!(records_of(&w, 0), [1, 1, 1]);
        // A k = 1 donor that never saw 3 act reads its score as 0: refuses.
        w.agents[2].strategy = Strategy::K(1);
        w.agents[3].strategy = Strategy::K(1);
        w.interact(0, 2, 3);
        assert_eq!(w.agents[3].received, 1, "only 0's help");
        // One that saw 0 act (record 1) helps it.
        w.interact(0, 3, 0);
        assert_eq!(w.agents[0].received, 1);
    }

    #[test]
    fn a_donor_judges_its_own_standing_by_its_own_record_of_the_recipient() {
        // Only the recipient sees; 1 believes 2 bad, the others believe it good.
        let mut w = world(1, 4, Strategy::Standing, |c| {
            c.information = Information::Observers;
            c.observers = 0.0;
        });
        let n = w.n();
        w.marks[n + 2] &= !GOOD;
        w.interact(0, 1, 2);
        // 1 refuses and still believes itself good; 2 saw it refuse someone
        // 2 believes good (itself) and marks it bad; so does the truth.
        assert_eq!(w.agents[2].received, 0);
        assert!(w.marks[n + 1] & GOOD != 0);
        assert!(w.marks[2 * n + 1] & GOOD == 0);
        assert!(!w.agents[1].good);
    }

    #[test]
    fn random_rounds_end_with_chance_one_over_m() {
        let mut w = world(4, 10, Strategy::K(0), |c| {
            c.rounds = 10;
            c.rounds_kind = RoundsKind::Random;
        });
        let mut total = 0;
        for _ in 0..500 {
            w.play();
            total += w.rounds_played;
        }
        let mean = total as f64 / 2000.0;
        assert!((mean - 10.0).abs() < 0.5, "{mean}");
        w.config.rounds = 1;
        w.play();
        assert_eq!(w.rounds_played, 4, "one round per group");
        w.config.rounds_kind = RoundsKind::Fixed;
        w.config.rounds = 7;
        w.play();
        assert_eq!(w.rounds_played, 28);
    }

    #[test]
    fn parents_are_drawn_by_payoff_from_the_group_or_the_whole_population() {
        let setup = |local: f64| {
            let mut w = world(2, 4, Strategy::K(0), |c| c.local = local);
            for (i, a) in w.agents.iter_mut().enumerate() {
                a.strategy = Strategy::K(i as i8 - 4);
                a.payoff = 0.0;
            }
            w.agents[0].payoff = 1.0;
            w.agents[3].payoff = 3.0;
            w
        };
        // Local: group 0 copies agents 0 and 3, one to three; group 1, with
        // no payoff, copies its own members uniformly.
        let mut counts = [0u32; 8];
        let mut w = setup(1.0);
        for _ in 0..4000 {
            let mut v = w.clone();
            v.reproduce();
            for a in &v.agents[..4] {
                counts[(a.strategy.k().unwrap() + 4) as usize] += 1;
            }
            assert!(v.agents[4..].iter().all(|a| a.strategy.k().unwrap() >= 0));
            w.rng = v.rng;
        }
        assert_eq!((counts[1], counts[2]), (0, 0));
        let r = f64::from(counts[3]) / f64::from(counts[0]);
        assert!((r - 3.0).abs() < 0.15, "{counts:?}");
        // Global: every offspring, in either group, is a child of 0 or 3.
        let mut w = setup(0.0);
        w.reproduce();
        assert!(w
            .agents
            .iter()
            .all(|a| matches!(a.strategy, Strategy::K(-4) | Strategy::K(-1))));
        assert_eq!(w.agents[0].id, 9, "offspring are new agents");
    }

    #[test]
    fn negative_payoffs_weigh_nothing_and_an_empty_pool_is_uniform() {
        let cum = [0.0, 0.0, 2.0, 2.0, 3.0];
        assert_eq!(roulette(&cum, 0.0), Some(2));
        assert_eq!(roulette(&cum, 1.999), Some(2));
        assert_eq!(roulette(&cum, 2.0), Some(4));
        assert_eq!(
            roulette(&cum, 3.0),
            Some(4),
            "a draw rounded up to the total"
        );
        assert_eq!(roulette(&[0.0, 0.0], 0.0), None);
        let mut w = world(1, 4, Strategy::K(0), |c| c.offset = Offset::None);
        for (i, a) in w.agents.iter_mut().enumerate() {
            a.strategy = Strategy::K(i as i8);
            a.payoff = -0.1;
        }
        w.agents[2].payoff = 0.0;
        // Every payoff is negative or 0: the pool is empty, and a uniform
        // draw should give each of the four distinct strategies roughly the
        // same share, not always the same one.
        let mut counts = [0u32; 4];
        for _ in 0..1000 {
            let mut v = w.clone();
            v.reproduce();
            for a in &v.agents {
                counts[a.strategy.k().unwrap() as usize] += 1;
            }
            w.rng = v.rng;
        }
        assert!(counts.iter().filter(|&&c| c > 0).count() > 1, "{counts:?}");
        let total = f64::from(counts.iter().sum::<u32>());
        for c in counts {
            assert!((f64::from(c) / total - 0.25).abs() < 0.05, "{counts:?}");
        }
    }

    #[test]
    fn mutation_redraws_uniformly_over_the_allowed_set() {
        let mut w = world(1, 100, Strategy::K(0), |c| {
            c.strategies = vec![Class::K, Class::Standing];
            c.mutation = 1.0;
        });
        let mut counts = std::collections::HashMap::new();
        for _ in 0..130 {
            w.reproduce();
            for a in &w.agents {
                *counts.entry(a.strategy).or_insert(0u32) += 1;
            }
        }
        assert_eq!(counts.len(), 13);
        for (s, c) in counts {
            assert!(
                (f64::from(c) / 13_000.0 - 1.0 / 13.0).abs() < 0.01,
                "{s:?}: {c}"
            );
        }
    }

    #[test]
    fn a_seeded_start_puts_the_invaders_first_in_each_group() {
        let w = world(3, 10, Strategy::K(0), |c| {
            c.strategies = vec![Class::K, Class::H];
            c.initial = Initial::Seeded(Seeded {
                only: Strategy::K(0),
                invader: Some(Strategy::H(1)),
                share: 0.2,
            });
        });
        for g in 0..3 {
            let s: Vec<Strategy> = w.group(g).iter().map(|a| a.strategy).collect();
            assert_eq!(s[..2], [Strategy::H(1); 2]);
            assert!(s[2..].iter().all(|&x| x == Strategy::K(0)));
        }
        let s = w.stats.latest().unwrap();
        assert!((s.h - 0.2).abs() < 1e-12 && (s.k_cooperative - 0.8).abs() < 1e-12);
        // A uniform start draws from every allowed strategy.
        let w = ImageWorld::new(
            ImageConfig {
                strategies: vec![Class::Or],
                group_size: 500,
                ..Default::default()
            },
            3,
        )
        .unwrap();
        let kinds: std::collections::HashSet<_> = w.agents.iter().map(|a| a.strategy).collect();
        assert!(kinds.len() > 120 && kinds.iter().all(|s| s.class() == Class::Or));
    }

    #[test]
    fn the_first_generation_plays_its_starting_strategies() {
        let c = ImageConfig {
            mutation: 0.5,
            ..Default::default()
        };
        let mut w = ImageWorld::new(c, 4).unwrap();
        let start: Vec<Strategy> = w.agents.iter().map(|a| a.strategy).collect();
        assert!(w.stats.latest().unwrap().help_rate.is_nan());
        w.step();
        let played: Vec<Strategy> = w.agents.iter().map(|a| a.strategy).collect();
        assert_eq!(start, played);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.tick, w.rounds_played), (1, 125));
        assert!((s.help_rate - s.helps as f64 / 125.0).abs() < 1e-15);
        w.step();
        assert_ne!(
            start,
            w.agents.iter().map(|a| a.strategy).collect::<Vec<_>>()
        );
    }

    #[test]
    fn statistics_count_each_class() {
        let mut w = world(1, 10, Strategy::K(0), |c| {
            c.strategies = vec![
                Class::K,
                Class::H,
                Class::And,
                Class::Or,
                Class::OwnOnly,
                Class::Standing,
                Class::Q,
            ];
        });
        let s = [
            Strategy::K(-2),
            Strategy::K(3),
            Strategy::H(1),
            Strategy::OwnOnly(0),
            Strategy::And { k: 0, h: 1 },
            Strategy::Or { k: 4, h: 0 },
            Strategy::Standing,
            Strategy::Q(10),
            Strategy::Q(90),
            Strategy::K(0),
        ];
        for (a, s) in w.agents.iter_mut().zip(s) {
            a.strategy = s;
            a.score = 1;
        }
        w.record();
        let t = w.stats.latest().unwrap().clone();
        let near = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(near(t.k_cooperative, 0.2) && near(t.k_defective, 0.1));
        assert!(near(t.h, 0.1) && near(t.own_only, 0.1));
        assert!(near(t.and, 0.1) && near(t.or, 0.1));
        assert!(near(t.standing, 0.1) && near(t.q, 0.2));
        // k ≤ 0 twice, h 1, AND (0, 1), standing and both q: cooperative.
        assert!(near(t.cooperative, 0.7), "{}", t.cooperative);
        assert!(near(t.mean_k, (-2.0 + 3.0 + 0.0 + 4.0 + 0.0) / 5.0));
        assert_eq!((t.mean_score, t.binary_c), (1.0, 0.0));
        let mut w = world(1, 3, Strategy::Binary(0), |_| {});
        w.agents[0].strategy = Strategy::Binary(-1);
        w.agents[2].strategy = Strategy::Binary(1);
        w.record();
        let t = w.stats.latest().unwrap();
        assert!(near(t.binary_c, 1.0 / 3.0) && near(t.binary_x, 1.0 / 3.0));
        assert!(near(t.binary_d, 1.0 / 3.0) && near(t.mean_k, 0.0));
    }

    #[test]
    fn frames_tile_the_groups_and_inspect_reports_the_agent() {
        let mut w = world(5, 7, Strategy::K(0), |c| {
            c.information = Information::Observers;
        });
        // Tiles of 3 × 3 in three columns and two rows, with one-cell gaps.
        assert_eq!(Model::size(&w), (11, 7));
        assert_eq!(w.cell(0), (0, 0));
        assert_eq!(w.cell(6), (0, 2));
        assert_eq!(w.cell(7), (4, 0));
        assert_eq!(w.cell(4 * 7), (4, 4));
        for i in 0..w.agents.len() {
            let (x, y) = w.cell(i);
            assert_eq!(w.at(x, y).1, Some(i));
        }
        assert_eq!(w.at(3, 0), (None, None), "a gap");
        assert_eq!(w.at(2, 2), (Some(0), None), "a tile's unused cell");
        assert_eq!(w.at(8, 4), (None, None), "no sixth group");
        w.play();
        let mut buf = Vec::new();
        for mode in ["strategy", "score", "payoff"] {
            Model::render(&w, mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 11 * 7 * 4);
        }
        assert!(Model::render(&w, "wealth", "", &mut buf).is_err());
        let v: serde_json::Value = serde_json::from_str(&w.inspect_json(4, 0).unwrap()).unwrap();
        assert_eq!(v["group"], 1);
        assert_eq!(v["agent"]["strategy"], "k = 0");
        assert_eq!(v["agent"]["class"], "k");
        assert_eq!(v["agent"]["cooperative"], true);
        assert!(v["agent"]["known"].is_u64());
        let v: serde_json::Value = serde_json::from_str(&w.inspect_json(3, 0).unwrap()).unwrap();
        assert_eq!((v["group"].is_null(), v["agent"].is_null()), (true, true));
        assert!(w.inspect(11, 0).is_err());
        let mut w = world(1, 100, Strategy::K(0), |_| {});
        w.play();
        let v: serde_json::Value = serde_json::from_str(&w.inspect_json(0, 0).unwrap()).unwrap();
        assert!(v["agent"]["known"].is_null(), "perfect information");
    }

    #[test]
    fn runs_stop_at_the_end_and_follow_their_seed() {
        let c = ImageConfig {
            end: 30,
            mutation: 0.01,
            ..Default::default()
        };
        let mut a = ImageWorld::new(c.clone(), 7).unwrap();
        let mut b = ImageWorld::new(c.clone(), 7).unwrap();
        let mut other = ImageWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(40);
        }
        assert_eq!(a.tick, 30);
        assert!(a.is_finished() && Model::finished(&a));
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
        assert_eq!(a.stats.history().len(), 31);
        let first = a.agents[0].id;
        assert!(a.locate(first).is_some() && a.locate(0).is_none());
        assert_eq!(a.agents_csv().lines().count(), 101);
        assert_eq!(a.series_csv().lines().count(), 32);
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(1, 10, Strategy::K(0), |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("c".to_string(), json!(0.25))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.c, 0.1);
        w.step();
        assert_eq!(w.config.c, 0.25);
    }

    #[test]
    fn keyframes_replay_under_every_switch() {
        let c = ImageConfig {
            groups: 3,
            group_size: 20,
            local: 0.7,
            mutation: 0.05,
            strategies: vec![Class::And, Class::Standing, Class::Q],
            ..Default::default()
        };
        let mut w = ImageWorld::new(c, 2).unwrap();
        type Edit = fn(&mut ImageConfig);
        let edits: [Edit; 6] = [
            |c| c.execution_error = 0.05,
            |c| c.perception_error = 0.05,
            |c| c.rounds_kind = RoundsKind::Random,
            |c| c.offset = Offset::None,
            |c| c.records = Records::Score,
            |c| c.clamp = 3,
        ];
        for edit in edits {
            edit(&mut w.config);
            w.run(10);
        }
        let kept = w.keyframe();
        assert!(kept.views.is_empty() && kept.marks.is_empty());
        assert!(w.private(), "the live world keeps its records");
        let inspect_all = |w: &ImageWorld| {
            let (width, height) = Model::size(w);
            let mut out = Vec::new();
            for y in 0..height {
                for x in 0..width {
                    out.push(w.inspect_json(x, y).unwrap());
                }
            }
            out
        };
        assert_eq!(inspect_all(&kept), inspect_all(&w));
        let again = kept.clone().keyframe();
        assert_eq!(
            inspect_all(&again),
            inspect_all(&w),
            "a keyframe of a keyframe"
        );
        let mut back = kept;
        back.run(20);
        w.run(20);
        assert_eq!(back.fingerprint(), w.fingerprint());
        assert_eq!(inspect_all(&back), inspect_all(&w));
        assert!(
            back.seen.is_empty(),
            "the next generation drops the stand-in"
        );
    }

    #[test]
    fn a_group_of_two_with_observers_has_only_the_recipient_watching() {
        let mut w = world(1, 2, Strategy::K(0), |c| {
            c.information = Information::Observers;
            c.observers = 10.0;
        });
        assert_eq!(w.config.watch_probability(), 0.0);
        w.run(5);
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn more_observers_than_bystanders_means_everyone_watches() {
        let w = world(1, 5, Strategy::K(0), |c| {
            c.information = Information::Observers;
            c.observers = 10.0;
        });
        assert_eq!(w.config.watch_probability(), 1.0);
    }

    #[test]
    fn unbounded_scores_run_long_and_still_draw() {
        let mut w = world(2, 10, Strategy::K(-5), |c| {
            c.clamp = 0;
            c.rounds = 2000;
        });
        w.run(2);
        let mut buf = Vec::new();
        for mode in ["strategy", "score", "payoff"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert_eq!(w.population(), 20);
        // K(-5) always helps, so 2,000 rounds a group push some agent's
        // unbounded score well past the ±5 the colour scale clamps to; in
        // score mode its pixel is exactly the scale's extreme colour.
        let i = w
            .agents
            .iter()
            .position(|a| a.score.abs() > 5)
            .expect("2,000 rounds a group push some score past ±5");
        let extreme = if w.agents[i].score > 0 { BLUE } else { RED };
        w.render("score", "", &mut buf).unwrap();
        let (width, _) = Model::size(&w);
        let (x, y) = w.cell(i);
        let at = (y * width + x) as usize * 4;
        assert_eq!(&buf[at..at + 3], &extreme[..]);
    }

    #[test]
    fn a_single_round_generation_with_all_payoffs_zero_still_reproduces() {
        // Defectors with no offset: nobody earns (K(5) and K(6) both refuse
        // at score 0), the pool is empty, and the next generation is drawn
        // uniformly (Decision on empty pools). Seed each group half K(6),
        // half K(5): a uniform draw should keep both in roughly equal
        // shares, not collapse to one. The first generation only plays (its
        // strategies are the seed); the second is where `reproduce` first
        // runs, from that first generation's all-zero payoffs.
        let mut counts = [0u32; 2];
        for seed in 1..=200 {
            let mut w = ImageWorld::new(
                ImageConfig {
                    groups: 3,
                    group_size: 4,
                    rounds: 1,
                    offset: Offset::None,
                    initial: Initial::Seeded(Seeded {
                        only: Strategy::K(6),
                        invader: Some(Strategy::K(5)),
                        share: 0.5,
                    }),
                    ..Default::default()
                },
                seed,
            )
            .unwrap();
            w.run(2);
            assert_eq!(w.population(), 12);
            for a in &w.agents {
                match a.strategy {
                    Strategy::K(6) => counts[0] += 1,
                    Strategy::K(5) => counts[1] += 1,
                    other => panic!("unexpected strategy {other:?}"),
                }
            }
        }
        assert!(counts[0] > 0 && counts[1] > 0, "{counts:?}");
        let total = f64::from(counts[0] + counts[1]);
        for c in counts {
            assert!((f64::from(c) / total - 0.5).abs() < 0.05, "{counts:?}");
        }
    }
}
