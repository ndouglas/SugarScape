# A testimony game with an evolved decision policy

## Intent and scope

Connect the verified testimony inference layer to consequential decisions in a small, completely enumerable game. The user wants capability-defined mechanics, independently testable atoms, evidence that poor play loses in expectation, and eventual human/LLM participation without assuming those players are necessary for useful social deduction. Include an evolutionary controller to investigate whether selection on payoff can discover useful trust and intervention heuristics.

This architectural design is approved. Written implementation-plan review precedes implementation and collection. Preserve the existing uncommitted capability and testimony work; commit only on explicit request.

The increment consists of one calibration exchange, one unresolved exchange, and one decision. It measures learned heuristics against exact inference. Speakers use fixed persistent reporting profiles. Their reports are behavioral evidence, not direct facts. Adaptive speakers, strategic equilibrium, open conversation, unknown provenance, and integration into the existing Wink engine are later increments.

## Approaches considered

1. **A separate small testimony game, recommended:** exposes signal generation, reporting, delivery, inference, and scoring as distinct atoms. Complete enumeration gives independent expected-payoff and posterior references.
2. **Extend Wink immediately:** provides survival dynamics, but its present claims and observations cannot represent private Boolean signals or independent verification. Simultaneously changing those interfaces and introducing inference would make errors harder to locate.
3. **Coevolve speakers and listeners immediately:** addresses incentives more directly, but moving opponents obscure whether a learned listener is sound even against fixed behavior.

Use the first approach. Keep the new game beside the current deduction engine. It shares Agent IDs, the pure testimony model, and the expected-utility decision helper, but does not reinterpret existing capability effects, claims, objective teams, or archives. The game defines participation through permissions to receive signals, report, observe verification, and decide; no Mafia/Werewolf roles or thematic labels enter the logic. These permissions are explicit assembly data for this bounded game, not an arbitrary capability-language extension.

## Mechanics and information boundary

Three Agents participate: two reporting Agents and one deciding Agent. Agent IDs are explicit and distinct. Each reporting Agent has permission to receive one private Boolean signal per proposition and submit one Boolean report. The deciding Agent receives public reports and calibration verification, then may intervene or abstain. The fixed schedule is calibration reports, calibration verification, live reports, decision, completion.

There are two independent Boolean propositions, calibration C and live T, each with prior probability 1/2. Each speaker independently receives a persistent profile: copy with probability rho, invert otherwise. For each speaker and proposition, draw an independent private signal that equals the proposition with probability q. Profiles are independent of truths and signals. Copy emits the signal; invert emits its complement. This describes behavior, not inferred motivation. A copying speaker can be mistaken; an inverting speaker can report truth.

Reports for an exchange are buffered and published together. The second reporting Agent cannot see the first report before responding. Both reporters then see the calibration truth before the live exchange. The deciding Agent receives both calibration reports, the independently verified calibration truth, and both live reports. It never receives either private signal, actual profiles, live truth, seed, host archive, or full-state checksum. No observation of the live truth is possible before the decision.

Public information includes the game version, participants and permissions, proposition/group identifiers, q, rho, payoff rules, and legal actions. Signal-group IDs disclose the actual assembly's independent channels; they do not disclose signal values. Both live signals are independent conditional on T and the profiles. Persistent profiles couple the calibration and live exchanges.

A reporting request additionally exposes only that speaker's own current signal and assigned reporting profile. The fixed reporting controller uses those fields; an external controller can choose another report. It never receives the other speaker's signal/profile or an unverified proposition truth. Represent q/rho as numerator/denominator pairs with denominator 1..16 and numerator 0..denominator, rejecting malformed values. This bounds exact integer enumeration while retaining deterministic endpoints. The assembly has exactly two distinct reporters and one distinct decider, each with only its declared permissions; unsupported schedules/permission combinations are errors rather than partially implemented general rules.

Intervene scores +1 when T is true and -1 when T is false. Abstain scores 0. Completion reveals T and the realized decision payoff to participants, after the irrevocable decision. It does not reveal profiles or signals. This is a decision game, with no separate win threshold: payoff and regret are more informative than calling abstention a victory. Speakers have no optimized objective in this increment; assigning adversarial intent to fixed inversion would overstate what is implemented.

