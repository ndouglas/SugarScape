# Minds 9: spatial evolution of hoarding (design)

**Date:** 2026-10-02
**Status:** approved on 2026-10-02; implementation plan prepared for review. No implementation or diagnostic simulation runs yet.
**Program:** [Minds](../../studies/2026-09-27-minds.md), step 9; C1 of the
[collective-agency campaign map](2026-10-02-minds-collective-agency-program-design.md).
**Builds on:** Minds 5's caching, Minds 6's theft, Minds 7's inheritance and Minds 8's watching.
**Next afterward:** protection, then deception. Behavior trees and HTN keep their existing order.

## Purpose

Test hoarding strategies when storing, recovering, finding and defending food occur in the spatial
Sugarscape world. Separate effects of home travel, discovery, defense and selection. Ask whether
watching and non-caching strategies spread when they are inherited, including short observational
memory and the opportunity costs of raiding and guarding.

This is our experiment, not another reproduction of Vander Wall and Jenkins's non-spatial model.
Minds 7 was a partial reproduction contingent on gap choices. That is a reason to compare its
assumptions with spatial mechanisms, not a license to promise its threshold will reproduce.
The older Minds 7 design's conditional language about a later spatial study is superseded by the
approved program's decision to proceed with Minds 9.

## Sources and existing patterns

The primary source is Vander Wall and Jenkins (2003), *Reciprocal pilferage and the evolution of
food-hoarding behavior*, `papers/caching/vanderwall-jenkins-2003-behecol-reciprocal-pilferage.pdf`.
Printed pp. 662 and 664–666 distinguish scattered stores, a defended larder, non-overlapping
generations, stores-weighted reproduction and logit-scale quantitative inheritance. They explicitly
describe their model as a computational tool rather than a realistic mechanism.

Relevant existing implementations, inspected before this design:

- `minds/caching/`: surplus, reserve, burial cost, recovery, theft, watching and fate accounting.
- `minds/central.rs`: a home, real return travel, delivery and provisions. Its controller changes
  load capacity semantics and currently excludes theft/watching; it is a reference, not a switch
  to enable wholesale.
- `hoard/world.rs`: seasonal selection, two-parent inheritance, dead-parent generation means,
  zero/extinct cases and per-generation reporting.
- `rules/sex.rs`: overlapping Sugarscape births. These inherit the acting parent's caching rule
  and cheating flag, but children never watch. This is not the breeding regime for this study.
- `world.rs`, `rules/mod.rs` and `rules/movement.rs`: shuffled sequential turns, metabolism after
  actions, finite carrying room, pathfinding and occupancy.

Minds 8's measured span sensitivity motivates spans 2 and 7. Its first-winter convention supplies
the episode boundary: execute world ticks 0–199, measure the state at tick 200. No tick-to-bout
conversion makes this equivalent to the paper's 100 days of 20 bouts.

## Scope and alternatives

**Chosen:** a named spatial-hoarding extension to the existing `World`, plus a reusable core
experiment runner that breeds cohorts between independent spatial episodes. A single episode
runs in the existing CLI and browser. Multi-generation studies run through the survey and core
runner; a browser generation dashboard is outside this milestone.

**Alternative: extend ordinary rule S.** Rejected for this study because overlapping births,
endowment transfer, partner proximity and available birth sites change the selection question.
Ordinary sex remains unchanged; a later study can compare the two regimes.

**Alternative: add coordinates to `HoardWorld`.** Rejected because it would retain the paper's
food/search abstractions and duplicate movement, memory, theft and observation mechanics.

The scope excludes communication, learning new models, gates and other environmental controls,
concealment, combat and theory of mind. Minds 9 supplies a spatial defense baseline for later
protection and environmental-control experiments.

## Two separately verifiable layers

1. **Spatial episode:** fixed traits, home delivery, separate cache kinds, finite observation,
   explicit defense and measured action costs. No breeding occurs within the episode.
2. **Evolution runner:** freeze a breeding rule, construct fresh episodes, inherit traits and
   summarize lineage and strategy frequencies. Fixed-trait contests remain available separately.

Neither layer runs when its extension is off. Existing defaults, fingerprints, presets, ordinary
births, central-place worlds and golden entries stay unchanged.

## Spatial state and configuration

Use a reset-only `spatial_hoarding` config group. When enabled, validate: one good, walk speed 1,
positive carrying capacity, ordinary `central` disabled, no lab, no mixed caching, `caching.rule:
even`, and no sex, replacement, combat, disease, credit, trade or lifespan deaths. These restrictions
define the first experiment rather than introducing unrelated integration questions.

