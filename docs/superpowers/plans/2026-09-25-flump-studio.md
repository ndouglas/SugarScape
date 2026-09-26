# Flump Studio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Render the pilot Flump explainer, "Sugarscape", as a 1920 × 1080, 30 fps MP4 built entirely from real engine runs.

**Architecture:** A new `sugarscape shot` CLI command runs a small JSON "shot" (config, seed, overrides, placed agents) and writes a frame dump of every tick. A Python studio (`studio/`) loads dumps in Blender 5.2 and animates everything with one frame-change handler that calls pure functions (`pose(track, timing, frame)`, `levels_at(dump, tick)`, `camera_at(moves, frame)`), so the animation logic is unit-tested outside Blender and motion blur's subframes come for free. Each beat renders to a PNG sequence; ffmpeg cross-dissolves the beats into the final MP4.

**Tech Stack:** Rust (existing workspace, `serde_json`, `clap`); Blender 5.2.2 LTS (`bpy`, Eevee, geometry nodes); Python 3.13 standard library (`unittest`, `dataclasses`, `subprocess`); ffmpeg (`xfade`, `libx264`).

**Spec:** `docs/superpowers/specs/2026-09-25-flump-studio-design.md`

## Global Constraints

- Every Flump behavior on screen is a real engine run; nothing a Flump does is hand-animated. Diagrams (sight lines, meters) are drawn from dump data.
- Every caption that states a result is measured over 20 seeds before it is rendered; if it does not hold, the caption changes, not the data.
- The engine and playground are unchanged: every golden entry, legacy fixture and existing CLI invocation behaves as before. `studio/` is not part of the deployed site.
- A beat is a function of its shot file and its beat entry; rerunning the build gives the same frames.
- Flumps are an original design (amigurumi meets Kirby), not Primer's blobs.
- 16:9, 1920 × 1080, 30 fps; preview 960 × 540.
- Blender 5.2 LTS at `/Applications/Blender.app/Contents/MacOS/Blender`; only Blender's bundled Python modules inside Blender; tests use stdlib `unittest` (pytest is not installed).
- Render engine id is `BLENDER_EEVEE`. `Action.fcurves` does not exist in 5.2 — do not keyframe; drive everything from the frame-change handler.
- ffmpeg has no `drawtext` (no freetype): captions are rendered in Blender, never by ffmpeg.
- CLI errors print `field: message`, one per line, exit code 2; I/O errors exit 1.
- Commits end with the line `Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki`.
- CI runs the latest stable clippy: run `cargo clippy --workspace --all-targets -- -D warnings` before each Rust commit.

## Review Focus

1. **A placed agent that dies or moves on the tick it is placed** — its id must still appear in its placement frame and have a valid track; a track of length 1 must pose without index errors. (Task 2 test `placed_agent_dying_next_tick_has_a_death_event`; Task 5 test `single_frame_track_poses`.)
2. **Torus wrap moves on the 50 × 50 board** — an agent crossing an edge must not glide across the whole board. (Task 5 test `wrap_move_shrinks_instead_of_gliding`.)
3. **Frames outside a beat's tick range** (lead-in before tick 0, holds after the last tick) — must clamp, not index out of range. (Task 5 tests `tick_at_clamps` and `levels_before_start_are_frame_zero`.)
4. **Shot files with mistakes** (unknown key, both `preset` and `config`, a placement tick past `ticks`, an occupied site) — must fail with a named field, not panic or silently ignore. (Task 1 tests.)
5. **The cut's frame count** — dissolves shorten the video; a mismatch between the planned and encoded frame counts must fail the build loudly. (Task 10 test `total_frames_subtracts_dissolves` and the build's ffprobe check.)

---

## File Structure

**Rust**
- Create `crates/sugarscape-core/src/frames.rs` — `Shot`, `Place`, `FrameDump`, `Frame`, `run_shot`.
- Modify `crates/sugarscape-core/src/lib.rs` — `pub mod frames;`.
- Modify `crates/sugarscape-cli/src/main.rs` — `shot` subcommand.
- Modify `README.md` — Command line section: `shot`.

**Studio** (`studio/`)
- `studio/README.md` — how to build an episode.
- `studio/dump.py` — load a frame dump; `Dump`, `Agent`, `Frame`, `Track`, `tracks()`.
- `studio/animate.py` — pure: `Timing`, board geometry (`corner_heights`, `cell_height`, `cell_center`), `levels_at`, `pose`, `blink`, `sight_cells`, `histogram`.
- `studio/camera.py` — pure: `smootherstep`, `Move`, `camera_at`.
- `studio/cut.py` — pure: ffmpeg command for the cut, `total_frames`.
- `studio/episode.py` — pure: `Beat` dataclass, `load_episode`.
- `studio/measure.py` — runs the CLI over 20 seeds and writes a measurements report.
- `studio/build.py` — shots → dumps → renders → cut, with `--preview` and `--beat`.
- `studio/blender/materials.py` — knit, felt, gumdrop, eye, highlight, blush, caption materials; world and lights.
- `studio/blender/board.py` — felt board mesh, gumdrop points + geometry nodes.
- `studio/blender/flump.py` — `build_flump`, crowd prototypes, `crowd_instance`.
- `studio/blender/overlays.py` — caption, belly meter, sight dots, labels, dials, wealth stacks, histogram.
- `studio/blender/scene.py` — assembles a beat and installs the one frame-change handler.
- `studio/render.py` — entry point inside Blender.
- `studio/fonts/Baloo2.ttf`, `studio/fonts/OFL.txt`.
- `studio/tests/test_dump.py`, `test_animate.py`, `test_camera.py`, `test_cut.py`, `test_episode.py`, `test_measure.py`, `studio/tests/fixtures/tiny.frames.json`.
- `studio/episodes/sugarscape/beats.py`, `studio/episodes/sugarscape/shots/*.json`, `studio/episodes/sugarscape/measurements.md`.

Run the Python tests with: `python3 -m unittest discover -s studio/tests -t studio -v`

---

### Task 1: Shot files

**Files:**
- Create: `crates/sugarscape-core/src/frames.rs`
- Modify: `crates/sugarscape-core/src/lib.rs` (add `pub mod frames;` after `pub mod export;`)

**Interfaces:**
- Consumes: `presets::find(&str) -> Option<ModelPreset>` (`.config: ModelConfig`), `ModelConfig::from_value`, `ModelConfig::Sugarscape(Config)`, `Config::with_path(&str, &Value) -> Result<Config, FieldError>`, `Config::validate() -> Result<(), Vec<FieldError>>`, `FieldError::new(field, message)`.
- Produces: `pub struct Shot { preset: Option<String>, config: Option<Value>, seed: u64, ticks: u32, set: serde_json::Map<String, Value>, empty: bool, place: Vec<Place> }`, `pub struct Place { tick: u64, x: u32, y: u32, vision: Option<u32>, metabolism: Option<u32>, sugar: Option<f64> }`, `Shot::from_json(&str) -> Result<Shot, Vec<FieldError>>`, `Shot::config(&self) -> Result<Config, Vec<FieldError>>`.

