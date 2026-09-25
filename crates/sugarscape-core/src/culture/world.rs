//! The culture world: sites on a lattice, each with F features of q traits;
//! an event lets a site copy one differing feature from a neighbor with
//! probability equal to their similarity (Axelrod 1997, footnote 4).

use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{Activation, Changes, CultureConfig, Edges, Neighborhood};
use super::stats::{CultureSnapshot, Sets};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, NEUTRAL};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// A lane between sites that share everything but not identical: light gray.
const LIGHT: Rgb = [0xd8, 0xd4, 0xc8];
/// Identical neighbors in the Similarity mode (the paper's white).
const WHITE: Rgb = [0xf4, 0xf1, 0xe8];

/// The offsets of a neighborhood, row by row.
pub fn offsets(n: Neighborhood) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for dy in -2i32..=2 {
        for dx in -2i32..=2 {
            let (ax, ay) = (dx.abs(), dy.abs());
            let inside = match n {
                Neighborhood::VonNeumann => ax + ay == 1,
                Neighborhood::Moore => ax.max(ay) == 1,
                Neighborhood::Diamond => ax.max(ay) == 1 || (ax + ay == 2 && ax * ay == 0),
                Neighborhood::Soup => false,
            };
            if inside {
                out.push((dx, dy));
            }
        }
    }
    out
}

/// Every site's neighbors (not built for `soup`): site i's are
/// `list[start[i]..start[i + 1]]`, in offset order.
#[derive(Debug, PartialEq)]
pub struct Lattice {
    start: Vec<u32>,
    list: Vec<u32>,
}

impl Lattice {
    pub fn new(c: &CultureConfig) -> Self {
        let (w, h) = (c.width as i32, c.height as i32);
        let offs = offsets(c.neighborhood);
        let mut start = Vec::with_capacity(c.sites() + 1);
        let mut list = Vec::new();
        for y in 0..h {
            for x in 0..w {
                start.push(list.len() as u32);
                for &(dx, dy) in &offs {
                    let (mut nx, mut ny) = (x + dx, y + dy);
                    if c.boundary == Edges::Torus {
                        nx = nx.rem_euclid(w);
                        ny = ny.rem_euclid(h);
                    } else if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    list.push((ny * w + nx) as u32);
                }
            }
        }
        start.push(list.len() as u32);
        Lattice { start, list }
    }

    pub fn of(&self, i: usize) -> &[u32] {
        &self.list[self.start[i] as usize..self.start[i + 1] as usize]
    }

    /// Unordered neighboring pairs (i < j).
    pub fn pairs(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        (0..self.start.len() - 1).flat_map(move |i| {
            self.of(i)
                .iter()
                .filter(move |&&j| (j as usize) > i)
                .map(move |&j| (i, j as usize))
        })
    }
}

/// The diagram's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CultureMode {
    /// Each culture its color; lanes shaded by similarity.
    Culture,
    /// The paper's Fig. 1: only the lanes, shaded by similarity.
    Similarity,
    /// Each cultural zone its color; lanes black where nothing is shared.
    Zones,
}

impl std::str::FromStr for CultureMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "culture" => Self::Culture,
            "similarity" => Self::Similarity,
            "zones" => Self::Zones,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// A stable color for a key: a hue from its hash at fixed saturation and value.
pub fn hue_color(key: u64) -> Rgb {
    let h = (key % 360) as f64;
    let (s, v) = (0.6, 0.92);
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to = |v: f64| ((v + m) * 255.0).round() as u8;
    [to(r), to(g), to(b)]
}

fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// A culture's stable color (the Culture modes of both models).
pub fn culture_color(traits: &[u8]) -> Rgb {
    hue_color(fnv(traits))
}

fn luminance(c: Rgb) -> u32 {
    2 * u32::from(c[0]) + 5 * u32::from(c[1]) + u32::from(c[2])
}

