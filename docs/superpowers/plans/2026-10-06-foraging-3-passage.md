# F3 Private-Map Passage Foraging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement bounded seeded passage foraging with private learned topology, one-item food cargo and two-worker capacity throughout a supplied nest chamber and passages.

**Architecture:** Add `foraging::passage` alongside the unchanged F2 reference. An owned one-hop observation updates a worker's private map; pure private-map routing drives transactions against authoritative geometry and food. Candidate ticks commit state and RNG together; a bounded runner exposes replay inputs, diagnostics and observational views.

**Tech Stack:** Existing Rust, `FieldError`, `rng::SimRng`, rand 0.8, serde/serde_json and Cargo. No new dependency.

**Spec:** [F3 passage design](../specs/2026-10-05-foraging-3-passage-design.md), approved by the user on 2026-10-06. Read the binding spec and this plan together.

## Global Constraints

- Written spec approved on 2026-10-06. This plan awaits written review; implementation has not started. Execution method is already supplied: subagent-driven development.
- Dimensions 3–125; 1–256 explicit spawn positions in the nest; 0–256 uniquely identified food tokens on distinct open cells outside the nest.
- Nest contains at least two distinct open cells in one four-neighbor-connected chamber. At most two workers occupy any cell, including nest cells. No off-world queue.
- Disconnected food pockets are valid. Do not require all open cells/resources to connect to the nest. Workers start in the nest and learn their own return routes.
- Only five active parameters: p_search, p_return, lambda_fidelity, lambda_publish, lambda_waypoint. Retain F1 domains; fill unused angular adapter fields with zero. No evolved defaults.
- Fixed four-neighbor geometry and immediate cardinal observations; no diagonal sensing, authoritative route field, map sharing, remembered occupancy/food truth, excavation, spoil, drop, relay, regrowth or consumption.
- One sequential seeded RNG stream. One action opportunity per worker per tick in ascending ID order. Discrete nonempty selections consume one draw even for singleton candidates; empty selections consume none.
- 1–7,200 requested/cumulative ticks and at most 1,000,000 worker opportunities. Run the full horizon, including after delivery or inaccessible-food discovery by researchers.
- Each map and each route scratch structure is bounded by width * height. At most 4,000,000 cell classifications across private maps.
- `initial food = available food + carried food + delivered food`; cargo and resource ownership agree bijectively. Move, Pickup, Deposit and Wait exhaust the physical action classes.
- Pickup requires occupying food; Deposit requires occupying any nest cell. Neither combines with movement. Phase transitions and nest decisions accompany one action only.
- Congestion is an ordinary wait with destination retained; no displacement, forced swap, hidden detour/priority or promised progress. A failed tick rolls back observations/maps, physical state, advice, counters and RNG.
- Positive snapshot interval; initial/final frames without duplicates; compact snapshot JSON sum at most 64 MiB, excluding enclosing episode fields/delimiters. No ordinary full action/observation history.
- Snapshot, summary and single-worker knowledge view are observational. Episode contains normalized setup, five parameters and seed. No saved-state restoration or cross-platform trajectory claim.
- No F2/Burrow behavior change, ModelKind, CLI, WASM export, browser UI, universal world interface, scientific evaluation or automatic merge/push of this feature.

## Review Focus

1. Two nest cells accommodate more than two workers without bypassing per-cell capacity; a full nearest home entry waits rather than using a global occupancy-aware alternative (Tasks 1/2/4).
2. A recruited unseen site reveals its coordinate only; a route requiring movement away from it remains discoverable, while a disconnected resource remains unavailable without leaking that fact into decisions (Tasks 2/4/5).
3. A late worker fails after earlier workers learned cells, moved, picked up or published; the complete tick and subsequent RNG stream roll back (Task 4).
4. Public resource IDs include u64::MAX, and absent cargo/target uses Option rather than a sentinel; empty-world milestones remain absent (Tasks 3/4/5).
5. Construction observations count as computation without physical opportunities; queue peaks combine by max, all search entries/initial trips count, and snapshot/knowledge reads cannot perturb these totals (Tasks 1/4/5).

## Isolation, staging and evidence

The design worktree is `/Users/nathan/.config/superpowers/worktrees/SugarScape/foraging-3-design`, branch `foraging-3-design`, based on `6c9a50a`. Spec commit is `24e6b14`. Other campaigns advanced main afterward; do not pull, reset, merge main just for documentation or reuse their checkouts. Before execution inspect actual branch heads, clean tracked/untracked state, applicable AGENTS.md, current upstream changes and existing isolation. Reuse this worktree if appropriate. Any integration with newer main needs its own deliberate check and verification.

At execution, create root `IMPLEMENTATION_PLAN.md` with five stages matching Tasks 1–5. Each needs Goal, Success Criteria, Tests and Status fields as required by the user's AGENTS.md. Retain it through independent whole-branch review, update stage statuses after reviews and remove it when all stages are complete. Do not create the root execution tracker during planning.

For each task: inspect consuming code; write behavior tests first; save actual red output; implement minimally; verify green; self-review; obtain fresh spec-compliance and code-quality review under the SDD skill; update tracker/evidence and commit working increments. Task dependencies are sequential, not parallel implementations. Stop an approach after three failed attempts, record errors and reassess. Never bypass hooks, weaken existing assertions or disable tests. Save full commands/logs/exit codes and reviewer rulings outside the repository; retain outcomes here. Missing-API reds must be labeled honestly rather than claimed as behavioral failures.

