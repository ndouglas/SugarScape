# Deduction Capabilities Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Deliver a capability-composed deduction engine, measurable diagnostics, and a playable single-seat JSON controller interface.

**Architecture:** Add a standalone core module with private engine state and serializable requests, separate scenario assembly, and controllers that receive no Engine reference. A CLI host runs or pauses that engine and saves privileged replay archives. Four sequential tasks have independently reviewable behavioral tests.

**Tech Stack:** Existing Rust workspace; serde/serde_json, rand/rand_pcg, clap, existing integration test conventions.

**Spec:** `docs/superpowers/specs/2026-10-04-deduction-capabilities-design.md` (read in full before implementation).

## Global Constraints

- Use existing Rust, serde, serde_json, rand, rand_pcg, and clap dependencies; add no dependencies.
- Keep this experimental module outside ModelKind, ModelConfig, global presets, and the browser UI.
- Use Agent in code and documentation, American English, and plain scenario names.
- Preserve unrelated studio/README.md and crowd-series documentation edits. Do not stage .claude, papers, or survey output. Do not commit without an explicit request.
- Execute the implementation plan with subagents and maintain IMPLEMENTATION_PLAN.md during execution; remove it when complete.

Work directory for every command: `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/crowd`. Existing baseline: `cargo test --workspace` passed before this implementation. No burrow module exists in this worktree. The user's instruction to proceed authorizes execution; do not add another approval handoff. The standing no-commit instruction overrides the skill's routine commit steps.

## Review Focus

- Invalid hidden-state probes must preserve RNG, buffers, pending request, and every private memory; test in Task 1 and counterfactual views in Task 2.
- Several simultaneous statuses/accusations must have stable precedence independent of response collection details; test boundary collisions in Task 1.
- Zero memory and zero attention are valid ablations, while nonfinite Bayesian input must fail explicitly; test in Tasks 2 and 3.
- A policy may have private state but cannot read the archive, seed, or another seat's request; test in Tasks 3 and 4.
- EOF, stale replies, and a tampered/version-mismatched partial replay must not silently restart or corrupt play; test in Tasks 1 and 4.

## File map and shared interfaces

Task 1 owns `deduction/{mod,types,config,engine,replay}.rs` and core `lib.rs`. Task 2 owns `deduction/{observation,scenario}.rs` and adds observation hooks to engine. Task 3 owns `deduction/{policy,diagnostics}.rs`. Task 4 owns CLI `deduction.rs`, CLI `main.rs` integration, and `docs/deduction.md`. Each task may update public exports and its integration tests, but must preserve earlier contracts. Use inline module atom tests and focused integration files; do not refactor existing models.

The exact Engine, controller, action, phase, and response signatures are in the spec. Additional shared signatures:

```rust
pub fn wink_config(agent_count: u16) -> ScenarioConfig;
pub enum PolicyKind { Evidence, Random, Reckless, Passive }
impl BuiltinController { pub fn new(kind: PolicyKind, seed: u64) -> Self; }
pub fn posterior(prior: &[f64], likelihood: &[f64])
    -> Result<Vec<f64>, InferenceError>;
pub fn best_accusation(posterior: &[f64], correct: f64, wrong: f64, abstain: f64)
    -> Option<usize>;
pub fn diagnose() -> DiagnosticReport;
pub fn replay(archive: &ReplayArchive) -> Result<Engine, ReplayError>;
```

`DiagnosticReport` is serializable and records experiment version, frozen rules, seed range, policy/opponent identities, exact fixture results, per-policy full-game totals, paired differences, and hypothesis outcomes. Observation must include public capability definitions and objective capability ID/conditions, so policies never hardcode capability 0 or a Wink role. Status other than Inactive is active for scheduling/targeting, including Silenced. Define support structs next to their owning code and export only what the next task consumes. `ActionError` has one public variant, `InvalidResponse`; privileged Config/Replay/Inference errors may be descriptive.

---

### Task 1: Transactional capability engine and replay

