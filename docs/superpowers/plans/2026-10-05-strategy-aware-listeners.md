# Strategy-Aware Listeners Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. A fresh implementer and independent reviewer gate each task; independently review the complete increment before delivery. Steps use checkbox tracking. Plan review precedes execution.

**Goal:** Establish exact Bayesian decisions under a supplied reporting-policy prior, expose calibration identifiability limits, and diagnose fixed-policy misspecification.

**Architecture:** Add `deduction::strategy_inference` beside the frozen strategic-reporting assembly. Build an immutable policy/history/truth mass model from public rules and existing public exact history generation; separate catalog validation, conditioning, evaluation, and reporting. The actual-policy evaluator remains outside listener inference.

**Tech Stack:** Existing Rust, serde, serde_json, clap, and Rust test infrastructure; independent Python standard-library `fractions.Fraction` references. No new dependencies or RNG-based experiment.

**Spec:** `docs/superpowers/specs/2026-10-05-strategy-aware-listeners-design.md`, approved October 5, 2026.

## Global Constraints

- Work in the existing `.claude/worktrees/crowd`, branch `crowd`, starting at `eaf59cab3db3a44905c65e5c6ca8babf6a3994de`. Inspect status first; preserve unrelated work and retained evidence. Do not create another worktree.
- No staging or commits without explicit authorization for this increment. The previous commit authorization is exhausted. Never stage `.claude/`, `papers/`, `survey/out/`, or ignored evidence. No merge or push.
- American spelling; Agent in code/docs. No browser UI, videos, presets, provider integration, interactive host, new sessions, new reporter optimization, or robust adversarial decision rule.
- Keep previous source semantics and diagnostic outputs unchanged. Append to the complete existing guide prefix. Never run `cargo fmt` in `survey/`; this increment edits no survey files.
- At most eight distinct canonical policies; integer weights 0 through 32 and positive total at most 256. Uniform weights `[1,1,1,1,1]`; optimization-informed weights `[1,1,1,1,16]` in named-control order.
- Select policy once, independently of truths, fixed profile, and signal noise. Retain it through both phases. Prior weights are supplied assumptions, not inferred objective rationality.
- Frozen accuracies 4/5 and 3/5, fixed-copy prior 3/4, opposed reporter utility. Record both self-mixtures and all fixed-policy rows; never pool distinct mixtures into one opponent distribution.
- Listener inputs contain public rules/history and a declared catalog only. No actual policy, seed, privileged realized world, current evaluation score, or unverified truth enters inference.
- Exact ties abstain; unsupported evidence returns an explicit error. Support failure is neither abstention nor zero payoff. Negative transfer results are valid outcomes.
- Retain first reports, source/settings snapshots, reference evidence, task reports/reviews, and final verification. Disclose every postmeasurement revision. Reassess after three failed attempts on an issue.
- Fresh subagent implementation and independent task/final reviews. If the host prevents fresh agents, record the limitation and retain separate author/reviewer roles.

## Review Focus

1. Deserialized catalogs cannot bypass bounds, duplicate canonical behavior, or positive-total validation (Task 1).
2. Reordering catalog entries or renaming Agents cannot alter beliefs, and calibration must not consume live evidence (Task 2).
3. Singleton/zero-weight catalogs and endpoint channels can have unsupported actual histories; missing metrics must remain explicit (Tasks 2–3).
4. Zero mixture regret must use the mixture reference, while transfer uses the actual-policy reference; heuristic scores are not probabilities (Task 3).
5. Mutating a report with stale success flags, missing opponent rows, or changed prior metadata must invalidate integrity (Task 4).

## Files and shared interfaces

Create under `crates/sugarscape-core/src/deduction/strategy_inference/`:

- `mod.rs`: exports, version constants, descriptive Error.
- `catalog.rs`: strict catalog wire format, validation, structural canonicalization.
- `model.rs`: immutable exact masses and validated public calibration view.
- `conditioning.rs`: calibration posterior/prediction and complete-history decision.
- `evaluation.rs`: actual-distribution scoring, belief errors, and support accounting.
- `diagnostics.rs`: frozen tables, provenance, independent references, report integrity.
- `tests/{catalog,inference,evaluation,diagnostics}.rs`: behavior tests wired through `mod.rs`.

Modify `crates/sugarscape-core/src/deduction/mod.rs` only to export the namespace. Modify `crates/sugarscape-cli/src/deduction.rs` only for the additive command/emitter/tests. Create `crates/sugarscape-cli/tests/strategy_inference.rs`. Append results to `docs/deduction.md` after first collection. Do not modify `strategic_reporting/` or `testimony_game/`.

Reuse `strategic_reporting::{Config,Policy,Distribution,HistoryMass,DecisionObservation,DecisionAction,FrozenListener,Probability,UtilityTable,enumerate,histories}` and `testimony_game::{Listener,Genome}`. `histories(&Distribution,&Policy)` already returns all 32 complete public-history masses and true-T masses. Marginalizing those rows over live reports gives calibration masses without conditioning on live evidence. The independent oracle must prove equivalence to enumerating the full latent joint distribution, including fixed profile and all four signals.

Shared type/signature contract (each owning task implements its listed items):

```rust
pub const CATALOG_VERSION: u16 = 1;
pub const REPORT_VERSION: &str = "strategy-inference-diagnostic-v1";
pub enum Error {
    InvalidCatalog(&'static str), InvalidObservation(&'static str),
    ZeroEvidence, ArithmeticOverflow,
    Existing(strategic_reporting::Error),
}
pub struct WeightedPolicy { pub policy: Policy, pub weight: u8 }
pub struct Catalog { entries: Vec<WeightedPolicy>, total_weight: u16 }
pub fn canonical_bits(policy: &Policy) -> Result<u32, Error>;
impl Catalog {
    pub fn new(entries: Vec<WeightedPolicy>) -> Result<Self, Error>;
    pub fn entries(&self) -> &[WeightedPolicy];
    pub fn total_weight(&self) -> u16;
    pub fn uniform() -> Self;
    pub fn optimization_informed() -> Self;
}
pub struct Ratio { pub numerator: u64, pub denominator: u64 }
pub struct SignedRatio { pub numerator: i64, pub denominator: u64 }
pub struct PolicyProbability { pub canonical_bits: u32, pub probability: Ratio }
pub struct CalibrationView {
    pub rules: Config, pub calibration_truth: bool,
    pub calibration_reports: [bool; 2],
}
pub struct LivePrediction {
    pub live_reports: [bool; 2], pub probability: Ratio,
    pub truth_and_reports_probability: Ratio,
}
pub struct CalibrationBelief {
    pub policies: Vec<PolicyProbability>, pub live: Vec<LivePrediction>,
}
pub struct InferenceDecision {
    pub policies: Vec<PolicyProbability>, pub posterior_true: Ratio,
    pub action: DecisionAction,
}
struct PolicyMasses { canonical_bits: u32, histories: Vec<HistoryMass> }
pub struct Model {
    config: Config, catalog: Catalog, denominator: u64,
    policies: Vec<PolicyMasses>,
}
impl Model {
    pub fn new(config: &Config, catalog: &Catalog) -> Result<Self, Error>;
    pub fn config(&self) -> &Config;
    pub fn catalog(&self) -> &Catalog;
    pub fn denominator(&self) -> u64;
    pub fn calibration(&self, view: &CalibrationView) -> Result<CalibrationBelief, Error>;
    pub fn decide(&self, view: &DecisionObservation) -> Result<InferenceDecision, Error>;
}
pub enum EvaluatedListener { Strategy(Model), Legacy(FrozenListener) }
pub struct EvaluatedHistory {
    pub observation: DecisionObservation, pub actual_mass: u64,
    pub actual_true_mass: u64, pub actual_posterior: Option<Ratio>,
    pub listener_posterior: Option<Ratio>, pub belief_error: Option<SignedRatio>,
    pub action: Option<DecisionAction>, pub unsupported: bool,
}
pub struct Evaluation {
    pub denominator: u64, pub supported_mass: u64, pub unsupported_mass: u64,
    pub supported_payoff_numerator: i64, pub supported_regret_numerator: i64,
    pub supported_reporter_utility_numerator: i64,
    pub payoff: Option<SignedRatio>, pub decision_regret: Option<Ratio>,
    pub reporter_utility: Option<SignedRatio>, pub maximum_belief_error: Option<Ratio>,
    pub histories: Vec<EvaluatedHistory>,
}
pub fn evaluate_fixed(config: &Config, actual: &Policy,
    listener: &EvaluatedListener) -> Result<Evaluation, Error>;
pub fn evaluate_mixture(config: &Config, actual: &Catalog,
    listener: &EvaluatedListener) -> Result<Evaluation, Error>;
pub fn diagnose() -> Result<DiagnosticReport, Error>;
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error>;
```

