# Burrow 2 Resource Access Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure structural access to a supplied resource coordinate and compare legacy exploration with a locally applied known-goal heuristic.

**Architecture:** Compose an AccessConfig around the existing LabConfig and reuse the checked physical world. KnownGoal adds private per-worker completion memory through one optional controller-weight seam; Explore remains the legacy world with an external observer. Record event-time access in the shared runner and expose separate native/WASM adapters.

**Tech Stack:** Rust, serde/serde_json, existing clap and wasm-bindgen, TypeScript/Vitest; no new library or version. The CLI directly declares the existing workspace serde serializer.

**Spec:** [approved resource-access design](../specs/2026-10-04-burrow-2-resource-access-design.md).

## Global Constraints

- Status: implementation plan and design approved 2026-10-04; implementation and independent reviews complete; integration into `main` is complete at `6076aed`. Scientific registration/execution remain unchanged. This builds engineering examples; it does not define or execute a scientific campaign.
- Public fixtures are growing only; every task field is explicit, enum strings snake_case, unknown fields rejected. Goal starts in-bounds, solid and diggable; goal weight is positive and checked with cue weights and cell counts.
- Reuse four-neighbor physics, capacity-one carrying, capacity-two occupancy, identical action prices and spoil histories; no food ledger, collection, reward optimizer or blueprint.
- KnownGoal knows one supplied coordinate and its own local observations. Completion latches privately before decision, including loaded turns. No global completion signal, communicated discovery or hidden frontier.
- Preserve disposal/transport/pickup/retained-target/routing/wandering priorities. Only new frontier weights change; nonimproving faces stay eligible.
- Explore nested Episode bytes match `run_episode`; legacy schemas, fingerprints, counters and ASCII maps stay unchanged. Optional task absence hashes no sentinel. KnownGoal hashes task identity and all worker latches.
- Run the full fixed budget after first access. Freeze first event's 1-based opportunity cost and shortest open-cell route distance; inaccessible endpoints stay censored/null.
- Existing caps: 262144 cells, 4096 workers subject to fixture capacity, 1000000 requested opportunities, conservative retained ASCII 64 MiB. WASM validates original f64 before conversion to u32.
- No ModelKind/web controls/Minds/Hornvale changes. Execute in a separate global worktree after the measured harness, using fresh implementation/review subagents per task plus final whole-branch review.

## Review Focus

1. A remote worker learns completion without observing the goal: latch only from its ordinary local open cells, including loaded observations (Task 1).
2. A legal-looking task weight overflows selection arithmetic or the goal is invalid: reject before stepping/writing (Task 1).
3. Access occurs at the final action, or a later shortcut appears: distinguish success/censoring and preserve the first milestone (Task 2).
4. Adding task state perturbs ordinary replay or sampling changes behavior: enforce exact Explore/legacy bytes and weight-one physical reductions (Tasks 1–2).
5. JavaScript passes NaN, fraction, negative or oversized numbers, or CLI reuses output: reject at the original boundary without damaging output (Task 3).

---

## Staging and file map

At execution create root `IMPLEMENTATION_PLAN.md` with three stages matching Tasks 1–3 and the user-required Goal/Success Criteria/Tests/Status fields. Update status incrementally and remove the tracker on completion, after recording rulings and evidence here.

New `burrow/access.rs` owns public task/record types, validation, private task state and the observer. Existing controller owns the shared weighting/FSM; state/view own optional continuation state/fingerprinting; runner owns one shared action and episode loop. New access test file owns exact model checks. Separate CLI module and WASM export delegate to core; two JSON examples and a parity test demonstrate both policies without new browser UI.

### Task 1: Checked access config and private local guidance

**Files:**
- Create: `crates/sugarscape-core/src/burrow/access.rs`, `crates/sugarscape-core/src/burrow/tests/access.rs`.
- Modify: `crates/sugarscape-core/src/burrow/mod.rs`, `tests/mod.rs`, `state.rs`, `controller.rs`, `runner.rs`, `view.rs` within that directory.

