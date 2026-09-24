//! JavaScript-facing wrapper around the Sugarscape core. Errors cross the
//! boundary as JSON strings of `[{ field, message }]`.

use sugarscape_core::config::{Config, FieldError};
use sugarscape_core::edit::AgentOverrides;
use sugarscape_core::render::{self, ColorMode, Layer};
use sugarscape_core::sweep::{RunResult, Sweep, SweepResult};
use sugarscape_core::world::World;
use sugarscape_core::{export, network, presets, stats, sweep};
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

#[wasm_bindgen]
pub fn presets_json() -> String {
    serde_json::to_string(&presets::all()).expect("presets serialize")
}

#[wasm_bindgen]
pub fn default_config_json() -> String {
    serde_json::to_string(&Config::default()).expect("config serializes")
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

/// JSON list of the statistics series a config (either shape) records.
#[wasm_bindgen]
pub fn config_series_names(config: &str) -> Result<String, JsValue> {
    let config = Config::from_json(config).map_err(field_errors)?;
    Ok(serde_json::to_string(&stats::series_names(&config)).expect("names serialize"))
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

#[wasm_bindgen]
pub struct Sim {
    world: World,
    frame: Vec<u8>,
}

impl Sim {
    fn good(&self, good: u32) -> Result<usize, JsValue> {
        let g = good as usize;
        if g < self.world.config.goods.len() {
            Ok(g)
        } else {
            Err(edit_error(format!("there is no good {good}")))
        }
    }
}

#[wasm_bindgen]
impl Sim {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str, seed: u32, landscapes: JsValue) -> Result<Sim, JsValue> {
        let config = Config::from_json(config_json).map_err(field_errors)?;
        let landscapes = landscapes_from_js(&landscapes)?;
        let world =
            World::with_landscapes(config, u64::from(seed), &landscapes).map_err(field_errors)?;
        Ok(Sim {
            world,
            frame: Vec::new(),
        })
    }

    pub fn step(&mut self, n: u32) {
        self.world.run(n);
    }

    pub fn tick(&self) -> f64 {
        self.world.tick as f64
    }

    pub fn width(&self) -> u32 {
        self.world.torus.width
    }

    pub fn height(&self) -> u32 {
        self.world.torus.height
    }

    pub fn population(&self) -> u32 {
        self.world.population() as u32
    }

    /// Renders into the internal frame and returns a pointer into WASM memory.
    /// Re-create any JS view after each call: memory may have grown.
    pub fn render(&mut self, color_mode: &str, layer: &str) -> Result<usize, JsValue> {
        let mode: ColorMode = color_mode.parse().map_err(edit_error)?;
        let layer: Layer = layer.parse().map_err(edit_error)?;
        render::render(&self.world, mode, layer, &mut self.frame).map_err(edit_error)?;
        Ok(self.frame.as_ptr() as usize)
    }

    pub fn frame_len(&self) -> usize {
        self.frame.len()
    }

    pub fn stats_latest(&self) -> String {
        serde_json::to_string(&self.world.stats.latest()).expect("snapshot serializes")
    }

    pub fn series(&self, name: &str) -> Result<Vec<f64>, JsValue> {
        self.world
            .stats
            .series(name)
            .ok_or_else(|| edit_error(format!("unknown series {name:?}")))
    }

    pub fn lorenz(&self, points: usize) -> Vec<f64> {
        stats::lorenz(&stats::wealths(&self.world), points.max(2))
    }

    /// `[bin_width, count_0, …, count_{bins-1}]`.
    pub fn wealth_hist(&self, bins: usize) -> Vec<f64> {
        let (width, counts) = stats::histogram(&stats::wealths(&self.world), bins.max(1));
        std::iter::once(width).chain(counts).collect()
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<String, JsValue> {
        let inspection = self.world.inspect(x, y).map_err(edit_error)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    pub fn locate(&self, id: f64) -> Option<Vec<u32>> {
        self.world.locate(id as u64).map(|p| vec![p.x, p.y])
    }

    pub fn paint_capacity(
        &mut self,
        x: u32,
        y: u32,
        radius: u32,
        value: f64,
        good: u32,
    ) -> Result<(), JsValue> {
        self.world
            .paint_capacity(x, y, radius, value, good as usize)
            .map_err(edit_error)
    }

    pub fn place_agent(&mut self, x: u32, y: u32, overrides_json: &str) -> Result<f64, JsValue> {
        let overrides: AgentOverrides =
            serde_json::from_str(overrides_json).map_err(|e| edit_error(e.to_string()))?;
        self.world
            .place_agent(x, y, &overrides)
            .map(|id| id as f64)
            .map_err(edit_error)
    }

    pub fn remove_agent(&mut self, x: u32, y: u32) -> Result<(), JsValue> {
        self.world.remove_agent(x, y).map_err(edit_error)
    }

    pub fn set_config(&mut self, json: &str) -> Result<(), JsValue> {
        let config = Config::from_json(json).map_err(field_errors)?;
        self.world.set_config(config).map_err(field_errors)
    }

    pub fn export_config(&self) -> String {
        serde_json::to_string(&self.world.config).expect("config serializes")
    }

    /// Good `good`'s capacities rounded to bytes, row-major.
    pub fn export_landscape(&self, good: u32) -> Result<Vec<u8>, JsValue> {
        let g = self.good(good)?;
        Ok(self
            .world
            .capacities(g)
            .into_iter()
            .map(|c| c.round().clamp(0.0, 255.0) as u8)
            .collect())
    }

    /// Whether good `good`'s capacities differ from its generated map.
    pub fn landscape_edited(&self, good: u32) -> bool {
        self.world.landscape_edited(good as usize)
    }

    /// JSON list of this world's series names (they depend on its goods and
    /// pollutants).
    pub fn series_names(&self) -> String {
        serde_json::to_string(&stats::series_names(&self.world.config)).expect("names serialize")
    }

    pub fn export_series_csv(&self) -> String {
        export::series_csv(&self.world)
    }

    pub fn export_agents_csv(&self) -> String {
        export::agents_csv(&self.world)
    }

    /// Edges as `[x1, y1, x2, y2, …]` for `"trade"` (this tick), `"credit"` (outstanding) or `"disease"` (infector → infected, this tick).
    pub fn networks(&self, kind: &str) -> Result<Vec<u32>, JsValue> {
        let edges = match kind {
            "trade" => network::trade_edges(&self.world),
            "credit" => network::credit_edges(&self.world),
            "disease" => network::disease_edges(&self.world),
            _ => return Err(edit_error(format!("unknown network {kind:?}"))),
        };
        Ok(edges
            .into_iter()
            .flat_map(|(a, b)| [a.x, a.y, b.x, b.y])
            .collect())
    }

    /// JSON `[{ id, bits, carriers }]`.
    pub fn disease_list(&self) -> String {
        serde_json::to_string(&self.world.disease_list()).expect("list serializes")
    }

    /// Infects the agent at (x, y) with `disease` (−1 = a brand-new random
    /// disease). Returns whether it was infected.
    pub fn infect(&mut self, x: u32, y: u32, disease: i32) -> Result<bool, JsValue> {
        self.world
            .infect(x, y, i64::from(disease))
            .map_err(edit_error)
    }

    /// Vaccinates every agent within `radius` of (x, y) against `disease`.
    /// Returns how many agents were vaccinated.
    pub fn vaccinate(&mut self, x: u32, y: u32, radius: u32, disease: u32) -> Result<u32, JsValue> {
        self.world
            .vaccinate(x, y, radius, disease)
            .map_err(edit_error)
    }

    /// `[n, prices(n), demand(n), supply(n), eq_price, eq_quantity, actual_price, actual_quantity]`.
    pub fn supply_demand(&self) -> Vec<f64> {
        let sd = stats::supply_demand(&self.world);
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
    }
}
