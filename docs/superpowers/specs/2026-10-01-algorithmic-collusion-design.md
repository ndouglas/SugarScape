# SugarScape Milestone — Algorithmic Collusion — Design

**Date:** 2026-10-01
**Milestone number:** claimed at merge (other sessions claim numbers first).
**Builds on:** the milestone 1–33 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–29, the preset titles of `crates/sugarscape-core/src/titles.rs`, and milestones 27–29's method (one fit rule fixed before measuring; where a source's text and its figures, tables or code disagree, all are reported; decision rules written before measuring).
**Reading notes:** `2026-10-01-collusion-reading-notes.md`.
**Source texts** (local copies in `papers/ai-coordination/`):
- Emilio Calvano, Giacomo Calzolari, Vincenzo Denicolò & Sergio Pastorello, "Artificial Intelligence, Algorithmic Pricing, and Collusion," *AER* 110(10): 3267–97 (2020) (CCDP below), with its online appendix (CCDP-A) and the authors' replication package (Intel Fortran with OpenMP, R scripts, the published outputs; MIT license; from a verbatim GitHub mirror of openICPSR E119462V1).
- The critiques whose tests are switches here: Asker, Fershtman & Pakes (2021 NBER w28535; 2022 *AEA P&P*) (AFP); Waltman & Kaymak (2006 ERIM; 2008 *JEDC*); Epivent & Lambin (SSRN 4227229, the February 2023 working paper of their 2024 *Economics Letters* article) (E&L); Lambin, "Less than meets the eye" (SSRN 4498926, July 2024) (L24); den Boer, Meylahn & Schinkel, "Artificial Collusion" (Amsterdam Law School RP 2022-25, the revision of 19 February 2026) (dBMS); Abada & Lambin (2023, *Management Science*), known only from its abstract and citing papers; Eschenbaum, Mellgren & Zahn (2022); Schildknecht (2026, *JCRE*), the published replication.
- For later milestones in the same kind, not built here: Klein (2021, *RAND*), Calvano et al. (2021, *IJIO*), and the parts of L24 and dBMS listed under "Next in this kind".

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
| `memory` | 1 | reset | k: 0 (one state — test B1), 1 or 2; `memory = 0` keeps `delta` (L24's key test); CCDP-A's memoryless case is `memory = 0` with `delta = 0` |
| `grid` | `calvano` | reset | `calvano` (ξ beyond p^N and p^M), `symmetric` (dBMS's Ã: ξ = 0), `below_nash` (E&L App. C: 15 prices from 1.25 to `below_top`) |
| `below_top` | 1.47 | reset | the `below_nash` grid's top: 1.47 as E&L's text, or p^N (1.47293); the survey reports which reproduces their shares |
| `alpha` | 0.15 | live | learning rate |
| `beta` | 4 × 10⁻⁶ | reset | exploration decay per period |
| `delta` | 0.95 | live | discount factor (0 — test B2) |
| `exploration` | `decaying` | reset | `decaying` (ε = e^(−βt)), `constant` (ε fixed at `epsilon`), `boltzmann` (the code's type 2: choice probabilities ∝ exp((Q − max Q)/T), T starting at `temperature` and multiplied by (1 − `cooling`) each period), `two_phase` (L24: ε = 1 for `explore_for` periods, then 0) |
| `explore_for` | 1 000 | reset | for `two_phase` (L24) |
| `epsilon`, `temperature`, `cooling` | 0.05, 1 000, 10⁻⁵ | reset | for `constant` and `boltzmann` (the code starts T at 1 000 and takes the cooling as 10^x from its input; 10⁻⁵ is our default, CCDP-A's values used in its presets) |
| `update` | `asynchronous` | reset | `asynchronous` (only the price charged: CCDP) or `synchronous` (every price, toward the profit it would have earned against the rival's actual price and the state that would have followed: AFP; which next state a counterfactual price leads to is AFP's open point, read as the state with the firm's own price replaced) |
| `q_init` | `calvano` | reset | `calvano` (eq. 8), `zero`, `optimistic` (uniform on [`q_low`, `q_high`], AFP's reading), `random` (uniform on [`q_low`, `q_high`] per cell: L24's theory, fn 8; L24's figures start at the grid's mean price, which fits `zero` or `random`, not eq. 8 — an inference, reported) |
| `q_low`, `q_high` | 10, 20 | reset | the range for `optimistic` (AFP's values) and `random` |
| `ties` | `lowest` | reset | `lowest` (the paper) or `random` (the code) |
| `rng` | `ours` | reset | `ours` (the project's generator) or `calvano` (RAN2 seeded as the code, for docking) |
| `session` | 1 | reset | the session number (the code's seeds are −session) |
| `cap` | 10⁹ | reset | the stop (the code: 1.25 × 10⁹) |
| `window` | 100 000 | reset | periods of unchanged strategies that count as convergence |
| `equilibrium_check` | `best_response` | live | `best_response` (true Q by value iteration against the rival's strategy: the paper) or `one_shot` (the code's policy evaluation) |
| `impulse` | `best_response_down` | live | `best_response_down` (the paper: one period at the static best response), `every_price` (CCDP-A's and E&L's Table 1: one period at each grid price, below and above the pre-deviation price; E&L measure each firm's relative price change at τ + 1 against τ), `invitation` (E&L's Fig. 2: the deviator one grid step up for one period, the rival forced to match from the next period, the deviator regaining control after `invitation_hold` periods), `up` (one step up for one period: a convenience, not a paper's protocol). Deviations come after 10 periods of on-path play (E&L's τ = 10) |
| `invitation_hold` | 5 | live | E&L: the deviator regains control at τ = 16 |

## Step (one period)

Each firm draws its exploration uniform, then its price uniform (in the code's order when `rng = calvano`); explores with probability ε or plays its greedy price; the market clears; each firm updates its Q-table in firm order (one cell, or every price under `synchronous`) and its greedy strategy at the visited state (with `ties`); ε decays; the convergence counter grows if no firm's greedy price at the visited state changed, else resets to 1. At convergence or the cap the session is finished; the analysis then runs once.

## Statistics

`SERIES`: `price_1`, `price_2` (prices charged), `profit_gain` (Δ of this period's profits), `epsilon`, `stable` (the convergence counter), `explored` (share of firms exploring this period), `greedy_changes` (greedy changes this period). `greedy_price` (the mean over firms of the greedy price at the visited state: L24's measure). After the session finishes: `cycle_length`, `cycle_gain` (Δ over the limit cycle), `window_gain` (realized profit over the last 100 000 periods, exploration included: dBMS's and E&L's reading of π̄), `discounted_gain` (dBMS's Δ̃ from period 1 over T_δ periods, T_δ = 165 at δ = 0.95), `equilibrium_on_path`, `equilibrium_off_path` (shares of states), `rp_complete` (below), `stale_greedy` (the share of off-path states whose greedy price is still eq. 8's initial one: learning inertia), `fumbling` (periods between ε falling below 0.01 and the last greedy change), `converged`, `periods`. Long sessions thin the per-period series for display (every 1 000th period after the first 10 000); the statistics themselves use every period.

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
| `no-memory` | Pricing algorithms that remember nothing (B1: L24's test, δ = 0.95) |
| `myopic` | Pricing algorithms that ignore the future (B2) |
| `synchronous` | Algorithms that learn from every price, not only the one they charged (B4) |
| `explore-more` | Pricing algorithms that keep experimenting (B5) |
| `price-war` | One firm undercuts once; the other responds (impulse) |
| `invitation` | One firm raises its price and the other follows (E&L's invitation, B3c) |

## Experiments and CLI

Built-in sweeps (`sweep.rs` BUILTINS), each a set of sessions reporting mean, standard error and distribution: `collusion-table-i` (1 000 sessions), `collusion-alpha-beta` (a 10 × 10 subgrid of Figures 1–2, 100 sessions a cell), `collusion-delta` (Figure 6's δ), `collusion-impulse` (Figure 4), and one per B test (`collusion-memory`, `collusion-myopic`, `collusion-two-phase`, `collusion-every-price`, `collusion-below-nash`, `collusion-invitation`, `collusion-synchronous`, `collusion-exploration`, `collusion-repair`, `collusion-timescale`, `collusion-rp-complete`). Sessions run in parallel natively.

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
| A7 | The critics' own baselines | reported, not scored: E&L's census of outcomes (one symmetric point 27.9 %, one asymmetric point 36.2 %, period-2 cycles 18.3 %, longer 11.0 %, hybrid 6.5 %) and their mean price 1.79; L24's greedy-price paths (Fig. 1); dBMS's Δ against δ with `window_gain` (Fig. 10) — each against ours under the same measure |
| A8 | The critics against themselves | reported, not scored: L24's Lemma 2, eq. 9 and Theorem 2 contract by α where his recursion contracts by 1 − α(1 − δ), and Fig. 2's caption puts the switch at t = 10 against the text's 1 000; E&L never state δ or Fig. 1's deviation size, and their fn 4 restriction conflicts with Table 1's top rows; dBMS's Exp3 rate in Theorem 2 differs from its proof's, and their π̄ is a window average, not CCDP's limit cycle |

### B. Is it collusion?

A **punishment-like response** (fixed now): in the period after a one-period deviation, the rival's price is at least one grid step below its pre-deviation price, and both prices return to the pre-deviation cycle within 25 periods. A session is **RP-complete** (dBMS p. 21) if every one-period deviation by either firm to any other grid price, from every state of the limit cycle, gives a punishment-like response. The B rules below were set before measuring; those marked *ours* fill a threshold the paper does not give, and E&L's, L24's and dBMS's own criteria are used where they state one.

| # | Test | Supports the critique if |
|---|---|---|
| B1 | L24's key test: memory 0 with δ = 0.95, other parameters as baseline; CCDP-A's reading (memory 0, δ = 0) beside it | L24's claim: the mean greedy price at 1.5 × 10⁶ and 2 × 10⁶ periods **and** the converged Δ are each at least as high at k = 0 as at k = 1 (difference ≥ −2 SE). The weaker critique (set first, kept as the secondary verdict): Δ(k = 0) ≥ ½ Δ(k = 1) |
| B2 | δ = 0 (no future to protect) | Δ(δ = 0) > 0.1; Δ(δ) − Δ(0) is then reported as the part strategies can explain |
| B2b | L24's Theorem 1: `exploration = two_phase` (1 000 periods), memory 0 and 1, at δ = 0 and 0.95 | the theorem predicts both firms at 1.6990 (Δ = 0.707) at δ = 0 and at 1.7377 (Δ = 0.794) at δ = 0.95 on our grid (computed in planning); holds if at least 80 % of sessions end there (*ours*) |
| B3 | E&L's Table 1: from each converged session, one period at every grid price above and below the pre-deviation price (increments excluded where the pre-deviation price is at or above p^M; cuts excluded from the grid minimum) | E&L's claim: in every (pre-deviation price, upward deviation) cell with at least 30 sessions (*ours*), the non-deviator's mean relative change at τ + 1 is negative, and the mean over upward cells is at least half the mean over downward cells (*ours*). The first rule (punishment-like responses after upward deviations at least half as often as after the paper's downward one) is reported beside it |
| B3b | E&L App. C: `grid = below_nash`, 10 000 sessions | the shares converging to the top price, to cycles and to singletons below it, against E&L's 53 / 9 / 38 % (reported); in the below-Nash singletons, upward deviations get punishment-like responses at least half as often as downward ones |
| B3c | E&L's invitation (Fig. 2), on sessions that converged to a point | the deviator's mean price in the period it regains control is at least one grid step below its pre-deviation price |
| B4 | synchronous updating (AFP) | Δ falls by more than half |
| B5 | more exploration: `constant` ε = 0.05, and β ten times lower | Δ falls by more than half under either |
| B6 | re-pairing (Eschenbaum et al.; dBMS App. B is the same exercise): each firm trained in session s against the rival trained in session s + 1, greedy play from a random state | the cross pairs' Δ < ½ the original pairs' Δ |
| B7 | dBMS's timescale: `discounted_gain` over the first 165 periods, 1 000 baseline sessions | within 2 SE of uniformly random pricing's 0.497 on our grid (dBMS; checked in planning), and of −0.510 on `grid = symmetric`; also reported: the share of Q-cells updated by period 165, and the first period at which Δ̃ over a sliding 165-period window exceeds 0.547 |
| B8 | dBMS's "pattern is not a scheme": among baseline sessions that pass CCDP's test (the paper's deviation gives a punishment-like response), the share that are not RP-complete | at least a quarter (*ours*) |

Each verdict stays on whichever side of its threshold it lands. The survey's write-up states for each critique what its test found, and, where B2 holds, how much of the paper's Δ remains attributable to strategies.

## Page

`web/` gains the kind's view (strategy grids, price chart, impulse panel), presets with titles, and the A and B results in the description, with the paper's numbers beside ours.

## Testing

- **Demand and grid:** benchmarks to 5 decimals against the code's inputs for n = 2, 3, μ = 0.5 and the asymmetric costs; the 15 prices; π^N and π^M.
- **Q initialization:** the baseline row (5.790 … 6.278 … 4.111) and greedy price 5.
- **RAN2:** the port against the Fortran's first draws for seeds −1 and −7.
- **Learner:** the update on hand-worked cells; incremental greedy maintenance against a full recompute over a long random run; ties under both readings.
- **Docking (A2):** a fixture of a few sessions' converged strategies and periods from the authors' code, checked into the test data (MIT).
- **L24's Theorem 1:** its fixed points on our grid (1.6990 at δ = 0, 1.7377 at δ = 0.95) computed from the payoff table, and `two_phase` sessions reaching them.
- **dBMS's timescale:** uniformly random pricing's Δ̃ = 0.497 on our grid and −0.510 on `symmetric`, T_δ = 165 at δ = 0.95.
- **Analysis:** the limit cycle and Δ on constructed strategies; RP-completeness on a hand-built grim-trigger pair (complete) and on a pair that punishes only cuts (not); both equilibrium checks on a strategy pair known to be (and not to be) an equilibrium; impulse responses on a hand-built punishment strategy.
- **Determinism:** golden fingerprints for the presets, native and WASM.

## Next in this kind (not built here)

- **Lambin's remedies** (L24 §5): an exploration floor on one firm after convergence, and sequential entry with the incumbent no longer exploring; and his n-step approximations of the decaying ε (Figs. 3 and 10; the step construction is unstated).
- **dBMS's two-price game:** m = 2, every strategy classified (grim trigger, win-stay-lose-shift, always defect, other), against their final shares (40.9 / 2.4 / 3.6 / 53.1 %) and their bounds on when each strategy can first exist (periods 6 633 and 12 308).
- **Klein (2021)**, **Calvano et al. (2021)**, **AFP's homogeneous Bertrand**.

## Docs

`docs/papers.md` (the Reproduced row; the Queue's #1 removed; Wanted gains Calvano et al. 2023 ("Genuine or spurious?") and Abada & Lambin 2023, still unfound, and Klein's and Calvano 2021's code, available from the authors on request), the README's model list, and the module docs. The swarm-coordination study (`docs/studies/2026-09-27-swarm-coordination.md`) cites the milestone as its no-communication reference.

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
