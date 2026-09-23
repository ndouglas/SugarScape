//! JavaScript-facing wrapper around the Sugarscape core. Errors cross the
//! boundary as JSON strings of `[{ field, message }]`.

use sugarscape_core::config::{Config, FieldError};
use sugarscape_core::edit::AgentOverrides;
use sugarscape_core::render::{self, ColorMode, Layer};
use sugarscape_core::world::World;
use sugarscape_core::{export, network, presets, stats};
use wasm_bindgen::prelude::*;

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

#[wasm_bindgen]
pub fn series_names_json() -> String {
    serde_json::to_string(&stats::SERIES).expect("names serialize")
}

#[wasm_bindgen]
pub struct Sim {
    world: World,
    frame: Vec<u8>,
}

#[wasm_bindgen]
impl Sim {
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: &str, seed: u32, capacities: Option<Vec<u8>>) -> Result<Sim, JsValue> {
        let config = Config::from_json(config_json).map_err(field_errors)?;
        let caps: Option<Vec<f64>> = capacities.map(|c| c.into_iter().map(f64::from).collect());
        let world = World::with_capacities(config, u64::from(seed), caps.as_deref())
            .map_err(field_errors)?;
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
        render::render(&self.world, mode, layer, &mut self.frame);
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
    ) -> Result<(), JsValue> {
        self.world
            .paint_capacity(x, y, radius, value, 0)
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

    /// Capacities rounded to bytes, row-major.
    pub fn export_landscape(&self) -> Vec<u8> {
        self.world
            .capacities(0)
            .into_iter()
            .map(|c| c.round().clamp(0.0, 255.0) as u8)
            .collect()
    }

    pub fn landscape_edited(&self) -> bool {
        self.world.landscape_edited(0)
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
