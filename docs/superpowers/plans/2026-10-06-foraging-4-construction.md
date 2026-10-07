# F4 Shared-Worker Construction Foraging Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Couple paid excavation and direct spoil disposal to private-map food return, with one shared carrying slot and separately conserved food, spoil and terrain.

**Architecture:** Add `foraging::construction`, preserving F3 and Burrow. Dynamic local knowledge feeds private routing/face choice; a typed action layer owns terrain/material transactions. Complete candidate ticks include maps, ledgers, advice, access measurements and RNG, while bounded outputs remain observational.

**Tech Stack:** Existing Rust, FieldError, SimRng/rand 0.8, serde/serde_json, deterministic standard collections and Cargo. No new dependency.

**Spec:** [F4 construction design](../specs/2026-10-06-foraging-4-construction-design.md), approved by the user on 2026-10-06. Read it with this plan; the spec is binding.

## Global Constraints

- Spec approved on 2026-10-06. The user approved this written implementation plan on 2026-10-06; engineering execution is complete. Subagent-driven execution is already selected by the user's standing instructions.
- One homogeneous worker kind; one tagged Food/Spoil token or empty hands. Role allocation, relay transport, drops, loose/initial spoil, piles, sharing and mid-load switching are explicitly deferred.
- Dimensions 3–125; 1–256 spawn-list workers; food 0–256 with arbitrary unique u64 IDs/distinct cells outside nest/outlet; 1–7,200 ticks and at most 1,000,000 opportunities.
- Nest has at least two connected initially open cells; workers spawn there; capacity two applies to every cell. One initially open outlet lies outside nest and connects to it through initial open cells. It need not be diggable.
- Diggable mask is immutable, may overlap initially open cells, and reveals no hidden food. Only solid masked cells can open; no refill/collapse. Hidden protected food and disconnected exposed pockets are valid censored setups.
- Successful Dig opens one cell, creates one carried spoil, exposes any Hidden food, confirms only the digger's map, and consumes one action without movement/pickup/disposal.
- Food: initial = hidden + available + carried + delivered. Spoil: excavated = carried + disposed. Terrain: open = initially open + excavated. Cargo and each ledger agree bijectively, including equal numeric IDs across namespaces.
- Private observations are current/cardinal cells only. Old diggable walls may be stale after others dig; only own action or fresh local observation revises them. Never import F3's fixed-wall-equals-current-truth invariant.
- Keep the five active F3 parameters/domains and F1 zero-filled angular adapter. No dig probability, role fraction, relay length, spoil cue or hidden global resource/route guidance.
- Explore reachable private open frontiers before digging. Known face choice/approach routes use private knowledge only. Loaded cargo finishes its own transport before any other job, without movement/scheduler priority.
- Dig pauses Departing/Searching food intent; DisposeSpoil resumes it with no food arrival/publication draws and no food-trip counter inflation. Successful food Deposit alone can publish; empty food arrivals retain F3 fallback.
- Each tick processes ascending IDs on one seeded stream; one Move/Dig/PickupFood/DepositFood/DisposeSpoil/Wait per worker. Failed ticks roll back terrain, exposure, ledgers, cargo/intent, maps, advice, all observer/counter state and RNG.
- Maps/scratch <=grid size, at most 4,000,000 classifications. Spoil records <=initially solid diggable cells <=15,625. Per-token records fixed-size; no full action/observation/carrier history.
- Positive sample interval; initial/final frames once; compact Snapshot JSON sum <=64 MiB, excluding episode/configuration fields/delimiters. No successful partial run. Execute full horizon after milestones or exhaustion.
- Views and researcher connectivity do not affect decisions/RNG/worker computation. One physical multi-source nest BFS at construction and each successful Dig serves all food records; count researcher work separately.
- No baseline behavior/API change, universal mind/world abstraction, CLI/WASM/browser/ModelKind, dependency, restoration, cross-platform identity promise, scientific campaign, automatic merge or push.

## Review Focus

1. A remote worker remembers a diggable wall another worker opened: stored belief remains valid until local observation, then revises once without learning the cell twice; protected/open reversals fail atomically (Tasks 1/4).
2. Food ID0 and spoil ID0 both exist: wrong-kind handling must never resolve the other ledger, and a haul must neither publish nor erase the paused food target (Tasks 3/4).
3. A late worker fails after earlier Dig/exposure/own confirmation/access BFS, disposal or food publication: rollback includes observer caches/milestones and production RNG continuation (Task 4).
4. Outlet is physically reachable but privately Unknown: a full-handed worker explores known frontiers without digging, receiving no authoritative exit field; congestion retains cargo/intent (Tasks 2/4).
5. A Dig connects an already exposed pocket: first exposure, access, pickup and delivery remain distinct; initial access is not a fake tick-zero event, and readonly reads never increment connectivity work (Tasks 3/5).

## Isolation and execution discipline

Design worktree `/Users/nathan/.config/superpowers/worktrees/SugarScape/foraging-4-design`, branch `foraging-4-design`, base `e733b98`; spec commit `09ff783`. Main/other campaigns may advance. Before execution read actual heads/status/AGENTS.md and verify existing isolation; reuse this worktree if clean. Do not reset another checkout, refresh main for documentation, overwrite ignored files or merge concurrent work without a deliberate integration check.

Create root IMPLEMENTATION_PLAN.md only at execution, with five stages matching Tasks 1–5 and required Goal/Success Criteria/Tests/Status fields. Parent completes each stage after independent spec/quality review. Keep Stage 5/tracker through whole-branch review; archive results/rulings and remove tracker only after that gate.

Per task: study pertinent F1/F3/Burrow code, write tests first, await and inspect completed RED before implementation, save exact output/exit, implement minimally, GREEN, self-review and fresh task reviews, then commit reviewed working increments. Label compilation reds and supplemental coverage honestly. Stop after three failed attempts per issue, document errors/root cause and reassess; use systematic debugging. No hooks bypass, disabled tests or unsupported TODOs. Focus checks while iterating; full workspace once on final task revision before commit, not every edit.

Keep task briefs/reports/diff packages/progress ledger in the plan-specific ignored SDD workspace. Parent is the only review dispatcher; implementers/reviewers spawn no subagents. Task dependencies are sequential. Evidence-only bookkeeping does not justify repeatedly running successful unchanged-code suites.

## Source ownership

All new module paths below are under `crates/sugarscape-core/src/foraging/construction/`.

| File | Responsibility | Task |
|---|---|---|
| mod.rs | Private declarations/public exports/rustdoc | 1–5 |
| setup.rs | Validated normalized inputs, local geometry/site helpers | 1 |
| terrain.rs | Indexed monotonic physical terrain and dig capacity | 1 |
| observation.rs | Current/cardinal physical projection | 1 |
| knowledge.rs | Dynamic private classifications and atomic revisions | 1 |
| metrics.rs | Checked worker/researcher computation | 1, 5 |
| draws.rs | Checked uniform/PCG/discrete choice | 1 |
| navigation.rs | Private BFS/frontiers/face ranking/approaches | 2 |
| food.rs | Hidden/Available/Carried/Delivered identities | 3 |
| spoil.rs | Direct carried/disposed token identities and fixed history | 3 |
| state.rs | Tagged cargo, food intent, agent, physical counters | 3 |
| server.rs | Food-only F1 advice/lazy expiry | 3 |
| access.rs | Researcher connectivity/cache/bounded milestones | 3 |
| decision.rs | Worker-only mode/action decisions | 4 |
| actions.rs | Terrain/material/advice/observer transactions | 4 |
| controller.rs | Ordered opportunities and dynamic invariants | 4 |
| world.rs | Validated initial state and atomic PCG step | 4 |
| view.rs | Summary/snapshot/single-worker knowledge schemas | 5 |
| runner.rs | Full-horizon bounded run/storage | 5 |
| tests/mod.rs | Shared literal fixtures/scripted draws | 1 |
| tests/setup_learning.rs | Validation/dynamic learning | 1 |
| tests/navigation.rs | Routes/faces/outlet exploration | 2 |
| tests/material_access.rs | Ledgers/advice/access distinction | 3 |
| tests/controller.rs | Cadence/privacy/resumption/atomicity | 4 |
| tests/runner.rs | Limits/views/serialization/replay | 5 |

