//! Ring World (Chapter VI, animations VI-8 and VI-9): sugar harvesters on a
//! circle of sites who look only counterclockwise, move to the nearest best
//! site they see and eat its sugar, and fall into flocks.

use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::config::{FieldError, URange};
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::presets::ModelPreset;
use crate::render::{lerp, Rgb, BACKGROUND, SUGAR};
use crate::rng::{self, SimRng};
use crate::schema::{Apply, Param};
use crate::stats::{Series, Stats};

/// Rows of the space–time diagram: the last this many ticks.
pub const HISTORY: usize = 150;
/// An agent in the space–time diagram: the ramp's cool blue, which stands
/// out against both dark and full sites.
pub const AGENT: Rgb = [0x4f, 0x9d, 0xff];

/// How the agents start: scattered, or VI-9's "one megagroup".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// On random distinct sites.
    Random,
    /// On consecutive sites from a random site.
    Megagroup,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RingConfig {
    pub sites: u32,
    pub agents: u32,
    /// Each agent's vision, uniform in `[min, max]`.
    pub vision: URange,
    /// Sugar a site holds at most; initial sugar is uniform in `0..=capacity`.
    pub capacity: u32,
    /// Sugar every site grows back each tick, up to `capacity`.
    pub growback: f64,
    pub start: Start,
}

impl Default for RingConfig {
    /// Animation VI-8: "there are 150 sites. Initially, the sugar level is
    /// distributed randomly between values of 0 and 4, and 40 agents are
    /// distributed randomly around the ring … vision randomly chosen from
    /// some range (15 to 30) … sugar grows back at unit rate to a capacity
    /// value, which is 4".
    fn default() -> Self {
        RingConfig {
            sites: 150,
            agents: 40,
            vision: URange::new(15, 30),
            capacity: 4,
            growback: 1.0,
            start: Start::Random,
        }
    }
}

