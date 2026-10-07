# F4: shared-worker construction and food return

**Date:** 2026-10-06.
**Status:** conversational architecture approved: one worker kind can forage and excavate, hands hold one food or spoil token, loaded workers finish transport first, food and spoil have separate destinations, and private maps revise locally observed openings. The user explicitly deferred role allocation and relay transport. The user approved the complete written spec on 2026-10-06. The [implementation plan](../plans/2026-10-06-foraging-4-construction.md) was approved by the user on 2026-10-06; engineering execution is starting.
**Programme:** [F1–F5 foraging/construction sequence](2026-10-04-foraging-construction-design.md), construction B3.
**Baselines:** [F3 passage foraging](2026-10-05-foraging-3-passage-design.md), merged at `c0e551a` with verified integration recorded at `852fcbc`; [Burrow excavation](2026-10-03-burrow-1-excavation-design.md) and [structural access](2026-10-04-burrow-2-resource-access-design.md).

## Intent and binding choices

Make construction serve a measured function: workers can open access to food, collect finite tokens and return them to the nest while paying for excavation and spoil removal. This follows verified structural access and fixed-passage delivery. Opening a route alone is not food delivery; carrying spoil competes with collecting food for the same hands, movement opportunities and cell capacity.

The user's approved architecture supplies shared workers, direct transport, one shared carrying slot, cargo priority, separate destinations and local dynamic-map updates. Dedicated worker roles, role-allocation rules, relay handoffs, loose spoil and spoil piles remain future increments. The user approved the detailed exploration-first policy, interruption/resumption rules, hidden-food states, waste routing and diagnostics in this written spec on 2026-10-06. These remain supplied engineering conventions, not biological calibration.

Success means legal excavation, conservation of both materials, meaningful separation of access/exposure/collection/delivery, locally justified navigation and knowledge changes, complete atomic ticks and bounded reproducible output. Useful, harmful and negligible construction outcomes are all possible. No scientific treatment run, efficacy prediction, evolved settings or biological calibration follows from acceptance fixtures.

## Approaches and selected boundary

1. **Separate construction-foraging world with shared workers, selected.** Combine Burrow's one-cell excavation/spoil transaction conventions with F3's private navigation, capacity, food and nest advice. Preserve both baselines and expose each supplied adaptation.
2. **Assigned builders and foragers.** Adds population/role assignment and coordination policies before the shared-hands competition is understood. Explicitly deferred by the user.
3. **Relay transport and spoil piles.** Adds dropping, pickup, provenance, task arbitration and possible obstruction choices. Explicitly deferred by the user.

Add a distinct construction submodule alongside `foraging::fixed` and `foraging::passage`. Separate mutable terrain, private knowledge/navigation, typed cargo and ledgers, decisions/actions, ordered atomic world, bounded views/runner and researcher milestones. Reuse existing F1 information mathematics; study F3 and Burrow conventions. Exact Rust names and staging belong to the later plan. Do not turn their private types into a universal world/mind interface or change their existing trajectories/API.

No CLI, WASM, browser UI, ModelKind registration, new dependency, signal diffusion, global destination map, desired tunnel blueprint or experimental treatment harness is included. This is a core engineering increment.

## Setup, destinations and limits

- Keep F3 dimensions 3–125 per axis, 1–256 workers, at most 256 food tokens, 1–7,200 ticks and at most 1,000,000 worker opportunities. Validate requested and cumulative limits; run every requested tick.
- Supply initial open cells and an immutable diggable mask as distinct bounded coordinate lists. A mask entry may already be open, allowing compatible substrate metadata in pre-opened controls; only currently solid masked cells are excavatable. Every other solid cell is protected. Geometry changes only Solid to Open and never refills/collapses.
- Keep a connected nest chamber with at least two open cells, explicit spawn-list identities and at most two workers per cell. All workers spawn in the nest with empty hands and the same explicit policy parameters. No off-world queue or roles by ID.
- Supply one initially open waste outlet coordinate outside the nest. It must connect to the nest through initially open cells. It has the same two-worker capacity as every cell and remains open. Construction can cause congestion there; no priority override or automatic expansion is supplied.
- Supply unique arbitrary u64 food identities at distinct in-bounds cells outside both nest and waste outlet. Food on initially solid cells is Hidden; food on initially open cells is Available. Protected buried food and disconnected exposed pockets are valid inaccessible cases. Do not give their locations or count to workers.
- Resource coordinates always denote original food cells, including while carried/delivered. No food regrowth, consumption, ownership policy or spoil transformation exists.
- Digging creates at most one spoil token per opened cell. Bound the spoil ledger by the number of initially solid diggable cells, at most 15,625. No initial spoil or loose-spoil state is included in this first direct-transport slice.
- Private knowledge and routing scratch are each bounded by grid size, at most 4,000,000 classifications over all workers. Per-token records are fixed-size; no repeated carrier/observation/action history is retained.
- Keep positive snapshot intervals, initial/final samples once and a 64 MiB sum of compact snapshot JSON bytes, counting all snapshot fields but excluding episode/configuration fields and inter-frame delimiters. Large scenes can exhaust recording before their action budget; fail explicitly without a successful partial episode. This is not a total-process-memory or throughput guarantee.

