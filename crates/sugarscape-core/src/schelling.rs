//! The book's variant of Schelling's segregation model (Chapter VI,
//! animations VI-4 to VI-7): Red and Blue agents on a torus, each wanting at
//! least a fraction of its von Neumann neighbors to share its color, moving
//! to a random acceptable site when unsatisfied.

use std::collections::BTreeMap;
use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::export;
use crate::geometry::{Pos, Torus};
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::presets::ModelPreset;
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, RED};
use crate::rng::{self, SimRng};
use crate::schema::{Apply, Param};
use crate::stats::{Series, Stats};

/// A fraction range `[min, max]` (preferences).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FRange {
    pub min: f64,
    pub max: f64,
}

impl Default for FRange {
    /// The default preference, VI-4's 25%.
    fn default() -> Self {
        FRange {
            min: 0.25,
            max: 0.25,
        }
    }
}

/// Note 11's "maximum lifetime", read as a maximum residence: each agent
/// leaves after a whole number of ticks drawn from `[min, max]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Residence {
    pub enabled: bool,
    pub min: u32,
    pub max: u32,
}

impl Default for Residence {
    /// Off, with VI-5's 80–100 range ready if it is turned on.
    fn default() -> Self {
        Residence {
            enabled: false,
            min: 80,
            max: 100,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SchellingConfig {
    pub width: u32,
    pub height: u32,
    pub population: u32,
    /// Each agent's minimum share of like-colored neighbors, uniform in
    /// `[min, max]` (min = max for a fixed preference).
    pub preference: FRange,
    pub residence: Residence,
}

impl Default for SchellingConfig {
    /// Animation VI-4: "a 50 × 50 lattice … with 2000 Red and Blue agents …
    /// at least 25 percent of its neighbors", no maximum residence.
    fn default() -> Self {
        SchellingConfig {
            width: 50,
            height: 50,
            population: 2000,
            preference: FRange {
                min: 0.25,
                max: 0.25,
            },
            residence: Residence {
                enabled: false,
                min: 80,
                max: 100,
            },
        }
    }
}

impl SchellingConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (5..=500).contains(&self.width),
            "width",
            "must be between 5 and 500",
        );
        check(
            (5..=500).contains(&self.height),
            "height",
            "must be between 5 and 500",
        );
        check(
            u64::from(self.population) < u64::from(self.width) * u64::from(self.height),
            "population",
            "must be less than the number of sites (width × height)",
        );
        let p = self.preference;
        let fraction = |v: f64| (0.0..=1.0).contains(&v);
        check(
            fraction(p.min) && fraction(p.max),
            "preference",
            "must be fractions between 0 and 1",
        );
        check(p.min <= p.max, "preference", "min must be ≤ max");
        let r = self.residence;
        check(r.min >= 1, "residence.min", "must be ≥ 1");
        check(r.min <= r.max, "residence", "min must be ≤ max");
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that differ from `next` (every field changes only on reset).
    fn changes(&self, next: &SchellingConfig) -> Vec<FieldError> {
        let msg = "changes only on reset";
        let mut out = Vec::new();
        for (field, same) in [
            ("width", self.width == next.width),
            ("height", self.height == next.height),
            ("population", self.population == next.population),
            ("preference", self.preference == next.preference),
            ("residence", self.residence == next.residence),
        ] {
            if !same {
                out.push(FieldError::new(field, msg));
            }
        }
        out
    }
}

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 6] = [
    "unsatisfied",
    "segregation",
    "moves",
    "red_share",
    "quiet",
    "population",
];

/// One tick's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SchellingSnapshot {
    pub tick: u64,
    pub population: u32,
    /// Share of agents not satisfied where they stand (0 with no agents).
    pub unsatisfied: f64,
    /// Mean share of like-colored neighbors over the agents with at least
    /// one neighbor (0 when none has one).
    pub segregation: f64,
    /// Agents that moved this tick.
    pub moves: u32,
    /// Share of agents that are Red (0 with no agents).
    pub red_share: f64,
    /// 1 when a tick ran and no agent moved, else 0 (0 at t = 0).
    pub quiet: u32,
}

impl Series for SchellingSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "unsatisfied" => self.unsatisfied,
            "segregation" => self.segregation,
            "moves" => f64::from(self.moves),
            "red_share" => self.red_share,
            "quiet" => f64::from(self.quiet),
            _ => return None,
        })
    }
}

