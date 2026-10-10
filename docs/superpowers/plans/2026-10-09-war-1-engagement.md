# W1 Reciprocal Engagement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user has already selected subagent-driven execution.

**Goal:** Build a native engagement benchmark with exact literal-World controls, independently checked continuous references, reproducible finite casualties and bounded streamed records.

**Architecture:** A default-off `war-benchmarks` core module owns the finite engine and a separate reference calculator. A headless runner delegates the book control directly to `World::step`; a dedicated survey binary supplies strict input, create-only output and compiled-source receipts. W1 adds no browser model, exports or payload.

**Tech Stack:** Existing Rust, serde/serde_json, rand/rand_pcg, locked `libm = "=0.2.16"`, and survey's existing sha2; Cargo tests and existing format/lint tools.

**Spec:** [Approved W1 design](../specs/2026-10-09-war-1-engagement-design.md).

**Status:** Proposed implementation plan for review. Design approval authorizes this plan's preparation; execution awaits plan review. All five tasks are Not Started. This plan delivers engineering readiness, not scientific campaign completion.

## Global Constraints

- Preserve existing ordinary Config, World scheduling, Model registration, WASM exports, browser code, golden expectations, retained fixtures and scientific archives.
- The core module is guarded by `all(feature = "war-benchmarks", not(target_arch = "wasm32"))`. Both core and survey features default off; survey forwards its local feature to the core feature.
- Initial counts: integers in `0..=4096` per side; total at most 8192. Stable IDs do not recycle.
- Seed: unsigned 64-bit integer. Rates finite, nonnegative; `dt` finite and strictly positive.
- `max_steps`: integer `1..=1_000_000`; `max_steps * dt` must remain finite. Additionally, `total_initial * max_steps <= 64_000_000`, checked without integer overflow.
- Positive-rate inputs satisfy `max(b, r) * dt <= 0.1`; aggregate victim exposure is not thereby bounded to 0.1.
- Use stable `-libm::expm1(-h * dt)`. Reject nonfinite arithmetic, positive-product underflow and `0 < p < 2^-53`; retain saturation diagnostics.
- Use the spec's FNV-1a64 seed derivation, tags `war1-contact-v1` / `war1-casualty-v1`, locked shuffle and 53-bit uniform comparison. Realized probability is `q = ceil(p * 2^53) / 2^53`, not an exact arbitrary real `p`.
- Snapshot eligibility, validate candidate state/RNG copies, then settle both sides together. Invalid attempts preserve the last completed state and both streams.
- End precedence: double extinction, one-side extinction, both rates zero, horizon censoring. No-contact is inapplicable to either W1 geometry; malformed matching is invalid.
- Stream completed frames; retain at most 1024 explicitly requested diagnostic frames. Emit each casualty ID on its death frame once; do not repeat living-ID lists in routine frames.
- Dimensionless time, continuous force, finite individuals, contact counts, exposure budgets and resource measurements remain distinct. No fabricated shots, lethal attackers, geography, harvest flows or economic zeros.
- No new dependency or global portable-math rewrite. Keep W1 math inside its gated module; existing `portable::exp_neg` and its consumers remain unchanged.
- Only fixed engineering cases and prescribed software checks run during implementation. Scientific parameter arms, seed population, ensemble/refinement tolerances, output budget and horizon need a separate prospective protocol and approval before collection. Do not invoke historical collection/analysis commands or overwrite historical archives; existing CI test semantics remain unchanged.
- Tests first, normal hooks, working incremental commits; stop after three failed attempts on one issue and record/reassess. Never disable tests or change old goldens to fit new behavior.

## Review Focus

1. Tiny positive probabilities, signed zero, saturation and overflow: preserve the declared distribution or reject with field/operand context; never turn arithmetic failure into peace — Tasks 1–2.
2. Mutual deaths, bad matching and errors after contact draws: preserve both eligible contributions and roll back both streams on invalid attempts — Task 2.
3. Continuous asymptotes, rate-zero branches and crossing contact forces: floating underflow is unavailable reference arithmetic, not extinction; mathematical failures cannot overwrite valid finite outcomes — Task 3.
4. Literal-C observation and diagnostic requests after early ending: events/RNG stay unchanged; unavailable requested frames carry a reason rather than disappearing — Task 4.
5. Duplicate input fields, oversized records, occupied output paths and torn writes: refuse ambiguous input/overwrite, retain completed and persisted clocks separately, and never emit success for a partial artifact — Task 5.

## File and interface map

Paths abbreviated `war/` mean `crates/sugarscape-core/src/war/`.

| Task | Files | Responsibility |
|---|---|---|
| 1 | core Cargo.toml/lib.rs; war/mod.rs, config.rs, math.rs, config_tests.rs, math_tests.rs | Native feature boundary, validated config, probability and seed primitives |
| 2 | war/engine.rs, checkpoint.rs, records.rs, engine_tests.rs, checkpoint_tests.rs | Finite actors, atomic settlements, clocks and resumable semantic state |
| 3 | war/reference.rs, reference_tests.rs | Independent continuous references, exact invariant classification and numerical availability |
| 4 | war/runner.rs, runner_tests.rs; config.rs/records.rs additions | Named World/graph paths, observation, typed records and bounded capture |
| 5 | survey Cargo.toml/build.rs; survey/build_support/war1_source_identity.rs; survey/src/bin/war1.rs; survey/src/war1/{mod,wire,io,cli}.rs; survey/tests/war1_cli.rs; CI workflow; docs/war.md and study pointers | Native CLI, source binding, durable output, feature coverage and verified integration |

The survey binary includes its local module with `#[path = "../war1/mod.rs"] mod war1;`. It does not register W1 in survey's ordinary campaign dispatcher. Reuse patterns from [World](../../../crates/sugarscape-core/src/world.rs), [the behavior-tree runner](../../../crates/sugarscape-core/src/minds/behavior_tree/runner.rs), [democratic-peace atomic periods](../../../crates/sugarscape-core/src/democratic_peace/world.rs), [survey's runner](../../../survey/src/runner.rs), [create-only I/O](../../../survey/src/claims/behavior_trees/io.rs) and [compiled input binding](../../../survey/build_support/bt_source_identity.rs). Their model-specific schemas and private helpers are not imported into W1.

