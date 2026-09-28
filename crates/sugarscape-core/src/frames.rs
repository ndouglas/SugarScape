//! Frame dumps for the Flump studio (see
//! docs/superpowers/specs/2026-09-25-flump-studio-design.md): a shot — a
//! config, a seed, config overrides and agents placed by hand — run tick by
//! tick, recording every agent, the sugar at every site, deaths, births and
//! the statistics series. Spatial games' shots record each generation's
//! strategies instead, and the demographic Prisoner's Dilemma's each
//! cycle's agents, births and deaths (`run`). Other models are not filmed
//! yet.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agent::{Sex, Tribe};
use crate::config::{Config, FieldError};
use crate::dpd::{DpdConfig, DpdDeath, DpdWorld, SERIES as DPD_SERIES};
use crate::edit::AgentOverrides;
use crate::model::ModelConfig;
use crate::presets;
use crate::spatial::{Lattice, SpatialConfig, SpatialWorld, SERIES as SPATIAL_SERIES};
use crate::stats;
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

    /// A demographic-PD shot's config with `set` applied, validated.
    pub fn dpd_config(&self) -> Result<DpdConfig, Vec<FieldError>> {
        let mut config = self.base()?;
        if !matches!(config, ModelConfig::Dpd(_)) {
            return Err(vec![FieldError::new(
                "model",
                format!("not a demographic-PD shot: {}", config.kind().as_str()),
            )]);
        }
        for (path, value) in &self.set {
            config = config
                .with_path(path, value)
                .map_err(|e| vec![FieldError::new(format!("set.{path}"), e.message)])?;
        }
        config.validate()?;
        let ModelConfig::Dpd(config) = config else {
            unreachable!("set keeps the model")
        };
        Ok(config)
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
    Frame {
        tick: world.tick,
        tags,
        groups,
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

/// A shot's dump, of whichever model it runs.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum Dump {
    Sugarscape(Box<FrameDump>),
    Spatial(LatticeDump),
    Dpd(DpdDump),
}

/// Runs `shot`, whatever its model.
pub fn run(shot: &Shot) -> Result<Dump, Vec<FieldError>> {
    let only = |field: &str, model: &str| {
        vec![FieldError::new(field, format!("is for {model} shots only"))]
    };
    match shot.base()? {
        ModelConfig::Sugarscape(_) => {
            if shot.cells.is_some() {
                return Err(only("cells", "spatial-games"));
            }
            if shot.scores {
                return Err(only("scores", "spatial-games"));
            }
            run_shot(shot).map(|d| Dump::Sugarscape(Box::new(d)))
        }
        ModelConfig::Spatial(_) | ModelConfig::Dpd(_) => {
            if !shot.place.is_empty() {
                return Err(only("place", "sugarscape"));
            }
            if shot.empty {
                return Err(only("empty", "sugarscape"));
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
            run_dpd(shot).map(Dump::Dpd)
        }
        other => Err(vec![FieldError::new(
            "model",
            format!(
                "shots run the sugarscape, spatial games and the demographic PD, not {}",
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

    /// An 8×8 world with one hill at (4, 4): a Flump on the hill, one on
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
            err(r#"{"preset": "vi-4-schelling-25", "ticks": 1}"#),
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
}
