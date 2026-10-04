# Burrow 2 Resource Access Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Measure structural access to a supplied resource coordinate and compare legacy exploration with a locally applied known-goal heuristic.

**Architecture:** Compose an AccessConfig around the existing LabConfig and reuse the checked physical world. KnownGoal adds private per-worker completion memory through one optional controller-weight seam; Explore remains the legacy world with an external observer. Record event-time access in the shared runner and expose separate native/WASM adapters.

**Tech Stack:** Rust, serde/serde_json, existing clap and wasm-bindgen, TypeScript/Vitest; no new dependency.

**Spec:** [approved resource-access design](../specs/2026-10-04-burrow-2-resource-access-design.md).

## Global Constraints

- Status: implementation plan awaiting review; design approved 2026-10-04. This builds engineering examples; it does not define or execute a scientific campaign.
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

- [ ] **Step 1: Write tests for validation, ticket weights, memory and compatibility.** Construct the private setup exit `(0,1)`, open `(0,1),(1,1)`, worker `(1,1)`, only diggable goal `(2,1)`. Reject out-of-bounds, open and nondiggable goals, zero weight, cue×goal product overflow and unsupported public choice/corridor. Enumerate each ticket for weights `[3,1]`, checking the three improving tickets and one nonimproving ticket. Test a retained target stays retained and congested movement waits. Assert goal completion stays false for a solid goal, latches only when locally open, latches while loaded without overriding disposal, and stays false for a remote worker.

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

- [ ] **Step 2: Run RED.** `cargo test -p sugarscape-core burrow::tests::access` must fail at absent access types/seam.

- [ ] **Step 3: Implement the narrow seam and continuation state.** Validate actual setup with checked coordinates and weight products. Keep per-face weights u32 by checked cue×goal multiplication; sum candidate tickets in u64 with a checked cell-count bound. Distances use u64 sums of abs_diff. The factor is supplied heuristic only:

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

- [ ] **Step 4: Run GREEN.** `cargo test -p sugarscape-core burrow::` and workspace fmt. Verify ticket factors, local memory, transport priorities and unchanged legacy fingerprints. Check the previously implemented saved physical validator still accepts ordinary seed-7 episodes.

- [ ] **Step 5: Commit.** Stage the listed core files and execution tracker; commit `feat(burrow): add checked private goal guidance to local frontier selection` after core tests and lint pass. Review this task before adding runner exports.

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

- [ ] **Step 1: Write exact milestone, censoring and reductions tests.** On the private three-cell setup apply one legal dig and assert opportunity 1, tick at action clock, worker 0, distance 2, digs 1, carried 1, no other material. Full hands/blocked/far digs and waits never fabricate access. Construct a longer open detour and a later shortcut: first distance stays frozen while final distance decreases. Test last-action access succeeds, no access yields null milestone/distance and deadline_censored, zero ticks observes zero opportunities, and full budget continues after completion. Explore compares every Episode field and serialized bytes with legacy; KnownGoal weight 1 compares events/choices with Explore.

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

- [ ] **Step 2: Run RED.** `cargo test -p sugarscape-core burrow::tests::access` must fail at `run_access_episode`/observer records before runner implementation.

- [ ] **Step 3: Refactor shared hosting and observe after committed actions.** Extract existing `run_episode` initialization/sampling/assembly into `run_world` without changing sampling conditions, stop reasons, ledger finish or storage calculations. Preserve the existing action loop, allowing an external observer in its internal host path. The access runner validates growing fixture and task before stepping, installs GoalState for KnownGoal, creates the observer for both policies, and calls run_world. Reuse current exit topology field after each recorded transaction; no observer search touches the base BFS counters.

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

- [ ] **Step 4: Run GREEN.** `cargo test -p sugarscape-core burrow::` and formatting; run all core tests after shared runner changes. Confirm fixed-seed reductions and exact legacy bytes, censored/last-action semantics and historical distance. No success-rate threshold is an engineering acceptance gate.

- [ ] **Step 5: Commit.** Stage listed core files and tracker; commit `feat(burrow): record event-time resource access without early stopping` after tests/lint and task review.

### Task 3: Checked native/WASM exports and fixed-seed acceptance

**Files:**
- Create: `crates/sugarscape-cli/src/burrow_access.rs`, `crates/sugarscape-cli/tests/burrow_access.rs`.
- Modify: `crates/sugarscape-cli/src/main.rs`, `crates/sugarscape-cli/src/burrow.rs` (small shared exclusive export helpers only).
- Modify: `crates/sugarscape-wasm/src/lib.rs`.
- Create: `docs/examples/burrow/access-explore.json`, `docs/examples/burrow/access-known-goal.json`, `web/src/burrow-access-parity.test.ts`.
- Modify documentation: `docs/burrow.md`, `docs/superpowers/specs/2026-10-04-burrow-2-resource-access-design.md`, this plan.

