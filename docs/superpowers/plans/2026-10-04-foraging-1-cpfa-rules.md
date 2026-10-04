# Foraging 1 CPFA Rule Reference Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Implement and verify CPFA's checked mathematical rules and explicit publication/departure decisions without adding a simulated world.

**Architecture:** A standalone core `foraging` module exposes deterministic functions operating on parameters, supplied observations, waypoint snapshots and caller-supplied uniform variates. Configuration, numerical rules and information decisions have separate responsibilities. No function owns movement, resource truth, memory lifecycle, a server or an RNG.

**Tech Stack:** Existing Rust standard library and `FieldError`; existing Cargo test/format/clippy tools. No dependency, serializer or runtime adapter is added.

**Spec:** [approved research-anchored foraging/construction design](../specs/2026-10-04-foraging-construction-design.md).

## Global Constraints

- Status: written spec approved by the user on 2026-10-04; implementation plan approved by the user on 2026-10-04. Implementation and independent reviews complete; integration into `main` remains pending. Execution method: subagent-driven development. Scientific registration/execution remain separate.
- Implement F1 only. F2 world reconstruction, F3 passages, F4 excavation and F5 termite comparisons retain their own design gates.
- Seven parameters are explicit; no claimed evolved defaults. Probabilities are finite in `[0,1]`; `omega` is finite in `[0,4*pi]`; decay rates are finite and nonnegative.
- Counts are integers from 0 through 256; fidelity/publication rates are finite in `[0,256]`. These are supplied reference-utility bounds, not biological limits.
- Ages are finite and nonnegative; uniform variates are finite in `[0,1)`; waypoint strengths are finite in `[0,1]`. Reject invalid original inputs before a decision.
- Use the displayed lower-tail Poisson CDF and strict `probability > draw`. Preserve the source prose discrepancy in documentation.
- Fidelity has priority over recruitment, then uninformed departure. Publication is separate and uses an independent draw. No valid find means no new publication.
- Site IDs are abstract caller identifiers. Recruitment sees a supplied snapshot, not the world's remaining-resource state. No hidden RNG draws or state changes.
- Uniform waypoint selection is a supplied comparison variant; strength weighting is the inspected later ARGoS variant. Paper equality and later-source strict threshold conventions are explicit.
- No world, scheduler, heading sampler, food ledger, server, ModelKind, CLI, WASM export, UI, new dependency or source-code copying.
- Preserve existing runtime behavior, including Burrow replay. Use a separate global worktree at execution time; do not clean up siblings.

## Review Focus

1. Private fidelity would win despite an invalid unused waypoint/draw: validate the entire call first, including inactive records (Task 2).
2. Very large finite ages/rates overflow their product: decay returns its correct finite limit, and zero factors remain exactly one (Task 1).
3. A selection ticket lies exactly at a boundary or the largest valid draw rounds poorly: use half-open intervals and always choose a valid last interval (Task 2).
4. Duplicate record IDs occur on expired records or multiple IDs refer to the same site: reject all duplicate IDs; permit duplicate sites (Task 2).
5. Callers conflate previous memory with current global validity: demonstrate stale but active recruitment and separate unsuccessful-publication/fidelity inputs through public API tests (Task 3).

---

## Staging and file responsibilities

At execution, use the git-worktrees skill to create branch `foraging-1-cpfa-rules` in `~/.config/superpowers/worktrees/SugarScape/foraging-1-cpfa-rules/`. Check the actual starting main revision and clean state; do not hardcode this plan's documentation commit as a runtime base.

Create root `IMPLEMENTATION_PLAN.md` with three stages matching Tasks 1–3. Each has the required Goal, Success Criteria, Tests and Status fields. Update it after each reviewed task, remove it when complete, and preserve review rulings and actual acceptance commands/results in this plan. Use fresh implementation/review agents per task and a final whole-branch review. After three failed attempts at an issue, stop that approach and document/reassess under the project guidelines.

