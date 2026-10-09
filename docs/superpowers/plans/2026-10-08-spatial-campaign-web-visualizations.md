# Spatial Campaign Web Visualizations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Fresh producers and independent reviewers are the user's preserved execution method. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add five bounded Burrow/CPFA spatial episode viewers to the existing Experiments interface while preserving original engines, exports and scientific artifacts.

**Architecture:** Add strict browser input DTOs and native projection adapters around original runners/World APIs, extending the common episode envelope and existing worker exports. A shared spatial renderer receives only checkpoint-projected data; native engines supply behavior, snapshots and private memories. Existing Batch A records, links and renderer contracts remain valid.

**Tech Stack:** Existing Rust workspace, serde/serde_json, wasm-bindgen/wasm-pack, TypeScript, Vite, Vitest, plain DOM/SVG or canvas using current utilities, approved browser-control tooling. No new framework, dependency or training pipeline.

**Spec:** [Approved Batch B design](../specs/2026-10-08-spatial-campaign-web-visualizations-design.md). User approval: LGTM on October8,2026; original approved bytes/hash retained in `.superpowers/sdd/2026-10-08-spatial-campaign-web-visualizations/spec-approval.json`. This plan was approved by user LGTM! on October9,2026; execution is subagent-driven.

## Global Constraints

- Scope: `burrow_excavation`, `burrow_access`, `foraging_fixed`, `foraging_passage`, `foraging_construction`. Preserve the existing eight Batch A studies; final catalog has13 runnable entries. F5, P3/P4, raw surface discovery, new controllers and scientific collection are excluded.
- Existing engines, controller parameters, schedules, random streams, original exports, reports, and settings remain unchanged. No new biological, coordination, learning, optimality or treatment-effect claim. New examples are labeled engineering demonstrations.
- Retain64KiB raw/normalized input and decompressed-link limits,4096checkpoints,16MiB complete episode limit, one request/worker,60-second cancellation/timeout, and at most two successful records. Native stricter limits apply; reject oversized requests instead of changing their inputs.
- Browser spatial profile adds at most4096grid cells,16Agents and8192requested opportunities. These are resource limits for this viewer, not changes to the underlying labs. Validate dimensions/count/products/sample retention before World/trace allocation; also enforce the16MiB budget incrementally while capturing/projecting.
- Seeds, arbitrary u64 resource/material IDs, and unbounded u64 input counters such as Burrow freshness_window use canonical decimal strings at the browser boundary. Bounded numeric indexes/coordinates/config counts remain numbers. Payload integer atoms use the existing lossless string conversion; f64 outputs remain their original numeric values.
- Agent data stays checkpoint-local. Unknown cells stay unknown; remembered walls can be stale. No invented observations, observation ages, global food knowledge, paths, occupancy counts or CPFA per-action logs. Researcher and final result gates remain explicit.
- Burrow native frame order and distinct same-tick stages are preserved. CPFA snapshots describe completed ticks after sequential ascending-ID opportunities. Comparisons join actual clocks/stages, preserve setup/seed/horizon/sampling and vary only one declared compatible control; same seed does not promise paired random draws.
- Same-target fresh reconstruction is authority. No CPFA tolerance is added to the Batch A testimony-only1e-12 exception. Cross-target actions/geometry/identities/inventory/costs/clocks/shape/availability must match on declared fixtures; retain and report any floating differences and investigate required comparison failures before release.
- Extend `experiment-view` and existing four WASM episode exports, not separate page-thread engines. Preserve legacy Burrow exports and ordinary Playground/sweeps. No raw diagnostic archive is bundled to display a selected episode.
- Existing worktree `.claude/worktrees/crowd`, branch crowd. Do not recreate/remove it, discard/stash unrelated work, stage `.claude/`, `papers/`, `survey/out/`, ignored evidence, or the deferred surface-discovery draft. Preserve all old evidence. Commit reviewed working tasks; merge/push completed work and verify exact final-commit CI/Pages.
- American spelling; Agent in code/docs. Rustfmt only edited Rust files with child traversal disabled; workspace formatting check is allowed; no survey formatting/execution added locally. Stop after three unsuccessful attempts on an issue and document/reassess.

## Review Focus

1. Opened custom/invalid/missing controls preserve original input until explicit edits or contextual rejection; failures never replace the shown successful episode — Tasks1/6.
2. Same-tick Burrow choice frames and off-cadence terminal sampling never collapse or zip unequal frame/series arrays — Task2.
3. Private remembered solid cells and own cargo/finds never become fresh researcher truth/global food knowledge — Tasks3/4/6.
4. Maximum u64 IDs, equal food/spoil IDs, censored milestones, and sample/resource budgets preserve exact meaning — Tasks1/4/5/6.
5. Matched controls, repeated keyboard seeks, cancellation/disposal and navigation preserve clocks/focus/selection and release records correctly — Tasks5/6/7.

