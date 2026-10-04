# Algorithmic Collusion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Calvano, Calzolari, Denicolò & Pastorello's "Artificial Intelligence, Algorithmic Pricing, and Collusion" (AER 2020) as one model kind, `collusion` ("Algorithmic Collusion"): two (to four) firms set prices on a grid with Q-learning until their strategies settle; the session is then analyzed — limit cycle, profit gain Δ, equilibrium (by the paper's test and the code's), responses to deviations. Under the authors' own readings the engine reproduces their Fortran period for period (100 of 100 sessions). The critics' tests (Asker, Fershtman & Pakes; Lambin; Epivent & Lambin; den Boer, Meylahn & Schinkel; Eschenbaum, Mellgren & Zahn) are named switches. Ten titled presets, thirteen built-in sweeps and a survey of the spec's A and B tables, in every playground surface, without changing any existing run.

**Architecture:** A new core module `crates/sugarscape-core/src/collusion/`:

- `config.rs`: the parameters, nine reading enums, validation, the schema.
- `ran2.rs`: the authors' RAN2, exactly.
- `demand.rs`: logit demand, the Nash and monopoly benchmarks (bisection, rounded to 5 decimals as the authors' inputs are), the grid (the code's construction), the payoff table, den Boer et al.'s horizon T_δ.
- `learner.rs`: the state space, a firm's Q-table with its greedy price kept incrementally as the code keeps it, the random draws (ours, or the code's two RAN2 streams and shared initial-price stream).
- `analysis.rs`: the limit cycle, Δ, policy and optimal values, both equilibrium checks, deviations and responses, RP-completeness, the invitation, re-pairing, Lambin's Theorem 1.
- `world.rs`: `CollusionWorld` — one session: a tick of `periods_per_tick` periods (1 000), each period's prices and updates, convergence, the outcome, per-tick statistics, rendering, Inspect.
- `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`.

It is wired into `ModelConfig`/`ModelWorld` like the other models, with titles in `titles.rs`. The page adds the model's types, color modes, charts, Inspect rows, a Compare entry and an Experiments default; the survey adds the spec's claims.

**Tech Stack:** Rust core, `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md` (binding, as amended in Task 5), with `docs/superpowers/specs/2026-10-01-collusion-reading-notes.md`. Sources in `papers/ai-coordination/`; the replication package's Fortran was built in planning at `/private/tmp/claude-501/-Users-nathan-Projects-ndouglas-SugarScape/71002cc9-c4cc-431c-978a-bd677989e843/scratchpad/fortran-baseline/` (scratch; never committed).

## Global Constraints

- **Existing runs unchanged:** every existing `GOLDEN` and `MODEL_GOLDEN` entry and legacy fixture stays green and unedited. `MODEL_GOLDEN` gains ten `collusion-*` entries.
- **One engine path; deterministic; portable:** native and WASM fingerprints identical. Draws are `f64` samples and `u32` ranges (ours) or RAN2's 32-bit integer arithmetic (the code's); `exp` only through `portable::exp_neg` with a non-positive argument (logit shares shifted by their largest exponent; ε's factor `exp_neg(−β)`; Boltzmann's `exp_neg((q − max)/T)`); the payoff table and benchmarks computed once at reset.
- **Literal defaults, named departures, honest descriptions and titles:** the paper's text is the default (ties to the lowest price, a 10⁹ cap, the best-response equilibrium check, the static best response to the rival's price before the deviation); the code's choices are switches that the `collusion-code` preset turns on (`as_coded`). Descriptions and titles say what was measured, with the paper's numbers beside ours.
- **Copy (verbatim):**
  - Model label: **Algorithmic Collusion**.
  - Preset ids: `collusion-calvano`, `collusion-code`, `collusion-no-memory`, `collusion-myopic`, `collusion-two-phase`, `collusion-synchronous`, `collusion-explore-more`, `collusion-every-price`, `collusion-invitation`, `collusion-below-nash`.
  - Compare entry: **Learning from the price charged vs every price — Algorithmic Collusion (Compare)** (id `collusion-async-vs-sync`).
  - Color modes: **Price**, **Visits**.
  - Schema groups: **Market**, **Prices**, **Learning**, **Exploration**, **Session**, **Analysis**.
  - Charts: **Prices**, **Profit gain**, **Learning**, **Settling**; time axis **Ticks** (a tick is `periods_per_tick` periods, 1 000 by default).
  - Sweeps: `collusion-table-i`, `collusion-alpha-beta`, `collusion-delta`, `collusion-memory`, `collusion-myopic`, `collusion-two-phase`, `collusion-every-price`, `collusion-below-nash`, `collusion-invitation`, `collusion-synchronous`, `collusion-exploration`, `collusion-timescale`, `collusion-rp-complete`.
  - Series: `price_1, price_2, profit_gain, greedy_price, epsilon, explored, greedy_changes, stable, converged, cycle_length, cycle_gain, window_gain, discounted_gain, equilibrium_on_path, punishment_like, rp_complete, periods`.
  - Notice: `This session has finished at tick <tick> — Reset to run it again`.
  - CLI: `(it converged)`, or `(its cap)` when the cap stopped it.
- **Commits:** every commit message ends with a blank line and `Claude-Session: https://claude.ai/code/session_01XxEZRQqmFWQaPDVcu7Me1w`. Stage only the task's files; never stage `.claude/`, `papers/` or `web/node_modules`.
- **Rust:** `cargo fmt --all && cargo +stable clippy --all-targets -- -D warnings` (CI runs the latest stable clippy). In `survey/`, format only `survey/src/claims/collusion.rs`, and do not commit `survey/out/results-*.json`.
- **Web:** `(cd web && npm run build && npm test)`. In a fresh worktree, run `npm ci` and `npm run wasm` first.
- **Browser checks are the controller's:** Task 3's last step, and the full pass in Task 5.

## Review Focus