Define derived serialization for public report structs with unknown fields rejected at every nesting level. Catalog uses a private wire struct `{version,entries}` and custom decoding through `Catalog::new`; serialize canonical sorted entries and explicit version. Do not deserialize mutable Model caches or `EvaluatedListener`; construct them through validated APIs. Ratio serialization preserves bounded numerator/denominator pairs without requiring reduction. A denominator must be positive; zero probability is `0/d`, never `0/0`.

## Execution preparation

After plan approval and before Task 1, verify the existing linked worktree/branch/status, create this plan's ignored workspace with the SDD skill's workspace script, and record the approved spec/plan hashes and starting HEAD/status in its ledger. Create `IMPLEMENTATION_PLAN.md` with these four stages/statuses, update it during execution, and remove it at completion. Keep the prior increment's retained evidence directory untouched. Preflight shared signatures against existing public APIs before dispatching workers. No implementation has occurred during plan preparation.

## Task 1: Validated policy catalog

**Goal:** Public priors cannot grant duplicate encodings extra probability or bypass validation.
**Success Criteria:** Strict round trips, five unique controls, canonical aliases rejected, order-independent normalized entries.
**Tests:** Catalog limits, invalid policy bits, canonical alias, zero weights, wire versions/unknown fields, all calibration tables.
**Status:** Complete
**Files:** Create `mod.rs`, `catalog.rs`, `tests/catalog.rs`; export new module from `deduction/mod.rs`.
**Consumes:** Existing `Policy::new`, calibration/live indexing, five controls.
**Produces:** Catalog, WeightedPolicy, canonical_bits, Error, CATALOG_VERSION.

- [x] Write catalog behavior tests first. Include this alias test and named-prior weights/total assertions:

```rust
#[test]
fn equivalent_unreachable_rows_cannot_gain_prior_mass() {
    let a = Policy::new(81942).unwrap();
    let b = Policy::new(88214).unwrap();
    assert_eq!(canonical_bits(&a).unwrap(), canonical_bits(&b).unwrap());
    assert!(Catalog::new(vec![
        WeightedPolicy { policy: a, weight: 1 },
        WeightedPolicy { policy: b, weight: 1 },
    ]).is_err());
}
```

- [x] Run `cargo test -p sugarscape-core --lib strategy_inference::tests::catalog`; retain genuine failing output, distinguishing API compile-red from assertion-red.
- [x] Implement canonicalization by retaining bits 0–1 and a live row only when its own-report bit equals `policy.calibration(c_signal)`:

```rust
let mut bits = policy.bits & 3;
for row in 0..16u32 {
    let signal = row & 8 != 0;
    let report = row & 4 != 0;
    if report == policy.calibration(signal) {
        bits |= policy.bits & (1 << (2 + row));
    }
}
```