---

## Files and interfaces

| Unit | Responsibility |
| --- | --- |
| `browser_experiments/spatial/input.rs` | Strict boundary DTOs, decimal ID/counter conversion, delegation to native setup validation |
| `spatial/scenes.rs` + `fixtures/spatial-scenes.json` + `fixtures/spatial-identities.json` | Source-bound engineering examples, defaults, controls and comparison declarations |
| `spatial/budget.rs` | Checked sample/opportunity/record-size retention limits |
| `spatial/burrow.rs` | Original ordinary/access runner output and sampled frame/event projections |
| `spatial/fixed.rs` | F2 World capture and original native Episode payload |
| `spatial/passage.rs`, `spatial/construction.rs` | F3/F4 capture and private knowledge maps |
| `spatial/mod.rs` | Implemented-study dispatch/descriptors and source/target identities |
| `web/src/episodes/spatial-{controls,projection,view,comparison}.ts` | Named controls, pure permitted display data, map/inspector rendering, declared comparison recipes |
| Existing common catalog/input/mod, bridge and episode view modules | Small additive dispatch/types/integration changes only |

New normalized inputs:

```text
{study:"burrow_excavation",config:BurrowConfigInput,seed:decimal_u64,ticks:u32,sample_every:u32}
{study:"burrow_access",config:{lab:BurrowConfigInput,task:burrow::AccessTask},seed:decimal_u64,ticks:u32,sample_every:u32}
{study:"foraging_fixed",setup:FixedSetupInput,seed:decimal_u64,ticks:u32,sample_every:u32}
{study:"foraging_passage",setup:PassageSetupInput,seed:decimal_u64,ticks:u32,sample_every:u32}
{study:"foraging_construction",setup:ConstructionSetupInput,seed:decimal_u64,ticks:u32,sample_every:u32}
```

All new DTOs deny unknown fields. `IdText::value()->u64` validates digits/range/canonical serialization. `BurrowConfigInput` mirrors LabConfig with native Fixture/Transport/Cue, string freshness_window and original bounded u32 fields; `to_core()->Result<burrow::LabConfig,Vec<FieldError>>` delegates `.validate()`. Access input delegates the native task/config path. `PositionInput{x:u32,y:u32}` converts into each family's Pos. `ResourceInput{id:IdText,pos:PositionInput}` converts without rounding. Fixed parameters mirror all seven original CpfaParameters fields; passage/construction share the original five active fields. Each setup DTO's `to_core()` constructs the existing Setup and calls its `.validate()` before World allocation; source modules acquire no Deserialize derives or behavior edits.

New record payload is `{native:<complete original native Episode or AccessEpisode>,capture:{sampling:<declared input>,clock_semantics:"native_sampled_tick"|"completed_tick_boundary",target:<actual CPFA ARCH/OS, omitted for Burrow>}}`. The native subobject uses `wire::lossless_value` on the original serialized output. Checkpoints are in native boundary order; `kind` is `spatial_initial`, `spatial_sample`, or `spatial_terminal`. Clock includes decimal completed_ticks and native tick where applicable; the stage remains distinct even when clocks match. Local keys are actual Agent IDs.

Researcher data includes native frame/snapshot, decoded Burrow ASCII cells and overlay caveat or CPFA physical state. Local Burrow data is committed own events/choices and available native completion markers; observations/map/current holdings are null with explicit availability labels. Local F2 has only selected AgentView and supplied arena/nest; F3/F4 also have that Agent's captured KnowledgeView. `capture_at` describes capture time, not time each memory cell was learned. All geometry sent to the renderer is already permitted by the chosen perspective.

CPFA rules identities include original source digest, adapter schema version and actual ARCH/OS target label. Same-target import is exact; foreign target identities get a contextual incompatibility error rather than a tolerant import. Cross-target verification compares declared semantic projections while retaining complete native/WASM records and floating deltas; it does not certify universal portability. Existing eight rules identities and old Burrow exports remain unchanged.

## Execution tracker and reviews

After plan approval, create owned `IMPLEMENTATION_PLAN.md` with Goal/Success Criteria/Tests/Status fields for five stages:1contracts+Burrow(Tasks1–2);2CPFA(Tasks3–4);3bridges/parity(Task5);4spatial UI(Task6);5verified delivery(Task7). Update it as work proceeds, retain a completed evidence snapshot and remove only that tracker at delivery.

One implementer at a time. Task reviewer reads the full staged source/diff and source-bound report before the root commits; root only handles metadata/Git/evidence. Review both spec and quality, resolve material findings through producers with scoped rereview. Final whole-branch review is fresh and independent; retain all receipts and ruling correction costs. Do not delete evidence during finishing.

## Task 1: Strict spatial inputs, example provenance and retention profile

