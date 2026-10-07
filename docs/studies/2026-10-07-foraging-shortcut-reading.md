# F5 shortcut comparison: preliminary source reconciliation

**Date:** 2026-10-07. **Status:** source discovery and static analysis only. Complete biological methods/data reconciliation is pending; no comparison protocol, implementation or scientific execution is approved by this note.

**Engineering baseline:** F4 was locally merged at `568a3fa` and documented at `a2ebf4f`, with fresh workspace 2,852 passed/0 failed/103 ignored, formatter and core all-target Clippy clean. This isolated `foraging-5-design` worktree starts at main `0d2bc3b`, preserving the subsequent shared-surface experiment. F4's private-worker/researcher boundary, tagged direct transport and finite bounded episodes remain the starting point. [Programme](../superpowers/specs/2026-10-04-foraging-construction-design.md), [F4 design](../superpowers/specs/2026-10-06-foraging-4-construction-design.md).

## Primary sources and inspection boundary

- Michael, Chouvenc, Su and Lee (2023), *Finding shortcuts through collective tunnel excavations in a subterranean termite*, Behavioral Ecology 34:354–362. [Publisher](https://academic.oup.com/beheco/article/34/3/354/7059060), DOI 10.1093/beheco/arad007. Indexed methods/discussion were inspected; direct tool retrieval redirected to an unavailable publisher CDN. Figure geometry and supplementary images have not been inspected. This improves method detail over the [earlier reading](2026-10-04-foraging-construction-reading.md), without claiming unrestricted full-text access.
- [Authors’ Dryad deposit](https://datadryad.org/stash/dataset/doi:10.5061/dryad.9s4mw6mmn), DOI 10.5061/dryad.9s4mw6mmn, published 2023-01-19. The public landing page lists `data.xlsx` (17,146 bytes) and `README.md` (1,190 bytes). Page download links returned HTTP 403; the [documented public API route](https://github.com/datadryad/dryad-app/blob/main/documentation/apis/README.md) returned HTTP 401. Workbook/README contents were not obtained or audited. No authentication or access-control workaround was attempted.
- [Authors’ Zenodo material](https://zenodo.org/records/7552864), DOI 10.5281/zenodo.7552864. Public metadata and `R_Figure_Script_Michael.R` were downloaded and statically inspected. The 14,654-byte script matched its declared MD5 `e229a9ec6b2265dd87a371ea2e845bdd`. It was not executed; its package-install statements were not run.

Retrieved public material, hashes and retrieval outcomes are preserved in `/tmp/sugarscape-f5-source-audit-20261007/`. The original R script is retained there, not copied into SugarScape source. Its deposit identifies the Dryad data as a related source.

## Method facts inspected

Five colonies (~3,000 workers each) shared straight/detour/twisting arenas through a distributing chamber. Interiors were 18 × 18 × 0.2 cm; preformed tunnels were 3 mm wide. Straight length was 15 cm; detour segments were 3/15/15/12 cm, with twisting corners at 3 cm intervals. Branching, shortest distance and width were sampled every six hours for 24 hours; early branching at 2/4/6 hours. Width averaged ten sections. Analyses used colony-aware Poisson mixed models for branching and ANOVA for distance/width. Food-pad size differs: methods 3 cm, Figure 1 caption 2 cm. Figure geometry and that discrepancy remain unresolved. [Publisher methods](https://academic.oup.com/beheco/article/34/3/354/7059060).

## Static script findings

The deposited script reads `Rawdata.csv` at line 3 and `Ntunnel1.csv` at line 133, while the Dryad landing page advertises an Excel workbook. A mapping from workbook sheets to these CSVs cannot be established without reading the workbook/README. `data2` is used at line 28 without an initialization anywhere in the deposited file; `Figure_alt12` is referenced at line 71 without a definition. Its initial working directory is an author's Windows path. Therefore the script alone is not a clean-session reproducer of the reported analysis. This is a static inspection finding, not a failed reproduction run or an assertion that the data are wrong. [Deposited script](https://zenodo.org/records/7552864/files/R_Figure_Script_Michael.R).

## F4 adaptations that need explicit treatment

The accessible biological endpoints describe network geometry; they do not establish measured delivered-food benefit or energy conversion. F4 can test realized token delivery and paid work as additional computational outcomes, with that distinction explicit.

F4 supports at most 256 workers, two-worker cell capacity, finite nonregenerating food tokens, monotone opening and direct spoil delivery to an outlet. It supplies no calibrated conversion from grid cells/ticks to centimeters/hours, no tunnel refill, no continuous cellulose consumption and no biological controller inferred from the experiment. Independent simulation worlds do not reproduce simultaneous colony allocation among three connected arenas. Local wall learning and uniform/private face selection differ from hypothesized corner cues. Variable tunnel width and branching measurements also require an explicit grid interpretation before numerical comparison. These are model/source differences, not parameter values to infer silently from a successful engineering fixture.

Because F4 keeps food-access records at original resource positions after delivery, their cached current nest distances can describe structural route shortening separately from realized pickup/delivery. An initially accessible food site has no first-access event: its initial distance and subsequent current distance are the relevant comparison. No hidden physical distance may guide workers.

## Proposed first comparison, pending design discussion

The working recommendation is a mechanism-oriented test of whether paid excavation can improve finite-horizon food return in F4. It would use common nest, outlet, food patch, worker spawns, explicit fixed controller parameters, horizon and predetermined seed blocks across geometry-inspired straight/detour/twisting layouts.

For each layout, compare paid diggable terrain with the same initial open route and protected walls, plus an already-open shortcut reference. The reference separates route availability from construction expenditure; it is not assumed to maximize realized delivery. Changing diggability changes locally observable eligibility, which is part of the intervention. It must not introduce omniscient guidance or pretend identical seed labels preserve aligned later draw streams.

Candidate outcomes are delivered food by cutoff; pickup/delivery timing with censored failures; current shortest physical distances; excavation, food/spoil carrying, disposal and congestion work. Report components separately rather than inventing an energy-weighted net score. Protected or unfinished food/cargo remain recorded, and all worlds execute the same full horizon. No preferred seed, optimized parameters or guaranteed shortcut/delivery result is selected here.

A geometry-only benchmark is a smaller alternative, but would not establish the delivery/cost question. Quantitative biological reproduction is a larger alternative requiring recovered data, geometry/width/backfill/feeding definitions and calibration beyond current F4. The user has been asked which primary claim to pursue; no answer is presumed by this note.

## Remaining source/design gates

Recover and inspect the workbook and README through authorized public or user-provided access; reconcile sheets, units, colony identifiers, time points, missing entries and CSV mapping. Inspect Figure 1/supplementary geometry and resolve pad-size and twisting-path ambiguities before any source-faithful discretization. Separate seeded simulation replication from biological colony replication.

The brainstorming design discussion, written-spec review, implementation plan and scientific-protocol authorization retain their separate gates. This preliminary source audit does not enable a campaign run. Dedicated roles and relay transport remain deferred as the user requested.
