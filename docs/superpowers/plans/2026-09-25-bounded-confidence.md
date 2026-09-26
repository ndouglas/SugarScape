# Bounded Confidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Hegselmann and Krause's bounded-confidence opinion dynamics as a tenth model kind, `opinions` ("Bounded Confidence"), with the paper's symmetric, asymmetric and opinion-dependent confidence, its two unfigured claims (serial updating, lattice neighborhoods) as switches, thirteen presets, six measured sweeps and a 19-claim survey, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/opinions/` — `config.rs` (parameters, reach, validation, schema), `stats.rs` (clusters, splits, the snapshot), `view.rs` (the opinion × time frame's geometry and colors), `world.rs` (`OpinionsWorld`: the BC step in three update orders and on a lattice, the history the diagram draws, Inspect), `presets.rs`, `mod.rs` — wired into `ModelConfig`/`ModelWorld` like the classes model. The page adds the model's types, color modes, charts, Inspect, a Compare entry and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-25-bounded-confidence-design.md` (binding, as amended in Task 5). Source: Hegselmann & Krause, "Opinion Dynamics and Bounded Confidence: Models, Analysis, and Simulation", *JASSS* 5(3) (2002); Lorenz, "Consensus Strikes Back …", *JASSS* 9(1) (2006).

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited (`MODEL_GOLDEN` gains thirteen `hk-*` entries).
- **One engine path; deterministic; portable:** native and WASM fingerprints identical (verified in planning by `wasm-pack test` and the web determinism test).
- **Literal defaults, named departures, honest descriptions.**
- **Copy (verbatim):** model label **Bounded Confidence**; preset ids `hk-plurality`, `hk-polarisation`, `hk-consensus`, `hk-regular-50`, `hk-regular-plurality`, `hk-regular-consensus`, `hk-asym-a`, `hk-asym-b`, `hk-asym-c`, `hk-one-sided`, `hk-bias`, `hk-serial`, `hk-lattice`; Compare entry **Simultaneous vs serial updating — Bounded Confidence (Compare)** (id `hk-simultaneous-vs-serial`); color modes **Start**, **Opinion**; schema groups **Population**, **Confidence**, **Updating**, **Lattice**; charts **Clusters**, **Largest camps**, **Mean and median**, **Splits**, **Change**; time axis **Periods**; sweeps `hk-diagonal`, `hk-asymmetry`, `hk-bias`, `hk-updating`, `hk-lattice`, `hk-population`; series `clusters, largest, second, mean_opinion, median_opinion, range, splits, one_sided_splits, max_change, stable_at`; notice `Stable at t = 8: no opinion moves any more — Reset to run it again`; CLI `(stable)`.
- Every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE`. Stage only the task's files; never `.claude/`.
- Rust: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. In `survey/`, format only `survey/src/claims/opinions.rs` (`rustfmt --edition 2021`); its `ch6.rs` clippy warnings are not ours.
- Web: `(cd web && npm run build && npm test)`.
- **Browser checks are the controller's** (Task 3's list; the full pass in Task 5).

## Review Focus

1. **Floating-point stillness** — averaging k equal doubles need not return the same double, and serial updating closes camps only geometrically: a run must still stop, and one camp must count as one. Pinned in Task 1 by `a_split_profile_stabilizes_and_stops` (stability at 10⁻¹⁰, runs past the stop stay put) and `clusters_join_opinions_within_the_tolerance` (clusters at 10⁻⁶); in Task 4 by `opinions.serial.phases` (random draws still reach consensus).
2. **The fast path and the plain path agree** — the sorted prefix-sum mean must equal direct summation for every confidence shape, and equal opinions must stay equal. Pinned in Task 1 by `prefix_sums_agree_with_direct_summation`.
3. **Degenerate reaches and sizes** — ε 0 and 1, εl = εr = 0, m = 1 at the edges, two agents, a 3 × 3 torus: no panic, no NaN. Pinned in Task 1 by `degenerate_configs_run_without_panicking` and `the_reach_follows_each_confidence`.
4. **Live edits and keyframes** — confidence, updating and the stop apply live; agents, start and the lattice need Reset; stepping back restores the diagram's history, not just the opinions. Pinned in Task 1 by `live_edits_apply_and_the_population_waits_for_reset`, `keyframes_restore_opinions_and_the_diagram`, `schema_paths_exist_and_match_what_set_config_allows`.
5. **Inspect across the frame** — the oldest and newest diagram columns, the gap before the lattice, the lattice's sites, and off the frame. Pinned in Task 1 by `inspect_finds_lines_and_sites`; in Task 3 by `stops once when stable, matches the native golden entry and inspects a line`.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --workspace`, `cargo clippy --all-targets -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (37), `npm run build && npm test` (529) and the survey (19 claims in under a minute).

1. **Two tolerances (amends the spec):** a period is stable when no opinion moves more than 10⁻¹⁰ (`STILL`); opinions within 10⁻⁶ are one surviving opinion (`SAME`). One tolerance of 10⁻¹⁰ counted a serially updated consensus as two or three clusters (its camp was still closing geometrically when the moves fell below 10⁻¹⁰).
2. **The frame (amends the spec):** 241 × 201 cells (60 periods of 4 cells, plus the current column; opinion rows 0.005 apart). The lattice is not a color mode but a panel drawn right of the diagram (the frame grows to 450 wide), because Inspect has no color mode to tell which view a click is in. Color modes: **Start** and **Opinion**.
3. **Lattice sizes in the Rules panel** (the classes model left them out): `lattice.width` and `lattice.height` are schema fields shown only with the lattice; so are `epsilon_left`, `epsilon_right` (asymmetric) and `bias` (opinion-dependent), via `shown_if`.
4. **"Polarized"** means the second-largest camp holds at least a fifth of the agents (the survey and descriptions); `largest + second ≥ 0.9` wrongly called a lattice's one big camp with stranded minorities two camps.
5. **Sweep shapes:** a series cannot scale with x, so `hk-asymmetry` runs Fig. 12c's grid rows (fixed εl = 0.02, 0.1, 0.2 against εr) instead of Fig. 11's lines εl = k·εr. `hk-bias` reads the final `range`, `hk-lattice` the final `second` at 2000 periods, `hk-population` the final `largest`.
6. **The web golden list** gains only `hk-lattice` (every other preset stops before its 200 ticks); an engine test runs `hk-regular-50` to its stop at period 8 and checks its golden fingerprint there.
7. **Planning's findings** (20 seeds unless stated; the survey reproduces them): Fig. 2a's 38 survivors reproduce (median 37.5); **Fig. 2b's two camps at ε = 0.15 are the exception — 6 of 20 runs; 14 end with a third camp in the middle, usually as large**; Fig. 2c's consensus reproduces; **"less than 15 periods" holds for 53 of 60 runs, the slowest taking 168**; Fig. 3's phases reproduce (50 runs: consensus in 13 at 0.21, 30 at 0.22, 50 at 0.25); the evenly spaced figures reproduce exactly (split at t6 and still from period 8; 8 splits; consensus); the asymmetric drift and one-sided splits that close reproduce; the opinion-dependent break at ε = 0.6 comes between m = 0.44 (18 of 20 in consensus) and 0.52 (8 of 20), the paper's "≈ 0.4"; camps reach 0 and 1 at m = 1; serial updating keeps the phases and slightly more survivors; **HK's unfigured lattice claim holds** — one big camp and stranded local minorities instead of two camps (a second camp of a fifth in 9 of 200 lattice runs against 57 of 100 among everyone); and Lorenz's point holds (at ε = 0.22 consensus in 1 of 20 runs with 50 agents, 12 of 20 with 1000).

---

### Task 1: The bounded-confidence model in the core

**Files:**
- Create: `crates/sugarscape-core/src/opinions/{stats,view,config,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `model.rs`, `presets.rs`, `tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{Rgb, BACKGROUND}`, `crate::rng`, `crate::schema::{Apply, Param}` (with `shown_if`, `with_help`), `crate::presets::ModelPreset`.
- Produces: `opinions::{OpinionsConfig, LatticeConfig, Start, Confidence, Updating, Interaction, Neighborhood, schema, presets, SERIES, OpinionsSnapshot, STILL, SAME, clusters, splits, median, hue, row, opinion_at, HISTORY, STEP, WIDE, TALL, GAP, LATTICE_X, OpinionsWorld, OpinionsMode, OpinionsInspection, OpinionsCell, OpinionAgent}`; `OpinionsConfig::reach(x) -> (f64, f64)`; `ModelKind::Opinions` (`"opinions"`), `ModelConfig::Opinions`, `ModelWorld::Opinions`.

- [ ] **Step 1: Write the module**

Create `crates/sugarscape-core/src/opinions/stats.rs` with exactly this content:

```rust
//! The Bounded Confidence model's statistics: surviving opinions, camps,
//! splits and stability, read from the sorted opinion profile.

use serde::Serialize;

use crate::stats::Series;

/// A period whose largest move is this small is stable. Averaging k equal
/// doubles need not return the same double, so exact stillness is not
/// guaranteed.
pub const STILL: f64 = 1e-10;

/// Opinions this close are one surviving opinion: serial updating closes
/// camps geometrically, leaving spreads far above `STILL` but far below any
/// confidence the paper uses.
pub const SAME: f64 = 1e-6;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 10] = [
    "clusters",
    "largest",
    "second",
    "mean_opinion",
    "median_opinion",
    "range",
    "splits",
    "one_sided_splits",
    "max_change",
    "stable_at",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct OpinionsSnapshot {
    pub tick: u64,
    /// Surviving opinions: maximal runs of the sorted profile whose gaps
    /// are at most `SAME`.
    pub clusters: u32,
    /// Shares of agents in the biggest and second-biggest clusters.
    pub largest: f64,
    pub second: f64,
    pub mean_opinion: f64,
    pub median_opinion: f64,
    /// The largest opinion minus the smallest.
    pub range: f64,
    /// Gaps between sorted neighbors beyond both reaches (the lower one's
    /// εr, the upper one's εl), and gaps beyond exactly one of them.
    pub splits: u32,
    pub one_sided_splits: u32,
    /// The largest move this period (0 at the start).
    pub max_change: f64,
    /// The first stable period, else this one.
    pub stable_at: u64,
}

impl Series for OpinionsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "clusters" => f64::from(self.clusters),
            "largest" => self.largest,
            "second" => self.second,
            "mean_opinion" => self.mean_opinion,
            "median_opinion" => self.median_opinion,
            "range" => self.range,
            "splits" => f64::from(self.splits),
            "one_sided_splits" => f64::from(self.one_sided_splits),
            "max_change" => self.max_change,
            "stable_at" => self.stable_at as f64,
            _ => return None,
        })
    }
}

/// The sizes of the clusters of a sorted profile, in order.
pub fn clusters(sorted: &[f64]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut run = 0;
    for (k, &x) in sorted.iter().enumerate() {
        if k > 0 && x - sorted[k - 1] > SAME {
            out.push(run);
            run = 0;
        }
        run += 1;
    }
    if run > 0 {
        out.push(run);
    }
    out
}

/// Two-sided and one-sided splits of a sorted profile, given each opinion's
/// reach `(εl, εr)`.
pub fn splits(sorted: &[f64], reach: impl Fn(f64) -> (f64, f64)) -> (u32, u32) {
    let (mut two, mut one) = (0, 0);
    for w in sorted.windows(2) {
        let gap = w[1] - w[0];
        if gap <= SAME {
            continue;
        }
        let up = gap <= reach(w[0]).1;
        let down = gap <= reach(w[1]).0;
        match (up, down) {
            (false, false) => two += 1,
            (true, true) => {}
            _ => one += 1,
        }
    }
    (two, one)
}

/// The median of a sorted, nonempty profile.
pub fn median(sorted: &[f64]) -> f64 {
    let n = sorted.len();
    if !n.is_multiple_of(2) {
        sorted[n / 2]
    } else {
        (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clusters_join_opinions_within_the_tolerance() {
        assert_eq!(
            clusters(&[0.1, 0.1, 0.1 + 1e-7, 0.5, 0.9, 0.9 + 1e-5]),
            [3, 1, 1, 1]
        );
        assert_eq!(clusters(&[0.3]), [1]);
        assert!(clusters(&[]).is_empty());
    }

    #[test]
    fn splits_count_gaps_beyond_one_or_both_reaches() {
        let sym = |_| (0.1, 0.1);
        assert_eq!(splits(&[0.0, 0.05, 0.3, 0.3, 0.6], sym), (2, 0));
        // Reaching 0.2 right but 0.05 left: a 0.1 gap is one-sided.
        let asym = |_| (0.05, 0.2);
        assert_eq!(splits(&[0.0, 0.1, 0.5], asym), (1, 1));
    }

    #[test]
    fn medians_of_odd_and_even_profiles() {
        assert_eq!(median(&[0.1, 0.2, 0.9]), 0.2);
        assert!((median(&[0.1, 0.2, 0.4, 0.9]) - 0.3).abs() < 1e-12);
    }
}
```

Create `crates/sugarscape-core/src/opinions/view.rs` with exactly this content:

```rust
//! The opinion × time diagram (HK's Figs. 2, 4, 7, 8, 13 and 18): opinion
//! up, periods left to right, each agent a line colored red → magenta by its
//! starting opinion, gray where sorted neighbors are within each other's
//! reach; with a lattice, the torus to its right.

use crate::render::{Rgb, BACKGROUND};

/// Opinion profiles kept for the diagram: the current one and 60 before it.
pub const HISTORY: usize = 61;
/// Cells per period.
pub const STEP: usize = 4;
/// The diagram's width and height in cells.
pub const WIDE: usize = STEP * (HISTORY - 1) + 1;
pub const TALL: usize = 201;
/// Cells between the diagram and the lattice.
pub const GAP: usize = 8;
/// Where the lattice panel starts.
pub const LATTICE_X: usize = WIDE + GAP;

/// The gray between neighbors within each other's reach.
pub const BAND: Rgb = [0x3a, 0x38, 0x33];

/// The diagram's row for an opinion: 1 at the top, 0 at the bottom.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(0.0, 1.0)) * (TALL - 1) as f64).round() as usize
}

/// The opinion at a diagram row.
pub fn opinion_at(y: usize) -> f64 {
    1.0 - y as f64 / (TALL - 1) as f64
}

/// HK's coloring: red at 0 through the spectrum to magenta at 1.
pub fn hue(x: f64) -> Rgb {
    let h = x.clamp(0.0, 1.0) * 5.0; // six sextants of 60°; magenta ends the fifth
    let k = h.floor().min(4.0);
    let f = h - k;
    let (hi, lo) = (255.0, 40.0);
    let up = lo + (hi - lo) * f;
    let down = hi - (hi - lo) * f;
    let [r, g, b] = match k as u8 {
        0 => [hi, up, lo],
        1 => [down, hi, lo],
        2 => [lo, hi, up],
        3 => [lo, down, hi],
        _ => [up, lo, hi],
    };
    [r as u8, g as u8, b as u8]
}

/// Cells per lattice site, so the lattice fits the diagram's height.
pub fn site_cells(width: u32, height: u32) -> usize {
    (TALL / width.max(height) as usize).max(1)
}

/// A frame being drawn.
pub struct Canvas<'a> {
    pub buf: &'a mut Vec<u8>,
    pub wide: usize,
}

impl Canvas<'_> {
    pub fn clear(&mut self, wide: usize, tall: usize) {
        self.wide = wide;
        self.buf.clear();
        self.buf.resize(wide * tall * 4, 0);
        for k in 0..wide * tall {
            self.buf[k * 4..k * 4 + 4].copy_from_slice(&[
                BACKGROUND[0],
                BACKGROUND[1],
                BACKGROUND[2],
                255,
            ]);
        }
    }

    pub fn put(&mut self, x: usize, y: usize, c: Rgb) {
        let k = (y * self.wide + x) * 4;
        self.buf[k..k + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
    }

    /// A vertical run of cells from row `a` to row `b`, either order.
    pub fn column(&mut self, x: usize, a: usize, b: usize, c: Rgb) {
        for y in a.min(b)..=a.max(b) {
            self.put(x, y, c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_run_from_one_at_the_top_to_zero_at_the_bottom() {
        assert_eq!((row(1.0), row(0.0), row(0.5)), (0, TALL - 1, 100));
        assert!((opinion_at(row(0.37)) - 0.37).abs() <= 0.5 / (TALL - 1) as f64);
    }

    #[test]
    fn hues_go_from_red_to_magenta() {
        assert_eq!(hue(0.0), [255, 40, 40]);
        assert_eq!(hue(1.0), [255, 40, 255]);
        assert_eq!(hue(0.4), [40, 255, 40]);
    }

    #[test]
    fn the_lattice_fits_the_diagrams_height() {
        assert_eq!(site_cells(25, 25), 8);
        assert!(site_cells(44, 3) * 44 <= TALL);
        assert_eq!(WIDE, 241);
    }
}
```

Create `crates/sugarscape-core/src/opinions/config.rs` with exactly this content:

```rust
//! The Bounded Confidence model's parameters: Hegselmann and Krause's (2002)
//! symmetric, opinion-independent and opinion-dependent confidence, with
//! their two unfigured claims (random serial updating, local neighborhoods)
//! as switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How opinions start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// Uniform on [0, 1] (HK's random start profiles).
    Random,
    /// Evenly spaced, i/(n − 1): both ends included (HK's regular profiles).
    Regular,
}

/// The shape of every agent's confidence interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// εl = εr = ε (§4.1).
    Symmetric,
    /// Fixed εl and εr (§4.2.1).
    Asymmetric,
    /// A total ε split by the agent's own opinion (§4.2.2):
    /// εr = (m·x + (1 − m)/2)·ε, εl = ε − εr.
    OpinionDependent,
}

/// Who revises when.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Updating {
    /// Everyone at once from the last period's opinions (HK).
    Simultaneous,
    /// Each agent once per period, in a fresh random order, seeing opinions
    /// as they change.
    SerialShuffled,
    /// n agents drawn uniformly with replacement per period.
    SerialRandom,
}

/// Whose opinions an agent can take into account.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    /// Everyone (HK).
    All,
    /// Itself and its torus neighbors (HK §4.3's "first simulations").
    Lattice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    Moore,
    VonNeumann,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatticeConfig {
    pub width: u32,
    pub height: u32,
    pub neighborhood: Neighborhood,
}

impl Default for LatticeConfig {
    /// 25 × 25: HK's 625 agents on a torus, eight neighbors each.
    fn default() -> Self {
        LatticeConfig {
            width: 25,
            height: 25,
            neighborhood: Neighborhood::Moore,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct OpinionsConfig {
    /// n; with a lattice, width × height.
    pub agents: u32,
    pub start: Start,
    pub confidence: Confidence,
    /// ε: the reach each way (symmetric), or the total (opinion-dependent).
    pub epsilon: f64,
    /// εl and εr under `asymmetric`.
    pub epsilon_left: f64,
    pub epsilon_right: f64,
    /// m, the slope of the opinion-dependent bias.
    pub bias: f64,
    pub updating: Updating,
    pub interaction: Interaction,
    pub lattice: LatticeConfig,
    /// Stop at the first stable period.
    pub stop_when_stable: bool,
}

impl Default for OpinionsConfig {
    /// HK's Fig. 2b: 625 random opinions, ε = 0.15, simultaneous updating.
    fn default() -> Self {
        OpinionsConfig {
            agents: 625,
            start: Start::Random,
            confidence: Confidence::Symmetric,
            epsilon: 0.15,
            epsilon_left: 0.1,
            epsilon_right: 0.2,
            bias: 0.5,
            updating: Updating::Simultaneous,
            interaction: Interaction::All,
            lattice: LatticeConfig::default(),
            stop_when_stable: true,
        }
    }
}

impl OpinionsConfig {
    /// How far an agent at opinion `x` reaches left and right.
    pub fn reach(&self, x: f64) -> (f64, f64) {
        match self.confidence {
            Confidence::Symmetric => (self.epsilon, self.epsilon),
            Confidence::Asymmetric => (self.epsilon_left, self.epsilon_right),
            Confidence::OpinionDependent => {
                let right = (self.bias * x + (1.0 - self.bias) / 2.0) * self.epsilon;
                (self.epsilon - right, right)
            }
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (2..=2000).contains(&self.agents),
            "agents",
            "must be between 2 and 2000",
        );
        check(unit(self.epsilon), "epsilon", "must be between 0 and 1");
        check(
            unit(self.epsilon_left),
            "epsilon_left",
            "must be between 0 and 1",
        );
        check(
            unit(self.epsilon_right),
            "epsilon_right",
            "must be between 0 and 1",
        );
        check(unit(self.bias), "bias", "must be between 0 and 1");
        if self.interaction == Interaction::Lattice {
            let l = &self.lattice;
            check(
                (3..=44).contains(&l.width) && (3..=44).contains(&l.height),
                "lattice",
                "sides must be between 3 and 44",
            );
            check(
                self.agents == l.width * l.height,
                "agents",
                "must be the lattice's width × height",
            );
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &OpinionsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("start", self.start == next.start),
            ("interaction", self.interaction == next.interaction),
            ("lattice", self.lattice == next.lattice),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Population", "agents", "Agents (n)", (2, 2000), Reset)
            .with_help("HK: 625 random opinions, or 50 and 100 evenly spaced. With a lattice, width × height."),
        Param::choice(
            "Population",
            "start",
            "Opinions start",
            &[
                ("random", "At random (uniform)"),
                ("regular", "Evenly spaced"),
            ],
            Reset,
        ),
        Param::bool("Population", "stop_when_stable", "Stop when stable", Live)
            .with_help("Stable: no opinion moved more than 10⁻¹⁰ this period. Opinions within 10⁻⁶ count as one."),
        Param::choice(
            "Confidence",
            "confidence",
            "Confidence",
            &[
                ("symmetric", "Symmetric (§4.1)"),
                ("asymmetric", "Asymmetric, the same for all (§4.2.1)"),
                ("opinion_dependent", "Leaning with one's opinion (§4.2.2)"),
            ],
            Live,
        ),
        Param::number("Confidence", "epsilon", "Confidence (ε)", (0.0, 1.0, 0.01), Live)
            .with_help("Symmetric: the reach each way. Leaning: the total, split by the bias."),
        Param::number(
            "Confidence",
            "epsilon_left",
            "Left reach (εl)",
            (0.0, 1.0, 0.01),
            Live,
        )
        .shown_if("confidence", "asymmetric"),
        Param::number(
            "Confidence",
            "epsilon_right",
            "Right reach (εr)",
            (0.0, 1.0, 0.01),
            Live,
        )
        .shown_if("confidence", "asymmetric"),
        Param::number("Confidence", "bias", "Bias (m)", (0.0, 1.0, 0.01), Live)
            .shown_if("confidence", "opinion_dependent")
            .with_help("HK §4.2.2: 0 is symmetric; at 1 an agent at 0 or 1 listens only to its own side."),
        Param::choice(
            "Updating",
            "updating",
            "Updating",
            &[
                ("simultaneous", "Simultaneous (HK)"),
                ("serial_shuffled", "Serial, each once in random order"),
                ("serial_random", "Serial, n random draws"),
            ],
            Live,
        )
        .with_help("HK §4.3: 'none of the results … depends crucially on simultaneous updating' — which serial order is not said."),
        Param::choice(
            "Lattice",
            "interaction",
            "Who listens to whom",
            &[
                ("all", "Everyone (HK)"),
                ("lattice", "Lattice neighbors (HK §4.3)"),
            ],
            Reset,
        )
        .with_help("HK §4.3: with small overlapping neighborhoods 'polarization disappears' — reported, not shown."),
        Param::integer("Lattice", "lattice.width", "Width", (3, 44), Reset)
            .shown_if("interaction", "lattice"),
        Param::integer("Lattice", "lattice.height", "Height", (3, 44), Reset)
            .shown_if("interaction", "lattice"),
        Param::choice(
            "Lattice",
            "lattice.neighborhood",
            "Neighbors",
            &[("moore", "8 (Moore)"), ("von_neumann", "4 (von Neumann)")],
            Reset,
        )
        .shown_if("interaction", "lattice"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_hks_figure_2b() {
        let c = OpinionsConfig::default();
        assert_eq!((c.agents, c.epsilon), (625, 0.15));
        assert_eq!(
            (c.start, c.confidence, c.updating, c.interaction),
            (
                Start::Random,
                Confidence::Symmetric,
                Updating::Simultaneous,
                Interaction::All
            )
        );
        assert!(c.stop_when_stable && c.validate().is_ok());
    }

    #[test]
    fn the_reach_follows_each_confidence() {
        let mut c = OpinionsConfig::default();
        assert_eq!(c.reach(0.9), (0.15, 0.15));
        c.confidence = Confidence::Asymmetric;
        assert_eq!(c.reach(0.9), (0.1, 0.2));
        // HK's worked example: x 0.6, ε 0.4, m 0.5 → εl 0.18, εr 0.22.
        c.confidence = Confidence::OpinionDependent;
        c.epsilon = 0.4;
        let (l, r) = c.reach(0.6);
        assert!(
            (l - 0.18).abs() < 1e-12 && (r - 0.22).abs() < 1e-12,
            "{l} {r}"
        );
        let (l, r) = c.reach(0.5);
        assert!((l - r).abs() < 1e-12, "the center is symmetric");
        c.bias = 1.0;
        assert_eq!(
            c.reach(0.0),
            (0.4, 0.0),
            "at m = 1 the left edge looks only left"
        );
    }

    #[test]
    fn validation_names_fields() {
        let bad = OpinionsConfig {
            agents: 1,
            epsilon: 1.5,
            epsilon_left: -0.1,
            epsilon_right: 2.0,
            bias: 1.1,
            ..OpinionsConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["agents", "epsilon", "epsilon_left", "epsilon_right", "bias"]
        );
        let lattice = OpinionsConfig {
            agents: 600,
            interaction: Interaction::Lattice,
            ..OpinionsConfig::default()
        };
        assert_eq!(lattice.validate().unwrap_err()[0].field, "agents");
        let tiny = OpinionsConfig {
            agents: 4,
            interaction: Interaction::Lattice,
            lattice: LatticeConfig {
                width: 2,
                height: 2,
                neighborhood: Neighborhood::Moore,
            },
            ..OpinionsConfig::default()
        };
        assert_eq!(tiny.validate().unwrap_err()[0].field, "lattice");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Opinions(OpinionsConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

Create `crates/sugarscape-core/src/opinions/world.rs` with exactly this content:

```rust
//! The bounded-confidence world: each agent moves to the mean of the
//! opinions within its reach (HK's eq. BC), all at once or one at a time,
//! among everyone or among lattice neighbors.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{Interaction, Neighborhood, OpinionsConfig, Start, Updating};
use super::stats::{self, OpinionsSnapshot, SAME, STILL};
use super::view::{
    hue, opinion_at, row, site_cells, Canvas, BAND, HISTORY, LATTICE_X, STEP, TALL, WIDE,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Every site's torus neighbors (empty when everyone listens to everyone).
#[derive(Debug, Default, PartialEq)]
pub struct Neighbors {
    start: Vec<u32>,
    list: Vec<u32>,
}

impl Neighbors {
    fn new(c: &OpinionsConfig) -> Self {
        let l = &c.lattice;
        let (w, h) = (l.width as i32, l.height as i32);
        let offs: &[(i32, i32)] = match l.neighborhood {
            Neighborhood::Moore => &[
                (-1, -1),
                (0, -1),
                (1, -1),
                (-1, 0),
                (1, 0),
                (-1, 1),
                (0, 1),
                (1, 1),
            ],
            Neighborhood::VonNeumann => &[(0, -1), (-1, 0), (1, 0), (0, 1)],
        };
        let mut start = Vec::new();
        let mut list = Vec::new();
        for y in 0..h {
            for x in 0..w {
                start.push(list.len() as u32);
                for &(dx, dy) in offs {
                    list.push(((y + dy).rem_euclid(h) * w + (x + dx).rem_euclid(w)) as u32);
                }
            }
        }
        start.push(list.len() as u32);
        Neighbors { start, list }
    }

    pub fn of(&self, i: usize) -> &[u32] {
        &self.list[self.start[i] as usize..self.start[i + 1] as usize]
    }
}

/// The diagram's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpinionsMode {
    /// Each line by its starting opinion (HK).
    Start,
    /// Each line by its current opinion.
    Opinion,
}

impl std::str::FromStr for OpinionsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "start" => Self::Start,
            "opinion" => Self::Opinion,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OpinionsInspection {
    pub site: OpinionsCell,
    /// The period of the diagram column, null off the diagram.
    pub period: Option<u64>,
    /// The opinion at the diagram row, null off the diagram.
    pub opinion: Option<f64>,
    /// The lattice site clicked, null off the lattice.
    pub lattice_site: Option<OpinionsCell>,
    /// The agents whose lines pass within one cell, or the site's agent.
    pub agents: Vec<OpinionAgent>,
    /// Always null: agents have no place to follow.
    pub agent: Option<OpinionAgent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct OpinionsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct OpinionAgent {
    pub id: u64,
    pub start: f64,
    /// The opinion in the inspected period (the current one on the lattice).
    pub opinion: f64,
    pub epsilon_left: f64,
    pub epsilon_right: f64,
    /// How many agents it takes into account now, itself included.
    pub reaches: u32,
}

#[derive(Clone)]
pub struct OpinionsWorld {
    pub config: OpinionsConfig,
    /// Completed periods.
    pub tick: u64,
    start: Vec<f64>,
    x: Vec<f64>,
    neighbors: Arc<Neighbors>,
    rng: SimRng,
    /// The last `HISTORY` profiles, oldest first; the last is `x`.
    history: VecDeque<Arc<Vec<f64>>>,
    max_change: f64,
    stable_at: Option<u64>,
    pub stats: Stats<OpinionsSnapshot>,
}

impl OpinionsWorld {
    pub fn new(config: OpinionsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.agents as usize;
        let start: Vec<f64> = match config.start {
            Start::Random => (0..n).map(|_| rng.gen::<f64>()).collect(),
            Start::Regular => (0..n).map(|i| i as f64 / (n - 1) as f64).collect(),
        };
        let neighbors = Arc::new(match config.interaction {
            Interaction::All => Neighbors::default(),
            Interaction::Lattice => Neighbors::new(&config),
        });
        let mut world = OpinionsWorld {
            config,
            tick: 0,
            x: start.clone(),
            history: VecDeque::from([Arc::new(start.clone())]),
            start,
            neighbors,
            rng,
            max_change: 0.0,
            stable_at: None,
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    pub fn opinions(&self) -> &[f64] {
        &self.x
    }

    pub fn starts(&self) -> &[f64] {
        &self.start
    }

    /// Whether the run has stopped: stable with `stop_when_stable`.
    pub fn is_finished(&self) -> bool {
        self.config.stop_when_stable && self.stable_at.is_some()
    }

    /// Whether agent `i`, at `xi`, takes the opinion `xj` into account.
    fn within(&self, xi: f64, xj: f64) -> bool {
        let (l, r) = self.config.reach(xi);
        xi - l <= xj && xj <= xi + r
    }

    /// The mean of what agent `i` reaches in profile `x`, summed directly:
    /// itself and its neighbors on a lattice, everyone in index order otherwise.
    fn direct_mean(&self, i: usize, x: &[f64]) -> f64 {
        let xi = x[i];
        let (mut sum, mut count) = (0.0, 0u32);
        match self.config.interaction {
            Interaction::Lattice => {
                sum += xi;
                count += 1;
                for &j in self.neighbors.of(i) {
                    let xj = x[j as usize];
                    if self.within(xi, xj) {
                        sum += xj;
                        count += 1;
                    }
                }
            }
            Interaction::All => {
                for &xj in x {
                    if self.within(xi, xj) {
                        sum += xj;
                        count += 1;
                    }
                }
            }
        }
        sum / f64::from(count)
    }

    /// How many agents `i` reaches in profile `x`, itself included.
    fn reaches(&self, i: usize, x: &[f64]) -> u32 {
        match self.config.interaction {
            Interaction::Lattice => {
                1 + self
                    .neighbors
                    .of(i)
                    .iter()
                    .filter(|&&j| self.within(x[i], x[j as usize]))
                    .count() as u32
            }
            Interaction::All => x.iter().filter(|&&xj| self.within(x[i], xj)).count() as u32,
        }
    }

    /// Everyone's next opinion at once. Among everyone, each reach is a run
    /// of the sorted profile, summed from prefix sums.
    fn simultaneous(&self) -> Vec<f64> {
        let n = self.x.len();
        match self.config.interaction {
            Interaction::Lattice => (0..n).map(|i| self.direct_mean(i, &self.x)).collect(),
            Interaction::All => {
                let mut sorted = self.x.clone();
                sorted.sort_by(f64::total_cmp);
                let mut prefix = Vec::with_capacity(n + 1);
                prefix.push(0.0);
                for &v in &sorted {
                    prefix.push(prefix.last().unwrap() + v);
                }
                self.x
                    .iter()
                    .map(|&xi| {
                        let (l, r) = self.config.reach(xi);
                        let lo = sorted.partition_point(|&v| v < xi - l);
                        let hi = sorted.partition_point(|&v| v <= xi + r);
                        (prefix[hi] - prefix[lo]) / (hi - lo) as f64
                    })
                    .collect()
            }
        }
    }

    /// One period.
    pub fn step(&mut self) {
        let before = self.x.clone();
        match self.config.updating {
            Updating::Simultaneous => self.x = self.simultaneous(),
            Updating::SerialShuffled => {
                let mut order: Vec<usize> = (0..self.x.len()).collect();
                order.shuffle(&mut self.rng);
                for i in order {
                    self.x[i] = self.direct_mean(i, &self.x);
                }
            }
            Updating::SerialRandom => {
                let n = self.x.len();
                for _ in 0..n {
                    let i = self.rng.gen_range(0..n as u32) as usize;
                    self.x[i] = self.direct_mean(i, &self.x);
                }
            }
        }
        self.max_change = before
            .iter()
            .zip(&self.x)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f64::max);
        self.tick += 1;
        if self.stable_at.is_none() && self.max_change <= STILL {
            self.stable_at = Some(self.tick);
        }
        if self.history.len() == HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(Arc::new(self.x.clone()));
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    fn record(&mut self) {
        let mut sorted = self.x.clone();
        sorted.sort_by(f64::total_cmp);
        let n = sorted.len() as f64;
        let mut sizes = stats::clusters(&sorted);
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        let (splits, one_sided) = stats::splits(&sorted, |x| self.config.reach(x));
        self.stats.push(OpinionsSnapshot {
            tick: self.tick,
            clusters: sizes.len() as u32,
            largest: sizes.first().map_or(0.0, |&s| s as f64 / n),
            second: sizes.get(1).map_or(0.0, |&s| s as f64 / n),
            mean_opinion: sorted.iter().sum::<f64>() / n,
            median_opinion: stats::median(&sorted),
            range: sorted[sorted.len() - 1] - sorted[0],
            splits,
            one_sided_splits: one_sided,
            max_change: self.max_change,
            stable_at: self.stable_at.unwrap_or(self.tick),
        });
    }

    /// The period shown in diagram column `k` (0 is the oldest kept).
    fn period_of(&self, k: usize) -> u64 {
        self.tick + 1 - (self.history.len() - k) as u64
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<OpinionsInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let site = OpinionsCell { x, y };
        let (cx, cy) = (x as usize, y as usize);
        let view = |i: usize, opinion: f64| {
            let (l, r) = self.config.reach(self.x[i]);
            OpinionAgent {
                id: i as u64 + 1,
                start: self.start[i],
                opinion,
                epsilon_left: l,
                epsilon_right: r,
                reaches: self.reaches(i, &self.x),
            }
        };
        let mut out = OpinionsInspection {
            site,
            period: None,
            opinion: None,
            lattice_site: None,
            agents: Vec::new(),
            agent: None,
        };
        if cx < WIDE {
            // The nearest kept period at or left of the column.
            let k = (cx / STEP).min(self.history.len() - 1);
            let profile = &self.history[k];
            out.period = Some(self.period_of(k));
            out.opinion = Some(opinion_at(cy));
            out.agents = (0..profile.len())
                .filter(|&i| row(profile[i]).abs_diff(cy) <= 1)
                .map(|i| view(i, profile[i]))
                .collect();
        } else if self.config.interaction == Interaction::Lattice && cx >= LATTICE_X {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            let (sx, sy) = ((cx - LATTICE_X) / s, cy / s);
            if sx < l.width as usize && sy < l.height as usize {
                out.lattice_site = Some(OpinionsCell {
                    x: sx as u32,
                    y: sy as u32,
                });
                let i = sy * l.width as usize + sx;
                out.agents = vec![view(i, self.x[i])];
            }
        }
        Ok(out)
    }
}

impl Model for OpinionsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Opinions(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        OpinionsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.x.len()
    }

    /// FNV-1a over the tick and every opinion's bits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: [u8; 8]| {
            for b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick.to_le_bytes());
        for v in &self.x {
            eat(v.to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let w = match self.config.interaction {
            Interaction::All => WIDE,
            Interaction::Lattice => LATTICE_X + TALL,
        };
        (w as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: OpinionsMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let kept = self.history.len();
        // Gray between sorted neighbors within each other's reach, per period.
        for (k, profile) in self.history.iter().enumerate() {
            let mut sorted = profile.to_vec();
            sorted.sort_by(f64::total_cmp);
            let cols = if k + 1 == kept { 1 } else { STEP };
            for w in sorted.windows(2) {
                let gap = w[1] - w[0];
                if gap > SAME
                    && gap <= self.config.reach(w[0]).1
                    && gap <= self.config.reach(w[1]).0
                {
                    for d in 0..cols {
                        c.column(k * STEP + d, row(w[0]), row(w[1]), BAND);
                    }
                }
            }
        }
        // Lines, drawn in order of starting opinion (later ones on top, as in HK).
        let mut order: Vec<usize> = (0..self.x.len()).collect();
        order.sort_by(|&a, &b| self.start[a].total_cmp(&self.start[b]));
        for &i in &order {
            let color = match mode {
                OpinionsMode::Start => hue(self.start[i]),
                OpinionsMode::Opinion => hue(self.x[i]),
            };
            for k in 0..kept {
                let y0 = row(self.history[k][i]) as f64;
                if k + 1 == kept {
                    c.put(k * STEP, y0 as usize, color);
                    continue;
                }
                let y1 = row(self.history[k + 1][i]) as f64;
                for d in 0..STEP {
                    let a = y0 + (y1 - y0) * d as f64 / STEP as f64;
                    let b = y0 + (y1 - y0) * (d + 1) as f64 / STEP as f64;
                    c.column(k * STEP + d, a.round() as usize, b.round() as usize, color);
                }
            }
        }
        if self.config.interaction == Interaction::Lattice {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            for sy in 0..l.height as usize {
                for sx in 0..l.width as usize {
                    let color = hue(self.x[sy * l.width as usize + sx]);
                    for dy in 0..s {
                        for dx in 0..s {
                            c.put(LATTICE_X + sx * s + dx, sy * s + dy, color);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        super::SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,start,opinion,epsilon_left,epsilon_right,reaches\n");
        for i in 0..self.x.len() {
            let (l, r) = self.config.reach(self.x[i]);
            writeln!(
                out,
                "{},{},{},{},{},{}",
                i + 1,
                self.start[i],
                self.x[i],
                l,
                r,
                self.reaches(i, &self.x)
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Agents have no place: nothing to follow.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Opinions(next) = next else {
            return Err(wrong_model(ModelKind::Opinions, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stable: every agent's reach holds only its own cluster, so nothing moves again.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opinions::config::Confidence;

    fn config(edit: impl FnOnce(&mut OpinionsConfig)) -> OpinionsConfig {
        let mut c = OpinionsConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut OpinionsConfig)) -> OpinionsWorld {
        OpinionsWorld::new(config(edit), 1).unwrap()
    }

    #[test]
    fn starts_are_uniform_or_evenly_spaced() {
        let r = world(|_| {});
        assert!(r.starts().iter().all(|&x| (0.0..1.0).contains(&x)));
        let g = world(|c| {
            c.agents = 5;
            c.start = Start::Regular;
        });
        assert_eq!(g.starts(), [0.0, 0.25, 0.5, 0.75, 1.0]);
    }

    #[test]
    fn a_period_moves_everyone_to_the_mean_within_reach() {
        let mut w = world(|c| {
            c.agents = 5;
            c.start = Start::Regular;
            c.epsilon = 0.3;
        });
        w.step();
        // 0 reaches 0 and 0.25; 0.25 reaches 0, 0.25, 0.5; and so on.
        let want = [0.125, 0.25, 0.5, 0.75, 0.875];
        for (a, b) in w.opinions().iter().zip(want) {
            assert!((a - b).abs() < 1e-12, "{:?}", w.opinions());
        }
        let s = w.stats.latest().unwrap();
        assert!((s.max_change - 0.125).abs() < 1e-12 && s.clusters == 5);
    }

    #[test]
    fn prefix_sums_agree_with_direct_summation() {
        for confidence in [
            Confidence::Symmetric,
            Confidence::Asymmetric,
            Confidence::OpinionDependent,
        ] {
            let w = world(|c| {
                c.confidence = confidence;
                c.epsilon = 0.3;
            });
            let fast = w.simultaneous();
            for (i, v) in fast.iter().enumerate() {
                assert!((v - w.direct_mean(i, &w.x)).abs() < 1e-12, "{confidence:?}");
            }
        }
    }

    #[test]
    fn asymmetric_reach_pulls_toward_the_favored_side() {
        let mut w = world(|c| {
            c.agents = 3;
            c.start = Start::Regular;
            c.confidence = Confidence::Asymmetric;
            c.epsilon_left = 0.0;
            c.epsilon_right = 0.5;
        });
        w.step();
        // 0 sees 0 and 0.5; 0.5 sees 0.5 and 1; 1 sees only itself.
        assert_eq!(w.opinions(), [0.25, 0.75, 1.0]);
        assert_eq!(w.stats.latest().unwrap().one_sided_splits, 2);
    }

    #[test]
    fn serial_updating_sees_opinions_as_they_change() {
        let edit = |c: &mut OpinionsConfig| {
            c.agents = 10;
            c.start = Start::Regular;
            c.epsilon = 0.3;
        };
        let mut all = world(edit);
        all.step();
        let mut w = world(|c| {
            edit(c);
            c.updating = Updating::SerialShuffled;
        });
        w.step();
        assert_ne!(
            w.opinions(),
            all.opinions(),
            "later agents saw earlier moves"
        );
        let mut r = world(|c| c.updating = Updating::SerialRandom);
        r.run(3);
        assert!(r.stats.latest().unwrap().max_change > 0.0);
    }

    #[test]
    fn lattice_agents_hear_only_their_neighbors() {
        let mut w = world(|c| {
            c.interaction = Interaction::Lattice;
            c.lattice.neighborhood = Neighborhood::VonNeumann;
            c.epsilon = 1.0;
        });
        assert_eq!(w.neighbors.of(0), [600, 24, 1, 25], "the torus wraps");
        let x = w.x.clone();
        w.step();
        let want = (x[0] + x[600] + x[24] + x[1] + x[25]) / 5.0;
        assert!((w.opinions()[0] - want).abs() < 1e-12);
    }

    #[test]
    fn a_split_profile_stabilizes_and_stops() {
        let mut w = world(|c| {
            c.agents = 50;
            c.start = Start::Regular;
            c.epsilon = 0.2;
        });
        w.run(1000);
        assert!(w.is_finished());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.stable_at, w.tick);
        assert!(s.max_change <= STILL);
        assert!(s.clusters >= 2 && s.splits == s.clusters - 1);
        let mut on = w.clone();
        on.config.stop_when_stable = false;
        let before = on.opinions().to_vec();
        on.run(5);
        assert_eq!(on.tick, w.tick + 5, "without the stop it keeps stepping");
        for (a, b) in on.opinions().iter().zip(&before) {
            assert!((a - b).abs() <= STILL);
        }
    }

    #[test]
    fn the_diagram_draws_lines_bands_and_the_lattice() {
        let mut w = world(|c| {
            c.agents = 50;
            c.start = Start::Regular;
            c.epsilon = 0.2;
            c.stop_when_stable = false;
        });
        let mut buf = Vec::new();
        w.render("start", "", &mut buf).unwrap();
        assert_eq!(buf.len(), WIDE * TALL * 4);
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * WIDE + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        assert_eq!(px(&buf, 0, TALL - 1), hue(0.0), "agent 1 starts at 0");
        assert_eq!(px(&buf, 0, 0), hue(1.0), "agent 50 at 1");
        w.run(70);
        w.render("opinion", "", &mut buf).unwrap();
        assert!(w.history.len() == HISTORY);
        assert!(w.render("wealth", "", &mut buf).is_err());
        let l = world(|c| c.interaction = Interaction::Lattice);
        assert_eq!(Model::size(&l), ((LATTICE_X + TALL) as u32, TALL as u32));
        l.render("start", "", &mut buf).unwrap();
        let k = LATTICE_X * 4;
        assert_eq!(buf[k..k + 3], hue(l.opinions()[0]));
    }

    #[test]
    fn inspect_finds_lines_and_sites() {
        let mut w = world(|c| {
            c.agents = 5;
            c.start = Start::Regular;
            c.epsilon = 0.3;
        });
        w.run(2);
        let v = w.inspect(0, (TALL - 1) as u32).unwrap();
        assert_eq!((v.period, v.opinion), (Some(0), Some(0.0)));
        assert_eq!(v.agents.len(), 1);
        assert_eq!((v.agents[0].id, v.agents[0].start), (1, 0.0));
        let latest = w.inspect((2 * STEP) as u32, 100).unwrap();
        assert_eq!(latest.period, Some(2));
        assert!(latest.agents.iter().any(|a| a.id == 3));
        assert!(w.inspect(WIDE as u32, 0).is_err(), "no lattice, no panel");
        let l = world(|c| c.interaction = Interaction::Lattice);
        let v = l.inspect(LATTICE_X as u32 + 9, 1).unwrap();
        assert_eq!(v.lattice_site, Some(OpinionsCell { x: 1, y: 0 }));
        assert_eq!(v.agents[0].id, 2);
        assert!(
            l.inspect(WIDE as u32 + 1, 0).unwrap().agents.is_empty(),
            "the gap"
        );
        assert_eq!(Model::locate(&l, 1), None);
    }

    #[test]
    fn keyframes_restore_opinions_and_the_diagram() {
        let mut any = crate::model::ModelWorld::new(
            ModelConfig::Opinions(config(|c| c.stop_when_stable = false)),
            6,
        )
        .unwrap();
        any.model_mut().run(5);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("start", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("start", "", &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(any.model().series("clusters").unwrap().len(), 6);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.epsilon = 0.3;
            c.confidence = Confidence::OpinionDependent;
            c.updating = Updating::SerialRandom;
        });
        Model::set_config(&mut w, ModelConfig::Opinions(next.clone())).unwrap();
        w.step();
        let e = Model::set_config(
            &mut w,
            ModelConfig::Opinions(OpinionsConfig {
                agents: 624,
                ..next
            }),
        )
        .unwrap_err();
        assert_eq!(e[0].field, "agents");
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.agents = 2),
            config(|c| c.epsilon = 0.0),
            config(|c| c.epsilon = 1.0),
            config(|c| {
                c.confidence = Confidence::Asymmetric;
                c.epsilon_left = 0.0;
                c.epsilon_right = 0.0;
            }),
            config(|c| {
                c.confidence = Confidence::OpinionDependent;
                c.bias = 1.0;
                c.epsilon = 1.0;
            }),
            config(|c| {
                c.agents = 9;
                c.interaction = Interaction::Lattice;
                c.lattice.width = 3;
                c.lattice.height = 3;
                c.updating = Updating::SerialShuffled;
            }),
        ] {
            let mut w = OpinionsWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            let s = w.stats.latest().unwrap();
            assert!((0.0..=1.0).contains(&s.mean_opinion), "{c:?}");
            assert!(s.clusters >= 1);
        }
    }
}
```

Create `crates/sugarscape-core/src/opinions/presets.rs` with exactly this content:

```rust
//! The paper's runs and its two unfigured claims.

use super::config::{Confidence, Interaction, OpinionsConfig, Start, Updating};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut OpinionsConfig),
) -> ModelPreset {
    let mut c = OpinionsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source: HK,
        description,
        config: ModelConfig::Opinions(c),
    }
}

const HK: &str = "Hegselmann & Krause 2002";

fn regular(c: &mut OpinionsConfig, n: u32) {
    c.agents = n;
    c.start = Start::Regular;
}

fn asymmetric(c: &mut OpinionsConfig, left: f64, right: f64) {
    c.confidence = Confidence::Asymmetric;
    c.epsilon_left = left;
    c.epsilon_right = right;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("hk-plurality", "Little confidence: plurality", "Hegselmann and Krause's bounded confidence: 625 agents hold opinions between 0 and 1 and each period move, all at once, to the mean of every opinion within ε of their own (their own included). Lines are agents, colored by where they started (red at 0, magenta at 1); gray fills the gaps between neighbors still within reach. With ε = 0.01 (Fig. 2a) the paper's single run ends with 'exactly 38 different opinions' in under 15 periods. Measured (20 seeds): a median of 37.5 surviving opinions (34 to 43), stable by period 8 to 13.", |c| c.epsilon = 0.01),
        preset("hk-polarisation", "Middling confidence: two camps", "The same society with ε = 0.15 (Fig. 2b): the paper's run ends in two camps. Measured (20 seeds): two camps in only 6 runs; in 14 a third camp holds the middle, in 12 of them about as big as the other two (in 2 just 1 or 10 agents). Stable by period 7 to 11 in all but one run (88). Over 50 seeds (the hk-diagonal sweep) two camps are the rule only from ε = 0.16 to 0.21.", |_| {}),
        preset("hk-consensus", "Much confidence: consensus", "The same society with ε = 0.25 (Fig. 2c): consensus. Measured (20 seeds): consensus in every run, stable by period 10 (median), but some runs linger for up to 168 periods while two nearly merged camps close — the paper's 'less than 15 periods' holds for most runs, not all.", |c| c.epsilon = 0.25),
        preset("hk-regular-50", "Fifty evenly spaced, ε = 0.2", "Fifty evenly spaced opinions, ε = 0.2 (Figs. 4–5). The extremes move in first, condense, and the profile splits in two in period 6; 'from period 7 to 8 onwards nothing changes anymore'. Measured: exactly that — the split in period 6, two camps, stable at period 8. An evenly spaced start has no randomness, so every seed runs the same.", |c| {
            regular(c, 50);
            c.epsilon = 0.2;
        }),
        preset("hk-regular-plurality", "A hundred evenly spaced, ε = 0.05", "A hundred evenly spaced opinions, ε = 0.05 (Fig. 7): the paper counts 8 splits. Measured: 8 splits, 9 surviving opinions, stable at period 20; the splits open two at a time, from the ends inward.", |c| {
            regular(c, 100);
            c.epsilon = 0.05;
        }),
        preset("hk-regular-consensus", "A hundred evenly spaced, ε = 0.25", "A hundred evenly spaced opinions, ε = 0.25 (Fig. 8): 'no split, total consensus'. Measured: no split, consensus at period 10.", |c| {
            regular(c, 100);
            c.epsilon = 0.25;
        }),
        preset("hk-asym-a", "Asymmetric: 0.02 left, 0.04 right", "Asymmetric confidence the same for everyone (§4.2.1, Fig. 10a): agents listen 0.02 to the left and 0.04 to the right. The profile drifts right. Measured (20 seeds): 12 surviving opinions (median), a mean opinion of 0.54 instead of 0.50.", |c| asymmetric(c, 0.02, 0.04)),
        preset("hk-asym-b", "Asymmetric: 0.03 left, 0.15 right", "Asymmetric confidence, 0.03 left and 0.15 right (Fig. 10b): a big camp near the right border and a smaller one left of it. Measured (20 seeds): two camps of a fifth or more in 11 runs, consensus in 2, a mean opinion of 0.84; one-sided splits (a gap one side reaches across and the other does not) open and close in every run.", |c| asymmetric(c, 0.03, 0.15)),
        preset("hk-asym-c", "Asymmetric: 0.10 left, 0.25 right", "Asymmetric confidence, 0.10 left and 0.25 right (Fig. 10c). Measured (20 seeds): two camps of a fifth or more in 16 runs, consensus in 4, a mean opinion of 0.75.", |c| asymmetric(c, 0.10, 0.25)),
        preset("hk-one-sided", "One-sided splits", "A hundred evenly spaced opinions reaching 0.08 left and 0.24 right, to watch one-sided splits (Fig. 13). The caption says εl = 0.8, which would reach every opinion below and could not split anything; 0.08 is read here. Measured: a one-sided split opens in period 5 and closes in period 9 — 'contrary to two-sided splits one-sided splits can close again' — and the profile ends in consensus at 0.85.", |c| {
            regular(c, 100);
            asymmetric(c, 0.08, 0.24);
        }),
        preset("hk-bias", "Confidence leaning with opinion", "Confidence that leans with one's opinion (§4.2.2): a total reach of 0.6, split so that agents left of center look further left and those right of it further right, with bias m = 0.5 (Fig. 18c, fifty evenly spaced opinions). The paper: 'in period 4 the profile splits finally and two polarised opinion camps remain'. Measured: the split in period 4, two camps 0.50 apart, stable at period 6.", |c| {
            regular(c, 50);
            c.confidence = Confidence::OpinionDependent;
            c.epsilon = 0.6;
            c.bias = 0.5;
        }),
        preset("hk-serial", "Two camps, updated one at a time", "The two-camp society (ε = 0.15) updated one agent at a time in a fresh random order each period, each seeing the opinions already moved this period. HK §4.3: 'Random serial updating gives extreme opinions a slightly better chance to survive. But none of the results … depends crucially on simultaneous updating' — a claim with no figure, and no word on which serial order. Measured (50 seeds, hk-updating and the survey): the phases keep their places; serial updating leaves slightly more opinions at small ε.", |c| {
            c.updating = Updating::SerialShuffled;
        }),
        preset("hk-lattice", "Two camps on a lattice?", "The two-camp society on a 25 × 25 torus where each agent hears only itself and its eight neighbors — HK §4.3's 'first simulations': with small, overlapping neighborhoods 'polarization … disappears'. The torus is drawn right of the diagram. Measured (20 seeds, run to stability): a second camp holding a fifth of the agents in 7 runs at ε = 0.15 and in none at any other ε from 0.05 to 0.6; instead one big camp and dozens of stranded local minorities, settling only after thousands of periods. The claim largely holds.", |c| {
            c.interaction = Interaction::Lattice;
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, OpinionsConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Opinions(c) => (p.id, c),
                _ => panic!("{} is not a bounded-confidence preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("hk-polarisation"), OpinionsConfig::default());
        assert_eq!(find("hk-plurality").epsilon, 0.01);
        let r = find("hk-regular-50");
        assert_eq!((r.agents, r.start, r.epsilon), (50, Start::Regular, 0.2));
        let o = find("hk-one-sided");
        assert_eq!(
            (o.epsilon_left, o.epsilon_right, o.agents),
            (0.08, 0.24, 100)
        );
        let b = find("hk-bias");
        assert_eq!(
            (b.confidence, b.epsilon, b.bias),
            (Confidence::OpinionDependent, 0.6, 0.5)
        );
        assert_eq!(find("hk-serial").updating, Updating::SerialShuffled);
        assert_eq!(find("hk-lattice").interaction, Interaction::Lattice);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
```

Create `crates/sugarscape-core/src/opinions/mod.rs` with exactly this content:

```rust
//! Bounded Confidence (milestone 16): Hegselmann and Krause, "Opinion
//! Dynamics and Bounded Confidence: Models, Analysis, and Simulation" (JASSS
//! 2002), with its unfigured claims (serial updating, local neighborhoods) as
//! named switches. See docs/superpowers/specs/2026-09-25-bounded-confidence-design.md.

mod config;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Confidence, Interaction, LatticeConfig, Neighborhood, OpinionsConfig, Start, Updating,
};
pub use presets::presets;
pub use stats::{clusters, median, splits, OpinionsSnapshot, SAME, SERIES, STILL};
pub use view::{hue, opinion_at, row, GAP, HISTORY, LATTICE_X, STEP, TALL, WIDE};
pub use world::{OpinionAgent, OpinionsCell, OpinionsInspection, OpinionsMode, OpinionsWorld};
```

- [ ] **Step 2: See its tests not run yet**

Run: `cargo test -p sugarscape-core --lib opinions::`
Expected: 0 tests (the module is not in the crate).

- [ ] **Step 3: Wire it in**

Apply to `crates/sugarscape-core/src/lib.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index 767a8d3..abf24c5 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -18,6 +18,7 @@ pub mod landscape;
 mod legacy;
 pub mod model;
 pub mod network;
+pub mod opinions;
 pub mod portable;
 pub mod presets;
 pub mod render;
```

Apply to `crates/sugarscape-core/src/model.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index 063d144..884edf6 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -10,6 +10,7 @@ use crate::civil::{CivilConfig, CivilWorld};
 use crate::classes::{ClassesConfig, ClassesWorld};
 use crate::config::{Config, FieldError};
 use crate::culture::{CultureConfig, CultureWorld};
+use crate::opinions::{OpinionsConfig, OpinionsWorld};
 use crate::render::{self, ColorMode, Layer};
 use crate::ring::{RingConfig, RingWorld};
 use crate::schelling::{SchellingConfig, SchellingWorld};
@@ -17,7 +18,9 @@ use crate::schema::Param;
 use crate::spatial::{SpatialConfig, SpatialWorld};
 use crate::tags::{TagsConfig, TagsWorld};
 use crate::world::World;
-use crate::{anasazi, civil, classes, culture, export, ring, schelling, spatial, stats, tags};
+use crate::{
+    anasazi, civil, classes, culture, export, opinions, ring, schelling, spatial, stats, tags,
+};
 
 /// Which model a config or world is.
 #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
@@ -32,10 +35,11 @@ pub enum ModelKind {
     Tags,
     Culture,
     Classes,
+    Opinions,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 9] = [
+    pub const ALL: [ModelKind; 10] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -45,6 +49,7 @@ impl ModelKind {
         ModelKind::Tags,
         ModelKind::Culture,
         ModelKind::Classes,
+        ModelKind::Opinions,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -58,6 +63,7 @@ impl ModelKind {
             ModelKind::Tags => "tags",
             ModelKind::Culture => "culture",
             ModelKind::Classes => "classes",
+            ModelKind::Opinions => "opinions",
         }
     }
 
@@ -74,6 +80,7 @@ impl ModelKind {
             ModelKind::Tags => tags::schema(),
             ModelKind::Culture => culture::schema(),
             ModelKind::Classes => classes::schema(),
+            ModelKind::Opinions => opinions::schema(),
         }
     }
 }
@@ -96,6 +103,7 @@ pub enum ModelConfig {
     Tags(TagsConfig),
     Culture(CultureConfig),
     Classes(ClassesConfig),
+    Opinions(OpinionsConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -110,6 +118,7 @@ enum Tagged<'a> {
     Tags(&'a TagsConfig),
     Culture(&'a CultureConfig),
     Classes(&'a ClassesConfig),
+    Opinions(&'a OpinionsConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -131,6 +140,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Tags(c) => Tagged::Tags(c).serialize(s),
             ModelConfig::Culture(c) => Tagged::Culture(c).serialize(s),
             ModelConfig::Classes(c) => Tagged::Classes(c).serialize(s),
+            ModelConfig::Opinions(c) => Tagged::Opinions(c).serialize(s),
         }
     }
 }
@@ -147,6 +157,7 @@ impl ModelConfig {
             ModelConfig::Tags(_) => ModelKind::Tags,
             ModelConfig::Culture(_) => ModelKind::Culture,
             ModelConfig::Classes(_) => ModelKind::Classes,
+            ModelConfig::Opinions(_) => ModelKind::Opinions,
         }
     }
 
@@ -208,6 +219,9 @@ impl ModelConfig {
             "classes" => serde_json::from_value(value)
                 .map(ModelConfig::Classes)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "opinions" => serde_json::from_value(value)
+                .map(ModelConfig::Opinions)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
@@ -228,6 +242,7 @@ impl ModelConfig {
             ModelConfig::Tags(c) => c.validate(),
             ModelConfig::Culture(c) => c.validate(),
             ModelConfig::Classes(c) => c.validate(),
+            ModelConfig::Opinions(c) => c.validate(),
         }
     }
 
@@ -244,6 +259,7 @@ impl ModelConfig {
             ModelConfig::Tags(c) => set_path(c, path, value).map(ModelConfig::Tags),
             ModelConfig::Culture(c) => set_path(c, path, value).map(ModelConfig::Culture),
             ModelConfig::Classes(c) => set_path(c, path, value).map(ModelConfig::Classes),
+            ModelConfig::Opinions(c) => set_path(c, path, value).map(ModelConfig::Opinions),
         }
     }
 
@@ -259,7 +275,8 @@ impl ModelConfig {
             | ModelConfig::Civil(_)
             | ModelConfig::Spatial(_)
             | ModelConfig::Culture(_)
-            | ModelConfig::Classes(_) => None,
+            | ModelConfig::Classes(_)
+            | ModelConfig::Opinions(_) => None,
         }
     }
 
@@ -275,6 +292,7 @@ impl ModelConfig {
             ModelConfig::Tags(_) => tags::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Culture(_) => culture::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Classes(_) => classes::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Opinions(_) => opinions::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -453,6 +471,7 @@ pub enum ModelWorld {
     Tags(Box<TagsWorld>),
     Culture(Box<CultureWorld>),
     Classes(Box<ClassesWorld>),
+    Opinions(Box<OpinionsWorld>),
 }
 
 impl ModelWorld {
@@ -481,6 +500,9 @@ impl ModelWorld {
             ModelConfig::Tags(c) => ModelWorld::Tags(Box::new(TagsWorld::new(c, seed)?)),
             ModelConfig::Culture(c) => ModelWorld::Culture(Box::new(CultureWorld::new(c, seed)?)),
             ModelConfig::Classes(c) => ModelWorld::Classes(Box::new(ClassesWorld::new(c, seed)?)),
+            ModelConfig::Opinions(c) => {
+                ModelWorld::Opinions(Box::new(OpinionsWorld::new(c, seed)?))
+            }
         })
     }
 
@@ -495,6 +517,7 @@ impl ModelWorld {
             ModelWorld::Tags(_) => ModelKind::Tags,
             ModelWorld::Culture(_) => ModelKind::Culture,
             ModelWorld::Classes(_) => ModelKind::Classes,
+            ModelWorld::Opinions(_) => ModelKind::Opinions,
         }
     }
 
@@ -509,6 +532,7 @@ impl ModelWorld {
             ModelWorld::Tags(w) => w.as_ref(),
             ModelWorld::Culture(w) => w.as_ref(),
             ModelWorld::Classes(w) => w.as_ref(),
+            ModelWorld::Opinions(w) => w.as_ref(),
         }
     }
 
@@ -523,6 +547,7 @@ impl ModelWorld {
             ModelWorld::Tags(w) => w.as_mut(),
             ModelWorld::Culture(w) => w.as_mut(),
             ModelWorld::Classes(w) => w.as_mut(),
+            ModelWorld::Opinions(w) => w.as_mut(),
         }
     }
 
@@ -605,6 +630,7 @@ impl ModelWorld {
             ModelWorld::Tags(w) => copy_without_history!(Tags, w),
             ModelWorld::Culture(w) => copy_without_history!(Culture, w),
             ModelWorld::Classes(w) => copy_without_history!(Classes, w),
+            ModelWorld::Opinions(w) => copy_without_history!(Opinions, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -629,6 +655,7 @@ impl ModelWorld {
             (ModelWorld::Tags(live), ModelWorld::Tags(kept)) => restore_into!(live, kept),
             (ModelWorld::Culture(live), ModelWorld::Culture(kept)) => restore_into!(live, kept),
             (ModelWorld::Classes(live), ModelWorld::Classes(kept)) => restore_into!(live, kept),
+            (ModelWorld::Opinions(live), ModelWorld::Opinions(kept)) => restore_into!(live, kept),
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -885,7 +912,8 @@ mod tests {
                 "spatial",
                 "tags",
                 "culture",
-                "classes"
+                "classes",
+                "opinions"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
```

Apply to `crates/sugarscape-core/src/presets.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index 4178d33..d06f338 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -682,6 +682,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::tags::presets());
     out.extend(crate::culture::presets());
     out.extend(crate::classes::presets());
+    out.extend(crate::opinions::presets());
     out.extend(crate::spatial::presets());
     out
 }
```

Run: `cargo test -p sugarscape-core --lib opinions::`
Expected: PASS (23 tests).

- [ ] **Step 4: Golden entries (fail first)**

Run: `cargo test -p sugarscape-core --release --test golden`
Expected: FAIL — `every_model_preset_has_a_golden_entry` asks for `hk-plurality`.

Apply to `crates/sugarscape-core/tests/golden.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 54588a7..0514ef6 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -114,6 +114,19 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("pvplh-mode", 0xb90f0d0cab7966b9),
     ("pvplh-progressive", 0xca60c420d61a7ffc),
     ("pvplh-lattice", 0x7975f5afc2d06c10),
+    ("hk-plurality", 0x587fb8e7c2480e66),
+    ("hk-polarisation", 0x204ac33894adc63e),
+    ("hk-consensus", 0x7d014897a7133c02),
+    ("hk-regular-50", 0xbfd3a4ebe6714f04),
+    ("hk-regular-plurality", 0x8336bae6d0f6a7e9),
+    ("hk-regular-consensus", 0xa52300371b7c72af),
+    ("hk-asym-a", 0x9cde60c378eb0e27),
+    ("hk-asym-b", 0x5b3285bbf1750490),
+    ("hk-asym-c", 0x67b217d709a3f7d3),
+    ("hk-one-sided", 0x84fba5285f2ec536),
+    ("hk-bias", 0x9a2ed969514cbda2),
+    ("hk-serial", 0xb06db73333504889),
+    ("hk-lattice", 0xe33359f120b204d9),
 ];
 
 fn fingerprint(id: &str) -> u64 {
```

Run it again. Expected: PASS.

- [ ] **Step 5: Format, lint, test, commit**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test -p sugarscape-core`

```bash
git add crates/sugarscape-core/src/opinions/ crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs
git commit -m "Add Bounded Confidence (Hegselmann & Krause) as a model kind" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/hk-diagonal.json`, `sweeps/hk-asymmetry.json`, `sweeps/hk-bias.json`, `sweeps/hk-updating.json`, `sweeps/hk-lattice.json`, `sweeps/hk-population.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: presets and series from Task 1; `holds_when_finished` (true: a stable profile is read at its last values).
- Produces: the six built-in sweeps.

- [ ] **Step 1: The sweeps** (descriptions record what planning measured with exactly these files)

Create `sweeps/hk-diagonal.json` with exactly this content:

```json
{
  "name": "Bounded Confidence: walking the diagonal (Fig. 3)",
  "description": "Hegselmann and Krause's Fig. 3: surviving opinions after the dynamics stabilize, 625 random opinions, simultaneous updating, ε = 0.01 to 0.40, 50 runs each. They report plurality, then polarization, then — 'at about step 25' — a sudden consensus, and consensus always above 0.4. Measured (release, seeds 1–50, recorded 2026-09-25): 37.7 opinions at 0.01, 7.9 at 0.05, 3.6 at 0.10, 2.7 at 0.15 (three camps more often than two), 1.9 at 0.20; consensus in 30 of 50 runs at 0.22, 48 at 0.24 and every run from 0.25. The phases and the consensus threshold reproduce.",
  "base": {
    "preset": "hk-polarisation"
  },
  "x": {
    "label": "Confidence (ε)",
    "path": "epsilon",
    "values": [
      0.01,
      0.02,
      0.03,
      0.04,
      0.05,
      0.06,
      0.07,
      0.08,
      0.09,
      0.1,
      0.11,
      0.12,
      0.13,
      0.14,
      0.15,
      0.16,
      0.17,
      0.18,
      0.19,
      0.2,
      0.21,
      0.22,
      0.23,
      0.24,
      0.25,
      0.26,
      0.27,
      0.28,
      0.29,
      0.3,
      0.31,
      0.32,
      0.33,
      0.34,
      0.35,
      0.36,
      0.37,
      0.38,
      0.39,
      0.4
    ]
  },
  "seeds": {
    "from": 1,
    "count": 50
  },
  "ticks": 1000,
  "metric": {
    "kind": "final",
    "series": "clusters"
  }
}
```

Create `sweeps/hk-asymmetry.json` with exactly this content:

```json
{
  "name": "Bounded Confidence: asymmetric confidence and the mean (Fig. 12c)",
  "description": "Hegselmann and Krause's Fig. 12c: the final mean opinion when agents reach εl to the left and εr to the right, the same for everyone. They report that the mean moves toward the favored side, most when confidence in the other direction is small. Measured (release, seeds 1–20, recorded 2026-09-25): with εl = 0.02 the mean climbs from 0.50 to 0.94 as εr grows to 0.2 and stays there; with εl = 0.1 from 0.26 (εr = 0.02: favoring the left) to 0.81; with εl = 0.2 from 0.06 to 0.69. Symmetric points (εl = εr) sit at 0.50. It reproduces.",
  "base": {
    "preset": "hk-asym-c"
  },
  "x": {
    "label": "Right reach (εr)",
    "path": "epsilon_right",
    "values": [
      0.02,
      0.04,
      0.06,
      0.08,
      0.1,
      0.12,
      0.14,
      0.16,
      0.18,
      0.2,
      0.22,
      0.24,
      0.26,
      0.28,
      0.3,
      0.32,
      0.34,
      0.36,
      0.38,
      0.4
    ]
  },
  "series": {
    "label": "Left reach",
    "values": [
      {
        "at": 0,
        "name": "εl = 0.02",
        "set": {
          "epsilon_left": 0.02
        }
      },
      {
        "at": 1,
        "name": "εl = 0.1",
        "set": {
          "epsilon_left": 0.1
        }
      },
      {
        "at": 2,
        "name": "εl = 0.2",
        "set": {
          "epsilon_left": 0.2
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 1000,
  "metric": {
    "kind": "final",
    "series": "mean_opinion"
  }
}
```

Create `sweeps/hk-bias.json` with exactly this content:

```json
{
  "name": "Bounded Confidence: a bias leaning with opinion (Fig. 17)",
  "description": "Hegselmann and Krause's Fig. 17: confidence of total ε that leans with one's opinion, the bias m from 0 to 1 in 26 steps; the metric is the final range of opinions (0 at consensus, 1 with camps at both ends). They report sharper polarization with m, camps at 0 and 1 at m = 1, and at ε = 0.6 a consensus that breaks near m ≈ 0.4. Measured (release, 625 random opinions, seeds 1–20, recorded 2026-09-25): at ε = 0.6 the range is 0 up to m = 0.40 and grows from 0.44 (0.05) to 0.51 at 0.60 and 0.99 at 1; at ε = 0.2 and 0.4 it grows steadily with m to 0.99. It reproduces.",
  "base": {
    "preset": "hk-bias"
  },
  "x": {
    "label": "Bias (m)",
    "path": "bias",
    "values": [
      0.0,
      0.04,
      0.08,
      0.12,
      0.16,
      0.2,
      0.24,
      0.28,
      0.32,
      0.36,
      0.4,
      0.44,
      0.48,
      0.52,
      0.56,
      0.6,
      0.64,
      0.68,
      0.72,
      0.76,
      0.8,
      0.84,
      0.88,
      0.92,
      0.96,
      1.0
    ]
  },
  "series": {
    "label": "Confidence",
    "values": [
      {
        "at": 0,
        "name": "ε = 0.2",
        "set": {
          "epsilon": 0.2,
          "start": "random",
          "agents": 625
        }
      },
      {
        "at": 1,
        "name": "ε = 0.4",
        "set": {
          "epsilon": 0.4,
          "start": "random",
          "agents": 625
        }
      },
      {
        "at": 2,
        "name": "ε = 0.6",
        "set": {
          "epsilon": 0.6,
          "start": "random",
          "agents": 625
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 1000,
  "metric": {
    "kind": "final",
    "series": "range"
  }
}
```

Create `sweeps/hk-updating.json` with exactly this content:

```json
{
  "name": "Bounded Confidence: simultaneous or serial updating (§4.3)",
  "description": "HK §4.3: 'none of the results … depends crucially on simultaneous updating' — reported without a figure or the serial order. The metric is the surviving opinions, ε = 0.05 to 0.30, under simultaneous updating, each agent once per period in random order, and n random draws per period. Measured (release, seeds 1–10, recorded 2026-09-25): 7.6, 8.4 and 9.0 opinions at 0.05; 2.9, 2.7 and 3.0 at 0.15; consensus at 0.25 and 0.30 under every order (random draws leave a straggler in some runs: 1.2). Serial updating keeps slightly more opinions at small ε, as HK say; the phases keep their places.",
  "base": {
    "preset": "hk-polarisation"
  },
  "x": {
    "label": "Confidence (ε)",
    "path": "epsilon",
    "values": [
      0.05,
      0.1,
      0.15,
      0.2,
      0.25,
      0.3
    ]
  },
  "series": {
    "label": "Updating",
    "values": [
      {
        "at": 0,
        "name": "Simultaneous (HK)",
        "set": {
          "updating": "simultaneous"
        }
      },
      {
        "at": 1,
        "name": "Serial, shuffled",
        "set": {
          "updating": "serial_shuffled"
        }
      },
      {
        "at": 2,
        "name": "Serial, random draws",
        "set": {
          "updating": "serial_random"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 1000,
  "metric": {
    "kind": "final",
    "series": "clusters"
  }
}
```

Create `sweeps/hk-lattice.json` with exactly this content:

```json
{
  "name": "Bounded Confidence: does locality end polarization? (§4.3)",
  "description": "HK §4.3's unfigured claim: with small, overlapping neighborhoods, 'polarization … disappears'. The metric is the share of agents in the second-largest camp after 2000 periods (0 at consensus, near 0.5 with two equal camps), for everyone, a Moore torus and a von Neumann torus (25 × 25). Measured (release, seeds 1–5, recorded 2026-09-25): everyone listening, 0.31, 0.40 and 0.46 at ε = 0.10, 0.15 and 0.20, then 0; on either lattice never above 0.06 — one big camp and many small stranded minorities instead of two camps. The claim holds (run to stability, the survey finds a second camp of a fifth in 7 of 20 Moore runs at ε = 0.15 and nowhere else).",
  "base": {
    "preset": "hk-polarisation"
  },
  "x": {
    "label": "Confidence (ε)",
    "path": "epsilon",
    "values": [
      0.1,
      0.15,
      0.2,
      0.25,
      0.3,
      0.4,
      0.6
    ]
  },
  "series": {
    "label": "Who listens",
    "values": [
      {
        "at": 0,
        "name": "Everyone (HK)",
        "set": {
          "interaction": "all"
        }
      },
      {
        "at": 1,
        "name": "Moore lattice",
        "set": {
          "interaction": "lattice",
          "lattice": {
            "width": 25,
            "height": 25,
            "neighborhood": "moore"
          }
        }
      },
      {
        "at": 2,
        "name": "von Neumann lattice",
        "set": {
          "interaction": "lattice",
          "lattice": {
            "width": 25,
            "height": 25,
            "neighborhood": "von_neumann"
          }
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 5
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "second"
  }
}
```

Create `sweeps/hk-population.json` with exactly this content:

```json
{
  "name": "Bounded Confidence: consensus and the number of agents (Lorenz 2006)",
  "description": "Lorenz 2006 ('Consensus strikes back'): the confidence at which consensus becomes typical depends on the number of agents. The metric is the share of agents in the largest camp (1 at consensus). Measured (release, seeds 1–20, recorded 2026-09-25): at ε = 0.25, 0.73 with 25 agents, 0.89 with 200 and 1 from 625 up; at ε = 0.22, about 0.62 up to 200 agents and 0.82–0.91 from 625 — small groups miss the consensus that HK's 625 agents reach.",
  "base": {
    "preset": "hk-polarisation"
  },
  "x": {
    "label": "Agents (n)",
    "path": "agents",
    "values": [
      25,
      50,
      100,
      200,
      625,
      1000,
      2000
    ]
  },
  "series": {
    "label": "Confidence",
    "values": [
      {
        "at": 0,
        "name": "ε = 0.2",
        "set": {
          "epsilon": 0.2
        }
      },
      {
        "at": 1,
        "name": "ε = 0.22",
        "set": {
          "epsilon": 0.22
        }
      },
      {
        "at": 2,
        "name": "ε = 0.25",
        "set": {
          "epsilon": 0.25
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 1000,
  "metric": {
    "kind": "final",
    "series": "largest"
  }
}
```

Apply to `crates/sugarscape-core/src/sweep.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index f546784..5e1e843 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -954,7 +954,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 32] = [
+const BUILTINS: [Builtin; 38] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1083,6 +1083,30 @@ const BUILTINS: [Builtin; 32] = [
         id: "pvplh-payoffs",
         json: include_str!("../../../sweeps/pvplh-payoffs.json"),
     },
+    Builtin {
+        id: "hk-diagonal",
+        json: include_str!("../../../sweeps/hk-diagonal.json"),
+    },
+    Builtin {
+        id: "hk-asymmetry",
+        json: include_str!("../../../sweeps/hk-asymmetry.json"),
+    },
+    Builtin {
+        id: "hk-bias",
+        json: include_str!("../../../sweeps/hk-bias.json"),
+    },
+    Builtin {
+        id: "hk-updating",
+        json: include_str!("../../../sweeps/hk-updating.json"),
+    },
+    Builtin {
+        id: "hk-lattice",
+        json: include_str!("../../../sweeps/hk-lattice.json"),
+    },
+    Builtin {
+        id: "hk-population",
+        json: include_str!("../../../sweeps/hk-population.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -1927,7 +1951,13 @@ mod tests {
                 "aey-population",
                 "aey-first-attractor",
                 "aey-tag-regimes",
-                "pvplh-payoffs"
+                "pvplh-payoffs",
+                "hk-diagonal",
+                "hk-asymmetry",
+                "hk-bias",
+                "hk-updating",
+                "hk-lattice",
+                "hk-population"
             ]
         );
         for b in builtins() {
```

Run: `cargo test -p sugarscape-core --lib sweep::` — Expected: PASS.

- [ ] **Step 2: Measure natively**

Run: `cargo build --release -p sugarscape-cli && for s in hk-diagonal hk-asymmetry hk-bias hk-updating hk-lattice hk-population; do /usr/bin/time -p target/release/sugarscape sweep --builtin $s --quiet --out /dev/null --summary-csv /tmp/$s.csv; done`
Expected: each under a few seconds; means as the descriptions state.

- [ ] **Step 3: The CLI names the stop (test first)**

Apply to `crates/sugarscape-cli/tests/cli.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index 8c365e5..7e8ce70 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -80,6 +80,12 @@ fn presets_and_sweeps_are_listed() {
         "aey-first-attractor",
         "aey-tag-regimes",
         "pvplh-payoffs",
+        "hk-diagonal",
+        "hk-asymmetry",
+        "hk-bias",
+        "hk-updating",
+        "hk-lattice",
+        "hk-population",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -434,6 +440,13 @@ fn a_classes_run_stops_at_equity() {
     );
 }
 
+#[test]
+fn a_bounded_confidence_run_stops_when_stable() {
+    let out = sugarscape(&["run", "--preset", "hk-regular-50", "--ticks", "1000"]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    assert_eq!(stderr(&out), "finished at tick 8 (stable)\n");
+}
+
 #[test]
 fn a_culture_run_stops_when_the_lattice_is_stable() {
     let out = sugarscape(&["run", "--preset", "ac-sample-run", "--ticks", "100000"]);
```

Run: `cargo test -p sugarscape-cli a_bounded_confidence_run` — Expected: FAIL (`(its end year)`).

Apply to `crates/sugarscape-cli/src/main.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index 8568d87..51ec4e8 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -180,13 +180,15 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
     let world = world.model();
     if world.finished() && world.tick() < u64::from(args.ticks) {
         // The anasazi stops at its end year; civil violence when a group is gone;
-        // the tags model at its last generation; Axelrod's culture once stable,
+        // the tags model at its last generation; Axelrod's culture and bounded
+        // confidence once stable,
         // and a sugarscape under his rule once its cultures settle.
         let why = match config.kind() {
             ModelKind::Civil => "a group has died out",
             ModelKind::Tags => "its last generation",
             ModelKind::Culture => "the lattice is stable",
             ModelKind::Classes => "equity reached",
+            ModelKind::Opinions => "stable",
             ModelKind::Sugarscape => "the cultures have settled",
             _ => "its end year",
         };
```

Run: `cargo test -p sugarscape-cli` — Expected: PASS.

- [ ] **Step 4: WASM agrees**

Apply to `crates/sugarscape-wasm/tests/web.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index 5dc38dd..c4f242a 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -282,7 +282,13 @@ fn builtins_and_series_names_are_listed() {
             "aey-population",
             "aey-first-attractor",
             "aey-tag-regimes",
-            "pvplh-payoffs"
+            "pvplh-payoffs",
+            "hk-diagonal",
+            "hk-asymmetry",
+            "hk-bias",
+            "hk-updating",
+            "hk-lattice",
+            "hk-population"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -711,6 +717,21 @@ fn classes_sims_match_the_native_golden_entries() {
     }
 }
 
+#[wasm_bindgen_test]
+fn opinions_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
+    for (id, fp) in [
+        ("hk-polarisation", "0x204ac33894adc63e"),
+        ("hk-serial", "0xb06db73333504889"),
+        ("hk-lattice", "0xe33359f120b204d9"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "opinions");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
 #[wasm_bindgen_test]
 fn culture_sims_match_the_native_golden_entries() {
     // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN.
```

Run: `wasm-pack test --node crates/sugarscape-wasm` — Expected: PASS (37), including `opinions_sims_match_the_native_golden_entries`.

- [ ] **Step 5: Commit**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace`

```bash
git add sweeps/hk-diagonal.json sweeps/hk-asymmetry.json sweeps/hk-bias.json sweeps/hk-updating.json sweeps/hk-lattice.json sweeps/hk-population.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/src/main.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure Bounded Confidence: six sweeps, the CLI's stop and WASM agreement" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/ui/series-data.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/compare-presets.test.ts`, `web/src/experiments/form.test.ts`, `web/src/engine.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the core's JSON (`OpinionsConfig`, `OpinionsSnapshot`, `OpinionsInspection` with `agent: null`).
- Produces: `OpinionsConfig`, `OpinionsStats`, `OpinionsInspection`, `OpinionAgent` (types.ts); `isOpinionsView` (models.ts); the modes, charts, Inspect rows, Compare entry and Experiments default.

- [ ] **Step 1: The failing tests**

Apply to `web/src/models.test.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index d61523a..56b35a6 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -5,6 +5,7 @@ import {
   finishesUnpredictably,
   isCivilView,
   isClassesView,
+  isOpinionsView,
   isCultureView,
   isRingView,
   isSpatialView,
@@ -99,6 +100,28 @@ describe('the anasazi model', () => {
   });
 });
 
+describe('the bounded-confidence model', () => {
+  it('is read by its tag, and its inspections by their period and lattice site', () => {
+    const c = { model: 'opinions', stop_when_stable: true } as unknown as ModelConfig;
+    expect(modelOf(c)).toBe('opinions');
+    const cell = { site: { x: 1, y: 2 }, period: 3, opinion: 0.5, lattice_site: null, agents: [], agent: null } as unknown as AnyInspection;
+    const simplex = { site: { x: 1, y: 2 }, simplex: 'one', mix: [0.2, 0.5, 0.3], best_reply: 'M', agents: [], agent: null } as unknown as AnyInspection;
+    expect([cell, simplex].map(isOpinionsView)).toEqual([true, false]);
+    expect(isClassesView(cell) || isCultureView(cell) || isTagsView(cell) || isSugarView(cell)).toBe(false);
+  });
+
+  it('colors lines by start or opinion, has no overlays, and stops unpredictably when asked to stop at stability', () => {
+    expect(COLOR_MODES.opinions).toEqual([
+      ['start', 'Start'],
+      ['opinion', 'Opinion'],
+    ]);
+    expect(MODEL_OVERLAYS.opinions).toEqual([]);
+    const c = (stop_when_stable: boolean) => ({ model: 'opinions', stop_when_stable }) as unknown as ModelConfig;
+    expect([finishesUnpredictably(c(true)), finishesUnpredictably(c(false))]).toEqual([true, false]);
+    expect(ticksLeft(c(true), 5)).toBe(Infinity);
+  });
+});
+
 describe('the classes model', () => {
   it('is read by its tag, and its inspections by their simplex and mix', () => {
     const c = { model: 'classes', stop_at_equity: false } as unknown as ModelConfig;
```

Apply to `web/src/ui/series-data.test.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index 4351e27..09ab88a 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,13 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('bounded-confidence charts', () => {
+  it('chart clusters, camps, the center, splits and change over periods', () => {
+    expect(MODEL_CHARTS.opinions.map((c) => c.title)).toEqual(['Clusters', 'Largest camps', 'Mean and median', 'Splits', 'Change']);
+    expect(timeAxisLabel('opinions')).toBe('Periods');
+  });
+});
+
 describe('classes charts', () => {
   it('chart payoffs, outcomes, memory and the regime, and per-tag payoffs only with tags', () => {
     expect(MODEL_CHARTS.classes.map((c) => c.title)).toEqual(['Mean payoff', 'Outcomes', 'M in memory', 'Regime', 'Payoffs by tag', 'Inter-type advantage']);
```

Apply to `web/src/compare-presets.test.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index e5133a9..c52c2f6 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -42,6 +42,11 @@ describe('compare presets', () => {
     expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
   });
 
+  it('pairs simultaneous and serial updating', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['hk-simultaneous-vs-serial', 'hk-polarisation', 'hk-serial', 'Simultaneous vs serial updating — Bounded Confidence (Compare)']);
+  });
+
   it('pairs AEY’s rule and the mode rule with tags', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['aey-tags-vs-mode', 'aey-tags', 'pvplh-mode', 'AEY’s rule vs the mode rule, with tags — Emergence of Classes (Compare)']);
```

Apply to `web/src/experiments/form.test.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index ab0cf0c..72a3873 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -138,6 +138,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('opinions')).toMatchObject({
+      x: { path: 'epsilon', values: '0.05:0.3:0.05' },
+      ticks: 1000,
+      metric: { kind: 'final', series: 'clusters' },
+    });
     expect(defaultForm('classes')).toMatchObject({
       x: { path: 'memory', values: '6:14:2' },
       ticks: 100000,
```

Apply to `web/src/engine.test.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index e501531..a24ae76 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1150,6 +1150,7 @@ describe('Engine with other models', () => {
     expect(finishedNotice(e.config, 10)).toBe('This run has reached its end year (AD 810) — Reset to run it again');
     expect(finishedNotice(ring, 10)).toBe('This run has reached its end year — Reset to run it again');
     expect(finishedNotice({ model: 'civil' } as unknown as ModelConfig, 94)).toBe('A group has died out at t = 94 — Reset to run it again');
+    expect(finishedNotice({ model: 'opinions' } as unknown as ModelConfig, 8)).toBe('Stable at t = 8: no opinion moves any more — Reset to run it again');
     expect(finishedNotice({ model: 'classes' } as unknown as ModelConfig, 640)).toBe(
       'Equity reached at t = 640: every agent remembers mostly M — Reset to run it again',
     );
```

Apply to `web/src/determinism.test.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index 01c27b4..ec334ae 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -8,7 +8,7 @@ import { SimHost } from './sim-host';
 import { wasmSimModule } from './sim-module';
 import { InlineTransport } from './transport';
 import { decodeShare, encodeShare } from './share';
-import type { AnasaziStats, CivilConfig, CivilStats, ClassesConfig, ClassesInspection, ClassesStats, CultureInspection, CultureStats, Preset, Snapshot, TagsConfig, TagsInspection, TagsStats } from './types';
+import type { AnasaziStats, CivilConfig, CivilStats, ClassesConfig, ClassesInspection, ClassesStats, CultureInspection, CultureStats, OpinionsConfig, OpinionsInspection, OpinionsStats, Preset, Snapshot, TagsConfig, TagsInspection, TagsStats } from './types';
 import { MODEL_CHARTS } from './ui/series-data';
 import { config_series_names, initSync, presets_json, run_point, sweep_points } from './wasm-pkg/sugarscape.js';
 
@@ -390,6 +390,7 @@ describe('other models through the engine', () => {
     ['pvplh-mode', '0xb90f0d0cab7966b9'],
     ['pvplh-progressive', '0xca60c420d61a7ffc'],
     ['pvplh-lattice', '0x7975f5afc2d06c10'],
+    ['hk-lattice', '0xe33359f120b204d9'],
   ];
 
   it.each(GOLDEN_MODELS)('%s reproduces its golden fingerprint, whatever is watched', async (id, golden) => {
@@ -471,6 +472,26 @@ describe('the anasazi through the engine', () => {
   });
 });
 
+describe('the bounded-confidence model through the engine', () => {
+  it('stops once when stable, matches the native golden entry and inspects a line', async () => {
+    const r = presets.find((p) => p.id === 'hk-regular-50')!;
+    const e = await Engine.create({ config: structuredClone(r.config as OpinionsConfig), seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'opinion' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as OpinionsStats;
+    expect([e.finished, ends, e.tick, s.stable_at, s.clusters]).toEqual([true, 1, 8, 8, 2]);
+    // crates/sugarscape-core/tests/golden.rs: hk-regular-50 stops at period 8 of its 200.
+    expect(await e.fingerprint()).toBe('0xbfd3a4ebe6714f04');
+    // Agent 1 starts at 0: the diagram's bottom row, in the column of the oldest kept period.
+    await e.select(0, 200);
+    const v = e.inspection!.view as OpinionsInspection;
+    expect([v.period, v.opinion, v.agents[0].id, v.agents[0].start]).toEqual([0, 0, 1, 0]);
+    expect(e.inspection!.agentId).toBeNull();
+  });
+});
+
 describe('the classes model through the engine', () => {
   it('stops once at equity and inspects a simplex point', async () => {
     const t = presets.find((p) => p.id === 'aey-transition')!;
```

Run: `(cd web && npm run build)` — Expected: FAIL in `tsc` (`'opinions'` is not a `ModelKind`; `isOpinionsView`, `OpinionsStats` missing).

- [ ] **Step 2: Implement**

Apply to `web/src/types.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/types.ts b/web/src/types.ts
index dd6ec9c..4b729f3 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -81,7 +81,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'opinions';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -262,7 +262,27 @@ export interface ClassesConfig {
   stop_at_equity: boolean;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig;
+/**
+ * Hegselmann and Krause's bounded confidence (milestone 16): agents move to the mean of the opinions
+ * within their reach, with the paper's asymmetric and opinion-dependent confidence and its unfigured
+ * claims (serial updating, lattice neighborhoods) as switches.
+ */
+export interface OpinionsConfig {
+  model: 'opinions';
+  agents: number;
+  start: 'random' | 'regular';
+  confidence: 'symmetric' | 'asymmetric' | 'opinion_dependent';
+  epsilon: number;
+  epsilon_left: number;
+  epsilon_right: number;
+  bias: number;
+  updating: 'simultaneous' | 'serial_shuffled' | 'serial_random';
+  interaction: 'all' | 'lattice';
+  lattice: { width: number; height: number; neighborhood: 'moore' | 'von_neumann' };
+  stop_when_stable: boolean;
+}
+
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | OpinionsConfig;
 
 export interface Preset { id: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -431,7 +451,22 @@ export interface ClassesStats {
   realized_noise: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats;
+export interface OpinionsStats {
+  tick: number;
+  /** Surviving opinions: groups of opinions within 10⁻⁶ of each other. */
+  clusters: number;
+  largest: number;
+  second: number;
+  mean_opinion: number;
+  median_opinion: number;
+  range: number;
+  splits: number;
+  one_sided_splits: number;
+  max_change: number;
+  stable_at: number;
+}
+
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | OpinionsStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -600,7 +635,22 @@ export interface ClassesInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection;
+/** An agent whose line passes an inspected point, or a lattice site's agent. */
+export interface OpinionAgent { id: number; start: number; opinion: number; epsilon_left: number; epsilon_right: number; reaches: number }
+/**
+ * A cell of the opinion × time diagram (its period and opinion, and the agents whose lines pass
+ * within a cell) or of the lattice to its right. `agent` is always null: agents have no place to follow.
+ */
+export interface OpinionsInspection {
+  site: { x: number; y: number };
+  period: number | null;
+  opinion: number | null;
+  lattice_site: { x: number; y: number } | null;
+  agents: OpinionAgent[];
+  agent: null;
+}
+
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | OpinionsInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -635,7 +685,9 @@ export type ColorMode =
   | 'similarity'
   | 'zones'
   | 'best_reply'
-  | 'payoff';
+  | 'payoff'
+  | 'start'
+  | 'opinion';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
```

Apply to `web/src/models.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/models.ts b/web/src/models.ts
index 037176e..1acca77 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -9,6 +9,8 @@ import type {
   ClassesInspection,
   CultureConfig,
   CultureInspection,
+  OpinionsConfig,
+  OpinionsInspection,
   ColorMode,
   Config,
   Inspection,
@@ -21,7 +23,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'opinions'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -34,12 +36,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   tags: 'Tag Cooperation',
   culture: 'Axelrod Culture',
   classes: 'Emergence of Classes',
+  opinions: 'Bounded Confidence',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'opinions'
     ? tag
     : 'sugarscape';
 }
@@ -83,6 +86,11 @@ export function isClassesView(v: AnyInspection): v is ClassesInspection {
   return 'simplex' in v && 'mix' in v;
 }
 
+/** A cell of the bounded-confidence frame (it names its period and lattice site). */
+export function isOpinionsView(v: AnyInspection): v is OpinionsInspection {
+  return 'period' in v && 'lattice_site' in v;
+}
+
 /** A cell of the culture frame (it says whether it is a site or a lane). */
 export function isCultureView(v: AnyInspection): v is CultureInspection {
   return 'kind' in v && 'neighbors' in v;
@@ -112,6 +120,7 @@ export function finishesUnpredictably(c: ModelConfig): boolean {
   const model = modelOf(c);
   if (model === 'culture') return (c as CultureConfig).stop_when_stable && (c as CultureConfig).drift === 0;
   if (model === 'classes') return (c as ClassesConfig).stop_at_equity;
+  if (model === 'opinions') return (c as OpinionsConfig).stop_when_stable;
   if (model === 'sugarscape') return (c as Config).culture.rule === 'axelrod' && (c as Config).culture.stop_when_settled === true;
   return model === 'civil' && (c as CivilConfig).variant === 'ethnic' && (c as CivilConfig).stop_at_extinction;
 }
@@ -183,6 +192,11 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['best_reply', 'Best reply'],
     ['payoff', 'Payoff'],
   ],
+  // Lines colored by where each agent started (HK's figures) or where it is now.
+  opinions: [
+    ['start', 'Start'],
+    ['opinion', 'Opinion'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -196,4 +210,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   tags: [],
   culture: [],
   classes: [],
+  opinions: [],
 };
```

Apply to `web/src/engine.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/engine.ts b/web/src/engine.ts
index a1e3ee9..5f724b2 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -54,6 +54,7 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
   if (modelOf(config) === 'tags') return `This run has reached its last generation (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'classes') return `Equity reached at t = ${tick}: every agent remembers mostly M — Reset to run it again`;
+  if (modelOf(config) === 'opinions') return `Stable at t = ${tick}: no opinion moves any more — Reset to run it again`;
   if (modelOf(config) === 'culture') return `The lattice is stable at t = ${tick}: no two neighbors can interact — Reset to run it again`;
   if (modelOf(config) === 'sugarscape') return `The cultures have settled at t = ${tick}: every two share all or nothing — Reset to run it again`;
   const year = calendarYear(config, tick);
```

Apply to `web/src/ui/series-data.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index ab67b20..cf5b5e3 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -269,11 +269,38 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
     },
     { title: 'Inter-type advantage', lines: [{ key: 'payoff_inter', label: 'Dark minus light, against each other', color: '--c3' }], shown: hasTags },
   ],
+  opinions: [
+    { title: 'Clusters', lines: [{ key: 'clusters', label: 'Surviving opinions', color: '--c1' }] },
+    {
+      title: 'Largest camps',
+      lines: [
+        { key: 'largest', label: 'Largest', color: '--c2' },
+        { key: 'second', label: 'Second', color: '--c3' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'Mean and median',
+      lines: [
+        { key: 'mean_opinion', label: 'Mean', color: '--c1' },
+        { key: 'median_opinion', label: 'Median', color: '--c4' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'Splits',
+      lines: [
+        { key: 'splits', label: 'Two-sided', color: '--red' },
+        { key: 'one_sided_splits', label: 'One-sided', color: '--c3' },
+      ],
+    },
+    { title: 'Change', lines: [{ key: 'max_change', label: 'Largest move this period', color: '--c4' }] },
+  ],
 };
 
 /** A model's time charts count calendar years (the anasazi's) or ticks. */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' ? 'Periods' : 'Tick';
+  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' ? 'Periods' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
```

Apply to `web/src/compare-presets.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index a2eb1e6..8486659 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -62,6 +62,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'aey-tags',
     b: 'pvplh-mode',
   },
+  {
+    id: 'hk-simultaneous-vs-serial',
+    label: 'Simultaneous vs serial updating — Bounded Confidence (Compare)',
+    a: 'hk-polarisation',
+    b: 'hk-serial',
+  },
 ];
 
 /**
```

Apply to `web/src/experiments/form.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index 8737641..48d3b44 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -47,6 +47,10 @@ export function defaultForm(model: ModelKind = 'sugarscape'): SweepForm {
       metric: { ...form.metric, kind: 'final', series: 'fit' },
     };
   }
+  if (model === 'opinions') {
+    // Fig. 3's axis (the built-in hk-diagonal): surviving opinions against confidence.
+    return { ...form, x: { path: 'epsilon', values: '0.05:0.3:0.05' }, ticks: 1000, metric: { ...form.metric, kind: 'final', series: 'clusters' } };
+  }
   if (model === 'classes') {
     // Fig. 4's axis (the built-in aey-memory): the way out of a fractious start against memory.
     return { ...form, x: { path: 'memory', values: '6:14:2' }, ticks: 100000, metric: { ...form.metric, kind: 'final', series: 'equity_at' } };
```

Apply to `web/src/ui/inspect-panel.ts` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index b269b2a..e94cd39 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -1,12 +1,13 @@
 import { citizenRows, shownCitizen } from '../civil';
 import type { Engine } from '../engine';
-import { isCivilView, isClassesView, isCultureView, isRingView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isCivilView, isClassesView, isCultureView, isOpinionsView, isRingView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
   AnasaziInspection,
   CivilInspection,
   ClassesInspection,
+  OpinionsInspection,
   CultureInspection,
   CultureSiteView,
   LinkView,
@@ -150,6 +151,22 @@ export class InspectPanel {
     return rows;
   }
 
+  /** A cell of the opinion × time diagram (its period, opinion and the agents passing) or of the lattice. */
+  private opinionsRows(view: OpinionsInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    const rows: HTMLElement[] = [];
+    if (view.period !== null && view.opinion !== null) rows.push(row('Period', String(view.period)), row('Opinion', fmt(view.opinion)));
+    else if (view.lattice_site) rows.push(row('Site', `(${view.lattice_site.x}, ${view.lattice_site.y})`));
+    else return [row('Point', 'between the diagram and the lattice')];
+    if (view.agents.length === 0) return [...rows, row('Agents', 'none here')];
+    const shown = view.agents.slice(0, 12);
+    for (const a of shown) {
+      rows.push(row(`#${a.id}`, `${fmt(a.opinion)} (started ${fmt(a.start)}) · reach −${fmt(a.epsilon_left)} +${fmt(a.epsilon_right)} · hears ${a.reaches}`));
+    }
+    if (view.agents.length > shown.length) rows.push(row('', `and ${view.agents.length - shown.length} more`));
+    return rows;
+  }
+
   /** A point of a memory simplex: its mix, the best reply there, and the agents whose memory plots there. */
   private classesRows(view: ClassesInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -273,7 +290,9 @@ export class InspectPanel {
           ? `Agent #${shown.agentId} is gone: killed, or dead of old age.`
           : `Agent #${shown.agentId} has left.`;
       const note = gone ? [h('p', { class: 'error' }, left)] : [];
-      const rows = isClassesView(view)
+      const rows = isOpinionsView(view)
+        ? this.opinionsRows(view)
+        : isClassesView(view)
         ? this.classesRows(view)
         : isCultureView(view)
         ? this.cultureRows(view)
```

- [ ] **Step 3: Build and test**

Run: `(cd web && npm run build && npm test)` — Expected: 529 pass (43 files).

- [ ] **Step 4: Commit**

```bash
git add web/src/types.ts web/src/models.ts web/src/engine.ts web/src/ui/series-data.ts web/src/compare-presets.ts web/src/experiments/form.ts web/src/ui/inspect-panel.ts web/src/models.test.ts web/src/ui/series-data.test.ts web/src/compare-presets.test.ts web/src/experiments/form.test.ts web/src/engine.test.ts web/src/determinism.test.ts
git commit -m "Carry Bounded Confidence through the page" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

**Browser (controller):**
1. Presets menu: a **Bounded Confidence** group (13) and the Compare entry.
2. `hk-regular-50`: lines red at the bottom to magenta at the top converging into two camps with gray between reachable neighbors (HK's Fig. 4); it stops at period 8 with the notice. **Opinion** recolors the lines.
3. `hk-polarisation`: camps form within about 10 periods; the diagram scrolls after 60 periods with the stop off.
4. `hk-lattice`: the torus drawn right of the diagram, colored by opinion.
5. Inspect a line: period, opinion, agents (start, reach, how many it hears); a lattice site; the gap; no Follow.
6. Rules panel groups Population, Confidence, Updating, Lattice; εl/εr only under asymmetric, bias only under leaning, lattice sizes only with the lattice; confidence edits apply live; Agents asks for Reset.
7. The timeline steps back and the diagram with it; the Compare pair runs.
8. Experiments: `hk-diagonal` runs; "From current world" starts at `epsilon`, `0.05:0.3:0.05`, final `clusters`.
9. Recording works; every other model's preset renders.

---

### Task 4: The survey's bounded-confidence claims

**Files:**
- Create: `survey/src/claims/opinions.rs`
- Modify: `survey/src/claims/mod.rs`

- [ ] **Step 1: The claims**

Create `survey/src/claims/opinions.rs` with exactly this content:

```rust
//! Hegselmann & Krause's bounded confidence (milestone 16): the paper's
//! Section 4 figures, its two unfigured claims (serial updating, lattice
//! neighborhoods) and Lorenz's dependence on the number of agents. Runs go
//! to stability; runs that several claims share are memoized per process,
//! keyed by the config and the seeds.

use std::sync::{Arc, Mutex};

use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::opinions::{Confidence, Interaction, Neighborhood, OpinionsConfig, Updating};

use crate::claim::{greater, range, Claim, Outcome, Source, Verdict};
use crate::runner::{model_after, model_preset};

const HK: &str = "Hegselmann & Krause 2002, JASSS 5(3)";
const LORENZ: &str = "Lorenz 2006, JASSS 9(1)";
/// Far past any all-to-all stability time (hundreds of periods at most).
const CAP: u32 = 20_000;

/// One run at its end.
#[derive(Clone, Copy, Debug)]
struct Run {
    clusters: f64,
    largest: f64,
    second: f64,
    mean: f64,
    range: f64,
    stable_at: f64,
    /// The first period with a two-sided split (NaN if none).
    split_at: f64,
    /// Whether a one-sided split ever closed (their count ever fell).
    one_sided_closed: bool,
}

impl Run {
    fn consensus(&self) -> bool {
        self.largest >= 0.99
    }

    /// Two or more major camps: the second holds at least a fifth.
    fn polarized(&self) -> bool {
        self.second >= 0.2
    }
}

fn summarize(w: &ModelWorld) -> Run {
    let s = |n: &str| w.model().series(n).expect("an opinions series");
    let last = |n: &str| *s(n).last().unwrap();
    let splits = s("splits");
    let one = s("one_sided_splits");
    Run {
        clusters: last("clusters"),
        largest: last("largest"),
        second: last("second"),
        mean: last("mean_opinion"),
        range: last("range"),
        stable_at: last("stable_at"),
        split_at: splits
            .iter()
            .position(|&k| k > 0.0)
            .map_or(f64::NAN, |t| t as f64),
        one_sided_closed: one.windows(2).any(|w| w[1] < w[0]),
    }
}

/// HK's defaults (625 random opinions, ε 0.15) with `edit`, run to stability.
fn runs(seeds: &[u64], edit: impl FnOnce(&mut OpinionsConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, Vec<u64>, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = OpinionsConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, _)| *k == key && s == seeds)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(
        &ModelConfig::Opinions(c),
        seeds,
        CAP,
        summarize,
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds.to_vec(), v.clone()));
    v
}

/// A preset's single run (the evenly spaced ones have no randomness).
fn preset_run(id: &str) -> Run {
    model_after(&model_preset(id), &[1], CAP, summarize)[0]
}

/// HK's Fig. 3 and 11 used 50 runs per point.
fn fifty() -> Vec<u64> {
    (1..=50).collect()
}

fn eps(e: f64) -> impl FnOnce(&mut OpinionsConfig) {
    move |c| c.epsilon = e
}

fn col(runs: &[Run], f: impl Fn(&Run) -> f64) -> Vec<f64> {
    runs.iter().map(f).collect()
}

fn count(runs: &[Run], pred: impl Fn(&Run) -> bool) -> usize {
    runs.iter().filter(|r| pred(r)).count()
}

fn outcome(holds: bool, measured: String) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: String::new(),
    }
}

/// A share of runs with `pred`: holds when it lies in `lo..=hi`.
fn share(runs: &[Run], pred: impl Fn(&Run) -> bool, lo: f64, hi: f64, what: &str) -> Outcome {
    let k = count(runs, pred);
    let s = k as f64 / runs.len() as f64;
    outcome(
        (lo..=hi).contains(&s),
        format!("{k}/{} runs {what} ({:.0} %)", runs.len(), 100.0 * s),
    )
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "opinions.fig-2a.survivors",
            item: "hk-plurality",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2a: with ε = 0.01 'exactly 38 different opinions survive' (one run; 35–41 in 80 % of runs)",
            check: |s| range(&col(&runs(s, eps(0.01)), |r| r.clusters), 35.0, 41.0, false),
        },
        Claim {
            id: "opinions.fig-2b.two-camps",
            item: "hk-polarisation",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2b: with ε = 0.15 'the agents end up in two camps' (exactly two surviving opinions in most runs)",
            check: |s| share(&runs(s, eps(0.15)), |r| r.clusters == 2.0, 0.5, 1.0, "in exactly two camps"),
        },
        Claim {
            id: "opinions.fig-2c.consensus",
            item: "hk-consensus",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2c: with ε = 0.25 'the result of the dynamics is consensus'",
            check: |s| share(&runs(s, eps(0.25)), Run::consensus, 0.9, 1.0, "in consensus"),
        },
        Claim {
            id: "opinions.fig-2.fast",
            item: "hk-consensus",
            source: Source::Book,
            citation: HK,
            text: "Fig. 2: 'it takes less than 15 periods to get a stable pattern' (at ε = 0.01, 0.15 and 0.25)",
            check: |s| {
                let mut all = Vec::new();
                for e in [0.01, 0.15, 0.25] {
                    all.extend(runs(s, eps(e)).iter().copied());
                }
                let slowest = all.iter().map(|r| r.stable_at).fold(0.0, f64::max);
                let mut o = share(&all, |r| r.stable_at < 15.0, 0.9, 1.0, "stable before period 15");
                o.detail = format!("slowest: stable at period {slowest}");
                o
            },
        },
        Claim {
            id: "opinions.fig-3.phases",
            item: "hk-diagonal",
            source: Source::Book,
            citation: HK,
            text: "Fig. 3: 'we step from fragmentation (plurality) over polarisation (polarity) to consensus' (50 runs: ≥ 5 opinions at ε 0.05, two or more major camps at 0.18, consensus at 0.3)",
            check: |_| {
                let s = fifty();
                let plural = runs(&s, eps(0.05));
                let polar = runs(&s, eps(0.18));
                let one = runs(&s, eps(0.3));
                let (a, b, c) = (
                    count(&plural, |r| r.clusters >= 5.0),
                    count(&polar, Run::polarized),
                    count(&one, Run::consensus),
                );
                outcome(
                    a >= 45 && b >= 40 && c >= 45,
                    format!("plurality {a}/50 at 0.05, polarization {b}/50 at 0.18, consensus {c}/50 at 0.30"),
                )
            },
        },
        Claim {
            id: "opinions.fig-3.step-25",
            item: "hk-diagonal",
            source: Source::Book,
            citation: HK,
            text: "Fig. 3: 'at about step 25' (ε = 0.25) the camps end and consensus takes over (50 runs: consensus in at most half at 0.21, in 90 % from 0.25)",
            check: |_| {
                let s = fifty();
                let before = count(&runs(&s, eps(0.21)), Run::consensus);
                let at = count(&runs(&s, eps(0.25)), Run::consensus);
                let mid = count(&runs(&s, eps(0.22)), Run::consensus);
                outcome(
                    before <= 25 && at >= 45,
                    format!("consensus in {before}/50 at 0.21, {mid}/50 at 0.22, {at}/50 at 0.25"),
                )
            },
        },
        Claim {
            id: "opinions.fig-3.above-04",
            item: "hk-diagonal",
            source: Source::Book,
            citation: HK,
            text: "'For all values εl, εr > 0.4 the result is always conformity' (ε = 0.45, 0.6, 0.8)",
            check: |s| {
                let mut all = Vec::new();
                for e in [0.45, 0.6, 0.8] {
                    all.extend(runs(s, eps(e)).iter().copied());
                }
                share(&all, Run::consensus, 1.0, 1.0, "in consensus")
            },
        },
        Claim {
            id: "opinions.fig-4.still",
            item: "hk-regular-50",
            source: Source::Book,
            citation: HK,
            text: "Figs. 4–5 (50 evenly spaced, ε 0.2): 'the ε-profile splits in t6'; 'from period 7 to 8 onwards nothing changes anymore'",
            check: |_| {
                let r = preset_run("hk-regular-50");
                outcome(
                    r.split_at == 6.0 && r.stable_at == 8.0,
                    format!("split at period {}, stable at period {}, {} camps", r.split_at, r.stable_at, r.clusters),
                )
            },
        },
        Claim {
            id: "opinions.fig-7.splits",
            item: "hk-regular-plurality",
            source: Source::Book,
            citation: HK,
            text: "Fig. 7 (100 evenly spaced, ε 0.05): 'the profile splits 8 times'",
            check: |_| {
                let r = preset_run("hk-regular-plurality");
                outcome(
                    r.clusters == 9.0,
                    format!("{} surviving opinions ({} splits)", r.clusters, r.clusters - 1.0),
                )
            },
        },
        Claim {
            id: "opinions.fig-8.consensus",
            item: "hk-regular-consensus",
            source: Source::Book,
            citation: HK,
            text: "Fig. 8 (100 evenly spaced, ε 0.25): 'no split, total consensus'",
            check: |_| {
                let r = preset_run("hk-regular-consensus");
                outcome(
                    r.clusters == 1.0 && r.split_at.is_nan(),
                    format!("{} surviving opinion(s), stable at period {}", r.clusters, r.stable_at),
                )
            },
        },
        Claim {
            id: "opinions.fig-12c.drift",
            item: "hk-asymmetry",
            source: Source::Book,
            citation: HK,
            text: "Fig. 12c: 'with asymmetric confidence the mean opinion moves into the direction favoured by the asymmetry. This effect is extreme if there is only little confidence in the non favoured direction' (εr 0.2: εl 0.02 against 0.18)",
            check: |s| {
                let asym = |left: f64| {
                    move |c: &mut OpinionsConfig| {
                        c.confidence = Confidence::Asymmetric;
                        c.epsilon_left = left;
                        c.epsilon_right = 0.2;
                    }
                };
                greater(
                    &col(&runs(s, asym(0.02)), |r| r.mean),
                    &col(&runs(s, asym(0.18)), |r| r.mean),
                    "εl 0.02",
                    "εl 0.18",
                )
            },
        },
        Claim {
            id: "opinions.fig-13.closing",
            item: "hk-one-sided",
            source: Source::Book,
            citation: HK,
            text: "Fig. 13: 'contrary to two-sided splits one-sided splits can close again' (100 evenly spaced, εl 0.08, εr 0.24)",
            check: |_| {
                let r = preset_run("hk-one-sided");
                outcome(
                    r.one_sided_closed,
                    format!("a one-sided split closed: {}; {} camp(s) at the end", r.one_sided_closed, r.clusters),
                )
            },
        },
        Claim {
            id: "opinions.fig-17.break",
            item: "hk-bias",
            source: Source::Book,
            citation: HK,
            text: "Fig. 17c (ε 0.6): consensus survives a mild bias, and at 'about step 11, i.e. m ≈ 0.4' it breaks down (consensus in every run at m 0.36, in at most half by m 0.52)",
            check: |s| {
                let at = |m: f64| {
                    count(
                        &runs(s, move |c| {
                            c.confidence = Confidence::OpinionDependent;
                            c.epsilon = 0.6;
                            c.bias = m;
                        }),
                        Run::consensus,
                    )
                };
                let (a, b, c) = (at(0.36), at(0.44), at(0.52));
                let n = s.len();
                outcome(
                    a == n && 2 * c <= n,
                    format!("consensus in {a}/{n} at m 0.36, {b}/{n} at 0.44, {c}/{n} at 0.52"),
                )
            },
        },
        Claim {
            id: "opinions.fig-17.extremes",
            item: "hk-bias",
            source: Source::Book,
            citation: HK,
            text: "Fig. 17: 'for an m = 1 the two camps occupy the most extreme positions 0 and 1' (ε 0.2, 0.4, 0.6: final range at least 0.95)",
            check: |s| {
                let mut ranges = Vec::new();
                for e in [0.2, 0.4, 0.6] {
                    ranges.extend(col(
                        &runs(s, move |c| {
                            c.confidence = Confidence::OpinionDependent;
                            c.epsilon = e;
                            c.bias = 1.0;
                        }),
                        |r| r.range,
                    ));
                }
                range(&ranges, 0.95, 1.0, false)
            },
        },
        Claim {
            id: "opinions.fig-17.slower",
            item: "hk-bias",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'If consensus is still feasible, it takes more time to get there' as the bias grows (ε 0.6: m 0.4 against 0)",
            check: |s| {
                let at = |m: f64| {
                    col(
                        &runs(s, move |c| {
                            c.confidence = Confidence::OpinionDependent;
                            c.epsilon = 0.6;
                            c.bias = m;
                        }),
                        |r| r.stable_at,
                    )
                };
                greater(&at(0.4), &at(0.0), "m 0.4", "m 0")
            },
        },
        Claim {
            id: "opinions.serial.phases",
            item: "hk-updating",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'none of the results stated above depends crucially on simultaneous updating' (both serial readings: plurality at 0.05, polarization at 0.18, consensus at 0.3, as simultaneous)",
            check: |s| {
                let mut out = Vec::new();
                let mut ok = true;
                for (name, u) in [
                    ("shuffled", Updating::SerialShuffled),
                    ("random draws", Updating::SerialRandom),
                ] {
                    let at = |e: f64| {
                        runs(s, move |c| {
                            c.epsilon = e;
                            c.updating = u;
                        })
                    };
                    let n = s.len();
                    let (a, b, c) = (
                        count(&at(0.05), |r| r.clusters >= 5.0),
                        count(&at(0.18), Run::polarized),
                        count(&at(0.3), Run::consensus),
                    );
                    ok &= 10 * a >= 9 * n && 10 * b >= 8 * n && 10 * c >= 9 * n;
                    out.push(format!("{name}: {a}, {b}, {c} of {n}"));
                }
                outcome(ok, out.join("; "))
            },
        },
        Claim {
            id: "opinions.serial.extremes",
            item: "hk-updating",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'Random serial updating gives extreme opinions a slightly better chance to survive' (more surviving opinions at ε 0.05, shuffled order)",
            check: |s| {
                let at = |u: Updating| {
                    col(
                        &runs(s, move |c| {
                            c.epsilon = 0.05;
                            c.updating = u;
                        }),
                        |r| r.clusters,
                    )
                };
                greater(
                    &at(Updating::SerialShuffled),
                    &at(Updating::Simultaneous),
                    "serial",
                    "simultaneous",
                )
            },
        },
        Claim {
            id: "opinions.lattice.no-polarization",
            item: "hk-lattice",
            source: Source::Book,
            citation: HK,
            text: "§4.3: 'If the neighbourhoods in which the agents interact are fairly small (though overlapping!), then … polarization, disappears' (25 × 25 Moore and von Neumann tori, ε 0.1–0.3: runs with a second camp of a fifth, pooled, at most 10 %)",
            check: |s| {
                let es = [0.1, 0.15, 0.2, 0.25, 0.3];
                let (mut lattice, mut all, mut worst) = (0, 0, String::new());
                let mut worst_k = 0;
                for e in es {
                    all += count(&runs(s, eps(e)), Run::polarized);
                    for (name, nb) in [
                        ("Moore", Neighborhood::Moore),
                        ("von Neumann", Neighborhood::VonNeumann),
                    ] {
                        let k = count(
                            &runs(s, move |c| {
                                c.epsilon = e;
                                c.interaction = Interaction::Lattice;
                                c.lattice.neighborhood = nb;
                            }),
                            Run::polarized,
                        );
                        lattice += k;
                        if k > worst_k {
                            worst_k = k;
                            worst = format!("; most: {name} at ε {e}, {k}/{}", s.len());
                        }
                    }
                }
                let n = s.len();
                outcome(
                    lattice * 10 <= 2 * es.len() * n,
                    format!(
                        "polarized: lattices {lattice}/{}, everyone {all}/{}{worst}",
                        2 * es.len() * n,
                        es.len() * n
                    ),
                )
            },
        },
        Claim {
            id: "opinions.lorenz.agents",
            item: "hk-population",
            source: Source::Comment,
            citation: LORENZ,
            text: "the confidence at which consensus becomes typical depends on the number of agents (ε 0.22: consensus more often with 1000 agents than with 50)",
            check: |s| {
                let at = |n: u32| {
                    col(
                        &runs(s, move |c| {
                            c.epsilon = 0.22;
                            c.agents = n;
                        }),
                        |r| f64::from(u8::from(r.consensus())),
                    )
                };
                greater(&at(1000), &at(50), "1000 agents", "50 agents")
            },
        },
    ]
}
```

Apply to `survey/src/claims/mod.rs` (a unified diff against `main` at `9a5ad57`; `git apply` accepts it from a file, or make the same edits by hand):

```diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index 9f42e0e..6d8407c 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -6,6 +6,7 @@ mod ch6;
 mod civil;
 mod classes;
 mod culture;
+mod opinions;
 mod spatial;
 mod tags;
 
@@ -21,6 +22,7 @@ pub fn all() -> Vec<Claim> {
         civil::claims(),
         classes::claims(),
         culture::claims(),
+        opinions::claims(),
         spatial::claims(),
         tags::claims(),
     ]
```

- [ ] **Step 2: Format the new file; run**

Run: `rustfmt --edition 2021 survey/src/claims/opinions.rs && (cd survey && cargo run --release -- --only opinions)`
Expected (as planning measured; stop and report any difference):

| id | verdict |
|---|---|
| opinions.fig-2a.survivors, fig-2c.consensus | Holds |
| opinions.fig-2b.two-camps | **Fails** (6/20 in exactly two camps) |
| opinions.fig-2.fast | **Fails** (53/60 stable before period 15; slowest 168) |
| opinions.fig-3.phases, fig-3.step-25, fig-3.above-04 | Holds |
| opinions.fig-4.still, fig-7.splits, fig-8.consensus | Holds |
| opinions.fig-12c.drift, fig-13.closing | Holds |
| opinions.fig-17.break, fig-17.extremes, fig-17.slower | Holds |
| opinions.serial.phases, serial.extremes | Holds |
| opinions.lattice.no-polarization | Holds (9/200 lattice runs polarized, 57/100 among everyone) |
| opinions.lorenz.agents | Holds |

Delete `survey/out/results-opinions.json`.

- [ ] **Step 3: Test, check clippy, commit**

Run: `(cd survey && cargo test --release && cargo clippy --all-targets 2>&1 | grep -c 'claims/opinions.rs')` — Expected: pass; `0`.

```bash
git add survey/src/claims/opinions.rs survey/src/claims/mod.rs
git commit -m "Survey Bounded Confidence and its unfigured claims" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---
### Task 5: README, roadmap, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/superpowers/specs/2026-09-25-bounded-confidence-design.md`

- [ ] **Step 1: README — the intro names the model**

Replace `**Axelrod Culture** and **Emergence of Classes**.` with `**Axelrod Culture**, **Emergence of Classes** and **Bounded Confidence**.`

- [ ] **Step 2: README — the Bounded Confidence section**

Insert immediately before the line `## Experiments`:

```markdown
### Bounded Confidence (Hegselmann & Krause 2002)

Opinions between 0 and 1. Each period every agent moves to the mean of the opinions within its
reach, its own included; everything further away is ignored. The defaults are the paper's: 625
opinions drawn uniformly, updated all at once, each agent reaching ε = 0.15 either way. Agents who
cannot reach each other drift apart for good, so the profile freezes into camps — many with little
confidence (plurality), two or three in between (polarization), one with much (consensus). A period
is stable when no opinion moves more than 10⁻¹⁰; opinions within 10⁻⁶ count as one.

What reproduces (20 seeds unless stated): Fig. 2a's "exactly 38" surviving opinions at ε = 0.01
(median 37.5); consensus at 0.25; Fig. 3's walk along ε — plurality, then camps, then a consensus
that takes over between 0.21 and 0.25 (50 runs: 13, 30 and 50 in consensus at 0.21, 0.22, 0.25),
always above 0.4; the evenly spaced figures exactly (50 opinions at 0.2 split in period 6 and are
still from period 8; 100 at 0.05 split 8 times; 100 at 0.25 agree). What does not: **Fig. 2b's two
camps at ε = 0.15 are the exception — 6 runs of 20; 14 end with a third camp in the middle, usually
as large** (two camps are the rule only from 0.16 to 0.21); and "less than 15 periods to a stable
pattern" holds for 53 runs of 60, the slowest taking 168 while two nearly merged camps close.

The paper's asymmetries are settings. **Asymmetric** confidence, the same for everyone (§4.2.1):
the mean drifts toward the side agents listen to (at εr = 0.2, from 0.53 with εl = 0.18 to 0.94
with εl = 0.02), and one-sided splits — a gap one side reaches across and the other does not —
close again, as the paper says two-sided ones never do. Confidence **leaning with one's opinion**
(§4.2.2, bias m): camps grow and move outward, reaching 0 and 1 at m = 1; at ε = 0.6 consensus holds
to m = 0.36 and breaks between 0.44 (18 of 20 runs in consensus) and 0.52 (8 of 20) — the paper
says "m ≈ 0.4" — and takes longer before it breaks.

Two of the paper's claims have no figure, so they are switches here. **Updating**: "none of the
results … depends crucially on simultaneous updating", without saying which serial order — each
agent once per period in random order, or n random draws. Measured, the phases keep their places
under both, and serial updating leaves slightly more opinions at small ε (9 against 8 at ε = 0.05),
as the paper says. **Who listens to whom**: on a torus where agents hear only their neighbors,
"polarization … disappears". Measured on a 25 × 25 torus, it does: one big camp with dozens of
stranded local minorities, settling only after thousands of periods; a second camp of a fifth of
the agents in 9 of 200 lattice runs (ε 0.1–0.3, both neighborhoods) against 57 of 100 among
everyone. Lorenz (2006) showed that the consensus threshold depends on the number of agents: at
ε = 0.22 consensus in 1 run of 20 with 50 agents, 12 with 1000.

The view is the paper's opinion × time diagram: each agent a line, red where it started at 0 to
magenta at 1 (**Start**; **Opinion** colors by where it is now), gray between neighbors still within
each other's reach, the last 60 periods; with a lattice the torus is drawn to the right. Inspect a
point for its period, opinion and the agents passing (start, reach, how many they hear), or a site.
Charts: Clusters; Largest camps; Mean and median; Splits (two-sided, one-sided); Change. A run stops
when stable. Presets: `hk-plurality`, `hk-polarisation`, `hk-consensus`, `hk-regular-50`,
`hk-regular-plurality`, `hk-regular-consensus`, `hk-asym-a`, `hk-asym-b`, `hk-asym-c`,
`hk-one-sided` (Fig. 13's caption says εl = 0.8, read as 0.08), `hk-bias`, `hk-serial`,
`hk-lattice`. **Compare** entry: "Simultaneous vs serial updating — Bounded Confidence (Compare)".
Built-in sweeps: `hk-diagonal`, `hk-asymmetry`, `hk-bias`, `hk-updating`, `hk-lattice`,
`hk-population`.

Credit: Rainer Hegselmann and Ulrich Krause, "Opinion Dynamics and Bounded Confidence: Models,
Analysis, and Simulation," *Journal of Artificial Societies and Social Simulation* 5(3) (2002), 2;
Jan Lorenz, "Consensus Strikes Back in the Hegselmann-Krause Model of Continuous Opinion Dynamics
Under Bounded Confidence," *JASSS* 9(1) (2006), 8. See
`docs/superpowers/specs/2026-09-25-bounded-confidence-design.md`.
```

- [ ] **Step 3: Roadmap**

Insert before `## Experiments and science`:

```markdown
## Milestone 16: Bounded Confidence (done)

Hegselmann and Krause's opinion dynamics under bounded confidence (2002) as a tenth model kind, with
symmetric, asymmetric and opinion-dependent confidence as settings and the paper's two unfigured
claims — random serial updating, lattice neighborhoods — as switches. The survivors at small
confidence, the walk from plurality through polarization to consensus, the evenly spaced figures,
the asymmetric drift and the bias's break reproduce; Fig. 2b's two camps are the exception at its
confidence, and the lattice claim holds. See
`docs/superpowers/specs/2026-09-25-bounded-confidence-design.md`.
```

and after `- **Axtell, Epstein & Young's emergence of classes**: done (Milestone 15).` add:

```markdown
- **Hegselmann & Krause's bounded confidence**: done (Milestone 16).
```

- [ ] **Step 4: Spec amendments**

In `docs/superpowers/specs/2026-09-25-bounded-confidence-design.md`:
1. In **Step**, the **Numerics** bullet: replace "clusters are maximal runs of sorted opinions whose neighboring gaps are at most 10⁻¹⁰" with "clusters are maximal runs of sorted opinions whose neighboring gaps are at most 10⁻⁶ (serial updating closes camps only geometrically: a 10⁻¹⁰ tolerance counted one serially updated consensus as two or three)", and "the tolerance is far below any ε" with "both tolerances are far below any ε".
2. In **Statistics**, `clusters (surviving opinions, as above)` stays; add after the list: "A run is **polarized** (survey, descriptions) when its second camp holds at least a fifth of the agents."
3. In **Views**: "240 × 200 cells" → "241 × 201 cells (60 periods of 4 cells plus the current column; with `lattice`, 450 wide: the torus is a panel 8 cells right of the diagram, as tall as it)"; the **Color modes** bullet → "**Start** (each line by its starting opinion, red at 0 through magenta at 1, as in HK) and **Opinion** (by the current opinion). The lattice is a panel, not a mode: Inspect has no mode to tell which view a click is in."; the **Inspect** bullet: "on the Lattice mode, a site" → "the nearest kept period at or left of the column; on the lattice panel, a site".
4. In **Config**: after the table's paragraph add "The Rules panel shows `epsilon_left` and `epsilon_right` only under `asymmetric`, `bias` only under `opinion_dependent`, and the lattice's size and neighborhood only with `lattice`."
5. In **Experiments and CLI**, replace the six sweep bullets with:
   - `hk-diagonal`: final `clusters` against ε = 0.01 … 0.40, 50 seeds (Fig. 3).
   - `hk-asymmetry`: final `mean_opinion` against εr = 0.02 … 0.40, series εl = 0.02, 0.1, 0.2 (Fig. 12c's grid; a series cannot scale with x, so Fig. 11's lines εl = k·εr are the survey's).
   - `hk-bias`: final `range` against m = 0 … 1 in 26 steps, series ε = 0.2, 0.4, 0.6, 625 random opinions (Fig. 17).
   - `hk-updating`: final `clusters` against ε = 0.05 … 0.30, series the three updating modes.
   - `hk-lattice`: final `second` after 2000 periods against ε = 0.1 … 0.6, series everyone, Moore, von Neumann.
   - `hk-population`: final `largest` against n = 25 … 2000, series ε = 0.2, 0.22, 0.25 (Lorenz 2006).
6. In **Testing**, Web: "determinism through the engine" → "determinism through the engine (`hk-lattice` in the golden list — every other preset stops before 200 ticks — and `hk-regular-50` run to its stop at period 8)".
7. Add before **Architecture**:

```markdown
## Measured in planning

20 seeds unless stated; the survey reproduces each.

- Fig. 2 (625 random): at ε 0.01 a median of 37.5 survivors (34–43; the paper's run: 38); at 0.15 exactly two camps in 6 runs, a third camp in the middle in 14 (in 12 about as large as the others); at 0.25 consensus in all. Stable before period 15 in 53 of 60 runs; the slowest at period 168.
- Fig. 3 (50 runs): 37.7, 7.9, 3.6, 2.7, 1.9 survivors at ε 0.01, 0.05, 0.10, 0.15, 0.20; two camps the rule from 0.16 to 0.21; consensus in 13, 30, 48, 50 runs at 0.21, 0.22, 0.24, 0.25; consensus in every run above 0.4.
- Evenly spaced: 50 at 0.2 split in period 6 and are still from period 8; 100 at 0.05 split 8 times (9 survivors, stable at 20); 100 at 0.25 agree at period 10.
- Asymmetric: at εr 0.2 the mean is 0.53, 0.68 and 0.94 with εl 0.18, 0.1 and 0.02; `hk-one-sided` (100 evenly spaced, 0.08/0.24) opens a one-sided split in period 5 that closes in period 9, ending in consensus at 0.85.
- Opinion-dependent (ε 0.6): consensus in 20, 18 and 8 runs of 20 at m 0.36, 0.44 and 0.52, none from 0.64; range 0.99 at m = 1 for ε 0.2, 0.4, 0.6; median stable period 5 at m 0, 24 at m 0.4. Fig. 18c (50 evenly spaced, m 0.5) splits in period 4.
- Serial: at ε 0.05, 7.9 survivors simultaneous, 8.4 shuffled, 9.0 random draws (50 runs); the phases in place under both orders.
- Lattice (25 × 25, run to stability, capped at 20 000): a second camp of a fifth in 7 of 20 Moore runs at 0.15 and 2 of 20 von Neumann runs at 0.2, never otherwise (ε 0.05–0.6); everyone listening: 20, 20, 17 of 20 at 0.1, 0.15, 0.2. Stability takes hundreds to tens of thousands of periods.
- Lorenz: consensus at ε 0.22 in 1 of 20 runs with 50 agents, 12 with 625, 16 with 2000; at 0.25, 7 with 50 and 20 from 625.
```

- [ ] **Step 5: Full verification**

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test book -- --ignored
cargo build -p sugarscape-core --target wasm32-unknown-unknown
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && cargo test --release)
```

Expected: all clean and passing. **Browser (controller):** Task 3's list in full, plus one preset of every other model.

- [ ] **Step 6: Commit**

```bash
git add README.md docs/roadmap.md docs/superpowers/specs/2026-09-25-bounded-confidence-design.md
git commit -m "Document Bounded Confidence and its unfigured claims; mark milestone 16 done" -m "Claude-Session: https://claude.ai/code/session_01Rt9P4zfGCkP3zL1H71NZcE"
```

---

## Self-review (planning)

- **Spec coverage:** config, reach, step (three orders, lattice), numerics, statistics, views, Inspect, presets → Task 1; experiments and CLI → Task 2; page and Compare → Task 3; survey → Task 4; docs → Task 5; testing → each task.
- **Amendments** (two tolerances, the frame and lattice panel, the shown-if fields, "polarized", the sweep shapes, the web golden list, the measurements) → Task 5 Step 4.
- **Placeholders:** none; code is the verified prototype's, diffs are exact.
