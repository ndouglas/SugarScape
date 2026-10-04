# Foraging and construction: source audit

**Date:** 2026-10-04.
**Status:** initial methods/code audit supporting the approved campaign design. The [F1 rule reference](../foraging.md) is implemented and independently reviewed, not yet integrated; no simulated results.

## Purpose

Anchor resource collection and delivery in published research before composing it with Burrow excavation. Keep source-rule reconstruction, environment adaptation and biological comparison distinguishable. The user's eventual target remains useful, diverse underworlds and inspectable collective behavior.

## Primary controller anchor

Joshua P. Hecker and Melanie E. Moses (2015), *Beyond pheromones: evolving error-tolerant, flexible, and scalable ant-inspired robot swarms*, Swarm Intelligence 9:43–70, DOI 10.1007/s11721-015-0104-z.
[Author-hosted paper](https://moseslab.cs.unm.edu/website-archives/publications/beyond-pheromones.pdf).

Methods sections 3.1–3.6, Algorithm 1, equations 1–5, Table 1 and the experimental setup were inspected using the PDF text layer. This is not a complete methods/supplement audit or a numerical reproduction. A personal copy and hash/provenance record are stored in gitignored `papers/foraging/`.

### Rules sufficiently explicit for a first reference increment

- Seven controller parameters govern transition to search, unsuccessful return, uninformed turning, informed-search decay, site fidelity, publication and waypoint decay.
- Equation 1 draws the next heading from a normal distribution centered on the previous heading. Equation 2 uses fixed angular standard deviation `omega` for uninformed search.
- Equation 3 sets informed-search angular standard deviation to `omega + (4*pi - omega)*exp(-lambda_id*t)`. It starts broadly turning and approaches the uninformed value with increasing search age.
- Equation 4 is the lower-tail Poisson CDF through `floor(c)`. Algorithm 1 compares this value with an independent uniform draw for publication and for private site fidelity. The prose's description of an “at least” probability does not match that equation. Implement the displayed equation, disclose the inconsistency, and do not silently substitute the upper tail.
- Site fidelity has priority over recruitment; a new random departure is the fallback. Resource information originates in a previous find or a received waypoint, rather than supplied knowledge of all food locations.
- Equation 5 gives waypoint strength `exp(-lambda_pd*t)`, initialized to one. The paper describes removal below 0.001.
- Communication uses a central server's waypoint list, offered to robots returning to the nest. It is not a spatial chemical field, an unrestricted hive mind or peer-to-peer messaging.

### Environment and outcome distinctions

The 2015 simulation uses a 125 by 125 grid representing 8 cm cells, a one-hour window and 256 resources. Resource placements include four clusters of 64, a heterogeneous arrangement of one 64-item cluster, four 16-item clusters, sixteen 4-item clusters and 64 scattered items, and a random distribution. Angular search is still part of the controller; a gridded environment does not make it a four-neighbor random walk.

The parsimonious fitness simulation omits explicit collision detection. The physical robots read one-use QR tags and return to the nest; they do not physically carry those tags. Our desired conserved physical food cargo is therefore a declared extension. Collection/removal and completed nest return must be measured separately; the original scoring event needs verification before comparing numerical efficiencies.

The paper's larger results include genetic optimization, sensor/navigation error and evaluations across swarm sizes. A fixed-parameter controller does not reproduce those evolutionary results. Parameter initialization distributions in Table 1 are not published optimal parameter sets.

## Related public implementation

The lab's [CPFA-ARGoS repository](https://github.com/BCLab-UNM/CPFA-ARGoS) was inspected read-only at revision `18fc0d9813e37bcc01c54ec9896435f1038f4295`, dated 2019-04-09. It is a later ARGoS implementation, not established as the simulator used for the 2015 results.

Inspected files:

- [Controller](https://github.com/BCLab-UNM/CPFA-ARGoS/blob/18fc0d9813e37bcc01c54ec9896435f1038f4295/source/CPFA/CPFA_controller.cpp): departure/search/return logic, density calculation, waypoint choice and Poisson CDF.
- [Waypoint implementation](https://github.com/BCLab-UNM/CPFA-ARGoS/blob/18fc0d9813e37bcc01c54ec9896435f1038f4295/source/Base/Pheromone.cpp): exponential decay and active threshold.
- [Example configuration](https://github.com/BCLab-UNM/CPFA-ARGoS/blob/18fc0d9813e37bcc01c54ec9896435f1038f4295/experiments/CPFAExample.xml): example parameters and timing; not evidence of 2015 evolved settings.

Important reconciliation points:

1. `GetPoissonCDF` agrees with the displayed lower-tail equation.
2. `SetLocalResourceDensity` includes the picked-up item plus nearby remaining items. The paper's neighborhood/count wording does not fully resolve that convention.
3. `SetTargetPheromone` selects by waypoint strength. The paper states that the server chooses a waypoint without fully specifying that selection rule.
4. `IsActive` requires strength strictly greater than 0.001; the paper says removal below the threshold. Equality requires an explicit source variant.
5. Departure switching is checked at a particular simulator cadence; give-up decisions occur at search-target arrival. A per-tick implementation changes effective transition hazards.
6. Informed turning includes an angular bounding operation in the later controller. That operation is not explicit in equation 1.

The local source clone reported a case-insensitive filename collision between two GA makefiles. Only the listed controller/configuration files were used; the clone was not built or executed. No source code was copied into SugarScape and no ARGoS dependency was installed.

Before a whole-model reference world is specified, extract movement/timing/boundary/detection conventions and establish the historical source's relationship to the paper. If those remain unavailable, name the model a paper-based reconstruction and enumerate supplied choices. Do not claim exact replication from the later repository.

## Direct construction/foraging experimental target

Zion Michael, Thomas Chouvenc, Nan-Yao Su and Sang-Bin Lee (2023), *Finding shortcuts through collective tunnel excavations in a subterranean termite*, Behavioral Ecology 34:354–362, DOI 10.1093/beheco/arad007.
[Publisher article](https://academic.oup.com/beheco/article/34/3/354/7059060).

Search-accessible methods/discussion describe planar sand arenas with straight, detour and detour-plus-twisting preformed routes to cellulose food. Measurements include branching, shortest route length and tunnel width over time. Termites used existing routes, then excavated shortcuts in detour conditions; later widening followed. Direct page access was inconsistent, so the accessible indexed text is the current inspection boundary; data and supplementary materials were not inspected.

This supplies a closely related biological phenomenon and potential geometry/measurement benchmark. It does not identify a unique verified controller. The authors discuss possible local cues and directional information while acknowledging uncertainty about the mechanism. Shortening a route is distinct from measuring delivered food, net energy benefit or altruism.

## Continuity and later expansion

[Pielström and Roces (2013)](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0057040) remain the soil-transport/local-cue anchor; see the [existing excavation reading](2026-10-03-burrow-excavation-reading.md). Soil disposal and nutritional delivery have different destinations and benefits, even when both use carrying transactions.

[Lu, Moses and Hecker's multiple-place foraging model](https://arxiv.org/abs/1612.00480) provides a later lead for distributed depots and the travel/communication trade-off. Its abstract and accessible article were located, but its full rules were not audited here. Multiple nests remain a separate increment.

## Recommended campaign sequence

1. Verify source mathematics and information decisions without a movement adapter.
2. Build a fixed-environment reference with documented timing, search and resource conventions; separately adapt that reference to four-neighbor passages.
3. Compose the passage adaptation with conserved food cargo and Burrow excavation. Measure construction expenditure and actual delivery independently.
4. Design a termite-inspired shortcut comparison after the composed world is verified.
5. Add multiple depots, differentiated beneficiaries, assistance and conflict through further explicit designs.

The [approved campaign and first-increment spec](../superpowers/specs/2026-10-04-foraging-construction-design.md) records this sequence, the implementation boundary and review gate.
