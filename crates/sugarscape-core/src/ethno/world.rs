//! The ethnocentrism world: agents with a tag and help bits on a von Neumann
//! torus. Each period immigrants arrive, every agent decides whether to help
//! each neighbor (paying `cost` of its potential to reproduce, the neighbor
//! gaining `benefit`), agents reproduce with probability PTR into an empty
//! site near them, and each dies with probability `death`.

use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{Discrimination, EthnoConfig, KinBasis, Offspring, PairPlay, Start, Strategy};
use super::stats::{ratio, EthnoSnapshot, Tally, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{
    lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, LENDER, NEUTRAL, POLLUTION, RED, SPICE,
};
use crate::rng::{self, SimRng};
use crate::spatial::{self, Geometry};
use crate::stats::Stats;

/// The Strategy view's colours.
pub const ETHNOCENTRIC: Rgb = LENDER;
pub const HUMANITARIAN: Rgb = BLUE;
pub const SELFISH: Rgb = RED;
pub const TRAITOROUS: Rgb = BOTH;
pub const KIN: Rgb = POLLUTION;
pub const NONKIN: Rgb = SPICE;
pub const MIXED: Rgb = NEUTRAL;

/// The four neighbors of a site, in the lattice's order: up, left, right,
/// down. The neighbor in direction `d` sees this site in direction `3 − d`.
const DIRECTIONS: usize = 4;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EthnoMode {
    Strategy,
    Tag,
    Lineage,
    Ptr,
}

impl std::str::FromStr for EthnoMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "strategy" => Self::Strategy,
            "tag" => Self::Tag,
            "lineage" => Self::Lineage,
            "ptr" => Self::Ptr,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// An agent on the lattice.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: u64,
    pub tag: u32,
    /// Help bits. `same_other`: bit 0 helps the same colour (or kin), bit 1
    /// other colours; `none`: bit 0 helps everyone; `each_color`: bit k
    /// helps colour k.
    pub help: u64,
    /// With kin strategies: same/other is judged by the kin marker.
    pub kin_basis: bool,
    /// The founding immigrant's id (never mutates).
    pub lineage: u64,
    /// The kin marker: the id of the family's founder (an immigrant or a
    /// mutated offspring).
    pub family: u64,
    /// The period it arrived or was born in (0: a full start).
    pub born: u64,
    /// This period's potential to reproduce.
    pub ptr: f64,
    /// Helps given and received this period.
    pub given: u32,
    pub received: u32,
    /// Helps given this period to the neighbor in each direction.
    pub gave: [u8; DIRECTIONS],
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EthnoInspection {
    pub site: Site,
    /// The agent on the site, or none.
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Site {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub tag: u32,
    pub strategy: Strategy,
    /// "tag" or "kin": what same/other is judged by.
    pub basis: &'static str,
    pub ptr: f64,
    pub given: u32,
    pub received: u32,
    pub lineage: u64,
    pub kin_marker: u64,
    pub age: u64,
    /// The occupied neighbors: up, left, right, down.
    pub neighbors: Vec<NeighborView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NeighborView {
    pub x: u32,
    pub y: u32,
    pub tag: u32,
    pub strategy: Strategy,
    /// A common founding immigrant.
    pub related: bool,
    /// Helps from the agent to this neighbor this period, and back.
    pub helped: u8,
    pub helped_by: u8,
}

#[derive(Clone)]
pub struct EthnoWorld {
    pub config: EthnoConfig,
    /// The periodic von Neumann lattice, fixed once built, so keyframes share it.
    pub geometry: Arc<Geometry>,
    /// Completed periods.
    pub tick: u64,
    sites: Vec<Option<Agent>>,
    /// The empty sites (in no particular order) and each site's index in it
    /// (`u32::MAX` when occupied).
    empty: Vec<u32>,
    slot: Vec<u32>,
    next_id: u64,
    /// This period's interaction counts.
    tally: Tally,
    rng: SimRng,
    pub stats: Stats<EthnoSnapshot>,
}

/// The lowest `bits` bits.
fn mask(bits: u32) -> u64 {
    if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

impl EthnoWorld {
    pub fn new(config: EthnoConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let lattice = spatial::SpatialConfig {
            width: config.width,
            height: config.width,
            neighborhood: spatial::Neighborhood::VonNeumann,
            boundary: spatial::Boundary::Periodic,
            ..Default::default()
        };
        // A square lattice draws nothing; the world's stream starts untouched.
        let geometry = Geometry::new(&lattice, &mut rng::seeded(0));
        let n = geometry.len();
        let mut w = EthnoWorld {
            config,
            geometry: Arc::new(geometry),
            tick: 0,
            sites: vec![None; n],
            empty: (0..n as u32).collect(),
            slot: (0..n as u32).collect(),
            next_id: 1,
            tally: Tally::default(),
            rng: rng::seeded(seed),
            stats: Stats::default(),
        };
        match w.config.start {
            Start::Empty => {}
            Start::Random => {
                for s in 0..n {
                    let a = w.newcomer(0);
                    w.place(s, a);
                }
            }
            Start::Selfish => {
                for s in 0..n {
                    let mut a = w.newcomer(0);
                    a.help = 0;
                    a.kin_basis = false;
                    w.place(s, a);
                }
            }
        }
        w.record();
        Ok(w)
    }

    pub fn sites(&self) -> usize {
        self.sites.len()
    }

    pub fn agent(&self, site: usize) -> Option<&Agent> {
        self.sites[site].as_ref()
    }

    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.sites.iter().flatten()
    }

    pub fn population(&self) -> usize {
        self.sites.len() - self.empty.len()
    }

    /// Whether the run has reached its last period.
    pub fn is_finished(&self) -> bool {
        self.config.end > 0 && self.tick >= u64::from(self.config.end)
    }

    /// The number of help bits a strategy has.
    fn help_bits(&self) -> u32 {
        match self.config.discrimination {
            Discrimination::SameOther => 2,
            Discrimination::None => 1,
            Discrimination::EachColor => self.config.colors,
        }
    }

    /// Whether help bits `help` make an allowed strategy.
    fn permitted(&self, help: u64) -> bool {
        self.config.allows_all()
            || self
                .config
                .allowed
                .contains(&Strategy::of_bits(help & 1 != 0, help & 2 != 0))
    }

    /// The strategy `a` plays.
    pub fn strategy(&self, a: &Agent) -> Strategy {
        match self.config.discrimination {
            Discrimination::SameOther => {
                match (
                    Strategy::of_bits(a.help & 1 != 0, a.help & 2 != 0),
                    a.kin_basis,
                ) {
                    (Strategy::Ethnocentric, true) => Strategy::Kin,
                    (Strategy::Traitorous, true) => Strategy::Nonkin,
                    (s, _) => s,
                }
            }
            Discrimination::None => {
                if a.help & 1 != 0 {
                    Strategy::Humanitarian
                } else {
                    Strategy::Selfish
                }
            }
            Discrimination::EachColor => {
                let all = mask(self.config.colors);
                let own = 1u64 << a.tag;
                match a.help {
                    0 => Strategy::Selfish,
                    h if h == all => Strategy::Humanitarian,
                    h if h == own => Strategy::Ethnocentric,
                    h if h == all & !own => Strategy::Traitorous,
                    _ => Strategy::Mixed,
                }
            }
        }
    }

    /// An agent arriving in period `born` (0: a full start) with a uniformly
    /// random tag, random help bits redrawn until allowed, and (with kin
    /// strategies) a random basis bit; it founds its own lineage and family.
    fn newcomer(&mut self, born: u64) -> Agent {
        let tag = self.rng.gen_range(0..self.config.colors);
        let bits = mask(self.help_bits());
        let help = loop {
            let h = self.rng.gen::<u64>() & bits;
            if self.permitted(h) {
                break h;
            }
        };
        let kin_basis = self.config.kin_strategies && self.rng.gen::<bool>();
        let id = self.next_id;
        self.next_id += 1;
        Agent {
            id,
            tag,
            help,
            kin_basis,
            lineage: id,
            family: id,
            born,
            ptr: self.config.base_ptr,
            given: 0,
            received: 0,
            gave: [0; DIRECTIONS],
        }
    }

    /// `parent`'s offspring: a copy whose strategy bits (in order), basis
    /// bit and tag may mutate, and which may found a new family.
    fn offspring_of(&mut self, parent: &Agent) -> Agent {
        let c = &self.config;
        let (mutation, tag_rate, kin_rate) = (c.mutation, c.tag_rate(), c.kin_mutation);
        let (colors, kin) = (c.colors, c.kin_strategies);
        let basis_mutates = kin && c.kin_basis == KinBasis::Mutates;
        let mut help = parent.help;
        for k in 0..self.help_bits() {
            if self.rng.gen::<f64>() < mutation {
                let next = help ^ (1 << k);
                // HKS13: "that mutation is ignored".
                if self.permitted(next) {
                    help = next;
                }
            }
        }
        let mut kin_basis = parent.kin_basis;
        if basis_mutates && self.rng.gen::<f64>() < mutation {
            kin_basis = !kin_basis;
        }
        let mut tag = parent.tag;
        if self.rng.gen::<f64>() < tag_rate && colors > 1 {
            // Uniform over the other colours (HA-Java redraws until different).
            let t = self.rng.gen_range(0..colors - 1);
            tag = if t >= parent.tag { t + 1 } else { t };
        }
        let id = self.next_id;
        self.next_id += 1;
        let family = if kin && self.rng.gen::<f64>() < kin_rate {
            id
        } else {
            parent.family
        };
        Agent {
            id,
            tag,
            help,
            kin_basis,
            lineage: parent.lineage,
            family,
            born: self.tick + 1,
            ptr: self.config.base_ptr,
            given: 0,
            received: 0,
            gave: [0; DIRECTIONS],
        }
    }

    fn place(&mut self, site: usize, a: Agent) {
        debug_assert!(self.sites[site].is_none(), "site {site} is occupied");
        let k = self.slot[site] as usize;
        let last = *self.empty.last().expect("an empty site");
        self.empty.swap_remove(k);
        if last as usize != site {
            self.slot[last as usize] = k as u32;
        }
        self.slot[site] = u32::MAX;
        self.sites[site] = Some(a);
    }

    fn vacate(&mut self, site: usize) {
        if self.sites[site].take().is_some() {
            self.slot[site] = self.empty.len() as u32;
            self.empty.push(site as u32);
        }
    }

    /// A uniformly chosen empty site, if any.
    fn random_empty(&mut self) -> Option<usize> {
        if self.empty.is_empty() {
            None
        } else {
            let k = self.rng.gen_range(0..self.empty.len() as u32) as usize;
            Some(self.empty[k] as usize)
        }
    }

    /// One period.
    pub fn step(&mut self) {
        self.apply_schedule();
        self.immigrate();
        self.interact();
        self.reproduce();
        self.die();
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
        for (path, value) in due {
            let next = ModelConfig::Ethno(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Ethno(next)) = next {
                self.config = next;
            }
        }
    }

    /// Step 1: ⌊r⌋ immigrants, one more with probability r − ⌊r⌋, each at a
    /// uniformly chosen empty site (lost when the lattice is full).
    fn immigrate(&mut self) {
        let r = self.config.immigration;
        let whole = r.floor();
        let mut count = whole as u32;
        if r > whole && self.rng.gen::<f64>() < r - whole {
            count += 1;
        }
        for _ in 0..count {
            let Some(site) = self.random_empty() else {
                break;
            };
            let a = self.newcomer(self.tick + 1);
            self.place(site, a);
        }
    }

    /// Whether `a` helps `b` in one decision (drawing misperception).
    fn helps(&mut self, a: &Agent, b: &Agent) -> bool {
        match self.config.discrimination {
            Discrimination::SameOther => {
                let mut same = if a.kin_basis {
                    a.family == b.family
                } else {
                    a.tag == b.tag
                };
                let p = self.config.misperception;
                if p > 0.0 && self.rng.gen::<f64>() < p {
                    same = !same;
                }
                a.help & if same { 1 } else { 2 } != 0
            }
            Discrimination::None => a.help & 1 != 0,
            Discrimination::EachColor => a.help & (1 << b.tag) != 0,
        }
    }

    /// Step 2: every PTR set to the base, then in site order each agent
    /// decides for each occupied neighbor (up, left, right, down) whether to
    /// help it — twice with `pair_play: twice`.
    fn interact(&mut self) {
        let base = self.config.base_ptr;
        for a in self.sites.iter_mut().flatten() {
            a.ptr = base;
            a.given = 0;
            a.received = 0;
            a.gave = [0; DIRECTIONS];
        }
        let times = match self.config.pair_play {
            PairPlay::Once => 1,
            PairPlay::Twice => 2,
        };
        let (cost, benefit) = (self.config.cost, self.config.benefit);
        let geometry = Arc::clone(&self.geometry);
        let mut t = Tally::default();
        for i in 0..self.sites.len() {
            if self.sites[i].is_none() {
                continue;
            }
            for (d, &j) in geometry.neighbors(i).iter().enumerate() {
                let j = j as usize;
                let (Some(a), Some(b)) = (&self.sites[i], &self.sites[j]) else {
                    continue;
                };
                let (a, b) = (a.clone(), b.clone());
                let related = a.lineage == b.lineage;
                let same = a.tag == b.tag;
                t.pairs += 1;
                t.related += u64::from(related);
                t.same += u64::from(same);
                t.related_same += u64::from(related && same);
                for _ in 0..times {
                    t.decisions += 1;
                    t.same_tag += u64::from(same);
                    if self.helps(&a, &b) {
                        t.helps += 1;
                        t.helps_to_relatives += u64::from(related);
                        let giver = self.sites[i].as_mut().unwrap();
                        giver.ptr -= cost;
                        giver.given += 1;
                        giver.gave[d] += 1;
                        let taker = self.sites[j].as_mut().unwrap();
                        taker.ptr += benefit;
                        taker.received += 1;
                    }
                }
            }
        }
        self.tally = t;
    }

    /// Step 3: the agents present, in a random order, each reproduce with
    /// probability PTR into a random empty neighbor (or, `anywhere`, a
    /// random empty site). Offspring do not reproduce this period.
    fn reproduce(&mut self) {
        let mut order: Vec<usize> = (0..self.sites.len())
            .filter(|&s| self.sites[s].is_some())
            .collect();
        order.shuffle(&mut self.rng);
        let geometry = Arc::clone(&self.geometry);
        for s in order {
            let u = self.rng.gen::<f64>();
            let parent = self.sites[s].clone().expect("parents stay put");
            if u >= parent.ptr {
                continue;
            }
            let target = match self.config.offspring {
                Offspring::Adjacent => {
                    let free: Vec<usize> = geometry
                        .neighbors(s)
                        .iter()
                        .map(|&j| j as usize)
                        .filter(|&j| self.sites[j].is_none())
                        .collect();
                    if free.is_empty() {
                        continue;
                    }
                    free[self.rng.gen_range(0..free.len() as u32) as usize]
                }
                Offspring::Anywhere => match self.random_empty() {
                    Some(site) => site,
                    None => continue,
                },
            };
            let child = self.offspring_of(&parent);
            self.place(target, child);
            // Nobody helped the newcomer this period.
            for (d, &j) in geometry.neighbors(target).iter().enumerate() {
                if let Some(n) = self.sites[j as usize].as_mut() {
                    n.gave[DIRECTIONS - 1 - d] = 0;
                }
            }
        }
    }

    /// Step 4: in site order every agent, newcomers included, dies with
    /// probability `death`.
    fn die(&mut self) {
        let death = self.config.death;
        for s in 0..self.sites.len() {
            if self.sites[s].is_some() && self.rng.gen::<f64>() < death {
                self.vacate(s);
            }
        }
    }

    fn record(&mut self) {
        // Indexed by `Strategy`'s declaration order.
        let mut counts = [0u32; 7];
        for a in self.agents() {
            counts[self.strategy(a) as usize] += 1;
        }
        let n = self.population() as u64;
        let share = |s: Strategy| ratio(u64::from(counts[s as usize]), n);
        let t = self.tally;
        let s = EthnoSnapshot {
            tick: self.tick,
            population: n as u32,
            ethnocentric: share(Strategy::Ethnocentric),
            humanitarian: share(Strategy::Humanitarian),
            selfish: share(Strategy::Selfish),
            traitorous: share(Strategy::Traitorous),
            kin: share(Strategy::Kin),
            nonkin: share(Strategy::Nonkin),
            mixed: share(Strategy::Mixed),
            cooperation: ratio(t.helps, t.decisions),
            same_tag: ratio(t.same_tag, t.decisions),
            relatives: ratio(t.related, t.pairs),
            kin_help: ratio(t.helps_to_relatives, t.helps),
            tag_given_relative: ratio(t.related_same, t.related),
            relative_given_tag: ratio(t.related_same, t.same),
        };
        self.stats.push(s);
    }

    fn color(&self, a: &Agent, mode: EthnoMode) -> Rgb {
        match mode {
            EthnoMode::Strategy => match self.strategy(a) {
                Strategy::Ethnocentric => ETHNOCENTRIC,
                Strategy::Humanitarian => HUMANITARIAN,
                Strategy::Selfish => SELFISH,
                Strategy::Traitorous => TRAITOROUS,
                Strategy::Kin => KIN,
                Strategy::Nonkin => NONKIN,
                Strategy::Mixed => MIXED,
            },
            EthnoMode::Tag => tag_color(a.tag),
            EthnoMode::Lineage => lineage_color(a.lineage),
            EthnoMode::Ptr => {
                let times = match self.config.pair_play {
                    PairPlay::Once => 4.0,
                    PairPlay::Twice => 8.0,
                };
                let c = &self.config;
                let (lo, hi) = (c.base_ptr - times * c.cost, c.base_ptr + times * c.benefit);
                let t = if hi > lo {
                    (a.ptr - lo) / (hi - lo)
                } else {
                    0.5
                };
                lerp(COOL, HOT, t)
            }
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<EthnoInspection, String> {
        let site = self
            .geometry
            .at(x, y, 0)
            .ok_or_else(|| format!("({x}, {y}) is outside the lattice"))?;
        let agent = self.sites[site].as_ref().map(|a| {
            let neighbors = self
                .geometry
                .neighbors(site)
                .iter()
                .enumerate()
                .filter_map(|(d, &j)| {
                    let b = self.sites[j as usize].as_ref()?;
                    let (x, y, _) = self.geometry.xyz(j as usize);
                    Some(NeighborView {
                        x,
                        y,
                        tag: b.tag,
                        strategy: self.strategy(b),
                        related: a.lineage == b.lineage,
                        helped: a.gave[d],
                        helped_by: b.gave[DIRECTIONS - 1 - d],
                    })
                })
                .collect();
            AgentView {
                id: a.id,
                tag: a.tag,
                strategy: self.strategy(a),
                basis: if a.kin_basis { "kin" } else { "tag" },
                ptr: a.ptr,
                given: a.given,
                received: a.received,
                lineage: a.lineage,
                kin_marker: a.family,
                age: self.tick - a.born.min(self.tick),
                neighbors,
            }
        });
        Ok(EthnoInspection {
            site: Site { x, y },
            agent,
        })
    }
}

/// Tags 0–3 in the Java's colours (blue, red, green, yellow), then hues a
/// golden angle apart.
pub fn tag_color(tag: u32) -> Rgb {
    const FIRST: [Rgb; 4] = [BLUE, RED, LENDER, BOTH];
    if let Some(&c) = FIRST.get(tag as usize) {
        return c;
    }
    let hue = (f64::from(tag) * 0.618_033_988_75).fract() * 6.0;
    let (sector, f) = (hue.floor() as u32, hue.fract());
    let (hi, lo) = (230.0, 70.0);
    let up = lo + (hi - lo) * f;
    let down = hi - (hi - lo) * f;
    let [r, g, b] = match sector {
        0 => [hi, up, lo],
        1 => [down, hi, lo],
        2 => [lo, hi, up],
        3 => [lo, down, hi],
        4 => [up, lo, hi],
        _ => [hi, lo, down],
    };
    [r as u8, g as u8, b as u8]
}

/// A colour hashed from a lineage id (splitmix64), kept away from black.
pub fn lineage_color(lineage: u64) -> Rgb {
    let mut z = lineage.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    std::array::from_fn(|k| 64 + ((z >> (8 * k)) as u8 as u32 * 191 / 255) as u8)
}

impl Model for EthnoWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Ethno(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        EthnoWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        EthnoWorld::population(self)
    }

    /// FNV-1a over the tick, the id counter and every site's agent.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        eat(self.next_id);
        for site in &self.sites {
            match site {
                None => eat(u64::MAX),
                Some(a) => {
                    eat(a.id);
                    eat(a.help);
                    eat(u64::from(a.tag) | u64::from(a.kin_basis) << 32);
                    eat(a.lineage);
                    eat(a.family);
                    eat(a.born);
                    eat(a.ptr.to_bits());
                    eat(u64::from(a.given) << 32 | u64::from(a.received));
                }
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.width)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: EthnoMode = mode.parse()?;
        buf.clear();
        buf.resize(self.sites.len() * 4, 0);
        for (px, site) in buf.as_chunks_mut::<4>().0.iter_mut().zip(&self.sites) {
            let rgb = site.as_ref().map_or(BACKGROUND, |a| self.color(a, mode));
            px.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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
        let mut out =
            String::from("id,x,y,tag,strategy,basis,lineage,kin_marker,age,ptr,given,received\n");
        for (s, a) in self.sites.iter().enumerate() {
            let Some(a) = a else { continue };
            let (x, y, _) = self.geometry.xyz(s);
            writeln!(
                out,
                "{},{x},{y},{},{},{},{},{},{},{},{},{}",
                a.id,
                a.tag,
                self.strategy(a).letter(),
                if a.kin_basis { "kin" } else { "tag" },
                a.lineage,
                a.family,
                self.tick - a.born.min(self.tick),
                a.ptr,
                a.given,
                a.received
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
        let s = self
            .sites
            .iter()
            .position(|a| a.as_ref().is_some_and(|a| a.id == id))?;
        let (x, y, _) = self.geometry.xyz(s);
        Some((x, y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Ethno(next) = next else {
            return Err(wrong_model(ModelKind::Ethno, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
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
    use crate::config::ScheduledChange;
    use serde_json::json;

    /// An empty `w` × `w` world without immigration, after `edit`.
    fn world(w: u32, edit: impl FnOnce(&mut EthnoConfig)) -> EthnoWorld {
        let mut c = EthnoConfig {
            width: w,
            immigration: 0.0,
            ..Default::default()
        };
        edit(&mut c);
        EthnoWorld::new(c, 1).unwrap()
    }

    /// Puts an agent with `tag` and help bits `help` at (x, y), founding
    /// lineage and family `lineage`; returns its site.
    fn put(w: &mut EthnoWorld, (x, y): (u32, u32), tag: u32, help: u64, lineage: u64) -> usize {
        let s = w.geometry.at(x, y, 0).unwrap();
        let id = w.next_id;
        w.next_id += 1;
        w.place(
            s,
            Agent {
                id,
                tag,
                help,
                kin_basis: false,
                lineage,
                family: lineage,
                born: 0,
                ptr: 0.0,
                given: 0,
                received: 0,
                gave: [0; DIRECTIONS],
            },
        );
        s
    }

    const E: u64 = 0b01;
    const H: u64 = 0b11;
    const S: u64 = 0b00;
    const T: u64 = 0b10;

    fn ptr(w: &EthnoWorld, s: usize) -> f64 {
        w.agent(s).unwrap().ptr
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn each_strategy_pair_moves_ptr_by_cost_and_benefit() {
        // (a's bits, b's bits, same tag) → a's and b's PTR after one period's
        // interaction, as a pair alone on the lattice.
        let cases = [
            (E, E, true, 0.12 - 0.01 + 0.03, 0.12 - 0.01 + 0.03),
            (E, E, false, 0.12, 0.12),
            (E, H, false, 0.12 + 0.03, 0.12 - 0.01),
            (H, H, false, 0.14, 0.14),
            (S, H, true, 0.15, 0.11),
            (T, T, false, 0.14, 0.14),
            (T, E, false, 0.11, 0.15),
            (S, S, true, 0.12, 0.12),
        ];
        for (a_bits, b_bits, same, pa, pb) in cases {
            let mut w = world(5, |_| {});
            let a = put(&mut w, (1, 1), 0, a_bits, 1);
            let b = put(&mut w, (2, 1), if same { 0 } else { 1 }, b_bits, 2);
            w.interact();
            assert!(
                close(ptr(&w, a), pa) && close(ptr(&w, b), pb),
                "{a_bits:b} vs {b_bits:b}, same {same}: {} {}",
                ptr(&w, a),
                ptr(&w, b)
            );
        }
    }

    #[test]
    fn an_agent_pays_for_each_neighbor_it_helps() {
        let mut w = world(5, |_| {});
        let mid = put(&mut w, (2, 2), 0, H, 1);
        for (k, xy) in [(2, 1), (1, 2), (3, 2), (2, 3)].into_iter().enumerate() {
            put(&mut w, xy, k as u32, S, 2);
        }
        w.interact();
        assert!(close(ptr(&w, mid), 0.12 - 4.0 * 0.01));
        let a = w.agent(mid).unwrap();
        assert_eq!((a.given, a.received, a.gave), (4, 0, [1; 4]));
        let up = w.geometry.at(2, 1, 0).unwrap();
        assert!(close(ptr(&w, up), 0.15));
    }

    #[test]
    fn twice_decides_every_direction_twice() {
        let mut w = world(5, |c| c.pair_play = PairPlay::Twice);
        let a = put(&mut w, (1, 1), 0, H, 1);
        let b = put(&mut w, (1, 2), 0, S, 2);
        w.interact();
        assert!(close(ptr(&w, a), 0.12 - 2.0 * 0.01));
        assert!(close(ptr(&w, b), 0.12 + 2.0 * 0.03));
        assert_eq!((w.tally.decisions, w.tally.helps, w.tally.pairs), (4, 2, 2));
    }

    #[test]
    fn misperception_one_inverts_every_judgment() {
        let mut w = world(5, |c| c.misperception = 1.0);
        let a = put(&mut w, (1, 1), 0, E, 1);
        let b = put(&mut w, (2, 1), 0, T, 2);
        w.interact();
        // Each sees the other as another colour: the E defects, the T helps.
        assert!(close(ptr(&w, a), 0.15) && close(ptr(&w, b), 0.11));
    }

    #[test]
    fn blind_and_each_colour_agents_decide_by_their_bits() {
        let mut blind = world(5, |c| c.discrimination = Discrimination::None);
        let a = put(&mut blind, (1, 1), 0, 1, 1);
        let b = put(&mut blind, (2, 1), 1, 0, 2);
        blind.interact();
        assert!(close(ptr(&blind, a), 0.11) && close(ptr(&blind, b), 0.15));
        assert_eq!(
            blind.strategy(blind.agent(a).unwrap()),
            Strategy::Humanitarian
        );
        assert_eq!(blind.strategy(blind.agent(b).unwrap()), Strategy::Selfish);
        let mut each = world(5, |c| {
            c.discrimination = Discrimination::EachColor;
            c.colors = 3;
        });
        // Helps colours 0 and 2; its neighbors are colours 1 and 2.
        let a = put(&mut each, (1, 1), 0, 0b101, 1);
        let b = put(&mut each, (2, 1), 1, 0, 2);
        let c = put(&mut each, (1, 2), 2, 0, 3);
        each.interact();
        assert!(close(ptr(&each, a), 0.11));
        assert!(close(ptr(&each, b), 0.12) && close(ptr(&each, c), 0.15));
        let kind = |w: &EthnoWorld, tag: u32, help: u64| {
            let mut probe = w.agent(a).unwrap().clone();
            (probe.tag, probe.help) = (tag, help);
            w.strategy(&probe)
        };
        assert_eq!(kind(&each, 0, 0b101), Strategy::Mixed);
        assert_eq!(kind(&each, 1, 0b010), Strategy::Ethnocentric);
        assert_eq!(kind(&each, 1, 0b101), Strategy::Traitorous);
        assert_eq!(kind(&each, 2, 0b111), Strategy::Humanitarian);
        assert_eq!(kind(&each, 2, 0), Strategy::Selfish);
    }

    #[test]
    fn kin_strategies_judge_by_the_kin_marker() {
        let mut w = world(5, |c| c.kin_strategies = true);
        let a = put(&mut w, (1, 1), 0, E, 1);
        put(&mut w, (2, 1), 1, S, 1);
        w.sites[a].as_mut().unwrap().kin_basis = true;
        w.interact();
        assert!(
            close(ptr(&w, a), 0.11),
            "the same family, another tag: helps"
        );
        assert_eq!(w.strategy(w.agent(a).unwrap()), Strategy::Kin);
        w.sites[a].as_mut().unwrap().help = T;
        assert_eq!(w.strategy(w.agent(a).unwrap()), Strategy::Nonkin);
        w.sites[a].as_mut().unwrap().help = H;
        assert_eq!(w.strategy(w.agent(a).unwrap()), Strategy::Humanitarian);
    }

    #[test]
    fn a_mutated_tag_is_never_the_parents_and_is_uniform_over_the_rest() {
        let mut w = world(5, |c| {
            c.colors = 5;
            c.tag_mutation = Some(1.0);
            c.mutation = 0.0;
        });
        let p = put(&mut w, (0, 0), 2, E, 1);
        let parent = w.agent(p).unwrap().clone();
        let mut counts = [0u32; 5];
        for _ in 0..20_000 {
            let child = w.offspring_of(&parent);
            counts[child.tag as usize] += 1;
            assert_eq!(child.help, E, "strategy bits do not mutate at rate 0");
        }
        assert_eq!(counts[2], 0);
        for (t, &n) in counts.iter().enumerate() {
            if t != 2 {
                assert!((4700..=5300).contains(&n), "tag {t}: {n}");
            }
        }
    }

    #[test]
    fn allowed_strategies_bind_immigrants_and_offspring() {
        let mut w = world(10, |c| {
            c.allowed = vec![Strategy::Humanitarian, Strategy::Selfish];
            c.immigration = 3.0;
            c.mutation = 0.3;
            c.base_ptr = 0.5;
        });
        for _ in 0..200 {
            w.step();
            for a in w.agents() {
                assert!(
                    matches!(w.strategy(a), Strategy::Humanitarian | Strategy::Selfish),
                    "{a:?}"
                );
            }
        }
        assert!(w.population() > 20);
        // One allowed strategy: its bits never change.
        let mut solo = world(10, |c| {
            c.allowed = vec![Strategy::Traitorous];
            c.immigration = 2.0;
            c.mutation = 1.0;
        });
        solo.run(50);
        assert!(solo.agents().all(|a| a.help == T));
    }

    #[test]
    fn lineage_is_inherited_and_the_kin_marker_can_found_a_family() {
        let mut w = world(5, |c| {
            c.kin_strategies = true;
            c.kin_mutation = 1.0;
        });
        let immigrant = w.newcomer(1);
        assert_eq!(
            (immigrant.lineage, immigrant.family),
            (immigrant.id, immigrant.id)
        );
        let child = w.offspring_of(&immigrant);
        assert_eq!(child.lineage, immigrant.lineage);
        assert_eq!(child.family, child.id, "a new family");
        w.config.kin_mutation = 0.0;
        let grandchild = w.offspring_of(&child);
        assert_eq!(
            (grandchild.lineage, grandchild.family),
            (immigrant.id, child.id)
        );
        // The basis bit flips at the mutation rate, unless it is fixed.
        w.config.mutation = 1.0;
        assert!(w.offspring_of(&grandchild).kin_basis != grandchild.kin_basis);
        w.config.kin_basis = KinBasis::Fixed;
        assert_eq!(w.offspring_of(&grandchild).kin_basis, grandchild.kin_basis);
        // Without kin strategies the marker never mutates.
        let mut plain = world(5, |c| c.kin_mutation = 1.0);
        let a = plain.newcomer(1);
        assert_eq!(plain.offspring_of(&a).family, a.family);
    }

    #[test]
    fn fractional_immigration_is_a_chance_of_one_more() {
        let mut w = world(100, |c| c.immigration = 0.5);
        for _ in 0..2000 {
            w.immigrate();
        }
        assert!((900..=1100).contains(&w.population()), "{}", w.population());
        let mut two = world(10, |c| c.immigration = 2.0);
        two.immigrate();
        assert_eq!(two.population(), 2);
        let mut full = world(5, |c| {
            c.start = Start::Random;
            c.immigration = 3.0;
        });
        let next = full.next_id;
        full.immigrate();
        assert_eq!(
            (full.population(), full.next_id),
            (25, next),
            "immigrants are lost"
        );
    }

    #[test]
    fn offspring_go_to_an_empty_neighbor_or_anywhere() {
        // A parent at (3, 3) certain to reproduce, with `taken` neighbors
        // that cannot.
        let parent_with = |offspring: Offspring, taken: &[(u32, u32)]| {
            let mut w = world(7, |c| {
                c.offspring = offspring;
                c.mutation = 0.0;
            });
            put(&mut w, (3, 3), 0, E, 1);
            for &xy in taken {
                put(&mut w, xy, 0, S, 2);
            }
            for a in w.sites.iter_mut().flatten() {
                a.ptr = if a.lineage == 1 { 1.0 } else { 0.0 };
            }
            w.reproduce();
            w
        };
        let three = [(3, 2), (2, 3), (4, 3)];
        let w = parent_with(Offspring::Adjacent, &three);
        assert_eq!(w.population(), 5);
        let open = w.geometry.at(3, 4, 0).unwrap();
        assert_eq!(
            w.agent(open).map(|a| a.lineage),
            Some(1),
            "the one empty neighbor"
        );
        let four = [(3, 2), (2, 3), (4, 3), (3, 4)];
        assert_eq!(
            parent_with(Offspring::Adjacent, &four).population(),
            5,
            "no room"
        );
        let far = parent_with(Offspring::Anywhere, &four);
        assert_eq!(far.population(), 6, "anywhere finds room");
        let child = far.agents().find(|a| a.id == 6).unwrap();
        assert_eq!(child.lineage, 1);
    }

    #[test]
    fn offspring_do_not_reproduce_in_the_period_they_are_born() {
        let mut w = world(9, |c| {
            c.base_ptr = 1.0;
            c.cost = 0.0;
            c.death = 0.0;
            c.mutation = 0.0;
        });
        put(&mut w, (4, 4), 0, S, 1);
        w.step();
        assert_eq!(w.population(), 2, "one parent, one child");
        w.step();
        assert_eq!(w.population(), 4);
        assert!(w.agents().all(|a| a.lineage == 1));
    }

    #[test]
    fn death_takes_newcomers_too() {
        let mut w = world(9, |c| {
            c.immigration = 1.0;
            c.base_ptr = 1.0;
            c.death = 1.0;
        });
        put(&mut w, (4, 4), 0, S, 1);
        w.step();
        assert_eq!(
            w.population(),
            0,
            "the parent, its child and the immigrant all die"
        );
        assert!(w.next_id >= 4, "the immigrant and a child were made");
    }

    #[test]
    fn the_statistics_count_decisions_pairs_and_shares() {
        let mut w = world(6, |c| c.death = 0.0);
        // A row: E(tag 0, lineage 1), E(tag 0, lineage 1), T(tag 1, lineage 2),
        // H(tag 0, lineage 3); no other neighbors.
        put(&mut w, (0, 0), 0, E, 1);
        put(&mut w, (1, 0), 0, E, 1);
        put(&mut w, (2, 0), 1, T, 2);
        put(&mut w, (3, 0), 0, H, 3);
        w.interact();
        w.tick += 1;
        w.record();
        let s = w.stats.latest().unwrap().clone();
        // Ordered pairs: (0,1) (1,0) (1,2) (2,1) (2,3) (3,2) = 6.
        // Helps: 0→1, 1→0 (same), 2→1, 2→3 (T to others), 3→2 (H) = 5.
        assert_eq!(s.population, 4);
        assert!(close(s.ethnocentric, 0.5) && close(s.traitorous, 0.25));
        assert!(close(s.humanitarian, 0.25) && s.selfish == 0.0 && s.kin == 0.0);
        assert!(close(s.cooperation, 5.0 / 6.0));
        assert!(close(s.same_tag, 2.0 / 6.0));
        assert!(close(s.relatives, 2.0 / 6.0));
        assert!(close(s.kin_help, 2.0 / 5.0));
        assert!(close(s.tag_given_relative, 1.0));
        assert!(close(s.relative_given_tag, 1.0));
        let empty = world(5, |_| {});
        let s0 = empty.stats.latest().unwrap();
        assert!(s0.cooperation.is_nan() && s0.ethnocentric.is_nan());
    }

    #[test]
    fn full_starts_fill_the_lattice() {
        let w = world(10, |c| c.start = Start::Random);
        assert_eq!(w.population(), 100);
        let lineages: std::collections::BTreeSet<u64> = w.agents().map(|a| a.lineage).collect();
        assert_eq!(lineages.len(), 100, "each founds its own lineage");
        let s = world(10, |c| c.start = Start::Selfish);
        assert!(s.agents().all(|a| s.strategy(a) == Strategy::Selfish));
        let tags: std::collections::BTreeSet<u32> = s.agents().map(|a| a.tag).collect();
        assert_eq!(tags.len(), 4);
        let five = world(20, |c| {
            c.start = Start::Random;
            c.colors = 5;
        });
        assert!(five.agents().any(|a| a.tag == 4), "five colours draw tag 4");
    }

    #[test]
    fn inspect_describes_the_agent_and_who_helped_whom() {
        let mut w = world(5, |_| {});
        put(&mut w, (1, 1), 0, E, 1);
        put(&mut w, (2, 1), 0, H, 1);
        put(&mut w, (1, 2), 1, H, 2);
        w.interact();
        let v = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!(
            (v.strategy, v.basis, v.given, v.received),
            (Strategy::Ethnocentric, "tag", 1, 2)
        );
        assert_eq!(v.neighbors.len(), 2);
        let right = &v.neighbors[0];
        assert_eq!(
            (
                right.x,
                right.y,
                right.related,
                right.helped,
                right.helped_by
            ),
            (2, 1, true, 1, 1)
        );
        let down = &v.neighbors[1];
        assert_eq!(
            (down.tag, down.related, down.helped, down.helped_by),
            (1, false, 0, 1)
        );
        assert!(w.inspect(0, 0).unwrap().agent.is_none());
        assert!(w.inspect(5, 0).is_err());
        let json: serde_json::Value = serde_json::from_str(&w.inspect_json(1, 1).unwrap()).unwrap();
        assert_eq!(json["agent"]["strategy"], "E");
        assert_eq!(json["agent"]["neighbors"][1]["strategy"], "H");
    }

    #[test]
    fn frames_draw_empty_sites_dark_and_every_mode() {
        let mut w = world(4, |_| {});
        put(&mut w, (1, 0), 2, T, 1);
        let mut buf = Vec::new();
        w.render("strategy", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 4 * 4 * 4);
        assert_eq!(&buf[0..3], &BACKGROUND);
        assert_eq!(&buf[4..7], &TRAITOROUS);
        w.render("tag", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &LENDER, "tag 2 is the Java's green");
        for mode in ["lineage", "ptr"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_ne!(&buf[4..7], &BACKGROUND);
        }
        assert!(w.render("nope", "", &mut buf).is_err());
        let distinct: std::collections::BTreeSet<Rgb> = (0..40).map(tag_color).collect();
        assert_eq!(distinct.len(), 40);
    }

    #[test]
    fn runs_stop_at_the_end_and_follow_their_seed() {
        let c = EthnoConfig {
            width: 20,
            end: 30,
            ..Default::default()
        };
        let mut a = EthnoWorld::new(c.clone(), 7).unwrap();
        let mut b = EthnoWorld::new(c.clone(), 7).unwrap();
        let mut other = EthnoWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(40);
        }
        assert_eq!(a.tick, 30);
        assert!(a.is_finished() && Model::finished(&a));
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
        assert_eq!(a.stats.history().len(), 31);
        let first = a.agents().next().unwrap().id;
        assert!(a.locate(first).is_some() && a.locate(0).is_none());
        assert_eq!(a.agents_csv().lines().count(), a.population() + 1);
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(10, |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("cost".to_string(), json!(0.02))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.cost, 0.01);
        w.step();
        assert_eq!(w.config.cost, 0.02);
    }

    /// The empty-site list holds exactly the unoccupied sites, each at its slot.
    fn empty_list_is_consistent(w: &EthnoWorld) {
        let unoccupied = w.sites.iter().filter(|a| a.is_none()).count();
        assert_eq!(w.empty.len(), unoccupied);
        for (k, &s) in w.empty.iter().enumerate() {
            assert!(w.sites[s as usize].is_none(), "listed site {s} is occupied");
            assert_eq!(w.slot[s as usize], k as u32);
        }
    }

    #[test]
    fn a_world_that_empties_reports_nan_and_still_draws() {
        let mut w = world(10, |c| c.death = 1.0);
        put(&mut w, (2, 2), 0, H, 1);
        put(&mut w, (2, 3), 0, E, 2);
        w.run(3);
        assert_eq!(w.population(), 0);
        let last = w.stats.latest().unwrap();
        assert!(last.cooperation.is_nan() && last.ethnocentric.is_nan());
        assert!(w.latest_json().contains("\"cooperation\":null"));
        let mut buf = Vec::new();
        for mode in ["strategy", "tag", "lineage", "ptr"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 10 * 10 * 4);
        }
        assert!(w.inspect_json(2, 2).unwrap().contains("\"agent\":null"));
        empty_list_is_consistent(&w);
    }

    #[test]
    fn extreme_colour_counts_run_in_every_discrimination() {
        for colors in [1, 2, 40] {
            for discrimination in [
                Discrimination::SameOther,
                Discrimination::None,
                Discrimination::EachColor,
            ] {
                let mut w = world(12, |c| {
                    c.colors = colors;
                    c.discrimination = discrimination;
                    c.immigration = 3.0;
                    c.mutation = 0.2;
                });
                w.run(60);
                assert!(w.population() > 0, "{colors} {discrimination:?}");
                assert!(w.agents().all(|a| a.tag < colors));
                let bits = w.help_bits();
                assert!(w.agents().all(|a| a.help & !mask(bits) == 0));
                empty_list_is_consistent(&w);
            }
        }
    }

    #[test]
    fn ptr_outside_zero_to_one_only_saturates_the_chance() {
        // PTR 0 never reproduces; PTR above 1 always does (if there is room).
        let mut w = world(10, |c| {
            c.base_ptr = 0.0;
            c.cost = 0.5;
            c.benefit = 0.5;
            c.death = 0.0;
        });
        put(&mut w, (4, 4), 0, H, 1);
        put(&mut w, (4, 5), 0, H, 2);
        w.run(5);
        assert_eq!(w.population(), 2, "PTR 0 − 0.5 + 0.5 = 0: no offspring");
        let mut w = world(10, |c| {
            c.base_ptr = 0.0;
            c.benefit = 5.0;
            c.cost = 0.0;
            c.death = 0.0;
        });
        put(&mut w, (4, 4), 0, H, 1);
        put(&mut w, (4, 5), 0, H, 2);
        w.step();
        assert_eq!(w.population(), 4, "PTR 5 > 1: both reproduce");
        empty_list_is_consistent(&w);
    }

    #[test]
    fn the_empty_list_survives_long_runs_live_changes_and_keyframes() {
        let mut w = world(15, |c| {
            c.immigration = 2.5;
            c.death = 0.2;
        });
        w.run(80);
        empty_list_is_consistent(&w);
        let kept = w.clone();
        w.config.offspring = Offspring::Anywhere;
        w.config.pair_play = PairPlay::Twice;
        w.run(80);
        empty_list_is_consistent(&w);
        let mut back = kept.clone();
        back.run(80);
        let mut again = kept;
        again.run(80);
        assert_eq!(back.fingerprint(), again.fingerprint());
        empty_list_is_consistent(&back);
    }
}
