# Minds P4 False Caching Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user has already selected this execution method; do not ask them to choose again.

**Goal:** Implement a bounded false-caching laboratory that distinguishes a misleading observation, a receiver detour and an economic benefit, with durable provenance before any scientific execution.

**Architecture:** A dedicated default-off `minds::deception` module supplies physical actions, a single observer-input adapter and a fixed laboratory runner. Existing movement/arrival semantics and the pure original-food ledger remain authoritative; a separate survey route validates and reports saved data. No general communication, learning or UI subsystem is introduced.

**Tech Stack:** Existing Rust workspace, serde/serde_json, existing survey paired statistics, wasm-bindgen, existing Vitest/native trace fixtures; no new dependency.

**Spec:** [approved written design](../specs/2026-10-07-minds-deception-design.md). Original user-reviewed proposal SHA256 `c385f7c553e38c5c1395cd309fc8dc70310152f25a14e6b2a24b3fff82224702`; approval receipt is retained in `.superpowers/sdd/2026-10-07-minds-deception-design/design-approval.json`.

**Planning base:** record actual HEAD when execution starts; this plan was prepared against `a2ebf4fcbd63b353cecaa1191b61a57a33ad249a`. Fetch/reconcile newer main before work and integration, preserving unrelated changes. This document is a plan, not an executable registration or an acceptance receipt.

## Global Constraints

- “It makes no claim of learned deceptive intent, theory of mind, animal cognition, evolutionary stability or numerical reproduction of a corvid experiment.”
- “It moves **zero food**, creates no physical cache or cohort, and records no gross burial.”
- “Ambiguous real and sham bouts must produce the same public observation when their visible gestures match: no is_sham flag, actual deposited quantity, sender policy or hidden stock balance.”
- “Effort must be paid before a signal is emitted.” An unaffordable attempt consumes an action, records failure and emits no completed cue; ordinary metabolism still applies.
- “Proposed budget: **96 cells × 40 paired seeds = 3,840 episodes**, seeds 20001–20040.” Three senders × two views × two opportunities × two layouts × two effort charges × two orientations; the scientific survey route permits no matrix/seed/quantity override; checked core/WASM construction helpers are separate.
- “64 primary descriptive estimates”; retain the 64 Sham-minus-ordinary secondary estimates separately. Use existing paired Student-t summaries, no multiplicity/overall Holds/Fails, no favorable pooling.
- Owner/observer holdings44/96, metabolism1, capacity128, reserve4, visions2/6; one12-unit original cache, two4-unit finite patches, 9×9 opaque border; nominal gesture quantity12, observer memory span64,64ticks.
- “No growback, reproduction, replacement, guards, larders, trait evolution, discovery draws, generic communication or extra controller families.”
- “Persist an attempt receipt before constructing each episode, then its outcome or error”; preserve partial/invalid/unavailable/pending counts and forbid overwrites, automatic retry/resume or seed replacement.
- “Any shared accounting refactor needs explicit equivalence fixtures.” P3 registration/data/archive/binary and historical studies remain untouched; no P3 remeasurement.
- Construction uses7/8 in base orientation; check reflected geometry without outcome pilots. Registered seeds20001–20040 stay held until actual independent prospective review of the committed protocol/manifest/opportunity packet.
- Never disable tests or bypass hooks. After three failed attempts on an issue, stop, document and reassess.
- Integration follows the user's reviewed local-main merge then normal push, with no PR. Verify actual remote head, matching CI/Pages and appropriate served hashes; archive exact evidence before owned cleanup.

## Review Focus

1. A clear **zero new transfer** cannot erase an earlier legitimate cache belief; arrival inspection is a different event. Task1 pins this distinction.
2. Legacy positive-burial callbacks cannot leak actual quantity or double-add a cue for a P4 observer. Tasks1–2 test the complete physical call path.
3. Changing a receiver's private memory or view must not change sender policy choices before any physical collision. Task2 tests this counterfactual.
4. A late I/O failure or interruption must leave the exact started condition/seed and every previously persisted frame, with no valid-zero surrogate. Task4 injects this failure.
5. Ordinary-reference cost-axis aliases and owner occupation must not masquerade as independent samples or successful signal diversion. Tasks3–4 preserve separate identities, occupancy and repeated outcomes.

## File and interface map

| File | Responsibility / owning task |
| --- | --- |
| `crates/sugarscape-core/src/minds/deception/state.rs` | Typed treatments, owner intent, runtime identities; Task1 |
| `minds/deception/observation.rs` | Public evidence only, visibility dispatch, bounded existing seen memory; Task1, physical hook Task2 |
| `minds/deception/lab.rs` | Config validation/roles/geometry; Task1; complete fixed schedules/conditions Task3 |
| `minds/deception/controller.rs` | Paid neutral/sham action and cancellation; Task2 |
| `minds/deception/accounting.rs` | P4-only diagnostic adapters over existing pure Ledger; Task2 |
| `minds/deception/records.rs` | Research-only frames/actions/cues/choices/deaths; Task3 |
| `minds/deception/runner.rs` | Completed ticks, optional diagnostic collection and frame sink; Task3 |
| `survey/src/claims/deception.rs` | Fixed manifest and strict CLI; Task4 |
| `survey/src/claims/deception_archive.rs` | Exclusive writes, attempts, streamed frames and strict saved-data validation; Task4 |
| `survey/src/claims/deception_report.rs` | Complete cell endpoints and registered descriptive estimates; Task4 |
| `docs/superpowers/specs/2026-10-07-minds-deception-protocol.md` | Actual numerical/scheduling contract produced and reviewed before science; Task3 |
| `crates/sugarscape-core/tests/deception.rs`, `tests/checkpoint.rs`, `web/src/determinism.test.ts` | Authoritative state/default-off/native-WASM verification; Tasks1/5 |

