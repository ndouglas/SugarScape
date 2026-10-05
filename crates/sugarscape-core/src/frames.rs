//! Frame dumps for the Flump studio (see
//! docs/superpowers/specs/2026-09-25-flump-studio-design.md): a shot — a
//! config, a seed, config overrides and agents placed by hand — run tick by
//! tick, recording every agent, the sugar at every site, deaths, births and
//! the statistics series. Spatial games' shots record each generation's
//! strategies instead, the demographic Prisoner's Dilemma's each cycle's
//! agents, births and deaths, ethnocentrism's each period's agents, the tags
//! model's each generation's agents and gifts, image scoring's each
//! generation's agents and meetings, the norms game's each generation's
//! agents and events, and social structure's each period's agents and
//! partners (`run`). Ants shots retain actual source choices and optional
//! source-change events. Other models are not filmed yet.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::{Sex, Tribe};
use crate::config::{Config, FieldError};
use crate::dpd::{DpdConfig, DpdDeath, DpdWorld, SERIES as DPD_SERIES};
use crate::edit::AgentOverrides;
use crate::ethno::{EthnoConfig, EthnoWorld, Strategy, SERIES as ETHNO_SERIES};
use crate::image::{ImageConfig, ImageWorld, SERIES as IMAGE_SERIES};
use crate::model::{Model, ModelConfig, ModelWorld};
use crate::norms::{NormsConfig, NormsWorld, SERIES as NORMS_SERIES};
use crate::presets;
use crate::spatial::{Lattice, SpatialConfig, SpatialWorld, SERIES as SPATIAL_SERIES};
use crate::stats;
use crate::structure::{StructureConfig, StructureWorld, SERIES as STRUCTURE_SERIES};
use crate::tags::{TagsConfig, TagsWorld, SERIES as TAGS_SERIES};
use crate::world::{DeathCause, Trade, World};

/// The dump format's version.
pub const FORMAT: u32 = 1;

/// One beat's run.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shot {
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub config: Option<Value>,
    #[serde(default = "first_seed")]
    pub seed: u64,
    pub ticks: u32,
    /// Config paths to override, applied in sorted key order.
    #[serde(default)]
    pub set: serde_json::Map<String, Value>,
    /// Start every site with no sugar instead of full.
    #[serde(default)]
    pub empty: bool,
    #[serde(default)]
    pub place: Vec<Place>,
    /// Spatial games: rows of `C` and `D`, top row first, replacing the
    /// config's start (a close-up's hand-made board).
    #[serde(default)]
    pub cells: Option<Vec<String>>,
    /// Spatial games: record every player's score in each frame.
    #[serde(default)]
    pub scores: bool,
    /// Tags: record each generation's gifts; image scoring: its meetings; the
    /// norms game: its events; ants: actual source changes (one meeting per
    /// tick for recorded Kirman teaching shots).
    #[serde(default)]
    pub gifts: bool,
    /// Sampled models: record every `every`th update only (a million
    /// updates, filmable); frames keep their true model clock.
    #[serde(default = "every_generation")]
    pub every: u32,
    /// Enable retirement policy after this completed warm-up period.
    #[serde(default)]
    pub retirement_policy_at: Option<u32>,
}

fn every_generation() -> u32 {
    1
}

/// An agent placed by hand when the world reaches `tick`, with the given
/// traits and the rest drawn as for any agent.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Place {
    #[serde(default)]
    pub tick: u64,
    pub x: u32,
    pub y: u32,
    #[serde(default)]
    pub vision: Option<u32>,
    #[serde(default)]
    pub metabolism: Option<u32>,
    #[serde(default)]
    pub sugar: Option<f64>,
    #[serde(default)]
    pub sex: Option<Sex>,
    #[serde(default)]
    pub tribe: Option<Tribe>,
    #[serde(default)]
    pub spice: Option<f64>,
    #[serde(default)]
    pub spice_metabolism: Option<u32>,
    #[serde(default)]
    pub age: Option<u32>,
    #[serde(default)]
    pub endowment: Option<f64>,
}

fn first_seed() -> u64 {
    1
}

impl Shot {
    pub fn from_json(json: &str) -> Result<Shot, Vec<FieldError>> {
        serde_json::from_str(json).map_err(|e| vec![FieldError::new("shot", e.to_string())])
    }

    /// The preset or config, before `set`.
    fn base(&self) -> Result<ModelConfig, Vec<FieldError>> {
        match (&self.preset, &self.config) {
            (Some(id), None) => presets::find(id).map(|p| p.config).ok_or_else(|| {
                vec![FieldError::new(
                    "preset",
                    format!("unknown preset {id:?} (see `sugarscape presets`)"),
                )]
            }),
            (None, Some(value)) => ModelConfig::from_value(value.clone()).map_err(|e| vec![e]),
            _ => Err(vec![FieldError::new(
                "shot",
                "give exactly one of preset or config",
            )]),
        }
    }

    /// The preset or config with `set` applied, validated.
    pub fn config(&self) -> Result<Config, Vec<FieldError>> {
        let mut config = match self.base()? {
            ModelConfig::Sugarscape(c) => c,
            other => {
                return Err(vec![FieldError::new(
                    "model",
                    format!(
                        "shots run the sugarscape only, not {}",
                        other.kind().as_str()
                    ),
                )])
            }
        };
        for (path, value) in &self.set {
            config = config
                .with_path(path, value)
                .map_err(|e| vec![FieldError::new(format!("set.{path}"), e.message)])?;
        }
        config.validate()?;
        Ok(config)
    }

    /// The preset or config with `set` applied and validated, for a model
    /// other than the Sugarscape.
    fn model_config(&self) -> Result<ModelConfig, Vec<FieldError>> {
        let mut config = self.base()?;
        for (path, value) in &self.set {
            config = config
                .with_path(path, value)
                .map_err(|e| vec![FieldError::new(format!("set.{path}"), e.message)])?;
        }
        config.validate()?;
        Ok(config)
    }

    /// A demographic-PD shot's config with `set` applied, validated.
    pub fn dpd_config(&self) -> Result<DpdConfig, Vec<FieldError>> {
        match self.model_config()? {
            ModelConfig::Dpd(c) => Ok(c),
            other => Err(vec![FieldError::new(
                "model",
                format!("not a demographic-PD shot: {}", other.kind().as_str()),
            )]),
        }
    }

    /// A social-structure shot's config with `set` applied, validated.
    pub fn structure_config(&self) -> Result<StructureConfig, Vec<FieldError>> {
        match self.model_config()? {
            ModelConfig::Structure(c) => Ok(c),
            other => Err(vec![FieldError::new(
                "model",
                format!("not a social-structure shot: {}", other.kind().as_str()),
            )]),
        }
    }

    /// A norms shot's config with `set` applied, validated.
    pub fn norms_config(&self) -> Result<NormsConfig, Vec<FieldError>> {
        match self.model_config()? {
            ModelConfig::Norms(c) => Ok(c),
            other => Err(vec![FieldError::new(
                "model",
                format!("not a norms shot: {}", other.kind().as_str()),
            )]),
        }
    }

    /// An image-scoring shot's config with `set` applied, validated.
    pub fn image_config(&self) -> Result<ImageConfig, Vec<FieldError>> {
        match self.model_config()? {
            ModelConfig::Image(c) => Ok(c),
            other => Err(vec![FieldError::new(
                "model",
                format!("not an image-scoring shot: {}", other.kind().as_str()),
            )]),
        }
    }

    /// A tags shot's config with `set` applied, validated.
    pub fn tags_config(&self) -> Result<TagsConfig, Vec<FieldError>> {
        match self.model_config()? {
            ModelConfig::Tags(c) => Ok(c),
            other => Err(vec![FieldError::new(
                "model",
                format!("not a tags shot: {}", other.kind().as_str()),
            )]),
        }
    }

    /// An ethnocentrism shot's config with `set` applied, validated.
    pub fn ethno_config(&self) -> Result<EthnoConfig, Vec<FieldError>> {
        match self.model_config()? {
            ModelConfig::Ethno(c) => Ok(c),
            other => Err(vec![FieldError::new(
                "model",
                format!("not an ethnocentrism shot: {}", other.kind().as_str()),
            )]),
        }
    }

    /// A spatial-games shot's config with `set` applied, validated: a flat
    /// lattice (a square grid or a random array), as a board can show.
    pub fn spatial_config(&self) -> Result<SpatialConfig, Vec<FieldError>> {
        let mut config = self.base()?;
        if !matches!(config, ModelConfig::Spatial(_)) {
            return Err(vec![FieldError::new(
                "model",
                format!("not a spatial-games shot: {}", config.kind().as_str()),
            )]);
        }
        for (path, value) in &self.set {
            config = config
                .with_path(path, value)
                .map_err(|e| vec![FieldError::new(format!("set.{path}"), e.message)])?;
        }
        config.validate()?;
        let ModelConfig::Spatial(config) = config else {
            unreachable!("set keeps the model")
        };
        if config.lattice == Lattice::Cube {
            return Err(vec![FieldError::new(
                "lattice",
                "a board shows one layer: film a square grid or a random array",
            )]);
        }
        Ok(config)
    }
}

/// `[id, x, y, sugar, age, vision, metabolism]`.
pub type AgentRow = (u64, u32, u32, f64, u32, u32, u32);

/// `[id, sex, [parent, parent] | null]`.
pub type Birth = (u64, Sex, Option<[u64; 2]>);

/// A tick's exchanges between two agents under rule T, merged:
/// `[sugar giver, spice giver, sugar, spice, exchanges]`.
pub type PairTrade = (u64, u64, f64, f64, u32);

/// The world after a tick and that tick's placements.
#[derive(Clone, Debug, Serialize)]
pub struct Frame {
    pub tick: u64,
    pub agents: Vec<AgentRow>,
    /// Good 0's level at every site, row-major.
    pub sugar: Vec<f64>,
    /// Pollutant 0's level at every site, row-major (zero while pollution is off).
    pub pollution: Vec<f64>,
    /// Agents that died during this tick, with the cause.
    pub deaths: Vec<(u64, &'static str)>,
    /// Agents first seen in this frame (the initial population, replacements
    /// and placements).
    pub born: Vec<u64>,
    /// The same agents as `[id, sex, [parent, parent] | null]`: parents for
    /// children born under rule S, none for founders, placements and
    /// replacements.
    pub births: Vec<Birth>,
    /// Each agent's cultural tags as a bit string, in `agents`' order.
    pub tags: Vec<String>,
    /// Each agent's group (tribe) under the config's groups, in `agents`' order.
    pub groups: Vec<usize>,
    /// Good 1's level at every site, row-major (absent with one good).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spice: Vec<f64>,
    /// Each agent's `[spice, spice metabolism]`, in `agents`' order (absent
    /// with one good).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spice_agents: Vec<(f64, u32)>,
    /// This tick's sugar-for-spice exchanges, merged by pair (absent when
    /// there were none).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub trades: Vec<PairTrade>,
    /// This tick's kills under rule C as `[attacker, victim, loot]`, in the
    /// order they happened (absent when there were none).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub kills: Vec<(u64, u64, f64)>,
    /// The loans outstanding after this tick under rule L as `[lender,
    /// borrower, amount due, due tick]`, by loan id (absent when none). A
    /// loan whose due tick is this frame's is settled in the next step.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub loans: Vec<(u64, u64, f64, u64)>,
    /// The newcomers' fertile ages as `[id, onset, end]`, with sex on (absent
    /// otherwise): lenders past childbearing, borrowers within it.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fertility: Vec<(u64, u32, u32)>,
    /// How many diseases each agent carries, in `agents`' order (absent with
    /// disease off).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diseases: Vec<usize>,
    /// This tick's infections under rule E as `[infector | null, infected,
    /// disease]`, null for an outbreak (absent when none).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub infections: Vec<(Option<u64>, u64, u32)>,
    /// Each agent's Axelrod culture (its traits), in `agents`' order (absent
    /// unless rule K is Axelrod's).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cultures: Vec<Vec<u8>>,
}

/// A whole shot, tick 0 first. Stats are the engine's series as recorded
/// (each tick's snapshot is taken before that tick's placements).
#[derive(Clone, Debug, Serialize)]
pub struct FrameDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub width: u32,
    pub height: u32,
    /// Good 0's capacity at every site, row-major.
    pub capacity: Vec<f64>,
    /// Good 1's capacity at every site, row-major (absent with one good).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub spice_capacity: Vec<f64>,
    /// The placed agents' ids, in the order of the shot's `place`.
    pub placed: Vec<u64>,
    pub config: Config,
    pub frames: Vec<Frame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

fn cause_name(cause: DeathCause) -> &'static str {
    match cause {
        DeathCause::Starvation => "starvation",
        DeathCause::OldAge => "old_age",
        DeathCause::Combat => "combat",
    }
}

/// Merges a tick's exchanges of sugar (good 0) for spice (good 1) by pair,
/// in the order of the pairs' ids; exchanges of other goods are left out.
fn trades(events: &[Trade]) -> Vec<PairTrade> {
    let mut pairs: BTreeMap<(u64, u64), (f64, f64, u32)> = BTreeMap::new();
    for t in events {
        let (got, paid) = (t.amount, t.amount * t.price);
        let (key, sugar, spice) = match t.goods {
            (0, 1) => ((t.seller, t.buyer), got, paid),
            (1, 0) => ((t.buyer, t.seller), paid, got),
            _ => continue,
        };
        let e = pairs.entry(key).or_default();
        e.0 += sugar;
        e.1 += spice;
        e.2 += 1;
    }
    pairs
        .into_iter()
        .map(|((a, b), (sugar, spice, n))| (a, b, sugar, spice, n))
        .collect()
}

