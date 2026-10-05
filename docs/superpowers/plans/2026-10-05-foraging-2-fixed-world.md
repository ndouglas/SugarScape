# Foraging 2 Fixed World Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a bounded, seeded zero-error central-place foraging reconstruction with angular grid movement, conserved resource tokens and nest-return scoring.

**Architecture:** Extend `foraging` with a standalone `fixed` submodule. Checked geometry and injected draws support independently tested movement, resource bookkeeping and server decisions; an ordered controller composes them inside atomic ticks. A read-only runner exposes authoritative summaries and bounded snapshots.

**Tech Stack:** Existing Rust, `FieldError`, `rng::SimRng` (Pcg64Mcg), rand 0.8, serde/serde_json and Cargo verification tools. No new dependency.

**Spec:** [F2 fixed-world design](../specs/2026-10-04-foraging-2-fixed-world-design.md), approved by the user on 2026-10-05. [Historical audit](../../studies/2026-10-04-foraging-fixed-world-reading.md) travels with it.

## Global Constraints

- Status: implementation plan pending user review. Execution method already supplied: subagent-driven development. Implementation and scientific execution have not started.
- Grid width/height from 3 through 125, inclusive; agents 1–256; resources 0–256 with unique u64 IDs and distinct valid cells, none at the nest.
- Seven CPFA parameters are explicit and pass existing F1 validation; all agents share them. No claimed evolved defaults.
- Run horizon 1–7,200 ticks; `agents * horizon <= 1,000,000` opportunities. Single-step operation enforces the same cumulative limits.
- Snapshot interval is positive. Include initial and final snapshots without duplicates; total serialized snapshot bytes are at most 64 MiB.
- Angular headings, eight-neighbor geometry, clipped normal turns, nine survey waits and exact nest-cell arrival follow the approved reconstruction. Informed age is turns; waypoint age is ticks.
- Search tests give-up before moving, turns before forward-cell detection, and never senses on departure or while delayed.
- Process agents in ascending ID order. No exclusion/collision, hidden scheduler randomization, global food guidance, regrowth or consumption.
- `initial = available + assigned + delivered` after each committed tick; capacity is one token. Only completed nest return scores delivery.
- Publication precedes departure; own new record is visible. Duplicate sites are allowed; strength weighting with `PaperBelow` retains threshold equality. Empty returns use F1's recruitment fallback.
- One sequential seeded stream; explicitly scheduled state-specific draws. A failed tick commits neither world nor RNG changes; no successful partial-run summary.
- No ModelKind, CLI, WASM, browser, GA, error model, physical cargo physics, excavation, passage adaptation, generated evaluation campaign or new dependency.
- Preserve the F1 public API spelling, including `LaterArgosStrengthWeighted`; document its newly established historical support.
- Engineering fixtures and acceptance tests do not constitute quantitative reproduction or scientific efficacy results.

## Review Focus

1. A later agent fails after an earlier agent picked up or published: rollback the whole tick and RNG, not just the failing agent (Task 4).
2. Public callers use every u64 resource identity, including `u64::MAX`: no sentinel identity or resource-ID/site-ID confusion (Task 2).
3. A nest is on an edge and an uninformed target equals it: transition without a spurious move or divide-by-zero (Tasks 1 and 4).
4. Recording approaches the byte cap or final sampling duplicates a frame: fail at the exact bound and do not perturb simulation state (Task 5).
5. A successful neighborhood changes during nine survey waits: preserve the pickup-time count and prevent publication of stale finds after empty return (Tasks 2–4).

## Staging and execution discipline

Keep this approved-design worktree on `foraging-2-design` for planning. At execution, use the worktree skill to ensure isolation in the user's global directory. Reusing this clean isolated worktree is permitted; do not create nested worktrees. Verify branch/base and concurrent changes before implementation. Never reset another campaign's checkout or merge main merely to refresh documentation.

Create root `IMPLEMENTATION_PLAN.md` when execution begins, with five stages matching Tasks 1–5. Each stage has the user's required Goal, Success Criteria, Tests and Status fields. Advance status after the task's fresh implementation and independent review, commit working increments, and remove the tracker when all stages are complete. Keep actual commands/results and review rulings in this archived plan. Do not claim stages complete before evidence exists.

