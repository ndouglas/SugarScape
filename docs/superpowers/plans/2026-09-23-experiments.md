# Experiments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the playground into an instrument: parameter sweeps (a grid of config values × seeds → one summary statistic per run, aggregated per cell) implemented once in `sugarscape-core`, run natively by a new `sugarscape` CLI (`presets`, `sweeps`, `run`, `sweep`) and in the browser by an Experiments view over a pool of module Web Workers, with the book's parametric figures (II-5, IV-6, IV-10/11) and an N-goods carrying-capacity sweep shipped as measured, ready-made sweep files — without changing a single simulation result.

**Architecture:** A new module `sugarscape_core::sweep` owns everything numeric: the serde `Sweep` model (shorthand axes expanded on read), point expansion (series-major, then x, then seed), config building (`base` → `set` → series value → x value via `Config::with_path`, then `validate`), metrics over `Stats` histories, aggregation, the `SweepResult` file, the two CSV writers, `run_all` on std threads with results ordered by point, and the built-in sweeps embedded from `sweeps/*.json`. `crates/sugarscape-cli` is a thin clap front end. `sugarscape-wasm` gains free functions (JSON in, JSON out, errors as `[{field, message}]`). The web app gets `web/src/experiments/`: a worker pool (each worker its own WASM instance, one point at a time), a header switch, a picker, a read-only panel and an editable form, a uPlot chart with ±1 sd bands, exports and `#x=` share links. TypeScript never computes statistics; it maps the core's summary to chart arrays.

**Tech Stack:** Rust (`sugarscape-core`, `sugarscape-wasm`, **`sugarscape-cli` (new) with `clap` 4 derive (new)**, serde, serde_json, std threads), wasm-bindgen, Vite (module workers) + TypeScript + uPlot + Vitest.

**Spec:** docs/superpowers/specs/2026-09-23-experiments-design.md

## Global Constraints

- **No simulation change.** `crates/sugarscape-core/tests/golden.rs` and `tests/legacy.rs` pass, **unedited**, at the end of every task. No file under `crates/sugarscape-core/src/` other than `lib.rs` (one `pub mod sweep;` line) and the new `sweep.rs` is touched. Sweeps only build configs (`Config::with_path`, `Config::validate`) and read `World::stats`.
- **One implementation.** Expansion, config building, metrics, aggregation, the result JSON and both CSVs live in `sugarscape_core::sweep`. The CLI and the WASM functions only parse arguments, call the core and write its output. TypeScript formats and maps the core's summary; it never averages runs.
- **Determinism.** A run is a function of (config, seed). `run_all` and `aggregate` order runs by point index before anything is summed, so output files are byte-identical for any `--jobs` or worker count on one platform. Native and WASM builds may differ in the last bits of `powf`/`ln`; this is documented (Task 16), never hidden.
- **Errors** are `Vec<FieldError>`. WASM throws them as a JSON string `[{field, message}]` (the existing `field_errors`). The CLI prints them one per line as `field: message` on stderr and exits 2; I/O failures exit 1.
- **Seeds reach the browser as `u32`**: a sweep's seeds must stay ≤ 4 294 967 295 (Decision 2), so any run can be reproduced in the playground.
- Every commit message ends with a blank line and then:

  `Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3`

  (the commit commands below pass it as a second `-m`). Stage only the files named in the task.
- Rust tasks end with `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (the whole workspace, which includes `golden.rs` and `legacy.rs`) passing. Tasks that touch `sugarscape-wasm` also run `wasm-pack test --node crates/sugarscape-wasm`. Web tasks end with `(cd web && npm run build && npm test)` passing (the build runs `wasm-pack` first).
- If clippy flags `needless_range_loop` in code below, rewrite that loop with `iter().enumerate()` without changing what is summed or in what order. If it suggests `sum()` for a `fold(0.0, |a, b| a + b)`, accept it: `sum()` adds in the same order (it only starts from `-0.0`, which changes nothing for these values).
- **Nobody browser-checks until the controller's puppeteer pass after Task 16** (the spec's "Browser" test: run a tiny built-in sweep, cancel one mid-run, open a CLI result file). Web tasks are verified by `tsc`, `vite build` and Vitest.

## Why this task order

The suggested order is kept for the core (Tasks 1–4), the built-in sweeps (5), the CLI (6–7) and WASM (8): each layer is tested before anything consumes it, the built-ins exist before `sugarscape sweeps`/`--builtin` and `builtin_sweeps()` need them, and `run_all`'s jobs-independence is proven before the CLI promises byte-identical files. In the browser, the **chart (Task 11) moves ahead of the form (Tasks 12–13)**: the shell (Task 10) can already run built-in sweeps, so the chart has real data to show at once, and the form task's full rewrite of `view.ts` then already contains the chart instead of patching it twice. The book-style tests (Task 15) only read the built-ins; they sit late because they are slow and change nothing that other tasks depend on.

## Decisions (where the spec leaves room)

These are binding for this plan; each is also stated in the task that implements it.

1. **Axes.** The shorthand `{ label?, path, values }` is expanded when read (`#[serde(try_from)]`): value i becomes `{ at, set: { path: v } }` with `at` = v when v is a JSON number, otherwise i; `label` defaults to the path. The full form requires `label`. Axes are always **written** in full form (so result files and `builtin_sweeps()`' re-serializations are unambiguous). `AxisValue.set` defaults to `{}`. `Sweep`, `AxisValue`, `Seeds` and the shorthand reject unknown fields.
2. **Points and limits.** `index = (series · nx + x) · seeds + k`, seed = `seeds.from + k`. Besides the spec's limits (1–64 x values, 1–16 series values, `seeds.count` 1–100, `ticks` 1–100 000, ≤ 10 000 points), `name` must not be blank and the last seed must be ≤ 4 294 967 295 (field `seeds.from`).
3. **Error fields.** Parse errors: field `sweep`. Base errors (unknown preset, unreadable config): `base`. A path that `with_path` cannot set: `set.<path>`, `series[j].set.<path>` or `x[i].set.<path>` with `with_path`'s message (e.g. `unknown field visionn`). A built config that fails `validate`: field `points`, message `point P (x=i, series=j): <field>: <message>` where i, j are **indices** and P is the first point (lowest seed) of that cell. A metric series missing from a cell's `series_names`: field `metric.series` with the same prefix. Errors are deduplicated across cells (so one bad `set` path is reported once).
4. **A `timeseries` metric needs exactly one x value** (field `metric.kind`): the spec's timeseries summary CSV (`series,series_name,t,…`) and chart (x = tick) have no x dimension. `every` must be 1–`ticks`; `window_mean` needs `from ≤ to ≤ ticks` (`to` defaults to `ticks`; fields `metric.from`/`metric.to`).
5. **NaN.** Run values and summary statistics are `f64` in Rust; JSON writes non-finite numbers as `null` and reads `null` back as NaN. CSVs write NaN as an empty field.
6. **Indices vs values.** `RunResult` JSON carries the `series` and `x` **indices** (and `point`, `seed`); CSVs carry x's `at` in the `x` column and the series **index** in `series` (plus `series_name` in the summary). The timeseries runs CSV has columns `series,x,seed,t,value` (one row per block).
7. **Summary JSON** is `{ "kind": "scalar" | "timeseries", "rows": [...] }`. Scalar rows: `series, series_name, x, at, n, nan, mean, sd, min, max`; block rows: `series, series_name, t, n, mean, sd`. Every cell (and every block) gets a row even with `n = 0` (statistics NaN), so partial results chart with a stable shape. `aggregate` sorts runs by point first. With no `series` axis, the single line is named after the metric's statistics series (e.g. `population`).
8. **`run_all(sweep, jobs, progress) -> Result<SweepResult, Vec<FieldError>>`** (the spec's signature had no error or progress): `progress(done, &point)` runs on the calling thread after each run, in completion order. `jobs = 1` runs on the calling thread without spawning (so it also works on wasm32); otherwise `std::thread::scope` workers pull indices from an `AtomicUsize` and send results over an `mpsc` channel. No rayon: a shared counter over ≤ 10 000 coarse runs needs nothing more.
9. **`run_point(&Sweep, &Point) -> RunResult`** panics if the point's config is invalid; callers validate first (`points`, or `point` + `config_for`). Helpers beyond the spec: `Sweep::{from_json, series_count, point_count, series_name, check_shape, point}`, `measure`, `blocks`, `check_runs`, `SweepResult::{new, to_json}`.
10. **`SweepResult.incomplete`** (written only when true) marks results with fewer runs than points (a cancelled browser sweep). `SweepResult::new` sorts runs and recomputes the summary. `to_json` is pretty JSON plus a trailing newline — the CLI's `--out`/stdout and the browser's download are the same function.
11. **Built-in sweep files live at the repository root, `sweeps/<id>.json`**, embedded with `include_str!("../../../sweeps/<id>.json")`; `builtins()` lists `[fig-ii-5, fig-iv-6, fig-iv-10-11, n-goods-carrying-capacity]`.
12. **Mean-trait axes.** "Mean vision m" and "mean metabolism m" are the uniform integer ranges `1..=(2m − 1)` (mean m).
13. **`n-goods-carrying-capacity` sweeps 2–6 goods, not 1–6**: trade needs two goods (`validate`), so the trade line cannot have an x = 1 point. The goods are `n-4-peaks`'s four corner-peak goods, then `tea` at (25, 25) and `wool` at (25, 0), all with `n-4-peaks`'s traits (metabolism 1–2, endowment 25–50); each x value sets `goods` and `pollution.pollutants` (the book pollutant for n goods). With trade at x = 4 the config equals the `n-4-peaks` preset (tested).
14. **Built-in settings are measured** (Task 5): ticks T ∈ {300, 500, 1000}, window `from = T − 100`, seeds 10 → 5 → 3 by a runtime budget; the rules are in Task 5.
15. **CLI.** `presets` prints `id<TAB>source<TAB>name`; `sweeps` prints `id<TAB>name`. An unknown preset or built-in is a validation error (fields `preset`, `builtin`; exit 2). `--config-out` writes the config as loaded (before any scheduled change fires), pretty-printed. `--jobs` is 1–1024. Progress lines are `[done/total] series=<series name> x=<at> seed=<seed>` in completion order on stderr.
16. **WASM.** Besides the spec's `sweep_points`, `run_point`, `aggregate` and `builtin_sweeps`, the crate exports `config_series_names(config)` (the form's statistic list), `sweep_result(spec, runs)` and `sweep_csv(spec, runs, kind)` so the browser's exports come from the core's writers. `aggregate`, `sweep_result` and `sweep_csv` reject runs that do not belong to the sweep (`check_runs`). `builtin_sweeps()` returns `[{ id, sweep }]` with each file's JSON as written.
17. **Browser view.** Switching to Experiments pauses the playground (its world is kept) and hides the header's play/seed controls and the Share/Export menu. The picker lists the built-ins, then "From current world"; "Open file…" is a button. "From current world" uses `{ preset }` when the world is an unmodified preset, otherwise `{ config: baseConfig }` (painted maps are not carried; the form says so), and is captured when chosen.
18. **Form.** Axis 1 is `x`, axis 2 (optional) is `series` (one line per value). Axes are written in shorthand; seeds start at 1. With a `timeseries` metric the form hides the x axis and writes the single x value `{ label: "All runs", values: [{ at: 0, set: {} }] }`. Values are `a, b, c` (all numbers or all `true`/`false`) or `from:to:step` (numbers, step > 0, inclusive, float steps tidied to 12 significant digits). A file or link sweep opens in the form when `sweepToForm` can express it (no `set`, seeds from 1, each axis one path with scalar values at their natural `at`, no line names), otherwise read-only with seeds and ticks editable.
19. **Worker pool.** Workers are created per run and terminated at the end or on Cancel; size `max(1, (hardwareConcurrency ?? 2) − 1)`. The main thread hands out indices in order; the first point error or worker error stops the run. Partial results are re-aggregated at most every 250 ms. Vite builds workers as ES modules (`worker.format = 'es'`), because the wasm-bindgen module locates its `.wasm` with `new URL(…, import.meta.url)`.
20. **Opening a `SweepResult` file** recomputes the summary from its runs with `aggregate` (the file's own `summary` is ignored), so a hand-edited or partial file still charts consistently, and a file whose runs do not match its sweep is rejected.
21. **Chart.** x positions are sorted by `at`. The ±1 sd bands are painted in a uPlot `drawAxes` hook (under the lines, no hidden series in the legend); the y range covers the bands. Line colors are `--c1…--c4`, then four fixed colors, cycling.
22. **`#x=` links** are base64url(deflate-raw(sweep JSON)) with the same codec and 1 MiB inflate cap as `#s=`; "Share link" encodes the sweep currently in the editor (not its results) and `#x=` opens it in Experiments without running.

## File Structure

```
sweeps/                                  NEW  built-in sweeps (Task 5)
  fig-ii-5.json, fig-iv-6.json, fig-iv-10-11.json, n-goods-carrying-capacity.json
crates/sugarscape-core/
  src/sweep.rs        NEW  model, points, configs (1); metrics, run_point (2); aggregation,
                           SweepResult, CSV (3); run_all (4); builtins (5)
  src/lib.rs          MOD  pub mod sweep (1)
  tests/book.rs       MOD  temporary measurement (5, deleted before commit); book-style sweep tests (15)
crates/sugarscape-cli/                   NEW  (6, 7)
  Cargo.toml, src/main.rs, tests/cli.rs
Cargo.toml            MOD  workspace member (6)
Cargo.lock            MOD  clap (6)
crates/sugarscape-wasm/src/lib.rs, tests/web.rs   MOD (8)
web/vite.config.ts    MOD  ES module workers (9)
web/index.html        MOD  #playground and #experiments mains (10)
web/src/main.ts       MOD  view switch (10), view gets the engine (13), #x= links (14)
web/src/style.css     MOD  Experiments styles (10)
web/src/share.ts, share.test.ts          MOD  #x= codec (14)
web/src/experiments/
  types.ts                               NEW  sweep JSON shapes (9)
  pool.ts, pool.test.ts, worker.ts       NEW  worker pool (9)
  labels.ts, labels.test.ts              NEW  axis/metric/base labels (10)
  fixed-panel.ts, results-table.ts, view.ts   NEW  shell (10); view.ts MOD (11, 13, 14)
  chart-data.ts, chart-data.test.ts, chart.ts NEW (11)
  values.ts, values.test.ts, form.ts, form.test.ts   NEW (12)
  form-view.ts                           NEW (13)
  file.ts, file.test.ts                  NEW (14)
README.md, docs/roadmap.md, .github/workflows/ci.yml   MOD (16)
```

---

### Task 1: The sweep model, point expansion and config building

*Mechanical (full code).*

**Files:**
- Create: `crates/sugarscape-core/src/sweep.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`

**Interfaces:**
- Consumes: `Config::{from_value, with_path, validate}`, `FieldError::new`, `presets::by_id`, `stats::series_names` (all unchanged).
- Produces (module `sugarscape_core::sweep`):
  - `pub const MAX_X_VALUES: usize = 64; MAX_SERIES_VALUES: usize = 16; MAX_SEEDS: u32 = 100; MAX_TICKS: u32 = 100_000; MAX_POINTS: usize = 10_000;`
  - `pub enum Base { Preset(String), Config(serde_json::Value) }`
  - `pub struct AxisValue { pub at: f64, pub name: Option<String>, pub set: BTreeMap<String, Value> }` with `pub fn display_name(&self, label: &str) -> String`
  - `pub struct Axis { pub label: String, pub values: Vec<AxisValue> }`
  - `pub struct Seeds { pub from: u64, pub count: u32 }`
  - `pub enum Metric { Final { series }, WindowMean { series, from: u32, to: Option<u32> }, Timeseries { series, every: u32 } }` with `pub fn series(&self) -> &str`
  - `pub struct Sweep { name, description: Option<String>, base: Base, set: BTreeMap<String, Value>, x: Axis, series: Option<Axis>, seeds: Seeds, ticks: u32, metric: Metric }`
  - `pub struct Point { pub index: usize, pub series: usize, pub x: usize, pub seed: u64 }`
  - `impl Sweep`: `pub fn from_json(json: &str) -> Result<Sweep, Vec<FieldError>>`, `pub fn series_count(&self) -> usize`, `pub fn point_count(&self) -> usize`, `pub fn series_name(&self, series: usize) -> String`, `pub fn check_shape(&self) -> Result<(), Vec<FieldError>>`, `pub fn point(&self, index: usize) -> Result<Point, Vec<FieldError>>`, `pub fn points(&self) -> Result<Vec<Point>, Vec<FieldError>>`, `pub fn config_for(&self, point: &Point) -> Result<Config, Vec<FieldError>>`; private `point_at`, `first_point`, `prepare`, `base_config`, `config_at`.

- [ ] **Step 1: Confirm the baseline**

Run: `cargo test -p sugarscape-core --test golden --test legacy`
Expected: golden `2 passed; 0 failed; 1 ignored`, legacy `2 passed`. If anything fails, stop: the branch is not at the expected state.

- [ ] **Step 2: Write the failing tests**

Create `crates/sugarscape-core/src/sweep.rs` with only the test module for now:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 2 series × 3 x values × 2 seeds on ii-2-unit with 50 agents. The CLI
    /// and WASM tests use the same sweep.
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
        let p = |index, series, x, seed| Point { index, series, x, seed };
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
        assert_eq!((first.goods[0].metabolism.max, first.growback.rate), (1, 2.0));
        let last = s.config_for(&s.point(11).unwrap()).unwrap();
        assert_eq!((last.population, last.vision.max), (60, 6));
        assert_eq!((last.goods[0].metabolism.max, last.growback.rate), (5, 3.0));
    }

    #[test]
    fn a_base_config_may_use_either_shape() {
        let legacy = with(without(tiny(), "set"), "base", json!({ "config": { "population": 100 } }));
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
        has(with(tiny(), "seeds", json!({ "from": 1, "count": 0 })), "seeds.count");
        has(with(tiny(), "seeds", json!({ "from": 1, "count": 101 })), "seeds.count");
        has(
            with(tiny(), "seeds", json!({ "from": 4_294_967_295u64, "count": 2 })),
            "seeds.from",
        );
        assert!(sweep(with(tiny(), "seeds", json!({ "from": 4_294_967_294u64, "count": 2 })))
            .check_shape()
            .is_ok());
        has(
            with(tiny(), "x", json!({ "path": "vision.max", "values": vec![2; 65] })),
            "x.values",
        );
        has(
            with(tiny(), "x", json!({ "path": "vision.max", "values": [] })),
            "x.values",
        );
        has(
            with(tiny(), "series", json!({ "path": "growback.rate", "values": vec![1.0; 17] })),
            "series.values",
        );
        let mut big = with(tiny(), "x", json!({ "path": "vision.max", "values": vec![2; 64] }));
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
        let ts = |every: u32| json!({ "kind": "timeseries", "series": "population", "every": every });
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
}
```

Add to `crates/sugarscape-core/src/lib.rs`, after `pub mod stats;`:
```rust
pub mod sweep;
```

Run: `cargo test -p sugarscape-core sweep::`
Expected: compile errors (`Sweep`, `Point`, … not found).

- [ ] **Step 3: Implement**

Insert above the test module in `crates/sugarscape-core/src/sweep.rs`:
```rust
//! Parameter sweeps (milestone 5): a grid of config values × seeds, each run
//! summarized by one statistic. The CLI and the browser both call this
//! module; see docs/superpowers/specs/2026-09-23-experiments-design.md.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::config::{Config, FieldError};
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
            let label = input
                .label
                .ok_or("an axis without a path needs a label")?;
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
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core sweep::`
Expected: all 12 tests pass. (`x[1].set.visionn` works because `with_path` reports `unknown field <path>` with field `schedule`; only its message is kept.)

- [ ] **Step 5: Verify**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
```
Expected: everything passes, including `golden` and `legacy` (unedited).

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/sweep.rs crates/sugarscape-core/src/lib.rs
git commit -m "Add the sweep model: axes, point expansion and config building" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 2: Metrics and single runs

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/sweep.rs`

**Interfaces:**
- Consumes: Task 1's `Sweep`, `Point`, `Metric`, `config_for`; `World::{new, run, stats}`, `Stats::series`.
- Produces:
  - `pub enum Outcome { Scalar { value: f64 }, Series { values: Vec<f64> } }` (serde untagged; NaN ↔ `null`)
  - `pub struct RunResult { pub point: usize, pub series: usize, pub x: usize, pub seed: u64, #[serde(flatten)] pub outcome: Outcome }`
  - `pub fn blocks(ticks: u32, every: u32) -> Vec<(u32, u32)>`
  - `pub fn measure(metric: &Metric, ticks: u32, history: &[f64]) -> Outcome`
  - `pub fn run_point(sweep: &Sweep, point: &Point) -> RunResult` (panics on an invalid point config, Decision 9)
  - private `fn run_config(sweep: &Sweep, point: &Point, config: Config) -> RunResult`, `fn finite_mean(values: &[f64]) -> f64`, modules `nan_as_null`, `nan_as_null_vec`.

- [ ] **Step 1: Write the failing tests** (append inside `mod tests`)

```rust
    #[test]
    fn metrics_read_the_history_by_tick() {
        let history: Vec<f64> = (0..=10).map(f64::from).collect();
        let scalar = |m: Metric| match measure(&m, 10, &history) {
            Outcome::Scalar { value } => value,
            other => panic!("{other:?}"),
        };
        let p = || "p".to_string();
        assert_eq!(scalar(Metric::Final { series: p() }), 10.0);
        assert_eq!(scalar(Metric::WindowMean { series: p(), from: 4, to: None }), 7.0);
        assert_eq!(scalar(Metric::WindowMean { series: p(), from: 4, to: Some(6) }), 5.0);
        assert_eq!(scalar(Metric::WindowMean { series: p(), from: 0, to: Some(0) }), 0.0);
        assert_eq!(
            measure(&Metric::Timeseries { series: p(), every: 4 }, 10, &history),
            Outcome::Series { values: vec![2.5, 6.5, 9.5] }
        );
        assert_eq!(blocks(10, 4), vec![(1, 4), (5, 8), (9, 10)]);
        assert_eq!(blocks(10, 10), vec![(1, 10)]);
        assert_eq!(blocks(3, 1), vec![(1, 1), (2, 2), (3, 3)]);
    }

    #[test]
    fn nan_values_are_skipped_and_an_all_nan_block_is_nan() {
        let mut history: Vec<f64> = (0..=10).map(f64::from).collect();
        history[5] = f64::NAN;
        let window = Metric::WindowMean { series: "p".into(), from: 4, to: Some(6) };
        assert_eq!(measure(&window, 10, &history), Outcome::Scalar { value: 5.0 });
        history[9] = f64::NAN;
        history[10] = f64::NAN;
        let Outcome::Series { values } =
            measure(&Metric::Timeseries { series: "p".into(), every: 4 }, 10, &history)
        else {
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
        assert_eq!(json, r#"{"point":3,"series":1,"x":0,"seed":7,"value":null}"#);
        let back: RunResult = serde_json::from_str(&json).unwrap();
        assert!(matches!(back.outcome, Outcome::Scalar { value } if value.is_nan()));
        let series = RunResult {
            outcome: Outcome::Series { values: vec![1.5, f64::NAN] },
            ..run
        };
        let json = serde_json::to_string(&series).unwrap();
        assert_eq!(json, r#"{"point":3,"series":1,"x":0,"seed":7,"values":[1.5,null]}"#);
        let back: RunResult = serde_json::from_str(&json).unwrap();
        let Outcome::Series { values } = back.outcome else { panic!("series expected") };
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
```

Run: `cargo test -p sugarscape-core sweep::`
Expected: compile errors (`measure`, `Outcome`, … not found).

- [ ] **Step 2: Implement**

In the `use` block at the top of `sweep.rs`, add after `use crate::config::{Config, FieldError};`:
```rust
use crate::world::World;
```

Append after `impl Sweep { … }` (above `#[cfg(test)]`):
```rust
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
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p sugarscape-core sweep::`
Expected: 17 passed.

- [ ] **Step 4: Verify** — `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test`

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/sweep.rs
git commit -m "Measure sweep runs: final value, window mean and block means" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 3: Aggregation, the result file and CSV

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/sweep.rs`

**Interfaces:**
- Consumes: Tasks 1–2 (`Sweep`, `RunResult`, `Outcome`, `blocks`, `point_at`, `nan_as_null`).
- Produces:
  - `pub struct ScalarRow { series, series_name: String, x: usize, at: f64, n: usize, nan: usize, mean, sd, min, max: f64 }`
  - `pub struct BlockRow { series, series_name: String, t: u32, n: usize, mean, sd: f64 }`
  - `pub enum Summary { Scalar(Vec<ScalarRow>), Timeseries(Vec<BlockRow>) }` (`{"kind", "rows"}`)
  - `pub fn aggregate(sweep: &Sweep, runs: &[RunResult]) -> Summary`
  - `pub fn check_runs(sweep: &Sweep, runs: &[RunResult]) -> Result<(), Vec<FieldError>>`
  - `pub const RESULT_VERSION: u32 = 1;` `pub struct SweepResult { version, sweep, runs, summary, incomplete: bool }` with `pub fn new(sweep: Sweep, runs: Vec<RunResult>) -> SweepResult` and `pub fn to_json(&self) -> String`
  - `pub fn runs_csv(result: &SweepResult) -> String`, `pub fn summary_csv(result: &SweepResult) -> String`

- [ ] **Step 1: Write the failing tests** (append inside `mod tests`)

```rust
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
        assert_eq!((r.n, r.nan, r.mean, r.sd, r.min, r.max), (1, 1, 5.0, 0.0, 5.0, 5.0));
        assert!(rows[2].mean.is_nan() && rows[2].n == 0 && rows[2].nan == 0);
        assert_eq!((rows[5].series_name.as_str(), rows[5].n, rows[5].mean), ("wide", 1, 9.0));
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
        assert_eq!((rows[0].t, rows[0].n, rows[0].mean, rows[0].sd), (8, 2, 2.0, 2.0f64.sqrt()));
        assert_eq!((rows[2].t, rows[2].n, rows[2].mean, rows[2].sd), (20, 1, 3.0, 0.0));
        assert!(rows[3].mean.is_nan() && rows[3].n == 0 && rows[3].series_name == "wide");
    }

    #[test]
    fn results_serialize_as_documented() {
        let s = sweep(tiny());
        let result = SweepResult::new(s.clone(), vec![scalar_run(1, &s, 2.0), scalar_run(0, &s, 1.0)]);
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
        let complete = SweepResult::new(s.clone(), (0..12).map(|i| scalar_run(i, &s, i as f64)).collect());
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
        let beyond = RunResult { point: 12, ..scalar_run(0, &s, 1.0) };
        let wrong_kind = series_run(1, &s, vec![1.0]);
        let e = check_runs(&s, &[scalar_run(0, &s, 1.0), wrong_seed, beyond, wrong_kind]).unwrap_err();
        assert_eq!(
            e.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
            ["runs[1]", "runs[2]", "runs[3]"]
        );
        let t = tiny_timeseries();
        assert!(check_runs(&t, &[series_run(0, &t, vec![1.0, 2.0, 3.0])]).is_ok());
        assert!(check_runs(&t, &[series_run(0, &t, vec![1.0, 2.0])]).is_err());
    }
