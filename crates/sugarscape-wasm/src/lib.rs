//! JavaScript-facing wrapper around the Sugarscape core. Errors cross the
//! boundary as JSON strings of `[{ field, message }]`.

mod experiment_view;
pub use experiment_view::{
    experiment_catalog_json, experiment_recorded_json, experiment_run_json,
    experiment_validate_json,
};

use sugarscape_core::config::{Config, FieldError};
use sugarscape_core::edit::AgentOverrides;
use sugarscape_core::geometry::Pos;
use sugarscape_core::model::{Model, ModelConfig, ModelKind, ModelWorld};
use sugarscape_core::sweep::{RunResult, Sweep, SweepResult};
use sugarscape_core::world::World;
use sugarscape_core::{network, presets, stats, sweep};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn field_errors(errors: Vec<FieldError>) -> JsValue {
    JsValue::from_str(&serde_json::to_string(&errors).expect("errors serialize"))
}

fn edit_error(message: String) -> JsValue {
    field_errors(vec![FieldError::new("edit", message)])
}

/// A world fingerprint as `0x` and 16 hex digits, leading zeros kept, so every fingerprint has the
/// same width as the golden tests' `0x…` literals.
pub fn fingerprint_hex(fingerprint: u64) -> String {
    format!("{fingerprint:#018x}")
}

/// JSON list of every model's presets (`presets::catalog`): the
/// sugarscape's first, each exactly as before milestone 9.
#[wasm_bindgen]
pub fn presets_json() -> String {
    serde_json::to_string(&presets::catalog()).expect("presets serialize")
}

/// JSON `{ <model>: [param, …] }`: the Rules panel's fields of every model
/// that has a schema (every model but the sugarscape).
#[wasm_bindgen]
pub fn model_schemas_json() -> String {
    let schemas: serde_json::Map<String, serde_json::Value> = ModelKind::ALL
        .iter()
        .filter(|k| !k.schema().is_empty())
        .map(|k| {
            let schema = serde_json::to_value(k.schema()).expect("schemas serialize");
            (k.as_str().to_string(), schema)
        })
        .collect();
    serde_json::to_string(&schemas).expect("schemas serialize")
}

#[wasm_bindgen]
pub fn default_config_json() -> String {
    serde_json::to_string(&Config::default()).expect("config serializes")
}

/// Checked laboratory configuration, using the same core construction as the CLI.
#[wasm_bindgen]
pub fn protection_config_json(lab_json: &str) -> Result<String, JsValue> {
    let config = read_protection_config(lab_json)?;
    Ok(serde_json::to_string(&config).expect("config serializes"))
}

fn read_protection_config(lab_json: &str) -> Result<Config, JsValue> {
    let lab = serde_json::from_str(lab_json).map_err(|error| {
        field_errors(vec![FieldError::new("protection_lab", error.to_string())])
    })?;
    let config = sugarscape_core::minds::protection::lab::rig_config(lab);
    config.validate().map_err(field_errors)?;
    Ok(config)
}

/// One core episode with researcher-only action and cohort records.
#[wasm_bindgen]
pub fn protection_episode_json(lab_json: &str, seed: &str) -> Result<String, JsValue> {
    let config = read_protection_config(lab_json)?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let record = sugarscape_core::minds::protection::runner::run_episode(
        config.protection_lab.expect("checked lab"),
        seed,
        true,
    )
    .map_err(|message| field_errors(vec![FieldError::new("episode", message)]))?;
    Ok(serde_json::to_string(&record).expect("episode serializes"))
}

/// Checked P4 construction configuration, sharing the native fixed rig and validation.
#[wasm_bindgen]
pub fn deception_config_json(lab_json: &str) -> Result<String, JsValue> {
    let config = read_deception_config(lab_json)?;
    Ok(serde_json::to_string(&config).expect("config serializes"))
}

fn read_deception_config(lab_json: &str) -> Result<Config, JsValue> {
    let lab = serde_json::from_str(lab_json)
        .map_err(|error| field_errors(vec![FieldError::new("deception_lab", error.to_string())]))?;
    let config = sugarscape_core::minds::deception::lab::rig_config(lab);
    config.validate().map_err(field_errors)?;
    Ok(config)
}

/// Research-only P4 episode DTO; never a new sender or receiver policy input.
#[wasm_bindgen]
pub fn deception_episode_json(lab_json: &str, seed: &str) -> Result<String, JsValue> {
    let config = read_deception_config(lab_json)?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let record = sugarscape_core::minds::deception::run_episode(
        config.deception_lab.expect("checked lab"),
        seed,
        true,
    )
    .map_err(|failure| field_errors(vec![FieldError::new("episode", failure.message)]))?;
    Ok(serde_json::to_string(&record).expect("episode serializes"))
}

/// Checked food-task construction, sharing the native rig and full validation.
#[wasm_bindgen]
pub fn behavior_tree_config_json(lab_json: &str) -> Result<String, JsValue> {
    let config = read_behavior_tree_config(lab_json)?;
    Ok(serde_json::to_string(&config).expect("config serializes"))
}

