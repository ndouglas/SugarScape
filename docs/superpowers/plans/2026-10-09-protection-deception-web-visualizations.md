# Protection and Deception Web Visualizations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Fresh implementers and independent reviewers are the user's preserved execution method. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add P3 re-caching and P4 supplied-gesture viewers to Experiments → Episodes without changing native behavior or scientific records.

**Architecture:** Strict browser DTOs delegate validation to existing rig constructors; adapters execute the original ledger-enabled runner once and project its frames. The existing spatial shell renders explicitly permitted Agent evidence and separately gated researcher data. Common worker, replay, export, sharing, and reconstruction APIs remain authoritative.

**Tech Stack:** Existing Rust/serde workspace, wasm-bindgen/wasm-pack, TypeScript, Vite, Vitest, plain DOM and the inherited spatial renderer; no new dependency or framework.

**Spec:** [Approved Batch C design](../specs/2026-10-09-protection-deception-web-visualizations-design.md), written-spec approval by user LGTM on 2026-10-09. Plan approved by user LGTM! on 2026-10-09; execution is subagent-driven.

## Global Constraints

- Add `protection_recaching` and `deception_gestures`; preserve the existing thirteen entries, links, original lab exports, native engines/controllers/schedules/random streams, measured reports, and frozen settings.
- Fixed 9×9/two-initial-Agent rigs, native 64 requested ticks, initial plus every completed-step frame; no horizon/sampling editor or added scientific campaign execution.
- Retain 64 KiB input/decompressed-link, 16 MiB record, 4,096 checkpoint, one-worker-request, 60-second timeout/cancellation, and two-successful-record limits. Fixed episodes have at most 65 native frames. Reject unsupported retention rather than altering inputs.
- Seeds, IDs, and unbounded u64 clock/config fields cross the new browser boundary as canonical decimal strings. Bounded site indexes/coordinates remain validated numbers; floats remain unchanged.
- Research DTOs require field allowlists, not just actor filtering. Agent views contain neither peer state nor actual-watcher/hidden-transfer/hidden-stock/future/terminal diagnostics. Clear-view transfer signals remain legitimate observations. Reference layout is labeled setup scaffolding; memory is labeled and aged.
- Same-source imports require exact fresh reconstruction; add no floating tolerance. Preserve original source identities and receipts; separately identify new adapters and maintain existing workspace source-integrity guards.
- Existing `.claude/worktrees/crowd`, branch crowd; inspect status before writes and preserve all unrelated work/evidence. Never stage `.claude/`, `papers/`, `survey/out/`, ignored evidence, or the untracked surface-discovery draft.
- Fresh producer and independent spec/quality review per task, then fresh whole-increment review. One producer/build owner at a time. Root handles orchestration, metadata, evidence, and Git; product implementation belongs to producers.
- Commit reviewed working increments; safely merge completed work into main, push, and verify CI/Pages for the exact delivered commit. Preserve concurrent work; never force push or bypass hooks.
- American spelling; Agent in code/docs. Rustfmt only edited Rust files with child traversal disabled; no survey formatting or added local survey campaign execution. Stop after three failed attempts on an issue and document/reassess.

## Review Focus

1. Actor-owned diagnostics containing privileged facts must still be withheld; pin P3 actual watchers and P4 actual stock/transfer/value exclusion — Tasks 1–3.
2. Native action/observation tick versus post-step boundary must not reveal evidence early, and dead Agents must not acquire invented state — Tasks 1–2.
3. Invalid complete lab JSON, including edits before blur, must block Run/Share while retaining the visible successful record — Task 3.
4. Maximum seed/exposure-span strings, missing/extra fixture fields, and malformed imports must reject or retain exact values without JavaScript rounding/default replacement — Tasks 1–4.
5. Mirrored layouts, perspective changes, repeated seeks, and comparison/navigation/disposal must preserve evidence layers, clock matching, and lifecycle — Tasks 3–5.

## File Structure and Interfaces