**Interfaces:**
- Public serde records: `AccessObjective { Explore, KnownGoal }`, `AccessTask { goal: Pos, objective: AccessObjective, goal_weight: u32 }`, `AccessConfig { lab: LabConfig, task: AccessTask }`. Derive Clone/Debug/PartialEq/Eq and serde; deny unknown struct fields, no task defaults.
- Private `GoalState { task: AccessTask, seen_open: Vec<bool>, diagnostics: AccessDiagnostics }`; optional `World.goal_state: Option<GoalState>` exists only for KnownGoal.
- Private `GoalGuidance { goal: Pos, weight: u32, seen_open: bool }`, Copy, contains no global accessibility flag.
- Add `decide_with_goal_measured(o:&Observation,w:&WorkerView,c:&LabConfig,at_exit:bool,outward:&[ExitNeighbor],goal:Option<GoalGuidance>,rng:&mut SimRng)->(Decision,BfsStats,u64)`; last value counts goal-weight evaluations. Existing `decide_measured` forwards None and drops that last value, retaining its signature.
- `AccessDiagnostics { completion_observations: Vec<GoalObservation>, local_completion_checks: u64, goal_weight_evaluations: u64 }`, Default/Clone/Debug/Eq/serde; `GoalObservation { tick:u64, opportunities_before:u64, worker:u32 }`.
- Private `validate_task(task:&AccessTask,world:&World)->Result<(),Vec<FieldError>>` and `install_goal_state(world:&mut World,task:&AccessTask)->Result<(),Vec<FieldError>>`; the latter validates and installs Some only for KnownGoal. Separate public-fixture check precedes these in Task 2. Private tests may call setup validation on constructed three-cell worlds without weakening the public growing-only rule.

- [x] **Step 1: Write tests for validation, ticket weights, memory and compatibility.** Construct the private setup exit `(0,1)`, open `(0,1),(1,1)`, worker `(1,1)`, only diggable goal `(2,1)`. Reject out-of-bounds, open and nondiggable goals, zero weight, cue×goal product overflow and unsupported public choice/corridor. Enumerate each ticket for weights `[3,1]`, checking the three improving tickets and one nonimproving ticket. Test a retained target stays retained and congested movement waits. Assert goal completion stays false for a solid goal, latches only when locally open, latches while loaded without overriding disposal, and stays false for a remote worker.

```rust
#[test]
fn access_goal_ticket_keeps_nonimproving_face_eligible() {
    let weights = [3, 1];
    let choices: Vec<_> = (0..4).map(|ticket| select_ticket(&weights, ticket)).collect();
    assert_eq!(choices, vec![0, 0, 0, 1]);
}
#[test]
fn access_task_requires_explicit_goal_fields() {
    let json = r#"{"lab":{},"task":{"objective":"explore","goal_weight":3}}"#;
    assert!(serde_json::from_str::<AccessConfig>(json).is_err());
}
```

The ticket test must also build weights through the new goal-weight function, not just test existing `select_ticket`: define private `goal_factor(guidance:Option<GoalGuidance>,worker:Pos,target:Pos)->u32` and assert it returns 3 for an improving target, 1 for a nonimproving target, 1 after the latch, and 1 for None. Use a remote two-worker setup to verify there is no broadcast.

- [x] **Step 2: Run RED.** `cargo test -p sugarscape-core burrow::tests::access` must fail at absent access types/seam.

- [x] **Step 3: Implement the narrow seam and continuation state.** Validate actual setup with checked coordinates and weight products. Keep per-face weights u32 by checked cue×goal multiplication; sum candidate tickets in u64 with a checked cell-count bound. Distances use u64 sums of abs_diff. The factor is supplied heuristic only:

```rust
fn goal_factor(g: Option<GoalGuidance>, worker: Pos, target: Pos) -> u32 {
    let Some(g) = g else { return 1; };
    let distance = |p: Pos| u64::from(p.x.abs_diff(g.goal.x))
        + u64::from(p.y.abs_diff(g.goal.y));
    if !g.seen_open && distance(target) < distance(worker) { g.weight } else { 1 }
}
```

