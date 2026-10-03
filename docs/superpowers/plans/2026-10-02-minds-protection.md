# Minds P3 Protection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user has already selected that method. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** implement and verify a bounded, costly scatter-cache relocation experiment with original-food accounting and a reproducible measured campaign route.

**Architecture:** a default-absent protection laboratory configuration selects fixed fixtures and policies. Agent state holds local exposure memory and one physical relocation intent; the lab overrides selected actions while preserving ordinary turn order, metabolism and death. A separate optional cohort ledger and survey runner collect outcomes without informing the controller.

**Tech Stack:** existing Rust workspace, serde/serde_json, existing survey statistics, wasm-bindgen and Vitest; no new dependencies.

**Spec:** [approved design](../specs/2026-10-02-minds-protection-design.md).
**Protocol:** [proposed exact campaign](../specs/2026-10-02-minds-protection-protocol.md).
**Baseline:** branch `minds-protection-design`, based on main `1223c47`; design committed through `073b1a4` before this plan.

## Global Constraints

- “Default-off must preserve existing state, RNG consumption, hash/checkpoint behavior and native/WASM trajectories.”
- “Reading another agent's `seen` map is forbidden for all experimental controllers.”
- “Each walking turn is a protective action that replaces harvesting, raiding and ordinary cache recovery.”
- “Reburial never creates another eligible source.”
- “Ledger-disabled and ledger-enabled trajectories must agree.”
- “Missing/capped lineage makes cohort outcomes unavailable, never zero or success.”
- “No product UI, trait evolution or broad free-world switch is required for the first deliverable.”
- “Run only the reviewed campaign, then report causal limits and costs alongside successful protection.”
- Construction tests are allowed during implementation. Scientific runs require successful opportunity checks and review of the committed executable manifest; a provenance flag is not approval.
- Follow the existing Rust/serde/testing conventions. Add no new dependency, scheduler, UI preset, statistical verdict or controller family.

## Review Focus

1. A valid owner cue can disagree with a real observer sighting; private observer state must never change policy selection (Tasks 1/3).
2. A cost or metabolism debit can exhaust transported food; intent must not duplicate holdings or resurrect food (Tasks 2/3).
3. Source/destination occupancy can change during an attempt; consume the failed action and cancel without teleporting or retry loops (Task 2).
4. An archive can have correct run counts but wrong seeds, configs, schedules or lineage; reject it before estimating effects (Task 4).
5. A checkpoint during transport must preserve intent, attempts, schedule, RNG and diagnostics without changing continuation (Task 5).

## File structure and ownership

Create `crates/sugarscape-core/src/minds/protection/`:

| File | Responsibility |
| --- | --- |
| `mod.rs` | Public lab types/reexports, small integration seams |
| `state.rs` | Policies, fixture identifiers/configuration, bounded exposure and attempt/intent state |
| `controller.rs` | Local cue, source/destination choice, paid relocation state machine |
| `ledger.rs` | Optional original-food bookkeeping with absorbing transfer outcomes |
| `lab.rs` | Fixed role construction, schedules and per-action overrides |
| `runner.rs` | Checked episode execution and serializable complete records |
| `tests_support.rs` | `#[cfg(test)]` hand-worked helpers shared by protection unit tests |

Modify `agent.rs`, `config.rs`, `world.rs`, `testkit.rs`, `minds/mod.rs`, `minds/caching/mod.rs`, `minds/caching/theft.rs`, `rules/mod.rs`, `rules/movement.rs`, `rules/lifecycle.rs` for precise hooks. Existing `model.rs` checkpoints clone the core world; retain that approach. Add `survey/src/claims/protection.rs`, `protection_archive.rs`, `protection_report.rs` and register the route in `survey/src/claims/mod.rs`/`survey/src/main.rs`. Extend CLI/WASM verification in `crates/sugarscape-core/tests/checkpoint.rs`, `crates/sugarscape-wasm/src/lib.rs`, `web/src/determinism.test.ts`; no frontend controls.

`IMPLEMENTATION_PLAN.md` tracks the five stages below. Update it with each passing task commit and remove it only after all implementation stages are complete. These five tasks run sequentially because their interfaces depend on preceding tasks; each receives a fresh implementer and review under the subagent-driven workflow. Do not spread concurrent edits across shared world/turn files.

### Task 1: configuration, role construction and local exposure memory

**Files:** create protection `mod.rs`, `state.rs`, `lab.rs` (construction only), `controller.rs` (cue only), `tests_support.rs`; modify `agent.rs`, `config.rs`, `world.rs`, `testkit.rs`, `minds/mod.rs`, `minds/caching/mod.rs`.

