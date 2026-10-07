# Active Surface Experiments Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. The user has already selected subagent-driven execution and independent reviews; do not ask them to choose again.

**Goal:** Let one Agent decide whether another paid probe round or an attempted exchange improves its expected remaining net task utility, while retaining a fixed responding Agent and exact local-information boundaries.

**Architecture:** Add a sibling `active_surface` laboratory that composes frozen `shared_surface::World`, role-local physical controller rules, and exact probability/value types. A bounded decision tree compiles the experimenting policy; a separate chronological replay calculates both Agents' local beliefs under that compiled policy. The host evaluates actual episodes and emits a compact reconstructible diagnostic whose integrity is recomputed from its current payload.

**Tech Stack:** Existing Rust workspace, serde/serde_json, clap, checked `Probability` and `ScoreFraction`, existing Rust/WASM/web/Studio verification tools. No new dependency or browser UI.

**Spec:** [approved design](../specs/2026-10-07-active-surface-experiments-design.md). User-reviewed proposal SHA256 `4b91565a48c1ca5b6c3e693ccccce3c0bd42f1155b3144e10c42084b96c117f9`; approval response `LGTM!` is retained in `.superpowers/sdd/2026-10-07-active-surface-experiments/design-approval.json`.

**Status:** Written plan approved by the user October 7, 2026; subagent-driven execution authorized. Actual execution HEAD is recorded in retained plan-approval.json; collection still requires the independent precollection gate. Planning started from crowd `de690b664d6c858b0b04e6a7ed2f13aa430967cf`; newer main and concurrent work must be reconciled before integration without discarding either.

## Global Constraints

- “The active controller maximizes the experimenting Agent's expected net task utility.” Responder and group utility are separate reported outcomes; no autonomous willingness to assist or negotiated cooperation.
- “Retain the four physical mechanisms: shared persistent, shared resetting, private persistent, and inert.” Keep the seven symbols, opaque IDs, original chronological role ownership, generic write acceptance and round/phase resets.
- “Each completed probe round costs each Agent two credits.” “Ending probing creates no further calibration slots and incurs no skipped-slot wait charges.” At most three rounds, in original order; stopping is a free boundary decision, not a free observation or physical wait.
- “The public phase length can convey information about the experimenter's model belief”; record and infer from it. Live routine selection is private to the selecting Agent; do not announce it to the responder.
- “Retain 48 starting credits per Agent, read/write/wait cost one, trusted inspection cost four, and reward 12 for a correct prediction and zero for an incorrect one.” Four independent fair-bit pairs, free prediction, target ties predict zero, cumulative credits across all trials.
- “On equal expected utility, stop probing before continuing; in live routine ties choose inspection.” Use numeric exact comparisons, not derived ordering of numerator/denominator pairs.
- “A chosen experiment is an intervention: do not multiply a candidate by a likelihood for why the experimenter selected its own action.” Own known bits condition once; future bits stay latent until their original issue event.
- “Never select a candidate using the responder's actual history, actual mechanism, or actual target bit.” Candidate responder prior is point mass at that candidate model. For DataFlip, actual responder receives nominal SP; experimenting Agent remains uniform.
- “Do not reuse the old frozen Ensemble's likelihoods for a partner whose decision policy has changed.” Responder action simulation needs no target posterior; its target posterior/prediction comes from a separate replay of the solved active policy.
- “40 settings”, “10,240 actual episodes”; six secondary DataFlip settings, 1,536 episodes. No after-result prior, cost, reward, horizon, or probe-order changes/sweeps. Expected zero benefit and ties are valid outcomes.
- Preserve old deduction and shared-surface source/settings/outputs; no runtime edits to those namespaces. Only additive `lib.rs`/CLI dispatch seams. Never stage `.claude/`, `papers/`, `survey/out/`, or ignored evidence. American spelling and Agent terminology.
- Preserve exclusive first reports, settings/source/binary snapshots and reviews. Disclose any revision after measurement; retain original and revised series. Never relabel a revised series first.
- Use the existing crowd worktree. Fresh task implementers and independent reviewers; independent full precollection and final reviews. Commit working increments normally, no hook bypass; merge completed work into main, push and check CI for the exact commit, plus Pages when triggered. Preserve worktree/evidence.
- Rustfmt only Rust files edited, with `skip_children=true` when appropriate; never run cargo fmt inside survey. Workspace formatting **checks** are allowed. Stop/reassess after three failed attempts on an issue.

## Review Focus

1. Zero-round stopping must enter Live(0) without a virtual calibration reset, skipped waits, or early task disclosure; Task 1 tests all three.
2. Experimenter B chooses at its first owned live slot after responder A's unobserved first action; no role swap or peer-action receipt; Tasks 1–2 test this chronology.
3. Equal model marginals can hide different physical/responder states and remaining decision values; Task 2 pins full-prefix/domain caching and signed numeric ties.
4. Public stopping is evidence, but private routine choice is not a responder observation; Task 3 tests complete replay and rejects side-channel additions.
5. Forged alternative values, chosen routines, missing episodes, stale passed flags and late flush failure must not succeed; Task 4 owns tamper and emitter tests, Task 5 runs the nonignored success test after first collection.