**Files:** Create core `src/deduction/mod.rs`, `types.rs`, `config.rs`, `engine.rs`, `replay.rs`; modify `crates/sugarscape-core/src/lib.rs`; create `crates/sugarscape-core/tests/deduction.rs`; create root `IMPLEMENTATION_PLAN.md` with these four stages and update it throughout.

**Interfaces:** Produces all core types and Engine/replay methods from the spec. Requests initially project public roster, own permissions/budget, and an empty event memory; Task 2 replaces that projection with the isolated observation module. Final type shapes must already carry the observation/event fields. Scenario labels never appear in engine branches.

- [x] **Step 1: Write failing atom tests and small fixture builders.** Within engine tests, define `fixture()` returning a validated three-Agent config: Agent 0 owns capability 0; capability 0 sets Inactive with Watched visibility and delay 1; all Agents can accuse once; max rounds 4; accuser team is Agents 1 and 2. Add `respond(engine, action)` that copies actor/request_id from `request()` and calls `submit`, and `finish_phase_with_passes(engine)` that repeatedly passes until the starting phase/round changes. Keep these helpers local to tests; production code has no omniscient fixture shortcuts.

```rust
#[test]
fn wrong_actor_is_atomic() {
    let mut engine = Engine::new(fixture(), 7).unwrap();
    let before = engine.archive();
    let request = engine.request().unwrap();
    let bad = TurnResponse {
        request_id: request.request_id,
        actor: (request.actor + 1) % 3,
        action: Action::Pass,
    };
    assert_eq!(engine.submit(bad), Err(ActionError::InvalidResponse));
    assert_eq!(engine.archive(), before);
    assert_eq!(engine.request(), Some(request));
}
```

Add separate tests for stale request, illegal phase, duplicate watch targets, unknown capability/Agent, exhausted accusation, each invalid config bound/reference, and unknown JSON fields. Tests assert no mutation and a single generic response error. Add delay 0 vs 1 tests, inactive-target no-op after accepted simultaneous use, Inactive-over-Silenced precedence, same-target simultaneous correct accusations, objective tie order, and grant/revoke changing own legal permissions. Test effect semantics reused with capability ID 7 and an unrelated label. Add replay tests for partial phase buffers, version mismatch, forged action, and hash mismatch.

- [x] **Step 2: Run the new tests red.** `cargo test -p sugarscape-core deduction`. Expected: unresolved module/types initially; after scaffolding, behavioral assertions fail until implemented. Record actual failure, not just command execution.

- [x] **Step 3: Implement config/types and private engine state.** Use serde tagged enums with snake_case vocabulary, `deny_unknown_fields` where applicable, ordered Vec/BTreeMap collections, and the spec's integer bounds. Give Agent state status, grants, accusation budget, attention, and private memory. Use a private buffered action per actor per phase and deterministic round cursor. Validate the entire response against the outstanding private request's public/own facts before touching buffers/RNG.

```rust
pub fn submit(&mut self, response: TurnResponse) -> Result<(), ActionError> {
    self.validate_response(&response)?;
    self.accept_response(response);
    self.advance_until_request_or_outcome();
    Ok(())
}
```

Implement `validate_response`, `accept_response`, and `advance_until_request_or_outcome` as private engine methods. Boundary resolution follows the spec exactly: phase-start validity, ordered delayed effects, complete Discuss accusations, then objective precedence. Constructor draws any configured hidden assignment through `crate::rng::seeded`; no global random calls. Pure request projection must never draw RNG.

- [x] **Step 4: Implement replay/checksum.** Archive normalized config, seed, versions, accepted responses and terminal checksum. Replay reconstructs with Engine::new and submit; check complete archive fingerprint. Use explicit stable serialization into FNV-1a, covering all state including a sample from a cloned RNG. Reject bad archives rather than accepting their stored hidden state. Ensure archive equality can support the atomicity tests.

