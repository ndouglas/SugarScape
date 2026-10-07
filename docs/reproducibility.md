# Reproducibility register

This is the persistent working list of reported discrepancies, unresolved source identity,
measurement gaps and corrected problems in SugarScape. Backfilled **2026-10-06** from the
[paper catalog](papers.md), the README's scientific results and linked dated evidence.
It covers the **31 completed catalog groups** (1–8 together, then 9–38), with additional
Minds and foraging/construction coverage below. It is an inventory of existing documentation,
not a new execution, publication audit or claim that a paper is false or irreproducible.
The original findings, figures, source trees, binaries and archives remain authoritative.

## How to read an entry

IDs are permanent. The subsection names the paper, model and milestone for every row beneath it.
Each row names a target, the documented outcome and its scope, an issue category/status and
evidence level, and a concrete next action. The linked record in the status cell supplies the
canonical evidence. All entries were added/reviewed for this backfill on **2026-10-06**;
evidence dates belong to their linked records. An entry is not a publication-ready verified
claim unless its evidence level and source-identity work support that use.

| Category | Meaning |
|---|---|
| Mismatch | Measured disagreement conditional on the named reconstruction, metric and sample |
| Reading | Specified-rule conflict, interpretation or parameter/statistic sensitivity |
| Provenance | Original code, data, runtime, RNG, settings or statistic equivalence unavailable/unverified |
| Question | A claim or causal explanation has not been measured with an adequate corresponding test |
| Local correction | Our implementation, reporting or claim interpretation was corrected; history retained |
| Lead | Reading/retrieval only; no reproduction verdict |
| Own hypothesis | Our extension/control hypothesis failed or is weak; not a failure of a source paper |

Statuses are **Open** (a concrete unresolved task), **Blocked on source** (missing source inputs),
**Not measured**, **Resolved** (the specified local problem is closed), or **Superseded** (a newer
study replaces the earlier availability limitation). A resolved reading does not resolve other
claims about the model. `Fails`, `Weak`, `Compatible`, `Incompatible`, `Inconclusive`, `Holds`,
`Unresolved` and `Untestable` quoted below are the linked study's verdicts, not new judgments.

| Evidence level | Strength and limit |
|---|---|
| D | Documentary/source reading only; no new measured test |
| L | Legacy README/survey comparison; sample and choices as documented, not freshly audited against original source/raw output here |
| A | Dated retained numerical audit or independent reconstruction; source equivalence and retrospective choices still limited as stated |
| R | Registered frozen study with retained findings/provenance; conditional reconstruction, not automatically original-code equivalence |
| X | Explicit original-code docking or analytic identity for the named target only |

## Triage and ongoing updates

1. Start with source/metric clarification: distinguish a single illustrated run from a frequency,
   our proxy from the published statistic, and source identity from conditional compatibility.
   The claim/evidence audit in each legacy row comes before a new run or scholarly allegation.
2. Prioritize the current geopolitics source-identity and unavailable-statistic gates
   (RR-EP-04/05, RR-GS-02/03/04, RR-DP-04/05). More resampling alone does not repair those gates.
   Keep defense/alliance positive hypotheses separate from reproduced negative source directions.
3. Review documented high-impact quantitative mismatches next: RR-SS-01, RR-CV-01,
   RR-DPD-01/02, RR-PUN-01, RR-FIRM-02/03, RR-COL-02 and the registered auction endpoints.
   Preserve original rules, seeds, judges and outputs; declare any follow-up as a new study.
4. Revisit reading sensitivity before expanding workloads: ethnocentrism's TPB source,
   norms ties, image-scoring horizon/observer conventions, retirement age norms, hoarding
   detection/defense and democratic-peace printed/prose probability. Never select a reading by fit.

Whenever a scientific study reports a new problem, add or update its claim-level entry with
paper/model/milestone, precise target, observed result and scope, evidence link/level, date,
category/status and next action. Cross-link this register from the corresponding catalog entry
or dated findings. Keep stable IDs and resolved/superseded history; append the later evidence and
explain what it changes. Separate distinct causes and next actions. Queued papers and untested
variants stay leads/questions. No entry authorizes a rerun, source transplant or alteration of a
frozen scientific identity.

## Completed catalog coverage

Every catalog group was inspected. “No open issue found” means no actionable discrepancy in the
examined documentation, not certification of all claims in a paper. Successful controls remain
visible here so the denominator is not only failed claims. IDs below index the detailed rows.

