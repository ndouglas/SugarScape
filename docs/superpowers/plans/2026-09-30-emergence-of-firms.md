# The Emergence of Firms Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Axtell's firm-formation model (Brookings CSED WP 3, 1999), with his 2013 parameterization, as one model kind, `firms` ("The Emergence of Firms"). Agents with Cobb–Douglas preferences for income and leisure choose their effort in teams with increasing returns, moving between their own firm, a start-up and their friends' firms. The §2 analytics (Nash efforts, optimal and maximum stable sizes, Table 1) are tested functions. Every §4 variation (Tables 3–13) is a named switch, along with two readings the paper leaves open that turn out to decide its tables. The milestone adds 18 titled presets, 13 measured sweeps and a 25-claim survey, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/firms/`:

- `config.rs`: the parameters, twelve reading enums, validation and the schema.
- `effort.rs`: production, preferences and pay shares; the best effort (closed forms, a safeguarded Newton solve, and a kink-aware root search on the analytic slope); the §2 analytics.
- `fit.rs`: the records and the fits (the paper's OLS µ, the exact discrete maximum-likelihood µ, the output exponent, productivity, Laplace against Gaussian growth, γ, lifetimes).
- `stats.rs`: the 15 series and the snapshot.
- `view.rs`: the frame (firms as rows, the size distribution beside them).
- `world.rs`: `FirmsWorld`, covering activation, the options, the period's production and pay, the records, rendering and Inspect.
- `presets.rs` and `mod.rs`.

It is wired into `ModelConfig`/`ModelWorld` like the other models, with titles in `titles.rs`. The page adds the model's types, color modes, charts, Inspect rows, a Compare entry and an Experiments default.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md` (binding, as amended in Task 5). Sources:

- `papers/firms/axtell-1999-emergence-of-firms.pdf` (A99; printed page + 7 = PDF page);
- `papers/firms/axtell-2013-lem-endogenous-dynamics-of-firms-and-labor.pdf` (A13).

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited. `MODEL_GOLDEN` gains eighteen `firms-*` entries.
- **One engine path; deterministic; portable:** native and WASM fingerprints must be identical. Planning verified this with `wasm-pack test` on ten presets.
  - Draws use `u32` ranges and `f64` samples only.
  - The effort optimum uses `+ − × ÷`, `sqrt` and the project's portable `ln`/`exp`; non-integer powers go through `effort::powf`, built on them.
  - Fits sum in a fixed order.
- **Literal defaults, named departures, honest descriptions and titles:**
  - The text is the default: others' effort is inferred from last period's output, activation is random with replacement, the effort is the exact optimum, and sticky effort and groping apply everywhere.
  - Every reading the paper leaves open is a named switch, stated in the README.
  - Every description and title says what was measured, with the paper's numbers beside ours.
- **Copy (verbatim):**
  - Model label: **The Emergence of Firms**.
  - Preset ids: `firms-base`, `firms-live`, `firms-uniform`, `firms-beta-17`, `firms-beta-21`, `firms-b-15`, `firms-b-random`, `firms-theta-075`, `firms-friends-10`, `firms-random-firms-10`, `firms-loyal-10`, `firms-sticky`, `firms-groping`, `firms-seniority-5`, `firms-base-pay-80`, `firms-hiring-100`, `firms-random-choices`, `firms-2013`.
  - Compare entry: **Last period's effort vs live effort — The Emergence of Firms (Compare)** (id `firms-last-vs-live`).
  - Color modes: **Founder**, **θ**, **Effort**, **Income**.
  - Schema groups: **Agents**, **Production**, **Preferences**, **Network**, **Decisions**, **Pay**, **Measurement**.
  - Charts: **Firms**, **Sizes**, **Effort and pay**, **Output**, **Scaling**; time axis **Periods**.
  - Sweeps: `firms-beta`, `firms-b`, `firms-preferences`, `firms-friends`, `firms-random-firms`, `firms-loyalty`, `firms-sticky`, `firms-groping`, `firms-seniority`, `firms-base-pay`, `firms-hiring`, `firms-readings`, `firms-population`.
  - Series: `firms, births, deaths, mean_size, largest, singletons, effort, output, income, utility, largest_output_share, mu, mu_mle, lifetime, period`.
  - Notice: `This run has reached its last period (<tick>) — Reset to run it again`.
  - CLI: `(its last period)`.
- **Commits:** every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4`. Stage only the task's files; never stage `.claude/`, `papers/` or `web/node_modules`.
- **Rust:** `cargo fmt --all && cargo +stable clippy --all-targets -- -D warnings` (CI runs the latest stable clippy). In `survey/`, format only `survey/src/claims/firms.rs`, and do not commit `survey/out/results-*.json`.
- **Web:** `(cd web && npm run build && npm test)`. In a fresh worktree, run `npm ci` and `npm run wasm` first.
- **Browser checks are the controller's:** Task 3's Step 6, and the full pass in Task 5.

## Review Focus

1. **The best effort at the edges.** Covers others' effort of zero, a window clamping the optimum against 0 or 1, constant returns (b = 0 or β = 1), and base pay's kink, where the bonus switches on inside the range. The optimum must be the true maximum on [lo, hi], not a local one. Pinned in Task 1 by `a_window_clamps_the_optimum`, `constant_returns_follow_equation_6`, `general_beta_ces_and_base_pay_find_the_maximum` and `the_solvers_match_a_fine_grid_on_random_cases` (600 random cases against a 5 000-step grid).
2. **The books.** Every period, pay sums to output under every pay rule: base pay is paid even when output falls short. Every agent is in exactly one firm, empty firms die, and lifetimes count from founding to the last departure. Pinned by `firms_form_grow_and_the_books_balance`, `seniority_shares_sum_to_output_and_fall_with_rank`, `base_pay_is_paid_even_when_output_falls_short` and `lifetimes_are_counted_after_the_burn_in`.
3. **The options.** The hiring standard filters joins against the longest-serving member's θ. Loyalty holds an agent until its count runs out. Sticky effort and groping bound effort by `adjust_scope`. Seniority ranks by `seniority_order`. Pinned by `a_hiring_standard_admits_only_those_near_the_senior_member`, `loyal_agents_stay_until_their_count_runs_out`, `sticky_effort_moves_at_most_half_its_window`, `sticky_effort_in_the_own_firm_only_leaves_moves_free` and `junior_first_seniority_pays_the_newest_most`.
4. **Live edits against reset.** Changing the population, preferences or network waits for Reset. Changing pay recomputes base pay at once. Pinned by `live_edits_apply_and_the_population_waits_for_reset` and `the_population_and_network_change_only_on_reset`.
5. **Portability.** Native and WASM fingerprints agree, including a `powf` path (β drawn per firm). Pinned in Task 2 by `firms_sims_match_the_native_golden_entries`.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed:

- `cargo test --release --workspace` (1 405);
- `cargo +stable clippy --all-targets -- -D warnings`;
- `wasm-pack test --node crates/sugarscape-wasm` (64);
- `npm run build && npm test` (802, 50 files);
- the survey (25 claims).

The decisions:

1. **Two readings found by measuring.**
   - `adjust_scope`: whether sticky effort and groping bound effort in a firm joined or founded. Everywhere (the literal default) sends the whole population into one firm.
   - `seniority_order`: the text's founder-first ranking keeps every firm at 4 or fewer, and newest-first gives near-giant firms. Neither gives Table 11's 1.11.
2. **Ranges.** A maximum at or below its value means fixed, so each field can be changed on its own.
3. **The solvers.**
   - Closed form (5) for β = 2 and (6) for constant returns.
   - A safeguarded Newton solve for other β.
   - For base pay and CES, a root search on the analytic slope, split at the kink, with the bonus regime decided per piece. This replaced a golden-section search that was 18× slower and missed optima at the kink.
4. **Two estimators.** The paper's OLS µ, and the exact discrete maximum-likelihood µ (Clauset, Shalizi and Newman eq. 3.5), updated every 10 periods.
5. **The view.** The largest firms go first, with 3-pixel cells. In founding order, the large firms fell off the frame.
6. **A fifth chart.** Output gets its own chart.
7. **Planning's findings** (the survey reproduces them):
   - Table 1 and the §2 analytics reproduce exactly.
   - The base case gives µ ≈ 2.5 by OLS (1.44 by maximum likelihood), against 1.28, with mean lifetimes of 3.9 against 23.4. The paper's own counts cannot give 23.4.
   - Growth rates are Laplace, and γ is 0.171 against 0.174.
   - The §4 directions hold for β, b, random firms, loyalty, base pay and hiring. The friends table reverses. Seniority fails under both orders.
   - Random effort gives one giant firm.
   - Myopic agents join even under constant returns (fn 19).
   - A13's parameterization gives Zipf's law (µ ≈ 1.00) with lifetimes near 77, but also a giant firm of half the population.

---

### Task 1: The firms model in the core

**Files:**
- Create: `crates/sugarscape-core/src/firms/{config,effort,fit,stats,view,world,presets,mod}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::Rgb`, `crate::rng::{self, SimRng}`, `crate::schema::{Apply, Param}`, `crate::presets::ModelPreset`, `crate::config::FieldError`, and the portable `ln`/`exp` helpers.
- Produces:
  - `firms::{FirmsConfig, Preferences, CesSign, Network, Activation, OthersEffort, EffortSearch, Pay, BasePay, RandomBehavior, Initial, AdjustScope, SeniorityOrder, schema, presets}`.
  - `firms::effort::{powf, Tech, Prefs, Share, Choice, Search, closed_form, best, nash, jacobian_k, eigenvalue, max_stable_size, optimal_size}`.
  - `firms::fit::{Records, ols, mu_ols, mu_mle, output_exponent, productivity, growth_fit, gamma, lifetimes}`.
  - `firms::{SERIES, FirmsSnapshot, FirmsWorld, FirmsInspection, FirmView, AgentView}`.
  - `FirmsWorld::{new, step, run, records, is_finished, inspect}` plus `pub tick` and `pub stats`.
  - `ModelKind::Firms` (`"firms"`), `ModelConfig::Firms`, `ModelWorld::Firms`.
  - Eighteen titles.

- [ ] **Step 1: Write the module**

The tests are in each file:

- **config:** Axtell's base case as the defaults, validation, reset fields and the schema.
- **effort:** Table 1 to three places, the optimal and stable sizes by θ, eq. (6), the closed form as the line search's limit, the windowed optimum, CES's limits, `powf` against the library, and the solvers against a fine grid on 600 random cases.
- **fit:** both estimators on a known exponent, the OLS's drops, Laplace against Gaussian, γ, lifetimes and productivity.
- **view:** the frame.
- **world:** sixteen tests, listed in the Review Focus and below.

Create `crates/sugarscape-core/src/firms/config.rs` with exactly this content:

````rust
//! The Emergence of Firms' parameters: Axtell's (1999) base case, every
//! variation of his §4, the 2013 paper's parameterization, and every detail
//! the paper leaves open as a named switch.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// How preferences for income against leisure are distributed (A99 §4.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preferences {
    /// Cobb–Douglas, θ ~ U[0, 1] (the base case).
    Uniform,
    /// θ ~ U[0.25, 0.75].
    Middle,
    /// θ triangular on [0, 1] with mode 0.5.
    Triangular,
    /// θ triangular on [0, 1] with mode 0.75.
    TriangularHigh,
    /// θ normal (mean 0.5, variance ½) truncated to [0, 1]; the mean is our
    /// reading.
    Normal,
    /// θ ~ Beta(1, 2) (mean ⅓; the parameters' order is our reading).
    Beta,
    /// Every θ = `theta`.
    Fixed,
    /// CES: δ ~ U[0, 1], ρ ~ U[`rho`, `rho_max`].
    Ces,
}

/// The CES exponent's sign convention (A99 p. 73's printed formula against
/// its text's limits).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CesSign {
    /// (δ y^−ρ + (1−δ) l^−ρ)^(−1/ρ): linear at ρ = −1, Cobb–Douglas as
    /// ρ → 0, Leontief as ρ grows — the limits the text states.
    Text,
    /// (δ y^ρ + (1−δ) l^ρ)^(1/ρ), as printed.
    Printed,
}

/// Where an agent looks for other firms (A99 §4.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    /// ν fixed random agents, assigned at the start: their current firms.
    Friends,
    /// ν firms drawn uniformly at each activation, not the agent's own.
    RandomFirms,
}

/// Who is activated in a period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Activation {
    /// Drawn uniformly with replacement (A99: "random activation").
    Random,
    /// Each agent at most once, in random order.
    Uniform,
}

/// What an agent takes as the other members' effort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OthersEffort {
    /// Inferred from last period's output (A99 pp. 9, 26; A13 p. 8).
    LastPeriod,
    /// Their current efforts.
    Live,
}

/// How an agent finds its best effort.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffortSearch {
    /// The optimum (the limit of A99's line search).
    Exact,
    /// The best of `grid_steps` + 1 evenly spaced efforts.
    Grid,
}

/// How a firm divides its output (A99 §4.7–4.8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pay {
    /// Equal shares (the base case).
    Equal,
    /// Shares ∝ `seniority_base`^−rank, rank 1 the longest-serving member.
    Seniority,
    /// Base pay Φ plus an equal share of what output exceeds the base pay.
    Base,
}

/// Where sticky effort's window and groping apply (A99 §4.6 does not say).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustScope {
    /// To effort in any firm: a start-up or a firm joined too.
    Everywhere,
    /// Only to effort in the agent's own firm; in a new one it chooses freely.
    OwnFirm,
}

/// Which way seniority shares run (A99 §4.7: "i = 1 referring to the firm
/// founder").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeniorityOrder {
    /// The longest-serving member has the largest share (the text).
    SeniorFirst,
    /// The newest member has the largest share.
    JuniorFirst,
}

/// Whose singleton income sets base pay.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasePay {
    /// Each agent's own.
    Own,
    /// The median agent's (θ = ½).
    Median,
    /// The mean over the population.
    Mean,
}

/// A99 §4.1's random behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RandomBehavior {
    None,
    /// Stay, move or start up at random (a random option), then the best
    /// effort.
    Choices,
    /// The best option at a random effort.
    Effort,
}

/// How agents start.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Initial {
    /// Every agent alone (the base case).
    Alone,
    /// Groups of geometrically distributed size, mean 4 (our reading).
    RandomGroups,
    /// One firm of everyone.
    OneFirm,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FirmsConfig {
    pub agents: u32,
    /// Output a·E + b·E^β; each drawn per firm at founding between the value
    /// and its maximum when the maximum is above it (otherwise fixed).
    pub a: f64,
    pub a_max: f64,
    pub b: f64,
    pub b_max: f64,
    pub beta: f64,
    pub beta_max: f64,
    pub preferences: Preferences,
    pub theta: f64,
    pub rho: f64,
    pub rho_max: f64,
    pub ces_sign: CesSign,
    pub network: Network,
    /// ν, or drawn per agent between the two.
    pub neighbors: u32,
    pub neighbors_max: u32,
    pub activation: Activation,
    /// Activations a period ÷ agents.
    pub activation_rate: f64,
    pub others_effort: OthersEffort,
    pub effort_search: EffortSearch,
    pub grid_steps: u32,
    /// Sticky effort: a new effort within ±window/2 of the current one.
    pub effort_window: f64,
    pub groping: bool,
    pub adjust_scope: AdjustScope,
    /// λ: moves only after wanting to more than λ times.
    pub loyalty: u32,
    pub loyalty_max: u32,
    pub pay: Pay,
    pub seniority_base: f64,
    pub seniority_order: SeniorityOrder,
    pub base_pay: BasePay,
    pub base_share: f64,
    /// φ: a firm admits only θ ≥ φ·θ of its longest-serving member.
    pub hiring: f64,
    pub hiring_max: f64,
    pub random_behavior: RandomBehavior,
    pub initial: Initial,
    /// The size, growth and lifetime records start after `burn_in` periods;
    /// sizes are sampled every `sample_every` periods.
    pub burn_in: u32,
    pub sample_every: u32,
    pub stop_at: u32,
}

impl Default for FirmsConfig {
    /// Axtell's base case (A99 Table 2).
    fn default() -> Self {
        FirmsConfig {
            agents: 1000,
            a: 1.0,
            a_max: 0.0,
            b: 1.0,
            b_max: 0.0,
            beta: 2.0,
            beta_max: 0.0,
            preferences: Preferences::Uniform,
            theta: 0.75,
            rho: -1.0,
            rho_max: 0.0,
            ces_sign: CesSign::Text,
            network: Network::Friends,
            neighbors: 2,
            neighbors_max: 0,
            activation: Activation::Random,
            activation_rate: 1.0,
            others_effort: OthersEffort::LastPeriod,
            effort_search: EffortSearch::Exact,
            grid_steps: 100,
            effort_window: 1.0,
            groping: false,
            adjust_scope: AdjustScope::Everywhere,
            loyalty: 0,
            loyalty_max: 0,
            pay: Pay::Equal,
            seniority_base: 2.0,
            seniority_order: SeniorityOrder::SeniorFirst,
            base_pay: BasePay::Own,
            base_share: 0.5,
            hiring: 0.0,
            hiring_max: 0.0,
            random_behavior: RandomBehavior::None,
            initial: Initial::Alone,
            burn_in: 500,
            sample_every: 1,
            stop_at: 5000,
        }
    }
}

impl FirmsConfig {
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |v: f64| (0.0..=1.0).contains(&v);
        check(
            (2..=200_000).contains(&self.agents),
            "agents",
            "must be between 2 and 200000",
        );
        check(
            (0.0..=10.0).contains(&self.a) && (0.0..=10.0).contains(&self.a_max),
            "a",
            "a and its maximum must be between 0 and 10",
        );
        check(
            (0.0..=10.0).contains(&self.b) && (0.0..=10.0).contains(&self.b_max),
            "b",
            "b and its maximum must be between 0 and 10",
        );
        check(
            (1.0..=3.0).contains(&self.beta) && (0.0..=3.0).contains(&self.beta_max),
            "beta",
            "β must be between 1 and 3 (its maximum at most 3)",
        );
        check(self.a + self.b > 0.0, "b", "a and b cannot both be 0");
        check(unit(self.theta), "theta", "must be between 0 and 1");
        check(
            (-1.0..=10.0).contains(&self.rho) && (-1.0..=10.0).contains(&self.rho_max),
            "rho",
            "ρ and its maximum must be between −1 and 10",
        );
        check(
            (1..=100).contains(&self.neighbors) && self.neighbors_max <= 100,
            "neighbors",
            "ν must be between 1 and 100 (its maximum at most 100)",
        );
        check(
            self.activation_rate > 0.0 && self.activation_rate <= 10.0,
            "activation_rate",
            "must be above 0 and at most 10",
        );
        check(
            (1..=10_000).contains(&self.grid_steps),
            "grid_steps",
            "must be between 1 and 10000",
        );
        check(
            self.effort_window > 0.0 && self.effort_window <= 1.0,
            "effort_window",
            "must be above 0 and at most 1",
        );
        check(
            self.loyalty <= 1000 && self.loyalty_max <= 1000,
            "loyalty",
            "λ and its maximum must be at most 1000",
        );
        check(
            (1.0..=100.0).contains(&self.seniority_base),
            "seniority_base",
            "must be between 1 and 100",
        );
        check(
            unit(self.base_share),
            "base_share",
            "must be between 0 and 1",
        );
        check(
            unit(self.hiring) && unit(self.hiring_max),
            "hiring",
            "φ and its maximum must be between 0 and 1",
        );
        check(
            (1..=100_000).contains(&self.sample_every),
            "sample_every",
            "must be between 1 and 100000",
        );
        check(self.burn_in <= 100_000, "burn_in", "must be at most 100000");
        check(self.stop_at <= 100_000, "stop_at", "must be at most 100000");
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &FirmsConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("agents", self.agents == next.agents),
            ("preferences", self.preferences == next.preferences),
            ("theta", self.theta == next.theta),
            ("rho", (self.rho, self.rho_max) == (next.rho, next.rho_max)),
            ("ces_sign", self.ces_sign == next.ces_sign),
            ("network", self.network == next.network),
            (
                "neighbors",
                (self.neighbors, self.neighbors_max) == (next.neighbors, next.neighbors_max),
            ),
            (
                "loyalty",
                (self.loyalty, self.loyalty_max) == (next.loyalty, next.loyalty_max),
            ),
            ("initial", self.initial == next.initial),
        ] {
            if !same {
                out.push(FieldError::new(field, "changes only on reset"));
            }
        }
        out
    }

    /// Whether a firm's output parameters are drawn per firm.
    pub fn drawn(lo: f64, hi: f64) -> bool {
        hi > lo
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::integer("Agents", "agents", "Agents", (2, 200_000), Reset),
        Param::choice(
            "Agents",
            "initial",
            "Start",
            &[
                ("alone", "Everyone alone (Axtell)"),
                ("random_groups", "In random groups (mean size 4)"),
                ("one_firm", "All in one firm"),
            ],
            Reset,
        ),
        Param::number(
            "Production",
            "a",
            "Constant returns a",
            (0.0, 10.0, 0.05),
            Live,
        ),
        Param::number(
            "Production",
            "a_max",
            "a up to (0: fixed)",
            (0.0, 10.0, 0.05),
            Live,
        )
        .with_help("Above a: each firm draws its a at founding."),
        Param::number(
            "Production",
            "b",
            "Increasing returns b",
            (0.0, 10.0, 0.05),
            Live,
        ),
        Param::number(
            "Production",
            "b_max",
            "b up to (0: fixed)",
            (0.0, 10.0, 0.05),
            Live,
        )
        .with_help("Above b: each firm draws its b at founding (Axtell §4.2)."),
        Param::number("Production", "beta", "Exponent β", (1.0, 3.0, 0.05), Live),
        Param::number(
            "Production",
            "beta_max",
            "β up to (0: fixed)",
            (0.0, 3.0, 0.05),
            Live,
        )
        .with_help("Above β: each firm draws its β at founding (our reading of §4.2)."),
        Param::choice(
            "Preferences",
            "preferences",
            "Preferences",
            &[
                ("uniform", "Cobb–Douglas, θ uniform on [0, 1] (Axtell)"),
                ("middle", "θ uniform on [0.25, 0.75]"),
                ("triangular", "θ triangular, mode 0.5"),
                ("triangular_high", "θ triangular, mode 0.75"),
                ("normal", "θ normal (0.5, variance ½), truncated"),
                ("beta", "θ Beta(1, 2)"),
                ("fixed", "Every θ the same"),
                ("ces", "CES"),
            ],
            Reset,
        ),
        Param::number("Preferences", "theta", "θ", (0.0, 1.0, 0.01), Reset)
            .shown_if("preferences", "fixed"),
        Param::number("Preferences", "rho", "ρ from", (-1.0, 10.0, 0.1), Reset)
            .shown_if("preferences", "ces"),
        Param::number(
            "Preferences",
            "rho_max",
            "ρ up to (0: fixed)",
            (0.0, 10.0, 0.1),
            Reset,
        )
        .shown_if("preferences", "ces"),
        Param::choice(
            "Preferences",
            "ces_sign",
            "CES exponent",
            &[
                ("text", "As the text's limits read (−ρ)"),
                ("printed", "As printed (+ρ)"),
            ],
            Reset,
        )
        .shown_if("preferences", "ces"),
        Param::choice(
            "Network",
            "network",
            "Agents look at",
            &[
                ("friends", "Their friends' firms (Axtell)"),
                ("random_firms", "Random firms, each time"),
            ],
            Reset,
        ),
        Param::integer(
            "Network",
            "neighbors",
            "Friends or firms ν",
            (1, 100),
            Reset,
        ),
        Param::integer(
            "Network",
            "neighbors_max",
            "ν up to (0: fixed)",
            (0, 100),
            Reset,
        ),
        Param::choice(
            "Decisions",
            "activation",
            "Activation",
            &[
                ("random", "Random, with replacement (Axtell)"),
                ("uniform", "Each agent once, in random order"),
            ],
            Live,
        ),
        Param::number(
            "Decisions",
            "activation_rate",
            "Activations a period ÷ agents",
            (0.01, 10.0, 0.01),
            Live,
        )
        .with_help("Axtell 1999: 1 (a period is 1 000 activations of 1 000 agents); 2013: 0.04."),
        Param::choice(
            "Decisions",
            "others_effort",
            "Others' effort",
            &[
                ("last_period", "From last period's output (the text)"),
                ("live", "Their current efforts"),
            ],
            Live,
        ),
        Param::choice(
            "Decisions",
            "effort_search",
            "Best effort",
            &[
                ("exact", "The optimum"),
                ("grid", "The best of a grid (a coarse line search)"),
            ],
            Live,
        ),
        Param::integer("Decisions", "grid_steps", "Grid steps", (1, 10_000), Live)
            .shown_if("effort_search", "grid"),
        Param::number(
            "Decisions",
            "effort_window",
            "Effort changes by at most (window)",
            (0.01, 1.0, 0.01),
            Live,
        )
        .with_help("1: free. Axtell's sticky effort: 0.10 (±0.05)."),
        Param::bool(
            "Decisions",
            "groping",
            "Grope for effort (one random try)",
            Live,
        ),
        Param::choice(
            "Decisions",
            "adjust_scope",
            "Sticky effort and groping apply",
            &[
                ("everywhere", "In any firm, joined or founded too"),
                ("own_firm", "Only in the agent's own firm"),
            ],
            Live,
        ),
        Param::integer("Decisions", "loyalty", "Loyalty λ", (0, 1000), Reset),
        Param::integer(
            "Decisions",
            "loyalty_max",
            "λ up to (0: fixed)",
            (0, 1000),
            Reset,
        ),
        Param::choice(
            "Decisions",
            "random_behavior",
            "Random behavior (§4.1)",
            &[
                ("none", "None (Axtell)"),
                ("choices", "Random choices"),
                ("effort", "Random effort"),
            ],
            Live,
        ),
        Param::choice(
            "Pay",
            "pay",
            "Output is shared",
            &[
                ("equal", "Equally (Axtell)"),
                ("seniority", "By seniority"),
                ("base", "Base pay plus a bonus"),
            ],
            Live,
        ),
        Param::number(
            "Pay",
            "seniority_base",
            "Seniority p (shares ∝ p^−rank)",
            (1.0, 100.0, 0.5),
            Live,
        )
        .shown_if("pay", "seniority"),
        Param::choice(
            "Pay",
            "seniority_order",
            "The largest share goes to",
            &[
                ("senior_first", "The longest-serving (the text)"),
                ("junior_first", "The newest"),
            ],
            Live,
        )
        .shown_if("pay", "seniority"),
        Param::choice(
            "Pay",
            "base_pay",
            "Base pay from",
            &[
                ("own", "Each agent's own singleton income"),
                ("median", "The median agent's"),
                ("mean", "The mean agent's"),
            ],
            Live,
        )
        .shown_if("pay", "base"),
        Param::number(
            "Pay",
            "base_share",
            "Base pay share",
            (0.0, 1.0, 0.05),
            Live,
        )
        .shown_if("pay", "base"),
        Param::number("Pay", "hiring", "Hiring standard φ", (0.0, 1.0, 0.05), Live),
        Param::number(
            "Pay",
            "hiring_max",
            "φ up to (0: fixed)",
            (0.0, 1.0, 0.05),
            Live,
        ),
        Param::integer(
            "Measurement",
            "burn_in",
            "Records start after period",
            (0, 100_000),
            Live,
        ),
        Param::integer(
            "Measurement",
            "sample_every",
            "Sample sizes every",
            (1, 100_000),
            Live,
        )
        .with_help("Axtell sampled less often than the longest firm lifetime."),
        Param::integer(
            "Measurement",
            "stop_at",
            "Stop after period",
            (0, 100_000),
            Live,
        )
        .with_help("0: never."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_axtells_base_case() {
        let c = FirmsConfig::default();
        assert_eq!(
            (c.agents, c.a, c.b, c.beta, c.neighbors),
            (1000, 1.0, 1.0, 2.0, 2)
        );
        assert_eq!(
            (
                c.preferences,
                c.network,
                c.activation,
                c.others_effort,
                c.pay
            ),
            (
                Preferences::Uniform,
                Network::Friends,
                Activation::Random,
                OthersEffort::LastPeriod,
                Pay::Equal
            )
        );
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = FirmsConfig {
            agents: 1,
            beta: 0.5,
            theta: 2.0,
            neighbors: 0,
            activation_rate: 0.0,
            effort_window: 0.0,
            seniority_base: 0.5,
            hiring: 1.5,
            sample_every: 0,
            stop_at: 200_000,
            ..FirmsConfig::default()
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
                "beta",
                "theta",
                "neighbors",
                "activation_rate",
                "effort_window",
                "seniority_base",
                "hiring",
                "sample_every",
                "stop_at"
            ]
        );
        // A maximum at or below its value means fixed.
        assert!(FirmsConfig {
            b: 1.0,
            b_max: 0.5,
            ..FirmsConfig::default()
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn the_population_and_network_change_only_on_reset() {
        let next = FirmsConfig {
            neighbors: 4,
            beta: 1.8,
            ..FirmsConfig::default()
        };
        let changes = FirmsConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "neighbors");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Firms(FirmsConfig {
            agents: 50,
            ..FirmsConfig::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
````

Create `crates/sugarscape-core/src/firms/effort.rs` with exactly this content:

````rust
//! An agent's best effort, and the analytics of Axtell's §2.
//!
//! Output is O(E) = a·E + b·E^β over a firm's total effort E; income is an
//! equal share (or a seniority share, or base pay plus a bonus); utility is
//! Cobb–Douglas, (income)^θ (1 − e)^(1−θ), or CES. For the base case (β = 2,
//! Cobb–Douglas, shares proportional to output) the optimum is A99's
//! closed form (5); for constant returns it is (6); otherwise a bisection on
//! the first-order condition (Cobb–Douglas) or a bracketed search (base pay,
//! CES). Non-integer powers go through the portable ln and exp.

use crate::portable::{exp_neg, ln};

/// x^y for x ≥ 0, bit-for-bit the same on every platform.
pub fn powf(x: f64, y: f64) -> f64 {
    if x <= 0.0 {
        return if y > 0.0 {
            0.0
        } else if y == 0.0 {
            1.0
        } else {
            f64::INFINITY
        };
    }
    if y == 0.0 || x == 1.0 {
        return 1.0;
    }
    if y == 1.0 {
        return x;
    }
    if y == 2.0 {
        return x * x;
    }
    let z = y * ln(x);
    if z <= 0.0 {
        exp_neg(z)
    } else if z > 708.0 {
        f64::INFINITY
    } else {
        1.0 / exp_neg(-z)
    }
}

/// A firm's technology.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tech {
    pub a: f64,
    pub b: f64,
    pub beta: f64,
}

impl Tech {
    pub fn output(&self, e: f64) -> f64 {
        let e = e.max(0.0);
        self.a * e + self.b * powf(e, self.beta)
    }
}

/// An agent's preferences between income and leisure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prefs {
    /// (income)^θ (leisure)^(1−θ).
    CobbDouglas { theta: f64 },
    /// CES with weight δ on income; `minus` is the text's (−ρ) convention.
    Ces { delta: f64, rho: f64, minus: bool },
}

impl Prefs {
    /// The weight on income (θ, or δ for CES): what hiring standards compare.
    pub fn weight(&self) -> f64 {
        match *self {
            Prefs::CobbDouglas { theta } => theta,
            Prefs::Ces { delta, .. } => delta,
        }
    }

    pub fn utility(&self, income: f64, leisure: f64) -> f64 {
        let (y, l) = (income.max(0.0), leisure.clamp(0.0, 1.0));
        match *self {
            Prefs::CobbDouglas { theta } => powf(y, theta) * powf(l, 1.0 - theta),
            Prefs::Ces { delta, rho, minus } => {
                let r = if minus { -rho } else { rho };
                if r.abs() < 1e-9 {
                    return powf(y, delta) * powf(l, 1.0 - delta);
                }
                let term = |x: f64| {
                    if x <= 0.0 {
                        if r > 0.0 {
                            0.0
                        } else {
                            f64::INFINITY
                        }
                    } else {
                        powf(x, r)
                    }
                };
                let s = delta * term(y) + (1.0 - delta) * term(l);
                if s.is_infinite() {
                    return 0.0;
                }
                powf(s, 1.0 / r)
            }
        }
    }
}

/// How a firm pays the agent choosing its effort.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Share {
    /// A fixed fraction of output (1/N for equal shares, a seniority weight).
    Fraction(f64),
    /// Base pay `own` plus an equal share of max(0, O − `own` − `others`)
    /// among `n` members.
    Base { own: f64, others: f64, n: f64 },
}

impl Share {
    pub fn income(&self, output: f64) -> f64 {
        match *self {
            Share::Fraction(w) => w * output,
            Share::Base { own, others, n } => own + ((output - own - others) / n).max(0.0),
        }
    }
}

/// One option an agent weighs: a firm's technology, the others' effort, and
/// its share.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Choice {
    pub tech: Tech,
    pub others: f64,
    pub share: Share,
}

impl Choice {
    pub fn utility(&self, prefs: &Prefs, e: f64) -> f64 {
        prefs.utility(
            self.share.income(self.tech.output(e + self.others)),
            1.0 - e,
        )
    }
}

/// How the effort is searched.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Search {
    Exact,
    Grid(u32),
}

/// A99 (5): the Cobb–Douglas optimum for β = 2, b > 0, equal shares,
/// given the others' effort `others`.
pub fn closed_form(theta: f64, others: f64, a: f64, b: f64) -> f64 {
    let t = theta;
    let x = 1.0 + others;
    let disc = a * a + 4.0 * a * b * t * t * x + 4.0 * b * b * t * t * x * x;
    let e = (-a - 2.0 * b * (others - t) + disc.sqrt()) / (2.0 * b * (1.0 + t));
    e.clamp(0.0, 1.0)
}

/// The best effort in [lo, hi] and its utility.
pub fn best(prefs: &Prefs, choice: &Choice, lo: f64, hi: f64, search: Search) -> (f64, f64) {
    let (lo, hi) = (lo.clamp(0.0, 1.0), hi.clamp(0.0, 1.0));
    let e = match search {
        Search::Grid(steps) => {
            let steps = steps.max(1);
            let mut top = (lo, choice.utility(prefs, lo));
            for k in 0..=steps {
                let e = f64::from(k) / f64::from(steps);
                if e < lo || e > hi {
                    continue;
                }
                let u = choice.utility(prefs, e);
                if u > top.1 {
                    top = (e, u);
                }
            }
            return top;
        }
        Search::Exact => exact(prefs, choice, lo, hi),
    };
    (e, choice.utility(prefs, e))
}

fn exact(prefs: &Prefs, choice: &Choice, lo: f64, hi: f64) -> f64 {
    let t = choice.tech;
    match (prefs, choice.share) {
        (Prefs::CobbDouglas { theta }, Share::Fraction(_)) => {
            // Shares proportional to output: the optimum ignores the share.
            if t.beta == 2.0 && t.b > 0.0 {
                closed_form(*theta, choice.others, t.a, t.b).clamp(lo, hi)
            } else if t.b == 0.0 || t.beta == 1.0 {
                // A99 (6): constant returns.
                (theta - choice.others * (1.0 - theta)).clamp(lo, hi)
            } else {
                bisect_cd(*theta, &t, choice.others, lo, hi)
            }
        }
        _ => bracket(prefs, choice, lo, hi),
    }
}

/// The root of d/de ln U = θ O′(E)/O(E) − (1−θ)/(1−e) in [lo, hi]
/// (decreasing in e): safeguarded Newton, falling back to bisection.
fn bisect_cd(theta: f64, t: &Tech, others: f64, lo: f64, hi: f64) -> f64 {
    // g and g′ from one power: E^β (E^(β−1) = E^β/E, E^(β−2) = E^β/E²).
    let g = |e: f64| -> (f64, f64) {
        let big = e + others;
        let cost = if e >= 1.0 {
            if theta < 1.0 {
                f64::INFINITY
            } else {
                0.0
            }
        } else {
            (1.0 - theta) / (1.0 - e)
        };
        let dcost = if e >= 1.0 {
            f64::INFINITY
        } else {
            (1.0 - theta) / ((1.0 - e) * (1.0 - e))
        };
        if big <= 0.0 {
            return (
                if theta > 0.0 { f64::INFINITY } else { -cost },
                f64::NEG_INFINITY,
            );
        }
        let p = powf(big, t.beta);
        let o = t.a * big + t.b * p;
        let o1 = t.a + t.b * t.beta * p / big;
        let o2 = t.b * t.beta * (t.beta - 1.0) * p / (big * big);
        let gain = theta * o1 / o;
        let dgain = theta * (o2 * o - o1 * o1) / (o * o);
        (gain - cost, dgain - dcost)
    };
    if g(lo).0 <= 0.0 {
        return lo;
    }
    if g(hi).0 >= 0.0 {
        return hi;
    }
    let (mut l, mut h) = (lo, hi);
    let mut x = 0.5 * (l + h);
    for _ in 0..100 {
        let (v, d) = g(x);
        if v > 0.0 {
            l = x;
        } else {
            h = x;
        }
        let newton = x - v / d;
        let next = if d < 0.0 && newton > l && newton < h {
            newton
        } else {
            0.5 * (l + h)
        };
        if (next - x).abs() <= 1e-14 || h - l <= 1e-14 {
            return next;
        }
        x = next;
    }
    x
}

/// Where `f` (f(lo) > 0 > f(hi)) crosses zero: the Illinois variant of
/// regula falsi, which keeps the bracket and converges superlinearly.
fn root(f: impl Fn(f64) -> f64, lo: f64, hi: f64) -> f64 {
    let (mut a, mut b) = (lo, hi);
    let (mut fa, mut fb) = (f(a), f(b));
    if !fa.is_finite() || !fb.is_finite() {
        // Infinite ends: halve until both are finite, keeping the signs.
        for _ in 0..60 {
            let m = 0.5 * (a + b);
            let fm = f(m);
            if fm > 0.0 {
                a = m;
                fa = fm;
            } else {
                b = m;
                fb = fm;
            }
            if fa.is_finite() && fb.is_finite() {
                break;
            }
        }
        if !fa.is_finite() || !fb.is_finite() {
            return 0.5 * (a + b);
        }
    }
    let mut side = 0;
    for _ in 0..100 {
        if b - a <= 1e-13 {
            break;
        }
        // A false-position step, or a bisection when extreme values push it
        // onto the bracket's edge.
        let mut c = (a * fb - b * fa) / (fb - fa);
        if !(c > a && c < b) {
            c = 0.5 * (a + b);
            side = 0;
        }
        let fc = f(c);
        if fc == 0.0 {
            return c;
        }
        if fc > 0.0 {
            a = c;
            fa = fc;
            if side == 1 {
                fb *= 0.5;
            }
            side = 1;
        } else {
            b = c;
            fb = fc;
            if side == -1 {
                fa *= 0.5;
            }
            side = -1;
        }
        if (b - a).abs() <= 1e-13 {
            break;
        }
    }
    0.5 * (a + b)
}

/// The best of `candidates` by utility.
fn best_of(prefs: &Prefs, choice: &Choice, candidates: &[f64]) -> f64 {
    let mut top = (candidates[0], choice.utility(prefs, candidates[0]));
    for &e in &candidates[1..] {
        let u = choice.utility(prefs, e);
        if u > top.1 {
            top = (e, u);
        }
    }
    top.0
}

/// Base pay or CES: the maximum on [lo, hi], split where the bonus starts
/// (income is kinked there). Cobb–Douglas solves its first-order condition
/// analytically in each piece; CES by the sign of its utility's slope (a
/// central difference). Endpoints are always candidates.
fn bracket(prefs: &Prefs, choice: &Choice, lo: f64, hi: f64) -> f64 {
    if hi <= lo {
        return lo;
    }
    let t = choice.tech;
    let mut cuts = vec![lo];
    if let Share::Base { own, others, .. } = choice.share {
        let floor = own + others;
        let over = |e: f64| t.output(e + choice.others) - floor;
        if over(lo) < 0.0 && over(hi) > 0.0 {
            cuts.push(root(|e| -over(e), lo, hi));
        }
    }
    cuts.push(hi);
    let mut candidates = cuts.clone();
    // Whether income rises with output on a piece: always for proportional
    // shares; for base pay, above the kink (decided per piece, not per
    // point, so rounding at the kink cannot hide a piece's slope).
    let floor = match choice.share {
        Share::Base { own, others, .. } => own + others,
        Share::Fraction(_) => f64::NEG_INFINITY,
    };
    for w in cuts.windows(2) {
        let (a, b) = (w[0], w[1]);
        if b - a <= 1e-12 {
            continue;
        }
        let bonus = t.output(0.5 * (a + b) + choice.others) >= floor;
        let slope: Box<dyn Fn(f64) -> f64> = match (prefs, choice.share) {
            (Prefs::CobbDouglas { theta }, Share::Base { n, .. }) => {
                let theta = *theta;
                Box::new(move |e: f64| {
                    let big = e + choice.others;
                    let y = choice.share.income(t.output(big));
                    let dy = if bonus {
                        let p = if big > 0.0 { powf(big, t.beta) } else { 0.0 };
                        (t.a + if big > 0.0 {
                            t.b * t.beta * p / big
                        } else {
                            0.0
                        }) / n
                    } else {
                        0.0
                    };
                    let gain = if y > 0.0 {
                        theta * dy / y
                    } else if dy > 0.0 && theta > 0.0 {
                        f64::INFINITY
                    } else {
                        0.0
                    };
                    let cost = if e >= 1.0 {
                        f64::INFINITY
                    } else {
                        (1.0 - theta) / (1.0 - e)
                    };
                    gain - cost
                })
            }
            (Prefs::Ces { delta, rho, minus }, share) => {
                // The sign of dU/de: δ y^(r−1) y′ − (1−δ) l^(r−1), with r the
                // exponent in the convention used (CES utility rises with its
                // inner sum when r > 0 and falls when r < 0; the r factor cancels).
                let (delta, r) = (*delta, if *minus { -*rho } else { *rho });
                Box::new(move |e: f64| {
                    let big = e + choice.others;
                    let o = t.output(big);
                    let y = share.income(o);
                    let o1 = if big > 0.0 {
                        t.a + t.b * t.beta * powf(big, t.beta) / big
                    } else {
                        t.a
                    };
                    let dy = match share {
                        Share::Fraction(w) => w * o1,
                        Share::Base { n, .. } => {
                            if bonus {
                                o1 / n
                            } else {
                                0.0
                            }
                        }
                    };
                    let l = 1.0 - e;
                    if r.abs() < 1e-9 {
                        let gain = if y > 0.0 {
                            delta * dy / y
                        } else if dy > 0.0 {
                            f64::INFINITY
                        } else {
                            0.0
                        };
                        return gain
                            - if l > 0.0 {
                                (1.0 - delta) / l
                            } else {
                                f64::INFINITY
                            };
                    }
                    let py = if y > 0.0 {
                        powf(y, r - 1.0)
                    } else if r < 1.0 {
                        f64::INFINITY
                    } else {
                        0.0
                    };
                    let pl = if l > 0.0 {
                        powf(l, r - 1.0)
                    } else if r < 1.0 {
                        f64::INFINITY
                    } else {
                        0.0
                    };
                    let (g, c) = (delta * py * dy, (1.0 - delta) * pl);
                    if g.is_infinite() && c.is_infinite() {
                        0.0
                    } else {
                        g - c
                    }
                })
            }
            (Prefs::CobbDouglas { .. }, _) => {
                unreachable!("Cobb–Douglas with proportional shares is solved earlier")
            }
        };
        if slope(a) > 0.0 && slope(b) < 0.0 {
            candidates.push(root(&slope, a, b));
        }
    }
    best_of(prefs, choice, &candidates)
}

/// §2: the symmetric Nash effort in a homogeneous group of `n` agents of
/// preference θ (equal shares, β = 2), and each agent's utility there.
pub fn nash(theta: f64, n: u32, a: f64, b: f64) -> (f64, f64) {
    let others = f64::from(n - 1);
    let (mut l, mut h) = (0.0, 1.0);
    for _ in 0..80 {
        let m = 0.5 * (l + h);
        if closed_form(theta, others * m, a, b) > m {
            l = m;
        } else {
            h = m;
        }
    }
    let e = 0.5 * (l + h);
    let big = f64::from(n) * e;
    let u = Prefs::CobbDouglas { theta }.utility((a * big + b * big * big) / f64::from(n), 1.0 - e);
    (e, u)
}

/// A99 (9): the Jacobian's off-diagonal entry for an agent of preference θ
/// against others' effort `others` (β = 2).
pub fn jacobian_k(theta: f64, others: f64, a: f64, b: f64) -> f64 {
    let x = 1.0 + others;
    let root = (a * a + 4.0 * b * theta * theta * x * (a + b * x)).sqrt();
    (-1.0 + theta * theta * (a + 2.0 * b * x) / root) / (1.0 + theta)
}

/// The dominant eigenvalue (N − 1)·k of a homogeneous group at its Nash
/// equilibrium; the group is stable while it is at least −1.
pub fn eigenvalue(theta: f64, n: u32, a: f64, b: f64) -> f64 {
    if n < 2 {
        return 0.0;
    }
    let (e, _) = nash(theta, n, a, b);
    f64::from(n - 1) * jacobian_k(theta, f64::from(n - 1) * e, a, b)
}

/// The largest stable homogeneous group of preference θ (up to `cap`).
pub fn max_stable_size(theta: f64, a: f64, b: f64, cap: u32) -> u32 {
    (1..=cap)
        .take_while(|&n| eigenvalue(theta, n, a, b) >= -1.0)
        .last()
        .unwrap_or(1)
}

/// The homogeneous group size that maximizes each member's Nash utility.
pub fn optimal_size(theta: f64, a: f64, b: f64, cap: u32) -> u32 {
    let mut top = (1, f64::MIN);
    for n in 1..=cap {
        let u = nash(theta, n, a, b).1;
        if u > top.1 {
            top = (n, u);
        }
    }
    top.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cd(theta: f64) -> Prefs {
        Prefs::CobbDouglas { theta }
    }

    fn equal(n: f64, others: f64) -> Choice {
        Choice {
            tech: Tech {
                a: 1.0,
                b: 1.0,
                beta: 2.0,
            },
            others,
            share: Share::Fraction(1.0 / n),
        }
    }

    #[test]
    fn the_closed_form_is_the_line_searchs_limit() {
        for &theta in &[0.1, 0.5, 0.7, 0.95] {
            for &others in &[0.0, 0.5, 2.0, 10.0] {
                let c = equal(3.0, others);
                let (e, _) = best(&cd(theta), &c, 0.0, 1.0, Search::Exact);
                let (g, _) = best(&cd(theta), &c, 0.0, 1.0, Search::Grid(100_000));
                assert!((e - g).abs() < 2e-5, "θ {theta} E {others}: {e} vs {g}");
                // And the bisection agrees with (5) at β = 2.
                let b = bisect_cd(theta, &c.tech, others, 0.0, 1.0);
                assert!((e - b).abs() < 1e-9, "{e} vs {b}");
            }
        }
    }

    #[test]
    fn table_1_reproduces_to_three_places() {
        // A99 Table 1, θ = 0.7: e*, U(e*), k and (N − 1)k for N = 1..7.
        let rows = [
            (1, 0.770, 0.799, f64::NAN),
            (2, 0.646, 0.964, -0.188),
            (3, 0.558, 1.036, -0.368),
            (4, 0.492, 1.065, -0.547),
            (5, 0.441, 1.069, -0.726),
            (6, 0.399, 1.061, -0.904),
            (7, 0.364, 1.045, -1.082),
        ];
        for (n, e, u, lambda) in rows {
            let (ne, nu) = nash(0.7, n, 1.0, 1.0);
            assert!(
                (ne - e).abs() < 5e-4 && (nu - u).abs() < 5e-4,
                "N {n}: {ne} {nu}"
            );
            if n > 1 {
                assert!(
                    (eigenvalue(0.7, n, 1.0, 1.0) - lambda).abs() < 5e-4,
                    "N {n}"
                );
            }
        }
        assert_eq!(
            (
                max_stable_size(0.7, 1.0, 1.0, 50),
                optimal_size(0.7, 1.0, 1.0, 50)
            ),
            (6, 5)
        );
    }

    #[test]
    fn optimal_and_stable_sizes_grow_with_theta() {
        let sizes: Vec<(u32, u32)> = [0.5, 0.8, 0.9, 0.95]
            .iter()
            .map(|&t| {
                (
                    optimal_size(t, 1.0, 1.0, 200),
                    max_stable_size(t, 1.0, 1.0, 200),
                )
            })
            .collect();
        assert_eq!(sizes, [(2, 3), (8, 9), (18, 19), (38, 39)]);
    }

    #[test]
    fn constant_returns_follow_equation_6() {
        let c = Choice {
            tech: Tech {
                a: 1.0,
                b: 0.0,
                beta: 2.0,
            },
            others: 0.5,
            share: Share::Fraction(0.5),
        };
        let (e, _) = best(&cd(0.6), &c, 0.0, 1.0, Search::Exact);
        assert!((e - (0.6 - 0.5 * 0.4)).abs() < 1e-12);
        let (g, _) = best(&cd(0.6), &c, 0.0, 1.0, Search::Grid(100_000));
        assert!((e - g).abs() < 2e-5);
    }

    #[test]
    fn general_beta_ces_and_base_pay_find_the_maximum() {
        let prefs = [
            cd(0.6),
            Prefs::Ces {
                delta: 0.6,
                rho: 0.5,
                minus: true,
            },
            Prefs::Ces {
                delta: 0.4,
                rho: -0.5,
                minus: false,
            },
        ];
        let choices = [
            Choice {
                tech: Tech {
                    a: 1.0,
                    b: 1.0,
                    beta: 1.7,
                },
                others: 1.2,
                share: Share::Fraction(0.25),
            },
            Choice {
                tech: Tech {
                    a: 0.3,
                    b: 1.1,
                    beta: 2.1,
                },
                others: 0.0,
                share: Share::Fraction(1.0),
            },
            Choice {
                tech: Tech {
                    a: 1.0,
                    b: 1.0,
                    beta: 2.0,
                },
                others: 2.0,
                share: Share::Base {
                    own: 0.4,
                    others: 1.0,
                    n: 4.0,
                },
            },
        ];
        for p in &prefs {
            for c in &choices {
                let (e, u) = best(p, c, 0.0, 1.0, Search::Exact);
                let (_, ug) = best(p, c, 0.0, 1.0, Search::Grid(20_000));
                assert!(u >= ug - 1e-6, "{p:?} {c:?}: {e} {u} < {ug}");
            }
        }
    }

    #[test]
    fn the_solvers_match_a_fine_grid_on_random_cases() {
        use crate::rng;
        use rand::Rng;
        let mut r = rng::seeded(11);
        for case in 0..600 {
            let prefs = if case % 2 == 0 {
                Prefs::CobbDouglas { theta: r.gen() }
            } else {
                Prefs::Ces {
                    delta: r.gen(),
                    rho: r.gen_range(-1.0..10.0),
                    minus: r.gen(),
                }
            };
            let tech = Tech {
                a: r.gen_range(0.0..1.0),
                b: r.gen_range(0.5..1.5),
                beta: r.gen_range(1.5..2.1),
            };
            let n: f64 = r.gen_range(1.0..20.0_f64).floor();
            let share = if r.gen::<bool>() {
                Share::Fraction(1.0 / n)
            } else {
                let own = r.gen_range(0.0..1.0);
                Share::Base {
                    own,
                    others: own * (n - 1.0) * r.gen_range(0.5..1.5),
                    n,
                }
            };
            let choice = Choice {
                tech,
                others: r.gen_range(0.0..(n - 1.0) * 0.8 + 0.01),
                share,
            };
            let (lo, hi) = if r.gen::<bool>() {
                (0.0, 1.0)
            } else {
                let c: f64 = r.gen();
                (c - 0.05, c + 0.05)
            };
            let (e, u) = best(&prefs, &choice, lo, hi, Search::Exact);
            // (Checked once at 20 000 cases against a 20 000-step grid.)
            let (g, ug) = best(&prefs, &choice, lo, hi, Search::Grid(5_000));
            assert!(
                u >= ug - 1e-7 * ug.abs().max(1.0),
                "case {case}: {prefs:?} {choice:?} [{lo}, {hi}]: exact {e} {u} < grid {g} {ug}"
            );
        }
    }

    #[test]
    fn a_window_clamps_the_optimum() {
        let c = equal(2.0, 0.3);
        let (free, _) = best(&cd(0.9), &c, 0.0, 1.0, Search::Exact);
        let (held, _) = best(&cd(0.9), &c, 0.1, 0.2, Search::Exact);
        assert!(free > 0.2);
        assert_eq!(held, 0.2);
    }

    #[test]
    fn ces_approaches_its_limits() {
        // The text's convention: ρ = −1 is linear, ρ → 0 Cobb–Douglas.
        let linear = Prefs::Ces {
            delta: 0.3,
            rho: -1.0,
            minus: true,
        };
        assert!((linear.utility(2.0, 0.5) - (0.3 * 2.0 + 0.7 * 0.5)).abs() < 1e-12);
        let near = Prefs::Ces {
            delta: 0.3,
            rho: 1e-12,
            minus: true,
        };
        assert!((near.utility(2.0, 0.5) - cd(0.3).utility(2.0, 0.5)).abs() < 1e-9);
        // Large ρ: Leontief, near the smaller input.
        let leontief = Prefs::Ces {
            delta: 0.5,
            rho: 200.0,
            minus: true,
        };
        assert!((leontief.utility(2.0, 0.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn powf_matches_the_library() {
        for &(x, y) in &[(0.3, 0.7), (2.5, 1.9), (10.0, 2.1), (0.01, 3.0)] {
            let (p, q) = (powf(x, y), x.powf(y));
            assert!(((p - q) / q).abs() < 1e-12, "{x}^{y}: {p} {q}");
        }
        assert_eq!(
            (powf(0.0, 0.5), powf(0.0, 0.0), powf(3.0, 2.0)),
            (0.0, 1.0, 9.0)
        );
    }
}
````

Create `crates/sugarscape-core/src/firms/fit.rs` with exactly this content:

````rust
//! The records a run keeps for Axtell's §3.4 measurements, in bounded
//! memory (histograms and running sums), and the fits made from them: the
//! firm-size exponent µ by the paper's OLS and by maximum likelihood, the
//! output exponent, productivity, the growth-rate distribution (Laplace
//! against Gaussian) and σ_r's dependence on size (γ), and lifetimes.

use crate::portable::ln;

/// Output bins: width 0.05 in ln(output), from output 1.
const OUT_BIN: f64 = 0.05;
/// Growth-rate bins over [−5, 5].
const R_MAX: f64 = 5.0;
const R_BINS: usize = 2000;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Records {
    /// Firms of each size, pooled over the sampled periods (index = size).
    pub sizes: Vec<u64>,
    /// Firms by ln(output) bin, and per size: firms and their summed output.
    pub outputs: Vec<u64>,
    pub size_output: Vec<(u64, f64)>,
    /// Growth rates r = ln(s_t / s_{t−1}): a histogram, and per prior size
    /// the count, Σr and Σr².
    pub growth: Vec<u64>,
    pub growth_by_size: Vec<(u64, f64, f64)>,
    /// Lifetimes (periods) of firms that died, by lifetime; and the same for
    /// firms that ever had a second member.
    pub lifetimes: Vec<u64>,
    pub lifetimes_team: Vec<u64>,
}

fn bump<T: Clone + Default>(v: &mut Vec<T>, i: usize) -> &mut T {
    if v.len() <= i {
        v.resize(i + 1, T::default());
    }
    &mut v[i]
}

impl Records {
    /// One sampled period's firms: (size, output).
    pub fn sample(&mut self, firms: &[(u32, f64)]) {
        for &(s, o) in firms {
            *bump(&mut self.sizes, s as usize) += 1;
            if o >= 1.0 {
                *bump(&mut self.outputs, (ln(o) / OUT_BIN) as usize) += 1;
            }
            let cell = bump(&mut self.size_output, s as usize);
            cell.0 += 1;
            cell.1 += o;
        }
    }

    /// A surviving firm's size last period and now.
    pub fn grow(&mut self, before: u32, after: u32) {
        if before == 0 || after == 0 {
            return;
        }
        let r = ln(f64::from(after) / f64::from(before));
        let bin = (((r + R_MAX) / (2.0 * R_MAX)) * R_BINS as f64).clamp(0.0, (R_BINS - 1) as f64)
            as usize;
        if self.growth.len() < R_BINS {
            self.growth.resize(R_BINS, 0);
        }
        self.growth[bin] += 1;
        let cell = bump(&mut self.growth_by_size, before as usize);
        cell.0 += 1;
        cell.1 += r;
        cell.2 += r * r;
    }

    pub fn died(&mut self, lifetime: u64, team: bool) {
        *bump(&mut self.lifetimes, lifetime as usize) += 1;
        if team {
            *bump(&mut self.lifetimes_team, lifetime as usize) += 1;
        }
    }
}

/// e^x, portable.
fn exp(x: f64) -> f64 {
    if x <= 0.0 {
        crate::portable::exp_neg(x)
    } else {
        1.0 / crate::portable::exp_neg(-x)
    }
}

/// Ordinary least squares slope and intercept of y on x.
pub fn ols(points: &[(f64, f64)]) -> Option<(f64, f64)> {
    let n = points.len() as f64;
    if points.len() < 2 {
        return None;
    }
    let mx = points.iter().map(|p| p.0).sum::<f64>() / n;
    let my = points.iter().map(|p| p.1).sum::<f64>() / n;
    let sxx: f64 = points.iter().map(|p| (p.0 - mx) * (p.0 - mx)).sum();
    if sxx == 0.0 {
        return None;
    }
    let slope = points.iter().map(|p| (p.0 - mx) * (p.1 - my)).sum::<f64>() / sxx;
    Some((slope, my - slope * mx))
}

/// A99 §3.4's µ: OLS of ln p(s) on ln s, dropping size 1 and frequencies
/// below 10⁻⁵; p(s) ∝ s^−(1+µ). NaN without enough points.
pub fn mu_ols(sizes: &[u64]) -> f64 {
    let total: u64 = sizes.iter().sum();
    if total == 0 {
        return f64::NAN;
    }
    let points: Vec<(f64, f64)> = sizes
        .iter()
        .enumerate()
        .skip(2)
        .filter(|&(_, &c)| c > 0 && c as f64 / total as f64 >= 1e-5)
        .map(|(s, &c)| (ln(s as f64), ln(c as f64 / total as f64)))
        .collect();
    ols(&points).map_or(f64::NAN, |(slope, _)| -slope - 1.0)
}

/// The discrete power law's maximum-likelihood exponent for sizes ≥ `smin`
/// (Clauset, Shalizi and Newman 2009, eq. 3.5: maximizing −n ln ζ(α, smin) −
/// α Σ ln s, the Hurwitz zeta summed to the largest size seen and the tail
/// by its integral), as µ = α − 1.
pub fn mu_mle(sizes: &[u64], smin: usize) -> f64 {
    let (mut n, mut sum_ln) = (0.0, 0.0);
    for (s, &c) in sizes.iter().enumerate().skip(smin) {
        if c > 0 {
            n += c as f64;
            sum_ln += c as f64 * ln(s as f64);
        }
    }
    if n == 0.0 || sizes.len() <= smin + 1 {
        return f64::NAN;
    }
    let top = sizes.len() as f64;
    let zeta = |alpha: f64| {
        let mut z = 0.0;
        for s in smin..sizes.len() {
            z += 1.0 / super::effort::powf(s as f64, alpha);
        }
        // The rest, from `top`: ∫ x^−α dx from top − ½.
        z + super::effort::powf(top - 0.5, 1.0 - alpha) / (alpha - 1.0)
    };
    let loglik = |alpha: f64| -n * ln(zeta(alpha)) - alpha * sum_ln;
    let (mut l, mut h) = (1.000_1, 6.0);
    let phi = 0.5 * (5.0_f64.sqrt() - 1.0);
    for _ in 0..60 {
        let (x1, x2) = (h - phi * (h - l), l + phi * (h - l));
        if loglik(x1) >= loglik(x2) {
            h = x2;
        } else {
            l = x1;
        }
    }
    0.5 * (l + h) - 1.0
}

/// The output-distribution exponent on the µ convention (density per unit
/// output over the ln bins; dropping densities below 10⁻⁵ of the total).
pub fn output_exponent(outputs: &[u64]) -> f64 {
    let total: u64 = outputs.iter().sum();
    if total == 0 {
        return f64::NAN;
    }
    let points: Vec<(f64, f64)> = outputs
        .iter()
        .enumerate()
        .filter(|&(_, &c)| c > 0 && c as f64 / total as f64 >= 1e-5)
        .map(|(k, &c)| {
            let lo = k as f64 * OUT_BIN;
            let (x0, x1) = (exp(lo), exp(lo + OUT_BIN));
            let mid = 0.5 * (x0 + x1);
            (ln(mid), ln(c as f64 / total as f64 / (x1 - x0)))
        })
        .collect();
    ols(&points).map_or(f64::NAN, |(slope, _)| -slope - 1.0)
}

/// Productivity: mean output against size, fitted as c·s^k (A99 fig. 19).
pub fn productivity(size_output: &[(u64, f64)]) -> (f64, f64) {
    let points: Vec<(f64, f64)> = size_output
        .iter()
        .enumerate()
        .skip(1)
        .filter(|(_, c)| c.0 > 0 && c.1 > 0.0)
        .map(|(s, c)| (ln(s as f64), ln(c.1 / c.0 as f64)))
        .collect();
    ols(&points).map_or((f64::NAN, f64::NAN), |(slope, icept)| (exp(icept), slope))
}

/// The growth-rate distribution: mean, standard deviation, and the mean
/// log-likelihood per observation of the fitted Laplace and Gaussian.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GrowthFit {
    pub n: u64,
    pub mean: f64,
    pub sd: f64,
    pub laplace_ll: f64,
    pub gauss_ll: f64,
}

pub fn growth_fit(growth: &[u64]) -> GrowthFit {
    let n: u64 = growth.iter().sum();
    let center = |k: usize| -R_MAX + (k as f64 + 0.5) * 2.0 * R_MAX / R_BINS as f64;
    if n < 2 {
        return GrowthFit {
            n,
            mean: f64::NAN,
            sd: f64::NAN,
            laplace_ll: f64::NAN,
            gauss_ll: f64::NAN,
        };
    }
    let nf = n as f64;
    let mean = growth
        .iter()
        .enumerate()
        .map(|(k, &c)| c as f64 * center(k))
        .sum::<f64>()
        / nf;
    let var = growth
        .iter()
        .enumerate()
        .map(|(k, &c)| c as f64 * (center(k) - mean).powi(2))
        .sum::<f64>()
        / nf;
    // The Laplace's MLE: location the median, scale the mean absolute deviation.
    let mut seen = 0;
    let mut median = 0.0;
    for (k, &c) in growth.iter().enumerate() {
        seen += c;
        if 2 * seen >= n {
            median = center(k);
            break;
        }
    }
    let scale = growth
        .iter()
        .enumerate()
        .map(|(k, &c)| c as f64 * (center(k) - median).abs())
        .sum::<f64>()
        / nf;
    let two_pi_ln = ln(2.0 * std::f64::consts::PI);
    let laplace_ll = if scale > 0.0 {
        -ln(2.0 * scale) - 1.0
    } else {
        f64::INFINITY
    };
    let gauss_ll = if var > 0.0 {
        -0.5 * (two_pi_ln + ln(var)) - 0.5
    } else {
        f64::INFINITY
    };
    GrowthFit {
        n,
        mean,
        sd: var.sqrt(),
        laplace_ll,
        gauss_ll,
    }
}

/// σ_r by prior size, and γ in σ_r ∝ s^−γ by OLS over sizes from `from` to
/// `to` with at least `min_n` observations (A99 drops the first two sizes and
/// the noisy large ones).
pub fn gamma(growth_by_size: &[(u64, f64, f64)], from: usize, to: usize, min_n: u64) -> f64 {
    let points: Vec<(f64, f64)> = growth_by_size
        .iter()
        .enumerate()
        .filter(|&(s, c)| s >= from && s <= to && c.0 >= min_n)
        .filter_map(|(s, c)| {
            let n = c.0 as f64;
            let var = c.2 / n - (c.1 / n) * (c.1 / n);
            (var > 0.0).then(|| (ln(s as f64), 0.5 * ln(var)))
        })
        .collect();
    ols(&points).map_or(f64::NAN, |(slope, _)| -slope)
}

/// Lifetimes: count, mean, standard deviation, and the slope of lifetime on
/// log₁₀ rank (longest first; the 3 longest dropped, as A99 fig. 22).
pub fn lifetimes(hist: &[u64]) -> (u64, f64, f64, f64) {
    let n: u64 = hist.iter().sum();
    if n == 0 {
        return (0, f64::NAN, f64::NAN, f64::NAN);
    }
    let nf = n as f64;
    let mean = hist
        .iter()
        .enumerate()
        .map(|(l, &c)| c as f64 * l as f64)
        .sum::<f64>()
        / nf;
    let var = hist
        .iter()
        .enumerate()
        .map(|(l, &c)| c as f64 * (l as f64 - mean).powi(2))
        .sum::<f64>()
        / nf;
    // Rank plot, streamed: ranks 4.. in descending order of lifetime.
    let (mut sx, mut sy, mut sxx, mut sxy, mut m) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut rank = 0u64;
    let log10 = ln(10.0);
    for (l, &c) in hist.iter().enumerate().rev() {
        for _ in 0..c {
            rank += 1;
            if rank <= 3 {
                continue;
            }
            let x = ln(rank as f64) / log10;
            let y = l as f64;
            sx += x;
            sy += y;
            sxx += x * x;
            sxy += x * y;
            m += 1.0;
        }
    }
    let slope = if m >= 2.0 {
        (m * sxy - sx * sy) / (m * sxx - sx * sx)
    } else {
        f64::NAN
    };
    (n, mean, var.sqrt(), slope)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;
    use rand::Rng;

    /// A synthetic discrete power law with exponent µ (p(s) ∝ s^−(1+µ)),
    /// sizes 1..=cap, `n` draws by inversion.
    fn power_law(mu: f64, n: usize, cap: usize) -> Vec<u64> {
        let weights: Vec<f64> = (1..=cap).map(|s| (s as f64).powf(-(1.0 + mu))).collect();
        let total: f64 = weights.iter().sum();
        let mut cdf = Vec::with_capacity(cap);
        let mut acc = 0.0;
        for w in &weights {
            acc += w / total;
            cdf.push(acc);
        }
        let mut r = rng::seeded(3);
        let mut hist = vec![0u64; cap + 1];
        for _ in 0..n {
            let u: f64 = r.gen();
            let s = cdf.partition_point(|&c| c < u) + 1;
            hist[s.min(cap)] += 1;
        }
        hist
    }

    #[test]
    fn both_estimators_recover_a_known_exponent() {
        let hist = power_law(1.3, 1_000_000, 5000);
        let (o, m) = (mu_ols(&hist), mu_mle(&hist, 2));
        assert!((o - 1.3).abs() < 0.1, "OLS {o}");
        assert!((m - 1.3).abs() < 0.02, "MLE {m}");
    }

    #[test]
    fn the_ols_drops_size_one_and_rare_sizes() {
        let mut hist = vec![0u64; 11];
        for (s, c) in hist.iter_mut().enumerate().skip(2) {
            *c = (1_000_000.0 / (s as f64).powi(3)) as u64;
        }
        let base = mu_ols(&hist);
        hist[1] = 10_000_000; // size 1 is dropped
        hist.push(1); // frequency below 10⁻⁵ is dropped
        assert!((mu_ols(&hist) - base).abs() < 1e-9);
        assert!((base - 2.0).abs() < 0.01, "{base}");
    }

    #[test]
    fn laplace_wins_on_laplace_data_and_loses_on_gaussian() {
        let mut r = rng::seeded(5);
        let (mut lap, mut gau) = (Records::default(), Records::default());
        for _ in 0..200_000 {
            let u: f64 = r.gen::<f64>() - 0.5;
            let x = -0.3 * u.signum() * (1.0 - 2.0 * u.abs()).ln();
            lap.grow(100, (100.0 * x.exp()).round().max(1.0) as u32);
            let (u1, u2): (f64, f64) = (r.gen(), r.gen());
            let z = (-2.0 * (1.0 - u1).ln()).sqrt() * (std::f64::consts::TAU * u2).cos() * 0.3;
            gau.grow(100, (100.0 * z.exp()).round().max(1.0) as u32);
        }
        let (fl, fg) = (growth_fit(&lap.growth), growth_fit(&gau.growth));
        assert!(fl.laplace_ll > fl.gauss_ll, "{fl:?}");
        assert!(fg.gauss_ll > fg.laplace_ll, "{fg:?}");
    }

    #[test]
    fn gamma_recovers_a_known_scaling() {
        let mut by = vec![(0u64, 0.0, 0.0); 200];
        for (s, cell) in by.iter_mut().enumerate().skip(1) {
            let sd = 0.5 * (s as f64).powf(-0.2);
            // A two-point distribution ±sd has mean 0 and variance sd².
            *cell = (1000, 0.0, 1000.0 * sd * sd);
        }
        assert!((gamma(&by, 3, 150, 10) - 0.2).abs() < 1e-9);
    }

    #[test]
    fn lifetimes_report_mean_sd_and_rank_slope() {
        let mut hist = vec![0u64; 11];
        hist[2] = 3;
        hist[10] = 1;
        let (n, mean, sd, _) = lifetimes(&hist);
        assert_eq!(n, 4);
        assert!((mean - 4.0).abs() < 1e-12 && (sd - 12f64.sqrt()).abs() < 1e-12);
        // Exponential lifetimes give a straight line in log rank.
        let mut exp = vec![0u64; 200];
        for (l, c) in exp.iter_mut().enumerate() {
            *c = (10_000.0 * (-(l as f64) / 20.0).exp()) as u64;
        }
        let slope = lifetimes(&exp).3;
        assert!((slope + 20.0 * 10f64.ln()).abs() < 3.0, "{slope}");
    }

    #[test]
    fn productivity_recovers_constant_returns() {
        let so: Vec<(u64, f64)> = (0..100)
            .map(|s| {
                if s == 0 {
                    (0, 0.0)
                } else {
                    (10, 10.0 * 2.0 * s as f64)
                }
            })
            .collect();
        let (c, k) = productivity(&so);
        assert!((k - 1.0).abs() < 1e-9 && (c - 2.0).abs() < 1e-9, "{c} {k}");
    }
}
````

Create `crates/sugarscape-core/src/firms/stats.rs` with exactly this content:

````rust
//! The Emergence of Firms' statistics: each period's firms, births and
//! deaths, sizes, effort, output, pay and utility, and the running scaling
//! exponents and mean lifetime from the records since the burn-in.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 15] = [
    "firms",
    "births",
    "deaths",
    "mean_size",
    "largest",
    "singletons",
    "effort",
    "output",
    "income",
    "utility",
    "largest_output_share",
    "mu",
    "mu_mle",
    "lifetime",
    "period",
];

/// One period's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct FirmsSnapshot {
    pub tick: u64,
    /// Firms alive at the period's end, founded and dissolved during it.
    pub firms: u32,
    pub births: u32,
    pub deaths: u32,
    /// Agents per firm, the largest firm, and the share of firms of one.
    pub mean_size: f64,
    pub largest: u32,
    pub singletons: f64,
    /// Means over agents at the period's production.
    pub effort: f64,
    /// Total output of all firms.
    pub output: f64,
    pub income: f64,
    pub utility: f64,
    /// The largest firm's share of total output.
    pub largest_output_share: f64,
    /// The size exponent by A99's OLS and by maximum likelihood, over the
    /// sizes sampled since the burn-in (NaN before).
    pub mu: f64,
    pub mu_mle: f64,
    /// Mean lifetime of firms dissolved since the burn-in (NaN before).
    pub lifetime: f64,
    pub period: u64,
}

impl Series for FirmsSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "firms" => f64::from(self.firms),
            "births" => f64::from(self.births),
            "deaths" => f64::from(self.deaths),
            "mean_size" => self.mean_size,
            "largest" => f64::from(self.largest),
            "singletons" => self.singletons,
            "effort" => self.effort,
            "output" => self.output,
            "income" => self.income,
            "utility" => self.utility,
            "largest_output_share" => self.largest_output_share,
            "mu" => self.mu,
            "mu_mle" => self.mu_mle,
            "lifetime" => self.lifetime,
            "period" => self.period as f64,
            _ => return None,
        })
    }
}
````

Create `crates/sugarscape-core/src/firms/view.rs` with exactly this content:

````rust
//! The frame: after A99's Animation 1 — each firm a row of cells, its
//! longest-serving member first, the largest firms first — and beside it
//! the firm-size distribution on log-log axes with its OLS line.

use crate::render::{lerp, Rgb};

/// Firm rows: cells of `CELL` pixels, up to `COLS` members and `ROWS` firms.
pub const CELL: usize = 3;
pub const COLS: usize = 200;
pub const ROWS: usize = 166;
pub const FIRMS_W: usize = COLS * CELL;
pub const FIRMS_H: usize = ROWS * CELL;
/// The size-distribution plot beside it.
pub const GAP: usize = 8;
pub const PLOT: usize = 200;
pub const PLOT_X: usize = FIRMS_W + GAP;
pub const WIDE: usize = FIRMS_W + GAP + PLOT;
pub const TALL: usize = FIRMS_H;
/// The plot's axes: ln size 0…ln 1000, ln frequency ln 10⁻⁷…0.
pub const LN_SIZE_MAX: f64 = 6.907_755_278_982_137;
pub const LN_FREQ_MIN: f64 = -16.118_095_650_958_32;

pub const FOUNDER: Rgb = [0xe0, 0x3c, 0x31];
pub const MEMBER: Rgb = [0x4a, 0x7c, 0xd8];
pub const LOW: Rgb = [0x2a, 0x26, 0x20];
pub const HIGH: Rgb = [0xf2, 0xc1, 0x4e];
pub const AXIS: Rgb = [0x5a, 0x56, 0x50];
pub const POINT: Rgb = [0xd8, 0xd4, 0xca];
pub const FIT: Rgb = [0xe0, 0x3c, 0x31];

pub fn scale(t: f64, from: Rgb, to: Rgb) -> Rgb {
    lerp(from, to, t.clamp(0.0, 1.0))
}

/// A plot point's pixel for ln size `x` and ln frequency `y` (None if off
/// the plot).
pub fn plot_at(x: f64, y: f64) -> Option<(usize, usize)> {
    let px = x / LN_SIZE_MAX;
    let py = 1.0 - (y - LN_FREQ_MIN) / -LN_FREQ_MIN;
    ((0.0..=1.0).contains(&px) && (0.0..=1.0).contains(&py)).then(|| {
        (
            PLOT_X + (px * (PLOT - 1) as f64) as usize,
            (py * (PLOT - 1) as f64) as usize,
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_frame_holds_the_rows_and_the_plot() {
        assert_eq!((WIDE, TALL), (600 + 8 + 200, 498));
        assert_eq!(plot_at(0.0, 0.0), Some((PLOT_X, 0)));
        assert_eq!(
            plot_at(LN_SIZE_MAX, LN_FREQ_MIN),
            Some((PLOT_X + PLOT - 1, PLOT - 1))
        );
        assert_eq!(plot_at(-0.1, 0.0), None);
    }
}
````

Create `crates/sugarscape-core/src/firms/world.rs` with exactly this content:

````rust
//! The Emergence of Firms world. Each tick is a period: a number of agent
//! activations, then production. An activated agent weighs staying in its
//! firm (re-choosing its effort), starting a firm alone, and joining each
//! firm its network shows it, and takes the best (A99 §3.1). At the
//! period's end every firm produces a·E + b·E^β and pays its members.

use std::fmt::Write;

use rand::Rng;
use serde::Serialize;

use super::config::{
    Activation, AdjustScope, BasePay, CesSign, EffortSearch, FirmsConfig, Initial, Network,
    OthersEffort, Pay, Preferences, RandomBehavior, SeniorityOrder,
};
use super::effort::{best, Choice, Prefs, Search, Share, Tech};
use super::fit::{mu_mle, mu_ols, ols, Records};
use super::stats::FirmsSnapshot;
use super::view::{
    plot_at, scale, AXIS, CELL, COLS, FIRMS_H, FIRMS_W, FIT, FOUNDER, HIGH, LOW, MEMBER, PLOT,
    PLOT_X, POINT, ROWS, TALL, WIDE,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::portable::{exp_neg, ln};
use crate::rng::{self, SimRng};
use crate::stats::{Series, Stats};

/// What an activated agent weighs: staying, a start-up (its choice, drawn
/// technology and hiring standard) and the firms it may join.
type Options = (Choice, Option<(Choice, Tech, f64)>, Vec<(usize, Choice)>);

/// No firm (an agent before its first production).
const NONE: u64 = u64::MAX;

/// The frame's color modes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirmsMode {
    Founder,
    Theta,
    Effort,
    Income,
}

impl std::str::FromStr for FirmsMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "founder" => Self::Founder,
            "theta" => Self::Theta,
            "effort" => Self::Effort,
            "income" => Self::Income,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub prefs: Prefs,
    pub effort: f64,
    /// The slot of the agent's firm.
    pub firm: usize,
    /// Effort and firm (its serial id) at the last production.
    pub last_effort: f64,
    pub last_firm: u64,
    pub friends: Vec<u32>,
    pub loyalty: u32,
    /// Times it wanted to move but stayed (loyalty).
    pub wants: u32,
    /// The period it joined its firm.
    pub joined: u64,
    /// Base pay (for `Pay::Base`).
    pub base: f64,
    pub income: f64,
    pub utility: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Firm {
    /// A serial number, never reused.
    pub id: u64,
    pub alive: bool,
    /// Members in order of joining; the first is the longest-serving.
    pub members: Vec<u32>,
    pub tech: Tech,
    pub hiring: f64,
    pub born: u64,
    /// Members' current efforts, summed; and at the last production.
    pub total: f64,
    pub last_total: f64,
    /// Size at the last production (0 if it had none).
    pub last_size: u32,
    /// The sum of its members' base pay.
    pub base_sum: f64,
    /// Whether it ever had a second member.
    pub team: bool,
    pub output: f64,
}

/// What Inspect shows.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FirmsInspection {
    pub site: FirmsCell,
    pub panel: Option<&'static str>,
    pub firm: Option<FirmView>,
    /// The member at the cell (called `member` so no other model's view is
    /// mistaken for it); `agent` is always null.
    pub member: Option<AgentView>,
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct FirmsCell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct FirmView {
    pub id: u64,
    pub size: u32,
    pub output: f64,
    pub age: u64,
    pub a: f64,
    pub b: f64,
    pub beta: f64,
    pub mean_theta: f64,
    pub mean_effort: f64,
    /// Members putting in no effort.
    pub free_riders: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u32,
    pub theta: f64,
    pub effort: f64,
    pub income: f64,
    pub utility: f64,
    pub tenure: u64,
    pub firm: u64,
}

#[derive(Clone)]
pub struct FirmsWorld {
    pub config: FirmsConfig,
    /// Periods run.
    pub tick: u64,
    rng: SimRng,
    agents: Vec<Agent>,
    firms: Vec<Firm>,
    free: Vec<usize>,
    /// Live firm slots, and each slot's position in it.
    live: Vec<usize>,
    pos: Vec<usize>,
    next_id: u64,
    births: u32,
    deaths: u32,
    records: Records,
    pub stats: Stats<FirmsSnapshot>,
}

/// A uniform draw between `lo` and `hi` when `hi` is above `lo`; else `lo`.
fn draw(rng: &mut SimRng, lo: f64, hi: f64) -> f64 {
    if hi > lo {
        lo + (hi - lo) * rng.gen::<f64>()
    } else {
        lo
    }
}

fn draw_u32(rng: &mut SimRng, lo: u32, hi: u32) -> u32 {
    if hi > lo {
        rng.gen_range(lo..=hi)
    } else {
        lo
    }
}

fn prefs_of(c: &FirmsConfig, rng: &mut SimRng) -> Prefs {
    let theta = match c.preferences {
        Preferences::Uniform => rng.gen::<f64>(),
        Preferences::Middle => 0.25 + 0.5 * rng.gen::<f64>(),
        Preferences::Triangular | Preferences::TriangularHigh => {
            let m = if c.preferences == Preferences::Triangular {
                0.5
            } else {
                0.75
            };
            let u: f64 = rng.gen();
            if u < m {
                (u * m).sqrt()
            } else {
                1.0 - ((1.0 - u) * (1.0 - m)).sqrt()
            }
        }
        Preferences::Normal => loop {
            // Rejection from U[0, 1] against the normal (0.5, variance ½).
            let (x, u): (f64, f64) = (rng.gen(), rng.gen());
            if u < exp_neg(-(x - 0.5) * (x - 0.5)) {
                break x;
            }
        },
        Preferences::Beta => 1.0 - (1.0 - rng.gen::<f64>()).sqrt(),
        Preferences::Fixed => c.theta,
        Preferences::Ces => {
            let delta = rng.gen::<f64>();
            let rho = draw(rng, c.rho, c.rho_max);
            return Prefs::Ces {
                delta,
                rho,
                minus: c.ces_sign == CesSign::Text,
            };
        }
    };
    Prefs::CobbDouglas { theta }
}

impl FirmsWorld {
    pub fn new(config: FirmsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let n = config.agents as usize;
        let base_tech = Tech {
            a: config.a,
            b: config.b,
            beta: config.beta,
        };
        let search = search_of(&config);
        let alone = |p: &Prefs| {
            best(
                p,
                &Choice {
                    tech: base_tech,
                    others: 0.0,
                    share: Share::Fraction(1.0),
                },
                0.0,
                1.0,
                search,
            )
        };
        let mut agents: Vec<Agent> = (0..n)
            .map(|_| {
                let prefs = prefs_of(&config, &mut rng);
                Agent {
                    prefs,
                    effort: 0.0,
                    firm: 0,
                    last_effort: 0.0,
                    last_firm: NONE,
                    friends: Vec::new(),
                    loyalty: 0,
                    wants: 0,
                    joined: 0,
                    base: 0.0,
                    income: 0.0,
                    utility: 0.0,
                }
            })
            .collect();
        for (i, agent) in agents.iter_mut().enumerate() {
            let k = draw_u32(&mut rng, config.neighbors, config.neighbors_max).min(n as u32 - 1);
            let mut friends = Vec::with_capacity(k as usize);
            while friends.len() < k as usize {
                let j = rng.gen_range(0..n as u32);
                if j as usize != i && !friends.contains(&j) {
                    friends.push(j);
                }
            }
            agent.friends = friends;
            agent.loyalty = draw_u32(&mut rng, config.loyalty, config.loyalty_max);
        }
        // Base pay: singleton income, own or the median or mean agent's.
        let singleton: Vec<f64> = agents
            .iter()
            .map(|a| base_tech.output(alone(&a.prefs).0))
            .collect();
        let median = base_tech.output(alone(&Prefs::CobbDouglas { theta: 0.5 }).0);
        let mean = singleton.iter().sum::<f64>() / n as f64;
        for (a, own) in agents.iter_mut().zip(&singleton) {
            a.base = config.base_share
                * match config.base_pay {
                    BasePay::Own => *own,
                    BasePay::Median => median,
                    BasePay::Mean => mean,
                };
        }
        let mut world = FirmsWorld {
            config,
            tick: 0,
            rng,
            agents,
            firms: Vec::new(),
            free: Vec::new(),
            live: Vec::new(),
            pos: Vec::new(),
            next_id: 0,
            births: 0,
            deaths: 0,
            records: Records::default(),
            stats: Stats::default(),
        };
        // The starting firms.
        let mut order: Vec<u32> = (0..n as u32).collect();
        match world.config.initial {
            Initial::Alone => {
                for i in 0..n {
                    let f = world.found();
                    world.enter(i, f);
                }
            }
            Initial::OneFirm => {
                let f = world.found();
                for i in 0..n {
                    world.enter(i, f);
                }
            }
            Initial::RandomGroups => {
                shuffle(&mut world.rng, &mut order);
                let mut k = 0;
                while k < n {
                    let mut size = 1;
                    while world.rng.gen::<f64>() >= 0.25 {
                        size += 1;
                    }
                    let f = world.found();
                    for &i in order[k..(k + size).min(n)].iter() {
                        world.enter(i as usize, f);
                    }
                    k += size;
                }
            }
        }
        for i in 0..n {
            let e = alone(&world.agents[i].prefs).0;
            world.set_effort(i, e);
        }
        world.births = 0;
        world.produce();
        world.record();
        Ok(world)
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn records(&self) -> &Records {
        &self.records
    }

    /// Live firms, in order of founding.
    pub fn firms(&self) -> Vec<&Firm> {
        let mut v: Vec<&Firm> = self.live.iter().map(|&f| &self.firms[f]).collect();
        v.sort_by_key(|f| (f.born, f.id));
        v
    }

    pub fn is_finished(&self) -> bool {
        self.config.stop_at > 0 && self.tick >= u64::from(self.config.stop_at)
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// A new, empty firm with its technology and hiring standard drawn.
    fn found(&mut self) -> usize {
        let c = &self.config;
        let tech = Tech {
            a: draw(&mut self.rng, c.a, c.a_max),
            b: draw(&mut self.rng, c.b, c.b_max),
            beta: draw(&mut self.rng, c.beta, c.beta_max),
        };
        let hiring = draw(&mut self.rng, c.hiring, c.hiring_max);
        self.open(tech, hiring)
    }

    fn open(&mut self, tech: Tech, hiring: f64) -> usize {
        let firm = Firm {
            id: self.next_id,
            alive: true,
            members: Vec::new(),
            tech,
            hiring,
            born: self.tick,
            total: 0.0,
            last_total: 0.0,
            last_size: 0,
            base_sum: 0.0,
            team: false,
            output: 0.0,
        };
        self.next_id += 1;
        self.births += 1;
        let slot = if let Some(s) = self.free.pop() {
            self.firms[s] = firm;
            s
        } else {
            self.firms.push(firm);
            self.pos.push(0);
            self.firms.len() - 1
        };
        self.pos[slot] = self.live.len();
        self.live.push(slot);
        slot
    }

    fn enter(&mut self, i: usize, f: usize) {
        let firm = &mut self.firms[f];
        firm.members.push(i as u32);
        firm.total += self.agents[i].effort;
        firm.base_sum += self.agents[i].base;
        if firm.members.len() >= 2 {
            firm.team = true;
        }
        self.agents[i].firm = f;
        self.agents[i].joined = self.tick;
    }

    fn leave(&mut self, i: usize) {
        let f = self.agents[i].firm;
        let firm = &mut self.firms[f];
        firm.members.retain(|&m| m as usize != i);
        firm.total -= self.agents[i].effort;
        firm.base_sum -= self.agents[i].base;
        if firm.members.is_empty() {
            firm.alive = false;
            firm.total = 0.0;
            let (lifetime, team) = (self.tick.saturating_sub(firm.born), firm.team);
            if self.tick > u64::from(self.config.burn_in) {
                self.records.died(lifetime, team);
            }
            self.deaths += 1;
            // Out of the live list.
            let p = self.pos[f];
            let last = *self.live.last().unwrap();
            self.live.swap_remove(p);
            if last != f {
                self.pos[last] = p;
            }
            self.free.push(f);
        }
    }

    fn set_effort(&mut self, i: usize, e: f64) {
        let f = self.agents[i].firm;
        self.firms[f].total += e - self.agents[i].effort;
        self.agents[i].effort = e;
    }

    /// The others' effort an agent sees in firm `f` (its own excluded if a
    /// member).
    fn others(&self, i: usize, f: usize) -> f64 {
        let firm = &self.firms[f];
        let member = self.agents[i].firm == f;
        match self.config.others_effort {
            OthersEffort::Live => {
                if member {
                    (firm.total - self.agents[i].effort).max(0.0)
                } else {
                    firm.total.max(0.0)
                }
            }
            OthersEffort::LastPeriod => {
                let own = if member && self.agents[i].last_firm == firm.id {
                    self.agents[i].last_effort
                } else {
                    0.0
                };
                (firm.last_total - own).max(0.0)
            }
        }
    }

    /// Agent i's share in firm `f` of `n` members at seniority `rank` (1 is
    /// the longest-serving).
    fn share(&self, i: usize, n: usize, rank: usize, base_others: f64) -> Share {
        match self.config.pay {
            Pay::Equal => Share::Fraction(1.0 / n as f64),
            Pay::Seniority => {
                let p = self.config.seniority_base;
                let w = |r: usize| 1.0 / crate::firms::effort::powf(p, r as f64);
                let total: f64 = (1..=n).map(w).sum();
                let r = match self.config.seniority_order {
                    SeniorityOrder::SeniorFirst => rank,
                    SeniorityOrder::JuniorFirst => n + 1 - rank,
                };
                Share::Fraction(w(r) / total)
            }
            Pay::Base => Share::Base {
                own: self.agents[i].base,
                others: base_others,
                n: n as f64,
            },
        }
    }

    /// The options agent i weighs: its own firm, a start-up (technology and
    /// hiring standard drawn now), and the firms its network shows it that
    /// admit it. (None for the start-up if it is alone: starting again is
    /// staying.)
    fn options(&mut self, i: usize) -> Options {
        let own = self.agents[i].firm;
        let n_own = self.firms[own].members.len();
        let rank = self.firms[own]
            .members
            .iter()
            .position(|&m| m as usize == i)
            .unwrap()
            + 1;
        let stay = Choice {
            tech: self.firms[own].tech,
            others: self.others(i, own),
            share: self.share(
                i,
                n_own,
                rank,
                self.firms[own].base_sum - self.agents[i].base,
            ),
        };
        let start = if n_own > 1 {
            let c = &self.config;
            let tech = Tech {
                a: draw(&mut self.rng, c.a, c.a_max),
                b: draw(&mut self.rng, c.b, c.b_max),
                beta: draw(&mut self.rng, c.beta, c.beta_max),
            };
            let hiring = draw(&mut self.rng, c.hiring, c.hiring_max);
            Some((
                Choice {
                    tech,
                    others: 0.0,
                    share: self.share(i, 1, 1, 0.0),
                },
                tech,
                hiring,
            ))
        } else {
            None
        };
        let mut targets: Vec<usize> = Vec::new();
        match self.config.network {
            Network::Friends => {
                for &j in &self.agents[i].friends {
                    let f = self.agents[j as usize].firm;
                    if f != own && !targets.contains(&f) {
                        targets.push(f);
                    }
                }
            }
            Network::RandomFirms => {
                let want = self.agents[i].friends.len();
                let others = self.live.len() - 1;
                if others <= want {
                    targets.extend(self.live.iter().copied().filter(|&f| f != own));
                } else {
                    while targets.len() < want {
                        let f = self.live[self.rng.gen_range(0..self.live.len() as u32) as usize];
                        if f != own && !targets.contains(&f) {
                            targets.push(f);
                        }
                    }
                }
            }
        }
        let joins = targets
            .into_iter()
            .filter(|&f| self.admits(f, i))
            .map(|f| {
                let n = self.firms[f].members.len() + 1;
                (
                    f,
                    Choice {
                        tech: self.firms[f].tech,
                        others: self.others(i, f),
                        share: self.share(i, n, n, self.firms[f].base_sum),
                    },
                )
            })
            .collect();
        (stay, start, joins)
    }

    /// Whether firm `f` admits agent i: its θ at least φ times that of the
    /// firm's longest-serving member (A99 §4.9).
    pub fn admits(&self, f: usize, i: usize) -> bool {
        let firm = &self.firms[f];
        let senior = self.agents[firm.members[0] as usize].prefs.weight();
        self.agents[i].prefs.weight() >= firm.hiring * senior
    }

    fn activate(&mut self, i: usize) {
        let prefs = self.agents[i].prefs;
        let current = self.agents[i].effort;
        let search = search_of(&self.config);
        let (lo, hi) = if self.config.effort_window >= 1.0 {
            (0.0, 1.0)
        } else {
            let half = 0.5 * self.config.effort_window;
            (current - half, current + half)
        };
        // Sticky effort's window for moves too, or only at home.
        let (mlo, mhi) = if self.config.adjust_scope == AdjustScope::Everywhere {
            (lo, hi)
        } else {
            (0.0, 1.0)
        };
        let (stay, start, joins) = self.options(i);
        // The best of the options: 0 stay, 1 start, 2.. join.
        let mut pick: (usize, f64, f64);
        match self.config.random_behavior {
            RandomBehavior::Choices => {
                let which = self.rng.gen_range(0..3u32);
                pick = match which {
                    1 if start.is_some() => {
                        let (e, u) = best(&prefs, &start.unwrap().0, lo, hi, search);
                        (1, e, u)
                    }
                    2 if !joins.is_empty() => {
                        let k = self.rng.gen_range(0..joins.len() as u32) as usize;
                        let (e, u) = best(&prefs, &joins[k].1, lo, hi, search);
                        (2 + k, e, u)
                    }
                    _ => {
                        let (e, u) = best(&prefs, &stay, lo, hi, search);
                        (0, e, u)
                    }
                };
                self.apply(i, pick.0, pick.1, start, &joins, true);
                return;
            }
            RandomBehavior::Effort => {
                let e: f64 = self.rng.gen();
                pick = (0, e, stay.utility(&prefs, e));
                if let Some((c, _, _)) = &start {
                    let u = c.utility(&prefs, e);
                    if u > pick.2 {
                        pick = (1, e, u);
                    }
                }
                for (k, (_, c)) in joins.iter().enumerate() {
                    let u = c.utility(&prefs, e);
                    if u > pick.2 {
                        pick = (2 + k, e, u);
                    }
                }
            }
            RandomBehavior::None if self.config.groping => {
                // One random try at a new effort here; the other options at
                // the current effort (our reading of §4.6).
                let trial: f64 = self.rng.gen();
                let (uc, ut) = (stay.utility(&prefs, current), stay.utility(&prefs, trial));
                pick = if ut > uc {
                    (0, trial, ut)
                } else {
                    (0, current, uc)
                };
                // Elsewhere: at the current effort, or (own firm only) the best.
                let elsewhere = |c: &Choice| {
                    if self.config.adjust_scope == AdjustScope::Everywhere {
                        (current, c.utility(&prefs, current))
                    } else {
                        best(&prefs, c, 0.0, 1.0, search)
                    }
                };
                if let Some((c, _, _)) = &start {
                    let (e, u) = elsewhere(c);
                    if u > pick.2 {
                        pick = (1, e, u);
                    }
                }
                for (k, (_, c)) in joins.iter().enumerate() {
                    let (e, u) = elsewhere(c);
                    if u > pick.2 {
                        pick = (2 + k, e, u);
                    }
                }
            }
            RandomBehavior::None => {
                let (e, u) = best(&prefs, &stay, lo, hi, search);
                pick = (0, e, u);
                if let Some((c, _, _)) = &start {
                    let (e, u) = best(&prefs, c, mlo, mhi, search);
                    if u > pick.2 {
                        pick = (1, e, u);
                    }
                }
                for (k, (_, c)) in joins.iter().enumerate() {
                    let (e, u) = best(&prefs, c, mlo, mhi, search);
                    if u > pick.2 {
                        pick = (2 + k, e, u);
                    }
                }
            }
        }
        // Loyalty: stay (re-choosing effort here) until wanting to move more
        // than λ times.
        if pick.0 != 0 && self.agents[i].wants < self.agents[i].loyalty {
            self.agents[i].wants += 1;
            let (e, _) = if self.config.groping {
                (current, 0.0)
            } else {
                best(&prefs, &stay, lo, hi, search)
            };
            pick = (0, e, 0.0);
        }
        self.apply(i, pick.0, pick.1, start, &joins, false);
    }

    fn apply(
        &mut self,
        i: usize,
        which: usize,
        e: f64,
        start: Option<(Choice, Tech, f64)>,
        joins: &[(usize, Choice)],
        _random: bool,
    ) {
        match which {
            0 => self.set_effort(i, e),
            1 => {
                let (_, tech, hiring) = start.unwrap();
                self.leave(i);
                let f = self.open(tech, hiring);
                self.agents[i].effort = e;
                self.agents[i].wants = 0;
                self.enter(i, f);
            }
            k => {
                let f = joins[k - 2].0;
                self.leave(i);
                self.agents[i].effort = e;
                self.agents[i].wants = 0;
                self.enter(i, f);
            }
        }
    }

    pub fn step(&mut self) {
        self.tick += 1;
        self.births = 0;
        self.deaths = 0;
        let n = self.agents.len();
        let count = ((self.config.activation_rate * n as f64).round() as usize).max(1);
        match self.config.activation {
            Activation::Random => {
                for _ in 0..count {
                    let i = self.rng.gen_range(0..n as u32) as usize;
                    self.activate(i);
                }
            }
            Activation::Uniform => {
                let mut order: Vec<u32> = (0..n as u32).collect();
                let mut done = 0;
                while done < count {
                    shuffle(&mut self.rng, &mut order);
                    for &i in order.iter().take(count - done) {
                        self.activate(i as usize);
                    }
                    done += n.min(count - done);
                }
            }
        }
        self.produce();
        self.record();
    }

    /// Every firm produces and pays; the records take the period.
    fn produce(&mut self) {
        let sampling = self.tick > u64::from(self.config.burn_in);
        let sample = sampling
            && self
                .tick
                .is_multiple_of(u64::from(self.config.sample_every));
        let mut sizes: Vec<(u32, f64)> = Vec::new();
        for k in 0..self.live.len() {
            let f = self.live[k];
            let members = self.firms[f].members.clone();
            let total: f64 = members
                .iter()
                .map(|&m| self.agents[m as usize].effort)
                .sum();
            let tech = self.firms[f].tech;
            let output = tech.output(total);
            let n = members.len();
            let id = self.firms[f].id;
            let base_sum = self.firms[f].base_sum;
            for (r, &m) in members.iter().enumerate() {
                let m = m as usize;
                let share = self.share(m, n, r + 1, base_sum - self.agents[m].base);
                let income = share.income(output);
                let a = &mut self.agents[m];
                a.income = income;
                a.utility = a.prefs.utility(income, 1.0 - a.effort);
                a.last_effort = a.effort;
                a.last_firm = id;
            }
            let firm = &mut self.firms[f];
            if sampling && firm.last_size > 0 {
                let before = firm.last_size;
                self.records.grow(before, n as u32);
            }
            let firm = &mut self.firms[f];
            firm.total = total;
            firm.last_total = total;
            firm.last_size = n as u32;
            firm.output = output;
            if sample {
                sizes.push((n as u32, output));
            }
        }
        if sample {
            self.records.sample(&sizes);
        }
    }

    fn record(&mut self) {
        let n = self.agents.len() as f64;
        let firms = self.live.len();
        let largest = self
            .live
            .iter()
            .map(|&f| self.firms[f].members.len())
            .max()
            .unwrap_or(0);
        let singletons = self
            .live
            .iter()
            .filter(|&&f| self.firms[f].members.len() == 1)
            .count();
        let output: f64 = self.live.iter().map(|&f| self.firms[f].output).sum();
        let top = self
            .live
            .iter()
            .map(|&f| &self.firms[f])
            .max_by_key(|f| (f.members.len(), std::cmp::Reverse(f.id)));
        let (_, mean, _, _) = super::fit::lifetimes(&self.records.lifetimes);
        self.stats.push(FirmsSnapshot {
            tick: self.tick,
            firms: firms as u32,
            births: self.births,
            deaths: self.deaths,
            mean_size: n / firms.max(1) as f64,
            largest: largest as u32,
            singletons: singletons as f64 / firms.max(1) as f64,
            effort: self.agents.iter().map(|a| a.effort).sum::<f64>() / n,
            output,
            income: self.agents.iter().map(|a| a.income).sum::<f64>() / n,
            utility: self.agents.iter().map(|a| a.utility).sum::<f64>() / n,
            largest_output_share: top.map_or(f64::NAN, |f| {
                if output > 0.0 {
                    f.output / output
                } else {
                    f64::NAN
                }
            }),
            mu: mu_ols(&self.records.sizes),
            // The exact maximum likelihood is costlier: every 10 periods.
            mu_mle: if self.tick.is_multiple_of(10) {
                mu_mle(&self.records.sizes, 2)
            } else {
                self.stats.latest().map_or(f64::NAN, |s| s.mu_mle)
            },
            lifetime: mean,
            period: self.tick,
        });
    }

    fn firm_view(&self, f: usize) -> FirmView {
        let firm = &self.firms[f];
        let n = firm.members.len().max(1) as f64;
        FirmView {
            id: firm.id,
            size: firm.members.len() as u32,
            output: firm.output,
            age: self.tick - firm.born,
            a: firm.tech.a,
            b: firm.tech.b,
            beta: firm.tech.beta,
            mean_theta: firm
                .members
                .iter()
                .map(|&m| self.agents[m as usize].prefs.weight())
                .sum::<f64>()
                / n,
            mean_effort: firm
                .members
                .iter()
                .map(|&m| self.agents[m as usize].effort)
                .sum::<f64>()
                / n,
            free_riders: firm
                .members
                .iter()
                .filter(|&&m| self.agents[m as usize].effort <= 1e-9)
                .count() as u32,
        }
    }

    fn agent_view(&self, i: usize) -> AgentView {
        let a = &self.agents[i];
        AgentView {
            id: i as u32 + 1,
            theta: a.prefs.weight(),
            effort: a.effort,
            income: a.income,
            utility: a.utility,
            tenure: self.tick - a.joined,
            firm: self.firms[a.firm].id,
        }
    }

    /// The rows the frame shows: the `ROWS` largest firms, oldest first
    /// among equals.
    fn rows(&self) -> Vec<usize> {
        let mut v: Vec<usize> = self.live.clone();
        v.sort_by_key(|&f| {
            (
                std::cmp::Reverse(self.firms[f].members.len()),
                self.firms[f].born,
                self.firms[f].id,
            )
        });
        v.truncate(ROWS);
        v
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<FirmsInspection, String> {
        if x as usize >= WIDE || y as usize >= TALL {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let mut out = FirmsInspection {
            site: FirmsCell { x, y },
            panel: None,
            firm: None,
            member: None,
            agent: None,
        };
        if (x as usize) < FIRMS_W && (y as usize) < FIRMS_H {
            out.panel = Some("firms");
            let rows = self.rows();
            if let Some(&f) = rows.get(y as usize / CELL) {
                out.firm = Some(self.firm_view(f));
                if let Some(&m) = self.firms[f].members.get(x as usize / CELL) {
                    out.member = Some(self.agent_view(m as usize));
                }
            }
        } else if x as usize >= PLOT_X {
            out.panel = Some("sizes");
        }
        Ok(out)
    }
}

fn search_of(c: &FirmsConfig) -> Search {
    match c.effort_search {
        EffortSearch::Exact => Search::Exact,
        EffortSearch::Grid => Search::Grid(c.grid_steps),
    }
}

/// Fisher–Yates with `u32` draws.
fn shuffle(rng: &mut SimRng, v: &mut [u32]) {
    for k in (1..v.len()).rev() {
        let j = rng.gen_range(0..=k as u32) as usize;
        v.swap(k, j);
    }
}

fn line(c: &mut Canvas, a: (usize, usize), b: (usize, usize), color: [u8; 3]) {
    let (mut x0, mut y0, x1, y1) = (a.0 as i64, a.1 as i64, b.0 as i64, b.1 as i64);
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let mut err = dx + dy;
    loop {
        if (0..WIDE as i64).contains(&x0) && (0..TALL as i64).contains(&y0) {
            c.put(x0 as usize, y0 as usize, color);
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

impl Model for FirmsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Firms(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        FirmsWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.agents.len()
    }

    /// FNV-1a over the tick and every agent's firm, effort and income.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        for a in &self.agents {
            eat(&self.firms[a.firm].id.to_le_bytes());
            eat(&a.effort.to_bits().to_le_bytes());
            eat(&a.income.to_bits().to_le_bytes());
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (WIDE as u32, TALL as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: FirmsMode = mode.parse()?;
        let mut c = Canvas { buf, wide: 0 };
        c.clear(WIDE, TALL);
        for (row, &f) in self.rows().iter().enumerate() {
            for (col, &m) in self.firms[f].members.iter().take(COLS).enumerate() {
                let a = &self.agents[m as usize];
                let color = match mode {
                    FirmsMode::Founder => {
                        if col == 0 {
                            FOUNDER
                        } else {
                            MEMBER
                        }
                    }
                    FirmsMode::Theta => scale(a.prefs.weight(), LOW, HIGH),
                    FirmsMode::Effort => scale(a.effort, LOW, HIGH),
                    FirmsMode::Income => scale(a.income / 4.0, LOW, HIGH),
                };
                for dy in 0..CELL - 1 {
                    for dx in 0..CELL - 1 {
                        c.put(col * CELL + dx, row * CELL + dy, color);
                    }
                }
            }
        }
        // The size distribution: axes, points and the OLS line.
        line(
            &mut c,
            (PLOT_X, PLOT - 1),
            (PLOT_X + PLOT - 1, PLOT - 1),
            AXIS,
        );
        line(&mut c, (PLOT_X, 0), (PLOT_X, PLOT - 1), AXIS);
        let sizes = &self.records.sizes;
        let total: u64 = sizes.iter().sum();
        if total > 0 {
            let mut fitted = Vec::new();
            for (s, &k) in sizes.iter().enumerate().skip(1) {
                if k == 0 {
                    continue;
                }
                let (x, y) = (ln(s as f64), ln(k as f64 / total as f64));
                if let Some((px, py)) = plot_at(x, y) {
                    c.put(px, py, POINT);
                    if py + 1 < PLOT {
                        c.put(px, py + 1, POINT);
                    }
                }
                if s >= 2 && k as f64 / total as f64 >= 1e-5 {
                    fitted.push((x, y));
                }
            }
            if let Some((slope, icept)) = ols(&fitted) {
                let ends: Vec<(usize, usize)> = [ln(2.0), fitted.last().map_or(0.0, |p| p.0)]
                    .iter()
                    .filter_map(|&x| plot_at(x, icept + slope * x))
                    .collect();
                if ends.len() == 2 {
                    line(&mut c, ends[0], ends[1], FIT);
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
        let mut out = String::from("id,theta,effort,income,utility,firm,firm_size,tenure\n");
        for (i, a) in self.agents.iter().enumerate() {
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                i + 1,
                a.prefs.weight(),
                a.effort,
                a.income,
                a.utility,
                self.firms[a.firm].id,
                self.firms[a.firm].members.len(),
                self.tick - a.joined
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    /// An agent's cell in the firm rows (None if its firm is off the frame).
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let i = usize::try_from(id.checked_sub(1)?).ok()?;
        let a = self.agents.get(i)?;
        let row = self.rows().iter().position(|&f| f == a.firm)?;
        let col = self.firms[a.firm]
            .members
            .iter()
            .position(|&m| m as usize == i)?;
        (col < COLS).then(|| ((col * CELL) as u32, (row * CELL) as u32))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Firms(next) = next else {
            return Err(wrong_model(ModelKind::Firms, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        // Base pay follows its settings at once.
        if (next.base_pay, next.base_share) != (self.config.base_pay, self.config.base_share) {
            let tech = Tech {
                a: next.a,
                b: next.b,
                beta: next.beta,
            };
            let search = search_of(&next);
            let single = |p: &Prefs| {
                tech.output(
                    best(
                        p,
                        &Choice {
                            tech,
                            others: 0.0,
                            share: Share::Fraction(1.0),
                        },
                        0.0,
                        1.0,
                        search,
                    )
                    .0,
                )
            };
            let own: Vec<f64> = self.agents.iter().map(|a| single(&a.prefs)).collect();
            let (median, mean) = (
                single(&Prefs::CobbDouglas { theta: 0.5 }),
                own.iter().sum::<f64>() / own.len() as f64,
            );
            for (a, o) in self.agents.iter_mut().zip(&own) {
                a.base = next.base_share
                    * match next.base_pay {
                        BasePay::Own => *o,
                        BasePay::Median => median,
                        BasePay::Mean => mean,
                    };
            }
            for &f in &self.live {
                self.firms[f].base_sum = self.firms[f]
                    .members
                    .iter()
                    .map(|&m| self.agents[m as usize].base)
                    .sum();
            }
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// Stopped after its last period: a sweep reads it there.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(edit: impl FnOnce(&mut FirmsConfig)) -> FirmsWorld {
        let mut c = FirmsConfig {
            agents: 200,
            burn_in: 20,
            stop_at: 200,
            ..FirmsConfig::default()
        };
        edit(&mut c);
        FirmsWorld::new(c, 1).unwrap()
    }

    fn check_invariants(w: &FirmsWorld) {
        let mut seen = vec![0; w.agents.len()];
        for &f in &w.live {
            let firm = &w.firms[f];
            assert!(firm.alive && !firm.members.is_empty());
            let total: f64 = firm
                .members
                .iter()
                .map(|&m| w.agents[m as usize].effort)
                .sum();
            assert!(
                (total - firm.total).abs() < 1e-9,
                "firm {} total drift",
                firm.id
            );
            for &m in &firm.members {
                assert_eq!(w.agents[m as usize].firm, f);
                seen[m as usize] += 1;
            }
        }
        assert!(
            seen.iter().all(|&k| k == 1),
            "every agent in exactly one live firm"
        );
    }

    #[test]
    fn everyone_starts_alone_at_their_optimum() {
        let w = world(|_| {});
        assert_eq!(w.live.len(), 200);
        check_invariants(&w);
        let s = w.stats.latest().unwrap();
        // A99's singletons: mean output per agent ≈ 0.934 over θ ~ U[0, 1].
        assert!((s.output / 200.0 - 0.934).abs() < 0.08, "{}", s.output);
        assert_eq!((s.firms, s.largest), (200, 1));
    }

    #[test]
    fn firms_form_grow_and_the_books_balance() {
        let mut w = world(|_| {});
        w.run(200);
        check_invariants(&w);
        let s = w.stats.latest().unwrap();
        assert!(s.largest >= 5 && s.firms < 200, "{s:?}");
        assert!(s.mu.is_finite() && s.lifetime.is_finite());
        // Births and deaths balance the firm count.
        let hist = w.stats.history();
        for pair in hist.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            assert_eq!(
                i64::from(b.firms),
                i64::from(a.firms) + i64::from(b.births) - i64::from(b.deaths)
            );
        }
    }

    #[test]
    fn under_constant_returns_even_identical_agents_join_myopically() {
        // A99 fn 19: under constant returns working together is never
        // individually rational — at equilibrium. A myopic joiner takes the
        // other's effort as fixed: alone at e = θ, joining at θ² gains the
        // factor (1 + θ)/2^θ > 1 (≈ 1.04 at θ = 0.75), so firms still form.
        let theta: f64 = 0.75;
        assert!((1.0 + theta) / 2f64.powf(theta) > 1.0);
        let mut w = world(|c| {
            c.b = 0.0;
            c.preferences = Preferences::Fixed;
            c.theta = theta;
        });
        w.run(20);
        check_invariants(&w);
        assert!(w.stats.latest().unwrap().largest >= 2);
    }

    #[test]
    fn others_effort_readings_differ() {
        let mut a = world(|_| {});
        let mut b = world(|c| c.others_effort = OthersEffort::Live);
        a.run(50);
        b.run(50);
        assert_ne!(Model::fingerprint(&a), Model::fingerprint(&b));
    }

    #[test]
    fn loyal_agents_stay_until_their_count_runs_out() {
        let mut w = world(|c| {
            c.loyalty = 1000;
        });
        w.run(10);
        // Nobody may move before wanting to 1000 times: no firm grows.
        assert_eq!(w.stats.latest().unwrap().largest, 1);
    }

    #[test]
    fn a_hiring_standard_admits_only_those_near_the_senior_member() {
        let w = world(|c| c.hiring = 0.5);
        let weight = |i: usize| w.agents[i].prefs.weight();
        // Everyone starts alone: firm slot i holds agent i, its founder.
        let (hi, lo) = (0..200).fold((0, 0), |(h, l), i| {
            (
                if weight(i) > weight(h) { i } else { h },
                if weight(i) < weight(l) { i } else { l },
            )
        });
        assert!(
            !w.admits(w.agents[hi].firm, lo),
            "θ {} under half of {}",
            weight(lo),
            weight(hi)
        );
        assert!(w.admits(w.agents[lo].firm, hi));
        let open = world(|_| {});
        assert!(open.admits(open.agents[hi].firm, lo));
    }

    #[test]
    fn sticky_effort_moves_at_most_half_its_window() {
        // Uniform activation: each agent at most once a period.
        let mut w = world(|c| {
            c.effort_window = 0.1;
            c.activation = Activation::Uniform;
        });
        let before: Vec<f64> = w.agents.iter().map(|a| a.effort).collect();
        w.step();
        for (a, e) in w.agents.iter().zip(before) {
            assert!((a.effort - e).abs() <= 0.05 + 1e-12, "{} → {}", e, a.effort);
        }
    }

    #[test]
    fn sticky_effort_in_the_own_firm_only_leaves_moves_free() {
        use super::super::config::AdjustScope;
        // Uniform activation, window ±0.05, but only for effort at home: an
        // agent that moved may change effort by more.
        let mut w = world(|c| {
            c.effort_window = 0.1;
            c.activation = Activation::Uniform;
            c.adjust_scope = AdjustScope::OwnFirm;
        });
        let mut free_move = false;
        for _ in 0..20 {
            let before: Vec<(f64, u64)> = w
                .agents
                .iter()
                .map(|a| (a.effort, w.firms[a.firm].id))
                .collect();
            w.step();
            for (a, (e, f)) in w.agents.iter().zip(before) {
                if w.firms[a.firm].id == f {
                    assert!((a.effort - e).abs() <= 0.05 + 1e-12);
                } else if (a.effort - e).abs() > 0.05 {
                    free_move = true;
                }
            }
        }
        assert!(
            free_move,
            "some mover changed effort by more than the window"
        );
    }

    #[test]
    fn junior_first_seniority_pays_the_newest_most() {
        use super::super::config::SeniorityOrder;
        let mut w = world(|c| {
            c.pay = Pay::Seniority;
            c.seniority_order = SeniorityOrder::JuniorFirst;
        });
        w.run(100);
        let f = w
            .live
            .iter()
            .copied()
            .find(|&f| w.firms[f].members.len() >= 3)
            .expect("a firm of three");
        let incomes: Vec<f64> = w.firms[f]
            .members
            .iter()
            .map(|&m| w.agents[m as usize].income)
            .collect();
        assert!(
            incomes.windows(2).all(|p| p[0] <= p[1] + 1e-12),
            "{incomes:?}"
        );
    }

    #[test]
    fn seniority_shares_sum_to_output_and_fall_with_rank() {
        let mut w = world(|c| {
            c.pay = Pay::Seniority;
            c.seniority_base = 2.0;
        });
        w.run(100);
        for &f in &w.live {
            let firm = &w.firms[f];
            let paid: f64 = firm
                .members
                .iter()
                .map(|&m| w.agents[m as usize].income)
                .sum();
            assert!((paid - firm.output).abs() < 1e-9 * firm.output.max(1.0));
            let incomes: Vec<f64> = firm
                .members
                .iter()
                .map(|&m| w.agents[m as usize].income)
                .collect();
            assert!(incomes.windows(2).all(|p| p[0] >= p[1] - 1e-12));
        }
    }

    #[test]
    fn base_pay_is_paid_even_when_output_falls_short() {
        let mut w = world(|c| {
            c.pay = Pay::Base;
            c.base_share = 0.8;
        });
        w.run(50);
        for a in &w.agents {
            assert!(a.income >= a.base - 1e-12);
        }
    }

    #[test]
    fn the_2013_parameterization_draws_per_firm() {
        let mut w = world(|c| {
            c.a = 0.0;
            c.a_max = 0.5;
            c.b = 0.75;
            c.b_max = 1.25;
            c.beta = 1.5;
            c.beta_max = 2.0;
            c.neighbors = 2;
            c.neighbors_max = 6;
            c.activation_rate = 0.04;
        });
        w.run(100);
        check_invariants(&w);
        let techs: Vec<Tech> = w.live.iter().map(|&f| w.firms[f].tech).collect();
        assert!(techs.iter().all(|t| (0.0..=0.5).contains(&t.a)
            && (0.75..=1.25).contains(&t.b)
            && (1.5..=2.0).contains(&t.beta)));
        assert!(techs.windows(2).any(|p| p[0] != p[1]));
    }

    #[test]
    fn random_firms_uniform_activation_and_initial_groups_run() {
        for edit in [
            (|c: &mut FirmsConfig| c.network = Network::RandomFirms) as fn(&mut FirmsConfig),
            |c| c.activation = Activation::Uniform,
            |c| c.initial = Initial::RandomGroups,
            |c| c.initial = Initial::OneFirm,
            |c| c.random_behavior = RandomBehavior::Choices,
            |c| c.random_behavior = RandomBehavior::Effort,
            |c| c.groping = true,
            |c| c.effort_search = EffortSearch::Grid,
            |c| c.preferences = Preferences::Ces,
            |c| c.preferences = Preferences::Normal,
        ] {
            let mut w = world(edit);
            w.run(30);
            check_invariants(&w);
        }
    }

    #[test]
    fn lifetimes_are_counted_after_the_burn_in() {
        let mut w = world(|c| c.burn_in = 50);
        w.run(50);
        assert_eq!(w.records.lifetimes.iter().sum::<u64>(), 0);
        w.run(50);
        assert!(w.records.lifetimes.iter().sum::<u64>() > 0);
    }

    #[test]
    fn the_view_and_inspect_read_a_firm_and_its_member() {
        let mut w = world(|_| {});
        w.run(60);
        let mut buf = Vec::new();
        for mode in ["founder", "theta", "effort", "income"] {
            Model::render(&w, mode, "", &mut buf).unwrap();
        }
        assert!(Model::render(&w, "wealth", "", &mut buf).is_err());
        let i = w.inspect(0, 0).unwrap();
        assert_eq!(i.panel, Some("firms"));
        let (f, m) = (i.firm.unwrap(), i.member.unwrap());
        // The first row is the largest firm.
        assert_eq!(f.size, w.stats.latest().unwrap().largest);
        assert_eq!(m.firm, f.id);
        let (x, y) = Model::locate(&w, u64::from(m.id)).unwrap();
        assert_eq!((x, y), (0, 0));
        assert_eq!(
            w.inspect(PLOT_X as u32 + 5, 5).unwrap().panel,
            Some("sizes")
        );
        assert!(w.inspect(WIDE as u32, 0).is_err());
    }

    #[test]
    fn live_edits_apply_and_the_population_waits_for_reset() {
        let mut w = world(|_| {});
        let mut next = w.config.clone();
        next.beta = 1.8;
        next.pay = Pay::Base;
        Model::set_config(&mut w, ModelConfig::Firms(next.clone())).unwrap();
        assert!(w.agents.iter().all(|a| a.base > 0.0));
        next.agents = 300;
        assert!(Model::set_config(&mut w, ModelConfig::Firms(next)).is_err());
    }
}
````

Create `crates/sugarscape-core/src/firms/presets.rs` with exactly this content:

````rust
//! Axtell's base case, its readings, a preset for each §4 variation at its
//! most striking setting, and the 2013 parameterization.

use super::config::{
    Activation, FirmsConfig, Network, OthersEffort, Pay, Preferences, RandomBehavior,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const A99: &str = "Axtell 1999, Brookings CSED WP 3";
const A13: &str = "Axtell 2013, Endogenous Dynamics of Firms and Labor";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut FirmsConfig),
) -> ModelPreset {
    let mut c = FirmsConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Firms(c),
    }
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset("firms-base", "Base case", A99, "Axtell's base case (his Table 2): 1 000 agents with preferences θ ~ U[0, 1] between income and leisure, each with 2 random friends; a firm makes a·E + b·E² (a = b = 1) from its members' total effort and shares it equally; 1 000 random activations a period, each agent weighing its own firm, starting alone and its friends' firms at its best effort, others' effort read from last period's output (the text's reading); everyone starts alone. Measured (5 seeds × 5 000 periods, after 500): about 435 firms, mean size 2.3, the largest firm reaching 62–139, µ 2.29–2.79 by the paper's OLS (1.38–1.50 by maximum likelihood) against Axtell's 1.28, and a mean firm lifetime of 3.9 periods against his 23.4.", |_| {}),
        preset("firms-live", "Base case, others' effort live", A99, "The base case with others' effort read live (their current efforts), not from last period's output. Measured (5 seeds × 5 000 periods, after 500): about 374 firms, the largest reaching 195–318 (Axtell's typical run: about 205), µ 1.91–2.26 (maximum likelihood 1.25–1.38), lifetime 3.2 — the closest reading to the paper, still far from µ 1.28 and lifetimes of 23.4.", |c| c.others_effort = OthersEffort::Live),
        preset("firms-uniform", "Base case, uniform activation", A99, "The base case with each agent activated once a period, in random order (Axtell: 'essentially no difference'). Measured (5 seeds × 5 000 periods, after 500): about 466 firms, the largest 59–153, µ 2.35–2.79, lifetime 2.7 — like the base case.", |c| c.activation = Activation::Uniform),
        preset("firms-beta-17", "Weaker increasing returns (β 1.7)", A99, "The base case with weaker increasing returns, β = 1.7 (Table 3: µ 2.06). Measured (5 seeds × 5 000 periods, after 500): about 540 firms, the largest only 25–32, µ 3.55–3.81 (maximum likelihood 1.85–1.91).", |c| c.beta = 1.7),
        preset("firms-beta-21", "Stronger increasing returns (β 2.1)", A99, "The base case with stronger increasing returns, β = 2.1 (Table 3: µ 0.95). Measured (5 seeds × 5 000 periods, after 500): about 376 firms, the largest 192–394, µ 1.83–2.32 (maximum likelihood 1.20–1.36) — larger firms, as Table 3's direction says.", |c| c.beta = 2.1),
        preset("firms-b-15", "Stronger increasing returns (b 1.5)", A99, "The base case with b = 1.5 (Table 4: µ 0.53). Measured (5 seeds × 5 000 periods, after 500): about 363 firms, the largest 127–324, µ 1.88–2.39, total output about 1 000.", |c| c.b = 1.5),
        preset("firms-b-random", "Each firm draws its b", A99, "Each firm draws its b from [0.5, 1.5] at founding (Table 4: µ 0.89, 'much more like b = 1.25'). Measured (5 seeds × 5 000 periods, after 500): about 347 firms, the largest 198–397, µ 1.68–2.10.", |c| {
            c.b = 0.5;
            c.b_max = 1.5;
        }),
        preset("firms-theta-075", "Everyone alike (θ 0.75)", A99, "Everyone alike: every θ = 0.75 (Table 5: µ 0.91). Measured (5 seeds × 5 000 periods, after 500): about 91 firms of mean size 11.5, but none beyond 40 — µ 1.10–1.30 by OLS on a distribution with no tail (maximum likelihood 0.50); lifetime 7.7; mean utility 1.05, well above the base case's 0.73.", |c| {
            c.preferences = Preferences::Fixed;
            c.theta = 0.75;
        }),
        preset("firms-friends-10", "Ten friends each", A99, "Ten friends each (Table 6: µ 0.99). Measured (5 seeds × 5 000 periods, after 500): about 259 firms of mean size 3.9, the largest only 49–61, µ 2.53–2.83 (maximum likelihood 0.99–1.11) — more mid-sized firms and fewer large ones.", |c| c.neighbors = 10),
        preset("firms-random-firms-10", "Ten random firms each time", A99, "Agents look at ten random firms each time instead of friends' firms (Table 7: µ 1.03). Measured (5 seeds × 5 000 periods, after 500): about 242 firms, the largest 49–57, µ 2.67–2.94, lifetime 7.3.", |c| {
            c.network = Network::RandomFirms;
            c.neighbors = 10;
        }),
        preset("firms-loyal-10", "Loyal agents (λ 10)", A99, "Loyal agents: each moves only after wanting to 11 times (λ = 10; Table 8: µ 0.77). Measured (5 seeds × 5 000 periods, after 500): about 307 firms, the largest 94–177, µ 1.83–2.44, and firms live 34.8 periods on average — loyalty is the one change that gives the paper's long lifetimes.", |c| c.loyalty = 10),
        preset("firms-sticky", "Sticky effort (±0.05)", A99, "Sticky effort (Table 9): each new effort within ±0.05 of the agent's current one — read as applying in any firm, joined or founded too. Measured (5 seeds × 5 000 periods, after 500): the whole population ends up in one firm at times in every seed (the largest 1 000), the largest firm making 59 % of all output; free riding sets in too slowly to stop joiners. Axtell reports a milder effect (µ 0.92 against 1.28). Restricted to the agent's own firm, sticky effort behaves like the base case with somewhat larger firms (the largest 176–189 over 3 seeds; the firms-sticky sweep).", |c| c.effort_window = 0.1),
        preset("firms-groping", "Groping for effort", A99, "Groping (Table 10): one random try at a new effort, kept if it does better; other firms weighed at the current effort. Measured (5 seeds × 5 000 periods, after 500): as with sticky effort, everyone joins one firm at times in every seed (the largest firm makes 78 % of output). Restricted to the agent's own firm, groping behaves like the base case (the firms-groping sweep).", |c| c.groping = true),
        preset("firms-seniority-5", "Seniority pay (5^−rank)", A99, "Seniority pay (Table 11): shares ∝ 5^−rank, the longest-serving member first, as the text says. Measured (5 seeds × 5 000 periods, after 500): firms barely form — the largest has 4 members, µ about 10 — since a joiner's share of a pair's output is 1/31. Axtell's µ 1.11 comes neither from this reading nor from paying the newest most (µ about 0.2; the firms-seniority sweep).", |c| {
            c.pay = Pay::Seniority;
            c.seniority_base = 5.0;
        }),
        preset("firms-base-pay-80", "Base pay at 80 % of singleton income", A99, "Base pay (Table 12): each agent is paid 80 % of its own singleton income, plus an equal share of what output exceeds the base pay (Axtell: µ 0.85). Measured (5 seeds × 5 000 periods, after 500): about 307 firms, the largest 183–267, µ 1.60–1.95, lifetime 14.7 — but effort collapses to 0.10 and total output to about 290, under two fifths of the base case's.", |c| {
            c.pay = Pay::Base;
            c.base_share = 0.8;
        }),
        preset("firms-hiring-100", "Hiring only the as-hardworking", A99, "A hiring standard of 100 %: a firm admits only agents whose θ is at least its longest-serving member's (Table 13, captioned 'target output': 'not well described by a power law'). Measured (5 seeds × 5 000 periods, after 500): about 719 firms, none beyond 20–37 members.", |c| c.hiring = 1.0),
        preset("firms-random-choices", "Random choices", A99, "§4.1's random behavior: each activated agent stays, moves or starts up at random (a random friend's firm), then chooses its best effort ('firms greater than 9 or 10 are rarely observed'). Measured (5 seeds × 5 000 periods, after 500): about 692 firms, the largest 19–23, µ 3.99–4.14.", |c| c.random_behavior = RandomBehavior::Choices),
        preset("firms-2013", "Axtell's 2013 parameterization", A13, "Axtell's 2013 parameterization at 10 000 agents: each firm draws a from [0, ½], b from [¾, 5/4] and β from [3/2, 2]; 2–6 friends each; 4 % of agents activated a period (his month). Measured (5 seeds × 2 000 periods, after 500): about 1 455 firms, µ 0.98–1.05 by OLS (0.88–0.90 by maximum likelihood) — Zipf's law, as the 2013 paper reports (α ≈ 1.06) — with firms living 77 periods on average; but the largest firm grows to 3 000–5 800 agents.", |c| {
            c.agents = 10_000;
            c.a = 0.0;
            c.a_max = 0.5;
            c.b = 0.75;
            c.b_max = 1.25;
            c.beta = 1.5;
            c.beta_max = 2.0;
            c.neighbors = 2;
            c.neighbors_max = 6;
            c.activation_rate = 0.04;
            c.stop_at = 2000;
        }),
    ]
}
````

Create `crates/sugarscape-core/src/firms/mod.rs` with exactly this content:

````rust
//! The Emergence of Firms (milestone 30): Axtell, "The Emergence of Firms in a
//! Population of Agents" (Brookings CSED Working Paper 3, 1999), with his
//! 2013 parameterization ("Endogenous Dynamics of Firms and Labor with Large
//! Numbers of Simple Agents"). See
//! docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md.

mod config;
pub mod effort;
pub mod fit;
mod presets;
mod stats;
mod view;
mod world;

pub use config::{
    schema, Activation, AdjustScope, BasePay, CesSign, EffortSearch, FirmsConfig, Initial, Network,
    OthersEffort, Pay, Preferences, RandomBehavior, SeniorityOrder,
};
pub use presets::presets;
pub use stats::{FirmsSnapshot, SERIES};
pub use view::{FIRMS_H, FIRMS_W, PLOT_X, TALL, WIDE};
pub use world::{
    Agent, AgentView, Firm, FirmView, FirmsCell, FirmsInspection, FirmsMode, FirmsWorld,
};
````

- [ ] **Step 2: Wire the model kind and title its presets**

Modify `crates/sugarscape-core/src/lib.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index 386e03e..3040a95 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -19,6 +19,7 @@ pub mod edit;
 pub mod ethno;
 pub mod export;
 pub mod farol;
+pub mod firms;
 pub mod frames;
 pub mod geometry;
 pub mod graph;
````

Modify `crates/sugarscape-core/src/model.rs` — every match gains `Firms`; the reader gains its `"firms"` arm; a round-trip test pins it (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index aebb587..dc03003 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -16,6 +16,7 @@ use crate::culture::{CultureConfig, CultureWorld};
 use crate::dpd::{DpdConfig, DpdWorld};
 use crate::ethno::{EthnoConfig, EthnoWorld};
 use crate::farol::{FarolConfig, FarolWorld};
+use crate::firms::{FirmsConfig, FirmsWorld};
 use crate::image::{ImageConfig, ImageWorld};
 use crate::line::{LineConfig, LineWorld};
 use crate::norms::{NormsConfig, NormsWorld};
@@ -68,10 +69,11 @@ pub enum ModelKind {
     Bali,
     Line,
     Tipping,
+    Firms,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 25] = [
+    pub const ALL: [ModelKind; 26] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -97,6 +99,7 @@ impl ModelKind {
         ModelKind::Bali,
         ModelKind::Line,
         ModelKind::Tipping,
+        ModelKind::Firms,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -126,6 +129,7 @@ impl ModelKind {
             ModelKind::Bali => "bali",
             ModelKind::Line => "line",
             ModelKind::Tipping => "tipping",
+            ModelKind::Firms => "firms",
         }
     }
 
@@ -158,6 +162,7 @@ impl ModelKind {
             ModelKind::Bali => bali::schema(),
             ModelKind::Line => crate::line::schema(),
             ModelKind::Tipping => crate::tipping::schema(),
+            ModelKind::Firms => crate::firms::schema(),
         }
     }
 }
@@ -196,6 +201,7 @@ pub enum ModelConfig {
     Bali(BaliConfig),
     Line(LineConfig),
     Tipping(TippingConfig),
+    Firms(FirmsConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -226,6 +232,7 @@ enum Tagged<'a> {
     Bali(&'a BaliConfig),
     Line(&'a LineConfig),
     Tipping(&'a TippingConfig),
+    Firms(&'a FirmsConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -263,6 +270,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Bali(c) => Tagged::Bali(c).serialize(s),
             ModelConfig::Line(c) => Tagged::Line(c).serialize(s),
             ModelConfig::Tipping(c) => Tagged::Tipping(c).serialize(s),
+            ModelConfig::Firms(c) => Tagged::Firms(c).serialize(s),
         }
     }
 }
@@ -295,6 +303,7 @@ impl ModelConfig {
             ModelConfig::Bali(_) => ModelKind::Bali,
             ModelConfig::Line(_) => ModelKind::Line,
             ModelConfig::Tipping(_) => ModelKind::Tipping,
+            ModelConfig::Firms(_) => ModelKind::Firms,
         }
     }
 
@@ -398,6 +407,9 @@ impl ModelConfig {
             "tipping" => serde_json::from_value(value)
                 .map(ModelConfig::Tipping)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "firms" => serde_json::from_value(value)
+                .map(ModelConfig::Firms)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             "zi" => serde_json::from_value(value)
                 .map(ModelConfig::Zi)
                 .map_err(|e| FieldError::new("config", e.to_string())),
@@ -440,6 +452,7 @@ impl ModelConfig {
             ModelConfig::Bali(c) => c.validate(),
             ModelConfig::Line(c) => c.validate(),
             ModelConfig::Tipping(c) => c.validate(),
+            ModelConfig::Firms(c) => c.validate(),
         }
     }
 
@@ -472,6 +485,7 @@ impl ModelConfig {
             ModelConfig::Bali(c) => set_path(c, path, value).map(ModelConfig::Bali),
             ModelConfig::Line(c) => set_path(c, path, value).map(ModelConfig::Line),
             ModelConfig::Tipping(c) => set_path(c, path, value).map(ModelConfig::Tipping),
+            ModelConfig::Firms(c) => set_path(c, path, value).map(ModelConfig::Firms),
         }
     }
 
@@ -503,7 +517,8 @@ impl ModelConfig {
             | ModelConfig::Zi(_)
             | ModelConfig::Bali(_)
             | ModelConfig::Line(_)
-            | ModelConfig::Tipping(_) => None,
+            | ModelConfig::Tipping(_)
+            | ModelConfig::Firms(_) => None,
         }
     }
 
@@ -544,6 +559,7 @@ impl ModelConfig {
                 .iter()
                 .map(|s| s.to_string())
                 .collect(),
+            ModelConfig::Firms(_) => crate::firms::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -738,6 +754,7 @@ pub enum ModelWorld {
     Bali(Box<BaliWorld>),
     Line(Box<LineWorld>),
     Tipping(Box<TippingWorld>),
+    Firms(Box<FirmsWorld>),
 }
 
 impl ModelWorld {
@@ -794,6 +811,7 @@ impl ModelWorld {
             ModelConfig::Bali(c) => ModelWorld::Bali(Box::new(BaliWorld::new(c, seed)?)),
             ModelConfig::Line(c) => ModelWorld::Line(Box::new(LineWorld::new(c, seed)?)),
             ModelConfig::Tipping(c) => ModelWorld::Tipping(Box::new(TippingWorld::new(c, seed)?)),
+            ModelConfig::Firms(c) => ModelWorld::Firms(Box::new(FirmsWorld::new(c, seed)?)),
         })
     }
 
@@ -824,6 +842,7 @@ impl ModelWorld {
             ModelWorld::Bali(_) => ModelKind::Bali,
             ModelWorld::Line(_) => ModelKind::Line,
             ModelWorld::Tipping(_) => ModelKind::Tipping,
+            ModelWorld::Firms(_) => ModelKind::Firms,
         }
     }
 
