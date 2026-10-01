# SugarScape Milestone — Algorithmic Collusion — Design

**Date:** 2026-10-01
**Milestone number:** claimed at merge (other sessions claim numbers first).
**Builds on:** the milestone 1–33 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–29, the preset titles of `crates/sugarscape-core/src/titles.rs`, and milestones 27–29's method (one fit rule fixed before measuring; where a source's text and its figures, tables or code disagree, all are reported; decision rules written before measuring).
**Reading notes:** `2026-10-01-collusion-reading-notes.md`.
**Source texts** (local copies in `papers/ai-coordination/`):
- Emilio Calvano, Giacomo Calzolari, Vincenzo Denicolò & Sergio Pastorello, "Artificial Intelligence, Algorithmic Pricing, and Collusion," *AER* 110(10): 3267–97 (2020) (CCDP below), with its online appendix (CCDP-A) and the authors' replication package (Intel Fortran with OpenMP, R scripts, the published outputs; MIT license; from a verbatim GitHub mirror of openICPSR E119462V1).
- The critiques whose tests are switches here: Asker, Fershtman & Pakes (2021 NBER w28535; 2022 *AEA P&P*) (AFP); Waltman & Kaymak (2006 ERIM; 2008 *JEDC*); Epivent & Lambin (2024, *Economics Letters*) and Abada & Lambin (2023, *Management Science*), both known only from abstracts and citing papers until found; Eschenbaum, Mellgren & Zahn (2022); Schildknecht (2026, *JCRE*), the published replication.
- For later milestones in the same kind, not built here: Klein (2021, *RAND*), Calvano et al. (2021, *IJIO*).

## Goal

Calvano et al.'s model as one model kind, `collusion` ("Algorithmic Collusion"), a full citizen of the playground: two firms selling differentiated products under logit demand each set prices with a Q-learning algorithm that remembers last period's prices, explores less and less, and — the paper says — learns to keep prices well above the competitive level, sustained by punishing a rival's price cut. Two questions, in order:

1. **Does it reproduce?** Against the authors' own code, which builds and runs here, and against the paper's figures.
2. **Is it collusion?** The paper's reading is that the algorithms learn reward–punishment schemes. The critiques say much of the high price comes without memory, without a future, or from how exploration and updating work. Each critique becomes a switch on the same engine, with a decision rule fixed before measuring.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited.
- **Faithful where the source is specific**; where the paper's text and the authors' code differ, the text is the default and the code's choice is a named switch (the `calvano-code` preset turns all of them on); every choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only; `exp` via `portable::exp_neg` and its reciprocal; the payoff table and benchmarks computed once at reset).
- **Truthful descriptions and titles**; the paper's numbers are quoted beside ours, never in place of them.
- **The survey's decision rules are written before measuring** (below), and every rule set after seeing a number says so.

## Source summary (CCDP §II, as the code implements it)