## Execution setup and five-stage tracker

After plan approval, use `superpowers:using-git-worktrees` to create an owned execution checkout at `~/.config/superpowers/worktrees/SugarScape/war-1-engagement`, branched from the approved plan commit. Do not execute in the user's main checkout or touch untracked `survey/out/foraging-shortcuts/`.

Create `IMPLEMENTATION_PLAN.md` in that checkout with five stages matching Tasks 1–5, each containing Goal, Success Criteria, Tests and Status. Preserve a completed evidence snapshot before removing the owned tracker at delivery. Maintain an ignored ledger at `.superpowers/sdd/2026-10-09-war-1-engagement/` recording source/lock/toolchain/flags, commands/exits, RED/GREEN evidence, task commits and reviews. Read it before dispatch and skip completed tasks.

After approval, keep this plan and the approved spec as immutable execution inputs. Track completed checkboxes/stages in the execution tracker and ledger, not by editing these source-bound documents after builds. A necessary design/plan amendment is a named reviewed change requiring a new stamp/build and affected checks. Living readiness notes are separate from the compiled input closure.

Use fresh implementation and independent review agents per task, then a fresh whole-branch reviewer. Do not ask for the execution method again. Resolve substantive findings before dependent tasks; root performs coordination and self-review.

Preserve existing outputs and scientific environments. Bind a reusable baseline to actual source/locks/toolchain/flags; otherwise run an unchanged baseline once. Do not repeat an unchanged broad baseline per task. Use each task's targeted checks after its changes, then final broad gates. Native root and survey are separate Cargo workspaces; use the survey manifest explicitly. Build the WASM bundle before browser tests, as CI does.

## Task 1: Native boundary, config and numerical primitives

**Goal:** An independently testable opt-in configuration/probability library.
**Success Criteria:** Default and WASM builds exclude the module; validated scalar inputs, seed streams and probabilities follow the approved rules.
**Tests:** Config boundary table, checked work limit, tiny hazard/quantization, overflow/underflow, stream derivation and zero-draw rules.
**Status:** Not Started.

**Files:** Create war/mod.rs, config.rs, math.rs, config_tests.rs, math_tests.rs; modify core Cargo.toml and src/lib.rs only for feature/module declaration.

**Interfaces produced:**

```rust
// config.rs: serde snake_case enums; structs deny unknown fields.
pub const RULES_ID: &str = "war1-engagement-v1";
pub const CHECKPOINT_SCHEMA: &str = "war1-checkpoint-v1";
pub const RECORD_SCHEMA: &str = "war1-records-v1";
pub enum Side { Blue, Red }
pub enum Geometry { AimedFire, DuelContact }
pub struct EngagementConfig {
    pub blue: u32, pub red: u32,
    pub blue_rate: f64, pub red_rate: f64,
    pub dt: f64, pub max_steps: u64, pub geometry: Geometry,
}
impl EngagementConfig {
    pub fn validate(&self) -> Result<(), Vec<crate::config::FieldError>>;
}
// math.rs: public pure primitives within the gated war module.
pub struct NumericIssue { pub field: String, pub detail: String }
pub struct Probability {
    pub hazard: f64, pub dose: f64,
    pub ideal: f64, pub realized: f64, pub saturated: bool,
}
pub fn probability(hazard: f64, dt: f64) -> Result<Probability, NumericIssue>;
pub fn derive_seed(seed: u64, tag: &[u8]) -> u64;
pub fn uniform53(word: u64) -> f64;
pub fn chance(word: u64, probability: &Probability) -> bool;
```

These signatures are contracts, not a skeleton to commit with empty bodies. Numeric errors store contextual text so a nonfinite operand cannot make an error receipt itself unserializable. Side/Geometry derive Clone/Copy/Debug/PartialEq/Eq/serde; configs, NumericIssue and probability records derive Clone/Debug/PartialEq/serde as appropriate. Keep ordinary Config unchanged.

- [ ] **Step 1: RED the boundary and numerical behaviors.** Start with actual assertions such as:

```rust
#[test]
fn valid_tiny_hazard_retains_probability() {
    let p = probability(1e-12, 1.0).unwrap();
    assert!((p.ideal - 9.999_999_999_995e-13).abs() < 1e-27);
}
#[test]
fn below_sampler_resolution_is_contextual_error() {
    assert_eq!(probability(1e-18, 1.0).unwrap_err().field, "probability");
}
#[test]
fn maximum_uniform_is_less_than_one() {
    assert_eq!(uniform53(u64::MAX), 1.0 - 2.0_f64.powi(-53));
}
```

Also test the underlying `libm::expm1` helper below resolution before runner rejection; rates -0.0 behave as zero. Table-test counts 0/4096/4097, NaN/infinity/negative rates, zero/negative dt, max_steps 0/1/1_000_000/1_000_001, 64-million work boundary, and checked products. Numeric functions reject overflow and positive-product underflow; dose 400 yields saturation diagnostics. Verify `q` bounds and comparisons at uniform 0, threshold and maximum. Independent seed-vector fixtures: seed 7 plus contact tag derives 16934061643845198696; the casualty tag derives 14728233557300059986.

- [ ] **Step 2: Wire test discovery and run RED.** Add only the feature/module/test declarations shown below so the authored tests reach Rust compilation. `cargo test -p sugarscape-core --features war-benchmarks war::` must then fail for the missing W1 API or authored assertion, with receipt recorded. A misspelled command or absent Cargo feature is not a valid RED receipt.
- [ ] **Step 3: Implement the feature and primitives.** Add the native boundary:

```toml
[features]
default = []
war-benchmarks = []
```

```rust
#[cfg(all(feature = "war-benchmarks", not(target_arch = "wasm32")))]
pub mod war;
```

Use the following probability algorithm, preserving explicit validation around it:

```rust
let dose = hazard * dt;
let ideal = -libm::expm1(-dose);
let resolution = 2.0_f64.powi(-53);
// Nonfinite arithmetic and positive-product underflow are rejected first.
if ideal > 0.0 && ideal < resolution {
    return Err(NumericIssue {
        field: "probability".into(), detail: "below 53-bit resolution".into(),
    });
}
let realized = libm::ceil(ideal / resolution) * resolution;
```