Existing patterns to study: F1 config/information/rules; F2 setup/draws/ledger/server/controller/world/runner and its tests; Burrow observations/actions/private-observation routing and capacity tests. Do not run any scientific campaign as verification. Documentation-only plan checks do not establish runtime acceptance.

## File ownership

All new source paths are under `crates/sugarscape-core/src/foraging/passage/`.

| File | Responsibility | Task |
|---|---|---|
| mod.rs | Module declarations, public exports, rustdoc | 1, 3, 4, 5 |
| setup.rs | Pos, Resource, Parameters, Setup, validation/normalization | 1 |
| draws.rs | Checked variates, PCG seam, discrete selections | 1 |
| observation.rs | Own current/cardinal observation, no remote sensing | 1 |
| knowledge.rs | Bounded immutable-topology private map and learned views | 1 |
| metrics.rs | Checked computational totals, queue maximum | 1 |
| navigation.rs | Known-open BFS, frontiers, private target ranking and steps | 2 |
| ledger.rs | Available/Carried/Delivered identities and invariants | 3 |
| state.rs | Agent phases/cargo/targets and checked physical counters | 3 |
| server.rs | F1 adapter, publication/departure and lazy expiry | 3 |
| actions.rs | Physical movement/handling validation and action counters | 4 |
| decision.rs | Worker-only phase decisions, no authoritative world input | 4 |
| controller.rs | Owned observations, ordered transactions and invariant checks | 4 |
| world.rs | Validated initialization and atomic seeded tick | 4 |
| view.rs | Read-only worker/world/knowledge researcher schemas | 5 |
| runner.rs | Options, summaries, episode, bounded snapshot writer | 5 |
| tests/mod.rs | Exact shared fixtures and scripted variates | 1 |
| tests/setup_learning.rs | Aggregate validation and one-hop learning | 1 |
| tests/navigation.rs | Private BFS, frontiers and congestion | 2 |
| tests/ledger_server.rs | Food identity/conservation and advice | 3 |
| tests/controller.rs | Action cadence, information isolation, complete rollback | 4 |
| tests/runner.rs | Bounds, views, replay and output byte cap | 5 |

Modify only parent `crates/sugarscape-core/src/foraging/mod.rs` to export the new submodule/rustdoc, `docs/foraging.md`, the approved F3 spec and this plan/tracker for status/evidence. Create public integration tests at `crates/sugarscape-core/tests/foraging_passage.rs`. No Cargo dependency change.

## Shared signatures and fixture contract

Use `type Checked<T> = Result<T, Vec<crate::config::FieldError>>` internally. Public functions spell their Result types explicitly. Use Clone/PartialEq on private candidate state for rollback assertions; public replay views additionally derive Serialize and PartialEq. Setup/parameters/positions/resources derive Serialize, Clone and PartialEq; no input deserialization/restoration API is required.

Task 1 defines these exact interfaces:

```rust
pub struct Pos { pub x: u32, pub y: u32 }
pub struct Resource { pub id: u64, pub pos: Pos }
pub struct Parameters {
    pub p_search: f64, pub p_return: f64,
    pub lambda_fidelity: f64, pub lambda_publish: f64,
    pub lambda_waypoint: f64,
}
pub struct Setup {
    pub width: u32, pub height: u32,
    pub open: Vec<Pos>, pub nest: Vec<Pos>, pub workers: Vec<Pos>,
    pub resources: Vec<Resource>, pub parameters: Parameters,
}
// Pos::neighbors(self, width: u32, height: u32) -> Vec<Pos>, N,S,E,W
// Parameters::validate(&self) -> Checked<()>
// Parameters::information(&self) -> crate::foraging::CpfaParameters
// Setup::validate(&self) -> Checked<()>
// Setup::normalized(self) -> Checked<Setup>
// Setup::site(&self, pos: Pos) -> Checked<u64>
// Setup::position(&self, site: u64) -> Checked<Pos>
trait DrawSource { fn uniform(&mut self) -> Checked<f64>; }
struct PcgDraws<'a>(&'a mut crate::rng::SimRng);
// checked_uniform(&mut impl DrawSource) -> Checked<f64>
// draw_index(&mut impl DrawSource, length: u32) -> Checked<u32>
// choose(&[Pos], &mut impl DrawSource) -> Checked<Option<Pos>>
pub enum CellKnowledge { Unknown, KnownOpen, KnownSolid }
pub struct KnownCell { pub pos: Pos, pub kind: CellKnowledge }
struct Knowledge { width: u32, height: u32, cells: Vec<CellKnowledge> }
// Knowledge::new(width: u32, height: u32) -> Checked<Knowledge>
// Knowledge::kind(&self, pos: Pos) -> Checked<CellKnowledge>
// Knowledge::learn(&mut self, observation: &Observation) -> Checked<u64>
// Knowledge::known(&self) -> Vec<KnownCell>, sorted Pos, excludes Unknown
// Knowledge::counts(&self) -> (u32, u32), open/solid
struct ObservedCell { pos: Pos, open: bool, occupants: u32, food: bool }
struct Observation { origin: Pos, cells: Vec<ObservedCell> }
// observe(&Setup, origin: Pos, &BTreeMap<Pos,u32>, &BTreeSet<Pos>)
//     -> Checked<Observation>; look up only current and N,S,E,W
struct ComputeCounts {
    observations: u64, cells_inspected: u64, cells_learned: u64,
    route_calls: u64, route_visits: u64, peak_queue: u64, frontier_scans: u64,
}
// ComputeCounts::checked_include(&mut self, &ComputeCounts) -> Checked<()>
```

