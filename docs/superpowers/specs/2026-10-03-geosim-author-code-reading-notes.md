# GeoSim author-code provenance and docking notes

Date: 2026-10-03. Research and archive inspection only; no author code copied into the product, no build or simulation run attempted.

## Conclusion

The recovered GROWLab GeoSim2 is a strong author-code docking candidate. Neither inspected archive has been established as the exact source tree used for the February 2003 APSR results. The 2004 RePast GeoSim0 explicitly attributes itself to that article, but its implemented mechanics are a simpler state-formation model. The later GeoSim2 port contains distance extraction, technological shocks, campaign behavior, and war-cluster instrumentation, with defaults explicitly linked to the reported experiment.

## Public source search

Targeted searches for Cederman/Girardin GeoSim and GROWLab repositories on GitHub did not identify an APSR source repository. This is a search outcome, not proof that no repository exists.

- [ETH GeoSim](https://icr.ethz.ch/research/geosim/) describes the transition from Java RePast to GROWLab.
- [ETH GROWLab models](https://icr.ethz.ch/research/growlab/models/) lists distinct GeoSim0, GeoSim2, GeoSim4, and GeoSim5 models.
- [ETH source installation](https://icr.ethz.ch/research/growlab/documentation/installation/) gives an author-controlled Subversion location, historically requiring an account: `http://cederman.ethz.ch/svn/java/ethz/trunk/growlab/`. The page describes an IntelliJ project and Java3D prerequisite. The SVN endpoint and historical revisions were not recovered in this inspection.
- [ETH current downloads](https://icr.ethz.ch/research/growlab/download/) provides GROWLab 0.9.7 installers. These are later than the local 0.9.5 archive.
- [ETH GROWLab](https://icr.ethz.ch/research/growlab/) dates release 0.9.7 to August 23, 2022 and names Girardin and Weidmann as contacts.

## RePast GeoSim0 archive

Local archive: `papers/geopolitics/cederman-icr-2004-geosim0-repast-source.zip`.

`geosim0/src/ch/ethz/icr/geosim0/Model.java:40–52` associates the model with Cederman's 1997 architecture and APSR 97 (February 2003): 135–150; authors are Luc Girardin and Lars-Erik Cederman, version 3.0. Similar attribution appears in `ModelGUI.java`, `ModelCluster.java`, `Relation.java`, and `Utility.java`. `ModelBatch.java` also retains older Harvard/March 11, 2002/version 2.0 documentation. ZIP member timestamps are June 2004; these timestamps and inherited documentation do not establish the date of the scientific run.

The implemented mechanics support the narrower classification:

- `Model.java:149–166`: defaults seed 1, 30×30 world, initial polarity 200, attack probability 0.2, superiority threshold 3, superiority slope 0, mobile-resource proportion 1, horizon 10,000.
- `Actor.java:286–300`: resource update starts with one capital resource unit and adds one unit per dependent province. There is no distance-dependent extraction in this method.
- The source members are `Actor.java`, `Model.java`, `ModelBatch.java`, `ModelCluster.java`, `ModelGUI.java`, `Relation.java`, and `Utility.java`; there is no `War.java` or corresponding shock/distance extraction mechanism.
- `ModelBatch.java:85–89` records polarity (`sov`) and security (`sec`), not APSR war severity.
- `geosim0/dataset/params.txt` sweeps RNG seeds 1–10; it is not evidence of the published war-size parameter configuration.

Thus the earlier inference that this recovered GeoSim0 lacks technological shocks is supported by its mechanics. Its APSR attribution is insufficient to identify it as the full war-size implementation.

## GROWLab GeoSim2 archive

Outer archive: `papers/geopolitics/cederman-girardin-weidmann-icr-2017-growlab-0.9.5-source-geosim-geocontest.zip`.

Nested archive: `models/geosim2-0.9.0.zip`. All members below are within that nested archive, under `src/ch/ethz/icr/growlab/model/geosim2/` unless specified otherwise.

`Geosim2Model.java:27–51` attributes the model to APSR 2003 and the Cederman/Gleditsch democracy work, names Cederman and Girardin, and gives version 2.0.1. It retains documentation requiring RePast 1.4 and warning that RePast 2.0's changed RNG destroys numerical identity. The actual class extends GROWLab `AbstractModel` and uses GROWLab structures: the RePast requirement is inherited documentation, not proof that the port requires RePast at runtime.

`Geosim2Lab.java:161–238` explicitly links the defaults to Cederman 2003 and calls seed 5 its representative run. Relevant defaults:

| Setting | Default |
| --- | --- |
| Seed and lattice | 5; 50×50 |
| Initial polarity | 200 |
| Superiority and victory thresholds/slopes | 3; 20 |
| Unprovoked attack probability | 0.01 |
| Stalemate probability | 0.1 |
| Mobile resource proportion | 0.5 |
| Battle cost | 0.1 |
| Democracy / democratization / alliances | false |
| Distance extraction | enabled; threshold 2; slope 3; offset 0.1 |
| Resource adjustment proportion | 0.01 |
| Stationary-period start / run horizon | 500 / 10,500 |
| Shock size / probability | −20 / 0.0001 |
| War shadow | 20 |
| Output | severity |

`Actor.java:274–279` uses the local state's extraction threshold in the logistic distance discount. `Geosim2Model.java:864–868,883–896` implements a negative technological shock by updating a selected state's extraction threshold to `2 + 20(t−500)/10000` at the default settings. This is occasional state-level updating to a time-dependent threshold, not adding a fixed increment on each selected shock. `Geosim2Model.java:973–983` applies shocks after the stationary-period threshold and enables war counting at that threshold.

`War.java:10–20` defines wars as fighting-linked spatio-temporal clusters with participant memory shadows. `War.java:125–148,152–227,231–259` removes exhausted/non-sovereign participants, decrements inactive shadows, consolidates linked wars, and terminates empty clusters. `Geosim2Model.java:539–544` accumulates discounted combat damage for mutual fighting in the war's resource-damage measure.

**Severity serialization matters:** `War.java:267–269` and `War.java:338–339` return `(int) (100.0 * res)`. The observed author output is integer truncation after multiplication by 100, not raw floating-point damage. This scale/truncation must be accounted for before comparing severity thresholds or distributions.

`plugin.xml` identifies plugin `geosim2`, version 0.9.0, and GROWLab core dependency. The outer package also includes nested GeoContest, GeoSim0, GeoSim4, and GeoSim5 archives. The 2017 packaging date does not identify the original implementation date.

## Licensing boundary

GeoSim0 Java headers explicitly declare GPL version 2 or later. No code is to be transplanted into the product.

The GROWLab archive is not uniformly documented as GPL: outer `src/ch/ethz/icr/growlab/GROWLab.java` and `src/ch/ethz/icr/growlab/core/plugin.xml` declare LGPL version 2.1 or later. Inspected nested GeoSim2 Java files instead have the template header `Copyright (c) 2006, Your Corporation. All Rights Reserved.` Their model-specific licensing is unresolved by these headers. Treat the entire author distribution as reference-only; do not infer permission to reuse model source from the framework's license.

## Build feasibility and limits

The installed runtime reports OpenJDK 25.0.1. GeoSim0 `Model.class` has class-file major version 46 (Java 1.2); GeoSim2 `Geosim2Model.class` has major version 51 (Java 7).

GeoSim0 bundles Colt and compiled classes. `geosim0/GeoSim 0.iml` references an external RePast module; the ZIP does not bundle that dependency. Core/GUI/batch source imports old `uchicago.src.sim.*` APIs. Cluster execution adds distributed RePast/ProActive concerns and is unnecessary for an initial docking attempt.

The GROWLab outer ZIP bundles framework binaries and dependencies including Colt, JPF, charting, Groovy, Macrofocus libraries, Java3D, and compiled model classes. It contains no root build file in the inspected listing. A separate launcher against its existing binaries may be more practical than rebuilding the entire old GUI/toolkit, but this has not been tested. Source compatibility with Java 25, framework startup, legacy dependency linkage, plugin loading, simulator time, and collector behavior remain unverified.

Before running anything, inspect a minimal GROWLab simulator entry point, dependencies needed to instantiate the lab, time propagation, stopping behavior, and how completed-war buffers are drained. Do not silently modify model behavior to make it run. Record dependency versions and any compatibility patches separately.

## What docking could establish

Agreement with this GROWLab port would establish agreement with the archived author implementation under explicitly matched settings. It would not alone prove reproduction of the exact APSR implementation or the published seed-5 numbers. Figure-level similarity also cannot settle RNG or event-definition equivalence.

The next source verification should seek author SVN historical revisions or the original RePast 1.4 model and seed-5 settings/output through the authors' documented provenance. Compare the port with that version for RNG streams, draw consumption, shuffle/list ordering, tick boundaries, campaign transitions, shock timing, battle damage, cluster merging/termination, burn-in, terminal censoring, and collector draining. Preserve the distinction between author lineage, documented correspondence, observed run agreement, and exact publication-source identity.

## Targeted audit: primary specification versus recovered artifact

This follow-up inspected the APSR appendix on printed pages 146–148 and the nested GeoSim2 source directly. The primary paper defines the source-first scientific scope. Artifact quirks belong in a separately identified docking/reference lane. These are differences between the paper and the recovered GROWLab port; without the original APSR source, they cannot be dated as changes introduced by the port.

All Java references below are inside `models/geosim2-0.9.0.zip`, under `src/ch/ethz/icr/growlab/model/geosim2/`, unless otherwise stated.

### Distance and projection

`Actor.java:252–253` computes Euclidean distance. Annexation caches distance from the new capital in `Geosim2Model.java:614`. `Actor.java:274–279` combined with `Geosim2Model.java:1084–1088` yields `offset + (1-offset)/(1+(distance/threshold)^slope)`, with distance zero returning 1. The exponent is positive in this decreasing distance curve. The printed formula on APSR p.146 displays a negative exponent, inconsistent with the accompanying decreasing-extraction interpretation and the artifact. This discrepancy requires an explicit interpretation, not an unnoticed transcription.

`Actor.powerRatio`, `Actor.java:675–682`, discounts both the initiator's launch resources and the opponent's target resources using the initiator's extraction curve. The paper's local balance on p.148 refers to each state's respective capital-to-battle distance function. `Geosim2Model.battle`, lines 566–568, uses each respective owner's curve. The source-first lane should use respective state curves; the artifact lane can name and reproduce the initiator-curve decision calculation.

### Loss accounting and severity

`Geosim2Model.java:525–543` records negative `dres` for resources lost to the opponent's projected forces. During unilateral attack, only the cooperating victim has nonzero damage; the attacker has zero. During mutual fighting both sides incur damage.

`Actor.updateRes`, `Actor.java:354–377`, sums these negative losses and computes `dRes = harvest - damage`. The resulting resource target adds the magnitude of losses. The paper's p.147 pseudocode subtracts total damage. Source-first behavior should subtract losses; artifact docking must explicitly identify the recovered add-loss feedback rather than silently correcting it.

The artifact adds damage to cluster severity only in the mutual-fighting branch (`Geosim2Model.java:539–544`). Both unilateral and mutual attack fronts nevertheless create or refresh cluster membership (`497–515`). The paper p.148 describes cumulative damage for conflict clusters without disclosing this mutual-only measurement restriction. Keep event participation and measured severity distinct.

### Stalemate and victory

`probStalemate` occurs only in `Geosim2Lab.java:80,173,189`: a field, default 0.1, and exposed parameter list. It is not read anywhere else in the nested GeoSim2 Java source. It has no operative random-stalemate effect in the inspected artifact. Do not infer a functioning probability knob from its label.

`Geosim2Model.java:550–575` tests one victory claim per side using independent Bernoulli calls, except in deterministic or zero-opponent branches. Any defending-side claim clears both claims and ends fighting (`553–584`), giving defender victory priority when both sides claim. The paper p.148 permits simultaneous claims but does not settle shared versus independent random draws.

### Initialization, RNG, and campaign timing

`Geosim2Model.java:153–158` creates two independent Mersenne Twister instances initialized with the same seed. Initial actor-order shuffle uses the main stream (`190`); the rebuilt sovereign list uses the second stream (`840–854`). Seed 5 is the documented representative run (`Geosim2Lab.java:166`). Numerical docking depends on both streams and their consumption order, not merely matching the seed label.

`Actor.java:79–90` calls the normal helper for superiority and distance threshold even at zero SD, then initializes each primitive actor's resources to 100 with probability 0.2 or otherwise 1. `Geosim2Model.normal`, lines 1067–1068, consumes the normal draw before multiplying by SD. All actors start `newlyIndep=true` (`Actor.java:115`).

The first 200 actors from the initially shuffled list become founders (`Geosim2Model.java:211–217`). In fixed founder order, each pass annexes one uniformly chosen adjacent eligible nonfounder atom per founder; passes continue until none changes (`222–250`). Annexation transfers the child's resources at tax rate 1 (`631–634`). The first resource update replaces founder resource totals with distance-extracted harvest because they remain newly independent; subsequent updates smooth with fraction 0.01 (`Actor.java:374–383`). Thus the initial 100/1 values are overwritten in the default founder setup, while their RNG draws still affect later choices.

Campaign dropping is checked whenever a decision invocation begins with a campaign target, before the attack-attempt decision, with no battle-completion guard (`Actor.java:713–718`). A target excluded from the eligible victim list also clears (`761–763`). Table A1 on p.147 describes shifting target after battle; the artifact's every-decision timing must be named separately.

### Cluster lifecycle and FIFO export

`War.java:152–227` repeatedly merges clusters linked by current fighting; it does not split previously joined clusters. A merge adds damage, retains the earliest start, and adds the absorbed cluster's current members (`171–179`). Adding a member refreshes its shadow (`103–110`).

Structural change precedes cluster garbage collection (`Geosim2Model.java:1014–1021`). Annexation clears a conquered actor's war pointer (`611`); `War.trim`, lines 127–148, removes actors that are no longer sovereign and removes expired shadows. An inactive actor with positive shadow decrements it in this trim; removal occurs on a later trim after it reaches zero. Retained historical cluster damage is not split or reassigned because a member loses sovereignty.

`War.addWar`, lines 284–286, queues a completed event if raw severity is positive. `checkWarSize`, lines 309–315, peeks despite its misleading removal comment. `getWarSize`, lines 320–325, pops one FIFO event. `Geosim2Model.java:1023–1024` records once per tick when the queue is nonempty; `Geosim2Lab.java:260–265` retrieves one event. The outer framework member `src/ch/ethz/icr/growlab/collector/PowerLawSeriesCollector.java`, method `record`, evaluates its variable once. Consequently the inspected visual path exports one queued event per tick; backlogs can survive termination. A complete analysis exporter should drain explicitly and record terminal censoring, with the difference disclosed in artifact comparisons.

Severity conversion at `War.java:267–269,338–339` is `(int)(100.0 * res)`. Java floating-point-to-int narrowing truncates toward zero and saturates beyond signed-int limits; positive infinity/overflow becomes 2,147,483,647 and NaN becomes zero. It does not wrap modulo 32 bits. See the [Java Language Specification, narrowing primitive conversion](https://docs.oracle.com/javase/specs/jls/se25/html/jls-5.html#jls-5.1.3). A positive raw severity below 0.01 can enter the queue yet serialize to zero; `Geosim2Lab.java:404–413` excludes nonpositive serialized values from its legacy plotting list.

## 2026-10-03 scratch execution and premeasurement reference audit

The isolated scratch task executed unchanged bundled GeoSim2/GROWLab binaries on OpenJDK25.0.1. Original `Geosim2Lab batch` starts but throws a null-collector exception at its first export; `CallableSimulator.call` catches runtime exceptions, so the process still exits0. The collector is initialized only by `Geosim2Lab.buildUI`. An independently authored wrapper supplies the original framework `PowerLawSeriesCollector`, retrieves one `War.getWarSize` per record, and advances **lab** time before each step. It does not compile/patch the author model, replace RNGs, change settings, or execute a judge. Complete commands, raw errors, SHA256 member/dependency manifests and limitations are in `survey/out/geosim-reference-preparation.md` in the scratch worktree.

The wrapper's default seed5/horizon10500 observation contains249 exported severities, ends with12 sovereigns, zero queued completed events and zero active clusters. A prior full-run repeat had byte-identical stdout/stderr. Wrapper-specific tests pass and require an explicit horizon-completion record, not merely exit0. These are **later-port execution observations**, not published APSR reproduction or evidence of power-law fit. Full per-front/draw docking remains future work.

Additional source conventions: `Geosim2Lab.java:320–328` and outer `src/ch/ethz/icr/growlab/collector/PowerLawSeriesCollector.java:81–88` both use inclusive `size>=observed_size` survival ranks over the full stored list; duplicate events retain multiplicity. Lab `addPoint:404–413` filters nonpositive serialized sizes; outer framework collector does not. Lab cutoff `x1=2.5` applies to log10 of integer100severity. Regression selects `x>=x1` (`347–367`), but its R2 residual loop selects `x>x1` (`383–393`). Printed `linePoints` is eligible-point count; its cumulative denominator includes all positive stored events. The lab computes an `x2` maximum with minimum `x1+1`, but exports no named range statistic; this cannot establish the published Table1 range convention. The bridge runs the framework collector, not the separate GUI regression fitter.

The archive has an active-cluster list and completed FIFO, but no comprehensive terminal event/censoring ledger. `War.getNum` counts active clusters and `getNumWars` queued completions. Wrapper-added completion/time/queue/active counts cannot recover full censored participant/damage records. Finished-but-unexported versus still-active distinctions remain necessary in the reconstruction.

Official CoWv4 participant CSV and codebook were recovered, hashed and preserved in scratch output:337 participant rows,95 war IDs, onsets1823–2003. One participant death value is unknown (`-9`, Thailand, war170); unknown values cannot enter death sums as negative numbers or be silently imputed. Known-sum input endpoints match the Clauset main article's1000 minimum and16634907 WorldWarII maximum, but this is an integrity check, not a validated empirical severity series. Clauset SupplementaryS1 is identified in recovered primary JATS as `aao3580_SM.pdf`,609073 bytes; two groups of three publisher/repository/static access routes failed after reassessment. Precise S1 estimator/bootstrap settings remain **Unresolved**. Clauset's public GPL2 fitting toolkit was downloaded reference-only; MATLAB/Octave were unavailable and no fitting code was executed or transplanted. Three historical-input routes also failed; Cederman's1820–1997 input remains **Unresolved**, distinct from recovered CoWv4.