Aggregate indexed errors on original dimensions, list lengths, duplicates, masks, destinations, spawns, food and parameters before allocating dense terrain/maps. Normalize geometry by coordinate and food by identity; preserve worker order. Do not silently deduplicate, clip or repair inputs. Waste connectivity is a constructor validation, not a route field supplied to workers.

## Typed hands and independent material accounting

Hands are Empty, Food(id) or Spoil(id), with exactly one token at a time. Food and spoil IDs occupy separate namespaces and may have the same numeric value. No sentinel ID denotes empty hands. Every carried token agrees bijectively with its worker's tagged cargo.

| Material | States | Legal terminal destination | Conservation after each committed tick |
|---|---|---|---|
| Food | Hidden, Available, Carried(worker), Delivered | Any nest cell, with food cargo | initial = hidden + available + carried + delivered |
| Spoil | Carried(worker), Disposed | Waste outlet, with spoil cargo | excavated cells = carried spoil + disposed spoil |
| Terrain | Initially open, remaining solid, excavated open | No material destination | current open cells = initial open cells + excavated cells |

One successful Dig changes one eligible cell to open, creates one uniquely identified spoil token in the digger's hands, and exposes any Hidden food at that cell. Those effects commit together. The digger stays in its current cell; Dig cannot also move, collect food or dispose. Food exposure creates availability, not cargo or delivery.

PickupFood requires empty hands and occupying the Available food cell. DepositFood requires food cargo and occupying a nest cell. DisposeSpoil requires spoil cargo and occupying the waste outlet. Each consumes its own action opportunity. Food cargo at the waste outlet and spoil cargo at the nest cannot use the other material's terminal transaction. Crossing either destination while carrying the other material has no special handling or advice effect.

No digging while carrying, dropping, relaying, food sharing or mid-load switching exists. Deadline cargo remains carried and unfinished; it is never silently discarded to end a run.

## Observations and dynamic private knowledge

Retain ordinary construction-time spawn observations and fresh observations before each opportunity. Workers sense only their current cell and in-bounds cardinal neighbors. Open cells reveal current occupancy and Available food presence; solid cells reveal whether they are diggable. Solid cells reveal no Hidden food. There is no diagonal sensing or view through a face. Occupancy/food is ephemeral; only terrain classification and observed solid-face diggability persist.

Each worker knows grid bounds, its own position/cargo/food intent, nest coordinates and the waste outlet coordinate as supplied destination labels. These labels do not populate a route or mark unseen cells open. Researchers see complete terrain/material truth, but their connectivity fields and resource lists never become decision inputs.

Private terrain can be Unknown, KnownOpen, or KnownSolid with observed diggability. Unknown becomes an observed classification; a previously KnownSolid diggable cell can become KnownOpen through a fresh local observation. KnownOpen never becomes solid. Protected KnownSolid cannot legally open, and observed solid-face diggability cannot change. Unexpected reversals or contradictory substrate observations are contextual invariant errors.

A successful own Dig confirms its target cell open and updates only the digger's classification in the same candidate action. It supplies no hidden food list or newly exposed food identity to the decision layer. Other workers keep their previous wall classification until their own observation revises it. No global invalidation, map sharing or publication of construction geometry occurs.

Dynamic invariants must allow a worker's stale KnownSolid diggable entry to correspond to an authoritative open cell. The F3 invariant equating every stored solid classification with present truth is inappropriate here. KnownOpen must remain physically open; protected walls remain solid. Validate local observation batches before mutation and roll back both first-learning and revision counts with the complete tick.

Count first classifications separately from KnownSolid-to-KnownOpen revisions and own-dig confirmations. A revision changes known-open/solid counts but does not count the same cell as newly learned twice.

## Private routes and retained destinations

Keep F3 shortest routes on privately KnownOpen cells and north/south/east/west next-step order. Filter only immediate decreasing-distance steps using fresh occupancy; select uniformly among legal choices. Blocked shortest steps wait with the destination retained. Do not introduce global occupancy-aware detours, swaps, displacement or loaded-worker scheduling priority.

