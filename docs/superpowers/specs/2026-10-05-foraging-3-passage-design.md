# F3: passage foraging with private learned maps

**Date:** 2026-10-05.
**Status:** conversational choices approved: private learned maps, local observations, coordinate-only recruitment and a multi-cell nest chamber with two-worker capacity throughout. Written architectural spec approved by the user on 2026-10-06. The [implementation plan](../plans/2026-10-06-foraging-3-passage.md) was approved by the user on 2026-10-06; all five stages and independent task/whole-branch reviews are complete; local integration into `main` completed at `c0e551a` on 2026-10-06. Final runtime `693c02b` passed workspace tests (2,563 passed, 0 failed, 103 ignored), formatting and core clippy. Fresh merged-tree verification passed (2,663 passed, 0 failed, 103 ignored), with formatting/core clippy clean. Scientific execution remains separate.
**Programme:** [F1–F5 foraging/construction sequence](2026-10-04-foraging-construction-design.md), construction B3.
**Baseline:** [F2 design](2026-10-04-foraging-2-fixed-world-design.md), merged at `a194f8e`, integration record `6c9a50a`; [public guide](../../foraging.md).
**Related components:** [Burrow 1](2026-10-03-burrow-1-excavation-design.md) and [Burrow resource access](2026-10-04-burrow-2-resource-access-design.md).

## Intent, decisions and success

Adapt central-place foraging to fixed four-neighbor passages before adding construction. Workers discover passage geometry through local observations, collect finite food into one-item hands, and deliver it to a supplied nest chamber. Recruitment can provide a destination without providing a route. The intended foundation is an inspectable delivery mechanism whose navigation knowledge and physical congestion can later interact with excavation.

The user selected private learned maps over a supplied passage map, then approved the proposed local information boundary and multi-cell nest chamber. Those choices are binding. The user approved the complete written spec on 2026-10-06, including the precise exploration policy, handling cadence, parameter subset and diagnostics below. These remain supplied engineering conventions, not biological provenance.

Success means tested information boundaries, legal routes through privately known geometry, occupancy and food conservation, separate collection/delivery measurements, atomic ticks and bounded replay. An unsuccessful or stalled run remains a valid bounded outcome. No scientific evaluation, tuned layout generator, quantitative reproduction or excavation is included.

## Alternatives and scope

1. **Separate passage-foraging world with private maps, selected.** Keeps F2 available as the historical reconstruction and makes each adaptation explicit. Reuse checked F1 information rules and existing implementation patterns without changing F2 trajectories.
2. **Fixed topology supplied to workers.** Simpler routing benchmark, but removes the selected navigation-learning question. It is not an additional policy in this increment.
3. **Food directly in Burrow.** Reuses construction transactions but couples navigation, digging and spoil transport immediately. Defer that composition to F4.

F3 uses fixed open/solid geometry, food, workers and nest metadata. F4 separately specifies excavation costs, shared hands, loaded priority, task switching and distinct spoil/food destinations. Existing Burrow and F2 behavior stays unchanged. No CLI, WASM export, browser UI, ModelKind registration, universal world abstraction or new dependency is required here.

## Architecture and ownership

Add a distinct passage submodule alongside `foraging::fixed`. Separate checked setup, local observations, private map/routing, food ledger, nest server, agent/controller, atomic world and bounded runner/views. Exact Rust names and implementation staging belong to the subsequent plan.

The controller receives an owned local observation, its own state/map, supplied nest coordinates and its selected destination. It never receives the full open mask, resource list, another worker's map or researcher route measurements. The physical action layer validates legality against authoritative state. Observation construction can inspect that state but emits only the specified local view.

Reuse F1 publication, departure, Poisson and waypoint-strength calculations through a narrow adapter. F2's internal helpers currently depend on its single-cell nest and angular setup; do not make them public or generalize F2 merely to obtain reuse. Prefer small passage-owned code over a universal interface. Any shared pure helper extracted later must preserve existing behavior with focused regression evidence.

## Setup, geometry and limits

