# Deduction and Surface Web Visualizations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user already selected subagent-driven development and independent reviews.

**Goal:** Make the completed deduction, testimony/reporting/listener, shared-surface, and active-surface studies runnable and inspectable as bounded visual episodes in the existing web Experiments interface.

**Architecture:** Add a separate `browser_experiments` core adapter module around existing study APIs, with strict input/export validation and study-specific payloads. A dedicated WASM worker returns checked bounded records; a common episode shell delegates presentation to game, testimony, and surface renderers. Recorded search results are compact, provenance-bound projections of retained first reports, separate from newly selected demonstrations.

**Tech Stack:** Existing Rust workspace, serde/serde_json, clap, wasm-bindgen/wasm-pack, TypeScript, Vite, Vitest, browser Worker, and the existing DOM helper. No new framework, simulation engine, or dependency.

**Spec:** `docs/superpowers/specs/2026-10-08-deduction-surface-web-visualizations-design.md` (approved October 8, 2026: LGTM!).

**Status:** Approved October 8, 2026 (LGTM); implementation in progress using subagents and independent reviews.

## Global Constraints

- Batch includes Wink; noisy testimony; testimony decision game; strategic reporting; strategy-aware listeners; adversarial audit; shared surface; active surface. The two testimony selections are separate catalog entries, giving eight working entries across the spec's seven study selections.
- Existing measured engine namespaces, scientific settings, policies, and first reports remain unchanged except the separately user-approved [portability amendment](../specs/2026-10-08-deduction-surface-web-portability-amendment.md): five runtime index draws in three deduction files use fixed64-bit sampling, preserving original native outputs. Other changes remain additive adapters/exports/CLI dispatch/presentation. Computed legacy floats use original1e-12 tolerance only at explicit paths; all other structural/value comparisons remain exact.
- Accept at most **64 KiB** of normalized input and **4,096 display checkpoints**; cap a complete episode export at **16 MiB** of compact JSON. Bound raw input bytes before parsing as well, preventing oversized whitespace from bypassing the limit.
- **One worker runs one requested episode at a time**, with **one retained compiled engine** and a **60-second wall-clock timeout** terminating an incomplete request. Keep **at most two successful episode records** for matched comparison. Release compiled engines and replaced records; terminating a worker releases all WASM state.
- Native adapters enforce input/checkpoint/serialized-byte limits. Browser timeout is a worker lifecycle limit, not a change to any scientific clock or planner objective.
- Use a decimal string for seeds; accept digits only within u64, including `18446744073709551615`. Fingerprints remain hexadecimal strings. Transport payload integers losslessly as strings, with bounded envelope versions/indexes/counts as ordinary numbers. Exact fraction numerators/denominators never pass through an approximate calculation to reach a controller.
- Agent view is checkpoint-local; Researcher view is explicitly labeled and purely presentational. Do not reveal future bits, observations or outcomes in earlier Agent views.
- No GA/random-search training or complete diagnostic census executes to render an episode. No ignored raw evidence enters a commit or web bundle.
- Preserve ordinary Playground/sweeps and existing share-link formats. Do not stage `.claude/`, `papers/`, `survey/out/`, ignored evidence, or the deferred raw-discovery draft.
- Use the existing `.claude/worktrees/crowd` branch/worktree, despite the general preference for global worktrees. Preserve concurrent main work, crowd, and all evidence. Never stash/discard unrelated changes, use `--no-verify`, force-push, or remove this worktree.
- American spelling; Agent in product/code/docs. Rustfmt only Rust files edited by this task, with child traversal disabled; workspace format **check** is allowed. Never run cargo fmt in survey.
- Commit working tasks incrementally after independent review; merge/push completed work and check CI/Pages for the exact commit. Stop after three unsuccessful attempts on an issue and document/reassess it.

## Review Focus

1. A forged saved episode, even with a true cached integrity flag, must fail fresh Rust reconstruction and preserve the displayed previous episode (Tasks 1, 5, 6).
2. Cancel, timeout, worker-load failure, or a late stale reply must not hang the interface or replace a newer/previously successful result (Tasks 5, 6).
3. A large u64 seed, exact fraction, or malformed/oversized shared input must retain precision or fail contextually before expensive work (Tasks 1, 5, 6).
4. A rewind, Agent switch, or Researcher toggle must not reveal future/private evidence or alter policy outputs; absence of a Bayesian model must remain absence (Tasks 1-4, 6, 7).
5. Comparing different policy inputs must preserve the same actual world/case, and unsupported termination must retain costs without invented terminal metrics (Tasks 3, 4, 6, 7).

## Context and existing patterns

