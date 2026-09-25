//! Model kinds (milestone 9): the sugarscape and the other artificial
//! societies of Chapter VI behind one config type and one trait, so the
//! worker host, sweeps and the CLI run any of them the same way. See
//! docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md.

use serde::{Serialize, Serializer};

use crate::config::{Config, FieldError};
use crate::render::{self, ColorMode, Layer};
use crate::world::World;
use crate::{export, stats};

/// Which model a config or world is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Sugarscape,
}

impl ModelKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ModelKind::Sugarscape => "sugarscape",
        }
    }
}

/// A config of any model. On the wire it is the model's own config object;
/// every model but the sugarscape carries `"model": "<kind>"`, and an object
/// without a `model` key (every config, link, session and sweep written
/// before milestone 9) is a sugarscape config.
#[derive(Clone, Debug, PartialEq)]
pub enum ModelConfig {
    Sugarscape(Config),
}

impl From<Config> for ModelConfig {
    fn from(c: Config) -> Self {
        ModelConfig::Sugarscape(c)
    }
}

impl Serialize for ModelConfig {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            // Untagged, so sugarscape configs serialize exactly as before.
            ModelConfig::Sugarscape(c) => c.serialize(s),
        }
    }
}

impl ModelConfig {
    pub fn kind(&self) -> ModelKind {
        match self {
            ModelConfig::Sugarscape(_) => ModelKind::Sugarscape,
        }
    }

    /// The sugarscape config, if this is one.
    pub fn sugarscape(&self) -> Option<&Config> {
        match self {
            ModelConfig::Sugarscape(c) => Some(c),
        }
    }

    /// Parses and validates a config of any model.
    pub fn from_json(json: &str) -> Result<Self, Vec<FieldError>> {
        let value: serde_json::Value = serde_json::from_str(json)
            .map_err(|e| vec![FieldError::new("config", e.to_string())])?;
        let config = Self::from_value(value).map_err(|e| vec![e])?;
        config.validate()?;
        Ok(config)
    }

    /// Reads a config of any model by its `model` key: absent or
    /// `"sugarscape"` is a sugarscape config in either shape
    /// (`Config::from_value`).
    pub fn from_value(mut value: serde_json::Value) -> Result<Self, FieldError> {
        let tag = match value.as_object_mut().and_then(|o| o.remove("model")) {
            None => "sugarscape".to_string(),
            Some(serde_json::Value::String(tag)) => tag,
            Some(other) => {
                return Err(FieldError::new(
                    "model",
                    format!("must be a model name, not {other}"),
                ))
            }
        };
        match tag.as_str() {
            "sugarscape" => Config::from_value(value).map(ModelConfig::Sugarscape),
            _ => Err(FieldError::new(
                "model",
                format!("unknown model {tag:?} (expected sugarscape)"),
            )),
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        match self {
            ModelConfig::Sugarscape(c) => c.validate(),
        }
    }

    /// A copy with one dotted `path` set to `value` (the model cannot be
    /// changed this way: `model` is not a field of any model's config).
    pub fn with_path(&self, path: &str, value: &serde_json::Value) -> Result<Self, FieldError> {
        match self {
            ModelConfig::Sugarscape(c) => c.with_path(path, value).map(ModelConfig::Sugarscape),
        }
    }

    /// The statistics series a world with this config records.
    pub fn series_names(&self) -> Vec<String> {
        match self {
            ModelConfig::Sugarscape(c) => stats::series_names(c),
        }
    }
}

/// What the host, sweeps and the CLI need from a running model. `run` is
/// the spec's `step(n)`, named after `World::run` so it cannot be mistaken
/// for `World::step` (one tick).
pub trait Model {
    /// The live config (after scheduled changes and `set_config`).
    fn config(&self) -> ModelConfig;
    /// Runs `ticks` ticks.
    fn run(&mut self, ticks: u32);
    /// Completed ticks.
    fn tick(&self) -> u64;
    fn population(&self) -> usize;
    /// A hash of the full dynamic state (the golden tests' fingerprint).
    fn fingerprint(&self) -> u64;
    /// The rendered frame's width and height in cells.
    fn size(&self) -> (u32, u32);
    /// Renders the frame as RGBA into `buf` (`size()` cells). `mode` and
    /// `layer` name the model's color mode and landscape layer; models
    /// without them ignore them.
    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String>;
    /// The latest statistics snapshot as JSON.
    fn latest_json(&self) -> String;
    fn series_names(&self) -> Vec<String>;
    /// The full history of series `name` (or `"tick"`), or `None` if unknown.
    fn series(&self, name: &str) -> Option<Vec<f64>>;
    /// The statistics history as CSV (`tick`, then `series_names`).
    fn series_csv(&self) -> String;
    /// The agents alive now as CSV.
    fn agents_csv(&self) -> String;
    /// The site (x, y) and its agent as JSON.
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String>;
    /// Where agent `id` is, in frame cells, while it lives.
    fn locate(&self, id: u64) -> Option<(u32, u32)>;
    /// Applies a changed config to the running world; fields that change
    /// only on reset are refused.
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>>;
}

impl Model for World {
    fn config(&self) -> ModelConfig {
        ModelConfig::Sugarscape(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        World::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        World::population(self)
    }

    fn fingerprint(&self) -> u64 {
        World::fingerprint(self)
    }

    fn size(&self) -> (u32, u32) {
        (self.torus.width, self.torus.height)
    }

    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ColorMode = mode.parse()?;
        let layer: Layer = layer.parse()?;
        render::render(self, mode, layer, buf)
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        stats::series_names(&self.config)
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::series_csv(self)
    }