/// What Inspect shows: a site (with each neighbor's shared features), or a
/// lane between two sites.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CultureInspection {
    pub site: CultureCell,
    /// `"site"` or `"lane"`.
    pub kind: &'static str,
    pub a: SiteView,
    /// The lane's other site.
    pub b: Option<SiteView>,
    /// Features the lane's two sites share.
    pub shared: Option<u32>,
    /// The site's neighbors and what each shares with it (empty for a lane
    /// and in `soup`).
    pub neighbors: Vec<NeighborView>,
    /// Always null: sites do not move.
    pub agent: Option<SiteView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CultureCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SiteView {
    pub x: u32,
    pub y: u32,
    pub traits: Vec<u8>,
    pub region_size: u32,
    pub zone_size: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct NeighborView {
    pub x: u32,
    pub y: u32,
    pub shared: u32,
}

#[derive(Clone)]
pub struct CultureWorld {
    pub config: CultureConfig,
    /// Completed ticks (N events each).
    pub tick: u64,
    /// Site i's traits are `traits[i·F..(i + 1)·F]`, sites row-major.
    traits: Vec<u8>,
    /// Neighbors (None in `soup`), shared by keyframes.
    lattice: Option<Arc<Lattice>>,
    rng: SimRng,
    /// Neighboring pairs sharing some but not all features (lattice only).
    active_bonds: u64,
    /// The sum of shared features over neighboring pairs (lattice only).
    shared_sum: u64,
    /// Traits changed this tick.
    changes: u64,
    /// Whether traits changed since the labels were computed.
    dirty: bool,
    stable_at: Option<u64>,
    region_of: Vec<u32>,
    region_sizes: Vec<u32>,
    zone_of: Vec<u32>,
    zone_sizes: Vec<u32>,
    cultures: u32,
    /// `soup`'s active bonds and mean similarity, from the last labelling.
    soup_bonds: u64,
    soup_similarity: f64,
    pub stats: Stats<CultureSnapshot>,
}

impl CultureWorld {
    pub fn new(config: CultureConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let q = config.traits;
        let traits = (0..config.sites() * config.features as usize)
            .map(|_| rng.gen_range(0..q) as u8)
            .collect();
        let lattice =
            (config.neighborhood != Neighborhood::Soup).then(|| Arc::new(Lattice::new(&config)));
        let mut world = CultureWorld {
            config,
            tick: 0,
            traits,
            lattice,
            rng,
            active_bonds: 0,
            shared_sum: 0,
            changes: 0,
            dirty: true,
            stable_at: None,
            region_of: Vec::new(),
            region_sizes: Vec::new(),
            zone_of: Vec::new(),
            zone_sizes: Vec::new(),
            cultures: 0,
            soup_bonds: 0,
            soup_similarity: 0.0,
            stats: Stats::default(),
        };
        world.count_bonds();
        world.record();
        Ok(world)
    }

    fn f(&self) -> usize {
        self.config.features as usize
    }

    /// Site i's traits.
    pub fn culture(&self, i: usize) -> &[u8] {
        let f = self.f();
        &self.traits[i * f..(i + 1) * f]
    }

    /// Features sites i and j share.
    pub fn shared(&self, i: usize, j: usize) -> u32 {
        self.culture(i)
            .iter()
            .zip(self.culture(j))
            .filter(|(a, b)| a == b)
            .count() as u32
    }

    fn count_bonds(&mut self) {
        let (mut active, mut sum) = (0, 0);
        if let Some(l) = self.lattice.clone() {
            let f = self.config.features;
            for (i, j) in l.pairs() {
                let s = self.shared(i, j);
                sum += u64::from(s);
                active += u64::from(s > 0 && s < f);
            }
        }
        self.active_bonds = active;
        self.shared_sum = sum;
    }

    /// Whether no pair can interact any more.
    pub fn is_stable(&self) -> bool {
        match self.lattice {
            Some(_) => self.active_bonds == 0,
            None => self.soup_bonds == 0 && !self.dirty,
        }
    }

    /// Whether the run has stopped: stable, stopping when stable, and no drift.
    pub fn is_finished(&self) -> bool {
        self.config.stop_when_stable && self.config.drift == 0.0 && self.stable_at.is_some()
    }

    /// Sets site i's feature g to trait t, keeping the bond counts.
    fn set(&mut self, i: usize, g: usize, t: u8) {
        let f = self.f();
        let old = self.traits[i * f + g];
        if old == t {
            return;
        }
        if let Some(l) = self.lattice.clone() {
            let full = self.config.features;
            for &j in l.of(i) {
                let other = self.traits[j as usize * f + g];
                let before = self.shared(i, j as usize);
                let after = before + u32::from(other == t) - u32::from(other == old);
                let active = |s: u32| u64::from(s > 0 && s < full);
                self.active_bonds = self.active_bonds + active(after) - active(before);
                self.shared_sum = self.shared_sum + u64::from(after) - u64::from(before);
            }
        }
        self.traits[i * f + g] = t;
        self.changes += 1;
        self.dirty = true;
    }

    /// One event with site s active.
    fn event(&mut self, s: usize) {
        let c = &self.config;
        let (f, q, drift, changes) = (self.f(), c.traits, c.drift, c.changes);
        if drift > 0.0 && self.rng.gen_bool(drift) {
            let g = self.rng.gen_range(0..f as u32) as usize;
            let t = self.rng.gen_range(0..q) as u8;
            self.set(s, g, t);
            return;
        }
        let n = match &self.lattice {
            Some(l) => {
                let nb = l.of(s);
                if nb.is_empty() {
                    return;
                }
                nb[self.rng.gen_range(0..nb.len() as u32) as usize] as usize
            }
            None => {
                let n = self.rng.gen_range(0..self.config.sites() as u32 - 1) as usize;
                if n >= s {
                    n + 1
                } else {
                    n
                }
            }
        };
        let probe = self.rng.gen_range(0..f as u32) as usize;
        if self.traits[s * f + probe] != self.traits[n * f + probe] {
            return;
        }
        let mut differ = [0u8; 32];
        let mut k = 0;
        for g in 0..f {
            if self.traits[s * f + g] != self.traits[n * f + g] {
                differ[k] = g as u8;
                k += 1;
            }
        }
        if k == 0 {
            return;
        }
        let g = differ[self.rng.gen_range(0..k as u32) as usize] as usize;
        let (to, from) = match changes {
            Changes::Active => (s, n),
            Changes::Neighbor => (n, s),
        };
        let t = self.traits[from * f + g];
        self.set(to, g, t);
    }

    /// One tick: N events.
    pub fn step(&mut self) {
        let n = self.config.sites();
        self.changes = 0;
        match self.config.activation {
            Activation::Random => {
                for _ in 0..n {
                    let s = self.rng.gen_range(0..n as u32) as usize;
                    self.event(s);
                }
            }
            Activation::Sweep => {
                let mut order: Vec<u32> = (0..n as u32).collect();
                order.shuffle(&mut self.rng);
                for s in order {
                    self.event(s as usize);
                }
            }
        }
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

    /// Recomputes regions, zones and cultures if traits changed.
    fn label(&mut self) {
        if !self.dirty {
            return;
        }
        let n = self.config.sites();
        let f = self.f();
        match self.lattice.clone() {
            Some(l) => {
                let (mut regions, mut zones) = (Sets::new(n), Sets::new(n));
                for (i, j) in l.pairs() {
                    if self.culture(i) == self.culture(j) {
                        regions.union(i as u32, j as u32);
                        zones.union(i as u32, j as u32);
                    } else if self.shared(i, j) > 0 {
                        zones.union(i as u32, j as u32);
                    }
                }
                (self.region_of, self.region_sizes) = regions.labels();
                (self.zone_of, self.zone_sizes) = zones.labels();
                // Distinct cultures: one representative per region (a region
                // is one culture), sorted.
                let mut first = vec![u32::MAX; self.region_sizes.len()];
                for (i, &r) in self.region_of.iter().enumerate() {
                    if first[r as usize] == u32::MAX {
                        first[r as usize] = i as u32;
                    }
                }
                first.sort_by(|&a, &b| self.culture(a as usize).cmp(self.culture(b as usize)));
                self.cultures = 1 + first
                    .windows(2)
                    .filter(|p| self.culture(p[0] as usize) != self.culture(p[1] as usize))
                    .count() as u32;
            }
            None => {
                // Soup: every two sites are neighbors, so a region is a culture
                // and a zone joins cultures sharing any (feature, trait).
                let mut order: Vec<u32> = (0..n as u32).collect();
                order.sort_by(|&a, &b| self.culture(a as usize).cmp(self.culture(b as usize)));
                let mut culture_of = vec![0u32; n];
                let mut k = 0u32;
                for (idx, &i) in order.iter().enumerate() {
                    if idx > 0 && self.culture(i as usize) != self.culture(order[idx - 1] as usize)
                    {
                        k += 1;
                    }
                    culture_of[i as usize] = k;
                }
                self.cultures = k + 1;
                let cultures = self.cultures as usize;
                let mut sizes = vec![0u32; cultures];
                for &c in &culture_of {
                    sizes[c as usize] += 1;
                }
                let mut zones = Sets::new(cultures);
                let q = self.config.traits as usize;
                let (mut bonds, mut pairs) = (0u64, 0u64);
                for g in 0..f {
                    let mut holder = vec![u32::MAX; q];
                    let mut holders = vec![0u32; q];
                    let mut sites = vec![0u64; q];
                    for (idx, &i) in order.iter().enumerate() {
                        let t = self.traits[i as usize * f + g] as usize;
                        sites[t] += 1;
                        let c = culture_of[i as usize];
                        let first_of_culture = idx == 0 || culture_of[order[idx - 1] as usize] != c;
                        if !first_of_culture {
                            continue;
                        }
                        holders[t] += 1;
                        if holder[t] == u32::MAX {
                            holder[t] = c;
                        } else {
                            zones.union(holder[t], c);
                        }
                    }
                    bonds += holders.iter().filter(|&&h| h >= 2).count() as u64;
                    pairs += sites
                        .iter()
                        .map(|&m| m * m.saturating_sub(1) / 2)
                        .sum::<u64>();
                }
                let (zone_of_culture, zone_counts) = zones.labels();
                let mut zone_sizes = vec![0u32; zone_counts.len()];
                self.zone_of = culture_of
                    .iter()
                    .map(|&c| zone_of_culture[c as usize])
                    .collect();
                for &z in &self.zone_of {
                    zone_sizes[z as usize] += 1;
                }
                self.zone_sizes = zone_sizes;
                self.region_of = culture_of;
                self.region_sizes = sizes;
                self.soup_bonds = bonds;
                let all = (n as u64 * (n as u64 - 1) / 2) as f64 * f as f64;
                self.soup_similarity = pairs as f64 / all;
            }
        }
        self.dirty = false;
    }

    /// Pushes this tick's statistics.
    fn record(&mut self) {
        self.label();
        let n = self.config.sites();
        let (bonds, similarity) = match &self.lattice {
            Some(l) => {
                let pairs = l.list.len() as f64 / 2.0;
                let mean = if pairs == 0.0 {
                    1.0
                } else {
                    self.shared_sum as f64 / (pairs * self.f() as f64)
                };
                (self.active_bonds, mean)
            }
            None => (self.soup_bonds, self.soup_similarity),
        };
        if self.stable_at.is_none() && self.is_stable() {
            self.stable_at = Some(self.tick);
        }
        if self.stable_at.is_some() && !self.is_stable() {
            // Drift (or a live change) broke the stability.
            self.stable_at = None;
        }
        self.stats.push(CultureSnapshot {
            tick: self.tick,
            regions: self.region_sizes.len() as u32,
            zones: self.zone_sizes.len() as u32,
            cultures: self.cultures,
            largest_region: f64::from(self.region_sizes.iter().copied().max().unwrap_or(0))
                / n as f64,
            mean_similarity: similarity,
            active_bonds: bonds,
            changes: self.changes,
            stable_at: self.stable_at.unwrap_or(self.tick),
        });
    }

    fn site_view(&self, i: usize) -> SiteView {
        let w = self.config.width as usize;
        SiteView {
            x: (i % w) as u32,
            y: (i / w) as u32,
            traits: self.culture(i).to_vec(),
            region_size: self.region_sizes[self.region_of[i] as usize],
            zone_size: self.zone_sizes[self.zone_of[i] as usize],
        }
    }

    /// The frame cell (x, y): a site's block, or the lane between two sites
    /// (a crossing shows its top-left site).
    pub fn inspect(&self, x: u32, y: u32) -> Result<CultureInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let w = self.config.width as usize;
        let (sx, sy) = ((x / 3) as usize, (y / 3) as usize);
        let i = sy * w + sx;
        let cell = CultureCell { x, y };
        let lane = match (x % 3 == 2, y % 3 == 2) {
            (true, false) => Some(i + 1),
            (false, true) => Some(i + w),
            _ => None,
        };
        if let Some(j) = lane {
            return Ok(CultureInspection {
                site: cell,
                kind: "lane",
                a: self.site_view(i),
                b: Some(self.site_view(j)),
                shared: Some(self.shared(i, j)),
                neighbors: Vec::new(),
                agent: None,
            });
        }
        let neighbors = match &self.lattice {
            Some(l) => l
                .of(i)
                .iter()
                .map(|&j| NeighborView {
                    x: j % w as u32,
                    y: j / w as u32,
                    shared: self.shared(i, j as usize),
                })
                .collect(),
            None => Vec::new(),
        };
        Ok(CultureInspection {
            site: cell,
            kind: "site",
            a: self.site_view(i),
            b: None,
            shared: None,
            neighbors,
            agent: None,
        })
    }

    fn site_color(&self, i: usize, mode: CultureMode) -> Rgb {
        match mode {
            CultureMode::Culture => culture_color(self.culture(i)),
            CultureMode::Similarity => NEUTRAL,
            CultureMode::Zones => hue_color(fnv(&self.zone_of[i].to_le_bytes()).rotate_left(17)),
        }
    }

    fn lane_color(&self, i: usize, j: usize, mode: CultureMode) -> Rgb {
        let f = self.config.features;
        let s = self.shared(i, j);
        let gray = lerp(BACKGROUND, LIGHT, f64::from(s) / f64::from(f));
        match mode {
            CultureMode::Culture if s == f => self.site_color(i, mode),
            CultureMode::Similarity if s == f => WHITE,
            CultureMode::Zones if s > 0 => self.site_color(i, mode),
            CultureMode::Zones => BACKGROUND,
            _ => gray,
        }
    }
}

impl Model for CultureWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Culture(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        CultureWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.config.sites()
    }

    /// FNV-1a over the tick, every trait and the tick the world became stable.
    fn fingerprint(&self) -> u64 {
        let mut h = fnv(&self.tick.to_le_bytes());
        for &b in &self.traits {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        for b in self.stable_at.map_or(u64::MAX, |t| t).to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    }

    /// Each site a 2 × 2 block with a one-cell lane between neighbors.
    fn size(&self) -> (u32, u32) {
        (3 * self.config.width - 1, 3 * self.config.height - 1)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: CultureMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let (w, h) = (self.config.width as usize, self.config.height as usize);
        buf.clear();
        buf.resize((fw * fh * 4) as usize, 0);
        let mut put = |x: usize, y: usize, c: Rgb| {
            let k = (y * fw as usize + x) * 4;
            buf[k..k + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        };
        for sy in 0..h {
            for sx in 0..w {
                let i = sy * w + sx;
                let c = self.site_color(i, mode);
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    put(3 * sx + dx, 3 * sy + dy, c);
                }
                let right = (sx + 1 < w).then(|| self.lane_color(i, i + 1, mode));
                let down = (sy + 1 < h).then(|| self.lane_color(i, i + w, mode));
                if let Some(c) = right {
                    put(3 * sx + 2, 3 * sy, c);
                    put(3 * sx + 2, 3 * sy + 1, c);
                }
                if let Some(c) = down {
                    put(3 * sx, 3 * sy + 2, c);
                    put(3 * sx + 1, 3 * sy + 2, c);
                }
                if let (Some(r), Some(d)) = (right, down) {
                    // The crossing takes the darkest of its four lanes.
                    let lanes = [
                        r,
                        d,
                        self.lane_color(i + 1, i + 1 + w, mode),
                        self.lane_color(i + w, i + w + 1, mode),
                    ];
                    let dark = *lanes.iter().min_by_key(|&&c| luminance(c)).unwrap();
                    put(3 * sx + 2, 3 * sy + 2, dark);
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

    /// One row per site: its position, region, zone and traits.
    fn agents_csv(&self) -> String {
        let mut out = String::from("x,y,region,zone,traits\n");
        let w = self.config.width as usize;
        for i in 0..self.config.sites() {
            let traits: Vec<String> = self.culture(i).iter().map(|t| t.to_string()).collect();
            writeln!(
                out,
                "{},{},{},{},{}",
                i % w,
                i / w,
                self.region_of[i],
                self.zone_of[i],
                traits.join(" ")
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Sites do not move; there is nothing to follow.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Culture(next) = next else {
            return Err(wrong_model(ModelKind::Culture, &next));
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

    /// Stable without drift: no trait can ever change again.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(edit: impl FnOnce(&mut CultureConfig)) -> CultureConfig {
        let mut c = CultureConfig::default();
        edit(&mut c);
        c
    }

    /// A world of `config` with its traits replaced by `cultures` (one
    /// culture per site, row-major) and its counts rebuilt.
    fn world(c: CultureConfig, cultures: &[&[u8]]) -> CultureWorld {
        let mut w = CultureWorld::new(c, 1).unwrap();
        w.traits = cultures.concat();
        w.count_bonds();
        w.dirty = true;
        w.stats = Stats::default();
        w.stable_at = None;
        w.record();
        w
    }

    #[test]
    fn neighborhoods_have_the_papers_4_8_and_12_sites() {
        let n = |k| offsets(k).len();
        assert_eq!(
            (
                n(Neighborhood::VonNeumann),
                n(Neighborhood::Moore),
                n(Neighborhood::Diamond)
            ),
            (4, 8, 12)
        );
        assert!(offsets(Neighborhood::Diamond).contains(&(0, -2)));
        assert!(!offsets(Neighborhood::Diamond).contains(&(1, -2)));
        let l = Lattice::new(&config(|_| {}));
        assert_eq!(
            (l.of(0).len(), l.of(5).len(), l.of(55).len()),
            (2, 3, 4),
            "corner, edge, interior"
        );
        let t = Lattice::new(&config(|c| c.boundary = Edges::Torus));
        assert!((0..100).all(|i| t.of(i).len() == 4));
        assert!(
            t.of(0).contains(&9) && t.of(0).contains(&90),
            "wraps both ways"
        );
        let d = Lattice::new(&config(|c| c.neighborhood = Neighborhood::Diamond));
        assert_eq!(d.of(55).len(), 12);
        assert_eq!(l.pairs().count(), 180, "10 × 9 × 2 bonds");
    }

    #[test]
    fn an_event_copies_only_when_the_drawn_feature_agrees() {
        // Two sites: identical ones never change; ones sharing nothing never interact.
        let mut same = world(
            config(|c| {
                c.width = 2;
                c.height = 1;
                c.features = 3
            }),
            &[&[1, 2, 3], &[1, 2, 3]],
        );
        let mut apart = world(
            config(|c| {
                c.width = 2;
                c.height = 1;
                c.features = 3
            }),
            &[&[1, 2, 3], &[4, 5, 6]],
        );
        for _ in 0..50 {
            same.step();
            apart.step();
        }
        assert_eq!(
            (same.culture(0), same.culture(1)),
            (&[1u8, 2, 3][..], &[1u8, 2, 3][..])
        );
        assert_eq!(
            (apart.culture(0), apart.culture(1)),
            (&[1u8, 2, 3][..], &[4u8, 5, 6][..])
        );
        assert!(same.is_stable() && apart.is_stable());
        // Sharing one of two features, the pair converges.
        let mut half = world(
            config(|c| {
                c.width = 2;
                c.height = 1;
                c.features = 2
            }),
            &[&[1, 2], &[1, 3]],
        );
        assert_eq!(half.stats.latest().unwrap().active_bonds, 1);
        half.run(1000);
        assert_eq!(half.culture(0), half.culture(1));
        assert!(half.is_finished());
    }

    #[test]
    fn the_active_site_or_its_neighbor_changes() {
        // Site 1 (the middle) is the only one with a partner it can copy.
        let lone = |changes| {
            let mut w = world(
                config(|c| {
                    c.width = 3;
                    c.height = 1;
                    c.features = 2;
                    c.changes = changes
                }),
                &[&[9, 9], &[1, 2], &[1, 3]],
            );
            w.run(1000);
            (
                w.culture(0).to_vec(),
                w.culture(1).to_vec(),
                w.culture(2).to_vec(),
            )
        };
        let (a0, a1, a2) = lone(Changes::Active);
        assert_eq!(a0, [9, 9]);
        assert_eq!(a1, a2, "the pair converged");
        let (n0, n1, n2) = lone(Changes::Neighbor);
        assert_eq!(n0, [9, 9]);
        assert_eq!(n1, n2);
    }

    #[test]
    fn incremental_bond_counts_match_a_recount() {
        let mut w = CultureWorld::new(
            config(|c| {
                c.width = 15;
                c.height = 15;
                c.traits = 6;
                c.neighborhood = Neighborhood::Diamond;
                c.boundary = Edges::Torus;
                c.drift = 0.001
            }),
            3,
        )
        .unwrap();
        for _ in 0..40 {
            w.step();
            let (active, sum) = (w.active_bonds, w.shared_sum);
            w.count_bonds();
            assert_eq!(
                (w.active_bonds, w.shared_sum),
                (active, sum),
                "tick {}",
                w.tick
            );
        }
    }

    #[test]
    fn regions_zones_and_cultures_are_counted_through_the_neighborhood() {
        // A 3 × 1 strip: A A B where A and B share one feature, and a far C.
        let c = config(|c| {
            c.width = 4;
            c.height = 1;
            c.features = 2
        });
        let w = world(c, &[&[1, 1], &[1, 1], &[1, 2], &[1, 1]]);
        let s = w.stats.latest().unwrap();
        assert_eq!(
            (s.regions, s.zones, s.cultures),
            (3, 1, 2),
            "the last site repeats the first culture"
        );
        assert_eq!(s.largest_region, 0.5);
        assert_eq!(s.active_bonds, 2);
        assert!((s.mean_similarity - (2.0 + 1.0 + 1.0) / 6.0).abs() < 1e-12);
        let apart = world(
            config(|c| {
                c.width = 3;
                c.height = 1;
                c.features = 2
            }),
            &[&[1, 1], &[2, 2], &[1, 1]],
        );
        let s = apart.stats.latest().unwrap();
        assert_eq!(
            (s.regions, s.zones, s.cultures, s.active_bonds),
            (3, 3, 2, 0)
        );
        assert_eq!(s.stable_at, 0);
    }

    #[test]
    fn soup_counts_cultures_zones_and_shared_values() {
        let c = config(|c| {
            c.width = 4;
            c.height = 1;
            c.features = 2;
            c.neighborhood = Neighborhood::Soup
        });
        let w = world(c.clone(), &[&[1, 1], &[1, 1], &[1, 2], &[3, 3]]);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.regions, s.cultures, s.zones), (3, 3, 2));
        assert_eq!(
            s.active_bonds, 1,
            "feature 0, trait 1 is held by two cultures"
        );
        // Pairs sharing: (0,1) 2, (0,2) 1, (1,2) 1 of 6 pairs × 2 features.
        assert!((s.mean_similarity - 4.0 / 12.0).abs() < 1e-12);
        let stable = world(c, &[&[1, 1], &[1, 1], &[2, 2], &[3, 3]]);
        assert!(stable.is_stable());
    }

    #[test]
    fn drift_keeps_the_world_changing_and_it_never_finishes() {
        let mut w = world(
            config(|c| {
                c.width = 2;
                c.height = 1;
                c.features = 2;
                c.drift = 0.5
            }),
            &[&[1, 1], &[1, 1]],
        );
        w.run(200);
        assert_eq!(w.tick, 200);
        assert!(!w.is_finished());
        assert!(w.stats.history().iter().map(|s| s.changes).sum::<u64>() > 0);
    }

    #[test]
    fn sweeps_visit_every_site_once_a_tick() {
        // With every site able to change and q = 2, F = 1, any site's copy flips it.
        let mut w = CultureWorld::new(
            config(|c| {
                c.activation = Activation::Sweep;
                c.stop_when_stable = false
            }),
            5,
        )
        .unwrap();
        w.run(3);
        assert_eq!(w.tick, 3);
        assert_eq!(w.stats.history().len(), 4);
    }

    #[test]
    fn a_run_stops_at_the_tick_it_becomes_stable() {
        let mut w = CultureWorld::new(config(|c| c.traits = 5), 2).unwrap();
        w.run(100_000);
        let s = w.stats.latest().unwrap();
        assert!(w.is_finished());
        assert_eq!((s.stable_at, s.active_bonds), (w.tick, 0));
        assert_eq!(s.regions, s.zones);
        let mut on = CultureWorld::new(
            config(|c| {
                c.traits = 5;
                c.stop_when_stable = false
            }),
            2,
        )
        .unwrap();
        on.run(w.tick as u32 + 10);
        assert_eq!(on.tick, w.tick + 10);
        assert_eq!(on.stats.latest().unwrap().stable_at, w.tick);
    }

    #[test]
    fn the_frame_draws_sites_and_lanes_and_inspect_maps_back() {
        let w = world(
            config(|c| {
                c.width = 2;
                c.height = 2;
                c.features = 2
            }),
            &[&[1, 1], &[1, 1], &[1, 2], &[3, 3]],
        );
        assert_eq!(Model::size(&w), (5, 5));
        let mut buf = Vec::new();
        w.render("culture", "", &mut buf).unwrap();
        let px = |b: &[u8], x: usize, y: usize| {
            [
                b[(y * 5 + x) * 4],
                b[(y * 5 + x) * 4 + 1],
                b[(y * 5 + x) * 4 + 2],
            ]
        };
        let a = hue_color(fnv(&[1, 1]));
        assert_eq!((px(&buf, 0, 0), px(&buf, 1, 1), px(&buf, 3, 0)), (a, a, a));
        assert_eq!(
            px(&buf, 2, 0),
            a,
            "identical neighbors: the lane is their color"
        );
        assert_eq!(
            px(&buf, 0, 2),
            lerp(BACKGROUND, LIGHT, 0.5),
            "one of two shared"
        );
        w.render("similarity", "", &mut buf).unwrap();
        assert_eq!((px(&buf, 0, 0), px(&buf, 2, 0)), (NEUTRAL, WHITE));
        w.render("zones", "", &mut buf).unwrap();
        assert_eq!(px(&buf, 3, 2), BACKGROUND, "sites 1 and 3 share nothing");
        assert!(w.render("wealth", "", &mut buf).is_err());
        let site = w.inspect(4, 4).unwrap();
        assert_eq!(
            (site.kind, site.a.x, site.a.y, site.a.traits.clone()),
            ("site", 1, 1, vec![3, 3])
        );
        assert_eq!(site.neighbors.len(), 2);
        let lane = w.inspect(2, 1).unwrap();
        assert_eq!(
            (lane.kind, lane.shared, lane.b.as_ref().unwrap().x),
            ("lane", Some(2), 1)
        );
        assert_eq!(w.inspect(1, 2).unwrap().shared, Some(1));
        assert!(w.inspect(5, 0).is_err());
        assert_eq!(Model::locate(&w, 1), None);
    }

    #[test]
    fn keyframes_keep_the_lattice_and_the_counts() {
        let mut any = crate::model::ModelWorld::new(
            ModelConfig::Culture(config(|c| c.stop_when_stable = false)),
            4,
        )
        .unwrap();
        any.model_mut().run(20);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut at20 = Vec::new();
        any.model().render("culture", "", &mut at20).unwrap();
        any.model_mut().run(30);
        any.restore(&cp).unwrap();
        let mut again = Vec::new();
        any.model().render("culture", "", &mut again).unwrap();
        assert_eq!((any.model().fingerprint(), at20), (print, again));
        assert_eq!(any.model().series("regions").unwrap().len(), 21);
    }

    #[test]
    fn live_edits_apply_and_the_lattice_waits_for_reset() {
        let mut w = CultureWorld::new(CultureConfig::default(), 1).unwrap();
        let next = config(|c| {
            c.activation = Activation::Sweep;
            c.drift = 0.01;
            c.changes = Changes::Neighbor
        });
        Model::set_config(&mut w, ModelConfig::Culture(next.clone())).unwrap();
        w.step();
        let e = Model::set_config(
            &mut w,
            ModelConfig::Culture(CultureConfig { traits: 11, ..next }),
        )
        .unwrap_err();
        assert_eq!(e[0].field, "traits");
    }

    #[test]
    fn degenerate_lattices_run_without_panicking() {
        for c in [
            config(|c| {
                c.width = 2;
                c.height = 2;
                c.features = 1;
                c.traits = 2
            }),
            config(|c| {
                c.width = 2;
                c.height = 2;
                c.neighborhood = Neighborhood::Soup
            }),
            config(|c| {
                c.width = 3;
                c.height = 3;
                c.boundary = Edges::Torus;
                c.neighborhood = Neighborhood::Moore
            }),
            config(|c| {
                c.features = 32;
                c.traits = 255;
                c.drift = 1.0
            }),
        ] {
            let mut w = CultureWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            let s = w.stats.latest().unwrap();
            assert!(
                s.regions >= 1 && (0.0..=1.0).contains(&s.mean_similarity),
                "{c:?}"
            );
        }
    }
}
