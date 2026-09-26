//! The bounded-confidence world: each agent moves to the mean of the
//! opinions within its reach (HK's eq. BC), all at once or one at a time,
//! among everyone or among lattice neighbors.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{Interaction, Neighborhood, OpinionsConfig, Start, Updating};
use super::stats::{self, OpinionsSnapshot, SAME, STILL};
use super::view::{
    hue, opinion_at, row, site_cells, Canvas, BAND, HISTORY, LATTICE_X, STEP, TALL, WIDE,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Every site's torus neighbors (empty when everyone listens to everyone).
#[derive(Debug, Default, PartialEq)]
pub struct Neighbors {
    start: Vec<u32>,
    list: Vec<u32>,
}

impl Neighbors {
    fn new(c: &OpinionsConfig) -> Self {
        let l = &c.lattice;
        let (w, h) = (l.width as i32, l.height as i32);
        let offs: &[(i32, i32)] = match l.neighborhood {
            Neighborhood::Moore => &[
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ],
            Neighborhood::VonNeumann => &[(0, -1), (-1, 0), (1, 0), (0, 1)],
        };
        let mut start = Vec::new();
        let mut list = Vec::new();
        for y in 0..h {
            for x in 0..w {
                start.push(list.len() as u32);
                for &(dx, dy) in offs {
                    list.push(((y + dy).rem_euclid(h) * w + (x + dx).rem_euclid(w)) as u32);
                }
            }
        }
        start.push(list.len() as u32);
        Neighbors { start, list }
    }

    pub fn of(&self, i: usize) -> &[u32] {
        &self.list[self.start[i] as usize..self.start[i + 1] as usize]
    }
}

/// The diagram's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpinionsMode {
    /// Each line by its starting opinion (HK).
    Start,
    /// Each line by its current opinion.
    Opinion,
}