Knowledge fields remain private; implement the declared bounded array in Task 1. All internal cross-file types/methods use pub(super) as needed; their fields stay internal to the passage module. Observation visibility is internal; public KnownCell views never contain historical occupancy or food. Derive Copy/Eq/Ord on Pos and Copy/Eq on CellKnowledge. Normalize open/nest by Pos order and resources by ID; never reorder workers because that changes identity. Validate original inputs before sorting/allocation. Constructors reject duplicate inputs, not deduplicate them.

Shared fixture in tests/mod.rs (all tasks use this literal):

```rust
fn pos(x: u32, y: u32) -> Pos { Pos { x, y } }
fn setup() -> Setup {
    Setup {
        width: 5, height: 5,
        open: vec![pos(0,0), pos(1,0), pos(2,0), pos(3,0), pos(3,1)],
        nest: vec![pos(0,0), pos(1,0)], workers: vec![pos(0,0)],
        resources: vec![],
        parameters: Parameters {
            p_search: 1.0, p_return: 0.0, lambda_fidelity: 0.0,
            lambda_publish: 0.0, lambda_waypoint: 0.0,
        },
    }
}
#[derive(Clone)]
struct Scripted { values: Vec<f64>, next: usize }
impl Scripted { fn new(values: &[f64]) -> Self { Self { values: values.to_vec(), next: 0 } } }
impl DrawSource for Scripted {
    fn uniform(&mut self) -> Checked<f64> {
        let value = self.values.get(self.next).copied()
            .ok_or_else(|| vec![FieldError::new("draw", "script exhausted")])?;
        self.next += 1;
        Ok(value) // consumers independently check even malicious injected values
    }
}
```

### Task 1: Checked setup, one-hop sensing and private learning

**Files:** Create mod.rs, setup.rs, draws.rs, observation.rs, knowledge.rs, metrics.rs, tests/mod.rs and tests/setup_learning.rs; export `passage` from the parent; create tracker Stage 1 at execution.
**Consumes:** FieldError, F1 parameter validation and rng::SimRng.
**Produces:** Every Task 1 signature above and independent bounded geometry/maps.

- [ ] Write tests including these exact representative cases; add indexed rejection cases for dimensions 2/126, zero/257 workers, 257 resources, duplicate IDs/cells/open/nest, singleton/disconnected nest, spawn outside nest and three same-cell workers, food in solid/nest/out-of-bounds cells, NaN and invalid numeric domains. Include `u64::MAX` identity, zero food, valid 125x125 dimensions, disconnected open food and four workers distributed two per nest cell.

```rust
#[test]
fn chamber_capacity_applies_per_cell() {
    let mut s = setup();
    s.workers = vec![pos(0,0), pos(0,0), pos(1,0), pos(1,0)];
    assert!(s.validate().is_ok());
    s.workers.push(pos(0,0));
    assert!(s.validate().unwrap_err().iter().any(|e| e.field == "workers[4]"));
}
#[test]
fn learning_does_not_reveal_second_hop_food_or_topology() {
    let s = setup();
    let o = observe(&s, pos(0,0), &BTreeMap::from([(pos(0,0),1)]),
                    &BTreeSet::from([pos(3,0)])).unwrap();
    let mut k = Knowledge::new(5,5).unwrap();
    assert_eq!(k.learn(&o).unwrap(), 3);
    assert_eq!(k.kind(pos(2,0)).unwrap(), CellKnowledge::Unknown);
    assert!(o.cells.iter().all(|c| c.pos != pos(3,0)));
}
#[test]
fn one_candidate_still_draws() {
    let mut d = Scripted::new(&[0.9]);
    assert_eq!(choose(&[pos(1,0)], &mut d).unwrap(), Some(pos(1,0)));
    assert_eq!(d.next, 1);
}
```

- [ ] Run `cargo test -p sugarscape-core foraging::passage::tests::setup_learning`; record intended red/missing-API failure. Then implement aggregate original-input validation, normalized setup, signed-safe neighbor/index conversions, checked injected variates and map learning.

```rust
// No allocation of a dense map precedes validation of dimensions/lists.
let information = crate::foraging::CpfaParameters {
    p_search: self.p_search, p_return: self.p_return,
    omega: 0.0, lambda_informed: 0.0,
    lambda_fidelity: self.lambda_fidelity,
    lambda_publish: self.lambda_publish,
    lambda_waypoint: self.lambda_waypoint,
};
// A cell classification may only move Unknown -> observed kind.
match (old, observed) {
    (CellKnowledge::Unknown, kind) => { cells[index] = kind; learned += 1; }
    (kind, same) if kind == same => {}
    _ => return Err(vec![FieldError::new("knowledge", "fixed topology observation conflicts")]),
}
```

