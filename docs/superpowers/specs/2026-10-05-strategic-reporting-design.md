# Strategic reporting against frozen listeners

## Intent and status

The user approved exploring this increment on October 5, 2026. The written design was approved on October 5, 2026; implementation-plan review precedes implementation and experimental collection. Commit only on explicit request.

Study whether an Agent with an explicit incentive to mislead can choose reports that exploit a listener's interpretation of calibration. Preserve capability-defined mechanics, independently testable atoms, exact checks, and eventual external-player compatibility. The question is whether trust earned on a verified proposition can be exploited on an unresolved one. That outcome is a hypothesis, not a requirement.

The previous testimony-game experiment established listener benchmarks under fixed reporting profiles. GA and equal-budget random search both achieved the training optimum; transfer remained imperfect. This increment changes reporter behavior deliberately. Existing reports and their interpretation remain unchanged.

## Approach and boundary

Use one strategic reporter, one reporter with a fixed persistent copy/invert profile, and one deciding Agent. Freeze listeners before reporter search. Alternatives are simultaneous coevolution, which makes improvements harder to attribute, and a repeated reputation game, which introduces memory and verification choices before this bounded case is understood. Defer both.

Implement a separate assembly beside `deduction::testimony_game`, reusing its pure listener and decision atoms through explicit adapters. Do not change its Config semantics, profile observations, archives, diagnostics, or frozen outputs. A copy prior used by a legacy listener is an assumption of that listener, not the true generative description of a strategic reporter.

Permission to receive a signal, report, observe verification, and decide remains independent of utility. No thematic roles, global preset, new dependency, browser UI, or video is needed. No interactive CLI or LLM provider is included.

## Game and information

Retain calibration C and live proposition T, independent with prior 1/2. Each reporter receives one independent private signal per proposition, accurate with probability q conditional on truth. The fixed reporter receives a persistent copy profile with probability rho and invert otherwise. The strategic reporter has no assigned copy/invert profile: its chosen controller determines both reports.

The schedule remains buffered calibration reports, independent verification of C, buffered live reports, irreversible intervene/abstain decision, completion. A reporter cannot see the other reporter's current buffered report. After verification both reporters see the public calibration reports and C. T is revealed only after the decision. Delivery and verification are host events rather than discretionary speaker actions.

A strategic controller may use its own calibration signal, its own calibration report, verified C, and its own live signal. It intentionally ignores the other reporter's earlier calibration report in this first policy family. That is a representation restriction, not unavailable public information. It cannot inspect the fixed reporter's private profile/signals, T, seeds, archives, or evaluator state.

The deciding Agent sees calibration reports, C, live reports, q, and the declared assembly and utility rules. Legacy listener adapters additionally carry explicitly recorded model assumptions. They deliberately continue to interpret both reporters using their old channel model. Labels and documentation must call their outputs assumed-model beliefs and distinguish them from correct inference under the candidate reporting policy.

Public rules disclose the existence of an incentivized reporter and its objective. The baseline listeners' failure to use this disclosure is a declared limitation, not secret information withheld by the evaluator. The policy itself is withheld from these listeners; the policy-aware reference described below knows it by construction.

## Consequences and utility

The decision payoff remains +1 for intervention when T is true, -1 when T is false, and zero for abstention. The strategic reporter's primary utility is the negative of this payoff. Thus its objective is explicitly opposed to the decider's; a Boolean lie is not itself rewarded. The fixed reporter has no optimized utility.

Expose scoring as a pure validated table indexed by T and the decision, separate from signal/report permissions. Support aligned and opposed utility tables for exact fixtures and controls, but search only the opposed table in this collection. Arbitrary role objectives and general utility expressions are outside scope.

Report decision payoff, reporter utility, intervention rate, false-intervention mass, missed-beneficial-intervention mass, and report/truth agreement by phase. Calibration accuracy alone does not demonstrate manufactured trust. Distinguish a report that opposes a private signal from a false report about actual truth: noisy signals make these different events.

## Finite reporting policies

Use an explicit 18-bit truth table, not a score presented as reasoning:

- Two calibration bits map the private calibration signal to a report.
- Sixteen live bits map `(calibration signal, own calibration report, verified C, live signal)` to a report.

Define canonical Boolean indexing and serialization in the implementation plan. Validate the range, reject unknown fields, and use deterministic canonical tie breaking. Some live rows are unreachable for a particular calibration table; retain them in the representation and disclose this redundancy. Never interpret a favored encoding as evidence of psychological simplicity.

Named controls are signal copying, signal inversion, always positive, always negative, and copy during calibration then invert during live reporting. The last is an explicit control for the hypothesis, not an assumed successful strategy. Every controller consumes the same permitted observation type.

## Exact evaluator and reference checks

