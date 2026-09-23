//! Parameter sweeps (milestone 5): a grid of config values × seeds, each run
//! summarized by one statistic. The CLI and the browser both call this
//! module; see docs/superpowers/specs/2026-09-23-experiments-design.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::{Config, FieldError};
use crate::world::World;
use crate::{presets, stats};

pub const MAX_X_VALUES: usize = 64;
pub const MAX_SERIES_VALUES: usize = 16;
pub const MAX_SEEDS: u32 = 100;
pub const MAX_TICKS: u32 = 100_000;
pub const MAX_POINTS: usize = 10_000;

/// Where every run's config starts: a preset, or a config in either shape.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Base {
    Preset(String),
    Config(Value),
}

/// One value of an axis: where it sits on the chart (`at`), an optional line
/// name, and the dotted config paths it sets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AxisValue {
    pub at: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub set: BTreeMap<String, Value>,
}

impl AxisValue {
    /// `name`, or `"<label> = <at>"`.
    pub fn display_name(&self, label: &str) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| format!("{label} = {}", self.at))
    }
}

/// An axis in full form. The shorthand `{ label?, path, values }` is expanded
/// when read (Decision 1); axes are always written in full.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "AxisInput")]
pub struct Axis {
    pub label: String,
    pub values: Vec<AxisValue>,
}

/// Either axis form, as read.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AxisInput {
    label: Option<String>,
    path: Option<String>,
    values: Vec<Value>,
}

impl TryFrom<AxisInput> for Axis {
    type Error = String;

    fn try_from(input: AxisInput) -> Result<Self, String> {
        let Some(path) = input.path else {
            let label = input.label.ok_or("an axis without a path needs a label")?;
            let values = input
                .values
                .into_iter()
                .map(serde_json::from_value)
                .collect::<Result<_, _>>()
                .map_err(|e| format!("axis value: {e}"))?;
            return Ok(Axis { label, values });
        };
        // Value i sets `path`; it sits at the value itself when that is a
        // number, otherwise at i.
        let values = input
            .values
            .into_iter()
            .enumerate()
            .map(|(i, v)| AxisValue {
                at: v.as_f64().unwrap_or(i as f64),
                name: None,
                set: BTreeMap::from([(path.clone(), v)]),
            })
            .collect();
        Ok(Axis {
            label: input.label.unwrap_or(path),
            values,
        })
    }
}

/// Seeds `from, from + 1, …, from + count − 1`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seeds {
    pub from: u64,
    pub count: u32,
}

/// What each run is summarized by, over one statistics series.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Metric {
    /// The value at the last tick.
    Final { series: String },
    /// The mean over ticks `from..=to` (`to` defaults to the last tick).
    WindowMean {
        series: String,
        from: u32,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<u32>,
    },
    /// Means over blocks of `every` ticks (see `blocks`).
    Timeseries { series: String, every: u32 },
}

impl Metric {
    /// The statistics series the metric reads.
    pub fn series(&self) -> &str {
        match self {
            Metric::Final { series }
            | Metric::WindowMean { series, .. }
            | Metric::Timeseries { series, .. } => series,
        }
    }
}

/// A parameter sweep (the spec's "Sweep format").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sweep {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub base: Base,
    /// Applied to every run, before the series and x values.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub set: BTreeMap<String, Value>,
    pub x: Axis,
    /// One line per value; `None` is one implicit line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<Axis>,
    pub seeds: Seeds,
    pub ticks: u32,
    pub metric: Metric,
}

/// One run of a sweep. `index` is its position in series-major, then x, then
/// seed order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub index: usize,
    pub series: usize,
    pub x: usize,
    pub seed: u64,
}

impl Sweep {
    /// Parses a sweep; errors have field `sweep`.
    pub fn from_json(json: &str) -> Result<Sweep, Vec<FieldError>> {
        serde_json::from_str(json).map_err(|e| vec![FieldError::new("sweep", e.to_string())])
    }

    /// Number of lines: the series axis's values, or 1 without one.
    pub fn series_count(&self) -> usize {
        self.series.as_ref().map_or(1, |axis| axis.values.len())
    }

    pub fn point_count(&self) -> usize {
        self.series_count()
            .saturating_mul(self.x.values.len())
            .saturating_mul(self.seeds.count as usize)
    }