```

Run: `cargo test -p sugarscape-core sweep::` — Expected: compile errors.

- [ ] **Step 2: Implement**

In the `use` block, add as the first line (before `use std::collections::BTreeMap;`):
```rust
use std::fmt::Write;
```

Append after `run_config` (above `#[cfg(test)]`):
```rust
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
/// number of values for the metric.
pub fn check_runs(sweep: &Sweep, runs: &[RunResult]) -> Result<(), Vec<FieldError>> {
    sweep.check_shape()?;
    let count = sweep.point_count();
    let block_count = match &sweep.metric {
        Metric::Timeseries { every, .. } => Some(blocks(sweep.ticks, *every).len()),
        Metric::Final { .. } | Metric::WindowMean { .. } => None,
    };
    let errors: Vec<FieldError> = runs
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
                        writeln!(out, "{},{},{},{},{}", r.series, at(r.x), r.seed, t, csv_number(*v))
                            .unwrap();
                    }
                }
            }
        }
        Metric::Final { .. } | Metric::WindowMean { .. } => {
            out.push_str("series,x,seed,value\n");
            for r in &result.runs {
                if let Outcome::Scalar { value } = &r.outcome {
                    writeln!(out, "{},{},{},{}", r.series, at(r.x), r.seed, csv_number(*value))
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
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p sugarscape-core sweep::`
Expected: 23 passed. (`0.0f64.to_string()` is `"0"` and `2.0` is `"2"`, which the CSV expectations rely on.)

- [ ] **Step 4: Verify** — `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test`

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/sweep.rs
git commit -m "Aggregate sweep runs and write the result file and CSVs" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 4: Running a sweep on threads

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-core/src/sweep.rs`

**Interfaces:**
- Consumes: `Sweep::prepare`, `run_config`, `SweepResult::new`.
- Produces: `pub fn run_all(sweep: &Sweep, jobs: usize, progress: impl FnMut(usize, &Point)) -> Result<SweepResult, Vec<FieldError>>` (Decision 8).

- [ ] **Step 1: Write the failing tests** (append inside `mod tests`)

```rust
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
```

Run: `cargo test -p sugarscape-core sweep::` — Expected: compile error (`run_all` not found).

- [ ] **Step 2: Implement**

In the `use` block, add after `use std::fmt::Write;`:
```rust
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
```

Append after `summary_csv` (above `#[cfg(test)]`):
```rust
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
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p sugarscape-core sweep::`
Expected: 25 passed.

- [ ] **Step 4: Verify** — `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test`, then confirm the core still builds for the browser: `cargo build -p sugarscape-core --target wasm32-unknown-unknown` (std threads compile there; only `jobs > 1` would fail at run time, and WASM never asks for it).

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/sweep.rs
git commit -m "Run sweeps on threads with results in point order" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 5: Built-in sweeps, measured

*Needs judgement: ticks, windows and seed counts come from measurement.*

**Files:**
- Create: `sweeps/fig-ii-5.json`, `sweeps/fig-iv-6.json`, `sweeps/fig-iv-10-11.json`, `sweeps/n-goods-carrying-capacity.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`
- Modify temporarily (not committed): `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: `run_all`, `measure`, `Metric`, `Sweep::{points, config_for, series_name}`, `summary_csv`, presets `ii-2-unit`, `iv-3-trade`, `n-4-peaks`.
- Produces: `pub struct Builtin { pub id: &'static str, pub json: &'static str }`, `pub fn builtins() -> &'static [Builtin]`, `pub fn builtin(id: &str) -> Option<Sweep>`; the four files (Decisions 11–14).

- [ ] **Step 1: Write the three hand-written files with their starting settings**

These are **starting values** that Step 5 replaces with measured ones; the `description`s are rewritten in Step 6.

`sweeps/fig-ii-5.json`:
```json
{
  "name": "Figure II-5: carrying capacity vs vision and metabolism",
  "description": "Starting settings; measured in the experiments plan, Task 5.",
  "base": { "preset": "ii-2-unit" },
  "set": { "population": 500 },
  "x": {
    "label": "Mean vision",
    "values": [
      { "at": 1, "set": { "vision": { "min": 1, "max": 1 } } },
      { "at": 2, "set": { "vision": { "min": 1, "max": 3 } } },
      { "at": 3, "set": { "vision": { "min": 1, "max": 5 } } },
      { "at": 4, "set": { "vision": { "min": 1, "max": 7 } } },
      { "at": 5, "set": { "vision": { "min": 1, "max": 9 } } },
      { "at": 6, "set": { "vision": { "min": 1, "max": 11 } } }
    ]
  },
  "series": {
    "label": "Mean metabolism",
    "values": [
      { "at": 1, "set": { "goods.0.metabolism": { "min": 1, "max": 1 } } },
      { "at": 2, "set": { "goods.0.metabolism": { "min": 1, "max": 3 } } },
      { "at": 3, "set": { "goods.0.metabolism": { "min": 1, "max": 5 } } }
    ]
  },
  "seeds": { "from": 1, "count": 10 },
  "ticks": 500,
  "metric": { "kind": "window_mean", "series": "population", "from": 400 }
}
```

`sweeps/fig-iv-6.json`:
```json
{
  "name": "Figure IV-6: carrying capacity with and without trade",
  "description": "Starting settings; measured in the experiments plan, Task 5.",
  "base": { "preset": "iv-3-trade" },
  "x": {
    "label": "Mean vision",
    "values": [
      { "at": 1, "set": { "vision": { "min": 1, "max": 1 } } },
      { "at": 2, "set": { "vision": { "min": 1, "max": 3 } } },
      { "at": 3, "set": { "vision": { "min": 1, "max": 5 } } },
      { "at": 4, "set": { "vision": { "min": 1, "max": 7 } } },
      { "at": 5, "set": { "vision": { "min": 1, "max": 9 } } },
      { "at": 6, "set": { "vision": { "min": 1, "max": 11 } } }
    ]
  },
  "series": {
    "label": "Trade",
    "values": [
      { "at": 0, "name": "No trade", "set": { "trade.enabled": false } },
      { "at": 1, "name": "Trade", "set": { "trade.enabled": true } }
    ]
  },
  "seeds": { "from": 1, "count": 10 },
  "ticks": 500,
  "metric": { "kind": "window_mean", "series": "population", "from": 400 }
}
```

`sweeps/fig-iv-10-11.json`:
```json
{
  "name": "Figures IV-10 and IV-11: price dispersion vs lifetime",
  "description": "Starting settings; measured in the experiments plan, Task 5.",
  "base": { "preset": "iv-3-trade" },
  "set": { "lifespan.enabled": true, "replacement.enabled": true },
  "x": { "label": "All runs", "values": [ { "at": 0, "set": {} } ] },
  "series": {
    "label": "Lifetimes",
    "values": [
      { "at": 60, "name": "Lifetimes 60–100", "set": { "lifespan.max_age": { "min": 60, "max": 100 } } },
      { "at": 960, "name": "Lifetimes 960–1000", "set": { "lifespan.max_age": { "min": 960, "max": 1000 } } }
    ]
  },
  "seeds": { "from": 1, "count": 10 },
  "ticks": 1000,
  "metric": { "kind": "timeseries", "series": "sd_log_price", "every": 50 }
}
```

- [ ] **Step 2: Generate the N-goods file**

Its x values carry whole goods lists (Decision 13), so it is generated rather than typed. Run from the repository root (the script is not committed):
```bash
node -e '
const fs = require("fs");
const good = ([name, color, x, y]) => ({
  name, color,
  map: { kind: "peaks", peaks: [{ x, y, radius: 20, height: 4 }] },
  metabolism: { min: 1, max: 2 },
  endowment: { min: 25, max: 50 },
});
const goods = [
  ["sugar", "#f2c14e", 10, 10], ["spice", "#e07a3f", 39, 10], ["salt", "#7fb3d5", 10, 39],
  ["silk", "#8fcf6b", 39, 39], ["tea", "#b07cc6", 25, 25], ["wool", "#c9c2b2", 25, 0],
].map(good);
const book = (n) => ({
  name: "pollution",
  production: Array.from({ length: n }, (_, i) => (i === 0 ? 1 : 0)),
  consumption: Array.from({ length: n }, (_, i) => (i === 0 ? 1 : 0)),
  devalues: Array.from({ length: n }, (_, i) => i === 0),
});
const sweep = {
  name: "N goods: carrying capacity vs number of goods",
  description: "Starting settings; measured in the experiments plan, Task 5.",
  base: { preset: "n-4-peaks" },
  x: {
    label: "Number of goods",
    values: [2, 3, 4, 5, 6].map((n) => ({
      at: n,
      set: { goods: goods.slice(0, n), "pollution.pollutants": [book(n)] },
    })),
  },
  series: {
    label: "Trade",
    values: [
      { at: 0, name: "No trade", set: { "trade.enabled": false } },
      { at: 1, name: "Trade", set: { "trade.enabled": true } },
    ],
  },
  seeds: { from: 1, count: 10 },
  ticks: 500,
  metric: { kind: "window_mean", series: "population", from: 400 },
};
fs.writeFileSync("sweeps/n-goods-carrying-capacity.json", JSON.stringify(sweep, null, 2) + "\n");'
```

- [ ] **Step 3: Write the failing tests** (append inside `mod tests` in `sweep.rs`)

```rust
    #[test]
    fn builtin_sweeps_parse_and_validate() {
        let ids: Vec<&str> = builtins().iter().map(|b| b.id).collect();
        assert_eq!(ids, ["fig-ii-5", "fig-iv-6", "fig-iv-10-11", "n-goods-carrying-capacity"]);
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
            s.config_for(&point).unwrap(),
            presets::by_id("n-4-peaks").unwrap().config
        );
    }
```

Run: `cargo test -p sugarscape-core sweep::` — Expected: compile errors (`builtins` not found).

- [ ] **Step 4: Implement**

Append after `run_all` (above `#[cfg(test)]`):
```rust
/// A built-in sweep: `sweeps/<id>.json` at the repository root (Decision 11).
pub struct Builtin {
    pub id: &'static str,
    pub json: &'static str,
}

const BUILTINS: [Builtin; 4] = [
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
```

Run: `cargo test -p sugarscape-core sweep::`
Expected: 27 passed. If `four_goods_with_trade_is_the_n_4_peaks_preset` fails, compare the two configs field by field and fix the **generator** in Step 2 (never the preset).

- [ ] **Step 5: Measure**

Append this temporary test to `crates/sugarscape-core/tests/book.rs` (and `use sugarscape_core::sweep::{self, Metric, Outcome};` at the top):
```rust
#[test]
#[ignore]
fn measure_builtin_sweeps() {
    // 1. Where does the population settle? 50-tick block means over
    //    t = 1..1000, seeds 1–5, for every (series, x) cell.
    for id in ["fig-ii-5", "fig-iv-6", "n-goods-carrying-capacity"] {
        let mut probe = sweep::builtin(id).unwrap();
        probe.ticks = 1000;
        probe.seeds.count = 5;
        let blocks = Metric::Timeseries { series: "population".into(), every: 50 };
        println!("{id}: mean population per 50-tick block ending t = 50, 100, …, 1000");
        let points = probe.points().unwrap();
        for series in 0..probe.series_count() {
            for (x, value) in probe.x.values.iter().enumerate() {
                let mut means = vec![0.0; 20];
                for point in points.iter().filter(|p| p.series == series && p.x == x) {
                    let mut w = World::new(probe.config_for(point).unwrap(), point.seed).unwrap();
                    w.run(1000);
                    let history = w.stats.series("population").unwrap();
                    let Outcome::Series { values } = sweep::measure(&blocks, 1000, &history) else {
                        unreachable!()
                    };
                    for (m, v) in means.iter_mut().zip(values) {
                        *m += v / 5.0;
                    }
                }
                let shown: Vec<f64> = means.iter().map(|m| (m * 10.0).round() / 10.0).collect();
                println!("  {} | x = {}: {shown:?}", probe.series_name(series), value.at);
            }
        }
    }
    // 2. Runtime and results at the files' current settings, on one thread.
    for id in ["fig-ii-5", "fig-iv-6", "fig-iv-10-11", "n-goods-carrying-capacity"] {
        let s = sweep::builtin(id).unwrap();
        let start = std::time::Instant::now();
        let result = sweep::run_all(&s, 1, |_, _| {}).unwrap();
        println!(
            "{id}: {} runs in {:.1} s on one thread",
            result.runs.len(),
            start.elapsed().as_secs_f64()
        );
        print!("{}", sweep::summary_csv(&result));
    }
}
```
Run: `cargo test -p sugarscape-core --release --test book -- --ignored --nocapture measure_builtin_sweeps`

Choose, **with these rules only** (do not invent other candidates):

- **Ticks and window** (`fig-ii-5`, `fig-iv-6`, `n-goods-carrying-capacity`, each separately). Let B(t) be a cell's printed block mean ending at tick t. A candidate T ∈ {300, 500, 1000} is *settled* when, for **every** cell, |B(T − 100) − B(T)| and |B(T − 50) − B(T)| are both ≤ max(3, 0.05 · B(T)). Use the **smallest** settled T (1000 if none is, and say so in the description): set `"ticks": T` and `"metric": { …, "from": T − 100 }` (the window is the last 100 ticks; `to` stays omitted).
- **`fig-iv-10-11`** keeps `ticks` 1000 and `every` 50.
- **Seeds.** Re-run the command after editing ticks and windows; let t be a file's one-thread time at 10 seeds. Keep `"count": 10` (the book's 10 runs per point) if t ≤ 60 s; otherwise 5 if t/2 ≤ 60 s; otherwise 3 (time is proportional to the seed count). The budget assumes the browser build is up to about twice as slow as native and runs on three or more workers, so 60 s here is roughly 20–40 s there; the controller confirms this in the browser pass after Task 16.
- **Qualitative check at the chosen settings** (from the second printout, after the final edit): `fig-ii-5` — within each metabolism line the mean at mean vision 6 exceeds the mean at 1, and at every x the means fall as metabolism rises; `fig-iv-6` — the Trade mean exceeds the No-trade mean at every x; `fig-iv-10-11` — the last block (t = 1000) of `Lifetimes 960–1000` has a lower mean than `Lifetimes 60–100`. If any fails, stop and report **BLOCKED** with the printed tables; do not change axes, presets or seeds to force it. `n-goods-carrying-capacity` has no required outcome.

- [ ] **Step 6: Record the settings in the descriptions**

Replace each `description` with the text below, filling the bracketed parts from the measurement (T, F = T − 100, N = seed count, and the printed means):

- `fig-ii-5`: `"Figure II-5: carrying capacity (mean population over ticks F–T, after it has settled) against mean vision, one line per mean metabolism, starting from 500 agents under ({G₁}, {M}). A mean of m is the uniform range 1–(2m−1). Seeds 1–N; the book averages 10 runs per point. Measured (release, seeds 1–N): <the mean at vision 1 and at vision 6 for each metabolism line>."`
- `fig-iv-6`: `"Figure IV-6: carrying capacity (mean population over ticks F–T) against mean vision, with and without trade, in the ({G₁}, {M, T}) market of iv-3-trade. A mean of m is the uniform range 1–(2m−1). Seeds 1–N; the book averages 10 runs per point. Measured (release, seeds 1–N): <the two means at vision 1 and at vision 6>."`
- `fig-iv-10-11`: `"Figures IV-10 and IV-11: the standard deviation of ln(trade price) per tick, averaged over 50-tick blocks, in iv-3-trade with finite lives and replacement, for lifetimes 60–100 and 960–1000. Seeds 1–N. Measured (release, seeds 1–N): <each line's mean in the first and last block>."`
- `n-goods-carrying-capacity`: `"Carrying capacity (mean population over ticks F–T) against the number of goods, with and without trade: goods 1–4 are n-4-peaks' corner peaks, then tea at (25, 25) and wool at (25, 0), each with metabolism 1–2 and endowment 25–50. Two goods at least, since trade needs two. Seeds 1–N. Measured (release, seeds 1–N): <the two means at 2 and at 6 goods>."`

(For `n-goods-carrying-capacity`, edit the generated file directly; do not re-run the generator.)

Then delete `measure_builtin_sweeps` and the `use sugarscape_core::sweep::…` line from `tests/book.rs`; `git diff crates/sugarscape-core/tests/book.rs` must be empty.

- [ ] **Step 7: Verify**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
```
Expected: all pass, including `builtin_sweeps_parse_and_validate` with the final files.

- [ ] **Step 8: Commit**

```bash
git add sweeps/fig-ii-5.json sweeps/fig-iv-6.json sweeps/fig-iv-10-11.json sweeps/n-goods-carrying-capacity.json crates/sugarscape-core/src/sweep.rs
git commit -m "Ship measured sweeps for Figures II-5, IV-6, IV-10/11 and N goods" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 6: The `sugarscape` CLI: `presets`, `sweeps` and `run`

*Mechanical (full code).*