- [ ] Validate the entire incoming Observation before mutating Knowledge: origin is KnownOpen in the observation, entries are distinct current/cardinal in-bounds cells, solid entries have zero occupants/no food, occupants <=2. A later conflicting classification leaves all map entries unchanged. Repeated observations learn zero new cells; changing local occupancy/food never changes stored classifications. KnownCell output excludes Unknown and sorts by Pos. Test boundary neighbor order, malformed observations, conflict rollback, uniform 0/largest-below-1, NaN/out-of-range variates, zero length and empty choose consuming no draw.
- [ ] Implement ComputeCounts with checked preparation before mutation, additive totals and `peak_queue = max`. Test overflow atomicity and peak max versus sum. Initial observation counting is integrated in Task 4.
- [ ] Verify focused tests, `cargo fmt --all --check`, `cargo clippy -p sugarscape-core --all-targets -- -D warnings`. Permit narrowly scoped documented `#[allow(dead_code)]` only for internal helpers whose consumer is named in Tasks 2–4; remove every such allowance by Task 4. Do not expose internal helpers merely to satisfy lint.
- [ ] Self-review and fresh task spec/quality reviews; update Stage 1 and commit `feat(foraging): add checked passage setup and private learning`.

### Task 2: Routes and frontiers from private knowledge

**Files:** Create navigation.rs and tests/navigation.rs; register them in mod.rs/tests/mod.rs; update Stage 2.
**Consumes:** Knowledge/Observation/Pos/choose/ComputeCounts from Task 1. Routing functions do not accept Setup, physical food or global worker lists.
**Produces:** These exact internal signatures:

```rust
struct Frontier { pos: Pos, unknown_neighbors: Vec<Pos>, distance: u32 }
enum Navigation { AtGoal, Move(Pos), Blocked, Unreachable }
// frontiers(&Knowledge, origin: Pos) -> Checked<(Vec<Frontier>, ComputeCounts)>
// select_frontier(&Knowledge, origin: Pos, informed: Option<Pos>,
//                 &mut impl DrawSource) -> Checked<(Option<Pos>, ComputeCounts)>
// route_step(&Knowledge, origin: Pos, goals: &[Pos], &Observation,
//            &mut impl DrawSource) -> Checked<(Navigation, ComputeCounts)>
// wander(origin: Pos, &Observation, &mut impl DrawSource) -> Checked<Navigation>
```

- [ ] Write deterministic route/frontier tests. Fixture for a mandatory detour: KnownOpen path `(1,1),(1,2),(2,2),(3,2),(3,1)`; goal `(3,1)`; map `(2,1)` KnownSolid. The first move is south even though Manhattan distance increases. Informed-frontier ranking uses min distance from Unknown neighbors, then known origin route distance; all nonimproving candidates remain eligible. Tests may build knowledge from successive legal one-hop observations using `learn`, preserving the information boundary.

```rust
#[test]
fn known_route_accepts_a_step_away_from_the_destination() {
    let mut k = Knowledge::new(5,5).unwrap();
    let mut s = setup();
    s.open = vec![pos(1,1),pos(1,2),pos(2,2),pos(3,2),pos(3,1)];
    s.nest = vec![pos(1,1),pos(1,2)]; s.workers = vec![pos(1,1)];
    s.validate().unwrap();
    for p in s.open.iter().copied() {
        k.learn(&observe(&s,p,&BTreeMap::new(),&BTreeSet::new()).unwrap()).unwrap();
    }
    let o = observe(&s,pos(1,1),&BTreeMap::new(),&BTreeSet::new()).unwrap();
    let mut d = Scripted::new(&[0.5]);
    let (step, _) = route_step(&k,pos(1,1),&[pos(3,1)],&o,&mut d).unwrap();
    assert_eq!(step, Navigation::Move(pos(1,2)));
    assert_eq!(d.next,1);
}
```

- [ ] Run `cargo test -p sugarscape-core foraging::passage::tests::navigation` for red. Implement BFS over KnownOpen only using N,S,E,W; frontier lists sort by Pos; an informed selection ranks `(min_unknown_neighbor_manhattan, distance)` before uniformly selecting among sorted ties. Uninformed selection is uniform across all reachable sorted frontiers.

```rust
let rank = |f: &Frontier| (
    f.unknown_neighbors.iter().map(|p| p.x.abs_diff(site.x) + p.y.abs_diff(site.y))
        .min().expect("a frontier has unknown neighbors"),
    f.distance,
);
// Reverse multi-source BFS from eligible KnownOpen goals produces distance.
// Immediate decreasing-distance neighbors are filtered by fresh occupancy <2.
// Only nonempty eligible choices draw; geometry/BFS itself never draws.
```

- [ ] Cover multiple home goals, goals already reached (no draw), unknown/solid/unreachable goals, zero frontiers, observed reachable nonvisited cells, ties at exact cumulative boundaries, and retained destination under full next-step capacity. Occupancy filter only affects the adjacent first step; changing unobserved remote occupancy cannot change a step. A full nearest nest route returns Blocked even if a longer route to another home cell is free. A blocked choice consumes no selection draw. route_step returns AtGoal for current goal, Unreachable for no privately known route, Blocked for an existing shortest route with no legal immediate step, and Move otherwise. wander maps an empty legal-neighbor set to Blocked, interpreted by decision as NoNeighbor rather than route congestion.
- [ ] Assert BFS visits/peak queue on a literal corridor, frontier cell scans, scratch size <=grid size and malformed origin/observation rejection before drawing. Counts must reflect executed computation, not guessed costs; document whether each frontier scan counts examined known-open cells (use that convention throughout).
- [ ] Verify focused tests/fmt/core clippy; fresh reviews; update Stage 2 and commit `feat(foraging): route through privately learned passages`.