**Files:** Create `crates/sugarscape-core/src/browser_experiments/spatial/{mod,input,scenes,budget}.rs`, `fixtures/{spatial-scenes,spatial-identities}.json`, `tests/spatial_input.rs`; modify common `mod.rs`, `input.rs`, `catalog.rs`, `tests/mod.rs`. No Burrow/foraging engine edits.

**Interfaces:** Produce the five DTO variants above, their `to_core` conversions, `IdText`, `spatial::validate_input(&Input)->Result<(),Vec<FieldError>>`, `spatial::scenes::input_for(&str)->Result<Input,Vec<FieldError>>`, `spatial::budget::preflight(&Input)->Result<(),Vec<FieldError>>` and `CaptureBudget::charge<T:Serialize>(&T)->Result<(),Vec<FieldError>>`. `CaptureBudget` accounts each retained component plus wrapper/delimiter allowance and rejects before appending; final `check_bounds` remains authority. Catalog includes only implemented runnable adapters; unavailable dispatch returns contextual error, never a synthetic success.

- [x] **Step1: Baseline/provenance.** Bind all original `burrow/` and `foraging/` files and preserved Batch A protected sources by SHA256 in owned evidence. Save legacy ordinary/access replay bytes for the six docs/examples/burrow files at seed7 and u64max using original APIs. Read existing engineering reference receipts without changing them; no survey campaigns. Preserve a source-bound core baseline command/log before new behavior. The verified Batch A fullcore execution may be reused only if all relevant current source and compiled artifact hashes match its retained receipt; otherwise run a new baseline. Bind supporting RNG/information/rules files and Cargo.lock as well as lab folders in spatial-identities.json; old identities stay unchanged.
- [x] **Step2: RED strict input/budget tests.** Missing required fields, extra coordinate keys, malformed seed/ID/window strings, duplicate IDs, out-of-bounds positions, native parameter violations, valid native65×65grid/Agent17/threeAgents×2731ticks (8193opportunities), zero sampling and excessive frames must reject without normalization to defaults. Pin exact ID conversion:

```rust
let input = normalize_input(&raw_fixed_with_id("18446744073709551615")).unwrap();
let Input::ForagingFixed { setup, .. } = input else { panic!("wrong study") };
assert_eq!(setup.to_core().unwrap().resources[0].id, u64::MAX);
assert!(normalize_input(&raw_fixed_with_id("18446744073709551616")).is_err());
```

`raw_fixed_with_id` is the test-owned JSON builder using the fixed default below. Run `cargo test -p sugarscape-core browser_experiments::tests::spatial_input`; retain compiled assertion RED, not just absent-module output.
- [x] **Step3: Implement codecs and conservative preflight.** Use original typed validators; estimate bounded retained geometry, traces, inventory/history and all-Agent sparse maps from declared counts before stepping. Reject arithmetic overflow. Burrow config.validate is public; corridor dimensions can use `fixtures::corridor_setup` without constructing a World; growing fields provide dimensions, choice uses its documented9×7/one-worker fixture. No private helper calls or extra World for validation. Keep the original runner's own limits.

```rust
let opportunities = u64::from(workers).checked_mul(u64::from(ticks))
    .ok_or_else(|| error("opportunities", "checked product overflow"))?;
if opportunities > 8192 { return Err(error("opportunities", "browser spatial limit is 8192")); }
// Native setup/config validation precedes allocation; retained bytes are also charged before append.
``` Bound all stored components while projecting/capturing, including repeated prefixes/private maps and complete native payload, not just native snapshot_bytes.
- [x] **Step4: Pin examples and GREEN.** Defaults: ordinary `direct-blind.json`, access `access-explore.json`, both seed7/ticks128/sample16; retain all native config values with wire freshness_window="32". F2 seed12/ticks20/sample7:5×5,nest(2,2),oneAgent,resourceu64max at(3,2),p_search1,p_return0,omega0 and all lambdas0. F3 seed12/ticks40/sample7:5×5,open[(0,0),(1,0),(2,0),(3,0),(3,1)],nest[(0,0),(1,0)],worker[(0,0)],resourceu64max at(3,0),p_search1,p_return0 and all three lambdas0. F4 seed12/ticks40/sample7:5×3,open[(0,0),(1,0),(2,0),(0,1)],diggable[(3,0)],nest[(0,0),(1,0)],waste(0,1),worker[(0,0)],foodu64max at(3,0),same five parameters. These match product-guide setups; browser Burrow horizons are explicitly illustrative. Scene IDs are `burrow-excavation-default`, `burrow-access-default`, `foraging-fixed-default`, `foraging-passage-default`, `foraging-construction-default`. Supplemental IDs: `burrow-choice` as Task2; `burrow-corridor` length9/workers2/direct/blind with the same32/3/3/2 config fields,seed7/ticks32/sample7; `burrow-access-known-goal` changes only objective in the access default. `foraging-passage-disconnected` changes the F3 open mask to[(0,0),(1,0),(3,3)] and moves its resource to(3,3). `foraging-construction-protected` changes the F4 mask to[(2,0)], leaving food buried at(3,0). `foraging-construction-no-dig` uses the F3 geometry/resources as F4 food, empty mask and waste(3,1). `foraging-passage-congestion` adds a second worker at(0,0), within native capacity. `foraging-construction-equal-ids` changes the default food ID to"0", so the first native spoil unit can share the numeric ID while retaining its distinct namespace. Information variants for each CPFA default use p_search0.5,p_return0.25,lambda_fidelity1,lambda_publish1,lambda_waypoint0.1; fixed additionally omega0.5,lambda_informed0.5. Supplemental CPFA IDs append `-information` to the corresponding default stem and use seed12/ticks40/sample7. Preserve a provenance row describing each intentional engineering variant before outputs are judged; no efficacy claim. Run focused tests/edited-file rustfmt/core Clippy; self-review and independent contract gate.
- [x] **Step5: Commit reviewed source.** `feat(experiments): define bounded spatial episode inputs`.