## Components and interface

Keep responsibilities in focused modules under `crates/sugarscape-core/src/deduction/`, with names finalized in the implementation plan:

- Game assembly/configuration validates permissions, participant IDs, rational q/rho, and the fixed schedule.
- Private world generation and pure reporting atoms describe truths, profiles, signals, and emissions. Privileged state stays host-side.
- A small transactional session delivers actor-specific requests, buffers reports, verifies calibration, and resolves the decision.
- Listener controllers consume only the deciding Agent's observation, build their own beliefs, and select a legal action.
- Finite enumeration independently computes history masses and conditional references; evaluation joins controller actions with host truth only for scoring.
- Evolution manages genomes and evaluates them through the same observation/action interface.

Use separate versioned game request/response types rather than changing the existing `TurnRequest` and `TurnResponse`. A request contains protocol version, monotonically increasing request ID, actor, step, actor-specific observation, and legal actions. Responses are report-positive/report-negative or intervene/abstain as appropriate. Calibration verification is a host event, not a player action. All public input structs and structured variants reject unknown fields, including fieldless actions through the existing strict-decoding pattern.

Wrong actor, stale request, malformed response, and illegal action reject atomically without advancing state or RNG. Protocol rejection is generic. Configuration errors remain descriptive. Completed sessions have no outstanding request. Accepted responses plus config, seed, and version form a privileged replay archive; deterministic replay must reconstruct the same requests and outcome. Do not add a new checksum algorithm or claim archive authentication.

The externally controllable session interface leaves room for human/LLM players, but this increment adds no network provider, API dependency, or interactive CLI session mode. The existing Wink `play` continues unchanged. An external reporting Agent may deviate from a fixed profile, so Bayesian optimality claims apply only to the declared reporting model; deviation tests verify protocol behavior without asserting optimality.

Use existing serde, RNG, errors, and test patterns. No new dependencies, global model/preset registry entries, browser UI, or video assets.

## Six listener controllers

Every listener receives the same public information and history. Controller construction must not accept a private world, host seed, or replay archive. New policy types remain separate from existing `PolicyKind`.

1. **Bayesian:** construct the existing `TestimonyModel` with 16 hypotheses for C, T, and the two persistent profiles; four independent signal groups. Assimilate calibration reports, legitimate calibration verification, and live reports. Intervene iff P(T) > 1/2; ties abstain. Use the existing decision helper with the same permitted-action mapping as testimony-v1.
2. **Credulous:** assume both speakers copy and use the public signal accuracy q. Ignore calibration-derived credibility. For q > 1/2, this means intervene only when both live reports are positive; disagreement ties abstain. Implement from its assumed channel, so q = 1/2 and endpoints behave correctly.
3. **Skeptical:** use public rho as a fixed per-speaker copy probability; ignore calibration history. Compute its posterior for live reports under that fixed model, and use the same decision rule. This explicitly tests the cost of failing to update trust rather than an unexplained discount constant.
4. **Direct evidence only:** verified C provides no evidence about independent T. Keep P(T) = 1/2 and abstain. This is intentionally behaviorally identical to Passive in this assembly; report that redundancy rather than inventing an advantage.
5. **Passive:** always abstain, without maintaining a belief.
6. **Evolved:** use the small inspectable genome below, trained only on expected decision payoff.

All controllers return only legal actions. Only controllers with an explicitly defined probability model receive posterior/calibration metrics; an evolved score is not called a posterior.

## Inspectable evolutionary policy

Use four integer genes, decoded to real-valued parameters:

| Gene | Integer range | Decoding | Meaning |
| --- | --- | --- | --- |
| b | -16..16 | b/4 | Offset to initial copy-profile log odds |
| u | 0..16 | u/4 | Trust increment when calibration report matches verified C |
| d | 0..16 | d/4 | Trust decrement when calibration report differs from verified C |
| k | -16..16 | k/8 | Intervention threshold |