Paths abbreviated with `minds/` in this table are under `crates/sugarscape-core/src/`. Every task below names its complete edits.

## Stage 1 / Task 1: Typed lab, public observations and state persistence

**Goal:** Enable checked construction and a pure observation contract without a sham action or experiment.
**Success Criteria:** Zero-transfer semantics, quantity privacy, bounded memory and default-off roundtrip verified.
**Status:** Not Started.

**Files:** Create `minds/deception/{mod,state,observation,lab}.rs`; modify core `config.rs`, `agent.rs`, `world.rs`, `minds/mod.rs`; create core `tests/deception.rs`. All module paths here use the core source prefix above.

**Interfaces produced:**

```rust
// state.rs; all treatment enums serde snake_case + Clone/Debug/PartialEq.
pub enum SenderPolicy { Ordinary, MatchedNeutral, Sham }
pub enum View { Ambiguous, Clear }
pub enum Layout { OnRoute, OffRoute }
pub struct LabConfig {
    pub sender: SenderPolicy, pub view: View, pub display_seen: bool,
    pub layout: Layout, pub effort_cost: f64, pub mirrored: bool,
}
pub enum Stage { Preparation, ToDisplay, Display, Return, Ordinary, Departure, Cancelled }
pub struct SenderState { pub stage: Stage, pub attempted: bool, pub pending_departure: bool }
pub struct Runtime { pub source: u32, pub display: u32, pub prepared: bool, pub diagnostics: bool }
// lab.rs
pub fn rig_config(lab: LabConfig) -> crate::config::Config;
pub(crate) fn validation_errors(c: &crate::config::Config) -> Vec<crate::config::FieldError>;
pub(crate) fn initialize(w: &mut crate::world::World);
// observation.rs; no sender-policy or ground-stock field in either public type.
pub enum Signal { Cue { nominal_amount: f64 }, VisibleTransfer { amount: f64 } }
pub struct Observation { pub actor: u64, pub site: u32, pub tick: u64, pub signal: Signal }
pub fn perceive(actor: u64, site: u32, tick: u64, actual_transfer: f64,
               view: View, nominal: f64) -> Result<Observation, String>;
pub fn update_seen(seen: &mut std::collections::BTreeMap<(u32,u64),
    crate::minds::caching::watching::SeenCache>, obs: &Observation, cap: usize)
    -> Result<(), String>;
```

`LabConfig::default()` is Ordinary/Ambiguous/seen/OffRoute/cost0/base. Constants are nominal12, span64, ticks64. Add `Config.deception_lab: Option<LabConfig>` and `Agent.deception: Option<SenderState>` with serde default/skip-none; initialize None in current constructors. Add `World.deception: Option<Runtime>`. Tasks2–3 extend Runtime with diagnostic ledger/record buffers; those are not policy inputs. Public config/state/observation/record types derive Clone, Debug, PartialEq, serde Serialize/Deserialize; LabConfig denies unknown fields. Enum defaults and explicit SenderState initialization avoid uninitialized stages. Validate mutually exclusive P3/P4/legacy labs, one good and the complete fixed rig; reject negative/nonfinite/unsupported effort and hot changes to this reset-only option. Extend the zero-growback lab allowance explicitly. Hash active P4 config, owner state and authoritative runtime fields with domain marker `u64::from_le_bytes(*b"deceptP4")`; skip the entire new block when off, and exclude diagnostic fields.

- [ ] **Step1: Add failing observation and persistence tests.** Put the following within observation.rs's unit tests with its types imported; separately test NaN/negative values, memory cap eviction by `(tick,site,actor)`, zero-transfer preservation, config/state serialization and active-field fingerprint coverage.

```rust
#[test]
fn ambiguous_real_and_sham_have_identical_public_evidence() {
    let fake = perceive(1, 48, 13, 0.0, View::Ambiguous, 12.0).unwrap();
    let real = perceive(1, 48, 13, 12.0, View::Ambiguous, 12.0).unwrap();
    assert_eq!(fake, real);
}
#[test]
fn clear_no_transfer_keeps_an_earlier_real_cache() {
    use crate::minds::caching::watching::SeenCache;
    let mut seen = std::collections::BTreeMap::from([
        ((48, 1), SeenCache { amount: 4.0, tick: 3 })
    ]);
    let obs = perceive(1, 48, 13, 0.0, View::Clear, 12.0).unwrap();
    update_seen(&mut seen, &obs, 8).unwrap();
    assert_eq!(seen[&(48,1)], SeenCache { amount: 4.0, tick: 3 });
}
```