The baseline retains ordinary holdings capacity, caching reserve R, burial share, burial cost,
dig threshold, owner memory, loot and raid-choice rules. It does not adopt central-place load-only
capacity or free provisions.

| Setting | Initial value | Meaning |
|---|---|---|
| `enabled` | false | Activate the episode extension; off reduces to the existing world. |
| `larder` | 0.15 | Fixed larder-choice probability L for an ordinary episode. |
| `defense` | 0.5 | Fixed defense-target parameter D. |
| `guard` | true | Allow costly guarding; false isolates unguarded larder behavior. |
| `defense_slope` | 10 | Logistic slope, a disclosed Minds 7 gap choice. |
| `find_larder` | 0.25 | Discovery draw per contacted foreign larder; baseline equals scatter find. |

Validate probabilities in [0,1] and positive finite slope. Named presets override discovery rates
explicitly. `find_larder` is not the paper's apparency parameter: it is a contact-conditional chance,
and encounter frequency still comes from geometry and movement.

Each agent gains, only under the extension: a fixed home at its founder position, L, D, a larder
amount and age/fate ledger, a pending-delivery intent, and observed-larder entries. Scattered caches
keep their existing map. A scatter cache at home and a larder at home remain distinct stocks.

The runner supplies per-slot L, D, cheating and watching flags before the initial snapshot is
recorded. A checked cohort constructor handles this initialization; no public ad hoc mutation of
agents after world initialization. Ordinary `World::new` retains its existing behavior.

New behavioral state is included in keyframes, exports and fingerprints only when enabled. Read-only
inspection shows the home, traits, stores, delivery intent and current guard action.

## Contact and access: occupation is not defense

A home can be accessed from the home cell or one orthogonally adjacent, non-wall cell in the same
movement component. A home wall cannot occur because homes originate at valid founder positions.
This contact range models a larder entrance. Scatter caches continue to require their own cell.

Occupation of the home does not itself prevent larder access. Candidate generation targets reachable
free contact cells, including the actor's current contact cell. Rank contact endpoints by actual
path length, then site index; use the existing bounded pathfinder. A failed path follows the existing
give-up convention and costs the attempted turn. No diagonal or through-wall access is allowed.

This is a stated spatial choice, not a measured animal parameter. Its larger contact area makes
larders easier to encounter than individual scatter caches. Report contact counts alongside draw
success; do not infer a mechanistic threshold from a post-hoc ratio of loss rates.

## Storing and transporting food

After an ordinary non-guard turn, a non-cheater with positive surplus
and no active delivery computes q = `caching.share` × surplus, clamped by existing burial cost.
Draw one larder/scatter choice per positive allocation. At L=0 or L=1 take the deterministic endpoint
without a draw. Allocation is by batch, not by individual item; disclose this difference from V&J.

- **Scatter:** bury q at the current cell through the existing accounting.
- **Larder while in home contact:** deposit q into the separate home larder.
- **Larder away from home:** mark q as a delivery intent and return toward home on following turns.
  The food stays in holdings, occupies carrying capacity and can be metabolized. There is no remote
  burial and no second stock created by the intent.

While delivery is pending, take the ordinary walk/arrival action toward a home contact endpoint.
Incidental gathering, owner recovery and theft retain ordinary arrival semantics; forgoing scroungers
cannot acquire a delivery because they are cheaters. Do not start another allocation while returning.
After each turn clamp intended delivery to min(previous intent, current surplus). If none remains,
cancel. At contact, deposit that amount after the turn and clear the intent; count actual delivered
food and elapsed return turns. An occupied or unreachable destination never teleports food.

Burial cost applies at deposit, not at commitment. The intent is capped again so holdings cover both
deposit and cost. Food gathered later on the return is not automatically added to this delivery.
After deposit, the next turn resumes the chosen decision engine. With L=0, guard off and no larder
stocks, the episode controller follows the existing scatter controller without extra RNG draws.

## Recovery, observation and theft

Own-scatter recovery remains first on arrival. If it takes food, that is the turn's recovery.
Otherwise an owner in home contact and below the configured dig threshold can take from its own
larder up to carrying room, replacing that turn's harvest. It knows its own home; owner-memory off
continues to affect scattered caches, not this fixed home.

Larder deposits emit a burial event at the home entrance. Watchers use existing sight geometry and
span, but entries include the larder kind so they cannot be mistaken for scatter sugar at that cell.
Deposit observations give only the amount deposited, not omniscient current total stock. Sightings
of later deposits refresh and add as in Minds 8. No permanent foreign-home knowledge is supplied.

