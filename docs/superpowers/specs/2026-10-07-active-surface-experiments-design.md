# Choosing paid experiments on an uncertain surface

## Status and purpose

**Date:** October 7, 2026. **Status:** proposed design for review; no implementation or new experiment collection authorized by this document.

The user accepted the direction of replacing mandatory calibration with choices about probing, communicating, and trusted inspection. This proposal develops that direction into a bounded first study. The first study uses one experimenting Agent and one supplied responding Agent; this is the recommended scope assumption pending the user's answer. Simultaneous experiment selection by both Agents is a separate extension.

The question is whether an Agent chooses evidence that improves its expected task outcome after paying for that evidence. Model identification, using another Agent's information, and benefiting from communication remain separate measurements. An optimal controller in a finite supplied model is an engineering comparison, not empirical evidence of spontaneous cooperation or open-ended scientific discovery.

This builds on the completed [shared-surface study](../../shared-surface.md) and its [design](2026-10-06-shared-surface-design.md). Its original source, settings, first/repeat reports, oracle, and results remain unchanged.

## Alternatives and selected first scope

1. **One experimenter and a fixed responder, selected:** isolates experiment selection and permits an exact finite-horizon value calculation without another adaptive planner inside the planning problem.
2. **Two adaptive experimenters:** closer to collective research, but additionally requires an explicit account of each Agent's uncertainty about the other's information and decisions. Matching a centralized planner is a different objective from maximizing individual utility.
3. **Selecting only for model information:** useful as a comparator, but identifying a model can be irrelevant to the task. It must not define the main controller's reward.

The active controller maximizes the experimenting Agent's expected net task utility. Report the responder's costs and net utility, and group utility, separately. A policy advantageous to the experimenter need not pay its partner for responding. No reciprocity, negotiated cooperation, or autonomous willingness to assist is claimed.

## Retained world and information boundary

Retain the four physical mechanisms: shared persistent, shared resetting, private persistent, and inert. Retain opaque identifiers, the seven-symbol alphabet, generic write acceptance, four-slot rounds, and phase resets. Reads and writes do not receive delivery receipts. No spatial movement, gate, key, access restriction, new symbol semantics, or additional mechanism is introduced.

The experimenting Agent starts with a uniform prior over the four mechanisms. The responder is the existing supplied known-mechanism controller, with a point prior at the actual mechanism. Its prior is not an input to the experimenter. In each hypothetical model, simulate the responder with the point prior at that candidate model, its candidate private bits, and its candidate local history. Never select a candidate using the responder's actual history, actual mechanism, or actual target bit.

The responder's calibrated assistance is supplied structure. It is neither a second investigator nor a neutral model-free communication device. Its action-dependent evidence must be accounted for in the experimenter's likelihood, including live evidence when it is available.

Run both role assignments: experimenter A with responder B, and responder A with experimenter B. Retain the existing chronological slot ownership and role-specific probe/live grammar. Changing which role experiments must not exchange physical role labels or change which Agent owns a slot.

## Paid experiments and real stopping

Retain the three paid probe rounds from the existing calibration grammar as a supplied experimental apparatus. At the start of each round, the experimenter can either request one additional round or end probing and enter the live phase. There are at most three rounds. The probe round uses the existing role-local calibration actions and generic outcomes, including conditional acknowledgments. Choosing a round chooses a supplied experiment, not an invented handshake or an unrestricted sequence of actions.

Each completed probe round costs each Agent two credits, because every owned read, write, or wait costs one credit. Ending probing creates no further calibration slots and incurs no skipped-slot wait charges. Stop is a control decision at a phase boundary, not a free observation or a zero-cost physical wait. The live phase still resets the surface and issues fresh private bits.

The responder learns only that the public calibration phase ended. Before that boundary it follows the supplied round schedule; afterward it follows the supplied live schedule. There are no task bits during calibration, so stopping cannot encode an undisclosed task bit. The public phase length can convey information about the experimenter's model belief; record it as a public event and do not claim all information travels through surface reads.

The decision is renewed after every completed round, using only the experimenter's own prefix. The first study does not yet let the Agent invent a different probe within a round. This removes mandatory probe quantity while preserving interpretable experiment semantics. Choosing raw action sequences and simultaneous active partners remain later designs.

## Live choices, budgets, and rewards

