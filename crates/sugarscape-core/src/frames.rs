//! Frame dumps for the Flump studio (see
//! docs/superpowers/specs/2026-09-25-flump-studio-design.md): a shot — a
//! config, a seed, config overrides and agents placed by hand — run tick by
//! tick, recording every agent, the sugar at every site, deaths, births and
//! the statistics series. Sugarscape only for now.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::{Config, FieldError};
use crate::edit::AgentOverrides;
use crate::model::ModelConfig;
use crate::presets;
use crate::stats;
use crate::world::{DeathCause, World};

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
}

fn first_seed() -> u64 {
    1
}

impl Shot {
    pub fn from_json(json: &str) -> Result<Shot, Vec<FieldError>> {
        serde_json::from_str(json).map_err(|e| vec![FieldError::new("shot", e.to_string())])
    }

    /// The preset or config with `set` applied, validated.
    pub fn config(&self) -> Result<Config, Vec<FieldError>> {
        let base = match (&self.preset, &self.config) {
            (Some(id), None) => presets::find(id).map(|p| p.config).ok_or_else(|| {
                vec![FieldError::new(
                    "preset",
                    format!("unknown preset {id:?} (see `sugarscape presets`)"),
                )]
            })?,
            (None, Some(value)) => ModelConfig::from_value(value.clone()).map_err(|e| vec![e])?,
            _ => {
                return Err(vec![FieldError::new(
                    "shot",
                    "give exactly one of preset or config",
                )])
            }
        };
        let mut config = match base {
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
}

/// `[id, x, y, sugar, age, vision, metabolism]`.
pub type AgentRow = (u64, u32, u32, f64, u32, u32, u32);

/// The world after a tick and that tick's placements.
#[derive(Clone, Debug, Serialize)]
pub struct Frame {
    pub tick: u64,
    pub agents: Vec<AgentRow>,
    /// Good 0's level at every site, row-major.
    pub sugar: Vec<f64>,
    /// Agents that died during this tick, with the cause.
    pub deaths: Vec<(u64, &'static str)>,
    /// Agents first seen in this frame (the initial population, replacements
    /// and placements).
    pub born: Vec<u64>,
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

fn frame(world: &World, seen: &mut BTreeSet<u64>, deaths: Vec<(u64, &'static str)>) -> Frame {
    let agents: Vec<AgentRow> = world
        .agents()
        .map(|a| {
            let (x, y) = (a.pos.x, a.pos.y);
            (a.id, x, y, a.holdings[0], a.age, a.vision, a.metabolism[0])
        })
        .collect();
    let born = agents
        .iter()
        .map(|a| a.0)
        .filter(|&id| seen.insert(id))
        .collect();
    Frame {
        tick: world.tick,
        agents,
        sugar: world.sites.iter().map(|s| s.resource[0]).collect(),
        deaths,
        born,
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
            ..AgentOverrides::default()
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
    let mut seen = BTreeSet::new();
    let mut placed = Vec::new();
    place(&mut world, shot, &mut placed)?;
    let mut frames = vec![frame(&world, &mut seen, Vec::new())];
    for _ in 0..shot.ticks {
        world.step();
        let deaths = world
            .events()
            .deaths
            .iter()
            .map(|d| (d.id, cause_name(d.cause)))
            .collect();
        place(&mut world, shot, &mut placed)?;
        frames.push(frame(&world, &mut seen, deaths));
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
        placed: placed.into_iter().map(|(_, id)| id).collect(),
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

    #[test]
    fn an_invalid_result_fails_validation() {
        let s =
            shot(r#"{"preset": "ii-2-unit", "ticks": 1, "set": {"population": 999999}}"#).unwrap();
        assert!(fields(s.config().unwrap_err()).contains(&"population".to_string()));
    }
}