### Task 3: Conserved food, agent accounting and nest advice

**Files:** Create ledger.rs, state.rs, server.rs and tests/ledger_server.rs; register exports/tests and update Stage 3.
**Consumes:** Task 1 types and F1 information utilities; agent Knowledge; no full-world navigation.
**Produces:** Exact physical/agent/server contracts:

```rust
pub enum ResourceState { Available, Carried { agent: u32 }, Delivered }
pub struct ResourceView { pub resource: Resource, pub state: ResourceState }
pub struct Inventory { pub initial: u32, pub available: u32, pub carried: u32, pub delivered: u32 }
struct Ledger { resources: Vec<ResourceView> }
// Ledger::new(&Setup) -> Ledger
// Ledger::claim(pos: Pos, agent: u32) -> Checked<Option<u64>>
// Ledger::deposit(resource: u64, agent: u32) -> Checked<()>
// Ledger::available() -> BTreeSet<Pos>; observation builder only
// Ledger::inventory() -> Inventory
// Ledger::views() -> &[ResourceView]
// Ledger::check(&[Agent]) -> Checked<()>
pub enum Phase { Departing, Searching, Returning }
struct Agent {
    id: u32, pos: Pos, phase: Phase, map: Knowledge,
    cargo: Option<u64>, find: Option<crate::foraging::FindRecord>,
    site: Option<Pos>, frontier: Option<Pos>,
    work: WorkCounts, compute: ComputeCounts,
}
pub struct WorkCounts {
    pub opportunities: u64, pub moves: u64,
    pub departure_moves: u64, pub search_moves: u64,
    pub empty_return_moves: u64, pub loaded_return_moves: u64,
    pub pickups: u64, pub deposits: u64, pub waits: u64,
    pub transition_waits: u64, pub congestion_waits: u64,
    pub no_neighbor_waits: u64, pub empty_arrival_waits: u64,
    pub search_entries: u64, pub empty_returns: u64,
    pub fidelity_departures: u64, pub recruited_departures: u64,
    pub uninformed_departures: u64, pub publications: u64,
    pub abandoned_targets: u64,
}
// WorkCounts::checked_include(&mut self, &WorkCounts) -> Checked<()>
// WorkCounts::check(&self) -> Checked<()>; action and subcategory equations
struct Arrival { departure: crate::foraging::Departure, published: bool }
struct ServerRecordView { id: u64, site: u64, created_tick: u32, strength: f64 }
struct Record { id: u64, site: u64, created_tick: u32 }
struct Server { records: Vec<Record>, next_id: u64, expired: u64 }
// Server::arrive(&Parameters, tick: u32, capacity: u32,
//                find: Option<FindRecord>, draws: [f64;3]) -> Checked<Arrival>
// Server::views(&Parameters, tick: u32) -> Checked<Vec<ServerRecordView>>
// Server::expired(&self) -> u64
```

Server representation is `records: Vec<Record>, next_id: u64, expired: u64`; Record has `id: u64, site: u64, created_tick: u32`. Derive Default for empty Server and zero counters. Expose physical and computational counters as public read-only view values, not mutable Agent/Knowledge/Ledger/Server. On initial Agent construction Task 4 sets `uninformed_departures=1`.

- [ ] Write ledger and server reds: preserve arbitrary resource IDs; claim occupies supplied cell and capacity one; wrong-owner/double deposit leave state unchanged; inventory sums; frozen density will be owned by Task 4 observations, not recomputed in Ledger. Explicit representative assertions:

```rust
#[test]
fn maximal_resource_identity_has_no_sentinel_meaning() {
    let mut s = setup();
    s.resources = vec![Resource { id: u64::MAX, pos: pos(3,0) }];
    let mut l = Ledger::new(&s);
    assert_eq!(l.claim(pos(3,0),7).unwrap(),Some(u64::MAX));
    l.deposit(u64::MAX,7).unwrap();
    assert_eq!(l.inventory(),Inventory { initial:1,available:0,carried:0,delivered:1 });
}
#[test]
fn publication_is_visible_to_the_same_arrival_departure() {
    let mut s = setup(); s.parameters.lambda_fidelity = 256.0;
    let mut server = Server::default();
    let find = Some(FindRecord { site: s.site(pos(3,0)).unwrap(), count: 1 });
    let result = server.arrive(&s.parameters,0,1,find,[0.0,0.99,0.0]).unwrap();
    assert!(result.published);
    assert!(matches!(result.departure,Departure::Recruitment { .. }));
}
```

- [ ] Run `cargo test -p sugarscape-core foraging::passage::tests::ledger_server` and record red. Implement sorted identity ledger, checked staged mutation and owner invariants. Do not return densities from a global ledger lookup to the controller.
- [ ] Implement server using `Parameters::information`, F1 publication/departure, PaperBelow and LaterArgosStrengthWeighted. Validate all independent supplied draws before committing; clone pending server and apply expiry/publication/departure atomically. Empty arrival publishes nothing but may recruit. Publication gets monotonically increasing ID and current processing tick; duplicate sites remain distinct. Views calculate strengths without expiry/mutation. Reject future creation ticks, capacity/ID/expired-count overflow contextually.