- Read `web/src/experiments/view.ts`, `worker.ts`, `pool.ts`: workers own WASM, report contextual errors, terminate on end/cancel, and keep runs distinct from sweep editing.
- Read `web/src/main.ts`, `ui/dom.ts`, `ui/tabs.ts`, `share.ts`, `downloads.ts`: plain DOM components, existing navigation, bounded compressed links, exports and share formats.
- Read `crates/sugarscape-wasm/src/lib.rs` checked P3/P4 and burrow boundaries and `web/src/burrow-parity.test.ts`: strict numeric/seed checking and complete native/WASM record parity.
- Read `deduction::{Engine,Controller,ReplayArchive}`, `policy_seed`, `ExperimentThreatController`; `testimony::{Belief,EvidenceRecord}`; `testimony_game::{Session,enumerate,evaluate,Listener}`; reporting `Policy`, `FrozenListener`, `histories`, `evaluate`; strategy `Model`, `Catalog`; audit `FrozenActions`, `history_view`.
- Read `shared_surface::{Ensemble,run_episode,World}` and `active_surface::{CompiledPolicy,Replay,run_episode,EpisodeState,advance_one}`. The adapters never substitute the old fixed partner model for active replay.
- Existing UI tests use Vitest and small DOM fakes when necessary (`auctions-grid.test.ts`); use pure view projections for detailed semantics and actual browser checks for complete DOM interaction. Do not introduce jsdom solely for this feature.

## Five execution stages

Create an owned root `IMPLEMENTATION_PLAN.md` only after plan approval, with the user's required Goal/Success Criteria/Tests/Status fields. Stages: 1 checked contracts and deduction/testimony adapters (Tasks 1-3); 2 surface adapters (Task 4); 3 WASM/worker boundary (Task 5); 4 browser shell/renderers (Tasks 6-7); 5 reference assets, verification and delivery (Task 8). Keep status current; retain a completed evidence copy and remove the tracker at delivery.

## File map and common interfaces

Create `crates/sugarscape-core/src/browser_experiments/` with `mod.rs`, `input.rs`, `record.rs`, `wire.rs`, `catalog.rs`, `wink.rs`, `testimony.rs`, `reporting.rs`, `surfaces.rs`, `recorded.rs`, `fixtures/`, and `tests/`. Each study family owns its adapters and tests; shared record/input helpers stay small. Add only `pub mod browser_experiments;` to the existing core `lib.rs`.

Create `crates/sugarscape-cli/src/experiment_view.rs` and add the `experiment-view` subcommand; old commands/modules are unchanged. Split WASM functions into `crates/sugarscape-wasm/src/experiment_view.rs`, registered additively by `lib.rs`.

Create `web/src/episodes/` containing types, worker client, worker, input catalog controls, replay controller, sharing, file validation, render projections, three renderers, recorded results, shell, and focused tests. Add a small outer Experiments component alongside the existing sweep view; wire it in `web/src/main.ts` and add scoped styles to `style.css`.

Core functions supplied by Task 1 and extended by Tasks 2-4 (`recorded_results_json` is supplied by Task3):

```rust
pub fn catalog() -> Vec<StudyDescriptor>;
pub fn normalize_input(json: &str) -> Result<Input, Vec<FieldError>>;
pub fn run(input: &Input) -> Result<EpisodeRecord, Vec<FieldError>>;
pub fn validate_episode(json: &str) -> Result<EpisodeRecord, Vec<FieldError>>;
pub fn episode_json(record: &EpisodeRecord) -> Result<String, Vec<FieldError>>;
pub fn recorded_results_json() -> Result<String, Vec<FieldError>>;
```

`StudyId` serializes snake_case: `wink`, `testimony`, `testimony_game`, `strategic_reporting`, `strategy_inference`, `adversarial_audit`, `shared_surface`, `active_surface`. `Input` is a strict internally tagged enum (`study`), with these fields:

| Study | Input fields and legal values |
| --- | --- |
| Wink | `seed: SeedText`, `policy: deduction::PolicyKind`, `mode: WinkMode` (`ordinary` or `diagnostic`) |
| Testimony | `fixture: String`, constrained to fixture names from unchanged `diagnose_testimony()` |
| Testimony game | `environment: String`, existing recorded named environment; `history: u8` in 0..32; `listener: String`, existing named or retained evolved listener |
| Strategic reporting | named `environment`, `history` in 0..32, `policy: String`, `listener: String`; policy/listener resolve to named controls or explicitly retained search outputs |
| Strategy inference | named `environment`, `history` in 0..32, `catalog: CatalogId` (`uniform` or `optimization_informed`) |
| Adversarial audit | named `environment`, `history` in 0..32, `controller: String`, `witness: String`; names resolve only to recorded named controls/witnesses |
| Shared surface | `protocol: shared_surface::Protocol`, `environment: shared_surface::Environment`, `sequence: u16` in 0..256 |
| Active surface | `protocol: active_surface::Protocol`, `environment: active_surface::Environment`, `sequence: u16` in 0..256; only the original 46 declared protocol/environment settings |

Case history bits follow unchanged audit `history_view`: verified calibration truth (bit4), two calibration reports (bits3/2), two live reports (bits1/0). A selected history is a conditional evaluation case, not a sampled realized private world. Reporter signals/live truth are absent unless provided by an actual existing record; do not infer a unique private state from an ambiguous observed history. Show conditional expected payoff as such. Testimony fixtures provide ordered evidence assimilation; only existing `Belief::observe` creates sequential posteriors.

Record transport shape:

```typescript
type Json = null | boolean | number | string | Json[] | {[key: string]: Json};
type StudyId = 'wink'|'testimony'|'testimony_game'|'strategic_reporting'|
  'strategy_inference'|'adversarial_audit'|'shared_surface'|'active_surface';
type EpisodeInput = {study: StudyId} & {[key:string]: Json};
type SurfaceInput = EpisodeInput & {protocol: {[key:string]: Json}; environment: Json; sequence: number};
interface Checkpoint {
  index: number; clock: Json; kind: string;
  public: Json; local: Record<string, Json>; researcher: Json | null;
}
interface EpisodeRecord {
  kind: 'experiment_episode'; version: 1; study: StudyId;
  rules_identity: string; input: Json;
  semantics: 'trajectory'|'conditional_case'|'evidence_sequence';
  checkpoints: Checkpoint[]; payload: Json;
}
interface StudyDescriptor {
  id: StudyId; family: 'game'|'testimony'|'surface'; title: string;
  supplied: string; question: string; default_input: Json; controls: Json;
}
```

Rust uses corresponding serde types with unknown-field rejection, including `Semantics::{Trajectory,ConditionalCase,EvidenceSequence}` with snake_case serialization. The Input enum may declare later study variants up front, but unavailable variants return a contextual error and remain absent from the runnable catalog until their owning task supplies dispatch. `payload` contains the complete study-specific bounded result; checkpoint clocks preserve original study enums. `public`, `local[AgentID]`, and `researcher` are explicitly clock-filtered display projections. `rules_identity` includes original rule versions and the retained source-identity receipt for that engine; adapter format version is separate. No authentication claim is made.

`wire.rs::lossless_value<T: Serialize>(&T) -> Result<Value, Vec<FieldError>>` converts integer atoms in study payloads/projections to decimal strings before WASM serialization, retaining floating testimony probabilities as floating values. Envelope versions/indexes remain bounded numeric fields. Use this one helper in every adapter and test native/WASM round trips at u64 max.

`validate_episode` reconstructs from the saved normalized input, then compares the entire fresh record with the decoded submitted record, including identity, ordering, payload, local/researcher views and outcome. No saved `passed` flag can bypass reconstruction. Source/viewer version mismatch fails before running. Bound bytes before deserialization and check fresh output size with a bounded counting writer, not an unlimited intermediate allocation.

## Task 1: Checked envelope, retained identity, catalog foundation, and Wink

**Files:** Create common core files from the map, `wink.rs`, `tests/{mod,record,wink}.rs`, `fixtures/engine-identities.json`; modify only core `lib.rs`. Later tasks add their catalog entries when runnable; do not ship fake working entries.

**Interfaces:** Consumes existing `Engine::new/request/submit/archive/outcome/fingerprint`, `Controller::respond`, `policy_seed`, `ExperimentThreatController`, `wink_config`. Produces common functions/types above, `wink::run(&Input)->Result<EpisodeRecord,Vec<FieldError>>`, and catalog's normalization/dispatch infrastructure.

- [x] **Step 1: Preserve baseline and write contract/Wink RED tests.** Create only this plan's ignored evidence directory and ledger; bind the approved spec/plan hashes and execution base. Inspect/fetch remote and merge current committed main into crowd only when no active Git operation blocks it; preserve the unrelated deferred draft. Record prior first-report read provenance and protected engine module hashes. Write tests for strict seed syntax, byte/checkpoint/output limits, forged saved record, deterministic play and ordinary-versus-diagnostic distinction.

```rust
#[test]
fn forged_checkpoint_cannot_validate() {
    let input = normalize_input(r#"{"study":"wink","seed":"7","policy":"evidence","mode":"ordinary"}"#).unwrap();
    let mut record = run(&input).unwrap();
    record.checkpoints[0].public = serde_json::json!({"forged": true});
    assert!(validate_episode(&serde_json::to_string(&record).unwrap()).is_err());
}
#[test]
fn max_seed_stays_decimal_text() {
    let input = normalize_input(r#"{"study":"wink","seed":"18446744073709551615","policy":"passive","mode":"ordinary"}"#).unwrap();
    let record = run(&input).unwrap();
    assert_eq!(record.input["seed"], "18446744073709551615");
}
```

- [x] **Step 2: Run genuine RED.** `cargo test -p sugarscape-core browser_experiments::tests::record` and `...::wink`; retain completed exit/output before runtime code.
- [x] **Step 3: Implement bounded contracts and request-based Wink runner.** Create controllers using unchanged derivation; ordinary uses selected policy for all Agents, diagnostic assigns `ExperimentThreatController` only to the actual holder according to its own `TurnRequest.observation.objective`. Store each before-request and after-submission checkpoint. Buffered submitted actions are labeled pending until the public phase commits. Public changes use actual request/engine outcomes, never a guessed delayed-effect simulation. Use archive/replay to bind terminal fingerprints. Researcher state contains only available current requests/committed host records, not guessed hidden values.

```rust
while let Some(request) = engine.request() {
    // push_before is a local helper in wink.rs: enforce checkpoint bound,
    // record this request without future observations.
    push_before(&mut checkpoints, &request)?;
    let response = controllers[usize::from(request.actor)].respond(&request);
    engine.submit(response.clone()).map_err(action_field_error)?;
    push_after(&mut checkpoints, &request, &response, &engine)?;
}
```

