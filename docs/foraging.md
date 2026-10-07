# CPFA rules (F1), fixed-world reference (F2), passage (F3) and construction (F4)

The top-level public `sugarscape_core::foraging` API verifies stateless rules from Hecker and Moses (2015). Engineering implementation and independent task/whole-branch reviews are complete; this work merged into `main` at `5819a04`. F1 neither simulates foraging nor reproduces evolved performance. It supplies no world, heading sampler, scheduler, food ledger or waypoint server.

See the [source audit](studies/2026-10-04-foraging-construction-reading.md), [approved design](superpowers/specs/2026-10-04-foraging-construction-design.md) and [approved implementation plan](superpowers/plans/2026-10-04-foraging-1-cpfa-rules.md) for provenance and acceptance evidence. The later ARGoS source was inspected, not copied, built or run. Its inspected revision is `18fc0d9813e37bcc01c54ec9896435f1038f4295`; it is not established as the simulator behind the 2015 results.

## Explicit parameters and mathematical contract

All seven parameters must be provided. There are no claimed evolved defaults.

| Parameter | Meaning | Accepted domain |
| --- | --- | --- |
| `p_search` | Probability of switching to search | Finite `[0,1]` |
| `p_return` | Probability of unsuccessful return | Finite `[0,1]` |
| `omega` | Uninformed angular standard deviation, radians | Finite `[0,4*pi]` |
| `lambda_informed` | Decay toward uninformed turning variation | Finite, nonnegative |
| `lambda_fidelity` | Poisson rate for private site fidelity | Finite `[0,256]` |
| `lambda_publish` | Poisson rate for waypoint publication | Finite `[0,256]` |
| `lambda_waypoint` | Waypoint strength decay rate | Finite, nonnegative |

`uninformed_variation` returns `omega`. `informed_variation` returns `omega + (4*pi - omega)*exp(-lambda_informed*age)`. `waypoint_strength` returns `exp(-lambda_waypoint*age)`, initially one. No function samples headings or turns these probabilities into per-second hazards.

`poisson_cdf(c, lambda)` calculates the inclusive lower tail `P(X <= c) = exp(-lambda) * sum(lambda^k / k!, k=0..c)`. The source prose describes an “at least” probability, which conflicts with its displayed equation; F1 follows the equation, also supported by the inspected later source. Decisions use strict `probability > draw`. At count one and rate one the probability is `2/e`, about 0.73576.

Counts are integers in `0..=256`; fidelity/publication rates are bounded by 256. These are supplied reference-utility engineering bounds, not biological limits or the paper's evolutionary initialization range `[0,20]`. Ages must be finite and nonnegative; all random variates must be finite in `[0,1)`; snapshot strengths must be finite in `[0,1]`.

Errors are contextual `Vec<FieldError>` values. Decision validation checks all original parameters, draws and records before choosing an outcome, including inactive records and unused recruitment draws. Fields include `find.count`, `memory.count`, `fidelity_draw`, `recruitment_draw` and indexed `waypoints[i].strength` or `.id`. Duplicate waypoint identities are rejected; sites have no sentinel values. Inputs are never silently clamped or truncated.

## Caller example and information boundary

The following uses public exports and independent publication, fidelity and recruitment variates. These explicit parameters are illustrative.

```rust
use sugarscape_core::foraging::{
    departure, publication, CpfaParameters, Departure, FindRecord,
    Waypoint, WaypointSelection, WaypointThreshold,
};
let parameters = CpfaParameters {
    p_search: 0.5, p_return: 0.5, omega: 1.0,
    lambda_informed: 1.0, lambda_fidelity: 1.0,
    lambda_publish: 1.0, lambda_waypoint: 1.0,
};
let find = Some(FindRecord { site: 77, count: 1 });
let snapshot = [Waypoint { id: 9, site: 88, strength: 0.8 }];
assert_eq!(publication(&parameters, find, 0.8).unwrap(), None);
assert_eq!(departure(&parameters, find, &snapshot,
    WaypointSelection::LaterArgosStrengthWeighted,
    WaypointThreshold::LaterArgosStrict, 0.7, 0.0).unwrap(),
    Departure::SiteFidelity { site: 77 });
assert_eq!(publication(&parameters, find, 0.7).unwrap(), Some(77));
assert_eq!(departure(&parameters, find, &snapshot,
    WaypointSelection::LaterArgosStrengthWeighted,
    WaypointThreshold::LaterArgosStrict, 0.8, 0.0).unwrap(),
    Departure::Recruitment { waypoint: 9, site: 88 });
```