```rust
let p = parameters.information();
let request = crate::foraging::publication(&p, find, draws[0])?;
// Append request to pending records, expire weaker-than-threshold records,
// then produce ordered F1 Waypoint values including the new record.
let choice = crate::foraging::departure(&p, find, &waypoints,
    WaypointSelection::LaterArgosStrengthWeighted, WaypointThreshold::PaperBelow,
    draws[1], draws[2])?;
```

- [ ] Add tests for exact 0.001 retention, weaker expiry, stale-site recruitment with no ledger input, private fidelity priority, duplicate publications at a site, views not expiring records, independent invalid unused draws, maximal ID overflow and checked-counter rollback. WorkCounts checks action and subcategory equations; adding counts prepares all sums before assignment.
- [ ] Verify focused tests/fmt/core clippy; reviews; update Stage 3 and commit `feat(foraging): conserve passage food and compose nest advice`.

### Task 4: Worker-only decisions and complete atomic ticks

**Files:** Create decision.rs, actions.rs, controller.rs, world.rs and tests/controller.rs; add World/Phase/counter exports; update Stage 4 and remove all staging dead-code allowances.
**Consumes:** All earlier contracts, including the physically separate observation builder and private navigation.
**Produces:** Public `World::new(setup: Setup, seed: u64) -> Checked<World>` and `World::step(&mut self) -> Checked<()>`; private contracts:

```rust
struct Policy { width: u32, height: u32, nest: Vec<Pos>, parameters: Parameters }
enum WaitReason { Transition, Congestion, NoNeighbor, EmptyArrival }
enum Action { Move(Pos), Pickup, Deposit, Wait(WaitReason) }
struct Decision { agent: Agent, action: Action }
// decide(&Policy, agent: &Agent, &Observation, &mut impl DrawSource)
//     -> Checked<Decision>; never accepts Setup, Ledger, Server or State
struct State {
    tick: u32, agents: Vec<Agent>, ledger: Ledger, server: Server,
    first_pickup_tick: Option<u32>, first_delivery_tick: Option<u32>,
    all_delivered_tick: Option<u32>,
}
pub struct World { setup: Setup, state: State, rng: crate::rng::SimRng }
// apply(&Setup, &mut State, index: usize, Decision, &Observation,
//       &mut impl DrawSource) -> Checked<()>
// advance(&Setup, &mut State, &mut impl DrawSource) -> Checked<()>
// check(&Setup, &State) -> Checked<()>
```

World representation is normalized `setup: Setup, state: State, rng: SimRng`. Policy copies dimensions/nest/active parameters only, never open/resource/spawn lists. Decision clones only the worker; learn its fresh observation before routing. The decision layer adds one observation, cells.len() inspected cells and the returned learned count to that worker's computation counters; constructor initialization performs the same accounting with zero physical opportunities. Navigation returns measured computation to checked_include on the decision's worker clone. Nest arrival is handled by apply after physical Deposit/empty-arrival action, with narrowly provided server advice; decide cannot access the global server. apply assigns the returned agent before applying physical transaction against candidate state; failures remain within the cloned tick.

- [ ] Write behavior tests using Scripted draws and literal initial/controller states. Cover initial observations/learned counts and one uninformed trip per worker; two workers diverge in branch maps; two physical worlds differing only outside local view yield identical worker decisions/maps/draw consumption; recruitment sets only site and never marks unknown route cells. Create helper `advance_scripted(world: &mut World, draws: &mut Scripted) -> Checked<()>` that clones State and Scripted, calls advance and commits both only on success.

```rust
#[test]
fn a_failed_tick_preserves_knowledge_physics_and_draw_position() {
    let mut w = World::new(setup(),12).unwrap();
    let before = w.state.clone();
    let mut d = Scripted::new(&[]); // first p_search draw fails after observation
    assert!(advance_scripted(&mut w,&mut d).is_err());
    assert_eq!(w.state,before);
    assert_eq!(d.next,0);
}
```

- [ ] Run `cargo test -p sugarscape-core foraging::passage::tests::controller` and record red. Initialize all workers with empty private maps/cargo/find/site/frontier, Departing phase, one initial uninformed trip; build/count only normal spawn observations; consume zero initialization draws. Build occupancy and available-food indexes solely for observe, not routing inputs.
- [ ] Implement ordered observation -> worker-only decision -> physical action -> optional nest advice. All actions count exactly once. Pickup computes fresh density from Observation before claim, captures `find.site = setup.site(agent.pos)`, increments pickups and first-pickup tick, sets Returning/cargo and clears site/frontier. Deposit increments delivery milestones only after owner-checked deposit, then processes three independent arrival draws, increments chosen departure/publication counters, clears find/cargo/frontier and sets Departing/site as appropriate.

```rust
// World::step, borrowing fields of a fully cloned candidate:
let mut candidate = self.clone();
advance(&candidate.setup,&mut candidate.state,&mut PcgDraws(&mut candidate.rng))?;
*self = candidate;
// Empty worlds keep all_delivered_tick None; only a real final Deposit sets it.
```

- [ ] Pin the following complete draw order. Test endpoints p_search/p_return 0 and 1 with literal draw counts and transition waits. Invalid injected values always fail at the consumer.