Food return routes to the nearest privately known nest cell. Its existence follows from spawning in the nest and learning every legal movement; a missing home route is an invariant error.

Spoil transport targets the waste outlet. If it is Unknown, use F3-style private frontier exploration ranked by minimum Manhattan distance from unknown frontier neighbors to the supplied outlet, then known route distance; keep nonimproving frontiers eligible. When it becomes KnownOpen, use its private shortest route. No digging or food collection is allowed while hauling spoil. Constructor connectivity guarantees an initial physical route, but does not give workers its geometry. Congestion or unfinished exploration can censor disposal at cutoff; no shared exit-distance field is imported from Burrow.

A known solid waste outlet, or an unknown outlet after genuinely exhausting the original connected open region's private frontiers, contradicts the validated destination/learning history and is an invariant error. Merely lacking a known route while the outlet is still Unknown and frontiers remain is ordinary exploration.

## Empty-worker policy: explore before excavating

Keep the five active F3 parameters and their numeric domains: p_search, p_return, lambda_fidelity, lambda_publish and lambda_waypoint. Preserve the F1 zero-filled unused angular adapter; no angular controls, dig probability, role fraction, relay length or spoil-cue response parameter is added.

Food trip intent retains Departing, Searching or Returning plus an optional informed site. Loaded cargo dispatch overrides that intent. Informed destination coordinates can guide private frontier ranking, but are never global food truth.

- **Uninformed Departing:** preserve F3's p_search-before-target-equality ordering and private open-frontier travel. If none remains, enter Searching with a transition wait. Do not combine that transition with Dig.
- **Informed Departing:** equality enters Searching on a transition wait; a KnownOpen site uses its private route. A remembered protected solid site is abandoned. For an Unknown or remembered diggable-solid site, prefer reachable private open frontiers ranked as in F3. If those are exhausted, choose a reachable remembered diggable face, ranking face-to-site Manhattan distance then shortest known distance to an approach cell, with uniform exact ties. Nonimproving faces remain candidates. If no route, frontier or eligible face remains, abandon the pursuit and enter Searching without deleting server advice. A remembered diggable wall is not immediately rejected as in F3: another worker may have opened that published site since the last observation.
- **Searching:** test p_return first, then collect current-cell Available food if still continuing. Otherwise retain/select an open exploration frontier as in F3. Only when no reachable private open frontier remains, retain/select a known diggable face uniformly across eligible faces. If neither exists, wander among legal fresh open neighbors or wait.
- **Empty Returning:** finish the existing empty food trip to the nest and its normal advice processing. Do not start digging on that return.

A known diggable face is a remembered KnownSolid diggable cell with at least one privately KnownOpen approach neighbor reachable from the worker. Select among stable coordinate-ordered faces; route to a nearest known approach using local first-step capacity. Dig only when currently adjacent and a fresh observation confirms the face is still solid/diggable. Retain a valid face through travel/congestion while the no-open-frontier condition holds; a newly observed open frontier clears the face commitment and takes priority. Reselect after observation shows a face opened or otherwise ceased eligibility. Do not filter faces by unobserved food or fresh global terrain.

The no-frontier condition is epistemic: another worker may have opened a wall that this worker has not revisited. Stale-face pursuit can waste travel, and fresh observations can reveal a new open frontier. This policy is supplied engineering behavior, not an optimized biological excavation rule.

## Cargo priority and food-trip resumption

- Food cargo goes to the nest and DepositFood before any new exploration or excavation.
- Spoil cargo goes directly to the waste outlet and DisposeSpoil before any new job.
- Dig suspends the current empty food-trip intent. Keep its Departing/Searching phase and selected informed site, if any, without a nested task queue. Clear short-lived open-frontier/dig-face commitments because the chosen face just changed state.
- After DisposeSpoil, resume that same empty food-trip intent on the next opportunity. Do not count a new food departure, empty food return or search entry merely because spoil was disposed. No food-server access or three arrival draws occur at disposal. Updated private geometry remains available for the resumed trip.
- A spoil hauler may observe food but does not collect it or retain its availability as permanent map knowledge. The resumed policy can revisit an opened cell as an open frontier; success is not forced by remembering global food truth.

This pause/resume rule expresses task switching within one homogeneous worker population. It does not allocate permanent roles or introduce a policy that asks other workers for assistance. Loaded-job priority does not overrule two-worker cell capacity or ascending-ID scheduling.

## Food advice and cadence

Retain F3 food-nest publication/fidelity/recruitment and independent arrival draws. Only a completed DepositFood provides a successful frozen find for publication. Neither Dig, food exposure, DisposeSpoil nor crossing the nest while hauling spoil publishes advice.

