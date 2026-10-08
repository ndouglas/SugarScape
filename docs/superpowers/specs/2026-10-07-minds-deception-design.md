# Minds P4: false caching and receiver information — proposed design

**Date:** 2026-10-07. **Status:** Written design accepted for implementation planning. Implementation-plan review and the independent prospective scientific execution gate remain separate.
**Intended repository path:** docs/superpowers/specs/2026-10-07-minds-deception-design.md.
**Continuation:** P3 execution, publication and archival are complete. Its frozen registration, original data, executable and accepted archive remain separate.

## Purpose and scope

Test whether a caching-like action at an empty site can divert an observer, reduce theft of an original food cohort and improve owner lifetime after costs. A policy that creates a false estimate, a receiver that changes its behavior, and an owner that benefits are three separate outcomes.

The first experiment supplies sender policies and receiver observation rules. It measures functional effects within a bounded laboratory. It makes no claim of learned deceptive intent, theory of mind, animal cognition, evolutionary stability or numerical reproduction of a corvid experiment.

## Approaches and recommendation

| Approach | What it tests | Main tradeoff |
| --- | --- | --- |
| **False caching, recommended** | A visible caching gesture at an empty location, followed by ordinary receiver decisions | Extends the existing watching–memory–raid chain; requires an explicit distinction between visible action and physical food |
| Leading a competitor away from food | Following behavior, dominance and competition at rewarded/unrewarded sites | Closer to Bugnyar and Kotrschal's 2004 apparatus, but requires additional mechanisms beyond the current caching laboratory |
| Learned deceptive signaling | Sender/receiver policies adapting over repeated encounters | Tests emergence and counter-tactics, but adds training, feedback and evaluation commitments; keep it a subsequent registered study |

Start with false caching. Use its results to decide whether learned sender/receiver adaptation merits a separate design. General language, messages, reputation, behavior trees and HTN are outside this implementation.

## Evidence and what it does not establish

P3 found lower theft without necessarily longer owner lifetime, and mixed-history outcomes depended on recovery geometry. Therefore P4 retains separate economic endpoints and layouts, rather than treating empty raids as protection.

