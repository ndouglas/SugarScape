# Adversarial Reporting Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Fresh implementers and independent task reviewers gate each stage; an independent whole-increment review gates delivery. Steps use checkbox tracking. The user approved this plan on October 6, 2026.

**Goal:** Calculate exact targeted best responses against two supplied-prior listeners, verify the fixed-only minimax bound, and measure nominal/worst-case tradeoffs.

**Architecture:** Add `deduction::adversarial_audit` beside the frozen reporting/inference assemblies. Build immutable public-view action tables, a 36-query additive attack scorer, actual-distribution evaluation, and a versioned fixed diagnostic. Keep controller knowledge separate from privileged scoring and independent references.

**Tech Stack:** Existing Rust, serde, serde_json and clap; existing Rust tests and Python standard-library Fraction references. No new dependencies, optimizer or RNG experiment.

**Spec:** `docs/superpowers/specs/2026-10-06-adversarial-reporting-audit-design.md`, approved October 6, 2026.

## Global Constraints

- Use existing `.claude/worktrees/crowd`, branch `crowd`, currently at `aa8815f2be3712c2514bcbc7e3cd1f362f092c7b`. Inspect status and applicable instructions first; do not recreate, discard, stash or silently update the worktree. Main has advanced independently.
- Preserve the uncommitted institutional-corruption entry in `docs/roadmap.md` and unrelated work. This file is outside this increment's implementation/review/commit scope.
- No staging, commits, merges or pushes without explicit authorization for this increment. Never stage ignored evidence, `.claude/`, `papers/` or `survey/out/`.
- Keep old strategic-reporting, strategy-inference and testimony algorithms/schemas/outputs unchanged. New code consumes their public APIs; no private World access.
- American spelling; Agent in code/docs. No UI, videos, ModelKind integration, presets, external provider, interactive sessions, repeated learning, active verification or institutional-corruption implementation.
- Freeze q=4/5 and 3/5, rho=3/4, opposed utility; four controllers: Strategy Uniform, Strategy Optimization-informed, Fixed-only, Passive. Reuse named priors [1,1,1,1,1] and [1,1,1,1,16] without tuning.
- Retain 262,144 raw encodings and 1,024 structural canonical behaviors with 256 aliases each. Reporter knows the public controller rule but selects its policy before private state is realized.
- During play, retain the existing private strategic view and representation restriction on ignoring the other reporter's calibration report. Controllers never receive actual policy, private world, seed/archive or current-game evaluator scores.
- Exact ties abstain; unsupported inference is an error. No floor, fallback action or zero-payoff substitution. Scalar probabilities and payoffs stay exact.
- The fixed channel and genuine C verification remain trusted assumptions. Minimax protection is expected payoff within this game, not a realized-game or empirical guarantee.
- Independent references precede collection. Preserve first reports/source/settings/binary, reviews and verification; disclose later revisions. Keep results in product docs, development failures in retained evidence.
- Root runs standing integration gates; workers run targeted tests and owned-file formatting. Never run cargo fmt in survey; no survey edits belong here. Reassess after three failed attempts on an issue.

## Review Focus

1. A snapshot bearing a valid controller label must not certify altered actions, probabilities, order, Config or version (Task 1).
2. Wrong live-row coordinates, unreachable aliases and zero-delta ties must not change the exact optimum or witness identity (Task 2).
3. A nominal mixture's informed reference must condition pooled history masses, not average references that know each realized policy (Task 3).
4. Guarantee shortfall F−V, actual-policy regret and nominal performance differences must remain distinct; empty or impossible evidence is not abstention (Tasks 1–3).
5. Stale passed flags, deleted fitness/transfer rows, changed provenance and incorrect checks must fail current-payload integrity (Task 4).

## Files and shared contracts

Create under `crates/sugarscape-core/src/deduction/adversarial_audit/`:

- `mod.rs`: exports, Error, action/report versions.
- `fixed_only.rs`: fixed-evidence marginal and exact decision.
- `actions.rs`: named controllers, public history indexing, immutable action snapshots.
- `scoring.rs`: shared exact history contributions and fitness scores.
- `best_response.rs`: canonical universe, basis deltas, ranking and exhaustive verification.
- `evaluation.rs`: full scoring, pooled nominal mixtures and guarantee differences.
- `diagnostics.rs`: fixed protocol, comparisons, integrity.
- `diagnostics/references.rs`: checked-in independent fixtures and exact checks.
- `tests/{actions,best_response,evaluation,diagnostics}.rs` and bounded checked-in reference JSON where useful.

Modify `crates/sugarscape-core/src/deduction/mod.rs` only for the new namespace export. Modify `crates/sugarscape-cli/src/deduction.rs` only for additive command/emitter/tests. Create `crates/sugarscape-cli/tests/adversarial_audit.rs`. Append measured methods/results to `docs/deduction.md` after first collection. Split large new helpers by responsibility; do not restructure earlier modules.

Reuse `strategic_reporting::{Config,Policy,DecisionObservation,DecisionAction,Probability,UtilityTable,Distribution,HistoryMass,FrozenListener,Listener,enumerate,histories,evaluate}`; `strategy_inference::{Catalog,Model,Ratio,SignedRatio,canonical_bits}`. Existing `strategic_reporting::Evaluation` provides the required full result shape; reexport it as `AuditEvaluation` without changing its meaning. It includes informed reference payoffs/posteriors, decision regret, action/error masses and phase agreement/opposition.

Shared interfaces (owning tasks implement the following signatures):

```rust
pub const ACTION_VERSION: u16 = 1;
pub const REPORT_VERSION: &str = "adversarial-audit-diagnostic-v1";
pub enum Error {
    InvalidRules(&'static str), InvalidActions(&'static str),
    InvalidReport(&'static str), ZeroEvidence, ArithmeticOverflow,
    Reporting(strategic_reporting::Error),
    Inference(strategy_inference::Error),
}
pub enum ControllerKind {
    StrategyUniform, StrategyOptimizationInformed, FixedOnly, Passive,
}
pub struct FixedView {
    pub rules: Config, pub calibration_truth: bool,
    pub calibration_report: bool, pub live_report: bool,
}
pub struct FixedMass { view: FixedView, total_mass: u64, true_mass: u64 }
pub struct FixedModel { config: Config, denominator: u64, rows: [FixedMass; 8] }
pub struct FrozenDecision { pub posterior_true: Option<Ratio>, pub action: DecisionAction }
impl FixedModel {
    pub fn new(config: &Config) -> Result<Self, Error>;
    pub fn decide(&self, view: &FixedView) -> Result<FrozenDecision, Error>;
}
pub fn history_view(config: &Config, index: u8) -> Result<DecisionObservation, Error>;
pub fn history_index(view: &DecisionObservation) -> usize;
pub struct ActionRow {
    pub index: u8, pub observation: DecisionObservation, pub decision: FrozenDecision,
}
pub struct ActionSnapshot {
    pub version: u16, pub rules: Config, pub controller: ControllerKind,
    pub rows: Vec<ActionRow>,
}
pub struct FrozenActions { config: Config, controller: ControllerKind, rows: [ActionRow; 32] }
impl FrozenActions {
    pub fn freeze(config: &Config, controller: ControllerKind) -> Result<Self, Error>;
    pub fn from_snapshot(snapshot: &ActionSnapshot) -> Result<Self, Error>;
    pub fn config(&self) -> &Config;
    pub fn controller(&self) -> &ControllerKind;
    pub fn rows(&self) -> &[ActionRow; 32];
    pub fn snapshot(&self) -> ActionSnapshot;
}
pub struct Score {
    pub rules: Config, pub denominator: u64,
    pub payoff_numerator: i64, pub utility_numerator: i64,
}
impl Score { pub fn validate(&self) -> Result<(), Error>; }
pub struct BasisRow {
    pub calibration: u8, pub base_utility_numerator: i64,
    pub deltas: [Option<i64>; 16],
}
pub struct AttackBasis { config: Config, denominator: u64, rows: [BasisRow; 4] }
impl AttackBasis {
    pub fn build(actions: &FrozenActions) -> Result<Self, Error>;
    pub fn denominator(&self) -> u64;
    pub fn rows(&self) -> &[BasisRow; 4];
    pub fn fitness(&self, policy: &Policy) -> Result<i64, Error>;
}
pub struct FitnessRow { pub canonical_bits: u32, pub utility_numerator: i64 }
pub struct BestResponse { pub policy: Policy, pub score: Score }
pub fn canonical_policies() -> Vec<Policy>;
pub fn score(actions: &FrozenActions, policy: &Policy) -> Result<Score, Error>;
pub fn exact_best_response(basis: &AttackBasis) -> Result<BestResponse, Error>;
pub fn fitness_table(basis: &AttackBasis) -> Result<Vec<FitnessRow>, Error>;
pub fn evaluate_fixed(actions: &FrozenActions, actual: &Policy) -> Result<AuditEvaluation, Error>;
pub fn evaluate_mixture(actions: &FrozenActions, actual: &Catalog) -> Result<AuditEvaluation, Error>;
pub fn guarantee_shortfall(fixed: &Score, worst: &Score) -> Result<Ratio, Error>;
pub fn nominal_difference(nominal: &Score, fixed: &Score) -> Result<SignedRatio, Error>;
pub fn diagnose() -> Result<DiagnosticReport, Error>;
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error>;
```