Modify parent `crates/sugarscape-core/src/foraging/mod.rs` only for the new module and accurate rustdoc; create `crates/sugarscape-core/tests/foraging_construction.rs`. Update docs/foraging.md, the F4 spec/plan and execution tracker for accurate status/evidence. No Cargo dependency or existing F3/Burrow runtime changes.

## Shared type, visibility and fixture contract

Internal alias `Checked<T> = Result<T, Vec<crate::config::FieldError>>`; public signatures spell Vec<FieldError> explicitly. Internal cross-file types/methods are pub(super) as needed. Mutable World fields, agents, maps, terrain, ledgers and observers never become public. Private candidate state derives Clone/PartialEq; public views derive Serialize/PartialEq; positions/enums with no floats use Eq as appropriate.

Reuse F3's public value types without widening its internals:

```rust
pub use crate::foraging::passage::{Parameters, Pos, Resource};
pub struct Setup {
    pub width: u32, pub height: u32,
    pub open: Vec<Pos>, pub diggable: Vec<Pos>, pub nest: Vec<Pos>,
    pub waste: Pos, pub workers: Vec<Pos>, pub food: Vec<Resource>,
    pub parameters: Parameters,
}
// Setup::validate(&self) -> Checked<()>
// Setup::normalized(self) -> Checked<Setup>
// Setup::site(&self, Pos) -> Checked<u64>
// Setup::position(&self, u64) -> Checked<Pos>
// neighbors(Pos, width:u32, height:u32) -> Vec<Pos>, N,S,E,W
struct Terrain {
    width:u32, height:u32, initial_open:Vec<bool>, open:Vec<bool>,
    diggable:Vec<bool>, excavated:u32, capacity:u32,
}
// Terrain::new(&Setup) -> Checked<Terrain>, validates before dense allocation
// Terrain::is_open(&self, Pos) -> Checked<bool>
// Terrain::is_diggable(&self, Pos) -> Checked<bool>, eligibility = solid && mask
// Terrain::was_excavated(&self, Pos) -> Checked<bool>, open now but initially solid
// Terrain::dig(&mut self, Pos) -> Checked<()>, no cargo responsibility here
// Terrain::counts(&self) -> TerrainInventory
// Terrain::open_positions(&self) -> Vec<Pos>, sorted Pos
// Terrain::capacity(&self) -> u32
// Terrain::dimensions(&self) -> (u32,u32)
// Terrain::check(&self) -> Checked<()>
pub struct TerrainInventory { pub initial_open:u32, pub open:u32, pub excavated:u32 }
pub enum CellKnowledge { Unknown, KnownOpen, KnownSolid { diggable:bool } }
pub struct KnownCell { pub pos:Pos, pub kind:CellKnowledge }
struct Knowledge { width:u32, height:u32, cells:Vec<CellKnowledge> }
struct LearnDelta { first:u64, observed_revisions:u64, dig_confirmations:u64 }
// Knowledge::new(width:u32,height:u32) -> Checked<Knowledge>
// Knowledge::dimensions(&self) -> (u32,u32)
// Knowledge::kind(&self, Pos) -> Checked<CellKnowledge>
// Knowledge::learn(&mut self,&Observation) -> Checked<LearnDelta>
// Knowledge::confirm_dig(&mut self, origin:Pos, target:Pos) -> Checked<LearnDelta>
// Knowledge::known(&self) -> Vec<KnownCell>, sorted/excludes Unknown
// Knowledge::counts(&self) -> (u32,u32,u32), open/solid/diggable-solid
struct ObservedCell { pos:Pos, open:bool, diggable:bool, occupants:u32, food:bool }
struct Observation { origin:Pos, cells:Vec<ObservedCell> }
// Observation::validate(&self,width:u32,height:u32) -> Checked<()>, complete
// observe(&Terrain, origin:Pos, &BTreeMap<Pos,u32>, &BTreeSet<Pos>) -> Checked<Observation>
struct ComputeCounts {
    observations:u64, cells_inspected:u64, cells_learned:u64,
    observed_revisions:u64, dig_confirmations:u64,
    route_calls:u64, route_visits:u64, peak_queue:u64,
    frontier_scans:u64, face_scans:u64,
}
struct AccessCompute { calls:u64, visits:u64, peak_queue:u64 }
// both computation structs: checked_include(&mut self,&Self) -> Checked<()>
trait DrawSource { fn uniform(&mut self) -> Checked<f64>; }
struct PcgDraws<'a>(&'a mut crate::rng::SimRng);
// checked_uniform(&mut impl DrawSource) -> Checked<f64>
// draw_index(&mut impl DrawSource,length:u32) -> Checked<u32>
// choose(&[Pos],&mut impl DrawSource) -> Checked<Option<Pos>>
```

Do not invoke private F3 `Pos::neighbors` or `Parameters::information`. Implement the tiny construction-local geometry helper and construct the unchanged CpfaParameters adapter in Task 3 using public parameter fields with omega/lambda_informed zero. Public aliased Parameters::validate is available. Terrain::was_excavated is a read-only physical history predicate for spoil-origin checks; it never enters worker inputs. AccessObserver::check keeps its cache/record validation encapsulated rather than exposing mutable fields to controller.

Fixture in tests/mod.rs:

```rust
fn pos(x:u32,y:u32)->Pos { Pos { x,y } }
fn setup()->Setup {
    Setup {
        width:5,height:3,
        open:vec![pos(0,0),pos(1,0),pos(2,0),pos(0,1)],
        diggable:vec![pos(3,0)],nest:vec![pos(0,0),pos(1,0)],
        waste:pos(0,1),workers:vec![pos(0,0)],
        food:vec![Resource { id:u64::MAX,pos:pos(3,0) }],
        parameters:Parameters { p_search:1.0,p_return:0.0,
            lambda_fidelity:0.0,lambda_publish:0.0,lambda_waypoint:0.0 },
    }
}
#[derive(Clone)]
struct Scripted { values:Vec<f64>,next:usize }
impl Scripted { fn new(values:&[f64])->Self { Self { values:values.to_vec(),next:0 } } }
impl DrawSource for Scripted {
    fn uniform(&mut self)->Checked<f64> {
        let value=self.values.get(self.next).copied()
            .ok_or_else(||vec![FieldError::new("draw","script exhausted")])?;
        self.next+=1; Ok(value) // every consumer checks even a malicious source
    }
}
```

### Task 1: Validated terrain and dynamic local knowledge

**Files:** Create setup.rs, terrain.rs, observation.rs, knowledge.rs, metrics.rs, draws.rs, mod.rs, tests/mod.rs and tests/setup_learning.rs; parent module registration; tracker Stage 1 at execution.
**Consumes:** Public F3 Pos/Parameters/Resource, FieldError, existing SimRng/Serde.
**Produces:** All shared Task 1 signatures above.

- [x] Write tests first for dimensions 2/126, worker 0/257, food257, duplicate/invalid mask/open/nest/food, disconnected/singleton nest, outlet solid/in-nest/disconnected, three same-cell spawns and invalid parameters. Include open outlet not in diggable mask, diggable overlap with initial open cells, protected buried food, disconnected exposed food and IDs0/u64::MAX. Validate original indices before sorting; preserve worker order. Representative tests:

```rust
#[test]
fn open_outlet_need_not_be_diggable() {
    let s=setup(); assert!(!s.diggable.contains(&s.waste));
    assert!(s.validate().is_ok());
}
#[test]
fn remote_wall_memory_waits_for_local_revision() {
    let s=setup(); let mut t=Terrain::new(&s).unwrap();
    let mut a=Knowledge::new(5,3).unwrap(); let mut b=a.clone();
    let o=observe(&t,pos(2,0),&BTreeMap::new(),&BTreeSet::new()).unwrap();
    a.learn(&o).unwrap(); b.learn(&o).unwrap(); t.dig(pos(3,0)).unwrap();
    a.confirm_dig(pos(2,0),pos(3,0)).unwrap();
    assert_eq!(b.kind(pos(3,0)).unwrap(),CellKnowledge::KnownSolid { diggable:true });
    let d=b.learn(&observe(&t,pos(2,0),&BTreeMap::new(),&BTreeSet::new()).unwrap()).unwrap();
    assert_eq!((d.first,d.observed_revisions,d.dig_confirmations),(0,1,0));
}
```

- [x] Run `cargo test -p sugarscape-core foraging::construction::tests::setup_learning`; wait for/inspect red before implementation; record missing-interface failures honestly.
- [x] Implement original-input aggregate validation/normalization and indexed terrain. Mask eligibility is `!open[index] && diggable[index]`; capacity counts initial solid masked cells. Terrain.dig rejects already open/protected/out-of-bounds without changing arrays/counts and prepares checked increment before mutation.

```rust
if self.open[index] || !self.diggable[index] {
    return Err(vec![FieldError::new("terrain.dig","requires a solid masked cell")]);
}
let next=self.excavated.checked_add(1)
    .ok_or_else(||vec![FieldError::new("terrain.excavated","overflow" )])?;
self.open[index]=true; self.excavated=next;
```

- [x] Implement complete current/cardinal observations; open entries have diggable=false, occupancy<=2 and optional available food; solid entries have zero occupants/food and observed mask value. Source lists/material truth are lookup-only inside observe and never passed to decision/navigation. Validate all incoming map updates before mutation: Unknown→observed; equal→unchanged; KnownSolid(true)→Open revision; protected opening, Open→Solid or changed solid mask flag→error. confirm_dig requires adjacent known-open origin and known diggable-solid target, counts own confirmation once; repeat is an error, next ordinary observation adds no revision.
- [x] Add late-batch contradiction rollback, malformed/missing/nonlocal observation cells, boundary ordering, no hidden-food exposure, map size/counts/sorted view, malicious uniform/half-open endpoints, singleton/empty draws, and checked computation overflow tests. Queue peaks use max; all other counts checked sums prepared before assignment.
- [x] Focused GREEN, fmt check, core all-target clippy -D warnings and workspace final task gate; self-review/fresh task reviews, tracker evidence and commit `feat(foraging): add checked construction terrain and local revisions`. Only narrowly named staged dead-code allowances for Tasks 2–4 consumers are allowed; remove all by Task 4.

### Task 2: Private open routes, dig faces and outlet exploration

**Files:** Create navigation.rs and tests/navigation.rs; registrations/Stage 2.
**Consumes:** Task 1 map/observation/dimensions, draws and ComputeCounts; never Terrain/Setup/ledgers.
**Produces:** Exact private contracts:

```rust
enum Navigation { AtGoal,Move(Pos),Blocked,Unreachable }
struct Frontier { pos:Pos,unknown_neighbors:Vec<Pos>,distance:u32 }
struct Face { pos:Pos,approaches:Vec<Pos>,distance:u32 }
// frontiers(&Knowledge,origin:Pos) -> Checked<(Vec<Frontier>,ComputeCounts)>
// faces(&Knowledge,origin:Pos) -> Checked<(Vec<Face>,ComputeCounts)>
// select_frontier(&Knowledge,origin:Pos,site:Option<Pos>,&mut impl DrawSource)
//     -> Checked<(Option<Pos>,ComputeCounts)>
// select_face(&Knowledge,origin:Pos,site:Option<Pos>,&mut impl DrawSource)
//     -> Checked<(Option<Pos>,ComputeCounts)>
// face_approaches(&Knowledge,origin:Pos,face:Pos) -> Checked<(Vec<Pos>,ComputeCounts)>
// route_step(&Knowledge,origin:Pos,goals:&[Pos],&Observation,&mut impl DrawSource)
//     -> Checked<(Navigation,ComputeCounts)>
// wander(origin:Pos,&Observation,&mut impl DrawSource) -> Checked<Navigation>
```

- [x] Write private route/dig/outlet tests: U-shaped known-open path `(1,1),(1,2),(2,2),(3,2),(3,1)` around protected `(2,1)` moves south first; multiple nearest home goals; full immediate steps wait rather than taking longer escape. Build each map using legal one-hop observations. Face tests include unreachable approaches, protected faces, stale walls and newly observed openings. Representative face assertion:

```rust
#[test]
fn only_locally_known_diggable_faces_are_candidates() {
    let s=setup();let t=Terrain::new(&s).unwrap();let mut k=Knowledge::new(5,3).unwrap();
    k.learn(&observe(&t,pos(2,0),&BTreeMap::new(),&BTreeSet::new()).unwrap()).unwrap();
    let (f,_)=faces(&k,pos(2,0)).unwrap();
    assert_eq!(f.iter().map(|f|f.pos).collect::<Vec<_>>(),vec![pos(3,0)]);
}
```

- [x] Await `cargo test -p sugarscape-core foraging::construction::tests::navigation` RED; implement deterministic bounded BFS on KnownOpen. Enumerate steps N,S,E,W; frontier/face candidate order Pos. Frontiers rank `(min_unknown_neighbor_manhattan,known_distance)` for informed goal/outlet; faces rank `(face_manhattan,nearest_approach_distance)`; uninformed selections uniform. Keep nonimproving candidates. Every nonempty uniform selection draws once, including singleton; blocked/empty/AtGoal draw none.
- [x] Use a reverse multi-source known distance field for nearest home/face approaches, filtering occupancy only at next step. Preserve destination outside these pure functions; Task 4 owns retention. Unknown outlet chooses private open frontier, not dig face; any retained outlet frontier uses the same validity rule as food exploration.

```rust
let (width,height)=knowledge.dimensions();
// distances is the bounded reverse-BFS Vec<Option<u32>> computed above.
let eligible:Vec<Pos>=neighbors(origin,width,height).into_iter()
    .filter(|p|distances[(p.y*width+p.x) as usize].is_some_and(|d|d+1==origin_distance))
    .filter(|p|observation.cells.iter().any(|c|c.pos==*p && c.open && c.occupants<2))
    .collect();
// Empty eligible on a known route is Blocked; no global occupancy query.
```

- [x] Pin frontier scans to reachable KnownOpen cells examined; face scans to known solid entries examined; route visits to dequeues and peak queue to max unique queue length. Scratch <=grid; complete actual-dimension observations validated before route draw. wander documents its complete validated-view precondition and checks locally enforceable payloads; decision must validate actual dimensions first.
- [x] Test equality/cumulative boundary draws, empty/unknown/protected goals, malformed observations, remote occupancy irrelevance, one-step approach already reached, and stale-face eligibility lost only after local revision. Verify focused GREEN/fmt/core clippy/workspace; fresh reviews/Stage 2; commit `feat(foraging): navigate private construction frontiers and outlets`.

### Task 3: Tagged cargo, conserved materials, food advice and access

**Files:** Create food.rs, spoil.rs, state.rs, server.rs, access.rs, tests/material_access.rs; registrations/Stage 3.
**Consumes:** Validated Setup/Terrain/Knowledge, F1 information calculations and computation structs.
**Produces:** These contracts (public view types are values; mutable containers internal):

