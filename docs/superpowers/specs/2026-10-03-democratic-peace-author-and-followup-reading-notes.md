# Democratic peace — author-code and follow-up audit

Date: 2026-10-03. Research only. No recovered model was built or run, no code was transplanted, and no experiment or acceptance rule was registered.

## Author-code search and identity

The primary source is Cederman (2001), *Modeling the Democratic Peace as a Kantian Selection Process*, JCR45(4),470–502, DOI10.1177/0022002701045004004. The [author's publication page](https://icr.ethz.ch/publications/modeling-democratic-peace/) identifies strategic tagging, regime-sensitive alliances, and collective security as three distinct mechanisms. Printed p.478 explicitly says the model recodes the earlier Pascal specification in Java/RePast. The Pascal ancestor is not this paper's executable.

The [ETH GeoSim page](https://icr.ethz.ch/research/geosim/) describes a family for state formation, nationalism and democratic peace. The [GROWLab model list](https://icr.ethz.ch/research/growlab/models/) separately lists GeoSim0,2,4,5 and GeoContest. This family relationship does not identify the version behind any particular published experiment.

Targeted web searches for Cederman/GeoSim democratic-peace source and replication, followed by actual `gh search repos 'geosim Cederman'` and `gh search repos 'GeopolDem'`, returned no candidate repository in this bounded pass. Both CLI searches returned empty lists. This is a limited search outcome, not proof of nonexistence. No author was contacted.

The historical author SVN endpoint `http://cederman.ethz.ch/svn/java/ethz/trunk/growlab/` returned HTTP401. The old Gleditsch homepage `https://privatewww.essex.ac.uk/~ksg/` failed DNS resolution. A publisher copy of Cederman/Rao at `https://us.sagepub.com/sites/default/files/upm-binaries/2906_12jcr01.pdf` returned HTTP403; the existing local scan remained usable through OCR. Failures and successful local extraction evidence remain retained, not silently replaced.

## Recovered distribution inspected

Local archive: `papers/geopolitics/cederman-girardin-weidmann-icr-2017-growlab-0.9.5-source-geosim-geocontest.zip`. The earlier [GeoSim author audit](2026-10-03-geosim-author-code-reading-notes.md) supplies the framework/version/license history. This pass retained95 Java/plugin members from nested GeoSim2,4,5 and GeoContest archives, each bound by member path, size and SHA256 in the ignored research evidence. Extraction used known archive members solely for reference reading.

| Candidate | Evidence inspected | Interpretation |
|---|---|---|
| GeoSim2 | `Geosim2Model.java:27–51`, `:258–316`, `:340–347`, `:421–439`, `:911–952`, `:973–1024`; `Actor.java:480–492`, `:541–582`, `:709–791` | Includes democracy initialization, contextual regime change, alliance/pariah state and democracy measurements. Documentation links APSR2003 and a Cederman/Gleditsch conference precursor. It is a later combined model, not a verified2001 executable. |
| GeoSim4 | `Geosim4Model.java:37–69` | Describes a GeoSim2 derivative with nationalism, resource dependence and later extensions. Its comment explicitly says it does not feature democracies, despite inherited democracy-related material elsewhere. No2001 mapping is established. |
| GeoSim5 | `Geosim5Model.java:42–70`, `:263–315` | Cites2001 as a description of a variation, yet has later nationalism/state-formation defaults, initial polarity1000, and democracy/alliance switches initially false. Citation and dormant fields do not identify the published2001 runs. |
| GeoContest | `strategies/KantianStrategy.java` and model documentation | A separate strategic-competition implementation. A strategy named Kantian is insufficient to identify the2001 model. |

GeoSim2 has the later resource-before-allocation clock, alert/campaign state, logistic distance extraction, damage recurrence and optional technological change. The2001 source instead specifies225 initially independent actors, instantaneous exponential extraction after combat, and costless mutual-defection combat. Matching democracy flags would not reconcile these differences.

`Geosim2Model.java:1084–1088` implements an increasing superiority probability as `1/(1+exp(c*log(t/ratio)))`, with zero ratio returning zero. This supports a **later-artifact interpretation** of the2001 prose; it does not authorize silently replacing the2001 printed positive-exponent formula. Its regime-change and pariah mechanics similarly cannot be assumed to settle every2001 ambiguity.

Framework LGPL2.1-or-later, recovered GeoSim0 GPL2-or-later, and unresolved nested model template headers remain distinct licensing facts. No permission to transplant nested model source has been established. Original paper source version, RePast runtime/RNG, seed lists, event ordering and raw results remain unrecovered in this pass.

## Cederman and Rao2001: empirical context, separate protocol

*Exploring the Dynamics of the Democratic Peace*, JCR45,818–833, DOI10.1177/0022002701045006006. [Author publication record](https://icr.ethz.ch/publications/exploring-dynamics-democratic-peace/).

The local16-page PDF is scanned; ordinary text extraction mostly returns download watermarks. All16 pages were rendered at140dpi and OCRed with the existing Tesseract installation. OCR is a navigation aid: numerical/equation transcription would require visual verification before a future empirical reproduction.

Printed pp.822–824 describe a varying-coefficient GLM with logit link for binary dyad-year militarized disputes. Democracy is POLITYIII score>=6. Estimation covers1837–1992, excludes1914–1918 and1939–1945, and selects politically relevant dyads. The text reports1,974 of2,430 MIDs and52,276–68,764 dyad-year observations across the two specifications. Later controls include alliance and capability measures. The appendix discusses local likelihood, smoothing bandwidth and pointwise intervals.

This is empirical evidence about a historically varying association, not an additional GeoSim mechanism or a direct validation target for generated lattice histories. Reproducing it would require its actual dyad-year data, coding/filtering and estimation settings. An abstract reporting robustness does not supply those missing inputs.

## Cederman and Gleditsch2004: a mechanistic extension, separate protocol

*Conquest and Regime Change: An Evolutionary Model of the Spread of Democracy and Peace*, ISQ48(3),603–629, DOI10.1111/j.0020-8833.2004.00317.x. [Publisher record](https://academic.oup.com/isq/article-abstract/48/3/603/1823527). The local text-bearing PDF was extracted with `pdftotext -layout`.

The extension adds context-dependent transitions between democratic and authoritarian regimes. Its central qualification is that selection alone struggles to produce realistic systemic democratization; adaptive regime change with collective security can generate stronger expansion and spatial clustering. This narrows the interpretation of a successful2001 selection experiment rather than making it a universal historical explanation.

The paper itself warns that its results are not directly comparable to2001 (p.609, footnote10). Its protocol uses a50×50 lattice,200 founder-grown states,500 preliminary periods, then10% democratic states and10,000 observation periods (p.612). It reports20 replications per main configuration. The appendix's TableA1 lists mobile share.9, superiority/victory thresholds3 with exponents20, battle damage.1, and collective-security obligation probability.5. These differ from2001 and some later archived defaults; don't merge their settings.

Main configurations compare selection without regime change, contextual regime change, and contextual regime change plus collective security. Reported mean final democratic territory shares for the first two are.072 and.064 (pp.612–613); the collective-security treatment has larger variation and a stronger expansion effect. These are source reports, not our measured values. Footnote16 separately discusses collective security without regime change. The paper plots histories at500-step intervals and uses Moran's I for clustering, with a row-standardized neighborhood matrix (pp.618–620, footnote17). This is not the2001 survivor-conditioned size-weighted exposure ratio.

Before treating this as a later registered environment, visually verify its own formulas/defaults, retrieve the empirical transition calibration, and freeze its own statistic definitions. It should not expand the initial2001 milestone by implication.

## Critique search and coordination framing

Search surfaced a DomGeoSim lead titled *Domestic Structure, Learning, and the Democratic Peace*, associated with Maurits van der Veen and claims of corrections to Cederman's code. Only third-party transcription/book leads were found in this pass; a verified author-hosted paper, exact source archive and correction protocol were not recovered. It remains a retrieval lead, not an adopted critique or test specification.

The verified2004 follow-up provides a useful mechanism challenge: selection, adaptation and security obligations should be distinguished. The2001 agents have prescribed policies and regime tags, not learned preferences. Its substantive tests concern the survival/expansion of cooperation, alliance protection and their dependence on resource allocation. Absence of democratic–democratic attack is partly built into the rules and would be a tautological success criterion by itself.

For the coordination research program, the model can examine how partner tags, local pooling and enforcement affect systemic outcomes. It supplies no direct evidence about AI intent, learned cooperation or real-world democratic causality.

## Retained research evidence

Main-checkout ignored root: `/Users/nathan/Projects/ndouglas/SugarScape/survey/out/geosim-democratic-peace-research/`.

- `sources/archive-members.json`: archive identity and95 inspected member hashes.
- `sources/retrieval-attempts.json`: actual HTTP/DNS failure receipts.
- `sources/geosim2/`, `geosim4/`, `geosim5/`, `geocontest/`: reference-only source excerpts.
- `sources/cederman-gleditsch-2004.txt`: local primary-paper extraction.
- `sources/rao-ocr/`:16 rendered pages, individual OCR and concatenation.
- `reader/`: visually inspected2001 primary-paper pages.

Reopening original-code docking requires a verifiable version-to-publication mapping, executable/runtime/RNG identity, settings and source outputs. A later archive with matching subject matter alone does not meet that condition.