fn read_behavior_tree_config(lab_json: &str) -> Result<Config, JsValue> {
    let lab = serde_json::from_str(lab_json).map_err(|error| {
        field_errors(vec![FieldError::new(
            "behavior_tree_lab",
            error.to_string(),
        )])
    })?;
    let config = sugarscape_core::minds::behavior_tree::lab::rig_config(lab);
    config.validate().map_err(field_errors)?;
    Ok(config)
}

/// Research-only episode records from the same core policy as native execution.
#[wasm_bindgen]
pub fn behavior_tree_episode_json(lab_json: &str, seed: &str) -> Result<String, JsValue> {
    let config = read_behavior_tree_config(lab_json)?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let record = sugarscape_core::minds::behavior_tree::runner::run_episode(
        config.behavior_tree_lab.expect("checked lab"),
        seed,
        sugarscape_core::minds::behavior_tree::RunOptions {
            diagnostics: true,
            controller_timing: false,
        },
    )
    .map_err(|failure| field_errors(vec![FieldError::new("episode", failure.message)]))?;
    Ok(serde_json::to_string(&record).expect("episode serializes"))
}

/// Checked standalone excavation replay; serializes the same core record as the native CLI.
#[wasm_bindgen]
pub fn burrow_replay_json(
    config_json: &str,
    seed: &str,
    ticks: f64,
    sample_every: f64,
) -> Result<String, JsValue> {
    let config: sugarscape_core::burrow::LabConfig = serde_json::from_str(config_json)
        .map_err(|error| field_errors(vec![FieldError::new("burrow_config", error.to_string())]))?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let ticks = checked_burrow_u32(ticks, "ticks", 0)?;
    let sample_every = checked_burrow_u32(sample_every, "sample_every", 1)?;
    let episode = sugarscape_core::burrow::run_episode(
        config,
        seed,
        sugarscape_core::burrow::RunOptions {
            ticks,
            sample_every,
        },
    )
    .map_err(field_errors)?;
    Ok(serde_json::to_string(&episode).expect("episode serializes"))
}

/// Checked resource-access replay; returns the complete shared outer record.
#[wasm_bindgen]
pub fn burrow_access_replay_json(
    config_json: &str,
    seed: &str,
    ticks: f64,
    sample_every: f64,
) -> Result<String, JsValue> {
    let config: sugarscape_core::burrow::AccessConfig =
        serde_json::from_str(config_json).map_err(|error| {
            field_errors(vec![FieldError::new(
                "burrow_access_config",
                error.to_string(),
            )])
        })?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let ticks = checked_burrow_u32(ticks, "ticks", 0)?;
    let sample_every = checked_burrow_u32(sample_every, "sample_every", 1)?;
    let episode = sugarscape_core::burrow::run_access_episode(
        config,
        seed,
        sugarscape_core::burrow::RunOptions {
            ticks,
            sample_every,
        },
    )
    .map_err(field_errors)?;
    Ok(serde_json::to_string(&episode).expect("access episode serializes"))
}

fn checked_burrow_u32(value: f64, field: &str, minimum: u32) -> Result<u32, JsValue> {
    if !value.is_finite()
        || value.fract() != 0.0
        || value < f64::from(minimum)
        || value > f64::from(u32::MAX)
    {
        return Err(field_errors(vec![FieldError::new(
            field,
            format!("expected finite integer in {minimum}..={}", u32::MAX),
        )]));
    }
    Ok(value as u32)
}

fn decimal_seed(seed: &str) -> Result<u64, &'static str> {
    if seed.is_empty() || !seed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("expected unsigned decimal u64");
    }
    seed.parse().map_err(|_| "seed exceeds u64")
}

fn read_sweep(spec: &str) -> Result<Sweep, JsValue> {
    Sweep::from_json(spec).map_err(field_errors)
}

/// The sweep in full form (axes written as `{ label, values }`) after
/// parsing it and checking its shape; builds no config. For opening files
/// and links before anything is shown.
#[wasm_bindgen]
pub fn parse_sweep(spec: &str) -> Result<String, JsValue> {
    let sweep = read_sweep(spec)?;
    sweep.check_shape().map_err(field_errors)?;
    Ok(serde_json::to_string(&sweep).expect("sweeps serialize"))
}

/// `runs` (a JSON array of `RunResult`), checked against `sweep`.
fn parse_runs(sweep: &Sweep, runs: &str) -> Result<Vec<RunResult>, JsValue> {
    let runs: Vec<RunResult> = serde_json::from_str(runs)
        .map_err(|e| field_errors(vec![FieldError::new("runs", e.to_string())]))?;
    sweep::check_runs(sweep, &runs).map_err(field_errors)?;
    Ok(runs)
}

