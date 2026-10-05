# Exact Testimony Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Deliver an exact, bounded testimony model and known-answer diagnostics that distinguish signal quality, persistent reporting strategy, and shared evidence.

**Architecture:** Add a pure finite model beside the existing deduction engine. Infer joint hypothesis weights by marginalizing shared Boolean signals in log space; retain an atomic evidence ledger and expose marginals. A separate diagnostic module and CLI command exercise fixed rational reference cases without adding a claim-aware game policy.

**Tech Stack:** Existing Rust workspace, serde/serde_json, clap, existing Rust integration/unit tests; standard-library Python Fraction only for independent development-time reference checks.

**Spec:** `docs/superpowers/specs/2026-10-04-deduction-testimony-design.md` (approved; read in full).

## Global Constraints

- Work in `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/crowd`, branch crowd. Do not create another worktree.
- No new dependencies, engine fields, protocol version, PolicyKind, ModelKind, browser integration, or renderer.
- Preserve the existing uncommitted capability engine/CLI/docs and filmmaking guidance. Snapshot current files before execution: HEAD does not contain this baseline yet.
- Keep deduction-v1 rules, policies, measurements, diagnose output, session format, and play behavior intact. Do not edit the preserved first collection.
- Bounds: 1..16 propositions, 1..32 speakers, 1..16 profiles, 1..256 hypotheses, 0..256 signal groups, 512 accepted distinct records maximum. IDs are unique within their kind and may be sparse; counts do not bound numeric ID values.
- Channel/prior probabilities are finite in [0,1]; priors sum to one within 1e-12 with positive mass. Complete per-hypothesis proposition and speaker assignments required.
- New serialized input structs reject unknown fields. Rejected observations never change model, ledger, or posterior. Zero support is an error; do not floor impossible probabilities.
- Use Agent in code/docs, American spelling. Commit only on request; never stage .claude/, papers/, or survey/out/.
- Use fresh implementation/review agents per task and a whole-change review at the end. Root owns plan status and a three-stage IMPLEMENTATION_PLAN.md, removed after completion.
- Max three failed attempts per issue, then document failures and reassess. Record actual red/green evidence without claiming that every extra regression was independently red.

## Review Focus

- Recomputing from the full ledger must start with the model prior, not the last posterior; otherwise old evidence is counted twice. Task 2 tests 13/20 followed by 49/68.
- Same-speaker records in different groups still share a persistent profile; group independence is conditional. Task 2 contrasts 49/68 with the incorrect 169/218.
- Unknown provenance or independent verification cannot be inferred from current game claims. Task 3 documents declared assumptions and passes no Engine/archive into belief operations.
- Duplicate IDs at full capacity remain idempotent, while conflicting duplicates fail without mutation. Task 2 protects validation order with a filled ledger.
- A history can have tiny positive likelihood under every world; log-space normalization must distinguish it from true zero support. Task 2 tests 512 reports with emission probability 1/1000 and a separate contradiction.

## Files and interfaces

- `crates/sugarscape-core/src/deduction/testimony.rs`: model/input/output types, validation, private channel math, Belief, evidence ledger and marginalization.
- `crates/sugarscape-core/src/deduction/testimony_diagnostics.rs`: fixed testimony-v1 cases, reference comparisons, permitted decisions and report.
- `crates/sugarscape-core/src/deduction/mod.rs`: additive public exports only.
- `crates/sugarscape-core/tests/testimony.rs`: public behavior tests and exact fixture builders.
- `crates/sugarscape-cli/src/deduction.rs`: additive Testimony command using current emit/error helpers.
- `crates/sugarscape-cli/tests/testimony.rs`: real-binary JSON and compatibility checks.
- `docs/deduction.md`: appended model/command/findings/limitations section.

Task 1 defines the following complete serializable input contracts. Use serde deny_unknown_fields on each input struct and the internally tagged structured evidence variants.

```rust
pub type PropositionId = u16;
pub type ProfileId = u16;
pub type HypothesisId = u16;
pub type SignalGroupId = u16;
pub type EvidenceId = u64;
pub struct Proposition { pub id: PropositionId, pub label: Option<String> }
pub struct ReportingProfile {
    pub id: ProfileId,
    pub positive_given_signal: [f64; 2],
}
pub struct PropositionValue { pub proposition: PropositionId, pub value: bool }
pub struct SpeakerProfile { pub speaker: AgentId, pub profile: ProfileId }
pub struct TestimonyHypothesis {
    pub id: HypothesisId,
    pub prior: f64,
    pub propositions: Vec<PropositionValue>,
    pub profiles: Vec<SpeakerProfile>,
}
pub struct SignalGroup {
    pub id: SignalGroupId,
    pub proposition: PropositionId,
    pub accuracy: f64,
}
pub struct TestimonyModel {
    pub propositions: Vec<Proposition>,
    pub speakers: Vec<AgentId>,
    pub profiles: Vec<ReportingProfile>,
    pub hypotheses: Vec<TestimonyHypothesis>,
    pub groups: Vec<SignalGroup>,
}
pub struct EvidenceRecord { pub id: EvidenceId, pub content: TestimonyEvidence }
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TestimonyEvidence {
    Report { group: SignalGroupId, speaker: AgentId, positive: bool },
    Verified { proposition: PropositionId, value: bool },
}
```