- [ ] **Step2: Run the focused tests red.** `cargo test -p sugarscape-core --lib minds::deception:: --locked` and `cargo test -p sugarscape-core --test deception --locked`; retain compile/test failure receipts.
- [ ] **Step3: Implement the typed construction and adapter.** Use the existing walls/P3 role setup pattern, not P3 LabConfig or protection state. `perceive` validates finite nonnegative transfer and finite positive nominal; Ambiguous constructs Cue with nominal, Clear constructs VisibleTransfer with actual transfer. `update_seen` adds only positive inferred amounts, leaves prior memory untouched for zero, rejects overflow before mutation, and evicts oldest entries at the declared cap. The core branch is:

```rust
let signal = match view {
    View::Ambiguous => Signal::Cue { nominal_amount: nominal },
    View::Clear => Signal::VisibleTransfer { amount: actual_transfer },
};
Ok(Observation { actor, site, tick, signal })
```

Only the world-side sensor calls this adapter with physical transfer; the receiver consumes Observation alone. Initialize roles1/2 at `(3,3)/(3,6)`, holdings44/96, visions2/6, metabolism1 and fixed watch/cheat traits; scatter capacity128, reserve4, Book/Walk speed1, memory/span64, no surplus burials. MapFlat0 plus resource/capacity4 at `(2,2)/(2,3)`. Mirror every x via8−x. No source is deposited until the scheduled tick0 action. Constants/senders use the declared snake_case serde names; canonical condition labels use ordinary/matched-neutral/sham, ambiguous/clear, seen/unseen, on-route/off-route, cost0/3 and m0/1.
- [ ] **Step4: Run green plus reductions.** Focused tests above, `cargo test -p sugarscape-core --test golden earlier_presets_are_unchanged --locked`, workspace fmt check. Compare old config JSON without the new option and None state fingerprints; do not regenerate goldens to hide a mismatch.
- [ ] **Step5: Self-review and normal commit.** Confirm serializer adds no default keys and input types carry no hidden truth. Stage only the declared paths; `git commit -m "feat(minds): define checked deception observations and state"`.
- [ ] **Step6: Fresh task review.** Controller supplies approved spec/plan, diff, test receipts and interfaces to a new reviewer. Complete all material findings before Task2.

## Stage 2 / Task 2: Paid actions, exclusive observation dispatch and food accounting

**Goal:** Execute a neutral/sham bout without changing physical cache stock and without bypassing or duplicating information channels.
**Success Criteria:** Exact same cost and action budget, cue only after payment/sight, conservation and sender privacy; P3 fixtures unchanged.
**Status:** Not Started.

**Files:** Create core `minds/deception/{controller,accounting}.rs`; modify its mod/state/observation modules and `world.rs`, `minds/caching/{mod,watching,theft}.rs`, `rules/{mod,movement,lifecycle}.rs`, and `minds/protection/ledger.rs` for an additive pure ActionCost variant only. Tests live beside the new modules and in `tests/deception.rs`.

**Consumes:** Task1 types, `watching::watchers_of`, normal `movement::arrive`, `Harvest`, `Ledger::{new,prepare,withdraw,pilfer,outflow,harvest,lose_owner,reconcile,reconcile_physical}`.
**Produces:**

```rust
pub enum BoutKind { Neutral, Sham }
pub enum BoutResult { Completed, Unaffordable, Occupied, Unreachable, OwnerDied }
pub(crate) fn perform_bout(w: &mut World, actor: u64, kind: BoutKind,
                         effort: f64) -> Result<BoutResult,String>;
pub(crate) fn dispatch(w: &mut World, actor: u64, site: u32,
                      actual_transfer: f64, initial_clear: bool) -> Result<(),String>;
pub(crate) fn on_real_burial(w: &mut World, actor: u64, site: u32, q: f64) -> bool;
// accounting.rs, absorbing diagnostic failures; no biology decision reads its output.
pub(crate) fn update(w: &mut World, actor: u64, f: impl FnOnce(&mut Ledger)
                   -> Result<(),String>);
pub(crate) fn on_deposit(w: &mut World, actor: u64, site: u32, q: f64);
pub(crate) fn reconcile_world(w: &mut World);
```

Runtime now adds `ledger: Option<Ledger>` and `ledger_errors: Vec<String>`. Initialize the ledger from actual owner holdings only when diagnostics are enabled. Adding `Outflow::ActionCost` maps labelled debits to `CohortBalance.cost`, preserving all existing variant encodings and calculations. P4 record buffers later distinguish display effort from physical burial cost; do not label a fake bout gross burial.

- [ ] **Step1: Add failing physical-action tests.** A hand-constructed rig at its current legal owner position suffices; direct `perform_bout` is the generic action primitive, while registered-site scheduling arrives in Task3. Assert cost3 removes exactly3, caches/gross burial/cohort count unchanged, receiver gets exactly one nominal12 cue if visible/ambiguous, and no cue if clear, unseen or unaffordable. Test Neutral has identical debit without a caching cue. A second call after an attempted bout must error without a second debit or cue; even an unaffordable attempt is final for that episode. Test genuine burial through the existing `caching::bury` path adds exactly one observation and hides its amount in Ambiguous mode.

```rust
#[test]
fn sham_spends_effort_without_creating_food() {
    let mut w = World::new(rig_config(LabConfig::default()), 7).unwrap();
    let before = w.agent(1).unwrap().caches.clone();
    perform_bout(&mut w, 1, BoutKind::Sham, 3.0).unwrap();
    assert_eq!(w.agent(1).unwrap().holdings[0], 41.0);
    assert_eq!(w.agent(1).unwrap().caches, before);
}
```

