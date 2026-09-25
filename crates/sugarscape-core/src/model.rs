//! Model kinds (milestone 9): the sugarscape and the other artificial
//! societies of Chapter VI behind one config type and one trait, so the
//! worker host, sweeps and the CLI run any of them the same way. See
//! docs/superpowers/specs/2026-09-25-other-artificial-societies-design.md.

use serde::{Serialize, Serializer};

use crate::anasazi::{AnasaziConfig, AnasaziWorld};
use crate::civil::{CivilConfig, CivilWorld};
use crate::config::{Config, FieldError};
use crate::render::{self, ColorMode, Layer};
use crate::ring::{RingConfig, RingWorld};
use crate::schelling::{SchellingConfig, SchellingWorld};
use crate::schema::Param;
use crate::spatial::{SpatialConfig, SpatialWorld};
use crate::world::World;
use crate::{anasazi, civil, export, ring, schelling, spatial, stats};

/// Which model a config or world is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    Sugarscape,
    Schelling,
    Ring,
    Anasazi,
    Civil,
    Spatial,
}

impl ModelKind {
    pub const ALL: [ModelKind; 6] = [
        ModelKind::Sugarscape,
        ModelKind::Schelling,
        ModelKind::Ring,
        ModelKind::Anasazi,
        ModelKind::Civil,
        ModelKind::Spatial,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ModelKind::Sugarscape => "sugarscape",
            ModelKind::Schelling => "schelling",
            ModelKind::Ring => "ring",
            ModelKind::Anasazi => "anasazi",
            ModelKind::Civil => "civil",
            ModelKind::Spatial => "spatial",
        }
    }

    /// The Rules panel's fields; empty for the sugarscape, whose panel is
    /// hand-built.
    pub fn schema(self) -> Vec<Param> {
        match self {
            ModelKind::Sugarscape => Vec::new(),
            ModelKind::Schelling => schelling::schema(),
            ModelKind::Ring => ring::schema(),
            ModelKind::Anasazi => anasazi::schema(),
            ModelKind::Civil => civil::schema(),
            ModelKind::Spatial => spatial::schema(),
        }
    }
}

/// A config of any model. On the wire it is the model's own config object;
/// every model but the sugarscape carries `"model": "<kind>"`, and an object
/// without a `model` key (every config, link, session and sweep written
/// before milestone 9) is a sugarscape config.
// Configs are cloned rarely (never per tick), so the sugarscape's larger
// variant is not boxed.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ModelConfig {
    Sugarscape(Config),
    Schelling(SchellingConfig),
    Ring(RingConfig),
    Anasazi(AnasaziConfig),
    Civil(CivilConfig),
    Spatial(SpatialConfig),
}

