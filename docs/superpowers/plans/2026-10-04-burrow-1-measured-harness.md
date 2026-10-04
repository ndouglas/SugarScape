# Burrow 1 Measured Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build immutable experiment archives and deterministic saved-record analysis for the approved Burrow 1 protocol, without running the scientific campaign.

**Architecture:** Add a physical-record validator inside core, where it can reuse checked transactions and material accounting. Add separate manifest, archive and report modules in the existing native survey binary; keep raw execution separate from saved-only analysis. The candidate manifest stays unregistered until a separate executable-manifest review.

**Tech Stack:** Rust, serde/serde_json, existing survey sha2 0.10.9, existing `stats::paired_summary`, Cargo; no new dependency.

**Spec:** [approved measured protocol](../specs/2026-10-04-burrow-1-measured-protocol.md); [candidate manifest](../specs/2026-10-04-burrow-1-draft-manifest.json).

## Global Constraints

- Status: implementation plan approved 2026-10-04; engineering implementation and acceptance complete; controller-owned fresh branch review pending. Protocol design approved 2026-10-04. Scientific execution and executable registration remain separately gated.
- Scientific seeds: decimal strings 10001–10040; construction seeds: 7 and 8. Never execute scientific seeds during implementation, tests or acceptance.
- Exactly 32 scientific conditions / 1280 episodes; 18 construction conditions / 36 episodes. Growing scientific denominator 4096; corridor construction denominator 512; choice denominator zero.
- Exactly six primary estimates: three signed contrasts × two rates. No overall Holds/Fails verdict, tuning, selected subset, sample-fit band or animal calibration claim.
- Legacy lab config, controller, setup, Episode schema and seeded continuation remain unchanged. Validation never invokes `World::step` or `run_episode`.
- Canonical condition order then ascending seed; serial execution, exclusive writes, new directories, full revisions, SHA-256 binding, complete raw Episode before analysis; no silent resume.
- Retain censored material and zero/null rates. Canonical analysis includes no clock timing or absolute machine paths.
- Execute this plan first in its own global worktree, with fresh implementation/review subagents per task and a whole-branch review. Do not combine its branch with resource access.

## Review Focus

1. An archive omits or duplicates a replicate: reject with condition/seed, never shrink the denominator (Task 3).
2. A raw path escapes its archive or bytes change after indexing: reject before parsing/analysis (Task 3).
3. A plausible event invents material, repeats a worker within a round or falsifies final inventory: reconstruct and reject without stochastic replay (Task 1).
4. An undelivered unit or within-round disposal distorts latency: preserve censoring and distinguish ticks from opportunity indices (Task 4).
5. A default invocation accidentally runs experiments, or reduction checks compare fingerprints: default prints only; compare config-independent action/choice projections (Tasks 2 and 4).

---

## Staging and file map

At execution create root `IMPLEMENTATION_PLAN.md` with four stages matching Tasks 1–4, each containing Goal, Success Criteria, Tests and Status. Update each status as work proceeds; remove it when all stages are complete. Persist rulings and validation evidence in this plan before removing the tracker.

Core `burrow/validation.rs` owns physical replay validation. Survey `claims/burrow.rs` owns strict manifest types and CLI routing; `burrow_archive.rs` owns provenance and immutable I/O; `burrow_report.rs` owns saved-only statistics. Each has a focused sibling test module. Keep protection modules unchanged rather than generalizing their archive format. Existing `survey/src/stats.rs` is reused unchanged.

### Task 1: Validate a saved physical episode without running its controller

**Files:**
- Create: `crates/sugarscape-core/src/burrow/validation.rs`.
- Create: `crates/sugarscape-core/src/burrow/tests/validation.rs`.
- Modify: `crates/sugarscape-core/src/burrow/mod.rs`, `crates/sugarscape-core/src/burrow/tests/mod.rs`.
- Modify narrowly if needed: `crates/sugarscape-core/src/burrow/runner.rs` (expose deterministic recording helpers within burrow), `ledger.rs` (reuse existing history methods).

**Interfaces:**
- Consumes existing `World::new(LabConfig,u64)`, crate-internal `World::from_setup`, `World::apply(u32,Action)`, `Ledger::{new,record,finish}`, `Recording::{new,record}` and `run_episode(LabConfig,u64,RunOptions)` for construction tests only.
- Produces public `validate_episode(record: &Episode, options: &RunOptions) -> Result<(), String>` reexported from burrow. It validates the constructor-generated setup, not arbitrary external setups. Error text includes event/frame/series indices; archive adds condition/seed context.