ControllerKind serializes as the strings `strategy_uniform`, `strategy_optimization_informed`, `fixed_only`, `passive`; unknown strings fail. Public snapshots/report structs deny unknown fields at all nesting levels, reusing existing strict observation/ratio decoding. Private models/bases/action caches are not deserializable. `from_snapshot` validates shape/order/rules and reconstructs the named controller to compare the complete snapshot, rather than trusting its label or cached data. Config validation also requires opposed utility for audit scoring. Error conversions retain causes and descriptive context.

History ordering is the existing five-bit order: C at bit4, own/fixed calibration at bits3/2, own/fixed live at bits1/0. Fixed-only eight-row indexing uses C at bit2, fixed calibration at bit1, fixed live at bit0. Reject complete indices outside 0..32. A ratio has a positive denominator; no 0/0. Comparison arithmetic uses checked i128 intermediates and checked narrowing.

## Execution preparation

After plan approval, verify the linked worktree and current status, create this plan's ignored SDD workspace and ledger with the plan path, and record approved artifacts/source hashes and starting HEAD. Preserve earlier evidence directories. Root creates `IMPLEMENTATION_PLAN.md` with the four stages below, updates it during execution and removes it at completion. Preflight shared signatures and task/file ownership before dispatch.

Retain the existing release executable, full guide and six baseline outputs exclusively: `deduction diagnose`, `testimony`, `testimony-game`, `strategic-reporting`, `strategy-inference`, and `run --scenario wink --seed 7 --policy evidence`. Record exit statuses and hashes. These are reproductions, not revisions of earlier experimental settings.

## Task 1: Fixed-only inference and validated action freezes

**Goal:** Four controllers expose auditable decisions through public evidence only.
**Success Criteria:** Named snapshots reconstruct exactly; fixed-only ignores both strategic reports; unsupported evidence remains explicit.
**Tests:** 32 rows/order/version, Config mismatch, corruption, renamed IDs, exact ties/endpoints, redaction, fixed-only marginalization.
**Status:** Complete
**Files:** Create `mod.rs`, `fixed_only.rs`, `actions.rs`, `tests/actions.rs`; export namespace from `deduction/mod.rs`.
**Consumes:** Existing public Config/history enumeration, strategy Models and priors.
**Produces:** Error/constants, ControllerKind, FixedModel/View, FrozenDecision, ActionSnapshot/Row, FrozenActions, history_view/index.

- [x] Write behavior tests first, including a mutation that must not certify a valid named snapshot:

```rust
#[test]
fn changed_named_actions_are_not_a_valid_freeze() {
    let c = Config::standard(Probability { numerator: 4, denominator: 5 },
        Probability { numerator: 3, denominator: 4 }, UtilityTable::opposed());
    let mut snapshot = FrozenActions::freeze(&c, ControllerKind::Passive).unwrap().snapshot();
    snapshot.rows[0].decision.action = DecisionAction::Intervene;
    assert!(FrozenActions::from_snapshot(&snapshot).is_err());
}
```

- [x] Run `cargo test -p sugarscape-core --lib adversarial_audit::tests::actions`; retain compile-red separately from genuine behavioral red. Add only the scaffolding needed to observe the validation/privacy failures.
- [x] Build FixedModel from public rules by obtaining constant-positive history masses through `histories(enumerate(config), Policy::positive())` and marginalizing away strategic reports. This constant is an internal marginalization device, not an actual-policy input or assumption used to classify the reporter. Independent latent enumeration will verify the fixed-only conditional distribution. Its eight private rows carry actual fixed-evidence masses; zero total returns ZeroEvidence. Validate rules and opposed utility before allocation.
- [x] Build both existing Models from their unchanged named catalogs, FixedModel from fixed evidence, and Passive directly. Query all 32 public histories. Extract only C/fixed report coordinates for fixed-only. Store complete decision rows in an immutable private array. Reconstruct and compare deserialized snapshots; reject wrong version/count/order/index, mismatched rules, changed posterior/action and unknown fields.
- [x] Test fixed-only strategic-bit invariance across all 32 rows, direct marginal agreement under several representative hypothetical reporting policies, Agent renaming, Config mismatch/nonmutation, q=1/2 ties, q/rho endpoints with supported and unsupported fixed histories, and rejection of private signal/truth/policy fields in FixedView. An endpoint freeze requiring unsupported rows returns error, not a fabricated action.
- [x] Run scoped actions tests, earlier `strategy_inference` tests and owned-file formatting checks. Freeze source/report evidence and dispatch a fresh independent task review. No commits or collection.

## Task 2: Exact scoring, canonical universe and attack decomposition

**Goal:** Every target has a provably exact, canonically ranked best response.
**Success Criteria:** 36-query construction equals direct scores for all 1,024 canonical policies per target/environment; every raw alias agrees.
**Tests:** Canonical counts/multiplicity, every row/flip, nonconstant additivity, zero deltas, deterministic ties, invalid policy, exact independent references.
**Status:** Complete
**Files:** Create `scoring.rs`, `best_response.rs`, `tests/best_response.rs`, checked-in attack reference fixture; update new exports.
**Consumes:** Task 1 frozen action tables and existing public policy/history masses.
**Produces:** Score, BasisRow/AttackBasis, canonical_policies, score, exact_best_response, fitness_table; namespace-private shared scoring helpers for Task 3.

- [x] Dispatch a separate independent Fraction-oracle author alongside this implementer. Independently enumerate C,T,fixed profile,and four signals; apply reporting-table coordinates directly. Derive named strategy action tables from the five-hypothesis joint model and fixed-only from the fixed marginal, without importing Rust output or previous numeric fixtures. Source/report hashes and any exposure limitations remain explicit.
- [x] Write tests first for the exact baseline and tie conventions:

```rust
#[test]
fn passive_best_response_uses_smallest_canonical_encoding() {
    let c = Config::standard(Probability { numerator: 4, denominator: 5 },
        Probability { numerator: 3, denominator: 4 }, UtilityTable::opposed());
    let actions = FrozenActions::freeze(&c, ControllerKind::Passive).unwrap();
    let response = exact_best_response(&AttackBasis::build(&actions).unwrap()).unwrap();
    assert_eq!(response.policy.bits, 0);
    assert_eq!(response.score.utility_numerator, 0);
}
```