**Files:**
- Create: `crates/sugarscape-cli/Cargo.toml`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`
- Modify: `Cargo.toml` (workspace members), `Cargo.lock` (generated)

**Interfaces:**
- Consumes: `presets::{all, by_id}`, `Config::from_json`, `World::{new, run, fingerprint}`, `export::{series_csv, agents_csv}`, `sweep::{builtins, Sweep::from_json}`.
- Produces: binary `sugarscape` with `presets`, `sweeps`, `run` (Decision 15); private `Cli`, `Command`, `RunArgs`, `ConfigSource`, `Failure { Io(String), Invalid(Vec<FieldError>) }`, `fn run(cli: Cli) -> Result<(), Failure>`, `fn read(&Path) -> Result<String, Failure>`, `fn write(&Path, &str) -> Result<(), Failure>`. Task 7 adds `Sweep(SweepArgs)`.

- [ ] **Step 1: Add the crate**

In the root `Cargo.toml`, change the members line to:
```toml
members = ["crates/sugarscape-core", "crates/sugarscape-wasm", "crates/sugarscape-cli"]
```

Create `crates/sugarscape-cli/Cargo.toml`:
```toml
[package]
name = "sugarscape-cli"
description = "Command-line runner for Sugarscape worlds and parameter sweeps"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "sugarscape"
path = "src/main.rs"

[dependencies]
sugarscape-core = { path = "../sugarscape-core" }
clap = { version = "4", features = ["derive"] }
serde_json.workspace = true
```

- [ ] **Step 2: Write the failing integration tests**

Create `crates/sugarscape-cli/tests/cli.rs`:
```rust
//! End-to-end checks of the `sugarscape` binary.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sugarscape_core::config::Config;
use sugarscape_core::presets;

fn sugarscape(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(args)
        .output()
        .expect("the binary runs")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).unwrap()
}

