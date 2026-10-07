# Shared-Surface Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. A fresh implementer and independent task reviewer gate each task; an independent whole-increment review gates delivery. Steps use checkbox tracking. The user approved this written plan on October 7, 2026; execute using the preserved subagent-driven method.

**Goal:** Build an exact, inspectable two-Agent experiment that separates learning a supplied shared surface, transmitting task information, and saving recipient inspection costs.

**Architecture:** Add a standalone `shared_surface` core namespace and additive `shared-surface diagnose` CLI command. Replay a synchronized finite candidate ensemble in public chronological order; compute each candidate Agent's beliefs by grouping only its local prefix, never recursively re-simulating peers. Keep privileged transitions, local controller DTOs, inference, evaluation, and losslessly interned diagnostics separate.

**Tech Stack:** Existing Rust, serde, serde_json, clap, Rust tests; independently authored Python standard-library `Fraction` enumerator. No new dependency, RNG, optimizer, web UI, Minds API change, or ModelKind integration.

**Spec:** `docs/superpowers/specs/2026-10-06-shared-surface-design.md`, approved October 6, 2026.

## Global Constraints

- Use existing `.claude/worktrees/crowd`, branch `crowd`; inspect status before edits. Never recreate, discard, or stash unrelated work; retain this worktree and all evidence.
- Four in-family mechanisms: shared persistent (SP), shared resetting (SR), private persistent (PP), inert (IN). Fixed model order SP, SR, PP, IN. Four independent live trials, uniform X/Y bits, calibration lengths 0, 1, 2, 3.
- Alphabet: blank, probe0, ack0, probe1, ack1, data0, data1. Blank is not writable. Generic write acceptance supplies no delivery or persistence evidence.
- Every Agent starts with 48 credits per episode. Read/write/wait cost 1; own-target inspection costs 4; prediction costs 0. Correct prediction earns 12. Public resets clear fields, not model memory.
- Supplied participation uses P(SP)>1/2; equality inspects. Exact target-posterior ties predict zero. B's read choice uses its pre-read posterior; its subsequent write choice uses its updated posterior.
- Controllers receive only their local transcript, public schedule/reset events, own budget/current private bit, and supplied prior/protocol. They never receive privileged world state, peer realized history, true model, evaluator lineage/rewards, host seed, or archive.
- An informed candidate peer uses delta at the candidate mechanism, never delta at the actual hidden mechanism. Uniform unknown prior; no silent resets, smoothing, false receipt, future-bit access, or fallback for unsupported histories.
- Operational errors fail explicitly; controller failure stops that rollout and retains trace/spending. Terminal metrics are unavailable, not zero. Report out-of-family coverage separately.
- Preserve all existing deduction schemas, algorithms, seven frozen command outputs, and product guide prefixes. No unrelated refactor, experiments, survey edits, or `cargo fmt` invocation in survey.
- American spelling; Agent in code/docs. Science boundary: supplied roles/probes/codec/controller, learned finite mechanics. Fully inspecting baseline has perfect accuracy; benefit is possible cost saving, not greater gross correctness.
- Freeze protocol/source/settings and independent references before first diagnostic collection. Retain first/repeat bytes and disclose any subsequent revision. Never stage ignored evidence, `.claude/`, `papers/`, or `survey/out/`.
- Commit working increments frequently. Once the approved feature is complete, merge into main, push, and verify CI for the exact commit. Inspect concurrent main changes and markers first; preserve all unrelated work.
- Create a 3–5-stage root `IMPLEMENTATION_PLAN.md` only after this plan is approved, update it during execution, and remove it at completion. Stop and reassess after three unsuccessful attempts on one issue.

## Review Focus

1. A public scheduled peer opportunity is not the peer's realized action; candidate inference must preserve private-state equivalence (Task 2).
2. A known model at time zero is supplied information, not paid discovery; no-communication unexplored ambiguity and censored discovery remain distinct (Task 3).
3. Same-round visibility does not imply delayed persistence; B may change its send decision after a read, including asymmetric participation (Tasks 1–2).
4. Stale point masses can fail and out-of-family flipped data can be confidently wrong without failure; terminal metrics and aggregate availability must preserve that distinction (Tasks 2–3).
5. Interned traces, exact fractions, episode references, and passed flags can be malformed or edited; current report integrity must reject inconsistent payloads without losing raw histories (Task 4).

## File ownership and common contracts

Create `crates/sugarscape-core/src/shared_surface/{mod,types,world,protocol,inference,controller,evaluation,diagnostics}.rs`, `shared_surface/diagnostics/{protocol,references}.rs`, and `shared_surface/tests/{world,inference,evaluation,diagnostics}.rs`. Create bounded independent JSON fixtures in `shared_surface/tests/fixtures/`. Modify `crates/sugarscape-core/src/lib.rs` only for `pub mod shared_surface;`.

Create `crates/sugarscape-cli/src/shared_surface.rs` and `crates/sugarscape-cli/tests/shared_surface.rs`; modify `crates/sugarscape-cli/src/main.rs` only for the additive module, subcommand, and dispatch. Create `docs/shared-surface.md` only after first collection; append a brief link to `docs/deduction.md` without changing its existing bytes. Evidence belongs in ignored `.superpowers/sdd/2026-10-06-shared-surface/`.