**Interfaces produced:**

```rust
// state.rs; derive Clone, Debug, PartialEq and serde for authoritative/exported types.
// Policy uses snake_case; Fixture uses internally tagged {kind: snake_case, ...}.
pub enum Policy { Off, Selective, Indiscriminate, Erased }
pub enum Fixture {
    Single { initial_observed: bool, redeposit_observed: bool },
    Mixed { observed_first: bool },
    CueVisibleNonwatcher,
    CueUnseenWatcher,
    Stumble { initial_observed: bool },
}
pub struct LabConfig {
    pub policy: Policy, pub fixture: Fixture, pub mirrored: bool,
    pub reburial_cost: f64, pub discovery: f64,
    pub exposure_span: u64, pub observer_span: u32,
}
// Default = Off, Single{true,false}, false, 0.25, 0, 64, 64.
pub struct Exposure { pub tick: u64, pub exposed: bool }
pub struct Source { pub tick: u64, pub initial_amount: f64, pub attempted: bool }
pub struct ExposureMemory { pub entries: BTreeMap<u32, Exposure> }
impl ExposureMemory {
    pub fn remember(&mut self, site: u32, tick: u64, exposed: bool);
    pub fn sweep(&mut self, now: u64, span: u64);
}
pub enum Stage { ToSource, Retrieve, ToDestination, Deposit }
pub struct Intent { pub source: u32, pub destination: Pos, pub amount: f64, pub stage: Stage }
pub struct ProtectionState {
    pub sources: BTreeMap<u32, Source>, pub exposure: ExposureMemory,
    pub intent: Option<Intent>,
}
// controller.rs
pub(crate) fn perceived_exposure(world: &World, owner: AgentId) -> bool;
// lab.rs
pub fn rig_config(lab: LabConfig) -> Config;
pub(crate) fn initialize(world: &mut World);
pub(crate) fn note_prepared_deposit(world: &mut World, owner: AgentId, site: u32, amount: f64);
// tests_support.rs, cfg(test), never exported in production
pub(crate) fn rig(policy: Policy) -> World;
pub(crate) fn source_site(world: &World) -> u32;
```

Add `Config.protection_lab: Option<LabConfig>` with `serde(default, skip_serializing_if="Option::is_none")`; default None. Add `Agent.protection: Option<ProtectionState>`; None in ordinary constructors/testkit, Some only for the owner in this lab. World construction uses `lab::initialize` instead of `populate` when the lab is present, before the tick-zero snapshot. Generate each role through `Agent::random`, then set the fixed role traits/endowments/positions and insert in id order; no shuffling role positions and no post-start placement edits. Initialize finite resource patches before snapshot.

Validation requires the rig's exact dimensions/population/one good, Book/Walk speed 1, finite nonnegative costs, discovery in [0,1], positive spans, and the specified inactive modules. Reject spatial larders, existing `lab`, central, non-Book decision, Jump, extra goods, arbitrary schedules and unrelated active rules with a field-specific error. Keep ordinary validators unchanged when None. The runner will enforce the fixed campaign values; unit fixtures may use positive shorter spans and varied costs/discovery.

- [x] Write failing pure-memory tests and local-cue tests. For example:

```rust
#[test]
fn protection_exposure_expires_only_after_the_inclusive_span() {
    let mut m = ExposureMemory { entries: BTreeMap::new() };
    m.remember(30, 0, true);
    m.sweep(2, 2);
    assert!(m.entries.contains_key(&30));
    m.sweep(3, 2);
    assert!(m.entries.is_empty());
}
#[test]
fn protection_visible_nonwatcher_is_a_possible_witness() {
    let mut w = rig(Policy::Selective);
    w.agent_mut(2).unwrap().watches = false;
    assert!(perceived_exposure(&w, 1));
}
```

- [x] Add cap eviction test with `MEMORY_CAP+1` entries: oldest tick then lowest site loses; replacing a site does not consume another slot. Test unseen watcher, opaque wall, own-agent exclusion, no watcher-private-state reads, None-config round-trip omission, invalid combinations and exact role positions/holdings at tick zero.
- [x] Run `cargo test -p sugarscape-core protection_`; expect compile failures for the new API, then implement memory/cue/checked rig construction and rerun to pass. `perceived_exposure` walks `world.sight(owner.pos, owner.vision)` and checks occupancy only; it never filters on watches/cheater.
The memory implementation is a small deterministic map:

```rust
impl ExposureMemory {
    pub fn remember(&mut self, site: u32, tick: u64, exposed: bool) {
        self.entries.insert(site, Exposure { tick, exposed });
        if self.entries.len() > crate::minds::memory::MEMORY_CAP {
            let drop = self.entries.iter()
                .min_by_key(|(site, e)| (e.tick, **site))
                .map(|(site, _)| *site).unwrap();
            self.entries.remove(&drop);
        }
    }
    pub fn sweep(&mut self, now: u64, span: u64) {
        self.entries.retain(|_, e| now.saturating_sub(e.tick) <= span);
    }
}
```

- [x] Hook initial positive `caching::bury` into `note_prepared_deposit` after the existing watching operation. Only scheduled preparation owner/site/ticks register a source; reburials never register. Preserve legacy gross flows and default-off fast paths.
- [x] Format, run core tests/strict Clippy, update Stage 1 and commit: `feat(minds): add bounded protection lab and local exposure memory`.

### Task 2: paid physical relocation and turn integration

**Files:** implement `protection/controller.rs`; modify `state.rs`, `rules/movement.rs`, `minds/mod.rs`, `rules/mod.rs`, `world.rs`, `minds/caching/mod.rs`. Keep fixture scheduling for Task 3.

**Consumes:** Task 1's state/rig/cue/source-registration APIs, existing `caching::{reserve,surplus,dig,bury}` and ordinary movement legality.
**Produces:**

```rust
pub(crate) enum WalkOutcome { Arrived, Advanced, Blocked, Unreachable }
// rules/movement.rs: movement/path only, no food/raid/truffle operations
pub(crate) fn walk_without_gather(world: &mut World, id: AgentId, target: Pos) -> WalkOutcome;
// protection/controller.rs
pub(crate) fn act(world: &mut World, owner: AgentId) -> Option<Harvest>;
pub(crate) fn clamp_after_metabolism(world: &mut World, owner: AgentId);
pub(crate) fn cancel(world: &mut World, owner: AgentId, reason: CancelReason);
// state.rs: derive serde; all reasons are exported counts, not controller input
// CancelReason additionally derives Eq, Ord, PartialOrd for sorted count maps.
pub enum CancelReason {
    NoRoom, SourceMissing, Unreachable, Occupied, SurplusExhausted,
    WitnessVisible, Expired, OwnerDied,
}
pub struct SourceEvent {
    pub source: u32, pub started: bool, pub withdrawn: f64,
    pub redeposited: f64, pub cancellation: Option<CancelReason>,
}
pub struct RelocationEvents {
    pub source_events: Vec<SourceEvent>,
    pub starts: u32, pub completions: u32, pub action_ticks: u32,
    pub distance: u32, pub withdrawn: f64, pub redeposited: f64,
    pub burial_cost: f64, pub cancellations: BTreeMap<CancelReason, u32>,
}
```

`act` returns None only when this owner has no protective action this turn. A cancellation after starting consumes the action and returns zero Harvest. No destination/current privacy means no start: let ordinary behavior act, rather than invent a waiting action. Use `Source.attempted` to exhaust a selected source immediately on starting; intent continues to carry the source id. Selective needs a fresh true entry; Erased uses that same predicate; Indiscriminate uses fresh prepared-source metadata; Off never starts. All source spans use inclusive freshness. Expiry during intent cancels it. Source and destination selection obey the spec's oldest-tick/site and distance/site orders.

- [x] Add a failing action sequence test. Task 1's `rig` creates observed Single/source `(3,3)`; use this explicit setup in controller tests (setup mutation is not a campaign schedule):

```rust
#[test]
fn protection_retrieval_walk_and_deposit_are_separate_actions() {
    let mut w = rig(Policy::Selective);
    assert_eq!(caching::bury(&mut w, 1, 12.0), 12.0);
    w.move_agent(2, Pos::new(7, 6));
    w.tick = 8;
    let source = source_site(&w);
    assert!(act(&mut w, 1).is_some());
    assert!(!w.agent(1).unwrap().caches.contains_key(&source));
    assert_eq!(w.agent(1).unwrap().pos, Pos::new(3, 3));
    assert!(act(&mut w, 1).is_some());
    assert_eq!(w.agent(1).unwrap().pos, Pos::new(3, 2));
    assert!(w.agent(1).unwrap().caches.is_empty());
    assert!(act(&mut w, 1).is_some());
    assert!(w.agent(1).unwrap().protection.as_ref().unwrap().intent.is_none());
}
```

