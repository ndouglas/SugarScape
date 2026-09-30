//! Schelling's bounded neighborhood (1971, "Dynamic Models of Segregation",
//! pp. 167–186; 1969, pp. 491–493): one area that everybody prefers to its
//! alternatives; each person inside unless "the percentage of residents of
//! opposite color exceeds some limit", their tolerance. "the least tolerant
//! leave first and the most tolerant enter first". His whites and blacks are
//! Red and Blue here.

use std::fmt::Write;

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::presets::ModelPreset;
use crate::render::{lerp, Rgb, BACKGROUND};
use crate::rng::{self, SimRng};
use crate::schema::{Apply, Param};
use crate::stats::{Series, Stats};

const RED: Rgb = [0xd9, 0x4a, 0x3f];
const BLUE: Rgb = [0x3f, 0x7f, 0xd9];
const BOTH: Rgb = [0xe8, 0xdc, 0xc2];
const PATH: Rgb = [0xff, 0xff, 0xff];
const NOW: Rgb = [0xf2, 0xc9, 0x4c];

/// A tolerance schedule: the tolerance of the i-th most tolerant of n people
/// (i = 1…n), the most of the other color per person of one's own accepted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum Schedule {
    /// A straight line from `intercept` down to 0 (Figs. 18 and 19): a·(1 − i/n).
    Line { intercept: f64 },
    /// A rectangular hyperbola (Fig. 27): i people tolerate `k` of the other
    /// color in all, so each tolerates k/i.
    Hyperbola { k: f64 },
    /// Steps (Fig. 26): each `(share, tolerance)` in turn, most tolerant first.
    Tiers { tiers: Vec<(f64, f64)> },
}

impl Schedule {
    /// The tolerance at rank `x` (1…n, fractional for random draws) of `n`.
    fn at(&self, x: f64, n: u32) -> f64 {
        let n = f64::from(n.max(1));
        match self {
            Schedule::Line { intercept } => (intercept * (1.0 - x / n)).max(0.0),
            Schedule::Hyperbola { k } => k / x.max(1.0),
            Schedule::Tiers { tiers } => {
                let (mut upto, q) = (0.0, x / n);
                for &(share, tolerance) in tiers {
                    upto += share;
                    if q <= upto + 1e-12 {
                        return tolerance;
                    }
                }
                tiers.last().map_or(0.0, |t| t.1)
            }
        }
    }

    /// The tolerances of `n` people, most tolerant first, with the least
    /// tolerant `intolerant` share made intolerant (0).
    pub fn tolerances(&self, n: u32, intolerant: f64) -> Vec<f64> {
        let keep = (f64::from(n) * (1.0 - intolerant)).round() as u32;
        (1..=n)
            .map(|i| {
                if i > keep {
                    0.0
                } else {
                    self.at(f64::from(i), n)
                }
            })
            .collect()
    }
}

/// How each person's tolerance is set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Draws {
    /// Exactly the schedule, rank by rank (Schelling's analysis).
    Schedule,
    /// Drawn from the schedule's distribution (runs differ by seed).
    Random,
}

/// Who is inside at the start.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Start {
    /// The most tolerant `red` Red and `blue` Blue (his analysis).
    Given { red: u32, blue: u32 },
    /// Each person inside with this chance.
    Random { chance: f64 },
}

/// Which color moves first in a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// Red, then Blue, each step.
    Alternate,
    /// Both colors' moves decided together, move by move (at speed 1, from
    /// the step's start); two entries into the last free place, Red's first.
    Simultaneous,
    /// Blue, then Red.
    BlueFirst,
}

/// How an outsider judges the ratio it would enter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Entry {
    /// Counting itself (his curves: n people tolerate n·R(n) of the other).
    CountingSelf,
    /// The ratio inside as it stands.
    AsIs,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TippingConfig {
    /// People of each color (his whites and blacks).
    pub red: u32,
    pub blue: u32,
    pub red_schedule: Schedule,
    pub blue_schedule: Schedule,
    /// The least tolerant share of each color made intolerant (1969: "Make
    /// the least tolerant 60 percent … absolutely intolerant").
    pub intolerant_red: f64,
    pub intolerant_blue: f64,
    pub draws: Draws,
    pub start: Start,
    /// Moves a step for each color ("relative speeds … we can watch and see
    /// how they matter").
    pub speed_red: u32,
    pub speed_blue: u32,
    pub order: Order,
    pub entry: Entry,
    /// The most of each color, and in all, allowed inside (0: no limit).
    pub limit_red: u32,
    pub limit_blue: u32,
    pub limit_total: u32,
}

