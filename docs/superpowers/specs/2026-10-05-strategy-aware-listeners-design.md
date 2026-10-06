# Strategy-aware listeners without a revealed reporting policy

## Intent and status

Written design approved by the user on October 5, 2026. The user authorized continuing from the completed strategic-reporting increment. Implementation-plan review precedes execution and collection. Commit only on explicit instruction.

The user selected correct Bayesian inference and its identifiability limits as the primary objective. Fixed-policy transfer remains a diagnostic of sensitivity to model assumptions. Robust adversarial decision rules, adaptive reporters and protection guarantees are deferred.

A deciding Agent should reason over possible reporting strategies and their incentives rather than receive the actual reporter policy. Preserve capability-defined mechanics, independently testable atoms, strict information boundaries and exact numerical checks. Determine what one calibration exchange can identify, and how much protection comes from a supplied prior rather than information discovered during play.

## Approaches considered

1. **A small weighted policy catalog, recommended:** inspectable hypotheses, exact rational inference, direct identifiability tests and inexpensive independent enumeration.
2. **Uniform uncertainty over all 262,144 encodings:** broader support, but independent arbitrary calibration/live table bits need not permit calibration to predict live behavior. It is a useful later reference, not automatically a useful trust model.
3. **Repeated interaction or simultaneous coevolution:** makes learning reporter tendencies possible but adds memory, verification schedules and moving opponents. Defer until this bounded inference model is verified.

Start with the first approach. This models uncertainty about strategy within a supplied family, not arbitrary social intelligence or discovery of motives from nothing.

## Existing game and new boundary

Keep the strategic-reporting assembly and its historical diagnostic unchanged: one strategic reporter, one persistent-profile reporter, calibration verification, live reports, and intervene/abstain. Truths, signals, permissions, utility and public observations retain their existing meanings.

Create a sibling `deduction::strategy_inference` module. A listener receives public Config, public history, and its own explicit policy prior. It must not receive the actual selected policy, reporter search seed, genotype, private signals/profile, truth before verification, host archive, or evaluator scores from the current game.

A model may enumerate hypothetical worlds from public rules. This is inference, not access to the realized private world. Reuse existing exact history generation through its public API. Do not expose private `World` objects to controllers or change the meaning of `FrozenListener`.

## Policy catalog and exact prior

Default catalog contains five existing controls: copy signals, invert signals, always positive, always negative, and copy during calibration then invert during live reporting. These are explicit supplied hypotheses. The true policy may be absent.

In the well-specified model, select one policy before play from the declared prior, independently of C, T, the fixed reporter's profile and all signal noise. Retain that policy through calibration and live reporting. Policy weights encode supplied uncertainty; the listener does not derive a policy-selection likelihood from the reporter's utility or assume the reporter adapts to its current decisions. Fixed-policy transfer instead holds the actual policy constant across the enumerated games.

Represent at most eight policy entries, each with an unsigned integer weight from 0 through 32; normalize by their positive total, at most 256. Reject empty/all-zero catalogs, invalid encodings, unsupported versions and unknown fields. Zero-weight entries remain disclosed but have no prior mass.

Identify policies by canonical reachable behavior: preserve the two calibration bits, and clear live rows whose own calibration-report bit cannot follow that calibration table. Reject duplicate canonical entries rather than granting duplicated encodings additional probability. Canonicalization is structural, not conditioned on a particular q or observed sample. The previous forty champions do not become forty independent behavioral hypotheses simply because some encodings differ.

Two priors are fixed before measurement:

- **Uniform:** weight 1 on each named control.
- **Optimization-informed:** weight 1 on each of the first four controls and 16 on copy-calibration/invert-live, giving the latter probability 4/5.

The second is a declared modeling choice informed by the previous experiment's opposed utility and legacy training panel. Equivalently, it mixes a uniform catalog prior with probability 1/4 and a catalog-best-response prior with probability 3/4; that panel has one best control. It does not assert that real reporters optimize this panel or that this is a population estimate. Expose the weight vector directly, separate from permissions and utility. Do not silently regenerate it from the current listener's own actions or holdout results.

Its optimization target was the previous legacy panel, not these new listeners. Transfer results may therefore expose a poor behavioral prediction. Any benefit before distinguishing evidence arrives must be attributed to the supplied prior rather than learned strategy recognition.

## Inference and identifiability

Construct exact joint masses over catalog policy, C, T, fixed profile and four signals. Maximum state count is 8*128=1,024. With existing probability bounds, common denominator is at most 4,194,304*256=1,073,741,824; use checked bounded integer arithmetic.

Condition on calibration reports and verified C to obtain a policy posterior and predictive live distribution. Condition on the complete public history to obtain P(T) and the final policy posterior. Compute the final condition from the same original joint distribution, or prove equivalent sequential conditioning; never assimilate calibration twice.

Choose intervention by comparing true-T and false-T integer masses; exact ties abstain. Unsupported histories return an explicit zero-evidence error. Do not insert a probability floor or invent a fallback posterior. Model support errors are reported separately from decision payoff.

Required identifiability fixture: Copy and CopyCalibrationInvertLive have identical calibration likelihoods. Their posterior odds remain their prior odds after any supported calibration history. Calibration accuracy cannot distinguish these two policies. Live reports may change the odds through evidence from the fixed reporter; this is indirect probabilistic evidence, not live-truth verification.