Publication returns a request; it does not insert a record. The caller owns publication order and whether a publisher sees its newly published record in the subsequent snapshot. Departure prioritizes private fidelity, then recruitment, then `Departure::Uninformed`. `publication(&parameters, None, 0.0)` returns no request, while departure may still use separately retained valid memory. The caller decides whether an unsuccessful search invalidates memory.

A site ID identifies a caller's previously observed location. A supplied waypoint snapshot is available information, not global resource truth: recruitment can select an active waypoint whose site is depleted. No resource lookup, hidden RNG draw, memory mutation or snapshot mutation occurs.

## Named source variants

`WaypointSelection::UniformComparison` selects equally among active records and is a supplied comparison rule; the paper does not fully specify server selection. `LaterArgosStrengthWeighted` selects proportionally to current strength, as in the inspected later ARGoS controller. Caller order is stable; tickets use half-open intervals, with equality advancing to the next record.

`WaypointThreshold::PaperBelow` retains strength equal to `0.001`, following the paper's removal-below wording. `LaterArgosStrict` requires strength strictly greater than `0.001`, following the inspected later source. Both discard weaker records. The caller supplies current strength; F1 does not expire or maintain server records.

## F2 source reconciliation

The F2 audit reconciles historical simulator provenance; heading updates and angular bounding; displacement and boundaries; detection area and whether density includes the picked-up item; survey time; decision cadence and give-up events; empty-return memory; nest radius; resource removal and delivery scoring. The approved design fixes publication/server access timing, self-visibility, duplicate sites, capacity and storage costs, labeling supplied reconstruction choices separately from historical source behavior.

F3 passage adaptation, F4 excavation coupling and F5 termite comparison retain separate design gates. Quantitative evolutionary reproduction also requires parameter provenance, evolutionary settings, independent evaluation layouts, outcome definitions and an approved scientific protocol. Passing F1 rule tests establishes none of those scientific outcomes.

## F2 implementation status

The [historical source audit](studies/2026-10-04-foraging-fixed-world-reading.md) recovered a prepublication iAnt-Sim implementation with eight-neighbor angular movement, explicit delays and nest-return scoring. The [approved fixed-world design](superpowers/specs/2026-10-04-foraging-2-fixed-world-design.md) records source conventions and deliberate reconstruction choices; the user approved the written spec on 2026-10-05; the [implementation plan](superpowers/plans/2026-10-05-foraging-2-fixed-world.md) was approved on 2026-10-05. Strength-weighted recruitment is now also verified in the historical simulator, while the existing `LaterArgosStrengthWeighted` API spelling remains unchanged.

F2 engineering and independent task/whole-branch reviews are complete; local integration into `main` completed at `a194f8e` on 2026-10-05. The final counter correction passed scoped review. Fresh feature and merged-tree workspace verification each report 2,463 passed, 0 failed and 103 ignored, with formatting and core clippy clean. The implementation plan preserves commands, rulings and evidence. Scientific execution remains separate; F3 passage adaptation is implemented below.

## Fixed-world public usage

`foraging::fixed` adds validated `World::new`, atomic `step`, observational
`snapshot` and `summary`, and bounded `run`. F1 exports remain compatible.
The parameters below are supplied engineering values, not evolved defaults.

```rust
use sugarscape_core::foraging::{CpfaParameters, fixed::{run, Pos, Resource, Setup, RunOptions}};
let setup = Setup {
    width: 5, height: 5, nest: Pos { x: 2, y: 2 }, agents: 1,
    resources: vec![Resource { id: u64::MAX, pos: Pos { x: 3, y: 2 } }],
    parameters: CpfaParameters {
        p_search: 1.0, p_return: 0.0, omega: 0.0,
        lambda_informed: 0.0, lambda_fidelity: 0.0,
        lambda_publish: 0.0, lambda_waypoint: 0.0,
    },
};
let episode = run(setup, 12, RunOptions { ticks: 20, sample_every: 7, snapshots: true }).unwrap();
assert_eq!(episode.summary.completed_ticks, 20);
assert_eq!(episode.summary.work.opportunities, 20);
assert_eq!(episode.snapshots.first().unwrap().completed_ticks, 0);
assert_eq!(episode.snapshots.last().unwrap().completed_ticks, 20);
```

