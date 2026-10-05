# Deduction capabilities: first executable experiment

## Intent and boundary

Build an experimental Rust capability engine, a diagnostic ladder, and a playable Wink-inspired scenario. The experiment asks whether Agents can perceive, remember, infer, communicate, and choose actions whose consequences reward better play. Human, built-in, and external controllers use the same private observation and validated action contract. No provider API is needed.

This is the approved first deliverable, not the eventual catalog of social deduction games. Theme is presentation metadata. Hidden assignments, permissions, status changes, observables, and objective conditions are separate data. Future roles can combine those pieces; this deliverable does not implement a universal rules language, full AIWolf protocol, learned behavioral model, renderer, or video.

Global constraints:

- Use existing Rust, serde, serde_json, rand, rand_pcg, and clap dependencies; add no dependencies.
- Keep this experimental module outside ModelKind, ModelConfig, global presets, and the browser UI.
- Use Agent in code and documentation, American English, and plain scenario names.
- Preserve unrelated studio/README.md and crowd-series documentation edits. Do not stage .claude, papers, or survey output. Do not commit without an explicit request.
- Execute the implementation plan with subagents and maintain IMPLEMENTATION_PLAN.md during execution; remove it when complete.

## Existing patterns and choice

`model.rs` is a closed registry connecting configuration, rendering, presets, checkpoints, and statistics. Integrating here would multiply unrelated changes before the experiment earns a visual model. Add `pub mod deduction` to core and a CLI subcommand instead. The checked crowd worktree has run/sweep/shot commands and no burrow command.

`image/world.rs` keeps per-observer records distinct from true state. Its inspection view is privileged, so it must not be reused as a controller observation. `minds/memory.rs` supplies the useful precedent of bounded memory with deterministic oldest-first eviction. `rng.rs` provides the seeded PCG stream; existing world fingerprints use FNV-1a. CLI integration tests invoke the real binary through `CARGO_BIN_EXE_sugarscape`.

## Module boundaries and contracts

Files under `crates/sugarscape-core/src/deduction/`:

| File | Responsibility |
| --- | --- |
| `mod.rs`, `types.rs` | Public exports and serializable controller vocabulary |
| `config.rs` | Bounded scenario data and validation |
| `engine.rs` | Private state, transactional action acceptance, rounds, effects, outcomes |
| `observation.rs` | Visibility filtering, attention, bounded memory |
| `scenario.rs` | Wink assembly and small diagnostic fixture assembly |
| `policy.rs` | Controllers consuming only TurnRequest, explicit inference helpers |
| `diagnostics.rs` | Frozen experiment definition, exact reference cases, seeded reports |
| `replay.rs` | Privileged archive, deterministic replay, fingerprint |

`crates/sugarscape-cli/src/deduction.rs` owns host I/O. Integration tests live in `crates/sugarscape-core/tests/deduction.rs` and `crates/sugarscape-cli/tests/deduction.rs`; atom tests stay next to their implementation. A usage document lives at `docs/deduction.md`.

Public API (all controller types derive Clone, Debug, PartialEq, Serialize, Deserialize):

```rust
pub type AgentId = u16;
pub type CapabilityId = u16;
pub type Round = u32;
pub enum Phase { Attention, Act, Discuss }
pub enum Status { Active, Inactive, Silenced }
pub enum Action {
    Pass,
    Watch { agents: Vec<AgentId> },
    Use { capability: CapabilityId, target: AgentId },
    Say { claim: Claim },
    Accuse { target: AgentId },
}
pub enum Claim {
    Suspect { agent: AgentId },
    SawUse { source: AgentId, target: AgentId, round: Round },
    DenyUse { round: Round },
}
pub struct TurnResponse { pub request_id: u64, pub actor: AgentId, pub action: Action }
pub struct TurnRequest {
    pub protocol_version: u16, pub request_id: u64,
    pub actor: AgentId, pub round: Round, pub phase: Phase,
    pub observation: Observation, pub legal: LegalActions,
}
impl Engine {
    pub fn new(config: ScenarioConfig, seed: u64) -> Result<Self, Vec<FieldError>>;
    pub fn request(&self) -> Option<TurnRequest>;
    pub fn submit(&mut self, response: TurnResponse) -> Result<(), ActionError>;
    pub fn outcome(&self) -> Option<&Outcome>;
    pub fn archive(&self) -> ReplayArchive; // privileged host API
    pub fn fingerprint(&self) -> u64;       // privileged host API
}
pub fn replay(archive: &ReplayArchive) -> Result<Engine, ReplayError>;
pub trait Controller { fn respond(&mut self, request: &TurnRequest) -> TurnResponse; }
```