## File and interface map

| File | Responsibility / task |
| --- | --- |
| `crates/sugarscape-core/src/active_surface/types.rs` | Protocol, typed boundaries/choices/local entries, errors, records; Task 1 |
| `active_surface/kernel.rs` | Checked chronological transitions and fixed physical routine adapter; Task 1 |
| `active_surface/belief.rs` | Original-mass candidate domains and local outcome partitions; Task 2 |
| `active_surface/planner.rs` | Exact value recursion, deterministic choices and compiled policy; Task 2 |
| `active_surface/replay.rs` | Both-role inference under solved policy including public stop; Task 3 |
| `active_surface/evaluation.rs` | Actual host rollout, metrics and exhaustive panels; Task 3 |
| `active_surface/diagnostics.rs`, `diagnostics/protocol.rs`, `diagnostics/references.rs` | Frozen 46-setting census, compact histories, payload validation/oracle projections; Task 4 |
| `active_surface/tests/{kernel,planner,replay,evaluation,diagnostics}.rs` | Behavioral boundaries and exact fixtures; owning task |
| `active_surface/tests/fixtures/{planning-reference,evaluation-reference}.json` | Small independent numeric/trace projections and provenance; Tasks 2–4 |
| `crates/sugarscape-cli/src/active_surface.rs`, `tests/active_surface.rs` | `active-surface diagnose`, flush/exit handling and success test; Task 4 |
| `crates/sugarscape-core/src/lib.rs`, CLI `src/main.rs` | Additive module/export/dispatch only; Tasks 1/4 |
| `docs/active-surface.md`, `docs/shared-surface.md` | Measured guide and appended guide link; Task 5 |

All abbreviated core paths above are below `crates/sugarscape-core/src/`. Do not introduce a general controller framework, optimizer dependency, new old-protocol switches, or WASM/UI adapters for this CLI experiment.

## Shared contracts

Task 1 defines these public lab types; existing physical types are re-exported from `shared_surface` without modification:

```rust
pub use crate::shared_surface::{Action, Belief, Environment, EpisodeBits, Event,
    Ids, LocalEntry, Mechanism, OwnPrior, Phase, Position, Probability, Role,
    ScoreFraction, Symbol};

pub enum PolicyKind { Adaptive, FixedThree, NoProbe, InspectOnly, Known }
pub enum Choice { ContinueProbe, StopProbing, Inspect, AttemptCommunication }
pub enum Boundary {
    ProbeChoice { completed: u8 },
    TrialChoice { trial: u8 },
}
pub enum Checkpoint {
    Boundary(Boundary), BeforeSlot(Position), AfterSlot(Position),
    Prediction { trial: u8 }, Finished,
}
pub enum Entry {
    Physical(LocalEntry),
    OwnChoice { boundary: Boundary, choice: Choice },
    PublicProbeStop { completed: u8 },
}
pub struct Protocol { pub experimenter: Role, pub policy: PolicyKind, pub ids: Ids }
pub struct Prefix {
    pub role: Role, pub ids: Ids, pub own_prior: OwnPrior,
    pub checkpoint: Checkpoint, pub entries: Vec<Entry>,
}
pub struct View { pub prefix: Prefix, pub credits: u8,
    pub private_bit: Option<bool>, pub belief: Belief }
pub enum Error {
    InvalidProtocol(String), InvalidHistory(String),
    UnsupportedHistory { prefix: Prefix },
    InvalidReport(String), ArithmeticOverflow,
    Physical(crate::shared_surface::Error),
}
```

Add checked constructors/serde validation for completed rounds 0..=3, trial 0..=3, consistent role/clock, unknown fields, legal boundary choices, valid rational forms and opaque IDs. Value/key types derive Clone/Debug/PartialEq/Eq and the enums derive Copy; ordered cache keys use stable structural ordering, but exact values use numeric checked comparison. Serde fields deny unknown keys and bounded numeric wrappers validate on decode. `TrialChoice` resolves to A at Live/trial/round1/slot1 or B at Live/trial/round1/slot2. Do not serialize a privileged environment or candidate roster into `View`. `OwnChoice` is appended only to the chooser; `PublicProbeStop` is appended to both histories. A current private bit must agree with the latest issued physical entry, and credits with the latest own paid event or initial 48. Task 1 implements `View::from_prefix(prefix: Prefix, belief: Belief) -> Result<View, Error>` by extracting credits/current own bit from validated own entries; it receives no host state.

Kernel state is privileged and never a controller argument. Its snapshots expose histories for evaluator/replay use only:

```rust
pub struct EpisodeState { /* private checked physical/runtime state */ }
pub struct Step { pub histories: [Prefix; 2], pub events: Vec<(Role, Event)>,
    pub next: Option<Boundary>, pub spent_delta: [u8; 2] }
impl Protocol { pub fn new(experimenter: Role, policy: PolicyKind) -> Self; }
impl EpisodeState {
    pub fn new(protocol: &Protocol, environment: Environment,
        bits: EpisodeBits, priors: [OwnPrior; 2]) -> Result<Self, Error>;
    pub fn prefix(&self, role: Role) -> &Prefix;
    pub fn credits(&self, role: Role) -> u8;
}
pub fn apply_choice(state: &mut EpisodeState, choice: Choice) -> Result<Step, Error>;
pub fn select_choice(state: &mut EpisodeState, choice: Choice) -> Result<(), Error>;
pub fn advance_one(state: &mut EpisodeState) -> Result<Step, Error>;
```