Grid dimensions are 3–125 cells, agents 1–256, resources 0–256. Resource IDs
are arbitrary unique `u64` values; resource cells must be distinct, valid and
away from the nest. All seven parameters are explicit and use F1 validation.
Runs request 1–7,200 ticks and at most 1,000,000 agent opportunities. Individual
steps enforce the same cumulative bounds. Snapshot intervals must be positive,
even with recording disabled. Initial and final frames are included once;
disabled recording yields no frames and zero bytes. The 64 MiB bound counts
canonical compact JSON bytes for each Snapshot, excluding Episode fields and
inter-frame delimiters. Serialization uses a bounded counting writer before
committing each frame. Errors return no successful partial Episode.

Ticks process stable agent IDs in ascending order, which can bias competitive
access. Events use processing ticks starting at zero; snapshots count completed
ticks. Every requested tick runs even after exhaustion or completed delivery.
Each eligible agent gets one opportunity per tick; delay waits consume an
opportunity without drawing, sensing or moving. Headings and standard deviations
are radians. Informed age counts turns, waypoint age counts ticks. Cells reflect
the historical 8 cm scale; the inferred half-second tick interpretation does not
convert probability parameters into per-second hazards.

Resources are conserved one-item tokens: `initial = available + assigned + delivered`.
Pickup is assignment from the cell ahead after turning; only exact nest return
scores delivery. `ResourceView.resource.pos` always records the original resource
cell; its `state` describes assignment or delivery. Assigned tokens at cutoff are
unfinished returns. Missing pickup/delivery times are censored, including empty
worlds. Work totals are checked sums of per-agent counters; calculate rates with
an explicit tick or opportunity denominator. Researcher views expose all resources,
but controllers only sense the forward cell and a frozen successful Moore-neighborhood
count. Waypoints can remain stale, and their strengths are evaluated observationally
without expiring records or consulting current resource availability.

Replays require identical setup, seed, implementation version and supported
platform. Sampling and summaries consume no RNG; native-source and cross-platform
floating-point trajectory identity are not claimed. Reconstruction deliberately
uses two cosine Box–Muller draws per turn, floating absolute turn delays, bounded
boundary proposals and atomic failed ticks. It follows paper/F1 empty-return
recruitment rather than the historical suppression flag. Successful arrival draws
publication, fidelity and recruitment independently; publication precedes departure,
so the publisher sees its own record. Duplicate sites remain separate records;
strength weighting retains the `0.001` threshold equality. The historical audit also
supports strength weighting; `LaterArgosStrengthWeighted` retains its existing spelling.

F3 separately replaces angular eight-neighbor movement with passage navigation
and capacity. F4 adds paid excavation and separate spoil/food logistics. Error
models, generated evaluation layouts, evolved settings and quantitative reproduction
require further source reconciliation and a registered scientific protocol.

## F3 passage foraging

The [approved passage spec](superpowers/specs/2026-10-05-foraging-3-passage-design.md)
and [implementation plan](superpowers/plans/2026-10-06-foraging-3-passage.md)
were approved on 2026-10-06. Engineering implementation, all five task reviews
and the whole-branch review are complete; local integration into `main` completed at `c0e551a` on 2026-10-06.
Final runtime `693c02b` passed feature workspace tests: 2,563
passed, 0 failed and 103 ignored; formatting and core clippy passed.
Scientific execution remains separate; F4 construction coupling is next.
Fresh merged-tree verification passed: 2,663 tests, 0 failed and 103 ignored,
with formatting and core clippy clean.

`foraging::passage` exposes checked `World::new`, atomic `step`, observational
`summary`, `snapshot`, single-worker `knowledge`, and bounded `run`. This public
example uses explicit engineering values and asserts replay/accounting rather
than assuming a winning trajectory or guaranteed delivery.