impl Default for TippingConfig {
    /// Fig. 18 (1971): 100 whites and 50 blacks, each schedule a straight line
    /// from 2.0 to 0, starting with 25 of each inside.
    fn default() -> Self {
        TippingConfig {
            red: 100,
            blue: 50,
            red_schedule: Schedule::Line { intercept: 2.0 },
            blue_schedule: Schedule::Line { intercept: 2.0 },
            intolerant_red: 0.0,
            intolerant_blue: 0.0,
            draws: Draws::Schedule,
            start: Start::Given { red: 25, blue: 25 },
            speed_red: 1,
            speed_blue: 1,
            order: Order::Alternate,
            entry: Entry::CountingSelf,
            limit_red: 0,
            limit_blue: 0,
            limit_total: 0,
        }
    }
}

impl TippingConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let fraction = |v: f64| (0.0..=1.0).contains(&v);
        check(
            self.red <= 5000 && self.blue <= 5000,
            "red",
            "each color at most 5000",
        );
        check(self.red + self.blue >= 1, "red", "someone must exist");
        for (field, s) in [
            ("red_schedule", &self.red_schedule),
            ("blue_schedule", &self.blue_schedule),
        ] {
            let ok = match s {
                Schedule::Line { intercept } => *intercept >= 0.0,
                Schedule::Hyperbola { k } => *k >= 0.0,
                Schedule::Tiers { tiers } => {
                    !tiers.is_empty() && tiers.iter().all(|&(a, b)| a > 0.0 && b >= 0.0)
                }
            };
            check(ok, field, "tolerances must not be negative");
        }
        check(
            fraction(self.intolerant_red) && fraction(self.intolerant_blue),
            "intolerant_red",
            "must be fractions",
        );
        match self.start {
            Start::Given { red, blue } => check(
                red <= self.red && blue <= self.blue,
                "start",
                "cannot start with more inside than exist",
            ),
            Start::Random { chance } => {
                check(fraction(chance), "start", "the chance must be a fraction")
            }
        }
        check(
            self.speed_red >= 1 && self.speed_blue >= 1,
            "speed_red",
            "speeds must be at least 1",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }
}

/// The statistics series.
pub const SERIES: [&str; 6] = [
    "red_in",
    "blue_in",
    "red_unhappy",
    "blue_unhappy",
    "still",
    "blue_share",
];

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TippingSnapshot {
    pub tick: u64,
    pub red_in: u32,
    pub blue_in: u32,
    /// Insiders of each color who would leave (their tolerance exceeded).
    pub red_unhappy: u32,
    pub blue_unhappy: u32,
    /// 1 when a step ran and nobody moved.
    pub still: u32,
    /// Blue's share of those inside (0 when empty).
    pub blue_share: f64,
}

impl Series for TippingSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "red_in" => f64::from(self.red_in),
            "blue_in" => f64::from(self.blue_in),
            "red_unhappy" => f64::from(self.red_unhappy),
            "blue_unhappy" => f64::from(self.blue_unhappy),
            "still" => f64::from(self.still),
            "blue_share" => self.blue_share,
            _ => return None,
        })
    }
}

/// A point of the plane under Inspect: the state and who is content there.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TippingInspection {
    pub red_in: u32,
    pub blue_in: u32,
    /// Whether the most tolerant `red_in` Red (and `blue_in` Blue) would all be content.
    pub red_content: bool,
    pub blue_content: bool,
    pub now: bool,
    /// No one person to follow on the plane (the page's shared Inspect reads `agent`).
    pub agent: Option<u64>,
}