The explanatory private-state block above is not permission to leave an empty runtime struct: Task 1 implements World, both histories/credits/private bits, current clock, completed probe count and active routine. `Protocol::new` supplies `Ids { surface: "status-field".into(), agents: ["Agent-A".into(), "Agent-B".into()] }`, matching frozen shared-surface defaults without calling its private settings helper. `select_choice` records a checked free boundary choice and queues the routine, without executing paid slots. `advance_one` advances one paid slot or one public phase/boundary transition and returns its exact histories/checkpoint; a paid-step return is AfterSlot, before any subsequent round reset/bit/slot. `apply_choice` is a convenience for **hypothetical/construction** execution that selects and repeatedly advances until the next choice or terminal point. The actual host uses select/advance_one and checks support after each paid observation before advancing again. Do not implement host execution as an atomic whole-routine call followed by retrospective failure detection.

## Task 1: Checked boundaries and supplied physical routines

**Files:** Create `active_surface/{mod,types,kernel}.rs`, `tests/kernel.rs`; add the sibling module export to core `lib.rs` only.

**Consumes:** Frozen public World/apply/reset/finish_round, `shared_surface::decide`, types and exact fractions. **Produces:** The Shared contracts above and checked candidate/actual transition kernel used by Tasks 2–4.

- [ ] **Step 1: Preserve baseline before runtime edits.** In the new plan's ignored workspace, retain HEAD/status, exact old namespace/Cargo.lock/manifest hashes, lib/main source copies, source/settings of the prior first report and its original path/hash, copied current release CLI and seven old-command before stdout/stderr. Preserve the prior shared-surface first/repeat files in place; new evidence can reference their immutable hashes without copying another 10 GB. Record every read of previous campaign evidence and never write there. Record the old `docs/shared-surface.md` prefix before appending a guide link later.
- [ ] **Step 2: Write failing transition tests.** Use genuine missing-module/API errors. Define a local helper inside `tests/kernel.rs`:

```rust
fn state(role: Role, policy: PolicyKind, model: Mechanism) -> EpisodeState {
    let p = Protocol::new(role, policy);
    let mut priors = [OwnPrior::PointMass(model); 2];
    priors[role.index()] = if policy == PolicyKind::Known {
        OwnPrior::PointMass(model)
    } else { OwnPrior::Uniform };
    EpisodeState::new(&p, Environment::InFamily(model),
        EpisodeBits::from_index(0).unwrap(), priors).unwrap()
}
#[test]
fn one_probe_round_charges_both_agents_two_credits() {
    let mut s = state(Role::A, PolicyKind::Adaptive, Mechanism::SharedPersistent);
    let step = apply_choice(&mut s, Choice::ContinueProbe).unwrap();
    assert_eq!(step.spent_delta, [2, 2]);
    assert_eq!(step.next, Some(Boundary::ProbeChoice { completed: 1 }));
}
```

Also test Stop at completed0 produces no Calibration Reset/Action entries, exactly one Live0 reset and own bit, no additional calibration cost, and the correct first live boundary. Test both orientations: B's boundary occurs after responder A's unobserved first owned slot and A's event is absent from B's local entries. Test all 0/1/2/3 stop paths, max3 rejects another probe, a live routine commits one choice for that whole trial, Inspect costs9 and Attempt costs6 over an entire experimenter trial, 4trial limits, own private bits, insufficient credits and wrong-phase choices leave state unchanged. At most 42 credits per Agent on complete paths. Test generic Accepted remains generic and Stop cannot inspect a target.

Add a select/advance_one test proving one paid event returns before round reset or a later task bit, and that the host can halt there without executing remaining routine slots. Use the whole-routine convenience only for successful candidate/construction budget checks.
- [ ] **Step 3: Run RED.** `cargo test -p sugarscape-core active_surface::tests::kernel`; retain command/stdout/stderr/exit. Compile failure must be caused by the proposed interface, not a broken existing test.
- [ ] **Step 4: Implement checked adapters.** For probe slots call frozen `decide` with its local physical prefix; calibration logic is identical for Known/Unknown. For fixed responder slots call it with a point model belief and current local read guards/private bit; its target posterior is unnecessary for action selection. Use candidate model for hypothetical responder priors and supplied nominal prior for actual controls. Do not call frozen Ensemble. Experimenter Inspect uses the old NoCommunication slot actions; Attempt uses A Write-own/R1S1 and Read/R3S1, or B Read/R2S2 and Write-own/R2S3, with Wait in other owned slots. Actual World remains responsible for symbols, effects, budget validation and hidden truth inspection.

```rust
// Pattern inside the new adapter; no edits to the frozen controller.
let event = world.apply(role, position, &action, target, credits)?;
credits = event.credits_after;
history.push(Entry::Physical(LocalEntry::Action(event.clone())));
```