Zero hazard returns positive-zero probability without a draw; other nonfinite/out-of-range results fail. `derive_seed` uses offset 14695981039346656037 and wrapping prime 1099511628211 over seed little-endian bytes then the tag. `uniform53 = (word >> 11) as f64 / 9007199254740992.0`; `chance` compares it to ideal p, whose realized probability is q. Do not replace it with rand's different Bernoulli sampler or modify existing portable functions.
- [ ] **Step 4: GREEN and boundary checks.** Run the targeted tests and `cargo check -p sugarscape-core --no-default-features`; also `cargo check -p sugarscape-core --target wasm32-unknown-unknown --features war-benchmarks`. Inspect cfg to verify war is absent on wasm32 even with the feature requested. Run cargo fmt and targeted clippy with the feature.
- [ ] **Step 5: Review and commit.** Fresh reviewer checks numerical guards and source exclusion. Commit `feat(war): add native engagement configuration and numeric primitives` with the reason for the feature/probability boundary.

## Task 2: Finite engine, simultaneous deaths and checkpoints

**Goal:** Reproducible bounded engagements with atomic semantic steps.
**Success Criteria:** Geometry/exposure, mutual deaths, end precedence and restoration work without economic or shot semantics.
**Tests:** Supplied-outcome pairs, matching invariants, clocks, q expectations, invalid-attempt rollback and checkpoint field mutations.
**Status:** Not Started.

**Files:** Create war/engine.rs, checkpoint.rs, records.rs, engine_tests.rs, checkpoint_tests.rs; export their public interfaces from war/mod.rs.

**Consumes:** EngagementConfig, Side, Geometry, Probability, NumericIssue and math primitives from Task 1.
**Produces:**

```rust
pub enum EndReason { DoubleExtinction, OneSideExtinction, RateZero, Horizon }
pub struct Ending { pub reason: EndReason, pub winner: Option<Side>, pub censored: bool }
pub struct Casualty { pub id: u32, pub side: Side }
pub struct Exposure {
    pub contact_pairs: u64, pub exposed: [u32; 2],
    pub contributed_rate: [f64; 2], pub integrated: [f64; 2],
    pub target_probability: [Option<Probability>; 2],
}
pub struct Frame {
    pub step: u64, pub start: [u32; 2], pub survivors: [u32; 2],
    pub active_steps: u64, pub calendar_time: f64, pub active_time: f64,
    pub exposure: Exposure, pub casualties: Vec<Casualty>, pub ending: Option<Ending>,
}
pub struct StepFailure { pub attempted_step: u64, pub issue: NumericIssue }
pub struct Checkpoint {
    pub schema: String, pub config: EngagementConfig, pub seed: u64,
    pub step: u64, pub active_steps: u64,
    pub blue_alive: Vec<u32>, pub red_alive: Vec<u32>,
    pub contact_rng: String, pub casualty_rng: String, pub ending: Option<Ending>,
}
pub(super) struct ValidatedCheckpoint {
    pub(super) wire: Checkpoint,
    pub(super) contact_rng: crate::rng::SimRng,
    pub(super) casualty_rng: crate::rng::SimRng,
}
pub(super) fn validate_for(saved: &Checkpoint, expected: &EngagementConfig, seed: u64)
    -> Result<ValidatedCheckpoint, Vec<crate::config::FieldError>>;
pub struct Engagement {
    config: EngagementConfig, seed: u64, step: u64, active_steps: u64,
    blue_alive: Vec<u32>, red_alive: Vec<u32>,
    contact_rng: crate::rng::SimRng, casualty_rng: crate::rng::SimRng,
    ending: Option<Ending>,
}
impl Engagement {
    pub fn new(config: EngagementConfig, seed: u64) -> Result<Self, Vec<crate::config::FieldError>>;
    pub fn counts(&self) -> [u32; 2];
    pub fn ending(&self) -> Option<&Ending>;
    pub fn step(&mut self) -> Result<Option<Frame>, StepFailure>;
    pub fn checkpoint(&self) -> Checkpoint;
    pub fn restore(&mut self, saved: &Checkpoint) -> Result<(), Vec<crate::config::FieldError>>;
}
```

All two-element arrays are ordered Blue then Red. Stable global IDs are Blue `0..blue`, Red `blue..blue+red`; no recycling. Frame casualties carry cause `benchmark_exposure` in the runner envelope; geography, resources and individual killer are unavailable. Serialize checkpoint RNGs with existing `rng::state_json`, never convert their u128 state through serde_json::Value. Frame/Checkpoint use strict serde records; Engagement itself keeps RNG objects private.

Frame, Exposure, Casualty, Ending, EndReason, StepFailure and Checkpoint derive Clone/Debug/PartialEq/serde; typed RNG candidates remain private and do not derive wire serialization. Register sibling unit-test modules explicitly in war/mod.rs behind cfg(test), keeping production state fields private.

- [ ] **Step 1: RED settlement and checkpoint contracts.** Define a cfg(test), `pub(super)` helper `Engagement::step_supplied(&mut self, matching: &[(u32, u32)], words: &[u64]) -> Result<Option<Frame>, StepFailure>`, using the production candidate validation and settlement functions. Supplied words follow ascending Blue then Red exposed IDs; bad matching and an incorrect supplied word count fail. Its test does not change the public seed protocol.

```rust
#[test]
fn invalid_numeric_attempt_keeps_both_streams() {
    let config = EngagementConfig {
        blue: 2, red: 2, blue_rate: 1e-18, red_rate: 1.0,
        dt: 0.1, max_steps: 2, geometry: Geometry::DuelContact,
    };
    let mut engine = Engagement::new(config, 7).unwrap();
    let before = serde_json::to_vec(&engine.checkpoint()).unwrap();
    assert!(engine.step().is_err());
    assert_eq!(serde_json::to_vec(&engine.checkpoint()).unwrap(), before);
}
```