| File | Responsibility |
|---|---|
| `crates/sugarscape-core/src/foraging/mod.rs` | Public exports and scope/source rustdoc |
| `crates/sugarscape-core/src/foraging/config.rs` | Seven parameters, constants and reusable checked-input helpers |
| `crates/sugarscape-core/src/foraging/rules.rs` | Checked Poisson CDF, variation and exponential strength |
| `crates/sugarscape-core/src/foraging/information.rs` | Typed records/variants and deterministic information choices |
| `crates/sugarscape-core/src/foraging/tests/mod.rs` | Private test modules |
| `crates/sugarscape-core/src/foraging/tests/rules.rs` | Parameter/numerical behavior tests |
| `crates/sugarscape-core/src/foraging/tests/information.rs` | Decisions, validation, variants and selection boundaries |
| `crates/sugarscape-core/tests/foraging_reference.rs` | Public-API examples of independent publication, stale information and no input mutation |
| `crates/sugarscape-core/src/lib.rs` | Add only `pub mod foraging;` |
| `docs/foraging.md` | How to use this reference and what it does/does not establish |

All checked functions return `Result<T, Vec<FieldError>>`, following core configuration conventions. Test numeric outputs with absolute tolerance `1e-12` unless exact endpoint behavior is specified. Monotonicity permits `1e-14` roundoff. These are engineering tolerances, not measured biological error bars.

### Task 1: Checked parameters and mathematical reference

**Files:** Create `foraging/mod.rs`, `config.rs`, `rules.rs`, `tests/mod.rs`, `tests/rules.rs` under the core source directory; modify core `lib.rs` and the execution tracker.