| Milestone | Paper/model scope | Disposition and retained successful comparison | Register IDs |
|---|---|---|---|
| 1–8 | Epstein–Axtell 1996, Sugarscape II–VI | Capacity/skew/VI-3 supported; disease, fertility, VI-2 and waves remain conditional questions/mismatches | [RR-SS-01–05](#rr-ss-01) |
| 9 | Book Schelling variant and Ring World | Segregation/flocking reproduced qualitatively; flock/segregation measures not defined by source | [RR-BOOK-01](#rr-book-01) |
| 10 | Janssen 2009 / Axtell et al. 2002, Anasazi | Published Janssen shape approximately recovered; documented model and valley abandonment unresolved | [RR-ANA-01–03](#rr-ana-01) |
| 11 | Epstein 2002, civil violence | Rounded Model I and some Model II directions recovered; literal rule/timing/peacekeeping differ | [RR-CV-01–07](#rr-cv-01) |
| 12 | Riolo et al. 2001; Edmonds–Hales / Roberts–Sherratt, tags | Current-agent ties reproduce tables; local “unstated” claim corrected; dynamics and critique differ | [RR-TAG-01–05](#rr-tag-01); [RR-FIX-01](#rr-fix-01) |
| 13 | NM92 / HG93 / NBM94, spatial games | Kaleidoscope/constant/thresholds and HG93's actual regime recovered; generalizations and readings limited | [RR-SP-01–05](#rr-sp-01) |
| 14 | Axelrod 1997 / docking 1996, culture | Docking/soup recovered; sample discrepancy within documented sampling error | [RR-CULT-01–03](#rr-cult-01) |
| 15 | AEY 2000/2001 / Poza et al. 2011, classes | Equity/error-rate directions recovered; fractious/classes/transitions differ | [RR-CLASS-01–04](#rr-class-01) |
| 16 | HA06 / Hartshorn et al. / Jansson 2013, ethnocentrism | Many baseline/HKS/kin-neighbor claims recovered; Table 1 and follow-up particulars differ | [RR-ETH-01–08](#rr-eth-01) |
| 17 | HK02 / Lorenz 2006, opinions | Lattice/serial/consensus directions recovered; two-camp typicality and timing limited | [RR-HK-01–03](#rr-hk-01) |
| 18 | Cohen et al. 2001, structure | Table 2/context preservation recovered; implied threshold/noise readings need attribution | [RR-STRUCT-01–03](#rr-struct-01) |
| 19 | Epstein 1998/GSS / Radax–Rengs, demographic PD | Spatial cooperation/soup/per-game identity recovered; tables/cycles/metabolism differ | [RR-DPD-01–08](#rr-dpd-01) |
| 20 | Axelrod 1986 / Galán–Izquierdo 2005, norms | Original 100-generation conclusions recovered; million-generation persistence is reading dependent | [RR-NORM-01–02](#rr-norm-01); [RR-FIX-02](#rr-fix-02) |
| 21 | NS98 / LH01, image scoring | Analytic constant/standing and some cycles recovered; horizon, observer, island and finite-sample comparisons limited | [RR-IMG-01–10](#rr-img-01) |
| 22 | Deffuant lineage / Meadows–Cliff / 2013 reply, agreement | Reply's two adjustments and lattice example recovered; printed BC/statistic/network details differ | [RR-RA-01–06](#rr-ra-01) |
| 23 | Arthur / CZ97 / SMR / CZ98 / CMO, farol | Constructed-library mean and qualitative memory transition supported; historical bank/shape/purity limited | [RR-EF-01–06](#rr-ef-01) |
| 24 | Kirman / Alfarano–Milaković, ants | Exact chain/asymmetry/switches recovered; 80–20 interpretation and network N/update scope limited | [RR-ANT-01–04](#rr-ant-01) |
| 25 | Granovetter / Watts, thresholds | Exact crowds/continuous jump/cascade window recovered; upper critical/degree/hub readings unresolved | [RR-TH-01–04](#rr-th-01) |
| 26 | Axtell–Epstein / GSS, retirement | Qualitative sensitivity/coupling supported by dated audit; age norm, policy and reconstruction choices limited | [RR-RET-01–04](#rr-ret-01) |
| 27 | Boyd et al. / Cooney / Janssen, punishment | Shape directions recovered; reach/conflict/traits/calibration and PDE attribution differ | [RR-PUN-01–08](#rr-pun-01) |
| 28 | Gode–Sunder / Cliff, ZI | Efficiency/dispersion with sufficient shouts and ZIP convergence recovered; timing/predictions/code-text limited | [RR-ZI-01–04](#rr-zi-01) |
| 29 | Lansing–Kremer / Janssen, Bali | Table 1/imitation/two-node threshold recovered; network resemblance/perturbation/scale/extensions differ | [RR-BALI-01–10](#rr-bali-01) |
| 30 | Schelling 1971 board and line | Direction and many hand-example ranges recovered; typical quantitative outcomes/line claims differ | [RR-S71-01–06](#rr-s71-01) |
| 31 | Schelling 1969/1971 bounded neighborhood, tipping | No open discrepancy found: all documented deterministic schedules/tipping claims recovered | No open issue found; [record][tipping] |
| 32 | Pancs–Vriend / Gauvin et al. / Singh et al. / Zhang | Small-city/phase/ring/Ising identity recovered; spiked and Zhang waiting/β/speed differ | [RR-VAR-01–04](#rr-var-01) |
| 33 | Axtell 1999 / 2013, firms | Table 1 analytics and later Zipf supported; 1999 distributions/lifetimes/rules differ | [RR-FIRM-01–11](#rr-firm-01); [RR-FIX-03](#rr-fix-03) |
| 34 | Calvano et al. / critics, collusion | Exact 100-session docking and code tables recovered; prose equilibrium/deviation/generalizations differ | [RR-COL-01–07](#rr-col-01) |
| 35 | Banchio–Skrzypacz / follow-ups, auctions | Registered baseline/feedback directions Hold; four endpoint Fails, one coverage Inconclusive | [RR-AUC-01–07](#rr-auc-01) |
| 36 | Cederman 1994/1997 / adaptations, polarity | 45 Compatible, 24 Incompatible, 3 Unresolved source rows; negative directions distinct from P2/P3 Fails | [RR-EP-01–06](#rr-ep-01); [RR-FIX-04](#rr-fix-04) |
| 37 | Cederman 2003 / precursor / tail follow-ups, geosim | 10 Unresolved, one Incompatible conditional comparison; exact identity unresolved | [RR-GS-01–06](#rr-gs-01); [RR-FIX-05](#rr-fix-05) |
| 38 | Cederman 2001 / later context, democratic peace | Historical 3,240 complete; new 21,600 complete; conditional mismatch and reading dependence distinct | [RR-DP-01–06](#rr-dp-01); [RR-FIX-06–08](#rr-fix-06); [RR-LEAD-01–03](#rr-lead-01) |

## Catalog claim-level entries

### Milestones 1–8 — Epstein and Axtell 1996, `sugarscape`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-ss-01"></a>RR-SS-01 | Mismatch / Open / L; [record][sugar] | VI-2 extinction versus VI-3 doubling. Documented VI-3 follows the curve; VI-2 also recovers. The 216-configuration/seeds 1–5 search found only a knife-edge separation; 16 later switch combinations rarely separate (at most 1/20). Unreported software details are a hypothesis. | Audit original VI-2/3 software/settings or archive; preserve the current stated-rule mismatch and both search scopes. |
| <a id="rr-ss-02"></a>RR-SS-02 | Reading / Open / L; [record][sugar] | V-2 endemic infection: footnote-16 per-agent learning/next-tick cure clears V-1 and V-2 in 20/20; per-disease reading leaves residue and endemic V-2. VI-1 flares then clears between outbreaks. | Verify learning/cure order against original disease implementation before any mechanism claim. |
| <a id="rr-ss-03"></a>RR-SS-03 | Mismatch / Open / L; [record][sugar] | IV-14 fertility ends: `iv-15-trade-sex` dies out in 15/20 under women 35–45/men 45–55 and stated rules. | Recover corresponding source population/age/endowment and fertility setup; distinguish reproductive extinction from VI-2. |
| <a id="rr-ss-04"></a>RR-SS-04 | Question / Open / L; [record][sugar] | II-6 waves: README Minds 2 reports only 0.8% beyond 25 cells under jumping at tick 100 versus a project quarter-population target; walking 0.6% does not rescue it. This is not a digitized source threshold. | Audit original wave animation, starting block and an explicit source-compatible propagation metric; see also [RR-M2-01](#rr-m2-01). |
| <a id="rr-ss-05"></a>RR-SS-05 | Reading / Open / D; [record][sugar] | Two-good “total wealth” is undefined by the book; repository sums holdings. Native/browser floating-point differences are documented for trade/multiple goods. | Record the statistic definition and execution platform with any exact numerical comparison; do not equate browser/CLI last-bit disagreement with source failure. |

### Milestone 9 — Epstein–Axtell book variants, `schelling` and `ring`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-book-01"></a>RR-BOOK-01 | Reading / Open / L; [record][book] | Ring World flock means depend on our “at most one empty site apart” count; source defines none. VI-5 residence segregation about .76 versus VI-4 .63/VI-6 .95 cannot directly judge the book's unquantified “comparable” degree. | Specify source-comparable flock/segregation measurements before claiming numerical agreement or disagreement; retain qualitative successful directions. |

### Milestone 10 — Janssen 2009/2013 replication of Axtell et al. 2002, `anasazi`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-ana-01"></a>RR-ANA-01 | Mismatch / Open / L; [record][anasazi] | Calibrated documented rules yield plateaus 41/80 households and 7/15 empty by 1350, versus published-replication 172/180. Fresh offspring corn rather than parent's third materially changes fit. | Compare each of the ten named replication quirks against Janssen's ODD/code and published fit independently. |
| <a id="rr-ana-02"></a>RR-ANA-02 | Mismatch / Open / L; [record][anasazi] | Archaeological abandonment: published reconstruction retains 59 households in 1300 and 22 in 1350 where record reaches zero (seeds 1–15). It recovers Janssen's reported failure to empty the valley. | Separate reproduction of Janssen's curve from its fit to archaeology; audit abandonment inputs and source residuals. |
| <a id="rr-ana-03"></a>RR-ANA-03 | Provenance / Blocked on source / D; [record][anasazi] | These presets approximate Janssen's NetLogo replication, not original 2002 executable equivalence; water/hydrology/drought mappings include replication-derived details. | Retrieve original implementation/settings/output mapping before describing the 2002 model as docked. |

### Milestone 11 — Epstein 2002, `civil`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-cv-01"></a>RR-CV-01 | Mismatch / Open / L; [record][civil] | Literal P=1−exp(−k C/A), Run 2: no >50-active outbursts in 20 seeds ×3,000 ticks (peaks 13–34); floor(C/A) yields 101–122/peaks 290–368. Tested schedule/vision alternatives do not rescue literal behavior. | Recover original arrest implementation and Run 2 settings; label rounded success as a departure. |
| <a id="rr-cv-02"></a>RR-CV-02 | Mismatch / Open / L; [record][civil] | Rounded Run 2 waiting times average 22 ticks versus source 60±55; activation amplitude is compatible. | Audit cycle normalization, outburst/end definitions and jail timing separately from amplitude. |
| <a id="rr-cv-03"></a>RR-CV-03 | Mismatch / Open / L; [record][civil] | Salami versus jump: peak direction holds 17/20 but larger final jail only 7/20. | Verify jail observation horizon and source comparison before extending the successful peak result. |
| <a id="rr-cv-04"></a>RR-CV-04 | Reading / Open / L; [record][civil] | Run 8 prose “stable, nasty” versus genocide in 20/20 at t=75–882; source Fig. 15 also shows rapid genocide. | Reconcile prose/figure before treating the illustrated trajectory as a stable-state claim. |
| <a id="rr-cv-05"></a>RR-CV-05 | Mismatch / Open / L; [record][civil] | Peacekeeping across densities 0–.1: all 220 runs reach genocide within 3,000; longest 1,596, versus source delays past 15,000. Delay/spread directions recover. | Audit reproduction/death/clone clock and source long-horizon setup before measuring long-delay tails. |
| <a id="rr-cv-06"></a>RR-CV-06 | Reading / Open / D; [record][civil] | Worked outbursts 60+100+120+95+80 sum 455/mean 91, not printed 500/100. Tests use 455. | Preserve arithmetic correction and verify whether source activation statistic used additional unprinted events. |
| <a id="rr-cv-07"></a>RR-CV-07 | Mismatch / Open / L; [record][civil] | Safe havens with peacekeepers at t=50 still reach genocide in 20/20; denser forces only delay it. | Recover source deployment placement/timing and haven definition separately from baseline Run 8. |

### Milestone 12 — Riolo–Cohen–Axelrod 2001; Edmonds–Hales and Roberts–Sherratt, `tags`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-tag-01"></a>RR-TAG-01 | Reading / Resolved / L; [record][tags] | Current-agent tie rule is explicit on p. 442 and reproduces tables. Coin-flip reading at P=2 yields 42% versus source 4.3%; presets use current, config default remains coin flip. | Retain the resolved source reading and audit preset/default attribution; see RR-FIX-01. |
| <a id="rr-tag-02"></a>RR-TAG-02 | Mismatch / Open / L; [record][tags] | Cluster takeover dynamics: median 29 takeovers/30,000 generations versus two in illustrated 500; dominance 86% versus 75–80%, takeover exact-tag 91% versus 79%, final tolerance .039 versus .027 (20 seeds). Our cluster/takeover definition is not specified by source. | Recover source cluster definition and treat Fig. 1 as one realization; compare frequencies only after matching measurement. |
| <a id="rr-tag-03"></a>RR-TAG-03 | Mismatch / Open / L; [record][tags] | Edmonds–Hales N=200 vanished-donation statement: 3/20 runs never cooperate; others average 74%, under current ties. | Retrieve critique's tie/init/horizon and sample-specific claim; do not assign this to RCA's table claim. |
| <a id="rr-tag-04"></a>RR-TAG-04 | Reading / Open / L; [record][tags] | Adoption probability has no printed scale: using b+c recovers 49% at P=1 (.488 measured); generation score range gives 7%. | Verify original adoption normalization from source code or supplementary description. |
| <a id="rr-tag-05"></a>RR-TAG-05 | Reading / Open / L; [record][tags] | Exact clones force donation under ≤ and T≥0; strict/noisy/negative-tolerance variants collapse to 1–5%; zero tolerance with coin-flip ties gives 75.3%. Roberts–Sherratt and Edmonds–Hales raise distinct clone-mechanism critiques. | Distinguish forced exact-tag donation from tolerance selection and audit each critique’s own tie/controller assumptions. |

### Milestone 13 — Nowak–May 1992, Huberman–Glance 1993, NBM1994, `spatial`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-sp-01"></a>RR-SP-01 | Mismatch / Open / L; [record][spatial] | NM92 four-neighbor cooperation .379–.382 over t 501–1,000 versus ~.374; outside project .005 margin. | Verify source precision, edges, start and horizon before interpreting the small difference. |
| <a id="rr-sp-02"></a>RR-SP-02 | Reading / Resolved / L; [record][spatial] | HG93 single-defector takeover reproduces at b 1.9 (all D t 56–149); it does not generalize below that regime (b 1.7 f_C .974–.999). Legacy survey's “always” Fail is broader than HG93's actual starting-state claim. | Narrow the claim to HG93's tested regime, retain NBM94's counterexamples to the broader conclusion. |
| <a id="rr-sp-03"></a>RR-SP-03 | Mismatch / Open / L; [record][spatial] | NBM94 m 1 without self-interaction retains f_C .16–.33 at b 1.05; disappearance holds at b 1.13/1.35. | Verify low-b source horizon/lattice/start before testing persistence rather than finite survival. |
| <a id="rr-sp-04"></a>RR-SP-04 | Reading / Open / L; [record][spatial] | Random-array rc≈9 needs unreported 50% initial defectors; at 10%, no radius≤11 ends all D. Arena/cube initial settings are inferred or project-chosen; hexagonal variant is not implemented. | Recover original initial conditions; mark hexagons unmeasured, not failed. |
| <a id="rr-sp-05"></a>RR-SP-05 | Mismatch / Open / L; [record][spatial] | NBM94 m 1 discrete thumbnails retain C at b 1.9/2.01 where our runs are all D; captions omit shown b 1.77. Small deterministic lattice has unreported finite-seed collapses at b 1.9. | Audit caption/value mapping, lattice size and illustration horizon; avoid treating one thumbnail as typical-frequency evidence. |

### Milestone 14 — Axelrod 1997 and Axtell et al. 1996 docking, `culture`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-cult-01"></a>RR-CULT-01 | Reading / Open / L; [record][culture] | Sample setup mean 4.33/median 4/18% above six (1,000 seeds) versus 3.2/3/10%. README says original small samples are within sampling error; scan-feature choice changes nothing. | Verify original sample sizes and sampling comparison; retain as a discrepancy to audit, not a confirmed failed claim. |
| <a id="rr-cult-02"></a>RR-CULT-02 | Reading / Open / L; [record][culture] | Mobility docking gives 1.15/2.25 cultures with σ=25, height 4; many broad/tall mountains fit, narrow/low leave 24–42. Paper specifies no mountain width/height. | Retrieve exact landscape before interpreting the matching presets as unique reproduction. |
| <a id="rr-cult-03"></a>RR-CULT-03 | Reading / Open / L; [record][culture] | “Larger territories fewer regions” holds at source traits 15 but reverses at 25 (medians 207 at 20², 424 at 30²); Castellano et al. supply transition context. | Bound original qualitative claim to trait regime; separate later phase-transition extension. |

### Milestone 15 — Axtell–Epstein–Young 2000/2001 and Poza et al. 2011, `classes`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-class-01"></a>RR-CLASS-01 | Mismatch / Open / L; [record][classes] | AEY fractious attractor never reached from random starts; planted lasts 1–10 periods, all equity by 53, payoff ~18 versus 25 and claimed >10⁹ persistence (20 seeds). | Audit best-response/memory/activation against source implementation; do not fit an alternate rule silently. |
| <a id="rr-class-02"></a>RR-CLASS-02 | Mismatch / Open / L; [record][classes] | Fig. 4 transition at m 13, ε=.1 median 600 versus >10⁵ periods. | Verify period normalization and transition stopping criterion separately from attractor stability. |
| <a id="rr-class-03"></a>RR-CLASS-03 | Mismatch / Open / L; [record][classes] | AEY classes/equity-above division-below appear 0/20 under stated parameters, including Poza's small setup; planted classes persist 18/20 at 20,000. Mode-rule/lattice departures can create classes. | Reconcile original and Poza memory/decision/setup before using successful departures as source reproductions. |
| <a id="rr-class-04"></a>RR-CLASS-04 | Mismatch / Open / L; [record][classes] | Poza empty/growing-memory initialization reportedly lengthens transition; documented comparison finds no difference. | Retrieve follow-up's exact initial memory and transition statistic/sample and review the legacy comparison. |

### Milestone 16 — Hammond–Axelrod 2006 and Jansson 2013, `ethno`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-eth-01"></a>RR-ETH-01 | Reading / Open / L; [record][ethno] | Appendix mutation .05 versus text/Table 1/code .005, twice-directed interaction versus once, Java five colors/full start/no immigration versus four/immigration. These give different paths even when final outcomes agree. | Maintain a versioned rule/source map; verify which archived Java actually generated each reported table. |
| <a id="rr-eth-02"></a>RR-ETH-02 | Mismatch / Blocked on source / L; [record][ethno] | Doubled cost: seeing 64.7% versus 56%, blind 41.8% versus 14%; tested blind readings don't jointly recover either. HA06 attributes comparison to its unread TPB 2006 precursor. | Read/verify the TPB controller/game/setup first; no caption alleging the 14% figure is wrong until then. |
| <a id="rr-eth-03"></a>RR-ETH-03 | Mismatch / Open / L; [record][ethno] | Table 1 row l after 500: 57.3% ethnocentric versus 73.9%; takeover appears later. HKS around 300 is median-compatible but broad 21–596. | Audit row l averaging horizon/start/version separately from HKS sample variability. |
| <a id="rr-eth-04"></a>RR-ETH-04 | Reading / Open / L; [record][ethno] | Each-color “80% ethnocentric”: 27% help own only versus 84.3% own and refuse at least one other. | Resolve source strategy/category definition before numerical grading. |
| <a id="rr-eth-05"></a>RR-ETH-05 | Mismatch / Open / L; [record][ethno] | Table 1 c/g/i/j/k exceed 3-point margin: mutation 1% 63.0 versus 67.1, immigration 2 70.5 versus 74.4, 25 × 25 lattice: 64.4 versus 70.5; cost 2 and large-grid cooperation differ. Mean cooperation +2.4 points. | Audit each row's sample/time/code variant and uncertainty; treat broad bias separately from doubled-cost precursor. |
| <a id="rr-eth-06"></a>RR-ETH-06 | Provenance / Open / L; [record][ethno] | Table 1 fits 2/4/8 and code 2/5/9 colors similarly; cannot identify four versus five from final shares. | Recover exact source tag draw/runtime semantics rather than selecting by table fit. |
| <a id="rr-eth-07"></a>RR-ETH-07 | Reading / Open / L; [record][ethno] | Jansson kin Table 5: mutating basis 52.1% kin / 26.7% tag versus 76.2/16.4; fixed basis 65.5/12.6. Basis inheritance unspecified; color-gap convergence near 12 or 40 differs from 36. | Retrieve basis inheritance/mutation and kin classification; audit follow-up separately from HA06 baseline. |
| <a id="rr-eth-08"></a>RR-ETH-08 | Mismatch / Open / L; [record][ethno] | Jansson high tag mutation: at 60%, traitors 20.8 versus ethnocentrics 21.8 (pass at 75); at 90%, traitors 39.8 versus humanitarians 43.6, contrary to reported order. | Verify source strategy/basis rules and final-window averaging before claiming high-mutation ordering. |

### Milestone 17 — Hegselmann–Krause 2002 and Lorenz 2006, `opinions`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-hk-01"></a>RR-HK-01 | Mismatch / Open / L; [record][opinions] | Fig. 2b “fairly typical” two camps at ε=.15 occurs 16/50; most keep central camp, also visible in source Fig. 3 and independent implementation. | Audit initial draws and quantify typicality with source-compatible cluster definitions; do not label the single example impossible. |
| <a id="rr-hk-02"></a>RR-HK-02 | Mismatch / Open / L; [record][opinions] | “Less than 15 periods” stable in 53/60, slowest 168 under 10⁻¹⁰ motion tolerance. | Recover source stopping tolerance and distinguish near-merged camps from numerical settling. |
| <a id="rr-hk-03"></a>RR-HK-03 | Reading / Open / D; [record][opinions] | Fig. 13 ε left .8 read as .08; serial order unstated. Population-dependent threshold documented by Lorenz, with 1/20 consensus N 50 versus 12/20 N 1,000 at ε=.22. | Verify caption decimal and bind consensus claims to size/order/tolerance. |

### Milestone 18 — Cohen–Riolo–Axelrod 2001, `structure`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-struct-01"></a>RR-STRUCT-01 | Reading / Open / L; [record][structure] | High-cooperation threshold 2.3 is implied by p. 20 and fits Table 2, not formally defined; 2.2/2.4 materially miss. | Verify original collector/high-state definition and retain inferred/fitted status. |
| <a id="rr-struct-02"></a>RR-STRUCT-02 | Reading / Open / L; [record][structure] | Appendix all-agent noise closely recovers Table 2; §2 copy-only reading overshoots fixed structures .05–.09 and narrows FRNE−2DK from +.021 to +.007. | Keep appendix primary and preserve the loose-prose sensitivity; retrieve original noise application for equivalence. |
| <a id="rr-struct-03"></a>RR-STRUCT-03 | Question / Open / L; [record][structure] | FFR-.3 bistability supported by our lenient 50-period diagnostic (25/30 across whole run; 11/20 after 1,000); source describes long high/low stretches without this numeric rule. | Define source-compatible dwell/state criterion before treating our proxy as exact bistability reproduction. |

### Milestone 19 — Epstein 1998/GSS2006 and Radax–Rengs 2009/2010, `dpd`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-dpd-01"></a>RR-DPD-01 | Mismatch / Open / L; [record][dpd] | Runs 1/2 tables reject both counts: 729/171 versus 779/121 and 695/196 versus 784/99 (30 seeds). No tested six-switch setting reproduces both; Radax–Rengs best 703/164 versus 780/97. | Dock transition order/newborns/deaths with original or Repast source before interpreting same switches as same model. |
| <a id="rr-dpd-02"></a>RR-DPD-02 | Mismatch / Open / L; [record][dpd] | Table 9.3: 22/90 means inside 95% intervals; both counts in 1/45 cells; five cells pass both tests. Lone defectors survive where source ranges(0, 0). | Audit payoff-vector mapping, death/metabolism and source confidence-interval construction cell by cell. |
| <a id="rr-dpd-03"></a>RR-DPD-03 | Reading / Open / L; [record][dpd] | Closest table fit needs working-paper random neighbor, zero-founder wealth and immediate newborns; 7/512 combinations fit both, all working-paper rule. Run 2 loses fit in fresh seeds 31–90. | Recover founder wealth/play rule/version, retain fragile retrospective fit as sensitivity not source validation. |
| <a id="rr-dpd-04"></a>RR-DPD-04 | Mismatch / Open / L; [record][dpd] | Run 4 R 1 gives 26/30 extinct by 500/all by 2,000, not sustained 300–500-cycle oscillations or cooperative monopoly; no tested timing rescue. | Retrieve source Run 4 history/settings and cycle/count definition. |
| <a id="rr-dpd-05"></a>RR-DPD-05 | Mismatch / Open / L; [record][dpd] | Shifted payoffs 12/11/1/0 retain both types in 30/30 at 500 and 2,000, not Fig. 13 pure defection. Per-cycle metabolism 6 is different; per-game subtraction is exact identity. | Separate shifted-payoff extinction premise from algebraic per-game equivalence; verify source charging clock. |
| <a id="rr-dpd-06"></a>RR-DPD-06 | Mismatch / Open / L; [record][dpd] | Footnote 27 payoffs 16/11/5/4, lifetime 10: monopoly 1/30 by 2,000, none by 500; R 15 alternate also fails. | Resolve “hiked by ten” arithmetic and source horizon/death rule before retesting monopoly. |
| <a id="rr-dpd-07"></a>RR-DPD-07 | Mismatch / Open / L; [record][dpd] | Coordination norm maps form when both persist, but coexistence 17/30 at 500, 16/30 at 2,000, 11/30 at 5,000. | Clarify whether source claims one illustrated map or reliable long-run coexistence. |
| <a id="rr-dpd-08"></a>RR-DPD-08 | Reading / Open / D; [record][dpd] | GSS references wrong shifted-payoff figure; Table 9.3(4, 2) defector interval 254–376 conflicts with mean 265/SD 32 (253–277). | Verify original table/errata and preserve documentary correction separately from model discrepancy. |

### Milestone 20 — Axelrod 1986 and Galán–Izquierdo 2005, `norms`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-norm-01"></a>RR-NORM-01 | Reading / Open / L; [record][norms] | Long metanorm persistence depends on zero-SD ties/population adjustment: at 10⁶ GI reading collapses 18/20; Axelrod words hold 14/20. Ranked adjustment collapses 50/50 by 10⁵; that variant is ours. Original 100-generation conclusions reproduce; GI explicitly disclaims critique of that horizon. | Recover selection/adjustment implementation; compare horizons/readings separately and do not call original 100-generation claim refuted. |
| <a id="rr-norm-02"></a>RR-NORM-02 | Mismatch / Open / L; [record][norms] | Literal dominance without metanorms: strong group bold 97/100; weak bold 50/100 and below .2 in 39, versus qualitative high-both claim. Everyone-touched variant gives .94/.86 but is a departure. | Audit group-local punishment/selection and source definition of high boldness; keep RR-FIX-02 closed. |

### Milestone 21 — Nowak–Sigmund 1998 and Leimar–Hammerstein 2001, `image`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-img-01"></a>RR-IMG-01 | Reading / Open / L; [record][image] | Fig. 1 k 0 fixation=20/100 (any cooperative 40/100); source shows one 166-generation run. Not a claim k 0 is impossible; more interactions raise wins 91/100 and 100/100. | Identify source typicality claim and exact init/offset before judging ensemble fixation frequency. |
| <a id="rr-img-02"></a>RR-IMG-02 | Mismatch / Open / L; [record][image] | About two interactions/lifetime suffices: at m=100 cooperative strategies 18% of generations 1,001–20,000; reach half at m=200/four interactions. Source averages 10⁷, our studies 10–100× shorter. | Audit sufficiency definition and source-length convergence before declaring long-run insufficiency. |
| <a id="rr-img-03"></a>RR-IMG-03 | Reading / Open / L; [record][image] | Fig. 3 n effect depends on personal tally versus copying whole donor score (.97/.93/.92 across n=20/50/100), and visibility .1 versus ten observers. | Recover observer update/visibility convention before judging source group-size effect. |
| <a id="rr-img-04"></a>RR-IMG-04 | Mismatch / Open / L; [record][image] | Fig. 4 b–d help/most-common strategies differ (4 b 53% versus 57%; 4 c 78 versus 70; 4 d 85 versus 80); 4 c long run not judged by survey. | Review panel-specific rounds/strategies/source horizon; separate unjudged 4 c from measured short-window discrepancy. |
| <a id="rr-img-05"></a>RR-IMG-05 | Mismatch / Open / L; [record][image] | Own-score-only help .19% versus <.1%; uniform mutant floor can help initially. | Verify mutation distribution and measurement window in source. |
| <a id="rr-img-06"></a>RR-IMG-06 | Reading / Open / L; [record][image] | Methods xmin≈.123 over five fixed one-game/player rounds versus random-pair payoff crossing .16 (2,000 seeds/share); these are different sampling protocols. | Reconcile methods rounds with simulation pairing before testing the same threshold. |
| <a id="rr-img-07"></a>RR-IMG-07 | Mismatch / Open / L; [record][image] | LH01 islands: Fig. 2b help 44% at 5,000/35% at 20,000 versus 9%; 2 c 15% versus 2%; gene-flow/cost monotonicity differs. Source horizon 10⁵–10⁶. | Audit local reproduction, offset/errors and matched long-run statistic; do not attribute LH result to NS baseline. |
| <a id="rr-img-08"></a>RR-IMG-08 | Mismatch / Open / L; [record][image] | LH Fig. 1 b invasion 1.8% by 150/42% by 1,000, much slower; Fig. 3 a 52% versus 45% at 3,000 narrows 47% at 10,000; 3 b q share 26→18% versus 12%. | Recover panel-specific mutation/errors/init and match horizon before grading rates and speed. |
| <a id="rr-img-09"></a>RR-IMG-09 | Question / Not measured / D; [record][image] | Panchanathan–Boyd, Lotem et al. and Ohtsuki–Iwasa are attributed follow-up sources, not demonstrated full replications here. Offset ablation raises cooperation but is our test, not original-code identity. | Define independent source protocols before claiming those critiques reproduced. |
| <a id="rr-img-10"></a>RR-IMG-10 | Reading / Open / X; [record][image] | Analytic .7380294688360… constant reproduces all printed digits only with negatives at−1 and sufficiently high positives; finite lower starts give smaller thresholds. Source initialization unstated. | Recover analytic score initialization and distinguish exact constant derivation from finite simulation starts. |

### Milestone 22 — Deffuant lineage, Meadows–Cliff 2012 and reply 2013, `agreement`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-ra-01"></a>RR-RA-01 | Reading / Open / A; [record][agreement] | Reply's convergence 1,200 and cutoff .7 jointly give mean y .98 versus .00 at 200/.8; 2002 omitted both; reply cutoff chosen after trajectory inspection. Java/Python update conventions differ. | Retrieve historical cutoff/clock/code mapping; keep successful reply adjustment separate from original equivalence. |
| <a id="rr-ra-02"></a>RR-RA-02 | Reading / Open / A; [record][agreement] | Single extremes fall 60→16→5→0/100 at N=200/1,000/2,000/4,000 (pe .1, U 1.6); at pe .05 still 52/100 at 4,000. Extremist uncertainty alters nearby-boundary drift. | Bound claim to measured slices; preregister any population-limit or uncertainty generalization. |
| <a id="rr-ra-03"></a>RR-RA-03 | Mismatch / Open / A; [record][agreement] | Fig. 7 printed μ=.5 gives no single/central outcome in 1,000 under stated counting, while μ=.2 does. Figures are examples; cutoff/iteration normalization unspecified. | Recover μ/counting/clock before judging whether the single illustrated outcome is possible. |
| <a id="rr-ra-04"></a>RR-RA-04 | Mismatch / Open / A; [record][agreement] | Printed Eq. 11 influencer BC window: all 600 runs y 0 across six U, δ=0/.1; listener window gives 50/50 single at U 1.0. Scope pe .05, N 1,000, μ=.2 only. | Verify equation versus implementation for Fig. 20 and freeze a new source-consistent test; no whole-§6 failure inference. |
| <a id="rr-ra-05"></a>RR-RA-05 | Reading / Open / A; [record][agreement] | Sparse-network cluster labels depend on unspecified cutoff; small-world onset differs from majority single, and pair-sampling/cap choices matter. Tight cluster gap alone cannot grade the visually compatible 2000 lattice. | Recover network sampling/counting/transition definitions separately from trajectories; preserve the source-spike audit. |
| <a id="rr-ra-06"></a>RR-RA-06 | Reading / Open / D; [record][agreement] | Fig. 5 caption 4% joining and y=.03 cannot both satisfy stated squared-share indicator. | Verify original caption/errata and indicator arithmetic separately from trajectory reproduction. |

### Milestone 23 — Arthur 1994, CZ97/98, SMR99 and CMO04, `farol`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-ef-01"></a>RR-EF-01 | Provenance / Blocked on source / A; [independent audit][farol], [fixed-library README][farolreadme] | Arthur library 48/scoring/ties is constructed, not identified historical bank; random reties depart from deterministic prose. Mean 58–60 and negative lag 1 reproduce this variant, not source identity or persistent cycles. | Retrieve historical predictor bank/scoring/ties; label existing native/independent agreement as reconstruction evidence. |
| <a id="rr-ef-02"></a>RR-EF-02 | Question / Not measured / A; [record][farol] | Predictor-type robustness and minimum bank size have no reproducible corresponding tests for the constructed 48-predictor bank. | Declare bank-family controls before judging robustness or necessary library size. |
| <a id="rr-ef-03"></a>RR-EF-03 | Mismatch / Open / A; [record][farol] | CZ97 Fig. 4 rounded payoff concentrates ~99.8% centrally and fails visible ~350/650 lobes; independent sticky/redraw agrees. Exact payoff has central/multiple side peaks. Central share proxy cannot judge bimodality; horizon/init/ties unreported. | Recover figure statistic/setup then define shape test; retain retrospective diagnostic and omit categorical source-failure language. |
| <a id="rr-ef-04"></a>RR-EF-04 | Question / Not measured / A; [record][farol] | CZ97 Fig. 10 evolving-population proxy never measured purity; homogeneous audit shows extreme waste under both tie rules but is not exact illustration replication. CZ98 reports diversity without mutation. | Measure population identity and match replacement window/init/horizon to Fig. 10 in a separate protocol. |
| <a id="rr-ef-05"></a>RR-EF-05 | Reading / Open / D; [record][farolreadme] | CMO non-half binary uses step-win updates/equality convention rather than Eq. 4 linear mismatch/Θ(0)=1; critical .3374 belongs to later theory, not CZ98 approximate .5. | Preserve attribution and implement/declare exact source contract before claiming CMO/CZ numerical reproduction. |
| <a id="rr-ef-06"></a>RR-EF-06 | Question / Not measured / L; [record][farolreadme] | SMR best-agent finite win rates 54% at M 6/49% at M 10 are not the paper’s formal statistical-significance test. | Recover source win-rate/null/sample protocol and uncertainty before asserting significance reproduction. |

### Milestone 24 — Kirman 1993 and Alfarano–Milaković 2007/2009, `ants`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-ant-01"></a>RR-ANT-01 | Reading / Open / A; [record][ants] | Exact base chain has no interior 80–20 stationary mode across 17,820 settings; real ants show transient imbalance. Project Becker multiplier gives 18/82 modes but Kirman supplied no formula/numeric target. | Separate transient occupancy from stationary mode; retrieve or declare the majority-attraction law before claiming exact 80–20 reproduction. |
| <a id="rr-ant-02"></a>RR-ANT-02 | Reading / Open / A; [record][ants] | Fig. IIb half-time mean is compatible in 229/1,000 finite records; compatibility rule was revised after the result was known. Pooled residual regime times are not exact-state Markov age invariance. | Retain retrospective status and exact-state conditioning; do not treat nonmatching finite examples as paper failure. |
| <a id="rr-ant-03"></a>RR-ANT-03 | Reading / Open / A; [record][ants] | AM Fig. 3 omits N: at N=100 ring/small-world variance is 19–33% low; N=50 ring is within 15% at all α (20 seeds), other networks not rerun. Update rule and realized versus nominal degree differ. | Retrieve N/update protocol; keep conditional N=100 shortfall distinct from exact published-figure verdict. |
| <a id="rr-ant-04"></a>RR-ANT-04 | Question / Open / A; [record][ants] | AM Fig. 4 random-network variance is robust over 50–1,050 (three seeds/size), but original fit reaches ~5,000. Pairwise Kirman meetings do not cure N dependence; footnote 18 rescales variance. | Audit original network generator/update/scale and declare any fit-reproduction extension separately from qualitative direction. |

### Milestone 25 — Granovetter 1978 and Watts 2002, `thresholds`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-th-01"></a>RR-TH-01 | Reading / Open / A; [record][thresholds] | Watts Fig. 3, z=6.14, n=1,000, assuming 18% threshold: 232/1,000 global versus 1/1,000; at n=10,000, 20/1,000. Axis starts .0001, conflicting with n=1,000 count granularity; pair probability z/(n−1) versus z/n. | Resolve network size, threshold/normalization and graph probability before a fresh source-frequency test. |
| <a id="rr-th-02"></a>RR-TH-02 | Reading / Open / A; [record][thresholds] | Fig. 4 a normal tails unspecified: clipped sensitivity shows wider high-z cascade reach and lower low-z frequency, not a refutation of region comparison; source says normalized [0, 1]. | Recover truncated-normal/tail and zero-threshold convention; judge analytic regions separately from point frequencies. |
| <a id="rr-th-03"></a>RR-TH-03 | Reading / Open / A; [record][thresholds] | Fig. 4 b normalized integer k≥1, τ=2.5 has mean ≤1.95 and no 18%-threshold cascade condition; cannot span plotted means. | Retrieve missing minimum-degree/scaling family and verify analytic normalization. |
| <a id="rr-th-04"></a>RR-TH-04 | Question / Open / A; [record][thresholds] | Dense hub advantage persists at rare finite n=2,000, z=6.6 (97 versus 16/1,000; independent 96 versus 26), outside infinite-network window. Maximum-node selection differs from conditioning on fixed degree. | Define source asymptotic versus finite estimands and boundary sequence; retain unresolved historical claim. |

### Milestone 26 — Axtell–Epstein 1999 and GSS2006 ch7, `retirement`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-ret-01"></a>RR-RET-01 | Reading / Open / A; [record][retirement] | First 95 is an operational eligible-retired crossing, not undefined age 65/62 norm or absorbing state. Five-percent rational mean 61.5 versus illustrated plateau 375; 15/20% rapid outcomes are not exact six-period trajectory reproduction. | Recover numeric norm/absorption criterion and source trajectory data before interpreting proxy gaps. |
| <a id="rr-ret-02"></a>RR-RET-02 | Provenance / Blocked on source / A; [record][retirement] | Slot renewal/cohort traversal/mortality choices unspecified; all-members gives 0/50 first 95 within 600 versus eligible 50/50; Slot crosses even at 0/2% rational within 2,000. This does not establish infinite-time criticality or prove Replace necessary. | Retrieve original pointer/activation/denominator rules; separate alternative reconstructions and criticality target. |
| <a id="rr-ret-03"></a>RR-RET-03 | Reading / Open / A; [record][retirement] | AE threshold .5 differs from GSS U[.5, 1]. GSS 5% policy: 22/50 reach within 100 (conditional 43.73±32.90), independent 28/50 (49.61±31.88). Event-mode 70 at switch uses different native/independent windows; crossings can reverse. | Match source warmup/age norm, retain censoring and identical event window; avoid categorical policy norm failure. |
| <a id="rr-ret-04"></a>RR-RET-04 | Question / Open / A; [record][retirement] | Sensitivity/coupling directions and C=200/300 proxy equivalence supported; source trajectories/norm and exact author implementation not identified. | Audit source numerical targets and independent statistic agreement; do not turn qualitative/crossing support into exact reproduction. |

### Milestone 27 — Boyd–Gintis–Bowles–Richerson 2003; Cooney and Janssen, `punishment`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-pun-01"></a>RR-PUN-01 | Mismatch / Open / L; [record][punishment] | Stated conflict rules recover 2/14 figure curves versus 14/14 at 2ε (mean gap .006–.049); at n=128 cooperation .17 versus figure .64. Caption rates conflict with legend; baseline unstated. Either-can-start 2ε−ε² gets 13/14. | Recover original pairing/conflict/baseline and reconcile caption/legend; declare new variants before measuring. |
| <a id="rr-pun-02"></a>RR-PUN-02 | Question / Not measured / L; [record][punishment] | Twice-conflict fit does not distinguish either-start from Janssen challenge pairing; doubling victory slope untested. | Define discriminating event-frequency/victory controls before attributing cause. |
| <a id="rr-pun-03"></a>RR-PUN-03 | Mismatch / Open / L; [record][punishment] | Mixing m=.01 reportedly equalizes two groups in ~50; 58% difference remains under copied-agent transition. | Recover source equalization tolerance and imitation/mixing clock. |
| <a id="rr-pun-04"></a>RR-PUN-04 | Mismatch / Open / L; [record][punishment] | Continuous traits not similar: 94% at n=32/90% at 256 versus base 69/12; uniform mutations maintain punishment near half. | Verify source continuous-trait mutation/range rule; separate discrete baseline from extension. |
| <a id="rr-pun-05"></a>RR-PUN-05 | Mismatch / Open / L; [record][punishment] | Ring without conflict still has about half cooperative at n=4/8 (little by 32), against cooperation-free statement. | Recover initial/local imitation/mutation and source size scope. |
| <a id="rr-pun-06"></a>RR-PUN-06 | Reading / Open / L; [record][punishment] | Janssen benefit/challenge/current-act/sequential readings come close but no-punishment cooperation .42 at n=16 versus figure .20. These are distinct replication rules, not original-code docking. | Document each NetLogo reading and source departure separately before assigning successful curve fit. |
| <a id="rr-pun-07"></a>RR-PUN-07 | Mismatch / Open / L; [record][punishment] | Cooney PDE dip occurs under every tested victory rule, not only normalized rule blamed in Remark 6.1; higher punishing cost never raises punishers here. | Compare PDE assumptions/limit with finite ABM and victory functions; do not call theorem disproved by a different model. |
| <a id="rr-pun-08"></a>RR-PUN-08 | Mismatch / Open / L; [record][punishment] | Fewer groups changes cooperation mean .56 at 8 groups versus .69 at 128, rather than only increasing noise. | Audit source group-count setup and compare mean shifts separately from variance. |

### Milestone 28 — Gode–Sunder 1993 and Cliff 1997, `zi`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-zi-01"></a>RR-ZI-01 | Provenance / Blocked on source / L; [record][zi] | “30 seconds” untranslated into shouts: 100 gives 44–86% efficiency; ~500–1,000 needed for source numbers. Markets 1–4 recovered using Table 2 calibration; market 5 approximate ZI-U efficiency 86.7 versus 86.0. | Retrieve timing/quote-generation/schedules and identify calibration versus validation targets. |
| <a id="rr-zi-02"></a>RR-ZI-02 | Mismatch / Open / L; [record][zi] | Cliff box-market mean prices miss predictions by 12/10; printed 233⅓ is simulation, formula gives 241⅔. Gode–Sunder mechanism shifts deviations about halfway toward equilibrium. | Audit formula integrals/box schedules and code clearing rule; preserve critique direction and numeric discrepancy. |
| <a id="rr-zi-03"></a>RR-ZI-03 | Reading / Open / L; [record][zi] | ZIP text momentum U[.2, .8], code U[0, .1]; text 100 failed shouts ends day, code only session (9/11 sessions). Efficiency/speed differ. | Bind each source claim to code/control-file or text choice and inspect original runtime/settings. |
| <a id="rr-zi-04"></a>RR-ZI-04 | Mismatch / Open / L; [record][zi] | Footnote trading-order correlations are higher for ZI-C than ZI-U as claimed, but .91/.85 versus .74/.42. | Recover source trade ordering/correlation definition and matched quote horizon. |

### Milestone 29 — Lansing–Kremer 1993 and Janssen 2007, `bali`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-bali-01"></a>RR-BALI-01 | Reading / Open / L; [record][bali] | Temple resemblance mostly pest-link partition: network ARI .33, imitation .37 (above in 6/10), below retrospective +.05 margin. | Recover source patch/temple comparison and test increment over network separately from raw resemblance. |
| <a id="rr-bali-02"></a>RR-BALI-02 | Mismatch / Open / L; [record][bali] | Only 7 subaks changing in year 8 / 2 in year 30 versus 20. | Verify year/change/persistence definitions and source initial plans before testing continued dynamism. |
| <a id="rr-bali-03"></a>RR-BALI-03 | Mismatch / Open / L; [record][bali] | Fig. 11 same-plan start never recovers after 18.1→16.5; random plans recover in 3/10 within seven years; “twice as long” rise does not appear. Perturbation magnitudes are ours. | Retrieve perturbation/time/initial specification and separate recovery from rise-duration claim. |
| <a id="rr-bali-04"></a>RR-BALI-04 | Mismatch / Open / L; [record][bali] | Janssen coordination 1→172 improves .7% under code columns /3.7% physical, up to 7% low rain versus ~30% Fig. 1; temple scale never best. | Recover search starts/group definitions/data columns/water units and audit the coordination curve. |
| <a id="rr-bali-05"></a>RR-BALI-05 | Provenance / Open / L; [record][bali] | Highlands/lowlands unidentified; level 2 uses rivers, 7 adjacent mascetis, 28 second column yielding only 22 groups; source vegetable crop absent from 21-plan data. Code random-dam/no-upstream/swapped columns and pest reset differ from texts. | Resolve watershed/group/crop provenance before exact source equivalence; retain named departures. |
| <a id="rr-bali-06"></a>RR-BALI-06 | Mismatch / Open / L; [record][bali] | Adaptive water-threshold harvest nearly flat 26.0/26.2/25.3/26.0; source best (.05, .02) is 7% below (.05, .05). | Verify tolerance units/controller plant duration and Fig. 12 numerical target. |
| <a id="rr-bali-07"></a>RR-BALI-07 | Mismatch / Open / L; [record][bali] | Eq. 4 imitators lose ~4% when half links removed (p=.08), and 6% when added (p=.007), contrary to claimed indifference to adding; adaptive direction recovers. | Audit link direction/innovation and source sensitivity procedure; keep uncertain removal separate from measured added-link effect. |
| <a id="rr-bali-08"></a>RR-BALI-08 | Mismatch / Open / L; [record][bali] | Rising harvest inequality with finer coordination is not recovered under the tested data/search readings. | Recover source inequality statistic and group partition/search outputs independently of mean harvest. |
| <a id="rr-bali-09"></a>RR-BALI-09 | Mismatch / Open / L; [record][bali] | Janssen’s benefit of coordination only at pest growth g=2.2 is not recovered in the documented growth sweep. | Audit pest-growth/plan-search assumptions and source comparison grid. |
| <a id="rr-bali-10"></a>RR-BALI-10 | Mismatch / Open / L; [record][bali] | Janssen’s high-dispersal harvest losses are not recovered under the documented dispersal sweep. | Audit pest diffusion/reset and source dispersal units separately from growth and coordination scale. |

### Milestone 30 — Schelling 1971 board and line, `schelling` / `line`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-s71-01"></a>RR-S71-01 | Mismatch / Open / L; [record][schelling] | Board Fig. 8 typical results .80 alike / 38% unmixed versus 90% / two-thirds; ratio 3.6 versus upwards of four. His hand-worked samples explicitly small; Fig. 9 closer. | Recover hand board/tie/vacancy/order and distinguish an illustrated extreme from ensemble claim. |
| <a id="rr-s71-02"></a>RR-S71-02 | Mismatch / Open / L; [record][schelling] | Minority board Fig. 13 like-to-unlike ratio 1.3 versus 2: 1, while density direction recovers. | Verify source minority counts and pooled versus agent-weighted metric. |
| <a id="rr-s71-03"></a>RR-S71-03 | Mismatch / Open / L; [record][schelling] | Halved line minority .77 alike versus .78 equal, contrary to more-segregated minority; pooled total .85 is majority effect. | Match source minority-specific/pooled neighbor denominator and initial line operation. |
| <a id="rr-s71-04"></a>RR-S71-04 | Reading / Open / L; [record][schelling] | Restricted line 20% minority, at most 10 passed, leaves 6% unsatisfied versus 3% unlimited, against “everybody”; source reach radius unspecified. | Retrieve travel radius/fallback and horizon before claiming source-specific failure. |
| <a id="rr-s71-05"></a>RR-S71-05 | Reading / Open / L; [record][schelling] | 24 neighbors attenuate at a third (.55 versus .66) but increase at half (.86 versus .80); source explicitly qualifies moderate demands. Line 81.5% pools neighbors versus our mean .78. | Keep source demand qualifier/denominators; do not generalize attenuation or compare different aggregations directly. |
| <a id="rr-s71-06"></a>RR-S71-06 | Mismatch / Open / L; [record][schelling] | Board demand curve rises .56/.66/.72/.80 at 20–50%, faster 20–35 than stated rapidly rising 35–50 range. | Match source demand table and progression metric rather than extrapolate from individual hand boards. |

### Milestone 32 — Pancs–Vriend 2007 and Zhang 2004 JEBO, variations

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-var-01"></a>RR-VAR-01 | Mismatch / Open / L; [record][variations] | PV footnote 23 spiked “very similar” to p 100: 6.99 versus 4.69 clusters (500 seeds); most other PV/Gauvin/Singh results recovered. | Verify spiked utility/ties/equilibrium definition and report separately from successful p 100. |
| <a id="rr-var-02"></a>RR-VAR-02 | Reading / Open / L; [record][variations] | Zhang Fig. 8 below 600 mixed pairs impossible literally at torus half-split minimum 600; scaled potential 600 means 8,000 pairs, reached in ~100,000 draws versus 40 million. | Resolve source potential/scaling/minimum and iteration clock before waiting-time comparison. |
| <a id="rr-var-03"></a>RR-VAR-03 | Mismatch / Open / L; [record][variations] | Zhang sharp transition between β=4 and 8, not near 2; no sorting below 2 does hold. | Recover utility/β normalization and phase/statistic target before a fresh transition test. |
| <a id="rr-var-04"></a>RR-VAR-04 | Mismatch / Open / L; [record][variations] | Eight neighbors take .62 of four-neighbor time (16 versus 10 steps), not less than half; twelve take 8.5. | Audit speed clock/neighborhood utility normalization and source horizon. |

### Milestone 33 — Axtell 1999 and 2013, `firms`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-firm-01"></a>RR-FIRM-01 | Mismatch / Open / X; [record][firms] | §2 optimal groups “under 10” for θ<.85: θ=.84 optimizes at 11; Table 1 analytics recover to three decimals. | Verify domain/endpoints and integer optimization; retain analytic exception separately from simulation. |
| <a id="rr-firm-02"></a>RR-FIRM-02 | Mismatch / Open / L; [record][firms] | 1999 base OLS µ~2.5 (ML 1.4) versus 1.28 under tested readings; largest 62–150 versus illustrated 205; output ~750 versus 450–600; productivity size^.67 versus ^1.15. | Audit original size/output data, fit method and effort/activation; compare each statistic without switching estimators. |
| <a id="rr-firm-03"></a>RR-FIRM-03 | Mismatch / Open / L; [record][firms] | 1999 firm lifetime ~4 periods versus 23.4; loyalty 10 gives ~35. 2013 ~77 periods activates only 4% of agents: ~3.1 decisions per agent, not stronger lifetime evidence. | Recover firm birth/death definition and source clock; separate later parameters from 1999 target. |
| <a id="rr-firm-04"></a>RR-FIRM-04 | Reading / Open / D; [record][firms] | Source counts/means/lifetimes inconsistent with 1,000 agents; output below all-alone start; Table 11 four rows are one identical model; caption “target output” has no rule. | Compile page/table-specific consistency worksheet from original paper and errata before attributing numerical gaps. |
| <a id="rr-firm-05"></a>RR-FIRM-05 | Reading / Open / L; [record][firms] | Growth Laplace wins 10/10 only including 38–41% exact zeros; nonzero growth Gaussian wins 10/10. γ median .171 versus .174 but seeds range .12–.24. | Recover growth sampling/zero treatment and judge finite uncertainty before interpreting distribution fit. |
| <a id="rr-firm-06"></a>RR-FIRM-06 | Mismatch / Open / L; [record][firms] | §4 fixed-friend µ direction reverses: more friends raise µ where Table 6 lowers it. Many correctly signed other-table effects remain 1–2 higher in level. | Audit fixed-friend network and activation/source fit definition separately from successful random-firm effects. |
| <a id="rr-firm-07"></a>RR-FIRM-07 | Reading / Open / L; [record][firms] | Sticky effort/groping in any firm creates giant firms/negative µ; at home barely changes results. Text says groping more pronounced, tables say sticky effort. | Recover effort-constraint scope; negative fitted µ is not power-law confirmation. |
| <a id="rr-firm-08"></a>RR-FIRM-08 | Reading / Open / L; [record][firms] | Base pay 80 literal pays 2.85× output; scaling to output changes effort .10→.24 / output 290→674. Who funds shortfall unstated. Constant-returns equilibrium claim does not preclude myopic joining. | Resolve budget constraint and equilibrium versus transient decision target before table comparison. |
| <a id="rr-firm-09"></a>RR-FIRM-09 | Reading / Open / L; [record][firms] | 2013 Zipf µ~1.0 recovers, but giant peaks 3,000–5,800/10,000 (five seeds) versus usual median 780–955. | Match source peak versus time-typical largest firm and exact 2013 parameterization; do not attribute 1999 failure to later model. |
| <a id="rr-firm-10"></a>RR-FIRM-10 | Mismatch / Open / L; [record][firms] | Full hiring standard retains largest firm 20–52, not under 20 at source “breaks down” claim. | Recover hiring cutoff/test order and source breakdown statistic. |
| <a id="rr-firm-11"></a>RR-FIRM-11 | Mismatch / Open / L; [record][firms] | Seniority pay raises µ=2.49→10.03 versus Table 11’s slight 1.28→1.11 decline; neither founder-most nor newest-most reading fits its modest effect. | Resolve rank/order/pay formula and verify source Table 11 outputs. |

### Milestone 34 — Calvano et al. 2020 and follow-up critics, `collusion`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-col-01"></a>RR-COL-01 | Reading / Open / X; [record][collusion] | Code Table A 5 unprofitable deviation 93.6%, not text >95%; Table I/A 5 digits and 100/100 period-for-period docking recover. | Retain prose/table arithmetic clarification and code-identity scope; no claim whole paper failed. |
| <a id="rr-col-02"></a>RR-COL-02 | Mismatch / Open / L; [record][collusion] | Described strategy best-response equilibrium holds .2% versus one-period test 49.7%; reoptimization gains 5–41%. | Verify full best-response computation and paper definition; distinguish one-step deviation from strategy equilibrium. |
| <a id="rr-col-03"></a>RR-COL-03 | Reading / Open / X; [record][collusion] | Fig. 4 impulse-response code passes cycle position where it means state; named corrected reading answers state itself. | Preserve exact docking and named corrected reading; compare figure against actual state before interpreting punishment. |
| <a id="rr-col-04"></a>RR-COL-04 | Reading / Open / L; [record][collusion] | High prices without memory Δ=.958 and myopic Δ=.212; cuts and increases both trigger cuts, re-paired rivals Δ=.125, early 165 periods random-like .498 versus .497. | Keep critics' distinct price-level, punishment, transfer and early-horizon controls; identify exact scope of each source claim. |
| <a id="rr-col-05"></a>RR-COL-05 | Mismatch / Open / L; [record][collusion] | Tenfold slower exploration still Δ=.727 rather than removing high prices; synchronous all-price updates Δ=.345; constant ε=.05 never settles. | Audit critique's exploration claim and stopping criterion; don't grade constant-exploration arms by terminal stability alone. |
| <a id="rr-col-06"></a>RR-COL-06 | Mismatch / Open / L; [record][collusion] | Lambin Theorem 1 mean-field prices 1.6990/1.7377 not typical finite-α outcomes (27%, 13%, .4%, 3.6% across four arms). | Verify limit assumptions and α/noise convergence before theorem versus simulation comparison. |
| <a id="rr-col-07"></a>RR-COL-07 | Provenance / Blocked on source / D; [record][collusion] | Some follow-up code (Klein 2021 / Calvano 2021) only available on request; wanted 2023 genuine/spurious and Abada–Lambin remain leads. Critics' switches do not certify all source packages. | Retrieve publication-specific packages/inputs before classifying their claims as reproduced or failed. |

### Milestone 35 — Banchio–Skrzypacz 2022, `auctions`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-auc-01"></a>RR-AUC-01 | Mismatch / Open / R; [record][auctions] | Fig. 5 feedback profile (.90, .90): 432/500=.864 below registered .90 concentration; feedback revenue .906800 direction Holds. | Recover author initialization/ties/endpoint collection; separate figure endpoint from successful feedback effect. |
| <a id="rr-auc-02"></a>RR-AUC-02 | Mismatch / Open / R; [record][auctions] | Primary downward reading SPA below-top .015015 among 999 stable versus .50 required; FPA 1.0. Alternative triggers/clocks reported, not selected. | Retrieve unspecified source perturbation trigger/time origin; freeze distinct follow-up if source clarifies. |
| <a id="rr-auc-03"></a>RR-AUC-03 | Mismatch / Open / R; [record][auctions] | Fringe concentration 485/999=.485485 below .50; strategic mean bid .576727 passes >.50. Interval crosses acceptance boundary, but fixed joint verdict Fails. | Audit source fringe profile/statistic; keep borderline concentration and direction as distinct components. |
| <a id="rr-auc-04"></a>RR-AUC-04 | Mismatch / Open / R; [record][auctions] | Persistent ε=.001, seed 1 ×100 million/format: SPA played-pair top occupancy .309875 (.310061 late), not near all top; FPA .000029. | Verify source occupancy/clock/seed/horizon contract; do not substitute terminal greedy policies or shorter ensembles. |
| <a id="rr-auc-05"></a>RR-AUC-05 | Provenance / Open / R; [record][auctions] | Three bidders δ=.999 finish but miss 95% stable coverage; Inconclusive, not missing history or selected low-bid reproduction. | Inspect retained stable census and locate source convergence/horizon; declare any duration extension prospectively. |
| <a id="rr-auc-06"></a>RR-AUC-06 | Reading / Open / R; [record][auctions] | Q initialization/ties/hindsight/trigger choices unspecified; initialization can remove baseline format distinction. Fig. 1 FPA implies .2265 versus text .24; SPA 994 top / six exceptions, including off-diagonal. Fig. 5 colorbar normalization unresolved. | Resolve author initialization/statistic/figure-prose inputs; retain descriptive arms without new binary failure inference. |
| <a id="rr-auc-07"></a>RR-AUC-07 | Provenance / Blocked on source / D; [record][auctions] | No original simulator located in checked public author sources; follow-up context does not establish exact source equivalence. | Seek publication-mapped code/RNG/settings/output package; no author contact is claimed. |

### Milestone 36 — Cederman 1994/1997 and Radax/Störmer adaptations, `polarity`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-ep-01"></a>RR-EP-01 | Mismatch / Open / R; [record][polarity] | Source 72 mean/category rows: 45 Compatible, 24 Incompatible, 3 Unresolved; 24=17 original categories +1 original mean +2 PRA categories +4 tax categories. Conditional reconstruction only. | Audit each row's digitization/statistic/settings from linked matrix; distinguish category versus mean compatibility. |
| <a id="rr-ep-02"></a>RR-EP-02 | Mismatch / Open / R; [record][polarity] | Defense P 2 positive 2–10-sovereigns aggregate Fails (−.488929). Mean polarity rises; negative published probability direction reproduces. | Resolve “stability” estimand against source; preserve positive hypothesis versus negative counterexample. |
| <a id="rr-ep-03"></a>RR-EP-03 | Mismatch / Open / R; [record][polarity] | Alliance P3 positive 2–10 aggregate Fails (−.326786); mean polarity rises / hegemony falls. Measures and strata are distinct. | Audit alliance/source-direction components separately and retain inconclusive strata. |
| <a id="rr-ep-04"></a>RR-EP-04 | Provenance / Open / R; [record][polarity] | Complete 572 arms / 28,520 sessions: 28,281 valid / 239 invalid (235 ambiguity +4 allocation), retained in raw/registered counts. Missing precision tax knots give 3 Unresolved; allocation/support have additional unresolved controls. | Review invalid-policy/eligibility and missing-target gates; never replace invalid seeds or label invalidity itself paper failure. |
| <a id="rr-ep-05"></a>RR-EP-05 | Provenance / Blocked on source / R; [record][polarity] | No original EPM executable; source knots digitized. Störmer 110 configuration means ×10 repeats is adaptation, not exact 75% inertness cutoff. Overextension 10×10 illustration seed unknown. | Retrieve version-mapped source/RNG/raw settings/statistic definitions; keep adaptation and illustration identity separate. |
| <a id="rr-ep-06"></a>RR-EP-06 | Question / Not measured / R; [record][polarity] | Initial/terminal nonpositive-stock frequencies measured; period-exposure frequency unmeasured. Abstract signed resource creation/destruction not casualties; footnote 5 allocation factorial has no published target. | Define exposure denominator/source comparison before measurement; keep controls distinct from paper reproduction. |

### Milestone 37 — Cederman 2003/2002 and tail-analysis follow-ups, `geosim`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-gs-01"></a>RR-GS-01 | Mismatch / Open / R; [record][geosim] | Grid 75 only conditionally Incompatible among 11 comparisons; six fixed-family rejections (slope minimum, three R², median range/count). All source-identity verdicts Unresolved. | Verify grid 75 severity/count/range/collector identity before interpreting conditional difference historically. |
| <a id="rr-gs-02"></a>RR-GS-02 | Provenance / Open / R; [record][geosim] | 37 arms /1,490 attempts =1,486 complete +4 invalid; only 8/88 source targets available. Ten comparisons and all six technology/context contrasts Unresolved because of fit/eligibility gates. | Audit per-arm eligible census and fit requirements; retain unavailable multiplicity slots and invalid records. |
| <a id="rr-gs-03"></a>RR-GS-03 | Provenance / Blocked on source / R; [record][geosim] | Original APSR severity scale/log range/war count/collector unverified; strict unique-threshold fit differs from later inclusive/event-weighted collector. Modern pooled iid: 15 reject /5 nonreject /2 inconclusive; no historical reproduction inference. | Recover original statistic implementation/raw outputs; check iid assumptions separately from source compatibility. |
| <a id="rr-gs-04"></a>RR-GS-04 | Provenance / Open / A; [record][geosim] | Recovered 2017 GeoSim2 full certification Unresolved: two-stream normal draws, founder/order/resource/shock/front/claim/shadow/queue/saturation differences. 249 reference exports are not APSR identity; [certification audit][gsaudit] lists blockers. | Start first-divergence certification at constructor/RNG and version mapping; retain reference-only licensing/immutable archive. |
| <a id="rr-gs-05"></a>RR-GS-05 | Reading / Not measured / D; [record][geosim] | Printed acting-state own cost versus table/resource opponent-inflicted damage conflict; named acting-party control unmeasured, outside registered 37 arms; [damage audit][gsdamage] retains the source conflict. Reciprocal defender threshold was our default correction (RR-FIX-05). | Resolve incidence/magnitude against source; declare any damage control separately without rewriting measurement. |
| <a id="rr-gs-06"></a>RR-GS-06 | Lead / Not measured / D; [record][geosim] | Clauset modern-tail follow-ups and 2002 predecessor provide attributed questions; modern fit checks are not full replications of each source. | Freeze each follow-up dataset/estimand before claiming reproduction; changed metric is not APSR failure. |

### Milestone 38 — Cederman 2001, `democratic_peace`, including dated precision follow-up

<a id="rr-dp-03"></a>**RR-DP-03 — census context, not an open issue.**
The [dated follow-up][dp] contains 21,600 complete new histories: 10,800 literal and
10,800 prose, with zero missing, pending, partial or invalid histories. Literal extinction
900 / clustering defined 9,900; prose extinction 2,780 / clustering defined 8,020.
Extinction and undefined clustering do not imply incomplete execution. The 3,240 historical
literal histories remain a separate dataset; no historical prose-original population exists.

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-dp-01"></a>RR-DP-01 | Mismatch / Open / R; [record][dp] | 2026-10-05 literal: all / Figs. 9/10/11 conditionally Incompatible. Prose: all / Figs. 9/11 Incompatible; Fig. 10 Compatible only for 15/33 readable slots. Combined 78/105 readable. | Inspect fixed per-slot comparisons/source envelopes/statistic definitions; retain limited Fig. 10 success and both combined mismatches. |
| <a id="rr-dp-02"></a>RR-DP-02 | Reading / Open / R; [record][dp] | Printed decreasing versus prose increasing probability: literal primary Inconclusive / secondary Fails; prose Holds/Holds. All six literal primary Inconclusive; two secondary fail at density .3 (alliances/security mobile-share comparisons), four Inconclusive. | Recover publication-specific probability code; reading differences are descriptive, not a cross-reading significance test or fit-based selection. |
| <a id="rr-dp-04"></a>RR-DP-04 | Provenance / Blocked on source / R; [record][dp] | 26 overlapped unreadable slots +one absent trace in 105-family; Fig. 10's 18 gaps unfilled. Inferred exposure/curve envelopes are not paper confidence intervals. | Recover raw figure data/collector or better trace identity; do not fill gaps by interpolation or compatible prose coordinates. |
| <a id="rr-dp-05"></a>RR-DP-05 | Provenance / Blocked on source / R; [record][dp] | Historical 2001 Java/Repast/RNG/statistic/settings identity Unresolved; later GeoSim2/4/5 or Pascal ancestor are not certified 2001 source. No author contact verified; see the [author audit][dpauthor]. | Obtain version-to-publication mapping and runtime/seed/event/statistic outputs; later probability formula supports a reading only. |
| <a id="rr-dp-06"></a>RR-DP-06 | Provenance / Superseded / R; [record][dp] | Historical 108 arms ×30 =3,240 complete, no invalid/missing. Precision not registered under fixed runtime gate, so original source/mechanism families Unresolved. Follow-up now measures all 21,600 new histories; historical dataset separate; [original findings][dporiginal] retain its unavailable precision. | Use 2026-10-05 for current precision status; retain original 2026-10-03 30-history record. No historical prose population exists. |

## Minds and foraging/construction coverage

These records are outside the 31 catalog groups. Minds 1–6 and 8 are explicitly our own
experiments with comparisons to attested regularities; failing a target in a different agent
or habitat does not refute the biological/theoretical source. Minds 7 starts as a paper
reconstruction and then adds new controls. Minds 9 declares measurements without binary
verdicts. The program's [durable record][minds] preserves superseded rounds and extensions.

| Programme | Examined scope and successful/closed disposition | Entries |
|---|---|---|
| Minds 1 | Utility reduction to rule M; undermatching/crowding measured; theory conditions not all satisfied | [RR-M1-01–04](#rr-m1-01) |
| Minds 2 | A* benchmark/shortest-path correctness and wealth/seasons controls pass; altered walking rules limit book/theory comparison | [RR-M2-01–03](#rr-m2-01) |
| Minds 3 | Memory/truffles/traplining measured; expected information advantage often fails | [RR-M3-01–05](#rr-m3-01) |
| Minds 4 | Planner correctness and conditional memory advantage supported; MVT/overstaying/survival limitations | [RR-M4-01–04](#rr-m4-01) |
| Minds 5 | Amodio Bayesian Table 2 recovered; caching signatures and winter survival supported; central-place/FPH limitations | [RR-M5-01–07](#rr-m5-01); [RR-FIX-09](#rr-fix-09) |
| Minds 6 | Theft conservation/usage/equal-draw control supported; threshold/reciprocity/fitness questions differ | [RR-M6-01–06](#rr-m6-01); [RR-FIX-10](#rr-fix-10) |
| Minds 7 | Hoarding endpoints/threshold approximately recovered under per-burrow reading; speed/loss and source readings differ | [RR-HOARD-01–06](#rr-hoard-01) |
| Minds 8 | Second-round hazard/condition/forgoing frequency directions supported; other own hypotheses fail; first-round overclaims withdrawn | [RR-M8-01–06](#rr-m8-01); [RR-FIX-14](#rr-fix-14) |
| Minds 9 | Complete measured campaign, no formal Holds/Fails or stability judgment; replay/biology checks recovered | [RR-M9-01–03](#rr-m9-01) |
| Protection/deception | P3 fixed campaign measured: all 7,680 episodes/208 estimates retained, independently reviewed; publication pending; P4 prospective | [RR-P3-01–04](#rr-p3-01); [RR-LEAD-04](#rr-lead-04) |
| Burrow/construction | Selected excavation anchors/design readings; no scientific reproduction results yet | [RR-LEAD-05](#rr-lead-05) |
| Foraging F1/F2 | CPFA controller rules/variants and fixed-world implementation documented; scientific execution separate | [RR-LEAD-06](#rr-lead-06) |

### Minds 1–6 — our experiments and source-comparison limits

| ID | Programme / category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-m1-01"></a>RR-M1-01 | Minds 1 / Reading / Open / L; [record][m1] | Parker ideal matching s=1 versus rule M medians .72/.63/.90 across vision ranges; “free” condition fails (median 66/100 off patches with no starvation). Not a reproduction of ideal-free animals. | Audit information/mobility assumptions before testing input-matching theory in this habitat. |
| <a id="rr-m1-02"></a>RR-M1-02 | Minds 1 / Own hypothesis / Open / L; [record][m1] | Wandering expected to improve matching instead lowers s=.72→.40; catchment prediction .709 near median .722 but only 6/20 within .1 versus 80% required. | Isolate arrival/catchment and patch capacity rather than infer causes from median proximity. |
| <a id="rr-m1-03"></a>RR-M1-03 | Minds 1 / Reading / Open / L; [record][m1] | Baum–Kraft travel reduced undermatching in animals; local hyperbolic preference lowers s=.90→.73 at k=.5. Cost of a switch is different from nearby-sugar preference. | Match travel mechanism/protocol before calling this a biological replication failure. |
| <a id="rr-m1-04"></a>RR-M1-04 | Minds 1 / Question / Not measured / L; [record][m1] | Wander split likely follows arrival rather than input; not isolated. Expected crowding direction reversed but Sutherland m=1 matching supported (median .95). | Test the arrival explanation independently; retain crowding's supported theory comparison separately from our expectation. |
| <a id="rr-m2-01"></a>RR-M2-01 | Minds 2 / Own hypothesis / Open / L; [record][m2] | Walking capacity median 181 versus jump 228; no seed within book 214–234 target. Walking also fails to rescue waves (.6% beyond 25 cells versus project 25% target). Source jump rule was deliberately changed. | Separate expected capacity reduction from unresolved original wave target [RR-SS-04](#rr-ss-04); inspect source-compatible wave metric. |
| <a id="rr-m2-02"></a>RR-M2-02 | Minds 2 / Reading / Open / L; [record][m2] | Baum–Kraft detour direction reverses: far fence s=.85 versus .90 no fence; fences put 57% of random starts on poor side. Confound unmeasured; gap-offset sweep is not fence/no-fence test. | Declare balanced catchments and matched travel controls before attributing the effect. |
| <a id="rr-m2-03"></a>RR-M2-03 | Minds 2 / Reading / Open / L; [record][m2] | Visual-barrier comparison Weak: wall s=.91 versus fence .88, only 10/20 within .05. Single 2.10: 1 walking ratios similar and untested; five-ratio slopes are a different estimand. | Match source barrier/metric and quantify uncertainty for ratio versus slope separately. |
| <a id="rr-m3-01"></a>RR-M3-01 | Minds 3 / Own hypothesis / Open / L; [record][m3] | Memory poorer in five of six worlds (open −113, wall −114, truffles −82); remembering truffles gathers 2.3× more yet poorer in 18/20. Travel pricing raises wealth to near neutral while greatly reducing memory use. | Isolate travel cost/occupancy versus information value; a neutral wealth difference is not a demonstrated memory benefit. |
| <a id="rr-m3-02"></a>RR-M3-02 | Minds 3 / Reading / Open / L; [record][m3] | Projection worse than recall, opposite Hornvale comparison; chosen-target belief error is selected and competition prediction is missing. | Match planner/competition assumptions before attributing contrast to projection itself. |
| <a id="rr-m3-03"></a>RR-M3-03 | Minds 3 / Mismatch / Open / L; [record][m3] | Gill competition regularity: median revisit interval 50 ticks for both 5 and 20 agents, ~12% early revisits; not a reconstruction of Gill's animals. | Check encounter/regrowth and comparable visit statistics before extending the comparison. |
| <a id="rr-m3-04"></a>RR-M3-04 | Minds 3 / Reading / Open / L; [record][m3] | Bracis forgetting/regrowth comparison Weak; best span is least harmful and often at grid floor 10. Projection best span 10 at every rate; memory does not restore walking capacity (154 versus 181). | Explore below-floor spans in a new protocol and separate harmful-memory minimization from beneficial forgetting adaptation. |
| <a id="rr-m3-05"></a>RR-M3-05 | Minds 3 / Question / Not measured / L; [record][m3] | MVT untestable in `mem-mvt`: zero departures over 20 seeds; rich patches never exhaust. | Use a separately declared depleting-patch protocol; later Minds 4 comparisons do not retroactively make this target testable. |
| <a id="rr-m4-01"></a>RR-M4-01 | Minds 4 / Reading / Open / L; [record][m4] | GOAP overstaying Holds at 57% versus marginal-value 39% Fails, but excluding transit gives 20–33% / 2–7%, no seed over half. Last tick often walk out. | Match Constantino–Daw patch-departure measure and distinguish travel ticks from harvesting. |
| <a id="rr-m4-02"></a>RR-M4-02 | Minds 4 / Mismatch / Open / L; [record][m4] | Overstaying decreases with travel (GOAP 61→46%, MVT 56→35%), opposite Constantino–Daw long-travel result; rule M/value-shortlist also reverse MVT residence direction. | Audit opportunity-rate learning/shortlist and comparable departure measure; account for death-shortened visits. |
| <a id="rr-m4-03"></a>RR-M4-03 | Minds 4 / Reading / Open / L; [record][m4] | Memory advantage judged among survivors; dead-inclusive advantage misses 80%-seed target in truffles/wall. GOAP nonrememberers much poorer than rule M; remembering planners are not richer than rule-M rememberers. | Separate survivorship, population density and candidate-loss bias with independently specified controls. |
| <a id="rr-m4-04"></a>RR-M4-04 | Minds 4 / Question / Not measured / L; [record][m4] | Population gain likely partly rate fallback (52% of ticks in one probe); density and plan invalidation contributions unisolated. K/share sweeps descriptive, no formal tests. | Compare matched fallback/planning and candidate retention before attributing planner benefit; do not promote descriptive sweeps to judged claims. |
| <a id="rr-m5-01"></a>RR-M5-01 | Minds 5 / Reading / Open / L; [record][m5] | FPH 1 implemented as tomorrow versus Amodio's next day without food; planners cache zero in breakfast-first / Empty-First. Raby planner Weak (160/320 preferential) while compensation directional pattern Holds but 26.4/3.6 differs birds 16.3/5.4. | Reconcile planning horizon and keep constructed mechanism signatures distinct from exact bird reproduction. |
| <a id="rr-m5-02"></a>RR-M5-02 | Minds 5 / Own hypothesis / Open / L; [record][m5] | Expected winter planner mechanism absent: survivors forecast mean 54 site intake (0/2,932 under 10), second-summer median bury 5.4 versus 82 first. Initial standing food likely matters. | Isolate standing-stock intake and survivor conditioning before crediting winter forecasting. |
| <a id="rr-m5-03"></a>RR-M5-03 | Minds 5 / Reading / Open / L; [record][m5] | Central-place staircase learned loads 49/60/65 look like optima 51/60/65 but parity costs and decayed rate cancel; true-rate loads 45/60/63. At α=.2 distance trend nearly vanishes. | Test matched per-step costs and rate learning; do not count cancellation as analytic theorem reproduction. |
| <a id="rr-m5-04"></a>RR-M5-04 | Minds 5 / Mismatch / Open / L; [record][m5] | Lima equal near/far load target Fails; zero of 20 seeds meet the registered tolerance under the preregistered majority-load assignment (54 versus 120). Near patch lies on far route; noncrossing follow-ups still differ 14–35%, selection unisolated. | Declare identical loading curves/route/target selection and matched rate before claiming a source-theory contradiction. |
| <a id="rr-m5-05"></a>RR-M5-05 | Minds 5 / Reading / Open / L; [record][m5] | Linear loading predicts distance-independent loads; 121 near versus 151 far. Pulsed delivery rate causes 54% near-immediate returns; other 46% fill limit. | Isolate rate decay and carrying cap; distinguish resemblance to starling observations from theorem mechanism. |
| <a id="rr-m5-06"></a>RR-M5-06 | Minds 5 / Own hypothesis / Open / L; [record][m5] | Cache recovery low: 55.6–75.6% still undug at 1,000 (end snapshot, not permanent loss); many die targeting unreachable big caches. GOAP deliver-G starves far (0/100 alive). | Audit reachable-cache ranking/reserve and delivery goal; isolate causes without treating undug snapshots as ultimate failure. |
| <a id="rr-m5-07"></a>RR-M5-07 | Minds 5 / Question / Not measured / L; [record][m5] | Capacity/winter sweeps only three-seed checks, not 20-seed verification. Original metabolism 1–4 world could not meet design surplus balance; final metabolism 1 world chosen by mechanical calibration. | Preserve calibration scope and declare adequate comparative sweep sample before extrapolation. |
| <a id="rr-m6-01"></a>RR-M6-01 | Minds 6 / Reading / Open / L; [record][m6] | Andersson–Krebs condition agrees with wealth in 186/300=62% (Weak), rises 90.7% treating buried stores as zero; equal per-draw recovery is not equal exposure/recovery. | Match theory payoff/recovery denominators and cache valuation; distinguish biological assumptions from this extension. |
| <a id="rr-m6-02"></a>RR-M6-02 | Minds 6 / Own hypothesis / Open / L; [record][m6] | Frequency independence fails: survival advantage slope −.17 at find=.25; sign flips at low find. Mixed equilibrium absent at anchor but crossings at .02/.05. | Separate own anchor controls from source unequal-thief assumption; declare an evolutionary stability test rather than infer stability from crossing. |
| <a id="rr-m6-03"></a>RR-M6-03 | Minds 6 / Reading / Open / L; [record][m6] | Owner advantage nearly unused: ps=.015 versus po=.977; digging below full reserve lifts ps=.28 and shrinks cheater lead 13→3 points. | Isolate retrieval clock/threshold before attributing benefit of owner memory or failure of AK condition. |
| <a id="rr-m6-04"></a>RR-M6-04 | Minds 6 / Own hypothesis / Open / L; [record][m6] | Reciprocity stores increase (86 versus 47 caches) but keep survival 90.4% versus eat 93.3%; keep leads only at find=.5/1. | Distinguish increased stores from survival benefit and audit recaching/exposure versus loot consumption. |
| <a id="rr-m6-05"></a>RR-M6-05 | Minds 6 / Question / Not measured / L; [record][m6] | Vander Wall–Jenkins 18% resistance target Untestable: maximum 8.9% in sweep; stumbling's winter-world maximum 7% versus field median 9% is context, not matched field protocol. | Declare source-comparable search/exposure before testing high-rate resistance; later watching does not repair this sample. |
| <a id="rr-m6-06"></a>RR-M6-06 | Minds 6 / Reading / Open / L; [record][m6] | Mild-winter cheater target Fails on wealth (−98 at β=2 versus −10 at β=32), but holdings/stomach favor cheaters and milder winter; buried-cache valuation causes reversal. | Resolve source fitness/valuation; distinguish judged wealth from descriptive holdings and retain both. |

### Minds 7 — Vander Wall and Jenkins 2003, `hoard`

| ID | Category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-hoard-01"></a>RR-HOARD-01 | Mismatch / Open / L; [record][m7] | Endpoints low/high in 1,331/1,350 (98.6%), but takeover by 10 generations only 68/634 (10.7%), median 15; 37.7% within 10 after first .2 crossing. Source “usually 10”, its Fig. 2 A near 16. | Match takeover time origin and inherited variance; preserve source example versus typicality distinction. |
| <a id="rr-hoard-02"></a>RR-HOARD-02 | Mismatch / Open / L; [record][m7] | Threshold Weak: logistic midpoint .235 (SE .003) versus .219, shallower curve; 7/450 takeover below .2 where source reports none; above .3, 95% recovers. | Retrieve source variance/logit initialization/sample; compare curve shape separately from midpoint tolerance. |
| <a id="rr-hoard-03"></a>RR-HOARD-03 | Reading / Open / L; [record][m7] | Larder loss exceeds scatter in 1,266/1,350 (93.8%) versus all 35; 84 misses at takeover ratios .35–.45. Primary hazard loss 90%/27% versus source 186%/24%, whose rate is undefined. | Recover rate denominator/time averaging before numeric loss comparison; source percentages cannot be literal proportions. |
| <a id="rr-hoard-04"></a>RR-HOARD-04 | Reading / Open / L; [record][m7] | Per-item gives no takeover in 1,350 but larder loss/CV near source; per-burrow recovers endpoints but misses loss/speed. Default changed after diagnostic failure; defense slope 10 and Vseg=.5 unprinted. Other pairs miss threshold; speeding Vseg=1 shifts midpoint .174. | Recover source detection/defense/inheritance parameters; retain retrospective default history and full alternative rows. |
| <a id="rr-hoard-05"></a>RR-HOARD-05 | Local correction / Open / L; [record][m7] | Planned CV judge omitted before runs, disclosed; no judge added after. Primary within-run ratio 2.85 (across runs 1.49) versus source 1.73; per-item 1.75. | Audit omission and specify any new CV definition/judge prospectively; do not turn unjudged measured diagnostic into registered Fail. |
| <a id="rr-hoard-06"></a>RR-HOARD-06 | Question / Blocked on source / D; [record][m7] | Source promised full sensitivity elsewhere; three retained searches did not find it. Early-larder predictor 84.5% accurate has ratio-only 83.7% baseline and partly circular measurement. | Retrieve sensitivity publication/code; separate predictive increment from causal explanation. Limited retrieval does not prove nonexistence. |

### Minds 8–9 — watching and spatial seasonal hoarding, our campaigns

| ID | Programme / category / status / evidence | Target and documented outcome | Next action |
|---|---|---|---|
| <a id="rr-m8-01"></a>RR-M8-01 | Minds 8 / Own hypothesis / Open / L; [record][m8] | Fitness follows AK condition only 62/100 versus 80% requirement despite condition flip. | Audit payoff/recovery and separate condition from finite-season fitness; do not claim AK theory refuted. |
| <a id="rr-m8-02"></a>RR-M8-02 | Minds 8 / Own hypothesis / Open / L; [record][m8] | Being-watched cost target Fails; zero of 20 seeds show a cost of at least .05 (mean .012, CI .008–.017). Own-raiding cost target Fails; zero of 20 seeds show a cost of at least .05 (mean .003, not detected). | Distinguish paid harvest opportunity and theft impact; retain non-detection versus absence. |
| <a id="rr-m8-03"></a>RR-M8-03 | Minds 8 / Own hypothesis / Open / L; [record][m8] | Forgoing scroungers stable mixture Fails: ahead when rare 13/60 versus 48 required; shortfall .056=.021 baseline +.035 watching. | Isolate forgoing/never-caching baseline and declare new rare/common strategy protocol before a stability claim. |
| <a id="rr-m8-04"></a>RR-M8-04 | Minds 8 / Own hypothesis / Open / L; [record][m8] | Burying watcher lead fall not detected: slope .011, 90% CI −.004–.027; span 7 social dilemma Holds with lead .009. | Preserve flat judgment and limited dilemma; do not infer absence of frequency dependence from non-detection. |
| <a id="rr-m8-05"></a>RR-M8-05 | Minds 8 / Reading / Open / L; [record][m8] | Span 1/2 reverses hazard/condition/forgoing slope; dilemma not detected through 3. Span 7 generous relative to exact-site bird memory (2 days versus general areas at 7). | Match memory resolution/time unit and ecology before generalizing upper-bound watching results. |
| <a id="rr-m8-06"></a>RR-M8-06 | Minds 8 / Question / Not measured / L; [record][m8] | New second-round presets not timed; first-design performance does not certify current cost. Owners do not hide/defend in campaign; protection/deception remain future. | Record timing only under new declared comparison and keep biological extensions separate. |
| <a id="rr-m9-01"></a>RR-M9-01 | Minds 9 / Question / Open / R; [record][m9] | Paid perfect-block defense costs survival while preventing theft; larder/travel/exposure move together. All 3,345 envelopes complete, no extinct or zero-fitness run; no Holds/Fails registered. | Declare separate opportunity-cost/partial-protection controls; do not promote descriptive negative contrast to theory failure. |
| <a id="rr-m9-02"></a>RR-M9-02 | Minds 9 / Question / Open / R; [record][m9] | Frozen rare/common directions and interior trait endpoints do not establish stable mix; selection and inheritance noise alter means under Neutral. Joint cheating/watching not isolated. | Register longer trajectories/new ecologies and separate cheating/watching effects; retain no-stability-verdict boundary. |
| <a id="rr-m9-03"></a>RR-M9-03 | Minds 9 / Question / Not measured / R; [record][m9] | FIFO fate ledger not collected: no age/fate-fraction inference. Endpoint missing groups give undefined survival gaps; watcher-off spread unused. | Declare additional ledger/contrasts before measurement; keep missingness and unused components intact. |

## Resolved local implementation and reporting history

Closure here applies to our specified correction, not to historical source equivalence or every
scientific claim. The next action is retention/regression monitoring, not rerunning frozen studies.

| ID | Model / category / status / evidence | Problem and closure | Next action |
|---|---|---|---|
| <a id="rr-fix-01"></a>RR-FIX-01 | tags, M12 / Local correction / Resolved / D; [record][tags] | README once called tie rule unstated; corrected 2026-09-28 because p. 442 explicitly keeps current agent. Tables reproduce with that rule. | Retain correction date and avoid reopening it as source ambiguity. |
| <a id="rr-fix-02"></a>RR-FIX-02 | norms, M20 / Local correction / Resolved / L; [record][norms] | First dominance implementation touched everyone; current default follows group-local source punishment/selection. Everyone reading remains named alternative. | Preserve corrected default and distinguish RR-NORM-02's remaining claim. |
| <a id="rr-fix-03"></a>RR-FIX-03 | firms, M33 / Local correction / Resolved / L; [record][firms] | Random-effort survey Fail borrowed random-choice “rarely above 10” threshold; all 1,000 in one firm satisfies source “nothing like power law”. | Retain original verdict with claim-interpretation correction; do not cite as failed random-effort claim. |
| <a id="rr-fix-04"></a>RR-FIX-04 | polarity, M36 / Local correction / Resolved / R; [record][eprepair] | Sequential stale-front repair approved and same-seed outputs retained; original verdicts preserved alongside before/after data. | Keep original/repaired identities separate and retain amendment receipts. |
| <a id="rr-fix-05"></a>RR-FIX-05 | geosim, M37 / Local correction / Resolved / D; [record][gsdef] | Same-positive defender threshold was our premeasurement error; printed p. 148 specifies reciprocal. Named reciprocal default corrected before study. | Retain audit/correction; do not call this author-port disagreement. |
| <a id="rr-fix-06"></a>RR-FIX-06 | democratic_peace, M38 / Local correction / Resolved / R; [record][dpreport] | 24 contrast references mislabeled unregistered precision as registered/missing. Reporting amendment sets null censuses, separate required N=100; original 3,240 empirical values/verdicts unchanged. | Preserve distinct reporting-code and measurement identities. |
| <a id="rr-fix-07"></a>RR-FIX-07 | democratic_peace, M38 / Local correction / Resolved / R; [record][dp] | Plotter expected absent .findings.json suffix; CLI emitted literal_precision.json / prose_precision.json. Additive path recovery consumed existing bytes; 6,083 authorities verified immutable. | Retain failed attempt and accepted recovery; no inference/history rerun. |
| <a id="rr-fix-08"></a>RR-FIX-08 | democratic_peace, M38 / Local correction / Resolved in product, frozen limitation retained / R; [record][dp] | Frozen validate-only external-gate protection limitation; actual finite preparation used no external options and review accepted normal-run protections. Product fix has separate identity. | Do not attribute later fix to frozen binary or call accepted normal-run histories invalid. |
| <a id="rr-fix-09"></a>RR-FIX-09 | Minds 5 / Reading / Resolved / D; [record][m5] | Amodio Fig. 6 FPH 2 caption K 1/K 2 conflicts with Methods K 1/K 3; Methods comparison and our hand oracle use correct K 1/K 3. | Retain caption clarification without claiming Bayesian comparison failed. |
| <a id="rr-fix-10"></a>RR-FIX-10 | Minds 6 / Own hypothesis / Resolved / L; [record][m6] | Visits×find preset tolerance Fails (.789 versus .8); one-take probability 1−(1−find)^k explains within 3%. | Preserve failed own approximation and verified one-take rule; no source-failure inference. |
| <a id="rr-fix-11"></a>RR-FIX-11 | polarity, M36 / Local correction / Resolved / R; [record][epdomestic] | Per-period domestic-decision repair reran 1,180 provincial histories with same seeds; 27,340 nonprovincial raw lines and prior findings preserved; verdicts unchanged. | Retain original/domestic-amended provenance and complete comparison receipts. |
| <a id="rr-fix-12"></a>RR-FIX-12 | geosim, M37 / Local correction / Resolved in product, measurement identity preserved / R; [record][gsrecorder] | Postmeasurement recorder amendment fixes product history behavior, not retained scientific binary. [Portability][gsportable] and [finite-technology][gsfinite] fixes have separate premeasurement amendments. | Keep amendment ordering/identities; no frozen binary/source/environment rewrite. |
| <a id="rr-fix-13"></a>RR-FIX-13 | polarity, M36 / Local correction / Resolved / R; [record][eptelemetry] | Direct stock telemetry and strict JSON [export correction][epexport] retain measured same-seed counts/receipts; no period-exposure measurement implied. | Retain telemetry/export amendments with source clock and denominators. |
| <a id="rr-fix-14"></a>RR-FIX-14 | Minds 8 / Local correction / Superseded / L; [record][minds] | First-design field band/condition/scrounger comparisons inadequate; four interpretation overclaims withdrawn. Second round replaced judges and preserves first-round data. | Use later results without pooling rounds; retain first-round audit/withdrawal. |

## Minds P3 — dated execution update, campaign 2026-10-06

Added on 2026-10-07 UTC (2026-10-06 America/New_York). All entries concern our own bounded
mechanism experiment; independent empirical/reporting review is Approved; publication and final archival remain pending. They do not change the registered protocol,
historical Minds comparisons or any source-paper verdict.

| ID | Category / status / evidence | Target and measured scope | Next action |
|---|---|---|---|
| <a id="rr-p3-01"></a>RR-P3-01 | Minds P3 / Question / Open / R; [findings][p3] | Private relocation after observed preparation changes Selective−Off by −12 transferred units/+12 free or +9 paid lifetime ticks; renewed observation gives −8 units/0 free or −3 paid ticks, separately in both orientations. Lower theft need not improve lifetime. | Retain all 64 primary estimates and separate tagged consumption/cost/loss; any broader ecology or cost study requires a new registration. |
| <a id="rr-p3-02"></a>RR-P3-02 | Minds P3 / Question / Open / R; [findings][p3] | Mixed Selective selectivity is programmed (A=1/B=0 withdrawal fractions). Base owner lifetime is 38/37 free/paid versus reflected 44/43; lost tagged stock differs. Gross reburial does not guarantee usable original food. | Preserve both orders/orientations and realized routes/recovery; register any geometry intervention separately rather than pool or tune this archive. |
| <a id="rr-p3-03"></a>RR-P3-03 | Minds P3 / Provenance / Open / R; [actual gate](../survey/out/minds-protection-2026-10-06/provenance/task-p3-prospective-review.md) | Actual independent native gate preceded one complete campaign and byte-identical reanalysis. Historical WASM/Vitest log and old review ledger unavailable; unchanged source parity/checkpoint blocks and fresh native receipts were accepted for unchanged native execution. | Preserve the receipt limitation and scientific identity; obtain fresh platform verification before a future P3/platform-boundary change. |
| <a id="rr-p3-04"></a>RR-P3-04 | Minds P3 / Question / Open / R; [findings][p3] | Visible nonwatcher induces unnecessary paid relocation; unseen watcher defeats Selective's cue. Supplied stumble route raises residual transfer while lifetime contrast can be zero; paired draws diverge, endpoints repeat despite unique full frames. | Keep all cue/contact cells, n=40 descriptive intervals and supplied-rule boundary; separately design learning, observer knowledge or field contact before broader claims. |

## Reading-only leads and unmeasured future studies

These are retrieval or prospective work, with no failure verdict. The catalog queue/wanted list
and construction programme remain the larger research queue; missing reading copies alone are
not evidence of a failed model.

| ID | Source / category / status / evidence | Existing record and boundary | Next action |
|---|---|---|---|
| <a id="rr-lead-01"></a>RR-LEAD-01 | Cederman–Rao 2001 / Lead / Not measured / D; [record][dpauthor] | Empirical varying-coefficient dyad-year GLM needs actual data/coding/filtering/bandwidth; not additional 2001 lattice dynamics or current validation. | Retrieve data/estimation protocol and visually verify numerical OCR before empirical reproduction. |
| <a id="rr-lead-02"></a>RR-LEAD-02 | Cederman–Gleditsch 2004 / Lead / Not measured / D; [record][dpauthor] | Regime-change extension explicitly not directly comparable to 2001; 50×50 /200 states /500+10,000 periods and Moran's I differ from 2001 exposure. | Freeze its own calibration/equations/statistics/protocol; do not fold source-reported .072/.064 into our results. |
| <a id="rr-lead-03"></a>RR-LEAD-03 | Rousseau 2005 / DomGeoSim / Lead / Blocked on source / D; [primary book][rousseau] and [earlier audit][dpauthor] | Rousseau’s Democracy and War, chapter 7, printed p. 337 notes 2–3, reports reproducing Cederman before extending it, using Cederman-provided code programmed by van der Veen. It reports similar, nonidentical results and minor corrections said to leave substantive findings unchanged. This is a reported reproduction, not our independent exact-2001-code verification; the earlier bounded audit remains unchanged. | Retrieve publication-mapped DomGeoSim/code/corrections/RNG/settings and outputs before docking or adopting a source-equivalence verdict. |
| <a id="rr-lead-04"></a>RR-LEAD-04 | Minds protection/deception / Lead / Superseded for first P3; P4 Not measured / D with later R; [original record][m9], [dated execution][p3] | The 2026-10-06 backfill recorded P3 as implemented but unexecuted. Later actual prospective gate and 7,680-episode campaign close that availability gap; independently reviewed; publication pending. P4/behavior trees/HTN/collective agency remain prospective. | Use dated P3 evidence without rewriting its registration or historical results; P4 and broader protection need their own designs. |
| <a id="rr-lead-05"></a>RR-LEAD-05 | Burrow/construction / Lead / Not measured / D; [record][burrow] | Pielström–Roces 2013 selected transport/cue anchor; Green 2017 /Prasath 2023 later alternatives; architecture books research sources, not reproduced models. | Refine approved excavation contract and specify independent validation; no existing failure verdict. |
| <a id="rr-lead-06"></a>RR-LEAD-06 | Foraging/construction / Lead / Not measured / D; [record][foraging] | Hecker–Moses 2015 CPFA controller reference and Michael 2023 termite comparison; F1/F2 implemented rules are not executed scientific reproductions. | Bind source controller/world/measurement before execution; keep biological shortcut comparison separate. |

## Evidence links

The README links are legacy evidence locators; dated audits/findings carry their own sample,
judge and provenance limits. Ignored personal PDFs/raw archives are located by those records
and are not redistributed or replaced by this register.

[sugar]: ../README.md#notes
[book]: ../README.md#ring-world-animations-vi-8-and-vi-9
[anasazi]: ../README.md#artificial-anasazi-the-long-house-valley-ad-8001350
[civil]: ../README.md#civil-violence-epstein-2002
[tags]: ../README.md#tag-cooperation-riolo-cohen--axelrod-2001
[spatial]: ../README.md#spatial-games-nowak--may-1992-and-its-critics
[culture]: ../README.md#axelrod-culture-axelrod-1997-and-its-docking-with-sugarscape
[classes]: ../README.md#emergence-of-classes-axtell-epstein--young-2000
[ethno]: ../README.md#ethnocentrism-hammond--axelrod-2006-and-its-critics
[opinions]: ../README.md#bounded-confidence-hegselmann--krause-2002
[structure]: ../README.md#social-structure-cohen-riolo--axelrod-2001
[dpd]: ../README.md#demographic-prisoners-dilemma-epstein-1998-and-its-replication
[norms]: ../README.md#norms-and-metanorms-axelrod-1986-galán--izquierdo-2005
[image]: ../README.md#image-scoring-nowak--sigmund-1998-and-its-critics
[agreement]: superpowers/specs/2026-10-01-agreement-spike.md
[farol]: superpowers/specs/2026-10-02-farol-audit/numerical-review.md
[ants]: ../README.md#ants-and-recruitment-kirman-1993-alfarano--milaković-2007
[thresholds]: ../README.md#threshold-models-granovetter-1978-watts-2002
[retirement]: superpowers/specs/2026-10-03-retirement-audit/README.md
[punishment]: ../README.md#altruistic-punishment-boyd-gintis-bowles--richerson-2003
[zi]: ../README.md#zero-intelligence-traders-gode--sunder-1993-cliff-1997
[bali]: ../README.md#balinese-water-temples-lansing--kremer-1993-janssen-2007
[schelling]: ../README.md#schelling-segregation-schelling-1971-epstein--axtells-variant-vi-4-to-vi-7
[tipping]: ../README.md#schellings-tipping-schelling-1971-pp-167186-1969
[variations]: ../README.md#variations-on-schelling-milestone-32
[firms]: ../README.md#the-emergence-of-firms-axtell-1999-axtell-2013
[collusion]: ../README.md#algorithmic-collusion-calvano-calzolari-denicolò--pastorello-2020-and-its-critics
[auctions]: superpowers/specs/2026-10-02-q-learning-auctions-findings.md
[polarity]: superpowers/specs/2026-10-02-emergent-polarity-findings.md
[geosim]: superpowers/specs/2026-10-03-geosim-findings.md
[dp]: superpowers/specs/2026-10-05-democratic-peace-precision-findings.md
[minds]: studies/2026-09-27-minds.md
[m1]: ../README.md#minds-1-the-utility-mind-and-the-ideal-free-distribution
[m2]: ../README.md#minds-2-a-and-walking
[m3]: ../README.md#minds-3-memory-belief-and-truffles
[m4]: ../README.md#minds-4-goap-and-the-marginal-value-theorem
[m5]: ../README.md#minds-5-caching-for-the-future
[m6]: ../README.md#minds-6-theft
[m7]: ../README.md#minds-7-the-evolution-of-hoarding
[m8]: ../README.md#minds-8-watching
[m9]: studies/2026-09-27-minds.md#minds-9-spatial-hoarding-paid-defense-and-seasonal-inheritance
[p3]: superpowers/specs/2026-10-06-minds-protection-findings.md
[eprepair]: superpowers/specs/2026-10-03-emergent-polarity-sequential-amendment.md
[epdomestic]: superpowers/specs/2026-10-03-emergent-polarity-domestic-amendment.md
[eptelemetry]: superpowers/specs/2026-10-03-emergent-polarity-telemetry-amendment.md
[gsdef]: superpowers/specs/2026-10-03-geosim-defender-threshold-amendment.md
[gsrecorder]: superpowers/specs/2026-10-03-geosim-postmeasurement-recorder-amendment.md
[dpreport]: superpowers/specs/2026-10-04-democratic-peace-reporting-amendment.md
[dpauthor]: superpowers/specs/2026-10-03-democratic-peace-author-and-followup-reading-notes.md
[burrow]: studies/2026-10-03-burrow-excavation-reading.md
[foraging]: studies/2026-10-04-foraging-construction-reading.md
[gsaudit]: superpowers/specs/2026-10-03-geosim-artifact-audit.md
[gsdamage]: superpowers/specs/2026-10-03-geosim-damage-incidence-audit.md
[dporiginal]: superpowers/specs/2026-10-03-democratic-peace-findings.md
[epexport]: superpowers/specs/2026-10-03-emergent-polarity-export-amendment.md
[gsportable]: superpowers/specs/2026-10-03-geosim-portability-amendment.md
[gsfinite]: superpowers/specs/2026-10-03-geosim-finite-technology-amendment.md
[rousseau]: https://library.uc.edu.kh/userfiles/pdf/53.Democracy%20and%20war%20%20institutions%20norms%20and%20the%20evolution%20of%20international%20conflict%20.pdf#page=358
[farolreadme]: ../README.md#el-farol-and-the-minority-game-arthur-1994-challet--zhang-1997