Create `crates/sugarscape-core/src/browser_experiments/caching/{mod,input,scenes,projection,protection,deception}.rs`. Keep strict codecs, frozen example provenance, allowlisted projection helpers, and each native adapter separate. Add `fixtures/caching-scenes.json` and `fixtures/caching-identities.json`. Modify common `mod.rs`, `input.rs`, `catalog.rs`, `tests/mod.rs`; add `tests/caching_{input,protection,deception}.rs`. Do not edit native protection/deception engines to improve display coverage.

New normalized input shapes:

```json
{"study":"protection_recaching","seed":"7","lab":{"policy":"selective","fixture":{"kind":"single","initial_observed":true,"redeposit_observed":false},"mirrored":false,"reburial_cost":0.25,"discovery":0.0,"exposure_span":"64","observer_span":64}}
{"study":"deception_gestures","seed":"7","lab":{"sender":"sham","view":"ambiguous","display_seen":true,"layout":"off_route","effort_cost":3.0,"mirrored":false}}
```

`Input::ProtectionRecaching { lab: ProtectionInput, seed: SeedText }` and `Input::DeceptionGestures { lab: DeceptionInput, seed: SeedText }` use the existing canonical seed decoder. Both DTOs require every declared field and deny unknown fields; strict fixture DTOs also deny unknown/missing variant fields. Native policy/view/layout enums retain their spellings. `ProtectionInput::to_core()` and `DeceptionInput::to_core()` return original LabConfig through existing rig construction/Config validation; exposure_span uses existing `spatial::IdText` conversion. Native P4 defaulting must not silently fill missing browser input.

`caching::validate_input(&Input) -> Result<(), Vec<FieldError>>`, `caching::run(&Input) -> Result<EpisodeRecord, Vec<FieldError>>`, `caching::scenes::input_for(&str) -> Result<Input, Vec<FieldError>>`, and `caching::rules_identity(StudyId) -> Result<String, Vec<FieldError>>` integrate with common dispatch. Register descriptors only when runnable. Reuse family `spatial`; descriptor controls identify `lab_kind: "protection" | "deception"` and the declared comparison axes.

Checkpoints retain `index`, `clock`, `kind`, `public`, `local`, `researcher`. Public data has `arena:{width:9,height:9}`, `layout_kind:"reference_layout"`, lab kind, and justified fixed boundary cells only. Clock has decimal-string `native_tick`; kind is `caching_initial`/`caching_step`/`caching_terminal`, with a separate terminal flag for initial/terminal coincidence if required. Do not duplicate or discard native frames. Record action interval semantics explicitly after verifying native runner timing.

Each selected local value contains `agent:{id,pos,holdings,caches}`, `seen`, `own_actions`, `availability`; P3 adds `protection`, P4 adds `sender` and `received_observations`. All arbitrary integers use lossless transport. Researcher has original `frame`, with additional derived physical display layers clearly labeled. Payload is `{native:<lossless original EpisodeRecord>,capture:{clock_semantics:"native_completed_step_boundary",requested_ticks:"64"}}`. Renderer-facing local layers use site-to-coordinate conversion with the native 9×9 indexing, not guessed coordinates.

P3 `own_actions` permits phase/action/harvest/metabolic demand-consumed/gross dug-buried/burial cost/source/target/perceived exposure for the selected actor. Withhold actual_watchers and research raid/discovery diagnostics initially. P4 permits own phase/action/target/harvest/dug/buried/effort/metabolic demand-consumed/position/walk outcome, withholding cancellation/occupant/recovery diagnostics unless native provenance explicitly demonstrates own availability. Received observations contain only the native `ObservedRecord.public` addressed to the selected receiver. Keep P4 ChoiceRecord entirely researcher-only initially; own remembered targets/amounts are available through seen evidence and committed actions. Record this conservative allowlist with exact source provenance in the Task 1 contract receipt; any extension requires review and a native-boundary test.