For speaker s, define `c_s = sigmoid(logit(rho) + b/4 + adjustment_s)`, where adjustment is u/4 for a calibration match and -d/4 for a mismatch. At rho = 0 or 1, c_s is exactly that endpoint; do not approximate with probability floors. Then:

`score = sum_s (2*q - 1) * (2*c_s - 1) * (2*live_report_s - 1)`.

Intervene iff score > k/8, otherwise abstain. The same parameters apply to both speakers. Speaker renaming cannot change an action. The game presents q and rho to every controller; the evolved policy uses them explicitly rather than receiving an undisclosed privileged environmental label.

This representation already supplies the shape of a trust heuristic and evidence aggregation. Evolution discovers weights and a threshold, not arbitrary reasoning or a posterior. Calibration verification contributes a feature only after actual delivery. The policy must never use live truth, actual profiles, private signals, or evaluator results as runtime features. Document these inductive choices when interpreting success.

## Frozen training and controls

Training environment: q = 4/5, rho = 3/4. Independently enumerate all 256 combinations of C, T, two profiles, and four signals, assigning exact rational probability masses. Fitness is expected realized payoff over the complete distribution. There are no sampled game seeds and no stochastic fitness estimates.

For the fixed rational configurations, compute mass and payoff numerators with bounded integer arithmetic and a common denominator. A history's independent reference posterior is its true-T mass divided by total history mass. This reference is calculated from world enumeration, not from `Belief`. Obtain independent Python `Fraction` references for histories and aggregate baseline metrics before first collection. Production uses existing numeric types, not a new rational library.

Run 20 independent GA training seeds, 0..20. Fixed settings:

- Population 64; uniform independent initialization over each integer gene's range.
- Evaluate the initial population and 50 replacement generations.
- Retain the best two unchanged. Generate 62 offspring per generation.
- Parent selection: tournament of three sampled population members with replacement, using the ordering below.
- Uniform crossover: independently select each gene from either parent with probability 1/2.
- Mutation: independently for each gene, probability 1/4 of adding +1 or -1 with equal probability; clamp to the valid range. Clamping may leave a boundary gene unchanged.
- Order by higher exact fitness, then lower integer L1 norm `abs(b)+u+d+abs(k)`, then lexicographically smaller gene tuple. Use deterministic handling of fully equal entries.
- Evaluate every candidate, including repeated genomes; no cache changes the declared budget. Each run evaluates 64 + 50*62 = 3,164 candidates.

The integer L1 tie-break only selects between identical payoffs; it does not penalize useful but larger weights. Describe it as a parameter-size preference, not proof of simpler behavior. Use exact payoff numerators for training comparisons, not approximate-tolerance sorting.

For each training seed, independently evaluate 3,164 uniformly sampled genomes as a random-search control, with the same best-candidate ordering. Use separate reproducible RNG streams for GA and random search and record the derivation/version in reports before collection. Training seeds control search only; world enumeration is identical for every candidate. Report best-so-far curves by evaluation count, including the initial population's evaluation cost. Do not claim a GA advantage merely because its selected policy beats hand-written baselines.

Select the GA and random-search champion separately within each seed using training fitness only. Freeze all 40 champions before holdout evaluation. Never select a champion, change a threshold, stop early, or tune representation/settings using holdout outcomes.

## Frozen evaluation and claims

Evaluate the six named controllers and each frozen champion on:

| Environment | q | rho | Purpose |
| --- | --- | --- | --- |
| Training distribution | 4/5 | 3/4 | Learn and measure an attainable decision rule |
| Inversion prevalent | 4/5 | 1/4 | Test transfer when most profiles invert |
| Less accurate signals | 3/5 | 3/4 | Test transfer with weaker private information |
| Uninformative signals | 1/2 | 3/4 | Verify no controller obtains positive expected payoff from information-free reports |

These holdouts are different declared environments, not undisclosed distribution shifts: public q/rho update, but learned genes remain fixed. A later misspecification experiment can distinguish public assumptions from actual generating channels.

For each environment report exact history masses; Bayesian posterior/reference maximum error; expected payoff; optimal attainable payoff under that same public history; expected regret; intervention probability; and correct/incorrect intervention mass. Include per-history decisions and conditional regret so aggregate success cannot hide poor decisions. Probability-model controllers additionally report mass-weighted squared posterior error against the exact conditional posterior and Brier score against realized T. Passive and the evolved score do not acquire artificial probability outputs.