For every task: read its consuming interfaces and spec, write failing behavior tests, verify the intended red result, implement minimally, verify green, self-review, then obtain fresh spec-compliance and code-quality review using the subagent-driven skill. The parent owns cross-task integration. Stop an approach after three failed attempts and document/reassess. No hook bypass, disabled test or unnumbered TODO.

## File map and ownership

All source paths below are relative to `crates/sugarscape-core/src/foraging/fixed/` unless fully qualified.

| File | Responsibility | Task |
|---|---|---|
| `mod.rs` | Public reexports and source/scope rustdoc | 1, 4, 5 |
| `setup.rs` | Pos, Resource, Setup, aggregate validation, site encoding | 1 |
| `draws.rs` | Checked draw seam, PCG adapter and integer selection | 1 |
| `movement.rs` | Angular/normal/edge/neighbor movement with explicit draws | 1 |
| `ledger.rs` | Resource identities, local density, claim and deposit | 2 |
| `state.rs` | Private Agent, Phase, WorkCounts and views | 2, 4 |
| `server.rs` | Publication lifecycle, lazy decay, F1 departure composition | 3 |
| `world.rs` | Runtime State, seeded construction and atomic tick facade | 4 |
| `controller.rs` | Ordered opportunities and per-agent state transitions | 4 |
| `runner.rs` | Options, bounded snapshots, summary and run | 5 |
| `tests/mod.rs` | Shared deterministic fixtures and scripted draws | 1–5 |
| `tests/setup_movement.rs` | Geometry, validation and sampling | 1 |
| `tests/ledger.rs` | Resource conservation and observation | 2 |
| `tests/server.rs` | Server decisions and decay | 3 |
| `tests/controller.rs` | Timing, ordered contention and rollback | 4 |
| `tests/runner.rs` | Sampling, byte budget and replay | 5 |
| `crates/sugarscape-core/tests/foraging_fixed.rs` | Public API acceptance/replay | 5 |
| `crates/sugarscape-core/src/foraging/mod.rs` | Add `pub mod fixed;`; retain F1 exports | 1 |
| `docs/foraging.md` | Reference usage, units, changes and limits | 5 |

Source-relative imports in examples assume unit tests are under `fixed::tests`, with explicit imports from the owning sibling modules. Setup geometry helpers are internal; public callers submit Setup to validated World construction. Internal items use `pub(super)` or `pub(crate)` as needed by siblings; do not expose mutable agents, server or ledger publicly. Public views derive Serialize and PartialEq; private state is Clone/PartialEq for behavior assertions. Do not derive serde on F1 types merely to serialize a new view.

## Shared interfaces and test fixtures

Task 1 produces these types and functions; signatures are contracts for later tasks:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct Pos { pub x: u32, pub y: u32 }
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Resource { pub id: u64, pub pos: Pos }
#[derive(Clone, Debug, PartialEq)]
pub struct Setup {
    pub width: u32, pub height: u32, pub nest: Pos,
    pub agents: u32, pub resources: Vec<Resource>,
    pub parameters: crate::foraging::CpfaParameters,
}
// Setup::validate(&self) -> Result<(), Vec<FieldError>>
// Setup::contains(&self, pos: Pos) -> bool
// Setup::site(&self, pos: Pos) -> u64: y * width + x on valid geometry
// Setup::position(&self, site: u64) -> Result<Pos, Vec<FieldError>>

// Internal draw seam (all values finite in [0,1)):
trait DrawSource { fn uniform(&mut self) -> Result<f64, Vec<FieldError>>; }
struct PcgDraws<'a>(&'a mut crate::rng::SimRng);
// PcgDraws implements DrawSource with rand::Rng::gen::<f64>().
// draw_index(draws: &mut impl DrawSource, length: u32) -> Result<u32, Vec<FieldError>>
// Uses floor(u * length); rejects zero length. Edge uses length 4, then
// one coordinate draw with width/height; no hidden integer RNG stream.