`Observation` contains public capability definitions (effects, delays, targeting and visibility), public objective capability ID/conditions, public roster/statuses, own grants and remaining accusation budget, own visible events in memory, public rule summary, and own objective identifier. It never contains seed, hidden assignment, hidden pending effects, other Agents' attention, private histories, full-state hash, or internal event numbering. Visible events get observer-local contiguous sequence numbers, round, and typed content. `LegalActions` describes public target IDs, allowed phase actions, own usable capability IDs, attention capacity, and remaining accusation count. It is descriptive, not an enumeration over hidden state.

## Capability and scenario data

`ScenarioConfig` contains version, display name, `agents: Vec<AgentSpec>`, `capabilities: Vec<CapabilitySpec>`, `observation: ObservationRules`, `memory: MemoryRules`, `accusation: AccusationRules`, `objectives: ObjectiveRules`, and `max_rounds`. Agent IDs are contiguous from zero. Each AgentSpec has its initial grants and objective team; optional seeded assignment chooses exactly one holder of one configured capability among a public eligible set. Assignment is performed once at construction; the selected holder receives the threat objective team and every non-holder the accuser team for this seeded assembly. The engine does not branch on role or scenario names.

CapabilitySpec fields: ID; typed `Effect` (`SetStatus { status: Inactive | Silenced }` or `Grant { capability }` or `Revoke { capability }`); delay in complete rounds; `TargetRule` (`SelfOnly` or `OtherActive`); and `Visibility` (`Public`, `Recipient`, or `Watched`). Effects change state; visibility independently controls evidence of use. A capability may be configured but granted to nobody. Grant/revoke can change permissions without changing identity. Use status setters idempotently; Inactive is terminal, and no first-version effect resurrects an Agent. Silenced Agents retain physical actions but cannot speak or accuse. Granting an unknown capability is invalid configuration.

ObservationRules has `attention_capacity` and `detection_per_mille` (integer 0..1000). MemoryRules has event capacity and retention rounds. AccusationRules has enabled, budget per eligible Agent, eligible IDs, and `wrong_cost` (`SpendOnly` or `DeactivateAccuser`). ObjectiveRules names the capability whose current holder is the accusation target, the eligible accuser team, a non-holder-survivor threshold, and the horizon winner. Engine objective evaluation uses current capability possession, never a role string. Configuration must ensure at least one target holder initially and an eligible accuser; grant/revoke scenarios may create multiple holders and a correct accusation removes that target's threat grant, with the accuser team winning when no active threat holders remain.

No unbounded callbacks or expression parser. This small typed vocabulary is deliberately extensible and independently controllable. Tests must show an effect reused under a different capability ID/label and grant/revoke changing legal actions without replacing a role. Tests may disable accusation or all active capabilities and still run to the horizon.

## Round resolution

Throughout scheduling, targeting, and survivor counts, active means status is not Inactive; Silenced Agents remain physically active. Rounds start at zero. Each phase schedules every currently active Agent in ascending ID order; actor scheduling never depends on hidden grants. Collect actions into a phase buffer. Earlier submissions cannot change another actor's same-phase view. Attention accepts Watch or Pass (Pass watches nobody), Act accepts Use or Pass, Discuss accepts Say, Accuse, or Pass. One action per Agent per phase. Attention applies for that round only.

At the end of Act, resolve accepted uses in ascending source ID. Capture all valid-at-phase-start uses before applying effects, so a same-phase elimination cannot cancel an already accepted action. A use at round r with delay d is due at the end of Act in round r+d. Existing due effects precede newly due effects; ties use creation order (round, source ID). SetStatus(Inactive) dominates Silenced. Grant/revoke of the same capability follow creation order. Effects whose target is inactive become harmless no-ops. No-op uses still produce the same configured use evidence and spend the action, without exposing hidden reasons.

At the end of Discuss, publish claims and resolve accusations in ascending accuser ID. Correct accusations revoke the target threat capability. Wrong accusations spend one token and optionally deactivate the accuser. All buffered accusations resolve even if another resolution deactivates an accuser. Repeated correct accusations at the same target are judged against the phase-start grants, so they do not become wrong merely because an earlier accusation succeeded. Evaluate outcomes after the complete boundary: no active threat holders => accuser victory; else non-holder active count at/below threshold => threat victory; else after max_rounds completed rounds => configured horizon winner. This order breaks simultaneous victory ties explicitly. Before the next Attention phase, expire memory and skip inactive actors. Empty phases advance automatically. No request exists after termination.

## Observation, memory, claims, and privacy