Web additions: `web/src/episodes/caching-{controls,projection,view}.ts`, `caching-projection.test.ts`, `caching.browser-test.ts`, `caching-parity.test.ts`; additive integration in `types.ts`, `controls.ts`, `view.ts`, `spatial-comparison.ts` and existing projection dispatch as needed. Reuse common lifecycle/share/file modules without new protocols. `cachingDisplay(at:Checkpoint):Json` consumes only an already perspective-projected checkpoint. `renderCaching(at:Checkpoint, descriptor:RenderDescriptor):HTMLElement` follows the inherited spatial renderer lifecycle. `CachingControls` exposes `readonly el:HTMLElement`, constructor `(descriptor:StudyDescriptor, input:Json=descriptor.default_input, changed:(input:Json)=>void=()=>{})`, and `input():Json`; it parses current raw JSON at Run/Share boundaries. `RenderDescriptor` is the existing safe descriptor type from `catalog.ts`.

## Tracker and Source-bound Evidence

After plan approval, create owned `IMPLEMENTATION_PLAN.md` with five stages matching Tasks 1–5 and the required Goal/Success Criteria/Tests/Status fields. Before each source change record current Git state. Retain task reports, source manifests, RED/GREEN logs, reviewer findings, corrections, browser evidence, and final verification under `.superpowers/sdd/2026-10-09-protection-deception-web-visualizations/`; preserve all earlier evidence. At delivery snapshot the completed tracker into that directory and remove only the owned tracker.

Before native adapter edits bind original `minds/protection`, `minds/deception`, shared world/rules/memory/RNG/config, Cargo.lock, original WASM boundaries, existing browser adapters, and product guide prefixes by SHA256. Save original P3/P4 raw export fixtures at seeds 7 and u64-max for selected treatments. Retain source identity path inventories/digest recipe and first/repeat artifacts; do not overwrite originals after reviewing output. New example identities include relevant engine dependencies and adapter schema. Old identities change only if their protected source changes, with explicit disclosure and review.

## Task 1 / Stage 1: Strict Inputs and P3 Native Adapter

**Goal:** Runnable P3 catalog entry with exact original payload and safe projections.
**Files:** Common Rust dispatch/input/catalog, new caching modules except deception implementation, scene/identity fixtures, `tests/caching_input.rs`, `tests/caching_protection.rs`.
**Interfaces:** Produce codecs and common functions above, P3 adapter, and projection shape consumed by Tasks 2–4.

- [x] **Step 1: Bind baseline and field contract.** Save manifest/original exports and a table mapping each local/public field to exact native source. Verify native site indexing, boundary/action clocks, memory caps, and whether dead role absence removes local state. Establish a conservative preflight for fixed native record sizes before runner allocation; charge payload and each projected checkpoint with the existing serialized-size/budget helpers before retaining them.
- [x] **Step 2: Write RED codec and payload tests.** Add strict nested-field/fixture tests, missing P4 fields, malformed decimal/overflow seeds/spans, domain violations, and original P3 payload/fingerprint comparisons. Test helpers parse the exact JSON examples in this plan through `normalize_input`; do not invent normalization defaults.

```rust
let input = browser::normalize_input(P3_JSON).unwrap();
let Input::ProtectionRecaching { lab, seed } = &input else { panic!("wrong study") };
let native = crate::minds::protection::runner::run_episode(lab.to_core().unwrap(), seed.value(), true);
let record = browser::run(&input).unwrap();
assert_eq!(record.payload["native"], browser::wire::lossless_value(&native).unwrap());
assert_eq!(record.checkpoints.len(), native.frames.len());
for (checkpoint, frame) in record.checkpoints.iter().zip(&native.frames) {
    assert_eq!(checkpoint.clock["native_tick"], frame.tick.to_string());
    assert_eq!(checkpoint.researcher.as_ref().unwrap()["frame"]["fingerprint"], frame.fingerprint);
    for local in checkpoint.local.values() {
        assert!(!serde_json::to_string(local).unwrap().contains("actual_watchers"));
    }
}
```