// Internal movement API; validated setup and positions required:
// edge_target(&Setup, &mut impl DrawSource) -> Result<Pos, Vec<FieldError>>
// normal_increment(stddev: f64, &mut impl DrawSource) -> Result<f64, Vec<FieldError>>
// turn(heading: f64, stddev: f64, &mut impl DrawSource)
//     -> Result<(f64, u32), Vec<FieldError>> // wrapped heading, added delay
// ahead(&Setup, Pos, heading: f64) -> Option<Pos>
// search_step(&Setup, Pos, heading: &mut f64, &mut impl DrawSource)
//     -> Result<Pos, Vec<FieldError>> // may replace heading at boundary
// directed_step(&Setup, from: Pos, target: Pos, &mut impl DrawSource)
//     -> Result<Pos, Vec<FieldError>> // from == target consumes no draw
```

Use this explicit test parameter set and setup; these are supplied engineering fixtures, not model defaults:

```rust
fn setup() -> Setup {
    Setup {
        width: 5, height: 5, nest: Pos { x: 2, y: 2 }, agents: 1,
        resources: vec![],
        parameters: crate::foraging::CpfaParameters {
            p_search: 1.0, p_return: 0.0, omega: 0.0,
            lambda_informed: 0.0, lambda_fidelity: 0.0,
            lambda_publish: 0.0, lambda_waypoint: 0.0,
        },
    }
}
#[derive(Clone)]
struct Scripted { values: Vec<f64>, next: usize }
impl Scripted {
    fn new(values: &[f64]) -> Self { Self { values: values.to_vec(), next: 0 } }
}
impl DrawSource for Scripted {
    fn uniform(&mut self) -> Result<f64, Vec<crate::config::FieldError>> {
        let value = self.values.get(self.next).copied()
            .ok_or_else(|| vec![crate::config::FieldError::new("draw", "script exhausted")])?;
        self.next += 1;
        if !value.is_finite() || !(0.0..1.0).contains(&value) {
            return Err(vec![crate::config::FieldError::new("draw", "must be finite in [0,1)")]);
        }
        Ok(value)
    }
}
```

Tests use supplied variates for specific outcomes and tolerance `1e-12` for independently calculated non-endpoint float values. Exact endpoints, integer ledgers and replay views use equality. No same-seed cross-platform claim.

### Task 1: Checked setup and angular movement

**Files:** Create the Task 1 files in the map, add the parent module export and tracker Stage 1.
**Consumes:** F1 `CpfaParameters::validate`, existing `FieldError`, `rng::SimRng`.
**Produces:** All shared interfaces above, independently usable movement helpers and common test fixtures.

- [ ] Write failing setup tests for dimensions 2/126, agents 0/257, 257 resources, duplicate IDs/cells, out-of-bounds/nest resources and aggregate invalid CPFA inputs. Validate original values before allocating a dense grid; resource list is already caller-owned. Contextual errors name indexed resources. Include valid 3×3 and 125×125 setups and site round trips at the final cell.

```rust
#[test]
fn setup_rejects_multiple_original_inputs() {
    let mut s = setup();
    s.width = 2;
    s.agents = 0;
    s.parameters.p_search = f64::NAN;
    let errors = s.validate().unwrap_err();
    assert!(errors.iter().any(|e| e.field == "width"));
    assert!(errors.iter().any(|e| e.field == "agents"));
    assert!(errors.iter().any(|e| e.field == "parameters.p_search"));
}
```

- [ ] Write failing movement tests. With stddev zero, draws `[0.5,0.5]` give unchanged heading and delay one. With stddev 4*pi, `[0.5,0.0]` clips to pi and delay four. At (2,2), heading pi/4 targets (3,3). At a corner, an outward heading and 32 outward redraws fail; no fallback. Directed travel to an adjacent target consumes no draw; from (0,0) toward (2,0), draw zero selects (1,0) because scan order and positive reductions are explicit. Test the largest representable uniform below one, cumulative boundaries, same-position target and edge-target/nest equality.

```rust
#[test]
fn a_zero_turn_still_waits_one_tick() {
    let mut d = Scripted::new(&[0.5, 0.5]);
    assert_eq!(turn(0.0, 0.0, &mut d).unwrap(), (0.0, 1));
    assert_eq!(d.next, 2);
}
#[test]
fn diagonal_search_is_not_four_neighbor_navigation() {
    let mut heading = std::f64::consts::FRAC_PI_4;
    let mut d = Scripted::new(&[]);
    assert_eq!(search_step(&setup(), Pos { x: 2, y: 2 }, &mut heading, &mut d).unwrap(),
               Pos { x: 3, y: 3 });
}
```

- [ ] Run `cargo test -p sugarscape-core foraging::fixed::tests::setup_movement`; confirm intended missing API/test behavior failure.
- [ ] Implement validation, geometry and draw/movement helpers. Prefix forwarded parameter fields with `parameters.`. Use signed intermediates for neighbor/rounding coordinates. Validate DrawSource values at the consumer boundary too, so malicious test implementations cannot sneak NaN into geometry. Use normalized cumulative intervals with final upper boundary exactly one.

```rust
let delta = normal_increment(stddev, draws)?.clamp(-PI, PI);
let wrapped = (heading + delta).rem_euclid(2.0 * PI);
let new_heading = if wrapped >= 2.0 * PI { 0.0 } else { wrapped };
let delay = (delta.abs() / (PI / 4.0 + 0.001)).floor() as u32 + 1;
```

Implement the initial search proposal without consuming a boundary draw. If illegal, allow 32 uniform replacement-heading proposals, then error. Normals always consume two draws, even at stddev zero. Invalid stddev/heading must error before sampling. Test tiny negative increments near heading zero; floating rem_euclid may round to 2*pi, which must normalize to zero to retain the half-open heading range. Directed neighbor weights use Euclidean distances, ordered dx then dy; same/adjacent target takes no selection draw.

- [ ] Rerun targeted tests, `cargo fmt --all -- --check`, and `cargo clippy -p sugarscape-core --all-targets -- -D warnings`; inspect every exit/result.
- [ ] Self-review, obtain fresh task reviews, update Stage 1 and commit only the Task 1 files plus tracker with `feat(foraging): add checked fixed-world movement`.

### Task 2: Conserved resources and agent accounting

**Files:** Create `ledger.rs`, `state.rs`, `tests/ledger.rs`; register modules/tests and update Stage 2.
**Consumes:** Task 1 `Setup`, `Pos`, `Resource`, site encoding.
**Produces:** The following private ledger/agent interfaces and serializable resource views:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub enum ResourceState { Available, Assigned { agent: u32 }, Delivered }
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ResourceView { pub resource: Resource, pub state: ResourceState }
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Inventory { pub initial: u32, pub available: u32, pub assigned: u32, pub delivered: u32 }
struct Ledger { resources: Vec<ResourceView> } // sorted by resource ID
// Ledger::new(&Setup) -> Self // setup already validated
// Ledger::claim(&mut self, &Setup, cell: Pos, agent: u32)
//     -> Result<Option<(u64, crate::foraging::FindRecord)>, Vec<FieldError>>
// Ledger::deposit(&mut self, resource: u64, agent: u32) -> Result<(), Vec<FieldError>>
// Ledger::inventory(&self) -> Inventory
// Ledger::views(&self) -> &[ResourceView]
// Ledger::check(&self, agents: &[Agent]) -> Result<(), Vec<FieldError>>
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Phase { Departing, Searching, Returning }
#[derive(Clone, Debug, PartialEq)]
struct Agent {
    id: u32, pos: Pos, heading: f64, target: Pos, phase: Phase,
    informed: bool, informed_turns: u32, delay: u32,
    cargo: Option<u64>, find: Option<crate::foraging::FindRecord>,
    work: WorkCounts,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct WorkCounts {
    pub opportunities: u64, pub waits: u64,
    pub directed_moves: u64, pub search_moves: u64,
    pub pickups: u64, pub deliveries: u64, pub empty_returns: u64,
    pub search_switches: u64, pub publications: u64,
    pub fidelity_departures: u64, pub recruited_departures: u64,
    pub uninformed_departures: u64,
}
// WorkCounts::checked_add_assign(&mut self, other: &Self) -> Result<(), Vec<FieldError>>
```