```rust
#[test]
fn paused_game_replays_the_same_request() {
    let mut engine = Engine::new(fixture(), 19).unwrap();
    respond(&mut engine, Action::Watch { agents: vec![1] });
    let restored = replay(&engine.archive()).unwrap();
    assert_eq!(restored.request(), engine.request());
    assert_eq!(restored.fingerprint(), engine.fingerprint());
}
```

- [x] **Step 5: Verify and review task.** Run `cargo fmt --all -- --check` and `cargo test -p sugarscape-core deduction`; fix warnings/failures. Review transactionality and phase rules before marking Stage 1 complete. Leave changes uncommitted.

### Task 2: Private evidence, bounded memory, and Wink assembly

**Files:** Create core `src/deduction/observation.rs`, `scenario.rs`; modify `engine.rs`, `mod.rs`, and core `tests/deduction.rs`.

**Interfaces:** Consumes existing Engine/event/config contracts. Produces `wink_config` and final private TurnRequest observations. Observation hooks are internal to engine; no raw world state is exported to policies. Engine owns all RNG consumption, passing detection outcomes to observation helpers or calling their explicitly supplied RNG argument at resolution only.

- [x] **Step 1: Write failing visibility and scenario tests.** In observation unit tests, construct one use from Agent 0 toward Agent 2 with Agent 1 watching 0. Detection 1000 exposes the complete use only to watcher and actor; detection 0 exposes it only to actor. Recipient visibility omits source. Public collapse carries no cause. Repeat with Agent 1 watching 2 to prove attention is selective. Test two differently numbered hidden assignments that produce identical observations have byte-identical request payloads (except own known permissions when these differ, which is legitimately visible).

```rust
#[test]
fn wink_is_seeded_and_replayable() {
    let config = wink_config(6);
    let engine = Engine::new(config.clone(), 31).unwrap();
    let other = Engine::new(config, 31).unwrap();
    assert_eq!(engine.fingerprint(), other.fingerprint());
    assert_eq!(engine.request(), other.request());
}
```

Add tests for oldest-first same-round memory eviction, age cutoff equality, capacity 0, attention 0, claims surviving speaker elimination, acceptance of a deliberate false SawUse claim, no trust promotion of claims into direct sightings, and hidden pending effects not changing legal targets or response errors. A phase earlier actor must not leak its buffered choice to a later actor.

- [x] **Step 2: Run red.** `cargo test -p sugarscape-core deduction`. Expected: missing observation/scenario module or failed visibility assertions; confirm the tests actually exercise the new behavior.

- [x] **Step 3: Implement observation filtering and memory.** Filter use events at resolution, assign only observer-local sequence IDs, deliver public status/claims at their specified boundaries, then prune by age and capacity. Separate direct use, recipient notice, status change, and attributed claim variants. Build LegalActions from public roster/phase and own permissions only. Keep request generation pure.

```rust
// Eviction order is insertion/observer sequence order, never hidden event order.
memory.retain(|event| round.saturating_sub(event.round) <= retention_rounds);
let excess = memory.len().saturating_sub(capacity);
memory.drain(..excess);
```

- [x] **Step 4: Implement scenario assembly as data.** `wink_config(6)` uses the exact defaults in the spec. Keep display name apart from mechanical fields. Add a second small assembly test with accusation disabled, changed capability visibility/delay, and renamed capability to prove construction is reusable; it runs to its configured horizon through the same engine. No new ModelKind or global preset/title entry.

- [x] **Step 5: Verify and review task.** Run `cargo fmt --all -- --check`, `cargo test -p sugarscape-core deduction`. Review hidden-state equivalence tests and actual serialized requests; mark Stage 2 complete. Leave changes uncommitted.

### Task 3: Controller policies and a frozen diagnostic ladder

**Files:** Create core `src/deduction/policy.rs`, `diagnostics.rs`; modify `mod.rs`, core `tests/deduction.rs`; add experiment definition to `docs/deduction.md` before first full-game run.