Also compare sender actions with receiver seen maps and View changed but the same own holdings/position/physical occupancy. For the full legacy-hook quantity test, use a hand world at tick13, matching visible positions, and real amounts3 versus9: both ambiguous receiver maps must gain exactly nominal12, not the physical amount or a double addition. Fresh positive-burial evidence followed by a real withdrawal must remain a valid historical cue, not be classified as a born-false sham.
- [ ] **Step2: Run red.** Core deception filter and existing protection filter; retain the initial missing-action assertion. The existing protection filter must remain green before edits.
- [ ] **Step3: Implement one physical operation and one input channel.** Guard alive actor/finite effort/available holdings before emitting. Debit effort into ActionCost, return a zero-gather Harvest from its scheduled turn and allow ordinary lifecycle metabolism afterward. Sham calls dispatch with actual transfer0 and does not call bury or prepare a cohort. Neutral emits no caching observation. Add an early P4-specific dispatch branch inside watching::see; return true from on_real_burial only for the enabled bounded P4 path. Outside P4 preserve the entire legacy hook. Initial true deposit is physically clear for both receiver views; later bouts use the registered view. Clear zero does not erase prior knowledge.

```rust
// In the existing positive-burial observation hook, before legacy memory updates:
if crate::minds::deception::observation::on_real_burial(world, owner, site, q) {
    return;
}
// In the existing pure labelled outflow match, preserve existing arms:
Outflow::ActionCost => c.cost += q,
```

Keep sham observation counters in P4 diagnostics, not the legacy physical burials_seen/gross-burial counters; an actual positive burial may update its legacy observation counters once. Add parallel P4-only callbacks at actual initial deposit, subsequent deposit/withdrawal, pilfer, environmental harvest, capped nonnegative consumption and owner removal. Use the existing pure Ledger math; do not select the P3 ledger or modify P3 outcomes. First source preparation labels only the tick0 original12 at the registered source. Failure records a reason and disables diagnostics, not biology. Pin the mixed-labelled/unlabelled ActionCost example below and owner-death terminal accounting:

```rust
#[test]
fn effort_debits_labelled_food_proportionally() {
    let mut l = Ledger::new(1, 18.0);
    l.prepare(30, 12.0).unwrap();
    l.withdraw(30, 12.0).unwrap();
    l.outflow(3.0, Outflow::ActionCost).unwrap();
    assert_eq!(l.cohorts[&30].cost, 2.0);
}
```
- [ ] **Step4: Run green.** Deception tests, protection tests, `cargo test -p sugarscape-core --test golden earlier_presets_are_unchanged --locked`, fmt. Compare diagnostics-on/off biological holdings/positions/fingerprints for a hand action sequence, and verify bounded mutation without hidden receiver access.
- [ ] **Step5: Self-review and commit.** Trace every physical food operation and every cue recipient; no old hook can double-add. `git commit -m "feat(minds): execute paid sham caching without phantom food"`.
- [ ] **Step6: Fresh task review.** Review actual complete-call-path privacy tests and the shared ledger variant diff before scheduling an episode.

## Stage 3 / Task 3: Fixed schedules, complete traces and prospective protocol

**Goal:** Construct every cell deterministically, with actual observable opportunities and no outcome tuning.
**Success Criteria:** All96 config identities and supplied schedules valid; main pair physically matched before receiver release, no free guarding; construction data separate from science.
**Status:** Not Started.

**Files:** Create core `minds/deception/{records,runner}.rs`; extend lab/state/mod/controller/accounting; modify core `world.rs`, `rules/mod.rs`, `rules/movement.rs`, `minds/mod.rs`; create `docs/superpowers/specs/2026-10-07-minds-deception-protocol.md`. Unit tests in runner/lab modules.

**Consumes:** Tasks1–2 config/state, perform_bout/dispatch/accounting, `World::step`, normal arrival/raid and default shuffle.
**Produces:**

```rust
pub fn conditions() -> Vec<LabConfig>;
pub fn condition_id(lab: &LabConfig) -> String;
pub(crate) fn begin_tick(w: &mut World);
pub(crate) fn scripted_action(w: &mut World, actor: u64) -> Option<Harvest>;
pub fn run_episode(lab: LabConfig, seed: u64, diagnostics: bool)
    -> Result<EpisodeRecord, EpisodeFailure>;
pub fn run_episode_with_sink<F: FnMut(&FrameRecord)->Result<(),String>>(
    lab: LabConfig, seed: u64, diagnostics: bool, sink: F)
    -> Result<EpisodeRecord, EpisodeFailure>;
pub struct EpisodeFailure { pub message: String, pub partial: Option<EpisodeRecord> }
```