- [x] Validate encoding before bit operations. Reject 0 or more than 8 entries, weights above 32, all-zero total, and duplicate canonical entries including zero-weight duplicates. Sort by canonical bits; preserve disclosed zero weights. Implement Display/Error with context and strict wire decoding.
- [x] Test all four calibration tables, every live-row bit, invalid bits `1<<18`, 8/9-entry boundary with distinct policies, 32/33-weight boundary, singleton and zero-weight entries. Reverse entries and compare serialized catalogs. Unknown fields in catalog, entries, and Policy, unsupported version, and invalid serde payloads must fail.
- [x] Run scoped tests and `cargo fmt --all -- --check`; format only owned Rust files with rustfmt if needed. Freeze a source manifest and dispatch fresh independent task review. Resolve findings and record approval; do not commit.

## Task 2: Immutable exact conditioning and independent oracle

**Goal:** Infer policy uncertainty and T from only public observations and the declared prior.
**Success Criteria:** Independent per-history equality, preserved calibration odds, zero mixture decision regret, singleton agreement, unsupported evidence explicit.
**Tests:** All supported histories, endpoints, exact ties, direct/sequential equivalence, redaction, permutation, Agent renaming, rule mismatch.
**Status:** Complete
**Files:** Create `model.rs`, `conditioning.rs`, `tests/inference.rs`; update exports in `mod.rs`.
**Consumes:** Task 1 Catalog; existing `enumerate` and `histories`.
**Produces:** Model, Ratio, SignedRatio, CalibrationView/Belief, LivePrediction, PolicyProbability, InferenceDecision.

- [x] Dispatch a separate reference author alongside the implementer, with no shared production authorship. Store the Python Fraction oracle and its outputs under this plan's ignored evidence directory. Independently enumerate Boolean C,T,profile,and four signals; apply reporting-table bits directly; assign weight `prior(policy) * 1/4 * profile_probability * product(signal_likelihoods)`. Do not import Rust outputs, existing reference scripts, or listener caches as the calculation.
- [x] Write tests first for calibration odds and a deliberately unsupported perfect-channel singleton history:

```rust
#[test]
fn singleton_copy_rejects_impossible_calibration() {
    let config = Config::standard(
        Probability { numerator: 1, denominator: 1 },
        Probability { numerator: 1, denominator: 1 }, UtilityTable::opposed());
    let catalog = Catalog::new(vec![WeightedPolicy { policy: Policy::copy(), weight: 1 }]).unwrap();
    let model = Model::new(&config, &catalog).unwrap();
    let view = CalibrationView { rules: config, calibration_truth: true,
        calibration_reports: [false, true] };
    assert!(matches!(model.calibration(&view), Err(Error::ZeroEvidence)));
}
```