- [x] Run `cargo test -p sugarscape-core --lib adversarial_audit::tests::best_response`; preserve genuine failed assertions before implementing fitness/ranking.
- [x] Score actual policy masses through `histories` and frozen actions. For a positive-mass history, intervention gain is `2*true_mass-total_mass`; abstention gain is zero; opposed utility is its negative. Validate Policy first, Config equality on rows, mass normalization and true<=total. Use one shared history-contribution helper for direct scoring and detailed evaluation. Score stores the public Config so later comparisons can reject incompatible environments. Score::validate rejects invalid/opposed-mismatched rules, zero or greater-than-2^30 denominator, numerators with twice their absolute payoff greater than the denominator, and utility unequal to checked negative payoff. The half-denominator payoff bound follows from uniform T and the intervene/abstain payoff table; use i128 for validation to avoid overflowing on forged i64 extremes.
- [x] Generate canonical policies by four calibration tables and all 256 assignments to their eight structurally reachable live rows, clear unreachable bits, sort by unsigned encoding, and verify count/uniqueness. Build each basis with its all-false-live policy and eight single-row flips. For row r, retain Some(flipped_utility-base_utility) if reachable, None otherwise. Recombine fitness as base plus deltas of selected reachable bits. Raw aliases must score identically.
- [x] Select each positive delta, false on zero, then choose the highest utility of four calibration candidates with lowest-encoding ties. Emit 1,024 sorted FitnessRows. Independent full-world canonical evaluation verifies all 8,192 fitness values and eight optima. The oracle additionally walks all 262,144 raw encodings, checks reachable behavior against its canonical equivalent and raw score/ranking through independently derived contributions; avoid hundreds of millions of redundant Fraction world calculations.
- [x] Test every raw canonicalization/multiplicity and score identity using cheap basis fitness, all live-row coordinates including unreachable flips, nonconstant multi-bit additivity versus direct history scoring, altered private-signal/report indexing fixtures, zero-delta ties, maximum-valid probability denominators, invalid policy bits and mutated snapshot rejection. Embed frozen oracle values in a portable checked-in fixture; no runtime dependency on ignored evidence.
- [x] Run all scoped audit tests, formatting, and independent task review over source plus oracle provenance. Do not run a new external audit command or collect results. No attack payoff sign or superiority assertion is a correctness requirement.

## Task 3: Detailed evaluation, nominal mixtures and minimax comparisons

**Goal:** Distinguish targeted loss, privileged reference regret and nominal usefulness.
**Success Criteria:** Complete metrics match independent Fraction values; F is invariant across all policies; mixtures use pooled evidence.
**Tests:** Fixed-only bound, constant witnesses, pooled-versus-policy-aware reference counterexample, regret/mass identities, source isolation, nominal differences.
**Status:** Complete
**Files:** Create `evaluation.rs`, `tests/evaluation.rs`, checked-in detailed reference fixture; update exports and minimal shared scoring helpers as necessary.
**Consumes:** Task 2 scores/bases/fitness and Task 1 snapshots, existing Catalog and AuditEvaluation shape.
**Produces:** evaluate_fixed, evaluate_mixture, guarantee_shortfall, nominal_difference; audited bound/reference comparisons.

- [x] Write tests first for fixed-only invariant payoff using previously published predictions, labeled as such:

```rust
#[test]
fn fixed_only_has_the_published_constant_policy_benchmark() {
    let c = Config::standard(Probability { numerator: 4, denominator: 5 },
        Probability { numerator: 3, denominator: 4 }, UtilityTable::opposed());
    let frozen = FrozenActions::freeze(&c, ControllerKind::FixedOnly).unwrap();
    for p in canonical_policies() {
        let s = score(&frozen, &p).unwrap();
        assert_eq!(s.payoff_numerator * 50, 9 * s.denominator as i64);
    }
}
```