- Grid dimensions remain 3–125 in each direction. Supplied distinct open cells form a fixed four-neighbor graph; all other in-bounds cells are solid. No wraparound, diagonals, digging or moving through solid cells.
- Nest contains at least two distinct open cells in one four-neighbor-connected chamber. Every nest cell accepts delivery and provides access to the same nest server. Chamber size is supplied geometry; it is not generated or enlarged by the simulator.
- Supply 1–256 worker spawn coordinates, all inside the nest. IDs follow spawn-list order. At most two workers occupy any cell, including nest cells. Duplicate spawns are allowed up to that limit; capacity must accommodate the population without an off-world queue.
- Supply 0–256 food resources with arbitrary unique `u64` identities and distinct open cells outside the nest. Food has no regrowth or consumption. A resource may lie in an open component disconnected from the nest; it then remains physically inaccessible in F3. Setup validation must not require all resources to be reachable.
- Validate dimensions, list lengths, indexed duplicates/coordinates, open-cell membership, nest connectivity, spawn occupancy and numeric parameters before constructing dense authoritative arrays. Empty open lists and invalid masks produce contextual aggregate errors, not clipping.
- Retain 1–7,200 ticks and at most 1,000,000 worker opportunities, including single-step cumulative limits. Retain positive sampling intervals and a 64 MiB total compact snapshot-JSON budget, excluding enclosing episode fields and delimiters.
- Each private map has at most `width * height` entries. At maximum setup, at most 4,000,000 cell classifications exist across all worker maps. Store one classification per cell without an observation history. Route scratch storage is bounded by the same grid size and discarded/reused between calls.

Grid cells, action opportunities and topology are engineering units. F2's historical 8 cm grid and inferred half-second timing are not imported as F3 calibration.

## Local observations and private knowledge

At construction, each worker receives one ordinary observation at its spawn. Before each later opportunity, observe the current cell and its in-bounds north, south, east and west neighbors. Each listed cell supplies its open/solid classification. Open cells additionally supply current occupant count and available-food presence. There is no sight beyond those cells or through solid cells, and no diagonal density lookup.

Each worker starts with an otherwise Unknown map and updates only classifications in its own observations. KnownOpen and KnownSolid persist because F3 topology is fixed; a conflicting later classification is an invariant error. Out-of-bounds positions are boundary constraints, not map entries. Observations do not consume RNG or a separate physical action opportunity; they are counted as computational work, including the construction-time observations and classifications.

Occupancy and food availability are ephemeral local data. Do not store them as enduring topology or refresh them from researcher state. Apart from its topology map and physical state, the worker retains its current successful find, chosen departure site and an optional private frontier travel target. Frontier targets are navigation commitments, not remembered food availability. A coordinate received from recruitment does not mark that cell KnownOpen, populate intervening cells or import the publisher's route.

The supplied nest-coordinate set identifies home and legal delivery destinations. It does not prepopulate nest geometry in the private map. Initially observed cells remain connected to the worker's spawn; subsequent legal movement and observations preserve a known path back to at least one nest cell.

## Routing and exploration

A private frontier is a KnownOpen cell with at least one in-bounds Unknown four-neighbor. Frontier choice and route search inspect only that worker's map. A frontier may already be known open without having been occupied; entering it reveals its immediate neighborhood.

Compute shortest routes on KnownOpen cells, ignoring occupancy except at the immediate next step. Enumerate neighbors north, south, east, west. Among immediate steps that reduce the selected known route distance and currently have fewer than two occupants, select uniformly. If none is legal, wait and retain the destination. Do not use stale or global occupancy to select a route, take a longer detour implicitly, force a swap, displace another worker or treat congestion as an execution error.

**Uninformed departure:** select uniformly among reachable private frontiers and retain the selected frontier during travel. Before each departure movement, test `p_search`; a successful draw, target equality or absence of a frontier enters Searching and consumes the opportunity without movement. Reselect a frontier that has ceased to be a frontier before arrival. No map-edge coordinate is sampled from the full world.

**Searching:** after the give-up and pickup checks described below, retain a valid private frontier or choose uniformly among reachable frontiers. Move toward it using known shortest routes. If already at a selected frontier, the fresh observation resolves it; choose a remaining frontier in that same search opportunity. If none exists, move uniformly among currently unfilled open neighbors, or wait when none is available. Continued wandering does not reveal hidden resources elsewhere or prove global exhaustion to the worker.