Refactor the existing controller body once; multiply ordinary cue weights only in its fresh-frontier selection branch. Retained targets skip weight evaluations. Preserve RNG draw count and integer-ticket type. In `worker_opportunity`, observe first, update only that worker's task latch from `observation.open`, append at most one first GoalObservation with pre-action event count, then pass its coordinate/weight/latch view into the shared decision. Count one local check per KnownGoal opportunity even after its latch; count weight evaluation per candidate actually weighted. Diagnostics do not enter another agent view or consume RNG.

Initialize `goal_state=None` in legacy constructors; access construction installs it for KnownGoal only. Fingerprint hash extends at one fixed position only when Some: explicit KnownGoal discriminant, packed coordinate, weight, vector length and ordered boolean latches. No None sentinel, diagnostics or observation counters are hashed. Add known-goal latch changes to continuation tests and assert old seed-7 fingerprints remain identical to pre-change values captured by tests before modification. Keep all new private fields internal; legacy public schemas untouched.

- [x] **Step 4: Run GREEN.** `cargo test -p sugarscape-core burrow::` and workspace fmt. Verify ticket factors, local memory, transport priorities and unchanged legacy fingerprints. Check the previously implemented saved physical validator still accepts ordinary seed-7 episodes.

- [x] **Step 5: Commit.** Stage the listed core files and execution tracker; commit `feat(burrow): add checked private goal guidance to local frontier selection` after core tests and lint pass. Review this task before adding runner exports.

### Task 2: Record structural access through one shared runner

**Files:**
- Modify: `crates/sugarscape-core/src/burrow/access.rs`, `runner.rs`, `mod.rs`, `tests/access.rs`.
- Modify narrowly: `state.rs` if observer hosting needs an internal field.

**Interfaces:**
- Consumes Task 1 AccessConfig/GoalState and existing Episode/RunOptions/Recording/exit field.
- Produces public `run_access_episode(config:AccessConfig,seed:u64,options:RunOptions)->Result<AccessEpisode,Vec<FieldError>>`.
- Public `AccessMilestone { tick:u64, opportunity:u64, worker:u32, goal:Pos, digs:u64, disposed:u64, carried:u64, loose:u64, exit_distance:u32 }`.
- Public `AccessSummary { structurally_accessible:bool, first_access:Option<AccessMilestone>, final_exit_distance:Option<u32>, observed_opportunities:u64, deadline_censored:bool }`.
- Public `AccessEpisode { config:AccessConfig, episode:Episode, access:AccessSummary, task_assumptions:Vec<String>, task_diagnostics:AccessDiagnostics }` with serde/Clone/Debug/PartialEq. Reexport all public task/record types from burrow.
- Private `AccessObserver { goal:Pos, first:Option<AccessMilestone> }`, independent of agent goal state. Add private runner host `run_world(world:World,options:RunOptions,observer:Option<AccessObserver>)->Result<(Episode,Option<AccessSummary>,AccessDiagnostics),Vec<FieldError>>`; `run_episode` delegates with None. Add `World::step_observed(&mut self,observer:Option<&mut AccessObserver>)` internally and forward public `World::step` with None. Pass the same optional observer through the existing `worker_opportunity`; reborrow per worker and never maintain a second action loop.

- [x] **Step 1: Write exact milestone, censoring and reductions tests.** On the private three-cell setup apply one legal dig and assert opportunity 1, tick at action clock, worker 0, distance 2, digs 1, carried 1, no other material. Full hands/blocked/far digs and waits never fabricate access. Construct a longer open detour and a later shortcut: first distance stays frozen while final distance decreases. Test last-action access succeeds, no access yields null milestone/distance and deadline_censored, zero ticks observes zero opportunities, and full budget continues after completion. Explore compares every Episode field and serialized bytes with legacy; KnownGoal weight 1 compares events/choices with Explore.

```rust
#[test]
fn access_explore_is_byte_identical_to_legacy_episode() {
    let lab = LabConfig::default();
    let options = RunOptions { ticks: 32, sample_every: 7 };
    let ordinary = run_episode(lab.clone(), 7, options.clone()).unwrap();
    let access = run_access_episode(AccessConfig {
        lab, task: AccessTask { goal: Pos { x: 7, y: 12 },
            objective: AccessObjective::Explore, goal_weight: 3 },
    }, 7, options).unwrap();
    assert_eq!(serde_json::to_vec(&access.episode).unwrap(),
        serde_json::to_vec(&ordinary).unwrap());
}
```