## Task 2: Burrow excavation and structural-access display adapters

**Files:** Create `spatial/burrow.rs`, `tests/spatial_burrow.rs`; modify `spatial/{mod,scenes}.rs` and common catalog dispatch only.

**Interfaces:** `spatial::burrow::run(&Input)->Result<EpisodeRecord,Vec<FieldError>>`; original run_episode/run_access_episode called once. Register two descriptors with family spatial. Preserve full original native payload and matching frame/series semantics.

- [x] **Step1: RED real boundary tests.** Compare the full native subpayload with original APIs, at zero ticks, default128/16, off-cadence17/8 and choice1/1. Initial/terminal same-tick frames remain distinct; frames/series may differ in length. No zip by index. Own traces stop at the chosen sampled boundary; Explore never gets goal coordinates/private completion from KnownGoal. Blocked outcomes and Wait stay distinct, material ID stays string, zero-op rates/null access distances stay unavailable.

```rust
let record = run(&scenes::input_for("burrow-choice").unwrap()).unwrap();
assert_eq!(record.checkpoints.len(), 2);
assert_eq!(record.checkpoints[0].clock, record.checkpoints[1].clock);
assert_ne!(record.checkpoints[0].kind, record.checkpoints[1].kind);
assert!(record.payload["native"]["events"].as_array().unwrap().is_empty());
```

The choice fixture uses side left/pile fresh_accumulation, direct/blind, freshness32,relay3,response3,minimum_recent2,seed7,ticks1,sample1. Run `cargo test -p sugarscape-core browser_experiments::tests::spatial_burrow` and retain RED.
- [x] **Step2: Implement projection.** Call preflight then original runner; convert payload losslessly. Decode exactly width×height exported ASCII glyphs in Rust, ignoring legend bytes as cells, preserving caveat and fingerprint. Match snapshots by native tick/opportunity context, allowing one diagnostic snapshot to describe unchanged physical quantities at distinct same-tick stages. For normal sampled boundaries include only events/choices from completed rounds; for the terminal choice frame include the actual selected choice without a paid action. Access completion markers are filtered by exported opportunity prefix; structural access stays researcher-only. Never use final delivery records to fill early holdings. Do not treat the existing physical validate_episode function as authentication of controller choices/RNG; fresh original-run reconstruction remains the import authority.

```rust
let native = burrow::run_episode(config.to_core()?, seed.value(),
    burrow::RunOptions { ticks, sample_every })?;
let native_value = wire::lossless_value(serde_json::to_value(&native)
    .map_err(|e| error("episode", e.to_string()))?);
// Project native.frames in order; native_value is the complete gated payload.
```
- [x] **Step3: GREEN/preservation.** Compare complete ordinary/access outputs against Task1 baseline and original runners at both seeds. Show raw native equality before comparing wire values. Preserve original fingerprint/RNG source and all original counts, labels and stop reasons. Run focused tests, core Clippy/edited formatting and fullcore once final producer source is stable.
- [x] **Step4: Independent gate.** Reviewer checks future/goal privacy, exact frame clock joins, projection/cargo availability, budget assumptions and old export bytes. Fix through producer/scoped rereview.
- [x] **Step5: Commit.** `feat(experiments): adapt Burrow excavation and access replays`.

## Task 3: CPFA F2 World capture and own-state projections

**Files:** Create `spatial/fixed.rs`, `tests/spatial_fixed.rs`; modify spatial dispatch/scenes/source identities.