`P3_JSON` is a literal test constant containing the first input above; the test imports `crate::browser_experiments as browser` and `browser::Input`. Run `cargo test -p sugarscape-core browser_experiments::tests::caching`; retain behavioral RED after the skeleton compiles.
- [x] **Step 3: Implement minimal P3 wrapper/projections.** Call original runner once, reject fixture/ledger errors contextually, project fixed boundary geometry and own fields only, retain full researcher frame, set common identity/semantics and check final record bounds. Native core capture/controller code remains untouched. Add off/selective/indiscriminate/erased, mixed/stumble/cue mismatch, mirrors, seed-max/span-max and death/early-terminal coverage with actual clock assertions.
- [x] **Step 4: Freeze GREEN examples and review.** P3 default uses the exact first JSON above. Named variants change only policy, or a native fixture with required fields; record each delta before output evaluation. Define policy comparison axis preserving fixture/seed/all other settings. Run focused tests, edited-file rustfmt (`rustfmt --edition 2021 --config skip_children=true <edited files>`), and core Clippy. Reviewer checks native equality, allowlist and preflight. Resolve findings through producer and scoped rereview.
- [x] **Step 5: Commit scoped reviewed files.** `feat(experiments): visualize protection episode evidence`.

## Task 2 / Stage 2: P4 Native Adapter and Gesture Evidence

**Goal:** Runnable P4 entry with exact native payload and receiver-visible signals.
**Files:** `caching/deception.rs`, shared caching scenes/projection/dispatch/identities, `tests/caching_deception.rs`, common catalog as needed.
**Interfaces:** Consume Task 1 DTOs/checkpoint shape; produce both completed catalog entries and receiver-only observation projection.

- [x] **Step 1: Write RED signal separation tests.** Run sham/ambiguous/seen and sham/clear/seen examples. Compare each local receiver's observations to that boundary's native public observations and forbid added physical diagnostics:

```rust
let record = browser::run(&input).unwrap();
for checkpoint in &record.checkpoints {
    for local in checkpoint.local.values() {
        let serialized = serde_json::to_string(local).unwrap();
        for forbidden in ["actual_transfer", "actual_stock", "actual_value", "target_occupant"] {
            assert!(!serialized.contains(forbidden), "leaked {forbidden}");
        }
    }
}
```

`input` is parsed from the second literal JSON above. Add direct equality tests for received_observations against actor-filtered native `public` fields, and sentinel tests using contrasting researcher values to detect leakage without relying solely on key scans. Run `cargo test -p sugarscape-core browser_experiments::tests::caching_deception` and retain behavioral RED.
- [x] **Step 2: Implement original single-run wrapper.** Convert `EpisodeFailure` into contextual common FieldErrors; preserve partial data only as failure context, never verified success. Keep all Choices and physical stock/transfer diagnostics under Researcher. Show legitimate clear zero/positive transfer observations even when earlier seen memory remains unchanged. Verify native begin-tick reset and post-step association before appending signals/actions.
- [x] **Step 3: GREEN parity and provenance matrix.** Compare full original native payload and all frame fingerprints for ordinary/matched-neutral/sham × ambiguous/clear × seen/unseen × on/off-route × costs 0/3 × mirror choices using bounded representative fixtures; ensure every axis and crucial interaction is covered. Include seeds 7/u64-max, stale seen evidence, clear zero, absent receiver, and repeated-run exact equality. Default is the second literal JSON; neutral, clear, unseen and zero-cost presets change one field each. Comparison axes sender/view/display_seen/effort_cost preserve geometry and other settings. Do not compare layouts/mirror as matched treatment axes.
- [x] **Step 4: Review and commit.** Focused core tests, edited-file rustfmt/core Clippy; independent native/visibility review and corrections. Commit `feat(experiments): visualize supplied gestures and receiver evidence`.

## Task 3 / Stage 3: Browser Controls, Reference Maps, and Replay

**Goal:** Both entries work in the shared Episodes UI with precise perspective boundaries.
**Files:** New caching controls/projection/view and tests, common TS types/controls/view dispatch, spatial comparison declarations; scoped style additions only if needed.
**Interfaces:** Consume Rust descriptors/local/researcher shapes. Export `cachingDisplay(at:Checkpoint):Json`, `renderCaching(at:Checkpoint, descriptor:RenderDescriptor):HTMLElement`, and `CachingControls` with the signatures above. Never give the renderer original input/payload/future checkpoints.