Known foreign larders offer contact endpoints while their entries are fresh. Apply the existing
`raid_when`, `raid_if` and `value` gates using the endpoint's ordinary site value and carrying room.
A positive raid replaces harvest; no additional stumble is drawn on that turn. On a denied or
empty attempted larder raid, clear that larder entry, count its outcome, then allow ordinary arrival
fallback. Clearing avoids repeatedly targeting an inaccessible stock from unchanged exact memory.

If no recovery or deliberate raid takes food, foreign larders in contact are stumble candidates.
Visit by owner id; draw `find_larder` once per nonempty foreign larder in contact. A hit on a guarded
larder counts a blocked discovery and transfers nothing; continue the ordered search. Take from the
first successful unguarded one. If no larder is taken, perform existing scatter stumbling and then
ordinary harvesting. This larder-first precedence is a gap choice; record skipped scatter draws and
compare the alternative order in the reported sensitivity panel.

A discovered larder is not automatically remembered for later raids: only watching supplies a
directed target in this milestone. Repeated contacts can independently discover it again. Loot
semantics, carrying room and metabolism are unchanged.

On arrival use this precedence: own scatter recovery, own larder recovery, observed larder raid,
observed scatter raid, larder stumble, scatter stumble, ordinary harvest. Within each kind use
owner-id order, taking at most one cache. Contact endpoint candidates sum fresh observed-larder
amounts accessible there, as Minds 8 sums observed scatter amounts at a site. This can overestimate
a single take and is reported rather than silently corrected. A successful recovery or take stops
the arrival sequence. A pending delivery is handled once in the post-action phase, with no second
allocation on its completion turn.

Fate accounting distinguishes cache kind through burial, recovery, pilferage and owner death.
Larders are lost with their owner as ordinary spatial caches are, unlike Minds 7's default retained
dead stores. Maintain per-kind amount conservation and aggregate compatibility.
Extend the pilfering/accounting gates to cover larder discovery even if scatter find and watching
are off. Existing off-path worlds incur no new indexing or logging.

## Guarding, cost and timing

Before shuffled turns, compute guard intentions in agent-id order from the tick-start state.
An eligible non-cheater is in home contact and has a positive larder. With guard enabled it draws
with probability:

`P(guard) = logistic(defense_slope × (larder / T − 0.5))`

where `T = max(R, 1) + D × capacity`. This finite target is a spatial-model choice; it is not V&J's
remaining-season minimum/maximum formula. Slope 10 is inherited from Minds 7's disclosed choice.

A guard holds its position for this turn and gets no site harvest or foreign theft. If hungry it
may recover its own larder under the same threshold and capacity as above; otherwise its harvest
is empty. Metabolism and mortality still occur. It starts no new caching allocation.

The guard blocks all foreign larder takes for the tick while the owner remains alive. It does not
block movement, scatter recovery, scatter theft or visibility. A dead owner's guard immediately
ceases. Frozen intentions avoid turning random initiative into whether an owner has already
announced defense. The boolean means invulnerable defense, explicitly an upper-bound baseline.

Count intended guards, executed guard turns, sugar recovered while guarding, blocked raids and
blocked discoveries. Compare guard off and the opportunity-cost probe described below. Neither
occupancy nor a blocked attempt alone is counted as a successful paid guard.

## Evolution between episodes

The core runner owns the episode configuration, an archived cohort, generation summaries and a
seeded breeding RNG. Each episode is a fresh `World`; normal reproduction and replacement stay off.
Default: 175 founder slots, 200 ticks per episode and 60 non-overlapping generations. An entirely
dead cohort stops as extinct; otherwise even a depleted episode completes at tick 200 for reporting.

All generations use the same episode seed, keeping initial landscape, founder slots and initial
endowments fixed while genotypes change. The breeding RNG is a separate explicitly seeded stream
using the run seed. Record both seeds, draw order, config and each cohort for exact replay. This
repeated environment is a choice, not environmental inheritance; varying environment between
generations is a reported follow-up, not silently added here.

### Fitness

**Primary: survival.** A founder alive at tick 200 has parent weight 1; all others have 0. Survivors
are picked uniformly with replacement. This permits non-cachers to reproduce if they survive and
keeps stored sugar out of the fitness definition.

**Neutral control:** explicitly sample parents uniformly from all archived founders, including the
dead, to remove mortality selection while retaining inheritance and sampling drift. This is a
counterfactual experimental control, not a biological breeding rule or a fallback for zero weights.
Run it beside the continuous-evolution panel, with the same initial cohorts and recorded seeds.