- [x] **Step 1: Write behavioral tests.** A seed-7 growing episode passes. Tampering with a dig's material ID fails; duplicating a worker in one round fails; changing final inventory fails. Add legal blocked-action, relay handoff, old-material choice, malformed fingerprint, short horizon and off-cadence frame cases.

```rust
#[test]
fn validation_rejects_forged_final_inventory() {
    let options = RunOptions { ticks: 8, sample_every: 3 };
    let mut record = run_episode(LabConfig::default(), 7, options.clone()).unwrap();
    record.final_summary.inventory.disposed += 1;
    assert!(validate_episode(&record, &options).is_err());
}
#[test]
fn validation_accepts_checked_construction_record() {
    let options = RunOptions { ticks: 8, sample_every: 3 };
    let record = run_episode(LabConfig::default(), 7, options.clone()).unwrap();
    assert_eq!(validate_episode(&record, &options), Ok(()));
}
```

- [x] **Step 2: Run RED.** `cargo test -p sugarscape-core burrow::tests::validation` must fail because the validation API does not exist.

- [x] **Step 3: Implement deterministic recorded-action reconstruction.** Parse seed as canonical decimal u64; construct `World::new(record.config.clone(), seed)` and require exact setup equality. Reuse option validation and limits before allocating replay work. For growing/corridor require requested/completed ticks equal options, stop `tick_budget_exhausted`, exactly workers × ticks events and final clock start + ticks. Require each round's events have its clock and every worker exactly once; order is recorded, not regenerated. For choice require exactly one valid locally observed selection, no events/completed ticks/opportunities, stop `choice_selected` and unchanged start clock. Choice final target assignment can change fingerprints, so retain IDs rather than claiming recomputation.

For each event reject bad worker/coordinates before indexing, set the replay clock to its round, derive pre-action target exit distance, apply the recorded action with the existing checked physics, require the resulting ActionEvent equal the saved event, check invariants, and feed the existing ledger/work recorder. This is transaction replay with no RNG consumption:

```rust
let actual = world.apply(saved.worker, saved.action.clone());
if actual != *saved {
    return Err(format!("event {index}: checked transaction differs"));
}
world.check_invariants().map_err(|e| format!("event {index}: {e}"))?;
recording.record(&actual, &world, distance);
```

Compare recomputed inventory, integer action/worker/spatial/travel summaries, deliveries, frozen dig distances and derived rates against the saved values. Validate choice targets against the ordinary local frontier at the corresponding pre-action state; enforce increasing trace clocks and valid worker IDs. Require exact expected sampled clocks, initial/terminal records, and ASCII projections from replayed physical states. Check storage collection sizes, record sums, ASCII bytes and cell bounds. Check BFS counters are finite integers, sampled cumulative values are monotone, peaks fit geometry, and final/sampled diagnostics agree at shared clocks. Controller search counters and RNG fingerprints cannot be independently authenticated by physical transactions: retain their provenance-bound values, require 16 lower-case hex fingerprint format, and document that limit. Do not replace this with seeded controller replay or claim policy fidelity from fingerprints.

- [x] **Step 4: Run GREEN and regressions.** `cargo test -p sugarscape-core burrow::` and `cargo fmt --all -- --check`. Verify unchanged seed-7 native Episode JSON before/after using the existing CLI on the same construction config/options; validation must have no effect on runtime bytes.

- [x] **Step 5: Commit working validator.** Stage the exact core files listed above and the execution tracker. Commit `feat(burrow): validate saved physical transactions without controller replay` after core tests and lint pass.

### Task 2: Declare the complete manifest and a non-running survey route

**Files:**
- Create: `survey/src/claims/burrow.rs`, `survey/src/claims/burrow_tests.rs`.
- Modify: `survey/src/claims/mod.rs`, `survey/src/main.rs`.
- Read unchanged: `docs/superpowers/specs/2026-10-04-burrow-1-draft-manifest.json`.

**Interfaces:**
- Produces strict serde types below, with `deny_unknown_fields`; seed fields remain strings. `ConstructionCondition` uses the same `Condition` shape, so use `Vec<Condition>` for both panels rather than a redundant second type.

