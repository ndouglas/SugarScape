# Exact inference from testimony

## Intent and scope

Continue the capability-composed deduction experiment toward reasoning about deceptive claims. The user wants scenario-independent mechanics, individually testable atoms, and a definition of poor play grounded in available information and expected consequences. This increment supplies an exact, bounded reference model of testimony before integrating any claim-aware game controller.

A statement is an observation of a speaker's behavior. Its likelihood depends on a proposition, the speaker's private signal, and the speaker's reporting strategy. Infer proposition truth and persistent reporting strategy jointly. A mistaken statement need not come from a deceptive strategy; a true statement need not establish a trustworthy speaker.

This architectural design is approved. The existing deduction engine, protocol, policies, CLI sessions, and frozen deduction-v1 experiment remain as implemented.

## Approaches considered

1. **A fixed credibility weight:** cheap, but conflates signal quality with reporting behavior and fails to preserve uncertainty about a persistent speaker across statements.
2. **A small exact joint model (recommended):** enumerate plausible proposition/strategy assignments, marginalize private signals, and condition on attributed reports. Exact fixtures expose incorrect assumptions and decisions.
3. **Recursive strategic belief models:** eventually useful for incentives and opponent adaptation, but premature before the likelihood and dependence atoms have known-answer tests.

The chosen model is intentionally finite. It describes behavior under declared reporting policies; it does not establish psychological intent or solve strategic conversation.

## Components and data

Add `crates/sugarscape-core/src/deduction/testimony.rs`, with exports from the existing module. Reuse the current bounded decision helper for diagnostic choices. Core never reads stdin or an Engine archive.

All identifiers are explicit numeric IDs, unrelated to scenario names or roles:

- `PropositionId`: one Boolean proposition, with an optional display label.
- `ProfileId`: a reporting profile with probabilities `[P(positive | signal false), P(positive | signal true)]`. Copying, inverting, and always reporting positive are data values `[0,1]`, `[1,0]`, and `[1,1]` respectively, not branches in the inference algorithm.
- Speaker IDs use the existing `AgentId` type.
- A hypothesis has an ID, prior probability, one truth value per declared proposition, and one reporting-profile assignment per declared speaker. These assignments are latent; the model never receives the actual game assignment.
- A signal group has an ID, a proposition ID, and known signal accuracy `q`. One latent Boolean signal is drawn for the whole group. Groups are independent conditional on the hypothesis. Multiple reports may share that signal, including reports from different speakers.
- An evidence record has a unique `EvidenceId` and either an attributed positive/negative report `(group, speaker, value)` or an independently verified proposition `(proposition, value)`.

Signal groups and verification are explicit assumptions supplied by the caller. The present game protocol cannot identify which claims share a private signal or turn an accusation into a verified fact. This increment does not invent that knowledge. Unknown provenance and uncertain signal quality can later become additional latent hypotheses.

Use ordered vectors and deterministic ID lookup. No new dependencies, engine fields, protocol version, PolicyKind, ModelKind, browser integration, or renderer. New public structs reject unknown serialized fields. Structural validation happens before any conditioning.

Proposed API responsibilities:

```rust
Belief::new(model: TestimonyModel) -> Result<Belief, TestimonyError>
Belief::observe(record: EvidenceRecord) -> Result<(), TestimonyError>
Belief::snapshot() -> BeliefSnapshot
```

`TestimonyModel` owns propositions, speakers, profiles, hypotheses, and signal groups. `Belief` owns its validated model, accepted evidence ledger, and normalized joint posterior. `BeliefSnapshot` returns hypothesis probabilities, proposition marginals, and per-speaker profile marginals in declared ID order. No actual hidden truth is attached to a snapshot.

Bounds: 1..16 propositions, 1..32 speakers, 1..16 profiles, 1..256 hypotheses, 0..256 signal groups, and at most 512 accepted distinct evidence records. IDs must be unique and references complete. Every hypothesis assigns every proposition and speaker exactly once. Prior probabilities and channel probabilities must be finite and in [0,1]; priors sum to one within 1e-12, with positive mass. Zero prior support and deterministic channel endpoints are valid. Empty evidence returns the prior. Reject capacity overflow rather than silently forgetting evidence.

This ceiling keeps enumeration and full-ledger recomputation simple. It is not a claim that all combinations of 32 speakers can be enumerated: callers must choose an explicit supported hypothesis set. General capability grants and objectives are outside the testimony model.

## Likelihood and correlation

For a group g about truth t with accuracy q:

`P(Z = t | h) = q`, and `P(Z != t | h) = 1-q`.

For a positive report from speaker s with profile p, use `p[Z]`; for a negative report use `1-p[Z]`. Reports within a group are independent conditional on its shared signal and the persistent profiles, as explicitly assumed by this channel. Thus:

`L_g(h) = sum_Z P(Z | h) * product_i P(report_i | Z, profile_of_speaker_i_in_h)`.

The joint posterior is proportional to the prior times all group likelihoods times the constraints from independently verified propositions. Persistent profiles belong to the hypothesis, so they are marginalized after combining groups, not independently re-drawn for each statement.

Compute group terms and posterior weights in log space, using log-sum-exp for the two signal possibilities and final normalization. Represent impossible events as negative infinity; never introduce a probability floor that makes impossible observations possible. This avoids underflow from many small nonzero likelihoods.

An identical replay of an existing evidence ID is an idempotent success, even at capacity: no ledger or posterior changes. Reusing an ID with different content is an explicit error. A fresh report ID is a new emission: the declared signal group determines its correlation, rather than string similarity or message count. The caller is responsible for assigning provenance honestly.

