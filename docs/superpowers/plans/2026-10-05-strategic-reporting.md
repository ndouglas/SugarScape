# Strategic Reporting Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. A fresh implementer and fresh reviewer gate each task; independently review the complete increment before delivery.

**Goal:** Measure finite strategic reporting policies against frozen listeners with independently verified exact best responses and equal-budget search controls.

**Architecture:** Add `deduction::strategic_reporting` beside the frozen testimony game. Separate permissions, utility, reporter observations, legacy listener assumptions, exact evaluation, search, and transactional sessions. The evaluator may know private worlds; controllers receive only observations.

**Tech Stack:** Existing Rust, serde, rand, clap, and unittest infrastructure; independent Python standard-library `fractions.Fraction` references. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-05-strategic-reporting-design.md` (approved October 5, 2026).

## Global Constraints

- Existing capability, testimony and testimony-game rules, outputs, and interpretations remain unchanged.
- American spelling; Agent in code/docs; no thematic roles, UI, presets, videos, provider integration or arbitrary role language.
- One strategic reporter, one fixed-profile reporter, one decider; exactly the declared five-response schedule.
- No commits without explicit user instruction. Never stage `.claude/`, `papers/`, or `survey/out/`.
- Use existing crowd worktree; preserve unrelated work. Do not format survey or edit it for this increment.
- First collection is frozen; learning success is never a correctness gate. Disclose any subsequent rule revision as retrospective.
- Follow test-first implementation, retain actual red/green evidence, and reassess after three failed attempts per issue.

## Review Focus

1. Reject malformed probabilities, repeated IDs and unsupported permission assemblies before RNG or state allocation (Task 1).
2. Distinguish public disclosure of strategic incentives from legacy listeners' incorrect assumed channel; do not label their probabilities correct posteriors (Tasks 1–2).
3. Unreachable policy rows and duplicated encodings must not change best-response regret or hide search-budget differences (Tasks 2–3).
4. Invalid responses must leave every subsequent request/outcome unchanged, not merely the current request (Task 4).
5. Renaming IDs, zero-mass histories, endpoints, and exact ties must not introduce fabricated evidence or numerical decision errors (Tasks 1–2 and 4).

## Files and handoff interfaces

Create under `crates/sugarscape-core/src/deduction/strategic_reporting/`:

- `mod.rs`: exports, Error and version constants.
- `types.rs`: strict config, utility, observations and outcomes.
- `policy.rs`: finite reporting and named controls.
- `listeners.rs`: explicit adapters for existing frozen algorithms.
- `enumeration.rs`: private worlds, exact masses, histories and reference inference.
- `best_response.rs`: exact optimization within the finite family.
- `evolution.rs`: immutable training evaluator, GA/random search.
- `session.rs`: private world generation, request delivery and replay.
- `diagnostics.rs`: fixed collection, report and integrity checks.
- `tests/{mod,atoms,enumeration,best_response,evolution,session}.rs`: behavior tests.

Modify `crates/sugarscape-core/src/deduction/mod.rs` to export the namespace; modify `crates/sugarscape-cli/src/deduction.rs` only to add the diagnostic mode. Create `crates/sugarscape-cli/tests/strategic_reporting.rs`. Extend `docs/deduction.md` after first collection.

Use existing `AgentId`, `FieldError`, `testimony_game::{Probability, Permissions, Profile, DecisionAction, Listener, Genome}` where their meaning is identical. Do not reuse the old Config as this game's public rules. Declare the new assembly separately:

```rust
pub struct UtilityTable { pub intervene_true: i8, pub intervene_false: i8,
                          pub abstain_true: i8, pub abstain_false: i8 }
