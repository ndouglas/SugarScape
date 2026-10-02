# Emergent polarity: verified research handoff

**Date:** 2026-10-02. Brainstorming evidence, not an approved design or implementation plan.
No SugarScape polarity simulations have been run. Numbers here are source statements or
documentary checks. Read alongside `2026-10-01-cederman-reading-notes.md` and the geopolitics
lineage plan; the corrections below supersede their affected statements.

## Intent and scope under discussion

Implement the lineage's originator as a new kind, `polarity`, then test its account of
conquest, alliances and limits to empire growth. The existing campaign proposes the 1994
model and book chapters 4–5, including tax/revolt and overextension. Later GeoSim war-size
and democratic-peace models remain separate milestones. Findings, including reconstruction
dependence and failed reproduction, are the deliverable. Agreement with the figures is not
a reason to change an unreported rule after measuring.

Readers independently examined the original, author-code lineage and critics. The controller
checked the campaign plan and existing `civil`, `bali` and `auctions` integration patterns.
Keep source PDFs and extracted third-party code outside tracked product files.

## Corrections to the October 1 notes

1. **Literal TFT need not alternate forever.** A pure TFT dyad beginning DC alternates CD/DC.
   But Fig.7 also allows predator initiation. If its guard checks the predator's provisional
   own actions, it can initiate again when its TFT action is C, producing DD if superiority
   and target selection still permit it. DD then persists through ordinary TFT. A guard that
   counts unresolved dyadic hostilities produces a different result. This is a deduction
   from the printed algorithm, not evidence about the authors' simulator. Separate the
   action-memory rule from the Schlieffen guard.
2. **The original result plots have no point markers.** Digitize curves and stacked boundaries
   at the eight stated experimental settings. Interpolated positions are not extra observations.
3. **The allocation/alliance question was preliminarily tested by the author.** Book p.121
   fn5 says preliminary runs confirm dependence. No factorial dataset or full protocol is
   reported. Original p.530 fn37 already cautions that changed alignment mechanisms might
   reverse the alliance result.
4. **Störmer is not the only replication.** Radax's 2010 thesis also examines EPM, and a 2019
   Köhler-Bußmeier/Störmer paper follows up the 2018 work. Their protocols and attribution
   require separate treatment.
5. **“Three quarters inert” is too precise.** Störmer's claim concerns configuration-average
   final state counts near the initial count, rather than an exact share of runs with zero
   changes. Her prose and appendix code also differ. Neither is an automatic protocol for
   the original model.
6. **GeoSim0 is later comparative evidence.** The 2004 source identifies the APSR 2003 model.
   It does not implement the original predator-share/trust-alliance experiment. Do not claim
   original-code docking or use later defaults to resolve original ambiguities.

## Original model: source-grounded reconstruction

Primary sources are the local `cederman-1994-isq-emergent-polarity.pdf` and
`cederman-1997-emergent-actors-in-world-politics.pdf` in `papers/geopolitics/`.
References below use printed pages. The original's PDF page index is printed page minus
499; the book's is printed page plus 15. Rendered figures, rather than OCR subscripts, were
used to check the decision and combat rules.

The reported experiments use a **10×10 grid**, not the 20×20 illustrative maps. Primitive
actors have predator/status-quo types and initially independent sovereignty. Corporate
actors have a capital and directly ruled provinces; the capital determines foreign strategy.
Four-neighbor bounded topology is supported by the source's “at most four” neighbors and
contrast with Duffy's wrapping lattice (1994 pp.507,511). Non-wrapping is a supported inference.
Initial stocks are independent normal draws with mean 50 and SD 10. Each primitive unit
contributes a period harvest drawn with mean 2 and SD 5 (pp.511,514–515). Corporate harvests
sum primitive draws. Floors and nonpositive-stock policies are unspecified.

Decisions precede interactions and structural changes (pp.511–514). Both parties choose
before encounters resolve. A randomized locking mechanism prevents repeatedly changing
units in a period (pp.510–511 fn7), but its lock scope and skip/defer behavior are unreported.
Serial update order is therefore a robustness treatment, not a silent optimization.

Status-quo states play previous-action TFT. Predators also initiate against their weakest
sovereign neighbor when the no-other-action/war guard permits it and their **total** stock
ratio exceeds the superiority threshold (Fig.7, pp.512–513). Target ties and the precise
guard interpretation are unreported. An attacking border cell is sampled first, then an
adjacent defender cell. This differs from uniform sampling over border edges. Persistence
or resampling of this path remains ambiguous.

Resources are split equally across distinct sovereign neighbors, not border edges. These
**local allocations** determine victory; initiation uses total stocks. Victory requires a
strict ratio above the victory threshold. In Fig.8, CC has no loss; DC damages only the
defender by .05 times the attacker's allocation; CD is its mirror; DD damages both parties
by .05 times their opponent's allocation. Timing of victory relative to losses and harvest
is insufficiently specified. Book chapter 5 explicitly suppresses DC/CD losses (p.109 fn1).
That chapter distinction must survive source profiles.

Primitive capture transfers sovereignty. Province capture transfers that province. Capital
fall fragments the corporate actor; neighboring states subsequently compete for the
fragments (p.515 fn19). The original R5 example supports this sequence rather than instant
whole-empire capture. Disconnected provinces become independent, regain latent types and
receive an equal per-unit share of the former center's stock. Captured stock accounting and
combined transfer/collapse ordering remain reconstruction choices.