`search_switches` counts every entry into Searching, including informed target arrival. Initial uninformed departures are counted once per agent; later departures count the actual F1 choice. Server expiration is counted globally, not attributed to an arbitrary agent. All IDs accept their full declared range; site IDs encode cells separately from resource identities.

- [ ] Write failing tests for a single claim/deposit, contention, frozen count, clipped neighborhood, duplicate deposit and wrong-agent rejection. Claim at the same resource twice returns one success then None. Density is one plus currently available Moore neighbors, never assigned/delivered resources. `check` must verify the bijection between assigned tokens and cargo owners, no worker holds multiple items, and counts sum to initial.

```rust
#[test]
fn maximum_resource_id_is_a_real_resource() {
    let mut s = setup();
    s.resources = vec![Resource { id: u64::MAX, pos: Pos { x: 1, y: 1 } }];
    let mut ledger = Ledger::new(&s);
    let (id, find) = ledger.claim(&s, Pos { x: 1, y: 1 }, 0).unwrap().unwrap();
    assert_eq!(id, u64::MAX);
    assert_eq!(find.count, 1);
    assert_eq!(ledger.inventory(), Inventory { initial: 1, available: 0, assigned: 1, delivered: 0 });
    ledger.deposit(id, 0).unwrap();
    assert!(ledger.deposit(id, 0).is_err());
}
```