/// Another model's config on the wire: its fields and `"model": "<kind>"`.
#[derive(Serialize)]
#[serde(tag = "model", rename_all = "snake_case")]
enum Tagged<'a> {
    Schelling(&'a SchellingConfig),
    Ring(&'a RingConfig),
    Anasazi(&'a AnasaziConfig),
    Civil(&'a CivilConfig),
    Spatial(&'a SpatialConfig),
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
            ModelConfig::Schelling(c) => Tagged::Schelling(c).serialize(s),
            ModelConfig::Ring(c) => Tagged::Ring(c).serialize(s),
            ModelConfig::Anasazi(c) => Tagged::Anasazi(c).serialize(s),
            ModelConfig::Civil(c) => Tagged::Civil(c).serialize(s),
            ModelConfig::Spatial(c) => Tagged::Spatial(c).serialize(s),
        }
    }
}

impl ModelConfig {
    pub fn kind(&self) -> ModelKind {
        match self {
            ModelConfig::Sugarscape(_) => ModelKind::Sugarscape,
            ModelConfig::Schelling(_) => ModelKind::Schelling,
            ModelConfig::Ring(_) => ModelKind::Ring,
            ModelConfig::Anasazi(_) => ModelKind::Anasazi,
            ModelConfig::Civil(_) => ModelKind::Civil,
            ModelConfig::Spatial(_) => ModelKind::Spatial,
        }
    }

    /// The sugarscape config, if this is one.
    pub fn sugarscape(&self) -> Option<&Config> {
        match self {
            ModelConfig::Sugarscape(c) => Some(c),
            _ => None,
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
    /// (`Config::from_value`); another model's missing fields take its
    /// defaults, and unknown fields are errors.
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
            "schelling" => serde_json::from_value(value)
                .map(ModelConfig::Schelling)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "ring" => serde_json::from_value(value)
                .map(ModelConfig::Ring)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "anasazi" => serde_json::from_value(value)
                .map(ModelConfig::Anasazi)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "civil" => serde_json::from_value(value)
                .map(ModelConfig::Civil)
                .map_err(|e| FieldError::new("config", e.to_string())),
            "spatial" => serde_json::from_value(value)
                .map(ModelConfig::Spatial)
                .map_err(|e| FieldError::new("config", e.to_string())),
            _ => Err(FieldError::new(
                "model",
                format!(
                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil or spatial)"
                ),
            )),
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        match self {
            ModelConfig::Sugarscape(c) => c.validate(),
            ModelConfig::Schelling(c) => c.validate(),
            ModelConfig::Ring(c) => c.validate(),
            ModelConfig::Anasazi(c) => c.validate(),
            ModelConfig::Civil(c) => c.validate(),
            ModelConfig::Spatial(c) => c.validate(),
        }
    }

    /// A copy with one dotted `path` set to `value` (the model cannot be
    /// changed this way: `model` is not a field of any model's config).
    pub fn with_path(&self, path: &str, value: &serde_json::Value) -> Result<Self, FieldError> {
        match self {
            ModelConfig::Sugarscape(c) => c.with_path(path, value).map(ModelConfig::Sugarscape),
            ModelConfig::Schelling(c) => set_path(c, path, value).map(ModelConfig::Schelling),
            ModelConfig::Ring(c) => set_path(c, path, value).map(ModelConfig::Ring),
            ModelConfig::Anasazi(c) => set_path(c, path, value).map(ModelConfig::Anasazi),
            ModelConfig::Civil(c) => set_path(c, path, value).map(ModelConfig::Civil),
            ModelConfig::Spatial(c) => set_path(c, path, value).map(ModelConfig::Spatial),
        }
    }

    /// The most ticks a world with this config can run, when it stops on
    /// its own (the anasazi at its end year); `None` when it runs forever.
    pub fn max_ticks(&self) -> Option<u32> {
        match self {
            ModelConfig::Anasazi(c) => Some(c.end_year.saturating_sub(c.start_year)),
            ModelConfig::Sugarscape(_)
            | ModelConfig::Schelling(_)
            | ModelConfig::Ring(_)
            | ModelConfig::Civil(_)
            | ModelConfig::Spatial(_) => None,
        }
    }

    /// The statistics series a world with this config records.
    pub fn series_names(&self) -> Vec<String> {
        match self {
            ModelConfig::Sugarscape(c) => stats::series_names(c),
            ModelConfig::Schelling(_) => schelling::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Ring(_) => ring::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Anasazi(_) => anasazi::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Civil(_) => civil::SERIES.iter().map(|s| s.to_string()).collect(),
            ModelConfig::Spatial(_) => spatial::SERIES.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// `config` with the dotted `path` set to `value` (through its JSON, like
/// `Config::with_path`; the error field is `schedule`, as there).
fn set_path<T: Serialize + serde::de::DeserializeOwned>(
    config: &T,
    path: &str,
    value: &serde_json::Value,
) -> Result<T, FieldError> {
    let mut json = serde_json::to_value(config).expect("config serializes");
    let unknown = || FieldError::new("schedule", format!("unknown field {path}"));
    let mut slot = &mut json;
    for key in path.split('.') {
        slot = slot.get_mut(key).ok_or_else(unknown)?;
    }
    *slot = value.clone();
    serde_json::from_value(json).map_err(|e| FieldError::new("schedule", format!("{path}: {e}")))
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
    /// The latest value of series `name` (or `"tick"`), or `None` if unknown or there is no history yet.
    fn latest_value(&self, name: &str) -> Option<f64> {
        self.series(name).and_then(|v| v.last().copied())
    }
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
    /// Whether the world has run its course (the anasazi's end year):
    /// `run` then does nothing. Other models never finish.
    fn finished(&self) -> bool {
        false
    }
}

/// The error for handing a world another model's config.
pub(crate) fn wrong_model(want: ModelKind, got: &ModelConfig) -> Vec<FieldError> {
    vec![FieldError::new(
        "model",
        format!(
            "a {} world cannot take a {} config",
            want.as_str(),
            got.kind().as_str()
        ),
    )]
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

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
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
            other => Err(wrong_model(ModelKind::Sugarscape, &other)),
        }
    }
}

/// A world of any model. Sugarscape-only calls (painting, networks, the
/// credit graph …) go through `sugarscape()`/`sugarscape_mut()`.
pub enum ModelWorld {
    Sugarscape(Box<World>),
    Schelling(Box<SchellingWorld>),
    Ring(Box<RingWorld>),
    Anasazi(Box<AnasaziWorld>),
    Civil(Box<CivilWorld>),
    Spatial(Box<SpatialWorld>),
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
            ModelConfig::Schelling(c) => {
                ModelWorld::Schelling(Box::new(SchellingWorld::new(c, seed)?))
            }
            ModelConfig::Ring(c) => ModelWorld::Ring(Box::new(RingWorld::new(c, seed)?)),
            ModelConfig::Anasazi(c) => ModelWorld::Anasazi(Box::new(AnasaziWorld::new(c, seed)?)),
            ModelConfig::Civil(c) => ModelWorld::Civil(Box::new(CivilWorld::new(c, seed)?)),
            ModelConfig::Spatial(c) => ModelWorld::Spatial(Box::new(SpatialWorld::new(c, seed)?)),
        })
    }

    pub fn kind(&self) -> ModelKind {
        match self {
            ModelWorld::Sugarscape(_) => ModelKind::Sugarscape,
            ModelWorld::Schelling(_) => ModelKind::Schelling,
            ModelWorld::Ring(_) => ModelKind::Ring,
            ModelWorld::Anasazi(_) => ModelKind::Anasazi,
            ModelWorld::Civil(_) => ModelKind::Civil,
            ModelWorld::Spatial(_) => ModelKind::Spatial,
        }
    }

    pub fn model(&self) -> &dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_ref(),
            ModelWorld::Schelling(w) => w.as_ref(),
            ModelWorld::Ring(w) => w.as_ref(),
            ModelWorld::Anasazi(w) => w.as_ref(),
            ModelWorld::Civil(w) => w.as_ref(),
            ModelWorld::Spatial(w) => w.as_ref(),
        }
    }

    pub fn model_mut(&mut self) -> &mut dyn Model {
        match self {
            ModelWorld::Sugarscape(w) => w.as_mut(),
            ModelWorld::Schelling(w) => w.as_mut(),
            ModelWorld::Ring(w) => w.as_mut(),
            ModelWorld::Anasazi(w) => w.as_mut(),
            ModelWorld::Civil(w) => w.as_mut(),
            ModelWorld::Spatial(w) => w.as_mut(),
        }
    }

    pub fn sugarscape(&self) -> Option<&World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
            _ => None,
        }
    }

    pub fn sugarscape_mut(&mut self) -> Option<&mut World> {
        match self {
            ModelWorld::Sugarscape(w) => Some(w),
            _ => None,
        }
    }

    pub fn ring(&self) -> Option<&RingWorld> {
        match self {
            ModelWorld::Ring(w) => Some(w),
            _ => None,
        }
    }

    pub fn anasazi(&self) -> Option<&AnasaziWorld> {
        match self {
            ModelWorld::Anasazi(w) => Some(w),
            _ => None,
        }
    }
}