- **Demand (eq. 5):** qᵢ = exp((aᵢ − pᵢ)/μ) / (Σⱼ exp((aⱼ − pⱼ)/μ) + exp(a₀/μ)); πᵢ = (pᵢ − cᵢ)qᵢ. Baseline n = 2, cᵢ = 1, aᵢ = 2, a₀ = 0, μ = ¼.
- **Benchmarks:** the one-shot Bertrand–Nash price p^N = 1.47293 and the joint-profit maximum p^M = 1.92498 (the code takes both as inputs, from an unpublished Mathematica script; re-derived in planning: 1.472927 and 1.924981); π^N = 0.22293, π^M = 0.33749 at the continuous prices.
- **Grid:** m = 15 prices equally spaced on [p^N − ξ(p^M − p^N), p^M + ξ(p^M − p^N)], ξ = 0.1, built per agent by cumulative addition (1.42773 … 1.97018).
- **State:** the last k periods' prices of all firms (k = 1: 225 states; Q is 225 × 15 per firm).
- **Update (eq. 4):** Q(s, a) ← (1 − α)Q(s, a) + α[π + δ maxₐ′ Q(s′, a′)], only the visited cell, firms in order; α = 0.15, δ = 0.95.
- **Exploration (eq. 7):** ε_t = e^(−βt), β = 4 × 10⁻⁶; in the code ε starts at 1 and is multiplied by e^(−β) after each use; a firm explores if u ≤ ε and then picks uniformly among all 15 prices.
- **Initialization (eq. 8):** Q₀(s, a) = Σ over the rival's prices of π(a, a₋ᵢ) / ((1 − δ)m), the same in every state (greedy price 5, 1.58271, everywhere); the first state is drawn at random.
- **Convergence:** each firm's greedy strategy unchanged for 100 000 consecutive periods, or a cap.
- **Outcome:** from the converged strategies and the last state, replay without exploration until a state repeats; profit is averaged over that limit cycle; Δ = (π̄ − π^N)/(π^M − π^N) per firm, averaged over firms, then over 1 000 sessions; unconverged sessions are included.
- **Analysis:** an equilibrium check on the limit path and off it; impulse responses to a one-period deviation (the paper's: to the static best response).

## The paper against its code (read in planning)

- **Ties:** the paper breaks ties toward the lowest price; the code at random (lowest-index only in the deviation analysis).
- **Cap:** the paper says 10⁹ periods; the code stops at 50 000 episodes of 25 000, 1.25 × 10⁹.
- **Equilibrium check:** the paper describes solving for the true Q and comparing best responses; the code evaluates the learned strategies' Q (policy evaluation with all firms, including the deviator, returning to their strategies) — a one-shot-deviation test, necessary but not sufficient.
- **Mislabels:** Table I's "Q-loss (all states)" is off-path states only; Figure 7 plots a price ratio, not a percentage cut; Figure 5 uses only symmetric constant-price sessions at the modal price.
- **Text against code and tables:** footnote 20's ν formula gives 0.29 where the text says 4 (Schildknecht says referees flagged it); "14 %" exploration after 100 000 periods is 37 %; ">95 %" unprofitable deviations against the code's 93.6 %; CCDP-A's β = 10⁻⁴ for the memoryless case is 10⁻³ in the code; the ξ = 0.5 robustness run silently changes the Q initialization; the heat maps plot 91 rows of α (α ≥ 0.025 in steps of 0.0025), not "100 points".

## Measured in planning

- **The authors' code builds and runs here.** gfortran 14 compiles the baseline program after porting Intel extensions in a scratch copy: `SORTQQ` (percentile summaries only) replaced by an insertion sort; 88 variable-width formats `<expr>` replaced by their baseline values (2 firms, 15 prices, 225 states); one per-session record rewritten as list-directed writes (it is read back list-directed); `NOT` on logicals as `.NOT.`; static linking for the toolchain. None touches the learning.
- **It reproduces Table I:** 100 sessions of the Table I input in 16 s on 8 cores: mean Δ **0.849** (median 0.868), **50 %** of sessions in equilibrium on path (the paper: 0.849 and 50.5 %), all 100 converged, mean 1.74 × 10⁶ periods to convergence.
- **The seeding is portable:** each session's exploration and tie-break/initialization streams are Numerical Recipes RAN2 (L'Ecuyer with the Bays–Durham shuffle) seeded −(session number); initial prices come from one RAN2 stream seeded −1 shared across sessions; per period the uniforms are drawn in the order u(1, firm 1), u(1, firm 2), u(2, firm 1), u(2, firm 2). A port of RAN2 should reproduce sessions period for period (test A2).
- **Schildknecht (2026)** also reproduces Δ ≈ 0.86 in Python, but reports slower convergence (3.76 × 10⁶), only 62.5 % converged within his 5 × 10⁶ cap, and Δ ≈ 0.2 at δ = 0 — the last is test B2.

## Architecture

Model kind `collusion` ("Algorithmic Collusion"): `ModelKind::Collusion`, `ModelConfig::Collusion(CollusionConfig)` tagged `"model": "collusion"`, a `CollusionWorld` implementing `Model`. Code in `crates/sugarscape-core/src/collusion/`:

