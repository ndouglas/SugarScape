# Burrow 1 excavation and soil transport

**Date:** 2026-10-03

**Status:** proposed first-world design for review; implementation and judged runs have not begun.

**Programme:** [Cultures, construction and underworlds](../../studies/2026-10-03-cultures-construction-and-underworlds.md),
branch B, beginning with its B1/B2 questions. This is not a new Minds milestone.

## Intended outcome and acceptance scene

Build a cheap, deterministic experiment in collective excavation that can become a foundation
for individual, diverse Hornvale underworlds. The first study investigates how transport rules
and local construction cues affect growth and work. It supplies work motivation and controller
rules; it does not claim agents invent architecture, roles or culture.

The acceptance scene is a seeded CLI replay showing the starting substrate, successive excavated
maps, workers, carried and deposited spoil, and an external disposal count. A JSON event trace
and action/material summary accompany the map. Running the same configuration and seed again
must reproduce the same trace. This command and its implementation do not exist yet.

## Scientific question and alternatives

**Question:** how do relay transport and responsiveness to recent spoil deposits change the
amount, distribution and cost of excavation in a bounded, initially simple world?

The selected [reading anchor](../../studies/2026-10-03-burrow-excavation-reading.md) is Pielström
and Roces's leaf-cutting-ant transport and choice study. The computational rules below are
proposed abstractions; the study does not establish their parameters or predict that relays
improve our model's performance.

Considered approaches:

1. **Local excavation and explicit transport, recommended:** separates material and cue effects
   with a small action model, and provides an inspectable route toward inhabited underworlds.
2. **Reproduce an existing ant/robot excavation model:** stronger initial quantitative target,
   but its escape-barrier task and physical/controller assumptions address a different question.
   Retain as a separate candidate campaign after full equation and code review.
3. **Generate complete burrow plans with constraints:** immediately yields layouts, but answers
   a generation question rather than how agents construct them. Retain in branch B's generator work.

The first experiment must allow useful, harmful and negligible effects. A supplied cue response
can concentrate work while reducing output; fixed relay segments can add handling overhead.

## Scope and architecture

Propose a `burrow` module in `sugarscape-core`, with configuration, substrate/material state,
actions, controllers, fixtures, ledger and replay views separated by responsibility. Add a
`burrow` CLI subcommand following existing argument and error conventions. Use existing Serde,
Clap, portable RNG and deterministic data structures. No new dependency is required by this design.

Do not insert excavation into the harvest-returning Minds dispatch or the Kirman `ants` model.
The first lab does not register a new `ModelKind`, change Sugarscape defaults, add web controls,
or modify Hornvale. Native core behavior and a small WASM replay parity check are part of
verification; interactive web hosting and full model/sweep integration belong to a later design.

The suggested command accepts a configuration file, seed, tick budget and output directory.
Its output contains normalized configuration, final map, sampled maps, per-tick summaries and
an event trace. Exact flag names are implementation-plan details; existing CLI success, I/O and
validation exit-code conventions apply.

## World and material semantics

- Finite, bounded, horizontal four-neighbor lattice; no wraparound, diagonals or gravity.
- Cells are solid substrate or open space. Excavation converts exactly one adjacent solid cell
  into open space and creates exactly one uniquely identified spoil unit in the worker's hands.
- An immutable diggable mask can protect solid cells in prepared fixtures. Growing-world substrate
  is all diggable. A solid cell outside the mask is neither traversable nor excavatable.
- Each worker carries zero or one unit. Digging with full hands is illegal. No partial excavation
  is included in this slice: one successful dig action completes a cell.
- Loose spoil occupies an open cell as material overlaid on traversable floor. It does not refill
  the cell or physically block a passage. This approximation excludes pile-induced obstruction.
- Disposal at the designated exit moves a carried unit into an external counter. Units never
  disappear through aging, pickup, drop or failed actions. Excavation timestamps do not reset on drop.
- Workers have stable IDs and occupy open cells, at most two per cell. Capacity is a supplied
  coarse constraint, not a body-scale measurement. Spawn configurations must respect it.
- Boundaries cannot be excavated beyond the array. The exit remains open and cannot be removed.