**Interfaces:** `spatial::fixed::run(&Input)->Result<EpisodeRecord,Vec<FieldError>>`; register fixed descriptor. Capture one original fixed::World, original Snapshot/summary and native Episode shape including original compact snapshot byte count.

- [x] **Step1: RED replay/read-only tests.** At default20/7 and horizons1/1,8/2 compare all original snapshots/summary/seed/snapshot_bytes to fixed::run with snapshots=true. Initial/final snapshots occur exactly once; no early stop after food depletion. Selected own state does not include other Agents/resources/server advice. Large cargo/resource IDs stay exact and censored event times remain null. A World stepped with extra observational calls must reach the same next snapshot as one stepped without them.

```rust
let mut plain = fixed::World::new(setup.clone(), 12).unwrap();
let mut observed = fixed::World::new(setup, 12).unwrap();
for _ in 0..20 {
    let _ = observed.snapshot().unwrap();
    let _ = observed.summary().unwrap();
    plain.step().unwrap(); observed.step().unwrap();
    assert_eq!(plain.snapshot().unwrap(), observed.snapshot().unwrap());
}
```

Run `cargo test -p sugarscape-core browser_experiments::tests::spatial_fixed`; retain genuine RED for new adapter assertions.
- [x] **Step2: Implement original step loop and bounded capture.** `World::new(setup,seed)`, initial capture, `for completed in1..=ticks { world.step()?; if completed%sample_every==0 || completed==ticks { capture } }`, final summary. Use native serialization to reproduce snapshot_bytes; no physics/controller code copied. Prefix reader data and native Episode components are charged to CaptureBudget before retention. Supplied arena/nest can be public; own AgentView is local; resources/otherAgents/waypoints are researcher-only. Do not infer local food observations.

```rust
let mut world = fixed::World::new(setup.to_core()?, seed.value())?;
capture_fixed(&world, "spatial_initial", &mut capture)?;
for completed in 1..=ticks {
    world.step()?;
    if completed % sample_every == 0 || completed == ticks {
        capture_fixed(&world, if completed == ticks { "spatial_terminal" } else { "spatial_sample" }, &mut capture)?;
    }
}
```

`capture_fixed(&fixed::World,&str,&mut Capture)->Result<(),Vec<FieldError>>` is internal to fixed.rs; `Capture` owns checked original Snapshot retention plus checkpoint/budget state. It reproduces native snapshot_bytes using original Snapshot serialization before wire conversion.
- [x] **Step3: GREEN/profile.** Run focused tests, core Clippy/edited formatting; reference metadata binds original engine digest and target ARCH/OS. Preserve all original fields, including original target-dependent floats; no new tolerance path.
- [x] **Step4: Independent gate.** Reviewer checks runner equivalence, RNG/capture, private/global separation, exact IDs and budget accounting.
- [x] **Step5: Commit.** `feat(experiments): capture bounded fixed-world foraging episodes`.

## Task 4: CPFA F3/F4 physical snapshots and private-memory capture

**Files:** Create `spatial/{passage,construction}.rs`, `tests/spatial_passage.rs`, `tests/spatial_construction.rs`; modify spatial dispatch/scenes only.

**Interfaces:** `spatial::passage::run(&Input)->Result<EpisodeRecord,Vec<FieldError>>` and `spatial::construction::run(&Input)->Result<EpisodeRecord,Vec<FieldError>>`; all five descriptors now runnable. Use existing World::knowledge(id), snapshot/summary/step. A shared adapter capture helper may factor bounded serialization and sample scheduling, not world/controller behavior.

- [x] **Step1: RED native equivalence/privacy tests.** Compare complete native Episodes to each original runner at40/7 and8/2. At every recorded boundary compare private maps with the corresponding original World. Validate map/counter/RNG continuation invariance under extra read-only captures. The no-dig F4 example keeps original F3 physical projections where comparable. Censored access/pickup/delivery is null, not zero. Food/spoil equal numeric IDs have different cargo tags. A remembered solid wall remains a solid memory even when researcher terrain is open; adapter must copy knowledge exactly rather than replace it from snapshot.

```rust
let local = world.knowledge(0).unwrap();
let at = adapter_checkpoint(&world).unwrap();
assert_eq!(at.local["0"]["knowledge"], wire::lossless_value(serde_json::to_value(local).unwrap()));
assert!(at.local["0"].get("global_food").is_none());
```

`adapter_checkpoint` is a test-visible capture helper within the owning adapter, returning Checkpoint and using no RNG. Run `cargo test -p sugarscape-core browser_experiments::tests::spatial_passage` and `cargo test -p sugarscape-core browser_experiments::tests::spatial_construction`, retaining genuine RED.
- [x] **Step2: Implement one-World drivers.** Use original normalizing World constructor and sampled step schedule. Capture all bounded private maps one Agent at a time, charging each before retaining. Label capture clock separately from unavailable cell observation timestamps. Preserve F4 native Cargo enum namespace, food/spoil/terrain/access/milestone fields and typed partial states. No global access cache or hidden-food origin lookup enriches local cargo/find fields. Include full normalized native setup in payload.

