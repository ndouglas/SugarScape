# GeoSim and the size of wars — brainstorming reading notes

Date: 2026-10-03. Status: research before design approval; no implementation or registered measurement. Builds on the published Milestone 36 polarity handoff, not its mechanics or study populations. The purpose is source-faithful reconstruction and findings, including failures, rather than a visually convincing heavy tail.

## Source hierarchy and scope

The main target is Cederman 2003, *APSR* 97(1), 135–150, “Modeling the Size of Wars: From Billiard Balls to Sandpiles,” local `papers/geopolitics/cederman-2003-apsr-modeling-the-size-of-wars.pdf`. Read the 2002 working paper and PNAS boundary paper as separately attributed predecessors. They do not jointly define an interchangeable default. The mechanics and author-code reader companions document detailed source/version checks.

- [Mechanics reading](2026-10-03-geosim-mechanics-reading-notes.md).
- [Author-code reading](2026-10-03-geosim-author-code-reading-notes.md).
- [Published polarity handoff](../../studies/2026-09-26-war-and-society.md#measured-polarity-handoff-2026-10-03).

The campaign also names the separate state-size/lognormal paper and Abramson’s polity census. Recommendation for discussion: keep those as contextual readings and a later independently specified empirical study; do not silently add terrain, historical calibration or democratic regimes to the APSR war-size default.

## Corrections to earlier campaign readings

APSR Table 1 p.143 row 3 is slope minimum −1.43, median −1.32, maximum −1.17. The October 1 notes transpose these values. Row 7 changes attack superiority to 2.5 and carries footnote a (shockSize10), not the base shock20 setting. The accompanying text on p.149 resolves the table label: this arm changes both superiority and victory thresholds to2.5 as well as shockSize10. It is not a single-factor sensitivity.

The October 1 claim that code exists needs a version-to-publication audit before any docking claim. A later archive naming APSR in JavaDoc does not by itself establish that its defaults, technology driver, cluster measurement or RNG match the reported runs. The recovered GeoSim2 port is a docking candidate: its collector exports accumulated damage multiplied by100 and cast to integer. Preserve raw damage and the attributed integer export as separate measures; the rounding/scaling rule can change cutoff, ties and fitting. GeoSim0 explicitly cites APSR but lacks this model’s shocks and war collector. GROWLab core is LGPL; the nested model has unresolved template copyright headers, so the full archive is not established as uniformly GPL. All archived code remains reference-only.

The literal published distance expression and decreasing-distance prose conflict; record both, with the justified default determined in the design. EPM Gaussian harvests, predator/status-quo labels and dyadic episodes cannot substitute for GeoSim resource relaxation, founder-grown states, contextual alerting and spatiotemporal wars.

## Published experiment and statistic definitions

APSR pp.140–143, Table1 and A1: counting starts after a500-period initial phase; runs end at10500. Each of11 Table1 rows has15 independently seeded histories. The sample run was selected for median R², not a representative slope; it has218 wars, slope−0.64, R².991, whereas base median slope is−0.55 and median wars204. Replicate the source’s per-history fits and min/median/max summaries before any pooled estimator.

War severity is accumulated battle damage over a spatiotemporal conflict cluster. The20-period shadow connects brief inactive intervals; spatial binding also requires conflictual interaction. It is neither a dyadic episode count nor simply participants or conquered cells. Cluster merging, splitting, surviving identity and unfinished wars at10500 need explicit algorithms and observables; the paper does not fully specify them. Retain censored clusters without treating them as completed wars.

Source regression is log cumulative frequency against log size. APSR p.141 footnote8 excludes log10 severity below2.5. The collector’s ×100 integer export means its scale must be reconciled with that cutoff rather than assumed; normalization/ties and the exact fitted observations still need author-tooling checks before the judge is frozen. Source slope is a cumulative slope, distinct from a probability-density exponent. Under an ideal unbounded continuous power-law tail, slope b corresponds to density alpha=1−b. This algebra does not equate differently truncated fits or empirical populations.

## Clauset and statistical follow-ups

Local sources: Clauset2018 *Science Advances* 4 eaao3580 and Clauset2019 arXiv1901.05086 (a review, not an independent replication of GeoSim). Clauset2018 pp.2–4/Fig2 analyzes95 CoW v4.0 interstate wars1823–2003, absolute battle deaths1000–16634907, without world-population or dyadic normalization. Its estimated tail has xmin7061 and density alpha1.53±.07; bootstrap95% interval1.37–1.76 and fitted-tail KS Monte Carlo p=.78±.03. These are dataset-specific reported results, not acceptance tolerances for GeoSim.

Primary sources:
- [Clauset2018 full article](https://pmc.ncbi.nlm.nih.gov/articles/PMC5834001/).
- [Clauset2019 review](https://arxiv.org/abs/1901.05086).
- [Clauset–Shalizi–Newman2009](https://arxiv.org/abs/0706.1062).
- [Authors’ fitting/testing code](https://aaronclauset.github.io/powerlaws/).
- [Official CoW v4.0 downloads and codebook](https://correlatesofwar.org/data-sets/cow-war/).

CSN uses maximum-likelihood fitting, fitted lower cutoff, KS goodness-of-fit with generated and refitted samples, and likelihood-ratio comparison against alternatives. Its author page supplies continuous and discrete methods and identifies which implementations are by the authors versus community ports. Do not classify data solely because values happen to be integers: fix the paper’s actual empirical approximation and the model’s raw resource-damage measure and author-port integer export separately. Verify the2018 Supplementary S1 details and recovered dataset aggregation before a literal replication judge. This research has not yet reconstructed that95-war input. Author `plfit.m` v1.0.11 and `plpva.m` v1.0.8 (both2012) were read from the companion page; each declares GPL2.0 and remains reference-only. `plpva` documents a continuous approximation for integer-valued inputs when their minimum exceeds1000, a default1000 Monte Carlo replications, and a configurable exponent grid for discrete estimation. These implementation settings need explicit attribution; do not substitute a package default silently. Their SHA256 hashes on2026-10-03 are `c1f92eb7b57dac9f0212a120cf647887c2b7e8e1076da3317de3cd38ff9be55f` (`plfit`) and `6740485dce069a6323f44d759c195755e0ed3d79f22038ee25ee5b33504ce131` (`plpva`).

A straight CCDF line or high R² does not test a power-law generative model. Nonrejection does not identify self-organized criticality, prove stationarity, or show superiority to lognormal/truncated alternatives. Within-history clusters are generated by one evolving geopolitical system; retain history IDs and distinguish the source per-history estimate, any pooled descriptive estimate, and whole-history uncertainty resampling. A cluster bootstrap would be our dependence sensitivity, not Clauset’s original iid protocol. Declared small-tail or failed-fit outcomes must remain unavailable/inconclusive rather than disappear from summaries.

## Direct conceptual critique

[Piepers2006](https://arxiv.org/abs/nlin/0606054), PDF p.5 §4, challenges Cederman on scope (intra-state versus inter-state casualty dynamics), an unidentified critical point, and technological change as a suitable slow driver. It supplies conceptual objections and a competing explanation, not a docking protocol or numerical acceptance cutoff. Record these objections as limits and mechanism questions. Its own illustrated power-law plots do not establish the competing mechanism either. Finite-grid scaling, shock/no-shock and activation/no-activation comparisons can diagnose the model; no one diagnostic establishes real-world causality.

## Recommended design boundary, pending discussion

A separate `geosim` kind is the leading option, preserving `polarity` outputs and goldens. Reuse generic host/RNG/clock/export infrastructure; share territorial primitives only if their source semantics actually match. Literal APSR presets and all Table1 arms, period batching, complete cluster ledgers and native ensembles would feed two explicitly separate reports: original regression summaries and modern tail/model-comparison evidence. Named ambiguities and frozen decisions precede measured runs. Large native studies stay outside CI; CI uses small deterministic mechanics, cluster-identity, estimator and native/WASM parity fixtures.

Before the spec: complete archive/source attribution, author collector’s scale versus source fit cutoff, cluster identity rules and printed distance reading. No new numeric acceptance thresholds, seed count increases, pooled judge or maximum workload are approved by these notes.