`TestimonyError` variants: InvalidModel(Vec<FieldError>), UnknownGroup(SignalGroupId), UnknownSpeaker(AgentId), UnknownProposition(PropositionId), ConflictingEvidence(EvidenceId), CapacityExceeded, ZeroEvidence. Implement Display/Error with descriptive context, using existing FieldError. No inference error contains an actual hidden world.

`BeliefSnapshot` has `hypotheses: Vec<HypothesisProbability>`, `propositions: Vec<PropositionProbability>`, `speakers: Vec<SpeakerMarginal>`, and `evidence_count: usize`. Supporting fields:

```rust
pub struct HypothesisProbability { pub id: HypothesisId, pub probability: f64 }
pub struct PropositionProbability { pub id: PropositionId, pub probability_true: f64 }
pub struct ProfileProbability { pub id: ProfileId, pub probability: f64 }
pub struct SpeakerMarginal { pub speaker: AgentId, pub profiles: Vec<ProfileProbability> }
```

Snapshot order follows declared hypothesis/proposition/speaker/profile order, including unused zero-mass profiles. Numerical invariance under reordering/renaming is checked by corresponding ID within 1e-12, not raw vector equality. Belief derives Clone/Debug/PartialEq to make complete-state atomicity testable; fields stay private. Constructor normalizes accepted priors to a probability distribution after validation. Empty-history weights equal that normalized prior.

---

### Task 1: Validated finite model and channel atoms

**Files:** Create testimony.rs and tests/testimony.rs; modify deduction/mod.rs. Root creates the three-stage execution progress file before dispatch.

**Interfaces:** Produces all types above, Belief::new(model) -> Result<Belief, TestimonyError>, and Belief::snapshot() -> BeliefSnapshot. No observe stub. Task 2 consumes the private helper `group_log_likelihood(model: &TestimonyModel, hypothesis: &TestimonyHypothesis, group: &SignalGroup, records: &[EvidenceRecord]) -> f64`, which filters reports by the supplied group ID and ignores Verified records. The model/assignments/report references are validated before this helper is called; its result is a finite log likelihood or negative infinity. No public Engine input.

- [x] **Step 1: Write constructor, snapshot and private channel tests.** A two-hypothesis `binary_model(profile, accuracy)` builder declares proposition 7, speaker 19, profile 3, hypotheses 4/9 for false/true with priors 1/2, and signal groups 11/12. Hypotheses assign the sole proposition/speaker explicitly. Inline channel atom tests supply report records directly to the validated private helper.

```rust
#[test]
fn testimony_empty_history_returns_normalized_prior() {
    let belief = Belief::new(binary_model([0.0, 1.0], 0.8)).unwrap();
    let s = belief.snapshot();
    assert_eq!(s.evidence_count, 0);
    assert!((s.propositions[0].probability_true - 0.5).abs() < 1e-12);
}
```

Test every count bound, duplicate IDs, missing/duplicate assignments, unknown references, priors with NaN/infinity/negative/out-of-range/zero total/incorrect sum, channel invalid values, sparse IDs, zero prior worlds, zero groups, unknown input JSON fields, and wrong channel array length. Copy/invert/always-positive channel tests check the two conditional likelihoods at q=0, 1/2, 4/5 and 1. Repeating deterministic reports within one group must not square a marginalized signal likelihood.

- [x] **Step 2: Run red.** `cargo test -p sugarscape-core testimony`. Record the actual missing types/export failure, then behavioral failures after scaffolding if any.

- [x] **Step 3: Implement types, validation, prior snapshot and channel helpers.** Keep probability validation shared and assignments checked against declared ID sets. Constructor does not silently omit invalid zero-prior hypotheses. Compute each group's two signal possibilities using the assigned reporting profiles, with endpoint-safe log helpers:

```rust
fn log_probability(p: f64, positive: bool) -> f64 {
    if positive { p.ln() } else { (-p).ln_1p() }
}
fn log_add(a: f64, b: f64) -> f64 {
    let m = a.max(b);
    if m == f64::NEG_INFINITY { m }
    else { m + ((a - m).exp() + (b - m).exp()).ln() }
}
```

For each signal, add its log probability and every conditional emission log probability; combine the two alternatives with log_add. An empty group has log likelihood zero. Probability zero is negative infinity, not an arbitrary epsilon. Only validated model paths reach private indexed lookups.