The following signatures are shared contracts, implemented by their owning task. All wire DTOs derive Clone/Debug/PartialEq/Eq/Serialize/Deserialize with unknown fields rejected; custom deserializers validate fractions and indices. Prefix/key types also support deterministic ordering for grouping. Keep fields private where invariants require constructors. Reexport public names from mod.rs; do not expose World through the controller API.

```rust
pub enum Mechanism { SharedPersistent, SharedResetting, PrivatePersistent, Inert }
pub enum Environment { InFamily(Mechanism), DataFlip }
pub enum Role { A, B }
pub enum Symbol { Blank, Probe0, Ack0, Probe1, Ack1, Data0, Data1 }
pub enum Action { Read, Write(Symbol), Wait, InspectOwnTarget }
pub enum Outcome { Read(Symbol), Accepted, Waited, Inspected(bool) }
pub enum Phase { Calibration, Live { trial: u8 } }
pub struct Position { pub phase: Phase, pub round: u8, pub slot: u8 }
pub enum Checkpoint { EpisodeStart, Action(Position), Prediction { trial: u8 } }
pub enum OwnPrior { Uniform, PointMass(Mechanism) }
pub struct Ids { pub surface: String, pub agents: [String; 2] }
pub struct Event { pub position: Position, pub action: Action, pub outcome: Outcome,
                   pub credits_after: u8 }
pub enum LocalEntry { Reset { phase: Phase }, PrivateBit { trial: u8, bit: bool },
                      Action(Event) }
pub struct LocalPrefix { pub role: Role, pub ids: Ids, pub checkpoint: Checkpoint,
                         pub own_prior: OwnPrior, pub entries: Vec<LocalEntry> }
pub enum Knowledge { Unknown, Known, NoCommunication }
pub struct Pair { pub roles: [Knowledge; 2] }
pub enum PriorMode { Treatment, RestartUniform, Stale(Mechanism) }
pub struct Protocol { pub calibration_rounds: u8, pub pair: Pair,
                      pub prior_mode: PriorMode, pub ids: Ids }
pub struct EpisodeBits(pub [(bool, bool); 4]); // (X, Y), indexed by trial
pub struct Probability { pub numerator: u64, pub denominator: u64 }
pub struct ScoreFraction { pub numerator: i64, pub denominator: u64 }
impl Probability { pub fn new(numerator: u64, denominator: u64) -> Result<Self, Error>; }
impl ScoreFraction { pub fn new(numerator: i64, denominator: u64) -> Result<Self, Error>; }
pub struct Belief { pub models: [Probability; 4], pub target: Option<Probability> }
pub struct LocalView { pub prefix: LocalPrefix, pub credits: u8,
                       pub private_bit: Option<bool>, pub belief: Belief }
pub struct Decision { pub action: Action }
pub enum Error { InvalidProtocol(String), InvalidAction(String), InvalidReport(String),
                 UnsupportedHistory { prefix: LocalPrefix }, ArithmeticOverflow }
pub fn decide(view: &LocalView, knowledge: Knowledge) -> Result<Decision, Error>;
pub fn predict(view: &LocalView) -> Result<bool, Error>;
pub struct Ensemble; // internal candidate states, no privileged getters
impl Ensemble {
    pub fn build(protocol: &Protocol) -> Result<Self, Error>;
    pub fn infer(&self, prefix: &LocalPrefix) -> Result<Belief, Error>;
    pub fn candidate_count(&self) -> usize;
}
pub struct World; // privileged evaluator/transition seam only
impl World {
    pub fn new(environment: Environment, ids: Ids) -> Self;
    pub fn apply(&mut self, role: Role, position: Position, action: &Action,
                 target: Option<bool>, credits: u8) -> Result<Event, Error>;
    pub fn reset(&mut self, phase: Phase);
    pub fn finish_round(&mut self);
}
pub struct LocalStep { pub prefix: LocalPrefix, pub belief: Option<Belief>,
                       pub decision: Option<Action>, pub prediction: Option<bool>, pub credits: u8 }
pub struct AgentTrace { pub role: Role, pub own_prior: OwnPrior, pub steps: Vec<LocalStep> }
pub struct Lineage { pub writer: Role, pub write_position: Position, pub symbol: Symbol,
                     pub task_trial: Option<u8> }
pub struct PrivilegedStep { pub role: Role, pub event: Event,
                            pub read_lineage: Option<Lineage> }
pub struct FailureTrace { pub role: Role, pub checkpoint: Checkpoint,
                          pub prefix: LocalPrefix, pub prior: OwnPrior,
                          pub last_supported_belief: Belief, pub spent: [u8; 2] }
pub struct AgentTerminal { pub predictions: [bool; 4], pub correct: [bool; 4],
                           pub gross_reward: i64, pub net_utility: i64 }
pub enum Property { Visibility, Retention, UsefulChannel, TrueMechanism }
pub enum Certainty { Supplied, Acquired { checkpoint: Checkpoint, credits: u8 }, Censored }
pub struct Discovery { pub property: Property, pub certainty: Certainty }
pub struct AgentMetrics { pub spent: u8, pub attempted_sends: u16, pub reads: u16,
                          pub decoded_data: u16, pub causal_transfers: u16, pub inspections: u16,
                          pub discoveries: Vec<Discovery>, pub terminal: Option<AgentTerminal> }
pub struct Episode { pub sequence: u16, pub local: [AgentTrace; 2],
                     pub privileged: Vec<PrivilegedStep>, pub metrics: [AgentMetrics; 2],
                     pub failure: Option<FailureTrace>, pub group_net_utility: Option<i64> }
pub fn run_episode(protocol: &Protocol, environment: Environment, bits: &EpisodeBits,
                   ensemble: &Ensemble) -> Result<Episode, Error>;
pub struct AgentAggregate { pub expected_credits: ScoreFraction,
                            pub expected_accuracy: Option<Probability>,
                            pub expected_correct_count: Option<ScoreFraction>,
                            pub expected_net: Option<ScoreFraction>,
                            pub expected_gross: Option<ScoreFraction>,
                            pub benefit_gross: Option<ScoreFraction>,
                            pub benefit_net: Option<ScoreFraction> }
pub struct Aggregate { pub total: u16, pub valid: u16, pub failed: u16,
                       pub failure_mass: Probability, pub agents: [AgentAggregate; 2],
                       pub expected_group_net: Option<ScoreFraction> }
pub struct Sensitivity { pub sequence: u16, pub paired_sequence: u16, pub trial: u8,
                         pub sender: Role, pub read_changed: Option<bool>,
                         pub posterior_changed: Option<bool>, pub prediction_changed: Option<bool> }
pub struct Panel { pub protocol: Protocol, pub environment: Environment,
                   pub episodes: Vec<Episode>, pub aggregate: Aggregate,
                   pub sensitivity: Vec<Sensitivity> }
impl Panel {
    pub fn episode_count(&self) -> usize;
    pub fn agent_expected_credits(&self, role: Role) -> ScoreFraction;
    pub fn agent_expected_accuracy(&self, role: Role) -> Option<Probability>;
    pub fn agent_expected_correct_count(&self, role: Role) -> Option<ScoreFraction>;
    pub fn agent_expected_net(&self, role: Role) -> Option<ScoreFraction>;
}
pub fn evaluate_panel(protocol: &Protocol, environment: Environment) -> Result<Panel, Error>;
pub enum PanelKind { Primary, Asymmetric, Restart, Stale, DataFlip }
pub struct EpisodeReference { pub sequence: u16, pub agent_traces: [u64; 2],
                              pub privileged: Vec<PrivilegedStep>, pub metrics: [AgentMetrics; 2],
                              pub failure: Option<FailureTrace>, pub group_net_utility: Option<i64> }
pub struct TransferOrigin { pub old_mechanism: Mechanism, pub acquisition: [AgentTrace; 2],
                            pub posterior: [Belief; 2], pub spent: [u8; 2] }
pub struct PanelReport { pub kind: PanelKind, pub protocol: Protocol, pub environment: Environment,
                         pub origin: Option<TransferOrigin>, pub episodes: Vec<EpisodeReference>,
                         pub aggregate: Aggregate, pub sensitivity: Vec<Sensitivity> }
pub struct DiagnosticCheck { pub name: String, pub passed: bool, pub detail: String }
pub struct DiagnosticReport { pub version: String, pub protocol_version: u16,
                              pub supplied_structure: Vec<String>, pub traces: Vec<AgentTrace>,
                              pub primary: Vec<PanelReport>, pub secondary: Vec<PanelReport>,
                              pub checks: Vec<DiagnosticCheck>, pub passed: bool }
pub fn diagnose() -> Result<DiagnosticReport, Error>;
pub fn report_integrity(report: &DiagnosticReport) -> Result<bool, Error>;
```