**Sensitivity: stores.** Parent weight is the living founder's larder plus scatter sugar left at
tick 200, as in V&J. Holdings are reported separately and not substituted into this definition.
If weights total zero, stop with `zero_fitness`, distinct from extinction. Never silently use uniform
parents or introduce a cheater subsidy. This rule structurally penalizes non-caching agents; their
disappearance here is not evidence against a survival-based mixed strategy.

Report survival and ticks alive per founder separately from selection weights. Survival is the
primary ecological outcome; compressed ticks-alive values remain secondary. No score from a dead
agent survives through its discarded cache holdings.

### Inheritance

Pick two parents independently by weight for each new founder slot in slot order. For L and D:

`z_child = h² × (z_parent1 + z_parent2)/2 + (1 − h²) × mean(z_generation) + Normal(0, V_seg)`

where z is the logit and the generation mean includes archived founders that died. Map back with
the inverse logit. Use Minds 7's finite endpoint clamp and normal sampler; defaults h²=0.8 and
V_seg=0.5. Initial L/D distributions reuse Minds 7's documented initialization at centers 0.15/0.5.
Values are centers, not guaranteed realized population means; report the realized means.

Cheating and watching flags come together from the first parent, preserving a heritable strategy
combination rather than introducing unmeasured recombination. No mutation of these flags. Initial
shares use the existing deterministic founder assignment unless a specified cohort is supplied.
In fixed-trait contest panels L and D do not mutate or regress. Ordinary rule-S births still have
their existing inheritance outside this runner.

Fresh episodes reset holdings to the episode's ordinary endowment, all caches, memories, paths,
delivery intents, fate logs and guards. Only traits and flags are inherited. Archive lineage using
(generation, founder slot), not recurring local agent ids. Close each episode's ledger with its
remaining stocks; reset is an experimental boundary, not metabolism or pilferage. Show generation
summaries even for terminal extinct or zero-fitness episodes; do not silently drop them.

## Presets and experimental panels

Single-episode presets use `theft-winter`'s world: 175 agents, capacity 50, reserve horizon 20,
100-tick summer followed by 100-tick winter, ordinary half-surplus allocation and owner memory.
The baseline uses `find` and `find_larder` 0.25, watching span 2, and `raid_if: better`.
Presets receive neutral mechanism names until measurements justify descriptive titles.

1. **Fixed-strategy panel:** L=0, 0.5 and 1, D=0.5; guard off/on; watching off/on at spans 2 and 7.
   Paired home-travel and guard effects; report stock exposure, contact, consumption and survival.
2. **Discovery panel:** scatter find 0, 0.02, 0.05, 0.25 and 1; larder find 0.25; watching off,
   span 2 and span 7. This is a predeclared mechanism sweep, not fitting apparency to a threshold.
3. **Continuous evolution:** L/D evolve; no cheaters; all watching off or all watching on at span
   2 and 7. Start at the defaults; report trajectories, endpoints and terminal failures.
4. **Strategy contests:** L/D frozen at 0/0.5, producers versus forgoing scroungers, with scrounger
   shares 0.1, 0.5 and 0.9; spans 2 and 7, plus watching off. Breed under survival. Separately compare
   burying watchers with burying non-watchers under the same shares and fixed traits.
5. **Sensitivity:** stores weighting, h²=1/V_seg=0, slope 5 and 20, reverse larder/scatter stumble
   precedence, and the guard-cost probe. Report separately, with no new main verdict after seeing it.

Use seeds 1–40 for spatial episodes and generations. Each compared condition uses matching run
seeds and full-run differences; a seed matches initial conditions, not a promise of identical RNG
histories once actions differ. Timings use seeds 1–5 and report whole-tick cost per agent-tick and
runner time per completed generation, together with survival and terminal rates.

The **guard-cost probe** permits a guarding agent to harvest its current cell after guard recovery
only when it did not recover sugar. It changes one action-budget assumption, records that extra
harvest explicitly, and is never presented as the ordinary defended condition. No probe changes
the paper's judge or any previously committed Minds verdict.

## Questions, measurements and judging pre-mortem

The primary deliverable is measured contrasts and evolutionary outcomes. Bind any Holds/Fails
judges in a separate reviewed judge amendment before running diagnostics or surveys; this design
does not manufacture a threshold to match the non-spatial result.