**Interfaces:**

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CpfaParameters {
    pub p_search: f64,
    pub p_return: f64,
    pub omega: f64,
    pub lambda_informed: f64,
    pub lambda_fidelity: f64,
    pub lambda_publish: f64,
    pub lambda_waypoint: f64,
}
// Constants exported from mod.rs:
pub const MAX_RESOURCE_COUNT: u32 = 256;
pub const MAX_INFORMATION_RATE: f64 = 256.0;
pub const WAYPOINT_THRESHOLD: f64 = 0.001;
// Methods/functions produced by Task 1:
// CpfaParameters::validate(&self) -> Result<(), Vec<FieldError>>
// uninformed_variation(omega: f64) -> Result<f64, Vec<FieldError>>
// informed_variation(omega: f64, rate: f64, age: f64) -> Result<f64, Vec<FieldError>>
// poisson_cdf(count: u32, rate: f64) -> Result<f64, Vec<FieldError>>
// waypoint_strength(rate: f64, age: f64) -> Result<f64, Vec<FieldError>>
```

No serde/default derives are required. `p_search`/`p_return` are validated but not executed as movement transitions in F1. Count typing prevents fractional inputs; reject values above 256 without narrowing. Private `pub(super)` helpers append contextual errors for a finite closed interval, nonnegative finite values, `[0,1)` draws and counts; Task 2 may reuse them. Error field names match parameter names or function argument names. Parameter validation aggregates errors in declaration order.

- [x] **Step 1: Write failing tests and module declarations.** Add the module export and test wiring, then tests that refer to absent rule APIs; do not add successful placeholder bodies. Pin finite bounds, all seven invalid fields, NaN/infinities, negative inputs, count 257, and exact accepted endpoints. Mathematical expectations include:

```rust
#[test]
fn poisson_reference_includes_the_count_endpoint() {
    let actual = poisson_cdf(1, 1.0).unwrap();
    assert!((actual - 0.7357588823428846).abs() <= 1e-12);
}
#[test]
fn finite_decay_product_overflow_expires_instead_of_nan() {
    assert_eq!(waypoint_strength(f64::MAX, f64::MAX).unwrap(), 0.0);
}
```

Additional independent CDF fixtures: `(0,1)=0.3678794411714423`, `(4,2)=0.9473469826562888`, `(128,128)=0.5234844486527008`, `(256,256)=0.5166143010472157`. These were calculated using Python standard-library Decimal at 90-digit precision while planning; tests use literals, not the production implementation. For rate zero, every accepted count yields exactly one. Check count monotonicity for all 0–256 at rates 0, 1, 20, 128 and 256; check inverse rate monotonicity at counts 0, 1, 64 and 256. Check `(256,1)` stays within `[0,1]` despite accumulation roundoff.

Variation cases: age zero yields `4*pi`; zero decay yields `4*pi`; `omega=4*pi` stays constant; positive decay approaches `omega`; with `omega=0, rate=1, age=ln(2)` the expected value is `2*pi`. Verify increasing age never increases variation beyond tolerance. Decay fixtures: zero factors produce exactly one, `(rate=1,age=ln(2))` approximately 0.5, large positive products yield zero. Check zero times `f64::MAX` explicitly.

- [x] **Step 2: Run RED.** Run `cargo test -p sugarscape-core foraging::tests::rules`; expected failure is the absent module/API rather than an unrelated build failure. Record the actual failure.

- [x] **Step 3: Implement checked numerical functions.** Validate before calculation. Implement a small internal exponential helper handling zero factors, positive overflow and ordinary finite products. The core recurrence is:

```rust
// Called only after count/rate validation.
let mut term = (-rate).exp();
let mut total = term;
for k in 1..=count {
    term *= rate / f64::from(k);
    total += term;
}
// Correct only a roundoff-scale endpoint excursion; a larger excursion
// produces a contextual FieldError rather than hiding a numerical defect.
```

Use `1e-12` as the maximum permitted endpoint correction for the CDF; reject nonfinite output or larger excursions. Do not broadly clamp arbitrary invalid calculations. `exp(-256)` remains representable, so the stated bounds avoid recurrence underflow/overflow. Use standard `exp`, consistent with the spec; cross-platform bit identity is not a claim of this F1 API. Keep bounds and source provenance in rustdoc.

- [x] **Step 4: Run GREEN and checks.** Run the targeted test command, `cargo fmt --all -- --check`, `cargo clippy -p sugarscape-core --all-targets -- -D warnings`, then `cargo test --workspace`. Fix failures without bypassing checks. Run formatting before the check if needed.

- [x] **Step 5: Commit and review.** Stage only Task 1 files and tracker; commit `feat(foraging): add checked CPFA numerical reference rules`. Have a fresh reviewer check the spec/plan and actual tests before Task 2 begins.

### Task 2: Explicit publication and departure decisions

**Files:** Create `foraging/information.rs` and `tests/information.rs`; update `mod.rs`, `tests/mod.rs` and tracker. Consume Task 1 validation/math APIs without altering their contracts.

**Interfaces:**

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FindRecord { pub site: u64, pub count: u32 }
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Waypoint { pub id: u64, pub site: u64, pub strength: f64 }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaypointSelection { UniformComparison, LaterArgosStrengthWeighted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaypointThreshold { PaperBelow, LaterArgosStrict }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Departure {
    Uninformed,
    SiteFidelity { site: u64 },
    Recruitment { waypoint: u64, site: u64 },
}
// Produced public signatures:
// publication(parameters: &CpfaParameters, find: Option<FindRecord>, draw: f64)
//   -> Result<Option<u64>, Vec<FieldError>>
// departure(parameters: &CpfaParameters, memory: Option<FindRecord>,
//   waypoints: &[Waypoint], selection: WaypointSelection,
//   threshold: WaypointThreshold, fidelity_draw: f64, recruitment_draw: f64)
//   -> Result<Departure, Vec<FieldError>>
```

The returned publication site is a request, not a server insertion. Both functions validate all supplied parameters and draws even if no-find/fidelity/empty-list branches make some inputs irrelevant. Departure also validates the whole snapshot, inactive records included, before calculating probabilities. Duplicate IDs are rejected with indexed contextual errors; duplicate sites are legal. Use `BTreeSet` as in existing bounded core validation. Input structs have no defaults; identifiers use the whole `u64` range without sentinel meanings.

- [x] **Step 1: Write failing tests.** Verify independent draws, fidelity priority, uniform and weighted fallbacks, both threshold variants, inactive filtering, no-find publication and absent-memory departure. Use exact weighted intervals with strengths 0.25 and 0.75 and IDs 11/22:

```rust
#[test]
fn weighted_boundary_belongs_to_the_next_record() {
    let parameters = valid_parameters(); // Test helper supplies all seven fields.
    let records = [
        Waypoint { id: 11, site: 101, strength: 0.25 },
        Waypoint { id: 22, site: 202, strength: 0.75 },
    ];
    assert_eq!(
        departure(&parameters, None, &records,
            WaypointSelection::LaterArgosStrengthWeighted,
            WaypointThreshold::LaterArgosStrict, 0.0, 0.25).unwrap(),
        Departure::Recruitment { waypoint: 22, site: 202 }
    );
}
```

Define `valid_parameters()` locally in this test module with probabilities 0.5, `omega=1`, informed/waypoint decay 1, fidelity/publication rates 1. Test helper construction is test-only, not a public default. With `count=1,rate=1`, draw 0.7 succeeds and 0.8 fails using the independent CDF fixture from Task 1. Use a lambda-zero fixture for exact probability-one; obtain a computed probability once only to test strict comparison equality, not as an independent numeric oracle.

For every API, inject NaN, positive/negative infinity, negative draw, draw one and the largest valid draw `f64::from_bits(1.0_f64.to_bits()-1)`. Test an invalid inactive strength or duplicate ID still rejects when fidelity would otherwise succeed. Test valid duplicate sites with distinct IDs, all inactive records, empty snapshot, exact 0.001 strength under both threshold policies, three uniform intervals at draws 0, 0.5 and the largest valid draw, and a weighted largest-draw case with strengths 0.001 and 0.2. Check indexed error field `waypoints[1].strength` for an invalid second record and duplicate-ID field `waypoints[1].id`.

- [x] **Step 2: Run RED.** `cargo test -p sugarscape-core foraging::tests::information` must fail for missing information APIs. Record that failure before implementing them.

- [x] **Step 3: Implement validated decisions without RNG ownership.** After validation, publication returns None for no find; otherwise its strict CDF comparison returns the supplied site. Departure first tests valid memory using `lambda_fidelity`, then builds the eligible ordered record view, then selects or returns Uninformed. The threshold predicates are explicit:

```rust
let active = match threshold {
    WaypointThreshold::PaperBelow => strength >= WAYPOINT_THRESHOLD,
    WaypointThreshold::LaterArgosStrict => strength > WAYPOINT_THRESHOLD,
};
```

Uniform selection uses equal intervals. Weighted selection sums active strengths in stable order and compares the original draw to cumulative normalized boundaries. Use the same ordered sum for total and the final cumulative value, making the last normalized upper boundary exactly one; this avoids multiplying a largest valid draw into a rounded out-of-range ticket. At exact equality advance to the next interval. No active records requires no selection calculation. Do not mutate snapshots, consult a resource grid, expire records internally, or suppress invalid inputs because fidelity succeeds. Parameter errors precede observation/snapshot errors; append indexed snapshot errors in order.

- [x] **Step 4: Run GREEN and regression checks.** Run `cargo test -p sugarscape-core foraging::`, `cargo fmt --all -- --check`, `cargo clippy -p sugarscape-core --all-targets -- -D warnings`, and `cargo test --workspace`. Self-review source labels and error contexts; keep information policy enums explicit rather than a silent default.

- [x] **Step 5: Commit and review.** Stage only Task 2 files and tracker; commit `feat(foraging): add explicit CPFA information decision variants`. Fresh task review must pass before Task 3.

### Task 3: Public reference acceptance and research handoff

**Files:** Create `crates/sugarscape-core/tests/foraging_reference.rs` and `docs/foraging.md`; update module rustdoc, source audit/design status, programme/papers links and tracker. Changes to existing documentation must accurately record actual acceptance and preserve separate scientific gates.

**Interfaces:** Consume the public exports from Tasks 1–2. Add no runtime API. The integration test is a caller example of the stateless contract, not a new simulator.

- [x] **Step 1: Write public-API acceptance tests.** A successful find at site 77 with count one and rates one, publication draw 0.8 and fidelity draw 0.7, must suppress publication while retaining private fidelity. A second call with publication draw 0.7 and fidelity draw 0.8 must publish while taking recruitment if a waypoint exists. Neither choice implies that the other draw was reused. Also test no publication for `find=None` alongside valid departure memory: an empty return and caller-retained memory are separate inputs.

