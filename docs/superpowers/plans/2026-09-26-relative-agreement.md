# Relative Agreement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Deffuant et al.'s relative agreement model with extremists as a model kind, `agreement` ("Relative Agreement"), with their pairwise bounded confidence and §6 variants, Amblard and Deffuant's lattices and small worlds, Weisbuch's scale-free networks, and Meadows and Cliff's and the authors' readings of what the 2002 paper left unstated as switches; sixteen presets, eight measured sweeps and a 36-claim survey, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/agreement/` — `config.rs` (parameters, validation, schema), `network.rs` (lattice, Watts–Strogatz, Barabási–Albert), `stats.rs` (y, outcomes, cluster measures), `view.rs` (frame geometry and colors), `world.rs` (`AgreementWorld`: pair meetings, the four rules, stability, the compressing history, rendering, Inspect), `presets.rs`, `mod.rs` — wired into `ModelConfig`/`ModelWorld` like the other models, sharing `opinions`' cluster and canvas helpers. The page adds the model's types, color modes, charts, Inspect rows, a Compare entry and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-26-relative-agreement-design.md` (binding, as amended in Task 5). Sources in `papers/bounded-confidence/`: `deffuant-neau-amblard-weisbuch-2000-acs-mixing-beliefs.pdf`, `deffuant-amblard-weisbuch-faure-2002-jasss-how-can-extremism-prevail.html` (its equations are images at jasss.org/5/4/1/eq*.gif and eq1.jpg), `amblard-deffuant-2004-network-topology-and-extremism.pdf`, `weisbuch-2003-bounded-confidence-and-social-networks.pdf`, `meadows-cliff-2012-jasss-reexamining-the-relative-agreement-model.html`, `deffuant-amblard-weisbuch-2013-jasss-meadows-and-cliff-are-wrong.html`.

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited (`MODEL_GOLDEN` gains sixteen `dnaw-*`/`ra-*`/`ad-*`/`w-*` entries). `opinions`' `clusters` keeps its tolerance (10⁻⁶) through the new `groups(sorted, gap)`.
- **One engine path; deterministic; portable:** native and WASM fingerprints identical (verified in planning by `wasm-pack test` and the web determinism test). Every random draw uses `u32` ranges or `f64` samples, never `usize` ranges.
- **Literal defaults, named departures, honest descriptions:** the defaults are the 2002 paper as stated (extremists drawn, run to stability, eq. 11's printed window); Meadows and Cliff's and the reply's readings and the listener's window are switches that presets turn on; every description says what it measured.
- **Copy (verbatim):** model label **Relative Agreement**; preset ids `dnaw-consensus`, `dnaw-clusters`, `dnaw-lattice`, `dnaw-lattice-clusters`, `ra-uniform`, `ra-central`, `ra-both`, `ra-single`, `ra-literal`, `ra-meadows-cliff`, `ra-deffuant-2013`, `ra-bc-extremists`, `ra-bc-printed`, `ad-moore`, `ad-small-world`, `w-scale-free`; Compare entry **Meadows and Cliff vs Deffuant et al.’s reply — Relative Agreement (Compare)** (id `ra-meadows-cliff-vs-reply`); color modes **Uncertainty**, **Role**, **Start**; schema groups **Population**, **Interaction**, **Extremists**, **Network**, **Stopping**; charts **Convergence**, **Clusters**, **Dispersion**, **Opinion and uncertainty**, **Change**; time axis **Periods**; sweeps `ra-clusters`, `ra-map`, `ra-readings`, `ra-population`, `ra-rules`, `ra-delta`, `ad-connectivity`, `w-dispersion`; series `y, p_plus, p_minus, outcome, clusters, major, isolated, largest, second, dispersion, unmoved, mean_opinion, mean_uncertainty, max_change, stable_at`; notices `Stable at t = N: no opinion or uncertainty moves any more — Reset, or change the rule, to run it again` and `This run has reached its last period (N) — Reset to run it again`; CLI `(stable)` and `(its last period)`.
- Every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_011pY2cJwAezJH4xb32Aw8xr`. Stage only the task's files; never `.claude/` or `papers/`.
- Rust: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. In `survey/`, format only `survey/src/claims/agreement.rs` (`rustfmt --edition 2021`); its `ch6.rs` clippy warnings are not ours.
- Web: `(cd web && npm run build && npm test)` (run `npm ci` first in a fresh worktree).
- **Browser checks are the controller's** (Task 3's Step 6; the full pass in Task 5).

## Review Focus

1. **A meeting's update order** — `simultaneous` computes both influences from the old values, `sequential` lets the second agent's new values act on the first, `one_way` moves only the first; anyone-meets-anyone never pairs an agent with itself. Pinned in Task 1 by `a_meeting_updates_both_one_after_the_other_or_one` and `pairs_are_distinct_or_follow_the_network`.
2. **What y counts** — each placement's boundary (the innermost drawn extremist; ±1; the band edge) less the margin, 0 on a side without extremists, moderates only; an odd extremist count at δ 0 leans by a coin, not always positive. Pinned in Task 1 by `extremists_are_counted_placed_and_bounded` and `y_counts_moderates_past_the_boundary_less_the_margin`.
3. **Stopping** — stable (nothing moved by more than 10⁻⁶ in a period) or at `stop_at`, whichever comes first; a live edit to how agents move resumes a stable run, a margin edit does not; a stopped run holds its values for sweeps. Pinned in Task 1 by `a_run_stabilizes_and_stops_or_stops_at_its_period` and `a_live_edit_after_stability_resumes_the_run`; in Task 2 by the CLI test; in Task 3 by the engine tests.
4. **The history** — it halves when 240 periods are kept, so a run of any length stays in the diagram, the current period is always its last column, and keyframes restore it. Pinned in Task 1 by `the_history_halves_to_keep_every_run_in_view` and `keyframes_restore_opinions_and_the_diagram`.
5. **Networks are well formed** — no loops or double links, links both ways, the stated degrees, rewiring keeps the link count, a grid substrate's radius fits its sides, the same seed builds the same graph. Pinned in Task 1 by the `network` tests and `networks_validate_their_own_fields`.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --workspace` (754), `cargo clippy --all-targets -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (43), `npm run build && npm test` (626) and the survey.

1. **Inspect reads cells, not agents (amends the spec):** `agent` is always null and `locate` returns nothing, as in `opinions`. The page re-reads a tracked agent wherever `locate` puts it; a line clicked in the diagram would jump to the agent's dot, usually crowded by neighbors.
2. **The torus is colored by current opinion in every mode** (the spec said "by opinion"; under Start it would otherwise show where agents began, not the lattice's state).
3. **Pairs:** anyone-meets-anyone draws the first agent, then the second among the others (one draw each); on a network `edge` draws a link and a coin for its order, `node` an agent and then a neighbor (a neighborless agent's turn passes).
4. **Barabási–Albert starts from a complete graph on max(3, m + 1) agents** (Weisbuch's triangle for m 2; K5 for m 4, his eight-link network).
5. **`agents` is ignored on the torus** (a lattice, or a small world grown on one): the population is width × height, stated in the field's help.
6. **Weisbuch's lattice line** in `w-dispersion` uses his pairing and one-way updating; the well-mixed line uses DNAW's symmetric meetings.
7. **The web golden list** holds the nine presets still running at period 200 on seed 1; engine tests run `ra-single` to its stop at 71 and `ra-meadows-cliff` to 200.
8. **Planning's findings** (the survey reproduces them; spec amendments list the numbers): Fig. 9's layout reproduces as the paper states the model; Meadows and Cliff's failure reproduces under their reading and both of the reply's fixes are needed (y 0.00, 0.40, 0.27, 0.98); balanced extremists' single extreme vanishes as N grows (Fig. 9's zone holds in 14 of 35 cells at its stated N 1000, 35 of 35 at N 200); Figs. 5 and 7 do not reproduce at their stated parameters (48 % against 4 %; both extremes in 39 of 40 runs at μ 0.5); eq. 11 as printed reproduces none of §6, the listener's window all of it; ue matters and the reply's cutoff misreads it; Amblard and Deffuant's threshold sits at k 32–256, not "around 8", and their low-k both extremes need a 0.7 cutoff.

---

### Task 1: The relative agreement model in the core

**Files:**
- Create: `crates/sugarscape-core/src/agreement/{config,network,stats,view,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/opinions/stats.rs`, `crates/sugarscape-core/src/opinions/mod.rs`, `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{lerp, Rgb}`, `crate::rng::{self, SimRng}`, `crate::schema::{Apply, Param}` (`shown_if`, `with_help`), `crate::presets::ModelPreset`, and from `opinions`: `Neighborhood`, `hue`, `site_cells`, `Canvas` and the new `groups(sorted, gap)`.
- Produces: `agreement::{AgreementConfig, LatticeConfig, SmallWorldConfig, ScaleFreeConfig, Rule, Window, Placement, PairUpdate, Network, Pairing, Substrate, MAX_AGENTS, MAX_SIDE, schema, presets, Graph, grouping, AgreementSnapshot, Outcome, GAP, SERIES, STILL, row, scatter_col, COLUMNS, KEPT, SCATTER_X, TALL, TORUS_X, extremist_counts, influence, AgreementWorld, AgreementAgent, AgreementCell, AgreementInspection, AgreementMode, Role}`; `AgreementConfig::{population, on_torus, grid_radius, validate}`; `AgreementWorld::{new, step, run, is_finished, opinions, uncertainties, starts, roles, inspect, config, tick, stats}`; `ModelKind::Agreement` (`"agreement"`), `ModelConfig::Agreement`, `ModelWorld::Agreement`; `opinions::groups`.

- [ ] **Step 1: Write the module**

Create `crates/sugarscape-core/src/agreement/config.rs` with exactly this content:

```rust
//! The Relative Agreement model's parameters: Deffuant et al.'s (2002)
//! relative agreement and the three bounded-confidence rules they compare
//! it with, extremists, the networks of Deffuant et al. (2000), Amblard and
//! Deffuant (2004) and Weisbuch (2004), and the readings the papers leave
//! open (Meadows and Cliff's, the 2013 reply's, eq. 11's window) as switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::opinions::Neighborhood;
use crate::schema::{Apply, Param};

/// How one agent influences another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// Relative agreement (DAWF eqs. 1–6).
    Ra,
    /// Bounded confidence (DAWF eq. 11; DNAW).
    Bc,
    /// Bounded confidence with averaging uncertainties (eqs. 11–12).
    BcAveraging,
    /// Bounded confidence with uncertainty from variance (eqs. 13–14).
    BcVariance,
}

/// Whose uncertainty the bounded-confidence rules' window uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Window {
    /// Eq. 11 as printed: |x − x′| < u′, the influencer's.
    Influencer,
    /// The listener's own: |x − x′| < u (RA's hᵢⱼ > uᵢ when uᵢ < uⱼ).
    Listener,
}

/// Where the extremists start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    /// DAWF: N uniform draws; the most extreme take ue.
    Drawn,
    /// AD: the same agents, set to ±1.
    Bounds,
    /// M&C: extremists uniform in [b, 1] and [−1, −b], moderates on (−b, b).
    Band,
}

/// How the two agents of a meeting update.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairUpdate {
    /// Both from their old values (`melsimp.c`, M&C).
    Simultaneous,
    /// The first acts on the second, then the second, updated, on the first.
    Sequential,
    /// Only the first updates (Weisbuch 2004).
    OneWay,
}

/// Who can meet whom.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// Anyone (DNAW §2, DAWF).
    All,
    /// A torus (DNAW §3, AD §3.1).
    Lattice,
    /// Watts–Strogatz (AD §3.2).
    SmallWorld,
    /// Barabási–Albert (Weisbuch).
    ScaleFree,
}

/// How a meeting's pair is drawn on a network.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pairing {
    /// A uniform link, its order uniform (DNAW, AD).
    Edge,
    /// A uniform agent, then a uniform neighbor (Weisbuch).
    Node,
}

/// A small world's regular start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Substrate {
    /// A circle, k/2 neighbors each side (AD Fig. 4).
    Ring,
    /// The lattice's torus with a Moore neighborhood of radius r (AD Fig. 6).
    Grid,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LatticeConfig {
    pub width: u32,
    pub height: u32,
    pub neighborhood: Neighborhood,
}

