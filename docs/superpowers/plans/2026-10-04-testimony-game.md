# Testimony Game Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Deliver a finite testimony decision game, exact evaluation, and an inspectable evolved listener compared with fixed controllers and equal-budget random search.

**Architecture:** Add a namespaced game alongside the existing deduction engine. A transactional session isolates private reporting observations; independent finite enumeration scores listeners against the best attainable decision. Evolution searches a four-gene policy using exact payoff, and a separate diagnostic command reports frozen training and transfer results.

**Tech Stack:** Existing Rust workspace, serde/serde_json, clap, rand and the existing PCG wrapper; Rust integration/unit tests; development-time standard-library Python Fraction references. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-10-04-testimony-game-design.md` (approved; read in full).

## Global Constraints

- Work only in `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/crowd`, branch crowd; retain its current uncommitted baseline.
- No new dependencies, global model/preset registry entries, browser UI, or video assets. Do not change existing Wink requests, claims, policies, sessions, diagnostics, or testimony-v1.
- Exactly two distinct reporters and one distinct decider. Permissions are explicit assembly data; unsupported schedules/permission combinations are errors.
- Probability numerator 0..denominator; denominator 1..16. Deterministic endpoints are valid. Truth priors are independent 1/2; profiles persist; four private signal channels are conditionally independent.
- New public input structs and structured variants reject unknown fields, including fieldless actions. Protocol rejection is generic and atomic.
- Hidden world, host seed, archive, and evaluator truth never enter listener input. Legitimate calibration verification precedes the live decision; live verification follows it.
- Fixed training q=4/5, rho=3/4; holdouts (4/5,1/4), (3/5,3/4), (1/2,3/4). All 256 worlds enumerated; 20 search seeds 0..20; 3,164 evaluations per method per seed.
- GA population 64, 50 replacement generations, two elites, tournament three with replacement, uniform per-gene crossover, independent mutation probability 1/4 with equiprobable +/-1 and clamping.
- Exact fitness first, lower integer L1 norm second, lexicographically smaller genes third. Freeze champions on training fitness before holdouts.
- Required numerical tolerance 1e-12. Learning success, superiority to random search, and generalization are measured outcomes, not correctness gates.
- Agent in code/docs; American spelling. Commit only on explicit request. Never stage .claude/, papers/, or survey/out/. No survey changes.
- Always subagent-driven execution with fresh task implementers and reviewers, then an independent whole-change reviewer. Root owns four-stage IMPLEMENTATION_PLAN.md; remove it after completion.
- Maximum three failed attempts per issue; document and reassess after the third. Preserve real red/green evidence, not reconstructed claims of test-first work.

## Review Focus

- A second reporter must not observe an earlier buffered report in the same exchange. Task 1 compares both calibration and live request views.
- Actual reporting deviations can contradict the Bayesian model at deterministic endpoints. Task 2 returns a model error rather than inventing a posterior, while Task 1 still accepts legal deviations.
- Zero-mass histories cannot have a conditional posterior; deterministic environments must omit them from conditional evaluation. Task 2 tests all q/rho endpoints and no division by zero.
- A search method must not get credit for a larger budget, cached evaluations, or holdout-based selection. Task 3 checks counts and ordering; Task 4 freezes both methods' champions before transfer evaluation.
- Zero regret in an uninformative environment does not mean sensible behavior or calibrated beliefs. Tasks 2/4 retain intervention rates and exclude evolved scores from posterior metrics.

## Files and interface map

Create `crates/sugarscape-core/src/deduction/testimony_game/`:

- `mod.rs`: exports within the new namespace only.
- `types.rs`: strict wire/configuration types, probability/config validation, errors, pure reporting atom.
- `session.rs`: private world generation, actor-specific requests, buffered exchanges, archive/replay.
- `enumeration.rs`: integer-mass finite reference distribution and listener evaluation.
- `listeners.rs`: five fixed listeners, Genome and its evolved heuristic.
- `evolution.rs`: frozen GA/random search, exact ordering, budgets, seed derivation and curves.
- `diagnostics.rs`: frozen environments, known-answer checks, training/transfer collection and report schema.

Modify `deduction/mod.rs` only to add `pub mod testimony_game;`. Other existing exports remain intact. Create core integration tests `tests/testimony_game.rs` and `tests/testimony_game_search.rs`, and CLI integration tests `tests/testimony_game.rs`. Add only the new subcommand/helper to CLI `src/deduction.rs`; append to `docs/deduction.md`. Existing RNG lives in `crate::rng::{seeded, SimRng}`; do not introduce another generator.

All following names live under `sugarscape_core::deduction::testimony_game`, avoiding collisions with existing types. Serializable structs derive Clone/Debug/PartialEq and strict serde; integer-only structs may also derive Eq. Private session state need not serialize or expose RNG. Diagnostic output types may be Serialize-only.

### Task 1 contracts

```rust
pub struct Probability { pub numerator: u16, pub denominator: u16 }
pub struct Permissions {
    pub agent: AgentId,
    pub receive_signal: bool,
    pub report: bool,
    pub observe_verification: bool,
    pub decide: bool,
}
pub struct Config {
    pub version: u16,
    pub reporters: [AgentId; 2],
    pub decider: AgentId,
    pub permissions: Vec<Permissions>,
    pub accuracy: Probability, // q
    pub copy_prior: Probability, // rho
}
pub enum Profile { Copy, Invert }
pub enum Step { CalibrationReports, LiveReports, Decision }
pub enum Action { Report { positive: bool }, Intervene, Abstain }
pub enum DecisionAction { Intervene, Abstain }
pub enum LegalAction { Report, Intervene, Abstain }
pub struct ReporterObservation {
    pub rules: Config,
    pub signal: bool,
    pub profile: Profile,
    pub calibration_reports: Option<[bool; 2]>,
    pub calibration_truth: Option<bool>,
}
pub struct DecisionObservation {
    pub rules: Config,
    pub calibration_reports: [bool; 2],
    pub calibration_truth: bool,
    pub live_reports: [bool; 2],
}
pub enum Observation {
    Reporter { view: ReporterObservation },
    Decider { view: DecisionObservation },
}
pub struct Request {
    pub protocol_version: u16,
    pub request_id: u64,
    pub actor: AgentId,
    pub step: Step,
    pub observation: Observation,
    pub legal: Vec<LegalAction>,
}
pub struct Response { pub request_id: u64, pub actor: AgentId, pub action: Action }
pub struct Outcome { pub live_truth: bool, pub action: DecisionAction, pub payoff: i8 }
pub struct Checkpoint { pub request: Option<Request>, pub outcome: Option<Outcome> }
pub struct Archive {
    pub protocol_version: u16,
    pub config: Config,
    pub seed: u64,
    pub responses: Vec<Response>,
    pub checkpoint: Checkpoint,
}
pub enum Error {
    InvalidConfig(Vec<FieldError>),
    InvalidResponse,
    VersionMismatch,
    InvalidArchive { index: Option<usize> },
    InvalidGenome,
    Inference(TestimonyError),
}
```

Version constants are `GAME_VERSION: u16 = 1` and `GAME_PROTOCOL_VERSION: u16 = 1`. Tagged enums use snake_case `kind` and strict decoding. Profile, Step, DecisionAction, LegalAction are ordinary snake_case enums. Action implements the existing custom Deserialize pattern with zero-field WireAction::Intervene{} and Abstain{}, so extra fields cannot slip past serde unit variants. Config contains only public assumptions and permissions, never a hidden world or RNG seed.

Config::standard(accuracy, copy_prior) returns IDs [0,1], decider 2 with the explicit permissions. Config::validate() -> Result<(), Error> checks version, probability bounds, distinct IDs, exactly three unique permission entries with complete reference coverage, reporters' receive_signal/report/observe_verification=true and decide=false, and decider's receive_signal/report=false and observe_verification/decide=true. Array order defines reporter slots and buffering order; renaming IDs preserves those slots. Errors implement Display/Error, with InvalidResponse exposing no rejection detail.

Session::new(config: Config, seed: u64) -> Result<Session, Error>; request(&self) -> Option<Request>; submit(&mut self, response: Response) -> Result<(), Error>; outcome(&self) -> Option<&Outcome>; archive(&self) -> Archive. `replay(archive: &Archive) -> Result<Session, Error>` reconstructs and compares the declared checkpoint; no new checksum scheme. `report(profile: Profile, signal: bool) -> bool` is a pure copy/invert atom.

### Task 2 contracts

```rust
pub struct HistoryMass {
    pub observation: DecisionObservation,
    pub total_mass: u64,
    pub true_mass: u64,
}
pub struct Distribution {
    pub config: Config,
    pub denominator: u64,
    pub histories: Vec<HistoryMass>,
}
pub struct Genome { pub b: i16, pub u: i16, pub d: i16, pub k: i16 }
pub enum Listener {
    Bayesian, Credulous, Skeptical, DirectEvidence, Passive, Evolved(Genome),
}
pub struct ListenerDecision {
    pub action: DecisionAction,
    pub posterior_true: Option<f64>,
}
pub struct HistoryEvaluation {
    pub observation: DecisionObservation,
    pub total_mass: u64,
    pub true_mass: u64,
    pub reference_posterior: f64,
    pub decision: ListenerDecision,
    pub conditional_regret: f64,
}
pub struct Evaluation {
    pub payoff_numerator: i64,
    pub optimal_numerator: i64,
    pub regret_numerator: i64,
    pub denominator: u64,
    pub intervention_mass: u64,
    pub correct_intervention_mass: u64,
    pub incorrect_intervention_mass: u64,
    pub posterior_max_error: Option<f64>,
    pub posterior_squared_error: Option<f64>,
    pub brier: Option<f64>,
    pub histories: Vec<HistoryEvaluation>,
}
```

`enumerate(config: &Config) -> Result<Distribution, Error>` validates input and emits positive-mass histories in ascending Boolean tuple order `(C, calibration_report_0, calibration_report_1, live_report_0, live_report_1)`. `Listener::decide(&self, observation: &DecisionObservation) -> Result<ListenerDecision, Error>` validates public rules; no private-state argument. `evaluate(distribution: &Distribution, listener: &Listener) -> Result<Evaluation, Error>` validates mass consistency before division. `Genome::validate() -> Result<(), Error>` rejects values outside the spec ranges. `fitness(distribution: &Distribution, genome: &Genome) -> Result<i64, Error>` produces the same payoff numerator as evaluate, without assembling large per-history reports. New numerical helpers remain private unless needed by a later task.

### Task 3 contracts

```rust
pub enum SearchMethod { Genetic, Random }
pub struct SearchSettings {
    pub population: u32, pub generations: u32, pub elites: u32,
    pub tournament: u32, pub mutation_numerator: u32,
    pub mutation_denominator: u32, pub evaluations: u32,
}
pub struct ProgressPoint {
    pub evaluations: u32, pub best_fitness_numerator: i64, pub champion: Genome,
}
pub struct SearchRun {
    pub seed: u64, pub derived_seed: u64, pub method: SearchMethod,
    pub seed_derivation: String, pub settings: SearchSettings,
    pub champion: Genome, pub fitness_numerator: i64,
    pub denominator: u64, pub evaluations: u32,
    pub curve: Vec<ProgressPoint>,
}
```

`search_ga(distribution: &Distribution, seed: u64) -> Result<SearchRun, Error>` and `search_random(distribution: &Distribution, seed: u64) -> Result<SearchRun, Error>` use fixed public settings. A private parameterized search helper may support small unit fixtures, but diagnostic configuration is never overridable. `search_seed(seed: u64, method: SearchMethod) -> u64` uses wrapping `seed * 6364136223846793005 + identity * 1442695040888963407`, Genetic identity 1, Random identity 2; label `testimony-search-seed-v1`. These streams never generate game worlds.

### Task 4 contracts

`diagnose_testimony_game() -> Result<GameReport, Error>` produces a Serialize-only report. Required named records:

- GameReport: version `testimony-game-v1`, protocol/game/listener/search versions, tolerance, named environment configurations, `checks: Vec<ReferenceCheck>`, fixed-controller evaluations, `runs: Vec<SearchRun>`, frozen-champion transfer evaluations, per-method/environment summaries, paired search differences, and `passed: bool`.
- ReferenceCheck: quantity, expected_exact string, expected/actual/error f64, passed. Integrity checks use quantity/expected/actual/passed Boolean records separately; exploratory findings never contribute to passed.
- NamedEvaluation: environment String, policy String, optional search seed/genome, Evaluation.
- SearchSummary: method, environment, count, minimum/median/maximum payoff as f64. Median for 20 runs is the average of the middle two.
- PairedDifference: seed, environment, GA-minus-random payoff numerator and common denominator.

The report includes exact mass numerators/denominators alongside readable f64 metrics, all settings/seed derivation, all 40 runs and champions, all 32 public histories when supported, and all four environments. Keep no sampled superiority tests or hidden tuning options.

---

### Task 1: Private signals and transactional game sessions

**Files:** Create testimony_game/mod.rs, types.rs, session.rs, core/tests/testimony_game.rs; modify deduction/mod.rs additively. Produces Task 1 contracts above.

- [x] **Step 1: Write red configuration/reporting tests.** Cover q/rho denominators 0/17, numerator above denominator, valid endpoints, sparse distinct IDs, duplicate/missing/unknown permissions, invalid permissions/version, and strict unknown-field decoding at every nested wire/config type. Check report(Copy, true)=true and report(Invert, true)=false, with false inputs too.

```rust
#[test]
fn testimony_game_rejects_extra_fields_on_abstain() {
    let parsed = serde_json::from_str::<Action>(
        r#"{"kind":"abstain","truth":true}"#);
    assert!(parsed.is_err());
}
```

- [x] **Step 2: Run red, then implement contracts and pure atoms.** Run `cargo test -p sugarscape-core --test testimony_game`. Record missing exports/types and actual subsequent behavioral failures. Create the namespace and strict types; implement descriptive validation and generic response errors without affecting old exports.

- [x] **Step 3: Write buffered-delivery/atomicity/replay tests.** Add a test helper that submits report(profile, signal) using only each ReporterObservation; manually driven legal false reports must also work. After the first calibration report, the second request still has calibration_reports=None and calibration_truth=None. After both, the first live request has both reports and verified C. The second live reporter sees calibration data but neither live report. The deciding request has both live reports but no signal/profile/seed/live truth keys.

```rust
#[test]
fn testimony_game_stale_response_preserves_request_and_archive() {
    let config = Config::standard(
        Probability { numerator: 4, denominator: 5 },
        Probability { numerator: 3, denominator: 4 });
    let mut game = Session::new(config, 7).unwrap();
    let before = game.archive();
    let r = game.request().unwrap();
    let error = game.submit(Response {
        request_id: r.request_id + 1, actor: r.actor,
        action: Action::Report { positive: true },
    });
    assert_eq!(error, Err(Error::InvalidResponse));
    assert_eq!(game.archive(), before);
}
```

Also reject wrong actor, decision during report, report during decision, and post-completion response. Compare complete future request sequences after a rejection, not merely current views. Replay archives after zero/one/two/three/four/five accepted responses; tampered version, illegal response index, request/outcome checkpoint mismatch must fail. Outcome follows the fixed payoff table and completion reveals live truth only then. ID renaming preserves behavior and perspective.

- [x] **Step 4: Run red and implement session.** Generate privileged state once with seeded(seed): draw C then T as gen_bool(0.5), then two copy profiles with integer rational Bernoulli, then signals in order C-speaker0, C-speaker1, T-speaker0, T-speaker1. Rational Bernoulli uses `rng.gen_range(0..denominator) < numerator`, including endpoints. Keep the exact order versioned; never pass RNG/world into observations. Validate the full response before buffering it. After the second calibration response, automatically verify C and advance to live; after the fourth response, advance to decision. Final submit computes outcome. Replay reconstructs from seed, resubmits accepted responses, and compares Checkpoint.

- [x] **Step 5: Verify and independently review.** Targeted tests and workspace fmt check. Retain uncommitted before/after snapshots, real red/green logs and a task report. Fresh reviewer checks permissions, buffering, strict decoding, endpoint draws and complete-future atomicity. No commit.

### Task 2: Exact histories, listener policies and attainable regret

**Files:** Create enumeration.rs and listeners.rs; modify new mod.rs and core/tests/testimony_game.rs. Consumes Task 1 and existing Belief/best_accusation; produces Task 2 contracts.

- [x] **Step 1: Establish independent Fraction references before collection.** Root/worker stores `reference_check.py` and reference JSON in this plan's ignored review workspace. The script enumerates all eight Boolean variables directly with itertools.product, assigning independent truth/profile/signal factors; it never calls Rust or imports production data. Its central mass calculation is:

```python
from fractions import Fraction
from itertools import product
def worlds(q, rho):
    for c, t, p0, p1, c0, c1, t0, t1 in product((False, True), repeat=8):
        mass = Fraction(1, 4)
        for copies in (p0, p1):
            mass *= rho if copies else 1-rho
        for truth, signal in ((c,c0), (c,c1), (t,t0), (t,t1)):
            mass *= q if signal == truth else 1-q
        key = (c, c0 if p0 else not c0, c1 if p1 else not c1,
               t0 if p0 else not t0, t1 if p1 else not t1)
        yield key, t, mass
```

Aggregate exact total/true mass by history for all four fixed environments. Compute conditional posterior, optimal action/utility, and each fixed policy's payoff/regret independently. Credulous assumes accuracy q; Skeptical's effective report accuracy is `rho*q + (1-rho)*(1-q)`. Their live-report likelihoods are products under true/false; ties abstain. Freeze exact reference strings in tests/diagnostics before any GA or first diagnostic run. Reference computation is a development check, not a product dependency.

The planning-time independent enumeration gives these exact expected-payoff targets; reproduce them with the retained script and have the task reviewer cross-check them before collection. DirectEvidence equals Passive here. Regret is the Bayesian column minus the policy's column.

| Environment | Bayesian/attainable optimum | Credulous | Skeptical | Passive |
| --- | --- | --- | --- | --- |
| Training | 57/250 | 3/20 | 3/20 | 0 |
| Inversion prevalent | 57/250 | -3/20 | 3/20 | 0 |
| Less accurate signals | 23/400 | 1/20 | 1/20 | 0 |
| Uninformative signals | 0 | 0 | 0 | 0 |

- [x] **Step 2: Write failing enumeration/accounting tests.** Check denominator `4 * rho_den^2 * q_den^4`, summed mass equals denominator, true mass sums to denominator/2, no zero-mass emitted histories, and each frozen posterior/mass agrees with Fraction references. Exercise q/rho=0/1 and denominators up to 16. Malformed public Distribution (zero denominator, inconsistent history masses, bad rules, duplicate observation keys, overflowing/mismatched total mass) must return InvalidConfig, not panic or silently score nonsense. Use checked accumulation during validation.

```rust
#[test]
fn testimony_game_uninformative_channel_has_no_attainable_gain() {
    let cfg = Config::standard(
        Probability { numerator: 1, denominator: 2 },
        Probability { numerator: 3, denominator: 4 });
    let dist = enumerate(&cfg).unwrap();
    let result = evaluate(&dist, &Listener::Bayesian).unwrap();
    assert_eq!(result.optimal_numerator, 0);
    assert_eq!(result.payoff_numerator, 0);
    assert_eq!(result.intervention_mass, 0);
}
```

- [x] **Step 3: Run red and implement finite reference distribution.** Iterate all 256 Boolean assignments; report emission is the pure copy/invert mapping, while posterior references come only from summed world weights. A world's mass numerator is the product of its two profile numerators and four signal numerators; independent C/T contribute denominator 4. Zero weights can be skipped; merge public histories in canonical order. Use bounded u64 masses, i64 signed payoff. Avoid storing hidden variables in DecisionObservation. Evaluate each distinct history once, rather than recomputing the same action for all underlying worlds.

- [x] **Step 4: Write failing listener/genome tests.** Bayesian must agree with every frozen positive-mass history. Credulous ignores calibration; Skeptical ignores it but uses rho; DirectEvidence maintains 1/2 and Passive has no posterior. Both abstain in this assembly. Construct a history where calibration changes Bayesian's action and verify the action differs from its own no-calibration counterpart. Known inversion reverses evidence. Endpoint histories contradicting an assumed deterministic model return Error::Inference without fallback. Bayesian uses legitimate delivered calibration verification only.

Genome bounds: b/k -16..16, u/d 0..16, division/scaling exactly as spec. Test trust at rho endpoints, matching/mismatching calibration, q=1/2 zero evidence score, strict score-threshold tie, ID renaming, and no posterior for evolved scores. Invalid genome is an error.

```rust
#[test]
fn testimony_game_evolved_score_is_not_a_probability() {
    let dist = enumerate(&training_config()).unwrap();
    let policy = Listener::Evolved(Genome { b: 0, u: 6, d: 6, k: 0 });
    let result = evaluate(&dist, &policy).unwrap();
    assert_eq!(result.posterior_max_error, None);
    assert_eq!(result.brier, None);
}
```

`training_config()` is the typed q=4/5, rho=3/4 Config::standard helper local to tests, not an undefined production API.

- [x] **Step 5: Run red and implement policies/evaluation.** Build Bayesian's 16 hypotheses using two Boolean propositions, two persistent copy/invert assignments, and four groups with q; evidence sequence is calibration report0/report1, Verified C, live report0/report1. Priors use independent 1/2 truths and rho profile assignments. Do not feed enumerator truth masses into this policy. Credulous/Skeptical implement their distinct assumed-channel likelihoods from public data. Decision helper mapping permits only positive intervention or abstention and favors abstention on an actual tie.

For Genome, compute stable sigmoid/logit with exact rho endpoints and the spec's score; no floors. For each history, intervention contributes `2*true_mass-total_mass`, abstention zero; attainable best contributes `max(2*true_mass-total_mass, 0)`. Regret numerator is best minus chosen, so aggregate exact regret cannot be a floating-roundoff negative. Posterior-model metrics use independent `true_mass/total_mass` and mass weights. Brier contribution is `true_mass*(1-p)^2 + (total_mass-true_mass)*p^2`, divided by full denominator. Conditional regret is corresponding best-minus-chosen divided by history mass. Correct/incorrect intervention masses remain separate. Validate probabilities finite/in-range, errors propagate, no random draws.

At q=1/2 the Bayesian model mathematically leaves T at exactly its prior: return posterior 1/2 and abstain after validating/conditioning the delivered evidence, rather than letting numerical summation choose a side. This channel identity does not consult hidden truth or the enumerator. For other histories, normalize Bayesian posteriors within 1e-12 of 1/2 to exactly 1/2 before using the helper. This enforces mathematical tie abstention: the maximum rational history denominator is 67,108,864, so any non-tie posterior differs from 1/2 by at least 1/(2*67,108,864), greater than the tolerance. This is numerical normalization, not a probability floor or a changed decision threshold. Test every exact-mass tie, fixed intervention masses, and a near-threshold admissible non-tie. Revisit this reasoning if channel bounds expand.

- [x] **Step 6: Verify and independently review.** Targeted tests, fmt, actual independent script output, baseline policy exact constants. Fresh reviewer checks inference versus independent enumeration, no truth leakage, mass arithmetic, unsupported histories, expected-loss examples and calibration metrics. Preserve snapshot evidence; no commit.

### Task 3: Frozen evolutionary search and equal-budget control

**Files:** Create evolution.rs and core/tests/testimony_game_search.rs; modify new mod.rs; add core tests (private operator unit tests belong in evolution.rs). Consumes Distribution, Genome, fitness; produces Task 3 contracts.

- [x] **Step 1: Write red operator/ordering tests.** Rank a higher-payoff genome over a lower one regardless of norm; with equal fitness select smaller L1 norm, then lexicographically smaller tuple. Two identical candidates retain deterministic order. Test uniform gene initialization across allowed bounds, crossover genes always from a parent, mutation +/-1/clamp, mutation probability through an injected deterministic RNG (not noisy assertions), and tournament sampling with replacement. Equal-budget random search uses the same candidate ordering.

```rust
#[test]
fn testimony_game_search_seed_streams_are_distinct_and_repeatable() {
    assert_eq!(search_seed(7, SearchMethod::Genetic),
               search_seed(7, SearchMethod::Genetic));
    assert_ne!(search_seed(7, SearchMethod::Genetic),
               search_seed(7, SearchMethod::Random));
}
```

Add seed golden values computed independently from the wrapping formula; include u64::MAX overflow behavior. Operator implementations use existing seeded/SimRng and rand::Rng; do not introduce a testing RNG dependency.

- [x] **Step 2: Run red and implement fixed genetic operators.** Gene initialization draws b then u then d then k from inclusive integer ranges. Each offspring selects its two tournament winners (three index draws per tournament, with replacement), chooses four parent-gene Boolean draws in b/u/d/k order, then considers four mutations in that order. Draw the +/-1 direction only when mutation occurs. Boundary clamping is a possible unchanged mutation. Retain two sorted elites without new fitness evaluations. Use the exact integer numerator comparator; do not use tolerance-based nontransitive ordering or a fitness cache.

- [x] **Step 3: Write failing end-to-end search tests.** With a counting fitness closure in a private small-search fixture, prove repeated genomes still count, elites preserve best fitness, and best-so-far never falls. Public frozen searches must each report exactly 3,164 evaluations and curve points, with evaluation indices 1..=3,164. Curve endpoints agree with champion/rank/fitness, and exact fitness recomputation agrees. Repeat a public seed run byte-for-byte; random search also repeats. Do not assert a particular learned payoff improvement as a correctness condition.

```rust
#[test]
fn testimony_game_search_controls_use_the_same_budget() {
    let dist = enumerate(&training_config()).unwrap();
    let ga = search_ga(&dist, 0).unwrap();
    let random = search_random(&dist, 0).unwrap();
    assert_eq!(ga.evaluations, 3164);
    assert_eq!(ga.evaluations, random.evaluations);
}
```

- [x] **Step 4: Run red and implement loops/records.** Evaluate 64 initial genomes, then 50 generations of 62 offspring. Save one best-so-far point per evaluation, including initial population evaluation costs. Random draws/evaluates 3,164 independent uniform genomes. Both return explicit derived seeds, full fixed settings, training denominator, champion and exact fitness numerator. Search takes only a training Distribution and seed; there is no holdout argument or champion-selection callback.

- [x] **Step 5: Verify and independently review.** Targeted tests and fmt; fresh reviewer checks rank direction, seed isolation, operator draw order, endpoint mutation, counted budgets and no test-imposed learning outcome. Test runs are development probes, not the first published collection; freeze source/settings beforehand and record any scientific revisions after a probe result. No commit.

### Task 4: Diagnostic host, frozen collection and documentation

**Files:** Create diagnostics.rs and CLI/tests/testimony_game.rs; modify new mod.rs, CLI/src/deduction.rs, core/tests/testimony_game.rs, docs/deduction.md. Produces Task 4 contracts.

- [x] **Step 1: Protect the baseline and write red report/CLI tests.** Before edits, root saves current `deduction diagnose`, `deduction testimony`, and seed-7 `deduction run --scenario wink --seed 7 --policy evidence` JSON output in this plan's ignored workspace. Snapshot all baseline tracked/untracked sources and filming guidance. Tests require version, exact check flags, four configs, five fixed controllers per environment, 40 search runs, 160 frozen-champion evaluations, 20 paired differences per environment, summaries and settings. The sixth controller is represented by each GA champion rather than an unspecified single evolved policy. Random-search champions are separate controls.

```rust
#[test]
fn testimony_game_command_rejects_tuning_flags() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["deduction", "testimony-game", "--seed", "7"])
        .output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