A watched use is detected only if its source is in the observer's Watch set, then with the configured probability. At 0 and 1000 there is no random ambiguity. Detected use exposes source, target, capability ID, and round, but not hidden grants or eventual result. The user's own accepted use is always known to them. Recipient visibility exposes a recipient notice with target and capability but no source. Public status transitions disclose status and Agent ID, never cause, attacker, or pending effect. Use evidence and status evidence are separate typed events. Public and recipient visibility do not also generate a watched event unless explicitly represented by a separate capability; avoid implicit double evidence.

Claims are attributed utterances, never trusted world facts. Suspect, SawUse, and DenyUse support a bounded first conversation. Validate only syntax, known public IDs, and past/current rounds. Do not reject a lie for conflicting with private truth. Claims become visible only at the Discuss boundary. Public claims can outlive their speaker within memory limits. Built-in inference trusts direct observations; treating others' claims as evidence requires an explicit source-reliability model and is outside this first policy.

Store only events delivered to each Agent. Evict oldest visible sequence first after applying age cutoff (`current_round - event_round > retention_rounds`), then capacity. Capacity zero is valid and yields no retained events. Observation projection is pure and consumes no RNG. Public roster/current status remain available even if memory is empty. The engine's memory bound describes what the server supplies; an external controller may retain its own past requests, so benchmarks comparing memory must use declared policies with matching retention, not claim to erase a human's memory.

No hidden-state action validation: publicly active other Agents remain targetable even when a delayed effect is pending. Grants are known to their holder. Unknown target IDs, self targets disallowed by the public rule, duplicates/oversized watch sets, unauthorized capabilities, wrong phase, invalid claims, exhausted accusations, stale/wrong actor requests all return the same external `ActionError::InvalidResponse`. Validation occurs before state mutation or RNG consumption. Invalid responses preserve request, buffered actions, archive, fingerprint, and random stream. Error details may identify malformed JSON in the host, but never a hidden-state reason. Tests compare byte-identical requests and errors for counterfactual hidden assignments compatible with the same observation.

## Wink-inspired scenario

`wink_config(agent_count: u16) -> ScenarioConfig` assembles “Watch for the wink”: six Agents by default (accepted range 4..12), one uniformly seeded holder of capability 0, all other Agents on the accuser team. Capability 0 is a watched SetStatus(Inactive), OtherActive, delay one. Every active Agent may watch one Agent, use its grants, discuss, and make at most one accusation. A wrong accusation deactivates its accuser. Detection is 750/1000, event memory capacity 64 and retention 8 rounds, horizon 12 rounds, threat wins if at most one non-holder remains or at horizon. Own capability possession tells the holder its objective; other Agents know only their own lack of that grant. The holder may also speak and accuse; objective evaluation is unaffected by the speaker's claimed role.