```rust
let snapshot = world.snapshot()?;
for agent in &snapshot.agents {
    let knowledge = world.knowledge(agent.id)?;
    budget.charge(&knowledge)?;
    // Pair knowledge with this AgentView only; snapshot global fields remain researcher data.
}
```

Family-local capture helpers convert these original typed views into the declared Checkpoint; charge converted wire data too before retention.
- [x] **Step3: GREEN/ref cases.** Public setup fixtures include disconnected exposed food and protected buried food, empty food, duplicate nest occupancy within capacity, mask already open, and nonzero publication/fidelity/waypoint parameters. Use existing core tests as semantic references; tests may use synthetic projected checkpoints for stale-wall display, but native capture equality must use real Worlds. No claim that every selected run produces delivery or a stale wall. Run focused tests, edited formatting/core Clippy, and fullcore on final source.
- [x] **Step4: Independent gate.** Check stale memory, namespaces, censored milestones, original normalization and accounting, all-Agent memory budget and no unsupported action-level narrative.
- [x] **Step5: Commit.** `feat(experiments): capture passage and construction knowledge`.

## Task 5: Shared bridges, complete parity and actual browser feasibility

**Files:** Modify `crates/sugarscape-cli/tests/experiment_view.rs`, `crates/sugarscape-wasm/tests/experiment_view_tests.rs`, `web/src/episodes/{types,parity.test,client.test}.ts`; create `web/src/episodes/spatial-parity.test.ts`. Existing bridge implementation changes only if exhaustive dispatch requires them; no new page-thread exports or worker concurrency.

**Interfaces:** Existing CLI `experiment-view catalog|run|validate`, WASM `experiment_catalog_json/experiment_run_json/experiment_validate_json`, and EpisodeClient unchanged. StudyId gains five values, descriptor family gains spatial. `rendererAvailable` remains false for unrendered spatial family until Task6 supplies real renderer.

- [x] **Step1: RED bridge/parity tests.** All five defaults produce complete checked records, catalog count13, inputs/export bounds enforced, missing/invalid/large-ID fields reject, malformed saved shape/source/target/payload reject, same-target import returns fresh equal authority. Preserve old Burrow full-record parity tests and Batch A41-case behavior. CPFA target identities differ explicitly; foreign-target imports reject before promising equivalence.
- [x] **Step2: Compare complete native and WASM outputs.** Build release CLI, build actual WASM, then generate bounded test inputs through both.

```typescript
const native = runNativeFixture(input);
const browser = JSON.parse(wasm.experiment_run_json(JSON.stringify(input)));
expect(canonicalSpatialSemantics(browser)).toEqual(canonicalSpatialSemantics(native));
```

`runNativeFixture(input:Json):EpisodeRecord` writes a test-owned input file and invokes the release experiment-view CLI; `canonicalSpatialSemantics(record:EpisodeRecord):Json` retains all declared shape/action/geometry/identity/inventory/cost/clock/null data and records target/float differences in a separate complete-record receipt. Only declared target metadata, computed float inspection leaves and their serialization-dependent retention bytes are separated from the semantic comparison. Require original-source/adapter identity prefixes to match before handling the declared CPFA target suffix. Computed float paths are F2 Snapshot/own-Agent heading and each CPFA Snapshot's waypoint strength, including their corresponding checkpoint projections; parameters and every other numeric/shape/null field remain exact. Preserve each excluded path/value in the float-delta receipt. Native snapshot_bytes is retained/recomputed exactly per target and reported separately, because original f64 spelling can change byte length; it is not a paid-work cost. Use no tolerance, rounding or zeroing of arbitrary fields and never reuse this cross-target diagnostic projection for import authentication. Burrow has no exclusions. These helpers live in spatial-parity.test.ts; original records remain untouched. Burrow records should match completely under existing portable contract. CPFA retain complete records and compare every declared semantic shape/action/geometry/identity/inventory/cost/clock/null value exactly; include actual full float deltas rather than quietly dropping them. Same-target roundtrips compare full records including floats. If semantic parity or same-target reconstruction fails, stop the dependent UI task for diagnosis/review; any new tolerance or source revision needs an approved design amendment.
- [x] **Step3: Lifecycle and feasibility GREEN.** Run `cargo test -p sugarscape-cli --test experiment_view`, `wasm-pack test --node crates/sugarscape-wasm`, web original parity plus spatial parity and client tests, TypeScript. Use the approved browser skill to run the real worker on five defaults, choice/zero/off-cadence cases and declared expensive bounded examples. Record wall time/record bytes/peak live workers and termination/cancel/60-second timeout using documented real nonreply-worker injection. Defaults must finish within60seconds and16MiB; no hidden horizon/sampling/parameter adjustment.
- [x] **Step4: Independent gate.** Check complete source/target parity receipts, float handling, source-bound executable bytes, old exports, limits and worker release. Tested cases do not imply universal platform/RSS guarantees.
- [x] **Step5: Commit.** `feat(wasm): expose bounded spatial experiment records`.