`records.rs` defines serde/PartialEq DTOs: EpisodeRecord `{schema, lab, seed, requested_ticks, completed_ticks, owner_ticks_alive, owner_alive, frames, fixture_errors, ledger_errors, cohorts: Option<Ledger>, thief_transferred: Option<f64>, diagnostics_enabled: bool, lineage_unavailable_reason: Option<String>}`; FrameRecord `{tick, fingerprint, roles, actions, observations, choices, deaths, restrictions}`. RoleRecord holds id/position/holdings/physical caches/private SenderState/seen entries. ActionRecord holds actor/phase/actual action/target/physical harvest,dug,buried,effort and metabolic demand/consumption/cancellation. ObservedRecord holds receiver/public Observation plus **research-only** actual transfer/stock for creation-time versus stale error classification. ChoiceRecord holds actor/target/remembered value/diagnostic actual value/arrival,raid,wasted outcome. DeathRecord holds actor/position/cause. Neither diagnostics nor frame DTOs are receiver/controller inputs; sink I/O does not alter RNG/biology. Exact field types: schema/fingerprint/error/phase/action/cause are String; ticks/seed/actor/ids are u64 (AgentId); sites are u32; pos/target use Pos (optional target where absent); quantities are f64; record collections are Vec of their named DTO; roles carry `Vec<SeenRecord { site: u32, owner: u64, amount: f64, tick: u64 }>`; restrictions are BTreeMap<u64,u64>; cohorts are Option<Ledger>. Declared ActionRecord quantities are `harvest, dug, buried, effort, metabolic_demand, metabolic_consumed`; cancellation is Option<BoutResult>. ObservedRecord fields are `receiver: u64, public: Observation, actual_transfer: f64, actual_stock: f64`. ChoiceRecord fields are `actor: u64, target: Pos, remembered_value: f64, actual_value: f64, arrived: bool, raid_amount: f64, wasted: bool`. DeathRecord fields are `actor: u64, pos: Pos, cause: String`. Task3 core exports `pub const SCHEMA: &str = "minds-deception-measured-v1"`; Task4 reexports that identity rather than inventing a second version. Diagnostic-disabled or failed lineage must set a nonblank unavailability reason, never a zero surrogate.

- [ ] **Step1: Write failing canonical/configuration and schedule tests.** Enumerate96 configs in module tests, uniqueness via condition_id,32 strata, role/stock budgets and x→8−x geometry. Construction seeds7/8 only. Test matching neutral/sham role positions, holdings, costs/harvest and physical stock through tick31, independently of their differing memories. Test an on-route decoy need not add movement; never assert a winning outcome. Run Ordinary at seed7 with cost0/3 and compare physical frames/endpoints (excluding config identity and fingerprints): declare these effective aliases rather than extra independent replication. Test source/display occupancy at release and the common post-recovery departure without requiring that a sham wins.

```rust
#[test]
fn fixed_conditions_have_complete_unique_identities() {
    let configs = conditions();
    let ids: std::collections::BTreeSet<_> = configs.iter().map(condition_id).collect();
    assert_eq!(ids.len(), 96);
}
```

- [ ] **Step2: Run red.** `cargo test -p sugarscape-core --lib minds::deception:: --locked`; missing schedules/records fail before new behavior.
- [ ] **Step3: Implement the following exact prospective construction candidate.** This is a protocol candidate, not measurement authorization. Positions are base; mirror every coordinate and apply ordinary index tie-breaking afterward.

| Boundary | Owner | Observer |
| --- | --- | --- |
| Tick0 | bury12 at A=(3,3), no initial cost | hold at (3,6), actually see a clear real transfer |
| Ticks1–7 | hold | hold |
| Ticks8–12, neutral/sham | OnRoute: walk(3,4),(3,5), then hold; OffRoute: walk(4,3),(5,3),(5,4),(5,5),(5,6) | Seen: hold(3,6). Unseen: walk(4,6),(5,6),(6,6),(7,6),(7,7) |
| Tick13 | one neutral/sham action at B=(3,5) or(5,6), cost0/3, nominal12 for sham only | hold at final seen/unseen position |
| Ticks14–19, neutral/sham | OnRoute: walk(2,5),(2,4),(2,3), then hold. OffRoute: walk(5,5),(4,5),(3,5),(2,5),(2,4),(2,3) | Seen: hold. Unseen: walk(7,6),(6,6),(5,6),(4,6),(3,6), then hold |
| From tick20, neutral/sham | ordinary Book behavior | hold through31 |
| Ordinary reference from tick8 | ordinary Book; no display walk/bout/effort charge | same opportunity-specific scripted path/holds |
| From tick32 | ordinary, except common post-recovery departure rule below | ordinary watching/raid decisions, no forced target |

Preparation holds and observer scripted walks gather zero and pay ordinary metabolism. Use existing `movement::walk_without_gather(world, id, target)` for these walks, returning a zero Harvest and recording its actual WalkOutcome/position; do not call normal arrival then discard a real raid/harvest after it happened. Display/return owner walks use normal legal arrival/gather semantics; neutral/sham display and designated holds gather zero. Receiver initial-source memory span64 and no discovery draw. Check body occupancy at every waypoint (the off-route owner returns via y5 to avoid the observer at(3,6)). Never teleport or substitute remote digging.

After a successful owner recovery at A, queue one legal departure toward(3,2) for its next turn, identically for every treatment, preserving costs/failure. This prevents standing on the recovered site from manufacturing a guarding mechanism; record all occupancy and whether real food had already been recovered when an arrival was blocked. Do not mandate an outcome or a mirrored destination beyond the declared transformed departure target.

If any candidate construction is invalid, retain the failure receipt, record a prospective protocol revision and correct its geometry/scheduling before the gate. Do not examine scientific-seed outcomes or modify budgets/axes to manufacture gains. A changed accepted design parameter requires a separately visible design amendment, not a quiet protocol edit. Valid zero/negative effects stay valid.