```rust
pub enum Cargo { Food(u64),Spoil(u64) } // Option<Cargo> None = empty
pub enum FoodPhase { Departing,Searching,Returning }
struct Agent {
    id:u32,pos:Pos,phase:FoodPhase,map:Knowledge,cargo:Option<Cargo>,
    find:Option<crate::foraging::FindRecord>,site:Option<Pos>,
    frontier:Option<Pos>,face:Option<Pos>,work:WorkCounts,compute:ComputeCounts,
}
pub enum FoodState { Hidden,Available,Carried { agent:u32 },Delivered }
pub struct FoodView { pub resource:Resource,pub state:FoodState }
pub struct FoodInventory { pub initial:u32,pub hidden:u32,pub available:u32,pub carried:u32,pub delivered:u32 }
struct FoodLedger { records:Vec<FoodView> }
// FoodLedger::new(&Setup,&Terrain) -> Checked<FoodLedger>
// expose(&mut self,Pos) -> Checked<Option<u64>>; Hidden only, None when no food
// claim(&mut self,Pos,agent:u32) -> Checked<Option<u64>>
// deposit(&mut self,id:u64,agent:u32) -> Checked<()>
// available(&self) -> BTreeSet<Pos>; observation builder only
// inventory(&self) -> FoodInventory; views(&self) -> &[FoodView]
// check(&self,&Terrain,&[Agent]) -> Checked<()>
pub enum SpoilState { Carried { agent:u32 },Disposed { tick:u32 } }
pub struct SpoilView { pub id:u64,pub origin:Pos,pub creator:u32,pub born_tick:u32,pub state:SpoilState }
pub struct SpoilInventory { pub excavated:u32,pub carried:u32,pub disposed:u32 }
struct SpoilLedger { records:Vec<SpoilView>,next_id:u64,capacity:u32 }
// SpoilLedger::new(capacity:u32) -> Checked<SpoilLedger>
// spawn(&mut self,origin:Pos,agent:u32,tick:u32) -> Checked<u64>
// dispose(&mut self,id:u64,agent:u32,tick:u32) -> Checked<()>
// inventory(&self) -> SpoilInventory; views(&self) -> &[SpoilView]
// check(&self,&Terrain,&[Agent]) -> Checked<()>
pub struct WorkCounts {
    pub opportunities:u64,pub moves:u64,pub digs:u64,pub pickups:u64,
    pub deposits:u64,pub disposals:u64,pub waits:u64,
    pub departure_moves:u64,pub search_moves:u64,pub empty_return_moves:u64,
    pub food_moves:u64,pub spoil_moves:u64,
    pub transition_waits:u64,pub empty_arrival_waits:u64,pub no_neighbor_waits:u64,
    pub empty_congestion_waits:u64,pub food_congestion_waits:u64,pub spoil_congestion_waits:u64,
    pub search_entries:u64,pub empty_returns:u64,pub fidelity_departures:u64,
    pub recruited_departures:u64,pub uninformed_departures:u64,pub publications:u64,
    pub abandoned_targets:u64,pub spoil_hauls:u64,
}
// WorkCounts::checked_include(&mut self,&Self) -> Checked<()>
// WorkCounts::check(&self) -> Checked<()>, action and move/wait subcategory equations
struct Arrival { departure:crate::foraging::Departure,published:bool }
struct Record { id:u64,site:u64,created_tick:u32 }
struct Server { records:Vec<Record>,next_id:u64,expired:u64 }
struct ServerRecordView { id:u64,site:u64,created_tick:u32,strength:f64 }
// Server::arrive(&Parameters,tick:u32,capacity:u32,Option<FindRecord>,[f64;3]) -> Checked<Arrival>
// Server::views(&Parameters,tick:u32) -> Checked<Vec<ServerRecordView>>
// Server::expired(&self) -> u64
pub struct EventContext { pub tick:u32,pub opportunity:u64,pub worker:u32,
    pub excavated:u32,pub spoil_disposed:u32,pub food_delivered:u32 }
pub struct EventMilestone { pub context:EventContext,pub pos:Pos,pub nest_distance:u32 }
pub struct FoodAccessRecord {
    pub id:u64,pub initially_exposed:bool,pub initially_accessible:bool,
    pub first_exposure:Option<EventMilestone>,pub first_access:Option<EventMilestone>,
    pub accessible:bool,pub distance:Option<u32>,
}
pub struct AccessSummary { pub initially_exposed:u32,pub initially_accessible:u32,
    pub accessible:u32,pub records:Vec<FoodAccessRecord>,pub compute:AccessCompute }
struct AccessObserver { width:u32,height:u32,workers:u32,seen_digs:u32,last_opportunity:u64,
    records:Vec<FoodAccessRecord>,distances:Vec<Option<u32>>,observed_open:Vec<bool>,compute:AccessCompute }
struct AccessDelta { exposure:Option<EventMilestone>,access:Option<EventMilestone> }
// AccessObserver::new(&Setup,&Terrain,&FoodLedger) -> Checked<AccessObserver>
// after_dig(&mut self,&Setup,&Terrain,&FoodLedger,&EventContext) -> Checked<AccessDelta>
// milestone(&self,&EventContext,pos:Pos) -> Checked<EventMilestone>
// summary(&self) -> Checked<AccessSummary>
// distance(&self,pos:Pos) -> Checked<Option<u32>>
// check(&self,&Setup,&Terrain,&FoodLedger) -> Checked<()>, researcher cache/records
pub struct Milestones {
    pub first_excavation:Option<EventMilestone>,pub first_exposure:Option<EventMilestone>,
    pub first_access:Option<EventMilestone>,pub first_disposal:Option<EventMilestone>,
    pub first_pickup_tick:Option<u32>,pub first_delivery_tick:Option<u32>,
    pub all_food_delivered_tick:Option<u32>,
}
```

AccessObserver stores dimensions/population and observed-dig/last-opportunity provenance, plus one private grid-sized observed_open bitmap. Validate the prior cache locally against this stored graph before after_dig; current terrain must differ by exactly one false-to-true eligible excavated cell with no reversals. Stage the new bitmap with the single BFS update. check requires stored openness to equal current terrain. This distinguishes a newly opened cell from corrupted prior cache None entries without a second BFS or history. Include a corrupted-prior-cache rollback test. Validate context with `tick == (opportunity-1)/workers`, `worker == (opportunity-1)%workers`, positive/in-range opportunity and legal complete-tick budget. after_dig requires one terrain increment beyond seen_digs, matching context excavation/food-delivered counts and strictly increasing opportunity, then stages all updates before commit. milestone reads use the stored dimensions/cache and current seen_digs without recording another update. For simultaneous newly accessible foods, aggregate first_access uses the lowest food ID; per-food records preserve every event. Require new observers to start on unexcavated initial terrain with uncarried food; later states use the update method, not re-initialization. Keep AccessObserver's distance cache researcher-only, updated once at construction and each successful Dig; clone/rollback it with State. It must never enter Policy, decisions or navigation. Use it for current physical access/invariant checks without rerunning one BFS per food. ComputeCounts and AccessCompute become public view structs at Task 5; until then fields internal. Do not reexport AccessSummary or other views containing those computational types before Task 5; keep their intermediate declarations within the private module. Task 3 may expose its standalone cargo/material/physical-counter/milestone value types, but never mutable containers or raw server records. Milestones/record flags represent initial access separately from events.

- [x] Write ledger/server/access reds, including equal numeric IDs in separate tagged namespaces and maximal food ID. Expose opens availability without claim/score; wrong-owner/repeated transactions leave containers unchanged. Representative food/spoil assertions:

```rust
#[test]
fn exposure_is_not_collection_or_delivery() {
    let s=setup();let mut t=Terrain::new(&s).unwrap();let mut f=FoodLedger::new(&s,&t).unwrap();
    let mut sp=SpoilLedger::new(t.capacity()).unwrap();
    t.dig(pos(3,0)).unwrap();let sid=sp.spawn(pos(3,0),0,0).unwrap();
    assert_eq!(sid,0);assert_eq!(f.expose(pos(3,0)).unwrap(),Some(u64::MAX));
    assert_eq!(f.inventory(),FoodInventory { initial:1,hidden:0,available:1,carried:0,delivered:0 });
}
```