`Probability` validates positive denominator and 0<=numerator<=denominator. `ScoreFraction` validates only positive denominator: utilities exceed one in magnitude. Existing `strategy_inference::SignedRatio` is unsuitable because its wire validator restricts absolute numerator<=denominator. Use checked integer sums/products and canonical reduced fractions; comparisons use checked cross multiplication. `OwnPrior` is part of every local-prefix inference key, including time zero. Under Treatment, uniform is required for unknown/no-communication roles and a known role supplies its own lawful point mass explicitly. RestartUniform requires uniform for the frozen restarted unknown pair; Stale is the explicit exception supplying the declared old point mass. `Ensemble::infer` validates its shape against Knowledge/PriorMode, without consulting an actual World. In Treatment candidate replay, an informed peer's prior remains delta_m for each candidate m; an Agent's explicit actual point-mass prior cannot be forwarded into that peer. Stale mode instead requires both supplied point masses to equal the public old mechanism. Wrong IDs or prior treatment combinations are operational InvalidProtocol errors, not zero-evidence scientific findings.

`Checkpoint::Action(position)` means immediately before that public slot; append the action outcome, then advance the public checkpoint. `Checkpoint::Prediction { trial }` occurs after round-3 slot 4 has completed and before the next trial reset or any future private bit arrives. Both roles use their own final local prefix at this public prediction checkpoint. EpisodeStart has no current task bit. These phases must never alias in lookup keys.

