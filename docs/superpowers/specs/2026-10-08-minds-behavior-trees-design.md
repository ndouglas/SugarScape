# Minds 10 proposal: behavior trees and interrupted food-collection tasks

**Date:** 2026-10-08.
**Status:** design draft for user review; no implementation or scientific execution authorized by this document.
**Source inspected:** `b6788868df470bb3d7392715be12c51cca5623c6`.
**Reading:** [primary-source and code notes](../../studies/2026-10-08-minds-behavior-trees-reading.md).
**Program:** [Minds](../../studies/2026-09-27-minds.md) and [collective agency](2026-10-02-minds-collective-agency-program-design.md).

## Question and interpretation

What do a supplied persistent routine and its interruption checks buy or cost when an agent must collect a specified amount of food in a changing world? Compare task completion, resource use and computational work. Separately verify that a behavior tree implements its stated policy.

The user selected behavior trees after the completed pilfering campaign, followed by HTN and eventually a gate/message experiment. This proposal covers only the behavior-tree atom and a small comparison laboratory. It introduces no learning, social knowledge, communication, gates, new ownership rules or HTN.

A tree is a representation and execution mechanism. It does not acquire a policy merely because it contains a persistent sequence. The same guarded policy expressed as a small state machine must produce the same physical actions and RNG continuation. Any benefit over a reactive selector is a benefit of the supplied policy under these conditions, not evidence that trees are inherently more intelligent.

This is a controlled construction study, not a numerical animal reproduction. Existing foraging regularities remain possible later validation targets, but are not claimed by this task laboratory. Completed P3/P4, Democratic Peace and other studies remain immutable.

## Approaches considered

1. **A controlled foraging task, selected here.** Reuse food, movement, observation, memory and the existing GOAP search. Add a small tree executor and explicit task progress. Matched state-machine and persistence controls distinguish execution correctness from policy effects.
2. **Replace the P3/P4 scripts with trees.** Useful eventually for maintenance, but cannot validate a new decision procedure: their routes and decisions were supplied. It would also risk changing completed scientific sources.
3. **Start with a gate, tool or multi-agent task.** A valuable later direction, but introduces new action prerequisites, object mechanics and coordination alongside the controller. That makes an initial comparison harder to interpret.

## Controllers and common task contract

Each agent receives the same food-collection quota and its own cumulative gross harvest. Once the quota is reached, all controllers stop the assigned task. Their common laboratory wrapper then holds the actor in place with zero further gathering; ordinary metabolism continues. This post-task hold is an explicit supplied restriction. Lifetime differences can reflect excess food collected on the completing action and must not be interpreted as free-world foraging fitness.

Before quota completion, every controller can make one ordinary movement/gathering action per actor turn through `movement::arrive`. Walking speed is one. Gathering on intervening sites is retained. No tree node grants an extra move, free harvest or metabolism exemption.

| Controller | Supplied choice/execution policy | Role |
|---|---|---|
| Reactive utility | Existing `utility::act`, travel coefficient 1, crowding 0, idle stay; reselect each turn | Existing reactive reference |
| Guarded tree | Select by the same utility score and tie rule; retain the target while its common observed/remembered value is positive and the current known route has not failed; abandon invalid targets | Proposed routine |
| Matched state machine | Independently encode exactly the guarded-tree state transitions, candidate rules and tie calls | Execution equivalence control |
| Unguarded tree | Retain a selected target until arrival or physical route failure; omit the pre-action value invalidation check | Interruption-check ablation |
| Task GOAP | Existing GOAP search and foraging action/cost abstraction, with the remaining public quota as the search goal; retain/revalidate a plan | Matched-task search reference |
| Legacy GOAP | Existing `goap::forage::act` unchanged, horizon equal to the initial quota because metabolism is one; same common completion wrapper | Existing-policy reference |

Task GOAP is a new study adapter, not a relabelled result for the existing ecological GOAP policy. It reuses `goap::plan` and the existing foraging domain without changing legacy `act`. The initial task goal is quota minus actual gross harvest, clamped at zero. A retained valid plan can overcollect; the common completion wrapper stops it immediately after the completing physical action. Legacy GOAP continues to use `metabolism × horizon`, and this difference remains visible in the report.

For the guarded policy, a failure known from the previous physical attempt clears the target before a new selection. A currently observed zero also clears it. A stale positive memory can still lead to an empty arrival; there is no remote true-stock check. A merely better alternative does not invalidate a positive retained target.