| Opportunity | Draw sequence |
|---|---|
| Initialization | None |
| Uninformed Departing, switch succeeds | p_search only; enter Searching/clear targets |
| Uninformed Departing, switch fails and current frontier target equals position | p_search only; enter Searching |
| Uninformed Departing, no retained valid frontier | p_search; frontier selection if nonempty; immediate route-step selection if nonempty |
| Uninformed Departing, retained valid frontier | p_search; immediate route-step selection if nonempty |
| Uninformed Departing, no frontier anywhere | p_search only; enter Searching |
| Informed Departing at site / impossible pursuit | None; enter Searching, abandonment only for impossible pursuit |
| Informed Departing to KnownOpen site | Immediate route-step selection if nonempty |
| Informed Departing to Unknown site, no valid retained frontier | Ranked frontier selection if nonempty; immediate route-step selection if nonempty |
| Informed Departing to Unknown site, retained valid frontier | Immediate route-step selection if nonempty |
| Searching gives up | p_return only; enter empty Returning |
| Searching picks current-cell food | p_return only; Pickup and enter loaded Returning |
| Searching explores | p_return; frontier selection if needed/nonempty; immediate route-step selection if nonempty |
| Searching, exhausted private frontier set | p_return; wandering selection if nonempty |
| Returning outside chamber | Immediate route-step selection if nonempty; blocked waits draw none |
| Returning inside chamber, loaded or empty | publication, fidelity, recruitment in that order; no movement/frontier-selection draw |

- [ ] Test move-into-food then distinct Pickup, move-into-nest then distinct Deposit, give-up-before-pickup, no pickup during Departing, no nine-tick survey delay, absent angular draws, local count center/cardinal only and sequential pickup contention. Maps persist across trips; successful-find counts clear after arrival and cannot republish on empty return. KnownSolid/exhausted-Unknown informed pursuit abandons without deleting server advice or consulting resource truth.
- [ ] Test full-next-cell capacity inside/outside chamber, later-agent visibility, blocked destination retention, no forced swaps, and multi-source nearest known-home routing with no hidden longer detour. Initial chamber occupancy two per cell remains valid. Only current observed occupancy affects next-step choice.
- [ ] Extend rollback beyond the representative test: two workers with the second draw failing after the first worker learns and moves/picks up; a loaded first worker deposits/publishes before the later failure; compare State/maps/ledger/server/all counters and Scripted position. For production World, use private unit fixtures to trigger checked counter failure, then compare subsequent PCG draws/replay against an unchanged cloned reference. No public failure-injection API.
- [ ] check validates stable ID/index correspondence, map dimensions/topology truth for known cells, physical open positions and occupancy<=2, cargo/find/Returning coherence, find sites/counts, target geometry, ledger bijection, work equations and cumulative budgets. Researcher invariant checks may inspect authoritative truth; decision cannot. Error fields include processing tick/worker context. Check candidate after the complete tick; no successful partial commit.
- [ ] Verify all passage tests, fmt and core clippy; inspect for remaining staged allowances. Fresh reviews; update Stage 4 and commit `feat(foraging): execute atomic private-map passage ticks`.

### Task 5: Bounded runner, researcher views and public acceptance

**Files:** Create view.rs, runner.rs, tests/runner.rs and `crates/sugarscape-core/tests/foraging_passage.rs`; extend module rustdoc/exports and docs/foraging.md; update spec/plan/tracker evidence.
**Consumes:** Normalized setup, World/State, Knowledge, counters, ledger/server and controller::check.
**Produces:** Public view/output contracts below. All are Serialize/PartialEq; KnowledgeView contains no food/occupancy history.

```rust
pub struct RunOptions { pub ticks: u32, pub sample_every: u32, pub snapshots: bool }
pub struct KnowledgeView { pub agent: u32, pub cells: Vec<KnownCell> }
pub struct FindView { pub site: Pos, pub count: u32 }
pub struct WaypointView { pub id: u64, pub site: Pos, pub created_tick: u32, pub strength: f64 }
pub struct AgentView {
    pub id: u32, pub pos: Pos, pub phase: Phase, pub cargo: Option<u64>,
    pub find: Option<FindView>, pub site: Option<Pos>, pub frontier: Option<Pos>,
    pub known_open: u32, pub known_solid: u32,
    pub work: WorkCounts, pub compute: ComputeCounts,
}
pub struct Summary {
    pub completed_ticks: u32, pub inventory: Inventory,
    pub work: WorkCounts, pub compute: ComputeCounts,
    pub per_agent_work: Vec<WorkCounts>, pub per_agent_compute: Vec<ComputeCounts>,
    pub expired_records: u64, pub first_pickup_tick: Option<u32>,
    pub first_delivery_tick: Option<u32>, pub all_delivered_tick: Option<u32>,
}
pub struct Snapshot {
    pub summary: Summary, pub open: Vec<Pos>, pub nest: Vec<Pos>,
    pub agents: Vec<AgentView>, pub resources: Vec<ResourceView>,
    pub waypoints: Vec<WaypointView>,
}
pub struct Episode {
    pub setup: Setup, pub seed: u64, pub summary: Summary,
    pub snapshots: Vec<Snapshot>, pub snapshot_bytes: u64,
}
// World::summary(&self) -> Checked<Summary>
// World::snapshot(&self) -> Checked<Snapshot>
// World::knowledge(&self, agent: u32) -> Checked<KnowledgeView>
// run(setup: Setup, seed: u64, options: RunOptions) -> Checked<Episode>
```

