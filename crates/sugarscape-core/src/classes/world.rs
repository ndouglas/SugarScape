//! The bargaining world: agents paired at random (or on a lattice) play the
//! Nash demand game, each best-replying to what it remembers of its last m
//! opponents, with an occasional random demand.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{ClassesConfig, Decision, Interaction, Layout, Neighborhood, Start, TagMemory};
use super::simplex::{self, GAP, SIDE, TALL};
use super::stats::{
    equity, expected_best, mode_best, regime, ClassesSnapshot, View, EQUITY, FRACTIOUS, H, L, M,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, COOL, HOT};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// The simplex's regions: where L, M or H is the best reply.
const L_REGION: Rgb = [0x3a, 0x38, 0x33];
const M_REGION: Rgb = [0xb8, 0xb4, 0xa8];
const H_REGION: Rgb = [0x6e, 0x6a, 0x60];
/// Dots brighten toward this where several agents coincide.
const BRIGHT: Rgb = [0xff, 0xf6, 0xe0];

/// A demand's letter.
pub fn letter(d: u8) -> char {
    ['L', 'M', 'H'][d as usize]
}

/// What an agent remembers: its last opponents' demands, each with the
/// opponent's tag (entry = demand | tag << 2), and their counts per tag.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory {
    entries: VecDeque<u8>,
    counts: [[u32; 3]; 2],
}

impl Memory {
    /// Appends a demand made by an opponent with `tag`, dropping the oldest
    /// entry once the memory holds `cap`.
    pub fn push(&mut self, demand: u8, tag: u8, cap: usize) {
        if self.entries.len() == cap {
            let old = self.entries.pop_front().expect("a full memory");
            self.counts[(old >> 2) as usize][(old & 3) as usize] -= 1;
        }
        self.entries.push_back(demand | tag << 2);
        self.counts[tag as usize][demand as usize] += 1;
    }

    /// The counts (L, M, H) of demands by opponents with `tag`.
    pub fn view(&self, tag: u8) -> View {
        self.counts[tag as usize]
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn bytes(&self) -> impl Iterator<Item = u8> + '_ {
        self.entries.iter().copied()
    }
}

/// One agent: its tag, its memories and this period's play.
#[derive(Clone, Debug, PartialEq)]
pub struct Bargainer {
    pub id: u64,
    /// 0 dark, 1 light (always 0 without tags).
    pub tag: u8,
    /// Without tags or with a shared memory only `memory[0]` is used; with
    /// one memory per tag, `memory[t]` holds what opponents of tag t did.
    pub memory: [Memory; 2],
    pub last_demand: Option<u8>,
    /// This period's payoffs and matches, and those against the other tag.
    payoff: u32,
    matches: u32,
    inter_payoff: u32,
    inter_matches: u32,
}

impl Bargainer {
    /// Mean payoff per match this period (0 without matches).
    pub fn mean_payoff(&self) -> f64 {
        if self.matches == 0 {
            0.0
        } else {
            f64::from(self.payoff) / f64::from(self.matches)
        }
    }
}

/// Every agent's lattice neighbors (empty with random pairing).
#[derive(Debug, Default, PartialEq)]
pub struct Neighbors {
    start: Vec<u32>,
    list: Vec<u32>,
}