Supplied zero uniform words for a one-versus-one positive-rate pair must kill both actors in one completed step. The two-versus-two invalid example consumes matching draws on its candidate before failing and must leave the real contact stream unchanged. Test reversed supplied settlement traversal without suppressing either action, duplicates/bad sides/self contact, and incomplete matching with both populations alive. Tests cover one zero rate, both zero at initialization, empty sides, final-step extinction versus censoring, clocks starting at one, and no repeated casualty IDs. For `B=2,R=1,b=r=1,dt=.05`, aimed target hazards are Blue .5 / Red 2; duel hazards are 1 for matched targets and zero for unmatched actors.

- [ ] **Step 2: Run RED.** `cargo test -p sugarscape-core --features war-benchmarks war::engine_tests`; then checkpoint tests. Record the specific missing API/assertion failure.
- [ ] **Step 3: Implement an O(B+R) candidate step.** The load-bearing sequence is:

```text
if already ended: return Ok(None)
clone living vectors and BOTH RNG streams into candidate
duel: shuffle ascending Blue then Red vectors with contact stream, zip min lengths
aimed: compute hazards from counts without dense graph allocation
validate matching, exposure rates/doses/probabilities and candidate clocks
visit ascending living Blue IDs then Red IDs; positive hazard consumes one next_u64
collect all casualty IDs before removing anyone
validate unique IDs, exposure/count reconciliation and ending precedence
commit candidate state and both streams; return completed Frame
```

Contact pairs mean undirected pairs; exposed counts mean targets with positive incoming hazard. Contributed rates mean outgoing totals by source side: aimed `[b*B,r*R]`, duel `[b*M,r*M]`. Integrated rate is contributed rate times dt. All products must be finite. Initial ended states produce no settlement or RNG consumption. Horizon is censored; invalid numeric state is a failed attempted step and never stored as a terminal biological state.

Checkpoint validation rejects unknown schema, different seed/config bit patterns, malformed or oversized RNG text, duplicate/unsorted/out-of-range/mis-sided IDs, inconsistent endings, step beyond horizon and active_steps beyond step. Restore copies nothing into the live engine until all checks pass. Keep checkpoint wire types/validation in checkpoint.rs; implement Engagement::checkpoint/restore in engine.rs, where its private fields are accessible. The validator returns a `pub(super)` candidate with typed RNG objects and validated fields; it does not mutate Engagement. Checkpoint field validation is reproducibility checking, not authentication of historical execution.

- [ ] **Step 4: GREEN and independence.** Restore at steps 0, 1, after deaths and at terminal state; compare continuation frames and both RNG strings exactly. Test field mutation rejection and unchanged state after failed restore. Compare diagnostics toggles and seed execution order using only construction seeds 7/8. Verify conditional expected casualty quantities with both ideal p and realized q; do not run a new ensemble study.
- [ ] **Step 5: Review and commit.** Run targeted tests, format/clippy, fresh independent review. Commit `feat(war): settle finite engagements atomically with exact checkpoints`.

## Task 3: Independent continuous references

**Goal:** A mathematical oracle that does not reuse finite-kernel outcomes or imply historical calibration.
**Success Criteria:** Both laws, rate-zero cases, crossings and numerical failures have explicit independent results.
**Tests:** Closed-form fixtures, exact weighted-sign classification, both invariants, side exchange, finite boundary and asymptotic underflow.
**Status:** Not Started.

**Files:** Create war/reference.rs and reference_tests.rs; export the reference API. No solver dependency or modification of finite engine.

**Consumes:** EngagementConfig/Geometry and Side; locked libm. It does not consume Engagement, Frame, its RNG or probability output.
**Produces:**

```rust
pub enum ReferenceRegime { InitialExtinction, FiniteExtinction, Asymptotic, RateZero }
pub struct ReferencePoint {
    pub requested_time: f64, pub evaluated_time: f64,
    pub forces: [f64; 2], pub regime: ReferenceRegime,
    pub extinction_time: Option<f64>, pub at_boundary: bool, pub survivor: Option<Side>,
}
pub struct ReferenceFailure { pub field: String, pub detail: String }
pub fn reference_at(config: &EngagementConfig, time: f64)
    -> Result<ReferencePoint, ReferenceFailure>;
```

Validate time in `0..=max_steps*dt`; reject unrepresentable required intermediates, unresolved cancellation or positive force underflow. A reference failure is unavailable reference evidence; it does not invalidate an independently valid finite engagement. Stop evaluation at a finite boundary and retain requested versus evaluated time. Regime names classify the mathematical solution; at_boundary distinguishes actual evaluated extinction, and survivor is populated only there. Matched positive populations have no finite extinction boundary; RateZero describes the both-rates-zero constant reference, not the one-positive-rate branches.

- [ ] **Step 1: RED exact fixtures.** Use absolute tolerance `2e-12` for these small construction fixtures, independent of any future scientific fit criterion:

| Geometry | B0,R0,b,r | Query | Expected |
|---|---|---|---|
| Aimed | 1,1,1,1 | ln(2) | .5,.5; asymptotic |
| Aimed | 1,2,4,1 | ln(2)/2 | .5,1; weighted balance |
| Aimed | 3,4,1,1 | ln(7)/2 and later | 0,sqrt(7); same finite boundary |
| Aimed | 1,1,4,1 | ln(3)/4 | sqrt(3)/2,0 |
| Aimed | 1,3,2,0 | 1.5 | 1,0 |
| Duel | 1,1,2,1 | ln(2)/2 | .75,.5; asymptotic |
| Duel | 1,3,2,1 | ln(2) | .5,2 |
| Duel | 1,2,3,1 | ln(2)+ln(2)/3 | 5/12,.25; one crossing |
| Duel | 1,3,2,0 | 1+ln(2)/2 | 1,.5 |

Fixture configs use dt .01 and max_steps 1000 so queries fit the declared horizon and rate interval. Example assertion:

```rust
let c = EngagementConfig {
    blue: 3, red: 4, blue_rate: 1.0, red_rate: 1.0,
    dt: 0.01, max_steps: 1000, geometry: Geometry::AimedFire,
};
let p = reference_at(&c, libm::log(7.0) / 2.0).unwrap();
assert!((p.forces[1] - libm::sqrt(7.0)).abs() < 2e-12);
```

Also test t=0, initial empty sides, both rates zero, mirrored inputs, just before/at/after first extinction, unequal-rate equal-count duel selection at time zero, and a balanced large exponent whose floating force underflows without a finite extinction event.