Make ComputeCounts fields public at this stage for researcher consumption; methods remain checked. Public Result signatures use Vec<FieldError>, not an inaccessible private alias. No full private maps occur in Snapshot; geometry duplicates are counted within the declared byte cap.

- [ ] Write runner/public API tests before implementing these methods. Representative public acceptance fixture uses the Task 1 literal Setup and adds `Resource { id: u64::MAX, pos: Pos { x:3,y:0 } }`; use explicit Parameters, seed 12 and 40 ticks. Assert accounting/replay, not an assumed winning trajectory or guaranteed delivery.

```rust
#[test]
fn run_and_repeated_step_share_summary_and_private_knowledge() {
    let s = setup();
    let e = run(s.clone(),12,RunOptions { ticks:20,sample_every:7,snapshots:true }).unwrap();
    let mut w = World::new(s,12).unwrap();
    for _ in 0..20 { w.step().unwrap(); }
    assert_eq!(e.summary,w.summary().unwrap());
    assert_eq!(e.snapshots.first().unwrap().summary.completed_ticks,0);
    assert_eq!(e.snapshots.last().unwrap().summary.completed_ticks,20);
    assert_eq!(e.summary.work.opportunities,20);
}
```

- [ ] Run `cargo test -p sugarscape-core foraging::passage::tests::runner` and the new public integration-test target for red. Implement full-horizon run, normalized replay setup, checked totals and read-only views. Invalid worker ID returns contextual error. Summary includes constructor compute work but zero constructor physical opportunities; aggregates use max queue peaks. Server views remain observational/lazy.
- [ ] Use a bounded counting Write adapter before appending each compact JSON Snapshot, matching F2's budget definition. Test exact successful byte limit and one-byte-short failure through an internal `append_snapshot` seam, leaving frame list/byte counter unchanged on failure. Include initial/final once; disabled snapshots produce zero frames/bytes but still reject zero sample interval. Serialization/storage failures return no successful Episode.

```rust
let selected = completed % options.sample_every == 0 || completed == options.ticks;
if options.snapshots && selected {
    append_snapshot(&mut frames,&mut bytes,world.snapshot()?,64*1024*1024)?;
}
```

- [ ] Cover ticks 0/7201, exact legal 7200, agent*ticks boundary with 256 workers in >=128 valid nest cells (3906 legal ticks/3907 illegal), single-step cumulative exhaustion, positive sample interval, zero/maximum seed, empty world absent milestones and disconnected food remaining available. Use budget validation/helper states for boundary tests rather than repeatedly running huge scenes solely to mirror arithmetic.
- [ ] Compare same-seed runs at different sampling cadence and disabled recording; summaries and final single-worker knowledge must match, including computation counters. Call knowledge/snapshot/summary between steps on one clone and compare subsequent state/PCG outputs with the untouched clone. Validate knowledge view sorted classifications excludes Unknown and historical occupancy/food.
- [ ] Document the complete public example in passage module rustdoc so cargo doctests compile it. Update docs/foraging.md with limits, five active parameters, private learning/recruitment distinction, occupancy/deadlocks, action cadence, local density, replay identity, map views, supplied adaptations and F4 boundary. Preserve the F2 example and historical wording.
- [ ] Run final acceptance, save full logs/exits and inspect each result:

```bash
cargo test -p sugarscape-core --test foraging_passage
cargo fmt --all --check
cargo clippy -p sugarscape-core --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

- [ ] Fresh Task 5 reviews, then whole-branch spec/quality review against the approved spec/plan and actual evidence. Resolve findings in consolidated scoped corrections with behavioral reds where applicable, green checks and fresh scoped re-review. Retain root tracker until this gate passes; archive outcomes/rulings here, mark spec/guide accurately and remove the completed tracker. Commit `feat(foraging): expose bounded passage runs and knowledge views`, followed by evidence-only archival commit when appropriate. No integration/push/scientific execution is part of this plan.

## Plan self-review and handoff

Spec coverage: intent/scope/architecture -> all stages; setup/limits/observations/learning -> Task 1; routing/frontiers -> Task 2; parameters/food/advice -> Tasks 1/3/4; phase actions/ordered atomicity -> Task 4; draws -> Tasks 1/2/4; errors/metrics/views/storage/replay -> Tasks 1/4/5; acceptance -> all task tests plus Task 5 workspace gate; F4 separation -> Task 5 docs. Each of the five Review Focus inputs has an owning test step.

Type handoffs were checked across the five tasks. No authoritative Setup/food/worker list enters decide or routing. Constructor observations and zero initialization draws, transition order, stale advice, retained private maps, multi-source home routing and separate physical/computational costs have explicit owners. Queue peaks use max; all additive counters use checked sums. Per-worker phase/cargo invariants constrain the four move subcategories and four wait subcategories.

The user approved the written spec on 2026-10-06 and has already chosen subagent-driven execution. This written plan requires review before execution; do not ask the user to choose the execution method again. At execution preserve current concurrent main/remote work, verify the isolated base, create the five-stage tracker and begin Task 1. Current planning/validation is documentation-only, not evidence that F3 behavior is implemented or verified.
