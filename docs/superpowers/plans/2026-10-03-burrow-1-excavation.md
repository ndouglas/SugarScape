# Burrow 1 Excavation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user has already selected that method. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** deliver a reproducible excavation lab with explicit spoil transport, local cue treatments and an inspectable CLI replay.

**Architecture:** add a standalone `burrow` core module and a CLI subcommand, leaving existing model dispatch and Minds actions intact. Share one checked replay function between native CLI and a small WASM export. Authoritative state, controller observations and researcher accounting remain separate.

**Tech Stack:** existing Rust workspace, Serde/serde_json, Clap, rand/rand_pcg, wasm-bindgen and Vitest; no new dependencies.

**Spec:** [approved Burrow 1 design](../specs/2026-10-03-burrow-1-excavation-design.md).

**Status:** Stages 1–3 implemented and independently reviewed; Stage 4 adapters, acceptance and repository checks complete. Whole-branch review remains pending. Design approved by the user on 2026-10-03. Implementation method is subagent-driven; implementation completion authorizes demonstrations and correctness checks, not judged scientific runs.

## Global Constraints

- “Finite, bounded, horizontal four-neighbor lattice; no wraparound, diagonals or gravity.”
- “Each worker carries zero or one unit. Digging with full hands is illegal.”
- “Workers have stable IDs and occupy open cells, at most two per cell.”
- “Excavation timestamps do not reset on drop.”
- “Blocked attempts and waits also consume one, and are logged separately.”
- “Unloaded workers do not receive global frontier targets.”
- “Cue blindness removes only the influence on frontier selection, not the ability to see and transport loose spoil.”
- “No controller has a desired room count, prescribed nest shape or global construction plan.”
- “No new dependency is required by this design.”
- No `ModelKind`, web controls, survey claims, Hornvale modifications or changes to existing default trajectories.
- Demonstration defaults: 41 by 25, exit `(0,12)`, open `x=0..2,y=10..14`, eight workers, 32-tick freshness, three-move relay legs, cue weight three, minimum two recent units, 512 requested ticks.
- Choice fixture: 9 by 7, open `(2,3)..(6,3)`, worker `(4,3)`, only `(1,3)` and `(7,3)` diggable, four/one recent or old units at an approach, starting clock twice freshness window.
- One seeded worker permutation per tick; sequential decisions/commits; one opportunity per worker.
- Unit and replay checks establish correctness. Preference rules are supplied; no biological validation or comparative success claim follows from implementing them.

## Review Focus

1. Configuration can have an open spawn disconnected from the exit or multiply overflowed dimensions; reject before allocation (Task 1).
2. Two workers can compete for the same unit or occupy a formerly available cell; resolve against committed state without duplicating material (Tasks 1/3).
3. A dropped unit can look fresh indefinitely if its clock resets, and cue response can leak through blind pickup; preserve original creation time and isolate selection effects (Tasks 1/2).
4. Requested runs can end with carried/loose units or an early-ended choice fixture; report censored deliveries and actual opportunities (Task 3).
5. Large seeds and output paths can be misread or overwritten; preserve full u64 seed precision and refuse nonempty output directories (Task 4).

## Files and execution staging

Create `crates/sugarscape-core/src/burrow/` with `mod.rs`, `config.rs`, `state.rs`, `fixtures.rs`,
`actions.rs`, `observation.rs`, `controller.rs`, `ledger.rs`, `runner.rs`, `view.rs` and unit tests
under `tests/`. `mod.rs` reexports only the public lab contract; internal mutable state stays private.
Modify `crates/sugarscape-core/src/lib.rs` to expose the module. Add CLI handling in
`crates/sugarscape-cli/src/burrow.rs`, register it in `src/main.rs`, and add
`crates/sugarscape-cli/tests/burrow.rs`. Add the WASM export in its existing `src/lib.rs` and a
focused `web/src/burrow-parity.test.ts` test. Add demonstration configurations under
`docs/examples/burrow/` and usage under `docs/burrow.md`.

At execution start use the worktree skill, create an isolated worktree under
`~/.config/superpowers/worktrees/SugarScape/`, and create root `IMPLEMENTATION_PLAN.md` with the
four stage blocks below, using the user's Goal/Success Criteria/Tests/Status format. Update it
with each task commit; remove it after all four stages and final review complete. Preserve this
detailed plan. Tasks are sequential because their interfaces depend on previous stages. Use a
fresh implementer and fresh reviewers per the subagent-driven skill; do not dispatch concurrent
mutations to shared core/CLI/WASM seams.

