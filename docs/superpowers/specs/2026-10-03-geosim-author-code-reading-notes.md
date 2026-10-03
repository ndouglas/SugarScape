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