impl RingConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (10..=1000).contains(&self.sites),
            "sites",
            "must be between 10 and 1000",
        );
        check(
            self.agents < self.sites,
            "agents",
            "must be less than the number of sites",
        );
        let v = self.vision;
        check(v.min >= 1, "vision.min", "must be ≥ 1");
        check(v.min <= v.max, "vision", "min must be ≤ max");
        check(
            v.max < self.sites,
            "vision.max",
            "must be less than the number of sites",
        );
        check(
            (1..=100).contains(&self.capacity),
            "capacity",
            "must be between 1 and 100",
        );
        check(
            self.growback.is_finite() && self.growback > 0.0,
            "growback",
            "must be a number > 0",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next` (capacity and
    /// growback apply to the running world).
    fn structural_changes(&self, next: &RingConfig) -> Vec<FieldError> {
        let msg = "changes only on reset";
        let mut out = Vec::new();
        for (field, same) in [
            ("sites", self.sites == next.sites),
            ("agents", self.agents == next.agents),
            ("vision", self.vision == next.vision),
            ("start", self.start == next.start),
        ] {
            if !same {
                out.push(FieldError::new(field, msg));
            }
        }
        out
    }
}

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 5] = [
    "flocks",
    "mean_flock",
    "largest_flock",
    "mean_distance",
    "population",
];

/// One tick's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct RingSnapshot {
    pub tick: u64,
    pub population: u32,
    /// Flocks (see `flocks`); 0 with no agents.
    pub flocks: u32,
    /// Agents per flock (0 with no agents).
    pub mean_flock: f64,
    pub largest_flock: u32,
    /// Sites moved per agent this tick (0 at t = 0).
    pub mean_distance: f64,
}

impl Series for RingSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "flocks" => f64::from(self.flocks),
            "mean_flock" => self.mean_flock,
            "largest_flock" => f64::from(self.largest_flock),
            "mean_distance" => self.mean_distance,
            _ => return None,
        })
    }
}

/// One harvester: its site and vision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Walker {
    pub id: u64,
    pub site: u32,
    pub vision: u32,
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RingInspection {
    pub site: RingSiteView,
    pub agent: Option<WalkerView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct RingSiteView {
    /// The site's index (counterclockwise from 0).
    pub x: u32,
    pub sugar: f64,
    pub capacity: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct WalkerView {
    pub id: u64,
    pub vision: u32,
}

/// The sizes of the flocks formed by agents at `sites` on a ring of `n`
/// sites, starting after the widest-apart pair: a flock is a maximal run of
/// agents (in ring order) whose consecutive gaps are at most 2 sites — at
/// most one empty site between neighbors. (The book does not define a
/// flock; the spec states this one.)
pub fn flocks(sites: &[u32], n: u32) -> Vec<u32> {
    let mut sorted = sites.to_vec();
    sorted.sort_unstable();
    let k = sorted.len();
    if k == 0 {
        return Vec::new();
    }
    // The gap after agent i, to the next around the ring.
    let gap = |i: usize| (sorted[(i + 1) % k] + n - sorted[i]) % n;
    let gaps: Vec<u32> = (0..k).map(|i| if k == 1 { n } else { gap(i) }).collect();
    let Some(first_break) = gaps.iter().position(|&g| g > 2) else {
        return vec![k as u32];
    };
    let mut out = Vec::new();
    let mut size = 0;
    for step in 1..=k {
        let i = (first_break + step) % k;
        size += 1;
        if gaps[i] > 2 {
            out.push(size);
            size = 0;
        }
    }
    out
}

pub struct RingWorld {
    pub config: RingConfig,
    /// Completed ticks.
    pub tick: u64,
    sugar: Vec<f64>,
    occupant: Vec<Option<u64>>,
    agents: BTreeMap<u64, Walker>,
    rng: SimRng,
    /// Sites moved by all agents this tick.
    distance: u64,
    /// The space–time diagram's rows (oldest first, at most `HISTORY`): each
    /// site's sugar as a fraction of capacity, or `None` where an agent stood.
    history: VecDeque<Vec<Option<f32>>>,
    pub stats: Stats<RingSnapshot>,
}

impl RingWorld {
    pub fn new(config: RingConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let n = config.sites as usize;
        let mut rng = rng::seeded(seed);
        let sugar = (0..n)
            .map(|_| f64::from(rng.gen_range(0..=config.capacity)))
            .collect();
        let places: Vec<u32> = match config.start {
            Start::Random => {
                let mut all: Vec<u32> = (0..config.sites).collect();
                all.shuffle(&mut rng);
                all.truncate(config.agents as usize);
                all
            }
            Start::Megagroup => {
                let from = rng.gen_range(0..config.sites);
                (0..config.agents)
                    .map(|k| (from + k) % config.sites)
                    .collect()
            }
        };
        let mut world = RingWorld {
            tick: 0,
            sugar,
            occupant: vec![None; n],
            agents: BTreeMap::new(),
            rng,
            distance: 0,
            history: VecDeque::with_capacity(HISTORY),
            stats: Stats::default(),
            config,
        };
        for (k, site) in places.into_iter().enumerate() {
            let id = k as u64 + 1;
            let vision = world.config.vision.sample(&mut world.rng);
            world.occupant[site as usize] = Some(id);
            world.agents.insert(id, Walker { id, site, vision });
        }
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> impl Iterator<Item = &Walker> {
        self.agents.values()
    }

    /// Each site's sugar, site 0 first.
    pub fn sugar(&self) -> &[f64] {
        &self.sugar
    }

    /// Each agent's site, in id order.
    pub fn agent_sites(&self) -> Vec<u32> {
        self.agents.values().map(|a| a.site).collect()
    }

    /// Where agent `a` goes: among the unoccupied sites at distances
    /// 1…vision counterclockwise (increasing index), the nearest with the
    /// most sugar (even if that is 0), as (site, distance); `None` if every
    /// one is occupied.
    fn target(&self, a: &Walker) -> Option<(u32, u32)> {
        let n = self.config.sites;
        let mut best: Option<(u32, u32, f64)> = None;
        for d in 1..=a.vision {
            let s = (a.site + d) % n;
            if self.occupant[s as usize].is_some() {
                continue;
            }
            let sugar = self.sugar[s as usize];
            if best.is_none_or(|(_, _, b)| sugar > b) {
                best = Some((s, d, sugar));
            }
        }
        best.map(|(s, d, _)| (s, d))
    }

    /// One tick: agents act in a random order — "Inspect all unoccupied sites
    /// within your vision, select the nearest site with maximum sugar, go
    /// there and eat the sugar" — then every site grows back.
    pub fn step(&mut self) {
        let mut order: Vec<u64> = self.agents.keys().copied().collect();
        order.shuffle(&mut self.rng);
        let mut distance = 0;
        for id in order {
            let a = self.agents[&id];
            let Some((to, d)) = self.target(&a) else {
                continue;
            };
            self.occupant[a.site as usize] = None;
            self.occupant[to as usize] = Some(id);
            self.agents.get_mut(&id).expect("living agent").site = to;
            self.sugar[to as usize] = 0.0;
            distance += u64::from(d);
        }
        let (rate, cap) = (self.config.growback, f64::from(self.config.capacity));
        for s in &mut self.sugar {
            *s = (*s + rate).min(cap);
        }
        self.distance = distance;
        self.tick += 1;
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    /// Pushes this tick's statistics and space–time row.
    fn record(&mut self) {
        let cap = f64::from(self.config.capacity);
        let row = self
            .sugar
            .iter()
            .zip(&self.occupant)
            .map(|(&s, o)| o.is_none().then_some((s / cap) as f32))
            .collect();
        if self.history.len() == HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(row);
        let snapshot = self.snapshot();
        self.stats.push(snapshot);
    }

    fn snapshot(&self) -> RingSnapshot {
        let n = self.agents.len();
        let sizes = flocks(&self.agent_sites(), self.config.sites);
        RingSnapshot {
            tick: self.tick,
            population: n as u32,
            flocks: sizes.len() as u32,
            mean_flock: if sizes.is_empty() {
                0.0
            } else {
                n as f64 / sizes.len() as f64
            },
            largest_flock: sizes.iter().copied().max().unwrap_or(0),
            mean_distance: if n == 0 {
                0.0
            } else {
                self.distance as f64 / n as f64
            },
        }
    }

    pub fn inspect(&self, x: u32) -> Result<RingInspection, String> {
        if x >= self.config.sites {
            return Err(format!("site {x} is not on the ring"));
        }
        Ok(RingInspection {
            site: RingSiteView {
                x,
                sugar: self.sugar[x as usize],
                capacity: self.config.capacity,
            },
            agent: self.occupant[x as usize].map(|id| {
                let a = self.agents[&id];
                WalkerView {
                    id,
                    vision: a.vision,
                }
            }),
        })
    }
}

impl Model for RingWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Ring(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        RingWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick, every site's sugar, and every agent's id, site
    /// and vision.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for s in &self.sugar {
            eat(s.to_bits());
        }
        for a in self.agents.values() {
            eat(a.id);
            eat((u64::from(a.site) << 32) | u64::from(a.vision));
        }
        h
    }

    /// The space–time diagram: one column per site, one row per tick, the
    /// current tick at the bottom.
    fn size(&self) -> (u32, u32) {
        (self.config.sites, HISTORY as u32)
    }

    /// Sugar shaded from dark to the sugar color, agents light; rows before
    /// t = 0 dark. `mode` and `layer` are ignored.
    fn render(&self, _mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let n = self.config.sites as usize;
        buf.clear();
        buf.resize(n * HISTORY * 4, 0);
        let blank = HISTORY - self.history.len();
        for (y, px) in buf.chunks_exact_mut(n * 4).enumerate() {
            let row = y.checked_sub(blank).map(|r| &self.history[r]);
            for (x, p) in px.chunks_exact_mut(4).enumerate() {
                let rgb = match row.map(|r| r[x]) {
                    None => BACKGROUND,
                    Some(None) => AGENT,
                    Some(Some(level)) => lerp(BACKGROUND, SUGAR, f64::from(level)),
                };
                p.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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
        let mut out = String::from("id,site,vision\n");
        for a in self.agents.values() {
            writeln!(out, "{},{},{}", a.id, a.site, a.vision).unwrap();
        }
        out
    }

    /// Site `x` (`y`, a row of the diagram, is ignored).
    fn inspect_json(&self, x: u32, _y: u32) -> Result<String, String> {
        let inspection = self.inspect(x)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// The agent's site, in the diagram's current (bottom) row.
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        self.agents.get(&id).map(|a| (a.site, HISTORY as u32 - 1))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Ring(next) = next else {
            return Err(wrong_model(ModelKind::Ring, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }
}

/// The Rules panel's fields: capacity and growback apply to the running
/// world; the rest rebuild it.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Setup", "sites", "Sites", (10, 1000), Reset),
        Param::integer("Setup", "agents", "Agents", (1, 999), Reset),
        Param::choice(
            "Setup",
            "start",
            "Start",
            &[
                ("random", "Scattered at random"),
                ("megagroup", "One megagroup"),
            ],
            Reset,
        ),
        Param::range(
            "Agents",
            "vision",
            "Vision (sites)",
            (1.0, 100.0, 1.0),
            Reset,
        ),
        Param::integer("Sugar", "capacity", "Capacity", (1, 20), Live),
        Param::number(
            "Sugar",
            "growback",
            "Growback per tick",
            (0.1, 10.0, 0.1),
            Live,
        ),
    ]
}

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    start: Start,
) -> ModelPreset {
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Ring(RingConfig {
            start,
            ..RingConfig::default()
        }),
    }
}

/// Animations VI-8 and VI-9.
pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "vi-8-ring-world",
            "Ring World: flocking on the ring",
            "Animation VI-8",
            "40 agents with vision 15–30 on a ring of 150 sugar sites (0–4 sugar, growing back 1 per tick) look only counterclockwise, jump to the nearest richest empty site they see and eat it. With no social rule at all they fall into flocks that tread around the ring.",
            Start::Random,
        ),
        preset(
            "vi-9-ring-megagroup",
            "Ring World: a megagroup breaks up",
            "Animation VI-9",
            "The same ring with all 40 agents starting as one megagroup on consecutive sites: nobody can see past the front of a group of 40, so it breaks up into smaller flocks.",
            Start::Megagroup,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A ring of `n` sites with sugar `sugar` everywhere and agents
    /// `(site, vision)` (ids 1, 2, … in that order).
    fn ring(n: u32, sugar: f64, agents: &[(u32, u32)]) -> RingWorld {
        let mut w = RingWorld::new(
            RingConfig {
                sites: n,
                agents: 0,
                vision: URange::new(1, 1),
                ..RingConfig::default()
            },
            1,
        )
        .unwrap();
        w.sugar = vec![sugar; n as usize];
        for (k, &(site, vision)) in agents.iter().enumerate() {
            let id = k as u64 + 1;
            w.occupant[site as usize] = Some(id);
            w.agents.insert(id, Walker { id, site, vision });
        }
        w
    }

    fn site(w: &RingWorld, id: u64) -> u32 {
        w.agents[&id].site
    }

    #[test]
    fn agents_look_counterclockwise_and_wrap() {
        let mut w = ring(12, 0.0, &[(10, 3)]);
        w.sugar[1] = 4.0; // distance 3 across the wrap
        w.sugar[9] = 4.0; // behind it: never seen
        assert_eq!(w.target(&w.agents[&1]), Some((1, 3)));
    }

    #[test]
    fn the_nearest_site_with_the_most_sugar_wins() {
        let mut w = ring(20, 1.0, &[(0, 6)]);
        w.sugar[3] = 3.0;
        w.sugar[5] = 3.0;
        w.sugar[6] = 2.0;
        assert_eq!(
            w.target(&w.agents[&1]),
            Some((3, 3)),
            "nearest of the two 3s"
        );
        let flat = ring(20, 0.0, &[(0, 6)]);
        assert_eq!(
            flat.target(&flat.agents[&1]),
            Some((1, 1)),
            "all zero: the nearest"
        );
    }

    #[test]
    fn occupied_sites_are_skipped_and_a_blocked_agent_stays() {
        let mut w = ring(20, 1.0, &[(0, 3), (1, 5), (3, 5)]);
        w.sugar[1] = 4.0;
        assert_eq!(w.target(&w.agents[&1]), Some((2, 2)), "site 1 is occupied");
        let blocked = ring(20, 4.0, &[(0, 2), (1, 5), (2, 5)]);
        assert_eq!(blocked.target(&blocked.agents[&1]), None);
    }

    #[test]
    fn movers_eat_and_sites_grow_back_after_everyone_moved() {
        let mut w = ring(20, 0.0, &[(0, 3)]);
        w.sugar[2] = 4.0;
        w.config.growback = 1.0;
        w.step();
        assert_eq!(site(&w, 1), 2);
        assert_eq!(w.sugar[2], 1.0, "eaten to 0, then grew back 1");
        assert_eq!(w.sugar[5], 1.0);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.mean_distance, s.population), (2.0, 1));
        w.config.capacity = 1;
        w.run(3);
        assert!(w.sugar.iter().all(|&s| s <= 1.0), "capped at capacity");
    }

    #[test]
    fn a_follower_leapfrogs_its_leader() {
        // The book's two-agent analysis: with all sugar at 4 the follower
        // jumps to the site just in front of the leader.
        let mut w = ring(30, 4.0, &[(5, 10), (6, 10)]);
        w.config.growback = 4.0;
        for _ in 0..20 {
            w.step();
            let (a, b) = (site(&w, 1), site(&w, 2));
            assert!((a + 30 - b) % 30 == 1 || (b + 30 - a) % 30 == 1, "{a} {b}");
        }
    }

    #[test]
    fn flocks_split_at_gaps_of_more_than_two_sites_across_the_wrap() {
        assert_eq!(flocks(&[], 10), Vec::<u32>::new());
        assert_eq!(flocks(&[4], 10), vec![1]);
        assert_eq!(flocks(&[0, 1, 3], 10), vec![3], "gaps of 1 and 2");
        assert_eq!(flocks(&[0, 1, 4], 10), vec![1, 2], "a gap of 3 splits");
        // 9 → 0 wraps: 8, 9, 0, 1 is one flock; 5 is alone.
        let mut sizes = flocks(&[0, 1, 5, 8, 9], 10);
        sizes.sort_unstable();
        assert_eq!(sizes, vec![1, 4]);
        assert_eq!(flocks(&[0, 2, 4, 6, 8], 10), vec![5], "every gap is 2");
    }

    #[test]
    fn statistics_count_flocks() {
        let w = ring(
            30,
            0.0,
            &[(0, 1), (1, 1), (2, 1), (10, 1), (20, 1), (21, 1)],
        );
        let s = w.snapshot();
        assert_eq!((s.flocks, s.largest_flock, s.population), (3, 3, 6));
        assert_eq!(s.mean_flock, 2.0);
    }

    #[test]
    fn setup_scatters_or_masses_the_agents() {
        let random = RingWorld::new(RingConfig::default(), 2).unwrap();
        assert_eq!(random.agents().count(), 40);
        assert!(random.agents().all(|a| (15..=30).contains(&a.vision)));
        assert!(random
            .sugar
            .iter()
            .all(|&s| s.fract() == 0.0 && (0.0..=4.0).contains(&s)));
        assert!(random.sugar.contains(&0.0) && random.sugar.contains(&4.0));
        let mega = RingWorld::new(
            RingConfig {
                start: Start::Megagroup,
                ..RingConfig::default()
            },
            2,
        )
        .unwrap();
        assert_eq!(flocks(&mega.agent_sites(), 150), vec![40]);
        let mut sites = mega.agent_sites();
        sites.sort_unstable();
        let gaps = sites.windows(2).filter(|p| p[1] - p[0] != 1).count();
        assert!(gaps <= 1, "consecutive sites, possibly across the wrap");
        assert_eq!(mega.stats.latest().unwrap().flocks, 1);
    }

    #[test]
    fn the_space_time_diagram_keeps_the_last_150_ticks() {
        let mut w = RingWorld::new(RingConfig::default(), 3).unwrap();
        let mut buf = Vec::new();
        w.render("", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 150 * 150 * 4);
        assert_eq!(&buf[..3], &BACKGROUND, "rows before t = 0 are dark");
        let bottom = &buf[149 * 150 * 4..];
        let agents = bottom.chunks_exact(4).filter(|p| p[..3] == AGENT).count();
        assert_eq!(agents, 40, "the current tick is the bottom row");
        w.run(200);
        assert_eq!(w.history.len(), HISTORY);
        w.render("", "", &mut buf).unwrap();
        let top = &buf[..150 * 4];
        assert_eq!(
            top.chunks_exact(4).filter(|p| p[..3] == AGENT).count(),
            40,
            "full: no dark rows"
        );
        assert_eq!(Model::locate(&w, 1).unwrap().1, 149);
    }

    #[test]
    fn validation_names_fields() {
        let bad = RingConfig {
            sites: 5,
            agents: 5,
            vision: URange::new(0, 9),
            capacity: 0,
            growback: 0.0,
            start: Start::Random,
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            [
                "sites",
                "agents",
                "vision.min",
                "vision.max",
                "capacity",
                "growback"
            ]
        );
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Ring(RingConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }

    #[test]
    fn presets_follow_the_book() {
        let ps = presets();
        let configs: Vec<RingConfig> = ps
            .iter()
            .map(|p| match &p.config {
                ModelConfig::Ring(c) => c.clone(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(configs[0], RingConfig::default());
        assert_eq!(
            (
                configs[0].sites,
                configs[0].agents,
                configs[0].vision,
                configs[0].capacity
            ),
            (150, 40, URange::new(15, 30), 4)
        );
        assert_eq!(configs[1].start, Start::Megagroup);
        assert_eq!(
            RingConfig {
                start: Start::Random,
                ..configs[1].clone()
            },
            configs[0]
        );
    }
}