## Stage 1: checked world and material actions

**Goal:** construct checked fixtures and resolve paid physical actions with material conservation.

**Success Criteria:** exact action examples, rejected configurations and invariants pass; the new module compiles independently of controllers.

**Tests:** `cargo test -p sugarscape-core burrow::tests::material`.

**Status:** Complete

### Task 1: configuration, fixtures, state and action resolution

**Files:** create `mod.rs`, `config.rs`, `state.rs`, `fixtures.rs`, `actions.rs`,
`tests/mod.rs`, `tests/material.rs`; modify core `src/lib.rs` and root tracker.

**Interfaces:** define the following contract, deriving Clone/Debug/PartialEq and Serde for
configuration and exported records, Eq for integer-only state. Deny unknown configuration fields.
Coordinates use the lab's bounded `Pos`, not the torus `geometry::Pos`.

```rust
pub struct Pos { pub x: u32, pub y: u32 }
pub enum Transport { Direct, Relay }
pub enum Cue { Blind, Responsive }
pub enum Side { Left, Right }
pub enum Pile { FreshAccumulation, OldAccumulation, SingleFresh }
pub enum Fixture {
    Growing { width: u32, height: u32, workers: u32 },
    Choice { side: Side, pile: Pile },
    Corridor { length: u32, workers: u32 },
}
pub struct LabConfig {
    pub fixture: Fixture, pub transport: Transport, pub cue: Cue,
    pub freshness_window: u64, pub relay_distance: u32,
    pub response_weight: u32, pub minimum_recent_units: u32,
}
pub struct InitialSpoil { pub pos: Pos, pub born: u64 }
pub struct Setup {
    pub width: u32, pub height: u32, pub exit: Pos, pub start_tick: u64,
    pub open: Vec<Pos>, pub diggable: Vec<Pos>,
    pub workers: Vec<Pos>, pub spoil: Vec<InitialSpoil>,
}
pub enum Action { Move(Pos), Dig(Pos), Pickup, Drop, Dispose, Wait }
pub enum Outcome { Success, Blocked { reason: String } }
pub struct ActionEvent {
    pub tick: u64, pub worker: u32, pub action: Action,
    pub outcome: Outcome, pub material: Option<u64>, pub from: Pos, pub to: Pos,
}
pub struct Inventory { pub initial: u64, pub excavated: u64, pub carried: u64,
                       pub loose: u64, pub disposed: u64 }
pub(crate) struct Worker {
    pub id: u32, pub pos: Pos, pub carried: Option<u64>,
    pub target: Option<Pos>, pub loaded_moves: u32,
}
pub(crate) enum UnitLocation { Loose(Pos), Carried(u32), Disposed }
pub(crate) struct Unit { pub id: u64, pub born: u64, pub location: UnitLocation }
pub struct World {
    config: LabConfig, setup: Setup, tick: u64,
    open: Vec<bool>, diggable: Vec<bool>, workers: Vec<Worker>,
    units: std::collections::BTreeMap<u64, Unit>,
    next_material: u64, excavated: u64, rng: crate::rng::SimRng,
}
impl World {
    pub fn new(config: LabConfig, seed: u64) -> Result<Self, Vec<FieldError>>;
    // Internal checked constructor for hand-worked unit tests, not an unchecked bypass.
    pub(crate) fn from_setup(config: LabConfig, setup: Setup, seed: u64)
        -> Result<Self, Vec<FieldError>>;
    pub(crate) fn apply(&mut self, worker: u32, action: Action) -> ActionEvent;
    pub fn inventory(&self) -> Inventory;
    pub fn check_invariants(&self) -> Result<(), String>;
}
```

`FieldError` is the existing `crate::config::FieldError`. Store units by stable monotonic ID;
each unit has original creation time and exactly one location: loose cell, worker, or disposed.
Workers store position, optional carried ID, optional frontier target and successful loaded moves.
Use stable vectors/BTreeMap and fixed N,S,E,W neighbor order. A failed action may update action
accounting but cannot update material or geometry. Invalid worker IDs yield contextual blocked
records rather than indexing panics. Pickup selects the smallest loose ID on the current cell.