- [ ] Run `cargo test -p sugarscape-core foraging::fixed::tests::ledger`; confirm the intended red failure.
- [ ] Implement bounded lookup over at most 256 resources, without a redundant dense ownership grid. Freeze the count during claim before modifying availability (or compensate exactly once afterward). Use resource ID lookup for deposit, and verify ownership before mutation. Implement checked work aggregation; overflow returns an error.

```rust
// Successful claim return: resource identity is separate from encoded site.
let find = FindRecord { site: setup.site(cell), count: observed_count };
// Store exactly one ResourceState::Assigned { agent }, return Some((id, find)).
// Deposit accepts only that resource's matching Assigned owner.
```

- [ ] Verify targeted tests plus all `foraging` tests, formatting and core clippy. Self-review and obtain fresh task reviews; update Stage 2 and commit `feat(foraging): conserve claimed and returned resources`.

### Task 3: Nest server and private departure decisions

**Files:** Create `server.rs`, `tests/server.rs`; register modules/tests and update Stage 3. Clarify historical support in the existing F1 enum rustdoc without changing its name/behavior.
**Consumes:** Task 1 site encoding; existing F1 `publication`, `departure`, `FindRecord`, `Waypoint`, `Departure`, `waypoint_strength` and source variants.
**Produces:** Private Server and a read-only serializable server view:

```rust
struct Record { id: u64, site: u64, created_tick: u32 }
#[derive(Clone, Debug, Default, PartialEq)]
struct Server { records: Vec<Record>, next_id: u64, expired: u64 }
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct WaypointView {
    pub id: u64, pub site: Pos, pub created_tick: u32, pub strength: f64,
}
// Server::arrive(&mut self, &Setup, tick: u32, find: Option<FindRecord>,
//     draws: [f64; 3]) -> Result<(Departure, bool), Vec<FieldError>>
// Draw order: publication, fidelity, recruitment; bool is publication success.
// Server::views(&self, &Setup, tick: u32) -> Result<Vec<WaypointView>, Vec<FieldError>>
// Server::expired(&self) -> u64
```

Decay/expiration occurs lazily when processing nest arrival, matching server access. Read-only views show all retained records with strength at the requested tick, including a weak resident record pending its next access-time removal; views must never mutate or consume RNG. Expired count means actual removed records. Zero decay leaves records at one. Record count is at most setup resources, because each resource can complete only once; enforce the bound defensively. Never let server validation inspect availability to improve advice.

- [ ] Write failing tests for publication before selection/self-recruitment, fidelity priority, independent decisions, duplicate site publications, monotonic IDs, decay/expiration, equality convention through F1's existing explicit-weight tests, empty-return recruitment with fidelity draw zero, and stale/depleted-site recruitment.

```rust
#[test]
fn an_empty_return_can_use_its_predecessors_message() {
    let s = setup();
    let find = FindRecord { site: s.site(Pos { x: 1, y: 1 }), count: 1 };
    let mut server = Server::default();
    // This server-only fixture needs capacity for a real publication.
    let mut s = s;
    s.resources.push(Resource { id: 9, pos: Pos { x: 1, y: 1 } });
    server.arrive(&s, 0, Some(find), [0.0, 0.0, 0.0]).unwrap();
    let (choice, published) = server.arrive(&s, 1, None, [0.0, 0.0, 0.0]).unwrap();
    assert!(!published);
    assert_eq!(choice, Departure::Recruitment { waypoint: 0, site: find.site });
}
```

Use rate `ln(10)` and elapsed four ticks to prove below-threshold expiration without relying on equality rounding. For precise equality retain the existing supplied-weight F1 test; do not invent an exponential age that must round exactly to .001. For weighted-selection intervals use publications at separate creation ticks and independently calculated expected weights; test no published record remains when both counts are zero and decay has removed the last record.