- [x] **Step 1: Write RED projection/browser tests.** Use synthetic checkpoints with private sentinels and real common perspective selection to verify selected own position, caches, exposure/sender state and received cue. Hide peer markers/stocks/watchers; label reference layout and memory ages. Test legitimate clear transfer, stale remembered amount versus researcher stock, initial state and missing/dead role. Pin replay determinism by repeated seeks without requests.

```ts
const display = cachingDisplay(projectedAgentCheckpoint);
expect(JSON.stringify(display)).not.toContain('private-peer-sentinel');
expect(JSON.stringify(display)).not.toContain('hidden-stock-sentinel');
expect(JSON.stringify(display)).toContain('reference_layout');
```

`projectedAgentCheckpoint` is a test-built actual Checkpoint with one permitted local entry and researcher=null, produced through existing perspective projection; place sentinels in the unprojected record's peer/researcher/future data. Add DOM tests changing lab textarea before blur, clicking Run/Share, and confirming no request/clipboard replacement while the last successful record remains.
- [x] **Step 2: Implement controls and layers.** Present native field names/domains, fixed 64-tick explanation, named examples, exact strings for seed/span. Validate current raw lab JSON at action boundaries; preserve invalid/missing custom fields. Convert bounded site indexes to cells, show labeled own cache versus remembered evidence layers and inspector timestamps, with an explicit reference-layout legend. Researcher shows all native frame diagnostics without mixing them into local layers.
- [x] **Step 3: Integrate lifecycle and comparison.** Extend StudyId unions/dispatch and allowed comparison axes. Keep input changes separate from the shown record; retain/pause across navigation, cancel current request, preserve successful replacement on error, and clear references on disposal. Test mirror coordinate mapping, repeated focus-preserving seeks, keyboard/narrow/reduced motion, incompatible sources/geometry, missing terminal matches and exact input differences.
- [x] **Step 4: GREEN, independent UI review, commit.** Run `cd web` then `npx vitest run src/episodes/caching-projection.test.ts src/episodes/caching.browser-test.ts` and `npm run typecheck`. Tests use existing DOM test environment/utilities. Review all fields reaching rendered layers, current-input error behavior, and old spatial-entry regression. Commit `feat(web): add protection and gesture episode viewers`.

## Task 4 / Stage 4: Cross-boundary Parity and Preservation

**Goal:** New envelope/import paths reproduce native episodes and preserve original exports/source.
**Files:** `web/src/episodes/caching-parity.test.ts`, `web/src/determinism.test.ts`, core caching tests/identity fixtures, existing source-integrity tests if their checked manifest needs an additive entry; `crates/sugarscape-wasm/tests/experiment_view_tests.rs`.
**Interfaces:** Existing CLI `experiment-view catalog|run|validate|recorded`, common WASM exports, old P3/P4 exports; no new worker operation.

- [x] **Step 1: Write RED reconstruction/transport tests.** Save a freshly run envelope, tamper local received evidence, researcher fingerprint, terminal payload, input, version and source identity one at a time; each import rejects. Roundtrip correct envelopes exactly, preserve max seed/span and unknown-field errors, and exercise configured byte bounds before parse/retention. Keep P4 cost 1 invalid and P3 nonfinite/negative/out-of-domain fields rejected.
- [x] **Step 2: Run native/WASM matrix.** Use built release CLI and actual generated WASM with both literal inputs, all named presets, max seeds, and representative mirror/clear-zero/cue-mismatch cases. Compare full lossless payload, projected clocks/local/researcher fields and fingerprint strings exactly. Repeat original bare P3/P4 exports and compare with preimplementation fixtures; generic Sim fingerprint stepping still matches original records. Add no tolerance; retain actual deltas/failing artifacts if any comparison fails.
- [x] **Step 3: Confirm protected source and artifacts.** Verify original engine/settings/report files and guide prefixes against saved hashes. New adapters/descriptors may add source identities; test hash inventories/digests against actual workspace files and disclose any source rebinding. Capture first/repeat reports separately and verify exact equality; exclude raw evidence from production assets. Native scientific campaigns are not rerun.
- [x] **Step 4: GREEN and independent boundary review.** Build release CLI, run `npm run wasm` in web, focused caching parity and old P3/P4 determinism tests; inspect native/WASM receipts with reviewer. Commit `test(experiments): verify caching viewer parity and preservation`.