Retain four independent fair-bit task pairs, the supplied encoder/decoder, and the original delayed three-round opportunity schedule per trial. Retain 48 starting credits per Agent, read/write/wait cost one, trusted inspection cost four, and reward 12 for a correct prediction and zero for an incorrect one. Costs persist across the episode. Prediction remains free, and target-posterior ties predict zero.

At each live trial's first decision opportunity, the experimenter chooses one of two supplied routines:

- **Inspect:** use the original no-communication routine for this role, inspecting its target and paying for its remaining scheduled waits.
- **Attempt communication:** use that role's supplied send/read routine, replacing its probability threshold with this explicit routine selection. An attempted exchange does not guarantee that the responder participates or that a usable symbol arrives. Infer the target from the resulting own observations; do not assign a successful decode in advance.

The responder continues its original known-mechanism policy. Actual and hypothetical rollouts must use the same contract. Choices can differ between trials as local evidence changes. The planner optimizes the entire remaining episode, including future routine choices and remaining credits. Neither target truth, realized reward feedback, nor future private bits are issued before their original observation boundary.

The responder's existing action decisions depend on its model prior, public clock, own read guards, own previous actions, and current private bit; they do not depend on its target posterior. Simulate that fixed action rule during planning. Once the active policy is solved, derive the responder's target beliefs and predictions by forward replay under the actual declared active policy. Do not reuse the old frozen Ensemble's likelihoods for a partner whose decision policy has changed. This distinction avoids recursive adaptive planning while retaining correct role-local prediction inference.

This supplies a small decision vocabulary: continue a probe round, stop probing, inspect, or attempt the existing communication routine. It tests choosing experiments and tactics; it does not demonstrate discovery of a previously unknown action vocabulary, invented language, or autonomous partner incentives.

## Exact planning and action-conditioned inference

Construct finite hypothesis states from mechanism, latent task bits, physical surface state, responder-local state, and the experimenter's complete local prefix. The experimenter's available state contains only its own prior, local actions/outcomes, current own private bit, credits, and public clock/reset/phase information. Research-only latent records stay behind the inference/planning boundary.

For every permitted decision, enumerate its possible local outcomes and update the original hypothesis masses. A chosen experiment is an intervention: do not multiply a candidate by a likelihood for why the experimenter selected its own action. Own known bits condition the target/model distribution once. Apply the hypothetical responder policy to each candidate rather than substituting the actual responder's decision.

Compute exact expected remaining reward minus additional paid credits by backward induction over the bounded decision tree. Include the local outcome and continuation value of a paid experiment, rather than rewarding information gain itself. Cache by an explicit sufficient state whose equivalence is tested against full-prefix enumeration; never merge states solely because their mechanism probabilities match when surface state or responder memory differs.

Use exact rational arithmetic with checked bounds. On equal expected utility, stop probing before continuing; in live routine ties choose inspection. Fix these rules before collection. Report search states, branch counts, time, and memory independently of world credits. The planner's computational effort is not assumed free in a broader intelligence comparison; this first rig prices physical actions and exposes computational cost as a separate quantity.

## Comparisons and frozen first matrix

Compare four unknown-mechanism decision policies against the same supplied responder:

1. Exact expected-net-utility planner with adaptive stopping and live selection.
2. The original fixed three-round probe policy and probability-threshold live controller.
3. A no-probe policy using expected-net-utility live selection, isolating the value of the optional experiment phase.
4. An inspection-only policy with no probe rounds, establishing the trusted-information accuracy ceiling.

Add a known-mechanism experimenter using the same live decision vocabulary as a separate positive control. Its time-zero certainty is supplied, not discovered. Do not call it an oracle over unrestricted actions.

The first matrix crosses these five policies with four actual mechanisms and two role orientations: **40 settings**, each covering all **256 task-bit sequences**, for **10,240 actual episodes**. Report paired exact policy differences on identical mechanism/orientation/sequence distributions. Roles remain separate; an overall prior-weighted value is a separately labeled summary, not a substitute for individual mechanism results.

Repeat the adaptive planner, no-probe live planner, and inspection-only policy against the already specified out-of-family data-inversion surface in both orientations: **six secondary settings**, **1,536 episodes**. Priors, routines, costs, and stopping rules remain frozen. This is a misspecification diagnostic, not an optimization target. The planner is optimal only for its specified family and decision vocabulary.