```

Test actual failed numeric reference and Boolean integrity checks feed aggregate passed=false. Inject those failures through the comparator, not only by manually clearing a pass flag. CLI output-helper tests print failed checks before Failure::Invalid and propagate deliberately failing Write/flush as Failure::Io. All correctness gates ignore exploratory learning outcomes.

- [x] **Step 2: Run red and implement frozen orchestration.** Define named public configs and exact Fraction reference strings before collection. Build distributions; evaluate five fixed controllers against independent references. Train both methods for seeds 0..20 into a fully owned immutable runs collection. Only after all champions are selected, evaluate frozen genomes on all four environments. Summarize per method/environment, calculate paired differences in a common exact denominator, and retain all runs/curves/history evaluations. Store exact parameter bounds and decoded-genome formula/version in report metadata so four numbers do not masquerade as unrestricted learned reasoning.

Report passed requires finite valid output, mass normalization, exact expected baseline/payoff checks, per-history Bayesian agreement, zero Bayesian regret, budget/fitness/champion-freeze integrity, and uninformative-payoff checks for every evaluated policy. No requirement that GA improves, wins, transfers, or beats random search. Retain intervention rates at q=1/2 and posterior fields only for defined models. Aggregate minima/medians/maxima use actual 20 results, not a single selected run.

- [x] **Step 3: Add CLI command and real-binary test.** Add Mode::TestimonyGame (clap spelling testimony-game), run diagnose_testimony_game(), emit one JSON report using existing emit helper, then return Invalid only for failed correctness checks. No tuning flags, provider calls, logging on stdout, or interactive-host changes.

```rust
fn emit_testimony_game(report: GameReport, out: &mut impl Write) -> Result<(), Failure> {
    let passed = report.passed;
    let value = serde_json::to_value(report)
        .map_err(|_| invalid("invalid testimony-game report"))?;
    emit(&value, out)?;
    if passed { Ok(()) }
    else { Err(invalid("testimony-game correctness checks failed")) }
}
```

Run the binary command twice and compare stdout bytes, parse JSON, require success/required correctness fields and expected seed/budget metadata. Preserve the first frozen diagnostic command report with source/settings snapshot in the ignored plan workspace. Build-only failures do not count as collected results, but record actual behavior without retrospective retuning.

- [x] **Step 4: Document actual findings and compatibility.** Append game, command, observation/archive boundaries, explicit permissions, fixed behaviors versus strategic incentives, exact methods, all seed results/summaries, GA/random budget comparison and transfer findings to docs/deduction.md. Explain hand-supplied policy structure, payoff rather than wins, expected regret rather than guaranteed realized loss, no evolved probability output, public-channel transfer versus unknown-channel misspecification, and no existing play-mode integration. Report negative results plainly. Compare all three prior command outputs byte-for-byte and preserve the old guide prefix and filming edits.

- [x] **Step 5: Full verification and independent whole-change review.** Run and inspect all statuses:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
wasm-pack test --node crates/sugarscape-wasm
```