- [ ] **Step 2: Run RED.** `cargo test -p sugarscape-core --features war-benchmarks war::reference_tests`.
- [ ] **Step 3: Implement the independent formulas.** Classify signs of `b*B0²-r*R0²` (aimed) or `b*B0-r*R0` (duel) exactly for initial binary64 rates and integer counts. Decode normal rate as 53-bit mantissa times `2^(exponent-1075)` and subnormal rate as fraction times `2^-1074`. Multiply mantissa by count/count² in u128, compare leading exponent first, then align only equal-leading products. Maximum aimed coefficient needs 77 bits. This avoids overflowing products or mistaking a rounded near-balance for exact balance.

For aimed positive rates, normalize by `m=max(b,r)` before forming `x=sqrt(b/m)*B0`, `y=sqrt(r/m)*R0`, S=x+y and D=x-y. Exact balance uses `B0*exp(-z),R0*exp(-z)` directly; `z=sqrt(b*t)*sqrt(r*t)`. Otherwise `z*=log(S/abs(D))/2`; stop at its finite t boundary. For pre-boundary force, use `(S*exp(-z) ± D*exp(z))/2` and undo the normalized weights. Compute the boundary survivor from normalized weights, not a potentially overflowing r/b ratio. If D is zero, has a sign inconsistent with the exact invariant, or `abs(D) <= 8*f64::EPSILON*S` for a nonzero exact imbalance, return ReferenceFailure rather than treating the rounded gap as a reliable magnitude. Handle one zero rate as constant force plus linear opposing decline to its boundary, and both zero as constants. Numeric arithmetic that cannot resolve boundary/forces is unavailable, never silently clipped.

For matched B<=R with r>0, use `B=B0*exp(-r*t)` and `R=R0-b*t*B0*phi(r*t)`, where `phi(u)=-expm1(-u)/u` and `phi(0)=1`. A crossing exists only for exact positive `b*B0-r*R0`; compute `f_c=(r/(b-r))*((R0-B0)/B0)`, `t_c=-log1p(-f_c)/r`, and restart once from equal crossing counts with Red smaller. Mirror for R<=B. At initial equality choose the future smaller side from rates. With one rate zero, use the linear segment to equal counts followed by the appropriate exponential; both zero stay constant. If branch arithmetic violates positivity or the crossing domain, report numerical failure. No recursive solver, finite-agent averages or arbitrary extinction threshold.

- [ ] **Step 4: GREEN and invariants.** Compare independent fixed-time finite differences to both ODEs away from boundaries and check weighted invariants with scale-aware tolerance. A `pub(super)` reference-only `invariant_sign(&EngagementConfig) -> Result<std::cmp::Ordering, ReferenceFailure>` helper allows the sibling reference_tests module to test exact integer/dyadic classification and near-balanced next-representable rates. Reject unavailable arithmetic explicitly; never present a forced zero as asymptotic extinction. Statistical finite-ensemble convergence remains a registered-study requirement, not an automatic CI campaign.
- [ ] **Step 5: Review and commit.** Fresh numerical reviewer checks branches/fixtures against the spec; targeted tests and lint pass. Commit `feat(war): add independent attrition reference calculations`.

## Task 4: Literal-C dispatch, records and bounded observation

**Goal:** A single headless runner with honest, source-identifiable World and graph records.
**Success Criteria:** Baseline trajectories/events/RNG match direct World execution; capture/output failures retain accurate clocks without changing model behavior.
**Tests:** C presets/seeds, event copies, observational independence, early-end capture, failed sinks and reference-unavailable records.
**Status:** Not Started.

**Files:** Create war/runner.rs and runner_tests.rs; extend config.rs/records.rs and exports. Do not modify World, rules, ordinary Config, Snapshot or existing golden tests.

**Consumes:** Tasks 1–3 APIs and actual `World::new`, `World::step`, `World::events`, `World::fingerprint`, `Snapshot::of`, `rng::state_json`.
**Produces:**

```rust
pub enum StudyInput {
    BookC { config: crate::config::Config, seed: u64, max_steps: u64 },
    ReciprocalGraph { config: EngagementConfig, seed: u64 },
}
pub struct Capture { pub retain_steps: Vec<u64> }
pub struct EqualityBasis { pub counts: Option<bool>, pub rates: Option<bool>, pub resources: Option<bool> }
pub struct RunHeader {
    pub input: StudyInput, pub equality: EqualityBasis,
    pub unavailable: Vec<UnavailableObservation>,
    pub force_unit: String, pub clock_unit: String,
}
pub enum ObservedFrame {
    Initial { counts: Option<[u32; 2]> },
    Graph { frame: Frame, reference: Option<ReferencePoint>, reference_error: Option<ReferenceFailure> },
    Book { frame: BookFrame },
}
pub enum RunPayload {
    Header { header: RunHeader },
    Observed { frame: ObservedFrame },
    Terminal { summary: RunSummary },
}
pub struct RunRecord { pub schema: String, pub input_identity: String, pub payload: RunPayload }
pub struct BookFrame {
    pub tick: u64, pub fingerprint: String, pub rng_state: String,
    pub snapshot: crate::stats::Snapshot, pub combat_enabled: bool,
    pub deaths: Vec<BookDeath>, pub kills: Vec<BookKill>,
    pub agent_stores: Vec<f64>, pub site_stores: Vec<f64>,
    pub unavailable: Vec<UnavailableObservation>,
}
pub struct UnavailableObservation { pub quantity: String, pub reason: String }
pub struct BookDeath { pub id: u64, pub tribe: String, pub cause: String }
pub struct BookKill { pub attacker: u64, pub victim: u64, pub loot: f64 }
pub enum CapturedFrame {
    Available { input_identity: String, frame: ObservedFrame },
    Unavailable { step: u64, reason: String },
}
pub struct RunSummary {
    pub completed_steps: u64, pub ending: Option<Ending>,
    pub capture: Vec<CapturedFrame>,
}
pub struct RunFailure {
    pub kind: String, pub detail: String, pub attempted_step: Option<u64>,
    pub completed_steps: u64, pub emitted_steps: u64,
    pub checkpoint: Option<Checkpoint>,
}
pub fn run_to(input: &StudyInput, capture: &Capture,
    emit: &mut dyn FnMut(&RunRecord) -> Result<(), String>)
    -> Result<RunSummary, RunFailure>;
```