For this secondary surface, the responder receives the nominal SP point prior, exactly as in the original data-inversion control. It does not receive a description of inversion or an out-of-catalog prior. The experimenter retains its original four-model uniform prior. Neither controller learns the withheld inversion rule from the experiment configuration.

Do not add reward, probe-price, horizon, or prior sweeps after seeing outcomes. Any such comparison receives a prospective amendment and retains the original series. In particular, an exact controller may find no benefit from probing at the retained costs and horizon; that is a valid result. Prepare a mathematical pre-mortem of break-even costs, identification opportunities, and policy ties before collection, marking all derived predictions as predictions rather than measured results.

## Measurements and independent verification

Record every boundary decision, candidate masses, chosen action/routine, expected continuation values for every alternative, stopping round, own observations, posterior, paid costs, final predictions, and actual outcomes. Preserve the responder's local trace separately as privileged diagnostics. Report attempted messages, other-Agent write lineage received, inspection use, per-Agent reward/net utility, group utility, and model-identification censoring.

Distinguish a stopped investigation from successful identification, an attempted exchange from transmission, and transmission from benefit. Relative to the inspection-only policy, communication can save costs but cannot exceed its perfect target accuracy. Report public phase-length signaling separately from task-bearing surface transfers.

Before first production collection, independently enumerate the small decision tree and verify its values and tie cases. Check action-conditioned updates, full-prefix versus cached-state equality, absence of peer-history/private-state leakage, every legal budget path, both orientations, exact known/unknown controls, stop-at-zero/no-extra-wait semantics, and full current-payload replay. Deliberately changed alternative values or chosen routines must invalidate a report's success claim.

Keep operational errors distinct from an unsupported local history. Unsupported observations retain their incurred cost and stop before later spending. Failure mass prevents unconditional terminal means from being reported as complete. Supported but incorrect predictions on the data-inversion surface are findings, not automatically engineering failures.

## Scientific anchors and limits

[Khetarpal et al. (2020)](https://proceedings.mlr.press/v119/khetarpal20a.html) motivates explicit affordance intents and distinguishing supplied possibilities from learned consequences. [Wang, Wang, and Powell (2016)](https://proceedings.mlr.press/v48/wangb16.html) provides a value-of-information anchor for selecting costly experiments in support of decisions. Their binary-feedback model and knowledge-gradient algorithm are not implemented or numerically reproduced here. This proposal uses an exact finite decision tree instead.

An information-seeking comparator would address a different objective: reducing uncertainty rather than improving downstream task utility. [Pacheco and Fisher (2019)](https://proceedings.mlr.press/v89/pacheco19a.html) supplies an explicit mutual-information planning reference for a later separately designed comparison. No claim that task utility and model information are interchangeable is made.

The first study remains a controlled engineering extension with a supplied helper, finite mechanism catalog, probe grammar, task relevance, and message meanings. It does not establish spontaneous collaboration, empirical incident fidelity, learned social preferences, unrestricted exploit discovery, or empirical reproduction of these papers.

## Architecture, preservation, and handoff

Add a separate active-experiment namespace and CLI diagnostic beside the frozen shared-surface protocol. Compose checked physical transitions and exact probability utilities; do not mutate the original controller, protocol, diagnostic settings, or first reports to support the new study. Any unavoidable shared change needs an independently reviewed equivalence argument and old-output comparison before collection.

Preserve the existing crowd worktree and all experimental evidence. Never stage `.claude/`, `papers/`, `survey/out/`, or ignored evidence. Retain exclusive first/repeat outputs, source/settings/binary snapshots, independent reference, reviews, all declared settings, and failures. A more compact trace representation can reduce repeated prefixes only if reconstruction and current-payload integrity remain exact; report-size improvement cannot silently drop the raw observation contract.

This written design requires user review, followed by a separately reviewed implementation plan. Implementation uses subagent-driven development with independent task and whole-source reviews. Independently verify and freeze source/settings before collection, commit working increments, merge completed work into main while preserving concurrent work, push normally, and check CI for the exact pushed commit.

After this study, selecting raw probes or adding a second adaptive investigator requires a new design. Gates/access constraints, complementary reusable discoveries, and oversight remain later increments in the collective-agency campaign.
