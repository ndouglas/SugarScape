# SugarScape Milestone 5 — Experiments — Design

**Date:** 2026-09-23
**Builds on:** the milestone 1–4 specs in `docs/superpowers/specs/`; all remain binding where not changed here.
**Source text:** Epstein & Axtell, *Growing Artificial Societies*: Figure II-5 (carrying capacity vs. initial mean vision, parameterized by mean metabolism; 500 agents; each point the mean of 10 runs), Figure IV-6 (carrying capacity vs. mean vision, with and without trade), Figures IV-10 and IV-11 (standard deviation of the logarithm of average trade price over time, lifetimes 60–100 vs. 960–1000).

## Goal

Turn the playground into an instrument: parameter sweeps (a grid of config values × seeds → summary metrics) that run both in a native `sugarscape` CLI and in the browser, with the book's parametric figures shipped as ready-made sweeps.

## Non-negotiable constraints

- **No simulation change.** Every golden entry and legacy fixture stays green and unedited; sweeps only build configs and read statistics.
- **One implementation.** Sweep expansion, config building, metrics and aggregation live in `sugarscape-core`; the CLI and the browser call the same code.
- **Determinism.** A run is a function of (config, seed). On one platform a sweep's results are identical regardless of thread/worker count or completion order. Native and WASM builds may differ in the last bits of transcendental functions (`powf`, `ln`); this is documented, not hidden.

## Architecture

- `sugarscape-core::sweep` (new module): `Sweep` (serde), `Sweep::points(&self) -> Result<Vec<Point>, Vec<FieldError>>`, `Sweep::config_for(&self, point) -> Result<Config, Vec<FieldError>>`, `run_point(&Sweep, &Point) -> RunResult`, `aggregate(&Sweep, &[RunResult]) -> Summary`, `SweepResult { sweep, runs, summary }` (serde), CSV writers for runs and summary, and `run_all(&Sweep, jobs) -> SweepResult` (std threads; results ordered by point index). Built-in sweeps are JSON files in `sweeps/` embedded with `include_str!` and listed by `sweep::builtins()`.
- `crates/sugarscape-cli` (new): binary `sugarscape` over `sugarscape-core` with `clap` (derive).
- `sugarscape-wasm`: free functions `sweep_points(spec)`, `run_point(spec, i)`, `aggregate(spec, runs)`, `builtin_sweeps()` (JSON strings in/out; errors as JSON `[{field, message}]`). `Sim` is unchanged.
- `web/`: an Experiments view with a Web Worker pool.

## Sweep format

```jsonc
{
  "name": "Figure II-5: carrying capacity vs vision",
  "description": "…",                                   // optional
  "base": { "preset": "ii-2-unit" },                    // or { "config": { … } } (legacy or new shape)
  "set": { "population": 500 },                         // optional; applied to every run
  "x": { "label": "Mean vision", "values": [ { "at": 1, "set": { "vision": { "min": 1, "max": 1 } } } ] },
  "series": { "label": "Mean metabolism", "values": [ … ] },   // optional
  "seeds": { "from": 1, "count": 10 },
  "ticks": 500,
  "metric": { "kind": "window_mean", "series": "population", "from": 400 }
}
```