## Task 6: Spatial controls, maps, perspectives and matched comparisons

**Files:** Create `web/src/episodes/{spatial-controls,spatial-projection,spatial-view,spatial-comparison}.ts` and pure `spatial-projection.test.ts` / `spatial-comparison.test.ts` plus real-DOM `spatial.browser-test.ts`; minimally modify `controls.ts`, `catalog.ts`, `view.ts`, `comparison.ts`, `types.ts`, `web/src/style.css`. No framework or existing simulation engine rewrite.

**Interfaces:** `renderSpatial(at:Checkpoint,descriptor:RenderDescriptor):HTMLElement`; `spatialDisplay(at:Checkpoint):Json`; `renderSpatialResult(payload:Json):HTMLElement`; `SpatialControls(descriptor:StudyDescriptor,input:Json,changed?:(input:Json)=>void)` exposes el and input(). Comparison declares `SpatialAxis{id:string,path:string[],values:Json[]}` in catalog controls, `spatialComparisonKey(input:Json,axis:SpatialAxis):string`, `spatialMatchedInputs(input:Json,descriptor:StudyDescriptor,axisId:string):Json[]`. Preserve existing surface helpers. EpisodeView stores selected axis/recipe and verifies key equality on install; generic comparison joins existing clock+kind helper. Export existing `canonical(value:Json):string` from comparison.ts for spatial-comparison reuse; avoid reciprocal module imports. View compare accepts optional catalog SpatialAxis and uses its matching key only for spatial recipes, preserving surface behavior.

- [ ] **Step1: RED pure/display tests.** Agent memory does not reveal researcher cells, food or peer positions. Burrow glyph W does not turn into invented Agent ID or precise pile count. Equal food/spoil ID strings retain distinct names; null/censored milestones remain unavailable. Sample interval changes do not invent a path. Comparisons deep-clone and alter only a catalog-declared path; seed/setup/horizon/sampling/worker order/goal stay unchanged. Invalid/missing/custom input remains intact.

```typescript
const shown = projectCheckpoint(record, 1, {kind:'agent',agent:'0'});
expect(shown.researcher).toBeNull();
expect(spatialDisplay(shown)).not.toHaveProperty('global_food');
const other = spatialMatchedInputs(input, descriptor, 'transport')[0];
expect(spatialComparisonKey(other, axis)).toBe(spatialComparisonKey(input, axis));
```

Tests own synthetic checkpoint/native fixture values; do not assert fabricated actual-world states. Run `npx vitest run src/episodes/spatial-projection.test.ts src/episodes/spatial-comparison.test.ts` and retain behavior RED. Renderer/control behavior uses the actual browser test module, not a copied DOM selection implementation or new fake-DOM dependency.
- [ ] **Step2: Implement readable controls and map.** Reuse EpisodeControls preservation patterns and inherited theme. Named setups populate only on explicit selection; numeric CPFA parameter inputs use native fields/domains and retain malformed/missing opened values for contextual rejection. Resource IDs/window/seed stay text. A map fits/zooms the permitted grid; glyphs/labels identify terrain, nest/exit/outlet, namespaces and available Agent states. Inspector distinguishes current physical state, captured own state, historical action origins and remembered classifications. Legend states Burrow overlays can hide spoil/multiple workers. Map dimensions come from safe checkpoint metadata, never from future full payload.
- [ ] **Step3: Integrate persistent comparison controls.** Declare Burrow paths/values explicitly: ordinary config.transport direct/relay and config.cue blind/responsive; access config.lab.transport/config.lab.cue and config.task.objective explore/known_goal. No recipe changes fixture, task goal or weight. CPFA each comparison axis changes one of its original parameters using the values0 and1 for probabilities and0 and1 for supported lambda fields; fixed additionally omega0 and1 andlambda_informed0 and1. All declared values are valid originals, not optimized defaults. Preserve all unrelated fields; catalog declarations plus a family path allowlist, not arbitrary property deletion, determine axes. Reject unrecognized axis IDs or any path into geometry/resources/worker order before a request.

```typescript
const axis = declaredSpatialAxis(descriptor, axisId);
const candidate = spatialMatchedInputs(original.input, descriptor, axisId)[0];
if (spatialComparisonKey(candidate, axis) !== spatialComparisonKey(original.input, axis)) throw new Error("comparison inputs differ");
const next = await client.request("run", JSON.stringify(candidate));
// Install only if the same operation generation/original record is still current.
```