After each paid slot, retain AfterSlot before a round reset/future bit. Ending calibration records both public stop entries, then enters the original live phase reset. For B's first live choice advance the known A action before exposing the B boundary, without announcing that action. Validate a protocol/prior mismatch at construction; no user-provided label may silently create known-mechanism knowledge.
- [ ] **Step 5: GREEN, scoped formatting, review, commit.** Repeat kernel tests; run `cargo clippy -p sugarscape-core --all-targets -- -D warnings`, rustfmt **edited** files only and `git diff --check`. Commit explicitly owned files as `feat(active-surface): add paid probe and live-routine boundaries`. Fresh independent reviewer checks spec and quality before Task 2.

## Task 2: Exact local planning and independent decision reference

**Files:** Create `active_surface/{belief,planner}.rs`, `tests/planner.rs`, small `tests/fixtures/planning-reference.json`; update new namespace exports only. Independent reference scripts/output stay ignored.

**Consumes:** Protocol/Prefix/View/Boundary/Choice/Step/EpisodeState, checked kernel, existing exact fractions. **Produces:**

```rust
pub struct ChoiceValue { pub choice: Choice, pub value: ScoreFraction }
pub struct Decision { pub choice: Choice, pub alternatives: Vec<ChoiceValue>,
    pub continuation_policy: PolicyKind }
pub struct SearchStats { pub candidate_worlds: usize, pub positive_worlds: usize,
    pub decision_states: usize, pub branches: usize }
pub struct CompiledPolicy { /* private domain, prefix/value/decision tables */ }
impl CompiledPolicy {
    pub fn build(protocol: &Protocol, own_prior: OwnPrior) -> Result<Self, Error>;
    pub fn infer(&self, prefix: &Prefix) -> Result<Belief, Error>;
    pub fn decide(&self, view: &View) -> Result<Decision, Error>;
    pub fn stats(&self) -> &SearchStats;
}
```

`infer`/`decide` accept no actual environment, sequence, peer prior/history or evaluator record. Tables are immutable after compilation. Private hypothesis domain retains 4×256 original candidate IDs; a point prior gives only256 positive records. Preserve original weights through all local partitions; zero-mass candidates do not contribute or execute.

- [ ] **Step 1: Dispatch the independent reference at execution, before showing it new implementation.** A separate fresh reference author gets approved spec, physical public interfaces, exact chosen kernel semantics and pre-mortem questions, not new planner/source/results. It independently enumerates all five policies, both role assignments, four models and DataFlip controls, all256 sequences; retains full alternative values and designated full local traces in ignored evidence. Independent reviewer checks its arithmetic and exposure disclosure. It may read frozen physical semantics but must not copy production planner logic or expected outputs. Run this work alongside implementation only after plan approval.
- [ ] **Step 2: Write failing decision/belief tests.** Pin signed numeric comparison and exact ties:

```rust
#[test]
fn signed_values_compare_numerically() {
    let a = ScoreFraction::new(-1, 2).unwrap();
    let b = ScoreFraction::new(-1, 3).unwrap();
    assert_eq!(a.checked_cmp(b).unwrap(), std::cmp::Ordering::Less);
}
#[test]
fn point_control_retains_domain_but_only_one_model_has_mass() {
    let p = Protocol::new(Role::B, PolicyKind::Known);
    let c = CompiledPolicy::build(&p,
        OwnPrior::PointMass(Mechanism::SharedPersistent)).unwrap();
    assert_eq!(c.stats().candidate_worlds, 1024);
    assert_eq!(c.stats().positive_worlds, 256);
}
```

Add exact independent fixture cases for continue/stop and inspect/attempt value ties, own-bit conditioning once, a live observation that changes later-trial choices, and target prediction ties. Full-prefix reference enumeration must agree with cached values at every reachable decision. Use distinct physical/responder states with equal model marginals to prove the cache does not merge them. Change only the actual peer's private bit/history before it could affect an own observation: the experimenter's view/value/choice must remain identical. Test malformed past choices, extra/future observations and forged current bits/credits versus well-formed unsupported observations.
- [ ] **Step 3: Run RED.** `cargo test -p sugarscape-core active_surface::tests::planner`; retain genuine failing evidence. Do not assign guessed numerical fixture values; use reviewed independent reference projections with source/output identities.
- [ ] **Step 4: Implement bounded decision recursion.** Enumerate possible own outcomes by transitioning each candidate through the same kernel under an intervention. Partition by the resulting **whole own prefix**, including original own prior, own choices, credits/clock/issued bits and public stop events. Cache complete normalized candidate domains/positions with full prefix initially; do not introduce a lossy marginal-only cache. A branch's probability is its original positive mass divided by incoming positive mass. At the trial prediction checkpoint expected reward is12 times exact maximum target mass; subtract paid experimenter costs only and recurse into later trials. Future bit issue creates local outcome branches; it never queries actual future bits.

```rust
// Checked arithmetic pattern for each possible own-outcome branch.
let weight = ScoreFraction::new(branch_mass as i64, incoming_mass)?;
let branch_value = continuation.checked_add(
    ScoreFraction::new(-i64::from(experimenter_spent), 1)?)?;
let weighted = weight.checked_mul(branch_value)?;
total = total.checked_add(weighted)?;
```

