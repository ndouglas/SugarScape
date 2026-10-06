# Exact adversarial audit of reporting listeners

## Intent and status

Written design approved by the user on October 6, 2026: an exact adversarial audit and a fixed-reporter-only baseline, followed by a decision about bounded model uncertainty or active verification. Implementation-plan review precedes implementation and collection. Commit only on explicit request.

Measure the loss an opposed reporter can induce by choosing a policy specifically against each frozen listener. The completed strategy-inference experiment established correct Bayesian decisions under declared priors and fixed-policy transfer; it did not compute targeted best responses against those listeners. Preserve that experiment and its interpretation.

This increment belongs to the deduction branch of the broader social-knowledge and oversight program. It establishes a bounded attack/defense benchmark. Active inspection, repeated learning, richer multi-Agent games, robust prior sets and institutional-corruption models remain separate increments.

## Scope and frozen game

Keep the strategic-reporting assembly unchanged: independent uniform calibration truth C and live truth T, one strategic reporter, one persistent fixed copy/invert reporter, independent private signals conditional on truth, buffered reports, genuine verification of C, and an irreversible intervene/abstain decision. Receiver payoff is +1 for true-T intervention, -1 for false-T intervention, and zero for abstention; reporter utility is its negative.

Use the same two environments: q=4/5 and q=3/5, fixed-copy prior rho=3/4. Both q and rho are known public rules. The fixed reporter's channel and the genuine calibration verification are trusted components of this model. They are not optimized, colluding, or subject to attacker intervention. A robustness guarantee here is conditional on those assumptions and is an expected-payoff guarantee, not protection in every realized game.

The attacker knows the target listener's public decision rule, declared prior where applicable, rules and utility. It selects one policy before the game's truths, profile and noise are realized, then retains that policy through both phases. This is an offline best response to a known controller, not online learning or policy selection using realized private world state.

During play, the policy uses only the existing permitted strategic view: own calibration signal/report, verified C and own live signal. The existing truth-table family ignores the other reporter's earlier calibration report, even though that report is public; this remains a representation restriction. It never receives T, the fixed reporter's private signals/profile, host seed/archive or current-game evaluator scores. The optimizer may enumerate hypothetical worlds from public rules to calculate expected utility; it cannot provide realized hidden information to a controller.

## Reporting-policy family and identities

Audit the complete existing 18-bit family: two calibration bits and sixteen live bits indexed by `(calibration signal, own calibration report, verified C, live signal)`. There are 262,144 raw encodings.

For each of four calibration tables, exactly eight live rows are structurally reachable. Clearing the other eight rows gives 4*2^8=1,024 canonical behaviors, with 256 raw aliases per behavior. Use the completed strategy-inference canonicalization atom; do not condition deduplication on q, rho or observed histories. Exhaustive canonical evaluation is equivalent to exhaustive raw evaluation for utility, but both representation counts must remain disclosed.

Resolve equal reporter utility by the smallest unsigned canonical encoding, including false live bits on zero deltas and zero unreachable bits. This is a deterministic representation convention, not a preference for psychological simplicity. Retain attack witnesses, canonical identities and any alias checks. Negative, zero and tied attack payoffs are valid outcomes.

## Four frozen listeners

Freeze each listener before optimizing any reporter:

1. **Strategy Uniform:** existing five-policy catalog with weights [1,1,1,1,1].
2. **Strategy Optimization-informed:** existing named-order weights [1,1,1,1,16], motivated by the earlier legacy training panel. No new prior tuning.
3. **Fixed-only:** exact inference from `(verified C, fixed calibration report, fixed live report)` under the known fixed-profile prior and signal channel. It discards both strategic reports. It must not receive the actual strategic policy.
4. **Passive:** always abstain.

The two strategy-aware listeners are reused unchanged. The new fixed-only atom marginalizes the fixed profile and its two signals from public generative rules; independent full-world enumeration verifies the same marginal. Its eight fixed-report histories determine actions for all 32 complete public histories. Compare true/false-T masses directly, with exact ties abstaining. Passive has no inferred probability.

Materialize an immutable, validated action table over the 32 complete public histories by querying each controller through public observations. Record its controller identity, Config and complete table before attack construction. The default frozen environments support all required queries. Unsupported controller evidence is an explicit error, never an invented abstention or zero-payoff result. Reordered histories, mismatched rules, malformed tables and invalid versions must fail validation before scoring. Serialized tables are auditable snapshots, not trusted mutable evaluator caches.

## Exact scoring and best response

Create a sibling `deduction::adversarial_audit` namespace. Reuse existing public Config, observations, policy controls, canonicalization, enumeration and history masses. Do not change FrozenListener's meaning or alter the old strategic-reporting/strategy-inference algorithms, diagnostic schemas or outputs. No new dependency is needed.

Score a candidate against its target's frozen action table by summing actual history masses and true-T masses from `strategic_reporting::histories`. The evaluator joins the action with privileged truth mass only for scoring. The controller receives only the public history; actual-policy posterior and informed reference remain evaluation quantities.

Derive the best response through disjoint live-row contributions. For each calibration table, use the policy with all live bits false as a base. For each reachable live row, score the policy that flips only that row to true; subtract the base utility to obtain its delta. Those row effects apply to disjoint private-world subsets. Choose every positive delta, use false on equality, and compare the four resulting calibration candidates. This can use 36 public history-mass queries per target/environment and requires no access to private World objects or new search algorithm.

Independently verify this decomposition against all 1,024 canonical behaviors for every target/environment, and verify raw/canonical equivalence over all 262,144 encodings. The independent Python Fraction implementation must derive reporting and world probabilities directly, rather than invoking the Rust scorer or importing its expected results.

