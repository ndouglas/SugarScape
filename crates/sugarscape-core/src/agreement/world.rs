//! The relative agreement world: random pairs meet, N meetings a period,
//! and move each other's opinions and uncertainties by relative agreement
//! (DAWF eqs. 1–6) or one of the bounded-confidence rules of §6.

use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{AgreementConfig, PairUpdate, Pairing, Placement, Rule, Window};
use super::stats::{grouping, AgreementSnapshot, Outcome, STILL};
use super::view::{
    opinion_at, opinion_color, row, scatter_col, uncertainty_color, COLUMNS, DIAGONAL, KEPT, MINUS,
    MODERATE_HIGH, MODERATE_LOW, PLUS, SCATTER_X, TALL, TORUS_X,
};
use crate::config::FieldError;
use crate::export;
use crate::graph::Graph;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::{site_cells, Canvas};
use crate::render::{lerp, Rgb};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Where an agent started: an extremist on either side, or a moderate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Plus,
    Minus,
    Moderate,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgreementMode {
    /// Confident red to uncertain green (DAWF's figures).
    Uncertainty,
    /// The initial extremists by side; moderates by opinion.
    Role,
    /// By starting opinion.
    Start,
}

impl std::str::FromStr for AgreementMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "uncertainty" => Self::Uncertainty,
            "role" => Self::Role,
            "start" => Self::Start,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgreementInspection {
    pub site: AgreementCell,
    /// `diagram`, `scatter` or `torus`; null between panels.
    pub panel: Option<&'static str>,
    /// The period of the diagram column (the current one on the panels).
    pub period: Option<u64>,
    /// The opinion at the clicked row, null on the torus.
    pub opinion: Option<f64>,
    /// The agents within one cell, or the torus site's agent.
    pub agents: Vec<AgreementAgent>,
    /// Always null: a clicked cell is read again where it is each period
    /// (a line or dot moves, and a tracked agent's dot is usually crowded).
    pub agent: Option<AgreementAgent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AgreementCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgreementAgent {
    pub id: u64,
    pub role: Role,
    pub start: f64,
    /// The opinion and uncertainty in the inspected period.
    pub opinion: f64,
    pub uncertainty: f64,
    /// Neighbors (N − 1 when anyone meets anyone).
    pub degree: u32,
    /// Meetings so far, and how many changed its opinion.
    pub meetings: u32,
    pub moves: u32,
}

/// One kept period of the diagram.
#[derive(Clone, Debug)]
struct Column {
    period: u64,
    x: Arc<Vec<f64>>,
    u: Arc<Vec<f64>>,
}

/// Agent j's opinion and uncertainty after agent i's influence, or `None`
/// when i has none on j.
pub fn influence(
    c: &AgreementConfig,
    (xi, ui): (f64, f64),
    (xj, uj): (f64, f64),
) -> Option<(f64, f64)> {
    let mu = c.mu;
    let window = || match c.window {
        Window::Influencer => ui,
        Window::Listener => uj,
    };
    match c.rule {
        Rule::Ra => {
            let h = (xi + ui).min(xj + uj) - (xi - ui).max(xj - uj);
            (h > ui).then(|| {
                let ra = h / ui - 1.0;
                (xj + mu * ra * (xi - xj), uj + mu * ra * (ui - uj))
            })
        }
        Rule::Bc => ((xi - xj).abs() < window()).then_some((xj + mu * (xi - xj), uj)),
        Rule::BcAveraging => {
            ((xi - xj).abs() < window()).then_some((xj + mu * (xi - xj), uj + mu * (ui - uj)))
        }
        Rule::BcVariance => ((xi - xj).abs() < window()).then(|| {
            let a = c.alpha;
            let d = xj - xi;
            (
                a * xj + (1.0 - a) * xi,
                (a * uj * uj + a * (1.0 - a) * d * d).sqrt(),
            )
        }),
    }
}

/// How many extremists start on each side: nₑ = round(N·pe),
/// n₊ = round(nₑ(1 + δ)/2) with a half broken by a coin flip, n₋ = nₑ − n₊.
pub fn extremist_counts(c: &AgreementConfig, n: usize, rng: &mut SimRng) -> (usize, usize) {
    let total = ((n as f64 * c.extremists).round() as usize).min(n);
    let half = total as f64 * (1.0 + c.delta) / 2.0;
    let plus = if (half - half.floor() - 0.5).abs() < 1e-9 {
        if rng.gen::<bool>() {
            half.ceil()
        } else {
            half.floor()
        }
    } else {
        half.round()
    } as usize;
    let plus = plus.min(total);
    (plus, total - plus)
}

#[derive(Clone)]
pub struct AgreementWorld {
    pub config: AgreementConfig,
    /// Completed periods.
    pub tick: u64,
    start: Vec<f64>,
    x: Vec<f64>,
    u: Vec<f64>,
    role: Vec<Role>,
    meetings: Vec<u32>,
    moves: Vec<u32>,
    graph: Arc<Graph>,
    rng: SimRng,
    /// The boundary a moderate must pass, less the margin, to count as a new
    /// extremist on each side; none on a side without extremists.
    plus_bound: Option<f64>,
    minus_bound: Option<f64>,
    /// Kept periods, oldest first: every `interval`-th, at most `KEPT`.
    history: Vec<Column>,
    interval: u64,
    max_change: f64,
    stable_at: Option<u64>,
    pub stats: Stats<AgreementSnapshot>,
}

impl AgreementWorld {
    pub fn new(config: AgreementConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let graph = Arc::new(Graph::new(&config, &mut rng));
        let n = config.population();
        let (plus, minus) = extremist_counts(&config, n, &mut rng);
        let mut x = vec![0.0f64; n];
        let mut role = vec![Role::Moderate; n];
        match config.placement {
            Placement::Drawn | Placement::Bounds => {
                for v in x.iter_mut() {
                    *v = rng.gen_range(-1.0..1.0);
                }
                let mut order: Vec<usize> = (0..n).collect();
                if config.placement == Placement::Bounds && plus + minus > 0 {
                    // AD draws moderates across the full opinion interval;
                    // choosing extremist roles must not remove its tails.
                    order.shuffle(&mut rng);
                } else {
                    order.sort_by(|&a, &b| x[a].total_cmp(&x[b]).then(a.cmp(&b)));
                }
                for &i in &order[..minus] {
                    role[i] = Role::Minus;
                }
                for &i in &order[n - plus..] {
                    role[i] = Role::Plus;
                }
                if config.placement == Placement::Bounds {
                    for i in 0..n {
                        match role[i] {
                            Role::Plus => x[i] = 1.0,
                            Role::Minus => x[i] = -1.0,
                            Role::Moderate => {}
                        }
                    }
                }
            }
            Placement::Band => {
                let b = config.band;
                let mut order: Vec<usize> = (0..n).collect();
                order.shuffle(&mut rng);
                for (k, &i) in order.iter().enumerate() {
                    if k < plus {
                        role[i] = Role::Plus;
                        x[i] = rng.gen_range(b..1.0);
                    } else if k < plus + minus {
                        role[i] = Role::Minus;
                        x[i] = -rng.gen_range(b..1.0);
                    } else {
                        x[i] = rng.gen_range(-b..b);
                    }
                }
            }
        }
        let u: Vec<f64> = role
            .iter()
            .map(|r| match r {
                Role::Moderate => config.uncertainty,
                _ => config.extremist_uncertainty,
            })
            .collect();
        let innermost = |side: Role, inner: fn(f64, f64) -> f64, bound: f64| {
            let mut it = (0..n).filter(|&i| role[i] == side).map(|i| x[i]);
            let first = it.next()?;
            Some(match config.placement {
                Placement::Drawn => it.fold(first, inner),
                Placement::Bounds => bound,
                Placement::Band => bound * config.band,
            })
        };
        let plus_bound = innermost(Role::Plus, f64::min, 1.0);
        let minus_bound = innermost(Role::Minus, f64::max, -1.0);
        let mut world = AgreementWorld {
            history: vec![Column {
                period: 0,
                x: Arc::new(x.clone()),
                u: Arc::new(u.clone()),
            }],
            config,
            tick: 0,
            start: x.clone(),
            x,
            u,
            role,
            meetings: vec![0; n],
            moves: vec![0; n],
            graph,
            rng,
            plus_bound,
            minus_bound,
            interval: 1,
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

    pub fn uncertainties(&self) -> &[f64] {
        &self.u
    }

    pub fn starts(&self) -> &[f64] {
        &self.start
    }

    pub fn roles(&self) -> &[Role] {
        &self.role
    }

    /// Whether the run has stopped: stable with `stop_when_stable`, or at `stop_at`.
    pub fn is_finished(&self) -> bool {
        (self.config.stop_when_stable && self.stable_at.is_some())
            || (self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at))
    }

    fn degree(&self, i: usize) -> u32 {
        if self.graph.is_complete() {
            self.x.len() as u32 - 1
        } else {
            self.graph.of(i).len() as u32
        }
    }

    /// The next meeting's pair, first agent first; `None` when the drawn
    /// agent has no neighbor.
    fn pair(&mut self) -> Option<(usize, usize)> {
        let n = self.x.len() as u32;
        let rng = &mut self.rng;
        if self.graph.is_complete() {
            let a = rng.gen_range(0..n);
            let mut b = rng.gen_range(0..n - 1);
            if b >= a {
                b += 1;
            }
            return Some((a as usize, b as usize));
        }
        match self.config.pairing {
            Pairing::Edge => {
                let edges = self.graph.edges();
                if edges.is_empty() {
                    return None;
                }
                let (p, q) = edges[rng.gen_range(0..edges.len() as u32) as usize];
                let (a, b) = if rng.gen::<bool>() { (p, q) } else { (q, p) };
                Some((a as usize, b as usize))
            }
            Pairing::Node => {
                let a = rng.gen_range(0..n) as usize;
                let ns = self.graph.of(a);
                if ns.is_empty() {
                    return None;
                }
                Some((a, ns[rng.gen_range(0..ns.len() as u32) as usize] as usize))
            }
        }
    }

    fn apply(&mut self, i: usize, next: Option<(f64, f64)>) {
        if let Some((nx, nu)) = next {
            let change = (nx - self.x[i]).abs().max((nu - self.u[i]).abs());
            self.max_change = self.max_change.max(change);
            if nx != self.x[i] {
                self.moves[i] += 1;
            }
            self.x[i] = nx;
            self.u[i] = nu;
        }
    }

    /// One meeting of `a` (first) and `b`.
    fn meet(&mut self, a: usize, b: usize) {
        self.meetings[a] += 1;
        self.meetings[b] += 1;
        let c = &self.config;
        let (pa, pb) = ((self.x[a], self.u[a]), (self.x[b], self.u[b]));
        match c.pair_update {
            PairUpdate::Simultaneous => {
                let (nb, na) = (influence(c, pa, pb), influence(c, pb, pa));
                self.apply(b, nb);
                self.apply(a, na);
            }
            PairUpdate::Sequential => {
                let nb = influence(c, pa, pb);
                self.apply(b, nb);
                let na = influence(&self.config, (self.x[b], self.u[b]), pa);
                self.apply(a, na);
            }
            PairUpdate::OneWay => {
                let na = influence(c, pb, pa);
                self.apply(a, na);
            }
        }
    }

    /// One period: N meetings.
    pub fn step(&mut self) {
        self.max_change = 0.0;
        for _ in 0..self.x.len() {
            if let Some((a, b)) = self.pair() {
                self.meet(a, b);
            }
        }
        self.tick += 1;
        if self.max_change > STILL {
            self.stable_at = None;
        }
        if self.stable_at.is_none() && self.max_change <= STILL {
            self.stable_at = Some(self.tick);
        }
        self.keep();
        self.record();
    }

    /// Keeps this period for the diagram if it falls on the interval,
    /// halving the history first when it is full.
    fn keep(&mut self) {
        if !self.tick.is_multiple_of(self.interval) {
            return;
        }
        if self.history.len() == KEPT {
            let every = 2 * self.interval;
            self.history.retain(|col| col.period % every == 0);
            self.interval = every;
            if !self.tick.is_multiple_of(self.interval) {
                return;
            }
        }
        self.history.push(Column {
            period: self.tick,
            x: Arc::new(self.x.clone()),
            u: Arc::new(self.u.clone()),
        });
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// Shares of the initial moderates past each side's boundary less the margin.
    fn new_extremists(&self) -> (f64, f64) {
        let moderates = self.role.iter().filter(|&&r| r == Role::Moderate).count();
        if moderates == 0 {
            return (0.0, 0.0);
        }
        let m = self.config.extreme_margin;
        let share = |past: &dyn Fn(f64) -> bool| {
            (0..self.x.len())
                .filter(|&i| self.role[i] == Role::Moderate && past(self.x[i]))
                .count() as f64
                / moderates as f64
        };
        (
            self.plus_bound.map_or(0.0, |b| share(&|x| x > b - m)),
            self.minus_bound.map_or(0.0, |b| share(&|x| x < b + m)),
        )
    }

    fn record(&mut self) {
        let n = self.x.len() as f64;
        let mut sorted = self.x.clone();
        sorted.sort_by(f64::total_cmp);
        let (clusters, major, isolated, largest, second, dispersion) = grouping(&sorted);
        let (p_plus, p_minus) = self.new_extremists();
        self.stats.push(AgreementSnapshot {
            tick: self.tick,
            y: p_plus * p_plus + p_minus * p_minus,
            p_plus,
            p_minus,
            outcome: Outcome::of(p_plus, p_minus) as u32,
            clusters,
            major,
            isolated,
            largest,
            second,
            dispersion,
            unmoved: self.moves.iter().filter(|&&m| m == 0).count() as f64 / n,
            mean_opinion: self.x.iter().sum::<f64>() / n,
            mean_uncertainty: self.u.iter().sum::<f64>() / n,
            max_change: self.max_change,
            stable_at: self.stable_at.unwrap_or(self.tick),
        });
    }

    /// The diagram's columns: the kept periods, then the current one if it
    /// was not kept.
    fn columns(&self) -> Vec<Column> {
        let mut cols = self.history.clone();
        if cols.last().map(|c| c.period) != Some(self.tick) {
            cols.push(Column {
                period: self.tick,
                x: Arc::new(self.x.clone()),
                u: Arc::new(self.u.clone()),
            });
        }
        cols
    }

    fn color(&self, mode: AgreementMode, i: usize, x: f64, u: f64) -> Rgb {
        let top = self
            .config
            .uncertainty
            .max(self.config.extremist_uncertainty);
        match mode {
            AgreementMode::Uncertainty => uncertainty_color(u, top),
            AgreementMode::Start => opinion_color(self.start[i]),
            AgreementMode::Role => match self.role[i] {
                Role::Plus => PLUS,
                Role::Minus => MINUS,
                Role::Moderate => lerp(MODERATE_LOW, MODERATE_HIGH, (x + 1.0) / 2.0),
            },
        }
    }

    fn view(&self, i: usize, x: f64, u: f64) -> AgreementAgent {
        AgreementAgent {
            id: i as u64 + 1,
            role: self.role[i],
            start: self.start[i],
            opinion: x,
            uncertainty: u,
            degree: self.degree(i),
            meetings: self.meetings[i],
            moves: self.moves[i],
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<AgreementInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let mut out = AgreementInspection {
            site: AgreementCell { x, y },
            panel: None,
            period: None,
            opinion: None,
            agents: Vec::new(),
            agent: None,
        };
        if cx < COLUMNS {
            let cols = self.columns();
            if let Some(col) = cols.get(cx) {
                out.panel = Some("diagram");
                out.period = Some(col.period);
                out.opinion = Some(opinion_at(cy));
                out.agents = (0..col.x.len())
                    .filter(|&i| row(col.x[i]).abs_diff(cy) <= 1)
                    .map(|i| self.view(i, col.x[i], col.u[i]))
                    .collect();
            }
        } else if (SCATTER_X..SCATTER_X + TALL).contains(&cx) {
            let sx = cx - SCATTER_X;
            out.panel = Some("scatter");
            out.period = Some(self.tick);
            out.opinion = Some(opinion_at(cy));
            out.agents = (0..self.x.len())
                .filter(|&i| {
                    scatter_col(self.start[i]).abs_diff(sx) <= 1 && row(self.x[i]).abs_diff(cy) <= 1
                })
                .map(|i| self.view(i, self.x[i], self.u[i]))
                .collect();
        } else if self.config.on_torus() && cx >= TORUS_X {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            let (sx, sy) = ((cx - TORUS_X) / s, cy / s);
            if sx < l.width as usize && sy < l.height as usize {
                let i = sy * l.width as usize + sx;
                out.panel = Some("torus");
                out.period = Some(self.tick);
                out.agents = vec![self.view(i, self.x[i], self.u[i])];
            }
        }
        Ok(out)
    }
}

impl Model for AgreementWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Agreement(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        AgreementWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.x.len()
    }

    /// FNV-1a over the tick and every opinion's and uncertainty's bits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: [u8; 8]| {
            for b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick.to_le_bytes());
        for (x, u) in self.x.iter().zip(&self.u) {
            eat(x.to_bits().to_le_bytes());
            eat(u.to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let w = if self.config.on_torus() {
            TORUS_X + TALL
        } else {
            SCATTER_X + TALL
        };
        (w as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: AgreementMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        // Extremists last, so they are drawn over the moderates.
        let mut order: Vec<usize> = (0..self.x.len()).collect();
        order.sort_by(|&a, &b| {
            (self.role[a] != Role::Moderate)
                .cmp(&(self.role[b] != Role::Moderate))
                .then(self.start[a].total_cmp(&self.start[b]))
        });
        let cols = self.columns();
        for &i in &order {
            for (k, col) in cols.iter().enumerate() {
                let color = self.color(mode, i, col.x[i], col.u[i]);
                match cols.get(k + 1) {
                    Some(next) => c.column(k, row(col.x[i]), row(next.x[i]), color),
                    None => c.put(k, row(col.x[i]), color),
                }
            }
        }
        for k in 0..TALL {
            c.put(SCATTER_X + k, TALL - 1 - k, DIAGONAL);
        }
        for &i in &order {
            let color = self.color(mode, i, self.x[i], self.u[i]);
            c.put(
                SCATTER_X + scatter_col(self.start[i]),
                row(self.x[i]),
                color,
            );
        }
        if self.config.on_torus() {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            for sy in 0..l.height as usize {
                for sx in 0..l.width as usize {
                    // By opinion now, whatever the mode: the lattice's state.
                    let color = opinion_color(self.x[sy * l.width as usize + sx]);
                    for dy in 0..s {
                        for dx in 0..s {
                            c.put(TORUS_X + sx * s + dx, sy * s + dy, color);
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
        let mut out = String::from("id,role,start,opinion,uncertainty,degree,meetings,moves\n");
        for i in 0..self.x.len() {
            let role = match self.role[i] {
                Role::Plus => "plus",
                Role::Minus => "minus",
                Role::Moderate => "moderate",
            };
            writeln!(
                out,
                "{},{role},{},{},{},{},{},{}",
                i + 1,
                self.start[i],
                self.x[i],
                self.u[i],
                self.degree(i),
                self.meetings[i],
                self.moves[i]
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Nothing to follow: Inspect reads a cell, not an agent.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Agreement(next) = next else {
            return Err(wrong_model(ModelKind::Agreement, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        // A live edit to how agents move can unsettle a stable run; resuming
        // it needs `stable_at` cleared now, since `run()` checks
        // `is_finished()` before stepping at all.
        if self.config.moves_differently(&next) {
            self.stable_at = None;
        }
        let recount = self.config.extreme_margin != next.extreme_margin;
        self.config = next;
        // y is a reading of the current opinions: a new margin recounts this
        // period's snapshot at once, so a stopped run shows it too.
        if recount {
            self.stats.truncate(self.stats.history().len() - 1);
            self.record();
        }
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stable: nothing moves again (until a live edit); stopped at
    /// `stop_at`: the run's reading is its last period.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agreement::config::{Network, Substrate};

    fn config(edit: impl FnOnce(&mut AgreementConfig)) -> AgreementConfig {
        let mut c = AgreementConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut AgreementConfig)) -> AgreementWorld {
        AgreementWorld::new(config(edit), 1).unwrap()
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12
    }

    #[test]
    fn relative_agreement_scales_by_overlap_over_the_influencers_uncertainty() {
        let c = AgreementConfig::default(); // μ 0.2
                                            // i at 0 (u 0.5), j at 0.4 (u 0.5): h = 0.5 − (−0.1) = 0.6 > 0.5,
                                            // RA = 0.6/0.5 − 1 = 0.2; xj moves by 0.2·0.2·(−0.4).
        let got = influence(&c, (0.0, 0.5), (0.4, 0.5)).unwrap();
        assert!(close(got, (0.4 - 0.016, 0.5)), "{got:?}");
        // At h = uᵢ there is no influence.
        assert_eq!(influence(&c, (0.0, 0.5), (0.5, 0.5)), None);
        // A confident agent moves an uncertain one; not the other way round.
        let confident = (0.9, 0.1);
        let unsure = (0.5, 1.0);
        assert!(influence(&c, confident, unsure).is_some());
        assert_eq!(influence(&c, unsure, confident), None, "h = 0.2 ≤ 1");
        // Uncertainty moves toward the influencer's: j's 1.0 toward 0.1.
        let (_, u) = influence(&c, confident, unsure).unwrap();
        assert!(u < 1.0);
    }

    #[test]
    fn bounded_confidence_windows_read_either_uncertainty() {
        let mut c = config(|c| c.rule = Rule::Bc);
        let extremist = (1.0, 0.1);
        let moderate = (0.5, 1.0);
        // Eq. 11 as printed: the influencer's u′ decides.
        assert_eq!(influence(&c, extremist, moderate), None, "0.5 ≥ 0.1");
        assert!(close(
            influence(&c, moderate, extremist).unwrap(),
            (0.9, 0.1)
        ));
        c.window = Window::Listener;
        assert!(close(
            influence(&c, extremist, moderate).unwrap(),
            (0.6, 1.0)
        ));
        assert_eq!(influence(&c, moderate, extremist), None);
        c.rule = Rule::BcAveraging;
        assert!(close(
            influence(&c, extremist, moderate).unwrap(),
            (0.6, 0.82)
        ));
        c.rule = Rule::BcVariance;
        let (x, u) = influence(&c, extremist, moderate).unwrap();
        // x = 0.8·0.5 + 0.2·1; u² = 0.8·1 + 0.8·0.2·0.25.
        assert!(close((x, u), (0.6, (0.8f64 + 0.04).sqrt())));
    }

    #[test]
    fn a_meeting_updates_both_one_after_the_other_or_one() {
        let pair = |pair_update| {
            let mut w = world(|c| {
                c.agents = 2;
                c.extremists = 0.0;
                c.uncertainty = 4.0; // wide enough that any two overlap
                c.mu = 0.5;
                c.pair_update = pair_update;
            });
            let before = (w.x.clone(), w.u.clone());
            w.meet(0, 1);
            (before, w.x.clone())
        };
        let ((x0, _), sim) = pair(PairUpdate::Simultaneous);
        let ((_, _), seq) = pair(PairUpdate::Sequential);
        let ((_, _), one) = pair(PairUpdate::OneWay);
        let c = config(|c| {
            c.uncertainty = 4.0;
            c.mu = 0.5;
        });
        let (a, b) = ((x0[0], 4.0), (x0[1], 4.0));
        assert_eq!(sim[1], influence(&c, a, b).unwrap().0);
        assert_eq!(sim[0], influence(&c, b, a).unwrap().0);
        let nb = influence(&c, a, b).unwrap();
        assert_eq!(seq[1], nb.0);
        assert_eq!(
            seq[0],
            influence(&c, nb, a).unwrap().0,
            "the first sees the new second"
        );
        assert_ne!(seq[0], sim[0]);
        assert_eq!(one[1], x0[1], "only the first moves");
        assert_eq!(one[0], sim[0]);
    }

    #[test]
    fn extremists_are_counted_placed_and_bounded() {
        let mut rng = rng::seeded(1);
        let c = config(|c| c.extremists = 0.1);
        assert_eq!(extremist_counts(&c, 200, &mut rng), (10, 10));
        let lean = config(|c| {
            c.extremists = 0.1;
            c.delta = 0.2;
        });
        assert_eq!(extremist_counts(&lean, 200, &mut rng), (12, 8));
        // 75 extremists at δ 0: 37 or 38 on the positive side, by a coin.
        let odd = config(|c| c.extremists = 0.075);
        let sides: Vec<usize> = (1..=20)
            .map(|s| extremist_counts(&odd, 1000, &mut rng::seeded(s)).0)
            .collect();
        assert!(sides.iter().all(|&p| p == 37 || p == 38));
        assert!(sides.contains(&37) && sides.contains(&38), "{sides:?}");

        let w = world(|_| {});
        let ext: Vec<usize> = (0..200).filter(|&i| w.role[i] != Role::Moderate).collect();
        assert_eq!(ext.len(), 20);
        let lowest_plus = (0..200)
            .filter(|&i| w.role[i] == Role::Plus)
            .map(|i| w.x[i])
            .fold(f64::INFINITY, f64::min);
        assert!((0..200)
            .filter(|&i| w.role[i] == Role::Moderate)
            .all(|i| w.x[i] < lowest_plus));
        assert_eq!(w.plus_bound, Some(lowest_plus));
        assert!(ext.iter().all(|&i| w.u[i] == 0.1));
        assert!((0..200)
            .filter(|&i| w.role[i] == Role::Moderate)
            .all(|i| w.u[i] == 1.0));

        let b = world(|c| c.placement = Placement::Bounds);
        assert!((0..200).all(|i| match b.role[i] {
            Role::Plus => b.x[i] == 1.0,
            Role::Minus => b.x[i] == -1.0,
            Role::Moderate => b.x[i].abs() < 1.0,
        }));
        assert_eq!((b.plus_bound, b.minus_bound), (Some(1.0), Some(-1.0)));

        let band = world(|c| c.placement = Placement::Band);
        assert!((0..200).all(|i| match band.role[i] {
            Role::Plus => band.x[i] >= 0.8,
            Role::Minus => band.x[i] <= -0.8,
            Role::Moderate => band.x[i].abs() < 0.8,
        }));
        assert_eq!((band.plus_bound, band.minus_bound), (Some(0.8), Some(-0.8)));

        let none = world(|c| c.extremists = 0.0);
        assert_eq!((none.plus_bound, none.minus_bound), (None, None));
        assert_eq!(none.stats.latest().unwrap().y, 0.0);
    }

    #[test]
    fn bounds_moderates_cover_the_full_uniform_opinion_range() {
        // Selecting extremists by opinion rank would remove both tails.
        let w = world(|c| {
            c.agents = 4000;
            c.extremists = 0.5;
            c.placement = Placement::Bounds;
        });
        let mut bins = [0; 10];
        for i in 0..w.x.len() {
            if w.role[i] == Role::Moderate {
                bins[((w.x[i] + 1.0) * 5.0) as usize] += 1;
            }
        }
        // 2000 independent uniform moderate draws: about 200 per tenth.
        assert!(bins.iter().all(|&n| (150..=250).contains(&n)), "{bins:?}");
    }

    #[test]
    fn bounds_without_extremists_keeps_the_drawn_population() {
        let drawn = world(|c| c.extremists = 0.0);
        let bounds = world(|c| {
            c.extremists = 0.0;
            c.placement = Placement::Bounds;
        });
        assert_eq!(bounds.x, drawn.x);
        assert_eq!((bounds.plus_bound, bounds.minus_bound), (None, None));
    }

    #[test]
    fn bounds_with_only_extremists_places_every_agent_at_a_bound() {
        let w = world(|c| {
            c.extremists = 1.0;
            c.delta = 1.0;
            c.placement = Placement::Bounds;
        });
        assert!(w.x.iter().all(|&x| x == 1.0));
        assert!(w.role.iter().all(|&r| r == Role::Plus));
        assert_eq!((w.plus_bound, w.minus_bound), (Some(1.0), None));
    }

    #[test]
    fn y_counts_moderates_past_the_boundary_less_the_margin() {
        let mut w = world(|c| {
            c.agents = 10;
            c.extremists = 0.2;
            c.placement = Placement::Bounds;
        });
        let moderates: Vec<usize> = (0..10).filter(|&i| w.role[i] == Role::Moderate).collect();
        assert_eq!(moderates.len(), 8);
        for (k, &i) in moderates.iter().enumerate() {
            w.x[i] = match k {
                0..=3 => 0.95, // past 1 − 0.1
                4 => 0.85,     // not past
                5 => -0.91,    // past −1 + 0.1
                _ => 0.0,
            };
        }
        let (p, m) = w.new_extremists();
        assert_eq!((p, m), (0.5, 0.125));
        w.config.extreme_margin = 0.2;
        assert_eq!(w.new_extremists(), (0.625, 0.125));
        w.record();
        let s = w.stats.latest().unwrap();
        assert!((s.y - (0.625f64.powi(2) + 0.125f64.powi(2))).abs() < 1e-12);
        assert_eq!(s.outcome, Outcome::Intermediate as u32);
    }

    #[test]
    fn pairs_are_distinct_or_follow_the_network() {
        let mut w = world(|_| {});
        for _ in 0..1000 {
            let (a, b) = w.pair().unwrap();
            assert_ne!(a, b);
        }
        let mut l = world(|c| c.network = Network::Lattice);
        for _ in 0..1000 {
            let (a, b) = l.pair().unwrap();
            assert!(l.graph.of(a).contains(&(b as u32)));
        }
        let mut s = world(|c| {
            c.network = Network::ScaleFree;
            c.pairing = Pairing::Node;
        });
        let mut firsts = vec![0u32; 200];
        for _ in 0..20_000 {
            let (a, b) = s.pair().unwrap();
            assert!(s.graph.of(a).contains(&(b as u32)));
            firsts[a] += 1;
        }
        let hub = (0..200).max_by_key(|&i| s.graph.of(i).len()).unwrap();
        assert!(
            firsts[hub] < 250,
            "node pairing picks the first agent uniformly, hubs included ({})",
            firsts[hub]
        );
    }

    #[test]
    fn a_run_stabilizes_and_stops_or_stops_at_its_period() {
        let mut w = world(|c| c.stop_at = 0);
        w.run(10_000);
        assert!(w.is_finished());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.stable_at, w.tick);
        assert!(s.max_change <= STILL);
        let mut capped = world(|c| {
            c.stop_when_stable = false;
            c.stop_at = 7;
        });
        capped.run(100);
        assert_eq!((capped.tick, capped.is_finished()), (7, true));
        let mut on = w.clone();
        on.config.stop_when_stable = false;
        on.config.stop_at = 0;
        on.run(3);
        assert_eq!(on.tick, w.tick + 3, "without a stop it keeps stepping");
    }

    #[test]
    fn a_live_edit_after_stability_resumes_the_run() {
        let mut w = world(|c| {
            c.extremists = 0.0;
            c.uncertainty = 0.2;
        });
        w.run(10_000);
        assert!(w.is_finished());
        let mut same = w.clone();
        let margin = AgreementConfig {
            extreme_margin: 0.3,
            ..same.config.clone()
        };
        Model::set_config(&mut same, ModelConfig::Agreement(margin)).unwrap();
        assert!(same.is_finished(), "the margin only changes what y counts");
        let faster = AgreementConfig {
            rule: Rule::Bc,
            window: Window::Listener,
            ..w.config.clone()
        };
        Model::set_config(&mut w, ModelConfig::Agreement(faster)).unwrap();
        assert!(!w.is_finished(), "a new rule unsettles the run");
    }

    #[test]
    fn a_margin_edit_recounts_a_stopped_run() {
        // Meadows and Cliff's reading, stopped at 200: the majority has
        // drifted but not past 0.8, so y is 0 until the cutoff moves in.
        let mut w = world(|c| {
            c.extremists = 0.05;
            c.uncertainty = 1.4;
            c.placement = Placement::Band;
            c.extreme_margin = 0.0;
            c.stop_when_stable = false;
            c.stop_at = 200;
        });
        w.run(1000);
        assert!(w.is_finished());
        let before = w.stats.latest().unwrap().clone();
        assert_eq!(before.y, 0.0);
        let wider = AgreementConfig {
            extreme_margin: 0.5,
            ..w.config.clone()
        };
        Model::set_config(&mut w, ModelConfig::Agreement(wider)).unwrap();
        let after = w.stats.latest().unwrap();
        assert!(after.y > 0.0, "y follows the new cutoff at once");
        assert_eq!(
            (after.tick, w.stats.history().len()),
            (200, 201),
            "recounted, not stepped"
        );
        assert_eq!(after.mean_opinion, before.mean_opinion);
    }

    #[test]
    fn the_history_halves_to_keep_every_run_in_view() {
        let mut w = world(|c| {
            c.agents = 20;
            c.stop_when_stable = false;
            c.stop_at = 0;
        });
        w.run(KEPT as u32 - 1);
        assert_eq!((w.history.len(), w.interval), (KEPT, 1));
        w.run(1);
        assert_eq!(w.interval, 2);
        assert_eq!(w.history.len(), KEPT / 2 + 1);
        assert!(w.history.iter().all(|c| c.period % 2 == 0));
        assert_eq!(w.history.last().unwrap().period, 240);
        w.run(1);
        assert_eq!(
            w.columns().len(),
            KEPT / 2 + 2,
            "the current period is drawn too"
        );
        w.run(1000);
        assert!(w.columns().len() <= COLUMNS);
        assert_eq!(w.columns().last().unwrap().period, w.tick);
        assert_eq!(w.history[0].period, 0);
    }

    #[test]
    fn the_frame_draws_the_diagram_the_scatter_and_the_torus() {
        let mut w = world(|c| c.stop_at = 0);
        let mut buf = Vec::new();
        w.render("uncertainty", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw as usize, fh as usize), (SCATTER_X + TALL, TALL));
        assert_eq!(buf.len(), (fw * fh * 4) as usize);
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * fw as usize + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        let top = (0..200).max_by(|&a, &b| w.x[a].total_cmp(&w.x[b])).unwrap();
        assert_eq!(
            px(&buf, 0, row(w.x[top])),
            uncertainty_color(0.1, 1.0),
            "the top line is an extremist's"
        );
        let bare = (0..TALL)
            .find(|&k| w.starts().iter().all(|&s| scatter_col(s) != k))
            .unwrap();
        assert_eq!(px(&buf, SCATTER_X + bare, TALL - 1 - bare), DIAGONAL);
        w.run(5);
        w.render("role", "", &mut buf).unwrap();
        w.render("start", "", &mut buf).unwrap();
        assert!(w.render("wealth", "", &mut buf).is_err());
        let t = world(|c| {
            c.network = Network::SmallWorld;
            c.small_world.substrate = Substrate::Grid;
            c.lattice.width = 10;
            c.lattice.height = 10;
        });
        assert_eq!(Model::size(&t), ((TORUS_X + TALL) as u32, TALL as u32));
        t.render("start", "", &mut buf).unwrap();
        let k = TORUS_X * 4;
        assert_eq!(buf[k..k + 3], opinion_color(t.starts()[0]));
    }

    #[test]
    fn inspect_finds_lines_dots_and_sites() {
        let mut w = world(|c| {
            c.agents = 5;
            c.extremists = 0.0;
            c.stop_when_stable = false;
            c.stop_at = 0;
        });
        w.run(3);
        let i = 2;
        let v = w.inspect(0, row(w.starts()[i]) as u32).unwrap();
        assert_eq!((v.panel, v.period), (Some("diagram"), Some(0)));
        assert!(v
            .agents
            .iter()
            .any(|a| a.id == 3 && a.opinion == w.starts()[i]));
        let now = w.inspect(3, row(w.x[i]) as u32).unwrap();
        assert_eq!(now.period, Some(3));
        let (sx, sy) = (SCATTER_X + scatter_col(w.starts()[i]), row(w.x[i]));
        let dot = w.inspect(sx as u32, sy as u32).unwrap();
        assert_eq!(dot.panel, Some("scatter"));
        assert!(dot.agents.iter().any(|a| a.id == 3 && a.degree == 4));
        assert!(dot.agent.is_none(), "a cell, not an agent");
        assert!(
            w.inspect(4, 0).unwrap().panel.is_none(),
            "past the last column"
        );
        assert!(
            w.inspect(COLUMNS as u32 + 1, 0).unwrap().panel.is_none(),
            "the gap"
        );
        assert_eq!(Model::locate(&w, 3), None);
        let l = world(|c| {
            c.network = Network::Lattice;
            c.lattice.width = 10;
            c.lattice.height = 10;
        });
        let s = site_cells(10, 10) as u32;
        let v = l.inspect(TORUS_X as u32 + s + 1, 1).unwrap();
        assert_eq!(v.panel, Some("torus"));
        assert_eq!((v.agents[0].id, v.agents[0].degree), (2, 4));
    }

    #[test]
    fn keyframes_restore_opinions_and_the_diagram() {
        let mut any =
            crate::model::ModelWorld::new(ModelConfig::Agreement(config(|c| c.stop_at = 0)), 6)
                .unwrap();
        any.model_mut().run(5);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("uncertainty", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("uncertainty", "", &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(any.model().series("y").unwrap().len(), 6);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.rule = Rule::BcVariance;
            c.mu = 0.4;
            c.pair_update = PairUpdate::OneWay;
            c.extreme_margin = 0.2;
        });
        Model::set_config(&mut w, ModelConfig::Agreement(next.clone())).unwrap();
        w.step();
        for (field, edit) in [
            ("agents", config(|c| c.agents = 201)),
            ("placement", config(|c| c.placement = Placement::Band)),
            ("network", config(|c| c.network = Network::Lattice)),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Agreement(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.agents = 2),
            config(|c| c.extremists = 0.0),
            config(|c| c.extremists = 1.0),
            config(|c| {
                c.agents = 3;
                c.extremists = 1.0;
                c.delta = 1.0;
            }),
            config(|c| c.uncertainty = 0.1),
            config(|c| c.mu = 0.0),
            config(|c| c.mu = 1.0),
            config(|c| {
                c.rule = Rule::BcVariance;
                c.alpha = 0.0;
            }),
            config(|c| c.placement = Placement::Band),
            config(|c| {
                c.network = Network::SmallWorld;
                c.small_world.degree = 2;
                c.small_world.rewire = 1.0;
            }),
            config(|c| {
                c.network = Network::ScaleFree;
                c.scale_free.links = 1;
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            }),
            config(|c| {
                c.network = Network::Lattice;
                c.lattice.width = 3;
                c.lattice.height = 3;
            }),
        ] {
            let mut w = AgreementWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            let s = w.stats.latest().unwrap();
            assert!((-1.0..=1.0).contains(&s.mean_opinion), "{c:?}");
            assert!((0.0..=2.0).contains(&s.y), "{c:?}");
            assert!(s.mean_uncertainty.is_finite(), "{c:?}");
        }
    }
}