Freeze food density at PickupFood from Available food in the current/cardinal open-cell observation, including the collected token once. Hidden food and spoil never contribute. Deposit first, publication next, then departure with private fidelity before strength-weighted recruitment and uninformed fallback. Duplicate and stale sites remain supported; expiry is lazy at food-nest arrivals, with literal 0.001 retained. Advice shares coordinates/strength, never construction routes or maps.

Waypoint age is elapsed ticks, including time spent digging/hauling spoil. A chosen site can remain in the suspended food intent while its public record decays; workers do not remotely refresh advice. Clear successful food finds after food arrival so empty food returns cannot republish them.

Each tick processes ascending spawn-list IDs on one sequential seeded stream. Each worker performs one Move, Dig, PickupFood, DepositFood, DisposeSpoil or Wait. Movement into food/nest/outlet cannot also handle cargo. Constructor observations, topology revisions, target selection and advice are computational/state work accompanying that action, not additional unreported physical actions. State transitions consume waits where F3 does.

The implementation plan must specify a complete per-mode draw table. Follow F3 nonempty discrete selections drawing once even for singletons, empty selections drawing none, and no RNG for sensing, geometry, views, milestones or serialization. Food arrival consumes the existing independent three draws; spoil disposal consumes none.

## Atomicity, bounds and public output

Prepare the complete tick on cloned candidate world and RNG. A late failure rolls back terrain, Hidden-to-Available changes, both ledgers, cargo/resume intent, all maps/revisions, advice, milestones and counters. No committed partial tick or successful partial run is returned. Conflicting knowledge or illegal transactions fail with tick/worker/material context; normal congestion remains a counted wait.

Provide checked construction, stepping, bounded run, summary, snapshot and single-worker read-only knowledge views following F3. Episode contains normalized setup, five parameters, seed and run options for replay. Restoration/deserialization, external CLI/WASM/browser exposure and cross-platform trajectory identity remain outside the increment. Knowledge views expose classifications/diggability only; snapshots expose small knowledge counts and full researcher terrain/material state, not every map or an unbounded trace.

Use indexed authoritative membership for invariant checks and bounded map/route structures, keeping indexes outside workers' policy inputs. Do not repeat the F3 linear full-open-list membership issue. The runtime need not cache or broadcast global distance fields.

## Measurements: access is not delivery

Report checked aggregate/per-worker physical counters whose Move + Dig + PickupFood + DepositFood + DisposeSpoil + Wait sum equals opportunities. Split movement/cargo and congestion by food, spoil and empty purpose; retain F3 food-trip counters, and add excavation, spoil-haul starts and disposal counts. Computational observations, first classifications, revisions, route work/queue maximum and face scans remain distinct from paid actions.

Report final food Hidden/Available/Carried/Delivered and spoil Carried/Disposed inventories, excavation and open-cell counts, plus fixed-size per-spoil identity/origin/birth/owner/disposal data. Missing disposal at cutoff is censored, not a dropped unit. No relay carrier history is invented.

Retain F3 optional first pickup/delivery/all-food-delivered processing ticks. Add optional first excavation, first new food exposure, first newly established food access and first spoil disposal. Empty material sets create no fictitious completion events.

Researchers compute physical food access through current open-cell connectivity from the nest, ignoring temporary occupancy and private knowledge. Record initially exposed and initially accessible food counts separately, plus one bounded access record per food token indicating initial exposure/access and optional first new exposure/access milestones. Initially available disconnected food is exposed but not accessible. Initially hidden food opens into availability when excavated; a later corridor can also connect an already-exposed pocket.

Evaluate first new access after each successful Dig, using one multi-source nest BFS over physical open cells for all food records, without broadcasting success. Count this researcher connectivity computation separately from worker route work; do not recompute one global field per resource. Milestones include processing tick, one-based committed opportunity index, responsible worker, excavation/disposal work and current food delivered count; route distance is shortest physical open path to any nest cell, not Manhattan rank. Pickup and delivery remain separate later transactions. A first-access event does not establish that a worker learned or used that route.

Snapshots count completed ticks; events use processing ticks starting at zero. Initial exposure/access flags are not fabricated tick-zero actions. The full requested budget continues after every milestone and after all food has been delivered.

## Acceptance fixtures and review focus

Use deterministic engineering fixtures and supplied draws. No seeds/layouts are chosen to claim a policy wins; no scientific evaluation harness runs as part of acceptance.