- [x] Add behavior tests for partial capacity, no room, source taken before arrival, a blocked source/destination, zero surplus after metabolism, visible witness at deposit, zero/positive/unaffordable cost, retained holdings after cancellation, expiry, no chain relocation, dead owner and destination exclusions. Tests for capacity/consumption must assert actual balances, not just event counters.
- [x] Run `cargo test -p sugarscape-core protection_` red. Extract the existing A* movement/plan/occupied-target arithmetic into a helper used by ordinary `arrive` and `walk_without_gather`; preserve `arrive` gathering and unreachable-memory cleanup order exactly. Ordinary movement RNG/golden traces must remain unchanged. Do not call `arrive` and then undo its harvest.
- [x] Implement the stage machine. At source underfoot, retrieval is this action; reaching a source by walking defers retrieval to the next action. Credit the scalar returned by scatter dig exactly once. Reaching destination by walking defers deposit. Use zero Harvest for all protective actions; after `lifecycle::metabolize`, clamp intent to current surplus before death checks. Suppress generic surplus burial whenever the protection lab is active, including Off; preparation/relocation bury explicitly.
Use this clamping logic after physiology; travel to a source has no carried batch yet:

```rust
pub(crate) fn clamp_after_metabolism(world: &mut World, owner: AgentId) {
    let carried = world.agent(owner).and_then(|a| a.protection.as_ref())
        .and_then(|s| s.intent.as_ref()).and_then(|i| {
            matches!(i.stage, Stage::ToDestination | Stage::Deposit).then_some(i.amount)
        });
    let Some(carried) = carried else { return; };
    let amount = carried.min(caching::surplus(world, owner));
    if amount <= 0.0 {
        cancel(world, owner, CancelReason::SurplusExhausted);
    } else {
        world.agent_mut(owner).unwrap().protection.as_mut().unwrap()
            .intent.as_mut().unwrap().amount = amount;
    }
}
```

- [x] Introduce World optional per-tick relocation events. Initialize them only while lab-enabled, use them for export later, and never hash them. Add a protection domain to `fingerprint` only when lab is Some: config, roles/flags/visions/metabolisms, full exposure/source/intent state, observer seen entries and current cost/discovery settings. Existing cache stocks are already hashed; do not append anything when None. Tick derives immutable schedule phase; hash any mutable phase state actually introduced.
- [x] Run core movement/golden/checkpoint tests and strict Clippy, update Stage 2 and commit: `feat(minds): relocate exposed food through paid physical actions`.

### Task 3: original-food ledger, fixed lab schedules and episode records

**Files:** create `protection/ledger.rs`, `protection/runner.rs`; complete `protection/lab.rs`; modify `protection/mod.rs`, `world.rs`, `rules/mod.rs`, `rules/lifecycle.rs`, `minds/caching/mod.rs`, `minds/caching/theft.rs`.

**Consumes:** Tasks 1/2 APIs and the exact protocol. Controller logic must never receive a ledger reference.
**Produces:**

```rust
pub type CohortId = u32; // source site index, unique in the bounded episode
pub enum Outflow { Consumption, BurialCost, Deposit { site: u32 } }
pub struct CohortBalance {
    pub initial: f64, pub carried: f64, pub cached: BTreeMap<u32, f64>,
    pub consumed: f64, pub transferred: f64, pub cost: f64,
    pub lost_carried: f64, pub lost_cached: BTreeMap<u32, f64>,
}
pub struct Ledger {
    pub owner: AgentId, pub cohorts: BTreeMap<CohortId, CohortBalance>,
    pub unlabelled_carried: f64, pub unlabelled_cached: BTreeMap<u32,f64>,
}
impl Ledger {
    pub fn new(owner: AgentId, holdings: f64) -> Self;
    pub fn prepare(&mut self, source: u32, amount: f64) -> Result<(),String>;
    pub fn withdraw(&mut self, site: u32, amount: f64) -> Result<(),String>;
    pub fn outflow(&mut self, amount: f64, kind: Outflow) -> Result<(),String>;
    pub fn harvest(&mut self, amount: f64) -> Result<(),String>;
    pub fn pilfer(&mut self, site: u32, amount: f64) -> Result<(),String>;
    pub fn lose_owner(&mut self);
    pub fn reconcile(&self) -> Result<(), String>;
}
// lab.rs; called for either role before protection/ordinary mind dispatch
pub(crate) fn scripted_action(world: &mut World, id: AgentId) -> Option<Harvest>;
pub(crate) fn begin_tick(world: &mut World);
// runner.rs: structs derive Clone, Debug, PartialEq, Serialize, Deserialize
pub struct EpisodeRecord {
    pub schema: String, pub lab: LabConfig, pub seed: u64,
    pub requested_ticks: u64, pub completed_ticks: u64,
    pub owner_ticks_alive: u64, pub owner_alive: bool,
    pub ledger_errors: Vec<String>,
    pub thief_transferred: Option<f64>, pub cohorts: Option<Ledger>,
    pub frames: Vec<FrameRecord>, pub fixture_errors: Vec<String>,
}
pub struct FrameRecord {
    pub tick: u64, pub fingerprint: String, pub roles: Vec<RoleRecord>,
    pub relocation: RelocationEvents, pub deaths: Vec<DeathRecord>,
    pub actions: Vec<ActionRecord>, pub current_bury_cost: f64,
    pub restriction_ticks: BTreeMap<AgentId,u64>,
}
pub struct RoleRecord {
    pub id: AgentId, pub pos: Pos, pub holdings: f64,
    pub caches: BTreeMap<u32,f64>, pub protection: Option<ProtectionState>,
    pub seen: Vec<SeenRecord>,
}
pub struct SeenRecord { pub site: u32, pub owner: AgentId, pub amount: f64, pub tick: u64 }
pub struct DeathRecord { pub id: AgentId, pub pos: Pos, pub cause: String }
pub struct ActionRecord {
    pub id: AgentId, pub phase: String, pub action: String,
    pub harvest: f64, pub metabolic_demand: f64, pub metabolic_consumed: f64,
    pub gross_dug: f64, pub gross_buried: f64, pub burial_cost: f64,
    pub source: Option<u32>, pub target: Option<Pos>,
    pub perceived_exposure: Option<bool>, pub actual_watchers: Vec<AgentId>,
    pub raid_site: Option<u32>, pub raid_amount: f64, pub raid_wasted: bool,
    pub discovery_site: Option<u32>, pub discovery_amount: f64,
}
pub fn run_episode(lab: LabConfig, seed: u64, collect_ledger: bool) -> Result<EpisodeRecord,String>;
```

