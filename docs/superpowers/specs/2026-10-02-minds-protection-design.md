# Minds P3: costly re-caching after observation

**Date:** 2026-10-02
**Status:** written spec approved by the user; implementation and verification are ready for whole-branch review. Scientific execution remains pending review of the committed executable manifest and opportunity evidence; no campaign has run.
**Baseline:** main `1223c47`, after the completed Minds 9 campaign.
**Plan/protocol:** [implementation plan](../plans/2026-10-02-minds-protection.md), [proposed measured protocol](2026-10-02-minds-protection-protocol.md).
**Related:** [Minds study](../../studies/2026-09-27-minds.md), [collective agency program](2026-10-02-minds-collective-agency-program-design.md), [research notes](../../studies/2026-10-02-minds-protection-reading.md).

## Decision and question

Make the first P3 experiment **re-caching scatter stores after a visible conspecific was present at burial**. An owner remembers that local event, retrieves food, carries it to another site, and pays for burying it again. The observer retains its old memory and can witness the new burial. Protection is a possible consequence of changing information and location; it is not a theft-blocking flag.

The discriminating question is: **does cache-specific exposure memory preserve more usable food than no relocation or indiscriminate relocation, once transport, metabolism, burial costs and renewed observation are counted?** Behavior selectivity and economic benefit are separate outcomes. A controller programmed to select exposed caches demonstrates the consequences of that supplied rule, not discovery of protection or animal-like reasoning.

The previous paid guard reduced survival across Minds 9's registered comparisons. This is a new mechanism and protocol; its results cannot replace or reinterpret those comparisons. P4 deception follows P3, and behavior trees/HTN remain later work.

## Alternatives considered

| First mechanism | Benefit | Reason for sequencing |
| --- | --- | --- |
| Re-caching after observation | Reuses scatter caches, sight, memory, walking and pilfering; exposes an information/cost tradeoff | Selected; smallest direct bridge from P2 watching to P3 protection |
| Concealment through shade or new environmental controls | Strong connection to ecological opportunities and agent abilities | Later: shade requires a new perceptual model, and choosing hidden sites changes both initial caching and observation |
| Cheaper or imperfect guarding | Tests the cost and effectiveness curve of the existing guard | Separate follow-up: does not answer whether remembering an exposed cache helps |

Existing opaque walls provide the first geometric opportunities. Their current sight semantics are sufficient; adding doors, locks, lighting or probabilistic visibility would confound this first test. Those remain concrete directions in C2.

## Empirical motivation and claim limits

Emery and Clayton's observed/private caching experiments motivated delayed relocation and a mixed-history control. Their recovery phase was private; the experiment did not establish a survival advantage from re-caching. The original article has a published erratum whose full text was unavailable during this review; this design uses the qualitative experiment and does not copy its numerical effects as targets. [Original study](https://doi.org/10.1038/35106560), [erratum](https://doi.org/10.1038/416349a).

