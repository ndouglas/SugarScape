# W1: reciprocal engagement benchmark

**Date:** 2026-10-09

**Status:** Approved for implementation planning on 2026-10-09. This is a design, not
implementation approval, a registered measurement protocol, or new findings.

**Research basis:** [first focused literature pass](../../studies/2026-10-09-war-literature-pass.md).

**Question:** under specified engagement capacity, can comparable finite forces exchange
casualties reciprocally, and how do their trajectories compare with mathematical references?

W1 benchmarks stationary engagements. It does not explain whole wars, historically calibrated
durations, morale, supply, retreat, or civilian mortality. A long surviving engagement is
not automatically a long campaign.

## Architecture and scope

Use a default-off native core feature, `war-benchmarks`, and a dedicated
`survey/src/bin/war1.rs` executable. The survey crate also declares a default-off local
feature `war-benchmarks = ["sugarscape-core/war-benchmarks"]`; the binary
`required-features = ["war-benchmarks"]` refers to that survey-local feature.
Guard the core module with both the feature and `not(target_arch = "wasm32")`. Do not add
a global `ModelKind`, ordinary `Config` fields, bindgen exports, browser controls, presets
or global golden fixtures.
The gate keeps this research runner outside the browser's reachable code while semantics
are reviewed; it is not a claim that it fixes the existing WASM artifact size.

The proposed `war` module contains a small validated config, finite-cohort kernel,
mathematical reference, records, and runner. Follow existing Rust/serde, seeded RNG,
portable-math and survey conventions. No generic combat framework is needed.

The initial enabled kernel owns stationary actors identified by stable IDs and sides. Each
actor represents one mortal individual, with its own survival state; a homogeneous side
shares a casualty-production rate. This is a controlled finite population, not a complete
Sugarscape agent, household, military unit of arbitrary size, or continuous force mass.
Continuous reference force is a different quantity with explicitly declared units.

Run two named paths:

- **`book_c`:** build the existing `World` from its existing configuration and seed; delegate
  every tick to `World::step`. Observe the existing events without new RNG draws. Preserve
  the original shuffled sequential update, rewards, movement and economy exactly.
- **`reciprocal_graph`:** run the independent stationary kernel described below. No movement,
  terrain, HP, morale, supply, loot, inheritance, replacement, harvest or metabolism.

The all-switches-off exact-C requirement applies to this dispatcher: the disabled study
path is the actual World implementation. It does not assert that enabling a stationary
contact graph preserves the economy or scheduling. W1 does not need an enabled World
adapter; pure kernel plus literal C baseline suffices. Later integration as a new war
model on World requires its own design and review.

## Finite engagement procedure

Choose one geometry for a run:

1. **Aimed fire:** every living actor can contribute exposure to every living opponent.
2. **Duel contact:** each step independently draws a random disjoint cross-side matching
   with exactly `min(B, R)` pairs. Unmatched actors contribute and receive no exposure.
   Reshuffling is an explicit contact assumption, not simulated physical movement.

Construct eligibility from the immutable step-start population. Enemy status and the graph
permit contributions regardless of possessions or relative force size. Both sides contribute
before any deaths settle. This separates permission to engage from casualty adjudication.

A living actor on side B contributes total exposure rate `b`; on side R it contributes
`r`. Divide that rate evenly among its eligible opponents. A target j receives

```
h_j = sum over eligible i -> j of rate_i / outdegree_i
p_j = 1 - exp(-h_j * dt)
```

Draw one independent Bernoulli casualty trial per exposed target, then settle all successful
trials simultaneously. Each actor can die at most once. No trial is drawn for a zero-hazard
target. Compute probability stably as `p_j = -libm::expm1(-h_j * dt)`, mathematically equivalent
to the expression above. Pin the existing locked libm version and the portable wrapper
boundary in implementation review; do not silently substitute subtraction of an
exponential. Reject nonfinite hazards/products or probabilities outside finite `[0, 1]`.
If strictly positive hazard and `dt` multiply to zero, record `invalid_numeric` with the
actor and operands rather than treating the exposure as zero. If the finite calculation
rounds to `p_j = 1`, retain `h_j * dt` and a saturation diagnostic; rounding is not an
accuracy guarantee. Use the declared draw order below.

For aimed fire, red targets have hazard `b * B / R`, and blue targets `r * R / B`.
For matched duels, red targets have hazard `b` and blue targets `r`. Compute these directly;
do not materialize a dense all-to-all edge matrix. Handle empty sides before any division.

Exposure is a rate budget. This implementation does not generate individual arrows, shots,
attempts or attacker-selected hits. Do not synthesize shot/attempt counts or assign a unique
lethal attacker from aggregate hazard. It does not promise equivalence to a fixed one-shot
per-attacker-per-step implementation, whose collision/deduplication process differs.

Maintain total contributed exposure rate by side, integrated exposure over `dt`, exposed
actor counts, geometry, contact capacity, casualties and survivors. For aimed geometry,
contact-pair count may be computed as `B * R`; for duel geometry it is `min(B, R)` undirected
pairs. Neither count is an attack count. Define these denominators in the output schema.