Bugnyar and Kotrschal (2004) studied four ravens, principally a subordinate/dominant male pair. Visits to unrewarded clusters could divert the dominant; later responses changed. The full paper reports a limited immediate advantage, but no detectable difference in total cheese gained per trial with versus without those moves in the specified first-phase comparison (p.73). Other birds and additional training complicate phase comparisons. This motivates separate diversion, total benefit and receiver-response measurements; it is not a target effect or independent population replication. [Primary article](https://doi.org/10.1007/s10071-003-0189-4).

Bugnyar and Kotrschal (2002) distinguish observing a final cache location from merely observing food handling and discuss false caches while citing Heinrich and Pepper (1998). This motivates separating perception from physical deposits, not assuming a quantified payoff for sham caching. [Primary article](https://doi.org/10.1006/anbe.2002.3056).

The cached Heinrich–Pepper text search did not locate an explicit false-cache measurement. This is an unresolved citation-tracing lead, not a demonstrated error or failed replication. Check the relevant original passage before attributing a false-caching result to it. Extracts and PDF/text identities are retained in this draft's source-audit.json.

## Information and physical action contract

Current watching receives the actual positive quantity from caching::bury. A zero-quantity burial returns before observation. Simply calling bury(0) cannot produce a believable fake cache.

Introduce a dedicated P4 action and observation path:

1. The sender legally walks to a registered display site. Movement consumes ordinary actions and metabolism.
2. A sham bout consumes one action and its declared effort charge. It moves **zero food**, creates no physical cache or cohort, and records no gross burial.
3. Observers whose existing lattice sight actually covers the bout receive a public observation. Hidden physical quantity and researcher truth are excluded from an ambiguous observation.
4. An ambiguous observation supplies a fixed nominal amount inferred from the gesture. A clear observation supplies only a physically visible transfer or verified no-transfer outcome. These are supplied perceptual assumptions, not a new validated vision model.
Ambiguous real and sham bouts must produce the same public observation when their visible gestures match: no is_sham flag, actual deposited quantity, sender policy or hidden stock balance. Route P4 receiver observations through one adapter; the legacy honest-burial hook must not also inject exact quantities or double-update that receiver. Outside P4, preserve the legacy path unchanged.

5. The receiver uses its resulting local memory through ordinary candidate selection, walking and arrival inspection. At an empty site it can discover the miss and clear that entry. It cannot inspect a remote cache map to reject a cue before arrival.

The sender policy deliberately attempts to induce an inaccurate estimate, but cannot read the receiver's private memory or actual knowledge. It knows its own action, possessions and locally available information. The initial study emits at most one sham bout; no infinite signal spam or fabricated remembered amounts.

Effort must be paid before a signal is emitted. An unaffordable or blocked attempt consumes its attempted action, records failure and emits no completed cue. Metabolism, resource removal, owner death and cancellations keep their ordinary accounting. A nominal amount is a belief parameter and never a second food balance.

## Treatments and proposed first matrix

Three sender treatments:

- **Ordinary reference:** resumes ordinary caching-lab behavior without a display excursion. It provides the total strategy-cost comparison.
- **Matched neutral:** follows the same display/return route, timing and effort charge as the sham sender, but performs a neutral action that carries no caching cue. Sham minus neutral isolates information effects.
- **Sham:** the same physical schedule and charge, with the caching-like observation at the display site.

Cross them with two observation modes (ambiguous versus clear food handling), two display opportunities (seen versus unseen), two layouts (display on the direct raid route versus off it), two effort charges (0 versus 3 food units per completed bout), and base/reflected orientation. Proposed budget: **96 cells × 40 paired seeds = 3,840 episodes**, seeds 20001–20040. These are a proposed registration, not executed measurements.

Use the familiar finite two-agent laboratory as the initial construction template: 9×9 with opaque border, one original 12-unit cache, owner/observer holdings 44/96, metabolism one, capacity128, reserve4, visions2/6 and two finite four-unit harvest patches. No growback, reproduction, replacement, guards, larders, trait evolution, discovery draws, generic communication or extra controller families. Run64ticks and continue after owner death; preserve actual early termination if both die.

Initial real caching must be genuinely observed in all cells so the receiver has a usable food opportunity without any sham. Initial observation is clear in both later observation treatments. A display site on the direct route is an important negative control: an empty raid there need not add any journey or preserve food.

The executable protocol will fix every coordinate, departure, bout, release, return and ordinary-behavior boundary before measurement. It must retain the same physical schedule and effort in the matched-neutral/sham pair; the ordinary reference can have different actions, with those differences reported. Candidate sites must be reachable and unoccupied at release. Owner occupation must not create free guarding. Register both observations and actual encounters rather than replacing blocked paths with teleports or remote raids.

Construction checks establish geometry, timing, sight, food balances and feasible opportunities. They must not require a favorable theft or lifetime result. Do not choose release times, cue strength or routes by trying outcome estimates. If a fixture is invalid, correct/version it before the prospective gate. If it is valid but effects are zero or negative, retain that result.

Truthful bouts, emptied-after-observation caches and affordable/unaffordable actions receive separate unit/construction checks. A receiver's response to a genuine positive cue must work; a legitimately depleted cache must not be relabeled a deliberate sham. Those checks are not additional campaign estimates or a hidden calibration panel.

## Measurements, alternatives and interpretation

For each of the32 receiver/visibility/layout/effort/orientation strata, compute Sham minus Matched-neutral on original food first transferred to the thief and owner ticks alive at tick start: **64 primary descriptive estimates**. Also retain the64 Sham-minus-ordinary economic contrasts as separately identified secondary estimates. No orientation pooling, favorable subset selection or overall Holds/Fails.

Use the existing paired-summary method, exact paired seed sets and descriptive Student-t95% intervals. Report sign counts, unavailable seeds, zero-width intervals, repeated endpoints and duplicate complete trajectories. Paired seeds do not guarantee identical RNG consumption after policies diverge and do not establish animal-data independence.

Retain every cell's actual cue exposure, believed site/amount, belief error, receiver target changes, detour distance/time, empty visits, subsequent correction, actual raids, original-food transfer/consumption/cost/loss, harvest, holdings, sender action costs, restrictions, deaths and horizon. A never-emitted or never-seen cue is not a successful deception. A detour is not itself food preservation; food preserved is not automatically longer life.

Competing explanations include real information diversion; unchanged direct-route travel despite a wasted raid; altered owner recovery due to effort or delayed harvesting; visible food transfer defeating the false cue; insufficient receiver access; body blocking; and index-dependent reflected recovery. In particular, paying food can change when hunger triggers owner recovery. The matched physical-cost control and per-tick traces must expose this instead of crediting all changes to misinformation.

## Architecture and verification

Use a dedicated, default-off deception lab/state/runner and survey route. Reuse ordinary movement, watching visibility, inspection and pure original-food ledger operations by composition. Keep P3's action vocabulary, registered outputs and frozen scientific identities intact. Do not add a general communication framework or rewrite existing controllers to support this one experiment.

Three existing patterns guide implementation: watching's local memory and arrival clearing; protection's explicit action/cancellation records and paired archive; spatial delivery's ordinary holdings and clamped pending intentions. Research truth belongs only in diagnostics. Any shared accounting refactor needs explicit equivalence fixtures.

Verification must cover zero food movement on a sham; budget/cost reconciliation; no cue when unseen or unpaid; identical public ambiguous observations for matched real/sham gestures, with no legacy side channel or double counting; clear empty/positive outcomes; truthful cache depletion; legal movement; occupied/unreachable/owner-death cancellation; bounded memory; default-off state/RNG/checkpoint equality; native/WASM parity; diagnostics on/off equality; strict archives and byte-identical saved-data reanalysis.

Provide --deception manifest/help routes without simulations, a clean committed-source/fresh-destination run gate, and a fresh saved-data analysis route. Persist an attempt receipt before constructing each episode, then its outcome or error, so an abort cannot silently lose the attempted-cell identity. Retain invalid, partial, dead, undefined, unavailable and pending counts. Never overwrite, resume or replace a failed registered attempt automatically.

A separate executable protocol, full manifest, actual build/source/binary identities and construction opportunity packet require independent prospective review before any registered seed runs. Postmeasurement findings, actual PNG/SVG inputs/renders, fresh empirical/whole-branch review, local-main integration/push and verified external archival follow the established workflow.

## Ontology and completion boundary

The physical cache is an owner-associated food stock; a caching-like bout is an action; an observation is the receiver's available sensory evidence; a remembered estimate is private agent state. False estimation is measured against diagnostic truth. Functional deception requires a supplied misleading policy and a resulting mistaken estimate; behavioral diversion and economic benefit are reported separately. None establishes a representation of another mind.

The written design is the current reviewable deliverable. After its review, write implementation tasks and the prospective protocol using subagent-driven development with fresh independent reviewers. Implementation approval does not substitute for the actual later scientific gate. P3 and historical Minds artifacts remain unchanged throughout.

**Planning approval:** The user accepted the reviewed written proposal with “That sounds reasonable.” The original proposal SHA-256 is `c385f7c553e38c5c1395cd309fc8dc70310152f25a14e6b2a24b3fff82224702`; only approval metadata differs in this repository copy.