/// A copy of a world's state without its statistics history (a keyframe):
/// about one copy of the current state, however long the run.
pub struct Checkpoint {
    world: ModelWorld,
    tick: u64,
}

impl Checkpoint {
    /// The tick the copy was taken at.
    pub fn tick(&self) -> u64 {
        self.tick
    }
}

/// Moves `$w`'s history out, clones it, and moves the history back.
macro_rules! copy_without_history {
    ($variant:ident, $w:expr) => {{
        let stats = std::mem::take(&mut $w.stats);
        let copy = (**$w).clone();
        $w.stats = stats;
        ModelWorld::$variant(Box::new(copy))
    }};
}

/// Replaces `$live` by a copy of `$kept`, giving it `$live`'s history cut to `$kept`'s tick.
macro_rules! restore_into {
    ($live:expr, $kept:expr) => {{
        let mut stats = std::mem::take(&mut $live.stats);
        stats.truncate($kept.tick as usize + 1);
        let mut next = (**$kept).clone();
        next.stats = stats;
        **$live = next;
    }};
}

impl ModelWorld {
    /// A keyframe of this world, or `None` for a model without them.
    #[allow(unreachable_patterns)]
    pub fn checkpoint(&mut self) -> Option<Checkpoint> {
        let tick = self.model().tick();
        let world = match self {
            ModelWorld::Sugarscape(w) => copy_without_history!(Sugarscape, w),
            ModelWorld::Schelling(w) => copy_without_history!(Schelling, w),
            ModelWorld::Ring(w) => copy_without_history!(Ring, w),
            ModelWorld::Anasazi(w) => copy_without_history!(Anasazi, w),
            ModelWorld::Civil(w) => copy_without_history!(Civil, w),
            ModelWorld::Spatial(w) => copy_without_history!(Spatial, w),
            _ => return None,
        };
        Some(Checkpoint { world, tick })
    }