- [ ] Run `cargo test -p sugarscape-core foraging::fixed::tests::server`; verify red.
- [ ] Implement the arrival composition with fresh F1 inputs. Validate all three draws even if find is None. Decode/validate sites; reject future record ages/ID overflow before mutation. Append strength-one publication, expire weak records, form ordered F1 snapshot and choose departure with the existing policies:

```rust
let choice = crate::foraging::departure(
    &setup.parameters, find, &snapshot,
    crate::foraging::WaypointSelection::LaterArgosStrengthWeighted,
    crate::foraging::WaypointThreshold::PaperBelow,
    draws[1], draws[2],
)?;
```

World-level atomicity comes in Task 4. Make internal arrival validation happen before effects where practical, and test malformed inputs produce no publication; no external mutation API is exposed.

- [ ] Run server and all foraging tests, formatting and core clippy. Review, update Stage 3 and commit `feat(foraging): compose nest waypoint decisions`.

### Task 4: Ordered controller and atomic ticks

**Files:** Create `world.rs`, `controller.rs`, `tests/controller.rs`; finish `state.rs` views, export public World/types and update Stage 4.
**Consumes:** Tasks 1–3 movement, Ledger, Agent/WorkCounts, Server and F1 variation rules.
**Produces:** Seeded World, committed ticks and read-only snapshots required by Task 5:

```rust
#[derive(Clone, Debug, PartialEq)]
struct State {
    tick: u32, agents: Vec<Agent>, ledger: Ledger, server: Server,
    first_pickup_tick: Option<u32>, first_delivery_tick: Option<u32>,
    all_delivered_tick: Option<u32>,
}
#[derive(Clone)]
pub struct World { setup: Setup, state: State, rng: crate::rng::SimRng }
// World::new(setup: Setup, seed: u64) -> Result<Self, Vec<FieldError>>
// World::step(&mut self) -> Result<(), Vec<FieldError>>
// World::snapshot(&self) -> Result<Snapshot, Vec<FieldError>>
// World::summary(&self) -> Result<Summary, Vec<FieldError>> // implemented in Task 5
// Internal advance(setup: &Setup, state: &mut State, draws: &mut impl DrawSource)
//     -> Result<(), Vec<FieldError>> // one complete tick on scratch state
```

Snapshot's public shape (define it now in `state.rs`, Task 5 consumes it):

```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct FindView { pub site: Pos, pub count: u32 }
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct AgentView {
    pub id: u32, pub pos: Pos, pub heading: f64, pub target: Pos,
    pub phase: Phase, pub informed: bool, pub informed_turns: u32,
    pub delay: u32, pub cargo: Option<u64>, pub find: Option<FindView>,
    pub work: WorkCounts,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Snapshot {
    pub completed_ticks: u32, pub inventory: Inventory,
    pub agents: Vec<AgentView>, pub resources: Vec<ResourceView>,
    pub waypoints: Vec<WaypointView>, pub expired_records: u64,
    pub work: WorkCounts, pub first_pickup_tick: Option<u32>,
    pub first_delivery_tick: Option<u32>, pub all_delivered_tick: Option<u32>,
}
```

Stage 4 includes all transitions. Final byte budgets/runner Summary are Stage 5. Snapshot conversion decodes find/server site IDs and reports invariant errors; it never samples draws or prunes server records. Initial all-delivered time is None for empty setups, because no delivery event occurred. For nonempty setups, set all-delivered time on the actual final deposit.

- [ ] Write failing controller tests using private State/Agent fixtures plus Scripted draws. An east-facing uninformed searcher at (1,2), nest (2,2), and resource (3,2) with zero variation first moves to (2,2), detects (3,2), freezes count, installs delay nine and claims a token. After nine tick opportunities, returning at the nest deposits exactly once on tick ten. Give-up at p_return=1 suppresses that pickup. Departure never picks it up.

```rust
#[test]
fn survey_waits_do_not_deliver_a_token_early() {
    let mut s = setup();
    s.resources.push(Resource { id: 17, pos: Pos { x: 3, y: 2 } });
    let mut w = World::new(s, 0).unwrap();
    let a = &mut w.state.agents[0];
    a.phase = Phase::Searching;
    a.pos = Pos { x: 1, y: 2 };
    a.heading = 0.0;
    a.delay = 0;
    let mut d = Scripted::new(&[0.5, 0.0, 0.0]); // give-up, two zero-turn draws
    advance(&w.setup, &mut w.state, &mut d).unwrap();
    for _ in 0..9 {
        advance(&w.setup, &mut w.state, &mut Scripted::new(&[])).unwrap();
    }
    assert_eq!(w.state.ledger.inventory().assigned, 1);
    advance(&w.setup, &mut w.state, &mut Scripted::new(&[0.0, 0.0, 0.0])).unwrap();
    assert_eq!(w.state.ledger.inventory().delivered, 1);
    assert_eq!(w.state.first_delivery_tick, Some(10));
}
```

