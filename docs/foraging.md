# CPFA rule reference (F1)

The public `sugarscape_core::foraging` module verifies stateless rules from Hecker and Moses (2015). Engineering implementation and independent task/whole-branch reviews are complete; this work merged into `main` at `5819a04`. F1 neither simulates foraging nor reproduces evolved performance. It supplies no world, heading sampler, scheduler, food ledger or waypoint server.

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

## Next gate: F2 source reconciliation

Before specifying a fixed world, reconcile historical simulator provenance; heading updates and angular bounding; displacement and boundaries; detection area and whether density includes the picked-up item; survey time; decision cadence and give-up events; empty-return memory; nest radius; resource removal and delivery scoring. Also decide publication/server access timing, self-visibility, duplicate sites, capacity and storage costs. Unresolved conventions must be labeled supplied reconstruction choices in an approved F2 design, rather than treated as runtime commitments here.

F3 passage adaptation, F4 excavation coupling and F5 termite comparison retain separate design gates. Quantitative evolutionary reproduction also requires parameter provenance, evolutionary settings, independent evaluation layouts, outcome definitions and an approved scientific protocol. Passing F1 rule tests establishes none of those scientific outcomes.

## F2 design follow-up

The [historical source audit](studies/2026-10-04-foraging-fixed-world-reading.md) recovered a prepublication iAnt-Sim implementation with eight-neighbor angular movement, explicit delays and nest-return scoring. The [draft fixed-world design](superpowers/specs/2026-10-04-foraging-2-fixed-world-design.md) records proposed source conventions and deliberate reconstruction choices; written-spec review is pending. Strength-weighted recruitment is now also verified in the historical simulator, while the existing `LaterArgosStrengthWeighted` API spelling remains unchanged.