The DTOs above are the cross-task minimum; additional measured fields must be typed and documented in the owning task before another task consumes them. LocalStep contains no privileged fields. PrivilegedStep's lineage is evaluator-only. Failure retains the exact prior/last supported belief and unavailable terminal results. Aggregate terminal/accessor Options are None for any nonzero failed mass; spent credits remain measurable. Sensitivity Options are None if the applicable read/posterior/prediction is unavailable, rather than counting absence as no effect. No aggregate accepts a float or invented terminal zero.

Integer candidate counts never exceed 1,024 per configuration; exact likelihood uses these counts, not repeated rational multiplication.

## Frozen execution and report sizes

All panels enumerate all 256 four-trial bit sequences in ascending 8-bit index, interpreting bit 2t as X_t and bit 2t+1 as Y_t. There is no sampling or selective success reporting.

| Panel | Frozen settings | Episode rows |
|---|---|---:|
| Primary | 4 mechanisms × 4 calibration lengths × 3 pairs (unknown/unknown, known/known, no-communication/no-communication) | 12,288 |
| Asymmetric | 4 mechanisms × 4 lengths × 2 pairs (known/unknown, unknown/known) | 8,192 |
| Changed-mechanics restart | 16 ordered old/new mechanisms × 4 new lengths; unknown/unknown after public uniform restart | 16,384 |
| Stale diagnostic | Same 16×4, both roles retain their old full-calibration point mass without restart | 16,384 |
| Data-flip control | 4 lengths × 3 primary pairs; known means supplied point mass SP | 3,072 |

The old episode portion of each transfer setting is the fixed three-round old-model calibration; it records the actual learned old posterior and spent calibration credits. The new episode starts fresh with 48 credits, new fields/bits, and public possible-change event. Old acquisition costs and new relearning costs are separate, not added to the new episode's task utility. No-change old=new settings receive the same restart event. Stale uses publicly specified old point masses for both roles in candidate replay; it does not smuggle the new mechanism into any prior.

A consistently renamed rerun of all 12,288 primary episodes is an invariance check, retained separately, not an extra utility treatment. Paired sensitivity reruns use the complete same panel distribution, flipping one sender private bit in one trial, with all other bits unchanged; reuse matching episodes from that exhaustive panel. Report all direction/trial cases including no change and failures.

Preserve every episode's complete per-Agent trace using a deterministic lossless trace library plus explicit episode-to-trace references. Intern by the complete typed trace including opaque IDs, supplied prior, posterior at each local step, and failure data; first occurrence determines stable ID. Do not intern solely by predictions, model labels, or outcomes. Validate episode/reference counts, trace referential integrity, unique keys, canonical row order and exact reconstruction. Check projected allocations against known protocol counts before growing vectors; no undocumented episode or trace cap. Keep privileged traces separately marked and inaccessible to LocalView.

## Task 1: Finite transitions, local observations, and independently frozen reference

**Files:** Create mod.rs, types.rs, world.rs, protocol.rs, tests/world.rs; additive lib.rs export. Independent oracle author owns only ignored `reference.py`, `reference.json`, `reference-report.md`; fixture packaging is reviewed separately. Implementer owns baseline capture scripts in the ignored evidence directory.

**Interfaces:** Produces world/types/protocol contracts above, complete deterministic public slot schedule, role ownership, reset events, bit-index conversions, and checked exact fraction constructors. No controller or diagnostic execution yet.

- [x] Before any runtime edit, inspect status and hash frozen earlier source/guides. The preserved-source manifest covers the existing deduction subtree, its CLI implementation/tests, and Cargo.lock. Record core lib.rs and CLI main.rs baseline hashes separately: their planned additive export/dispatch edits are checked as scoped diffs rather than required to remain byte-identical. Build the current release CLI and copy it to `before-sugarscape`. Capture seven commands into new files with exclusive creation, exit status, stderr, and SHA256:

```python
commands = [
 ['deduction','diagnose'], ['deduction','testimony'], ['deduction','testimony-game'],
 ['deduction','strategic-reporting'], ['deduction','strategy-inference'],
 ['deduction','adversarial-audit'],
 ['deduction','run','--scenario','wink','--seed','7','--policy','evidence']]
# subprocess.run writes stdout/stderr to files opened 'xb'; retain binary/source hashes.
```

Run `cargo build --release -p sugarscape-cli`. Preserve original guide bytes and Cargo.lock. This baseline build and capture must precede Task 1 source edits, not merely precede the new command.

- [x] Dispatch an independent reference author before production measurements. They read the approved protocol and derive calibration likelihoods, synchronized hidden trajectories, B's chronology, threshold/ties, private-state invariance, costs, unsupported histories, and flipped-data deception using Python Fraction. They do not copy Rust outputs or anticipated values. Retain author/source/output hashes and exposure disclosure. Reviewer independently checks the oracle's complete panel specification before it becomes a fixture. Oracle generation may proceed alongside Tasks 1–3 but must finish before collection and diagnostic success tests.

- [x] Write world tests before implementation, including delayed SP/SR difference, private PP visibility, generic IN acceptance, overwrite lineage, nonwritable blank, phase-forbidden inspection, unknown/malformed identifiers at command/view boundaries, depleted budget, all-role slot ownership, public reset clearing state while controller memory is external, and fraction validation. Because the typed Action does not carry an identifier, validate IDs at protocol validation before World construction and reject mismatched surface/Agent IDs in incoming LocalPrefix at Ensemble::infer; there is no unmodeled addressed-mailbox operation.