- [x] Run `cargo test -p sugarscape-core --lib strategy_inference::tests::inference`; preserve red output.
- [x] Model construction validates Config before allocation, calls `enumerate` once, then `histories` once per canonical entry. Multiply history and true masses by integer catalog weight; common denominator is distribution denominator times total weight. Check all additions/products. Retain per-policy rows privately, including disclosed zero-weight entries. The existing enumeration integrates fixed profile and signals exactly; the independent full-joint oracle verifies this marginalization.
- [x] Implement calibration by selecting matching verified C/report rows and summing over all four live-report combinations. Policy probabilities divide these marginal masses by calibration total. Each prediction reports the live-pair mass and true-T/live-pair joint mass divided by that same calibration total. No selected live report or privileged truth participates.
- [x] Implement final conditioning directly from original joint masses for the complete public history. Return policy probabilities and true/total posterior; intervene iff `true_mass > total_mass - true_mass`. Return ZeroEvidence on zero total. Reject invalid or mismatched public rules without mutating Model. Use checked u64 masses bounded by 1,073,741,824; use i128 intermediates for signed probability subtraction and comparisons, narrowing with checks.
- [x] Independently compare every supported history for both priors/accuracies to Fraction fixtures embedded as Rust integer literals before collection. At q=1/2, every supported final P(T) is 1/2 and action abstains. At both frozen accuracies, assert Copy:CopyCalibrationInvertLive posterior mass cross-products give 1:1 or 1:16 for every supported calibration history.
- [x] Add fixtures for all q/rho endpoints, prior singleton/zero weights, true/false exact ties, zero evidence, reordered entries, renamed Agent IDs, malformed calibration views and Config mismatch. Compare singleton final beliefs/actions to existing `HistoryMass` actual-policy reference for all supported histories. Compare independently implemented sequential conditioning with direct conditioning, including a fixture that would fail if calibration were counted twice. Verify the public calibration view rejects a live-report/truth field rather than ignoring it.
- [x] Run scoped tests, `cargo test -p sugarscape-core --lib strategic_reporting`, and formatting checks. Retain oracle source hash and fixture output hashes. Independent task reviewer checks information boundaries, normalization, arithmetic bounds and oracle agreement before Task 3. Do not collect or commit.

## Task 3: Evaluation, exact belief errors, support accounting

**Goal:** Score the listener on declared mixtures and fixed policies without concealing missing support or reference information.
**Success Criteria:** Mixture correctness exact; transfer metrics use actual distribution; unsupported mass makes unconditional metrics unavailable.
**Tests:** Passive, singleton support failure, all-unsupported case, rare-history maximum error, missing heuristic beliefs, mass/payoff/regret identities.
**Status:** Complete
**Files:** Create `evaluation.rs`, `tests/evaluation.rs`; update `mod.rs` exports.
**Consumes:** Model::decide; existing public histories and FrozenListener::decide.
**Produces:** EvaluatedListener, EvaluatedHistory, Evaluation, evaluate_fixed/evaluate_mixture.

- [x] Write a support-failure test using the q=rho=1 Config and Copy singleton from Task 2, evaluated against Policy::invert(). All actual calibration histories are unsupported: assert unsupported mass equals denominator, supported mass is zero, and unconditional payoff, decision regret, reporter utility, and maximum belief error are None. Write Passive fixture: payoff zero and no listener probability.
- [x] Run `cargo test -p sugarscape-core --lib strategy_inference::tests::evaluation`; retain red output.
- [x] Build actual masses from `histories` for fixed policy, or the same weighted mixture construction used in Task 2 for actual catalog. Actual policy/catalog is an evaluator argument only, never an argument to Model::decide. All 32 history rows remain present; skip action calls for zero actual mass.
- [x] Call Strategy Model or the unchanged legacy adapter. Translate only explicit ZeroEvidence (including the existing wrapped legacy ZeroEvidence variant) into unsupported mass; propagate invalid input/arithmetic/other legacy errors. For supported rows compute payoff and actual-distribution informed regret:

```rust
let gain = 2 * actual_true_mass as i64 - actual_mass as i64;
let payoff = if action == DecisionAction::Intervene { gain } else { 0 };
let regret = gain.max(0) - payoff;
```

- [x] Accumulate reporter utility with the existing validated utility table. Populate unconditional ratios only when unsupported mass is zero. Supported sums always use the disclosed original denominator; never present them as unconditional results. No new reporter regret or best-response optimization is computed.
- [x] For Strategy listeners, compute signed error `p_model - p_actual` by cross multiplication with checked intermediates. Store unreduced exact signed numerator/positive denominator. Compare absolute errors using i128 cross-products; error denominators are at most `(1,073,741,824)^2`, so comparison products fit i128. Retain actual history mass next to every error. Maximum is None if no positive-mass supported belief exists. Legacy adapters supply actions only; their listener_posterior/error/max fields remain None, avoiding fabricated probabilities for Evolved/Passive or changes to frozen APIs.
- [x] Check self-mixture zero decision regret and every belief error zero against the mixture-aware oracle, separately for each prior/q. A fixed policy uses its own informed reference. Include a case where the informed-policy reference outperforms the mixture reference, proving they are not interchangeable. Test supported/unsupported mass sums, zero-weight actual entries, partial support, all unsupported, exact maximum versus an independent Fraction calculation, catalog permutations, renamed IDs, and immutable models after failures.
- [x] Compare legacy payoff/action rows to the previous independent legacy oracle at the same Config and fixed policies; no inference superiority assertion. Run scoped/core regression tests and formatting, freeze manifest, and dispatch fresh independent task review. No collection or commit.