Define `push_before`, `push_after` and `action_field_error` locally; only common record APIs are exported. Do not edit `deduction/engine.rs` to add viewer state.
- [x] **Step 4: GREEN and review.** Run both filters, edited-file rustfmt and core Clippy. Check seed7 ordinary Evidence fingerprint against CLI's existing `2685439217561942331`; compare diagnostic seed cases with retained per-game results. Test absence of invented model beliefs, phase-buffering clocks, replay identity and view switches via record comparisons. Fresh independent spec/quality reviewer checks actual source and preserved engines; resolve findings through producer and rereview.
- [x] **Step 5: Commit reviewed working task.** Explicitly stage common/Wink files and additive lib line; message `feat(experiments): add checked bounded Wink episode records`. Update owned tracker/ledger.

## Task 2: Noisy testimony and finite testimony decision cases

**Files:** Create `browser_experiments/testimony.rs`, `tests/testimony.rs`, `fixtures/testimony-game-recorded.json`; extend common dispatch/catalog only.

**Interfaces:** Consumes Task1 envelope/helpers; existing `diagnose_testimony`, `Belief::new/observe/snapshot`, `testimony_game::{enumerate,evaluate,Listener}`. Produces `testimony::run(&Input)->Result<EpisodeRecord,Vec<FieldError>>` for two study IDs and their real catalog descriptors.

- [x] **Step 1: Prepare bounded retained definitions and RED cases.** Locate unchanged testimony/game first reports through their ledgers; hash and project named environments and retained evolved listener genomes with exclusive extraction receipts. Do not call searches. Fixture JSON uses original values and records provenance. Test ordered evidence, duplicate-ID idempotence/conflict behavior, copy/invert ambiguity, history31, missing posterior for Passive, invalid named selection, and missing realized private truth in conditional cases.

```rust
#[test]
fn passive_case_has_no_invented_posterior() {
    let input = normalize_input(r#"{"study":"testimony_game","environment":"training","history":31,"listener":"passive"}"#).unwrap();
    let record = run(&input).unwrap();
    assert_eq!(record.semantics, Semantics::ConditionalCase);
    assert!(record.payload["decision"]["posterior_true"].is_null());
}
```

Use the exact environment ID extracted from the first report; normalize documented UI aliases once in the retained-definition resolver. `training` is the browser alias for that unchanged environment, not a new setting.
- [x] **Step 2: Run RED.** `cargo test -p sugarscape-core browser_experiments::tests::testimony`.
- [x] **Step 3: Implement evidence and conditional-case projections.** For noisy testimony, clone fixture model, create `Belief`, add one attributed record at a time, snapshot after successful observation, and terminate a contradictory evidence sequence with actual contextual error/last supported snapshot. For game, enumerate the selected named environment, select matching history row from unchanged `evaluate`, and call existing `Listener::decide` on that row's actual public observation. Use expected payoff/conditional mass terminology; no invented private signal or realized reward.

```rust
let mut belief = deduction::Belief::new(fixture.model.clone()).map_err(testimony_error)?;
for evidence in &fixture.records {
    belief.observe(evidence.clone()).map_err(testimony_error)?;
    let snapshot = belief.snapshot();
    // Evidence checkpoints are real sequential conditioning here.
    push_evidence_checkpoint(&mut checkpoints, evidence, &snapshot)?;
}
```

`testimony_error` and `push_evidence_checkpoint` are local functions. Bound fixture/evidence counts before constructing output.
- [x] **Step 4: GREEN and independent gate.** Run filter, scoped formatting, core Clippy; compare exact references within existing tolerance for floating testimony models and all32 game history projections for documented listeners. Verify reporter information absent where not available and no future verification in earlier checkpoint. Review source/provenance independently.
- [x] **Step 5: Commit.** `feat(experiments): expose testimony evidence and decision cases`; stage only owned files/catalog additions.

## Task 3: Reporting, strategy inference, audit witnesses, and recorded searches

**Files:** Create `browser_experiments/reporting.rs`, `recorded.rs`, `tests/reporting.rs`, `tests/recorded.rs`, `fixtures/{strategic-recorded,strategy-recorded,audit-recorded}.json`; extend common catalog/dispatch. Keep extraction scripts/receipts ignored in this plan's evidence directory.

**Interfaces:** Consumes Tasks1-2 and unchanged reporting `enumerate/histories/evaluate/Policy::report/FrozenListener::decide`, strategy `Model::new/calibration/decide`, audit `history_view/FrozenActions::freeze/rows`, public diagnostic record types. Produces `reporting::run(&Input)`, `recorded_results_json`, name resolvers for exact policies/listeners/environment configs, and three descriptors.

- [x] **Step 1: Extract first-report definitions and write RED.** Read owned prior ledgers to locate their immutable first files. Bounded extraction retains all20 seeds per GA/random method, best policies, training and every declared holdout, summaries, paired differences, exact config/listener assumptions, policy catalogs and six audit control identities/witnesses. Include source report SHA and transformation version in each compact asset. Fixtures must parse to existing public DTOs; never use a fresh diagnose/search as the recorded source.

```rust
#[test]
fn identical_calibration_likelihood_does_not_reveal_actual_policy() {
    let record = run(&normalize_input(r#"{"study":"strategy_inference","environment":"training","history":0,"catalog":"uniform"}"#).unwrap()).unwrap();
    let supplied = &record.payload["calibration_belief"];
    assert!(supplied.get("actual_policy").is_none());
}
```