1. **Matching the authors' code exactly.** Under `as_coded` a session is their Fortran's session period for period: RAN2's 32-bit Schrage arithmetic and its first draws; the draw order (two exploration uniforms per firm per period, firm order; the shared initial-price stream skipped by (session − 1)·k·n); random ties only when a greedy price's own value falls; the convergence counter resetting to 1; the strategy at the cap being the one before that period's update. Pinned in Task 1 by `the_first_draws_are_the_fortran_s`, `the_authors_rng_seeds_by_session` and `tests/collusion.rs` (100 fixture sessions, ignored test for all; the survey's `collusion.ccdp.docking` re-runs them).
2. **The paper's readings against the code's.** Ties to the lowest price must rescan whenever the maximum may have moved (an exact tie with a new value takes the lower price); Fig. 4's static best response answers the state the code names by cycle position (`BestResponseTo::Code`) while Table A5's answers the state itself; a "deviation" to the price the strategy charges anyway is not unprofitable (Table A5's IC). Pinned by `the_greedy_price_follows_the_tie_rule`, `incremental_greedy_prices_match_a_full_rescan` (20 000 random updates), `the_codes_best_response_answers_state_one`, and the survey's `collusion.ccdp.impulse` (Table A5 to the digit).
3. **Ticks are blocks of periods.** A session finishes mid-tick; `tick`, `period`, the cap (in periods), the snapshot's per-tick sums and the page's `ticksLeft` (in ticks: ⌈(cap + 1)/periods_per_tick⌉) must agree, and a finished world must not move. Pinned by `a_short_session_converges_and_is_analyzed` (tick = ⌈period/1 000⌉, fingerprint unchanged after `run`), `the_cap_stops_a_session_that_never_settles` (period 5 001, tick 6), and `models.test.ts`'s collusion `ticksLeft`.
4. **Equilibrium by both tests.** Optimal values by value iteration and policy values by exact cycle sums; every best-response equilibrium must pass the one-period test. Pinned by `grim_trigger_at_monopoly_is_an_equilibrium_and_rp_complete` (both tests, and impatience breaking it), `policy_values_sum_the_discounted_cycle`, and the survey's `collusion.ccdp.equilibrium` (containment checked over 1 000 sessions).
5. **Portability.** Logit shares, ε's factor and Boltzmann choice all go through `exp_neg` with a non-positive argument; native and WASM fingerprints agree on six presets, Boltzmann and three firms with random Q-values. Pinned in Task 2 by `collusion_sims_match_the_native_golden_entries` and `collusion_boltzmann_and_three_firms_match_the_native_fingerprints`.

## Decisions (where the spec leaves room, or planning changed it)

All code here was implemented in a scratch copy during planning and passed `cargo test --release --workspace` (1 601 passed, 101 ignored), `cargo +stable clippy --all-targets -- -D warnings`, `wasm-pack test --node crates/sugarscape-wasm` (74), `npm run build && npm test` (852, 53 files), and the survey (20 claims). Task 5 amends the spec with these.

1. **The authors' code builds and is the reference.** gfortran 14 builds `AER_fcode/baseline` after scratch-only ports of Intel extensions (`SORTQQ`, 88 `<expr>` formats, `.NOT.`, `READONLY`, static linking); it reproduces Table I from 100 sessions. Our engine under `as_coded` reproduces its sessions 1–100 exactly, so seeds 1–1 000 under `as_coded` are the authors' sessions: Table I to the digit (Δ 0.849, 50.5 %, cycles 64.3 / 23.8 / 11.9 %) and Table A5 to the digit (IR −0.127, IC 0.936, punishment 5.705).
2. **A tick is `periods_per_tick` periods (1 000).** The spec's "one tick is one period" cannot work: the page stops every run at 10⁶ ticks and sweeps at 10⁵, while a session needs about 1.8 × 10⁶ periods. At 1 000 periods a tick, a session takes about 1 800 ticks, the page's ceiling is the paper's 10⁹-period cap, and each charted point sums one tick (mean profit gain, share exploring, greedy changes). `sample_every` is gone.
3. **The seed is the session number** under `rng = calvano` (the spec's `session` field is dropped).
4. **Benchmarks are rounded to 5 decimals**, as the authors' inputs are (their Mathematica script is unpublished); Nash prices by best-response bisection, monopoly by equal-markup bisection (a damped fixed point oscillates once Σq is large).
5. **`q_init`: `calvano`, `zero`, `random`.** AFP's "optimistic" start is `random` with their 10–20, the default range; the spec's separate `optimistic` would be the same draw.
6. **`grid = symmetric`** is den Boer et al.'s Ã with the config's ξ: [p^N − (1 + ξ)ζ, p^N + (1 + ξ)ζ] (B7 sets ξ = 0); their Â is not built.
7. **A new switch, `best_response_to`** (`path` | `code`): found in planning, the code's impulse-response routine passes the cycle position to `ComputeStaticBestResponse`, which reads it as a state number. Fig. 4 uses that routine; its detailed analysis (Table A5) does not. `as_coded` sets `code`; the responses differ in 94 % of sessions.
8. **Responses carry the code's `ShockLength`** (`settled`: periods until play enters any cycle), Table A5's punishment length; `returned` (back on the pre-deviation cycle) drives the punishment-like rule.
9. **`rp_complete` is a series**, so its sweep exists; re-pairing has no sweep (the survey runs it). Sweeps have at most 100 seeds and 10⁵ ticks (the playground's limits); the survey's claims run 1 000 sessions (10 000 for E&L's grid).
10. **B5 split in two claims, and the constant arm read at 10⁷ periods** (100 sessions): set after measuring — no session with ε = 0.05 settled in 10⁸ periods (4 of 4), so the 10⁹ cap would take hours.
11. **A3 compares the paper's readings with the figure** (each cell its color bin's midpoint, bins 0.016 wide), since the code's readings are the figure's own sessions; Fig. 3 (Δ against δ) is read exactly from its paths.
12. **A6–A8 as claims where they can be scored** (`collusion.ccdp.equilibrium`, `collusion.ccdp.figure-4-reading`); the rest are reported in the docs.
13. **Lambin's Theorem 1** is a tested function (`lambin_point`): I = 1.7377 at δ = 0.95, 1.6990 at δ = 0, matching the reader's computation.
14. **Planning's findings** (the survey reproduces them):
    - The paper's readings give the same Δ as the code's (0.851 against 0.849).
    - Under the equilibrium test the paper describes, 0.2 % of sessions are equilibria on path (the code's one-period test: 49.7 %); re-optimizing gains a firm 5–41 % of its value.
    - The text's "more than 95 %" of deviations unprofitable is 93.6 % in its own Table A5.
    - Memoryless firms (δ = 0.95) price higher: Δ 0.958 (L24 holds); the code cannot run this (it sets δ = 0 without memory).
    - δ = 0 gives Δ 0.212, as the paper's own Fig. 3 shows: a quarter of the baseline needs no future.
    - Synchronous updating halves Δ (0.345); slower decay does not (0.727); constant ε = 0.05 never settles (0.559 at 10⁷).
    - Price increases draw the same responses as cuts (E&L hold); invitations are met by cuts; 91 % of "punishing" sessions are not RP-complete; firms re-paired across sessions earn Δ 0.125; the first 165 periods earn uniform random play's 0.497.
    - Lambin's Theorem 1 fails as a point prediction in every case (27 %, 13 %, 0.4 %, 3.6 % at I).

---

### Task 1: The collusion model in the core

**Files:**
- Create: `crates/sugarscape-core/src/collusion/{config,ran2,demand,learner,analysis,stats,view,world,presets,mod}.rs`, `crates/sugarscape-core/tests/collusion.rs`, `crates/sugarscape-core/tests/fixtures/calvano-sessions.json`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`

**Interfaces:**
- Consumes: `crate::model::{Model, ModelConfig, ModelKind, wrong_model}`, `crate::stats::{Series, Stats}`, `crate::export::history_csv`, `crate::render::{lerp, Rgb}`, `crate::opinions::Canvas`, `crate::rng::{self, SimRng}`, `crate::schema::{Apply, Param}`, `crate::presets::ModelPreset`, `crate::config::FieldError`, `crate::portable::{exp_neg, ln}`.
- Produces:
  - `collusion::{CollusionConfig, Grid, Exploration, Update, QInit, Ties, RngKind, EquilibriumCheck, Impulse, BestResponseTo, BELOW_BOTTOM, Q_BUDGET, schema, presets, as_coded}`.
  - `collusion::ran2::Ran2::{new, next_f64}`.
  - `collusion::demand::{Market, Game, five}`; `Game::{new, profile, profit, gain, nearest, static_best_response, horizon}` and its `firms, prices, grid, payoff, nash, monopoly, nash_profit, monopoly_profit`.
  - `collusion::learner::{Space, Draws, Firm, initial_prices, TIE}`; `Space::{of, encode, next, last, with_own}`; `Firm::{new, row, rescan, set}`.
  - `collusion::analysis::{Strategies, Cycle, Equilibrium, Response (with `returned` and the code's `settled`), HORIZON, limit_cycle, policy_values, optimal_values, best_responses, equilibrium, deviate, best_response_deviation, every_deviation, rp_complete, invitation, lambin_point, repair}`; `Response::{punishment_like, change}`.
  - `collusion::{SERIES, CollusionSnapshot, CollusionWorld, Outcome, CollusionInspection, CollusionMode, StateView}`; `CollusionWorld::{new, step (one tick), step_period, run (ticks), period, game, space, firms, state, current_state, is_finished, outcome, strategies, discounted_gain, horizon, inspect}` plus `pub tick`, `pub config` and `pub stats`.
  - `ModelKind::Collusion` (`"collusion"`), `ModelConfig::Collusion`, `ModelWorld::Collusion`; ten titles.

- [ ] **Step 1: Write the module**

Each file carries its tests (config: defaults, validation, budgets, reset fields, the schema; ran2: the Fortran's first draws; demand: the code's benchmark inputs, the grid to 7 decimals, payoff order, den Boer's 0.497 and −0.510; learner: state encoding, tie rules, incremental greedy against a full rescan; analysis: cycles, both equilibrium checks on grim trigger, punishment-like responses, the code's best-response reading, the invitation, policy and optimal values, Lambin's Theorem 1; world: eq. 8, convergence, the cap, ε's decay, synchronous updates, memory 0, the code's seeding, Inspect and live re-analysis, the Boltzmann and three-firm pins; presets).

Create `crates/sugarscape-core/src/collusion/config.rs` with exactly this content:

```rust
//! Algorithmic Collusion's parameters: Calvano, Calzolari, Denicolò and
//! Pastorello's (2020) baseline, every place where their paper and their
//! code differ, and the critics' tests (Asker, Fershtman & Pakes; Lambin;
//! Epivent & Lambin; den Boer, Meylahn & Schinkel) as named switches.

use serde::{Deserialize, Serialize};

use crate::config::FieldError;
use crate::schema::{Apply, Param};

/// The price grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grid {
    /// m prices evenly from p^N − ξ(p^M − p^N) to p^M + ξ(p^M − p^N) (CCDP).
    Calvano,
    /// m prices evenly from p^N − (1 + ξ)ζ to p^N + (1 + ξ)ζ, ζ = p^M − p^N:
    /// centered on the Nash price (den Boer, Meylahn & Schinkel's Ã).
    Symmetric,
    /// m prices evenly from 1.25 to `below_top` (Epivent & Lambin, App. C).
    BelowNash,
}

/// How a firm explores.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Exploration {
    /// ε = e^(−βt): with probability ε a price drawn uniformly (CCDP eq. 7).
    Decaying,
    /// ε fixed at `epsilon`.
    Constant,
    /// Choice probabilities ∝ exp((Q − max Q)/T), T starting at `temperature`
    /// and multiplied by (1 − `cooling`) each period (the code's type 2).
    Boltzmann,
    /// ε = 1 for `explore_for` periods, then 0 (Lambin 2024).
    TwoPhase,
}

/// Which Q-values a firm updates each period.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Only the price it charged (CCDP eq. 4).
    Asynchronous,
    /// Every price, toward the profit it would have earned against the
    /// rivals' actual prices and the state that would have followed (Asker,
    /// Fershtman & Pakes).
    Synchronous,
}

/// The starting Q-table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QInit {
    /// The discounted profit against a uniformly random rival, the same in
    /// every state (CCDP eq. 8).
    Calvano,
    Zero,
    /// Each cell uniform on [`q_low`, `q_high`] (Asker, Fershtman & Pakes's
    /// optimistic start with their 10–20; Lambin's theory).
    Random,
}

/// How the greedy price is chosen among equal Q-values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ties {
    /// The lowest price (CCDP p. 3275).
    Lowest,
    /// At random, and only when the greedy price's own value falls (the code).
    Random,
}

/// The random numbers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RngKind {
    /// The project's generator.
    Ours,
    /// Numerical Recipes' RAN2, seeded as the authors' code seeds session
    /// number `seed`: sessions match the code's period for period.
    Calvano,
}

/// What counts as an equilibrium after convergence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquilibriumCheck {
    /// Each firm's price is a best response to the rivals' strategies, by
    /// value iteration on the true Q (the paper's description).
    BestResponse,
    /// No one-period deviation pays when everyone, the deviator included,
    /// returns to the learned strategies (the code).
    OneShot,
}

/// The deviation the impulse response applies.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Impulse {
    /// One period at the static best response (CCDP Fig. 4).
    BestResponseDown,
    /// One period at every other grid price (CCDP-A; Epivent & Lambin's Table 1).
    EveryPrice,
    /// One grid step up, the rival forced to match the next period on, the
    /// deviator regaining control after `invitation_hold` periods (Epivent
    /// & Lambin's Fig. 2).
    Invitation,
    /// One period one grid step up.
    Up,
}

/// Which state the static best response answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BestResponseTo {
    /// The rivals' prices at the pre-deviation state (the paper's words).
    Path,
    /// The rivals' prices at the state numbered by the cycle position (1, 2,
    /// …): the code passes the position where it means the state.
    Code,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CollusionConfig {
    /// n.
    pub firms: u32,
    /// m.
    pub prices: u32,
    pub grid: Grid,
    /// ξ.
    pub xi: f64,
    /// The top of the `below_nash` grid (Epivent & Lambin's text: 1.47).
    pub below_top: f64,
    /// cᵢ.
    pub cost: f64,
    /// Firm 2's cost, if different (CCDP-A's asymmetric firms).
    pub cost2: Option<f64>,
    /// aᵢ.
    pub quality: f64,
    /// a₀.
    pub outside: f64,
    /// μ.
    pub mu: f64,
    /// k: 0, 1 or 2 periods of everyone's prices.
    pub memory: u32,
    /// α.
    pub alpha: f64,
    /// β, per period.
    pub beta: f64,
    /// δ.
    pub delta: f64,
    pub exploration: Exploration,
    pub epsilon: f64,
    pub temperature: f64,
    pub cooling: f64,
    pub explore_for: u32,
    pub update: Update,
    pub q_init: QInit,
    pub q_low: f64,
    pub q_high: f64,
    pub ties: Ties,
    pub rng: RngKind,
    /// The stop when strategies never settle.
    pub cap: u32,
    /// Periods of unchanged strategies that count as convergence.
    pub window: u32,
    pub equilibrium_check: EquilibriumCheck,
    pub impulse: Impulse,
    pub best_response_to: BestResponseTo,
    pub invitation_hold: u32,
    /// Periods a tick runs: a session (about 2 × 10⁶ periods) takes a few
    /// thousand ticks, and each charted point summarizes one tick.
    pub periods_per_tick: u32,
}

impl Default for CollusionConfig {
    /// CCDP's baseline (p. 3274): two firms, 15 prices, one period of memory.
    fn default() -> Self {
        CollusionConfig {
            firms: 2,
            prices: 15,
            grid: Grid::Calvano,
            xi: 0.1,
            below_top: 1.47,
            cost: 1.0,
            cost2: None,
            quality: 2.0,
            outside: 0.0,
            mu: 0.25,
            memory: 1,
            alpha: 0.15,
            beta: 4e-6,
            delta: 0.95,
            exploration: Exploration::Decaying,
            epsilon: 0.05,
            temperature: 1000.0,
            cooling: 1e-5,
            explore_for: 1000,
            update: Update::Asynchronous,
            q_init: QInit::Calvano,
            q_low: 10.0,
            q_high: 20.0,
            ties: Ties::Lowest,
            rng: RngKind::Ours,
            cap: 1_000_000_000,
            window: 100_000,
            equilibrium_check: EquilibriumCheck::BestResponse,
            impulse: Impulse::BestResponseDown,
            best_response_to: BestResponseTo::Path,
            invitation_hold: 5,
            periods_per_tick: 1000,
        }
    }
}

/// The bottom of the `below_nash` grid (Epivent & Lambin, App. C).
pub const BELOW_BOTTOM: f64 = 1.25;
/// The most Q-values a world may hold (states × prices × firms).
pub const Q_BUDGET: u64 = 1 << 24;

impl CollusionConfig {
    /// The number of states, m^(n·k).
    pub fn states(&self) -> u64 {
        u64::from(self.prices).saturating_pow(self.firms * self.memory)
    }

    /// Firm `i`'s marginal cost.
    pub fn cost_of(&self, i: usize) -> f64 {
        match (i, self.cost2) {
            (1, Some(c)) => c,
            _ => self.cost,
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        check(
            (2..=4).contains(&self.firms),
            "firms",
            "must be between 2 and 4",
        );
        check(
            (2..=100).contains(&self.prices),
            "prices",
            "must be between 2 and 100",
        );
        check(
            (0.0..=1.0).contains(&self.xi),
            "xi",
            "must be between 0 and 1",
        );
        check(
            self.below_top > BELOW_BOTTOM && self.below_top <= 10.0,
            "below_top",
            "must be above 1.25 and at most 10",
        );
        check(
            (0.0..=10.0).contains(&self.cost),
            "cost",
            "must be between 0 and 10",
        );
        check(
            self.cost2.is_none_or(|c| (0.0..=10.0).contains(&c)),
            "cost2",
            "must be between 0 and 10",
        );
        check(
            self.quality > self.cost && self.quality <= 20.0,
            "quality",
            "must be above the cost and at most 20",
        );
        check(
            (-10.0..=10.0).contains(&self.outside),
            "outside",
            "must be between −10 and 10",
        );
        check(
            (0.01..=10.0).contains(&self.mu),
            "mu",
            "must be between 0.01 and 10",
        );
        check(self.memory <= 2, "memory", "must be 0, 1 or 2");
        check(
            self.states() * u64::from(self.prices) * u64::from(self.firms) <= Q_BUDGET,
            "memory",
            "too many states for this many firms and prices (states × prices × firms must be at most 2^24)",
        );
        check(
            self.alpha > 0.0 && self.alpha <= 1.0,
            "alpha",
            "must be above 0 and at most 1",
        );
        check(
            (0.0..=1.0).contains(&self.beta),
            "beta",
            "must be between 0 and 1",
        );
        check(
            (0.0..1.0).contains(&self.delta),
            "delta",
            "must be at least 0 and below 1",
        );
        check(
            (0.0..=1.0).contains(&self.epsilon),
            "epsilon",
            "must be between 0 and 1",
        );
        check(
            self.temperature > 0.0 && self.temperature <= 1e6,
            "temperature",
            "must be above 0 and at most 1000000",
        );
        check(
            (0.0..1.0).contains(&self.cooling),
            "cooling",
            "must be at least 0 and below 1",
        );
        check(
            self.q_low <= self.q_high && self.q_low.abs() <= 1e6 && self.q_high.abs() <= 1e6,
            "q_high",
            "must be at least q_low (both within ±1000000)",
        );
        check(self.cap >= 1, "cap", "must be at least 1");
        check(
            self.window >= 1 && self.window <= self.cap,
            "window",
            "must be at least 1 and at most the cap",
        );
        check(
            (1..=100).contains(&self.invitation_hold),
            "invitation_hold",
            "must be between 1 and 100",
        );
        check(
            (1..=1_000_000).contains(&self.periods_per_tick),
            "periods_per_tick",
            "must be between 1 and 1000000",
        );
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    /// Fields that change only on reset and differ in `next`.
    pub(crate) fn structural_changes(&self, next: &CollusionConfig) -> Vec<FieldError> {
        let mut out = Vec::new();
        for (field, same) in [
            ("firms", self.firms == next.firms),
            ("prices", self.prices == next.prices),
            ("grid", self.grid == next.grid),
            ("xi", self.xi == next.xi),
            ("below_top", self.below_top == next.below_top),
            ("cost", self.cost == next.cost),
            ("cost2", self.cost2 == next.cost2),
            ("quality", self.quality == next.quality),
            ("outside", self.outside == next.outside),
            ("mu", self.mu == next.mu),
            ("memory", self.memory == next.memory),
            ("beta", self.beta == next.beta),
            ("exploration", self.exploration == next.exploration),
            ("epsilon", self.epsilon == next.epsilon),
            ("temperature", self.temperature == next.temperature),
            ("cooling", self.cooling == next.cooling),
            ("explore_for", self.explore_for == next.explore_for),
            ("update", self.update == next.update),
            ("q_init", self.q_init == next.q_init),
            ("q_low", self.q_low == next.q_low),
            ("q_high", self.q_high == next.q_high),
            ("ties", self.ties == next.ties),
            ("rng", self.rng == next.rng),
            ("cap", self.cap == next.cap),
            ("window", self.window == next.window),
            (
                "periods_per_tick",
                self.periods_per_tick == next.periods_per_tick,
            ),
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
        Param::integer("Market", "firms", "Firms (n)", (2, 4), Reset),
        Param::number("Market", "cost", "Cost (c)", (0.0, 10.0, 0.05), Reset),
        Param::number("Market", "cost2", "Firm 2's cost", (0.0, 10.0, 0.05), Reset)
            .nullable()
            .with_help("Empty: the same as the others (CCDP-A's asymmetric firms set it lower)."),
        Param::number("Market", "quality", "Quality (a)", (0.0, 20.0, 0.05), Reset),
        Param::number("Market", "outside", "Outside good (a₀)", (-10.0, 10.0, 0.05), Reset),
        Param::number("Market", "mu", "Differentiation (μ)", (0.01, 10.0, 0.01), Reset)
            .with_help("Logit demand's horizontal differentiation: 0.25 in the baseline."),
        Param::integer("Prices", "prices", "Prices (m)", (2, 100), Reset),
        Param::choice(
            "Prices",
            "grid",
            "Grid",
            &[
                ("calvano", "Around Nash and monopoly (CCDP)"),
                ("symmetric", "Centered on Nash (den Boer et al.)"),
                ("below_nash", "From 1.25 up to Nash (Epivent & Lambin)"),
            ],
            Reset,
        ),
        Param::number("Prices", "xi", "Extension (ξ)", (0.0, 1.0, 0.05), Reset),
        Param::number("Prices", "below_top", "Top price", (1.26, 10.0, 0.01), Reset)
            .shown_if("grid", "below_nash")
            .with_help("Epivent & Lambin's text: 1.47; the Nash price is 1.47293."),
        Param::integer("Learning", "memory", "Memory (k)", (0, 2), Reset)
            .with_help("Periods of everyone's prices a firm conditions on. 0: one state, so no punishment is possible (Lambin's test keeps δ)."),
        Param::number("Learning", "alpha", "Learning rate (α)", (0.0025, 1.0, 0.0025), Live),
        Param::number("Learning", "delta", "Discount factor (δ)", (0.0, 0.99, 0.01), Live),
        Param::choice(
            "Learning",
            "update",
            "Update",
            &[
                ("asynchronous", "The price charged (CCDP)"),
                ("synchronous", "Every price (Asker, Fershtman & Pakes)"),
            ],
            Reset,
        ),
        Param::choice(
            "Learning",
            "q_init",
            "Starting Q",
            &[
                ("calvano", "Profit against a random rival (CCDP)"),
                ("zero", "Zero"),
                ("random", "Uniform between two values"),
            ],
            Reset,
        ),
        Param::number("Learning", "q_low", "Lowest", (-1000.0, 1000.0, 0.5), Reset)
            .shown_if("q_init", "random"),
        Param::number("Learning", "q_high", "Highest", (-1000.0, 1000.0, 0.5), Reset)
            .shown_if("q_init", "random"),
        Param::choice(
            "Learning",
            "ties",
            "Equal Q-values",
            &[
                ("lowest", "The lowest price (the paper)"),
                ("random", "At random (the code)"),
            ],
            Reset,
        ),
        Param::choice(
            "Exploration",
            "exploration",
            "Exploration",
            &[
                ("decaying", "ε = e^(−βt) (CCDP)"),
                ("constant", "A constant ε"),
                ("boltzmann", "Boltzmann (the code's type 2)"),
                ("two_phase", "All at first, then none (Lambin)"),
            ],
            Reset,
        ),
        Param::number("Exploration", "beta", "Decay (β)", (0.0, 1.0, 1e-6), Reset)
            .shown_if("exploration", "decaying"),
        Param::number("Exploration", "epsilon", "ε", (0.0, 1.0, 0.005), Reset)
            .shown_if("exploration", "constant"),
        Param::number("Exploration", "temperature", "Starting temperature", (0.001, 1e6, 1.0), Reset)
            .shown_if("exploration", "boltzmann"),
        Param::number("Exploration", "cooling", "Cooling", (0.0, 0.999, 1e-6), Reset)
            .shown_if("exploration", "boltzmann"),
        Param::integer("Exploration", "explore_for", "Periods of exploring", (0, 10_000_000), Reset)
            .shown_if("exploration", "two_phase"),
        Param::choice(
            "Session",
            "rng",
            "Random numbers",
            &[
                ("ours", "The playground's"),
                ("calvano", "The authors' (session number = seed)"),
            ],
            Reset,
        )
        .with_help("The authors' generator and seeding: a session runs period for period as in their code."),
        Param::integer("Session", "cap", "Stop at most at period", (1, 4_000_000_000), Reset)
            .with_help("The paper: 10⁹. The code: 1.25 × 10⁹."),
        Param::integer("Session", "window", "Converged after", (1, 10_000_000), Reset)
            .with_help("Periods of unchanged strategies (CCDP: 100 000)."),
        Param::choice(
            "Analysis",
            "equilibrium_check",
            "Equilibrium",
            &[
                ("best_response", "A best response (the paper)"),
                ("one_shot", "No one-period deviation pays (the code)"),
            ],
            Live,
        ),
        Param::choice(
            "Analysis",
            "impulse",
            "Deviation",
            &[
                ("best_response_down", "To the static best response (CCDP Fig. 4)"),
                ("every_price", "To every price (Epivent & Lambin's Table 1)"),
                ("invitation", "An invitation: up a step, the rival made to follow"),
                ("up", "Up one step"),
            ],
            Live,
        ),
        Param::choice(
            "Analysis",
            "best_response_to",
            "Best response to",
            &[
                ("path", "The rival's price before the deviation (the paper)"),
                ("code", "The state numbered by the cycle position (the code)"),
            ],
            Live,
        )
        .shown_if("impulse", "best_response_down"),
        Param::integer("Analysis", "invitation_hold", "Deviator held for", (1, 100), Live)
            .shown_if("impulse", "invitation"),
        Param::integer("Session", "periods_per_tick", "Periods a tick", (1, 1_000_000), Reset)
            .with_help("A tick runs this many periods; the charts count ticks, each point summarizing one tick. 1 000: a session takes a few thousand ticks."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelConfig, ModelWorld};

    #[test]
    fn defaults_are_calvano_s_baseline() {
        let c = CollusionConfig::default();
        assert_eq!((c.firms, c.prices, c.memory, c.states()), (2, 15, 1, 225));
        assert_eq!((c.alpha, c.beta, c.delta), (0.15, 4e-6, 0.95));
        assert_eq!(
            (c.ties, c.cap, c.window),
            (Ties::Lowest, 1_000_000_000, 100_000)
        );
        assert!(c.validate().is_ok());
    }

    #[test]
    fn validation_names_fields() {
        let bad = CollusionConfig {
            firms: 5,
            prices: 1,
            xi: 2.0,
            mu: 0.0,
            alpha: 0.0,
            delta: 1.0,
            q_low: 5.0,
            q_high: 1.0,
            window: 0,
            ..CollusionConfig::default()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        assert_eq!(
            fields,
            ["firms", "prices", "xi", "mu", "alpha", "delta", "q_high", "window"]
        );
    }

    #[test]
    fn big_tables_are_refused() {
        let c = CollusionConfig {
            firms: 4,
            memory: 2,
            ..CollusionConfig::default()
        };
        assert_eq!(c.validate().unwrap_err()[0].field, "memory");
        let ok = CollusionConfig {
            firms: 2,
            memory: 2,
            ..CollusionConfig::default()
        };
        assert_eq!(ok.states(), 50_625);
        assert!(ok.validate().is_ok());
        assert_eq!(
            CollusionConfig {
                memory: 0,
                ..CollusionConfig::default()
            }
            .states(),
            1
        );
    }

    #[test]
    fn firm_two_may_have_its_own_cost() {
        let c = CollusionConfig {
            cost2: Some(0.75),
            ..CollusionConfig::default()
        };
        assert_eq!((c.cost_of(0), c.cost_of(1)), (1.0, 0.75));
        let json = serde_json::to_value(CollusionConfig::default()).unwrap();
        assert!(json["cost2"].is_null());
    }

    #[test]
    fn learning_rules_change_only_on_reset() {
        let next = CollusionConfig {
            update: Update::Synchronous,
            alpha: 0.1,
            ..CollusionConfig::default()
        };
        let changes = CollusionConfig::default().structural_changes(&next);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].field, "update");
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Collusion(CollusionConfig {
            cost2: Some(1.0),
            ..CollusionConfig::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

Create `crates/sugarscape-core/src/collusion/ran2.rs` with exactly this content:

```rust
//! Numerical Recipes' RAN2 (L'Ecuyer's combined generator with a
//! Bays–Durham shuffle), exactly as the authors' `generic_routines.f90`
//! writes it: 32-bit integer arithmetic by Schrage's method, so every
//! platform gives the same draws as their Fortran.

const IM1: i32 = 2_147_483_563;
const IM2: i32 = 2_147_483_399;
const IMM1: i32 = IM1 - 1;
const IA1: i32 = 40_014;
const IA2: i32 = 40_692;
const IQ1: i32 = 53_668;
const IQ2: i32 = 52_774;
const IR1: i32 = 12_211;
const IR2: i32 = 3_791;
const NDIV: i32 = 1 + IMM1 / 32;
const AM: f64 = 1.0 / IM1 as f64;
const RNMX: f64 = 1.0 - 1.2e-7;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ran2 {
    idum: i32,
    idum2: i32,
    iy: i32,
    iv: [i32; 32],
}

impl Ran2 {
    /// A stream seeded as the code seeds it: `idum` negative (the code uses
    /// −session, or −1 for initial prices), `idum2` = 123456789, `iv` and
    /// `iy` zero; the first draw initializes.
    pub fn new(idum: i32) -> Self {
        Ran2 {
            idum,
            idum2: 123_456_789,
            iy: 0,
            iv: [0; 32],
        }
    }

    /// A uniform deviate in (0, 1).
    pub fn next_f64(&mut self) -> f64 {
        if self.idum <= 0 {
            self.idum = (-self.idum).max(1);
            self.idum2 = self.idum;
            for j in (1..=40).rev() {
                let k = self.idum / IQ1;
                self.idum = IA1 * (self.idum - k * IQ1) - k * IR1;
                if self.idum < 0 {
                    self.idum += IM1;
                }
                if j <= 32 {
                    self.iv[j - 1] = self.idum;
                }
            }
            self.iy = self.iv[0];
        }
        let k = self.idum / IQ1;
        self.idum = IA1 * (self.idum - k * IQ1) - k * IR1;
        if self.idum < 0 {
            self.idum += IM1;
        }
        let k = self.idum2 / IQ2;
        self.idum2 = IA2 * (self.idum2 - k * IQ2) - k * IR2;
        if self.idum2 < 0 {
            self.idum2 += IM2;
        }
        let j = (self.iy / NDIV) as usize;
        self.iy = self.iv[j] - self.idum2;
        self.iv[j] = self.idum;
        if self.iy < 1 {
            self.iy += IMM1;
        }
        (AM * f64::from(self.iy)).min(RNMX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_draws_are_the_fortran_s() {
        // The authors' ran2, compiled with gfortran 14, printed to 17 digits.
        let mut one = Ran2::new(-1);
        let mut seven = Ran2::new(-7);
        for want in [0.2853808990946861, 0.2533581892659171, 0.09346853100919404] {
            assert_eq!(one.next_f64(), want);
        }
        for want in [0.45206034994923033, 0.8885129427181557, 0.3178740604870464] {
            assert_eq!(seven.next_f64(), want);
        }
    }

    #[test]
    fn draws_are_uniform_and_reproducible() {
        let mut a = Ran2::new(-7);
        let mut b = Ran2::new(-7);
        let xs: Vec<f64> = (0..10_000).map(|_| a.next_f64()).collect();
        assert!(xs.iter().all(|&x| x > 0.0 && x < 1.0));
        assert_eq!(xs, (0..10_000).map(|_| b.next_f64()).collect::<Vec<_>>());
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        assert!((mean - 0.5).abs() < 0.01, "{mean}");
        assert_ne!(Ran2::new(-1).next_f64(), Ran2::new(-2).next_f64());
    }
}
```

Create `crates/sugarscape-core/src/collusion/demand.rs` with exactly this content:

```rust
//! The stage game: logit demand (CCDP eq. 5), the one-shot Bertrand–Nash and
//! joint-monopoly prices, the price grid and the payoff table.

use super::config::{CollusionConfig, Grid, BELOW_BOTTOM};
use crate::portable::exp_neg;

/// The market's demand parameters, per firm.
#[derive(Clone, Debug, PartialEq)]
pub struct Market {
    pub quality: Vec<f64>,
    pub cost: Vec<f64>,
    pub outside: f64,
    pub mu: f64,
}

impl Market {
    pub fn of(c: &CollusionConfig) -> Self {
        let n = c.firms as usize;
        Market {
            quality: vec![c.quality; n],
            cost: (0..n).map(|i| c.cost_of(i)).collect(),
            outside: c.outside,
            mu: c.mu,
        }
    }

    pub fn firms(&self) -> usize {
        self.cost.len()
    }

    /// Market shares at prices `p`: exp((aᵢ − pᵢ)/μ) / (Σ exp((aⱼ − pⱼ)/μ) +
    /// exp(a₀/μ)), every exponent shifted by the largest so the portable
    /// exp sees only non-positive arguments.
    pub fn shares(&self, p: &[f64]) -> Vec<f64> {
        let z: Vec<f64> = (0..self.firms())
            .map(|i| (self.quality[i] - p[i]) / self.mu)
            .collect();
        let z0 = self.outside / self.mu;
        let top = z.iter().copied().fold(z0, f64::max);
        let e: Vec<f64> = z.iter().map(|&x| exp_neg(x - top)).collect();
        let total = e.iter().sum::<f64>() + exp_neg(z0 - top);
        e.iter().map(|x| x / total).collect()
    }

    pub fn profits(&self, p: &[f64]) -> Vec<f64> {
        self.shares(p)
            .iter()
            .enumerate()
            .map(|(i, q)| (p[i] - self.cost[i]) * q)
            .collect()
    }

    /// The root of an increasing `f` on [lo, hi], by bisection to the last bit.
    fn bisect(lo: f64, hi: f64, mut f: impl FnMut(f64) -> f64) -> f64 {
        let (mut lo, mut hi) = (lo, hi);
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            if f(mid) < 0.0 {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        0.5 * (lo + hi)
    }

    /// The one-shot Bertrand–Nash prices: each firm's best response solves
    /// pᵢ − cᵢ = μ/(1 − qᵢ) given the others' prices (increasing in pᵢ, so
    /// bisection), repeated until no price moves.
    pub fn nash(&self) -> Vec<f64> {
        let mut p: Vec<f64> = self.cost.iter().map(|c| c + self.mu).collect();
        for _ in 0..10_000 {
            let mut moved: f64 = 0.0;
            for i in 0..self.firms() {
                let mut trial = p.clone();
                let best = Self::bisect(self.cost[i], self.cost[i] + 1000.0 * self.mu, |x| {
                    trial[i] = x;
                    x - self.cost[i] - self.mu / (1.0 - self.shares(&trial)[i])
                });
                moved = moved.max((best - p[i]).abs());
                p[i] = best;
            }
            if moved == 0.0 {
                break;
            }
        }
        p
    }

    /// The joint-profit maximum: equal markups m = μ/(1 − Σq), the root of
    /// an increasing function of m.
    pub fn monopoly(&self) -> Vec<f64> {
        let at = |m: f64| -> Vec<f64> { self.cost.iter().map(|c| c + m).collect() };
        let m = Self::bisect(0.0, 1000.0 * self.mu, |m| {
            m - self.mu / (1.0 - self.shares(&at(m)).iter().sum::<f64>())
        });
        at(m)
    }
}

/// Rounds to 5 decimals, as the authors' inputs are.
pub fn five(x: f64) -> f64 {
    (x * 1e5).round() / 1e5
}

/// The stage game a session is played on.
#[derive(Clone, Debug, PartialEq)]
pub struct Game {
    pub firms: usize,
    pub prices: usize,
    /// Each firm's grid, `grid[i][a]`.
    pub grid: Vec<Vec<f64>>,
    /// Profits: `payoff[profile * firms + i]`, profile = Σ aᵢ·m^(n−1−i).
    pub payoff: Vec<f64>,
    pub nash: Vec<f64>,
    pub monopoly: Vec<f64>,
    /// π^N and π^M per firm, at the (rounded) benchmark prices.
    pub nash_profit: Vec<f64>,
    pub monopoly_profit: Vec<f64>,
}

impl Game {
    pub fn new(c: &CollusionConfig) -> Self {
        let market = Market::of(c);
        let n = c.firms as usize;
        let m = c.prices as usize;
        let nash: Vec<f64> = market.nash().into_iter().map(five).collect();
        let monopoly: Vec<f64> = market.monopoly().into_iter().map(five).collect();
        let grid: Vec<Vec<f64>> = (0..n)
            .map(|i| {
                let zeta = monopoly[i] - nash[i];
                let (lo, hi) = match c.grid {
                    Grid::Calvano => (nash[i] - c.xi * zeta, monopoly[i] + c.xi * zeta),
                    Grid::Symmetric => {
                        (nash[i] - (1.0 + c.xi) * zeta, nash[i] + (1.0 + c.xi) * zeta)
                    }
                    Grid::BelowNash => (BELOW_BOTTOM, c.below_top),
                };
                // The code's construction: the ends set, the inside by
                // cumulative addition.
                let step = (hi - lo) / (m - 1) as f64;
                let mut g = vec![0.0; m];
                g[0] = lo;
                for a in 1..m - 1 {
                    g[a] = g[a - 1] + step;
                }
                g[m - 1] = hi;
                g
            })
            .collect();
        let profiles = m.pow(n as u32);
        let mut payoff = vec![0.0; profiles * n];
        let mut prices = vec![0.0; n];
        for profile in 0..profiles {
            let mut rest = profile;
            for i in (0..n).rev() {
                prices[i] = grid[i][rest % m];
                rest /= m;
            }
            let pi = market.profits(&prices);
            payoff[profile * n..(profile + 1) * n].copy_from_slice(&pi);
        }
        Game {
            firms: n,
            prices: m,
            nash_profit: market.profits(&nash),
            monopoly_profit: market.profits(&monopoly),
            grid,
            payoff,
            nash,
            monopoly,
        }
    }

    /// The profile index of `actions` (firm 0 most significant).
    pub fn profile(&self, actions: &[u8]) -> usize {
        actions
            .iter()
            .fold(0, |acc, &a| acc * self.prices + usize::from(a))
    }

    pub fn profit(&self, actions: &[u8], i: usize) -> f64 {
        self.payoff[self.profile(actions) * self.firms + i]
    }

    /// Firm `i`'s profit gain (π − π^N)/(π^M − π^N).
    pub fn gain(&self, i: usize, profit: f64) -> f64 {
        (profit - self.nash_profit[i]) / (self.monopoly_profit[i] - self.nash_profit[i])
    }

    /// The grid index nearest `price` for firm `i`.
    pub fn nearest(&self, i: usize, price: f64) -> u8 {
        let mut best = 0;
        for a in 1..self.prices {
            if (self.grid[i][a] - price).abs() < (self.grid[i][best] - price).abs() {
                best = a;
            }
        }
        best as u8
    }

    /// The best one-period price for firm `i` against `actions` (lowest on ties).
    pub fn static_best_response(&self, actions: &[u8], i: usize) -> u8 {
        let mut a = actions.to_vec();
        let mut best = (0u8, f64::NEG_INFINITY);
        for p in 0..self.prices as u8 {
            a[i] = p;
            let v = self.profit(&a, i);
            if v > best.1 {
                best = (p, v);
            }
        }
        best.0
    }

    /// den Boer, Meylahn & Schinkel's effective horizon T_δ =
    /// ⌈ln(π_min/(1000 π_max))/ln δ⌉ over the positive profits in the table
    /// (1 when δ = 0).
    pub fn horizon(&self, delta: f64) -> u32 {
        if delta <= 0.0 {
            return 1;
        }
        let positive = self.payoff.iter().copied().filter(|&p| p > 0.0);
        let (lo, hi) = positive.fold((f64::INFINITY, 0.0_f64), |(lo, hi), p| {
            (lo.min(p), hi.max(p))
        });
        if !lo.is_finite() {
            return 1;
        }
        let t = crate::portable::ln(lo / (1000.0 * hi)) / crate::portable::ln(delta);
        t.ceil().max(1.0) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game(edit: impl FnOnce(&mut CollusionConfig)) -> Game {
        let mut c = CollusionConfig::default();
        edit(&mut c);
        Game::new(&c)
    }

    #[test]
    fn the_benchmarks_are_the_codes_inputs() {
        let g = game(|_| {});
        assert_eq!((g.nash[0], g.monopoly[0]), (1.47293, 1.92498));
        let unrounded = Market::of(&CollusionConfig::default());
        assert!((unrounded.nash()[0] - 1.472927).abs() < 1e-6);
        assert!((unrounded.monopoly()[0] - 1.924981).abs() < 1e-6);
        assert!((g.nash_profit[0] - 0.22293).abs() < 1e-5);
        assert!((g.monopoly_profit[0] - 0.33749).abs() < 1e-5);
        let three = game(|c| c.firms = 3);
        assert_eq!((three.nash[0], three.monopoly[0]), (1.37016, 2.0));
        let wide = game(|c| c.mu = 0.5);
        assert_eq!((wide.nash[0], wide.monopoly[0]), (1.79947, 2.18741));
    }

    #[test]
    // 1.4142 is the code's input for firm 1's Nash price at c₂ = 0.75, not √2.
    #[allow(clippy::approx_constant)]
    fn asymmetric_costs_match_the_codes_inputs() {
        for (c2, n1, n2, m1, m2) in [
            (0.875, 1.44135, 1.39588, 1.97674, 1.85174),
            (0.75, 1.4142, 1.32539, 2.04051, 1.79051),
            (0.5, 1.37233, 1.20377, 2.1984, 1.6984),
            (0.25, 1.34369, 1.10519, 2.38411, 1.63411),
        ] {
            let g = game(|c| c.cost2 = Some(c2));
            assert_eq!(
                (g.nash[0], g.nash[1], g.monopoly[0], g.monopoly[1]),
                (n1, n2, m1, m2),
                "c2 {c2}"
            );
        }
    }

    #[test]
    fn the_grid_is_the_codes() {
        let g = game(|_| {});
        // The authors' run prints its grid to 7 decimals (A_res.txt).
        let want = [
            1.4277250, 1.4664721, 1.5052193, 1.5439664, 1.5827136, 1.6214607, 1.6602079, 1.6989550,
            1.7377021, 1.7764493, 1.8151964, 1.8539436, 1.8926907, 1.9314379, 1.9701850,
        ];
        for (a, w) in want.iter().enumerate() {
            assert!(
                (g.grid[0][a] - w).abs() < 6e-8,
                "price {a}: {}",
                g.grid[0][a]
            );
        }
        assert_eq!(g.grid[0][14], 1.92498 + 0.1 * (1.92498 - 1.47293));
        let below = game(|c| c.grid = Grid::BelowNash);
        assert_eq!((below.grid[0][0], below.grid[0][14]), (1.25, 1.47));
        let sym = game(|c| {
            c.grid = Grid::Symmetric;
            c.xi = 0.0;
        });
        assert!((sym.grid[0][7] - 1.47293).abs() < 1e-12, "centered on Nash");
        assert_eq!(sym.grid[0][14], 1.92498);
    }

    #[test]
    fn payoffs_follow_the_profile_order() {
        let g = game(|_| {});
        let m = Market::of(&CollusionConfig::default());
        let p = m.profits(&[g.grid[0][3], g.grid[1][9]]);
        assert_eq!(g.profit(&[3, 9], 0), p[0]);
        assert_eq!(g.profit(&[3, 9], 1), p[1]);
        assert_eq!(g.profile(&[3, 9]), 3 * 15 + 9);
        // Symmetric prices: Δ at index 9 (1.7377) is 0.794 (Lambin's I at δ = 0.95).
        let pi = g.profit(&[8, 8], 0);
        assert!((g.gain(0, pi) - 0.794).abs() < 0.001, "{}", g.gain(0, pi));
    }

    #[test]
    fn uniform_random_pricing_earns_den_boer_s_0_497() {
        let g = game(|_| {});
        let mean: f64 = (0..225).map(|k| g.payoff[k * 2]).sum::<f64>() / 225.0;
        assert!(
            (g.gain(0, mean) - 0.497).abs() < 0.0005,
            "{}",
            g.gain(0, mean)
        );
        let sym = game(|c| {
            c.grid = Grid::Symmetric;
            c.xi = 0.0;
        });
        let mean: f64 = (0..225).map(|k| sym.payoff[k * 2]).sum::<f64>() / 225.0;
        assert!(
            (sym.gain(0, mean) + 0.510).abs() < 0.0005,
            "{}",
            sym.gain(0, mean)
        );
        assert_eq!(g.horizon(0.95), 165);
        assert_eq!(g.horizon(0.0), 1);
    }

    #[test]
    fn the_static_best_response_undercuts_a_high_rival() {
        let g = game(|_| {});
        let br = g.static_best_response(&[14, 14], 0);
        assert!(br < 14 && br > 0, "{br}");
        assert_eq!(g.nearest(0, 1.47293), 1);
    }
}
```

Create `crates/sugarscape-core/src/collusion/learner.rs` with exactly this content:

```rust
//! The learners: the state space (the last k periods' prices), a firm's
//! Q-table and greedy strategy, kept incrementally as the authors' code
//! keeps them, and the random numbers.

use rand::Rng;

use super::config::{CollusionConfig, Ties};
use super::ran2::Ran2;
use crate::rng::{self, SimRng};

/// Two Q-values this close are equal (the code's `AreEqualReals`:
/// `|a − b| ≤ EPSILON(1d0)`).
pub const TIE: f64 = f64::EPSILON;

/// The states: everyone's prices in the last k periods, the latest first,
/// firm 0 most significant (the code's `computeStateNumber`, 0-based).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Space {
    pub firms: usize,
    pub prices: usize,
    pub memory: usize,
    pub states: usize,
}

impl Space {
    pub fn of(c: &CollusionConfig) -> Self {
        Space {
            firms: c.firms as usize,
            prices: c.prices as usize,
            memory: c.memory as usize,
            states: c.states() as usize,
        }
    }

    /// The state from prices `p[d][i]` (d periods ago, firm i).
    pub fn encode(&self, p: &[Vec<u8>]) -> usize {
        let mut s = 0;
        for depth in p.iter().take(self.memory) {
            for &a in depth {
                s = s * self.prices + usize::from(a);
            }
        }
        s
    }

    /// The state after everyone plays `actions` in state `s`.
    pub fn next(&self, s: usize, actions: &[u8]) -> usize {
        if self.memory == 0 {
            return 0;
        }
        let block = self.prices.pow(self.firms as u32);
        let code = actions
            .iter()
            .fold(0, |acc, &a| acc * self.prices + usize::from(a));
        code * block.pow(self.memory as u32 - 1) + s / block
    }

    /// The prices the state records for the last period (none without memory).
    pub fn last(&self, s: usize) -> Option<Vec<u8>> {
        if self.memory == 0 {
            return None;
        }
        let block = self.prices.pow(self.firms as u32);
        let mut code = s / block.pow(self.memory as u32 - 1);
        let mut out = vec![0u8; self.firms];
        for i in (0..self.firms).rev() {
            out[i] = (code % self.prices) as u8;
            code /= self.prices;
        }
        Some(out)
    }

    /// `s` with firm `i`'s latest price replaced by `a` (the state a
    /// counterfactual price would have led to).
    pub fn with_own(&self, s: usize, i: usize, a: u8) -> usize {
        let Some(last) = self.last(s) else {
            return 0;
        };
        let place = self.prices.pow((self.firms - 1 - i) as u32)
            * self.prices.pow((self.firms * (self.memory - 1)) as u32);
        s - usize::from(last[i]) * place + usize::from(a) * place
    }
}

/// The random numbers a session draws.
#[derive(Clone, Debug)]
pub enum Draws {
    Ours(SimRng),
    /// The code's streams: exploration, and ties and Q initialization, both
    /// seeded −session.
    Calvano(Box<[Ran2; 2]>),
}

impl Draws {
    pub fn new(c: &CollusionConfig, seed: u64) -> Self {
        match c.rng {
            super::config::RngKind::Ours => Draws::Ours(rng::seeded(seed)),
            super::config::RngKind::Calvano => {
                let session = -(seed.clamp(1, i32::MAX as u64) as i32);
                Draws::Calvano(Box::new([Ran2::new(session), Ran2::new(session)]))
            }
        }
    }

    /// One period's exploration uniforms, in the code's order: u(1, firm 1),
    /// u(1, firm 2), …, then u(2, firm 1), ….
    pub fn explore(&mut self, out: &mut [[f64; 2]]) {
        for k in 0..2 {
            for u in out.iter_mut() {
                u[k] = match self {
                    Draws::Ours(r) => r.gen::<f64>(),
                    Draws::Calvano(streams) => streams[0].next_f64(),
                };
            }
        }
    }

    /// A uniform for breaking a tie or drawing a starting Q-value.
    pub fn tie(&mut self) -> f64 {
        match self {
            Draws::Ours(r) => r.gen::<f64>(),
            Draws::Calvano(streams) => streams[1].next_f64(),
        }
    }

    /// A price index uniform on 0..m.
    pub fn price(&mut self, m: usize) -> u8 {
        match self {
            Draws::Ours(r) => r.gen_range(0..m as u32) as u8,
            Draws::Calvano(streams) => (m as f64 * streams[1].next_f64()) as u8,
        }
    }
}

/// The starting prices: under `calvano`, the code's shared stream seeded −1,
/// session s taking draws after the (s − 1)·k·n before it; otherwise drawn.
pub fn initial_prices(c: &CollusionConfig, seed: u64, draws: &mut Draws) -> Vec<Vec<u8>> {
    let (n, m) = (c.firms as usize, c.prices as usize);
    let depth = (c.memory as usize).max(1);
    match draws {
        Draws::Calvano(_) => {
            let mut stream = Ran2::new(-1);
            let skip = (seed.max(1) - 1) * (depth * n) as u64;
            for _ in 0..skip {
                stream.next_f64();
            }
            (0..depth)
                .map(|_| {
                    (0..n)
                        .map(|_| (m as f64 * stream.next_f64()) as u8)
                        .collect()
                })
                .collect()
        }
        Draws::Ours(_) => (0..depth)
            .map(|_| (0..n).map(|_| draws.price(m)).collect())
            .collect(),
    }
}

/// One firm's learner.
#[derive(Clone, Debug, PartialEq)]
pub struct Firm {
    /// Q(s, a) at `q[s * m + a]`.
    pub q: Vec<f64>,
    /// max_a Q(s, a).
    pub best: Vec<f64>,
    /// The greedy price in each state.
    pub greedy: Vec<u8>,
    /// The exploration rate (or Boltzmann temperature) now.
    pub eps: f64,
}

impl Firm {
    pub fn new(q: Vec<f64>, m: usize, ties: Ties, draws: &mut Draws, eps: f64) -> Self {
        let states = q.len() / m;
        let mut f = Firm {
            q,
            best: vec![0.0; states],
            greedy: vec![0; states],
            eps,
        };
        for s in 0..states {
            f.rescan(s, m, ties, draws);
        }
        f
    }

    pub fn row(&self, s: usize, m: usize) -> &[f64] {
        &self.q[s * m..(s + 1) * m]
    }

    /// Recomputes state `s`'s maximum and greedy price: among values within
    /// `TIE` of the maximum, the lowest, or one drawn (the code's
    /// `MaxLocBreakTies`, which draws only when there is more than one).
    pub fn rescan(&mut self, s: usize, m: usize, ties: Ties, draws: &mut Draws) {
        let row = &self.q[s * m..(s + 1) * m];
        let top = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let tied: Vec<u8> = (0..m as u8)
            .filter(|&a| (row[a as usize] - top).abs() <= TIE)
            .collect();
        self.best[s] = top;
        self.greedy[s] = match ties {
            Ties::Lowest => tied[0],
            Ties::Random if tied.len() > 1 => tied[(tied.len() as f64 * draws.tie()) as usize],
            Ties::Random => tied[0],
        };
    }

    /// Writes `new` into Q(s, a) and keeps the greedy price as `ties` says:
    /// under `random` exactly the code's rule (a new maximum takes over; the
    /// greedy price's own value falling triggers a rescan); under `lowest`
    /// a rescan whenever the maximum may have moved.
    pub fn set(&mut self, s: usize, a: u8, new: f64, m: usize, ties: Ties, draws: &mut Draws) {
        let k = s * m + usize::from(a);
        self.q[k] = new;
        match ties {
            Ties::Random => {
                if new > self.best[s] {
                    self.best[s] = new;
                    self.greedy[s] = a;
                }
                if new < self.best[s] && self.greedy[s] == a {
                    self.rescan(s, m, ties, draws);
                }
            }
            Ties::Lowest => {
                if self.greedy[s] == a || new >= self.best[s] - TIE {
                    self.rescan(s, m, ties, draws);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space(firms: u32, memory: u32) -> Space {
        Space::of(&CollusionConfig {
            firms,
            memory,
            ..CollusionConfig::default()
        })
    }

    #[test]
    fn states_record_the_last_prices_firm_one_first() {
        let s = space(2, 1);
        assert_eq!(s.states, 225);
        assert_eq!(s.encode(&[vec![3, 9]]), 3 * 15 + 9);
        assert_eq!(s.next(0, &[3, 9]), 54);
        assert_eq!(s.last(54), Some(vec![3, 9]));
        assert_eq!(s.with_own(54, 1, 2), s.encode(&[vec![3, 2]]));
        assert_eq!(s.with_own(54, 0, 14), s.encode(&[vec![14, 9]]));
    }

    #[test]
    fn two_periods_of_memory_shift_the_older_prices_down() {
        let s = space(2, 2);
        let start = s.encode(&[vec![1, 2], vec![3, 4]]);
        let after = s.next(start, &[5, 6]);
        assert_eq!(after, s.encode(&[vec![5, 6], vec![1, 2]]));
        assert_eq!(s.last(after), Some(vec![5, 6]));
        assert_eq!(s.with_own(after, 0, 7), s.encode(&[vec![7, 6], vec![1, 2]]));
        let none = space(2, 0);
        assert_eq!(
            (none.states, none.next(0, &[3, 4]), none.last(0)),
            (1, 0, None)
        );
    }

    #[test]
    fn the_greedy_price_follows_the_tie_rule() {
        let mut draws = Draws::new(&CollusionConfig::default(), 1);
        let q = vec![1.0, 3.0, 3.0, 2.0];
        let low = Firm::new(q.clone(), 4, Ties::Lowest, &mut draws, 1.0);
        assert_eq!((low.greedy[0], low.best[0]), (1, 3.0));
        // Lowering the greedy price's value hands the lead to the tied one.
        let mut f = low.clone();
        f.set(0, 1, 0.5, 4, Ties::Lowest, &mut draws);
        assert_eq!(f.greedy[0], 2);
        // Under `lowest` a new value equal to the maximum takes over if lower.
        let mut g = Firm::new(vec![1.0, 2.0, 3.0, 0.0], 4, Ties::Lowest, &mut draws, 1.0);
        g.set(0, 0, 3.0, 4, Ties::Lowest, &mut draws);
        assert_eq!(g.greedy[0], 0);
        // The code's rule leaves an exact tie alone.
        let mut r = Firm::new(vec![1.0, 2.0, 3.0, 0.0], 4, Ties::Random, &mut draws, 1.0);
        r.set(0, 0, 3.0, 4, Ties::Random, &mut draws);
        assert_eq!(r.greedy[0], 2);
        r.set(0, 3, 4.0, 4, Ties::Random, &mut draws);
        assert_eq!((r.greedy[0], r.best[0]), (3, 4.0));
    }

    #[test]
    fn incremental_greedy_prices_match_a_full_rescan() {
        let mut draws = Draws::new(&CollusionConfig::default(), 3);
        let mut f = Firm::new(vec![0.0; 6 * 5], 5, Ties::Lowest, &mut draws, 1.0);
        let mut r = rng::seeded(9);
        for _ in 0..20_000 {
            let (s, a) = (r.gen_range(0..6u32) as usize, r.gen_range(0..5u32) as u8);
            let v = f64::from(r.gen_range(0..8u32)) * 0.5;
            f.set(s, a, v, 5, Ties::Lowest, &mut draws);
            let mut check = f.clone();
            check.rescan(s, 5, Ties::Lowest, &mut draws);
            assert_eq!((f.greedy[s], f.best[s]), (check.greedy[s], check.best[s]));
        }
    }
}
```

Create `crates/sugarscape-core/src/collusion/analysis.rs` with exactly this content:

```rust
//! What a session learned: the limit cycle its strategies settle into
//! (the code's `ConvergenceResults`), the profit gain Δ, whether the
//! strategies are an equilibrium (by either reading), the responses to
//! deviations (the code's `computeIndividualIR`, and the critics'), den Boer,
//! Meylahn & Schinkel's RP-completeness, and re-pairing firms from different
//! sessions.

use serde::Serialize;

use super::config::{BestResponseTo, EquilibriumCheck};
use super::demand::Game;
use super::learner::Space;

/// Periods an impulse response follows.
pub const HORIZON: usize = 25;

/// Each firm's greedy price in each state: `strategies[i][s]`.
pub type Strategies = Vec<Vec<u8>>;

fn play(strategies: &Strategies, s: usize) -> Vec<u8> {
    strategies.iter().map(|f| f[s]).collect()
}

/// The deterministic path from a state and the cycle it ends in.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Cycle {
    /// The cycle's states, in order: the state after each period's prices.
    pub states: Vec<usize>,
    /// The prices played into each cycle state.
    pub actions: Vec<Vec<u8>>,
    /// Each firm's mean profit over the cycle.
    pub profits: Vec<f64>,
    /// Each firm's mean price over the cycle.
    pub prices: Vec<f64>,
}

impl Cycle {
    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Δ per firm.
    pub fn gains(&self, game: &Game) -> Vec<f64> {
        (0..game.firms)
            .map(|i| game.gain(i, self.profits[i]))
            .collect()
    }

    /// The session's Δ: the firms' mean.
    pub fn gain(&self, game: &Game) -> f64 {
        let g = self.gains(game);
        g.iter().sum::<f64>() / g.len() as f64
    }

    /// A single price profile repeated: every firm at one price.
    pub fn is_point(&self) -> bool {
        self.len() == 1
    }
}

/// Replays `strategies` from `start` without exploration until a state
/// repeats; the repeated stretch is the limit cycle.
pub fn limit_cycle(game: &Game, space: &Space, strategies: &Strategies, start: usize) -> Cycle {
    let mut seen: Vec<usize> = Vec::new();
    let mut acts: Vec<Vec<u8>> = Vec::new();
    let mut s = start;
    loop {
        let a = play(strategies, s);
        s = space.next(s, &a);
        if let Some(k) = seen.iter().position(|&x| x == s) {
            // The state was first reached at k: the cycle is k.. plus this
            // period, which re-enters it.
            let mut states = seen[k + 1..].to_vec();
            states.push(s);
            let mut actions = acts[k + 1..].to_vec();
            actions.push(a);
            let len = actions.len() as f64;
            let profits = (0..game.firms)
                .map(|i| actions.iter().map(|x| game.profit(x, i)).sum::<f64>() / len)
                .collect();
            let prices = (0..game.firms)
                .map(|i| {
                    actions
                        .iter()
                        .map(|x| game.grid[i][usize::from(x[i])])
                        .sum::<f64>()
                        / len
                })
                .collect();
            return Cycle {
                states,
                actions,
                profits,
                prices,
            };
        }
        seen.push(s);
        acts.push(a);
    }
}

/// Firm `i`'s values when everyone follows `strategies`: V(s) = π(σ(s)) +
/// δ V(next), exact on each path's cycle (the code's `computeQCell`).
pub fn policy_values(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    i: usize,
    delta: f64,
) -> Vec<f64> {
    let mut v = vec![f64::NAN; space.states];
    for start in 0..space.states {
        if !v[start].is_nan() {
            continue;
        }
        let mut path = Vec::new();
        let mut s = start;
        while v[s].is_nan() && !path.contains(&s) {
            path.push(s);
            s = space.next(s, &play(strategies, s));
        }
        let mut tail = if v[s].is_nan() {
            // s is on the path: a new cycle from its position.
            let k = path.iter().position(|&x| x == s).unwrap();
            let cycle = &path[k..];
            let rewards: Vec<f64> = cycle
                .iter()
                .map(|&x| game.profit(&play(strategies, x), i))
                .collect();
            let l = cycle.len();
            let dl = delta.powi(l as i32);
            for (j, &x) in cycle.iter().enumerate() {
                let mut sum = 0.0;
                for t in 0..l {
                    sum += delta.powi(t as i32) * rewards[(j + t) % l];
                }
                v[x] = sum / (1.0 - dl);
            }
            path.truncate(k);
            v[s]
        } else {
            v[s]
        };
        for &x in path.iter().rev() {
            tail = game.profit(&play(strategies, x), i) + delta * tail;
            v[x] = tail;
        }
    }
    v
}

/// Firm `i`'s Q(s, a) for each state and price when the rivals follow
/// `strategies` and firm `i` then plays as `values` says.
fn q_against(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    i: usize,
    delta: f64,
    values: &[f64],
) -> Vec<f64> {
    let m = game.prices;
    let mut q = vec![0.0; space.states * m];
    for s in 0..space.states {
        let mut a = play(strategies, s);
        for p in 0..m as u8 {
            a[i] = p;
            q[s * m + usize::from(p)] = game.profit(&a, i) + delta * values[space.next(s, &a)];
        }
    }
    q
}

/// Firm `i`'s optimal values against the rivals' `strategies`, by value
/// iteration to 1e-13.
pub fn optimal_values(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    i: usize,
    delta: f64,
) -> Vec<f64> {
    let m = game.prices;
    let mut v = vec![0.0; space.states];
    loop {
        let q = q_against(game, space, strategies, i, delta, &v);
        let mut moved: f64 = 0.0;
        for s in 0..space.states {
            let best = q[s * m..(s + 1) * m]
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            moved = moved.max((best - v[s]).abs());
            v[s] = best;
        }
        if moved < 1e-13 || delta == 0.0 {
            return v;
        }
    }
}

/// Whether each firm's strategy is a best response in each state, by
/// `check`: `best[i][s]`.
pub fn best_responses(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    delta: f64,
    check: EquilibriumCheck,
) -> Vec<Vec<bool>> {
    let m = game.prices;
    (0..game.firms)
        .map(|i| {
            let v = match check {
                EquilibriumCheck::OneShot => policy_values(game, space, strategies, i, delta),
                EquilibriumCheck::BestResponse => optimal_values(game, space, strategies, i, delta),
            };
            let q = q_against(game, space, strategies, i, delta, &v);
            (0..space.states)
                .map(|s| {
                    let row = &q[s * m..(s + 1) * m];
                    let top = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    top - row[usize::from(strategies[i][s])] <= 1e-10 * top.abs().max(1.0)
                })
                .collect()
        })
        .collect()
}

/// Equilibrium flags: on the limit path, off it, and everywhere — each the
/// share of states where every firm best responds, and whether that share is 1.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Equilibrium {
    pub on_path: bool,
    pub off_path_share: f64,
    pub all_share: f64,
}

pub fn equilibrium(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    delta: f64,
    check: EquilibriumCheck,
) -> Equilibrium {
    let best = best_responses(game, space, strategies, delta, check);
    let all = |s: usize| best.iter().all(|f| f[s]);
    let on_path = cycle.states.iter().all(|&s| all(s));
    let off: Vec<usize> = (0..space.states)
        .filter(|s| !cycle.states.contains(s))
        .collect();
    let share = |set: &[usize]| {
        if set.is_empty() {
            1.0
        } else {
            set.iter().filter(|&&s| all(s)).count() as f64 / set.len() as f64
        }
    };
    let every: Vec<usize> = (0..space.states).collect();
    Equilibrium {
        on_path,
        off_path_share: share(&off),
        all_share: share(&every),
    }
}

/// One deviation and what followed.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Response {
    /// The deviating firm and the cycle position it deviated from.
    pub firm: usize,
    pub position: usize,
    /// Each firm's price index before the deviation (the cycle's prices into
    /// the deviation state) and at the deviation.
    pub before: Vec<u8>,
    pub deviation: u8,
    /// Prices (grid indices) in the deviation period and the `HORIZON` − 1
    /// after: `path[t][j]`.
    pub path: Vec<Vec<u8>>,
    /// Periods until the state is back on the pre-deviation cycle (the
    /// code's `PunishmentStrategy`), if it returns within the horizon.
    pub returned: Option<usize>,
    /// Periods until play enters a cycle — the pre-deviation one or a new
    /// one (the code's `ShockLength`, Table A5's punishment length).
    pub settled: usize,
}

impl Response {
    /// The punishment-like response the survey fixed: in the period after
    /// the deviation every other firm's price is at least one grid step
    /// below what it charged in the deviation period, and the state is back
    /// on the cycle within `HORIZON` periods.
    pub fn punishment_like(&self) -> bool {
        let (dev, next) = (&self.path[0], &self.path[1]);
        (0..dev.len())
            .filter(|&j| j != self.firm)
            .all(|j| next[j] < dev[j])
            && self.returned.is_some()
    }

    /// Firm j's relative price change from the deviation period to the next
    /// (Epivent & Lambin's measure).
    pub fn change(&self, game: &Game, j: usize) -> f64 {
        let (a, b) = (
            game.grid[j][usize::from(self.path[0][j])],
            game.grid[j][usize::from(self.path[1][j])],
        );
        (b - a) / a
    }
}

/// A one-period deviation by `firm` to `price` from cycle position `k`: the
/// deviation period's other prices are the strategies' at the cycle state,
/// then everyone follows the strategies (the code's `computeIndividualIR`).
pub fn deviate(
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    k: usize,
    firm: usize,
    price: u8,
) -> Response {
    let start = cycle.states[k];
    let before = play(strategies, start);
    let mut a = before.clone();
    a[firm] = price;
    let mut path = Vec::with_capacity(HORIZON);
    let mut seen: Vec<usize> = Vec::new();
    let mut s = start;
    let mut returned = None;
    let mut settled = None;
    // As the code's computeIndividualIR: the first period the state is on
    // the pre-deviation cycle, or else where a new cycle begins.
    for t in 1..=HORIZON.max(space.states + 1) {
        s = space.next(s, &a);
        if t <= HORIZON {
            path.push(a.clone());
        }
        if settled.is_none() {
            if cycle.states.contains(&s) {
                settled = Some(t);
            } else if let Some(first) = seen.iter().position(|&x| x == s) {
                settled = Some(first + 1);
            }
        }
        if returned.is_none() && t <= HORIZON && cycle.states.contains(&s) {
            returned = Some(t);
        }
        seen.push(s);
        if settled.is_some() && t >= HORIZON {
            break;
        }
        a = play(strategies, s);
    }
    Response {
        firm,
        position: k,
        before,
        deviation: price,
        path,
        returned,
        settled: settled.expect("a deterministic path enters a cycle"),
    }
}

/// The paper's deviation: one period at the static best response — to the
/// rivals' prices at the cycle state (`path`), or at the state the code
/// passes, the one numbered by the cycle position (`code`).
pub fn best_response_deviation(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    k: usize,
    firm: usize,
    to: BestResponseTo,
) -> Response {
    let against = match to {
        BestResponseTo::Path => cycle.states[k],
        BestResponseTo::Code => k.min(space.states - 1),
    };
    let br = game.static_best_response(&play(strategies, against), firm);
    deviate(space, strategies, cycle, k, firm, br)
}

/// Every one-period deviation to every other price, by every firm, from
/// every cycle position (CCDP-A; Epivent & Lambin's Table 1).
pub fn every_deviation(space: &Space, strategies: &Strategies, cycle: &Cycle) -> Vec<Response> {
    let mut out = Vec::new();
    for k in 0..cycle.len() {
        let on = play(strategies, cycle.states[k]);
        for (firm, &own) in on.iter().enumerate() {
            for p in 0..space.prices as u8 {
                if p != own {
                    out.push(deviate(space, strategies, cycle, k, firm, p));
                }
            }
        }
    }
    out
}

/// den Boer, Meylahn & Schinkel's RP-completeness: every one-period
/// deviation, by either firm to any other price, from every cycle state,
/// draws a punishment-like response.
pub fn rp_complete(space: &Space, strategies: &Strategies, cycle: &Cycle) -> bool {
    every_deviation(space, strategies, cycle)
        .iter()
        .all(Response::punishment_like)
}

/// Epivent & Lambin's invitation from a point cycle: `firm` one step up for
/// `hold` + 1 periods, every other firm forced to match from the second
/// period on; returns the deviator's price index when it regains control and
/// its price before, or `None` if the cycle is not a point or it is at the
/// top of the grid.
pub fn invitation(
    game: &Game,
    space: &Space,
    strategies: &Strategies,
    cycle: &Cycle,
    firm: usize,
    hold: u32,
) -> Option<(u8, u8)> {
    if !cycle.is_point() {
        return None;
    }
    let mut s = cycle.states[0];
    let before = play(strategies, s)[firm];
    if usize::from(before) + 1 >= game.prices {
        return None;
    }
    let up = before + 1;
    let raised = game.grid[firm][usize::from(up)];
    for t in 0..=hold {
        let mut a = play(strategies, s);
        a[firm] = up;
        if t >= 1 {
            for (j, x) in a.iter_mut().enumerate() {
                if j != firm {
                    *x = game.nearest(j, raised);
                }
            }
        }
        s = space.next(s, &a);
    }
    Some((play(strategies, s)[firm], before))
}

/// Lambin's (2024) Theorem 1 for two symmetric firms: rank prices by π̃,
/// the profit against a uniformly random rival; after long exploration the
/// firms try them in that order, keeping the best symmetric profit found,
/// until it beats (1 − δ)π̃(next) + δπ̃(first). Returns the price index they
/// settle on (his eq. 6).
pub fn lambin_point(game: &Game, delta: f64) -> u8 {
    let m = game.prices;
    let tilde: Vec<f64> = (0..m as u8)
        .map(|a| (0..m as u8).map(|q| game.profit(&[a, q], 0)).sum::<f64>() / m as f64)
        .collect();
    let mut order: Vec<u8> = (0..m as u8).collect();
    order.sort_by(|&a, &b| tilde[usize::from(b)].total_cmp(&tilde[usize::from(a)]));
    let top = tilde[usize::from(order[0])];
    let mut best = (order[0], game.profit(&[order[0], order[0]], 0));
    for j in 0..m - 1 {
        let tried = order[j];
        let p = game.profit(&[tried, tried], 0);
        if p > best.1 {
            best = (tried, p);
        }
        let next = usize::from(order[j + 1]);
        if best.1 > (1.0 - delta) * tilde[next] + delta * top {
            return best.0;
        }
    }
    best.0
}

/// Pairs firm 0's strategy from one session with firm 1's from another
/// (Eschenbaum, Mellgren & Zahn; two firms only) and replays from `start`.
pub fn repair(
    game: &Game,
    space: &Space,
    first: &Strategies,
    second: &Strategies,
    start: usize,
) -> Cycle {
    let mixed: Strategies = vec![first[0].clone(), second[1].clone()];
    limit_cycle(game, space, &mixed, start)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collusion::config::CollusionConfig;

    fn setup() -> (Game, Space) {
        let c = CollusionConfig::default();
        (Game::new(&c), Space::of(&c))
    }

    /// Both firms at `high` while both were at `high` last period; otherwise
    /// at `low` (grim trigger on the grid).
    fn grim(space: &Space, high: u8, low: u8) -> Strategies {
        let both_high = space.encode(&[vec![high, high]]);
        let one = (0..space.states)
            .map(|s| if s == both_high { high } else { low })
            .collect::<Vec<u8>>();
        vec![one.clone(), one]
    }

    #[test]
    fn a_constant_strategy_cycles_at_one_point() {
        let (game, space) = setup();
        let strategies = vec![vec![8u8; 225], vec![8u8; 225]];
        let c = limit_cycle(&game, &space, &strategies, 0);
        assert!(c.is_point());
        assert_eq!(c.states, vec![space.encode(&[vec![8, 8]])]);
        assert!((c.gain(&game) - 0.794).abs() < 0.001);
    }

    #[test]
    fn an_alternating_pair_cycles_in_two() {
        let (game, space) = setup();
        // Each firm plays the other's last price, starting from (3, 9).
        let s: Vec<u8> = (0..225).map(|x| (x % 15) as u8).collect();
        let t: Vec<u8> = (0..225).map(|x| (x / 15) as u8).collect();
        let c = limit_cycle(&game, &space, &vec![s, t], space.encode(&[vec![3, 9]]));
        assert_eq!(c.len(), 2);
        assert_eq!(c.actions, vec![vec![3, 9], vec![9, 3]]);
        let mean = (game.grid[0][3] + game.grid[0][9]) / 2.0;
        assert!((c.prices[0] - mean).abs() < 1e-12);
    }

    #[test]
    fn grim_trigger_at_monopoly_is_an_equilibrium_and_rp_complete() {
        let (game, space) = setup();
        let g = grim(&space, 12, 1);
        let c = limit_cycle(&game, &space, &g, space.encode(&[vec![12, 12]]));
        assert!(c.is_point());
        for check in [EquilibriumCheck::BestResponse, EquilibriumCheck::OneShot] {
            let e = equilibrium(&game, &space, &g, &c, 0.95, check);
            assert!(e.on_path, "{check:?}");
        }
        // Too impatient to sustain it.
        let e = equilibrium(&game, &space, &g, &c, 0.1, EquilibriumCheck::BestResponse);
        assert!(!e.on_path);
        // A deviation is punished forever, so it never returns: not
        // punishment-like by the survey's rule (no return within 25 periods).
        let r = deviate(&space, &g, &c, 0, 0, 5);
        assert_eq!(r.path[1], vec![1, 1]);
        assert_eq!(r.returned, None);
        // A new cycle at (1, 1), entered in period 2.
        assert_eq!(r.settled, 2);
        assert!(!rp_complete(&space, &g, &c));
    }

    #[test]
    fn a_one_period_punishment_is_punishment_like_and_rp_complete() {
        let (game, space) = setup();
        // At (12, 12) both stay; after anything else both play 1 for a period
        // and then return to 12: from (1, 1) back to 12.
        let cartel = space.encode(&[vec![12, 12]]);
        let punish = space.encode(&[vec![1, 1]]);
        let one: Vec<u8> = (0..225)
            .map(|s| if s == cartel || s == punish { 12 } else { 1 })
            .collect();
        let st = vec![one.clone(), one];
        let c = limit_cycle(&game, &space, &st, cartel);
        assert!(c.is_point());
        let r = deviate(&space, &st, &c, 0, 0, 3);
        assert_eq!(r.path[0], vec![3, 12]);
        assert_eq!(r.path[1], vec![1, 1]);
        assert_eq!((r.returned, r.settled), (Some(3), 3));
        assert!(r.punishment_like());
        assert!(r.change(&game, 1) < 0.0);
        assert!(rp_complete(&space, &st, &c));
    }

    #[test]
    fn the_codes_best_response_answers_state_one() {
        let (game, space) = setup();
        // Rivals play 14 at state 0, 8 elsewhere.
        let mut one = vec![8u8; 225];
        one[0] = 14;
        let st = vec![one.clone(), one];
        let c = limit_cycle(&game, &space, &st, space.encode(&[vec![8, 8]]));
        let path = best_response_deviation(&game, &space, &st, &c, 0, 0, BestResponseTo::Path);
        let code = best_response_deviation(&game, &space, &st, &c, 0, 0, BestResponseTo::Code);
        assert_eq!(path.deviation, game.static_best_response(&[8, 8], 0));
        assert_eq!(code.deviation, game.static_best_response(&[14, 14], 0));
        assert_ne!(path.deviation, code.deviation);
    }

    #[test]
    fn an_invitation_ends_where_the_strategy_sends_the_deviator() {
        let (game, space) = setup();
        let st = vec![vec![8u8; 225], vec![8u8; 225]];
        let c = limit_cycle(&game, &space, &st, 0);
        assert_eq!(invitation(&game, &space, &st, &c, 0, 5), Some((8, 8)));
        let top = vec![vec![14u8; 225], vec![14u8; 225]];
        let ct = limit_cycle(&game, &space, &top, 0);
        assert_eq!(invitation(&game, &space, &top, &ct, 0, 5), None);
    }

    #[test]
    fn lambin_s_theorem_one_on_calvano_s_grid() {
        let (game, _) = setup();
        // The reader's computation (reading notes §5): I = 1.73770 at δ = 0.95,
        // 1.69895 at δ = 0; the first price tried is π̃'s best, 1.58271.
        assert_eq!(lambin_point(&game, 0.95), 8);
        assert_eq!(lambin_point(&game, 0.0), 7);
        assert!((game.grid[0][8] - 1.7377021).abs() < 1e-6);
    }

    #[test]
    fn policy_values_sum_the_discounted_cycle() {
        let (game, space) = setup();
        let st = vec![vec![8u8; 225], vec![8u8; 225]];
        let v = policy_values(&game, &space, &st, 0, 0.9);
        let pi = game.profit(&[8, 8], 0);
        assert!(v.iter().all(|x| (x - pi / 0.1).abs() < 1e-9));
        let opt = optimal_values(&game, &space, &st, 0, 0.9);
        let br = game.static_best_response(&[8, 8], 0);
        assert!(opt
            .iter()
            .all(|x| (x - game.profit(&[br, 8], 0) / 0.1).abs() < 1e-9));
        let repaired = repair(&game, &space, &st, &st, 0);
        assert!(repaired.is_point());
    }
}
```

Create `crates/sugarscape-core/src/collusion/stats.rs` with exactly this content:

```rust
//! Algorithmic Collusion's statistics: each charted period's prices, profit
//! gain and learning, and — once the session has finished — what it learned.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 17] = [
    "price_1",
    "price_2",
    "profit_gain",
    "greedy_price",
    "epsilon",
    "explored",
    "greedy_changes",
    "stable",
    "converged",
    "cycle_length",
    "cycle_gain",
    "window_gain",
    "discounted_gain",
    "equilibrium_on_path",
    "punishment_like",
    "rp_complete",
    "periods",
];

/// One charted period. The session's results are NaN until it finishes.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CollusionSnapshot {
    pub tick: u64,
    /// Firms 1 and 2's prices this period.
    pub price_1: f64,
    pub price_2: f64,
    /// Δ of this period's profits, the firms' mean.
    pub profit_gain: f64,
    /// The firms' mean greedy price at this period's state (Lambin's measure).
    pub greedy_price: f64,
    /// Firm 1's exploration rate (or temperature).
    pub epsilon: f64,
    /// The share of firms that explored this period.
    pub explored: f64,
    /// Greedy prices that changed this period.
    pub greedy_changes: u32,
    /// Periods the strategies have stayed the same.
    pub stable: u32,
    /// 1 converged, 0 stopped at the cap, NaN still learning.
    pub converged: f64,
    pub cycle_length: f64,
    /// Δ over the limit cycle (CCDP's).
    pub cycle_gain: f64,
    /// Δ of the realized profits over the last `window` periods.
    pub window_gain: f64,
    /// den Boer, Meylahn & Schinkel's Δ̃ over the first T_δ periods.
    pub discounted_gain: f64,
    /// 1 if every firm best responds on the cycle.
    pub equilibrium_on_path: f64,
    /// The share of deviations (as `impulse` says) answered by a
    /// punishment-like response.
    pub punishment_like: f64,
    /// 1 if every one-period deviation draws a punishment-like response
    /// (den Boer, Meylahn & Schinkel).
    pub rp_complete: f64,
    /// Periods to convergence (excluding the window), as the code reports.
    pub periods: f64,
}

impl Default for CollusionSnapshot {
    fn default() -> Self {
        CollusionSnapshot {
            tick: 0,
            price_1: f64::NAN,
            price_2: f64::NAN,
            profit_gain: f64::NAN,
            greedy_price: f64::NAN,
            epsilon: f64::NAN,
            explored: f64::NAN,
            greedy_changes: 0,
            stable: 0,
            converged: f64::NAN,
            cycle_length: f64::NAN,
            cycle_gain: f64::NAN,
            window_gain: f64::NAN,
            discounted_gain: f64::NAN,
            equilibrium_on_path: f64::NAN,
            punishment_like: f64::NAN,
            rp_complete: f64::NAN,
            periods: f64::NAN,
        }
    }
}

impl Series for CollusionSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "price_1" => self.price_1,
            "price_2" => self.price_2,
            "profit_gain" => self.profit_gain,
            "greedy_price" => self.greedy_price,
            "epsilon" => self.epsilon,
            "explored" => self.explored,
            "greedy_changes" => f64::from(self.greedy_changes),
            "stable" => f64::from(self.stable),
            "converged" => self.converged,
            "cycle_length" => self.cycle_length,
            "cycle_gain" => self.cycle_gain,
            "window_gain" => self.window_gain,
            "discounted_gain" => self.discounted_gain,
            "equilibrium_on_path" => self.equilibrium_on_path,
            "punishment_like" => self.punishment_like,
            "rp_complete" => self.rp_complete,
            "periods" => self.periods,
            _ => return None,
        })
    }
}
```

Create `crates/sugarscape-core/src/collusion/view.rs` with exactly this content:

```rust
//! The frame: each firm's strategy as a map (its own last price across, the
//! next firm's down, the price it would charge as the color), the last
//! periods' prices with the Nash and monopoly prices marked, and — once the
//! session has finished — the response to a deviation.

use crate::render::{lerp, Rgb};

/// Periods the price panel shows.
pub const SHOWN: usize = 240;
/// The price and response panels' height.
pub const PANEL_H: usize = 121;
/// Cells between panels.
pub const GAP: usize = 8;
/// Pixels per period in the response panel.
pub const STEP: usize = 8;

pub const BACK: Rgb = [0x16, 0x18, 0x1e];
pub const COOL: Rgb = [0x3a, 0x6e, 0xd8];
pub const WARM: Rgb = [0xf0, 0x7a, 0x3a];
pub const NASH: Rgb = [0x5a, 0x9a, 0x6a];
pub const MONOPOLY: Rgb = [0xb0, 0x4a, 0x4a];
pub const MARK: Rgb = [0xf4, 0xf1, 0xea];
pub const AXIS: Rgb = [0x44, 0x48, 0x52];
/// Firms' colors in the price panels.
pub const FIRMS: [Rgb; 4] = [
    [0x8f, 0xc8, 0xff],
    [0xff, 0xc8, 0x6e],
    [0xb8, 0x8a, 0xe8],
    [0x7c, 0xe0, 0x8a],
];

/// Pixels per price in a strategy map: the map stays near 120 wide.
pub fn cell(prices: usize) -> usize {
    (120 / prices).max(2)
}

/// A strategy map's side in pixels.
pub fn side(prices: usize) -> usize {
    cell(prices) * prices
}

/// Price index `a` of `m` as a color: low prices cool, high prices warm.
pub fn price_color(a: u8, m: usize) -> Rgb {
    lerp(COOL, WARM, f64::from(a) / (m - 1).max(1) as f64)
}

/// How often a state was visited as a color (log scale against the most).
pub fn visit_color(visits: u32, most: u32) -> Rgb {
    if visits == 0 || most == 0 {
        return BACK;
    }
    let t = (f64::from(visits).ln_1p() / f64::from(most).ln_1p()).clamp(0.0, 1.0);
    lerp(AXIS, MARK, t)
}

/// The panel row of `price` between `lo` (bottom) and `hi` (top).
pub fn row(price: f64, lo: f64, hi: f64) -> usize {
    let t = if hi > lo {
        ((price - lo) / (hi - lo)).clamp(0.0, 1.0)
    } else {
        0.5
    };
    ((1.0 - t) * (PANEL_H - 1) as f64).round() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_rows_fit_their_panels() {
        assert_eq!((cell(15), side(15)), (8, 120));
        assert_eq!((cell(100), side(100)), (2, 200));
        assert_eq!(row(2.0, 1.0, 2.0), 0);
        assert_eq!(row(1.0, 1.0, 2.0), PANEL_H - 1);
        assert_eq!(price_color(0, 15), COOL);
        assert_eq!(price_color(14, 15), WARM);
        assert_eq!(visit_color(0, 10), BACK);
        assert_eq!(visit_color(10, 10), MARK);
    }
}
```

Create `crates/sugarscape-core/src/collusion/world.rs` with exactly this content:

```rust
//! One session: n firms learning to price together until their strategies
//! settle (CCDP §II), then what they learned.

use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

use serde::Serialize;

use super::analysis::{
    best_response_deviation, deviate, equilibrium, every_deviation, invitation, limit_cycle,
    rp_complete, Cycle, Equilibrium, Response, Strategies, HORIZON,
};
use super::config::{CollusionConfig, Exploration, Impulse, QInit, Update};
use super::demand::Game;
use super::learner::{initial_prices, Draws, Firm, Space};
use super::stats::CollusionSnapshot;
use super::view::{
    cell, price_color, row, side, visit_color, AXIS, BACK, FIRMS, GAP, MARK, MONOPOLY, NASH,
    PANEL_H, SHOWN, STEP,
};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::opinions::Canvas;
use crate::portable::exp_neg;
use crate::stats::{Series, Stats};

/// What a finished session learned.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    /// Whether the strategies settled (or the cap stopped the session).
    pub converged: bool,
    /// Periods to convergence, excluding the window (the code's measure).
    pub periods: u64,
    #[serde(skip)]
    pub strategies: Strategies,
    pub cycle: Cycle,
    pub gains: Vec<f64>,
    pub gain: f64,
    pub window_gain: f64,
    pub discounted_gain: f64,
    pub equilibrium: Equilibrium,
    /// The deviations `impulse` asks for and their responses.
    #[serde(skip)]
    pub responses: Vec<Response>,
    pub punishment_like: f64,
    pub rp_complete: bool,
    /// Off-path states whose greedy price is still the starting one.
    pub stale_greedy: f64,
    /// Periods from ε falling below 0.01 to the last greedy change.
    pub fumbling: Option<u64>,
    /// The share of Q-values ever updated.
    pub touched: f64,
}

#[derive(Clone)]
pub struct CollusionWorld {
    pub config: CollusionConfig,
    /// Completed ticks (`periods_per_tick` periods each, the last perhaps
    /// cut short when the session finished).
    pub tick: u64,
    /// Completed periods.
    period: u64,
    game: Arc<Game>,
    space: Space,
    firms: Vec<Firm>,
    draws: Draws,
    /// The state the next period's prices are chosen in.
    state: usize,
    decay: f64,
    stable: u32,
    /// The state at convergence (the code's `stateFix`).
    fixed: Option<usize>,
    converged: bool,
    initial: Strategies,
    touched: Vec<Vec<bool>>,
    touched_count: u64,
    visits: Vec<u32>,
    streak_profit: Vec<f64>,
    streak_len: u64,
    discounted: Vec<f64>,
    weight: f64,
    horizon: u32,
    last_change: u64,
    quiet_from: Option<u64>,
    recent: VecDeque<Vec<u8>>,
    explored: u32,
    changes: u32,
    /// This tick's sums: the profit gain, the share of firms exploring, and
    /// greedy changes, over `tick_periods` periods.
    tick_gain: f64,
    tick_explored: f64,
    tick_changes: u32,
    tick_periods: u32,
    outcome: Option<Outcome>,
    pub stats: Stats<CollusionSnapshot>,
}

/// One state of the Inspect panel.
#[derive(Clone, Debug, Serialize)]
pub struct StateView {
    pub state: usize,
    /// The prices the state records, latest first: `prices[d][i]`.
    pub prices: Vec<Vec<f64>>,
    /// Each firm's Q-values and greedy price there.
    pub q: Vec<Vec<f64>>,
    pub greedy: Vec<f64>,
    pub visits: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CollusionInspection {
    pub x: u32,
    pub y: u32,
    pub panel: Option<&'static str>,
    pub firm: Option<usize>,
    pub state: Option<StateView>,
    pub tick: u64,
    pub period: u64,
    pub nash: Vec<f64>,
    pub monopoly: Vec<f64>,
    pub outcome: Option<Outcome>,
    /// Always null: there are no agents to follow, only firms.
    pub agent: Option<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollusionMode {
    Price,
    Visits,
}

impl std::str::FromStr for CollusionMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "price" | "" => Ok(CollusionMode::Price),
            "visits" => Ok(CollusionMode::Visits),
            other => Err(format!(
                "unknown color mode {other:?} (expected price or visits)"
            )),
        }
    }
}

impl CollusionWorld {
    pub fn new(config: CollusionConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let game = Game::new(&config);
        let space = Space::of(&config);
        let (n, m) = (space.firms, space.prices);
        let mut draws = Draws::new(&config, seed);
        // Every firm's Q first, then the greedy prices (the code's order).
        let tables: Vec<Vec<f64>> = (0..n)
            .map(|i| match config.q_init {
                QInit::Calvano => {
                    let row: Vec<f64> = (0..m)
                        .map(|a| {
                            let mut sum = 0.0;
                            let mut count = 0u32;
                            for profile in 0..m.pow(n as u32) {
                                let own = profile / m.pow((n - 1 - i) as u32) % m;
                                if own == a {
                                    sum += game.payoff[profile * n + i];
                                    count += 1;
                                }
                            }
                            sum / (f64::from(count) * (1.0 - config.delta))
                        })
                        .collect();
                    row.repeat(space.states)
                }
                QInit::Zero => vec![0.0; space.states * m],
                QInit::Random => (0..space.states * m)
                    .map(|_| config.q_low + (config.q_high - config.q_low) * draws.tie())
                    .collect(),
            })
            .collect();
        let eps = match config.exploration {
            Exploration::Decaying | Exploration::TwoPhase => 1.0,
            Exploration::Constant => config.epsilon,
            Exploration::Boltzmann => config.temperature,
        };
        let firms: Vec<Firm> = tables
            .into_iter()
            .map(|q| Firm::new(q, m, config.ties, &mut draws, eps))
            .collect();
        let start = initial_prices(&config, seed, &mut draws);
        let state = space.encode(&start);
        let initial = firms.iter().map(|f| f.greedy.clone()).collect();
        let horizon = game.horizon(config.delta);
        let mut world = CollusionWorld {
            decay: exp_neg(-config.beta),
            game: Arc::new(game),
            space,
            initial,
            touched: vec![vec![false; space.states * m]; n],
            touched_count: 0,
            visits: vec![0; space.states],
            streak_profit: vec![0.0; n],
            streak_len: 0,
            discounted: vec![0.0; n],
            weight: 1.0,
            horizon,
            last_change: 0,
            quiet_from: None,
            recent: VecDeque::new(),
            explored: 0,
            changes: 0,
            tick_gain: 0.0,
            tick_explored: 0.0,
            tick_changes: 0,
            tick_periods: 0,
            firms,
            draws,
            state,
            stable: 0,
            fixed: None,
            converged: false,
            outcome: None,
            config,
            tick: 0,
            period: 0,
            stats: Stats::default(),
        };
        world.record();
        Ok(world)
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn space(&self) -> &Space {
        &self.space
    }

    pub fn firms(&self) -> &[Firm] {
        &self.firms
    }

    /// Completed periods.
    pub fn period(&self) -> u64 {
        self.period
    }

    pub fn state(&self) -> usize {
        self.state
    }

    /// The state the strategies are read in: where the session settled, or
    /// the current one.
    pub fn current_state(&self) -> usize {
        self.fixed.unwrap_or(self.state)
    }

    pub fn is_finished(&self) -> bool {
        self.fixed.is_some()
    }

    pub fn outcome(&self) -> Option<&Outcome> {
        self.outcome.as_ref()
    }

    /// The greedy strategies now.
    pub fn strategies(&self) -> Strategies {
        self.firms.iter().map(|f| f.greedy.clone()).collect()
    }

    /// Each firm's price this period.
    fn choose(&mut self, u: &[[f64; 2]]) -> Vec<u8> {
        let m = self.space.prices;
        let s = self.state;
        let t = self.period + 1;
        let mut out = Vec::with_capacity(self.firms.len());
        self.explored = 0;
        for (i, f) in self.firms.iter_mut().enumerate() {
            let greedy = f.greedy[s];
            let a = match self.config.exploration {
                Exploration::Decaying | Exploration::Constant | Exploration::TwoPhase => {
                    let eps = match self.config.exploration {
                        Exploration::TwoPhase => {
                            if t <= u64::from(self.config.explore_for) {
                                1.0
                            } else {
                                0.0
                            }
                        }
                        _ => f.eps,
                    };
                    let a = if u[i][0] <= eps {
                        (m as f64 * u[i][1]) as u8
                    } else {
                        greedy
                    };
                    if self.config.exploration == Exploration::Decaying {
                        f.eps *= self.decay;
                    }
                    a
                }
                Exploration::Boltzmann => {
                    let row = f.row(s, m);
                    let top = f.best[s];
                    let probs: Vec<f64> = row
                        .iter()
                        .map(|q| exp_neg((q - top).min(0.0) / f.eps))
                        .collect();
                    let target = u[i][0] * probs.iter().sum::<f64>();
                    let mut cum = 0.0;
                    let mut pick = (m - 1) as u8;
                    for (a, p) in probs.iter().enumerate() {
                        cum += p;
                        if target <= cum {
                            pick = a as u8;
                            break;
                        }
                    }
                    f.eps *= 1.0 - self.config.cooling;
                    pick
                }
            };
            if a != greedy {
                self.explored += 1;
            }
            out.push(a);
        }
        out
    }

    /// One tick: up to `periods_per_tick` periods, fewer if the session
    /// finishes; then a statistics snapshot.
    pub fn step(&mut self) {
        if self.is_finished() {
            return;
        }
        for _ in 0..self.config.periods_per_tick {
            self.step_period();
            if self.is_finished() {
                break;
            }
        }
        self.tick += 1;
        self.record();
    }

    /// One period: prices, profits, the firms' updates, convergence.
    pub fn step_period(&mut self) {
        if self.is_finished() {
            return;
        }
        let n = self.space.firms;
        let m = self.space.prices;
        let mut u = vec![[0.0; 2]; n];
        self.draws.explore(&mut u);
        let actions = self.choose(&u);
        let s = self.state;
        let next = self.space.next(s, &actions);
        let profile = self.game.profile(&actions);
        let before: Vec<u8> = self.firms.iter().map(|f| f.greedy[s]).collect();
        let (alpha, delta, ties) = (self.config.alpha, self.config.delta, self.config.ties);
        for i in 0..n {
            match self.config.update {
                Update::Asynchronous => {
                    let a = actions[i];
                    let f = &mut self.firms[i];
                    let old = f.q[s * m + usize::from(a)];
                    let target = self.game.payoff[profile * n + i] + delta * f.best[next];
                    f.set(s, a, old + alpha * (target - old), m, ties, &mut self.draws);
                    let k = s * m + usize::from(a);
                    if !self.touched[i][k] {
                        self.touched[i][k] = true;
                        self.touched_count += 1;
                    }
                }
                Update::Synchronous => {
                    let mut a = actions.clone();
                    let targets: Vec<f64> = (0..m as u8)
                        .map(|p| {
                            a[i] = p;
                            let r = self.game.profit(&a, i);
                            r + delta * self.firms[i].best[self.space.with_own(next, i, p)]
                        })
                        .collect();
                    let f = &mut self.firms[i];
                    for (p, target) in targets.into_iter().enumerate() {
                        let old = f.q[s * m + p];
                        f.set(
                            s,
                            p as u8,
                            old + alpha * (target - old),
                            m,
                            ties,
                            &mut self.draws,
                        );
                        if !self.touched[i][s * m + p] {
                            self.touched[i][s * m + p] = true;
                            self.touched_count += 1;
                        }
                    }
                }
            }
        }
        let t = self.period + 1;
        self.changes = (0..n)
            .filter(|&i| self.firms[i].greedy[s] != before[i])
            .count() as u32;
        let profits: Vec<f64> = (0..n).map(|i| self.game.payoff[profile * n + i]).collect();
        if self.changes == 0 {
            self.stable += 1;
            self.streak_len += 1;
            for (sum, p) in self.streak_profit.iter_mut().zip(&profits) {
                *sum += p;
            }
        } else {
            self.stable = 1;
            self.last_change = t;
            self.streak_len = 1;
            self.streak_profit.clone_from(&profits);
        }
        if t <= u64::from(self.horizon) {
            for (d, p) in self.discounted.iter_mut().zip(&profits) {
                *d += self.weight * p;
            }
            self.weight *= delta;
        }
        if self.quiet_from.is_none()
            && self.config.exploration == Exploration::Decaying
            && self.firms[0].eps < 0.01
        {
            self.quiet_from = Some(t);
        }
        self.visits[s] = self.visits[s].saturating_add(1);
        if self.recent.len() == SHOWN {
            self.recent.pop_front();
        }
        self.tick_gain += profits
            .iter()
            .enumerate()
            .map(|(i, &p)| self.game.gain(i, p))
            .sum::<f64>()
            / n as f64;
        self.tick_explored += f64::from(self.explored) / n as f64;
        self.tick_changes += self.changes;
        self.tick_periods += 1;
        self.recent.push_back(actions);
        self.period = t;
        if t > u64::from(self.config.cap) {
            // The code keeps the strategy from before this period's update.
            for (f, &b) in self.firms.iter_mut().zip(&before) {
                f.greedy[s] = b;
            }
            self.converged = false;
            self.fixed = Some(s);
        } else if self.stable == self.config.window {
            self.converged = true;
            self.fixed = Some(s);
        } else {
            self.state = next;
        }
        if self.fixed.is_some() {
            self.outcome = Some(self.analyze());
        }
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// The deviations `impulse` asks for, from every cycle position.
    fn responses(&self, strategies: &Strategies, cycle: &Cycle) -> Vec<Response> {
        let (game, space) = (&*self.game, &self.space);
        let mut out = Vec::new();
        match self.config.impulse {
            Impulse::BestResponseDown => {
                for firm in 0..space.firms {
                    for k in 0..cycle.len() {
                        out.push(best_response_deviation(
                            game,
                            space,
                            strategies,
                            cycle,
                            k,
                            firm,
                            self.config.best_response_to,
                        ));
                    }
                }
            }
            Impulse::EveryPrice => out = every_deviation(space, strategies, cycle),
            Impulse::Up | Impulse::Invitation => {
                for firm in 0..space.firms {
                    for k in 0..cycle.len() {
                        let own = strategies[firm][cycle.states[k]];
                        if usize::from(own) + 1 < space.prices {
                            out.push(deviate(space, strategies, cycle, k, firm, own + 1));
                        }
                    }
                }
            }
        }
        out
    }

    fn analyze(&self) -> Outcome {
        let fixed = self.fixed.expect("analyzed when finished");
        let (game, space) = (&*self.game, &self.space);
        let strategies = self.strategies();
        let cycle = limit_cycle(game, space, &strategies, fixed);
        let gains = cycle.gains(game);
        let gain = gains.iter().sum::<f64>() / gains.len() as f64;
        let mean_gain = |profits: &[f64], scale: f64| {
            (0..space.firms)
                .map(|i| game.gain(i, profits[i] * scale))
                .sum::<f64>()
                / space.firms as f64
        };
        let window_gain = mean_gain(&self.streak_profit, 1.0 / self.streak_len.max(1) as f64);
        let discounted_gain = self.discounted_gain();
        let eq = equilibrium(
            game,
            space,
            &strategies,
            &cycle,
            self.config.delta,
            self.config.equilibrium_check,
        );
        let (responses, punishment_like) = if self.config.impulse == Impulse::Invitation {
            let results: Vec<(u8, u8)> = (0..space.firms)
                .filter_map(|f| {
                    invitation(
                        game,
                        space,
                        &strategies,
                        &cycle,
                        f,
                        self.config.invitation_hold,
                    )
                })
                .collect();
            let share = if results.is_empty() {
                f64::NAN
            } else {
                results
                    .iter()
                    .filter(|(back, before)| back < before)
                    .count() as f64
                    / results.len() as f64
            };
            (self.responses(&strategies, &cycle), share)
        } else {
            let r = self.responses(&strategies, &cycle);
            let share = if r.is_empty() {
                f64::NAN
            } else {
                r.iter().filter(|x| x.punishment_like()).count() as f64 / r.len() as f64
            };
            (r, share)
        };
        let off: Vec<usize> = (0..space.states)
            .filter(|s| !cycle.states.contains(s))
            .collect();
        let stale_greedy = if off.is_empty() {
            0.0
        } else {
            off.iter()
                .map(|&s| {
                    (0..space.firms)
                        .filter(|&i| strategies[i][s] == self.initial[i][s])
                        .count()
                })
                .sum::<usize>() as f64
                / (off.len() * space.firms) as f64
        };
        Outcome {
            converged: self.converged,
            periods: self.period.saturating_sub(u64::from(self.config.window)),
            rp_complete: rp_complete(space, &strategies, &cycle),
            strategies,
            gains,
            gain,
            window_gain,
            discounted_gain,
            equilibrium: eq,
            responses,
            punishment_like,
            stale_greedy,
            fumbling: self.quiet_from.map(|q| self.last_change.saturating_sub(q)),
            touched: self.touched_count as f64 / (space.firms * space.states * space.prices) as f64,
            cycle,
        }
    }

    /// den Boer, Meylahn & Schinkel's Δ̃: the discounted profit over the
    /// first T_δ periods, normalized by the weights' sum (NaN before then,
    /// unless the session ended first).
    pub fn discounted_gain(&self) -> f64 {
        let t_d = u64::from(self.horizon).min(self.period);
        if t_d == 0 || (self.period < u64::from(self.horizon) && !self.is_finished()) {
            return f64::NAN;
        }
        let d = self.config.delta;
        let sum_w = if d > 0.0 {
            (1.0 - d.powi(t_d as i32)) / (1.0 - d)
        } else {
            1.0
        };
        (0..self.space.firms)
            .map(|i| self.game.gain(i, self.discounted[i] / sum_w))
            .sum::<f64>()
            / self.space.firms as f64
    }

    /// T_δ, the periods `discounted_gain` covers.
    pub fn horizon(&self) -> u32 {
        self.horizon
    }

    fn record(&mut self) {
        let n = self.space.firms;
        let mut s = CollusionSnapshot {
            tick: self.tick,
            stable: self.stable,
            greedy_changes: self.tick_changes,
            ..CollusionSnapshot::default()
        };
        if let Some(a) = self.recent.back() {
            s.price_1 = self.game.grid[0][usize::from(a[0])];
            s.price_2 = self.game.grid[1][usize::from(a[1])];
        }
        if self.tick_periods > 0 {
            let k = f64::from(self.tick_periods);
            s.profit_gain = self.tick_gain / k;
            s.explored = self.tick_explored / k;
        }
        (
            self.tick_gain,
            self.tick_explored,
            self.tick_changes,
            self.tick_periods,
        ) = (0.0, 0.0, 0, 0);
        let st = self.fixed.unwrap_or(self.state);
        s.greedy_price = (0..n)
            .map(|i| self.game.grid[i][usize::from(self.firms[i].greedy[st])])
            .sum::<f64>()
            / n as f64;
        s.epsilon = match self.config.exploration {
            Exploration::TwoPhase => {
                if self.period < u64::from(self.config.explore_for) {
                    1.0
                } else {
                    0.0
                }
            }
            _ => self.firms[0].eps,
        };
        self.results(&mut s);
        self.stats.push(s);
    }

    /// The session's results into snapshot `s` (NaN while it learns).
    fn results(&self, s: &mut CollusionSnapshot) {
        s.discounted_gain = self.discounted_gain();
        if let Some(o) = &self.outcome {
            s.converged = if o.converged { 1.0 } else { 0.0 };
            s.cycle_length = o.cycle.len() as f64;
            s.cycle_gain = o.gain;
            s.window_gain = o.window_gain;
            s.equilibrium_on_path = if o.equilibrium.on_path { 1.0 } else { 0.0 };
            s.punishment_like = o.punishment_like;
            s.rp_complete = if o.rp_complete { 1.0 } else { 0.0 };
            s.periods = o.periods as f64;
        }
    }

    /// The state firm `i`'s map cell (own price `own`, the next firm's
    /// `rival`) stands for: the current state with those two prices replaced.
    fn map_state(&self, i: usize, own: u8, rival: u8) -> usize {
        let sp = &self.space;
        if sp.memory == 0 {
            return 0;
        }
        let s = self.fixed.unwrap_or(self.state);
        let mut digits = vec![vec![0u8; sp.firms]; sp.memory];
        let mut rest = s;
        for d in (0..sp.memory).rev() {
            for j in (0..sp.firms).rev() {
                digits[d][j] = (rest % sp.prices) as u8;
                rest /= sp.prices;
            }
        }
        digits[0][i] = own;
        digits[0][(i + 1) % sp.firms] = rival;
        sp.encode(&digits)
    }

    fn view_state(&self, s: usize) -> StateView {
        let sp = &self.space;
        let m = sp.prices;
        let mut prices = vec![vec![0.0; sp.firms]; sp.memory];
        let mut rest = s;
        for d in (0..sp.memory).rev() {
            for j in (0..sp.firms).rev() {
                prices[d][j] = self.game.grid[j][rest % m];
                rest /= m;
            }
        }
        StateView {
            state: s,
            prices,
            q: self.firms.iter().map(|f| f.row(s, m).to_vec()).collect(),
            greedy: (0..sp.firms)
                .map(|i| self.game.grid[i][usize::from(self.firms[i].greedy[s])])
                .collect(),
            visits: self.visits[s],
        }
    }

    fn price_x(&self) -> usize {
        self.space.firms * (side(self.space.prices) + GAP)
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<CollusionInspection, String> {
        let (fw, fh) = Model::size(self);
        if x >= fw || y >= fh {
            return Err(format!("({x}, {y}) is not in the frame"));
        }
        let (cx, cy) = (x as usize, y as usize);
        let m = self.space.prices;
        let (side, cell) = (side(m), cell(m));
        let mut out = CollusionInspection {
            x,
            y,
            panel: None,
            firm: None,
            state: None,
            tick: self.tick,
            period: self.period,
            nash: self.game.nash.clone(),
            monopoly: self.game.monopoly.clone(),
            outcome: self.outcome.clone(),
            agent: None,
        };
        let i = cx / (side + GAP);
        if i < self.space.firms && cx % (side + GAP) < side && cy < side {
            let own = ((cx % (side + GAP)) / cell) as u8;
            let rival = (m - 1 - cy / cell) as u8;
            out.panel = Some("strategy");
            out.firm = Some(i);
            out.state = Some(self.view_state(self.map_state(i, own, rival)));
        } else if cx >= self.price_x() && cx < self.price_x() + SHOWN {
            out.panel = Some("prices");
            out.state = Some(self.view_state(self.fixed.unwrap_or(self.state)));
        } else if cx >= self.price_x() + SHOWN + GAP {
            out.panel = Some("response");
        }
        Ok(out)
    }
}

impl Model for CollusionWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Collusion(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        CollusionWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.firms.len()
    }

    /// FNV-1a over the tick, the state, the counter, and every firm's
    /// Q-values, greedy prices and exploration rate.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |bytes: &[u8]| {
            for &b in bytes {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(&self.tick.to_le_bytes());
        eat(&self.period.to_le_bytes());
        eat(&(self.state as u64).to_le_bytes());
        eat(&self.stable.to_le_bytes());
        for f in &self.firms {
            eat(&f.eps.to_bits().to_le_bytes());
            eat(&f.greedy);
            for q in &f.q {
                eat(&q.to_bits().to_le_bytes());
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let w = self.price_x() + SHOWN + GAP + HORIZON * STEP;
        let h = side(self.space.prices).max(PANEL_H);
        (w as u32, h as u32)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: CollusionMode = mode.parse()?;
        let (fw, fh) = Model::size(self);
        let mut c = Canvas { buf, wide: 0 };
        c.clear(fw as usize, fh as usize);
        let sp = &self.space;
        let m = sp.prices;
        let (side, cell) = (side(m), cell(m));
        let current = self.fixed.unwrap_or(self.state);
        let most = self.visits.iter().copied().max().unwrap_or(0);
        for i in 0..sp.firms {
            let x0 = i * (side + GAP);
            let last = sp.last(current);
            for own in 0..m as u8 {
                for rival in 0..m as u8 {
                    let s = self.map_state(i, own, rival);
                    let color = match mode {
                        CollusionMode::Price => price_color(self.firms[i].greedy[s], m),
                        CollusionMode::Visits => visit_color(self.visits[s], most),
                    };
                    let (px, py) = (
                        x0 + usize::from(own) * cell,
                        (m - 1 - usize::from(rival)) * cell,
                    );
                    let here = last
                        .as_ref()
                        .is_some_and(|l| l[i] == own && l[(i + 1) % sp.firms] == rival);
                    for dy in 0..cell {
                        for dx in 0..cell {
                            let edge = dx == 0 || dy == 0 || dx == cell - 1 || dy == cell - 1;
                            c.put(px + dx, py + dy, if here && edge { MARK } else { color });
                        }
                    }
                }
            }
        }
        // The last periods' prices, on firm 1's grid's scale.
        let (lo, hi) = (self.game.grid[0][0], self.game.grid[0][m - 1]);
        let x0 = self.price_x();
        for x in 0..SHOWN {
            c.put(x0 + x, row(self.game.nash[0], lo, hi), NASH);
            c.put(x0 + x, row(self.game.monopoly[0], lo, hi), MONOPOLY);
        }
        for (k, a) in self.recent.iter().enumerate() {
            for (j, &p) in a.iter().enumerate() {
                c.put(
                    x0 + k,
                    row(self.game.grid[j][usize::from(p)], lo, hi),
                    FIRMS[j % 4],
                );
            }
        }
        // The response to the first deviation, once finished.
        let x1 = x0 + SHOWN + GAP;
        for x in 0..HORIZON * STEP {
            c.put(x1 + x, PANEL_H - 1, AXIS);
            c.put(x1 + x, row(self.game.nash[0], lo, hi), NASH);
            c.put(x1 + x, row(self.game.monopoly[0], lo, hi), MONOPOLY);
        }
        if let Some(r) = self.outcome.as_ref().and_then(|o| o.responses.first()) {
            for (t, a) in r.path.iter().enumerate() {
                for (j, &p) in a.iter().enumerate() {
                    let y = row(self.game.grid[j][usize::from(p)], lo, hi);
                    for dx in 0..STEP - 1 {
                        c.put(x1 + t * STEP + dx, y, FIRMS[j % 4]);
                    }
                }
            }
        } else {
            for y in 0..PANEL_H - 1 {
                c.put(x1, y, BACK);
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

    /// One row per firm: its price now, its greedy price in the current
    /// state, its exploration rate and, once finished, its Δ.
    fn agents_csv(&self) -> String {
        let mut out = String::from("firm,price,greedy_price,epsilon,gain\n");
        let s = self.fixed.unwrap_or(self.state);
        for (i, f) in self.firms.iter().enumerate() {
            let price = self
                .recent
                .back()
                .map_or(f64::NAN, |a| self.game.grid[i][usize::from(a[i])]);
            let gain = self.outcome.as_ref().map_or(f64::NAN, |o| o.gains[i]);
            writeln!(
                out,
                "{},{},{},{},{}",
                i + 1,
                price,
                self.game.grid[i][usize::from(f.greedy[s])],
                f.eps,
                gain
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Collusion(next) = next else {
            return Err(wrong_model(ModelKind::Collusion, &next));
        };
        next.validate()?;
        let changes = self.config.structural_changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        if self.is_finished() {
            self.outcome = Some(self.analyze());
            let mut last = self.stats.latest().cloned().unwrap_or_default();
            self.results(&mut last);
            self.stats.truncate(self.stats.history().len() - 1);
            self.stats.push(last);
        }
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }

    /// A finished session holds its results: a sweep reads them.
    fn holds_when_finished(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collusion::config::{RngKind, Ties};

    fn world(edit: impl FnOnce(&mut CollusionConfig)) -> CollusionWorld {
        let mut c = CollusionConfig::default();
        edit(&mut c);
        CollusionWorld::new(c, 1).unwrap()
    }

    #[test]
    fn the_starting_q_is_equation_eight() {
        let w = world(|_| {});
        let want = [
            5.790, 6.008, 6.162, 6.252, 6.278, 6.244, 6.153, 6.010, 5.821, 5.593, 5.332, 5.047,
            4.744, 4.430, 4.111,
        ];
        for (a, v) in want.iter().enumerate() {
            assert!(
                (w.firms()[0].q[a] - v).abs() < 0.001,
                "{a}: {}",
                w.firms()[0].q[a]
            );
        }
        assert!(w.firms().iter().all(|f| f.greedy.iter().all(|&g| g == 4)));
        assert_eq!(w.stats.history().len(), 1);
    }

    #[test]
    fn a_short_session_converges_and_is_analyzed() {
        // Fast exploration decay and a short window: a session in seconds.
        let mut w = world(|c| {
            c.beta = 2e-4;
            c.window = 2_000;
        });
        w.run(2_000_000);
        assert!(w.is_finished());
        let o = w.outcome().unwrap();
        assert!(o.converged);
        assert_eq!(o.periods, w.period() - 2_000);
        assert_eq!(w.tick, w.period().div_ceil(1000));
        assert!(o.gain > -0.5 && o.gain <= 1.0, "{}", o.gain);
        assert!(!o.cycle.is_empty());
        let last = w.stats.latest().unwrap();
        assert_eq!(last.tick, w.tick);
        assert_eq!(last.cycle_gain, o.gain);
        assert!(!o.responses.is_empty());
        // Finished worlds do not move.
        let fp = w.fingerprint();
        w.run(10);
        assert_eq!(w.fingerprint(), fp);
    }

    #[test]
    fn the_cap_stops_a_session_that_never_settles() {
        let mut w = world(|c| {
            c.exploration = Exploration::Constant;
            c.epsilon = 1.0;
            c.cap = 5_000;
            c.window = 1_000;
        });
        w.run(10_000);
        assert!(w.is_finished());
        assert_eq!((w.period(), w.tick), (5_001, 6));
        assert!(!w.outcome().unwrap().converged);
    }

    #[test]
    fn exploration_decays_by_e_to_the_minus_beta() {
        let mut w = world(|c| c.periods_per_tick = 1);
        w.run(1000);
        let want = exp_neg(-4e-6 * 1000.0);
        assert!((w.firms()[0].eps - want).abs() < 1e-12);
        let mut two = world(|c| {
            c.exploration = Exploration::TwoPhase;
            c.explore_for = 10;
            c.periods_per_tick = 1;
        });
        two.run(10);
        assert_eq!(two.stats.latest().unwrap().epsilon, 0.0);
    }

    #[test]
    fn synchronous_updates_touch_every_price_in_the_state() {
        let mut w = world(|c| c.update = Update::Synchronous);
        let s = w.state();
        let before = w.firms()[0].row(s, 15).to_vec();
        w.step_period();
        let after = w.firms()[0].row(s, 15).to_vec();
        assert!(before.iter().zip(&after).all(|(a, b)| a != b));
        let mut a = world(|_| {});
        let s = a.state();
        let before = a.firms()[0].row(s, 15).to_vec();
        a.step_period();
        let changed = before
            .iter()
            .zip(a.firms()[0].row(s, 15))
            .filter(|(x, y)| x != y)
            .count();
        assert_eq!(changed, 1);
    }

    #[test]
    fn memoryless_firms_live_in_one_state_and_keep_delta() {
        let mut w = world(|c| {
            c.memory = 0;
            c.beta = 2e-4;
            c.window = 2_000;
        });
        w.run(2_000_000);
        let o = w.outcome().unwrap();
        assert_eq!(w.space().states, 1);
        assert!(o.cycle.is_point());
        assert_eq!(w.config.delta, 0.95);
    }

    #[test]
    fn the_authors_rng_seeds_by_session() {
        let a = world(|c| c.rng = RngKind::Calvano);
        let b = CollusionWorld::new(
            CollusionConfig {
                rng: RngKind::Calvano,
                ..CollusionConfig::default()
            },
            1,
        )
        .unwrap();
        assert_eq!(a.state(), b.state());
        let other = CollusionWorld::new(
            CollusionConfig {
                rng: RngKind::Calvano,
                ties: Ties::Random,
                ..CollusionConfig::default()
            },
            2,
        )
        .unwrap();
        // Session 1's first prices are the shared stream's first two draws:
        // ⌊15 · 0.2854⌋ = 4 and ⌊15 · 0.2534⌋ = 3.
        assert_eq!(a.space().last(a.state()), Some(vec![4, 3]));
        // Session 2's are its next two: ⌊15 · 0.0935⌋ = 1, then the fourth.
        assert_eq!(other.space().last(other.state()).unwrap()[0], 1);
    }

    #[test]
    fn boltzmann_and_three_firms_reach_their_pinned_fingerprints() {
        // crates/sugarscape-wasm/tests/web.rs repeats these: Boltzmann choice
        // runs on the portable exp, and no preset uses it.
        let mut b = world(|c| {
            c.exploration = Exploration::Boltzmann;
            c.temperature = 0.01;
            c.cooling = 1e-4;
        });
        b.run(500);
        assert_eq!(b.fingerprint(), 0xffb749a8db1a5481);
        let mut three = world(|c| {
            c.q_init = QInit::Random;
            c.firms = 3;
        });
        three.run(200);
        assert_eq!(three.fingerprint(), 0x0bab95a370db18eb);
    }

    #[test]
    fn inspect_reads_a_strategy_cell_and_live_edits_reanalyze() {
        let mut w = world(|c| {
            c.beta = 2e-4;
            c.window = 2_000;
        });
        w.run(2_000_000);
        let i = w.inspect(0, 0).unwrap();
        assert_eq!((i.panel, i.firm), (Some("strategy"), Some(0)));
        let st = i.state.unwrap();
        assert_eq!(st.q.len(), 2);
        assert_eq!(
            st.prices[0][1],
            w.game().grid[1][14],
            "top row: the rival at the top price"
        );
        let mut next = w.config.clone();
        next.impulse = Impulse::EveryPrice;
        w.set_config(ModelConfig::Collusion(next)).unwrap();
        let o = w.outcome().unwrap();
        assert_eq!(o.responses.len(), o.cycle.len() * 2 * 14);
        let mut bad = w.config.clone();
        bad.memory = 2;
        assert_eq!(
            w.set_config(ModelConfig::Collusion(bad)).unwrap_err()[0].field,
            "memory"
        );
        let mut buf = Vec::new();
        Model::render(&w, "price", "", &mut buf).unwrap();
        Model::render(&w, "visits", "", &mut buf).unwrap();
        assert!(Model::render(&w, "nope", "", &mut buf).is_err());
    }
}
```

Create `crates/sugarscape-core/src/collusion/presets.rs` with exactly this content:

```rust
//! Calvano et al.'s baseline, their code's readings, and the critics' tests.

use super::config::{
    BestResponseTo, CollusionConfig, EquilibriumCheck, Exploration, Grid, Impulse, RngKind, Ties,
    Update,
};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

const CCDP: &str = "Calvano, Calzolari, Denicolò & Pastorello 2020, AER 110(10)";
const AFP: &str = "Asker, Fershtman & Pakes 2021, NBER w28535";
const L24: &str = "Lambin 2024, SSRN 4498926";
const EL: &str = "Epivent & Lambin 2024, Economics Letters 237";

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut CollusionConfig),
) -> ModelPreset {
    let mut c = CollusionConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Collusion(c),
    }
}

/// The authors' code's readings where it differs from the paper.
pub fn as_coded(c: &mut CollusionConfig) {
    c.ties = Ties::Random;
    c.rng = RngKind::Calvano;
    c.cap = 1_250_000_000;
    c.equilibrium_check = EquilibriumCheck::OneShot;
    c.best_response_to = BestResponseTo::Code;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "collusion-calvano",
            "Two pricing algorithms",
            CCDP,
            "Calvano, Calzolari, Denicolò and Pastorello's baseline: two firms selling differentiated goods each pick one of 15 prices, using Q-learning on last period's two prices, exploring at random less and less (ε = e^(−βt), β = 4 × 10⁻⁶); a session ends when neither firm's strategy has changed for 100 000 periods. The paper: the algorithms learn to price far above the competitive level (profit gain Δ = 0.849) and punish a rival's price cut. Measured (1 000 sessions, the paper's readings): Δ 0.851 ± 0.004, settled after about 1.76 million periods; 62.5 % settle on one price pair, the rest cycle. After a one-period cut to the static best response the rival cuts too in 92 % of cases and comes back within about 10 periods. But only 0.2 % of sessions are a best response to each other on the path (the code's weaker one-period test passes 49.7 %), and in 91 % of the sessions that punish a cut some other deviation goes unpunished. Watch the strategy maps warm up from blue to orange; Inspect a cell for both firms' Q-values.",
            |_| {},
        ),
        preset(
            "collusion-code",
            "As the authors' code ran it",
            CCDP,
            "The paper's model as its authors' Fortran runs it: ties broken at random, their RAN2 generator seeded by session number (the seed), a cap of 1.25 × 10⁹, an equilibrium test that checks only one-period deviations, and Fig. 4's deviation as the code computes it. Every session is theirs exactly: the first 100 match the code built from their replication package in the same period and all 450 strategy cells. Measured (sessions 1–1 000): Table I to the digit — Δ 0.849, 50.5 % in equilibrium on the path (by the code's test), cycles of 1, 2 and 3+ periods 64.3 / 23.8 / 11.9 %; Table A5's responses — the rival's price change −0.127, deviations unprofitable in 0.936 (the text says 'more than 95 %'), punishment 5.705 periods.",
            as_coded,
        ),
        preset(
            "collusion-no-memory",
            "Algorithms that remember nothing",
            L24,
            "Lambin's 'key robustness test': firms that condition on nothing (one state), so no punishment is possible, with the future still valued (δ = 0.95). The paper's appendix only ran this with δ = 0 — its code sets δ to 0 whenever memory is 0. Lambin: memoryless algorithms 'seem to collude even more effectively'. Measured (1 000 sessions): Δ 0.958 against 0.851 with memory; mean greedy price 1.862 against 1.791 at 2 × 10⁶ periods (Lambin: about 1.85 against 1.76). No session is an equilibrium and none answers any deviation: high prices without any scheme to sustain them. With δ = 0 (the appendix's reading) Δ is 0.255.",
            |c| c.memory = 0,
        ),
        preset(
            "collusion-myopic",
            "Algorithms that ignore the future",
            CCDP,
            "δ = 0: firms value only this period's profit, so no reward–punishment scheme can pay. Measured (1 000 sessions): Δ 0.212 ± 0.003 — the paper's own Fig. 3 shows 0.212 at δ = 0 — about a quarter of the baseline's 0.851. Whatever keeps these prices up, it is not collusion; strategies can account for at most the remaining 0.64.",
            |c| c.delta = 0.0,
        ),
        preset(
            "collusion-two-phase",
            "Explore first, then never",
            L24,
            "Lambin's Theorem 1: after a phase of purely random pricing (here 1 000 periods) and none afterwards, myopic firms (δ = 0) settle where the best symmetric profit they have tried beats what their stale estimates promise — on this grid both at 1.6990 (Δ 0.707) — with or without memory. Measured (1 000 sessions, one period of memory): Δ 0.687, but only 27 % of sessions end with both at 1.6990 (the most common outcome); without memory 13 %; at δ = 0.95 the theorem's 1.7377 is reached in under 4 %. The theorem is a mean-field limit; with α = 0.15 the Q-values stay noisy.",
            |c| {
                c.exploration = Exploration::TwoPhase;
                c.delta = 0.0;
            },
        ),
        preset(
            "collusion-synchronous",
            "Learning from every price",
            AFP,
            "Asker, Fershtman and Pakes: each period a firm updates the value of every price toward what it would have earned against the rival's actual price, not only the price it charged. Measured (1 000 sessions): Δ 0.345 against 0.851 — less than half. Three sessions in four still settle on one price pair; 73 % are a best response on the path.",
            |c| c.update = Update::Synchronous,
        ),
        preset(
            "collusion-explore-more",
            "Algorithms that keep experimenting",
            CCDP,
            "β ten times smaller (4 × 10⁻⁷): the firms experiment for ten times longer before settling (about 15 million periods). Abada and Lambin argue more exploration restores competition. Measured (200 sessions): Δ 0.727 against 0.851 — lower, but not by half. A constant ε = 0.05 never settles (no session in 10⁸ periods); read at 10⁷ periods, Δ 0.559 (100 sessions).",
            |c| c.beta = 4e-7,
        ),
        preset(
            "collusion-every-price",
            "Deviations up as well as down",
            EL,
            "Epivent and Lambin's test (their Table 1): one-period deviations to every other price, up as well as down. If the price cuts that follow were punishment, a rival's price rise — an invitation to collude — should not draw one. Measured (1 000 sessions, sessions settled on a symmetric price): in all 25 (price, higher price) cells with at least 30 sessions the rival cuts its price the next period, by 9.9 % on average against 13.0 % after a cut; responses that look like punishment follow 83 % of increases and 93 % of cuts.",
            |c| c.impulse = Impulse::EveryPrice,
        ),
        preset(
            "collusion-invitation",
            "An invitation to raise prices",
            EL,
            "Epivent and Lambin's invitation (their Fig. 2): one firm raises its price a step and the rival is made to match it for the next periods; after 5 periods the first firm decides again. Measured (617 sessions settled on one price): it cuts back below where it started in 84 % of them, by 5.0 grid steps on average.",
            |c| c.impulse = Impulse::Invitation,
        ),
        preset(
            "collusion-below-nash",
            "A grid below the Nash price",
            EL,
            "Epivent and Lambin's appendix grid: 15 prices from 1.25 up to 1.47, the Nash price (1.47293). Measured (10 000 sessions): 72 % end with both firms at the top price, 5 % cycle, 22 % settle on other points (they report 53, 9 and 38 %); Δ is near 0. In the sessions below the top, price increases draw punishment-like responses more often than cuts (96 % against 85 %).",
            |c| {
                c.grid = Grid::BelowNash;
                c.impulse = Impulse::EveryPrice;
            },
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_set_what_they_say() {
        let got: Vec<(&str, CollusionConfig)> = presets()
            .into_iter()
            .map(|p| match p.config {
                ModelConfig::Collusion(c) => (p.id, c),
                _ => panic!("{} is not a collusion preset", p.id),
            })
            .collect();
        let find = |id| got.iter().find(|(i, _)| *i == id).unwrap().1.clone();
        assert_eq!(find("collusion-calvano"), CollusionConfig::default());
        let code = find("collusion-code");
        assert_eq!(
            (code.ties, code.rng, code.cap, code.equilibrium_check),
            (
                Ties::Random,
                RngKind::Calvano,
                1_250_000_000,
                EquilibriumCheck::OneShot
            )
        );
        let none = find("collusion-no-memory");
        assert_eq!((none.memory, none.delta), (0, 0.95));
        assert_eq!(find("collusion-myopic").delta, 0.0);
        assert_eq!(find("collusion-synchronous").update, Update::Synchronous);
        assert_eq!(find("collusion-below-nash").grid, Grid::BelowNash);
        assert!(got.iter().all(|(_, c)| c.validate().is_ok()));
        assert!(presets().iter().all(|p| !p.description.trim().is_empty()));
    }
}
```

Create `crates/sugarscape-core/src/collusion/mod.rs` with exactly this content:

```rust
//! Algorithmic Collusion: Calvano, Calzolari, Denicolò and Pastorello,
//! "Artificial Intelligence, Algorithmic Pricing, and Collusion" (AER 2020),
//! checked against the authors' own code, with the critics' tests — Asker,
//! Fershtman and Pakes; Lambin; Epivent and Lambin; den Boer, Meylahn and
//! Schinkel; Eschenbaum, Mellgren and Zahn — as named switches.
//! See docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md.

pub mod analysis;
mod config;
pub mod demand;
pub mod learner;
mod presets;
pub mod ran2;
mod stats;
mod view;
mod world;

pub use config::{
    schema, BestResponseTo, CollusionConfig, EquilibriumCheck, Exploration, Grid, Impulse, QInit,
    RngKind, Ties, Update, BELOW_BOTTOM, Q_BUDGET,
};
pub use presets::{as_coded, presets};
pub use stats::{CollusionSnapshot, SERIES};
pub use world::{CollusionInspection, CollusionMode, CollusionWorld, Outcome, StateView};
```

- [ ] **Step 2: Wire the kind in**

The kind joins every match beside `Firms`, the catalog gains the presets after `hoard`, `titles.rs` gains ten titles, and `MODEL_GOLDEN` ten entries. Save this as `/tmp/collusion-task.patch` and apply it with `git apply /tmp/collusion-task.patch` (it touches only `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/titles.rs`, `crates/sugarscape-core/tests/golden.rs`):

```diff
diff --git a/crates/sugarscape-core/src/lib.rs b/crates/sugarscape-core/src/lib.rs
index 5373156..735223d 100644
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -11,6 +11,7 @@ pub mod bali;
 pub mod bits;
 pub mod civil;
 pub mod classes;
+pub mod collusion;
 pub mod config;
 pub mod culture;
 pub mod dpd;
diff --git a/crates/sugarscape-core/src/model.rs b/crates/sugarscape-core/src/model.rs
index 855bb72..2b4ecfb 100644
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -11,6 +11,7 @@ use crate::ants::{AntsConfig, AntsWorld};
 use crate::bali::{BaliConfig, BaliWorld};
 use crate::civil::{CivilConfig, CivilWorld};
 use crate::classes::{ClassesConfig, ClassesWorld};
+use crate::collusion::{CollusionConfig, CollusionWorld};
 use crate::config::{Config, FieldError};
 use crate::culture::{CultureConfig, CultureWorld};
 use crate::dpd::{DpdConfig, DpdWorld};
@@ -72,10 +73,11 @@ pub enum ModelKind {
     Tipping,
     Hoard,
     Firms,
+    Collusion,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 27] = [
+    pub const ALL: [ModelKind; 28] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -103,6 +105,7 @@ impl ModelKind {
         ModelKind::Tipping,
         ModelKind::Hoard,
         ModelKind::Firms,
+        ModelKind::Collusion,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -134,6 +137,7 @@ impl ModelKind {
             ModelKind::Tipping => "tipping",
             ModelKind::Hoard => "hoard",
             ModelKind::Firms => "firms",
+            ModelKind::Collusion => "collusion",
         }
     }
 
@@ -168,6 +172,7 @@ impl ModelKind {
             ModelKind::Tipping => crate::tipping::schema(),
             ModelKind::Hoard => crate::hoard::schema(),
             ModelKind::Firms => crate::firms::schema(),
+            ModelKind::Collusion => crate::collusion::schema(),
         }
     }
 }
@@ -208,6 +213,7 @@ pub enum ModelConfig {
     Tipping(TippingConfig),
     Hoard(HoardConfig),
     Firms(FirmsConfig),
+    Collusion(CollusionConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -240,6 +246,7 @@ enum Tagged<'a> {
     Tipping(&'a TippingConfig),
     Hoard(&'a HoardConfig),
     Firms(&'a FirmsConfig),
+    Collusion(&'a CollusionConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -279,6 +286,7 @@ impl Serialize for ModelConfig {
             ModelConfig::Tipping(c) => Tagged::Tipping(c).serialize(s),
             ModelConfig::Hoard(c) => Tagged::Hoard(c).serialize(s),
             ModelConfig::Firms(c) => Tagged::Firms(c).serialize(s),
+            ModelConfig::Collusion(c) => Tagged::Collusion(c).serialize(s),
         }
     }
 }
@@ -313,6 +321,7 @@ impl ModelConfig {
             ModelConfig::Tipping(_) => ModelKind::Tipping,
             ModelConfig::Hoard(_) => ModelKind::Hoard,
             ModelConfig::Firms(_) => ModelKind::Firms,
+            ModelConfig::Collusion(_) => ModelKind::Collusion,
         }
     }
 
@@ -422,6 +431,9 @@ impl ModelConfig {
             "firms" => serde_json::from_value(value)
                 .map(ModelConfig::Firms)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "collusion" => serde_json::from_value(value)
+                .map(ModelConfig::Collusion)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             "zi" => serde_json::from_value(value)
                 .map(ModelConfig::Zi)
                 .map_err(|e| FieldError::new("config", e.to_string())),
@@ -431,7 +443,7 @@ impl ModelConfig {
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds, retirement, punishment, zi, bali, line, tipping, hoard or firms)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd, norms, agreement, image, farol, ants, thresholds, retirement, punishment, zi, bali, line, tipping, hoard, firms or collusion)"
                 ),
             )),
         }
@@ -466,6 +478,7 @@ impl ModelConfig {
             ModelConfig::Tipping(c) => c.validate(),
             ModelConfig::Hoard(c) => c.validate(),
             ModelConfig::Firms(c) => c.validate(),
+            ModelConfig::Collusion(c) => c.validate(),
         }
     }
 
@@ -500,6 +513,7 @@ impl ModelConfig {
             ModelConfig::Tipping(c) => set_path(c, path, value).map(ModelConfig::Tipping),
             ModelConfig::Hoard(c) => set_path(c, path, value).map(ModelConfig::Hoard),
             ModelConfig::Firms(c) => set_path(c, path, value).map(ModelConfig::Firms),
+            ModelConfig::Collusion(c) => set_path(c, path, value).map(ModelConfig::Collusion),
         }
     }
 
@@ -533,7 +547,8 @@ impl ModelConfig {
             | ModelConfig::Line(_)
             | ModelConfig::Tipping(_)
             | ModelConfig::Hoard(_)
-            | ModelConfig::Firms(_) => None,
+            | ModelConfig::Firms(_)
+            | ModelConfig::Collusion(_) => None,
         }
     }
 
@@ -576,6 +591,10 @@ impl ModelConfig {
                 .map(|s| s.to_string())
                 .collect(),
             ModelConfig::Firms(_) => crate::firms::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Collusion(_) => crate::collusion::SERIES
+                .iter()
+                .map(|s| s.to_string())
+                .collect(),
         }
     }
 }
@@ -772,6 +791,7 @@ pub enum ModelWorld {
     Tipping(Box<TippingWorld>),
     Hoard(Box<HoardWorld>),
     Firms(Box<FirmsWorld>),
+    Collusion(Box<CollusionWorld>),
 }
 
 impl ModelWorld {
@@ -830,6 +850,9 @@ impl ModelWorld {
             ModelConfig::Tipping(c) => ModelWorld::Tipping(Box::new(TippingWorld::new(c, seed)?)),
             ModelConfig::Hoard(c) => ModelWorld::Hoard(Box::new(HoardWorld::new(c, seed)?)),
             ModelConfig::Firms(c) => ModelWorld::Firms(Box::new(FirmsWorld::new(c, seed)?)),
+            ModelConfig::Collusion(c) => {
+                ModelWorld::Collusion(Box::new(CollusionWorld::new(c, seed)?))
+            }
         })
     }
 
@@ -862,6 +885,7 @@ impl ModelWorld {
             ModelWorld::Tipping(_) => ModelKind::Tipping,
             ModelWorld::Hoard(_) => ModelKind::Hoard,
             ModelWorld::Firms(_) => ModelKind::Firms,
+            ModelWorld::Collusion(_) => ModelKind::Collusion,
         }
     }
 