Alliances update neighbor trust before decisions and form after structural changes. Trust
is `(1-s)*old + s*(±1000)`, with negative learning .5 and positive .01 (p.516 fn23).
At least two states naming the same prime threat form a defensive coalition. Deterrence
uses coalition resources, without merging sovereignty or stocks. Attacker support is
conditional on the alliance naming the victim as its threat. Aggression against a fellow
member expels the aggressor. Trust initialization, threat threshold, event observation and
the timing of coalition obligations are unspecified (pp.515–517).

## Original experiments and statistics

The base design is predator shares `{0,.05,.1,.2,.4,.6,.8,1}` × ratios `{2,3}` × **20 seeds**,
stopping at one surviving sovereign actor or **1,000 periods** (p.517). Repeat with alliances
for the same inherited dimensions (pp.523–524): 640 sessions across both treatments.
Seed values and treatment pairing are unavailable. Predator-count wording supports exact
counts with randomized placement, but does not specify the sampling algorithm.

Fig.10 is arithmetic mean terminal polarity. Early hegemonies contribute terminal count 1.
Figs.11 and 13 use five categories: **1, 2, 3–10, 11–90, 91–100**. “Power politics” means
**2–10**, excluding unipolarity (p.518). The exact textual reference cell at defense/no
alliances/60% predators is **5/9/5/0/1** across these categories. The book reuses these results;
it is not an independent validation sample.

Digitization can recover 16 mean points and four cumulative category boundaries at each
of eight settings in two panels of each stacked figure. Record source images, axis
calibration, sampled coordinates and pixel uncertainty; enforce twenty runs per setting.
Do not invent exact counts from obscured boundaries. The source supplies no acceptance
tolerance, uncertainty estimates or inferential test. Statistical decision rules must be
registered in the design before native runs.

Distinguish the original hypotheses from the author's reported counterexamples. Defense
can raise mean survival while raising universal-empire probability and lowering the
2–10-state probability. Alliances protect some low-predator settings while increasing
hegemonic outcomes at higher predator density. Reproducing these figures differs from
confirming a blanket “defense and alliances stabilize power politics” proposition.

Illustrative maps and the single 400-unit trajectory lack seed provenance. They are not
frequency targets. Resources destroyed, war durations and episode end causes can serve
the war-and-society study, but have no original quantitative reference dataset. Resource
loss is not a count of people killed.

## Chapter 5 boundaries

Proportional resource allocation (PRA) uses opposing front commitments. Its passive-front
formula describes conditional commitments which need not sum to the actor's stock. The
prose attack inequality conflicts with pseudocode, and a worked example conflicts with the
formula. Preserve source distinctions and test the published equations independently.
The allocation/alliance comparison is a robustness question motivated by preliminary
author runs (p.121 fn5), without published numerical replication targets.

Two-level action allows provinces to retain resources, pay taxes on harvest and revolt;
the center divides commitments across domestic and foreign fronts. Foreign initiation is
blocked during domestic war. The tax sweep uses all predators and offense ratio 2 to
1,000 periods (pp.123–126). Twenty replications are inherited, rather than restated;
the exact tax settings require figure extraction and documented inference.

Overextension combines stochastic decisions/combat with tax `.4 * .7^distance`
(pp.126–134). The illustrated response curve has threshold 3 and exponent 5, but the
narrative run's threshold and competing victory draws are not fully specified. Its single
run lasts at least 3,800 periods. Do not impose the base model's 1,000-period experiment
or claim a representative trajectory match to an unknown seed.

## Design implications

Use an explicit ambiguity manifest with source citations and documented selected defaults.
The critical dimensions are action memory and initiation guard; path memory; structural
locking; victory timing; collapse and stock transfers; trust and coalition timing; and
numerical handling of nonpositive stocks. Avoid an uncontrolled Cartesian product. A
literal reconstruction is the baseline, with bounded, registered sensitivity families.

Follow existing model/schema/export/survey interfaces and deterministic seeded RNG. Keep
territory/sovereignty, decision/alliance, combat/accounting, statistics and rendering in
focused modules. Shared native/WASM mechanics and mechanistic tests matter more than an
early general-purpose GeoSim abstraction. CI should exercise seconds of mechanics and
integration; native studies retain complete sessions outside CI. The later written plan
will use verified scratch implementation and byte-for-byte reconstruction, then subagent
execution and a fresh final reviewer, following the previous milestone's workflow.

## Appendix A: detailed author-code audit


Research date: 2026-10-02. Read-only product repository; archives extracted only into `/tmp/polarity-author-code/`. No model executed, dependencies installed, native study run, or source copied into SugarScape. Findings below distinguish static source evidence from untested runtime feasibility.

#### Conclusion and evidence boundary

Neither supplied archive is the original 1994/1997 Emergent Polarity Model (EPM). They are descendants. They offer valuable evidence for persistent war, hierarchical actors, conquest and contiguity, but cannot validate an EPM reproduction by docking. Reproducing GeoSim0 with their defaults would reproduce a different experiment. The book's original Pascal code remains unfound in this search, rather than proven unavailable.