- [x] Run `cargo test -p sugarscape-core --lib adversarial_audit::tests::evaluation`; retain behavioral red for missing metrics/mix pooling/shortfall logic. Confirm q=3/5 F=1/20 separately with independently derived reference values.
- [x] Build fixed-policy rows from public history masses and pooled mixture rows by multiplying each canonical policy's history masses by its declared integer weight and summing before conditioning. For every pooled positive-mass history compute reference posterior/action and gain from pooled true/total masses. Do not average per-policy informed optima or regrets. Zero actual-mass histories have absent posteriors/actions and zero contributions; normalization still includes all 32 rows.
- [x] Populate AuditEvaluation with the actual distribution denominator, listener payoff, opposed utility, informed optimum, exact regret, action/false/missed masses and phase statistics. For action-independent truth/signal agreement statistics, reuse public `strategic_reporting::evaluate` with its Passive adapter and extract only those statistics; do not take its receiver actions as the new target's decisions. Mixture phase statistics aggregate linearly by policy weights, while its informed reference is rebuilt from pooled evidence. No private World access or duplicated latent-world generator enters production.
- [x] Compute F−V using checked cross-products after validating both Scores and requiring equal public Config; require nonnegative value. Compute nominal payoff minus fixed-only payoff separately as SignedRatio with the same rules check. Bounds: D<=2^22, row delta<=2D; arbitrary validated catalog total<=256 gives mixture denominator<=2^30. For legitimate comparisons to fixed-only, products are at most 2^52; public comparison helpers also use checked i128 arithmetic and narrowing to reject malformed caller-created Scores safely. All actual utility/action accumulations use checked i64 additions. Test incompatible Configs and forged zero/out-of-range Score fields explicitly.
- [x] Independently verify the constant-policy upper bound and fixed-only lower bound. Against constant-positive and constant-negative controls, the actual-policy informed optimum equals F; any frozen controller payoff is at most F. Fixed-only equals F for all 1,024 behaviors; Passive equals zero. This proves the stated minimax value without a receiver-policy search. Document that randomized actions cannot exceed a conditional best action against the constant witness.
- [x] Extend the oracle for all eight targeted, 48 control, 16 nominal and 32 cross-target evaluations, each with all history masses and metrics. Include a concrete mixture where averaging informed per-policy references exceeds the pooled mixture optimum. Confirm current Strategy self-mixture results agree with their earlier published values, as regression/provenance checks rather than new findings.
- [x] Test renamed IDs, mismatched rules, private-policy input absence, expected versus realized payoff language, full phase agreement/opposition identities, proper zero-mass rows, partial probability endpoints, exact regret/shortfall distinction, and canonical tie witnesses. Run scoped/old regression tests and formatting; independent task review gates Task 4. No collection or commits.

## Task 4: Frozen diagnostic, corruption checks, collection and product results

**Goal:** Deliver a complete, reproducible adversarial audit with interpretable guarantees and costs.
**Success Criteria:** Strict additive CLI, current-payload integrity, complete row identities, retained first/repeat bytes and all six prior outputs, final independent approval and standing checks.
**Tests:** Protocol corruption, deleted rows, altered prior/basis/provenance, failed-flag/write/flush handling, flag rejection, complete repeatable CLI output.
**Status:** Complete
**Files:** Create `diagnostics.rs`, `diagnostics/references.rs`, `tests/diagnostics.rs`, diagnostic reference JSON and CLI integration test; modify new exports, CLI deduction dispatch/emitter; append `docs/deduction.md` after measurement.
**Consumes:** Tasks 1–3, independent oracle, retained baseline/provenance.
**Produces:** DiagnosticReport, diagnose, report_integrity; fieldless `deduction adversarial-audit`.