```rust
use sugarscape_core::foraging::passage::{run, Parameters, Pos, Resource, RunOptions, Setup, World};
let pos = |x, y| Pos { x, y };
let setup = Setup {
    width: 5, height: 5,
    open: vec![pos(0,0), pos(1,0), pos(2,0), pos(3,0), pos(3,1)],
    nest: vec![pos(0,0), pos(1,0)], workers: vec![pos(0,0)],
    resources: vec![Resource { id: u64::MAX, pos: pos(3,0) }],
    parameters: Parameters {
        p_search: 1.0, p_return: 0.0, lambda_fidelity: 0.0,
        lambda_publish: 0.0, lambda_waypoint: 0.0,
    },
};
let episode = run(setup.clone(), 12,
    RunOptions { ticks: 40, sample_every: 7, snapshots: true }).unwrap();
let mut world = World::new(setup, 12).unwrap();
for _ in 0..40 { world.step().unwrap(); }
assert_eq!(episode.summary, world.summary().unwrap());
assert_eq!(episode.summary.work.opportunities, 40);
let inventory = episode.summary.inventory;
assert_eq!(inventory.initial, inventory.available + inventory.carried + inventory.delivered);
assert_eq!(episode.snapshots.first().unwrap().summary.completed_ticks, 0);
assert_eq!(episode.snapshots.last().unwrap().summary.completed_ticks, 40);
let knowledge = world.knowledge(0).unwrap();
assert_eq!(knowledge.agent, 0);
```

Supply dimensions 3–125, an explicit open-cell mask, at least two distinct open
nest cells in one cardinal-connected chamber, 1–256 explicit nest spawns and
0–256 uniquely identified food tokens at distinct open cells outside the nest.
At most two workers occupy any cell, including nest cells; duplicate spawn cells
are accepted within that capacity. Food pockets disconnected from the nest are
valid and remain inaccessible. The five active parameters are `p_search` and
`p_return` (finite `[0,1]`), `lambda_fidelity` and `lambda_publish` (finite `[0,256]`),
and `lambda_waypoint` (finite, nonnegative). Unused F1 angular adapter fields
are zero; F3 accepts no angular controls or evolved defaults.

Private maps store only persistent Unknown/KnownOpen/KnownSolid classifications.
Ordinary spawn observations and each opportunity reveal only the current cell
and in-bounds cardinal neighbors. Current occupancy and food are ephemeral;
private maps retain neither. Recruitment provides a site coordinate without
marking it open, providing a route or sharing the publisher's map. Routes and
frontiers use only the worker's learned graph. Unknown informed sites use the
supplied Manhattan/frontier ranking, including frontiers that require moving
away from the site. Frontier travel commitments persist through congestion.
A nearest privately known nest route brings cargo home.

Each tick processes workers by ascending spawn-list ID on one seeded RNG stream.
One opportunity performs exactly one Move, Pickup, Deposit or Wait. Movement
into food cannot also pick up; movement into the nest cannot also deposit.
Departure/search/empty-return transitions consume waits. Searching tests give-up
before pickup; pickup freezes the current/cardinal available-food count including
the picked token once, excluding diagonals and solids. Only Deposit scores
food delivery. Loaded arrival deposits, then publishes, then chooses departure,
with independent publication/fidelity/recruitment draws. The publisher can see
its own record. Weak records expire lazily on nest arrival; researcher views
compute strengths without expiry or authoritative depletion filtering.

Capacity can cause prolonged waits or deadlocks. There is no forced swap,
displacement, hidden detour, priority override or promised progress. All requested
ticks run, including after delivery or with inaccessible food. Events use
processing ticks starting at zero; summaries count completed ticks. Missing
milestones remain censored, including every milestone in an empty world.
`initial = available + carried + delivered`; carried food at cutoff remains an
unfinished return. Resource coordinates always describe their original cells.

Request 1–7,200 ticks with at most 1,000,000 worker opportunities; individual
steps enforce cumulative limits. At 256 workers, 3,906 ticks are legal and
3,907 exceed the opportunity budget. Each private map and routing scratch
structure is bounded by `width * height`; the maximum population uses at most
4,000,000 classifications. Sampling intervals must be positive even with
recording disabled. Initial/final frames occur once; disabled recording yields
zero frames and bytes. A bounded counting writer caps the sum of compact
snapshot JSON at 64 MiB, including repeated geometry and summaries, excluding
Episode fields/setup and inter-frame delimiters. Serialization/storage failure
returns no successful partial Episode.