impl std::str::FromStr for OpinionsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "start" => Self::Start,
            "opinion" => Self::Opinion,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OpinionsInspection {
    pub site: OpinionsCell,
    /// The period of the diagram column, null off the diagram.
    pub period: Option<u64>,
    /// The opinion at the diagram row, null off the diagram.
    pub opinion: Option<f64>,
    /// The lattice site clicked, null off the lattice.
    pub lattice_site: Option<OpinionsCell>,
    /// The agents whose lines pass within one cell, or the site's agent.
    pub agents: Vec<OpinionAgent>,
    /// Always null: agents have no place to follow.
    pub agent: Option<OpinionAgent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct OpinionsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OpinionAgent {
    pub id: u64,
    pub start: f64,
    /// The opinion in the inspected period (the current one on the lattice).
    pub opinion: f64,
    pub epsilon_left: f64,
    pub epsilon_right: f64,
    /// How many agents it takes into account now, itself included.
    pub reaches: u32,
}

#[derive(Clone)]
pub struct OpinionsWorld {
    pub config: OpinionsConfig,
    /// Completed periods.
    pub tick: u64,
    start: Vec<f64>,
    x: Vec<f64>,
    neighbors: Arc<Neighbors>,
    rng: SimRng,
    /// The last `HISTORY` profiles, oldest first; the last is `x`.
    history: VecDeque<Arc<Vec<f64>>>,
    max_change: f64,
    stable_at: Option<u64>,
    pub stats: Stats<OpinionsSnapshot>,
}

impl OpinionsWorld {
    pub fn new(config: OpinionsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.agents as usize;
        let start: Vec<f64> = match config.start {
            Start::Random => (0..n).map(|_| rng.gen::<f64>()).collect(),
            Start::Regular => (0..n).map(|i| i as f64 / (n - 1) as f64).collect(),
        };
        let neighbors = Arc::new(match config.interaction {
            Interaction::All => Neighbors::default(),
            Interaction::Lattice => Neighbors::new(&config),
        });
        let mut world = OpinionsWorld {
            config,
            tick: 0,
            x: start.clone(),
            history: VecDeque::from([Arc::new(start.clone())]),
            start,
            neighbors,
            rng,
            max_change: 0.0,
            stable_at: None,
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    pub fn opinions(&self) -> &[f64] {
        &self.x
    }

    pub fn starts(&self) -> &[f64] {
        &self.start
    }

    /// Whether the run has stopped: stable with `stop_when_stable`.
    pub fn is_finished(&self) -> bool {
        self.config.stop_when_stable && self.stable_at.is_some()
    }

    /// Whether agent `i`, at `xi`, takes the opinion `xj` into account.
    fn within(&self, xi: f64, xj: f64) -> bool {
        let (l, r) = self.config.reach(xi);
        xi - l <= xj && xj <= xi + r
    }

    /// The mean of what agent `i` reaches in profile `x`, summed directly:
    /// itself and its neighbors on a lattice, everyone in index order otherwise.
    fn direct_mean(&self, i: usize, x: &[f64]) -> f64 {
        let xi = x[i];
        let (mut sum, mut count) = (0.0, 0u32);
        match self.config.interaction {
            Interaction::Lattice => {
                sum += xi;
                count += 1;
                for &j in self.neighbors.of(i) {
                    let xj = x[j as usize];
                    if self.within(xi, xj) {
                        sum += xj;
                        count += 1;
                    }
                }
            }
            Interaction::All => {
                for &xj in x {
                    if self.within(xi, xj) {
                        sum += xj;
                        count += 1;
                    }
                }
            }
        }
        sum / f64::from(count)
    }

    /// How many agents `i` reaches in profile `x`, itself included.
    fn reaches(&self, i: usize, x: &[f64]) -> u32 {
        match self.config.interaction {
            Interaction::Lattice => {
                1 + self
                    .neighbors
                    .of(i)
                    .iter()
                    .filter(|&&j| self.within(x[i], x[j as usize]))
                    .count() as u32
            }
            Interaction::All => x.iter().filter(|&&xj| self.within(x[i], xj)).count() as u32,
        }
    }

    /// Everyone's next opinion at once. Among everyone, each reach is a run
    /// of the sorted profile, summed from prefix sums.
    fn simultaneous(&self) -> Vec<f64> {
        let n = self.x.len();
        match self.config.interaction {
            Interaction::Lattice => (0..n).map(|i| self.direct_mean(i, &self.x)).collect(),
            Interaction::All => {
                let mut sorted = self.x.clone();
                sorted.sort_by(f64::total_cmp);
                let mut prefix = Vec::with_capacity(n + 1);
                prefix.push(0.0);
                for &v in &sorted {
                    prefix.push(prefix.last().unwrap() + v);
                }
                self.x
                    .iter()
                    .map(|&xi| {
                        let (l, r) = self.config.reach(xi);
                        let lo = sorted.partition_point(|&v| v < xi - l);
                        let hi = sorted.partition_point(|&v| v <= xi + r);
                        (prefix[hi] - prefix[lo]) / (hi - lo) as f64
                    })
                    .collect()
            }
        }
    }

    /// One period.
    pub fn step(&mut self) {
        let before = self.x.clone();
        match self.config.updating {
            Updating::Simultaneous => self.x = self.simultaneous(),
            Updating::SerialShuffled => {
                let mut order: Vec<usize> = (0..self.x.len()).collect();
                order.shuffle(&mut self.rng);
                for i in order {
                    self.x[i] = self.direct_mean(i, &self.x);
                }
            }
            Updating::SerialRandom => {
                let n = self.x.len();
                for _ in 0..n {
                    let i = self.rng.gen_range(0..n as u32) as usize;
                    self.x[i] = self.direct_mean(i, &self.x);
                }
            }
        }
        self.max_change = before
            .iter()
            .zip(&self.x)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        self.tick += 1;
        if self.max_change > STILL {
            self.stable_at = None;
        }
        if self.stable_at.is_none() && self.max_change <= STILL {
            self.stable_at = Some(self.tick);
        }
        if self.history.len() == HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(Arc::new(self.x.clone()));
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
        let mut sorted = self.x.clone();
        sorted.sort_by(f64::total_cmp);
        let n = sorted.len() as f64;
        let mut sizes = stats::clusters(&sorted);
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        let (splits, one_sided) = stats::splits(&sorted, |x| self.config.reach(x));
        self.stats.push(OpinionsSnapshot {
            tick: self.tick,
            clusters: sizes.len() as u32,
            largest: sizes.first().map_or(0.0, |&s| s as f64 / n),
            second: sizes.get(1).map_or(0.0, |&s| s as f64 / n),
            mean_opinion: sorted.iter().sum::<f64>() / n,
            median_opinion: stats::median(&sorted),
            range: sorted[sorted.len() - 1] - sorted[0],
            splits,
            one_sided_splits: one_sided,
            max_change: self.max_change,
            stable_at: self.stable_at.unwrap_or(self.tick),
        });
    }

    /// The period shown in diagram column `k` (0 is the oldest kept).
    fn period_of(&self, k: usize) -> u64 {
        self.tick + 1 - (self.history.len() - k) as u64
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<OpinionsInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let site = OpinionsCell { x, y };
        let (cx, cy) = (x as usize, y as usize);
        let view = |i: usize, opinion: f64| {
            let (l, r) = self.config.reach(self.x[i]);
            OpinionAgent {
                id: i as u64 + 1,
                start: self.start[i],
                opinion,
                epsilon_left: l,
                epsilon_right: r,
                reaches: self.reaches(i, &self.x),
            }
        };
        let mut out = OpinionsInspection {
            site,
            period: None,
            opinion: None,
            lattice_site: None,
            agents: Vec::new(),
            agent: None,
        };
        if cx < WIDE {
            // The nearest kept period at or left of the column.
            let k = (cx / STEP).min(self.history.len() - 1);
            let profile = &self.history[k];
            out.period = Some(self.period_of(k));
            out.opinion = Some(opinion_at(cy));
            out.agents = (0..profile.len())
                .filter(|&i| row(profile[i]).abs_diff(cy) <= 1)
                .map(|i| view(i, profile[i]))
                .collect();
        } else if self.config.interaction == Interaction::Lattice && cx >= LATTICE_X {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            let (sx, sy) = ((cx - LATTICE_X) / s, cy / s);
            if sx < l.width as usize && sy < l.height as usize {
                out.lattice_site = Some(OpinionsCell {
                    x: sx as u32,
                    y: sy as u32,
                });
                let i = sy * l.width as usize + sx;
                out.agents = vec![view(i, self.x[i])];
            }
        }
        Ok(out)
    }
}

impl Model for OpinionsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Opinions(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        OpinionsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.x.len()
    }

    /// FNV-1a over the tick and every opinion's bits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: [u8; 8]| {
            for b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick.to_le_bytes());
        for v in &self.x {
            eat(v.to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let w = match self.config.interaction {
            Interaction::All => WIDE,
            Interaction::Lattice => LATTICE_X + TALL,
        };
        (w as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: OpinionsMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let kept = self.history.len();
        // Gray between sorted neighbors within each other's reach, per period.
        for (k, profile) in self.history.iter().enumerate() {
            let mut sorted = profile.to_vec();
            sorted.sort_by(f64::total_cmp);
            let cols = if k + 1 == kept { 1 } else { STEP };
            for w in sorted.windows(2) {
                let gap = w[1] - w[0];
                if gap > SAME
                    && gap <= self.config.reach(w[0]).1
                    && gap <= self.config.reach(w[1]).0
                {
                    for d in 0..cols {
                        c.column(k * STEP + d, row(w[0]), row(w[1]), BAND);
                    }
                }
            }
        }
        // Lines, drawn in order of starting opinion (later ones on top, as in HK).
        let mut order: Vec<usize> = (0..self.x.len()).collect();
        order.sort_by(|&a, &b| self.start[a].total_cmp(&self.start[b]));
        for &i in &order {
            let color = match mode {
                OpinionsMode::Start => hue(self.start[i]),
                OpinionsMode::Opinion => hue(self.x[i]),
            };
            for k in 0..kept {
                let y0 = row(self.history[k][i]) as f64;
                if k + 1 == kept {
                    c.put(k * STEP, y0 as usize, color);
                    continue;
                }
                let y1 = row(self.history[k + 1][i]) as f64;
                for d in 0..STEP {
                    let a = y0 + (y1 - y0) * d as f64 / STEP as f64;
                    let b = y0 + (y1 - y0) * (d + 1) as f64 / STEP as f64;
                    c.column(k * STEP + d, a.round() as usize, b.round() as usize, color);
                }
            }
        }
        if self.config.interaction == Interaction::Lattice {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            for sy in 0..l.height as usize {
                for sx in 0..l.width as usize {
                    let color = hue(self.x[sy * l.width as usize + sx]);
                    for dy in 0..s {
                        for dx in 0..s {
                            c.put(LATTICE_X + sx * s + dx, sy * s + dy, color);
                        }
                    }
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
        let mut out = String::from("id,start,opinion,epsilon_left,epsilon_right,reaches\n");
        for i in 0..self.x.len() {
            let (l, r) = self.config.reach(self.x[i]);
            writeln!(
                out,
                "{},{},{},{},{},{}",
                i + 1,
                self.start[i],
                self.x[i],
                l,
                r,
                self.reaches(i, &self.x)
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Agents have no place: nothing to follow.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Opinions(next) = next else {
            return Err(wrong_model(ModelKind::Opinions, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        // A live edit to how agents move can unsettle a stable run; resuming
        // it needs `stable_at` cleared now, not left for the next `step()`,
        // since `run()` checks `is_finished()` before stepping at all.
        if self.config.confidence != next.confidence
            || self.config.epsilon != next.epsilon
            || self.config.epsilon_left != next.epsilon_left
            || self.config.epsilon_right != next.epsilon_right
            || self.config.bias != next.bias
            || self.config.updating != next.updating
        {
            self.stable_at = None;
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stable: every agent's reach holds only its own cluster, so nothing
    /// moves again — until a live edit changes the rule and unsettles it.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opinions::config::Confidence;

    fn config(edit: impl FnOnce(&mut OpinionsConfig)) -> OpinionsConfig {
        let mut c = OpinionsConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut OpinionsConfig)) -> OpinionsWorld {
        OpinionsWorld::new(config(edit), 1).unwrap()
    }

    #[test]
    fn starts_are_uniform_or_evenly_spaced() {
        let r = world(|_| {});
        assert!(r.starts().iter().all(|&x| (0.0..1.0).contains(&x)));
        let g = world(|c| {
            c.agents = 5;
            c.start = Start::Regular;
        });
        assert_eq!(g.starts(), [0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn a_period_moves_everyone_to_the_mean_within_reach() {
        let mut w = world(|c| {
            c.agents = 5;
            c.start = Start::Regular;
            c.epsilon = 0.3;
        });
        w.step();
        // 0 reaches 0 and 0.25; 0.25 reaches 0, 0.25, 0.5; and so on.
        let want = [0.125, 0.25, 0.5, 0.75, 0.875];
        for (a, b) in w.opinions().iter().zip(want) {
            assert!((a - b).abs() < 1e-12, "{:?}", w.opinions());
        }
        let s = w.stats.latest().unwrap();
        assert!((s.max_change - 0.125).abs() < 1e-12 && s.clusters == 5);
    }

    #[test]
    fn prefix_sums_agree_with_direct_summation() {
        for confidence in [
            Confidence::Symmetric,
            Confidence::Asymmetric,
            Confidence::OpinionDependent,
        ] {
            let w = world(|c| {
                c.confidence = confidence;
                c.epsilon = 0.3;
            });
            let fast = w.simultaneous();
            for (i, v) in fast.iter().enumerate() {
                assert!((v - w.direct_mean(i, &w.x)).abs() < 1e-12, "{confidence:?}");
            }
        }
    }

    #[test]
    fn asymmetric_reach_pulls_toward_the_favored_side() {
        let mut w = world(|c| {
            c.agents = 3;
            c.start = Start::Regular;
            c.confidence = Confidence::Asymmetric;
            c.epsilon_left = 0.0;
            c.epsilon_right = 0.5;
        });
        w.step();
        // 0 sees 0 and 0.5; 0.5 sees 0.5 and 1; 1 sees only itself.
        assert_eq!(w.opinions(), [0.25, 0.75, 1.0]);
        assert_eq!(w.stats.latest().unwrap().one_sided_splits, 2);
    }

    #[test]
    fn serial_updating_sees_opinions_as_they_change() {
        let edit = |c: &mut OpinionsConfig| {
            c.agents = 10;
            c.start = Start::Regular;
            c.epsilon = 0.3;
        };
        let mut all = world(edit);
        all.step();
        let mut w = world(|c| {
            edit(c);
            c.updating = Updating::SerialShuffled;
        });
        w.step();
        assert_ne!(
            w.opinions(),
            all.opinions(),
            "later agents saw earlier moves"
        );
        let mut r = world(|c| c.updating = Updating::SerialRandom);
        r.run(3);
        assert!(r.stats.latest().unwrap().max_change > 0.0);
    }

    #[test]
    fn lattice_agents_hear_only_their_neighbors() {
        let mut w = world(|c| {
            c.interaction = Interaction::Lattice;
            c.lattice.neighborhood = Neighborhood::VonNeumann;
            c.epsilon = 1.0;
        });
        assert_eq!(w.neighbors.of(0), [600, 24, 1, 25], "the torus wraps");
        let x = w.x.clone();
        w.step();
        let want = (x[0] + x[600] + x[24] + x[1] + x[25]) / 5.0;
        assert!((w.opinions()[0] - want).abs() < 1e-12);
    }

    #[test]
    fn a_split_profile_stabilizes_and_stops() {
        let mut w = world(|c| {
            c.agents = 50;
            c.start = Start::Regular;
            c.epsilon = 0.2;
        });
        w.run(1000);
        assert!(w.is_finished());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.stable_at, w.tick);
        assert!(s.max_change <= STILL);
        assert!(s.clusters >= 2 && s.splits == s.clusters - 1);
        let mut on = w.clone();
        on.config.stop_when_stable = false;
        let before = on.opinions().to_vec();
        on.run(5);
        assert_eq!(on.tick, w.tick + 5, "without the stop it keeps stepping");
        for (a, b) in on.opinions().iter().zip(&before) {
            assert!((a - b).abs() <= STILL);
        }
    }

    #[test]
    fn a_live_edit_after_stability_resumes_the_run() {
        let mut w = world(|c| {
            c.agents = 50;
            c.start = Start::Regular;
            c.epsilon = 0.2;
        });
        w.run(1000);
        assert!(w.is_finished());
        let stable_tick = w.tick;
        assert_eq!(stable_tick, 8);

        // Changing only stop_when_stable must not clear stable_at.
        let mut same = w.clone();
        let same_config = OpinionsConfig {
            stop_when_stable: false,
            ..same.config.clone()
        };
        Model::set_config(&mut same, ModelConfig::Opinions(same_config)).unwrap();
        assert_eq!(same.stats.latest().unwrap().stable_at, stable_tick);

        // A live edit to epsilon resumes a stopped run.
        let wider = OpinionsConfig {
            epsilon: 0.5,
            ..w.config.clone()
        };
        Model::set_config(&mut w, ModelConfig::Opinions(wider)).unwrap();
        assert!(!w.is_finished(), "the edit unsettles the run");
        w.run(1000);
        assert!(w.is_finished());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.clusters, 1, "wider confidence pulls everyone together");
        assert!(s.stable_at > stable_tick);
    }

    #[test]
    fn the_diagram_draws_lines_bands_and_the_lattice() {
        let mut w = world(|c| {
            c.agents = 50;
            c.start = Start::Regular;
            c.epsilon = 0.2;
            c.stop_when_stable = false;
        });
        let mut buf = Vec::new();
        w.render("start", "", &mut buf).unwrap();
        assert_eq!(buf.len(), WIDE * TALL * 4);
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * WIDE + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        assert_eq!(px(&buf, 0, TALL - 1), hue(0.0), "agent 1 starts at 0");
        assert_eq!(px(&buf, 0, 0), hue(1.0), "agent 50 at 1");
        w.run(70);
        w.render("opinion", "", &mut buf).unwrap();
        assert!(w.history.len() == HISTORY);
        assert!(w.render("wealth", "", &mut buf).is_err());
        let l = world(|c| c.interaction = Interaction::Lattice);
        assert_eq!(Model::size(&l), ((LATTICE_X + TALL) as u32, TALL as u32));
        l.render("start", "", &mut buf).unwrap();
        let k = LATTICE_X * 4;
        assert_eq!(buf[k..k + 3], hue(l.opinions()[0]));
    }

    #[test]
    fn inspect_finds_lines_and_sites() {
        let mut w = world(|c| {
            c.agents = 5;
            c.start = Start::Regular;
            c.epsilon = 0.3;
        });
        w.run(2);
        let v = w.inspect(0, (TALL - 1) as u32).unwrap();
        assert_eq!((v.period, v.opinion), (Some(0), Some(0.0)));
        assert_eq!(v.agents.len(), 1);
        assert_eq!((v.agents[0].id, v.agents[0].start), (1, 0.0));
        let latest = w.inspect((2 * STEP) as u32, 100).unwrap();
        assert_eq!(latest.period, Some(2));
        assert!(latest.agents.iter().any(|a| a.id == 3));
        assert!(w.inspect(WIDE as u32, 0).is_err(), "no lattice, no panel");
        let l = world(|c| c.interaction = Interaction::Lattice);
        let v = l.inspect(LATTICE_X as u32 + 9, 1).unwrap();
        assert_eq!(v.lattice_site, Some(OpinionsCell { x: 1, y: 0 }));
        assert_eq!(v.agents[0].id, 2);
        assert!(
            l.inspect(WIDE as u32 + 1, 0).unwrap().agents.is_empty(),
            "the gap"
        );
        assert_eq!(Model::locate(&l, 1), None);
    }

    #[test]
    fn keyframes_restore_opinions_and_the_diagram() {
        let mut any = crate::model::ModelWorld::new(
            ModelConfig::Opinions(config(|c| c.stop_when_stable = false)),
            6,
        )
        .unwrap();
        any.model_mut().run(5);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("start", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("start", "", &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(any.model().series("clusters").unwrap().len(), 6);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.epsilon = 0.3;
            c.confidence = Confidence::OpinionDependent;
            c.updating = Updating::SerialRandom;
        });
        Model::set_config(&mut w, ModelConfig::Opinions(next.clone())).unwrap();
        w.step();
        let e = Model::set_config(
            &mut w,
            ModelConfig::Opinions(OpinionsConfig {
                agents: 624,
                ..next
            }),
        )
        .unwrap_err();
        assert_eq!(e[0].field, "agents");
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.agents = 2),
            config(|c| c.epsilon = 0.0),
            config(|c| c.epsilon = 1.0),
            config(|c| {
                c.confidence = Confidence::Asymmetric;
                c.epsilon_left = 0.0;
                c.epsilon_right = 0.0;
            }),
            config(|c| {
                c.confidence = Confidence::OpinionDependent;
                c.bias = 1.0;
                c.epsilon = 1.0;
            }),
            config(|c| {
                c.agents = 9;
                c.interaction = Interaction::Lattice;
                c.lattice.width = 3;
                c.lattice.height = 3;
                c.updating = Updating::SerialShuffled;
            }),
        ] {
            let mut w = OpinionsWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.mean_opinion), "{c:?}");
            assert!(s.clusters >= 1);
        }
    }
}