```rust
struct Manifest {
    schema: String, status: String, execution_authorized: bool,
    protocol: String, engine_baseline: String,
    scientific_seeds: Vec<String>, construction_seeds: Vec<String>,
    expected_conditions: usize, expected_scientific_episodes: usize,
    expected_construction_conditions: usize, expected_construction_episodes: usize,
    primary_outcomes: Vec<String>, primary_contrasts: Vec<Contrast>,
    negative_control: NegativeControl,
    conditions: Vec<Condition>, construction_conditions: Vec<Condition>,
}
struct Contrast { id: String, plus: String, minus: String }
struct NegativeControl { plus: String, minus: String, comparison: String }
struct Condition { id: String, panel: String, config: LabConfig, options: RunOptions }
```

Make fields `pub(super)` where the sibling archive/report modules consume them; derive Clone/Debug/PartialEq/Serialize/Deserialize on these types. No serde defaults on manifest fields.
- Produces `manifest() -> Result<Manifest,String>`, `validate_manifest(&Manifest) -> Result<(),String>`, `manifest_bytes() -> &'static [u8]`, `manifest_sha256() -> String`, `cli(args: &[String]) -> Result<(),String>`.
- Produces `Panel { Scientific, Construction }` with Clone/Copy/Debug/PartialEq/Eq/serde and snake_case strings, and `RunKey { condition: String, seed: String }` with Clone/Debug/PartialEq/Eq/Ord/PartialOrd/serde, plus `expected_keys(&Manifest, Panel) -> Vec<RunKey>` in canonical declared order (ascending seeds inside each condition).
- Exact schema fields above match the committed candidate. Validate each declared count, contrast and negative-control identity as well as the complete config/options values.

- [x] **Step 1: Write manifest/default-route tests.** Assert 1280 scientific and 36 construction keys, unique IDs, all growing budgets 4096, corridors 512, choice zero committed opportunities, all seven lab fields explicit and seeds disjoint. Reject unknown fields, missing lab fields even though LabConfig has defaults, duplicate IDs, changed weights, noncanonical seed strings and changed contrasts. A no-argument route must only print the manifest.

```rust
#[test]
fn burrow_manifest_has_all_scientific_replicates() {
    let m = manifest().unwrap();
    assert_eq!(expected_keys(&m, Panel::Scientific).len(), 1280);
}
#[test]
fn burrow_manifest_is_not_execution_authorization() {
    let m = manifest().unwrap();
    assert!(!m.execution_authorized);
}
```

- [x] **Step 2: Run RED.** `cargo test --manifest-path survey/Cargo.toml burrow_manifest` must fail at missing module/API before implementation.

- [x] **Step 3: Implement manifest identity and routing.** Embed the candidate with `include_bytes!` from its existing repo path. Hash exact bytes using `sha2::{Digest,Sha256}`. Validate raw JSON config field presence before deserializing defaulted LabConfig; require exact candidate identity and explicitly resolved values, not merely matching counts. Use `World::new(config,7)` for setup validation with zero stepping. Verify current manifest hash `3ef020036fa96ebc5a440480dccfcab64fd8991dfc2ff2dc4129b05466acb890`; a reviewed amendment must update its expected identity deliberately.

```rust
pub fn manifest_sha256() -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(manifest_bytes()))
}
```

Add `--burrow` routing before the ordinary claim runner. Default and `--manifest` print resolved manifest only; `--help` explains gates. Unknown/duplicate/conflicting flags fail. At this task boundary reject `--run` and `--analyze` with a descriptive unavailable-route error; Task 3 installs archive execution and Task 4 saved analysis. Do not register a generic Claim or add a Holds/Fails classifier.

- [x] **Step 4: Run GREEN.** `cargo test --manifest-path survey/Cargo.toml burrow_manifest`; `cargo run --manifest-path survey/Cargo.toml -- --burrow`; `cargo fmt --manifest-path survey/Cargo.toml -- --check`. Inspect stdout JSON and verify no output directories or episode calls occur.

- [x] **Step 5: Commit.** Stage the four listed survey files and tracker; commit `feat(survey): declare burrow manifest without implicit experiment execution` after the full existing survey tests pass.

### Task 3: Write immutable raw archives and reject invalid saved evidence