#[derive(Clone)]
pub struct TippingWorld {
    pub config: TippingConfig,
    pub tick: u64,
    /// Each color's tolerances, most tolerant first (`[blue, red]`).
    tolerance: [Vec<f64>; 2],
    /// Who is inside, by rank (`[blue, red]`).
    inside: [Vec<bool>; 2],
    /// The states visited, in order.
    path: Vec<(u32, u32)>,
    moved: bool,
    pub stats: Stats<TippingSnapshot>,
}

impl TippingWorld {
    pub fn new(config: TippingConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng: SimRng = rng::seeded(seed);
        let make = |n: u32, s: &Schedule, cut: f64, rng: &mut SimRng| -> Vec<f64> {
            match config.draws {
                Draws::Schedule => s.tolerances(n, cut),
                Draws::Random => {
                    // Each at a uniform rank in (0, n], then the least
                    // tolerant share made intolerant, as the schedule does.
                    let mut v: Vec<f64> = (0..n)
                        .map(|_| s.at((1.0 - rng.gen::<f64>()) * f64::from(n), n))
                        .collect();
                    v.sort_by(|a, b| b.total_cmp(a));
                    let keep = (f64::from(n) * (1.0 - cut)).round() as usize;
                    for t in v.iter_mut().skip(keep) {
                        *t = 0.0;
                    }
                    v
                }
            }
        };
        let tolerance = [
            make(
                config.blue,
                &config.blue_schedule,
                config.intolerant_blue,
                &mut rng,
            ),
            make(
                config.red,
                &config.red_schedule,
                config.intolerant_red,
                &mut rng,
            ),
        ];
        let inside = match config.start {
            Start::Given { red, blue } => [
                (0..config.blue).map(|i| i < blue).collect(),
                (0..config.red).map(|i| i < red).collect(),
            ],
            Start::Random { chance } => [
                (0..config.blue)
                    .map(|_| rng.gen::<f64>() < chance)
                    .collect(),
                (0..config.red).map(|_| rng.gen::<f64>() < chance).collect(),
            ],
        };
        let mut w = TippingWorld {
            config,
            tick: 0,
            tolerance,
            inside,
            path: Vec::new(),
            moved: false,
            stats: Stats::default(),
        };
        w.path.push(w.inside());
        let s = w.snapshot();
        w.stats.push(s);
        Ok(w)
    }

    /// Red and Blue inside.
    pub fn inside(&self) -> (u32, u32) {
        (self.count(1), self.count(0))
    }

    fn count(&self, c: usize) -> u32 {
        self.inside[c].iter().filter(|&&x| x).count() as u32
    }

    /// A color's tolerances, most tolerant first.
    pub fn tolerances(&self, red: bool) -> &[f64] {
        &self.tolerance[usize::from(red)]
    }

    /// Whether someone of color `c` with `tolerance`, among `own` of its color
    /// and `other` of the other, is content (no one of the other color is
    /// always content; an intolerant person with no company is content).
    fn content(tolerance: f64, own: u32, other: u32) -> bool {
        other == 0 || (own > 0 && f64::from(other) / f64::from(own) <= tolerance + 1e-12)
    }

    fn limit_allows(&self, c: usize) -> bool {
        let (r, b) = self.inside();
        let own_limit = if c == 1 {
            self.config.limit_red
        } else {
            self.config.limit_blue
        };
        let own = if c == 1 { r } else { b };
        (own_limit == 0 || own < own_limit)
            && (self.config.limit_total == 0 || r + b < self.config.limit_total)
    }

    /// Color `c`'s move against `state` (own, other inside): the least
    /// tolerant insider leaving if it is discontent, else the most tolerant
    /// outsider entering if it would be content and the limits allow.
    fn decide(&self, c: usize, own: u32, other: u32) -> Option<(usize, bool)> {
        let t = &self.tolerance[c];
        let ins = &self.inside[c];
        if let Some(i) = (0..t.len()).rev().find(|&i| ins[i]) {
            if !Self::content(t[i], own, other) {
                return Some((i, false));
            }
        }
        if let Some(i) = (0..t.len()).find(|&i| !ins[i]) {
            let own_after = match self.config.entry {
                Entry::CountingSelf => own + 1,
                Entry::AsIs => own.max(1),
            };
            if Self::content(t[i], own_after, other) && self.limit_allows(c) {
                return Some((i, true));
            }
        }
        None
    }