SeenRecord lists are sorted by site/owner; tuple-keyed maps cannot serialize as JSON objects. DeathRecord uses stable lowercase cause strings (`starvation`, `old_age`, `combat`) and captures position before removal; the existing Death struct lacks position and serde. ActionRecord uses phases `preparation`, `relocation`, `observer_hold`, `encounter`, `ordinary` and actions `hold`, `walk`, `prepare_deposit`, `retrieve`, `redeposit`, `foraging`, `cancel` and captures each actual operation once, including watched initial/reburials and ordinary arrivals. The tick-zero frame has empty action/death lists and zero relocation counts. Diagnostics are independent of ledger collection. `restriction_ticks` counts scheduled holds/preparation walking per role; phase boundaries are recoverable from action records. Add `World.protection_actions: Vec<ActionRecord>` only while enabled and reset it each tick alongside relocation events. Never expose those records to controller decisions.

- [ ] Write failing ledger arithmetic test:

```rust
#[test]
fn protection_ledger_preserves_original_food_through_reburial() {
    let mut l = Ledger::new(1, 20.0);
    l.prepare(30, 10.0).unwrap(); // unlabelled holdings 10, source cohort 10
    l.withdraw(30, 10.0).unwrap(); // holdings 20, half tagged
    l.outflow(2.0, Outflow::Consumption).unwrap(); // tagged consumption 1
    l.outflow(8.0, Outflow::Deposit { site: 21 }).unwrap(); // tagged redeposit 4
    l.pilfer(21, 2.0).unwrap(); // cache is half tagged; absorbing transfer 1
    let c = &l.cohorts[&30];
    assert_eq!((c.carried, c.cached[&21], c.consumed, c.transferred), (5.0,3.0,1.0,1.0));
    l.reconcile().unwrap();
}
```

Ledger tracks both labelled and unlabelled carried/cached balances. A cache withdrawal or pilfer allocates its actual amount proportionally to that site's whole pre-operation stock; a holdings outflow uses whole pre-operation holdings. This is necessary because reburial can contain both original and environmental food. The test above checks exactly that mixed-stock case.

- [ ] Add zero/partial/full outflow, rounding remainder, oversized demand, two-cohort mixing, later thief use, unlabelled harvest, owner death and legacy fate-log-off tests. `prepare` subtracts the initial deposit from unlabelled holdings and creates a cohort; it is not another expenditure after ordinary bury. Actual cohort transfers use pre-operation balances and reconcile against engine holdings/stocks after each completed operation/turn.
- [ ] Run ledger tests red; implement proportional allocation in ascending source order with the protocol tolerance. Add `World.protection_ledger: Option<Ledger>` (diagnostic only). Hook each **actual** owner scatter withdrawal, deposit, burial-cost debit, harvest, holdings metabolism debit, pilfer and removal; account once, and cap physiological consumption at available nonnegative holdings. Initial deposits need a preparation-specific ledger operation rather than a normal labelled deposit. Do not record theft twice through raid and common loot helpers.
Ledger mutators reject nonfinite/negative amounts and insufficient balance for a requested transfer. Physiological callers pass the actual capped consumed amount, not demand. Persist any diagnostic failure in `ledger_errors`, mark original-food outcomes unavailable and stop further ledger mutation; continue the same biological trajectory. Never discard that error. Archive validation accepts explicitly unavailable lineage only with its recorded diagnostic reason; it rejects a present but inconsistent ledger or food total.