A computational stress/memory account produced several re-caching patterns without attributing mental states. Thom and Clayton subsequently tested a prediction of that account and found selective relocation of previously observed food, but no increased caching after unexpected cache loss. The lesson is to construct discriminating interventions, rather than claim that a simple successful simulation explains the birds. Our indiscriminate comparator is a minimal alternative, **not an implementation of that stress model**. [Model](https://doi.org/10.1371/journal.pone.0032904), [experimental test](https://doi.org/10.1371/journal.pone.0052936).

Later Eurasian-jay experiments found little evidence for previously reported perspective/desire effects, including replication attempts. These concern a different species and different paradigms; they do not directly overturn the scrub-jay re-caching experiments. They do warrant reporting uncertainty, motivation and failed opportunities. [Amodio et al., 2021](https://doi.org/10.7554/eLife.69647).

P3 is a mechanism experiment in SugarScape, not calibrated bird biology, evidence of theory of mind, or an AI incident reproduction.

## Bounded first world

Start with a survey laboratory episode containing one owner, one potential pilferer, scatter food and opaque walls. Use ordinary movement, carrying limits, metabolism, burial and raid semantics. Disable spatial larders, guarding, reproduction, inheritance, trade and other unrelated modules. Restrict the first supported controller to the existing rule M movement path; reject unsupported combinations explicitly.

The laboratory supplies matched initial holdings, fixed burial opportunities and phase schedules. Once the initial deposits are made through the normal burial operation, suppress **new surplus burials** for every policy so the same original food can be followed. Reburial of the transported batch remains allowed. This is a labelled experimental restriction, not ordinary free-running ecology. Harvest opportunities remain present; protective actions displace them. Report the supplied schedule and restrictions with every result.

Use two fixture families:

1. **Single-cache economics:** observed/private initial burial, followed by a relocation opportunity and then ordinary pilfer/recovery opportunities. Cross whether the second burial can actually be witnessed and whether reburial is free or paid. The zero-cost condition still pays time and metabolism. Hold initial burial cost, deposited amounts and post-preparation holdings identical across cost treatments; change the live burial-cost setting only at the declared relocation-phase boundary, for every policy. Do not replenish holdings during the episode.
2. **Mixed-history selectivity:** equal deposits at two distinct source sites, one observed and one private. Mirror positions and reverse deposition order. Both become available for retrieval in the same episode. Equalize route lengths and opportunities so site preference is not exposure selectivity.

Phases release agents to act; they never teleport food or erase observer memories. During preparation, the pilferer can observe but cannot access the owner's stores. Later access uses predeclared action scheduling, without introducing an in-world door. Instrument time during which that restriction applies. The observed-second-burial treatment must use a fixed opportunity schedule, not reposition an observer in response to the owner's hidden intent. Because equal-range sight is symmetric, a redeposit that passes the owner's privacy cue but is actually observed requires a predeclared vision asymmetry. Keep that asymmetry fixed across the matched private/observed redeposit contrast; vary the supplied observer schedule, and report realized sightings as well as the assigned treatment.

The first fixtures deposit once per source and prohibit new deposits at a source after preparation. This deliberately avoids mixing newly buried private food with previously exposed food at the same owner/site. Supporting arbitrary mixed deposits is a subsequent design, not a hidden assumption that a site flag identifies individual food items.

## Controller, information and action contract

### Information available to the owner

At its own initial burial, the owner records whether another living agent is visible **in the owner's ordinary sight**. It does not inspect `Agent.watches`, `cheater`, another agent's vision, private memories, or actual successful sightings. Any visible conspecific is a possible witness. A separate researcher event records whether an observer really saw the deposit, using existing watching semantics.

Keep an exposure entry keyed by source site with burial tick and perceived-exposure bit. Bound entries by the existing memory cap; evict oldest burial tick, then lowest site index. Expire by a named span. Reading another agent's `seen` map is forbidden for all experimental controllers. Asymmetric vision can therefore produce false reassurance or unnecessary protection; measure those errors.

This first mechanism remembers **an event**, not observer identity or an estimate of another mind. Visible non-watchers and unseen long-vision watchers are required diagnostic controls.

### Policies

| Policy | Trigger for one relocation attempt | Purpose |
| --- | --- | --- |
| Off | None | Existing caching/watching baseline within the same lab schedule |
| Selective | Fresh exposure entry marked exposed | Cache-specific memory mechanism |
| Indiscriminate | Any prepared own source cache, regardless of exposure | Separates moving food from selecting exposed food |
| Memory erased | Same machinery as selective, with exposure entries cleared before retrieval | Isolates remembered events from current cues |

All moving policies use the same destination rule and physical actions. In a mixed-history fixture, each source receives at most one attempt; a failed or completed attempt exhausts that source's opportunity. Reburial never creates another eligible source. Off and erased controls retain the same legal action opportunities and timing.

### One relocation attempt

1. At the owner's action boundary, select an eligible source by oldest burial tick, then site index. Eligibility reads only owned source records and local event memory, never observer state. Do not start while holdings are below the ordinary caching reserve.
2. Select a destination from currently visible traversable sites with no own cache, excluding both prepared source sites. Minimize toroidal distance from source, then site index. Select only when no conspecific is currently visible to the owner; this is a **current privacy cue**, not a prediction about future visibility. Stay with that destination for the attempt.
3. Walk to the source using the existing legal movement/path rules. Each walking turn is a protective action that replaces harvesting, raiding and ordinary cache recovery. Pay ordinary metabolism. No remote retrieval, deposit or path reservation.
4. At the source, spend one retrieval action. Withdraw at most the current source stock and carrying room into ordinary holdings. Preserve the ordinary reserve. Record the amount as pending transport intent; it is not a second food store. A missing cache or no room ends the attempt with a named reason.
5. Walk to the destination, one ordinary movement action per tick, without gathering. After metabolism, clamp pending amount to current surplus; food consumed en route cannot be deposited later. Cancelling the intent leaves remaining holdings available for ordinary behavior.
6. On a separate deposit action, recheck the owner's current privacy cue and destination legality. If a conspecific is visible, cancel and retain carried food. Otherwise re-bury the affordable pending amount at the owner's actual position, using the existing burial cost. Any remainder stays in holdings. Existing observers may witness this burial and remember it normally.
7. Clear the intent, mark the source attempted and resume ordinary behavior next turn. An unreachable source/destination, depleted surplus, occupied destination at deposit, death or expired source memory cancels the attempt. No repeated waiting, rerouting or chain relocation in this first version.

Occupied cells and walls follow current movement rules. A failed path still consumes the attempted action. Ordinary agent ordering remains authoritative: another agent may pilfer before retrieval or witness a later deposit. Determine the trigger when the owner acts, not through a privileged pre-shuffle scan. Persist and hash all behavior-affecting memory/intent/attempt state; diagnostics must neither draw RNG nor affect behavior.

## Food accounting and observability

Reuse the underlying scatter withdrawal/deposit semantics, but add a dedicated relocation path: generic hungry-cache recovery must not accidentally replace a planned transport, and burying the pending food must not start ordinary surplus caching. Verify each caller's holdings update; the scatter `dig` helper returns an amount whereas larder transfer helpers update holdings internally.

Maintain both views of the food:

- **Operation counts:** legacy dug/buried flows can count repeated handling. Export explicit relocation withdrawal, transport and redeposit amounts, burial cost, action ticks, distance, cancellation reasons and observed redeposits.
- **Original-food outcome:** identify each prepared source cohort and follow it through cache, holdings, transport and reburial. Count each original unit once. Do not treat a relocation withdrawal as final beneficial recovery or its redeposit as new food production.

The bounded lab must collect a cohort ledger independent of the legacy optional FIFO fate log. Allocate holdings outflows proportionally across labelled cohorts and unlabelled holdings, recording consumption and costs separately; this is an accounting convention, not extra perception. A relocation withdrawal/redeposit retains source identity under that convention. Treat the first transfer to the thief as an absorbing outcome for the owner's cohort: later thief consumption or reburial must not count it again. Owner death terminates the remaining cohort as loss, with its location recorded. Reconcile per cohort: initial deposited amount equals live cached/carried amount plus owner consumption, thief transfer, costs and terminal loss, within a declared floating-point tolerance. Owner consumption of tagged food does not by itself prove benefit: compare whole-owner survival, harvest and holdings too.

The current `Fate::Dug` terminal event and new burial record do not supply original-food lineage automatically. Do not silently extend their meaning or double-count them. Ledger-disabled and ledger-enabled trajectories must agree. Missing/capped lineage makes cohort outcomes unavailable, never zero or success. All controls collect the same ledger, including watching-off cases where legacy fate logging may be absent.

Default-off must preserve existing state, RNG consumption, hash/checkpoint behavior and native/WASM trajectories. Reject enabling this bounded lab with larders or unsupported controllers. No product UI, trait evolution or broad free-world switch is required for the first deliverable; survey output must make the mechanism and lab restrictions inspectable.

## Measurements and competing predictions

Primary measurements are (a) the paired difference in thief-transferred original food and (b) the paired difference in owner survival time. Report separately, without folding them into a score. Secondary outcomes include owner-consumed original food, closing live stock, total harvest, holdings, metabolic/burial costs, action displacement, successful attempts, cue errors and old/new-site raid outcomes. Exposure selectivity uses relocated amount divided by initial amount **for each source**, with zero attempts retained in the denominator.

| Explanation | Expected diagnostic pattern |
| --- | --- |
| Changing observer information protects food | Less original food transferred to thief after private redeposit; smaller advantage when redeposit is actually observed |
| Generic relocation is sufficient | Indiscriminate matches selective economics; mixed-history selectivity adds no useful advantage |
| Costs dominate | Theft may fall while survival/holdings/harvest worsen, especially with paid reburial |
| Transport consumption explains lower theft | More food consumed/costed before redeposit, without improved survival or terminal availability |
| Past exposure memory drives the supplied policy | Erased memory suppresses selective attempts despite unchanged current privacy cues; this verifies the intervention, not learned cognition |
| Cue mismatch explains failure | Visible non-watchers prompt unnecessary moves; unseen watchers retain information despite perceived privacy |

Selective mixed-history behavior is a supplied-rule verification. Its economic advantage, failure under renewed observation and sensitivity to costs are the substantive mechanism tests. A wasted raid at an old location is not itself proof of false-belief reasoning or deception: the owner has moved real food, not fabricated a signal.

## Protocol to freeze before scientific runs

The implementation plan must include a separately reviewed, executable judge protocol specifying exact maps, deposits, holdings/reserve/cap, action schedules, memory spans, harvest/metabolism, cost values, stumble probability, episode horizon, seeds, matrix and estimands. This design is not a substitute for that registration.

Begin with the four policies crossed with initial observed/private burial, actual second-burial observed/private opportunity and zero/positive reburial cost: 32 cells. Add the mixed-history and cue-error controls as named fixtures, plus a separately declared stumble contrast. Pair initial worlds and seeds across policies; identical seeds do not guarantee identical stochastic draws after policies diverge. No evolution or policy tuning. Choose seed counts for uncertainty/reporting requirements, not the direction of a pilot effect.

Use hand-worked deterministic verification fixtures to establish that routes, costs, deposition, observation and eventual raid/recovery opportunities are possible. Those are construction checks and cannot be counted as independent supporting samples. Hold judged seed streams and mirrored transfer maps apart from construction fixtures. Report all registered cells and terminal episodes; use paired estimates and uncertainty with explicit denominators, and leave negligible/ambiguous effects inconclusive.

The pre-mortem must cover guaranteed policy selectivity, vision asymmetry, unreachable paths, zero carrying room, source theft before retrieval, re-observation, costs exhausting food, finite horizons, floor/ceiling theft, legacy ledger caps, and behavior changes induced by phase restrictions. Failure of a registered opportunity check invalidates that fixture's intended comparison; preserve its output and version a correction before rerunning. No post-result replacement maps, costs, thresholds or claims.

## Acceptance and handoff

Implementation acceptance requires deterministic tests for local exposure versus true sightings, mixed-history selection/order reversal, memory expiry/eviction/erasure, actual-position retrieval/deposit, partial carrying capacity, metabolism clamping, burial affordability, one-attempt limits, cancellation/death, old-memory persistence, re-observation and residual stumble theft. Test cohort reconciliation and diagnostic noninterference, unsupported-config errors, default-off reductions, serialization/hash coverage and native/WASM agreement.

After written-spec approval, write the implementation plan and judge protocol. Execute the approved plan with subagents, incremental tests and review. Run only the reviewed campaign, then report causal limits and costs alongside successful protection.

The subsequent decisions depend on results: concealment/site choice introduces capability-dependent opportunities (C2); identity-specific exposure memory introduces social knowledge (C5); false burials or misleading traces begin P4 deception. Costly helping, private messages, shared minds and distributed research retain their own campaign designs. This experiment supplies a reusable observation–memory–physical-action chain for that program; it does not establish those later capabilities.