/// JSON `[{ index, series, x, seed }]`: every point of the sweep, after
/// checking its shape and every cell's config.
#[wasm_bindgen]
pub fn sweep_points(spec: &str) -> Result<String, JsValue> {
    let points = read_sweep(spec)?.points().map_err(field_errors)?;
    Ok(serde_json::to_string(&points).expect("points serialize"))
}

/// Runs point `index` of the sweep; returns its `RunResult` JSON.
#[wasm_bindgen]
pub fn run_point(spec: &str, index: u32) -> Result<String, JsValue> {
    let sweep = read_sweep(spec)?;
    let point = sweep.point(index as usize).map_err(field_errors)?;
    let config = sweep.config_for(&point).map_err(field_errors)?;
    let run = sweep::run_config(&sweep, &point, config);
    Ok(serde_json::to_string(&run).expect("runs serialize"))
}

/// The `Summary` JSON of `runs` (any order, possibly partial).
#[wasm_bindgen]
pub fn aggregate(spec: &str, runs: &str) -> Result<String, JsValue> {
    let sweep = read_sweep(spec)?;
    let runs = parse_runs(&sweep, runs)?;
    Ok(serde_json::to_string(&sweep::aggregate(&sweep, &runs)).expect("summaries serialize"))
}

/// JSON `[{ id, sweep }]`: the built-in sweep files as written.
#[wasm_bindgen]
pub fn builtin_sweeps() -> String {
    let list: Vec<serde_json::Value> = sweep::builtins()
        .iter()
        .map(|b| {
            let sweep: serde_json::Value =
                serde_json::from_str(b.json).expect("built-in sweeps are JSON");
            serde_json::json!({ "id": b.id, "sweep": sweep })
        })
        .collect();
    serde_json::to_string(&list).expect("sweeps serialize")
}

/// JSON list of the statistics series a config of any model records.
#[wasm_bindgen]
pub fn config_series_names(config: &str) -> Result<String, JsValue> {
    let config = ModelConfig::from_json(config).map_err(field_errors)?;
    Ok(serde_json::to_string(&config.series_names()).expect("names serialize"))
}

/// The CLI's result file for `runs`, marked incomplete when points are missing.
#[wasm_bindgen]
pub fn sweep_result(spec: &str, runs: &str) -> Result<String, JsValue> {
    let sweep = read_sweep(spec)?;
    let runs = parse_runs(&sweep, runs)?;
    Ok(SweepResult::new(sweep, runs).to_json())
}

/// The CLI's runs CSV (`kind = "runs"`) or summary CSV (`"summary"`).
#[wasm_bindgen]
pub fn sweep_csv(spec: &str, runs: &str, kind: &str) -> Result<String, JsValue> {
    let sweep = read_sweep(spec)?;
    let runs = parse_runs(&sweep, runs)?;
    let result = SweepResult::new(sweep, runs);
    match kind {
        "runs" => Ok(sweep::runs_csv(&result)),
        "summary" => Ok(sweep::summary_csv(&result)),
        _ => Err(field_errors(vec![FieldError::new(
            "kind",
            format!("unknown CSV {kind:?} (expected runs or summary)"),
        )])),
    }
}

/// Per-good landscapes from JS (Decision 16): null/undefined → none; a
/// Uint8Array → good 0's (a pre-N-goods share link); an array → one entry per
/// good, each a Uint8Array or null.
fn landscapes_from_js(value: &JsValue) -> Result<Vec<Option<Vec<f64>>>, JsValue> {
    let bytes = |v: &JsValue| -> Vec<f64> {
        js_sys::Uint8Array::new(v)
            .to_vec()
            .into_iter()
            .map(f64::from)
            .collect()
    };
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }
    if value.is_instance_of::<js_sys::Uint8Array>() {
        return Ok(vec![Some(bytes(value))]);
    }
    if js_sys::Array::is_array(value) {
        return Ok(js_sys::Array::from(value)
            .iter()
            .map(|v| {
                if v.is_null() || v.is_undefined() {
                    None
                } else {
                    Some(bytes(&v))
                }
            })
            .collect());
    }
    Err(field_errors(vec![FieldError::new(
        "landscape",
        "expected null, a Uint8Array or an array of Uint8Array | null",
    )]))
}

/// One world of any model (Decision 5): the sugarscape-only calls (maps,
/// editing, trails, networks, the credit graph, the disease list and the
/// wealth views) answer empty (or, for edits, a field error) for the other
/// models, `ring_sugar`/`ring_agents` answer empty for every model but
/// Ring World, and the `anasazi_*` overlays for every model but the
/// anasazi.
#[wasm_bindgen]
pub struct Sim {
    world: ModelWorld,
    frame: Vec<u8>,
    /// `cache_sites`' per-site tally and output, kept between frames.
    cache_scratch: Vec<(f64, u32)>,
    cache_flat: Vec<f64>,
}