```rust
#[test]
fn inert_acceptance_is_not_delivery() {
    let ids = Ids { surface: "s".into(), agents: ["a".into(), "b".into()] };
    let mut world = World::new(Environment::InFamily(Mechanism::Inert), ids);
    let p = Position { phase: Phase::Calibration, round: 1, slot: 1 };
    let event = world.apply(Role::A, p, &Action::Write(Symbol::Probe0), None, 48).unwrap();
    assert_eq!(event.outcome, Outcome::Accepted);
    let q = Position { phase: Phase::Calibration, round: 1, slot: 2 };
    assert_eq!(world.apply(Role::B, q, &Action::Read, None, 48).unwrap().outcome,
               Outcome::Read(Symbol::Blank));
}
```

- [x] Run `cargo test -p sugarscape-core shared_surface::tests::world`; record genuine red (missing namespace or behavior), then implement raw transitions without interpreting symbols. Validate role/slot/legal action/budget before state mutation. Maintain raw write lineage privately for later evaluator use.

```rust
// Core transition rule: model controls only the affected field and round reset.
match environment {
    Environment::InFamily(Mechanism::Inert) => {},
    Environment::InFamily(Mechanism::PrivatePersistent) => private[role_index] = symbol,
    Environment::InFamily(_) | Environment::DataFlip => shared = stored_symbol,
}
// DataFlip transforms only Data0/Data1; calibration symbols remain unchanged.
```

- [x] Rerun targeted tests green; rustfmt only owned Rust files. Independent task review checks local/privileged boundaries, validation before mutation, no stale index assumptions, and oracle independence. Resolve findings with new red/green evidence. Commit only explicit owned paths: `feat(shared-surface): model paid finite surface transitions`.

## Task 2: Exact local-prefix inference and chronological controllers

**Files:** Create inference.rs, controller.rs, tests/inference.rs; extend only owned reexports/types in mod.rs/types.rs. Independent reviewer receives Task 1 contracts, full approved spec, and oracle calibration fixtures when available.

**Interfaces:** Consumes raw transitions and schedule. Produces Ensemble, LocalView/Belief, decide/predict. A public helper validates protocol and constructs exactly 4×256 candidates; informed beliefs filter to that role's candidate model, unknown beliefs use all supported models, no-communication beliefs use uniform prior but never messages.

- [x] Write failing tests for exact calibration posteriors at every local slot, three-round unique identification, SP/SR ambiguity after ack0, B PP/IN ambiguity before round 3, visibility versus retention, exact half threshold, majority ties, and identical local prefix/prior despite different hidden peer bits. Include known/unknown pairs so true-model leakage through informed-peer priors is detectable. Include an unsupported appended read and stale point-mass contradiction.

```rust
#[test]
fn half_probability_inspects_instead_of_sending() {
    let view = fixture_live_a_view([1, 0, 1, 0], 2, Some(false));
    // fixture constructs a legal LocalView with P(SP)=1/2 and no target evidence.
    assert_eq!(decide(&view, Knowledge::Unknown).unwrap().action, Action::InspectOwnTarget);
}
#[test]
fn zero_target_evidence_is_not_a_zero_prediction_probability() {
    let mut view = fixture_live_a_view([1, 1, 1, 1], 4, Some(false));
    view.prefix.checkpoint = Checkpoint::Prediction { trial: 0 };
    assert!(!predict(&view).unwrap()); // target prior is 1/2, tie predicts zero
}
```

`fixture_live_a_view(weights: [u64;4], total: u64, private_bit: Option<bool>) -> LocalView` is a test helper owned here: legal live trial 0, round 1, slot 1, 48 credits, OwnPrior::Uniform, a matching Reset and PrivateBit entry, and target=1/2. Live fixtures require Some(bit); None is rejected because own current bit has already been issued. Prediction fixture uses the terminal Prediction checkpoint; the decision-only unit fixture does not claim an exhausted full protocol prefix. It constructs model fractions weights/total and rejects a mismatched sum. Do not use it as a privileged real-world controller entry point.

- [x] Run `cargo test -p sugarscape-core shared_surface::tests::inference`; retain red evidence. Implement Ensemble as one chronological synchronized pass, bounded at 1,024 candidates. At each local decision checkpoint, group candidate trajectories by `(role, complete_local_prefix, own_prior)` and count model/target masses. Candidate known-role group restricts model to its candidate m; unknown role groups across all four. Include the current public Checkpoint and explicit OwnPrior in the prefix key even at checkpoints with no new local observation; inference must not confuse a pre-read and post-peer-action checkpoint. Peer actions are generated simultaneously from each candidate peer's own corresponding group. No recursive peer inference, posterior multiplication, future decision feedback, or per-query fresh full-world construction.

```rust
// At one chronological checkpoint, candidates have only earlier realized local entries.
let masses = group_local_prefixes(&candidates, role, &protocol)?;
for candidate in &mut candidates {
    let belief = masses.belief_for(candidate, role)?;
    let view = candidate.local_view(role, belief); // contains no World or evaluator fields
    let decision = decide(&view, protocol.pair.roles[role_index])?;
    candidate.apply_local_decision(role, &decision)?;
}
```

