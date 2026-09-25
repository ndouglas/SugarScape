//! Parameter sweeps (milestone 5): a grid of config values × seeds, each run
//! summarized by one statistic. The CLI and the browser both call this
//! module; see docs/superpowers/specs/2026-09-23-experiments-design.md.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::FieldError;
use crate::model::{ModelConfig, ModelWorld};
use crate::presets;

pub const MAX_X_VALUES: usize = 64;
pub const MAX_SERIES_VALUES: usize = 16;
pub const MAX_SEEDS: u32 = 100;
pub const MAX_TICKS: u32 = 100_000;
pub const MAX_POINTS: usize = 10_000;
/// At most this many blocks per `timeseries` run: ceil(ticks / every).
pub const MAX_BLOCKS: u32 = 2_000;

/// Where every run's config starts: a preset of any model, or a config of any
/// model (a sugarscape config in either shape).
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
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
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
                    *every == 0 || self.ticks.div_ceil(*every) <= MAX_BLOCKS,
                    "metric.every",
                    format!("at most {MAX_BLOCKS} blocks (ticks / every, rounded up)"),
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
    fn prepare(&self) -> Result<(Vec<Point>, Vec<ModelConfig>), Vec<FieldError>> {
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
    pub fn config_for(&self, point: &Point) -> Result<ModelConfig, Vec<FieldError>> {
        self.config_at(point.series, point.x)
    }

    fn base_config(&self) -> Result<ModelConfig, Vec<FieldError>> {
        match &self.base {
            Base::Preset(id) => presets::find(id)
                .map(|p| p.config)
                .ok_or_else(|| vec![FieldError::new("base", format!("unknown preset {id:?}"))]),
            Base::Config(value) => ModelConfig::from_value(value.clone())
                .map_err(|e| vec![FieldError::new("base", e.message)]),
        }
    }

    /// `base`, then `set`, then the series value's `set`, then the x value's
    /// `set` (each in key order), then `validate` and the metric's series.
    fn config_at(&self, series: usize, x: usize) -> Result<ModelConfig, Vec<FieldError>> {
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
        if !config.series_names().iter().any(|n| n == name) {
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

/// Runs `point` with an already-built `config` and measures it. The config must be
/// the point's valid config (e.g. from `Sweep::config_for`); an invalid config panics.
pub fn run_config(sweep: &Sweep, point: &Point, config: ModelConfig) -> RunResult {
    let mut world = ModelWorld::new(config, point.seed).expect("sweep configs are validated");
    world.model_mut().run(sweep.ticks);
    let history = world
        .model()
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

/// One (series, x) cell of a scalar metric, over seeds (Decision 7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScalarRow {
    pub series: usize,
    pub series_name: String,
    pub x: usize,
    pub at: f64,
    /// Runs with a finite value.
    pub n: usize,
    /// Runs whose value is NaN (excluded from the statistics).
    pub nan: usize,
    #[serde(with = "nan_as_null")]
    pub mean: f64,
    /// Sample standard deviation (n − 1); 0 when n = 1.
    #[serde(with = "nan_as_null")]
    pub sd: f64,
    #[serde(with = "nan_as_null")]
    pub min: f64,
    #[serde(with = "nan_as_null")]
    pub max: f64,
}

/// One block of one line of a `timeseries` metric, over seeds.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockRow {
    pub series: usize,
    pub series_name: String,
    /// The block's last tick.
    pub t: u32,
    pub n: usize,
    #[serde(with = "nan_as_null")]
    pub mean: f64,
    #[serde(with = "nan_as_null")]
    pub sd: f64,
}

/// Rows in series order, then x (or block) order; one per cell even when
/// it has no runs yet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "rows", rename_all = "snake_case")]
pub enum Summary {
    Scalar(Vec<ScalarRow>),
    Timeseries(Vec<BlockRow>),
}

struct Moments {
    n: usize,
    mean: f64,
    sd: f64,
    min: f64,
    max: f64,
}

/// Statistics of the finite values, summed in the given order.
fn moments(values: &[f64]) -> Moments {
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let n = finite.len();
    if n == 0 {
        return Moments {
            n,
            mean: f64::NAN,
            sd: f64::NAN,
            min: f64::NAN,
            max: f64::NAN,
        };
    }
    let mean = finite.iter().fold(0.0, |a, b| a + b) / n as f64;
    let sd = if n == 1 {
        0.0
    } else {
        (finite.iter().fold(0.0, |a, v| a + (v - mean).powi(2)) / (n - 1) as f64).sqrt()
    };
    Moments {
        n,
        mean,
        sd,
        min: finite.iter().copied().fold(f64::INFINITY, f64::min),
        max: finite.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    }
}

/// Summarizes `runs` (any order, possibly partial) per cell. Runs are
/// sorted by point first, so the result does not depend on completion order.
pub fn aggregate(sweep: &Sweep, runs: &[RunResult]) -> Summary {
    let mut sorted: Vec<&RunResult> = runs.iter().collect();
    sorted.sort_by_key(|r| r.point);
    let cell = |series: usize, x: usize| {
        sorted
            .iter()
            .filter(move |r| r.series == series && r.x == x)
    };
    match &sweep.metric {
        Metric::Timeseries { every, .. } => {
            let mut rows = Vec::new();
            for series in 0..sweep.series_count() {
                let series_name = sweep.series_name(series);
                for (k, &(_, t)) in blocks(sweep.ticks, *every).iter().enumerate() {
                    let values: Vec<f64> = cell(series, 0)
                        .filter_map(|r| match &r.outcome {
                            Outcome::Series { values } => values.get(k).copied(),
                            Outcome::Scalar { .. } => None,
                        })
                        .collect();
                    let m = moments(&values);
                    rows.push(BlockRow {
                        series,
                        series_name: series_name.clone(),
                        t,
                        n: m.n,
                        mean: m.mean,
                        sd: m.sd,
                    });
                }
            }
            Summary::Timeseries(rows)
        }
        Metric::Final { .. } | Metric::WindowMean { .. } => {
            let mut rows = Vec::new();
            for series in 0..sweep.series_count() {
                let series_name = sweep.series_name(series);
                for (x, value) in sweep.x.values.iter().enumerate() {
                    let values: Vec<f64> = cell(series, x)
                        .filter_map(|r| match &r.outcome {
                            Outcome::Scalar { value } => Some(*value),
                            Outcome::Series { .. } => None,
                        })
                        .collect();
                    let m = moments(&values);
                    rows.push(ScalarRow {
                        series,
                        series_name: series_name.clone(),
                        x,
                        at: value.at,
                        n: m.n,
                        nan: values.len() - m.n,
                        mean: m.mean,
                        sd: m.sd,
                        min: m.min,
                        max: m.max,
                    });
                }
            }
            Summary::Scalar(rows)
        }
    }
}

/// Errors (field `runs[i]`) for runs that are not this sweep's: a point out
/// of range, a series/x/seed that is not the point's, or the wrong kind or
/// number of values for the metric, or a second run of a point.
pub fn check_runs(sweep: &Sweep, runs: &[RunResult]) -> Result<(), Vec<FieldError>> {
    sweep.check_shape()?;
    let count = sweep.point_count();
    let block_count = match &sweep.metric {
        Metric::Timeseries { every, .. } => Some(blocks(sweep.ticks, *every).len()),
        Metric::Final { .. } | Metric::WindowMean { .. } => None,
    };
    let mut seen = vec![false; count];
    let mut errors: Vec<FieldError> = runs
        .iter()
        .enumerate()
        .filter(|(_, r)| {
            let belongs = r.point < count && {
                let p = sweep.point_at(r.point);
                (p.series, p.x, p.seed) == (r.series, r.x, r.seed)
            };
            let shaped = match (&r.outcome, block_count) {
                (Outcome::Scalar { .. }, None) => true,
                (Outcome::Series { values }, Some(n)) => values.len() == n,
                _ => false,
            };
            !(belongs && shaped)
        })
        .map(|(i, _)| {
            FieldError::new(
                format!("runs[{i}]"),
                "does not match the sweep's points or metric",
            )
        })
        .collect();
    for (i, r) in runs.iter().enumerate() {
        if r.point < count && std::mem::replace(&mut seen[r.point], true) {
            errors.push(FieldError::new(
                format!("runs[{i}]"),
                format!("a second run of point {}", r.point),
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub const RESULT_VERSION: u32 = 1;

/// The result file: `{ version, sweep, runs, summary, incomplete? }`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SweepResult {
    pub version: u32,
    pub sweep: Sweep,
    pub runs: Vec<RunResult>,
    pub summary: Summary,
    /// Written only when some points have no run (Decision 10).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub incomplete: bool,
}

impl SweepResult {
    /// Sorts `runs` by point and summarizes them.
    pub fn new(sweep: Sweep, mut runs: Vec<RunResult>) -> Self {
        runs.sort_by_key(|r| r.point);
        let summary = aggregate(&sweep, &runs);
        let incomplete = runs.len() < sweep.point_count();
        Self {
            version: RESULT_VERSION,
            sweep,
            runs,
            summary,
            incomplete,
        }
    }

    /// Pretty JSON with a trailing newline (the CLI's output and the
    /// browser's download).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("results serialize") + "\n"
    }
}

/// A CSV number: shortest round-trip form; NaN is an empty field.
fn csv_number(v: f64) -> String {
    if v.is_finite() {
        v.to_string()
    } else {
        String::new()
    }
}

/// Quotes a field containing a comma, quote or newline.
fn csv_text(s: &str) -> String {
    if s.contains([',', '"', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// `series,x,seed,value` (x = the x value's `at`), or
/// `series,x,seed,t,value` with one row per block.
pub fn runs_csv(result: &SweepResult) -> String {
    let sweep = &result.sweep;
    let at = |x: usize| sweep.x.values[x].at;
    let mut out = String::new();
    match &sweep.metric {
        Metric::Timeseries { every, .. } => {
            out.push_str("series,x,seed,t,value\n");
            let blocks = blocks(sweep.ticks, *every);
            for r in &result.runs {
                if let Outcome::Series { values } = &r.outcome {
                    for (&(_, t), v) in blocks.iter().zip(values) {
                        writeln!(
                            out,
                            "{},{},{},{},{}",
                            r.series,
                            at(r.x),
                            r.seed,
                            t,
                            csv_number(*v)
                        )
                        .unwrap();
                    }
                }
            }
        }
        Metric::Final { .. } | Metric::WindowMean { .. } => {
            out.push_str("series,x,seed,value\n");
            for r in &result.runs {
                if let Outcome::Scalar { value } = &r.outcome {
                    writeln!(
                        out,
                        "{},{},{},{}",
                        r.series,
                        at(r.x),
                        r.seed,
                        csv_number(*value)
                    )
                    .unwrap();
                }
            }
        }
    }
    out
}

/// `series,series_name,x,n,mean,sd,min,max` or `series,series_name,t,n,mean,sd`.
pub fn summary_csv(result: &SweepResult) -> String {
    let mut out = String::new();
    match &result.summary {
        Summary::Scalar(rows) => {
            out.push_str("series,series_name,x,n,mean,sd,min,max\n");
            for r in rows {
                writeln!(
                    out,
                    "{},{},{},{},{},{},{},{}",
                    r.series,
                    csv_text(&r.series_name),
                    r.at,
                    r.n,
                    csv_number(r.mean),
                    csv_number(r.sd),
                    csv_number(r.min),
                    csv_number(r.max)
                )
                .unwrap();
            }
        }
        Summary::Timeseries(rows) => {
            out.push_str("series,series_name,t,n,mean,sd\n");
            for r in rows {
                writeln!(
                    out,
                    "{},{},{},{},{},{}",
                    r.series,
                    csv_text(&r.series_name),
                    r.t,
                    r.n,
                    csv_number(r.mean),
                    csv_number(r.sd)
                )
                .unwrap();
            }
        }
    }
    out
}

/// Runs every point on `jobs` threads and collects the runs in point order,
/// so the result is the same for any `jobs` (Decision 8). `jobs = 1` runs on
/// this thread. `progress(done, point)` is called on this thread after each
/// run, in completion order.
pub fn run_all(
    sweep: &Sweep,
    jobs: usize,
    mut progress: impl FnMut(usize, &Point),
) -> Result<SweepResult, Vec<FieldError>> {
    let (points, configs) = sweep.prepare()?;
    let xs = sweep.x.values.len();
    let config_of = |p: &Point| configs[p.series * xs + p.x].clone();
    let mut slots: Vec<Option<RunResult>> = (0..points.len()).map(|_| None).collect();
    let jobs = jobs.clamp(1, points.len());
    if jobs == 1 {
        for (done, point) in points.iter().enumerate() {
            slots[point.index] = Some(run_config(sweep, point, config_of(point)));
            progress(done + 1, point);
        }
    } else {
        let next = AtomicUsize::new(0);
        let (tx, rx) = mpsc::channel();
        std::thread::scope(|scope| {
            for _ in 0..jobs {
                let tx = tx.clone();
                let (next, points, config_of) = (&next, &points, &config_of);
                scope.spawn(move || {
                    while let Some(point) = points.get(next.fetch_add(1, Ordering::Relaxed)) {
                        if tx.send(run_config(sweep, point, config_of(point))).is_err() {
                            break;
                        }
                    }
                });
            }
            drop(tx);
            for (done, run) in rx.iter().enumerate() {
                let index = run.point;
                slots[index] = Some(run);
                progress(done + 1, &points[index]);
            }
        });
    }
    let runs = slots
        .into_iter()
        .map(|r| r.expect("every point ran"))
        .collect();
    Ok(SweepResult::new(sweep.clone(), runs))
}

/// A built-in sweep: `sweeps/<id>.json` at the repository root (Decision 11).
pub struct Builtin {
    pub id: &'static str,
    pub json: &'static str,
}

const BUILTINS: [Builtin; 5] = [
    Builtin {
        id: "fig-ii-5",
        json: include_str!("../../../sweeps/fig-ii-5.json"),
    },
    Builtin {
        id: "fig-iv-6",
        json: include_str!("../../../sweeps/fig-iv-6.json"),
    },
    Builtin {
        id: "fig-iv-10-11",
        json: include_str!("../../../sweeps/fig-iv-10-11.json"),
    },
    Builtin {
        id: "n-goods-carrying-capacity",
        json: include_str!("../../../sweeps/n-goods-carrying-capacity.json"),
    },
    Builtin {
        id: "bargaining-rules",
        json: include_str!("../../../sweeps/bargaining-rules.json"),
    },
];

/// The built-in sweeps, in display order.
pub fn builtins() -> &'static [Builtin] {
    &BUILTINS
}

/// Built-in sweep `id`, parsed.
pub fn builtin(id: &str) -> Option<Sweep> {
    BUILTINS
        .iter()
        .find(|b| b.id == id)
        .map(|b| Sweep::from_json(b.json).expect("built-in sweeps parse"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::world::World;
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

    /// Point `point`'s config, which must be a sugarscape config.
    fn sugarscape(s: &Sweep, point: &Point) -> Config {
        s.config_for(point)
            .unwrap()
            .sugarscape()
            .cloned()
            .expect("a sugarscape config")
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
    fn tag_length_sweeps_rebuild_the_default_groups() {
        let s = sweep(json!({
            "name": "tags",
            "base": { "preset": "iii-6-culture" },
            "x": { "path": "tag_length", "values": [5, 11] },
            "seeds": { "from": 1, "count": 1 },
            "ticks": 10,
            "metric": { "kind": "final", "series": "population" }
        }));
        let points = s.points().unwrap();
        for point in &points {
            let config = sugarscape(&s, point);
            assert_eq!(
                config.culture.groups,
                crate::config::default_groups(config.tag_length)
            );
        }
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
    fn unknown_metric_and_base_fields_are_parse_errors() {
        let metric = json!({ "kind": "window_mean", "series": "population", "from": 5, "too": 6 });
        let err = Sweep::from_json(&with(tiny(), "metric", metric).to_string()).unwrap_err();
        assert!(err[0].message.contains("unknown field `too`"), "{err:?}");
        let base = json!({ "preset": "ii-2-unit", "extra": 1 });
        assert!(Sweep::from_json(&with(tiny(), "base", base).to_string()).is_err());
    }

    #[test]
    fn configs_apply_set_then_series_then_x() {
        let mut v = tiny();
        v["set"] = json!({ "population": 50, "vision.max": 3, "growback.rate": 2.0 });
        v["series"]["values"][1]["set"]["population"] = json!(60);
        v["series"]["values"][1]["set"]["growback.rate"] = json!(3.0);
        let s = sweep(v);
        let first = sugarscape(&s, &s.point(0).unwrap());
        assert_eq!(
            (first.population, first.vision.min, first.vision.max),
            (50, 1, 2)
        );
        assert_eq!(
            (first.goods[0].metabolism.max, first.growback.rate),
            (1, 2.0)
        );
        let last = sugarscape(&s, &s.point(11).unwrap());
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
        let config = sugarscape(&s, &s.point(0).unwrap());
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
        assert_eq!(fields(single.clone()), Vec::<String>::new());
        // At most 2000 blocks: ceil(ticks / every).
        let mut blocks_ok = with(single.clone(), "ticks", json!(4000));
        blocks_ok["metric"]["every"] = json!(2);
        assert_eq!(fields(blocks_ok), Vec::<String>::new());
        let mut too_many = with(single, "ticks", json!(4001));
        too_many["metric"]["every"] = json!(2);
        has(too_many, "metric.every");
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
        let mut w = World::new(sugarscape(&s, &point), 6).unwrap();
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

    fn scalar_run(point: usize, s: &Sweep, value: f64) -> RunResult {
        let p = s.point(point).unwrap();
        RunResult {
            point,
            series: p.series,
            x: p.x,
            seed: p.seed,
            outcome: Outcome::Scalar { value },
        }
    }

    fn series_run(point: usize, s: &Sweep, values: Vec<f64>) -> RunResult {
        RunResult {
            outcome: Outcome::Series { values },
            ..scalar_run(point, s, 0.0)
        }
    }

    /// `tiny()` with one x value and 8-tick block means (blocks end 8, 16, 20).
    fn tiny_timeseries() -> Sweep {
        let mut v = tiny();
        v["x"] = json!({ "label": "all", "values": [{ "at": 0, "set": {} }] });
        v["metric"] = json!({ "kind": "timeseries", "series": "population", "every": 8 });
        sweep(v)
    }

    #[test]
    fn aggregation_summarizes_each_cell_over_seeds() {
        let mut v = tiny();
        v["seeds"]["count"] = json!(4);
        let s = sweep(v);
        // Cell (0, 0): 1, 2, 3, 4. Cell (0, 1): 5 and NaN. Cell (1, 2): 9.
        let runs = vec![
            scalar_run(3, &s, 4.0),
            scalar_run(0, &s, 1.0),
            scalar_run(2, &s, 3.0),
            scalar_run(1, &s, 2.0),
            scalar_run(4, &s, 5.0),
            scalar_run(5, &s, f64::NAN),
            scalar_run(20, &s, 9.0),
        ];
        let Summary::Scalar(rows) = aggregate(&s, &runs) else {
            panic!("scalar expected");
        };
        assert_eq!(rows.len(), 6);
        let r = &rows[0];
        assert_eq!((r.series, r.x, r.at, r.n, r.nan), (0, 0, 2.0, 4, 0));
        assert_eq!((r.mean, r.min, r.max), (2.5, 1.0, 4.0));
        assert_eq!(r.sd, (5.0f64 / 3.0).sqrt());
        let r = &rows[1];
        assert_eq!(
            (r.n, r.nan, r.mean, r.sd, r.min, r.max),
            (1, 1, 5.0, 0.0, 5.0, 5.0)
        );
        assert!(rows[2].mean.is_nan() && rows[2].n == 0 && rows[2].nan == 0);
        assert_eq!(
            (rows[5].series_name.as_str(), rows[5].n, rows[5].mean),
            ("wide", 1, 9.0)
        );
        let reversed: Vec<RunResult> = runs.iter().rev().cloned().collect();
        assert_eq!(
            serde_json::to_string(&aggregate(&s, &reversed)).unwrap(),
            serde_json::to_string(&aggregate(&s, &runs)).unwrap()
        );
    }

    #[test]
    fn timeseries_aggregate_per_block() {
        let s = tiny_timeseries();
        let runs = vec![
            series_run(0, &s, vec![1.0, 2.0, 3.0]),
            series_run(1, &s, vec![3.0, 4.0, f64::NAN]),
        ];
        let Summary::Timeseries(rows) = aggregate(&s, &runs) else {
            panic!("timeseries expected");
        };
        assert_eq!(rows.len(), 6);
        assert_eq!(
            (rows[0].t, rows[0].n, rows[0].mean, rows[0].sd),
            (8, 2, 2.0, 2.0f64.sqrt())
        );
        assert_eq!(
            (rows[2].t, rows[2].n, rows[2].mean, rows[2].sd),
            (20, 1, 3.0, 0.0)
        );
        assert!(rows[3].mean.is_nan() && rows[3].n == 0 && rows[3].series_name == "wide");
    }

    #[test]
    fn results_serialize_as_documented() {
        let s = sweep(tiny());
        let result = SweepResult::new(
            s.clone(),
            vec![scalar_run(1, &s, 2.0), scalar_run(0, &s, 1.0)],
        );
        assert!(result.incomplete);
        assert_eq!(result.runs[0].point, 0);
        let value: Value = serde_json::from_str(&result.to_json()).unwrap();
        assert_eq!(value["version"], 1);
        assert_eq!(value["incomplete"], true);
        assert_eq!(value["summary"]["kind"], "scalar");
        assert_eq!(
            value["summary"]["rows"][0],
            json!({ "series": 0, "series_name": "Metabolism = 1", "x": 0, "at": 2.0, "n": 2,
                    "nan": 0, "mean": 1.5, "sd": 0.5f64.sqrt(), "min": 1.0, "max": 2.0 })
        );
        assert_eq!(value["summary"]["rows"][1]["mean"], Value::Null);
        assert!(result.to_json().ends_with("}\n"));
        let back: SweepResult = serde_json::from_str(&result.to_json()).unwrap();
        assert_eq!(back.to_json(), result.to_json());
        let complete = SweepResult::new(
            s.clone(),
            (0..12).map(|i| scalar_run(i, &s, i as f64)).collect(),
        );
        assert!(!complete.incomplete);
        assert!(!complete.to_json().contains("incomplete"));
    }

    #[test]
    fn scalar_csvs() {
        let mut v = tiny();
        v["seeds"]["count"] = json!(1);
        v["series"]["values"][1]["name"] = json!("wide, \"5\"");
        let s = sweep(v);
        let runs = (0..6)
            .map(|i| scalar_run(i, &s, if i == 1 { f64::NAN } else { i as f64 * 0.5 }))
            .collect();
        let result = SweepResult::new(s, runs);
        assert_eq!(
            runs_csv(&result),
            "series,x,seed,value\n0,2,5,0\n0,4,5,\n0,6,5,1\n1,2,5,1.5\n1,4,5,2\n1,6,5,2.5\n"
        );
        assert_eq!(
            summary_csv(&result),
            "series,series_name,x,n,mean,sd,min,max\n\
             0,Metabolism = 1,2,1,0,0,0,0\n\
             0,Metabolism = 1,4,0,,,,\n\
             0,Metabolism = 1,6,1,1,0,1,1\n\
             1,\"wide, \"\"5\"\"\",2,1,1.5,0,1.5,1.5\n\
             1,\"wide, \"\"5\"\"\",4,1,2,0,2,2\n\
             1,\"wide, \"\"5\"\"\",6,1,2.5,0,2.5,2.5\n"
        );
    }

    #[test]
    fn timeseries_csvs() {
        let mut s = tiny_timeseries();
        s.seeds.count = 1;
        let runs = vec![
            series_run(0, &s, vec![1.0, 2.0, 3.0]),
            series_run(1, &s, vec![4.0, f64::NAN, 6.0]),
        ];
        let result = SweepResult::new(s, runs);
        assert_eq!(
            runs_csv(&result),
            "series,x,seed,t,value\n0,0,5,8,1\n0,0,5,16,2\n0,0,5,20,3\n1,0,5,8,4\n1,0,5,16,\n1,0,5,20,6\n"
        );
        assert_eq!(
            summary_csv(&result),
            "series,series_name,t,n,mean,sd\n0,Metabolism = 1,8,1,1,0\n0,Metabolism = 1,16,1,2,0\n\
             0,Metabolism = 1,20,1,3,0\n1,wide,8,1,4,0\n1,wide,16,0,,\n1,wide,20,1,6,0\n"
        );
    }

    #[test]
    fn check_runs_rejects_runs_from_another_sweep() {
        let s = sweep(tiny());
        assert!(check_runs(&s, &[scalar_run(3, &s, 1.0)]).is_ok());
        let mut wrong_seed = scalar_run(3, &s, 1.0);
        wrong_seed.seed = 99;
        let beyond = RunResult {
            point: 12,
            ..scalar_run(0, &s, 1.0)
        };
        let wrong_kind = series_run(1, &s, vec![1.0]);
        let e = check_runs(
            &s,
            &[scalar_run(0, &s, 1.0), wrong_seed, beyond, wrong_kind],
        )
        .unwrap_err();
        assert_eq!(
            e.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
            ["runs[1]", "runs[2]", "runs[3]"]
        );
        let t = tiny_timeseries();
        assert!(check_runs(&t, &[series_run(0, &t, vec![1.0, 2.0, 3.0])]).is_ok());
        assert!(check_runs(&t, &[series_run(0, &t, vec![1.0, 2.0])]).is_err());
    }

    #[test]
    fn check_runs_rejects_duplicate_points() {
        let s = sweep(tiny());
        let e = check_runs(
            &s,
            &[
                scalar_run(3, &s, 1.0),
                scalar_run(0, &s, 1.0),
                scalar_run(3, &s, 2.0),
            ],
        )
        .unwrap_err();
        assert_eq!(e.len(), 1, "{e:?}");
        assert_eq!(e[0].field, "runs[2]");
        assert!(e[0].message.contains("point 3"), "{e:?}");
    }

    #[test]
    fn run_all_is_the_same_for_any_number_of_threads() {
        let s = sweep(tiny());
        let mut seen = Vec::new();
        let one = run_all(&s, 1, |done, p| seen.push((done, p.index))).unwrap();
        assert_eq!(seen, (1..=12).zip(0..12).collect::<Vec<_>>());
        let mut count = 0;
        let four = run_all(&s, 4, |done, _| {
            count += 1;
            assert_eq!(done, count);
        })
        .unwrap();
        assert_eq!(count, 12);
        assert_eq!(four.to_json(), one.to_json());
        assert!(!one.incomplete);
        for (i, run) in one.runs.iter().enumerate() {
            assert_eq!(run.point, i);
            assert_eq!(*run, run_point(&s, &s.point(i).unwrap()));
        }
        assert_eq!(run_all(&s, 64, |_, _| {}).unwrap().to_json(), one.to_json());
    }

    #[test]
    fn run_all_reports_invalid_sweeps() {
        let e = run_all(&sweep(with(tiny(), "ticks", json!(0))), 2, |_, _| {}).unwrap_err();
        assert!(e.iter().any(|e| e.field == "ticks"), "{e:?}");
    }

    #[test]
    fn builtin_sweeps_parse_and_validate() {
        let ids: Vec<&str> = builtins().iter().map(|b| b.id).collect();
        assert_eq!(
            ids,
            [
                "fig-ii-5",
                "fig-iv-6",
                "fig-iv-10-11",
                "n-goods-carrying-capacity",
                "bargaining-rules"
            ]
        );
        for b in builtins() {
            let s = builtin(b.id).unwrap();
            let points = s.points().unwrap_or_else(|e| panic!("{}: {e:?}", b.id));
            assert!(!points.is_empty());
            assert!(
                s.description.as_deref().is_some_and(|d| !d.is_empty()),
                "{} needs a description",
                b.id
            );
        }
        assert!(builtin("nope").is_none());
    }

    #[test]
    fn four_goods_with_trade_is_the_n_4_peaks_preset() {
        let s = builtin("n-goods-carrying-capacity").unwrap();
        let x = s.x.values.iter().position(|v| v.at == 4.0).unwrap();
        let point = s
            .points()
            .unwrap()
            .into_iter()
            .find(|p| p.series == 1 && p.x == x)
            .unwrap();
        assert_eq!(
            sugarscape(&s, &point),
            presets::by_id("n-4-peaks").unwrap().config
        );
    }
}