@@ -854,6 +873,7 @@ impl ModelWorld {
             ModelWorld::Bali(w) => w.as_ref(),
             ModelWorld::Line(w) => w.as_ref(),
             ModelWorld::Tipping(w) => w.as_ref(),
+            ModelWorld::Firms(w) => w.as_ref(),
         }
     }
 
@@ -884,6 +904,7 @@ impl ModelWorld {
             ModelWorld::Bali(w) => w.as_mut(),
             ModelWorld::Line(w) => w.as_mut(),
             ModelWorld::Tipping(w) => w.as_mut(),
+            ModelWorld::Firms(w) => w.as_mut(),
         }
     }
 
@@ -983,6 +1004,7 @@ impl ModelWorld {
             ModelWorld::Bali(w) => copy_without_history!(Bali, w),
             ModelWorld::Line(w) => copy_without_history!(Line, w),
             ModelWorld::Tipping(w) => copy_without_history!(Tipping, w),
+            ModelWorld::Firms(w) => copy_without_history!(Firms, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -1035,6 +1057,7 @@ impl ModelWorld {
             }
             (ModelWorld::Line(live), ModelWorld::Line(kept)) => restore_into!(live, kept),
             (ModelWorld::Tipping(live), ModelWorld::Tipping(kept)) => restore_into!(live, kept),
+            (ModelWorld::Firms(live), ModelWorld::Firms(kept)) => restore_into!(live, kept),
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1459,6 +1482,30 @@ mod tests {
         assert_eq!(w.model().tick(), 0);
     }
 
+    #[test]
+    fn firms_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "firms", "agents": 50, "beta": 1.8, "stop_at": 3}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Firms);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(
+            (json["model"].as_str(), json["neighbors"].as_u64()),
+            (Some("firms"), Some(2))
+        );
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["firms", "births"]);
+        let e = ModelConfig::from_json(r#"{"model": "firms", "agents": 1}"#).unwrap_err();
+        assert_eq!(e[0].field, "agents");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!(w.kind(), ModelKind::Firms);
+        let cp = w.checkpoint().expect("firms worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
     #[test]
     fn bali_configs_round_trip_with_their_tag() {
         let c = ModelConfig::from_json(
@@ -1555,7 +1602,8 @@ mod tests {
                 "zi",
                 "bali",
                 "line",
-                "tipping"
+                "tipping",
+                "firms"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
````

Modify `crates/sugarscape-core/src/presets.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index ccd4a52..efc5c04 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -1332,6 +1332,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::punishment::presets());
     out.extend(crate::zi::presets());
     out.extend(crate::bali::presets());
+    out.extend(crate::firms::presets());
     out.extend(crate::line::presets());
     out.extend(crate::tipping::presets());
     out
````

Modify `crates/sugarscape-core/src/titles.rs` — eighteen titles (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/titles.rs b/crates/sugarscape-core/src/titles.rs
index a88f85b..ea50ad5 100644
--- a/crates/sugarscape-core/src/titles.rs
+++ b/crates/sugarscape-core/src/titles.rs
@@ -3,7 +3,7 @@
 //! paper (`source`) stay on the preset as its reference.
 
 /// Titles by preset id, in catalog order.
-pub const TITLES: [(&str, &str); 337] = [
+pub const TITLES: [(&str, &str); 355] = [
     (
         "ii-1-instant",
         "Sugar grows back at once: agents climb the best ridges and the poorly endowed starve",
@@ -1313,6 +1313,78 @@ pub const TITLES: [(&str, &str); 337] = [
         "janssen-fewer-links",
         "With half the pest links gone, copying stalls early",
     ),
+    (
+        "firms-base",
+        "Workers team up, free-ride and scatter: firms stay small",
+    ),
+    (
+        "firms-live",
+        "Seeing others' current effort lets bigger firms form",
+    ),
+    (
+        "firms-uniform",
+        "Activating each worker once a period changes little",
+    ),
+    (
+        "firms-beta-17",
+        "Weaker returns to teamwork keep every firm small",
+    ),
+    (
+        "firms-beta-21",
+        "Stronger returns to teamwork grow firms of hundreds",
+    ),
+    (
+        "firms-b-15",
+        "A bigger team bonus grows bigger firms",
+    ),
+    (
+        "firms-b-random",
+        "Firms with lucky technology grow largest",
+    ),
+    (
+        "firms-theta-075",
+        "Identical workers form many mid-sized firms and are happier",
+    ),
+    (
+        "firms-friends-10",
+        "More friends spread workers over mid-sized firms",
+    ),
+    (
+        "firms-random-firms-10",
+        "Shopping among random firms keeps firms mid-sized",
+    ),
+    (
+        "firms-loyal-10",
+        "Loyal workers keep firms alive ten times longer",
+    ),
+    (
+        "firms-sticky",
+        "Slow-changing effort swallows everyone into one firm",
+    ),
+    (
+        "firms-groping",
+        "Trial-and-error effort swallows everyone into one firm",
+    ),
+    (
+        "firms-seniority-5",
+        "Paying founders most stops anyone from joining",
+    ),
+    (
+        "firms-base-pay-80",
+        "Guaranteed pay keeps firms together but kills effort",
+    ),
+    (
+        "firms-hiring-100",
+        "Hiring only the as-eager keeps firms small",
+    ),
+    (
+        "firms-random-choices",
+        "Random moves never build a big firm",
+    ),
+    (
+        "firms-2013",
+        "Axtell's 2013 settings give Zipf's law and one giant firm",
+    ),
 ];
 
 /// The title of preset `id`, or "" if it has none.
````

- [ ] **Step 3: Run the model's tests**

Run: `cargo test --release -p sugarscape-core --lib firms`
Expected: PASS (37, the round-trip test in `model.rs` included).

- [ ] **Step 4: Watch the golden check fail, then record the entries**

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: FAIL, naming the first `firms-` preset without a fingerprint (run `print_golden`).

Modify `crates/sugarscape-core/tests/golden.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 55ef072..17d91f7 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -359,6 +359,25 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("janssen-generalized", 0xa2ae0ad93a97f180),
     ("janssen-adaptive", 0xea4877048a644fad),
     ("janssen-fewer-links", 0x878f982a182d0672),
+    // Milestone 30: The Emergence of Firms (200 periods).
+    ("firms-base", 0xf289726485a9084c),
+    ("firms-live", 0x8cb9364c43b4d2ad),
+    ("firms-uniform", 0xa8f9007712e0714),
+    ("firms-beta-17", 0x51a0c706a25d803a),
+    ("firms-beta-21", 0x967e02c2d9f1f5b),
+    ("firms-b-15", 0xb9b2b6650eb6c678),
+    ("firms-b-random", 0xd82d8e6ac458f55b),
+    ("firms-theta-075", 0xf13e84290fb10535),
+    ("firms-friends-10", 0x61a9eadb71f6d389),
+    ("firms-random-firms-10", 0xef5ffd52a915ef40),
+    ("firms-loyal-10", 0x7f561840895caf83),
+    ("firms-sticky", 0x4ce2e768347c9fb2),
+    ("firms-groping", 0x317cb37372a9b66f),
+    ("firms-seniority-5", 0x493639d792f2a77b),
+    ("firms-base-pay-80", 0x6ed803ed3a45d863),
+    ("firms-hiring-100", 0xd8fb44c3f2acfa13),
+    ("firms-random-choices", 0xd5650a4bd4548db4),
+    ("firms-2013", 0xc7acf4a34472ca50),
 ];
 
 fn fingerprint(id: &str) -> u64 {
````

Run: `cargo test --release -p sugarscape-core --test golden`
Expected: PASS.

- [ ] **Step 5: Format, lint, run everything, commit**

```bash
cargo fmt --all && cargo +stable clippy --all-targets -- -D warnings && cargo test --release --workspace
git add crates/sugarscape-core/src/firms crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs crates/sugarscape-core/tests/golden.rs
```
```bash
git commit -m "Add The Emergence of Firms (Axtell 1999, with his 2013 parameterization) as a model kind

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/{firms-beta,firms-b,firms-preferences,firms-friends,firms-random-firms,firms-loyalty,firms-sticky,firms-groping,firms-seniority,firms-base-pay,firms-hiring,firms-readings,firms-population}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets and series (`mu`).
- Produces: thirteen built-in sweeps; the CLI's stop `(its last period)`.

- [ ] **Step 1: Write the failing tests**

Modify `crates/sugarscape-cli/tests/cli.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index dadf29e..400e37e 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -187,6 +187,19 @@ fn presets_and_sweeps_are_listed() {
         "bali-gamma",
         "bali-adaptive",
         "bali-links",
+        "firms-beta",
+        "firms-b",
+        "firms-preferences",
+        "firms-friends",
+        "firms-random-firms",
+        "firms-loyalty",
+        "firms-sticky",
+        "firms-groping",
+        "firms-seniority",
+        "firms-base-pay",
+        "firms-hiring",
+        "firms-readings",
+        "firms-population",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -626,6 +639,22 @@ fn a_zi_run_stops_at_its_last_period() {
     assert_eq!(stderr(&out), "finished at tick 300 (its last period)\n");
 }
 
+#[test]
+fn a_firms_run_stops_at_its_last_period() {
+    let dir = scratch("firms");
+    let config = dir.join("firms.json");
+    std::fs::write(&config, r#"{"model": "firms", "agents": 50, "stop_at": 7}"#).unwrap();
+    let out = sugarscape(&[
+        "run",
+        "--config",
+        config.to_str().unwrap(),
+        "--ticks",
+        "1000",
+    ]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    assert_eq!(stderr(&out), "finished at tick 7 (its last period)\n");
+}
+
 #[test]
 fn a_bali_run_stops_at_its_last_year() {
     let dir = scratch("bali");
````

Modify `crates/sugarscape-wasm/tests/web.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index b810027..b6e1166 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -417,7 +417,20 @@ fn builtins_and_series_names_are_listed() {
             "bali-links",
             "cache-capacity",
             "cache-winter",
-            "central-distance"
+            "central-distance",
+            "firms-beta",
+            "firms-b",
+            "firms-preferences",
+            "firms-friends",
+            "firms-random-firms",
+            "firms-loyalty",
+            "firms-sticky",
+            "firms-groping",
+            "firms-seniority",
+            "firms-base-pay",
+            "firms-hiring",
+            "firms-readings",
+            "firms-population"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -1234,6 +1247,30 @@ fn zi_sims_match_the_native_golden_entries() {
     }
 }
 
+#[wasm_bindgen_test]
+fn firms_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the closed-form
+    // optimum, general β (portable powers), per-firm draws, base pay,
+    // groping and random behavior.
+    for (id, fp) in [
+        ("firms-base", "0xf289726485a9084c"),
+        ("firms-live", "0x8cb9364c43b4d2ad"),
+        ("firms-beta-21", "0x0967e02c2d9f1f5b"),
+        ("firms-b-random", "0xd82d8e6ac458f55b"),
+        ("firms-theta-075", "0xf13e84290fb10535"),
+        ("firms-random-firms-10", "0xef5ffd52a915ef40"),
+        ("firms-seniority-5", "0x493639d792f2a77b"),
+        ("firms-base-pay-80", "0x6ed803ed3a45d863"),
+        ("firms-groping", "0x317cb37372a9b66f"),
+        ("firms-random-choices", "0xd5650a4bd4548db4"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "firms");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
 #[wasm_bindgen_test]
 fn bali_sims_match_the_native_golden_entries() {
     // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: rain, water and
````

Run: `cargo test --release -p sugarscape-cli`
Expected: FAIL. `a_firms_run_stops_at_its_last_period` gets another stop reason, and the listing test misses `firms-beta`.

- [ ] **Step 2: Write the sweeps, register them and name the stop**

Create `sweeps/firms-beta.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: increasing returns β (Table 3)",
  "description": "Axtell's Table 3: the size exponent µ against increasing returns β. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: β 1.7, 1.8, 1.9, 2.0, 2.1: 3.63, 3.29, 2.93, 2.51, 2.02; β drawn per firm from 1.7–2.1: 2.55. It falls as β rises, as the table's does (2.06, 1.62, 1.32, 1.28, 0.95; drawn 1.12) — about 1 higher throughout. The β ≠ 2 cells need a numeric effort solve: about a minute a run.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "β",
    "values": [
      {
        "at": 1,
        "name": "β = 1.7",
        "set": {
          "beta": 1.7
        }
      },
      {
        "at": 2,
        "name": "β = 1.8",
        "set": {
          "beta": 1.8
        }
      },
      {
        "at": 3,
        "name": "β = 1.9",
        "set": {
          "beta": 1.9
        }
      },
      {
        "at": 4,
        "name": "β = 2.0",
        "set": {
          "beta": 2.0
        }
      },
      {
        "at": 5,
        "name": "β = 2.1",
        "set": {
          "beta": 2.1
        }
      },
      {
        "at": 6,
        "name": "β drawn per firm, 1.7–2.1",
        "set": {
          "beta": 1.7,
          "beta_max": 2.1
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-b.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: increasing returns b (Table 4)",
  "description": "Axtell's Table 4: µ against the increasing-returns coefficient b. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: b 0.5, 0.75, 1.0, 1.25, 1.5: 3.38, 2.83, 2.51, 2.26, 2.09; b drawn per firm from 0.5–1.5: 1.83 — below b = 1.25's, so 'much more like b = 1.25, not b = 1.0' holds. The table: 2.09, 1.33, 1.28, 0.91, 0.53; drawn 0.89.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "b",
    "values": [
      {
        "at": 1,
        "name": "b = 0.5",
        "set": {
          "b": 0.5
        }
      },
      {
        "at": 2,
        "name": "b = 0.75",
        "set": {
          "b": 0.75
        }
      },
      {
        "at": 3,
        "name": "b = 1.0",
        "set": {
          "b": 1.0
        }
      },
      {
        "at": 4,
        "name": "b = 1.25",
        "set": {
          "b": 1.25
        }
      },
      {
        "at": 5,
        "name": "b = 1.5",
        "set": {
          "b": 1.5
        }
      },
      {
        "at": 6,
        "name": "b drawn per firm, 0.5–1.5",
        "set": {
          "b": 0.5,
          "b_max": 1.5
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-preferences.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: preferences (Table 5)",
  "description": "Axtell's Table 5: µ under nine distributions of preferences. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: θ uniform on [0, 1] 2.51; on [0.25, 0.75] 3.30; triangular (mode 0.5) 3.06; triangular (mode 0.75) 2.52; truncated normal 2.55; Beta(1, 2) 3.29; all θ = 0.75 1.25; CES with ρ on [−1, 0] 0.94; CES with ρ on [0, 10] 0.72. The table: 1.28, 1.21, 1.31, 1.01, 1.30, 0.99, 0.91, 1.56, 1.26. Only the homogeneous and CES rows come near its numbers.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Preferences",
    "values": [
      {
        "at": 1,
        "name": "θ uniform on [0, 1]",
        "set": {
          "preferences": "uniform"
        }
      },
      {
        "at": 2,
        "name": "θ uniform on [0.25, 0.75]",
        "set": {
          "preferences": "middle"
        }
      },
      {
        "at": 3,
        "name": "θ triangular, mode 0.5",
        "set": {
          "preferences": "triangular"
        }
      },
      {
        "at": 4,
        "name": "θ triangular, mode 0.75",
        "set": {
          "preferences": "triangular_high"
        }
      },
      {
        "at": 5,
        "name": "θ truncated normal",
        "set": {
          "preferences": "normal"
        }
      },
      {
        "at": 6,
        "name": "θ Beta(1, 2)",
        "set": {
          "preferences": "beta"
        }
      },
      {
        "at": 7,
        "name": "θ = 0.75 for all",
        "set": {
          "preferences": "fixed",
          "theta": 0.75
        }
      },
      {
        "at": 8,
        "name": "CES, ρ on [−1, 0]",
        "set": {
          "preferences": "ces",
          "rho": -1.0,
          "rho_max": 0.0
        }
      },
      {
        "at": 9,
        "name": "CES, ρ on [0, 10]",
        "set": {
          "preferences": "ces",
          "rho": 0.0,
          "rho_max": 10.0
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-friends.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: friends ν (Table 6)",
  "description": "Axtell's Table 6: µ against the number of fixed friends ν. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: ν 2, 4, 6, 8, 10: 2.51, 2.59, 2.69, 2.71, 2.66; ν drawn per agent from 2–10: 2.69. µ rises with ν — the opposite of the table (1.28, 1.11, 1.08, 1.08, 0.99; drawn 1.11), whose text says larger networks 'stabilize large firms'.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Friends",
    "values": [
      {
        "at": 1,
        "name": "ν = 2",
        "set": {
          "neighbors": 2
        }
      },
      {
        "at": 2,
        "name": "ν = 4",
        "set": {
          "neighbors": 4
        }
      },
      {
        "at": 3,
        "name": "ν = 6",
        "set": {
          "neighbors": 6
        }
      },
      {
        "at": 4,
        "name": "ν = 8",
        "set": {
          "neighbors": 8
        }
      },
      {
        "at": 5,
        "name": "ν = 10",
        "set": {
          "neighbors": 10
        }
      },
      {
        "at": 6,
        "name": "ν drawn per agent, 2–10",
        "set": {
          "neighbors": 2,
          "neighbors_max": 10
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-random-firms.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: random firms ν (Table 7)",
  "description": "Axtell's Table 7: µ when agents look at ν random firms, redrawn at each activation, instead of friends' firms. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: ν 2, 4, 6, 8, 10: 3.29, 2.96, 2.89, 2.86, 2.81; ν drawn 2–10: 2.97. It falls with ν, as the table's does (1.28, 1.22, 1.18, 1.07, 1.03; drawn 1.02). The table's ν = 2 row repeats the base case's 1.28 though the rule differs; here the rules give 3.29 against 2.51.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Random firms",
    "values": [
      {
        "at": 1,
        "name": "ν = 2",
        "set": {
          "network": "random_firms",
          "neighbors": 2
        }
      },
      {
        "at": 2,
        "name": "ν = 4",
        "set": {
          "network": "random_firms",
          "neighbors": 4
        }
      },
      {
        "at": 3,
        "name": "ν = 6",
        "set": {
          "network": "random_firms",
          "neighbors": 6
        }
      },
      {
        "at": 4,
        "name": "ν = 8",
        "set": {
          "network": "random_firms",
          "neighbors": 8
        }
      },
      {
        "at": 5,
        "name": "ν = 10",
        "set": {
          "network": "random_firms",
          "neighbors": 10
        }
      },
      {
        "at": 6,
        "name": "ν drawn per agent, 2–10",
        "set": {
          "network": "random_firms",
          "neighbors": 2,
          "neighbors_max": 10
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-loyalty.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: loyalty λ (Table 8)",
  "description": "Axtell's Table 8: µ against loyalty λ (an agent moves only after wanting to more than λ times). Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: λ 0, 2, 5, 10: 2.51, 2.12, 2.01, 1.97; λ drawn 0–10: 2.18. It falls, as the table's does (1.28, 1.14, 0.85, 0.77; drawn 0.79), less steeply.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Loyalty",
    "values": [
      {
        "at": 1,
        "name": "λ = 0",
        "set": {
          "loyalty": 0
        }
      },
      {
        "at": 2,
        "name": "λ = 2",
        "set": {
          "loyalty": 2
        }
      },
      {
        "at": 3,
        "name": "λ = 5",
        "set": {
          "loyalty": 5
        }
      },
      {
        "at": 4,
        "name": "λ = 10",
        "set": {
          "loyalty": 10
        }
      },
      {
        "at": 5,
        "name": "λ drawn per agent, 0–10",
        "set": {
          "loyalty": 0,
          "loyalty_max": 10
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-sticky.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: sticky effort (Table 9)",
  "description": "Axtell's Table 9: µ against β with sticky effort (a new effort within ±0.05 of the current one), read two ways. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: free: 3.63, 3.29, 2.93, 2.51, 2.02 (β 1.7–2.1); sticky in any firm: 3.06, 0.57, −0.03, −0.11, −0.13 — from β 1.8 on, the whole population merges into one firm at times and the OLS fit loses meaning; sticky only in the agent's own firm: 3.53, 3.23, 2.82, 2.41, 1.88, a mild lowering like the table's (µ_sticky 1.54, 1.25, 1.17, 0.92, 0.95). About 25 minutes.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "β",
    "path": "beta",
    "values": [
      1.7,
      1.8,
      1.9,
      2.0,
      2.1
    ]
  },
  "series": {
    "label": "Effort",
    "values": [
      {
        "at": 1,
        "name": "Free",
        "set": {
          "effort_window": 1.0
        }
      },
      {
        "at": 2,
        "name": "Sticky (±0.05), in any firm",
        "set": {
          "effort_window": 0.1
        }
      },
      {
        "at": 3,
        "name": "Sticky (±0.05), own firm only",
        "set": {
          "effort_window": 0.1,
          "adjust_scope": "own_firm"
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-groping.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: groping for effort (Table 10)",
  "description": "Axtell's Table 10: µ against β when agents grope for effort (one random try, kept if better), read two ways. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: optimal: 3.63, 3.29, 2.93, 2.51, 2.02; groping in any firm (other firms weighed at the current effort): 2.67, 0.07, −0.22, −0.23, −0.20 — one firm swallows the population; groping only in the agent's own firm: 3.57, 3.22, 2.84, 2.41, 1.89. The table: 1.69, 1.49, 1.33, 1.19, 1.00 — and its text's 'more pronounced' than stickiness contradicts it.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "β",
    "path": "beta",
    "values": [
      1.7,
      1.8,
      1.9,
      2.0,
      2.1
    ]
  },
  "series": {
    "label": "Effort",
    "values": [
      {
        "at": 1,
        "name": "Optimal",
        "set": {
          "groping": false
        }
      },
      {
        "at": 2,
        "name": "Groping, in any firm",
        "set": {
          "groping": true
        }
      },
      {
        "at": 3,
        "name": "Groping, own firm only",
        "set": {
          "groping": true,
          "adjust_scope": "own_firm"
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-seniority.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: seniority pay (Table 11)",
  "description": "Axtell's Table 11: µ under seniority shares ∝ p^−rank. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: 5^−rank 10.12, 4^−rank 8.55, 3^−rank 8.39, 2^−rank 5.41, equal shares 2.51; 2^−(rank+1), 2^−(rank+2), 2^−(rank+3): 5.41, 5.41, 5.41 — identical, since normalized shares are the same (the table's 0.89, 0.99, 1.07, 1.04 for them is its run-to-run noise); paying the newest most, 5^−rank 0.22 and 2^−rank 0.18. With the longest-serving paid most (the text), joiners get too little and firms barely form; the table's 1.11–0.89 comes from neither ordering.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Shares",
    "values": [
      {
        "at": 1,
        "name": "5^−rank",
        "set": {
          "pay": "seniority",
          "seniority_base": 5.0
        }
      },
      {
        "at": 2,
        "name": "4^−rank",
        "set": {
          "pay": "seniority",
          "seniority_base": 4.0
        }
      },
      {
        "at": 3,
        "name": "3^−rank",
        "set": {
          "pay": "seniority",
          "seniority_base": 3.0
        }
      },
      {
        "at": 4,
        "name": "2^−rank",
        "set": {
          "pay": "seniority",
          "seniority_base": 2.0
        }
      },
      {
        "at": 5,
        "name": "Equal shares (1^−rank)",
        "set": {
          "pay": "equal"
        }
      },
      {
        "at": 6,
        "name": "2^−(rank+1) (the same shares as 2^−rank)",
        "set": {
          "pay": "seniority",
          "seniority_base": 2.0
        }
      },
      {
        "at": 7,
        "name": "2^−(rank+2) (the same shares as 2^−rank)",
        "set": {
          "pay": "seniority",
          "seniority_base": 2.0
        }
      },
      {
        "at": 8,
        "name": "2^−(rank+3) (the same shares as 2^−rank)",
        "set": {
          "pay": "seniority",
          "seniority_base": 2.0
        }
      },
      {
        "at": 9,
        "name": "5^−rank, newest first",
        "set": {
          "pay": "seniority",
          "seniority_base": 5.0,
          "seniority_order": "junior_first"
        }
      },
      {
        "at": 10,
        "name": "2^−rank, newest first",
        "set": {
          "pay": "seniority",
          "seniority_base": 2.0,
          "seniority_order": "junior_first"
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-base-pay.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: base pay plus a bonus (Table 12)",
  "description": "Axtell's Table 12: µ with base pay plus an equal bonus from output above it. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: none 2.51; 20, 50, 80 % of each agent's singleton income: 2.31, 2.03, 1.73; the median agent's singleton income 2.45; the mean agent's 2.43. It falls with the base, as the table's does (1.28, 1.26, 1.06, 0.85, 0.99, 1.01), while effort and output collapse at 80 %.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Base pay",
    "values": [
      {
        "at": 1,
        "name": "None (equal shares)",
        "set": {
          "pay": "equal"
        }
      },
      {
        "at": 2,
        "name": "20 % of own singleton income",
        "set": {
          "pay": "base",
          "base_pay": "own",
          "base_share": 0.2
        }
      },
      {
        "at": 3,
        "name": "50 % of own singleton income",
        "set": {
          "pay": "base",
          "base_pay": "own",
          "base_share": 0.5
        }
      },
      {
        "at": 4,
        "name": "80 % of own singleton income",
        "set": {
          "pay": "base",
          "base_pay": "own",
          "base_share": 0.8
        }
      },
      {
        "at": 5,
        "name": "The median agent's singleton income",
        "set": {
          "pay": "base",
          "base_pay": "median",
          "base_share": 1.0
        }
      },
      {
        "at": 6,
        "name": "The mean agent's singleton income",
        "set": {
          "pay": "base",
          "base_pay": "mean",
          "base_share": 1.0
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-hiring.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: hiring standards (Table 13)",
  "description": "Axtell's Table 13 (captioned 'target output' but about hiring standards): µ when firms admit only θ at least φ times their longest-serving member's. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: φ 0, 20, 40, 60, 80, 100 %: 2.51, 2.30, 2.01, 1.99, 2.33, 2.49; φ drawn per firm 0–100 %: 2.40 — falling, then rising again, as the table's (1.28, 1.27, 1.22, 1.03, 1.17, no power law; drawn 1.13).",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Hiring",
    "values": [
      {
        "at": 1,
        "name": "φ = 0 %",
        "set": {
          "hiring": 0
        }
      },
      {
        "at": 2,
        "name": "φ = 20 %",
        "set": {
          "hiring": 0.2
        }
      },
      {
        "at": 3,
        "name": "φ = 40 %",
        "set": {
          "hiring": 0.4
        }
      },
      {
        "at": 4,
        "name": "φ = 60 %",
        "set": {
          "hiring": 0.6
        }
      },
      {
        "at": 5,
        "name": "φ = 80 %",
        "set": {
          "hiring": 0.8
        }
      },
      {
        "at": 6,
        "name": "φ = 100 %",
        "set": {
          "hiring": 1.0
        }
      },
      {
        "at": 7,
        "name": "φ drawn per firm, 0–100 %",
        "set": {
          "hiring": 0.0,
          "hiring_max": 1.0
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-readings.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: readings of the paper",
  "description": "The paper's unstated rules, read each way. Measured (release, seeds 1–10, 5 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: literal (others' effort from last period's output, random activation) 2.51; others' effort live 1.98; uniform activation 2.54; live and uniform 1.79; a 10-step line search 2.48; starting in random groups 2.49; starting in one firm 2.50. No reading comes near Axtell's 1.28; the start does not matter, as he says.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Reading",
    "values": [
      {
        "at": 1,
        "name": "Literal: last period's effort, random activation",
        "set": {}
      },
      {
        "at": 2,
        "name": "Others' effort live",
        "set": {
          "others_effort": "live"
        }
      },
      {
        "at": 3,
        "name": "Uniform activation",
        "set": {
          "activation": "uniform"
        }
      },
      {
        "at": 4,
        "name": "Live and uniform",
        "set": {
          "others_effort": "live",
          "activation": "uniform"
        }
      },
      {
        "at": 5,
        "name": "A coarse line search (10 steps)",
        "set": {
          "effort_search": "grid",
          "grid_steps": 10
        }
      },
      {
        "at": 6,
        "name": "Start in random groups",
        "set": {
          "initial": "random_groups"
        }
      },
      {
        "at": 7,
        "name": "Start in one firm",
        "set": {
          "initial": "one_firm"
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
    "series": "mu"
  }
}
````

Create `sweeps/firms-population.json` with exactly this content:

````json
{
  "name": "The Emergence of Firms: population size",
  "description": "The base case at 1 000 to 30 000 agents (Axtell: 'invariant'). Measured (release, seeds 1–3, 2 000 periods, burn-in 500, recorded 2026-09-30), µ by the paper's OLS: 2.50, 2.45, 2.50, 2.49 at 1 000, 3 000, 10 000 and 30 000 agents — invariant.",
  "base": {
    "preset": "firms-base"
  },
  "x": {
    "label": "Agents",
    "path": "agents",
    "values": [
      1000,
      3000,
      10000,
      30000
    ]
  },
  "seeds": {
    "from": 1,
    "count": 3
  },
  "ticks": 2000,
  "metric": {
    "kind": "final",
    "series": "mu"
  }
}
````

Modify `crates/sugarscape-core/src/sweep.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index bb411de..f068789 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -965,7 +965,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 163] = [
+const BUILTINS: [Builtin; 176] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1618,6 +1618,58 @@ const BUILTINS: [Builtin; 163] = [
         id: "central-distance",
         json: include_str!("../../../sweeps/central-distance.json"),
     },
+    Builtin {
+        id: "firms-beta",
+        json: include_str!("../../../sweeps/firms-beta.json"),
+    },
+    Builtin {
+        id: "firms-b",
+        json: include_str!("../../../sweeps/firms-b.json"),
+    },
+    Builtin {
+        id: "firms-preferences",
+        json: include_str!("../../../sweeps/firms-preferences.json"),
+    },
+    Builtin {
+        id: "firms-friends",
+        json: include_str!("../../../sweeps/firms-friends.json"),
+    },
+    Builtin {
+        id: "firms-random-firms",
+        json: include_str!("../../../sweeps/firms-random-firms.json"),
+    },
+    Builtin {
+        id: "firms-loyalty",
+        json: include_str!("../../../sweeps/firms-loyalty.json"),
+    },
+    Builtin {
+        id: "firms-sticky",
+        json: include_str!("../../../sweeps/firms-sticky.json"),
+    },
+    Builtin {
+        id: "firms-groping",
+        json: include_str!("../../../sweeps/firms-groping.json"),
+    },
+    Builtin {
+        id: "firms-seniority",
+        json: include_str!("../../../sweeps/firms-seniority.json"),
+    },
+    Builtin {
+        id: "firms-base-pay",
+        json: include_str!("../../../sweeps/firms-base-pay.json"),
+    },
+    Builtin {
+        id: "firms-hiring",
+        json: include_str!("../../../sweeps/firms-hiring.json"),
+    },
+    Builtin {
+        id: "firms-readings",
+        json: include_str!("../../../sweeps/firms-readings.json"),
+    },
+    Builtin {
+        id: "firms-population",
+        json: include_str!("../../../sweeps/firms-population.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2603,7 +2655,20 @@ mod tests {
                 "bali-links",
                 "cache-capacity",
                 "cache-winter",
-                "central-distance"
+                "central-distance",
+                "firms-beta",
+                "firms-b",
+                "firms-preferences",
+                "firms-friends",
+                "firms-random-firms",
+                "firms-loyalty",
+                "firms-sticky",
+                "firms-groping",
+                "firms-seniority",
+                "firms-base-pay",
+                "firms-hiring",
+                "firms-readings",
+                "firms-population"
             ]
         );
         for b in builtins() {
````

Modify `crates/sugarscape-cli/src/main.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index e6f4052..42342f9 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -238,6 +238,7 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             ModelKind::Ants | ModelKind::Thresholds => "its last step",
             ModelKind::Punishment | ModelKind::Zi => "its last period",
             ModelKind::Bali => "its last year",
+            ModelKind::Firms => "its last period",
             ModelKind::Retirement => match &config {
                 ModelConfig::Retirement(c)
                     if c.stop_at_norm
````

- [ ] **Step 3: Run the tests and measure the sweeps**

Run: `cargo test --release --workspace && wasm-pack test --node crates/sugarscape-wasm`, then `cargo build --release -p sugarscape-cli` and, for each sweep, `./target/release/sugarscape sweep --builtin <id> --quiet --summary-csv /tmp/<id>.csv --out /dev/null`.
Expected:
- The tests PASS: 64 WASM tests, including `firms_sims_match_the_native_golden_entries`.
- Each summary's means match what its description records.
- Timings: most sweeps take one to two minutes (`firms-loyalty`, `firms-hiring`, `firms-b`, `firms-friends`, `firms-random-firms`, `firms-readings`, `firms-population`). `firms-beta` and `firms-seniority` take about 8 minutes, `firms-groping` about 15, `firms-sticky` about 27, `firms-preferences` about 36 and `firms-base-pay` about 46. Base pay and CES use the slower root search.

- [ ] **Step 4: Format, lint, commit**

```bash
cargo fmt --all && cargo +stable clippy --all-targets -- -D warnings
git add sweeps/firms-beta.json sweeps/firms-b.json sweeps/firms-preferences.json sweeps/firms-friends.json sweeps/firms-random-firms.json sweeps/firms-loyalty.json sweeps/firms-sticky.json sweeps/firms-groping.json sweeps/firms-seniority.json sweeps/firms-base-pay.json sweeps/firms-hiring.json sweeps/firms-readings.json sweeps/firms-population.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli crates/sugarscape-wasm/tests/web.rs
```
```bash
git commit -m "Measure The Emergence of Firms: thirteen sweeps, the CLI's stop and WASM agreement

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/compare-presets.test.ts`, `web/src/engine.test.ts`, `web/src/experiments/form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the WASM build of Tasks 1–2.
- Produces:
  - In types.ts: `FirmsConfig`, `FirmsStats` and `FirmsInspection`, plus the color modes `founder`, `theta`, `effort` and `income`.
  - `isFirmsView`, which tests `panel`, `firm` and `member`.
  - `MODEL_CHARTS.firms`.
  - The Compare entry `firms-last-vs-live`.
  - `ticksLeft` for `stop_at` periods.
  - The Experiments default: `mu` against `beta` `1.7:2.1:0.1` over 5 000 periods.

- [ ] **Step 1: Write the failing tests**

Modify `web/src/models.test.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index de980c4..f4402d7 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -10,6 +10,7 @@ import {
   usesMinds,
   worldMenu,
   isBaliView,
+  isFirmsView,
   isThresholdsView,
   isPunishmentView,
   isZiView,
@@ -230,6 +231,25 @@ describe('the presets menu', () => {
   });
 });
 
+describe('the firms model', () => {
+  it('is read by its tag, and its inspections by `firm` and `member`, before the others with a panel', () => {
+    expect(modelOf({ model: 'firms' } as unknown as ModelConfig)).toBe('firms');
+    const cell = { site: { x: 1, y: 2 }, panel: 'firms', firm: null, member: null, agent: null } as unknown as AnyInspection;
+    const bali = { site: { x: 1, y: 2 }, panel: 'map', subak: null, dam: null, month: null, stress: null, agent: null } as unknown as AnyInspection;
+    expect([cell, bali].map(isFirmsView)).toEqual([true, false]);
+    expect([isBaliView(cell), isZiView(cell), isPunishmentView(cell), isRetirementView(cell), isThresholdsView(cell)]).toEqual([false, false, false, false, false]);
+  });
+
+  it('colors four ways, has no overlays, and ends after its periods', () => {
+    expect(COLOR_MODES.firms.map(([m]) => m)).toEqual(['founder', 'theta', 'effort', 'income']);
+    expect(MODEL_OVERLAYS.firms).toEqual([]);
+    const c = { model: 'firms', stop_at: 5000 } as unknown as ModelConfig;
+    expect(ticksLeft(c, 4990)).toBe(10);
+    expect(ticksLeft({ ...c, stop_at: 0 } as unknown as ModelConfig, 50)).toBe(Infinity);
+    expect(finishesUnpredictably(c)).toBe(false);
+  });
+});
+
 describe('the bali model', () => {
   it('is read by its tag, and its inspections by `subak` and `dam`, before the others with a panel', () => {
     expect(modelOf({ model: 'bali' } as unknown as ModelConfig)).toBe('bali');
````

Modify `web/src/compare-presets.test.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index 1ed8b81..06e0558 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -49,6 +49,11 @@ describe('compare presets', () => {
     expect(ids).toContainEqual(['zi-c-vs-zip', 'cliff-excess-demand', 'zip-excess-demand', 'ZI-C vs ZIP in a box market — Zero-Intelligence Traders (Compare)']);
   });
 
+  it('pairs the literal reading of others’ effort with the live one', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['firms-last-vs-live', 'firms-base', 'firms-live', "Last period's effort vs live effort — The Emergence of Firms (Compare)"]);
+  });
+
   it('pairs imitation with the same plans fixed', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['lk-random-vs-fixed', 'lk-random', 'lk-random-fixed', 'Imitating neighbors vs fixed random plans — Balinese Water Temples (Compare)']);
````

Modify `web/src/engine.test.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index ad4f545..500ab1c 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1165,6 +1165,7 @@ describe('Engine with other models', () => {
     expect(finishedNotice({ model: 'farol', stop_at: 100 } as unknown as ModelConfig, 100)).toBe('This run has reached its last round (100) — Reset to run it again');
     expect(finishedNotice({ model: 'ants', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last step (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'thresholds', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last step (50) — Reset to run it again');
+    expect(finishedNotice({ model: 'firms', stop_at: 5000 } as unknown as ModelConfig, 5000)).toBe('This run has reached its last period (5000) — Reset to run it again');
     expect(finishedNotice({ model: 'bali', stop_at: 30 } as unknown as ModelConfig, 360)).toBe('This run has reached its last year — Reset to run it again');
     expect(finishedNotice({ model: 'zi', stop_at: 6 } as unknown as ModelConfig, 12000)).toBe('This run has reached its last period — Reset to run it again');
     expect(finishedNotice({ model: 'punishment', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last period (2000) — Reset to run it again');
````

Modify `web/src/experiments/form.test.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index 2e8796a..ef84335 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -147,6 +147,11 @@ describe('sweeps over other models', () => {
       ticks: 550,
       metric: { kind: 'final', series: 'fit' },
     });
+    expect(defaultForm('firms')).toMatchObject({
+      x: { path: 'beta', values: '1.7:2.1:0.1' },
+      ticks: 5000,
+      metric: { kind: 'final', series: 'mu' },
+    });
     expect(defaultForm('bali')).toMatchObject({
       x: { path: 'growth', values: '2:2.4:0.1' },
       ticks: 360,
````

Modify `web/src/ui/series-data.test.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index eeebe19..0a85a5c 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -227,6 +227,13 @@ describe('the anasazi’s charts', () => {
   });
 });
 
+describe('firms charts', () => {
+  it('chart firms, sizes, effort and pay, output and the scaling exponent over periods', () => {
+    expect(MODEL_CHARTS.firms.map((c) => c.title)).toEqual(['Firms', 'Sizes', 'Effort and pay', 'Output', 'Scaling']);
+    expect(timeAxisLabel('firms')).toBe('Periods');
+  });
+});
+
 describe('bali charts', () => {
   it('chart harvest, changing plans, water and pests, patches and the temple match over months', () => {
     expect(MODEL_CHARTS.bali.map((c) => c.title)).toEqual(['Harvest', 'Changing plans', 'Water and pests', 'Patches', 'Temple match']);
````

Modify `web/src/determinism.test.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index dad39d9..bc159e1 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -15,6 +15,9 @@ import type {
   AgreementConfig,
   ZiConfig,
   BaliConfig,
+  FirmsConfig,
+  FirmsInspection,
+  FirmsStats,
   BaliInspection,
   BaliStats,
   ZiInspection,
@@ -744,6 +747,29 @@ describe('the Minds menu over the real presets', () => {
   });
 });
 
+describe('the firms model through the engine', () => {
+  it('stops after its last period and inspects a firm, its member and the size plot', async () => {
+    const r = presets.find((p) => p.id === 'firms-base')!;
+    const config = { ...structuredClone(r.config as FirmsConfig), agents: 60, burn_in: 5, stop_at: 30 };
+    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'founder' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(1_000_000);
+    const s = e.latest as FirmsStats;
+    expect([e.finished, ends, e.tick, s.period]).toEqual([true, 1, 30, 30]);
+    expect(s.firms).toBeLessThan(60);
+    // The first row is the largest firm; its first cell its longest-serving member.
+    await e.select(0, 0);
+    const v = e.inspection!.view as FirmsInspection;
+    expect(v.panel).toBe('firms');
+    expect(v.member?.firm).toBe(v.firm?.id);
+    // The size plot starts 8 pixels right of the 600-pixel rows.
+    await e.select(700, 50);
+    expect((e.inspection!.view as FirmsInspection).panel).toBe('sizes');
+  });
+});
+
 describe('the bali model through the engine', () => {
   it('stops after its last year and inspects a subak, and a dam in the water strip', async () => {
     const r = presets.find((p) => p.id === 'lk-random')!;
````

- [ ] **Step 2: Run them to see them fail**

Run: `cd web && npm ci && npm run wasm && npx vitest run`
Expected: failures in the six files above.

- [ ] **Step 3: Carry the model through the page**

Modify `web/src/types.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/types.ts b/web/src/types.ts
index 51532e2..407ffdd 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -193,7 +193,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment' | 'zi' | 'bali' | 'line' | 'tipping';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment' | 'zi' | 'bali' | 'line' | 'tipping' | 'firms';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -605,7 +605,7 @@ export interface AgreementConfig {
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig | ZiConfig | BaliConfig | LineConfig | TippingConfig;
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig | ZiConfig | BaliConfig | LineConfig | TippingConfig | FirmsConfig;
 
 /**
  * Arthur's El Farol bar and Challet and Zhang's minority game (milestone 23), with Challet, Marsili
@@ -1401,7 +1401,7 @@ export interface AgreementStats {
   stable_at: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats | ZiStats | BaliStats;
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats | ZiStats | BaliStats | FirmsStats;
 
 export interface SiteView {
   x: number;
@@ -1527,6 +1527,81 @@ export interface TippingConfig {
   limit_blue: number;
   limit_total: number;
 }
+/**
+ * Axtell's emergence of firms (milestone 30): agents choosing effort in teams with increasing returns and
+ * equal shares, moving between their firm, a start-up and their friends' firms. One tick is a period.
+ */
+export interface FirmsConfig {
+  model: 'firms';
+  agents: number;
+  a: number;
+  a_max: number;
+  b: number;
+  b_max: number;
+  beta: number;
+  beta_max: number;
+  preferences: 'uniform' | 'middle' | 'triangular' | 'triangular_high' | 'normal' | 'beta' | 'fixed' | 'ces';
+  theta: number;
+  rho: number;
+  rho_max: number;
+  ces_sign: 'text' | 'printed';
+  network: 'friends' | 'random_firms';
+  neighbors: number;
+  neighbors_max: number;
+  activation: 'random' | 'uniform';
+  activation_rate: number;
+  others_effort: 'last_period' | 'live';
+  effort_search: 'exact' | 'grid';
+  grid_steps: number;
+  effort_window: number;
+  groping: boolean;
+  loyalty: number;
+  loyalty_max: number;
+  pay: 'equal' | 'seniority' | 'base';
+  seniority_base: number;
+  base_pay: 'own' | 'median' | 'mean';
+  base_share: number;
+  hiring: number;
+  hiring_max: number;
+  random_behavior: 'none' | 'choices' | 'effort';
+  initial: 'alone' | 'random_groups' | 'one_firm';
+  burn_in: number;
+  sample_every: number;
+  stop_at: number;
+}
+
+/** A period's statistics (µ, its maximum-likelihood twin and the mean lifetime are null before the burn-in). */
+export interface FirmsStats {
+  tick: number;
+  firms: number;
+  births: number;
+  deaths: number;
+  mean_size: number;
+  largest: number;
+  singletons: number;
+  effort: number;
+  output: number;
+  income: number;
+  utility: number;
+  largest_output_share: number | null;
+  mu: number | null;
+  mu_mle: number | null;
+  lifetime: number | null;
+  period: number;
+}
+
+export interface FirmsFirmView { id: number; size: number; output: number; age: number; a: number; b: number; beta: number; mean_theta: number; mean_effort: number; free_riders: number }
+export interface FirmsMemberView { id: number; theta: number; effort: number; income: number; utility: number; tenure: number; firm: number }
+
+/** A cell of the firms frame: a firm's row and the member at it, or the size plot. */
+export interface FirmsInspection {
+  site: { x: number; y: number };
+  panel: 'firms' | 'sizes' | null;
+  firm: FirmsFirmView | null;
+  member: FirmsMemberView | null;
+  /** Always null: cells are read where they are. */
+  agent: null;
+}
 /** A point of his plane: Red and Blue inside, and whether the most tolerant of each would all be content there. */
 export interface TippingInspection { red_in: number; blue_in: number; red_content: boolean; blue_content: boolean; now: boolean; agent: null }
 export interface RingInspection { site: { x: number; sugar: number; capacity: number }; agent: { id: number; vision: number } | null }
@@ -1805,7 +1880,7 @@ export interface AgreementInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection | ZiInspection | BaliInspection | LineInspection | TippingInspection;
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection | ZiInspection | BaliInspection | LineInspection | TippingInspection | FirmsInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -1877,7 +1952,11 @@ export type ColorMode =
   | 'pests'
   | 'water'
   | 'crop'
-  | 'plane';
+  | 'plane'
+  | 'founder'
+  | 'theta'
+  | 'effort'
+  | 'income';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
````

Modify `web/src/models.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/models.ts b/web/src/models.ts
index ccca557..58d1d2a 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -3,6 +3,8 @@ import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
 import type {
   LineInspection,
   TippingInspection,
+  FirmsConfig,
+  FirmsInspection,
   ZiInspection,
   BaliConfig,
   BaliInspection,
@@ -50,7 +52,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment', 'zi', 'bali', 'line', 'tipping'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment', 'zi', 'bali', 'line', 'tipping', 'firms'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -79,12 +81,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   bali: 'Balinese Water Temples',
   line: "Schelling's line",
   tipping: "Schelling's tipping",
+  firms: 'The Emergence of Firms',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment' || tag === 'zi' || tag === 'bali' || tag === 'line' || tag === 'tipping'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment' || tag === 'zi' || tag === 'bali' || tag === 'line' || tag === 'tipping' || tag === 'firms'
     ? tag
     : 'sugarscape';
 }
@@ -184,6 +187,11 @@ export function isBaliView(v: AnyInspection): v is BaliInspection {
 }
 
 /** A cell of the zi frame (a panel, a `trade` and a step's `supply`); check it first. */
+/** A cell of the firms frame (a panel, a `firm` and a `member`); check it before the others with a panel. */
+export function isFirmsView(v: AnyInspection): v is FirmsInspection {
+  return 'panel' in v && 'firm' in v && 'member' in v;
+}
+
 /** A point of Schelling's tipping plane. */
 export function isTippingView(v: AnyInspection): v is TippingInspection {
   return 'red_content' in v && 'blue_content' in v;
@@ -252,6 +260,7 @@ export function ticksLeft(c: ModelConfig, tick: number): number {
     const b = c as BaliConfig;
     return Math.max(0, b.stop_at * (b.watershed === 'two_node' ? b.node_periods : 12) - tick);
   }
+  if (modelOf(c) === 'firms' && (c as FirmsConfig).stop_at > 0) return Math.max(0, (c as FirmsConfig).stop_at - tick);
   return Infinity;
 }
 
@@ -544,6 +553,13 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
   ],
   // His plane: Red inside across, Blue inside up; where each color is content tinted.
   tipping: [['plane', 'Plane']],
+  // Axtell's red founders and blue members first; then preference, effort and pay.
+  firms: [
+    ['founder', 'Founder'],
+    ['theta', 'θ (income)'],
+    ['effort', 'Effort'],
+    ['income', 'Income'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -573,4 +589,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   bali: [],
   line: [],
   tipping: [],
+  firms: [],
 };
````

Modify `web/src/engine.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/engine.ts b/web/src/engine.ts
index a0418f3..9603ba1 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -54,6 +54,7 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
   if (modelOf(config) === 'farol') return `This run has reached its last round (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'ants' || modelOf(config) === 'thresholds') return `This run has reached its last step (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'firms') return `This run has reached its last period (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'bali') return `This run has reached its last year — Reset to run it again`;
   if (modelOf(config) === 'zi') return `This run has reached its last period — Reset to run it again`;
   if (modelOf(config) === 'punishment') return `This run has reached its last period (${tick}) — Reset to run it again`;
````

Modify `web/src/compare-presets.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index 02602dc..9c0461c 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -200,6 +200,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'lk-random',
     b: 'lk-random-fixed',
   },
+  {
+    id: 'firms-last-vs-live',
+    label: "Last period's effort vs live effort — The Emergence of Firms (Compare)",
+    a: 'firms-base',
+    b: 'firms-live',
+  },
 ];
 
 /**
````

Modify `web/src/experiments/form.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index 0322f03..781d386 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -104,6 +104,10 @@ export function defaultForm(model: ModelKind = 'sugarscape', config?: ModelConfi
     // The built-in ef-predictors' axis: how far attendance swings against predictors per agent.
     return { ...form, x: { path: 'strategies', values: '2:24:2' }, ticks: 2000, metric: { ...form.metric, kind: 'final', series: 'fluctuation' } };
   }
+  if (model === 'firms') {
+    // The built-in firms-beta's axis: the size exponent against increasing returns (A99 Table 3).
+    return { ...form, x: { path: 'beta', values: '1.7:2.1:0.1' }, ticks: 5000, metric: { ...form.metric, kind: 'final', series: 'mu' } };
+  }
   if (model === 'bali') {
     // The built-in bali-imitation-growth's axis: the scored harvest against pest growth.
     return { ...form, x: { path: 'growth', values: '2:2.4:0.1' }, ticks: 360, metric: { ...form.metric, kind: 'final', series: 'scored' } };
````

Modify `web/src/ui/series-data.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index 373a376..44f950d 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -735,6 +735,39 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
     { title: 'Like neighbors', lines: [{ key: 'like_share', label: 'Mean share alike', color: '--c3' }], range: [0, 1] },
     { title: 'Unsatisfied', lines: [{ key: 'unsatisfied', label: 'Unsatisfied share', color: '--red' }], range: [0, 1] },
   ],
+  firms: [
+    {
+      title: 'Firms',
+      lines: [
+        { key: 'firms', label: 'Firms', color: '--c1' },
+        { key: 'births', label: 'Founded this period', color: '--c3' },
+        { key: 'deaths', label: 'Dissolved this period', color: '--red' },
+      ],
+    },
+    {
+      title: 'Sizes',
+      lines: [
+        { key: 'mean_size', label: 'Mean size', color: '--c1' },
+        { key: 'largest', label: 'Largest firm', color: '--c2' },
+      ],
+    },
+    {
+      title: 'Effort and pay',
+      lines: [
+        { key: 'effort', label: 'Mean effort', color: '--c1' },
+        { key: 'income', label: 'Mean income', color: '--c2' },
+        { key: 'utility', label: 'Mean utility', color: '--c4' },
+      ],
+    },
+    { title: 'Output', lines: [{ key: 'output', label: 'Total output', color: '--c1' }] },
+    {
+      title: 'Scaling',
+      lines: [
+        { key: 'mu', label: 'µ (Axtell\'s OLS)', color: '--c1' },
+        { key: 'mu_mle', label: 'µ (maximum likelihood)', color: '--c4' },
+      ],
+    },
+  ],
   tipping: [
     {
       title: 'Inside',
@@ -758,7 +791,7 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
  * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'zi' ? 'Shouts' : model === 'bali' ? 'Months' : model === 'line' ? 'Rounds' : model === 'tipping' ? 'Steps' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
+  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'zi' ? 'Shouts' : model === 'bali' ? 'Months' : model === 'line' ? 'Rounds' : model === 'tipping' ? 'Steps' : model === 'firms' ? 'Periods' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
````

Modify `web/src/ui/inspect-panel.ts` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index 3c36088..5440c02 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -3,7 +3,7 @@ import { dpdRows } from '../dpd';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
 import { imageRows } from '../image-scoring';
-import { isAgreementView, isAntsView, isBaliView, isLineView, isTippingView, isPunishmentView, isZiView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { isAgreementView, isAntsView, isBaliView, isFirmsView, isLineView, isTippingView, isPunishmentView, isZiView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
@@ -15,6 +15,7 @@ import type {
   BaliInspection,
   LineInspection,
   TippingInspection,
+  FirmsInspection,
   RetirementInspection,
   ThresholdsInspection,
   FarolInspection,
@@ -231,6 +232,22 @@ export class InspectPanel {
     ];
   }
 
+  /** A firm and the member at the cell, or the size plot. */
+  private firmsRows(view: FirmsInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    if (view.panel === 'sizes') return [row('Plot', 'firm sizes since the burn-in, log-log, with the OLS fit')];
+    const f = view.firm;
+    if (!f) return [row('Row', 'no firm here')];
+    const rows = [
+      row('Firm', `#${f.id} · ${f.size} member${f.size === 1 ? '' : 's'} · age ${f.age}`),
+      row('Output', `${fmt(f.output)} (a ${fmt(f.a)}, b ${fmt(f.b)}, β ${fmt(f.beta)})`),
+      row('Members', `mean θ ${fmt(f.mean_theta)} · mean effort ${fmt(f.mean_effort)} · ${f.free_riders} free rider${f.free_riders === 1 ? '' : 's'}`),
+    ];
+    const m = view.member;
+    if (m) rows.push(row('Agent', `#${m.id} · θ ${fmt(m.theta)} · effort ${fmt(m.effort)} · income ${fmt(m.income)} · utility ${fmt(m.utility)} · tenure ${m.tenure}`));
+    return rows;
+  }
+
   /** A point of Schelling's tipping plane: the state, and whether each color's most tolerant would be content there. */
   private tippingRows(view: TippingInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -704,6 +721,8 @@ export class InspectPanel {
             ? this.normsRows(view)
           : isAgreementView(view)
             ? this.agreementRows(view)
+          : isFirmsView(view)
+            ? this.firmsRows(view)
           : isTippingView(view)
             ? this.tippingRows(view)
           : isLineView(view)
````

- [ ] **Step 4: Run the page's build and tests**

Run: `cd web && npm run build && npm test`
Expected: the build succeeds; 802 tests pass (50 files).

- [ ] **Step 5: Commit**

```bash
git add web/src
```
```bash
git commit -m "Carry The Emergence of Firms through the page

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

- [ ] **Step 6 (controller): check it in the browser**

Run `cd web && npm run build && npx vite preview`, then open the page with `?debug` and check:

- **`firms-base` in Founder colors:** after a few hundred periods, rows of firms with the largest on top. Founders are red, members blue, and the log-log size plot with its OLS line is on the right.
- **θ, Effort and Income colors** recolor the same rows.
- **`firms-sticky`:** one row spans the frame, which is the giant firm.
- **`firms-2013`:** 10 000 agents, with a heavy-tailed plot near slope −2.
- **Inspect:** a firm (size, output, age, members' θ and efforts, free riders), a member, and the sizes panel.
- **The Rules panel:** groups as listed. Changing `agents` waits for Reset, while changing `pay` applies at once.
- **Compare:** the Compare entry.
- **Experiments with a firms preset:** the default axis is β against `mu`. Run the built-in `firms-loyalty`.

---

### Task 4: The survey's firms claims

**Files:**
- Create: `survey/src/claims/firms.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `sugarscape_core::firms::{FirmsConfig, FirmsWorld, AdjustScope, SeniorityOrder, Pay, RandomBehavior, Network, Preferences, Activation, OthersEffort, EffortSearch, Initial}`, `firms::effort::{eigenvalue, max_stable_size, nash, optimal_size}`, `firms::fit::{gamma, growth_fit, lifetimes, mu_mle, mu_ols, output_exponent, productivity}`, `crate::claim::{all_of, greater, Claim, Outcome, Source, Verdict}`, `crate::runner::model_after`.
- Produces: 25 claims (`firms.a99.*`, `firms.a13.zipf`, `firms.ours.*`).

- [ ] **Step 1: Write the claims**

Create `survey/src/claims/firms.rs` with exactly this content:

````rust
//! The Emergence of Firms (milestone 30): Axtell (1999). The simulation
//! claims run 10 seeds × 5 000 periods (burn-in 500) of 1 000 agents unless
//! stated; µ is the paper's OLS on the log-log size pmf (size 1 and
//! frequencies below 10⁻⁵ dropped) unless the claim says maximum likelihood.
//! §4's tables are single estimates with no standard errors, and our base µ
//! is not the paper's, so each table is judged by direction (Mann–Whitney
//! between its extremes), with the paper's numbers beside ours.

use sugarscape_core::firms::effort::{eigenvalue, max_stable_size, nash, optimal_size};
use sugarscape_core::firms::fit::{
    gamma, growth_fit, lifetimes, mu_mle, mu_ols, output_exponent, productivity,
};
use sugarscape_core::firms::{
    Activation, AdjustScope, EffortSearch, FirmsConfig, FirmsWorld, Initial, Network, OthersEffort,
    Pay, Preferences, RandomBehavior, SeniorityOrder,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, greater, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const A99: &str = "Axtell 1999, Brookings CSED WP 3";

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

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

fn worlds(c: FirmsConfig, n: u64) -> Vec<FirmsWorld> {
    model_after(&ModelConfig::Firms(c), &seeds(n), 100_000, |w| match w {
        ModelWorld::Firms(f) => f.as_ref().clone(),
        _ => unreachable!(),
    })
}

fn base() -> FirmsConfig {
    FirmsConfig::default()
}

fn median(v: &[f64]) -> f64 {
    let mut s: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    s.sort_by(f64::total_cmp);
    if s.is_empty() {
        return f64::NAN;
    }
    let m = s.len() / 2;
    if s.len().is_multiple_of(2) {
        0.5 * (s[m - 1] + s[m])
    } else {
        s[m]
    }
}

fn mean(v: &[f64]) -> f64 {
    let f: Vec<f64> = v.iter().copied().filter(|x| x.is_finite()).collect();
    f.iter().sum::<f64>() / f.len().max(1) as f64
}

fn show(v: &[f64]) -> String {
    format!(
        "[{}]",
        v.iter()
            .map(|x| format!("{x:.2}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

/// Each seed's µ (the paper's OLS).
fn mus(ws: &[FirmsWorld]) -> Vec<f64> {
    ws.iter().map(|w| mu_ols(&w.records().sizes)).collect()
}

fn mus_of(c: FirmsConfig) -> Vec<f64> {
    mus(&worlds(c, 10))
}

/// A series' mean over the periods after the burn-in, per seed.
fn after_burn(ws: &[FirmsWorld], f: fn(&sugarscape_core::firms::FirmsSnapshot) -> f64) -> Vec<f64> {
    ws.iter()
        .map(|w| {
            let h = w.stats.history();
            let from = w.config.burn_in as usize + 1;
            mean(&h[from.min(h.len())..].iter().map(f).collect::<Vec<_>>())
        })
        .collect()
}

/// "µ falls from `lo_name` to `hi_name`": Mann–Whitney that `hi`'s µ are below `lo`'s.
fn falls(lo: Vec<f64>, hi: Vec<f64>, lo_name: &str, hi_name: &str, paper: &str) -> Outcome {
    greater(&lo, &hi, lo_name, hi_name).with(&format!(
        "medians {:.2} and {:.2}; the paper: {paper}",
        median(&lo),
        median(&hi)
    ))
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "firms.a99.table1",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Table 1 (θ = 0.7, a = b = 1): the Nash effort, utility and dominant eigenvalue (N − 1)k for N = 1–7 — each to 3 decimal places; stable to 6, optimal at 5",
            check: |_| {
                let rows = [(1, 0.770, 0.799, 0.0), (2, 0.646, 0.964, -0.188), (3, 0.558, 1.036, -0.368), (4, 0.492, 1.065, -0.547), (5, 0.441, 1.069, -0.726), (6, 0.399, 1.061, -0.904), (7, 0.364, 1.045, -1.082)];
                let ok = rows.iter().all(|&(n, e, u, l)| {
                    let (ne, nu) = nash(0.7, n, 1.0, 1.0);
                    (ne - e).abs() < 5e-4 && (nu - u).abs() < 5e-4 && (n == 1 || (eigenvalue(0.7, n, 1.0, 1.0) - l).abs() < 5e-4)
                });
                let sizes = (max_stable_size(0.7, 1.0, 1.0, 50), optimal_size(0.7, 1.0, 1.0, 50));
                outcome(ok && sizes == (6, 5), format!("every row within 0.0005; stable to {}, optimal {}", sizes.0, sizes.1))
                    .with("The text's eigenvalue for N = 3, −0.552, is not the table's −0.368 (the table is right).")
            },
        },
        Claim {
            id: "firms.a99.sizes",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Figs. 6 and 9: 'Optimal group sizes are relatively small—less than 10—for agents having θ < 0.85, then rise quickly', and 'the optimal size of a homogeneous group is very nearly at the stability boundary' — optimal size under 10 at θ = 0.84, at least 30 at θ = 0.95, and the maximum stable size within 1 of the optimum for θ from 0.3 to 0.95",
            check: |_| {
                let grid: Vec<f64> = (0..14).map(|k| 0.3 + 0.05 * f64::from(k)).collect();
                let pairs: Vec<(u32, u32)> = grid.iter().map(|&t| (optimal_size(t, 1.0, 1.0, 400), max_stable_size(t, 1.0, 1.0, 400))).collect();
                let near = pairs.iter().all(|&(o, s)| s >= o && s - o <= 1);
                let (o84, o95) = (optimal_size(0.84, 1.0, 1.0, 400), optimal_size(0.95, 1.0, 1.0, 400));
                outcome(near && o84 < 10 && o95 >= 30, format!("optimal 0.84 → {o84}, 0.95 → {o95}; (optimal, stable) θ 0.3–0.95: {pairs:?}"))
            },
        },
        Claim {
            id: "firms.a99.mu",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "§3.4: firm sizes follow a power law with 'µ = 1.28' (OLS, adjusted R² 0.99) — the median over 10 seeds within 0.15 of 1.28 (0.15: about the spread of Table 11's four identical rows)",
            check: |_| {
                let ws = worlds(base(), 10);
                let (m, mle) = (mus(&ws), ws.iter().map(|w| mu_mle(&w.records().sizes, 2)).collect::<Vec<_>>());
                outcome((median(&m) - 1.28).abs() <= 0.15, format!("OLS µ by seed {} (median {:.2}); maximum likelihood {} (median {:.2})", show(&m), median(&m), show(&mle), median(&mle)))
            },
        },
        Claim {
            id: "firms.a99.output",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Fig. 18: firm output follows a power law with exponent 0.88; fig. 19: productivity 0.58·s^1.15, 'the hypothesis of constant returns cannot be rejected' — the output exponent within 0.15 of 0.88, and the productivity exponent between 0.9 and 1.25 (medians, 10 seeds)",
            check: |_| {
                let ws = worlds(base(), 10);
                let out: Vec<f64> = ws.iter().map(|w| output_exponent(&w.records().outputs)).collect();
                let prod: Vec<f64> = ws.iter().map(|w| productivity(&w.records().size_output).1).collect();
                all_of(vec![
                    ("output exponent".into(), outcome((median(&out) - 0.88).abs() <= 0.15, format!("{} (median {:.2})", show(&out), median(&out)))),
                    ("productivity".into(), outcome((0.9..=1.25).contains(&median(&prod)), format!("{} (median {:.2})", show(&prod), median(&prod)))),
                ])
            },
        },
        Claim {
            id: "firms.a99.growth",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "Fig. 20: growth rates are better fit by a Laplace than a Gaussian; fig. 21: σ_r ∝ s^−γ with 'γ = 0.174 ± 0.004' — the Laplace's log-likelihood higher in at least 8 of 10 seeds, and γ (sizes 3–300, at least 30 observations each) within 0.008 (twice the reported error) of 0.174",
            check: |_| {
                let ws = worlds(base(), 10);
                let fits: Vec<_> = ws.iter().map(|w| growth_fit(&w.records().growth)).collect();
                let laplace = fits.iter().filter(|f| f.laplace_ll > f.gauss_ll).count();
                let g: Vec<f64> = ws.iter().map(|w| gamma(&w.records().growth_by_size, 3, 300, 30)).collect();
                all_of(vec![
                    ("Laplace".into(), outcome(laplace >= 8, format!("{laplace} of 10 seeds; mean sd of r {:.3}", mean(&fits.iter().map(|f| f.sd).collect::<Vec<_>>())))),
                    ("γ".into(), outcome((median(&g) - 0.174).abs() <= 0.008, format!("{} (median {:.3})", show(&g), median(&g)))),
                ])
            },
        },
        Claim {
            id: "firms.a99.lifetimes",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "p. 46–47: firm lifetimes average 23.4 periods (sd 27.1), and lifetime is linear in log rank with slope about −70 — the mean within 25 % of 23.4 and the slope between −105 and −35 (medians, 10 seeds)",
            check: |_| {
                let ws = worlds(base(), 10);
                let l: Vec<(u64, f64, f64, f64)> = ws.iter().map(|w| lifetimes(&w.records().lifetimes)).collect();
                let team: Vec<f64> = ws.iter().map(|w| lifetimes(&w.records().lifetimes_team).1).collect();
                let (m, slope) = (median(&l.iter().map(|x| x.1).collect::<Vec<_>>()), median(&l.iter().map(|x| x.3).collect::<Vec<_>>()));
                outcome((m - 23.4).abs() <= 0.25 * 23.4 && (-105.0..=-35.0).contains(&slope), format!("mean {m:.1} (sd {:.1}); firms that ever had two members {:.1}; rank slope {slope:.1}", median(&l.iter().map(|x| x.2).collect::<Vec<_>>()), median(&team)))
                    .with("The paper's own counts disagree: about 45 births a period × 23.4 periods would need about 1 050 firms of 1 000 agents.")
            },
        },
        Claim {
            id: "firms.a99.levels",
            item: "firms-base",
            source: Source::Book,
            citation: A99,
            text: "§3.3–3.4: 'The average firm size is about 4', the largest firm reaches about 200, and figures 14–16 show total output of about 450–600 — mean size between 3 and 5, the largest firm at least 150 in at least 5 of 10 seeds, and output between 450 and 600 (means after the burn-in)",
            check: |_| {
                let ws = worlds(base(), 10);
                let size = after_burn(&ws, |s| s.mean_size);
                let largest: Vec<f64> = ws.iter().map(|w| w.stats.history().iter().skip(w.config.burn_in as usize).map(|s| f64::from(s.largest)).fold(0.0, f64::max)).collect();
                let out = after_burn(&ws, |s| s.output);
                all_of(vec![
                    ("mean size".into(), outcome((3.0..=5.0).contains(&median(&size)), format!("{} (median {:.2})", show(&size), median(&size)))),
                    ("largest".into(), outcome(largest.iter().filter(|&&x| x >= 150.0).count() >= 5, show(&largest))),
                    ("output".into(), outcome((450.0..=600.0).contains(&median(&out)), format!("{} (median {:.0})", show(&out), median(&out)))),
                ])
                .with("With everyone alone at the start, output is already about 934 (mean singleton output 0.934 × 1 000).")
            },
        },
        Claim {
            id: "firms.a99.beta",
            item: "firms-beta",
            source: Source::Book,
            citation: A99,
            text: "Table 3: µ falls as increasing returns β rise (2.06 at β 1.7 to 0.95 at 2.1) — µ at β 1.7 above µ at β 2.1",
            check: |_| falls(mus_of(FirmsConfig { beta: 1.7, ..base() }), mus_of(FirmsConfig { beta: 2.1, ..base() }), "β 1.7", "β 2.1", "2.06 and 0.95"),
        },
        Claim {
            id: "firms.a99.b",
            item: "firms-b",
            source: Source::Book,
            citation: A99,
            text: "Table 4: µ falls as b rises (2.09 at b 0.5 to 0.53 at 1.5), and b drawn per firm from [0.5, 1.5] 'behaves much more like b = 1.25, not b = 1.0' — µ at b 0.5 above µ at 1.5, and the drawn b's median µ nearer b 1.25's than b 1.0's",
            check: |_| {
                let at = |b: f64| mus_of(FirmsConfig { b, ..base() });
                let (lo, hi, one, one25) = (at(0.5), at(1.5), at(1.0), at(1.25));
                let drawn = mus_of(FirmsConfig { b: 0.5, b_max: 1.5, ..base() });
                let nearer = (median(&drawn) - median(&one25)).abs() < (median(&drawn) - median(&one)).abs();
                all_of(vec![
                    ("falls".into(), falls(lo, hi, "b 0.5", "b 1.5", "2.09 and 0.53")),
                    ("drawn b".into(), outcome(nearer, format!("drawn {:.2}; b 1.0 {:.2}; b 1.25 {:.2} (the paper: 0.89, 1.28, 0.91)", median(&drawn), median(&one), median(&one25)))),
                ])
            },
        },
        Claim {
            id: "firms.a99.preferences",
            item: "firms-preferences",
            source: Source::Book,
            citation: A99,
            text: "Table 5: 'the general power-law character … remain[s]' across preference distributions, and the homogeneous θ = 0.75 gives more large firms (0.91 against 1.28) — every row's median µ (OLS) between 0.5 and 3, and µ at θ = 0.75 below θ ~ U[0, 1]'s",
            check: |_| {
                let rows: [(&str, FirmsConfig); 9] = [
                    ("uniform", base()),
                    ("middle", FirmsConfig { preferences: Preferences::Middle, ..base() }),
                    ("triangular", FirmsConfig { preferences: Preferences::Triangular, ..base() }),
                    ("triangular 0.75", FirmsConfig { preferences: Preferences::TriangularHigh, ..base() }),
                    ("normal", FirmsConfig { preferences: Preferences::Normal, ..base() }),
                    ("beta", FirmsConfig { preferences: Preferences::Beta, ..base() }),
                    ("θ 0.75", FirmsConfig { preferences: Preferences::Fixed, theta: 0.75, ..base() }),
                    ("CES ρ −1–0", FirmsConfig { preferences: Preferences::Ces, rho: -1.0, rho_max: 0.0, ..base() }),
                    ("CES ρ 0–10", FirmsConfig { preferences: Preferences::Ces, rho: 0.0, rho_max: 10.0, ..base() }),
                ];
                let results: Vec<(&str, Vec<f64>)> = rows.into_iter().map(|(n, c)| (n, mus_of(c))).collect();
                let medians: Vec<f64> = results.iter().map(|(_, m)| median(m)).collect();
                let character = medians.iter().all(|m| (0.5..=3.0).contains(m));
                all_of(vec![
                    ("power-law character".into(), outcome(character, format!("medians {} (the paper: 1.28, 1.21, 1.31, 1.01, 1.30, 0.99, 0.91, 1.56, 1.26)", show(&medians)))),
                    ("homogeneous".into(), falls(results[0].1.clone(), results[6].1.clone(), "uniform", "θ 0.75", "1.28 and 0.91")),
                ])
            },
        },
        Claim {
            id: "firms.a99.networks",
            item: "firms-friends",
            source: Source::Book,
            citation: A99,
            text: "Tables 6–7: larger networks 'stabilize large firms' — µ at ν 2 above µ at ν 10, for fixed friends (1.28 to 0.99) and for random firms (1.28 to 1.03)",
            check: |_| {
                let f = |network, neighbors| mus_of(FirmsConfig { network, neighbors, ..base() });
                all_of(vec![
                    ("friends".into(), falls(f(Network::Friends, 2), f(Network::Friends, 10), "ν 2", "ν 10", "1.28 and 0.99")),
                    ("random firms".into(), falls(f(Network::RandomFirms, 2), f(Network::RandomFirms, 10), "ν 2", "ν 10", "1.28 and 1.03")),
                ])
            },
        },
        Claim {
            id: "firms.a99.loyalty",
            item: "firms-loyalty",
            source: Source::Book,
            citation: A99,
            text: "Table 8: 'loyalty is a stabilizing factor for large firms … relatively strong effect' — µ at λ 0 above µ at λ 10 (1.28 and 0.77)",
            check: |_| falls(mus_of(base()), mus_of(FirmsConfig { loyalty: 10, ..base() }), "λ 0", "λ 10", "1.28 and 0.77"),
        },
        Claim {
            id: "firms.a99.sticky-groping",
            item: "firms-sticky",
            source: Source::Book,
            citation: A99,
            text: "Tables 9–10 (β 2): sticky effort (±0.05) and groping both lower µ (1.28 to 0.92 and 1.19), groping's effect 'more pronounced' (the text; the tables say less) — µ free above µ sticky, µ free above µ groping, and µ groping below µ sticky (the text's claim)",
            check: |_| {
                let (free, sticky, groping) = (mus_of(base()), mus_of(FirmsConfig { effort_window: 0.1, ..base() }), mus_of(FirmsConfig { groping: true, ..base() }));
                all_of(vec![
                    ("sticky".into(), falls(free.clone(), sticky.clone(), "free", "sticky", "1.28 and 0.92")),
                    ("groping".into(), falls(free, groping.clone(), "free", "groping", "1.28 and 1.19")),
                    ("more pronounced".into(), falls(sticky, groping, "sticky", "groping", "0.92 and 1.19 — the table contradicts the text")),
                ])
            },
        },
        Claim {
            id: "firms.a99.seniority",
            item: "firms-seniority",
            source: Source::Book,
            citation: A99,
            text: "Table 11: seniority shares 'make large firms somewhat more stable' (5^−rank 1.11 against equal shares 1.28) — µ under equal shares above µ under 5^−rank",
            check: |_| falls(mus_of(base()), mus_of(FirmsConfig { pay: Pay::Seniority, seniority_base: 5.0, ..base() }), "equal", "5^−rank", "1.28 and 1.11"),
        },
        Claim {
            id: "firms.a99.base-pay",
            item: "firms-base-pay",
            source: Source::Book,
            citation: A99,
            text: "Table 12: base pay 'stabilize[s] large firms somewhat' (80 % of singleton income: 0.85 against 1.28) — µ under equal shares above µ with base pay at 80 % of each agent's singleton income",
            check: |_| falls(mus_of(base()), mus_of(FirmsConfig { pay: Pay::Base, base_share: 0.8, ..base() }), "equal", "base 80 %", "1.28 and 0.85"),
        },
        Claim {
            id: "firms.a99.hiring",
            item: "firms-hiring",
            source: Source::Book,
            citation: A99,
            text: "Table 13 (captioned 'target output', a rule the paper never gives; the rows are hiring standards): standards 'first stabilize large firms to some extent … and then … destabilize them', and at φ = 100 % the power law 'breaks down' — µ at φ 60 % below µ at 0 % and the largest firm at φ 100 % (median over seeds) under 20",
            check: |_| {
                let at = |h: f64| worlds(FirmsConfig { hiring: h, ..base() }, 10);
                let (zero, sixty, full) = (at(0.0), at(0.6), at(1.0));
                let largest: Vec<f64> = full.iter().map(|w| w.stats.history().iter().map(|s| f64::from(s.largest)).fold(0.0, f64::max)).collect();
                all_of(vec![
                    ("stabilize".into(), falls(mus(&zero), mus(&sixty), "φ 0", "φ 60 %", "1.28 and 1.03")),
                    ("breaks down".into(), outcome(median(&largest) < 20.0, format!("largest firm at φ 100 %: {}", show(&largest)))),
                ])
            },
        },
        Claim {
            id: "firms.a99.random",
            item: "firms-random-choices",
            source: Source::Book,
            citation: A99,
            text: "§4.1: random choices 'fail to yield a power law … firms greater than 9 or 10 are rarely observed', and random effort gives 'nothing like power law size distributions' — under 1 % of sampled firms larger than 10, for each (median over 10 seeds)",
            check: |_| {
                let share = |c: FirmsConfig| {
                    worlds(c, 10)
                        .iter()
                        .map(|w| {
                            let s = &w.records().sizes;
                            let total: u64 = s.iter().sum();
                            let largest = w.stats.history().iter().map(|x| x.largest).max().unwrap_or(0);
                            (s.iter().skip(11).sum::<u64>() as f64 / total.max(1) as f64, f64::from(largest))
                        })
                        .collect::<Vec<(f64, f64)>>()
                };
                let (cl, el) = (share(FirmsConfig { random_behavior: RandomBehavior::Choices, ..base() }), share(FirmsConfig { random_behavior: RandomBehavior::Effort, ..base() }));
                let (choices, effort): (Vec<f64>, Vec<f64>) = (cl.iter().map(|x| x.0).collect(), el.iter().map(|x| x.0).collect());
                let largest: Vec<f64> = el.iter().map(|x| x.1).collect();
                all_of(vec![
                    ("random choices".into(), outcome(median(&choices) < 0.01, format!("share above 10: {}", show(&choices.iter().map(|x| 100.0 * x).collect::<Vec<_>>())) + " %")),
                    ("random effort".into(), outcome(median(&effort) < 0.01, format!("share above 10: {}", show(&effort.iter().map(|x| 100.0 * x).collect::<Vec<_>>())) + " %")),
                ])
                .with(&format!("Under random effort the largest firm reaches {} of 1 000 agents: not a power law, but of the opposite kind — agents gather in giant firms.", show(&largest)))
            },
        },
        Claim {
            id: "firms.a99.invariances",
            item: "firms-readings",
            source: Source::Book,
            citation: A99,
            text: "§4 introduction: population size ('invariant'), uniform against random activation ('essentially no difference') and the initial condition ('did not seem to matter much') leave µ alone — each within 0.15 of the base case's median µ (10 000 agents: 3 seeds, 2 000 periods)",
            check: |_| {
                let b = median(&mus_of(base()));
                let near = |name: &str, m: f64| (name.to_string(), outcome((m - b).abs() <= 0.15, format!("{m:.2} against {b:.2}")));
                let big = median(&mus(&worlds(FirmsConfig { agents: 10_000, stop_at: 2000, ..base() }, 3)));
                let small = median(&mus(&worlds(FirmsConfig { stop_at: 2000, ..base() }, 3)));
                all_of(vec![
                    ("10 000 agents".into(), outcome((big - small).abs() <= 0.15, format!("{big:.2} against {small:.2} at 1 000 (2 000 periods)"))),
                    near("uniform activation", median(&mus_of(FirmsConfig { activation: Activation::Uniform, ..base() }))),
                    near("random groups", median(&mus_of(FirmsConfig { initial: Initial::RandomGroups, ..base() }))),
                    near("one firm", median(&mus_of(FirmsConfig { initial: Initial::OneFirm, ..base() }))),
                ])
            },
        },
        Claim {
            id: "firms.ours.readings",
            item: "firms-readings",
            source: Source::Comment,
            citation: A99,
            text: "Ours: no reading of the unstated rules gives µ = 1.28 — the median µ over 10 seeds more than 0.15 from 1.28 under the literal reading, live efforts, uniform activation, both, and a coarse line search",
            check: |_| {
                let cases: [(&str, FirmsConfig); 5] = [
                    ("literal", base()),
                    ("live", FirmsConfig { others_effort: OthersEffort::Live, ..base() }),
                    ("uniform", FirmsConfig { activation: Activation::Uniform, ..base() }),
                    ("live and uniform", FirmsConfig { others_effort: OthersEffort::Live, activation: Activation::Uniform, ..base() }),
                    ("grid of 10", FirmsConfig { effort_search: EffortSearch::Grid, grid_steps: 10, ..base() }),
                ];
                all_of(cases.into_iter().map(|(n, c)| {
                    let m = median(&mus_of(c));
                    (n.to_string(), outcome((m - 1.28).abs() > 0.15, format!("{m:.2}")))
                }).collect())
            },
        },
        Claim {
            id: "firms.ours.estimators",
            item: "firms-base",
            source: Source::Comment,
            citation: A99,
            text: "Ours: the paper's OLS and a maximum-likelihood fit disagree by more than 0.5 on the same sizes (medians, 10 seeds) — the distribution is not one clean power law",
            check: |_| {
                let ws = worlds(base(), 10);
                let (o, m) = (median(&mus(&ws)), median(&ws.iter().map(|w| mu_mle(&w.records().sizes, 2)).collect::<Vec<_>>()));
                outcome((o - m).abs() > 0.5, format!("OLS {o:.2}, maximum likelihood {m:.2}"))
            },
        },
        Claim {
            id: "firms.ours.noise",
            item: "firms-seniority",
            source: Source::Comment,
            citation: A99,
            text: "Ours: Table 11's rows 2^−rank, 2^−(rank+1), 2^−(rank+2) and 2^−(rank+3) are one model (normalized shares are identical), so their spread (0.89–1.07) is the paper's run-to-run noise; ours, seed to seed at the base case, is at least as large (the range of 10 seeds at least 0.18)",
            check: |_| {
                let m = mus_of(base());
                let range = m.iter().copied().fold(f64::MIN, f64::max) - m.iter().copied().fold(f64::MAX, f64::min);
                outcome(range >= 0.18, format!("range {range:.2} over {}", show(&m)))
            },
        },
        Claim {
            id: "firms.ours.myopic",
            item: "firms-base",
            source: Source::Comment,
            citation: A99,
            text: "Ours: fn 19 says that under constant returns working together is never individually rational — at equilibrium; myopic agents who take others' effort as given still join (with b = 0 and every θ = 0.75, firms of two or more form in every seed)",
            check: |_| {
                let ws = worlds(FirmsConfig { b: 0.0, preferences: Preferences::Fixed, theta: 0.75, stop_at: 200, ..base() }, 10);
                let formed = ws.iter().filter(|w| w.stats.history().iter().any(|s| s.largest >= 2)).count();
                outcome(formed == 10, format!("{formed} of 10 seeds"))
            },
        },
        Claim {
            id: "firms.a13.zipf",
            item: "firms-2013",
            source: Source::Book,
            citation: "Axtell 2013, Endogenous Dynamics of Firms and Labor",
            text: "Axtell's 2013 parameterization (per-firm a, b and β; 2–6 friends; 4 % activated a period) gives Zipf-distributed firm sizes, α ≈ 1.06 — at 10 000 agents, the median µ (the paper's OLS) within 0.15 of 1.06 (3 seeds × 2 000 periods)",
            check: |_| {
                let c = FirmsConfig {
                    agents: 10_000,
                    a: 0.0,
                    a_max: 0.5,
                    b: 0.75,
                    b_max: 1.25,
                    beta: 1.5,
                    beta_max: 2.0,
                    neighbors: 2,
                    neighbors_max: 6,
                    activation_rate: 0.04,
                    stop_at: 2000,
                    ..base()
                };
                let ws = worlds(c, 3);
                let m = mus(&ws);
                let largest: Vec<f64> = ws.iter().map(|w| w.stats.history().iter().map(|s| f64::from(s.largest)).fold(0.0, f64::max)).collect();
                outcome((median(&m) - 1.06).abs() <= 0.15, format!("µ {} (median {:.2}); the largest firm {}", show(&m), median(&m), show(&largest)))
                    .with("The rule was written after the planning runs had measured µ 0.98–1.05.")
            },
        },
        Claim {
            id: "firms.ours.adjust-scope",
            item: "firms-sticky",
            source: Source::Comment,
            citation: A99,
            text: "Ours: whether sticky effort and groping apply outside the agent's own firm (A99 does not say) decides the outcome — applied in any firm, the whole population is in one firm at some point in at least 8 of 10 seeds; applied only at home, the largest firm stays below half the population in every seed",
            check: |_| {
                let reached = |c: FirmsConfig| worlds(c, 10).iter().map(|w| w.stats.history().iter().map(|s| s.largest).max().unwrap_or(0)).collect::<Vec<u32>>();
                let parts: Vec<(String, Outcome)> = [("sticky", FirmsConfig { effort_window: 0.1, ..base() }), ("groping", FirmsConfig { groping: true, ..base() })]
                    .into_iter()
                    .flat_map(|(name, c)| {
                        let any = reached(c.clone());
                        let home = reached(FirmsConfig { adjust_scope: AdjustScope::OwnFirm, ..c });
                        [
                            (format!("{name}, any firm"), outcome(any.iter().filter(|&&x| x == 1000).count() >= 8, format!("largest {any:?}"))),
                            (format!("{name}, own firm"), outcome(home.iter().all(|&x| x < 500), format!("largest {home:?}"))),
                        ]
                    })
                    .collect();
                all_of(parts).with("The rule was written after the planning runs.")
            },
        },
        Claim {
            id: "firms.ours.seniority-order",
            item: "firms-seniority",
            source: Source::Comment,
            citation: A99,
            text: "Ours: Table 11's modest effect (µ 1.11 at 5^−rank) comes from neither ordering of seniority shares — paying the longest-serving most (the text), no firm grows past 10 in any seed; paying the newest most, the median µ is below 0.5",
            check: |_| {
                let senior = worlds(FirmsConfig { pay: Pay::Seniority, seniority_base: 5.0, ..base() }, 10);
                let junior = mus_of(FirmsConfig { pay: Pay::Seniority, seniority_base: 5.0, seniority_order: SeniorityOrder::JuniorFirst, ..base() });
                let largest: Vec<u32> = senior.iter().map(|w| w.stats.history().iter().map(|s| s.largest).max().unwrap_or(0)).collect();
                all_of(vec![
                    ("senior first".into(), outcome(largest.iter().all(|&x| x <= 10), format!("largest {largest:?}"))),
                    ("newest first".into(), outcome(median(&junior) < 0.5, format!("µ {} (median {:.2})", show(&junior), median(&junior)))),
                ])
                .with("The rule was written after the planning runs.")
            },
        },
    ]
}
````

Modify `survey/src/claims/mod.rs` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index 6598d8e..99d79ee 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -12,6 +12,7 @@ mod culture;
 mod dpd;
 mod ethno;
 mod farol;
+mod firms;
 mod image;
 mod minds1;
 mod minds2;
@@ -48,6 +49,7 @@ pub fn all() -> Vec<Claim> {
         dpd::claims(),
         ethno::claims(),
         farol::claims(),
+        firms::claims(),
         norms::claims(),
         image::claims(),
         minds1::claims(),
````

- [ ] **Step 2: Run them**

Run: `cd survey && rustfmt --edition 2021 src/claims/firms.rs && cargo build --release && ./target/release/survey --only firms.`
Expected: 25 claims. The run takes tens of minutes, because each §4 claim runs 10 seeds × 5 000 periods.

- **15 Holds:**
  - a99.table1: every row within 0.0005;
  - a99.growth: Laplace in 10 of 10 seeds, γ median 0.171;
  - a99.beta: 3.64 → 2.00;
  - a99.b: 3.37 → 2.06, with drawn b like 1.25;
  - a99.loyalty: 2.49 → 1.91;
  - a99.sticky-groping: −0.11 and −0.23;
  - a99.base-pay: 2.49 → 1.71;
  - a99.invariances;
  - ours.readings;
  - ours.estimators: OLS 2.49, maximum likelihood 1.44;
  - ours.noise: range 0.50;
  - ours.myopic: 10 of 10;
  - a13.zipf: µ median 1.00;
  - ours.adjust-scope;
  - ours.seniority-order.
- **10 Fails:**
  - a99.sizes: optimal 11 at θ 0.84;
  - a99.mu: OLS median 2.49 against 1.28;
  - a99.output: exponent 2.02, productivity 0.67;
  - a99.lifetimes: 3.9 against 23.4;
  - a99.levels: mean size 2.30, largest 62–150, output 749;
  - a99.preferences: the spread;
  - a99.networks: friends reverse (2.49 → 2.64);
  - a99.seniority: 10.03;
  - a99.hiring: no breakdown at 100 %;
  - a99.random: random effort gives one firm.

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/firms.rs survey/src/claims/mod.rs
```
```bash
git commit -m "Survey The Emergence of Firms: the analytics, the base case's scaling, each section 4 table, Axtell 2013's Zipf law and the readings that decide them

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

---

### Task 5: README, roadmap, papers index, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md`

- [ ] **Step 1: Write the docs**

Modify `README.md` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/README.md b/README.md
index 4122fbc..0300bf2 100644
--- a/README.md
+++ b/README.md
@@ -155,7 +155,7 @@ The presets menu groups its presets by model: **Sugarscape**, **Minds**, **Schel
 **Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence**, **Social Structure**,
 **Demographic PD**, **Norms and Metanorms**, **Relative Agreement**,
 **Image Scoring**, **El Farol and the Minority Game**, **Ants and Recruitment**, **Threshold Models**,
-**The Timing of Retirement**, **Altruistic Punishment**, **Zero-Intelligence Traders** and **Balinese Water Temples**.
+**The Timing of Retirement**, **Altruistic Punishment**, **Zero-Intelligence Traders**, **Balinese Water Temples** and **The Emergence of Firms**.
 Each preset is listed by a plain title saying what happens in it; under the menu, the chosen
 preset's source (the book's figure or animation, or the paper) and its rules sit above its description.
 Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
@@ -2709,6 +2709,83 @@ An Analysis of the Lansing–Kremer Model of Bali," *Agricultural Systems* 93: 1
 watershed data from Janssen's "Lansing–Kremer model" (CoMSES Net 2221, v1.2.0, GPL-2.0). See
 `docs/superpowers/specs/2026-09-28-bali-water-temples-design.md`.
 
+### The Emergence of Firms (Axtell 1999; Axtell 2013)
+
+**The model.** Why do firms exist, and why are their sizes spread like a power law? Axtell's agents
+each prefer income and leisure in their own proportion (θ, uniform on [0, 1]). A firm's output grows
+faster than its members' total effort (a·E + b·E², increasing returns), and is shared equally — so
+working together pays, but each member's share is only weakly tied to its own effort, and free
+riders creep in. Each period every agent, activated at random, weighs re-choosing its effort at
+home, starting a firm alone, and joining one of its two friends' firms, and takes the best. Firms
+form, grow, fill with free riders and collapse. One tick is a period.
+
+**How the paper was read.** It is a 108-page working paper with the rules in prose, a text layer, and
+no code. Its §2 analytics — each agent's best effort (a closed form), the Nash equilibria of a group
+of like agents, the size beyond which that equilibrium turns unstable (Table 1) — are exact, and
+reproduce to three decimals. Its simulation leaves much unstated, and each gap is a switch: what an
+agent sees of others' effort (last period's, inferred from output, as the text says; or live), how
+agents are activated (at random with replacement, or once each), how finely effort is searched, and
+for §4: whether sticky effort and groping apply in a firm an agent joins or founds, and which way
+seniority pay runs. Its own figures, tables and text disagree in places — firm counts, mean sizes
+and lifetimes that can't all be true of 1 000 agents; output levels half of what the all-alone start
+already gives; four rows of Table 11 that are one model (so their spread, 0.89–1.07, is the paper's
+noise); text that says a table rises where it falls; and a table captioned for a rule ("target
+output") that appears nowhere. Axtell's 2013 paper recasts the model (per-firm a, b and β; 2–6
+friends; 4 % of agents activated a period, a month) and is a preset.
+
+Measured (the survey — 25 claims, 15 hold and 10 fail — and the presets' and sweeps' descriptions):
+
+- **The base case does not reproduce the paper's numbers, under any reading.** Firm sizes fall off
+  with µ ≈ 2.5 by Axtell's own OLS (1.4 by maximum likelihood), not 1.28; firms live about 4
+  periods, not 23.4; the largest firm reaches 60–140 (his typical run: about 205); output runs
+  near the all-alone level, 750, not his 450–600; productivity shows decreasing returns (output ∝
+  size^0.67), not his near-constant ones (s^1.15). Reading others' effort live and activating each
+  agent once comes closest (µ 1.79). Population size and the start don't matter, as he says.
+- **But the growth of firms reproduces.** Growth rates are Laplace-distributed, not Gaussian (10 of
+  10 seeds), and their spread falls with size as s^−0.171 — his γ is 0.174 ± 0.004.
+- **Most of §4's directions hold, one reverses.** µ falls as increasing returns (β, b) strengthen,
+  as random-firm networks widen, as loyalty grows and as base pay rises, as his tables say, and
+  hiring standards first steady large firms and then undo them — all about 1 above his values.
+  More fixed friends *raise* µ, where his Table 6 lowers it. Loyalty is the one change that gives
+  his long lifetimes (about 35 periods at λ = 10).
+- **Two unstated details decide whole tables.** Sticky effort (±0.05 a step) and groping, applied to
+  effort in any firm, pull the whole population into one firm at times, in every seed; applied only
+  at home, they barely matter. Seniority pay with the founder paid most (the text's order) stops
+  firms forming (a joiner's share of a pair's output under 5^−rank is 1/31); with the newest paid
+  most, near-giant firms form. His modest Table 11 effect comes from neither.
+- **Random behavior fails differently than he says.** Random choices of firm keep every firm small,
+  as he reports; random effort pulls all 1 000 agents into one firm in every seed — not a power law,
+  as he says, but of the opposite kind.
+- **Myopic agents join even without increasing returns.** His footnote 19 says working together
+  never pays under constant returns — true at equilibrium; an agent taking others' effort as given
+  still gains by joining an identical one, and firms form.
+- **His 2013 parameterization gives Zipf's law.** µ ≈ 1.0 (his 2013 α ≈ 1.06), with firms living
+  about 77 periods — the famous result belongs to the later model, not the 1999 one.
+
+Switches: **Agents** and **Start**; **Constant returns a**, **Increasing returns b**, **Exponent β**
+(each fixed, or drawn per firm up to a maximum); **Preferences** (nine distributions, including CES
+with **CES exponent** convention); **Agents look at** (friends' firms or random firms) and **Friends
+or firms ν**; **Activation**, **Activations a period ÷ agents**, **Others' effort**, **Best effort**,
+**Effort changes by at most**, **Grope for effort**, **Sticky effort and groping apply**, **Loyalty
+λ**, **Random behavior**; **Output is shared** (equally, by seniority with **The largest share goes
+to**, or base pay plus a bonus), **Hiring standard φ**; and the measurement's burn-in, sampling and
+stop. The view follows Axtell's Animation 1 — each firm a row, its longest-serving member first, the
+largest firms first — beside the firm-size distribution on log-log axes with its OLS line. Color modes: **Founder**, **θ
+(income)**, **Effort**, **Income**. Charts: Firms; Sizes; Effort and pay; Output; Scaling. Presets:
+`firms-base`, `firms-live`, `firms-uniform`, `firms-beta-17`, `firms-beta-21`, `firms-b-15`,
+`firms-b-random`, `firms-theta-075`, `firms-friends-10`, `firms-random-firms-10`, `firms-loyal-10`,
+`firms-sticky`, `firms-groping`, `firms-seniority-5`, `firms-base-pay-80`, `firms-hiring-100`,
+`firms-random-choices`, `firms-2013`. **Compare** entry: "Last period's effort vs live effort — The
+Emergence of Firms (Compare)". Built-in sweeps: `firms-beta`, `firms-b`, `firms-preferences`,
+`firms-friends`, `firms-random-firms`, `firms-loyalty`, `firms-sticky`, `firms-groping`,
+`firms-seniority`, `firms-base-pay`, `firms-hiring`, `firms-readings`, `firms-population`.
+
+Credit: Robert L. Axtell, "The Emergence of Firms in a Population of Agents: Local Increasing
+Returns, Unstable Nash Equilibria, and Power Law Size Distributions," Brookings Institution CSED
+Working Paper 3 (1999); Robert L. Axtell, "Endogenous Dynamics of Firms and Labor with Large Numbers
+of Simple Agents" (working paper, 2013). See
+`docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md`.
+
 ## Experiments
 
 The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
````

Modify `docs/roadmap.md` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/docs/roadmap.md b/docs/roadmap.md
index c7c6011..bf14506 100644
--- a/docs/roadmap.md
+++ b/docs/roadmap.md
@@ -291,6 +291,18 @@ physical one — never Janssen's +30 %, and the temple scale is never best; his
 exactly.
 See `docs/superpowers/specs/2026-09-28-bali-water-temples-design.md`.
 
+## Milestone 30: The Emergence of Firms (done)
+
+Axtell's emergence of firms (Brookings working paper, 1999) as a model kind, with his 2013
+parameterization: agents with preferences between income and leisure choose their effort in teams
+with increasing returns and equal shares, moving between their firm, a start-up and their friends'
+firms; every variation of his §4 and every unstated rule a switch. The §2 analytics reproduce
+exactly, but the base case does not: firm sizes fall off far faster than his µ = 1.28 and firms
+live about 4 periods rather than 23.4, under every reading. Most of §4's tables hold in direction;
+two unstated details — whether sticky effort applies in a new firm, and which way seniority pay
+runs — decide whole tables; and his 2013 parameterization gives Zipf's law.
+See `docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md`.
+
 ## Experiments and science
 
 - **Parameter sweeps / batch runs**: done (Milestone 5).
@@ -316,6 +328,7 @@ See `docs/superpowers/specs/2026-09-28-bali-water-temples-design.md`.
 - **Boyd, Gintis, Bowles and Richerson's altruistic punishment** (and Cooney's PDE critique): done (Milestone 27).
 - **Gode and Sunder's zero-intelligence traders** (and Cliff's critique and ZIP traders): done (Milestone 28).
 - **Lansing and Kremer's Balinese water temples** (and Janssen's reanalysis): done (Milestone 29).
+- **Axtell's emergence of firms** (and his 2013 parameterization): done (Milestone 30).
 - **Minds 1: the utility mind and the ideal free distribution** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Minds 2: A\* and walking; which of the book's results need the jump** (our experiment; docs/studies/2026-09-27-minds.md): done.
 - **Minds 3: memory, belief and truffles; memory's value as an information asymmetry** (our experiment; docs/studies/2026-09-27-minds.md): done. Memory mostly hurts under rule M, which prices no travel; the marginal value theorem moves to Minds 4.
````

Modify `docs/papers.md` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/docs/papers.md b/docs/papers.md
index 1e4215a..85ca02a 100644
--- a/docs/papers.md
+++ b/docs/papers.md
@@ -37,6 +37,7 @@ read online or from another copy; add it when found. Scanned PDFs (no text layer
 | 27 | `punishment` | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf`; the critique `punishment/cooney-2024-arxiv-altruistic-punishment-pde-multilevel-selection.pdf` (published in *Bull. Math. Biol.* 2025); Janssen's NetLogo replication `punishment/janssen-comses-2223-netlogo/` (GPL-3.0: readings only) | the shapes hold but not the reach; the baseline is unstated and the caption contradicts the legend; the figures fit twice the stated conflict rate (14 of 14 curves; the text's rate 2); continuous traits are not similar; Cooney's dip appears under every victory rule |
 | 28 | `zi` | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*, read by OCR); the critique `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (with its C source) | the five markets recovered from the figures (Table 2 pins four exactly); efficiencies and dispersions reproduce, but only with enough shouts — "30 seconds" is never translated; Cliff's predictions miss the box markets and his 233⅓ is not his formula's; ZIP converges; his code's momentum is not his text's |
 | 29 | `bali` | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*); the reanalysis `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; Janssen's CoMSES model 2221 for the watershed data (GPL-2.0, shipped as data in `data/bali/`) | imitation's rise and Table 1 reproduce; the temple resemblance is the pest network's own; the subaks settle harder and the paper's perturbed run never recovers; the scale of coordination moves the harvest by 1–7 %, depending on the reading of the subak–dam columns, never Janssen's +30 %, and the temple scale is never best; the two-node threshold holds exactly; his code's departures barely matter |
+| 30 | `firms` | `firms/axtell-1999-emergence-of-firms.pdf`; the follow-up `firms/axtell-2013-lem-endogenous-dynamics-of-firms-and-labor.pdf` (his 2013 parameterization) | the §2 analytics reproduce exactly; the 1999 base case gives µ ≈ 2.5, not 1.28, and firms living ~4 periods, not 23.4, under every reading; most §4 tables' directions hold, the friends table's reverses; two unstated details (where sticky effort applies, which way seniority pay runs) decide whole tables; the 2013 parameterization gives Zipf's law |
 | 30 | `schelling`, `line` | `schelling/schelling-1971-jms-dynamic-models-of-segregation.pdf` (*scan*), `schelling/schelling-1969-aer-models-of-segregation.pdf`; for the variations to come, `schelling/pancs-vriend-2007-…`, `schelling/zhang-2004-jms-…`, `schelling/zhang-2004-jebo-…`, `schelling/gauvin-vannimenus-nadal-2009-…`, `schelling/singh-vainchtein-weiss-2009-…`, `schelling/bruch-mare-2008-…` | Schelling's own checkerboard and line; on the board his direction holds throughout, but his hand-worked boards sit at the segregated end of what his rules give (0.80 alike and 38 % unmixed, not Fig. 8's 90 % and two-thirds; ratio 3.6, not four to one; the 2:1 minority at 1.3); a halved minority on the line is no more segregated; 24 neighbors attenuate at a third but sort more at half; restricted travel leaves 6 % unsatisfied |
 | 31 | `tipping` | `schelling/schelling-1971-jms-dynamic-models-of-segregation.pdf` (*scan*), pp. 167–186; `schelling/schelling-1969-aer-models-of-segregation.pdf` | Schelling's bounded neighborhood reproduces completely: the two one-color states of Fig. 18, the 80–80 mixture from over 40 %, more than 25 % to tip in, the 3.0 threshold, forty apiece, the minority that must be the more tolerant and the less tolerant two-thirds |
 
@@ -47,47 +48,46 @@ worth doing; "size" is a guess at the milestone's scale.
 
 | # | Model | Original | Critique or follow-up | Size | Shape |
 |---|---|---|---|---|---|
-| 1 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
-| 2 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
-| 3 | Algorithmic collusion | `ai-coordination/calvano-calzolari-denicolo-pastorello-2020-aer-ai-algorithmic-pricing-and-collusion.pdf` | `ai-coordination/`: Klein 2021 (sequential pricing), Asker, Fershtman & Pakes 2021–22 (learning rules), Calvano et al. 2021 (imperfect monitoring); Abada & Lambin 2023 wanted | medium | new kind (Q-learning pricing agents; coordination with no communication — AI safety) |
-| 4 | Q-learning auctions | `multi-agent-coordination/banchio-skrzypacz-2022-arxiv-ai-and-auction-design.pdf` | — | small | collusion in first- but not second-price auctions |
-| 5 | Inspection games | `ai-coordination/avenhaus-von-stengel-zamir-2002-handbook-gt-inspection-games.pdf` | — | small | monitoring against violation; the oversight core of the swarm-coordination study |
-| 6 | Naming game | `language/steels-1995-artificial-life-self-organizing-spatial-vocabulary.pdf`; `language/baronchelli-felici-loreto-caglioti-steels-2006-jstatmech-sharp-transition-shared-vocabularies.pdf` | `language/`: Dall'Asta et al. 2006 (networks); Baronchelli 2016 (the minimal rules) | small | new kind (conventions emerge; N^1.5 consensus) |
-| 7 | Category game | `language/puglisi-baronchelli-loreto-2008-pnas-cultural-route-linguistic-categories.pdf` | `language/baronchelli-gong-puglisi-loreto-2010-pnas-universality-color-naming.pdf` (World Color Survey) | medium | naming game on a continuum; color terms |
-| 8 | Lewis signaling games | `language/huttegger-skyrms-smead-zollman-2010-synthese-lewis-signaling-partial-pooling.pdf` | `language/barrett-2009-…`; Skyrms 2010 (partial) | small | codes invented by reinforcement; partial pooling |
-| 9 | Iterated learning | `language/kirby-2001-ieee-tec-spontaneous-evolution-linguistic-structure.pdf` | `language/griffiths-kalish-2007-…` (converges to the prior); `language/kirby-cornish-smith-2008-pnas-…` (the lab data) | medium | compositional structure from a transmission bottleneck |
-| 10 | The evolution of language | `language/nowak-krakauer-1999-pnas-evolution-of-language.pdf` | `language/nowak-komarova-niyogi-2001-science-…` | small | words, then grammar, past an error threshold |
-| 11 | Vowel systems | `language/de-boer-2000-j-phonetics-self-organization-vowel-systems-proof.pdf` | `language/de-boer-1999-…` | medium | imitation games in formant space; typology |
-| 12 | Language death | `language/abrams-strogatz-2003-nature-modelling-dynamics-language-death.pdf` | `language/`: Stauffer et al. 2007 (microscopic), Mira & Paredes 2005 (similarity), Castelló et al. 2006 (bilingualism) | medium | two-language competition; does one always die? |
-| 13 | Spatial dialects | `language/burridge-2017-prx-spatial-evolution-human-dialects.pdf` | `language/nettle-1999-lingua-…` (social impact; rate of change) | medium | dialect boundaries under surface tension; cities; terrain |
-| 14 | Superstition as adaptive error | `belief/foster-kokko-2009-proc-r-soc-b-evolution-superstitious-behaviour.pdf` | `belief/skinner-1948-…` (the pigeons); Beck & Forstmeier 2007 wanted | small | false associations pay when misses cost more than false alarms |
-| 15 | Rumors and hoaxes | `belief/galam-2003-physica-a-modelling-rumors-no-plane-pentagon.pdf` | `belief/zanette-2002-…`, `belief/moreno-nekovee-pacheco-2004-…` (rumors on networks) | small | belief without evidence spreads by local majority |
-| 16 | Religion ABMs | `belief/shults-gore-wildman-lynch-lane-toft-2018-jasss-mutual-escalation-anxiety-religious-groups.pdf` | `belief/`: Whitehouse et al. 2012 (modes of religiosity), Upal 2005, Gore et al. 2018 | medium | anxiety, escalation, doctrinal against imagistic modes |
-| 17 | Dynamic social impact | `social-psych/latane-1996-j-communication-dynamic-social-impact.pdf` (Nowak, Szamrej & Latané 1990 wanted) | `social-psych/`: Hołyst et al. 2000, Dworak & Malarz 2023 (exact rules, phase transitions) | medium | consolidation, clustering, continuing diversity |
-| 18 | The emperor's dilemma | `social-psych/centola-willer-macy-2005-ajs-emperors-dilemma-self-enforcing-norms.pdf` | — | small | enforcing a norm most privately reject |
-| 19 | Rogers' paradox | `social-psych/rogers-1988-am-anthropologist-does-biology-constrain-culture.pdf` | `social-psych/rendell-fogarty-laland-2010-…` (recast and resolved); Henrich & Boyd 1998 (conformism); Rendell et al. 2010 (the tournament) | small | social learning adds no fitness at equilibrium |
-| 20 | Minority opinion spreading | `social-psych/galam-2002-epjb-minority-opinion-spreading-random-geometry.pdf` | `social-psych/castellano-fortunato-loreto-2009-rmp-…` (review) | small | ties to the status quo let a minority win |
-| 21 | Differentiation without distancing | `social-psych/mas-flache-2013-plos-one-differentiation-without-distancing.pdf` | `social-psych/smaldino-epstein-2015-…` (conformity from distinctiveness); Antal et al. 2005 (social balance) | small | bipolarization without negative influence |
-| 22 | Agent_Zero | `social-psych/epstein-chelen-2016-strungmann-advancing-agent-zero.pdf` (the book wanted) | — | medium | affect, deliberation and social contagion in one agent |
-| 23 | The garbage can | Cohen, March & Olsen 1972 wanted; their Fortran in `organizations/garbage-can-code/` | `organizations/`: Fioretti & Lomi 2008 (agent-based), 2009 (buck-passing), Glynn et al. 2020; Bendor, Moe & Shotts 2001 wanted (code ≠ theory; their code saved) | medium | decisions by oversight and flight |
-| 24 | Exploration and exploitation | March 1991 wanted | `organizations/jo-2024-arxiv-exact-solutions-simplified-march-model.pdf`; Chanda & Miller 2019 replication wanted | small | slow learners and turnover help the code |
-| 25 | Santa Fe artificial stock market | `markets/palmer-arthur-holland-lebaron-tayler-1994-physica-d-artificial-economic-life.pdf`; `markets/arthur-holland-lebaron-palmer-tayler-1996-sfi-wp-…` | `markets/lebaron-2002-…`; Ehrentreich 2008 wanted (the mutation operator's bias) | large | classifier-system traders; the complex regime |
-| 26 | Leadership in animal groups | `collective-motion/couzin-krause-franks-levin-2005-nature-effective-leadership-animal-groups.pdf` | `collective-motion/vicsek-…-1995-prl-…`; Couzin et al. 2002 wanted | medium | an informed minority steers; Vicsek's transition |
-| 27 | Adaptive parties | `politics/kollman-miller-page-1992-sfi-wp-adaptive-parties-in-spatial-elections.pdf` | `politics/kollman-miller-page-1993-sfi-wp-…`; Laver 2005 wanted | medium | parties climbing an electoral landscape |
-| 28 | Division of labor | `multi-agent-coordination/theraulaz-bonabeau-deneubourg-1998-sfi-wp-response-threshold-reinforcement.pdf` | Bonabeau et al. 1996 wanted | small | response thresholds; specialists emerge |
-| 29 | Sequential social dilemmas | `multi-agent-coordination/leibo-et-al-2017-arxiv-marl-in-sequential-social-dilemmas.pdf` | `multi-agent-coordination/`: Perolat et al. 2017 (commons), Hughes et al. 2018 (inequity aversion) | medium | learners in gathering and commons games |
-| 30 | War, space and states | `archaeology/turchin-currie-turner-gavrilets-2013-pnas-war-space-old-world-complex-societies.pdf` (+ SI) | `archaeology/currie-…-2020-hssc-…` with its gridded data (CC0) | large | military technology from the steppe on a real map, 1500 BCE–1500 CE; geography and states |
-| 31 | Chiefdom cycling | `archaeology/gavrilets-anderson-turchin-2010-cliodynamics-cycling-in-complexity-of-early-societies.pdf` | — | medium | conquest and collapse of chiefdoms on a lattice |
-| 32 | Circumscription | `archaeology/williams-mesoudi-2024-jas-formal-test-abm-circumscription-theory.pdf` (Carneiro 1970 wanted) | `archaeology/`: Williams & Mesoudi 2025 (Oaxaca), Zinkina et al. 2016 (cross-cultural) | medium | states where arable land is walled in; a natural test on generated terrain |
-| 33 | Village Ecodynamics | `archaeology/crabtree-bocinsky-hooper-ryan-kohler-2017-am-antiquity-how-to-make-a-polity.pdf`; code MIT on GitHub, GPL-2.0 on CoMSES | `archaeology/kohler-…` SFI working papers; Kohler et al. 2012 wanted | very large | the Anasazi model's successor: farming, hunting, exchange, warfare |
-| 34 | Random drift in culture | `archaeology/bentley-hahn-shennan-2004-proc-r-soc-b-random-drift-and-culture-change.pdf` (Neiman 1995 wanted) | — | small | neutral copying; pottery, names, patents |
-| 35 | MERCURY | `archaeology/brughmans-poblome-2016-antiquity-roman-bazaar-or-market-economy.pdf`, `…-jasss-…` (code AFL-3.0 on CoMSES) | `archaeology/`: Kanters et al. 2021 (sensitivity), Carrignon et al. 2020 (ABC) | medium | Roman tableware and the Roman economy |
-| 36 | Raiding and consolidation | `archaeology/griffin-stanish-2007-structure-dynamics-abm-titicaca-political-consolidation.pdf` | — | medium | raids and settlement hierarchy (Titicaca) |
-| 37 | Farming and property | `agriculture/bowles-choi-2013-pnas-coevolution-of-farming-and-private-property.pdf` | `agriculture/bowles-choi-2016-sfi-wp-…` (the JPE version) | medium | farming takes off only with property |
-| 38 | The wave of advance | `agriculture/pinhasi-fort-ammerman-2005-plos-biology-…`; Ammerman & Cavalli-Sforza 1971 wanted | `agriculture/`: Fort 2012, 2015 (demic against cultural), Ackland et al. 2007 (hitchhiking), Lemmen et al. 2011, LaPolice et al. 2025, Aoki 2020 | medium | farming's front at ~1 km a year; radiocarbon targets |
-| 39 | Scattered strips | `agriculture/mccloskey-1976-research-econ-history-open-fields-behavior-towards-risk.pdf` | `agriculture/mccloskey-1991-jeh-prudent-peasant-open-fields.pdf` | small | medieval open fields as insurance |
-| 40 | Chayanov's rule | `agriculture/hammel-2005-pnas-chayanov-revisited.pdf` | `agriculture/puleston-tuljapurkar-winterhalder-2014-plos-one-invisible-cliff.pdf` | small | household labor over the family cycle |
-| 41 | Generated terrain and hydrology | `terrain/`: Perlin 2002, Gustavson 2005; Barnes et al. 2014 (priority-flood, flats), Tarboton 1997 (D∞), Mark 1983 (D8); Cordonnier et al. 2016, Génevaux et al. 2013 (erosion, hydrology); Smith & Barstad 2004 (orographic rain) | `terrain/`: Hack 1957, Rinaldo et al. 2014, Carraro et al. 2020 (OCNet) as realism checks; Horton 1945 and Braun & Willett 2013 wanted | large | our own milestone: a terrain generator validated against geomorphology's laws, then geography-driven models on ensembles of worlds |
+| 1 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
+| 2 | Algorithmic collusion | `ai-coordination/calvano-calzolari-denicolo-pastorello-2020-aer-ai-algorithmic-pricing-and-collusion.pdf` | `ai-coordination/`: Klein 2021 (sequential pricing), Asker, Fershtman & Pakes 2021–22 (learning rules), Calvano et al. 2021 (imperfect monitoring); Abada & Lambin 2023 wanted | medium | new kind (Q-learning pricing agents; coordination with no communication — AI safety) |
+| 3 | Q-learning auctions | `multi-agent-coordination/banchio-skrzypacz-2022-arxiv-ai-and-auction-design.pdf` | — | small | collusion in first- but not second-price auctions |
+| 4 | Inspection games | `ai-coordination/avenhaus-von-stengel-zamir-2002-handbook-gt-inspection-games.pdf` | — | small | monitoring against violation; the oversight core of the swarm-coordination study |
+| 5 | Naming game | `language/steels-1995-artificial-life-self-organizing-spatial-vocabulary.pdf`; `language/baronchelli-felici-loreto-caglioti-steels-2006-jstatmech-sharp-transition-shared-vocabularies.pdf` | `language/`: Dall'Asta et al. 2006 (networks); Baronchelli 2016 (the minimal rules) | small | new kind (conventions emerge; N^1.5 consensus) |
+| 6 | Category game | `language/puglisi-baronchelli-loreto-2008-pnas-cultural-route-linguistic-categories.pdf` | `language/baronchelli-gong-puglisi-loreto-2010-pnas-universality-color-naming.pdf` (World Color Survey) | medium | naming game on a continuum; color terms |
+| 7 | Lewis signaling games | `language/huttegger-skyrms-smead-zollman-2010-synthese-lewis-signaling-partial-pooling.pdf` | `language/barrett-2009-…`; Skyrms 2010 (partial) | small | codes invented by reinforcement; partial pooling |
+| 8 | Iterated learning | `language/kirby-2001-ieee-tec-spontaneous-evolution-linguistic-structure.pdf` | `language/griffiths-kalish-2007-…` (converges to the prior); `language/kirby-cornish-smith-2008-pnas-…` (the lab data) | medium | compositional structure from a transmission bottleneck |
+| 9 | The evolution of language | `language/nowak-krakauer-1999-pnas-evolution-of-language.pdf` | `language/nowak-komarova-niyogi-2001-science-…` | small | words, then grammar, past an error threshold |
+| 10 | Vowel systems | `language/de-boer-2000-j-phonetics-self-organization-vowel-systems-proof.pdf` | `language/de-boer-1999-…` | medium | imitation games in formant space; typology |
+| 11 | Language death | `language/abrams-strogatz-2003-nature-modelling-dynamics-language-death.pdf` | `language/`: Stauffer et al. 2007 (microscopic), Mira & Paredes 2005 (similarity), Castelló et al. 2006 (bilingualism) | medium | two-language competition; does one always die? |
+| 12 | Spatial dialects | `language/burridge-2017-prx-spatial-evolution-human-dialects.pdf` | `language/nettle-1999-lingua-…` (social impact; rate of change) | medium | dialect boundaries under surface tension; cities; terrain |
+| 13 | Superstition as adaptive error | `belief/foster-kokko-2009-proc-r-soc-b-evolution-superstitious-behaviour.pdf` | `belief/skinner-1948-…` (the pigeons); Beck & Forstmeier 2007 wanted | small | false associations pay when misses cost more than false alarms |
+| 14 | Rumors and hoaxes | `belief/galam-2003-physica-a-modelling-rumors-no-plane-pentagon.pdf` | `belief/zanette-2002-…`, `belief/moreno-nekovee-pacheco-2004-…` (rumors on networks) | small | belief without evidence spreads by local majority |
+| 15 | Religion ABMs | `belief/shults-gore-wildman-lynch-lane-toft-2018-jasss-mutual-escalation-anxiety-religious-groups.pdf` | `belief/`: Whitehouse et al. 2012 (modes of religiosity), Upal 2005, Gore et al. 2018 | medium | anxiety, escalation, doctrinal against imagistic modes |
+| 16 | Dynamic social impact | `social-psych/latane-1996-j-communication-dynamic-social-impact.pdf` (Nowak, Szamrej & Latané 1990 wanted) | `social-psych/`: Hołyst et al. 2000, Dworak & Malarz 2023 (exact rules, phase transitions) | medium | consolidation, clustering, continuing diversity |
+| 17 | The emperor's dilemma | `social-psych/centola-willer-macy-2005-ajs-emperors-dilemma-self-enforcing-norms.pdf` | — | small | enforcing a norm most privately reject |
+| 18 | Rogers' paradox | `social-psych/rogers-1988-am-anthropologist-does-biology-constrain-culture.pdf` | `social-psych/rendell-fogarty-laland-2010-…` (recast and resolved); Henrich & Boyd 1998 (conformism); Rendell et al. 2010 (the tournament) | small | social learning adds no fitness at equilibrium |
+| 19 | Minority opinion spreading | `social-psych/galam-2002-epjb-minority-opinion-spreading-random-geometry.pdf` | `social-psych/castellano-fortunato-loreto-2009-rmp-…` (review) | small | ties to the status quo let a minority win |
+| 20 | Differentiation without distancing | `social-psych/mas-flache-2013-plos-one-differentiation-without-distancing.pdf` | `social-psych/smaldino-epstein-2015-…` (conformity from distinctiveness); Antal et al. 2005 (social balance) | small | bipolarization without negative influence |
+| 21 | Agent_Zero | `social-psych/epstein-chelen-2016-strungmann-advancing-agent-zero.pdf` (the book wanted) | — | medium | affect, deliberation and social contagion in one agent |
+| 22 | The garbage can | Cohen, March & Olsen 1972 wanted; their Fortran in `organizations/garbage-can-code/` | `organizations/`: Fioretti & Lomi 2008 (agent-based), 2009 (buck-passing), Glynn et al. 2020; Bendor, Moe & Shotts 2001 wanted (code ≠ theory; their code saved) | medium | decisions by oversight and flight |
+| 23 | Exploration and exploitation | March 1991 wanted | `organizations/jo-2024-arxiv-exact-solutions-simplified-march-model.pdf`; Chanda & Miller 2019 replication wanted | small | slow learners and turnover help the code |
+| 24 | Santa Fe artificial stock market | `markets/palmer-arthur-holland-lebaron-tayler-1994-physica-d-artificial-economic-life.pdf`; `markets/arthur-holland-lebaron-palmer-tayler-1996-sfi-wp-…` | `markets/lebaron-2002-…`; Ehrentreich 2008 wanted (the mutation operator's bias) | large | classifier-system traders; the complex regime |
+| 25 | Leadership in animal groups | `collective-motion/couzin-krause-franks-levin-2005-nature-effective-leadership-animal-groups.pdf` | `collective-motion/vicsek-…-1995-prl-…`; Couzin et al. 2002 wanted | medium | an informed minority steers; Vicsek's transition |
+| 26 | Adaptive parties | `politics/kollman-miller-page-1992-sfi-wp-adaptive-parties-in-spatial-elections.pdf` | `politics/kollman-miller-page-1993-sfi-wp-…`; Laver 2005 wanted | medium | parties climbing an electoral landscape |
+| 27 | Division of labor | `multi-agent-coordination/theraulaz-bonabeau-deneubourg-1998-sfi-wp-response-threshold-reinforcement.pdf` | Bonabeau et al. 1996 wanted | small | response thresholds; specialists emerge |
+| 28 | Sequential social dilemmas | `multi-agent-coordination/leibo-et-al-2017-arxiv-marl-in-sequential-social-dilemmas.pdf` | `multi-agent-coordination/`: Perolat et al. 2017 (commons), Hughes et al. 2018 (inequity aversion) | medium | learners in gathering and commons games |
+| 29 | War, space and states | `archaeology/turchin-currie-turner-gavrilets-2013-pnas-war-space-old-world-complex-societies.pdf` (+ SI) | `archaeology/currie-…-2020-hssc-…` with its gridded data (CC0) | large | military technology from the steppe on a real map, 1500 BCE–1500 CE; geography and states |
+| 30 | Chiefdom cycling | `archaeology/gavrilets-anderson-turchin-2010-cliodynamics-cycling-in-complexity-of-early-societies.pdf` | — | medium | conquest and collapse of chiefdoms on a lattice |
+| 31 | Circumscription | `archaeology/williams-mesoudi-2024-jas-formal-test-abm-circumscription-theory.pdf` (Carneiro 1970 wanted) | `archaeology/`: Williams & Mesoudi 2025 (Oaxaca), Zinkina et al. 2016 (cross-cultural) | medium | states where arable land is walled in; a natural test on generated terrain |
+| 32 | Village Ecodynamics | `archaeology/crabtree-bocinsky-hooper-ryan-kohler-2017-am-antiquity-how-to-make-a-polity.pdf`; code MIT on GitHub, GPL-2.0 on CoMSES | `archaeology/kohler-…` SFI working papers; Kohler et al. 2012 wanted | very large | the Anasazi model's successor: farming, hunting, exchange, warfare |
+| 33 | Random drift in culture | `archaeology/bentley-hahn-shennan-2004-proc-r-soc-b-random-drift-and-culture-change.pdf` (Neiman 1995 wanted) | — | small | neutral copying; pottery, names, patents |
+| 34 | MERCURY | `archaeology/brughmans-poblome-2016-antiquity-roman-bazaar-or-market-economy.pdf`, `…-jasss-…` (code AFL-3.0 on CoMSES) | `archaeology/`: Kanters et al. 2021 (sensitivity), Carrignon et al. 2020 (ABC) | medium | Roman tableware and the Roman economy |
+| 35 | Raiding and consolidation | `archaeology/griffin-stanish-2007-structure-dynamics-abm-titicaca-political-consolidation.pdf` | — | medium | raids and settlement hierarchy (Titicaca) |
+| 36 | Farming and property | `agriculture/bowles-choi-2013-pnas-coevolution-of-farming-and-private-property.pdf` | `agriculture/bowles-choi-2016-sfi-wp-…` (the JPE version) | medium | farming takes off only with property |
+| 37 | The wave of advance | `agriculture/pinhasi-fort-ammerman-2005-plos-biology-…`; Ammerman & Cavalli-Sforza 1971 wanted | `agriculture/`: Fort 2012, 2015 (demic against cultural), Ackland et al. 2007 (hitchhiking), Lemmen et al. 2011, LaPolice et al. 2025, Aoki 2020 | medium | farming's front at ~1 km a year; radiocarbon targets |
+| 38 | Scattered strips | `agriculture/mccloskey-1976-research-econ-history-open-fields-behavior-towards-risk.pdf` | `agriculture/mccloskey-1991-jeh-prudent-peasant-open-fields.pdf` | small | medieval open fields as insurance |
+| 39 | Chayanov's rule | `agriculture/hammel-2005-pnas-chayanov-revisited.pdf` | `agriculture/puleston-tuljapurkar-winterhalder-2014-plos-one-invisible-cliff.pdf` | small | household labor over the family cycle |
+| 40 | Generated terrain and hydrology | `terrain/`: Perlin 2002, Gustavson 2005; Barnes et al. 2014 (priority-flood, flats), Tarboton 1997 (D∞), Mark 1983 (D8); Cordonnier et al. 2016, Génevaux et al. 2013 (erosion, hydrology); Smith & Barstad 2004 (orographic rain) | `terrain/`: Hack 1957, Rinaldo et al. 2014, Carraro et al. 2020 (OCNet) as realism checks; Horton 1945 and Braun & Willett 2013 wanted | large | our own milestone: a terrain generator validated against geomorphology's laws, then geography-driven models on ensembles of worlds |
 
 ## Wanted
 
@@ -115,6 +115,20 @@ archaeology, agriculture, migration and terrain); what it couldn't find free:
 - **Terrain:** Perlin (1985); O'Callaghan & Mark (1984); Horton (1945); Braun & Willett (2013),
   *Geomorphology* 180.
 
+A second search (2026-09-30) added, still missing or found only as re-posts:
+
+- **Firms:** Axtell (2018), *Handbook of Computational Economics* 4 ch. 3,
+  doi:10.1016/bs.hescom.2018.05.001 (his latest statement of the rules); Axtell (2001), "Zipf
+  distribution of U.S. firm sizes", *Science* 293:1818 — both found only as re-posts.
+- **Belief and social psychology:** Abbott & Sherratt (2011), *Animal Behaviour* 82:85,
+  doi:10.1016/j.anbehav.2011.04.002; Staddon & Simmelhag (1971) and Enquist, Eriksson & Ghirlanda
+  (2007), *American Anthropologist* 109:727 — re-posts only.
+- **Collective motion:** Conradt & Roper (2003), *Nature* 421:155.
+- **Archaeology:** Reynolds (2000) on raiding in the Valley of Oaxaca; Barceló, Capuzzo &
+  Bogdanović (2014), *J. Archaeological Method and Theory* 21:486; Rogers et al. (2012), *Ecological
+  Modelling* 241:5; Ullah's AgModel article (submitted 2023).
+- **Terrain:** Tucker & Hancock (2010), *Earth Surface Processes and Landforms* 35:28.
+
 Earlier:
 
 - For Schelling's variations: van de Rijt, Siegel & Macy (2009), "Neighborhood chance and neighborhood change", *AJS* 114(4), with Bruch & Mare's reply, before any caption on continuous versus threshold preferences; and Schelling's *Micromotives and Macrobehavior* (1978), whose 8 × 8 reshuffled board Singh et al. critique.
````

Modify `docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md` (a unified diff against the branch's base: save it and `git apply --recount` it, or make the edits by hand):

````diff
diff --git a/docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md b/docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md
index 60a7e2a..c343538 100644
--- a/docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md
+++ b/docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md
@@ -126,3 +126,16 @@ The presets menu gains **The Emergence of Firms**; the Rules panel is generated
 ## Docs
 
 README: an Emergence of Firms section (the model, how the paper was read, its contradictions, switches, presets, sweeps, findings). `docs/papers.md`: the milestone's row; the Queue's first entry removed; roadmap: Milestone 30 done.
+
+## Amendments (implementation planning)
+
+- **Two more readings, found by measuring:**
+  - **`adjust_scope`** (`everywhere` | `own_firm`): whether sticky effort's window and groping apply to effort in a firm joined or founded (A99 §4.6 does not say). Read as `everywhere` (the default, as specified above), both send the whole population into one firm at times, in every seed: output grows as E², so a newcomer arriving with its old effort earns much more, while free riding sets in only ±0.05 a step. At home only, both behave like the base case with somewhat larger firms.
+  - **`seniority_order`** (`senior_first` | `junior_first`): the text ranks the founder first, so a joiner's share of a pair's output under 5^−rank is 1/31 and no firm grows past 4; paying the newest most gives near-giant firms (µ ≈ 0.2). Table 11's µ 1.11 comes from neither.
+- **Ranges:** a maximum at or below its value means fixed (not an error), so each field accepts a change on its own.
+- **The effort optimum:** the closed form (5) for β = 2; otherwise a safeguarded Newton solve of the first-order condition (Cobb–Douglas, output-proportional shares), or (base pay, CES) a root search on the analytic slope: the range is split at base pay's kink, each piece's bonus regime is decided at its midpoint, and each piece's root is found by Illinois regula falsi with a bisection fallback, the best candidate winning. A test checks the solvers against a fine grid on random cases. The maximum-likelihood µ is the exact discrete one (Clauset, Shalizi and Newman's eq. 3.5), updated every 10 periods.
+- **Constant returns (fn 19):** "never strictly individually-rational to work together" holds at equilibrium; A99's myopic agents, taking others' effort as given, still join (alone at e = θ, joining an identical agent at θ² gains (1 + θ)/2^θ > 1). A unit test and a survey claim pin it.
+- **Axtell 2013 as a claim:** its parameterization gives µ ≈ 1.0 (Zipf's law, as A13 reports α ≈ 1.06) with mean lifetimes near 77 periods — where the 1999 base case gives µ 2.3–2.8 and lifetimes near 4. The survey adds it (`firms.a13.zipf`) with the two readings above (`firms.ours.adjust-scope`, `firms.ours.seniority-order`), each rule written after the planning runs and saying so.
+- **Page:** the sweeps list adds the new readings (the sticky and groping sweeps each gain an own-firm line; the seniority sweep gains newest-first rows).
+- **The view:** rows are the largest firms first (3-pixel cells, up to 166 firms of up to 200 members), not in order of founding: with about 400 firms, mostly of one or two, founding order showed old pairs and dropped the large firms off the frame.
+- **Measured in implementation:** the survey has 25 claims: 15 hold and 10 fail (the base case's µ, output and productivity, lifetimes and levels; §2's 'less than 10 for θ < 0.85' by a hair; the friends table, which reverses; the preferences table's spread; seniority; the hiring standard's breakdown; random effort, which gives one giant firm).
````

- [ ] **Step 2: Verify everything**

Run: `cargo fmt --all --check && cargo +stable clippy --all-targets -- -D warnings && cargo test --release --workspace && wasm-pack test --node crates/sugarscape-wasm && (cd web && npm run build && npm test)`
Expected: all green (1 405 Rust tests, 64 WASM, 802 web). The survey's verdicts are as in Task 4.

- [ ] **Step 3 (controller): the full browser pass.** Repeat Task 3's Step 6 list on the final build.

- [ ] **Step 4: Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-09-30-emergence-of-firms-design.md
```
```bash
git commit -m "Document The Emergence of Firms, how the paper was read and what reproduces; mark milestone 30 done

Claude-Session: https://claude.ai/code/session_01CDy35L1s2fYHpcryzQj8w4"
```

## Self-review (planning)

- **Spec coverage:**
  - Architecture, Config, Step, Statistics, Views, Presets, Compare, Experiments and CLI → Tasks 1–3.
  - Survey → Task 4.
  - Docs → Task 5.
  - Departures are recorded in the spec's Amendments.
- **Placeholders:** none. Every file is given in full or as a diff against `52375c8`.
- **Types:** `FirmsInspection` (types.ts) matches `FirmsInspection`, `FirmView` and `AgentView` (world.rs). `isFirmsView` tests `panel`, `firm` and `member`, which no other inspection has.