**Informed departure:** retain the supplied fidelity/recruitment site. If it is KnownOpen, route to it on the private graph. If it is Unknown, pursue it by selecting a reachable private frontier: rank frontiers first by the minimum Manhattan distance from their Unknown neighbors to the site, then by known route distance from the worker; break exact ties uniformly. Retain that frontier while it remains valid. All frontiers remain candidates, including those that require moving away from the site. Manhattan ranking is explicitly a supplied exploration heuristic, not scent or a navigable route through unknown cells.

On informed-site arrival, enter Searching on the next opportunity. If the site is KnownSolid, or it remains Unknown after all private frontiers are exhausted, abandon that pursuit and enter Searching with an abandonment diagnostic. Do not remove the public waypoint, broadcast failure or inspect global food availability. Informed departure has no give-up draw, matching F2's phase distinction; its exploration and congestion still consume the finite horizon.

**Return:** route to the nearest KnownOpen nest cell in the private map, with all equal-distance home destinations represented in the shortest-route field. Use legal decreasing-distance steps as above. A missing private route home is an invariant failure because each worker started in the nest and learned every successful movement. Occupancy can stall the route; those waits are ordinary censored execution.

Two-worker capacity does not guarantee progress. Sequential moves, full chambers and opposing traffic can cause prolonged waits or deadlock; F3 reports that consequence without adding undocumented priority or escape rules.

## Parameters and explicit adaptations

Supply five active parameters: `p_search`, `p_return`, `lambda_fidelity`, `lambda_publish` and `lambda_waypoint`. Probability domains and rate domains match their F1 validation: probabilities finite in `[0,1]`; fidelity/publication rates finite in `[0,256]`; waypoint decay finite and nonnegative. No evolved defaults are claimed.

Angular `omega` and `lambda_informed` are absent from passage configuration. A narrow F1 information adapter fills those unused fields with zero solely to call the unchanged checked information API; they do not affect passage movement. Do not accept inert angular controls as if they were meaningful F3 parameters.

F3 replaces angular turning, informed angular age, turn delays, edge targets, Euclidean directed travel and nine survey waits with the explicit graph policy above. Exploration memory is a supplied finite-state mechanism. These changes make F3 a documented adaptation rather than another historical CPFA reconstruction.

## Actions, scheduling and tick atomicity

Each tick processes every worker once in ascending ID order, retaining F2's declared sequential bias. Later workers see earlier committed candidate actions in that tick. Every worker gets exactly one physical action: Move, Pickup, Deposit or Wait. State transitions and nest information decisions can accompany that action but never permit a second movement/handling action.

1. Observe locally and update private topology before the decision.
2. **Departing:** perform the applicable departure policy. Entering Searching consumes a Wait with a phase-transition reason; no pickup occurs during departure travel.
3. **Searching:** draw `p_return` before any pickup or movement. On success clear the trip's successful find, enter empty Returning and consume a transition Wait. Otherwise, if available food occupies the current cell, perform Pickup, freeze the find, enter loaded Returning and end the opportunity. If no food is present, perform the exploration move or wait.
4. **Returning outside the nest:** perform at most one move or congestion wait. Entering a nest cell does not deposit or choose the next trip in that same opportunity.
5. **Returning in the nest:** if loaded, Deposit; if empty, Wait with an empty-arrival reason. Process publication/departure in that opportunity, clear cargo/find, and enter Departing. The next trip starts moving no earlier than the next opportunity.

Pickup requires occupying the resource cell with empty hands; it cannot collect remotely from an adjacent cell. Deposit requires carrying the assigned token while occupying a nest cell. Each successful action and each blocked/waiting opportunity is included in authoritative counters. Transient congestion is a wait; illegal geometry, duplicate ownership, arithmetic overflow and inconsistent maps are contextual errors.