Add both-policy sampling tests comparing actions/choices/final fingerprints and task diagnostics at sample_every 1 versus 7; assert off-cadence terminal frames present. Exercise requested opportunity/ASCII/clock limits without executing huge budgets. Assert completion_observations length ≤ workers and each worker appears once, all opportunities_before precede its decision, outer lab equals nested config, and Explore diagnostics are zero/empty.

- [x] **Step 2: Run RED.** `cargo test -p sugarscape-core burrow::tests::access` must fail at `run_access_episode`/observer records before runner implementation.

- [x] **Step 3: Refactor shared hosting and observe after committed actions.** Extract existing `run_episode` initialization/sampling/assembly into `run_world` without changing sampling conditions, stop reasons, ledger finish or storage calculations. Preserve the existing action loop, allowing an external observer in its internal host path. The access runner validates growing fixture and task before stepping, installs GoalState for KnownGoal, creates the observer for both policies, and calls run_world. Reuse current exit topology field after each recorded transaction; no observer search touches the base BFS counters.

When goal is open and first milestone absent, freeze inventory/action totals and actual distance:

```rust
if observer.first.is_none() && world.open[world.index(observer.goal).unwrap()] {
    let inventory = world.inventory();
    observer.first = Some(AccessMilestone {
        tick: event.tick, opportunity: recording.events.len() as u64,
        worker: event.worker, goal: observer.goal,
        digs: world.excavated, disposed: inventory.disposed,
        carried: inventory.carried, loose: inventory.loose,
        exit_distance: recording.exit_field[world.index(observer.goal).unwrap()].unwrap(),
    });
}
```

This hook runs only after action recording and topology refresh. No task success changes `recording.stopped`. At finish accessible iff goal open, final distance comes from the same topology field; first_access remains historical, observed_opportunities equals final base count, deadline_censored iff inaccessible. Goal is initially solid so initial access is false. Preserve one initial frame for zero ticks and ordinary terminal behavior.

Record task assumptions explicitly: supplied coordinate knowledge for KnownGoal, Manhattan weighting rather than physical signal, local completion memory, structural rather than realized consumer access. Explore assumptions state task is observer-only. Completion-record logical storage is `task_diagnostics.completion_observations.len()`; document it separately from unchanged base storage. Keep checks and weighted candidate counts separate from base BFS metrics. KnownGoal nested Episode alone is not reproducible without outer AccessConfig; label that boundary.

- [x] **Step 4: Run GREEN.** `cargo test -p sugarscape-core burrow::` and formatting; run all core tests after shared runner changes. Confirm fixed-seed reductions and exact legacy bytes, censored/last-action semantics and historical distance. No success-rate threshold is an engineering acceptance gate.

- [x] **Step 5: Commit.** Stage listed core files and tracker; commit `feat(burrow): record event-time resource access without early stopping` after tests/lint and task review.

### Task 3: Checked native/WASM exports and fixed-seed acceptance

**Files:**
- Create: `crates/sugarscape-cli/src/burrow_access.rs`, `crates/sugarscape-cli/tests/burrow_access.rs`.
- Modify: `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/src/burrow.rs` (small shared exclusive export helpers only).
- Modify: `crates/sugarscape-wasm/src/lib.rs`.
- Modify: `crates/sugarscape-cli/Cargo.toml`, `Cargo.lock` (existing workspace serde dependency edge for the typed adapter summary).
- Create: `docs/examples/burrow/access-explore.json`, `docs/examples/burrow/access-known-goal.json`, `web/src/burrow-access-parity.test.ts`.
- Modify documentation: `docs/burrow.md`, `docs/superpowers/specs/2026-10-04-burrow-2-resource-access-design.md`, this plan.