| Question | Primary measurement | Explanations to distinguish | Pre-mortem check |
|---|---|---|---|
| Does a larder pay? | Paired survival difference, L=1 minus L=0, guard off | Concentrated recovery versus return travel and concentrated theft | No free transfer home; report deliveries and returns. |
| Does defense pay? | Paired survival difference, guard on minus off at L=1 | Prevented losses versus displaced harvest | Occupancy must not defend; report attempts, guards and stocks exposed. |
| Does discovery alter evolution? | L trajectory and endpoint across declared discovery settings | Encounter mechanisms versus inherited regression/drift | No equality claim with V&J's 0.219; compare no-selection/inheritance controls. |
| Do scroungers invade or persist? | Inherited share change from rare and common starts | Frequency dependence versus never-caching costs and forgoing | Survival weighting primary; observing an interior share alone is insufficient. |
| Does watching spread at group expense? | Watcher share change and whole-cohort survival | Individual advantage versus collective action cost | Require a contrast against no watching and report span 2 beside 7. |

For paired episode contrasts report the mean, 95% t interval and per-seed signs. For evolution
report all trajectories, endpoint distributions, extinction/zero-fitness rates and parent-weight
distributions. Do not pool generations as independent samples; the run seed is the sampling unit.
Do not select the favorable cell from a sweep and label the mechanism generally beneficial.

An interior frequency is not a stable mixture. Evidence for restoring frequency dependence requires
contrasts from both rare and common starts with frozen continuous traits, repeatable directional
changes and sufficient survival. Evolution with logit regression/segregation variance cannot by
itself establish stability. A low or high L endpoint is reported with its exact definition if a
later judge adopts one; Minds 7's thresholds are not imported as empirical facts.

Cost probes, continuous-trait inheritance controls and a neutral survival-selection fixture must
separate mechanical bias from selection. If a mechanism is unused, a ratio undefined, or an episode
terminal, report it; a future judge amendment must define the corresponding Untestable treatment.

## Verification requirements

- Off: all existing core and web tests, golden fingerprints and serialization behavior unchanged.
- Endpoint: L=0/guard off reproduces the scatter controller without additional allocation draws.
- Delivery: no remote deposit; intended food remains in holdings, uses capacity, can be consumed,
  cancels when depleted and deposits only on contact. Burial costs cannot create negative holdings.
- Access: occupant present with guard off does not grant immunity; diagonal, wall-separated and
  out-of-range agents cannot access a larder. Multiple accessible endpoints have deterministic ties.
- Guard: fixed tick-start intention, no foreign harvest/theft, owner recovery permitted, mortality
  clears protection; blocked attempts counted separately from positive takes.
- Accounting: distinct scatter/larder stocks at the same home; conservation through partial digs,
  kept/eaten loot, burial costs, owner death and episode closing stocks.
- Watching: kind-specific observations, no omniscient total, span boundaries, opaque walls, stale
  or denied-target forgetting, raid choice and capacity behavior.
- Selection: hand-computed survival and stores weights; zero weights distinguished from extinction;
  parent sampling expectation and first-parent categorical inheritance.
- Continuous inheritance: hand-computed zero-variance logit regression, dead founders included in
  the reference mean, finite endpoints and documented draw order.
- Replay: cohort initialization before tick-zero snapshot, delivery/guard/observation state restored
  from keyframes, per-generation seed/cohort reproduction and matching native/WASM episode traces.
- Analysis: known paired differences, per-kind exposure denominators, terminal runs retained and
  fixed rare/common fixtures. No tests that merely copy an implementation formula without a behavior.

## Implementation boundaries and deliverables

Add the spatial episode controller under `minds/`, beside existing controllers, with explicit
helpers for larder contact, transfers and guarding. Reuse pathfinding, arrival, carrying and fate
accounting; avoid duplicating the whole world or borrowing central-place capacity semantics.
The core runner uses the checked cohort constructor and exposes completed generation records to
survey code. Configuration validation rejects incompatible combinations descriptively.

Deliver the config/schema, core episode and runner, single-episode CLI/browser presets and inspect
fields, native/WASM trace checks, survey panels, tracked results and documentation. The survey
registers a `minds9` module; UI generation playback and other biological reproduction regimes are
future work. Extend map inspection for home/guard and distinguish store kinds in the existing
caching measures, rather than inventing a new model menu for this experiment.

Implementation proceeds in stages: accounting/access; delivery/guard/controller; cohort breeding;
UI/CLI and replay; judge amendment and survey/reporting. The reviewed implementation plan will
assign precise files, meaningful tests, commit boundaries and subagents. This document authorizes
none of those steps until reviewed.

## Review decisions

The concrete proposed choices are survival-first selection, separate seasonal breeding, larder
contact from orthogonal neighbors, real home-delivery costs, paid guard turns, short-memory primary
comparisons, and explicit reporting of terminal and unused mechanisms. The full long-term program
does not require agreeing to every one of these choices; this campaign's review settles them before
the implementation plan is written.