**Interfaces:**
- CLI `sugarscape burrow-access --config FILE --seed U64 --ticks U32 --sample-every U32 --out DIR`; defaults seed 7/ticks 512/sample_every 32, config/out required. Existing Failure conversions give validation code 2/I/O code 1.
- WASM `burrow_access_replay_json(config_json:&str,seed:&str,ticks:f64,sample_every:f64)->Result<String,JsValue>` returns core AccessEpisode JSON.
- Native summary schema `AccessExportSummary { base: Snapshot, access: AccessSummary }`, serialize only in adapter; normalized outer config.json, outer episode.json, maps.txt and summary.json. Maps retain base glyphs; heading reports supplied task coordinate/policy and structural access metadata.

- [ ] **Step 1: Write CLI preservation/numeric/parity tests first.** CLI: both policies and maximum u64 seed round-trip normalized config, full outer Episode and summary; invalid task creates no outputs; existing nonempty/file out paths remain untouched; race/exclusive-write failure reports partial path; zero ticks works. WASM: malformed JSON/contextual fields, unsupported fixture, invalid goal, strict seed failures and ticks/sample values 1.5/-1/NaN/±Infinity/4294967296 reject before u32 conversion; sample zero rejects, ticks zero accepts. Full-record parity matrix two policies × seeds 7 and 18446744073709551615, ticks 16/sample 4.

```typescript
it.each([1.5, -1, NaN, Infinity, -Infinity, 4294967296])(
  'rejects invalid original ticks %s', value => {
    expect(() => burrow_access_replay_json(configText, '7', value, 4)).toThrow();
  },
);
```

`configText` is loaded from `access-known-goal.json` with the existing node:fs/fileURLToPath/initSync pattern in `web/src/burrow-parity.test.ts`; use existing node shims and string paths, no new node:path import. Also assert the parsed thrown field is `ticks`, and run the equivalent table for sampling.

- [ ] **Step 2: Run RED.** `cargo test -p sugarscape-cli --test burrow_access` must fail before command exists. Add the parity test before export implementation, then `npm run wasm` and `npx vitest run src/burrow-access-parity.test.ts` from web must fail at missing export. Do not treat a stale generated WASM package as implementation failure.

- [ ] **Step 3: Implement thin checked adapters and examples.** Reuse existing decimal_seed, checked_burrow_u32 and field_errors at the WASM boundary:

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

- [ ] **Step 4: Verify and record acceptance.** Run `cargo test --workspace`, `cargo test --manifest-path survey/Cargo.toml`, `cargo fmt --all -- --check`, `cargo fmt --manifest-path survey/Cargo.toml -- --check`, `cargo clippy --all-targets -- -D warnings`, `cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings` and `wasm-pack test --node crates/sugarscape-wasm`. From web run `npm test`, `npm run typecheck`, `npm run build` after regenerated WASM; inspect complete parity fields for both policies/full-width seeds. Run both 512-tick seed-7 example exports twice into new temporary directories and compare outer Episode JSON byte-for-byte within each policy. Inspect maps and material/access summaries without a preferred result or required success. Verify legacy CLI and WASM regression tests remain unchanged and pass. Update docs with checked commands, structural definition, supplied-information caveat, censoring, private latch and separate task storage. Record actual evidence and rulings here; remove completed execution tracker.

- [ ] **Step 5: Commit and final review.** Commit `feat(burrow): expose checked resource-access replays across native and wasm` with listed adapter/tests/examples/docs. Fresh whole-branch reviewer checks local information, no duplicate controller loop, optional hashing compatibility, event-time measurements, numeric boundary checks and regression evidence. Fix defects with tests and separate commits. Present branch for user review; merge/push only on authorization. No scientific treatment report is produced.

## Self-review and next gate

Task 1 covers validation, local information/FSM and continuation identity; Task 2 covers all public access records, shared runner, event-time distances, budget/censoring and reductions; Task 3 covers all adapters, examples, parity and acceptance. All five Review Focus classes have explicit owner tests. Biological/cultural additions remain in the wider programme: this increment establishes structural usefulness without treating a supplied destination as learned culture.

After plan review execute with the user's standing subagent-driven preference. After implementation review, the next design can add actual resource extraction/delivery or local discovery; those need their own material/information rules and protocol.