**Interfaces:**
- CLI `sugarscape burrow-access --config FILE --seed U64 --ticks U32 --sample-every U32 --out DIR`; defaults seed 7/ticks 512/sample_every 32, config/out required. Existing Failure conversions give validation code 2/I/O code 1.
- WASM `burrow_access_replay_json(config_json:&str,seed:&str,ticks:f64,sample_every:f64)->Result<String,JsValue>` returns core AccessEpisode JSON.
- Native summary schema `AccessExportSummary { base: Snapshot, access: AccessSummary }`, serialize only in adapter; normalized outer config.json, outer episode.json, maps.txt and summary.json. Maps retain base glyphs; heading reports supplied task coordinate/policy and structural access metadata.

- [x] **Step 1: Write CLI preservation/numeric/parity tests first.** CLI: both policies and maximum u64 seed round-trip normalized config, full outer Episode and summary; invalid task creates no outputs; existing nonempty/file out paths remain untouched; race/exclusive-write failure reports partial path; zero ticks works. WASM: malformed JSON/contextual fields, unsupported fixture, invalid goal, strict seed failures and ticks/sample values 1.5/-1/NaN/±Infinity/4294967296 reject before u32 conversion; sample zero rejects, ticks zero accepts. Full-record parity matrix two policies × seeds 7 and 18446744073709551615, ticks 16/sample 4.

```typescript
it.each([1.5, -1, NaN, Infinity, -Infinity, 4294967296])(
  'rejects invalid original ticks %s', value => {
    expect(() => burrow_access_replay_json(configText, '7', value, 4)).toThrow();
  },
);
```

`configText` is loaded from `access-known-goal.json` with the existing node:fs/fileURLToPath/initSync pattern in `web/src/burrow-parity.test.ts`; use existing node shims and string paths, no new node:path import. Also assert the parsed thrown field is `ticks`, and run the equivalent table for sampling.

- [x] **Step 2: Run RED.** `cargo test -p sugarscape-cli --test burrow_access` must fail before command exists. Add the parity test before export implementation, then `npm run wasm` and `npx vitest run src/burrow-access-parity.test.ts` from web must fail at missing export. Do not treat a stale generated WASM package as implementation failure.

- [x] **Step 3: Implement thin checked adapters and examples.** Reuse existing decimal_seed, checked_burrow_u32 and field_errors at the WASM boundary:

```rust
let ticks = checked_burrow_u32(ticks, "ticks", 0)?;
let sample_every = checked_burrow_u32(sample_every, "sample_every", 1)?;
let episode = sugarscape_core::burrow::run_access_episode(
    config, seed, sugarscape_core::burrow::RunOptions { ticks, sample_every },
).map_err(field_errors)?;
Ok(serde_json::to_string(&episode).expect("access episode serializes"))
```

Parse config as AccessConfig with contextual `burrow_access_config` field and seed through decimal_seed before this snippet. Add the CLI command enum/match branch, validate/run core before output preparation, then exclusively write exactly four files. Extract current burrow's prepare_directory and exclusive-file-writing logic only if needed to share it; prove existing burrow outputs/error behavior unchanged. Do not add a general export framework. Error includes failing file and partial-output directory. Print final base map plus task/access/integer action summary.

Examples contain every LabConfig field, growing41×25/workers8/direct/blind/freshness32/relay3/response3/minrecent2, goal `(7,12)`, goal_weight3; objective alone differs between files. They are fixed engineering scenes, not treatment evidence. Preserve base # at the solid goal; metadata marks its coordinate externally.

- [x] **Step 4: Verify and record acceptance.** Run `cargo test --workspace`, `cargo test --manifest-path survey/Cargo.toml`, `cargo fmt --all -- --check`, `cargo fmt --manifest-path survey/Cargo.toml -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings` and `wasm-pack test --node crates/sugarscape-wasm`. From web run `npm test`, `npm run typecheck`, `npm run build` after regenerated WASM; inspect complete parity fields for both policies/full-width seeds. Run both 512-tick seed-7 example exports twice into new temporary directories and compare outer Episode JSON byte-for-byte within each policy. Inspect maps and material/access summaries without a preferred result or required success. Verify legacy CLI and WASM regression tests remain unchanged and pass. Update docs with checked commands, structural definition, supplied-information caveat, censoring, private latch and separate task storage. Record actual evidence and rulings here; remove completed execution tracker.