- [x] **Step 4: Verify and review.** Targeted testimony tests and `cargo fmt --all -- --check`; report actual results and contracts. Root produces an uncommitted diff against the pre-task snapshot and dispatches a fresh reviewer. Leave all work uncommitted.

### Task 2: Atomic joint conditioning and provenance

**Files:** Modify testimony.rs and tests/testimony.rs only.

**Interfaces:** Consumes Task 1 types/channel helpers. Produces Belief::observe(record: EvidenceRecord) -> Result<(), TestimonyError> and final snapshot marginals.

- [x] **Step 1: Write exact conditioning and transaction tests.** `uncertain_speaker_model()` uses proposition 7, speaker 19, copy/invert profiles 3/8, and four joint hypotheses from truth prior 1/2 and copy prior 3/4; groups 11/12 both accuracy 4/5. `report(id, group, speaker, positive)` creates a typed record. Helpers retrieve marginals by ID, independent of ordering.

```rust
#[test]
fn testimony_independent_signals_keep_the_speaker_profile_persistent() {
    let mut belief = Belief::new(uncertain_speaker_model()).unwrap();
    belief.observe(report(1, 11, 19, true)).unwrap();
    assert!((truth(&belief, 7) - 13.0/20.0).abs() < 1e-12);
    belief.observe(report(2, 12, 19, true)).unwrap();
    assert!((truth(&belief, 7) - 49.0/68.0).abs() < 1e-12);
}
```

Cover the approved fixture matrix: one uncertain speaker 13/20 with profile marginal 3/4; two signals 49/68; shared signal 13/20; duplicate no-op; known-copy independent witnesses 16/17 vs shared 4/5; known inversion 1/5; always-positive and q=1/2 uninformative; verified truth copy posterior 12/13; fresh second independent proposition positive posterior 49/65. The transfer fixture enumerates eight worlds for two independent prior-1/2 propositions and the persistent profile.

Add complete Belief equality after unknown group/speaker/proposition, conflicting ID, contradiction and overflow. Fill 512 records with a known always-positive profile, then accept an identical duplicate and reject new ID 513. Separately, 512 positive reports with profile [0.001,0.001] must leave a nonuniform prior unchanged even though ordinary likelihood products underflow. Preserve a true zero-support contradiction as ZeroEvidence. Shuffle model assignments/declarations and rename sparse IDs; compare corresponding marginals. All normalized snapshots and repeated projections must stay finite, deterministic and pure.

- [x] **Step 2: Run red.** `cargo test -p sugarscape-core testimony`. Missing observe first, then report actual behavioral failures until conditioning is implemented.

- [x] **Step 3: Implement candidate-ledger recomputation and marginals.** Duplicate equality is checked before capacity. Validate references; clone the bounded evidence ledger; add the record; calculate log weights from normalized model priors and the entire candidate ledger, applying verified-proposition constraints and grouping reports by signal group. Normalize by maximum finite log weight. If every weight is negative infinity return ZeroEvidence. Only commit after successful calculation:

```rust
let mut candidate = self.evidence.clone();
candidate.push(record);
let posterior = self.condition(&candidate)?;
self.evidence = candidate;
self.posterior = posterior;
Ok(())
```

The previous posterior is never the base for a full-ledger recomputation. Marginalization sums joint weights for each declared proposition value/profile assignment. Include zero probabilities for unused profiles. No age eviction, random draws, guessing about message provenance, or reading actual game truth.

- [x] **Step 4: Verify and review.** Targeted tests and fmt check; record completed exact fixtures and actual red/green evidence. Independent task review focuses on conditional dependence, full-ledger double counting, transactional failure ordering, and log-space support handling. Leave work uncommitted.

### Task 3: Frozen diagnostics, CLI and guide

**Files:** Create deduction/testimony_diagnostics.rs and CLI/tests/testimony.rs; modify deduction/mod.rs, CLI/src/deduction.rs, docs/deduction.md, and core/tests/testimony.rs.

**Interfaces:** Consumes Belief, snapshot/model types and existing best_accusation. Produces `diagnose_testimony() -> Result<TestimonyReport, TestimonyError>`. Report is serializable, with version testimony-v1, tolerance 1e-12, fixture results, decision results and aggregate passed flag. CLI adds only Mode::Testimony with no flags.

`TestimonyReport` contains `version: String`, `tolerance: f64`, `fixtures: Vec<TestimonyFixtureResult>`, `decisions: Vec<TestimonyDecisionResult>`, `passed: bool`. Each fixture records its name, complete model, supplied records, `references: Vec<TestimonyReferenceCheck>`, `checks: Vec<TestimonyBooleanCheck>`, and passed flag. Numeric reference fields: quantity, expected_exact string, expected f64, actual f64, absolute_error, passed. Boolean check fields: quantity, expected, actual, passed. Contradiction and duplicate cases use explicit Boolean checks of the expected error/no mutation; no fabricated posterior for impossible evidence.