RunPayload and ObservedFrame are tagged `kind`/`data`. StudyInput is Clone/Debug/Serialize; it is a resolved core input, not the CLI wire decoder. RunHeader, EqualityBasis, unavailable/event DTOs, RunRecord, BookFrame, ObservedFrame and capture/summary records derive Clone/Debug/Serialize, without Deserialize because existing Snapshot is Serialize-only. CapturedFrame retains only nonterminal ObservedFrames, so terminal summaries cannot recursively contain themselves. Book fingerprints use `0x` plus 16 lowercase hexadecimal digits. Graph unavailable observations are fixed metadata on the Header; Book unavailable fields use the explicit vector.

- [ ] **Step 1: RED baseline and failure behavior.** Inside core unit tests, compare direct World and captured Book frames for `iii-9-combat`, `iii-11-combat-fixed`, `iii-14-combat-culture`, seeds 7/8, over 32 ticks. Compare actual `Snapshot::of`, full World fingerprints, deaths/kills and `rng::state_json(&world.rng)` after every tick; RNG access belongs inside core, not the survey binary.

```rust
#[test]
fn literal_book_frame_has_direct_rng_and_fingerprint() {
    let config = crate::presets::by_id("iii-9-combat").unwrap().config;
    let mut direct = crate::world::World::new(config.clone(), 7).unwrap();
    direct.step();
    let input = StudyInput::BookC { config, seed: 7, max_steps: 1 };
    let mut records = Vec::new();
    run_to(&input, &Capture { retain_steps: vec![] }, &mut |r| {
        records.push(r.clone()); Ok(())
    }).unwrap();
    let book = records.iter().find_map(|r| match &r.payload {
        RunPayload::Observed { frame: ObservedFrame::Book { frame } } => Some(frame),
        _ => None,
    }).unwrap();
    assert_eq!((book.rng_state.clone(), book.fingerprint.clone()),
        (crate::rng::state_json(&direct.rng), format!("0x{:016x}", direct.fingerprint())));
}
```

Author a sink that fails on the first settlement and another that fails after a prefix. Assert completed_steps versus emitted_steps accurately distinguish a completed semantic frame from a failed external write. Test a terminal-at-zero graph capture `[0,1]`: initial is available, requested step 1 is unavailable with ending reason. Reject duplicate/out-of-horizon requests or >1024 captures before construction. Reference failure must produce an unavailable reference field while preserving the successful finite frame.

- [ ] **Step 2: Run RED.** `cargo test -p sugarscape-core --features war-benchmarks war::runner_tests`.
- [ ] **Step 3: Implement literal delegation and records.** Construct a World from the supplied Config/seed without mutating either. Call step once per requested tick and copy events immediately before their next reset. C control runs its requested horizon even after population reaches zero; kernel extinction rules do not alter World scheduling. Read actual combat_enabled, preserving disabled-combat controls without calling them C findings. No callbacks are added inside existing World actions.

Sum actual living holdings and site resources per configured good as stock observations. Copy existing kill loot/death cause facts. Ordinary events lack complete harvest/removal wealth or death-site positions: represent those quantities as unavailable with reasons in record metadata; do not infer them from net stock changes or fabricate a conservation decomposition. Graph resources/geography/killer are likewise unavailable. Book baseline has tick time, not the graph's dimensionless dt or an asserted battle-active clock.

Validate graph bounds and capture before construction. Book controls get runner resource bounds: max_steps in `1..=1_000_000`, at most 4096 sites and at most 64 million site-ticks, checked without overflow; accepted Configs still execute exactly through World. Input identity is a versioned FNV checksum over ordered serde serialization of resolved StudyInput, encoded as hexadecimal text; it is not cryptographic authentication. Raw-file/source/binary SHA256 is the survey layer's responsibility.

Emit Header, one Initial observed record, every completed observed frame, and one Terminal record. Initial step 0 is distinct from settlements starting at 1. Header equality fields are defined for the graph's counts/rates; resources are unavailable, and book control equality is not inferred from its aggregate snapshot. The callback is observational: it receives immutable records and no RNG/engine handle. Keep only requested nonterminal ObservedFrames in memory. Early-ended future captures return Unavailable. On engine failure stop with its last checkpoint/prefix; on external failure record semantic and emitted clocks separately and return failure, never success. No retry or automatic continuation.

- [ ] **Step 4: GREEN and compatibility.** Run runner tests, feature-enabled golden tests and no-default-features golden tests. Compare enabled/default-disabled ordinary World behavior. Test requested captures and reference/diagnostics toggles do not change Graph frames or checkpoints. Test construction seed order changes cannot change a per-seed record.
- [ ] **Step 5: Review and commit.** Fresh reviewer checks baseline observation, serialization and all unavailable fields. Commit `feat(war): add literal World controls and bounded headless records`.

## Task 5: Native CLI, source binding and verified delivery

**Goal:** A usable native-only engineering-case interface whose failures and provenance are reviewable.
**Success Criteria:** Strict bounded input, create-only synchronized outputs, feature/default CI and documentation pass; no scientific campaign has run.
**Tests:** Real binary help/validation/run, duplicate/unknown fields, bounds, preexisting paths, sink/flush failures, source-stamp determinism and saved-record integrity.
**Status:** Not Started.

**Files:** Modify survey Cargo.toml and build.rs; create survey/build_support/war1_source_identity.rs, survey/src/bin/war1.rs, survey/src/war1/{mod,wire,io,cli}.rs, survey/tests/war1_cli.rs. Begin the integration test file with `#![cfg(feature = "war-benchmarks")]` so default survey builds do not expand an unavailable CARGO_BIN_EXE_war1 macro. Modify CI workflow using yq, add docs/war.md, update living war pointers and the execution tracker. Preserve all existing build stamp selection/logic and historical study assets; current-build digests can legitimately change as their existing selected source tree changes.

**Consumes:** Task 4 runner/records and existing sha2.
**Produces:**