Maintain the invariant:

`initial loose spoil + excavated cells = carried spoil + loose spoil + externally disposed spoil`.

Fixture-introduced material is included on the left side. Also verify the solid/open cell count
against the number of successful digs. Separate geometry from spoil state in the authoritative schema.

Each tick gives every worker one action opportunity in a seeded permutation of IDs. Decisions
and commits are sequential: later workers observe earlier committed actions. Successful move,
dig, pickup, drop and disposal each consume one opportunity. Blocked attempts and waits also
consume one, and are logged separately. A tick is not a physical second. Report worker action
opportunities alongside ticks so population changes cannot masquerade as efficiency improvements.

## Observations and navigation scaffold

Unloaded workers observe their current cell and open cells within two traversable hops, the
solid faces adjoining those cells, nearby occupancy and nearby loose spoil. Solid cells occlude
observations beyond the frontier. Researchers retain the full world and material provenance.

The cue observation is the count of nearby units younger than the configured freshness window,
not exact creation timestamps, creator identity or unobserved work history. Freshness is a supplied
sensory abstraction; this design does not establish the chemical cause or kinetics in real ants.

Loaded workers use a shortest-distance-to-exit field computed over current open cells. This is
explicitly supplied navigation shared by every treatment, not learned route knowledge or emergent
communication. Record its computation cost. Replace it with local learned navigation in a later
experiment if that question becomes important. Unloaded workers do not receive global frontier targets.

## Controller and action rules

All workers use the same state machine; roles are not assigned by ID. Controller state contains
an optional locally selected frontier target and, while loaded, the number of successful moves
since acquiring a unit. Random choices use the lab's existing seeded RNG implementation. Equal
seeds ensure replay; different treatments may consume randomness differently, so equal seeds alone
do not guarantee identical decisions.

Loaded workers:

1. At the exit, dispose of the unit.
2. Under relay transport, drop the unit after the configured number of successful loaded moves,
   provided the current cell is not the exit. Dropping resets the worker's carry state.
3. Otherwise move to an unfilled neighboring cell with strictly smaller exit distance; break ties
   randomly. Wait if no such move is legal. Carry distance counts successful moves, not waits.

Unloaded workers:

1. If there is loose spoil at the current cell, attempt pickup with probability one half. Pick
   the smallest material ID so material selection is deterministic. If not picked, continue.
2. Continue a valid frontier target, or choose among observed frontier cells reachable through
   observed open cells. An observed frontier is a solid cell adjoining observed open space.
3. Move one step along a shortest observed path to an open cell beside that target; dig when
   adjacent. Keep the target until dug or until it leaves the observed reachable set. Then reselect.
4. If no frontier target is available, move uniformly among legal open neighbors, or wait.

For a target face, count recent units on observed open cells adjoining that face. Cue-blind
selection assigns every target weight one. Cue-responsive selection assigns weight three when
at least two recent units are present, otherwise weight one. Draw proportional to integer weights.
These weights and thresholds are supplied experimental choices, not fitted biological estimates.
No controller has a desired room count, prescribed nest shape or global construction plan.

Expose the positive integer response weight, positive minimum unit count, freshness window and
positive relay distance as validated lab parameters; the values above are demonstration defaults.
Reject invalid dimensions, arithmetic overflow, out-of-bounds fixtures, disconnected spawn sites,
non-open exits, excess occupancy and zero freshness windows with contextual configuration errors.

Pickup handles physical material under both cue treatments. Cue blindness removes only the
influence on frontier selection, not the ability to see and transport loose spoil.

## Fixtures and demonstration defaults

### Cue choice fixture

Use a 9-by-7 world, with open cells `(2,3)` through `(6,3)`, a worker at `(4,3)`, and eligible
solid digging faces restricted to `(1,3)` and `(7,3)`. Other adjacent substrate is non-diggable
in this fixture. Both face approaches are equally distant and visible under the observation rule.
Place a matched pile at either approach, mirror its side, and vary recent versus old units.
End at the first frontier selection, before pickup or transport changes the offered cue.