- [ ] Add tests for informed age at entry/turn and no advance during waits/travel; target equality at an edge nest; empty-return memory clearing; F1 choice counters; weighted directed move at return; snapshot observational purity; sequential two-agent contention and same-tick publication visibility.
- [ ] Add a failure-after-earlier-pickup test. Inject exhausted/malformed draws through a private transactional helper that clones State and a Clone DrawSource, calls `advance`, then commits both only on success. The helper is a test seam for the same commit pattern as World::step, not a separate controller. Compare full State and script cursor to their pre-call values. Also clone World before a deliberate limit failure, then compare snapshots and the next native RNG sample to prove production RNG rollback.
- [ ] Run `cargo test -p sugarscape-core foraging::fixed::tests::controller`; verify red, then implement.

```rust
// Production World::step:
let mut candidate = self.clone();
advance(&candidate.setup, &mut candidate.state,
        &mut PcgDraws(&mut candidate.rng))?;
*self = candidate;
Ok(())
```

`advance` validates cumulative tick/opportunity budgets before work, processes stable agent order and uses checked counters. Attach tick/agent context to helper errors. It checks ledger/agent invariants and finite in-bounds geometry before advancing completed ticks. A delayed agent counts one opportunity/wait and draws nothing. At departure draw the switch for every eligible uninformed agent, even when target equality also holds. On entering search turn once; add its delay and informed age. Search consumes give-up first; a successful give-up ends the opportunity. Otherwise move, turn, then detect. Pickup replaces the new turn delay with nine. Return processes arrival only after its possible directed move, draws all three arrival uniforms even when empty, deposits, publishes, chooses/installs target, clears find/cargo and resets informed age. Edge targeting consumes two further uniforms only for an uninformed choice. Record the event processing tick before increasing completed ticks.

- [ ] Verify all foraging tests plus `cargo fmt --all -- --check` and core clippy; check invariants after each tick in the unit fixtures. Obtain fresh reviews, update Stage 4 and commit `feat(foraging): execute ordered atomic foraging ticks`.

### Task 5: Bounded runner, acceptance and public documentation