Add separate numeric equality checks for honest-copy and calibration-copy/live-invert likelihoods using original model outputs, and unequal catalog prior odds under explicitly informed prior. Test zero-mass history, missing Passive posterior, actual-policy versus listener posterior distinction, all six control resolutions, and complete20-seed paired asset census.
- [x] **Step 2: Run RED.** `cargo test -p sugarscape-core browser_experiments::tests::reporting` and `...::recorded`.
- [x] **Step 3: Implement public-history adapters.** Build `DecisionObservation` with original history index encoding, named original rules, and selected policy/listener. Strategic evaluation uses existing `histories/evaluate` for actual-policy conditional mass and reference payoff; frozen listener assumptions stay separate. Strategy calibration uses `CalibrationView` before the live result; actual policy never enters either Model API. Audit obtains original controls/witness evaluations from compact original DTOs and certifies public rows through existing frozen actions. Unsupported/zero-mass cases are explicitly unavailable, not made into a successful realization.

```rust
let model = strategy_inference::Model::new(&rules, &catalog).map_err(inference_error)?;
let calibration = model.calibration(&strategy_inference::CalibrationView {
    rules: rules.clone(),
    calibration_reports: view.calibration_reports,
    calibration_truth: view.calibration_truth,
}).map_err(inference_error)?;
let decision = model.decide(&view).map_err(inference_error)?;
```

Match inspected `CalibrationView` fields exactly; `inference_error` is a local contextual mapper. No copied likelihood implementation. Study stages exposing data before a complete decision are labeled observation reveal, not new posterior updates.
- [x] **Step 4: GREEN and review.** Validate all32 histories per selected named control with original exact reference fixtures; compare retained aggregate/search rows without filtering seeds. Test reference truth probabilities cannot appear as listener knowledge. Run filter, scoped fmt/Clippy; independent reviewer checks definitions, unavailable cases, all controls, prior interpretation and extraction provenance.
- [x] **Step 5: Commit.** `feat(experiments): expose reporting and listener audit cases`.

## Task 4: Chronological shared and active surface adapters

**Files:** Create `browser_experiments/surfaces.rs`, `tests/surfaces.rs`, `fixtures/surface-examples.json`; extend catalog/dispatch. No edits to frozen shared_surface/active_surface namespaces.

**Interfaces:** Consumes envelope/wire helper; existing shared `Ensemble::build/run_episode` and active `CompiledPolicy::build/Replay::build/run_episode`, protocol/types, physical `World::apply/reset/finish_round/read_lineage`. Produces `surfaces::run(&Input)` and two descriptors; complete records contain the original per-episode typed result plus ordered local/privileged display checkpoints.

- [x] **Step 1: RED for real timing and failure.** Write in-family examples at both investigator roles, sequence0/255, private/inert writes, shared resets, and DataFlip supported-wrong/unsupported cases. Assert B InspectOnly DataFlip terminates at Live0R3S1 with original costs A5/B7, no terminal net; roleB TrialChoice excludes private routine from helperA view. Test before/after-slot belief ordering, free zero-probe stop, supplied certainty versus observed certainty and actual-policy continuation labels.

```rust
#[test]
fn failed_dataflip_record_has_costs_but_no_terminal_net() {
    let input = active_example(Role::B, PolicyKind::InspectOnly, Environment::DataFlip, 0);
    let record = run(&input).unwrap();
    assert_eq!(record.payload["metrics"][0]["spent"], "5");
    assert_eq!(record.payload["metrics"][1]["spent"], "7");
    assert!(record.payload["metrics"][0]["net"].is_null());
}
```

Define `active_example` as an owned test helper creating the original protocol/IDs; selected case is one of the declared46.
- [x] **Step 2: Run RED.** `cargo test -p sugarscape-core browser_experiments::tests::surfaces`.
- [x] **Step 3: Implement via frozen evaluators and full-prefix inference.** Compile exactly one appropriate engine/replay per run, query each actual own prefix, and preserve complete partial failure histories. Construct display clock order from original prefixes/events/decision checkpoints, not local array position alone. Preserve simultaneous public information and private actor choices separately. For active per-slot beliefs, query `Replay::infer` on its complete prefix, not just model marginals.

```rust
let policy = active_surface::CompiledPolicy::build(&protocol, own_prior)?;
let replay = active_surface::Replay::build(&protocol, &policy)?;
let episode = active_surface::run_episode(&protocol, environment, sequence, &policy, &replay)?;
```

Resolve Known priors only from caller's valid declared original control; reject undeclared Known/DataFlip. Project actual field visibility from existing privileged records/World replay; do not reconstruct physics in TypeScript. If obtaining a field symbol requires an observation probe, use a cloned evaluator World with existing `apply(Read)` and throw away its cost/outcome from controller histories. These display-only probes consume no RNG and cannot alter original episode results. Never show original write-lineage symbol as the inverted stored symbol without checking actual World behavior.
- [x] **Step 4: GREEN/reference/independent gate.** Compare complete native episode payloads against retained first-report/proof projections for all46 active settings on sequences0/255 and bounded shared settings including restart/stale controls; use streaming extraction for large originals. No new full measurement series. Verify perspectives and ID renaming cannot change behavior. Run filter/fmt/Clippy and independent timing/privacy review. Record one-engine native time/memory without claiming browser performance.
- [x] **Step 5: Commit.** `feat(experiments): expose chronological surface episode records`.