    fn apply(&mut self, c: usize, m: Option<(usize, bool)>) -> bool {
        match m {
            Some((i, enter)) => {
                self.inside[c][i] = enter;
                true
            }
            None => false,
        }
    }

    /// One step: each color in `order`, up to its speed of moves.
    pub fn step(&mut self) {
        let mut moved = false;
        let (sr, sb) = (self.config.speed_red, self.config.speed_blue);
        match self.config.order {
            Order::Simultaneous => {
                for k in 0..sr.max(sb) {
                    let (r, b) = self.inside();
                    let mr = if k < sr { self.decide(1, r, b) } else { None };
                    let mut mb = if k < sb { self.decide(0, b, r) } else { None };
                    // Two entries into the last free place: Red's is taken.
                    let total = self.config.limit_total;
                    if total > 0
                        && matches!((mr, mb), (Some((_, true)), Some((_, true))))
                        && r + b + 2 > total
                    {
                        mb = None;
                    }
                    moved |= self.apply(1, mr) | self.apply(0, mb);
                }
            }
            Order::Alternate | Order::BlueFirst => {
                let colors: [(usize, u32); 2] = if self.config.order == Order::Alternate {
                    [(1, sr), (0, sb)]
                } else {
                    [(0, sb), (1, sr)]
                };
                for (c, speed) in colors {
                    for _ in 0..speed {
                        let (r, b) = self.inside();
                        let (own, other) = if c == 1 { (r, b) } else { (b, r) };
                        let m = self.decide(c, own, other);
                        if !self.apply(c, m) {
                            break;
                        }
                        moved = true;
                    }
                }
            }
        }
        self.moved = moved;
        self.tick += 1;
        self.path.push(self.inside());
        let s = self.snapshot();
        self.stats.push(s);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    fn unhappy(&self, c: usize, own: u32, other: u32) -> u32 {
        (0..self.tolerance[c].len())
            .filter(|&i| self.inside[c][i] && !Self::content(self.tolerance[c][i], own, other))
            .count() as u32
    }

    fn snapshot(&self) -> TippingSnapshot {
        let (r, b) = self.inside();
        TippingSnapshot {
            tick: self.tick,
            red_in: r,
            blue_in: b,
            red_unhappy: self.unhappy(1, r, b),
            blue_unhappy: self.unhappy(0, b, r),
            still: u32::from(self.tick > 0 && !self.moved),
            blue_share: if r + b == 0 {
                0.0
            } else {
                f64::from(b) / f64::from(r + b)
            },
        }
    }

    /// Whether the most tolerant `n` of color `c` are all content with
    /// `other` of the other color: n people tolerate n·R(n).
    pub fn curve(&self, c: usize, n: u32, other: u32) -> bool {
        n == 0 || Self::content(self.tolerance[c][n as usize - 1], n, other)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<TippingInspection, String> {
        let (w, h) = (self.config.red + 1, self.config.blue + 1);
        if x >= w || y >= h {
            return Err(format!("({x}, {y}) is outside the plane"));
        }
        let (r, b) = (x, self.config.blue - y);
        Ok(TippingInspection {
            red_in: r,
            blue_in: b,
            red_content: self.curve(1, r, b),
            blue_content: self.curve(0, b, r),
            now: self.inside() == (r, b),
            agent: None,
        })
    }
}

impl Model for TippingWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Tipping(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        TippingWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        let (r, b) = self.inside();
        (r + b) as usize
    }

    /// FNV-1a over the tick, every tolerance and who is inside.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for c in 0..2 {
            for (t, &i) in self.tolerance[c].iter().zip(&self.inside[c]) {
                eat(t.to_bits());
                eat(u64::from(i));
            }
        }
        h
    }

    /// The plane: Red inside across, Blue inside up; where the most tolerant
    /// of Red are content tinted red, of Blue blue, of both cream; the path
    /// so far white and the state now yellow.
    fn size(&self) -> (u32, u32) {
        (self.config.red + 1, self.config.blue + 1)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        if mode != "plane" {
            return Err(format!("unknown color mode {mode:?}"));
        }
        let (w, h) = self.size();
        buf.clear();
        buf.resize((w * h) as usize * 4, 0);
        for y in 0..h {
            for x in 0..w {
                let (r, b) = (x, self.config.blue - y);
                let rgb = match (self.curve(1, r, b), self.curve(0, b, r)) {
                    (true, true) => BOTH,
                    (true, false) => lerp(BACKGROUND, RED, 0.45),
                    (false, true) => lerp(BACKGROUND, BLUE, 0.45),
                    (false, false) => BACKGROUND,
                };
                let k = ((y * w + x) * 4) as usize;
                buf[k..k + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
        let mut mark = |r: u32, b: u32, rgb: Rgb| {
            let k = (((self.config.blue - b) * w + r) * 4) as usize;
            buf[k..k + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        };
        for &(r, b) in &self.path {
            mark(r, b, PATH);
        }
        let (r, b) = self.inside();
        mark(r, b, NOW);
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

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("color,rank,tolerance,inside\n");
        for (c, name) in [(1, "red"), (0, "blue")] {
            for (i, (t, ins)) in self.tolerance[c].iter().zip(&self.inside[c]).enumerate() {
                writeln!(out, "{name},{},{t},{ins}", i + 1).unwrap();
            }
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let i = self.inspect(x, y)?;
        Ok(serde_json::to_string(&i).expect("inspection serializes"))
    }

    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Tipping(next) = next else {
            return Err(wrong_model(ModelKind::Tipping, &next));
        };
        next.validate()?;
        if next != self.config {
            return Err(vec![FieldError::new("config", "changes only on reset")]);
        }
        Ok(())
    }
}

/// The Rules panel's fields: every one rebuilds the area.
pub fn schema() -> Vec<Param> {
    use Apply::Reset;
    vec![
        Param::integer("Population", "red", "Red (his whites)", (0, 2000), Reset),
        Param::integer("Population", "blue", "Blue (his blacks)", (0, 2000), Reset),
        Param::integer("Population", "start.red", "Red inside at the start", (0, 2000), Reset)
            .with_help("The most tolerant of each color start inside."),
        Param::integer("Population", "start.blue", "Blue inside at the start", (0, 2000), Reset),
        Param::number("Tolerance", "red_schedule.intercept", "Red's most tolerant (straight line)", (0.0, 10.0, 0.1), Reset)
            .with_help("Schelling's straight schedules fall from this to 0: Fig. 18, 2.0; Fig. 19, 5.0."),
        Param::number("Tolerance", "blue_schedule.intercept", "Blue's most tolerant (straight line)", (0.0, 10.0, 0.1), Reset),
        Param::number("Tolerance", "intolerant_red", "Red's least tolerant made intolerant (share)", (0.0, 1.0, 0.05), Reset),
        Param::number("Tolerance", "intolerant_blue", "Blue's least tolerant made intolerant (share)", (0.0, 1.0, 0.05), Reset),
        Param::choice(
            "Tolerance",
            "draws",
            "Tolerances",
            &[("schedule", "Exactly the schedule (Schelling)"), ("random", "Drawn from it at random")],
            Reset,
        ),
        Param::integer("Moves", "speed_red", "Red moves a step", (1, 20), Reset)
            .with_help("\"we need not stipulate in advance whether whites move in or out more rapidly than blacks\""),
        Param::integer("Moves", "speed_blue", "Blue moves a step", (1, 20), Reset),
        Param::choice(
            "Moves",
            "order",
            "Order",
            &[
                ("alternate", "Red, then Blue"),
                ("blue_first", "Blue, then Red"),
                ("simultaneous", "Both at once"),
            ],
            Reset,
        ),
        Param::choice(
            "Moves",
            "entry",
            "An outsider judges",
            &[("counting_self", "Counting itself (his curves)"), ("as_is", "The ratio as it stands (with none of its color inside, as one)")],
            Reset,
        ),
        Param::integer("Limits", "limit_red", "Most Red inside (0: none)", (0, 2000), Reset),
        Param::integer("Limits", "limit_blue", "Most Blue inside (0: none)", (0, 2000), Reset),
        Param::integer("Limits", "limit_total", "Most inside in all (0: none)", (0, 4000), Reset),
    ]
}

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut TippingConfig),
) -> ModelPreset {
    let mut c = TippingConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Tipping(c),
    }
}