`fixtures::corridor_setup(length, workers)` builds a width `length+2`, height-three map, exit
`(0,1)`, open `(0,1)..(length-1,1)`, one diggable face `(length,1)`, and workers placed from the
deep end toward the exit. Growing spawns use distinct row-major non-exit cells on their first
pass, then a second occupancy pass if needed. Choice units use the spec's initial clock.

- [x] **Step 1: write exact failing action/configuration tests.**

```rust
#[test]
fn dig_creates_one_unit_and_full_hands_cannot_dig_again() {
    let mut c = LabConfig::default();
    c.fixture = Fixture::Corridor { length: 7, workers: 1 };
    let mut setup = fixtures::corridor_setup(7, 1).unwrap();
    setup.diggable.push(Pos { x: 6, y: 0 });
    let mut w = World::from_setup(c, setup, 7).unwrap();
    assert_eq!(w.apply(0, Action::Dig(Pos { x: 7, y: 1 })).outcome, Outcome::Success);
    let before = w.inventory();
    let rejected = w.apply(0, Action::Dig(Pos { x: 6, y: 0 }));
    assert!(matches!(rejected.outcome,
                     Outcome::Blocked { reason } if reason.contains("full hands")));
    assert_eq!(w.inventory(), before);
    assert_eq!(before.carried, 1);
    w.check_invariants().unwrap();
}
```

Add constructed tests for drop/pickup identity, preserved birth tick, smallest-ID pickup,
disposal only at exit, non-neighbor movement/digging, array edges, full destination, unknown worker,
two successive pickups competing for one unit, non-diggable solid faces, disconnected open spawns,
duplicate open/spawn constraints, empty dimensions, zero parameters, future-born spoil and overflow.
Use `from_setup` and explicit coordinates rather than seed sweeps.

- [x] **Step 2:** run the stage test command; observe missing-module/API failures.
- [x] **Step 3: implement checked construction and transactions.** Validate before allocating.
  Check products/additions/weight sums/timestamp arithmetic; verify open-spawn/exit connectivity by
  BFS. Proposed operational bounds for this small lab are 262,144 cells and 4,096 workers; errors
  state those limits. These bounds are implementation choices for plan review, not biological claims.

```rust
// Conservation must also include researcher-seeded fixture material.
let i = self.inventory();
if i.initial.checked_add(i.excavated)
    != i.carried.checked_add(i.loose).and_then(|n| n.checked_add(i.disposed)) {
    return Err("burrow material conservation failed".into());
}
```

- [x] **Step 4:** run the exact tests, core formatting and stage-relevant linting; self-review
  occupancy, connected growth and mutation ordering. Do not introduce dummy controller hooks.
- [x] **Step 5:** update the tracker and commit working stage code with
  `feat(burrow): add checked excavation and conserved material actions`.

## Stage 2: local observations and controllers

**Goal:** implement the four transport/cue combinations using only declared observations.

**Success Criteria:** occlusion, selection weights, freshness and relay-leg rules pass exact tests.

**Tests:** `cargo test -p sugarscape-core burrow::tests::controller`.

**Status:** Complete

### Task 2: observation boundaries, navigation and decisions

**Files:** create `observation.rs`, `controller.rs`, `tests/controller.rs`; modify module reexports
and tracker. Consume Task 1's checked state, actions and configurations.

**Interfaces:** internal decision APIs consume owned observations so immutable world reads and
mutable RNG borrows do not overlap. `SimRng` is `crate::rng::SimRng`.

```rust
pub struct ObservedCell { pub pos: Pos, pub occupants: u32,
                          pub loose: u32, pub recent: u32 }
pub struct Observation { pub origin: Pos, pub open: Vec<ObservedCell>,
                         pub frontier: Vec<Pos> }
pub struct ExitNeighbor { pub pos: Pos, pub distance: u32, pub occupants: u32 }
pub struct WorkerView { pub id: u32, pub pos: Pos, pub carrying: bool,
                        pub target: Option<Pos>, pub loaded_moves: u32 }
pub struct Decision { pub action: Action, pub target: Option<Pos>,
                      pub selected: Option<Pos> }
pub(crate) fn observe(world: &World, worker: u32) -> Observation;
pub(crate) fn exit_distances(world: &World) -> Vec<Option<u32>>;
pub(crate) fn decide(o: &Observation, w: &WorkerView, c: &LabConfig,
    at_exit: bool, outward: &[ExitNeighbor], rng: &mut SimRng) -> Decision;
pub(crate) fn target_weight(c: &LabConfig, recent_units: u32) -> u32;
pub(crate) fn select_ticket(weights: &[u32], ticket: u64) -> usize;
```