- **Axis:** `{ label, values: [AxisValue] }` or the shorthand `{ label?, path, values: [JSON] }`, which expands to `{ at: i, set: { path: v } }` for the i-th value (`at` = the value itself when it is a number; label defaults to the path). `AxisValue = { at: f64, name?: string, set: { path: JSON } }`; `name` labels a series line (default `"<label> = <at>"`).
- **Points:** every (series value, x value, seed) — series-major, then x, then seed; a point's index is its position in that order. With no `series`, there is one implicit series.
- **Config for a point:** start from `base` (a preset's config or `Config::from_value(config)`), apply `set`, then the series value's `set`, then the x value's `set`, each via `Config::with_path` in key order, then `validate()`. Errors are reported with field `x[i].set.<path>` / `series[j].set.<path>` / `set.<path>` / `base`, plus validation errors prefixed with the point (`point 12 (x=3, series=1): <field>: <message>`).
- **Limits:** 1–64 x values, 1–16 series values, `seeds.count` 1–100, `ticks` 1–100 000, at most 10 000 points.
- **Metrics** (over one statistics series, which must be in `stats::series_names(config)` for every point's config):
  - `final` — the value at the last tick;
  - `window_mean { from, to? }` — mean over ticks `from..=to` (default: last tick); `from ≤ to ≤ ticks`;
  - `timeseries { every }` — block means: for k = 0.., the mean over ticks `k·every+1 ..= min((k+1)·every, ticks)`.
  A run whose population hits 0 keeps running (its statistics continue at 0 / NaN as today); NaN values are skipped in means, and a block or window with no finite value yields NaN.
- **Aggregation** per (series, x): for scalar metrics `{ n, mean, sd, min, max }` over seeds (sample sd with n−1; 0 when n = 1; NaN runs excluded from all and counted in `nan`); for `timeseries`, per block `{ t, mean, sd }` (t = the block's last tick).
- **Result file:** `SweepResult { version: 1, sweep, runs: [{ point, series, x, seed, value | values }], summary }`. CSV: runs (`series,x,seed,value` or one row per block with `t`), summary (`series,series_name,x,n,mean,sd,min,max` or `series,series_name,t,n,mean,sd`).

## Ready-made sweeps (`sweeps/`)

| id | Base | x | series | Metric |
|---|---|---|---|---|
| `fig-ii-5` | `ii-2-unit`, 500 agents | mean vision 1–6 (vision ranges with that mean) | mean metabolism (metabolism ranges with that mean) | `window_mean` population over the tail |
| `fig-iv-6` | `iv-3-trade` | mean vision 1–6 | trade off / on | `window_mean` population over the tail |
| `fig-iv-10-11` | `iv-3-trade` with replacement | — (x has one value) | lifespan 60–100 / 960–1000 | `timeseries` of `sd_log_price` |
| `n-goods-carrying-capacity` | `n-4-peaks`-style peaks goods | number of goods 1–6 | trade off / on | `window_mean` population over the tail |

Exact ranges, ticks, windows and seeds are chosen during implementation (measured so the curves show the book's qualitative result and a sweep finishes in reasonable time in the browser) and recorded in each file's `description`. Unlike the book's 10 runs per point, a sweep's default seed count may be lower; the file says so.

## CLI

```
sugarscape presets
sugarscape sweeps
sugarscape run  (--preset ID | --config FILE) [--seed N] [--ticks N]
                [--series-csv PATH] [--agents-csv PATH] [--config-out PATH] [--fingerprint]
sugarscape sweep (FILE | --builtin NAME) [--jobs N] [--seeds N] [--ticks N]
                 [--out PATH] [--runs-csv PATH] [--summary-csv PATH] [--quiet]
```

- `run` defaults: seed 1, ticks 1000; outputs use `export::series_csv`/`agents_csv`; `--fingerprint` prints the final fingerprint as `0x%016x`.
- `sweep` defaults: jobs = available parallelism; `--seeds N` overrides `seeds.count`; `--ticks N` overrides `ticks` (a window/timeseries that no longer fits is a validation error). Without `--out`, the result JSON goes to stdout. Progress (`[i/total] series=… x=… seed=…`) goes to stderr unless `--quiet`.
- Exit codes: 0 success; 1 I/O error; 2 usage or validation error (printed one per line as `field: message`).
- Output files are byte-identical for any `--jobs`.

## Browser

- **Switch:** the header gets a Playground | Experiments toggle; Experiments replaces the main area (grid and side panel) while keeping the playground's world intact.
- **Picker:** the built-in sweeps, "From current world", and "Open file…" (a sweep JSON, or a `SweepResult` JSON, which is shown without running).
- **Form:** name; base (current world's normalized config, or the built-in's base, read-only); up to two axes, each a path (text input with suggestions of numeric/boolean config paths) and values (`a, b, c` or `from:to:step`, numbers or `true`/`false`); seeds count; ticks; metric kind, series (select from `series_names` of the base), window/every. Built-in sweeps open read-only except seeds and ticks. Validation errors (from `sweep_points`) show inline.
- **Running:** a pool of module Web Workers, `max(1, hardwareConcurrency − 1)`, each with its own WASM instance, running one point at a time via `run_point`. The main thread dispatches indices in order, collects results, and aggregates with `aggregate` (partial aggregation redrawn as results arrive). Progress bar and Cancel (terminates the workers; partial results stay visible and exportable, marked incomplete).
- **Chart (uPlot):** scalar metrics — x = `at`, one line per series (mean) with a ±1 sd band, legend with series names, hover shows mean/sd/min/max/n; timeseries — x = tick, one line per series with a band.
- **Outputs:** result JSON (CLI format), runs CSV, summary CSV, chart PNG; share link `#x=` = base64url(deflate(sweep JSON)), opening straight into Experiments with the sweep loaded (not run).

## Testing

- **Core:** point expansion order and indices; shorthand axes; config building order and error fields; metrics (final, window bounds, timeseries blocks, NaN handling); aggregation (sd, n = 1, NaN counting); `run_all` output independent of `jobs` (1 vs 4, byte-identical JSON); built-in sweeps parse and validate; golden and legacy unchanged.
- **CLI:** argument parsing; `run` writes the CSVs and fingerprint; `sweep` on a tiny sweep produces the expected files; exit codes for bad input.
- **WASM:** the four functions; `run_point` + `aggregate` equals the core `run_all` on the same sweep (same platform).
- **Web (vitest):** form ↔ sweep conversion; value-list parsing (lists, ranges, booleans, errors); `#x=` round trip; result → chart data mapping.
- **Book-style (`#[ignore]`, release):** `fig-ii-5` — carrying capacity rises with vision and falls with metabolism (at the recorded settings); `fig-iv-6` — the trade line lies above the no-trade line at every x; `fig-iv-10-11` — the long-lifespan series ends with lower mean sd(ln price) than the short one.
- **Browser:** the controller runs a tiny built-in sweep, cancels one mid-run, and opens a CLI result file, with the puppeteer harness.

## Docs

README: CLI usage and an Experiments section; roadmap: mark "Parameter sweeps / batch runs" and "Headless CLI" done.