Use checked conversion for counts rather than unchecked casts if extending bounded counts. Return ArithmeticOverflow on invalid cross-products. Known/Adaptive choose maximal value with frozen ties. NoProbe forces Stop at probe boundaries then optimizes remaining live choices; its actual path stops at completed0. InspectOnly forces Stop and Inspect. FixedThree forces Continue through round3 and then chooses its original threshold routine; its **physical behavior** must match the frozen Unknown/Known three-round comparison. Baseline forced choices need not maximize recorded alternatives: validation recomputes that policy's rule, not the Adaptive rule. `continuation_policy` explicitly labels every alternative: intervene with that one legal choice, then follow this declared policy on subsequent boundaries. Even a forced baseline's unchosen Continue alternative uses its own subsequent stopping rule; it does not silently optimize future actions. Signed terminal values include costs without treating model entropy as reward.
- [ ] **Step 5: GREEN, independent exact check, review, commit.** Run planner tests, bounded whole-prefix oracle comparison, scoped core Clippy and edited-file format checks. Keep independent numerical predictions in `pre-mortem.md`, clearly labeled derived/before measurement. Commit `feat(active-surface): choose experiments by exact remaining net utility`; fresh task reviewer checks boundaries, branch weights, intervention updates and both role clocks.

## Task 3: Correct responder replay and exhaustive actual evaluation

**Files:** Create `active_surface/{replay,evaluation}.rs`, `tests/{replay,evaluation}.rs`, small `tests/fixtures/evaluation-reference.json`; extend new types/exports.

**Consumes:** CompiledPolicy and kernel, candidate-local histories/priors, original-mass partitions and immutable decision tables. **Produces:**

```rust
pub struct Replay { /* private both-role posterior tables under a solved policy */ }
impl Replay {
    pub fn build(protocol: &Protocol, policy: &CompiledPolicy) -> Result<Self, Error>;
    pub fn infer(&self, prefix: &Prefix) -> Result<Belief, Error>;
}
pub struct DecisionTrace { pub prefix: Prefix, pub decision: Decision }
pub struct AgentMetrics {
    pub spent: u8, pub reward: Option<i64>, pub net: Option<i64>,
    pub correct: Option<u8>, pub reads: u8, pub writes: u8,
    pub waits: u8, pub inspections: u8, pub lineage_reads: u8,
}
pub struct Failure { pub role: Role, pub prefix: Prefix,
    pub last_supported: Option<Belief>, pub spent: [u8; 2] }
pub struct ModelProjection {
    pub final_belief: Belief, pub true_model_mass: Option<Probability>,
    pub uniquely_identified: Option<bool>, pub first_identification: Option<Checkpoint>,
    pub spent_to_identify: Option<u8>,
}
pub struct Episode {
    pub sequence: u16, pub histories: [Prefix; 2],
    pub decisions: Vec<DecisionTrace>, pub metrics: [AgentMetrics; 2],
    pub predictions: [Vec<bool>; 2], pub failure: Option<Failure>,
    pub model_projections: [Option<ModelProjection>; 2],
}
pub struct Aggregate { pub valid_mass: Probability, pub failed_mass: Probability,
    pub spent: [ScoreFraction; 2], pub reward: [Option<ScoreFraction>; 2],
    pub net: [Option<ScoreFraction>; 2], pub group_net: Option<ScoreFraction> }
pub struct Panel { pub protocol: Protocol, pub environment: Environment,
    pub episodes: Vec<Episode>, pub aggregate: Aggregate }
pub fn run_episode(protocol: &Protocol, environment: Environment, sequence: u16,
    policy: &CompiledPolicy, replay: &Replay) -> Result<Episode, Error>;
pub fn evaluate_panel(protocol: &Protocol, environment: Environment,
    policy: &CompiledPolicy, replay: &Replay) -> Result<Panel, Error>;
```

ModelProjection grades in-catalog final beliefs and identification. Known controls record supplied time-zero certainty; unknown controls record actual observed identification or censored None. DataFlip true-model grading is None, while its full nominal-family posterior remains available. A failed role's unavailable final projection is None and its partial belief stays in Failure. All boundary and prediction beliefs are obtainable through the retained own histories/replay; the compact diagnostic records those at their exact prefix references. Public actual environment/sequence above are host-only API inputs; decision APIs still accept View alone.

- [ ] **Step 1: Write failing complete-path tests.** For every hypothetical world, replay the **compiled active policy**, with responder point prior at that candidate model. Group separately by each role's own prior and prefix; count each original world once. Pin a responder belief after an experimenter live choice that differs from frozen controller behavior. Pin private routine choice absence in responder entries and public stop presence in both. In an initial calibration/no-task state, none of the stop outcomes may expose future bits.

```rust
#[test]
fn inspection_only_has_zero_probes_and_perfect_experimenter_accuracy() {
    let p = Protocol::new(Role::A, PolicyKind::InspectOnly);
    let c = CompiledPolicy::build(&p, OwnPrior::Uniform).unwrap();
    let replay = Replay::build(&p, &c).unwrap();
    let panel = evaluate_panel(&p,
        Environment::InFamily(Mechanism::Inert), &c, &replay).unwrap();
    assert_eq!(panel.episodes.len(), 256);
    for episode in panel.episodes {
        assert_eq!(episode.metrics[Role::A.index()].correct, Some(4));
        assert_eq!(episode.metrics[Role::A.index()].spent, 36);
    }
}
```