At tick start count each living-owner tick once; record initial frame0 and every completed frame. Continue to64 or until both die. Call sink on initial and subsequent frames; preserve known partial frames on a returned failure. Clear per-tick diagnostic buffers, not memories/physical stocks. Add P4 begin_tick/scripted_action beside existing seams with no default-off mutation. Normal surplus burial stays suppressed for P4. Capture actual targeting at movement choice and actual inspection at arrival, not targets inferred from the resulting position. Keep public observation and research truth separate.
- [ ] **Step4: Run all construction checks green.** Complete48 base configs×7/8, plus hand-world truthful/clear/empty/blocked/expired/owner-death controls and pure reflection geometry. `run_episode(lab.clone(), 7, true)` versus `run_episode(lab, 7, false)` fingerprints/positions/food agree. Retain receipts and opportunity JSON without paired payoff estimates. No full reflected outcome pilots or registered seeds.
- [ ] **Step5: Self-review/write protocol and commit.** Protocol fixes the above config,coordinates,bout/release/return/departure rules, attempt limits, seed/tick budget, ledger tolerance `1e-9*max(1,initial)`, exact64+64 comparisons and descriptive timing boundaries. Include explicit supplied restrictions, ordinary-reference aliases and no numerical animal target. `git commit -m "feat(minds): add fixed deception laboratory and construction protocol"`.
- [ ] **Step6: Fresh task review.** Reviewer verifies actual preparation budgets, sight/cue modes, matched physical traces, legal access and anti-blocking behavior, without requiring economic success. It is construction approval, not scientific acceptance.

## Stage 4 / Task 4: Durable attempt archive, strict CLI and complete reporting

**Goal:** Save every attempted identity and known frame before analysis, and deterministically analyze only structurally valid complete saved data.
**Success Criteria:**96×40 budget enforced; interrupted/error/partial attempts never vanish;64 primary and64 secondary estimates fully retained or explicitly unavailable.
**Status:** Not Started.

**Files:** Create `survey/src/claims/{deception,deception_archive,deception_report}.rs`; modify `survey/src/{main,claims/mod}.rs`. Existing `survey/src/stats.rs` is consumed unchanged. Rust unit tests within the three new modules.

**Consumes:** Task3 conditions/ids/run_episode_with_sink/EpisodeRecord/FrameRecord/EpisodeFailure and existing `stats::paired_summary`.
**Produces:**

```rust
pub const SCHEMA: &str = sugarscape_core::minds::deception::SCHEMA;
pub struct Condition { pub id: String, pub lab: LabConfig }
pub struct Manifest { pub schema: String, pub conditions: Vec<Condition>, pub seeds: Vec<u64> }
pub fn manifest() -> Manifest;
pub fn cli(args: &[String]) -> Result<(),String>;
pub fn run(protocol_revision: &str, out: &std::path::Path) -> Result<(),String>;
pub fn load(index: &std::path::Path) -> Result<Archive,String>;
pub fn validate_archive(archive: &Archive) -> Result<(),String>;
pub fn analyze(archive: &Archive) -> Result<Analysis,String>;
pub fn render_results(analysis: &Analysis) -> String;
```

Archive types: Index carries schema, exact source/protocol/binary/manifest identities, full Manifest, completion boolean and relative attempt references. AttemptStart records condition/seed/revisions/binary/manifest/start time **before world construction**. AttemptOutcome is Complete(EpisodeRecord) or Failed(EpisodeFailure); a start with no final outcome is Pending/Interrupted, never Complete. Archive owns Index and typed attempts. Analysis owns full endpoint rows/cell diagnostics/duplicate diagnostics/estimates/census. Estimate holds id,metric,primary,optional PairedSummary,unavailable seeds+reasons. All reference paths must remain confined to the archive. `Manifest` is a typed field of Index; attempt/frame references are relative Strings; revisions and hash identities are Strings, seed is u64, elapsed time is finite nonnegative f64. Attempt lists and endpoint/estimate/cell lists are Vec, with uniqueness checked by exact `(condition,seed)` keys. Runtime tests use private dependency injection; no public seed override is introduced.

- [ ] **Step1: Add failing manifest/parser/archive/report tests.** Verify96 distinct canonical IDs,40 exact seeds, denied seed/axis overrides, no default/help/manifest execution, duplicate flags, missing values and unknown options. Any real-world injection uses construction seed7; alternatively inject static frames/failure DTOs without a World/RNG invocation. Inject a sink failure after an initial saved frame and assert start identity/frames persist with failure, no Complete output, no retry. Reject absolute/traversal/symlink paths, duplicate/mismatched identities, altered configs/budgets/horizons, wrong source/protocol, nonfinite ledgers and conflicting existing destinations. Build **synthetic static** complete wire records for full report tests; labels20001–20040 are not world/RNG invocations.

```rust
#[test]
fn manifest_is_fixed_before_execution() {
    let m = manifest();
    assert_eq!(m.conditions.len(), 96);
    assert_eq!(m.seeds, (20001..=20040).collect::<Vec<_>>());
}
#[test]
fn known_zero_difference_is_not_unavailable() {
    let values = std::collections::BTreeMap::from([(20001,4.0),(20002,4.0)]);
    let s = crate::stats::paired_summary(&values, &values).unwrap();
    assert_eq!(s.ci95, Some((0.0,0.0)));
}
```