`declaredSpatialAxis(descriptor:StudyDescriptor,axisId:string):SpatialAxis` is exported from spatial-comparison.ts and validates the family/path allowlist. Stable controls keep the selected axis/value until its candidate set changes. Run one checked request, guard generation/identity before installing two records, preserve original if failure/stale. Keep select/slider nodes stable through draw, perspective and autoplay; empty compatible choices remain disabled after busy release. Each pane projects/gates itself independently.
- [ ] **Step4: GREEN actual browser.** Focused tests/fullweb/TypeScript/production build; actual browser all five defaults plus Batch A smoke. Track source-at-execution via served module snapshots/maps and WASM bytes. Inspect Burrow researchermap/action-only Agent view, F2 own state, F3/F4 private vs physical maps, supported/censored outcomes, comparison clocks, imports/exports/u64max, invalid/missing opened values and unsupported settings. Repeat real keyboard seeks, perspective/policy selection during redraw/autoplay, navigation away/back,390px/dark/reduced-motion, cancel/failure/timeout retention and disposal clearing all record references. Capture screenshots in owned evidence using approved browser tooling.
- [ ] **Step5: Independent gate and commit.** Fresh reviewer evaluates real readability, privacy, precision, availability and source-bound behavior, resolves findings through producer/scoped rereview. Commit `feat(web): visualize Burrow and CPFA spatial episodes`.

## Task 7: Preservation, guide, whole-branch verification and exact delivery

**Files:** Update `docs/experiment-viewer.md`; append brief viewer links/engineering-status text to `docs/burrow.md`, `docs/foraging.md` preserving original prefixes. Update this plan/tracker status. Keep all baseline, source, approved spec/plan, parity, browser and review receipts in owned ignored evidence.

**Interfaces:** Complete13-entry application, frozen original sources/exports, source-bound local/remote verification, documented actual support. No scientific campaign or additional viewer family.

- [ ] **Step1: Preservation and guide.** Rehash protected engines against Task1 manifests; compare original Burrow exports and CPFA native reference payloads/seed continuation. Preserve existing scientific first/repeat files and saved references using streaming hashes/reused verified receipts where unchanged; never parse multi-GB reports. Verify old guide prefixes, existing8-entry input/record compatibility, all5new defaults and target-identity behavior. Guide explains engineering examples, sampled ticks vs actions, unavailable Burrow observations, stale private memory, namespaces/censoring, caps, original-target portability/import limits and separate saved measurements.
- [ ] **Step2: Final local gates.** Record final-source cwd/exits/log hashes/counts. Release CLI precedes web parity. No source mutation during gates.

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
npm run build
# repository root
python3 -m unittest discover -s studio/tests -t studio
```

Retain ignored-test counts and informational notices; don't suppress errors. Existing published public assets are inventoried separately from new viewer bytes; no ignored evidence/harness file is bundled.
- [ ] **Step3: Actual acceptance and final independent review.** Verify all13entries, three old render families+spatial, new native-reference cases, oldPlayground/Sweep/#s/#c/#x/#e routes, source identity, imports, altered/oversized records, visibility, invalid controls, navigation, comparison, repeated keyboard focus, narrow/reduced motion and cancellation/disposal. Use actual browser, not only Node/native tests. Fresh most-capable whole-branch reviewer checks approved scope/source/parity/limits/preservation/docs/all deferred findings. One final producer fix wave and scoped rereview; updated affected evidence on any changes, original receipts retained.
- [ ] **Step4: Commit reviewed docs/status and integrate.** Inspect main/remote status, operation markers/locks and concurrent commits; preserve unrelated work. Merge reviewed crowd normally; compare merged runtime bytes to verified source. Differences require named appropriate checks, conflict fixes through producer/reviewer. Push normally, confirm exact `git ls-remote` SHA, wait allCI+applicablePages jobs for that SHA. Any final status-only commit also gets exact-commit remote verification.
- [ ] **Step5: Completion.** Retain URLs, job/source receipts and ruling/correction-cost list. Complete five-stage tracker, preserve its snapshot, remove only owned temporary tracker; retain crowd/evidence/deferred draft and sync branch to verified delivered commit. Do not declare delivery complete with remote gates pending. Commit title `docs: document verified spatial experiment viewers` for guide/status changes, with subsequent status-only commit if required by truthful gate timing.

## Planning self-review and handoff

This plan is reviewed inline against the approved spec before user handoff. Check all spec sections have owning tasks, the five Review Focus rows have explicit tests, signatures are consistent, actual API visibility/serialization matches source, examples have provenance, no unavailable observation promise or placeholder remains, and no frozen-source/float exception is implied. Architectural approvals are sequential: this written plan requires user review before execution. The preserved execution method is subagent-driven; do not ask the user to choose it again.
