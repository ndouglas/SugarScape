# SugarScape milestone — Q-learning auctions — Design

**Date:** 2026-10-02. **Milestone number:** assigned at merge.
**Kind:** `auctions`, displayed as **Q-learning Auctions**.
**Design basis:** architecture and scope approved in chat on 2026-10-02.
**Reading notes:** `2026-10-02-q-learning-auctions-reading-notes.md`.
**Recovered source data:** `2026-10-02-q-learning-auctions-source-figures.json`.
**Closest relative:** milestone 34, Algorithmic Collusion, merged `bcfd7a8`; its design,
reading notes and implementation plan supply conventions, not auction rules.
**Primary source:** Banchio & Skrzypacz, *Artificial Intelligence and Auction Design*,
[arXiv 2202.05947v1](https://arxiv.org/abs/2202.05947v1), 2022-02-12. Page references below
are printed pages. The local original PDF is in `papers/multi-agent-coordination/`.

## Goal and intended use

Reproduce the paper's fixed-value, memoryless Q-learning bidders as a full playground model.
Ask whether first-price bidding settles low while second-price bidding reaches the top bid,
and whether revealing rival bids **and using them for counterfactual updates** changes that.
Cover the paper's other experiments with named reconstruction choices. The playground, CLI
and survey must make the source claims, observed outcomes and limits easy to compare.

Success means a faithful, inspectable experiment with fixed decision rules, including a
truthful account when a result depends on an unspecified convention or fails to reproduce.
It does not require obtaining the paper's numbers. Low bids measure seller revenue loss and
bidder benefit; they do not by themselves diagnose intent, communication or punishment.

## Binding constraints

- Existing configs, fingerprints and fixtures remain unchanged. Add a new model kind and
  new golden entries; do not refactor the pricing engine to accommodate auctions.
- One sequential learner path serves native and WASM. Follow existing seeded RNG and
  portable arithmetic conventions: `f64` draws, `u32` ranges, `portable::exp_neg` on
  nonpositive arguments. No platform-specific fast-learning implementation.
- One tick executes `periods_per_tick` complete auctions, default 1,000. Exploration,
  learning, occupancy and stability are updated **each period**, not once per tick.
- Every run reaches its configured horizon. Final greedy stability selects the paper's
  terminal sample; it never stops a run early. Constant exploration also reaches its horizon.
- Keep all raw sessions and their convergence flags. Terminal paper summaries select
  converged sessions; unconditional summaries and discard counts remain available.
- Original author simulator has not been located. Do not offer an author-code preset or
  claim exact docking. Public sources searched and ACM access limitation are in the notes.
- Literal specified rules are defaults. Unspecified defaults are identified as our
  reconstruction; later variants and ambiguity switches retain their attribution.
- Commit this spec and its decision rules before any learning measurement. Planning may
  then build and verify scratch code as requested by the user. New rules after looking at
  results require an explicit dated amendment and a separate verdict; original rules stand.
- Paper PDFs and original document-source archives stay gitignored in `papers/`.

## Source model and documentary checks

Two bidders value one item at 1, in each repeated auction. The grid has 19 positive bids:
`b[k] = k / (19+1)`, k=1..19, hence .05 through .95. There is no history state.
Each bidder owns one Q vector. The highest bidder wins; payment for format parameter a is
`(2-a)*highest_bid + (a-1)*second_bid`, a in [1,2]. First price is a=1; second price a=2.
Winner reward is `1-payment`, loser reward zero (§2, pp.6–7).

Chosen-action learning is
`Q_next(chosen) = (1-lr)*Q(chosen) + lr*(reward + discount*max(Q_old))`.
Other entries stay fixed. Baseline learning rate .05, discount .99, exploration probability
`.025*exp(-.0002*t)`, uniform exploration over the entire grid, optimistic initialization
(§§3–4, pp.8–10). Its numerical initialization, greedy tie rule, seed protocol and simulated
auction tie rewards are not specified.

The horizon is 1,000,000 periods. The final 1,000 post-update greedy profiles must all be
identical for a session to enter the paper's terminal sample. Main experiments use 1,000
sessions; feedback 500, local exploration 100, three bidders 500. These exceptions come
from the captions. Do not substitute Calvano's convergence stopping protocol.

Known source distinctions, retained in documentation and survey outcomes:

- Figure 1's annotated FPA counts imply mean revenue **.2265**; prose reports **.24**.
  SPA's figure implies **.9471**, which rounds to the prose's .95, and 994/1,000 at (.95,.95),
  with six exceptions including (.65,.95). The categorical no-dispersion/no-off-diagonal
  wording is stronger than its plotted sample. No cause for the difference is established.
- Figure 5a annotates 500/500 at **(.90,.90)**. Footnote 16 explicitly names that endpoint.
  Its colorbar normalization is unexplained; use annotated counts, not its scale, as source data.
- On the stated FPA grid, (.90,.90) is a **weak Nash equilibrium**: a fair tie gives .05
  per bidder; raising to .95 also gives .05. (.95,.95) is strict. This qualifies §2.1's
  unique-equilibrium claim for all formats. Enumerate the game's equilibria; do not use
  below-top bidding as a synonym for a profitable deviation or for non-equilibrium play.
- Figure 2's TeX supplies collusive percentages 100/100/100/100/86/41/15/2/0/0 at left
  endpoints 1/1.1/.../1.9; (2,0) closes an interval bar. The exact definition of collusive
  is unstated. The nearby prose reverses the low/high-format direction; equation, result
  and plot agree. Score an explicitly named reading of that figure, not an invented definition.
- Figure 3 and Figure 5b are example paths. Their moving-average/time-origin details are
  missing. They do not supply frequency claims about representative trajectories.

## Architecture

Add `ModelKind::Auctions`, `ModelConfig::Auctions(AuctionsConfig)` with `model: auctions`,
and `AuctionsWorld` implementing `Model`. Code is under `crates/sugarscape-core/src/auctions/`:

| Unit | Responsibility and dependencies |
|---|---|
| `config.rs` | Reading enums, config, validation and existing schema primitives |
| `mechanism.rs` | Grid, allocation/payment/rewards, counterfactual payoffs, static benchmarks; depends on config |
| `learner.rs` | One Q vector, greedy policy, initialization/choice/update; consumes explicit rewards and draws |
| `analysis.rs` | Terminal policy evaluation, occupancy summaries, deviation gains and source comparisons; depends on mechanism |
| `world.rs` | Draw schedule, sequential periods, fixed horizon, final-window stability, aggregation and model interface |
| `stats.rs`, `view.rs` | Series, bounded frame history, Q/occupancy rendering and inspection |
| `presets.rs`, `mod.rs` | Source-labelled treatments and public model exports |

Reuse existing RNG, portable math, schema, canvas, stats and experiment infrastructure.
Keep auction-specific learning here: Calvano's tie cache, price-history state, stopping rule,
limit-cycle and punishment analysis have different semantics. A generic Q-game framework is
deferred until matching duplicated semantics justify it.

Wire the kind into model parsing/dispatch, schema, preset titles, export, CLI, WASM, page
types/controls/charts/Inspect/Compare/Experiments, built-in sweeps and the standalone survey.
Integration locations follow the current `collusion` addition. No new dependencies are planned.

## Configuration and named readings

All fields apply at reset: fixed experiment parameters must not change mid-run. A reset is
required for even learning-rate changes. Defaults below define the **stated baseline plus
documented reconstruction**, not an inferred author implementation.

| Fields | Defaults | Allowed values / semantics |
|---|---|---|
| `bidders` | 2 | 2 or 3 strategic bidders, all valued at 1 |
| `bids` | 19 | 2..99 positive bids `k/(bids+1)`; highest remains strictly below 1 |
| `auction` | `first_price` | `first_price`, `second_price`, `mixture` |
| `auction_alpha` | 1 | [1,2], used only by `mixture`; endpoint enums resolve to 1/2 |
| `reserve` | 0 | [0, highest grid bid] |
| `reserve_payment` | `floor` | `floor` or `eligibility_only`, as defined below |
| `out_bids` | 0 | 0..16 distinct nonparticipation actions; 7 yields -.30,-.25,...,0 on baseline grid |
| `fringe` | `none` | `none` or `uniform`: one additional U[0,1) bid per auction |
| `learning_rate`, `discount` | .05, .99 | (0,1], [0,.9999] |
| `feedback` | `outcome` | `outcome` or `rival_bids` |
| `update` | `chosen` | `chosen` or `all`; `all` requires `rival_bids` |
| `auction_ties` | `sampled` | Fair sampled winner or `expected` fractional rewards |
| `hindsight_ties` | `expected` | `expected` share or `realized`, using the actual allocation convention/priority |
| `greedy_ties` | `lowest` | `lowest`, `highest`, `random`, `incumbent` |
| `q_tolerance` | 0 | [0,1e-8]; Q actions within this absolute distance of the maximum tie |
| `q_init` | `optimistic` | `optimistic`, `constant`, `biased` |
| `optimism` | `discounted` | `discounted` or `stage`; only affects `optimistic` |
| `q_scale`, `q_level` | 1, 100 | Scale [1,10]; constant level [0,1e6] |
| `bias_bid`, `bias_q`, `bias_rest` | .40, 30, 0 | Bid must be a positive grid action when `biased`; Q values [0,1e6] |
| `exploration` | `decaying` | `decaying` or `constant` |
| `epsilon`, `beta` | .025, .0002 | Probability [0,1]; decay [0,1] |
| `exploration_set` | `all` | `all`, `other` (exclude cached greedy), `neighbors` |
| `neighbor_boundary` | `available` | Uniform over existing neighbors, or `clamp` a missing neighbor to the edge action |
| `period_origin` | `zero` | `zero` or `one`; origin for the first auction's epsilon |
| `convergence_phase` | `post_update` | `post_update` or `pre_update`; greedy profile observed for stability |
| `downward_trigger` | `off` | `off`, `stable`, `period`; enable the extra low-bid choice once |
| `downward_at` | 100,000 | Positive period threshold for `period` trigger |
| `downward_clock` | `activation` | `activation` or `global` |
| `downward_chi`, `downward_beta`, `downward_gap` | .62, .002, .30 | [0,1], [0,1], [0,1e6]; gap is in raw Q units |
| `horizon`, `window` | 1,000,000, 1,000 | 1..100,000,000 periods; 1..horizon final stability observations |
| `periods_per_tick` | 1,000 | 1..1,000,000 complete sequential periods |

Unused fields remain serialized so a config is explicit and round-trips through existing
schema/export patterns. Validation applies universal type/range/finite checks and conditional
requirements when a field is active; e.g. a non-grid `bias_bid` is rejected only for `biased`.
Reject unknown fields, nonfinite values, all-action learning without sufficient feedback, and
invalid ranges with the project's contextual `FieldError` conventions. No silently clamped config.
Schema groups are **Auction**, **Bids**, **Learning**, **Information**, **Exploration** and
**Session**. Disabled controls explain their activating choice through existing schema copy.

### Initialization

For `optimistic`, every action starts at `q_scale/(1-discount)` under `discounted` (100 at
baseline), or `q_scale` under `stage` (1). The baseline's maximum one-period reward is .95;
the discounted upper bound is 95. These two readings make the ambiguity about optimistic
payoff versus optimistic action value measurable. Never choose one after seeing its fit.
`constant` starts every action at `q_level`. `biased` starts `bias_bid` at `bias_q` and every
other action at `bias_rest`; 30 corresponds to the equal-.40 two-bidder discounted payoff
at discount .99, but the off-action zeros are our choice. This treatment is a reconstruction
of Figure 3's stated bias, not its unpublished Q vector.

### Allocation, reserves, nonparticipation and fringe

Positive bids below reserve are ineligible; equality with reserve is eligible. Each added
out action is distinct for exploration/Q updating but never wins or pays, even at reserve 0.
For K=`out_bids`, add actions `j/(bids+1)` for j=1-K,...,0. This exact extra grid is our
reading of the -.3..0 range; it is not specified by the source. If nobody is eligible,
the item is unsold, revenue and all rewards zero.

Highest eligible bid wins. Let c be the highest competing positive bid (0 if none), including
bids below reserve. Under `floor`, payment is `(2-a)*winning_bid+(a-1)*max(reserve,c)`.
Under `eligibility_only`, use c without that floor. Below-reserve bidders cannot win but
their bids can affect c. Negative out actions are excluded from c. The default is the usual
reserve payment floor; the source does not specify this convention. Fringe bids obey the
same eligibility/payment rules, compete in ranking and contribute seller revenue when they win.
No fringe Q vector or strategic fringe payoff is modeled.

For equal highest eligible bids, `sampled` allocates to one uniformly random tied participant,
giving that winner `1-payment` and everyone else zero. `expected` allocates fractional win
shares and gives each tied strategic bidder its expected reward. Revenue is the same payment
under either tie convention. Track allocation share as well as strategic bidder reward.
Strategic values are 1; the fringe is simply a bid generator, so do not infer social welfare
by summing only the strategic bidders' rewards and seller revenue in the fringe treatment.

### Information and counterfactual ties

`outcome/chosen` is baseline. `rival_bids/all` is the paper's feedback intervention.
`rival_bids/chosen` is an information-unused control and must produce identical economic
state to baseline with the same seed. Higher disclosure alone has no behavioral path here.
An unsupported `outcome/all` config is an error, not an implicit oracle.

All-action rewards evaluate each candidate bid against the actual competing bids, including
fringe. `rival_bids` provides the complete competing-bid vector: for two strategic bidders
without fringe this is the one rival bid in the paper. With additional participants this is
our richer-information generalization, sufficient to count tied competitors; it is not a
claim that revealing only the highest rival bid suffices to identify all tie shares.
At a counterfactual tie, `expected` uses the fair expected share even when the actual
auction sampled a winner. `realized` uses the actual tie mode; when sampled, the same
per-period participant priorities are reused for all hypothetical bids. This is a coupled
counterfactual experiment, not newly drawn independent winners for each candidate.
Actual sampled rewards versus expected hindsight rewards are separate named assumptions.

### Greedy choice and exploration

Greedy choice uses the Q maximum with the configured absolute tolerance. `incumbent` retains
the previous greedy action if it belongs to the maximizing set; otherwise use the lowest
maximizer. Initialize its incumbent to the lowest action. `random` selects uniformly among
tied maximizers. Recompute the cached greedy action after each Q update and use it next period.

Global `all` exploration includes the greedy action, as §3 formally describes. `other` chooses
uniformly from the remaining actions. Neighbor exploration uses immediate indices in the full
ordered action set; at an edge, `available` has a single neighbor, while `clamp` chooses an
up/down direction equally and stays at the edge when that direction is unavailable.

Epsilon is evaluated from the global completed-period count plus the configured origin;
use `epsilon*exp_neg(-beta*t)` for decay and `epsilon` for constant exploration. Calculate
from t rather than repeatedly multiplying a decay factor; the paper supplies no code convention.

`stable` downward activation occurs on the next auction after a greedy profile, observed
under `convergence_phase`, has persisted for `window` observations. `period` starts at the
next auction when the number
of completed periods reaches `downward_at`. Record activation period tau, activate once, and
continue for the horizon. After activation, with probability
`downward_chi*exp_neg(-downward_beta*s)`, choose the lowest action whose Q lies within
`downward_gap` of max Q; otherwise use epsilon-greedy. s is completed auctions since activation
(first s=0) or the same global clock as epsilon under `global`. These are named readings of
Figure 4's unspecified steady-state trigger and decay origin, not an inferred author protocol.
Tau records the completed-period count at the start of the first active auction; that auction
has ordinal tau+1. The activation clock uses `completed_periods-tau`, without the epsilon
origin offset, so its initial extra-choice probability is exactly `downward_chi`.

## Sequential periods, random numbers and final stability

Use the existing `SimRng`, seed passed through the standard model interface; no author seed
emulation. The draw schedule is fixed for reproducibility and paired ambiguity treatments:

1. At reset, draw one `f64` per strategic bidder for its initial greedy tie selection,
   including when the configured rule does not use that draw.
2. Each period, in bidder order, draw `f64` uniforms for downward choice, epsilon choice,
   exploratory-action selection, and post-update greedy tie selection, always all four.
3. Draw a fringe-bid uniform, then one priority uniform per strategic bidder and one for
   the possible fringe, even if fringe/ties are disabled. Lower priority wins a sampled tie;
   exact equal priorities favor lower participant index. Fixed participant count within a
   treatment makes unused-feedback and reward-noise comparisons use the same base draws.

Uniform action/tie selection maps a draw u to `floor(u*set_length)` on the ordered candidate
set. Do not draw additional conditional randomness. All bidders select from cached old Q,
then the auction clears, then each Q vector updates using its own pre-update maximum.
Record played bids/rewards/occupancy, recompute post-update greedy choices, update stability,
and increment period. All-action updating freezes the maximum for the entire vector update.
Construct each grid point by direct integer-to-`f64` division once, including out actions.
Use exact bid equality for allocation, and absolute 1e-12 to resolve `bias_bid` to a canonical
grid index. Q updates follow the displayed weighted-sum order, without fused multiply-add.
Probability tests use `uniform < probability`, so probability zero never selects that branch.

Stability counts identical consecutive **post-update profiles** by default; `pre_update`
instead observes the cached profile used at that period's start. Terminal policies are always
the post-update greedy profile after the final auction, with the stability phase named in the
export. Start the streak at 1 for the first observation; a different profile resets it to 1.
At the horizon, `converged` is true iff
the streak is at least `window`. A run that settles early and changes later can fail this check.
Finish exactly at the horizon; the final tick may be short. A finished world is immutable under
subsequent `step`/`run` calls. Report `tick`, economic `period`, and actual periods in the last tick.

## Statistics and analysis

Per-period accounting uses every auction. Display snapshots summarize one tick: played bids,
revenue, profits, exploration/downward share and policy changes are tick means or counts as
labelled; greedy actions and stability are values at tick end. Final outcomes appear only at
the horizon, following existing unavailable-series/JSON conventions.

Core series: `bid_1`, `bid_2`, `bid_3`, `greedy_1`, `greedy_2`, `greedy_3`, `revenue`,
`profit_1`, `profit_2`, `profit_3`, `epsilon`, `explored`, `downward`, `greedy_changes`,
`stable`, `converged`, `terminal_revenue`, `terminal_deviation_gain`, `top_profile`, `periods`.
Third-bidder series are unavailable for two-bidder sessions. CLI CSV time remains ticks,
with `periods` providing the economic clock; page captions show periods per tick explicitly.

Each session retains:

- Config, seed, horizon, final Q vectors/greedy actions, final stability/convergence, update
  counts per action, played-action counts, win shares, activation period if any.
- Whole-run realized mean seller revenue/profits, and last-20%-of-periods means, including
  unsold and fringe-won auctions. The last window is periods `floor(.8*T)+1` through T.
- Played-pair occupancy of bidders 1 and 2 for the whole run and last 20%, counting every
  auction even with a third bidder. Do not select only diagonal pairs or successful sales.
- Terminal greedy-policy expected revenue/profits and each bidder's maximum profitable
  one-period deviation against fixed rival greedy bids. For fringe, integrate over U[0,1).
  Do not use the last realized payment as expected terminal revenue.
- Static pure-equilibrium profiles for the finite game without fringe. Baseline includes
  (.90,.90) and (.95,.95) in FPA and the top pair in SPA. An unequal profile cannot be a
  pure equilibrium here: a losing bidder can join the highest eligible bid and earn strictly
  positive expected reward; an unsold profile admits a profitable eligible top bid. Thus test
  each equal eligible profile against every unilateral
  action, rather than enumerate a large Cartesian product. With fringe report terminal
  expected deviation gains, without an exhaustive equilibrium search. Expected-payoff
  comparisons use absolute 1e-12 tolerance.

For two-bidder no-fringe terminal outcomes, expected revenue is the highest bid for FPA and
lowest bid for SPA. With reserve/nonparticipation apply the specified mechanism. Integrating
the fringe is deterministic: split [0,1] at strategic bids and reserve; allocation is constant
and payments/rewards affine inside each interval. Integrate each affine interval exactly
using interior quarter/three-quarter points, avoiding boundary tie discontinuities.

`top_profile` means all strategic greedy bids equal the largest grid bid. `below_top` means
all are strictly below it. `profitable_deviation` means expected one-period gain >1e-12;
`low_non_nash` means below_top plus a profitable deviation. Keep all three measures, rather
than assigning collusion by top-grid status. Stale estimates are inspectable through action
update counts; do not infer a punishment mechanism from price/bid movements.

Histograms are counts (`u64`) indexed by actions, not per-period vectors. The largest permitted
pair histogram is 115x115. Store whole-run and last-window histograms and bounded existing
chart history; do not retain 100m auctions in memory. A third bidder's policy is always recorded
in terminal data, even when the pair view displays only the first two.

## Playground views and user copy

Use the established canvas/Inspect pattern:

- **Bids view:** a played-pair occupancy grid with axes as bids, selectable whole run or
  final 20%; mark the current played pair and report total auctions counted. The view is
  one session's occupancy, not Figure 1's ensemble of terminal policies.
- **Values view:** each bidder's Q vector with greedy/played actions and update-count rows;
  show auction payment, winner/share, epsilon, final-window stability and completed periods.
- Inspect a pair cell for its count/frequency, or a Q/action cell for Q, times chosen,
  times updated and the current hypothetical reward against the displayed competing bids.
- Charts: **Bids** (played and greedy), **Seller revenue**, **Bidder rewards**, **Learning**.
  Titles distinguish tick means from end-of-tick policies; changing display aggregation does
  not change the learner. Describe three-bidder views as the first two bidders' pair projection.

Compare entries: **First price vs second price** and **Learning from one bid vs every bid**.
The latter changes feedback and updating together, matching the paper. Provide a separate
unused-feedback preset/control. Required notice: a session finishes at its fixed horizon,
with whether its final strategy window was stable; transient stability is not completion.
No UI promise that either format prevents collusion in general. Preset descriptions quote
the paper's targets beside measurements when available; titles remain neutral until measured.

## Presets

All use the default reconstruction unless explicitly changed. Each description names numerical
initialization and sampled/expected tie assumptions, plus source figure and sample unit.

| ID | Initial title | Changed fields |
|---|---|---|
| `auctions-first-price` | Two bidders learn how much to pay | Default |
| `auctions-second-price` | The winner pays the other bidder's bid | auction=second_price |
| `auctions-feedback` | Bidders learn from every bid they could have made | feedback=rival_bids, update=all |
| `auctions-unused-feedback` | Bidders receive information they do not use | feedback=rival_bids, update=chosen |
| `auctions-local` | Bidders try the next bid up or down | exploration_set=neighbors |
| `auctions-biased` | Bidders start with a preference for low bids | q_init=biased, epsilon=.25; bias .40/30/0 |
| `auctions-downward` | Bidders are nudged toward lower bids | downward_trigger=stable |
| `auctions-nonparticipation` | Bidders can choose to sit out | out_bids=7 |
| `auctions-reserve` | The seller sets a minimum bid | reserve=.20 |
| `auctions-three` | Three bidders learn together | bidders=3, discount=.99 |
| `auctions-three-patient` | Three bidders put more weight on future rewards | bidders=3, discount=.999 |
| `auctions-fringe` | A random bid joins the auction | fringe=uniform |
| `auctions-persistent` | Bidders keep experimenting for a hundred million auctions | exploration=constant, epsilon=.001, horizon=100m |

Second-price and other companions are sweep axes/config variants, not duplicate presets.
Update titles only to truthful measured behavior after the survey; retain source attribution.

## Experiments, workloads and export

Native survey and playground sweeps call the same engine. The native survey may run a paper's
1,000/500 sessions; playground sweeps remain at <=100 seeds and <=100,000 ticks. Default
1,000 periods per tick fits the longest 100m horizon in exactly 100,000 ticks. Page limit remains
1,000,000 ticks. For arbitrary batching, compute ticks remaining with ceiling division and
handle partial final ticks. Do not change legacy caps to accommodate the paper.

Built-ins, capped at 100 seeds unless otherwise noted:

| Sweep | Arms / horizon |
|---|---|
| `auctions-formats` | auction alpha 1..2 in .1 steps, 1m |
| `auctions-feedback` | outcome/chosen, rival_bids/chosen, rival_bids/all, 1m |
| `auctions-initialization` | optimism stage/discounted and constant levels 2/10/100/1,000, both formats, 1m |
| `auctions-ties` | actual sampled/expected x greedy lowest/highest/random/incumbent, both formats, 1m |
| `auctions-hindsight` | actual sampled/expected x hindsight expected/realized, first-price all updates, 1m |
| `auctions-local` | global all/other and local available/clamp, both formats, 1m |
| `auctions-biased` | FPA/SPA x bias_rest 0/29.7, bias_q=30, epsilon=.25, 1m |
| `auctions-downward` | stable/period trigger x activation/global clock, both formats, 1m |
| `auctions-market` | baseline/out_bids=7/reserve=.2/fringe=uniform, both formats, 1m |
| `auctions-bidders` | 2/3 bidders x discounts .99/.999, both formats, 1m |
| `auctions-persistent` | 1 seed, constant epsilon=.001, FPA/SPA, 100m |
| `auctions-duration` | horizon 1m/10m/100m, FPA/SPA, 20 seeds; runtime explicitly shown |

Use the existing finite sweep axes rather than introduce a general treatment language. Fields
whose enum combinations need different configs can follow current built-in construction patterns.
Every export specifies horizon/periods_per_tick, config, seed list, convergence selection,
metric names, and denominators. Native per-session JSON summaries are available through the
existing inspection/export route, without embedding Q arrays in every chart snapshot.

## Pre-registered survey rules

These are project acceptance rules, not thresholds claimed by the paper. They are fixed in
this spec before implementing or running learners. The default reconstruction is always primary;
ambiguity arms never replace it based on fit. Cache sessions by full config and seed for reuse.

All terminal paper populations use seeds 1..N, the fixed 1m horizon and the final stability
selection. Record requested N, converged n, and discarded seeds. Require n>=.95N for a scored
terminal reproduction; otherwise **Inconclusive** for terminal-fit claims, with observed fit
and unconditional outcomes reported. This .95 coverage threshold is ours, translating the
paper's nearly-all language; it is not an automatic failure of its economic directional claim.
Mechanism/protocol identity checks do not use that coverage gate. Figure histograms normalize
by n, not requested N. Report means, session SE and interval estimates, even when a fixed
tolerance determines the verdict. No statistical significance test treats periods as independent.

For Figure 1 histogram comparison, sort the two bids within each source/observed pair because
bidder labels in the source x/y axes are unspecified and bidders are symmetric. Use all bins,
including off-diagonal and out actions. Define TV=.5*sum(abs(observed_frequency-source_frequency)).
Histogram fit requires TV<=.10 and the largest single-bin gap<=.05. Mean fit requires absolute
gap<=.02 except SPA's rounded revenue target, whose tolerance is .005. These tolerances are ours.

| ID suffix (`auctions.bs.`) | Workload / metric | Holds if |
|---|---|---|
| `baseline-direction` | 1,000 FPA and SPA sessions | Conditional terminal revenue FPA<.50, SPA>=.90, and SPA-FPA>=.40; also report unconditional values |
| `figure-1-fpa` | Same FPA runs; recovered full histogram/.2265 | Histogram rule and mean gap<=.02 |
| `text-fpa-revenue` | Same; textual .24 | Absolute mean gap<=.02, scored separately from figure |
| `figure-1-spa` | Same SPA runs; recovered histogram/.9471 | Histogram rule and absolute mean gap<=.005 |
| `text-spa-revenue` | Same; textual .95 | Absolute mean gap<=.005 |
| `feedback-direction` | 500 all-action FPA and 500 matched chosen-action FPA | All-action terminal revenue>=.85 and at least .30 above chosen-action |
| `figure-5` | Same 500 all-action FPA; target all at (.90,.90) | At least .90 of converged profiles equal (.90,.90), mean revenue within .02 of .90 |
| `figure-2-below-top` | 1,000 sessions each a=1,1.1,...,2 | Proxy below_top shares versus TeX percentages: mean gap<=.05, worst gap<=.10 over a=1..1.9; report a=2 separately |
| `local` | 100 sessions each FPA/SPA; neighbor available | Same directional thresholds as baseline-direction; report global/local shifts |
| `biased-reading` | 1,000 each format; explicit .40/30/0 bias, epsilon=.25 | FPA mean terminal revenue<.50; SPA>=.90; this scores our reconstruction, not the unpublished Figure 3 path |
| `downward-reading` | 1,000 each format; stable trigger and activation clock | At least .50 of converged profiles below_top in each format; alternate triggers/clocks reported separately |
| `nonparticipation` | 1,000 each format; out_bids=7 | FPA mean terminal revenue at least .025 below baseline; report lowest-positive profile frequency and all-out outcomes |
| `reserve` | 1,000 FPA; reserve=.2 | Mean greedy bid averaged over bidders at least .025 above baseline; report mass at reserve and unsold fraction |
| `three-bidders` | 500 each format/discount .99,.999 | Three-bidder FPA below_top share increases by >=.10 at .999; SPA top-profile share>=.95 at each discount |
| `fringe` | 1,000 each format; U[0,1) fringe | FPA mean strategic greedy bid>.50, and >=.50 of converged profiles have all strategic bids in {.60,.65}; SPA top-profile share>=.95 |
| `persistent` | One 100m-period run each, seed 1, epsilon=.001 | Whole-run SPA top played-pair occupancy>=.80; FPA mean realized seller revenue<.50; record late-window results and changes without early stopping |

Figure 2, biased and downward readings carry an explicit underdetermined-source caveat even
when their scores pass or fail. Source claims needing an unstated threshold/vector/time origin
cannot be called literally disproven by these reading scores. The .025, .10 and .50 extension
effect thresholds are ours, fixed now rather than inferred from our samples.

Separate protocol/control records (`auctions.controls.*`):

- `unused-feedback`: same seed/config except feedback outcome versus rival_bids, chosen
  updates in both. Ten short CI seeds and 100 full survey seeds must have identical Q vectors,
  actions, period statistics and outcomes; config-inclusive fingerprints need not match.
- `stage-game`: enumerate baseline pure equilibria and verify the .90 weak FPA equilibrium,
  strict .95 equilibrium and SPA profitable .90-to-.95 deviation. Documentary, not learner-dependent.
- `source-consistency`: preserve figure-vs-prose revenue, rare SPA exceptions, Figure 2's
  reversed sentence and Figure 5's colorbar/endpoint. Report checked arithmetic; no invented cause.
- `initialization`, `ties`, `hindsight`: all predeclared ambiguity arms use seeds 1..100 at 1m.
  Report every arm's revenue/coverage/histogram gap, largest effect and whether the primary
  directional distinction survives. They are sensitivity tables, not a search for a winner.
- `duration`: 20 seeds each at 1m/10m/100m under decay; report means/distributions and discard
  counts. A descriptive footnote-13 reconstruction, with counts chosen by us; no exact author count.
- `persistent-seeds`: 20 seeds each format at 10m, constant epsilon=.001; report distributions
  of occupancy/revenue. This is our seed-sensitivity study, distinct from the source's one 100m run.

For scored paired extensions, each arm must meet the coverage rule; missing coverage yields
Inconclusive, not a silently shortened denominator. Findings include resulting uncertainty.
The permanent verdict registry distinguishes source results, reading scores and controls.
The survey's generic `--seeds` flag does not silently reduce these published-count claims:
each paper check uses its fixed N. A user-selected smaller experiment is labelled exploratory
and does not replace the registered result. Prefixes separate baseline/feedback checks from
extension and long-duration controls so they can be invoked incrementally.

## Verification and performance

Seconds-only CI checks behavior on short horizons:

1. Auction outcomes/payments for distinct bids, fair ties, mixtures, reserves, out actions,
   third bidders and fringe; expected reward sum for a no-fringe sale equals 1-revenue.
2. Hand-calculated chosen/all-action Q updates; unchanged unchosen entries and frozen old
   maximum; expected and shared-priority hindsight rewards; feedback validation/control identity.
3. Greedy ties, exploration action sets/boundaries, epsilon origin and period indexing,
   optimistic/biased initialization and one-time downward activation/clock.
4. Final stability counts and exact horizon: a transient stable segment does not finish;
   last-window changes exclude the run; partial ticks stop exactly and finished state is unchanged.
5. Batching equivalence for 1/7/1,000 periods per tick: Q, policies, occupancy, stability and
   outcome match at equal economic horizons, though display ticks/fingerprints may differ.
6. Histogram denominators, conditional versus unconditional samples, last-20% window boundaries,
   static equilibrium enumeration and deterministic fringe integrals against hand calculations.
7. Short native/WASM golden fingerprints; existing goldens unedited. CI excludes full ensembles
   and long persistent/duration studies, including ignored tests unless explicitly invoked.

Use existing Rust/web build, test, lint and formatting commands; no new tooling. Native surveys
parallelize independent sessions through existing survey runner facilities, with deterministic
seed assignment and no shared RNG. During planning, benchmark without altering survey rules;
show a runtime estimate before explicit long experiment commands. Check browser responsiveness
with default and long-horizon presets; bound worker/render batches if required, without changing
the economic algorithm. A selected `periods_per_tick` above the remaining horizon still runs
only remaining periods.

## Documentation and implementation handoff

After verified implementation and findings, update README, module docs, paper index, survey
records and model roadmap. Assign the milestone number at merge and move Queue #1 to Reproduced
only then. Descriptions state measured reconstruction, source values and unresolved gaps.
The AI-safety connection belongs in the model description and swarm-coordination study as an
interaction/feedback result with limits, not evidence of advanced-agent deception.

The implementation plan must use 3–5 stages tracked in `IMPLEMENTATION_PLAN.md`, removed
when complete, and task-by-task instructions in `docs/superpowers/plans/`. Follow the user's
requested scratch workflow: implement and verify a scratch copy during planning, derive the
plan from that code, and verify executing the plan reconstructs it byte for byte. Use subagent
execution without asking which execution mode to select; obtain one fresh independent final
review of rules and measurements before integration. Merge/push/CI/Pages follow the established
project workflow after required verification; this spec does not assert those stages are done.

## Next in this kind

- Banchio & Mantegazza's three-keyword participation experiment: second-price market splitting
  concerns eight participation subsets, not this milestone's scalar bid action space.
- History-state disclosure, including the authors' warning and Decarolis et al.'s different
  information result. Keep memory changes separate from counterfactual updates.
- Rawat's bandits/pacing/asymmetric factorials, with their own protocols and paper version.
- Xu & Zhao's deterministic split-tie, sufficiently fine-grid persistence model. Their theorem
  requires strict incentives that the original .90 FPA equilibrium violates; its stationary
  epsilon-to-zero limit is different from fixed-epsilon finite occupancy. Figure 11's grid is
  unstated, so exact replication remains unresolved. Do not borrow its accelerated deterministic
  dynamics for stochastic sampled ties.

These follow-ups motivate diagnostics and interpretation; their environments are not silently
substituted into the original paper's reproduction.