**Files:**
- Create: `survey/src/claims/burrow_archive.rs`, `survey/src/claims/burrow_archive_tests.rs`.
- Modify: `survey/src/claims/mod.rs`, `survey/src/claims/burrow.rs`.
- Read for conventions: `survey/src/claims/protection_archive.rs`; do not borrow its weaker hash-free schema.

**Interfaces:**
- Consumes Task 1 validator and Task 2 manifest/key functions; import `sha2::Digest` for hash calls.
- Produces `run(panel: Panel, protocol_revision: &str, approval_context: &str, out: &Path) -> Result<(),String>` and `load(index_path: &Path) -> Result<Archive,String>`.
- `Archive { index: Index, records: BTreeMap<RunKey,Episode> }` is in-memory validated data.
- Serializable `Index { schema: String, code_revision: String, protocol_revision: String, manifest_sha256: String, manifest: Manifest, panel: Panel, approval_context: String, expected_keys: Vec<RunKey>, completed: bool, runs: Vec<RawRef> }`.
- `RawRef { key: RunKey, path: String, sha256: String }`; `Envelope { schema: String, key: RunKey, code_revision: String, protocol_revision: String, manifest_sha256: String, options: RunOptions, episode: Episode }`. Schema literal `burrow-archive-v1`. No elapsed time in canonical records.

- [x] **Step 1: Write immutable-I/O and rejection tests.** Use real seed-7/8 construction episodes through a private test fixture manifest, clearly labelled `construction_fixture` and rejected by public canonical loading. Test missing/duplicate/extra keys; config/seed/setup/options/horizon/stop mismatch; hash tampering; absolute/parent paths and symlink escape; already-existing output; interruption after one raw record. Public CLI integration uses the declared 36 construction episodes, never scientific seeds.

```rust
#[test]
fn burrow_archive_does_not_overwrite_existing_directory() {
    let out = std::env::temp_dir().join(format!("burrow-existing-{}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let result = run(Panel::Construction, "bad-revision", "construction test", &out);
    std::fs::remove_dir(&out).unwrap();
    assert!(result.is_err());
}
```

For fixture tests implement `#[cfg(test)]` fixture builders inside the archive test module using the same writer/validator with an explicitly passed expected manifest; no alternate production route accepts arbitrary subsets. No test edits the committed scientific manifest to make a campaign small. Allocate unique temporary directories using an atomic counter plus process ID, following the existing archive-test helpers; do not add a temp-file dependency.

- [x] **Step 2: Run RED.** `cargo test --manifest-path survey/Cargo.toml burrow_archive` must fail because archive functions are absent.

- [x] **Step 3: Implement strict preflight, serial writes and loading.** `--run --construction --protocol-revision COMMIT --approval-context TEXT --out NEW_DIR` selects the engineering panel. Scientific `--run` additionally requires a separately reviewed committed executable registration; with the current unregistered candidate it fails before creating directories or simulating. Registration is outside this implementation: do not change `execution_authorized` as part of it. A typed context records an actual approval reference, not authorization by itself. The scientific gate checks `manifest.status == "registered" && manifest.execution_authorized`; the current strictly pinned candidate fails this check. Do not introduce a command-line override, hidden registration file or hypothetical approval lookup. A future reviewed registration amendment must update the committed manifest identity and this strict validation deliberately.

Require full 40-hex commits, HEAD code revision, committed protocol and manifest bytes matching the checkout, clean tracked tree and a genuinely new output directory. Use new directory creation and `OpenOptions::create_new(true)` plus `write_all`/`sync_all`. Write `index.incomplete.json` with the full expected key set before any episode. Each episode is written and hashed as `raw/{condition_ordinal:03}-{seed}.json` before any statistical analysis; no automatic analysis at run end. Keep append-only `progress/{run_ordinal:04}.json` receipts after each successful raw write so interrupted work remains identifiable. On recoverable execution/I/O failure retain prior bytes, write `failure.json` exclusively with failing key and error if possible, and report its archive path. An abrupt crash leaves incomplete index plus raw files/receipts; never resume or label complete.

```rust
let bytes = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
write_new(&out.join(&relative), &bytes)?;
let digest = format!("{:x}", sha2::Sha256::digest(&bytes));
```

Define local `write_new(path:&Path, bytes:&[u8])->Result<(),String>` in this module using the established exclusive-write convention. Publish immutable completed `index.json` only after every expected raw key is saved and passes validation. Loader first validates index identities and canonical expected key set. Require one RawRef per expected key, no reused paths, exact hashes of bytes, matching envelope provenance/options and Episode config/seed/setup. Resolve canonicalized relative paths inside archive root; reject symlink escape and `..`/absolute components. Call `validate_episode` before accepting any record. Errors prepend condition/seed. Reject incomplete scientific and construction archives instead of analysing available subsets.