Prepare a complete tick on candidate world state and RNG, then commit both only after all actions and invariant checks succeed. A late worker's failure rolls back earlier observations, maps, food assignments, deliveries, server decisions and RNG consumption. Failed runs return an error without a successful partial episode. No scientific run is implied by engineering fixtures.

## Food conservation and successful finds

Every food identity is Available at its original cell, Carried by exactly one worker, or Delivered. The worker cargo and food owner must agree bijectively. For every committed tick:

`initial food = available food + carried food + delivered food`.

Hands hold at most one token. Moving carries the token without changing its original resource coordinate. No drop, relay, food extraction from solid cells, paid mass/energy model or transformation into spoil exists in F3. At horizon cutoff, carried tokens are unfinished returns, not deliveries.

A successful find records the occupied pickup cell and the available-food count in the fresh current-cell/four-neighbor observation immediately before assignment, including the picked-up token exactly once. Solid cells and diagonals contribute nothing. Freeze the count; later agents' observations reflect earlier pickups, but the publisher's count is not refreshed on return. This replaces F2's Moore-neighborhood density with an explicitly local passage observation.

Only Deposit scores delivery. Record optional first pickup, first delivery and all-delivered processing ticks, preserving missing events as censored. Empty worlds have no fictitious completion milestone.

## Nest advice and information lifecycle

All nest cells access one logical server; there is no remote query, map transfer, route broadcast or pheromone field. Server access is available only during returning-in-nest processing, not every time an exploring worker crosses the chamber.

On loaded arrival, deposit first; evaluate F1 publication with the frozen successful find; append any publication at strength one; then evaluate F1 departure with private fidelity first, strength-weighted recruitment second, and uninformed departure otherwise. Empty arrivals have no successful find and can recruit normally. Supply independent publication, fidelity and recruitment uniforms on every nest arrival, including empty arrivals and decisions where a later variate will not affect the result.

Records have stable monotonic identities, creation processing ticks and site coordinates. Allow duplicate sites. Evaluate decay in elapsed ticks; expire strength below `0.001`, retaining equality. Expiration is lazy at nest-arrival server processing; researcher waypoint views compute current strengths without expiring records or mutating counters. Record expired-record totals separately. Records are bounded by initial food count because a resource can publish only on its sole deposit. Preserve self-visibility and later-worker visibility after publication. No omniscient depletion filtering or refresh occurs.

After selecting the next trip, clear the find; retain the selected informed site only for its pursuit. Private topology persists across trips. Uninformed frontier selection occurs during the subsequent departure opportunity, avoiding hidden map-transfer or initialization draws.

## Draws, errors and bounded researcher views

Use one existing sequential seeded RNG stream. Uniform choices use stable ordered candidates and checked variates in `[0,1)` with half-open intervals. A discrete uniform selection consumes one draw even for a singleton; an empty candidate set consumes none. For uninformed departure, test `p_search` first; after a failed switch, recognize target equality, then validate/reselect a frontier before movement. Switching or recognizing equality consumes no frontier/route choice draw. For informed departure, recognize site equality before selecting a route/frontier. Entering Searching clears the departure/frontier travel commitment; entering Returning clears those targets but preserves topology. Phase draws occur only in the phase specified above. Geometry, BFS, observations, snapshots and summaries consume no RNG. The implementation plan must provide a complete per-phase draw table and deterministic scripted tests.

Expose checked construction, atomic stepping, a bounded runner, authoritative summaries and optional sampled snapshots following F2's public shape. Include normalized setup and active parameters with seed in the passage episode so its replay inputs are self-contained. Replay requires the same implementation and supported platform; no cross-platform floating-point identity is promised.

Snapshots expose geometry, food states, worker positions/cargo/phases, selected destinations, per-worker known-open/known-solid counts and current waypoint views. They do not replicate full private maps into every frame. Provide a separate read-only knowledge view for one validated worker ID, containing sorted known cell classifications only; its size is bounded by the grid. Researcher access to that view cannot change controller state or advice. No saved-state restoration API is included.

Report aggregate counters as checked sums of per-worker counters: opportunities, moves split by departure/search/return and cargo state, pickups, deposits, waits by cause, search entries, empty returns, each departure type, publications and abandoned informed targets. Assert `opportunities = moves + pickups + deposits + waits`. Count one initial uninformed trip per worker and every actual entry into Searching exactly once.