    /// Returns this world to `cp`, keeping its statistics history up to `cp`'s tick. The world
    /// must have reached that tick (its history must hold it) and be of the same model.
    #[allow(unreachable_patterns)]
    pub fn restore(&mut self, cp: &Checkpoint) -> Result<(), String> {
        if self.model().tick() < cp.tick {
            return Err(format!("this world has not reached tick {}", cp.tick));
        }
        match (self, &cp.world) {
            (ModelWorld::Sugarscape(live), ModelWorld::Sugarscape(kept)) => {
                restore_into!(live, kept)
            }
            (ModelWorld::Schelling(live), ModelWorld::Schelling(kept)) => restore_into!(live, kept),
            (ModelWorld::Ring(live), ModelWorld::Ring(kept)) => restore_into!(live, kept),
            (ModelWorld::Anasazi(live), ModelWorld::Anasazi(kept)) => restore_into!(live, kept),
            (ModelWorld::Civil(live), ModelWorld::Civil(kept)) => restore_into!(live, kept),
            (ModelWorld::Spatial(live), ModelWorld::Spatial(kept)) => restore_into!(live, kept),
            _ => return Err("the keyframe is of another model".into()),
        }
        Ok(())
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
    fn schelling_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "schelling", "population": 100}"#).unwrap();
        assert_eq!(c.kind(), ModelKind::Schelling);
        let ModelConfig::Schelling(s) = &c else {
            unreachable!()
        };
        assert_eq!(
            (s.population, s.width),
            (100, 50),
            "missing fields take the defaults"
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "schelling");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        let e = ModelConfig::from_json(r#"{"model": "schelling", "vision": 3}"#).unwrap_err();
        assert!(e[0].message.contains("vision"), "{e:?}");
        let e =
            ModelConfig::from_json(r#"{"model": "schelling", "population": 2500}"#).unwrap_err();
        assert_eq!(e[0].field, "population");
    }

    #[test]
    fn with_path_sets_another_models_fields() {
        let c = ModelConfig::Schelling(SchellingConfig::default());
        let next = c.with_path("preference.max", &json!(0.5)).unwrap();
        let ModelConfig::Schelling(s) = &next else {
            unreachable!()
        };
        assert_eq!((s.preference.min, s.preference.max), (0.25, 0.5));
        assert_eq!(
            c.with_path("vision.max", &json!(3)).unwrap_err().message,
            "unknown field vision.max"
        );
        assert!(c.with_path("model", &json!("sugarscape")).is_err());
        assert!(c.with_path("population", &json!("many")).is_err());
    }

    #[test]
    fn a_world_refuses_another_models_config() {
        let mut any = ModelWorld::new(ModelConfig::from(Config::default()), 1).unwrap();
        let e = any
            .model_mut()
            .set_config(ModelConfig::Schelling(SchellingConfig::default()))
            .unwrap_err();
        assert_eq!(e[0].field, "model");
        let mut s = ModelWorld::new(ModelConfig::Schelling(SchellingConfig::default()), 1).unwrap();
        assert!(s.model_mut().set_config(Config::default().into()).is_err());
        assert!(s.sugarscape().is_none());
    }

    #[test]
    fn ring_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "ring", "start": "megagroup"}"#).unwrap();
        assert_eq!(c.kind(), ModelKind::Ring);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(
            (json["model"].as_str(), json["sites"].as_u64()),
            (Some("ring"), Some(150))
        );
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(
            c.series_names(),
            [
                "flocks",
                "mean_flock",
                "largest_flock",
                "mean_distance",
                "population"
            ]
        );
        let e = ModelConfig::from_json(r#"{"model": "ring", "start": "clumps"}"#).unwrap_err();
        assert_eq!(e[0].field, "config");
    }