Summary exposes checked aggregate and per-worker physical/computational counters.
Constructor observations contribute computation but zero physical opportunities.
Additive computation counts sum; queue peaks use a maximum. Snapshots include
geometry, physical/resource states, selected sites/frontiers, frozen finds,
waypoint strengths, counters and per-worker known-open/solid counts, without
full private maps. `knowledge(agent)` returns a separate grid-bounded sorted
classification view and rejects invalid IDs. All researcher views and sampling
preserve state, advice, counters and RNG. Ordinary output retains no full action
or observation history.

Episode stores normalized setup, all five parameters and seed. Open/nest cells
sort by position and resources by identity; worker order and original food
coordinates remain intact. Replays require identical implementation and supported
platform. There is no saved-state restoration or cross-platform floating-point
trajectory identity claim.

F3 replaces F2's angular movement, informed angular age, turn delays, world-edge
targets, Euclidean travel and survey waits with fixed graph exploration and local
handling. Two-worker capacity, multi-cell nests, immutable private maps and
cardinal density are supplied engineering adaptations. F2's historical grid/time
calibration is not imported. F4 must explicitly handle excavation changing
Unknown/KnownSolid cells and maintain separate spoil/food ledgers, destinations,
hands and paid actions; immutable F3 knowledge is insufficient for dynamic maps.

## F4 implementation in progress

The [construction coupling spec](superpowers/specs/2026-10-06-foraging-4-construction-design.md)
proposes shared workers, one tagged food/spoil carrying slot, paid excavation,
direct spoil transport to a separate outlet, and private local map revisions.
The user approved that conversational architecture and deferred role allocation
and relay transport. The written spec was approved on 2026-10-06. The
[five-stage implementation plan](superpowers/plans/2026-10-06-foraging-4-construction.md)
was approved on 2026-10-06; subagent-driven engineering execution is starting.
Scientific evaluation has not started.


## Construction public usage (F4)

`foraging::construction` adds `World::new`, atomic `step`, read-only `summary`,
`snapshot` and single-worker `knowledge`, plus bounded `run`. F1/F2/F3 and
Burrow retain their existing APIs and behavior. The [approved F4 design](superpowers/specs/2026-10-06-foraging-4-construction-design.md)
defines one homogeneous worker population, shared hands and direct transport.

```rust
use sugarscape_core::foraging::construction::{
    run, Parameters, Pos, Resource, RunOptions, Setup, World,
};
let pos = |x, y| Pos { x, y };
let setup = Setup {
    width: 5, height: 3,
    open: vec![pos(0,0), pos(1,0), pos(2,0), pos(0,1)],
    diggable: vec![pos(3,0)], nest: vec![pos(0,0), pos(1,0)],
    waste: pos(0,1), workers: vec![pos(0,0)],
    food: vec![Resource { id: u64::MAX, pos: pos(3,0) }],
    parameters: Parameters {
        p_search: 1.0, p_return: 0.0, lambda_fidelity: 0.0,
        lambda_publish: 0.0, lambda_waypoint: 0.0,
    },
};
let episode = run(setup.clone(), 12,
    RunOptions { ticks: 40, sample_every: 7, snapshots: true }).unwrap();
let mut world = World::new(setup, 12).unwrap();
for _ in 0..40 { world.step().unwrap(); }
assert_eq!(episode.summary, world.summary().unwrap());
assert_eq!(episode.summary.work.opportunities, 40);
assert_eq!(episode.snapshots.first().unwrap().summary.completed_ticks, 0);
assert_eq!(episode.snapshots.last().unwrap().summary.completed_ticks, 40);
assert_eq!(run(episode.setup.clone(), episode.seed, episode.options.clone()).unwrap(), episode);
```

All five active passage parameters are explicit: `p_search` and `p_return`
are finite in `[0,1]`; `lambda_fidelity` and `lambda_publish` are finite in
`[0,256]`; `lambda_waypoint` is finite and nonnegative. Construction uses
F1 information mathematics with zero angular fields. There is no dig
probability, role fraction, relay length or global food/route guidance.
The example is an engineering acceptance fixture, not an efficacy claim.