- [ ] **Step2: Run red.** `cargo test --locked --manifest-path survey/Cargo.toml --bin survey deception_`; retain red receipts. No CLI --run invocation.
- [ ] **Step3: Implement exclusive writes and the saved-data pipeline.** Support exactly `--deception [--manifest|--help]`, `--deception --run --protocol-revision FULL40HEX --out NEW_DIR`, and `--deception --analyze INDEX --out NEW_DIR`. Strip only the route flag and deny mixed routes/extra overrides. Print/schema routes never build a world. Run preflight validates a full existing commit, **exact raw committed/current protocol bytes** (no trim normalization), clean tracked tree and fresh output. Actual controller acceptance is a separate prospective receipt; provenance flags do not create it.

For each sorted condition then seed, exclusively create/sync its AttemptStart, open its exclusive `frames.jsonl`, and stream/sync each completed frame through the sink before final outcome. Record ordinal/seed paths, exact SHA/byte/mode inventory and construction+stepping+diagnostics+durable-frame-I/O timing boundary; exclude final envelope I/O/analysis rather than pretending frame I/O is absent. A known returned partial failure retains partial DTO and durable frames. Process death leaves a start/persisted prefix with pending/interrupted status. A biological/config failure writes an error and can continue to retain the remaining attempted census; an I/O failure stops with its external command/exit receipt and no automatic resumption. Write final index/census with accurate attempted/complete/failed/partial/pending counts.

Strict analysis requires the complete canonical logical matrix and valid identities/initial balances/phase routes/action charges/actual observations/horizons/physical and cohort reconciliations. Malformed, incomplete or biologically invalid archives are analysis errors with an execution census; do not pair surviving subsets. Valid complete records with unavailable lineage retain the full40-seed denominator and list affected seeds, with the food estimate unavailable rather than zero. Use config matching to find baselines, not fragile substring editing. Sham−Matched is primary, Sham−Ordinary secondary; two metrics×32strata each. Do not add significance verdicts or unregistered interactions.

```rust
let summary = if unavailable_seeds.is_empty() {
    Some(crate::stats::paired_summary(&sham_values, &baseline_values)?)
} else {
    None
};
```

Export every cell's evidence/belief/target/inspection/occupancy/cost/food/lifetime/count/timing diagnostics. Group full biological frames with seed/identity/wall-time/fingerprints excluded, and separately report repeated endpoint vectors and effective aliases; no claimed sample-independence from uniqueness. `analysis.json`/`results.md` must reproduce byte-for-byte into a fresh destination from the same saved data.
- [ ] **Step4: Run green and adversarial fixtures.** Focused survey tests, strict saved-reanalysis equality, path attacks, injected interrupted prefix and unavailable-lineage cases; `cargo fmt --manifest-path survey/Cargo.toml -- --check`. Generated test data belongs only in owned test destinations, never historical study output.
- [ ] **Step5: Self-review and commit.** Inspect error persistence from start→world→frame→outcome→index→analysis, no swallowed failures. `git commit -m "feat(survey): archive and report fixed deception attempts"`.
- [ ] **Step6: Fresh task review.** New reviewer checks producer/consumer contract, partial persistence, family counts, paired seeds and output destinations. No science dispatch yet.

## Stage 5 / Task 5: Native/WASM boundary, broad verification and scientific readiness handoff

**Goal:** Finish implementation and an actual prospective packet, not run the campaign on implementation approval alone.
**Success Criteria:** Default-off/P3/native-WASM/checkpoint invariance, final source/binary/manifest/opportunity identities and fresh whole-branch review; explicit measurement hold.
**Status:** Not Started.

**Files:** Modify `crates/sugarscape-wasm/src/lib.rs`, core `tests/{deception,checkpoint}.rs`, `web/src/determinism.test.ts`; update only scoped P4 status/navigation in `README.md`, `docs/{papers,roadmap,reproducibility}.md`, `docs/studies/2026-09-27-minds.md` and the collective-agency program. Do not overwrite P3's registered documents/data/reviews. Readiness/implementation ledgers and test/build outputs stay in this plan's owned workspace.

**Consumes:** Tasks1–4 rig_config/run_episode/manifest/CLI/archive/report interfaces.
**Produces:**

```rust
#[wasm_bindgen]
pub fn deception_config_json(lab_json: &str) -> Result<String, JsValue>;
#[wasm_bindgen]
pub fn deception_episode_json(lab_json: &str, seed: &str) -> Result<String, JsValue>;
```

Decode with serde then full rig validation; use the existing strict `decimal_seed` and structured field errors. Episode JSON contains the same research DTO as native, never a new policy input.

- [ ] **Step1: Add failing real boundary/checkpoint tests.** Mirror the existing protection Vitest fixture mechanism using generated deception exports and the existing native trace-directory fixture. Invoke the native CLI with its actual temporary config/trace paths as shown below. Compare every tick0–64 against WASM EpisodeRecord and `Engine.create`. Cover Sham/Ambiguous/seen/off-route/paid, Neutral with the same factors, Sham/Clear, and Sham/unseen in base orientation. Reject `'', '-1', '+7', '7.0', ' 7', '18446744073709551616'`, malformed/unknown lab fields and unsupported costs. Check restoration during preparation, transit, paid bout, return and pending departure, retaining observer memory and exact RNG continuation.

