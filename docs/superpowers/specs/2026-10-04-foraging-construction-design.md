# Research-anchored foraging and construction design

**Date:** 2026-10-04.
**Status:** written design approved by the user on 2026-10-04. The [F1 implementation plan](../plans/2026-10-04-foraging-1-cpfa-rules.md) was approved by the user on 2026-10-04; the [F1 reference](../../foraging.md) is implemented and independently reviewed, merged into `main` at `5819a04`; F2 is also implemented, independently reviewed and locally merged into `main` at `a194f8e`, with [execution evidence](../plans/2026-10-05-foraging-2-fixed-world.md#execution-and-review-evidence--2026-10-05); fresh merged-tree verification passed. F3 is implemented, independently reviewed and locally merged into `main` at `c0e551a`, with 2,663 merged-tree tests passed, 0 failed and 103 ignored. F4 shared-worker construction is implemented, independently reviewed and locally merged at `568a3fa` on 2026-10-07, with fresh merged-tree verification (2,852 passed, 0 failed, 103 existing ignored), formatting and core Clippy clean. F5 is implemented, independently reviewed and locally merged into `main` at `c314283` on 2026-10-08; fresh merged verification passed survey425/0/0 and workspace3046/0/105, with formatting and Clippy clean. Scientific registration/execution remain separate.
**Programme:** construction B3, with later bridges to collective-agency communication and costly assistance.
**Evidence:** [source audit](../../studies/2026-10-04-foraging-construction-reading.md).
**Implemented starting point:** [Burrow resource access](2026-10-04-burrow-2-resource-access-design.md), integrated into main.

## Intent and success

Develop useful constructed underworlds from inspectable agents, with published models providing reference mechanisms. First reconstruct and verify central-place foraging rules, then establish collection/delivery in a fixed world, then ask whether construction changes its benefits and costs. Preserve separate claims for mathematical verification, computational reproduction, adaptations and biological comparisons.

The user requested incorporation of relevant existing research. The selected controller anchor is Hecker and Moses (2015). Michael and colleagues (2023) supply a later termite shortcut phenomenon, not a complete algorithm. Existing Burrow transport/cue evidence remains distinct from both.

## Alternatives and recommendation

1. **Source-rule reference first, recommended.** A small deterministic reference for CPFA search and information rules precedes its environment adapter. This exposes source discrepancies before movement or construction obscures them.
2. **Add food directly to current Burrow.** Quickly establishes delivery but would be a new composition, without a reproduced controller baseline. Defer until the passage adaptation is defined.
3. **Reproduce the later ARGoS system externally.** Useful for future comparison, but adds a simulator/toolchain and does not establish identity with the 2015 simulation. Keep the inspected repository as an explicitly dated source variant; do not install it for the first increment.

This spec covers the programme sequence and the first source-rule increment. Subsequent worlds require their own specs. It does not freeze their physics, statistical protocols or integrations.

## Campaign sequence

| Increment | Deliverable | Claim permitted after verification |
|---|---|---|
| F1: CPFA rule reference | Checked mathematics, source variants and information decisions | Specified published rules were implemented correctly |
| F2: fixed-world reference | Search, finite resources and nest return with explicit movement, sensing and timing | Paper-based model reconstruction; quantitative reproduction only with reconciled settings and registered targets |
| F3: passage adaptation | Four-neighbor passages, resource cargo and explicit navigation/capacity rules | Documented adaptation of the reference to our world |
| F4: construction coupling | Excavation and spoil/food interaction with measured delivery | New composition of verified components |
| F5: shortcut comparison | Straight/detour/twisting geometries and cost/route/delivery measurements | Defined computational comparison with a biological phenomenon |

F1–F5 are local labels for this branch of B3, not replacements for Minds or C1–C10. F1 and F2 are merged and verified; F3 engineering, independent reviews and verified local integration into `main` at `c0e551a` are complete. F4 construction coupling is also implemented, reviewed and locally merged at `568a3fa`. F5 shortcut comparison is implemented, independently reviewed and locally integrated at `c314283`, with separate source-reconciliation and scientific-protocol gates. Food extraction, physical carrying and excavation are later increments rather than hidden additions to F1.

## F1 boundary and architecture

Add a standalone `foraging` core module, following the bounded lab convention rather than extending Sugarscape harvesting, the Kirman `ants` model or Burrow's worker/actions. Use small files for checked parameters, mathematical rules, information decisions and behavior tests. Export the module from the core library. No universal mind interface or ontology engine is needed.

F1 receives explicit observations and deterministic random variates as inputs. It returns calculated probabilities, angular variation and information choices. It owns no world, scheduler, resource grid, simulated chemistry or waypoint server. The later world will own worker memory, records, movement and scheduling. This lets the same rule functions be tested before integration without pretending a supplied observation is learned by an agent.

No ModelKind registration, CLI command, WASM export, browser UI or new dependency is included in F1. Numerical utilities should use existing Rust standard-library operations. If stable computation cannot satisfy the stated limits with those operations, revise the plan before introducing a library.

## Checked parameter and input contract

Represent all seven source parameters explicitly, with no claimed evolved defaults:

- Search-switch probability `p_search` and unsuccessful-return probability `p_return`: finite, in `[0,1]`.
- Uninformed angular variation `omega`: finite, in `[0,4*pi]` radians.
- Informed decay `lambda_informed`, fidelity `lambda_fidelity`, publication `lambda_publish` and waypoint decay `lambda_waypoint`: finite and nonnegative.

The `[0,20]` initialization ranges for fidelity/publication in the paper are evolutionary initialization choices, not universal validity limits. Zero rates are permitted as explicit mathematical boundary cases, not claimed evolved results.

For the reference utility, accept density count `c` as an integer from 0 through 256 and fidelity/publication rates through 256. These upper limits are supplied engineering bounds for the 256-item first reference setting, not biological limits. A later large-world design must deliberately extend them. The other decay rates have no supplied finite upper cap beyond representability.

Elapsed search/waypoint ages must be finite and nonnegative, and uniform variates must be finite in `[0,1)`. Reject invalid values with contextual `FieldError` values before producing a decision. Validation must inspect original numeric inputs; truncating counts or replacing nonfinite values is forbidden.

F1 operates on abstract site identifiers, not world coordinates. A site identifier denotes a caller's previously observed location. There is no API accepting a global resource list or researcher access predicate. The caller validates location geometry in F2/F3; F1 validates only the supplied count, rates, ages, weights and draws.

## Mathematical rules

Implement the displayed equations as explicit reference functions:

1. Uninformed angular standard deviation is `omega`.
2. Informed angular standard deviation at age `t` is `omega + (4*pi - omega)*exp(-lambda_informed*t)`. Return the value; F1 does not sample headings or choose a four-neighbor direction.
3. `poisson_cdf(c, lambda)` is the probability of a Poisson variable being at most `c`, including `c`. Compute the lower tail, despite the source prose's conflicting description. With the declared rate/count bounds, use the recurrence starting at `exp(-lambda)`; avoid factorial overflow. Return a finite probability in `[0,1]`, permitting only roundoff-scale endpoint correction.
4. Waypoint strength is `exp(-lambda_waypoint*age)`. Handle zero age/rate explicitly so an overflowing intermediate product cannot turn a mathematically defined limit into NaN. Large positive products may produce zero strength, which is valid expiration.

An input comparison succeeds only when `probability > draw`, matching Algorithm 1. Probabilities zero and one therefore behave deterministically for valid draws. F1 does not equate a probability with a transition rate per second; decision cadence belongs to the later world.

## Information decision contract

Separate the publication decision from departure choice so the future server can process publication before providing its departure-time snapshot.

Publication takes the previous successful-find site and observed count, publication rate, and an independent uniform draw. It returns either a publication request for that site or no request. An unsuccessful return has no valid new find and cannot publish a stale observation as a new find.

Departure takes an optional valid previous-find record, fidelity rate/draw, the current available waypoint snapshot, and a separate recruitment-selection draw. It returns one of private site fidelity, recruited site with waypoint identity, or uninformed departure. Private fidelity has priority; otherwise choose an active waypoint; otherwise depart uninformed. A future world must explicitly determine whether an unsuccessful search invalidates private memory. F1 receives that validity from its caller and tests both cases, rather than guessing a lifecycle.

The paper does not fully specify server waypoint selection. Offer two named selection policies: uniform among active records as a supplied comparison rule, and strength-weighted selection as the inspected later-ARGoS variant. Never call uniform selection the documented 2015 rule. Iterate records in caller-provided stable order, reject duplicate waypoint identities, use half-open ticket intervals, and return the selected record's identity and site. F1 takes supplied variates and therefore cannot consume hidden random draws. No records means uninformed fallback without a selection calculation.

Offer two named threshold conventions: paper wording retains strength equal to 0.001, while the inspected later source requires strength strictly greater than 0.001. Both reject weaker records. Test equality directly with supplied weights; do not depend on constructing an exponential age that happens to round to the threshold.

A waypoint snapshot contains identity, site and current strength. Weights must be finite in `[0,1]`. F1 does not maintain or broadcast the list. Publication/server access timing, publisher visibility of its own new record, duplicate sites, capacity limits and storage costs are explicit F2 design decisions. The source server's existence is not evidence for arbitrary private channels or unlimited public memory.

## Ontology record

| Concept | State and visibility | Mechanism and measurement | Evidence boundary |
|---|---|---|---|
| Resource observation | Caller-supplied local count and site | Successful find supplies a decision input | F1 does not sense or discover resources |
| Private site fidelity | Caller supplies a valid previous-find record | Lower-tail CDF decides next-trip reuse | Published decision rule; memory lifecycle requires a world |
| Waypoint publication | Request carrying an observed site | Independent CDF comparison | Published server-mediated mechanism; no physical pheromone |
| Recruitment | Caller supplies an active-record snapshot | Explicit uniform or strength-weighted choice | Uniform is supplied; weighted is a later source variant |
| Search variation | Parameters and elapsed informed-search age | Calculated angular standard deviation | Source equation; no movement behavior established yet |
| Collective benefit | Not represented in F1 | Later completed delivery and beneficiary accounting | No efficacy, altruism or welfare conclusion from rule tests |

## F1 acceptance and tests

- Reject nonfinite/out-of-range rates, probabilities, ages, counts, weights and draws; no partial decisions on invalid inputs.
- Check Poisson values against independent hand-derived cases: rate zero; `c=0, lambda=1`; `c=1, lambda=1`; and selected higher-count bounds. Check monotonicity in count and inverse monotonicity in rate, with tolerances stated by the implementation plan.
- Verify informed variation at age zero, zero decay, increasing ages and the long-age limit; check bounded finite output and the `omega=4*pi` constant case.
- Verify waypoint strength at zero age/rate, ordinary decay and expiration, including a finite-input product that overflows; distinguish both threshold equality conventions.
- Verify independent publication/fidelity decisions, fidelity priority, recruitment fallback, no-find publication suppression, empty-list fallback, duplicate identity rejection and exact cumulative-interval boundary behavior.
- Demonstrate a strong old waypoint can lead to a depleted site because site validity is not inspected against hidden resource truth. This tests information separation, not a resource world.
- Tests use specified inputs/variates and independently calculated expected outcomes. They must not merely call the same formula in expected-value code. No preferred efficiency result or tuned demonstration seed is required.
- Existing workspace behavior, especially Burrow legacy/access replay, remains unchanged. Run relevant tests, formatting and core checks before review and commit. Broad integration checks follow the eventual implementation plan.

## Gates before F2 and later integration

Complete F1 review, then write the fixed-world spec with a source/assumption table covering heading updates, displacement, boundaries, detection area, count convention, survey time, decision cadence, empty return, nest radius, removal and delivery scoring. Establish whether historical code or supplementary data identify those choices; otherwise label them supplied reconstruction choices.

A four-neighbor passage world changes angular movement, body/collision rules and navigation. Treat that as F3 even if it reuses F2 functions. Conserved food requires distinct identities and an equation such as `initial = available + carried + delivered`, with extraction/collection, movement and deposit costs. Spoil cannot become food or share a bookkeeping destination by accident. F4 must define hand capacity, loaded priority, task switching and spoil/food interaction before using existing Burrow actions.

Future construction comparisons should retain initially inaccessible and already-open controls, hold opportunities/resources/information comparable, record first access separately from first delivery, report censored failures, and measure actual paid work and congestion. These are design requirements for a later protocol, not an executable treatment matrix approved here.

Quantitative reproduction of the paper's optimized performance requires its evolutionary settings, parameter provenance, independent evaluation layouts, outcome definition and an approved protocol. Termite comparison requires a full methods/data audit and explicit differences in substrate, body, sensing and scale. Neither follows automatically from passing F1 tests.

## Review handoff

The written design was checked for scope, source provenance and separation of verified rules from supplied choices, and approved by the user on 2026-10-04. Review of the [F1 implementation plan](../plans/2026-10-04-foraging-1-cpfa-rules.md) was approved by the user on 2026-10-04. Execution followed the user's standing subagent-driven preference; implementation, acceptance and independent reviews are complete. Integration into `main` completed at `5819a04`. Scientific registration/execution remain separate from engineering implementation.

## F2 follow-up

The [F2 historical audit](../../studies/2026-10-04-foraging-fixed-world-reading.md) and [fixed-world design](2026-10-04-foraging-2-fixed-world-design.md) are complete; the user approved the written spec on 2026-10-05. The [F2 implementation plan](../plans/2026-10-05-foraging-2-fixed-world.md) was approved on 2026-10-05; implementation and independent reviews are complete. Local integration into `main` completed at `a194f8e` on 2026-10-05. Fresh merged-tree verification passed: `cargo test --workspace` reported 2,463 passed, 0 failed and 103 ignored; `cargo fmt --all --check` and core clippy with warnings denied passed. F3 passage adaptation is implemented and independently reviewed; F4 next adds excavation and separate spoil/food logistics. Scientific execution has not started. The recovered historical source informs eight-neighbor angular movement, delays and completed-return scoring; release identity and quantitative reproduction remain unestablished.

## F3 follow-up

The [F3 passage design](2026-10-05-foraging-3-passage-design.md) was approved by the
user on 2026-10-06. The user selected private learned maps and approved local
observations, coordinate-only recruitment and a multi-cell nest chamber with
two-worker capacity throughout. The spec makes supplied exploration, action
cadence, information boundaries and bounded diagnostics explicit. The [five-stage implementation plan](../plans/2026-10-06-foraging-3-passage.md) is
approved on 2026-10-06; engineering implementation and independent
task/whole-branch reviews are complete; local integration into `main` completed
at `c0e551a` on 2026-10-06. Final runtime `693c02b` passed
workspace verification (2,563 passed, 0 failed, 103 ignored), formatting and core
clippy. Fresh merged-tree verification reports 2,663 passed, 0 failed and 103
ignored, with formatting and core clippy clean. Scientific evaluation remains
separate. F4 is the
separate excavation and spoil/food integration increment described below.

## F4 follow-up

The [F4 construction design](2026-10-06-foraging-4-construction-design.md) was
approved by the user on 2026-10-06. Shared workers, direct transport, one common
carrying slot, distinct food/spoil destinations and local dynamic-map updates
were approved conversationally on 2026-10-06. Role allocation and relay
transport are explicitly deferred. The spec proposes exploration before
excavation, paused food-trip resumption after spoil disposal, separate material
conservation and structural-access versus realized-delivery measurements.
The [five-stage implementation plan](../plans/2026-10-06-foraging-4-construction.md)
was approved on 2026-10-06. All five engineering stages and independent task/whole-branch reviews are complete. Runtime `25375c3` passed workspace 2,852/0/103 (passed/failed/existing ignored), rustdoc, formatting and core Clippy. Verified local integration into `main` completed at `568a3fa` on 2026-10-07; fresh merged-tree workspace verification also reports 2,852 passed, 0 failed and 103 existing ignored, with formatting and core all-target Clippy clean. Evidence and cleanup are recorded in the plan; only this feature worktree/branch were retired. Scientific evaluation has not started.

## F5 engineering handoff

The user approved mechanism-first scope and an immutable collector/saved-only analyzer architecture on 2026-10-07. The [approved F5 comparison specification](2026-10-07-foraging-5-shortcut-comparison-design.md) records supplied geometry, controls, fixed candidate workload, outcomes and archive boundaries; the user approved the written spec on 2026-10-07. The user approved its [implementation plan](../plans/2026-10-07-foraging-5-shortcut-comparison.md); all five subagent-driven engineering stages, task reviews, whole-branch review and one consolidated correction/re-review are complete. Final runtime `e67934b` adds exact spoil lifetime checks and binds recorded code revision to compiled source before collection. Corrected survey382/0/0, formatting and all-targetClippy pass; the unchanged root workspace2919/0/103 and root format/core Clippy gates remain applicable through input identity proof. One nonblocking cumulative-test isolation follow-up remains recorded in the plan. F5 was locally merged into `main` at `c314283` on 2026-10-08, preserving concurrent work. Fresh merged verification passed survey425/0/0 and workspace3046/0/105, both formatting checks and both all-targetClippy gates. The maintained source selector also binds two new active-surface fixture includes. Integration receipts and feature cleanup are recorded in the plan. The [preliminary source audit](../../studies/2026-10-07-foraging-shortcut-reading.md) retains unresolved data/figure access and model/source differences. Executable scientific registration and scientific collection remain separate; the candidate stays draft/unauthorized and no F5 scientific outcome exists.