Report computational work separately: observations/cells inspected, classifications first learned, route calls/visited cells/peak queue and frontier scans. Sum computational totals across workers; combine peak queue sizes with a maximum rather than a sum. Those counters do not constitute extra physical action costs. Each run includes initial/final snapshots without duplicates; observational sampling cannot alter trajectories. Storage overflow must fail explicitly. Do not retain an unbounded full action or observation history in ordinary run output.

## Acceptance and review requirements

Use existing Rust testing, formatting, linting and workspace tools after a separately approved implementation plan. Fixtures are hand-controlled engineering cases, not an evaluated treatment matrix or seeds selected to produce a win.

- **Setup:** reject single-cell/solid/duplicate nest chambers, disconnected nest masks, invalid spawn membership/capacity, duplicate resources and invalid parameters; accept disconnected resource pockets and a chamber large enough for the supplied population.
- **Learning:** initial knowledge contains only ordinary spawn observations; unknown/solid cells occlude farther food and passages; occupancy/food is fresh and never frozen into topology. Two workers visiting different branches retain different private maps.
- **Information isolation:** alter unobserved geometry/food outside a worker's view and show its next decision and draws remain unchanged; recruiting a never-visited coordinate reveals no route cells or food truth.
- **Navigation:** a detour requiring increased Manhattan distance still permits private frontier exploration; route home uses a learned path rather than Euclidean improvement or an authoritative exit field; tied routes and targets have stable selection boundaries.
- **Capacity:** two-worker limits apply to all chamber and passage cells; later workers see earlier moves; full next steps wait without forced swaps, hidden global-occupancy routing or target loss. A blocked return at cutoff remains carried food.
- **Handling:** collection requires occupying the food cell; movement into food cannot also pick up; movement into the nest cannot also deposit; pickup and deposit each consume their own opportunities. Local frozen density excludes diagonal/across-wall food.
- **Advice:** publication follows deposit and precedes departure; own record is visible; private maps survive trips without being shared; duplicate/stale sites, exact decay equality and empty-return fallback remain supported.
- **Counters and failures:** every phase transition, initial trip and wait has exact per-worker/aggregate counts; action counts sum to opportunities. A failure after earlier movement/pickup/publication rolls back maps, physical state, counters, server and RNG.
- **Bounds and replay:** exact legal tick/opportunity and output boundaries; map/scratch limits; same-seed step/run equality; sampling/knowledge views do not change state; unfinished/unreachable food and missing milestones are represented honestly. Preserve all F1/F2 and Burrow regression behavior.

Self-review completed against the F1 parameter/information API, F2 controller/ledger/runner and Burrow observation/actions/routing patterns. It checked information leaks, topology/occupancy separation, chamber initialization, disconnected food, one-action cadence, conservation, map/storage bounds and parameters with real consumers. Corrections made during self-review require at least two nest cells, count initial observations, keep researcher waypoint views observational, combine queue peaks with a maximum, and specify transition/choice ordering. The spec contains no placeholders; local Markdown link targets resolve. These are documentation checks, not runtime acceptance. Implementation subsequently received independent task/whole-branch reviews under the user's subagent-driven preference; the plan preserves runtime verification and review evidence.

## Handoff and next increment

This spec was approved on 2026-10-06 on `foraging-3-design`, isolated from main and other campaigns. The [five-stage implementation plan](../plans/2026-10-06-foraging-3-passage.md) was approved by the user on 2026-10-06. Engineering execution and independent task/whole-branch reviews are complete at runtime `693c02b`; the plan preserves the final verification and decisions. Local integration into `main` completed at `c0e551a` on 2026-10-06; fresh merged-tree tests, formatting and core clippy passed. Scientific evaluation and construction coupling remain separate.

F4 must extend the fixed-map assumptions deliberately: excavation can turn Unknown/KnownSolid into open cells, food and spoil must retain separate ledgers/destinations, and their competition for hands and paid actions needs explicit rules. F3's immutable topology knowledge must not be silently reused as a correct dynamic-map model.
