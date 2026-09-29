//! The Balinese Water Temples world. Each tick is a month of rain, water
//! sharing, rice growth, pests and harvest on the Oos and Petanu (or on
//! Janssen's two nodes); at each year's end the subaks decide their plans:
//! copying their best neighbor (Lansing and Kremer), copying by network
//! distance or innovating (Janssen's eq. 4), or — adaptive subaks — planting
//! month by month when water and pests allow.

use std::collections::VecDeque;
use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{BaliConfig, Decision, Plans, Watershed};
use super::data::watershed;
use super::engine::{
    area_mean, crop_of, month, scenario, stage_of, Network, Params, State, MIN_PESTS,
};
use super::search::{groups, search};
use super::stats::{adjusted_rand, BaliSnapshot};
use super::two_node::{best, NodePlan, Nodes};
use super::view::{
    at, option_color, radius, scale, CELL_H, CELL_W, DAM, DRY, FALLOW, FOUR, GAP, HIGH, HYV, LINK,
    LOW, MAP_H, RIVER, SIX, STRIP_H, TALL, TEMPLES, VEG, WET, WIDE,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaliMode {
    Plan,
    Temple,
    Harvest,
    Pests,
    Water,
    Crop,
}

impl std::str::FromStr for BaliMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "plan" => Self::Plan,
            "temple" => Self::Temple,
            "harvest" => Self::Harvest,
            "pests" => Self::Pests,
            "water" => Self::Water,
            "crop" => Self::Crop,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BaliInspection {
    pub site: BaliCell,
    /// `map` or `strip`; null elsewhere.
    pub panel: Option<&'static str>,
    pub subak: Option<SubakView>,
    pub dam: Option<DamView>,
    /// Strip: the month (1–12) and that dam's water then.
    pub month: Option<u32>,
    pub stress: Option<f64>,
    /// Always null: cells are read where they are.
    pub agent: Option<SubakView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct BaliCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SubakView {
    pub id: u32,
    pub area: f64,
    pub masceti: u32,
    pub source: u32,
    pub ret: u32,
    pub plan: u32,
    pub start: u32,
    pub crop: u32,
    /// The last year's harvest, t/ha; pests now; the water its source dam met.
    pub harvest: f64,
    pub pests: f64,
    pub water: f64,
    pub neighbors: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DamView {
    pub id: u32,
    /// This month's inflow and demand, m³/day, and the share of demand met.
    pub inflow: f64,
    pub demand: f64,
    pub stress: f64,
}

#[derive(Clone)]
pub struct BaliWorld {
    pub config: BaliConfig,
    /// Months run.
    pub tick: u64,
    rng: SimRng,
    net: Network,
    state: State,
    /// Each subak's plan and start month.
    plans: Vec<(u8, u8)>,
    /// Adaptive subaks: months left of the crop they planted.
    grow_left: Vec<u8>,
    /// The last year's harvest per subak, t/ha.
    last: Vec<f64>,
    /// Each year's mean harvest.
    years: Vec<f64>,
    changing: u32,
    /// This year's rain scenario.
    scenario: usize,
    /// Distances in the pest and water networks (generalized imitation).
    distances: Option<Distances>,
    /// Each dam's water over the last 12 months.
    strip: VecDeque<Vec<f64>>,
    /// The two-node model, if that is the watershed.
    nodes: Option<Nodes>,
    network_match: f64,
    /// The last year's water shortfall per growing month and share of its
    /// potential harvest lost to pests (area-weighted).
    stress: f64,
    lost: f64,
    pub stats: Stats<BaliSnapshot>,
}

/// Hop counts between subaks in the pest and the water network.
type Distances = (Vec<Vec<u32>>, Vec<Vec<u32>>);

/// Breadth-first hop counts from every node in `adj` (u32::MAX if unreached).
fn hops(adj: &[Vec<usize>], from: usize) -> Vec<u32> {
    let mut d = vec![u32::MAX; adj.len()];
    let mut q = VecDeque::from([from]);
    d[from] = 0;
    while let Some(x) = q.pop_front() {
        for &y in &adj[x] {
            if d[y] == u32::MAX {
                d[y] = d[x] + 1;
                q.push_back(y);
            }
        }
    }
    d
}

/// Components of equal labels over undirected `pairs`.
fn patches(n: usize, pairs: &[(usize, usize)], same: impl Fn(usize, usize) -> bool) -> Vec<usize> {
    let mut adj = vec![Vec::new(); n];
    for &(a, b) in pairs {
        if same(a, b) {
            adj[a].push(b);
            adj[b].push(a);
        }
    }
    let mut label = vec![usize::MAX; n];
    let mut k = 0;
    for i in 0..n {
        if label[i] != usize::MAX {
            continue;
        }
        let mut st = vec![i];
        label[i] = k;
        while let Some(x) = st.pop() {
            for &y in &adj[x] {
                if label[y] == usize::MAX {
                    label[y] = k;
                    st.push(y);
                }
            }
        }
        k += 1;
    }
    label
}

impl BaliWorld {
    pub fn new(config: BaliConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let w = watershed();
        let n = w.subaks.len();
        let net = Network::new(&config, &mut rng);
        let random =
            |rng: &mut SimRng| (rng.gen_range(0..21u32) as u8, rng.gen_range(0..12u32) as u8);
        let plans: Vec<(u8, u8)> = match config.plans {
            Plans::Random => (0..n).map(|_| random(&mut rng)).collect(),
            Plans::Traditional => (0..n).map(|_| (6, rng.gen_range(0..12u32) as u8)).collect(),
            Plans::Hyv => (0..n).map(|_| (1, rng.gen_range(0..12u32) as u8)).collect(),
            Plans::Temples => {
                let per: Vec<(u8, u8)> = (0..14).map(|_| random(&mut rng)).collect();
                w.subaks
                    .iter()
                    .map(|s| per[s.masceti as usize - 1])
                    .collect()
            }
            Plans::Search => {
                let g = groups(config.level, &net);
                let k = g.iter().max().map_or(0, |m| m + 1);
                let start: Vec<(u8, u8)> = (0..k).map(|_| random(&mut rng)).collect();
                let rain = if config.rain == super::config::Rain::Random {
                    1
                } else {
                    scenario(config.rain, &mut rng)
                };
                let p = Params::of(&config, 1, rain);
                search(&net, &p, &g, start, &mut rng)
            }
        };
        let mut state = State::new(n, w.dams.len(), config.routing);
        for (i, &(plan, start)) in plans.iter().enumerate() {
            state.stage[i] = stage_of(w, plan, start, 0);
        }
        let nodes = (config.watershed == Watershed::TwoNode).then(|| {
            let periods = config.node_periods as usize;
            let (pair, _) = best(
                config.growth,
                config.dispersal,
                config.node_rain,
                periods,
                60,
            );
            Nodes::new(
                pair,
                config.growth,
                config.dispersal,
                config.node_rain,
                periods,
            )
        });
        let distances = (config.decision == Decision::Generalized).then(|| Self::distances(&net));
        let pairs = net.pairs();
        let components = patches(n, &pairs, |_, _| true);
        let temples: Vec<usize> = w.subaks.iter().map(|s| s.masceti as usize).collect();
        let network_match = adjusted_rand(&components, &temples);
        let mut world = BaliWorld {
            config,
            tick: 0,
            rng,
            net,
            state,
            plans,
            grow_left: vec![0; n],
            last: vec![0.0; n],
            years: Vec::new(),
            changing: 0,
            scenario: 1,
            distances,
            strip: VecDeque::new(),
            nodes,
            network_match,
            stress: f64::NAN,
            lost: f64::NAN,
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    /// Hop counts between subaks in the pest network (undirected) and the
    /// water network (subaks, their source and return dams, and the rivers).
    fn distances(net: &Network) -> Distances {
        let w = watershed();
        let n = w.subaks.len();
        let mut pest = vec![Vec::new(); n];
        for (a, b) in net.pairs() {
            pest[a].push(b);
            pest[b].push(a);
        }
        let mut water = vec![Vec::new(); n + w.dams.len()];
        for i in 0..n {
            for d in [net.source[i], net.ret[i]] {
                water[i].push(n + d);
                water[n + d].push(i);
            }
        }
        for (d, ups) in w.upstream.iter().enumerate() {
            for &u in ups {
                water[n + d].push(n + u);
                water[n + u].push(n + d);
            }
        }
        let p = (0..n).map(|i| hops(&pest, i)).collect();
        let wd = (0..n).map(|i| hops(&water, i)[..n].to_vec()).collect();
        (p, wd)
    }

    pub fn plans(&self) -> &[(u8, u8)] {
        &self.plans
    }

    pub fn last_harvest(&self) -> &[f64] {
        &self.last
    }

    /// Each completed year's mean harvest.
    pub fn yearly(&self) -> &[f64] {
        &self.years
    }

    pub fn nodes(&self) -> Option<&Nodes> {
        self.nodes.as_ref()
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at) * self.periods()
    }

    /// Ticks a year: 12 months (2 for two nodes with two periods).
    fn periods(&self) -> u64 {
        self.nodes.as_ref().map_or(12, |n| n.periods as u64)
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
        if let Some(nodes) = &mut self.nodes {
            self.tick += 1;
            if let Some(h) = nodes.step() {
                self.years.push(h);
            }
            self.record();
            return;
        }
        let w = watershed();
        let n = w.subaks.len();
        let m = (self.tick % 12) as usize;
        let year = self.tick / 12 + 1;
        if m == 0 {
            self.scenario = scenario(self.config.rain, &mut self.rng);
        }
        let p = Params::of(&self.config, year, self.scenario);
        let adaptive = self.config.decision == Decision::Adaptive;
        let next: Vec<u8>;
        if adaptive {
            // Plant a three-month crop when the source dam's water per
            // hectare it serves exceeds m_w and the neighborhood's pests are
            // below m_p (Janssen §5; the units are stated choices).
            for i in 0..n {
                if self.grow_left[i] > 0 {
                    continue;
                }
                let src = self.net.source[i];
                let water = self.state.inflow[src] / (self.net.served[src].max(1.0) * 1e4);
                let hood: Vec<f64> = std::iter::once(i)
                    .chain(self.net.inn[i].iter().copied())
                    .map(|k| self.state.pests[k])
                    .collect();
                let pests = hood.iter().sum::<f64>() / hood.len() as f64;
                if (self.tick == 0 || water > self.config.m_w) && pests < self.config.m_p {
                    self.grow_left[i] = 3;
                }
            }
            for i in 0..n {
                self.state.crop[i] = if self.grow_left[i] > 0 { 3 } else { 0 };
            }
            next = self
                .grow_left
                .iter()
                .map(|&g| if g > 1 { 3 } else { 0 })
                .collect();
        } else {
            for (i, &(plan, start)) in self.plans.iter().enumerate() {
                self.state.crop[i] = crop_of(w, plan, start, m);
            }
            next = self
                .plans
                .iter()
                .map(|&(plan, start)| crop_of(w, plan, start, m + 1))
                .collect();
        }
        month(&self.net, &p, m, &mut self.state, &next, &mut self.rng);
        if adaptive {
            for g in &mut self.grow_left {
                *g = g.saturating_sub(1);
            }
        }
        if self.strip.len() == 12 {
            self.strip.pop_front();
        }
        self.strip.push_back(self.state.wsd.clone());
        self.tick += 1;
        if m == 11 {
            self.end_year();
        }
        self.record();
    }

    fn end_year(&mut self) {
        self.last = self.state.harvest.clone();
        self.years.push(area_mean(&self.last));
        let w = watershed();
        let (mut short, mut grown, mut lost, mut kept) = (0.0, 0.0, 0.0, 0.0);
        for (i, s) in w.subaks.iter().enumerate() {
            short += self.state.short[i] * s.area;
            grown += f64::from(self.state.growing[i]) * s.area;
            lost += self.state.lost[i] * s.area;
            kept += self.state.harvest[i] * s.area;
        }
        self.stress = if grown > 0.0 { short / grown } else { f64::NAN };
        self.lost = if lost + kept > 0.0 {
            lost / (lost + kept)
        } else {
            f64::NAN
        };
        self.changing = match self.config.decision {
            Decision::Imitate => self.imitate(),
            Decision::Generalized => self.generalize(),
            Decision::Adaptive | Decision::Fixed => 0,
        };
        for v in [
            &mut self.state.harvest,
            &mut self.state.lost,
            &mut self.state.short,
        ] {
            v.iter_mut().for_each(|x| *x = 0.0);
        }
        self.state.growing.iter_mut().for_each(|x| *x = 0);
        if self.config.pest_reset {
            self.state.pests.iter_mut().for_each(|x| *x = MIN_PESTS);
        }
    }

    /// Lansing and Kremer: every subak copies the plan of its best
    /// out-neighbor if that one's harvest was strictly higher, all at once.
    fn imitate(&mut self) -> u32 {
        let old = self.plans.clone();
        let mut changed = 0;
        for i in 0..old.len() {
            let mut best = (self.last[i], None);
            for &k in &self.net.out[i] {
                if self.last[k] > best.0 {
                    best = (self.last[k], Some(k));
                }
            }
            if let Some(k) = best.1 {
                if old[k] != old[i] {
                    changed += 1;
                }
                self.plans[i] = old[k];
            }
        }
        changed
    }

    /// Janssen's eq. 4: copy the best subak j with Hᵢ < Hⱼ / (1 + min(γp χp²,
    /// γw χw²)); failing that, below the mean harvest, innovate with
    /// probability ρ (a stated order).
    fn generalize(&mut self) -> u32 {
        let (pest, water) = self
            .distances
            .clone()
            .unwrap_or_else(|| Self::distances(&self.net));
        let old = self.plans.clone();
        let mean = area_mean(&self.last);
        let (gp, gw) = (self.config.gamma_p, self.config.gamma_w);
        let mut changed = 0;
        for i in 0..old.len() {
            let mut best: Option<(f64, usize)> = None;
            for j in 0..old.len() {
                if j == i {
                    continue;
                }
                let cost = |g: f64, h: u32| {
                    if h == u32::MAX {
                        f64::INFINITY
                    } else {
                        g * f64::from(h) * f64::from(h)
                    }
                };
                let c = cost(gp, pest[i][j]).min(cost(gw, water[i][j]));
                if c.is_finite()
                    && self.last[i] < self.last[j] / (1.0 + c)
                    && best.is_none_or(|(h, _)| self.last[j] > h)
                {
                    best = Some((self.last[j], j));
                }
            }
            let next = if let Some((_, j)) = best {
                old[j]
            } else if self.last[i] < mean && self.rng.gen::<f64>() < self.config.innovation {
                (
                    self.rng.gen_range(0..21u32) as u8,
                    self.rng.gen_range(0..12u32) as u8,
                )
            } else {
                old[i]
            };
            if next != old[i] {
                changed += 1;
            }
            self.plans[i] = next;
        }
        changed
    }

    fn record(&mut self) {
        let w = watershed();
        let n = w.subaks.len();
        let done = self.years.len() as u64;
        let scored: Vec<f64> = self
            .years
            .iter()
            .skip(self.config.score_from as usize - 1)
            .copied()
            .collect();
        let scored = if scored.is_empty() {
            f64::NAN
        } else {
            scored.iter().sum::<f64>() / scored.len() as f64
        };
        let latest = self.years.last().copied().unwrap_or(f64::NAN);
        if self.nodes.is_some() {
            self.stats.push(BaliSnapshot {
                tick: self.tick,
                harvest: latest,
                spread: f64::NAN,
                scored,
                changing: 0,
                water_stress: f64::NAN,
                pest_loss: f64::NAN,
                patches: f64::NAN,
                strategies: f64::NAN,
                temple_match: f64::NAN,
                network_match: f64::NAN,
                year: done,
            });
            return;
        }
        let total: f64 = w.subaks.iter().map(|s| s.area).sum();
        let spread = (w
            .subaks
            .iter()
            .zip(&self.last)
            .map(|(s, h)| s.area * (h - latest) * (h - latest))
            .sum::<f64>()
            / total)
            .sqrt();
        let adaptive = self.config.decision == Decision::Adaptive;
        let (patch_count, strategies, temple_match) = if adaptive {
            (f64::NAN, f64::NAN, f64::NAN)
        } else {
            let label = patches(n, &self.net.pairs(), |a, b| self.plans[a] == self.plans[b]);
            let temples: Vec<usize> = w.subaks.iter().map(|s| s.masceti as usize).collect();
            let mut distinct = self.plans.clone();
            distinct.sort_unstable();
            distinct.dedup();
            (
                (label.iter().max().unwrap_or(&0) + 1) as f64,
                distinct.len() as f64,
                adjusted_rand(&label, &temples),
            )
        };
        self.stats.push(BaliSnapshot {
            tick: self.tick,
            harvest: latest,
            spread: if done == 0 { f64::NAN } else { spread },
            scored,
            changing: self.changing,
            water_stress: self.stress,
            pest_loss: self.lost,
            patches: patch_count,
            strategies,
            temple_match,
            network_match: self.network_match,
            year: done,
        });
    }

    fn view(&self, i: usize) -> SubakView {
        let w = watershed();
        let s = &w.subaks[i];
        SubakView {
            id: i as u32 + 1,
            area: s.area,
            masceti: s.masceti,
            source: self.net.source[i] as u32,
            ret: self.net.ret[i] as u32,
            plan: u32::from(self.plans[i].0),
            start: u32::from(self.plans[i].1),
            crop: u32::from(self.state.crop[i]),
            harvest: self.last[i],
            pests: self.state.pests[i],
            water: self.state.wsd[self.net.source[i]],
            neighbors: self.net.out[i].len() as u32,
        }
    }

    fn color(&self, mode: BaliMode, i: usize) -> [u8; 3] {
        let w = watershed();
        match mode {
            BaliMode::Plan => {
                if self.config.decision == Decision::Adaptive {
                    if self.grow_left[i] > 0 {
                        HYV
                    } else {
                        FALLOW
                    }
                } else {
                    option_color(self.plans[i].0, self.plans[i].1)
                }
            }
            BaliMode::Temple => TEMPLES[(w.subaks[i].masceti as usize - 1) % 14],
            BaliMode::Harvest => scale(self.last[i] / 30.0, LOW, HIGH),
            BaliMode::Pests => scale(self.state.pests[i], LOW, DRY),
            BaliMode::Water => scale(self.state.wsd[self.net.source[i]], DRY, WET),
            BaliMode::Crop => match self.state.crop[i] {
                1 => SIX,
                2 => FOUR,
                3 => HYV,
                4 => VEG,
                _ => FALLOW,
            },
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<BaliInspection, String> {
        if x as usize >= WIDE || y as usize >= TALL {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let mut out = BaliInspection {
            site: BaliCell { x, y },
            panel: None,
            subak: None,
            dam: None,
            month: None,
            stress: None,
            agent: None,
        };
        if self.nodes.is_some() {
            return Ok(out);
        }
        let w = watershed();
        let (px, py) = (f64::from(x), f64::from(y));
        if (y as usize) < MAP_H {
            out.panel = Some("map");
            let near = |sx: f64, sy: f64, r: f64| {
                let (cx, cy) = at(sx, sy);
                (cx - px).powi(2) + (cy - py).powi(2) <= r * r
            };
            if let Some(j) = (0..w.dams.len()).find(|&j| near(w.dams[j].x, w.dams[j].y, 5.0)) {
                out.dam = Some(DamView {
                    id: j as u32,
                    inflow: self.state.inflow[j],
                    demand: self.state.demand[j],
                    stress: self.state.wsd[j],
                });
            } else if let Some(i) = (0..w.subaks.len())
                .find(|&i| near(w.subaks[i].x, w.subaks[i].y, radius(w.subaks[i].area) + 1.0))
            {
                out.subak = Some(self.view(i));
            }
        } else if (y as usize) >= MAP_H + GAP {
            let (col, dam) = (x as usize / CELL_W, (y as usize - MAP_H - GAP) / CELL_H);
            if let Some(v) = self.strip.get(col) {
                if dam < v.len() {
                    out.panel = Some("strip");
                    out.month =
                        Some(((self.tick as usize + 12 - self.strip.len() + col) % 12) as u32 + 1);
                    out.stress = Some(v[dam]);
                    out.dam = Some(DamView {
                        id: dam as u32,
                        inflow: self.state.inflow[dam],
                        demand: self.state.demand[dam],
                        stress: v[dam],
                    });
                }
            }
        }
        Ok(out)
    }
}

/// A line of pixels (Bresenham).
fn line(c: &mut Canvas, a: (f64, f64), b: (f64, f64), color: [u8; 3]) {
    let (mut x0, mut y0, x1, y1) = (a.0 as i64, a.1 as i64, b.0 as i64, b.1 as i64);
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    loop {
        if (0..WIDE as i64).contains(&x0) && (0..MAP_H as i64).contains(&y0) {
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

fn disc(c: &mut Canvas, (cx, cy): (f64, f64), r: f64, color: [u8; 3]) {
    let ri = r.ceil() as i64;
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            if (dx * dx + dy * dy) as f64 <= r * r {
                let (x, y) = (cx as i64 + dx, cy as i64 + dy);
                if (0..WIDE as i64).contains(&x) && (0..MAP_H as i64).contains(&y) {
                    c.put(x as usize, y as usize, color);
                }
            }
        }
    }
}

impl Model for BaliWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Bali(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        BaliWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        if self.nodes.is_some() {
            2
        } else {
            self.plans.len()
        }
    }

    /// FNV-1a over the tick, the plans and every subak's and node's state.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        for (i, &(p, s)) in self.plans.iter().enumerate() {
            eat(&[p, s, self.grow_left[i], self.state.crop[i]]);
            eat(&self.state.pests[i].to_bits().to_le_bytes());
            eat(&self.state.stage[i].to_bits().to_le_bytes());
            eat(&self.last[i].to_bits().to_le_bytes());
        }
        for x in &self.state.wsd {
            eat(&x.to_bits().to_le_bytes());
        }
        if let Some(nodes) = &self.nodes {
            for k in 0..2 {
                eat(&[nodes.pair[k].pattern, nodes.pair[k].start]);
                eat(&nodes.pests[k].to_bits().to_le_bytes());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (WIDE as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: BaliMode = mode.parse()?;
        let mut c = Canvas { buf, wide: 0 };
        c.clear(WIDE, TALL);
        let w = watershed();
        if let Some(nodes) = &self.nodes {
            // Two nodes: upstream above, downstream below, colored by pests.
            for k in 0..2 {
                let color = match mode {
                    BaliMode::Crop | BaliMode::Plan => {
                        if nodes.growing(k) {
                            HYV
                        } else {
                            FALLOW
                        }
                    }
                    _ => scale(nodes.pests[k], LOW, DRY),
                };
                disc(
                    &mut c,
                    (WIDE as f64 / 2.0, MAP_H as f64 * (0.3 + 0.4 * k as f64)),
                    40.0,
                    color,
                );
            }
            line(
                &mut c,
                (WIDE as f64 / 2.0, MAP_H as f64 * 0.3 + 40.0),
                (WIDE as f64 / 2.0, MAP_H as f64 * 0.7 - 40.0),
                RIVER,
            );
            return Ok(());
        }
        // The rivers, then the pest links, then the subaks and dams.
        for (d, ups) in w.upstream.iter().enumerate() {
            for &u in ups {
                line(
                    &mut c,
                    at(w.dams[u].x, w.dams[u].y),
                    at(w.dams[d].x, w.dams[d].y),
                    RIVER,
                );
            }
        }
        for (a, b) in self.net.pairs() {
            line(
                &mut c,
                at(w.subaks[a].x, w.subaks[a].y),
                at(w.subaks[b].x, w.subaks[b].y),
                LINK,
            );
        }
        for i in 0..w.subaks.len() {
            let s = &w.subaks[i];
            disc(&mut c, at(s.x, s.y), radius(s.area), self.color(mode, i));
        }
        for d in &w.dams {
            let (x, y) = at(d.x, d.y);
            for dy in -3..=3i64 {
                for dx in -3..=3i64 {
                    let (px, py) = (x as i64 + dx, y as i64 + dy);
                    if (0..WIDE as i64).contains(&px) && (0..MAP_H as i64).contains(&py) {
                        c.put(px as usize, py as usize, DAM);
                    }
                }
            }
        }
        // The strip: each dam's water over the last months.
        for (col, v) in self.strip.iter().enumerate() {
            for (dam, &x) in v.iter().enumerate() {
                let color = scale(x, DRY, WET);
                for dy in 0..CELL_H - 1 {
                    for dx in 0..CELL_W - 1 {
                        c.put(col * CELL_W + dx, MAP_H + GAP + dam * CELL_H + dy, color);
                    }
                }
            }
        }
        let _ = STRIP_H;
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
        let mut out = String::from(
            "id,area,masceti,source,return,plan,start,crop,harvest,pests,water,neighbors\n",
        );
        if self.nodes.is_some() {
            return out;
        }
        for i in 0..self.plans.len() {
            let v = self.view(i);
            writeln!(
                out,
                "{},{},{},{},{},{},{},{},{},{},{},{}",
                v.id,
                v.area,
                v.masceti,
                v.source,
                v.ret,
                v.plan,
                v.start,
                v.crop,
                v.harvest,
                v.pests,
                v.water,
                v.neighbors
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Subaks stay where they are.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let i = usize::try_from(id.checked_sub(1)?).ok()?;
        let w = watershed();
        if self.nodes.is_some() || i >= w.subaks.len() {
            return None;
        }
        let (x, y) = at(w.subaks[i].x, w.subaks[i].y);
        Some((x as u32, y as u32))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Bali(next) = next else {
            return Err(wrong_model(ModelKind::Bali, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        // Two nodes run the best pair for their pests: new pests, a new pair
        // (the nodes' pests and the year's harvest so far carry on).
        if let Some(nodes) = &mut self.nodes {
            if (next.growth, next.dispersal) != (nodes.g, nodes.d) {
                nodes.g = next.growth;
                nodes.d = next.dispersal;
                nodes.pair = best(next.growth, next.dispersal, nodes.rain, nodes.periods, 60).0;
            }
        }
        if next.decision == Decision::Generalized && self.distances.is_none() {
            self.distances = Some(Self::distances(&self.net));
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped after its last year: a sweep reads it there.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

impl NodePlan {
    /// A plan's label.
    pub fn describe(&self) -> String {
        format!("pattern {} from month {}", self.pattern, self.start + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bali::config::{Perturb, Rain, Routing};

    fn world(edit: impl FnOnce(&mut BaliConfig)) -> BaliWorld {
        let mut c = BaliConfig::default();
        edit(&mut c);
        BaliWorld::new(c, 1).unwrap()
    }

    #[test]
    fn imitation_raises_yields_and_settles() {
        let mut w = world(|c| c.stop_at = 15);
        w.run(10_000);
        let y = w.yearly();
        assert_eq!(y.len(), 15);
        assert!(y[14] > y[0] + 5.0, "{:?}", y);
        let s = w.stats.latest().unwrap();
        assert!(s.changing < 30, "{}", s.changing);
        assert!((0.0..=1.0).contains(&s.temple_match.abs()));
    }

    #[test]
    fn imitation_copies_only_a_strictly_better_out_neighbor() {
        let mut w = world(|_| {});
        let i = (0..172).find(|&i| w.net.out[i].len() >= 2).unwrap();
        let (a, b) = (w.net.out[i][0], w.net.out[i][1]);
        w.last = vec![0.0; 172];
        w.last[i] = 10.0;
        w.last[a] = 10.0;
        w.last[b] = 12.0;
        w.plans[b] = (5, 7);
        let before = w.plans[i];
        w.imitate();
        assert_eq!(w.plans[i], (5, 7));
        w.plans[i] = before;
        w.last[b] = 10.0;
        let copy = w.plans.clone();
        w.imitate();
        assert_eq!(w.plans[i], copy[i], "a tie is not better");
    }

    #[test]
    fn generalized_imitation_discounts_distance() {
        let generalized = |gamma: f64| {
            world(|c| {
                c.decision = Decision::Generalized;
                c.innovation = 0.0;
                c.gamma_p = gamma;
                c.gamma_w = gamma;
            })
        };
        // A subak three or more hops from subak `i` in both networks.
        let mut w = generalized(1.0);
        let (pest, water) = w.distances.clone().unwrap();
        let far_from = |i: usize| {
            (0..172).find(|&j| pest[i][j] >= 3 && water[i][j] >= 3 && water[i][j] != u32::MAX)
        };
        let i = (0..172).find(|&i| far_from(i).is_some()).unwrap();
        let far = far_from(i).unwrap();
        let hops = pest[i][far].min(water[i][far]);
        // With γ 1 a harvest twice as high does not tempt from ≥ 3 hops
        // (it would need more than 1 + 9 times as much)...
        w.last = vec![10.0; 172];
        w.last[far] = 20.0;
        w.plans[far] = (3, 3);
        let before = w.plans[i];
        w.generalize();
        assert!(hops >= 3);
        assert_eq!(w.plans[i], before);
        // ...but with γ 0 every subak connected to it by either network
        // copies it (the Oos and Petanu share no dam).
        let mut g = generalized(0.0);
        g.last = vec![10.0; 172];
        g.last[far] = 20.0;
        g.plans[far] = (3, 3);
        g.generalize();
        assert_eq!(g.plans[i], (3, 3));
        let reached = (0..172)
            .filter(|&j| pest[far][j] != u32::MAX || water[far][j] != u32::MAX)
            .count();
        assert!(reached > 50 && reached < 172, "{reached}");
        assert_eq!(g.plans.iter().filter(|&&p| p == (3, 3)).count(), reached);
    }

    #[test]
    fn innovation_changes_plans_below_the_mean() {
        // γ 100: no one copies (harvests differ by less than 101×).
        let mut w = world(|c| {
            c.decision = Decision::Generalized;
            c.innovation = 1.0;
            c.gamma_p = 100.0;
            c.gamma_w = 100.0;
        });
        w.last = (0..172).map(|i| 100.0 + i as f64).collect();
        let before = w.plans.clone();
        w.generalize();
        let changed_low = (0..60).filter(|&i| w.plans[i] != before[i]).count();
        let changed_high = (120..172).filter(|&i| w.plans[i] != before[i]).count();
        assert!(changed_low > 40, "{changed_low}");
        assert_eq!(changed_high, 0);
    }

    #[test]
    fn adaptive_subaks_plant_when_water_and_pests_allow() {
        let mut w = world(|c| {
            c.decision = Decision::Adaptive;
            c.stop_at = 3;
        });
        w.run(1000);
        assert!(w.yearly().iter().all(|&h| h > 0.0), "{:?}", w.yearly());
        let mut none = world(|c| {
            c.decision = Decision::Adaptive;
            c.m_p = 0.0;
            c.stop_at = 2;
        });
        none.run(1000);
        assert!(none.yearly().iter().all(|&h| h == 0.0));
    }

    #[test]
    fn a_years_water_stress_and_pest_loss_are_read_at_its_end() {
        let mut w = world(|c| c.stop_at = 2);
        assert!(w.stats.latest().unwrap().pest_loss.is_nan());
        w.run(12);
        let s = w.stats.latest().unwrap().clone();
        assert!((0.0..0.5).contains(&s.water_stress), "{}", s.water_stress);
        assert!(s.pest_loss > 0.0 && s.pest_loss < 1.0, "{}", s.pest_loss);
        w.run(1);
        assert_eq!(
            w.stats.latest().unwrap().pest_loss,
            s.pest_loss,
            "held until the next year ends"
        );
    }

    #[test]
    fn without_the_pest_reset_harvests_collapse() {
        let mut on = world(|c| c.stop_at = 6);
        let mut off = world(|c| {
            c.stop_at = 6;
            c.pest_reset = false;
        });
        on.run(1000);
        off.run(1000);
        assert!(
            off.yearly()[5] < on.yearly()[5] / 2.0,
            "{:?} {:?}",
            off.yearly(),
            on.yearly()
        );
    }

    #[test]
    fn the_perturbation_strikes_from_its_year() {
        let mut w = world(|c| {
            c.stop_at = 4;
            c.perturb = Perturb {
                enabled: true,
                at: 3,
                ..Perturb::default()
            };
        });
        w.run(1000);
        let y = w.yearly();
        assert!(y[2] < y[1], "{:?}", y);
    }

    #[test]
    fn plans_start_as_configured() {
        let t = world(|c| c.plans = Plans::Traditional);
        assert!(t.plans.iter().all(|p| p.0 == 6));
        let temples = world(|c| c.plans = Plans::Temples);
        let w = watershed();
        for i in 0..172 {
            for j in 0..172 {
                if w.subaks[i].masceti == w.subaks[j].masceti {
                    assert_eq!(temples.plans[i], temples.plans[j]);
                }
            }
        }
        let s = world(|c| {
            c.plans = Plans::Search;
            c.level = 14;
            c.decision = Decision::Fixed;
        });
        let r = world(|c| {
            c.plans = Plans::Temples;
            c.decision = Decision::Fixed;
        });
        let score = |w: &BaliWorld| {
            area_mean(&super::super::engine::steady_year(
                &w.net,
                &Params::of(&w.config, 1, 1),
                &w.plans,
                &mut rng::seeded(1),
            ))
        };
        assert!(score(&s) >= score(&r));
    }

    #[test]
    fn two_nodes_run_their_best_pair() {
        let mut w = world(|c| {
            c.watershed = Watershed::TwoNode;
            c.growth = 2.0;
            c.stop_at = 5;
        });
        w.run(1000);
        assert_eq!(w.yearly().len(), 5);
        assert!(w.yearly()[4] > 5.0, "{:?}", w.yearly());
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn janssens_code_and_random_rain_run() {
        let mut w = world(|c| {
            c.routing = Routing::JanssenCode;
            c.rain = Rain::Random;
            c.stop_at = 3;
        });
        w.run(1000);
        assert_eq!(w.yearly().len(), 3);
    }

    #[test]
    fn the_view_and_inspect_read_subaks_dams_and_months() {
        let mut w = world(|_| {});
        w.run(14);
        let mut buf = Vec::new();
        for mode in ["plan", "temple", "harvest", "pests", "water", "crop"] {
            w.render(mode, "", &mut buf).unwrap();
        }
        assert!(w.render("wealth", "", &mut buf).is_err());
        let ws = watershed();
        let (x, y) = at(ws.subaks[5].x, ws.subaks[5].y);
        let i = w.inspect(x as u32, y as u32).unwrap();
        assert_eq!(
            (i.panel, i.subak.as_ref().map(|s| s.id)),
            (Some("map"), Some(6))
        );
        let (x, y) = at(ws.dams[0].x, ws.dams[0].y);
        assert_eq!(w.inspect(x as u32, y as u32).unwrap().dam.unwrap().id, 0);
        let s = w.inspect(5, (MAP_H + GAP + 1) as u32).unwrap();
        assert_eq!((s.panel, s.dam.unwrap().id), (Some("strip"), 0));
        assert_eq!(
            Model::locate(&w, 6),
            Some((x as u32, y as u32))
                .filter(|_| false)
                .or(Model::locate(&w, 6))
        );
        assert!(Model::locate(&w, 173).is_none());
    }

    #[test]
    fn two_nodes_take_live_edits_of_growth_dispersal_and_their_best_plans() {
        let mut w = world(|c| {
            c.watershed = Watershed::TwoNode;
            c.growth = 2.0;
        });
        let mut next = w.config.clone();
        next.growth = 2.8;
        next.dispersal = 1.0;
        Model::set_config(&mut w, ModelConfig::Bali(next)).unwrap();
        let n = w.nodes().unwrap();
        assert_eq!((n.g, n.d), (2.8, 1.0));
        assert_eq!(
            n.pair,
            best(2.8, 1.0, 2.0, 12, 60).0,
            "re-planned for the new pests"
        );
    }

    #[test]
    fn live_edits_apply_and_the_network_waits_for_reset() {
        let mut w = world(|_| {});
        let mut next = w.config.clone();
        next.growth = 2.4;
        next.decision = Decision::Generalized;
        Model::set_config(&mut w, ModelConfig::Bali(next.clone())).unwrap();
        assert!(w.distances.is_some());
        next.plans = Plans::Hyv;
        assert!(Model::set_config(&mut w, ModelConfig::Bali(next)).is_err());
    }
}