The timestep freezes exposure until settlement. Its expected casualties are
`R * (1 - exp(-b * B / R * dt))` for red under aimed fire and
`min(B, R) * (1 - exp(-b * dt))` under duels, with mirrored blue expressions. These displayed conditional expectations use ideal `p`. The realized 53-bit sampler uses
the quantized probability `q` defined below. They saturate at the exposed population; they are not exactly the continuum casualty rates. Smaller
steps and ensemble comparisons are required to assess approximation, particularly near
extinction. No fit of the ensemble mean establishes exact finite-process ODE equivalence.

## Mathematical reference

Keep deterministic reference calculations independent of the finite kernel and RNG.
Use the declared homogeneous continuous-force assumptions:

- Aimed fire: `dB/dt = -r * R`, `dR/dt = -b * B`; invariant `b * B² - r * R²`.
  With equal forces and equal positive rates `k`, `N(t) = N(0) * exp(-k * t)`.
- Matched contact capacity: `dB/dt = -r * min(B, R)`,
  `dR/dt = -b * min(B, R)`; invariant `b * B - r * R`.

References stop at the first extinction boundary; they do not continue to negative force
or silently clip an invalid numerical trajectory. Rates may be zero, with the corresponding
closed-form boundary cases checked independently. Use analytic solutions or independently
checked integration with an explicit tolerance; do not reuse finite outcomes as the oracle.

These equations are mathematical benchmarks, not empirically fitted laws. Deterministic
uniform HP damage is excluded: synchronized health thresholds can create mass deaths and
cannot be presented as continuous attrition. HP granularity would require a separate study.

## Clocks, endings and provisional bounds

`dt` is a dimensionless model interval. `step` counts completed settlements; calendar model
time is `step * dt`. An active step has positive eligible exposure; record active-step count
and active model time separately. Record initial population, rates, geometry and the equality
basis: equal counts, equal rates, or both. Equal resources are not a kernel equality basis.

Check endings at initialization and after every completed settlement, in this order:

1. `double_extinction`: neither side has living actors; no winner.
2. `one_side_extinction`: exactly one side has living actors; identify surviving side.
3. `rate_zero`: contacts exist but neither side contributes positive exposure.
4. `horizon`: configured completed-step limit reached with surviving exposed populations;
   record administrative censoring, not a proved stalemate or voluntary peace.

A single zero rate does not end the engagement if the opposing rate is positive. One-sided
extinction at the final permitted step takes precedence over horizon censoring. A failed
attempt is `invalid`, with the attempted step, reason and last completed prefix; it is not
a biological or military ending. `no_contact` is inapplicable in W1: either declared
geometry supplies contact whenever both populations are positive. An incorrectly constructed matching is invalid, not a
no-contact ending. A future geometry may define that ending in a separately versioned
design; it is never a fallback for kernel errors.

Provisional implementation limits, to review before coding:

- Initial counts: integers in `0..=4096` per side; total at most 8192. Stable IDs do not recycle.
- Seed: unsigned 64-bit integer. Rates finite, nonnegative; `dt` finite and strictly positive.
- `max_steps`: integer `1..=1_000_000`; `max_steps * dt` must remain finite. Additionally,
  `total_initial * max_steps <= 64_000_000`, checked without integer overflow, bounds
  worst-case actor-step work.
- Positive-rate inputs satisfy `max(b, r) * dt <= 0.1`. This limits per-source interval
  exposure, not aggregate victim hazard, and supplies no accuracy guarantee. High aggregate
  hazards must remain finite; reject overflow and flag saturation diagnostics.
- Do not retain all per-step frames in memory. Stream completed frames; retain at most 1024
  explicitly requested diagnostic frames. Reject an impossible retention request rather
  than silently dropping records. Casualty receipts are bounded by initial population.
- A runner failure preserves completed receipts and records partial output explicitly.
  Set an output-size limit in the prospective protocol; do not silently truncate an archive.

A registered measurement protocol must separately declare parameter arms, seed population,
timestep refinement grid, ensemble size, numerical acceptance criteria, output budget and
administrative horizons. These design bounds are not such a registration.

## Records and accounting

Each completed frame records input identity, initial condition, step-start counts, geometry,
exposure rates, active/calendar clocks, casualty IDs/sides, survivor counts, ending and
censoring. Emit an actor's casualty ID once, on its death frame; routine frames do not
duplicate complete living-ID lists. Checkpoints explicitly retain living IDs.
A casualty is attributed to opposing aggregate exposure, not a fabricated individual killer.
Contributor exposure can be recorded optionally; it must not affect the RNG or outcome.

The pure kernel has no sugar, possessions or resource destruction; use unavailable resource
fields, not invented zero-valued economic measurements. The C path reports existing World
stores, loot and death causes as separate observations. Existing ordinary events do not
expose complete harvest or removed-wealth flows; those quantities remain unavailable,
rather than being inferred from a net wealth change. If a later adapter
freezes possessions from World, label them diagnostic frozen stores, record zero movement/
transfers/destruction by construction, and debit a dead actor's possessions to an explicit
removal sink. Removal loss is not combat resource destruction or a casualty count.