    /// A line's name: its series value's `display_name`, or the metric's
    /// statistics series when there is no series axis (Decision 7).
    pub fn series_name(&self, series: usize) -> String {
        match &self.series {
            Some(axis) => axis.values[series].display_name(&axis.label),
            None => self.metric.series().to_string(),
        }
    }

    /// Limits and metric bounds (Decisions 2 and 4); builds no config.
    pub fn check_shape(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = Vec::new();
        let mut check = |ok: bool, field: &str, message: String| {
            if !ok {
                errors.push(FieldError::new(field, message));
            }
        };
        check(
            !self.name.trim().is_empty(),
            "name",
            "must not be empty".into(),
        );
        check(
            (1..=MAX_X_VALUES).contains(&self.x.values.len()),
            "x.values",
            format!("must list 1 to {MAX_X_VALUES} values"),
        );
        if let Some(axis) = &self.series {
            check(
                (1..=MAX_SERIES_VALUES).contains(&axis.values.len()),
                "series.values",
                format!("must list 1 to {MAX_SERIES_VALUES} values"),
            );
        }
        check(
            (1..=MAX_SEEDS).contains(&self.seeds.count),
            "seeds.count",
            format!("must be 1 to {MAX_SEEDS}"),
        );
        check(
            self.seeds.from.saturating_add(u64::from(self.seeds.count)) <= u64::from(u32::MAX) + 1,
            "seeds.from",
            "seeds must stay ≤ 4294967295 (the playground's seed range)".into(),
        );
        check(
            (1..=MAX_TICKS).contains(&self.ticks),
            "ticks",
            format!("must be 1 to {MAX_TICKS}"),
        );
        check(
            self.point_count() <= MAX_POINTS,
            "points",
            format!(
                "at most {MAX_POINTS} points (x values × series values × seeds); this sweep has {}",
                self.point_count()
            ),
        );
        match &self.metric {
            Metric::Final { .. } => {}
            Metric::WindowMean { from, to, .. } => {
                let to = to.unwrap_or(self.ticks);
                check(to <= self.ticks, "metric.to", "must be ≤ ticks".into());
                check(
                    *from <= to,
                    "metric.from",
                    "must be ≤ the window's end".into(),
                );
            }
            Metric::Timeseries { every, .. } => {
                check(
                    (1..=self.ticks).contains(every),
                    "metric.every",
                    "must be 1 to ticks".into(),
                );
                check(
                    self.x.values.len() == 1,
                    "metric.kind",
                    "a timeseries needs a single x value".into(),
                );
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Point `index`, after `check_shape`.
    pub fn point(&self, index: usize) -> Result<Point, Vec<FieldError>> {
        self.check_shape()?;
        let count = self.point_count();
        if index >= count {
            return Err(vec![FieldError::new(
                "point",
                format!("point {index} is out of range (the sweep has {count})"),
            )]);
        }
        Ok(self.point_at(index))
    }

    fn point_at(&self, index: usize) -> Point {
        let seeds = self.seeds.count as usize;
        let xs = self.x.values.len();
        Point {
            index,
            series: index / (seeds * xs),
            x: index / seeds % xs,
            seed: self.seeds.from + (index % seeds) as u64,
        }
    }

    /// Index of the first point (lowest seed) at (series, x).
    fn first_point(&self, series: usize, x: usize) -> usize {
        (series * self.x.values.len() + x) * self.seeds.count as usize
    }

    /// Every point, after checking the shape and building every cell's config.
    pub fn points(&self) -> Result<Vec<Point>, Vec<FieldError>> {
        self.prepare().map(|(points, _)| points)
    }

    /// The points and each (series, x) cell's config, indexed
    /// `series · nx + x`. Errors are deduplicated across cells (Decision 3).
    fn prepare(&self) -> Result<(Vec<Point>, Vec<Config>), Vec<FieldError>> {
        self.check_shape()?;
        let mut configs = Vec::new();
        let mut errors: Vec<FieldError> = Vec::new();
        for series in 0..self.series_count() {
            for x in 0..self.x.values.len() {
                match self.config_at(series, x) {
                    Ok(config) => configs.push(config),
                    Err(errs) => {
                        for e in errs {
                            if !errors.contains(&e) {
                                errors.push(e);
                            }
                        }
                    }
                }
            }
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        let points = (0..self.point_count()).map(|i| self.point_at(i)).collect();
        Ok((points, configs))
    }

    /// The config `point` runs (it depends on the series and x value only).
    /// `point` must come from this sweep (`points` or `point`).
    pub fn config_for(&self, point: &Point) -> Result<Config, Vec<FieldError>> {
        self.config_at(point.series, point.x)
    }

    fn base_config(&self) -> Result<Config, Vec<FieldError>> {
        match &self.base {
            Base::Preset(id) => presets::by_id(id)
                .map(|p| p.config)
                .ok_or_else(|| vec![FieldError::new("base", format!("unknown preset {id:?}"))]),
            Base::Config(value) => Config::from_value(value.clone())
                .map_err(|e| vec![FieldError::new("base", e.message)]),
        }
    }

    /// `base`, then `set`, then the series value's `set`, then the x value's
    /// `set` (each in key order), then `validate` and the metric's series.
    fn config_at(&self, series: usize, x: usize) -> Result<Config, Vec<FieldError>> {
        let mut config = self.base_config()?;
        let mut layers = vec![("set".to_string(), &self.set)];
        if let Some(axis) = &self.series {
            layers.push((format!("series[{series}].set"), &axis.values[series].set));
        }
        layers.push((format!("x[{x}].set"), &self.x.values[x].set));
        for (field, set) in layers {
            for (path, value) in set {
                config = config
                    .with_path(path, value)
                    .map_err(|e| vec![FieldError::new(format!("{field}.{path}"), e.message)])?;
            }
        }
        let prefix = format!(
            "point {} (x={x}, series={series})",
            self.first_point(series, x)
        );
        config.validate().map_err(|errs| {
            errs.into_iter()
                .map(|e| FieldError::new("points", format!("{prefix}: {}: {}", e.field, e.message)))
                .collect::<Vec<_>>()
        })?;
        let name = self.metric.series();
        if !stats::series_names(&config).iter().any(|n| n == name) {
            return Err(vec![FieldError::new(
                "metric.series",
                format!("{prefix}: no statistics series {name:?} in this config"),
            )]);
        }
        Ok(config)
    }
}

/// Writes non-finite numbers as JSON `null` and reads `null` back as NaN
/// (Decision 5).
mod nan_as_null {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &f64, s: S) -> Result<S::Ok, S::Error> {
        if v.is_finite() {
            s.serialize_f64(*v)
        } else {
            s.serialize_none()
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        Ok(Option::<f64>::deserialize(d)?.unwrap_or(f64::NAN))
    }
}

/// `nan_as_null` for each element.
mod nan_as_null_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[f64], s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(v.iter().map(|x| x.is_finite().then_some(*x)))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<f64>, D::Error> {
        Ok(Vec::<Option<f64>>::deserialize(d)?
            .into_iter()
            .map(|x| x.unwrap_or(f64::NAN))
            .collect())
    }
}

/// A run's metric: one value, or one per block for `timeseries`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Outcome {
    Scalar {
        #[serde(with = "nan_as_null")]
        value: f64,
    },
    Series {
        #[serde(with = "nan_as_null_vec")]
        values: Vec<f64>,
    },
}

/// One finished run: `{ point, series, x, seed, value | values }`
/// (series and x are indices, Decision 6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunResult {
    pub point: usize,
    pub series: usize,
    pub x: usize,
    pub seed: u64,
    #[serde(flatten)]
    pub outcome: Outcome,
}

/// The mean of the finite values (summed in order from 0.0), or NaN when
/// there are none.
fn finite_mean(values: &[f64]) -> f64 {
    let (sum, n) = values
        .iter()
        .filter(|v| v.is_finite())
        .fold((0.0, 0usize), |(sum, n), v| (sum + v, n + 1));
    if n == 0 {
        f64::NAN
    } else {
        sum / n as f64
    }
}

/// Blocks of `every` ticks as `(first, last)`: block k covers ticks
/// `k·every + 1 ..= min((k + 1)·every, ticks)`. `every` must be ≥ 1.
pub fn blocks(ticks: u32, every: u32) -> Vec<(u32, u32)> {
    (0..ticks.div_ceil(every))
        .map(|k| (k * every + 1, ((k + 1) * every).min(ticks)))
        .collect()
}

/// `metric` over one run whose statistics history is `history`
/// (`history[t]` is tick t's value for t = 0..=ticks). A population of 0
/// needs no special case: its statistics simply continue.
pub fn measure(metric: &Metric, ticks: u32, history: &[f64]) -> Outcome {
    let window = |from: u32, to: u32| finite_mean(&history[from as usize..=to as usize]);
    match metric {
        Metric::Final { .. } => Outcome::Scalar {
            value: history[ticks as usize],
        },
        Metric::WindowMean { from, to, .. } => Outcome::Scalar {
            value: window(*from, to.unwrap_or(ticks)),
        },
        Metric::Timeseries { every, .. } => Outcome::Series {
            values: blocks(ticks, *every)
                .into_iter()
                .map(|(first, last)| window(first, last))
                .collect(),
        },
    }
}

/// Runs `point` and measures it. The point's config must be valid (take
/// points from `Sweep::points`, or check `Sweep::config_for` first).
pub fn run_point(sweep: &Sweep, point: &Point) -> RunResult {
    let config = sweep
        .config_for(point)
        .unwrap_or_else(|e| panic!("point {} has an invalid config: {e:?}", point.index));
    run_config(sweep, point, config)
}

fn run_config(sweep: &Sweep, point: &Point, config: Config) -> RunResult {
    let mut world = World::new(config, point.seed).expect("sweep configs are validated");
    world.run(sweep.ticks);
    let history = world
        .stats
        .series(sweep.metric.series())
        .expect("the metric's series is checked against every config");
    RunResult {
        point: point.index,
        series: point.series,
        x: point.x,
        seed: point.seed,
        outcome: measure(&sweep.metric, sweep.ticks, &history),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 2 series × 3 x values × 2 seeds on ii-2-unit with 50 agents. The CLI
    /// and WASM tests keep their own copies.
    pub(crate) fn tiny() -> Value {
        json!({
            "name": "tiny",
            "base": { "preset": "ii-2-unit" },
            "set": { "population": 50 },
            "x": { "path": "vision.max", "values": [2, 4, 6] },
            "series": { "label": "Metabolism", "values": [
                { "at": 1, "set": { "goods.0.metabolism": { "min": 1, "max": 1 } } },
                { "at": 3, "name": "wide", "set": { "goods.0.metabolism": { "min": 1, "max": 5 } } }
            ] },
            "seeds": { "from": 5, "count": 2 },
            "ticks": 20,
            "metric": { "kind": "window_mean", "series": "population", "from": 10 }
        })
    }

    pub(crate) fn sweep(value: Value) -> Sweep {
        serde_json::from_value(value).unwrap()
    }

    fn with(mut value: Value, key: &str, v: Value) -> Value {
        value[key] = v;
        value
    }

    fn without(mut value: Value, key: &str) -> Value {
        value.as_object_mut().unwrap().remove(key);
        value
    }

    #[test]
    fn points_run_series_major_then_x_then_seed() {
        let s = sweep(tiny());
        let points = s.points().unwrap();
        assert_eq!(points.len(), 12);
        let p = |index, series, x, seed| Point {
            index,
            series,
            x,
            seed,
        };
        assert_eq!(points[0], p(0, 0, 0, 5));
        assert_eq!(points[1], p(1, 0, 0, 6));
        assert_eq!(points[2], p(2, 0, 1, 5));
        assert_eq!(points[7], p(7, 1, 0, 6));
        assert_eq!(points[11], p(11, 1, 2, 6));
        for (i, point) in points.iter().enumerate() {
            assert_eq!(point.index, i);
            assert_eq!(s.point(i).unwrap(), *point);
        }
        assert_eq!(s.point(12).unwrap_err()[0].field, "point");
    }

    #[test]
    fn without_series_there_is_one_line_named_after_the_statistic() {
        let s = sweep(without(tiny(), "series"));
        assert_eq!(s.series_count(), 1);
        assert_eq!(s.points().unwrap().len(), 6);
        assert_eq!(s.series_name(0), "population");
    }

    #[test]
    fn series_lines_are_named() {
        let s = sweep(tiny());
        assert_eq!(s.series_name(0), "Metabolism = 1");
        assert_eq!(s.series_name(1), "wide");
    }

    #[test]
    fn shorthand_axes_expand() {
        let s = sweep(tiny());
        assert_eq!(s.x.label, "vision.max");
        assert_eq!(
            s.x.values[1],
            AxisValue {
                at: 4.0,
                name: None,
                set: BTreeMap::from([("vision.max".to_string(), json!(4))]),
            }
        );
        let b = sweep(with(
            tiny(),
            "x",
            json!({ "label": "Trade", "path": "trade.enabled", "values": [false, true] }),
        ));
        assert_eq!(b.x.label, "Trade");
        assert_eq!((b.x.values[0].at, b.x.values[1].at), (0.0, 1.0));
        assert_eq!(b.x.values[1].set["trade.enabled"], json!(true));
    }

    #[test]
    fn full_axes_need_a_label_and_axes_are_written_in_full() {
        let unlabeled = with(tiny(), "x", json!({ "values": [{ "at": 1, "set": {} }] }));
        let err = Sweep::from_json(&unlabeled.to_string()).unwrap_err();
        assert_eq!(err[0].field, "sweep");
        assert!(err[0].message.contains("needs a label"), "{err:?}");
        let s = sweep(tiny());
        let written = serde_json::to_value(&s).unwrap();
        assert!(written["x"].get("path").is_none());
        assert_eq!(
            written["x"]["values"][0],
            json!({ "at": 2.0, "set": { "vision.max": 2 } })
        );
        assert_eq!(sweep(written), s);
    }

    #[test]
    fn unknown_fields_are_parse_errors() {
        let err = Sweep::from_json(&with(tiny(), "seed", json!(1)).to_string()).unwrap_err();
        assert_eq!(err[0].field, "sweep");
        assert!(err[0].message.contains("unknown field `seed`"), "{err:?}");
        assert_eq!(Sweep::from_json("{").unwrap_err()[0].field, "sweep");
    }

    #[test]
    fn configs_apply_set_then_series_then_x() {
        let mut v = tiny();
        v["set"] = json!({ "population": 50, "vision.max": 3, "growback.rate": 2.0 });
        v["series"]["values"][1]["set"]["population"] = json!(60);
        v["series"]["values"][1]["set"]["growback.rate"] = json!(3.0);
        let s = sweep(v);
        let first = s.config_for(&s.point(0).unwrap()).unwrap();
        assert_eq!(
            (first.population, first.vision.min, first.vision.max),
            (50, 1, 2)
        );
        assert_eq!(
            (first.goods[0].metabolism.max, first.growback.rate),
            (1, 2.0)
        );
        let last = s.config_for(&s.point(11).unwrap()).unwrap();
        assert_eq!((last.population, last.vision.max), (60, 6));
        assert_eq!((last.goods[0].metabolism.max, last.growback.rate), (5, 3.0));
    }

    #[test]
    fn a_base_config_may_use_either_shape() {
        let legacy = with(
            without(tiny(), "set"),
            "base",
            json!({ "config": { "population": 100 } }),
        );
        let s = sweep(legacy);
        let config = s.config_for(&s.point(0).unwrap()).unwrap();
        assert_eq!((config.population, config.goods.len()), (100, 1));
        let bad = sweep(with(tiny(), "base", json!({ "config": { "goods": 7 } })));
        assert_eq!(bad.points().unwrap_err()[0].field, "base");
    }

    #[test]
    fn path_errors_name_the_set_they_came_from() {
        let mut v = tiny();
        v["set"] = json!({ "population": "many" });
        let e = sweep(v).points().unwrap_err();
        assert_eq!(e.len(), 1, "one error, not one per cell: {e:?}");
        assert_eq!(e[0].field, "set.population");
        assert!(e[0].message.starts_with("population:"), "{e:?}");

        let mut v = tiny();
        v["series"]["values"][1]["set"] = json!({ "goods.0.metabolism.maxx": 2 });
        assert_eq!(
            sweep(v).points().unwrap_err(),
            vec![FieldError::new(
                "series[1].set.goods.0.metabolism.maxx",
                "unknown field goods.0.metabolism.maxx"
            )]
        );

        let v = with(
            tiny(),
            "x",
            json!({ "label": "v", "values": [
                { "at": 1, "set": { "vision.max": 2 } },
                { "at": 2, "set": { "visionn": 3 } }
            ] }),
        );
        assert_eq!(
            sweep(v).points().unwrap_err(),
            vec![FieldError::new("x[1].set.visionn", "unknown field visionn")]
        );

        let v = with(tiny(), "base", json!({ "preset": "no-such" }));
        assert_eq!(
            sweep(v).points().unwrap_err(),
            vec![FieldError::new("base", "unknown preset \"no-such\"")]
        );
    }

    #[test]
    fn invalid_configs_name_the_first_point_of_their_cell() {
        let v = with(
            without(tiny(), "series"),
            "x",
            json!({ "path": "population", "values": [50, 3000] }),
        );
        assert_eq!(
            sweep(v).points().unwrap_err(),
            vec![FieldError::new(
                "points",
                "point 2 (x=1, series=0): population: cannot exceed the number of sites"
            )]
        );
    }

    #[test]
    fn the_metric_series_must_exist_in_every_config() {
        let mut v = tiny();
        v["metric"]["series"] = json!("mean_holding_1");
        let e = sweep(v).points().unwrap_err();
        assert_eq!(e.len(), 6, "one per cell: {e:?}");
        assert_eq!(e[0].field, "metric.series");
        assert_eq!(
            e[0].message,
            "point 0 (x=0, series=0): no statistics series \"mean_holding_1\" in this config"
        );
    }

    #[test]
    fn limits_are_checked() {
        let fields = |v: Value| -> Vec<String> {
            match sweep(v).points() {
                Ok(_) => Vec::new(),
                Err(e) => e.into_iter().map(|e| e.field).collect(),
            }
        };
        let has = |v: Value, field: &str| {
            let f = fields(v);
            assert!(f.iter().any(|x| x == field), "{field} not in {f:?}");
        };
        has(with(tiny(), "ticks", json!(0)), "ticks");
        has(with(tiny(), "ticks", json!(100_001)), "ticks");
        has(with(tiny(), "name", json!("  ")), "name");
        has(
            with(tiny(), "seeds", json!({ "from": 1, "count": 0 })),
            "seeds.count",
        );
        has(
            with(tiny(), "seeds", json!({ "from": 1, "count": 101 })),
            "seeds.count",
        );
        has(
            with(
                tiny(),
                "seeds",
                json!({ "from": 4_294_967_295u64, "count": 2 }),
            ),
            "seeds.from",
        );
        assert!(sweep(with(
            tiny(),
            "seeds",
            json!({ "from": 4_294_967_294u64, "count": 2 })
        ))
        .check_shape()
        .is_ok());
        has(
            with(
                tiny(),
                "x",
                json!({ "path": "vision.max", "values": vec![2; 65] }),
            ),
            "x.values",
        );
        has(
            with(tiny(), "x", json!({ "path": "vision.max", "values": [] })),
            "x.values",
        );
        has(
            with(
                tiny(),
                "series",
                json!({ "path": "growback.rate", "values": vec![1.0; 17] }),
            ),
            "series.values",
        );
        let mut big = with(
            tiny(),
            "x",
            json!({ "path": "vision.max", "values": vec![2; 64] }),
        );
        big["series"] = json!({ "path": "growback.rate", "values": vec![1.0; 16] });
        big["seeds"]["count"] = json!(10);
        has(big, "points");
        let mut v = tiny();
        v["metric"]["to"] = json!(21);
        has(v, "metric.to");
        let mut v = tiny();
        v["metric"]["from"] = json!(15);
        v["metric"]["to"] = json!(12);
        has(v, "metric.from");
        let ts =
            |every: u32| json!({ "kind": "timeseries", "series": "population", "every": every });
        has(with(tiny(), "metric", ts(0)), "metric.every");
        has(with(tiny(), "metric", ts(21)), "metric.every");
        has(with(tiny(), "metric", ts(5)), "metric.kind");
        let single = with(
            with(tiny(), "metric", ts(5)),
            "x",
            json!({ "label": "all", "values": [{ "at": 0 }] }),
        );
        assert_eq!(fields(single), Vec::<String>::new());
    }

    #[test]
    fn metrics_read_the_history_by_tick() {
        let history: Vec<f64> = (0..=10).map(f64::from).collect();
        let scalar = |m: Metric| match measure(&m, 10, &history) {
            Outcome::Scalar { value } => value,
            other => panic!("{other:?}"),
        };
        let p = || "p".to_string();
        assert_eq!(scalar(Metric::Final { series: p() }), 10.0);
        assert_eq!(
            scalar(Metric::WindowMean {
                series: p(),
                from: 4,
                to: None
            }),
            7.0
        );
        assert_eq!(
            scalar(Metric::WindowMean {
                series: p(),
                from: 4,
                to: Some(6)
            }),
            5.0
        );
        assert_eq!(
            scalar(Metric::WindowMean {
                series: p(),
                from: 0,
                to: Some(0)
            }),
            0.0
        );
        assert_eq!(
            measure(
                &Metric::Timeseries {
                    series: p(),
                    every: 4
                },
                10,
                &history
            ),
            Outcome::Series {
                values: vec![2.5, 6.5, 9.5]
            }
        );
        assert_eq!(blocks(10, 4), vec![(1, 4), (5, 8), (9, 10)]);
        assert_eq!(blocks(10, 10), vec![(1, 10)]);
        assert_eq!(blocks(3, 1), vec![(1, 1), (2, 2), (3, 3)]);
    }

    #[test]
    fn nan_values_are_skipped_and_an_all_nan_block_is_nan() {
        let mut history: Vec<f64> = (0..=10).map(f64::from).collect();
        history[5] = f64::NAN;
        let window = Metric::WindowMean {
            series: "p".into(),
            from: 4,
            to: Some(6),
        };
        assert_eq!(
            measure(&window, 10, &history),
            Outcome::Scalar { value: 5.0 }
        );
        history[9] = f64::NAN;
        history[10] = f64::NAN;
        let Outcome::Series { values } = measure(
            &Metric::Timeseries {
                series: "p".into(),
                every: 4,
            },
            10,
            &history,
        ) else {
            panic!("series expected");
        };
        assert_eq!(&values[..2], &[2.5, 7.0]);
        assert!(values[2].is_nan());
    }

    #[test]
    fn run_results_write_nan_as_null() {
        let run = RunResult {
            point: 3,
            series: 1,
            x: 0,
            seed: 7,
            outcome: Outcome::Scalar { value: f64::NAN },
        };
        let json = serde_json::to_string(&run).unwrap();
        assert_eq!(
            json,
            r#"{"point":3,"series":1,"x":0,"seed":7,"value":null}"#
        );
        let back: RunResult = serde_json::from_str(&json).unwrap();
        assert!(matches!(back.outcome, Outcome::Scalar { value } if value.is_nan()));
        let series = RunResult {
            outcome: Outcome::Series {
                values: vec![1.5, f64::NAN],
            },
            ..run
        };
        let json = serde_json::to_string(&series).unwrap();
        assert_eq!(
            json,
            r#"{"point":3,"series":1,"x":0,"seed":7,"values":[1.5,null]}"#
        );
        let back: RunResult = serde_json::from_str(&json).unwrap();
        let Outcome::Series { values } = back.outcome else {
            panic!("series expected")
        };
        assert_eq!(values[0], 1.5);
        assert!(values[1].is_nan());
        let int: RunResult =
            serde_json::from_str(r#"{"point":0,"series":0,"x":0,"seed":1,"value":224}"#).unwrap();
        assert_eq!(int.outcome, Outcome::Scalar { value: 224.0 });
    }

    #[test]
    fn run_point_measures_one_world() {
        let s = sweep(tiny());
        let point = s.point(7).unwrap();
        let run = run_point(&s, &point);
        assert_eq!((run.point, run.series, run.x, run.seed), (7, 1, 0, 6));
        let mut w = World::new(s.config_for(&point).unwrap(), 6).unwrap();
        w.run(20);
        let pops = w.stats.series("population").unwrap();
        let expected = pops[10..=20].iter().fold(0.0, |a, b| a + b) / 11.0;
        assert_eq!(run.outcome, Outcome::Scalar { value: expected });
    }

    #[test]
    fn an_empty_world_still_yields_a_value() {
        let mut v = tiny();
        v["set"] = json!({ "population": 0 });
        v["metric"] = json!({ "kind": "final", "series": "population" });
        let s = sweep(v);
        assert_eq!(
            run_point(&s, &s.point(0).unwrap()).outcome,
            Outcome::Scalar { value: 0.0 }
        );
    }
}
