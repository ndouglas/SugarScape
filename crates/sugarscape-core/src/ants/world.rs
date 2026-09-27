//! The Ants and Recruitment world. Under Kirman's rule each step is a number
//! of random meetings: an ant changes source on its own with probability ε,
//! or is recruited by the ant it meets; under Alfarano and Milaković's, each
//! ant in turn switches with a probability that grows with its neighbors
//! elsewhere.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{AntsConfig, Conversion, Network, Rule, Start, MAX_SOURCES};
use super::stats::{AntsSnapshot, Regimes, Running, HOLD};
use super::theory;
use super::view::{
    degree_color, grid, row, row_of, share_at, BAR, GRID_X, HERDER, HIST_W, HIST_X, LINE, LONER,
    MARK, SHOWN, SOURCES, TALL, THEORY, TIME_W,
};
use crate::config::FieldError;
use crate::export;
use crate::graph::{self, Graph};
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Ants at each source (unused sources 0).
type Split = [u32; MAX_SOURCES as usize];

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AntsMode {
    Source,
    Independent,
    Degree,
}

impl std::str::FromStr for AntsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "source" => Self::Source,
            "independent" => Self::Independent,
            "degree" => Self::Degree,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AntsInspection {
    pub site: AntsCell,
    /// `time`, `histogram` or `ants`; null between panels.
    pub panel: Option<&'static str>,
    /// The step of a time-panel column.
    pub step: Option<u64>,
    /// Each source's share at that step.
    pub shares: Option<Vec<f64>>,
    /// The first source's share a histogram row stands for.
    pub share: Option<f64>,
    /// Steps spent at that share (histogram).
    pub count: Option<u32>,
    /// The long-run share of steps theory gives that row, if any.
    pub theory: Option<f64>,
    /// The ant at a grid cell.
    pub member: Option<AntView>,
    /// Always null: cells are read where they are.
    pub agent: Option<AntView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AntsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AntView {
    pub id: u64,
    /// Its source, from 1.
    pub source: u32,
    /// Whether it never herds.
    pub independent: bool,
    /// Whom it can meet (N − 1 on the complete graph).
    pub degree: u32,
    /// How many of those are at another source.
    pub elsewhere: u32,
}

#[derive(Clone)]
pub struct AntsWorld {
    pub config: AntsConfig,
    /// Completed steps.
    pub tick: u64,
    rng: SimRng,
    graph: Arc<Graph>,
    independent: Arc<Vec<bool>>,
    source: Vec<u8>,
    counts: Split,
    /// Each ant's neighbors at another source (empty on the complete graph).
    elsewhere: Vec<u32>,
    /// The split after each of the last `SHOWN` steps, newest last.
    recent: VecDeque<Split>,
    /// Steps by ants at the first source, 0..=N.
    hist: Vec<u32>,
    /// The long-run distribution of ants at the first source, if theory applies.
    theory: Option<Arc<Vec<f64>>>,
    running: Running,
    regimes: Regimes,
    extreme_steps: u64,
    pub stats: Stats<AntsSnapshot>,
}

/// Who can meet whom, drawn from the seed before anything else.
fn build(c: &AntsConfig, rng: &mut SimRng) -> Graph {
    let n = c.ants as usize;
    let d = c.degree as usize;
    let adj = match c.network {
        Network::Complete => return Graph::default(),
        Network::Ring => graph::ring(n, d),
        Network::SmallWorld => {
            let mut adj = graph::ring(n, d);
            graph::shortcuts(&mut adj, (0.1 * n as f64).round() as usize, rng);
            adj
        }
        Network::Random => graph::gnp(n, c.link, rng),
        Network::ScaleFree => graph::barabasi_albert(n, d / 2, rng),
    };
    Graph::from_lists(adj)
}

/// The distribution theory gives for `c` on `g`: Kirman's exact chain on the
/// complete graph with two sources, or Alfarano and Milaković's mean field;
/// neither with ants that never herd.
fn long_run(c: &AntsConfig, g: &Graph) -> Option<Vec<f64>> {
    if c.independent_count() > 0 {
        return None;
    }
    match c.rule {
        Rule::Kirman if c.network == Network::Complete && c.sources == 2 => theory::kirman(c),
        Rule::Kirman => None,
        Rule::Alfarano => {
            let n = f64::from(c.ants);
            let d = if g.is_complete() {
                n - 1.0
            } else {
                2.0 * g.edges().len() as f64 / n
            };
            theory::alfarano(c, d)
        }
    }
}

impl AntsWorld {
    pub fn new(config: AntsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.ants as usize;
        let graph = build(&config, &mut rng);
        // q·N ants never herd: a partial Fisher–Yates shuffle.
        let q = config.independent_count();
        let mut order: Vec<u32> = (0..n as u32).collect();
        let mut independent = vec![false; n];
        for i in 0..q {
            let j = i + rng.gen_range(0..(n - i) as u32) as usize;
            order.swap(i, j);
            independent[order[i] as usize] = true;
        }
        let source: Vec<u8> = match config.start {
            Start::Random => (0..n)
                .map(|_| rng.gen_range(0..config.sources) as u8)
                .collect(),
            Start::One => vec![0; n],
        };
        let mut counts = Split::default();
        for &s in &source {
            counts[s as usize] += 1;
        }
        let theory = long_run(&config, &graph).map(Arc::new);
        let mut world = AntsWorld {
            hist: vec![0; n + 1],
            elsewhere: Vec::new(),
            graph: Arc::new(graph),
            independent: Arc::new(independent),
            source,
            counts,
            config,
            tick: 0,
            rng,
            recent: VecDeque::new(),
            theory,
            running: Running::default(),
            regimes: Regimes::default(),
            extreme_steps: 0,
            stats: Stats::default(),
        };
        world.recount();
        world.record();
        Ok(world)
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    /// Ants at each source now.
    pub fn counts(&self) -> &[u32] {
        &self.counts[..self.config.sources as usize]
    }

    pub fn sources(&self) -> &[u8] {
        &self.source
    }

    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// The long-run distribution of ants at the first source, if theory applies.
    pub fn theory(&self) -> Option<&[f64]> {
        self.theory.as_deref().map(Vec::as_slice)
    }

    fn recount(&mut self) {
        if self.graph.is_complete() {
            self.elsewhere.clear();
            return;
        }
        self.elsewhere = (0..self.source.len())
            .map(|i| {
                self.graph
                    .of(i)
                    .iter()
                    .filter(|&&j| self.source[j as usize] != self.source[i])
                    .count() as u32
            })
            .collect();
    }

    /// Ant `i`'s neighbors at another source.
    fn away(&self, i: usize) -> u32 {
        if self.graph.is_complete() {
            self.config.ants - self.counts[self.source[i] as usize]
        } else {
            self.elsewhere[i]
        }
    }

    fn degree(&self, i: usize) -> u32 {
        if self.graph.is_complete() {
            self.config.ants - 1
        } else {
            self.graph.of(i).len() as u32
        }
    }

    fn move_to(&mut self, i: usize, s: u8) {
        let old = self.source[i];
        if old == s {
            return;
        }
        self.counts[old as usize] -= 1;
        self.counts[s as usize] += 1;
        self.source[i] = s;
        if !self.graph.is_complete() {
            let g = Arc::clone(&self.graph);
            let mut away = 0;
            for &j in g.of(i) {
                let j = j as usize;
                let sj = self.source[j];
                if sj == old {
                    self.elsewhere[j] += 1;
                } else if sj == s {
                    self.elsewhere[j] -= 1;
                }
                if sj != s {
                    away += 1;
                }
            }
            self.elsewhere[i] = away;
        }
    }

    /// A uniformly random source other than `s`.
    fn other(&mut self, s: u8) -> u8 {
        if self.config.sources == 2 {
            1 - s
        } else {
            let r = self.rng.gen_range(0..self.config.sources - 1) as u8;
            if r >= s {
                r + 1
            } else {
                r
            }
        }
    }

    /// One of Kirman's meetings.
    fn meet(&mut self) {
        let n = self.config.ants;
        let i = self.rng.gen_range(0..n) as usize;
        let u: f64 = self.rng.gen();
        let eps = self.config.epsilon;
        if u < eps {
            let s = self.other(self.source[i]);
            self.move_to(i, s);
            return;
        }
        if self.independent[i] {
            return;
        }
        let j = if self.graph.is_complete() {
            let r = self.rng.gen_range(0..n - 1) as usize;
            if r >= i {
                r + 1
            } else {
                r
            }
        } else {
            let k = self.graph.of(i).len();
            if k == 0 {
                return;
            }
            self.graph.of(i)[self.rng.gen_range(0..k as u32) as usize] as usize
        };
        let (si, sj) = (self.source[i], self.source[j]);
        if si == sj {
            return;
        }
        let nf = f64::from(n);
        let xi = f64::from(self.counts[si as usize]) / nf;
        let xj = f64::from(self.counts[sj as usize]) / nf;
        let f = 1.0 + self.config.pull * (xj - xi);
        let joins = match self.config.conversion {
            Conversion::Kirman => u < eps + ((1.0 - self.config.delta) * f).clamp(0.0, 1.0),
            Conversion::Footnote => {
                self.rng.gen::<f64>() < (self.config.gamma() * f).clamp(0.0, 1.0)
            }
        };
        if joins {
            self.move_to(i, sj);
        }
    }

    /// One of Alfarano and Milaković's sweeps: each ant in turn.
    fn sweep(&mut self) {
        let n = self.config.ants;
        let (a, lambda) = (self.config.a, self.config.lambda);
        let denominator = a + lambda * f64::from(n);
        for i in 0..n as usize {
            let l = if self.independent[i] { 0.0 } else { lambda };
            let p = (a + l * f64::from(self.away(i))) / denominator;
            if self.rng.gen::<f64>() < p {
                self.move_to(i, 1 - self.source[i]);
            }
        }
    }

    pub fn step(&mut self) {
        match self.config.rule {
            Rule::Kirman => {
                for _ in 0..self.config.meetings {
                    self.meet();
                }
            }
            Rule::Alfarano => self.sweep(),
        }
        self.tick += 1;
        if self.recent.len() == SHOWN {
            self.recent.pop_front();
        }
        self.recent.push_back(self.counts);
        let n = f64::from(self.config.ants);
        self.hist[self.counts[0] as usize] += 1;
        self.running.push(f64::from(self.counts[0]) / n);
        let (top, x) = self.top();
        self.regimes.see(self.tick, top, x);
        if x >= HOLD {
            self.extreme_steps += 1;
        }
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

    /// The largest source (the lower on a tie) and its share.
    fn top(&self) -> (u32, f64) {
        let (s, &k) = self
            .counts()
            .iter()
            .enumerate()
            .rev()
            .max_by_key(|(_, &k)| k)
            .expect("at least two sources");
        (s as u32, f64::from(k) / f64::from(self.config.ants))
    }

    fn record(&mut self) {
        let n = f64::from(self.config.ants);
        let s = AntsSnapshot {
            tick: self.tick,
            share: f64::from(self.counts[0]) / n,
            top_share: self.top().1,
            variance: self.running.variance(),
            theory_variance: self
                .theory
                .as_deref()
                .map_or(f64::NAN, |p| theory::variance(p)),
            flips: self.regimes.flips,
            residence: self.regimes.residence(),
            extreme: if self.tick == 0 {
                0.0
            } else {
                self.extreme_steps as f64 / self.tick as f64
            },
        };
        self.stats.push(s);
    }

    fn color(&self, mode: AntsMode, i: usize, top: usize) -> [u8; 3] {
        match mode {
            AntsMode::Source => SOURCES[self.source[i] as usize],
            AntsMode::Independent => {
                if self.independent[i] {
                    LONER
                } else {
                    HERDER
                }
            }
            AntsMode::Degree => degree_color(self.degree(i) as usize, top),
        }
    }

    fn view(&self, i: usize) -> AntView {
        AntView {
            id: i as u64 + 1,
            source: u32::from(self.source[i]) + 1,
            independent: self.independent[i],
            degree: self.degree(i),
            elsewhere: self.away(i),
        }
    }

    /// Theory's long-run share of steps per histogram row.
    fn theory_rows(&self) -> Option<Vec<f64>> {
        let p = self.theory.as_deref()?;
        let n = self.config.ants;
        let mut rows = vec![0.0; TALL];
        for (k, &q) in p.iter().enumerate() {
            rows[row_of(k as u32, n)] += q;
        }
        Some(rows)
    }

    fn hist_rows(&self) -> Vec<u32> {
        let n = self.config.ants;
        let mut rows = vec![0u32; TALL];
        for (k, &c) in self.hist.iter().enumerate() {
            rows[row_of(k as u32, n)] += c;
        }
        rows
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<AntsInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let n = self.config.ants;
        let mut out = AntsInspection {
            site: AntsCell { x, y },
            panel: None,
            step: None,
            shares: None,
            share: None,
            count: None,
            theory: None,
            member: None,
            agent: None,
        };
        if cx < TIME_W {
            if let Some(split) = self.recent.get(cx) {
                out.panel = Some("time");
                out.step = Some(self.tick - self.recent.len() as u64 + cx as u64 + 1);
                out.shares = Some(
                    split[..self.config.sources as usize]
                        .iter()
                        .map(|&k| f64::from(k) / f64::from(n))
                        .collect(),
                );
            }
        } else if (HIST_X..HIST_X + HIST_W).contains(&cx) {
            out.panel = Some("histogram");
            out.share = Some(share_at(cy));
            out.count = Some(self.hist_rows()[cy]);
            out.theory = self.theory_rows().map(|r| r[cy]);
        } else if cx >= GRID_X {
            let (side, cell) = grid(n);
            let (gx, gy) = ((cx - GRID_X) / cell, cy / cell);
            let i = gy * side + gx;
            if gx < side && i < n as usize {
                out.panel = Some("ants");
                out.member = Some(self.view(i));
            }
        }
        Ok(out)
    }
}

impl Model for AntsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Ants(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        AntsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.source.len()
    }

    /// FNV-1a over the tick, every ant's source and the running statistics.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.source);
        for c in &self.hist {
            eat(&c.to_le_bytes());
        }
        for b in self.running.bits() {
            eat(&b.to_le_bytes());
        }
        eat(&self.regimes.flips.to_le_bytes());
        eat(&self.regimes.span.to_le_bytes());
        eat(&self.extreme_steps.to_le_bytes());
        h
    }

    fn size(&self) -> (u32, u32) {
        let (side, cell) = grid(self.config.ants);
        ((GRID_X + side * cell) as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: AntsMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let n = self.config.ants;
        let marks = [row(HOLD), row(1.0 - HOLD)];
        for x in (0..TIME_W).chain(HIST_X..HIST_X + HIST_W) {
            for &y in &marks {
                c.put(x, y, MARK);
            }
        }
        // One line with two sources (Kirman's Figure II); one per source otherwise.
        let sources = self.config.sources as usize;
        let lines: Vec<(usize, [u8; 3])> = if sources == 2 {
            vec![(0, LINE)]
        } else {
            (0..sources).rev().map(|s| (s, SOURCES[s])).collect()
        };
        for (s, color) in lines {
            for (k, split) in self.recent.iter().enumerate() {
                let y = row_of(split[s], n);
                let to = self.recent.get(k + 1).map_or(y, |next| row_of(next[s], n));
                c.column(k, y, to, color);
            }
        }
        // The histogram on the same vertical scale, theory as a dot per row.
        let rows = self.hist_rows();
        let steps = self.tick.max(1) as f64;
        let seen: Vec<f64> = rows.iter().map(|&k| f64::from(k) / steps).collect();
        let expected = self.theory_rows();
        let top = seen
            .iter()
            .chain(expected.iter().flatten())
            .copied()
            .fold(0.0, f64::max);
        let len = |v: f64| {
            if top > 0.0 {
                (v / top * (HIST_W - 1) as f64).round() as usize
            } else {
                0
            }
        };
        for (y, &v) in seen.iter().enumerate() {
            for x in 0..len(v) {
                c.put(HIST_X + x, y, BAR);
            }
        }
        if let Some(e) = &expected {
            for (y, &v) in e.iter().enumerate() {
                if v > 0.0 {
                    c.put(HIST_X + len(v), y, THEORY);
                }
            }
        }
        let (side, cell) = grid(n);
        let most = (0..self.source.len())
            .map(|i| self.degree(i))
            .max()
            .unwrap_or(0) as usize;
        for i in 0..self.source.len() {
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
        let mut out = String::from("id,source,independent,degree,elsewhere\n");
        for i in 0..self.source.len() {
            let v = self.view(i);
            writeln!(
                out,
                "{},{},{},{},{}",
                v.id,
                v.source,
                u8::from(v.independent),
                v.degree,
                v.elsewhere
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
        let ModelConfig::Ants(next) = next else {
            return Err(wrong_model(ModelKind::Ants, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.theory = long_run(&next, &self.graph).map(Arc::new);
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

    fn config(edit: impl FnOnce(&mut AntsConfig)) -> AntsConfig {
        let mut c = AntsConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut AntsConfig)) -> AntsWorld {
        AntsWorld::new(config(edit), 1).unwrap()
    }

    /// The elsewhere counts kept incrementally match a recount.
    fn consistent(w: &AntsWorld) {
        let mut fresh = w.clone();
        fresh.recount();
        assert_eq!(w.elsewhere, fresh.elsewhere);
        let mut counts = Split::default();
        for &s in &w.source {
            counts[s as usize] += 1;
        }
        assert_eq!(w.counts, counts);
    }

    #[test]
    fn ants_start_at_random_sources_or_all_at_one() {
        let w = world(|c| c.ants = 1000);
        assert!((400..600).contains(&w.counts()[0]), "{:?}", w.counts());
        let one = world(|c| c.start = Start::One);
        assert_eq!(one.counts(), [100, 0]);
        let three = world(|c| c.sources = 3);
        assert_eq!(three.counts().len(), 3);
        assert!(three.counts().iter().all(|&k| k > 15));
    }

    #[test]
    fn a_long_run_matches_kirman_s_exact_distribution() {
        // Figure Ic: centered; the histogram of 100 000 steps of 50 meetings
        // against the exact chain.
        let mut w = world(|c| {
            c.epsilon = 0.15;
            c.delta = 0.3;
        });
        w.run(100_000);
        let p = w.theory().unwrap().to_vec();
        let total: u32 = w.hist.iter().sum();
        let tv: f64 = w
            .hist
            .iter()
            .zip(&p)
            .map(|(&h, &q)| (f64::from(h) / f64::from(total) - q).abs())
            .sum::<f64>()
            / 2.0;
        assert!(tv < 0.03, "total variation {tv}");
        let s = w.stats.latest().unwrap();
        assert!((s.variance - s.theory_variance).abs() < 0.001, "{s:?}");
    }

    #[test]
    fn self_conversion_moves_to_another_source() {
        // ε 1: every meeting moves its ant, never to where it was.
        let mut w = world(|c| {
            c.sources = 4;
            c.epsilon = 1.0;
            c.delta = 1.0;
            c.meetings = 1;
        });
        for _ in 0..200 {
            let before = w.source.clone();
            w.step();
            let moved: Vec<usize> = (0..100).filter(|&i| before[i] != w.source[i]).collect();
            assert_eq!(moved.len(), 1);
        }
        assert!(w.counts().iter().all(|&k| k > 0));
    }

    #[test]
    fn recruiting_follows_one_minus_delta_and_the_pull() {
        // ε 0, δ 0: a meeting across sources always converts; the colony is
        // absorbed at one source (Kirman's martingale).
        let mut w = world(|c| {
            c.epsilon = 0.0;
            c.delta = 0.0;
        });
        w.run(2000);
        assert!(w.counts().contains(&100), "{:?}", w.counts());
        assert!(w.theory().is_none(), "no stationary distribution at ε 0");
        // δ 1 and ε 0: nobody ever moves.
        let mut still = world(|c| {
            c.epsilon = 0.0;
            c.delta = 1.0;
        });
        let before = still.source.clone();
        still.run(100);
        assert_eq!(still.source, before);
    }

    #[test]
    fn independent_ants_are_never_recruited() {
        let mut w = world(|c| {
            c.independent = 0.2;
            c.epsilon = 0.0;
            c.delta = 0.0;
        });
        let loners: Vec<usize> = (0..100).filter(|&i| w.independent[i]).collect();
        assert_eq!(loners.len(), 20);
        let before: Vec<u8> = loners.iter().map(|&i| w.source[i]).collect();
        w.run(500);
        let after: Vec<u8> = loners.iter().map(|&i| w.source[i]).collect();
        assert_eq!(before, after);
        assert!(w.theory().is_none(), "theory assumes everyone herds");
    }

    #[test]
    fn networks_have_their_degrees_and_keep_counts_right() {
        for (network, links) in [
            (Network::Ring, 500),
            (Network::SmallWorld, 510),
            (Network::ScaleFree, 15 + 94 * 5),
        ] {
            let mut w = world(|c| {
                c.network = network;
                c.epsilon = 0.01;
            });
            assert_eq!(w.graph().edges().len(), links, "{network:?}");
            w.run(300);
            consistent(&w);
        }
        let mut r = world(|c| {
            c.network = Network::Random;
            c.link = 0.1;
            c.rule = Rule::Alfarano;
        });
        let links = r.graph().edges().len() as f64;
        assert!((links - 495.0).abs() < 90.0, "{links}");
        r.run(300);
        consistent(&r);
    }

    #[test]
    fn meetings_on_a_network_are_with_neighbors() {
        // A ring of degree 2 from all at the first source, with one ant moved:
        // only its two neighbors can be recruited to it.
        let mut w = world(|c| {
            c.network = Network::Ring;
            c.degree = 2;
            c.start = Start::One;
            c.epsilon = 0.0;
            c.delta = 0.0;
            c.meetings = 1;
        });
        w.move_to(50, 1);
        for _ in 0..50 {
            w.step();
        }
        consistent(&w);
        let moved: Vec<usize> = (0..100).filter(|&i| w.source[i] == 1).collect();
        assert!(
            moved.windows(2).all(|p| p[1] == p[0] + 1),
            "one run: {moved:?}"
        );
    }

    #[test]
    fn alfarano_s_rule_switches_with_neighbors_elsewhere() {
        // λ 0: each ant switches with probability a/a = 1 every sweep.
        let mut w = world(|c| {
            c.rule = Rule::Alfarano;
            c.a = 1.0;
            c.lambda = 0.0;
        });
        let before = w.source.clone();
        w.step();
        assert!((0..100).all(|i| w.source[i] != before[i]));
        // a 0 from all at one source: nobody is ever elsewhere, nobody moves.
        let mut stuck = world(|c| {
            c.rule = Rule::Alfarano;
            c.a = 0.0;
            c.start = Start::One;
        });
        stuck.run(10);
        assert_eq!(stuck.counts(), [100, 0]);
        // The mean field sets the theory on a random graph.
        let r = world(|c| {
            c.rule = Rule::Alfarano;
            c.network = Network::Random;
            c.a = 0.05;
        });
        assert!(r.stats.latest().unwrap().theory_variance > 0.0);
    }

    #[test]
    fn statistics_track_variance_flips_and_extremes() {
        let mut w = world(|_| {});
        w.run(20_000);
        let s = w.stats.latest().unwrap().clone();
        let z = w.series("share").unwrap();
        let steps = &z[1..];
        let mean = steps.iter().sum::<f64>() / steps.len() as f64;
        let var = steps.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / steps.len() as f64;
        assert!((s.variance - var).abs() < 1e-9);
        assert!(s.flips >= 1, "Figure IIb flips within a million meetings");
        assert!(s.residence > 0.0);
        let held = steps.iter().filter(|&&x| x >= 0.8 || x <= 0.2).count() as f64;
        assert!((s.extreme - held / steps.len() as f64).abs() < 1e-12);
        assert!(
            (s.theory_variance - 0.1793).abs() < 1e-3,
            "{}",
            s.theory_variance
        );
        let three = {
            let mut t = world(|c| c.sources = 3);
            t.run(10);
            t.stats.latest().unwrap().clone()
        };
        assert!(
            three.theory_variance.is_nan(),
            "no theory for three sources"
        );
        let mut t = world(|c| c.sources = 3);
        t.run(1);
        assert!(
            Model::latest_json(&t).contains("\"theory_variance\":null"),
            "the page reads no theory as null"
        );
    }

    #[test]
    fn the_frame_draws_time_histogram_and_ants() {
        let mut w = world(|_| {});
        w.run(30);
        let mut buf = Vec::new();
        w.render("source", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw as usize, fh as usize), (GRID_X + 200, TALL));
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * fw as usize + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        let last = w.recent.back().unwrap()[0];
        assert_eq!(px(&buf, 29, row_of(last, 100)), LINE);
        assert_eq!(px(&buf, GRID_X, 0), SOURCES[w.source[0] as usize]);
        for mode in ["independent", "degree"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
    }

    #[test]
    fn inspect_reads_steps_histogram_rows_and_ants() {
        let mut w = world(|c| c.network = Network::Ring);
        w.run(5);
        let v = w.inspect(4, 0).unwrap();
        assert_eq!((v.panel, v.step), (Some("time"), Some(5)));
        let shares = v.shares.unwrap();
        assert!((shares.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(w.inspect(5, 0).unwrap().panel.is_none(), "no step yet");
        let y = row_of(w.counts()[0], 100) as u32;
        let h = w.inspect(HIST_X as u32, y).unwrap();
        assert_eq!(h.panel, Some("histogram"));
        assert!(h.count.unwrap() >= 1);
        assert!(
            h.theory.is_none(),
            "no theory on a ring under Kirman's rule"
        );
        let g = w.inspect(GRID_X as u32, 0).unwrap();
        let m = g.member.unwrap();
        assert_eq!((m.id, m.degree), (1, 10));
        assert!(m.elsewhere <= 10);
        assert!(g.agent.is_none());
        assert_eq!(Model::locate(&w, 1), None);
    }

    #[test]
    fn keyframes_restore_the_world_and_its_view() {
        let c = config(|c| c.network = Network::SmallWorld);
        let mut any = crate::model::ModelWorld::new(ModelConfig::Ants(c.clone()), 6).unwrap();
        any.model_mut().run(20);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("source", "", &mut before).unwrap();
        any.model_mut().run(30);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("source", "", &mut after).unwrap();
        assert_eq!(before, after);
        any.model_mut().run(30);
        let mut fresh = crate::model::ModelWorld::new(ModelConfig::Ants(c), 6).unwrap();
        fresh.model_mut().run(50);
        assert_eq!(
            any.model().fingerprint(),
            fresh.model().fingerprint(),
            "replays the same"
        );
    }

    #[test]
    fn live_edits_apply_and_the_colony_waits_for_reset() {
        let mut w = world(|_| {});
        let before = w.stats.latest().unwrap().theory_variance;
        let next = config(|c| {
            c.epsilon = 0.15;
            c.delta = 0.3;
            c.stop_at = 10;
        });
        Model::set_config(&mut w, ModelConfig::Ants(next)).unwrap();
        w.run(100);
        assert_eq!(w.tick, 10);
        let after = w.stats.latest().unwrap().theory_variance;
        assert!(after < before, "theory follows ε and δ: {before} → {after}");
        for (field, edit) in [
            ("ants", config(|c| c.ants = 101)),
            ("network", config(|c| c.network = Network::Ring)),
            ("sources", config(|c| c.sources = 3)),
            ("independent", config(|c| c.independent = 0.1)),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Ants(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.ants = 2),
            config(|c| {
                c.ants = 3;
                c.network = Network::Ring;
                c.degree = 2;
            }),
            config(|c| c.epsilon = 0.0),
            config(|c| {
                c.epsilon = 1.0;
                c.delta = 0.0;
            }),
            config(|c| c.pull = 10.0),
            config(|c| c.independent = 1.0),
            config(|c| {
                c.network = Network::Random;
                c.link = 0.001;
            }),
            config(|c| {
                c.conversion = Conversion::Footnote;
                c.epsilon = 1.0;
                c.delta = 1.0;
            }),
            config(|c| {
                c.rule = Rule::Alfarano;
                c.ants = 2;
                c.network = Network::Complete;
            }),
            config(|c| {
                c.rule = Rule::Alfarano;
                c.network = Network::ScaleFree;
                c.degree = 2;
                c.independent = 0.5;
            }),
            config(|c| {
                c.sources = 6;
                c.network = Network::SmallWorld;
                c.degree = 4;
            }),
        ] {
            let mut w = AntsWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            consistent(&w);
            let s = w.stats.latest().unwrap();
            assert!(s.share >= 0.0 && s.share <= 1.0, "{c:?}");
            assert!(s.variance.is_finite() && s.residence.is_finite(), "{c:?}");
            let mut buf = Vec::new();
            w.render("degree", "", &mut buf).unwrap();
        }
    }
}