Aggregate metrics are exact over game outcomes, but training varies across seeds. Report all 20 training results, selected genomes, minimum/median/maximum training and holdout payoff, paired GA-minus-random-search differences, and evaluation budgets. Preserve raw output. No universal superiority or statistical-significance claim follows from a descriptive 20-seed comparison.

Predeclare questions and checks:

- Bayesian posterior and payoff agree with independent enumeration within 1e-12; conditional regret is zero within that tolerance.
- Credulous has strictly positive expected regret when inversion is prevalent. Passive has positive regret whenever the information supports useful intervention. Establish exact expected values independently before collection.
- Does GA find a positive-payoff rule in the training environment, and how close does it get to the optimum? This is an empirical outcome, not a required pass condition.
- Does GA outperform equal-budget random search, and do frozen champions transfer? Report negative/equal results as such; neither is a completion requirement.
- All controllers have expected payoff zero with q = 1/2. An always-intervening policy may also achieve zero there; zero regret does not imply sensible tie behavior. Report intervention rates separately.

Rules, search settings, seed range, exact references, and holdouts are fixed before first collection. No result-dependent changes disguised as original rules. If a reference or implementation bug is found, fix and independently recheck it; if the scientific rule changes after a result is known, explicitly label the new version and revised rule in claim text.

## Diagnostics and records

Add a separate `sugarscape deduction testimony-game` diagnostic command, taking no experimental tuning flags in this increment. It evaluates exact fixtures and fixed controllers, runs GA and random search, then emits one versioned JSON report. Record model, policy, protocol, and search versions; permissions; complete channel/payoff definitions; search seeds/derivation; genomes; exact references; observed metrics; and validation pass flags. Do not silently change the existing `deduction testimony` or `deduction diagnose` schema.

Exit 0 when required correctness checks pass, even if GA fails to learn or loses to random search. Failed exact/integrity checks print the report then exit 2; argument/configuration errors exit 2; output failures exit 1. Distinguish correctness checks from exploratory outcomes in the schema. No progress or logging text contaminates JSON stdout.

Extend `docs/deduction.md` with the game, permission/interface boundaries, frozen methods, first measured results, and limits. Retain first reports and independent enumeration evidence in a new ignored per-plan `.superpowers/sdd/` directory. Do not stage those files. Existing results and filmmaking guidance remain intact.

## Validation and delivery

Implement test-first in three or four independently reviewed stages, with subagent-driven development. The implementation plan will name files and exact APIs and record deterministic RNG derivation. Update root `IMPLEMENTATION_PLAN.md` during execution and remove it when complete. No commit without the user's explicit request.

Tests must cover private signal/reporting endpoints; permission enforcement; simultaneous report delivery; actor-specific observation redaction; verification timing; invalid-response atomicity; replay of partial and completed sessions; strict unknown-field decoding; independent enumeration normalization and conditional references; ID renaming; all listener action rules; payoff/regret accounting; and genome decoding, boundaries, selection, crossover, mutation, reproducibility, budget, champion freeze, and random-search parity.

Require exact independently established baseline values and per-history Bayesian checks. Evolutionary success is a measured finding, not a brittle test assertion. Include deliberate poor policies demonstrating expected loss where supported; do not require a poor policy to lose every realized game.

Completion requires an independent final reviewer, a real first frozen diagnostic run, preserved prior diagnostic output, and full standing checks: workspace formatting, Clippy, Rust tests, WASM Node tests, web WASM/TypeScript/Vitest, and studio unittests. No survey changes are expected.

## Later increments

Evolve bounded reporting policies with explicit speaker utilities only after the listener benchmark is sound. Then consider withholding/fabrication, opponent archives and fixed evaluation panels, larger rule representations, recurrent memory, uncertain channel assumptions, and interactive human/LLM hosting. Each is separate design work. Success here establishes learned decision heuristics under a declared finite channel; it does not establish strategic deception, general social intelligence, or compatibility with AIWolf.