The guarded tree, unguarded tree and matched state machine share an explicit failure-exclusion rule. A physical route failure to site s on tick t clears that target and records s with expiry t+3. Exclude s from selection on t+1 and t+2; expire the entry at the start of t+3, before quota checks or selection. Each failed site has its own expiry, in a map bounded by the finite grid's site count. A failure at another site does not erase an earlier exclusion. Retry after expiry is an ordinary selected physical attempt with the same one-action token, not a free probe. No intervention label or early opening notification shortens the expiry. A value invalidation is not a route failure and creates no cooldown. If no positive, unexcluded candidate remains, take the existing idle-stay action for that turn. Persist and hash the exclusion map. The two-turn cooldown is supplied policy, not learned recovery. Task/Legacy GOAP and the existing reactive reference retain their separately stated policies; do not attribute the complete primary contrast to persistence alone.

## Information and computation

Use the ordinary candidate/observation and memory channels. All agents are rememberers with map-prior memory, span 128, observation share 1, and speed-one walking. They know the initial food map; later resource changes become known only through normal observation and memory updates. They receive no scenario name, intervention timetable, future event, researcher stock or comparator result.

The physical pathfinder uses the existing terrain/occupancy model. The guard may inspect only a recorded route failure and the same current candidate information; it does not perform an additional omniscient reachability query. Existing GOAP also filters candidates by global wall connectivity. In the proposed connected-grid/single-blocker fixtures, the opportunity packet must prove that this extra filter removes no positive candidate solely for connectivity. It must also report any recorded-failure filtering separately. A disconnected-world extension would require a new information/fairness decision, rather than silently crediting the tree or planner with a larger observation model. Different trajectories can produce different observations. Match observation rules and initial information, not post-treatment observation contents.

All positive patches fit within GOAP's K=8 shortlist. Preserve its 4,096-expansion limit and fallback accounting. Bound a tree to 32 nodes/depth 8 and 64 visits per actor tick. Exhaustion prevents new node entries and is recorded; the transition before or after a physical attempt is defined below. These limits are different engineering units. Report node visits, candidate evaluations, target selections, path queries, search expansions, fallbacks and controller time separately; do not equate a node visit with an A* expansion or claim equal computational expenditure.

Primary comparisons match physical action opportunities and information rules. Computation is a measured cost, not secretly converted into food or simulation time. No compute-cost optimum is claimed.

## Tick semantics and implementation boundary

Implement a small typed Rust executor with `Success`, `Failure` and `Running`, conditions, actions, reactive sequence/fallback and a sequence with memory. Ordered children have fixed semantics. Conditions are read-only, have no RNG draw and never return `Running`. A halted child receives one halt transition, clearing its active cursor/target according to the profile contract.

An actor tick has a single physical-action token. Pure checks and target selection may proceed until one movement/gathering action is attempted. A successful or failed physical attempt consumes that token. Later consuming leaves are deferred until the next actor turn; a fallback cannot execute a second movement after a failed first one. The agent still receives ordinary metabolism once.

Physical-action settlement is mandatory and independent of the remaining traversal budget: retain the actual movement and harvest, update gross task progress and quota attainment immediately, record any route failure, and propagate the completed leaf's status through already-entered ancestors. This bounded unwind enters no new child and performs no physical action; its computational cost is still measured. If a new node cannot be entered because the visit budget is exhausted before any physical attempt, yield an idle physical turn. If exhaustion occurs after an attempt, retain that attempt and its actual harvest; never replace it with an idle/zero-harvest surrogate or roll back RNG/state. In either case retain the selected target, completed-child statuses and first unticked child cursor, with an exhaustion event identifying whether the token was consumed. Resume on the next actor turn after the outer quota/validity checks; those checks may halt the retained child. Quota attainment always takes priority and settles completion/halt/reset on the completing tick, even when further ordinary traversal is exhausted. Metabolism is applied exactly once outside the executor. Completed/failed root evaluations clear their traversal cursor; running/deferred evaluations retain it. The finite exclusion map follows its independent expiry rule.

Memory sequences resume their active child; reactive parents recheck their earlier conditions. Halting, completion, failure and budget exhaustion have explicit trace events and deterministic reset rules. No parallel nodes, background execution, arbitrary scripts, visual editor, learned tree generation or plugin dependency are included.

Add `decision.rule: behavior_tree` with closed supplied profiles. A `book_leaf` profile directly calls the existing book movement action once and keeps no tree runtime state. A `utility_leaf` profile calls the existing utility action once. These are reduction/verification profiles, not extra registered treatments. The guarded and unguarded task policies operate through a separately enabled laboratory configuration. Study-only controller variants and task progress remain authoritative state rather than diagnostic flags.

Suggested module split is `minds/behavior_tree/{runtime,forage,lab,records,runner}`. Reuse existing physical functions; keep the pure executor independent of `World`. Task GOAP exposes only the foraging-domain seam needed to supply a goal; legacy goal selection and execution remain unchanged. The implementation plan must name the precise adapters after the written design is approved.