fn lines(c: &mut TippingConfig, intercept: f64) {
    c.red_schedule = Schedule::Line { intercept };
    c.blue_schedule = Schedule::Line { intercept };
}

/// Schelling's bounded-neighborhood figures (1971; 1969).
pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "tipping-fig18",
            "Schelling's tipping: Fig. 18",
            "Schelling 1971, Fig. 18",
            "100 Red and 50 Blue, each color's tolerance falling in a straight line from 2.0 to 0 (the median Red abides one Blue per Red); 25 of each start inside. Schelling: \"There are only two stable equilibria. One consists of all the blacks and no whites, the other all the whites and no blacks.\" Measured (exact schedules): from 25 and 25 it ends all Red; every start on a grid ends with one color gone.",
            |_| {},
        ),
        preset(
            "tipping-fig19",
            "Schelling's tipping: Fig. 19",
            "Schelling 1971, Fig. 19",
            "100 of each, tolerances falling from 5.0 (median 2.5); 50 of each start inside. Schelling: \"there is a stable mixture at 80 blacks and 80 whites\", reached \"as long as half or more of both colors are present—actually, slightly over 40%\". Measured: 80 and 80, reached from every start with 41 or more of each; from all Red, 28 Blue entering together reach it and 26 do not.",
            |c| {
                c.blue = 100;
                lines(c, 5.0);
                c.start = Start::Given { red: 50, blue: 50 };
            },
        ),
        preset(
            "tipping-fig20",
            "Schelling's tipping: Fig. 20",
            "Schelling 1971, Fig. 20",
            "Fig. 19's schedules with 100 Red and 50 Blue. Schelling: \"The stable equilibrium generated in Figure 19 disappears if … whites exceed blacks by, say, two to one.\" Measured: every start with both inside ends with one color gone (from 60 and 40, all Red).",
            |c| {
                c.red = 100;
                c.blue = 50;
                lines(c, 5.0);
                c.start = Start::Given { red: 60, blue: 40 };
            },
        ),
        preset(
            "tipping-fig21",
            "Schelling's tipping: the threshold",
            "Schelling 1971, Fig. 21",
            "Equal numbers (100 each) with straight lines from 3.0: the border. Schelling: \"there is no stable intersection of the two parabolas unless the tolerance schedules have vertical intercepts of 3.0\". Measured: from unequal starts (steps of 5), none ends mixed at intercepts up to 2.9; some do from 2.95 (104 of 420 at 3.0): a mix holds at 3.0 (71 Red, 61 Blue from 55 and 45).",
            |c| {
                c.blue = 100;
                lines(c, 3.0);
                // Off the knife edge: at exactly 50 and 50 nobody moves, stable or not.
                c.start = Start::Given { red: 55, blue: 45 };
            },
        ),
        preset(
            "tipping-fig22",
            "Schelling's tipping: Red limited to 40",
            "Schelling 1971, Fig. 22",
            "Fig. 20's numbers (100 Red, 50 Blue, Fig. 19's schedules) with at most 40 Red inside, \"the most tolerant 40 … the first to enter and the last to leave\". Schelling: \"a stable mixture at 40 whites and a comparable number of blacks\". Measured: 40 Red and 40 Blue.",
            |c| {
                c.red = 100;
                c.blue = 50;
                lines(c, 5.0);
                c.limit_red = 40;
                c.start = Start::Given { red: 40, blue: 40 };
            },
        ),
        preset(
            "tipping-intolerant",
            "Schelling's tipping: the least tolerant 60 % intolerant",
            "Schelling 1969",
            "Equal numbers (100 each), straight lines from 2.0, the least tolerant 60 % of each made \"absolutely intolerant\". Schelling (1969): \"a stable equilibrium will occur at forty apiece\". Measured: 40 and 40, exactly.",
            |c| {
                c.blue = 100;
                c.intolerant_red = 0.6;
                c.intolerant_blue = 0.6;
            },
        ),
        preset(
            "tipping-minority",
            "Schelling's tipping: a small minority as tolerant as the rest",
            "Schelling 1971, p. 179",
            "500 Red and 100 Blue, both with Fig. 19's schedules. Schelling: \"for a stable mixture, the minority must be the more tolerant of the two groups\". Measured: every start with both inside ends with the minority pushed out.",
            |c| {
                c.red = 500;
                c.blue = 100;
                lines(c, 5.0);
                c.start = Start::Given { red: 80, blue: 80 };
            },
        ),
        preset(
            "tipping-less-tolerant",
            "Schelling's tipping: the least tolerant made less tolerant",
            "Schelling 1971, p. 174",
            "Fig. 20's numbers (100 Red, 50 Blue), with the least tolerant two-thirds of Red made intolerant instead of Red being limited. Schelling: \"replacing the two-thirds least tolerant whites … by even less tolerant whites keeps the whites from overwhelming the blacks by their numbers. This would not happen if we made all whites less tolerant.\" Measured: a stable mixture at 33 Red and 42 Blue, where the same numbers as they are, or with all Red less tolerant, end all Red.",
            |c| {
                c.red = 100;
                c.blue = 50;
                lines(c, 5.0);
                c.intolerant_red = 2.0 / 3.0;
                c.start = Start::Given { red: 40, blue: 40 };
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn line(red: u32, blue: u32, intercept: f64, start: (u32, u32)) -> TippingConfig {
        TippingConfig {
            red,
            blue,
            red_schedule: Schedule::Line { intercept },
            blue_schedule: Schedule::Line { intercept },
            start: Start::Given {
                red: start.0,
                blue: start.1,
            },
            ..TippingConfig::default()
        }
    }

    fn fig18(r: u32, b: u32) -> TippingConfig {
        line(100, 50, 2.0, (r, b))
    }

    fn fig19(r: u32, b: u32) -> TippingConfig {
        line(100, 100, 5.0, (r, b))
    }

    fn run_to_rest(c: TippingConfig) -> (u32, u32) {
        let mut w = TippingWorld::new(c, 1).unwrap();
        for _ in 0..10_000 {
            w.step();
            if w.stats.latest().unwrap().still == 1 {
                break;
            }
        }
        w.inside()
    }

    #[test]
    fn schedules_rank_as_schelling_counts_them() {
        let l = Schedule::Line { intercept: 2.0 };
        let t = l.tolerances(100, 0.0);
        assert!(
            close(t[0], 1.98) && close(t[49], 1.0) && close(t[99], 0.0),
            "50 of 100 abide 1.0 or more"
        );
        let n = |k: usize| k as f64 * t[k - 1];
        assert!(
            close(n(50), 50.0) && close(n(75), 37.5) && close(n(90), 18.0),
            "p. 169"
        );
        assert!(close(n(20), 32.0), "20 tolerate 32 (his 36 is a slip)");
        assert!(close(
            Schedule::Hyperbola { k: 90.0 }.tolerances(100, 0.0)[9],
            9.0
        ));
        let tiers = Schedule::Tiers {
            tiers: vec![(0.25, 3.0), (0.75, 0.5)],
        };
        let tt = tiers.tolerances(8, 0.0);
        assert_eq!((tt[0], tt[1], tt[2], tt[7]), (3.0, 3.0, 0.5, 0.5));
        let cut = l.tolerances(100, 0.6);
        assert!(
            cut[40..].iter().all(|&r| r == 0.0) && cut[39] > 0.0,
            "the least tolerant 60 % intolerant"
        );
    }

    #[test]
    fn the_least_tolerant_leave_and_the_most_tolerant_enter() {
        // Four of each, tolerances 1.5, 1.0, 0.5 and 0. Three Red and two
        // Blue inside: the third Red (0.5) sees 2 Blue per 3 Red and leaves;
        // then two and two are content, and the third Blue, counting itself,
        // would face 2 Red per 3 Blue, over its 0.5: nobody else moves.
        let mut w = TippingWorld::new(line(4, 4, 2.0, (3, 2)), 1).unwrap();
        assert_eq!(w.tolerances(true), &[1.5, 1.0, 0.5, 0.0]);
        w.step();
        assert_eq!(w.inside(), (2, 2));
        assert_eq!(w.stats.latest().unwrap().still, 0);
        w.step();
        assert_eq!(w.inside(), (2, 2));
        assert_eq!(w.stats.latest().unwrap().still, 1);
    }

    #[test]
    fn fig_18_ends_all_one_color_and_fig_19_holds_80_80() {
        let end = run_to_rest(fig18(25, 25));
        assert!(end.0 == 0 || end.1 == 0, "{end:?}");
        assert_eq!(run_to_rest(fig19(50, 50)), (80, 80));
        assert_eq!(
            run_to_rest(fig19(100, 20)).1,
            0,
            "20 Blue entering together are too few"
        );
    }

    #[test]
    fn a_limit_stops_entry_at_the_cap() {
        let c = TippingConfig {
            limit_red: 4,
            ..line(10, 0, 2.0, (0, 0))
        };
        assert_eq!(run_to_rest(c), (4, 0));
    }

    #[test]
    fn simultaneous_moves_read_the_same_state() {
        // From three Red and two Blue: taking turns, the Red leaves and the
        // Blue is then content; together, the Blue reads 3 Red per 2 Blue
        // (over its 1.0) and leaves too.
        let mut turns = TippingWorld::new(line(4, 4, 2.0, (3, 2)), 1).unwrap();
        turns.step();
        let mut together = TippingWorld::new(
            TippingConfig {
                order: Order::Simultaneous,
                ..line(4, 4, 2.0, (3, 2))
            },
            1,
        )
        .unwrap();
        together.step();
        assert_eq!((turns.inside(), together.inside()), ((2, 2), (2, 1)));
    }

    #[test]
    fn random_draws_and_starts_differ_by_seed_but_rank_the_same_way() {
        let c = TippingConfig {
            draws: Draws::Random,
            start: Start::Random { chance: 0.5 },
            ..fig18(0, 0)
        };
        let (a, b) = (
            TippingWorld::new(c.clone(), 1).unwrap(),
            TippingWorld::new(c, 2).unwrap(),
        );
        assert_ne!(a.tolerances(true), b.tolerances(true));
        assert!(
            a.tolerances(true).windows(2).all(|p| p[0] >= p[1]),
            "most tolerant first"
        );
    }

    #[test]
    fn random_draws_make_the_least_tolerant_share_intolerant() {
        let c = TippingConfig {
            draws: Draws::Random,
            intolerant_red: 0.6,
            ..line(100, 50, 2.0, (0, 0))
        };
        let w = TippingWorld::new(c, 3).unwrap();
        let t = w.tolerances(true);
        assert!(
            t[..40].iter().all(|&r| r > 0.0),
            "the most tolerant 40 % keep a tolerance"
        );
        assert!(
            t[40..].iter().all(|&r| r == 0.0),
            "the least tolerant 60 % are intolerant"
        );
        assert!(t[0] < 2.0, "no draw above the schedule's top");
    }

    #[test]
    fn together_the_colors_cannot_overfill_a_total_limit() {
        let c = TippingConfig {
            order: Order::Simultaneous,
            limit_total: 3,
            ..line(10, 10, 2.0, (1, 1))
        };
        let mut w = TippingWorld::new(c, 1).unwrap();
        w.run(20);
        let (r, b) = w.inside();
        assert!(r + b <= 3, "{r} + {b}");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Tipping(TippingConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }

    #[test]
    fn renders_the_plane() {
        let mut w = TippingWorld::new(fig18(25, 25), 1).unwrap();
        assert_eq!(w.size(), (101, 51));
        let mut buf = Vec::new();
        w.render("plane", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 101 * 51 * 4);
        w.run(5);
        assert!(w.inspect(25, 50 - 25).is_ok());
        assert!(w.inspect(200, 0).is_err());
    }
}