Setup dimensions are 3–125 per axis; workers 1–256; food 0–256 with unique
arbitrary `u64` IDs and distinct original coordinates outside nest/outlet.
A nest contains at least two connected initially open cells. Worker order
identifies workers; at most two can occupy any cell. The initially open waste
outlet lies outside the nest and connects to it through initially open cells.
The immutable mask may overlap open cells; only currently solid masked cells
can be excavated. Protected buried food and disconnected exposed pockets are
valid censored inputs. Geometry and food are sorted for replay without
reordering workers or changing food origins. Invalid fields are aggregated
against original indices before dense allocation.

Hands hold `None`, `Cargo::Food(id)` or `Cargo::Spoil(id)`; equal numeric IDs
in the two material namespaces remain distinct. A successful Dig pays one
opportunity, opens one cell, exposes any Hidden food and creates carried spoil.
It does not move or collect food. Food returns to a nest cell; spoil goes directly
to the waste outlet. Loaded workers finish their current transport first,
subject to the same capacity and ascending-ID scheduler as empty workers.
DisposeSpoil resumes the paused Departing/Searching food intent on the next
opportunity, with no food arrival/publication draws or inflated food-trip counts.
Only DepositFood can publish its successful frozen food find.

Food conservation is `initial = hidden + available + carried + delivered`;
spoil conservation is `excavated = carried + disposed`; terrain conservation
is `open = initial_open + excavated`. Hidden food becomes Available at Dig;
physical accessibility means a current open route to the nest, ignoring workers.
Exposed food can remain inaccessible, and access can precede pickup and delivery.
`Summary.access` reports fixed per-food initial flags, first new exposure/access
and current shortest physical nest distances. First-event distances and contexts
stay frozen as later geometry changes. Researchers pay one multi-source BFS
at construction and each Dig, reported in `AccessCompute` separately from
worker `ComputeCounts`. Workers never receive that cache or hidden-food locations.

Private maps contain current/cardinal classifications and diggability only.
Workers prefer reachable private open frontiers before known diggable faces.
Another worker's dig leaves remote KnownSolid beliefs stale until fresh local
observation; these old walls are valid private beliefs. `KnowledgeView` labels
this memory rather than current physical truth. Snapshots expose researcher
terrain/food/spoil/advice and small knowledge counts, with full maps requested
one worker at a time. Views do not learn, run BFS, expire weak waypoints, revise
milestones or draw RNG. Queue peaks aggregate by maximum; other counts use
checked sums. Move + Dig + PickupFood + DepositFood + DisposeSpoil + Wait
always equals paid opportunities. Food/spoil/empty movement and congestion
remain separate, along with unpaid computation.

Runs request 1–7,200 ticks and at most 1,000,000 worker opportunities;
individual steps enforce the same cumulative bounds. Every requested tick
runs after milestones, exhaustion or delivery. Sampling intervals must be
positive even when disabled. Initial/final samples occur once; disabled
recording yields no frames and zero bytes. The 64 MiB cap sums compact JSON
for every Snapshot, including repeated inventories, access records and
geometry, excluding the Episode/setup/options wrapper and frame delimiters.
A bounded counting writer checks before committing each frame and byte count.
A run error returns no successful partial Episode. Maps and route scratch are
grid-bounded, with at most 4,000,000 classifications and at most 15,625 fixed
spoil records; initially open mask entries do not add spoil capacity. This
output cap is not a total-process-memory or throughput guarantee.

Events use processing ticks starting at zero and one-based committed opportunity
indices; snapshots count completed ticks. Missing pickup/delivery/access/disposal
at cutoff is censored; carried food/spoil remains unfinished. Empty food sets
create no fictitious completion milestone. Replay requires the same setup,
seed, options, implementation and supported platform; no JSON restoration or
cross-platform identity is promised. Compatible no-dig scenes are checked against
F3 per-step food/worker projections and cloned PCG continuation, including an
initially open mask entry. Extra construction diagnostics need not match F3 bytes.
Dedicated roles, relay/drop/pile transport and F5 scientific comparisons remain
separately deferred and require their own approved design/protocol.