Use checked integer arithmetic and the existing probability bounds. Actual single-policy distribution denominators are at most 4,194,304. Base utility is within plus/minus that denominator; a row delta is within twice that bound. Nominal mixture weighting and aggregate comparisons require their own explicit checked bounds in the implementation plan. Do not approximate decision thresholds or report scores with floating-point arithmetic.

## Fixed-only minimax bound

Let F be the expected receiver payoff of the fixed-only listener. Its action ignores the strategic reports, so its payoff is F against every policy in this family. An allowed constant reporter supplies no information about T beyond the fixed evidence. Even a receiver that knows that constant policy cannot exceed the fixed-only Bayesian optimum against it. Thus:

`max_receiver min_reporting_policy expected_receiver_payoff = F`.

This bound applies to randomized as well as deterministic receiver decisions: randomization cannot outperform the conditional best action against the constant-report witness. Establish the upper-bound witness and the fixed-only lower bound independently; no exhaustive search over all receiver action tables or new minimax solver is required.

Pre-existing reference predictions are F=9/50 at q=4/5 and F=1/20 at q=3/5, from the completed experiment's informed constant-policy benchmarks. These are previously observed values, not new audit measurements. Independently derive them before new collection. The fixed-only attack witness may be a constant policy selected solely by the canonical tie rule; its constant utility does not imply the reporter learned a special exploit.

The audit measures how the two supplied-prior listeners compare with this limit. A fully conservative optimum can discard strategic testimony in this game. This does not show that testimony is generally useless, that all reporters are hostile, or that active information gathering cannot improve a different game.

## Comparisons and metrics

There are eight targeted audits: four listeners at two accuracies. For each, retain the complete frozen action table, exact best-response policy, worst-case receiver payoff V, opposed reporter utility, informed actual-policy reference payoff, decision regret against that reference, intervention/false-intervention/missed-beneficial-intervention masses, and calibration/live report agreement with truth and private signals.

Report `F - V` separately as the shortfall from the game's attainable worst-case guarantee. It is not the same as regret against a reference that knows the actual reporting policy. The upper-bound argument implies this shortfall is nonnegative for valid complete controllers. Passive supplies the zero-payoff control; the fixed-only listener attains F. Neither a negative payoff nor a new attack outperforming previous controls is a required outcome for the two supplied-prior listeners.

Retain all 1,024 canonical fitness entries per listener/environment, including ties, so the claimed maximum and chosen witness are inspectable. Preserve the five named controls, withheld encoding 98342 and previous forty champion identities as reference/provenance, deduplicating behavior without treating clones as independent challenges. Score these controls against the same frozen tables.

Evaluate each of the four listeners against both existing nominal mixtures at both accuracies: sixteen nominal rows. These are common-distribution comparisons, including the other listener's prior distribution; do not pool their populations or label a different mixture's payoff as superiority on the same opponent distribution. Report the nominal performance surrendered or gained relative to fixed-only alongside the worst-case shortfall. This is a measured tradeoff, not a new robust prior or a claim of optimal performance under every population.

Cross-evaluate each targeted attack witness against all four listeners at its original accuracy: thirty-two transfer rows before any behavioral deduplication. Retain target identity and witness provenance even when witnesses coincide. This transfer is descriptive; witnesses are selected by exact utility against their declared target, never by favorable transfer.

## Delivery, evidence and review

Add a separate fixed diagnostic, proposed as `deduction adversarial-audit`, accepting no tuning flags. Its versioned strict report includes environments, frozen controller/action identities, full canonical fitness tables, controls/provenance, eight targeted audits, sixteen nominal rows, thirty-two cross-target rows, minimax-bound checks and independent reference comparisons. Integrity must recompute current payload relationships and the frozen protocol, not trust stored success flags. Numerical failure emits a failed report and exits 2; operational and write/flush failures propagate through the existing CLI conventions.

Before collection, independently verify action tables, fixed-only posteriors/payoff, all best responses and canonical ranking, normalization, score/regret identities, constant-policy bounds, alias invariance, public-view privacy, malformed input rejection, and report corruption detection. Include exact ties, renamed Agent IDs, swapped strategic-report bits for fixed-only, and proof that actual-policy identity cannot enter listener construction or queries. The audit's declared environments remain fixed; endpoint fixtures must not create fallback posteriors or claim support they lack.

Preserve all six existing diagnostic outputs and the complete deduction guide prefix. Retain first audit output, repeat output, independent oracle, source/settings/binary snapshots, all task reports/reviews and final verification. No setting or rule changes after seeing outcomes without preserving and labeling the first result as the earlier protocol. Product documentation records measured methods, tradeoffs and limits, rather than development bug history.

Execution requires an approved implementation plan, subagent-driven test-first stages, independent task and whole-increment reviews, and the standing Rust/WASM/web/Studio checks. Rebuild the native release CLI before web parity tests. Work in the existing crowd worktree; preserve the uncommitted institutional-corruption roadmap entry and unrelated work. Do not stage ignored evidence, `.claude/`, `papers/` or `survey/out/`. No commits, merges or pushes without explicit instruction for this increment.

## What this establishes

This establishes exact targeted exploitability and an attainable minimax payoff within a declared finite game, plus the nominal cost or benefit of conservative decisions. It does not establish empirical robustness, resilience to corrupt verification or colluding fixed reporters, unknown objectives, unrestricted reporting strategies, repeated reputation learning, active inspection or a strategic equilibrium for richer games. The resulting evidence will guide the next choice between bounded prior uncertainty and costly verification.