## Task 5: Native command, checked WASM exports, and cancellable worker

**Files:** Create CLI/WASM `experiment_view.rs`, CLI `tests/experiment_view.rs`, WASM `experiment_view_tests.rs`; add dispatch/module declarations. Create `web/src/episodes/{types,worker,client}.ts`, `client.test.ts`, `parity.test.ts`.

**Interfaces:** Consumes complete core run/validation/catalog/recorded APIs. Produces WASM `experiment_catalog_json()`, `experiment_run_json(input_json:&str)`, `experiment_validate_json(record_json:&str)`, `experiment_recorded_json()` returning `Result<String,JsValue>`; native `experiment-view catalog|run|validate|recorded` matching those records. `run` takes `--input FILE`; `validate` takes `--input FILE` (episode). Normal operational/usage exit mapping1/2 remains unchanged.

Client contract:

```typescript
type WorkerRequest = {id: number; op: 'run'|'validate'; text: string};
type WorkerReply = {id: number; record: EpisodeRecord} |
  {id: number; errors: {field:string;message:string}[]};
interface EpisodeClient {
  request(op: 'run'|'validate', text: string): Promise<EpisodeRecord>;
  cancel(): void;
  dispose(): void;
}
```

- [x] **Step 1: Write RED boundary/parity/client tests.** Native/WASM malformed input, max seed, u64 overflow, forged record with true arbitrary flag, zero denominator, unknown version/field, raw>64KiB and record>16MiB. All8 study default records must compare completely between CLI and actual WASM. Use worker fakes with stale reply/load failure/cancel/timeout scenarios and Vitest fake timers.

```typescript
it('late replies cannot replace a canceled request', async () => {
  const {client, worker} = clientFixture(); // owned fake WorkerLike factory
  const pending = client.request('run', defaultInputText('wink'));
  client.cancel();
  await expect(pending).rejects.toThrow(/canceled/);
  worker.deliver({id: 1, record: winkFixture});
  expect(worker.terminated).toBe(true);
});
```

`clientFixture`, `defaultInputText` and `winkFixture` are local test helpers/data built from core catalog/fixtures, not production simulator doubles.
- [x] **Step 2: RED commands.** `cargo test -p sugarscape-cli --test experiment_view`; `npx vitest run src/episodes/client.test.ts` in web. Boundary tests compile before running actual WASM.
- [x] **Step 3: Implement checked bridges and worker lifecycle.** Map errors with existing `FieldError` JSON shape; validate raw bytes before calls. Native JSON writer adds newline and explicitly flushes, exits1 on write/flush failure. Worker imports WASM and awaits initialization inside try; monotonically assigned IDs and a60s page-side timer reject/terminate on all end states. No successful partial record; no synchronous WASM on page. Serial requests permit at most one engine; termination clears all compiled state.

```typescript
const ready = init();
addEventListener('message', async ({data}: MessageEvent<WorkerRequest>) => {
  try {
    await ready;
    const text = data.op === 'run' ? experiment_run_json(data.text) : experiment_validate_json(data.text);
    postMessage({id: data.id, record: JSON.parse(text)} satisfies WorkerReply);
  } catch (error) {
    postMessage({id: data.id, errors: parseErrors(error, 'episode')} satisfies WorkerReply);
  }
});
```

- [x] **Step 4: GREEN and feasibility gate.** Build release CLI, web WASM, then native/WASM parity, client tests, TypeScript and Node WASM tests. Measure cold WASM generation separately from playback for all8 defaults and slow representative active controls on actual browser worker. Verify no>60s default,>4096 checkpoints or>16MiB export. If a bound fails, stop/reassess and request a concrete spec limit revision; never truncate or silently tune experiment settings. Fresh boundary/worker reviewer checks complete records and memory lifecycle.
- [x] **Step 5: Commit.** `feat(wasm): expose bounded experiment episodes and cancellable workers`.

## Task 6: Episode shell, selection, playback, game/testimony renderers, and recorded results

**Files:** Create `web/src/episodes/{catalog,controls,replay,projection,share,file,recorded,view,game-view,testimony-view}.ts`, corresponding focused tests; create `web/src/experiments/experiments-shell.ts`; minimally modify main.ts/style.css. No existing sweep engine refactor.

**Interfaces:** Consumes Task5 client and typed envelope/catalog. Produces `EpisodeView.el`, `EpisodeView.openInput(input:Json):void`, `EpisodeView.dispose():void`; `ReplayController` with record/index/playing and `load/step/seek/reset/pause`; `projectCheckpoint(record,index,perspective)`; renderer `renderGame`/`renderTestimony`. `Perspective` is `{kind:'agent';agent:string}|{kind:'researcher'}`. Surface renderer is connected in Task7; until then its catalog selection must say unavailable in this unshipped intermediate branch, never fake a trace.