**Interfaces:** Consumes TurnRequest/TurnResponse only for controller decisions; produces Controller, PolicyKind/BuiltinController, posterior, best_accusation, diagnose/DiagnosticReport signatures listed above. The diagnostic host may inspect outcomes and archive for scoring; it must never pass those privileged values into controller decisions.

- [x] **Step 1: Write exact-reference tests before policies.** Test independently computed posterior, decision utility, and abstention tie behavior. Add length mismatch, NaN/infinity, negative mass and zero evidence mass tests for posterior. best_accusation returns None for malformed/non-normalized probabilities, nonfinite utilities, empty inputs, and utility ties with abstention; document this bounded helper behavior.

```rust
#[test]
fn noisy_evidence_has_an_exact_reference() {
    let p = posterior(&[1.0 / 3.0; 3], &[0.75, 0.25, 0.25]).unwrap();
    for (actual, expected) in p.iter().zip([0.6, 0.2, 0.2]) {
        assert!((actual - expected).abs() < 1e-12);
    }
    assert_eq!(best_accusation(&p, 1.0, -1.0, 0.0), Some(0));
    let reference: f64 = p.iter().enumerate()
        .map(|(truth, mass)| mass * if truth == 0 { 1.0 } else { -1.0 }).sum();
    assert!((reference - 0.2).abs() < 1e-12);
    assert_eq!(best_accusation(&[1.0 / 3.0; 3], 1.0, -1.0, 0.0), None);
}
```

Test that Evidence responds identically to cloned private requests regardless of the surrounding hidden worlds; that claims alone do not trigger a direct-evidence accusation; and that all four policies generate valid responses over complete seeded games. Include policy name/version in report metadata, not outcome grading logic.

- [x] **Step 2: Run red.** `cargo test -p sugarscape-core deduction`. Expected unresolved helpers then failing expected-utility/policy checks.

- [x] **Step 3: Implement finite inference and request-only policies.** posterior divides each prior×likelihood by the total after validation. best_accusation calculates `p * correct + (1-p) * wrong`; choose a strictly better value than abstention and lowest-index remaining tie. Implement phase-aware policies from the spec with separate seeded policy RNGs. Evidence watches a rotating active other Agent, reports direct observations at most once, and accuses only on retained direct threat-capability evidence. Tests must not give it hidden holder IDs.

```rust
let weights: Vec<f64> = prior.iter().zip(likelihood)
    .map(|(p, l)| p * l).collect();
let total: f64 = weights.iter().sum();
// Validate finite positive total before division.
let normalized: Vec<f64> = weights.into_iter().map(|w| w / total).collect();
```

- [x] **Step 4: Freeze experiment, then collect.** Before calling diagnose, document `deduction-v1`, Wink defaults, seeds 0..128, fixed random-target/no-talk/no-accusation threat behavior, and Evidence/Reckless/Passive civilian comparisons. Declare measured quantities and the two directional hypotheses exactly as in the spec. Derive policy seeds deterministically from world seed, policy identity, and Agent ID using fixed integer arithmetic; record the derivation. Use separate controller RNG so engine random draws do not depend on policy RNG consumption. Implement complete games through request/respond/submit; report exact diagnostics plus whole-game totals and paired differences. Record failed hypotheses as failed; do not tune until they pass.

- [x] **Step 5: Verify and review task.** Run `cargo test -p sugarscape-core deduction`; compare two diagnose serializations for exact equality; review report numerical definitions (e.g. civilian loss excludes the hidden holder). Run `cargo fmt --all -- --check`. Mark Stage 3 complete and preserve first experiment results for final reporting. Leave changes uncommitted.

### Task 4: Runnable CLI, single-seat privacy, and final verification

**Files:** Create `crates/sugarscape-cli/src/deduction.rs`, `crates/sugarscape-cli/tests/deduction.rs`; modify CLI `main.rs`; finish `docs/deduction.md`; update/remove root `IMPLEMENTATION_PLAN.md` only after all stages pass.