/// One agent: its color, preference, age and (with residence on) the age at
/// which it leaves.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resident {
    pub id: u64,
    pub pos: Pos,
    pub red: bool,
    pub preference: f64,
    pub age: u32,
    /// The age at which it leaves; 0 with residence off.
    pub residence: u32,
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SchellingInspection {
    pub site: SiteXy,
    pub agent: Option<ResidentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct SiteXy {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ResidentView {
    pub id: u64,
    pub color: &'static str,
    pub preference: f64,
    pub satisfied: bool,
    /// Like-colored and all occupied von Neumann neighbors.
    pub like: u32,
    pub neighbors: u32,
    pub age: u32,
    /// The age at which it leaves, or null with residence off.
    pub residence: Option<u32>,
}

/// The color modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchellingMode {
    Color,
    Satisfaction,
    Preference,
}

impl std::str::FromStr for SchellingMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "color" => Self::Color,
            "satisfaction" => Self::Satisfaction,
            "preference" => Self::Preference,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// Whether `like` of `occupied` neighbors meet `preference` ("if this number
/// is greater than or equal to its preference the agent is … satisfied");
/// an agent with no neighbors is satisfied.
pub fn satisfied(like: u32, occupied: u32, preference: f64) -> bool {
    occupied == 0 || f64::from(like) / f64::from(occupied) >= preference
}

/// A (like, occupied) neighbor class: `occupied` in 0..=4, `like` in
/// 0..=occupied, numbered `occupied·(occupied + 1)/2 + like` (15 classes).
const CLASSES: usize = 15;
const UNFILED: u8 = u8::MAX;

fn class(like: u8, occupied: u8) -> usize {
    usize::from(occupied) * (usize::from(occupied) + 1) / 2 + usize::from(like)
}

/// Whether an agent with `preference` is satisfied in each class.
fn acceptable_classes(preference: f64) -> [bool; CLASSES] {
    let mut out = [false; CLASSES];
    for occupied in 0..=4u8 {
        for like in 0..=occupied {
            out[class(like, occupied)] =
                satisfied(u32::from(like), u32::from(occupied), preference);
        }
    }
    out
}

pub struct SchellingWorld {
    pub config: SchellingConfig,
    pub torus: Torus,
    /// Completed ticks.
    pub tick: u64,
    agents: BTreeMap<u64, Resident>,
    /// The agent on each site (row-major).
    grid: Vec<Option<u64>>,
    /// Occupied von Neumann neighbors of each site: `[blue, red]` counts.
    counts: [Vec<u8>; 2],
    /// Every empty site filed, for each color (`[blue, red]`), under the
    /// class it would give an agent of that color standing there.
    pools: [[Vec<u32>; CLASSES]; 2],
    /// Each site's class in `pools` (`UNFILED` when occupied, or while its
    /// agent considers moving) and its index there.
    filed: [Vec<u8>; 2],
    slot: [Vec<u32>; 2],
    rng: SimRng,
    next_id: u64,
    moves: u32,
    pub stats: Stats<SchellingSnapshot>,
}

impl SchellingWorld {
    pub fn new(config: SchellingConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let torus = Torus::new(config.width, config.height);
        let n = torus.len();
        let mut world = SchellingWorld {
            torus,
            tick: 0,
            agents: BTreeMap::new(),
            grid: vec![None; n],
            counts: [vec![0; n], vec![0; n]],
            pools: Default::default(),
            filed: [vec![UNFILED; n], vec![UNFILED; n]],
            slot: [vec![0; n], vec![0; n]],
            rng: rng::seeded(seed),
            next_id: 1,
            moves: 0,
            stats: Stats::default(),
            config,
        };
        for i in 0..n {
            world.file(i);
        }
        let mut cells: Vec<usize> = (0..n).collect();
        cells.shuffle(&mut world.rng);
        for &i in cells.iter().take(world.config.population as usize) {
            let agent = world.newcomer(torus.pos(i));
            world.insert(agent);
        }
        let snapshot = world.snapshot();
        world.stats.push(snapshot);
        Ok(world)
    }

    /// A new agent at `pos`: Red or Blue with probability ½, a preference
    /// uniform in the configured range, and (residence on) a residence
    /// uniform in its range.
    fn newcomer(&mut self, pos: Pos) -> Resident {
        let c = &self.config;
        let (p, r) = (c.preference, c.residence);
        let red = self.rng.gen_bool(0.5);
        let preference = self.rng.gen_range(p.min..=p.max);
        let residence = if r.enabled {
            self.rng.gen_range(r.min..=r.max)
        } else {
            0
        };
        let id = self.next_id;
        self.next_id += 1;
        Resident {
            id,
            pos,
            red,
            preference,
            age: 0,
            residence,
        }
    }

    /// Files empty site `i` under its class for each color.
    fn file(&mut self, i: usize) {
        let (blues, reds) = (self.counts[0][i], self.counts[1][i]);
        for (c, like) in [(0, blues), (1, reds)] {
            let k = class(like, blues + reds);
            self.filed[c][i] = k as u8;
            self.slot[c][i] = self.pools[c][k].len() as u32;
            self.pools[c][k].push(i as u32);
        }
    }

    fn unfile(&mut self, i: usize) {
        for c in 0..2 {
            let k = self.filed[c][i];
            if k == UNFILED {
                continue;
            }
            let pool = &mut self.pools[c][usize::from(k)];
            let at = self.slot[c][i] as usize;
            pool.swap_remove(at);
            if let Some(&moved) = pool.get(at) {
                self.slot[c][moved as usize] = at as u32;
            }
            self.filed[c][i] = UNFILED;
        }
    }

    /// Adds `delta` to the `red` count of each neighbor of site `i`,
    /// refiling the empty ones.
    fn tell_neighbors(&mut self, i: usize, red: bool, add: bool) {
        for n in self.torus.neighbors(self.torus.pos(i)) {
            let j = self.torus.index(n);
            let count = &mut self.counts[usize::from(red)][j];
            if add {
                *count += 1;
            } else {
                *count -= 1;
            }
            if self.filed[0][j] != UNFILED {
                self.unfile(j);
                self.file(j);
            }
        }
    }

    /// Puts agent `id` of color `red` on site `i`.
    fn occupy(&mut self, i: usize, id: u64, red: bool) {
        self.unfile(i);
        debug_assert!(self.grid[i].is_none());
        self.grid[i] = Some(id);
        self.tell_neighbors(i, red, true);
    }

    /// Takes the agent of color `red` off site `i`, leaving it unfiled.
    fn vacate(&mut self, i: usize, red: bool) {
        self.grid[i] = None;
        self.tell_neighbors(i, red, false);
    }

    fn insert(&mut self, a: Resident) {
        self.occupy(self.torus.index(a.pos), a.id, a.red);
        self.agents.insert(a.id, a);
    }

    /// A uniformly random filed site that satisfies an agent of color `red`
    /// with `preference` (every filed site when `preference` is `None`).
    fn pick(&mut self, red: bool, preference: Option<f64>) -> Option<usize> {
        let ok = preference.map_or([true; CLASSES], acceptable_classes);
        let pools = &self.pools[usize::from(red)];
        let total: usize = (0..CLASSES)
            .filter(|&k| ok[k])
            .map(|k| pools[k].len())
            .sum();
        if total == 0 {
            return None;
        }
        // Sampled as u32 (sites ≤ 250 000): a usize range would draw
        // differently on wasm32 than on 64-bit targets.
        let mut r = self.rng.gen_range(0..total as u32) as usize;
        for k in (0..CLASSES).filter(|&k| ok[k]) {
            let pool = &pools[k];
            if r < pool.len() {
                return Some(pool[r] as usize);
            }
            r -= pool.len();
        }
        unreachable!("r < total")
    }

    pub fn agents(&self) -> impl Iterator<Item = &Resident> {
        self.agents.values()
    }

    pub fn agent_at(&self, pos: Pos) -> Option<&Resident> {
        self.grid[self.torus.index(pos)].and_then(|id| self.agents.get(&id))
    }

    /// Like-colored and occupied neighbors of `a` where it stands.
    fn neighbors(&self, a: &Resident) -> (u32, u32) {
        let i = self.torus.index(a.pos);
        let (blues, reds) = (self.counts[0][i], self.counts[1][i]);
        let like = if a.red { reds } else { blues };
        (u32::from(like), u32::from(blues + reds))
    }

    /// Whether `a` is satisfied where it stands.
    pub fn is_satisfied(&self, a: &Resident) -> bool {
        let (like, occupied) = self.neighbors(a);
        satisfied(like, occupied, a.preference)
    }

    /// One tick: agents act in a random order, and each unsatisfied one moves
    /// to a site chosen uniformly at random among the empty sites where it
    /// would be satisfied — judged with its own site vacated, so it does not
    /// count itself as a neighbor — or stays if there is none. Then every
    /// agent ages, and with residence on each that reaches its maximum is
    /// replaced.
    pub fn step(&mut self) {
        let mut order: Vec<u64> = self.agents.keys().copied().collect();
        order.shuffle(&mut self.rng);
        let mut moves = 0;
        for id in order {
            let a = self.agents[&id];
            if self.is_satisfied(&a) {
                continue;
            }
            let from = self.torus.index(a.pos);
            self.vacate(from, a.red);
            match self.pick(a.red, Some(a.preference)) {
                Some(to) => {
                    self.occupy(to, id, a.red);
                    self.file(from);
                    self.agents.get_mut(&id).expect("living agent").pos = self.torus.pos(to);
                    moves += 1;
                }
                None => self.occupy(from, id, a.red),
            }
        }
        for a in self.agents.values_mut() {
            a.age += 1;
        }
        if self.config.residence.enabled {
            self.replace_departed();
        }
        self.moves = moves;
        self.tick += 1;
        let snapshot = self.snapshot();
        self.stats.push(snapshot);
    }

    /// VI-5: each agent that has reached its maximum residence leaves (in id
    /// order) and "is replaced … with a new agent of random color … placed
    /// at a randomly selected position satisfying its preference" — any
    /// empty site if none satisfies it.
    fn replace_departed(&mut self) {
        let departed: Vec<u64> = self
            .agents
            .values()
            .filter(|a| a.residence > 0 && a.age >= a.residence)
            .map(|a| a.id)
            .collect();
        for id in departed {
            let old = self.agents.remove(&id).expect("living agent");
            let i = self.torus.index(old.pos);
            self.vacate(i, old.red);
            self.file(i);
            let mut new = self.newcomer(old.pos);
            let to = self
                .pick(new.red, Some(new.preference))
                .or_else(|| self.pick(new.red, None))
                .expect("the departed agent's site is empty");
            new.pos = self.torus.pos(to);
            self.insert(new);
        }
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    fn snapshot(&self) -> SchellingSnapshot {
        let n = self.agents.len();
        let (mut unsatisfied, mut reds, mut shares, mut counted) = (0usize, 0usize, 0.0, 0usize);
        for a in self.agents.values() {
            let (like, occupied) = self.neighbors(a);
            if !satisfied(like, occupied, a.preference) {
                unsatisfied += 1;
            }
            if a.red {
                reds += 1;
            }
            if occupied > 0 {
                shares += f64::from(like) / f64::from(occupied);
                counted += 1;
            }
        }
        let share = |k: usize, of: usize| if of == 0 { 0.0 } else { k as f64 / of as f64 };
        SchellingSnapshot {
            tick: self.tick,
            population: n as u32,
            unsatisfied: share(unsatisfied, n),
            segregation: if counted == 0 {
                0.0
            } else {
                shares / counted as f64
            },
            moves: self.moves,
            red_share: share(reds, n),
            quiet: u32::from(self.tick > 0 && self.moves == 0),
        }
    }

    fn color(&self, a: &Resident, mode: SchellingMode) -> Rgb {
        let own = if a.red { RED } else { BLUE };
        match mode {
            SchellingMode::Color => own,
            SchellingMode::Satisfaction => {
                if self.is_satisfied(a) {
                    lerp(BACKGROUND, own, 0.3)
                } else {
                    BOTH
                }
            }
            SchellingMode::Preference => lerp(COOL, HOT, a.preference),
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<SchellingInspection, String> {
        if x >= self.torus.width || y >= self.torus.height {
            return Err(format!("({x}, {y}) is outside the grid"));
        }
        let agent = self.agent_at(Pos::new(x, y)).map(|a| {
            let (like, neighbors) = self.neighbors(a);
            ResidentView {
                id: a.id,
                color: if a.red { "red" } else { "blue" },
                preference: a.preference,
                satisfied: satisfied(like, neighbors, a.preference),
                like,
                neighbors,
                age: a.age,
                residence: (a.residence > 0).then_some(a.residence),
            }
        });
        Ok(SchellingInspection {
            site: SiteXy { x, y },
            agent,
        })
    }
}
impl Model for SchellingWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Schelling(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        SchellingWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick and every agent's id, site, color, preference,
    /// age and residence.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.pos.x) << 32) | u64::from(a.pos.y));
            eat(u64::from(a.red));
            eat(a.preference.to_bits());
            eat((u64::from(a.age) << 32) | u64::from(a.residence));
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.torus.width, self.torus.height)
    }

    /// Empty sites dark; agents Red/Blue (`color`), dimmed when satisfied and
    /// yellow when not (`satisfaction`), or cool-to-hot by preference
    /// (`preference`). `layer` is ignored.
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: SchellingMode = mode.parse()?;
        buf.resize(self.grid.len() * 4, 0);
        for (i, slot) in self.grid.iter().enumerate() {
            let rgb = match slot {
                Some(id) => self.color(&self.agents[id], mode),
                None => BACKGROUND,
            };
            buf[i * 4..i * 4 + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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
        let mut out = String::from("id,x,y,color,preference,satisfied,age,residence\n");
        for a in self.agents.values() {
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                a.id,
                a.pos.x,
                a.pos.y,
                if a.red { "red" } else { "blue" },
                a.preference,
                self.is_satisfied(a),
                a.age,
                a.residence
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
        self.agents.get(&id).map(|a| (a.pos.x, a.pos.y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Schelling(next) = next else {
            return Err(wrong_model(ModelKind::Schelling, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }
}

/// The Rules panel's fields: every one rebuilds the world (agents draw their
/// preference and residence when they are created).
pub fn schema() -> Vec<Param> {
    use Apply::Reset;
    // UI limits, narrower than `validate()` on purpose (performance): the
    // panel offers width and height 5–200 and residence 1–1000 ticks, while
    // configs from files and links may use `validate()`'s full range (width
    // and height 5–500, any population below width × height, residence ≥ 1).
    vec![
        Param::integer("Setup", "width", "Width", (5, 200), Reset),
        Param::integer("Setup", "height", "Height", (5, 200), Reset),
        Param::integer("Setup", "population", "Agents", (0, 39_999), Reset),
        Param::range(
            "Preference",
            "preference",
            "Like neighbors wanted (fraction)",
            (0.0, 1.0, 0.05),
            Reset,
        ),
        Param::bool("Residence", "residence.enabled", "Maximum residence", Reset),
        Param::range(
            "Residence",
            "residence",
            "Residence (ticks)",
            (1.0, 1000.0, 1.0),
            Reset,
        ),
    ]
}

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut SchellingConfig),
) -> ModelPreset {
    let mut c = SchellingConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Schelling(c),
    }
}

fn residence(c: &mut SchellingConfig) {
    c.residence = Residence {
        enabled: true,
        min: 80,
        max: 100,
    };
}

/// Animations VI-4 to VI-7.
pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "vi-4-schelling-25",
            "Schelling segregation, 25% like neighbors",
            "Animation VI-4",
            "2000 Red and Blue agents on a 50×50 torus, each wanting at least 25% of its von Neumann neighbors to share its color; an unsatisfied agent moves to a random site where it would be satisfied. Within a few ticks nobody moves, and the pattern is clearly more segregated than the start (segregation about 0.50 → 0.63).",
            |_| {},
        ),
        preset(
            "vi-5-schelling-25-residence",
            "Schelling segregation, 25%, residence 80–100",
            "Animation VI-5",
            "As VI-4, but each agent leaves after 80–100 ticks and is replaced by a new agent of random color placed where it is satisfied: the pattern never settles. Its segregation climbs to about 0.76, somewhat above VI-4's (the book calls the two comparable).",
            residence,
        ),
        preset(
            "vi-6-schelling-50-residence",
            "Schelling segregation, 50%, residence 80–100",
            "Animation VI-6",
            "As VI-5 with every agent wanting at least half of its neighbors to share its color: far more segregated (about 0.95).",
            |c| {
                residence(c);
                c.preference = FRange { min: 0.5, max: 0.5 };
            },
        ),
        preset(
            "vi-7-schelling-mixed",
            "Schelling segregation, 25–50% mixed, residence 80–100",
            "Animation VI-7",
            "As VI-5 with preferences spread uniformly between 25% and 50%: the tolerant agents are not enough to undo the segregation, which stays high (about 0.93), close to VI-6's.",
            |c| {
                residence(c);
                c.preference = FRange {
                    min: 0.25,
                    max: 0.5,
                };
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `w` × `h` world with nobody on it (agents are placed by `put`).
    fn empty(w: u32, h: u32) -> SchellingWorld {
        SchellingWorld::new(
            SchellingConfig {
                width: w,
                height: h,
                population: 0,
                ..SchellingConfig::default()
            },
            1,
        )
        .unwrap()
    }

    fn put(w: &mut SchellingWorld, x: u32, y: u32, red: bool, preference: f64) -> u64 {
        let mut a = w.newcomer(Pos::new(x, y));
        a.red = red;
        a.preference = preference;
        w.insert(a);
        a.id
    }

    fn agent(w: &SchellingWorld, id: u64) -> Resident {
        w.agents[&id]
    }

    /// The reference the pools must match: every empty site (in site order)
    /// where an agent of color `red` with `preference` would be satisfied,
    /// counted from the grid; `skip` (its own site) is neither a neighbor
    /// nor an option.
    fn brute(w: &SchellingWorld, red: bool, preference: f64, skip: Option<Pos>) -> Vec<Pos> {
        (0..w.torus.len())
            .map(|i| w.torus.pos(i))
            .filter(|&p| w.agent_at(p).is_none() && Some(p) != skip)
            .filter(|&p| {
                let (mut like, mut occupied) = (0, 0);
                for n in w.torus.neighbors(p) {
                    if Some(n) == skip {
                        continue;
                    }
                    if let Some(other) = w.agent_at(n) {
                        occupied += 1;
                        like += u32::from(other.red == red);
                    }
                }
                satisfied(like, occupied, preference)
            })
            .collect()
    }

    /// The filed sites an agent of color `red` with `preference` may pick, in site order.
    fn pooled(w: &SchellingWorld, red: bool, preference: f64) -> Vec<Pos> {
        let ok = acceptable_classes(preference);
        let mut out: Vec<Pos> = (0..CLASSES)
            .filter(|&k| ok[k])
            .flat_map(|k| {
                w.pools[usize::from(red)][k]
                    .iter()
                    .map(|&i| w.torus.pos(i as usize))
            })
            .collect();
        out.sort_by_key(|p| w.torus.index(*p));
        out
    }

    #[test]
    fn the_pools_match_a_direct_count_as_agents_move_and_leave() {
        let mut c = SchellingConfig {
            width: 12,
            height: 9,
            population: 80,
            preference: FRange { min: 0.2, max: 0.8 },
            ..SchellingConfig::default()
        };
        c.residence = Residence {
            enabled: true,
            min: 2,
            max: 6,
        };
        let mut w = SchellingWorld::new(c, 11).unwrap();
        for _ in 0..25 {
            for red in [false, true] {
                for p in [0.0, 0.25, 1.0 / 3.0, 0.5, 0.75, 1.0] {
                    assert_eq!(
                        pooled(&w, red, p),
                        brute(&w, red, p, None),
                        "t={} red={red} p={p}",
                        w.tick
                    );
                }
            }
            for a in w.agents() {
                let (like, occupied) = w.neighbors(a);
                let direct = w
                    .torus
                    .neighbors(a.pos)
                    .into_iter()
                    .filter_map(|n| w.agent_at(n))
                    .fold((0, 0), |(l, o), b| (l + u32::from(b.red == a.red), o + 1));
                assert_eq!((like, occupied), direct);
            }
            w.step();
        }
        // While an agent considers moving its site is vacated and unfiled:
        // the pools then hold exactly the sites acceptable without counting it.
        let a = *w.agents().next().unwrap();
        let from = w.torus.index(a.pos);
        w.vacate(from, a.red);
        for p in [0.25, 0.5, 1.0] {
            assert_eq!(pooled(&w, a.red, p), brute(&w, a.red, p, Some(a.pos)));
        }
    }

    #[test]
    fn satisfaction_counts_occupied_neighbors_and_ties_satisfy() {
        assert!(satisfied(0, 0, 1.0), "no neighbors: satisfied");
        assert!(satisfied(1, 4, 0.25), "exactly at the threshold");
        assert!(!satisfied(0, 4, 0.25));
        assert!(satisfied(2, 4, 0.5) && !satisfied(1, 3, 0.5));
        let mut w = empty(10, 10);
        let me = put(&mut w, 5, 5, true, 0.5);
        put(&mut w, 5, 4, false, 0.5);
        put(&mut w, 6, 6, false, 0.5); // diagonal: not a neighbor
        assert!(!w.is_satisfied(&agent(&w, me)), "0 of 1 alike");
        put(&mut w, 4, 5, true, 0.5);
        assert!(w.is_satisfied(&agent(&w, me)), "1 of 2 alike meets 50%");
    }

    #[test]
    fn acceptable_sites_do_not_count_the_movers_own_site() {
        // A Red wanting 50% at (2, 2) and a Blue at (4, 2). At (3, 2) its own
        // site would be a like neighbor (1 of 2); not counted, it has 0 of 1.
        let mut w = empty(7, 7);
        let me = put(&mut w, 2, 2, true, 0.5);
        put(&mut w, 4, 2, false, 0.0);
        let a = agent(&w, me);
        assert!(!brute(&w, a.red, a.preference, Some(a.pos)).contains(&Pos::new(3, 2)));
        assert!(brute(&w, a.red, a.preference, None).contains(&Pos::new(3, 2)));
        let options = brute(&w, a.red, a.preference, Some(a.pos));
        assert!(
            !options.contains(&Pos::new(2, 2)),
            "occupied sites are never options"
        );
        assert!(
            options.contains(&Pos::new(0, 0)),
            "a site with no neighbors is acceptable"
        );
    }

    #[test]
    fn an_unsatisfied_agent_with_no_acceptable_site_stays() {
        // 5 × 5 of Blues but a Red wanting 100% at (2, 2) and three empty
        // sites, each next to Blues only.
        let mut w = empty(5, 5);
        let mut me = 0;
        for y in 0..5 {
            for x in 0..5 {
                match (x, y) {
                    (2, 2) => me = put(&mut w, x, y, true, 1.0),
                    (0, 0) | (4, 4) | (0, 4) => {}
                    _ => {
                        put(&mut w, x, y, false, 0.0);
                    }
                }
            }
        }
        assert!(!w.is_satisfied(&agent(&w, me)));
        assert!(brute(&w, true, 1.0, Some(Pos::new(2, 2))).is_empty());
        w.step();
        assert_eq!(agent(&w, me).pos, Pos::new(2, 2));
        let s = w.stats.latest().unwrap();
        assert_eq!((s.moves, s.quiet), (0, 1));
    }

    #[test]
    fn a_mover_picks_among_every_acceptable_site_at_random() {
        let mut seen = std::collections::BTreeSet::new();
        for seed in 1..=40 {
            let mut w = empty(6, 6);
            w.rng = rng::seeded(seed);
            let me = put(&mut w, 0, 0, true, 1.0);
            put(&mut w, 1, 0, false, 0.0);
            w.step();
            seen.insert(agent(&w, me).pos);
        }
        // Any empty site not next to the Blue at (1, 0) is acceptable: many are chosen.
        assert!(seen.len() > 10, "{seen:?}");
        for p in seen {
            assert!(
                ![
                    Pos::new(0, 0),
                    Pos::new(2, 0),
                    Pos::new(1, 1),
                    Pos::new(1, 5)
                ]
                .contains(&p),
                "{p:?}"
            );
        }
    }

    #[test]
    fn residence_replaces_agents_and_keeps_the_population() {
        let mut c = SchellingConfig {
            width: 20,
            height: 20,
            population: 300,
            ..SchellingConfig::default()
        };
        c.residence = Residence {
            enabled: true,
            min: 3,
            max: 5,
        };
        let mut w = SchellingWorld::new(c, 7).unwrap();
        let first: Vec<u64> = w.agents().map(|a| a.id).collect();
        assert!(w
            .agents()
            .all(|a| (3..=5).contains(&a.residence) && a.age == 0));
        w.run(5);
        assert_eq!(w.agents().count(), 300);
        assert!(
            w.agents().all(|a| !first.contains(&a.id)),
            "everyone left by t = 5"
        );
        assert!(w.agents().all(|a| a.age < a.residence));
        let mut sites: Vec<Pos> = w.agents().map(|a| a.pos).collect();
        sites.sort();
        sites.dedup();
        assert_eq!(sites.len(), 300, "one agent per site");
        for (i, slot) in w.grid.iter().enumerate() {
            if let Some(id) = slot {
                assert_eq!(w.agents[id].pos, w.torus.pos(i));
            }
        }
    }

    #[test]
    fn a_newcomer_is_placed_where_it_is_satisfied_when_possible() {
        let mut w = empty(5, 5);
        let old = put(&mut w, 2, 2, true, 1.0);
        for (x, y) in [(0, 0), (4, 4)] {
            put(&mut w, x, y, false, 0.0);
        }
        w.agents.get_mut(&old).unwrap().residence = 1;
        w.config.residence = Residence {
            enabled: true,
            min: 5,
            max: 5,
        };
        w.config.preference = FRange { min: 1.0, max: 1.0 };
        w.step();
        assert!(!w.agents.contains_key(&old), "it left at age 1");
        let new = w.agents().find(|a| a.preference == 1.0).unwrap();
        assert!(w.is_satisfied(new), "placed where it is satisfied");
        assert_eq!((new.age, new.residence), (0, 5));
        assert_eq!(w.agents().count(), 3);
    }

    #[test]
    fn a_newcomer_always_finds_a_site() {
        // 5 × 5 of Blues but one empty site; the leaver's site frees another.
        let mut w = empty(5, 5);
        let mut leaver = 0;
        for y in 0..5 {
            for x in 0..5 {
                if (x, y) == (0, 0) {
                    continue;
                }
                let id = put(&mut w, x, y, false, 0.0);
                if (x, y) == (3, 3) {
                    leaver = id;
                }
            }
        }
        w.agents.get_mut(&leaver).unwrap().residence = 1;
        w.config.residence = Residence {
            enabled: true,
            min: 9,
            max: 9,
        };
        w.config.preference = FRange { min: 1.0, max: 1.0 };
        w.step();
        assert_eq!(w.agents().count(), 24);
        assert!(!w.agents.contains_key(&leaver));
        let empty: Vec<usize> = (0..25).filter(|&i| w.grid[i].is_none()).collect();
        assert_eq!(empty.len(), 1);
    }

    #[test]
    fn statistics_follow_the_definitions() {
        let mut w = empty(10, 10);
        let a = put(&mut w, 1, 1, true, 0.5);
        put(&mut w, 1, 2, true, 0.5);
        put(&mut w, 2, 1, false, 0.5);
        put(&mut w, 8, 8, false, 0.9); // no neighbors: satisfied, not in segregation
        let s = w.snapshot();
        // a: 1 of 2 alike; (1,2): 1 of 1; (2,1): 0 of 1 (unsatisfied).
        assert_eq!(s.population, 4);
        assert_eq!(s.unsatisfied, 0.25);
        assert!((s.segregation - (0.5 + 1.0 + 0.0) / 3.0).abs() < 1e-12);
        assert_eq!(s.red_share, 0.5);
        assert_eq!((s.moves, s.quiet), (0, 0), "t = 0 is not quiet");
        assert!(w.is_satisfied(&agent(&w, a)));
    }

    #[test]
    fn setup_places_the_population_on_distinct_sites_with_both_colors() {
        let w = SchellingWorld::new(SchellingConfig::default(), 3).unwrap();
        assert_eq!(w.agents().count(), 2000);
        let reds = w.agents().filter(|a| a.red).count();
        assert!((900..=1100).contains(&reds), "{reds}");
        assert!(w.agents().all(|a| a.preference == 0.25 && a.residence == 0));
        let mixed = SchellingWorld::new(
            SchellingConfig {
                preference: FRange {
                    min: 0.25,
                    max: 0.5,
                },
                ..SchellingConfig::default()
            },
            3,
        )
        .unwrap();
        assert!(mixed.agents().all(|a| (0.25..=0.5).contains(&a.preference)));
        assert!(
            mixed.agents().any(|a| a.preference > 0.4)
                && mixed.agents().any(|a| a.preference < 0.3)
        );
    }

    #[test]
    fn validation_names_fields() {
        let bad = SchellingConfig {
            width: 4,
            population: 2500,
            preference: FRange { min: 0.6, max: 0.5 },
            residence: Residence {
                enabled: true,
                min: 0,
                max: 5,
            },
            ..SchellingConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["width", "population", "preference", "residence.min"]
        );
    }

    #[test]
    fn renders_colors_satisfaction_and_preference() {
        let mut w = empty(5, 5);
        put(&mut w, 0, 0, true, 0.25);
        put(&mut w, 1, 0, false, 1.0);
        let mut buf = Vec::new();
        let pixel = |buf: &[u8], i: usize| [buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2]];
        w.render("color", "", &mut buf).unwrap();
        assert_eq!(
            (pixel(&buf, 0), pixel(&buf, 1), pixel(&buf, 2)),
            (RED, BLUE, BACKGROUND)
        );
        w.render("satisfaction", "", &mut buf).unwrap();
        assert_eq!(pixel(&buf, 0), BOTH, "0 of 1 alike < 25%: unsatisfied");
        assert_eq!(pixel(&buf, 1), BOTH);
        w.render("preference", "", &mut buf).unwrap();
        assert_eq!(pixel(&buf, 1), HOT);
        assert!(w.render("tribe", "", &mut buf).is_err());
    }

    #[test]
    fn set_config_refuses_every_change() {
        let mut w = SchellingWorld::new(SchellingConfig::default(), 1).unwrap();
        let same = ModelConfig::Schelling(SchellingConfig::default());
        assert!(Model::set_config(&mut w, same).is_ok());
        let next = SchellingConfig {
            population: 10,
            ..SchellingConfig::default()
        };
        let e = Model::set_config(&mut w, ModelConfig::Schelling(next)).unwrap_err();
        assert_eq!(e[0].field, "population");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Schelling(SchellingConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }

    #[test]
    fn f_range_denies_unknown_fields_and_takes_defaults_when_partial() {
        let partial: FRange = serde_json::from_str(r#"{"min": 0.1}"#).unwrap();
        assert_eq!(
            partial,
            FRange {
                min: 0.1,
                max: 0.25
            },
            "max takes the default"
        );
        let err = serde_json::from_str::<FRange>(r#"{"min": 0.1, "mzx": 0.9}"#).unwrap_err();
        assert!(err.to_string().contains("mzx"), "{err}");
    }

    #[test]
    fn residence_denies_unknown_fields_and_takes_defaults_when_partial() {
        let partial: Residence = serde_json::from_str(r#"{"enabled": true}"#).unwrap();
        assert_eq!(
            partial,
            Residence {
                enabled: true,
                min: 80,
                max: 100
            },
            "min and max take their defaults"
        );
        let err = serde_json::from_str::<Residence>(r#"{"enable": true}"#).unwrap_err();
        assert!(err.to_string().contains("enable"), "{err}");
    }

    #[test]
    fn presets_follow_the_book() {
        let ps = presets();
        let get = |id: &str| match &ps.iter().find(|p| p.id == id).unwrap().config {
            ModelConfig::Schelling(c) => c.clone(),
            _ => unreachable!(),
        };
        let vi4 = get("vi-4-schelling-25");
        assert_eq!((vi4.width, vi4.height, vi4.population), (50, 50, 2000));
        assert_eq!(
            vi4.preference,
            FRange {
                min: 0.25,
                max: 0.25
            }
        );
        assert!(!vi4.residence.enabled);
        let vi5 = get("vi-5-schelling-25-residence");
        assert_eq!((vi5.residence.min, vi5.residence.max), (80, 100));
        assert!(vi5.residence.enabled);
        assert_eq!(
            get("vi-6-schelling-50-residence").preference,
            FRange { min: 0.5, max: 0.5 }
        );
        assert_eq!(
            get("vi-7-schelling-mixed").preference,
            FRange {
                min: 0.25,
                max: 0.5
            }
        );
        for p in &ps {
            p.config.validate().unwrap();
        }
    }
}