/// A fresh directory for one test's files.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("sugarscape-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn path(p: &Path) -> &str {
    p.to_str().unwrap()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn presets_and_sweeps_are_listed() {
    let out = sugarscape(&["presets"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(stdout(&out).lines().any(|l| l.starts_with("ii-2-unit\tAnimation II-2\t")));
    let out = sugarscape(&["sweeps"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let text = stdout(&out);
    for id in ["fig-ii-5", "fig-iv-6", "fig-iv-10-11", "n-goods-carrying-capacity"] {
        assert!(text.lines().any(|l| l.starts_with(&format!("{id}\t"))), "{text}");
    }
}

#[test]
fn run_prints_the_golden_fingerprint_and_writes_its_files() {
    let dir = scratch("run");
    let (series, agents, config) = (dir.join("series.csv"), dir.join("agents.csv"), dir.join("config.json"));
    let out = sugarscape(&[
        "run", "--preset", "ii-2-unit", "--ticks", "200", "--fingerprint",
        "--series-csv", path(&series), "--agents-csv", path(&agents), "--config-out", path(&config),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    // tests/golden.rs: ii-2-unit after 200 ticks from seed 1.
    assert_eq!(stdout(&out), "0x75b93943813545e4\n");
    let series = read(&series);
    assert!(series.starts_with("tick,population,"));
    assert_eq!(series.lines().count(), 202, "header and ticks 0..=200");
    assert!(read(&agents).starts_with("id,x,y,"));
    let written = Config::from_json(&read(&config)).unwrap();
    assert_eq!(written, presets::by_id("ii-2-unit").unwrap().config);

    // The written config reproduces the run.
    let again = sugarscape(&["run", "--config", path(&config), "--ticks", "200", "--fingerprint"]);
    assert_eq!(stdout(&again), "0x75b93943813545e4\n");
}

#[test]
fn run_errors_have_exit_codes() {
    assert_eq!(sugarscape(&[]).status.code(), Some(2));
    assert_eq!(sugarscape(&["run"]).status.code(), Some(2));
    let out = sugarscape(&["run", "--preset", "no-such-preset"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).starts_with("preset: unknown preset \"no-such-preset\""), "{}", stderr(&out));
    let out = sugarscape(&["run", "--config", "/nonexistent/sugarscape/config.json"]);
    assert_eq!(out.status.code(), Some(1));
    let dir = scratch("bad-config");
    let bad = dir.join("bad.json");
    std::fs::write(&bad, r#"{"population": 99999}"#).unwrap();
    let out = sugarscape(&["run", "--config", path(&bad)]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).lines().any(|l| l.starts_with("population: ")), "{}", stderr(&out));
}
```

Run: `cargo test -p sugarscape-cli` — Expected: fails to build (`src/main.rs` missing).

- [ ] **Step 3: Implement**

Create `crates/sugarscape-cli/src/main.rs`:
```rust
//! `sugarscape`: run Sugarscape worlds and parameter sweeps from the command
//! line (milestone 5). Exit codes: 0 success, 1 I/O error, 2 usage or
//! validation error (printed as `field: message`, one per line).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use sugarscape_core::config::{Config, FieldError};
use sugarscape_core::sweep::{self, Sweep};
use sugarscape_core::world::World;
use sugarscape_core::{export, presets};

#[derive(Debug, Parser)]
#[command(name = "sugarscape", version, about = "Run Sugarscape worlds and parameter sweeps")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List the presets (id, source, name).
    Presets,
    /// List the built-in sweeps (id, name).
    Sweeps,
    /// Run one world and write its statistics.
    Run(RunArgs),
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct ConfigSource {
    /// A preset id (see `sugarscape presets`).
    #[arg(long, value_name = "ID")]
    preset: Option<String>,
    /// A config JSON file (current or pre-N-goods shape).
    #[arg(long, value_name = "FILE")]
    config: Option<PathBuf>,
}

#[derive(Debug, Args)]
struct RunArgs {
    #[command(flatten)]
    source: ConfigSource,
    #[arg(long, default_value_t = 1)]
    seed: u64,
    #[arg(long, default_value_t = 1000)]
    ticks: u32,
    /// Write the statistics history (one row per tick).
    #[arg(long, value_name = "PATH")]
    series_csv: Option<PathBuf>,
    /// Write the agents alive at the end.
    #[arg(long, value_name = "PATH")]
    agents_csv: Option<PathBuf>,
    /// Write the config as loaded (normalized JSON).
    #[arg(long, value_name = "PATH")]
    config_out: Option<PathBuf>,
    /// Print the final world's fingerprint as 0x%016x.
    #[arg(long)]
    fingerprint: bool,
}

/// Why a command failed.
#[derive(Debug)]
enum Failure {
    /// Exit code 1.
    Io(String),
    /// Exit code 2.
    Invalid(Vec<FieldError>),
}

impl From<Vec<FieldError>> for Failure {
    fn from(errors: Vec<FieldError>) -> Self {
        Failure::Invalid(errors)
    }
}

fn read(path: &Path) -> Result<String, Failure> {
    std::fs::read_to_string(path)
        .map_err(|e| Failure::Io(format!("cannot read {}: {e}", path.display())))
}

fn write(path: &Path, text: &str) -> Result<(), Failure> {
    std::fs::write(path, text)
        .map_err(|e| Failure::Io(format!("cannot write {}: {e}", path.display())))
}

fn main() -> ExitCode {
    // Usage errors exit with 2 inside `parse`.
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Io(message)) => {
            eprintln!("error: {message}");
            ExitCode::from(1)
        }
        Err(Failure::Invalid(errors)) => {
            for e in errors {
                eprintln!("{}: {}", e.field, e.message);
            }
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<(), Failure> {
    match cli.command {
        Command::Presets => {
            for p in presets::all() {
                println!("{}\t{}\t{}", p.id, p.source, p.name);
            }
            Ok(())
        }
        Command::Sweeps => {
            for b in sweep::builtins() {
                println!("{}\t{}", b.id, Sweep::from_json(b.json)?.name);
            }
            Ok(())
        }
        Command::Run(args) => run_world(args),
    }
}

fn run_world(args: RunArgs) -> Result<(), Failure> {
    let config = match (&args.source.preset, &args.source.config) {
        (Some(id), _) => presets::by_id(id).map(|p| p.config).ok_or_else(|| {
            Failure::Invalid(vec![FieldError::new(
                "preset",
                format!("unknown preset {id:?} (see `sugarscape presets`)"),
            )])
        })?,
        (None, Some(path)) => Config::from_json(&read(path)?)?,
        (None, None) => unreachable!("clap requires --preset or --config"),
    };
    let mut world = World::new(config.clone(), args.seed)?;
    world.run(args.ticks);
    if let Some(path) = &args.series_csv {
        write(path, &export::series_csv(&world))?;
    }
    if let Some(path) = &args.agents_csv {
        write(path, &export::agents_csv(&world))?;
    }
    if let Some(path) = &args.config_out {
        let json = serde_json::to_string_pretty(&config).expect("configs serialize");
        write(path, &(json + "\n"))?;
    }
    if args.fingerprint {
        println!("{:#018x}", world.fingerprint());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn the_command_line_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn run_has_defaults_and_needs_exactly_one_source() {
        let cli = Cli::try_parse_from(["sugarscape", "run", "--preset", "ii-2-unit"]).unwrap();
        let Command::Run(args) = cli.command else {
            panic!("run expected");
        };
        assert_eq!((args.seed, args.ticks, args.fingerprint), (1, 1000, false));
        assert!(Cli::try_parse_from(["sugarscape", "run"]).is_err());
        assert!(Cli::try_parse_from(["sugarscape", "run", "--preset", "a", "--config", "b.json"]).is_err());
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-cli`
Expected: 2 unit tests and 3 integration tests pass (the first build downloads `clap`).

- [ ] **Step 5: Verify** — `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test`

- [ ] **Step 6: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add Cargo.toml Cargo.lock crates/sugarscape-cli/Cargo.toml crates/sugarscape-cli/src/main.rs crates/sugarscape-cli/tests/cli.rs
git commit -m "Add the sugarscape CLI with presets, sweeps and run" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 7: `sugarscape sweep`

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`

**Interfaces:**
- Consumes: `sweep::{builtin, run_all, runs_csv, summary_csv}`, `Sweep::{from_json, point_count, series_name}`, `SweepResult::to_json`.
- Produces: `Command::Sweep(SweepArgs)`, `SweepSource { file, builtin }`, `fn run_sweep(args: SweepArgs) -> Result<(), Failure>`.

- [ ] **Step 1: Write the failing tests** (append to `tests/cli.rs`)

```rust
/// The core's `tiny()` sweep: 2 series × 3 x values × 2 seeds, 20 ticks.
const TINY: &str = r#"{
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
}"#;

#[test]
fn sweep_files_are_the_same_for_any_jobs() {
    let dir = scratch("sweep");
    let spec = dir.join("tiny.json");
    std::fs::write(&spec, TINY).unwrap();
    let run = |jobs: &str| {
        let out = dir.join(format!("out-{jobs}.json"));
        let runs = dir.join(format!("runs-{jobs}.csv"));
        let summary = dir.join(format!("summary-{jobs}.csv"));
        let o = sugarscape(&[
            "sweep", path(&spec), "--jobs", jobs, "--out", path(&out),
            "--runs-csv", path(&runs), "--summary-csv", path(&summary),
        ]);
        assert!(o.status.success(), "{}", stderr(&o));
        assert_eq!(stdout(&o), "", "the result went to --out");
        let progress: Vec<String> = stderr(&o).lines().map(String::from).collect();
        assert_eq!(progress.len(), 12);
        assert!(progress[11].starts_with("[12/12] series="), "{progress:?}");
        (read(&out), read(&runs), read(&summary))
    };
    let one = run("1");
    let four = run("4");
    assert_eq!(one, four);
    assert!(one.0.contains("\"version\": 1"));
    assert_eq!(one.1.lines().next(), Some("series,x,seed,value"));
    assert_eq!(one.1.lines().count(), 13);
    assert_eq!(one.2.lines().next(), Some("series,series_name,x,n,mean,sd,min,max"));
    assert_eq!(one.2.lines().count(), 7);
}

#[test]
fn sweep_prints_json_and_quiet_silences_progress() {
    let dir = scratch("sweep-stdout");
    let spec = dir.join("tiny.json");
    std::fs::write(&spec, TINY).unwrap();
    let out = sugarscape(&["sweep", path(&spec), "--quiet", "--seeds", "1", "--ticks", "15"]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stderr(&out), "");
    let result: sugarscape_core::sweep::SweepResult = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!((result.runs.len(), result.sweep.seeds.count, result.sweep.ticks), (6, 1, 15));
}

#[test]
fn sweep_errors_have_exit_codes() {
    let dir = scratch("sweep-errors");
    let spec = dir.join("tiny.json");
    std::fs::write(&spec, TINY).unwrap();
    // The window starts at tick 10: 5 ticks is too short.
    let out = sugarscape(&["sweep", path(&spec), "--ticks", "5", "--quiet"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).lines().any(|l| l.starts_with("metric.from: ")), "{}", stderr(&out));
    let out = sugarscape(&["sweep", "--builtin", "nope"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).starts_with("builtin: unknown sweep \"nope\""));
    let broken = dir.join("broken.json");
    std::fs::write(&broken, "{").unwrap();
    let out = sugarscape(&["sweep", path(&broken)]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).starts_with("sweep: "));
    assert_eq!(sugarscape(&["sweep", "/nonexistent/sweep.json"]).status.code(), Some(1));
    assert_eq!(sugarscape(&["sweep"]).status.code(), Some(2));
    assert_eq!(sugarscape(&["sweep", path(&spec), "--jobs", "0"]).status.code(), Some(2));
}
```

(`serde_json` is already a dependency of the crate, so the integration tests can use it.)

Run: `cargo test -p sugarscape-cli` — Expected: the three new tests fail (`sweep` is not a subcommand).

- [ ] **Step 2: Implement**

In `src/main.rs`, add the variant to `Command` (after `Run(RunArgs),`):
```rust
    /// Run a parameter sweep.
    Sweep(SweepArgs),
```

Add after `struct RunArgs { … }`:
```rust
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
struct SweepSource {
    /// A sweep JSON file.
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    /// A built-in sweep id (see `sugarscape sweeps`).
    #[arg(long, value_name = "NAME")]
    builtin: Option<String>,
}

#[derive(Debug, Args)]
struct SweepArgs {
    #[command(flatten)]
    source: SweepSource,
    /// Worker threads (default: available parallelism).
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1024))]
    jobs: Option<u32>,
    /// Override the sweep's seed count.
    #[arg(long, value_name = "N")]
    seeds: Option<u32>,
    /// Override the sweep's ticks (a window or block length must still fit).
    #[arg(long, value_name = "N")]
    ticks: Option<u32>,
    /// Write the result JSON here instead of stdout.
    #[arg(long, value_name = "PATH")]
    out: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    runs_csv: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    summary_csv: Option<PathBuf>,
    /// No progress on stderr.
    #[arg(long)]
    quiet: bool,
}
```

In `fn run`, add the arm after `Command::Run(args) => run_world(args),`:
```rust
        Command::Sweep(args) => run_sweep(args),
```

Add after `fn run_world`:
```rust
fn run_sweep(args: SweepArgs) -> Result<(), Failure> {
    let mut sweep = match (&args.source.file, &args.source.builtin) {
        (Some(path), _) => Sweep::from_json(&read(path)?)?,
        (None, Some(id)) => sweep::builtin(id).ok_or_else(|| {
            Failure::Invalid(vec![FieldError::new(
                "builtin",
                format!("unknown sweep {id:?} (see `sugarscape sweeps`)"),
            )])
        })?,
        (None, None) => unreachable!("clap requires a file or --builtin"),
    };
    if let Some(n) = args.seeds {
        sweep.seeds.count = n;
    }
    if let Some(t) = args.ticks {
        sweep.ticks = t;
    }
    let jobs = args.jobs.map_or_else(
        || std::thread::available_parallelism().map_or(1, |n| n.get()),
        |j| j as usize,
    );
    let total = sweep.point_count();
    let result = sweep::run_all(&sweep, jobs, |done, point| {
        if !args.quiet {
            eprintln!(
                "[{done}/{total}] series={} x={} seed={}",
                sweep.series_name(point.series),
                sweep.x.values[point.x].at,
                point.seed
            );
        }
    })?;
    let json = result.to_json();
    match &args.out {
        Some(path) => write(path, &json)?,
        None => print!("{json}"),
    }
    if let Some(path) = &args.runs_csv {
        write(path, &sweep::runs_csv(&result))?;
    }
    if let Some(path) = &args.summary_csv {
        write(path, &sweep::summary_csv(&result))?;
    }
    Ok(())
}
```

Add to the unit tests in `main.rs`:
```rust
    #[test]
    fn sweep_takes_a_file_or_a_builtin() {
        assert!(Cli::try_parse_from(["sugarscape", "sweep", "s.json"]).is_ok());
        assert!(Cli::try_parse_from(["sugarscape", "sweep", "--builtin", "fig-ii-5"]).is_ok());
        assert!(Cli::try_parse_from(["sugarscape", "sweep"]).is_err());
        assert!(Cli::try_parse_from(["sugarscape", "sweep", "s.json", "--builtin", "fig-ii-5"]).is_err());
        assert!(Cli::try_parse_from(["sugarscape", "sweep", "s.json", "--jobs", "0"]).is_err());
    }
```

- [ ] **Step 3: Run the tests**

Run: `cargo test -p sugarscape-cli`
Expected: 3 unit tests and 6 integration tests pass.

Then a smoke run of a built-in at reduced size:
`cargo run --release -p sugarscape-cli -- sweep --builtin fig-iv-10-11 --seeds 1 --ticks 100 --quiet | head -3`
Expected: `{`, `  "version": 1,`, `  "sweep": {`.

- [ ] **Step 4: Verify** — `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test`

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-cli/src/main.rs crates/sugarscape-cli/tests/cli.rs
git commit -m "Add sugarscape sweep with byte-identical output for any --jobs" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 8: Sweep functions in WASM

*Mechanical (full code).*

**Files:**
- Modify: `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: `sweep::{Sweep, RunResult, SweepResult, run_point, aggregate, check_runs, builtins, runs_csv, summary_csv}`, `stats::series_names`, `Config::from_json`, the existing `field_errors`.
- Produces (all `#[wasm_bindgen]` free functions; errors thrown as JSON `[{field, message}]`, Decision 16):
  - `sweep_points(spec: &str) -> Result<String, JsValue>` — JSON `[{index, series, x, seed}]`
  - `run_point(spec: &str, index: u32) -> Result<String, JsValue>` — `RunResult` JSON
  - `aggregate(spec: &str, runs: &str) -> Result<String, JsValue>` — `Summary` JSON
  - `builtin_sweeps() -> String` — JSON `[{id, sweep}]`
  - `config_series_names(config: &str) -> Result<String, JsValue>` — JSON string list
  - `sweep_result(spec: &str, runs: &str) -> Result<String, JsValue>` — `SweepResult::to_json`
  - `sweep_csv(spec: &str, runs: &str, kind: &str) -> Result<String, JsValue>` — `kind` = `"runs"` | `"summary"`

- [ ] **Step 1: Write the failing tests** (append to `crates/sugarscape-wasm/tests/web.rs`)

```rust
use sugarscape_core::sweep::{self as core_sweep, Sweep};
use sugarscape_wasm::{
    aggregate, builtin_sweeps, config_series_names, run_point, sweep_csv, sweep_points, sweep_result,
};

/// The core's `tiny()` sweep: 2 series × 3 x values × 2 seeds, 20 ticks.
const TINY: &str = r#"{
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
}"#;

#[wasm_bindgen_test]
fn sweep_points_lists_points_or_errors() {
    let points: serde_json::Value = serde_json::from_str(&sweep_points(TINY).unwrap()).unwrap();
    assert_eq!(points.as_array().unwrap().len(), 12);
    assert_eq!(points[7], serde_json::json!({ "index": 7, "series": 1, "x": 0, "seed": 6 }));
    let err = sweep_points(&TINY.replace("\"ticks\": 20", "\"ticks\": 0")).unwrap_err();
    assert!(err.as_string().unwrap().contains(r#""field":"ticks""#));
    let err = sweep_points("{").unwrap_err();
    assert!(err.as_string().unwrap().contains(r#""field":"sweep""#));
}

#[wasm_bindgen_test]
fn run_point_and_aggregate_match_the_core_run_all() {
    // Completion order does not matter: run the points backwards.
    let runs: Vec<String> = (0..12u32).rev().map(|i| run_point(TINY, i).unwrap()).collect();
    let runs = format!("[{}]", runs.join(","));
    let expected = core_sweep::run_all(&Sweep::from_json(TINY).unwrap(), 1, |_, _| {}).unwrap();
    assert_eq!(sweep_result(TINY, &runs).unwrap(), expected.to_json());
    assert_eq!(
        aggregate(TINY, &runs).unwrap(),
        serde_json::to_string(&expected.summary).unwrap()
    );
    assert_eq!(sweep_csv(TINY, &runs, "runs").unwrap(), core_sweep::runs_csv(&expected));
    assert_eq!(sweep_csv(TINY, &runs, "summary").unwrap(), core_sweep::summary_csv(&expected));
    assert!(sweep_csv(TINY, &runs, "other").is_err());
    assert!(run_point(TINY, 12).is_err());
}

#[wasm_bindgen_test]
fn partial_runs_aggregate_and_foreign_runs_are_rejected() {
    let one = run_point(TINY, 3).unwrap();
    let summary: serde_json::Value =
        serde_json::from_str(&aggregate(TINY, &format!("[{one}]")).unwrap()).unwrap();
    assert_eq!(summary["rows"].as_array().unwrap().len(), 6);
    assert_eq!(summary["rows"][1]["n"], 1);
    assert!(sweep_result(TINY, &format!("[{one}]")).unwrap().contains("\"incomplete\": true"));
    let foreign = one.replace("\"seed\":6", "\"seed\":99");
    let err = aggregate(TINY, &format!("[{foreign}]")).unwrap_err();
    assert!(err.as_string().unwrap().contains("runs[0]"));
    assert!(aggregate(TINY, "not json").is_err());
}

#[wasm_bindgen_test]
fn builtins_and_series_names_are_listed() {
    let list: serde_json::Value = serde_json::from_str(&builtin_sweeps()).unwrap();
    let ids: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["fig-ii-5", "fig-iv-6", "fig-iv-10-11", "n-goods-carrying-capacity"]);
    assert!(list[0]["sweep"]["name"].as_str().unwrap().starts_with("Figure II-5"));
    let names: Vec<String> = serde_json::from_str(&config_series_names("{}").unwrap()).unwrap();
    assert!(names.iter().any(|n| n == "population"));
    assert!(names.iter().any(|n| n == "mean_holding_0"));
    assert!(config_series_names(r#"{"population": -1}"#).is_err());
}
```
(Point 3 is series 0, x 1, seed 6 — the second row of the summary.)

Run: `wasm-pack test --node crates/sugarscape-wasm` — Expected: compile errors (functions not found).

- [ ] **Step 2: Implement**

In `crates/sugarscape-wasm/src/lib.rs`, change the core imports to:
```rust
use sugarscape_core::config::{Config, FieldError};
use sugarscape_core::edit::AgentOverrides;
use sugarscape_core::render::{self, ColorMode, Layer};
use sugarscape_core::sweep::{RunResult, Sweep, SweepResult};
use sugarscape_core::world::World;
use sugarscape_core::{export, network, presets, stats, sweep};
```

Append after `default_config_json` (before `landscapes_from_js`):
```rust
fn parse_sweep(spec: &str) -> Result<Sweep, JsValue> {
    Sweep::from_json(spec).map_err(field_errors)
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
    let points = parse_sweep(spec)?.points().map_err(field_errors)?;
    Ok(serde_json::to_string(&points).expect("points serialize"))
}

/// Runs point `index` of the sweep; returns its `RunResult` JSON.
#[wasm_bindgen]
pub fn run_point(spec: &str, index: u32) -> Result<String, JsValue> {
    let sweep = parse_sweep(spec)?;
    let point = sweep.point(index as usize).map_err(field_errors)?;
    sweep.config_for(&point).map_err(field_errors)?;
    let run = sweep::run_point(&sweep, &point);
    Ok(serde_json::to_string(&run).expect("runs serialize"))
}

/// The `Summary` JSON of `runs` (any order, possibly partial).
#[wasm_bindgen]
pub fn aggregate(spec: &str, runs: &str) -> Result<String, JsValue> {
    let sweep = parse_sweep(spec)?;
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
    let sweep = parse_sweep(spec)?;
    let runs = parse_runs(&sweep, runs)?;
    Ok(SweepResult::new(sweep, runs).to_json())
}

/// The CLI's runs CSV (`kind = "runs"`) or summary CSV (`"summary"`).
#[wasm_bindgen]
pub fn sweep_csv(spec: &str, runs: &str, kind: &str) -> Result<String, JsValue> {
    let sweep = parse_sweep(spec)?;
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
```

- [ ] **Step 3: Run the tests**

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: every earlier test and the four new ones pass.

- [ ] **Step 4: Verify**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
(cd web && npm run build && npm test)
```
(The web build regenerates `web/src/wasm-pkg` with the new exports; nothing in `web/src` uses them yet.)

- [ ] **Step 5: Commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-wasm/src/lib.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Expose sweep points, runs, aggregation and exports to WASM" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 9: The worker pool

*Mechanical (full code).*

**Files:**
- Create: `web/src/experiments/types.ts`, `web/src/experiments/pool.ts`, `web/src/experiments/pool.test.ts`, `web/src/experiments/worker.ts`
- Modify: `web/vite.config.ts`

**Interfaces:**
- Consumes: the WASM export `run_point` (Task 8), `init` (the wasm-bindgen default export, as `engine.ts` uses it), `parseErrors` and `FieldError` from `web/src/types.ts`.
- Produces:
  - `experiments/types.ts`: `AxisValue`, `Axis`, `ShorthandAxis`, `SweepBase`, `Metric`, `Sweep`, `Point`, `RunResult`, `ScalarRow`, `BlockRow`, `Summary`, `SweepResult`, `BuiltinSweep` (mirrors of the core's JSON).
  - `experiments/pool.ts`: `interface PointRequest { spec: string; index: number }`, `type PointReply = { index; run: RunResult } | { index; errors: FieldError[] }`, `interface WorkerLike { onmessage; onerror; postMessage(message: unknown): void; terminate(): void }`, `function poolSize(hardwareConcurrency: number | undefined): number`, `type PoolOutcome = 'done' | 'cancelled'`, `class WorkerPool { constructor(size: number, create: () => WorkerLike); run(spec: string, count: number, onResult: (run: RunResult) => void): Promise<PoolOutcome>; cancel(): void }`.
  - `experiments/worker.ts`: a module worker answering each `PointRequest` with a `PointReply`.

- [ ] **Step 1: Types**

Create `web/src/experiments/types.ts`:
```ts
// Mirrors of the sweep JSON shapes in sugarscape-core's `sweep` module.
import type { Config } from '../types';

export interface AxisValue { at: number; name?: string; set: Record<string, unknown> }
/** The full form (what the core writes). */
export interface Axis { label: string; values: AxisValue[] }
/** The shorthand: value i sets `path` and sits at the value itself (numbers) or at i. */
export interface ShorthandAxis { label?: string; path: string; values: unknown[] }
export type SweepBase = { preset: string } | { config: Config };
export type Metric =
  | { kind: 'final'; series: string }
  | { kind: 'window_mean'; series: string; from: number; to?: number }
  | { kind: 'timeseries'; series: string; every: number };
export interface Sweep {
  name: string;
  description?: string;
  base: SweepBase;
  set?: Record<string, unknown>;
  x: Axis | ShorthandAxis;
  series?: Axis | ShorthandAxis;
  seeds: { from: number; count: number };
  ticks: number;
  metric: Metric;
}
export interface Point { index: number; series: number; x: number; seed: number }
interface RunBase { point: number; series: number; x: number; seed: number }
/** `null` stands for NaN. */
export type RunResult = (RunBase & { value: number | null }) | (RunBase & { values: (number | null)[] });
export interface ScalarRow {
  series: number;
  series_name: string;
  x: number;
  at: number;
  n: number;
  nan: number;
  mean: number | null;
  sd: number | null;
  min: number | null;
  max: number | null;
}
export interface BlockRow { series: number; series_name: string; t: number; n: number; mean: number | null; sd: number | null }
export type Summary = { kind: 'scalar'; rows: ScalarRow[] } | { kind: 'timeseries'; rows: BlockRow[] };
export interface SweepResult { version: 1; sweep: Sweep; runs: RunResult[]; summary: Summary; incomplete?: boolean }
export interface BuiltinSweep { id: string; sweep: Sweep }
```

- [ ] **Step 2: Write the failing pool tests**

Create `web/src/experiments/pool.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { poolSize, WorkerPool, type PointReply, type PointRequest, type WorkerLike } from './pool';
import type { RunResult } from './types';

/** Answers each request after a short, index-dependent delay (so replies arrive out of order). */
class FakeWorker implements WorkerLike {
  onmessage: ((event: MessageEvent) => void) | null = null;
  onerror: ((event: ErrorEvent) => void) | null = null;
  terminated = false;

  constructor(private log: number[], private failAt?: number) {}

  postMessage(message: unknown): void {
    const { index } = message as PointRequest;
    this.log.push(index);
    setTimeout(() => {
      if (this.terminated) return;
      const reply: PointReply =
        index === this.failAt
          ? { index, errors: [{ field: 'points', message: 'boom' }] }
          : { index, run: { point: index, series: 0, x: 0, seed: 1, value: index } };
      this.onmessage?.({ data: reply } as MessageEvent);
    }, (index * 7) % 5);
  }

  terminate(): void {
    this.terminated = true;
  }
}

function fakes(failAt?: number) {
  const log: number[] = [];
  const workers: FakeWorker[] = [];
  const create = () => {
    const w = new FakeWorker(log, failAt);
    workers.push(w);
    return w;
  };
  return { log, workers, create };
}

describe('worker pool', () => {
  it('leaves a core for the page', () => {
    expect(poolSize(8)).toBe(7);
    expect(poolSize(1)).toBe(1);
    expect(poolSize(undefined)).toBe(1);
  });

  it('hands out every index once, in order, and terminates its workers', async () => {
    const { log, workers, create } = fakes();
    const runs: RunResult[] = [];
    const outcome = await new WorkerPool(3, create).run('{}', 10, (run) => runs.push(run));
    expect(outcome).toBe('done');
    expect(log).toEqual([0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    expect(runs.map((r) => r.point).sort((a, b) => a - b)).toEqual(log);
    expect(workers).toHaveLength(3);
    expect(workers.every((w) => w.terminated)).toBe(true);
  });

  it('creates no more workers than points', async () => {
    const { workers, create } = fakes();
    await new WorkerPool(8, create).run('{}', 2, () => {});
    expect(workers).toHaveLength(2);
    expect(await new WorkerPool(8, create).run('{}', 0, () => {})).toBe('done');
    expect(workers).toHaveLength(2);
  });

  it('stops on cancel and keeps what finished', async () => {
    const { workers, create } = fakes();
    const pool = new WorkerPool(3, create);
    const runs: RunResult[] = [];
    const outcome = await pool.run('{}', 50, (run) => {
      runs.push(run);
      if (runs.length === 4) pool.cancel();
    });
    expect(outcome).toBe('cancelled');
    expect(runs).toHaveLength(4);
    expect(workers.every((w) => w.terminated)).toBe(true);
  });

  it('rejects on a point error', async () => {
    const { workers, create } = fakes(5);
    await expect(new WorkerPool(2, create).run('{}', 10, () => {})).rejects.toThrow('points: boom');
    expect(workers.every((w) => w.terminated)).toBe(true);
  });
});
```

Run: `(cd web && npx vitest run src/experiments/pool.test.ts)` — Expected: fails (`./pool` not found).

- [ ] **Step 3: Implement the pool**

Create `web/src/experiments/pool.ts`:
```ts
import type { FieldError } from '../types';
import type { RunResult } from './types';

export interface PointRequest { spec: string; index: number }
export type PointReply = { index: number; run: RunResult } | { index: number; errors: FieldError[] };

/** The part of `Worker` the pool uses (tests pass fakes). */
export interface WorkerLike {
  onmessage: ((event: MessageEvent) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  postMessage(message: unknown): void;
  terminate(): void;
}

/** `max(1, hardwareConcurrency − 1)`: leave a core for the page (Decision 19). */
export function poolSize(hardwareConcurrency: number | undefined): number {
  return Math.max(1, (hardwareConcurrency ?? 2) - 1);
}

export type PoolOutcome = 'done' | 'cancelled';

/**
 * Runs a sweep's points on workers, one point per worker at a time, handing
 * out indices in order. Workers are created per run and terminated when it
 * ends, fails or is cancelled.
 */
export class WorkerPool {
  private workers: WorkerLike[] = [];
  /** Ends the current run as cancelled; null when no run is active. */
  private stop: (() => void) | null = null;

  constructor(
    private size: number,
    private create: () => WorkerLike,
  ) {}

  run(spec: string, count: number, onResult: (run: RunResult) => void): Promise<PoolOutcome> {
    return new Promise<PoolOutcome>((resolve, reject) => {
      if (count === 0) {
        resolve('done');
        return;
      }
      let next = 0;
      let finished = 0;
      const end = (settle: () => void) => {
        for (const w of this.workers) w.terminate();
        this.workers = [];
        this.stop = null;
        settle();
      };
      this.stop = () => end(() => resolve('cancelled'));
      const feed = (w: WorkerLike) => {
        if (next < count) {
          const request: PointRequest = { spec, index: next++ };
          w.postMessage(request);
        }
      };
      for (let i = 0; i < Math.min(this.size, count); i++) {
        const w = this.create();
        w.onmessage = (event: MessageEvent) => {
          if (!this.stop) return;
          const reply = event.data as PointReply;
          if ('errors' in reply) {
            const message = reply.errors.map((e) => `${e.field}: ${e.message}`).join('; ');
            end(() => reject(new Error(message)));
            return;
          }
          onResult(reply.run);
          finished += 1;
          if (finished === count) end(() => resolve('done'));
          else if (this.stop) feed(w);
        };
        w.onerror = (event: ErrorEvent) => {
          if (this.stop) end(() => reject(new Error(event.message || 'a sweep worker failed')));
        };
        this.workers.push(w);
      }
      for (const w of this.workers) feed(w);
    });
  }

  cancel(): void {
    this.stop?.();
  }
}
```
(`else if (this.stop)`: `onResult` may itself cancel, as the test does.)

- [ ] **Step 4: The worker**

Create `web/src/experiments/worker.ts`:
```ts
// A sweep worker: its own WASM instance, one point per message.
import init, { run_point } from '../wasm-pkg/sugarscape.js';
import { parseErrors } from '../types';
import type { PointReply, PointRequest } from './pool';
import type { RunResult } from './types';

const ready = init();

addEventListener('message', async (event: MessageEvent<PointRequest>) => {
  const { spec, index } = event.data;
  await ready;
  let reply: PointReply;
  try {
    reply = { index, run: JSON.parse(run_point(spec, index)) as RunResult };
  } catch (e) {
    reply = { index, errors: parseErrors(e) };
  }
  postMessage(reply);
});
```

In `web/vite.config.ts`, add the `worker` option (after `base: './',`):
```ts
  // Sweep workers import the wasm-bindgen module, which finds its .wasm with
  // new URL(…, import.meta.url): build workers as ES modules.
  worker: { format: 'es' },
```

- [ ] **Step 5: Verify**

Run: `(cd web && npm run build && npm test)`
Expected: the build passes `tsc` (the worker is type-checked; nothing imports it yet, so Vite does not bundle it until Task 10) and every Vitest suite passes, including the 5 new pool tests.

- [ ] **Step 6: Commit**

```bash
git add web/src/experiments/types.ts web/src/experiments/pool.ts web/src/experiments/pool.test.ts web/src/experiments/worker.ts web/vite.config.ts
git commit -m "Add the sweep worker pool and module worker" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 10: The Experiments view shell

*Needs judgement (layout); the code is complete.*

**Files:**
- Create: `web/src/experiments/labels.ts`, `web/src/experiments/labels.test.ts`, `web/src/experiments/fixed-panel.ts`, `web/src/experiments/results-table.ts`, `web/src/experiments/view.ts`
- Modify: `web/index.html`, `web/src/main.ts`, `web/src/style.css`

**Interfaces:**
- Consumes: WASM `builtin_sweeps`, `sweep_points`, `aggregate`; `WorkerPool`, `poolSize`, `WorkerLike` (Task 9); `h`; `Engine.setRunning`.
- Produces:
  - `labels.ts`: `axisLabel(axis: Axis | ShorthandAxis): string`, `metricLabel(sweep: Sweep): string`, `baseLabel(sweep: Sweep): string`
  - `fixed-panel.ts`: `class FixedPanel { readonly el: HTMLElement; constructor(base: Sweep, onChange: () => void); sweep(): Sweep; showErrors(errors: FieldError[]): void }`
  - `results-table.ts`: `resultsTable(summary: Summary): HTMLElement`
  - `view.ts`: `interface SweepEditor { readonly el: HTMLElement; sweep(): Sweep | null; showErrors(errors: FieldError[]): void }`, `class ExperimentsView { readonly el: HTMLElement; constructor() }` (Task 13 adds the engine parameter)
  - `index.html`: `<main id="playground" class="layout">` and `<main id="experiments" class="experiments" hidden>`; the header's `.view-switch`.

- [ ] **Step 1: Labels, with tests**

Create `web/src/experiments/labels.ts`:
```ts
import type { Axis, ShorthandAxis, Sweep } from './types';

/** An axis's label (the shorthand's defaults to its path). */
export function axisLabel(axis: Axis | ShorthandAxis): string {
  return 'path' in axis ? (axis.label ?? axis.path) : axis.label;
}

/** What each run is summarized by, for the chart's y axis and the read-only panel. */
export function metricLabel(sweep: Sweep): string {
  const m = sweep.metric;
  switch (m.kind) {
    case 'final':
      return `${m.series} at t = ${sweep.ticks}`;
    case 'window_mean':
      return `${m.series}, mean over t = ${m.from}–${m.to ?? sweep.ticks}`;
    case 'timeseries':
      return `${m.series}, ${m.every}-tick means`;
  }
}

export function baseLabel(sweep: Sweep): string {
  return 'preset' in sweep.base ? `preset ${sweep.base.preset}` : 'a custom config';
}
```

Create `web/src/experiments/labels.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import type { Config } from '../types';
import { axisLabel, baseLabel, metricLabel } from './labels';
import type { Sweep } from './types';

const sweep: Sweep = {
  name: 's',
  base: { preset: 'ii-2-unit' },
  x: { path: 'vision.max', values: [1, 2] },
  seeds: { from: 1, count: 1 },
  ticks: 500,
  metric: { kind: 'final', series: 'population' },
};

describe('labels', () => {
  it('labels axes', () => {
    expect(axisLabel({ path: 'vision.max', values: [] })).toBe('vision.max');
    expect(axisLabel({ label: 'Vision', path: 'vision.max', values: [] })).toBe('Vision');
    expect(axisLabel({ label: 'Mean vision', values: [] })).toBe('Mean vision');
  });

  it('labels metrics', () => {
    expect(metricLabel(sweep)).toBe('population at t = 500');
    expect(metricLabel({ ...sweep, metric: { kind: 'window_mean', series: 'population', from: 400 } })).toBe(
      'population, mean over t = 400–500',
    );
    expect(metricLabel({ ...sweep, metric: { kind: 'window_mean', series: 'gini', from: 10, to: 20 } })).toBe(
      'gini, mean over t = 10–20',
    );
    expect(metricLabel({ ...sweep, metric: { kind: 'timeseries', series: 'sd_log_price', every: 50 } })).toBe(
      'sd_log_price, 50-tick means',
    );
  });

  it('labels bases', () => {
    expect(baseLabel(sweep)).toBe('preset ii-2-unit');
    expect(baseLabel({ ...sweep, base: { config: {} as Config } })).toBe('a custom config');
  });
});
```

- [ ] **Step 2: The read-only panel and the summary table**

Create `web/src/experiments/fixed-panel.ts`:
```ts
import type { FieldError } from '../types';
import { h } from '../ui/dom';
import { axisLabel, baseLabel, metricLabel } from './labels';
import type { Sweep } from './types';

/** A sweep shown read-only except its seed count and ticks: the built-ins, and files the form cannot express. */
export class FixedPanel {
  readonly el: HTMLElement;
  private readonly seeds: HTMLInputElement;
  private readonly ticks: HTMLInputElement;
  private readonly errors = h('div', { class: 'error' });

  constructor(
    private readonly base: Sweep,
    onChange: () => void,
  ) {
    this.seeds = h('input', { type: 'number', class: 'num', min: 1, max: 100, value: String(base.seeds.count), onchange: onChange });
    this.ticks = h('input', { type: 'number', class: 'num', min: 1, max: 100000, value: String(base.ticks), onchange: onChange });
    const lines = base.series ? `${axisLabel(base.series)}: ${base.series.values.length} lines` : 'one line';
    this.el = h(
      'div',
      { class: 'sweep-fixed' },
      h('h2', {}, base.name),
      base.description ? h('p', { class: 'hint' }, base.description) : null,
      h(
        'dl',
        {},
        h('dt', {}, 'Base'),
        h('dd', {}, baseLabel(base)),
        h('dt', {}, 'x'),
        h('dd', {}, `${axisLabel(base.x)}: ${base.x.values.length} values`),
        h('dt', {}, 'Lines'),
        h('dd', {}, lines),
        h('dt', {}, 'Metric'),
        h('dd', {}, metricLabel(base)),
      ),
      h('div', { class: 'row' }, h('label', {}, 'Seeds ', this.seeds), h('label', {}, 'Ticks ', this.ticks)),
      this.errors,
    );
  }

  sweep(): Sweep {
    return {
      ...structuredClone(this.base),
      seeds: { ...this.base.seeds, count: Number(this.seeds.value) },
      ticks: Number(this.ticks.value),
    };
  }

  showErrors(errors: FieldError[]): void {
    this.errors.replaceChildren(...errors.map((e) => h('p', {}, `${e.field}: ${e.message}`)));
  }
}
```

Create `web/src/experiments/results-table.ts`:
```ts
import { h } from '../ui/dom';
import type { Summary } from './types';

const fmt = (v: number | null): string => (v === null ? '—' : Number(v.toPrecision(4)).toString());

const cells = (tag: 'th' | 'td', values: string[]): HTMLTableCellElement[] => values.map((v) => h(tag, {}, v));

/** The summary as a table; collapsed for time series, which can be long. */
export function resultsTable(summary: Summary): HTMLElement {
  const table =
    summary.kind === 'scalar'
      ? h(
          'table',
          {},
          h('thead', {}, h('tr', {}, ...cells('th', ['Line', 'x', 'n', 'mean', 'sd', 'min', 'max']))),
          h(
            'tbody',
            {},
            ...summary.rows.map((r) =>
              h('tr', {}, ...cells('td', [r.series_name, fmt(r.at), String(r.n), fmt(r.mean), fmt(r.sd), fmt(r.min), fmt(r.max)])),
            ),
          ),
        )
      : h(
          'table',
          {},
          h('thead', {}, h('tr', {}, ...cells('th', ['Line', 't', 'n', 'mean', 'sd']))),
          h(
            'tbody',
            {},
            ...summary.rows.map((r) => h('tr', {}, ...cells('td', [r.series_name, String(r.t), String(r.n), fmt(r.mean), fmt(r.sd)]))),
          ),
        );
  return h('details', { class: 'sweep-table', open: summary.kind === 'scalar' }, h('summary', {}, 'Summary table'), table);
}
```

- [ ] **Step 3: The view**

Create `web/src/experiments/view.ts`:
```ts
import { aggregate, builtin_sweeps, sweep_points } from '../wasm-pkg/sugarscape.js';
import { parseErrors, type FieldError } from '../types';
import { h } from '../ui/dom';
import { FixedPanel } from './fixed-panel';
import { poolSize, WorkerPool, type WorkerLike } from './pool';
import { resultsTable } from './results-table';
import type { BuiltinSweep, Point, RunResult, Summary, Sweep } from './types';

/** Each worker loads its own WASM instance (Decision 19). */
const createWorker = (): WorkerLike => new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });

/** Partial results are re-aggregated and redrawn at most this often. */
const REDRAW_MS = 250;

/** The sweep being edited: the form, or a fixed sweep whose seeds and ticks alone change. */
export interface SweepEditor {
  readonly el: HTMLElement;
  /** The sweep, or null (with its errors shown) when the editor cannot build one. */
  sweep(): Sweep | null;
  showErrors(errors: FieldError[]): void;
}

/** Runs on screen and the sweep they belong to (with its JSON, as sent to WASM). */
interface Shown { sweep: Sweep; spec: string; runs: RunResult[] }

/** The Experiments view: pick a sweep, run it on workers, show the results. */
export class ExperimentsView {
  readonly el: HTMLElement;
  private readonly builtins = JSON.parse(builtin_sweeps()) as BuiltinSweep[];
  private readonly picker: HTMLSelectElement;
  private readonly editorSlot = h('div', { class: 'sweep-editor' });
  private editor: SweepEditor | null = null;
  private readonly runButton = h('button', { class: 'primary', onclick: () => void this.run() }, 'Run');
  private readonly cancelButton = h('button', { disabled: true, onclick: () => this.pool?.cancel() }, 'Cancel');
  private readonly bar = h('progress', { max: 1, value: 0 });
  private readonly status = h('span', { class: 'hint', role: 'status' });
  private readonly table = h('div');
  private pool: WorkerPool | null = null;
  private shown: Shown | null = null;

  constructor() {
    this.picker = h(
      'select',
      { 'aria-label': 'Sweep', onchange: () => this.pick(this.picker.value) },
      h('optgroup', { label: 'Built-in' }, ...this.builtins.map((b) => h('option', { value: `builtin:${b.id}` }, b.sweep.name))),
    );
    this.el = h(
      'div',
      { class: 'experiments-view' },
      h(
        'section',
        { class: 'sweep-setup' },
        h('div', { class: 'row' }, h('label', {}, 'Sweep ', this.picker)),
        this.editorSlot,
        h('div', { class: 'run-bar' }, this.runButton, this.cancelButton, this.bar, this.status),
      ),
      h('section', { class: 'sweep-results' }, this.table),
    );
    this.pick(this.picker.value);
  }

  private pick(choice: string): void {
    const builtin = this.builtins.find((b) => choice === `builtin:${b.id}`);
    if (builtin) this.setEditor(new FixedPanel(builtin.sweep, () => this.validate()));
  }

  private setEditor(editor: SweepEditor): void {
    this.editor = editor;
    this.editorSlot.replaceChildren(editor.el);
    this.validate();
  }

  /** Checks the editor's sweep with the core, shows its errors or size, and returns it when valid. */
  private validate(): { sweep: Sweep; spec: string; count: number } | null {
    const editor = this.editor;
    const sweep = editor?.sweep() ?? null;
    if (!editor || !sweep) {
      this.status.textContent = '';
      return null;
    }
    const spec = JSON.stringify(sweep);
    try {
      const count = (JSON.parse(sweep_points(spec)) as Point[]).length;
      editor.showErrors([]);
      this.status.textContent = `${count} runs`;
      return { sweep, spec, count };
    } catch (e) {
      editor.showErrors(parseErrors(e));
      this.status.textContent = '';
      return null;
    }
  }

  private async run(): Promise<void> {
    if (this.pool) return;
    const checked = this.validate();
    if (!checked) return;
    const { sweep, spec, count } = checked;
    const shown: Shown = { sweep, spec, runs: [] };
    this.shown = shown;
    this.showResults();
    const pool = new WorkerPool(poolSize(navigator.hardwareConcurrency), createWorker);
    this.pool = pool;
    this.setRunning(true);
    this.bar.max = count;
    this.bar.value = 0;
    let dirty = false;
    const timer = setInterval(() => {
      if (dirty) {
        dirty = false;
        this.showResults();
      }
    }, REDRAW_MS);
    try {
      const outcome = await pool.run(spec, count, (run) => {
        shown.runs.push(run);
        this.bar.value = shown.runs.length;
        this.status.textContent = `${shown.runs.length} / ${count} runs`;
        dirty = true;
      });
      this.status.textContent =
        outcome === 'done' ? `${count} runs done` : `Cancelled after ${shown.runs.length} of ${count} runs: the results are incomplete`;
    } catch (e) {
      this.status.textContent = `The sweep failed: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      clearInterval(timer);
      this.pool = null;
      this.setRunning(false);
      this.showResults();
    }
  }

  private setRunning(on: boolean): void {
    this.runButton.disabled = on;
    this.cancelButton.disabled = !on;
    this.picker.disabled = on;
  }

  /** Re-aggregates the runs on screen with the core and redraws them. */
  private showResults(): void {
    const shown = this.shown;
    if (!shown || shown.runs.length === 0) {
      this.table.replaceChildren();
      return;
    }
    const summary = JSON.parse(aggregate(shown.spec, JSON.stringify(shown.runs))) as Summary;
    this.table.replaceChildren(resultsTable(summary));
  }
}
```

- [ ] **Step 4: The page and the switch**

In `web/index.html`, replace `<main class="layout">` with `<main id="playground" class="layout">`, and add after that element's closing `</main>`:
```html
    <main id="experiments" class="experiments" hidden></main>
```

In `web/src/main.ts`, add the import (after `import { decodeShare, encodeShare, readHash } from './share';`):
```ts
import { ExperimentsView } from './experiments/view';
```
and after `document.querySelector('#toolbar')!.append(buildToolbar(engine));` add:
```ts
  const experiments = new ExperimentsView();
  document.querySelector('#experiments')!.append(experiments.el);
  const views = { playground: 'Playground', experiments: 'Experiments' } as const;
  type View = keyof typeof views;
  const viewButtons = (Object.keys(views) as View[]).map((view) =>
    h('button', { 'data-view': view, onclick: () => showView(view) }, views[view]),
  );
  const showView = (view: View): void => {
    // The playground's world is kept, paused, while Experiments is shown.
    if (view === 'experiments') engine.setRunning(false);
    document.body.dataset.view = view;
    document.querySelector<HTMLElement>('#playground')!.hidden = view !== 'playground';
    document.querySelector<HTMLElement>('#experiments')!.hidden = view !== 'experiments';
    for (const b of viewButtons) b.setAttribute('aria-pressed', String(b.dataset.view === view));
  };
  document
    .querySelector('.toolbar h1')!
    .after(h('div', { class: 'view-switch', role: 'group', 'aria-label': 'View' }, ...viewButtons));
  showView('playground');
```
(The `experiments` constant is used again in Task 14.)

- [ ] **Step 5: Styles**

Append to `web/src/style.css` (these also cover the chart, form and outputs of Tasks 11–14):
```css
.view-switch { display: flex; }
.view-switch button { border-radius: 0; }
.view-switch button:first-child { border-radius: 6px 0 0 6px; }
.view-switch button:last-child { border-radius: 0 6px 6px 0; margin-left: -1px; }
body[data-view='experiments'] .toolbar .group,
body[data-view='experiments'] .toolbar .readout,
body[data-view='experiments'] .toolbar-end { display: none; }
.experiments { padding: 16px; }
.experiments-view { display: grid; grid-template-columns: 380px minmax(0, 1fr); gap: 16px; align-items: start; }
.sweep-setup { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 12px 14px; display: grid; gap: 10px; }
.sweep-setup .row { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.sweep-setup select { max-width: 100%; }
.sweep-fixed h2 { font-size: 15px; margin: 0; }
.sweep-fixed dl { display: grid; grid-template-columns: auto 1fr; gap: 2px 10px; margin: 6px 0; }
.sweep-fixed dt { color: var(--muted); }
.sweep-fixed dd { margin: 0; }
.sweep-form { display: grid; gap: 6px; }
.sweep-form fieldset { border: 1px solid var(--border); border-radius: 6px; padding: 6px 10px; margin: 0; display: grid; gap: 4px; }
.sweep-form .control { display: grid; gap: 2px; }
.sweep-form .row { display: flex; gap: 6px; align-items: center; }
.sweep-form input[type='text'] { width: 100%; }
.sweep-form .num { width: 7em; }
.run-bar { display: flex; gap: 8px; align-items: center; flex-wrap: wrap; }
.run-bar progress { flex: 1; min-width: 6em; }
.sweep-results { display: grid; gap: 12px; min-width: 0; }
.sweep-chart { margin: 0; }
.sweep-table { font-size: 12px; overflow-x: auto; }
.sweep-table table { border-collapse: collapse; }
.sweep-table th, .sweep-table td { padding: 2px 8px; text-align: right; font-variant-numeric: tabular-nums; font-weight: normal; }
.sweep-table th:first-child, .sweep-table td:first-child { text-align: left; }
.sweep-table th { color: var(--muted); }
.sweep-outputs { display: flex; gap: 6px; flex-wrap: wrap; }
@media (max-width: 860px) {
  .experiments-view { grid-template-columns: 1fr; }
}
```

- [ ] **Step 6: Verify**

```bash
(cd web && npm run build && npm test)
ls web/dist/assets
grep -l "run_point" web/dist/assets/*.js
```
Expected: build and tests pass (3 new label tests). `web/dist/assets` now holds a separate worker chunk (its name starts with `worker`), and the `grep` lists that chunk: Vite found `new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' })` and bundled the worker with its own copy of the wasm-bindgen glue.

- [ ] **Step 7: Commit**

```bash
git add web/src/experiments/labels.ts web/src/experiments/labels.test.ts web/src/experiments/fixed-panel.ts web/src/experiments/results-table.ts web/src/experiments/view.ts web/index.html web/src/main.ts web/src/style.css
git commit -m "Add the Experiments view: switch, built-in picker, runs on workers" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 11: The sweep chart

*Mechanical (full code).*

**Files:**
- Create: `web/src/experiments/chart-data.ts`, `web/src/experiments/chart-data.test.ts`, `web/src/experiments/chart.ts`
- Modify: `web/src/experiments/view.ts`

**Interfaces:**
- Consumes: `Summary`, `Sweep` (Task 9), `axisLabel`, `metricLabel` (Task 10), `compactNumber`, uPlot.
- Produces:
  - `chart-data.ts`: `interface ChartLine { name: string; mean, sd, lo, hi, min, max: (number | null)[]; n: number[] }`, `interface ChartData { xLabel: string; yLabel: string; x: number[]; lines: ChartLine[] }`, `chartData(sweep: Sweep, summary: Summary): ChartData`, `describePoint(line: ChartLine, i: number): string`
  - `chart.ts`: `class SweepChart { readonly el: HTMLElement; draw(data: ChartData): void; clear(): void; canvas(): HTMLCanvasElement | null }`

- [ ] **Step 1: Write the failing tests**

Create `web/src/experiments/chart-data.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { chartData, describePoint } from './chart-data';
import type { ScalarRow, Summary, Sweep } from './types';

const sweep: Sweep = {
  name: 's',
  base: { preset: 'p' },
  x: { label: 'Mean vision', values: [] },
  seeds: { from: 1, count: 2 },
  ticks: 500,
  metric: { kind: 'window_mean', series: 'population', from: 400 },
};

const row = (series: number, name: string, x: number, at: number, mean: number | null, sd: number | null): ScalarRow => ({
  series,
  series_name: name,
  x,
  at,
  n: mean === null ? 0 : 2,
  nan: 0,
  mean,
  sd,
  min: mean,
  max: mean,
});

describe('chart data', () => {
  it('maps scalar rows to one line per series, x sorted by at', () => {
    const summary: Summary = {
      kind: 'scalar',
      rows: [row(0, 'a', 0, 3, 10, 1), row(0, 'a', 1, 1, 20, 2), row(1, 'b', 0, 3, 30, 0), row(1, 'b', 1, 1, null, null)],
    };
    const data = chartData(sweep, summary);
    expect(data.x).toEqual([1, 3]);
    expect(data.xLabel).toBe('Mean vision');
    expect(data.yLabel).toBe('population, mean over t = 400–500');
    expect(data.lines.map((l) => l.name)).toEqual(['a', 'b']);
    expect(data.lines[0].mean).toEqual([20, 10]);
    expect(data.lines[0].lo).toEqual([18, 9]);
    expect(data.lines[0].hi).toEqual([22, 11]);
    expect(data.lines[1].mean).toEqual([null, 30]);
    expect(data.lines[1].lo).toEqual([null, 30]);
    expect(data.lines[1].n).toEqual([0, 2]);
  });

  it('maps time-series blocks to ticks', () => {
    const ts: Sweep = { ...sweep, metric: { kind: 'timeseries', series: 'sd_log_price', every: 50 } };
    const summary: Summary = {
      kind: 'timeseries',
      rows: [
        { series: 0, series_name: 'short', t: 50, n: 2, mean: 0.5, sd: 0.1 },
        { series: 0, series_name: 'short', t: 100, n: 2, mean: 0.4, sd: 0.1 },
        { series: 1, series_name: 'long', t: 50, n: 2, mean: 0.3, sd: null },
        { series: 1, series_name: 'long', t: 100, n: 2, mean: 0.2, sd: 0 },
      ],
    };
    const data = chartData(ts, summary);
    expect(data.x).toEqual([50, 100]);
    expect(data.xLabel).toBe('Tick');
    expect(data.yLabel).toBe('sd_log_price, 50-tick means');
    expect(data.lines.map((l) => l.name)).toEqual(['short', 'long']);
    expect(data.lines[1].mean).toEqual([0.3, 0.2]);
    expect(data.lines[1].lo).toEqual([null, 0.2]);
    expect(data.lines[0].min).toEqual([null, null]);
  });

  it('describes a hovered point', () => {
    const data = chartData(sweep, { kind: 'scalar', rows: [row(0, 'a', 0, 1, 12.3456, 1.5)] });
    expect(describePoint(data.lines[0], 0)).toBe('12.35 ± 1.5, 12.35–12.35, n = 2');
    const empty = chartData(sweep, { kind: 'scalar', rows: [row(0, 'a', 0, 1, null, null)] });
    expect(describePoint(empty.lines[0], 0)).toBe('— (n = 0)');
  });
});
```

Run: `(cd web && npx vitest run src/experiments/chart-data.test.ts)` — Expected: fails (`./chart-data` not found).

- [ ] **Step 2: Implement the mapping**

Create `web/src/experiments/chart-data.ts`:
```ts
import { axisLabel, metricLabel } from './labels';
import type { Summary, Sweep } from './types';

/** One chart line: per x position, the mean over seeds and its ±1 sd band. `null` is a gap. */
export interface ChartLine {
  name: string;
  mean: (number | null)[];
  sd: (number | null)[];
  lo: (number | null)[];
  hi: (number | null)[];
  min: (number | null)[];
  max: (number | null)[];
  n: number[];
}

export interface ChartData { xLabel: string; yLabel: string; x: number[]; lines: ChartLine[] }

interface Cell { mean: number | null; sd: number | null; n: number; min?: number | null; max?: number | null }

const finite = (v: number | null | undefined): number | null => (typeof v === 'number' && Number.isFinite(v) ? v : null);

const EMPTY: Cell = { mean: null, sd: null, n: 0 };

function lineOf(name: string, cells: Cell[]): ChartLine {
  const line: ChartLine = { name, mean: [], sd: [], lo: [], hi: [], min: [], max: [], n: [] };
  for (const cell of cells) {
    const mean = finite(cell.mean);
    const sd = finite(cell.sd);
    line.mean.push(mean);
    line.sd.push(sd);
    line.lo.push(mean !== null && sd !== null ? mean - sd : null);
    line.hi.push(mean !== null && sd !== null ? mean + sd : null);
    line.min.push(finite(cell.min));
    line.max.push(finite(cell.max));
    line.n.push(cell.n);
  }
  return line;
}

/** The core's summary as chart arrays: x = `at` (sorted) for scalar metrics, the tick for time series. */
export function chartData(sweep: Sweep, summary: Summary): ChartData {
  const yLabel = metricLabel(sweep);
  if (summary.kind === 'timeseries') {
    const rows = summary.rows;
    const ticks = [...new Set(rows.map((r) => r.t))].sort((a, b) => a - b);
    const lines = [...new Set(rows.map((r) => r.series))].map((s) => {
      const own = rows.filter((r) => r.series === s);
      return lineOf(
        own[0].series_name,
        ticks.map((t) => own.find((r) => r.t === t) ?? EMPTY),
      );
    });
    return { xLabel: 'Tick', yLabel, x: ticks, lines };
  }
  const rows = summary.rows;
  const positions = [...new Map(rows.map((r) => [r.x, r.at] as const)).entries()].sort((a, b) => a[1] - b[1]);
  const lines = [...new Set(rows.map((r) => r.series))].map((s) => {
    const own = rows.filter((r) => r.series === s);
    return lineOf(
      own[0].series_name,
      positions.map(([x]) => own.find((r) => r.x === x) ?? EMPTY),
    );
  });
  return { xLabel: axisLabel(sweep.x), yLabel, x: positions.map(([, at]) => at), lines };
}

const fmt = (v: number | null | undefined): string =>
  v === null || v === undefined ? '—' : Number(v.toPrecision(4)).toString();

/** The legend text for a hovered point: mean ± sd, the range over seeds where known, and n. */
export function describePoint(line: ChartLine, i: number): string {
  const mean = line.mean[i];
  const n = line.n[i] ?? 0;
  if (mean === null || mean === undefined) return `— (n = ${n})`;
  const min = line.min[i];
  const max = line.max[i];
  const range = min !== null && min !== undefined && max !== null && max !== undefined ? `, ${fmt(min)}–${fmt(max)}` : '';
  return `${fmt(mean)} ± ${fmt(line.sd[i])}${range}, n = ${n}`;
}
```

Run: `(cd web && npx vitest run src/experiments/chart-data.test.ts)` — Expected: 3 passed.

- [ ] **Step 3: The uPlot chart**

Create `web/src/experiments/chart.ts`:
```ts
import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';
import { h } from '../ui/dom';
import { compactNumber } from '../ui/format';
import { describePoint, type ChartData } from './chart-data';

const HEIGHT = 360;
/** Line colors after --c1…--c4 (Decision 21). */
const EXTRA_COLORS = ['#b8860b', '#17a2b8', '#d63384', '#6c757d'];

/** One line per series (the mean over seeds) with a ±1 sd band; the legend shows mean, sd, range and n on hover. */
export class SweepChart {
  readonly el = h('figure', { class: 'chart sweep-chart' });
  private plot: uPlot | null = null;
  /** Labels and line names of the plot on screen; a change rebuilds it. */
  private signature = '';
  private data: ChartData | null = null;

  constructor() {
    new ResizeObserver(() => this.plot?.setSize({ width: this.width(), height: HEIGHT })).observe(this.el);
  }

  draw(data: ChartData): void {
    this.data = data;
    const aligned: uPlot.AlignedData = [data.x, ...data.lines.map((l) => l.mean)];
    const signature = JSON.stringify([data.xLabel, data.yLabel, data.lines.map((l) => l.name)]);
    if (this.plot && signature === this.signature) {
      this.plot.setData(aligned);
      return;
    }
    this.plot?.destroy();
    this.signature = signature;
    this.plot = new uPlot(this.options(data), aligned, this.el);
  }

  clear(): void {
    this.plot?.destroy();
    this.plot = null;
    this.signature = '';
    this.data = null;
  }

  canvas(): HTMLCanvasElement | null {
    return this.plot?.ctx.canvas ?? null;
  }

  private width(): number {
    return Math.max(320, this.el.clientWidth - 4);
  }

  private options(data: ChartData): uPlot.Options {
    const css = getComputedStyle(document.documentElement);
    const token = (name: string, fallback: string) => css.getPropertyValue(name).trim() || fallback;
    const colors = data.lines.map((_, k) => (k < 4 ? token(`--c${k + 1}`, '#888') : EXTRA_COLORS[(k - 4) % EXTRA_COLORS.length]));
    const axis = { stroke: token('--muted', '#888'), grid: { stroke: token('--grid', '#ddd') }, ticks: { stroke: token('--grid', '#ddd') } };
    return {
      width: this.width(),
      height: HEIGHT,
      scales: { x: { time: false }, y: { range: (_u, min, max) => this.yRange(min, max) } },
      axes: [
        { ...axis, label: data.xLabel },
        { ...axis, label: data.yLabel, size: 56, values: (_u, splits) => splits.map(compactNumber) },
      ],
      series: [
        { label: data.xLabel },
        ...data.lines.map((line, k) => ({
          label: line.name,
          stroke: colors[k],
          width: 2,
          points: { show: true, size: 5 },
          value: (_u: uPlot, _v: number | null, _s: number, i: number | null) =>
            i === null || !this.data ? '—' : describePoint(this.data.lines[k], i),
        })),
      ],
      hooks: { drawAxes: [(u: uPlot) => this.drawBands(u, colors)] },
    };
  }

  /** The data range widened to cover the bands, padded by 5%. */
  private yRange(min: number, max: number): [number, number] {
    let lo = Number.isFinite(min) ? min : Infinity;
    let hi = Number.isFinite(max) ? max : -Infinity;
    for (const line of this.data?.lines ?? []) {
      for (const v of line.lo) if (v !== null && v < lo) lo = v;
      for (const v of line.hi) if (v !== null && v > hi) hi = v;
    }
    if (!Number.isFinite(lo) || !Number.isFinite(hi)) return [0, 1];
    if (lo === hi) return [lo - 1, hi + 1];
    const pad = (hi - lo) * 0.05;
    return [lo - pad, hi + pad];
  }

  /** Paints each line's mean ± sd band under the lines (Decision 21); gaps split a band. */
  private drawBands(u: uPlot, colors: string[]): void {
    const data = this.data;
    if (!data) return;
    const ctx = u.ctx;
    ctx.save();
    ctx.beginPath();
    ctx.rect(u.bbox.left, u.bbox.top, u.bbox.width, u.bbox.height);
    ctx.clip();
    ctx.globalAlpha = 0.18;
    data.lines.forEach((line, k) => {
      ctx.fillStyle = colors[k];
      let run: [number, number, number][] = [];
      const flush = () => {
        if (run.length > 1) {
          ctx.beginPath();
          run.forEach(([x, hi], i) => (i === 0 ? ctx.moveTo(x, hi) : ctx.lineTo(x, hi)));
          for (let i = run.length - 1; i >= 0; i--) ctx.lineTo(run[i][0], run[i][2]);
          ctx.closePath();
          ctx.fill();
        }
        run = [];
      };
      data.x.forEach((xv, i) => {
        const hi = line.hi[i];
        const lo = line.lo[i];
        if (hi === null || lo === null) {
          flush();
          return;
        }
        run.push([u.valToPos(xv, 'x', true), u.valToPos(hi, 'y', true), u.valToPos(lo, 'y', true)]);
      });
      flush();
    });
    ctx.restore();
  }
}
```

- [ ] **Step 4: Show it in the view**

In `web/src/experiments/view.ts`:

1. After `import { h } from '../ui/dom';` add:
```ts
import { SweepChart } from './chart';
import { chartData } from './chart-data';
```
2. Replace `  private readonly table = h('div');` with:
```ts
  private readonly chart = new SweepChart();
  private readonly table = h('div');
```
3. Replace `      h('section', { class: 'sweep-results' }, this.table),` with:
```ts
      h('section', { class: 'sweep-results' }, this.chart.el, this.table),
```
4. Replace the whole `showResults` method with:
```ts
  /** Re-aggregates the runs on screen with the core and redraws them. */
  private showResults(): void {
    const shown = this.shown;
    if (!shown || shown.runs.length === 0) {
      this.chart.clear();
      this.table.replaceChildren();
      return;
    }
    const summary = JSON.parse(aggregate(shown.spec, JSON.stringify(shown.runs))) as Summary;
    this.chart.draw(chartData(shown.sweep, summary));
    this.table.replaceChildren(resultsTable(summary));
  }
```

- [ ] **Step 5: Verify** — `(cd web && npm run build && npm test)`: build passes, 3 new chart-data tests pass.

- [ ] **Step 6: Commit**

```bash
git add web/src/experiments/chart-data.ts web/src/experiments/chart-data.test.ts web/src/experiments/chart.ts web/src/experiments/view.ts
git commit -m "Chart sweep results with mean lines and sd bands" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 12: Value lists and form ↔ sweep conversion

*Mechanical (full code).*

**Files:**
- Create: `web/src/experiments/values.ts`, `web/src/experiments/values.test.ts`, `web/src/experiments/form.ts`, `web/src/experiments/form.test.ts`

**Interfaces:**
- Consumes: `Sweep`, `Axis`, `ShorthandAxis`, `Metric`, `SweepBase` (Task 9); `Config`, `FieldError`.
- Produces:
  - `values.ts`: `type AxisScalar = number | boolean`, `type Parsed = { values: AxisScalar[] } | { error: string }`, `parseValues(text: string, max: number): Parsed`, `formatValues(values: AxisScalar[]): string`
  - `form.ts`: `interface AxisForm { path: string; values: string }`, `interface MetricForm { kind: Metric['kind']; series: string; from: number; to: number | null; every: number }`, `interface SweepForm { name: string; x: AxisForm; series: AxisForm | null; seeds: number; ticks: number; metric: MetricForm }`, `MAX_X_VALUES = 64`, `MAX_SERIES_VALUES = 16`, `TIMESERIES_X: Axis`, `defaultForm(): SweepForm`, `formToSweep(form: SweepForm, base: SweepBase): { sweep: Sweep | null; errors: FieldError[] }`, `axisToForm(axis: Axis | ShorthandAxis): AxisForm | null`, `sweepToForm(sweep: Sweep): SweepForm | null`, `controlFor(field: string): string`, `numericPaths(config: Config): string[]` (Decision 18).

- [ ] **Step 1: Write the failing tests**

Create `web/src/experiments/values.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { formatValues, parseValues } from './values';

describe('value lists', () => {
  it('reads comma lists of numbers', () => {
    expect(parseValues('1, 2.5, -3, 1e3', 64)).toEqual({ values: [1, 2.5, -3, 1000] });
  });

  it('reads booleans', () => {
    expect(parseValues('false,true', 64)).toEqual({ values: [false, true] });
  });

  it('reads inclusive ranges with tidy steps', () => {
    expect(parseValues('1:6:1', 64)).toEqual({ values: [1, 2, 3, 4, 5, 6] });
    expect(parseValues('0:0.3:0.1', 64)).toEqual({ values: [0, 0.1, 0.2, 0.3] });
    expect(parseValues(' 2 : 3 : 5 ', 64)).toEqual({ values: [2] });
    expect(parseValues('-1:1:1', 64)).toEqual({ values: [-1, 0, 1] });
  });

  it('rejects bad input', () => {
    for (const text of ['', '1,,2', 'a', '1, true', '1:2', '1:2:0', '3:1:1', '1:x:1', 'true:false:1']) {
      expect('error' in parseValues(text, 64), text).toBe(true);
    }
  });

  it('enforces the axis limit', () => {
    expect(parseValues('1:64:1', 64)).toHaveProperty('values');
    expect(parseValues('1:65:1', 64)).toEqual({ error: 'At most 64 values' });
    expect(parseValues(Array.from({ length: 17 }, (_, i) => i).join(','), 16)).toEqual({ error: 'At most 16 values' });
  });

  it('formats values back', () => {
    expect(formatValues([1, 2.5, true])).toBe('1, 2.5, true');
  });
});
```

Create `web/src/experiments/form.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import type { Config } from '../types';
import { controlFor, formToSweep, numericPaths, sweepToForm, TIMESERIES_X, type SweepForm } from './form';
import type { Sweep } from './types';

// every = round(ticks / 20) and from = ticks − 100: what sweepToForm fills in for unused metric fields.
const form: SweepForm = {
  name: 'Vision',
  x: { path: 'vision.max', values: '1, 2, 3' },
  series: { path: 'trade.enabled', values: 'false, true' },
  seeds: 4,
  ticks: 300,
  metric: { kind: 'window_mean', series: 'population', from: 200, to: null, every: 15 },
};

describe('form → sweep', () => {
  it('writes shorthand axes', () => {
    const { sweep, errors } = formToSweep(form, { preset: 'iv-3-trade' });
    expect(errors).toEqual([]);
    expect(sweep).toEqual({
      name: 'Vision',
      base: { preset: 'iv-3-trade' },
      x: { path: 'vision.max', values: [1, 2, 3] },
      series: { path: 'trade.enabled', values: [false, true] },
      seeds: { from: 1, count: 4 },
      ticks: 300,
      metric: { kind: 'window_mean', series: 'population', from: 200 },
    });
  });

  it('keeps an explicit window end and writes the other metric kinds', () => {
    const final = formToSweep({ ...form, metric: { ...form.metric, kind: 'final' } }, { preset: 'p' }).sweep!;
    expect(final.metric).toEqual({ kind: 'final', series: 'population' });
    const windowed = formToSweep({ ...form, metric: { ...form.metric, to: 250 } }, { preset: 'p' }).sweep!;
    expect(windowed.metric).toEqual({ kind: 'window_mean', series: 'population', from: 200, to: 250 });
  });

  it('gives a time series its single implicit x value', () => {
    const ts = formToSweep({ ...form, x: { path: '', values: '' }, metric: { ...form.metric, kind: 'timeseries' } }, { preset: 'p' });
    expect(ts.errors).toEqual([]);
    expect(ts.sweep!.x).toEqual(TIMESERIES_X);
    expect(ts.sweep!.metric).toEqual({ kind: 'timeseries', series: 'population', every: 15 });
  });

  it('reports path and value errors on their controls', () => {
    const { sweep, errors } = formToSweep(
      { ...form, x: { path: '', values: 'a' }, series: { path: 's', values: '1:2' } },
      { preset: 'p' },
    );
    expect(sweep).toBeNull();
    expect(errors.map((e) => e.field)).toEqual(['x.path', 'x.values', 'series.values']);
  });

  it('drops the second axis when unset', () => {
    expect(formToSweep({ ...form, series: null }, { preset: 'p' }).sweep).not.toHaveProperty('series');
  });
});

describe('sweep → form', () => {
  it('round-trips a form-made sweep', () => {
    expect(sweepToForm(formToSweep(form, { preset: 'iv-3-trade' }).sweep!)).toEqual(form);
  });

  it('reads the full axes the core writes', () => {
    const full: Sweep = {
      ...formToSweep(form, { preset: 'p' }).sweep!,
      x: { label: 'vision.max', values: [1, 2, 3].map((v) => ({ at: v, set: { 'vision.max': v } })) },
      series: { label: 'trade.enabled', values: [false, true].map((v, i) => ({ at: i, set: { 'trade.enabled': v } })) },
    };
    expect(sweepToForm(full)).toEqual(form);
  });

  it('round-trips a time series', () => {
    const ts: SweepForm = { ...form, x: { path: '', values: '' }, metric: { ...form.metric, kind: 'timeseries' } };
    expect(sweepToForm(formToSweep(ts, { preset: 'p' }).sweep!)).toEqual(ts);
  });

  it('declines sweeps the form cannot express', () => {
    const base = formToSweep(form, { preset: 'p' }).sweep!;
    const ranges: Sweep = { ...base, x: { label: 'Mean vision', values: [{ at: 1, set: { vision: { min: 1, max: 1 } } }] } };
    expect(sweepToForm(ranges)).toBeNull();
    const named: Sweep = { ...base, series: { label: 'trade.enabled', values: [{ at: 0, name: 'No trade', set: { 'trade.enabled': false } }] } };
    expect(sweepToForm(named)).toBeNull();
    expect(sweepToForm({ ...base, set: { population: 500 } })).toBeNull();
    expect(sweepToForm({ ...base, seeds: { from: 3, count: 2 } })).toBeNull();
  });
});

describe('error placement', () => {
  it('maps core fields to form controls', () => {
    expect(controlFor('x[2].set.vision.max')).toBe('x.path');
    expect(controlFor('series[0].set.trade.enabled')).toBe('series.path');
    expect(controlFor('x.values')).toBe('x.values');
    expect(controlFor('metric.from')).toBe('metric.from');
    expect(controlFor('seeds.count')).toBe('seeds.count');
    expect(controlFor('points')).toBe('general');
    expect(controlFor('base')).toBe('general');
  });
});

describe('path suggestions', () => {
  it('lists numeric and boolean leaves, skipping the schedule and outbreaks', () => {
    const config = {
      population: 400,
      vision: { min: 1, max: 6 },
      trade: { enabled: false },
      goods: [{ name: 'sugar', metabolism: { min: 1, max: 4 } }],
      schedule: [{ tick: 5, set: {} }],
      disease: { enabled: false, outbreaks: [{ tick: 1, agents: 2 }] },
    } as unknown as Config;
    expect(numericPaths(config)).toEqual([
      'population',
      'vision.min',
      'vision.max',
      'trade.enabled',
      'goods.0.metabolism.min',
      'goods.0.metabolism.max',
      'disease.enabled',
    ]);
  });
});
```

Run: `(cd web && npx vitest run src/experiments/values.test.ts src/experiments/form.test.ts)` — Expected: fails (modules not found).

- [ ] **Step 2: Implement**

Create `web/src/experiments/values.ts`:
```ts
export type AxisScalar = number | boolean;
export type Parsed = { values: AxisScalar[] } | { error: string };

const NUMBER = /^[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?$/;

function number(token: string): number | null {
  return NUMBER.test(token) ? Number(token) : null;
}

/** Tidies float steps: 0 + 3 × 0.1 is 0.3, not 0.30000000000000004. */
function tidy(v: number): number {
  return Number(v.toPrecision(12));
}

/**
 * Parses an axis's values: `a, b, c` (all numbers or all true/false) or
 * `from:to:step` (numbers, step > 0, `to` included when on a step).
 */
export function parseValues(text: string, max: number): Parsed {
  const t = text.trim();
  if (t === '') return { error: 'Enter values: a, b, c or from:to:step' };
  if (t.includes(':')) {
    const parts = t.split(':').map((s) => s.trim());
    if (parts.length !== 3) return { error: 'A range is from:to:step' };
    const [from, to, step] = parts.map(number);
    if (from === null || to === null || step === null) return { error: 'A range needs three numbers' };
    if (!(step > 0)) return { error: 'The step must be > 0' };
    if (to < from) return { error: 'The range must not end before it starts' };
    const count = Math.floor((to - from) / step + 1e-9) + 1;
    if (count > max) return { error: `At most ${max} values` };
    return { values: Array.from({ length: count }, (_, k) => tidy(from + k * step)) };
  }
  const values: AxisScalar[] = [];
  for (const token of t.split(',').map((s) => s.trim())) {
    if (token === 'true' || token === 'false') {
      values.push(token === 'true');
      continue;
    }
    const n = number(token);
    if (n === null) return { error: `"${token}" is not a number, true or false` };
    values.push(n);
  }
  if (new Set(values.map((v) => typeof v)).size > 1) return { error: 'Use numbers or true/false, not both' };
  if (values.length > max) return { error: `At most ${max} values` };
  return { values };
}

export function formatValues(values: AxisScalar[]): string {
  return values.map(String).join(', ');
}
```

Create `web/src/experiments/form.ts`:
```ts
import type { Config, FieldError } from '../types';
import type { Axis, Metric, ShorthandAxis, Sweep, SweepBase } from './types';
import { formatValues, parseValues, type AxisScalar } from './values';

export interface AxisForm { path: string; values: string }
export interface MetricForm { kind: Metric['kind']; series: string; from: number; to: number | null; every: number }
export interface SweepForm {
  name: string;
  x: AxisForm;
  /** The second axis: one line per value. */
  series: AxisForm | null;
  seeds: number;
  ticks: number;
  metric: MetricForm;
}

export const MAX_X_VALUES = 64;
export const MAX_SERIES_VALUES = 16;

/** A time series' single x value: the chart's x axis is the tick (Decisions 4 and 18). */
export const TIMESERIES_X: Axis = { label: 'All runs', values: [{ at: 0, set: {} }] };

export function defaultForm(): SweepForm {
  return {
    name: 'Untitled sweep',
    x: { path: 'vision.max', values: '1:6:1' },
    series: null,
    seeds: 3,
    ticks: 500,
    metric: { kind: 'window_mean', series: 'population', from: 400, to: null, every: 25 },
  };
}

const isScalar = (v: unknown): v is AxisScalar => typeof v === 'number' || typeof v === 'boolean';

function axisFromForm(axis: AxisForm, field: 'x' | 'series', max: number, errors: FieldError[]): ShorthandAxis | null {
  const path = axis.path.trim();
  if (path === '') errors.push({ field: `${field}.path`, message: 'Enter a config path' });
  const parsed = parseValues(axis.values, max);
  if ('error' in parsed) {
    errors.push({ field: `${field}.values`, message: parsed.error });
    return null;
  }
  return path === '' ? null : { path, values: parsed.values };
}

/** The sweep the form describes (shorthand axes, seeds from 1), or the form's own errors. */
export function formToSweep(form: SweepForm, base: SweepBase): { sweep: Sweep | null; errors: FieldError[] } {
  const errors: FieldError[] = [];
  const m = form.metric;
  const x = m.kind === 'timeseries' ? structuredClone(TIMESERIES_X) : axisFromForm(form.x, 'x', MAX_X_VALUES, errors);
  const series = form.series ? axisFromForm(form.series, 'series', MAX_SERIES_VALUES, errors) : undefined;
  if (errors.length > 0 || !x || series === null) return { sweep: null, errors };
  const metric: Metric =
    m.kind === 'final'
      ? { kind: 'final', series: m.series }
      : m.kind === 'timeseries'
        ? { kind: 'timeseries', series: m.series, every: m.every }
        : { kind: 'window_mean', series: m.series, from: m.from, ...(m.to === null ? {} : { to: m.to }) };
  const sweep: Sweep = {
    name: form.name.trim() || 'Untitled sweep',
    base,
    x,
    seeds: { from: 1, count: form.seeds },
    ticks: form.ticks,
    metric,
  };
  if (series) sweep.series = series;
  return { sweep, errors: [] };
}

/** An axis the form can edit (one path, scalar values at their natural `at`, no line names), or null. */
export function axisToForm(axis: Axis | ShorthandAxis): AxisForm | null {
  if ('path' in axis) {
    if (axis.label !== undefined && axis.label !== axis.path) return null;
    return axis.values.every(isScalar) ? { path: axis.path, values: formatValues(axis.values as AxisScalar[]) } : null;
  }
  const keys = axis.values.map((v) => Object.keys(v.set));
  const path = keys[0]?.[0];
  if (path === undefined || axis.label !== path) return null;
  if (keys.some((k) => k.length !== 1 || k[0] !== path) || axis.values.some((v) => v.name !== undefined)) return null;
  const values = axis.values.map((v) => v.set[path]);
  if (!values.every(isScalar)) return null;
  if (!axis.values.every((v, i) => v.at === (typeof values[i] === 'number' ? values[i] : i))) return null;
  return { path, values: formatValues(values) };
}

function isTimeseriesX(axis: Axis | ShorthandAxis): boolean {
  return !('path' in axis) && axis.values.length === 1 && Object.keys(axis.values[0].set).length === 0;
}

/** The form for a sweep it can express (Decision 18), or null. */
export function sweepToForm(sweep: Sweep): SweepForm | null {
  if (Object.keys(sweep.set ?? {}).length > 0 || sweep.seeds.from !== 1) return null;
  const m = sweep.metric;
  const x: AxisForm | null =
    m.kind === 'timeseries' ? (isTimeseriesX(sweep.x) ? { path: '', values: '' } : null) : axisToForm(sweep.x);
  const series = sweep.series ? axisToForm(sweep.series) : null;
  if (!x || (sweep.series && !series)) return null;
  return {
    name: sweep.name,
    x,
    series,
    seeds: sweep.seeds.count,
    ticks: sweep.ticks,
    metric: {
      kind: m.kind,
      series: m.series,
      from: m.kind === 'window_mean' ? m.from : Math.max(0, sweep.ticks - 100),
      to: m.kind === 'window_mean' ? (m.to ?? null) : null,
      every: m.kind === 'timeseries' ? m.every : Math.max(1, Math.round(sweep.ticks / 20)),
    },
  };
}

const CONTROLS = new Set([
  'name',
  'x.path',
  'x.values',
  'series.path',
  'series.values',
  'seeds.count',
  'ticks',
  'metric.kind',
  'metric.series',
  'metric.from',
  'metric.to',
  'metric.every',
]);

/** The form control that shows a core error: `x[i].set.<path>` → `x.path`; unknown fields → `general`. */
export function controlFor(field: string): string {
  const axis = /^(x|series)\[\d+\]/.exec(field);
  if (axis) return `${axis[1]}.path`;
  return CONTROLS.has(field) ? field : 'general';
}

/** Dotted paths of every number or boolean in a config (the path input's suggestions). */
export function numericPaths(config: Config): string[] {
  const out: string[] = [];
  const walk = (value: unknown, path: string) => {
    if (path === 'schedule' || path === 'disease.outbreaks') return;
    if (typeof value === 'number' || typeof value === 'boolean') {
      out.push(path);
      return;
    }
    if (value && typeof value === 'object') {
      for (const [key, child] of Object.entries(value)) walk(child, path ? `${path}.${key}` : key);
    }
  };
  walk(config, '');
  return out;
}
```

- [ ] **Step 3: Run the tests**

Run: `(cd web && npx vitest run src/experiments/values.test.ts src/experiments/form.test.ts)`
Expected: 6 + 11 passed.

- [ ] **Step 4: Verify** — `(cd web && npm run build && npm test)`

- [ ] **Step 5: Commit**

```bash
git add web/src/experiments/values.ts web/src/experiments/values.test.ts web/src/experiments/form.ts web/src/experiments/form.test.ts
git commit -m "Parse sweep value lists and convert between form and sweep" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---
### Task 13: The sweep form and "From current world"

*Needs judgement (form layout); the code is complete.*

**Files:**
- Create: `web/src/experiments/form-view.ts`
- Modify: `web/src/experiments/view.ts` (full replacement), `web/src/main.ts`

**Interfaces:**
- Consumes: `SweepForm`, `AxisForm`, `formToSweep`, `controlFor`, `defaultForm`, `numericPaths` (Task 12); WASM `config_series_names`; `Engine.{presetId, isModified(), baseConfig, presets, editedLandscapes()}`.
- Produces: `class FormView { readonly el: HTMLElement; constructor(form: SweepForm, base: SweepBase, baseNote: string, paths: string[], seriesNames: string[], onChange: () => void); sweep(): Sweep | null; showErrors(errors: FieldError[]): void }`; `ExperimentsView` now `constructor(engine: Engine)` with the picker entry "From current world" (Decision 17).

- [ ] **Step 1: The form**

Create `web/src/experiments/form-view.ts`:
```ts
import type { FieldError } from '../types';
import { h } from '../ui/dom';
import { controlFor, formToSweep, type AxisForm, type SweepForm } from './form';
import type { Metric, Sweep, SweepBase } from './types';

const KINDS: { value: Metric['kind']; label: string }[] = [
  { value: 'window_mean', label: 'Mean over a window of ticks' },
  { value: 'final', label: 'Value at the last tick' },
  { value: 'timeseries', label: 'Time series (block means)' },
];

/** Suggestions for the config-path inputs. */
const PATH_LIST = 'sweep-config-paths';

/** The editable sweep: a config path and values per axis, seeds, ticks and the metric (Decision 18). */
export class FormView {
  readonly el = h('div', { class: 'sweep-form' });
  private readonly errorEls = new Map<string, HTMLElement>();

  constructor(
    private readonly form: SweepForm,
    private readonly base: SweepBase,
    private readonly baseNote: string,
    private readonly paths: string[],
    private readonly seriesNames: string[],
    private readonly onChange: () => void,
  ) {
    this.render();
  }

  sweep(): Sweep | null {
    const { sweep, errors } = formToSweep(this.form, this.base);
    if (!sweep) this.showErrors(errors);
    return sweep;
  }

  showErrors(errors: FieldError[]): void {
    for (const el of this.errorEls.values()) el.replaceChildren();
    for (const e of errors) {
      const key = controlFor(e.field);
      const el = this.errorEls.get(key) ?? this.errorEls.get('general');
      el?.append(h('p', {}, key === e.field ? e.message : `${e.field}: ${e.message}`));
    }
  }

  private errorSlot(key: string): HTMLElement {
    const el = h('div', { class: 'error' });
    this.errorEls.set(key, el);
    return el;
  }

  private control(label: string, key: string, ...inputs: HTMLElement[]): HTMLElement {
    return h('div', { class: 'control' }, h('span', {}, label), h('div', { class: 'row' }, ...inputs), this.errorSlot(key));
  }

  private text(value: string, set: (v: string) => void, placeholder: string, list?: string): HTMLInputElement {
    const input = h('input', {
      type: 'text',
      value,
      placeholder,
      onchange: () => {
        set(input.value);
        this.onChange();
      },
    });
    // `list` is read-only as a property; it can only be set as an attribute.
    if (list) input.setAttribute('list', list);
    return input;
  }

  private number(value: number | null, set: (v: number | null) => void, props: Record<string, unknown> = {}): HTMLInputElement {
    const input = h('input', {
      type: 'number',
      class: 'num',
      value: value === null ? '' : String(value),
      ...props,
      onchange: () => {
        set(input.value === '' ? null : Number(input.value));
        this.onChange();
      },
    });
    return input;
  }

  private axis(title: string, key: 'x' | 'series', axis: AxisForm): HTMLElement {
    return h(
      'fieldset',
      {},
      h('legend', {}, title),
      this.control('Config path', `${key}.path`, this.text(axis.path, (v) => (axis.path = v), 'e.g. vision.max', PATH_LIST)),
      this.control('Values', `${key}.values`, this.text(axis.values, (v) => (axis.values = v), '1, 2, 3 or 1:6:1')),
    );
  }

  private render(): void {
    this.errorEls.clear();
    const f = this.form;
    const m = f.metric;
    const timeseries = m.kind === 'timeseries';
    const second = h('input', {
      type: 'checkbox',
      checked: f.series !== null,
      onchange: () => {
        f.series = second.checked ? { path: '', values: '' } : null;
        this.render();
        this.onChange();
      },
    });
    const kind = h(
      'select',
      {
        onchange: () => {
          m.kind = kind.value as Metric['kind'];
          this.render();
          this.onChange();
        },
      },
      ...KINDS.map((k) => h('option', { value: k.value, selected: k.value === m.kind }, k.label)),
    );
    const names = this.seriesNames.includes(m.series) ? this.seriesNames : [m.series, ...this.seriesNames];
    const statistic = h(
      'select',
      {
        onchange: () => {
          m.series = statistic.value;
          this.onChange();
        },
      },
      ...names.map((n) => h('option', { value: n, selected: n === m.series }, n)),
    );
    this.el.replaceChildren(
      h('datalist', { id: PATH_LIST }, ...this.paths.map((p) => h('option', { value: p }))),
      this.control('Name', 'name', this.text(f.name, (v) => (f.name = v), 'Untitled sweep')),
      h('p', { class: 'hint' }, `Base: ${this.baseNote}`),
      timeseries
        ? h('p', { class: 'hint' }, 'A time series is plotted against the tick; the optional axis below gives one line per value.')
        : this.axis('x axis', 'x', f.x),
      h('label', {}, second, ' Second axis: one line per value'),
      f.series ? this.axis('Lines', 'series', f.series) : null,
      this.control('Seeds', 'seeds.count', this.number(f.seeds, (v) => (f.seeds = v ?? 0), { min: 1, max: 100 })),
      this.control('Ticks', 'ticks', this.number(f.ticks, (v) => (f.ticks = v ?? 0), { min: 1, max: 100000 })),
      this.control('Metric', 'metric.kind', kind),
      this.control('Statistic', 'metric.series', statistic),
      m.kind === 'window_mean'
        ? this.control('Window from tick', 'metric.from', this.number(m.from, (v) => (m.from = v ?? 0), { min: 0 }))
        : null,
      m.kind === 'window_mean'
        ? this.control('to tick (blank: the last)', 'metric.to', this.number(m.to, (v) => (m.to = v), { min: 0 }))
        : null,
      timeseries
        ? this.control('Block length (ticks)', 'metric.every', this.number(m.every, (v) => (m.every = v ?? 0), { min: 1 }))
        : null,
      this.errorSlot('general'),
    );
  }
}
```

- [ ] **Step 2: The view with the form**

Replace `web/src/experiments/view.ts` with:
```ts
import { aggregate, builtin_sweeps, config_series_names, sweep_points } from '../wasm-pkg/sugarscape.js';
import type { Engine } from '../engine';
import { parseErrors, type Config, type FieldError } from '../types';
import { h } from '../ui/dom';
import { SweepChart } from './chart';
import { chartData } from './chart-data';
import { FixedPanel } from './fixed-panel';
import { defaultForm, numericPaths, type SweepForm } from './form';
import { FormView } from './form-view';
import { poolSize, WorkerPool, type WorkerLike } from './pool';
import { resultsTable } from './results-table';
import type { BuiltinSweep, Point, RunResult, Summary, Sweep, SweepBase } from './types';

/** Each worker loads its own WASM instance (Decision 19). */
const createWorker = (): WorkerLike => new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });

/** Partial results are re-aggregated and redrawn at most this often. */
const REDRAW_MS = 250;

/** The sweep being edited: the form, or a fixed sweep whose seeds and ticks alone change. */
export interface SweepEditor {
  readonly el: HTMLElement;
  /** The sweep, or null (with its errors shown) when the editor cannot build one. */
  sweep(): Sweep | null;
  showErrors(errors: FieldError[]): void;
}

/** Runs on screen and the sweep they belong to (with its JSON, as sent to WASM). */
interface Shown { sweep: Sweep; spec: string; runs: RunResult[] }

const baseNote = (base: SweepBase): string => ('preset' in base ? `preset ${base.preset}` : 'a custom config');

/** The Experiments view: pick or build a sweep, run it on workers, show the results. */
export class ExperimentsView {
  readonly el: HTMLElement;
  private readonly builtins = JSON.parse(builtin_sweeps()) as BuiltinSweep[];
  private readonly picker: HTMLSelectElement;
  private readonly editorSlot = h('div', { class: 'sweep-editor' });
  private editor: SweepEditor | null = null;
  private readonly runButton = h('button', { class: 'primary', onclick: () => void this.run() }, 'Run');
  private readonly cancelButton = h('button', { disabled: true, onclick: () => this.pool?.cancel() }, 'Cancel');
  private readonly bar = h('progress', { max: 1, value: 0 });
  private readonly status = h('span', { class: 'hint', role: 'status' });
  private readonly chart = new SweepChart();
  private readonly table = h('div');
  private pool: WorkerPool | null = null;
  private shown: Shown | null = null;

  constructor(private readonly engine: Engine) {
    this.picker = h(
      'select',
      { 'aria-label': 'Sweep', onchange: () => this.pick(this.picker.value) },
      h('optgroup', { label: 'Built-in' }, ...this.builtins.map((b) => h('option', { value: `builtin:${b.id}` }, b.sweep.name))),
      h('option', { value: 'current' }, 'From current world'),
    );
    this.el = h(
      'div',
      { class: 'experiments-view' },
      h(
        'section',
        { class: 'sweep-setup' },
        h('div', { class: 'row' }, h('label', {}, 'Sweep ', this.picker)),
        this.editorSlot,
        h('div', { class: 'run-bar' }, this.runButton, this.cancelButton, this.bar, this.status),
      ),
      h('section', { class: 'sweep-results' }, this.chart.el, this.table),
    );
    this.pick(this.picker.value);
  }

  private pick(choice: string): void {
    if (choice === 'current') {
      const base = this.currentBase();
      const painted = 'config' in base && this.engine.editedLandscapes() !== undefined;
      const note = `${baseNote(base)} from the current world${painted ? ' (painted maps are not included)' : ''}, captured when chosen`;
      this.setEditor(this.formView(defaultForm(), base, note));
      return;
    }
    const builtin = this.builtins.find((b) => choice === `builtin:${b.id}`);
    if (builtin) this.setEditor(new FixedPanel(builtin.sweep, () => this.validate()));
  }

  /** The current world as a base: its preset when unmodified, otherwise its config (Decision 17). */
  private currentBase(): SweepBase {
    const e = this.engine;
    return e.presetId !== null && !e.isModified() ? { preset: e.presetId } : { config: structuredClone(e.baseConfig) };
  }

  private configOf(base: SweepBase): Config | null {
    return 'preset' in base ? (this.engine.presets.find((p) => p.id === base.preset)?.config ?? null) : base.config;
  }

  private formView(form: SweepForm, base: SweepBase, note: string): FormView {
    const config = this.configOf(base);
    let names: string[] = [];
    try {
      if (config) names = JSON.parse(config_series_names(JSON.stringify(config))) as string[];
    } catch {
      names = [];
    }
    return new FormView(form, base, note, config ? numericPaths(config) : [], names, () => this.validate());
  }

  private setEditor(editor: SweepEditor): void {
    this.editor = editor;
    this.editorSlot.replaceChildren(editor.el);
    this.validate();
  }

  /** Checks the editor's sweep with the core, shows its errors or size, and returns it when valid. */
  private validate(): { sweep: Sweep; spec: string; count: number } | null {
    const editor = this.editor;
    const sweep = editor?.sweep() ?? null;
    if (!editor || !sweep) {
      this.status.textContent = '';
      return null;
    }
    const spec = JSON.stringify(sweep);
    try {
      const count = (JSON.parse(sweep_points(spec)) as Point[]).length;
      editor.showErrors([]);
      this.status.textContent = `${count} runs`;
      return { sweep, spec, count };
    } catch (e) {
      editor.showErrors(parseErrors(e));
      this.status.textContent = '';
      return null;
    }
  }

  private async run(): Promise<void> {
    if (this.pool) return;
    const checked = this.validate();
    if (!checked) return;
    const { sweep, spec, count } = checked;
    const shown: Shown = { sweep, spec, runs: [] };
    this.shown = shown;
    this.showResults();
    const pool = new WorkerPool(poolSize(navigator.hardwareConcurrency), createWorker);
    this.pool = pool;
    this.setRunning(true);
    this.bar.max = count;
    this.bar.value = 0;
    let dirty = false;
    const timer = setInterval(() => {
      if (dirty) {
        dirty = false;
        this.showResults();
      }
    }, REDRAW_MS);
    try {
      const outcome = await pool.run(spec, count, (run) => {
        shown.runs.push(run);
        this.bar.value = shown.runs.length;
        this.status.textContent = `${shown.runs.length} / ${count} runs`;
        dirty = true;
      });
      this.status.textContent =
        outcome === 'done' ? `${count} runs done` : `Cancelled after ${shown.runs.length} of ${count} runs: the results are incomplete`;
    } catch (e) {
      this.status.textContent = `The sweep failed: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      clearInterval(timer);
      this.pool = null;
      this.setRunning(false);
      this.showResults();
    }
  }

  private setRunning(on: boolean): void {
    this.runButton.disabled = on;
    this.cancelButton.disabled = !on;
    this.picker.disabled = on;
  }

  /** Re-aggregates the runs on screen with the core and redraws them. */
  private showResults(): void {
    const shown = this.shown;
    if (!shown || shown.runs.length === 0) {
      this.chart.clear();
      this.table.replaceChildren();
      return;
    }
    const summary = JSON.parse(aggregate(shown.spec, JSON.stringify(shown.runs))) as Summary;
    this.chart.draw(chartData(shown.sweep, summary));
    this.table.replaceChildren(resultsTable(summary));
  }
}
```

In `web/src/main.ts`, replace `const experiments = new ExperimentsView();` with:
```ts
  const experiments = new ExperimentsView(engine);
```

- [ ] **Step 3: Verify** — `(cd web && npm run build && npm test)`: `tsc` passes (in particular no unused members) and every suite passes.

- [ ] **Step 4: Commit**

```bash
git add web/src/experiments/form-view.ts web/src/experiments/view.ts web/src/main.ts
git commit -m "Build sweeps from the current world in an Experiments form" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 14: Outputs, `#x=` share links and opening files

*Mechanical (full code).*

**Files:**
- Create: `web/src/experiments/file.ts`, `web/src/experiments/file.test.ts`
- Modify: `web/src/share.ts`, `web/src/share.test.ts`, `web/src/experiments/view.ts` (full replacement), `web/src/main.ts`

**Interfaces:**
- Consumes: WASM `sweep_result`, `sweep_csv`, `aggregate`; `downloadText`, `downloadBlob`, `canvasBlob`; `sweepToForm` (Task 12); the private `deflate`/`inflateCapped` in `share.ts`.
- Produces:
  - `file.ts`: `type Opened = { kind: 'sweep'; sweep: Sweep } | { kind: 'result'; result: SweepResult } | { kind: 'error'; message: string }`, `classifyFile(json: unknown): Opened`, `slug(name: string): string`
  - `share.ts`: `encodeSweep(sweep: Sweep): Promise<string>`, `decodeSweep(token: string): Promise<Sweep>`, `readSweepHash(hash?: string): string | null` (Decision 22)
  - `ExperimentsView.openSweep(sweep: Sweep): void`; buttons "Open file…", "Share link", "Result (JSON)", "Runs (CSV)", "Summary (CSV)", "Chart (PNG)".

- [ ] **Step 1: Write the failing tests**

Create `web/src/experiments/file.test.ts`:
```ts
import { describe, expect, it } from 'vitest';
import { classifyFile, slug } from './file';
import type { Sweep } from './types';

const sweep: Sweep = {
  name: 'Vision',
  base: { preset: 'ii-2-unit' },
  x: { path: 'vision.max', values: [1, 2] },
  seeds: { from: 1, count: 2 },
  ticks: 100,
  metric: { kind: 'final', series: 'population' },
};

describe('opening files', () => {
  it('recognizes sweeps and results', () => {
    expect(classifyFile(sweep)).toEqual({ kind: 'sweep', sweep });
    const result = { version: 1, sweep, runs: [], summary: { kind: 'scalar', rows: [] } };
    expect(classifyFile(result)).toEqual({ kind: 'result', result });
  });

  it('explains what it cannot open', () => {
    expect(classifyFile([])).toEqual({ kind: 'error', message: 'expected a sweep or a sweep result (a JSON object)' });
    expect(classifyFile({ version: 2, sweep, runs: [] })).toEqual({ kind: 'error', message: 'unsupported result version 2' });
    expect(classifyFile({ version: 1, runs: [] })).toEqual({ kind: 'error', message: 'a result needs "sweep" and "runs"' });
    expect(classifyFile({ name: 'x' })).toEqual({
      kind: 'error',
      message: 'expected a sweep (with base, x and metric) or a sweep result',
    });
  });

  it('makes file names', () => {
    expect(slug('Figure II-5: carrying capacity')).toBe('figure-ii-5-carrying-capacity');
    expect(slug('!!!')).toBe('sweep');
  });
});
```

In `web/src/share.test.ts`, change the share import to:
```ts
import {
  base64UrlToBytes,
  bytesToBase64Url,
  decodeShare,
  decodeSweep,
  encodeShare,
  encodeSweep,
  readHash,
  readSweepHash,
} from './share';
import type { Sweep } from './experiments/types';
```
and append:
```ts
describe('experiment links', () => {
  const sweep: Sweep = {
    name: 'Vision',
    base: { preset: 'ii-2-unit' },
    x: { path: 'vision.max', values: [1, 2, 3] },
    seeds: { from: 1, count: 2 },
    ticks: 100,
    metric: { kind: 'final', series: 'population' },
  };

  it('round-trips a sweep', async () => {
    const token = await encodeSweep(sweep);
    expect(token).toMatch(/^[A-Za-z0-9_-]+$/);
    expect(await decodeSweep(token)).toEqual(sweep);
  });

  it('reads the #x= hash, and #s= stays the playground', () => {
    expect(readSweepHash('#x=ab_-9')).toBe('ab_-9');
    expect(readSweepHash('#s=abc')).toBeNull();
    expect(readHash('#x=abc')).toBeNull();
  });

  it('rejects playground links and garbage', async () => {
    const playground = await encodeShare({ config, seed: 1 });
    await expect(decodeSweep(playground)).rejects.toThrow('not a SugarScape experiment link');
    await expect(decodeSweep('garbage')).rejects.toThrow('not a SugarScape experiment link');
  });
});
```
(`config` is the constant already at the top of `share.test.ts`.)

Run: `(cd web && npx vitest run src/experiments/file.test.ts src/share.test.ts)` — Expected: fails (missing modules/exports).

- [ ] **Step 2: Implement `file.ts` and the link codec**

Create `web/src/experiments/file.ts`:
```ts
import type { Sweep, SweepResult } from './types';

export type Opened = { kind: 'sweep'; sweep: Sweep } | { kind: 'result'; result: SweepResult } | { kind: 'error'; message: string };

const isObject = (v: unknown): v is Record<string, unknown> => typeof v === 'object' && v !== null && !Array.isArray(v);

/** What an opened JSON file (or link payload) is. The core validates the contents when it is used. */
export function classifyFile(json: unknown): Opened {
  if (!isObject(json)) return { kind: 'error', message: 'expected a sweep or a sweep result (a JSON object)' };
  if ('version' in json || 'runs' in json || 'summary' in json) {
    if (json.version !== 1) return { kind: 'error', message: `unsupported result version ${String(json.version)}` };
    if (!isObject(json.sweep) || !Array.isArray(json.runs)) return { kind: 'error', message: 'a result needs "sweep" and "runs"' };
    return { kind: 'result', result: json as unknown as SweepResult };
  }
  if ('base' in json && 'x' in json && 'metric' in json) return { kind: 'sweep', sweep: json as unknown as Sweep };
  return { kind: 'error', message: 'expected a sweep (with base, x and metric) or a sweep result' };
}

/** A file-name stem for a sweep. */
export function slug(name: string): string {
  return (
    name
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, '-')
      .replace(/^-+|-+$/g, '') || 'sweep'
  );
}
```

In `web/src/share.ts`, add after `import type { Config } from './types';`:
```ts
import { classifyFile } from './experiments/file';
import type { Sweep } from './experiments/types';
```
and append at the end of the file:
```ts
/** `#x=` links (Decision 22): base64url(deflate-raw(sweep JSON)). */
export async function encodeSweep(sweep: Sweep): Promise<string> {
  return bytesToBase64Url(await deflate(new TextEncoder().encode(JSON.stringify(sweep))));
}

export async function decodeSweep(token: string): Promise<Sweep> {
  try {
    const json = JSON.parse(new TextDecoder().decode(await inflateCapped(base64UrlToBytes(token)))) as unknown;
    const opened = classifyFile(json);
    if (opened.kind !== 'sweep') throw new Error(opened.kind);
    return opened.sweep;
  } catch {
    throw new Error('not a SugarScape experiment link');
  }
}

export function readSweepHash(hash: string = location.hash): string | null {
  return /^#x=([A-Za-z0-9_-]+)$/.exec(hash)?.[1] ?? null;
}
```

Run: `(cd web && npx vitest run src/experiments/file.test.ts src/share.test.ts)` — Expected: all pass.

- [ ] **Step 3: The view with outputs, links and files**

Replace `web/src/experiments/view.ts` with:
```ts
import {
  aggregate,
  builtin_sweeps,
  config_series_names,
  sweep_csv,
  sweep_points,
  sweep_result,
} from '../wasm-pkg/sugarscape.js';
import { canvasBlob, downloadBlob, downloadText } from '../downloads';
import type { Engine } from '../engine';
import { encodeSweep } from '../share';
import { parseErrors, type Config, type FieldError } from '../types';
import { h } from '../ui/dom';
import { SweepChart } from './chart';
import { chartData } from './chart-data';
import { classifyFile, slug } from './file';
import { FixedPanel } from './fixed-panel';
import { defaultForm, numericPaths, sweepToForm, type SweepForm } from './form';
import { FormView } from './form-view';
import { poolSize, WorkerPool, type WorkerLike } from './pool';
import { resultsTable } from './results-table';
import type { BuiltinSweep, Point, RunResult, Summary, Sweep, SweepBase, SweepResult } from './types';

/** Each worker loads its own WASM instance (Decision 19). */
const createWorker = (): WorkerLike => new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });

/** Partial results are re-aggregated and redrawn at most this often. */
const REDRAW_MS = 250;

/** The sweep being edited: the form, or a fixed sweep whose seeds and ticks alone change. */
export interface SweepEditor {
  readonly el: HTMLElement;
  /** The sweep, or null (with its errors shown) when the editor cannot build one. */
  sweep(): Sweep | null;
  showErrors(errors: FieldError[]): void;
}

/** Runs on screen and the sweep they belong to (with its JSON, as sent to WASM). */
interface Shown { sweep: Sweep; spec: string; runs: RunResult[] }

const baseNote = (base: SweepBase): string => ('preset' in base ? `preset ${base.preset}` : 'a custom config');

const messages = (errors: FieldError[]): string => errors.map((e) => `${e.field}: ${e.message}`).join('; ');

/** The Experiments view: pick, build or open a sweep, run it on workers, show and export the results. */
export class ExperimentsView {
  readonly el: HTMLElement;
  private readonly builtins = JSON.parse(builtin_sweeps()) as BuiltinSweep[];
  private readonly picker: HTMLSelectElement;
  private readonly fileInput: HTMLInputElement;
  private readonly editorSlot = h('div', { class: 'sweep-editor' });
  private editor: SweepEditor | null = null;
  private readonly runButton = h('button', { class: 'primary', onclick: () => void this.run() }, 'Run');
  private readonly cancelButton = h('button', { disabled: true, onclick: () => this.pool?.cancel() }, 'Cancel');
  private readonly bar = h('progress', { max: 1, value: 0 });
  private readonly status = h('span', { class: 'hint', role: 'status' });
  private readonly chart = new SweepChart();
  private readonly table = h('div');
  private readonly outputs: HTMLButtonElement[];
  private pool: WorkerPool | null = null;
  private shown: Shown | null = null;

  constructor(private readonly engine: Engine) {
    this.picker = h(
      'select',
      { 'aria-label': 'Sweep', onchange: () => this.pick(this.picker.value) },
      h('optgroup', { label: 'Built-in' }, ...this.builtins.map((b) => h('option', { value: `builtin:${b.id}` }, b.sweep.name))),
      h('option', { value: 'current' }, 'From current world'),
    );
    this.fileInput = h('input', {
      type: 'file',
      accept: '.json,application/json',
      hidden: true,
      onchange: () => void this.openFile(),
    });
    this.outputs = [
      h('button', { onclick: () => this.download('result') }, 'Result (JSON)'),
      h('button', { onclick: () => this.download('runs') }, 'Runs (CSV)'),
      h('button', { onclick: () => this.download('summary') }, 'Summary (CSV)'),
      h('button', { onclick: () => void this.downloadChart() }, 'Chart (PNG)'),
    ];
    this.el = h(
      'div',
      { class: 'experiments-view' },
      h(
        'section',
        { class: 'sweep-setup' },
        h(
          'div',
          { class: 'row' },
          h('label', {}, 'Sweep ', this.picker),
          h('button', { onclick: () => this.fileInput.click(), title: 'Open a sweep or a result file (JSON)' }, 'Open file…'),
          this.fileInput,
        ),
        this.editorSlot,
        h('div', { class: 'run-bar' }, this.runButton, this.cancelButton, this.bar, this.status),
        h(
          'div',
          { class: 'row' },
          h('button', { onclick: () => void this.share(), title: 'Copy a link that opens this sweep (not its results)' }, 'Share link'),
        ),
      ),
      h('section', { class: 'sweep-results' }, this.chart.el, this.table, h('div', { class: 'sweep-outputs' }, ...this.outputs)),
    );
    this.pick(this.picker.value);
    this.showResults();
  }

  /** Opens a sweep from a link or a file for editing, without running it. */
  openSweep(sweep: Sweep): void {
    this.markOpened(sweep.name);
    const form = sweepToForm(sweep);
    this.setEditor(form ? this.formView(form, sweep.base, baseNote(sweep.base)) : new FixedPanel(sweep, () => this.validate()));
  }

  private pick(choice: string): void {
    if (choice === 'current') {
      const base = this.currentBase();
      const painted = 'config' in base && this.engine.editedLandscapes() !== undefined;
      const note = `${baseNote(base)} from the current world${painted ? ' (painted maps are not included)' : ''}, captured when chosen`;
      this.setEditor(this.formView(defaultForm(), base, note));
      return;
    }
    const builtin = this.builtins.find((b) => choice === `builtin:${b.id}`);
    if (builtin) this.setEditor(new FixedPanel(builtin.sweep, () => this.validate()));
  }

  /** Adds (or replaces) the picker entry for an opened link or file, and selects it. */
  private markOpened(name: string): void {
    this.picker.querySelector('option[value="opened"]')?.remove();
    this.picker.prepend(h('option', { value: 'opened' }, `Opened: ${name}`));
    this.picker.value = 'opened';
  }

  /** The current world as a base: its preset when unmodified, otherwise its config (Decision 17). */
  private currentBase(): SweepBase {
    const e = this.engine;
    return e.presetId !== null && !e.isModified() ? { preset: e.presetId } : { config: structuredClone(e.baseConfig) };
  }

  private configOf(base: SweepBase): Config | null {
    return 'preset' in base ? (this.engine.presets.find((p) => p.id === base.preset)?.config ?? null) : base.config;
  }

  private formView(form: SweepForm, base: SweepBase, note: string): FormView {
    const config = this.configOf(base);
    let names: string[] = [];
    try {
      if (config) names = JSON.parse(config_series_names(JSON.stringify(config))) as string[];
    } catch {
      names = [];
    }
    return new FormView(form, base, note, config ? numericPaths(config) : [], names, () => this.validate());
  }

  private setEditor(editor: SweepEditor): void {
    this.editor = editor;
    this.editorSlot.replaceChildren(editor.el);
    this.validate();
  }

  /** Checks the editor's sweep with the core, shows its errors or size, and returns it when valid. */
  private validate(): { sweep: Sweep; spec: string; count: number } | null {
    const editor = this.editor;
    const sweep = editor?.sweep() ?? null;
    if (!editor || !sweep) {
      this.status.textContent = '';
      return null;
    }
    const spec = JSON.stringify(sweep);
    try {
      const count = (JSON.parse(sweep_points(spec)) as Point[]).length;
      editor.showErrors([]);
      this.status.textContent = `${count} runs`;
      return { sweep, spec, count };
    } catch (e) {
      editor.showErrors(parseErrors(e));
      this.status.textContent = '';
      return null;
    }
  }

  private async run(): Promise<void> {
    if (this.pool) return;
    const checked = this.validate();
    if (!checked) return;
    const { sweep, spec, count } = checked;
    const shown: Shown = { sweep, spec, runs: [] };
    this.shown = shown;
    this.showResults();
    const pool = new WorkerPool(poolSize(navigator.hardwareConcurrency), createWorker);
    this.pool = pool;
    this.setRunning(true);
    this.bar.max = count;
    this.bar.value = 0;
    let dirty = false;
    const timer = setInterval(() => {
      if (dirty) {
        dirty = false;
        this.showResults();
      }
    }, REDRAW_MS);
    try {
      const outcome = await pool.run(spec, count, (run) => {
        shown.runs.push(run);
        this.bar.value = shown.runs.length;
        this.status.textContent = `${shown.runs.length} / ${count} runs`;
        dirty = true;
      });
      this.status.textContent =
        outcome === 'done' ? `${count} runs done` : `Cancelled after ${shown.runs.length} of ${count} runs: the results are incomplete`;
    } catch (e) {
      this.status.textContent = `The sweep failed: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      clearInterval(timer);
      this.pool = null;
      this.setRunning(false);
      this.showResults();
    }
  }

  private setRunning(on: boolean): void {
    this.runButton.disabled = on;
    this.cancelButton.disabled = !on;
    this.picker.disabled = on;
  }

  /** Re-aggregates the runs on screen with the core and redraws them; exports need at least one run. */
  private showResults(): void {
    const shown = this.shown;
    const empty = !shown || shown.runs.length === 0;
    for (const b of this.outputs) b.disabled = empty;
    if (!shown || empty) {
      this.chart.clear();
      this.table.replaceChildren();
      return;
    }
    const summary = JSON.parse(aggregate(shown.spec, JSON.stringify(shown.runs))) as Summary;
    this.chart.draw(chartData(shown.sweep, summary));
    this.table.replaceChildren(resultsTable(summary));
  }

  private async openFile(): Promise<void> {
    const file = this.fileInput.files?.[0];
    this.fileInput.value = '';
    if (!file || this.pool) return;
    let json: unknown;
    try {
      json = JSON.parse(await file.text());
    } catch {
      this.status.textContent = `${file.name} is not JSON`;
      return;
    }
    const opened = classifyFile(json);
    if (opened.kind === 'error') this.status.textContent = `${file.name}: ${opened.message}`;
    else if (opened.kind === 'sweep') this.openSweep(opened.sweep);
    else this.openResult(opened.result, file.name);
  }

  /** Shows a result file's runs without running them; the summary is recomputed from the runs (Decision 20). */
  private openResult(result: SweepResult, file: string): void {
    this.markOpened(result.sweep.name);
    this.setEditor(new FixedPanel(result.sweep, () => this.validate()));
    this.shown = { sweep: result.sweep, spec: JSON.stringify(result.sweep), runs: result.runs };
    try {
      this.showResults();
      this.status.textContent = `${file}: ${result.runs.length} runs${result.incomplete ? ' (incomplete)' : ''}`;
    } catch (e) {
      this.shown = null;
      this.showResults();
      this.status.textContent = `${file}: ${messages(parseErrors(e))}`;
    }
  }

  private download(kind: 'result' | 'runs' | 'summary'): void {
    const shown = this.shown;
    if (!shown) return;
    const runs = JSON.stringify(shown.runs);
    const name = slug(shown.sweep.name);
    try {
      if (kind === 'result') downloadText(`${name}-result.json`, sweep_result(shown.spec, runs), 'application/json');
      else downloadText(`${name}-${kind}.csv`, sweep_csv(shown.spec, runs, kind));
    } catch (e) {
      this.status.textContent = messages(parseErrors(e));
    }
  }

  private async downloadChart(): Promise<void> {
    const canvas = this.chart.canvas();
    if (this.shown && canvas) downloadBlob(`${slug(this.shown.sweep.name)}-chart.png`, await canvasBlob(canvas));
  }

  /** Puts a `#x=` link to the edited sweep (not its results) in the address bar and the clipboard. */
  private async share(): Promise<void> {
    const sweep = this.editor?.sweep() ?? null;
    if (!sweep) {
      this.status.textContent = 'Fix the sweep before sharing it';
      return;
    }
    history.replaceState(null, '', `#x=${await encodeSweep(sweep)}`);
    try {
      await navigator.clipboard.writeText(location.href);
      this.status.textContent = 'Link copied';
    } catch {
      this.status.textContent = 'Link in the address bar';
    }
  }
}
```

- [ ] **Step 4: Open `#x=` links at startup**

In `web/src/main.ts`, change the share import to:
```ts
import { decodeShare, decodeSweep, encodeShare, readHash, readSweepHash } from './share';
```
and after `showView('playground');` add:
```ts
  const sweepToken = readSweepHash();
  if (sweepToken) {
    try {
      experiments.openSweep(await decodeSweep(sweepToken));
      showView('experiments');
    } catch (e) {
      showBanner(`That experiment link could not be loaded (${e instanceof Error ? e.message : String(e)}).`);
    }
  }
```
(An `#x=` link is not an `#s=` link, so the playground starts from its default rule system.)

- [ ] **Step 5: Verify** — `(cd web && npm run build && npm test)`: build passes; 3 file tests and 3 new share tests pass with every earlier suite.

- [ ] **Step 6: Commit**

```bash
git add web/src/experiments/file.ts web/src/experiments/file.test.ts web/src/share.ts web/src/share.test.ts web/src/experiments/view.ts web/src/main.ts
git commit -m "Export sweep results, share sweeps as #x= links and open result files" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 15: Book-style sweep tests

*Needs judgement: the orderings are the book's claims; the recorded numbers come from measurement.*

**Files:**
- Modify: `crates/sugarscape-core/tests/book.rs`

**Interfaces:**
- Consumes: `sweep::{builtin, run_all, Summary, SweepResult}`, the Task 5 files at their recorded settings.
- Produces: `fig_ii_5_carrying_capacity_rises_with_vision_and_falls_with_metabolism`, `fig_iv_6_trade_raises_carrying_capacity_at_every_vision`, `fig_iv_10_11_long_lives_end_with_less_price_dispersion` (all `#[ignore]`, run in release).

- [ ] **Step 1: Write the tests**

Add `use sugarscape_core::sweep::{self, Summary, SweepResult};` to the imports of `crates/sugarscape-core/tests/book.rs`, and append:
```rust
/// A built-in sweep at its recorded settings, on every core.
fn run_builtin(id: &str) -> SweepResult {
    let s = sweep::builtin(id).unwrap();
    let jobs = std::thread::available_parallelism().map_or(1, |n| n.get());
    sweep::run_all(&s, jobs, |_, _| {}).unwrap()
}

/// A scalar sweep's means as `[series][x]`.
fn cell_means(result: &SweepResult) -> Vec<Vec<f64>> {
    let Summary::Scalar(rows) = &result.summary else {
        panic!("a scalar metric was expected");
    };
    let mut means = vec![vec![f64::NAN; result.sweep.x.values.len()]; result.sweep.series_count()];
    for r in rows {
        means[r.series][r.x] = r.mean;
    }
    means
}

#[test]
#[ignore]
fn fig_ii_5_carrying_capacity_rises_with_vision_and_falls_with_metabolism() {
    // Figure II-5 at `sweeps/fig-ii-5.json`'s settings. Measured means
    // (rows: mean metabolism 1, 2, 3; columns: mean vision 1–6): recorded in Step 2.
    let means = cell_means(&run_builtin("fig-ii-5"));
    for (s, line) in means.iter().enumerate() {
        assert!(line[line.len() - 1] > line[0], "metabolism line {s}: {line:?}");
    }
    for x in 0..means[0].len() {
        let column: Vec<f64> = means.iter().map(|line| line[x]).collect();
        assert!(column.windows(2).all(|w| w[1] < w[0]), "vision column {x}: {column:?}");
    }
}

#[test]
#[ignore]
fn fig_iv_6_trade_raises_carrying_capacity_at_every_vision() {
    // Figure IV-6 at `sweeps/fig-iv-6.json`'s settings (series 0 = no trade,
    // 1 = trade). Measured means: recorded in Step 2.
    let means = cell_means(&run_builtin("fig-iv-6"));
    for x in 0..means[0].len() {
        assert!(means[1][x] > means[0][x], "vision {x}: trade {} vs no trade {}", means[1][x], means[0][x]);
    }
}

#[test]
#[ignore]
fn fig_iv_10_11_long_lives_end_with_less_price_dispersion() {
    // Figures IV-10/IV-11 at `sweeps/fig-iv-10-11.json`'s settings (series 0 =
    // lifetimes 60–100, 1 = 960–1000). Measured last-block means: recorded in Step 2.
    let result = run_builtin("fig-iv-10-11");
    let Summary::Timeseries(rows) = &result.summary else {
        panic!("a timeseries metric was expected");
    };
    let last = |s: usize| rows.iter().rfind(|r| r.series == s).unwrap().mean;
    let (short, long) = (last(0), last(1));
    assert!(long < short, "lifetimes 60–100: {short}, 960–1000: {long}");
}
```

- [ ] **Step 2: Run and record**

Temporarily add `println!("{means:?}");` (and `println!("{short} {long}");`) and run:
`cargo test -p sugarscape-core --release --test book -- --ignored --nocapture fig_`
Replace each "recorded in Step 2" with the printed means, rounded to one decimal (three significant digits for sd(ln price)), then remove the `println!`s. If an assertion fails at the recorded settings, stop and report **BLOCKED** with the numbers: do not loosen an assertion, change seeds, or edit a sweep file here (Task 5 already checked these orderings; a failure means something changed since).

- [ ] **Step 3: Verify**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
cargo test -p sugarscape-core --release --test book -- --ignored
```
Expected: every book test, old and new, passes.

- [ ] **Step 4: Commit**

```bash
git add crates/sugarscape-core/tests/book.rs
git commit -m "Reproduce Figures II-5, IV-6 and IV-10/11 from the built-in sweeps" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

### Task 16: README, roadmap, CI and full verification

*Mechanical.*

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `.github/workflows/ci.yml`

- [ ] **Step 1: README**

Insert before `## Running locally`:
```markdown
## Experiments

The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
world is paused and kept). A sweep runs every combination of config values × seeds for a
number of ticks and summarizes each run by one statistic: its value at the last tick, its
mean over a window of ticks, or its means over blocks of ticks. The chart shows one line per
value of the second axis: the mean over seeds, with a ±1 sd band; hovering shows mean, sd,
range and the number of runs.

- **Built-in sweeps** (`sweeps/`): `fig-ii-5` (carrying capacity vs vision, one line per
  metabolism), `fig-iv-6` (with and without trade), `fig-iv-10-11` (price dispersion over
  time for short and long lifetimes) and `n-goods-carrying-capacity`. Each file's description
  records its measured settings; in the browser only seeds and ticks can be changed.
- **From current world**: a config path (the input suggests every number and on/off setting),
  values as `1, 2, 3`, `true, false` or `from:to:step`, an optional second axis, seeds, ticks
  and the metric.
- **Open file…**: a sweep, or a result from the CLI or an earlier export, which is shown
  without running.

Runs are spread over Web Workers (one per core, less one). Cancel stops them and keeps the
partial results, which export marked incomplete. Exports: the result JSON (the CLI's
format), runs and summary CSVs and the chart as PNG. **Share link** copies a `#x=` link that
opens the sweep (not its results).

## Command line

`crates/sugarscape-cli` builds a native `sugarscape` binary over the same core
(`cargo install --path crates/sugarscape-cli`, or `cargo run --release -p sugarscape-cli -- …`):

    sugarscape presets                                  # preset ids
    sugarscape sweeps                                   # built-in sweeps
    sugarscape run --preset ii-5-wealth --seed 7 --ticks 1000 --series-csv series.csv
    sugarscape run --config my-config.json --agents-csv agents.csv --fingerprint
    sugarscape sweep --builtin fig-ii-5 --out fig-ii-5.json --summary-csv fig-ii-5.csv
    sugarscape sweep my-sweep.json --jobs 4 --seeds 3 --ticks 300 --runs-csv runs.csv

`run` defaults to seed 1 and 1000 ticks; `--config-out` writes the config it ran and
`--fingerprint` prints the final world's fingerprint. `sweep` uses every core unless `--jobs`
says otherwise, prints the result JSON unless `--out` is given, and reports progress on
stderr unless `--quiet`. Exit codes: 0 success, 1 I/O error, 2 usage or validation error
(printed as `field: message`, one per line).

A run is a function of its config and seed, so a sweep's output files are byte-identical for
any `--jobs` (and in the browser, for any number of workers). Native and browser builds can
differ in the last bits of `powf`/`ln`, so runs with trade or several goods can give slightly
different numbers in the CLI and in the Experiments view.
```

In `## Tests`, add as the first line of the code block:
```
    cargo test                                                      # whole workspace, incl. the CLI
```

- [ ] **Step 2: Roadmap**

In `docs/roadmap.md`, insert after the Milestone 4 section:
```markdown
## Milestone 5: Experiments (done)

Parameter sweeps (config values × seeds, each run summarized by one statistic) in the core,
a native `sugarscape` CLI (`presets`, `sweeps`, `run`, `sweep`) and a browser Experiments
view on a Web Worker pool, with measured built-in sweeps for Figures II-5, IV-6 and
IV-10/11 and for carrying capacity vs the number of goods. See
`docs/superpowers/specs/2026-09-23-experiments-design.md`.
```
and under "Experiments and science" replace the "Parameter sweeps / batch runs" and "Headless CLI" bullets with:
```markdown
- **Parameter sweeps / batch runs**: done (Milestone 5).
- **Headless CLI**: done (Milestone 5).
```

- [ ] **Step 3: CI**

In `.github/workflows/ci.yml` (job `rust`), replace `- run: cargo test -p sugarscape-core` with:
```yaml
      - run: cargo test --workspace
      - run: cargo run --release -p sugarscape-cli -- sweep --builtin fig-iv-10-11 --seeds 1 --ticks 100 --quiet > /dev/null
```
(The Pages workflow needs no change: Vite bundles the worker.)

- [ ] **Step 4: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test
cargo test -p sugarscape-core --release --test book -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
git diff main -- crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/legacy.rs
```
All must pass, and the last command must print nothing. The controller then runs the puppeteer pass (implementers don't): switch to Experiments and back (the playground's world and tick are unchanged); run `fig-iv-10-11` with 1 seed and 100 ticks to completion (progress bar, chart with two lines and bands, hover legend, table); start `fig-ii-5` and Cancel mid-run (partial chart, "incomplete" status, Result JSON contains `"incomplete": true`); build a sweep "From current world" (`vision.max`, `1:6:1`, 2 seeds, 100 ticks) and check an error shows inline for a bad path; Share link → reload → the sweep opens in Experiments, not run; open a result file written by `sugarscape sweep` (it charts without running); time `fig-ii-5` at its default seeds and report it (Task 5 assumed ≤ ~40 s on three workers).

- [ ] **Step 5: Commit**

```bash
git add README.md docs/roadmap.md .github/workflows/ci.yml
git commit -m "Document the CLI and the Experiments view" -m "Claude-Session: https://claude.ai/code/session_01Kq7NyxbMrkNsPfAhnsVcK3"
```

---

## Spec coverage

| Spec requirement | Task |
|---|---|
| No simulation change; golden and legacy unedited | Global Constraints; every task's verification; 16 (diff check) |
| One implementation in `sugarscape-core::sweep`; CLI and browser call it | 1–5, 7, 8, 13–14 (TS only maps) |
| Determinism independent of threads/workers/completion order; native vs WASM documented | 3 (`aggregate` sorts), 4 (jobs 1 vs 4), 7 (CLI files), 8 (WASM = core), 16 (README) |
| Sweep format: base preset/config (either shape), `set`, x, optional series, seeds, ticks, metric | 1 |
| Shorthand axes, `at` defaults, labels, line names | 1 |
| Point order and indices | 1 |
| Config building order, `validate`, error fields and point prefix | 1 (Decision 3) |
| Limits | 1 (Decision 2) |
| Metrics `final`, `window_mean`, `timeseries`; NaN skipping; zero population keeps running | 2 |
| Aggregation (sample sd, n = 1, NaN counted) | 3 |
| Result file, runs/summary CSV | 3 |
| `run_all` on std threads, ordered by point | 4 |
| Ready-made sweeps (II-5, IV-6, IV-10/11, N goods), measured, recorded in descriptions | 5 |
| CLI `presets`, `sweeps`, `run` (defaults, CSVs, config-out, fingerprint) | 6 |
| CLI `sweep` (jobs, seeds/ticks overrides, out/stdout, CSVs, progress, quiet); exit codes; byte-identical files | 7 |
| WASM `sweep_points`, `run_point`, `aggregate`, `builtin_sweeps`; `run_point` + `aggregate` = core | 8 |
| Browser switch; world kept | 10 |
| Worker pool (`max(1, hc − 1)`, own WASM each, one point at a time, in-order dispatch), partial aggregation, progress, Cancel | 9, 10 |
| Picker: built-ins, From current world, Open file (sweep or result, shown without running) | 10, 13, 14 |
| Form: name, base, two axes with path suggestions and value lists, seeds, ticks, metric; built-ins read-only except seeds/ticks; inline errors | 12, 13 |
| Chart: scalar (x = at, mean + sd band per line, legend, hover stats), timeseries (x = tick) | 11 |
| Outputs: result JSON, runs CSV, summary CSV, chart PNG; `#x=` link opens without running | 14 |
| Tests: core, CLI, WASM, web (form ↔ sweep, value lists, `#x=`, chart mapping), book-style, browser | 1–4, 5, 6–7, 8, 9–14, 15, 16 (controller) |
| Docs: README CLI + Experiments; roadmap items done | 16 |

## Spec gaps and conflicts found

1. **`n-goods-carrying-capacity` cannot sweep 1–6 goods with a trade line**: trade requires two goods, so x = 1 with trade on fails validation. The sweep uses 2–6 goods (Decision 13).
2. **A `timeseries` metric with several x values** has no place in the spec's timeseries summary CSV (`series,series_name,t,…`) or chart (x = tick). This plan requires exactly one x value for time series (Decision 4).
3. **`run_all(&Sweep, jobs) -> SweepResult`** cannot report an invalid sweep or drive the CLI's progress lines; it gains a progress callback and returns `Result` (Decision 8). `run_point` keeps the spec's signature and panics on an invalid point (Decision 9).
4. **The four WASM functions** do not cover the form's statistic list or the browser's CSV/result exports without re-implementing the core's writers in TypeScript (against "one implementation"); three more functions are added (Decision 16).
5. **The validation-error format** `point 12 (x=3, series=1): …` does not say whether x/series are indices or `at` values, which field carries it, or which seed's point is named; resolved in Decision 3 (indices, field `points`, first point of the cell).
6. **JSON cannot hold NaN**; NaN run values and statistics are written as `null` (Decision 5). The spec's runs CSV for time series (`one row per block with t`) does not give the column order; it is `series,x,seed,t,value` (Decision 6).
7. **Browser-cancelled results** need a way to be "marked incomplete" in the exported file; `SweepResult` gains an `incomplete` flag written only when true (Decision 10).
8. "Output files are byte-identical for any `--jobs`" holds for `--out` and the CSVs; the stderr progress lines are in completion order and are not.
