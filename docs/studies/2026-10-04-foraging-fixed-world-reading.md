# F2 historical source and fixed-world audit

**Date:** 2026-10-04. **Status:** source inspection complete for a proposed zero-error reconstruction; no simulator built or executed, and no published result reproduced.

This extends the [F1 audit](2026-10-04-foraging-construction-reading.md). The [F2 design](../superpowers/specs/2026-10-04-foraging-2-fixed-world-design.md) records which conventions we propose to adopt.

## Primary materials and provenance

- Hecker and Moses (2015), [Beyond pheromones](https://moseslab.cs.unm.edu/website-archives/publications/beyond-pheromones.pdf), DOI [10.1007/s11721-015-0104-z](https://doi.org/10.1007/s11721-015-0104-z). Methods 3.1–3.6, Algorithm 1, equations and physical/simulated distinctions were inspected using the author-hosted PDF and extracted text.
- Publisher [electronic supplement](https://media.springernature.com/original/springer-static/esm/art%3A10.1007%2Fs11721-015-0104-z/MediaObjects/11721_2015_104_MOESM1_ESM.pdf), two pages. Its extracted text identifies Figures 13–14: evolved parameter trends/distributions for zero-error power-law layouts and swarm sizes 1–768. It supplies no executable release identity, complete numerical parameter table, movement contract or raw evaluation layouts. We did not digitize plots.
- Historical [BCLab-UNM/iAnt-Sim](https://github.com/BCLab-UNM/iAnt-Sim), cloned read-only for source/history inspection. Repository tip inspected: `cd3191f859005d0ff9177262e24d3996f1cd2e4f`, 2015-08-18. Historical comparison pinned to **`897fc9093b1562bd7e1b4b9285b9e88e40ffd890`**, 2015-02-12, the last commit before the publisher's 2015-02-15 publication date. History begins 2013-02-24. Prepublication timing and matching dimensions support relevance; they do **not** establish that this commit generated the paper's experiments. Submission was 2013-12-31 and acceptance 2015-01-21; the selected commit postdates acceptance.
- Later CPFA-ARGoS remains pinned to `18fc0d9813e37bcc01c54ec9896435f1038f4295` (2019-04-09) as documented in F1. It is a separate implementation, not evidence that ARGoS was the original fitness simulator.

Historical paths use `AntBot-GA/`, before the later `iAnt-Sim/` rename. Inspection initially attempted the renamed paths at the old revision, then resolved the paths with `git ls-tree`. No source was imported into SugarScape and no Cocoa, OpenCV or ARGoS toolchain was installed.

## Historical contract recovered

All historical observations below refer to the pinned revision:

- [Simulation.mm](https://github.com/BCLab-UNM/iAnt-Sim/blob/897fc9093b1562bd7e1b4b9285b9e88e40ffd890/AntBot-GA/Simulation.mm): initialization defaults to 125×125 cells, nest (62,62), six robots, 256 tags and 7,200 ticks. The paper supplies 8 cm cells and a one-hour run. Interpreting a tick as half a second follows from those two sources together, rather than an explicit time-unit declaration in the inspected controller.
- The same file's `stateTransition` iterates robots in array order. Departure tests uninformed switching on an eligible tick; informed travel instead reaches its target. Changing state to search performs a turn and ends the opportunity. Search checks give-up **before** moving or detecting food; delays suppress both decisions and motion.
- Search chooses the rounded endpoint of `(x+cos(theta), y+sin(theta))`. If outside the grid, it repeatedly redraws a uniform heading until legal. After moving, it turns, then detects food one rounded cell ahead under the **new** heading. Detection is not simply harvesting the occupied cell.
- A found tag is marked picked up immediately. The successful tag is first in `discoveredTags`; remaining detectable, unpicked tags in its Moore neighborhood follow. Thus zero-error count is **one plus available neighbors**, at most nine. It installs a nine-tick survey delay and targets the nest.
- A returning robot moves first and, on exact equality with the nest cell, adds the successful tag to `collectedTags`. `evaluateTeams` adds that count to fitness. Pickup and completed return are distinct events.
- [Robot.m](https://github.com/BCLab-UNM/iAnt-Sim/blob/897fc9093b1562bd7e1b4b9285b9e88e40ffd890/AntBot-GA/Robot.m): travel/return considers eight legal neighbors, chooses positive Euclidean-distance improvements with proportional weights, and jumps directly into an adjacent target. Turns sample a normal increment, clip it to [-pi,pi], then wrap heading into [0,2*pi). Both informed and uninformed turns are clipped. Informed age increments per turn, not wall-clock second. The turn-delay expression divides by `(pi/4+0.001)`, applies `abs`, converts to an integer and adds one; overload/conversion details need explicit reconstruction semantics.
- [Pheromone.m](https://github.com/BCLab-UNM/iAnt-Sim/blob/897fc9093b1562bd7e1b4b9285b9e88e40ffd890/AntBot-GA/Pheromone.m): server selection already weights records by strength in this historical code. Decay uses elapsed **ticks**; removal is strictly below 0.001. Publication precedes selection, allowing a publisher to select its new record. Duplicate sites are not coalesced.
- [Utilities.h](https://github.com/BCLab-UNM/iAnt-Sim/blob/897fc9093b1562bd7e1b4b9285b9e88e40ffd890/AntBot-GA/Utilities.h): lower-tail Poisson CDF, Box–Muller-style normal sampling, uniform selection of one of four edges and an integer coordinate along it. Native random-number sequence and float precision are specific to this implementation.
- [SensorError.m](https://github.com/BCLab-UNM/iAnt-Sim/blob/897fc9093b1562bd7e1b4b9285b9e88e40ffd890/AntBot-GA/SensorError.m): zero-error construction gives zero positional perturbation and detection probability one. The neighbor loop in `Simulation.mm` calls `detectTag`, despite a separate `detectNeighbor` method. This discrepancy matters to a future error-model reconstruction; it disappears in the proposed zero-error baseline.

## Discrepancies and limits

1. **Recruitment fallback:** the historical controller samples a fidelity decision even without a found tag and requires that flag to be false before recruitment. Consequently a successful fidelity draw can suppress recruitment on an empty return. F1 follows the paper's valid-find fidelity → recruitment → random ordering. F2 should preserve F1 and record this deliberate departure from the historical code.
2. **Angular bounds:** the paper's displayed normal equation does not describe clipping. Historical code clips both search modes; later ARGoS inspection showed clipping in informed search. These must not become one unnamed implementation.
3. **Timing:** informed age is number of turns, server age is ticks, and delays create opportunities with no motion. Rates cannot silently be reinterpreted per second.
4. **Survey:** historical nine-tick delay is concrete. The later ARGoS comment describes four seconds but its `Wait(4)` call is disabled. Comments are not executable timing evidence.
5. **Nest:** original historical code requires exact nest-cell arrival. Paper physical robots return within 50 cm; later ARGoS uses its own configured radius. F2 should use the historical discrete nest, without claiming the physical platform's geometry.
6. **Boundaries and replay:** historical unbounded rejection can hang with adversarial draws. A bounded failure contract and double-precision Rust sampling are engineering changes. Same-seed replay in SugarScape will not reproduce native Objective-C random trajectories.
7. **Extra knowledge:** this historical revision contains inferred cluster guidance behind a cutoff setting. Disable it; neither that branch nor GA/error-model evolution belongs in the first fixed-world controller.
8. **Release/data identity:** no audited manifest links this revision to original paper runs. Evolved tables and independently drawn layouts are not recovered by finding source code. Performance reproduction remains a separate provenance and protocol task.

## Consequence for F1 and F2

The F1 enum `LaterArgosStrengthWeighted` describes a verified behavior but its provenance is now broader: the historical simulator also uses strength-weighted selection. Keep the existing public spelling compatible; document the historical support and use it together with `PaperBelow`. A later API naming cleanup is optional, not a prerequisite for this campaign.

Prefer a **paper-based reconstruction informed by a pinned historical simulator**, with eight-neighbor angular discretization, explicit delays and conserved resource identities. This offers a better-founded baseline than inventing continuous geometry or moving immediately into four-neighbor passages. It remains a reconstruction with declared changes, not an exact historical executable or quantitative replication.