For holdings/site proportional transfers, implement this reusable allocation kernel. `balances` contains positive labelled balances in source order, followed by unlabelled balance (omit a zero final balance so the rounding remainder goes to the last positive cohort):

```rust
pub(crate) fn proportional_outflow(balances: &[f64], amount: f64) -> Result<Vec<f64>, String> {
    if !amount.is_finite() || amount < 0.0 ||
        balances.iter().any(|b| !b.is_finite() || *b <= 0.0) {
        return Err("invalid protection ledger outflow".into());
    }
    let total: f64 = balances.iter().sum();
    let tolerance = 1e-9 * total.max(1.0);
    if !total.is_finite() || amount > total + tolerance {
        return Err("protection ledger outflow exceeds available food".into());
    }
    if amount == 0.0 { return Ok(vec![0.0; balances.len()]); }
    let debit = amount.min(total); // ledger-only rounding clamp; engine transfer stays exact
    let mut shares = Vec::with_capacity(balances.len());
    let mut left = debit;
    for (i, balance) in balances.iter().enumerate() {
        let share = if i + 1 == balances.len() { left }
            else { (debit * (balance / total)).min(left) };
        shares.push(share);
        left -= share;
    }
    Ok(shares)
}
```

After debiting, validate each resulting balance within the reconciliation tolerance, reporting any meaningful negative remainder as a diagnostic error. Tiny rounding residuals must not alter the engine's own food arithmetic. Add direct tests for empty/zero requests, two cohorts, a rounding remainder, a sub-tolerance engine/ledger discrepancy and oversized/nonfinite requests. Accept only the declared rounding tolerance; compare every resulting cohort against its original balance, never grant a meaningful overdraw.

- [ ] Implement schedules using `begin_tick` before sweep/actions and `scripted_action` before protection/ordinary decisions. Tick 8 sets cost and erases owner exposure for Erased, once via exact tick match; the same settings reach all policies. Holds still metabolize. Scripted preparation walks use `walk_without_gather`; stumble patrol uses normal `movement::arrive`. New cache observations run the ordinary watching path. Every phase/action is recorded; no schedule reads intent or outcome to move the observer.
- [ ] Add deterministic construction checks for the exact protocol: initial deposits/holdings at tick 8, both source cues, all preparation routes, Selective private/observed redeposit actual sightings, owner departure before contact, surviving observer access, mixed order reversal, source-only registration and stumble contact. Add a mirrored-coordinate transformation test without using reflected outcomes to tune the controller. Give fixture failures descriptive errors and keep their records.
- [ ] Implement run_episode for 64 ticks, counting owner live at tick start, preserving death ticks and continuing while the observer lives. Serialize tick zero plus every completed tick and original-food outcomes. Diagnostic off returns None for food lineage; it does not change state/actions. Run paired ledger-on/off and legacy-fate-on/off trace equality tests. Record time outside the biological record in survey execution metadata.
- [ ] Run full core tests/strict Clippy, update Stage 3 and commit: `feat(minds): account original food in fixed protection episodes`.

### Task 4: measured manifest, archive validation and saved-data report

**Files:** create `survey/src/claims/protection.rs`, `protection_archive.rs`, `protection_report.rs`; modify `survey/src/main.rs`, `survey/src/claims/mod.rs`. Reuse `survey/src/stats.rs` without changing its numerical semantics.

**Consumes:** `LabConfig`, `Policy`, `Fixture`, `runner::run_episode`, `EpisodeRecord`; existing `paired_summary(&BTreeMap<u64,f64>, &BTreeMap<u64,f64>) -> Result<PairedSummary,String>`.
**Produces:**