`set` is applied in sorted key order (the workspace's `serde_json` has no `preserve_order`); since `with_path` does not validate, order does not matter and the result is validated once at the end. `empty: true` sets every site's sugar to 0 after the world is built (sites otherwise start full) — staging the starting level, after which growback runs by its real rule. It is an addition to the spec; record it there in Task 3.

- [ ] **Step 1: Write the failing tests** (at the bottom of `frames.rs`, which starts with the module doc and the types below left empty)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn shot(json: &str) -> Result<Shot, Vec<FieldError>> {
        Shot::from_json(json)
    }

    fn fields(errors: Vec<FieldError>) -> Vec<String> {
        errors.into_iter().map(|e| e.field).collect()
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
        let s = shot(r#"{"config": {"width": 12, "height": 12, "population": 0,
            "goods": [{"name": "sugar", "map": {"kind": "flat", "capacity": 2}}]}, "ticks": 3}"#)
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
        assert_eq!(fields(shot(r#"{"preset": "ii-2-unit", "ticks": 1, "tick": 2}"#).unwrap_err()), ["shot"]);
        let s = shot(r#"{"preset": "vi-4-schelling-25", "ticks": 1}"#).unwrap();
        assert_eq!(fields(s.config().unwrap_err()), ["model"]);
    }

    #[test]
    fn an_invalid_result_fails_validation() {
        let s = shot(r#"{"preset": "ii-2-unit", "ticks": 1, "set": {"population": 999999}}"#).unwrap();
        assert!(fields(s.config().unwrap_err()).contains(&"population".to_string()));
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p sugarscape-core frames::`
Expected: compile errors (`Shot` not defined).

- [ ] **Step 3: Write the implementation** (above the tests)

```rust
//! Frame dumps for the Flump studio (see
//! docs/superpowers/specs/2026-09-25-flump-studio-design.md): a shot — a
//! config, a seed, config overrides and agents placed by hand — run tick by
//! tick, recording every agent, the sugar at every site, deaths, births and
//! the statistics series. Sugarscape only for now.

use serde::Deserialize;
use serde_json::Value;

use crate::config::{Config, FieldError};
use crate::model::ModelConfig;
use crate::presets;

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
                    format!("shots run the sugarscape only, not {}", other.kind().as_str()),
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p sugarscape-core frames::`
Expected: 5 passed. If `a_config_shot_takes_a_full_config` fails because a good needs more fields, copy the missing ones (e.g. `color`, `metabolism`, `endowment`) from `cargo run -q -p sugarscape-cli -- run --preset ii-2-unit --ticks 0 --config-out /dev/stdout` into the test and into every `config` shot in Tasks 4, 8 and 12. If `vi-4-schelling-25` is not a preset id, pick any id from `cargo run -q -p sugarscape-cli -- presets` whose model is not the sugarscape.

- [ ] **Step 5: Lint and commit**

```bash
cargo fmt && cargo clippy --workspace --all-targets -- -D warnings
git add crates/sugarscape-core/src/frames.rs crates/sugarscape-core/src/lib.rs
git commit -m "Add shot files for the Flump studio's frame dumps

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 2: Running a shot into a frame dump

**Files:**
- Modify: `crates/sugarscape-core/src/frames.rs`

**Interfaces:**
- Consumes: Task 1's `Shot`, `Place`; `World::new(Config, u64)`, `World::step()`, `World::tick: u64`, `World::events() -> &TickEvents` (`.deaths: Vec<Death { id, cause: DeathCause, .. }>`), `World::agents()`, `World::sites: Vec<Site>` (`.resource[0]`), `World::capacities(0) -> Vec<f64>`, `World::place_agent(x, y, &AgentOverrides) -> Result<AgentId, String>`, `World::stats.series(&str) -> Option<Vec<f64>>`, `stats::series_names(&Config) -> Vec<String>`.
- Produces: `pub fn run_shot(shot: &Shot) -> Result<FrameDump, Vec<FieldError>>`; `FrameDump` serializes to the format below (`format`, `model`, `seed`, `ticks`, `width`, `height`, `capacity`, `placed`, `config`, `frames`, `stats`); `Frame { tick, agents: Vec<(u64, u32, u32, f64, u32, u32, u32)>, sugar: Vec<f64>, deaths: Vec<(u64, &'static str)>, born: Vec<u64> }`.

Frame `t` is the world after `t` ticks and after that tick's placements. The dump header's `placed` lists the placed agents' ids in the order of `place`, so beats can refer to "placed agent 0". Stats are the engine's per-tick series as recorded (a snapshot per tick, taken before that tick's placements).

- [ ] **Step 1: Write the failing tests** (add to the `tests` module)

```rust
    fn run(json: &str) -> FrameDump {
        run_shot(&Shot::from_json(json).unwrap()).unwrap()
    }

    const TINY: &str = r#"{"config": {"width": 8, "height": 8, "population": 0,
        "goods": [{"name": "sugar", "map": {"kind": "peaks",
            "peaks": [{"x": 4, "y": 4, "radius": 4, "height": 4}]}}]},
        "ticks": 6, "seed": 3,
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
        let d = run(r#"{"preset": "ii-2-unit", "ticks": 1, "set": {"population": 0}, "empty": true}"#);
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
        // Metabolism 4 with 1 sugar on a site of capacity 0: starves in tick 1.
        let d = run(TINY);
        let id = d.placed[1];
        assert!(d.frames[0].agents.iter().any(|a| a.0 == id));
        assert!(d.frames[1].agents.iter().all(|a| a.0 != id));
        assert_eq!(d.frames[1].deaths, vec![(id, "starvation")]);
    }

    #[test]
    fn ids_live_contiguously_until_their_death_and_never_return() {
        let d = run(r#"{"preset": "ii-5-wealth", "ticks": 120, "seed": 2}"#);
        let mut dead = std::collections::BTreeSet::new();
        for w in d.frames.windows(2) {
            let (prev, next) = (&w[0], &w[1]);
            let alive = |f: &Frame, id| f.agents.iter().any(|a| a.0 == id);
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
        let late = Shot::from_json(r#"{"preset": "ii-2-unit", "ticks": 2, "set": {"population": 0},
            "place": [{"tick": 3, "x": 1, "y": 1}]}"#).unwrap();
        assert_eq!(fields(run_shot(&late).unwrap_err()), ["place.0.tick"]);
        let twice = Shot::from_json(r#"{"preset": "ii-2-unit", "ticks": 2, "set": {"population": 0},
            "place": [{"x": 1, "y": 1}, {"x": 1, "y": 1}]}"#).unwrap();
        assert_eq!(fields(run_shot(&twice).unwrap_err()), ["place.1"]);
        let off = Shot::from_json(r#"{"preset": "ii-2-unit", "ticks": 2, "set": {"population": 0},
            "place": [{"x": 50, "y": 1}]}"#).unwrap();
        assert_eq!(fields(run_shot(&off).unwrap_err()), ["place.0"]);
    }
```

- [ ] **Step 2: Run to verify they fail**

Run: `cargo test -p sugarscape-core frames::`
Expected: compile errors (`run_shot`, `FrameDump`, `Frame` not defined).

- [ ] **Step 3: Implement** (add below `impl Shot`, and extend the `use` lines)

```rust
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::edit::AgentOverrides;
use crate::stats;
use crate::world::{DeathCause, World};

/// The dump format's version.
pub const FORMAT: u32 = 1;

/// `[id, x, y, sugar, age, vision, metabolism]`.
pub type AgentRow = (u64, u32, u32, f64, u32, u32, u32);

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
        .map(|a| (a.id, a.pos.x, a.pos.y, a.holdings[0], a.age, a.vision, a.metabolism[0]))
        .collect();
    let born = agents.iter().map(|a| a.0).filter(|&id| seen.insert(id)).collect();
    Frame {
        tick: world.tick,
        agents,
        sugar: world.sites.iter().map(|s| s.resource[0]).collect(),
        deaths,
        born,
    }
}

/// Places the shot's agents for the world's current tick.
fn place(world: &mut World, shot: &Shot, placed: &mut Vec<u64>) -> Result<(), Vec<FieldError>> {
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
        placed.push(id);
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
    // `placed` follows tick order; restore the shot's order.
    let mut order: Vec<usize> = (0..shot.place.len()).collect();
    order.sort_by_key(|&i| shot.place[i].tick);
    let mut by_index = vec![0; placed.len()];
    for (slot, &i) in order.iter().enumerate() {
        by_index[i] = placed[slot];
    }
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
        placed: by_index,
        config,
        frames,
        stats,
    })
}
```

Note: `sort_by_key` is stable, so placements at the same tick keep the order `place` gave them, which is the order they were placed.

- [ ] **Step 4: Run to verify they pass**

Run: `cargo test -p sugarscape-core frames::`
Expected: 12 passed. If `placed_agent_dying_next_tick_has_a_death_event` fails because the agent at (0, 0) found sugar, move it to a site of capacity 0 (the peak's radius 4 at (4, 4) leaves (0, 0) at distance ≈ 5.7, capacity 0 — check `d.capacity[0]` first); do not change the engine.

- [ ] **Step 5: Lint and commit**

```bash
cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/sugarscape-core/src/frames.rs
git commit -m "Run shots into frame dumps: every agent, every site, deaths and births per tick

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 3: The `shot` CLI command

**Files:**
- Modify: `crates/sugarscape-cli/src/main.rs`
- Modify: `README.md` (Command line section)
- Modify: `docs/superpowers/specs/2026-09-25-flump-studio-design.md` (record `empty`, `placed`, and the handler-driven animation and ffmpeg cut decisions)

**Interfaces:**
- Consumes: `sugarscape_core::frames::{Shot, run_shot}`.
- Produces: `sugarscape shot FILE [--out PATH]` — writes the dump JSON (compact, one line) to `PATH` or stdout.

- [ ] **Step 1: Write the failing test** (in `main.rs`'s existing `tests` module)

```rust
    #[test]
    fn shot_takes_a_file_and_an_optional_out() {
        assert!(Cli::try_parse_from(["sugarscape", "shot", "beat.json"]).is_ok());
        assert!(Cli::try_parse_from(["sugarscape", "shot", "beat.json", "--out", "d.json"]).is_ok());
        assert!(Cli::try_parse_from(["sugarscape", "shot"]).is_err());
    }
```

- [ ] **Step 2: Run to verify it fails**

Run: `cargo test -p sugarscape-cli`
Expected: FAIL (`shot` is not a subcommand).

- [ ] **Step 3: Implement**

Add to `enum Command` after `Sweep(SweepArgs)`:

```rust
    /// Run a shot file and write its frame dump (for the Flump studio).
    Shot(ShotArgs),
```

Add after `SweepArgs`:

```rust
#[derive(Debug, Args)]
struct ShotArgs {
    /// A shot JSON file.
    #[arg(value_name = "FILE")]
    file: PathBuf,
    /// Write the frame dump here instead of stdout.
    #[arg(long, value_name = "PATH")]
    out: Option<PathBuf>,
}
```

Add `use sugarscape_core::frames::{self, Shot};` and, in `run`, `Command::Shot(args) => run_shot(args),`, then:

```rust
fn run_shot(args: ShotArgs) -> Result<(), Failure> {
    let shot = Shot::from_json(&read(&args.file)?)?;
    let dump = frames::run_shot(&shot)?;
    let json = serde_json::to_string(&dump).expect("frame dumps serialize");
    match &args.out {
        Some(path) => write(path, &json),
        None => {
            println!("{json}");
            Ok(())
        }
    }
}
```

- [ ] **Step 4: Run to verify, then try it**

Run: `cargo test -p sugarscape-cli`
Expected: PASS.

Run:
```bash
echo '{"preset": "ii-2-unit", "ticks": 3}' > /tmp/s.json
cargo run -q --release -p sugarscape-cli -- shot /tmp/s.json | head -c 200; echo
echo '{"preset": "ii-2-unit", "ticks": 3, "bogus": 1}' > /tmp/s.json
cargo run -q --release -p sugarscape-cli -- shot /tmp/s.json; echo "exit $?"
```
Expected: the dump starts `{"format":1,"model":"sugarscape",...`; the second prints `shot: unknown field ...` and `exit 2`.

- [ ] **Step 5: Document**

In `README.md`'s Command line block add after the `sweep` lines:

```
    sugarscape shot beat.json --out beat.frames.json    # a frame dump for the Flump studio
```

and after the paragraph that begins "`run` runs a preset", add:

```
`shot` runs a shot file — `preset` or `config`, `seed` (default 1), `ticks`, `set` (config
paths to override), `empty` (start every site with no sugar) and `place` (agents placed by hand
at a tick with a given vision, metabolism and sugar) — and writes every tick of the run as JSON:
each agent (`[id, x, y, sugar, age, vision, metabolism]`), the sugar at every site, deaths with
their cause, births, and the statistics series. Frame t is the world after t ticks and after
that tick's placements. Sugarscape only; see `studio/README.md`.
```

In the spec's Part 1: add `empty` (a bool, default false: start every site with no sugar) to the shot file bullets and `placed` (placed agents' ids in `place` order) to the header bullet; in Part 2 replace "Shape keys for squash, stretch, a hungry droop and blinks. Canned actions" with "Squash, stretch, droop and blinks are scales on the Flump's parts, computed per frame. Canned motions", and replace the `cut.py` bullet's "in the video sequencer with captions (fade in/out) and cross-dissolves" with "with ffmpeg's `xfade` cross-dissolves (captions are rendered into each beat in Blender, since the installed ffmpeg has no `drawtext`)". Replace "`pytest` for the pure-Python tests outside it" and the Tests list's "`pytest`" with stdlib `unittest` (pytest is not installed). Add to Part 2's intro: "Animation is not keyframed: one frame-change handler sets every object from pure functions of the frame (Blender 5.2 has no `Action.fcurves`, and the handler also runs at motion blur's subframes)."

- [ ] **Step 6: Lint and commit**

```bash
cargo fmt && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
git add crates/sugarscape-cli/src/main.rs README.md docs/superpowers/specs/2026-09-25-flump-studio-design.md
git commit -m "Add sugarscape shot: frame dumps from the command line

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 4: Studio skeleton and dump loading

**Files:**
- Create: `studio/README.md`, `studio/dump.py`, `studio/tests/__init__.py` (empty), `studio/tests/test_dump.py`, `studio/tests/fixtures/tiny.frames.json`, `studio/fonts/Baloo2.ttf`, `studio/fonts/OFL.txt`

**Interfaces:**
- Consumes: Task 3's dump JSON.
- Produces (in `studio/dump.py`):
  - `Agent(id: int, x: int, y: int, sugar: float, age: int, vision: int, metabolism: int)` (frozen dataclass)
  - `Frame(tick: int, agents: dict[int, Agent], sugar: list[float], deaths: dict[int, str], born: list[int])`
  - `Dump(seed, ticks, width, height, capacity: list[float], placed: list[int], config: dict, frames: list[Frame], stats: dict[str, list[float]])`
  - `Track(id: int, first: int, cells: list[tuple[int, int]], sugar: list[float], death: int | None, cause: str | None)` — `cells[k]` is the agent's cell at tick `first + k`; `death` is the tick whose frame lists its death.
  - `parse(text: str) -> Dump`, `load(path) -> Dump`, `tracks(dump) -> dict[int, Track]`.

- [ ] **Step 1: Make the fixture**

```bash
mkdir -p studio/tests/fixtures
cat > /tmp/tiny-shot.json <<'EOF'
{"config": {"width": 8, "height": 8, "population": 0,
  "goods": [{"name": "sugar", "map": {"kind": "peaks", "peaks": [{"x": 4, "y": 4, "radius": 4, "height": 4}]}}]},
 "ticks": 6, "seed": 3,
 "place": [{"x": 4, "y": 4, "vision": 3, "metabolism": 1, "sugar": 5},
           {"x": 0, "y": 0, "vision": 1, "metabolism": 4, "sugar": 1},
           {"tick": 2, "x": 7, "y": 7, "vision": 2}]}
EOF
cargo run -q --release -p sugarscape-cli -- shot /tmp/tiny-shot.json --out studio/tests/fixtures/tiny.frames.json
```

- [ ] **Step 2: Write the failing tests** (`studio/tests/test_dump.py`)

```python
import pathlib
import unittest

import dump

FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "tiny.frames.json"


class DumpTest(unittest.TestCase):
    def setUp(self):
        self.d = dump.load(FIXTURE)

    def test_header(self):
        self.assertEqual((self.d.width, self.d.height, self.d.ticks), (8, 8, 6))
        self.assertEqual(len(self.d.frames), 7)
        self.assertEqual(len(self.d.capacity), 64)
        self.assertEqual(len(self.d.placed), 3)

    def test_frames_index_agents_by_id(self):
        first = self.d.placed[0]
        a = self.d.frames[0].agents[first]
        self.assertEqual((a.x, a.y, a.vision, a.metabolism, a.sugar), (4, 4, 3, 1, 5.0))

    def test_deaths_are_keyed_by_id(self):
        starving = self.d.placed[1]
        self.assertEqual(self.d.frames[1].deaths, {starving: "starvation"})

    def test_tracks_cover_life_and_record_death(self):
        t = dump.tracks(self.d)
        starving = t[self.d.placed[1]]
        self.assertEqual((starving.first, len(starving.cells), starving.death, starving.cause), (0, 1, 1, "starvation"))
        late = t[self.d.placed[2]]
        self.assertEqual(late.first, 2)
        self.assertEqual(late.cells[0], (7, 7))
        self.assertIsNone(late.death)
        self.assertEqual(len(late.cells), 5)

    def test_wrong_format_is_refused(self):
        with self.assertRaisesRegex(ValueError, "format"):
            dump.parse('{"format": 99}')


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 3: Run to verify they fail**

Run: `python3 -m unittest discover -s studio/tests -t studio -v`
Expected: `ModuleNotFoundError: No module named 'dump'`.

- [ ] **Step 4: Implement** (`studio/dump.py`)

```python
"""Loads a frame dump written by `sugarscape shot` (format 1)."""

import json
from dataclasses import dataclass

FORMAT = 1


@dataclass(frozen=True)
class Agent:
    id: int
    x: int
    y: int
    sugar: float
    age: int
    vision: int
    metabolism: int


@dataclass(frozen=True)
class Frame:
    tick: int
    agents: dict
    sugar: list
    deaths: dict
    born: list


@dataclass(frozen=True)
class Dump:
    seed: int
    ticks: int
    width: int
    height: int
    capacity: list
    placed: list
    config: dict
    frames: list
    stats: dict


@dataclass(frozen=True)
class Track:
    """One agent's life: `cells[k]` and `sugar[k]` at tick `first + k`;
    `death` is the tick whose frame lists its death (None if it survives)."""

    id: int
    first: int
    cells: list
    sugar: list
    death: int | None
    cause: str | None


def parse(text):
    raw = json.loads(text)
    if raw.get("format") != FORMAT:
        raise ValueError(f"frame dump format {raw.get('format')!r}, expected {FORMAT}")
    frames = [
        Frame(
            tick=f["tick"],
            agents={row[0]: Agent(*row) for row in f["agents"]},
            sugar=f["sugar"],
            deaths={id_: cause for id_, cause in f["deaths"]},
            born=f["born"],
        )
        for f in raw["frames"]
    ]
    return Dump(
        seed=raw["seed"],
        ticks=raw["ticks"],
        width=raw["width"],
        height=raw["height"],
        capacity=raw["capacity"],
        placed=raw["placed"],
        config=raw["config"],
        frames=frames,
        stats=raw["stats"],
    )


def load(path):
    with open(path, encoding="utf-8") as f:
        return parse(f.read())


def tracks(d):
    out = {}
    for frame in d.frames:
        for a in frame.agents.values():
            t = out.get(a.id)
            if t is None:
                t = out[a.id] = Track(a.id, frame.tick, [], [], None, None)
            t.cells.append((a.x, a.y))
            t.sugar.append(a.sugar)
        for id_, cause in frame.deaths.items():
            t = out[id_]
            out[id_] = Track(t.id, t.first, t.cells, t.sugar, frame.tick, cause)
    return out
```

- [ ] **Step 5: Run to verify they pass**

Run: `python3 -m unittest discover -s studio/tests -t studio -v`
Expected: 5 passed.

- [ ] **Step 6: Font and README**

```bash
mkdir -p studio/fonts
curl -fL -o studio/fonts/Baloo2.ttf "https://github.com/google/fonts/raw/main/ofl/baloo2/Baloo2%5Bwght%5D.ttf"
curl -fL -o studio/fonts/OFL.txt "https://github.com/google/fonts/raw/main/ofl/baloo2/OFL.txt"
file studio/fonts/Baloo2.ttf   # expect: TrueType Font data
```

`studio/README.md`:

```markdown
# Flump Studio

Short explainer videos rendered in Blender from real engine runs (see
`docs/superpowers/specs/2026-09-25-flump-studio-design.md`).

    python3 studio/build.py sugarscape            # the whole pilot, final quality
    python3 studio/build.py sugarscape --preview  # 960 × 540, low samples
    python3 studio/build.py sugarscape --beat 4   # one beat (1-based)
    python3 studio/measure.py                     # re-measure the pilot's captions over 20 seeds

Needs Blender 5.2 at /Applications/Blender.app (or `BLENDER=/path/to/blender`), ffmpeg, and cargo.
Outputs go to `studio/out/<episode>/` (ignored by git): `dumps/`, `beats/NN/` (PNG frames and the
beat's `.blend`), and `<episode>.mp4`. The `.blend` files open at the frame they were saved on;
the animation is driven by a handler installed at render time, so scrubbing them shows nothing.

Tests: `python3 -m unittest discover -s studio/tests -t studio -v`

Baloo 2 is © The Baloo 2 Project Authors, under the SIL Open Font License (`fonts/OFL.txt`).
```

- [ ] **Step 7: Commit**

```bash
git add studio
git commit -m "Studio: load frame dumps into tracks; Baloo 2 font

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 5: Pure animation planning

**Files:**
- Create: `studio/animate.py`, `studio/tests/test_animate.py`

**Interfaces:**
- Consumes: `dump.Dump`, `dump.Track`, `dump.load`, `dump.tracks`.
- Produces (`studio/animate.py`):
  - `Timing(ticks_per_second: float, start_tick: int = 0, end_tick: int | None = None, lead_in: float = 0.0, fps: int = 30)` with `.frame(tick: float) -> float` (Blender frames start at 1) and `.tick_at(frame: float) -> float` (clamped to `[start_tick, end_tick]`).
  - `HEIGHT_PER_SUGAR = 0.12`; `corner_heights(capacity, w, h) -> list[float]` ((w+1)·(h+1) corners); `cell_height(corners, x, y, w) -> float`; `cell_center(x, y, w, h) -> tuple[float, float]` (cell (0,0) at top-left, +Y north, 1 unit per cell).
  - `levels_at(d: Dump, tick: float) -> list[float]`.
  - `Pose(x, y, z, sx, sy, sz, yaw, visible: bool, sugar: float, hunger: float)`; `pose(track, timing, frame, corners, w, h) -> Pose`.
  - `blink(agent_id: int, frame: float) -> float` (eye z scale, 1 open).
  - `sight_cells(x, y, vision, w, h) -> list[list[tuple[int, int]]]` (four lists: north, east, south, west, nearest first).
  - `histogram(values, bins: int, top: float) -> list[int]`.
  - Constants `SPAWN_FRAMES = 12`, `POOF_FRAMES = 14`.

Motion within one tick interval (fraction `a` of it, `a` in [0, 1)): crouch until 0.15, airborne 0.15–0.85 (arc height `0.35 + 0.08·distance`, stretched), squash on landing 0.85–1. `sy = sx = 1/sqrt(sz)` (volume kept). A move whose cell step is more than half the board in x or y is a torus wrap: the Flump shrinks to 0 by `a = 0.5` at its old cell and grows back at its new cell. An agent is invisible before `frame(first) − SPAWN_FRAMES` and after its poof; it spawns by dropping from 1.5 units with a scale overshoot ending at `frame(first)`; it poofs (scale 1 → 1.15 → 0) over `POOF_FRAMES` starting half way through its death tick's interval. Hunger is `clamp(1 − sugar / 6, 0, 1)` — a display choice for the droop, stated in the module doc. Sugar decreases (eating) snap at `a = 0.85`, the landing; increases (growback) interpolate linearly.

- [ ] **Step 1: Write the failing tests** (`studio/tests/test_animate.py`)

```python
import math
import pathlib
import unittest

import animate
import dump

FIXTURE = pathlib.Path(__file__).parent / "fixtures" / "tiny.frames.json"


def track(first, cells, death=None, sugar=None):
    return dump.Track(1, first, cells, sugar or [10.0] * len(cells), death, "starvation" if death else None)


class TimingTest(unittest.TestCase):
    def test_frame_and_tick_are_inverse(self):
        t = animate.Timing(ticks_per_second=2, lead_in=1.0, end_tick=10)
        self.assertEqual(t.frame(0), 31)
        self.assertEqual(t.frame(1), 46)
        self.assertAlmostEqual(t.tick_at(t.frame(3.5)), 3.5)

    def test_tick_at_clamps(self):
        t = animate.Timing(ticks_per_second=2, start_tick=4, end_tick=6)
        self.assertEqual(t.tick_at(-100), 4)
        self.assertEqual(t.tick_at(10_000), 6)


class BoardTest(unittest.TestCase):
    def test_flat_board_is_flat_and_cells_are_centered(self):
        corners = animate.corner_heights([2.0] * 16, 4, 4)
        self.assertEqual(len(corners), 25)
        self.assertAlmostEqual(animate.cell_height(corners, 1, 2, 4), 2 * animate.HEIGHT_PER_SUGAR)
        self.assertEqual(animate.cell_center(0, 0, 4, 4), (-1.5, 1.5))
        self.assertEqual(animate.cell_center(3, 3, 4, 4), (1.5, -1.5))

    def test_levels_before_start_are_frame_zero(self):
        d = dump.load(FIXTURE)
        self.assertEqual(animate.levels_at(d, -3), d.frames[0].sugar)
        self.assertEqual(animate.levels_at(d, 99), d.frames[-1].sugar)

    def test_eaten_sugar_snaps_at_landing_and_growback_is_linear(self):
        d = dump.load(FIXTURE)
        eaten = [i for i, (a, b) in enumerate(zip(d.frames[0].sugar, d.frames[1].sugar)) if b < a]
        self.assertTrue(eaten)
        i = eaten[0]
        self.assertEqual(animate.levels_at(d, 0.8)[i], d.frames[0].sugar[i])
        self.assertEqual(animate.levels_at(d, 0.9)[i], d.frames[1].sugar[i])


class PoseTest(unittest.TestCase):
    corners = animate.corner_heights([0.0] * 64, 8, 8)
    timing = animate.Timing(ticks_per_second=1, lead_in=1.0)

    def at(self, t, frame):
        return animate.pose(t, self.timing, frame, self.corners, 8, 8)

    def test_still_agent_rests_on_its_cell(self):
        p = self.at(track(0, [(2, 2), (2, 2)]), self.timing.frame(0.5))
        self.assertEqual((p.x, p.y), animate.cell_center(2, 2, 8, 8))
        self.assertAlmostEqual(p.z, 0)
        self.assertTrue(p.visible)

    def test_hop_arcs_between_cells_and_keeps_volume(self):
        t = track(0, [(2, 2), (4, 2)])
        mid = self.at(t, self.timing.frame(0.5))
        self.assertGreater(mid.z, 0.3)
        self.assertAlmostEqual(mid.x, (animate.cell_center(2, 2, 8, 8)[0] + animate.cell_center(4, 2, 8, 8)[0]) / 2, places=1)
        self.assertAlmostEqual(mid.sx * mid.sy * mid.sz, 1, places=6)
        landed = self.at(t, self.timing.frame(1))
        self.assertEqual((landed.x, landed.y), animate.cell_center(4, 2, 8, 8))

    def test_wrap_move_shrinks_instead_of_gliding(self):
        t = track(0, [(0, 3), (7, 3)])
        half = self.at(t, self.timing.frame(0.5))
        self.assertLess(half.sz, 0.05)
        early = self.at(t, self.timing.frame(0.3))
        self.assertEqual(early.x, animate.cell_center(0, 3, 8, 8)[0])
        late = self.at(t, self.timing.frame(0.7))
        self.assertEqual(late.x, animate.cell_center(7, 3, 8, 8)[0])

    def test_spawn_and_poof_bound_visibility(self):
        t = track(2, [(1, 1), (1, 1)], death=4)
        self.assertFalse(self.at(t, self.timing.frame(2) - animate.SPAWN_FRAMES - 1).visible)
        self.assertTrue(self.at(t, self.timing.frame(2)).visible)
        self.assertTrue(self.at(t, self.timing.frame(3.4)).visible)
        self.assertFalse(self.at(t, self.timing.frame(3.5) + animate.POOF_FRAMES + 1).visible)

    def test_single_frame_track_poses(self):
        t = track(0, [(5, 5)], death=1)
        self.assertTrue(self.at(t, self.timing.frame(0.2)).visible)
        self.assertFalse(self.at(t, self.timing.frame(3)).visible)

    def test_hunger_rises_as_sugar_falls(self):
        full = self.at(track(0, [(1, 1)] * 2, sugar=[12.0, 12.0]), self.timing.frame(0))
        empty = self.at(track(0, [(1, 1)] * 2, sugar=[0.5, 0.5]), self.timing.frame(0))
        self.assertEqual(full.hunger, 0)
        self.assertGreater(empty.hunger, 0.9)


class HelpersTest(unittest.TestCase):
    def test_blink_is_mostly_open_and_deterministic(self):
        values = [animate.blink(7, f) for f in range(600)]
        self.assertGreater(sum(v == 1 for v in values), 550)
        self.assertLess(min(values), 0.2)
        self.assertEqual(values, [animate.blink(7, f) for f in range(600)])

    def test_sight_cells_wrap_nearest_first(self):
        n, e, s, w = animate.sight_cells(0, 0, 2, 5, 5)
        self.assertEqual(n, [(0, 4), (0, 3)])
        self.assertEqual(e, [(1, 0), (2, 0)])
        self.assertEqual(s, [(0, 1), (0, 2)])
        self.assertEqual(w, [(4, 0), (3, 0)])

    def test_histogram_bins_and_clamps(self):
        self.assertEqual(animate.histogram([0, 1, 4.9, 5, 9.9, 100], 2, 10), [3, 3])


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run to verify they fail**

Run: `python3 -m unittest discover -s studio/tests -t studio -v`
Expected: `ModuleNotFoundError: No module named 'animate'`.

- [ ] **Step 3: Implement** (`studio/animate.py`)

```python
"""Pure animation planning: everything the Blender handler sets on a frame is
a function of the frame, computed here and tested outside Blender.

Board space: one unit per cell, cell (0, 0) at the top-left (north-west), +X
east, +Y north, the felt's height `HEIGHT_PER_SUGAR` per unit of capacity.
Hunger (the droop) is 1 − sugar / 6, clamped: a display choice, not a rule.
"""

import math
from dataclasses import dataclass

HEIGHT_PER_SUGAR = 0.12
SPAWN_FRAMES = 12
POOF_FRAMES = 14
CROUCH, LAND = 0.15, 0.85


@dataclass(frozen=True)
class Timing:
    ticks_per_second: float
    start_tick: int = 0
    end_tick: int | None = None
    lead_in: float = 0.0
    fps: int = 30

    def frame(self, tick):
        seconds = self.lead_in + (tick - self.start_tick) / self.ticks_per_second
        return 1 + seconds * self.fps

    def tick_at(self, frame):
        seconds = (frame - 1) / self.fps - self.lead_in
        tick = self.start_tick + seconds * self.ticks_per_second
        tick = max(tick, self.start_tick)
        if self.end_tick is not None:
            tick = min(tick, self.end_tick)
        return tick


def corner_heights(capacity, w, h):
    """Each lattice corner's height: the mean capacity of the four cells
    around it (wrapping), so hills are smooth."""
    out = []
    for cy in range(h + 1):
        for cx in range(w + 1):
            cells = [((cx + dx) % w, (cy + dy) % h) for dx in (-1, 0) for dy in (-1, 0)]
            out.append(sum(capacity[y * w + x] for x, y in cells) / 4 * HEIGHT_PER_SUGAR)
    return out


def cell_height(corners, x, y, w):
    row = w + 1
    return (corners[y * row + x] + corners[y * row + x + 1] + corners[(y + 1) * row + x] + corners[(y + 1) * row + x + 1]) / 4


def cell_center(x, y, w, h):
    return (x - w / 2 + 0.5, h / 2 - y - 0.5)


def levels_at(d, tick):
    tick = min(max(tick, 0), d.ticks)
    lo = int(math.floor(tick))
    if lo >= d.ticks:
        return list(d.frames[d.ticks].sugar)
    a = tick - lo
    before, after = d.frames[lo].sugar, d.frames[lo + 1].sugar
    return [
        (b if a >= LAND else s) if b < s else s + (b - s) * a
        for s, b in zip(before, after)
    ]


@dataclass(frozen=True)
class Pose:
    x: float
    y: float
    z: float
    sx: float
    sy: float
    sz: float
    yaw: float
    visible: bool
    sugar: float
    hunger: float


HIDDEN = Pose(0, 0, 0, 0, 0, 0, 0, False, 0, 0)


def smoothstep(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * (3 - 2 * u)


def _squash(sz):
    s = 1 / math.sqrt(sz)
    return s, s, sz


def _ground(corners, cell, w):
    return cell_height(corners, cell[0], cell[1], w)


def _yaw(track, k, w, h):
    """Facing: toward the latest move up to index k (0 = facing the camera, −Y)."""
    for i in range(k, 0, -1):
        (x0, y0), (x1, y1) = track.cells[i - 1], track.cells[i]
        if (x0, y0) != (x1, y1):
            dx, dy = x1 - x0, y1 - y0
            if abs(dx) > w / 2:
                dx = -math.copysign(1, dx)
            if abs(dy) > h / 2:
                dy = -math.copysign(1, dy)
            return math.atan2(dx, dy) + math.pi  # board +Y is north; cell +y is south
    return 0.0


def pose(track, timing, frame, corners, w, h):
    born = timing.frame(track.first)
    if frame < born - SPAWN_FRAMES:
        return HIDDEN
    if track.death is not None:
        poof_start = timing.frame(track.death - 0.5)
        if frame > poof_start + POOF_FRAMES:
            return HIDDEN
    tick = timing.tick_at(frame)
    k = min(max(int(math.floor(tick)) - track.first, 0), len(track.cells) - 1)
    a = min(max(tick - (track.first + k), 0.0), 1.0) if k < len(track.cells) - 1 else 0.0
    here = track.cells[k]
    there = track.cells[k + 1] if k < len(track.cells) - 1 else here
    sugar = track.sugar[k] + (track.sugar[min(k + 1, len(track.sugar) - 1)] - track.sugar[k]) * a
    hunger = min(max(1 - sugar / 6, 0.0), 1.0)
    x0, y0 = cell_center(*here, w, h)
    x1, y1 = cell_center(*there, w, h)
    z0, z1 = _ground(corners, here, w), _ground(corners, there, w)
    sx = sy = sz = 1.0
    x, y, z = x0, y0, z0
    yaw = _yaw(track, k, w, h)
    if here != there:
        wrap = abs(there[0] - here[0]) > w / 2 or abs(there[1] - here[1]) > h / 2
        if wrap:
            s = abs(1 - 2 * a)
            sx = sy = sz = max(s, 0.0)
            if a >= 0.5:
                x, y, z = x1, y1, z1
        elif a < CROUCH:
            sx, sy, sz = _squash(1 - 0.25 * math.sin(math.pi / 2 * a / CROUCH))
        elif a < LAND:
            u = (a - CROUCH) / (LAND - CROUCH)
            p = smoothstep(u)
            dist = math.hypot(x1 - x0, y1 - y0)
            x, y = x0 + (x1 - x0) * p, y0 + (y1 - y0) * p
            z = z0 + (z1 - z0) * p + (0.35 + 0.08 * dist) * 4 * u * (1 - u)
            sx, sy, sz = _squash(1 + 0.2 * math.sin(math.pi * u))
            yaw = _yaw(track, k + 1, w, h)
        else:
            x, y, z = x1, y1, z1
            sx, sy, sz = _squash(1 - 0.2 * math.sin(math.pi * (a - LAND) / (1 - LAND)))
            yaw = _yaw(track, k + 1, w, h)
    if frame < born:
        u = 1 - (born - frame) / SPAWN_FRAMES
        z += 1.5 * (1 - smoothstep(u))
        grow = smoothstep(u) * (1 + 0.15 * math.sin(math.pi * u))
        sx, sy, sz = sx * grow, sy * grow, sz * grow
    if track.death is not None:
        poof_start = timing.frame(track.death - 0.5)
        if frame >= poof_start:
            u = (frame - poof_start) / POOF_FRAMES
            s = (1 + 0.15 * math.sin(math.pi * min(u * 2, 1))) * (1 - smoothstep(u))
            sx, sy, sz = sx * s, sy * s, sz * s
    sz *= 1 - 0.12 * hunger
    return Pose(x, y, z, sx, sy, sz, yaw, True, sugar, hunger)


def blink(agent_id, frame):
    """Eye height: 1 open; a 6-frame blink every 3–6 s, phased by id."""
    period = 90 + (agent_id * 37) % 90
    phase = (frame + agent_id * 53) % period
    if phase < 6:
        return max(0.1, abs(phase - 3) / 3)
    return 1.0


def sight_cells(x, y, vision, w, h):
    steps = [(0, -1), (1, 0), (0, 1), (-1, 0)]  # north, east, south, west
    return [[((x + dx * k) % w, (y + dy * k) % h) for k in range(1, vision + 1)] for dx, dy in steps]


def histogram(values, bins, top):
    counts = [0] * bins
    for v in values:
        i = min(max(int(v / top * bins), 0), bins - 1)
        counts[i] += 1
    return counts
```

- [ ] **Step 4: Run to verify they pass**

Run: `python3 -m unittest discover -s studio/tests -t studio -v`
Expected: all pass. If `test_spawn_and_poof_bound_visibility` fails at `frame(3.4)`, check the poof start (`death − 0.5` ticks = 3.5) — the agent must be visible through 3.4.

- [ ] **Step 5: Commit**

```bash
git add studio/animate.py studio/tests/test_animate.py
git commit -m "Studio: pure per-frame poses, sugar levels, blinks and sight lines

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 6: Camera moves

**Files:**
- Create: `studio/camera.py`, `studio/tests/test_camera.py`

**Interfaces:**
- Produces: `smootherstep(u) -> float`; `Move(start: float, end: float, eye0, target0, eye1, target1, lens0: float = 50, lens1: float = 50, orbit: float = 0.0)` — times in seconds from the beat's start, eyes and targets as `(x, y, z)`; `orbit` is extra radians swung around `target` from start to end; `camera_at(moves: list[Move], seconds: float) -> tuple[eye, target, lens]` — before the first move holds its start, between moves holds the previous end, after the last holds its end.

- [ ] **Step 1: Write the failing tests** (`studio/tests/test_camera.py`)

```python
import math
import unittest

import camera

A = ((0, -10, 5), (0, 0, 0))
B = ((0, -20, 15), (0, 0, 0))


class CameraTest(unittest.TestCase):
    def test_smootherstep_endpoints_and_monotonic(self):
        self.assertEqual((camera.smootherstep(0), camera.smootherstep(1)), (0, 1))
        values = [camera.smootherstep(i / 20) for i in range(21)]
        self.assertEqual(values, sorted(values))

    def test_moves_hold_before_between_and_after(self):
        moves = [camera.Move(1, 3, *A, *B, lens0=35, lens1=50)]
        self.assertEqual(camera.camera_at(moves, 0), (A[0], A[1], 35))
        self.assertEqual(camera.camera_at(moves, 5), (B[0], B[1], 50))
        eye, _, lens = camera.camera_at(moves, 2)
        self.assertAlmostEqual(eye[1], -15)
        self.assertAlmostEqual(lens, 42.5)

    def test_orbit_swings_around_the_target(self):
        moves = [camera.Move(0, 1, *A, *A, orbit=math.pi / 2)]
        eye, target, _ = camera.camera_at(moves, 1)
        self.assertAlmostEqual(eye[0], 10, places=6)
        self.assertAlmostEqual(eye[1], 0, places=6)
        self.assertEqual(eye[2], 5)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run to verify they fail** — `python3 -m unittest discover -s studio/tests -t studio -v`, expected `No module named 'camera'`.

- [ ] **Step 3: Implement** (`studio/camera.py`)

```python
"""Camera moves as pure functions of time: a beat's camera is a list of moves,
each easing eye, target and lens from a start pose to an end pose, with an
optional orbit around the target."""

import math
from dataclasses import dataclass


def smootherstep(u):
    u = min(max(u, 0.0), 1.0)
    return u * u * u * (u * (u * 6 - 15) + 10)


@dataclass(frozen=True)
class Move:
    start: float
    end: float
    eye0: tuple
    target0: tuple
    eye1: tuple
    target1: tuple
    lens0: float = 50.0
    lens1: float = 50.0
    orbit: float = 0.0


def _lerp(a, b, t):
    return tuple(x + (y - x) * t for x, y in zip(a, b))


def _at(m, u):
    e = smootherstep(u)
    eye, target = _lerp(m.eye0, m.eye1, e), _lerp(m.target0, m.target1, e)
    if m.orbit:
        angle = m.orbit * e
        dx, dy = eye[0] - target[0], eye[1] - target[1]
        c, s = math.cos(angle), math.sin(angle)
        eye = (target[0] + dx * c - dy * s, target[1] + dx * s + dy * c, eye[2])
    return eye, target, m.lens0 + (m.lens1 - m.lens0) * e


def camera_at(moves, seconds):
    current = moves[0]
    if seconds <= current.start:
        return _at(current, 0.0)
    for m in moves:
        if seconds < m.start:
            return _at(current, 1.0)
        current = m
        if seconds <= m.end:
            return _at(m, (seconds - m.start) / max(m.end - m.start, 1e-9))
    return _at(current, 1.0)
```

- [ ] **Step 4: Run to verify they pass** — expected: all pass. (Orbit check: eye starts at (0, −10), target (0, 0); +90° about +Z gives (10, 0).)

- [ ] **Step 5: Commit** — `git add studio/camera.py studio/tests/test_camera.py && git commit -m "Studio: eased camera moves with orbits" -m "Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"`

---

### Task 7: Episodes, beats and the Blender board

**Files:**
- Create: `studio/episode.py`, `studio/tests/test_episode.py`, `studio/blender/__init__.py` (empty), `studio/blender/materials.py`, `studio/blender/board.py`, `studio/blender/scene.py`, `studio/render.py`, `studio/episodes/sugarscape/beats.py` (beat 2 only for now), `studio/episodes/sugarscape/shots/landscape.json`

**Interfaces:**
- Consumes: `animate.Timing`, `animate.corner_heights`, `animate.cell_center`, `animate.cell_height`, `animate.levels_at`, `camera.Move`, `camera.camera_at`, `dump.load`.
- Produces:
  - `episode.Beat(name: str, caption: str, seconds: float, shot: str | None = None, ticks_per_second: float = 4.0, start_tick: int = 0, lead_in: float = 0.0, camera: tuple = (), closeup: bool = False, overlays: tuple = (), focus: tuple = ())` — `focus` holds indexes into the dump's `placed`; `overlays` names from Task 9.
  - `episode.load_episode(name) -> list[Beat]` (imports `studio/episodes/<name>/beats.py`'s `BEATS`).
  - `episode.Beat.timing(dump_ticks: int) -> animate.Timing`.
  - `episode.Beat.frames -> int` (`round(seconds × 30)`).
  - `blender.scene.build_beat(beat, dump, preview: bool) -> list[callable]` — builds the scene and returns the per-frame updaters; `blender.scene.install(updaters)` registers one `frame_change_pre` handler calling each `updater(frame: float)`.
  - `render.py` CLI: `blender -b --factory-startup -P studio/render.py -- EPISODE INDEX [--preview] [--still FRAME]` (INDEX 1-based). Writes `studio/out/<ep>/beats/NN/####.png` (or `studio/out/<ep>/preview/NN/` with `--preview`) and `beat.blend` beside them; `--still F` renders only frame F to `still.png`.

- [ ] **Step 1: Write the failing test** (`studio/tests/test_episode.py`)

```python
import unittest

import episode


class EpisodeTest(unittest.TestCase):
    def test_the_pilot_loads_and_its_beats_are_sane(self):
        beats = episode.load_episode("sugarscape")
        self.assertTrue(beats)
        for b in beats:
            self.assertGreater(b.seconds, 0)
            self.assertTrue(b.camera, b.name)
            self.assertEqual(b.frames, round(b.seconds * 30))

    def test_timing_ends_at_the_dumps_last_tick(self):
        b = episode.Beat("x", "", 4, shot="s", ticks_per_second=2, lead_in=1)
        t = b.timing(6)
        self.assertEqual((t.end_tick, t.lead_in, t.ticks_per_second), (6, 1, 2))


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run to verify it fails** — expected `No module named 'episode'`.

- [ ] **Step 3: Implement `studio/episode.py`**

```python
"""An episode is an ordered list of beats, each rendered from one shot."""

import importlib.util
import pathlib
from dataclasses import dataclass

import animate

ROOT = pathlib.Path(__file__).resolve().parent
FPS = 30


@dataclass(frozen=True)
class Beat:
    name: str
    caption: str
    seconds: float
    shot: str | None = None
    ticks_per_second: float = 4.0
    start_tick: int = 0
    lead_in: float = 0.0
    camera: tuple = ()
    closeup: bool = False
    overlays: tuple = ()
    focus: tuple = ()

    @property
    def frames(self):
        return round(self.seconds * FPS)

    def timing(self, dump_ticks):
        return animate.Timing(self.ticks_per_second, self.start_tick, dump_ticks, self.lead_in, FPS)


def episode_dir(name):
    return ROOT / "episodes" / name


def load_episode(name):
    path = episode_dir(name) / "beats.py"
    spec = importlib.util.spec_from_file_location(f"episodes.{name}.beats", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return list(module.BEATS)
```

- [ ] **Step 4: The first shot and beat**

`studio/episodes/sugarscape/shots/landscape.json`:

```json
{"preset": "ii-2-unit", "ticks": 0, "set": {"population": 0}}
```

`studio/episodes/sugarscape/beats.py`:

```python
"""The pilot, "Sugarscape" (spec Part 3). Board coordinates: one unit per
cell, the 50 × 50 board centered on the origin; the book's hills sit around
cells (35, 15) and (15, 35), i.e. board (10.5, 9.5) and (−9.5, −10.5)."""

from camera import Move
from episode import Beat

WIDE_EYE, WIDE_AT = (0, -52, 40), (0, -2, 0)

BEATS = [
    Beat(
        "landscape",
        "Sugar grows in some places… but not others.",
        6.0,
        shot="landscape",
        camera=(Move(0, 6, (10, -4, 4), (10.5, 9.5, 0.4), WIDE_EYE, WIDE_AT, lens0=35, lens1=40),),
    ),
]
```

- [ ] **Step 5: Run the test to verify it passes** — `python3 -m unittest discover -s studio/tests -t studio -v`. Expected: all pass.

- [ ] **Step 6: Implement `studio/blender/materials.py`**

```python
"""The handmade look: knit yarn, felt, gumdrops, glossy eyes; warm tabletop light."""

import bpy

YARN = {
    "cream": (0.93, 0.86, 0.72),
    "coral": (0.95, 0.45, 0.38),
    "teal": (0.25, 0.62, 0.62),
    "lilac": (0.66, 0.55, 0.85),
    "butter": (0.98, 0.82, 0.40),
}


def _principled(name):
    m = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    return m, nt, nt.nodes["Principled BSDF"]


def _bump(nt, p, scale, strength, wave=True):
    coord = nt.nodes.new("ShaderNodeTexCoord")
    noise = nt.nodes.new("ShaderNodeTexNoise")
    noise.inputs["Scale"].default_value = scale * 3
    nt.links.new(coord.outputs["Object"], noise.inputs["Vector"])
    height = noise.outputs["Fac"]
    if wave:
        w = nt.nodes.new("ShaderNodeTexWave")
        w.wave_type = "BANDS"
        w.bands_direction = "Z"
        w.inputs["Scale"].default_value = scale
        w.inputs["Distortion"].default_value = 3.0
        nt.links.new(coord.outputs["Object"], w.inputs["Vector"])
        add = nt.nodes.new("ShaderNodeMath")
        add.operation = "ADD"
        nt.links.new(w.outputs["Fac"], add.inputs[0])
        nt.links.new(noise.outputs["Fac"], add.inputs[1])
        height = add.outputs["Value"]
    bump = nt.nodes.new("ShaderNodeBump")
    bump.inputs["Strength"].default_value = strength
    nt.links.new(height, bump.inputs["Height"])
    nt.links.new(bump.outputs["Normal"], p.inputs["Normal"])


def knit(color_name):
    m, nt, p = _principled(f"knit-{color_name}")
    if len(nt.nodes) > 2:
        return m
    p.inputs["Base Color"].default_value = (*YARN[color_name], 1)
    p.inputs["Roughness"].default_value = 0.85
    p.inputs["Sheen Weight"].default_value = 0.6
    p.inputs["Sheen Roughness"].default_value = 0.35
    p.inputs["Subsurface Weight"].default_value = 0.12
    _bump(nt, p, scale=28, strength=0.35)
    return m


def felt():
    m, nt, p = _principled("felt")
    if len(nt.nodes) > 2:
        return m
    p.inputs["Base Color"].default_value = (0.42, 0.55, 0.36, 1)
    p.inputs["Roughness"].default_value = 1.0
    p.inputs["Sheen Weight"].default_value = 0.8
    _bump(nt, p, scale=6, strength=0.25, wave=False)
    return m


def gumdrop():
    m, nt, p = _principled("gumdrop")
    p.inputs["Base Color"].default_value = (1.0, 0.72, 0.18, 1)
    p.inputs["Roughness"].default_value = 0.18
    p.inputs["Subsurface Weight"].default_value = 0.5
    p.inputs["Subsurface Radius"].default_value = (1.0, 0.6, 0.2)
    p.inputs["Coat Weight"].default_value = 1.0
    return m


def gloss(name, color, emission=0.0):
    m, nt, p = _principled(name)
    p.inputs["Base Color"].default_value = (*color, 1)
    p.inputs["Roughness"].default_value = 0.05
    p.inputs["Coat Weight"].default_value = 1.0
    if emission:
        p.inputs["Emission Color"].default_value = (*color, 1)
        p.inputs["Emission Strength"].default_value = emission
    return m


def matte(name, color):
    m, nt, p = _principled(name)
    p.inputs["Base Color"].default_value = (*color, 1)
    p.inputs["Roughness"].default_value = 1.0
    return m


def fading(name, color, strength=1.0):
    """An emissive material whose opacity the handler sets through the
    Mix Shader's factor (`material.node_tree.nodes["Mix"].inputs[0]`)."""
    m = bpy.data.materials.get(name) or bpy.data.materials.new(name)
    m.use_nodes = True
    nt = m.node_tree
    nt.nodes.clear()
    out = nt.nodes.new("ShaderNodeOutputMaterial")
    mix = nt.nodes.new("ShaderNodeMixShader")
    mix.name = "Mix"
    clear = nt.nodes.new("ShaderNodeBsdfTransparent")
    glow = nt.nodes.new("ShaderNodeEmission")
    glow.inputs["Color"].default_value = (*color, 1)
    glow.inputs["Strength"].default_value = strength
    nt.links.new(clear.outputs[0], mix.inputs[1])
    nt.links.new(glow.outputs[0], mix.inputs[2])
    nt.links.new(mix.outputs[0], out.inputs["Surface"])
    mix.inputs[0].default_value = 1.0
    return m


def lights_and_world(scene, extent):
    """A big warm key, a cool fill and a rim, scaled to a board `extent` units across."""
    world = bpy.data.worlds.new("room")
    world.use_nodes = True
    bg = world.node_tree.nodes["Background"]
    bg.inputs["Color"].default_value = (0.20, 0.17, 0.15, 1)
    bg.inputs["Strength"].default_value = 0.6
    scene.world = world
    for name, loc, energy, color, size in [
        ("key", (-0.6, -0.8, 1.0), 900, (1.0, 0.9, 0.78), 0.5),
        ("fill", (0.9, -0.4, 0.5), 250, (0.8, 0.88, 1.0), 0.6),
        ("rim", (0.2, 1.0, 0.6), 500, (1.0, 0.85, 0.7), 0.3),
    ]:
        data = bpy.data.lights.new(name, "AREA")
        data.energy = energy * (extent / 12) ** 2
        data.color = color
        data.size = extent * size
        obj = bpy.data.objects.new(name, data)
        obj.location = tuple(c * extent for c in loc)
        scene.collection.objects.link(obj)
        # An area light shines along its −Z: pointing +Z along its own
        # position vector aims it at the origin.
        obj.rotation_euler = obj.location.to_track_quat("Z", "Y").to_euler()
```

- [ ] **Step 7: Implement `studio/blender/board.py`**

```python
"""The felt board (smooth hills from capacity) and the sugar: one points
object with a `level` attribute, instanced as gumdrops by geometry nodes."""

import bpy

import animate
from blender import materials

GUMDROP_SCALE = 0.17


def felt_board(d):
    w, h = d.width, d.height
    corners = animate.corner_heights(d.capacity, w, h)
    verts = [(cx - w / 2, h / 2 - cy, corners[cy * (w + 1) + cx]) for cy in range(h + 1) for cx in range(w + 1)]
    faces = [
        (cy * (w + 1) + cx, cy * (w + 1) + cx + 1, (cy + 1) * (w + 1) + cx + 1, (cy + 1) * (w + 1) + cx)
        for cy in range(h)
        for cx in range(w)
    ]
    mesh = bpy.data.meshes.new("felt")
    mesh.from_pydata(verts, [], faces)
    for poly in mesh.polygons:
        poly.use_smooth = True
    obj = bpy.data.objects.new("felt", mesh)
    obj.data.materials.append(materials.felt())
    sub = obj.modifiers.new("smooth", "SUBSURF")
    sub.levels = sub.render_levels = 2
    bpy.context.scene.collection.objects.link(obj)
    return obj, corners


def _gumdrop_prototype():
    bpy.ops.mesh.primitive_uv_sphere_add(radius=0.5, segments=24, ring_count=12)
    g = bpy.context.active_object
    g.name = "gumdrop"
    g.scale = (1, 1, 0.8)
    bpy.ops.object.shade_smooth()
    g.data.materials.append(materials.gumdrop())
    g.hide_render = True
    g.hide_viewport = True
    return g


def _instancer_tree(proto):
    ng = bpy.data.node_groups.new("gumdrops", "GeometryNodeTree")
    ng.interface.new_socket("Geometry", in_out="INPUT", socket_type="NodeSocketGeometry")
    ng.interface.new_socket("Geometry", in_out="OUTPUT", socket_type="NodeSocketGeometry")
    n = ng.nodes
    gin, gout = n.new("NodeGroupInput"), n.new("NodeGroupOutput")
    to_points = n.new("GeometryNodeMeshToPoints")
    info = n.new("GeometryNodeObjectInfo")
    info.inputs["Object"].default_value = proto
    info.inputs["As Instance"].default_value = True
    level = n.new("GeometryNodeInputNamedAttribute")
    level.data_type = "FLOAT"
    level.inputs["Name"].default_value = "level"
    # Scale ∝ cube root of the level so volume tracks sugar.
    root = n.new("ShaderNodeMath")
    root.operation = "POWER"
    root.inputs[1].default_value = 1 / 3
    scale = n.new("ShaderNodeMath")
    scale.operation = "MULTIPLY"
    scale.inputs[1].default_value = GUMDROP_SCALE * 2
    inst = n.new("GeometryNodeInstanceOnPoints")
    l = ng.links
    l.new(gin.outputs[0], to_points.inputs["Mesh"])
    l.new(to_points.outputs[0], inst.inputs["Points"])
    l.new(info.outputs["Geometry"], inst.inputs["Instance"])
    l.new(level.outputs["Attribute"], root.inputs[0])
    l.new(root.outputs[0], scale.inputs[0])
    l.new(scale.outputs[0], inst.inputs["Scale"])
    l.new(inst.outputs[0], gout.inputs[0])
    return ng


def sugar(d, corners, timing):
    """Returns (object, updater); the updater sets each site's level for a frame."""
    w, h = d.width, d.height
    verts = []
    for y in range(h):
        for x in range(w):
            cx, cy = animate.cell_center(x, y, w, h)
            verts.append((cx, cy, animate.cell_height(corners, x, y, w) + 0.05))
    mesh = bpy.data.meshes.new("sugar")
    mesh.from_pydata(verts, [], [])
    mesh.attributes.new("level", "FLOAT", "POINT")
    obj = bpy.data.objects.new("sugar", mesh)
    bpy.context.scene.collection.objects.link(obj)
    proto = _gumdrop_prototype()
    mod = obj.modifiers.new("gumdrops", "NODES")
    mod.node_group = _instancer_tree(proto)

    def update(frame):
        levels = animate.levels_at(d, timing.tick_at(frame))
        mesh.attributes["level"].data.foreach_set("value", levels)
        mesh.update()

    return obj, update
```

- [ ] **Step 8: Implement `studio/blender/scene.py`** (board and camera now; Flumps and overlays are added in Tasks 8–9)

```python
"""Builds one beat's scene and drives it with a single frame-change handler."""

import math

import bpy
from mathutils import Vector

import camera as cam
from blender import board, materials

UPDATERS = []


def reset(scene, preview):
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj)
    scene.render.engine = "BLENDER_EEVEE"
    scene.render.fps = 30
    scene.render.resolution_x, scene.render.resolution_y = (960, 540) if preview else (1920, 1080)
    scene.render.resolution_percentage = 100
    scene.render.use_motion_blur = True
    scene.eevee.taa_render_samples = 16 if preview else 96
    scene.eevee.use_raytracing = not preview
    scene.view_settings.view_transform = "AgX"
    scene.render.image_settings.file_format = "PNG"


def add_camera(scene, beat):
    data = bpy.data.cameras.new("camera")
    data.dof.use_dof = True
    data.dof.aperture_fstop = 4.0
    obj = bpy.data.objects.new("camera", data)
    scene.collection.objects.link(obj)
    scene.camera = obj

    def update(frame):
        eye, target, lens = cam.camera_at(beat.camera, (frame - 1) / 30)
        obj.location = eye
        direction = Vector(target) - Vector(eye)
        obj.rotation_euler = direction.to_track_quat("-Z", "Y").to_euler()
        data.lens = lens
        data.dof.focus_distance = direction.length

    return obj, update


def build_beat(beat, d, preview):
    scene = bpy.context.scene
    reset(scene, preview)
    scene.frame_start, scene.frame_end = 1, beat.frames
    updaters = []
    timing = corners = None
    tracks = {}
    if d is not None:
        timing = beat.timing(d.ticks)
        felt, corners = board.felt_board(d)
        _, update_sugar = board.sugar(d, corners, timing)
        updaters.append(update_sugar)
        materials.lights_and_world(scene, max(d.width, d.height))
    else:
        materials.lights_and_world(scene, 12)
    camera_obj, update_camera = add_camera(scene, beat)
    updaters.append(update_camera)
    return updaters


def install(updaters):
    UPDATERS[:] = updaters

    def on_frame(scene, depsgraph=None):
        frame = scene.frame_current + scene.frame_subframe
        for update in UPDATERS:
            update(frame)

    bpy.app.handlers.frame_change_pre.clear()
    bpy.app.handlers.frame_change_pre.append(on_frame)
    on_frame(bpy.context.scene)
```

- [ ] **Step 9: Implement `studio/render.py`**

```python
"""Renders one beat inside Blender:

    blender -b --factory-startup -P studio/render.py -- EPISODE INDEX [--preview] [--still FRAME]
"""

import argparse
import pathlib
import sys

STUDIO = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(STUDIO))

import bpy  # noqa: E402

import dump  # noqa: E402
import episode  # noqa: E402
from blender import scene  # noqa: E402


def main(argv):
    p = argparse.ArgumentParser()
    p.add_argument("episode")
    p.add_argument("index", type=int)
    p.add_argument("--preview", action="store_true")
    p.add_argument("--still", type=int)
    args = p.parse_args(argv)
    beats = episode.load_episode(args.episode)
    beat = beats[args.index - 1]
    out = STUDIO / "out" / args.episode
    d = dump.load(out / "dumps" / f"{beat.shot}.frames.json") if beat.shot else None
    updaters = scene.build_beat(beat, d, args.preview)
    scene.install(updaters)
    folder = out / ("preview" if args.preview else "beats") / f"{args.index:02d}"
    folder.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(folder / "beat.blend"))
    s = bpy.context.scene
    if args.still is not None:
        s.frame_set(args.still)
        s.render.filepath = str(folder / "still.png")
        bpy.ops.render.render(write_still=True)
    else:
        s.render.filepath = str(folder / "####")
        bpy.ops.render.render(animation=True)


main(sys.argv[sys.argv.index("--") + 1 :] if "--" in sys.argv else [])
```

- [ ] **Step 10: Smoke-render a still and look at it**

```bash
mkdir -p studio/out/sugarscape/dumps
cargo run -q --release -p sugarscape-cli -- shot studio/episodes/sugarscape/shots/landscape.json --out studio/out/sugarscape/dumps/landscape.frames.json
/Applications/Blender.app/Contents/MacOS/Blender -b --factory-startup -P studio/render.py -- sugarscape 1 --preview --still 180 2>&1 | grep -E "Error|Traceback|Saved" 
```
Expected: `Saved: '.../studio/out/sugarscape/preview/01/still.png'`, no traceback. Open `still.png` (Read tool) and check: green felt with two soft hills, gumdrops on the hills sized by capacity, none on the plains, warm light, camera looking down over the board. Fix API errors against Blender 5.2 as they appear (e.g. if `mesh.shade_smooth` is missing, use `for poly in mesh.polygons: poly.use_smooth = True`). Tune `GUMDROP_SCALE`, light energies and felt color until it reads as a toy tabletop; record the final values in the code.

- [ ] **Step 11: Commit**

```bash
python3 -m unittest discover -s studio/tests -t studio -v
git add studio
git commit -m "Studio: beats, felt board with gumdrop sugar, camera, render entry point

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 8: Flumps

**Files:**
- Create: `studio/blender/flump.py`
- Modify: `studio/blender/scene.py` (add agents)

**Interfaces:**
- Consumes: `animate.pose`, `animate.blink`, `animate.Pose`, `dump.tracks`, `materials.knit`, `materials.gloss`, `materials.matte`, `materials.YARN`.
- Produces:
  - `flump.build_flump(name: str, color: str) -> FlumpRig` — `FlumpRig(root, body, eyes)`: `root` is an empty at the Flump's feet (origin of squash), `eyes` an empty whose z scale blinks.
  - `flump.crowd_prototypes() -> list[bpy.types.Collection]` — one hidden collection per yarn color.
  - `flump.crowd_instance(name, collection) -> bpy.types.Object` — an empty instancing a prototype.
  - `flump.apply(obj, pose)` — sets location, rotation z, scale and visibility (`hide_render`) from a `Pose`.
  - In `scene.build_beat`: close-up beats build a rig per agent (color by placement order: cream, coral, teal, lilac, butter), crowd beats instance prototypes (color by `id % 5`); returns an updater posing every agent, and `scene.RIGS: dict[int, FlumpRig]` for overlays.

Flump anatomy, feet at z = 0, facing −Y (toward a camera at −Y): body sphere radius 0.4 at z 0.38 scaled (1, 0.95, 0.9), subsurf 2; four nubbins radius 0.1 (arms at (±0.38, 0, 0.4), feet at (±0.16, −0.06, 0.06) scaled (1, 1.3, 0.6)); eyes black gloss radius 0.075 at (±0.12, −0.36, 0.5) scaled (0.8, 0.5, 1.2), each with a white emissive highlight radius 0.022 at (∓0.025, −0.39, 0.53) parented to the eye; blush matte pink discs radius 0.06 at (±0.24, −0.33, 0.37) scaled (1, 0.3, 0.7). Everything is scaled 0.9 so a Flump fits in its cell.

- [ ] **Step 1: Implement `studio/blender/flump.py`**

```python
"""The Flump: a crocheted blob with nubbin limbs and glossy eyes, built from
primitives. The root empty sits at the feet so scaling it squashes toward
the ground; the eyes' empty blinks."""

from dataclasses import dataclass

import bmesh
import bpy

from blender import materials

SIZE = 0.9
BLUSH = (1.0, 0.55, 0.6)


@dataclass
class FlumpRig:
    root: object
    body: object
    eyes: object


def _sphere(name, radius, location, scale, material, parent, collection, smooth=True, segments=32):
    mesh = bpy.data.meshes.new(name)
    bm = bmesh.new()
    bmesh.ops.create_uvsphere(bm, u_segments=segments, v_segments=segments // 2, radius=radius)
    bm.to_mesh(mesh)
    bm.free()
    for poly in mesh.polygons:
        poly.use_smooth = smooth
    obj = bpy.data.objects.new(name, mesh)
    obj.location, obj.scale = location, scale
    obj.data.materials.append(material)
    obj.parent = parent
    collection.objects.link(obj)
    return obj


def _empty(name, parent, collection, location=(0, 0, 0)):
    e = bpy.data.objects.new(name, None)
    e.location = location
    e.parent = parent
    collection.objects.link(e)
    return e


def build_flump(name, color, collection=None):
    collection = collection or bpy.context.scene.collection
    root = _empty(name, None, collection)
    scale = _empty(f"{name}.size", root, collection)
    scale.scale = (SIZE,) * 3
    yarn = materials.knit(color)
    body = _sphere(f"{name}.body", 0.4, (0, 0, 0.38), (1, 0.95, 0.9), yarn, scale, collection)
    sub = body.modifiers.new("smooth", "SUBSURF")
    sub.levels = sub.render_levels = 1
    for side in (-1, 1):
        _sphere(f"{name}.arm", 0.1, (side * 0.38, 0, 0.4), (1, 1, 1), yarn, scale, collection, segments=16)
        _sphere(f"{name}.foot", 0.1, (side * 0.16, -0.06, 0.06), (1, 1.3, 0.6), yarn, scale, collection, segments=16)
        _sphere(f"{name}.blush", 0.06, (side * 0.24, -0.33, 0.37), (1, 0.3, 0.7), materials.matte("blush", BLUSH), scale, collection, segments=16)
    eyes = _empty(f"{name}.eyes", scale, collection, location=(0, -0.36, 0.5))
    for side in (-1, 1):
        eye = _sphere(f"{name}.eye", 0.075, (side * 0.12, 0, 0), (0.8, 0.5, 1.2), materials.gloss("eye", (0.01, 0.01, 0.012)), eyes, collection, segments=24)
        _sphere(f"{name}.shine", 0.022, (-0.025 / 0.8, -0.03 / 0.5, 0.03 / 1.2), (1 / 0.8, 1 / 0.5, 1 / 1.2), materials.gloss("shine", (1, 1, 1), emission=4.0), eye, collection, segments=12)
    return FlumpRig(root, body, eyes)


def crowd_prototypes():
    protos = []
    for color in materials.YARN:
        coll = bpy.data.collections.new(f"flump-{color}")
        build_flump(f"proto-{color}", color, coll)
        protos.append(coll)
    return protos


def crowd_instance(name, collection):
    e = bpy.data.objects.new(name, None)
    e.instance_type = "COLLECTION"
    e.instance_collection = collection
    bpy.context.scene.collection.objects.link(e)
    return e


def apply(obj, pose):
    obj.hide_render = not pose.visible
    obj.hide_viewport = not pose.visible
    if not pose.visible:
        return
    obj.location = (pose.x, pose.y, pose.z)
    obj.rotation_euler = (0, 0, pose.yaw)
    obj.scale = (pose.sx, pose.sy, pose.sz)
```

Prototype collections are never linked to the scene, so they do not render on their own; instances render them. For a close-up rig, `hide_render` on the root does not hide children — so for rigs, `apply` must set it on the root's whole hierarchy: in `scene.py` hide a rig with `for o in [rig.root, *rig.root.children_recursive]: o.hide_render = not visible`.

- [ ] **Step 2: Add agents to `scene.build_beat`** (inside `if d is not None:`, after the sugar updater; `tracks` replaces the empty dict set at the top)

```python
        tracks = dump_mod.tracks(d)
        w, h = d.width, d.height
        colors = list(materials.YARN)
        RIGS.clear()
        if beat.closeup:
            for i, id_ in enumerate(d.placed):
                RIGS[id_] = flump.build_flump(f"flump{id_}", colors[i % len(colors)])
            for id_, t in tracks.items():
                if id_ not in RIGS:
                    RIGS[id_] = flump.build_flump(f"flump{id_}", colors[id_ % len(colors)])
        else:
            protos = flump.crowd_prototypes()
            instances = {id_: flump.crowd_instance(f"flump{id_}", protos[id_ % len(protos)]) for id_ in tracks}

        def update_agents(frame):
            for id_, t in tracks.items():
                p = animate.pose(t, timing, frame, corners, w, h)
                if beat.closeup:
                    rig = RIGS[id_]
                    flump.apply(rig.root, p)
                    for o in rig.root.children_recursive:
                        o.hide_render = not p.visible
                    rig.eyes.scale = (1, 1, animate.blink(id_, frame) * (1 - 0.4 * p.hunger))
                else:
                    flump.apply(instances[id_], p)

        updaters.append(update_agents)
```

with `import animate`, `import dump as dump_mod`, `from blender import flump`, and module-level `RIGS = {}` in `scene.py`.

- [ ] **Step 3: A close-up shot and beat to look at** — add `studio/episodes/sugarscape/shots/meet.json`:

```json
{"config": {"width": 12, "height": 12, "population": 0,
  "goods": [{"name": "sugar", "map": {"kind": "peaks", "peaks": [{"x": 8, "y": 5, "radius": 5, "height": 4}]}}]},
 "ticks": 12, "seed": 1,
 "place": [{"x": 5, "y": 6, "vision": 3, "metabolism": 1, "sugar": 6}]}
```

and a beat (temporarily first in `BEATS`, to be ordered in Task 12):

```python
    Beat(
        "meet",
        "This is a Flump.",
        5.0,
        shot="meet",
        closeup=True,
        ticks_per_second=1.5,
        lead_in=1.0,
        focus=(0,),
        camera=(Move(0, 5, (-0.5, -5.5, 2.2), (-0.5, 0, 0.5), (0.2, -4.2, 1.8), (0.2, 0.3, 0.5), lens0=50, lens1=55),),
    ),
```

- [ ] **Step 4: Render stills and look**

```bash
cargo run -q --release -p sugarscape-cli -- shot studio/episodes/sugarscape/shots/meet.json --out studio/out/sugarscape/dumps/meet.frames.json
for f in 20 45 60; do /Applications/Blender.app/Contents/MacOS/Blender -b --factory-startup -P studio/render.py -- sugarscape 1 --preview --still $f 2>&1 | grep -E "Error|Traceback"; cp studio/out/sugarscape/preview/01/still.png /tmp/meet-$f.png; done
```
Expected: frame 20 — the Flump mid-drop/landing (spawn); 45 — standing, eyes and blush visible, facing the camera; 60 — mid-hop toward the sugar hill, stretched. Read each PNG. Tune proportions, yarn bump strength and eye shine until the character reads as amigurumi-meets-Kirby; show the user a still before moving on (it is the series' mascot).

- [ ] **Step 5: A crowd still**

Add a temporary crowd beat using a shot `crowd.json` = `{"preset": "ii-2-unit", "ticks": 40, "seed": 1}` with `ticks_per_second=8`, camera `Move(0, 5, WIDE_EYE, WIDE_AT, WIDE_EYE, WIDE_AT)`; render a still at frame 90. Expected: ~400 Flumps in five colors on the board. Note the render time per frame from Blender's output (`Time:`) at preview and, once, at final quality — record both in `studio/README.md`.

- [ ] **Step 6: Commit**

```bash
python3 -m unittest discover -s studio/tests -t studio -v
git add studio
git commit -m "Studio: Flumps — close-up rigs that blink and droop, crowd instances

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 9: Captions and overlays

**Files:**
- Create: `studio/blender/overlays.py`
- Modify: `studio/blender/scene.py` (caption on every beat; overlays by name)

**Interfaces:**
- Consumes: `scene.RIGS`, `animate.pose`, `animate.sight_cells`, `animate.histogram`, `animate.cell_center`, `animate.cell_height`, `materials.fading`, `materials.knit`, `materials.gumdrop`, `dump.Dump.stats`.
- Produces: `overlays.caption(camera_obj, text, seconds) -> updater` (fades in over 0.4 s from 0.3 s, out over 0.4 s before the end; lower third; cream text with a dark shadow copy behind it); overlay builders keyed by name in `overlays.BUILDERS: dict[str, callable(beat, d, ctx) -> updater]`, where `ctx` is a `SimpleNamespace(camera=camera_obj, timing=..., tracks=..., corners=..., rigs=scene.RIGS)`:
  - `"belly"` — a meter above each focus Flump: a felt capsule with a butter-yarn fill scaled by `min(sugar / 20, 1)`.
  - `"sight"` — for focus agent 0: glowing dots on each cell of `sight_cells` for its vision, fading in direction by direction (N, E, S, W, 0.25 s apart) between 0.3 and 0.9 of the first tick interval of each move, and out at the landing.
  - `"labels"` — above each focus Flump: "sees N · eats M" text facing the camera.
  - `"dials"` — two camera-attached felt bars at the upper right, "sight" and "hunger", lengths from `stats["mean_vision"]` / 6 and `stats["mean_metabolism"]` / 4 at the current tick, with the values as text (two decimals).
  - `"stacks"` — above every Flump, a column of sugar cubes (gumdrop material box) whose height is `sugar × 0.04`.
  - `"histogram"` — camera-attached bars at the right third: `histogram([a.sugar for a in agents], 12, top)` where `top` is the 99th percentile of sugar over the dump's last frame, bar heights normalized to the largest count.

- [ ] **Step 1: Implement `studio/blender/overlays.py`**

```python
"""On-screen text and data displays, each a builder returning a per-frame updater."""

import math
import pathlib

import bpy

import animate
from blender import materials

FONT = pathlib.Path(__file__).resolve().parent.parent / "fonts" / "Baloo2.ttf"
CREAM, INK = (1.0, 0.96, 0.88), (0.08, 0.06, 0.05)


def _font():
    return bpy.data.fonts.load(str(FONT), check_existing=True)


def _text(name, body, size, material, parent=None, align="CENTER"):
    cu = bpy.data.curves.new(name, type="FONT")
    cu.body = body
    cu.font = _font()
    cu.size = size
    cu.align_x = align
    cu.align_y = "CENTER"
    cu.extrude = 0.0
    obj = bpy.data.objects.new(name, cu)
    obj.data.materials.append(material)
    obj.parent = parent
    bpy.context.scene.collection.objects.link(obj)
    return obj


def _screen(camera_obj, obj, x, y, depth=4.0):
    """Places `obj` in the camera's view: x, y in [-1, 1] of the frame's width and height."""
    lens = camera_obj.data.lens
    half_w = depth * 18 / lens  # sensor width 36 mm
    obj.parent = camera_obj
    obj.location = (x * half_w, y * half_w * 9 / 16, -depth)
    obj.rotation_euler = (0, 0, 0)


def _fade(t, start, end, ramp=0.4):
    return max(0.0, min(1.0, (t - start) / ramp, (end - t) / ramp))


def caption(camera_obj, text, seconds):
    if not text:
        return lambda frame: None
    ink = materials.fading("caption-ink", INK, 1.0)
    cream = materials.fading("caption", CREAM, 1.5)
    shadow = _text("caption-shadow", text, 0.2, ink)
    front = _text("caption", text, 0.2, cream)
    _screen(camera_obj, shadow, 0.004, -0.72, depth=4.002)
    _screen(camera_obj, front, 0.0, -0.715)

    def update(frame):
        t = (frame - 1) / 30
        a = _fade(t, 0.3, seconds - 0.1)
        for m in (ink, cream):
            m.node_tree.nodes["Mix"].inputs[0].default_value = a
        # the camera's lens can change: keep the text's screen position
        _screen(camera_obj, shadow, 0.004, -0.72, depth=4.002)
        _screen(camera_obj, front, 0.0, -0.715)

    return update


def belly(beat, d, ctx):
    bars = {}
    for i in beat.focus:
        id_ = d.placed[i]
        rig = ctx.rigs[id_]
        case = bpy.data.objects.new(f"belly{id_}", None)
        case.parent = rig.root
        case.location = (0, 0, 1.15)
        bpy.context.scene.collection.objects.link(case)
        bpy.ops.mesh.primitive_cube_add(size=1)
        frame_obj = bpy.context.active_object
        frame_obj.scale = (0.5, 0.08, 0.12)
        frame_obj.parent = case
        frame_obj.data.materials.append(materials.knit("cream"))
        bpy.ops.mesh.primitive_cube_add(size=1)
        fill = bpy.context.active_object
        fill.parent = case
        fill.location = (-0.23, -0.05, 0)
        fill.data.materials.append(materials.knit("butter"))
        bars[id_] = (case, fill)

    def update(frame):
        for id_, (case, fill) in bars.items():
            p = animate.pose(ctx.tracks[id_], ctx.timing, frame, ctx.corners, d.width, d.height)
            case.hide_render = not p.visible
            for c in case.children:
                c.hide_render = not p.visible
            level = min(max(p.sugar / 20, 0.0), 1.0)
            fill.scale = (max(level, 0.001) * 0.46, 0.06, 0.09)
            fill.location.x = -0.23 + level * 0.23

    return update


def sight(beat, d, ctx):
    id_ = d.placed[beat.focus[0]]
    t = ctx.tracks[id_]
    glow = materials.fading("sight", (1.0, 0.95, 0.6), 3.0)
    dots = []
    for k in range(len(t.cells)):
        x, y = t.cells[k]
        dirs = animate.sight_cells(x, y, d.frames[t.first + k].agents[id_].vision, d.width, d.height)
        dots.append(dirs)
    pool = []
    most = max((sum(len(r) for r in dirs) for dirs in dots), default=0)
    for i in range(most):
        bpy.ops.mesh.primitive_uv_sphere_add(radius=0.07, segments=12, ring_count=6)
        o = bpy.context.active_object
        o.data.materials.append(glow)
        pool.append(o)

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        k = min(max(int(tick) - t.first, 0), len(dots) - 1)
        a = tick - int(tick)
        i = 0
        for j, row in enumerate(dots[k]):
            shown = animate.smoothstep((a - 0.1 - 0.12 * j) / 0.1) * (1 - animate.smoothstep((a - 0.8) / 0.08))
            for (cx, cy) in row:
                o = pool[i]
                i += 1
                x, y = animate.cell_center(cx, cy, d.width, d.height)
                o.location = (x, y, animate.cell_height(ctx.corners, cx, cy, d.width) + 0.12)
                o.scale = (shown,) * 3
                o.hide_render = shown <= 0.01
        for o in pool[i:]:
            o.hide_render = True

    return update


def labels(beat, d, ctx):
    items = []
    for i in beat.focus:
        id_ = d.placed[i]
        a = d.frames[ctx.tracks[id_].first].agents[id_]
        txt = _text(f"label{id_}", f"sees {a.vision} · eats {a.metabolism}", 0.28, materials.fading("label", CREAM, 1.2), parent=ctx.rigs[id_].root)
        txt.location = (0, 0, 1.3)
        items.append(txt)

    def update(frame):
        for txt in items:
            direction = ctx.camera.matrix_world.translation - txt.matrix_world.translation
            txt.rotation_euler = direction.to_track_quat("Z", "Y").to_euler()

    return update


def _bar(name, color, camera_obj, x, y):
    bpy.ops.mesh.primitive_cube_add(size=1)
    back = bpy.context.active_object
    back.name = f"{name}-back"
    back.data.materials.append(materials.knit("cream"))
    _screen(camera_obj, back, x, y)
    back.scale = (0.9, 0.12, 0.02)
    bpy.ops.mesh.primitive_cube_add(size=1)
    fill = bpy.context.active_object
    fill.name = name
    fill.data.materials.append(materials.knit(color))
    _screen(camera_obj, fill, x, y, depth=3.98)
    return back, fill


def dials(beat, d, ctx):
    rows = [("mean_vision", "sight", 6.0, "teal", 0.62), ("mean_metabolism", "hunger", 4.0, "coral", 0.45)]
    parts = []
    for key, title, top, color, y in rows:
        back, fill = _bar(title, color, ctx.camera, 0.62, y)
        text = _text(f"{title}-text", "", 0.1, materials.fading(f"{title}-ink", CREAM, 1.2), align="LEFT")
        _screen(ctx.camera, text, 0.43, y + 0.09)
        parts.append((d.stats[key], top, title, back, fill, text))

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        for series, top, title, back, fill, text in parts:
            i = min(int(round(tick)), len(series) - 1)
            v = series[i]
            share = min(max(v / top, 0), 1)
            base_x = back.location.x - 0.45 * back.scale.x / 0.9
            fill.scale = (max(share, 0.001) * 0.9 * back.scale.x / 0.9, 0.1, 0.02)
            fill.location.x = base_x + fill.scale.x / 2
            text.data.body = f"average {title}: {v:.2f}"

    return update


def stacks(beat, d, ctx):
    cube_mat = materials.gumdrop()
    columns = {}
    for id_ in ctx.tracks:
        bpy.ops.mesh.primitive_cube_add(size=1)
        c = bpy.context.active_object
        c.name = f"stack{id_}"
        c.data.materials.append(cube_mat)
        columns[id_] = c

    def update(frame):
        for id_, c in columns.items():
            p = animate.pose(ctx.tracks[id_], ctx.timing, frame, ctx.corners, d.width, d.height)
            c.hide_render = not p.visible
            if p.visible:
                height = max(p.sugar * 0.04, 0.001)
                c.scale = (0.35, 0.35, height)
                c.location = (p.x, p.y, p.z + 0.85 * p.sz + height / 2)

    return update


def histogram(beat, d, ctx):
    bins = 12
    last = sorted(a.sugar for a in d.frames[-1].agents.values())
    top = last[int(0.99 * (len(last) - 1))] if last else 1.0
    bars = []
    for i in range(bins):
        bpy.ops.mesh.primitive_cube_add(size=1)
        b = bpy.context.active_object
        b.data.materials.append(materials.knit("butter"))
        _screen(ctx.camera, b, 0.45 + i * 0.04, -0.35)
        bars.append(b)

    def update(frame):
        tick = ctx.timing.tick_at(frame)
        f = d.frames[min(int(round(tick)), d.ticks)]
        counts = animate.histogram([a.sugar for a in f.agents.values()], bins, top)
        most = max(max(counts), 1)
        for b, n in zip(bars, counts):
            height = max(n / most * 0.6, 0.002)
            b.scale = (0.035, height, 0.02)
            b.location.y = -0.35 * 4 * 18 / ctx.camera.data.lens * 9 / 16 + height / 2

    return update


BUILDERS = {
    "belly": belly,
    "sight": sight,
    "labels": labels,
    "dials": dials,
    "stacks": stacks,
    "histogram": histogram,
}
```

- [ ] **Step 2: Wire into `scene.build_beat`** — after the agents updater and the camera:

```python
    ctx = SimpleNamespace(camera=camera_obj, timing=timing, tracks=tracks, corners=corners, rigs=RIGS)
    for name in beat.overlays:
        updaters.append(overlays.BUILDERS[name](beat, d, ctx))
    updaters.append(overlays.caption(camera_obj, beat.caption, beat.seconds))
```

(`from types import SimpleNamespace`, `from blender import overlays`; build the camera before the overlays.) An unknown overlay name must fail: `overlays.BUILDERS[name]` raises `KeyError` naming it — keep it that way.

- [ ] **Step 3: Stills** — give the `meet` beat `overlays=("belly", "labels", "sight")` temporarily, render stills at frames 45 and 58, and read them. Expected: caption readable at the bottom with a soft shadow; the belly bar above the Flump; its label; sight dots along four rows during the hop's first half. For the crowd beat, try `overlays=("dials",)` and `("stacks", "histogram")` on a `ii-5-wealth` shot; check that nothing overlaps the caption. Then remove the temporary overlays (Task 12 sets each beat's).

- [ ] **Step 4: Commit**

```bash
git add studio
git commit -m "Studio: captions, belly meters, sight lines, labels, dials, wealth stacks, histogram

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 10: The cut and the build script

**Files:**
- Create: `studio/cut.py`, `studio/tests/test_cut.py`, `studio/build.py`

**Interfaces:**
- Consumes: `episode.load_episode`, `episode.Beat.frames`, `episode.episode_dir`.
- Produces: `cut.total_frames(frames: list[int], dissolve: int) -> int`; `cut.command(folders: list[str], frames: list[int], out: str, dissolve: int = 12, fps: int = 30) -> list[str]` (an ffmpeg argv); `python3 studio/build.py EPISODE [--preview] [--beat N] [--skip-shots] [--skip-render]`.

- [ ] **Step 1: Write the failing tests** (`studio/tests/test_cut.py`)

```python
import unittest

import cut


class CutTest(unittest.TestCase):
    def test_total_frames_subtracts_dissolves(self):
        self.assertEqual(cut.total_frames([90, 60, 30], 12), 180 - 24)
        self.assertEqual(cut.total_frames([90], 12), 90)

    def test_one_beat_is_a_plain_encode(self):
        argv = cut.command(["b/01"], [90], "o.mp4")
        self.assertIn("b/01/%04d.png", argv)
        self.assertNotIn("-filter_complex", argv)
        self.assertEqual(argv[-1], "o.mp4")
        self.assertIn("yuv420p", argv)

    def test_dissolve_offsets_accumulate(self):
        argv = cut.command(["a", "b", "c"], [90, 60, 30], "o.mp4", dissolve=12)
        graph = argv[argv.index("-filter_complex") + 1]
        # offsets: first at (90 − 12)/30 = 2.6 s; second at (90 + 60 − 24)/30 = 4.2 s
        self.assertIn("xfade=transition=fade:duration=0.4:offset=2.6", graph)
        self.assertIn("xfade=transition=fade:duration=0.4:offset=4.2", graph)
        self.assertTrue(graph.endswith("[v2]"))


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run to verify they fail** — expected `No module named 'cut'`.

- [ ] **Step 3: Implement `studio/cut.py`**

```python
"""The final cut: beats' PNG sequences joined with cross-dissolves by ffmpeg."""


def total_frames(frames, dissolve):
    return sum(frames) - dissolve * (len(frames) - 1)


def command(folders, frames, out, dissolve=12, fps=30):
    argv = ["ffmpeg", "-y", "-loglevel", "error"]
    for folder in folders:
        argv += ["-framerate", str(fps), "-start_number", "1", "-i", f"{folder}/%04d.png"]
    encode = ["-c:v", "libx264", "-preset", "slow", "-crf", "16", "-pix_fmt", "yuv420p", "-r", str(fps), "-movflags", "+faststart"]
    if len(folders) == 1:
        return argv + encode + [out]
    parts, label, elapsed = [], "[0:v]", 0
    for i in range(1, len(folders)):
        elapsed += frames[i - 1] - dissolve
        offset = round(elapsed / fps, 4)
        nxt = f"[v{i}]"
        parts.append(f"{label}[{i}:v]xfade=transition=fade:duration={dissolve / fps:g}:offset={offset:g}{nxt}")
        label = nxt
    return argv + ["-filter_complex", ";".join(parts), "-map", label] + encode + [out]
```

- [ ] **Step 4: Run to verify they pass.**

- [ ] **Step 5: Implement `studio/build.py`**

```python
"""Builds an episode: shots → frame dumps → beat renders → the cut.

    python3 studio/build.py EPISODE [--preview] [--beat N] [--skip-shots] [--skip-render]
"""

import argparse
import os
import pathlib
import subprocess
import sys

STUDIO = pathlib.Path(__file__).resolve().parent
REPO = STUDIO.parent
sys.path.insert(0, str(STUDIO))

import cut  # noqa: E402
import episode  # noqa: E402

BLENDER = os.environ.get("BLENDER", "/Applications/Blender.app/Contents/MacOS/Blender")
CLI = REPO / "target" / "release" / "sugarscape"


def run(argv):
    print("$", " ".join(str(a) for a in argv), flush=True)
    subprocess.run([str(a) for a in argv], check=True)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("episode")
    p.add_argument("--preview", action="store_true")
    p.add_argument("--beat", type=int)
    p.add_argument("--skip-shots", action="store_true")
    p.add_argument("--skip-render", action="store_true")
    args = p.parse_args()
    beats = episode.load_episode(args.episode)
    out = STUDIO / "out" / args.episode
    (out / "dumps").mkdir(parents=True, exist_ok=True)
    chosen = [args.beat] if args.beat else range(1, len(beats) + 1)
    if not args.skip_shots:
        run(["cargo", "build", "--release", "-q", "-p", "sugarscape-cli"])
        for shot in sorted({beats[i - 1].shot for i in chosen if beats[i - 1].shot}):
            run([CLI, "shot", episode.episode_dir(args.episode) / "shots" / f"{shot}.json", "--out", out / "dumps" / f"{shot}.frames.json"])
    if not args.skip_render:
        for i in chosen:
            extra = ["--preview"] if args.preview else []
            run([BLENDER, "-b", "--factory-startup", "-P", STUDIO / "render.py", "--", args.episode, i, *extra])
    if args.beat:
        return
    kind = "preview" if args.preview else "beats"
    folders = [str(out / kind / f"{i:02d}") for i in range(1, len(beats) + 1)]
    frames = [b.frames for b in beats]
    movie = out / f"{args.episode}{'-preview' if args.preview else ''}.mp4"
    run(cut.command(folders, frames, str(movie)))
    probe = subprocess.run(
        ["ffprobe", "-v", "error", "-count_frames", "-select_streams", "v:0", "-show_entries", "stream=nb_read_frames", "-of", "csv=p=0", str(movie)],
        check=True, capture_output=True, text=True,
    )
    got, want = int(probe.stdout.strip()), cut.total_frames(frames, 12)
    if got != want:
        sys.exit(f"{movie}: {got} frames, expected {want}")
    print(f"{movie}: {got} frames ({got / 30:.1f} s)")


if __name__ == "__main__":
    main()
```

- [ ] **Step 6: Try it on the beats so far** — `python3 studio/build.py sugarscape --preview`. Expected: every beat renders, the cut prints `...sugarscape-preview.mp4: N frames`. Play it (`open studio/out/sugarscape/sugarscape-preview.mp4`) and check the dissolves.

- [ ] **Step 7: Commit**

```bash
python3 -m unittest discover -s studio/tests -t studio -v
git add studio
git commit -m "Studio: ffmpeg cut with dissolves, and the build script with a frame-count check

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 11: Measure the pilot's claims

**Files:**
- Create: `studio/measure.py`, `studio/tests/test_measure.py`, `studio/episodes/sugarscape/measurements.md` (generated, committed)

**Interfaces:**
- Consumes: `target/release/sugarscape run --preset P --seed S --ticks T --series-csv PATH --agents-csv PATH`; series columns `population`, `mean_vision`, `mean_metabolism`, `gini`; agents CSV column `sugar`, `x`, `y`; `sugarscape shot` for the landscape's capacity.
- Produces: `measure.summary(values) -> dict(median, lo, hi)` (min and max); `measure.typical_seed(rows: dict[int, dict[str, float]], keys) -> int` (the seed minimizing the sum over `keys` of |value − median| / (hi − lo)); the report.

Claims measured (seeds 1–20):
- **Beat 7** (`ii-2-unit`, 300 ticks): population at ticks 0, 25, 300; share of survivors at tick 300 on cells of capacity ≥ 3 vs the share of such cells.
- **Beat 8** (same runs): mean vision and mean metabolism at 0 and 300; the number of seeds where vision rose and metabolism fell.
- **Beat 9** (`ii-5-wealth`, 500 ticks): Gini at 500; mean sugar ÷ median sugar; the top 10 %'s share of all sugar.

- [ ] **Step 1: Write the failing test** (`studio/tests/test_measure.py`)

```python
import unittest

import measure


class MeasureTest(unittest.TestCase):
    def test_summary(self):
        self.assertEqual(measure.summary([3, 1, 2]), {"median": 2, "lo": 1, "hi": 3})

    def test_typical_seed_is_closest_to_the_medians(self):
        rows = {1: {"a": 0, "b": 10}, 2: {"a": 5, "b": 5}, 3: {"a": 10, "b": 0}}
        self.assertEqual(measure.typical_seed(rows, ["a", "b"]), 2)


if __name__ == "__main__":
    unittest.main()
```

- [ ] **Step 2: Run to verify it fails.**

- [ ] **Step 3: Implement `studio/measure.py`**

```python
"""Measures the pilot's captioned claims over 20 seeds and writes
studio/episodes/sugarscape/measurements.md. Run: python3 studio/measure.py"""

import csv
import json
import pathlib
import statistics
import subprocess
import tempfile

STUDIO = pathlib.Path(__file__).resolve().parent
REPO = STUDIO.parent
CLI = REPO / "target" / "release" / "sugarscape"
SEEDS = range(1, 21)


def summary(values):
    return {"median": statistics.median(values), "lo": min(values), "hi": max(values)}


def typical_seed(rows, keys):
    stats = {k: summary([r[k] for r in rows.values()]) for k in keys}

    def distance(seed):
        return sum(abs(rows[seed][k] - stats[k]["median"]) / ((stats[k]["hi"] - stats[k]["lo"]) or 1) for k in keys)

    return min(sorted(rows), key=distance)


def _run(preset, seed, ticks, tmp):
    series, agents = tmp / f"{preset}-{seed}.csv", tmp / f"{preset}-{seed}-agents.csv"
    subprocess.run([CLI, "run", "--preset", preset, "--seed", str(seed), "--ticks", str(ticks), "--series-csv", series, "--agents-csv", agents], check=True)
    with open(series) as f:
        s = list(csv.DictReader(f))
    with open(agents) as f:
        a = list(csv.DictReader(f))
    return s, a


def _capacity():
    shot = json.dumps({"preset": "ii-2-unit", "ticks": 0, "set": {"population": 0}})
    out = subprocess.run([CLI, "shot", "/dev/stdin"], input=shot, capture_output=True, text=True, check=True)
    return json.loads(out.stdout)["capacity"]


def main():
    subprocess.run(["cargo", "build", "--release", "-q", "-p", "sugarscape-cli"], check=True, cwd=REPO)
    cap = _capacity()
    hill_share = sum(c >= 3 for c in cap) / len(cap)
    ii2, ii5 = {}, {}
    with tempfile.TemporaryDirectory() as t:
        tmp = pathlib.Path(t)
        for seed in SEEDS:
            s, a = _run("ii-2-unit", seed, 300, tmp)
            on_hills = sum(cap[int(r["y"]) * 50 + int(r["x"])] >= 3 for r in a) / max(len(a), 1)
            ii2[seed] = {
                "pop0": float(s[0]["population"]), "pop25": float(s[25]["population"]), "pop300": float(s[300]["population"]),
                "on_hills": on_hills,
                "vision0": float(s[0]["mean_vision"]), "vision300": float(s[300]["mean_vision"]),
                "metab0": float(s[0]["mean_metabolism"]), "metab300": float(s[300]["mean_metabolism"]),
            }
            s, a = _run("ii-5-wealth", seed, 500, tmp)
            sugar = sorted(float(r["sugar"]) for r in a)
            top = sugar[int(len(sugar) * 0.9):]
            ii5[seed] = {
                "gini": float(s[500]["gini"]),
                "mean_over_median": statistics.mean(sugar) / statistics.median(sugar),
                "top10_share": sum(top) / sum(sugar),
            }
    rose = sum(r["vision300"] > r["vision0"] and r["metab300"] < r["metab0"] for r in ii2.values())
    lines = ["# Pilot measurements (seeds 1–20)", "", "Generated by `python3 studio/measure.py`.", ""]
    lines += ["## ii-2-unit, 300 ticks (beats 7–8)", "", "| measure | median | min | max |", "|---|---|---|---|"]
    for k in ["pop0", "pop25", "pop300", "on_hills", "vision0", "vision300", "metab0", "metab300"]:
        v = summary([r[k] for r in ii2.values()])
        lines.append(f"| {k} | {v['median']:.3f} | {v['lo']:.3f} | {v['hi']:.3f} |")
    lines += ["", f"Share of cells with capacity ≥ 3: {hill_share:.3f}.", f"Seeds where mean vision rose and mean metabolism fell: {rose} of 20.",
              f"Typical seed: {typical_seed(ii2, ['pop300', 'on_hills', 'vision300', 'metab300'])}.", ""]
    lines += ["## ii-5-wealth, 500 ticks (beat 9)", "", "| measure | median | min | max |", "|---|---|---|---|"]
    for k in ["gini", "mean_over_median", "top10_share"]:
        v = summary([r[k] for r in ii5.values()])
        lines.append(f"| {k} | {v['median']:.3f} | {v['lo']:.3f} | {v['hi']:.3f} |")
    lines += ["", f"Typical seed: {typical_seed(ii5, ['gini', 'top10_share'])}.", ""]
    path = STUDIO / "episodes" / "sugarscape" / "measurements.md"
    path.write_text("\n".join(lines))
    print(path.read_text())


if __name__ == "__main__":
    main()
```

(`sugarscape shot /dev/stdin` works on macOS because `read_to_string` reads the pipe; if it does not, write the JSON to a temp file.)

- [ ] **Step 4: Run tests, then measure**

Run: `python3 -m unittest discover -s studio/tests -t studio -v` (pass), then `python3 studio/measure.py`.

- [ ] **Step 5: Judge each caption against the numbers, and write the verdicts into `measurements.md` under a "Captions" heading**

- Beat 7 "many poof early; survivors crowd the hills": holds if `pop300` median is well below 400 and `on_hills` median is well above the hill share.
- Beat 8 "Nobody told them to gather there" is about gathering (beat 7's numbers); the dials' claim — sight up, hunger down — holds only if it happens in at least 18 of 20 seeds. If not, drop the `dials` overlay from beat 8 and say what the numbers do show.
- Beat 9 "some are rich": holds if Gini median is clearly above the start (≈ 0.23 at tick 0 for ii-2 — read ii-5's tick-0 Gini from a run) and mean/median > 1.
- Record the typical seeds; Task 12 renders those.

If a claim fails, change the caption in Task 12 to what the numbers support, and tell the user.

- [ ] **Step 6: Commit**

```bash
git add studio/measure.py studio/tests/test_measure.py studio/episodes/sugarscape/measurements.md
git commit -m "Studio: measure the pilot's captions over 20 seeds

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 12: The pilot's beats

**Files:**
- Modify: `studio/episodes/sugarscape/beats.py`
- Create/modify: `studio/episodes/sugarscape/shots/{grow,landscape,meet,hunger,look,differ,crowd,wealth}.json`

**Interfaces:**
- Consumes: everything above; Task 11's typical seeds.

- [ ] **Step 1: Write the shots**

`grow.json` — beat 1, one gumdrop regrowing from empty on a small peak (G₁, a notch a tick):
```json
{"config": {"width": 5, "height": 5, "population": 0,
  "goods": [{"name": "sugar", "map": {"kind": "peaks", "peaks": [{"x": 2, "y": 2, "radius": 1.5, "height": 4}]}}]},
 "ticks": 6, "empty": true}
```
`landscape.json` — beat 2 (already written).
`meet.json` — beat 3 (already written; one Flump dropping in beside a small hill).
`hunger.json` — beat 4: one Flump beside sugar that eats it and then sees its belly drain; a second on bare felt, vision 1, metabolism 3, sugar 4, that starves:
```json
{"config": {"width": 12, "height": 12, "population": 0,
  "goods": [{"name": "sugar", "map": {"kind": "peaks", "peaks": [{"x": 4, "y": 6, "radius": 2.5, "height": 4}]}}]},
 "ticks": 6, "seed": 1,
 "place": [{"x": 4, "y": 6, "vision": 1, "metabolism": 2, "sugar": 4},
           {"x": 9, "y": 6, "vision": 1, "metabolism": 3, "sugar": 4}]}
```
Verify in the dump: placed[1] has a `starvation` death by tick 3 and placed[0] stays alive; adjust sugar/metabolism (never the engine) until it does.
`look.json` — beat 5: a Flump with vision 4 two cells off a hill:
```json
{"config": {"width": 12, "height": 12, "population": 0,
  "goods": [{"name": "sugar", "map": {"kind": "peaks", "peaks": [{"x": 8, "y": 5, "radius": 4, "height": 4}]}}]},
 "ticks": 4, "seed": 2, "empty": false,
 "place": [{"x": 3, "y": 5, "vision": 4, "metabolism": 1, "sugar": 6}]}
```
`differ.json` — beat 6: two Flumps side by side on flat sugar 1: vision 6 / metabolism 1 and vision 1 / metabolism 4:
```json
{"config": {"width": 12, "height": 12, "population": 0,
  "goods": [{"name": "sugar", "map": {"kind": "flat", "capacity": 1}}]},
 "ticks": 3, "seed": 1,
 "place": [{"x": 4, "y": 6, "vision": 6, "metabolism": 1, "sugar": 8},
           {"x": 7, "y": 6, "vision": 1, "metabolism": 4, "sugar": 8}]}
```
`crowd.json` — beats 7–8: `{"preset": "ii-2-unit", "ticks": 300, "seed": <typical ii-2 seed from measurements.md>}`.
`wealth.json` — beat 9: `{"preset": "ii-5-wealth", "ticks": 500, "seed": <typical ii-5 seed>}`.

- [ ] **Step 2: Write `BEATS`** (replace the file's list; captions exactly as below unless Task 11 changed them)

```python
HILL = (10.5, 9.5, 0.4)

BEATS = [
    Beat("grow", "This is sugar.", 5.0, shot="grow", closeup=True, ticks_per_second=1.5, lead_in=0.8,
         camera=(Move(0, 5, (0, -3.2, 1.6), (0, 0, 0.3), (0, -2.6, 1.3), (0, 0, 0.3), lens0=60, lens1=65),)),
    Beat("landscape", "Sugar grows in some places… but not others.", 6.0, shot="landscape",
         camera=(Move(0, 6, (10, -4, 4), HILL, WIDE_EYE, WIDE_AT, lens0=35, lens1=40),)),
    Beat("meet", "This is a Flump.", 5.0, shot="meet", closeup=True, ticks_per_second=1.0, lead_in=1.2, focus=(0,),
         camera=(Move(0, 5, (-0.5, -5.5, 2.2), (-0.5, 0, 0.5), (0.2, -4.2, 1.8), (0.2, 0.3, 0.5), lens0=50, lens1=55),)),
    Beat("hunger", "Flumps eat sugar. Without it, they die.", 7.0, shot="hunger", closeup=True, ticks_per_second=1.0,
         lead_in=0.8, focus=(0, 1), overlays=("belly",),
         camera=(Move(0, 7, (0.5, -7.5, 3.2), (0.5, 0, 0.4), (0.5, -6.5, 2.8), (0.5, 0, 0.4)),)),
    Beat("look", "A Flump can see a little way…", 6.0, shot="look", closeup=True, ticks_per_second=0.7, lead_in=0.8,
         focus=(0,), overlays=("sight",),
         camera=(Move(0, 6, (-1.5, -6.5, 4.5), (0, 0, 0.3), (0.5, -6, 4.2), (0.5, 0, 0.3)),)),
    Beat("differ", "…and every Flump is different.", 5.0, shot="differ", closeup=True, ticks_per_second=0.8,
         lead_in=0.8, focus=(0, 1), overlays=("labels",),
         camera=(Move(0, 5, (0, -5.5, 2.0), (0, 0, 0.6), (0, -5.0, 1.9), (0, 0, 0.6)),)),
    Beat("crowd", "Now: 400 Flumps.", 9.0, shot="crowd", ticks_per_second=10, lead_in=1.0,
         camera=(Move(0, 9, (0, -38, 26), (0, 0, 0), WIDE_EYE, WIDE_AT, orbit=0.35),)),
    Beat("gather", "Nobody told them to gather there.", 7.0, shot="crowd", ticks_per_second=30, start_tick=90,
         overlays=("dials",),
         camera=(Move(0, 7, (22, -18, 16), HILL, (16, -12, 12), HILL, lens0=40, lens1=45),)),
    Beat("rich", "Same rules for everyone. So why are some rich?", 10.0, shot="wealth", ticks_per_second=40,
         lead_in=0.5, overlays=("stacks", "histogram"),
         camera=(Move(0, 10, (-6, -40, 24), (-4, 0, 0), (-6, -34, 20), (-4, 0, 0), orbit=-0.2),)),
    Beat("end", "Sugarscape — Epstein & Axtell, 1996\nndouglas.github.io/SugarScape", 4.0,
         camera=(Move(0, 4, (0, -6, 2), (0, 0, 0), (0, -6, 2), (0, 0, 0)),)),
]
```

The end card has no shot: `scene.build_beat` with `d is None` builds only lights, a felt backdrop (add a 12 × 7 felt plane when `d is None`) and one Flump rig waving (its arm z rotation `0.4·sin(2π·t)`), plus the caption, centered (for the end beat, place the caption at y = 0 instead of the lower third: `caption(..., y=0.0)`; add that parameter with default −0.715).

Beat 8 (`gather`) runs its dials only if Task 11 found the rise in ≥ 18 of 20 seeds; otherwise remove `"dials"`.

- [ ] **Step 3: Preview every beat, then look**

Run: `python3 studio/build.py sugarscape --preview`
Then for each beat read 3 frames (`studio/out/sugarscape/preview/NN/0030.png`, the middle frame, the last frame) and check against the storyboard: the beat's action happens inside its seconds; captions readable and not covering the action; close-ups framed on the focus Flumps; nothing clipping the camera. Adjust camera moves, `ticks_per_second` and `lead_in` (never the shots' results). Play `sugarscape-preview.mp4`.

- [ ] **Step 4: Show the user the preview cut** (send the MP4) and apply their notes.

- [ ] **Step 5: Commit**

```bash
git add studio/episodes/sugarscape studio/blender
git commit -m "Pilot: the Sugarscape episode's shots and beats

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```

---

### Task 13: Final render

**Files:**
- Modify: `studio/README.md` (render times)

- [ ] **Step 1: Render** — `python3 studio/build.py sugarscape` (run in the background; it takes hours: note the per-frame time from Task 8 × the total frames).

- [ ] **Step 2: Verify** — the build's own check prints `sugarscape.mp4: N frames` with N = `cut.total_frames`. Then `ffprobe -v error -show_entries stream=width,height,r_frame_rate,pix_fmt -of csv=p=0 studio/out/sugarscape/sugarscape.mp4` → `1920,1080,30/1,yuv420p`. Check size < 100 MB (Bluesky's limit): `ls -lh studio/out/sugarscape/sugarscape.mp4`; if over, re-run only the cut with `-crf 20`.

- [ ] **Step 3: Watch it end to end** (open the MP4), then send it to the user.

- [ ] **Step 4: Record render times in `studio/README.md` and commit**

```bash
git add studio/README.md
git commit -m "Studio: record the pilot's render times

Claude-Session: https://claude.ai/code/session_01Usj7HuPZDjXNjYUHtaDZki"
```