- **Setup/destinations:** reject invalid/duplicate masks, solid outlet whether protected or diggable, outlet in nest or disconnected from the initial nest component, excess occupancy and duplicate/out-of-bounds food. Accept protected buried food and disconnected exposed pockets; keep food/spoil numeric IDs distinct by tag, including maximal food ID.
- **Two ledgers/hands:** each Dig creates exactly one carried spoil and one opened cell; wrong-kind/wrong-destination handling, double deposit/disposal and digging with full hands fail atomically. Verify both conservation equations and open-count relation after every committed tick.
- **Occlusion/exposure:** changing Hidden food behind an observed solid face cannot change the worker's decision/draws. Dig exposes without collecting/scoring, and a later worker's fresh observation can see the new availability. Density excludes hidden food, diagonals and spoil.
- **Dynamic privacy:** the digger learns its own opening; remote workers retain stale walls until observation. Legal KnownSolid-to-KnownOpen revisions count once without re-counting first learning. Protected-wall/open-to-solid conflicts error; current stale diggable walls are valid stored beliefs.
- **Explore-before-dig:** use open frontiers before face selection, preserve uninformed switch and search give-up order, choose only privately known eligible faces, preserve face travel through congestion and skip a remotely opened face after fresh sensing.
- **Cargo priority/resumption:** visible food cannot preempt carried spoil; digging cannot preempt food return. DisposeSpoil resumes the same intent without food publication/arrival draws/counter inflation. Informed site remains stored across the interruption.
- **Routes/capacity:** private detours, unknown outlet exploration, no supplied global exit field, all-cell two-worker limits, equal nearest nest steps and deadlocks/cutoff cargo. Waste congestion neither expands the outlet nor silently abandons material.
- **Actual benefit and censoring:** a scripted single buried-food cycle proves paid Dig, spoil transport/disposal, later food pickup and nest delivery as distinct actions. An exposed-but-disconnected pocket distinguishes exposure from first access; a protected case stays censored. Shortest measured structural access can precede any private-route knowledge or delivery.
- **No-dig reduction:** on compatible scenes with initially exposed food, no diggable solid faces and valid unused outlet metadata, compare F4's food/worker action-state projection and RNG continuation with F3 for the same inputs/seed/horizon. Extra construction observer fields and scan counters need not match F3 bytes. This pins baseline preservation without an undocumented all-configuration equivalence claim.
- **Complete rollback:** late failures after earlier digging/exposure/revision, spoil disposal or food publication restore terrain, ledgers, cargo/intent/maps, milestones/counters and RNG. Test both injected-draw candidate behavior and production seeded continuation.
- **Bounds/views:** exact legal opportunity/tick limits, spoil/map capacities, byte-cap success and one-byte-short failure, initial/final sampling once, step/run equality and observation/knowledge/summary purity. Preserve existing F1/F2/F3/Burrow regressions; no new test disabling or broad dead-code allowances remain at final verification.

After separately approved implementation planning, use the existing Rust test/format/core lint/workspace gates and independent task/whole-branch reviews under the user's subagent-driven preference. Root staging tracker and detailed evidence belong to that execution, not this spec draft.

Spec self-review checked the F3/Burrow source patterns, both conservation equations, cargo tags/destinations, one-action cadence, private stale-wall validity, local revisions, informed pursuit, pause/resume draws, hidden/exposed/access distinctions, normalization, bounds and no-dig reduction. Corrections during self-review make stale diggable-site pursuit explicit, qualify face retention under open-frontier priority, and bound structural evaluation to one researcher BFS per successful Dig. Local Markdown targets and placeholder checks are verified before commit; these are documentation checks rather than runtime acceptance. Planning clarification: the initially open outlet need not be marked diggable; the earlier acceptance wording is corrected to reject solid outlets, whether protected or diggable, matching the explicit setup contract.

## Handoff and deferred work

Conversational architecture and the complete written spec are approved; the implementation plan is approved on isolated `foraging-4-design`, created from current main `e733b98` after concurrent Crowd work advanced it. The foraging/Burrow baseline matches verified integration `852fcbc`; other campaigns were preserved. The [five-stage implementation plan](../plans/2026-10-06-foraging-4-construction.md) was approved on 2026-10-06; engineering execution uses the standing subagent-driven preference. Integration, push and scientific evaluation remain separate.

Future role allocation can vary who chooses food/excavation and how workers coordinate. Future relay transport must add loose material, drop/pickup/handoff costs, history bounds and obstruction choices without conflating food and spoil. Those features are explicitly deferred, not hidden incomplete behavior in the first F4 contract.

F5's shortcut/termite comparison and quantitative efficacy tests still need source reconciliation, controls, registered outcomes and an approved scientific protocol. This increment supplies measured construction and realized food return, not that comparison.