## Task 5 / Stage 5: Browser Acceptance, Product Docs, and Delivery

**Goal:** Reviewed source-bound delivery of fifteen episode entries, with precise limitations and exact-commit remote verification.
**Files:** `docs/experiment-viewer.md`, `docs/superpowers/specs/2026-10-08-campaign-visualization-inventory.md`, relevant product guide links/copy, owned tracker/spec/plan completion metadata. Scientific findings stay corrected product content, never an invented bug narrative.
**Interfaces:** Existing approved browser-control tooling, repository build/test commands and Git integration. Root owns final evidence/Git; producer owns product docs and any review fixes.

- [ ] **Step 1: Execute real browser acceptance.** Apply browser skill; start only owned server/tab resources. Verify both defaults, P3 cue mismatch/erased/selective evidence, P4 sham/neutral/clear/unseen/paid examples, researcher versus Agent switch, reference-map labels, stale ages, stepping, replay, comparison, share/export/import, malformed current JSON, cancellation/retention/disposal, keyboard/narrow/reduced-motion and representative A/B regressions. Bind served/executed JS, CSS and WASM to reviewed source/build; explicit expected module/event coverage prevents empty captures from counting as success. Restart owned Vite after source changes in this worktree. Record any unverified interaction explicitly; close only owned resources.
- [ ] **Step 2: Update docs and completion metadata.** Describe examples versus measured findings, private/reference layout contract, fixed schedule, supported controls, error/import behavior and actual limits. Mark A/B delivered and C delivered only when delivery is verified, removing stale proposed statuses with accurate wording. Update spec/plan implementation state without overwriting approval receipts. Review docs/relative links and diff whitespace.
- [ ] **Step 3: Run final local gates serially on final source.** Root binds a final manifest; dedicated build owner records complete command output/counts and ignored/skipped distinctions:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo build --release -p sugarscape-cli
```

In `web`: `npm run build`, `npm run typecheck`, `npm test`. From the worktree root: `python3 -m unittest discover -s studio/tests -t studio -v`. Workspace fmt is check-only; repair formatting only in edited Rust files. Do not repeat broader gates unless source changes/failures justify it. Native code/source-identity changes require parity and dependent browser evidence refresh, not reuse of old receipts. Existing browser-only WASM skips are disclosed.
- [ ] **Step 4: Fresh final independent review.** Reviewer compares full delivered diff to approved spec/plan and source-bound receipts, including all visibility/clock/invalid-input/source-preservation boundaries. Route findings to producers, scope corrections and rerun required checks. Snapshot/remove owned completed tracker, retain all evidence, commit reviewed docs/fixes with explicit files. No open material review findings at integration.
- [ ] **Step 5: Integrate and verify exact commit.** Inspect main and all relevant worktree status/HEADs; coordinate any concurrent integration using actual Git state without discarding/stashing others. Merge incoming main into crowd if needed, resolve through producer/review, refresh source identities/checks warranted by the merged source. Integrate reviewed crowd into main normally, push, and inspect CI plus Pages for that exact final SHA. If bookkeeping produces a later delivered commit, verify that SHA as well. Record remote run links/conclusions and final source receipt; report two new entries, checks, scientific preservation and material limits. Preserve crowd and evidence.

## Self-review and Handoff

Tasks 1–2 cover codecs, native authority, field provenance, clocks, fixed schedule, bounded capture and frozen examples. Task 3 covers perspective rendering, comparison, controls and lifecycle. Task 4 covers exact reconstruction/transport/native-WASM preservation. Task 5 covers browser behavior, source-bound verification, docs and safe delivery. Each Review Focus item has behavioral tests in its owning stage; no new scientific collection or sensory instrumentation is included.

Execution remains subagent-driven under the user's standing preference. Review of this written plan is the remaining prerequisite before creating the tracker and starting runtime implementation.