- [x] Await `cargo test -p sugarscape-core foraging::construction::tests::material_access` RED. Implement staged checked ledger updates; joined cargo checks inspect tags, identities and owner uniqueness. Hidden food requires solid original cell, Available/Carried/Delivered requires open original cell. Spoil origin must be a uniquely excavated cell, carrier equals its creator in direct transport, and disposal tick >=birth. Initial capacity counts masked solid cells, not all mask entries.
- [x] Implement food-only server with public Parameters fields -> CpfaParameters omega/lambda_informed zero, F1 publication/departure, PaperBelow and LaterArgosStrengthWeighted. Validate independent unused draws; pending expiry/publication/departure atomic; retain capacity after lazy expiry, original capacity<=256; no lifetime counter (Task 4 sole deposit derives it). Views do not expire. Cover self-visibility, stale/duplicate sites, empty fallback, capacity257/expired full-store replacement, ID/expiry overflow and literal retention equality through production predicate plus attainable strengths; no float rounding.
- [x] Implement multi-source physical BFS once per new/update observer call. Initial flags/counts are not events; after_dig records new exposures/access at supplied committed-candidate opportunity context and measured physical distances. Example structural pocket: initial open `(0,0),(1,0),(0,1),(3,0)`, nest first two, waste `(0,1)`, food at `(3,0)` initially exposed/inaccessible, masked gap `(2,0)`. Digging gap establishes access to existing Available food without first exposure.

```rust
#[test]
fn connected_exposed_pocket_gets_access_without_exposure_event() {
    let mut s=setup();s.open=vec![pos(0,0),pos(1,0),pos(0,1),pos(3,0)];
    s.diggable=vec![pos(2,0)];let mut t=Terrain::new(&s).unwrap();
    let f=FoodLedger::new(&s,&t).unwrap();let mut a=AccessObserver::new(&s,&t,&f).unwrap();
    t.dig(pos(2,0)).unwrap();
    let ctx=EventContext { tick:0,opportunity:1,worker:0,excavated:1,spoil_disposed:0,food_delivered:0 };
    let delta=a.after_dig(&s,&t,&f,&ctx).unwrap();
    assert!(delta.exposure.is_none());assert_eq!(delta.access.unwrap().nest_distance,2);
}
```

- [x] Test initial accessibility with None event, empty/protected censoring, stable per-food IDs, first milestone frozen after shorter later routes, cached current distances, readonly observer summaries, and separate calls/visits/max peaks. Concrete detour case: nest `(0,0),(1,0)`, waste `(0,1)`, initial open path `(1,1),(1,2),(2,2),(3,2)` and exposed food `(3,0)`; masked gap `(3,1)` establishes first access with distance6, then masked `(2,0)` shortens current distance to2 while first_access distance stays6. Use valid ascending-ID event contexts, for example tick10/opportunity11 then tick20/opportunity21 for one worker. Valid update provenance must be a successful one-cell terrain increment; wrong dimensions/future/inconsistent exposure/cache errors retain observer state. AccessObserver::check validates dimensions, seen-dig/call counts, context/record bounds and cached distance consistency: closed cells None, nest cells zero, every non-nest Some distance has an open predecessor one smaller, adjacent reachable open distances differ at most one, and an open None cell cannot adjoin a Some cell. No extra counted BFS is run by check/views. Work checked sums/equations atomic under overflow.
- [x] Verify focused GREEN/fmt/core clippy/workspace; fresh reviews/Stage 3; commit `feat(foraging): conserve construction materials and record food access`.

### Task 4: Coupled worker decisions and complete atomic ticks

**Files:** Create decision.rs, actions.rs, controller.rs, world.rs, tests/controller.rs; registrations/Stage 4; remove all staged allowances.
**Consumes:** Tasks 1–3 contracts; public F1 departure choices. No global distance field becomes a worker input.
**Produces:** Public World::new/step and these exact internal contracts:

```rust
struct Policy { width:u32,height:u32,nest:Vec<Pos>,waste:Pos,parameters:Parameters }
pub enum Mode { Departing,Searching,EmptyReturning,FoodReturning,SpoilHauling }
enum WaitReason { Transition,EmptyArrival,NoNeighbor,Congestion }
enum Action { Move(Pos),Dig(Pos),PickupFood,DepositFood,DisposeSpoil,Wait(WaitReason) }
struct Decision { agent:Agent,action:Action }
// decide(&Policy,&Agent,&Observation,&mut impl DrawSource) -> Checked<Decision>
struct State {
    tick:u32,terrain:Terrain,agents:Vec<Agent>,food:FoodLedger,spoil:SpoilLedger,
    server:Server,access:AccessObserver,milestones:Milestones,
}
pub struct World { setup:Setup,state:State,rng:crate::rng::SimRng }
// World::new(setup:Setup,seed:u64) -> Result<World,Vec<FieldError>>
// World::step(&mut self) -> Result<(),Vec<FieldError>>
// apply(&Setup,&mut State,index:usize,Decision,&Observation,&mut impl DrawSource) -> Checked<()>
// advance(&Setup,&mut State,&mut impl DrawSource) -> Checked<()>
// check(&Setup,&State) -> Checked<()>
// Agent::mode(&self) -> Mode, derived from tagged cargo before food phase
```

Agent phase/site are the paused food intent during Spoil cargo, not a separately nested resume queue. One frontier field is the current food/outlet open travel commitment; Dig and Dispose clear it/face. Spoil haul must never change food phase/site/find (find is None). Both food and spoil cargo overrides empty decisions. On Dispose clear cargo, keep phase/site, and resume next opportunity without advice or food-trip counts.

- [x] Write phase/privacy/cadence tests first. Helpers initialize valid private states by ordinary observations and typed claims rather than teleporting through full cells or retaining invalid targets. Cover initial one-uninformed-trip/zero-draw construction, Hidden food invisibility, old solid beliefs after other Dig, later local revision, explore-before-face, protected versus stale-diggable informed sites, no opaque role fields, retained blocked face/frontier and source-directed outlet exploration. Add concrete rollback red:

```rust
#[test]
fn late_failure_preserves_candidate_terrain_and_rng() {
    let mut w=World::new(setup(),12).unwrap();let before=w.state.clone();
    let mut d=Scripted::new(&[]);
    assert!(advance_scripted(&mut w,&mut d).is_err());
    assert_eq!(w.state,before);assert_eq!(d.next,0);
}
```

The test helper `advance_scripted(world:&mut World,draws:&mut Scripted) -> Checked<()>` clones State and Scripted and commits both only after successful advance, while production World::step clones World including PCG. Add nontrivial late-failure fixtures described below; the representative empty-script case is not sufficient alone.

- [x] Await `cargo test -p sugarscape-core foraging::construction::tests::controller` RED. Implement constructor normalized inputs, terrain/food/spoil/access and per-worker empty maps/cargo/targets/find; ordinary spawn observations/counts, initial uninformed1, zero physical opportunities and zero draws.
- [x] Implement worker-only decision: validate complete actual-dimension observation/origin; clone/learn only that Agent and add measured compute. Cargo first; then empty food phase. Preserve F3 switch/give-up/action order. Open frontier priority clears face; informed protected target abandons, diggable stale target can pursue known faces. Adjacent fresh eligible face Dig; otherwise route nearest approach. No global mask/food/outlet-distance query, even as a shortcut for empty masks.
- [x] Implement candidate physical apply with separate tagged operations. Dig validates empty hands/face geometry/mask, opens terrain, spawns spoil, exposes food, confirms only own map, increments digs/spoil_hauls, clears frontier/face but keeps food phase/site, updates one researcher BFS and event milestones. Rebuild local projections after earlier actors. No move/pickup/advice in Dig.

