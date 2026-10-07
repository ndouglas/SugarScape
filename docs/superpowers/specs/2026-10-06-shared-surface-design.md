# Learning a shared surface for task information

## Intent and approval status

The user approved the conversational scope and this written specification on October 6, 2026. Implementation planning is authorized; the written implementation plan requires separate review before product code or collection.

Build the first small integration of environmental affordances, finite-model learning, and useful communication. Two Agents have separate task assignments and private observations. A generic status surface can sometimes retain information across time and make it observable to another Agent. Agents learn its behavior through paid observations and a supplied probe/acknowledgment protocol. We measure channel identification, transmission, and recipient benefit separately.

This advances the C2/C3/C4 integration in the [collective-agency program](2026-10-02-minds-collective-agency-program-design.md). It does not simulate the full Hugging Face incident. Unknown exploit discovery, invented language, distributed cumulative research, gates and keys, scoring deception, and oversight remain later increments. Start nonspatial: distance and access constraints would obscure the initial question of whether observed effects identify a usable channel.

## Scientific basis and claim boundary

[Khetarpal et al., 2020](https://proceedings.mlr.press/v119/khetarpal20a.html) supplies an affordance framework with specified intents. Learning this surface's effects is an inspired engineering extension, not a replication of that paper. We supply both the possible operations and a finite hypothesis family; an Agent does not discover an unrestricted action vocabulary.

[Derex and Boyd, 2016](https://pmc.ncbi.nlm.nih.gov/articles/PMC4801235/) is a later anchor for cumulative innovation and communication topology. Its sharing networks are supplied, and it is not a benchmark for discovering a channel. [Steels, 1995](https://pubmed.ncbi.nlm.nih.gov/8925502/) becomes relevant when we study learned conventions; this increment supplies message meanings.

Before any judged scientific experiment, review full methods and relevant critiques, choose predictions and judges, independently review the protocol, and freeze source/settings. The first implementation is an inspectable engineering experiment with exact checks. Do not call an expected software property an empirical replication, or interpret successful supplied handshakes as spontaneous cooperation.

## Alternatives and selected scope

A known message board is a necessary positive control, but cannot demonstrate learning. A gate/key/board world connects more directly to the eventual incident scenario, but mixes discovery, navigation, access, and communication. Select a generic status field with finite hidden mechanics and supplied conventions. This tests a narrower causal chain before introducing new mechanisms.

The deliberately supplied structure includes two roles, public turn order, calibration probes, a handshake grammar, encoder/decoder, a finite model family, priors, and a live controller rule. The learned quantity is the surface mechanism and its consequence for an available communication tactic. Neither role discovery nor protocol invention is claimed.

## World and legal operations

One episode contains calibration followed by four independent task trials. Agent A must predict bit Y, initially observed only by B; B must predict bit X, initially observed only by A. At each trial, X and Y are independent fair bits. Fresh private observations are issued at that trial's start, never during calibration. An Agent can pay to inspect its own target bit through a trusted local operation. This creates an independent alternative to communication.

Both Agents can issue `Read(surface)`, `Write(surface, symbol)`, `Wait`, and, during live trials, `InspectOwnTarget`. Their final choice is `Predict(0)` or `Predict(1)`. The surface's identifier and the Agents' identifiers carry no semantic information. The field holds one symbol; it is not a list, inbox, or addressed mailbox. Writes overwrite that field. Public instructions describe it as a status field, not as a guaranteed communication channel.

Legal actions are public and identical across mechanisms. A write returns only generic acceptance; acceptance does not say whether state changed, persisted, or reached anyone. A read returns the current locally visible symbol. Inspection returns only the inspecting Agent's own target. Illegal phase/action combinations, unknown identifiers, malformed symbols, or insufficient budget produce explicit operational errors rather than evidence about the mechanism. Frozen runs budget for all specified paths, so such errors indicate a broken protocol, not a successful scientific observation.

The alphabet is `blank`, `probe0`, `ack0`, `probe1`, `ack1`, `data0`, and `data1`. `blank` is the reset value and is not writable. The world stores raw symbols and never interprets their message meanings. The supplied controller encodes bits as `data0`/`data1`; it treats probes and acknowledgments as calibration evidence, never as live task data.

## Four finite hidden mechanisms

| Model | State affected by a write | What a read sees | Between-round behavior |
|---|---|---|---|
| Shared persistent (SP) | One common field | Common field | Retained |
| Shared resetting (SR) | One common field | Common field | Cleared to blank |
| Private persistent (PP) | Writer's private field | Reader's own field | Retained |
| Inert (IN) | No state change | Blank | Blank |

A round has four public slots. Resetting occurs after slot four and before slot one of the next round, including a calibration/live boundary. Explicit phase/trial resets clear all fields for every model before fresh task observations; this reset event is public. Otherwise there is no hidden decay, noise, access restriction, or stochastic write failure. Mechanics remain fixed for an episode.

SP and SR can both transmit a same-round handshake. Only SP can carry the delayed live messages defined below. Therefore detecting shared visibility is insufficient to establish a useful live channel. An Agent must separately learn retention across a round boundary. PP is the noncommunicating stateful negative control; IN is the accepted-but-ineffective-write negative control.

## Observation boundary and exact model inference

A controller receives its own action/outcome history, public phase/round/slot schedule and reset events, own budget, current private observation, and its supplied model prior and protocol. It receives neither another Agent's actual action/observation history nor the actual mechanism, other private target, evaluator rewards, host seed, archive, or a delivery receipt. Scheduled peer opportunities are known; realized peer choices are not directly announced.

Unknown-mechanism Agents start with uniform weights over SP, SR, PP, IN. Known-mechanism controls start with a point mass on the actual model. This treatment-specific knowledge is the intended intervention; do not erase it in the name of matching controls. The known/unknown knowledge condition of each role is public, but another Agent's actual point-mass prior is not exposed. Under candidate mechanism m, reconstruct an informed peer's prior as delta_m, or an uninformed peer's prior as uniform, then simulate that peer from its own possible local evidence. Never inject the real mechanism through the peer's prior.

For each candidate model, replay possible hidden trajectories consistent with public rules, the public peer controller, and the Agent's local transcript. Marginalize hidden peer observations and any task bits, not their realized values. A deterministic calibration model has likelihood zero or one. During live play, use exact rational masses over the finite private-bit worlds and latent peer histories. Maintain the candidate trajectory masses from the preceding local prefix, condition only on the new local observation, and renormalize exactly. Equivalently, recompute the complete transcript likelihood from the original episode prior; never multiply a full-prefix likelihood into an already updated posterior. Known task observations condition that Agent's inference; they must not leak into the peer's posterior.

Histories with zero probability under every supplied model emit an `unsupported_history` controller failure. Never reset to uniform, silently ignore contradictory evidence, or convert it into proof of a channel. Preserve the local transcript and prior before failure. Stop that rollout on controller failure and report its trace, spent credits, and missing terminal metrics as unavailable, never as zero reward. Out-of-family mechanism runs report coverage failure separately from incorrect in-family inference.

A posterior is the primary output; a maximum-weight label is descriptive only. Label ties use SP, SR, PP, IN in that order and never influence live decisions. Model accuracy must report true-model posterior and whether the true mechanism is uniquely identified; it must not count a tied label as knowledge. Define visibility belief as P(SP)+P(SR), retention belief as P(SP)+P(PP), and useful delayed-channel belief as P(SP).

## Supplied calibration and observable handshake

Calibration has zero, one, two, or three rounds, selected before running the episode. No task bits exist during these rounds. The table names scheduled roles; it is not a privileged observation delivered to either controller.

| Round | Slot 1 | Slot 2 | Slot 3 | Slot 4 |
|---|---|---|---|---|
| 1 | A writes probe0 | B reads | B writes ack0 iff its slot-2 read was probe0, otherwise waits | A reads |
| 2 | A reads | B reads | B writes probe1 | A reads |
| 3 | A writes ack1 iff its preceding round-2 slot-4 read was probe1, otherwise waits | B reads | B waits | A waits |

Every Agent has exactly two slots per calibration round. Every slot costs one credit, including a wait. Round 1 lets A observe an acknowledgment causally dependent on B's local read; it does not inform A of that read directly. Round 2 exposes whether the preceding round's field survived, separately from the handshake. Round 3 supplies a reciprocal acknowledgment to B. Controllers retain local memory across rounds, so a probe read before a reset can produce a later acknowledgment; the reset clears field state, not memory.

The complete three-round protocol distinguishes all four in-family models for both Agents. That is a designed identifiability check, not a promised emergent discovery. Truncation panels expose asymmetric and incomplete evidence rather than reporting only this easy endpoint. In particular, successful same-round acknowledgment cannot distinguish SP from SR; delay evidence is necessary.

For the no-communication controller, all calibration slots are waits. It has the same number of slots and credits, but never sends a calibration or task message. Report that policy difference explicitly. Known-mechanism controls run the same conditional probes as their unknown counterparts; avoided exploration is not a hidden additional advantage.

## Live protocol, costs, and supplied participation rule

A trial lasts three rounds. Public resets clear all fields before the trial. Each Agent has six slots per trial. The direction and delays are fixed:

| Round | Slot 1 | Slot 2 | Slot 3 | Slot 4 |
|---|---|---|---|---|
| 1 | A chooses write/inspection | B waits | B waits | A waits |
| 2 | A waits | B reads or waits | B chooses write/inspection | A waits |
| 3 | A reads or waits | B waits | B waits | A waits |

At its write/inspection opportunity, an Agent writes its privately observed bit about the other Agent's task iff its current P(SP) is strictly greater than 1/2; otherwise it inspects its own target. Exact equality selects inspection. This threshold and reciprocal-participation assumption are supplied behavioral structure, not learned altruism or an equilibrium claim. The threshold is motivated by a simple assumed reciprocal return: a correct received bit replaces a fair guess, while communicating avoids three inspection credits. Actual rewards still depend on realized peer behavior; the evaluator does not guarantee reciprocity. Asymmetric beliefs can cause one-sided costs and failed transfer.

Read decisions follow the actual chronology. A reads at round-3 slot 1 iff it wrote at round-1 slot 1; otherwise it waits and retains its inspected target. B reads at round-2 slot 2 iff its pre-read P(SP) is strictly greater than 1/2, otherwise it waits. B then chooses write or inspection at round-2 slot 3 using its updated posterior after that read or wait. B may inspect after reading; inspection replaces its inferred target with verified truth. If B writes instead, it retains its read evidence for the final prediction. A live read of a valid data symbol is decoded through the inferred finite generative model and supplied peer controller, conditioning on its own private observations and previous actions. Candidate trajectory enumeration executes these same chronological rules, using only each simulated peer's own prefix; it never conditions an earlier choice on a future action or realized private trace. Predict the target's exact posterior-majority bit, with ties predicting zero. Do not blindly accept one's own retained write on PP as information about one's target. No valid external evidence means a fair-prior prediction of zero. Inspection bypasses this decoder using the verified local target.

Inference updates continue during live trials; failed or contradictory reads can change later participation. Trial resets preserve model posterior and controller memory but clear task bits, predictions, and field contents. New task bits are independent of previous bits. Predictions occur after round 3 and carry no observation or credit cost. Trial truth and correctness are withheld until the episode ends and never used by the controller.

Read, write, and wait cost one credit; inspection costs four. Each Agent starts with 48 credits for the whole episode, not a budget reset between calibration and trials. Three calibration rounds plus four trials cost at most 42 credits per Agent (six calibration credits and nine credits per fully inspected trial). Both individual and total-group spending are reported. All treatments receive the same initial budgets, live opportunities, task distribution, and horizons. Waiting is an explicit opportunity cost, not free computation disguised as inactivity.

An Agent earns 12 reward units for each correct task prediction and zero for an incorrect one, then pays its spent credits. Report individual net utility and group sum separately. Reward parameters are frozen engineering choices, not empirical estimates. Positive communication is not required: unknown beliefs, resetting/private surfaces, asymmetric participation, and withheld model coverage can make communicating costly or useless. The no-communication baseline always inspects, then waits, without model-based messaging; it cannot inspect the other Agent's private state. This fully inspecting baseline has perfect target accuracy. Communication cannot improve gross correctness or gross reward against it in this game; recipient net benefit can arise only by saving inspection costs. Report that ceiling explicitly instead of presenting cost savings as additional correct answers.

## Frozen primary comparisons and measurements

Use all four mechanisms and calibration lengths 0, 1, 2, 3. For each pair, compare unknown-mechanism, known-mechanism, and no-communication controller pairs. Mixed informed/unknown pairs are secondary asymmetric-participation diagnostics, not selected after favorable results. Enumerate all 256 four-trial `(X,Y)` sequences exactly with equal mass. There is no seed search or training optimizer in this increment.

The primary engineering assertions are:

1. Complete supplied calibration identifies every in-family mechanism from local evidence, including B's PP/IN ambiguity until its round-3 read of probe1. Under SP, both Agents identify persistence during their round-2 delayed reads of ack0.
2. Truncated calibration preserves exactly the ambiguities allowed by the local transcript; a handshake alone does not establish retention.
3. Known SP permits delayed task information to reach recipients. SR, PP, and IN cannot causally carry that delayed information under this schedule.
4. Utility comparisons include all calibration, send, read, inspection, and wait costs. Failed transfer, negative utility, and ties remain first-class outcomes.
5. Same local transcript and prior imply identical posterior/action, even when privileged hidden world state differs.

Report per-Agent true-model posterior, unique identification, visibility/retention/useful-channel beliefs, and first local slot when each property is established at probability one. Unestablished discoveries are censored, not assigned zero cost. Report cumulative credits spent by that time alongside total credits.

Separate attempted sends, reads, decoded messages, actual other-Agent causal transfers, target accuracy, inspection use, sender costs, recipient gross reward improvement, per-Agent net utility, and group utility. A correlation between two guesses is not transfer. Track privileged write/read lineage to determine whether a read received the other Agent's task-bearing write, rather than its own retained symbol. Add in-support paired-bit sensitivity: hold the recipient's private bit and all other trials fixed, vary the sender's private bit (which is the recipient's target), and rerun both actual controllers and world transitions. Report whether the recipient's local read/posterior/prediction changes before truth disclosure. The paired target changes too, so this is dependence evidence, not a reward-effect estimate. Do not erase a write while still assuming the deterministic sender sent it: that can create an unsupported transcript and cannot supply a utility counterfactual. These evaluator trace checks never become controller feedback. Define recipient benefit as its expected reward/net-utility difference from the no-communication policy on the same distribution, not as a count of successful-looking symbols.

Do not require unknown to beat known, communication to beat inspection, or every Agent to cooperate. Exact numbers follow implementation and an independent oracle; this specification does not insert anticipated measurements as results.

## Transfer and negative controls

Renaming transfer consistently changes opaque surface/Agent identifiers in observations, command references, and paired fixtures while preserving role order and semantics. Posterior/action/utility invariance tests absence of reliance on identifiers. It does not establish semantic transfer, novel affordance discovery, or learning a new object category.

Changed-mechanics transfer is a separate episode with fresh empty fields and task bits. The public episode-start event announces that mechanics may have changed, without identifying the new model. Retain the previous posterior only as a recorded diagnostic; operational inference restarts at the declared uniform prior over all four models. Repeat the frozen calibration panels and live protocol for every ordered old/new mechanism pair. Report relearning cost, not zero-shot transfer. A no-change condition has the same public restart and reset, so it cannot receive privileged warning information.

An intentionally stale-posterior, no-restart variant is a secondary failure diagnostic: point-mass certainty can produce unsupported histories after a change. It is not compared as calibrated Bayesian inference under a stationary model.

Add an out-of-family shared surface that deterministically swaps `data0` and `data1` on write while leaving calibration symbols unchanged. Freeze this mechanism before collection. Calibration appears SP, but the live decoder's reliable-peer assumption is violated; incorrect confident predictions may occur without an unsupported history. This demonstrates that posterior consistency is not a guarantee of correct inference outside the catalog. It is an adversarial negative control, not an optimization result or a learned exploit. Report it separately from in-family accuracy and do not tune priors after seeing it.

## Architecture and verification boundary

Add a sibling finite-world namespace `shared_surface` in the core, with narrow world-transition, local-observation, inference/controller, evaluator, and diagnostic seams. Follow deduction's typed local-observation and exact-probability patterns. Do not reuse a Minds controller interface that accepts a privileged full World, broaden old observation types, or refactor unrelated models. Surface transitions may inspect world state; controllers may not.

Provide an additive inspectable CLI diagnostic, proposed as `shared-surface diagnose`, with no tuning flags for the frozen first protocol. The report records protocol version, supplied structure, model/controller identities, complete per-Agent local traces and posteriors, exact aggregate comparisons, budget accounting, transfer controls, and validation checks. Privileged evaluator traces are marked as such and cannot be reused as observations. CLI operational errors propagate; failed engineering invariants produce an explicitly failed report and nonzero exit rather than an invented result.

Before collection, independently derive the four-model calibration likelihoods, finite live trajectories, cost/reward calculations, private-state invariance, malformed-action handling, exact posterior ties, unsupported histories, and out-of-family deception. Use an independently authored small enumerator rather than copying production expected values. Tests verify boundaries and causal behavior, not merely controller branches.

Preserve existing deduction outputs byte-for-byte and existing product guide prefixes. New documentation must distinguish supplied protocol, learned mechanics, and measured outcomes. Retain first report, exact repeat, source/settings snapshots, independent oracle, reviews, and source verification in the ignored `.superpowers/sdd/2026-10-06-shared-surface/` directory. Disclose and preserve any post-result revision. Never stage ignored evidence, `.claude/`, `papers/`, or `survey/out/`.

Execution follows written-spec approval, a separately reviewed implementation plan, subagent-driven test-first implementation, independent task/final reviews, and repository quality gates. Commit working increments frequently; merge completed tasks into main, push, and check CI for the exact commit under the user's current standing workflow. Preserve the existing crowd worktree and unrelated work. Written-spec approval is complete; implementation-plan review is next.

## Handoff and limits

This first world establishes controlled learning and use of a supplied environmental communication mechanism. It deliberately supplies coordination, private-information relevance, and a cooperative participation rule. A later increment can compare unscheduled exploration and autonomous incentives; another can remove supplied meanings and study convention formation. Gates and access restrictions should follow only after the current observation/cost contracts are stable.

Distributed research then adds complementary unknown task mechanisms and reusable discoveries. Scoring pressure and oversight enter afterward, with separate task success and harmful circumvention metrics. None of those mechanisms or their scientific judgments is implemented by this specification.