```typescript
const labJson = JSON.stringify({ sender: 'sham', view: 'ambiguous',
  display_seen: true, layout: 'off_route', effort_cost: 3, mirrored: false });
const config = JSON.parse(deception_config_json(labJson));
const record = JSON.parse(deception_episode_json(labJson, '7'));
expect(record.frames.map((f: {tick: number}) => f.tick))
  .toEqual(Array.from({length: 65}, (_, tick) => tick));
await withNativeTraceDirectory(async scratch => {
  const configPath = `${scratch}/config.json`, tracePath = `${scratch}/trace.json`;
  writeFileSync(configPath, JSON.stringify(config));
  execFileSync(`${root}target/release/sugarscape`, ['run', '--config', configPath,
    '--seed', '7', '--ticks', '64', '--fingerprint-trace', tracePath],
    { cwd: root, encoding: 'utf8' });
});
```

Use `root = fileURLToPath(new URL('../../', import.meta.url))` and existing fs/native-trace imports in that test file. For each additional fixture override only the named Task1 fields from the JSON helper; do not use a mock WASM implementation. Add source-side counterfactual state mutations to ensure fingerprints include each behavior-affecting P4 field.
- [ ] **Step2: Run red boundary checks.** Core checkpoint/deception filters and Vitest protection+deception boundary filters; retain genuine missing-export/fingerprint evidence. Native/wasm fixtures only use construction7, never registered science seeds.
- [ ] **Step3: Implement the adapters and status docs.** Use the protection boundary's read/validate/seed/error pattern, calling P4 APIs only. Explain supplied cue assumptions, separate false-estimate/diversion/economics, complete96-cell declaration and pending scientific execution. Record the unresolved1998citation trail as a lead, not a confirmed reproduction failure; preserve existing stable register IDs and earlier historical results.
- [ ] **Step4: Run final implementation gates on the actual candidate.** `cargo fmt --all -- --check`; `cargo fmt --manifest-path survey/Cargo.toml -- --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo test --workspace --locked`; focused survey deception suite; P3 protection/native-WASM fixtures; `wasm-pack test --node crates/sugarscape-wasm`; web `npm test` and `npm run build`; `cargo build -p sugarscape-core --target wasm32-unknown-unknown --locked`; release book tests `cargo test -p sugarscape-core --release --test book -- --ignored`. Capture actual commands/exits/HEAD/stdout/stderr, distinguish existing ignored tests, and use isolated targets. Do not rebuild or reuse the retained P3/DP scientific executables or environments. Verify all new links/scripts/actual help+manifest and no registered-world invocation.
- [ ] **Step5: Build the prospective packet, commit and whole-branch review.** Commit only working scoped files with normal hooks. Freeze a clean scientific candidate separately from later reporting integration. Retain exact source inventory/committed protocol bytes/actual full manifest/toolchain/inherited flags/build command/binary SHA and full base7/8 construction opportunity receipts. Retain actual historical limitations, never generate old acceptance. A fresh whole-branch reviewer examines the full diff and test receipts; task reviewers do not approve their own work.
- [ ] **Step6: Present the actual scientific gate.** Fresh independent prospective review checks the committed96×40/3,840 declaration,64+64 estimates, physical-cost controls, cue/no-truth-leak path, geometry/opportunities, output freshness and invariance. HOLD registered seeds until an actual accepted review. If accepted and the existing user authorization covers continuation, a separately recorded one-writer execution may follow; no flags or this plan can manufacture acceptance. Capture all raw attempts/frames/outcomes, verify the complete census and saved-data reanalysis, inspect real PNG/SVG figures/exact chart inputs, obtain fresh empirical review, locally integrate/push/reconcile main, verify actual published SHA/CI/Pages/served scientific bytes and archive everything before cleanup. Those post-gate activities receive their own execution ledger/status; do not mark them complete because implementation passed.

## Execution setup and durable handoff

1. Check this plan's owned ledger first. Tasks with a complete line/commit/review are done; never redispatch P3 or restart an approved P4 task after compaction.
2. Prefer the configured global worktree `~/.config/superpowers/worktrees/SugarScape/minds-deception`. Preserve the user checkout/unrelated work. Current session can write planning docs but Git metadata and that global location are read-only: keep these docs uncommitted. If execution remains restricted, an isolated writable temporary checkout is the only implementation fallback; never implement on main. Publication/archival that lacks writable/network capabilities must be reported precisely rather than faked or forced.
3. At execution, create a root `IMPLEMENTATION_PLAN.md` with the five stages above, exact deliverables/checks/status, and update it task-by-task. Preserve a completed snapshot externally before removing it. Use this plan's ignored SDD workspace for briefs/reviews/logs/rulings, not another plan's directory.
4. Each task: test red → minimal code → test green → self-review → normal commit → fresh independent spec/quality review. Controller routes fixes; review broken paths systematically; stop/reassess after three failed attempts. Run required new covering checks after fixes, avoid repeated broad checks without changed source or unresolved evidence.
5. Written-plan user review is next. Subagent-driven execution is already selected; implementation-plan approval permits code/declared construction verification, while actual scientific execution still requires the separate prospective gate.