Build observations by BFS through open cells up to two hops; expose adjoining diggable faces,
not substrate beyond them. Sort/deduplicate targets. Count each unit once per target on its
observed open approach cells. All route searches for unloaded agents use that observation.
The loaded exit BFS is a supplied global scaffold; pass only available descending neighbors to
the controller. Count BFS calls, visited cells and peak queue size as deterministic compute proxies.

- [x] **Step 1: write failing local-decision tests.**

```rust
#[test]
fn freshness_excludes_the_boundary_and_blind_weights_ignore_counts() {
    assert!(is_recent(31, 0, 32));
    assert!(!is_recent(32, 0, 32));
    let mut c = LabConfig::default();
    c.cue = Cue::Blind;
    assert_eq!(target_weight(&c, 4), 1);
    c.cue = Cue::Responsive;
    assert_eq!(target_weight(&c, 1), 1);
    assert_eq!(target_weight(&c, 2), 3);
}
```

Define internal `is_recent(now: u64, born: u64, window: u64) -> bool` in observation code.
Enumerate all four tickets for weights `[3,1]`, asserting three select index zero; enumerate
mirrored `[1,3]`, asserting three select index one. This checks exact mapping, not sampled fit.
Construct tests for hidden piles/targets behind solid cells, shared approach counts, equal-response
weight reduction, target removal by another worker, blind pickup retained, disposal priority,
drop after three successful moves, no leg advance on waits, and full descending neighbor cells.

- [x] **Step 2:** run the stage command and verify missing-function/behavior failures.
- [x] **Step 3: implement the spec's state machine and integer weighted selection.**

```rust
fn is_recent(now: u64, born: u64, window: u64) -> bool {
    now.checked_sub(born).is_some_and(|age| age < window)
}
fn target_weight(c: &LabConfig, count: u32) -> u32 {
    if c.cue == Cue::Responsive && count >= c.minimum_recent_units {
        c.response_weight
    } else { 1 }
}
```

Pickup uses one unbiased half-probability draw only when loose material exists. Frontier movement
uses an observed shortest path with legal occupancy; wait if a valid selected target has no
available next step. Random wandering applies only without a target. Direct transport never
internally drops. No learned roles, private inventories of other agents or global frontier scans.

- [x] **Step 4:** run Tasks 1/2 tests together and check cue-only treatment differences; verify
  changing observation or cue flags never bypasses physical action legality.
- [x] **Step 5:** update tracker and commit
  `feat(burrow): add local spoil cues and direct or relay controllers`.

## Stage 3: deterministic episodes and accounting

**Goal:** execute sequential worker opportunities and export complete, honest replay records.

**Success Criteria:** full repeatability, conserved per-action state and censored-delivery accounting.

**Tests:** `cargo test -p sugarscape-core burrow::tests::runner`.

**Status:** Complete

### Task 3: scheduling, ledger, summaries and sampled maps

**Files:** create `ledger.rs`, `runner.rs`, `view.rs`, `tests/runner.rs`; extend state/mod and tracker.
Consume Tasks 1/2's actions and controller contract. Do not add a survey verdict.

**Interfaces:** export these public records through `burrow::mod.rs`.