@@ -894,6 +918,7 @@ impl ModelWorld {
             ModelWorld::Tipping(w) => w.as_ref(),
             ModelWorld::Hoard(w) => w.as_ref(),
             ModelWorld::Firms(w) => w.as_ref(),
+            ModelWorld::Collusion(w) => w.as_ref(),
         }
     }
 
@@ -926,6 +951,7 @@ impl ModelWorld {
             ModelWorld::Tipping(w) => w.as_mut(),
             ModelWorld::Hoard(w) => w.as_mut(),
             ModelWorld::Firms(w) => w.as_mut(),
+            ModelWorld::Collusion(w) => w.as_mut(),
         }
     }
 
@@ -1034,6 +1060,7 @@ impl ModelWorld {
             ModelWorld::Tipping(w) => copy_without_history!(Tipping, w),
             ModelWorld::Hoard(w) => copy_without_history!(Hoard, w),
             ModelWorld::Firms(w) => copy_without_history!(Firms, w),
+            ModelWorld::Collusion(w) => copy_without_history!(Collusion, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -1088,6 +1115,7 @@ impl ModelWorld {
             (ModelWorld::Tipping(live), ModelWorld::Tipping(kept)) => restore_into!(live, kept),
             (ModelWorld::Hoard(live), ModelWorld::Hoard(kept)) => restore_into!(live, kept),
             (ModelWorld::Firms(live), ModelWorld::Firms(kept)) => restore_into!(live, kept),
+            (ModelWorld::Collusion(live), ModelWorld::Collusion(kept)) => restore_into!(live, kept),
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1536,6 +1564,30 @@ mod tests {
         assert_eq!(w.model().tick(), 0);
     }
 
+    #[test]
+    fn collusion_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "collusion", "memory": 0, "delta": 0.5, "window": 50}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Collusion);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(
+            (json["model"].as_str(), json["prices"].as_u64()),
+            (Some("collusion"), Some(15))
+        );
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["price_1", "price_2"]);
+        let e = ModelConfig::from_json(r#"{"model": "collusion", "firms": 9}"#).unwrap_err();
+        assert_eq!(e[0].field, "firms");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!(w.kind(), ModelKind::Collusion);
+        let cp = w.checkpoint().expect("collusion worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
     #[test]
     fn bali_configs_round_trip_with_their_tag() {
         let c = ModelConfig::from_json(
@@ -1656,7 +1708,8 @@ mod tests {
                 "line",
                 "tipping",
                 "hoard",
-                "firms"
+                "firms",
+                "collusion"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
diff --git a/crates/sugarscape-core/src/presets.rs b/crates/sugarscape-core/src/presets.rs
index 533373d..edeecf5 100644
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -1478,6 +1478,7 @@ pub fn catalog() -> Vec<ModelPreset> {
     out.extend(crate::line::presets());
     out.extend(crate::tipping::presets());
     out.extend(crate::hoard::presets());
+    out.extend(crate::collusion::presets());
     out
 }
 
diff --git a/crates/sugarscape-core/src/titles.rs b/crates/sugarscape-core/src/titles.rs
index 4d7103f..4ebc085 100644
--- a/crates/sugarscape-core/src/titles.rs
+++ b/crates/sugarscape-core/src/titles.rs
@@ -3,7 +3,7 @@
 //! paper (`source`) stay on the preset as its reference.
 
 /// Titles by preset id, in catalog order.
-pub const TITLES: [(&str, &str); 379] = [
+pub const TITLES: [(&str, &str); 389] = [
     (
         "ii-1-instant",
         "Sugar grows back at once: agents climb the best ridges and the poorly endowed starve",
@@ -1481,6 +1481,46 @@ pub const TITLES: [(&str, &str); 379] = [
         "firms-2013",
         "Axtell's 2013 settings give Zipf's law, with brief giant firms",
     ),
+    (
+        "collusion-calvano",
+        "Two pricing algorithms learn to keep prices high, but not as a best response",
+    ),
+    (
+        "collusion-code",
+        "The authors' own sessions, period for period",
+    ),
+    (
+        "collusion-no-memory",
+        "Algorithms that remember nothing price even higher",
+    ),
+    (
+        "collusion-myopic",
+        "Algorithms that ignore the future still price above Nash",
+    ),
+    (
+        "collusion-two-phase",
+        "Explore at random, then never: prices settle above Nash, but not where Lambin's theorem says",
+    ),
+    (
+        "collusion-synchronous",
+        "Learning from every price brings prices down",
+    ),
+    (
+        "collusion-explore-more",
+        "Ten times slower exploration decay barely lowers prices",
+    ),
+    (
+        "collusion-every-price",
+        "Prices are cut after a rival's price rise too",
+    ),
+    (
+        "collusion-invitation",
+        "An invitation to raise prices is met with a price cut",
+    ),
+    (
+        "collusion-below-nash",
+        "On a grid with no room above Nash, prices settle at the top",
+    ),
 ];
 
 /// The title of preset `id`, or "" if it has none.
diff --git a/crates/sugarscape-core/tests/golden.rs b/crates/sugarscape-core/tests/golden.rs
index 8113b0e..c2c9b73 100644
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -399,6 +399,18 @@ const MODEL_GOLDEN: &[(&str, u64)] = &[
     ("firms-hiring-100", 0xd8fb44c3f2acfa13),
     ("firms-random-choices", 0xd5650a4bd4548db4),
     ("firms-2013", 0xc7acf4a34472ca50),
+    // Algorithmic Collusion (200 ticks of 1 000 periods). The deviation presets learn as the
+    // baseline does (they differ only in the analysis after convergence).
+    ("collusion-calvano", 0xbfdc574972d2efcd),
+    ("collusion-code", 0xa6ab4cce772a8a70),
+    ("collusion-no-memory", 0x172d003af01b8b94),
+    ("collusion-myopic", 0xf53b3963ec92f7f8),
+    ("collusion-two-phase", 0xff7d1eb24dd21e2f),
+    ("collusion-synchronous", 0xcbb6eed6a17dd495),
+    ("collusion-explore-more", 0x90e7c74eb3f03d51),
+    ("collusion-every-price", 0xbfdc574972d2efcd),
+    ("collusion-invitation", 0xbfdc574972d2efcd),
+    ("collusion-below-nash", 0x4b11e4afb4a24a8c),
 ];
 
 fn fingerprint(id: &str) -> u64 {
```

- [ ] **Step 3: The docking test and its fixture**

Copy the fixture, which the authors' Fortran wrote in planning (100 sessions of Table I's input; MIT):

```bash
cp /private/tmp/claude-501/-Users-nathan-Projects-ndouglas-SugarScape/71002cc9-c4cc-431c-978a-bd677989e843/scratchpad/cs/crates/sugarscape-core/tests/fixtures/calvano-sessions.json crates/sugarscape-core/tests/fixtures/calvano-sessions.json
shasum -a 256 crates/sugarscape-core/tests/fixtures/calvano-sessions.json
```

Expected: `73e25435e7fb9ff8cf017dd01b893f80f844db6a600085d3ec2c7336cb7e6406`.

Create `crates/sugarscape-core/tests/collusion.rs` with exactly this content:

```rust
//! Algorithmic Collusion against the authors' own code: under their readings
//! (random ties, their RAN2 seeded −session, their cap), a session reaches
//! the same strategies in the same period as their Fortran does
//! (tests/fixtures/calvano-sessions.json, from their replication package).

use sugarscape_core::collusion::{as_coded, CollusionConfig, CollusionWorld};

#[derive(serde::Deserialize)]
struct Fixture {
    sessions: Vec<Session>,
}

#[derive(serde::Deserialize)]
struct Session {
    session: u64,
    periods: u64,
    cycle: usize,
    profits: [f64; 2],
    strategies: Vec<u8>,
}

fn fixture() -> Fixture {
    serde_json::from_str(include_str!("fixtures/calvano-sessions.json")).unwrap()
}

fn check(s: &Session) {
    let mut c = CollusionConfig::default();
    as_coded(&mut c);
    let mut w = CollusionWorld::new(c, s.session).unwrap();
    while !w.is_finished() {
        w.run(1_000_000);
    }
    let o = w.outcome().unwrap();
    assert!(o.converged, "session {}", s.session);
    assert_eq!(o.periods, s.periods, "session {}: periods", s.session);
    let ours: Vec<u8> = (0..225)
        .flat_map(|st| [o.strategies[0][st], o.strategies[1][st]])
        .collect();
    assert_eq!(ours, s.strategies, "session {}: strategies", s.session);
    assert_eq!(o.cycle.len(), s.cycle, "session {}: cycle", s.session);
    for i in 0..2 {
        assert!(
            (o.cycle.profits[i] - s.profits[i]).abs() < 1e-12,
            "session {}: firm {i}'s profit {} against {}",
            s.session,
            o.cycle.profits[i],
            s.profits[i]
        );
    }
}

#[test]
fn the_first_sessions_are_the_authors_period_for_period() {
    for s in fixture().sessions.iter().take(3) {
        check(s);
    }
}

/// All 100 (about a minute in release): `cargo test -p sugarscape-core
/// --release --test collusion -- --ignored`.
#[test]
#[ignore]
fn every_fixture_session_is_the_authors_period_for_period() {
    for s in &fixture().sessions {
        check(s);
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --release -p sugarscape-core --lib collusion`
Expected: 38 passed (config 6, ran2 2, demand 6, learner 4, analysis 8, view 1, world 9, presets 1, and `model::tests::collusion_configs_round_trip_with_their_tag`).

Run: `cargo test --release -p sugarscape-core --test golden --test collusion`
Expected: golden 6 passed (1 ignored); collusion 1 passed (1 ignored).

Run: `cargo test --release -p sugarscape-core --test collusion -- --ignored`
Expected: 1 passed (all 100 fixture sessions identical to the Fortran's: periods, the 450 strategy cells, the cycle and its profits to 10⁻¹²; about a minute).

Run: `cargo test --release -p sugarscape-core model::tests::collusion_configs_round_trip_with_their_tag titles presets`
Expected: all pass (every preset has a title and a golden entry).

Run: `cargo fmt --all && cargo +stable clippy --all-targets -- -D warnings`
Expected: no output from clippy.

- [ ] **Commit**

```bash
git add crates/sugarscape-core/src/collusion crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/titles.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/collusion.rs crates/sugarscape-core/tests/fixtures/calvano-sessions.json
git commit -F - <<'EOF'
Algorithmic collusion: the model kind (Calvano et al. 2020), docked against the authors' Fortran

Claude-Session: https://claude.ai/code/session_01XxEZRQqmFWQaPDVcu7Me1w
EOF
```

### Task 2: Sweeps, the CLI and WASM

**Files:**
- Create: `sweeps/{collusion-table-i,collusion-alpha-beta,collusion-delta,collusion-memory,collusion-myopic,collusion-two-phase,collusion-every-price,collusion-below-nash,collusion-invitation,collusion-synchronous,collusion-exploration,collusion-timescale,collusion-rp-complete}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets (`collusion-calvano` as every sweep's base) and series (`cycle_gain`, `punishment_like`, `discounted_gain`, `rp_complete`); `crate::model::Model::latest_value`.
- Produces: thirteen built-in sweeps (ids under Global Constraints); the CLI's `(it converged)` / `(its cap)`; WASM tests `collusion_sims_match_the_native_golden_entries` and `collusion_boltzmann_and_three_firms_match_the_native_fingerprints`.

- [ ] **Step 1: The sweeps**

Create `sweeps/collusion-table-i.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: Table I, the paper's readings and the code's",
  "description": "Calvano et al.'s Table I baseline under the paper's readings (ties to the lowest price, the best-response equilibrium test) and under the authors' code's (random ties, their RAN2 seeded by session, their cap): seeds 1–100 under the code's readings are the authors' own sessions 1–100. Measured (release, sessions 1–100, recorded 2026-10-01): Δ 0.861 (paper's readings) and 0.849 (code's). Over 1 000 sessions (the survey) both give the paper's 0.849–0.851.",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Readings",
    "values": [
      {
        "at": 1,
        "name": "The paper's",
        "set": {}
      },
      {
        "at": 2,
        "name": "The code's",
        "set": {
          "ties": "random",
          "rng": "calvano",
          "cap": 1250000000,
          "equilibrium_check": "one_shot",
          "best_response_to": "code"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-alpha-beta.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: the profit gain over α and β (Figs. 1–2)",
  "description": "Calvano et al.'s Figs. 1–2: the profit gain Δ over the learning rate α and the exploration decay β, on a 10 × 10 subgrid of their 91 × 100. Measured (release, sessions 1–100, recorded 2026-10-01), the paper's readings: Δ from 0.71 (α 0.025, β 2 × 10⁻⁵) to 0.89 (α 0.05, β 0.2 × 10⁻⁵); 0.86 at the baseline cell. Against Fig. 1's cells (read from the package's vector figure) the mean gap is 0.008 and the worst 0.040 (the survey). About 10 000 sessions: several minutes.",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "α",
    "values": [
      {
        "at": 1,
        "name": "α = 0.025",
        "set": {
          "alpha": 0.025
        }
      },
      {
        "at": 2,
        "name": "α = 0.05",
        "set": {
          "alpha": 0.05
        }
      },
      {
        "at": 3,
        "name": "α = 0.075",
        "set": {
          "alpha": 0.075
        }
      },
      {
        "at": 4,
        "name": "α = 0.1",
        "set": {
          "alpha": 0.1
        }
      },
      {
        "at": 5,
        "name": "α = 0.125",
        "set": {
          "alpha": 0.125
        }
      },
      {
        "at": 6,
        "name": "α = 0.15",
        "set": {
          "alpha": 0.15
        }
      },
      {
        "at": 7,
        "name": "α = 0.175",
        "set": {
          "alpha": 0.175
        }
      },
      {
        "at": 8,
        "name": "α = 0.2",
        "set": {
          "alpha": 0.2
        }
      },
      {
        "at": 9,
        "name": "α = 0.225",
        "set": {
          "alpha": 0.225
        }
      },
      {
        "at": 10,
        "name": "α = 0.25",
        "set": {
          "alpha": 0.25
        }
      }
    ]
  },
  "series": {
    "label": "β × 10⁵",
    "values": [
      {
        "at": 1,
        "name": "β = 0.2 × 10⁻⁵",
        "set": {
          "beta": 2.0000000000000003e-06
        }
      },
      {
        "at": 2,
        "name": "β = 0.4 × 10⁻⁵",
        "set": {
          "beta": 4.000000000000001e-06
        }
      },
      {
        "at": 3,
        "name": "β = 0.6 × 10⁻⁵",
        "set": {
          "beta": 6e-06
        }
      },
      {
        "at": 4,
        "name": "β = 0.8 × 10⁻⁵",
        "set": {
          "beta": 8.000000000000001e-06
        }
      },
      {
        "at": 5,
        "name": "β = 1.0 × 10⁻⁵",
        "set": {
          "beta": 1e-05
        }
      },
      {
        "at": 6,
        "name": "β = 1.2 × 10⁻⁵",
        "set": {
          "beta": 1.2e-05
        }
      },
      {
        "at": 7,
        "name": "β = 1.4 × 10⁻⁵",
        "set": {
          "beta": 1.4e-05
        }
      },
      {
        "at": 8,
        "name": "β = 1.6 × 10⁻⁵",
        "set": {
          "beta": 1.6000000000000003e-05
        }
      },
      {
        "at": 9,
        "name": "β = 1.8 × 10⁻⁵",
        "set": {
          "beta": 1.8e-05
        }
      },
      {
        "at": 10,
        "name": "β = 2.0 × 10⁻⁵",
        "set": {
          "beta": 2e-05
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-delta.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: the profit gain against δ (Fig. 6)",
  "description": "Calvano et al.'s Fig. 3: Δ against the discount factor δ. Measured (release, sessions 1–100, recorded 2026-10-01): 0.228 at δ = 0, lowest 0.157 at 0.3, 0.483 at 0.8, 0.861 at 0.95, 0.940 at 0.99 — the figure's 0.212, 0.156 at 0.34, 0.849 and 0.934. At δ = 0 no reward–punishment scheme can pay, yet Δ is a quarter of the baseline's, in the paper's own figure too.",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "δ",
    "values": [
      {
        "at": 1,
        "name": "δ = 0.0",
        "set": {
          "delta": 0.0
        }
      },
      {
        "at": 2,
        "name": "δ = 0.1",
        "set": {
          "delta": 0.1
        }
      },
      {
        "at": 3,
        "name": "δ = 0.2",
        "set": {
          "delta": 0.2
        }
      },
      {
        "at": 4,
        "name": "δ = 0.3",
        "set": {
          "delta": 0.3
        }
      },
      {
        "at": 5,
        "name": "δ = 0.4",
        "set": {
          "delta": 0.4
        }
      },
      {
        "at": 6,
        "name": "δ = 0.5",
        "set": {
          "delta": 0.5
        }
      },
      {
        "at": 7,
        "name": "δ = 0.6",
        "set": {
          "delta": 0.6
        }
      },
      {
        "at": 8,
        "name": "δ = 0.7",
        "set": {
          "delta": 0.7
        }
      },
      {
        "at": 9,
        "name": "δ = 0.8",
        "set": {
          "delta": 0.8
        }
      },
      {
        "at": 10,
        "name": "δ = 0.9",
        "set": {
          "delta": 0.9
        }
      },
      {
        "at": 11,
        "name": "δ = 0.95",
        "set": {
          "delta": 0.95
        }
      },
      {
        "at": 12,
        "name": "δ = 0.99",
        "set": {
          "delta": 0.99
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-memory.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: memory and the future (Lambin's test)",
  "description": "Lambin's key test: memoryless firms (one state, so no punishment is possible) keeping δ = 0.95, beside the paper's appendix reading (memoryless with δ = 0, which its code forces). Measured (release, sessions 1–100, recorded 2026-10-01): Δ 0.862 with one period of memory, 0.958 with none, 0.244 with none and δ = 0. Memoryless firms price higher (1 000 sessions in the survey: 0.958 against 0.851).",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Memory",
    "values": [
      {
        "at": 1,
        "name": "One period, δ = 0.95",
        "set": {}
      },
      {
        "at": 2,
        "name": "None, δ = 0.95",
        "set": {
          "memory": 0
        }
      },
      {
        "at": 3,
        "name": "None, δ = 0 (CCDP-A)",
        "set": {
          "memory": 0,
          "delta": 0.0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-myopic.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: no future to protect",
  "description": "δ = 0: no future, so no reward–punishment scheme can pay. Measured (release, sessions 1–100, recorded 2026-10-01): Δ 0.228 against 0.862 at δ = 0.95 (1 000 sessions in the survey: 0.212 against 0.851).",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "δ",
    "values": [
      {
        "at": 1,
        "name": "δ = 0.95",
        "set": {}
      },
      {
        "at": 2,
        "name": "δ = 0",
        "set": {
          "delta": 0.0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-two-phase.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: explore first, then never (Lambin's Theorem 1)",
  "description": "Lambin's Theorem 1: prices at random for 1 000 periods, then never; the theorem puts both firms at 1.6990 (Δ 0.707) when δ = 0 and at 1.7377 (Δ 0.794) when δ = 0.95. Measured (release, sessions 1–100, recorded 2026-10-01): Δ 0.683 (memory, δ = 0), 0.670 (none, δ = 0), 0.709 (memory, δ = 0.95), 0.924 (none, δ = 0.95). The averages near 0.7 hide spread: the survey finds the theorem's point in 27 %, 13 %, 0.4 % and 3.6 % of sessions.",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Memory and δ",
    "values": [
      {
        "at": 1,
        "name": "One period, δ = 0",
        "set": {
          "exploration": "two_phase",
          "delta": 0.0
        }
      },
      {
        "at": 2,
        "name": "None, δ = 0",
        "set": {
          "exploration": "two_phase",
          "delta": 0.0,
          "memory": 0
        }
      },
      {
        "at": 3,
        "name": "One period, δ = 0.95",
        "set": {
          "exploration": "two_phase"
        }
      },
      {
        "at": 4,
        "name": "None, δ = 0.95",
        "set": {
          "exploration": "two_phase",
          "memory": 0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-every-price.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: deviations down and up",
  "description": "Epivent & Lambin: are price increases punished like cuts? The share of one-period deviations answered by a punishment-like response (the rival a grid step lower the next period, back on the cycle within 25 periods). Measured (release, sessions 1–100, recorded 2026-10-01): 0.917 after the paper's deviation (the static best response), 0.877 after a one-step rise, 0.904 over deviations to every price.",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Deviation",
    "values": [
      {
        "at": 1,
        "name": "To the static best response",
        "set": {}
      },
      {
        "at": 2,
        "name": "Up one step",
        "set": {
          "impulse": "up"
        }
      },
      {
        "at": 3,
        "name": "To every price",
        "set": {
          "impulse": "every_price"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "punishment_like"
  }
}
```

Create `sweeps/collusion-below-nash.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: a grid below the Nash price (Epivent & Lambin)",
  "description": "Epivent & Lambin's App. C grid: 15 prices from 1.25 up to 1.47 (their text) or to the Nash price 1.47293. Measured (release, sessions 1–100, recorded 2026-10-01): Δ 0.862 on CCDP's grid, −0.020 and −0.013 on the low grids — the firms settle at or just under the competitive price. The survey's 10 000 sessions put 72 % (67 %) at the top price, against their 53 %.",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Grid",
    "values": [
      {
        "at": 1,
        "name": "CCDP's",
        "set": {}
      },
      {
        "at": 2,
        "name": "1.25–1.47",
        "set": {
          "grid": "below_nash"
        }
      },
      {
        "at": 3,
        "name": "1.25 to the Nash price",
        "set": {
          "grid": "below_nash",
          "below_top": 1.47293
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-invitation.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: an invitation to raise prices",
  "description": "Epivent & Lambin's invitation: one firm raises its price a step and the rival is made to follow; does the first firm keep the higher price when it decides again? Against a plain one-step rise. Measured (release, sessions 1–100, recorded 2026-10-01): a punishment-like response follows 0.877 of one-step rises; after an invitation the deviator cuts below its old price in 0.815 of the 62 sessions that settled on one price pair. (With one period of memory the hold does not matter: whenever released, the state is both firms at the raised price.)",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Deviation",
    "values": [
      {
        "at": 1,
        "name": "A step up for one period",
        "set": {
          "impulse": "up"
        }
      },
      {
        "at": 2,
        "name": "An invitation (held 5 periods)",
        "set": {
          "impulse": "invitation"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "punishment_like"
  }
}
```

Create `sweeps/collusion-synchronous.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: learning from every price (Asker, Fershtman & Pakes)",
  "description": "Asker, Fershtman & Pakes: each firm updates the value of every price toward what it would have earned against the rival's actual price. Measured (release, sessions 1–100, recorded 2026-10-01): Δ 0.348 against 0.862 (1 000 sessions in the survey: 0.345 against 0.851).",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Update",
    "values": [
      {
        "at": 1,
        "name": "The price charged",
        "set": {}
      },
      {
        "at": 2,
        "name": "Every price",
        "set": {
          "update": "synchronous"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-exploration.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: more exploration",
  "description": "More exploration: ε = e^(−βt) with β ten times smaller, and a constant ε = 0.05 (it never settles, so read at 10⁷ periods). Measured (release, sessions 1–100, recorded 2026-10-01): Δ 0.862, 0.741 and 0.559 — lower, but neither halves it.",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Exploration",
    "values": [
      {
        "at": 1,
        "name": "ε = e^(−βt), β = 4 × 10⁻⁶",
        "set": {}
      },
      {
        "at": 2,
        "name": "β = 4 × 10⁻⁷",
        "set": {
          "beta": 4e-07
        }
      },
      {
        "at": 3,
        "name": "ε = 0.05 (stopped at 10⁷)",
        "set": {
          "exploration": "constant",
          "epsilon": 0.05,
          "cap": 10000000
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "cycle_gain"
  }
}
```

Create `sweeps/collusion-timescale.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: the first T_δ periods (den Boer et al.)",
  "description": "den Boer, Meylahn & Schinkel: over the first T_δ = 165 periods — the horizon that matters at δ = 0.95 — Q-learning prices like uniform random play. The discounted profit gain Δ̃ of the first 165 periods (one tick). Measured (release, sessions 1–100, recorded 2026-10-01): 0.492 on CCDP's grid (random play: 0.497) and −0.523 on a grid centered on Nash (−0.510).",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Grid",
    "values": [
      {
        "at": 1,
        "name": "CCDP's",
        "set": {}
      },
      {
        "at": 2,
        "name": "Centered on Nash, ξ = 0",
        "set": {
          "grid": "symmetric",
          "xi": 0.0
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 1,
  "metric": {
    "kind": "final",
    "series": "discounted_gain"
  }
}
```

Create `sweeps/collusion-rp-complete.json` with exactly this content:

```json
{
  "name": "Algorithmic Collusion: every deviation punished? (den Boer et al.)",
  "description": "den Boer, Meylahn & Schinkel's RP-completeness: does every one-period deviation, by either firm to any price, draw a punishment-like response? Measured (release, sessions 1–100, recorded 2026-10-01): 9 % of sessions under the paper's readings, 10 % under the code's (the survey: 8.8 % of 1 000).",
  "base": {
    "preset": "collusion-calvano"
  },
  "x": {
    "label": "Readings",
    "values": [
      {
        "at": 1,
        "name": "The paper's",
        "set": {}
      },
      {
        "at": 2,
        "name": "The code's",
        "set": {
          "ties": "random",
          "rng": "calvano",
          "cap": 1250000000,
          "equilibrium_check": "one_shot",
          "best_response_to": "code"
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 100
  },
  "ticks": 100000,
  "metric": {
    "kind": "final",
    "series": "rp_complete"
  }
}
```

- [ ] **Step 2: Register them, and the CLI's and WASM's tests**

The thirteen built-ins after `firms-population`; the CLI's finish reason; the WASM fingerprints. Save this as `/tmp/collusion-task.patch` and apply it with `git apply /tmp/collusion-task.patch` (it touches only `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`):

```diff
diff --git a/crates/sugarscape-cli/src/main.rs b/crates/sugarscape-cli/src/main.rs
index d193105..a280c48 100644
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -240,6 +240,14 @@ fn run_world(args: RunArgs) -> Result<(), Failure> {
             ModelKind::Bali => "its last year",
             ModelKind::Hoard => "its last generation",
             ModelKind::Firms => "its last period",
+            // A session stops when its strategies settle, or at its cap.
+            ModelKind::Collusion => {
+                if world.latest_value("converged") == Some(0.0) {
+                    "its cap"
+                } else {
+                    "it converged"
+                }
+            }
             ModelKind::Retirement => match &config {
                 ModelConfig::Retirement(c)
                     if c.stop_at_norm
diff --git a/crates/sugarscape-cli/tests/cli.rs b/crates/sugarscape-cli/tests/cli.rs
index 400e37e..2bc2032 100644
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -200,6 +200,19 @@ fn presets_and_sweeps_are_listed() {
         "firms-hiring",
         "firms-readings",
         "firms-population",
+        "collusion-table-i",
+        "collusion-alpha-beta",
+        "collusion-delta",
+        "collusion-memory",
+        "collusion-myopic",
+        "collusion-two-phase",
+        "collusion-every-price",
+        "collusion-below-nash",
+        "collusion-invitation",
+        "collusion-synchronous",
+        "collusion-exploration",
+        "collusion-timescale",
+        "collusion-rp-complete",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
@@ -655,6 +668,30 @@ fn a_firms_run_stops_at_its_last_period() {
     assert_eq!(stderr(&out), "finished at tick 7 (its last period)\n");
 }
 
+#[test]
+fn a_collusion_session_stops_when_it_converges() {
+    let dir = scratch("collusion");
+    let config = dir.join("collusion.json");
+    std::fs::write(
+        &config,
+        r#"{"model": "collusion", "beta": 0.0002, "window": 2000}"#,
+    )
+    .unwrap();
+    let out = sugarscape(&[
+        "run",
+        "--config",
+        config.to_str().unwrap(),
+        "--ticks",
+        "5000",
+    ]);
+    assert!(out.status.success(), "{}", stderr(&out));
+    let err = stderr(&out);
+    assert!(
+        err.starts_with("finished at tick ") && err.ends_with(" (it converged)\n"),
+        "{err}"
+    );
+}
+
 #[test]
 fn a_bali_run_stops_at_its_last_year() {
     let dir = scratch("bali");
diff --git a/crates/sugarscape-core/src/sweep.rs b/crates/sugarscape-core/src/sweep.rs
index bc5b97f..7b54295 100644
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -965,7 +965,7 @@ pub struct Builtin {
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 187] = [
+const BUILTINS: [Builtin; 200] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1714,6 +1714,58 @@ const BUILTINS: [Builtin; 187] = [
         id: "firms-population",
         json: include_str!("../../../sweeps/firms-population.json"),
     },
+    Builtin {
+        id: "collusion-table-i",
+        json: include_str!("../../../sweeps/collusion-table-i.json"),
+    },
+    Builtin {
+        id: "collusion-alpha-beta",
+        json: include_str!("../../../sweeps/collusion-alpha-beta.json"),
+    },
+    Builtin {
+        id: "collusion-delta",
+        json: include_str!("../../../sweeps/collusion-delta.json"),
+    },
+    Builtin {
+        id: "collusion-memory",
+        json: include_str!("../../../sweeps/collusion-memory.json"),
+    },
+    Builtin {
+        id: "collusion-myopic",
+        json: include_str!("../../../sweeps/collusion-myopic.json"),
+    },
+    Builtin {
+        id: "collusion-two-phase",
+        json: include_str!("../../../sweeps/collusion-two-phase.json"),
+    },
+    Builtin {
+        id: "collusion-every-price",
+        json: include_str!("../../../sweeps/collusion-every-price.json"),
+    },
+    Builtin {
+        id: "collusion-below-nash",
+        json: include_str!("../../../sweeps/collusion-below-nash.json"),
+    },
+    Builtin {
+        id: "collusion-invitation",
+        json: include_str!("../../../sweeps/collusion-invitation.json"),
+    },
+    Builtin {
+        id: "collusion-synchronous",
+        json: include_str!("../../../sweeps/collusion-synchronous.json"),
+    },
+    Builtin {
+        id: "collusion-exploration",
+        json: include_str!("../../../sweeps/collusion-exploration.json"),
+    },
+    Builtin {
+        id: "collusion-timescale",
+        json: include_str!("../../../sweeps/collusion-timescale.json"),
+    },
+    Builtin {
+        id: "collusion-rp-complete",
+        json: include_str!("../../../sweeps/collusion-rp-complete.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -2723,7 +2775,20 @@ mod tests {
                 "firms-base-pay",
                 "firms-hiring",
                 "firms-readings",
-                "firms-population"
+                "firms-population",
+                "collusion-table-i",
+                "collusion-alpha-beta",
+                "collusion-delta",
+                "collusion-memory",
+                "collusion-myopic",
+                "collusion-two-phase",
+                "collusion-every-price",
+                "collusion-below-nash",
+                "collusion-invitation",
+                "collusion-synchronous",
+                "collusion-exploration",
+                "collusion-timescale",
+                "collusion-rp-complete"
             ]
         );
         for b in builtins() {
diff --git a/crates/sugarscape-wasm/tests/web.rs b/crates/sugarscape-wasm/tests/web.rs
index 868a277..f03b9a7 100644
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -441,7 +441,20 @@ fn builtins_and_series_names_are_listed() {
             "firms-base-pay",
             "firms-hiring",
             "firms-readings",
-            "firms-population"
+            "firms-population",
+            "collusion-table-i",
+            "collusion-alpha-beta",
+            "collusion-delta",
+            "collusion-memory",
+            "collusion-myopic",
+            "collusion-two-phase",
+            "collusion-every-price",
+            "collusion-below-nash",
+            "collusion-invitation",
+            "collusion-synchronous",
+            "collusion-exploration",
+            "collusion-timescale",
+            "collusion-rp-complete"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -1331,6 +1344,45 @@ fn firms_with_beta_drawn_per_firm_match_the_native_fingerprint() {
     assert_eq!(sim.fingerprint(), "0x8d1b43dd0c2b2cdb");
 }
 
+#[wasm_bindgen_test]
+fn collusion_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: the paper's
+    // readings, the authors' RAN2, no memory, two-phase exploration,
+    // synchronous updates and the shifted grid.
+    for (id, fp) in [
+        ("collusion-calvano", "0xbfdc574972d2efcd"),
+        ("collusion-code", "0xa6ab4cce772a8a70"),
+        ("collusion-no-memory", "0x172d003af01b8b94"),
+        ("collusion-two-phase", "0xff7d1eb24dd21e2f"),
+        ("collusion-synchronous", "0xcbb6eed6a17dd495"),
+        ("collusion-below-nash", "0x4b11e4afb4a24a8c"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "collusion");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
+#[wasm_bindgen_test]
+fn collusion_boltzmann_and_three_firms_match_the_native_fingerprints() {
+    // `collusion::world::tests::boltzmann_and_three_firms_reach_their_pinned_fingerprints`
+    // pins the same configs and values natively.
+    let mut b: serde_json::Value = serde_json::from_str(&preset_json("collusion-calvano")).unwrap();
+    b["exploration"] = serde_json::json!("boltzmann");
+    b["temperature"] = serde_json::json!(0.01);
+    b["cooling"] = serde_json::json!(1e-4);
+    let mut sim = Sim::new(&b.to_string(), 1, JsValue::NULL).unwrap();
+    sim.step(500);
+    assert_eq!(sim.fingerprint(), "0xffb749a8db1a5481");
+    let mut t: serde_json::Value = serde_json::from_str(&preset_json("collusion-calvano")).unwrap();
+    t["q_init"] = serde_json::json!("random");
+    t["firms"] = serde_json::json!(3);
+    let mut sim = Sim::new(&t.to_string(), 1, JsValue::NULL).unwrap();
+    sim.step(200);
+    assert_eq!(sim.fingerprint(), "0x0bab95a370db18eb");
+}
+
 #[wasm_bindgen_test]
 fn a_hoard_agent_is_inspected_with_its_traits_stores_and_losses() {
     let mut sim = Sim::new(&preset_json("hoard-threshold"), 1, JsValue::NULL).unwrap();
```

- [ ] **Step 3: Run the tests**

Run: `cargo test --release -p sugarscape-core sweep` and `cargo test --release -p sugarscape-cli`
Expected: all pass (the built-in list includes the thirteen; `a_collusion_session_stops_when_it_converges`).

Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: all pass, including the two new collusion tests (the native fingerprints, the code's RAN2, the portable exp under Boltzmann).

Run: `./target/release/sugarscape sweeps | grep collusion | wc -l`
Expected: `13`.

Run: `cargo fmt --all && cargo +stable clippy --all-targets -- -D warnings`
Expected: no output from clippy.

- [ ] **Commit**

```bash
git add sweeps/collusion-*.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/src/main.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -F - <<'EOF'
Algorithmic collusion: thirteen built-in sweeps, the CLI's finish reason and WASM fingerprints

Claude-Session: https://claude.ai/code/session_01XxEZRQqmFWQaPDVcu7Me1w
EOF
```

### Task 3: The page

**Files:**
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/models.test.ts`, `web/src/engine.ts`, `web/src/engine.test.ts`, `web/src/ui/series-data.ts`, `web/src/ui/series-data.test.ts`, `web/src/ui/inspect-panel.ts`, `web/src/experiments/form.ts`, `web/src/experiments/form.test.ts`, `web/src/compare-presets.ts`, `web/src/compare-presets.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: the WASM `Sim` (Task 2), the presets and schema (Task 1).
- Produces: `CollusionConfig`, `CollusionStats`, `CollusionInspection`, `CollusionOutcome`, `CollusionStateView` (types.ts); `isCollusionView`, `COLOR_MODES.collusion`, `MODEL_OVERLAYS.collusion`, collusion branches in `ticksLeft` and `finishesUnpredictably` (models.ts); `MODEL_CHARTS.collusion` (series-data.ts); `collusionRows` (inspect-panel.ts); the notice (engine.ts); `defaultForm('collusion')`; the Compare entry `collusion-async-vs-sync`.

- [ ] **Step 1: Types, model helpers, charts, Inspect, the notice, Experiments and Compare, with their tests**

Every list that names `firms` gains `collusion`. Save this as `/tmp/collusion-task.patch` and apply it with `git apply /tmp/collusion-task.patch` (it touches only `web/src/types.ts`, `web/src/models.ts`, `web/src/models.test.ts`, `web/src/engine.ts`, `web/src/engine.test.ts`, `web/src/ui/series-data.ts`, `web/src/ui/series-data.test.ts`, `web/src/ui/inspect-panel.ts`, `web/src/experiments/form.ts`, `web/src/experiments/form.test.ts`, `web/src/compare-presets.ts`, `web/src/compare-presets.test.ts`, `web/src/determinism.test.ts`):

```diff
diff --git a/web/src/compare-presets.test.ts b/web/src/compare-presets.test.ts
index 6f8cfe5..b9980ee 100644
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -54,6 +54,11 @@ describe('compare presets', () => {
     expect(ids).toContainEqual(['firms-last-vs-live', 'firms-base', 'firms-live', "Last period's effort vs live effort — The Emergence of Firms (Compare)"]);
   });
 
+  it('pairs learning from the price charged with learning from every price', () => {
+    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toContainEqual(['collusion-async-vs-sync', 'collusion-calvano', 'collusion-synchronous', 'Learning from the price charged vs every price — Algorithmic Collusion (Compare)']);
+  });
+
   it('pairs imitation with the same plans fixed', () => {
     const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
     expect(ids).toContainEqual(['lk-random-vs-fixed', 'lk-random', 'lk-random-fixed', 'Imitating neighbors vs fixed random plans — Balinese Water Temples (Compare)']);
diff --git a/web/src/compare-presets.ts b/web/src/compare-presets.ts
index 6ef0bd5..03ac9e4 100644
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -212,6 +212,12 @@ export const COMPARE_PRESETS: ComparePreset[] = [
     a: 'firms-base',
     b: 'firms-live',
   },
+  {
+    id: 'collusion-async-vs-sync',
+    label: 'Learning from the price charged vs every price — Algorithmic Collusion (Compare)',
+    a: 'collusion-calvano',
+    b: 'collusion-synchronous',
+  },
 ];
 
 /**
diff --git a/web/src/determinism.test.ts b/web/src/determinism.test.ts
index e2f607d..cb257cd 100644
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -19,6 +19,9 @@ import type {
   ZiConfig,
   BaliConfig,
   FirmsConfig,
+  CollusionConfig,
+  CollusionInspection,
+  CollusionStats,
   FirmsInspection,
   FirmsStats,
   BaliInspection,
@@ -813,6 +816,31 @@ describe('the firms model through the engine', () => {
   });
 });
 
+describe('the collusion model through the engine', () => {
+  it('finishes when its strategies settle and inspects a strategy cell and the session', async () => {
+    const r = presets.find((p) => p.id === 'collusion-calvano')!;
+    // Fast exploration decay and a short window: a session in a few seconds.
+    const config = { ...structuredClone(r.config as CollusionConfig), beta: 2e-4, window: 2000 };
+    const e = await Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+    e.setDisplay({ colorMode: 'price' });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    while (!e.finished) await e.advance(5_000);
+    const s = e.latest as CollusionStats;
+    expect([e.finished, ends, s.converged]).toEqual([true, 1, 1]);
+    expect(s.cycle_gain).toBeGreaterThan(-1);
+    // The top-left cell of firm 1's map: its own lowest price against the rival's highest.
+    await e.select(0, 0);
+    const v = e.inspection!.view as CollusionInspection;
+    expect([v.panel, v.firm]).toEqual(['strategy', 0]);
+    expect(v.state!.q).toHaveLength(2);
+    expect(v.outcome!.converged).toBe(true);
+    // The price panel starts 8 pixels right of the second firm's 120-pixel map.
+    await e.select(260, 50);
+    expect((e.inspection!.view as CollusionInspection).panel).toBe('prices');
+  });
+});
+
 describe('the bali model through the engine', () => {
   it('stops after its last year and inspects a subak, and a dam in the water strip', async () => {
     const r = presets.find((p) => p.id === 'lk-random')!;
diff --git a/web/src/engine.test.ts b/web/src/engine.test.ts
index b59cd9a..5767674 100644
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1166,6 +1166,7 @@ describe('Engine with other models', () => {
     expect(finishedNotice({ model: 'ants', stop_at: 2000 } as unknown as ModelConfig, 2000)).toBe('This run has reached its last step (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'thresholds', stop_at: 50 } as unknown as ModelConfig, 50)).toBe('This run has reached its last step (50) — Reset to run it again');
     expect(finishedNotice({ model: 'firms', stop_at: 5000 } as unknown as ModelConfig, 5000)).toBe('This run has reached its last period (5000) — Reset to run it again');
+    expect(finishedNotice({ model: 'collusion' } as unknown as ModelConfig, 1864)).toBe('This session has finished at tick 1864 — Reset to run it again');
     expect(finishedNotice({ model: 'bali', stop_at: 30 } as unknown as ModelConfig, 360)).toBe('This run has reached its last year — Reset to run it again');
     const hoard = { model: 'hoard', days: 100, bouts: 20, generations: 60 } as unknown as ModelConfig;
     expect(finishedNotice(hoard, 120_000)).toBe('This run has reached its last generation (60) — Reset to run it again');
diff --git a/web/src/engine.ts b/web/src/engine.ts
index 76fc8a3..1e453cd 100644
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -57,6 +57,7 @@ export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'farol') return `This run has reached its last round (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'ants' || modelOf(config) === 'thresholds') return `This run has reached its last step (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'firms') return `This run has reached its last period (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'collusion') return `This session has finished at tick ${tick} — Reset to run it again`;
   if (modelOf(config) === 'bali') return `This run has reached its last year — Reset to run it again`;
   if (modelOf(config) === 'hoard') {
     // A run ends at the end of generation `generations`' season, or earlier when every agent died.
diff --git a/web/src/experiments/form.test.ts b/web/src/experiments/form.test.ts
index d18a8a0..3dfc0e9 100644
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -152,6 +152,11 @@ describe('sweeps over other models', () => {
       ticks: 5000,
       metric: { kind: 'final', series: 'mu' },
     });
+    expect(defaultForm('collusion')).toMatchObject({
+      x: { path: 'delta', values: '0:0.9:0.15' },
+      ticks: 100_000,
+      metric: { kind: 'final', series: 'cycle_gain' },
+    });
     expect(defaultForm('bali')).toMatchObject({
       x: { path: 'growth', values: '2:2.4:0.1' },
       ticks: 360,
diff --git a/web/src/experiments/form.ts b/web/src/experiments/form.ts
index 2b5da55..b6ce090 100644
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -108,6 +108,10 @@ export function defaultForm(model: ModelKind = 'sugarscape', config?: ModelConfi
     // The built-in firms-beta's axis: the size exponent against increasing returns (A99 Table 3).
     return { ...form, x: { path: 'beta', values: '1.7:2.1:0.1' }, ticks: 5000, metric: { ...form.metric, kind: 'final', series: 'mu' } };
   }
+  if (model === 'collusion') {
+    // The built-in collusion-delta's axis: the profit gain against the discount factor (CCDP Fig. 6).
+    return { ...form, x: { path: 'delta', values: '0:0.9:0.15' }, ticks: 100_000, metric: { ...form.metric, kind: 'final', series: 'cycle_gain' } };
+  }
   if (model === 'bali') {
     // The built-in bali-imitation-growth's axis: the scored harvest against pest growth.
     return { ...form, x: { path: 'growth', values: '2:2.4:0.1' }, ticks: 360, metric: { ...form.metric, kind: 'final', series: 'scored' } };
diff --git a/web/src/models.test.ts b/web/src/models.test.ts
index bd5abc9..cba1b7f 100644
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -14,6 +14,7 @@ import {
   worldMenu,
   isBaliView,
   isFirmsView,
+  isCollusionView,
   isThresholdsView,
   isPunishmentView,
   isZiView,
@@ -281,6 +282,25 @@ describe('the firms model', () => {
   });
 });
 
+describe('the collusion model', () => {
+  it('is read by its tag, and its inspections by `outcome` and `monopoly`, before the others with a panel', () => {
+    expect(modelOf({ model: 'collusion' } as unknown as ModelConfig)).toBe('collusion');
+    const cell = { x: 1, y: 2, panel: 'strategy', firm: 0, state: null, tick: 5, nash: [1.47293, 1.47293], period: 5000, monopoly: [1.92498, 1.92498], outcome: null, agent: null } as unknown as AnyInspection;
+    const firms = { site: { x: 1, y: 2 }, panel: 'firms', firm: null, member: null, agent: null } as unknown as AnyInspection;
+    expect([cell, firms].map(isCollusionView)).toEqual([true, false]);
+    expect([isFirmsView(cell), isBaliView(cell), isZiView(cell), isPunishmentView(cell), isThresholdsView(cell)]).toEqual([false, false, false, false, false]);
+  });
+
+  it('colors two ways, has no overlays, and finishes when its strategies settle', () => {
+    expect(COLOR_MODES.collusion.map(([m]) => m)).toEqual(['price', 'visits']);
+    expect(MODEL_OVERLAYS.collusion).toEqual([]);
+    // The 10⁹-period cap in ticks of 1 000 periods: one past it is tick 1 000 001.
+    const c = { model: 'collusion', cap: 1_000_000_000, periods_per_tick: 1000 } as unknown as ModelConfig;
+    expect(ticksLeft(c, 999_990)).toBe(11);
+    expect(finishesUnpredictably(c)).toBe(true);
+  });
+});
+
 describe('the bali model', () => {
   it('is read by its tag, and its inspections by `subak` and `dam`, before the others with a panel', () => {
     expect(modelOf({ model: 'bali' } as unknown as ModelConfig)).toBe('bali');
diff --git a/web/src/models.ts b/web/src/models.ts
index 65e33ed..1cddadb 100644
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -7,6 +7,8 @@ import type {
   TippingInspection,
   FirmsConfig,
   FirmsInspection,
+  CollusionConfig,
+  CollusionInspection,
   ZiInspection,
   BaliConfig,
   BaliInspection,
@@ -54,7 +56,7 @@ import type {
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment', 'zi', 'bali', 'line', 'tipping', 'hoard', 'firms'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'norms', 'agreement', 'image', 'farol', 'ants', 'thresholds', 'retirement', 'punishment', 'zi', 'bali', 'line', 'tipping', 'hoard', 'firms', 'collusion'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -85,12 +87,13 @@ export const MODEL_LABELS: Record<ModelKind, string> = {
   tipping: "Schelling's tipping",
   hoard: 'The evolution of hoarding',
   firms: 'The Emergence of Firms',
+  collusion: 'Algorithmic Collusion',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment' || tag === 'zi' || tag === 'bali' || tag === 'line' || tag === 'tipping' || tag === 'hoard' || tag === 'firms'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'norms' || tag === 'agreement' || tag === 'image' || tag === 'farol' || tag === 'ants' || tag === 'thresholds' || tag === 'retirement' || tag === 'punishment' || tag === 'zi' || tag === 'bali' || tag === 'line' || tag === 'tipping' || tag === 'hoard' || tag === 'firms' || tag === 'collusion'
     ? tag
     : 'sugarscape';
 }
@@ -199,6 +202,11 @@ export function isFirmsView(v: AnyInspection): v is FirmsInspection {
   return 'panel' in v && 'firm' in v && 'member' in v;
 }
 
+/** A cell of the collusion frame (a panel, a `state` and the session's `outcome`); check it before the others with a panel. */
+export function isCollusionView(v: AnyInspection): v is CollusionInspection {
+  return 'panel' in v && 'outcome' in v && 'monopoly' in v;
+}
+
 /** A point of Schelling's tipping plane. */
 export function isTippingView(v: AnyInspection): v is TippingInspection {
   return 'red_content' in v && 'blue_content' in v;
@@ -276,6 +284,11 @@ export function ticksLeft(c: ModelConfig, tick: number): number {
     return Math.max(h.generations, Math.max(1, Math.ceil(tick / s))) * s - tick;
   }
   if (modelOf(c) === 'firms' && (c as FirmsConfig).stop_at > 0) return Math.max(0, (c as FirmsConfig).stop_at - tick);
+  // A session ends when its strategies settle, at the latest one period past the cap.
+  if (modelOf(c) === 'collusion') {
+    const k = c as CollusionConfig;
+    return Math.max(0, Math.ceil((k.cap + 1) / k.periods_per_tick) - tick);
+  }
   return Infinity;
 }
 
@@ -298,6 +311,7 @@ export function finishesUnpredictably(c: ModelConfig): boolean {
   if (model === 'opinions') return (c as OpinionsConfig).stop_when_stable;
   if (model === 'agreement') return (c as AgreementConfig).stop_when_stable;
   if (model === 'retirement') return (c as RetirementConfig).stop_at_norm;
+  if (model === 'collusion') return true;
   if (model === 'sugarscape') return (c as Config).culture.rule === 'axelrod' && (c as Config).culture.stop_when_settled === true;
   return model === 'civil' && (c as CivilConfig).variant === 'ethnic' && (c as CivilConfig).stop_at_extinction;
 }
@@ -614,6 +628,11 @@ export const COLOR_MODES: Record<ModelKind, [ColorMode, string][]> = {
     ['effort', 'Effort'],
     ['income', 'Income'],
   ],
+  // Each firm's strategy map: the price it would charge, or how often each state was visited.
+  collusion: [
+    ['price', 'Price'],
+    ['visits', 'Visits'],
+  ],
 };
 
 /** The overlays each model can draw: the sugarscape's networks, the valley's water, settlements and links. */
@@ -645,4 +664,5 @@ export const MODEL_OVERLAYS: Record<ModelKind, Overlay[]> = {
   tipping: [],
   hoard: [],
   firms: [],
+  collusion: [],
 };
diff --git a/web/src/types.ts b/web/src/types.ts
index 86b3cef..ff482fb 100644
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -214,7 +214,7 @@ export interface Config {
 }
 
 /** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment' | 'zi' | 'bali' | 'line' | 'tipping' | 'hoard' | 'firms';
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'norms' | 'agreement' | 'image' | 'farol' | 'ants' | 'thresholds' | 'retirement' | 'punishment' | 'zi' | 'bali' | 'line' | 'tipping' | 'hoard' | 'firms' | 'collusion';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -633,7 +633,7 @@ export interface AgreementConfig {
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig | ZiConfig | BaliConfig | LineConfig | TippingConfig | HoardConfig | FirmsConfig;
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | NormsConfig | AgreementConfig | ImageConfig | FarolConfig | AntsConfig | ThresholdsConfig | RetirementConfig | PunishmentConfig | ZiConfig | BaliConfig | LineConfig | TippingConfig | HoardConfig | FirmsConfig | CollusionConfig;
 
 /**
  * Arthur's El Farol bar and Challet and Zhang's minority game (milestone 23), with Challet, Marsili
@@ -1429,7 +1429,7 @@ export interface AgreementStats {
   stable_at: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats | ZiStats | BaliStats | HoardStats | FirmsStats;
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | NormsStats | AgreementStats | ImageStats | FarolStats | AntsStats | ThresholdsStats | RetirementStats | PunishmentStats | ZiStats | BaliStats | HoardStats | FirmsStats | CollusionStats;
 
 export interface SiteView {
   x: number;
@@ -1693,6 +1693,102 @@ export interface FirmsInspection {
   /** Always null: cells are read where they are. */
   agent: null;
 }
+/**
+ * Calvano, Calzolari, Denicolò & Pastorello's algorithmic collusion: n firms pricing with Q-learning on
+ * a grid of prices until their strategies settle; the critics' tests as switches. One tick is a period.
+ */
+export interface CollusionConfig {
+  model: 'collusion';
+  firms: number;
+  prices: number;
+  grid: 'calvano' | 'symmetric' | 'below_nash';
+  xi: number;
+  below_top: number;
+  cost: number;
+  /** Firm 2's cost, or null for the same as the others. */
+  cost2: number | null;
+  quality: number;
+  outside: number;
+  mu: number;
+  memory: number;
+  alpha: number;
+  beta: number;
+  delta: number;
+  exploration: 'decaying' | 'constant' | 'boltzmann' | 'two_phase';
+  epsilon: number;
+  temperature: number;
+  cooling: number;
+  explore_for: number;
+  update: 'asynchronous' | 'synchronous';
+  q_init: 'calvano' | 'zero' | 'random';
+  q_low: number;
+  q_high: number;
+  ties: 'lowest' | 'random';
+  rng: 'ours' | 'calvano';
+  cap: number;
+  window: number;
+  equilibrium_check: 'best_response' | 'one_shot';
+  impulse: 'best_response_down' | 'every_price' | 'invitation' | 'up';
+  best_response_to: 'path' | 'code';
+  invitation_hold: number;
+  /** Periods a tick runs (charts count ticks). */
+  periods_per_tick: number;
+}
+
+/** A charted period; the session's results are null until it has finished. */
+export interface CollusionStats {
+  tick: number;
+  price_1: number | null;
+  price_2: number | null;
+  profit_gain: number | null;
+  greedy_price: number | null;
+  epsilon: number | null;
+  explored: number | null;
+  greedy_changes: number;
+  stable: number;
+  converged: number | null;
+  cycle_length: number | null;
+  cycle_gain: number | null;
+  window_gain: number | null;
+  discounted_gain: number | null;
+  equilibrium_on_path: number | null;
+  punishment_like: number | null;
+  rp_complete: number | null;
+  periods: number | null;
+}
+
+export interface CollusionStateView { state: number; prices: number[][]; q: number[][]; greedy: number[]; visits: number }
+export interface CollusionOutcome {
+  converged: boolean;
+  periods: number;
+  cycle: { states: number[]; actions: number[][]; profits: number[]; prices: number[] };
+  gains: number[];
+  gain: number;
+  window_gain: number;
+  discounted_gain: number;
+  equilibrium: { on_path: boolean; off_path_share: number; all_share: number };
+  punishment_like: number | null;
+  rp_complete: boolean;
+  stale_greedy: number;
+  fumbling: number | null;
+  touched: number;
+}
+
+/** A cell of the collusion frame: a strategy map's state, the price panel, or the response panel. */
+export interface CollusionInspection {
+  x: number;
+  y: number;
+  panel: 'strategy' | 'prices' | 'response' | null;
+  firm: number | null;
+  state: CollusionStateView | null;
+  tick: number;
+  period: number;
+  nash: number[];
+  monopoly: number[];
+  outcome: CollusionOutcome | null;
+  /** Always null: there are no agents to follow, only firms. */
+  agent: null;
+}
 /** A point of his plane: Red and Blue inside, and whether the most tolerant of each would all be content there. */
 export interface TippingInspection { red_in: number; blue_in: number; red_content: boolean; blue_content: boolean; now: boolean; agent: null }
 /**
@@ -2061,7 +2157,7 @@ export interface AgreementInspection {
   agent: null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection | ZiInspection | BaliInspection | LineInspection | TippingInspection | HoardInspection | FirmsInspection;
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | NormsInspection | AgreementInspection | ImageInspection | FarolInspection | AntsInspection | ThresholdsInspection | RetirementInspection | PunishmentInspection | ZiInspection | BaliInspection | LineInspection | TippingInspection | HoardInspection | FirmsInspection | CollusionInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
@@ -2138,7 +2234,9 @@ export type ColorMode =
   | 'founder'
   | 'theta'
   | 'effort'
-  | 'income';
+  | 'income'
+  | 'price'
+  | 'visits';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
diff --git a/web/src/ui/inspect-panel.ts b/web/src/ui/inspect-panel.ts
index 6e6ca9e..0c12cd7 100644
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -4,7 +4,7 @@ import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
 import { imageRows } from '../image-scoring';
 import { hoardStatusText } from '../hoard';
-import { hasCaches, isHoardView, isFirmsView, isAgreementView, isAntsView, isBaliView, isLineView, isTippingView, isPunishmentView, isZiView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { hasCaches, isHoardView, isFirmsView, isCollusionView, isAgreementView, isAntsView, isBaliView, isLineView, isTippingView, isPunishmentView, isZiView, isRetirementView, isThresholdsView, isFarolView, isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isNormsView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
@@ -21,6 +21,7 @@ import type {
   HoardConfig,
   HoardInspection,
   FirmsInspection,
+  CollusionInspection,
   RetirementInspection,
   ThresholdsInspection,
   FarolInspection,
@@ -311,6 +312,33 @@ export class InspectPanel {
     return rows;
   }
 
+  /** A state of a firm's strategy map (its Q-values), or the session's results once it has finished. */
+  private collusionRows(view: CollusionInspection): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    const rows = [row('Benchmarks', `Nash ${view.nash.map(fmt).join(', ')} · monopoly ${view.monopoly.map(fmt).join(', ')}`)];
+    const st = view.state;
+    if (st) {
+      const last = st.prices.length ? st.prices[0].map(fmt).join(' and ') : 'none (no memory)';
+      rows.push(row('State', `#${st.state} · last prices ${last} · visited ${st.visits} times`));
+      st.q.forEach((q, i) => {
+        const best = Math.max(...q);
+        rows.push(row(`Firm ${i + 1}`, `charges ${fmt(st.greedy[i])} · Q from ${fmt(Math.min(...q))} to ${fmt(best)}`));
+      });
+    }
+    const o = view.outcome;
+    if (!o) {
+      rows.push(row('Session', `period ${view.period}: still learning`));
+      return rows;
+    }
+    rows.push(
+      row('Session', `${o.converged ? 'converged' : 'stopped at the cap'} after ${o.periods} periods · cycle of ${o.cycle.states.length}`),
+      row('Profit gain Δ', `${fmt(o.gain)} (firms ${o.gains.map(fmt).join(', ')}) · last window ${fmt(o.window_gain)} · first T_δ ${fmt(o.discounted_gain)}`),
+      row('Equilibrium', `${o.equilibrium.on_path ? 'on the path' : 'not on the path'} · ${fmt(100 * o.equilibrium.off_path_share)}% of other states`),
+      row('Deviations', `${o.punishment_like === null ? 'none' : `${fmt(100 * o.punishment_like)}%`} answered by a punishment-like response · ${o.rp_complete ? 'every one (RP-complete)' : 'not every one'}`),
+    );
+    return rows;
+  }
+
   /** A point of Schelling's tipping plane: the state, and whether each color's most tolerant would be content there. */
   private tippingRows(view: TippingInspection): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -786,6 +814,8 @@ export class InspectPanel {
             ? this.normsRows(view)
           : isAgreementView(view)
             ? this.agreementRows(view)
+          : isCollusionView(view)
+            ? this.collusionRows(view)
           : isFirmsView(view)
             ? this.firmsRows(view)
           : isTippingView(view)
diff --git a/web/src/ui/series-data.test.ts b/web/src/ui/series-data.test.ts
index ef698d6..d9543b6 100644
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -238,6 +238,13 @@ describe('firms charts', () => {
   });
 });
 
+describe('collusion charts', () => {
+  it('chart prices, the profit gain, learning and settling over ticks', () => {
+    expect(MODEL_CHARTS.collusion.map((c) => c.title)).toEqual(['Prices', 'Profit gain', 'Learning', 'Settling']);
+    expect(timeAxisLabel('collusion')).toBe('Ticks');
+  });
+});
+
 describe('bali charts', () => {
   it('chart harvest, changing plans, water and pests, patches and the temple match over months', () => {
     expect(MODEL_CHARTS.bali.map((c) => c.title)).toEqual(['Harvest', 'Changing plans', 'Water and pests', 'Patches', 'Temple match']);
diff --git a/web/src/ui/series-data.ts b/web/src/ui/series-data.ts
index 75ce260..519c31f 100644
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -748,6 +748,26 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
   ],
   // Minds 7 charts by generation and this season's bouts (HOARD_CHARTS), not over the whole run.
   hoard: [],
+  collusion: [
+    {
+      title: 'Prices',
+      lines: [
+        { key: 'price_1', label: 'Firm 1', color: '--c1' },
+        { key: 'price_2', label: 'Firm 2', color: '--c2' },
+        { key: 'greedy_price', label: 'Greedy price (mean)', color: '--c4' },
+      ],
+    },
+    { title: 'Profit gain', lines: [{ key: 'profit_gain', label: 'Δ this period', color: '--c1' }] },
+    {
+      title: 'Learning',
+      lines: [
+        { key: 'epsilon', label: 'Exploration rate', color: '--c1' },
+        { key: 'explored', label: 'Firms exploring', color: '--c3' },
+        { key: 'greedy_changes', label: 'Strategy changes', color: '--red' },
+      ],
+    },
+    { title: 'Settling', lines: [{ key: 'stable', label: 'Periods unchanged', color: '--c2' }] },
+  ],
   firms: [
     {
       title: 'Firms',
@@ -804,7 +824,7 @@ export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]
  * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'zi' ? 'Shouts' : model === 'bali' ? 'Months' : model === 'line' ? 'Rounds' : model === 'tipping' ? 'Steps' : model === 'hoard' ? 'Bouts' : model === 'firms' ? 'Periods' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
+  return model === 'farol' ? 'Rounds' : model === 'ants' || model === 'thresholds' ? 'Steps' : model === 'retirement' || model === 'punishment' ? 'Periods' : model === 'zi' ? 'Shouts' : model === 'bali' ? 'Months' : model === 'line' ? 'Rounds' : model === 'tipping' ? 'Steps' : model === 'hoard' ? 'Bouts' : model === 'firms' ? 'Periods' : model === 'collusion' ? 'Ticks' : model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' || model === 'agreement' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : model === 'norms' ? 'Generations' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
```

- [ ] **Step 2: Build and test**

Run: `(cd web && npm run wasm && npm run build && npm test)`
Expected: the build succeeds; every test passes, including the collusion ones in `models.test.ts`, `series-data.test.ts`, `engine.test.ts`, `form.test.ts`, `compare-presets.test.ts` and `determinism.test.ts` (a session run to convergence through the engine, Inspect on a strategy cell and on the price panel).

- [ ] **Step 3 (the controller): Look at it in the browser**

Run `(cd web && npm run dev)`, choose **Algorithmic Collusion → collusion-calvano**, set the speed to Max and check: the two strategy maps fill from cool to warm as the firms learn; the price panel shows both prices settling between the green (Nash) and red (monopoly) lines; the session finishes (about 2 × 10⁶ periods) with the notice and the response panel drawn; Inspect on a map cell shows the state and both firms' Q-values, and on the price panel the session's results; the Visits color mode lights the visited states. Then Compare **Learning from the price charged vs every price**.

- [ ] **Commit**

```bash
git add web/src/types.ts web/src/models.ts web/src/models.test.ts web/src/engine.ts web/src/engine.test.ts web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/ui/inspect-panel.ts web/src/experiments/form.ts web/src/experiments/form.test.ts web/src/compare-presets.ts web/src/compare-presets.test.ts web/src/determinism.test.ts
git commit -F - <<'EOF'
Algorithmic collusion on the page: strategy maps, charts, Inspect, Compare and Experiments

Claude-Session: https://claude.ai/code/session_01XxEZRQqmFWQaPDVcu7Me1w
EOF
```

### Task 4: The survey's collusion claims

**Files:**
- Create: `survey/src/claims/collusion.rs`, `survey/src/claims/collusion-figures.json`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: Task 1's public analysis API (`limit_cycle`, `equilibrium`, `best_response_deviation`, `every_deviation`, `rp_complete`, `invitation`, `repair`, `lambin_point`, `policy_values`), `CollusionWorld::{current_state, discounted_gain, horizon}`, `as_coded`, the fixture; the survey's `crate::runner::on_threads` and `crate::claim::{all_of, Claim, Outcome, Source, Verdict}`.
- Produces: twenty claims with ids `collusion.ccdp.*`, `collusion.l24.*`, `collusion.el.*`, `collusion.afp.*`, `collusion.critics.*`, `collusion.emz.*`, `collusion.dbms.*`.

- [ ] **Step 1: The figures read in planning and the claims**

Create `survey/src/claims/collusion-figures.json` with exactly this content:

```json
{
"fig1": [
[
0.025,
0.2,
0.8571
],
[
0.025,
0.4,
0.7923
],
[
0.025,
0.6,
0.7761
],
[
0.025,
0.8,
0.7438
],
[
0.025,
1.0,
0.7276
],
[
0.025,
1.2,
0.7276
],
[
0.025,
1.4,
0.7114
],
[
0.025,
1.6,
0.7114
],
[
0.025,
1.8,
0.7114
],
[
0.025,
2.0,
0.7114
],
[
0.05,
0.2,
0.8895
],
[
0.05,
0.4,
0.8571
],
[
0.05,
0.6,
0.8247
],
[
0.05,
0.8,
0.7923
],
[
0.05,
1.0,
0.7761
],
[
0.05,
1.2,
0.7599
],
[
0.05,
1.4,
0.7438
],
[
0.05,
1.6,
0.7438
],
[
0.05,
1.8,
0.7438
],
[
0.05,
2.0,
0.7276
],
[
0.075,
0.2,
0.8733
],
[
0.075,
0.4,
0.8733
],
[
0.075,
0.6,
0.8409
],
[
0.075,
0.8,
0.8247
],
[
0.075,
1.0,
0.8085
],
[
0.075,
1.2,
0.7923
],
[
0.075,
1.4,
0.7761
],
[
0.075,
1.6,
0.7599
],
[
0.075,
1.8,
0.7599
],
[
0.075,
2.0,
0.7438
],
[
0.1,
0.2,
0.8571
],
[
0.1,
0.4,
0.8733
],
[
0.1,
0.6,
0.8571
],
[
0.1,
0.8,
0.8409
],
[
0.1,
1.0,
0.8247
],
[
0.1,
1.2,
0.8085
],
[
0.1,
1.4,
0.7923
],
[
0.1,
1.6,
0.7923
],
[
0.1,
1.8,
0.7761
],
[
0.1,
2.0,
0.7761
],
[
0.125,
0.2,
0.8409
],
[
0.125,
0.4,
0.8571
],
[
0.125,
0.6,
0.8571
],
[
0.125,
0.8,
0.8409
],
[
0.125,
1.0,
0.8247
],
[
0.125,
1.2,
0.8085
],
[
0.125,
1.4,
0.8085
],
[
0.125,
1.6,
0.7923
],
[
0.125,
1.8,
0.7923
],
[
0.125,
2.0,
0.7761
],
[
0.15,
0.2,
0.8085
],
[
0.15,
0.4,
0.8409
],
[
0.15,
0.6,
0.8571
],
[
0.15,
0.8,
0.8409
],
[
0.15,
1.0,
0.8409
],
[
0.15,
1.2,
0.8247
],
[
0.15,
1.4,
0.8085
],
[
0.15,
1.6,
0.7923
],
[
0.15,
1.8,
0.7923
],
[
0.15,
2.0,
0.7923
],
[
0.175,
0.2,
0.7761
],
[
0.175,
0.4,
0.8409
],
[
0.175,
0.6,
0.8409
],
[
0.175,
0.8,
0.8409
],
[
0.175,
1.0,
0.8409
],
[
0.175,
1.2,
0.8247
],
[
0.175,
1.4,
0.8247
],
[
0.175,
1.6,
0.8085
],
[
0.175,
1.8,
0.7923
],
[
0.175,
2.0,
0.7923
],
[
0.2,
0.2,
0.7599
],
[
0.2,
0.4,
0.8247
],
[
0.2,
0.6,
0.8247
],
[
0.2,
0.8,
0.8409
],
[
0.2,
1.0,
0.8409
],
[
0.2,
1.2,
0.8247
],
[
0.2,
1.4,
0.8247
],
[
0.2,
1.6,
0.8085
],
[
0.2,
1.8,
0.8085
],
[
0.2,
2.0,
0.7923
],
[
0.225,
0.2,
0.7438
],
[
0.225,
0.4,
0.8085
],
[
0.225,
0.6,
0.8247
],
[
0.225,
0.8,
0.8247
],
[
0.225,
1.0,
0.8247
],
[
0.225,
1.2,
0.8247
],
[
0.225,
1.4,
0.8247
],
[
0.225,
1.6,
0.8085
],
[
0.225,
1.8,
0.8085
],
[
0.225,
2.0,
0.7923
],
[
0.25,
0.2,
0.7114
],
[
0.25,
0.4,
0.7761
],
[
0.25,
0.6,
0.8085
],
[
0.25,
0.8,
0.8247
],
[
0.25,
1.0,
0.8247
],
[
0.25,
1.2,
0.8247
],
[
0.25,
1.4,
0.8247
],
[
0.25,
1.6,
0.8085
],
[
0.25,
1.8,
0.8085
],
[
0.25,
2.0,
0.8085
]
],
"fig2": [
[
0.025,
0.2,
0.1604
],
[
0.025,
0.4,
0.0983
],
[
0.025,
0.6,
0.0361
],
[
0.025,
0.8,
0.0361
],
[
0.025,
1.0,
0.0361
],
[
0.025,
1.2,
0.0361
],
[
0.025,
1.4,
0.0361
],
[
0.025,
1.6,
0.0361
],
[
0.025,
1.8,
0.0361
],
[
0.025,
2.0,
0.0361
],
[
0.05,
0.2,
0.2847
],
[
0.05,
0.4,
0.1604
],
[
0.05,
0.6,
0.1604
],
[
0.05,
0.8,
0.0983
],
[
0.05,
1.0,
0.0983
],
[
0.05,
1.2,
0.0983
],
[
0.05,
1.4,
0.0361
],
[
0.05,
1.6,
0.0361
],
[
0.05,
1.8,
0.0361
],
[
0.05,
2.0,
0.0361
],
[
0.075,
0.2,
0.5332
],
[
0.075,
0.4,
0.2225
],
[
0.075,
0.6,
0.1604
],
[
0.075,
0.8,
0.1604
],
[
0.075,
1.0,
0.1604
],
[
0.075,
1.2,
0.0983
],
[
0.075,
1.4,
0.0983
],
[
0.075,
1.6,
0.0983
],
[
0.075,
1.8,
0.0983
],
[
0.075,
2.0,
0.0983
],
[
0.1,
0.2,
0.7196
],
[
0.1,
0.4,
0.3468
],
[
0.1,
0.6,
0.2225
],
[
0.1,
0.8,
0.2225
],
[
0.1,
1.0,
0.1604
],
[
0.1,
1.2,
0.1604
],
[
0.1,
1.4,
0.1604
],
[
0.1,
1.6,
0.0983
],
[
0.1,
1.8,
0.0983
],
[
0.1,
2.0,
0.0983
],
[
0.125,
0.2,
0.7196
],
[
0.125,
0.4,
0.4711
],
[
0.125,
0.6,
0.2847
],
[
0.125,
0.8,
0.2225
],
[
0.125,
1.0,
0.1604
],
[
0.125,
1.2,
0.1604
],
[
0.125,
1.4,
0.1604
],
[
0.125,
1.6,
0.1604
],
[
0.125,
1.8,
0.0983
],
[
0.125,
2.0,
0.0983
],
[
0.15,
0.2,
0.7196
],
[
0.15,
0.4,
0.5332
],
[
0.15,
0.6,
0.3468
],
[
0.15,
0.8,
0.2225
],
[
0.15,
1.0,
0.2225
],
[
0.15,
1.2,
0.1604
],
[
0.15,
1.4,
0.1604
],
[
0.15,
1.6,
0.1604
],
[
0.15,
1.8,
0.1604
],
[
0.15,
2.0,
0.1604
],
[
0.175,
0.2,
0.7196
],
[
0.175,
0.4,
0.5332
],
[
0.175,
0.6,
0.3468
],
[
0.175,
0.8,
0.2847
],
[
0.175,
1.0,
0.2225
],
[
0.175,
1.2,
0.1604
],
[
0.175,
1.4,
0.1604
],
[
0.175,
1.6,
0.1604
],
[
0.175,
1.8,
0.1604
],
[
0.175,
2.0,
0.1604
],
[
0.2,
0.2,
0.7196
],
[
0.2,
0.4,
0.5332
],
[
0.2,
0.6,
0.409
],
[
0.2,
0.8,
0.2847
],
[
0.2,
1.0,
0.2225
],
[
0.2,
1.2,
0.1604
],
[
0.2,
1.4,
0.1604
],
[
0.2,
1.6,
0.1604
],
[
0.2,
1.8,
0.1604
],
[
0.2,
2.0,
0.1604
],
[
0.225,
0.2,
0.7196
],
[
0.225,
0.4,
0.5953
],
[
0.225,
0.6,
0.4711
],
[
0.225,
0.8,
0.3468
],
[
0.225,
1.0,
0.2225
],
[
0.225,
1.2,
0.2225
],
[
0.225,
1.4,
0.2225
],
[
0.225,
1.6,
0.1604
],
[
0.225,
1.8,
0.1604
],
[
0.225,
2.0,
0.1604
],
[
0.25,
0.2,
0.7196
],
[
0.25,
0.4,
0.5953
],
[
0.25,
0.6,
0.4711
],
[
0.25,
0.8,
0.409
],
[
0.25,
1.0,
0.2847
],
[
0.25,
1.2,
0.2225
],
[
0.25,
1.4,
0.2225
],
[
0.25,
1.6,
0.1604
],
[
0.25,
1.8,
0.1604
],
[
0.25,
2.0,
0.1604
]
],
"delta": [
[
0.0,
0.2122
],
[
0.1,
0.2069
],
[
0.2,
0.1781
],
[
0.3,
0.1602
],
[
0.34,
0.1563
],
[
0.4,
0.1728
],
[
0.5,
0.2151
],
[
0.6,
0.2776
],
[
0.7,
0.3722
],
[
0.8,
0.4996
],
[
0.9,
0.7102
],
[
0.95,
0.8487
],
[
0.99,
0.934
]
],
"source": "CCDP 2020 replication package, AER_paper_Rscripts figure_1.pdf, figure_2.pdf (heat maps: each cell's color bin midpoint, bins 0.0162 and 0.0621 wide) and figure_3.pdf (\u0394 against \u03b4, exact path coordinates), read from the vector PDFs on 2026-10-01"
}
```

Create `survey/src/claims/collusion.rs` with exactly this content:

```rust
//! Algorithmic Collusion: Calvano, Calzolari, Denicolò & Pastorello (2020),
//! checked against the authors' own code, and the critics' tests (the spec's
//! A and B tables, docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md).
//! Sessions are memoized per process by config and count; every rule here
//! was fixed in the spec before measuring, except where a claim says so.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use sugarscape_core::collusion::analysis::{
    best_response_deviation, equilibrium, every_deviation, invitation, lambin_point, limit_cycle,
    policy_values, repair, rp_complete, Strategies,
};
use sugarscape_core::collusion::demand::Game;
use sugarscape_core::collusion::learner::Space;
use sugarscape_core::collusion::{
    as_coded, BestResponseTo, CollusionConfig, CollusionWorld, EquilibriumCheck, Exploration, Grid,
    Update,
};

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::on_threads;

const CCDP: &str = "Calvano, Calzolari, Denicolò & Pastorello 2020, AER 110(10)";
const L24: &str = "Lambin 2024, SSRN 4498926";
const EL: &str = "Epivent & Lambin 2024, Economics Letters 237 (SSRN 4227229)";
const DBMS: &str = "den Boer, Meylahn & Schinkel 2026, Amsterdam LSRP 2022-25";
const AFP: &str = "Asker, Fershtman & Pakes 2021, NBER w28535";
const EMZ: &str = "Eschenbaum, Mellgren & Zahn 2022, arXiv 2210.10528";

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

/// What the survey keeps of a session.
#[derive(Clone, Debug)]
struct Run {
    gain: f64,
    converged: bool,
    /// Both firms at one price every period: that price's index.
    point: Option<u8>,
    /// A one-price cycle, symmetric or not.
    single: bool,
    eq_best: bool,
    eq_one_shot: bool,
    /// Over every cycle position and deviating firm, the paper's deviation
    /// (the static best response to the rival's price there, or the code's
    /// impulse-response reading): punishment-like shares; the non-deviator's
    /// relative price change in the next period.
    punished: f64,
    punished_code: f64,
    response: f64,
    response_code: f64,
    /// Table A5's rows (the code's DetailedAnalysis: one per cycle position
    /// and deviating firm, the deviation to the static best response at the
    /// state): (IR, IC, punishment length).
    a5: Vec<(f64, f64, f64)>,
    /// Rival prices: in the deviation period and the 11 after (Fig. 4).
    rival_path: Vec<f64>,
    /// Every one-period deviation up and down: punishment-like shares.
    up: f64,
    down: f64,
    /// Firm 1 deviating from a symmetric point: (own, deviation, firm 2's
    /// relative change).
    table: Vec<(u8, u8, f64)>,
    rp: bool,
    invitation: Option<(u8, u8)>,
    window: f64,
    greedy: Vec<f64>,
    strategies: Strategies,
}

type Key = String;
static MEMO: OnceLock<Mutex<HashMap<Key, Arc<Vec<Run>>>>> = OnceLock::new();

/// `n` sessions (seeds 1..=n) of `c`, each read for the mean greedy price
/// at `at` periods (L24's Fig. 1) and then run to its end.
fn runs(c: &CollusionConfig, n: u64, at: &[u64]) -> Arc<Vec<Run>> {
    let key = format!("{}|{n}|{at:?}", serde_json::to_string(c).unwrap());
    let memo = MEMO.get_or_init(Default::default);
    if let Some(r) = memo.lock().unwrap().get(&key) {
        return r.clone();
    }
    let seeds: Vec<u64> = (1..=n).collect();
    let out = Arc::new(on_threads(&seeds, |seed| session(c, seed, at)));
    memo.lock().unwrap().insert(key, out.clone());
    out
}

fn session(c: &CollusionConfig, seed: u64, at: &[u64]) -> Run {
    let mut w = CollusionWorld::new(c.clone(), seed).unwrap();
    let mut greedy = Vec::new();
    for &t in at {
        while w.period() < t && !w.is_finished() {
            let ticks = (t - w.period()).div_ceil(u64::from(c.periods_per_tick));
            w.run(ticks.min(1_000_000) as u32);
        }
        let s = w.current_state();
        greedy.push(
            (0..w.space().firms)
                .map(|i| w.game().grid[i][usize::from(w.firms()[i].greedy[s])])
                .sum::<f64>()
                / w.space().firms as f64,
        );
    }
    while !w.is_finished() {
        w.run(1_000_000);
    }
    let o = w.outcome().unwrap().clone();
    let (g, sp) = (w.game(), w.space());
    let (st, cy) = (&o.strategies, &o.cycle);
    let n = sp.firms;
    let mut values = Vec::new();
    for i in 0..n {
        values.push(policy_values(g, sp, st, i, c.delta));
    }
    let (mut pun, mut pun_code, mut resp, mut resp_code, mut count) = (0.0, 0.0, 0.0, 0.0, 0.0);
    let mut a5 = Vec::new();
    let mut rival_path = vec![0.0; 12];
    for firm in 0..n {
        let rival = (firm + 1) % n;
        for k in 0..cy.len() {
            let r = best_response_deviation(g, sp, st, cy, k, firm, BestResponseTo::Path);
            let rc = best_response_deviation(g, sp, st, cy, k, firm, BestResponseTo::Code);
            pun += f64::from(u8::from(r.punishment_like()));
            pun_code += f64::from(u8::from(rc.punishment_like()));
            resp += r.change(g, rival);
            resp_code += rc.change(g, rival);
            for (t, slot) in rival_path.iter_mut().enumerate() {
                *slot += g.grid[rival][usize::from(rc.path[t][rival])];
            }
            // Table A5's IC: the deviation's discounted profit (deviate once,
            // then everyone follows the strategies) below the path's; a
            // "deviation" to the price the strategy charges anyway is not one.
            let s0 = cy.states[k];
            let a = &r.path[0];
            let dev = g.profit(a, firm) + c.delta * values[firm][sp.next(s0, a)];
            let ic = r.deviation != r.before[firm] && dev < values[firm][s0];
            a5.push((
                r.change(g, rival),
                f64::from(u8::from(ic)),
                r.settled as f64,
            ));
            count += 1.0;
        }
    }
    for x in rival_path.iter_mut() {
        *x /= count;
    }
    let all = every_deviation(sp, st, cy);
    let (mut up, mut upn, mut down, mut downn) = (0.0, 0.0, 0.0, 0.0);
    let mut table = Vec::new();
    for r in &all {
        let own = r.before[r.firm];
        let p = f64::from(u8::from(r.punishment_like()));
        if r.deviation > own {
            up += p;
            upn += 1.0;
        } else {
            down += p;
            downn += 1.0;
        }
        if cy.is_point() && r.firm == 0 && r.before.iter().all(|&x| x == own) {
            table.push((own, r.deviation, r.change(g, 1)));
        }
    }
    let point = (cy.is_point() && cy.actions[0].iter().all(|&x| x == cy.actions[0][0]))
        .then_some(cy.actions[0][0]);
    Run {
        gain: o.gain,
        converged: o.converged,
        point,
        single: cy.is_point(),
        eq_best: equilibrium(g, sp, st, cy, c.delta, EquilibriumCheck::BestResponse).on_path,
        eq_one_shot: equilibrium(g, sp, st, cy, c.delta, EquilibriumCheck::OneShot).on_path,
        punished: pun / count,
        punished_code: pun_code / count,
        response: resp / count,
        response_code: resp_code / count,
        a5,
        rival_path,
        up: if upn > 0.0 { up / upn } else { f64::NAN },
        down: if downn > 0.0 { down / downn } else { f64::NAN },
        table,
        rp: rp_complete(sp, st, cy),
        invitation: invitation(g, sp, st, cy, 0, c.invitation_hold),
        window: o.window_gain,
        greedy,
        strategies: st.clone(),
    }
}

fn mean(v: impl Iterator<Item = f64>) -> f64 {
    let v: Vec<f64> = v.filter(|x| x.is_finite()).collect();
    v.iter().sum::<f64>() / v.len() as f64
}

fn se(v: impl Iterator<Item = f64>) -> f64 {
    let v: Vec<f64> = v.filter(|x| x.is_finite()).collect();
    let m = v.iter().sum::<f64>() / v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() as f64 - 1.0)).sqrt()
        / (v.len() as f64).sqrt()
}

fn share(r: &[Run], f: impl Fn(&Run) -> bool) -> f64 {
    r.iter().filter(|x| f(x)).count() as f64 / r.len() as f64
}

fn base() -> CollusionConfig {
    CollusionConfig::default()
}

fn code() -> CollusionConfig {
    let mut c = base();
    as_coded(&mut c);
    c
}

fn with(edit: impl FnOnce(&mut CollusionConfig)) -> CollusionConfig {
    let mut c = base();
    edit(&mut c);
    c
}

fn gains(r: &[Run]) -> (f64, f64) {
    (mean(r.iter().map(|x| x.gain)), se(r.iter().map(|x| x.gain)))
}

/// The paper's figures, read from the replication package's vector PDFs
/// (collusion-figures.json): Fig. 1 (Δ) and Fig. 2 (the equilibrium share)
/// at the 10 × 10 subgrid, each cell its color bin's midpoint; Fig. 3, Δ
/// against δ.
#[derive(serde::Deserialize)]
struct Figures {
    fig1: Vec<[f64; 3]>,
    fig2: Vec<[f64; 3]>,
    delta: Vec<[f64; 2]>,
}

fn figures() -> Figures {
    serde_json::from_str(include_str!("collusion-figures.json")).unwrap()
}

/// Mean and largest absolute gap.
fn gaps(pairs: &[(f64, f64)]) -> (f64, f64) {
    let g: Vec<f64> = pairs.iter().map(|(a, b)| (a - b).abs()).collect();
    (
        mean(g.iter().copied()),
        g.iter().copied().fold(0.0, f64::max),
    )
}

#[derive(serde::Deserialize)]
struct Fixture {
    sessions: Vec<FixtureSession>,
}

#[derive(serde::Deserialize)]
struct FixtureSession {
    session: u64,
    periods: u64,
    strategies: Vec<u8>,
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "collusion.ccdp.table-i",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "Table I (1 000 sessions, α = 0.15, β = 4 × 10⁻⁶): Δ = 0.849 and 50.5 % of sessions in equilibrium on path — under the code's readings, mean Δ within 2 SE of 0.849 and the equilibrium share (the code's test) within 5 points of 50.5 %",
            check: |_| {
                let r = runs(&code(), 1000, &[]);
                let (m, s) = gains(&r);
                let eq = share(&r, |x| x.eq_one_shot);
                let lit = runs(&base(), 1000, &[]);
                let (lm, ls) = gains(&lit);
                outcome(
                    (m - 0.849).abs() <= 2.0 * s && (eq - 0.505).abs() <= 0.05,
                    format!("Δ {m:.4} ± {s:.4}, equilibrium {:.1} %; under the paper's readings Δ {lm:.4} ± {ls:.4}, equilibrium by the code's test {:.1} %", 100.0 * eq, 100.0 * share(&lit, |x| x.eq_one_shot)),
                )
                .with("Seeds 1–1 000 under the code's readings are the authors' own sessions 1–1 000 (collusion.ccdp.docking).")
            },
        },
        Claim {
            id: "collusion.ccdp.docking",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "The authors' code (replication package, built with gfortran): with their RAN2 seeded −session, random ties and their cap, our sessions 1–10 reach the same strategies in the same period — 10 of 10 (all 100 in the fixture reported)",
            check: |_| {
                let f: Fixture = serde_json::from_str(include_str!(
                    "../../../crates/sugarscape-core/tests/fixtures/calvano-sessions.json"
                ))
                .unwrap();
                let seeds: Vec<u64> = f.sessions.iter().map(|s| s.session).collect();
                let same = on_threads(&seeds, |seed| {
                    let s = &f.sessions[(seed - 1) as usize];
                    let mut w = CollusionWorld::new(code(), seed).unwrap();
                    while !w.is_finished() {
                        w.run(1_000_000);
                    }
                    let o = w.outcome().unwrap();
                    let ours: Vec<u8> = (0..225)
                        .flat_map(|st| [o.strategies[0][st], o.strategies[1][st]])
                        .collect();
                    o.periods == s.periods && ours == s.strategies
                });
                let first = same.iter().take(10).filter(|&&x| x).count();
                let all = same.iter().filter(|&&x| x).count();
                outcome(first == 10, format!("{first} of the first 10 identical; {all} of {}", same.len()))
            },
        },
        Claim {
            id: "collusion.ccdp.heat-map",
            item: "collusion-alpha-beta",
            source: Source::Book,
            citation: CCDP,
            text: "Figs. 1–2: Δ over α ∈ [0.025, 0.25] and β ∈ [0.02, 2] × 10⁻⁵ — on a 10 × 10 subgrid, 100 sessions a cell under the paper's readings, the mean absolute gap to the figure's cells at most 0.03 and the worst at most 0.08; Fig. 2's equilibrium share (the code's test) reported",
            check: |_| {
                let f = figures();
                let mut pairs = Vec::new();
                let mut eq = Vec::new();
                for (k, cell) in f.fig1.iter().enumerate() {
                    let c = with(|c| {
                        c.alpha = cell[0];
                        c.beta = cell[1] * 1e-5;
                    });
                    let r = runs(&c, 100, &[]);
                    pairs.push((gains(&r).0, cell[2]));
                    eq.push((share(&r, |x| x.eq_one_shot), f.fig2[k][2]));
                }
                let (m, w) = gaps(&pairs);
                let (em, ew) = gaps(&eq);
                outcome(m <= 0.03 && w <= 0.08, format!("Δ: mean gap {m:.4}, worst {w:.4}; equilibrium share: mean gap {em:.3}, worst {ew:.3}"))
            },
        },
        Claim {
            id: "collusion.ccdp.delta",
            item: "collusion-delta",
            source: Source::Book,
            citation: CCDP,
            text: "Fig. 3: Δ against δ falls to a minimum of 0.156 at δ = 0.34 and rises to 0.85 at 0.95 — at 13 values of δ (100 sessions each), the mean absolute gap to the curve at most 0.03 and the worst at most 0.08",
            check: |_| {
                let f = figures();
                let pairs: Vec<(f64, f64)> = f
                    .delta
                    .iter()
                    .map(|p| (gains(&runs(&with(|c| c.delta = p[0]), 100, &[])).0, p[1]))
                    .collect();
                let (m, w) = gaps(&pairs);
                let low = f.delta.iter().zip(&pairs).min_by(|a, b| a.1 .0.total_cmp(&b.1 .0)).unwrap();
                outcome(m <= 0.03 && w <= 0.08, format!("mean gap {m:.4}, worst {w:.4}; our lowest Δ {:.4} at δ = {}", low.1 .0, low.0[0]))
                    .with("The paper's own curve gives Δ = 0.212 at δ = 0, where no reward–punishment scheme can pay (collusion.critics.myopic).")
            },
        },
        Claim {
            id: "collusion.ccdp.impulse",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "Fig. 4: after a one-period deviation to the static best response the rival's price falls from 1.795 to 1.551 in the next period and is back near 1.79 by period 10; Table A5 (n = 2, 1 000 sessions): the rival's mean relative change −0.127, deviations unprofitable in 0.936 ('in more than 95 % of the cases', p. 3282), punishment 5.705 periods — each within 2 SE, under the code's readings (Table A5's rows pooled as its script pools them)",
            check: |_| {
                let r = runs(&code(), 1000, &[]);
                let path = |t: usize| mean(r.iter().map(|x| x.rival_path[t]));
                let path_se = |t: usize| se(r.iter().map(|x| x.rival_path[t]));
                let rows: Vec<(f64, f64, f64)> = r.iter().flat_map(|x| x.a5.iter().copied()).collect();
                let pooled = |f: fn(&(f64, f64, f64)) -> f64| (mean(rows.iter().map(f)), se(rows.iter().map(f)));
                let (ir, ic, len) = (pooled(|x| x.0), pooled(|x| x.1), pooled(|x| x.2));
                let within = |v: (f64, f64), want: f64| (v.0 - want).abs() <= 2.0 * v.1;
                all_of(vec![
                    ("rival at τ = 2 (Fig. 4)".into(), outcome(within((path(1), path_se(1)), 1.551), format!("{:.3} → {:.3} (± {:.3}), {:.3} at τ = 10", path(0), path(1), path_se(1), path(9)))),
                    ("IR".into(), outcome(within(ir, -0.127), format!("{:.4} ± {:.4} over {} rows", ir.0, ir.1, rows.len()))),
                    ("IC".into(), outcome(within(ic, 0.936), format!("{:.4} ± {:.4} (the text: more than 0.95)", ic.0, ic.1))),
                    ("punishment length".into(), outcome(within(len, 5.705), format!("{:.3} ± {:.3} periods", len.0, len.1))),
                ])
                .with("Fig. 4 comes from the code's impulse-response routine, which answers the state numbered by the cycle position (collusion.ccdp.figure-4-reading); Table A5 from its detailed analysis, which answers the state itself. On the authors' first 100 sessions ours equal their Fortran's to 10⁻⁹ (IR −0.12236, IC 0.92628, length 5.58013).")
            },
        },
        Claim {
            id: "collusion.ccdp.equilibrium",
            item: "collusion-calvano",
            source: Source::Book,
            citation: CCDP,
            text: "p. 3278: half the sessions are subgame-perfect-like on path — the paper describes solving for the true Q (eq. 3) and checking best responses; the code checks only one-period deviations. Under the paper's description, at least 40 % of sessions in equilibrium on path (the code's test gives 50.5 %; 40 % is ours)",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let b = share(&r, |x| x.eq_best);
                let o = share(&r, |x| x.eq_one_shot);
                let contained = r.iter().all(|x| !x.eq_best || x.eq_one_shot);
                outcome(b >= 0.4 && contained, format!("{:.1} % a best response on path; {:.1} % pass the one-period test; every best-response session also passes it: {contained}", 100.0 * b, 100.0 * o))
                    .with("Re-optimizing against the rival's learned strategy gains 5–41 % of a firm's value on path (20 sessions measured in planning): the strategies punish one-period deviations but can be exploited over longer ones.")
            },
        },
        Claim {
            id: "collusion.ccdp.figure-4-reading",
            item: "collusion-code",
            source: Source::Book,
            citation: CCDP,
            text: "Fig. 4's deviation is 'the static best response' to the rival's price — but the code's ImpulseResponse.f90 passes the cycle position where it means the state; reported: the share of deviations whose price differs between the two readings (at least 1 % counts as a difference, ours)",
            check: |_| {
                let r = runs(&code(), 1000, &[]);
                let differ = r.iter().filter(|x| (x.response - x.response_code).abs() > 1e-12).count() as f64 / r.len() as f64;
                outcome(differ < 0.01, format!("sessions whose responses differ between the readings: {:.1} %; punishment-like after the paper's deviation {:.3}, the code's {:.3}", 100.0 * differ, mean(r.iter().map(|x| x.punished)), mean(r.iter().map(|x| x.punished_code))))
            },
        },
        Claim {
            id: "collusion.l24.memoryless-higher",
            item: "collusion-no-memory",
            source: Source::Book,
            citation: L24,
            text: "Fig. 1: memoryless algorithms (δ = 0.95) price at least as high as algorithms with one period of memory — the mean greedy price at 1.5 × 10⁶ and 2 × 10⁶ periods, and the converged Δ, each at least as high at k = 0 (difference ≥ −2 SE)",
            check: |_| {
                let at = [1_500_000, 2_000_000];
                let one = runs(&base(), 1000, &at);
                let none = runs(&with(|c| c.memory = 0), 1000, &at);
                let cmp = |f: &dyn Fn(&Run) -> f64| {
                    let (a, b) = (mean(none.iter().map(f)), mean(one.iter().map(f)));
                    let s = (se(none.iter().map(f)).powi(2) + se(one.iter().map(f)).powi(2)).sqrt();
                    (a - b >= -2.0 * s, a, b)
                };
                let (h1, a1, b1) = cmp(&|x| x.greedy[0]);
                let (h2, a2, b2) = cmp(&|x| x.greedy[1]);
                let (h3, a3, b3) = cmp(&|x| x.gain);
                outcome(h1 && h2 && h3, format!("greedy price at 1.5M: {a1:.4} against {b1:.4}; at 2M: {a2:.4} against {b2:.4}; Δ {a3:.4} against {b3:.4} (k = 0 against k = 1)"))
                    .with("L24 Fig. 1: about 1.85 against 1.76 at 2 × 10⁶. No memoryless session is an equilibrium, and none answers any deviation.")
            },
        },
        Claim {
            id: "collusion.critics.memoryless-half",
            item: "collusion-no-memory",
            source: Source::Comment,
            citation: "the spec's B1 (secondary)",
            text: "B1's weaker critique, fixed before reading L24's own criterion: Δ(k = 0) ≥ ½ Δ(k = 1), at δ = 0.95; CCDP-A's memoryless reading (δ = 0) reported",
            check: |_| {
                let one = gains(&runs(&base(), 1000, &[])).0;
                let none = gains(&runs(&with(|c| c.memory = 0), 1000, &[])).0;
                let ccdp = gains(&runs(&with(|c| {
                    c.memory = 0;
                    c.delta = 0.0;
                }), 1000, &[]))
                .0;
                outcome(none >= 0.5 * one, format!("Δ {none:.4} against {one:.4}; memory 0 with δ = 0 (CCDP-A): {ccdp:.4}"))
                    .with("The code sets δ = 0 whenever memory is 0 (globals.f90), so it cannot run L24's test.")
            },
        },
        Claim {
            id: "collusion.critics.myopic",
            item: "collusion-myopic",
            source: Source::Comment,
            citation: "the spec's B2 (Schildknecht 2026's δ = 0)",
            text: "B2: with δ = 0 there is no future to protect, so no reward–punishment scheme can pay; the critique holds if Δ(δ = 0) > 0.1, and Δ(0.95) − Δ(0) is then the part strategies can explain",
            check: |_| {
                let (z, zs) = gains(&runs(&with(|c| c.delta = 0.0), 1000, &[]));
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(z > 0.1, format!("Δ(δ = 0) {z:.4} ± {zs:.4}; Δ(0.95) − Δ(0) = {:.4} ({:.0} % of the baseline)", b - z, 100.0 * (b - z) / b))
            },
        },
        Claim {
            id: "collusion.l24.theorem-1",
            item: "collusion-two-phase",
            source: Source::Book,
            citation: L24,
            text: "Theorem 1: after exploring every price at random (here 1 000 periods), the firms settle at I — 1.6990 at δ = 0 and 1.7377 at δ = 0.95 on CCDP's grid — with or without memory; holds if at least 80 % of sessions end with both firms at I in each of the four cases (80 % is ours)",
            check: |_| {
                let mut parts = Vec::new();
                for (memory, delta) in [(1, 0.0), (0, 0.0), (1, 0.95), (0, 0.95)] {
                    let c = with(|c| {
                        c.exploration = Exploration::TwoPhase;
                        c.memory = memory;
                        c.delta = delta;
                    });
                    let i = lambin_point(&Game::new(&c), delta);
                    let r = runs(&c, 1000, &[]);
                    let at = share(&r, |x| x.point == Some(i));
                    let mut modal: HashMap<u8, usize> = HashMap::new();
                    for x in r.iter() {
                        if let Some(p) = x.point {
                            *modal.entry(p).or_default() += 1;
                        }
                    }
                    let top = modal.iter().max_by_key(|(_, &n)| n).map(|(&p, &n)| (p, n));
                    parts.push((
                        format!("k = {memory}, δ = {delta}"),
                        outcome(at >= 0.8, format!("{:.1} % at I = index {i}; most common point {top:?}; Δ {:.4}", 100.0 * at, gains(&r).0)),
                    ));
                }
                // Reported beside the rule (k = 0, δ = 0.95): his figures' apparent
                // start (zero Q-values), and a hundred times more exploring.
                let extra = |edit: fn(&mut CollusionConfig)| {
                    let c = with(|c| {
                        c.exploration = Exploration::TwoPhase;
                        c.memory = 0;
                        edit(c);
                    });
                    let r = runs(&c, 1000, &[]);
                    (100.0 * share(&r, |x| x.point == Some(8)), gains(&r).0)
                };
                let (z, zg) = extra(|c| c.q_init = sugarscape_core::collusion::QInit::Zero);
                let (l, lg) = extra(|c| c.explore_for = 100_000);
                all_of(parts)
                    .with("Theorem 1 is a mean-field limit (every Q-value at its mean after exploring); with α = 0.15 the Q-values stay noisy.")
                    .with(&format!("k = 0, δ = 0.95 from zero Q-values: {z:.1} % at I, Δ {zg:.4}; exploring 10⁵ periods: {l:.1} % at I, Δ {lg:.4}."))
            },
        },
        Claim {
            id: "collusion.el.table-1",
            item: "collusion-every-price",
            source: Source::Book,
            citation: EL,
            text: "Table 1: price increases are 'also followed by aggressive price wars' — in every (pre-deviation price, upward deviation) cell with at least 30 sessions the non-deviator's mean relative change at τ + 1 is negative, and the mean over upward cells is at least half the mean over downward cells (30 and half are ours); the first rule (punishment-like responses after upward deviations at least half as often as after downward ones) reported",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let mut cells: HashMap<(u8, u8), (f64, usize)> = HashMap::new();
                for x in r.iter() {
                    for &(own, dev, ch) in &x.table {
                        let e = cells.entry((own, dev)).or_default();
                        e.0 += ch;
                        e.1 += 1;
                    }
                }
                let (mut ups, mut downs) = (Vec::new(), Vec::new());
                for (&(own, dev), &(sum, n)) in &cells {
                    if n >= 30 {
                        let m = sum / n as f64;
                        if dev > own { ups.push(m) } else { downs.push(m) }
                    }
                }
                let negative = ups.iter().filter(|&&m| m < 0.0).count();
                let (mu, md) = (mean(ups.iter().copied()), mean(downs.iter().copied()));
                let (pu, pd) = (mean(r.iter().map(|x| x.up)), mean(r.iter().map(|x| x.down)));
                outcome(negative == ups.len() && !ups.is_empty() && mu <= 0.5 * md, format!("{negative} of {} upward cells negative; mean change after increases {mu:.4}, after cuts {md:.4}; punishment-like after increases {pu:.3}, after cuts {pd:.3}", ups.len()))
            },
        },
        Claim {
            id: "collusion.el.invitation",
            item: "collusion-invitation",
            source: Source::Book,
            citation: EL,
            text: "Fig. 2: an 'invitation to collude' — one firm a step up, the rival made to follow — is answered by a price cut: the deviator's mean price when it regains control is at least one grid step below its price before",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let v: Vec<(u8, u8)> = r.iter().filter_map(|x| x.invitation).collect();
                let drop = mean(v.iter().map(|&(b, a)| f64::from(a) - f64::from(b)));
                outcome(drop >= 1.0, format!("{} point sessions; mean drop {drop:.2} grid steps; {:.0} % cut", v.len(), 100.0 * v.iter().filter(|(b, a)| b < a).count() as f64 / v.len() as f64))
            },
        },
        Claim {
            id: "collusion.el.below-nash",
            item: "collusion-below-nash",
            source: Source::Book,
            citation: EL,
            text: "App. C (15 prices from 1.25 to 1.47, 10 000 sessions): 53 % converge to the top price, 9 % to cycles, 38 % to points below it (reported); in the below-top points, upward deviations draw punishment-like responses at least half as often as downward ones",
            check: |_| {
                let mut parts = Vec::new();
                for (label, top) in [("1.47", 1.47), ("p^N = 1.47293", 1.47293)] {
                    let c = with(|c| {
                        c.grid = Grid::BelowNash;
                        c.below_top = top;
                    });
                    let r = runs(&c, 10_000, &[]);
                    let at_top = share(&r, |x| x.point == Some(14));
                    let cycles = share(&r, |x| !x.single);
                    let below: Vec<&Run> = r.iter().filter(|x| x.single && x.point != Some(14)).collect();
                    let (u, d) = (mean(below.iter().map(|x| x.up)), mean(below.iter().map(|x| x.down)));
                    parts.push((format!("top {label}"), outcome(u >= 0.5 * d, format!("top price {:.1} %, cycles {:.1} %, other points {:.1} %; there punishment-like after increases {u:.3}, after cuts {d:.3}", 100.0 * at_top, 100.0 * cycles, 100.0 * below.len() as f64 / r.len() as f64))));
                }
                all_of(parts)
            },
        },
        Claim {
            id: "collusion.afp.synchronous",
            item: "collusion-synchronous",
            source: Source::Book,
            citation: AFP,
            text: "Updating every price toward what it would have earned (synchronous learning) removes the supra-competitive prices: Δ falls by more than half",
            check: |_| {
                let s = gains(&runs(&with(|c| c.update = Update::Synchronous), 1000, &[]));
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(s.0 < 0.5 * b, format!("Δ {:.4} ± {:.4} against {b:.4}", s.0, s.1))
            },
        },
        Claim {
            id: "collusion.critics.slower-decay",
            item: "collusion-explore-more",
            source: Source::Comment,
            citation: "the spec's B5 (Abada & Lambin 2023; Calvano et al. 2023)",
            text: "B5: with ten times slower exploration decay (β = 4 × 10⁻⁷) Δ falls by more than half (200 sessions: a session takes about ten times longer)",
            check: |_| {
                let s = gains(&runs(&with(|c| c.beta = 4e-7), 200, &[]));
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(s.0 < 0.5 * b, format!("Δ {:.4} ± {:.4} against {b:.4}", s.0, s.1))
            },
        },
        Claim {
            id: "collusion.critics.constant-exploration",
            item: "collusion-explore-more",
            source: Source::Comment,
            citation: "the spec's B5",
            text: "B5: with a constant ε = 0.05 Δ falls by more than half — read at 10⁷ periods, 100 sessions (set after measuring: no session settled in 10⁸ periods, so the 10⁹ cap would take hours)",
            check: |_| {
                let c = with(|c| {
                    c.exploration = Exploration::Constant;
                    c.epsilon = 0.05;
                    c.cap = 10_000_000;
                });
                let r = runs(&c, 100, &[]);
                let s = gains(&r);
                let b = gains(&runs(&base(), 1000, &[])).0;
                outcome(s.0 < 0.5 * b, format!("Δ {:.4} ± {:.4} against {b:.4}; {:.0} % converged; realized Δ over the last window {:.4}", s.0, s.1, 100.0 * share(&r, |x| x.converged), mean(r.iter().map(|x| x.window))))
            },
        },
        Claim {
            id: "collusion.emz.repair",
            item: "collusion-calvano",
            source: Source::Book,
            citation: EMZ,
            text: "Collusion does not transfer: firm 1 trained in session s against firm 2 trained in session s + 1 (greedy play from a fixed state) earns less than half the original pairs' Δ",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let c = base();
                let (g, sp) = (Game::new(&c), Space::of(&c));
                let n = r.len();
                let start = |i: usize| (i * 7919) % sp.states;
                let cross = mean((0..n).map(|i| repair(&g, &sp, &r[i].strategies, &r[(i + 1) % n].strategies, start(i)).gain(&g)));
                let own = mean((0..n).map(|i| limit_cycle(&g, &sp, &r[i].strategies, start(i)).gain(&g)));
                outcome(cross < 0.5 * own, format!("cross pairs Δ {cross:.4}; the same pairs from the same states {own:.4}"))
            },
        },
        Claim {
            id: "collusion.dbms.timescale",
            item: "collusion-calvano",
            source: Source::Book,
            citation: DBMS,
            text: "§3: within the effective horizon T_δ = 165 periods, Q-learning prices like uniform random play: Δ̃ within 2 SE of 0.497 (and of −0.510 on the grid centered on Nash, ξ = 0)",
            check: |_| {
                let mut parts = Vec::new();
                for (label, want, c) in [
                    ("CCDP's grid", 0.497, base()),
                    ("centered on Nash", -0.510, with(|c| {
                        c.grid = Grid::Symmetric;
                        c.xi = 0.0;
                    })),
                ] {
                    let seeds: Vec<u64> = (1..=1000).collect();
                    let d = on_threads(&seeds, |seed| {
                        let mut w = CollusionWorld::new(c.clone(), seed).unwrap();
                        while w.period() < u64::from(w.horizon()) {
                            w.run(1);
                        }
                        w.discounted_gain()
                    });
                    let (m, s) = (mean(d.iter().copied()), se(d.iter().copied()));
                    parts.push((label.to_string(), outcome((m - want).abs() <= 2.0 * s, format!("Δ̃ {m:.4} ± {s:.4}"))));
                }
                all_of(parts)
            },
        },
        Claim {
            id: "collusion.dbms.rp-complete",
            item: "collusion-calvano",
            source: Source::Book,
            citation: DBMS,
            text: "§5: the pattern is not a scheme — among sessions where the paper's deviation draws a punishment-like response, at least a quarter are not RP-complete (some one-period deviation goes unpunished; a quarter is ours)",
            check: |_| {
                let r = runs(&base(), 1000, &[]);
                let pass: Vec<&Run> = r.iter().filter(|x| x.punished > 0.0).collect();
                let not = pass.iter().filter(|x| !x.rp).count();
                outcome(not as f64 >= 0.25 * pass.len() as f64, format!("{} sessions pass CCDP's test; {not} of them not RP-complete; {:.1} % of all sessions RP-complete", pass.len(), 100.0 * share(&r, |x| x.rp)))
            },
        },
    ]
}
```

Register the module. Save this as `/tmp/collusion-task.patch` and apply it with `git apply /tmp/collusion-task.patch` (it touches only `survey/src/claims/mod.rs`):

```diff
diff --git a/survey/src/claims/mod.rs b/survey/src/claims/mod.rs
index a899954..8839290 100644
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -7,6 +7,7 @@ mod ch4;
 mod ch5;
 mod ch6;
 mod civil;
+mod collusion;
 mod classes;
 mod culture;
 mod dpd;
@@ -53,6 +54,7 @@ pub fn all() -> Vec<Claim> {
         ethno::claims(),
         farol::claims(),
         firms::claims(),
+        collusion::claims(),
         norms::claims(),
         image::claims(),
         minds1::claims(),
```

- [ ] **Step 2: Run the survey**

Run: `(cd survey && cargo build --release && ./target/release/survey --only collusion.ccdp.docking)`
Expected: `| collusion.ccdp.docking | Book | Holds | 10 of the first 10 identical; 100 of 100 |` (about 10 s).

Run: `(cd survey && time ./target/release/survey --only collusion)`
Expected (about 50 minutes on 10 cores; the below-Nash claim's 20 000 sessions are half of it): 20 claims, 15 Holds and 5 Fails —

| id | verdict | measured (the decisive numbers) |
|---|---|---|
| `collusion.ccdp.table-i` | Holds | Δ 0.8487 ± 0.0035, equilibrium 50.5 %; the paper's readings Δ 0.8508 |
| `collusion.ccdp.docking` | Holds | 10 of 10; 100 of 100 |
| `collusion.ccdp.heat-map` | Holds | mean gap 0.0077, worst 0.0395 |
| `collusion.ccdp.delta` | Holds | mean gap 0.0106, worst 0.0248 |
| `collusion.ccdp.impulse` | Holds | rival 1.791 → 1.547; IR −0.1271, IC 0.9364, length 5.705 |
| `collusion.ccdp.equilibrium` | Fails | 0.2 % a best response on path; 49.7 % pass the one-period test |
| `collusion.ccdp.figure-4-reading` | Fails | responses differ in 94.0 % of sessions |
| `collusion.l24.memoryless-higher` | Holds | greedy price 1.8622 against 1.7914 at 2 × 10⁶; Δ 0.9578 against 0.8508 |
| `collusion.critics.memoryless-half` | Holds | Δ 0.9578 against 0.8508; δ = 0 reading 0.2546 |
| `collusion.critics.myopic` | Holds | Δ(δ = 0) 0.2120 |
| `collusion.l24.theorem-1` | Fails | 27.0 %, 12.9 %, 0.4 %, 3.6 % at I |
| `collusion.el.table-1` | Holds | 25 of 25 upward cells negative; −0.0993 against −0.1297 |
| `collusion.el.invitation` | Holds | 617 sessions, mean drop 5.02 steps |
| `collusion.el.below-nash` | Holds | top price 72.1 % / 67.4 %; increases 0.962 against cuts 0.848 |
| `collusion.afp.synchronous` | Holds | Δ 0.3454 |
| `collusion.critics.slower-decay` | Fails | Δ 0.7268 |
| `collusion.critics.constant-exploration` | Fails | Δ 0.5593, 0 % converged |
| `collusion.emz.repair` | Holds | cross pairs 0.1249 against 0.8509 |
| `collusion.dbms.timescale` | Holds | Δ̃ 0.4978; −0.5092 centered on Nash |
| `collusion.dbms.rp-complete` | Holds | 911 of 999 not RP-complete |

Run: `(cd survey && rustfmt --edition 2021 src/claims/collusion.rs)` (format only this file: the crate's others are not rustfmt-clean; do not commit `survey/out/results-*.json`).

- [ ] **Commit**

```bash
git add survey/src/claims/collusion.rs survey/src/claims/collusion-figures.json survey/src/claims/mod.rs
git commit -F - <<'EOF'
Algorithmic collusion: the survey's A and B claims

Claude-Session: https://claude.ai/code/session_01XxEZRQqmFWQaPDVcu7Me1w
EOF
```

### Task 5: README, roadmap, papers index, spec amendments and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md`, `docs/studies/2026-09-27-swarm-coordination.md`

**Interfaces:**
- Consumes: Tasks 1–4's ids and the survey's measured values (Task 4, Step 2).
- Produces: documentation only.

- [ ] **Step 1: README**

Insert this section immediately before the line `## Experiments` (after the firms section):

````markdown
### Algorithmic Collusion (Calvano, Calzolari, Denicolò & Pastorello 2020; and its critics)

**The model.** Can pricing algorithms learn to collude without being told to and without talking?
Two firms sell differentiated goods (logit demand) and each picks one of 15 prices a period, a grid
from a little below the competitive (Bertrand–Nash) price to a little above the joint-monopoly
price. Each firm learns by Q-learning on the state "both firms' prices last period": it keeps a
value for every price in every state, updates the value of the price it charged toward the profit
it earned plus the discounted value of the best price next period, and picks the best price —
except, with a probability that falls over time (ε = e^(−βt)), a price at random. A session ends
when neither firm's strategy has changed for 100 000 periods. The paper reports that the firms
settle far above the competitive price (profit gain Δ = 0.849 of the way from the Nash profit to
the monopoly profit) and punish a rival's price cut, then return: a reward–punishment scheme
learned from scratch. A tick is 1 000 periods; a session takes about 1 800 ticks.

**How the paper was read, and its code.** The paper comes with its authors' Fortran (MIT). Built
here with gfortran, it reproduces Table I; and under the code's own readings — ties broken at
random, its RAN2 generator seeded by session number (the seed), its cap, its tests — this model
reproduces the code's sessions exactly: the same strategies in the same period, 100 of 100. Where
the paper's text and its code differ, the text is the default and the code's choice a switch
(`collusion-code` turns them all on): ties to the lowest price; a cap of 10⁹ periods, not
1.25 × 10⁹; an equilibrium is a best response against the rival's strategy (the paper's
description) rather than "no one-period deviation pays" (the code's test); and the deviation in
Fig. 4 answers the rival's price at the state itself — the code's impulse-response routine passes
the cycle position where it means the state. The critics' tests are switches too: no memory
(Lambin), no future (δ = 0), synchronous updates of every price (Asker, Fershtman & Pakes),
exploration that decays more slowly, stays constant, or stops after a random phase (Lambin), a grid
below the Nash price and deviations up as well as down (Epivent & Lambin), RP-completeness and the
first periods' discounted gain (den Boer, Meylahn & Schinkel), and firms re-paired across sessions
(Eschenbaum, Mellgren & Zahn).

Measured (the survey — 20 claims, 15 hold and 5 fail — and the presets' and sweeps' descriptions):

- **The paper reproduces, exactly.** Under the code's readings, sessions 1–1 000 are the authors'
  own: Table I to the digit (Δ 0.849, 50.5 % in equilibrium on path; one-price cycles 64.3 %,
  two 23.8 %, longer 11.9 %) and Table A5's responses to a deviation (the rival's price −12.7 %,
  deviations unprofitable 93.6 %, punishment 5.7 periods). The paper's own readings give the same
  Δ (0.851). The α × β heat map (mean gap 0.008) and Δ against δ (minimum near δ = 0.34) match.
- **The text overstates its tables.** "In more than 95 % of the cases the punishment makes the
  deviation unprofitable" is 93.6 % in its own Table A5. And the equilibrium the paper describes —
  each firm's strategy a best response to the other's — holds on the path in 0.2 % of sessions;
  the code's test, which checks only one-period deviations, passes 49.7 %. A firm that re-optimizes
  against its rival's learned strategy gains 5–41 % of its value: the strategies punish a single
  deviation but can be exploited by longer ones.
- **High prices without strategies.** Firms with no memory, which cannot punish anything, price
  higher still (Δ 0.958 against 0.851; Lambin's test holds — the authors' code cannot run it, as
  it sets δ = 0 whenever memory is 0). Firms that value only the present (δ = 0) reach Δ 0.212, as
  the paper's own Fig. 3 shows: a quarter of the baseline needs no future at all.
- **The "punishment" is not specific to cuts.** A rival's price *increase* is answered by a price
  cut too, in all 25 cells Epivent & Lambin's Table 1 covers (9.9 % against 13.0 % after a cut); an
  invitation to raise prices is met by a cut in 84 % of sessions; in 91 % of the sessions that
  punish a cut, some other deviation goes unpunished (den Boer et al.'s RP-completeness).
- **It does not transfer, and it does not start early.** A firm paired with a rival trained in
  another session earns Δ 0.125, not 0.851. Over the first 165 periods — the horizon that matters
  at δ = 0.95 — the algorithms earn exactly what uniform random pricing earns (Δ̃ 0.498 against
  0.497).
- **How firms learn matters most.** Updating every price toward what it would have earned
  (synchronous learning) halves Δ (0.345). Exploration decaying ten times more slowly barely lowers
  it (0.727); a constant ε = 0.05 never settles.
- **Lambin's Theorem 1 does not predict the simulations.** After a random phase and none after,
  the theorem says the firms settle at 1.6990 (δ = 0) or 1.7377 (δ = 0.95); at the paper's α they
  do in 27 %, 13 %, 0.4 % and 3.6 % of sessions across memory and δ — the theorem is a mean-field
  limit and the Q-values stay noisy.

Presets: `collusion-calvano`, `collusion-code`, `collusion-no-memory`, `collusion-myopic`,
`collusion-two-phase`, `collusion-synchronous`, `collusion-explore-more`, `collusion-every-price`,
`collusion-invitation`, `collusion-below-nash`. Sweeps: `collusion-table-i`,
`collusion-alpha-beta`, `collusion-delta`, `collusion-memory`, `collusion-myopic`,
`collusion-two-phase`, `collusion-every-price`, `collusion-below-nash`, `collusion-invitation`,
`collusion-synchronous`, `collusion-exploration`, `collusion-timescale`, `collusion-rp-complete`.
Compare: **Learning from the price charged vs every price**. The view: each firm's strategy as a
map (its own last price across, the rival's down, the price it would charge as the color — or how
often each state was visited), the last 240 periods' prices between the Nash (green) and monopoly
(red) lines, and, once the session has finished, the response to a deviation. Inspect a map cell
for both firms' Q-values there, and any cell for the session's results.
See `docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md`.
````

If Task 4's survey printed a different number anywhere (it should not: the sessions are deterministic), use the survey's.

- [ ] **Step 2: Roadmap**

In `docs/roadmap.md`, after the "Milestone 33: The Emergence of Firms (done)" section and before `## Experiments and science`, add the section below. Its number is the next milestone number free on `main` when merging — 34 unless another branch has taken it; use the same number in Step 3 and Step 5.

```markdown
## Milestone 34: Algorithmic Collusion (done)

Calvano, Calzolari, Denicolò and Pastorello's Q-learning pricing algorithms (AER 2020) as a model
kind, checked against the authors' own Fortran: under the code's readings the sessions are theirs,
period for period, and Table I and Table A5 reproduce to the digit. The paper's text overstates its
tables (deviations unprofitable 93.6 %, not "more than 95 %"), and its equilibrium — a best response
— holds in 0.2 % of sessions where the code's one-period test passes half. The critics' tests are
switches, and most hold: memoryless firms price higher, myopic firms reach a quarter of the profit
gain, price increases draw the same "punishments" as cuts, synchronous learning halves the gain,
collusion does not survive a new rival, and the first 165 periods look like random pricing. Slower
exploration and Lambin's Theorem 1 do not hold up.
See `docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md`.
```

and in the list under `## Experiments and science`, after the firms line:

```markdown
- **Calvano, Calzolari, Denicolò and Pastorello's algorithmic collusion** (and its critics: Asker, Fershtman & Pakes; Lambin; Epivent & Lambin; den Boer, Meylahn & Schinkel; Eschenbaum, Mellgren & Zahn): done (Milestone 34).
```

- [ ] **Step 3: Papers index**

In `docs/papers.md`:

1. Append to the Reproduced table, after milestone 33's row:

```markdown
| 34 | `collusion` | `ai-coordination/calvano-calzolari-denicolo-pastorello-2020-aer-ai-algorithmic-pricing-and-collusion.pdf` with its online appendix and replication package (MIT; the Fortran built here as the reference); the critics in `ai-coordination/`: Asker, Fershtman & Pakes 2021–22, Lambin 2024, Epivent & Lambin 2023, den Boer, Meylahn & Schinkel 2026, Waltman & Kaymak 2006–08 | the code's sessions reproduced period for period (100 of 100), Table I and Table A5 to the digit; the text's "more than 95 %" is 93.6 %; the described equilibrium holds in 0.2 % of sessions (the code's one-period test: 49.7 %); Fig. 4's deviation answers the wrong state; memoryless firms price higher (Δ 0.958), δ = 0 gives 0.212, increases are "punished" like cuts, synchronous learning halves Δ, re-paired firms earn 0.125; slower exploration and Lambin's Theorem 1 fail |
```

2. Remove the Queue's row 1 (Algorithmic collusion) and renumber the remaining rows from 1.
3. In Wanted, under **AI coordination**, after "Abada & Lambin (2023), *Management Science*, doi:10.1287/mnsc.2022.4623;" insert: " Calvano, Calzolari, Denicolò & Pastorello (2023), 'Algorithmic collusion: genuine or spurious?' (*IJIO* 90); Klein's (2021) and Calvano et al.'s (2021) simulation code, available from the authors on request;".

- [ ] **Step 4: The spec's amendments**

Append to `docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md`:

```markdown
## Amendments (implementation planning)

The plan (`docs/superpowers/plans/2026-10-01-algorithmic-collusion.md`, Decisions) changed or filled in:

- **A tick is `periods_per_tick` periods** (1 000), not one: the page stops a run at 10⁶ ticks and sweeps at 10⁵, and a session needs about 1.8 × 10⁶ periods. Each charted point sums one tick; `sample_every` is gone.
- **The seed is the session number** under `rng = calvano`; the `session` field is dropped.
- **Benchmarks are rounded to 5 decimals**, as the authors' inputs are.
- **`q_init` is `calvano`, `zero` or `random`** (AFP's optimistic start is `random` with their 10–20).
- **`grid = symmetric`** is [p^N − (1 + ξ)ζ, p^N + (1 + ξ)ζ].
- **New switch `best_response_to`** (`path` | `code`): the code's impulse-response routine passes the cycle position where it means the state; Fig. 4 uses it, Table A5 does not.
- **Responses carry the code's `ShockLength`** (Table A5's punishment length); **`rp_complete` is a series**; re-pairing has no sweep; sweeps run at most 100 seeds and 10⁵ ticks.
- **A4 is Table A5's statistics**, pooled over its rows as its script pools them (the −0.127, 0.936 and 5.705 the spec quotes are Table A5's).
- **B5 is two claims**, the constant-ε arm read at 10⁷ periods with 100 sessions (set after measuring: no such session settled in 10⁸ periods).
- **A3 compares the paper's readings with the figure's cells**; A6's scorable items are claims (`collusion.ccdp.equilibrium`, `collusion.ccdp.figure-4-reading`), the rest reported in the README.
```

- [ ] **Step 5: The swarm-coordination study**

In `docs/studies/2026-09-27-swarm-coordination.md`, at the end of the **Algorithmic collusion** bullet (after "incident's rich channel."), add: " Reproduced as model kind `collusion` (milestone 34): under the critics' tests much of its 'collusion' needs no memory, no future and no punishment scheme."

- [ ] **Step 6: Full verification**

Run, from the repository root:

```bash
cargo fmt --all --check
cargo +stable clippy --all-targets -- -D warnings
cargo test --release --workspace
cargo test -p sugarscape-core --release --test collusion -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
```

Expected: no format or clippy output; 1 601 passed and 101 ignored (the ignored docking test passes when run); 74 WASM tests; 852 web tests in 53 files.

- [ ] **Step 7 (the controller): The browser pass**

As Task 3's last step, plus: each of the ten presets loads with its title and description; `collusion-code`'s session finishes at the same tick on every reset (the authors' session 1); the Experiments tab's default (Δ against δ) runs.

- [ ] **Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-10-01-algorithmic-collusion-design.md docs/studies/2026-09-27-swarm-coordination.md
git commit -F - <<'EOF'
Algorithmic collusion: README, roadmap, papers index and the spec's amendments

Claude-Session: https://claude.ai/code/session_01XxEZRQqmFWQaPDVcu7Me1w
EOF
```


## Self-review (planning)

- **Spec coverage.** Goal and both questions: Tasks 1 and 4. Constraints: Global Constraints (portability pinned in Task 2). Source summary and the paper against its code: Task 1's modules and `as_coded`; A6 in Task 4's claims and Task 5's README. Measured in planning: Decisions 1. Architecture and Config: Task 1 (amended: ticks of periods, `best_response_to`, `q_init`, the seed as session — Decisions 2–8). Step, Statistics, Views: Task 1's world, stats and view. Presets: Task 1 (ten, with titles). Experiments and CLI: Task 2 (thirteen sweeps; re-pairing in the survey only). Survey A1–A6 and B1–B8: Task 4 (twenty claims; B5 split; Decisions 10–12). Page: Task 3. Testing: Task 1 (demand, Q initialization, RAN2, learner, analysis, docking, determinism) and Task 2 (WASM). Docs: Task 5. "Next in this kind" stays unbuilt.
- **Placeholders.** None: every file is given whole or as an exact diff produced from the verified scratch copy; the milestone number (34) carries the rule for a clash at merge.
- **Type consistency.** The Interfaces blocks list the names each later task uses (`CollusionWorld::{period, step_period, current_state, discounted_gain, horizon}`, `Response::{returned, settled}`, `as_coded`, `lambin_point`); the survey compiles against Task 1's API as given.
- **Review Focus.** Each of the five lines names the tests that pin it, all inside Tasks 1–2 and the survey.