/// A keyframe of a `Sim`'s world (`ModelWorld::checkpoint`): the host keeps a few and frees them.
#[wasm_bindgen]
pub struct Checkpoint(sugarscape_core::model::Checkpoint);

const NO_CREDIT_GRAPH: &str = r#"{"agents":[],"loans":[]}"#;

impl Sim {
    fn model(&self) -> &dyn Model {
        self.world.model()
    }

    fn sugar(&self) -> Option<&World> {
        self.world.sugarscape()
    }

    /// The sugarscape world, for an edit; a field error for other models.
    fn sugar_mut(&mut self) -> Result<&mut World, JsValue> {
        self.world
            .sugarscape_mut()
            .ok_or_else(|| edit_error("this world is not a sugarscape".into()))
    }

    fn good(&self, good: u32) -> Result<usize, JsValue> {
        let g = good as usize;
        match self.sugar() {
            Some(w) if g < w.config.goods.len() => Ok(g),
            _ => Err(edit_error(format!("there is no good {good}"))),
        }
    }

    /// `f` of the sugarscape world, or `empty` for other models.
    fn sugar_or<T>(&self, empty: T, f: impl FnOnce(&World) -> T) -> T {
        self.sugar().map_or(empty, f)
    }
}

#[wasm_bindgen]
impl Sim {
    /// A world of the config's model (a sugarscape config without a `model`
    /// key, in either shape). `landscapes` are the sugarscape's painted maps;
    /// other models ignore them.
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str, seed: u32, landscapes: JsValue) -> Result<Sim, JsValue> {
        let config = ModelConfig::from_json(config_json).map_err(field_errors)?;
        let landscapes = landscapes_from_js(&landscapes)?;
        let world = ModelWorld::with_landscapes(config, u64::from(seed), &landscapes)
            .map_err(field_errors)?;
        Ok(Sim {
            world,
            frame: Vec::new(),
            cache_scratch: Vec::new(),
            cache_flat: Vec::new(),
        })
    }

    /// The world's model, as a config's `model` key names it
    /// (`"sugarscape"`, `"schelling"`, …, `"hoard"`; `ModelKind::as_str`).
    pub fn model_kind(&self) -> String {
        self.world.kind().as_str().to_string()
    }

    pub fn step(&mut self, n: u32) {
        self.world.model_mut().run(n);
    }

    pub fn tick(&self) -> f64 {
        self.model().tick() as f64
    }

    /// Whether the world has run its course (the anasazi's end year);
    /// stepping it then does nothing.
    pub fn finished(&self) -> bool {
        self.model().finished()
    }

    /// The frame's width in cells (Ring World: its sites).
    pub fn width(&self) -> u32 {
        self.model().size().0
    }

    /// The frame's height in cells (Ring World: the space–time diagram's rows).
    pub fn height(&self) -> u32 {
        self.model().size().1
    }

    pub fn population(&self) -> u32 {
        self.model().population() as u32
    }

    /// Renders into the internal frame and returns a pointer into WASM memory.
    /// Re-create any JS view after each call: memory may have grown.
    pub fn render(&mut self, color_mode: &str, layer: &str) -> Result<usize, JsValue> {
        let Sim { world, frame, .. } = self;
        world
            .model()
            .render(color_mode, layer, frame)
            .map_err(edit_error)?;
        Ok(frame.as_ptr() as usize)
    }

    pub fn frame_len(&self) -> usize {
        self.frame.len()
    }

    pub fn stats_latest(&self) -> String {
        self.model().latest_json()
    }

    pub fn series(&self, name: &str) -> Result<Vec<f64>, JsValue> {
        self.model()
            .series(name)
            .ok_or_else(|| edit_error(format!("unknown series {name:?}")))
    }

    /// The latest value of series `name` (or `"tick"`), or `undefined`.
    pub fn latest_value(&self, name: &str) -> Option<f64> {
        self.model().latest_value(name)
    }

    /// A keyframe of the world now, or `undefined` for a model without them.
    pub fn checkpoint(&mut self) -> Option<Checkpoint> {
        self.world.checkpoint().map(Checkpoint)
    }

    /// Returns the world to `cp` (see `ModelWorld::restore`); throws a field error if it cannot.
    pub fn restore(&mut self, cp: &Checkpoint) -> Result<(), JsValue> {
        self.world.restore(&cp.0).map_err(edit_error)
    }

    /// Series `name` cut to at most `max` points (`stats::downsample`) as
    /// `[tick, value, tick, value, …]`.
    pub fn series_downsampled(&self, name: &str, max: u32) -> Result<Vec<f64>, JsValue> {
        let values = self.series(name)?;
        let ticks = self.model().series("tick").unwrap_or_default();
        Ok(stats::downsample(&values, max as usize)
            .into_iter()
            .flat_map(|(i, v)| [ticks[i as usize], v])
            .collect())
    }

    /// Several series on one x axis (`stats::downsample_union`: each keeps
    /// at most `max` points of its own shape) as `[n, ticks (n), then n
    /// values per name]`. `names_json` is a JSON array of series names.
    pub fn series_group(&self, names_json: &str, max: u32) -> Result<Vec<f64>, JsValue> {
        let names: Vec<String> =
            serde_json::from_str(names_json).map_err(|e| edit_error(e.to_string()))?;
        let columns = names
            .iter()
            .map(|name| self.series(name))
            .collect::<Result<Vec<_>, _>>()?;
        let ticks = self.model().series("tick").unwrap_or_default();
        let keep = stats::downsample_union(&columns, max as usize);
        let mut out = Vec::with_capacity(1 + keep.len() * (1 + columns.len()));
        out.push(keep.len() as f64);
        out.extend(keep.iter().map(|&i| ticks[i]));
        for column in &columns {
            out.extend(keep.iter().map(|&i| column[i]));
        }
        Ok(out)
    }

    /// The model's fingerprint as `0x…` hex, the golden tests' format (see [`fingerprint_hex`]).
    pub fn fingerprint(&self) -> String {
        fingerprint_hex(self.model().fingerprint())
    }

    pub fn lorenz(&self, points: usize) -> Vec<f64> {
        self.sugar_or(Vec::new(), |w| {
            stats::lorenz(&stats::wealths(w), points.max(2))
        })
    }

    /// `[bin_width, count_0, …, count_{bins-1}]`.
    pub fn wealth_hist(&self, bins: usize) -> Vec<f64> {
        self.sugar_or(Vec::new(), |w| {
            let (width, counts) = stats::histogram(&stats::wealths(w), bins.max(1));
            std::iter::once(width).chain(counts).collect()
        })
    }

    /// `[bin_width, count_0, …, count_{bins-1}]` of good `good`'s holdings (the wealth
    /// histogram's bins, for any good).
    pub fn good_wealth_hist(&self, good: u32, bins: usize) -> Result<Vec<f64>, JsValue> {
        let g = self.good(good)?;
        let w = self.sugar().expect("goods exist only in a sugarscape");
        let (width, counts) = stats::histogram(&stats::good_wealths(w, g), bins.max(1));
        Ok(std::iter::once(width).chain(counts).collect())
    }

    /// The Lorenz curve of total wealth (every good's holdings summed).
    pub fn lorenz_total(&self, points: usize) -> Vec<f64> {
        self.sugar_or(Vec::new(), |w| {
            stats::lorenz(&stats::total_wealths(w), points.max(2))
        })
    }

    /// `[bin, count_0, …]`: living agents' ages in `bin`-tick bins (`stats::age_histogram`).
    pub fn age_hist(&self, bin: u32) -> Vec<f64> {
        let bin = bin.max(1);
        self.sugar_or(Vec::new(), |w| {
            std::iter::once(f64::from(bin))
                .chain(stats::age_histogram(w, bin))
                .collect()
        })
    }

    /// The percentage of living agents with a 0 at each tag position, position 0 first
    /// (`stats::tag_histogram`).
    pub fn tag_hist(&self) -> Vec<f64> {
        self.sugar_or(Vec::new(), stats::tag_histogram)
    }

    /// The site (x, y) and its agent, in the model's own JSON shape (Ring
    /// World: site `x`, whatever `y`).
    pub fn inspect(&self, x: u32, y: u32) -> Result<String, JsValue> {
        self.model().inspect_json(x, y).map_err(edit_error)
    }

    /// The agent at `(x, y)`'s remembered sites (Minds 3; `World::memory_view`),
    /// flattened as `[x, y, age, spot]` per site, in memory order. Empty for
    /// no agent there, one that doesn't remember, memory off, or a
    /// non-sugarscape model.
    pub fn inspect_memory(&self, x: u32, y: u32) -> Vec<u32> {
        self.sugar_or(Vec::new(), |w| {
            w.memory_view(Pos::new(x, y))
                .into_iter()
                .flatten()
                .collect()
        })
    }

    /// Minds 5–6: JSON `{ winter, homes, lab }` (`World::minds_view`): the
    /// season, every home and larder and the lab's schedule. `null` for a
    /// non-sugarscape model.
    pub fn minds_view(&self) -> String {
        self.sugar_or("null".into(), |w| {
            serde_json::to_string(&w.minds_view()).expect("views serialize")
        })
    }

    /// Minds 5–6: every site holding a cache, `[x, y, total, flags, …]`
    /// (`World::cache_sites`), built in buffers the `Sim` keeps; empty for a
    /// non-sugarscape model.
    pub fn cache_sites(&mut self) -> js_sys::Float64Array {
        let Sim {
            world,
            cache_scratch,
            cache_flat,
            ..
        } = self;
        match world.sugarscape() {
            Some(w) => {
                w.cache_sites(cache_scratch, cache_flat);
                js_sys::Float64Array::from(&cache_flat[..])
            }
            None => js_sys::Float64Array::new_with_length(0),
        }
    }

    pub fn locate(&self, id: f64) -> Option<Vec<u32>> {
        self.model().locate(id as u64).map(|(x, y)| vec![x, y])
    }

    /// Records agent `id`'s trail from now on (`World::follow`; sugarscape only).
    pub fn follow(&mut self, id: f64) {
        if let Some(w) = self.world.sugarscape_mut() {
            w.follow(Some(id as u64));
        }
    }

    pub fn unfollow(&mut self) {
        if let Some(w) = self.world.sugarscape_mut() {
            w.follow(None);
        }
    }

    /// The followed agent's trail as `[x0, y0, x1, y1, …]`, oldest first.
    pub fn trail(&self) -> Vec<u32> {
        self.sugar_or(Vec::new(), |w| {
            w.trail().iter().flat_map(|p| [p.x, p.y]).collect()
        })
    }

    /// The followed agent's id, or −1 when none is followed.
    pub fn followed(&self) -> f64 {
        self.sugar()
            .and_then(World::followed)
            .map_or(-1.0, |id| id as f64)
    }

    pub fn paint_capacity(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        value: f64,
        good: u32,
    ) -> Result<(), JsValue> {
        self.sugar_mut()?
            .paint_capacity(x, y, radius, value, good as usize)
            .map_err(edit_error)
    }

    /// Replaces good `good`'s capacities with `capacities` (row-major bytes,
    /// each 0–10): an imported image. The good then counts as painted.
    pub fn set_landscape(&mut self, good: u32, capacities: &[u8]) -> Result<(), JsValue> {
        let g = self.good(good)?;
        let caps: Vec<f64> = capacities.iter().map(|&c| f64::from(c)).collect();
        self.sugar_mut()?
            .set_capacities(g, &caps)
            .map_err(edit_error)
    }

    pub fn place_agent(&mut self, x: u32, y: u32, overrides_json: &str) -> Result<f64, JsValue> {
        let overrides: AgentOverrides =
            serde_json::from_str(overrides_json).map_err(|e| edit_error(e.to_string()))?;
        self.sugar_mut()?
            .place_agent(x, y, &overrides)
            .map(|id| id as f64)
            .map_err(edit_error)
    }

    pub fn remove_agent(&mut self, x: u32, y: u32) -> Result<(), JsValue> {
        self.sugar_mut()?.remove_agent(x, y).map_err(edit_error)
    }

    /// Applies a changed config of the world's model to the running world.
    pub fn set_config(&mut self, json: &str) -> Result<(), JsValue> {
        let config = ModelConfig::from_json(json).map_err(field_errors)?;
        self.world
            .model_mut()
            .set_config(config)
            .map_err(field_errors)
    }

    /// The live config, tagged with its model unless it is a sugarscape's.
    pub fn export_config(&self) -> String {
        serde_json::to_string(&self.model().config()).expect("config serializes")
    }

    /// Good `good`'s capacities rounded to bytes, row-major.
    pub fn export_landscape(&self, good: u32) -> Result<Vec<u8>, JsValue> {
        let g = self.good(good)?;
        let w = self.sugar().expect("goods exist only in a sugarscape");
        Ok(w.capacities(g)
            .into_iter()
            .map(|c| c.round().clamp(0.0, 255.0) as u8)
            .collect())
    }

    /// Whether good `good`'s capacities differ from its generated map.
    pub fn landscape_edited(&self, good: u32) -> bool {
        self.sugar_or(false, |w| w.landscape_edited(good as usize))
    }

    /// JSON list of this world's series names (a sugarscape's depend on its
    /// goods, pollutants and groups).
    pub fn series_names(&self) -> String {
        serde_json::to_string(&self.model().series_names()).expect("names serialize")
    }

    /// Full finite state, science availability reasons, and clocks for democratic peace.
    pub fn export_model_json(&self) -> Result<String, JsValue> {
        match &self.world {
            ModelWorld::DemocraticPeace(world) => Ok(world.state_json()),
            _ => Err(edit_error(
                "full model JSON is available for democratic_peace".into(),
            )),
        }
    }

    pub fn export_series_csv(&self) -> String {
        self.model().series_csv()
    }

    pub fn export_agents_csv(&self) -> String {
        self.model().agents_csv()
    }

    /// Edges as `[x1, y1, x2, y2, …]` for `"trade"` (this tick), `"credit"` (outstanding),
    /// `"disease"` (infector → infected, this tick), `"neighbors"` (agent → each agent on its
    /// neighbor list), `"friends"` (agent → friend) or `"family"` (parent → child); none in
    /// other models.
    pub fn networks(&self, kind: &str) -> Result<Vec<u32>, JsValue> {
        let Some(w) = self.sugar() else {
            return Ok(Vec::new());
        };
        let edges = match kind {
            "trade" => network::trade_edges(w),
            "credit" => network::credit_edges(w),
            "disease" => network::disease_edges(w),
            "neighbors" => w.neighbor_edges(),
            "friends" => w.friend_edges(),
            "family" => w.family_edges(),
            _ => return Err(edit_error(format!("unknown network {kind:?}"))),
        };
        Ok(edges
            .into_iter()
            .flat_map(|(a, b)| [a.x, a.y, b.x, b.y])
            .collect())
    }

    /// JSON `{ agents: [{ id, role }], loans: [{ lender, borrower, good, due }] }`
    /// over the outstanding loans.
    pub fn credit_graph(&self) -> String {
        self.sugar_or(NO_CREDIT_GRAPH.into(), |w| {
            serde_json::to_string(&network::credit_graph(w)).expect("graph serializes")
        })
    }

    /// JSON `[{ id, bits, carriers }]`.
    pub fn disease_list(&self) -> String {
        self.sugar_or("[]".into(), |w| {
            serde_json::to_string(&w.disease_list()).expect("list serializes")
        })
    }

    /// Infects the agent at (x, y) with `disease` (−1 = a brand-new random
    /// disease). Returns whether it was infected.
    pub fn infect(&mut self, x: u32, y: u32, disease: i32) -> Result<bool, JsValue> {
        self.sugar_mut()?
            .infect(x, y, i64::from(disease))
            .map_err(edit_error)
    }

    /// Vaccinates every agent within `radius` of (x, y) against `disease`.
    /// Returns how many agents were vaccinated.
    pub fn vaccinate(&mut self, x: u32, y: u32, radius: u32, disease: u32) -> Result<u32, JsValue> {
        self.sugar_mut()?
            .vaccinate(x, y, radius, disease)
            .map_err(edit_error)
    }

    /// `[n, prices(n), demand(n), supply(n), eq_price, eq_quantity, actual_price, actual_quantity]`.
    pub fn supply_demand(&self) -> Vec<f64> {
        self.sugar_or(Vec::new(), |w| {
            let sd = stats::supply_demand(w);
            let mut out = vec![sd.prices.len() as f64];
            out.extend(sd.prices);
            out.extend(sd.demand);
            out.extend(sd.supply);
            out.extend([
                sd.equilibrium_price,
                sd.equilibrium_quantity,
                sd.actual_price,
                sd.actual_quantity,
            ]);
            out
        })
    }

    /// Ring World's sugar per site, site 0 first (empty for other models).
    pub fn ring_sugar(&self) -> Vec<f64> {
        self.world.ring().map_or(Vec::new(), |r| r.sugar().to_vec())
    }

    /// Ring World's agents' sites, in id order (empty for other models).
    pub fn ring_agents(&self) -> Vec<u32> {
        self.world.ring().map_or(Vec::new(), |r| r.agent_sites())
    }

    /// The anasazi's water sources now, `[x, y, …]` (empty for other models).
    pub fn anasazi_water(&self) -> Vec<u32> {
        self.world.anasazi().map_or(Vec::new(), |a| a.water_xy())
    }

    /// The anasazi's inhabited cells, `[x, y, households, …]`.
    pub fn anasazi_settlements(&self) -> Vec<u32> {
        self.world
            .anasazi()
            .map_or(Vec::new(), |a| a.settlements_xy())
    }

    /// The anasazi's farm–home links, `[farm x, farm y, home x, home y, …]`.
    pub fn anasazi_links(&self) -> Vec<u32> {
        self.world.anasazi().map_or(Vec::new(), |a| a.links_xy())
    }

    /// The hoard world's 20 agents as a JSON array (the population panel):
    /// index, traits, stores, state, raid target and this season's losses.
    /// `[]` for other models.
    pub fn hoard_population(&self) -> String {
        self.world.hoard().map_or("[]".into(), |h| {
            serde_json::to_string(&h.population_views()).expect("agents serialize")
        })
    }

    /// Where the hoard run is, as JSON `{generation, day, bout, public,
    /// season_over, living}` (`day` and `bout` are the next to run, both
    /// 1-based); `null` for other models.
    pub fn hoard_status(&self) -> String {
        self.world.hoard().map_or("null".into(), |h| {
            serde_json::json!({
                "generation": h.generation(),
                "day": h.day(),
                "bout": h.bout(),
                "public": h.public(),
                "season_over": h.season_over(),
                "living": h.living(),
            })
            .to_string()
        })
    }

    /// The current season's per-bout values of a hoard series as `[tick,
    /// value, tick, value, …]` (from the season's first bout). Empty for
    /// other models and unknown names.
    pub fn hoard_season_series(&self, name: &str) -> Vec<f64> {
        self.world
            .hoard()
            .and_then(|h| h.season_series(name))
            .unwrap_or_default()
    }

    /// One value per finished season for a hoard generation series
    /// (`generation`, `mean_larder_prob`, `hoarder_larder_prob`,
    /// `mean_defense`, `survivors`, `larder_share`, `larder_loss_rate`,
    /// `scatter_loss_rate`; NaN where a season has none). Empty for other
    /// models and unknown names.
    pub fn hoard_generation_series(&self, name: &str) -> Vec<f64> {
        self.world
            .hoard()
            .and_then(|h| h.generation_series(name))
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod protection_boundary_tests {
    #[test]
    fn decimal_seed_is_strict_and_preserves_full_u64_range() {
        assert_eq!(super::decimal_seed("18446744073709551615"), Ok(u64::MAX));
        assert_eq!(super::decimal_seed("0007"), Ok(7));
        for seed in ["", "-1", "+7", "7.0", " 7", "7\n", "18446744073709551616"] {
            assert!(super::decimal_seed(seed).is_err(), "{seed:?}");
        }
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod deception_boundary_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    const LAB: &str = r#"{"sender":"sham","view":"ambiguous","display_seen":true,"layout":"off_route","effort_cost":3,"mirrored":false}"#;

    #[wasm_bindgen_test]
    fn deception_construction_serializes_the_complete_research_dto() {
        let config: Config = serde_json::from_str(&deception_config_json(LAB).unwrap()).unwrap();
        assert_eq!(config.deception_lab.as_ref().unwrap().effort_cost, 3.0);
        let episode: sugarscape_core::minds::deception::EpisodeRecord =
            serde_json::from_str(&deception_episode_json(LAB, "7").unwrap()).unwrap();
        assert_eq!(
            episode.frames.iter().map(|f| f.tick).collect::<Vec<_>>(),
            (0..=64).collect::<Vec<_>>()
        );
        assert!(episode.fixture_errors.is_empty() && episode.ledger_errors.is_empty());
    }

    #[wasm_bindgen_test]
    fn deception_boundary_errors_preserve_field_context() {
        for seed in ["", "-1", "+7", "7.0", " 7", "18446744073709551616"] {
            let error = deception_episode_json(LAB, seed)
                .unwrap_err()
                .as_string()
                .unwrap();
            let errors: Vec<serde_json::Value> = serde_json::from_str(&error).unwrap();
            assert_eq!(errors[0]["field"], "seed");
        }
        for input in ["{", r#"{"private_stock":12}"#] {
            let error = deception_config_json(input)
                .unwrap_err()
                .as_string()
                .unwrap();
            let errors: Vec<serde_json::Value> = serde_json::from_str(&error).unwrap();
            assert_eq!(errors[0]["field"], "deception_lab");
        }
        let error = deception_config_json(&LAB.replace("\"effort_cost\":3", "\"effort_cost\":1"))
            .unwrap_err()
            .as_string()
            .unwrap();
        let errors: Vec<serde_json::Value> = serde_json::from_str(&error).unwrap();
        assert!(errors
            .iter()
            .any(|e| e["field"] == "deception_lab.effort_cost"));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod behavior_tree_boundary_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    const LAB: &str = r#"{"controller":"guarded_tree","scenario":"better_alternative","quota":40,"mirrored":false}"#;

    #[wasm_bindgen_test]
    fn behavior_tree_checked_exports_use_canonical_core_records() {
        let config: Config =
            serde_json::from_str(&behavior_tree_config_json(LAB).unwrap()).unwrap();
        config.validate().unwrap();
        let episode: sugarscape_core::minds::behavior_tree::EpisodeRecord =
            serde_json::from_str(&behavior_tree_episode_json(LAB, "7").unwrap()).unwrap();
        assert_eq!(episode.frames.len(), 65);
        assert!(episode.errors.is_empty());
        assert!(episode
            .frames
            .iter()
            .all(|f| f.controller_seconds.is_none()));
    }

    #[wasm_bindgen_test]
    fn behavior_tree_boundary_rejects_unknown_fields_and_invalid_quota() {
        for input in [
            "{",
            &LAB.replace("40", "21"),
            &LAB.replace("false}", "false,\"extra\":1}"),
        ] {
            for error in [
                behavior_tree_config_json(input).unwrap_err(),
                behavior_tree_episode_json(input, "7").unwrap_err(),
            ] {
                let errors: Vec<serde_json::Value> =
                    serde_json::from_str(&error.as_string().unwrap()).unwrap();
                assert!(errors.iter().any(|e| e["field"]
                    .as_str()
                    .is_some_and(|f| f.starts_with("behavior_tree_lab"))));
            }
        }
        for seed in ["", "-1", "+7", "7.0", " 7", "7\n", "18446744073709551616"] {
            let error = behavior_tree_episode_json(LAB, seed).unwrap_err();
            let errors: Vec<serde_json::Value> =
                serde_json::from_str(&error.as_string().unwrap()).unwrap();
            assert_eq!(errors[0]["field"], "seed");
        }
    }

    #[wasm_bindgen_test]
    fn behavior_tree_decimal_seed_parser_accepts_full_u64_without_construction() {
        assert_eq!(decimal_seed("18446744073709551615"), Ok(u64::MAX));
        assert_eq!(decimal_seed("0007"), Ok(7));
    }
}
