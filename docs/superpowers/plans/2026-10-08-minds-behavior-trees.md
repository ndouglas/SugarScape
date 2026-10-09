# Minds Behavior Trees Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user already selected fresh implementers and independent reviewers; do not ask for the execution method again. Steps use checkbox lists.

**Goal:** Add a bounded behavior-tree executor and the approved food-task comparison laboratory, with verified persistence, matched controls and durable evidence before scientific measurement.

**Architecture:** A pure executor owns traversal, budget and action-token semantics. Physical adapters reuse existing movement, utility and GOAP machinery; a default-off laboratory owns task progress and fixed interventions. A separate survey route validates saved evidence and reports all registered contrasts. Stateless book/utility leaf profiles remain separate from task policies.

**Tech Stack:** Existing Rust workspaces, serde/serde_json, rand/rand_pcg, existing survey SHA-256 and paired summaries, wasm-bindgen and Vitest. Enable only the existing rand_pcg serde1 feature for exact RNG-state records; introduce no new package or planning algorithm.

**Spec:** [approved written design](../specs/2026-10-08-minds-behavior-trees-design.md), SHA256 813cbf93fc8446d5e7c52eb4f8798187c71af35d2d442ac33fc5ee4d81e8ac60 at commit8ce8e2d5797d73d6a3710a729320b074cf480275. User approval: “LGTM”. Receipt: .superpowers/sdd/2026-10-08-minds-behavior-trees-design/written-spec-approval.json.

**Planning source:** f439998610630c80699a66bbbf1ce044738c3c74, a normal merge of completed local-main F5 source aa485aa into the isolated design branch. Preserve F5 and all other workflows. Reconcile newer main before execution and final integration; record the actual execution base.

## Global Constraints

- “A tree is a representation and execution mechanism.” Match the guarded FSM's physical/task/RNG projection, not representation-specific hashes. Supplied routines are not learned skills.
- “Walking speed is one.” “Gathering on intervening sites is retained.” One physical attempt per actor turn, including a failed attempt; ordinary metabolism remains once.
- “Exclude s from selection on t+1 and t+2; expire the entry at the start of t+3.” Preserve each site's independent deadline; no free probe or intervention-based early expiry.
- “Physical-action settlement is mandatory and independent of the remaining traversal budget.” Never replace an already executed harvest with zero, undo RNG, delay quota recognition, or issue a second physical action.
- “32 nodes/depth 8 and 64 visits per actor tick.” Conditions are read-only/no-RNG/non-Running. No parallel nodes, arbitrary scripts, background workers or tree editor.
- 11×11 opaque boundary; actor(2,5), holding16, metabolism1, vision8; map prior/span128/share1; A(3,5)=4/cap4, B(7,5)=24/cap24, C(4,9)=24/cap36; no growback or other ecological/social features.
- Quotas20/40;64 completed ticks; reflect x→10−x. Before action3: add12 to C, drain B, or close food-free(5,5); reopen that cell before action7. Stable has no event. Reject an occupied wall-event site; never move the intervention.
- “6 controllers × 4 scenarios × 2 quotas × 2 orientations = 96 cells”;40 seeds30001–30040;3,840 episodes. Construction uses7/8. Hold registered Worlds until actual separate prospective acceptance.
- Four separately labelled64-estimate families: guarded−reactive primary, guarded−Task GOAP secondary, guarded−unguarded ablation, guarded−Legacy GOAP reference. All16 strata/family and all40 labels remain;640 guarded/FSM physical/task/RNG checks.
- Initial holdings do not count toward quota. Stop gathering immediately after completion; common idle hold/metabolism continues. Depleted/quota40 is a known floor; related food/lifetime endpoints are not independent corroboration.
- Incomplete/invalid/pending attempts are unavailable, not censored task failures. No automatic scientific retry/resume/overwrite/seed replacement. In-episode cooldown retry is a different mechanism.
- Keep book default, golden entries and historical disabled-extension hashes unchanged. Closed book_leaf/utility_leaf reduction profiles call their existing action once with no retained tree/task state.
- Preserve all existing sources, binaries, environments, datasets, reports, archives and unrelated work. Never rerun completed P3/P4/Democratic Peace/active-surface campaigns or regenerate their acceptance. F5 belongs to its existing workflow.
- TDD, incremental working commits and normal hooks. Never disable tests or use --no-verify. After three failed attempts on one issue, record failures and reassess before another approach.
- Local-main merge then normal push, no PR. Actual CI/Pages/served hashes and full evidence archival are required; keep scientific source/binary frozen separately from publication integration.

## Review Focus

1. Quota reached by the last budgeted physical leaf: settle actual food/completion/metabolism and halt without a second action or zero surrogate — Tasks1–3.
2. A high-valued failed target, multiple failures and later reopening: preserve t+3 eligibility; scientific attempts still never retry — Tasks2/4.
3. A remembered target versus hidden true stock or a scenario label: choice uses only permitted inputs; compare GOAP connectivity filtering explicitly — Tasks2/3.
4. Interventions after death or task completion, and an occupied blocker: full clock/accounting continues; invalid construction remains an error — Task3.
5. Torn streams, conflicting seed identities and unattained valid tasks: recover an accurate census; never pair a subset or call missing data a censored64 — Tasks4/5.

## File and interface map

Paths beginning with bt/ below mean crates/sugarscape-core/src/minds/behavior_tree/.

| Files | Responsibility / task |
|---|---|
| bt/mod.rs, runtime.rs, runtime_tests.rs, reference_tests.rs | Pure typed executor and independent oracle /1 |
| bt/state.rs, forage.rs, policy.rs, fsm.rs, telemetry.rs, policy_tests.rs | Public observations, policy state, physical/GOAP adapters and work counters /2 |
| core config.rs, minds/mod.rs; core world.rs telemetry field | Named stateless profiles, validation/reset rules, telemetry seam /2 |
| bt/lab.rs, records.rs, runner.rs, lab_tests.rs, runner_tests.rs | Fixed world, interventions, conservation, complete episodes /3 |
| core world.rs, rules/mod.rs, rules/lifecycle.rs, rng.rs; core Cargo.toml/lockfiles | Enabled authoritative state/hash/hooks, exact RNG serialization /3 |
| core tests/behavior_tree.rs, tests/checkpoint.rs | Public construction and persistence /3/5 |
| survey/src/claims/behavior_trees/{mod,manifest,cli,archive,io,validate,report,report_types}.rs | New CLI and saved-data analysis, no shared archive refactor /4 |
| survey/src/claims/behavior_trees/tests/{mod,archive,report,validation,support}.rs; survey/tests/behavior_trees_cli.rs | Actual archive mechanics and pure full-budget reporting tests /4 |
| survey/build_support/bt_source_identity.rs; survey/build.rs | Separate compiled study-input binding /4 |
| docs/superpowers/specs/2026-10-08-minds-behavior-trees-protocol.md | Actual numerical/wire/execution registration produced before science /4 |
| wasm src/lib.rs; web/src/{types,schema,schema.test,determinism.test}.ts | Checked exports and existing selector/native boundary /5 |
| README, papers, roadmap, reproducibility, Minds/program living docs | Engineering/readiness status and links /5 |

The existing F5 build stamp, selectors and study code remain intact. Add an independent BT stamp to survey/build.rs; do not conflate identities. Add only observation calls to legacy utility/GOAP/movement for work counters: their goal, candidates, scores, tie calls, plan retention, fallback and actions remain unchanged. The unchanged legacy policy is not replaced by the Task GOAP adapter. Observe actual results; unavailable failed-search expansion totals have an explicit reason rather than a fabricated zero.

## Execution setup

Use the configured global worktree location ~/.config/superpowers/worktrees/SugarScape/minds-behavior-trees-execution, branched from the user-approved plan commit. Planning work is sparse and is not an execution baseline. Materialize the full execution checkout for builds/provenance; currently tracked source is about401MB.

Use new owned paths for CARGO_HOME, CARGO_TARGET_DIR, npm cache and wasm-pack cache; do not reuse a retained scientific environment. Read cached package sources without changing global locks/config. Record actual toolchain, inherited flags/config, source modes and free space before builds. If storage is insufficient, stop with a recorded resource blocker; do not clean historical artifacts.

Create the five-stage IMPLEMENTATION_PLAN.md in the execution checkout using the user's Goal/Success Criteria/Tests/Status format. Every stage starts Not Started. Maintain the ignored execution ledger at .superpowers/sdd/2026-10-08-minds-behavior-trees/. Before any dispatch, read that ledger and skip completed tasks. Preserve a completed plan snapshot before removing the staging file; retain the scientific evidence workspace.