The named priors give Copy:CopyCalibrationInvertLive odds of 1:1 and 1:16 respectively. Assert these odds after every supported calibration history, even when evidence changes their combined posterior mass relative to other catalog entries. For general catalogs, compare positive-mass odds by exact cross multiplication; a zero-weight entry cannot gain posterior mass.

Report conditional policy probabilities, actual-policy-independent listener P(T), intervention and expected regret. Avoid claiming strategy classification where hypotheses are observationally indistinguishable. A singleton prior is only the informed-policy limit and must be labeled accordingly.

## Frozen comparisons

No GA training, new reporter optimization or interactive provider is included. Listener construction is complete before measuring any outcomes. Use q=4/5 and q=3/5, fixed-profile rho=3/4 and the existing opposed reporter utility.

Evaluate two distinct questions:

1. **Well-specified mixture:** draw the reporter policy from each listener's own declared prior and enumerate the complete joint model. The listener must have zero decision regret relative to an independently implemented mixture-aware reference. Compare the unchanged legacy listeners and Passive on this same distribution. This checks inference under its assumptions, not robustness against arbitrary adversaries.
2. **Fixed-policy transfer:** evaluate each listener against every named control, the prior experiment's forty frozen champions, and the independently established Evolved-target optimum encoding 98342. The last is absent from the default catalog and is an explicit misspecification case. Preserve every row; do not select favorable opponents after observing results.

Deduplicate behavior for aggregate reporting while retaining all seed/encoding provenance. Explain that the forty previous champions have one reachable reporting behavior, so they do not supply forty independent adversarial challenges. Report per-policy results rather than creating a misleading sampling confidence interval over those clones.

For fixed-policy transfer, distinguish listener regret against the actual-policy informed reference from reporter regret against any chosen listener. This increment does not compute a new strategic equilibrium or claim a new reporter best response. Belief calibration is measured under the declared mixture and separately under fixed-policy misspecification, with their different interpretation explicit.

For each positive-mass complete public history, report the listener's P(T), the independently computed actual-distribution P(T), their signed difference, and the history's actual probability mass. The belief metric is the maximum absolute conditional truth-probability error over supported positive-mass histories, retaining every history row so rare cases remain visible. It must be exactly zero under the well-specified mixture; fixed-policy values diagnose misspecification rather than strategy-classification accuracy. Do not pool the two priors' self-mixture results as though they used the same opponent distribution. Fixed-policy rows provide their comparison on common opponents.

For every evaluated listener, also report the actual probability mass of unsupported histories. If it is positive, unconditional payoff and decision regret are unavailable because the listener has no specified action there. Supported-history contributions and any payoff normalized by supported mass must be labeled explicitly, with the normalization mass disclosed. A zero-evidence error is neither abstention nor a zero-payoff action. The maximum belief error is likewise conditional on supported histories and is unavailable if none are supported. Expected support failures in declared misspecification or endpoint fixtures are valid diagnostic outcomes; the well-specified mixture must have zero unsupported actual mass.

Settings, priors, panels and metrics are fixed before collection. Negative results are valid outcomes. Any subsequent adjustment motivated by observed results must retain the first collection and be labeled retrospective.

## Atoms and validation

Keep catalog validation/canonicalization, joint mass generation, conditioning, decision selection, evaluation and reporting independently testable. Public inputs use the project's strict serde conventions and explicit versioning. Preserve old commands and guide claims byte-for-byte; append new methods/results separately.

Independent Python Fraction references must establish normalization, per-history policy/T posteriors, well-specified expected payoff and regret, the calibration-indistinguishability fixture, and fixed-policy baseline values before collection. Include prior endpoints, zero weights, q/rho endpoints, exact ties, unsupported histories, invalid catalogs, reordered entries and renamed Agent IDs. Catalog ordering cannot change a belief or decision.

Additional required tests: singleton prior agrees with the existing informed-policy reference; splitting equivalent encodings is rejected; sequential versus direct conditioning agrees; live signals or truth cannot enter calibration conditioning; rejecting malformed public input does not mutate an existing model; inference uses only public views and declared priors.

Reference and integrity checks also verify the per-history belief errors, their exact maximum, supported/unsupported mass accounting, and unavailable unconditional metrics when support fails. A well-specified listener's zero mixture decision regret does not require zero regret against the reference that knows each realized policy; those references have different information.

Add a separate fixed diagnostic, proposed as `deduction strategy-inference`. Its exit status reflects reference and integrity checks, never superiority over legacy listeners. Report checks must recompute relationships from current payloads. No change to existing sessions, archive semantics, CLI play, arbitrary capability language, browser UI or video is necessary.

Execution requires a reviewed implementation plan, subagent-driven test-first stages, independent final review, retained first results/source/settings, and all standing checks. Rebuild the native release CLI before web parity tests. New presets are not introduced.

## What this establishes

This can establish exact decision-making under uncertain reporting policies and expose what the chosen evidence cannot identify. It cannot establish inferred unknown objectives, a validated population prior, repeated reputation learning, strategic equilibrium or unrestricted human/LLM competence. Those remain separate increments.