- `config.rs` — parameters, reading enums, validation, schema.
- `demand.rs` — logit demand, the payoff table, the benchmarks (p^N by fixed-point iteration on the first-order condition, p^M by maximizing joint profit; tested against the code's inputs to 5 decimals), the price grid.
- `ran2.rs` — the authors' RAN2, for `rng = calvano` only.
- `learner.rs` — one firm: Q-table, greedy strategy kept incrementally as the code does, exploration, update (asynchronous or synchronous).
- `world.rs` — one session: the period loop, convergence, `finished()`.
- `analysis.rs` — limit cycle, Δ, the equilibrium check (both readings), impulse responses (each kind), re-pairing.
- `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`.

One tick is one period. Environments are enums from the start (`demand = logit`; `timing = simultaneous`) so that Klein's sequential moves, the 2021 Cournot game and AFP's homogeneous Bertrand can follow as later milestones without restructuring.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `firms` | 2 | reset | n (2–4; CCDP-A's 3 and 4) |
| `prices` | 15 | reset | m |
| `xi` | 0.1 | reset | the grid's extension beyond p^N and p^M |
| `cost`, `quality`, `outside`, `mu` | 1, 2, 0, 0.25 | reset | cᵢ, aᵢ, a₀, μ (one value for all firms; asymmetric costs as `cost2` for firm 2, default equal) |
| `memory` | 1 | reset | k: 0 (no state — test B1), 1 or 2 |
| `alpha` | 0.15 | live | learning rate |
| `beta` | 4 × 10⁻⁶ | reset | exploration decay per period |
| `delta` | 0.95 | live | discount factor (0 — test B2) |
| `exploration` | `decaying` | reset | `decaying` (ε = e^(−βt)), `constant` (ε fixed at `epsilon`), `boltzmann` (the code's type 2: choice probabilities ∝ exp((Q − max Q)/T), T starting at `temperature` and multiplied by (1 − `cooling`) each period) |
| `epsilon`, `temperature`, `cooling` | 0.05, 1 000, 10⁻⁵ | reset | for `constant` and `boltzmann` (the code starts T at 1 000 and takes the cooling as 10^x from its input; 10⁻⁵ is our default, CCDP-A's values used in its presets) |
| `update` | `asynchronous` | reset | `asynchronous` (only the price charged: CCDP) or `synchronous` (every price, toward the profit it would have earned against the rival's actual price and the state that would have followed: AFP; which next state a counterfactual price leads to is AFP's open point, read as the state with the firm's own price replaced) |
| `q_init` | `calvano` | reset | `calvano` (eq. 8), `zero`, `optimistic` (uniform on [`q_low`, `q_high`], AFP's reading) |
| `ties` | `lowest` | reset | `lowest` (the paper) or `random` (the code) |
| `rng` | `ours` | reset | `ours` (the project's generator) or `calvano` (RAN2 seeded as the code, for docking) |
| `session` | 1 | reset | the session number (the code's seeds are −session) |
| `cap` | 10⁹ | reset | the stop (the code: 1.25 × 10⁹) |
| `window` | 100 000 | reset | periods of unchanged strategies that count as convergence |
| `equilibrium_check` | `best_response` | live | `best_response` (true Q by value iteration against the rival's strategy: the paper) or `one_shot` (the code's policy evaluation) |
| `impulse` | `best_response_down` | live | `best_response_down` (the paper: one period at the static best response), `up` (one period one grid step above the pre-deviation price: Epivent & Lambin's reading, the step is ours), `every_price` (CCDP-A's) |

## Step (one period)

Each firm draws its exploration uniform, then its price uniform (in the code's order when `rng = calvano`); explores with probability ε or plays its greedy price; the market clears; each firm updates its Q-table in firm order (one cell, or every price under `synchronous`) and its greedy strategy at the visited state (with `ties`); ε decays; the convergence counter grows if no firm's greedy price at the visited state changed, else resets to 1. At convergence or the cap the session is finished; the analysis then runs once.

## Statistics

`SERIES`: `price_1`, `price_2` (prices charged), `profit_gain` (Δ of this period's profits), `epsilon`, `stable` (the convergence counter), `explored` (share of firms exploring this period), `greedy_changes` (greedy changes this period). After the session finishes: `cycle_length`, `cycle_gain` (Δ over the limit cycle), `equilibrium_on_path`, `equilibrium_off_path` (shares of states), `converged`, `periods`. Long sessions thin the per-period series for display (every 1 000th period after the first 10 000); the statistics themselves use every period.

## Views

- **Strategies:** for each firm, its greedy strategy as a 15 × 15 grid (rows: the rival's last price; columns: its own; color: the price it would charge, cool for competitive, warm for collusive), the visited state marked.
- **Beside it:** both prices over time on the grid's scale with p^N and p^M as lines; after convergence, the limit cycle and the impulse response (both prices, 25 periods, pre-deviation level dashed).
- **Inspect:** a state (both firms' Q-values for every price, the greedy price, visits) or the session (Δ, cycle, equilibrium flags).
- **Charts:** Prices (`price_1`, `price_2`); Gain (`profit_gain`); Learning (`epsilon`, `explored`, `greedy_changes`). Time axis: Periods.
- **Speed:** the default speed runs a baseline session to convergence (about 2 × 10⁶ periods) in about a minute in the browser.

## Presets

Titles describe what happens (final wording while building, in `titles.rs`):

| Preset | Title (draft) |
|---|---|
| `calvano` | Two pricing algorithms learn to keep prices high |
| `calvano-code` | …as the authors' code ran it |
| `no-memory` | Pricing algorithms that remember nothing (B1) |
| `myopic` | Pricing algorithms that ignore the future (B2) |
| `synchronous` | Algorithms that learn from every price, not only the one they charged (B4) |
| `explore-more` | Pricing algorithms that keep experimenting (B5) |
| `price-war` | One firm undercuts once; the other responds (impulse) |

## Experiments and CLI

Built-in sweeps (`sweep.rs` BUILTINS), each a set of sessions reporting mean, standard error and distribution: `collusion-table-i` (1 000 sessions), `collusion-alpha-beta` (a 10 × 10 subgrid of Figures 1–2, 100 sessions a cell), `collusion-delta` (Figure 6's δ), `collusion-impulse` (Figure 4), and one per B test (`collusion-memory`, `collusion-myopic`, `collusion-upward`, `collusion-synchronous`, `collusion-exploration`, `collusion-repair`). Sessions run in parallel natively.

## Survey

The reference for each A claim is the authors' code where it can run the configuration (built as in "Measured in planning", kept in the scratchpad, never committed into the crates), else the paper's figures, read from the package's vector PDFs.

### A. Does it reproduce?

| # | Claim | Holds if |
|---|---|---|
| A1 | Table I: Δ = 0.849, 50.5 % in equilibrium on path | our mean Δ within 2 SE of the authors' code over 1 000 sessions each, and the equilibrium share within 5 points |
| A2 | Our sessions are theirs | with `rng = calvano`, `ties = random` and the code's cap, 10 of 10 sessions reach the same converged strategies in the same period as the authors' code (if the seeding cannot be matched exactly, A2 is reported as not run, with the reason) |
| A3 | The α × β heat maps (Figures 1–2) | on the 10 × 10 subgrid, 100 sessions a cell: mean absolute gap to the authors' code (or the figure's cells) ≤ 0.03, worst cell ≤ 0.08 |
| A4 | Figure 4's impulse response: the rival from 1.795 to 1.551 in period 2, back near 1.79 by period 10; punishment 5.7 periods on average | each within 2 SE |
| A5 | Δ against δ (Figure 6), minimum 0.156 at δ = 0.34 | as A3 |
| A6 | The paper against its code | reported, not scored: each item of "The paper against its code" |

### B. Is it collusion?

A **punishment-like response** (fixed now): in the period after a one-period deviation, the rival's price is at least one grid step below its pre-deviation price, and both prices return to the pre-deviation cycle within 25 periods.

| # | Test | Supports the critique if |
|---|---|---|
| B1 | memory 0 (no state, so no punishment is possible) | Δ(k = 0) ≥ ½ Δ(k = 1) |
| B2 | δ = 0 (no future to protect) | Δ(δ = 0) > 0.1; Δ(δ) − Δ(0) is then reported as the part strategies can explain |
| B3 | an upward deviation | punishment-like responses after an upward deviation in at least half as many sessions as after the paper's downward one |
| B3b | sessions that settle below p^N (Epivent & Lambin) | punishment-like responses in them at least half as often as in sessions above |
| B4 | synchronous updating (AFP) | Δ falls by more than half |
| B5 | more exploration: `constant` ε = 0.05, and β ten times lower | Δ falls by more than half under either |
| B6 | re-pairing: each firm trained in session s against the rival trained in session s + 1, greedy play from a random state | the cross pairs' Δ < ½ the original pairs' Δ |

Each verdict stays on whichever side of its threshold it lands. The survey's write-up states for each critique what its test found, and, where B2 holds, how much of the paper's Δ remains attributable to strategies.

## Page

`web/` gains the kind's view (strategy grids, price chart, impulse panel), presets with titles, and the A and B results in the description, with the paper's numbers beside ours.

## Testing

- **Demand and grid:** benchmarks to 5 decimals against the code's inputs for n = 2, 3, μ = 0.5 and the asymmetric costs; the 15 prices; π^N and π^M.
- **Q initialization:** the baseline row (5.790 … 6.278 … 4.111) and greedy price 5.
- **RAN2:** the port against the Fortran's first draws for seeds −1 and −7.
- **Learner:** the update on hand-worked cells; incremental greedy maintenance against a full recompute over a long random run; ties under both readings.
- **Docking (A2):** a fixture of a few sessions' converged strategies and periods from the authors' code, checked into the test data (MIT).
- **Analysis:** the limit cycle and Δ on constructed strategies; both equilibrium checks on a strategy pair known to be (and not to be) an equilibrium; impulse responses on a hand-built punishment strategy.
- **Determinism:** golden fingerprints for the presets, native and WASM.

## Docs

`docs/papers.md` (the Reproduced row; the Queue's #1 removed; Wanted gains the four critiques to fetch by hand — den Boer, Meylahn & Schinkel 2026; Epivent & Lambin 2024; Lambin 2024; Calvano et al. 2023 — and Klein's and Calvano 2021's code, available from the authors on request), the README's model list, and the module docs. The swarm-coordination study (`docs/studies/2026-09-27-swarm-coordination.md`) cites the milestone as its no-communication reference.