```rust
pub struct RunOptions { pub ticks: u32, pub sample_every: u32 }
pub struct Delivery {
    pub material: u64, pub born: u64, pub disposed_at: Option<u64>,
    pub carriers: Vec<u32>, pub carried_moves: u64, pub waiting_ticks: u64,
}
pub struct Snapshot {
    pub tick: u64, pub opportunities: u64, pub inventory: Inventory,
    pub successful_moves: u64, pub digs: u64, pub pickups: u64, pub drops: u64,
    pub disposals: u64, pub blocked: u64, pub waits: u64,
    pub exit_bfs_calls: u64, pub exit_bfs_visits: u64, pub exit_bfs_peak_queue: u64,
    pub connected_open: u64,
}
pub struct Frame { pub tick: u64, pub fingerprint: String, pub ascii: String }
pub struct ChoiceEvent { pub tick: u64, pub worker: u32, pub target: Pos }
pub struct WorkerWork { pub worker: u32, pub moves: u64, pub digs: u64,
                        pub pickups: u64, pub drops: u64, pub disposed: u64,
                        pub blocked: u64, pub waits: u64 }
pub struct Episode {
    pub config: LabConfig, pub setup: Setup, pub seed: String,
    pub requested_ticks: u32, pub completed_ticks: u32, pub stop_reason: String,
    pub events: Vec<ActionEvent>, pub choices: Vec<ChoiceEvent>,
    pub frames: Vec<Frame>, pub series: Vec<Snapshot>,
    pub deliveries: Vec<Delivery>, pub worker_work: Vec<WorkerWork>,
    pub final_summary: Snapshot,
}
pub fn run_episode(c: LabConfig, seed: u64, options: RunOptions)
    -> Result<Episode, Vec<FieldError>>;
impl World { pub fn step(&mut self); pub fn fingerprint(&self) -> u64; }
```

Define `is_recent` and selection helpers only once; unit tests import their owning module.
Serializing decimal seed strings prevents loss of u64 precision in JavaScript. Compute excavation
and disposal rates from integer totals; zero opportunities produce unavailable/null rates in
presentation rather than divide-by-zero. Snapshot integer fields remain the authoritative totals.

Also retain spatial dig counts by coordinate, unique cells worked per worker, successful loaded
and unloaded travel, per-unit delivery age, and work-site distance from the exit at each dig.
Record that distance when the event occurs; later shortcuts must not rewrite it. Include these
in serialized episode diagnostics as `SpatialWork` records with `pos: Pos`, `digs: u64` and
`workers: Vec<u32>`, `LoadedTravel` totals with `loaded: u64` and `unloaded: u64`, and
`DigDistance` records with `tick: u64`, `worker: u32`, `pos: Pos`, `distance: u32`.
Add `spatial_work: Vec<SpatialWork>`, `travel: LoadedTravel`, and `dig_distances: Vec<DigDistance>`
to `Episode`; derive the other requested distributions from those records and unit histories.
Report deterministic retained record counts and peak storage sizes alongside route-field work;
wall-clock profiling is separate from canonical replay.

- [x] **Step 1: write failing replay/accounting tests.**

```rust
#[test]
fn identical_seed_and_config_reproduce_the_entire_record() {
    let opts = RunOptions { ticks: 16, sample_every: 4 };
    let a = run_episode(LabConfig::default(), 7, opts.clone()).unwrap();
    let b = run_episode(LabConfig::default(), 7, opts).unwrap();
    assert_eq!(serde_json::to_string(&a).unwrap(), serde_json::to_string(&b).unwrap());
}
#[test]
fn zero_ticks_still_exports_the_starting_frame() {
    let e = run_episode(LabConfig::default(), 7,
        RunOptions { ticks: 0, sample_every: 4 }).unwrap();
    assert_eq!(e.final_summary.opportunities, 0);
    assert_eq!(e.frames.len(), 1);
}
```

Add exact scripted-action ledger tests for two carriers, loose waiting, undelivered/carrying units,
and fixture-initialized old units. Define `ledger::record(&ActionEvent, &World)` and
`ledger::finish(&World) -> Vec<Delivery>`; use ordered deduplicated carrier IDs, count carried
moves per successful material-bearing move, and sum loose intervals including final censoring.
Initial old material begins its observed loose-wait interval at fixture start, not at its earlier
creation timestamp; preserve that earlier timestamp for sensory age and identify prior time as
supplied history. Add a named exact test for that distinction.
Test that the choice fixture stops on selection before an action commits; it has zero opportunities
when its initial decision selects immediately, despite a nonzero requested budget.
Test every worker receives one opportunity in a completed growing tick, sample cadence does not
change events/fingerprints, terminal frames are present, and blocked opportunities count once.

- [x] **Step 2:** run stage tests; observe unavailable runner/ledger failures.
- [x] **Step 3: implement scheduling, complete accounting and bounded replay.**