Use four units for the accumulation treatment. The prepared fixture starts at twice its freshness
window (tick 64 at the default), with recent units created at that starting tick and old units
created at tick zero. Validate the multiplication for overflow. This avoids negative timestamps;
the fixture's starting clock is explicit in its export. A one-unit condition tests the count threshold.

This fixture verifies weighted choice and side symmetry. It does not independently validate
a response rule whose preference was supplied. Exact probability checks should examine the
integer-weight choice mapping, not rely on a random seed sweep passing a loose band.

### Growing burrow fixture

Demonstration defaults: width 41, height 25, exit `(0,12)`, initially open staging cells with
`x = 0..2` and `y = 10..14`, eight workers, no initial spoil, and all remaining cells solid.
Spawn workers in row-major order in staging cells, excluding the exit, respecting capacity two.
The time-zero empty geometry is supplied; all subsequent excavation is recorded as construction.

Freshness window is 32 completed ticks; a unit is recent when its age is less than 32. Relay
distance is three successful loaded moves. The demonstration budget is 512 ticks. These defaults
show behavior and bound cost; they are neither physical calibration nor a registered scientific sweep.

Include a one-cell-wide supplied-corridor fixture for exact transport accounting and blocked
movement, with named length and worker positions constructed directly in tests.

## Comparisons and measurements

Cross two factors: direct versus relay transport, and cue-blind versus cue-responsive selection.
Direct transport never drops internally; relay transport permits intermediate handling. Both
can pick up loose units. Keep initial world, population, observations and work budgets matched.

Primary measurements are completed digs per total worker opportunity and externally disposed
units per opportunity. Report both: excavated volume alone can reward leaving unremoved material.
Also report undelivered material, unit delivery latency, carrier count per unit, handling actions,
successful travel, blocked attempts, waits and per-worker work. Unfinished units are censored
deliveries, not dropped observations or artificially infinite measured latencies.

Track first-choice distributions in the choice rig. In growth, track spatial distribution of
dig events, occupied work sites, distance from exit and final connected open area. Every opened
cell is created from an accessible frontier; connected area may therefore be structurally guaranteed
rather than a treatment achievement. Do not use that invariant as evidence of coordination.

Branching, loops, recognizable chambers and architectural diversity are exploratory until an
extraction rule is designed. No visual impression is a registered chamber measurement. Report
controller and route-field computation/memory costs separately from in-world action cost.

For later judged runs, vary corridor length, workforce and relay distance; mirror or rotate
layouts; test cue age/threshold sensitivity; compare outcomes at matched excavation amounts as
well as matched budgets. Registration must fix sample sizes, manifests, claims and uncertainty
methods before execution. This design authorizes no numerical claim from unregistered sweeps.

## Verification and definition of done

- Construct exact worlds for dig conversion, full-hand rejection, pickup/drop/disposal, boundary
  refusal, occupied movement and waits. Verify failed actions leave material and geometry unchanged.
- Trace one unit through multiple workers; verify identity, birth time, transport and disposal.
- Verify freshness exactly at the boundary and both target-weight regimes; verify the blind
  reduction and mirrored choice rig. A response coefficient of one reduces to uniform choice.
- Replay the same seed/configuration and compare full event traces and state fingerprints.
- Verify material conservation after each action, including seeded fixture material, and both
  worker capacity and connection invariants.
- Verify native/WASM parity for a small replay using existing infrastructure.
- Verify CLI validation and I/O errors, exported configuration, scene samples and summary agreement.
- Run relevant core, CLI and parity tests, repository formatting and lint checks. Existing defaults
  and unaffected golden fingerprints must remain unchanged.

Implement with the project's test-first process and subagent-driven development after design
review and a staged implementation plan. This document introduces no runtime changes itself.

## Later extensions and integration

The next studies can add functional tasks, body/substrate variation, self-organized allocation,
local navigation, persistent construction traditions, repairs and cohort inheritance. Verticality,
support, flooding and ventilation each need their own physical assumptions and verification.

An eventual Hornvale integration can translate construction state and history into richer scenes
or consume exported layouts. It must preserve which geometry was supplied and which was built.
The current cottage generator supplies examples of access/placement validation, not an excavation
engine. This first lab does not change either repository's existing Minds, exchange or safety sequence.