- [x] **Step 4: Run GREEN.** `cargo test --manifest-path survey/Cargo.toml burrow_archive` then full survey tests and fmt. After a clean committed implementation exists, exercise only the 36 construction runs via the explicit CLI; verify scientific `--run` rejects before output creation. Archive completion does not require corridor disposal by its deadline.

- [x] **Step 5: Commit.** Stage listed files and tracker; commit `feat(survey): archive burrow episodes with exact identity and integrity checks`. Record any construction acceptance path and hashes in the final task's evidence.

### Task 4: Analyze saved records, retain censoring and close the engineering plan

**Files:**
- Create: `survey/src/claims/burrow_report.rs`, `survey/src/claims/burrow_report_tests.rs`.
- Modify: `survey/src/claims/mod.rs`, `survey/src/claims/burrow.rs`.
- Modify documentation: `docs/burrow.md`, `docs/superpowers/specs/2026-10-04-burrow-1-measured-protocol.md`, this plan.

**Interfaces:**
- Consumes validated `Archive` only; existing `stats::paired_summary(&BTreeMap<u64,f64>, &BTreeMap<u64,f64>) -> Result<PairedSummary,String>`.
- Produces `analyze(archive: &Archive) -> Result<Analysis,String>`, `render_results(analysis: &Analysis) -> String`, `analyze_saved(index_path:&Path,out:&Path)->Result<(),String>`.
- `physical_projection(record:&Episode)->String` serializes exactly `(&record.events,&record.choices)` for equality/deduplication, excluding config/frames/fingerprints.
- Serializable report records (all fields public within the module; derive Serialize/PartialEq/Debug) are fixed as follows. Existing Snapshot/WorkerWork/SpatialWork/DigDistance/LoadedTravel/Storage are imported from burrow. Scalar rates/latencies are Option when unavailable, never nonfinite.

```rust
struct Analysis {
    schema: String, code_revision: String, protocol_revision: String,
    manifest_sha256: String, panel: Panel,
    rows: Vec<SeedRow>, means: Vec<ConditionMean>, contrasts: Vec<ContrastRow>,
    controls: Vec<ControlRow>, choices: Vec<ChoiceRow>,
    duplicates: Vec<DuplicateRow>, limitations: Vec<String>,
}
struct SeedRow {
    key: RunKey, completed_ticks: u32, summary: Snapshot,
    excavation_rate: Option<f64>, disposal_rate: Option<f64>,
    worker_work: Vec<WorkerWork>, spatial_work: Vec<SpatialWork>,
    dig_distances: Vec<DigDistance>, travel: LoadedTravel, storage: Storage,
    materials: Vec<MaterialRow>, delivered_only_mean_ticks: Option<f64>,
    delivered_only_mean_opportunities: Option<f64>,
    delivered: u64, censored_carried: u64, censored_loose: u64,
}
struct MaterialRow {
    material: u64, born: u64, disposed_at: Option<u64>, fate: MaterialFate,
    tick_age: u64, opportunity_age: Option<u64>, waiting_ticks: u64,
    carriers: Vec<u32>, carried_moves: u64,
}
enum MaterialFate { Delivered, CensoredCarried, CensoredLoose }
struct ConditionMean {
    condition: String, n: usize, excavation: Option<f64>, disposal: Option<f64>,
}
struct ContrastRow {
    id: String, stratum: String, outcome: String, primary: bool,
    n: usize, mean: f64, ci95: Option<(f64,f64)>,
    positive: usize, zero: usize, negative: usize,
    differences: Vec<(String,f64)>,
}
struct ControlRow { id: String, seeds_checked: usize, identical: bool }
struct ChoiceRow {
    condition: String, left: u64, right: u64, pile_side: String,
    expected_pile_probability: f64,
}
struct DuplicateRow { projection_sha256: String, keys: Vec<RunKey> }
```

Use snake_case fate strings. Copy existing PairedSummary fields into ContrastRow rather than changing the statistics API to serialize it. Contrasts/means are absent for construction; diagnostic rows remain complete.