```rust
// One fresh permutation per tick; all decisions use earlier committed state.
use rand::seq::SliceRandom;
let mut order: Vec<u32> = self.worker_ids();
order.shuffle(&mut self.rng);
for id in order { self.worker_opportunity(id); }
```

`worker_ids`, `worker_opportunity` are private methods added in this task. Update the selected
target before applying its action; preserve it unless invalidated by the observed world.
The choice fixture logs selection then stops before commit. Rebuild exit distances after topology
changes, caching only while geometry is unchanged; occupancy filtering remains per opportunity.
Record deterministic BFS work, not wall-clock timing in canonical replay.

Fingerprint canonical authoritative state, controller state, configuration, clock and RNG
continuation. Follow existing explicit integer-byte hashing conventions; include continuation
words drawn from a cloned RNG, leaving live RNG unchanged. Maps and diagnostics must not consume
randomness. Use 16-digit hexadecimal fingerprint strings. Preserve final frames even when the
last tick is off cadence. ASCII legend: `#` solid, `.` open, `E` exit, `o` loose material, `w`
worker, `W` loaded worker; exit/worker overlay takes precedence and legend notes multiple occupants.
Trace and inventory preserve details hidden by those glyphs.

Validate nonzero sampling and checked requested clock advance. Proposed cap: one million worker
opportunities per replay; reject larger products before reserving trace vectors. This is an
operational lab bound for plan review. Emit source/assumption labels and actual completion reason.
No chamber extractor, success classification or inferred individual learning.

- [x] **Step 4:** run all core burrow tests and review the conservation equation after every
  scripted transition. Verify diagnostic sampling cannot alter trajectories.
- [x] **Step 5:** update tracker and commit
  `feat(burrow): record deterministic excavation episodes and material histories`.

## Stage 4: CLI acceptance scene and native WASM parity

**Goal:** expose checked replay to users and prove the same episode survives the platform boundary.

**Success Criteria:** all four treatment configs replay; exports agree; CLI errors and full-record parity pass.

**Tests:** core/CLI workspace tests, focused Vitest native/WASM parity and repository formatting/lints.

**Status:** In Progress

Implementation and all runtime checks are complete; per-task and whole-branch reviews remain
pending with the controller. Step 5 and tracker removal follow those reviews.

### Task 4: checked boundaries, example configurations and final review

**Files:** create CLI `src/burrow.rs`, `tests/burrow.rs`, `web/src/burrow-parity.test.ts`,
`docs/burrow.md`, and four JSON files under `docs/examples/burrow/` named
`direct-blind.json`, `direct-responsive.json`, `relay-blind.json`, `relay-responsive.json`.
Modify CLI main, WASM lib, programme/spec status and tracker. No generated WASM assets committed.

**Interfaces:** CLI command `sugarscape burrow --config FILE --seed U64 --ticks U32
--sample-every U32 --out DIRECTORY`. Defaults: seed 7, ticks 512, sample interval 32; config and
output are required. Core validates the request. Export `config.json`, `episode.json`, `maps.txt`
and `summary.json`; display final map and concise integer summary on stdout.

```rust
#[derive(clap::Args, Debug)]
pub struct BurrowArgs {
    #[arg(long)] pub config: std::path::PathBuf,
    #[arg(long, default_value_t = 7)] pub seed: u64,
    #[arg(long, default_value_t = 512)] pub ticks: u32,
    #[arg(long, default_value_t = 32)] pub sample_every: u32,
    #[arg(long)] pub out: std::path::PathBuf,
}
// WASM uses the existing strict decimal_seed and field_errors helpers.
#[wasm_bindgen]
pub fn burrow_replay_json(config_json: &str, seed: &str,
    ticks: f64, sample_every: f64) -> Result<String, JsValue>;
```

Final-review amendment: the WASM adapter accepts the original JavaScript numbers and requires
finite integers in `0..=u32::MAX` for ticks and `1..=u32::MAX` for sampling before converting to
core `RunOptions`. Invalid values return contextual `field_errors`; core opportunity and ASCII
limits still apply. This prevents the original `u32` ABI from silently truncating or wrapping inputs.