Different groups do not imply unconditional independence: they still share latent proposition truths and speaker profiles. Multiplying separately marginalized credibility scores is not this model.

## Transactionality and error handling

Construction rejects malformed models descriptively. Observation validates the record and calculates a candidate posterior before changing the ledger or posterior. Unknown IDs, conflicting evidence IDs, exceeded capacity, and evidence with zero probability under every supported hypothesis are distinct errors. All errors leave the entire belief unchanged. Repeated snapshots are pure and deterministic.

An impossible observation means the declared model has no supporting world. Return that information; do not select an arbitrary hypothesis or silently reset to the prior. An independently verified fact can be assimilated only when the caller has a legitimate observation channel for it. Diagnostic truth is used for scoring, never secretly fed into inference.

## Exact fixtures

Fix the following cases before collecting any report. Use independent rational arithmetic to establish reference values; production uses f64 with error tolerance 1e-12.

| Fixture | Declared assumptions | Exact reference |
| --- | --- | --- |
| Empty evidence | Any valid prior | Joint posterior equals prior |
| One uncertain speaker | Truth prior 1/2; persistent copy profile prior 3/4, invert prior 1/4; signal accuracy 4/5; one positive report | Proposition probability 13/20; profile prior remains 3/4 |
| Two signals, same speaker | Previous model; two positive reports in different signal groups | Proposition probability 49/68 |
| Shared signal | Previous model; two positive emissions from the same signal group | Proposition probability remains 13/20 |
| Duplicate record | Re-submit the identical ID and content | No change to ledger or posterior |
| Two independent witnesses | Both known to copy; signal accuracy 4/5; distinct groups; both positive | Proposition probability 16/17 |
| Two witnesses, common signal | Same witnesses, one shared group | Proposition probability 4/5 |
| Inversion | Known invert profile; truth prior 1/2; accuracy 4/5; positive report | Proposition probability 1/5 |
| Uninformative reports | Known always-positive profile, or accuracy 1/2 | Positive report does not change truth prior |
| Verified truth and transfer | First uncertain-speaker fixture, then independently verify proposition true | Copy-profile probability 12/13; a positive report about a second independent, prior-1/2 proposition with a fresh 4/5-accuracy signal gives truth probability 49/65 |
| Model contradiction | Deterministic copy profile; one shared signal; contradictory reports | Zero-evidence error, no mutation |

Also test ID renaming/order invariance, malformed/NaN/infinite probabilities, incomplete assignments, unknown references, normalized marginals, transactional failures at nonempty history, deterministic endpoints, and long low-probability histories that would underflow naive products.

The second-signal fixture is a useful diagnostic: treating the speaker's credibility as independent on every statement would yield 169/218 rather than 49/68. That difference comes from the persistent latent strategy, even when the private signals themselves are independent.

## Decisions and poor play

Reuse `best_accusation` on a binary proposition posterior `[P(true), P(false)]`, with utilities +1 for a correct positive accusation, -1 for an incorrect one, and 0 for abstaining. Restrict the diagnostic choices to a positive accusation or abstention: map the helper's Some(0) to positive accusation, and Some(1) or None to abstention. For these fixed +1/-1/0 utilities this yields exactly the optimal permitted choice; an accusation about the complementary proposition is not an available action.

At posterior 13/20, a positive accusation has exact utility 3/10 and is better than abstention. At posterior 1/5, it has utility -3/5 and abstention is optimal. A credulous positive accusation loses expected utility 3/5 in the inversion fixture. At 1/2, tie favors abstention. These regret values compare against the optimum under the same posterior and declared information, not an omniscient choice for an individual realized world.

Establish reference posterior and utilities using independent rational finite hidden-state enumeration before implementation; diagnostic output compares against those fixed exact constants, rather than using the inference implementation to manufacture its own reference. Report posterior error, chosen action, expected utility, best attainable utility, and regret. Whole-game wins and claims about a universally better controller are not part of this increment.

## Diagnostic host and documentation

Add a separate `sugarscape deduction testimony` command for the known-answer diagnostic report. It runs the fixed fixtures, outputs JSON, and exits 0 only when all exact reference checks pass. I/O failure uses exit 1; malformed arguments or failed diagnostic checks use exit 2. Keep `deduction diagnose` and its deduction-v1 measurements/schema behavior intact.

Report a new `testimony-v1` fixture version, channel assumptions, exact reference strings, measured posterior/utility/regret, and pass flags. The fixture definitions and tolerances are fixed in source and documentation before the first command run. There is no seed or empirical superiority hypothesis for these enumerated cases.

Extend `docs/deduction.md` with the model, real diagnostic output, dependence assumptions, and operational limits. Explain that testimony can update belief without becoming an engine fact. Human/LLM agents retain the existing turn protocol; integration of this model into their decisions is a later design step.

## Validation and completion

Use test-first implementation with independently reviewable stages: the pure channel/validation atoms; joint conditioning, provenance and marginalization; then exact diagnostics/CLI/documentation. Follow the existing subagent-driven workflow, preserve current uncommitted work, and commit only on explicit request.

Completion requires the exact fixture matrix, atomicity/underflow tests, a real diagnostic command run, independent review, and the standing Rust/WASM/web/studio checks. No post-result tuning of fixture assumptions or expected answers. If an error is found in a reference calculation, recompute it independently and fix the reference rather than changing the inference algorithm to fit it.