impl Neighbors {
    fn new(c: &ClassesConfig) -> Self {
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
pub enum ClassesMode {
    /// The paper's shading of best replies, with the agents on it.
    BestReply,
    /// The agents colored by this period's mean payoff.
    Payoff,
}

impl std::str::FromStr for ClassesMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "best_reply" => Self::BestReply,
            "payoff" => Self::Payoff,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a simplex cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ClassesInspection {
    pub site: ClassesCell,
    /// `"one"` (no tags), `"intra"` or `"inter"`; null between or outside the triangles.
    pub simplex: Option<&'static str>,
    /// The mix of L, M and H remembered at this point.
    pub mix: Option<[f64; 3]>,
    /// The best reply there (`"L"`, `"M"` or `"H"`).
    pub best_reply: Option<char>,
    /// The agents whose memory plots at this cell.
    pub agents: Vec<BargainerView>,
    /// Always null: agents have no place to follow.
    pub agent: Option<BargainerView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct ClassesCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BargainerView {
    pub id: u64,
    /// `"dark"`, `"light"`, or null without tags.
    pub tag: Option<&'static str>,
    /// The memory plotted here: counts of L, M, H.
    pub memory: [u32; 3],
    pub last_demand: Option<char>,
    pub mean_payoff: f64,
}

#[derive(Clone)]
pub struct ClassesWorld {
    pub config: ClassesConfig,
    /// Completed periods.
    pub tick: u64,
    agents: Vec<Bargainer>,
    neighbors: Arc<Neighbors>,
    rng: SimRng,
    /// This period's match outcomes: both M, H against L, over 100, under 100.
    outcomes: [u32; 4],
    /// Demands this period, and those that differed from every best reply.
    demands: u32,
    errors: u32,
    equity_at: Option<u64>,
    first_attractor: u8,
    pub stats: Stats<ClassesSnapshot>,
}

impl ClassesWorld {
    pub fn new(config: ClassesConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.population() as usize;
        let tags = tag_layout(&config, &mut rng);
        let neighbors = Arc::new(match config.interaction {
            Interaction::Random => Neighbors::default(),
            Interaction::Lattice => Neighbors::new(&config),
        });
        let mut agents: Vec<Bargainer> = (0..n)
            .map(|i| Bargainer {
                id: i as u64 + 1,
                tag: tags[i],
                memory: Default::default(),
                last_demand: None,
                payoff: 0,
                matches: 0,
                inter_payoff: 0,
                inter_matches: 0,
            })
            .collect();
        let m = config.memory as usize;
        for a in &mut agents {
            if !config.tags {
                for d in start_memory(&config, a.tag, 0, m, &mut rng) {
                    a.memory[0].push(d, 0, m);
                }
            } else if config.tag_memory == TagMemory::PerTag {
                for t in 0..2 {
                    for d in start_memory(&config, a.tag, t, m, &mut rng) {
                        a.memory[t as usize].push(d, t, m);
                    }
                }
            } else {
                // One memory of the last m opponents: a random m of the 2m
                // drawn for the two tags.
                let mut entries: Vec<(u8, u8)> = Vec::new();
                for t in 0..2 {
                    entries.extend(
                        start_memory(&config, a.tag, t, m, &mut rng)
                            .into_iter()
                            .map(|d| (d, t)),
                    );
                }
                entries.shuffle(&mut rng);
                entries.truncate(m);
                for (d, t) in entries {
                    a.memory[0].push(d, t, m);
                }
            }
        }
        let mut world = ClassesWorld {
            config,
            tick: 0,
            agents,
            neighbors,
            rng,
            outcomes: [0; 4],
            demands: 0,
            errors: 0,
            equity_at: None,
            first_attractor: 0,
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> &[Bargainer] {
        &self.agents
    }

    /// Whether the run has stopped: equity reached with `stop_at_equity`.
    pub fn is_finished(&self) -> bool {
        self.config.stop_at_equity && self.equity_at.is_some()
    }

    /// The memory agent `i` consults against an opponent of tag `t`: its counts.
    pub fn view(&self, i: usize, t: u8) -> View {
        let a = &self.agents[i];
        if !self.config.tags {
            a.memory[0].view(0)
        } else if self.config.tag_memory == TagMemory::PerTag {
            a.memory[t as usize].view(t)
        } else {
            a.memory[0].view(t)
        }
    }

    fn best(&self, v: View) -> u8 {
        match self.config.decision {
            Decision::Expected => expected_best(v, self.config.low),
            Decision::Mode => mode_best(v),
        }
    }

    /// Agent `i`'s demand against an opponent of tag `t`: random with
    /// probability ε, else uniform among its best replies (all three when it
    /// remembers nothing of tag `t`).
    fn demand(&mut self, i: usize, t: u8) -> u8 {
        let best = self.best(self.view(i, t));
        self.demands += 1;
        if self.rng.gen_bool(self.config.noise) {
            let d = self.rng.gen_range(0..3u32) as u8;
            self.errors += u32::from(best & 1 << d == 0);
            return d;
        }
        let choices: Vec<u8> = (0..3).filter(|&d| best & 1 << d != 0).collect();
        if choices.len() == 1 {
            choices[0]
        } else {
            choices[self.rng.gen_range(0..choices.len() as u32) as usize]
        }
    }

    fn value(&self, d: u8) -> u32 {
        match d {
            L => self.config.low,
            M => 50,
            _ => self.config.high(),
        }
    }

    /// Agent `i` remembers that an opponent of tag `t` demanded `d`.
    fn remember(&mut self, i: usize, d: u8, t: u8) {
        let slot = if self.config.tags && self.config.tag_memory == TagMemory::PerTag {
            t
        } else {
            0
        };
        let cap = self.config.memory as usize;
        self.agents[i].memory[slot as usize].push(d, t, cap);
    }

    /// Two players for a match.
    fn pair(&mut self) -> (usize, usize) {
        let n = self.agents.len();
        let a = self.rng.gen_range(0..n as u32) as usize;
        let b = match self.config.interaction {
            Interaction::Random => {
                let b = self.rng.gen_range(0..n as u32 - 1) as usize;
                if b >= a {
                    b + 1
                } else {
                    b
                }
            }
            Interaction::Lattice => {
                let nb = self.neighbors.of(a);
                nb[self.rng.gen_range(0..nb.len() as u32) as usize] as usize
            }
        };
        (a, b)
    }

    /// One period: N/2 matches.
    pub fn step(&mut self) {
        for a in &mut self.agents {
            (a.payoff, a.matches, a.inter_payoff, a.inter_matches) = (0, 0, 0, 0);
        }
        self.outcomes = [0; 4];
        (self.demands, self.errors) = (0, 0);
        for _ in 0..self.agents.len() / 2 {
            let (a, b) = self.pair();
            let (ta, tb) = (self.agents[a].tag, self.agents[b].tag);
            let da = self.demand(a, tb);
            let db = self.demand(b, ta);
            let (va, vb) = (self.value(da), self.value(db));
            let fits = va + vb <= 100;
            let k = if da == M && db == M {
                0
            } else if (da == H && db == L) || (da == L && db == H) {
                1
            } else if !fits {
                2
            } else {
                3
            };
            self.outcomes[k] += 1;
            for (i, v, inter) in [(a, va, ta != tb), (b, vb, ta != tb)] {
                let p = if fits { v } else { 0 };
                let ag = &mut self.agents[i];
                ag.payoff += p;
                ag.matches += 1;
                if inter {
                    ag.inter_payoff += p;
                    ag.inter_matches += 1;
                }
            }
            self.agents[a].last_demand = Some(da);
            self.agents[b].last_demand = Some(db);
            self.remember(a, db, tb);
            self.remember(b, da, ta);
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

    /// Every agent's views in the four contexts: darks of darks, lights of
    /// lights, darks of lights, lights of darks (without tags: everyone's in the first).
    fn contexts(&self) -> [Vec<View>; 4] {
        let mut out: [Vec<View>; 4] = Default::default();
        for (i, a) in self.agents.iter().enumerate() {
            if !self.config.tags {
                out[0].push(self.view(i, 0));
            } else {
                out[a.tag as usize].push(self.view(i, a.tag));
                out[2 + a.tag as usize].push(self.view(i, 1 - a.tag));
            }
        }
        out
    }

    fn record(&mut self) {
        let c = self.contexts();
        let (low, decision) = (self.config.low, self.config.decision);
        let best = move |v: View| match decision {
            Decision::Expected => expected_best(v, low),
            Decision::Mode => mode_best(v),
        };
        let r = regime(self.config.tags, [&c[0], &c[1], &c[2], &c[3]], best);
        let every: Vec<View> = c.iter().flatten().copied().collect();
        if self.equity_at.is_none() && equity(&every, self.config.noise) {
            self.equity_at = Some(self.tick);
        }
        if self.first_attractor == 0 && (r == EQUITY || r == FRACTIOUS) {
            self.first_attractor = r;
        }
        let (mut m, mut all) = (0u64, 0u64);
        for a in &self.agents {
            for mem in &a.memory {
                for t in 0..2 {
                    let v = mem.view(t);
                    m += u64::from(v[1]);
                    all += u64::from(v[0] + v[1] + v[2]);
                }
            }
        }
        let matches: u32 = self.outcomes.iter().sum();
        let share = |k: usize| {
            if matches == 0 {
                0.0
            } else {
                f64::from(self.outcomes[k]) / f64::from(matches)
            }
        };
        let per = |sel: &dyn Fn(&Bargainer) -> bool, inter: bool| {
            let (p, n) = self
                .agents
                .iter()
                .filter(|a| sel(a))
                .fold((0u32, 0u32), |(p, n), a| {
                    if inter {
                        (p + a.inter_payoff, n + a.inter_matches)
                    } else {
                        (p + a.payoff, n + a.matches)
                    }
                });
            if n == 0 {
                0.0
            } else {
                f64::from(p) / f64::from(n)
            }
        };
        let tags = self.config.tags;
        self.stats.push(ClassesSnapshot {
            tick: self.tick,
            mean_payoff: per(&|_| true, false),
            m_share: if all == 0 { 0.0 } else { m as f64 / all as f64 },
            outcome_mm: share(0),
            outcome_hl: share(1),
            outcome_fail: share(2),
            outcome_waste: share(3),
            regime: r,
            segregated: u8::from(r == super::stats::CLASSES || r == super::stats::DIVIDED_BELOW),
            equity_at: self.equity_at.unwrap_or(self.tick),
            first_attractor: self.first_attractor,
            payoff_dark: if tags {
                per(&|a| a.tag == 0, false)
            } else {
                0.0
            },
            payoff_light: if tags {
                per(&|a| a.tag == 1, false)
            } else {
                0.0
            },
            payoff_inter: if tags {
                per(&|a| a.tag == 0, true) - per(&|a| a.tag == 1, true)
            } else {
                0.0
            },
            realized_noise: if self.demands == 0 {
                0.0
            } else {
                f64::from(self.errors) / f64::from(self.demands)
            },
        });
    }

    /// The simplexes shown: (name, x offset, tag context: false intra / true inter).
    fn panels(&self) -> Vec<(&'static str, usize, bool)> {
        if self.config.tags {
            vec![("intra", 0, false), ("inter", SIDE + GAP, true)]
        } else {
            vec![("one", 0, false)]
        }
    }

    /// Agent `i`'s view in a panel.
    fn panel_view(&self, i: usize, inter: bool) -> View {
        let t = self.agents[i].tag;
        self.view(i, if inter { 1 - t } else { t })
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<ClassesInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let cell = ClassesCell { x, y };
        let (x, y) = (x as usize, y as usize);
        let panel = self
            .panels()
            .into_iter()
            .find(|&(_, off, _)| x >= off && x < off + SIDE);
        let Some((name, off, inter)) = panel else {
            return Ok(ClassesInspection {
                site: cell,
                simplex: None,
                mix: None,
                best_reply: None,
                agents: Vec::new(),
                agent: None,
            });
        };
        let mix = simplex::mix(x - off, y);
        let agents = (0..self.agents.len())
            .filter(|&i| simplex::point(self.panel_view(i, inter)) == (x - off, y))
            .map(|i| {
                let a = &self.agents[i];
                BargainerView {
                    id: a.id,
                    tag: self
                        .config
                        .tags
                        .then_some(if a.tag == 0 { "dark" } else { "light" }),
                    memory: self.panel_view(i, inter),
                    last_demand: a.last_demand.map(letter),
                    mean_payoff: a.mean_payoff(),
                }
            })
            .collect();
        Ok(ClassesInspection {
            site: cell,
            simplex: mix.map(|_| name),
            best_reply: mix
                .map(|p| letter(simplex::reply(p, self.config.low, self.config.decision))),
            mix,
            agents,
            agent: None,
        })
    }
}

/// Each agent's tag: darks first without a lattice; on a lattice by layout.
fn tag_layout(c: &ClassesConfig, rng: &mut SimRng) -> Vec<u8> {
    let n = c.population() as usize;
    if !c.tags {
        return vec![0; n];
    }
    match c.interaction {
        Interaction::Random => (0..n).map(|i| u8::from(i >= n / 2)).collect(),
        Interaction::Lattice => {
            let (w, h) = (c.lattice.width as usize, c.lattice.height as usize);
            match c.lattice.layout {
                Layout::Random => {
                    let mut t: Vec<u8> = (0..n).map(|i| u8::from(i >= n / 2)).collect();
                    t.shuffle(rng);
                    t
                }
                Layout::FourZones => (0..n)
                    .map(|i| u8::from((i % w < w / 2) != (i / w < h / 2)))
                    .collect(),
                Layout::TwoZones => (0..n).map(|i| u8::from(i % w >= w / 2)).collect(),
            }
        }
    }
}

/// The m demands an agent of tag `own` starts remembering of opponents of
/// tag `of`.
fn start_memory(c: &ClassesConfig, own: u8, of: u8, m: usize, rng: &mut SimRng) -> Vec<u8> {
    match c.start {
        Start::Random => (0..m).map(|_| rng.gen_range(0..3u32) as u8).collect(),
        Start::Fractious => {
            let mut v: Vec<u8> = (0..m)
                .map(|k| if k < m.div_ceil(2) { H } else { L })
                .collect();
            v.shuffle(rng);
            v
        }
        Start::Progressive => Vec::new(),
        Start::Classes => {
            let d = if own == of {
                M
            } else if own == 0 {
                L
            } else {
                H
            };
            vec![d; m]
        }
    }
}

impl Model for ClassesWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Classes(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        ClassesWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick, every agent's tag, memories and last demand.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u8| {
            h ^= u64::from(v);
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        for b in self.tick.to_le_bytes() {
            eat(b);
        }
        for a in &self.agents {
            eat(a.tag);
            eat(a.last_demand.map_or(9, |d| d));
            for mem in &a.memory {
                eat(mem.len() as u8);
                for b in mem.bytes() {
                    eat(b);
                }
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let w = if self.config.tags {
            2 * SIDE + GAP
        } else {
            SIDE
        };
        (w as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ClassesMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let (fw, fh) = (fw as usize, fh as usize);
        buf.clear();
        buf.resize(fw * fh * 4, 0);
        let mut put = |x: usize, y: usize, c: Rgb| {
            let k = (y * fw + x) * 4;
            buf[k..k + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        };
        for y in 0..fh {
            for x in 0..fw {
                put(x, y, BACKGROUND);
            }
        }
        for (_, off, inter) in self.panels() {
            for y in 0..TALL {
                for x in 0..SIDE {
                    if let Some(p) = simplex::mix(x, y) {
                        let region = match simplex::reply(p, self.config.low, self.config.decision)
                        {
                            L => L_REGION,
                            M => M_REGION,
                            _ => H_REGION,
                        };
                        let c = match mode {
                            ClassesMode::BestReply => region,
                            ClassesMode::Payoff => lerp(BACKGROUND, region, 0.4),
                        };
                        put(off + x, y, c);
                    }
                }
            }
            // Agents at their memories: dark tag hot, light tag cool; brighter where several meet.
            let mut at: Vec<((usize, usize), u8, u32, f64)> = Vec::new();
            for i in 0..self.agents.len() {
                let p = simplex::point(self.panel_view(i, inter));
                let a = &self.agents[i];
                match at.iter_mut().find(|e| e.0 == p && e.1 == a.tag) {
                    Some(e) => {
                        e.2 += 1;
                        e.3 += a.mean_payoff();
                    }
                    None => at.push((p, a.tag, 1, a.mean_payoff())),
                }
            }
            for (p, tag, count, payoff) in at {
                let c = match mode {
                    ClassesMode::BestReply => {
                        let base = if tag == 0 { HOT } else { COOL };
                        lerp(base, BRIGHT, (f64::from(count - 1) / 4.0).min(1.0) * 0.6)
                    }
                    ClassesMode::Payoff => lerp(
                        COOL,
                        HOT,
                        payoff / f64::from(count) / f64::from(self.config.high()),
                    ),
                };
                put(off + p.0, p.1, c);
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
        let mut out = String::from(
            "id,tag,last_demand,mean_payoff,own_l,own_m,own_h,other_l,other_m,other_h\n",
        );
        for (i, a) in self.agents.iter().enumerate() {
            let own = self.panel_view(i, false);
            let other = if self.config.tags {
                self.panel_view(i, true)
            } else {
                [0; 3]
            };
            writeln!(
                out,
                "{},{},{},{},{},{},{},{},{},{}",
                a.id,
                a.tag,
                a.last_demand
                    .map_or(String::new(), |d| letter(d).to_string()),
                a.mean_payoff(),
                own[0],
                own[1],
                own[2],
                other[0],
                other[1],
                other[2]
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
        let ModelConfig::Classes(next) = next else {
            return Err(wrong_model(ModelKind::Classes, &next));
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(edit: impl FnOnce(&mut ClassesConfig)) -> ClassesConfig {
        let mut c = ClassesConfig::default();
        edit(&mut c);
        c
    }

    #[test]
    fn memories_keep_the_last_m_and_count_by_tag() {
        let mut mem = Memory::default();
        for (d, t) in [(L, 0), (M, 1), (H, 0), (H, 1)] {
            mem.push(d, t, 3);
        }
        assert_eq!(mem.len(), 3);
        assert_eq!(
            (mem.view(0), mem.view(1)),
            ([0, 0, 1], [0, 1, 1]),
            "the first L was dropped"
        );
    }

    #[test]
    fn starts_fill_memories_as_stated() {
        let w = ClassesWorld::new(config(|c| c.start = Start::Fractious), 1).unwrap();
        assert!(w.agents().iter().all(|a| a.memory[0].view(0) == [5, 0, 5]));
        let p = ClassesWorld::new(config(|c| c.start = Start::Progressive), 1).unwrap();
        assert!(p.agents().iter().all(|a| a.memory[0].is_empty()));
        let classes = ClassesWorld::new(
            config(|c| {
                c.tags = true;
                c.start = Start::Classes;
            }),
            1,
        )
        .unwrap();
        let dark = classes.agents().iter().position(|a| a.tag == 0).unwrap();
        let light = classes.agents().iter().position(|a| a.tag == 1).unwrap();
        assert_eq!(
            (classes.view(dark, 0), classes.view(dark, 1)),
            ([0, 10, 0], [10, 0, 0])
        );
        assert_eq!(
            (classes.view(light, 1), classes.view(light, 0)),
            ([0, 10, 0], [0, 0, 10])
        );
        assert_eq!(
            classes.stats.latest().unwrap().regime,
            super::super::stats::CLASSES
        );
        let shared = ClassesWorld::new(
            config(|c| {
                c.tags = true;
                c.tag_memory = TagMemory::Shared;
            }),
            1,
        )
        .unwrap();
        assert!(shared
            .agents()
            .iter()
            .all(|a| a.memory[0].len() == 10 && a.memory[1].is_empty()));
    }

    #[test]
    fn tags_split_the_population_in_half() {
        let w = ClassesWorld::new(config(|c| c.tags = true), 1).unwrap();
        assert_eq!(w.agents().iter().filter(|a| a.tag == 0).count(), 50);
        for layout in [Layout::Random, Layout::FourZones, Layout::TwoZones] {
            let l = ClassesWorld::new(
                config(|c| {
                    c.tags = true;
                    c.interaction = Interaction::Lattice;
                    c.lattice.layout = layout;
                }),
                1,
            )
            .unwrap();
            assert_eq!(
                l.agents().iter().filter(|a| a.tag == 0).count(),
                50,
                "{layout:?}"
            );
        }
    }

    #[test]
    fn a_period_plays_n_over_2_matches_and_pays_demands_that_fit() {
        let mut w = ClassesWorld::new(config(|c| c.noise = 0.0), 2).unwrap();
        w.step();
        let matches: u32 = w.agents().iter().map(|a| a.matches).sum();
        assert_eq!(matches, 100, "50 matches, two players each");
        let s = w.stats.latest().unwrap();
        let shares = s.outcome_mm + s.outcome_hl + s.outcome_fail + s.outcome_waste;
        assert!((shares - 1.0).abs() < 1e-9);
        assert_eq!(s.realized_noise, 0.0);
        // Everyone remembering only M plays M, gets 50 and keeps remembering M.
        let mut eq = ClassesWorld::new(config(|c| c.noise = 0.0), 1).unwrap();
        for a in &mut eq.agents {
            a.memory[0] = Memory::default();
            for _ in 0..10 {
                a.memory[0].push(M, 0, 10);
            }
        }
        eq.step();
        let s = eq.stats.latest().unwrap();
        assert_eq!(
            (s.mean_payoff, s.outcome_mm, s.m_share, s.regime),
            (50.0, 1.0, 1.0, EQUITY)
        );
    }

    #[test]
    fn noise_errs_at_two_thirds_of_epsilon() {
        let mut w = ClassesWorld::new(config(|c| c.noise = 0.3), 3).unwrap();
        for a in &mut w.agents {
            a.memory[0] = Memory::default();
            for _ in 0..10 {
                a.memory[0].push(M, 0, 10);
            }
        }
        let mut errs = 0.0;
        for _ in 0..40 {
            w.step();
            errs += w.stats.latest().unwrap().realized_noise;
        }
        assert!((errs / 40.0 - 0.2).abs() < 0.03, "{}", errs / 40.0);
    }

    #[test]
    fn lattice_players_meet_only_their_neighbors() {
        let c = config(|c| {
            c.interaction = Interaction::Lattice;
            c.lattice.neighborhood = Neighborhood::VonNeumann;
        });
        let mut w = ClassesWorld::new(c, 4).unwrap();
        assert_eq!(w.neighbors.of(0), [90, 9, 1, 10], "the torus wraps");
        for _ in 0..500 {
            let (a, b) = w.pair();
            assert!(w.neighbors.of(a).contains(&(b as u32)));
        }
        let mut r = ClassesWorld::new(ClassesConfig::default(), 4).unwrap();
        for _ in 0..500 {
            let (a, b) = r.pair();
            assert_ne!(a, b);
        }
    }

    #[test]
    fn equity_at_and_the_stop_follow_aeys_threshold() {
        let mut w = ClassesWorld::new(
            config(|c| {
                c.agents = 10;
                c.noise = 0.1;
                c.start = Start::Fractious;
                c.stop_at_equity = true;
            }),
            1,
        )
        .unwrap();
        assert_eq!(w.stats.latest().unwrap().first_attractor, FRACTIOUS);
        w.run(1_000_000);
        assert!(w.is_finished());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.equity_at, w.tick);
        let m = w.config.memory as f64;
        assert!(w
            .agents()
            .iter()
            .all(|a| f64::from(a.memory[0].view(0)[1]) >= 0.9 * m));
    }

    #[test]
    fn the_view_draws_simplexes_and_inspect_finds_agents() {
        let mut w = ClassesWorld::new(config(|c| c.tags = true), 5).unwrap();
        assert_eq!(Model::size(&w), ((2 * SIDE + GAP) as u32, TALL as u32));
        let mut buf = Vec::new();
        w.render("best_reply", "", &mut buf).unwrap();
        assert_eq!(buf.len(), (2 * SIDE + GAP) * TALL * 4);
        let px = |b: &[u8], x: usize, y: usize| {
            [
                b[(y * (2 * SIDE + GAP) + x) * 4],
                b[(y * (2 * SIDE + GAP) + x) * 4 + 1],
                b[(y * (2 * SIDE + GAP) + x) * 4 + 2],
            ]
        };
        assert_eq!(px(&buf, 0, 0), BACKGROUND, "outside the triangle");
        assert_eq!(px(&buf, 2, TALL - 1), M_REGION, "next to the M vertex");
        assert_eq!(
            px(&buf, SIDE + GAP + SIDE - 3, TALL - 1),
            H_REGION,
            "next to the L vertex: reply H"
        );
        assert!(
            w.render("payoff", "", &mut buf).is_ok() && w.render("wealth", "", &mut buf).is_err()
        );
        let (x, y) = simplex::point(w.view(0, 0));
        let v = w.inspect(x as u32, y as u32).unwrap();
        assert_eq!(v.simplex, Some("intra"));
        assert!(v.agents.iter().any(|a| a.id == 1 && a.tag == Some("dark")));
        let (x, y) = simplex::point(w.view(0, 1));
        let v = w.inspect((SIDE + GAP + x) as u32, y as u32).unwrap();
        assert_eq!(v.simplex, Some("inter"));
        assert!(v.agents.iter().any(|a| a.id == 1));
        assert_eq!(
            w.inspect(SIDE as u32 + 1, 0).unwrap().simplex,
            None,
            "the gap"
        );
        assert!(w.inspect(1000, 0).is_err());
        assert_eq!(Model::locate(&w, 1), None);
        w.run(3);
        assert_eq!(w.stats.history().len(), 4);
    }

    #[test]
    fn keyframes_restore_memories_and_counts() {
        let mut any =
            crate::model::ModelWorld::new(ModelConfig::Classes(config(|c| c.tags = true)), 6)
                .unwrap();
        any.model_mut().run(20);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        any.model_mut().run(30);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        assert_eq!(any.model().series("regime").unwrap().len(), 21);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = ClassesWorld::new(ClassesConfig::default(), 1).unwrap();
        let next = config(|c| {
            c.noise = 0.05;
            c.decision = Decision::Mode;
            c.low = 40;
        });
        Model::set_config(&mut w, ModelConfig::Classes(next.clone())).unwrap();
        w.step();
        let e = Model::set_config(
            &mut w,
            ModelConfig::Classes(ClassesConfig { memory: 11, ..next }),
        )
        .unwrap_err();
        assert_eq!(e[0].field, "memory");
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| {
                c.agents = 2;
                c.memory = 1;
            }),
            config(|c| c.noise = 0.0),
            config(|c| c.noise = 1.0),
            config(|c| c.low = 5),
            config(|c| {
                c.low = 45;
                c.tags = true;
                c.start = Start::Progressive;
            }),
            config(|c| {
                c.tags = true;
                c.tag_memory = TagMemory::Shared;
                c.start = Start::Classes;
            }),
        ] {
            let mut w = ClassesWorld::new(c.clone(), 1).unwrap();
            w.run(30);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=70.0).contains(&s.mean_payoff), "{c:?}");
        }
    }
}