- [x] **Step 1: Write reductions, censoring and deterministic-analysis tests.** Use seed-7/8 growing construction episodes for physical-control checks; use synthetic complete paired maps for statistics (10001–10040 are identifiers, no episodes executed). Assert three contrasts × two outcomes, sign direction, exact seed sets, separate workforce strata and no verdict. Tamper one direct trajectory and require analysis failure. Test weight-one reduction per transport, within-round delivered tick latency zero with positive opportunity latency, censored carried/loose units retained, choice old-age versus observed waiting, and byte-identical saved reanalysis. Construction report contains all 36 rows and never applies scientific growing contrasts.

```rust
#[test]
fn burrow_report_projection_ignores_configuration_fingerprints() {
    let options = RunOptions { ticks: 16, sample_every: 4 };
    let blind = LabConfig::default();
    let mut responsive = blind.clone();
    responsive.cue = Cue::Responsive;
    let a = run_episode(blind, 7, options.clone()).unwrap();
    let b = run_episode(responsive, 7, options).unwrap();
    assert_eq!(physical_projection(&a), physical_projection(&b));
}
```

- [x] **Step 2: Run RED.** `cargo test --manifest-path survey/Cargo.toml burrow_report` must fail at the missing report API.

- [x] **Step 3: Implement deterministic estimates and saved-only output.** Add `--analyze INDEX --out NEW_DIR`; reject combinations with `--run`, seed overrides or panel overrides. Load and validate all records before writing reports. Iterate manifest order and canonical seeds, using BTreeMap/BTreeSet for stable aggregates. Derive rate numerators from integer summaries; require scientific growing denominator exactly 4096. For each outcome create paired maps and call:

```rust
let contrast = crate::stats::paired_summary(&plus, &minus)?;
```

Compute relay cue, blind transport and responsive transport differences for both outcomes; report all condition means and raw seed rows. Interaction compares the two per-seed cue differences and is explicitly redundant when direct control holds. Repeat within each workforce stratum only. Before estimating, require exact direct blind/responsive projections at every workforce and both configured response weights, and weight-one cue reduction within each transport. Deduplicate canonical projections and report multiplicities.

For all materials use birth/disposal clocks from validated histories. For growing find successful Dig/Dispose indices in complete events (one-based); compute their difference when delivered, endpoint opportunity minus birth index when censored. Retain delivered and censored entries, classify carried/loose by initializing each supplied material as loose and folding the validated successful Dig/Pickup/Drop/Dispose events into a material-ID fate map, and report delivered-only means with denominators/censor counts. Emit work/spatial/dig-distance/travel/search/logical-storage tables. Choice reports mirrored side counts and supplied probabilities 3/4 or 1/2 without a fit verdict. Construction reports legality/conservation/consistency plus censored outcomes, not transport throughput claims. Include fixed assumptions about global exit scaffold, tick freshness sensitivity, seeded geometry, duplicate trajectories and descriptive intervals.

Write canonical pretty `analysis.json` and `results.md` exclusively into a new directory, after full successful validation and analysis. Never call engine stepping from report code. Retain code/protocol/hash identities but no absolute input path, current time or generation timestamp; identical inputs reproduce identical bytes.

- [x] **Step 4: Verify engineering acceptance and update docs.** Run `cargo test --manifest-path survey/Cargo.toml`, `cargo test --workspace`, `cargo fmt --all -- --check`, `cargo fmt --manifest-path survey/Cargo.toml -- --check`, `cargo clippy --all-targets -- -D warnings` and `cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings`. Use construction-only archive CLI on a clean committed tree, analyze twice into separate new `/tmp` directories and compare bytes with `cmp`. Malformed archive analysis must leave no results. Do not run a scientific campaign to produce a demonstration report. Update docs with exact CLI commands, unregistered gate, physical-validator limits, archive schema and separate registration/execution prerequisites. Record actual checks and review rulings here; remove root tracker when complete.

- [ ] **Step 5: Commit and request whole-branch review.** Commit `feat(survey): report saved burrow contrasts and censored material histories` with report, routing and docs files. A fresh reviewer checks spec coverage, key/hash validation, all six contrasts, reductions, censoring and absence of scientific execution. Fix reviewed defects with tests and separate commits. Offer the completed engineering branch for review; do not merge/push/run the scientific matrix without the corresponding user authorization.

## Self-review and next gate

