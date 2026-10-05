# F2: fixed-world central-place foraging reconstruction

**Date:** 2026-10-04. **Status:** written spec approved by the user on 2026-10-05. The [implementation plan](../plans/2026-10-05-foraging-2-fixed-world.md) was approved by the user on 2026-10-05; engineering implementation and independent task/whole-branch reviews are complete; local integration into `main` completed at `a194f8e` on 2026-10-05. Scientific execution remains separate.
**Programme:** [F1–F5 foraging/construction sequence](2026-10-04-foraging-construction-design.md), construction B3.
**Evidence:** [F2 historical audit](../../studies/2026-10-04-foraging-fixed-world-reading.md), extending the [F1 audit](../../studies/2026-10-04-foraging-construction-reading.md).
**Starting point:** [merged F1 rule reference](../../foraging.md).

## Intent and success

Build an inspectable, inexpensive fixed-world baseline in which agents search, claim a finite resource, return to a central nest and use private successful-find memory or a nest waypoint server for their next trip. This establishes a measured food-return mechanism before adapting movement to passages and coupling it to burrow construction.

Success means verified state transitions, conserved resources, explicit local information, bounded execution and replayable engineering fixtures. It does not mean reproducing the paper's evolved efficiency, establishing altruism or providing physical cargo mechanics. The first reference is zero-error and uses supplied fixed parameter sets, with no GA, global cluster inference, energy metabolism, excavation, collisions, private channels or browser integration.

This is a paper-based reconstruction informed by the pinned historical iAnt simulator. Its audit records release identity uncertainty and differences from both the paper and later ARGoS.

## Approaches

1. **Recommended: historical discrete world plus F1 decisions.** Retain angular headings, eight-neighbor movement, discrete sensing, delays and nest-return scoring from the inspected historical source. Explicitly preserve the paper/F1 recruitment fallback where source code differs. This minimizes unsupported geometry while exposing adaptations.
2. **Continuous-position reconstruction.** Easier to align with later ARGoS distances, but adds displacement, radius and boundary choices and does not reproduce the original discrete simulator. Defer as a separately named variant if useful.
3. **Food directly in Burrow.** Offers immediate underworld scenes, but mixes navigation and construction changes into the controller baseline. This remains F3/F4.

## Architecture and public boundary

Extend the existing `foraging` module with a distinct fixed-world submodule. Reuse `CpfaParameters`, checked rules and information decisions. Follow the existing Burrow patterns for validated setup, world state, runner and researcher diagnostics; inspect the existing foraging tests, Burrow runner and ants world before planning implementation. Do not refactor those models into a universal mind or world interface.

Separate responsibilities: checked setup/geometry; movement and sampling helpers; resource and agent state; nest server; ordered transitions; bounded runner/output. Small internal types/functions should make transitions testable with supplied variates. The runner uses the existing `rng::SimRng` and seeded construction. No new dependency, ModelKind registration, CLI command, WASM export or UI is included in F2's first implementation.

Provide validated construction from an explicit setup plus seed, one full-tick advancement, and a bounded run returning summary and optional sampled snapshots. Exact Rust type names belong to the implementation plan. Successful construction guarantees valid geometry and capacities. Reject invalid setup with contextual aggregate `FieldError` values before allocating large buffers; do not silently clip coordinates or parameters.

## Setup and budgets

- Rectangular grid: integer width/height from 3 through 125, inclusive; nest is one valid supplied cell. Cells represent the historical grid, with 8 cm as documented scale rather than continuous body geometry.
- Agents: 1–256, stable IDs assigned by setup order; all start at the nest with no cargo, no find memory, no delay and uninformed age zero. Agent positions may coincide.
- Resources: 0–256, explicit unique IDs and distinct valid cells, none at the nest. One cell holds at most one resource. No regrowth.
- All agents use the same validated seven CPFA parameters. Supply them explicitly; there are no claimed evolved defaults.
- Run horizon: 1–7,200 ticks; `agents * horizon <= 1,000,000` opportunities. Validate integer arithmetic before use. Tick count, not wall-clock speed, is authoritative.
- Runner snapshot interval: positive integer; include initial and final state without duplicating a coincident sample. Bound total serialized snapshot bytes at 64 MiB with an explicit error. Full per-opportunity traces are test-only, not an unbounded run output.

First fixtures are hand-specified empty, single-item, neighborhood and competing-agent setups. Layout generation and the paper's clustered/power-law/random evaluation campaign are deferred to a registered protocol. No seed is chosen to make a policy win.

## Geometry, headings and movement

Positions are integer cells. Headings are finite radians in [0,2*pi). Initialize each agent's heading uniformly; choose an uninformed target by uniformly choosing one of four grid edges, then one integer coordinate along it. Corners consequently appear through two edges, matching the historical mechanism. A target may equal the nest if the nest is on an edge; equality transitions to search on the next eligible opportunity.