Repeat both roles/allmechanisms. Add a genuine unsupported own observation at AfterSlot: retain its charge, last supported belief/partial local history and stop before later slots/reset/bits. Failed mass forbids unconditional terminal means; costs include failures. Supported DataFlip wrong predictions are classified as outcomes, not operational failures. Check target truth and realized reward are never appended as ordinary policy feedback. Perturb a sender private bit while holding recipient own bits/other trials constant to distinguish local information dependence from payoff effects.
- [ ] **Step 2: Run RED.** `cargo test -p sugarscape-core active_surface::tests::replay` and `... active_surface::tests::evaluation`; retain separate exits/logs.
- [ ] **Step 3: Implement chronological replay, then actual host evaluation.** Replay solves no new optimization; it executes the previously compiled policy over candidate worlds and records both-local posterior tables. Responder prediction uses this replay's target posterior and the frozen majority/tie rule. Compare experimenter replay posteriors with planning posteriors at all reachable checkpoints. Operational malformed histories error; well-shaped impossible histories return UnsupportedHistory.

```rust
// At an experimenter boundary: pass only its validated own prefix to policy.
let prefix = state.prefix(protocol.experimenter).clone();
let view = View::from_prefix(prefix.clone(), replay.infer(&prefix)?)?;
let decision = policy.decide(&view)?;
select_choice(&mut state, decision.choice)?;
// Host advancement is one step, never the whole-routine convenience.
let observed = advance_one(&mut state)?;
let support: Vec<Result<Belief, Error>> = observed.histories.iter()
    .map(|prefix| replay.infer(prefix)).collect();
```

Inspect each support result before another advance: on UnsupportedHistory construct Episode.failure with exact role/prefix/last-supported beliefs and current costs, set unconditional terminal fields None, and return that partial Episode. Other operational errors propagate normally. At a prediction checkpoint query each own target posterior, choose majority with zero tie, retain that belief/prediction and compute actual reward only in the host. Do not inject that computed reward into subsequent View.

The host creates a World from actual environment/bits, then invokes only role-local inference/choice APIs. At each paid own observation query support **before** advancing any subsequent slot. Complete an episode through four final predictions; if any role fails, all unconditional terminal episode rewards/nets are unavailable, while partial outcomes stay explicitly labeled. Track privileged World.read_lineage only in evaluator records. Compute exact aggregate counts, paired policy differences and separately labeled uniform-prior summaries without pooling roles.
- [ ] **Step 4: Compare full references and old fixed control.** Compare all 46settings×256 episodes against independently derived exact costs, target outcomes, stopping choices/values, and designated full local traces. For FixedThree, compare underlying physical events/costs/predictions against frozen shared-surface asymmetric Unknown/Known cal3 for both orientations; additional new own-choice/public-stop entries are explicitly described new apparatus metadata, never silently stripped to hide an observed difference. Test opaque-ID renaming on full typed episode reconstruction and local policies; retain original/mapping/normalized-hash evidence with row counts. Invariance does not mean new affordance learning.
- [ ] **Step 5: GREEN, review, commit.** Core replay/evaluation tests, full exact reference comparison, core Clippy, scoped formatting and diff check. Commit `feat(active-surface): replay solved policies and evaluate exact outcomes`; fresh reviewer checks privacy, stop inference, prediction timing, failure means and response costs.

## Task 4: Compact checked diagnostic, CLI, and precollection gate

**Files:** New diagnostic/protocol/reference files and `tests/diagnostics.rs`; new CLI `src/active_surface.rs`, `tests/active_surface.rs`; additive main dispatch; new namespace exports.

**Consumes:** 46 Panel/Episode records, both-local replay, decisions/alternative values, SearchStats and exact reference projections. **Produces:**

```rust
pub struct DiagnosticReport {
    pub version: String, pub protocol_version: u16,
    pub supplied_structure: Vec<String>, pub histories: Vec<Prefix>,
    pub panels: Vec<PanelReport>, pub checks: Vec<Check>, pub passed: bool,
}
pub struct PrefixRef { pub history: u64, pub entry_count: u64,
    pub checkpoint: Checkpoint }
pub struct CompactDecision { pub prefix: PrefixRef, pub decision: Decision }
pub struct BeliefRef { pub prefix: PrefixRef, pub belief: Belief }
pub struct EpisodeRef { pub sequence: u16, pub histories: [u64; 2],
    pub decisions: Vec<CompactDecision>, pub metrics: [AgentMetrics; 2],
    pub beliefs: Vec<BeliefRef>, pub model_projections: [Option<ModelProjection>; 2],
    pub predictions: [Vec<bool>; 2], pub failure: Option<Failure> }
pub struct PanelReport { pub protocol: Protocol, pub environment: Environment,
    pub episodes: Vec<EpisodeRef>, pub aggregate: Aggregate }
pub struct Check { pub name: String, pub passed: bool, pub detail: String }
pub fn frozen_settings() -> Vec<(Protocol, Environment)>;
pub fn diagnose() -> Result<DiagnosticReport, Error>;
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error>;
```