## Task 4: Fixed diagnostic, integrity, collection and product documentation

**Goal:** Deliver auditable frozen comparisons and preserve all earlier results.
**Success Criteria:** Additive CLI, complete provenance and numerical rows, integrity rejects mutated payloads, retained first/repeat bytes match, all standing gates pass.
**Tests:** Command flag rejection, strict report decoding, corruption tests, failed-write/flush propagation, preserved five old diagnostics/guide prefix.
**Status:** Complete
**Files:** Create `diagnostics.rs`, `tests/diagnostics.rs`, CLI integration test; modify new `mod.rs`, CLI deduction dispatch/emitter, and append to `docs/deduction.md` after measurement.
**Consumes:** Tasks 1–3; independent Fraction fixtures; forty previously committed champion encodings/provenance.
**Produces:** DiagnosticReport, diagnose, report_integrity; `deduction strategy-inference`.

- [x] Before collection, verify this plan's ledger shows independent approval of Tasks 1–3 and records the independent reference hashes. Verify the old evidence directory remains intact and the temporary tracker accurately reflects completed stages.
- [x] Retain baseline JSON from current release CLI: `deduction diagnose`, `deduction testimony`, `deduction testimony-game`, `deduction strategic-reporting`, and `deduction run --scenario wink --seed 7 --policy evidence`. Copy the complete existing guide. Record command statuses and source/binary hashes. Use exclusive-create output files.
- [x] Write tests first for a fixed command with no seed, tuning, output-path or positional arguments; report corruption tests must start from a valid search-free diagnostic and mutate current prior weights, environment q/rho, a policy probability, a decision, a payoff/error/support field, clone provenance, reference fixture identity, missing fixed-policy row, and stored passed flag. Retain real behavior-red evidence for stale flags and corrupted payloads.
- [x] Define strict DiagnosticReport fields: `version`, `passed`, `metadata`, `catalogs`, `environments`, `policy_provenance`, `inference_models`, `mixture_evaluations`, `fixed_evaluations`, `checks`. Metadata discloses utility, prior motivation, former-panel target, inference/support/tie rules, and absence of optimization. Environments record q/rho. Each mixture row records the actual catalog and listener identity; each fixed row records canonical policy and listener identity. Evaluations contain all histories and exact metrics from Task 3. Checks have stable names, expected/actual exact values and pass flags. Define row structs in diagnostics.rs with strict nested serde; don't serialize Model caches.
- [x] Add four `inference_models` rows (two priors by two accuracies), each containing eight calibration views/beliefs and 32 complete-history observations/decisions. Preserve every conditional policy probability, live prediction, T posterior and action from Task 2. These are actual-policy-independent model outputs, separate from actual-distribution scoring rows. Validate their exact values against the oracle and recompute them in report integrity; mutate a calibration policy probability and a predictive live mass in negative tests.
- [x] Freeze listeners: two Strategy priors, Bayesian, Credulous, Skeptical, Evolved genome `(-3,0,2,0)`, and Passive; legacy assumed-copy prior 3/4. For each q, each of two self-mixtures is evaluated with its matching Strategy listener and all five legacy listeners (24 mixture rows total). Compare Strategy priors on the same fixed-policy opponents separately.
- [x] Retain five named controls, all forty prior champions, and withheld encoding 98342. GA seeds 0–19 all use 81942. Random seeds 0–19 use `[88214,86550,86358,85142,91158,82838,86486,84246,82198,82902,89494,83542,83542,85782,88342,94934,83862,82070,82198,83542]`. Validate these against committed `docs/deduction.md` and retained first report before coding constants. Six canonical fixed behaviors remain after deduplication: five controls plus 98342; retain every seed/method/encoding as provenance, not independent challenges. Evaluate six behaviors against seven listeners at two q values (84 fixed rows). Record canonical mapping and verify 98342 is absent from both default catalogs.
- [x] Embed independent per-history posterior/action/score references and mixture/fixed payoff/error/support fixtures generated before collection. References are authored independently of production. Diagnostic correctness requires exact equality for Strategy inference/masses and the established legacy action comparisons; negative transfer/payoff differences never fail a check.
- [x] Implement report_integrity by rebuilding every expected field/evaluation/reference check from the frozen protocol without trusting payload flags, omitted rows or changed payload settings. Compare exact values including provenance and row counts. No search, RNG, or privileged controller input is introduced. Keep report decoding strict. Return false on inconsistent payloads; propagate operational errors.
- [x] Add fieldless clap `StrategyInference`. Recompute passed in emitter, emit false JSON before exit 2 for invalid numerical integrity, propagate write/flush errors through existing Failure::Io, and follow the current strategic-reporting emitter pattern. Integration tests run the full new command twice and check byte equality and complete row identities; tests do not assert payoff superiority.
- [x] Run new scoped core tests, CLI emitter/integration tests and formatting, then independent precollection Task 4 review. Fix integrity failures before measurement. Freeze source/settings/oracle/binary manifests only after review approval and targeted checks. Retain the first command output exclusively; repeat to a separate file. Preserve initial evidence if any rule changes later.
- [x] Append actual measured results and methods to the unchanged guide prefix: separate self-mixtures from common-opponent transfer, priors from learned evidence, calibration odds from live updates, inferred beliefs from privileged references, support errors from actions, and behavioral clones from independent challenges. Include all fixed rows and provenance; do not add product-facing bug history.
- [x] Run the standing checks below and capture exit statuses/logs. Dispatch fresh independent whole-increment review over final source, both retained reports, old/new preservation comparisons, manifests and verification. Address findings, recheck changed source as warranted, then remove temporary IMPLEMENTATION_PLAN.md. Preserve this plan's evidence. Report completion without staging, committing, merging or pushing.