```rust
let mut candidate=self.clone();
advance(&candidate.setup,&mut candidate.state,&mut PcgDraws(&mut candidate.rng))?;
*self=candidate;
// Event opportunity is the checked sum of candidate worker opportunities,
// after counting this action, including all waits/earlier actions.
```

- [x] DepositFood alone increments delivery/milestones, then processes three independent arrival draws, publication and food departure, clearing find/cargo/frontier/face. DisposeSpoil owner-checks at outlet, increments only disposal/appropriate physical counters and first-disposal milestone, clears cargo/frontier/face, leaves food phase/site unchanged and consumes zero draws. Entering empty food Returning clears targets/find; resumed Searching after disposal is not a new search entry. Moves/waits categorize by prior derived Mode/cargo, not paused food phase alone.
- [x] Pin full draw table with scripted endpoints/branches:

| Mode/branch | Draws in order |
|---|---|
| Initialization, sensing/revision/own confirmation, observer BFS/readout | None |
| Uninformed Departing switch/equality/no candidates | p_search only; one transition wait |
| Uninformed Departing travel | p_search; frontier choice only when selecting; route-step choice when nonempty |
| Informed Departing at site/protected/impossible | None; one transition wait/search entry |
| Informed known-site travel | route-step choice when nonempty |
| Informed unknown/diggable site, open exploration | new ranked frontier choice if needed; route-step choice when nonempty |
| Informed no-open-frontier excavation | new ranked face choice if needed; route-step choice if travelling; adjacent Dig no movement draw |
| Searching give-up/current-food pickup | p_return only |
| Searching open exploration | p_return; new frontier choice if needed; route-step choice when nonempty |
| Searching dig-face work | p_return; new uniform face choice if needed; route-step choice if travelling; adjacent Dig no movement draw |
| Searching no frontier/face | p_return; wander choice when nonempty |
| Food/empty return outside nest | route-step choice when nonempty |
| Food deposit/empty food arrival | publication, fidelity, recruitment; no movement/frontier draw |
| Spoil unknown outlet | new ranked open-frontier choice if needed; route-step choice when nonempty; never p_return/dig |
| Spoil known outlet | route-step choice when nonempty |
| DisposeSpoil | None |
| Blocked route | No step choice; prior mode/target-selection draws still count |

- [x] Script a full buried-food cycle on shared fixture proving separate Dig, spill-free spoil transport/disposal, later pickup/food travel/deposit. Explicitly record each action/draw assertion rather than assume a random seed wins. Test carried food ignores faces and carried spoil ignores current food; wrong-kind/wrong-destination/full-hand private transactions use clone-and-commit helper so all effects rollback on error.
- [x] Test two workers: earlier Dig reveals availability only in later worker's local view; remote cached wall remains stale/valid; a remembered face opened by another is dropped only after fresh sensing; no failed global-pruning shortcut. Unknown outlet hauling never digs and uses only own known frontiers; full outlet/nest waits retain cargo/site. Shared numeric Food0/Spoil0 cannot cross-handle.
- [x] Test late errors after earlier learning/movement, Dig/exposure/confirmation/access-cache updates, spoil disposal and food publication. Compare full State, Scripted position and production PCG continuation. Use a private checked-counter fault to force production late failure; do not expose an injection API. Every error gets tick/worker/material context; no partial successful run.
- [x] check validates ID/index, positions/capacity, map dimensions/known-open truth, protected wall truth, allowed stale diggable walls, original mask consistency, joined ownership/material/terrain equations, cargo/food-phase/find/targets, work subcategory sums, access cache/records and retained+expired publication totals from sole food deposits. First classifications equal known-cell count; observed revisions/own confirmations are not conflated. Use indexed membership. Budget checks precede mutation; restore cache/milestones with tick rollback.
- [x] Run all construction focus, fmt/core clippy and workspace final revision; verify no remaining staged allowances; self-review/fresh reviews; Stage 4 commit `feat(foraging): couple private excavation and cargo transport atomically`.

### Task 5: Bounded output, baseline reduction and public acceptance

**Files:** Create view.rs, runner.rs, tests/runner.rs and `crates/sugarscape-core/tests/foraging_construction.rs`; expose metrics view fields, public exports/rustdoc; add the test-only PCG probe in `foraging/passage/mod.rs` described below; docs/foraging.md/spec/plan/tracker evidence.
**Consumes:** World/State/check, both ledger/terrain inventories, observer cache/milestones and per-worker counts.

For the no-dig RNG continuation comparison, add one `#[cfg(test)] pub(crate)` free helper in `foraging/passage/mod.rs` returning `[u64;4]` via `RngCore::next_u64` from a cloned F3 World RNG. Keep existing field visibility and production behavior/API unchanged. Call this probe only from construction library unit tests; compare with a cloned construction RNG. This is a verification seam, not a restoration/input API.
**Produces:** Public output below; all Serialize/PartialEq, mutable containers remain private:

```rust
pub struct RunOptions { pub ticks:u32,pub sample_every:u32,pub snapshots:bool }
pub struct KnowledgeView { pub agent:u32,pub cells:Vec<KnownCell> }
pub struct FindView { pub site:Pos,pub count:u32 }
pub struct WaypointView { pub id:u64,pub site:Pos,pub created_tick:u32,pub strength:f64 }
pub struct AgentView {
    pub id:u32,pub pos:Pos,pub phase:FoodPhase,pub mode:Mode,pub cargo:Option<Cargo>,
    pub find:Option<FindView>,pub site:Option<Pos>,pub frontier:Option<Pos>,pub face:Option<Pos>,
    pub known_open:u32,pub known_solid:u32,pub known_diggable:u32,
    pub work:WorkCounts,pub compute:ComputeCounts,
}
pub struct Summary {
    pub completed_ticks:u32,pub food:FoodInventory,pub spoil:SpoilInventory,
    pub terrain:TerrainInventory,pub work:WorkCounts,pub compute:ComputeCounts,
    pub per_agent_work:Vec<WorkCounts>,pub per_agent_compute:Vec<ComputeCounts>,
    pub access:AccessSummary,pub milestones:Milestones,pub expired_records:u64,
}
pub struct Snapshot {
    pub summary:Summary,pub open:Vec<Pos>,pub nest:Vec<Pos>,pub waste:Pos,
    pub agents:Vec<AgentView>,pub food:Vec<FoodView>,pub spoil:Vec<SpoilView>,
    pub waypoints:Vec<WaypointView>,
}
pub struct Episode { pub setup:Setup,pub seed:u64,pub options:RunOptions,
    pub summary:Summary,pub snapshots:Vec<Snapshot>,pub snapshot_bytes:u64 }
// World::summary(&self) -> Result<Summary,Vec<FieldError>>
// World::snapshot(&self) -> Result<Snapshot,Vec<FieldError>>
// World::knowledge(&self,agent:u32) -> Result<KnowledgeView,Vec<FieldError>>
// run(setup:Setup,seed:u64,options:RunOptions) -> Result<Episode,Vec<FieldError>>
// append_snapshot(&mut Vec<Snapshot>,&mut u64,Snapshot,limit:u64) -> Checked<()>; private
```

At this stage make ComputeCounts/AccessCompute public with public fields. Return cloned fixed/bounded records. Knowledge views show remembered classifications/diggability, including stale walls; label their private-belief meaning. No JSON restoration API.

- [x] Write public/runner tests first using shared setup, explicit parameters, seed12/horizon40. Assert replay/accounting/conservation, not guaranteed delivery. Representative test:

```rust
#[test]
fn run_matches_repeated_steps_without_observer_side_effects() {
    let s=setup();let e=run(s.clone(),12,RunOptions { ticks:40,sample_every:7,snapshots:true }).unwrap();
    let mut w=World::new(s,12).unwrap();for _ in 0..40 { w.step().unwrap(); }
    assert_eq!(e.summary,w.summary().unwrap());
    assert_eq!(e.snapshots.first().unwrap().summary.completed_ticks,0);
    assert_eq!(e.snapshots.last().unwrap().summary.completed_ticks,40);
}
```

- [x] Await internal `cargo test -p sugarscape-core foraging::construction::tests::runner` and public `cargo test -p sugarscape-core --test foraging_construction` RED before implementing. Add complete normalized replay episode, aggregated option/setup validation, full horizon and observational views. Invalid knowledge ID is contextual. Summary performs checked worker sums/max peaks and cloned researcher totals without BFS/revision/advice expiry/RNG.
- [x] Implement bounded counting Write before appending compact Snapshot JSON. Test exact serialized-byte cap, one-byte-short error and transactional unchanged frames/bytes; include repeated inventories/access/spoil geometry in count, exclude Episode/setup/options wrapper. Positive sample interval even disabled; disabled returns zero frames/bytes; include initial/final once. Errors produce no successful partial Episode.

```rust
if options.snapshots && (completed % options.sample_every==0 || completed==options.ticks) {
    append_snapshot(&mut frames,&mut bytes,world.snapshot()?,64*1024*1024)?;
}
```

- [x] Cover legal 7200 ticks, 0/7201 rejection, population256 in >=128 connected nest cells with valid unused outlet, 3906/3907 opportunity boundary and single-step cumulative limit. Use validation/helper states rather than repeatedly executing million-opportunity scenes. Check spoil cap calculation including initially open mask entries, map/scratch cap, empty food no fake milestones and unfinished food/spoil at cutoff.
- [x] Compare cadence1/cadence7/disabled summaries and final knowledge; insert readonly methods between steps and compare full State/PCG continuation, including cached access computation and stale weak waypoints. Initial physical opportunities remain zero; constructor worker observations and one researcher BFS count separately. Access distances/first milestones stay frozen where specified.
- [x] Implement no-dig reduction using compatible literal setup: shared open/nest/workers, waste(0,1), food idMAX at initially open(2,0), diggable empty. Convert public F3 value types directly; compare each step's food-state/position/food-phase/cargo projection and food-trip physical counters for seeds0/12/MAX. Compare private-unit PCG continuation after identical steps; extra F4 scans/observer/schema bytes are excluded from equivalence. Repeat with an initially open mask entry to prove it cannot be dug. No global `diggable.is_empty()` flag reaches worker decisions.
- [x] Document complete compiling construction-module rustdoc and public guide: hidden/exposed/access/delivery distinction; one-slot tags, direct waste routing and paused intent; old walls as valid private beliefs; active parameters, costs, limits, output bytes, censoring and deferred roles/relay/science. Preserve F3/Burrow baseline docs/APIs.
- [x] Freeze final source/tests, then save/inspect required final gates once:

```bash
cargo test -p sugarscape-core --test foraging_construction
cargo fmt --all --check
cargo clippy -p sugarscape-core --all-targets -- -D warnings
cargo test --workspace
git diff --check
```

- [x] Fresh Task 5 spec/quality review, then parent whole-branch review against approved spec/plan/evidence. Resolve blocking findings through consolidated scoped corrections with observed reds, greens and fresh re-review. Parent owns tracker completion/removal and evidence archival after final gate; implementer does not spawn reviewers or claim review approval. Commit `feat(foraging): expose bounded construction runs and access outcomes`, then evidence-only archive as appropriate. No feature merge/push/scientific execution here.

## Planning clarification and self-review

Spec coverage: setup/terrain/observation/revision bounds -> Task 1; private routes/faces/outlet -> Task 2; cargo/ledgers/advice/access records -> Task 3; exact priorities/resumption/cadence/draws/atomicity and researcher-cache transaction -> Task 4; views/budgets/no-dig reduction/public documentation and final regression gates -> Task 5. Every Review Focus failure has an owning test step.

Plan type names/fields/signatures and visibility were checked across consuming tasks, including sibling-private F3 methods that cannot be reused directly. Refinements add stored observer dimensions/population/dig/opportunity provenance for checked event contexts, clarify equal-time access tie ordering, and keep computational-view reexports in Task 5 to avoid an intermediate private-interface leak. Agent food intent is preserved in its existing phase/site fields while Spoil cargo overrides it; no duplicated resume stack. Authoritative distance cache belongs only to AccessObserver and never reaches Policy/navigation. Observer computation is counted only on construction/Dig updates, not views. All public metrics expose at Task 5; queue peaks combine by max.

Clarification during planning: the spec's acceptance phrase “protected/solid outlet” meant a solid outlet is invalid whether protected or diggable. An initially open outlet need not be in the diggable mask; its explicit setup contract requires openness/connectivity, not excavatable substrate. The spec wording is corrected accordingly, and Task 1 pins the valid open-unmasked case. This changes no intended physics or architecture.

The user approved the written spec on 2026-10-06. The user approved this written plan on 2026-10-06; subagent-driven execution is complete. Current worktree/base/concurrent work were verified, the baseline and five-stage tracker established, and all five implementation stages completed without repeating approval. Scientific execution remains separate.

## Execution and review evidence — 2026-10-06

Baseline at approved tree: workspace 2,714 passed, 0 failed, 103 ignored. Task1
runtime `848b406`: 12 focused tests; workspace 2,726 passed, 0 failed, 103 ignored;
fmt/core clippy passed. Task review identified mandatory public signature spelling
and missing solid/masked outlet coverage. Scoped fix `96bbf8d` changed only the
two result spellings and one already-correct-behavior fixture; final focused13,
fmt and clippy passed; no redundant workspace repeat. Scoped re-review approved
both findings with no new breakage. Initial RED was missing-interface compilation;
supplemental test timing and frozen revision hashes are disclosed in reports.

Preflight rulings: validate remembered diggability against indexed immutable
setup mask, not current excavation eligibility, so stale opened walls remain
valid; if wrong, belief checks need rework. Add internal excavation-history
predicate and encapsulated AccessObserver::check to authenticate spoil origins
and cached records without field exposure or counted BFS repetition; if wrong,
internal interfaces need rework. Neither ruling adds worker/public knowledge.

Detailed evidence is in the plan-specific ignored SDD workspace and
`/tmp/sugarscape-f4-evidence-20261006/`. Other campaigns remain untouched.

Task2 runtime `d25ee0c`: focused30/workspace2,757 passed,0 failed,103 ignored; fmt/core clippy clean. Independent task review approved with no findings. Observation/learning ordering, retained targets and open-frontier priority remain Task4-owned integration checks.

Additional ruling: permit one cfg(test), crate-visible F3 PCG continuation probe returning four raw draws from a cloned RNG for the Task5 no-dig equivalence test. Sibling-private F3 RNG is otherwise inaccessible. No production API/behavior or field visibility changes; if wrong, remove/rework the test seam.

Task3 ruling: add one private grid-sized observed_open bitmap to the researcher observer. The planned fields cannot distinguish new openness from corrupted old cached None values, so the cache-error rollback requirement needs this provenance. Validate prior cache on its stored graph and exactly one terrain opening before one staged BFS; no worker/public input or history. If wrong, remove/rework this internal field and validation, costing one grid-sized bitmap per observer.

Task3 runtime `133964a`: final79 construction tests/workspace2,793 passed,0 failed,103 ignored; fmt/core clippy clean. Independent task review approved with no findings. Parent independently matched all462 frozen Rust hashes. Controller/RNG/publication integration and public output remain owned by4/5. Supplemental reds, style corrections and corrected one-worker contexts are disclosed in retained report.

