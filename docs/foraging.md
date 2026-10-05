# CPFA rules (F1) and fixed-world foraging (F2)

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

F2 engineering and independent task/whole-branch reviews are complete; local integration into `main` completed at `a194f8e` on 2026-10-05. The final counter correction passed scoped review. Fresh feature and merged-tree workspace verification each report 2,463 passed, 0 failed and 103 ignored, with formatting and core clippy clean. The implementation plan preserves commands, rulings and evidence. Scientific execution remains separate; F3 passage adaptation is the next design increment.

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