    fn agents_csv(&self) -> String {
        export::agents_csv(self)
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        World::locate(self, id).map(|p| (p.x, p.y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        match next {
            ModelConfig::Sugarscape(c) => World::set_config(self, c),
        }
    }
}

/// A world of any model. Sugarscape-only calls (painting, networks, the
/// credit graph …) go through `sugarscape()`/`sugarscape_mut()`.
pub enum ModelWorld {
    Sugarscape(Box<World>),
}

impl ModelWorld {
    pub fn new(config: ModelConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        Self::with_landscapes(config, seed, &[])
    }

    /// Like `new`; `landscapes` are the sugarscape's painted maps
    /// (`World::with_landscapes`). Other models have no landscapes and ignore them.
    pub fn with_landscapes(
        config: ModelConfig,
        seed: u64,
        landscapes: &[Option<Vec<f64>>],
    ) -> Result<Self, Vec<FieldError>> {
        Ok(match config {
            ModelConfig::Sugarscape(c) => {
                ModelWorld::Sugarscape(Box::new(World::with_landscapes(c, seed, landscapes)?))
            }
        })
    }

    pub fn kind(&self) -> ModelKind {
        match self {
            ModelWorld::Sugarscape(_) => ModelKind::Sugarscape,
        }
    }

    pub fn model(&self) -> &dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_ref(),
        }
    }

    pub fn model_mut(&mut self) -> &mut dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_mut(),
        }
    }

    pub fn sugarscape(&self) -> Option<&World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
        }
    }

    pub fn sugarscape_mut(&mut self) -> Option<&mut World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets;
    use serde_json::json;

    #[test]
    fn untagged_and_sugarscape_tagged_configs_are_sugarscape() {
        let plain = ModelConfig::from_json(r#"{"population": 100}"#).unwrap();
        let tagged =
            ModelConfig::from_json(r#"{"model": "sugarscape", "population": 100}"#).unwrap();
        assert_eq!(plain, tagged);
        assert_eq!(plain.kind(), ModelKind::Sugarscape);
        assert_eq!(plain.sugarscape().unwrap().population, 100);
    }

    #[test]
    fn sugarscape_configs_serialize_without_a_tag() {
        let c = presets::by_id("iv-3-trade").unwrap().config;
        let model = ModelConfig::from(c.clone());
        assert_eq!(
            serde_json::to_string(&model).unwrap(),
            serde_json::to_string(&c).unwrap()
        );
        assert!(!serde_json::to_string(&model).unwrap().contains("\"model\""));
    }

    #[test]
    fn unknown_models_are_field_errors() {
        let e = ModelConfig::from_json(r#"{"model": "boids"}"#).unwrap_err();
        assert_eq!(e[0].field, "model");
        assert!(e[0].message.contains("\"boids\""), "{e:?}");
        let e = ModelConfig::from_json(r#"{"model": 3}"#).unwrap_err();
        assert_eq!(e[0].field, "model");
    }

    #[test]
    fn with_path_cannot_change_the_model() {
        let c = ModelConfig::from(Config::default());
        assert!(c.with_path("model", &json!("schelling")).is_err());
        let next = c.with_path("population", &json!(10)).unwrap();
        assert_eq!(next.sugarscape().unwrap().population, 10);
    }

    #[test]
    fn a_sugarscape_model_world_is_the_world() {
        let config = presets::by_id("ii-2-unit").unwrap().config;
        let mut direct = World::new(config.clone(), 1).unwrap();
        let mut any = ModelWorld::new(config.into(), 1).unwrap();
        direct.run(50);
        any.model_mut().run(50);
        let m = any.model();
        assert_eq!(m.fingerprint(), direct.fingerprint());
        assert_eq!((m.tick(), m.population()), (50, direct.population()));
        assert_eq!(m.series("population"), direct.stats.series("population"));
        assert_eq!(m.series_csv(), export::series_csv(&direct));
        assert_eq!(m.size(), (50, 50));
        assert!(m.render("nope", "resource:0", &mut Vec::new()).is_err());
        assert!(any.sugarscape().is_some());
    }

    #[test]
    fn a_sugarscape_world_refuses_structural_changes() {
        let mut c = Config::default();
        c.goods[0].map = crate::config::Map::Flat { capacity: 4.0 };
        let mut any = ModelWorld::new(c.clone().into(), 1).unwrap();
        c.width = 60;
        let e = any.model_mut().set_config(c.into()).unwrap_err();
        assert_eq!(e[0].field, "width");
    }
}