- [x] Define strict report fields: `version`, `passed`, `metadata`, `environments`, `controller_snapshots`, `fixed_only_models`, `policy_provenance`, `fitness_tables`, `targeted_audits`, `control_evaluations`, `nominal_evaluations`, `cross_target_evaluations`, `bound_checks`, `checks`. Typed row structs carry environment/controller/policy identities and exact scores or AuditEvaluation; metadata discloses the threat model, trusted components, prior assumptions, canonical tie rule, 36-query decomposition, prior published benchmark predictions, source/reference hashes and absence of RNG/learning. fixed_only_models contains two eight-row posterior tables.
- [x] Freeze all eight snapshots before computing any attack. Store eight complete basis/fitness tables (8,192 fitness rows), eight targeted witnesses, 48 control evaluations (six distinct behaviors by four controllers by two q), sixteen nominal rows (both populations for every controller), and thirty-two cross-target witness rows. Preserve five named controls, withheld 98342, and all forty champion seed/method/raw encodings as 46 provenance entries. Read those exact identities from committed guide/retained prior report; do not rerun search or treat aliases as independent challenges.
- [x] Write corruption tests first, starting with a numerically valid, search-free report. Change a controller action/posterior, Config, prior metadata, a basis delta, canonical fitness, optimum/tie witness, shortfall, pooled nominal reference, cross-target identity, a provenance encoding, a bound value, a check flag/value; remove a row/check. Retain actual assertion-red evidence showing a flag-trusting gate accepts invalid current payloads before full integrity implementation.
- [x] Implement integrity by rebuilding all frozen protocol fields and independent checks, then comparing the current complete typed payload. Do not use submitted settings/caches/flags as expected values. Reconstruction performs deterministic arithmetic only. Require independent reference success; operational errors propagate. Invalid numerical payload emits passed=false before existing exit2 behavior; propagate write/flush failures unchanged.
- [x] Add the fieldless clap mode and focused emitter tests. CLI integration rejects seed/generation/prior/q/output-path/positional tuning and later checks complete identities plus repeatable success. Before the first retained external command, hold the successful-report integration test; flag rejection and unit/helper tests can run. Run targeted core/CLI/format checks, freeze source, and dispatch independent precollection review.
- [x] After approval, rebuild the native release CLI, record source/settings/oracle/binary hashes before invoking it, copy the exact executable, and use exclusive-create files for first/repeat output. Confirm byte equality and source/settings unchanged. Reproduce all six baseline commands and compare bytes; the full existing guide must remain an unchanged prefix. Preserve original results if any later correction is needed.
- [x] Release the held successful CLI test only after first-report retention. Root can run it through the full workspace suite; do not duplicate its run unnecessarily. Append actual targeted payoff/shortfall/reference-regret tables, nominal tradeoffs and cross-target outcomes to the guide. Explain the constant-report upper bound, trusted fixed channel, private-view boundary, clone provenance and expected-payoff scope. Include negative outcomes; avoid development bug history and broad adversarial/equilibrium claims.
- [x] Run the standing checks below, retain statuses/logs/source manifests, and dispatch a fresh independent whole-increment review covering source, all measured rows, oracle provenance, integrity, docs, first/repeat/preservation and verification. Resolve findings with scoped reruns as warranted. Mark plan/tracker complete and remove only this task's temporary tracker; retain all ignored evidence. Deliver uncommitted work unless the user explicitly authorizes integration.

## Standing verification

Root runs from this worktree:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo build --release -p sugarscape-cli
python3 -m unittest discover -s studio/tests -t studio
```

After the native release CLI rebuild, run from `web/`:

```bash
npm run wasm
npx tsc --noEmit
npx vitest run
```

The collection command is `target/release/sugarscape deduction adversarial-audit`. If a checkout lacks ignored Studio retirement dump inputs, inspect the failing test and existing verified fixtures, restore only absent required files byte-identically with provenance, and rerun; never disable tests or overwrite other experimental outputs. Do not repeat unchanged passing gates without a new failure, source revision or unresolved concern.

## Self-review and handoff

Spec coverage: public-only freezes and fixed marginal (Task 1); full family, aliases and exact attack/basis proofs (Task 2); minimax/reference distinctions, phase metrics and common-population tradeoffs (Task 3); complete protocol, corruption checks, CLI, retained collection and measured product docs (Task 4). Each Review Focus case has an owning behavioral test. Bound values are known prior predictions; targeted listener attack outcomes remain unmeasured. No optimization-informed prior tuning, new robust decision rule or active-verification mechanism is included.

The user approved this plan on October 6, 2026. The execution method is already selected by standing instruction: subagent-driven implementation with independent task/final reviews. Implementation is authorized by plan approval; commits, merges and pushes require separate explicit instructions.