`group_local_prefixes`, `belief_for`, `local_view`, and `apply_local_decision` are private Task 2 helpers, not cross-task APIs. Document their invariants with tests; cache immutable posterior/decision tables keyed by the entire local prefix. `infer(prefix)` returns its full-prefix posterior from the original prior, never an update multiplier. All hidden future bits exist only as latent candidate indices; issue only the current trial's permitted private bit when its public start event occurs. Prior/current transcript mass counts fit u64; overflow stays an explicit Error.

- [x] Implement fixed calibration and live rules from schedule; `predict` is legal only at the terminal live prediction checkpoint, where absent target evidence means the fair prior; phase-invalid prediction is an InvalidAction error.  B read at round-2 slot 2 evaluates pre-read belief and B write/inspect at slot 3 reevaluates after its actual read. A reads iff it actually wrote. Inspection gives verified target. PP retained own data is conditioned on its actual provenance in candidate trajectories, not blindly decoded. Live updates retain model evidence across trial resets, while target evidence uses fresh independent bits. Known treatment role priors are supplied; mark identification at time zero separately from learned discovery.

- [x] Add a bounded candidate-count test (exactly 1,024), all exhaustive local-prefix invariance comparisons, known/unknown no leakage, chronological B policy-switch fixture, wait-vs-read differences, and contradictory-history failure with preserved prior/prefix. At time zero, test the same empty-prefix role with each of the four different supplied own point masses: its posterior must equal its own prior, with no actual-model argument to infer. Reject unknown-role point masses and stale-prior mismatches as operational errors. Test `infer(prefix)` against independent full-prefix oracle counts and an independently updated incremental reference: equal posteriors, no double-conditioning. Rerun green, scoped rustfmt, independent review, commit `feat(shared-surface): infer mechanics from local chronological evidence`.

## Task 3: Exact evaluation, causal transfer, costs, and negative controls

**Files:** Create evaluation.rs, tests/evaluation.rs; extend owned types/reexports. Independent oracle author supplies exact reference panels before judged diagnostic collection; implementer uses bounded task fixtures rather than collecting or inspecting the eventual full report.

**Interfaces:** Consumes Ensemble and controllers. Produces Episode, Panel, run_episode, evaluate_panel; types contain local traces, marked privileged lineage, exact per-Agent terminal metrics, grouped comparisons, and failure information. Freeze all panel settings from the count table above in diagnostics/protocol.rs during Task 4; evaluators accept only validated Protocol/Environment.

- [x] Write failing tests for budget totals (maximum 42 per Agent), perfect fully inspecting baseline, zero gross-reward improvement ceiling, matched per-Agent/group net utility, SP delayed other-Agent lineage, SR loss across rounds, PP own-write false delivery, and IN no-op write. Failure metrics use `Option` with explicit reason, never default zero. Aggregates report total/valid/failed episode counts and exact failure mass; if any failure mass is nonzero, unconditional terminal means are unavailable. Any conditional-on-success means carry their denominator and label.

```rust
#[test]
fn fully_inspecting_baseline_accounts_for_waits_and_calibration() {
    let protocol = fixture_protocol(3, [Knowledge::NoCommunication; 2]);
    let panel = evaluate_panel(&protocol, Environment::InFamily(Mechanism::Inert)).unwrap();
    assert_eq!(panel.episode_count(), 256);
    assert_eq!(panel.agent_expected_credits(Role::A), ScoreFraction { numerator: 42, denominator: 1 });
    assert_eq!(panel.agent_expected_accuracy(Role::A), Some(Probability { numerator: 1, denominator: 1 }));
    assert_eq!(panel.agent_expected_net(Role::A), Some(ScoreFraction { numerator: 6, denominator: 1 }));
}
```

`fixture_protocol(calibration_rounds: u8, roles: [Knowledge;2]) -> Protocol` supplies fixed IDs, Pair, PriorMode::Treatment. Panel methods `episode_count`, `agent_expected_credits`, `agent_expected_accuracy`, `agent_expected_correct_count`, and `agent_expected_net` are Task 3 public read-only accessors; their fraction results are checked and canonical. Accuracy is the fraction of correct predictions (1 for the baseline), while expected correct count is 4 for that baseline; both terminal accessors return None if any failed episode mass exists.

- [x] Run `cargo test -p sugarscape-core shared_surface::tests::evaluation`; record red. Implement run_episode by actual chronological transitions, calling Ensemble::infer only on each actual local prefix. It may hold privileged bits/world in evaluator scope; LocalView remains restricted. Query each role independently from its own actual local prefix. Never jointly filter the hypothetical ensemble by both actual private histories: in DataFlip runs the two locally supported explanations may involve different hypothetical bit worlds. Candidate replay remains stationary under the declared prior even when the actual stale-diagnostic environment changes. Unsupported controller histories produce a retained failed Episode, not a top-level operational Error; malformed protocols/actions/arithmetic remain explicit errors.

```rust
// Success has terminal metrics; controller failure does not invent a prediction or reward.
match ensemble.infer(&prefix) {
    Ok(belief) => advance_local_controller(belief)?,
    Err(Error::UnsupportedHistory { prefix }) => return Ok(failed_episode(prefix, spent)),
    Err(error) => return Err(error),
}
// Utility is 12 * correct_count - all_spent_credits; no receive bonus.
```