```rust
pub struct Condition { pub id: String, pub panel: String, pub lab: LabConfig }
pub struct Manifest { pub schema: String, pub conditions: Vec<Condition>, pub seeds: Vec<u64> }
pub fn manifest() -> Manifest;
pub fn cli(args: &[String]) -> Result<(), String>;
// protection_archive.rs
pub struct RawRef { pub condition: String, pub seed: u64, pub path: String }
pub struct Index {
    pub schema: String, pub code_revision: String, pub protocol_revision: String,
    pub manifest: Manifest, pub completed: bool, pub runs: Vec<RawRef>,
}
pub fn validate_archive(index: &Index, records: &[EpisodeRecord]) -> Result<(),String>;
// protection_report.rs
pub fn analyze(index: &Index, records: &[EpisodeRecord]) -> Result<Analysis,String>;
pub fn render_results(analysis: &Analysis) -> String;
pub struct Estimate {
    pub id: String, pub metric: String, pub primary: bool,
    pub summary: Option<PairedSummary>, pub unavailable_seeds: Vec<u64>,
}
pub struct Analysis { pub schema: String, pub estimates: Vec<Estimate> }
```

Analysis must also retain per-cell endpoints, secondary metrics, cue/lineage/phase/opportunity validation and duplicate-trajectory counts; add typed rows next to Estimate. It is not sufficient to render only a favorable contrast. Existing PairedSummary/other imported types may need serde derives; retain arithmetic and tests. Own serde wire DTOs if importing a nonserializable diagnostic type would broaden unrelated exports.

- [ ] Write failing manifest tests:

```rust
#[test]
fn protection_manifest_contains_the_complete_registered_matrix() {
    let m = manifest();
    assert_eq!(m.conditions.len(), 192);
    assert_eq!(m.seeds, (10001..=10040).collect::<Vec<_>>());
    let ids: BTreeSet<_> = m.conditions.iter().map(|c| &c.id).collect();
    assert_eq!(ids.len(), m.conditions.len());
}
```

- [ ] Declare canonical IDs such as `single/p=selective/i=observed/r=private/c=0.25/m=0`; seed-qualified filenames use ordinal condition index and numeric seed, never raw slash-bearing IDs. Assert panel counts 64/32/32/64 and all factor combinations. Test help/default manifest does not execute; unknown/duplicate flags, seed overrides, truncated revisions and occupied output paths fail.
- [ ] Run `cargo test --manifest-path survey/Cargo.toml protection_` red, then implement matrix generation and a measured route independent of the Holds/Fails registry. `--run` requires full committed protocol provenance, clean tracked tree and new directory; no overwrite, automatic resume, seed override or hidden scientific pilot.
- [ ] Add archive-validation tests for missing/duplicate/wrong seed, wrong condition/config, path escape, missing raw file, false completion, inconsistent tick counts, an owner-death frame, invalid cue/schedule, nonfinite outcome and broken cohort reconciliation. A frame-only archive with 7,680 arbitrary records must not pass. One affected lineage seed makes that contrast unavailable without silently discarding the seed.
- [ ] Save raw envelope before the next serial episode. Record exact code/protocol revisions, full manifest, times, schema and condition/seed identities. Atomic file replacement can finish a just-created file but must not overwrite an existing completed run. Write completion only when the full matrix is present. Copy only the small provenance/path/CLI conventions from Minds 9; avoid moving or refactoring its established route.
- [ ] Add synthetic **analysis** fixtures with hand-calculated pairs (no generated scientific episode seeds), assert exact mean/sign/interval inputs and original-food denominators, all 64 primary estimates and unavailable-seed reporting. Test serial saved-data reanalysis byte equality of `analysis.json` and `results.md`; paths/times must not inject unstable output. Preserve terminal episodes. Scientific success is not inferred from programmed selectivity or from an interval alone.
- [ ] Run survey tests/format/strict Clippy, update Stage 4 and commit: `feat(survey): archive and analyze the registered protection campaign`.

### Task 5: persistence, native/WASM agreement and execution readiness

**Files:** modify `crates/sugarscape-core/tests/checkpoint.rs`, `crates/sugarscape-wasm/src/lib.rs`, `web/src/determinism.test.ts`; add `crates/sugarscape-core/tests/protection.rs`; update study/roadmap/spec/plan status and remove completed `IMPLEMENTATION_PLAN.md`.

**Consumes:** `rig_config`, all authoritative state, `run_episode`, full measured manifest and archive types.
**Produces:** minimal WASM functions `protection_config_json(lab_json: &str) -> Result<String, JsValue>` and `protection_episode_json(lab_json: &str, seed: &str) -> Result<String, JsValue>` that parses checked LabConfig and a u64 decimal seed, runs the same core episode and serializes EpisodeRecord. No web UI or preset. Native traces use existing CLI `run --config FILE --seed 7 --ticks 64 --fingerprint-trace FILE`; generated config JSON is rig_config output, so World construction/step is shared with the runner.