    #[test]
    fn anasazi_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(r#"{"model": "anasazi", "quirks": {"wrap_edges": false}}"#)
            .unwrap();
        assert_eq!(c.kind(), ModelKind::Anasazi);
        let ModelConfig::Anasazi(a) = &c else {
            unreachable!()
        };
        assert!(
            !a.quirks.wrap_edges && a.quirks.occupancy_leak,
            "missing fields take the defaults"
        );
        assert_eq!(a.harvest_adjustment, 0.56);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "anasazi");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..3], ["households", "historical", "fit"]);
        let next = c.with_path("quirks.occupancy_leak", &json!(false)).unwrap();
        let ModelConfig::Anasazi(n) = &next else {
            unreachable!()
        };
        assert!(!n.quirks.occupancy_leak);
        let e = ModelConfig::from_json(r#"{"model": "anasazi", "end_year": 700}"#).unwrap_err();
        assert_eq!(e[0].field, "end_year");
        let w = ModelWorld::new(c, 1).unwrap();
        assert!(w.anasazi().is_some() && w.sugarscape().is_none());
        assert_eq!(w.model().size(), (80, 120));
    }

    #[test]
    fn civil_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "civil", "variant": "ethnic", "quirks": {"floor_ratio": true}}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Civil);
        let ModelConfig::Civil(v) = &c else {
            unreachable!()
        };
        assert!(v.quirks.floor_ratio && !v.quirks.jailed_stay);
        assert_eq!(
            (v.width, v.legitimacy),
            (40, 0.82),
            "missing fields take the defaults"
        );
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "civil");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[..3], ["population", "active", "quiet"]);
        assert_eq!(c.max_ticks(), None);
        let next = c.with_path("vision.cop", &json!(3.0)).unwrap();
        let ModelConfig::Civil(n) = &next else {
            unreachable!()
        };
        assert_eq!(n.vision.cop, 3.0);
        let e = ModelConfig::from_json(r#"{"model": "civil", "legitimacy": 2}"#).unwrap_err();
        assert_eq!(e[0].field, "legitimacy");
        let w = ModelWorld::new(c, 1).unwrap();
        assert_eq!(w.kind(), ModelKind::Civil);
        assert_eq!(w.model().size(), (40, 40));
    }

    #[test]
    fn spatial_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "spatial", "lattice": "cube", "width": 10, "update": "asynchronous"}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Spatial);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "spatial");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[0], "fraction_c");
        let next = c.with_path("b", &json!(1.6)).unwrap();
        let ModelConfig::Spatial(s) = &next else {
            unreachable!()
        };
        assert_eq!(s.b, 1.6);
        let e = ModelConfig::from_json(r#"{"model": "spatial", "b": 0}"#).unwrap_err();
        assert_eq!(e[0].field, "b");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!((w.kind(), w.model().size()), (ModelKind::Spatial, (10, 10)));
        assert_eq!(w.model().population(), 1000);
        let cp = w.checkpoint().expect("spatial worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }

    #[test]
    fn only_the_anasazi_finishes() {
        let mut w = ModelWorld::new(
            ModelConfig::Anasazi(crate::anasazi::AnasaziConfig {
                start_year: 1349,
                ..Default::default()
            }),
            1,
        )
        .unwrap();
        assert!(!w.model().finished());
        w.model_mut().run(5);
        assert!(w.model().finished());
        assert_eq!(w.model().tick(), 1);
        let s = ModelWorld::new(ModelConfig::Ring(RingConfig::default()), 1).unwrap();
        assert!(!s.model().finished());
    }

    #[test]
    fn every_kind_names_itself_and_only_other_models_have_schemas() {
        let names: Vec<&str> = ModelKind::ALL.iter().map(|k| k.as_str()).collect();
        assert_eq!(
            names,
            [
                "sugarscape",
                "schelling",
                "ring",
                "anasazi",
                "civil",
                "spatial"
            ]
        );
        assert!(ModelKind::Sugarscape.schema().is_empty());
        for kind in &ModelKind::ALL[1..] {
            assert!(!kind.schema().is_empty(), "{kind:?}");
        }
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