impl Default for LatticeConfig {
    /// DNAW's 29 × 29 square lattice, four neighbors each.
    fn default() -> Self {
        LatticeConfig {
            width: 29,
            height: 29,
            neighborhood: Neighborhood::VonNeumann,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SmallWorldConfig {
    pub substrate: Substrate,
    /// k: neighbors each agent starts with.
    pub degree: u32,
    /// p: the chance each link is rewired.
    pub rewire: f64,
}

impl Default for SmallWorldConfig {
    fn default() -> Self {
        SmallWorldConfig {
            substrate: Substrate::Ring,
            degree: 8,
            rewire: 0.1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ScaleFreeConfig {
    /// m: links per new node (mean degree 2m).
    pub links: u32,
}

impl Default for ScaleFreeConfig {
    /// Weisbuch's two links per new node (mean degree 4).
    fn default() -> Self {
        ScaleFreeConfig { links: 2 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AgreementConfig {
    /// N; under a lattice or a grid-substrate small world, width × height.
    pub agents: u32,
    pub rule: Rule,
    pub window: Window,
    /// μ.
    pub mu: f64,
    /// α under `bc_variance`.
    pub alpha: f64,
    /// U: the moderates' starting uncertainty.
    pub uncertainty: f64,
    /// pe: the proportion of extremists.
    pub extremists: f64,
    /// ue.
    pub extremist_uncertainty: f64,
    /// δ = |p₊ − p₋| / pe; the surplus is positive.
    pub delta: f64,
    pub placement: Placement,
    /// b under `band`.
    pub band: f64,
    /// A moderate is a new extremist beyond its side's boundary minus this.
    pub extreme_margin: f64,
    pub pair_update: PairUpdate,
    pub network: Network,
    pub lattice: LatticeConfig,
    pub small_world: SmallWorldConfig,
    pub scale_free: ScaleFreeConfig,
    pub pairing: Pairing,
    /// Stop at the first stable period.
    pub stop_when_stable: bool,
    /// Stop at this period (0: never): the cap under stability.
    pub stop_at: u32,
}

impl Default for AgreementConfig {
    /// Relative agreement with extremists at Fig. 9's μ and ue: N 200, U 1,
    /// pe 0.1, the extremists drawn, run to stability.
    fn default() -> Self {
        AgreementConfig {
            agents: 200,
            rule: Rule::Ra,
            window: Window::Influencer,
            mu: 0.2,
            alpha: 0.8,
            uncertainty: 1.0,
            extremists: 0.1,
            extremist_uncertainty: 0.1,
            delta: 0.0,
            placement: Placement::Drawn,
            band: 0.8,
            extreme_margin: 0.1,
            pair_update: PairUpdate::Simultaneous,
            network: Network::All,
            lattice: LatticeConfig::default(),
            small_world: SmallWorldConfig::default(),
            scale_free: ScaleFreeConfig::default(),
            pairing: Pairing::Edge,
            stop_when_stable: true,
            stop_at: 20_000,
        }
    }
}

/// The largest lattice side.
pub const MAX_SIDE: u32 = 64;
/// The most agents.
pub const MAX_AGENTS: u32 = 4000;

impl AgreementConfig {
    /// Whether agents sit on the lattice's torus (a lattice, or a small world
    /// grown from it).
    pub fn on_torus(&self) -> bool {
        self.network == Network::Lattice
            || (self.network == Network::SmallWorld
                && self.small_world.substrate == Substrate::Grid)
    }

    /// The number of agents: width × height on the torus, else `agents`.
    pub fn population(&self) -> usize {
        if self.on_torus() {
            self.lattice.width as usize * self.lattice.height as usize
        } else {
            self.agents as usize
        }
    }

    /// The grid substrate's Moore radius for degree k = (2r + 1)² − 1, if k
    /// is of that form.
    pub fn grid_radius(&self) -> Option<u32> {
        (1..=MAX_SIDE / 2).find(|r| (2 * r + 1) * (2 * r + 1) - 1 == self.small_world.degree)
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
            (2..=MAX_AGENTS).contains(&self.agents),
            "agents",
            "must be between 2 and 4000",
        );
        check(unit(self.mu), "mu", "must be between 0 and 1");
        check(unit(self.alpha), "alpha", "must be between 0 and 1");
        check(
            self.uncertainty > 0.0 && self.uncertainty <= 4.0,
            "uncertainty",
            "must be above 0 and at most 4",
        );
        check(
            unit(self.extremists),
            "extremists",
            "must be between 0 and 1",
        );
        check(
            self.extremist_uncertainty > 0.0 && self.extremist_uncertainty <= 4.0,
            "extremist_uncertainty",
            "must be above 0 and at most 4",
        );
        check(unit(self.delta), "delta", "must be between 0 and 1");
        check(
            self.band > 0.0 && self.band < 1.0,
            "band",
            "must be between 0 and 1 (exclusive)",
        );
        check(
            unit(self.extreme_margin),
            "extreme_margin",
            "must be between 0 and 1",
        );
        check(
            self.stop_at <= 1_000_000,
            "stop_at",
            "must be at most 1000000",
        );
        if self.on_torus() {
            let l = &self.lattice;
            check(
                (3..=MAX_SIDE).contains(&l.width) && (3..=MAX_SIDE).contains(&l.height),
                "lattice",
                "sides must be between 3 and 64",
            );
        }
        let n = self.population() as u64;
        match self.network {
            Network::SmallWorld => {
                let s = &self.small_world;
                check(
                    unit(s.rewire),
                    "small_world",
                    "rewire must be between 0 and 1",
                );
                match s.substrate {
                    Substrate::Ring => check(
                        s.degree >= 2
                            && s.degree <= 256
                            && s.degree.is_multiple_of(2)
                            && u64::from(s.degree) < n,
                        "small_world",
                        "degree must be even, between 2 and 256, and below the number of agents",
                    ),
                    Substrate::Grid => {
                        let fits = self.grid_radius().is_some_and(|r| {
                            2 * r < self.lattice.width && 2 * r < self.lattice.height
                        });
                        check(
                            fits,
                            "small_world",
                            "on a grid the degree must be (2r + 1)² − 1 (8, 24, 48 …) with 2r below each side",
                        );
                    }
                }
            }
            Network::ScaleFree => {
                let m = self.scale_free.links;
                check(
                    (1..=8).contains(&m) && u64::from(m) + 2 <= n,
                    "scale_free",
                    "links must be between 1 and 8, with at least links + 2 agents",
                );
            }
            Network::All | Network::Lattice => {}
        }
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &AgreementConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("uncertainty", self.uncertainty == next.uncertainty),
            ("extremists", self.extremists == next.extremists),
            (
                "extremist_uncertainty",
                self.extremist_uncertainty == next.extremist_uncertainty,
            ),
            ("delta", self.delta == next.delta),
            ("placement", self.placement == next.placement),
            ("band", self.band == next.band),
            ("network", self.network == next.network),
            ("lattice", self.lattice == next.lattice),
            ("small_world", self.small_world == next.small_world),
            ("scale_free", self.scale_free == next.scale_free),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }

    /// Whether `next` changes how agents move (a stable run then resumes).
    pub(crate) fn moves_differently(&self, next: &AgreementConfig) -> bool {
        self.rule != next.rule
            || self.window != next.window
            || self.mu != next.mu
            || self.alpha != next.alpha
            || self.pair_update != next.pair_update
            || self.pairing != next.pairing
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Population", "agents", "Agents (N)", (2, MAX_AGENTS), Reset)
            .with_help("DAWF: 200 (Figs. 5–8) or 1000 (Fig. 9); Weisbuch: 900. On a lattice, or a small world grown on one, width × height instead."),
        Param::number(
            "Population",
            "uncertainty",
            "Moderates' uncertainty (U)",
            (0.05, 4.0, 0.05),
            Reset,
        )
        .with_help("Opinions run from −1 to 1. Deffuant 2000's threshold d on [0, 1] is U = 2d here."),
        Param::choice(
            "Interaction",
            "rule",
            "Rule",
            &[
                ("ra", "Relative agreement (DAWF)"),
                ("bc", "Bounded confidence (eq. 11)"),
                ("bc_averaging", "BC, averaging uncertainties (eq. 12)"),
                ("bc_variance", "BC, uncertainty from variance (eqs. 13–14)"),
            ],
            Live,
        ),
        Param::choice(
            "Interaction",
            "window",
            "BC window",
            &[
                ("influencer", "The influencer's uncertainty (eq. 11 as printed)"),
                ("listener", "The listener's own uncertainty"),
            ],
            Live,
        )
        .with_help("Eq. 11 reads |x − x′| < u′, the influencer's; §6's results need the listener's (measured). Unused by relative agreement."),
        Param::number("Interaction", "mu", "Speed (μ)", (0.0, 1.0, 0.01), Live),
        Param::number("Interaction", "alpha", "Memory (α)", (0.0, 1.0, 0.01), Live)
            .shown_if("rule", "bc_variance")
            .with_help("Eqs. 13–14's α; DAWF give no value (0.8 = 1 − μ here)."),
        Param::choice(
            "Interaction",
            "pair_update",
            "A meeting updates",
            &[
                ("simultaneous", "Both, from their old values"),
                ("sequential", "One, then the other"),
                ("one_way", "Only the first (Weisbuch)"),
            ],
            Live,
        )
        .with_help("DAWF do not say; melsimp.c and Meadows & Cliff update both from the old values."),
        Param::number(
            "Extremists",
            "extremists",
            "Extremists (pe)",
            (0.0, 1.0, 0.0125),
            Reset,
        ),
        Param::number(
            "Extremists",
            "extremist_uncertainty",
            "Their uncertainty (ue)",
            (0.01, 1.0, 0.01),
            Reset,
        ),
        Param::number("Extremists", "delta", "Lean (δ)", (0.0, 1.0, 0.05), Reset)
            .with_help("δ = |p₊ − p₋| / pe; the surplus goes to +1."),
        Param::choice(
            "Extremists",
            "placement",
            "Placement",
            &[
                ("drawn", "The most extreme draws (DAWF)"),
                ("bounds", "Set to ±1 (Amblard & Deffuant)"),
                ("band", "A band at the ends (Meadows & Cliff)"),
            ],
            Reset,
        ),
        Param::number("Extremists", "band", "Band edge (b)", (0.05, 0.95, 0.05), Reset)
            .shown_if("placement", "band"),
        Param::number(
            "Extremists",
            "extreme_margin",
            "New-extremist margin",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("y counts a moderate as a new extremist beyond its side's boundary minus this: 0.1 in Deffuant et al.'s 2013 reply, 0 in Meadows & Cliff."),
        Param::choice(
            "Network",
            "network",
            "Who meets whom",
            &[
                ("all", "Anyone"),
                ("lattice", "Lattice neighbors"),
                ("small_world", "A small world"),
                ("scale_free", "A scale-free network"),
            ],
            Reset,
        ),
        Param::integer("Network", "lattice.width", "Width", (3, MAX_SIDE), Reset)
            .with_help("Also the grid a small world can grow from."),
        Param::integer("Network", "lattice.height", "Height", (3, MAX_SIDE), Reset),
        Param::choice(
            "Network",
            "lattice.neighborhood",
            "Neighbors",
            &[("von_neumann", "4 (von Neumann)"), ("moore", "8 (Moore)")],
            Reset,
        )
        .shown_if("network", "lattice"),
        Param::choice(
            "Network",
            "small_world.substrate",
            "Grown from",
            &[("ring", "A ring"), ("grid", "The lattice")],
            Reset,
        )
        .shown_if("network", "small_world"),
        Param::integer(
            "Network",
            "small_world.degree",
            "Neighbors (k)",
            (2, 256),
            Reset,
        )
        .shown_if("network", "small_world")
        .with_help("Even on a ring; (2r + 1)² − 1 on the lattice (8, 24, 48 …)."),
        Param::number(
            "Network",
            "small_world.rewire",
            "Rewiring (p)",
            (0.0, 1.0, 0.05),
            Reset,
        )
        .shown_if("network", "small_world"),
        Param::integer(
            "Network",
            "scale_free.links",
            "Links per new agent (m)",
            (1, 8),
            Reset,
        )
        .shown_if("network", "scale_free"),
        Param::choice(
            "Network",
            "pairing",
            "Pairs",
            &[
                ("edge", "A random link (Deffuant, Amblard)"),
                ("node", "An agent, then a neighbor (Weisbuch)"),
            ],
            Live,
        )
        .with_help("Unused when anyone meets anyone."),
        Param::bool("Stopping", "stop_when_stable", "Stop when stable", Live)
            .with_help("Stable: no opinion or uncertainty moved more than 10⁻⁶ in a period of N meetings."),
        Param::integer("Stopping", "stop_at", "Stop at period", (0, 1_000_000), Live)
            .with_help("0: never. With the stop when stable, a cap; Meadows & Cliff stop at 200, Deffuant et al.'s reply at 1200."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_relative_agreement_at_fig_9s_speed() {
        let c = AgreementConfig::default();
        assert_eq!((c.agents, c.rule, c.mu), (200, Rule::Ra, 0.2));
        assert_eq!(
            (c.uncertainty, c.extremists, c.extremist_uncertainty),
            (1.0, 0.1, 0.1)
        );
        assert_eq!(
            (c.placement, c.extreme_margin, c.pair_update, c.network),
            (
                Placement::Drawn,
                0.1,
                PairUpdate::Simultaneous,
                Network::All
            )
        );
        assert!(c.stop_when_stable && c.stop_at == 20_000 && c.validate().is_ok());
        assert_eq!(c.population(), 200);
    }

    #[test]
    fn validation_names_fields() {
        let bad = AgreementConfig {
            agents: 1,
            mu: 1.5,
            alpha: -0.1,
            uncertainty: 0.0,
            extremists: 2.0,
            extremist_uncertainty: 0.0,
            delta: 1.5,
            band: 1.0,
            extreme_margin: -1.0,
            stop_at: 2_000_000,
            ..AgreementConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            [
                "agents",
                "mu",
                "alpha",
                "uncertainty",
                "extremists",
                "extremist_uncertainty",
                "delta",
                "band",
                "extreme_margin",
                "stop_at"
            ]
        );
    }

    #[test]
    fn networks_validate_their_own_fields() {
        let ring = |degree| AgreementConfig {
            network: Network::SmallWorld,
            small_world: SmallWorldConfig {
                degree,
                ..SmallWorldConfig::default()
            },
            ..AgreementConfig::default()
        };
        assert!(ring(8).validate().is_ok());
        for bad in [7, 0, 200, 258] {
            assert_eq!(
                ring(bad).validate().unwrap_err()[0].field,
                "small_world",
                "{bad}"
            );
        }
        let grid = |degree, side| AgreementConfig {
            network: Network::SmallWorld,
            lattice: LatticeConfig {
                width: side,
                height: side,
                ..LatticeConfig::default()
            },
            small_world: SmallWorldConfig {
                substrate: Substrate::Grid,
                degree,
                rewire: 0.1,
            },
            ..AgreementConfig::default()
        };
        assert_eq!(grid(24, 29).grid_radius(), Some(2));
        assert!(grid(24, 29).validate().is_ok());
        assert_eq!(
            grid(24, 29).population(),
            841,
            "the grid sets the population"
        );
        assert!(grid(10, 29).validate().is_err(), "not (2r + 1)² − 1");
        assert!(
            grid(48, 5).validate().is_err(),
            "radius 3 wraps a side of 5"
        );
        let lattice = AgreementConfig {
            network: Network::Lattice,
            lattice: LatticeConfig {
                width: 70_000,
                height: 70_000,
                ..LatticeConfig::default()
            },
            ..AgreementConfig::default()
        };
        assert_eq!(lattice.validate().unwrap_err()[0].field, "lattice");
        let sf = |links, agents| AgreementConfig {
            network: Network::ScaleFree,
            agents,
            scale_free: ScaleFreeConfig { links },
            ..AgreementConfig::default()
        };
        assert!(sf(2, 900).validate().is_ok());
        assert!(sf(0, 900).validate().is_err());
        assert!(sf(8, 9).validate().is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Agreement(AgreementConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

Create `crates/sugarscape-core/src/agreement/network.rs` with exactly this content:

```rust
//! Who can meet whom: a torus lattice (DNAW §3, AD §3.1), a Watts–Strogatz
//! small world grown from a ring or the torus (AD §3.2), or a Barabási–Albert
//! network (Weisbuch). Built once, from the seed, before any opinion is drawn.

use rand::Rng;

use super::config::{AgreementConfig, Network, Substrate};
use crate::opinions::Neighborhood;
use crate::rng::SimRng;

/// Every agent's neighbors (in the order links were made) and the links
/// themselves (lower index first). Empty when anyone meets anyone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Graph {
    start: Vec<u32>,
    list: Vec<u32>,
    edges: Vec<(u32, u32)>,
}

impl Graph {
    pub fn new(c: &AgreementConfig, rng: &mut SimRng) -> Self {
        let n = c.population();
        let adj = match c.network {
            Network::All => return Graph::default(),
            Network::Lattice => {
                let r = 1;
                torus(c, r, c.lattice.neighborhood == Neighborhood::VonNeumann)
            }
            Network::SmallWorld => {
                let mut adj = match c.small_world.substrate {
                    Substrate::Ring => ring(n, c.small_world.degree as usize),
                    Substrate::Grid => torus(c, c.grid_radius().expect("validated"), false),
                };
                rewire(&mut adj, c.small_world.rewire, rng);
                adj
            }
            Network::ScaleFree => barabasi_albert(n, c.scale_free.links as usize, rng),
        };
        Graph::from_lists(adj)
    }

    fn from_lists(adj: Vec<Vec<u32>>) -> Self {
        let mut start = Vec::with_capacity(adj.len() + 1);
        let mut list = Vec::new();
        let mut edges = Vec::new();
        for (a, ns) in adj.iter().enumerate() {
            start.push(list.len() as u32);
            list.extend_from_slice(ns);
            edges.extend(
                ns.iter()
                    .filter(|&&b| a < b as usize)
                    .map(|&b| (a as u32, b)),
            );
        }
        start.push(list.len() as u32);
        Graph { start, list, edges }
    }

    /// Whether anyone meets anyone (no graph).
    pub fn is_complete(&self) -> bool {
        self.start.is_empty()
    }

    pub fn of(&self, i: usize) -> &[u32] {
        &self.list[self.start[i] as usize..self.start[i + 1] as usize]
    }

    pub fn edges(&self) -> &[(u32, u32)] {
        &self.edges
    }
}

/// The torus's neighbors within Moore radius `r` (only the four nearest
/// with `von_neumann`), in row-major order of offset.
fn torus(c: &AgreementConfig, r: u32, von_neumann: bool) -> Vec<Vec<u32>> {
    let (w, h) = (c.lattice.width as i64, c.lattice.height as i64);
    let r = r as i64;
    let mut adj = vec![Vec::new(); (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let a = (y * w + x) as usize;
            for dy in -r..=r {
                for dx in -r..=r {
                    if (dx, dy) == (0, 0) || (von_neumann && dx.abs() + dy.abs() != 1) {
                        continue;
                    }
                    let b = (y + dy).rem_euclid(h) * w + (x + dx).rem_euclid(w);
                    adj[a].push(b as u32);
                }
            }
        }
    }
    adj
}

/// A ring: each agent linked to the k/2 nearest on each side.
fn ring(n: usize, k: usize) -> Vec<Vec<u32>> {
    let mut adj = vec![Vec::new(); n];
    for i in 0..n {
        for j in 1..=k / 2 {
            let b = (i + j) % n;
            adj[i].push(b as u32);
            adj[b].push(i as u32);
        }
    }
    adj
}

/// Watts–Strogatz: each link (a, b), a < b, in order of a then of a's list,
/// is with probability p replaced by (a, c) for a uniform c that is not a
/// and not already a's neighbor.
fn rewire(adj: &mut [Vec<u32>], p: f64, rng: &mut SimRng) {
    let n = adj.len() as u32;
    let links: Vec<(u32, u32)> = adj
        .iter()
        .enumerate()
        .flat_map(|(a, ns)| {
            ns.iter()
                .filter(move |&&b| (a as u32) < b)
                .map(move |&b| (a as u32, b))
        })
        .collect();
    for (a, b) in links {
        if rng.gen::<f64>() >= p || adj[a as usize].len() as u32 >= n - 1 {
            continue;
        }
        let c = loop {
            let c = rng.gen_range(0..n);
            if c != a && !adj[a as usize].contains(&c) {
                break c;
            }
        };
        adj[a as usize].retain(|&x| x != b);
        adj[b as usize].retain(|&x| x != a);
        adj[a as usize].push(c);
        adj[c as usize].push(a);
    }
}

/// Barabási–Albert: a complete graph on max(3, m + 1) agents, then each new
/// agent linked to m distinct earlier ones chosen in proportion to degree.
fn barabasi_albert(n: usize, m: usize, rng: &mut SimRng) -> Vec<Vec<u32>> {
    let seed = 3.max(m + 1).min(n);
    let mut adj = vec![Vec::new(); n];
    // Every link's two ends: a uniform pick is a pick by degree.
    let mut ends: Vec<u32> = Vec::new();
    for a in 0..seed {
        for b in a + 1..seed {
            adj[a].push(b as u32);
            adj[b].push(a as u32);
            ends.extend([a as u32, b as u32]);
        }
    }
    for v in seed..n {
        let mut targets: Vec<u32> = Vec::with_capacity(m);
        while targets.len() < m {
            let t = ends[rng.gen_range(0..ends.len() as u32) as usize];
            if !targets.contains(&t) {
                targets.push(t);
            }
        }
        for t in targets {
            adj[v].push(t);
            adj[t as usize].push(v as u32);
            ends.extend([v as u32, t]);
        }
    }
    adj
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agreement::config::{LatticeConfig, ScaleFreeConfig, SmallWorldConfig};
    use crate::rng;

    fn graph(edit: impl FnOnce(&mut AgreementConfig)) -> (AgreementConfig, Graph) {
        let mut c = AgreementConfig::default();
        edit(&mut c);
        c.validate().unwrap();
        let g = Graph::new(&c, &mut rng::seeded(1));
        (c, g)
    }

    fn simple(g: &Graph, n: usize) {
        for i in 0..n {
            let ns = g.of(i);
            assert!(!ns.contains(&(i as u32)), "no loop at {i}");
            let mut sorted = ns.to_vec();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), ns.len(), "no double link at {i}");
            for &j in ns {
                assert!(g.of(j as usize).contains(&(i as u32)), "{i}–{j} both ways");
            }
        }
    }

    #[test]
    fn anyone_meets_anyone_without_a_graph() {
        let (_, g) = graph(|_| {});
        assert!(g.is_complete() && g.edges().is_empty());
    }

    #[test]
    fn lattices_wrap_with_four_or_eight_neighbors() {
        let (c, g) = graph(|c| c.network = Network::Lattice);
        assert_eq!(c.population(), 841);
        assert_eq!(g.of(0), [812, 28, 1, 29], "von Neumann, wrapped");
        assert_eq!(g.edges().len(), 841 * 2);
        simple(&g, 841);
        let (_, m) = graph(|c| {
            c.network = Network::Lattice;
            c.lattice.neighborhood = Neighborhood::Moore;
        });
        assert!((0..841).all(|i| m.of(i).len() == 8));
        simple(&m, 841);
    }

    #[test]
    fn rings_rewire_into_small_worlds() {
        let ring = |p| {
            graph(|c| {
                c.agents = 100;
                c.network = Network::SmallWorld;
                c.small_world = SmallWorldConfig {
                    degree: 6,
                    rewire: p,
                    ..SmallWorldConfig::default()
                };
            })
            .1
        };
        let regular = ring(0.0);
        assert_eq!(regular.of(0), [1, 2, 3, 97, 98, 99]);
        assert_eq!(regular.edges().len(), 300);
        let small = ring(0.3);
        assert_eq!(
            small.edges().len(),
            300,
            "rewiring keeps the number of links"
        );
        simple(&small, 100);
        assert_ne!(small, regular);
        let random = ring(1.0);
        simple(&random, 100);
    }

    #[test]
    fn a_grid_substrate_uses_a_wider_moore_neighborhood() {
        let (c, g) = graph(|c| {
            c.network = Network::SmallWorld;
            c.lattice = LatticeConfig {
                width: 10,
                height: 10,
                ..LatticeConfig::default()
            };
            c.small_world = SmallWorldConfig {
                substrate: Substrate::Grid,
                degree: 24,
                rewire: 0.0,
            };
        });
        assert_eq!(c.population(), 100);
        assert!((0..100).all(|i| g.of(i).len() == 24));
        simple(&g, 100);
    }

    #[test]
    fn scale_free_networks_grow_by_preference() {
        let (_, g) = graph(|c| {
            c.agents = 900;
            c.network = Network::ScaleFree;
        });
        simple(&g, 900);
        assert_eq!(
            g.edges().len(),
            3 + 897 * 2,
            "a triangle, then two links each"
        );
        let max = (0..900).map(|i| g.of(i).len()).max().unwrap();
        assert!(max > 30, "hubs: the largest degree is {max}");
        assert!((3..900).all(|i| g.of(i).len() >= 2));
        let (_, four) = graph(|c| {
            c.agents = 100;
            c.network = Network::ScaleFree;
            c.scale_free = ScaleFreeConfig { links: 4 };
        });
        assert_eq!(four.edges().len(), 10 + 95 * 4, "K5, then four links each");
        simple(&four, 100);
    }

    #[test]
    fn graphs_are_deterministic_from_the_seed() {
        let c = AgreementConfig {
            network: Network::ScaleFree,
            ..AgreementConfig::default()
        };
        let a = Graph::new(&c, &mut rng::seeded(4));
        let b = Graph::new(&c, &mut rng::seeded(4));
        let d = Graph::new(&c, &mut rng::seeded(5));
        assert_eq!(a, b);
        assert_ne!(a, d);
    }
}
```

Create `crates/sugarscape-core/src/agreement/stats.rs` with exactly this content:

```rust
//! The Relative Agreement model's statistics: Deffuant et al.'s indicator y
//! and the outcome it stands for, clusters with and without DNAW's wings,
//! Weisbuch's dispersion and unmoved agents, and stability.

use serde::Serialize;

use crate::opinions::groups;
use crate::stats::Series;

/// A period in which no opinion or uncertainty moved more than this is
/// stable (planning: 10⁻⁴, 10⁻⁶ and 10⁻⁸ give the same y).
pub const STILL: f64 = 1e-6;

/// Sorted opinions this close are one cluster: pairwise meetings close
/// clusters only geometrically.
pub const GAP: f64 = 1e-3;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 15] = [
    "y",
    "p_plus",
    "p_minus",
    "outcome",
    "clusters",
    "major",
    "isolated",
    "largest",
    "second",
    "dispersion",
    "unmoved",
    "mean_opinion",
    "mean_uncertainty",
    "max_change",
    "stable_at",
];

/// How a run ended up, from p′₊ and p′₋.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Both below 0.15.
    Central = 0,
    /// Both at least 0.25.
    BothExtremes = 1,
    /// One at least 0.7, the other below 0.1.
    SingleExtreme = 2,
    Intermediate = 3,
}

impl Outcome {
    pub fn of(p_plus: f64, p_minus: f64) -> Self {
        let (hi, lo) = (p_plus.max(p_minus), p_plus.min(p_minus));
        if hi < 0.15 {
            Outcome::Central
        } else if lo >= 0.25 {
            Outcome::BothExtremes
        } else if hi >= 0.7 && lo < 0.1 {
            Outcome::SingleExtreme
        } else {
            Outcome::Intermediate
        }
    }
}

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct AgreementSnapshot {
    pub tick: u64,
    /// DAWF's indicator y = p′₊² + p′₋².
    pub y: f64,
    /// Shares of the initial moderates that became extremists on each side
    /// (0 on a side without extremists).
    pub p_plus: f64,
    pub p_minus: f64,
    /// 0 central, 1 both extremes, 2 single extreme, 3 intermediate.
    pub outcome: u32,
    /// Groups of at least two sorted opinions with gaps at most `GAP`.
    pub clusters: u32,
    /// Groups holding at least 1 % of agents (and two): DNAW's peaks
    /// without the wings.
    pub major: u32,
    /// Groups of one.
    pub isolated: u32,
    /// Shares of agents in the biggest and second-biggest groups.
    pub largest: f64,
    pub second: f64,
    /// Weisbuch's Σ sᵢ² / N² over every group.
    pub dispersion: f64,
    /// The share of agents whose opinion never changed.
    pub unmoved: f64,
    pub mean_opinion: f64,
    pub mean_uncertainty: f64,
    /// The largest move of an opinion or uncertainty this period.
    pub max_change: f64,
    /// The first stable period, else this one.
    pub stable_at: u64,
}

impl Series for AgreementSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "y" => self.y,
            "p_plus" => self.p_plus,
            "p_minus" => self.p_minus,
            "outcome" => f64::from(self.outcome),
            "clusters" => f64::from(self.clusters),
            "major" => f64::from(self.major),
            "isolated" => f64::from(self.isolated),
            "largest" => self.largest,
            "second" => self.second,
            "dispersion" => self.dispersion,
            "unmoved" => self.unmoved,
            "mean_opinion" => self.mean_opinion,
            "mean_uncertainty" => self.mean_uncertainty,
            "max_change" => self.max_change,
            "stable_at" => self.stable_at as f64,
            _ => return None,
        })
    }
}

/// The group measures of a sorted profile: clusters, major, isolated,
/// largest and second shares, dispersion.
pub fn grouping(sorted: &[f64]) -> (u32, u32, u32, f64, f64, f64) {
    let n = sorted.len();
    let mut sizes = groups(sorted, GAP);
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    let major_at = (0.01 * n as f64).max(2.0);
    let share = |k: Option<&usize>| k.map_or(0.0, |&s| s as f64 / n as f64);
    (
        sizes.iter().filter(|&&s| s >= 2).count() as u32,
        sizes.iter().filter(|&&s| s as f64 >= major_at).count() as u32,
        sizes.iter().filter(|&&s| s == 1).count() as u32,
        share(sizes.first()),
        share(sizes.get(1)),
        sizes.iter().map(|&s| (s * s) as f64).sum::<f64>() / (n * n) as f64,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_read_both_shares() {
        assert_eq!(Outcome::of(0.0, 0.04), Outcome::Central);
        assert_eq!(Outcome::of(0.43, 0.56), Outcome::BothExtremes);
        assert_eq!(Outcome::of(0.0, 0.9833), Outcome::SingleExtreme);
        // y = 0.49 looks like both extremes; the shares say single (M&C ¶4.10).
        assert_eq!(Outcome::of(0.7, 0.0), Outcome::SingleExtreme);
        assert_eq!(Outcome::of(0.3, 0.1), Outcome::Intermediate);
    }

    #[test]
    fn grouping_counts_clusters_wings_and_dispersion() {
        let mut p = vec![0.0; 150];
        p.extend([0.5; 48]);
        p.extend([0.9, 0.95]);
        p.sort_by(f64::total_cmp);
        let (clusters, major, isolated, largest, second, dispersion) = grouping(&p);
        assert_eq!((clusters, major, isolated), (2, 2, 2));
        assert_eq!((largest, second), (0.75, 0.24));
        let want = (150.0f64.powi(2) + 48.0f64.powi(2) + 2.0) / 200.0f64.powi(2);
        assert!((dispersion - want).abs() < 1e-12);
        let (c, m, i, l, _, d) = grouping(&[0.2, 0.2 + 5e-4, 0.2 + 1e-3]);
        assert_eq!((c, m, i, l, d), (1, 1, 0, 1.0, 1.0), "gaps up to 10⁻³ join");
    }
}
```

Create `crates/sugarscape-core/src/agreement/view.rs` with exactly this content:

```rust
//! The frame: opinion × time (DAWF's Figs. 3, 5–8), start against now
//! (DNAW Fig. 3, Weisbuch Figs. 4–5) and, on a torus, the lattice.

use crate::render::{lerp, Rgb};

/// Diagram columns: at most 240 kept periods and the current one.
pub const COLUMNS: usize = 241;
/// Periods the history keeps before it halves.
pub const KEPT: usize = COLUMNS - 1;
/// The diagram's and the panels' height; +1 at the top, −1 at the bottom.
pub const TALL: usize = 201;
/// Cells between panels.
pub const GAP: usize = 8;
/// Where the start-against-now panel starts; it is `TALL` square.
pub const SCATTER_X: usize = COLUMNS + GAP;
/// Where the torus starts, when there is one.
pub const TORUS_X: usize = SCATTER_X + TALL + GAP;

/// The gray of the scatter's diagonal (agents that never moved).
pub const DIAGONAL: Rgb = [0x3a, 0x38, 0x33];
/// Confident (small uncertainty) and uncertain, as in DAWF's figures.
pub const CONFIDENT: Rgb = [0xe0, 0x3c, 0x31];
pub const UNCERTAIN: Rgb = [0x4c, 0xc2, 0x5a];
/// The initial extremists under Role.
pub const PLUS: Rgb = [0xff, 0xc8, 0x3c];
pub const MINUS: Rgb = [0x4a, 0x9c, 0xff];
/// Moderates under Role, by opinion from −1 to +1.
pub const MODERATE_LOW: Rgb = [0x5c, 0x6a, 0x80];
pub const MODERATE_HIGH: Rgb = [0x80, 0x74, 0x5c];

/// The row of opinion `x`: +1 at the top, −1 at the bottom.
pub fn row(x: f64) -> usize {
    ((1.0 - x.clamp(-1.0, 1.0)) / 2.0 * (TALL - 1) as f64).round() as usize
}

/// The opinion at row `y`.
pub fn opinion_at(y: usize) -> f64 {
    1.0 - 2.0 * y as f64 / (TALL - 1) as f64
}

/// The scatter's column for a starting opinion: −1 at the left.
pub fn scatter_col(x: f64) -> usize {
    ((x.clamp(-1.0, 1.0) + 1.0) / 2.0 * (TALL - 1) as f64).round() as usize
}

/// Uncertainty `u` on a scale topping out at `top`.
pub fn uncertainty_color(u: f64, top: f64) -> Rgb {
    lerp(CONFIDENT, UNCERTAIN, (u / top).clamp(0.0, 1.0))
}

/// Opinion `x` on the Bounded Confidence model's spectrum (red at −1,
/// magenta at +1).
pub fn opinion_color(x: f64) -> Rgb {
    crate::opinions::hue((x.clamp(-1.0, 1.0) + 1.0) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_run_from_plus_one_at_the_top() {
        assert_eq!((row(1.0), row(-1.0), row(0.0)), (0, TALL - 1, 100));
        assert!((opinion_at(row(0.37)) - 0.37).abs() <= 1.0 / (TALL - 1) as f64);
        assert_eq!((scatter_col(-1.0), scatter_col(1.0)), (0, TALL - 1));
    }

    #[test]
    fn panels_sit_side_by_side() {
        assert_eq!((SCATTER_X, TORUS_X), (249, 458));
        assert_eq!(uncertainty_color(0.0, 1.0), CONFIDENT);
        assert_eq!(uncertainty_color(2.0, 1.0), UNCERTAIN);
        assert_eq!(opinion_color(-1.0), crate::opinions::hue(0.0));
    }
}
```

Create `crates/sugarscape-core/src/agreement/world.rs` with exactly this content:

```rust
//! The relative agreement world: random pairs meet, N meetings a period,
//! and move each other's opinions and uncertainties by relative agreement
//! (DAWF eqs. 1–6) or one of the bounded-confidence rules of §6.

use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{AgreementConfig, PairUpdate, Pairing, Placement, Rule, Window};
use super::network::Graph;
use super::stats::{grouping, AgreementSnapshot, Outcome, STILL};
use super::view::{
    opinion_at, opinion_color, row, scatter_col, uncertainty_color, COLUMNS, DIAGONAL, KEPT, MINUS,
    MODERATE_HIGH, MODERATE_LOW, PLUS, SCATTER_X, TALL, TORUS_X,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::{site_cells, Canvas};
use crate::render::{lerp, Rgb};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// Where an agent started: an extremist on either side, or a moderate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Plus,
    Minus,
    Moderate,
}

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgreementMode {
    /// Confident red to uncertain green (DAWF's figures).
    Uncertainty,
    /// The initial extremists by side; moderates by opinion.
    Role,
    /// By starting opinion.
    Start,
}

impl std::str::FromStr for AgreementMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "uncertainty" => Self::Uncertainty,
            "role" => Self::Role,
            "start" => Self::Start,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgreementInspection {
    pub site: AgreementCell,
    /// `diagram`, `scatter` or `torus`; null between panels.
    pub panel: Option<&'static str>,
    /// The period of the diagram column (the current one on the panels).
    pub period: Option<u64>,
    /// The opinion at the clicked row, null on the torus.
    pub opinion: Option<f64>,
    /// The agents within one cell, or the torus site's agent.
    pub agents: Vec<AgreementAgent>,
    /// Always null: a clicked cell is read again where it is each period
    /// (a line or dot moves, and a tracked agent's dot is usually crowded).
    pub agent: Option<AgreementAgent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct AgreementCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgreementAgent {
    pub id: u64,
    pub role: Role,
    pub start: f64,
    /// The opinion and uncertainty in the inspected period.
    pub opinion: f64,
    pub uncertainty: f64,
    /// Neighbors (N − 1 when anyone meets anyone).
    pub degree: u32,
    /// Meetings so far, and how many changed its opinion.
    pub meetings: u32,
    pub moves: u32,
}

/// One kept period of the diagram.
#[derive(Clone, Debug)]
struct Column {
    period: u64,
    x: Arc<Vec<f64>>,
    u: Arc<Vec<f64>>,
}

/// Agent j's opinion and uncertainty after agent i's influence, or `None`
/// when i has none on j.
pub fn influence(
    c: &AgreementConfig,
    (xi, ui): (f64, f64),
    (xj, uj): (f64, f64),
) -> Option<(f64, f64)> {
    let mu = c.mu;
    let window = || match c.window {
        Window::Influencer => ui,
        Window::Listener => uj,
    };
    match c.rule {
        Rule::Ra => {
            let h = (xi + ui).min(xj + uj) - (xi - ui).max(xj - uj);
            (h > ui).then(|| {
                let ra = h / ui - 1.0;
                (xj + mu * ra * (xi - xj), uj + mu * ra * (ui - uj))
            })
        }
        Rule::Bc => ((xi - xj).abs() < window()).then_some((xj + mu * (xi - xj), uj)),
        Rule::BcAveraging => {
            ((xi - xj).abs() < window()).then_some((xj + mu * (xi - xj), uj + mu * (ui - uj)))
        }
        Rule::BcVariance => ((xi - xj).abs() < window()).then(|| {
            let a = c.alpha;
            let d = xj - xi;
            (
                a * xj + (1.0 - a) * xi,
                (a * uj * uj + a * (1.0 - a) * d * d).sqrt(),
            )
        }),
    }
}

/// How many extremists start on each side: nₑ = round(N·pe),
/// n₊ = round(nₑ(1 + δ)/2) with a half broken by a coin flip, n₋ = nₑ − n₊.
pub fn extremist_counts(c: &AgreementConfig, n: usize, rng: &mut SimRng) -> (usize, usize) {
    let total = ((n as f64 * c.extremists).round() as usize).min(n);
    let half = total as f64 * (1.0 + c.delta) / 2.0;
    let plus = if (half - half.floor() - 0.5).abs() < 1e-9 {
        if rng.gen::<bool>() {
            half.ceil()
        } else {
            half.floor()
        }
    } else {
        half.round()
    } as usize;
    let plus = plus.min(total);
    (plus, total - plus)
}

#[derive(Clone)]
pub struct AgreementWorld {
    pub config: AgreementConfig,
    /// Completed periods.
    pub tick: u64,
    start: Vec<f64>,
    x: Vec<f64>,
    u: Vec<f64>,
    role: Vec<Role>,
    meetings: Vec<u32>,
    moves: Vec<u32>,
    graph: Arc<Graph>,
    rng: SimRng,
    /// The boundary a moderate must pass, less the margin, to count as a new
    /// extremist on each side; none on a side without extremists.
    plus_bound: Option<f64>,
    minus_bound: Option<f64>,
    /// Kept periods, oldest first: every `interval`-th, at most `KEPT`.
    history: Vec<Column>,
    interval: u64,
    max_change: f64,
    stable_at: Option<u64>,
    pub stats: Stats<AgreementSnapshot>,
}

impl AgreementWorld {
    pub fn new(config: AgreementConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let graph = Arc::new(Graph::new(&config, &mut rng));
        let n = config.population();
        let (plus, minus) = extremist_counts(&config, n, &mut rng);
        let mut x = vec![0.0f64; n];
        let mut role = vec![Role::Moderate; n];
        match config.placement {
            Placement::Drawn | Placement::Bounds => {
                for v in x.iter_mut() {
                    *v = rng.gen_range(-1.0..1.0);
                }
                let mut order: Vec<usize> = (0..n).collect();
                order.sort_by(|&a, &b| x[a].total_cmp(&x[b]).then(a.cmp(&b)));
                for &i in &order[..minus] {
                    role[i] = Role::Minus;
                }
                for &i in &order[n - plus..] {
                    role[i] = Role::Plus;
                }
                if config.placement == Placement::Bounds {
                    for i in 0..n {
                        match role[i] {
                            Role::Plus => x[i] = 1.0,
                            Role::Minus => x[i] = -1.0,
                            Role::Moderate => {}
                        }
                    }
                }
            }
            Placement::Band => {
                let b = config.band;
                let mut order: Vec<usize> = (0..n).collect();
                order.shuffle(&mut rng);
                for (k, &i) in order.iter().enumerate() {
                    if k < plus {
                        role[i] = Role::Plus;
                        x[i] = rng.gen_range(b..1.0);
                    } else if k < plus + minus {
                        role[i] = Role::Minus;
                        x[i] = -rng.gen_range(b..1.0);
                    } else {
                        x[i] = rng.gen_range(-b..b);
                    }
                }
            }
        }
        let u: Vec<f64> = role
            .iter()
            .map(|r| match r {
                Role::Moderate => config.uncertainty,
                _ => config.extremist_uncertainty,
            })
            .collect();
        let innermost = |side: Role, inner: fn(f64, f64) -> f64, bound: f64| {
            let mut it = (0..n).filter(|&i| role[i] == side).map(|i| x[i]);
            let first = it.next()?;
            Some(match config.placement {
                Placement::Drawn => it.fold(first, inner),
                Placement::Bounds => bound,
                Placement::Band => bound * config.band,
            })
        };
        let plus_bound = innermost(Role::Plus, f64::min, 1.0);
        let minus_bound = innermost(Role::Minus, f64::max, -1.0);
        let mut world = AgreementWorld {
            history: vec![Column {
                period: 0,
                x: Arc::new(x.clone()),
                u: Arc::new(u.clone()),
            }],
            config,
            tick: 0,
            start: x.clone(),
            x,
            u,
            role,
            meetings: vec![0; n],
            moves: vec![0; n],
            graph,
            rng,
            plus_bound,
            minus_bound,
            interval: 1,
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

    pub fn uncertainties(&self) -> &[f64] {
        &self.u
    }

    pub fn starts(&self) -> &[f64] {
        &self.start
    }

    pub fn roles(&self) -> &[Role] {
        &self.role
    }

    /// Whether the run has stopped: stable with `stop_when_stable`, or at `stop_at`.
    pub fn is_finished(&self) -> bool {
        (self.config.stop_when_stable && self.stable_at.is_some())
            || (self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at))
    }

    fn degree(&self, i: usize) -> u32 {
        if self.graph.is_complete() {
            self.x.len() as u32 - 1
        } else {
            self.graph.of(i).len() as u32
        }
    }

    /// The next meeting's pair, first agent first; `None` when the drawn
    /// agent has no neighbor.
    fn pair(&mut self) -> Option<(usize, usize)> {
        let n = self.x.len() as u32;
        let rng = &mut self.rng;
        if self.graph.is_complete() {
            let a = rng.gen_range(0..n);
            let mut b = rng.gen_range(0..n - 1);
            if b >= a {
                b += 1;
            }
            return Some((a as usize, b as usize));
        }
        match self.config.pairing {
            Pairing::Edge => {
                let edges = self.graph.edges();
                if edges.is_empty() {
                    return None;
                }
                let (p, q) = edges[rng.gen_range(0..edges.len() as u32) as usize];
                let (a, b) = if rng.gen::<bool>() { (p, q) } else { (q, p) };
                Some((a as usize, b as usize))
            }
            Pairing::Node => {
                let a = rng.gen_range(0..n) as usize;
                let ns = self.graph.of(a);
                if ns.is_empty() {
                    return None;
                }
                Some((a, ns[rng.gen_range(0..ns.len() as u32) as usize] as usize))
            }
        }
    }

    fn apply(&mut self, i: usize, next: Option<(f64, f64)>) {
        if let Some((nx, nu)) = next {
            let change = (nx - self.x[i]).abs().max((nu - self.u[i]).abs());
            self.max_change = self.max_change.max(change);
            if nx != self.x[i] {
                self.moves[i] += 1;
            }
            self.x[i] = nx;
            self.u[i] = nu;
        }
    }

    /// One meeting of `a` (first) and `b`.
    fn meet(&mut self, a: usize, b: usize) {
        self.meetings[a] += 1;
        self.meetings[b] += 1;
        let c = &self.config;
        let (pa, pb) = ((self.x[a], self.u[a]), (self.x[b], self.u[b]));
        match c.pair_update {
            PairUpdate::Simultaneous => {
                let (nb, na) = (influence(c, pa, pb), influence(c, pb, pa));
                self.apply(b, nb);
                self.apply(a, na);
            }
            PairUpdate::Sequential => {
                let nb = influence(c, pa, pb);
                self.apply(b, nb);
                let na = influence(&self.config, (self.x[b], self.u[b]), pa);
                self.apply(a, na);
            }
            PairUpdate::OneWay => {
                let na = influence(c, pb, pa);
                self.apply(a, na);
            }
        }
    }

    /// One period: N meetings.
    pub fn step(&mut self) {
        self.max_change = 0.0;
        for _ in 0..self.x.len() {
            if let Some((a, b)) = self.pair() {
                self.meet(a, b);
            }
        }
        self.tick += 1;
        if self.max_change > STILL {
            self.stable_at = None;
        }
        if self.stable_at.is_none() && self.max_change <= STILL {
            self.stable_at = Some(self.tick);
        }
        self.keep();
        self.record();
    }

    /// Keeps this period for the diagram if it falls on the interval,
    /// halving the history first when it is full.
    fn keep(&mut self) {
        if !self.tick.is_multiple_of(self.interval) {
            return;
        }
        if self.history.len() == KEPT {
            let every = 2 * self.interval;
            self.history.retain(|col| col.period % every == 0);
            self.interval = every;
            if !self.tick.is_multiple_of(self.interval) {
                return;
            }
        }
        self.history.push(Column {
            period: self.tick,
            x: Arc::new(self.x.clone()),
            u: Arc::new(self.u.clone()),
        });
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// Shares of the initial moderates past each side's boundary less the margin.
    fn new_extremists(&self) -> (f64, f64) {
        let moderates = self.role.iter().filter(|&&r| r == Role::Moderate).count();
        if moderates == 0 {
            return (0.0, 0.0);
        }
        let m = self.config.extreme_margin;
        let share = |past: &dyn Fn(f64) -> bool| {
            (0..self.x.len())
                .filter(|&i| self.role[i] == Role::Moderate && past(self.x[i]))
                .count() as f64
                / moderates as f64
        };
        (
            self.plus_bound.map_or(0.0, |b| share(&|x| x > b - m)),
            self.minus_bound.map_or(0.0, |b| share(&|x| x < b + m)),
        )
    }

    fn record(&mut self) {
        let n = self.x.len() as f64;
        let mut sorted = self.x.clone();
        sorted.sort_by(f64::total_cmp);
        let (clusters, major, isolated, largest, second, dispersion) = grouping(&sorted);
        let (p_plus, p_minus) = self.new_extremists();
        self.stats.push(AgreementSnapshot {
            tick: self.tick,
            y: p_plus * p_plus + p_minus * p_minus,
            p_plus,
            p_minus,
            outcome: Outcome::of(p_plus, p_minus) as u32,
            clusters,
            major,
            isolated,
            largest,
            second,
            dispersion,
            unmoved: self.moves.iter().filter(|&&m| m == 0).count() as f64 / n,
            mean_opinion: self.x.iter().sum::<f64>() / n,
            mean_uncertainty: self.u.iter().sum::<f64>() / n,
            max_change: self.max_change,
            stable_at: self.stable_at.unwrap_or(self.tick),
        });
    }

    /// The diagram's columns: the kept periods, then the current one if it
    /// was not kept.
    fn columns(&self) -> Vec<Column> {
        let mut cols = self.history.clone();
        if cols.last().map(|c| c.period) != Some(self.tick) {
            cols.push(Column {
                period: self.tick,
                x: Arc::new(self.x.clone()),
                u: Arc::new(self.u.clone()),
            });
        }
        cols
    }

    fn color(&self, mode: AgreementMode, i: usize, x: f64, u: f64) -> Rgb {
        let top = self
            .config
            .uncertainty
            .max(self.config.extremist_uncertainty);
        match mode {
            AgreementMode::Uncertainty => uncertainty_color(u, top),
            AgreementMode::Start => opinion_color(self.start[i]),
            AgreementMode::Role => match self.role[i] {
                Role::Plus => PLUS,
                Role::Minus => MINUS,
                Role::Moderate => lerp(MODERATE_LOW, MODERATE_HIGH, (x + 1.0) / 2.0),
            },
        }
    }

    fn view(&self, i: usize, x: f64, u: f64) -> AgreementAgent {
        AgreementAgent {
            id: i as u64 + 1,
            role: self.role[i],
            start: self.start[i],
            opinion: x,
            uncertainty: u,
            degree: self.degree(i),
            meetings: self.meetings[i],
            moves: self.moves[i],
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<AgreementInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let mut out = AgreementInspection {
            site: AgreementCell { x, y },
            panel: None,
            period: None,
            opinion: None,
            agents: Vec::new(),
            agent: None,
        };
        if cx < COLUMNS {
            let cols = self.columns();
            if let Some(col) = cols.get(cx) {
                out.panel = Some("diagram");
                out.period = Some(col.period);
                out.opinion = Some(opinion_at(cy));
                out.agents = (0..col.x.len())
                    .filter(|&i| row(col.x[i]).abs_diff(cy) <= 1)
                    .map(|i| self.view(i, col.x[i], col.u[i]))
                    .collect();
            }
        } else if (SCATTER_X..SCATTER_X + TALL).contains(&cx) {
            let sx = cx - SCATTER_X;
            out.panel = Some("scatter");
            out.period = Some(self.tick);
            out.opinion = Some(opinion_at(cy));
            out.agents = (0..self.x.len())
                .filter(|&i| {
                    scatter_col(self.start[i]).abs_diff(sx) <= 1 && row(self.x[i]).abs_diff(cy) <= 1
                })
                .map(|i| self.view(i, self.x[i], self.u[i]))
                .collect();
        } else if self.config.on_torus() && cx >= TORUS_X {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            let (sx, sy) = ((cx - TORUS_X) / s, cy / s);
            if sx < l.width as usize && sy < l.height as usize {
                let i = sy * l.width as usize + sx;
                out.panel = Some("torus");
                out.period = Some(self.tick);
                out.agents = vec![self.view(i, self.x[i], self.u[i])];
            }
        }
        Ok(out)
    }
}

impl Model for AgreementWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Agreement(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        AgreementWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.x.len()
    }

    /// FNV-1a over the tick and every opinion's and uncertainty's bits.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: [u8; 8]| {
            for b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick.to_le_bytes());
        for (x, u) in self.x.iter().zip(&self.u) {
            eat(x.to_bits().to_le_bytes());
            eat(u.to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let w = if self.config.on_torus() {
            TORUS_X + TALL
        } else {
            SCATTER_X + TALL
        };
        (w as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: AgreementMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        // Extremists last, so they are drawn over the moderates.
        let mut order: Vec<usize> = (0..self.x.len()).collect();
        order.sort_by(|&a, &b| {
            (self.role[a] != Role::Moderate)
                .cmp(&(self.role[b] != Role::Moderate))
                .then(self.start[a].total_cmp(&self.start[b]))
        });
        let cols = self.columns();
        for &i in &order {
            for (k, col) in cols.iter().enumerate() {
                let color = self.color(mode, i, col.x[i], col.u[i]);
                match cols.get(k + 1) {
                    Some(next) => c.column(k, row(col.x[i]), row(next.x[i]), color),
                    None => c.put(k, row(col.x[i]), color),
                }
            }
        }
        for k in 0..TALL {
            c.put(SCATTER_X + k, TALL - 1 - k, DIAGONAL);
        }
        for &i in &order {
            let color = self.color(mode, i, self.x[i], self.u[i]);
            c.put(
                SCATTER_X + scatter_col(self.start[i]),
                row(self.x[i]),
                color,
            );
        }
        if self.config.on_torus() {
            let l = &self.config.lattice;
            let s = site_cells(l.width, l.height);
            for sy in 0..l.height as usize {
                for sx in 0..l.width as usize {
                    // By opinion now, whatever the mode: the lattice's state.
                    let color = opinion_color(self.x[sy * l.width as usize + sx]);
                    for dy in 0..s {
                        for dx in 0..s {
                            c.put(TORUS_X + sx * s + dx, sy * s + dy, color);
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
        let mut out = String::from("id,role,start,opinion,uncertainty,degree,meetings,moves\n");
        for i in 0..self.x.len() {
            let role = match self.role[i] {
                Role::Plus => "plus",
                Role::Minus => "minus",
                Role::Moderate => "moderate",
            };
            writeln!(
                out,
                "{},{role},{},{},{},{},{},{}",
                i + 1,
                self.start[i],
                self.x[i],
                self.u[i],
                self.degree(i),
                self.meetings[i],
                self.moves[i]
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// Nothing to follow: Inspect reads a cell, not an agent.
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Agreement(next) = next else {
            return Err(wrong_model(ModelKind::Agreement, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        // A live edit to how agents move can unsettle a stable run; resuming
        // it needs `stable_at` cleared now, since `run()` checks
        // `is_finished()` before stepping at all.
        if self.config.moves_differently(&next) {
            self.stable_at = None;
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stable: nothing moves again (until a live edit); stopped at
    /// `stop_at`: the run's reading is its last period.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agreement::config::{Network, Substrate};

    fn config(edit: impl FnOnce(&mut AgreementConfig)) -> AgreementConfig {
        let mut c = AgreementConfig::default();
        edit(&mut c);
        c
    }

    fn world(edit: impl FnOnce(&mut AgreementConfig)) -> AgreementWorld {
        AgreementWorld::new(config(edit), 1).unwrap()
    }

    fn close(a: (f64, f64), b: (f64, f64)) -> bool {
        (a.0 - b.0).abs() < 1e-12 && (a.1 - b.1).abs() < 1e-12
    }

    #[test]
    fn relative_agreement_scales_by_overlap_over_the_influencers_uncertainty() {
        let c = AgreementConfig::default(); // μ 0.2
                                            // i at 0 (u 0.5), j at 0.4 (u 0.5): h = 0.5 − (−0.1) = 0.6 > 0.5,
                                            // RA = 0.6/0.5 − 1 = 0.2; xj moves by 0.2·0.2·(−0.4).
        let got = influence(&c, (0.0, 0.5), (0.4, 0.5)).unwrap();
        assert!(close(got, (0.4 - 0.016, 0.5)), "{got:?}");
        // At h = uᵢ there is no influence.
        assert_eq!(influence(&c, (0.0, 0.5), (0.5, 0.5)), None);
        // A confident agent moves an uncertain one; not the other way round.
        let confident = (0.9, 0.1);
        let unsure = (0.5, 1.0);
        assert!(influence(&c, confident, unsure).is_some());
        assert_eq!(influence(&c, unsure, confident), None, "h = 0.2 ≤ 1");
        // Uncertainty moves toward the influencer's: j's 1.0 toward 0.1.
        let (_, u) = influence(&c, confident, unsure).unwrap();
        assert!(u < 1.0);
    }

    #[test]
    fn bounded_confidence_windows_read_either_uncertainty() {
        let mut c = config(|c| c.rule = Rule::Bc);
        let extremist = (1.0, 0.1);
        let moderate = (0.5, 1.0);
        // Eq. 11 as printed: the influencer's u′ decides.
        assert_eq!(influence(&c, extremist, moderate), None, "0.5 ≥ 0.1");
        assert!(close(
            influence(&c, moderate, extremist).unwrap(),
            (0.9, 0.1)
        ));
        c.window = Window::Listener;
        assert!(close(
            influence(&c, extremist, moderate).unwrap(),
            (0.6, 1.0)
        ));
        assert_eq!(influence(&c, moderate, extremist), None);
        c.rule = Rule::BcAveraging;
        assert!(close(
            influence(&c, extremist, moderate).unwrap(),
            (0.6, 0.82)
        ));
        c.rule = Rule::BcVariance;
        let (x, u) = influence(&c, extremist, moderate).unwrap();
        // x = 0.8·0.5 + 0.2·1; u² = 0.8·1 + 0.8·0.2·0.25.
        assert!(close((x, u), (0.6, (0.8f64 + 0.04).sqrt())));
    }

    #[test]
    fn a_meeting_updates_both_one_after_the_other_or_one() {
        let pair = |pair_update| {
            let mut w = world(|c| {
                c.agents = 2;
                c.extremists = 0.0;
                c.uncertainty = 4.0; // wide enough that any two overlap
                c.mu = 0.5;
                c.pair_update = pair_update;
            });
            let before = (w.x.clone(), w.u.clone());
            w.meet(0, 1);
            (before, w.x.clone())
        };
        let ((x0, _), sim) = pair(PairUpdate::Simultaneous);
        let ((_, _), seq) = pair(PairUpdate::Sequential);
        let ((_, _), one) = pair(PairUpdate::OneWay);
        let c = config(|c| {
            c.uncertainty = 4.0;
            c.mu = 0.5;
        });
        let (a, b) = ((x0[0], 4.0), (x0[1], 4.0));
        assert_eq!(sim[1], influence(&c, a, b).unwrap().0);
        assert_eq!(sim[0], influence(&c, b, a).unwrap().0);
        let nb = influence(&c, a, b).unwrap();
        assert_eq!(seq[1], nb.0);
        assert_eq!(
            seq[0],
            influence(&c, nb, a).unwrap().0,
            "the first sees the new second"
        );
        assert_ne!(seq[0], sim[0]);
        assert_eq!(one[1], x0[1], "only the first moves");
        assert_eq!(one[0], sim[0]);
    }

    #[test]
    fn extremists_are_counted_placed_and_bounded() {
        let mut rng = rng::seeded(1);
        let c = config(|c| c.extremists = 0.1);
        assert_eq!(extremist_counts(&c, 200, &mut rng), (10, 10));
        let lean = config(|c| {
            c.extremists = 0.1;
            c.delta = 0.2;
        });
        assert_eq!(extremist_counts(&lean, 200, &mut rng), (12, 8));
        // 75 extremists at δ 0: 37 or 38 on the positive side, by a coin.
        let odd = config(|c| c.extremists = 0.075);
        let sides: Vec<usize> = (1..=20)
            .map(|s| extremist_counts(&odd, 1000, &mut rng::seeded(s)).0)
            .collect();
        assert!(sides.iter().all(|&p| p == 37 || p == 38));
        assert!(sides.contains(&37) && sides.contains(&38), "{sides:?}");

        let w = world(|_| {});
        let ext: Vec<usize> = (0..200).filter(|&i| w.role[i] != Role::Moderate).collect();
        assert_eq!(ext.len(), 20);
        let lowest_plus = (0..200)
            .filter(|&i| w.role[i] == Role::Plus)
            .map(|i| w.x[i])
            .fold(f64::INFINITY, f64::min);
        assert!((0..200)
            .filter(|&i| w.role[i] == Role::Moderate)
            .all(|i| w.x[i] < lowest_plus));
        assert_eq!(w.plus_bound, Some(lowest_plus));
        assert!(ext.iter().all(|&i| w.u[i] == 0.1));
        assert!((0..200)
            .filter(|&i| w.role[i] == Role::Moderate)
            .all(|i| w.u[i] == 1.0));

        let b = world(|c| c.placement = Placement::Bounds);
        assert!((0..200).all(|i| match b.role[i] {
            Role::Plus => b.x[i] == 1.0,
            Role::Minus => b.x[i] == -1.0,
            Role::Moderate => b.x[i].abs() < 1.0,
        }));
        assert_eq!((b.plus_bound, b.minus_bound), (Some(1.0), Some(-1.0)));

        let band = world(|c| c.placement = Placement::Band);
        assert!((0..200).all(|i| match band.role[i] {
            Role::Plus => band.x[i] >= 0.8,
            Role::Minus => band.x[i] <= -0.8,
            Role::Moderate => band.x[i].abs() < 0.8,
        }));
        assert_eq!((band.plus_bound, band.minus_bound), (Some(0.8), Some(-0.8)));

        let none = world(|c| c.extremists = 0.0);
        assert_eq!((none.plus_bound, none.minus_bound), (None, None));
        assert_eq!(none.stats.latest().unwrap().y, 0.0);
    }

    #[test]
    fn y_counts_moderates_past_the_boundary_less_the_margin() {
        let mut w = world(|c| {
            c.agents = 10;
            c.extremists = 0.2;
            c.placement = Placement::Bounds;
        });
        let moderates: Vec<usize> = (0..10).filter(|&i| w.role[i] == Role::Moderate).collect();
        assert_eq!(moderates.len(), 8);
        for (k, &i) in moderates.iter().enumerate() {
            w.x[i] = match k {
                0..=3 => 0.95, // past 1 − 0.1
                4 => 0.85,     // not past
                5 => -0.91,    // past −1 + 0.1
                _ => 0.0,
            };
        }
        let (p, m) = w.new_extremists();
        assert_eq!((p, m), (0.5, 0.125));
        w.config.extreme_margin = 0.2;
        assert_eq!(w.new_extremists(), (0.625, 0.125));
        w.record();
        let s = w.stats.latest().unwrap();
        assert!((s.y - (0.625f64.powi(2) + 0.125f64.powi(2))).abs() < 1e-12);
        assert_eq!(s.outcome, Outcome::Intermediate as u32);
    }

    #[test]
    fn pairs_are_distinct_or_follow_the_network() {
        let mut w = world(|_| {});
        for _ in 0..1000 {
            let (a, b) = w.pair().unwrap();
            assert_ne!(a, b);
        }
        let mut l = world(|c| c.network = Network::Lattice);
        for _ in 0..1000 {
            let (a, b) = l.pair().unwrap();
            assert!(l.graph.of(a).contains(&(b as u32)));
        }
        let mut s = world(|c| {
            c.network = Network::ScaleFree;
            c.pairing = Pairing::Node;
        });
        let mut firsts = vec![0u32; 200];
        for _ in 0..20_000 {
            let (a, b) = s.pair().unwrap();
            assert!(s.graph.of(a).contains(&(b as u32)));
            firsts[a] += 1;
        }
        let hub = (0..200).max_by_key(|&i| s.graph.of(i).len()).unwrap();
        assert!(
            firsts[hub] < 250,
            "node pairing picks the first agent uniformly, hubs included ({})",
            firsts[hub]
        );
    }

    #[test]
    fn a_run_stabilizes_and_stops_or_stops_at_its_period() {
        let mut w = world(|c| c.stop_at = 0);
        w.run(10_000);
        assert!(w.is_finished());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.stable_at, w.tick);
        assert!(s.max_change <= STILL);
        let mut capped = world(|c| {
            c.stop_when_stable = false;
            c.stop_at = 7;
        });
        capped.run(100);
        assert_eq!((capped.tick, capped.is_finished()), (7, true));
        let mut on = w.clone();
        on.config.stop_when_stable = false;
        on.config.stop_at = 0;
        on.run(3);
        assert_eq!(on.tick, w.tick + 3, "without a stop it keeps stepping");
    }

    #[test]
    fn a_live_edit_after_stability_resumes_the_run() {
        let mut w = world(|c| {
            c.extremists = 0.0;
            c.uncertainty = 0.2;
        });
        w.run(10_000);
        assert!(w.is_finished());
        let mut same = w.clone();
        let margin = AgreementConfig {
            extreme_margin: 0.3,
            ..same.config.clone()
        };
        Model::set_config(&mut same, ModelConfig::Agreement(margin)).unwrap();
        assert!(same.is_finished(), "the margin only changes what y counts");
        let faster = AgreementConfig {
            rule: Rule::Bc,
            window: Window::Listener,
            ..w.config.clone()
        };
        Model::set_config(&mut w, ModelConfig::Agreement(faster)).unwrap();
        assert!(!w.is_finished(), "a new rule unsettles the run");
    }

    #[test]
    fn the_history_halves_to_keep_every_run_in_view() {
        let mut w = world(|c| {
            c.agents = 20;
            c.stop_when_stable = false;
            c.stop_at = 0;
        });
        w.run(KEPT as u32 - 1);
        assert_eq!((w.history.len(), w.interval), (KEPT, 1));
        w.run(1);
        assert_eq!(w.interval, 2);
        assert_eq!(w.history.len(), KEPT / 2 + 1);
        assert!(w.history.iter().all(|c| c.period % 2 == 0));
        assert_eq!(w.history.last().unwrap().period, 240);
        w.run(1);
        assert_eq!(
            w.columns().len(),
            KEPT / 2 + 2,
            "the current period is drawn too"
        );
        w.run(1000);
        assert!(w.columns().len() <= COLUMNS);
        assert_eq!(w.columns().last().unwrap().period, w.tick);
        assert_eq!(w.history[0].period, 0);
    }

    #[test]
    fn the_frame_draws_the_diagram_the_scatter_and_the_torus() {
        let mut w = world(|c| c.stop_at = 0);
        let mut buf = Vec::new();
        w.render("uncertainty", "", &mut buf).unwrap();
        let (fw, fh) = Model::size(&w);
        assert_eq!((fw as usize, fh as usize), (SCATTER_X + TALL, TALL));
        assert_eq!(buf.len(), (fw * fh * 4) as usize);
        let px = |b: &[u8], x: usize, y: usize| {
            let k = (y * fw as usize + x) * 4;
            [b[k], b[k + 1], b[k + 2]]
        };
        let top = (0..200).max_by(|&a, &b| w.x[a].total_cmp(&w.x[b])).unwrap();
        assert_eq!(
            px(&buf, 0, row(w.x[top])),
            uncertainty_color(0.1, 1.0),
            "the top line is an extremist's"
        );
        let bare = (0..TALL)
            .find(|&k| w.starts().iter().all(|&s| scatter_col(s) != k))
            .unwrap();
        assert_eq!(px(&buf, SCATTER_X + bare, TALL - 1 - bare), DIAGONAL);
        w.run(5);
        w.render("role", "", &mut buf).unwrap();
        w.render("start", "", &mut buf).unwrap();
        assert!(w.render("wealth", "", &mut buf).is_err());
        let t = world(|c| {
            c.network = Network::SmallWorld;
            c.small_world.substrate = Substrate::Grid;
            c.lattice.width = 10;
            c.lattice.height = 10;
        });
        assert_eq!(Model::size(&t), ((TORUS_X + TALL) as u32, TALL as u32));
        t.render("start", "", &mut buf).unwrap();
        let k = TORUS_X * 4;
        assert_eq!(buf[k..k + 3], opinion_color(t.starts()[0]));
    }

    #[test]
    fn inspect_finds_lines_dots_and_sites() {
        let mut w = world(|c| {
            c.agents = 5;
            c.extremists = 0.0;
            c.stop_when_stable = false;
            c.stop_at = 0;
        });
        w.run(3);
        let i = 2;
        let v = w.inspect(0, row(w.starts()[i]) as u32).unwrap();
        assert_eq!((v.panel, v.period), (Some("diagram"), Some(0)));
        assert!(v
            .agents
            .iter()
            .any(|a| a.id == 3 && a.opinion == w.starts()[i]));
        let now = w.inspect(3, row(w.x[i]) as u32).unwrap();
        assert_eq!(now.period, Some(3));
        let (sx, sy) = (SCATTER_X + scatter_col(w.starts()[i]), row(w.x[i]));
        let dot = w.inspect(sx as u32, sy as u32).unwrap();
        assert_eq!(dot.panel, Some("scatter"));
        assert!(dot.agents.iter().any(|a| a.id == 3 && a.degree == 4));
        assert!(dot.agent.is_none(), "a cell, not an agent");
        assert!(
            w.inspect(4, 0).unwrap().panel.is_none(),
            "past the last column"
        );
        assert!(
            w.inspect(COLUMNS as u32 + 1, 0).unwrap().panel.is_none(),
            "the gap"
        );
        assert_eq!(Model::locate(&w, 3), None);
        let l = world(|c| {
            c.network = Network::Lattice;
            c.lattice.width = 10;
            c.lattice.height = 10;
        });
        let s = site_cells(10, 10) as u32;
        let v = l.inspect(TORUS_X as u32 + s + 1, 1).unwrap();
        assert_eq!(v.panel, Some("torus"));
        assert_eq!((v.agents[0].id, v.agents[0].degree), (2, 4));
    }

    #[test]
    fn keyframes_restore_opinions_and_the_diagram() {
        let mut any =
            crate::model::ModelWorld::new(ModelConfig::Agreement(config(|c| c.stop_at = 0)), 6)
                .unwrap();
        any.model_mut().run(5);
        let cp = any.checkpoint().unwrap();
        let print = any.model().fingerprint();
        let mut before = Vec::new();
        any.model().render("uncertainty", "", &mut before).unwrap();
        any.model_mut().run(10);
        any.restore(&cp).unwrap();
        assert_eq!(any.model().fingerprint(), print);
        let mut after = Vec::new();
        any.model().render("uncertainty", "", &mut after).unwrap();
        assert_eq!(before, after);
        assert_eq!(any.model().series("y").unwrap().len(), 6);
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let next = config(|c| {
            c.rule = Rule::BcVariance;
            c.mu = 0.4;
            c.pair_update = PairUpdate::OneWay;
            c.extreme_margin = 0.2;
        });
        Model::set_config(&mut w, ModelConfig::Agreement(next.clone())).unwrap();
        w.step();
        for (field, edit) in [
            ("agents", config(|c| c.agents = 201)),
            ("placement", config(|c| c.placement = Placement::Band)),
            ("network", config(|c| c.network = Network::Lattice)),
        ] {
            let e = Model::set_config(&mut w, ModelConfig::Agreement(edit)).unwrap_err();
            assert_eq!(e[0].field, field);
        }
    }

    #[test]
    fn degenerate_configs_run_without_panicking() {
        for c in [
            config(|c| c.agents = 2),
            config(|c| c.extremists = 0.0),
            config(|c| c.extremists = 1.0),
            config(|c| {
                c.agents = 3;
                c.extremists = 1.0;
                c.delta = 1.0;
            }),
            config(|c| c.uncertainty = 0.1),
            config(|c| c.mu = 0.0),
            config(|c| c.mu = 1.0),
            config(|c| {
                c.rule = Rule::BcVariance;
                c.alpha = 0.0;
            }),
            config(|c| c.placement = Placement::Band),
            config(|c| {
                c.network = Network::SmallWorld;
                c.small_world.degree = 2;
                c.small_world.rewire = 1.0;
            }),
            config(|c| {
                c.network = Network::ScaleFree;
                c.scale_free.links = 1;
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            }),
            config(|c| {
                c.network = Network::Lattice;
                c.lattice.width = 3;
                c.lattice.height = 3;
            }),
        ] {
            let mut w = AgreementWorld::new(c.clone(), 1).unwrap();
            w.run(50);
            let s = w.stats.latest().unwrap();
            assert!((-1.0..=1.0).contains(&s.mean_opinion), "{c:?}");
            assert!((0.0..=2.0).contains(&s.y), "{c:?}");
            assert!(s.mean_uncertainty.is_finite(), "{c:?}");
        }
    }
}
```

Create `crates/sugarscape-core/src/agreement/presets.rs` with exactly this content:

```rust
//! The papers' runs, the replication's and the reply's readings, and the
//! network studies.

use super::config::{
    AgreementConfig, LatticeConfig, Network, PairUpdate, Pairing, Placement, Rule, ScaleFreeConfig,
    SmallWorldConfig, Substrate, Window,
};
use crate::model::ModelConfig;
use crate::opinions::Neighborhood;
use crate::presets::ModelPreset;

const DNAW: &str = "Deffuant, Neau, Amblard & Weisbuch 2000";
const DAWF: &str = "Deffuant, Amblard, Weisbuch & Faure 2002";
const MC: &str = "Meadows & Cliff 2012";
const DAW: &str = "Deffuant, Amblard & Weisbuch 2013";
const AD: &str = "Amblard & Deffuant 2004";
const W: &str = "Weisbuch 2004";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut AgreementConfig),
) -> ModelPreset {
    let mut c = AgreementConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Agreement(c),
    }
}

/// DNAW's pairwise model: bounded confidence, no extremists, d on [0, 1]
/// as U = 2d here.
fn dnaw(c: &mut AgreementConfig, d: f64, mu: f64) {
    c.rule = Rule::Bc;
    c.extremists = 0.0;
    c.uncertainty = 2.0 * d;
    c.mu = mu;
}

/// DAWF's Figs. 5–8: N 200, μ 0.5.
fn regime(c: &mut AgreementConfig, pe: f64, u: f64) {
    c.agents = 200;
    c.mu = 0.5;
    c.extremists = pe;
    c.uncertainty = u;
}

/// Fig. 9's μ and ue at pe 0.05, U 1.4, N 200.
fn literal(c: &mut AgreementConfig) {
    c.extremists = 0.05;
    c.uncertainty = 1.4;
}

/// A reading that stops at a fixed period.
fn reading(c: &mut AgreementConfig, margin: f64, stop_at: u32) {
    literal(c);
    c.placement = Placement::Band;
    c.extreme_margin = margin;
    c.stop_when_stable = false;
    c.stop_at = stop_at;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "dnaw-consensus",
            "Pairs meet: consensus",
            DNAW,
            "Deffuant, Neau, Amblard and Weisbuch's pairwise bounded confidence: a thousand agents with opinions spread evenly; each period every agent meets, on average twice, a random partner, and when their opinions differ by less than d both move toward each other by μ = 0.5 of the gap. With d = 0.5 (Fig. 1) the population agrees. Opinions run from −1 to 1 here, so d = 0.5 on the paper's [0, 1] is an uncertainty of 1.0. Lines are agents, colored by uncertainty (here all the same); the right panel plots each agent's start against its opinion now. Measured (20 seeds): consensus in every run, still by period 32 (median).",
            |c| {
                c.agents = 1000;
                dnaw(c, 0.5, 0.5);
            },
        ),
        preset(
            "dnaw-clusters",
            "Pairs meet: two clusters",
            DNAW,
            "The same meetings with d = 0.2 (Fig. 2, uncertainty 0.4 here): two clusters. Fig. 4 counts peaks falling as d grows, about 1/(2d) at most (2.5 here). Measured (20 seeds): two major clusters in every run, holding about half each, plus a median of 3 stragglers near the ends (the paper's 'wings'); still by period 73.",
            |c| {
                c.agents = 1000;
                dnaw(c, 0.2, 0.5);
            },
        ),
        preset(
            "dnaw-lattice",
            "On a lattice: one cluster and stragglers",
            DNAW,
            "The same meetings on a 29 × 29 torus, each agent meeting only its four neighbors, d = 0.3, μ = 0.3 (Fig. 5): 'a large majority … reached consensus … apart from isolated agents which have extremist opinions'. The torus is drawn at the right. The caption's '100 000 iterations' are 119 meetings per agent. Measured (20 seeds): at period 119 the largest cluster (opinions within 10⁻³) holds only 41 % — the lattice is still settling; run to stability (median period 1 797) it holds 92 %, with 27 isolated agents: the figure's picture, but much later than its caption says.",
            |c| {
                dnaw(c, 0.3, 0.3);
                c.network = Network::Lattice;
            },
        ),
        preset(
            "dnaw-lattice-clusters",
            "On a lattice: many local clusters",
            DNAW,
            "The lattice with d = 0.15 (Fig. 6): one percolating cluster and 'smaller non-percolating clusters with similar but not equal opinions'. Measured (20 seeds, to stability, median period 3 963; one run reached the 20 000-period cap): a median of 16.5 clusters holding 1 % of agents or more, 141 isolated agents, and a largest cluster of 6 % — similar opinions, split across many local clusters.",
            |c| {
                dnaw(c, 0.15, 0.3);
                c.network = Network::Lattice;
            },
        ),
        preset(
            "ra-uniform",
            "Relative agreement, no extremists",
            DAWF,
            "Deffuant, Amblard, Weisbuch and Faure's relative agreement: each agent has an opinion and an uncertainty (a segment around the opinion); when two meet, each moves the other by μ times the overlap of their segments beyond half the influencer's width, divided by that width — so confident agents sway uncertain ones, and uncertainties move too. Without extremists (Fig. 3: 200 agents, uncertainty 0.4) the clusters number about w/2u = 2.5 (Fig. 4). Lines are colored from confident red to uncertain green. Measured (50 seeds): a median of 3 clusters (2 to 6), still by period 96.",
            |c| {
                regime(c, 0.0, 0.4);
            },
        ),
        preset(
            "ra-central",
            "Extremists: central convergence",
            DAWF,
            "Extremists: the 20 % most extreme opinions (10 % at each end) start confident (uncertainty 0.1), the rest at 0.4; μ = 0.5, 200 agents (Fig. 5). The paper: central convergence, 'only a marginal part of the initially non-extremists became extremist (4%)'. y (DAWF's indicator) is the sum of the squared shares of moderates that end up extremists at each end: 0 central, 0.5 both extremes, 1 a single extreme. Measured (40 seeds): 23 % of moderates per side end past the innermost extremist less 0.1 (y 0.12) — everyone within the extremists' reach joins them; the caption's 4 % does not reproduce (11 % per side with extremists set to ±1). The majority does stay central.",
            |c| {
                regime(c, 0.2, 0.4);
            },
        ),
        preset(
            "ra-both",
            "Extremists: both extremes",
            DAWF,
            "Fig. 6's extremists: a quarter of the agents, moderates much less sure (uncertainty 1.2), μ = 0.5. The paper: the moderates split between the two extremes (43 % and 56 %; y 0.49). Measured (20 seeds): both extremes in every run, y 0.51.",
            |c| {
                regime(c, 0.25, 1.2);
            },
        ),
        preset(
            "ra-single",
            "Extremists: a single extreme",
            DAWF,
            "Fig. 7's setup: 10 % extremists, moderates at uncertainty 1.4, μ = 0.5, 200 agents. The paper's run ends with 98.33 % of moderates at one extreme (y 0.97), and Fig. 8, 'for the same parameters', stays central — the instability the paper is about. Measured: at the stated μ = 0.5 both extremes in 39 of 40 runs (y 0.53), under every placement and update order tried; single and central appear only at Fig. 9's μ = 0.2 (10 and 10 of 20 runs), as §4.8 predicts ('when the intensity of interactions (μ) increases, the both extremes convergence zone increases'). The figures' μ looks misstated.",
            |c| {
                regime(c, 0.1, 1.4);
            },
        ),
        preset(
            "ra-literal",
            "Fig. 9's corner, as the paper states it",
            DAWF,
            "Fig. 9's corner, as the paper states it: μ = 0.2, extremists' uncertainty 0.1, 5 % extremists drawn as the most extreme of 200 uniform opinions, moderates at 1.4, run 'until … invariant opinions and uncertainties', a moderate counted as extremist once past the innermost extremist less 0.1. Meadows and Cliff (2012) could not reproduce Fig. 9 here; this reading does. Measured (20 seeds): a single extreme in 19 runs (y 0.95), still by period 270 (median).",
            literal,
        ),
        preset(
            "ra-meadows-cliff",
            "Meadows and Cliff's reading",
            MC,
            "Meadows and Cliff's reimplementation of Fig. 9's corner: moderates uniform on (−0.8, 0.8), extremists uniform in [0.8, 1] and [−1, −0.8], y measured after 200 meetings per agent with a moderate counted as extremist only past ±0.8. Their conclusion: 'no conditions under which single extreme convergence will occur in the majority of the simulations'. Measured (20 seeds): y 0.00 in every run — yet the mean opinion has already drifted to ±0.56: the majority is on its way to one extreme but short of 0.8 at period 200. Compare with ra-deffuant-2013.",
            |c| {
                reading(c, 0.0, 200);
            },
        ),
        preset(
            "ra-deffuant-2013",
            "The authors' reply",
            DAW,
            "Deffuant, Amblard and Weisbuch's reply (2013): Meadows and Cliff 'compute indicator y before model convergence'; with 1 200 meetings per agent and new extremists counted past ±0.7 ('a threshold … lower of 0.1 than the threshold for initial extremists') their own program gives Fig. 9. Same placement as ra-meadows-cliff. Measured (20 seeds): a single extreme in 19 runs (y 0.95). Neither fix is enough alone (the ra-readings sweep): the drifted majority settles between 0.7 and 0.8.",
            |c| {
                reading(c, 0.1, 1200);
            },
        ),
        preset(
            "ra-bc-extremists",
            "Bounded confidence with extremists",
            DAWF,
            "§6's plain bounded confidence with extremists (Fig. 20): 1000 agents, 5 % extremists at uncertainty 0.1, moderates at 1.0, μ = 0.2; an agent moves toward a partner within its window. With the window read as the listener's own uncertainty the paper's claims hold: single extreme 'limited to a zone of parameters around … U = 1'. Measured (20 seeds): a single extreme in every run (y 1.00). Eq. 11 as printed uses the influencer's uncertainty instead — see ra-bc-printed.",
            |c| {
                c.agents = 1000;
                c.rule = Rule::Bc;
                c.window = Window::Listener;
                c.extremists = 0.05;
                c.uncertainty = 1.0;
            },
        ),
        preset(
            "ra-bc-printed",
            "Bounded confidence, eq. 11 as printed",
            DAWF,
            "The same society with eq. 11 as printed: 'If |x − x′| < u′ the influence of x′ on x …', u′ being the influencer's uncertainty. Then the confident extremists move toward every uncertain moderate who meets them, and the moderates ignore them. Measured (20 seeds): consensus near the center in every run, y 0.00 — and 0.00 over the whole of Fig. 20's grid (the ra-rules sweep). §6's results need the listener's window.",
            |c| {
                c.agents = 1000;
                c.rule = Rule::Bc;
                c.window = Window::Influencer;
                c.extremists = 0.05;
                c.uncertainty = 1.0;
            },
        ),
        preset(
            "ad-moore",
            "Extremists on a Moore lattice",
            AD,
            "Amblard and Deffuant's lattice (Fig. 3c): a 30 × 30 torus with eight neighbors each, 20 % extremists set to ±1, moderates at uncertainty 1.4, μ = 0.2. 'The single extreme convergence never occurs': extremism spreads from each extremist through its neighborhood until it meets the other side's. Measured (20 seeds): both extremes in every run (y 0.49); settling is slow (median period 18 475; some runs reach the 20 000-period cap).",
            |c| {
                c.network = Network::Lattice;
                c.lattice = LatticeConfig {
                    width: 30,
                    height: 30,
                    neighborhood: Neighborhood::Moore,
                };
                c.extremists = 0.2;
                c.uncertainty = 1.4;
                c.placement = Placement::Bounds;
            },
        ),
        preset(
            "ad-small-world",
            "Extremists in a small world",
            AD,
            "Amblard and Deffuant's small world (Figs. 4–5): 1000 agents on a ring, each linked to 32 neighbors, 80 % of links rewired at random; 5 % extremists at ±1, moderates at 1.8, μ = 0.1. Fully connected, these parameters give a single extreme; on sparse networks both extremes. Measured (20 seeds): a single extreme in 11 runs, central in 9 — k = 32 is near the transition (the ad-connectivity sweep).",
            |c| {
                c.agents = 1000;
                c.mu = 0.1;
                c.uncertainty = 1.8;
                c.extremists = 0.05;
                c.placement = Placement::Bounds;
                c.network = Network::SmallWorld;
                c.small_world = SmallWorldConfig {
                    substrate: Substrate::Ring,
                    degree: 32,
                    rewire: 0.8,
                };
            },
        ),
        preset(
            "w-scale-free",
            "A scale-free network",
            W,
            "Weisbuch's scale-free network (Fig. 4): 900 agents grown by preferential attachment, two links each, pairwise bounded confidence with d = 0.2 (uncertainty 0.4 here), μ = 0.5; a random agent picks a random neighbor and only the first moves. Hubs are influential without being more influenced. Measured (20 seeds): two major clusters, but 116 agents (median) left isolated — 15 % never moved at all, Weisbuch's 'outlying' nodes; still by period 1 331 (median).",
            |c| {
                c.agents = 900;
                dnaw(c, 0.2, 0.5);
                c.network = Network::ScaleFree;
                c.scale_free = ScaleFreeConfig { links: 2 };
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, AgreementConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Agreement(c) => (p.id, c),
                _ => panic!("{} is not a relative agreement preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        let d = find("dnaw-clusters");
        assert_eq!(
            (d.rule, d.extremists, d.uncertainty, d.mu, d.agents),
            (Rule::Bc, 0.0, 0.4, 0.5, 1000)
        );
        assert_eq!(find("dnaw-lattice").population(), 841);
        let s = find("ra-single");
        assert_eq!(
            (s.agents, s.mu, s.extremists, s.uncertainty),
            (200, 0.5, 0.1, 1.4)
        );
        let lit = find("ra-literal");
        assert_eq!(
            (lit.mu, lit.extremists, lit.uncertainty, lit.placement),
            (0.2, 0.05, 1.4, Placement::Drawn)
        );
        let mc = find("ra-meadows-cliff");
        assert_eq!(
            (
                mc.placement,
                mc.extreme_margin,
                mc.stop_when_stable,
                mc.stop_at
            ),
            (Placement::Band, 0.0, false, 200)
        );
        let daw = find("ra-deffuant-2013");
        assert_eq!((daw.extreme_margin, daw.stop_at), (0.1, 1200));
        assert_eq!(find("ra-bc-extremists").window, Window::Listener);
        assert_eq!(find("ra-bc-printed").window, Window::Influencer);
        let sw = find("ad-small-world");
        assert_eq!((sw.small_world.degree, sw.small_world.rewire), (32, 0.8));
        let w = find("w-scale-free");
        assert_eq!(
            (w.network, w.pairing, w.pair_update, w.uncertainty),
            (Network::ScaleFree, Pairing::Node, PairUpdate::OneWay, 0.4)
        );
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
    }
}
```

Create `crates/sugarscape-core/src/agreement/mod.rs` with exactly this content:

```rust
//! Relative Agreement (milestone 22): Deffuant, Amblard, Weisbuch and Faure,
//! "How Can Extremism Prevail?" (JASSS 2002), with the pairwise bounded
//! confidence of Deffuant, Neau, Amblard and Weisbuch (2000), the networks of
//! Amblard and Deffuant (2004) and Weisbuch (2004), and Meadows and Cliff's
//! (2012) and the authors' (2013) readings as named switches. See
//! docs/superpowers/specs/2026-09-26-relative-agreement-design.md.

mod config;
mod network;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, AgreementConfig, LatticeConfig, Network, PairUpdate, Pairing, Placement, Rule,
    ScaleFreeConfig, SmallWorldConfig, Substrate, Window, MAX_AGENTS, MAX_SIDE,
};
pub use network::Graph;
pub use presets::presets;
pub use stats::{grouping, AgreementSnapshot, Outcome, GAP, SERIES, STILL};
pub use view::{row, scatter_col, COLUMNS, KEPT, SCATTER_X, TALL, TORUS_X};
pub use world::{
    extremist_counts, influence, AgreementAgent, AgreementCell, AgreementInspection, AgreementMode,
    AgreementWorld, Role,
};
```

- [ ] **Step 2: Share the cluster helper with `opinions`**

`opinions`' `clusters` becomes `groups(sorted, SAME)`, so its results (and golden entries) are unchanged; the canvas helpers are re-exported for `agreement`.

Modify `crates/sugarscape-core/src/opinions/stats.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-core/src/opinions/stats.rs
+++ b/crates/sugarscape-core/src/opinions/stats.rs
@@ -78,10 +78,16 @@ impl Series for OpinionsSnapshot {
 
 /// The sizes of the clusters of a sorted profile, in order.
 pub fn clusters(sorted: &[f64]) -> Vec<usize> {
+    groups(sorted, SAME)
+}
+
+/// The sizes of the runs of a sorted profile whose neighboring gaps are at
+/// most `gap`, in order.
+pub fn groups(sorted: &[f64], gap: f64) -> Vec<usize> {
     let mut out = Vec::new();
     let mut run = 0;
     for (k, &x) in sorted.iter().enumerate() {
-        if k > 0 && x - sorted[k - 1] > SAME {
+        if k > 0 && x - sorted[k - 1] > gap {
             out.push(run);
             run = 0;
         }
```

Modify `crates/sugarscape-core/src/opinions/mod.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-core/src/opinions/mod.rs
+++ b/crates/sugarscape-core/src/opinions/mod.rs
@@ -13,6 +13,8 @@ pub use config::{
     schema, Confidence, Interaction, LatticeConfig, Neighborhood, OpinionsConfig, Start, Updating,
 };
 pub use presets::presets;
-pub use stats::{clusters, median, splits, OpinionsSnapshot, SAME, SERIES, STILL};
-pub use view::{hue, opinion_at, row, GAP, HISTORY, LATTICE_X, STEP, TALL, WIDE};
+pub use stats::{clusters, groups, median, splits, OpinionsSnapshot, SAME, SERIES, STILL};
+pub use view::{
+    hue, opinion_at, row, site_cells, Canvas, GAP, HISTORY, LATTICE_X, STEP, TALL, WIDE,
+};
 pub use world::{OpinionAgent, OpinionsCell, OpinionsInspection, OpinionsMode, OpinionsWorld};
```

- [ ] **Step 3: Wire the model kind**

Modify `crates/sugarscape-core/src/lib.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -4,6 +4,7 @@
 //! rule statements in Chapters II–III.
 
 pub mod agent;
+pub mod agreement;
 pub mod anasazi;
 pub mod bits;
 pub mod civil;
```

Modify `crates/sugarscape-core/src/model.rs` — every match gains `Agreement`; the kind list test gains `"agreement"` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -5,6 +5,7 @@
 
 use serde::{Serialize, Serializer};
 
+use crate::agreement::{AgreementConfig, AgreementWorld};
 use crate::anasazi::{AnasaziConfig, AnasaziWorld};
 use crate::civil::{CivilConfig, CivilWorld};
 use crate::classes::{ClassesConfig, ClassesWorld};
@@ -23,8 +24,8 @@ use crate::structure::{StructureConfig, StructureWorld};
 use crate::tags::{TagsConfig, TagsWorld};
 use crate::world::World;
 use crate::{
-    anasazi, civil, classes, culture, dpd, ethno, export, norms, opinions, ring, schelling,
-    spatial, stats, structure, tags,
+    agreement, anasazi, civil, classes, culture, dpd, ethno, export, norms, opinions, ring,
+    schelling, spatial, stats, structure, tags,
 };
 
 /// Which model a config or world is.
@@ -45,10 +46,11 @@ pub enum ModelKind {
     Structure,
     Dpd,
     Norms,
+    Agreement,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 14] = [
+    pub const ALL: [ModelKind; 15] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -63,6 +65,7 @@ impl ModelKind {
         ModelKind::Structure,
         ModelKind::Dpd,
         ModelKind::Norms,
+        ModelKind::Agreement,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -81,6 +84,7 @@ impl ModelKind {
             ModelKind::Structure => "structure",
             ModelKind::Dpd => "dpd",
             ModelKind::Norms => "norms",
+            ModelKind::Agreement => "agreement",
         }
     }
 
@@ -102,6 +106,7 @@ impl ModelKind {
             ModelKind::Structure => structure::schema(),
             ModelKind::Dpd => dpd::schema(),
             ModelKind::Norms => norms::schema(),
+            ModelKind::Agreement => agreement::schema(),
         }
     }
 }
@@ -129,6 +134,7 @@ pub enum ModelConfig {
     Structure(StructureConfig),
     Dpd(DpdConfig),
     Norms(NormsConfig),
+    Agreement(AgreementConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -148,6 +154,7 @@ enum Tagged<'a> {
     Structure(&'a StructureConfig),
     Dpd(&'a DpdConfig),
     Norms(&'a NormsConfig),
+    Agreement(&'a AgreementConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -174,6 +181,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Structure(c) => Tagged::Structure(c).serialize(s),
             ModelConfig::Dpd(c) => Tagged::Dpd(c).serialize(s),
             ModelConfig::Norms(c) => Tagged::Norms(c).serialize(s),
+            ModelConfig::Agreement(c) => Tagged::Agreement(c).serialize(s),
         }
     }
 }
@@ -195,6 +203,7 @@ impl ModelConfig {
             ModelConfig::Structure(_) => ModelKind::Structure,
             ModelConfig::Dpd(_) => ModelKind::Dpd,
             ModelConfig::Norms(_) => ModelKind::Norms,
+            ModelConfig::Agreement(_) => ModelKind::Agreement,
         }
     }
 
@@ -271,10 +280,13 @@ impl ModelConfig {
             "norms" => serde_json::from_value(value)
                 .map(ModelConfig::Norms)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "agreement" => serde_json::from_value(value)
+                .map(ModelConfig::Agreement)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd or norms)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms or agreement)"
                 ),
             )),
         }
@@ -296,6 +308,7 @@ impl ModelConfig {
             ModelConfig::Structure(c) => c.validate(),
             ModelConfig::Dpd(c) => c.validate(),
             ModelConfig::Norms(c) => c.validate(),
+            ModelConfig::Agreement(c) => c.validate(),
         }
     }
 
@@ -317,6 +330,7 @@ impl ModelConfig {
             ModelConfig::Structure(c) => set_path(c, path, value).map(ModelConfig::Structure),
             ModelConfig::Dpd(c) => set_path(c, path, value).map(ModelConfig::Dpd),
             ModelConfig::Norms(c) => set_path(c, path, value).map(ModelConfig::Norms),
+            ModelConfig::Agreement(c) => set_path(c, path, value).map(ModelConfig::Agreement),
         }
     }
 
@@ -337,7 +351,8 @@ impl ModelConfig {
             | ModelConfig::Classes(_)
             | ModelConfig::Opinions(_)
             | ModelConfig::Structure(_)
-            | ModelConfig::Norms(_) => None,
+            | ModelConfig::Norms(_)
+            | ModelConfig::Agreement(_) => None,
         }
     }
 
@@ -358,6 +373,7 @@ impl ModelConfig {
             ModelConfig::Structure(_) => structure::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Dpd(_) => dpd::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Norms(_) => norms::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Agreement(_) => agreement::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -541,6 +557,7 @@ pub enum ModelWorld {
     Structure(Box<StructureWorld>),
     Dpd(Box<DpdWorld>),
     Norms(Box<NormsWorld>),
+    Agreement(Box<AgreementWorld>),
 }
 
 impl ModelWorld {
@@ -578,6 +595,9 @@ impl ModelWorld {
             }
             ModelConfig::Dpd(c) => ModelWorld::Dpd(Box::new(DpdWorld::new(c, seed)?)),
             ModelConfig::Norms(c) => ModelWorld::Norms(Box::new(NormsWorld::new(c, seed)?)),
+            ModelConfig::Agreement(c) => {
+                ModelWorld::Agreement(Box::new(AgreementWorld::new(c, seed)?))
+            }
         })
     }
 
@@ -597,6 +617,7 @@ impl ModelWorld {
             ModelWorld::Structure(_) => ModelKind::Structure,
             ModelWorld::Dpd(_) => ModelKind::Dpd,
             ModelWorld::Norms(_) => ModelKind::Norms,
+            ModelWorld::Agreement(_) => ModelKind::Agreement,
         }
     }
 
@@ -616,6 +637,7 @@ impl ModelWorld {
             ModelWorld::Structure(w) => w.as_ref(),
             ModelWorld::Dpd(w) => w.as_ref(),
             ModelWorld::Norms(w) => w.as_ref(),
+            ModelWorld::Agreement(w) => w.as_ref(),
         }
     }
 
@@ -635,6 +657,7 @@ impl ModelWorld {
             ModelWorld::Structure(w) => w.as_mut(),
             ModelWorld::Dpd(w) => w.as_mut(),
             ModelWorld::Norms(w) => w.as_mut(),
+            ModelWorld::Agreement(w) => w.as_mut(),
         }
     }
 
@@ -722,6 +745,7 @@ impl ModelWorld {
             ModelWorld::Structure(w) => copy_without_history!(Structure, w),
             ModelWorld::Dpd(w) => copy_without_history!(Dpd, w),
             ModelWorld::Norms(w) => copy_without_history!(Norms, w),
+            ModelWorld::Agreement(w) => copy_without_history!(Agreement, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -751,6 +775,9 @@ impl ModelWorld {
             (ModelWorld::Structure(live), ModelWorld::Structure(kept)) => restore_into!(live, kept),
             (ModelWorld::Dpd(live), ModelWorld::Dpd(kept)) => restore_into!(live, kept),
             (ModelWorld::Norms(live), ModelWorld::Norms(kept)) => restore_into!(live, kept),
+            (ModelWorld::Agreement(live), ModelWorld::Agreement(kept)) => {
+                restore_into!(live, kept)
+            }
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1066,7 +1093,8 @@ mod tests {
                 "opinions",
                 "structure",
                 "dpd",
-                "norms"
+                "norms",
+                "agreement"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
```

Modify `crates/sugarscape-core/src/presets.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -673,7 +673,8 @@ impl From<Preset> for ModelPreset {
 
 /// Every model's presets: the sugarscape's (`all`), then Schelling's, Ring
 /// World's, the anasazi's, civil violence's, the tags model's, the spatial
-/// games', the ethnocentrism model's and the demographic PD's.
+/// games', the ethnocentrism model's, the demographic PD's, the norms
+/// model's and relative agreement's.
 pub fn catalog() -> Vec<ModelPreset> {
     let mut out: Vec<ModelPreset> = all().into_iter().map(ModelPreset::from).collect();
     out.extend(crate::schelling::presets());
@@ -689,6 +690,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::ethno::presets());
     out.extend(crate::dpd::presets());
     out.extend(crate::norms::presets());
+    out.extend(crate::agreement::presets());
     out
 }
 
```

- [ ] **Step 4: Run the model's tests**

Run: `cargo test -p sugarscape-core --lib agreement`
Expected: 29 passed (`config` 4, `network` 6, `stats` 2, `view` 2, `presets` 1, `world` 14). Then `cargo test -p sugarscape-core --lib opinions model::` — the existing tests still pass.

- [ ] **Step 5: Record the golden entries**

Run: `cargo test --release -p sugarscape-core --test golden -- --ignored --nocapture 2>&1 | grep -E '"(dnaw|ra|ad|w)-'`
Expected: exactly the sixteen values below (planning's). If any differs, the code differs from this plan: find out why before recording anything.

Modify `crates/sugarscape-core/tests/golden.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -175,6 +175,23 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("gi-mild-metanorms", 0xb7435abcb67c192b),
     ("gi-temptation-10", 0xa191ff11a4f9ee68),
     ("gi-tournament", 0x95ea76458cee1a46),
+    // Milestone 22: relative agreement (the two readings share their first 200 periods).
+    ("dnaw-consensus", 0x1e241d02be7a49e5),
+    ("dnaw-clusters", 0x247643148886db0b),
+    ("dnaw-lattice", 0x60f9414e1157482d),
+    ("dnaw-lattice-clusters", 0x459f1c2e1f67651b),
+    ("ra-uniform", 0x9ed2107424379c37),
+    ("ra-central", 0xcd795ff8a5bd9e44),
+    ("ra-both", 0x6cc8db3561623a4c),
+    ("ra-single", 0x769843f421a9f63e),
+    ("ra-literal", 0x2a130dc7026094f7),
+    ("ra-meadows-cliff", 0x4b9cf5817bc155d6),
+    ("ra-deffuant-2013", 0x4b9cf5817bc155d6),
+    ("ra-bc-extremists", 0xc304b87400a0bb46),
+    ("ra-bc-printed", 0xef17ca06b9646fad),
+    ("ad-moore", 0xd39d77107d873634),
+    ("ad-small-world", 0xd29498f3ac055d5d),
+    ("w-scale-free", 0xed9e58b01a7987d8),
 ];
 
 fn fingerprint(id: &str) -> u64 {
```

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: PASS (every earlier entry unchanged).

- [ ] **Step 6: Format, lint and run everything**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings && cargo test --workspace --release`
Expected: clean; all pass (planning: 754 across the workspace after Task 2).

- [ ] **Step 7: Commit**

```bash
git add crates/sugarscape-core/src/agreement crates/sugarscape-core/src/opinions/stats.rs crates/sugarscape-core/src/opinions/mod.rs crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs
git commit -m "Add Relative Agreement (Deffuant et al.) as a model kind, with its readings and networks as switches

Claude-Session: https://claude.ai/code/session_011pY2cJwAezJH4xb32Aw8xr"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/{ra-clusters,ra-map,ra-readings,ra-population,ra-rules,ra-delta,ad-connectivity,w-dispersion}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets (`ra-literal`, `ad-small-world`, `w-scale-free`) and series (`y`, `major`, `dispersion`).
- Produces: built-in sweep ids `ra-clusters`, `ra-map`, `ra-readings`, `ra-population`, `ra-rules`, `ra-delta`, `ad-connectivity`, `w-dispersion` (in that order after `norms-dominance`); the CLI's stop reasons `(stable)` and `(its last period)` for `agreement`.

- [ ] **Step 1: Write the sweeps**

Each description records what it measured (release, seeds as stated, 2026-09-26); Step 4 re-measures them.

Create `sweeps/ra-clusters.json` with exactly this content:

```json
{
  "name": "Relative Agreement: clusters against w/2u (DAWF Fig. 4)",
  "description": "Deffuant et al. 2002, Fig. 4: without extremists, clusters against w/2U (the opinions' width, 2, over twice the uncertainty), 200 agents, μ = 0.5, run to stability; clusters counted if they hold at least 1 % of agents. The paper: relative agreement gives about w/2u clusters (r² = 0.99), bounded confidence about its integer part. Measured (release, seeds 1–20, recorded 2026-09-26): relative agreement 1.0, 1.1, 2.0, 2.1, 3.1, 3.7, 4.3, 6.0, 7.2, 11.3 at w/2u 1, 1.25, 1.67, 2, 2.5, 3.33, 4, 5, 6.67, 10 — about w/2u, a little above; bounded confidence 1.0, 1.1, 1.7, 2.0, 2.5, 3.5, 4.3, 5.1, 7.0, 10.0 — not the integer part (2.5 at 2.5, 3.5 at 3.33) at 200 agents; at 1000 agents it is the integer part up to about 3, then falls below.",
  "base": {
    "config": {
      "model": "agreement",
      "agents": 200,
      "extremists": 0.0,
      "mu": 0.5
    }
  },
  "x": {
    "label": "w/2U (the width, 2, over twice the uncertainty)",
    "values": [
      {
        "at": 1.0,
        "set": {
          "uncertainty": 1.0
        }
      },
      {
        "at": 1.25,
        "set": {
          "uncertainty": 0.8
        }
      },
      {
        "at": 1.6666666667,
        "set": {
          "uncertainty": 0.6
        }
      },
      {
        "at": 2.0,
        "set": {
          "uncertainty": 0.5
        }
      },
      {
        "at": 2.5,
        "set": {
          "uncertainty": 0.4
        }
      },
      {
        "at": 3.3333333333,
        "set": {
          "uncertainty": 0.3
        }
      },
      {
        "at": 4.0,
        "set": {
          "uncertainty": 0.25
        }
      },
      {
        "at": 5.0,
        "set": {
          "uncertainty": 0.2
        }
      },
      {
        "at": 6.6666666667,
        "set": {
          "uncertainty": 0.15
        }
      },
      {
        "at": 10.0,
        "set": {
          "uncertainty": 0.1
        }
      }
    ]
  },
  "series": {
    "label": "Rule",
    "values": [
      {
        "at": 0,
        "name": "Relative agreement",
        "set": {
          "rule": "ra"
        }
      },
      {
        "at": 1,
        "name": "Bounded confidence",
        "set": {
          "rule": "bc"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 20000,
  "metric": {
    "kind": "final",
    "series": "major"
  }
}
```

Create `sweeps/ra-map.json` with exactly this content:

```json
{
  "name": "Relative Agreement: y against U, by pe (DAWF Fig. 9)",
  "description": "Deffuant et al. 2002, Fig. 9: y against the moderates' uncertainty U for four proportions of extremists, as the paper states the model (μ 0.2, ue 0.1, the extremists drawn, run to stability), 200 agents where the figure has 1000 (the survey runs 1000). y is 0 when the moderates stay central, 0.5 when they split between the extremes, 1 when they all go to one. Measured (release, seeds 1–20, recorded 2026-09-26): the figure's zones — low y at small U; y ≈ 0.5 (both extremes) from U 0.7 for pe ≥ 0.1; the central dip near U 0.8–1.0 at small pe; y 1.00 from U 1.2 at pe 0.025 and 0.80–1.00 from 1.2 at pe 0.05. At pe 0.1 and above, large U gives 0.5–0.7: a mix of single and central runs.",
  "base": {
    "preset": "ra-literal"
  },
  "x": {
    "label": "Moderates' uncertainty (U)",
    "path": "uncertainty",
    "values": [
      0.2,
      0.3,
      0.4,
      0.5,
      0.6,
      0.7,
      0.8,
      0.9,
      1.0,
      1.1,
      1.2,
      1.3,
      1.4,
      1.5,
      1.6,
      1.7,
      1.8,
      1.9,
      2.0
    ]
  },
  "series": {
    "label": "Extremists (pe)",
    "values": [
      {
        "at": 0.025,
        "name": "pe = 0.025",
        "set": {
          "extremists": 0.025
        }
      },
      {
        "at": 0.05,
        "name": "pe = 0.05",
        "set": {
          "extremists": 0.05
        }
      },
      {
        "at": 0.1,
        "name": "pe = 0.1",
        "set": {
          "extremists": 0.1
        }
      },
      {
        "at": 0.2,
        "name": "pe = 0.2",
        "set": {
          "extremists": 0.2
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 20000,
  "metric": {
    "kind": "final",
    "series": "y"
  }
}
```

Create `sweeps/ra-readings.json` with exactly this content:

```json
{
  "name": "Relative Agreement: Meadows and Cliff against the reply",
  "description": "Why Meadows and Cliff (2012) could not reproduce Fig. 9, and the reply (Deffuant et al. 2013): y at pe 0.05 against U under five readings. Meadows and Cliff place the extremists in a band [0.8, 1], stop at 200 meetings per agent and count a moderate as extremist only past ±0.8; the reply runs 1 200 and counts past ±0.7. Measured (release, seeds 1–20, recorded 2026-09-26), y at U 1.2 / 1.6 / 2.0: Meadows and Cliff 0.00 / 0.00 / 0.00; their horizon fixed 0.45 / 0.40 / 0.50; their cutoff fixed 0.40 / 0.39 / 0.26; both fixed 1.00 / 1.00 / 1.00; the 2002 paper as stated 0.80 / 0.80 / 1.00. Both fixes are needed; the literal reading, run until nothing moves, agrees with the reply.",
  "base": {
    "preset": "ra-literal"
  },
  "x": {
    "label": "Moderates' uncertainty (U)",
    "path": "uncertainty",
    "values": [
      0.4,
      0.6,
      0.8,
      1.0,
      1.2,
      1.4,
      1.6,
      1.8,
      2.0
    ]
  },
  "series": {
    "label": "Reading",
    "values": [
      {
        "at": 0,
        "name": "Meadows & Cliff (band, cutoff 0.8, 200 periods)",
        "set": {
          "extreme_margin": 0.0,
          "stop_at": 200,
          "placement": "band",
          "stop_when_stable": false
        }
      },
      {
        "at": 1,
        "name": "Their horizon fixed (1200 periods)",
        "set": {
          "extreme_margin": 0.0,
          "stop_at": 1200,
          "placement": "band",
          "stop_when_stable": false
        }
      },
      {
        "at": 2,
        "name": "Their cutoff fixed (0.7)",
        "set": {
          "extreme_margin": 0.1,
          "stop_at": 200,
          "placement": "band",
          "stop_when_stable": false
        }
      },
      {
        "at": 3,
        "name": "The 2013 reply (both fixed)",
        "set": {
          "extreme_margin": 0.1,
          "stop_at": 1200,
          "placement": "band",
          "stop_when_stable": false
        }
      },
      {
        "at": 4,
        "name": "As the 2002 paper states it (drawn, to stability)",
        "set": {
          "placement": "drawn",
          "extreme_margin": 0.1,
          "stop_when_stable": true,
          "stop_at": 20000
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 20000,
  "metric": {
    "kind": "final",
    "series": "y"
  }
}
```

Create `sweeps/ra-population.json` with exactly this content:

```json
{
  "name": "Relative Agreement: single extreme against population size",
  "description": "Is the single extreme a finite-size effect? y at pe 0.1, U 1.6 (μ 0.2, the literal reading) against the number of agents, with the extremists balanced (δ 0) or leaning (δ 0.1). Meadows and Cliff found the single-extreme zone shrinking as N grows; the reply says it 'takes place with any large number of agents'. Measured (release, seeds 1–20, recorded 2026-09-26): δ 0: 0.74, 0.54, 0.34, 0.45, 0.15, 0.25, 0.05 for N 100, 200, 400, 700, 1000, 1500, 2000 — balanced extremists lose the single extreme as N grows (planning: none of 100 runs at N 4000); δ 0.1: 0.84 to 0.99, rising with N. A lean decides it; balanced, only chance does, and chance fades with N.",
  "base": {
    "preset": "ra-literal"
  },
  "set": {
    "extremists": 0.1,
    "uncertainty": 1.6
  },
  "x": {
    "label": "Agents (N)",
    "path": "agents",
    "values": [
      100,
      200,
      400,
      700,
      1000,
      1500,
      2000
    ]
  },
  "series": {
    "label": "Lean (δ)",
    "values": [
      {
        "at": 0,
        "name": "δ = 0",
        "set": {
          "delta": 0.0
        }
      },
      {
        "at": 1,
        "name": "δ = 0.1",
        "set": {
          "delta": 0.1
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 20000,
  "metric": {
    "kind": "final",
    "series": "y"
  }
}
```

Create `sweeps/ra-rules.json` with exactly this content:

```json
{
  "name": "Relative Agreement: the rules of §6 (DAWF Figs. 9, 20–22)",
  "description": "Deffuant et al. 2002 §6: relative agreement against three bounded-confidence rules, y against U at pe 0.05 and δ 0.1 (200 agents, μ 0.2, at most 2 000 periods: plain bounded confidence above U 1.2 never goes still). Measured (release, seeds 1–20, recorded 2026-09-26): relative agreement single extreme from U 1.2 (0.84–0.95); bounded confidence with eq. 11 as printed (the influencer's uncertainty) 0.00 everywhere; with the listener's uncertainty a single extreme at U 1.0 (0.90), none above 1.4 — the paper's 'limited to a zone … around U = 1'; averaging uncertainties 0.45–0.75 from U 1.2; the variance rule 0.00 everywhere ('the absence of single extreme convergence'). §6's text is reproduced only with the listener's window.",
  "base": {
    "preset": "ra-literal"
  },
  "set": {
    "delta": 0.1
  },
  "x": {
    "label": "Moderates' uncertainty (U)",
    "path": "uncertainty",
    "values": [
      0.2,
      0.4,
      0.6,
      0.8,
      1.0,
      1.2,
      1.4,
      1.6,
      1.8,
      2.0
    ]
  },
  "series": {
    "label": "Rule",
    "values": [
      {
        "at": 0,
        "name": "Relative agreement",
        "set": {
          "rule": "ra"
        }
      },
      {
        "at": 1,
        "name": "BC, eq. 11 as printed",
        "set": {
          "rule": "bc",
          "window": "influencer"
        }
      },
      {
        "at": 2,
        "name": "BC, the listener's window",
        "set": {
          "rule": "bc",
          "window": "listener"
        }
      },
      {
        "at": 3,
        "name": "BC averaging, the listener's window",
        "set": {
          "rule": "bc_averaging",
          "window": "listener"
        }
      },
      {
        "at": 4,
        "name": "BC from variance",
        "set": {
          "rule": "bc_variance",
          "window": "listener"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "y"
  }
}
```

Create `sweeps/ra-delta.json` with exactly this content:

```json
{
  "name": "Relative Agreement: y against the lean δ (DAWF §4.8)",
  "description": "Deffuant et al. 2002 §4.8: 'When the initial bias between the extremists (δ) increases the single extreme convergence zone increases'. y at pe 0.1, U 1.4 (200 agents, μ 0.2) against δ. Measured (release, seeds 1–20, recorded 2026-09-26): 0.48, 0.63, 0.63, 0.78, 0.88, 0.98, 0.98, 0.99 at δ 0, 0.05, 0.1, 0.15, 0.2, 0.3, 0.4, 0.5. The claim holds.",
  "base": {
    "preset": "ra-literal"
  },
  "set": {
    "extremists": 0.1
  },
  "x": {
    "label": "Lean (δ)",
    "path": "delta",
    "values": [
      0.0,
      0.05,
      0.1,
      0.15,
      0.2,
      0.3,
      0.4,
      0.5
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 20000,
  "metric": {
    "kind": "final",
    "series": "y"
  }
}
```

Create `sweeps/ad-connectivity.json` with exactly this content:

```json
{
  "name": "Relative Agreement: small worlds (Amblard & Deffuant Fig. 4)",
  "description": "Amblard and Deffuant 2004, Fig. 4: y against the number of neighbors k on a small-world ring, for three rewiring probabilities p (500 agents where the paper has 1000; U 1.8, pe 0.05 at ±1, μ 0.1, at most 5 000 periods). 'A transition from double extreme convergence to single extreme convergence … when the connectivity (k) increases … for higher connectivity when p decreases'. Measured (release, seeds 1–10, recorded 2026-09-26): single extremes from k 32 at p 0.8 and 1 (0.70, 0.60), from k 64 at p 0.2 (0.40; 0.90 at 128) — the threshold falls as p rises, as claimed, but sits above the paper's 'around 8'. Below it y ≈ 0 with a moderate counted as extremist past 0.9 (the reply's rule for extremists at ±1): central, not both extremes. Counted past 0.7 (the fourth line), low k gives y 0.46–0.53, the paper's both extremes: the local clusters settle between 0.7 and 0.9, and the paper does not say where it counted.",
  "base": {
    "preset": "ad-small-world"
  },
  "set": {
    "agents": 500
  },
  "x": {
    "label": "Neighbors (k)",
    "path": "small_world.degree",
    "values": [
      2,
      4,
      8,
      16,
      32,
      64,
      128
    ]
  },
  "series": {
    "label": "Rewiring and cutoff",
    "values": [
      {
        "at": 0.2,
        "name": "p = 0.2",
        "set": {
          "small_world.rewire": 0.2
        }
      },
      {
        "at": 0.8,
        "name": "p = 0.8",
        "set": {
          "small_world.rewire": 0.8
        }
      },
      {
        "at": 1.0,
        "name": "p = 1",
        "set": {
          "small_world.rewire": 1.0
        }
      },
      {
        "at": 0.81,
        "name": "p = 0.8, cutoff 0.7",
        "set": {
          "small_world.rewire": 0.8,
          "extreme_margin": 0.3
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 5000,
  "metric": {
    "kind": "final",
    "series": "y"
  }
}
```

Create `sweeps/w-dispersion.json` with exactly this content:

```json
{
  "name": "Relative Agreement: dispersion on networks (Weisbuch Fig. 3)",
  "description": "Weisbuch 2004, Fig. 3: the dispersion index (the sum of squared cluster shares) against the threshold d, pairwise bounded confidence, 900 agents, μ 0.5; on the networks a random agent picks a random neighbor and only it moves. Well mixed: 'two distinct steps at y = 0.5 and y = 0.33'; scale-free: 'a continuous increase … with only a kink in the d = 0.25, y = 0.7 region', close to the lattice, and closer to well mixed with twice the links. Measured (release, seeds 1–20, recorded 2026-09-26): well mixed 0.35 at d 0.15, 0.50–0.51 from 0.2 to 0.25, 0.94 from 0.3 — the steps; 4 links 0.16, 0.34, 0.62, 0.86 at d 0.15, 0.2, 0.25, 0.3 — no steps; the 30 × 30 lattice 0.02, 0.26, 0.70, 0.86; 8 links 0.32, 0.47, 0.56, 0.94. The claims hold.",
  "base": {
    "preset": "w-scale-free"
  },
  "x": {
    "label": "Threshold (d, on [0, 1])",
    "values": [
      {
        "at": 0.1,
        "set": {
          "uncertainty": 0.2
        }
      },
      {
        "at": 0.15,
        "set": {
          "uncertainty": 0.3
        }
      },
      {
        "at": 0.2,
        "set": {
          "uncertainty": 0.4
        }
      },
      {
        "at": 0.22,
        "set": {
          "uncertainty": 0.44
        }
      },
      {
        "at": 0.25,
        "set": {
          "uncertainty": 0.5
        }
      },
      {
        "at": 0.28,
        "set": {
          "uncertainty": 0.56
        }
      },
      {
        "at": 0.3,
        "set": {
          "uncertainty": 0.6
        }
      },
      {
        "at": 0.35,
        "set": {
          "uncertainty": 0.7
        }
      },
      {
        "at": 0.4,
        "set": {
          "uncertainty": 0.8
        }
      },
      {
        "at": 0.5,
        "set": {
          "uncertainty": 1.0
        }
      }
    ]
  },
  "series": {
    "label": "Network",
    "values": [
      {
        "at": 0,
        "name": "Well mixed",
        "set": {
          "network": "all",
          "pair_update": "simultaneous"
        }
      },
      {
        "at": 1,
        "name": "30 × 30 lattice",
        "set": {
          "network": "lattice",
          "lattice.width": 30,
          "lattice.height": 30
        }
      },
      {
        "at": 2,
        "name": "Scale-free, 4 links on average",
        "set": {
          "network": "scale_free",
          "scale_free.links": 2
        }
      },
      {
        "at": 3,
        "name": "Scale-free, 8 links on average",
        "set": {
          "network": "scale_free",
          "scale_free.links": 4
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 20
  },
  "ticks": 20000,
  "metric": {
    "kind": "final",
    "series": "dispersion"
  }
}
```

- [ ] **Step 2: Register them, name the CLI's stops, and pin WASM**

Modify `crates/sugarscape-core/src/sweep.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -962,7 +962,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 61] = [
+const BUILTINS: [Builtin; 69] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1207,6 +1207,38 @@ const BUILTINS: [Builtin; 61] = [
         id: "norms-dominance",
         json: include_str!("../../../sweeps/norms-dominance.json"),
     },
+    Builtin {
+        id: "ra-clusters",
+        json: include_str!("../../../sweeps/ra-clusters.json"),
+    },
+    Builtin {
+        id: "ra-map",
+        json: include_str!("../../../sweeps/ra-map.json"),
+    },
+    Builtin {
+        id: "ra-readings",
+        json: include_str!("../../../sweeps/ra-readings.json"),
+    },
+    Builtin {
+        id: "ra-population",
+        json: include_str!("../../../sweeps/ra-population.json"),
+    },
+    Builtin {
+        id: "ra-rules",
+        json: include_str!("../../../sweeps/ra-rules.json"),
+    },
+    Builtin {
+        id: "ra-delta",
+        json: include_str!("../../../sweeps/ra-delta.json"),
+    },
+    Builtin {
+        id: "ad-connectivity",
+        json: include_str!("../../../sweeps/ad-connectivity.json"),
+    },
+    Builtin {
+        id: "w-dispersion",
+        json: include_str!("../../../sweeps/w-dispersion.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2090,7 +2122,15 @@ mod tests {
                 "norms-temptation",
                 "norms-selection",
                 "norms-readings",
-                "norms-dominance"
+                "norms-dominance",
+                "ra-clusters",
+                "ra-map",
+                "ra-readings",
+                "ra-population",
+                "ra-rules",
+                "ra-delta",
+                "ad-connectivity",
+                "w-dispersion"
             ]
         );
         for b in builtins() {
```

Modify `crates/sugarscape-cli/src/main.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -210,7 +210,7 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
         // the tags model at its last generation; Axelrod's culture and bounded
         // confidence once stable, a sugarscape under his rule once its cultures
         // settle; ethnocentrism at its last period; the demographic PD at its
-        // last cycle.
+        // last cycle; norms at their last generation.
         let why = match config.kind() {
             ModelKind::Civil => "a group has died out",
             ModelKind::Tags => "its last generation",
@@ -222,6 +222,16 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             ModelKind::Ethno => "its last period",
             ModelKind::Dpd => "its last cycle",
             ModelKind::Norms => "its last generation",
+            // Relative agreement stops when stable, or at `stop_at` (a cap, or a
+            // reading's fixed horizon).
+            ModelKind::Agreement => match &config {
+                ModelConfig::Agreement(c)
+                    if c.stop_at > 0 && world.tick() >= u64::from(c.stop_at) =>
+                {
+                    "its last period"
+                }
+                _ => "stable",
+            },
             _ => "its end year",
         };
         eprintln!("finished at tick {} ({why})", world.tick());
```

Modify `crates/sugarscape-cli/tests/cli.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -109,6 +109,14 @@ fn presets_and_sweeps_are_listed() {
         "norms-selection",
         "norms-readings",
         "norms-dominance",
+        "ra-clusters",
+        "ra-map",
+        "ra-readings",
+        "ra-population",
+        "ra-rules",
+        "ra-delta",
+        "ad-connectivity",
+        "w-dispersion",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -470,6 +478,16 @@ fn a_norms_run_stops_at_its_last_generation() {
     assert_eq!(stderr(&out), "finished at tick 100 (its last generation)\n");
 }
 
+#[test]
+fn an_agreement_run_stops_when_stable_or_at_its_last_period() {
+    let out = sugarscape(&["run", "--preset", "ra-literal", "--ticks", "20000"]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    assert_eq!(stderr(&out), "finished at tick 376 (stable)\n");
+    let out = sugarscape(&["run", "--preset", "ra-meadows-cliff", "--ticks", "1000"]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    assert_eq!(stderr(&out), "finished at tick 200 (its last period)\n");
+}
+
 #[test]
 fn a_social_structure_run_stops_at_its_last_period() {
     let out = sugarscape(&["run", "--preset", "cra-rwr", "--ticks", "3000"]);
```

Modify `crates/sugarscape-wasm/tests/web.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -311,7 +311,15 @@ fn builtins_and_series_names_are_listed() {
             "norms-temptation",
             "norms-selection",
             "norms-readings",
-            "norms-dominance"
+            "norms-dominance",
+            "ra-clusters",
+            "ra-map",
+            "ra-readings",
+            "ra-population",
+            "ra-rules",
+            "ra-delta",
+            "ad-connectivity",
+            "w-dispersion"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -816,6 +824,24 @@ fn norms_sims_match_the_native_golden_entries() {
     }
 }
 
+#[wasm_bindgen_test]
+fn agreement_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: pair meetings on
+    // every network, and each rule.
+    for (id, fp) in [
+        ("ra-literal", "0x2a130dc7026094f7"),
+        ("ra-bc-extremists", "0xc304b87400a0bb46"),
+        ("dnaw-lattice", "0x60f9414e1157482d"),
+        ("ad-small-world", "0xd29498f3ac055d5d"),
+        ("w-scale-free", "0xed9e58b01a7987d8"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "agreement");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
 #[wasm_bindgen_test]
 fn dpd_sims_match_the_native_golden_entries() {
     // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: wealth sums and
```

- [ ] **Step 3: Run the tests**

Run: `cargo test --release -p sugarscape-core sweep:: && cargo test --release -p sugarscape-cli && wasm-pack test --node crates/sugarscape-wasm`
Expected: all pass; WASM 43 passed, including `agreement_sims_match_the_native_golden_entries` and the built-in list.

- [ ] **Step 4: Measure the sweeps against their descriptions**

```bash
cargo build --release -p sugarscape-cli
for s in ra-clusters ra-map ra-readings ra-population ra-rules ra-delta ad-connectivity w-dispersion; do
  ./target/release/sugarscape sweep --builtin $s --quiet --summary-csv /tmp/$s.csv >/dev/null && echo "== $s" && cut -d, -f2,3,5 /tmp/$s.csv
done
```
Expected: the means quoted in each description (to two decimals). All eight take about 25 s together; `w-dispersion` is the slowest (about 16 s).

- [ ] **Step 5: Format, lint, commit**

Run: `cargo fmt --all && cargo clippy --all-targets -- -D warnings`

```bash
git add sweeps/ra-*.json sweeps/ad-connectivity.json sweeps/w-dispersion.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure Relative Agreement: eight sweeps, the CLI's stops and WASM agreement

Claude-Session: https://claude.ai/code/session_011pY2cJwAezJH4xb32Aw8xr"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/{types,models,engine,compare-presets}.ts`, `web/src/experiments/form.ts`, `web/src/ui/{series-data,inspect-panel}.ts`
- Test: `web/src/{models,compare-presets,engine,determinism}.test.ts`, `web/src/experiments/form.test.ts`, `web/src/ui/series-data.test.ts`

**Interfaces:**
- Consumes: the WASM `Sim` (Task 1's model through the existing bindings), preset ids, series names, `AgreementInspection`'s JSON shape (`site`, `panel`, `period`, `opinion`, `agents`, `agent: null`).
- Produces: `AgreementConfig`, `AgreementStats`, `AgreementAgent`, `AgreementInspection` types; `isAgreementView`; `COLOR_MODES.agreement` (`uncertainty`, `role`, `start`); `MODEL_CHARTS.agreement`; the Compare entry `ra-meadows-cliff-vs-reply`.

- [ ] **Step 1: Write the failing tests**

Modify `web/src/models.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -3,6 +3,7 @@ import {
   calendarYear,
   COLOR_MODES,
   finishesUnpredictably,
+  isAgreementView,
   isCivilView,
   isClassesView,
   isOpinionsView,
@@ -104,6 +105,29 @@ describe('the anasazi model', () => {
   });
 });
 
+describe('the relative agreement model', () => {
+  it('is read by its tag, and its inspections by their panel', () => {
+    const c = { model: 'agreement', stop_at: 200 } as unknown as ModelConfig;
+    expect(modelOf(c)).toBe('agreement');
+    const cell = { site: { x: 1, y: 2 }, panel: 'diagram', period: 0, opinion: 0.5, agents: [], agent: null } as unknown as AnyInspection;
+    const hk = { site: { x: 1, y: 2 }, period: 0, opinion: 0.5, lattice_site: null, agents: [], agent: null } as unknown as AnyInspection;
+    expect([cell, hk].map(isAgreementView)).toEqual([true, false]);
+    expect([isOpinionsView(cell), isNormsView(cell)]).toEqual([false, false]);
+  });
+
+  it('colors three ways, has no overlays, and stops at its period or unpredictably when stable', () => {
+    expect(COLOR_MODES.agreement).toEqual([
+      ['uncertainty', 'Uncertainty'],
+      ['role', 'Role'],
+      ['start', 'Start'],
+    ]);
+    expect(MODEL_OVERLAYS.agreement).toEqual([]);
+    const c = (stop_when_stable: boolean, stop_at: number) => ({ model: 'agreement', stop_when_stable, stop_at }) as unknown as ModelConfig;
+    expect([finishesUnpredictably(c(false, 200)), ticksLeft(c(false, 200), 40)]).toEqual([false, 160]);
+    expect([finishesUnpredictably(c(true, 20000)), ticksLeft(c(true, 20000), 40), ticksLeft(c(true, 0), 40)]).toEqual([true, 19960, Infinity]);
+  });
+});
+
 describe('the norms model', () => {
   it('is read by its tag, and its inspections by their plane level', () => {
     const c = { model: 'norms', stop_at: 100 } as unknown as ModelConfig;
```

Modify `web/src/compare-presets.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -42,6 +42,11 @@ describe('compare presets', () => {
     expect([states.aSeed, states.b.seed]).toEqual([9, 9]);
   });
 
+  it('pairs Meadows and Cliff’s reading and the reply', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['ra-meadows-cliff-vs-reply', 'ra-meadows-cliff', 'ra-deffuant-2013', 'Meadows and Cliff vs Deffuant et al.’s reply — Relative Agreement (Compare)']);
+  });
+
   it('pairs Axelrod’s selection and a random tournament', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['gi-axelrod-vs-tournament', 'gi-metanorms-long', 'gi-tournament', 'Axelrod’s selection vs a random tournament — Norms and Metanorms (Compare)']);
```

Modify `web/src/engine.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -114,7 +114,7 @@ describe('Engine', () => {
     const { engine } = await setup();
     expect(await engine.applyConfig((c) => void (c.population = 5000))).toEqual([{ field: 'population', message: 'too many' }]);
     expect(sugar(engine.baseConfig).population).toBe(10);
-    expect(await engine.reset({ ...engine.baseConfig, population: 5000 })).toEqual([{ field: 'population', message: 'too many' }]);
+    expect(await engine.reset({ ...sugar(engine.baseConfig), population: 5000 })).toEqual([{ field: 'population', message: 'too many' }]);
     expect(await engine.erase(3, 2)).toEqual([{ field: 'edit', message: 'no agent at (3, 2)' }]);
     expect(await engine.applyConfig((c) => void (c.population = 20))).toBeNull();
     expect(sugar(engine.config).population).toBe(20);
@@ -1151,6 +1151,10 @@ describe('Engine with other models', () => {
     expect(finishedNotice(ring, 10)).toBe('This run has reached its end year — Reset to run it again');
     expect(finishedNotice({ model: 'civil' } as unknown as ModelConfig, 94)).toBe('A group has died out at t = 94 — Reset to run it again');
     expect(finishedNotice({ model: 'norms' } as unknown as ModelConfig, 100)).toBe('This run has reached its last generation (100) — Reset to run it again');
+    expect(finishedNotice({ model: 'agreement', stop_at: 200 } as unknown as ModelConfig, 200)).toBe('This run has reached its last period (200) — Reset to run it again');
+    expect(finishedNotice({ model: 'agreement', stop_at: 20000 } as unknown as ModelConfig, 376)).toBe(
+      'Stable at t = 376: no opinion or uncertainty moves any more — Reset, or change the rule, to run it again',
+    );
     expect(finishedNotice({ model: 'structure' } as unknown as ModelConfig, 2500)).toBe('This run has reached its last period (2500) — Reset to run it again');
     expect(finishedNotice({ model: 'opinions' } as unknown as ModelConfig, 8)).toBe(
       'Stable at t = 8: no opinion moves any more — Reset, or change confidence or updating, to run it again',
```

Modify `web/src/experiments/form.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -147,6 +147,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('agreement')).toMatchObject({
+      x: { path: 'uncertainty', values: '0.2:2:0.2' },
+      ticks: 20000,
+      metric: { kind: 'final', series: 'y' },
+    });
     expect(defaultForm('norms')).toMatchObject({
       x: { path: 'stop_at', values: '100,1000,10000' },
       ticks: 10000,
```

Modify `web/src/ui/series-data.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,13 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('relative agreement charts', () => {
+  it('chart convergence, clusters, dispersion, opinion and uncertainty, and change over periods', () => {
+    expect(MODEL_CHARTS.agreement.map((c) => c.title)).toEqual(['Convergence', 'Clusters', 'Dispersion', 'Opinion and uncertainty', 'Change']);
+    expect(timeAxisLabel('agreement')).toBe('Periods');
+  });
+});
+
 describe('norms charts', () => {
   it('chart boldness and vengefulness, events, payoff, the norm state and (with groups) each group', () => {
     expect(MODEL_CHARTS.norms.map((c) => c.title)).toEqual(['Boldness and vengefulness', 'Events', 'Mean payoff', 'Norm state', 'By group']);
```

Modify `web/src/determinism.test.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -12,6 +12,9 @@ import { wasmSimModule } from './sim-module';
 import { InlineTransport } from './transport';
 import { decodeShare, encodeShare } from './share';
 import type {
+  AgreementConfig,
+  AgreementInspection,
+  AgreementStats,
   AnasaziStats,
   CivilConfig,
   CivilStats,
@@ -468,6 +471,16 @@ describe('other models through the engine', () => {
     ['gi-mild-metanorms', '0xb7435abcb67c192b'],
     ['gi-temptation-10', '0xa191ff11a4f9ee68'],
     ['gi-tournament', '0x95ea76458cee1a46'],
+    // Relative agreement presets still running at period 200 on seed 1 (the others settle sooner).
+    ['dnaw-lattice', '0x60f9414e1157482d'],
+    ['dnaw-lattice-clusters', '0x459f1c2e1f67651b'],
+    ['ra-central', '0xcd795ff8a5bd9e44'],
+    ['ra-literal', '0x2a130dc7026094f7'],
+    ['ra-deffuant-2013', '0x4b9cf5817bc155d6'],
+    ['ra-bc-extremists', '0xc304b87400a0bb46'],
+    ['ad-moore', '0xd39d77107d873634'],
+    ['ad-small-world', '0xd29498f3ac055d5d'],
+    ['w-scale-free', '0xed9e58b01a7987d8'],
   ];
 
   it.each(GOLDEN_MODELS)('%s reproduces its golden fingerprint, whatever is watched', async (id, golden) => {
@@ -627,6 +640,36 @@ describe('the social-structure model through the engine', () => {
   });
 });
 
+describe('the relative agreement model through the engine', () => {
+  it('stops once when stable, matches the native golden entry and inspects a column and a dot', async () => {
+    const r = presets.find((p) => p.id === 'ra-single')!;
+    const e = await Engine.create({ config: structuredClone(r.config as AgreementConfig), seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'role' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as AgreementStats;
+    // Fig. 7's stated parameters: both extremes (outcome 1), not the figure's single extreme.
+    expect([e.finished, ends, e.tick, s.stable_at, s.outcome]).toEqual([true, 1, 71, 71, 1]);
+    // crates/sugarscape-core/tests/golden.rs: ra-single stops at period 71 of its 200.
+    expect(await e.fingerprint()).toBe('0x769843f421a9f63e');
+    await e.select(0, 100);
+    const v = e.inspection!.view as AgreementInspection;
+    expect([v.panel, v.period, v.opinion]).toEqual(['diagram', 0, 0]);
+    expect(e.inspection!.agentId).toBeNull();
+    await e.select(249 + 100, 100);
+    expect((e.inspection!.view as AgreementInspection).panel).toBe('scatter');
+  });
+
+  it('stops at Meadows and Cliff’s horizon', async () => {
+    const r = presets.find((p) => p.id === 'ra-meadows-cliff')!;
+    const e = await Engine.create({ config: structuredClone(r.config as AgreementConfig), seed: 1 }, { presets, transport: inline() });
+    await e.advance(1_000_000);
+    const s = e.latest as AgreementStats;
+    expect([e.finished, e.tick, s.y]).toEqual([true, 200, 0]);
+  });
+});
+
 describe('the bounded-confidence model through the engine', () => {
   it('stops once when stable, matches the native golden entry and inspects a line', async () => {
     const r = presets.find((p) => p.id === 'hk-regular-50')!;
```

Also fix one existing test whose object literal no longer type-checks against the wider `ModelConfig` union (its config is a sugarscape one; it already reads it through `sugar()`): included in the `engine.test.ts` diff above (`{ ...sugar(engine.baseConfig), population: 5000 }`).

- [ ] **Step 2: Run them to see them fail**

Run: `(cd web && npm run build)`
Expected: FAIL — `tsc` reports the missing `AgreementConfig`, `isAgreementView` and the `agreement` records.

- [ ] **Step 3: Carry the model through the page**

Modify `web/src/types.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -81,7 +81,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -396,7 +396,36 @@ export interface NormsConfig {
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig;
+/**
+ * Deffuant et al.'s relative agreement (milestone 22): random pairs meet and move each other's
+ * opinions and uncertainties; extremists, three bounded-confidence rules, networks, and the readings
+ * the papers leave open (Meadows and Cliff's, the 2013 reply's, eq. 11's window) as switches.
+ */
+export interface AgreementConfig {
+  model: 'agreement';
+  agents: number;
+  rule: 'ra' | 'bc' | 'bc_averaging' | 'bc_variance';
+  window: 'influencer' | 'listener';
+  mu: number;
+  alpha: number;
+  uncertainty: number;
+  extremists: number;
+  extremist_uncertainty: number;
+  delta: number;
+  placement: 'drawn' | 'bounds' | 'band';
+  band: number;
+  extreme_margin: number;
+  pair_update: 'simultaneous' | 'sequential' | 'one_way';
+  network: 'all' | 'lattice' | 'small_world' | 'scale_free';
+  lattice: { width: number; height: number; neighborhood: 'moore' | 'von_neumann' };
+  small_world: { substrate: 'ring' | 'grid'; degree: number; rewire: number };
+  scale_free: { links: number };
+  pairing: 'edge' | 'node';
+  stop_when_stable: boolean;
+  stop_at: number;
+}
+
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig;
 
 export interface Preset { id: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -651,7 +680,28 @@ export interface NormsStats {
   copied_equal: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats;
+export interface AgreementStats {
+  tick: number;
+  /** DAWF's indicator: the squared shares of moderates turned extremist at each end, summed. */
+  y: number;
+  p_plus: number;
+  p_minus: number;
+  /** 0 central, 1 both extremes, 2 single extreme, 3 intermediate. */
+  outcome: number;
+  clusters: number;
+  major: number;
+  isolated: number;
+  largest: number;
+  second: number;
+  dispersion: number;
+  unmoved: number;
+  mean_opinion: number;
+  mean_uncertainty: number;
+  max_change: number;
+  stable_at: number;
+}
+
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -922,7 +972,31 @@ export interface NormsInspection {
   agent: NormAgentView | null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection;
+/** An agent at an inspected cell: its opinion and uncertainty there, where it started, and its meetings. */
+export interface AgreementAgent {
+  id: number;
+  role: 'plus' | 'minus' | 'moderate';
+  start: number;
+  opinion: number;
+  uncertainty: number;
+  degree: number;
+  meetings: number;
+  moves: number;
+}
+/**
+ * A cell of the opinion × time diagram, the start-against-now panel or the torus, and the agents
+ * there. `agent` is always null: a clicked cell is read again each period.
+ */
+export interface AgreementInspection {
+  site: { x: number; y: number };
+  panel: 'diagram' | 'scatter' | 'torus' | null;
+  period: number | null;
+  opinion: number | null;
+  agents: AgreementAgent[];
+  agent: null;
+}
+
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -967,7 +1041,9 @@ export type ColorMode =
   | 'provocability'
   | 'strategy'
   | 'surrounded'
-  | 'agents';
+  | 'agents'
+  | 'uncertainty'
+  | 'role';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
```

Modify `web/src/models.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -1,6 +1,8 @@
 // Which model a config is (milestones 9–17), and what each model offers the page.
 import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
 import type {
+  AgreementConfig,
+  AgreementInspection,
   AnasaziInspection,
   AnyInspection,
   CivilConfig,
@@ -31,7 +33,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -49,12 +51,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   structure: 'Social Structure',
   dpd: 'Demographic PD',
   norms: 'Norms and Metanorms',
+  agreement: 'Relative Agreement',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement'
     ? tag
     : 'sugarscape';
 }
@@ -131,6 +134,11 @@ export function isNormsView(v: AnyInspection): v is NormsInspection {
   return 'level' in v && 'agents' in v;
 }
 
+/** A cell of the relative agreement frame (it names its panel). */
+export function isAgreementView(v: AnyInspection): v is AgreementInspection {
+  return 'panel' in v && 'agents' in v;
+}
+
 export function isDpdView(v: AnyInspection, model: ModelKind): v is DpdInspection {
   return model === 'dpd' && (v.agent === null || 'surrounded' in v.agent);
 }
@@ -152,6 +160,8 @@ export function ticksLeft(c: ModelConfig, tick: number): number {
   if (modelOf(c) === 'structure' && (c as StructureConfig).stop_at > 0) return Math.max(0, (c as StructureConfig).stop_at - tick);
   if (modelOf(c) === 'dpd' && (c as DpdConfig).end > 0) return Math.max(0, (c as DpdConfig).end - tick);
   if (modelOf(c) === 'norms' && (c as NormsConfig).stop_at > 0) return Math.max(0, (c as NormsConfig).stop_at - tick);
+  // Relative agreement's stop_at is its horizon, or a cap on a run that stops when stable.
+  if (modelOf(c) === 'agreement' && (c as AgreementConfig).stop_at > 0) return Math.max(0, (c as AgreementConfig).stop_at - tick);
   return Infinity;
 }
 
@@ -165,6 +175,7 @@ export function finishesUnpredictably(c: ModelConfig): boolean {
   if (model === 'culture') return (c as CultureConfig).stop_when_stable && (c as CultureConfig).drift === 0;
   if (model === 'classes') return (c as ClassesConfig).stop_at_equity;
   if (model === 'opinions') return (c as OpinionsConfig).stop_when_stable;
+  if (model === 'agreement') return (c as AgreementConfig).stop_when_stable;
   if (model === 'sugarscape') return (c as Config).culture.rule === 'axelrod' && (c as Config).culture.stop_when_settled === true;
   return model === 'civil' && (c as CivilConfig).variant === 'ethnic' && (c as CivilConfig).stop_at_extinction;
 }
@@ -268,6 +279,12 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['payoff', 'Payoff'],
     ['group', 'Group'],
   ],
+  // Confident red to uncertain green (DAWF's figures); the initial extremists; the start.
+  agreement: [
+    ['uncertainty', 'Uncertainty'],
+    ['role', 'Role'],
+    ['start', 'Start'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -286,4 +303,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   structure: [],
   dpd: [],
   norms: [],
+  agreement: [],
 };
```

Modify `web/src/engine.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -24,7 +24,7 @@ import { calendarYear, isSugar, modelOf, ticksLeft } from './models';
 import { MAX_TICKS, SimHost } from './sim-host';
 import { wasmSimModule } from './sim-module';
 import { InlineTransport, startWorker, type Transport } from './transport';
-import type { ColorMode, Config, FieldError, Layer, ModelConfig, ModelKind, ModelStats, Param, Preset } from './types';
+import type { AgreementConfig, ColorMode, Config, FieldError, Layer, ModelConfig, ModelKind, ModelStats, Param, Preset } from './types';
 import init, { model_schemas_json, presets_json } from './wasm-pkg/sugarscape.js';
 
 export type { Overlay, PlaceOverrides } from './protocol';
@@ -61,6 +61,12 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'ethno' || modelOf(config) === 'structure') return `This run has reached its last period (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'dpd') return `This run has reached its last cycle (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'norms') return `This run has reached its last generation (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'agreement') {
+    const stop = (config as AgreementConfig).stop_at;
+    return stop > 0 && tick >= stop
+      ? `This run has reached its last period (${tick}) — Reset to run it again`
+      : `Stable at t = ${tick}: no opinion or uncertainty moves any more — Reset, or change the rule, to run it again`;
+  }
   const year = calendarYear(config, tick);
   return `This run has reached its end year${year === null ? '' : ` (AD ${year})`} — Reset to run it again`;
 }
```

Modify `web/src/compare-presets.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -122,6 +122,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'gi-metanorms-long',
     b: 'gi-tournament',
   },
+  {
+    id: 'ra-meadows-cliff-vs-reply',
+    label: 'Meadows and Cliff vs Deffuant et al.’s reply — Relative Agreement (Compare)',
+    a: 'ra-meadows-cliff',
+    b: 'ra-deffuant-2013',
+  },
 ];
 
 /**
```

Modify `web/src/experiments/form.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -84,6 +84,10 @@ export function defaultForm(model: ModelKind = 'sugarscape'): SweepForm {
       metric: { ...form.metric, kind: 'window_mean', series: 'ethnocentric', from: 1901, to: null },
     };
   }
+  if (model === 'agreement') {
+    // Fig. 9's axis (the built-in ra-map): y against the moderates' uncertainty.
+    return { ...form, x: { path: 'uncertainty', values: '0.2:2:0.2' }, ticks: 20000, metric: { ...form.metric, kind: 'final', series: 'y' } };
+  }
   if (model === 'norms') {
     // Galán & Izquierdo's horizon (the built-in norms-horizon): collapse against how long runs last.
     return { ...form, x: { path: 'stop_at', values: '100,1000,10000' }, ticks: 10000, metric: { ...form.metric, kind: 'final', series: 'collapsed' } };
```

Modify `web/src/ui/series-data.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -427,6 +427,41 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
       shown: hasGroups,
     },
   ],
+  agreement: [
+    {
+      title: 'Convergence',
+      lines: [
+        { key: 'y', label: 'y', color: '--c1' },
+        { key: 'p_plus', label: 'Moderates at +1', color: '--red' },
+        { key: 'p_minus', label: 'Moderates at −1', color: '--blue' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'Clusters',
+      lines: [
+        { key: 'clusters', label: 'Clusters', color: '--c1' },
+        { key: 'major', label: 'Holding 1 % or more', color: '--c3' },
+        { key: 'isolated', label: 'Isolated agents', color: '--c4' },
+      ],
+    },
+    {
+      title: 'Dispersion',
+      lines: [
+        { key: 'dispersion', label: 'Dispersion (Σ shares²)', color: '--c1' },
+        { key: 'unmoved', label: 'Never moved', color: '--c4' },
+      ],
+      range: [0, 1],
+    },
+    {
+      title: 'Opinion and uncertainty',
+      lines: [
+        { key: 'mean_opinion', label: 'Mean opinion', color: '--c1' },
+        { key: 'mean_uncertainty', label: 'Mean uncertainty', color: '--c3' },
+      ],
+    },
+    { title: 'Change', lines: [{ key: 'max_change', label: 'Largest move this period', color: '--c2' }] },
+  ],
 };
 
 /**
@@ -434,7 +469,7 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
  * (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
+  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
```

Modify `web/src/ui/inspect-panel.ts` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -2,10 +2,11 @@ import { citizenRows, shownCitizen } from '../civil';
 import { dpdRows } from '../dpd';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
-import { isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isAgreementView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
+  AgreementInspection,
   AnasaziInspection,
   CivilInspection,
   ClassesInspection,
@@ -235,6 +236,29 @@ export class InspectPanel {
     return [row('Point', 'between the agents and the plane')];
   }
 
+  /** A cell of relative agreement's diagram, start-against-now panel or torus, and the agents there. */
+  private agreementRows(view: AgreementInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    if (view.panel === null) return [row('Point', 'between the panels')];
+    const panel = { diagram: 'Opinion × time', scatter: 'Start against now', torus: 'Lattice' }[view.panel];
+    const rows = [row('Panel', panel)];
+    if (view.period !== null) rows.push(row('Period', String(view.period)));
+    if (view.opinion !== null) rows.push(row('Opinion', fmt(view.opinion)));
+    if (view.agents.length === 0) return [...rows, row('Agents', 'none here')];
+    const role = { plus: 'extremist (+1)', minus: 'extremist (−1)', moderate: 'moderate' };
+    const shown = view.agents.slice(0, 12);
+    for (const a of shown) {
+      rows.push(
+        row(
+          `#${a.id}`,
+          `${fmt(a.opinion)} ± ${fmt(a.uncertainty)} · ${role[a.role]} · started ${fmt(a.start)} · ${a.degree} neighbors · moved in ${a.moves} of ${a.meetings} meetings`,
+        ),
+      );
+    }
+    if (view.agents.length > shown.length) rows.push(row('', `and ${view.agents.length - shown.length} more`));
+    return rows;
+  }
+
   /** A cell of the opinion × time diagram (its period, opinion and the agents passing) or of the lattice. */
   private opinionsRows(view: OpinionsInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -383,6 +407,8 @@ export class InspectPanel {
           ? this.dpdSiteRows(view, gone)
           : isNormsView(view)
             ? this.normsRows(view)
+          : isAgreementView(view)
+            ? this.agreementRows(view)
           : isStructureView(view)
             ? this.structureRows(view)
             : isOpinionsView(view)
```

- [ ] **Step 4: Run the page's build and tests**

Run: `(cd web && npm run build && npm test)`
Expected: builds; 626 tests pass (45 files).

- [ ] **Step 5: Commit**

```bash
git add web/src
git commit -m "Carry Relative Agreement through the page

Claude-Session: https://claude.ai/code/session_011pY2cJwAezJH4xb32Aw8xr"
```

- [ ] **Step 6 (controller): check it in the browser**

Run `(cd web && npm run dev)` and check, for each of the sixteen presets in the **Relative Agreement** group: the frame (opinion × time, start against now, and the torus for `dnaw-lattice`, `dnaw-lattice-clusters`, `ad-moore`), the three color modes, the five charts, Inspect on a diagram column, a scatter dot, a torus site and the gap between panels; the stops (`ra-single` stable at 71 with the notice "Stable at t = 71: no opinion or uncertainty moves any more — Reset, or change the rule, to run it again"; `ra-meadows-cliff` at 200 with "This run has reached its last period (200) — Reset to run it again"); the Compare entry; Rules panel visibility (`window` only for BC rules, `alpha` only for `bc_variance`, `band` only for `band`, each network's fields only with it); Experiments' default for this model; recording; and that every other model's presets still load.

---

### Task 4: The survey's relative agreement claims

**Files:**
- Create: `survey/src/claims/agreement.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `sugarscape_core::agreement::{AgreementConfig, Network, PairUpdate, Pairing, Placement, Rule, ScaleFreeConfig, SmallWorldConfig, Substrate, Window, GAP}`, `sugarscape_core::opinions::{groups, Neighborhood}`, `crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict}`, `crate::runner::model_after`.
- Produces: 36 claims with ids `agreement.*`.

- [ ] **Step 1: Write the claims**

Create `survey/src/claims/agreement.rs` with exactly this content:

```rust
//! Relative agreement (milestone 22): Deffuant et al.'s pairwise bounded
//! confidence (2000), relative agreement with extremists (2002) and its §6
//! variants, Meadows and Cliff's replication (2012) and the authors' reply
//! (2013), Amblard and Deffuant's networks (2004) and Weisbuch's (2004).
//! Runs go to stability unless a claim says otherwise; runs several claims
//! share are memoized per process, keyed by the config, the seeds and the cap.

use std::sync::{Arc, Mutex};

use sugarscape_core::agreement::{
    AgreementConfig, Network, PairUpdate, Pairing, Placement, Rule, ScaleFreeConfig,
    SmallWorldConfig, Substrate, Window, GAP,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};
use sugarscape_core::opinions::groups;

use crate::claim::{all_of, equivalent, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const DNAW: &str = "Deffuant, Neau, Amblard & Weisbuch 2000, ACS 3";
const DAWF: &str = "Deffuant, Amblard, Weisbuch & Faure 2002, JASSS 5(4) 1";
const MC: &str = "Meadows & Cliff 2012, JASSS 15(4) 4";
const DAW: &str = "Deffuant, Amblard & Weisbuch 2013, JASSS 16(1) 11";
const AD: &str = "Amblard & Deffuant 2004, Physica A 343";
const W: &str = "Weisbuch 2004, EPJ B 38";
/// Far past any fully connected stability time (hundreds of periods).
const CAP: u32 = 20_000;

/// One run at its end.
#[derive(Clone, Copy, Debug)]
struct Run {
    y: f64,
    p_plus: f64,
    p_minus: f64,
    outcome: u32,
    major: f64,
    isolated: f64,
    largest: f64,
    dispersion: f64,
    unmoved: f64,
    /// |mean opinion|: how far the population has drifted.
    drift: f64,
    tick: f64,
    stable: bool,
    /// The share of the ten best-connected agents in the largest cluster
    /// (NaN when anyone meets anyone).
    hubs_in_largest: f64,
}

impl Run {
    fn central(&self) -> bool {
        self.outcome == 0
    }
    fn both(&self) -> bool {
        self.outcome == 1
    }
    fn single(&self) -> bool {
        self.outcome == 2
    }
}

fn summarize(w: &ModelWorld) -> Run {
    let m = w.model();
    let last = |n: &str| m.latest_value(n).expect("an agreement series");
    let hubs_in_largest = match &w {
        ModelWorld::Agreement(a) if a.config.network != Network::All => {
            // agents.csv: id,role,start,opinion,uncertainty,degree,meetings,moves
            let rows: Vec<(f64, u32)> = m
                .agents_csv()
                .lines()
                .skip(1)
                .map(|l| {
                    let f: Vec<&str> = l.split(',').collect();
                    (f[3].parse().unwrap(), f[5].parse().unwrap())
                })
                .collect();
            let mut sorted: Vec<f64> = rows.iter().map(|r| r.0).collect();
            sorted.sort_by(f64::total_cmp);
            // The largest group's opinion range.
            let sizes = groups(&sorted, GAP);
            let (mut at, mut best) = (0, (0, 0));
            for &s in &sizes {
                if s > best.1 - best.0 {
                    best = (at, at + s);
                }
                at += s;
            }
            let (lo, hi) = (sorted[best.0], sorted[best.1 - 1]);
            let mut by_degree = rows.clone();
            by_degree.sort_by(|a, b| b.1.cmp(&a.1));
            let hubs = &by_degree[..10];
            hubs.iter().filter(|r| r.0 >= lo && r.0 <= hi).count() as f64 / 10.0
        }
        _ => f64::NAN,
    };
    let c = match m.config() {
        ModelConfig::Agreement(c) => c,
        _ => unreachable!(),
    };
    let tick = m.tick() as f64;
    Run {
        y: last("y"),
        p_plus: last("p_plus"),
        p_minus: last("p_minus"),
        outcome: last("outcome") as u32,
        major: last("major"),
        isolated: last("isolated"),
        largest: last("largest"),
        dispersion: last("dispersion"),
        unmoved: last("unmoved"),
        drift: last("mean_opinion").abs(),
        tick,
        stable: c.stop_when_stable
            && m.finished()
            && (c.stop_at == 0 || tick < f64::from(c.stop_at)),
        hubs_in_largest,
    }
}

/// The model's defaults with `edit`, seeds 1..=`seeds`, at most `cap` periods.
fn runs(seeds: u64, cap: u32, edit: impl FnOnce(&mut AgreementConfig)) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u64, u32, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = AgreementConfig::default();
    edit(&mut c);
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((_, _, _, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, s, t, _)| *k == key && *s == seeds && *t == cap)
    {
        return v.clone();
    }
    let seed_list: Vec<u64> = (1..=seeds).collect();
    let v = Arc::new(model_after(
        &ModelConfig::Agreement(c),
        &seed_list,
        cap,
        summarize,
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, seeds, cap, v.clone()));
    v
}

fn mean(runs: &[Run], f: impl Fn(&Run) -> f64) -> f64 {
    runs.iter().map(f).sum::<f64>() / runs.len() as f64
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

/// How many runs end central, in both extremes and in one.
fn outcomes(runs: &[Run]) -> String {
    format!(
        "central {}, both extremes {}, single extreme {}, intermediate {} of {}; mean y {:.2}",
        count(runs, Run::central),
        count(runs, Run::both),
        count(runs, Run::single),
        runs.len() - count(runs, Run::central) - count(runs, Run::both) - count(runs, Run::single),
        runs.len(),
        mean(runs, |r| r.y)
    )
}

/// DNAW's pairwise model: bounded confidence without extremists, d on
/// [0, 1] as U = 2d.
fn dnaw(d: f64, agents: u32) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.rule = Rule::Bc;
        c.extremists = 0.0;
        c.uncertainty = 2.0 * d;
        c.mu = 0.5;
        c.agents = agents;
    }
}

/// Relative agreement with extremists at Fig. 9's μ and ue.
fn ra(agents: u32, pe: f64, u: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.agents = agents;
        c.extremists = pe;
        c.uncertainty = u;
    }
}

/// Meadows and Cliff's placement at pe 0.05, U 1.4, N 200, read at `stop`
/// (0: to stability) with a margin.
fn reading(margin: f64, stop: u32) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.extremists = 0.05;
        c.uncertainty = 1.4;
        c.placement = Placement::Band;
        c.extreme_margin = margin;
        if stop > 0 {
            c.stop_when_stable = false;
            c.stop_at = stop;
        }
    }
}

/// §6's rules at N 1000, pe 0.05.
fn bc(rule: Rule, window: Window, u: f64, delta: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.agents = 1000;
        c.rule = rule;
        c.window = window;
        c.extremists = 0.05;
        c.uncertainty = u;
        c.delta = delta;
    }
}

/// Amblard and Deffuant's small world: N 1000, U 1.8, pe 0.05 at ±1, μ 0.1.
fn small_world(k: u32, p: f64, margin: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        c.agents = 1000;
        c.mu = 0.1;
        c.uncertainty = 1.8;
        c.extremists = 0.05;
        c.placement = Placement::Bounds;
        c.extreme_margin = margin;
        c.network = Network::SmallWorld;
        c.small_world = SmallWorldConfig {
            substrate: Substrate::Ring,
            degree: k,
            rewire: p,
        };
    }
}

/// The smallest k (powers of 2) at which most runs end in a single extreme.
fn critical_k(p: f64, margin: f64) -> Option<u32> {
    [2, 4, 8, 16, 32, 64, 128, 256]
        .into_iter()
        .find(|&k| 2 * count(&runs(20, 5_000, small_world(k, p, margin)), Run::single) > 20)
}

/// Weisbuch's networks: pairwise bounded confidence at d, N 900, μ 0.5; on
/// networks a random agent picks a random neighbor and only it moves.
fn weisbuch(net: &'static str, d: f64) -> impl FnOnce(&mut AgreementConfig) {
    move |c| {
        dnaw(d, 900)(c);
        match net {
            "all" => {}
            "lattice" => {
                c.network = Network::Lattice;
                c.lattice.width = 30;
                c.lattice.height = 30;
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            }
            _ => {
                c.network = Network::ScaleFree;
                c.scale_free = ScaleFreeConfig {
                    links: if net == "sf8" { 4 } else { 2 },
                };
                c.pairing = Pairing::Node;
                c.pair_update = PairUpdate::OneWay;
            }
        }
    }
}

const FIG9_U: [f64; 19] = [
    0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8, 1.9, 2.0,
];

/// Fig. 9's pe axis: 0.025 to 0.3 by 0.0125.
fn fig9_pe() -> Vec<f64> {
    (0..23).map(|i| 0.025 + 0.0125 * f64::from(i)).collect()
}

/// Mean y over 50 runs at each grid cell with `keep(U, pe)`, N agents, δ.
fn fig9_cells(agents: u32, delta: f64, keep: impl Fn(f64, f64) -> bool) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for pe in fig9_pe() {
        for u in FIG9_U {
            if keep(u, pe) {
                let r = runs(50, CAP, move |c| {
                    ra(agents, pe, u)(c);
                    c.delta = delta;
                });
                out.push((u, pe, mean(&r, |r| r.y)));
            }
        }
    }
    out
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "agreement.dnaw.consensus",
            item: "dnaw-consensus",
            source: Source::Book,
            citation: DNAW,
            text: "Figs. 1–2: 'uniformity is only achieved for the larger value of d' — consensus at d 0.5, two clusters at d 0.2 (N 1000, μ 0.5)",
            check: |_| {
                let one = runs(50, CAP, dnaw(0.5, 1000));
                let two = runs(50, CAP, dnaw(0.2, 1000));
                let (a, b) = (count(&one, |r| r.largest >= 0.99), count(&two, |r| r.major == 2.0));
                outcome(a >= 45 && b >= 45, format!("consensus in {a}/50 at d 0.5; two major clusters in {b}/50 at d 0.2"))
            },
        },
        Claim {
            id: "agreement.dnaw.peaks",
            item: "dnaw-clusters",
            source: Source::Book,
            citation: DNAW,
            text: "Fig. 4: the number of peaks falls as d grows, at most about 1/(2d) (N 1000, μ 0.5, wings excluded; 50 of the paper's 250 samples)",
            check: |_| {
                let ds = [0.1, 0.15, 0.2, 0.25, 0.3, 0.35, 0.4, 0.45];
                let means: Vec<f64> = ds.iter().map(|&d| mean(&runs(50, CAP, dnaw(d, 1000)), |r| r.major)).collect();
                let falling = means.windows(2).all(|w| w[1] <= w[0] + 0.05);
                let bounded = ds.iter().zip(&means).all(|(&d, &m)| m <= 1.0 / (2.0 * d) + 0.5);
                let shown: Vec<String> = ds.iter().zip(&means).map(|(d, m)| format!("{m:.2} at d {d}")).collect();
                outcome(falling && bounded, format!("major clusters {}", shown.join(", ")))
            },
        },
        Claim {
            id: "agreement.dnaw.lattice",
            item: "dnaw-lattice",
            source: Source::Book,
            citation: DNAW,
            text: "Fig. 5: on a 29 × 29 lattice at d 0.3 'a large majority of agents … reached consensus … apart from isolated agents' (run to stability)",
            check: |_| {
                let r = runs(20, CAP, |c| {
                    dnaw(0.3, 841)(c);
                    c.mu = 0.3;
                    c.network = Network::Lattice;
                });
                let k = count(&r, |r| r.largest >= 0.8 && r.isolated >= 5.0);
                let mut o = outcome(k >= 16, format!("{k}/20 runs with a cluster of 80 % or more and 5 or more isolated agents"));
                o.detail = format!("median isolated {:.0}; mean largest {:.2}", {
                    let mut v = col(&r, |r| r.isolated);
                    v.sort_by(f64::total_cmp);
                    v[10]
                }, mean(&r, |r| r.largest));
                o
            },
        },
        Claim {
            id: "agreement.dnaw.lattice-119",
            item: "dnaw-lattice",
            source: Source::Book,
            citation: DNAW,
            text: "Fig. 5's caption: that picture 'after 100 000 iterations' (119 meetings per agent on 841 agents)",
            check: |_| {
                let r = runs(20, 119, |c| {
                    dnaw(0.3, 841)(c);
                    c.mu = 0.3;
                    c.network = Network::Lattice;
                    c.stop_when_stable = false;
                    c.stop_at = 119;
                });
                let k = count(&r, |r| r.largest >= 0.8);
                outcome(k >= 16, format!("{k}/20 runs with a cluster of 80 % or more at period 119; mean largest {:.2}", mean(&r, |r| r.largest)))
                    .with("Run to stability the picture appears (agreement.dnaw.lattice), hundreds of periods later.")
            },
        },
        Claim {
            id: "agreement.dawf.fig4-ra",
            item: "ra-clusters",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 4: with identical uncertainties, relative agreement's cluster number 'is close to w/2u' (within a fifth; 50 runs, μ 0.5, N 1000 — the figure does not say; N 200 shown too)",
            check: |_| {
                let clusters = |agents: u32, w2u: f64| mean(&runs(50, CAP, move |c| {
                    c.agents = agents;
                    c.extremists = 0.0;
                    c.mu = 0.5;
                    c.uncertainty = 1.0 / w2u;
                }), |r| r.major);
                let points = [2.0, 2.5, 10.0 / 3.0, 5.0, 10.0];
                let parts = points
                    .into_iter()
                    .map(|w2u: f64| {
                        let m = clusters(1000, w2u);
                        (format!("w/2u {w2u:.2}"), outcome((m - w2u).abs() <= 0.2 * w2u, format!("{m:.2} clusters")))
                    })
                    .collect();
                let small: Vec<String> = points.iter().map(|&w| format!("{:.2}", clusters(200, w))).collect();
                all_of(parts).with(&format!("At N 200: {} (w/2u 2, 2.5, 3.33, 5, 10).", small.join(", ")))
            },
        },
        Claim {
            id: "agreement.dawf.fig4-bc",
            item: "ra-clusters",
            source: Source::Book,
            citation: DAWF,
            text: "§2.7: in the bounded-confidence model the cluster number 'is roughly the integer part of w/2u' (50 runs, N 1000, μ 0.5, clusters of 1 % or more)",
            check: |_| {
                let parts = [2.5, 10.0 / 3.0, 4.0, 5.0, 10.0]
                    .into_iter()
                    .map(|w2u: f64| {
                        let m = mean(&runs(50, CAP, dnaw(1.0 / (2.0 * w2u), 1000)), |r| r.major);
                        let want = w2u.floor();
                        (format!("w/2u {w2u:.2}"), outcome((m - want).abs() <= 0.5, format!("{m:.2} clusters (integer part {want})")))
                    })
                    .collect();
                all_of(parts)
            },
        },
        Claim {
            id: "agreement.dawf.fig5-central",
            item: "ra-central",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 5 (pe 0.2, U 0.4, μ 0.5, N 200): central convergence, 'only a marginal part of the initially non-extremists became extremist (4%)' (at most 10 % of moderates, over 40 runs)",
            check: |_| {
                let r = runs(40, CAP, |c| {
                    c.mu = 0.5;
                    c.extremists = 0.2;
                    c.uncertainty = 0.4;
                });
                let joined = mean(&r, |r| r.p_plus + r.p_minus);
                let bounds = mean(
                    &runs(40, CAP, |c| {
                        c.mu = 0.5;
                        c.extremists = 0.2;
                        c.uncertainty = 0.4;
                        c.placement = Placement::Bounds;
                    }),
                    |r| r.p_plus + r.p_minus,
                );
                outcome(joined <= 0.1, format!("{:.0} % of moderates became extremists ({:.0} % with extremists at ±1)", 100.0 * joined, 100.0 * bounds))
            },
        },
        Claim {
            id: "agreement.dawf.fig6-both",
            item: "ra-both",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 6 (pe 0.25, U 1.2, μ 0.5, N 200): the moderates 'split and become extremists' (both extremes in most runs)",
            check: |_| {
                let r = runs(40, CAP, |c| {
                    c.mu = 0.5;
                    c.extremists = 0.25;
                    c.uncertainty = 1.2;
                });
                share(&r, Run::both, 0.8, 1.0, "in both extremes")
            },
        },
        Claim {
            id: "agreement.dawf.fig7-single",
            item: "ra-single",
            source: Source::Book,
            citation: DAWF,
            text: "Figs. 7–8 (pe 0.1, U 1.4, μ 0.5, N 200): a single extreme (98 % of moderates) or, 'for another sample … all other parameters being equals', central convergence",
            check: |_| {
                let r = runs(40, CAP, |c| {
                    c.mu = 0.5;
                    c.extremists = 0.1;
                    c.uncertainty = 1.4;
                });
                let at = runs(40, CAP, ra(200, 0.1, 1.4));
                let k = count(&r, |r| r.single() || r.central());
                outcome(2 * k >= r.len(), format!("single or central in {k}/40 runs: {}", outcomes(&r)))
                    .with(&format!("At Fig. 9's μ 0.2: {}.", outcomes(&at)))
            },
        },
        Claim {
            id: "agreement.dawf.fig9-layout",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 9 (δ 0, μ 0.2, N 1000, 50 runs a point): central on the left, both extremes in the middle above, a central diagonal, a single extreme at the bottom right",
            check: |_| {
                let y = |pe: f64, u: f64| mean(&runs(50, CAP, ra(1000, pe, u)), |r| r.y);
                let parts = vec![
                    ("left, U 0.2, pe 0.2".to_string(), { let v = y(0.2, 0.2); outcome(v < 0.15, format!("y {v:.2}")) }),
                    ("middle, U 1.0, pe 0.2".to_string(), { let v = y(0.2, 1.0); outcome((0.4..=0.6).contains(&v), format!("y {v:.2}")) }),
                    ("diagonal, U 1.0, pe 0.05".to_string(), { let v = y(0.05, 1.0); outcome(v < 0.3, format!("y {v:.2}")) }),
                    ("bottom right, U 1.6, pe 0.025".to_string(), { let v = y(0.025, 1.6); outcome(v > 0.9, format!("y {v:.2}")) }),
                ];
                all_of(parts)
            },
        },
        Claim {
            id: "agreement.dawf.fig9-single-zone",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 9 (δ 0): the single-extreme zone (mean y ≥ 0.75, brown and red) covers pe up to 0.075 at U ≥ 1.4 (N 1000, 50 runs a cell; at least 80 % of those cells)",
            check: |_| {
                let keep = |u: f64, pe: f64| u >= 1.35 && pe <= 0.076;
                let cells = fig9_cells(1000, 0.0, keep);
                let k = cells.iter().filter(|c| c.2 >= 0.75).count();
                let small = fig9_cells(200, 0.0, keep);
                let k200 = small.iter().filter(|c| c.2 >= 0.75).count();
                let rows: Vec<String> = fig9_pe()
                    .into_iter()
                    .filter(|&pe| pe <= 0.076)
                    .map(|pe| {
                        let row: Vec<&(f64, f64, f64)> = cells.iter().filter(|c| c.1 == pe).collect();
                        format!("pe {pe:.4}: {:.2}", row.iter().map(|c| c.2).sum::<f64>() / row.len() as f64)
                    })
                    .collect();
                outcome(k * 5 >= cells.len() * 4, format!("{k}/{} cells at N 1000 ({k200}/{} at N 200); mean y by pe: {}", cells.len(), small.len(), rows.join(", ")))
            },
        },
        Claim {
            id: "agreement.dawf.fig9-delta",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 9 (δ 0.1): the single-extreme zone grows — mean y ≥ 0.75 at U ≥ 1.6 for pe up to 0.15 (N 1000, 50 runs a cell; at least 80 % of those cells)",
            check: |_| {
                let cells = fig9_cells(1000, 0.1, |u, pe| u >= 1.55 && pe <= 0.151);
                let k = cells.iter().filter(|c| c.2 >= 0.75).count();
                outcome(k * 5 >= cells.len() * 4, format!("{k}/{} cells", cells.len()))
            },
        },
        Claim {
            id: "agreement.dawf.fig10",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "Fig. 10 (pe 0.125, δ 0): 'In the medium u (0.5 < U < 1) we only get both extremes'; above U 1 'either central or single extreme clustering (y close to 0 or close to 1)' (N 1000, 50 runs)",
            check: |_| {
                let mid = runs(50, CAP, ra(1000, 0.125, 0.8));
                let hi: Vec<Run> = [1.4, 1.6, 1.8].into_iter().flat_map(|u| runs(50, CAP, ra(1000, 0.125, u)).to_vec()).collect();
                let bimodal = count(&hi, |r| r.y < 0.1 || r.y > 0.9);
                let (c, s) = (count(&hi, |r| r.y < 0.1), count(&hi, |r| r.y > 0.9));
                all_of(vec![
                    ("U 0.8".into(), share(&mid, Run::both, 0.9, 1.0, "in both extremes")),
                    ("U 1.4–1.8".into(), outcome(bimodal * 10 >= hi.len() * 9 && c > 0 && s > 0, format!("{bimodal}/{} runs near 0 or 1 ({c} central, {s} single)", hi.len()))),
                ])
            },
        },
        Claim {
            id: "agreement.dawf.mu",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "§4.8: 'When the intensity of interactions (μ) increases, the both extremes convergence zone increases and the single extreme convergence zone decreases' (pe 0.1, U 1.4, N 200, 50 runs)",
            check: |_| {
                let at = |mu: f64| runs(50, CAP, move |c| {
                    ra(200, 0.1, 1.4)(c);
                    c.mu = mu;
                });
                let (lo, hi) = (at(0.1), at(0.5));
                all_of(vec![
                    ("both".into(), greater(&col(&hi, |r| f64::from(u8::from(r.both()))), &col(&lo, |r| f64::from(u8::from(r.both()))), "μ 0.5", "μ 0.1")),
                    ("single".into(), greater(&col(&lo, |r| f64::from(u8::from(r.single()))), &col(&hi, |r| f64::from(u8::from(r.single()))), "μ 0.1", "μ 0.5")),
                ])
            },
        },
        Claim {
            id: "agreement.dawf.delta",
            item: "ra-delta",
            source: Source::Book,
            citation: DAWF,
            text: "§4.8: 'When the initial bias between the extremists (δ) increases the single extreme convergence zone increases' (pe 0.1, U 1.4, N 1000, 50 runs)",
            check: |_| {
                let at = |d: f64| runs(50, CAP, move |c| {
                    ra(1000, 0.1, 1.4)(c);
                    c.delta = d;
                });
                greater(&col(&at(0.2), |r| r.y), &col(&at(0.0), |r| r.y), "δ 0.2", "δ 0")
            },
        },
        Claim {
            id: "agreement.dawf.ue",
            item: "ra-map",
            source: Source::Book,
            citation: DAWF,
            text: "§4.8: 'We did not find any significant influence of the uncertainty of the extremists (ue)' (how far the population drifts, |mean opinion|, at pe 0.05, U 1.4, N 1000: ue 0.05 against 0.2, 50 runs)",
            check: |_| {
                let at = |ue: f64, margin: f64| runs(50, CAP, move |c| {
                    ra(1000, 0.05, 1.4)(c);
                    c.extremist_uncertainty = ue;
                    c.extreme_margin = margin;
                });
                let (lo, hi) = (at(0.05, 0.1), at(0.2, 0.1));
                equivalent(&col(&lo, |r| r.drift), &col(&hi, |r| r.drift), Some(0.15), "ue 0.05", "ue 0.2").with(&format!(
                    "y (new extremists past the innermost extremist less 0.1): {:.2} at ue 0.05, {:.2} at ue 0.2; counted past 0.3 inside it, {:.2} and {:.2} — with ue 0.2 the extreme cluster settles inside the reply's cutoff.",
                    mean(&lo, |r| r.y),
                    mean(&hi, |r| r.y),
                    mean(&at(0.05, 0.3), |r| r.y),
                    mean(&at(0.2, 0.3), |r| r.y)
                ))
            },
        },
        Claim {
            id: "agreement.dawf.fig20-printed",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 20, with eq. 11 as printed (|x − x′| < u′, the influencer's uncertainty): single extreme 'around … U = 1', both extremes near U 0.4–0.5 (δ 0.1, N 1000, 50 runs)",
            check: |_| {
                let one = runs(50, CAP, bc(Rule::Bc, Window::Influencer, 1.0, 0.1));
                let half = runs(50, CAP, bc(Rule::Bc, Window::Influencer, 0.5, 0.1));
                outcome(count(&one, Run::single) >= 25, format!("U 1.0: {}; U 0.5: {}", outcomes(&one), outcomes(&half)))
                    .with("The confident extremists move toward every uncertain moderate who meets them.")
            },
        },
        Claim {
            id: "agreement.dawf.fig20-listener",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 20, with the listener's own uncertainty as the window: single extreme around U = 1, only central above U 1.2 (δ 0.1, N 1000, 50 runs; at most 2 000 periods)",
            check: |_| {
                let one = runs(50, 2_000, bc(Rule::Bc, Window::Listener, 1.0, 0.1));
                let high = runs(50, 2_000, bc(Rule::Bc, Window::Listener, 1.6, 0.1));
                outcome(
                    count(&one, Run::single) >= 40 && count(&high, Run::single) == 0,
                    format!("U 1.0: {}; U 1.6: {}", outcomes(&one), outcomes(&high)),
                )
            },
        },
        Claim {
            id: "agreement.dawf.fig21",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 21 (averaging uncertainties, the listener's window): 'a band of central convergence for U close to 0.8'; above U 1.1 single or central, single when δ > 0 (N 1000, 50 runs)",
            check: |_| {
                let band = runs(50, CAP, bc(Rule::BcAveraging, Window::Listener, 0.8, 0.1));
                let high = runs(50, CAP, bc(Rule::BcAveraging, Window::Listener, 1.6, 0.1));
                all_of(vec![
                    ("U 0.8".into(), share(&band, Run::central, 0.8, 1.0, "central")),
                    ("U 1.6".into(), share(&high, Run::single, 0.5, 1.0, "in a single extreme")),
                ])
            },
        },
        Claim {
            id: "agreement.dawf.fig22",
            item: "ra-rules",
            source: Source::Book,
            citation: DAWF,
            text: "§6, Fig. 22 (uncertainty from variance): 'the absence of single extreme convergence, and the very rare presence of double extreme convergence' — mean y below 0.15 (the figure's white) at every point (U 0.4–2, pe 0.05 and 0.2, δ 0.1, N 1000, 20 runs a point, α 0.8, either window)",
            check: |_| {
                let mut single = 0;
                let mut both = 0;
                let mut n = 0;
                let mut top: f64 = 0.0;
                for window in [Window::Listener, Window::Influencer] {
                    for pe in [0.05, 0.2] {
                        for u in [0.4, 0.8, 1.2, 1.6, 2.0] {
                            let r = runs(20, CAP, move |c| {
                                bc(Rule::BcVariance, window, u, 0.1)(c);
                                c.extremists = pe;
                            });
                            single += count(&r, Run::single);
                            both += count(&r, Run::both);
                            n += r.len();
                            top = top.max(mean(&r, |r| r.y));
                        }
                    }
                }
                outcome(top < 0.15, format!("largest mean y {top:.2}; single extreme in {single}, both extremes in {both} of {n} runs"))
            },
        },
        Claim {
            id: "agreement.mc.no-single",
            item: "ra-meadows-cliff",
            source: Source::Book,
            citation: MC,
            text: "Meadows & Cliff: 'no conditions under which single extreme convergence will occur in the majority of the simulations' — under their reading (band, 200 periods, cutoff 0.8) at Fig. 9's corner (pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| {
                let r = runs(50, CAP, reading(0.0, 200));
                share(&r, Run::single, 0.0, 0.49, "in a single extreme")
                    .with(&format!("Mean |opinion| {:.2}: the majority has drifted but not past 0.8.", mean(&r, |r| r.drift)))
            },
        },
        Claim {
            id: "agreement.daw.single",
            item: "ra-deffuant-2013",
            source: Source::Book,
            citation: DAW,
            text: "The reply: with 1 200 meetings per agent and new extremists past ±0.7, 'the single extreme convergence is very frequent (often more than 80% of the simulations) for low values of pe and large values of U' (pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| share(&runs(50, CAP, reading(0.1, 1200)), Run::single, 0.8, 1.0, "in a single extreme"),
        },
        Claim {
            id: "agreement.daw.both-fixes",
            item: "ra-readings",
            source: Source::Book,
            citation: DAW,
            text: "The reply names two fixes (the horizon and the cutoff); neither alone reproduces the single extreme (each gives mean y below 0.6 where both give above 0.9; pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| {
                let y = |m: f64, stop: u32| mean(&runs(50, CAP, reading(m, stop)), |r| r.y);
                let (mc, horizon, cutoff, daw) = (y(0.0, 200), y(0.0, 1200), y(0.1, 200), y(0.1, 1200));
                outcome(
                    horizon < 0.6 && cutoff < 0.6 && daw > 0.9,
                    format!("mean y: Meadows & Cliff {mc:.2}, horizon fixed {horizon:.2}, cutoff fixed {cutoff:.2}, both {daw:.2}"),
                )
            },
        },
        Claim {
            id: "agreement.daw.stability",
            item: "ra-literal",
            source: Source::Book,
            citation: DAW,
            text: "The reply measures y 'when all the agent opinions are completely stabilized' by a fixed 1 200 meetings per agent; the literal rule (run until nothing moves by 10⁻⁶ in a period) reads the same y (band, cutoff 0.7, pe 0.05, U 1.4, N 200, 50 runs)",
            check: |_| {
                let fixed = runs(50, CAP, reading(0.1, 1200));
                let stable = runs(50, CAP, reading(0.1, 0));
                let unstable = count(&stable, |r| !r.stable);
                equivalent(&col(&stable, |r| r.y), &col(&fixed, |r| r.y), Some(0.1), "to stability", "1 200 periods")
                    .with(&format!("{unstable} runs reached the cap; median stable period {:.0}.", {
                        let mut v = col(&stable, |r| r.tick);
                        v.sort_by(f64::total_cmp);
                        v[25]
                    }))
            },
        },
        Claim {
            id: "agreement.mc.population",
            item: "ra-population",
            source: Source::Book,
            citation: MC,
            text: "Meadows & Cliff §5.3: as N grows the single-extreme zone shrinks (y at pe 0.1, U 1.6, δ 0: N 200 against N 2000, 50 runs)",
            check: |_| {
                let at = |n: u32| runs(50, CAP, ra(n, 0.1, 1.6));
                greater(&col(&at(200), |r| r.y), &col(&at(2000), |r| r.y), "N 200", "N 2000")
            },
        },
        Claim {
            id: "agreement.daw.large-n",
            item: "ra-population",
            source: Source::Book,
            citation: DAW,
            text: "The reply: single extreme convergence 'takes place with any large number of agents' (single extreme in at least a third of runs at N 2000; pe 0.05 and 0.1, U 1.6, δ 0, 50 runs)",
            check: |_| {
                let parts = [0.05, 0.1]
                    .into_iter()
                    .map(|pe| (format!("pe {pe}"), share(&runs(50, CAP, ra(2000, pe, 1.6)), Run::single, 0.33, 1.0, "in a single extreme")))
                    .collect();
                all_of(parts).with("With δ 0.1 the single extreme grows with N (the ra-population sweep): a lean decides it, balanced extremists only chance.")
            },
        },
        Claim {
            id: "agreement.ad.moore",
            item: "ad-moore",
            source: Source::Book,
            citation: AD,
            text: "Fig. 3: on a Moore torus 'y is always below 0.6 which shows that the single extreme convergence never occurs' (30 × 30, μ 0.2, extremists at ±1, U 0.4–1.8, pe 0.05–0.3, 10 runs a point, at most 20 000 periods)",
            check: |_| {
                let mut all = Vec::new();
                for pe in [0.05, 0.1, 0.2, 0.3] {
                    for u in [0.4, 0.8, 1.2, 1.8] {
                        all.extend(runs(10, CAP, move |c| {
                            c.network = Network::Lattice;
                            c.lattice.width = 30;
                            c.lattice.height = 30;
                            c.lattice.neighborhood = sugarscape_core::opinions::Neighborhood::Moore;
                            c.extremists = pe;
                            c.uncertainty = u;
                            c.placement = Placement::Bounds;
                        }).iter().copied());
                    }
                }
                let top = all.iter().map(|r| r.y).fold(0.0, f64::max);
                outcome(top < 0.6 && count(&all, Run::single) == 0, format!("largest y {top:.2}; {}", outcomes(&all)))
            },
        },
        Claim {
            id: "agreement.ad.critical-k",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Figs. 4–5: single extreme needs a critical connectivity, which 'takes place for higher connectivity when p decreases' (ring, N 1000, 20 runs a point, at most 5 000 periods)",
            check: |_| {
                let (low, high) = (critical_k(0.2, 0.1), critical_k(1.0, 0.1));
                let shown = |k: Option<u32>| k.map_or("none up to 256".to_string(), |k| k.to_string());
                outcome(
                    matches!((low, high), (Some(a), Some(b)) if a > b),
                    format!("most runs single from k {} at p 0.2, from k {} at p 1", shown(low), shown(high)),
                )
            },
        },
        Claim {
            id: "agreement.ad.around-8",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Fig. 5 (β 0.8): 'the phase transition … occurs for values of connectivity around 8' (most runs single from k 4, 8 or 16)",
            check: |_| {
                let k = critical_k(0.8, 0.1);
                outcome(matches!(k, Some(4..=16)), format!("most runs single from k {}", k.map_or("none".into(), |k| k.to_string())))
            },
        },
        Claim {
            id: "agreement.ad.low-k-both",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Fig. 4: at low connectivity 'double extreme convergence' (k 2 and 4, p 0.8; new extremists counted past 0.9, the reply's rule for extremists at ±1)",
            check: |_| {
                let low: Vec<Run> = [2, 4].into_iter().flat_map(|k| runs(20, 5_000, small_world(k, 0.8, 0.1)).to_vec()).collect();
                let wide: Vec<Run> = [2, 4].into_iter().flat_map(|k| runs(20, 5_000, small_world(k, 0.8, 0.3)).to_vec()).collect();
                share(&low, Run::both, 0.5, 1.0, "in both extremes")
                    .with(&format!("Counted past 0.7 instead: {}.", outcomes(&wide)))
            },
        },
        Claim {
            id: "agreement.ad.grid",
            item: "ad-connectivity",
            source: Source::Book,
            citation: AD,
            text: "Fig. 6: on a grid substrate 'the same phenomenon' — more single extremes as connectivity rises (32 × 32, p 0.8, k 8 against 120, 20 runs)",
            check: |_| {
                let at = |k: u32| runs(20, 5_000, move |c| {
                    small_world(k, 0.8, 0.1)(c);
                    c.lattice.width = 32;
                    c.lattice.height = 32;
                    c.small_world.substrate = Substrate::Grid;
                });
                greater(&col(&at(120), |r| r.y), &col(&at(8), |r| r.y), "k 120", "k 8")
            },
        },
        Claim {
            id: "agreement.w.steps",
            item: "w-dispersion",
            source: Source::Book,
            citation: W,
            text: "Fig. 2: well mixed, 'two distinct steps at y = 0.5 and y = 0.33' (dispersion 0.45–0.55 at d 0.2 and 0.25, 0.28–0.4 at d 0.15; N 900, 50 runs)",
            check: |_| {
                let d = |x: f64| mean(&runs(50, CAP, weisbuch("all", x)), |r| r.dispersion);
                let (a, b, c) = (d(0.15), d(0.2), d(0.25));
                outcome(
                    (0.28..=0.4).contains(&a) && (0.45..=0.55).contains(&b) && (0.45..=0.55).contains(&c),
                    format!("dispersion {a:.2}, {b:.2}, {c:.2} at d 0.15, 0.2, 0.25"),
                )
            },
        },
        Claim {
            id: "agreement.w.scale-free",
            item: "w-dispersion",
            source: Source::Book,
            citation: W,
            text: "Figs. 2–3: on scale-free networks 'a continuous increase … with only a kink in the d = 0.25, y = 0.7 region', similar to the square lattice (a rise through 0.5–0.8 at d 0.25, and a mean difference from the lattice of at most 0.1; N 900, 50 runs)",
            check: |_| {
                let ds = [0.15, 0.2, 0.25, 0.3];
                let curve = |net: &'static str| -> Vec<f64> { ds.iter().map(|&d| mean(&runs(50, CAP, weisbuch(net, d)), |r| r.dispersion)).collect() };
                let (sf, lat) = (curve("sf4"), curve("lattice"));
                let rising = sf.windows(2).all(|w| w[1] > w[0]);
                let near = sf.iter().zip(&lat).map(|(a, b)| (a - b).abs()).sum::<f64>() / ds.len() as f64 <= 0.1;
                let show = |v: &[f64]| v.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join(", ");
                outcome(rising && near && (0.5..=0.8).contains(&sf[2]), format!("scale-free {}; lattice {} (d 0.15–0.3)", show(&sf), show(&lat)))
            },
        },
        Claim {
            id: "agreement.w.connectivity",
            item: "w-dispersion",
            source: Source::Book,
            citation: W,
            text: "Fig. 3: 'Increasing the average connectivity by a factor 2 brings the scale free network results closer to those of the well-mixed case' (d 0.15–0.3, N 900, 50 runs)",
            check: |_| {
                let gap = |net: &'static str| -> f64 {
                    [0.15, 0.2, 0.25, 0.3]
                        .iter()
                        .map(|&d| (mean(&runs(50, CAP, weisbuch(net, d)), |r| r.dispersion) - mean(&runs(50, CAP, weisbuch("all", d)), |r| r.dispersion)).abs())
                        .sum()
                };
                let (four, eight) = (gap("sf4"), gap("sf8"));
                outcome(eight < four, format!("summed distance from well mixed: 4 links {four:.2}, 8 links {eight:.2}"))
            },
        },
        Claim {
            id: "agreement.w.hubs",
            item: "w-scale-free",
            source: Source::Book,
            citation: W,
            text: "Fig. 4: 'Most of the well connected nodes belong to horizontal cluster[s]' — the ten best-connected agents mostly end in the largest cluster (d 0.2, N 900, 50 runs)",
            check: |_| {
                let r = runs(50, CAP, weisbuch("sf4", 0.2));
                let hubs = mean(&r, |r| r.hubs_in_largest);
                let everyone = mean(&r, |r| r.largest);
                outcome(hubs > 0.5 && hubs > everyone, format!("{:.0} % of hubs in the largest cluster, which holds {:.0} % of all agents", 100.0 * hubs, 100.0 * everyone))
            },
        },
        Claim {
            id: "agreement.w.outlying",
            item: "w-scale-free",
            source: Source::Book,
            citation: W,
            text: "Weisbuch: on scale-free networks 'Many of them are not affected by the convergence process' (outlying nodes that never move), unlike the well-mixed case (d 0.2, N 900, 50 runs)",
            check: |_| {
                let sf = mean(&runs(50, CAP, weisbuch("sf4", 0.2)), |r| r.unmoved);
                let mixed = mean(&runs(50, CAP, weisbuch("all", 0.2)), |r| r.unmoved);
                outcome(sf >= 0.05 && mixed < 0.01, format!("never moved: {:.1} % on the network, {:.1} % well mixed", 100.0 * sf, 100.0 * mixed))
            },
        },
    ]
}
```

Modify `survey/src/claims/mod.rs` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -1,3 +1,4 @@
+mod agreement;
 mod ch2;
 mod ch3;
 mod ch4;
@@ -18,6 +19,7 @@ use crate::claim::Claim;
 
 pub fn all() -> Vec<Claim> {
     [
+        agreement::claims(),
         ch2::claims(),
         ch3::claims(),
         ch4::claims(),
```

- [ ] **Step 2: Run them**

Run: `cd survey && rustfmt --edition 2021 src/claims/agreement.rs && cargo build --release && ./target/release/survey --only agreement`
Expected (about 65 s on 10 cores): the verdicts below. Do not commit `survey/out/results-agreement.json` (only full-survey outputs are tracked).

| Claim | Verdict | Measured in planning |
|---|---|---|
| agreement.dnaw.consensus | Holds | consensus 50/50 at d 0.5; two major clusters 50/50 at d 0.2 |
| agreement.dnaw.peaks | Holds | 4.32, 2.98, 2.00, 1.98, 1.00 … at d 0.1 … 0.45 |
| agreement.dnaw.lattice | Holds | 20/20; median 28 isolated; mean largest 0.92 |
| agreement.dnaw.lattice-119 | Fails | 1/20 at period 119; mean largest 0.39 |
| agreement.dawf.fig4-ra | Holds | 2.00, 2.70, 3.40, 5.36, 10.54 at N 1000 (N 200: 2.10, 3.04, 3.70, 5.90, 11.28) |
| agreement.dawf.fig4-bc | Fails | 2.00, 2.98, then 3.44, 4.32, 8.92 at w/2u 4, 5, 10 |
| agreement.dawf.fig5-central | Fails | 48 % (22 % at ±1) against the caption's 4 % |
| agreement.dawf.fig6-both | Holds | 40/40 |
| agreement.dawf.fig7-single | Fails | both extremes 39/40 at μ 0.5; at μ 0.2 single 24, central 16 |
| agreement.dawf.fig9-layout | Holds | y 0.02, 0.50, 0.00, 1.00 |
| agreement.dawf.fig9-single-zone | Fails | 14/35 cells at N 1000; 35/35 at N 200 |
| agreement.dawf.fig9-delta | Holds | 55/55 cells |
| agreement.dawf.fig10 | Holds | 50/50 both at U 0.8; 150/150 near 0 or 1 (139 central, 11 single) |
| agreement.dawf.mu | Holds | both up, single down with μ |
| agreement.dawf.delta | Holds | median y 0.98 at δ 0.2, 0.00 at δ 0 |
| agreement.dawf.ue | Fails | median drift 0.03 at ue 0.05, 0.75 at 0.2; y 0.30 and 0.00 (1.00 counted 0.3 inside) |
| agreement.dawf.fig20-printed | Fails | central 50/50 at U 1.0 and 0.5 |
| agreement.dawf.fig20-listener | Holds | single 50/50 at U 1.0; central 50/50 at 1.6 |
| agreement.dawf.fig21 | Holds | central 50/50 at U 0.8; single 47/50 at 1.6 |
| agreement.dawf.fig22 | Holds | largest mean y 0.09; single 3 of 400 runs |
| agreement.mc.no-single | Holds | 0/50 under Meadows and Cliff's reading |
| agreement.daw.single | Holds | 49/50 under the reply's |
| agreement.daw.both-fixes | Holds | 0.00, 0.40, 0.27, 0.98 |
| agreement.daw.stability | Holds | equivalent; median stable period 472 |
| agreement.mc.population | Holds | N 200 median 0.99, N 2000 median 0.00 |
| agreement.daw.large-n | Fails | pe 0.05: 37/50; pe 0.1: 1/50 at N 2000 |
| agreement.ad.moore | Holds | largest y 0.51; single 0 of 160 |
| agreement.ad.critical-k | Holds | k 256 at p 0.2, k 64 at p 1 |
| agreement.ad.around-8 | Fails | k 32 at p 0.8 |
| agreement.ad.low-k-both | Fails | 0/40 with the 0.9 cutoff; 40/40 with 0.7 |
| agreement.ad.grid | Holds | median y 1.00 at k 120, 0.01 at k 8 |
| agreement.w.steps | Holds | 0.36, 0.50, 0.51 |
| agreement.w.scale-free | Holds | 0.17, 0.34, 0.60, 0.84 against the lattice's 0.02, 0.30, 0.70, 0.86 |
| agreement.w.connectivity | Holds | 0.22 (8 links) against 0.57 (4) |
| agreement.w.hubs | Holds | 62 % of hubs in a cluster of 49 % |
| agreement.w.outlying | Holds | 15.9 % against 0.1 % |

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/agreement.rs survey/src/claims/mod.rs
git commit -m "Survey Relative Agreement, Meadows and Cliff's replication and the reply

Claude-Session: https://claude.ai/code/session_011pY2cJwAezJH4xb32Aw8xr"
```

---

### Task 5: README, roadmap, papers index, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-26-relative-agreement-design.md`

- [ ] **Step 1: Write the docs**

If image scoring (Milestone 21) has merged into this branch first, `docs/papers.md`'s queue will already have lost its image-scoring row and gained a row 21: keep those, add row 22 after it, and remove only the relative-agreement queue row, renumbering what follows.

Modify `README.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/README.md
+++ b/README.md
@@ -142,7 +142,7 @@ Model extensions:
 The presets menu groups its presets by model: **Sugarscape**, **Schelling**, **Ring World**,
 **Artificial Anasazi**, **Civil Violence**, **Tag Cooperation**, **Spatial Games**, **Axelrod Culture**,
 **Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence**, **Social Structure**,
-**Demographic PD** and **Norms and Metanorms**.
+**Demographic PD**, **Norms and Metanorms** and **Relative Agreement**.
 Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
 (Max included), Share, Export, Record, Compare, Experiments and the CLI work the same for every
 model. A config without a `model` key is a sugarscape config, so every older config, link, session
@@ -1036,6 +1036,86 @@ Credit: Robert Axelrod, "An Evolutionary Approach to Norms," *American Political
 Lessons Learned Re-Implementing Axelrod's 'Evolutionary Approach to Norms'," *JASSS* 8(3) 2 (2005).
 See `docs/superpowers/specs/2026-09-26-norms-design.md`.
 
+### Relative Agreement (Deffuant et al. 2000, 2002; Meadows & Cliff 2012)
+
+Random pairs meet, N meetings a period. In Deffuant, Neau, Amblard and Weisbuch's pairwise bounded
+confidence (2000), two agents whose opinions differ by less than d each move a fraction μ of the way
+toward the other. In Deffuant, Amblard, Weisbuch and Faure's **relative agreement** (2002) each agent
+also has an uncertainty — a segment around its opinion — and a partner moves it by μ times the overlap
+of their segments beyond half the influencer's width, divided by that width: confident agents sway
+uncertain ones, and uncertainties move too. A few **extremists** — the most extreme opinions, very
+confident — can then leave the majority in the center, split it between the two extremes, or pull it
+all to one extreme. The paper maps which happens with an indicator y (the squared shares of moderates
+that end up extremists at each end, summed: 0 central, 0.5 both extremes, 1 a single extreme) over
+the moderates' uncertainty U and the share of extremists pe (Fig. 9). Opinions run from −1 to 1;
+Deffuant 2000's d on [0, 1] is U = 2d here.
+
+Meadows and Cliff (2012) reimplemented the model twice and could not reproduce Fig. 9; the authors
+replied (2013) that Meadows and Cliff measured y before the model converged, and counted too few
+moderates as extremists — neither detail is in the 2002 paper. Both readings are presets, and the
+paper as stated is the default. Measured (planning and the survey):
+
+- **Both fixes are needed.** At Fig. 9's corner (pe 0.05, U 1.4, 200 agents) Meadows and Cliff's
+  reading gives y 0.00; their horizon fixed, 0.40; their cutoff fixed, 0.27; both fixed, 0.98. The
+  majority has drifted most of the way to one extreme by their stop and settles between 0.7 and 0.8.
+  The paper as stated — the extremists drawn, run until nothing moves — agrees with the reply.
+- **The single extreme is a finite-size effect when the extremists are balanced.** At pe 0.1, U 1.6
+  it comes in 79 % of runs with 100 agents, 16 % with 1000 and none with 4000; Fig. 9's single-extreme
+  zone at its stated 1000 agents covers 14 of the 35 cells it shows at pe ≤ 0.075, U ≥ 1.4, and all 35
+  with 200 agents. With a lean (δ 0.1) it holds and strengthens with N. Meadows and Cliff were right
+  that it shrinks with N; the reply's 'any large number of agents' holds only at the smallest pe.
+- **Figs. 5 and 7 do not reproduce at their stated parameters.** Fig. 5's 'only … 4%' of moderates
+  becoming extremists is 48 % (22 % with extremists at ±1). Fig. 7's single extreme — and Fig. 8's
+  central convergence 'for the same parameters' — never appear at the stated μ = 0.5: both extremes in
+  39 of 40 runs. At Fig. 9's μ = 0.2 the pair appears (single 24, central 16), as §4.8's own μ result
+  predicts.
+- **Eq. 11 as printed reproduces none of §6.** The bounded-confidence window 'If |x − x′| < u′', u′
+  being the influencer's uncertainty, lets uncertain moderates pull the confident extremists in: y is
+  0.00 everywhere. With the listener's own uncertainty (**BC window** switch) §6's claims hold: a single
+  extreme only around U = 1, a central band at U 0.8 with averaged uncertainties, none with the variance
+  rule.
+- **ue matters, and the cutoff misreads it.** §4.8 finds no influence of the extremists' uncertainty;
+  at N 1000 single extremes rise from 7 to 20 of 20 runs as ue goes from 0.05 to 0.2 — and at 0.2 the
+  extreme cluster settles at ±0.75, inside the reply's 'innermost extremist less 0.1', so y reads 0.
+- **Networks.** On a Moore lattice there is never a single extreme (Amblard and Deffuant, 2004). On
+  small-world rings it needs a critical number of neighbors that falls as rewiring rises — as they say,
+  but at k 32 to 256 (most runs single from k 32 at p 0.8, 64 at p 1, 256 at p 0.2) rather than
+  'around 8' — and whether sparse rings end in both extremes or the center
+  depends, again, on the unstated cutoff. Weisbuch's scale-free networks (2004) reproduce: no steps in
+  the dispersion, close to the square lattice, closer to well mixed with twice the links; hubs end in
+  the big cluster; 16 % of agents never move. Deffuant 2000's lattice picture appears only when run to
+  stability, hundreds of periods after the caption's '100 000 iterations'.
+
+Switches for what the papers leave open: **Placement** (the most extreme draws; set to ±1; Meadows and
+Cliff's band), **New-extremist margin** (the reply's 0.1; 0 for Meadows and Cliff), **A meeting
+updates** (both from their old values; one after the other; only the first, as Weisbuch), **Pairs** on
+a network (a random link; an agent then a neighbor), **BC window**, and the stop (**Stop when stable**,
+**Stop at period**). Networks: anyone, a lattice (four or eight neighbors), a small world grown from a
+ring or the lattice, a scale-free network.
+
+The view has three panels: **opinion × time** (lines are agents, +1 at the top; the history halves
+when full, so a run of any length stays in view), **start against now** (each agent a dot; the diagonal
+marks those that never moved) and, on a lattice, the **torus** colored by opinion. Color modes:
+**Uncertainty** (confident red to uncertain green, the papers' coloring), **Role** (the initial
+extremists by side), **Start**. Inspect a column, dot or site for the agents there. Charts:
+Convergence (y, p₊, p₋); Clusters; Dispersion; Opinion and uncertainty; Change. Presets:
+`dnaw-consensus`, `dnaw-clusters`, `dnaw-lattice`, `dnaw-lattice-clusters`, `ra-uniform`, `ra-central`,
+`ra-both`, `ra-single`, `ra-literal`, `ra-meadows-cliff`, `ra-deffuant-2013`, `ra-bc-extremists`,
+`ra-bc-printed`, `ad-moore`, `ad-small-world`, `w-scale-free`. **Compare** entry: "Meadows and Cliff vs
+Deffuant et al.’s reply — Relative Agreement (Compare)". Built-in sweeps: `ra-clusters`, `ra-map`,
+`ra-readings`, `ra-population`, `ra-rules`, `ra-delta`, `ad-connectivity`, `w-dispersion`.
+
+Credit: Guillaume Deffuant, David Neau, Frédéric Amblard and Gérard Weisbuch, "Mixing Beliefs Among
+Interacting Agents," *Advances in Complex Systems* 3 (2000), 87–98; Guillaume Deffuant, Frédéric
+Amblard, Gérard Weisbuch and Thierry Faure, "How Can Extremism Prevail? A Study Based on the Relative
+Agreement Interaction Model," *JASSS* 5(4) 1 (2002); Frédéric Amblard and Guillaume Deffuant, "The
+Role of Network Topology on Extremism Propagation with the Relative Agreement Opinion Dynamics,"
+*Physica A* 343 (2004); Gérard Weisbuch, "Bounded Confidence and Social Networks," *European Physical
+Journal B* 38 (2004); Michael Meadows and Dave Cliff, "Reexamining the Relative Agreement Model of
+Opinion Dynamics," *JASSS* 15(4) 4 (2012); Guillaume Deffuant, Frédéric Amblard and Gérard Weisbuch,
+"The Results of Meadows and Cliff Are Wrong Because They Compute Indicator y Before Model
+Convergence," *JASSS* 16(1) 11 (2013). See `docs/superpowers/specs/2026-09-26-relative-agreement-design.md`.
+
 ## Experiments
 
 The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
```

Modify `docs/roadmap.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/docs/roadmap.md
+++ b/docs/roadmap.md
@@ -185,6 +185,17 @@ and sooner under milder meta-payoffs, lower mutation or any other selection rule
 decide the result: ties kept, the metanorm holds; a ranked refill, it collapses in every run. See
 `docs/superpowers/specs/2026-09-26-norms-design.md`.
 
+## Milestone 22: Relative Agreement (done)
+
+Deffuant et al.'s relative agreement model with extremists (2002) as a model kind, with their pairwise
+bounded confidence (2000) and §6 variants, Amblard and Deffuant's lattices and small worlds (2004),
+Weisbuch's scale-free networks (2004), and Meadows and Cliff's (2012) and the authors' (2013) readings
+of what the paper left unstated as switches. Fig. 9's layout reproduces as the paper states the model;
+Meadows and Cliff's failure reproduces under their reading, and both of the reply's fixes are needed.
+Balanced extremists' single extreme is a finite-size effect; Figs. 5 and 7 do not reproduce at their
+stated parameters; eq. 11 as printed reproduces none of §6; the unstated cutoff for counting extremists
+decides the network results. See `docs/superpowers/specs/2026-09-26-relative-agreement-design.md`.
+
 ## Experiments and science
 
 - **Parameter sweeps / batch runs**: done (Milestone 5).
@@ -201,6 +212,7 @@ decide the result: ties kept, the metanorm holds; a ranked refill, it collapses
 - **Cohen, Riolo & Axelrod's social structure**: done (Milestone 18).
 - **Epstein's demographic Prisoner's Dilemma** (and Radax & Rengs' replication): done (Milestone 19).
 - **Axelrod's norms and metanorms** (and Galán & Izquierdo's re-implementation): done (Milestone 20).
+- **Deffuant et al.'s relative agreement and extremism** (and Meadows & Cliff's replication): done (Milestone 22).
 - **Credit hierarchy view**: done (Milestone 6).
 
 ## Model extensions
```

Modify `docs/papers.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/docs/papers.md
+++ b/docs/papers.md
@@ -28,6 +28,7 @@ read online or from another copy; add it when found. Scanned PDFs (no text layer
 | 18 | `structure` | `social-structure/cohen-riolo-axelrod-2001-role-of-social-structure.pdf` | reproduces closely; the unstated threshold is 2.3; only the Appendix's noise rule keeps FRNE above 2DK |
 | 19 | `dpd` | `demographic-pd/epstein-1998-zones-of-cooperation-in-demographic-pd.pdf` (the working paper), `demographic-pd/epstein-2006-generative-social-science.pdf` (ch. 9: the published rule, Tables 9.1 and 9.3), `demographic-pd/radax-rengs-2009-mpra-replication-of-the-demographic-prisoners-dilemma.pdf` (published as JASSS 13(4) 1, 2010) | Tables 1 and 2 do not reproduce under the published rule; only unstated readings (founders with no wealth, the working paper's rule) come close; the metabolism "equivalence" holds only per game |
 | 20 | `norms` | `norms/axelrod-1986-apsr-evolutionary-approach-to-norms.pdf`, `norms/galan-izquierdo-2005-jasss-appearances-can-be-deceiving.html` | Axelrod's 100-generation results reproduce; metanorms usually collapse by 10⁶ (G&I); the unstated tie and refill rules decide whether the metanorm lasts |
+| 22 | `agreement` | `bounded-confidence/deffuant-neau-amblard-weisbuch-2000-acs-mixing-beliefs.pdf`, `bounded-confidence/deffuant-amblard-weisbuch-faure-2002-jasss-how-can-extremism-prevail.html`, `bounded-confidence/amblard-deffuant-2004-network-topology-and-extremism.pdf`, `bounded-confidence/weisbuch-2003-bounded-confidence-and-social-networks.pdf`; the replication and reply `bounded-confidence/meadows-cliff-2012-jasss-reexamining-the-relative-agreement-model.html`, `bounded-confidence/deffuant-amblard-weisbuch-2013-jasss-meadows-and-cliff-are-wrong.html` | both of the reply's fixes are needed, and the paper as stated agrees with it; balanced extremists' single extreme vanishes as N grows; Figs. 5 and 7 and eq. 11 as printed do not reproduce; the unstated cutoff decides the network results |
 
 ## Queue
 
@@ -36,17 +37,16 @@ worth doing; "size" is a guess at the milestone's scale.
 
 | # | Model | Original | Critique or follow-up | Size | Shape |
 |---|---|---|---|---|---|
-| 1 | Relative agreement and extremism | `bounded-confidence/deffuant-neau-amblard-weisbuch-2000-acs-mixing-beliefs.pdf` (the pairwise original), `bounded-confidence/deffuant-amblard-weisbuch-faure-2002-jasss-how-can-extremism-prevail.html` | `bounded-confidence/amblard-deffuant-2004-network-topology-and-extremism.pdf`, `bounded-confidence/weisbuch-2003-bounded-confidence-and-social-networks.pdf` | medium | extends `opinions` (pairwise updating, uncertainty, networks) |
-| 2 | Image scoring | `image-scoring/nowak-sigmund-1998-iiasa-indirect-reciprocity-by-image-scoring.pdf` | `image-scoring/leimar-hammerstein-2001-prsb-cooperation-through-indirect-reciprocity.pdf` | medium | new kind |
-| 3 | El Farol and the minority game | `el-farol/arthur-1994-aer-inductive-reasoning-and-bounded-rationality.pdf` | `el-farol/challet-zhang-1997-emergence-of-cooperation-minority-game.pdf` | small | new kind (predictor pools, the memory transition) |
-| 4 | Ants and recruitment (herding) | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf` | — | small | new kind (N agents, two sources, random recruitment and switching; the bimodal regime) |
-| 5 | Threshold models | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*) | — | small | new kind, or a Sugarscape rule |
-| 6 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
-| 7 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
-| 8 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
-| 9 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
-| 10 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
-| 11 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
+| 1 | Image scoring | `image-scoring/nowak-sigmund-1998-iiasa-indirect-reciprocity-by-image-scoring.pdf` | `image-scoring/leimar-hammerstein-2001-prsb-cooperation-through-indirect-reciprocity.pdf` | medium | new kind |
+| 2 | El Farol and the minority game | `el-farol/arthur-1994-aer-inductive-reasoning-and-bounded-rationality.pdf` | `el-farol/challet-zhang-1997-emergence-of-cooperation-minority-game.pdf` | small | new kind (predictor pools, the memory transition) |
+| 3 | Ants and recruitment (herding) | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf` | — | small | new kind (N agents, two sources, random recruitment and switching; the bimodal regime) |
+| 4 | Threshold models | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*) | — | small | new kind, or a Sugarscape rule |
+| 5 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
+| 6 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
+| 7 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
+| 8 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
+| 9 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
+| 10 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
 
 ## Wanted
 
```

Modify `docs/superpowers/specs/2026-09-26-relative-agreement-design.md` (a unified diff against the branch's base: save it and `git apply` it, or make the edits by hand):

```diff
--- a/docs/superpowers/specs/2026-09-26-relative-agreement-design.md
+++ b/docs/superpowers/specs/2026-09-26-relative-agreement-design.md
@@ -122,9 +122,9 @@ A period is **stable** when no opinion or uncertainty moved more than 10⁻⁶ i
 
 - **Opinion × time** (left): 241 × 201 cells, +1 at the top. The history keeps at most 241 columns: when full, every other kept period is dropped and later periods are kept at the doubled interval, so a run of any length stays in view. The history is world state: keyframes and step-back restore it.
 - **Start vs now** (right, 8-cell gap): 201 × 201, each agent a dot at (start, current opinion); the diagonal marks agents that never moved (DNAW Fig. 3, W Figs. 4–5).
-- **Torus** (further right, 8-cell gap) under `lattice` and the grid substrate: one cell per site, colored by opinion.
+- **Torus** (further right, 8-cell gap) under `lattice` and the grid substrate: one cell per site, colored by current opinion in every mode.
 - **Color modes:** **Uncertainty** (default; confident to uncertain, as DAWF's figures), **Role** (initial extremists by side; moderates by current opinion), **Start** (by starting opinion).
-- **Inspect:** a line or dot — the agent's id, role, start, opinion, uncertainty, degree, meetings and moves; a torus site the same. `locate` returns the agent's dot; Follow available.
+- **Inspect:** a column, dot or site — the agents there, each with its id, role, start, opinion and uncertainty (in that period), degree, meetings and moves. `agent` stays null and `locate` returns nothing, as in `opinions`: the page would otherwise re-read a tracked agent at its dot, where neighbors crowd it (amended in planning).
 - **Charts:** Convergence (`y`, `p_plus`, `p_minus`); Clusters (`clusters`, `major`, `isolated`); Dispersion (`dispersion`, `unmoved`); Opinion and uncertainty (`mean_opinion`, `mean_uncertainty`); Change (`max_change`). Time axis: Periods.
 
 ## Presets
@@ -184,3 +184,21 @@ The presets menu gains a **Relative Agreement** group and the Compare entry; the
 ## Docs
 
 README: a Relative Agreement section (the rules, the stated choices and switches, the readings and what each reproduces, presets, sweeps, and the findings: both fixes are needed; single extreme is a finite-size effect at δ 0; eq. 11's printed window reproduces none of §6; the cutoff decides AD's low-k regime). `docs/papers.md`: the milestone's row, with M&C and DAW as the critique and reply; roadmap: Milestone 22 done.
+
+## Amendments (implementation planning)
+
+The model was implemented in full while planning (`docs/superpowers/plans/2026-09-26-relative-agreement.md`) and measured with it; these change or extend the sections above.
+
+- **Inspect reads cells** (see Views): `agent` is always null and `locate` returns nothing.
+- **Weisbuch's lattice line** in `w-dispersion` uses his pairing and one-way updating, like the scale-free lines; the well-mixed line uses DNAW's symmetric meetings.
+- **The web golden list** holds the nine presets still running at period 200 on seed 1 (`dnaw-lattice`, `dnaw-lattice-clusters`, `ra-central`, `ra-literal`, `ra-deffuant-2013`, `ra-bc-extremists`, `ad-moore`, `ad-small-world`, `w-scale-free`); the rest settle sooner. An engine test runs `ra-single` to its stop at 71 and `ra-meadows-cliff` to 200.
+- **Measured with the implementation** (20–50 seeds; the survey's numbers):
+  - Fig. 5 (pe 0.2, U 0.4, μ 0.5): 48 % of moderates become extremists (22 % with extremists at ±1) against the caption's 4 %.
+  - Figs. 7–8 (pe 0.1, U 1.4, μ 0.5): both extremes in 39 of 40 runs, under every placement and update order; at Fig. 9's μ 0.2, single 24 and central 16 of 40. The figures' μ looks misstated (§4.8: larger μ widens both extremes).
+  - Fig. 9 at N 1000: the single-extreme zone (y ≥ 0.75, pe ≤ 0.075, U ≥ 1.4) holds in 14 of 35 cells; at N 200, 35 of 35. With δ 0.1 it holds in all 55 cells at U ≥ 1.6, pe ≤ 0.15.
+  - §4.8's ue: at N 1000, pe 0.05, U 1.4, the population drifts to one extreme in far fewer runs at ue 0.05 than at 0.2 (median |mean opinion| 0.03 against 0.75); and at ue 0.2 the extreme cluster settles at ±0.75, inside the reply's cutoff (y 0.00; 1.00 counted 0.3 inside the innermost extremist).
+  - The reply's "any large number of agents": single extreme in 37 of 50 runs at N 2000 for pe 0.05, 1 of 50 for pe 0.1 (δ 0). With δ 0.1 y rises with N (0.84 at N 100 to 0.99 at N 2000).
+  - Fig. 4 at N 1000: relative agreement 2.00, 2.70, 3.40, 5.36, 10.54 clusters at w/2u 2, 2.5, 3.33, 5, 10 (within a fifth of w/2u; 2.10, 3.04, 3.70, 5.90, 11.28 at N 200); bounded confidence 2.00, 2.98 at 2.5, 3.33, then 3.44, 4.32, 8.92 at 4, 5, 10 — the integer part only up to about 3.
+  - AD (N 1000, 20 runs): most runs single from k 32 at p 0.8, 64 at p 1, 256 at p 0.2; none single on the Moore torus (largest y 0.51 over 160 runs); at k 2–4 central with the 0.9 cutoff, both extremes in 40 of 40 with 0.7; the grid substrate's y rises from 0.01 at k 8 to 1.00 at k 120.
+  - Weisbuch (N 900, 50 runs): well-mixed dispersion 0.36, 0.50, 0.51 at d 0.15, 0.2, 0.25; scale-free 0.17, 0.34, 0.60, 0.84 and the lattice 0.02, 0.30, 0.70, 0.86 at d 0.15–0.3; 8 links 0.22 from well mixed against 0.57 for 4; 62 % of the ten best-connected agents in the largest cluster (which holds 49 %); 15.9 % never move (0.1 % well mixed).
+  - DNAW's lattice at d 0.3: at period 119 (the caption's 100 000 iterations) the largest cluster holds 39 % on average; at stability 92 %, with a median of 28 isolated agents.
```

- [ ] **Step 2: Verify everything**

Run:
```bash
cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings && cargo test --workspace --release
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && cargo build --release && ./target/release/survey --only agreement)
```
Expected: clean; 754 Rust tests, 43 WASM, 626 web; the survey's verdicts as in Task 4.

- [ ] **Step 3 (controller): the full browser pass** — Task 3's Step 6 list again, on the final build.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-09-26-relative-agreement-design.md
git commit -m "Document Relative Agreement and the readings that decide it; mark milestone 22 done

Claude-Session: https://claude.ai/code/session_011pY2cJwAezJH4xb32Aw8xr"
```

---

## Self-review (planning)

- **Spec coverage:** the model and rules (Task 1: `influence`, `meet`), extremists and placements (`AgreementWorld::new`, `extremist_counts`), networks (`network.rs`), stopping and stability (`step`, `is_finished`), statistics (`stats.rs`, `record`), views and color modes (`render`, `view.rs`), Inspect (`inspect`), presets (`presets.rs`), golden entries (Task 1), sweeps and the CLI (Task 2), WASM agreement (Task 2), the page (Task 3), the survey (Task 4), docs and spec amendments (Task 5). Two spec points changed in planning (Decisions 1 and 2) and are amended in Task 5.
- **Placeholder scan:** none; every file is given whole or as a diff taken from the verified scratch copy.
- **Type consistency:** the TypeScript `AgreementConfig`, `AgreementStats` and `AgreementInspection` match the Rust serializations field for field (checked by the web determinism and engine tests against the real WASM build); the survey uses only the names `agreement/mod.rs` exports.
- **Review Focus:** each of the five lines has its pinning tests in Task 1 (and Tasks 2–3 for stopping); none is left to the browser pass.