A turn uses F1's uninformed or informed standard deviation. Sample a normal increment using two independent uniforms and the cosine Box–Muller transform `sqrt(-2*ln(1-u))*cos(2*pi*v)`, multiply by the standard deviation, clip the increment to [-pi,pi], then wrap the heading. This sampling rule and f64 arithmetic are supplied Rust conventions; native source consumes an additional sine/cosine choice and uses floats. A zero standard deviation produces zero increment but follows the same declared draw schedule.

Each turn adds `floor(abs(increment)/(pi/4+0.001))+1` delay ticks. This explicit floating absolute-value convention resolves the native expression's conversion ambiguity; even a zero turn costs one delay tick. Informed age starts at zero and increments once per informed turn, including the initial turn on entering search. It does not increment during delays or travel. Overflow is checked, though the run budget prevents it.

Search target is `(round(x+cos(theta)), round(y+sin(theta)))`, using ties away from zero. It can be diagonal; it is not a four-neighbor walk. If the target is outside, redraw a uniform heading until legal, with at most 32 proposals. Failure returns a contextual execution error. Do not install an arbitrary fallback direction or silently continue. A failed tick commits neither state nor RNG consumption; prepare the tick on cloned working state and commit only on success.

Directed travel and return consider legal neighbors in order `dx=-1..1`, then `dy=-1..1`, excluding (0,0). If a neighbor equals the target, move there directly. Otherwise weight neighbors by positive reduction in Euclidean distance to the target and select with half-open cumulative intervals. No positive improvement is an invariant error, not a wait. Travel does not rotate the stored search heading. Collision and exclusion are absent, matching the parsimonious reference rather than a physical robot body.

## Tick order and agent transitions

A tick processes agents in ascending stable ID order, exposing earlier pickups/publications to later agents in that tick. This deliberate sequential convention matches the inspected source's array iteration and can bias competitive access; report it with results. No scheduler randomization is hidden.

At initialization choose each heading and edge target. At every agent opportunity:

1. If delay is positive, decrement it and end the opportunity. No search decision, movement, detection or nest processing occurs.
2. **Departing:** if uninformed, test `p_search` before movement. An informed departure ignores this switch. If switching or already at the target, enter search, perform one turn, then end the opportunity. Otherwise take one directed travel step; target arrival is recognized at the next eligible departure opportunity.
3. **Searching:** test `p_return` first. On success clear any find, set the nest target, enter empty return and end the opportunity. Otherwise form a legal heading-based target and move one cell. Perform a turn, then inspect the single rounded cell ahead under the new heading. An out-of-grid detection cell observes nothing and does not trigger another boundary redraw.
4. If that detection cell contains an available resource, claim it once, freeze the local find, install a nine-tick survey delay (replacing the turn delay), target the nest and enter loaded return. Pickup is a modeled one-item token assignment; the agent remains in its current cell and does not have to occupy the observed resource cell.
5. **Returning:** move one directed step toward the nest if not already there. On exact nest-cell arrival, deposit any token and process publication/departure in that same opportunity. Reset informed age and set state to departing. No movement on the new trip occurs until the next eligible opportunity.

Nine waiting opportunities follow a successful pickup before return travel. No separate biological survey mechanism is inferred from that delay. The convention follows historical state timing and is not later ARGoS's disabled four-second wait.

Ticks are numbered 0 through horizon-1; snapshots identify completed ticks. Event times use the processing tick. Always execute the requested horizon, including after exhaustion; summaries distinguish depletion from delivery completion. Half-second physical interpretation is a source-based inference, not a license to rescale all parameter rates.

## Resources and observation contract

Every resource has exactly one state: available at its initial cell, assigned to one returning agent, or delivered to the nest. Pickup removes availability immediately. Delivery increments only on nest arrival and cannot occur twice. Every committed tick satisfies:

`initial resources = available + assigned + delivered`.

Agent capacity is exactly one token. There is no consumption, spoil transformation, drop action, cargo mass or paid pickup/deposit beyond the defined opportunities. Assigned-at-horizon resources are censored unfinished returns, not delivered fitness.

The successful observation site is the **found resource's cell**, even though the agent stands one cell behind. Density is one for the claimed resource plus currently available resources in its eight-cell Moore neighborhood. The center is not counted twice, boundaries clip the neighborhood, and earlier agents' claimed resources are unavailable. Freeze this count at pickup; do not refresh it using hidden global truth on return.

The controller knows its own position, heading, nest, selected target and its last successful site/count for this trip. It detects only the defined forward cell and immediate successful-find neighborhood. Researcher summaries can inspect all resources, but controller decisions cannot use that view. There is no global food-location guidance.

## Nest server and private memory

On a loaded arrival, deposit first and call F1 publication with the successful find. A positive request appends a new record at strength one, using a monotonically increasing unique u64 identity and creation tick. Then construct the departure-time server snapshot; the new record is visible to its publisher and later agents in this tick. Two records may share a site. No merge, refresh, deletion on depletion, agent broadcast or remote server query occurs.