Reuse a verified baseline only when its actual source/lock/toolchain/flags match. Otherwise run once:
~~~bash
cargo test --workspace --locked --offline
cargo test --manifest-path survey/Cargo.toml --locked --offline
~~~
Record exits/counts/ignored reasons. Do not repeat an unchanged baseline or invoke historical scientific commands. Each task ends with a normal working commit and fresh independent spec/quality review; resolve load-bearing findings before the next task.

---

## Task 1: Pure executor with explicit physical-turn continuation

**Goal:** A World-independent executor with executable budget/settlement semantics.
**Success Criteria:** Reference traces agree, including failures after a consumed token and completion at exhaustion.
**Tests:** Exhaustive short status scripts, token/halting/exhaustion/shape rejection.
**Status:** Not Started.

**Files:** Create bt/mod.rs, runtime.rs, runtime_tests.rs and reference_tests.rs; add only pub mod behavior_tree to minds/mod.rs.

**Interfaces produced** in runtime.rs:
~~~rust
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status { Success, Failure, Running }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Node {
    Condition(u8), Logical(u8), Physical(u8),
    ReactiveSequence(Vec<u8>), ReactiveFallback(Vec<u8>),
    MemorySequence(Vec<u8>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree { pub nodes: Vec<Node> }
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreeState {
    pub cursors: BTreeMap<u8, u8>,
    pub statuses: BTreeMap<u8, Status>,
    pub running_leaves: BTreeSet<u8>,
    pub deferred: Option<u8>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Physical<R> { pub status: Status, pub receipt: R }
#[derive(Clone, Debug, PartialEq)]
pub struct Tick<R> {
    pub status: Status, pub receipt: Option<R>,
    pub visits: u16, pub exhausted: bool, pub deferred_physical: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError { pub message: String }

pub trait Host {
    type Receipt: Clone;
    fn supports(&self, node: &Node) -> bool;
    fn condition(&self, id: u8) -> bool;
    fn logical(&mut self, id: u8) -> Status;
    fn physical(&mut self, id: u8) -> Physical<Self::Receipt>;
    fn settle(&mut self, receipt: &Self::Receipt);
    fn complete(&self) -> bool;
    fn halt(&mut self, id: u8);
}
~~~

Produce Tree::new(nodes:Vec<Node>)->Result<Tree,RuntimeError>, Tree::validate(&self)->Result<(),RuntimeError>, tick<H:Host>(&Tree,&mut TreeState,&mut H,visits:u16)->Result<Tick<H::Receipt>,RuntimeError>, and halt<H:Host>(&Tree,&mut TreeState,&mut H)->Result<(),RuntimeError>. halt validates active IDs, invokes each active leaf once, then clears traversal state without a physical action or RNG draw. Reject empty/over32 nodes, depth>8, out-of-range child IDs, cycles/shared-child ownership/unreachable nodes, invalid saved cursor/status/active-leaf IDs, unsupported host leaves and visits outside1..=64 before any condition/logical/physical/settlement/halt callback. Host::supports is only a read-only leaf-registration probe. Produce TreeState::validate_for_tree(&self,&Tree)->Result<(),RuntimeError>. Root is node0.

A logical activation may span physical turns. Preserve the first unticked child after token/budget deferral. On resumption, reactive ancestors recheck earlier condition guards; do not reissue an already settled physical action solely to reconstruct the traversal. A Running physical leaf can execute its next step on the next turn. Completion/failure ends the activation; new activation starts at root. The independent reference interpreter must model this explicit one-action adaptation, not assume unlimited standard BT actions per tick.

- [ ] **Step1: Write concrete failing tests and an independent scripted host.** The host represents a pure action environment, not a mocked World. Implement this test fixture completely:
~~~rust
use super::runtime::*;
use std::collections::{BTreeMap, VecDeque};

struct Script {
    actions: BTreeMap<u8, VecDeque<(Status, u32)>>,
    attempts: usize, settled: Vec<u32>, halts: Vec<u8>,
    gathered: u32, quota: u32,
}
impl Script {
    fn two_actions(a: u32, b: u32, quota: u32) -> Self {
        Self {
            actions: BTreeMap::from([
                (0, VecDeque::from([(Status::Success, a)])),
                (1, VecDeque::from([(Status::Success, b)])),
            ]),
            attempts: 0, settled: vec![], halts: vec![],
            gathered: 0, quota,
        }
    }
}
impl Host for Script {
    type Receipt = u32;
    fn supports(&self, n: &Node) -> bool {
        match n {
            Node::Condition(id) | Node::Logical(id) => *id == 0,
            Node::Physical(id) => self.actions.contains_key(id),
            _ => true,
        }
    }
    fn condition(&self, _: u8) -> bool { true }
    fn logical(&mut self, _: u8) -> Status { Status::Success }
    fn physical(&mut self, id: u8) -> Physical<u32> {
        self.attempts += 1;
        let (status, receipt) = self.actions.get_mut(&id).unwrap().pop_front().unwrap();
        Physical { status, receipt }
    }
    fn settle(&mut self, r: &u32) { self.settled.push(*r); self.gathered += r; }
    fn complete(&self) -> bool { self.gathered >= self.quota }
    fn halt(&mut self, id: u8) { self.halts.push(id); }
}
fn pair_tree() -> Tree {
    Tree::new(vec![Node::MemorySequence(vec![1, 2]),
                   Node::Physical(0), Node::Physical(1)]).unwrap()
}
#[test]
fn behavior_tree_one_turn_cannot_execute_two_physical_leaves() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(12, 8, 100);
    let r = tick(&pair_tree(), &mut s, &mut h, 64).unwrap();
    assert_eq!((h.attempts, r.receipt, r.status), (1, Some(12), Status::Running));
}
#[test]
fn behavior_tree_last_visit_settles_harvest_before_deferral() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(12, 8, 100);
    let r = tick(&pair_tree(), &mut s, &mut h, 2).unwrap();
    assert_eq!((h.settled, r.receipt, r.exhausted), (vec![12], Some(12), true));
}
#[test]
fn behavior_tree_quota_completion_overrides_unentered_second_action() {
    let mut s = TreeState::default();
    let mut h = Script::two_actions(20, 8, 20);
    let r = tick(&pair_tree(), &mut s, &mut h, 2).unwrap();
    assert_eq!((h.attempts, h.gathered, r.status), (1, 20, Status::Success));
}
~~~
Add real tests for budget1/no action, consumed Failure then deferred fallback, memory continuation, changed reactive guard, exactly-once halt, invalid graph/unsupported leaf, and no repeated settlement. Independent reference code uses an explicit transition table and action log, not the production evaluator; enumerate scripts of length0–4 over all three statuses and budgets1/2/3/64. Length0 uses a condition-only tree; nonempty scripts supply enough action returns for the compared prefix, never an exhausted mock queue masquerading as a runtime error.

- [ ] **Step2: Retain real reds.**
~~~bash
cargo test -p sugarscape-core behavior_tree --locked --offline
~~~
The first red can identify missing API wiring. After minimal types/wiring, retain failing behavioral assertions before implementing token, settlement and continuation logic. Do not count fabricated assertion failures or unrun tests as TDD.

- [ ] **Step3: Implement the executor.** Keep a per-tick local action token, result receipt and visit counter. A physical callback always settles before normal traversal continues:
~~~rust
if physical_used {
    state.deferred = Some(node_id);
    return Status::Running;
}
physical_used = true;
let attempted = host.physical(action_id);
host.settle(&attempted.receipt);
receipt = Some(attempted.receipt);
let leaf_status = attempted.status;
~~~
This is the physical-leaf branch of tick, whose local variables are initialized once per call. Mandatory settlement and ancestor unwind consume no new node entry. Store continuation before attempting an unavailable visit. If host.complete(), settle stop/halt/reset immediately. Distinguish exhausted from a token-only deferral. Serialize active/deferred state, not diagnostic callback logs. Explain the finite unwind bound using depth8.

- [ ] **Step4: Green, self-review and commit.**
~~~bash
cargo test -p sugarscape-core behavior_tree --locked --offline
cargo test -p sugarscape-core --lib --locked --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
git add crates/sugarscape-core/src/minds
git commit -m "Add bounded behavior-tree executor with settled physical turns"
~~~
Inspect the staged path set before commit. Fresh reviewer checks actual exhaustive/reference receipts and every Review Focus1 branch.

## Task 2: Existing physical adapters, closed leaf profiles and matched policies

**Goal:** Working standalone policy adapters with no new fixed-world schedule.
**Success Criteria:** Exact stateless reductions; guarded/FSM decisions and failure clocks agree.
**Tests:** Actual movement/harvest, hidden-stock counterfactuals, exclusion/expiry and task-goal adapter.
**Status:** Not Started.

**Files:** Create bt/state.rs, forage.rs, policy.rs, fsm.rs, telemetry.rs, policy_tests.rs. Modify core config.rs, minds/mod.rs and world.rs (only optional work-counter field/initial None here); add observation-only counter calls in utility.rs, goap/forage.rs and rules/movement.rs. Lab config/initialization are Task3. Do not expose an executable half-built lab. Export public config/wire modules runtime/state and re-export WorkCounters; policy/forage/FSM helpers stay internal except the explicit tree_for_controller validation API.

**Interfaces:** Consume Task1 Tree/TreeState/Host/tick. Produce:
~~~rust
// state.rs; every wire/config type uses serde and denies unknown fields.
pub enum Profile { BookLeaf, UtilityLeaf } // Task3 adds the two task-only profiles.
pub struct Settings { pub profile: Profile, pub visits: u16 }
pub enum Controller {
    ReactiveUtility, GuardedTree, MatchedFsm,
    UnguardedTree, TaskGoap, LegacyGoap,
}
pub enum Scenario { Stable, BetterAlternative, DepletedTarget, TemporaryObstacle }
pub struct LabConfig { pub controller: Controller, pub scenario: Scenario,
                       pub quota: u32, pub mirrored: bool }
pub struct Candidate { pub site: u32, pub pos: crate::geometry::Pos,
                       pub distance: u32, pub value: f64, pub remembered: bool }
pub struct Observation { pub action_tick: u64, pub origin: crate::geometry::Pos,
                         pub quota: u32, pub gross: f64,
                         pub candidates: Vec<Candidate> }
pub struct TaskPlan { pub goal: f64, pub steps: Vec<(crate::geometry::Pos, f64)> }
pub struct TaskState {
    pub quota: u32, pub gross: f64, pub first_completion: Option<u64>,
    pub target: Option<u32>, pub failed_until: std::collections::BTreeMap<u32, u64>,
    pub tree: TreeState, pub fsm: FsmState, pub task_plan: Option<TaskPlan>,
}
pub struct FsmState { pub phase: FsmPhase }
pub enum FsmPhase { Selecting, Moving, Deferred, Finished }
pub struct PhysicalReceipt {
    pub action_tick: u64, pub actor: u64,
    pub origin: crate::geometry::Pos, pub target: crate::geometry::Pos,
    pub destination: crate::geometry::Pos,
    pub gathered: f64, pub route_failed: bool,
}
pub struct Turn {
    pub harvest: crate::rules::Harvest,
    pub receipt: Option<PhysicalReceipt>, pub status: Status,
    pub visits: u16, pub exhausted: bool,
}
pub struct PolicyError { pub message: String }
~~~
Derive Clone/Debug/PartialEq and serde where a type is serialized; all wire enums use snake_case, and serialized structs deny unknown fields; Turn does not need serde because Harvest is an existing non-serde type. Preserve Agent's current contract. TaskState::new(quota:u32)->Self is usable in software tests; LabConfig quotas are restricted to20/40 in Task3.

Expose exact internal functions:
~~~rust
// policy.rs: observation-only selection, no World or scenario argument.
pub(crate) fn expire_failed(s: &mut TaskState, action_tick: u64);
pub(crate) fn note_failure(s: &mut TaskState, site: u32, action_tick: u64)
    -> Result<(), PolicyError>;
pub(crate) fn select_target(o: &Observation, s: &TaskState, rng: &mut crate::rng::SimRng)
    -> Option<u32>;
// forage.rs/fsm.rs: one actual turn; callers apply metabolism.
pub(crate) fn observe(w: &crate::world::World, id: u64, s: &TaskState)
    -> Result<Observation, PolicyError>;
pub(crate) fn act_routine(w: &mut crate::world::World, id: u64,
                         s: &mut TaskState, guarded: bool, visits: u16)
    -> Result<Turn, PolicyError>;
pub(crate) fn act_fsm(w: &mut crate::world::World, id: u64, s: &mut TaskState)
    -> Result<Turn, PolicyError>;
pub(crate) fn act_task_goap(w: &mut crate::world::World, id: u64, s: &mut TaskState)
    -> Result<Turn, PolicyError>;
pub(crate) fn act(w: &mut crate::world::World, id: u64) -> crate::rules::Harvest;
// telemetry.rs: optional per-World measurement, never policy input.
pub struct WorkCounters {
    pub node_visits: u64, pub candidate_evaluations: u64,
    pub target_selections: u64, pub path_queries: u64,
    pub search_expansions: Option<u64>, pub search_unavailable_reason: Option<String>,
    pub fallback_short: u64, pub fallback_limit: u64,
}
pub(crate) fn note_candidates(w: &mut crate::world::World, count: u64);
pub(crate) fn note_selection(w: &mut crate::world::World);
pub(crate) fn note_path(w: &mut crate::world::World);
pub(crate) fn note_search(w: &mut crate::world::World, expanded: Option<u64>,
                         unavailable: Option<&str>);
~~~
WorkCounters derives Clone/Debug/PartialEq and serde. Its explicit Default sets observed operation counts and search_expansions to zero/Some(0), with no unavailable reason; only an actually unavailable result changes that field to None with a reason. World gets bt_work:Option<WorkCounters>, initialized None. Settings defaults to BookLeaf/64; DecisionRule gains BehaviorTree. Only BookLeaf/UtilityLeaf are executable outside a lab. Config reset/schedule validation covers all new settings. UtilityLeaf preserves Utility's combat rejection; BookLeaf preserves Book's allowed reductions.

- [ ] **Step1: Write real failing policy/physical tests.** Create a seed7 one-food software fixture with the existing blank_config/spawn/set_sugar helpers:
~~~rust
use super::{forage::act_routine, policy::{note_failure, expire_failed}, state::TaskState};
fn policy_world() -> (crate::world::World, u64) {
    use crate::{config::{Movement, MoveMode}, testkit::*};
    let mut c = blank_config(11, 11);
    c.movement = Movement { mode: MoveMode::Walk, speed: 1 };
    c.decision.travel = 1.0;
    c.decision.crowding = 0.0;
    let mut w = crate::world::World::new(c, 7).unwrap();
    let id = spawn(&mut w, 2, 5);
    w.agent_mut(id).unwrap().holdings[0] = 16.0;
    w.agent_mut(id).unwrap().metabolism[0] = 1;
    set_sugar(&mut w, 3, 5, 20.0);
    (w, id)
}
#[test]
fn behavior_tree_actual_harvest_completes_quota_on_one_step() {
    let (mut w, id) = policy_world();
    let mut s = TaskState::new(20);
    let t = act_routine(&mut w, id, &mut s, true, 64).unwrap();
    assert_eq!((t.harvest.gathered[0], s.gross, s.first_completion),
               (20.0, 20.0, Some(1)));
}
#[test]
fn behavior_tree_failed_sites_expire_independently_on_t_plus_three() {
    let mut s = TaskState::new(20);
    note_failure(&mut s, 60, 3).unwrap();
    note_failure(&mut s, 61, 4).unwrap();
    expire_failed(&mut s, 6);
    assert_eq!(s.failed_until, std::collections::BTreeMap::from([(61, 7)]));
}
#[test]
fn behavior_tree_hidden_stock_does_not_override_remembered_choice() {
    use crate::{geometry::Pos, minds::memory::Seen, testkit::set_sugar};
    let (mut full, id) = policy_world();
    full.config.memory.span = 128;
    full.config.growback.rate = 0.0;
    set_sugar(&mut full, 3, 5, 0.0);
    set_sugar(&mut full, 7, 5, 24.0);
    let site = full.torus.index(Pos::new(7, 5)) as u32;
    let a = full.agent_mut(id).unwrap();
    a.vision = 1;
    a.remembers = true;
    a.memory.sites.insert(site, Seen::new(&[24.0], &[24.0], 0));
    let mut empty = full.clone();
    empty.site_mut(Pos::new(7, 5)).resource[0] = 0.0;
    let mut left = TaskState::new(20);
    let mut right = TaskState::new(20);
    let a = act_routine(&mut full, id, &mut left, true, 64).unwrap();
    let b = act_routine(&mut empty, id, &mut right, true, 64).unwrap();
    assert_eq!((a.receipt, full.rng), (b.receipt, empty.rng));
}
~~~
Add t+1/t+2 exclusion, all-positive-failed idle, failure on retry, checked expiry overflow, and an independent FSM action/RNG-state equality test using actual cloned World.rng equality. Counterfactual selection holds Observation and RNG fixed while hidden stock/scenario metadata changes elsewhere. Test stale-positive empty arrival separately from physical route failure. Test Task GOAP remaining quota versus unchanged legacy horizon.

- [ ] **Step2: Run reds before adapter bodies.**
~~~bash
cargo test -p sugarscape-core behavior_tree --locked --offline
~~~

- [ ] **Step3: Implement precise adapters and counters.** Target selection preserves candidate order and uses utility::score/ordinary choose, excluding only nonpositive/active-failure entries. Failure update is:
~~~rust
pub(crate) fn note_failure(s: &mut TaskState, site: u32, t: u64)
    -> Result<(), PolicyError>
{
    let until = t.checked_add(3)
        .ok_or_else(|| PolicyError { message: "failed-target expiry overflow".into() })?;
    s.failed_until.insert(site, until);
    s.target = None;
    Ok(())
}
pub(crate) fn expire_failed(s: &mut TaskState, t: u64) {
    s.failed_until.retain(|_, until| t < *until);
}
~~~
Use immutable Observation for conditions; physical callbacks alone borrow World for arrive. Validate a live actor and one-good task environment before effects. The FSM independently implements selecting/moving/deferred/finished transitions; it may share scoring and physical primitives but not call the tree evaluator.

The supplied routine has this exact tree shape (conditions0/1 are task pending/retained target allowed; logical0 is selection; physical0 is move-or-idle):
~~~rust
fn routine_tree() -> Tree {
    Tree::new(vec![
        Node::ReactiveSequence(vec![1, 2, 3]),
        Node::Condition(0),
        Node::Condition(1),
        Node::MemorySequence(vec![4, 5]),
        Node::Logical(0),
        Node::Physical(0),
    ]).unwrap()
}
~~~
Before ticking this tree, expire failures and normalize a newly invalid retained target: guarded mode halts/resets the active routine and clears that target from the same Observation; unguarded mode omits only value invalidation. No new physical attempt occurs during normalization. A target of None is allowed to enter selection. Logical selection stores a target once per activation; if none is eligible, physical0 performs the idle-stay primitive. Arrival or route failure ends the routine and clears its retained target; a Running movement keeps it. This avoids switching between two equivalent moving branches and accidentally halting the target on every turn. Restored tiny-budget continuations retain completed selection rather than drawing again. The independent FSM implements the same normalization and lifetime of a commitment under the registered64-visit configuration; low-budget executor tests use the independent reference interpreter, not fabricated FSM node units.

Borrow traversal state separately from the host's policy fields, or temporarily take it and restore it before returning either success or error. Do not lose TreeState on an early error. Common physical settlement can be shared accounting code; the FSM must not share tree traversal decisions. Produce public tree_for_controller(Controller)->Option<Tree> for strict saved-state validation: only GuardedTree/UnguardedTree return the supplied graph; every other controller returns None and must retain no active tree state.

Task GOAP uses the existing pub(crate) Forage::new and Domain/plan APIs: shortlist K8 using the legacy rate/distance/site order, goal=max(quota−gross,0), preserve the existing abstract costs/heuristic and4,096 cap. Retain/revalidate TaskPlan with the legacy visible-half-value/failed-route rules, rather than copying the guarded tree's rule. Capture Plan.expanded from the actual search. Do not rerun a search to invent measured work. Preserve the legacy search result branch: successful expanded counts are observed; a failed search with no exposed exact count is None with the reason "failed search does not expose exact expansions". This leaves policy/goal behavior unchanged and never substitutes the cap as a measured count.

Stateless act matches the closed profile and directly returns movement::act or utility::act. Work-counter hooks are None-gated observation calls at actual scoring/selection/path/search sites; no altered legacy decision branch or RNG call. A failed search without an exposed expansion count records null and a cause. Conditions do not read work counters. target_selections counts new selection computations (a new tree/FSM target, utility choice, or new GOAP plan/fallback), not each physical request to a retained target; target changes are separately measured from receipts. Reset work counters explicitly each actor turn; do not double-count logical selection and record_choice. Candidate evaluations count actual score/domain-candidate evaluations; path_queries count actual nontrivial pathfinder calls. The constructor/independent researcher scan is not counted as free controller work.

- [ ] **Step4: Green, reductions, self-review and commit.**
~~~bash
cargo test -p sugarscape-core behavior_tree --locked --offline
cargo test -p sugarscape-core --test minds --test goap --test golden --locked --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
git add crates/sugarscape-core/src
git commit -m "Add closed tree profiles and matched food-task policies"
~~~
Before commit inspect exact source scope. Add tests comparing every golden book profile through book versus stateless book_leaf with the same seed/ticks, including combat-enabled book cases; utility_leaf uses unchanged Utility config. Run a full root workspace gate when the new config/dispatch seams warrant it. Fresh review checks no half-built lab and no legacy policy substitution.

## Task 3: Fixed rig, timed interventions and authoritative episode state

**Goal:** A complete checked core laboratory and64-tick runner.
**Success Criteria:** Initial information, timing, conservation and matched projections verified; full persistence.
**Tests:** All96 construction configurations on7/8, strict shape, death/completion/event cases.
**Status:** Not Started.

**Files:** Create bt/lab.rs, records.rs, runner.rs, lab_tests.rs, runner_tests.rs and core tests/behavior_tree.rs. Extend state.rs/mod.rs, config.rs, world.rs, minds/mod.rs, rules/mod.rs and rules/lifecycle.rs. Enable rand_pcg serde1 in core Cargo.toml and inspect root/survey lockfile feature-edge changes; add rng::state_json. Agent remains unchanged.

**Interfaces produced:**
~~~rust
// lab.rs; checked constructor, sorted canonical IDs.
pub fn rig_config(lab: LabConfig) -> crate::config::Config;
pub fn conditions() -> Vec<LabConfig>;
pub fn condition_id(lab: &LabConfig) -> String;
pub(crate) fn initialize(w: &mut crate::world::World);
pub(crate) fn begin_step(w: &mut crate::world::World) -> Result<(), String>;
pub(crate) fn validation_errors(c: &crate::config::Config) -> Vec<crate::config::FieldError>;
// state.rs
pub struct LabRuntime {
    pub task: TaskState, pub living_ticks: u64,
    pub external_added: f64, pub external_removed: f64,
    pub consumed: f64, pub death_loss: f64,
    pub diagnostics: bool, pub controller_timing: bool,
    pub fatal_error: Option<String>, pub errors: Vec<String>,
}
// records.rs; all source/wire enums/types deny unknown fields.
pub struct Cell { pub site: u32, pub food: f64, pub capacity: f64, pub wall: u8 }
pub struct MemoryRecord { pub site: u32, pub levels: Vec<f64>, pub most: Vec<f64>, pub tick: u64 }
pub struct MotionPlan { pub target: Option<crate::geometry::Pos>,
                        pub path: Vec<crate::geometry::Pos>, pub walked: bool }
pub struct Actor { pub id: u64, pub pos: crate::geometry::Pos, pub holdings: f64,
                   pub metabolism: u32, pub vision: u32, pub remembers: bool,
                   pub age: u32, pub max_age: u32, pub memory: Vec<MemoryRecord>,
                   pub motion_plan: MotionPlan }
pub struct LegacyPlan { pub steps: Vec<(crate::geometry::Pos, f64)>,
                        pub goal: f64, pub gathers: f64 }
pub struct Frame {
    pub tick: u64, pub fingerprint: String, pub rng_state_json: String,
    pub actor: Option<Actor>, pub cells: Vec<Cell>, pub task: TaskState,
    pub legacy_plan: Option<LegacyPlan>,
    pub observation: Option<Observation>, pub receipt: Option<PhysicalReceipt>,
    pub work: Option<WorkCounters>,
    pub controller_seconds: Option<f64>,
    pub external_added: f64, pub external_removed: f64,
    pub consumed: f64, pub death_loss: f64,
    pub living_ticks: u64, pub errors: Vec<String>,
}
pub enum UnattainedReason { DiedBeforeQuota, HorizonWithoutQuota }
pub struct EpisodeRecord {
    pub schema: String, pub lab: LabConfig, pub seed: u64,
    pub completed_ticks: u64, pub frames: Vec<Frame>,
    pub quota_attained: bool, pub first_completion: Option<u64>,
    pub restricted_completion_ticks: u64, pub right_censored: bool,
    pub unattained_reason: Option<UnattainedReason>,
    pub gross_gathered: f64, pub living_ticks: u64, pub alive_at_horizon: bool,
    pub errors: Vec<String>,
}
pub struct EpisodeFailure { pub message: String, pub partial: Option<Box<EpisodeRecord>> }
pub struct RunOptions { pub diagnostics: bool, pub controller_timing: bool }
// runner.rs
pub fn run_episode(lab: LabConfig, seed: u64, options: RunOptions)
    -> Result<EpisodeRecord, EpisodeFailure>;
pub fn run_episode_to(lab: LabConfig, seed: u64, options: RunOptions,
                      sink: impl FnMut(&Frame) -> Result<(), String>)
    -> Result<EpisodeRecord, EpisodeFailure>;
pub(crate) fn turn(w: &mut crate::world::World, id: u64)
    -> Option<crate::rules::Harvest>;
pub(crate) fn note_metabolism(w: &mut crate::world::World, consumed: f64);
pub(crate) fn note_removal(w: &mut crate::world::World, id: u64, loss: f64);
pub fn physical_projection(frame: &Frame) -> serde_json::Value;
// rng.rs; same generator/transition, serialization only.
pub fn state_json(rng: &SimRng) -> String;
~~~
Expose public lab/records/runner modules and re-export EpisodeRecord/EpisodeFailure/RunOptions at bt/mod.rs. Declare EPISODE_SCHEMA="minds-behavior-tree-episode-v1" there. World gets behavior_tree_lab:Option<LabRuntime>; Config gets behavior_tree_lab:Option<LabConfig>. Add guarded_rate/unguarded_rate profiles only with a valid lab. All labels map to their named controller; no lab accepts conflicting P3/P4/central/cache settings. Preserve default None initialization and old config decoding.

- [ ] **Step1: Write actual failing construction/clock tests.**
~~~rust
use sugarscape_core::minds::behavior_tree::{lab, runner, records::RunOptions, state::*};

fn stable(controller: Controller, quota: u32) -> LabConfig {
    LabConfig { controller, scenario: Scenario::Stable, quota, mirrored: false }
}
#[test]
fn behavior_tree_initial_prior_contains_levels_not_c_capacity() {
    let w = sugarscape_core::world::World::new(
        lab::rig_config(stable(Controller::GuardedTree, 20)), 7).unwrap();
    let a = w.agent(1).unwrap();
    let site = w.torus.index(sugarscape_core::geometry::Pos::new(4, 9)) as u32;
    assert_eq!(a.memory.sites[&site].levels()[0], 24.0);
}
#[test]
fn behavior_tree_complete_episode_has_all_sixty_five_frames() {
    let r = runner::run_episode(stable(Controller::GuardedTree, 20), 7,
        RunOptions { diagnostics: true, controller_timing: false }).unwrap();
    assert_eq!(r.frames.iter().map(|f| f.tick).collect::<Vec<_>>(),
               (0..=64).collect::<Vec<_>>());
}
#[test]
fn behavior_tree_strict_rig_rejects_unregistered_quota() {
    let mut c = lab::rig_config(stable(Controller::GuardedTree, 20));
    c.behavior_tree_lab.as_mut().unwrap().quota = 21;
    assert!(c.validate().unwrap_err().iter()
        .any(|e| e.field == "behavior_tree_lab.quota"));
}
~~~
The prior test uses the existing Seen::levels() accessor and pins24, not capacity36. Add before-action3/add12 and before-action7/reopen tests, post-completion zero harvest/metabolism, post-death full clock, occupied-block rejection without movement, stale hidden stock, independent metadata/RNG versus diagnostics, failure cursor retention and matched physical projection. Software edge fixtures use7/8 and are explicitly not registered worlds.

Add these real software boundary fixtures in bt/lab_tests.rs; their edited worlds are rejected by scientific initial-state validation:
~~~rust
use super::{lab, state::*};
use crate::{geometry::Pos, world::World};

fn event_world(scenario: Scenario) -> World {
    World::new(lab::rig_config(LabConfig {
        controller: Controller::GuardedTree, scenario, quota: 20, mirrored: false,
    }), 7).unwrap()
}
#[test]
fn behavior_tree_events_continue_after_actor_removal() {
    let mut w = event_world(Scenario::BetterAlternative);
    w.remove(1).unwrap();
    w.run(4);
    assert_eq!((w.tick, w.behavior_tree_lab.as_ref().unwrap().external_added), (4, 12.0));
}
#[test]
fn behavior_tree_occupied_event_stops_before_action_or_tick_increment() {
    let mut w = event_world(Scenario::TemporaryObstacle);
    w.run(2);
    w.move_agent(1, Pos::new(5, 5));
    let before = w.agent(1).unwrap().holdings[0];
    w.step();
    assert_eq!((w.tick, w.agent(1).unwrap().holdings[0]), (2, before));
    assert!(w.behavior_tree_lab.as_ref().unwrap().fatal_error.is_some());
}
~~~
The wall-event test uses a deliberate software relocation only; no scientific record may silently move the actor or blocker. Add a completion-clock fixture by supplying20 at adjacent A and declaring the test-only extra16 in the balance, reaching quota through one real action; then verify block3/open7 and metabolism still occur during the hold. This variant never enters the production manifest.

- [ ] **Step2: Retain reds.**
~~~bash
cargo test -p sugarscape-core behavior_tree --locked --offline
~~~

- [ ] **Step3: Wire the actual core pipeline.** Follow existing checked lab patterns: construct sites before Agent::random/insertion; set actor traits/holding/remembering, then use existing know_the_map after placement. Fix all nonregistered features via full-config validation, including caching.capacity=0 (the existing unlimited-carrying default), growth0, disabled lifespan death and explicit founder age0/max_age128. Every Config field outside the four typed treatment axes must match rig_config. Source tick is completed ticks, so action label is w.tick+1. Call begin_step after apply_schedule and before actor order/turns; it still runs with no living actor. On a rejected event, set LabRuntime.fatal_error and return from World::step before actor ordering/actions/tick increment. A later call with that fatal error cannot resume. World::is_finished returns true for the new fatal gate or the completed64-tick lab horizon; its legacy culture-settlement branch stays unchanged. Extinction alone does not stop the lab clock. The checked runner detects the error/nonprogress immediately and returns the known completed prefix; do not change the legacy step signature or silently continue an invalid episode. Reject occupied wall events without changing the wall/actor or choosing a different site. Hash the fatal gate because it affects continuation.

Dispatch turn only for the valid new lab, otherwise None. Common wrapper expires failures before quota checks, holds completed tasks with Harvest::default, then selects one of the six controllers. Update gross/first_completion during mandatory action settlement before core metabolism. Quota stop halts a Running child once and clears target/traversal/task-plan and legacy goap_plan continuation; preserve the actual attempt target/receipt and the physical motion-plan record. Completion does not rewrite the attempted action. Capture the returned real Harvest; call existing metabolism once. Add None-gated observation hooks using lifecycle's actual consumed value and removal's actual remaining nonnegative food. Preserve old controller ordering and every old enabled/disabled branch.

The conservation assertion is:
~~~rust
let supplied = 52.0 + 16.0 + r.external_added - r.external_removed;
let remaining = world.sites.iter().map(|s| s.resource[0]).sum::<f64>()
    + world.agents().map(|a| a.holdings[0].max(0.0)).sum::<f64>();
let residual = supplied - remaining - r.consumed - r.death_loss;
if !residual.is_finite() || residual.abs() > 1e-9 {
    r.errors.push(format!("food balance residual {residual}"));
}
~~~
Hash all semantic new config/task/tree/FSM/cooldown/plan/progress fields, including Actor.plan target/path/walked used by legacy failure filtering, and exact RNG state only while the lab is active; hash no counters, history or timing. Existing World::clone checkpoint machinery retains new state automatically; prove it. state_json serializes the existing RNG, storing the JSON as a String to preserve128-bit values across JavaScript. Debug is not a state proof.

Record65 frames even after death. physical_projection selects tick, actor traits/memory/position/holding, cells, actual receipt, quota/gross/first-completion/living facts and conservation totals; it excludes representation-specific tree/FSM/task-plan structures, instrumentation and RNG strings. Matched checks compare this projection plus exact canonical rng_state_json separately. Raw frames keep every excluded state for inspection. Parse RNG strings directly into SimRng and reserialize canonically; do not pass the128-bit state through serde_json::Value or a JS Number. Push a known completed frame into the partial DTO before sink I/O. A sink error returns that known partial record and no further step. Native-only controller timing spans the actual controller's observation/decision/physical work; common researcher projections, frame I/O and analysis stay outside. Do not reuse an untimed researcher candidate scan as free BT input. WASM timing is None with a declared platform limitation, not zero.

- [ ] **Step4: Green, full coverage and commit.**
~~~bash
cargo test -p sugarscape-core behavior_tree --locked --offline
cargo test -p sugarscape-core --test checkpoint --locked --offline
cargo test --workspace --locked --offline
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
git add crates/sugarscape-core Cargo.lock survey/Cargo.lock
git commit -m "Add fixed food-task laboratory with authoritative tree state"
~~~
If feature activation changes lock dependency edges, use the existing offline Cargo resolver once, inspect the exact rand_pcg→serde edge, then all checks are locked. Do not upgrade versions. Fresh review gets all96 configs,192 seed7/8 construction records, full96-case transformation and all32 construction guarded/FSM pairs; the640 registered checks remain pending, exact source/partial evidence, no registered effects.

## Task 4: Strict survey collection, saved-data validation and complete reporting

**Goal:** A fresh one-writer archive route and full256-estimate report.
**Success Criteria:** Durable receipt precedes construction; strict census/full pairing; no outcome overrides.
**Tests:** IO/crash prefixes, impossible valid tasks, malformed data, all96×40 pure endpoint coverage.
**Status:** Not Started.

**Files:** Create the survey behavior_trees directory/files in the map, new CLI integration test, bt_source_identity.rs and the actual protocol doc. Modify survey/src/{main,claims/mod}.rs and append the separate build stamp in survey/build.rs. Use existing SHA-256 and stats::paired_summary; change no P3/P4/F5 archive or report.

**Interfaces produced** (all archive wire structs deny unknown fields):
~~~rust
// manifest.rs
pub struct Condition { pub id: String, pub lab: LabConfig }
pub struct Manifest { pub schema: String, pub conditions: Vec<Condition>, pub seeds: Vec<u64> }
pub fn manifest() -> Manifest;
// mod.rs/cli.rs
pub fn route(args: &[String]) -> Option<Result<(), String>>;
pub fn cli(args: &[String]) -> Result<(), String>;
// archive.rs
pub struct FileIdentity { pub path: String, pub sha256: String, pub bytes: u64, pub mode: String }
pub struct Provenance {
    pub source_revision: String, pub protocol: FileIdentity,
    pub binary: FileIdentity, pub source_inventory: Vec<FileIdentity>,
    pub source_sha256: String, pub compiled_inputs_sha256: String,
    pub manifest_sha256: String,
}
pub struct AttemptIdentity {
    pub source_revision: String, pub protocol_sha256: String, pub binary_sha256: String,
    pub source_sha256: String, pub compiled_inputs_sha256: String, pub manifest_sha256: String,
}
pub struct AttemptStart { pub condition: String, pub seed: u64,
                         pub identity: AttemptIdentity, pub started_unix_ms: u128 }
pub enum Outcome { Complete(EpisodeRecord), Failed(EpisodeFailure) }
pub struct OutcomeReceipt { pub condition: String, pub seed: u64, pub outcome: Outcome,
                            pub elapsed_seconds: f64, pub timing_boundary: String }
pub struct AttemptRef { pub condition: String, pub seed: u64,
                        pub start: String, pub frames: String, pub outcome: String }
pub struct Census { pub planned: usize, pub attempted: usize, pub complete: usize,
                    pub failed: usize, pub invalid: usize, pub partial: usize,
                    pub pending: usize, pub unstarted: usize }
pub struct Index { pub schema: String, pub provenance: Provenance, pub manifest: Manifest,
                   pub completed: bool, pub attempts: Vec<AttemptRef>, pub census: Option<Census> }
pub struct Attempt { pub start: AttemptStart, pub frames: Vec<Frame>,
                     pub outcome: Option<OutcomeReceipt>, pub torn_tail: Option<String> }
pub struct Archive { pub index: Index, pub attempts: Vec<Attempt> }
pub fn run(revision: &str, out: &std::path::Path) -> Result<(), String>;
pub fn load(index: &std::path::Path) -> Result<Archive, String>;
// validate.rs
pub fn validate_episode(r: &EpisodeRecord, expected: &LabConfig, seed: u64) -> Result<(), String>;
// report_types.rs/report.rs
pub enum Family { Primary, Secondary, Ablation, LegacyReference }
pub enum Metric { QuotaAttained, RestrictedCompletionTicks, GrossGathered, LivingTicks }
pub struct Endpoint {
    pub condition: String, pub seed: u64, pub quota_attained: bool,
    pub first_completion: Option<u64>, pub restricted_completion_ticks: u64,
    pub right_censored: bool, pub unattained_reason: Option<UnattainedReason>,
    pub gross_gathered: f64, pub living_ticks: u64, pub alive_at_horizon: bool,
}
pub struct Estimate { pub id: String, pub family: Family, pub metric: Metric,
                      pub denominator: usize, pub summary: crate::stats::PairedSummary }
pub struct Analysis {
    pub schema: String, pub census: Census, pub endpoints: Vec<Endpoint>,
    pub estimates: Vec<Estimate>, pub matched_pairs: usize,
    pub duplicate_groups: Vec<Vec<(String, u64)>>,
    pub raw_frame_references: Vec<AttemptRef>,
}
pub fn analyze(a: &Archive) -> Result<Analysis, String>;
pub fn save(a: &Analysis, out: &std::path::Path) -> Result<(), String>;
~~~
Expose a private report helper summarize(m:&Manifest,rows:&[Endpoint])->Result<Vec<Estimate>,String> for pure tests. This helper does not manufacture a validated measured archive. Add named types in report_types.rs:
~~~rust
pub struct CompletionSummary { pub id: String, pub denominator: usize,
    pub summary: Option<crate::stats::PairedSummary>,
    pub unavailable: Vec<(u64, String)> }
pub struct CellDiagnostics { pub condition: String, pub seeds: Vec<u64>,
    pub completed_ticks: Vec<u64>, pub survival_at_64: Vec<bool>, pub work: Vec<Option<WorkCounters>>,
    pub controller_seconds_total: Vec<Option<f64>>,
    pub task_active_calls: Vec<u64>, pub hold_calls: Vec<u64>,
    pub physical_signature: Vec<String> }
pub struct MatchedCheck { pub stratum: String, pub seed: u64,
    pub equal_physical_task: bool, pub equal_rng: bool,
    pub first_difference_tick: Option<u64> }
pub struct AliasGroup { pub conditions: Vec<String>, pub reason: String }
~~~
Analysis additionally contains completion:Vec<CompletionSummary>, cells:Vec<CellDiagnostics>, matched:Vec<MatchedCheck> and aliases:Vec<AliasGroup>. Unrestricted completion summaries are null when any pair is unattained, with every unavailable seed/reason. Report per-invocation and per-episode timing and active/hold call counts, so different survival/hold durations are not mistaken for executor speed. Physical duplicate signatures omit representation/RNG/timing while exact RNG-state identities remain separately retained; unique RNG-bearing fingerprints are not an independence test.

Manifest schema is "minds-behavior-tree-manifest-v1", index/attempt schema is "minds-behavior-tree-archive-v1", and measured analysis schema is "minds-behavior-tree-measured-v1". Scalar test helpers never emit the measured-analysis label. Canonical ID is controller-scenario-quotaN-m0/m1; controller names reactive-utility, guarded-tree, matched-fsm, unguarded-tree, task-goap, legacy-goap; scenario names stable, better-alternative, depleted-target, temporary-obstacle. Serde enum values use snake_case. Schema labels distinguish manifest, raw episodes and measured analysis. Exact source/wire structs use serde with denied unknown fields; enum values are snake_case. No arbitrary tree graph is accepted from the scientific CLI.

- [ ] **Step1: Write failing pure report and actual IO tests.**
~~~rust
#[test]
fn behavior_tree_manifest_has_exact_registered_budget() {
    let m = manifest();
    assert_eq!((m.conditions.len(), m.seeds),
               (96, (30001..=30040).collect::<Vec<_>>()));
}
#[test]
fn behavior_tree_all_four_families_keep_all_pairs() {
    let m = manifest();
    let rows: Vec<Endpoint> = m.conditions.iter().flat_map(|c| m.seeds.iter().map(move |seed| Endpoint {
        condition: c.id.clone(), seed: *seed, quota_attained: false,
        first_completion: None, restricted_completion_ticks: 64,
        right_censored: true, unattained_reason: Some(UnattainedReason::DiedBeforeQuota),
        gross_gathered: 0.0, living_ticks: 16, alive_at_horizon: false,
    })).collect();
    let estimates = summarize(&m, &rows).unwrap();
    assert_eq!((estimates.len(), estimates.iter().all(|e| e.denominator == 40 && e.summary.n == 40)),
               (256, true));
}
~~~
The rows above are explicitly synthetic scalar tests, not actual valid controller trajectories or measured results. Test all family/metric IDs, zero-width intervals/signs, duplicate rows, one missing seed, no favorable subset, independent quotas/orientations and nullable unrestricted time.

Archive tests use actual seed7/8 core records for individual validate_episode/frame-prefix checks. A private declared-construction loader may test saved96×2 records; the public scientific load/CLI still requires the fixed30001–30040 manifest. A collector test injects a fake failure-only episode callback to assert start receipt/frame file existence before any constructor; it never creates a World at a registered seed or presents fake data as valid. Inject real file/frame/envelope/parent-sync errors and test start-only/torn-tail pending versus conflicting well-formed invalid records. Source/binary/protocol/compiled-input mutations must fail before construction.

- [ ] **Step2: Run the genuine missing-route/report/durability reds.**
~~~bash
cargo test --manifest-path survey/Cargo.toml behavior_tree --locked --offline
~~~

- [ ] **Step3: Implement fixed route, durability and validation.** Command forms are exact:
~~~text
survey --behavior-trees --manifest
survey --behavior-trees --help
survey --behavior-trees --run --protocol-revision FULL40HEX --out NEW_DIR
survey --behavior-trees --analyze FINAL_INDEX --out NEW_DIR
~~~
Default prints manifest. Reject duplicates, mixed claims/selectors, extra axes/seeds/threads/fuel/retry/resume flags and malformed revisions. Route BT before other scientific selectors so mixed input is rejected, not accidentally dispatched.

Use new/create-only directories/files, safe relative paths/no symlink traversal, flush+sync file and parent, immutable index.json plus separate final-index.json/census.json. Persist AttemptStart and an empty JSONL stream before constructing each episode. Biological errors get accurate Failed outcomes; hard IO failure stops the collector with its exact prefix/pending/unstarted counts. No overwrite or recovery execution.

Bind actual current_exe hash/mode, clean frozen full Git revision, raw committed protocol bytes, complete tracked source inventory, exact manifest, toolchain/build flags and compiled BT-input hash. The separate build stamp uses this explicit selection contract:
~~~rust
pub const DIRECTORIES: &[&str] = &[
    "crates/sugarscape-core", "survey/src", "survey/build_support",
];
pub const FILES: &[&str] = &[
    "Cargo.toml", "Cargo.lock", "survey/Cargo.toml", "survey/Cargo.lock", "survey/build.rs",
    "docs/superpowers/specs/2026-10-08-minds-behavior-trees-design.md",
    "docs/superpowers/specs/2026-10-08-minds-behavior-trees-protocol.md",
];
pub fn selected(path: &str) -> bool;
pub fn fingerprint(entries: impl IntoIterator<Item = Result<(String, Vec<u8>), String>>)
    -> Result<String, String>;
~~~
Select every regular file under the declared input directories except target, .git, .superpowers, node_modules and __pycache__ directory components, plus the exact FILES. Prefix a versioned namespace, then hash sorted unique UTF-8 relative names/byte lengths/bytes with SHA-256. Reject symlinks/nonregular selected inputs and duplicate names. A missing required source/lock is a build error. Missing not-yet-produced protocol is omitted from the present-file hash/list and encoded with COMPILED_BT_PROTOCOL_PRESENT=false, which makes scientific preflight unavailable. Present empty protocol includes its name/zero length and therefore differs from absence. Emit COMPILED_BT_INPUTS_SHA256, COMPILED_BT_INPUT_PATHS and COMPILED_BT_PROTOCOL_PRESENT in the generated Rust file. Archive preflight includes that file via include!(concat!(env!("OUT_DIR"), "/bt_compiled_inputs.rs")); no fake acceptance flag is introduced. Append bt_compiled_inputs.rs under OUT_DIR after the existing F5 stamp block. The module/build reader is separately unit-tested for source change, ordering, duplicate/path rejection, absent protocol and source-archive builds without Git. This stamp binds the declared study-input closure, not a claim that every unrelated artifact or generic survey dependency was compiled. Preserve the full Git inventory separately. Preflight compares actual source inputs with the embedded stamp, rejecting a stale same-path binary. Record selector scope; do not claim unrelated data/assets were compiled. Preserve the original F5 stamp code/selector/error semantics when appending the independent output.

Saved validation reconstructs physical balances and actions from raw frames without stepping a World: all65 ticks, initial sites/traits/prior/RNG binding, fixed event labels/actions, ≤one physical attempt, adjacent legal movement/capacities, actual harvest/metabolism/death, task progress/first completion/censor flags, cooldown deadlines, token/continuation state and legal plan/guard observations. Check Task/Legacy GOAP's separate goals/candidate filters and exact deterministic path using the existing pure pathfinder where needed. Do not infer action validity merely from endpoint consistency. For every640 guarded/FSM pair compare complete physical/task/RNG projections. Preserve raw input in every invalid/pending disposition.

Analyze only a canonical complete valid archive. All3840 endpoints and4×64 contrasts use exact40 seed sets. Unattained/dead valid task outcomes remain full-denominator data with censored64; a failed/partial scientific episode never becomes that outcome. Store raw-frame references rather than duplicating huge trajectories in analysis.json. Save full tables, exact per-cell/work/timing distributions, duplicate/alias groups and unavailable reasons. No overall Holds/Fails, significance or animal independence.

- [ ] **Step4: Write the committed protocol and verify/commit.** Protocol restates all numerical settings, scene clock, cooldown/budget settlement, information restrictions, task/legacy goal distinction, timing boundaries, raw wire fields, full96×40 declaration,4×64 estimates,640 checks and failure gates. It records no acceptance.
~~~bash
cargo test --manifest-path survey/Cargo.toml --locked --offline
cargo fmt --manifest-path survey/Cargo.toml -- --check
cargo clippy --manifest-path survey/Cargo.toml --all-targets --locked --offline -- -D warnings
git add survey/src/claims/behavior_trees survey/src/claims/mod.rs survey/src/main.rs
git add survey/tests/behavior_trees_cli.rs survey/build.rs survey/build_support/bt_source_identity.rs
git add docs/superpowers/specs/2026-10-08-minds-behavior-trees-protocol.md
git commit -m "Add durable behavior-tree study collection and full paired reporting"
~~~
Fresh review checks actual saved-data failures, parser side-effect freedom, stale binary rejection, synthetic versus construction evidence and raw-state validation. No registered run is permitted by this task.

## Task 5: Native/WASM, broad verification and frozen scientific handoff

**Goal:** Verify the actual implementation and prepare the prospective packet.
**Success Criteria:** Current candidate is independently reviewed with actual source/binary/opportunity identities; scientific hold remains explicit.
**Tests:** Strict exports, every-tick parity, field mutation/checkpoint, defaults and broad gates.
**Status:** Not Started.

**Files:** Modify wasm src/lib.rs, core tests/behavior_tree.rs/tests/checkpoint.rs, web/src/types.ts/schema.ts/schema.test.ts/determinism.test.ts; scoped readiness documentation only. Add no general tree editor or scenario UI.

**Consumes:** Tasks1–4 exact core lab/runner/record APIs. **Produces:**
~~~rust
#[wasm_bindgen]
pub fn behavior_tree_config_json(lab_json: &str) -> Result<String, JsValue>;
#[wasm_bindgen]
pub fn behavior_tree_episode_json(lab_json: &str, seed: &str) -> Result<String, JsValue>;
~~~
Decode with the existing field_errors/decimal_seed mechanism and full rig validation. Canonical parity uses diagnostics=true/controller_timing=false. Pure seed parsing accepts the full u64 range; actual construction fixtures use7/8 only. No JS reimplementation of policy.

- [ ] **Step1: Add red real boundary/checkpoint tests.**
~~~typescript
const labJson = JSON.stringify({
  controller: 'guarded_tree', scenario: 'better_alternative', quota: 40, mirrored: false,
});
const config = JSON.parse(behavior_tree_config_json(labJson));
const episode = JSON.parse(behavior_tree_episode_json(labJson, '7'));
expect(episode.frames.map((f: {tick: number}) => f.tick))
  .toEqual(Array.from({length: 65}, (_, tick) => tick));
await withNativeTraceDirectory(async scratch => {
  const configPath = scratch + '/config.json';
  const tracePath = scratch + '/trace.json';
  writeFileSync(configPath, JSON.stringify(config));
  execFileSync(resolve(root, env.CARGO_TARGET_DIR ?? 'target', 'release/sugarscape'),
    ['run', '--config', configPath, '--seed', '7', '--ticks', '64',
     '--fingerprint-trace', tracePath], {cwd: root, encoding: 'utf8'});
  const native = JSON.parse(readFileSync(tracePath, 'utf8'));
  expect(native.map((r: {tick: number}) => r.tick)).toEqual(episode.frames.map((f: {tick: number}) => f.tick));
});
~~~
Use existing root=fileURLToPath(new URL('../../',import.meta.url)) and native cleanup helper/imports. Extend this test to compare every native fingerprint against episode and Engine.create/advance, not just tick arrays. Cover all six controllers on the construction matrix and both orientations; preserve actor/RNG/quota state while comparing representations correctly. Reject malformed/unknown lab fields, quota21, invalid selector/profile/budget, decimal seed failures and overflow.

Checkpoint at selection/transit/depletion/cooldown/budget deferral/first completion/death; restoration must preserve exact later actions/physical/RNG/fingerprints. Mutate every semantic field individually to prove hash coverage, including legacy plan and failed-site deadlines. Diagnostic/timing toggles must not alter physical/RNG/hash behavior. Pin default-off legacy goldens.

- [ ] **Step2: Run red boundary checks, then implement only adapters/type support.** The Cargo command uses repository cwd; the Vitest command uses cwd=web.
~~~bash
cargo test -p sugarscape-core --test behavior_tree --test checkpoint --locked --offline
npx vitest run src/determinism.test.ts -t "behavior tree"
~~~
Add the selector option using the existing decision schema with BookLeaf default and a closed BookLeaf/UtilityLeaf choice; task profiles remain checked research configs. Update TS config shapes and help explaining supplied routines. Keep existing selectors/defaults/goldens intact. No synthetic preview is published as measured.

- [ ] **Step3: Final actual candidate gates.**
~~~bash
cargo fmt --all -- --check
cargo fmt --manifest-path survey/Cargo.toml -- --check
cargo clippy --workspace --all-targets --locked --offline -- -D warnings
cargo clippy --manifest-path survey/Cargo.toml --all-targets --locked --offline -- -D warnings
cargo test --workspace --locked --offline
cargo test --manifest-path survey/Cargo.toml --locked --offline
cargo build -p sugarscape-core --target wasm32-unknown-unknown --locked --offline
wasm-pack test --node crates/sugarscape-wasm
cargo test -p sugarscape-core --release --test book --locked --offline -- --ignored
npm --prefix web run build
npm --prefix web test
~~~
Run sequentially where build locks/output overlap. Invoke the focused web red test from cwd=web as npx vitest run src/determinism.test.ts -t "behavior tree"; all full npm commands use their shown prefix. Actual release WASM generation is required before full web tests. Capture actual argv/cwd/HEAD/env/start/end/exit/stdout/stderr hashes/counts; distinguish existing ignored/browser-only skips. The release WASM build precedes web parity. Use CARGO_TARGET_DIR-aware native paths; no legacy target edits. New failures justify covering reruns; unchanged successful gates do not.

- [ ] **Step4: Freeze, commit and independent whole-branch review.** Readiness docs say implemented/verified with measurement pending. Commit working scoped files using normal hooks. A fresh whole-branch reviewer sees the complete source diff, task reviews and actual gates; route load-bearing fixes through fresh/continued implementers and scoped reviewers.

Create a clean, complete frozen scientific source at a stable new owned evidence path, separate from later publication integration. Build the actual native survey executable there with locked/offline recorded flags. Retain all tracked source identities/modes, protocol/spec/manifest, compiled input selector/hash, lockfiles/toolchain/config/build receipt/binary hash/mode/platform/embedded source root, and all192 declared construction records/streams (96×7/8). Verify initial controls, actual value/perception opportunities, connected-positive filtering, block-site freedom, accounting, controller constraints and matched FSM pairs from saved data without registered outcome pilots. The exhaustive task oracle is software verification, not a source-paper reproduction.

The prospective packet names the exact proposed one-writer command and an actually absent output directory. Acceptance stays null/HOLD until a fresh actual reviewer decides; implementation/task/whole-branch approvals cannot manufacture it.

## Post-engineering scientific workflow

This workflow is part of the study proposal for plan review. Plan approval authorizes following it through its actual gates; approval is not itself prospective acceptance.

1. **Separate prospective gate:** fresh reviewer with no implementation role checks committed96×40/4×64/640 declaration, full construction/opportunity evidence, policy/info/cost controls, clocks/floors/duplicates, actual frozen source/executable/environment and exact fresh-output command. No registered simulation or paired economic estimate occurs in the gate.
2. **One writer after actual acceptance:** durably record the external verdict and command/environment/source/binary bindings before World construction. Execute the fixed native command once; preserve every state/outcome/partial/pending attempt. No automatic retry/resume/seed replacement.
3. **Saved-data verification:** strict complete census and raw validation, then one same-native saved-data reanalysis into a different fresh directory; compare entire analysis/tables byte-for-byte. Invalid/incomplete data stay unavailable.
4. **Real reporting and empirical review:** make all16 family/metric plots as PNG/SVG with8 base/8 reflected rows each; retain all256 exact native chart inputs, all96 cells/3840 endpoints,640 equivalence checks, raw diagnostic/time distributions, aliases/duplicates/limits. Inspect real assets. A fresh empirical reviewer checks full saved trace/accounting/completion/actual inputs and the bounded claims; corrections preserve prior artifacts. No favorable pooling, learned intent or independent-animal claim.
5. **Reviewed integration/publication:** retain scientific candidate unchanged, reconcile newer main, independently review source integration and faithful report/asset copies, verify merged gates, merge locally then push normally without PR. Require actual exact-head CI/Pages/artifact/deployment and hashes of all served scientific assets/archives.
6. **Immutable archival and closure:** preserve exact sources/binaries/locks/configs/toolchain/environments/raw attempts/reanalysis/plotting tools/reviews/all failed history/CI/Pages/served receipts under a new ~/.local/share/sugarscape/evidence/minds-behavior-trees-2026-10-08 root. Verify hashes/modes/full inventory before any owned cleanup. Public lossless archive cutoffs and final local operational receipts are explicit, without self-referential Git/hash cycles. Preserve every earlier study and the execution ledger; record completed status to prevent redispatch.

## Plan self-review record

| Approved requirement | Owning task/check |
|---|---|
| Typed tick/guards/halts/token/deferred settlement |1 scripted/exhaustive reference plus3 real physical settlement |
| Failure exclusions, t+3 expiry and no free probe |2 independent FSM/real-action tests;4 scientific retry rejection |
| Closed stateless switches and golden/RNG reductions |2 config/dispatch;5 all-platform golden boundaries |
| Six controllers and existing versus task GOAP distinction |2 adapters;3 mapping;4 separate report families |
| Exact rig/map levels/capacity/events64/reflection |3 constructors/clock/conservation;4 strict raw validator |
| Permitted information and connectivity limitation |2 pure observations/counterfactual;3 full opportunity evidence;4 source-choice checks |
| Semantic persistence/hash/RNG and nonsemantic instrumentation |3 authoritative state and RNG;5 mutation/replay/diagnostics/parity |
| Budget96×40/families4×64/pairs640/full denominators |4 manifest/pure scalar tests;5 packet; post-gate actual proof |
| Impossible valid task versus missing scientific record |3/4 death/floor/censor/unavailable tests |
| Durable preconstruction/streamed prefix/stale binary |4 real IO and compiled-source tests |
| Actual independent scientific acceptance and evidence/publication/archive |5 hold; separately recorded post-engineering workflow |

All five tasks are Not Started. This plan is a self-reviewed engineering argument to be assessed by the user; no runtime source, new World, scientific acceptance or measured result was created while writing it.