fn frame(
    world: &World,
    seen: &mut BTreeSet<u64>,
    deaths: Vec<(u64, &'static str)>,
    trades: Vec<PairTrade>,
    kills: Vec<(u64, u64, f64)>,
    infections: Vec<(Option<u64>, u64, u32)>,
) -> Frame {
    let agents: Vec<AgentRow> = world
        .agents()
        .map(|a| {
            let (x, y) = (a.pos.x, a.pos.y);
            (a.id, x, y, a.holdings[0], a.age, a.vision, a.metabolism[0])
        })
        .collect();
    let born: Vec<u64> = agents
        .iter()
        .map(|a| a.0)
        .filter(|&id| seen.insert(id))
        .collect();
    let births = born
        .iter()
        .filter_map(|&id| world.agent(id))
        .map(|a| (a.id, a.sex, a.parents))
        .collect();
    let (tags, groups) = world
        .agents()
        .map(|a| {
            (
                a.tags.to_bit_string(),
                a.group(&world.config.culture.groups),
            )
        })
        .unzip();
    let loans = world
        .loans()
        .map(|l| (l.lender, l.borrower, l.due, l.due_tick))
        .collect();
    let fertility = if world.config.sex.enabled {
        born.iter()
            .filter_map(|&id| world.agent(id))
            .map(|a| (a.id, a.fertility_onset, a.fertility_end))
            .collect()
    } else {
        Vec::new()
    };
    let diseases = if world.config.disease.enabled {
        world.agents().map(|a| a.diseases.len()).collect()
    } else {
        Vec::new()
    };
    let two_goods = world.config.goods.len() > 1;
    let (spice, spice_agents) = if two_goods {
        (
            world.sites.iter().map(|s| s.resource[1]).collect(),
            world
                .agents()
                .map(|a| (a.holdings[1], a.metabolism[1]))
                .collect(),
        )
    } else {
        (Vec::new(), Vec::new())
    };
    let cultures = if world.config.culture.axelrod() {
        world.agents().map(|a| a.culture.clone()).collect()
    } else {
        Vec::new()
    };
    Frame {
        tick: world.tick,
        tags,
        groups,
        cultures,
        spice,
        spice_agents,
        trades,
        kills,
        loans,
        fertility,
        diseases,
        infections,
        agents,
        sugar: world.sites.iter().map(|s| s.resource[0]).collect(),
        pollution: world.sites.iter().map(|s| s.pollution[0]).collect(),
        deaths,
        born,
        births,
    }
}

/// Places the shot's agents for the world's current tick, recording each
/// placement's index in `place` and its id.
fn place(
    world: &mut World,
    shot: &Shot,
    placed: &mut Vec<(usize, u64)>,
) -> Result<(), Vec<FieldError>> {
    for (i, p) in shot.place.iter().enumerate() {
        if p.tick != world.tick {
            continue;
        }
        let overrides = AgentOverrides {
            vision: p.vision,
            metabolism: p.metabolism,
            sugar: p.sugar,
            sex: p.sex,
            tribe: p.tribe,
            spice: p.spice,
            spice_metabolism: p.spice_metabolism,
            age: p.age,
            endowment: p.endowment,
        };
        let id = world
            .place_agent(p.x, p.y, &overrides)
            .map_err(|message| vec![FieldError::new(format!("place.{i}"), message)])?;
        placed.push((i, id));
    }
    Ok(())
}

/// Runs `shot` and records every tick.
pub fn run_shot(shot: &Shot) -> Result<FrameDump, Vec<FieldError>> {
    let config = shot.config()?;
    for (i, p) in shot.place.iter().enumerate() {
        if p.tick > u64::from(shot.ticks) {
            return Err(vec![FieldError::new(
                format!("place.{i}.tick"),
                format!("is after the shot's last tick ({})", shot.ticks),
            )]);
        }
    }
    let mut world = World::new(config.clone(), shot.seed)?;
    if shot.empty {
        for site in &mut world.sites {
            site.resource[0] = 0.0;
        }
    }
    let capacity = world.capacities(0);
    let spice_capacity = if config.goods.len() > 1 {
        world.capacities(1)
    } else {
        Vec::new()
    };
    let mut seen = BTreeSet::new();
    let mut placed = Vec::new();
    place(&mut world, shot, &mut placed)?;
    let mut frames = vec![frame(
        &world,
        &mut seen,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )];
    for _ in 0..shot.ticks {
        world.step();
        let deaths = world
            .events()
            .deaths
            .iter()
            .map(|d| (d.id, cause_name(d.cause)))
            .collect();
        let traded = trades(&world.events().trades);
        let kills = world
            .events()
            .kills
            .iter()
            .map(|k| (k.attacker, k.victim, k.loot))
            .collect();
        let infections = world
            .events()
            .infections
            .iter()
            .map(|i| (i.infector, i.infected, i.disease))
            .collect();
        place(&mut world, shot, &mut placed)?;
        frames.push(frame(&world, &mut seen, deaths, traded, kills, infections));
    }
    placed.sort_by_key(|&(i, _)| i);
    let stats = stats::series_names(&config)
        .into_iter()
        .filter_map(|name| world.stats.series(&name).map(|s| (name, s)))
        .collect();
    Ok(FrameDump {
        format: FORMAT,
        model: "sugarscape",
        seed: shot.seed,
        ticks: shot.ticks,
        width: config.width,
        height: config.height,
        capacity,
        spice_capacity,
        placed: placed.into_iter().map(|(_, id)| id).collect(),
        config,
        frames,
        stats,
    })
}

/// A spatial-games shot's world after one generation.
#[derive(Clone, Debug, Serialize)]
pub struct LatticeFrame {
    pub tick: u64,
    /// Each square's player, row-major: `C`, `D`, or `.` for none.
    pub strategies: String,
    /// Each player's score, in `strategies`' order of players (absent unless
    /// the shot asks).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scores: Vec<f64>,
}

/// A whole spatial-games shot, generation 0 first.
#[derive(Clone, Debug, Serialize)]
pub struct LatticeDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub width: u32,
    pub height: u32,
    pub config: SpatialConfig,
    pub frames: Vec<LatticeFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// `[id, x, y, wealth, age, "C" | "D"]`.
pub type DpdRow = (u64, u32, u32, f64, u32, &'static str);

/// A demographic-PD shot's world after one cycle.
#[derive(Clone, Debug, Serialize)]
pub struct DpdFrame {
    pub tick: u64,
    pub agents: Vec<DpdRow>,
    /// This cycle's births as `[offspring, parent]`, of the offspring alive
    /// at its end.
    pub births: Vec<(u64, u64)>,
    /// This cycle's deaths as `[id, "broke" | "old_age"]`, of agents alive
    /// at the cycle's start.
    pub deaths: Vec<(u64, &'static str)>,
}

/// A whole demographic-PD shot, cycle 0 first.
#[derive(Clone, Debug, Serialize)]
pub struct DpdDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub width: u32,
    pub height: u32,
    pub config: DpdConfig,
    pub frames: Vec<DpdFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// `[id, x, y, tag, kind, founding immigrant]`, the kind as a letter (E, H,
/// S, T) or a word (kin, nonkin, mixed).
pub type EthnoRow = (u64, u32, u32, u32, Strategy, u64);

/// An ethnocentrism shot's world after one period. Agents never move, so
/// births and deaths are the differences between consecutive frames.
#[derive(Clone, Debug, Serialize)]
pub struct EthnoFrame {
    pub tick: u64,
    pub agents: Vec<EthnoRow>,
}

/// A whole ethnocentrism shot, period 0 first.
#[derive(Clone, Debug, Serialize)]
pub struct EthnoDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub width: u32,
    pub height: u32,
    pub config: EthnoConfig,
    pub frames: Vec<EthnoFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// `[id, parent, tag, tolerance, gifts given, gifts received]`, in the
/// population's list order: place k's agent descends from place k's before
/// it or from its opponent's.
pub type TagsRow = (u64, u64, f64, f64, u32, u32);

/// A tags shot's population after one generation.
#[derive(Clone, Debug, Serialize)]
pub struct TagsFrame {
    pub tick: u64,
    pub agents: Vec<TagsRow>,
    /// This generation's gifts as `[giver, receiver]` places in the list,
    /// in the order they happened (absent unless the shot asks, with
    /// `"gifts": true`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gifts: Vec<(u32, u32)>,
}

/// A whole tags shot, generation 0 first.
#[derive(Clone, Debug, Serialize)]
pub struct TagsDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub config: TagsConfig,
    pub frames: Vec<TagsFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// `[id, k | null, score, payoff]`, in the population's list order (group
/// by group): k for the threshold strategies (k, AND, OR, binary), null for
/// the others.
pub type ImageRow = (u64, Option<i8>, i32, f64);

/// An image-scoring shot's population after one generation has played.
#[derive(Clone, Debug, Serialize)]
pub struct ImageFrame {
    pub tick: u64,
    pub agents: Vec<ImageRow>,
    /// This generation's meetings as `[donor, recipient, helped]` places in
    /// the list, in the order they happened (absent unless the shot asks,
    /// with `"gifts": true`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub meetings: Vec<(u32, u32, bool)>,
}

/// A whole image-scoring shot, generation 0 (before it plays) first.
#[derive(Clone, Debug, Serialize)]
pub struct ImageDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub config: ImageConfig,
    pub frames: Vec<ImageFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// `[boldness, vengefulness, payoff, parent | null]`, levels 0–7, in the
/// generation's list order; the parent is a place in the previous
/// generation's list.
pub type NormsRow = (u8, u8, f64, Option<u32>);

/// A norms shot's generation after it played.
#[derive(Clone, Debug, Serialize)]
pub struct NormsFrame {
    /// The true generation (frames are every `every`th).
    pub tick: u64,
    pub agents: Vec<NormsRow>,
    /// This generation's events, by places in `agents` (absent unless the
    /// shot asks, with `"gifts": true`): each cheat, each punishment as
    /// `[punisher, cheat]`, each metapunishment as `[punisher, onlooker]`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub cheats: Vec<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub punishments: Vec<(u32, u32)>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub metapunishments: Vec<(u32, u32)>,
}

/// A whole norms shot, generation 0 (before any play) first.
#[derive(Clone, Debug, Serialize)]
pub struct NormsDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    /// Generations run (frames: `ticks / every + 1`).
    pub ticks: u32,
    pub every: u32,
    pub config: NormsConfig,
    pub frames: Vec<NormsFrame>,
    /// The series at the recorded generations only.
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// `[y, p, q, score]`: a strategy as played this period, and its payoff
/// per move.
pub type StructureRow = (f64, f64, f64, f64);

/// A social-structure shot's population after one period.
#[derive(Clone, Debug, Serialize)]
pub struct StructureFrame {
    pub tick: u64,
    pub agents: Vec<StructureRow>,
    /// Each agent's partners this period (whether it chose them or they
    /// chose it), in agent order (absent unless the shot asks, with
    /// `"gifts": true`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub partners: Vec<Vec<u32>>,
}

/// A whole social-structure shot, period 0 (before any play) first.
#[derive(Clone, Debug, Serialize)]
pub struct StructureDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub config: StructureConfig,
    /// On the torus, each agent's square, row-major (16 × 16 for 256);
    /// empty for the other structures.
    pub site: Vec<u32>,
    pub frames: Vec<StructureFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// `[id, x, y, red, content]`: a Schelling agent's square, color and
/// whether it is content there (on the line, x is its place in the row).
pub type SchellingRow = (u64, u32, u32, bool, bool);

/// A Schelling board or line after one round.
#[derive(Clone, Debug, Serialize)]
pub struct SchellingFrame {
    pub tick: u64,
    /// Sorted by id.
    pub agents: Vec<SchellingRow>,
}

/// A whole Schelling shot (`model` "schelling" for the board, "line" for
/// the row, laid out `width` places a row), round 0 first.
#[derive(Clone, Debug, Serialize)]
pub struct SchellingDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    /// Frames after the first (the shot's `ticks / every`).
    pub ticks: u32,
    /// Steps between frames (frames keep their true step).
    pub every: u32,
    pub width: u32,
    pub height: u32,
    pub config: ModelConfig,
    pub frames: Vec<SchellingFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// One step of Schelling's bounded neighborhood: the insiders of each
/// color as `[rank, content]` (rank 0 the most tolerant).
#[derive(Clone, Debug, Serialize)]
pub struct TippingFrame {
    pub tick: u64,
    pub red: Vec<(u32, bool)>,
    pub blue: Vec<(u32, bool)>,
}

/// A whole tipping shot, step 0 first, with each color's tolerances (Red's
/// first, most tolerant first).
#[derive(Clone, Debug, Serialize)]
pub struct TippingDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub red: u32,
    pub blue: u32,
    pub tolerances: [Vec<f64>; 2],
    pub config: crate::tipping::TippingConfig,
    pub frames: Vec<TippingFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// One recorded step of Axelrod's culture model: every site's traits,
/// row-major, `features` per site.
#[derive(Clone, Debug, Serialize)]
pub struct CultureFrame {
    pub tick: u64,
    pub traits: Vec<u8>,
}

/// A culture shot: the lattice's traits every `every`th step (frames keep
/// their true step) and the model's statistics at the same steps.
#[derive(Clone, Debug, Serialize)]
pub struct CultureDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    /// Frames after the first (the shot's `ticks / every`).
    pub ticks: u32,
    pub every: u32,
    pub width: u32,
    pub height: u32,
    pub features: u32,
    pub config: ModelConfig,
    pub frames: Vec<CultureFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// A shot's dump, of whichever model it runs.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Dump {
    Sugarscape(Box<FrameDump>),
    Spatial(LatticeDump),
    Dpd(DpdDump),
    Ethno(EthnoDump),
    Tags(TagsDump),
    Image(ImageDump),
    Norms(NormsDump),
    Structure(StructureDump),
    Schelling(Box<SchellingDump>),
    Tipping(Box<TippingDump>),
    Culture(Box<CultureDump>),
    Opinions(Box<OpinionsDump>),
    Agreement(Box<AgreementDump>),
    Thresholds(Box<ThresholdsDump>),
    Ants(Box<AntsDump>),
    Farol(Box<FarolDump>),
    Retirement(Box<RetirementDump>),
}

/// Runs `shot`, whatever its model.
pub fn run(shot: &Shot) -> Result<Dump, Vec<FieldError>> {
    let only = |field: &str, model: &str| {
        vec![FieldError::new(field, format!("is for {model} shots only"))]
    };
    if shot.retirement_policy_at.is_some() && !matches!(shot.base()?, ModelConfig::Retirement(_)) {
        return Err(only("retirement_policy_at", "retirement"));
    }
    match shot.base()? {
        ModelConfig::Retirement(_) => run_retirement(shot).map(|d| Dump::Retirement(Box::new(d))),
        ModelConfig::Sugarscape(_) => {
            if shot.every != 1 {
                return Err(only("every", "norms"));
            }
            if shot.gifts {
                return Err(only("gifts", "tags and image-scoring"));
            }
            if shot.cells.is_some() {
                return Err(only("cells", "spatial-games"));
            }
            if shot.scores {
                return Err(only("scores", "spatial-games"));
            }
            run_shot(shot).map(|d| Dump::Sugarscape(Box::new(d)))
        }
        ModelConfig::Spatial(_)
        | ModelConfig::Dpd(_)
        | ModelConfig::Ethno(_)
        | ModelConfig::Tags(_)
        | ModelConfig::Image(_)
        | ModelConfig::Norms(_)
        | ModelConfig::Structure(_) => {
            if !shot.place.is_empty() {
                return Err(only("place", "sugarscape"));
            }
            if shot.empty {
                return Err(only("empty", "sugarscape"));
            }
            if matches!(shot.base()?, ModelConfig::Norms(_)) {
                if shot.cells.is_some() || shot.scores {
                    return Err(only(if shot.scores { "scores" } else { "cells" }, "spatial-games"));
                }
                return run_norms(shot).map(Dump::Norms);
            }
            if shot.every != 1 {
                return Err(only("every", "norms"));
            }
            if matches!(shot.base()?, ModelConfig::Structure(_)) {
                if shot.cells.is_some() || shot.scores {
                    return Err(only(if shot.scores { "scores" } else { "cells" }, "spatial-games"));
                }
                return run_structure(shot).map(Dump::Structure);
            }
            if matches!(shot.base()?, ModelConfig::Image(_)) {
                if shot.cells.is_some() || shot.scores {
                    return Err(only(if shot.scores { "scores" } else { "cells" }, "spatial-games"));
                }
                return run_image(shot).map(Dump::Image);
            }
            if matches!(shot.base()?, ModelConfig::Tags(_)) {
                if shot.cells.is_some() || shot.scores {
                    return Err(only(if shot.scores { "scores" } else { "cells" }, "spatial-games"));
                }
                return run_tags(shot).map(Dump::Tags);
            }
            if shot.gifts {
                return Err(only("gifts", "tags and image-scoring"));
            }
            if matches!(shot.base()?, ModelConfig::Spatial(_)) {
                return run_lattice(shot).map(Dump::Spatial);
            }
            if shot.cells.is_some() {
                return Err(only("cells", "spatial-games"));
            }
            if shot.scores {
                return Err(only("scores", "spatial-games"));
            }
            if matches!(shot.base()?, ModelConfig::Ethno(_)) {
                return run_ethno(shot).map(Dump::Ethno);
            }
            run_dpd(shot).map(Dump::Dpd)
        }
        ModelConfig::Schelling(_) | ModelConfig::Line(_) | ModelConfig::Tipping(_) => {
            for (bad, field) in [
                (!shot.place.is_empty(), "place"),
                (shot.empty, "empty"),
                (shot.gifts, "gifts"),
                (shot.cells.is_some(), "cells"),
                (shot.scores, "scores"),
            ] {
                if bad {
                    return Err(vec![FieldError::new(field, "is not for Schelling shots")]);
                }
            }
            if let ModelConfig::Tipping(_) = shot.base()? {
                if shot.every != 1 {
                    return Err(only("every", "norms and Schelling-board"));
                }
                return run_tipping(shot).map(|d| Dump::Tipping(Box::new(d)));
            }
            run_schelling(shot).map(|d| Dump::Schelling(Box::new(d)))
        }
        ModelConfig::Farol(_) => run_farol(shot).map(|d| Dump::Farol(Box::new(d))),
        ModelConfig::Ants(_) => run_ants(shot).map(|d| Dump::Ants(Box::new(d))),
        ModelConfig::Thresholds(_) => {
            run_thresholds(shot).map(|d| Dump::Thresholds(Box::new(d)))
        }
        ModelConfig::Agreement(_) => {
            run_agreement(shot).map(|d| Dump::Agreement(Box::new(d)))
        }
        ModelConfig::Opinions(_) => {
            for (bad, field) in [
                (!shot.place.is_empty(), "place"),
                (shot.empty, "empty"),
                (shot.gifts, "gifts"),
                (shot.cells.is_some(), "cells"),
                (shot.scores, "scores"),
            ] {
                if bad {
                    return Err(vec![FieldError::new(field, "is not for opinions shots")]);
                }
            }
            run_opinions(shot).map(|d| Dump::Opinions(Box::new(d)))
        }
        ModelConfig::Culture(_) => {
            for (bad, field) in [
                (!shot.place.is_empty(), "place"),
                (shot.empty, "empty"),
                (shot.gifts, "gifts"),
                (shot.cells.is_some(), "cells"),
                (shot.scores, "scores"),
            ] {
                if bad {
                    return Err(vec![FieldError::new(field, "is not for culture shots")]);
                }
            }
            run_culture(shot).map(|d| Dump::Culture(Box::new(d)))
        }
        other => Err(vec![FieldError::new(
            "model",
            format!(
                "shots run the sugarscape, spatial games, the demographic PD, ethnocentrism, tags, image scoring, norms, social structure, Schelling's board and line, Axelrod's culture and bounded confidence, not {}",
                other.kind().as_str()
            ),
        )]),
    }
}

/// The squares' strategies as `C`, `D` and `.`, row-major.
fn strategies(world: &SpatialWorld) -> String {
    let (w, h, _) = world.geometry.dims;
    (0..h)
        .flat_map(|y| (0..w).map(move |x| (x, y)))
        .map(|(x, y)| match world.geometry.at(x, y, 0) {
            Some(i) if world.is_cooperator(i) => 'C',
            Some(_) => 'D',
            None => '.',
        })
        .collect()
}

fn lattice_frame(world: &SpatialWorld, scores: bool) -> LatticeFrame {
    let (w, h, _) = world.geometry.dims;
    LatticeFrame {
        tick: world.tick,
        strategies: strategies(world),
        scores: if scores {
            (0..h)
                .flat_map(|y| (0..w).map(move |x| (x, y)))
                .filter_map(|(x, y)| world.geometry.at(x, y, 0))
                .map(|i| world.score(i))
                .collect()
        } else {
            Vec::new()
        },
    }
}

/// Reads a shot's `cells` into one strategy per player (true: C).
fn start_cells(world: &SpatialWorld, rows: &[String]) -> Result<Vec<bool>, Vec<FieldError>> {
    let bad = |message: String| vec![FieldError::new("cells", message)];
    if world.config.lattice != Lattice::Square {
        return Err(bad("a hand-made start needs a square grid".into()));
    }
    let (w, h, _) = world.geometry.dims;
    if rows.len() != h as usize {
        return Err(bad(format!("has {} rows; the grid has {h}", rows.len())));
    }
    let mut coop = vec![true; world.players()];
    for (y, row) in rows.iter().enumerate() {
        if row.chars().count() != w as usize {
            return Err(bad(format!("row {y} is not {w} squares wide")));
        }
        for (x, c) in row.chars().enumerate() {
            let i = world
                .geometry
                .at(x as u32, y as u32, 0)
                .expect("a square grid has a player on every square");
            coop[i] = match c {
                'C' => true,
                'D' => false,
                _ => return Err(bad(format!("row {y} has {c:?}: use C and D"))),
            };
        }
    }
    Ok(coop)
}

/// Runs a spatial-games shot and records every generation.
pub fn run_lattice(shot: &Shot) -> Result<LatticeDump, Vec<FieldError>> {
    let config = shot.spatial_config()?;
    let mut world = SpatialWorld::new(config.clone(), shot.seed)?;
    if let Some(rows) = &shot.cells {
        let coop = start_cells(&world, rows)?;
        world.set_start(coop);
    }
    let mut frames = vec![lattice_frame(&world, shot.scores)];
    for _ in 0..shot.ticks {
        world.step();
        frames.push(lattice_frame(&world, shot.scores));
    }
    let stats = SPATIAL_SERIES
        .iter()
        .filter_map(|&name| world.stats.series(name).map(|s| (name.to_string(), s)))
        .collect();
    let (width, height, _) = world.geometry.dims;
    Ok(LatticeDump {
        format: FORMAT,
        model: "spatial",
        seed: shot.seed,
        ticks: shot.ticks,
        width,
        height,
        config,
        frames,
        stats,
    })
}

fn dpd_frame(world: &DpdWorld, before: &BTreeSet<u64>, alive: &mut BTreeSet<u64>) -> DpdFrame {
    let w = world.config.width;
    let agents: Vec<DpdRow> = world
        .agents()
        .map(|a| {
            let strategy = if a.cooperator { "C" } else { "D" };
            (a.id, a.site % w, a.site / w, a.wealth, a.age, strategy)
        })
        .collect();
    *alive = agents.iter().map(|a| a.0).collect();
    let events = world.events();
    DpdFrame {
        tick: world.tick,
        births: events
            .births
            .iter()
            .filter(|b| alive.contains(&b.0))
            .copied()
            .collect(),
        deaths: events
            .deaths
            .iter()
            .filter(|d| before.contains(&d.0))
            .map(|&(id, cause)| {
                let cause = match cause {
                    DpdDeath::Broke => "broke",
                    DpdDeath::OldAge => "old_age",
                };
                (id, cause)
            })
            .collect(),
        agents,
    }
}

/// Runs a demographic-PD shot and records every cycle.
/// Every series at steps 0, `every`, 2·`every` … `ticks`; a model that stopped
/// early (stable) holds its last value.
fn stats_every(model: &dyn Model, ticks: u32, every: u32) -> BTreeMap<String, Vec<f64>> {
    model
        .series_names()
        .into_iter()
        .filter_map(|name| {
            model.series(&name).map(|s| {
                let every = every as usize;
                let last = *s.last().unwrap_or(&0.0);
                let picked = (0..=ticks as usize / every)
                    .map(|k| s.get(k * every).copied().unwrap_or(last))
                    .collect();
                (name, picked)
            })
        })
        .collect()
}

/// All actual slots, including renewed agents and arbitrarily large current cohorts.
#[derive(Clone, Debug, Serialize)]
pub struct RetirementAgent {
    pub id: u32,
    pub born: i64,
    pub age: u32,
    pub kind: crate::retirement::Kind,
    pub retired: bool,
    pub retired_at: Option<u32>,
    pub group: u8,
    pub threshold_units: u64,
    pub network: Vec<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RetirementFrame {
    #[serde(flatten)]
    pub period: crate::retirement::RetirementPeriod,
    pub agents: Vec<RetirementAgent>,
    pub cohort_counts: BTreeMap<u32, u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RetirementDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub every: u32,
    pub agents: usize,
    pub config: ModelConfig,
    pub retirement_policy_at: Option<u32>,
    pub policy_switched_at: Option<u64>,
    pub age_min: u32,
    pub age_max: u32,
    pub frames: Vec<RetirementFrame>,
    pub periods: Vec<crate::retirement::RetirementPeriod>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

pub fn run_retirement(shot: &Shot) -> Result<RetirementDump, Vec<FieldError>> {
    for (bad, field) in [
        (!shot.place.is_empty(), "place"),
        (shot.empty, "empty"),
        (shot.cells.is_some(), "cells"),
        (shot.scores, "scores"),
    ] {
        if bad {
            return Err(vec![FieldError::new(field, "is not for retirement shots")]);
        }
    }
    if shot.every == 0 || !shot.ticks.is_multiple_of(shot.every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let config = shot.model_config()?;
    let ModelConfig::Retirement(c) = &config else {
        return Err(vec![FieldError::new("model", "not a retirement shot")]);
    };
    if let Some(at) = shot.retirement_policy_at {
        if at == 0
            || at >= shot.ticks
            || c.policy.enabled
            || c.stop_at_norm
            || (c.stop_at > 0 && c.stop_at <= at)
        {
            return Err(vec![FieldError::new("retirement_policy_at",
                "requires a positive period before the horizon, initial policy disabled and no earlier stopping")]);
        }
    }
    let mut world = crate::retirement::RetirementWorld::new(c.clone(), shot.seed)?;
    let frame = |w: &crate::retirement::RetirementWorld, period| {
        let mut cohort_counts = BTreeMap::new();
        let agents = w
            .agents()
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let age = w.age(i);
                *cohort_counts.entry(age).or_insert(0) += 1;
                RetirementAgent {
                    id: i as u32,
                    born: a.born,
                    age,
                    kind: a.kind,
                    retired: a.retired,
                    retired_at: a.retired_at,
                    group: a.group,
                    threshold_units: a.threshold,
                    network: a.network.clone(),
                }
            })
            .collect();
        RetirementFrame {
            period,
            agents,
            cohort_counts,
        }
    };
    let mut periods = vec![world.initial_period()];
    let mut frames = vec![frame(&world, periods[0].clone())];
    let mut policy_switched_at = None;
    for _ in 0..shot.ticks {
        if world.is_finished() {
            break;
        }
        let period = world.step_recorded(shot.gifts);
        if period.policy_switched {
            policy_switched_at = Some(period.tick);
        }
        if world.tick.is_multiple_of(u64::from(shot.every)) {
            frames.push(frame(&world, period.clone()));
        }
        periods.push(period);
        if shot.retirement_policy_at == Some(world.tick as u32) {
            let mut next = world.config.clone();
            next.policy.enabled = true;
            world.set_config(ModelConfig::Retirement(next))?;
        }
    }
    if frames.last().unwrap().period.tick != world.tick {
        frames.push(frame(&world, periods.last().unwrap().clone()));
    }
    Ok(RetirementDump {
        format: FORMAT,
        model: "retirement",
        seed: shot.seed,
        ticks: (frames.len() - 1) as u32,
        every: shot.every,
        agents: world.agents().len(),
        config,
        retirement_policy_at: shot.retirement_policy_at,
        policy_switched_at,
        age_min: 20,
        age_max: 100,
        frames,
        periods,
        stats: stats_every(&world, world.tick as u32, 1),
    })
}

/// One recorded period of bounded confidence: every agent's opinion.
#[derive(Clone, Debug, Serialize)]
pub struct OpinionsFrame {
    pub tick: u64,
    pub opinions: Vec<f64>,
}

/// An opinions shot: every agent's opinion every `every`th period (frames
/// keep their true period), the starting opinions and the statistics.
#[derive(Clone, Debug, Serialize)]
pub struct OpinionsDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    /// Frames after the first (the shot's `ticks / every`).
    pub ticks: u32,
    pub every: u32,
    pub agents: usize,
    pub starts: Vec<f64>,
    pub config: ModelConfig,
    pub frames: Vec<OpinionsFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// Runs an opinions shot, recording every `every`th period.
pub fn run_opinions(shot: &Shot) -> Result<OpinionsDump, Vec<FieldError>> {
    let config = shot.model_config()?;
    let every = shot.every;
    if every == 0 || !shot.ticks.is_multiple_of(every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let mut world = ModelWorld::new(config.clone(), shot.seed)?;
    let ModelWorld::Opinions(first) = &world else {
        unreachable!("an opinions world")
    };
    let starts = first.starts().to_vec();
    let frame = |w: &ModelWorld, tick: u64| -> OpinionsFrame {
        let ModelWorld::Opinions(o) = w else {
            unreachable!("an opinions world")
        };
        OpinionsFrame {
            tick,
            opinions: o.opinions().to_vec(),
        }
    };
    let mut frames = vec![frame(&world, 0)];
    for k in 1..=shot.ticks / every {
        world.model_mut().run(every);
        // A stable world stops; its frames keep counting periods.
        frames.push(frame(&world, u64::from(k * every)));
    }
    let stats = stats_every(world.model(), shot.ticks, every);
    Ok(OpinionsDump {
        format: FORMAT,
        model: "opinions",
        seed: shot.seed,
        ticks: shot.ticks / every,
        every,
        agents: starts.len(),
        starts,
        config,
        frames,
        stats,
    })
}

/// One actor's exact disposition, current state and actual ties.
#[derive(Clone, Debug, Serialize)]
pub struct ThresholdsActor {
    #[serde(flatten)]
    pub state: crate::thresholds::ActorView,
    pub threshold_num: u64,
    pub threshold_den: u64,
    /// One-based actor ids; null means the actor observes its entire crowd.
    pub neighbors: Option<Vec<u64>>,
}

/// One sampled model step, including completed episodes and their final sizes.
#[derive(Clone, Debug, Serialize)]
pub struct ThresholdsFrame {
    pub tick: u64,
    pub step: u32,
    pub episodes: u32,
    pub agents: Vec<ThresholdsActor>,
    /// Completed episode counts by final participation percentage (0 through 100).
    pub sizes: Vec<u32>,
}

/// Recorded real Farol decisions; ticks is the animation frame count.
#[derive(Clone, Debug, Serialize)]
pub struct FarolDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub every: u32,
    pub agents: u32,
    pub game: crate::farol::Game,
    pub config: ModelConfig,
    pub frames: Vec<crate::farol::FarolDecision>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

pub fn run_farol(shot: &Shot) -> Result<FarolDump, Vec<FieldError>> {
    for (bad, field) in [
        (!shot.place.is_empty(), "place"),
        (shot.empty, "empty"),
        (shot.cells.is_some(), "cells"),
        (shot.scores, "scores"),
        (shot.gifts, "gifts"),
    ] {
        if bad {
            return Err(vec![FieldError::new(field, "is not for farol shots")]);
        }
    }
    if shot.every == 0 || !shot.ticks.is_multiple_of(shot.every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let config = shot.model_config()?;
    let ModelConfig::Farol(c) = &config else {
        return Err(vec![FieldError::new("model", "not a farol shot")]);
    };
    if c.evolution.enabled {
        return Err(vec![FieldError::new(
            "evolution.enabled",
            "farol shots require evolution disabled to preserve decision identity",
        )]);
    }
    let mut world = crate::farol::FarolWorld::new(c.clone(), shot.seed)?;
    let mut frames = vec![world.initial_decision()];
    for _ in 0..shot.ticks / shot.every {
        let mut frame = frames.last().unwrap().clone();
        for _ in 0..shot.every {
            if world.is_finished() {
                break;
            }
            frame = world.step_recorded();
        }
        frames.push(frame);
    }
    Ok(FarolDump {
        format: FORMAT,
        model: "farol",
        seed: shot.seed,
        ticks: shot.ticks / shot.every,
        every: shot.every,
        agents: c.agents,
        game: c.game,
        stats: stats_every(&world, shot.ticks, shot.every),
        config,
        frames,
    })
}

/// An actual source-choice state, with transitions since the preceding frame.
#[derive(Clone, Debug, Serialize)]
pub struct AntsFrame {
    pub tick: u64,
    pub agents: Vec<crate::ants::AntView>,
    pub counts: Vec<u32>,
    pub ants_events: Vec<crate::ants::AntsEvent>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AntsDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub every: u32,
    pub agents: u32,
    pub config: ModelConfig,
    /// Actual undirected links, with public IDs; empty on complete graphs.
    pub links: Vec<(u64, u64)>,
    pub frames: Vec<AntsFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

pub fn run_ants(shot: &Shot) -> Result<AntsDump, Vec<FieldError>> {
    for (bad, field) in [
        (!shot.place.is_empty(), "place"),
        (shot.empty, "empty"),
        (shot.cells.is_some(), "cells"),
        (shot.scores, "scores"),
    ] {
        if bad {
            return Err(vec![FieldError::new(field, "is not for ants shots")]);
        }
    }
    if shot.every == 0 || !shot.ticks.is_multiple_of(shot.every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let config = shot.model_config()?;
    let ModelConfig::Ants(c) = &config else {
        return Err(vec![FieldError::new("model", "not an ants shot")]);
    };
    if shot.gifts && c.rule == crate::ants::Rule::Kirman && c.meetings != 1 {
        return Err(vec![FieldError::new(
            "meetings",
            "recorded teaching shots require one meeting per tick",
        )]);
    }
    let mut world = crate::ants::AntsWorld::new(c.clone(), shot.seed)?;
    world.record_events(shot.gifts);
    let links = world
        .graph()
        .edges()
        .iter()
        .map(|&(a, b)| (u64::from(a) + 1, u64::from(b) + 1))
        .collect();
    let frame = |w: &crate::ants::AntsWorld, ants_events| AntsFrame {
        tick: w.tick,
        agents: w.members(),
        counts: w.counts().to_vec(),
        ants_events,
    };
    let mut frames = vec![frame(&world, Vec::new())];
    for _ in 0..shot.ticks / shot.every {
        let mut events = Vec::new();
        for _ in 0..shot.every {
            if world.is_finished() {
                break;
            }
            world.step();
            events.extend_from_slice(world.events());
        }
        frames.push(frame(&world, events));
    }
    Ok(AntsDump {
        format: FORMAT,
        model: "ants",
        seed: shot.seed,
        ticks: shot.ticks / shot.every,
        every: shot.every,
        agents: c.ants,
        links,
        stats: stats_every(&world, shot.ticks, shot.every),
        config,
        frames,
    })
}

#[derive(Clone, Debug, Serialize)]
pub struct ThresholdsDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    /// Frames after the first (the shot's ticks / every).
    pub ticks: u32,
    pub every: u32,
    pub agents: usize,
    pub config: ModelConfig,
    pub frames: Vec<ThresholdsFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// Records threshold worlds without substituting a synthetic animation clock.
pub fn run_thresholds(shot: &Shot) -> Result<ThresholdsDump, Vec<FieldError>> {
    for (bad, field) in [
        (!shot.place.is_empty(), "place"),
        (shot.empty, "empty"),
        (shot.gifts, "gifts"),
        (shot.cells.is_some(), "cells"),
        (shot.scores, "scores"),
    ] {
        if bad {
            return Err(vec![FieldError::new(field, "is not for thresholds shots")]);
        }
    }
    let every = shot.every;
    if every == 0 || !shot.ticks.is_multiple_of(every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let config = shot.model_config()?;
    let ModelConfig::Thresholds(c) = &config else {
        unreachable!("a thresholds shot")
    };
    let mut world = crate::thresholds::ThresholdsWorld::new(c.clone(), shot.seed)?;
    let frame = |w: &crate::thresholds::ThresholdsWorld| {
        let latest = w.stats.latest().expect("initial statistics recorded");
        let agents = (0..w.size())
            .map(|i| {
                let th = w.thresholds()[i];
                ThresholdsActor {
                    state: w.view(i),
                    threshold_num: th.num,
                    threshold_den: th.den,
                    neighbors: if w.watches().is_empty() {
                        None
                    } else {
                        Some(w.watches()[i].iter().map(|&j| u64::from(j) + 1).collect())
                    },
                }
            })
            .collect();
        ThresholdsFrame {
            tick: w.tick,
            step: latest.step,
            episodes: latest.episodes,
            agents,
            sizes: w.sizes().to_vec(),
        }
    };
    let mut frames = vec![frame(&world)];
    for _ in 1..=shot.ticks / every {
        world.run(every);
        frames.push(frame(&world));
    }
    let stats = stats_every(&world, shot.ticks, every);
    Ok(ThresholdsDump {
        format: FORMAT,
        model: "thresholds",
        seed: shot.seed,
        ticks: shot.ticks / every,
        every,
        agents: world.size(),
        config,
        frames,
        stats,
    })
}

/// One recorded period of relative agreement, retaining both state variables.
#[derive(Clone, Debug, Serialize)]
pub struct AgreementFrame {
    pub tick: u64,
    pub opinions: Vec<f64>,
    pub uncertainties: Vec<f64>,
}

/// Relative agreement's initial identities and sampled evolution.
#[derive(Clone, Debug, Serialize)]
pub struct AgreementDump {
    pub format: u32,
    pub model: &'static str,
    pub seed: u64,
    pub ticks: u32,
    pub every: u32,
    pub agents: usize,
    pub starts: Vec<f64>,
    pub roles: Vec<crate::agreement::Role>,
    pub config: ModelConfig,
    pub frames: Vec<AgreementFrame>,
    pub stats: BTreeMap<String, Vec<f64>>,
}

/// Records every requested period, holding the state after stabilization.
pub fn run_agreement(shot: &Shot) -> Result<AgreementDump, Vec<FieldError>> {
    for (bad, field) in [
        (!shot.place.is_empty(), "place"),
        (shot.empty, "empty"),
        (shot.gifts, "gifts"),
        (shot.cells.is_some(), "cells"),
        (shot.scores, "scores"),
    ] {
        if bad {
            return Err(vec![FieldError::new(field, "is not for agreement shots")]);
        }
    }
    let every = shot.every;
    if every == 0 || !shot.ticks.is_multiple_of(every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let config = shot.model_config()?;
    let mut world = ModelWorld::new(config.clone(), shot.seed)?;
    let ModelWorld::Agreement(first) = &world else {
        unreachable!("an agreement world")
    };
    let starts = first.starts().to_vec();
    let roles = first.roles().to_vec();
    let frame = |w: &ModelWorld, tick: u64| -> AgreementFrame {
        let ModelWorld::Agreement(a) = w else {
            unreachable!("an agreement world")
        };
        AgreementFrame {
            tick,
            opinions: a.opinions().to_vec(),
            uncertainties: a.uncertainties().to_vec(),
        }
    };
    let mut frames = vec![frame(&world, 0)];
    for k in 1..=shot.ticks / every {
        world.model_mut().run(every);
        frames.push(frame(&world, u64::from(k * every)));
    }
    let stats = stats_every(world.model(), shot.ticks, every);
    Ok(AgreementDump {
        format: FORMAT,
        model: "agreement",
        seed: shot.seed,
        ticks: shot.ticks / every,
        every,
        agents: starts.len(),
        starts,
        roles,
        config,
        frames,
        stats,
    })
}

/// Runs a culture shot, recording every `every`th step.
pub fn run_culture(shot: &Shot) -> Result<CultureDump, Vec<FieldError>> {
    let config = shot.model_config()?;
    let ModelConfig::Culture(c) = &config else {
        unreachable!("a culture shot")
    };
    let every = shot.every;
    if every == 0 || !shot.ticks.is_multiple_of(every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let mut world = ModelWorld::new(config.clone(), shot.seed)?;
    let (features, sites) = (c.features, c.sites());
    let frame = |w: &ModelWorld| -> CultureFrame {
        let ModelWorld::Culture(cw) = w else {
            unreachable!("a culture world")
        };
        CultureFrame {
            tick: w.model().tick(),
            traits: (0..sites).flat_map(|i| cw.culture(i).to_vec()).collect(),
        }
    };
    let mut frames = vec![frame(&world)];
    for k in 1..=shot.ticks / every {
        world.model_mut().run(every);
        let mut f = frame(&world);
        // A stable world stops; its frames keep counting steps.
        f.tick = u64::from(k * every);
        frames.push(f);
    }
    let stats = stats_every(world.model(), shot.ticks, every);
    // Sites, not the page's grid (which draws lanes between them).
    let (width, height) = (c.width, c.height);
    Ok(CultureDump {
        format: FORMAT,
        model: "culture",
        seed: shot.seed,
        ticks: shot.ticks / every,
        every,
        width,
        height,
        features,
        config,
        frames,
        stats,
    })
}

/// Runs a Schelling shot (his board, or his line laid out a row at a time)
/// and records every round.
pub fn run_schelling(shot: &Shot) -> Result<SchellingDump, Vec<FieldError>> {
    let config = shot.model_config()?;
    let mut world = ModelWorld::new(config.clone(), shot.seed)?;
    let frame = |w: &ModelWorld| -> SchellingFrame {
        let mut agents: Vec<SchellingRow> = match w {
            ModelWorld::Schelling(s) => s
                .agents()
                .map(|a| (a.id, a.pos.x, a.pos.y, a.red, s.is_satisfied(a)))
                .collect(),
            ModelWorld::Line(l) => {
                let wrap = Model::size(l.as_ref()).0;
                l.people()
                    .iter()
                    .enumerate()
                    .map(|(k, p)| {
                        (
                            p.id,
                            k as u32 % wrap,
                            k as u32 / wrap,
                            p.red,
                            l.is_satisfied(k),
                        )
                    })
                    .collect()
            }
            _ => unreachable!("a schelling shot"),
        };
        agents.sort_unstable_by_key(|r| r.0);
        SchellingFrame {
            tick: w.model().tick(),
            agents,
        }
    };
    let every = shot.every;
    if every == 0 || !shot.ticks.is_multiple_of(every) {
        return Err(vec![FieldError::new(
            "every",
            "must be at least 1 and divide ticks",
        )]);
    }
    let mut frames = vec![frame(&world)];
    for _ in 0..shot.ticks / every {
        world.model_mut().run(every);
        frames.push(frame(&world));
    }
    let model = world.model();
    let stats = model
        .series_names()
        .into_iter()
        .filter_map(|name| {
            model
                .series(&name)
                .map(|s| (name, s.into_iter().step_by(every as usize).collect()))
        })
        .collect();
    let (width, height) = model.size();
    Ok(SchellingDump {
        format: FORMAT,
        model: if matches!(config, ModelConfig::Line(_)) {
            "line"
        } else {
            "schelling"
        },
        seed: shot.seed,
        ticks: shot.ticks / every,
        every,
        width,
        height,
        config,
        frames,
        stats,
    })
}

/// Runs a tipping shot and records every step.
pub fn run_tipping(shot: &Shot) -> Result<TippingDump, Vec<FieldError>> {
    let ModelConfig::Tipping(config) = shot.model_config()? else {
        return Err(vec![FieldError::new("model", "not a tipping shot")]);
    };
    let mut w = crate::tipping::TippingWorld::new(config.clone(), shot.seed)?;
    let frame = |w: &crate::tipping::TippingWorld| TippingFrame {
        tick: w.tick,
        red: w.insiders(true),
        blue: w.insiders(false),
    };
    let mut frames = vec![frame(&w)];
    for _ in 0..shot.ticks {
        w.step();
        frames.push(frame(&w));
    }
    let stats = crate::tipping::SERIES
        .iter()
        .filter_map(|&name| w.stats.series(name).map(|s| (name.to_string(), s)))
        .collect();
    Ok(TippingDump {
        format: FORMAT,
        model: "tipping",
        seed: shot.seed,
        ticks: shot.ticks,
        red: config.red,
        blue: config.blue,
        tolerances: [w.tolerances(true).to_vec(), w.tolerances(false).to_vec()],
        config,
        frames,
        stats,
    })
}

pub fn run_dpd(shot: &Shot) -> Result<DpdDump, Vec<FieldError>> {
    let config = shot.dpd_config()?;
    let mut world = DpdWorld::new(config.clone(), shot.seed)?;
    let mut alive = BTreeSet::new();
    let mut frames = vec![dpd_frame(&world, &BTreeSet::new(), &mut alive)];
    for _ in 0..shot.ticks {
        world.step();
        let before = std::mem::take(&mut alive);
        frames.push(dpd_frame(&world, &before, &mut alive));
    }
    let stats = DPD_SERIES
        .iter()
        .filter_map(|&name| world.stats.series(name).map(|s| (name.to_string(), s)))
        .collect();
    Ok(DpdDump {
        format: FORMAT,
        model: "dpd",
        seed: shot.seed,
        ticks: shot.ticks,
        width: config.width,
        height: config.width,
        config,
        frames,
        stats,
    })
}

fn ethno_frame(world: &EthnoWorld) -> EthnoFrame {
    let w = world.config.width;
    EthnoFrame {
        tick: world.tick,
        agents: world
            .occupied()
            .map(|(s, a)| {
                let s = s as u32;
                (a.id, s % w, s / w, a.tag, world.strategy(a), a.lineage)
            })
            .collect(),
    }
}

/// Runs an ethnocentrism shot and records every period.
pub fn run_ethno(shot: &Shot) -> Result<EthnoDump, Vec<FieldError>> {
    let config = shot.ethno_config()?;
    let mut world = EthnoWorld::new(config.clone(), shot.seed)?;
    let mut frames = vec![ethno_frame(&world)];
    for _ in 0..shot.ticks {
        world.step();
        frames.push(ethno_frame(&world));
    }
    let stats = ETHNO_SERIES
        .iter()
        .filter_map(|&name| world.stats.series(name).map(|s| (name.to_string(), s)))
        .collect();
    Ok(EthnoDump {
        format: FORMAT,
        model: "ethno",
        seed: shot.seed,
        ticks: shot.ticks,
        width: config.width,
        height: config.width,
        config,
        frames,
        stats,
    })
}

fn tags_frame(world: &TagsWorld) -> TagsFrame {
    TagsFrame {
        tick: world.tick,
        agents: world
            .agents()
            .iter()
            .map(|a| (a.id, a.parent, a.tag, a.tolerance, a.given, a.received))
            .collect(),
        gifts: world.gifts().map(<[_]>::to_vec).unwrap_or_default(),
    }
}

/// Runs a tags shot and records every generation. Generation 0 is played
/// as the world is made, so its gifts are not recorded.
pub fn run_tags(shot: &Shot) -> Result<TagsDump, Vec<FieldError>> {
    let config = shot.tags_config()?;
    let mut world = TagsWorld::new(config.clone(), shot.seed)?;
    world.record_gifts(shot.gifts);
    let mut frames = vec![tags_frame(&world)];
    for _ in 0..shot.ticks {
        world.step();
        frames.push(tags_frame(&world));
    }
    let stats = TAGS_SERIES
        .iter()
        .filter_map(|&name| world.stats.series(name).map(|s| (name.to_string(), s)))
        .collect();
    Ok(TagsDump {
        format: FORMAT,
        model: "tags",
        seed: shot.seed,
        ticks: shot.ticks,
        config,
        frames,
        stats,
    })
}

fn image_frame(world: &ImageWorld) -> ImageFrame {
    ImageFrame {
        tick: world.tick,
        agents: world
            .agents()
            .iter()
            .map(|a| (a.id, a.strategy.k(), a.score, a.payoff))
            .collect(),
        meetings: world.meetings().map(<[_]>::to_vec).unwrap_or_default(),
    }
}

/// Runs an image-scoring shot and records every generation.
pub fn run_image(shot: &Shot) -> Result<ImageDump, Vec<FieldError>> {
    let config = shot.image_config()?;
    let mut world = ImageWorld::new(config.clone(), shot.seed)?;
    world.record_meetings(shot.gifts);
    let mut frames = vec![image_frame(&world)];
    for _ in 0..shot.ticks {
        world.step();
        frames.push(image_frame(&world));
    }
    let stats = IMAGE_SERIES
        .iter()
        .filter_map(|&name| world.stats.series(name).map(|s| (name.to_string(), s)))
        .collect();
    Ok(ImageDump {
        format: FORMAT,
        model: "image",
        seed: shot.seed,
        ticks: shot.ticks,
        config,
        frames,
        stats,
    })
}

fn norms_frame(world: &NormsWorld) -> NormsFrame {
    let events = world.events().cloned().unwrap_or_default();
    NormsFrame {
        tick: world.tick,
        agents: world
            .played()
            .iter()
            .map(|a| (a.boldness, a.vengefulness, a.payoff, a.parent))
            .collect(),
        cheats: events.cheats,
        punishments: events.punishments,
        metapunishments: events.metapunishments,
    }
}

/// Runs a norms shot and records every `every`th generation (and the first).
pub fn run_norms(shot: &Shot) -> Result<NormsDump, Vec<FieldError>> {
    let config = shot.norms_config()?;
    if shot.every == 0 {
        return Err(vec![FieldError::new("every", "must be at least 1")]);
    }
    let mut world = NormsWorld::new(config.clone(), shot.seed)?;
    let mut frames = vec![norms_frame(&world)];
    let mut kept = vec![0usize];
    for t in 1..=shot.ticks {
        // Only a recorded generation's events are kept, so only it records.
        world.record_events(shot.gifts && t % shot.every == 0);
        world.step();
        if t % shot.every == 0 {
            frames.push(norms_frame(&world));
            kept.push(t as usize);
        }
    }
    let stats = NORMS_SERIES
        .iter()
        .filter_map(|&name| {
            world
                .stats
                .series(name)
                .map(|s| (name.to_string(), kept.iter().map(|&t| s[t]).collect()))
        })
        .collect();
    Ok(NormsDump {
        format: FORMAT,
        model: "norms",
        seed: shot.seed,
        ticks: shot.ticks,
        every: shot.every,
        config,
        frames,
        stats,
    })
}

fn structure_frame(world: &StructureWorld, partners: bool) -> StructureFrame {
    StructureFrame {
        tick: world.tick,
        agents: world
            .played()
            .iter()
            .zip(world.scores())
            .map(|(s, &score)| (s.y, s.p, s.q, score))
            .collect(),
        partners: if partners {
            world
                .met()
                .iter()
                .map(|m| m.iter().map(|&(b, _)| b).collect())
                .collect()
        } else {
            Vec::new()
        },
    }
}

/// Runs a social-structure shot and records every period.
pub fn run_structure(shot: &Shot) -> Result<StructureDump, Vec<FieldError>> {
    let config = shot.structure_config()?;
    let mut world = StructureWorld::new(config.clone(), shot.seed)?;
    let mut frames = vec![structure_frame(&world, false)];
    for _ in 0..shot.ticks {
        world.step();
        frames.push(structure_frame(&world, shot.gifts));
    }
    let stats = STRUCTURE_SERIES
        .iter()
        .filter_map(|&name| world.stats.series(name).map(|s| (name.to_string(), s)))
        .collect();
    Ok(StructureDump {
        format: FORMAT,
        model: "structure",
        seed: shot.seed,
        ticks: shot.ticks,
        site: world.graph().site.clone(),
        config,
        frames,
        stats,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shot(json: &str) -> Result<Shot, Vec<FieldError>> {
        Shot::from_json(json)
    }

    fn fields(errors: Vec<FieldError>) -> Vec<String> {
        errors.into_iter().map(|e| e.field).collect()
    }

    fn run(json: &str) -> FrameDump {
        run_shot(&Shot::from_json(json).unwrap()).unwrap()
    }

    /// An 8×8 world with one hill at (4, 4): an agent on the hill, one on
    /// bare ground that starves in tick 1, and one placed at tick 2.
    const TINY: &str = r#"{"preset": "ii-2-unit", "ticks": 6, "seed": 3,
        "set": {"width": 8, "height": 8, "population": 0, "vision.max": 4,
            "goods.0.map": {"kind": "peaks", "peaks": [{"x": 4, "y": 4, "radius": 4, "height": 4}]}},
        "place": [{"x": 4, "y": 4, "vision": 3, "metabolism": 1, "sugar": 5},
                  {"x": 0, "y": 0, "vision": 1, "metabolism": 4, "sugar": 1},
                  {"tick": 2, "x": 7, "y": 7, "vision": 2}]}"#;

    #[test]
    fn the_same_shot_gives_byte_identical_dumps() {
        let a = serde_json::to_string(&run(TINY)).unwrap();
        let b = serde_json::to_string(&run(TINY)).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn there_is_a_frame_per_tick_and_a_stat_per_frame() {
        let d = run(r#"{"preset": "ii-2-unit", "ticks": 12}"#);
        assert_eq!(d.frames.len(), 13);
        assert!(d.frames.iter().enumerate().all(|(i, f)| f.tick == i as u64));
        assert_eq!(d.stats["population"].len(), 13);
        assert_eq!(d.frames[0].agents.len(), 400);
        assert_eq!(d.frames[0].born.len(), 400);
    }

    #[test]
    fn two_good_frames_carry_spice_and_each_ticks_trades_by_pair() {
        let d = run(r#"{"preset": "iv-3-trade", "ticks": 3, "seed": 1}"#);
        let sites = (d.width * d.height) as usize;
        assert_eq!(d.spice_capacity.len(), sites);
        assert!(d.spice_capacity.iter().any(|&c| c > 0.0));
        for f in &d.frames {
            assert_eq!(f.spice.len(), sites);
            assert_eq!(f.spice_agents.len(), f.agents.len());
            assert!(f.spice_agents.iter().all(|&(held, m)| held >= 0.0 && m > 0));
        }
        assert!(d.frames[0].trades.is_empty());
        let t1 = &d.frames[1].trades;
        assert!(!t1.is_empty(), "no trades in the first tick");
        let mut pairs = BTreeSet::new();
        for &(sugar_giver, spice_giver, sugar, spice, exchanges) in t1 {
            assert_ne!(sugar_giver, spice_giver);
            assert!(sugar > 0.0 && spice > 0.0 && exchanges > 0);
            assert!(
                pairs.insert((sugar_giver, spice_giver)),
                "a pair listed twice"
            );
        }
    }

    #[test]
    fn trades_account_for_every_exchange_and_the_goods_moved() {
        let config = presets::find("iv-3-trade").unwrap().config;
        let ModelConfig::Sugarscape(config) = config else {
            panic!("sugarscape")
        };
        let mut world = World::new(config, 1).unwrap();
        world.step();
        let events = &world.events().trades;
        let merged = trades(events);
        let exchanges: u32 = merged.iter().map(|t| t.4).sum();
        assert_eq!(exchanges as usize, events.len());
        let (mut sugar, mut spice) = (0.0, 0.0);
        for t in events {
            let (got, paid) = (t.amount, t.amount * t.price);
            let (s, p) = if t.goods == (0, 1) {
                (got, paid)
            } else {
                (paid, got)
            };
            sugar += s;
            spice += p;
        }
        let total = |f: fn(&(u64, u64, f64, f64, u32)) -> f64| merged.iter().map(f).sum::<f64>();
        assert!((total(|t| t.2) - sugar).abs() < 1e-9);
        assert!((total(|t| t.3) - spice).abs() < 1e-9);
    }

    #[test]
    fn a_placement_can_set_its_spice() {
        let d = run(
            r#"{"preset": "iv-1-spice", "ticks": 0, "set": {"population": 0},
            "place": [{"x": 1, "y": 1, "sugar": 30, "spice": 3, "spice_metabolism": 2}]}"#,
        );
        assert_eq!(d.frames[0].agents[0].3, 30.0);
        assert_eq!(d.frames[0].spice_agents[0], (3.0, 2));
    }

    #[test]
    fn frames_carry_each_ticks_kills_matching_its_combat_deaths() {
        let d = run(r#"{"preset": "iii-9-combat", "ticks": 5, "seed": 2,
            "set": {"width": 8, "height": 8, "population": 0, "vision.max": 3,
                "placement": {"kind": "random"}, "goods.0.map": {"kind": "flat", "capacity": 1}},
            "place": [{"x": 2, "y": 2, "sugar": 40, "vision": 3, "tribe": "blue"},
                      {"x": 2, "y": 4, "sugar": 2, "vision": 1, "tribe": "red"},
                      {"x": 4, "y": 2, "sugar": 2, "vision": 1, "tribe": "red"}]}"#);
        let mut kills = 0;
        for f in &d.frames {
            let victims: BTreeSet<u64> = f.kills.iter().map(|k| k.1).collect();
            let combat: BTreeSet<u64> = f
                .deaths
                .iter()
                .filter(|(_, c)| *c == "combat")
                .map(|(id, _)| *id)
                .collect();
            assert_eq!(victims, combat);
            for &(attacker, _, loot) in &f.kills {
                assert_eq!(attacker, d.placed[0]);
                assert!(loot > 0.0);
            }
            kills += f.kills.len();
        }
        assert!(kills > 0, "the rich Blue never attacked");
        let json = serde_json::to_value(run(TINY)).unwrap();
        assert!(json["frames"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f.get("kills").is_none()));
    }

    #[test]
    fn credit_frames_carry_outstanding_loans_and_fertile_ages() {
        let d = run(r#"{"preset": "iv-5-credit", "ticks": 60, "seed": 1}"#);
        let fertile: BTreeMap<u64, (u32, u32)> = d
            .frames
            .iter()
            .flat_map(|f| f.fertility.iter().map(|&(id, on, end)| (id, (on, end))))
            .collect();
        for f in &d.frames {
            for &(id, _) in f
                .births
                .iter()
                .map(|b| (b.0, ()))
                .collect::<Vec<_>>()
                .iter()
            {
                assert!(fertile.contains_key(&id), "a newcomer without fertile ages");
            }
            let alive: BTreeSet<u64> = f.agents.iter().map(|a| a.0).collect();
            for &(lender, borrower, due, due_tick) in &f.loans {
                assert_ne!(lender, borrower);
                assert!(alive.contains(&borrower), "a loan to the dead");
                // Settled in the step after the frame whose tick is its due tick.
                assert!(due > 0.0 && due_tick >= f.tick);
            }
        }
        assert!(
            d.frames.iter().any(|f| !f.loans.is_empty()),
            "no loans in 60 ticks"
        );
        let (on, end) = fertile.values().next().unwrap();
        assert!(on < end);
        let json = serde_json::to_value(run(TINY)).unwrap();
        assert!(json["frames"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f.get("loans").is_none() && f.get("fertility").is_none()));
    }

    #[test]
    fn disease_frames_carry_each_agents_diseases_and_the_ticks_infections() {
        let d = run(r#"{"preset": "v-1-rid", "ticks": 10, "seed": 1}"#);
        for f in &d.frames {
            assert_eq!(f.diseases.len(), f.agents.len());
            // An agent infected this tick may still die later in it.
            let seen: BTreeSet<u64> = f
                .agents
                .iter()
                .map(|a| a.0)
                .chain(f.deaths.iter().map(|d| d.0))
                .collect();
            for &(infector, infected, _) in &f.infections {
                assert!(seen.contains(&infected));
                assert_ne!(infector, Some(infected));
            }
        }
        // Endowment skips diseases an immune string already holds, so a few
        // agents start well.
        let sick = d.frames[0].diseases.iter().filter(|&&n| n > 0).count();
        assert!(
            sick * 10 > d.frames[0].diseases.len() * 9,
            "most agents start sick"
        );
        assert!(
            d.frames.iter().any(|f| !f.infections.is_empty()),
            "no infections in 10 ticks"
        );
        let json = serde_json::to_value(run(TINY)).unwrap();
        assert!(json["frames"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f.get("diseases").is_none() && f.get("infections").is_none()));
    }

    #[test]
    fn one_good_dumps_leave_spice_and_trades_out() {
        let d = run(TINY);
        assert!(d.spice_capacity.is_empty());
        assert!(d
            .frames
            .iter()
            .all(|f| f.spice.is_empty() && f.spice_agents.is_empty() && f.trades.is_empty()));
        // Absent, not empty, so one-good dumps keep their bytes.
        let json = serde_json::to_value(&d).unwrap();
        assert!(json.get("spice_capacity").is_none());
        for f in json["frames"].as_array().unwrap() {
            assert!(["spice", "spice_agents", "trades"]
                .iter()
                .all(|k| f.get(k).is_none()));
        }
    }

    #[test]
    fn frames_carry_each_agents_tags_and_group_beside_its_row() {
        let d = run(r#"{"preset": "iii-6-culture", "ticks": 5, "seed": 1}"#);
        for f in &d.frames {
            assert_eq!(f.tags.len(), f.agents.len());
            assert_eq!(f.groups.len(), f.agents.len());
            assert!(f
                .tags
                .iter()
                .all(|t| t.len() == d.config.tag_length as usize));
            // The book's two tribes: Blue (group 0) when zeros outnumber ones.
            for (t, &g) in f.tags.iter().zip(&f.groups) {
                let zeros = t.chars().filter(|&c| c == '0').count();
                assert_eq!(g, if 2 * zeros > t.len() { 0 } else { 1 });
            }
        }
        assert!(
            d.frames[0].tags != d.frames[5].tags,
            "no tag flipped in 5 ticks"
        );
    }

    #[test]
    fn a_placement_can_set_its_tribe() {
        let d = run(
            r#"{"preset": "iii-6-culture", "ticks": 0, "set": {"population": 0},
            "place": [{"x": 1, "y": 1, "tribe": "blue"}, {"x": 2, "y": 1, "tribe": "red"}]}"#,
        );
        assert_eq!(d.frames[0].groups, vec![0, 1]);
    }

    #[test]
    fn a_placement_can_set_its_sex() {
        let d = run(
            r#"{"preset": "iii-2-sex", "ticks": 1, "set": {"population": 0},
            "place": [{"x": 1, "y": 1, "sex": "female"}, {"x": 2, "y": 1, "sex": "male"}]}"#,
        );
        assert_eq!(d.frames[0].births[0].1, Sex::Female);
        assert_eq!(d.frames[0].births[1].1, Sex::Male);
    }

    #[test]
    fn births_record_sex_and_parents_who_were_alive_the_tick_before() {
        let d = run(r#"{"preset": "iii-2-sex", "ticks": 40, "seed": 1}"#);
        // The founders have no parents; every agent first seen is a birth.
        assert_eq!(d.frames[0].births.len(), d.frames[0].born.len());
        assert!(d.frames[0].births.iter().all(|b| b.2.is_none()));
        let children: Vec<_> = d
            .frames
            .iter()
            .flat_map(|f| f.births.iter().map(move |b| (f.tick, b)))
            .collect();
        let born_here: Vec<_> = children
            .iter()
            .filter(|(t, b)| *t > 0 && b.2.is_some())
            .collect();
        assert!(!born_here.is_empty(), "no children in 40 ticks");
        for (tick, (id, _, parents)) in born_here {
            let prev = &d.frames[*tick as usize - 1];
            for p in parents.unwrap() {
                assert!(
                    prev.agents.iter().any(|a| a.0 == p),
                    "{id}'s parent {p} was not alive"
                );
            }
        }
        let founders = &d.frames[0].births;
        assert!(founders.iter().any(|b| b.1 == Sex::Female));
        assert!(founders.iter().any(|b| b.1 == Sex::Male));
    }

    #[test]
    fn frames_carry_each_sites_pollution_once_it_forms() {
        let d = run(r#"{"preset": "ii-8-pollution", "ticks": 60, "seed": 1}"#);
        assert_eq!(d.frames[0].pollution.len(), d.capacity.len());
        assert!(d.frames[50].pollution.iter().all(|&p| p == 0.0));
        assert!(d.frames[60].pollution.iter().any(|&p| p > 0.0));
    }

    #[test]
    fn sites_start_full_or_empty() {
        let d = run(r#"{"preset": "ii-2-unit", "ticks": 0, "set": {"population": 0}}"#);
        assert_eq!(d.frames[0].sugar, d.capacity);
        let d =
            run(r#"{"preset": "ii-2-unit", "ticks": 1, "set": {"population": 0}, "empty": true}"#);
        assert!(d.frames[0].sugar.iter().all(|&s| s == 0.0));
        let grown: Vec<f64> = d.capacity.iter().map(|&c| c.min(1.0)).collect();
        assert_eq!(d.frames[1].sugar, grown);
    }

    #[test]
    fn placed_agents_appear_where_and_when_placed_with_their_traits() {
        let d = run(TINY);
        assert_eq!(d.placed.len(), 3);
        let find = |frame: &Frame, id: u64| frame.agents.iter().find(|a| a.0 == id).copied();
        let a = find(&d.frames[0], d.placed[0]).unwrap();
        assert_eq!((a.1, a.2, a.3, a.5, a.6), (4, 4, 5.0, 3, 1));
        assert!(find(&d.frames[1], d.placed[2]).is_none());
        let c = find(&d.frames[2], d.placed[2]).unwrap();
        assert_eq!((c.1, c.2, c.5), (7, 7, 2));
        assert!(d.frames[2].born.contains(&d.placed[2]));
    }

    #[test]
    fn placed_agent_dying_next_tick_has_a_death_event() {
        let d = run(TINY);
        assert_eq!(d.capacity[0], 0.0);
        let id = d.placed[1];
        assert!(d.frames[0].agents.iter().any(|a| a.0 == id));
        assert!(d.frames[1].agents.iter().all(|a| a.0 != id));
        assert_eq!(d.frames[1].deaths, vec![(id, "starvation")]);
    }

    #[test]
    fn ids_live_contiguously_until_their_death_and_never_return() {
        let d = run(r#"{"preset": "ii-5-wealth", "ticks": 120, "seed": 2}"#);
        let alive = |f: &Frame, id: u64| f.agents.iter().any(|a| a.0 == id);
        let mut dead = std::collections::BTreeSet::new();
        for w in d.frames.windows(2) {
            let (prev, next) = (&w[0], &w[1]);
            for &(id, _) in &next.deaths {
                assert!(alive(prev, id), "{id} died without being alive");
                assert!(!alive(next, id));
                dead.insert(id);
            }
            for a in &prev.agents {
                assert!(alive(next, a.0) || next.deaths.iter().any(|d| d.0 == a.0));
            }
            assert!(next.agents.iter().all(|a| !dead.contains(&a.0)));
            for &id in &next.born {
                assert!(!alive(prev, id));
            }
        }
        assert!(!dead.is_empty());
    }

    #[test]
    fn a_placement_past_the_end_or_on_an_occupied_site_is_an_error() {
        let late = Shot::from_json(
            r#"{"preset": "ii-2-unit", "ticks": 2, "set": {"population": 0},
            "place": [{"tick": 3, "x": 1, "y": 1}]}"#,
        )
        .unwrap();
        assert_eq!(fields(run_shot(&late).unwrap_err()), ["place.0.tick"]);
        let twice = Shot::from_json(
            r#"{"preset": "ii-2-unit", "ticks": 2, "set": {"population": 0},
            "place": [{"x": 1, "y": 1}, {"x": 1, "y": 1}]}"#,
        )
        .unwrap();
        assert_eq!(fields(run_shot(&twice).unwrap_err()), ["place.1"]);
        let off = Shot::from_json(
            r#"{"preset": "ii-2-unit", "ticks": 2, "set": {"population": 0},
            "place": [{"x": 50, "y": 1}]}"#,
        )
        .unwrap();
        assert_eq!(fields(run_shot(&off).unwrap_err()), ["place.0"]);
    }

    #[test]
    fn a_docked_shot_records_each_agents_axelrod_culture() {
        let d = run_shot(&shot(r#"{"preset": "dock-mobility-15", "ticks": 2}"#).unwrap()).unwrap();
        let f = &d.frames[0];
        assert_eq!(f.cultures.len(), f.agents.len());
        assert!(f
            .cultures
            .iter()
            .all(|c| c.len() == 5 && c.iter().all(|&t| t < 15)));
        let plain = run_shot(&shot(r#"{"preset": "ii-2-unit", "ticks": 1}"#).unwrap()).unwrap();
        assert!(
            plain.frames[0].cultures.is_empty(),
            "only under Axelrod's rule"
        );
    }

    #[test]
    fn a_preset_shot_resolves_to_its_config_with_overrides() {
        let s = shot(r#"{"preset": "ii-2-unit", "ticks": 5, "set": {"population": 10}}"#).unwrap();
        let c = s.config().unwrap();
        assert_eq!((c.width, c.height, c.population), (50, 50, 10));
        assert_eq!(s.seed, 1);
        assert!(!s.empty);
    }

    #[test]
    fn a_config_shot_takes_a_full_config() {
        let s = shot(
            r##"{"config": {"width": 12, "height": 12, "population": 0,
            "goods": [{"name": "sugar", "color": "#f2c14e", "map": {"kind": "flat", "capacity": 2},
                "metabolism": {"min": 1, "max": 4}, "endowment": {"min": 5, "max": 25}}]},
            "ticks": 3}"##,
        )
        .unwrap();
        let c = s.config().unwrap();
        assert_eq!((c.width, c.height, c.population), (12, 12, 0));
    }

    #[test]
    fn preset_and_config_are_exclusive() {
        let both = shot(r#"{"preset": "ii-2-unit", "config": {}, "ticks": 1}"#).unwrap();
        assert_eq!(fields(both.config().unwrap_err()), ["shot"]);
        let neither = shot(r#"{"ticks": 1}"#).unwrap();
        assert_eq!(fields(neither.config().unwrap_err()), ["shot"]);
    }

    #[test]
    fn unknown_presets_paths_keys_and_models_are_named() {
        let s = shot(r#"{"preset": "nope", "ticks": 1}"#).unwrap();
        assert_eq!(fields(s.config().unwrap_err()), ["preset"]);
        let s = shot(r#"{"preset": "ii-2-unit", "ticks": 1, "set": {"nope": 1}}"#).unwrap();
        assert_eq!(fields(s.config().unwrap_err()), ["set.nope"]);
        assert_eq!(
            fields(shot(r#"{"preset": "ii-2-unit", "ticks": 1, "tick": 2}"#).unwrap_err()),
            ["shot"]
        );
        let s = shot(r#"{"preset": "vi-4-schelling-25", "ticks": 1}"#).unwrap();
        assert_eq!(fields(s.config().unwrap_err()), ["model"]);
    }

    fn lattice(json: &str) -> LatticeDump {
        match super::run(&Shot::from_json(json).unwrap()).unwrap() {
            Dump::Spatial(d) => d,
            _ => panic!("a spatial shot"),
        }
    }

    #[test]
    fn a_spatial_shot_records_each_generations_strategies_and_the_series() {
        let d = lattice(r#"{"preset": "nm-3-kaleidoscope", "ticks": 3}"#);
        assert_eq!((d.model, d.width, d.height), ("spatial", 99, 99));
        assert_eq!(d.frames.len(), 4);
        assert_eq!(d.stats["fraction_c"].len(), 4);
        let first = &d.frames[0].strategies;
        assert_eq!(first.len(), 99 * 99);
        assert_eq!(first.matches('D').count(), 1);
        assert_eq!(first.find('D'), Some(49 * 99 + 49));
        for (f, share) in d.frames.iter().zip(&d.stats["fraction_c"]) {
            let c = f.strategies.matches('C').count() as f64;
            assert_eq!(c / (99.0 * 99.0), *share);
            assert!(f.scores.is_empty());
        }
    }

    #[test]
    fn a_hand_made_start_replaces_the_configs_and_scores_follow_the_rule() {
        let d = lattice(
            r#"{"preset": "nm-3-kaleidoscope", "ticks": 1, "scores": true,
            "set": {"width": 3, "height": 3},
            "cells": ["CCC", "CDC", "CCC"]}"#,
        );
        assert_eq!(d.frames[0].strategies, "CCCCDCCCC");
        // The cheat plays 8 helpers at b = 1.9 and itself for nothing; a corner
        // helper plays 2 helpers, the cheat, and itself.
        assert!((d.frames[0].scores[4] - 8.0 * 1.9).abs() < 1e-12);
        assert_eq!(d.frames[0].scores[0], 3.0);
        assert_eq!(d.stats["fraction_c"][0], 8.0 / 9.0);
        // The cheat out-earns every neighbor, so all copy it.
        assert_eq!(d.frames[1].strategies, "DDDDDDDDD");
    }

    #[test]
    fn spatial_shots_reject_the_sugarscapes_fields_and_a_bad_board() {
        let err = |json: &str| fields(super::run(&Shot::from_json(json).unwrap()).unwrap_err());
        let k = r#""preset": "nm-3-kaleidoscope", "ticks": 1"#;
        assert_eq!(
            err(&format!(r#"{{{k}, "place": [{{"x": 1, "y": 1}}]}}"#)),
            ["place"]
        );
        assert_eq!(err(&format!(r#"{{{k}, "empty": true}}"#)), ["empty"]);
        assert_eq!(err(&format!(r#"{{{k}, "cells": ["CD"]}}"#)), ["cells"]);
        assert_eq!(
            err(
                r#"{"preset": "nm-3-kaleidoscope", "ticks": 1, "set": {"width": 2, "height": 1}, "cells": ["CX"]}"#
            ),
            ["cells"]
        );
        assert_eq!(err(r#"{"preset": "nbm-cube", "ticks": 1}"#), ["lattice"]);
        assert_eq!(
            err(r#"{"preset": "ii-2-unit", "ticks": 1, "scores": true}"#),
            ["scores"]
        );
        assert_eq!(
            err(r#"{"preset": "vi-8-ring-world", "ticks": 1}"#),
            ["model"]
        );
    }

    fn dpd(json: &str) -> DpdDump {
        match super::run(&Shot::from_json(json).unwrap()).unwrap() {
            Dump::Dpd(d) => d,
            _ => panic!("a demographic-PD shot"),
        }
    }

    #[test]
    fn a_dpd_shot_records_agents_births_and_deaths_that_add_up() {
        let d = dpd(r#"{"preset": "dpd-run-2", "ticks": 40, "seed": 3}"#);
        assert_eq!(
            (d.model, d.width, d.height, d.frames.len()),
            ("dpd", 30, 30, 41)
        );
        assert_eq!(d.frames[0].agents.len(), 100);
        assert_eq!(d.stats["population"].len(), 41);
        let (mut births, mut deaths) = (0, 0);
        let side = |f: &DpdFrame, id: u64| f.agents.iter().find(|a| a.0 == id).map(|a| a.5);
        for k in 1..d.frames.len() {
            let before: BTreeSet<u64> = d.frames[k - 1].agents.iter().map(|a| a.0).collect();
            let after: BTreeSet<u64> = d.frames[k].agents.iter().map(|a| a.0).collect();
            let born: BTreeSet<u64> = d.frames[k].births.iter().map(|b| b.0).collect();
            let died: BTreeSet<u64> = d.frames[k].deaths.iter().map(|x| x.0).collect();
            // Everyone new was born this cycle, and everyone gone died in it.
            assert_eq!(&after - &before, born);
            assert_eq!(&before - &after, died);
            for &(child, parent) in &d.frames[k].births {
                assert!(parent < child);
                // No mutation in Run 2: a clone keeps its parent's strategy.
                let parent_side = side(&d.frames[k], parent).or(side(&d.frames[k - 1], parent));
                assert_eq!(side(&d.frames[k], child), parent_side);
            }
            births += born.len();
            deaths += died.len();
            assert_eq!(d.stats["population"][k], after.len() as f64);
        }
        assert!(births > 0 && deaths > 0);
        let causes: BTreeSet<&str> = d
            .frames
            .iter()
            .flat_map(|f| &f.deaths)
            .map(|x| x.1)
            .collect();
        assert_eq!(causes, BTreeSet::from(["broke", "old_age"]));
    }

    #[test]
    fn an_ethno_shot_records_each_periods_agents_where_they_stay() {
        let d = match super::run(
            &Shot::from_json(r#"{"preset": "ha-standard", "ticks": 60, "seed": 2}"#).unwrap(),
        )
        .unwrap()
        {
            Dump::Ethno(d) => d,
            _ => panic!("an ethnocentrism shot"),
        };
        assert_eq!(
            (d.model, d.width, d.height, d.frames.len()),
            ("ethno", 50, 50, 61)
        );
        assert!(d.frames[0].agents.is_empty());
        let json = serde_json::to_value(&d.frames[60]).unwrap();
        let kinds: BTreeSet<&str> = json["agents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a[4].as_str().unwrap())
            .collect();
        assert!(kinds.is_subset(&BTreeSet::from(["E", "H", "S", "T"])));
        for k in 1..d.frames.len() {
            assert_eq!(d.stats["population"][k], d.frames[k].agents.len() as f64);
            // An agent keeps its square and traits for its whole life.
            let before: BTreeMap<u64, EthnoRow> =
                d.frames[k - 1].agents.iter().map(|a| (a.0, *a)).collect();
            for a in &d.frames[k].agents {
                if let Some(b) = before.get(&a.0) {
                    assert_eq!(a, b);
                }
            }
        }
    }

    #[test]
    fn a_tags_shot_records_each_generation_and_its_gifts() {
        let d = match super::run(
            &Shot::from_json(
                r#"{"preset": "rca-published", "ticks": 30, "seed": 4, "gifts": true}"#,
            )
            .unwrap(),
        )
        .unwrap()
        {
            Dump::Tags(d) => d,
            _ => panic!("a tags shot"),
        };
        assert_eq!((d.model, d.frames.len()), ("tags", 31));
        assert!(d.frames[0].gifts.is_empty());
        for k in 1..d.frames.len() {
            let f = &d.frames[k];
            assert_eq!(f.agents.len(), 100);
            // Each place's agent descends from an agent of the generation before.
            let before: BTreeSet<u64> = d.frames[k - 1].agents.iter().map(|a| a.0).collect();
            assert!(f.agents.iter().all(|a| before.contains(&a.1)));
            // The gifts match the tallies and the donation rate.
            let mut given = vec![0u32; 100];
            let mut received = vec![0u32; 100];
            for &(g, r) in &f.gifts {
                assert_ne!(g, r);
                given[g as usize] += 1;
                received[r as usize] += 1;
            }
            assert!(f.agents.iter().zip(&given).all(|(a, &n)| a.4 == n));
            assert!(f.agents.iter().zip(&received).all(|(a, &n)| a.5 == n));
            assert_eq!(d.stats["donation_rate"][k], f.gifts.len() as f64 / 300.0);
        }
    }

    #[test]
    fn an_image_shot_records_meetings_that_replay_every_score() {
        let d = match super::run(
            &Shot::from_json(r#"{"preset": "ns-fig-1", "ticks": 12, "seed": 3, "gifts": true}"#)
                .unwrap(),
        )
        .unwrap()
        {
            Dump::Image(d) => d,
            _ => panic!("an image-scoring shot"),
        };
        assert_eq!((d.model, d.frames.len()), ("image", 13));
        for (k, f) in d.frames.iter().enumerate().skip(1) {
            assert_eq!(f.agents.len(), 100);
            assert_eq!(f.meetings.len(), 125);
            // Scores start at 0 and move one up for a gift, one down for a
            // refusal, within ±5.
            let mut scores = vec![0i32; 100];
            for &(donor, recipient, helped) in &f.meetings {
                assert_ne!(donor, recipient);
                let s = &mut scores[donor as usize];
                *s = (*s + if helped { 1 } else { -1 }).clamp(-5, 5);
            }
            assert!(f.agents.iter().zip(&scores).all(|(a, &s)| a.2 == s));
            let helps = f.meetings.iter().filter(|m| m.2).count();
            assert_eq!(d.stats["help_rate"][k], helps as f64 / 125.0);
            assert!(f
                .agents
                .iter()
                .all(|a| a.1.is_some_and(|k| (-5..=6).contains(&k))));
        }
    }

    fn norms(json: &str) -> NormsDump {
        match super::run(&Shot::from_json(json).unwrap()).unwrap() {
            Dump::Norms(d) => d,
            _ => panic!("a norms shot"),
        }
    }

    #[test]
    fn a_norms_shot_records_events_that_add_up_to_the_payoffs() {
        let d = norms(r#"{"preset": "ax-metanorms", "ticks": 8, "seed": 2, "gifts": true}"#);
        assert_eq!((d.model, d.frames.len()), ("norms", 9));
        let c = &d.config;
        for f in &d.frames[1..] {
            assert_eq!(f.agents.len(), 20);
            // Each agent's payoff is what the events give it.
            let mut pay = vec![0.0f64; 20];
            for &i in &f.cheats {
                pay[i as usize] += c.temptation;
                for (j, p) in pay.iter_mut().enumerate() {
                    if j != i as usize {
                        *p += c.hurt;
                    }
                }
            }
            for &(j, i) in &f.punishments {
                pay[i as usize] += c.punishment;
                pay[j as usize] += c.enforcement;
            }
            for &(k, j) in &f.metapunishments {
                pay[j as usize] += c.meta_punishment;
                pay[k as usize] += c.meta_enforcement;
            }
            for (a, p) in f.agents.iter().zip(&pay) {
                assert!((a.2 - p).abs() < 1e-9, "{} vs {p}", a.2);
            }
        }
        assert!(d.frames.iter().any(|f| !f.metapunishments.is_empty()));
    }

    #[test]
    fn a_norms_shot_can_keep_every_nth_generation() {
        let d = norms(
            r#"{"preset": "ax-metanorms", "ticks": 1000, "seed": 2, "every": 100, "set": {"stop_at": 0}}"#,
        );
        assert_eq!(
            d.frames.iter().map(|f| f.tick).collect::<Vec<_>>(),
            (0..=10).map(|k| k * 100).collect::<Vec<_>>()
        );
        assert_eq!(d.stats["mean_boldness"].len(), 11);
        let full =
            norms(r#"{"preset": "ax-metanorms", "ticks": 1000, "seed": 2, "set": {"stop_at": 0}}"#);
        assert_eq!(d.frames[10].agents, full.frames[1000].agents);
        assert_eq!(
            d.stats["mean_boldness"][10],
            full.stats["mean_boldness"][1000]
        );
        let err = |json: &str| fields(super::run(&Shot::from_json(json).unwrap()).unwrap_err());
        assert_eq!(
            err(r#"{"preset": "ii-2-unit", "ticks": 1, "every": 2}"#),
            ["every"]
        );
    }

    fn structure(json: &str) -> StructureDump {
        match super::run(&Shot::from_json(json).unwrap()).unwrap() {
            Dump::Structure(d) => d,
            _ => panic!("a social-structure shot"),
        }
    }

    #[test]
    fn a_structure_shot_records_partners_as_its_structure_gives_them() {
        let torus = structure(r#"{"preset": "cra-2dk", "ticks": 3, "seed": 2, "gifts": true}"#);
        assert_eq!(
            (torus.model, torus.frames.len(), torus.site.len()),
            ("structure", 4, 256)
        );
        let at: BTreeMap<u32, usize> = torus
            .site
            .iter()
            .enumerate()
            .map(|(a, &s)| (s, a))
            .collect();
        for f in &torus.frames[1..] {
            assert_eq!(f.agents.len(), 256);
            for (a, partners) in f.partners.iter().enumerate() {
                // On the torus, exactly the four squares next door.
                let s = torus.site[a];
                let (x, y) = (s % 16, s / 16);
                let mut expected: Vec<usize> = [
                    ((x + 1) % 16, y),
                    ((x + 15) % 16, y),
                    (x, (y + 1) % 16),
                    (x, (y + 15) % 16),
                ]
                .iter()
                .map(|&(x, y)| at[&(y * 16 + x)])
                .collect();
                let mut got: Vec<usize> = partners.iter().map(|&b| b as usize).collect();
                expected.sort();
                got.sort();
                assert_eq!(got, expected);
            }
        }
        let fixed = structure(r#"{"preset": "cra-frn", "ticks": 3, "seed": 2, "gifts": true}"#);
        assert!(fixed.site.is_empty());
        assert_eq!(fixed.frames[1].partners, fixed.frames[3].partners);
        let strangers = structure(r#"{"preset": "cra-rwr", "ticks": 3, "seed": 2, "gifts": true}"#);
        assert_ne!(strangers.frames[1].partners, strangers.frames[2].partners);
    }

    #[test]
    fn gifts_are_for_tags_shots_only() {
        let err = |json: &str| fields(super::run(&Shot::from_json(json).unwrap()).unwrap_err());
        assert_eq!(
            err(r#"{"preset": "dpd-run-1", "ticks": 1, "gifts": true}"#),
            ["gifts"]
        );
        assert_eq!(
            err(r#"{"preset": "ii-2-unit", "ticks": 1, "gifts": true}"#),
            ["gifts"]
        );
    }

    #[test]
    fn dpd_shots_reject_other_models_fields() {
        let err = |json: &str| fields(super::run(&Shot::from_json(json).unwrap()).unwrap_err());
        assert_eq!(
            err(r#"{"preset": "dpd-run-1", "ticks": 1, "scores": true}"#),
            ["scores"]
        );
        assert_eq!(
            err(r#"{"preset": "dpd-run-1", "ticks": 1, "cells": ["C"]}"#),
            ["cells"]
        );
        assert_eq!(
            err(r#"{"preset": "dpd-run-1", "ticks": 1, "empty": true}"#),
            ["empty"]
        );
    }

    #[test]
    fn a_random_array_marks_its_empty_squares() {
        let d = lattice(r#"{"preset": "nbm-random-array", "ticks": 1}"#);
        let s = &d.frames[0].strategies;
        assert_eq!(s.len(), 200 * 200);
        assert_eq!(s.matches('.').count(), 200 * 200 - 2000);
    }

    #[test]
    fn an_invalid_result_fails_validation() {
        let s =
            shot(r#"{"preset": "ii-2-unit", "ticks": 1, "set": {"population": 999999}}"#).unwrap();
        assert!(fields(s.config().unwrap_err()).contains(&"population".to_string()));
    }
    fn schelling(json: &str) -> SchellingDump {
        match super::run(&Shot::from_json(json).unwrap()).unwrap() {
            Dump::Schelling(d) => *d,
            _ => panic!("not a schelling dump"),
        }
    }

    #[test]
    fn thresholds_dump_preserves_agents_graph_and_episode_clock() {
        for config in [
            serde_json::json!({"model":"thresholds","actors":20}),
            serde_json::json!({"model":"thresholds","actors":20,"network":"random","degree":3,"trigger":"random","update":"asynchronous","repeat":true}),
            serde_json::json!({"model":"thresholds","actors":20,"friends":{"enabled":true,"symmetric":false,"acquaintance":0.3,"weight":2}}),
        ] {
            let shot = Shot::from_json(
                &serde_json::json!({"config":config,"ticks":30,"every":3,"seed":7}).to_string(),
            )
            .unwrap();
            let raw = serde_json::to_value(super::run(&shot).unwrap()).unwrap();
            assert_eq!(raw["model"], "thresholds");
            assert_eq!(raw["ticks"], 10);
            assert_eq!(raw["every"], 3);
            assert_eq!(
                raw["config"],
                serde_json::to_value(shot.model_config().unwrap()).unwrap()
            );
            let ModelConfig::Thresholds(c) = shot.model_config().unwrap() else {
                panic!("thresholds config")
            };
            let mut world = crate::thresholds::ThresholdsWorld::new(c, shot.seed).unwrap();
            let side = crate::thresholds::grid(world.size() as u32).0;
            let cell = crate::thresholds::grid(world.size() as u32).1;
            for frame in raw["frames"].as_array().unwrap() {
                let state = world.stats.latest().unwrap();
                assert_eq!(frame["tick"], world.tick);
                assert_eq!(frame["step"], state.step);
                assert_eq!(frame["episodes"], state.episodes);
                assert_eq!(frame["sizes"], serde_json::json!(world.sizes().to_vec()));
                assert_eq!(frame["agents"].as_array().unwrap().len(), world.size());
                for (i, agent) in frame["agents"].as_array().unwrap().iter().enumerate() {
                    let view = world
                        .inspect(
                            (crate::thresholds::GRID_X + i % side * cell) as u32,
                            (i / side * cell) as u32,
                        )
                        .unwrap()
                        .member
                        .unwrap();
                    let expected = serde_json::to_value(view).unwrap();
                    for (key, value) in expected.as_object().unwrap() {
                        assert_eq!(&agent[key], value, "{key} for actor {i}");
                    }
                    let th = world.thresholds()[i];
                    assert_eq!(agent["threshold_num"], th.num);
                    assert_eq!(agent["threshold_den"], th.den);
                    let neighbors = if world.watches().is_empty() {
                        serde_json::Value::Null
                    } else {
                        serde_json::json!(world.watches()[i]
                            .iter()
                            .map(|&j| u64::from(j) + 1)
                            .collect::<Vec<_>>())
                    };
                    assert_eq!(agent["neighbors"], neighbors);
                }
                world.run(3);
            }
            for series in raw["stats"].as_object().unwrap().values() {
                assert_eq!(series.as_array().unwrap().len(), 11);
            }
        }
    }

    #[test]
    fn thresholds_dump_holds_the_actual_clock_at_stop_at() {
        let shot = Shot::from_json(
            r#"{"config":{"model":"thresholds","actors":20,"stop_at":5},"ticks":12,"every":3}"#,
        )
        .unwrap();
        let raw = serde_json::to_value(super::run(&shot).unwrap()).unwrap();
        let ticks: Vec<_> = raw["frames"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| f["tick"].as_u64().unwrap())
            .collect();
        assert_eq!(ticks, vec![0, 3, 5, 5, 5]);
        assert_eq!(raw["frames"][2], raw["frames"][4]);
        assert_eq!(
            raw["stats"]["step"],
            serde_json::json!([0.0, 3.0, 5.0, 5.0, 5.0])
        );
    }

    #[test]
    fn thresholds_shot_rejects_unsupported_fields_and_bad_stride() {
        for (field, value) in [
            ("place", serde_json::json!([{"x":0,"y":0}])),
            ("empty", serde_json::json!(true)),
            ("gifts", serde_json::json!(true)),
            ("cells", serde_json::json!(["C"])),
            ("scores", serde_json::json!(true)),
            ("every", serde_json::json!(0)),
            ("every", serde_json::json!(4)),
        ] {
            let mut value_shot =
                serde_json::json!({"config":{"model":"thresholds","actors":20},"ticks":6});
            value_shot[field] = value;
            let shot = Shot::from_json(&value_shot.to_string()).unwrap();
            assert_eq!(super::run(&shot).unwrap_err()[0].field, field);
        }
    }

    #[test]
    fn agreement_dump_preserves_the_world_and_requested_clock() {
        let shot = Shot::from_json(
            r#"{"config":{"model":"agreement","agents":20},"ticks":12,"every":3,"seed":7}"#,
        )
        .unwrap();
        let raw = serde_json::to_value(super::run(&shot).unwrap()).unwrap();
        assert_eq!(raw["model"], "agreement");
        assert_eq!(raw["ticks"], 4);
        assert_eq!(raw["every"], 3);
        let ModelConfig::Agreement(c) = shot.model_config().unwrap() else {
            panic!("agreement config")
        };
        let mut world = crate::agreement::AgreementWorld::new(c, shot.seed).unwrap();
        assert_eq!(raw["starts"], serde_json::json!(world.starts()));
        assert_eq!(raw["roles"], serde_json::json!(world.roles()));
        for (k, frame) in raw["frames"].as_array().unwrap().iter().enumerate() {
            assert_eq!(frame["tick"], k * 3);
            assert_eq!(frame["opinions"], serde_json::json!(world.opinions()));
            assert_eq!(
                frame["uncertainties"],
                serde_json::json!(world.uncertainties())
            );
            world.run(3);
        }
        for series in raw["stats"].as_object().unwrap().values() {
            assert_eq!(series.as_array().unwrap().len(), 5);
        }
    }

    #[test]
    fn agreement_shot_rejects_unsupported_fields_and_bad_stride() {
        for (field, value) in [
            ("place", serde_json::json!([{"x":0,"y":0}])),
            ("empty", serde_json::json!(true)),
            ("gifts", serde_json::json!(true)),
            ("cells", serde_json::json!(["C"])),
            ("scores", serde_json::json!(true)),
            ("every", serde_json::json!(0)),
            ("every", serde_json::json!(4)),
        ] {
            let mut value_shot =
                serde_json::json!({"config":{"model":"agreement","agents":20},"ticks":6});
            value_shot[field] = value;
            let shot = Shot::from_json(&value_shot.to_string()).unwrap();
            assert_eq!(super::run(&shot).unwrap_err()[0].field, field);
        }
        assert!(
            Shot::from_json(r#"{"config":{"model":"agreement"},"ticks":0,"unknown":true}"#)
                .is_err()
        );
    }

    #[test]
    fn stable_agreement_frames_keep_counting_periods() {
        let shot = Shot::from_json(
            r#"{"config":{"model":"agreement","agents":2,"mu":0},"ticks":200,"every":10}"#,
        )
        .unwrap();
        let raw = serde_json::to_value(super::run(&shot).unwrap()).unwrap();
        let frames = raw["frames"].as_array().unwrap();
        assert_eq!(frames.last().unwrap()["tick"], 200);
        assert_eq!(frames[19]["opinions"], frames[20]["opinions"]);
        assert_eq!(frames[19]["uncertainties"], frames[20]["uncertainties"]);
    }

    #[test]
    fn an_opinions_shot_records_every_agents_opinion_each_period() {
        let d = match super::run(
            &Shot::from_json(r#"{"preset": "hk-polarisation", "ticks": 30, "seed": 1}"#).unwrap(),
        )
        .unwrap()
        {
            Dump::Opinions(d) => *d,
            _ => panic!("not an opinions dump"),
        };
        assert_eq!(
            (d.model, d.agents, d.ticks, d.every, d.frames.len()),
            ("opinions", 625, 30, 1, 31)
        );
        assert_eq!(d.starts.len(), 625);
        assert_eq!(d.frames[0].opinions, d.starts, "period 0 is the start");
        let mut w =
            crate::opinions::OpinionsWorld::new(crate::opinions::OpinionsConfig::default(), 1)
                .unwrap();
        w.run(5);
        assert_eq!(d.frames[5].opinions, w.opinions());
        // Stable by period 7 here: the rest hold the last state.
        assert_eq!(d.frames[29].opinions, d.frames[30].opinions);
        assert_eq!(d.stats["clusters"].len(), 31);
    }

    #[test]
    fn a_culture_shot_records_every_sites_traits() {
        let d = match super::run(
            &Shot::from_json(r#"{"preset": "ac-sample-run", "ticks": 20, "seed": 1, "every": 10}"#)
                .unwrap(),
        )
        .unwrap()
        {
            Dump::Culture(d) => *d,
            _ => panic!("not a culture dump"),
        };
        assert_eq!(
            (d.model, d.width, d.height, d.features, d.ticks, d.every),
            ("culture", 10, 10, 5, 2, 10)
        );
        let ticks: Vec<u64> = d.frames.iter().map(|f| f.tick).collect();
        assert_eq!(ticks, [0, 10, 20]);
        let mut w =
            crate::culture::CultureWorld::new(crate::culture::CultureConfig::default(), 1).unwrap();
        for f in &d.frames {
            assert_eq!(f.traits.len(), 100 * 5);
            let expect: Vec<u8> = (0..100).flat_map(|i| w.culture(i).to_vec()).collect();
            assert_eq!(f.traits, expect, "tick {}", f.tick);
            w.run(10);
        }
        assert_eq!(d.stats["regions"].len(), 3);
    }

    #[test]
    fn a_culture_shot_past_stability_holds_its_last_state() {
        // The sample run settles near step 800; the shot keeps counting.
        let d = match super::run(
            &Shot::from_json(
                r#"{"preset": "ac-sample-run", "ticks": 4000, "seed": 1, "every": 100}"#,
            )
            .unwrap(),
        )
        .unwrap()
        {
            Dump::Culture(d) => *d,
            _ => panic!("not a culture dump"),
        };
        assert_eq!(d.frames.len(), 41);
        assert_eq!(d.frames.last().unwrap().tick, 4000);
        assert_eq!(d.frames[39].traits, d.frames[40].traits);
        assert_eq!(d.stats["regions"].len(), 41);
        assert_eq!(d.stats["active_bonds"][40], 0.0, "stable at the end");
    }

    #[test]
    fn a_schelling_shot_records_each_round_its_squares_colors_and_content() {
        let d = schelling(r#"{"preset": "s71-board", "ticks": 3, "seed": 2}"#);
        assert_eq!(
            (d.model, d.width, d.height, d.frames.len()),
            ("schelling", 16, 13, 4)
        );
        let mut w =
            crate::schelling::SchellingWorld::new(crate::schelling::SchellingConfig::default(), 2)
                .unwrap();
        for f in &d.frames {
            assert_eq!(f.agents.len(), 138);
            let mut expect: Vec<SchellingRow> = w
                .agents()
                .map(|a| (a.id, a.pos.x, a.pos.y, a.red, w.is_satisfied(a)))
                .collect();
            expect.sort_unstable_by_key(|r| r.0);
            assert_eq!(f.agents, expect, "tick {}", f.tick);
            w.step();
        }
        let moved = d.frames[0]
            .agents
            .iter()
            .zip(&d.frames[1].agents)
            .filter(|(a, b)| (a.1, a.2) != (b.1, b.2))
            .count();
        assert_eq!(
            moved as f64, d.stats["moves"][1],
            "the frames show the round's moves"
        );
    }

    #[test]
    fn a_schelling_shot_can_record_every_nth_step() {
        // A big board over many steps: frames keep their true step, and the
        // statistics come at the same steps as the frames.
        let d = schelling(r#"{"preset": "s71-board", "ticks": 12, "seed": 2, "every": 4}"#);
        let ticks: Vec<u64> = d.frames.iter().map(|f| f.tick).collect();
        assert_eq!(ticks, [0, 4, 8, 12]);
        assert_eq!((d.ticks, d.every), (3, 4));
        let full = schelling(r#"{"preset": "s71-board", "ticks": 12, "seed": 2}"#);
        assert_eq!(d.frames[2].agents, full.frames[8].agents);
        assert_eq!(d.stats["segregation"].len(), 4);
        assert_eq!(d.stats["segregation"][3], full.stats["segregation"][12]);
    }

    #[test]
    fn a_line_shot_lays_its_row_out_as_squares() {
        let d = schelling(r#"{"preset": "s71-line", "ticks": 2, "seed": 3}"#);
        assert_eq!(
            (d.model, d.width, d.height, d.frames.len()),
            ("line", 70, 1, 3)
        );
        for f in &d.frames {
            let mut places: Vec<u32> = f.agents.iter().map(|r| r.1).collect();
            places.sort_unstable();
            assert_eq!(
                places,
                (0..70).collect::<Vec<_>>(),
                "everyone has a place, no gaps"
            );
            assert!(f.agents.iter().all(|r| r.2 == 0));
        }
        assert!(d.stats.contains_key("groups"));
    }
    #[test]
    fn a_tipping_shot_records_who_is_inside_each_step() {
        let d = match super::run(
            &Shot::from_json(r#"{"preset": "tipping-fig19", "ticks": 40, "seed": 1}"#).unwrap(),
        )
        .unwrap()
        {
            Dump::Tipping(d) => *d,
            _ => panic!("not a tipping dump"),
        };
        assert_eq!((d.red, d.blue, d.frames.len()), (100, 100, 41));
        assert_eq!((d.tolerances[0].len(), d.tolerances[1].len()), (100, 100));
        let first = &d.frames[0];
        assert_eq!(
            (first.red.len(), first.blue.len()),
            (50, 50),
            "the most tolerant 50 of each start inside"
        );
        assert!(first.red.iter().all(|&(rank, _)| rank < 50));
        let last = d.frames.last().unwrap();
        assert_eq!((last.red.len(), last.blue.len()), (80, 80));
        assert!(
            last.red
                .iter()
                .chain(&last.blue)
                .all(|&(_, content)| content),
            "at rest, everyone inside is content"
        );
        assert_eq!(d.stats["red_in"][40], 80.0);
    }
    #[test]
    fn ants_shot_retains_actual_members_counts_and_clock() {
        let shot = Shot::from_json(r#"{"config":{"model":"ants","ants":12,"network":"ring","degree":2,"independent":0.25,"stop_at":5},"seed":9,"ticks":12,"every":3}"#).unwrap();
        let raw = serde_json::to_value(super::run(&shot).unwrap()).unwrap();
        let ModelConfig::Ants(c) = shot.model_config().unwrap() else {
            panic!()
        };
        let mut world = crate::ants::AntsWorld::new(c, shot.seed).unwrap();
        for f in raw["frames"].as_array().unwrap() {
            let tick = f["tick"].as_u64().unwrap();
            world.run((tick - world.tick) as u32);
            assert_eq!(f["counts"], serde_json::json!(world.counts()));
            assert_eq!(f["agents"], serde_json::json!(world.members()));
            for (i, a) in f["agents"].as_array().unwrap().iter().enumerate() {
                assert_eq!(a["id"], i as u64 + 1);
                assert_eq!(a["source"], u32::from(world.sources()[i]) + 1);
                assert_eq!(a["degree"], 2);
                let away = world
                    .graph()
                    .of(i)
                    .iter()
                    .filter(|&&j| world.sources()[j as usize] != world.sources()[i])
                    .count();
                assert_eq!(a["elsewhere"], away);
            }
        }
        assert_eq!(
            raw["frames"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| f["tick"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            vec![0, 3, 5, 5, 5]
        );
    }

    #[test]
    fn ants_shot_rejects_unsupported_fields_and_invalid_stride() {
        for (extra, field) in [
            (r#", "every":0"#, "every"),
            (r#", "every":2"#, "every"),
            (r#", "place":[{"x":0,"y":0}]"#, "place"),
            (r#", "empty":true"#, "empty"),
            (r#", "cells":[]"#, "cells"),
            (r#", "scores":true"#, "scores"),
        ] {
            let json = format!(r#"{{"config":{{"model":"ants"}},"ticks":3{extra}}}"#);
            assert_eq!(
                super::run(&Shot::from_json(&json).unwrap()).unwrap_err()[0].field,
                field
            );
        }
    }

    #[test]
    fn ants_recorded_shots_keep_all_real_events_and_clear_held_frames() {
        let shot=Shot::from_json(r#"{"config":{"model":"ants","ants":12,"sources":3,"meetings":1,"epsilon":1.0,"stop_at":5},"ticks":12,"every":3,"gifts":true}"#).unwrap();
        let Dump::Ants(d) = super::run(&shot).unwrap() else {
            panic!()
        };
        assert!(d.links.is_empty());
        assert!(d.frames[0].ants_events.is_empty());
        assert_eq!(
            d.frames[1]
                .ants_events
                .iter()
                .map(|e| e.tick)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );
        assert_eq!(
            d.frames[2]
                .ants_events
                .iter()
                .map(|e| e.tick)
                .collect::<Vec<_>>(),
            vec![4, 5]
        );
        assert!(d.frames[3].ants_events.is_empty());
        assert!(d.frames[4].ants_events.is_empty());
        let ModelConfig::Ants(c) = shot.model_config().unwrap() else {
            panic!()
        };
        let mut w = crate::ants::AntsWorld::new(c, shot.seed).unwrap();
        for f in &d.frames {
            for e in &f.ants_events {
                let before = w.sources().to_vec();
                w.step();
                let changed: Vec<usize> = before
                    .iter()
                    .zip(w.sources())
                    .enumerate()
                    .filter(|(_, (a, b))| a != b)
                    .map(|(i, _)| i)
                    .collect();
                assert_eq!(changed, vec![(e.agent - 1) as usize]);
                assert_eq!(e.kind, "spontaneous");
                assert_eq!(e.partner, None);
                assert_eq!(e.update, 1);
                assert_eq!(e.from_source, u32::from(before[changed[0]]) + 1);
                assert_eq!(e.to_source, u32::from(w.sources()[changed[0]]) + 1);
            }
        }
    }

    #[test]
    fn ants_shots_keep_actual_network_edges_and_reject_batched_teaching() {
        let shot = Shot::from_json(
            r#"{"config":{"model":"ants","ants":12,"network":"ring","degree":2},"ticks":3}"#,
        )
        .unwrap();
        let Dump::Ants(d) = super::run(&shot).unwrap() else {
            panic!()
        };
        assert_eq!(d.links.len(), 12);
        assert!(d
            .links
            .iter()
            .all(|&(a, b)| a >= 1 && b >= 1 && a <= 12 && b <= 12));
        assert!(d.frames.iter().all(|f| f.ants_events.is_empty()));
        let shot =
            Shot::from_json(r#"{"config":{"model":"ants"},"ticks":3,"gifts":true}"#).unwrap();
        assert_eq!(super::run(&shot).unwrap_err()[0].field, "meetings");
    }
    #[test]
    fn farol_shots_record_actual_decisions_and_native_series() {
        let shot =
            Shot::from_json(r#"{"config":{"model":"farol"},"seed":1010,"ticks":12,"every":3}"#)
                .unwrap();
        let value =
            serde_json::to_value(super::run(&shot).expect("farol recording must be supported"))
                .unwrap();
        assert_eq!(value["model"], "farol");
        let frames = value["frames"].as_array().unwrap();
        assert_eq!(
            frames
                .iter()
                .map(|f| f["tick"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            vec![0, 3, 6, 9, 12]
        );
        assert!(frames[0]["agents"][0]["went"].is_null());
        assert!(frames[0]["agents"][0]["selected"].is_null());
        let ModelConfig::Farol(c) = shot.model_config().unwrap() else {
            panic!()
        };
        let mut native = crate::farol::FarolWorld::new(c, shot.seed).unwrap();
        native.run(12);
        assert_eq!(
            value["stats"],
            serde_json::to_value(stats_every(&native, 12, 3)).unwrap()
        );
        for f in &frames[1..] {
            let members = f["agents"].as_array().unwrap();
            let count = members.iter().filter(|a| a["went"] == true).count();
            assert_eq!(count as u64, f["attendance"].as_u64().unwrap());
            for a in members {
                let k = a["selected"].as_u64().unwrap() as usize;
                let forecast = a["strategies"][k]["forecast"].as_u64().unwrap();
                assert_eq!(a["went"], forecast < 60);
            }
        }
    }

    #[test]
    fn farol_shots_reject_unsupported_fields() {
        for (extra, field) in [
            (r#", "place":[{"x":0,"y":0}]"#, "place"),
            (r#", "every":0"#, "every"),
        ] {
            let shot = Shot::from_json(&format!(
                r#"{{"config":{{"model":"farol"}},"ticks":3{extra}}}"#
            ))
            .unwrap();
            assert_eq!(super::run(&shot).unwrap_err()[0].field, field);
        }
        let shot=Shot::from_json(r#"{"config":{"model":"farol","game":"minority","evolution":{"enabled":true}},"ticks":3}"#).unwrap();
        assert_eq!(super::run(&shot).unwrap_err()[0].field, "evolution.enabled");
    }

    #[test]
    fn farol_minority_initial_frame_has_no_invented_decision() {
        let shot = Shot::from_json(
            r#"{"config":{"model":"farol","game":"minority","agents":101},"ticks":2}"#,
        )
        .unwrap();
        let Dump::Farol(d) = super::run(&shot).unwrap() else {
            panic!()
        };
        assert_eq!(d.frames[0].agents.len(), 101);
        assert!(d.frames[0]
            .agents
            .iter()
            .all(|a| a.went.is_none() && a.selected.is_none()));
        assert!(d.frames[0].history_bits.is_none());
        assert!(d.frames[0].agents[0]
            .strategies
            .iter()
            .all(|s| s.attend.is_none()));
    }
    #[test]
    fn retirement_dump_preserves_every_slot_and_large_newborn_cohort() {
        let shot = Shot::from_json(r#"{"config":{"model":"retirement"},"ticks":1}"#).unwrap();
        let dump = serde_json::to_value(super::run(&shot).unwrap()).unwrap();
        let frames = dump["frames"].as_array().unwrap();
        assert_eq!(frames[0]["agents"].as_array().unwrap().len(), 8100);
        assert_eq!(frames[0]["agents"][0]["id"], 0);
        for frame in frames {
            let members = frame["agents"].as_array().unwrap();
            assert_eq!(members.len(), 8100);
            for (id, a) in members.iter().enumerate() {
                assert_eq!(a["id"], id as u64);
            }
            assert_eq!(
                frame["cohort_counts"]
                    .as_object()
                    .unwrap()
                    .values()
                    .map(|v| v.as_u64().unwrap())
                    .sum::<u64>(),
                8100
            );
        }
        assert_eq!(
            frames[0]["retirements_by_age"].as_array().unwrap().len(),
            81
        );
        assert!(frames[0]["working_exposure_by_age"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v == 0));
        assert_eq!(frames[0]["cohort_counts"]["100"], 100);
        assert!(frames[1]["cohort_counts"]["20"].as_u64().unwrap() > 100);
        assert_eq!(dump["periods"].as_array().unwrap().len(), 2);
        assert!(frames[1]["agents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["born"] == 1 && a["age"] == 20));
    }
    #[test]
    fn retirement_policy_enables_after_warmup_and_switches_after_next_decisions() {
        let shot = Shot::from_json(r#"{"config":{"model":"retirement","per_cohort":1,"rational":1,"random":0,"mandatory":70},"ticks":201,"every":201,"retirement_policy_at":100}"#).unwrap();
        let Dump::Retirement(d) = super::run(&shot).unwrap() else {
            panic!("retirement");
        };
        assert_eq!(d.policy_switched_at, Some(101));
        assert_eq!(d.periods[100].eligibility, 65);
        assert_eq!(
            (
                d.periods[101].decision_eligibility,
                d.periods[101].eligibility
            ),
            (65, 62)
        );
        assert_eq!(d.periods[102].decision_eligibility, 62);
        assert_eq!(d.frames.len(), 2);
        assert_eq!(d.stats["eligibility"].len(), 202);
        assert!(d.periods.iter().all(|p| p.decision.is_none()));
    }

    #[test]
    fn retirement_policy_rejects_invalid_timelines_and_other_models() {
        for json in [
            r#"{"config":{"model":"retirement"},"ticks":10,"retirement_policy_at":0}"#,
            r#"{"config":{"model":"retirement"},"ticks":10,"retirement_policy_at":10}"#,
            r#"{"config":{"model":"retirement","policy":{"enabled":true,"to":62}},"ticks":10,"retirement_policy_at":5}"#,
            r#"{"config":{"model":"opinions"},"ticks":10,"retirement_policy_at":5}"#,
        ] {
            let shot = Shot::from_json(json).unwrap();
            assert!(super::run(&shot).is_err());
        }
    }
    #[test]
    fn retirement_early_stop_preserves_final_actual_population_without_duplicates() {
        for (stop, expected) in [(3, vec![0, 3]), (10, vec![0, 10])] {
            let shot=Shot::from_json(&format!(r#"{{"config":{{"model":"retirement","per_cohort":1,"stop_at":{stop}}},"ticks":10,"every":10}}"#)).unwrap();
            let Dump::Retirement(d) = super::run(&shot).unwrap() else {
                panic!("retirement");
            };
            assert_eq!(
                d.frames.iter().map(|f| f.period.tick).collect::<Vec<_>>(),
                expected
            );
            assert_eq!(d.ticks, 1);
            assert_eq!(d.periods.len(), stop as usize + 1);
            assert_eq!(d.frames.last().unwrap().agents.len(), 81);
            assert_eq!(d.frames.last().unwrap().period.tick, stop as u64);
        }
    }
}