`advance_local_controller` and `failed_episode` are private evaluator helpers. Failure Episode records failing role/position, original prior, last supported belief, observed failing prefix, both spent credits and all prior traces. Every successful slot is charged exactly once.

- [x] Add discovery records for P(visibility), P(retention), and P(SP) reaching one, plus true-model probability/unique identification. Separate initially supplied certainty from paid acquired certainty; unidentified properties are censored. No-communication policy has uniform unexplored posterior and no acquired-discovery record, not a misleading wrong-model score. Report attempted writes, reads, decoded symbols, actual other-Agent task-bearing lineage, inspections, sender credits, recipient accuracy, gross/net benefit and group utility independently.

- [x] Implement exhaustive sensitivity using existing paired sequence rows: flip sender's current private bit, retain recipient's private bit and all other trials, and compare recipient read/posterior/prediction before terminal truth release. Include asymmetric/missing observations and failures; a valid data symbol alone is not a causal-transfer count. Do not claim reward effect from these flips or erase deterministic writes off-support.

- [x] Add all old/new restart and stale tests, including old=new public reset, exact old acquired point masses, separate acquisition/relearning costs, unsupported changed-model histories, and silent wrong confident DataFlip predictions. IDs renamed consistently throughout local/world/reference DTOs must preserve posterior/action/cost/utility; known supplied IDs carry no semantic role. Rerun green; compare all bounded oracle fixtures, retain task report and independent review, commit `feat(shared-surface): evaluate transfer and paid communication exactly`.

## Task 4: Versioned diagnostic, CLI integrity, and precollection review

**Files:** Create diagnostics.rs, diagnostics/protocol.rs, diagnostics/references.rs, tests/diagnostics.rs, reviewed reference fixture JSON, CLI shared_surface.rs, CLI integration test; additive main.rs dispatch. No collection or results prose in this task.

**Interfaces:** Produces DiagnosticReport, diagnose, report_integrity; report version `shared-surface-diagnostic-v1`, frozen protocol version 1. CLI uses `SharedSurfaceArgs` with a single `Diagnose` subcommand and no tuning flags; run returns existing Failure semantics.

- [x] Write failing bounded diagnostic tests for fixed panel counts/order and exact fraction/schema validation. Test lossless trace reconstruction, separate privileged lineage, duplicate/missing episode keys, out-of-range sequence IDs, unknown trace references, changed trace/posterior/cost, forged passed flags, and changed protocol metadata. Integrity reconstructs from frozen protocol and current payload, not cached flags. Include positive source-independent fixture checks but hold full CLI-success collection until first report is retained.

```rust
#[test]
fn changed_trace_reference_is_rejected_in_the_shared_validator() {
    let mut panel = bounded_panel_fixture(); // reviewed test-only panel, not a production report
    panel.episodes[0].agent_traces[0] = u64::MAX;
    assert!(!validate_panel_for_test(&panel).unwrap());
}
#[test]
fn incomplete_report_never_passes_production_integrity() {
    let report = incomplete_report_fixture();
    assert!(!report_integrity(&report).unwrap());
}
```

`bounded_panel_fixture() -> PanelReport` and `incomplete_report_fixture() -> DiagnosticReport` are test helpers. `validate_panel_for_test(&PanelReport) -> Result<bool, Error>` is private/test-only and invokes shared structural/replay validation without representing a production report. Production `report_integrity` always requires the full frozen settings/counts, and the CLI cannot accept fixture identities or a relaxed scope. `primary: Vec<PanelReport>`, `PanelReport.episodes: Vec<EpisodeReference>`, `EpisodeReference.agent_traces: [u64;2]` are Task 4 report fields; tests may edit them. All panel metadata explicitly indicates scope, denominators, initial priors, and failed/valid masses.

- [x] Run `cargo test -p sugarscape-core shared_surface::tests::diagnostics` and CLI unit emission tests; record red, then implement deterministic panel ordering and full-trace interning. There is no fixture scope or validation bypass in the production schema. Unit tests may call private structural validators on bounded panels; report_integrity and every CLI emission enforce complete frozen counts and schema. Validation can replay cached Ensemble configurations once per protocol, not once per episode; no recursive reconstruction or unchecked allocation from edited wire lengths.

```rust
pub enum SharedSurfaceMode { Diagnose }
pub struct SharedSurfaceArgs { pub mode: SharedSurfaceMode }
// clap derives Args/Subcommand and a required subcommand; no arbitrary model/prior flags.
// main.rs adds Command::SharedSurface(shared_surface::SharedSurfaceArgs)
// and dispatches to shared_surface::run(args).
```

The CLI serializes a typed report and writes it before returning Invalid if engineering checks fail; nonzero exit 2 for failed validation, 1 for I/O, 0 only for valid passed report. Emitter unit tests exercise incomplete/corrupted reports (exit 2) and write errors (exit 1) without invoking full diagnose. Author the normal nonignored `shared_surface_diagnose_success` CLI integration test in this task before freezing test source. Until first collection, run unit/bounded tests by their explicit filters and do not run that success test or an unfiltered workspace suite. The full Task 5 suite runs it without source changes. No hidden DataFlip failures are automatically engineering check failures: expected adversarial failures are valid findings with correct failure classification.