**Files:** Create `runner.rs`, `tests/runner.rs`, public `tests/foraging_fixed.rs`; export runner types/functions, update docs/foraging and tracker Stage 5.
**Consumes:** Task 4 World/step/snapshot, WorkCounts and Inventory.
**Produces:** Public run and Summary plus verified usage:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunOptions { pub ticks: u32, pub sample_every: u32, pub snapshots: bool }
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Summary {
    pub completed_ticks: u32, pub inventory: Inventory, pub work: WorkCounts,
    pub per_agent: Vec<WorkCounts>, pub expired_records: u64,
    pub first_pickup_tick: Option<u32>, pub first_delivery_tick: Option<u32>,
    pub all_delivered_tick: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct Episode {
    pub seed: u64, pub summary: Summary, pub snapshots: Vec<Snapshot>,
    pub snapshot_bytes: u64,
}
// World::summary(&self) -> Result<Summary, Vec<FieldError>>
// run(setup: Setup, seed: u64, options: RunOptions) -> Result<Episode, Vec<FieldError>>
// Internal append_snapshot(frames: &mut Vec<Snapshot>, bytes: &mut u64,
//     snapshot: Snapshot, limit: u64) -> Result<(), Vec<FieldError>>
```

`summary` recomputes aggregate work from per-agent authoritative counts with checked addition and validates the ledger. No derived efficacy rates are required in this first runner; users can calculate them using the explicit denominator. Public rustdoc records tick semantics, scheduling bias, local knowledge, reconstruction identity and censoring. No automatic completion stop.

- [ ] Write failing runner budget tests for ticks 0/7201, interval zero even when snapshots=false, agents*ticks >1,000,000 and whole-run requested limits before advancement. Initial/final frame counts: ticks=4, interval=2 yields completed ticks `[0,2,4]`; ticks=5, interval=2 yields `[0,2,4,5]`. With snapshots=false return an empty list and zero bytes; summary is unchanged. Empty fixtures run the full horizon with delivery times None.

```rust
#[test]
fn snapshot_sampling_is_an_observer() {
    let options = |sample_every, snapshots| RunOptions { ticks: 20, sample_every, snapshots };
    let sparse = run(setup(), 12, options(7, true)).unwrap();
    let dense = run(setup(), 12, options(1, true)).unwrap();
    let silent = run(setup(), 12, options(1, false)).unwrap();
    assert_eq!(sparse.summary, dense.summary);
    assert_eq!(sparse.summary, silent.summary);
    assert_eq!(sparse.snapshots.last(), dense.snapshots.last());
}
```

- [ ] Test exact byte boundaries cheaply with `append_snapshot` and a custom small limit. Count the canonical compact serde_json encoding of each Snapshot (no surrounding Episode fields or inter-frame delimiters). Exact remaining bytes succeeds; one byte too few errors before committing bytes/frame. Production limit is 64*1024*1024. Use a counting writer with a hard limit, propagating serialization/write errors; do not retain unbounded temporary JSON or cast unchecked sizes.
- [ ] Add public acceptance tests for repeated-seed Episode equality, run versus twenty manual steps, initial/final sampling, supplied resource identities and invariant inventory. Include nonempty hand-specified setups for replay and partial-return inventory; specific delivery timing stays in injected unit tests. Verify the public interface needs no private state access. Preserve F1 public tests.

```rust
#[test]
fn public_run_matches_manual_steps() {
    let s = setup();
    let episode = run(s.clone(), 12, RunOptions { ticks: 20, sample_every: 7, snapshots: true }).unwrap();
    let mut world = World::new(s, 12).unwrap();
    for _ in 0..20 { world.step().unwrap(); }
    assert_eq!(episode.summary, world.summary().unwrap());
    assert_eq!(episode.snapshots.last().unwrap(), &world.snapshot().unwrap());
}
```

The integration test defines its own explicit `setup()` using exported types and the shared fixture values, rather than importing private unit-test helpers.

- [ ] Run targeted runner and `cargo test -p sugarscape-core --test foraging_fixed`; verify red. Implement `run` after prevalidating setup and options together, using World::new then fixed-horizon stepping and deterministic observers. Output errors identify their budget/tick; returning Err yields no successful partial Episode. Per-snapshot appends commit only after serialization budget success. There is no RNG consumption in summary/serialization.
- [ ] Extend `docs/foraging.md` with a complete compiling public example, public limits, exact units/ordering, conserved-token meaning, stale information, deliberate historical-code deviations and distinct F3/F4 next steps. Update parent module rustdoc to describe both the unchanged stateless F1 reference and new fixed-world submodule. Do not continue calling the entire foraging module stateless after this addition.
- [ ] Run final acceptance once, saving exit codes and full logs outside the repo:

```bash
cargo fmt --all -- --check
cargo clippy -p sugarscape-core --all-targets -- -D warnings
cargo test --workspace
```

Any workspace failures need triage and scoped correction; do not bypass existing tests. After corrective edits repeat affected verification and broaden only for new integration concerns. Cargo doctests must compile the public example. No shell script is added; if execution introduces one, shellcheck it before committing.

- [ ] Self-review against every spec section, obtain task reviews, update Stage 5 and commit `feat(foraging): expose bounded fixed-world runs`. Request final whole-branch review against the approved spec/plan and recorded evidence. Resolve review findings with fresh scoped tests/review. Keep the tracker through final review, then remove it and record evidence/rulings in this plan and the spec/guide. Integration/merge remains a separate user action; no push or scientific protocol execution here.

## Plan self-review and handoff

Coverage maps: setup/budgets → Tasks 1/4/5; angular geometry/draw schedule → Tasks 1/4; identities/observations → Task 2; nest memory/server → Tasks 3/4; diagnostics/output/replay → Tasks 4/5; acceptance and documentation → Task 5. Review Focus items each have owning tests. File/interface names are shared explicitly; output views avoid extending F1 serde. The five stages each have an independently testable deliverable and are sequential dependencies, not parallel implementation work.

The user has already selected subagent-driven development. Written implementation-plan review remains pending; no need to ask them to choose an execution method again. After approval, begin at Task 1 with the root tracker and clean-worktree verification. No runtime code has been changed by writing this plan.