Enumerate 128 private worlds: two truths, one persistent fixed-reporter profile, and four private signals. Use bounded rational q/rho as in the previous assembly, with exact mass and payoff accumulation. Evaluate listeners through observations; the evaluator joins their decisions with privileged truth only for scoring.

Precompute each frozen listener's action for all supported public histories. The search evaluator must not mutate listener policies or use an approximation to their decisions. Treat numerical posterior ties with the existing verified convention; independently check supported-history actions.

The policy family contains 262,144 encodings. Derive an exact best response by enumerating the four calibration tables and separately optimizing each reachable live row against the fixed listener panel. With calibration fixed, expected utility is additive across those rows. Verify this decomposition against an independent full truth-table enumeration for at least the training panel, using a separately implemented Python rational reference. Arithmetic, indexing, tie handling, and unreachable rows must be independently checked before collection.

Compute a policy-aware Bayesian reference from the actual candidate-induced history masses. It knows the reporting policy and true generative rules, but receives no private realized truth or signal. Its decision regret is zero by construction and must be checked independently. It is an informed benchmark, not a claim that an uninformed player could recover the policy from one calibration exchange.

For each target panel, report reporter regret relative to its own exact best response. Separately report receiver regret against the policy-aware decision reference. These are different comparisons. Poor reporting means lower expected utility under its specified opponent and objective; require strict regret only in fixtures where independently established values support it, never a loss in every realized game.

## Frozen experiment

Training uses q=4/5, fixed-reporter rho=3/4. The equally weighted training panel contains the previous profile-model Bayesian listener and Credulous listener. The former assumes both reporters independently copy with prior 3/4; this assumption is intentionally misspecified for the strategic reporter. Credulous assumes both copy. These names identify existing algorithms, not guarantees of correct inference in the new game.

Search 20 seeds, 0 through 19, with population 64, two elites, 50 replacement generations, tournaments of three, independent half-probability per-bit crossover, and mutation probability 1/18 per bit. Initialize uniformly over 18-bit encodings. Rank by higher exact training utility, then smaller unsigned encoding, then stable order. This arbitrary encoding tie break is not a behavioral simplicity preference.

Evaluate 64 + 50*62 = 3,164 candidates per seed, counting repetitions. Equal-budget random search uses the identical uniform representation and ranking. The exact optimum is a reference and is not counted as either search method's budget. Specify disjoint RNG stream derivation and record all constants in the implementation plan before collection.

Freeze all champions on training alone. Evaluate every champion against the same training panel at q=3/5, and against withheld Skeptical and Evolved listeners at both accuracies. Use fixed-reporter rho=3/4 throughout. Evolved uses the already published genome (-3,0,2,0); its prior remains 3/4. Passive is a diagnostic control. Holdout panels are disclosed here but never queried for selection, tuning, or champion tie breaking. They establish transfer to these specific withheld policies, not an unseen-policy population.

Retain first raw results, source/settings snapshots, all per-seed champions, exact control values, regret, and paired GA/random comparisons. No search superiority or successful reputation exploitation is a correctness gate. If settings change after seeing results, retain the first collection and mark the new rule as retrospective.

## Delivery and validation

Provide pure utility, reporting, observation, exact evaluation, best-response, and search atoms, plus a separate transactional session and privileged replay archive for this assembly. Actor-specific requests must reject wrong actor, stale request, illegal action, unsupported version, and unknown fields atomically. Verification timing and simultaneous buffering must remain explicit. Archives are replay records, not authenticated evidence.

Add a separate fixed diagnostic command, proposed as `deduction strategic-reporting`, rather than changing testimony-game output. Its exit code reflects correctness/integrity checks, not learning success. No tuning flags or external-player CLI are introduced in this increment.

Implementation should use subagent-driven development and test-first stages: mechanics/permissions; exact enumeration and independent references; frozen-policy search; session/CLI/reporting and preservation. The written implementation plan will settle exact module/API boundaries, supported configuration limits, arithmetic bounds, reference values, and collection procedure before execution.

Tests cover utility independence from permissions; signal/profile endpoints; all truth-table rows; observation redaction; buffered delivery; legitimate verification; immutable listener panels; invalid-response atomicity; strict decoding and partial replay; mass normalization; per-history posteriors; reference regret; best-response equivalence; opponent-weight normalization; RNG and search-budget reproducibility; champion freezing; and byte-identical prior diagnostics.

Require independent final review and the standing Rust, WASM, web, and studio checks. Rebuild the native release CLI before web parity tests. Update and remove the temporary implementation plan during execution. No commit without explicit user instruction.

## What this establishes

This can establish strategic reporting and exact best responses within a declared finite game. It cannot establish general social deduction, equilibrium, long-term reputation, arbitrary roles, inferred unknown motives, or AIWolf compatibility. Repeated interactions, uncertain policy priors, richer action capabilities, opponent archives, coevolution, and human/LLM hosting remain separate increments.