Set version `active-surface-diagnostic-v1` and protocol_version1. Histories retain each role's complete append-only entries **once**; references reconstruct shorter own-prefix checkpoints without duplicating a whole prefix at every decision. Interner uses typed equality; IDs/prior belong in the identity. Checked indices/counts must reject overflows, out-of-range history/entry references and suffix/future disclosure. If inference checkpoints beyond boundary choices are emitted, use the same exact reference mechanism. Raw local entries and decisions remain reconstructible; privileged evaluator lineage/environment records are labeled and never reconstruct a controller view.

- [ ] **Step 1: Define frozen settings and bounded failing payload tests.** Primary order: Mechanism::ALL, [A,B] experimenter, [Adaptive,FixedThree,NoProbe,InspectOnly,Known]. Secondary order: [A,B], [Adaptive,NoProbe,InspectOnly] on DataFlip. Validate 40/6settings and256 unique sequence rows each, not just total length. Small test-only construction helpers can emit subsets for tests; production integrity/CLI must never accept a subset as a complete passed report.

```rust
#[test]
fn frozen_census_has_exact_declared_scope() {
    let settings = frozen_settings();
    assert_eq!(settings.len(), 46);
    assert_eq!(settings.iter().filter(|(_, e)|
        matches!(e, Environment::DataFlip)).count(), 6);
}
```

Create bounded complete Episode/Panel fixtures from public evaluation interfaces for current-payload tests. Change an alternative value, choice, posterior, credit, public Stop, private responder entry, prefix reference, scope row or aggregate independently; a genuine validator must reject it even when every cached flag says true. Verify complete reconstruction, same underlying histories under ID renaming, unsupported-terminal None fields and known time-zero certainty versus censored unknown discovery.
- [ ] **Step 2: Write CLI/emitter and success tests before freeze.** Add `active-surface diagnose` without tuning flags; reject missing/extra modes/options. Unit tests cover incomplete or forged-passed payload exit2, write/flush failures exit1 and validated report exit0. Success integration test must be normal/nonignored source authored now but **not executed before first collection**. Use exclusive tempfile/stdout streaming, typed BufReader decode and integrity replay; never retain both the multi-GB byte vector and decoded report.

```rust
// Production emitter pattern: actual payload integrity, newline, explicit flush.
report.passed = report_integrity(report)?;
serde_json::to_writer(&mut *writer, &report)?;
writer.write_all(b"\n")?;
writer.flush()?;
// Map operational I/O to existing CLI Failure::Io, false integrity to Invalid.
```

The new CLI follows existing clap Args/Subcommand and existing Failure mappings. Author `active_surface_diagnose_success` under CLI tests and compile it during filtered tests; until first collection run bounded/unit filters and CLI usage filter only. Neither `diagnose()` nor an unfiltered workspace suite runs before the gate.
- [ ] **Step 3: Run RED, then implement report/CLI.** Use `cargo test -p sugarscape-core active_surface::tests::diagnostics`, `cargo test -p sugarscape-cli --bin sugarscape active_surface::tests`, and usage-only `cargo test -p sugarscape-cli --test active_surface active_surface_usage`. Emitter recomputes integrity; a forged success flag cannot bypass it. Validator rebuilds policies/replay under the report's exact protocol, reconstructs each own prefix, verifies alternative values/selection according to PolicyKind, sequence uniqueness, histories, metrics, totals and reference projections. It does not require FixedThree to be optimal or every adversarial prediction to be correct.
- [ ] **Step 4: Full panel-only verification before report collection.** Expose a separately filtered test `precollection_grid_verification` that evaluates settings/reference/ID invariance and writes exclusive compact proof receipts to the owned evidence folder. It does not build/serialize DiagnosticReport. Keep per-setting identity, runtime/test/source manifest SHA, counts, exact aggregate oracle agreement, decision-table equality, designated traces and reference provenance. Computational reference independence and author exposures are disclosed. Preserve this first verification even if an implementation fix requires version2 receipts.
- [ ] **Step 5: Fresh independent whole-source precollection review and commit.** Review exact source/settings/test hashes, Cargo.lock, protocol version, full oracle/pre-mortem, 46setting/11,776episode proofs, raw observation contract, both role clocks, public stop/private routine, checked rational costs and compact reconstruction. Resolve findings with covering tests/re-reviews; retain approval in `precollection-review.md`. Commit `feat(active-surface): expose a checked frozen experiment diagnostic`. No result guide or production diagnostic before this gate. Normal success test remains authored and unexecuted.

## Task 5: Retained first run, measured guide, full verification, and delivery

**Files:** Create `docs/active-surface.md`; append only a short guide link to `docs/shared-surface.md`. Update this plan/status; remove only the owned root execution tracker after completion. Source/settings/test files stay frozen.

**Consumes:** Approved precollection source, compiled policies/diagnostic and normal CLI success test. **Produces:** Retained first/repeat reports, complete per-setting measured results, independent final approval and an exact pushed commit with completed CI/Pages.