`TestimonyDecision` serializes as the ordinary strings accuse/abstain. Decision results record fixture name, posterior_true, chosen action, chosen utility, exact best reference utility, regret, credulous utility/regret, passed. Export only the report/type names consumers need.

- [x] **Step 1: Freeze definitions and write failing diagnostic/binary tests.** Save testimony-v1 assumptions, all approved exact references, decision costs, and 1e-12 tolerance in source/docs before the first diagnostic collection. Independent rational references: 13/20, 49/68, 16/17, 4/5, 1/5, 12/13, 49/65; decision values 3/10, -3/5 and credulous regret 3/5. Do not derive expected results by running Belief.

```rust
#[test]
fn testimony_command_is_exact_and_deterministic() {
    let run = || std::process::Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["deduction", "testimony"])
        .output().unwrap();
    let a = run(); let b = run();
    assert!(a.status.success());
    assert_eq!(a.stdout, b.stdout);
    let report: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(report["version"], "testimony-v1");
    assert_eq!(report["passed"], true);
}
```

Core tests verify each fixture's exact values and Boolean checks, deterministic report JSON, and aggregate passed iff all numeric/Boolean/decision checks pass. CLI tests reject an unknown flag with exit 2. Cover output I/O error through a CLI unit test with a deliberately failing Write implementation, using the same helper as the command (no /dev/full platform dependency).

Before edits snapshot the current `deduction diagnose` output and a seed-7 run result as ignored reference artifacts, then compare after command integration to protect the frozen baseline. Preserve existing play/resume binary tests.

- [x] **Step 2: Run red.** `cargo test -p sugarscape-core testimony` and `cargo test -p sugarscape-cli --test testimony`. Record actual missing diagnostic/subcommand failures.

- [x] **Step 3: Implement fixed fixture host, decision checks and command.** Run the declared records through Belief; compare to rational constants and required Boolean outcomes. For contradiction, preserve the pre-error belief and inspect the declared ZeroEvidence result. Use the current decision helper and map only Some(0) to accuse, everything else to abstain, with fixed +1/-1/0 utilities. Calculate chosen/credulous utility as `2*p - 1` when accusing and zero when abstaining. Compare optimal utility against the independently fixed fixture reference, including zero for inversion/uniform. Regret is best reference utility minus chosen utility, checked within tolerance rather than silently clamped.

CLI follows the existing emit/Failure convention:

```rust
let report = diagnose_testimony().map_err(|e| invalid(&e.to_string()))?;
let passed = report.passed;
let value = serde_json::to_value(report).expect("finite report serializes");
emit(&value, &mut output)?;
if passed { Ok(()) } else { Err(invalid("testimony reference checks failed")) }
```

This prints useful failed checks before exit 2 and propagates output write/flush failure as exit 1. No Engine/archive reference is passed to the model. New code does not retune or route existing game policies through testimony.

- [x] **Step 4: Collect real output and document limits.** Run `cargo run -p sugarscape-cli -- deduction testimony`, parse its JSON, preserve the first report in this plan's ignored review workspace. Update docs with actual values and a compact command example. Explain persistent strategy vs signal quality, declaration of shared provenance, independent verification requirements, finite support/model contradiction, absence of a claim-aware game controller, and lack of strategic opponent adaptation. Re-run the two baseline output comparisons; changing anything in deduction-v1 is a failure.

- [x] **Step 5: Final checks and review.** Fresh full checks: cargo fmt --all -- --check; cargo clippy --workspace --all-targets --all-features -- -D warnings; cargo test --workspace; wasm-pack test --node crates/sugarscape-wasm; web npm run wasm, npx tsc --noEmit, npx vitest run; python3 -m unittest discover -s studio/tests -t studio. Inspect all exit statuses and preserve logs. Run git diff --check plus whitespace checks for newly created untracked files. Independently review the completed increment against its pre-execution snapshot, including the two baseline output comparisons and preservation of unrelated changes. Remove IMPLEMENTATION_PLAN.md only when all stages/checks/reviews pass. Retain uncommitted evidence; no staging, commit, merge or push.

## Plan self-review

Spec coverage maps model/validation/channel to Task 1, joint inference/provenance/atomicity/marginals to Task 2, and fixed diagnostics/decisions/host/docs/full checks to Task 3. All five Review Focus cases have explicit owning tests. Interfaces use one naming scheme and no later task consumes an undefined method. The plan intentionally does not integrate testimony into game controllers or assign provenance from existing claims. Standing subagent execution and no-commit preferences remain in force.