- [x] **Step 5: Commit and final review.** Commit `feat(burrow): expose checked resource-access replays across native and wasm` with listed adapter/tests/examples/docs. Fresh whole-branch reviewer checks local information, no duplicate controller loop, optional hashing compatibility, event-time measurements, numeric boundary checks and regression evidence. Fix defects with tests and separate commits. Present branch for user review; merge/push only on authorization. No scientific treatment report is produced.

## Self-review and next gate

Task 1 covers validation, local information/FSM and continuation identity; Task 2 covers all public access records, shared runner, event-time distances, budget/censoring and reductions; Task 3 covers all adapters, examples, parity and acceptance. All five Review Focus classes have explicit owner tests. Biological/cultural additions remain in the wider programme: this increment establishes structural usefulness without treating a supplied destination as learned culture.

After plan review execute with the user's standing subagent-driven preference. After implementation review, the next design can add actual resource extraction/delivery or local discovery; those need their own material/information rules and protocol.


## Engineering closure

The three implementation stages and independent reviews are complete. Task 1 was reviewed at
`35da8fd`, Task 2 at `e6f4067`, Task 3 at `eea12eb`, and the whole branch from `f31c14f`
through `eea12eb` was approved. This increment implements checked resource-access core records
and thin native/WASM exports. It does not register or execute a scientific campaign. The programme
pointers and ontology evidence boundaries remain in place.

### Recorded rulings and costs

Ruling: Run public unsupported-fixture end-to-end tests in Task 2 when run_access_episode exists; Task 1 tests validation directly and may stage that test until the entrypoint is introduced — the approved interfaces put public construction in Task 2 — cost if wrong: review must catch any omitted fixture rejection before adapters are built.

Ruling: Permit narrow temporary allow(dead_code) on validate_task and install_goal_state for Task 1; Task 2 must remove both when its public runner consumes them — the approved sequence stages private validation before its production host — cost if wrong: a staged unused helper could survive unnoticed, so Task 2 review explicitly checks removal.

Ruling: Reject unknown coordinate keys inside AccessTask.goal using a private strict deserializer, without changing legacy Pos parsing — new task records promise strict unknown-field rejection while old schemas must remain compatible — cost if wrong: stricter access inputs require callers to remove unused coordinate annotations; legacy inputs stay unaffected.

Ruling: Add seed:u64 as an explicit private run_world parameter instead of a World field — episode assembly needs original seed provenance, which the proposed host signature omitted; public wrappers already have it, and approved spec does not bind that private signature — cost if wrong: private interface differs from plan; exact Explore/full-width seed replay tests must demonstrate unchanged public behavior.

Ruling: Reuse serde.workspace=true directly in sugarscape-cli for the specified typed AccessExportSummary serializer; add CLI Cargo.toml and consequent Cargo.lock edge to Task 3 file map — no new library/version is introduced, but the CLI needs an explicit existing serializer dependency — cost if wrong: one additional direct dependency edge; existing transitive crate versions and runtime scope stay unchanged.

Ruling: Generated AccessEpisode records guarantee config.lab == episode.config; saved access-record ingestion/validation is outside this approved export increment, and the design must explicitly distinguish serde parsing from validation — the approved public interfaces and plan contain no AccessEpisode validator route, while generated equality is constructed and tested — cost if wrong: imported or tampered access records remain unvalidated until a dedicated ingestion increment.

Ruling: Accept final-review declined items 2–4 as outside this engineering increment: (2) biological validity, treatment efficacy and a new ontology-source audit remain scientific work; (3) extraction, embodiment, communication, learning, culture, institutions and Minds/Hornvale integration remain future increments; (4) scientific registration, authorization, execution and inference retain their separate gate — approved scope is checked structural generation/export with evidence boundaries — cost if wrong: empirical adequacy remains unestablished, the future mechanisms remain unimplemented here, and no scientific results are available.


### Boundary RED/GREEN and accepted checks