```rust
// wire.rs: strict typed serde input after recursive duplicate-field rejection.
pub enum Request {
    BookC { preset: String, seed: u64, max_steps: u64, capture_steps: Vec<u64> },
    ReciprocalGraph { config: EngagementConfig, seed: u64, capture_steps: Vec<u64> },
}
pub fn reject_unrepresentable_numbers(bytes: &[u8]) -> Result<(), String>;
pub fn decode(bytes: &[u8]) -> Result<Request, String>;
pub fn resolve(request: Request) -> Result<(StudyInput, Capture), String>;
// io.rs
pub fn read_input(path: &std::path::Path) -> Result<Vec<u8>, String>;
pub fn create_output_directory(path: &std::path::Path) -> Result<(), String>;
pub(crate) trait DurableWrite: std::io::Write {
    fn sync_all(&mut self) -> std::io::Result<()>;
}
// Implement DurableWrite for File via File::sync_all; inject fake writers in unit tests.
pub(crate) struct Journal<W: DurableWrite = std::fs::File> {
    writer: W, digest: sha2::Sha256, bytes: u64, persisted_steps: u64,
}
pub(crate) struct JournalReceipt {
    pub sha256: String, pub bytes: u64, pub acknowledged_steps: u64,
}
impl Journal<std::fs::File> {
    pub(crate) fn new(path: &std::path::Path) -> Result<Self, String>;
}
impl<W: DurableWrite> Journal<W> {
    pub(crate) fn append(&mut self, record: &RunRecord) -> Result<(), String>;
    pub(crate) fn acknowledged(&self) -> JournalReceipt;
    pub(crate) fn finish(&mut self) -> Result<JournalReceipt, String>;
}
// cli.rs
pub fn main_args(args: &[String]) -> Result<(), CliFailure>;
pub struct CliFailure { pub exit_code: i32, pub message: String }
// build_support/war1_source_identity.rs (shared only by W1 build/test modules)
pub fn fingerprint(entries: impl IntoIterator<Item = Result<(String, Vec<u8>), String>>)
    -> Result<String, String>;
```

The CLI exposes `war1 validate --input FILE` and `war1 run --input FILE --out DIRECTORY`, plus help. No batch, training, scientific collect, retry, overwrite or resume command. Request is internally tagged by mode, denies unknown fields, rejects duplicates recursively and trailing JSON. Book wire resolves an existing preset without importing or rewriting Config's legacy decoder. Headers retain the fully resolved Config and actual combat flag.

- [ ] **Step 1: RED the CLI and source stamp.** Use actual `env!("CARGO_BIN_EXE_war1")` integration tests. The engineering example is:

```json
{"mode":"reciprocal_graph","config":{"blue":2,"red":2,"blue_rate":1.0,"red_rate":1.0,"dt":0.05,"max_steps":8,"geometry":"aimed_fire"},"seed":7,"capture_steps":[0,1,8]}
```

Test seed u64::MAX remains exact; known/unknown/duplicate fields, floating or negative seed, trailing objects, nonexistent preset, a requested step beyond horizon, and >1 MiB input all fail before output creation. Book example uses `iii-9-combat`, seed 8, max_steps 4. A reused output directory must remain byte-identical after refusal. Inject short writes, flush/sync errors and output limits in Journal unit tests; status cannot be complete. Source hash is independent of enumeration order, changes on selected bytes, and rejects duplicate/missing required/symlink inputs.

Real binary tests follow this structure, using a canonicalized native temp parent before creating an owned unique directory:

```rust
#[test]
fn duplicate_seed_does_not_create_output() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let root = parent.join(format!("war1-cli-{}-duplicate", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let input = root.join("input.json");
    let output = root.join("out");
    std::fs::write(&input, br#"{"mode":"book_c","preset":"iii-9-combat","seed":7,"seed":8,"max_steps":1,"capture_steps":[]}"#).unwrap();
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_war1"))
        .args(["run", "--input"]).arg(&input).arg("--out").arg(&output)
        .output().unwrap();
    assert_eq!((result.status.code(), output.exists()), (Some(2), false));
    std::fs::remove_dir_all(root).unwrap();
}
```

- [ ] **Step 2: Run RED.** After adding the survey-local feature/bin declarations, run `cargo test --manifest-path survey/Cargo.toml --features war-benchmarks --bin war1 --test war1_cli`. Missing module/API or assertions must fail; do not commit a broken intermediate.
- [ ] **Step 3: Implement feature, strict wire and output.** Survey feature forwards to `sugarscape-core/war-benchmarks`; `[[bin]]` uses `required-features = ["war-benchmarks"]`. main_args reports usage/validation exit 2, execution/output exit 1, success 0. Fixed input read limit is 1 MiB, checkpoint RNG string limit 16 KiB per stream, maximum JSONL record 2 MiB and engineering journal 128 MiB. These are engineering resource caps, not a scientific output registration.

```toml
[features]
default = []
war-benchmarks = ["sugarscape-core/war-benchmarks"]

[[bin]]
name = "war1"
path = "src/bin/war1.rs"
required-features = ["war-benchmarks"]
```

```rust
// survey/src/bin/war1.rs
#[path = "../war1/mod.rs"]
mod war1;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Err(failure) = war1::cli::main_args(&args) {
        eprintln!("{}", failure.message);
        std::process::exit(failure.exit_code);
    }
}
```

Use exhaustive argv matching for the two exact command forms and help. Tag Request with `mode`/snake_case and `deny_unknown_fields`; resolve book presets through `presets::by_id` without changing their config. No parser dependency is introduced.

Read bytes with a bounded reader and reject symlinks/nonregular files. Decode using a local strict visitor patterned after existing democratic-peace strict_json; do not deserialize through a Value that has already discarded duplicates. Reject nonzero numeric JSON literals that parse to zero or nonfinite binary64 before typed decoding: a string-aware numeric-token check excludes quoted text, examines mantissa digits before e/E, and uses f64 parsing only for representability, never for integer seed decoding. Test rate `1e-400` is an input error rather than rate_zero, while literal zero is accepted. Create the output directory and each file with create-new semantics; never pre-delete or rename over an existing destination. Write exact input.json bytes and metadata.json, stream frames.jsonl, flush and sync it before success.json, then synchronize the parent directory. Metadata records `purpose: engineering_case`, `registered: false`, raw input SHA, resolved input identity, source/binary/toolchain/flags and paths.