All protocol requirements map to these four tasks: transaction consistency to Task 1; complete parameters to Task 2; immutability/provenance and missing-data rejection to Task 3; contrasts/reductions/censoring/saved-only reporting to Task 4. Five Review Focus classes have named behavioral tests above. Physical reconstruction authenticates action legality and derived quantities, while provenance and engine regressions support policy fidelity; this distinction is explicit rather than hidden.

After plan approval use subagent-driven execution. After harness acceptance separately review executable registration and only then seek scientific execution approval; the current candidate remains unchanged throughout this engineering plan.


## Engineering closure

All four engineering stages are implemented and verified. The root `IMPLEMENTATION_PLAN.md` tracker was removed after completion; this closure retains evidence and root review rulings. Fresh Task4/whole-branch review remains controller-owned. No merge, push, executable registration or scientific execution is authorized by this closure. Candidate bytes and scientific parameters remain unchanged.

### Review rulings retained verbatim

Ruling: Trace clocks are nondecreasing, with exact per-round clocks and unique worker opportunities — sequential actions share ticks in the approved engine — if wrong, malformed same-tick order could need stricter validation; physical transactions still enforce order.

Ruling: Permit a narrowly scoped temporary dead_code allowance on the saved-loader entrypoint/return type until Task4 consumes it — each staged commit must compile lint-clean without adding an unplanned CLI — if wrong, an unused interface could be hidden; Task4 must remove the temporary annotations.

Ruling: Keep build attestation outside this increment and require the documented fresh Cargo execution workflow — runtime code_revision identifies the clean checkout, not an independently attested retained binary — if wrong, direct stale-binary execution could misattribute controller policy to a newer revision; document the limitation before scientific registration.

### Disposition of every declined-to-judge item

| Item | Controller disposition |
|---|---|
| Controller/RNG/exact controller BFS authentication | Approved spec explicitly excludes controller replay; physical/provenance distinction retained. No changed requirement. |
| Strictly increasing opportunity clocks | Existing nondecreasing-clock ruling applies; exact round clocks and unique workers remain checked. |
| Scientific effect sizes/animal inference/campaign runtime | Unregistered/unauthorized campaign remains unexecuted; no result claim permitted. Explicit global constraint. |
| Rewritten archive authenticity/offline Git availability | Hashes provide consistency rather than signatures; offline saved validation intentionally does not need original checkout. |
| Stale native binary build identity | Ruling below; documented fresh Cargo workflow required, no new build-attestation subsystem. |
| Atomic two-file publication/directory power-loss durability | Exclusive creation/file sync is the approved contract; write errors may leave partial results. Document new-destination retry. |
| Zero-tick native choice record validation | Measured choice validator requires a selection; zero-tick native choice exports remain valid exports but outside this validator's measured-choice acceptance. Clarify public boundary in docs. |
| Raw Episode labels as physical evidence | Retained metadata, not numerical outcomes; reports use fixed explicit assumptions. No changed requirement. |

### Final source verification

After source freeze and ten passing focused report tests, each required final broad check ran once:

```bash
cargo test --manifest-path survey/Cargo.toml > /tmp/burrow-task4-survey.log 2>&1
cargo test --workspace > /tmp/burrow-task4-workspace.log 2>&1
cargo fmt --all -- --check
cargo fmt --manifest-path survey/Cargo.toml -- --check
cargo clippy --all-targets -- -D warnings > /tmp/burrow-task4-clippy-workspace.log 2>&1
cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings > /tmp/burrow-task4-clippy-survey.log 2>&1
```

Every command exited 0. Workspace: 2,111 passed, 0 failed, 102 preexisting ignored across 30 test suites. Survey: 234 passed, 0 failed across four test binaries (19 geosim, 13 polarity, 191 survey, 11 native integration). Both formatting checks and both clippy checks were clean; `git diff --check` also passed. Runtime commit: `a4e0effd0532194621afbdd5672bc21cd0594235` (`feat(survey): report saved burrow contrasts and censored material histories`). No source fixes were added during these broad suites. Documentation acceptance is committed separately after the runtime acceptance to preserve clean-tree execution.

### Clean-tree construction and deterministic saved reanalysis

The following exact commands ran after the runtime commit, with a clean tracked tree before and after acceptance:

```bash
cargo run --manifest-path survey/Cargo.toml --bin survey -- --burrow --run --construction --protocol-revision a4e0effd0532194621afbdd5672bc21cd0594235 --approval-context 'docs/superpowers/specs/2026-10-04-burrow-1-measured-protocol.md: design approval 2026-10-04; Task4 engineering construction acceptance' --out /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-construction
cargo run --manifest-path survey/Cargo.toml --bin survey -- --burrow --analyze /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-construction/index.json --out /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-a
cargo run --manifest-path survey/Cargo.toml --bin survey -- --burrow --analyze /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-construction/index.json --out /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-b
cargo run --manifest-path survey/Cargo.toml --bin survey -- --burrow --analyze /tmp/burrow-task3-construction-61bddebf80a14fc282fd9d68fcc8e74c/index.json --out /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-a
cargo run --manifest-path survey/Cargo.toml --bin survey -- --burrow --analyze /tmp/burrow-task3-construction-61bddebf80a14fc282fd9d68fcc8e74c/index.json --out /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-b
cargo run --manifest-path survey/Cargo.toml --bin survey -- --burrow --analyze /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-malformed/index.json --out /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-malformed-results
cmp /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-a/analysis.json /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-b/analysis.json
cmp /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-a/results.md /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-b/results.md
cmp /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-a/analysis.json /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-b/analysis.json
cmp /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-a/results.md /tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-b/results.md
```

Construction and all four saved-analysis commands exited 0; all four `cmp` calls exited 0. Malformed acceptance copied the complete construction archive and changed the first referenced raw file to `tampered bytes`; analysis exited 2 with exact condition/seed and SHA-256 mismatch, leaving the requested results directory absent. The archive has 36 rows with legality/conservation/consistency validation, and both mean/contrast collections are empty. Report generation never simulates. Synthetic statistical tests use scientific seed IDs only; no scientific episodes or campaign were executed.

Acceptance hashes and paths (`/tmp/burrow-task4-acceptance.json` is the operational metadata record):

```json
{
  "revision": "a4e0effd0532194621afbdd5672bc21cd0594235",
  "archive": "/tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-construction",
  "index_sha256": "f864bc7297b5a95de8890cd0c1219f455df945cecd62f501dfb186c9fc8ca962",
  "manifest_sha256": "3ef020036fa96ebc5a440480dccfcab64fd8991dfc2ff2dc4129b05466acb890",
  "rows": 36,
  "analysis_a": "/tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-a",
  "analysis_b": "/tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-analysis-b",
  "analysis_sha256": "80820fe08a69d718837a089a0cdf65c5cef9160bb22e9635b4f4ff722247680f",
  "results_sha256": "12449f25660a99cd1922fffd5d47f5f26b177fa10f10d1c00aae16d02cbb146b",
  "prior_archive": "/tmp/burrow-task3-construction-61bddebf80a14fc282fd9d68fcc8e74c/index.json",
  "prior_analysis_a": "/tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-a",
  "prior_analysis_b": "/tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-prior-b",
  "prior_analysis_sha256": "f7627dc7ebf10aabc63cd939d56cd815dc3d685cd9aa864bed4e4687453c0677",
  "prior_results_sha256": "7c8678b1d1c1ea10057965b7b12a5f82d08c142137aecb287d0e16e0d3650b45",
  "malformed_archive": "/tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-malformed",
  "malformed_results_absent": "/tmp/burrow-task4-e02a93aa40dd4d71a462d07969ab87fe-malformed-results",
  "clean_tracked_tree": true
}
```

### Limits and next gate

Physical reconstruction authenticates action legality and derived material/work quantities. It does not replay stochastic controller policy, RNG continuation or controller search counters; fingerprints are retained state identifiers, and code provenance plus unchanged engine regressions support policy fidelity. Reports retain zero/null rates, delivered-only means with denominators/censor counts, terminal carried/loose histories, opportunity-index versus tick clocks, observed loose waiting, conditional carrier distributions, duplicates and supplied mirrored-choice probabilities. Population strata remain separate, and redundant interaction is explicitly secondary. Supplied exit navigation, tick freshness, seeded geometry and descriptive interval limitations are fixed report assumptions. One-cell construction evidence is not a repeated-production throughput benchmark.

No known functional blocker remains. Controller/root will conduct fresh reviews and record subsequent rulings here. After harness branch review, executable registration requires a separate reviewed committed amendment; scientific execution then requires separate authorization. Resource-access work remains a separate branch and does not depend on scientific outcomes.