pub struct Config {
    pub version: u16, pub strategic: AgentId, pub fixed: AgentId,
    pub decider: AgentId, pub permissions: Vec<Permissions>,
    pub accuracy: Probability, pub fixed_copy_prior: Probability,
    pub strategic_utility: UtilityTable,
}
pub struct Policy { pub bits: u32 }
pub struct StrategicObservation {
    pub rules: Config, pub signal: bool,
    pub calibration_signal: Option<bool>,
    pub calibration_reports: Option<[bool; 2]>,
    pub calibration_truth: Option<bool>,
}
pub struct FixedObservation {
    pub rules: Config, pub signal: bool, pub profile: Profile,
    pub calibration_reports: Option<[bool; 2]>,
    pub calibration_truth: Option<bool>,
}
pub struct DecisionObservation {
    pub rules: Config, pub calibration_reports: [bool; 2],
    pub calibration_truth: bool, pub live_reports: [bool; 2],
}
pub struct FrozenListener {
    pub algorithm: Listener, pub assumed_copy_prior: Probability,
}
```

All public input structures and variants reject unknown fields. Fields above are public; internal worlds and candidate-ranking state remain private or crate-visible. Use fieldless-action custom decoding as in testimony-game. Version and protocol constants are 1 under a distinct namespace.

Utility entries are integers in -1..=1, validated independently of permissions. Decision payoff is fixed by truth/action. Export `UtilityTable::aligned()` and `::opposed()`; both abstain entries zero. Define `utility(&self, truth: bool, action: DecisionAction) -> i8` and pure `decision_payoff(truth, action) -> i8`.

## Preparation before Task 1

- [x] Confirm crowd and main ancestry/status; preserve all existing uncommitted design/plan files and unrelated work. Create per-plan ignored scratch and the temporary stage tracker.
- [x] Build and retain the pre-change CLI executable and capture `deduction diagnose`, `deduction testimony`, `deduction testimony-game`, and `deduction run --scenario wink --seed 7 --policy evidence`. Hash the executable/source snapshot and save the existing guide prefix. This occurs before any implementation, so Task 4 can make genuine before/after comparisons.

## Task 1: Permissions, finite policies, and frozen listener adapters

**Files:** Create mod/types/policy/listeners and tests/atoms; export module from deduction/mod.rs.

**Produces:** `Config::validate() -> Result<(), Error>`, `Policy::new(bits: u32) -> Result<Self, Error>`, `Policy::calibration(signal: bool) -> bool`, `Policy::live(c_signal: bool, c_report: bool, c_truth: bool, t_signal: bool) -> bool`, `Policy::report(&StrategicObservation) -> Result<bool, Error>`, `FrozenListener::decide(&DecisionObservation) -> Result<DecisionAction, Error>`.

- [x] Write failing tests covering every permission entry, missing/excess entries, repeated IDs, version mismatch, numerator above denominator, denominator zero/17, utility entry outside -1..1, policy bits at 2^18, and unknown serde fields.

```rust
#[test]
fn reporting_permission_does_not_determine_utility() {
    let aligned = UtilityTable::aligned();
    let opposed = UtilityTable::opposed();
    assert_eq!(aligned.utility(false, DecisionAction::Intervene), -1);
    assert_eq!(opposed.utility(false, DecisionAction::Intervene), 1);
}
#[test]
fn policy_rejects_nineteenth_bit() {
    assert!(Policy::new(1 << 18).is_err());
}
```

- [x] Run `cargo test -p sugarscape-core strategic_reporting::tests::atoms`; retain the actual failing output.
- [x] Implement canonical bit indexing. Calibration output is bit `u32::from(signal)`. Live row is `8*c_signal + 4*c_report + 2*c_truth + t_signal`, and its output is bit `2 + row`. Make copy, invert, positive, negative and calibration-copy/live-invert controls via these same row rules; no privileged truth input.

```rust
let row = (u32::from(c_signal) << 3) | (u32::from(c_report) << 2)
        | (u32::from(c_truth) << 1) | u32::from(t_signal);