Task3 further rulings: reject newly opened cells lacking physical nest distance as impossible worker-Dig provenance while preserving censored initial pockets (if wrong, revise observer primitive/event representation). Validate food-ID order and require new Dig spoil still carried (if wrong, revise local checks). Correct one-worker two-Dig fixtures to include prior disposal; decline an observer disposal lower-bound rule without a spoil-ledger input because coupled physical checks belong to controller (if wrong, revise fixture/validation ownership).

Task4 runtime `d75c2d9`:117 construction tests including38 controller/workspace2,831 passed,0 failed,103 ignored; fmt/core clippy clean. Independent task review approved with no findings, parent matched all467 frozen Rust hashes. Primitive internals rely on prior task reviews; bounded output and full no-dig acceptance remain Task5-owned. All staged allowances removed. Real reconciliation reds, supplemental fixture timing, lint correction and interrupted superseded suite retained in evidence.

Task4 ruling: KnownOpen informed-site travel clears obsolete face only while retaining F3 frontier cadence. Parent checked F3 branch and required direct retained-frontier/cleared-face/action/draw regression. If wrong, small private policy/test rework; no F3 production change.

Task5 runtime `25375c3`: runner14/public6/workspace2,852 passed,0 failed,103 ignored; construction rustdoc/fmt/core clippy/diffcheck clean. Independent task review approved with no findings, including a focused unchanged-helper view-purity check; parent matched all474 frozen files (471Rust plus guide/tracker/spec). Final whole-branch review approved the result; archival is complete. All five implementation task reviews are complete.

## Final review and completion

Whole-branch review of `e733b98..2fa0802` approved merge readiness with no Critical, Important or runtime findings. Its sole minor status-wording item is resolved by this evidence-only archive. All five stages are complete. Final runtime `25375c3` passed workspace2,852/0/103, runner14/public6, construction rustdoc, formatter and core all-target Clippy. Exact commands, frozen-source audits, review reports and supplemental test timing are preserved at `/tmp/sugarscape-f4-evidence-20261006/sdd/`. No runtime change follows that verification. The completed root tracker is removed after preservation; feature branch/worktree were retired after verified local integration and remaining-file preservation. No scientific execution was performed.

### Rulings I made

- Controller dynamic knowledge checks validate remembered diggability against an indexed immutable Setup.diggable set, not Terrain::is_diggable, which is current solid-and-mask eligibility — legitimate stale opened walls otherwise falsely fail; the authoritative index stays outside worker inputs — if wrong, belief validation needs rework; no added knowledge or runtime policy is introduced.
- Add internal Terrain::was_excavated for spoil-origin validation and AccessObserver::check for encapsulated cached-distance/record invariants — existing exact validation duties need physical history and private observer checks without opening fields or rerunning counted BFS — if wrong, internal accessors/checks need rework; worker knowledge and public API remain unchanged.
- Task5 may add one cfg(test) crate-visible F3 PCG probe in passage/mod.rs, returning a fixed array of raw outputs from a cloned World RNG — construction tests cannot access sibling-private F3 RNG but the approved plan requires no-dig continuation comparison; no field visibility, simulation behavior or production API changes — if wrong, remove/rework the test-only seam; original worlds remain unmodified. Apply tracked plan ownership update after Task2 commit/review to avoid dirtying implementer workspace mid-gate.
- Task3 AccessObserver adds private observed_open:Vec<bool> of exactly grid size. Existing dimensions/digcount/distances cannot distinguish new openings from corrupted prior-cache None entries; full planned rollback requires prior graph provenance. Validate old cache locally on stored graph, exactly one terrain false->true increment, then one staged BFS/candidate check. No worker/API/history or extra BFS; if wrong remove/rework field/validation at cost of one grid-sized bitmap. Plan/brief/shared contracts updated during Task3 by parent, implementer informed.
- Task3 provenance — reject newly opened cells with no physical nest distance, since legal worker Digs originate from nest-connected reachable cells and EventMilestone has nonoptional distance. Initial disconnected pockets stay valid/censored. No encoding/API change; if wrong, observer primitive provenance/event representation needs rework. Focused rollback coverage required; record tracked at next parent bookkeeping.
- Task3 self-review invariant — validate strictly ordered food IDs before binary-search transactions; require after-Dig context spoil_disposed < excavated because the new token remains carried. Accepted within planned invariant/provenance checks, targeted supplemental red/green required. If wrong, relax/rework local validation; no legal-worker behavior/API change. Track at next parent bookkeeping.
- Task3 fixture — require one-worker observer detour contexts to record prior spoil disposal before second Dig; fix helper only. Declined additional production disposal lower-bound rule in observer because no spoil ledger is passed and physical/joined checks belong to controller. Await running v2 suite, refreeze corrected fixture and final gates on exact tree; retain timing/evidence. If wrong, test context or observer validation responsibility needs rework. Track at next bookkeeping.
- Task4 KnownOpen informed-site pursuit clears obsolete face only, retaining frontier to match F3 target cadence. Parent checked F3 decision.rs120–126 after implementer raised compatibility concern. Direct test must assert retained frontier/cleared face plus unchanged action/draw behavior; no F3 production change. If wrong, small policy/test rework; authoritative state remains private.
- Final-review boundary — Accept scientific efficacy/biological fidelity/termite comparisons as deferred: this engineering increment supplies deterministic mechanics, not an approved controlled study. If wrong, source/protocol/evaluation work is required.
- Final-review boundary — Accept roles/relay/drops/piles/sharing/mid-load switching as excluded from homogeneous direct transport. If wrong, design and implement new allocation/material/handoff states and costs.
- Final-review boundary — Accept congestion, finite-horizon censoring and no universal completion/fairness guarantee; conservation, cargo retention and paid waits remain judged. If wrong, redesign scheduler/progress policy and tests.
- Final-review boundary — Accept no absolute throughput or resident-memory guarantee; finite grid/map/record/scratch and snapshot-JSON bounds are judged and documented. If wrong, define and verify performance/memory budgets.
- Final-review boundary — Accept no cross-platform/version trajectory identity or universal F3 equivalence beyond compatible no-dig scenes; native replay and specified projections are judged. If wrong, add portability/versioning and broader equivalence work.
- Final-review boundary — Accept no restoration or arbitrary external private-state editing contract; specified corruption/illegal-action rollback is judged. If wrong, add a restoration schema/validation and corruption-coverage contract.
- Final-review boundary — Accept CLI/WASM/browser/ModelKind/persistent migrations/deployment as excluded from this core-only API increment. If wrong, design and implement those integration surfaces.
- Final-review boundary — Accept merge/push/concurrent-tree verification as separate from read-only branch review; verify actual integration after its selection. If wrong, review and verify the actual merged tree before claiming integration.

## Local integration — 2026-10-07

The user explicitly selected local merge. Initial Git writes were blocked by a session policy marking `.git` read-only; after the user restored full access, main and feature working trees were confirmed clean and origin fetched. Main/origin were synchronized at `a69a994`; the reviewed feature ended at `392aba9`. Merge `568a3fa` preserved concurrent main changes and completed without conflicts.

Fresh merged-tree gates passed: `cargo test --workspace` 2,852 passed, 0 failed, 103 existing ignored; `cargo fmt --all --check`; `cargo clippy -p sugarscape-core --all-targets -- -D warnings`. All 533 tracked Rust file hashes remained unchanged during verification. Full commands, output, exits, timing and merge parents are preserved in `/tmp/sugarscape-f4-integration-20261007/`. The tracked-Rust audit includes files beyond the earlier 471-file crate source/test manifest; the merged Rust tree matches the reviewed feature.

The completed SDD evidence had already been archived. The remaining ignored `.superpowers/sdd/.gitignore` was copied and hash-verified before removing only the F4 worktree and deleting its merged branch. This integration record changes only documentation after the verified merge. No push or scientific evaluation was performed. F5 source reconciliation and comparison design are the next increment; role allocation and relay transport retain their separate future design gates.