- [x] **Step 1: RED pure lifecycle/privacy/share tests.** Check editing inputs keeps shown inputs, failed replacement/import preserves prior record, reset/seek does not run worker, no future outcome at earlier checkpoint, different Agent gets own local projection, reduced motion disables autoplay, invalid/incompatible/compression-bomb input fails before worker. Define new `#e=` input format with version1 and a64KiB decompressed cap; existing `#c=`, `#x=` and ordinary Playground links keep routing unchanged. Export imported records is always worker-validated.

```typescript
it('agent perspective excludes researcher truth at every index', () => {
  for (let i = 0; i < record.checkpoints.length; i++) {
    const shown = projectCheckpoint(record, i, {kind:'agent',agent:'0'});
    expect(shown.researcher).toBeNull();
    expect(shown.local).toEqual({'0': record.checkpoints[i].local['0']});
  }
});
```

- [x] **Step 2: Run RED.** `npx vitest run src/episodes/replay.test.ts src/episodes/projection.test.ts src/episodes/share.test.ts src/episodes/file.test.ts`.
- [x] **Step 3: Implement existing-style shell and real renderers.** Outer Experiments shell owns Sweep/Episode switch; leaving episode view pauses playback without discarding shown successful records. Game diagram shows phase/status, watches, submitted versus committed actions and actual observations. Testimony diagram labels observed reports, supplied assumptions, existing available beliefs, action, and conditional expected payoff; no invented reporter truth or intermediate Bayesian updates. Display recorded aggregate/search results in a separate panel using the full Task3 compact asset and its source metadata. Use textContent/DOM h for untrusted labels, never innerHTML.

```typescript
export function projectCheckpoint(record: EpisodeRecord, index: number, perspective: Perspective): Checkpoint {
  const at = record.checkpoints[index];
  if (!at) throw new Error('checkpoint out of range');
  return perspective.kind === 'researcher' ? at : {
    ...at, researcher: null, local: {[perspective.agent]: at.local[perspective.agent] ?? null},
  };
}
```

The renderer receives only this projected checkpoint plus nonprivileged descriptor metadata, never full future payload. Terminal payload is available to its labeled result pane only at final checkpoint/Researcher view, with checkpoint-local behavior retained.
- [x] **Step 4: GREEN and visual gate.** Run focused tests/TypeScript; reviewer inspects navigation, precision, stale results, provenance and code. Perform actual browser keyboard stepping, Agent/researcher toggle, narrow layout and game/testimony screenshots before closing task. Main beforeunload/dispose and legacy links remain functional.
- [x] **Step 5: Commit.** `feat(web): visualize deduction games and testimony decisions`.

## Task 7: Surface visualization and matched comparison

**Files:** Create `web/src/episodes/{surface-view,surface-projection,comparison}.ts` and corresponding tests; add renderer dispatch in view.ts and scoped styles.

**Interfaces:** Consumes full surface records and checkpoint projections; produces `renderSurface(root:HTMLElement,at:Checkpoint,descriptor:StudyDescriptor):void`, `comparisonKey(input:EpisodeInput):string`, `withActivePolicy(input:SurfaceInput,policy:string):SurfaceInput`, and shell matched-record controls. Canonical comparison key preserves study, mechanism, role, complete bit sequence and scientific settings while excluding only policy/control selector. Same-history reporting comparisons can use a separate family key; never compare trajectories by timeline array index as if their clocks matched.

- [x] **Step 1: RED tests.** Accepted write is not a received message; private fields stay private, round resets remove the visible field at exact clock, stop has no invented paid waits, B's unobserved A action stays hidden, and inverted stored symbol differs from write origin. Test supported wrong DataFlip completion versus unsupportedB InspectOnly with A5/B7 costs and unavailable terminal fields. Check two compared policies have identical actual mechanism/role/sequence and display differing timeline lengths honestly.

```typescript
it('failed completion remains unavailable rather than zero', () => {
  const shown = surfaceSummary(failedDataFlipRecord); // owned pure display helper
  expect(shown.net).toEqual([null, null]);
  expect(shown.spent).toEqual(['5','7']);
  expect(shown.status).toBe('unsupported history');
});
```

- [x] **Step 2: Run RED.** `npx vitest run src/episodes/surface-projection.test.ts src/episodes/comparison.test.ts`.
- [x] **Step 3: Implement interaction visual and event strip.** Use actual checked clock/field projections. Render two labeled Agents, surface ownership, observed symbol/read/write/wait/inspect/prediction, credit expenditure and existing alternate values. Use the selected Agent projection only; Researcher toggle adds explicit truth/lineage. Matched comparison holds at most2 successful records, sequential worker requests, independent timelines joined by public clocks where available, and a clear unavailable marker when one run terminates.

```typescript
const matched = withActivePolicy(currentInput, selectedComparisonPolicy);
if (comparisonKey(matched) !== comparisonKey(currentInput)) throw new Error('comparison inputs differ');
const next = await client.request('run', JSON.stringify(matched));
// Install only after the same generation is still current and validation succeeded.
```