## Standing verification and collection ordering

Run from this worktree except where noted:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo build --release -p sugarscape-cli
```

Only after the native release CLI is rebuilt, run from `web/`:

```bash
npm run wasm
npx tsc --noEmit
npx vitest run
```

Also run from the worktree:

```bash
python3 -m unittest discover -s studio/tests -t studio
```

The collection command is `target/release/sugarscape deduction strategy-inference`. Save first and repeat outputs separately without overwriting evidence. Re-run all five earlier diagnostic commands with the rebuilt CLI and compare baseline bytes; verify the old guide remains an exact prefix. Record source hashes before collection and after final checks; explicitly disclose any test/documentation-only postmeasurement changes and require any production/settings change to retain and label the original results.

## Self-review and execution handoff

Coverage: catalog and strict prior wire (Task 1); exact joint marginalization/public-only conditioning and independent oracle (Task 2); decision/reference distinctions, belief metric and support accounting (Task 3); full frozen comparisons, provenance, integrity, CLI, retained evidence and product results (Task 4). All five Review Focus cases have owning tests above. No new session, optimization, dependencies or adversarial guarantee is included.

This plan was approved by the user on October 5, 2026. The execution method is already selected: subagent-driven implementation with independent reviews. Written-spec approval does not authorize committing this increment; no implementation begins before plan review.