- [x] Package the independently authored oracle and exact checked-in small fixtures with source/output hashes and author exposure disclosure. Compare every setting's exact aggregate reference and designated full local-history traces; retain exhaustive oracle output separately in ignored evidence. Independent task reviewer checks identities, panel counts (56,320 actual episode rows plus renamed 12,288 invariance reruns), no selector leakage, genuine current-payload validation, reference computational independence, and intended unavailable aggregate semantics.

- [x] Before first collection, fresh independent review of complete source/protocol/oracle and observation/cost contract; resolve every finding with tests. Record `precollection-review.md` approval and exact hashes of all runtime/test/settings files, Cargo.lock, source commit, oracle and report protocol. No execution of the full success test, full `diagnose()` invocation or result guide before this gate. Commit `feat(shared-surface): expose a checked frozen diagnostic` when Git metadata is writable; current managed session rejects index.lock creation, so the reviewed source capsule binds this precollection gate.

## Task 5: Preserve first collection, publish measured guide, and integrate

**Files:** Create docs/shared-surface.md; append link to docs/deduction.md; update plan stage checkboxes and remove approved execution tracker. Root owns collection/quality/integration orchestration in ignored evidence; independent final reviewer reviews the whole increment including measured documentation.

**Interfaces:** Consumes approved precollection source/protocol, exact oracle, DiagnosticReport and CLI. Produces retained first/repeat/preservation/verification evidence and a measured product guide; no runtime or settings changes after measurement unless explicitly disclosed and re-reviewed.

- [x] Build release CLI; snapshot source/settings/oracle/binary with SHA256 to `first-collection-source-settings.json` and copy first binary/source. Use exclusive files, never overwrite prior evidence. Run `shared-surface diagnose` into `first-shared-surface.json`, then the frozen same binary into `repeat-shared-surface.json`. Check exit 0, typed report integrity, exact oracle agreement, and byte equality. Report counts/runtime/bytes as measurement metadata, not discovery outcomes. Run the already authored, normal nonignored full CLI success integration test only after these first artifacts exist. Do not enable, edit, or add test source after the runtime/test freeze; targeted filters kept it unexecuted before first collection.

```python
with first.open('xb') as out:
    subprocess.run([str(frozen_binary), 'shared-surface', 'diagnose'], stdout=out, check=True)
with repeat.open('xb') as out:
    subprocess.run([str(frozen_binary), 'shared-surface', 'diagnose'], stdout=out, check=True)
assert first.read_bytes() == repeat.read_bytes()
```

- [x] Regenerate all seven old CLI outputs from the new binary into separate after-files; compare byte-for-byte against before-files. Compare every preserved-source manifest hash and the original deduction guide prefix; verify lib.rs/main.rs baseline diffs contain only the planned additive export/dispatch changes. Verify runtime/settings hashes remained unchanged since first run. Disclose any unavoidable revision, retaining first and revised series rather than relabeling revised output first.

- [x] Write product methods/results from exact measured values, explaining supplied vs learned structure, deterministic designed identifiability, known time-zero certainty, censored discovery, baseline accuracy ceiling, paid wait/inspection costs, asymmetric costs, unavailable failures, silent DataFlip deception, and representation/finite-family limits. Link science anchors without replication claims. Do not turn development failures into product bug history. Append only a short new guide link to existing deduction.md. Independent final reviewer compares claims to retained oracle/first report and verifies old-output preservation and source hashes.

- [x] Run all nine quality gates and retain command/status/logs. Independent builds that share Cargo target locks can run in separate sequential groups; release CLI precedes web parity. These are required checks, not optional repeats:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
cargo build --release -p sugarscape-cli
# cwd web:
npm run wasm
npx tsc --noEmit
npx vitest run
# cwd worktree root:
python3 -m unittest discover -s studio/tests -t studio
```

Record exact test counts from logs rather than copying previous counts. Inspect required ignored Studio fixtures before judging failures; restore only missing byte-identical established fixtures, with provenance, and never edit tests to pass. `git diff --check`, final tracked/staged diff, no prohibited paths, expected source hashes and complete required checks precede the final docs commit `docs(shared-surface): report frozen learning and transfer measurements`.

- [ ] Mark all stages complete/remove root execution tracker, retain final review and verification-summary JSON. Inspect main status, active Git operations/locks, fetch origin and inspect concurrent commits before integrating; never overwrite/discard someone else's main work. Merge approved completed branch into main without destructive checkout, run checks warranted by merged source changes and regenerate preservation/new diagnostic comparisons. Commit conflict resolution only with implementation/review evidence if actual code changes occur.

- [ ] Push normal main update (no force/no hook bypass), resolve the exact pushed SHA with `git ls-remote`, and watch CI for that exact commit to completion. Verify Pages deployment when the workflow runs. Retain CI URLs/raw conclusions and final remote/source hashes. Preserve crowd worktree and evidence. If CI fails, fix through a fresh implementer/reviewer, verify and commit, then push/check the new exact SHA; disclose post-result source changes. Final answer links measured guide, merge SHA and CI evidence with material limits.

## Approval handoff

The user approved this written plan on October 7, 2026 and already selected subagent-driven development. Execution is authorized. Preserve first reports and frozen settings, complete all independent reviews and verification, and follow the standing commit/integration workflow without adding a permission gate.