Existing World `Death` records ID, tribe and cause. Its `Kill` means site capture and loot,
which reciprocal fire does not imply. A later adapter needs distinct casualty receipts,
not synthetic C kills. Simultaneous deaths with inheritance would require an explicit batch
inheritance policy; W1 avoids that issue by omitting the enabled adapter.

## RNG, validation and tests

Use the existing seeded portable `SimRng`. Derive two u64 seeds with FNV-1a64: start from
fixed offset basis `14695981039346656037`, process the base seed's eight little-endian
bytes followed by ASCII `war1-contact-v1` or `war1-casualty-v1`; for each byte xor it into
the state then multiply by `1099511628211` with u64 wrapping arithmetic. Pass each derived
seed to the existing seeded RNG constructor. Streams and tags are protocol data.

For duel matching, begin with ascending living IDs on each side, shuffle blue then red
using the contact stream and the existing locked rand shuffle implementation, and zip the
first `min(B, R)` entries. Aimed geometry consumes no contact draws. For casualty trials,
visit ascending living blue IDs then ascending living red IDs, drawing only for strictly
positive hazard. Set `u = (next_u64() >> 11) / 2^53` with floating-point division and settle
a casualty iff `u < p`. For an ideal probability `p`, the realized probability is
`q = ceil(p * 2^53) / 2^53`, with `0 <= q - p < 2^-53`. Reject `0 < p < 2^-53` as
`invalid_numeric`: these probabilities fall below the sampler's resolution. The absolute
quantization bound supplies no relative-accuracy guarantee. All probabilities are computed
before trials or removal. Changing stream derivation, RNG algorithm, shuffle source or
draw ordering changes the protocol
version; do not quietly reuse an earlier identifier.

Each attempted step computes a private candidate state and copies both RNG streams,
including the contact stream used for matching. Commit state and both streams only after
validation succeeds. An invalid attempt retains the original state/RNG and last completed
prefix; diagnostic failure details do not become a completed step.

Save both streams, step, living IDs, config and ending in a checkpoint. Round-trip restoration
must reproduce continuation exactly. No observer uses either semantic stream.

A side swap generally changes draw allocation. Require mathematical side-swap equivalence
and distributional finite symmetry; do not promise arbitrary same-seed trajectory equality.
A purpose-built coupled permutation fixture can test settlement invariance with supplied
outcomes independently of RNG.

Required behavior checks:

- Exact disabled dispatch against same-seed World trajectories, RNG continuation and the
  existing combat golden fingerprints; no edits to existing golden expectations.
- Exposure budgets and probability boundaries: empty sides, zero rates, finite validation,
  tiny positive hazards preserved without cancellation; mathematical probability helper
  checks below sampler resolution, runner rejection below resolution, and a valid tiny
  hazard around `1e-12`; positive-product underflow rejected,
  saturation diagnostics, no self/friendly contacts and each casualty settled once.
- A supplied mutual-lethal pair retains both eligible contributions and has no ID-order
  advantage; side swapping supplied matching/draw outcomes mirrors settlement.
- Random matching has disjoint pairs and capacity `min(B, R)`; unmatched actors survive
  that step absent another cause, since W1 supplies no other causes.
- Independent analytic symmetry and both weighted invariants; no negative-force continuation.
- Correct first-step clocks, final-step extinction versus censoring, invalid matching,
  checked work-budget bounds, failed-attempt prefix,
  diagnostic independence, checkpoint continuation and ordered seed outputs.
- Prospective ensemble/timestep comparisons against references, with tolerances declared
  before measurements; these tests do not prove universal historical fit.

A continuous-time count Markov process would be a useful later numerical cross-check for
homogeneous populations. It is not needed in W1's first implementation: a count-only
implementation lacks actor-ID receipts, although ID decoration is possible. Its sequential
events do not exercise the selected synchronous-settlement semantics. Do not add it merely
to increase the number of mechanics.

## Existing patterns and review boundary

- [World step](../../../crates/sugarscape-core/src/world.rs),
  [combat C](../../../crates/sugarscape-core/src/rules/combat.rs), and
  [golden tests](../../../crates/sugarscape-core/tests/golden.rs): literal baseline.
- [Behavior-tree runner](../../../crates/sugarscape-core/src/minds/behavior_tree/runner.rs):
  headless World-backed research, observer isolation, completed prefixes and accounting.
- [Democratic-peace world](../../../crates/sugarscape-core/src/democratic_peace/world.rs):
  candidate engine/RNG state validated before a completed period commits.
- [Survey runner](../../../survey/src/runner.rs) and
  [behavior-tree archive](../../../survey/src/claims/behavior_trees/archive.rs): ordered seeds,
  explicit failed receipts and provenance. Reuse their conventions without coupling W1
  to their model-specific schemas.

The user approved this design for implementation planning on 2026-10-09. Review the written
implementation plan before execution. Implementation, measurement registration,
calibration, archive generation and browser integration each remain unapproved here. No
new simulator behavior, casualty measurements, war-duration results or WASM optimizations
are claimed by this specification.