Serialize all behavior-affecting tree/task/control state, including active child, target, planned value where used, quota and cumulative harvest. The enabled laboratory must also capture/hash retained Task/Legacy GOAP plans needed for continuation, without changing the historical disabled-laboratory fingerprint stream. Include authoritative extension state only while active. Checkpoint restoration must retain exact future actions and RNG continuation. Diagnostic collection must change neither. Cross-controller state fingerprints need not be equal; the matched tree/state-machine requirement compares complete physical traces and RNG continuation while retaining separately hashed controller states. The stateless book-leaf reduction stores no runtime/task extension and must match legacy golden fingerprints without changing golden entries.

## Proposed first laboratory

This table fixes a proposed bounded task world. It is not an accepted executable registration.

| Property | Proposed value |
|---|---|
| Geometry | 11×11 lattice with opaque outer boundary; empty interior except the stated blocker |
| Actor | One actor at (2,5), initial food 16, metabolism 1, vision 8, no aging death within horizon |
| Resource | One ordinary food; no regrowth, caches, theft, truffles, disease, trade, reproduction or selection |
| Initial patches | A=(3,5):4 units; B=(7,5):24; C=(4,9):24; zero food elsewhere |
| Capacities | A=4, B=24, C=36; zero elsewhere; initial map-prior information uses actual levels, not the larger C capacity |
| Quota | 20 or 40 gross units collected after tick 0; initial holdings do not count |
| Clock | 64 completed ticks; continue the clock/intervention accounting after death, with no replacement |
| Orientation | Base and x→10−x reflection; transform actor, patches and blocker consistently |

Each episode has one of four fixed, externally timed scenarios. Interventions occur immediately before the actor turn at the stated tick, after ordinary world preparation. They do not consult the controller, its target or whether it is alive.

| Scenario | Intervention | Purpose/known boundary |
|---|---|---|
| Stable | None | Benign opportunity and executor overhead control |
| Better alternative | Before tick 3, increase C from 24 to 36 by adding exactly 12 | A persistent target can become inferior for one quota while remaining useful for another |
| Depleted target | Before tick 3, remove whatever ordinary food remains at B | Test observed versus remembered invalidation; quota 20 remains attainable, but quota 40 is knowingly impossible because only 28 gross food units remain obtainable including any earlier harvest |
| Temporary route obstacle | Before tick 3, make initially food-free (5,5) opaque; before tick 7 restore it | Connected detour/rerouting control with no food creation/removal; it need not cause a route failure or tree interruption |

The single-blocker event is rejected if its site is occupied at the event boundary. Construction must verify this for every treatment/opportunity used before registration; do not conditionally move the blocker, shift the event or overwrite a scientific failure. Capacity/resource bounds must admit C=36. Record external additions/removals, every harvest, consumption, holdings and losses in an explicit conservation balance. No wall event can destroy an actor or silently discard food.

The quotas are deliberately different tasks. Do not pool them or choose the one favoring a controller. Do not assume that a longer commitment, a quicker completion or a larger harvest must improve all outcomes. The common post-completion hold makes their tradeoffs particularly explicit.

The temporary obstacle retains connectivity, and the existing pathfinder recalculates a path on each physical action. A changed path is not a tree halt. Count actual policy preemptions, route failures and physical detours separately; genuine unreachable-target, all-failed and later-recovery cases belong in software verification rather than being falsely advertised as measured properties of this scenario.

Proposed logical budget: **6 controllers × 4 scenarios × 2 quotas × 2 orientations = 96 cells; 40 seeds per cell = 3,840 episodes**, with new study-scoped registered seeds **30001–30040**. Seeds 7/8 are reserved for construction and opportunity verification. No registered World is constructed before the actual independent prospective gate. Exact executable matrix/config/protocol/command/output identities are frozen after implementation and verification; a defect requires a recorded amendment rather than silent tuning.

## Measurements and contrasts

Retain every cell and seed, including impossible quotas, death and null completion times.

- Quota attained within the horizon (0/1), and its first completing tick when attained.
- Restricted completion time: first completing tick if attained, otherwise 64, explicitly right-censored. This is a declared task endpoint, not a repaired missing record; record the unattained/dead/censored flag separately.
- Gross ordinary food gathered while the assigned task is active, remaining holdings, consumption and intervention balance.
- Actor ticks alive at tick start, including its death tick; survival at 64 separately.
- Movement steps, target changes, empty arrivals, failed routes, interruption/halts, resumed routines, plan invalidations and quota overshoot.
- Computational counters and timing with the declared boundary: controller decision/execution computation only, excluding world construction, frame I/O and analysis. Also retain whole-episode timing with its separate boundary.