- [ ] Add a checkpoint integration test for prepared memory and every intent stage: construct `ModelWorld::Sugarscape` with the checked rig, checkpoint/restore at ticks 8, 9, 10 and 11, then run to 64 and compare full fingerprint traces, protection state and series with straight execution. Check ordinary None-config checkpoints and active-ledger clone noninterference. Add state JSON round-trips for every Stage, empty/erased memory and cancellation/death records. Do not create a new disk checkpoint format: the existing API clones worlds.
- [ ] Add fingerprint-sensitivity tests that individually mutate policy, source tick/attempted flag, exposure bit/tick, destination/stage/pending amount, observer seen memory, span and schedule fixture. Each must change an enabled hash. Changing only events, ledger collection or wall-time diagnostics must not. None-config fingerprints retain existing goldens.
- [ ] Write a native/WASM test with seed 7 and base orientation. Instantiate identical rig_config JSON through the existing Sim API and CLI trace route; compare tick-zero and all 64 completed tick fingerprints exactly. Use the new episode_json function to inspect actions/cohorts too. Test observed/private redeposit, Erased, Indiscriminate and one cap/cancellation construction fixture; JSON seed parsing must reject overflow/nondecimal input.
- [ ] Run targeted tests red; implement the small WASM export and complete persistence/hash coverage until they pass. For example, extend the existing Vitest child-process pattern:

```typescript
const root = fileURLToPath(new URL('../../', import.meta.url));
const lab = { policy: 'selective', fixture: {
  kind: 'single', initial_observed: true, redeposit_observed: false,
}, mirrored: false, reburial_cost: 0.25, discovery: 0,
exposure_span: 64, observer_span: 64 };
const config = JSON.parse(protection_config_json(JSON.stringify(lab)));
await withNativeTraceDirectory(async scratch => {
  const configPath = `${scratch}/protection-config.json`;
  const tracePath = `${scratch}/protection-trace.json`;
  writeFileSync(configPath, JSON.stringify(config));
  execFileSync(`${root}target/release/sugarscape`, [
    'run', '--config', configPath, '--seed', '7', '--ticks', '64',
    '--fingerprint-trace', tracePath,
  ], { cwd: root, encoding: 'utf8' });
  const trace = JSON.parse(readFileSync(tracePath, 'utf8')) as
    { tick: number; fingerprint: string }[];
  expect(trace.map(row => row.tick)).toEqual(Array.from({length: 65}, (_, t) => t));
  const e = await Engine.create({config, seed: 7}, {presets, transport: inline()});
  for (const row of trace) {
    if (row.tick > 0) await e.advance(1);
    expect(await e.fingerprint(), `protection tick ${row.tick}`).toBe(row.fingerprint);
  }
});
```

Import the new functions from the existing local WASM package; reuse `withNativeTraceDirectory`, which already removes its scratch files on success or failure. Core construction tests independently pin the generated configuration to the protocol; the two backends then execute the same checked configuration.

- [ ] Run final changed-code checks once: `cargo fmt --all -- --check`; `cargo test --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo fmt --manifest-path survey/Cargo.toml -- --check`; `cargo test --manifest-path survey/Cargo.toml`; `cargo clippy --manifest-path survey/Cargo.toml --all-targets -- -D warnings`; in `web`, `npm run build` then `npm test`. Use existing installed tools/dependencies. Record counts/logs; never bypass hooks or disable failing tests.
- [ ] Print the committed `--protection --manifest` and validate the condition count, exact configuration, protocol routes and opportunity checks. This command performs no simulations. Conduct the whole-branch review required by the subagent workflow; fix findings with focused regression tests and repeat only affected checks unless behavior broadly changed.
- [ ] Mark implementation complete only after passing tests and review. Remove root stage tracker; retain the completed detailed plan. Commit: `test(minds): verify protection persistence and native wasm replay`. Present the manifest/protocol and verification evidence for scientific execution review. No campaign data or scientific conclusion is part of this implementation task.

## Self-review and execution handoff

Spec coverage: local event memory (1), physical paid actions and cancellation (2), food conservation and schedules (3), all registered contrasts/archives (4), default reductions/persistence/platform agreement (5). Identity-aware cognition, arbitrary same-site deposit mixing, free-world support, UI, learning/evolution and deception remain outside this first version.

Before execution, review the plan and proposed protocol together. The preserved execution method is subagent-driven; do not ask the user to choose it again. After approval, implement sequentially with the five task gates and the root stage tracker. After implementation, scientific execution has a separate review of the concrete executable manifest and opportunity-check evidence.
