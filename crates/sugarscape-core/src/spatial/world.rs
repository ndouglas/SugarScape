//! The spatial Prisoner's Dilemma world: cooperators and defectors on a
//! lattice or random array, each scoring the sum of its games with its
//! neighbors and itself, each site then taken by the best-scoring candidate
//! (or by NBM94's Eq. 1), all at once or one site at a time.

use std::cell::Cell;
use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{SpatialConfig, Start, Update, Winning};
use super::geometry::Geometry;
use super::stats::{SpatialSnapshot, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::portable::{exp_neg, ln};
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, LENDER, RED};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

/// NM92's colours: blue C after C, red D after D, yellow D after C, green C
/// after D.
pub const C_AFTER_C: Rgb = BLUE;
pub const D_AFTER_D: Rgb = RED;
pub const D_AFTER_C: Rgb = BOTH;
pub const C_AFTER_D: Rgb = LENDER;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpatialMode {
    /// NM92's four colours.
    Change,
    /// Blue C, red D.
    Strategy,
    /// Cool to hot by score.
    Payoff,
}

impl std::str::FromStr for SpatialMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "change" => Self::Change,
            "strategy" => Self::Strategy,
            "payoff" => Self::Payoff,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SpatialInspection {
    pub site: CellXyz,
    /// The player on the cell (none on an empty cell of a random array).
    pub agent: Option<PlayerView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CellXyz {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlayerView {
    pub id: u64,
    /// "C" or "D", now and a generation ago.
    pub strategy: &'static str,
    pub previous: &'static str,
    pub score: f64,
    /// The player itself first, then its neighbors: who could take the site.
    pub candidates: Vec<Candidate>,
    /// The strategy the site would take next (deterministic winning), or
    /// null (probabilistic).
    pub next: Option<&'static str>,
    /// Eq. 1's probability that the site becomes C (probabilistic winning).
    pub p_c: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Candidate {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub strategy: &'static str,
    pub score: f64,
}

fn letter(c: bool) -> &'static str {
    if c {
        "C"
    } else {
        "D"
    }
}

#[derive(Clone)]
pub struct SpatialWorld {
    pub config: SpatialConfig,
    /// Fixed once built, so keyframes share it.
    pub geometry: Arc<Geometry>,
    /// Completed generations.
    pub tick: u64,
    /// Each player's strategy (true: C), now and a generation ago.
    coop: Vec<bool>,
    previous: Vec<bool>,
    /// How many of each player's neighbors cooperate, for `coop`: rebuilt
    /// by `rescore_all`, kept current by an asynchronous microstep.
    cooperating: Vec<u32>,
    /// Each player's score against its neighbors and itself, for `coop`.
    scores: Vec<f64>,
    rng: SimRng,
    /// The z-slice last drawn (a cube's view): where Inspect looks.
    view_z: Cell<u32>,
    pub stats: Stats<SpatialSnapshot>,
}

impl SpatialWorld {
    pub fn new(config: SpatialConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let geometry = Geometry::new(&config, &mut rng);
        let n = geometry.len();
        let mut coop = vec![true; n];
        match config.start {
            Start::Random => {
                let k = (config.defectors * n as f64).round() as usize;
                let mut order: Vec<usize> = (0..n).collect();
                order.shuffle(&mut rng);
                for &i in &order[..k] {
                    coop[i] = false;
                }
            }
            Start::SingleDefector => coop[geometry.central()] = false,
        }
        let mut w = SpatialWorld {
            config,
            previous: coop.clone(),
            scores: vec![0.0; n],
            cooperating: vec![0; n],
            coop,
            view_z: Cell::new(geometry.dims.2 / 2),
            geometry: Arc::new(geometry),
            tick: 0,
            rng,
            stats: Stats::default(),
        };
        w.rescore_all();
        w.record();
        Ok(w)
    }

    pub fn players(&self) -> usize {
        self.geometry.len()
    }

    pub fn is_cooperator(&self, i: usize) -> bool {
        self.coop[i]
    }

    /// Player `i`'s score for the current strategies: the sum over its
    /// neighbors of its payoff against each, plus a × its payoff against
    /// itself. A cooperator with k cooperating neighbors scores k + a; a
    /// defector scores k·b + (n − k)·ε + a·ε.
    pub fn score(&self, i: usize) -> f64 {
        let n = self.geometry.neighbors(i).len() as f64;
        let k = f64::from(self.cooperating[i]);
        let c = &self.config;
        if self.coop[i] {
            k + c.self_weight
        } else {
            k * c.b + (n - k) * c.epsilon + c.self_weight * c.epsilon
        }
    }

    /// Player `i`'s cooperating neighbors, counted afresh.
    fn count_cooperating(&self, i: usize) -> u32 {
        let nb = self.geometry.neighbors(i);
        nb.iter().filter(|&&j| self.coop[j as usize]).count() as u32
    }

    fn rescore_all(&mut self) {
        self.cooperating = (0..self.players())
            .map(|i| self.count_cooperating(i))
            .collect();
        self.scores = (0..self.players()).map(|i| self.score(i)).collect();
    }

    /// The strategy site `i` takes from its candidates (itself and its
    /// neighbors) with scores `score(j)`; probabilistic winning draws one
    /// uniform number.
    fn next_strategy(&mut self, i: usize, score: impl Fn(&Self, usize) -> f64) -> bool {
        match self.config.winning {
            Winning::Deterministic => self.winner(i, score),
            Winning::Probabilistic => {
                let u = self.rng.gen::<f64>();
                match self.p_c(i, &score) {
                    Some(p) => u < p,
                    None => self.coop[i],
                }
            }
        }
    }

    /// Deterministic winning: the strategy of the highest-scoring candidate;
    /// on a tie between a C and a D, the owner keeps its strategy.
    fn winner(&self, i: usize, score: impl Fn(&Self, usize) -> f64) -> bool {
        let candidates =
            std::iter::once(i as u32).chain(self.geometry.neighbors(i).iter().copied());
        let (mut best, mut c_top, mut d_top) = (f64::NEG_INFINITY, false, false);
        for j in candidates {
            let (s, c) = (score(self, j as usize), self.coop[j as usize]);
            if s > best {
                (best, c_top, d_top) = (s, c, !c);
            } else if s == best {
                c_top |= c;
                d_top |= !c;
            }
        }
        if c_top && d_top {
            self.coop[i]
        } else {
            c_top
        }
    }

    /// Eq. 1: Σ A^m s / Σ A^m over the candidates, or None when every
    /// weight is 0 (every score 0 with m > 0).
    fn p_c(&self, i: usize, score: impl Fn(&Self, usize) -> f64) -> Option<f64> {
        let candidates: Vec<(f64, bool)> = std::iter::once(i as u32)
            .chain(self.geometry.neighbors(i).iter().copied())
            .map(|j| (score(self, j as usize), self.coop[j as usize]))
            .collect();
        let m = self.config.m;
        let max = candidates.iter().map(|c| c.0).fold(0.0, f64::max);
        let weight = |a: f64| {
            if m == 0.0 {
                1.0 // 0^0 = 1: random drift
            } else if a == 0.0 {
                0.0
            } else {
                // A^m / max^m, from portable arithmetic.
                // Clamped: ln is good to a few ulps, so a score a hair
                // below the max can give ln(a) > ln(max).
                exp_neg((m * (ln(a) - ln(max))).min(0.0))
            }
        };
        if m > 0.0 && max == 0.0 {
            return None;
        }
        let (mut c_sum, mut total) = (0.0, 0.0);
        for &(a, c) in &candidates {
            let w = weight(a);
            total += w;
            if c {
                c_sum += w;
            }
        }
        (total > 0.0).then(|| c_sum / total)
    }

    /// One generation.
    pub fn step(&mut self) {
        self.apply_schedule();
        self.previous.clone_from(&self.coop);
        match self.config.update {
            Update::Synchronous => {
                let next: Vec<bool> = (0..self.players())
                    .map(|i| self.next_strategy(i, |w, j| w.scores[j]))
                    .collect();
                self.coop = next;
            }
            Update::Asynchronous => {
                let n = self.players() as u32;
                for _ in 0..n {
                    let i = self.rng.gen_range(0..n) as usize;
                    self.microstep(i);
                }
            }
        }
        self.rescore_all();
        self.tick += 1;
        self.record();
    }

    /// Asynchronous updating's microstep: site `i` and its neighbors are
    /// rescored from the current strategies, and `i` takes its new strategy
    /// at once.
    fn microstep(&mut self, i: usize) {
        let next = self.next_strategy(i, |w, j| w.score(j));
        if next != self.coop[i] {
            self.coop[i] = next;
            // Neighbor lists are symmetric: i is a neighbor of each of its
            // neighbors, once.
            for &j in self.geometry.neighbors(i) {
                let k = &mut self.cooperating[j as usize];
                if next {
                    *k += 1;
                } else {
                    *k -= 1;
                }
            }
        }
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
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
        if due.is_empty() {
            return;
        }
        for (path, value) in due {
            let next = ModelConfig::Spatial(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Spatial(next)) = next {
                self.config = next;
            }
        }
        // A new b, ε or a changes every score now.
        self.rescore_all();
    }

    fn record(&mut self) {
        let n = self.players();
        let mut s = SpatialSnapshot {
            tick: self.tick,
            players: n as u32,
            ..Default::default()
        };
        let (mut cs, mut ds, mut c_pay, mut d_pay) = (0u32, 0u32, 0.0, 0.0);
        for i in 0..n {
            let (now, was) = (self.coop[i], self.previous[i]);
            if now {
                cs += 1;
                c_pay += self.scores[i];
            } else {
                ds += 1;
                d_pay += self.scores[i];
            }
            match (was, now) {
                (true, false) => s.c_to_d += 1,
                (false, true) => s.d_to_c += 1,
                _ => {}
            }
        }
        s.fraction_c = if n == 0 {
            0.0
        } else {
            f64::from(cs) / n as f64
        };
        s.changed = if n == 0 {
            0.0
        } else {
            f64::from(s.c_to_d + s.d_to_c) / n as f64
        };
        s.mean_payoff_c = if cs == 0 {
            f64::NAN
        } else {
            c_pay / f64::from(cs)
        };
        s.mean_payoff_d = if ds == 0 {
            f64::NAN
        } else {
            d_pay / f64::from(ds)
        };
        self.stats.push(s);
    }

    fn color(&self, i: usize, mode: SpatialMode, max_score: f64) -> Rgb {
        let (now, was) = (self.coop[i], self.previous[i]);
        match mode {
            SpatialMode::Change => match (was, now) {
                (true, true) => C_AFTER_C,
                (false, false) => D_AFTER_D,
                (true, false) => D_AFTER_C,
                (false, true) => C_AFTER_D,
            },
            SpatialMode::Strategy => {
                if now {
                    C_AFTER_C
                } else {
                    D_AFTER_D
                }
            }
            SpatialMode::Payoff => {
                let t = if max_score > 0.0 {
                    self.scores[i] / max_score
                } else {
                    0.0
                };
                lerp(COOL, HOT, t)
            }
        }
    }

    /// The z-slice a frame shows: `slice:<z>` in a cube (clamped), else 0.
    fn slice(&self, layer: &str) -> u32 {
        let d = self.geometry.dims.2;
        layer
            .strip_prefix("slice:")
            .and_then(|z| z.parse::<u32>().ok())
            .map_or(d / 2, |z| z.min(d - 1))
    }

    pub fn inspect(&self, x: u32, y: u32, z: u32) -> Result<SpatialInspection, String> {
        let (w, h, d) = self.geometry.dims;
        if x >= w || y >= h || z >= d {
            return Err(format!("({x}, {y}, {z}) is outside the lattice"));
        }
        let agent = self.geometry.at(x, y, z).map(|i| {
            let candidates = std::iter::once(i as u32)
                .chain(self.geometry.neighbors(i).iter().copied())
                .map(|j| {
                    let (x, y, z) = self.geometry.xyz(j as usize);
                    Candidate {
                        x,
                        y,
                        z,
                        strategy: letter(self.coop[j as usize]),
                        score: self.scores[j as usize],
                    }
                })
                .collect();
            let (next, p_c) = match self.config.winning {
                Winning::Deterministic => (Some(letter(self.winner(i, |w, j| w.scores[j]))), None),
                Winning::Probabilistic => (None, self.p_c(i, |w, j| w.scores[j])),
            };
            PlayerView {
                id: i as u64,
                strategy: letter(self.coop[i]),
                previous: letter(self.previous[i]),
                score: self.scores[i],
                candidates,
                next,
                p_c,
            }
        });
        Ok(SpatialInspection {
            site: CellXyz { x, y, z },
            agent,
        })
    }
}

impl Model for SpatialWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Spatial(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        SpatialWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.players()
    }

    /// FNV-1a over the tick and every player's strategy now and a
    /// generation ago.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for chunk in self.coop.chunks(64).zip(self.previous.chunks(64)) {
            let pack = |bits: &[bool]| {
                bits.iter()
                    .enumerate()
                    .fold(0u64, |acc, (k, &b)| acc | (u64::from(b) << k))
            };
            eat(pack(chunk.0));
            eat(pack(chunk.1));
        }
        h
    }

    /// The frame is the lattice (a cube's `slice:<z>`), one pixel a cell.
    fn size(&self) -> (u32, u32) {
        (self.geometry.dims.0, self.geometry.dims.1)
    }

    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: SpatialMode = mode.parse()?;
        let (w, h, _) = self.geometry.dims;
        let z = self.slice(layer);
        self.view_z.set(z);
        let max_score = self.scores.iter().copied().fold(0.0, f64::max);
        buf.resize((w * h * 4) as usize, 0);
        for y in 0..h {
            for x in 0..w {
                let rgb = match self.geometry.at(x, y, z) {
                    Some(i) => self.color(i, mode, max_score),
                    None => BACKGROUND,
                };
                let o = ((y * w + x) * 4) as usize;
                buf[o..o + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
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
        let mut out = String::from("id,x,y,z,strategy,score\n");
        for i in 0..self.players() {
            let (x, y, z) = self.geometry.xyz(i);
            writeln!(
                out,
                "{i},{x},{y},{z},{},{}",
                letter(self.coop[i]),
                self.scores[i]
            )
            .unwrap();
        }
        out
    }

    /// The cell (x, y) of the slice last drawn (a cube's view).
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y, self.view_z.get())?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        (id < self.players() as u64).then(|| {
            let (x, y, _) = self.geometry.xyz(id as usize);
            (x, y)
        })
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Spatial(next) = next else {
            return Err(wrong_model(ModelKind::Spatial, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        self.rescore_all();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduledChange;
    use crate::spatial::config::{Boundary, Lattice};
    use serde_json::json;

    /// A `w` × `h` lattice of cooperators with fixed edges, after `edit`.
    fn world(w: u32, h: u32, edit: impl FnOnce(&mut SpatialConfig)) -> SpatialWorld {
        let mut c = SpatialConfig {
            width: w,
            height: h,
            defectors: 0.0,
            ..Default::default()
        };
        edit(&mut c);
        SpatialWorld::new(c, 1).unwrap()
    }

    /// Makes the players in the rectangle [x0, x1) × [y0, y1) defectors (or
    /// cooperators) and rescores.
    fn paint(w: &mut SpatialWorld, (x0, x1): (u32, u32), (y0, y1): (u32, u32), c: bool) {
        for y in y0..y1 {
            for x in x0..x1 {
                let i = w.geometry.at(x, y, 0).unwrap();
                w.coop[i] = c;
            }
        }
        w.previous.clone_from(&w.coop);
        w.rescore_all();
    }

    fn defectors(w: &SpatialWorld) -> usize {
        w.coop.iter().filter(|&&c| !c).count()
    }

    #[test]
    fn scores_sum_the_games_with_neighbors_and_self() {
        let mut w = world(5, 5, |c| c.b = 1.5);
        let mid = w.geometry.at(2, 2, 0).unwrap();
        assert_eq!(w.score(mid), 9.0, "a C among 8 C, with itself");
        paint(&mut w, (2, 3), (2, 3), false);
        assert_eq!(w.score(mid), 12.0, "a D among 8 C scores 8b");
        let next = w.geometry.at(1, 2, 0).unwrap();
        assert_eq!(w.score(next), 8.0, "a C beside the D: 7 C and itself");
        let mut solo = world(5, 5, |c| {
            c.self_weight = 0.0;
            c.epsilon = 0.1;
        });
        assert_eq!(solo.score(mid), 8.0, "no self-interaction");
        paint(&mut solo, (0, 5), (0, 5), false);
        assert!((solo.score(mid) - 0.8).abs() < 1e-12, "8 D neighbors × ε");
        solo.config.self_weight = 1.0;
        assert!((solo.score(mid) - 0.9).abs() < 1e-12, "plus a × ε");
    }

    #[test]
    fn nm92s_cluster_thresholds_hold_exactly() {
        // "If b > 1.8, a 2 × 2 cluster of D will continue to grow … for b <
        // 1.8, big D clusters shrink."
        for (b, grows) in [(1.85, true), (1.75, false)] {
            let mut w = world(30, 30, |c| c.b = b);
            paint(&mut w, (12, 18), (12, 18), false);
            w.step();
            assert_eq!(defectors(&w) > 36, grows, "a 6 × 6 D block at b = {b}");
        }
        let mut w = world(30, 30, |c| c.b = 1.85);
        paint(&mut w, (14, 16), (14, 16), false);
        w.step();
        assert!(defectors(&w) > 4, "a 2 × 2 D cluster grows at b = 1.85");
        // "If b < 2, a 2 × 2 or larger cluster of C will continue to grow;
        // for b > 2, C clusters do not grow."
        for (b, grows) in [(1.95, true), (2.05, false)] {
            let mut w = world(30, 30, |c| c.b = b);
            paint(&mut w, (0, 30), (0, 30), false);
            paint(&mut w, (14, 16), (14, 16), true);
            w.step();
            let cs = w.coop.iter().filter(|&&c| c).count();
            assert_eq!(cs > 4, grows, "a 2 × 2 C cluster at b = {b}");
        }
    }

    #[test]
    fn the_best_candidate_wins_and_a_c_d_tie_keeps_the_owner() {
        let mut w = world(3, 3, |_| {});
        paint(&mut w, (0, 1), (0, 1), false);
        let mid = w.geometry.at(1, 1, 0).unwrap();
        let corner = w.geometry.at(0, 0, 0).unwrap();
        let scores = |best: usize, top: f64| {
            move |_: &SpatialWorld, j: usize| if j == best { top } else { 1.0 }
        };
        assert!(
            !w.winner(mid, scores(corner, 5.0)),
            "the D corner scores highest"
        );
        assert!(
            w.winner(mid, scores(mid, 5.0)),
            "the C owner scores highest"
        );
        // The D corner and the C owner tie for the top score.
        let tie = |_: &SpatialWorld, j: usize| if j == corner || j == mid { 5.0 } else { 1.0 };
        assert!(w.winner(mid, tie), "C owner keeps its site");
        paint(&mut w, (1, 2), (1, 2), false);
        assert!(!w.winner(mid, tie), "D owner keeps its site");
    }

    #[test]
    fn eq_1_weighs_candidates_by_their_scores() {
        let mut w = world(3, 3, |c| c.winning = Winning::Probabilistic);
        paint(&mut w, (0, 3), (0, 1), false); // the top row defects
        let mid = w.geometry.at(1, 1, 0).unwrap();
        w.config.m = 0.0;
        let p = w.p_c(mid, |w, j| w.scores[j]).unwrap();
        assert!((p - 6.0 / 9.0).abs() < 1e-12, "m = 0: the share of C, {p}");
        w.config.m = 1.0;
        let flat = |w: &SpatialWorld, j: usize| if w.coop[j] { 1.0 } else { 3.0 };
        let p = w.p_c(mid, flat).unwrap();
        assert!((p - 6.0 / 15.0).abs() < 1e-12, "m = 1: proportional, {p}");
        w.config.m = 200.0;
        let p = w.p_c(mid, flat).unwrap();
        assert!(p < 1e-12, "large m: the best scorer, {p}");
        let zero = |_: &SpatialWorld, _: usize| 0.0;
        assert_eq!(w.p_c(mid, zero), None, "every score 0: the owner keeps it");
        w.config.m = 0.0;
        assert!(
            (w.p_c(mid, zero).unwrap() - 6.0 / 9.0).abs() < 1e-12,
            "0^0 = 1"
        );
    }

    #[test]
    fn portable_powers_match_the_library() {
        for (a, max, m) in [
            (3.0, 9.0, 1.0),
            (0.5, 8.0, 20.0),
            (7.2, 7.2, 100.0),
            (1.3, 12.0, 0.5),
        ] {
            let ours = exp_neg(m * (ln(a) - ln(max)));
            let std = (a / max).powf(m);
            assert!(
                (ours - std).abs() <= 1e-12 * std.max(1e-300),
                "{a} {max} {m}: {ours} vs {std}"
            );
        }
    }

    #[test]
    fn an_asynchronous_microstep_sees_the_sites_already_changed() {
        let mut w = world(5, 1, |c| {
            c.b = 1.9;
            c.update = Update::Asynchronous;
        });
        // C C D C C in a row: the D's neighbors each score 1 + 1 = 2 (one C
        // neighbor and itself); the D scores 2b = 3.8 and takes both.
        paint(&mut w, (2, 3), (0, 1), false);
        let (left, right) = (
            w.geometry.at(1, 0, 0).unwrap(),
            w.geometry.at(3, 0, 0).unwrap(),
        );
        w.microstep(left);
        assert!(!w.coop[left], "the left C falls to the D");
        // Now the D at (2, 0) has one C neighbor (right) and scores 1.9;
        // (3, 0) scores 2 with its C neighbor (4, 0) and itself: it stays C.
        w.microstep(right);
        assert!(w.coop[right], "rescored after the left site fell");
    }

    #[test]
    fn asynchronous_counts_of_cooperating_neighbors_match_a_recount() {
        let mut w = SpatialWorld::new(
            SpatialConfig {
                lattice: Lattice::Random,
                width: 40,
                height: 40,
                occupancy: 0.6,
                radius: 3.5,
                defectors: 0.4,
                update: Update::Asynchronous,
                winning: Winning::Probabilistic,
                m: 2.0,
                ..Default::default()
            },
            11,
        )
        .unwrap();
        let n = w.players() as u32;
        let mut flips = 0;
        for _ in 0..5 * n {
            let i = w.rng.gen_range(0..n) as usize;
            let was = w.coop[i];
            w.microstep(i);
            flips += usize::from(was != w.coop[i]);
        }
        assert!(flips > 100, "the microsteps flip sites ({flips})");
        for i in 0..w.players() {
            assert_eq!(w.cooperating[i], w.count_cooperating(i), "player {i}");
        }
    }

    #[test]
    fn starts_place_exact_counts_and_the_single_defector_at_the_center() {
        let w = SpatialWorld::new(
            SpatialConfig {
                width: 50,
                height: 40,
                defectors: 0.25,
                ..Default::default()
            },
            3,
        )
        .unwrap();
        assert_eq!(defectors(&w), 500);
        let k = world(99, 99, |c| c.start = Start::SingleDefector);
        assert_eq!(defectors(&k), 1);
        assert!(!k.coop[k.geometry.at(49, 49, 0).unwrap()]);
    }

    #[test]
    fn the_change_colours_follow_nm92() {
        let mut w = world(2, 2, |_| {});
        let (a, b, c, d) = (0, 1, 2, 3);
        w.previous = vec![true, false, true, false];
        w.coop = vec![true, false, false, true];
        let mut buf = Vec::new();
        w.render("change", "", &mut buf).unwrap();
        let px = |buf: &[u8], i: usize| [buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2]];
        assert_eq!(px(&buf, a), C_AFTER_C);
        assert_eq!(px(&buf, b), D_AFTER_D);
        assert_eq!(px(&buf, c), D_AFTER_C);
        assert_eq!(px(&buf, d), C_AFTER_D);
        w.render("strategy", "", &mut buf).unwrap();
        assert_eq!(px(&buf, c), D_AFTER_D);
        assert!(w.render("nope", "", &mut buf).is_err());
    }

    #[test]
    fn a_cube_draws_the_chosen_slice_and_inspects_it() {
        let mut w = world(4, 4, |c| {
            c.lattice = Lattice::Cube;
            c.boundary = Boundary::Periodic;
        });
        let top = w.geometry.at(1, 2, 3).unwrap();
        w.coop[top] = false;
        w.previous.clone_from(&w.coop);
        w.rescore_all();
        let mut buf = Vec::new();
        w.render("strategy", "slice:3", &mut buf).unwrap();
        assert_eq!(buf.len(), 4 * 4 * 4);
        assert_eq!(&buf[(2 * 4 + 1) * 4..(2 * 4 + 1) * 4 + 3], &D_AFTER_D);
        let seen: serde_json::Value = serde_json::from_str(&w.inspect_json(1, 2).unwrap()).unwrap();
        assert_eq!(
            seen["site"]["z"], 3,
            "Inspect looks in the slice last drawn"
        );
        assert_eq!(seen["agent"]["strategy"], "D");
        assert_eq!(seen["agent"]["candidates"].as_array().unwrap().len(), 27);
        w.render("strategy", "slice:99", &mut buf).unwrap();
        assert_eq!(w.view_z.get(), 3, "clamped to the cube");
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(10, 10, |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("b".to_string(), json!(1.5))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.b, 1.9);
        w.step();
        assert_eq!(w.config.b, 1.5);
    }

    #[test]
    fn worlds_follow_their_seed() {
        let c = SpatialConfig {
            width: 40,
            height: 40,
            update: Update::Asynchronous,
            winning: Winning::Probabilistic,
            ..Default::default()
        };
        let mut a = SpatialWorld::new(c.clone(), 7).unwrap();
        let mut b = SpatialWorld::new(c.clone(), 7).unwrap();
        let mut other = SpatialWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(20);
        }
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
    }

    #[test]
    fn inspect_names_the_next_owner() {
        let mut w = world(5, 5, |_| {});
        paint(&mut w, (2, 3), (2, 3), false);
        let v = w.inspect(1, 2, 0).unwrap().agent.unwrap();
        assert_eq!((v.strategy, v.next, v.p_c), ("C", Some("D"), None));
        assert_eq!(v.candidates.len(), 9);
        assert_eq!(v.candidates[0].strategy, "C", "itself first");
        assert!(w.inspect(5, 0, 0).is_err());
    }
}