The strongest internal provenance is `geosim2004/geosim0/src/ch/ethz/icr/geosim0/Model.java:36–53`: it names Girardin and Cederman, explicitly distinguishes the architecture introduced in the 1997 book from the current model corresponding to APSR 2003. The same declaration occurs in Actor and Relation. GeoSim0 omits that paper's technological shock mechanism, so the header is evidence of lineage, not proof that all APSR experiments are present. The 2004 zip has June 2004 source/class timestamps. The GROWLab outer archive has August 2017 timestamps, but inner model source identifies 2006 development: this is a 2017 distribution snapshot, not a new 2017 model.

Official primary sources agree: [ICR GeoSim](https://icr.ethz.ch/research/geosim/) describes migration from Repast into GROWLab; [ICR GeoContest](https://icr.ethz.ch/research/geocontest/) calls GeoContest a simplified GeoSim with pluggable conquest strategies. [The EPM publication page](https://icr.ethz.ch/publications/emergent-polarity/) identifies the 1994 ISQ article and DOI 10.2307/2600863 but provides no model source supplement. Historical source location recorded in existing notes: http://www.cederman.ethz.ch/teaching/archive/compmodels/ws2004/models/geosim0.zip (web fetch failed in this audit; local archive inspected).

Archive hashes (SHA-256):
- 2004 GeoSim0: `d80e982e92132e265ad28dbd742b76c31b03db9e023f8cd9029c800deb6eccad`
- GROWLab 0.9.5 snapshot: `2f20146bc34f9521302f251ffd6388443c4bbd21e8db47a527c120d220e98a5c`

#### Licensing correction

2004 Model/Actor/Relation carry explicit GPL version 2 or later notices (Model.java:1–20). No GPL source should enter the product implementation.

The existing reading-notes statement that the entire GROWLab archive is GPL is too broad. Framework `src/ch/ethz/icr/growlab/model/AbstractModel.java:1–20` and `simulator/SimulatorFactory.java:1–20` carry LGPL 2.1-or-later notices. GeoSim0 plugin `Geosim0Model.java:1–3` instead has a placeholder “Your Corporation. All Rights Reserved” header; GeoContest is mixed: `AbstractStrategy.java:1–20` and `Actor.java:1–20` GPL2+, `Strategy.java:1–3` a placeholder. A source ZIP containing these files is not sufficient evidence for a uniform plugin licence. This is a provenance uncertainty to resolve before redistribution, not a barrier to this read-only comparison. Separate third-party jars also need their own inventory if redistributed.

#### 2004 Repast GeoSim0: actual protocol

Paths in this section are relative to `/tmp/polarity-author-code/geosim2004/geosim0/src/ch/ethz/icr/geosim0/`.

**Reset/defaults.** Model.setup (139–170) calls superclass setup, clears actors/world, sets inherited seed=1, bounded 30×30 Object2DGrid, initPolarity=200, probAttack=.2, superiorityThreshold=3, slope=0, mobile-resource proportion=1, minThreat=−3, runUntil=10000. minThreat is assigned during setup, not dynamically coupled to later threshold setter changes. buildModel (177–211) creates one permanent atomic unit per cell, shuffles sovereign actors, builds von Neumann relations, then artificial initialization absorbs cells into selected founder states. Initial module (256–274) repeatedly chooses founders with reinforcement (each successful absorber appended again), until 700 absorptions. This is preformed compound states, unlike EPM's 100 independent units.

**RNG/update order.** Uniform draws use Repast Random; list shuffles use SimUtilities. The exact inherited seed-to-RNG engine/reset convention is not recoverable from this ZIP because Repast is absent. Setting the same integer seed in another runtime does not imply identical streams. step (218–232): resource update → allocation → decisions → dyadic interactions → claims resolved structurally → list rebuild/shuffle only if a claim occurred. makeList (705–719) also shuffles after initialization. This is not a fresh actor shuffle on every no-change tick. Decisions use old bilateral histories; interactions process each dyad once by lower actor id (357–365), then refresh history buffers. Structural claims mutate topology sequentially and recheck agent/target government ownership (676–697). There is no generic EPM per-cell lock field; ownership validity is a different conflict-resolution mechanism.

**Resources/allocation.** Actor constructor (125–145) initializes every cell to 10. Actor.updateResources (286–300) replaces capital resources by 10×(1+province count) every step. No Normal(50,10) start or Normal(2,5) harvest, accumulation, negative-resource damage, or war-cost matrix is applied. Actor.allocateResources (347–380) is modified PRA: fixed resources divided across neighbors, mobile pool responds to prior opponent allocations on active fronts; each passive front gets counterfactual mobilization. With mobil=1 and no active fronts each front can announce the entire state's resources. That is not equal actual splitting in EPM chapter 4.

**Action persistence.** Actor.decide (452–493) applies TFT, then overrides to D if self previously played D—explicit grim trigger (465–467). Every state has the same probability of attempting unprovoked attack. An idle state samples a random neighbor, not the weakest neighbor, and compares LOCAL allocated resources (502–524), not EPM's total resources. There is no fixed predator/prey orientation or predator-share parameter. The grim override is decisive evidence that later GeoSim deliberately keeps a unilateral attack alive until DD war or reset. It supports offering a persistent-war interpretation for EPM but cannot prove which original Pascal interpretation was used.

**Interaction/victory.** Model.interact (378–416) arbitrates two simultaneous unprovoked attacks with a coin flip, assigns offensive/defensive roles on CD/DC, and only invokes battle on DD. It inflicts no resource damage in any branch. Victory (424–438) compares current front resources BEFORE any damage (there is none), strict own>other×threshold; defensive threshold is reciprocal. Defender victory resets BOTH actions as a stalemate (414–415,446–448), rather than letting the defender seize the attacker province. Attacker victory creates a territorial claim. No-claim DD wars can persist. Relation.resetAction (189–198) clears allocation/path/history/claim/offensive flag; Relation.update (204–206) snapshots actions.

**Conquest/capital/fragmentation.** Model.conquest (660–669): transfer a foreign province, check remaining state's capital connectivity; conquered compound capital triggers collapse; add captured unit to conqueror. topology (612–637) flood-fills from capital and releases unreachable provinces separately. collapse (574–585) removes all provinces before capital incorporation. addProvince (462–480) transfers the captured unit's then-current stored resources to the capital. removeProvince (522–568) makes unit sovereign, resets fronts/rebuilds relations but does NOT allocate an equal fraction of capital's current resource stock; next resource update supplies 10 per independent unit. That is not the book p.89 equal-share independence rule.

**Alliances.** Actor.perceive (308–333) measures the negative LOCAL power ratio and marks mainThreat/security. There is no Alliance class or diplomacy/action-support phase. ModelBatch calls perceptions after step (95–106) for security output; this does not make states form alliances. Chapter-4 behavioral trust accumulation, common-threat alliances, pooled deterrence and allied retaliations are absent.

#### GROWLab GeoSim0 differs even from the 2004 ZIP

Nested archive `models/geosim0-0.9.0.zip` extracted into `/tmp/polarity-author-code/growlab0/`; relative source directory `src/ch/ethz/icr/growlab/model/geosim0/`.

Geosim0Lab constructor (122–154) sets seed=1, grid 30×30/VonNeumann/radius1, initPolarity=180, probAttack=.2, superiority=3, slope0, mobil1, but **cumulRes=true, warCost=.1, taxRate=1, harvest mean1 sd0, decisiveBattles=true**. These latter fields are not added as public parameters (136–137). Merely selecting “GeoSim0” in the later platform therefore changes scientific behavior.

Geosim0Model.buildModel (96–136) explicitly creates Colt MersenneTwister(seed), sharing engine between uniform and normal distributions. step (144–158) retains phases above and adds end-of-step perceptions. Actor.updateCumulativeResources (290–316) adds harvest plus delayed front damage, distributes untaxed harvest to provinces, clamps total resources at zero, and draws normal variates even when sd=0. A tape/count-aware docking adapter must preserve those apparently unnecessary draws. Geosim0Model.interact (314–350) now computes .1 damage for CD/DC and DD; when neither side claims, decisiveBattles resets the front immediately. Constructor hard-codes its own private decisiveBattles=true (79–89), so changing the similarly named lab field alone would not change this branch. The later model therefore combines grim persistence with short no-victory wars, not simply the 2004 persistence protocol.

#### GROWLab GeoSim2 and GeoContest are downstream references

Nested GeoSim2 source `growlab2/src/ch/ethz/icr/growlab/model/geosim2/`. Geosim2Lab:160–214 declares defaults as Cederman 2003/Cederman–Gleditsch 2002: seed5 representative run, 50×50, superiority/victory3, logistic slope20, attack .01, mobil .5, stalemate .1, democracy false, collective security0, alliances false, minThreat−2.8, obligation .5, contributions .5, stationary starts500, shockSize−20 and pShock=.0001; createSystem=true, initPolarity200, warCost=.1, secession unsupported. The shockSize parameter is constructed but absent from the addParameters call (189–190): exposed UI parameter list is not a full model-state inventory.

Step (`Geosim2Model.java:973–1021`) processes shocks after stationary onset → resource/regime changes → optional perceptions/diplomacy/alliance resources → optional collective security → allocation → decisions → interactions → war magnitude → structural changes/list shuffle → war garbage collection. Its optional alliances retain “two states share a threat” membership logic (366–409), but Actor.perceive (501–523) uses negative LOCAL capability ratios, excluding democratic dyads when appropriate, rather than the behavioral trust update of EPM ch4. Alliance.updateRes (115–123) pools distance-discounted FRONT allocations, not simply total member resources. Collective security/regime change, technology shock/diffusion, war episode counting and distance discounts must not leak into an EPM94/97 baseline.

GeoContest official page claims replicability of its 2008 strategic-conquest experiments in GROWLab; it does not claim replication of Cederman's original EPM. Its strategy interfaces are useful historical context, not a ready EPM action protocol.

#### External execution and honest comparison

No execution was attempted. JDK25 is installed locally, but that is not verified compatibility with these 2004/2006 classes.

2004 archive includes model source/classes and Colt jar, but **no Repast jar**. Its IntelliJ module file explicitly requires an external `RePast` module; imports additionally include Repast local recorder and legacy ProActive cluster interfaces. Model.main starts GUI; ModelBatch provides headless-style recording through Repast and emits final polarity/security (ModelBatch:81–111), not a complete state/event trace. initPolarity has a setter but is omitted from its parameter name array (73–75), another runner mismatch to resolve. Obtain a pinned compatible Repast distribution and run outside repo before saying it is executable. No modern Maven/Gradle build or complete launch wrapper was found.

GROWLab snapshot bundles growlab.jar, Colt, many Java3D/Groovy/chart/UI/support jars and compiled plugin classes. This is more promising for external execution. SimulatorFactory:34–55 accepts first argument `batch`, choosing ParallelBatchSimulator; default is DockingSimulator. **DockingSimulator here means GUI dockable windows, not scientific model docking.** Lab setup installs file collectors; GeoSim0 emits seed/time/polarity/security (Geosim0Lab:165–167), not ownership, actions, front allocations or RNG trace. Parallel execution also deserves reset/isolation inspection because GeoSim2 Alliance uses static model/list/counter state (Alliance:22–46). Do not infer repeat-run independence from seed parameter alone.

A feasible future reference protocol is process isolation plus pinned archive/version/hash, complete settings including hidden fields, finite horizon, explicit output schema, then small hand-constructed motif traces and random-draw tapes before aggregate multi-seed comparisons. Original EPM has no comparable executable source protocol recovered here. Dock a future GeoSim reproduction to GeoSim0/2 only under its matching semantics; use their conquest and persistence rules as triangulation/robustness evidence for EPM, never as proof of EPM fidelity.

#### Current availability / supplementary searches

[ICR GROWLab](https://icr.ethz.ch/research/growlab/) currently advertises v0.9.7 (2022 release), and [download page](https://icr.ethz.ch/research/growlab/download/) offers Windows/macOS/Linux installers. [Installation instructions](https://icr.ethz.ch/research/growlab/documentation/installation/) still describe an account-required SVN source repository, Java3D and an IntelliJ project. No public original Pascal EPM supplement was linked from the inspected ICR publication/model pages. Web searches for Cederman/EPM/Pascal/GitHub and primary GitHub repository searches for Cederman, EmergentPolarity, emergent polarity and Störmer returned no verified original source. Negative search is provisional.

One new public candidate is [sauerberg/Cederman-ABM-IR](https://github.com/sauerberg/Cederman-ABM-IR), containing NetLogo model, experiment CSVs and analysis notebook; it is a third-party reimplementation, not author code, and not yet audited against EPM. It invalidates an unqualified “no NetLogo implementation exists” statement in the existing notes. No verified Störmer Python source repository was found by these searches.

Störmer's [2018 thesis](https://reposit.haw-hamburg.de/bitstream/20.500.12738/8337/1/BA2239713.pdf), printed pp.35–37, describes preserving EPM call/algorithm structure, comparison using final state count, parameters adjusted when Cederman did not specify them, and incomplete original code requiring reverse engineering. Its claimed approximate reproduction is weaker than source-level docking. The university-hosted [2019 Köhler-Bußmeier–Störmer working paper](https://edoc.sub.uni-hamburg.de/informatik/volltexte/2019/247/pdf/koehler_stoermer_meta_analysis.pdf) is an additional primary sensitivity source; it should be read alongside the thesis before generalizing inertness. I have not inspected its supplementary source.

#### Decisions recommended for brainstorming

1. Keep EPM94/EPM97 as paper/book-first specifications; label persistent grim-trigger combat as a named alternative supported by later author code, not recovered original intent.
2. Keep author-code artifacts external and pinned; no dependency or GPL source imports to SugarScape.
3. Separate EPM uncertainty tests from downstream GeoSim execution. External reference execution is worthwhile later, but its result cannot settle EPM predator/prey, equal allocation, stochastic harvest, trust-alliance or independence-resource ambiguities because those rules differ or are absent.
4. Correct source-lineage/licence/default claims in eventual notes, and retain NetLogo/Python candidates as independently audited replicas rather than authoritative implementations.

Additional static audit cautions: 2004 `Actor.selectAgent` (415–428) calls `getNeighbors()` on the attacking capital while its comment claims neighbors of the target; do not silently treat comment as actual contiguous-path enforcement. This warrants a hand-built compound-state motif before trusting paths. GROWLab GeoSim0's `runUntil=10000` is a field assignment only; no use beyond declaration/assignment was found in that plugin, so an external runner must explicitly enforce horizon. Its `buildUI` replaces collectors with graphical collectors; reference output depends on batch versus GUI path. Neither discrepancy was exercised at runtime.

## Appendix B: detailed critic protocols


Read 2026-10-02. Research only; no repository changes. Printed page numbers below unless explicitly called PDF pages. Primary local documents: Störmer 2018 BA thesis, Duffy 1992 MCM, Cederman 1997 pp.117–121; campaign and prior reading notes. Supplementary primary sources: Radax 2010 TU Wien dissertation, Köhler-Bußmeier & Störmer 2019 working paper, Lazer 2001 review.

#### Findings that change the campaign's premise

1. The statement “the only replication (Störmer 2018)” is false. Radax 2010 directly attempted EPM replication and reported failure. Its scope must be distinguished from that thesis's subsequent GeoSim code audit.
2. Störmer did not establish that precisely 75% of independent EPM runs were inert. Her plots contain **110 configuration means**, each over ten runs. She describes approximately three quarters in a *small region near* (power struggles=0, final states=100). Exact (0,100) is a special case; proximity is not exact inertness. The 2019 followup describes stable or slightly decreasing state counts across a range of power-struggle values.
3. Her reported design and printed implementation disagree on horizon, attack threshold, initial resource spread, and battle costs. Her reimplementation also differs materially from the originator's mechanisms. Its empirical 75%-like result cannot be a numerical gate on a faithful Cederman engine.
4. Duffy concerns Bremer–Mihalka/Cusack–Stoll worlds, not a serial implementation of Cederman EPM. His concurrency changes the number of initiations per iteration, targeting resolution, some rules, distributions, and geography. A synchronous/asynchronous EPM comparison is a useful *new robustness control*, not a reproduction of Duffy's experiment.
5. Chapter 5 footnote 5 raises an actual originator robustness question about the alliance result under proportional resource allocation (PRA), without specifying a numeric expected outcome or published experiment. Register a precise interaction test, not a guessed “must reverse” result.

#### Störmer 2018: recoverable design

Source: [HAW repository record](https://reposit.haw-hamburg.de/handle/20.500.12738/8337), local PDF `papers/geopolitics/stoermer-2018-haw-ba-thesis-emergent-polarity-parameter-variation.pdf`. Its only repository attachment is the PDF; full Python code is printed in appendix A.1 (beginning appendix i, PDF67). No separately downloadable author repository was found in targeted searches; absence from this search is not evidence none exists.

##### Sample units and horizon

- Planned: 100 random configurations × ten simulations = 1,000 records (pp.43–44,49).
- Analysis adds ten initial test configurations generated under the same conditions; hence **110 configurations × ten**, nominally 1,100 runs (p.49). Figure 7.2 explicitly labels n=110 (p.51, PDF59). Those plotted observations are configuration means; fig.7.1 additionally shows standard deviations.
- Main text says **600 time units**, justified by stabilization within the first 500 (p.44). Appendix Main.run_simulation sets **1000**, line68 (appendix iii, PDF69). Thus retain `stoermer-text-600` and `stoermer-listing-1000` as different evidence readings; the thesis does not resolve which executed artifact generated the data.
- Listing uses bounded **10×10**, alliances always enabled, stops early at hegemony (appendix iii,v; pp.39–40). Minimum resources fixed at zero and sensitivity fixed at seven (appendix iv). No documented seed list is recovered.

##### Discrete sampling law

All printed calls are Python `random.randint(a,b)`, inclusive discrete uniform draws. Several dimensions are conditionally bounded by previously drawn means, so this is not an independent rectangular product prior.

| Dimension | Main text (pp.44–46) | Appendix actual assignment (i–iii) |
|---|---|---|
| Trust threshold | integers1…25 | Uinteger[1,25] |
| Threat threshold | integers1…25 | Uinteger[1,25] |
| Winning resource ratio |1.1…5 step0.1 | randint(11,50)/10 |
| Attack resource ratio |1.1…5 step0.1 | **randint(5,200)** using harvest bounds |
| Initial resource mean I |integers5…200 | Uinteger[5,200] |
| Harvest mean H |integers5…200 | Uinteger[5,200] |
| Initial deviation DI |integers0…I | sampled Uinteger[0,I], but **not applied** |
| Harvest deviation DH |integers0…H | Uinteger[0,H] |
| Applied initial deviation |should DI | **DH** via sys.deviation_initial_resources=self.deviation_harvest, line92 |
| Battle cost |1%…100% of I | **Uinteger[0,I]**, allows zero |
| Predator share P |integers0…100 percent | Uinteger[0,100] |
| Attack probability A |integers0…100 percent | Uinteger[0,100] |
| Tax rate |integers0…100 percent | randint(0,100)/100 |
| Distance tax discount |integers0…100 percent | randint(0,100)/100 |

The listing calls for individual state types and spontaneous attacks use **randint(1,101)** followed by `<= rate`; realized Bernoulli probability is rate/101, not rate/100. Thus a rate of100 does not yield all predators or certain attacks. Initial resources and per-tick harvest are **uniform integers around their means**, not the normal distributions of the originator (p.39 listing5.3; appendix vii,x). Applied initial spread DH can exceed I, producing negative initial resources despite the main text's assurance of nonnegative values.

##### Reimplementation deviations, not interchangeable with originator

The appendix visibly implements:

- **Moore neighbors**: both coordinate loops run from x−1 to x+1 and y−1 to y+1, excluding self; no cardinal-only restriction (appendix xvi, PDF82). Originator is four neighbors.
- Predator attack when own/weakest resources **< attack_threshold**, not >. Confirmed in rendered p.39 listing5.2 (PDF47) and appendix xviii. With appendix thresholds5…200, this encourages attacks at ratios below high thresholds.
- Threat likewise marks threatened for new_balance **<** threat_threshold (appendix xvii). Sensitivity7 is inserted into a recurrence containing `(1−sensitivity)`; this deserves literal documentation, not silent normalization.
- A state already at war chooses defection again (appendix xvii–xviii), hence explicit persistence rather than the printed TFT rule. TFT re-initiation may itself sustain DD; the Schlieffen guard that prevents spontaneous re-initiation must be distinguished from TFT semantics.
- Damage is a **fixed battle_cost subtracted from both primitive-unit resources**, including one-sided attacks (appendix ix), not a per-active-front proportional originator loss.
- Allocation equalizes the total over constituent **primitive units**, assigning the same resources to each province/capital (appendix xv). Combat compares each unit's resources divided again by its state_size when both state sizes are positive (p.40 listing5.5).
- Taxes are collected from province **resource stock**, then the capital calls equalizing allocation (appendix x,xv). Tax distance is **min(|dx|,|dy|)** despite the preceding comment saying largest coordinate distance; discount factor is discount^distance. This differs from the Manhattan/path distance interpretations of the originator's exponential tax decay. Taxation/discount dimensions therefore make the experiment a hybrid of extensions rather than base chapter4 sensitivity alone.
- Decisions are grouped before interactions, but interaction and structural-change loops mutate objects serially (appendix viii,x). It is not sufficient to call the whole listing synchronous solely from its phased structure.
- Mutable defaults occur at class level (`states`, pairings, children, allies, histories; appendix iv–v,xiv). Without executing a faithfully recovered artifact one cannot tell whether all intended per-run state was reset. This is another reason the printed listing cannot certify its numerical claim.

Störmer explicitly allows reverse engineering because original configuration/code were incomplete, reports small differences, and assumes equivalent functionality based on requirements and unit tests rather than demonstrating final relational equivalence (pp.36–37,42). That is evidence of the author's validation procedure, not proof of the original result.

##### What counts as inert, and what was actually measured?

Main text defines power struggles as **conquests and state disintegrations** (p.46), not all attacks, battles, war starts, or war durations. Exactly (0,100) means zero such structural events and unchanged final state count (p.53). It does not establish zero fighting or unchanged resource/trust state. The code increments `power_politics` on secession, collapse, and conquest paths (appendix xi–xiii), while casualties/unsuccessful attacks are not that counter.

The three-quarter statement is an eyeballed density-region description with **no exact neighborhood radius, cutoff, confidence interval, regression coefficient, or published numerical count for exact-zero events** (pp.49,53). Do not invent an inertness tolerance. A legitimate deterministic new measure is exact zero structural events plus final primitive-state count100; show its fraction and intervals, and separately label the paper's near-corner region descriptive.

#### Duffy 1992: what “serial” and “concurrent” mean

Source local `papers/geopolitics/duffy-1992-mcm-concurrent-interstate-conflict-simulations-serial-assumption.pdf`, MCM16(8/9):241–270, [DOI](https://doi.org/10.1016/0895-7177(92)90099-7).

##### Protocol differences

Serial Cusack–Stoll worlds admit **one interstate conflict per iteration** (p.242). A selected initiator that declines targeting or is deterred by the defending alliance ends that iteration. Concurrent worlds select many possible initiators; one abandoning attack does not abort everyone else's iteration (p.247).

Serial geography has98 hexagonal territories; concurrent geography is **8×16=128 cardinal-grid territories**, optionally toroidal (pp.244–245). Printed figure captions misstate96/126, while prose and 2³×2⁴ identify98/128. Toroidal adjacency removes peripheral advantages and alters alliance opportunities; disabling cross-edge communication is the controlled boundary option.

Concurrent primitive initiation uses Q=2^x with x=3+(EP_i−mean(EP))/sd(EP) (equation7 visually checked); each sovereign draws integer0…31 and initiates if Q exceeds the draw (p.252). Rational serial initiation samples one state's maximum positive expected utility proportional to the system sum; concurrent rational initiation admits every state with a positive-utility target unless attacked (p.253). This changes event opportunities per iteration, not just memory snapshot timing.

When multiple initiators target one state, the **weakest initiator goes first** and stronger initiators withdraw. States also withdraw attacks if themselves attacked. Targeting/withdrawal repeats, with an arbitrary finite cycle cap in practice (pp.253–254). Alliance phases: defenders recruit, attackers recruit if disadvantaged, defenders reinforce, attackers may withdraw. States already at war/allied cannot supply another alliance, and simultaneous bids are resolved by a chosen bid (p.254). These are substantial collision policies, not generic concurrent semantics.

Full concurrent model additionally varies misperception over time, removes self-misperception, and continues indecisive wars unless **both principals withdraw**; allies cannot switch to the previous enemy immediately (pp.248,257). It also corrects a defender-favoring serial combat-outcome rule (p.256), and runs civil-war checks every iteration with different weights (pp.249–252). Civil-war participants cannot initiate or join alliances that iteration but may be attacked (p.249). Footnote21 cautions that SIMD does **not** implement actual simultaneity between domestic and interstate wars (p.250); excluded engagement is their operational approximation.

##### A–E registered study matrix (table2, p.260; PDF20 visually checked)

| Study | Peripheral edges | Misperception | Self-misperception | Continue indecisive wars |
|---|---|---|---|---|
| A |yes |constant |yes |no |
| B |no |variable |no |yes |
| C |no |constant |yes |no |
| D |no |constant |yes |yes |
| E |no |variable |no |no |

All these are **concurrent** implementations. A is the closest comparator to the older serial model, not a serial arm. All primitive power seekers; civil wars disabled. Each run has256 worlds, horizon1000; parameters drawn from constrained normal distributions in table1, tuned to obtain both imperial and nonimperial outcomes (pp.259–260). Serial prior work instead used mostly high/medium/low levels, different initial mean power (two orders of magnitude), and fewer observations. Duffy expressly says direct coefficient magnitudes across serial/concurrent studies are not meaningful (p.260).

A yields9 empires/256, B250/256 (p.261); the author attributes much of that difference to misperception and topology, **not** serial updating. Closely matched variants initially show little serial-assumption effect (p.264). The strongest isolated divergence is rational-vs-primitive state endurance: serial rational advantage becomes concurrent rational disadvantage, whereas a greater system share of rational states still lengthens lives (pp.266–267). Cederman's predator/status-quo states are not this rational/primitive pair.

##### Honest transfer to SugarScape

Duffy supports retaining timing, boundary, collision, and persistence choices as explicit assumptions. He supplies **no quantitative target for EPM** and no warrant for assuming concurrency necessarily increases hegemony or persistence necessarily changes system endurance. EPM serial/snapshot switches should be designed as EPM controls with frozen parameters and initialization and an explicit collision/ownership policy. Label one-conflict-per-iteration selection as a different initiation protocol if included; do not disguise it as update order. Keep per-iteration and per-conflict clocks visible, since different event opportunity rates confound horizon comparisons.

#### Additional primary critics and scope

**Radax2010**, [TU Wien dissertation](https://repositum.tuwien.at/handle/20.500.12708/10274), ch.6.3 pp.162–166: directly reimplements published EPM; eight predator shares0,.05,.1,.2,.4,.6,.8,1, superiority/victory2 or3,20 experiments each,1000 steps. Literal TFT reproduces offense means partly but defense poorly. Grim-trigger substitution improves agreement without reproducing outcome distributions. It supports a persistence comparison, not verification of undocumented original code. Ch.6.4 separately audits **GeoSim** using author-provided code; its campaign/accounting bugs cannot be attributed automatically to EPM. This new direct replication invalidates “only Störmer.” Cached as `papers/geopolitics/radax-2010-tuwien-thesis-modeling-international-state-system.pdf`; no runnable EPM supplement found in targeted search.

**Köhler-Bußmeier & Störmer2019**, cached as `papers/geopolitics/koehler-bussmeier-stoermer-2019-meta-analysis-cederman-epm.pdf`, [working paper](https://edoc.sub.uni-hamburg.de/informatik/volltexte/2019/247/pdf/koehler_stoermer_meta_analysis.pdf), pp.8–12, recasts the same110 configurations×10 effort. They obtained a hard copy of almost all Cederman Pascal source and redesigned in Python. Page11 describes ≈75% in the stable/slightly-decreasing corner **across a range of struggle counts**. It neither supplies a reproducibility repository nor resolves appendix/text contradictions. This is a followup of Störmer, not independent replication evidence.

**Lazer2001**, [original JASSS review](https://jasss.soc.surrey.ac.uk/4/2/reviews/lazer.html), proposes conditioning on already-emerged small-n great-power worlds, physical/ethnic expansion barriers, and tests of how independent local production smooths large-state resources. These are clearly motivated robustness questions but have no fixed protocols, thresholds, or quantitative predicted effect. They belong to descriptive controls/new registered designs, not scored reproduction of a critic's experiment. The review's20×20 description also conflicts with the originator's experiment dimensions, so use it for criticism rather than parameter authority.

#### Chapter5 footnote5: the exact experiment to register

Verified directly from Cederman book **p.121 footnote5 (PDF136)**. PRA preserved the defense-orientation lesson in fig.5.6, but the footnote warns this does **not** imply alliances still produce universal empire and highly fragmented outcomes. The proposed mechanism is that prior hegemonic growth tolerated many active fronts; defensive alignments create those fronts, whereas PRA reallocates commitments. Prior notes locate PRA equations5.2–5.3 on pp.117–120; they contain active-front and passive-front asymmetry plus previous-period commitments, and are not merely “divide proportionally among all neighbors.” The footnote says preliminary alliance runs suggested nontrivial dependence, without publishing the design/results.

Register a **2×2 allocation×alliance interaction** on the originator predator-share×offense/defense design, holding persistence, damage convention, timing, boundaries, seed/init distribution, and judging fixed. Arms: equal allocation/no alliances; equal/alliances; literal PRA/no alliances; literal PRA/alliances. For each stratum report the alliance contrast in probabilities of originator outcome categories and the difference of those contrasts between allocation rules. This makes the doubt concrete without changing the judge or assuming reversal. The new test is an **author-motivated robustness question**, not a replication of a published fn5 experiment. No fabricated expected effect size or pass threshold. PRA ambiguities (passive commitments, first period, zero denominator, resource damage/bookkeeping, inheritance) must be declared before results.

#### Registered tests versus descriptive controls

- **Reproduction targets**: originator curve/category data under its published design; Radax's explicit20×8×2 TFT/GT design as an independently labeled replication comparison. Acceptance tolerance must come from recovered uncertainty/rounding/sample resolution or user-approved methodology, not improvised numbers.
- **Critic protocol recoveries**: Störmer text-design and appendix-design are separate named protocols. If preserving rules is out of scope, implement her prior as a sensitivity study on EPM and call it an adaptation. Report exact structural-event-zero rate and near-corner plots, never a required75% exact-inert pass.
- **New registered robustness**: fn5 interaction; EPM snapshot versus sequential order with documented conflict resolution; normal versus bounded-uniform resource variation if asking the Störmer deviation question. Freeze designs and outcome judge before observing outputs.
- **Descriptive only without further registration**: weak fights/states correlation, Duffy sign transfers to EPM, “hegemonic takeoff” curvature thresholds, small-n conditioning suggested by Lazer, war lengths/destroyed resources beyond originator's judge. None has a recovered universal numerical threshold.

Evidence renders inspected: Störmer p.39/PDF47 (attack inequality and uniform harvest), p.51/PDF59 (n110 and mean plot), appendix ii/PDF68 (actual sampled ranges), iii/PDF69 (1000 and spread assignment), xvi/PDF82 (Moore loops); Duffy p.260/PDF20 (table1 and A–E matrix), p.252/PDF12 (initiation equations6–7). Extracted texts retained in `/tmp/stoermer.txt`, `/tmp/duffy.txt`, `/tmp/radax.txt`, `/tmp/koehler-stoermer.txt`.