```rust
use sugarscape_core::foraging::{
    departure, publication, CpfaParameters, Departure, FindRecord,
    Waypoint, WaypointSelection, WaypointThreshold,
};

#[test]
fn recruitment_accepts_an_active_record_without_resource_truth() {
    let p = CpfaParameters {
        p_search: 0.5, p_return: 0.5, omega: 1.0,
        lambda_informed: 1.0, lambda_fidelity: 1.0,
        lambda_publish: 1.0, lambda_waypoint: 1.0,
    };
    // The caller may know site 77 is depleted; F1 receives no such truth.
    let records = [Waypoint { id: 9, site: 77, strength: 0.8 }];
    let before = records;
    assert_eq!(departure(&p, None, &records,
        WaypointSelection::LaterArgosStrengthWeighted,
        WaypointThreshold::LaterArgosStrict, 0.0, 0.0).unwrap(),
        Departure::Recruitment { waypoint: 9, site: 77 });
    assert_eq!(records, before);
}
```

These are composition regression tests. They may already pass when Tasks 1–2 are correct; record that honestly instead of inventing a RED phase or changing production behavior to force one. Use public imports to catch forgotten re-exports. Demonstrate publication and departure in rustdoc with the same explicit parameters and explain that the caller owns publication order, observations and random variates.

- [x] **Step 2: Write reference usage/evidence documentation.** `docs/foraging.md` must include the source equation/prose mismatch, lower-tail definition, engineering bounds, threshold and selection provenance, all seven parameter meanings, independent variates, contextual errors, and public examples. State that F1 verifies rules but neither simulates foraging nor reproduces evolved performance. Link the source audit, approved spec and this plan. State the F2 source-reconciliation questions without inserting new runtime commitments. Update the existing construction/ontology/papers links to the reference guide; mark implementation complete only after review and acceptance, not integrated before merging.

- [x] **Step 3: Run acceptance sequentially.** Run:

```bash
cargo test -p sugarscape-core --test foraging_reference
cargo test -p sugarscape-core foraging::
cargo test -p sugarscape-core burrow::
cargo test --workspace
cargo fmt --all -- --check
cargo clippy -p sugarscape-core --all-targets -- -D warnings
git diff --check
```

Commands shown together are separate verification actions, not a shell script to add to the repository. Run them sequentially to avoid contention. Check local Markdown targets and source-scope assertions: Cargo manifests/lockfile, existing Burrow runtime, CLI/WASM and browser files must have no changes. No web rebuild/survey/ARGoS run is needed for a core-only rule addition unless an observed change/failure gives a specific reason. Record test counts, revisions, commands and actual outcomes; do not reuse Burrow's earlier numbers as F1 evidence.

- [x] **Step 4: Finish tracker and commit.** Mark stages complete only when tests and task reviews pass, extract rulings/evidence into this durable plan, then remove `IMPLEMENTATION_PLAN.md`. Stage the listed acceptance/documentation files and commit `docs(foraging): document verified CPFA rule reference and next world gate`. Documentation-only changes after passing checks do not require repeating runtime tests; verify their diff/links.

- [x] **Step 5: Whole-branch review and handoff.** Request an independent whole-branch review against the spec, source audit and plan. Resolve findings with scoped fixes and appropriate reruns. Report the actual final revision, checks, source variants, material rulings and limits. Preserve the branch/worktree for user integration choice; neither merge/push nor scientific execution is part of this plan's implementation acceptance.

## Plan self-review and execution gate

Coverage: Task 1 implements the parameter/numeric contracts; Task 2 implements the separate information choices and all source variants; Task 3 exercises the public boundary and preserves the research handoff. Each Review Focus item has an owning test. Types/signatures match across tasks; later worlds are explicitly excluded. No new dependencies or schemas are required.

The written design is approved. The user approved this implementation plan on 2026-10-04; execution is now authorized. Preserve the user's standing subagent-driven method; do not ask them to choose it again. After plan approval, read the execution/worktree/review skills and proceed task by task.


## Task 3 acceptance evidence and execution rulings (2026-10-04)