In web run `npm run wasm`, then `npx tsc --noEmit`, then `npx vitest run`. In worktree root run `python3 -m unittest discover -s studio/tests -t studio`. Check `git diff --check` and whitespace on new untracked files. Retain logs and test counts. A fresh most-capable final reviewer examines the scoped uncommitted snapshot, spec/plan, independent references, first collection, preservation checks and all task review rulings. Route fixes to the owning implementer, then review them. Mark four stages complete and remove IMPLEMENTATION_PLAN.md only when required work passes. Leave all changes uncommitted; no staging, merge, or push.

## Execution records and self-review

Use `.superpowers/sdd/2026-10-04-testimony-game/` as the ignored per-plan evidence directory. Root creates it at execution, snapshots the complete uncommitted baseline, and maintains progress, task briefs, diffs, reviews, actual red/green output and rulings. Do not reuse another plan's scratch directory or reconstruct old reports from a changed tree. Root creates the four-stage IMPLEMENTATION_PLAN.md immediately before dispatch, not at this planning review checkpoint.

Spec coverage: permission/configuration/signal/reporting/protocol/replay in Task 1; finite references/listeners/genome/payoff/calibration in Task 2; all frozen search settings/budgets/operators in Task 3; diagnostic schemas/first collection/holdouts/findings/preservation/full checks in Task 4. All five Review Focus conditions have owning tests. New names are contained in one namespace; downstream contracts refer to explicitly declared types. Search comparison uses exact integer fitness and the same count; Bayesian references never depend on Belief's output. The preexisting game/diagnostics and filming guidance are protected by snapshots and byte comparisons.

The approved architectural spec limits capabilities to this explicit three-Agent assembly. This plan does not imply arbitrary role/rule configuration, strategic speaker learning, general-purpose trust, or a human/LLM CLI for the new game. Those remain separate designs.