- [ ] **Step 1: Freeze actual source/settings/oracle/toolchain, build and copy release binary.** Use owned ignored evidence `.superpowers/sdd/2026-10-07-active-surface-experiments/`, exclusive filenames, complete relevant source/test/fixture/Cargo.lock hashes, current committed-source identity and explicit dirty-status handling. Production first report must use the independently reviewed committed runtime/test source. Preserve original prior-experiment artifacts and read provenance. Retain final protocol settings/priors/tie rules and copied source/binary SHA before invocation.
- [ ] **Step 2: Collect once, then repeat from the same frozen binary.** Commands are the literal new CLI invocation, not a test helper:

```bash
FIRST_BINARY=.superpowers/sdd/2026-10-07-active-surface-experiments/first-sugarscape
"$FIRST_BINARY" active-surface diagnose
```

The collector creates `first-active-surface.json`/stderr exclusively before spawning that command, waits for exit, retains partial or failed output, and never auto-retries a failed first collection. On success create separate `repeat-active-surface.json`/stderr from the same binary. Use bounded streaming hash/byte comparison; read actual report typed integrity via the CLI and recorded validator, not an untrusted passed tail alone. Retain exact counts, bytes, wall runtime and accurately scoped resource measurements. Recheck source/binary hashes before both calls and after collection. Compute guide aggregates with a compact typed/projection reader or same-frozen per-setting proof projections whose identities are independently tied to the full emitted report; disclose extraction method. Never silently overwrite original reports/settings.
- [ ] **Step 3: Old-output preservation.** Regenerate seven old deduction/run stdout/stderr into new after-files; byte-compare all seven baselines. Regenerate original `shared-surface diagnose` into a separately retained file and stream-compare with preserved original first/report hash; do not parse its5GB output into a giant byte buffer. Check old namespace/fixture hashes and original guide prefix; lib/main changes must be purely additive. Preserve any observed divergence as a finding before a fix/revised collection.
- [ ] **Step 4: Run nine gates, now including normal success test.** Retain command/cwd/exits/stdout/stderr/log SHA; independent groups may parallelize with normal Cargo locks. Source is still frozen, so do not enable/add/edit success-test code now. Release CLI precedes web parity.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo build --release -p sugarscape-cli
# In web/:
npm run wasm
npx tsc --noEmit
npx vitest run
# In repository root:
python3 -m unittest discover -s studio/tests -t studio
```

Keep existing ignored tests visible. Do not count precollection numerical comparison or reference authors' tests as ordinary workspace tests. No survey formatting or new survey execution is required by this CLI-only increment.
- [ ] **Step 5: Write product methods/results and final review.** Guide exposes actual stopping choices and alternative values, paid evidence versus task exploitation, known time-zero controls, model-information/accuracy/net-benefit distinctions, responder and group costs, both role orientations, public phase information, nominal-SP DataFlip and failure availability. Keep derived pre-mortem predictions separate from measured results; do not require nonzero benefits. Link primary research anchors without replication claims. Document the supplied probe grammar/known helper/meanings and physical versus computation costs. Record first/repeat/source identities and no after-result revisions, or disclose retained revised series. New guide-link append preserves old guide bytes. Fresh final whole-branch reviewer checks source, original observation contracts, mathematical comparisons, all local gates and first/preservation artifacts; fresh independent reviews resolve material findings.
- [ ] **Step 6: Integrate approved work and verify exact delivery.** Inspect main status, active operation/lock markers and fetched remote history. Preserve concurrent work; never stash/discard it to make integration convenient. Merge into main, inspect actual source differences and run merged checks/report preservation warranted by them. Resolve actual code conflicts through an implementer/reviewer with covering evidence; do not silently call conflict resolution unchanged experimental source. Push normally, resolve remote main via `git ls-remote`, watch CI for that exact SHA and applicable Pages deployment. Retain URLs/conclusions and final source hashes. Remove the owned execution tracker only when all five stages are complete, keeping a completed snapshot in evidence. Preserve crowd and all evidence. If integration/network is blocked, keep an accurate handoff and incomplete status; do not claim delivery.

## Execution setup and review handoff

1. This written plan awaits the user's review. Their spec approval already fixes one experimenter/fixed responder and the selected execution method; it does not approve an unreviewed plan or numerical results.
2. On plan approval read this plan's ledger first, inspect Git status and retained artifacts, verify the existing crowd worktree/branch, and record actual HEAD. Do not recreate it or restart completed work. Init only this plan's SDD workspace with its plan-path identity.
3. Create root `IMPLEMENTATION_PLAN.md` with the five stages above, concrete deliverables/checks and Not Started/In Progress/Complete statuses. Update after reviewed stages and preserve it before eventual removal.
4. Each task uses a fresh implementer, genuine RED/GREEN, self-review, scoped commit and independent spec/quality review. Dispatch the independent numeric author/reviewer separately, without exposing new production planner/results to the author. Runtime fixes belong to implementers, not the controller session.
5. Retain briefs, interfaces, reports, review packages, exact hashes/logs and rulings. Do not delete any evidence at completion. Record any contract ruling explicitly and disclose its cost if wrong. No registered scientific/empirical replication is claimed by this exact engineering study.