Validate config before writing; create absent output directories, accept empty directories, reject
nonempty directories and paths that are files. Return existing I/O failure code 1 and validation
code 2. Report any partial output if a later write fails; do not silently present an incomplete
directory as successful. Use structured JSON functions and fixed filenames, not shell interpolation.

- [x] **Step 1: add failing CLI and parity checks.** CLI tests use the existing binary-env and
  scratch-directory conventions; construct JSON with `serde_json::to_string(LabConfig::default())`.

```rust
let out = std::process::Command::new(env!("CARGO_BIN_EXE_sugarscape"))
    .args(["burrow", "--config", config_path.to_str().unwrap(),
           "--seed", "18446744073709551615", "--ticks", "8",
           "--sample-every", "4", "--out", out_path.to_str().unwrap()])
    .output().unwrap();
assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
let e: Episode = serde_json::from_str(
    &std::fs::read_to_string(out_path.join("episode.json")).unwrap()).unwrap();
assert_eq!(e.seed, "18446744073709551615");
```

Add invalid-config and zero-sampling code-2 tests; output-file/nonempty-directory code-1 tests;
normalized config and final summary equality; map sequence matching episode frames; missing config
read failure; and unchanged existing preset listing. Temporary directories always clean up.

The parity test loads generated `wasm-pkg/sugarscape.js` using the existing `initSync` pattern,
runs the release native CLI into a temporary directory, and compares the entire parsed record:

```typescript
const wasm = JSON.parse(burrow_replay_json(configText, seed, 16, 4));
const native = JSON.parse(readFileSync(join(outDir, 'episode.json'), 'utf8'));
expect(wasm).toEqual(native);
```

Define `configText` from each of the four example files, use named seeds `7` and
`18446744073709551615`, and initialize WASM from the generated binary. Use `execFileSync` argument
arrays; `try/finally` removes temporary directories. Check strict seed rejection (`-1`, `7.0`,
overflow) and invalid sampling through the WASM boundary. These are parity fixtures, not scientific sweeps.

- [x] **Step 2:** run CLI tests and focused Vitest after generating WASM; verify failures from
  the missing command/export rather than fixture setup mistakes.
- [x] **Step 3: implement the thin adapters and examples.** Serialize only the shared core
  episode. Define explicit conversion from parse errors to contextual `FieldError` records.
  Example JSON follows the tagged fixture and snake_case enum conventions chosen in Task 1:

```json
{
  "fixture": { "growing": { "width": 41, "height": 25, "workers": 8 } },
  "transport": "relay", "cue": "responsive", "freshness_window": 32,
  "relay_distance": 3, "response_weight": 3, "minimum_recent_units": 2
}
```

The other three files vary only transport/cue. Document observation limits, supplied navigation,
spoil overlay, two-worker capacity, units/costs, censored deliveries and operational caps.

- [x] **Step 4: show the acceptance scene and complete verification.**

```bash
cargo run --release -p sugarscape-cli -- burrow --config docs/examples/burrow/relay-responsive.json --seed 7 --ticks 512 --sample-every 32 --out /tmp/sugarscape-burrow-1-review
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
npm --prefix web run wasm
npm --prefix web test -- src/burrow-parity.test.ts
npm --prefix web run build
```

Use a fresh empty acceptance directory. Inspect the actual map, trace and conservation summary;
replay into another directory and compare `episode.json` bytes. Reuse the generated WASM build
for focused checks, then run the existing web suite for boundary regressions. Run survey tests
using the existing survey manifest if core changes affect them. These commands are future checks,
not evidence of currently passing code.

- [ ] **Step 5:** obtain fresh whole-branch review after per-task reviews; resolve findings,
  update status/documentation, remove the completed tracker, and commit
  `feat(burrow): expose replay with checked CLI and native WASM parity`.

## Self-review and handoff

Coverage maps world/action requirements to Task 1, sensing/control to Task 2, scheduling/material
history/measurements to Task 3, and presentation/parity/regressions to Task 4. All five review-focus
conditions have named tests. Interface names are shared across tasks; the approved spec travels
with each implementer. Resource caps and CLI flags are explicit implementation choices for plan review.

Review this plan before implementation, preserving the user's subagent-driven execution method.
Implementation completion permits demonstrations and correctness checks, not biological validation
or registered scientific claims. A judged campaign needs a separate reviewed protocol and manifest.