let report = ((self.bits >> (2 + row)) & 1) != 0;
```

- [x] Validate exactly three distinct participants and their three permission entries: strategic/fixed receive/report/observe verification but cannot decide; decider observes verification/decides but does not receive/report. Invalid public observations fail descriptively, rather than defaulting missing live calibration fields to false.
- [x] Adapter constructs an old `testimony_game::DecisionObservation` using the explicit assumed prior and q; pass actual history unchanged. Serialize the assumption separately from new generative rules. Do not expose a legacy assumed probability as an actual-policy posterior.
- [x] Test all 16 live rows with single-bit policies; two calibration rows; malformed partial observations; ID renaming; invariance when the other calibration report changes for this restricted policy family. Verify adapter actions against direct old listener calls for every Boolean public history and q/rho endpoints.
- [x] Run scoped tests and workspace formatting check; fresh task reviewer checks permissions, representation restriction and assumption labeling. Keep work uncommitted.

## Task 2: Exact evaluation, independent references and best responses

**Files:** Create enumeration/best_response, tests/enumeration and tests/best_response. Create ignored `.superpowers/sdd/2026-10-05-strategic-reporting/reference_check.py` and retained rational output.

**Consumes:** Task 1 Config, Policy, FrozenListener, utility and observation interfaces.

**Produces:** `enumerate(&Config) -> Result<Distribution, Error>`, `evaluate(&Distribution, &Policy, &FrozenListener) -> Result<Evaluation, Error>`, `TrainingPanel::new(&Config, Vec<FrozenListener>) -> Result<TrainingPanel, Error>`, `TrainingPanel::fitness(&Policy) -> Result<i64, Error>`, `TrainingPanel::denominator() -> u64`, `exact_best_response(&TrainingPanel) -> Result<BestResponse, Error>`. Evaluation contains exact signed payoff/utility numerators, denominator, action/error masses and policy-aware Bayesian regret. BestResponse contains canonical Policy, fitness numerator and denominator. Panels use equal weights and reject empty panels or more than 16 listeners.

- [x] Write failing normalization and endpoint fixtures. For q=rho=1 and Credulous, strategic copying gives receiver payoff 1/2 and opposed reporter utility -1/2; strategic inversion must return an unsupported-assumed-history error because perfect-channel disagreement has zero likelihood in Credulous's model. Passive gives payoff and opposed utility zero under every policy. Distinguish expected fractions from realized losses.

```rust
#[test]
fn empty_panel_is_not_zero_fitness() {
    assert!(TrainingPanel::new(&valid_config(), vec![]).is_err());
}
```

`valid_config()` is a test helper introduced in tests/mod.rs returning version 1, IDs 0/1/2, legal permissions, q=4/5, rho=3/4 and opposed utility.

- [x] Run tests red, then implement 128 worlds in canonical order: C, T, fixed profile, calibration signals strategic/fixed, live signals strategic/fixed. Weight each truth by 1/2 and each signal/profile by its rational probability. Common denominator is `4 * rho.denominator * q.denominator.pow(4)`, at most 4,194,304. Accumulate masses in u64 and signed utility/payoff in i64. With maximum panel size 16, the fitness denominator remains at most 67,108,864. Validate before multiplication and use checked arithmetic.
- [x] Derive each candidate's reports through Policy, fixed profile and permitted observations; aggregate public history masses. Retain zero-mass worlds without conditioning on unsupported histories. Compute actual-policy posterior from true-T/history mass and exact policy-aware intervention from integer mass comparison, with ties abstaining. Policy-aware regret must be zero; legacy listener regret may be positive.
- [x] Precompute frozen listener actions for all 32 public histories at panel construction; reject panels descriptively if an assumed listener rejects a history. Direct single-policy evaluation visits only positive-mass reachable histories and propagates unsupported-assumed-history errors. No optimum is claimed for an unsupported endpoint panel. Build candidate fitness from exact world contributions; no runtime reads from policy-unavailable fields.
- [x] Implement best response: enumerate four calibration tables, then for each reachable live row compare its additive utility under false/true reporting, picking false on exact equality. Unreachable rows use false. Select by highest exact panel fitness then lowest encoding. Return a canonical global optimum.
- [x] Implement independent Python Fraction world generation, listener actions, all named controls and full 262,144-policy enumeration for the training panel. Derive history action references independently; do not call Rust or import its formulas as a surrogate reference. Optimization may precompute additive row contributions, but complete enumeration must explicitly visit every encoding. Record both full-enumeration and decomposition optima/tie choices and assert equality. Store generated exact fixtures in the Rust test module with provenance; generation is a correctness exercise, not search collection.
- [x] Test unreachable-row alterations leave fitness unchanged, optimum dominates every named control, both utility signs, endpoints q/rho=0/1, q=1/2, deterministic tie handling, renaming, actual-policy probabilities and reference masses for all supported histories. Add a positive-regret fixture only after independently proving a strict difference; retain the exact numerator in the test.
- [x] Run scoped tests and Python reference. Fresh reviewer independently checks arithmetic bounds, real versus assumed probabilities, best-response separability and reference independence. No first GA/random collection yet.

## Task 3: Frozen GA/random search and champion evaluation

**Files:** Create evolution and tests/evolution.

**Consumes:** Immutable TrainingPanel, Policy, exact best response. **Produces:** `search(&TrainingPanel, seed: u64, method: SearchMethod) -> Result<SearchRun, Error>` with independently versioned SearchMethod/Run/Settings, all constants, evaluations, champion and training curve; `search_seed(seed, method) -> u64`.

- [x] Test ranking on exact fitness ties, uniform valid initialization, single-bit crossover/mutation fixtures, tournament ties, boundary seeds, reproducibility and evaluation accounting before implementing operators.
- [x] Use existing seeded RNG and wrapping derivation `seed*6364136223846793005 + identity*1442695040888963407`, identities 3 for GA and 4 for random; name it `strategic-reporting-search-seed-v1`. These streams differ from prior testimony search streams. Session seeds never select search streams.
- [x] Initialize `rng.gen_range(0..(1u32 << 18))`. Crossover independently chooses each of 18 bits using `rng.gen::<bool>()`; mutation independently flips each bit with `rng.gen_ratio(1,18)`. Population 64, generations 50, elites 2, tournament 3 with replacement. Order higher integer fitness, then lower encoding, stable on identical entries. Count repeated evaluations; each method evaluates exactly 3,164 candidates.

```rust
for bit in 0..18 {
    if rng.gen_ratio(1, 18) { child.bits ^= 1 << bit; }
}
```

- [x] Return owned immutable champions from training. Search accepts no holdout panel or callback. Test that evaluating champions on a separate panel does not change champion, training fitness, curve or RNG accounting. Use small deterministic operator probes for red/green correctness, not full seeds 0..19 before collection.
- [x] Obtain literal reference RNG/operator fixtures with separately checked draws, test nontrivial crossover and mutation rather than only endpoint outputs. Search success versus exact optimum is reported later, never asserted as a mandatory GA result.
- [x] Run scoped tests/format/Clippy. Fresh task review gates RNG independence, panel immutability and fair budget. Keep uncommitted.

## Task 4: Session, replay, fixed CLI diagnostic and first collection

**Files:** Create session/diagnostics/tests/session; add CLI mode and tests/strategic_reporting; extend docs/deduction.md.

**Consumes:** Completed policy, listener, evaluator and search interfaces.

**Produces:** `Session::new(Config,u64) -> Result<Session,Error>`, `request() -> Option<Request>`, `submit(Response) -> Result<(),Error>`, `outcome() -> Option<Outcome>`, `archive() -> Archive`, `Session::replay(&Archive) -> Result<Session,Error>`, `diagnose() -> Result<DiagnosticReport,Error>` and report integrity predicate. Define all session types within this namespace.

- [x] Write failing session fixtures before host implementation. Request envelope has protocol_version/request_id/actor/step/observation/legal; tagged observations are Strategic, Fixed or Decider with their Task 1 views. Legal/action values are Report(bool), Intervene, Abstain; strict decoding rejects unknown fields including fieldless variants. Outcome exposes T, decision payoff and reporter utility only after completion. Archive contains config, seed, accepted responses and request/outcome checkpoint; it is privileged and unauthenticated.
- [x] Seeded draw order is C, T, fixed profile, calibration strategic signal, calibration fixed signal, live strategic signal, live fixed signal. Use rational draws even at endpoints to consume scheduled RNG. Deliver strategic then fixed reporting requests in each exchange, with reports buffered; requests 1..4 report and request 5 decides. Calibration verification becomes visible only after both calibration reports. Retain strategic calibration signal privately for its permitted later memory. No live report from the other reporter is visible before decision.
- [x] Test endpoint generation with literal RNG fixtures; redaction for every phase/actor; both buffered positions; wrong actor/stale ID/wrong action/post-completion rejection; partial archives after 0..5 accepted responses; archive tampering/version mismatch; live truth revealed only at completion. For rejected actions compare the entire future request sequence and final outcome against a clean baseline using otherwise legal continuations.
- [x] Add strict `StrategicReporting` clap variant without tuning flags. Diagnostic training is q=4/5, rho=3/4, opposed utility; equally weighted original Bayesian-assumption and Credulous listeners. Assumed prior for old Bayesian is 3/4. Freeze all 40 champions from seeds 0..19 before holdout evaluation. Holdouts: training panel at q=3/5; Skeptical and Evolved(-3,0,2,0) separately at q=4/5 and 3/5, all rho/assumed-prior 3/4. Passive remains a control. Publish full rules, assumptions, seeds/settings, named controls, panel-specific exact optima, per-champion results/regrets, phase agreement and paired differences.
- [x] CLI exit 0 means exact/reference/integrity checks pass; false integrity emits report then exit 2; operational I/O/error follows existing host handling. Reject extra CLI tuning arguments. Include deterministic report and explicit false-integrity path tests without mutating production search rules.
- [x] Verify the preparation snapshots exist before first collection: existing `deduction diagnose`, `testimony`, `testimony-game` and seed-7 Wink run JSON were captured using the retained pre-change executable. Also verify the preserved docs/deduction.md prefix and source/settings manifest. After implementing, compare outputs byte-for-byte. Save snapshots under ignored per-plan scratch; never overwrite the earlier first reports.
- [x] Run the first complete `deduction strategic-reporting` collection exactly once into `first-strategic-reporting.json`; retain raw JSON, source/settings hashes and a second reproducibility output. Report all actual results without filtering seeds. If any post-result numerical bug changes a result, retain the first evidence and document the revised rule/collection accurately; no stealth retuning.
- [x] Extend docs/deduction.md with observation boundaries, utility, exact optimum, panel assumptions, search methods, seed results and limits. Explain incorrect-channel Bayesian baseline versus policy-aware benchmark; payoff/regret versus guaranteed loss; finite policy restriction; no equilibrium claim; no new interactive host mode. Do not alter old experimental claims.
- [x] Fresh task and whole-increment independent reviews inspect scientific interpretation, protocol boundaries and frozen collection evidence. Fix identified correctness issues with tests; disclose retrospective experimental revisions.
- [x] Run full gates: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `cargo test --workspace`; `wasm-pack test --node crates/sugarscape-wasm`; `cargo build --release -p sugarscape-cli`; `(cd web && npm run wasm && npx tsc --noEmit && npx vitest run)`; `python3 -m unittest discover -s studio/tests -t studio`. Record actual exit codes. No repeated broad testing without a new change/failure.

## Execution and completion

Create temporary root `IMPLEMENTATION_PLAN.md` with the four task stages, criteria, test commands and statuses. Remove it only when tasks, reviews and required checks complete. Retain a per-task report and real red/green evidence in ignored scratch. Do not stage/commit automatically. After explicit merge instruction, push and check CI for the exact merged commit.

## Self-review

Spec coverage: permissions/utility and representation are Task 1; exact worlds, regret and independent optimality are Task 2; all frozen search/transfer constraints are Tasks 3–4; sessions/archives/CLI/preservation and delivery are Task 4. All five Review Focus cases have explicit owning tests. Interfaces use separate new rules and explicit legacy assumptions throughout. No new dependencies or broader game capabilities are implied.

## Completion evidence

All four implementation tasks and independent reviews are complete. Full formatting, Clippy, workspace Rust, WASM Node, release CLI, web WASM/TypeScript/Vitest and studio checks passed. First results/repeat and independent reference evidence are retained in the ignored per-plan workspace. Production and experimental settings did not change after collection. The increment remains uncommitted pending explicit instruction.