Starting revision `72e851b` provides independently approved Tasks 1–2. Public composition tests passed immediately; no artificial RED phase or production behavior change was made. Three tests cover independent publication/fidelity variates, empty publication with retained memory, and immutable active recruitment without resource truth. The public rustdoc example also passed in workspace doc tests.

Final-source acceptance: `cargo test -p sugarscape-core --test foraging_reference` (3 passed), `cargo test -p sugarscape-core foraging::` (27 passed), `cargo test -p sugarscape-core burrow::` (102 passed), `cargo test --workspace` (2,236 passed, 103 ignored, 0 failed across 36 test-result groups, including doc tests), `cargo fmt --all -- --check`, `cargo clippy -p sugarscape-core --all-targets -- -D warnings`, and `git diff --check` all exited 0. Local Markdown targets resolved. Source-scope inspection found no Cargo manifest/lockfile, existing Burrow runtime, CLI/WASM or browser changes.

At Task 3 acceptance, the controller retained `IMPLEMENTATION_PLAN.md` through task and whole-branch review and kept Stage 3 In Progress pending those verdicts. The review closure below records their subsequent approval and tracker removal. Integration into `main` remains pending; no merge, push or scientific execution occurred. The lower-tail/prose discrepancy, supplied engineering bounds, paper/later-source threshold distinction and uniform/later-source selection distinction remain documented. F2 must reconcile historical simulator provenance and world conventions before implementation; F3–F5 and scientific gates remain separate.


## Independent review and final acceptance closure (2026-10-04)

The branch started from main at `2fbdffb`. Execution setup was committed as `54de455`; Task 1 as `0271a5b`, Task 2 as `72e851b`, and Task 3 as `0f286fe`. Fresh task reviewers approved both spec compliance and quality for all three tasks. All cannot-verify items were resolved against earlier review/source evidence, actual final-source logs and changed-path/link inspection.

Whole-branch review at `0f286fe` returned Ready to merge: Yes, with no Critical/Important findings. Its one code-related minor was a monotonicity-test tolerance mismatch with the plan. Commit `ea3ae98` tightened only the three monotonicity comparisons from `1e-12` to `1e-14`; independent numerical fixture tolerances remain `1e-12`. Scoped re-review confirmed the finding ADDRESSED with no new breakage or out-of-scope observations. The Task 2 verification-scheduling minor was closed by final review: successful source-final checks and subsequent sequential acceptance establish no verification gap.

Final acceptance on `ea3ae98`: `cargo test -p sugarscape-core foraging::tests::rules` passed 14 tests; `cargo test --workspace` passed 2236, failed 0, with 103 existing ignored across 36 result groups; `cargo fmt --all -- --check` and `cargo clippy -p sugarscape-core --all-targets -- -D warnings` exited 0. Task 3 additionally verified 3 public integration tests, 27 foraging tests, 102 Burrow tests and a compiled rustdoc example. Controller checked all 59 local Markdown targets and the branch's allowed changed paths. Final correction logs and review reports were retained locally in `/tmp/foraging-f1-evidence-20261004/`; Task 3 command logs remain `/tmp/f1-task3-*.log`. These local files are supplemental; this section is the durable acceptance record.

After all reviews passed, the execution tracker was removed and statuses updated in a documentation-only closure. No production/runtime/test changes followed the accepted correction. The branch/worktree are preserved for the user's integration choice; no merge, push or scientific registration/run occurred.

### Rulings made during execution

Ruling: Implement the draw-validation helper in Task 2 at its first production use rather than leaving it unused in Task 1 — preserves clean clippy without temporary dead-code allowances — cost if wrong: small helper relocation during review.

Ruling: Keep IMPLEMENTATION_PLAN.md until task and final reviews pass, then remove it in a documentation-only closure commit — completion must reflect actual acceptance rather than anticipate the reviewer — cost if wrong: an additional bookkeeping commit.

Final review set aside historical/full-model reproduction, F2 world/server lifecycle, F3–F5 scientific outcomes and cross-platform bit identity because the approved F1 design explicitly excludes them. They remain future design/research questions, not unsupported F1 completion claims. The next design is F2 fixed-world reconstruction with reconciled movement, timing, sensing and scoring conventions.