Task 3 wrote adapter tests before implementation. Native `cargo test -p sugarscape-cli --test
burrow_access` exited 101 with eight missing-command failures. The exclusive-writer unit test
first exited 101 at missing `write_export_file`. Before the WASM adapter existed, `npm run wasm`
exited 0 and regenerated the package, then `npx vitest run src/burrow-access-parity.test.ts`
exited 1: the zero-tick test explicitly reported `burrow_access_replay_json is not a function`,
and rejection cases wrapped that missing-export TypeError. Four parity cases also lacked the
worktree-local native binary; that ENOENT is not missing-export RED evidence. First native GREEN
and local release compilation exited 101 because the CLI lacked a direct serde declaration;
adding existing `serde.workspace=true` and its Cargo.lock edge fixed it. Native GREEN then passed
all eight tests; regenerated WASM and a local release build both exited 0; focused access parity
GREEN passed all 25 tests. Logs are `/tmp/burrow-access-task3-{cli-red,exclusive-red,
wasm-red-build,web-red,cli-green,native-release,cli-green-fixed,wasm-green-build,
native-release-fixed,web-green}.log`.

Native cached workspace checks use `CARGO_TARGET_DIR=/Users/nathan/Projects/ndouglas/SugarScape/target`.
Local native release and web `npm test` explicitly unset that variable, so parity uses this
worktree's `target/release/sugarscape`. Survey has its own unchanged target directory.

| Required command | Exit/result | Log |
| --- | --- | --- |
| `cargo test --workspace` | 0; 2144 passed, 102 existing ignored, zero failed | `/tmp/burrow-access-task3-workspace-tests.log` |
| `cargo test --manifest-path survey/Cargo.toml` | 0; 234 passed, zero failed | `/tmp/burrow-access-task3-survey-tests.log` |
| `cargo fmt --all -- --check` | 0 | `/tmp/burrow-access-task3-workspace-fmt.log` |
| `cargo fmt --manifest-path survey/Cargo.toml -- --check` | 0 | `/tmp/burrow-access-task3-survey-fmt.log` |
| `cargo clippy --all-targets -- -D warnings` | 0 | `/tmp/burrow-access-task3-workspace-clippy.log` |
| `cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings` | 0 | `/tmp/burrow-access-task3-survey-clippy.log` |
| `wasm-pack test --node crates/sugarscape-wasm` | 0; 79 passed | `/tmp/burrow-access-task3-wasm-node.log` |
| `npm test` (web, isolated accepted run) | 0; 1000 passed across 64 files | `/tmp/burrow-access-task3-web-tests-isolated.log` |
| `npm run typecheck` (web) | 0 | `/tmp/burrow-access-task3-web-typecheck.log` |
| `npm run build` (web) | 0; current WASM regenerated, tsc/Vite passed | `/tmp/burrow-access-task3-web-build.log` |

The first full `npm test` exited 1 (995 passed, five timeouts) while independent acceptance jobs
were active. Four unmodified `determinism.test.ts` cases (`vi-5-schelling-25-residence`,
`vi-6-schelling-50-residence`, `nm-2a-universal`, `nbm-probabilistic`) exceeded their 5000ms gate;
`polarity-engine.test.ts` registered overextension trace exceeded its existing 30000ms gate.
There were no reported fingerprint/content mismatches. Log:
`/tmp/burrow-access-task3-web-tests.log`. After every heavy acceptance job finished, the exact full `npm test` was repeated alone and
exited 0: all 1000 tests across 64 files passed in 23.30s. This supports CPU contention as the
earlier timeout cause. No timeout/test/runtime changes were made. Isolated log:
`/tmp/burrow-access-task3-web-tests-isolated.log`. Initial failing wall durations were 5776ms,
5008ms, 5444ms and 5149ms for the four determinism cases respectively and 30792ms for the
overextension case. The isolated run is the accepted full web-suite result.

### Fixed engineering scenes

Using a worktree-local release binary, each command below exited 0 with a new destination.
The two complete outer `episode.json` byte arrays match exactly within each policy. Full stdout,
map inspection, normalized configs, material/access summaries and hashes are retained in
`/tmp/burrow-access-task3-example-acceptance.log` and the Task 3 SDD report.