**Interfaces:** Consumes Engine/scenario/policy/diagnose/replay APIs. Produces CLI modes and flags exactly as in the spec. CLI owns BufRead/Write I/O; core never reads stdin. Keep run/play controllers alive across requests rather than recreating policy state on every turn.

- [x] **Step 1: Write failing binary tests.** Follow existing `tests/cli.rs` command/scratch-directory convention. Cover deterministic `deduction run --scenario wink --seed 7 --policy evidence`, diagnose JSON metadata, and an EOF-paused play request for Agent 0. Spawn with piped stdin/stdout to send one valid response and a stale copy; verify stale rejection preserves the outstanding request. Tests parse JSON rather than matching incidental human text.

```rust
#[test]
fn play_exposes_only_the_selected_seat() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["deduction", "play", "--scenario", "wink", "--seed", "7",
               "--agent", "0", "--policy", "evidence"])
        .stdin(std::process::Stdio::null()).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    let first: serde_json::Value = serde_json::from_str(text.lines().next().unwrap()).unwrap();
    assert_eq!(first["actor"], 0);
    assert!(first.get("seed").is_none());
    assert!(first.get("fingerprint").is_none());
    assert!(first.get("assignments").is_none());
}
```

Add archive write/pause/resume preserving the exact next request; resume with incompatible overrides => exit 2; malformed JSON followed by valid response => recoverable retry; unavailable/inactive requested seat behavior => public terminal/pause output without dumping another Agent's view; corrupted archive => exit 2; unwritable path => exit 1. Confirm no full archive is printed by play or its errors.

- [x] **Step 2: Run red.** `cargo test -p sugarscape-cli --test deduction`. Expected unknown subcommand until command wiring is added.

- [x] **Step 3: Implement host command wiring.** Add `mod deduction` and `Command::Deduction(...)` in main.rs. Keep clap structs and host loop in the new module. Return existing Failure or a mapped result following current 0/1/2 conventions. Play prints/flushes only selected-seat requests; internal controllers handle other seats. On EOF, write archive if requested and return success without implicitly passing the selected Agent. On invalid reply, stderr gets generic protocol rejection and stdout re-emits the unchanged request. Resume reconstructs engine and policy states by replaying prior requests through controllers (or records/restores policy state explicitly); simply resetting controller RNG/state at the paused engine is incorrect.

```rust
// Host sequencing; each policy receives only this request.
while let Some(request) = engine.request() {
    let response = if Some(request.actor) == selected_agent {
        write_request_and_read_response(&request, input, output)?
    } else {
        Some(controllers[usize::from(request.actor)].respond(&request))
    };
    let Some(response) = response else { break }; // EOF: pause, never pass.
    if engine.submit(response).is_err() { report_invalid_response(errors)?; }
}
```

Implement the two I/O helpers as private CLI functions; handle flush/read/write failures with context. Archive remains optional and explicitly privileged. Run/replay/diagnose outputs can contain host-level checksums, but play protocol cannot.

- [x] **Step 4: Document real operation.** Run all four commands, paste a real selected-seat request and valid response into `docs/deduction.md`, explain EOF/resume and archive privacy, list exact rules/source limitations, and include actual initial diagnostic findings with seed/policy metadata. State that the Bayesian diagnostic is a finite reference case and full-game Evidence is a direct-sighting policy. Describe external controllers as JSON hosts with no bundled network provider.

- [x] **Step 5: Verify completed work.** Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`. Run `cargo run -p sugarscape-cli -- deduction diagnose` and `cargo run -p sugarscape-cli -- deduction run --scenario wink --seed 7 --policy evidence`; inspect valid JSON and reproducibility. Run `git diff --check`. If baseline clippy warnings exist, distinguish them from new warnings rather than unrelated refactoring. Remove IMPLEMENTATION_PLAN.md after marking all stages complete, review scoped diff and untracked files, and report test evidence, empirical hypothesis results, and limitations. Leave changes uncommitted.