`withActivePolicy` clones `input.protocol` and changes only its nested policy field; validate its existing enum spelling against catalog controls. `comparisonKey` must test nested policy exclusion and retain all other settings. Shared-surface comparisons may change only the declared pair/control choice and preserve the same calibration count, actual mechanism and sequence; refuse controls with incompatible scientific setup instead of silently adjusting the world.
- [x] **Step 4: GREEN/browser/independent gate.** Run focused tests, TypeScript, actual WASM parity. Review all8 catalog entries now working. In browser inspect primary shared/private/reset/inert, A one-probe and B zero-probe active behavior, supported wrong inversion and unsupported termination. Check keyboard/reduced-motion/narrow layout and controls, with retained screenshots. Reviewer verifies no display physics duplication and no privilege leak through alternative/result panes.
- [x] **Step 5: Commit.** `feat(web): visualize surface learning and paid experiment choices`.

## Task 8: Complete provenance, preservation, docs, whole-branch QA, and exact delivery

**Files:** Create `docs/experiment-viewer.md`; append brief viewer links to deduction/shared/active guides, preserving original prefixes. Update this plan/status and owned tracker. Retain all source/provenance/extraction/visual/verification/review receipts in this plan's ignored evidence directory; do not stage those receipts.

**Interfaces:** Consumes all8 complete descriptors, core records/validators, native/WASM bridges, actual UI, and original first-report identities. Produces verified published viewer and user guide. No new experimental claims or report collection required.

- [x] **Step 1: Final coverage/provenance checks.** Assert exact8-entry catalog/default census, every supported selection runnable, all20 GA and20 random seeds with all declared holdouts and pairs in recorded asset, original source hashes/configs unchanged, and no raw evidence in bundle. Stream-hash original large reports; retained first/repeat remain in place. Use bounded per-case replay/projection comparisons; do not parse multi-GB reports into memory. Check old CLI small outputs exactly and prior existing success tests. Any numerical difference is a finding requiring explanation/review, not a silently updated recorded asset.
- [x] **Step 2: Write guide from actual interface.** Describe navigation, controls, Agent/researcher boundary, conditional cases versus trajectories, recorded results versus selected examples, supported mistakes/unsupported failures, bounds/cancel/errors, export/share, and browser computation cost. Add only brief links to existing product guides; do not put debugging history in them.
- [x] **Step 3: Complete nine gates with receipts.** Run commands below on final source, recording cwd/exits/log hashes/counts. Rustfmt check must not edit unrelated files; no survey formatting/execution is added locally by this viewer change.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo build --release -p sugarscape-cli
# web/
npm run wasm
npx tsc --noEmit
npx vitest run
# repository root
python3 -m unittest discover -s studio/tests -t studio
```

Also run `npm run build` in web to verify the actual distributable, including bounded asset bundle. Release CLI precedes parity tests. Do not conflate numerical oracle tests with ordinary workspace counts or hide existing ignored tests.
- [x] **Step 4: Actual browser acceptance and independent final review.** Read/use the browser-control skill at this stage, run the real Vite app, and inspect every rendering family and all8 defaults; record exact browser/version/source/commands and screenshots. Test navigation away/back, old sweep and Playground, malformed share/import, canceled/timeout run, comparison, u64max seed, unsupported failure, narrow viewport, keyboard and reduced motion. A fresh whole-branch reviewer checks source, contracts, all task findings, results/asset provenance, protected engines and all receipts. Fix only through implementers and rereview; preserve original evidence and disclose every revision.
- [x] **Step 5: Commit and integrate.** Commit measured user guide/links/final status. Inspect main status, operation markers/locks and fetched remote history; preserve any unrelated work. Merge into main, compare actual merged source with verified tree, run checks warranted by concrete differences, and use an implementer/reviewer for actual conflicts. Push normally; resolve exact remote SHA with `git ls-remote`; wait for all CI jobs and applicable Pages build/deploy for that SHA. Retain conclusions/URLs. Complete the5-stage tracker, preserve snapshot, remove only owned tracker, retain crowd/evidence. Do not mark delivery complete while remote gates are pending.

## Controller handoff and self-review

- Read this plan's ledger before execution; resume existing task status rather than restart. No evidence cleanup/deletion.
- Use fresh task implementers and fresh independent reviewers where available; with limited slots, sequence reviews and do not share role between producer/reviewer. Root handles planning, evidence bookkeeping and Git, not runtime fixes.
- Common/consumer contract changes require an explicit ledger `Ruling:` with correction cost, communicated to affected tasks before implementation. Enumerate every ruling in final handoff; no silent scope changes.
- Public history cases deliberately omit unrealized private truth. Native APIs expose some fields only at later checkpoints; a renderer must display unavailable rather than invent a hidden world. Original complete engines stay protected.
- Inline self-review must confirm every spec section maps to a task; no placeholder instructions; consistent type/function names; all5 Review Focus items have owning tests. Final execution approval remains the user's review of this written plan.

## Verified execution

BatchA runtime delivered October8,2026 at merge `70d5fb672d73099ca2ea0204066c03967e2f88cc`. All seven [CI jobs](https://github.com/ndouglas/SugarScape/actions/runs/37836712594) and both [Pages jobs](https://github.com/ndouglas/SugarScape/actions/runs/37838193200) passed for that exact commit. The final documentation-status commit receives its own exact-commit checks; retained delivery receipts record that outcome. Crowd, the deferred discovery draft, original reports/settings, and all review/verification evidence remain preserved.