```bash
target/release/sugarscape burrow-access --config docs/examples/burrow/access-explore.json --seed 7 --ticks 512 --sample-every 32 --out /var/folders/_0/j0_zkq_d3jn0klz033gq33cc0000gn/T/burrow-access-task3-acceptance-fwzfq0zr/explore-a
target/release/sugarscape burrow-access --config docs/examples/burrow/access-explore.json --seed 7 --ticks 512 --sample-every 32 --out /var/folders/_0/j0_zkq_d3jn0klz033gq33cc0000gn/T/burrow-access-task3-acceptance-fwzfq0zr/explore-b
target/release/sugarscape burrow-access --config docs/examples/burrow/access-known-goal.json --seed 7 --ticks 512 --sample-every 32 --out /var/folders/_0/j0_zkq_d3jn0klz033gq33cc0000gn/T/burrow-access-task3-acceptance-fwzfq0zr/known-goal-a
target/release/sugarscape burrow-access --config docs/examples/burrow/access-known-goal.json --seed 7 --ticks 512 --sample-every 32 --out /var/folders/_0/j0_zkq_d3jn0klz033gq33cc0000gn/T/burrow-access-task3-acceptance-fwzfq0zr/known-goal-b
```

| Policy | Complete outer bytes / SHA-256 | Descriptive endpoint |
| --- | --- | --- |
| Explore | 1210796; `a19837474e3c63124576ca028b9c87284235e2fdf657fce1c31f035b9694d871` | 4096 opportunities; inaccessible/censored, null first access/distance; 10 digs, 4 disposed, 6 carried, 0 loose |
| KnownGoal | 1575128; `9553bd7cfa23468b2ff070d99fc284cb85fcb77d176ee085d9559f386d6f3644` | 4096 opportunities; first access opportunity 500, tick 62, shortest exit distance 7; final distance 7; 125 digs, 124 disposed, 1 carried, 0 loose |

Both initial maps retain `#` at `(7,12)` and all sampled maps preserve base glyphs. Metadata reports
the coordinate/policy and structural endpoint. KnownGoal's first local completion observations
occur after the first access action and are recorded per worker, with eight bounded entries;
Explore's task diagnostics are empty/zero. Summary has exactly `base` and `access` objects.
These outcomes are descriptive scene records, with no preferred acceptance success criterion,
scientific-seed execution, campaign registration, inference or tuning. Existing legacy CLI tests,
web Burrow parity tests and WASM regression tests were left unchanged.


### Execution tracker and review gate

All three task reviews and the whole-branch review are approved: Tasks 1, 2 and 3 at their recorded
commits, plus the full `f31c14f..eea12eb` review. The generated/saved-record validation wording is
clarified in the design. Existing WASM packaging repository/license metadata INFO notices remain
nonblocking. The accepted checks and counts, including the complete failed-run and isolated-rerun
evidence above, are unchanged. No merge, push or scientific execution occurred during implementation/review; user-authorized integration into `main` subsequently completed at `6076aed`.


### Verified integration into main

The user authorized local merge on 2026-10-04. `main` had advanced to `82753fb` with Democratic Peace publication/integration work. `git pull --ff-only` reported up to date; `git merge --no-edit burrow-resource-access` merged reviewed head `cbfbe5e` without conflicts at `6076aed`. The CLI and WASM auto-merge hunks preserve both lines of work.

Fresh checks on the merged runtime exited zero:

| Check | Result | Evidence |
|---|---|---|
| `cargo test --workspace` | 2205 passed, 0 failed, 103 existing ignored | `/tmp/burrow-access-merge-workspace.log` |
| `cargo test --manifest-path survey/Cargo.toml` | 248 passed, 0 failed | `/tmp/burrow-access-merge-survey.log` |
| `cargo fmt --all -- --check` | passed | direct command output, empty |
| `cargo fmt --manifest-path survey/Cargo.toml -- --check` | passed | direct command output, empty |
| `npm run build` in web | fresh merged WASM, type checking and Vite build passed | `/tmp/burrow-access-merge-web-build.log` |
| `npm test` in web | 1030 passed across 70 files | `/tmp/burrow-access-merge-web-tests.log` |

Heavy checks ran sequentially to avoid the contention observed during implementation. The post-merge bookkeeping changes only documentation; the verified runtime is unchanged. No push or Burrow scientific campaign was performed. The resource-access branch/worktree can be removed after this verified integration; other sessions' worktrees remain outside cleanup scope.