The precise phase structure, stochastic detection, delayed collapse, budgets, and victory thresholds are our reconstruction, not a claim to implement every traditional rule. The source describes secret winking, detective and general accusation variants, and different penalties; that variation motivates spelling out our choices. [Wink murder](https://en.wikipedia.org/wiki/Wink_murder).

Structured, attributed claims draw inspiration from AIWolf's vocabulary for estimates, role claims, actions/results, and agreement/request/reasoning forms. We implement only the three Claim variants above, not an AIWolf parser or compatibility promise. [AIWolf Protocol 3.6](https://aiwolf.org/control-panel/wp-content/uploads/2019/05/protocol_3_6.pdf).

## Policies and diagnostic ladder

Policies get only TurnRequest and their own private policy state. Ship `PolicyKind::{Evidence, Random, Reckless, Passive}` and `BuiltinController::new(kind, seed)`. Evidence watches a rotating public candidate, uses threat capability against a publicly active non-self target if it owns one, accuses an active source only after direct observed use of the objective capability, otherwise passes; it may report its last direct sighting once. Random samples legal actions using its separate policy RNG. Reckless accuses the lowest available other ID without evidence. Passive passes. Threat behavior in comparative experiments is frozen separately: choose a random publicly active target each round, no claims or accusations. The role assignment seed and policy seed are separate and recorded.

1. **Atoms:** permission, grant/revoke, target rule, attention, detection endpoints, delay, status precedence, local visibility, eviction/retention, claim vs fact, budgets, and objectives. Each has an isolated deterministic fixture with a known outcome and an independently changed parameter.
2. **Perception/inference:** a small exact finite hypothesis helper `posterior(prior: &[f64], likelihood: &[f64]) -> Result<Vec<f64>, InferenceError>` normalizes elementwise products, rejecting length mismatch, nonfinite/negative inputs, and zero evidence mass. Test uniform three-way prior and likelihoods `[0.75, 0.25, 0.25]` => `[0.6, 0.2, 0.2]`. This is an explicitly noisy source-label diagnostic, not the no-false-positive Wink observer. The fixture supplies the observation channel and labels; it does not inspect a hidden truth to manufacture posterior output.
3. **Decision:** `best_accusation(posterior: &[f64], correct: f64, wrong: f64, abstain: f64) -> Option<usize>` compares posterior-weighted utility with abstention, ties favor abstention and then lowest index. For reward +1, cost -1, abstain 0, `[0.6,0.2,0.2]` chooses 0 for expected 0.2; uniform thirds abstains, while reckless index 0 has expected -1/3. Enumerate all three hidden states weighted by the posterior to calculate the independent reference utility. This limited Bayesian helper is honest groundwork, not a claim of solved multi-round Bayesian social reasoning.
4. **Seeded whole games:** freeze experiment version, all rules, policies, seed set `0..128`, and criteria in source/documentation before collecting initial results. Pair Evidence, Reckless, and Passive civilian policies against the same frozen threat policy for each assignment/world seed. Report captures, threat wins, civilian losses, false accusations, rounds, and paired differences. Predeclared expectation: Evidence captures more often and makes fewer false accusations than Reckless; report whether it holds without adjusting seeds, rules, or thresholds after inspection. Do not make a stochastic superiority claim merely because the policy has a flattering name. The exact expected-utility diagnostic is the hard correctness gate; whole-game comparisons are an empirical report that may fail their hypothesis.

Each diagnostic result declares its layer, rules version, seed range, opponent behavior, measured quantity, reference or criterion, observed value, and pass/hypothesis outcome. Include separate measured fields: watched uses eligible/detected/missed for perception fixtures; maximum absolute posterior error against the exact reference for inference; and exact reference utility minus chosen utility for decision regret. Whole-game logs report direct evidence availability alongside choices, without claiming to attribute every loss to one cognitive cause. Never score inaccessible truth as if a controller should have observed it. Full-game output is evidence of the specified experiment only.

## Validation, replay, and host

Bounds: 2..32 Agents for general configs; 1..32 capability definitions (unused capabilities permitted); IDs unique and in range; max_rounds 1..1000; delays 0..1000; attention 0..agent_count-1; memory capacity 0..4096 and retention 0..1000; accusation budget 0..32; detection 0..1000. Validate every referenced Agent/capability, seeded eligible list uniqueness and nonemptiness, objective references, and meaningful initial assignments. Unknown serialized fields are rejected. Check effect due-round arithmetic. Config validation emits existing FieldError values; untrusted input must not panic.

ReplayArchive is privileged and explicitly labeled in documentation. Fields: protocol/rules version, complete normalized config, engine seed, accepted TurnResponses, and final full-state fingerprint. Archive partial games at a paused request; replay reconstructs the exact same pending request. Invalid responses are not accepted history. Fingerprint uses explicit stable encoding/FNV-1a covering config, seed, accepted transcript, hidden assignment, round/phase/cursor, buffers, statuses/grants, due effects, attention, local memories, outcome, and RNG continuation (e.g. fixed next-output samples from a cloned RNG). Avoid unordered maps and platform-dependent hashes. Reject version mismatch, invalid replay actions, or fingerprint mismatch descriptively in the privileged replay API. It is a reproducibility checksum, not authentication.

CLI modes:

```text
sugarscape deduction run --scenario wink --seed 7 --policy evidence
sugarscape deduction play --scenario wink --seed 7 --agent 0 --policy evidence [--archive FILE]
sugarscape deduction diagnose [--out FILE]
sugarscape deduction replay FILE
```

Run drives all Agents and prints a public result JSON. Play drives other seats internally, writes only the designated Agent's TurnRequest as one JSON line, reads one TurnResponse per line, and retries rejected responses without advancing. EOF ends cleanly at the outstanding request; `--archive` saves a privileged resumable transcript. A later `play --resume FILE --agent 0` resumes it; reject incompatible scenario/seed overrides. CLI session files wrap ReplayArchive with host version, designated seat, built-in policy kind and policy seed derivation. Reconstruct built-in policy state by replaying prior requests and calling each internal controller on each of its historical turns before applying the archived response; selected-seat human responses do not advance an internal controller. Reject a changed designated seat or policy on resume. Stdout contains protocol/result JSON only; explanations go to stderr. Never emit the other seats' private requests, world seed, fingerprint, or archive on play stdout. The local host necessarily holds privileged state; this is an observation boundary, not a security sandbox against a user reading their own archive. Human users can paste JSON, and an external LLM host can drive the same JSON lines without any core change.

Exit conventions match existing CLI: 0 success/pause, 1 I/O, 2 configuration/protocol/archive validation. Integration tests cover malformed input, EOF/pause/resume, mismatched actor, and single-seat isolation. Usage documents include an actual request/response example produced by the implemented serialization, not a hand-waved format.