Journal serializes a record to bounded bytes before append, uses write_all, tracks acknowledged complete records separately from attempted writes, and propagates write/flush/sync errors. A frame is acknowledged only after its write/flush/sync succeeds; an interrupted final line remains an explicitly partial artifact. Both Journal and DurableWrite are crate-visible local implementation details. JournalReceipt binds only the acknowledged complete prefix; it is the full frames.jsonl digest only after successful finish. Core Terminal records describe semantic completion; only a synchronized success.json declares artifact completion. On failure preserve partial frames and attempt create-only failure.json with semantic/emitted/acknowledged clocks, acknowledged-prefix receipt and reason; if that write also fails, report both errors on stderr and exit 1. Never fabricate artifact success or silently truncate. Output limiting is independent from numeric engine failure.

Add a separately gated W1 build stamp without changing F5/BT selection or helpers. Select all regular files under core/src (excluding generated/evidence/cache directories), core Cargo.toml, root Cargo.toml/Cargo.lock, survey Cargo.toml/Cargo.lock/build.rs, the new W1 binary/local sources/stamp, approved spec and this plan. Sort normalized relative paths, reject symlinks/duplicates/missing required files, and hash domain `war1-compiled-inputs-v1\0` plus length-prefixed path/data. Record rustc -vV and inherited build flags; generate W1 constants only with the survey feature enabled. At CLI invocation hash current_exe separately and compare declared source inputs against the compiled stamp before running, so post-build source changes fail descriptively. Do not claim that declared-input SHA replaces binary identity or an approved prospective manifest.

- [ ] **Step 4: GREEN and CI.** Run CLI/unit/integration checks and verify `cargo check --manifest-path survey/Cargo.toml --no-default-features --bins` omits war1. Add a `war` CI job with the existing checkout/toolchain/cache actions and these commands:

```sh
cargo test -p sugarscape-core --locked --features war-benchmarks war::
cargo test -p sugarscape-core --locked --features war-benchmarks --test golden
cargo test --manifest-path survey/Cargo.toml --locked --features war-benchmarks --bin war1 --test war1_cli
cargo clippy -p sugarscape-core --all-targets --features war-benchmarks -- -D warnings
cargo clippy --manifest-path survey/Cargo.toml --bin war1 --test war1_cli --features war-benchmarks -- -D warnings
```

Use yq for the structured workflow edit and inspect the resulting diff. This feature needs explicit CI because ordinary workspace tests leave it off. Do not change existing job semantics.

The job's setup mirrors existing CI:

```yaml
war:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v7
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy
    - uses: Swatinem/rust-cache@v2
    - run: cargo test -p sugarscape-core --locked --features war-benchmarks war::
    - run: cargo test -p sugarscape-core --locked --features war-benchmarks --test golden
    - run: cargo test --manifest-path survey/Cargo.toml --locked --features war-benchmarks --bin war1 --test war1_cli
    - run: cargo clippy -p sugarscape-core --all-targets --features war-benchmarks -- -D warnings
    - run: cargo clippy --manifest-path survey/Cargo.toml --bin war1 --test war1_cli --features war-benchmarks -- -D warnings
```

Save only this job mapping in owned ignored evidence, then apply it with `yq -i '.jobs.war = load(".superpowers/sdd/2026-10-09-war-1-engagement/war-ci-job.yaml").war' .github/workflows/ci.yml` and inspect that only the new job changed.

- [ ] **Step 5: Final broad gates and preservation.** Run once at the reviewed implementation head, preserving exits/counts and ignored notices:

```sh
cargo fmt --all --check
cargo fmt --manifest-path survey/Cargo.toml --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --features war-benchmarks war::
cargo test -p sugarscape-core --features war-benchmarks --test golden
cargo test --manifest-path survey/Cargo.toml --features war-benchmarks --bin war1 --test war1_cli
cargo check --manifest-path survey/Cargo.toml --no-default-features --bins
cargo check -p sugarscape-core --target wasm32-unknown-unknown --features war-benchmarks
# web/, sequential: build generated WASM before testing
npm run build
npm test
# repository root
python3 -m unittest discover -s studio/tests -t studio
```

Bind any reused baseline checks to matching runtime/locks/toolchain/features. Record default WASM raw/compressed sizes and embedded fixture hashes before/after; W1 is excluded, while metadata/toolchain/source-stamp byte changes must be explained rather than suppressed. Verify historical public study trees and owned evidence hashes unchanged, and normal C goldens unchanged. Do not introduce WASM size optimizations as part of this plan.

- [ ] **Step 6: Documentation, whole-branch review and delivery.** docs/war.md explains exact native commands, two path semantics, unavailable quantities, q versus p, clocks/censoring, source binding, checkpoint limitations and engineering-only status. Update living study pointers to implemented engineering readiness, leaving scientific registration pending. Fresh whole-branch reviewer checks source/spec coverage, narrow scope, scientific boundaries, source binding and preserved baseline. Resolve findings, commit `feat(war): deliver native engagement runner and provenance checks`, and retain final verification receipts.

Integrate the reviewed branch normally into local main while preserving concurrent work; push normally and verify CI for the exact pushed SHA. No scientific assets or browser feature are published by this engineering delivery. CI/Pages automation may run as usual; no research findings are inferred from its success. Complete the five-stage tracker, preserve its snapshot and remove only owned temporary tracking files. Retain the execution/evidence worktree until unique artifacts have been archived.

## Plan self-review and approval boundary

Spec ownership: architecture/feature gates → Tasks 1/5; finite exposure/sampler/RNG → Tasks 1/2; mathematical references → Task 3; endings/clocks/checkpoints → Tasks 2/4; records/accounting/observation → Task 4; bounded native I/O/provenance → Task 5. All five Review Focus rows have owning tests. Signatures and serialization constraints are resolved in the task contracts.

The user approved the written design and already selected subagent-driven execution. This written implementation plan now requires review before execution. Approval of this plan permits engineering work and prescribed software verification; it does not approve an unregistered ensemble study, calibration, historical fit, publication of findings or a WASM optimization project.