Evaluate record strength from its original creation tick with F1 `waypoint_strength`; age is elapsed **ticks**. Remove records with strength below 0.001, retaining equality (`PaperBelow`). Iterate remaining records in ID order and use strength-weighted selection. F1's existing enum spelling `LaterArgosStrengthWeighted` is retained compatibly; the audit establishes support in historical iAnt-Sim as well.

Successful-find fidelity takes priority, then server recruitment, then uninformed edge targeting. Fidelity uses F1's independent draw and frozen count. Empty return supplies no valid find and can recruit normally; this deliberately follows the paper/F1 instead of the historical source's flag that can suppress recruitment without a found tag. Once the departure choice is made, clear the successful-find observation. Its chosen target persists; the old count cannot be published again after a failed search.

Server records are bounded by initial resources, at most 256: one possible publication per resource, occurring only on its sole completed return. Thus no arbitrary extra cap or eviction rule is needed. Information storage and transfer carry no additional energy/time cost in F2. Stale sites remain eligible until decay, and can waste search effort; the server never checks resource availability to improve advice.

## Randomness, replay and failure

Use one sequential seeded RNG stream, stable agent/record/neighbor order and deterministic state transitions. Initialization draws heading, edge and edge coordinate for each agent in ID order. State-specific draw schedule is explicit: eligible uninformed switch; eligible search give-up; boundary proposals only if needed; two draws per performed normal turn; neighbor selection only when not adjacent to target; at each nest arrival independent publication, fidelity and recruitment draws passed to F1 (all three drawn even for empty return); edge/coordinate draws only for uninformed departure. Delay opportunities draw nothing. A successful run is reproducible with the same setup, seed, version and supported platform; no native-source or cross-platform floating-point trajectory identity is claimed.

Tests inject variates into small movement/transition helpers to verify behavior without finding convenient seeds. RNG is owned by the runner/world, never by F1 rules. A failed tick is atomic. Execution/serialization failures must identify tick, agent or budget as appropriate and produce no success summary for a partial run. Counter increments and IDs use checked arithmetic.

## Diagnostics and verification

Return authoritative integer counts: processed opportunities, delay waits, directed/search moves, pickup/delivery, empty returns, search switches, publications, fidelity/recruited/uninformed departures and expired records. Also report resource inventory and per-agent work, first pickup tick, first delivery tick and all-delivered tick as optional values. A missing event is censored at the fixed horizon. Report rates only from these counts and an explicit tick/opportunity denominator; do not call discoveries deliveries.

Acceptance tests must cover:

- Aggregate setup rejection, stable resource identity, duplicate cells, invalid nest, parameter limits and run/output budgets.
- Angular quantization, normal clipping, zero-turn delay, informed age, boundary redraw failure and tick/RNG rollback; finite in-bounds positions after every committed tick.
- Directed neighbor weighting, adjacent target arrival, exact nest arrival and stable tie/interval boundaries using independently calculated small geometries.
- Switch/give-up ordering, no detection on departure, turn-before-forward-detection, nine survey waits, and no decisions during delay.
- Frozen Moore count including pickup, clipped neighborhoods, sequential pickup contention and per-tick conservation; unfinished cargo at cutoff and deposit exactly once.
- Publication before departure, self-visibility, duplicate-site records, strength-weighted selection, threshold equality and expiration; independent publication/fidelity, empty-return recruitment and stale-site pursuit without omniscient correction.
- Repeatable fixed-seed summaries/snapshots, whole-run versus repeated-step equality, initial/final sampling and no state impact from snapshot cadence.
- Hand-controlled empty and single-resource cycles that prove counts and censoring without asserting a preferred policy's efficiency.

Use existing formatting, core lint and workspace checks before implementation review. Tests verify the selected reconstruction, not a claim that source and paper agree everywhere. No statistical efficacy assertion follows from acceptance fixtures.

## Road ahead and review handoff

The approved staged F2 implementation plan is complete. Fresh task reviews and the whole-branch review plus scoped correction review passed; final workspace verification reports 2,463 passed, 0 failed and 103 ignored, with formatting and core clippy clean. The [implementation plan](../plans/2026-10-05-foraging-2-fixed-world.md#execution-and-review-evidence--2026-10-05) preserves commands, review outcomes and execution rulings. Local integration into `main` completed at `a194f8e` on 2026-10-05; fresh merged-tree workspace tests, formatting and core clippy passed with the same test totals. Scientific execution remains separate.

F3 replaces this angular/eight-neighbor geometry with explicit passage navigation and capacity rules. F4 adds paid excavation and separate spoil/food logistics. F5 audits termite shortcut methods and compares registered geometries. Experimental reproduction, error models, generated evaluation layouts and evolved settings each require further source reconciliation and their own protocol.

Self-review checked scope, source provenance, timing units, pickup/deposit conservation, private/public information, failure atomicity, bounded storage and source differences. Written-spec review was approved by the user on 2026-10-05. Implementation-plan review was approved by the user on 2026-10-05; execution uses subagent-driven development.