Primary family: Guarded tree minus Reactive utility, separately in all 16 scenario/quota/orientation strata, for quota attainment, restricted completion time, gross gathered food and living ticks: **64 estimates**. Separate secondary family: Guarded tree minus Task GOAP, **64 estimates**. Separate ablation family: Guarded tree minus Unguarded tree, **64 estimates**. Legacy GOAP gets a complete reference table with the same endpoints, and its Guarded-tree reference differences form another separately labelled **64-estimate** family. No family or orientation is pooled into an overall verdict.

Food and lifetime are related accounting endpoints here. With fixed starting food, unit metabolism, no other losses and the common completion hold, collected food largely determines living ticks, subject to death before later collection and the horizon cap. Reporting both preserves their interpretation; two favorable signs do not constitute independent corroboration of benefit.

Matched state-machine comparisons are execution equivalence checks for all 640 paired episodes: physical sites/stocks/actions/holdings/deaths/task progress and RNG continuation must match exactly. They are not independent biological replications or an empirical claim of architectural superiority.

Use the repository's paired descriptive summaries, denominators and sign counts on all 40 seed labels. Report actual duplicate trajectories/endpoints and aliases. If the task is deterministic across labels, zero-width intervals mean repeated differences in a fixture, not population certainty. Unrestricted completion-time pairs are unavailable if any registered pair lacks attainment; do not summarize only successful episodes. Incomplete/invalid/pending scientific attempts are unavailable, never assigned a censored completion endpoint as a substitute.

## Pre-mortem and verification

The stable fixture may yield identical physical results. Fewer rank operations in a retained routine are partly guaranteed by construction; only total measured cost/outcomes can support a performance comparison. The depleted/40 quota cannot be attained and is a declared floor control, not a discovery. Map prior, cross-shaped vision, ordinary incidental harvest, Manhattan GOAP cost approximation, its abstraction's extra harvest tick, tie order and reflection can all explain differences. The common hold can make overshoot affect life independently of task speed.

Before scientific acceptance, show both favorable and adverse or null opportunity cases from the fixed construction fixtures, without selecting registered outcomes. Hand-derived candidate scores/path lengths and a small exhaustive task oracle verify the fixture; an oracle optimum is a software reference, not animal behavior. Reject ceilings/floors masquerading as discriminating targets. No positive-effect requirement is imposed.

Required verification includes:

1. Exhaustive short leaf-status traces against an independent reference interpreter/state machine: success, failure, running, preemption, halt/reset and at-most-one physical attempt. Tiny-budget traces must exhaust immediately before and after a consuming leaf and preserve exact action/harvest/quota/metabolism settlement, cursor and RNG continuation. Failure traces cover choosing an alternative, all positive candidates excluded, t+3 eligibility, repeated failure and later successful retry.
2. Golden book-leaf reduction and utility-leaf reduction, with identical RNG calls; legacy default-off/book/GOAP/protection/deception tests unchanged.
3. Checkpoint/replay at target selection, transit, invalidation and completion; mutate each behavior-affecting field to prove fingerprint coverage. Native/WASM per-tick agreement and diagnostics-on/off equivalence.
4. Matched-state-machine physical/RNG equivalence, target changes from permitted observations only, fixed intervention timing, reflection, conservation, impossible task/death and zero-food controls.
5. Save-data failure injection: exclusive attempt receipt before World construction, durable frames/outcomes, interrupted/pending/invalid census, full strict analysis and byte-identical fresh saved-data reanalysis.

## Gates, preservation and next artifact

The user's approval to proceed selected this design task. It does not approve a not-yet-reviewed written spec, implementation plan or numerical scientific run.

Next is user review of this written design, followed by a separately reviewed implementation plan executed through fresh implementers and independent task reviewers. Normal hooks, TDD, incremental working commits and the three-failure reassessment rule apply. Use a new isolated build environment; never rebuild a retained historical scientific executable.

Scientific measurement additionally requires a separate actual prospective review of the committed protocol, complete opportunity packet, source/binary/environment identities, fixed matrix/analysis and fresh output path. One writer then executes once; preserve failed/partial/invalid/unavailable/pending records without automatic retry, resume, overwrite or seed replacement. Reporting/empirical review, normal local-main integration/push, matching CI/Pages/served hashes and immutable archival follow only on actual evidence.

The planning branch contains only documents and ledger records. The user checkout's staged foraging work and every existing scientific source, binary, dataset, report and archive remain untouched.

## Ontology record

| Concept | Operational meaning | Does not establish |
|---|---|---|
| Task | Supplied quota and completion rule | An acquired preference or intention |
| Routine | Supplied tree with retained execution state | A learned skill |
| Interruption | Observed invalidation or physical execution failure | Knowledge of the intervention timetable |
| Recovery | Subsequent legal task action after halt/reselection | Success, benefit or adaptation by itself |
| Search | Existing GOAP machinery applied to stated action/goal model | Learning that model |
| Benefit | Separately measured task, food or lifetime contrast | A universal better controller |
