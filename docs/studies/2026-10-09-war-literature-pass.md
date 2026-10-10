# War inside a society: first focused literature pass

**Date:** 2026-10-09

**Status:** first research pass complete; proposed experiment scope only. No new simulator,
protocol registration, calibration, or war measurements are claimed.

**Question:** what lets comparable forces engage reciprocally and sustain fighting?
**Parent:** [War inside a society](2026-09-26-war-and-society.md).

## Scope and evidence

This pass covers engagement geometry, prolonged fighting, termination, and nearby modeling
work. It does not validate the complete nine-rung program. Historical spike observations
remain earlier measurements; the 14/20 conquest outcome is not a new result of this pass.

Keep four categories separate: mathematical benchmarks; theoretical mechanisms; empirical
observations with their population and coding; and findings generated inside a model.
Qualitative source accounts can motivate later ingredients but do not supply universal
coefficients. Calibrated cell distances, tick durations, combat hazards and empirical
near-equality duration estimates are unavailable in this pass.

The [current rule C implementation](../../crates/sugarscape-core/src/rules/combat.rs)
excludes targets whose holdings are at least the attacker's holdings.
Its retaliation filter uses the attacker's sight range, rather than each enemy's own sight.
Thus equal individuals cannot attack each other under C. Equal group resource totals may
still contain unequal individual pairs. Equality must distinguish population, resources,
efficiency and locally deployable fighting capacity. The source tests
`never_attacks_an_equal_or_wealthier_agent` and `avoids_sites_vulnerable_to_retaliation`
make these rules inspectable; this note reports no new runtime test result.

## Benchmarks and candidate regularities

| Behavior | Numbers and conditions | Source and evidence quality | Proposed ingredient | Measure |
|---|---|---|---|---|
| Aimed-fire attrition can make concentration disproportionately effective | Under homogeneous, continuously engaged aimed fire: dB/dt = −rR; dR/dt = −bB; invariant bB² − rR² | [Lanchester, Chapter V](https://en.wikisource.org/wiki/Aircraft_in_Warfare_(1916)/Chapter_5), §§20–22, 26–30: analytical benchmark, not a universal historical fit | W1 reciprocal damage under explicit aimed-fire assumptions | Attrition curves, invariant error, force-ratio response |
| Symmetric continuum fighting can persist while both forces decline | B = R and b = r = k gives N(t) = N₀ exp(−kt); exact extinction is asymptotic | Same Lanchester benchmark; derived solution, not an observed war-duration regularity | W1 symmetry and finite-population termination controls | Active duration, survival, depletion and censored endings |
| Geometry and unit definitions change the attrition law | No empirical coefficient or calibrated time unit supplied here; duel contact alone does not establish a linear law without specifying engagement capacity | [MacKay (2006)](https://arxiv.org/html/math/0606300), §§1, 3.1–3.4: mathematical review and benchmark cautions | W1 explicit contact capacity, force unit and timestep | Contacts per unit time, convergence under timestep refinement |
| Costly combat and negotiation can convey information and support settlement | Conditional theoretical result; no empirical duration calibration | [Slantchev (2003)](https://pages.ucsd.edu/~bslantchev/courses/pdf/slantchev-apsr2003v97n4.pdf), pp. 621–622: full primary theoretical paper | Separate later bargaining study | Belief convergence, offers, negotiated versus military endings |
| Civil-war duration depends on coding and can be prolonged without incomplete information | 128 civil-war cases, 1945–1999; start/end definitions and interruptions matter | [Fearon (2004)](https://fsi9-prod.s3.us-west-1.amazonaws.com/s3fs-public/275.pdf), pp. 278–279, Table II, p. 291: empirical duration analysis plus commitment model | Later campaign/commitment work; not a proof about equal groups | Calendar/active durations, interruptions, ending criteria |
| Information can motivate war without being a complete explanation of its persistence | No numerical calibration used | [Fearon (1995)](https://www.jstor.org/stable/2706903): official abstract/metadata only; private information with incentives to misrepresent and commitment problems | Separate information/bargaining study | Onset and continuation assessed separately |

Selected later-rung evidence clarifies what may be measured, without moving those rungs
into W1 or supplying calibration:

| Behavior | Numbers and conditions | Source and evidence quality | Proposed ingredient | Measure |
|---|---|---|---|---|
| Small-group attachment can contribute to fighting motivation | Selected interviews: over 40 US interviews and over 30 Iraqi POW interviews, 2003 Iraq; no cohesion coefficient | [Wong et al. (2003)](https://www.govinfo.gov/content/pkg/GOVPUB-D101-PURL-LPS35591/pdf/GOVPUB-D101-PURL-LPS35591.pdf), pp. 5–12: qualitative evidence; [published critique](https://journals.sagepub.com/doi/10.1177/0095327X05279181) limits causal/general claims | W2 cohesion candidate | Attachment, continued engagement and breaking tracked separately |
| Pursuit depends on rest, reorganization, ammunition, terrain and night | No universal pursuit casualty proportion or duration supplied | [Clausewitz, IV.12](https://www.clausewitzstudies.org/readings/OnWar1873/BK4ch12.html): military theory, not a quantitative dataset | W2 pursuit; W3 supply constraints | Pursuit duration, contacts, losses by phase and stopping reason |
| Restricted sleep can accumulate cognitive deficits | 48 healthy adults aged 21–38 across two experiments; chronic arm: 4/6/8 hours in bed for 14 days; separate total-deprivation arm: 0 hours for 3 days | [Van Dongen et al. (2003)](https://www.med.upenn.edu/uep/assets/user-content/documents/VanDongen2003CumulativeCost.pdf), methods/results: controlled laboratory evidence | Later fatigue sensitivity | Rest debt and task performance; no combat or fatigue-death coefficient inferred |
| Acute stress can impair delayed identification of people | 509 survival-school participants; delayed identification, not firing performance | [Morgan et al. (2004)](https://www.burtthompson.net/uploads/9/6/8/4/9684389/morgan_et_al_2004_stress_and_memory.pdf): primary study in military training | Later stress/perception sensitivity | Identification errors; no automatic mapping to sight range or accuracy |
| Supply failure can constrain continuation and retreat | Imphal/Kohima, 1944: 31st/15th divisions relied on local extraction after no food deliveries; 33rd delivery deteriorated despite rear stocks; hunger and malaria accompanied retreat | [Romanus and Sunderland, official history](https://www.ibiblio.org/hyperwar/USA/USA-CBI-Command/USA-CBI-Command-5.html), pp. 194–195: historical synthesis, not general coefficient | W3 supply reach/distribution | Delivered stores, consumption, starvation/disease losses and retreat |
| Cessation and durable peace are distinct outcomes | 48 dyadic ceasefires after wars, 1946–1997; observed through 1 January 1998 | [Fortna (2004)](https://www.columbia.edu/~vpf4/WP%20interstate%20pk%20offprint.pdf), pp. 492–495: peacekeeping associated with longer peace; observational/endogeneity limits | Later intervention/termination work | Cessation, renewed violence and censored peace spells |
| Indirect mortality shares depend on conflict and method | Iraq, 2003–2011: 2,000 households in 100 clusters; over 60% of estimated excess deaths attributed to direct violence | [Hagopian et al. (2013)](https://journals.plos.org/plosmedicine/article?id=10.1371/journal.pmed.1001533): primary household survey; recall, migration and baseline limits | Later civilian mortality accounting | Direct violence versus indirect deaths; excess mortality requires a counterfactual baseline |

The Imphal history reports that an order issued about 5 July was not fully grasped by
subordinates and attacks continued until 15 July; it does not isolate messenger travel as
the cause. The Iraq survey's direct violence category is not exclusively combat.
A model death ledger should preserve actor, combatant status, phase and cause. Counted model
deaths and counterfactual excess mortality are different estimands.

Still unverified: Swank's attributed 98%/60-day figure, a universal Shils causal claim,
pursuit-loss proportions, heart-rate arousal zones, forward-panic rates, Engels supply
distances, fodder commodity ranking, Fujiwara's attributed 60%, tooth-to-tail slopes and a
mission-command delay crossover. These remain candidates, not established targets.

A noisy perception rule is not a reproduction of Fearon's bargaining theory: its strategic
incentives and commitment structure require their own specification. Likewise, increasing
hit-point subdivisions can lengthen a simulation by construction. Injury granularity is a
sensitivity check, with fixed hazard per unit time, rather than an explanation of long wars.

## Prior-work matrix

| Source and inspected locator | Mechanisms or results | Evidence and limits; novelty boundary |
|---|---|---|
| [Brown (2000), ISAAC human dimension thesis](https://upload.wikimedia.org/wikipedia/commons/1/1f/Agent_based_simulation_as_an_exploratory_tool_in_the_study_of_the_human_dimension_of_combat_%28IA_agentbasedsimula00brow%29.pdf), printed pp. 13–17; full PDF text inspected | Personality-weighted movement; local force constraints; alive/injured/dead states; injury-sensitive mobility and optional personality changes; information sharing. Studies command, friction, mission duration and losses | Primary simulation application, not Ilachinski's original specification. Front matter disclaims validation of the experimental programs. Emergent formations, local numerical assessment, cohesion proxies and communications are prior art |
| [Fredlake and Wang (2008), EINSTein Goes to War](https://ntrl.ntis.gov/NTRL/dashboard/searchResults/titleDetail/ADA488178.xhtml), official NTIS abstract/metadata opened | Identifies Enhanced ISAAC Neural Simulation Toolkit; reviews CNA small-unit studies and Lanchester limitations; provides scenario tutorials | **Abstract-only coverage:** CNA/DTIC full report retrieval failed. Detailed rules, logistics coverage and validation remain unverified; do not infer absences. Bottom-up combat modeling is established |
| [Lauren and Stephen (2002), MANA](https://library.edvirtus.com/journals/journal-of-battlefield-technology/volume-05/issue-01/5-1-4-lauren), Structure, Limitations, Applications | Movement weights, local superiority/accompaniment constraints, sensing/weapons capabilities; contact, firing and injury can trigger personality changes; memory maps, communications and stochastic movement | Opened developer model account, not empirical validation. Explicit formation limitations. Adaptive behavior, information memory and parameter-landscape exploration are established |
| [McIntosh et al., Recent Developments in MANA](https://nps.edu/documents/106696734/108129278/IDFW13-Scythe-Mana.pdf/a029ba01-dd59-49c4-bebd-787673e1ba23?version=1.0), PDF pp. 1–2 | Range-dependent sensing/weapon probabilities; squad situational-awareness maps, sensor-to-map delays, between-squad links and reactions to received information; terrain and data farming | Opened developer capabilities report, with version-specific mechanisms. No empirical communication-effect or duration estimate established. Information delay and sharing are prior art |
| [Matsumura et al. (2004), Examining the Army's Future Warrior](https://www.rand.org/content/dam/rand/pubs/monographs/2004/RAND_MG140.pdf), printed pp. 7–14, 49–53 | Janus-centered force-on-force, terrain, weapons and sensor models; gamers initialize movement/fire coordination before repeated autonomous runs; component comparisons and defensive lines of sight | Opened primary scenario study. Model results, not universal defender ratios. Authors identify Janus limitations for civilians/interiors and request field experiments; JCATS/OTB supply some adjacent capabilities |
| [Epstein, Steinbruner and Parker (2001), Modeling Civil Violence](https://www.brookings.edu/wp-content/uploads/2016/06/cviolence.pdf), printed pp. 3–8, 27–40 | Grievance, local arrest risk and policing; Model II adds indiscriminate intergroup killing, births, inherited identity/grievance, aging/death and peacekeeping; spatial enclaves and intervention-sensitive survival | Opened full working paper, distinct from the 2002 published version. Idealized generative results; hardship/legitimacy are not measured. Intergroup violence with demographic replacement already exists |
| [Reichert, Garces and Lustick (2015), Modeling Endogeneity in Civil War](https://bpb-us-w2.wpmucdn.com/web.sas.upenn.edu/dist/7/497/files/2015/03/Garces.Lustick.Reichert_ISA2015.pdf), printed pp. 17–20 and conclusion; full PDF text inspected | Identity repertoires, switching/reclassification, local hierarchy assessment, entry/exit from competing orders; authority, identity and violence coevolve | Conference proof of concept and counterfactual methodological exercise, not empirical causal validation. Faction/authority emergence and collapse are established; this is not itself a bandit-to-tax-state model |
| [Dorff, Gallop and Minhas (2022 online), Network Competition and Civilian Targeting](https://www.cambridge.org/core/journals/british-journal-of-political-science/article/network-competition-and-civilian-targeting-during-civil-conflict/3A221DDDCE8C6B16589AD613F6DF765B), Model Environment, Sequential Order, Empirical Analysis | Civilian support supplies spatially weighted resources; probabilistic battles change control; imperfect information shapes victimization; civilians choose support/flight; population growth; competitive networks predict victimization in simulations | Opened full primary article, with separate observational ACLED support. Association does not identify the micro-mechanism. Preset turn-limit stalemate is not endogenous termination. Joint battles/resources/civilian harm/displacement/demography already exists |

These sources do not establish that anyone has or has not implemented the entire proposed
chain. A first/end-to-end novelty claim is unestablished. The defensible contribution to
investigate is the specific controlled comparison and literal resource/casualty accounting
on a reproduced Sugarscape base. A later logistics audit and full EINSTein retrieval remain
necessary before making narrower absence claims.

## Recommended first study: W1

Three approaches were considered:

1. **Implement all rungs together:** defer; too many interacting assumptions would obscure
   which change enables engagement or prolongs it.
2. **Begin with perception error:** useful for onset, but does not independently explain
   persistence after reciprocal fighting begins; retain as a separate question.
3. **Begin with reciprocal finite engagement and controlled geometry:** recommended. Separate
   permission to attack from damage adjudication, retain exact C as the baseline, and compare
   contact-limited duels with aimed fire under explicit engagement-capacity assumptions.

W1 initially benchmarks engagements; it does not explain a whole war. Any later war model
remains a new model kind built on the Sugarscape world, with all switches off reducing
exactly to C as a design requirement, preserving the
[golden baseline](../../crates/sugarscape-core/tests/golden.rs). W1 is a proposed scope, not an approved simulator
implementation or measurement protocol.
Avoid a deterministic one-hit winner rule that has no defined symmetric outcome. Include
exact and near equality, side exchange, update-order controls, finite-population effects,
and timestep refinement with fixed hazard per unit time. A synchronous settlement control
should preserve actions eligible at the start of a step rather than suppress them because
the other side resolved first. Define elimination thresholds and administrative horizons
before measuring outcomes. Distinguish an engagement from a campaign; sustained contact
and repeated attacks are different observations.

Record attacks and contact opportunities; engagement and campaign active/calendar durations;
unresolved and censored endings; losses by phase/cause; resource transfers and destruction;
retreat; and termination reason. Report casualties alongside outcomes from the first study.
Do not claim conflict taxonomies until their definitions and denominators are specified.

W2 can then examine morale, rout and pursuit; W3 can examine finite supply reach. Information
and bargaining require a separate specification. The remaining physiology, civilian,
disease, social and state-building rungs retain their original research status. Indirect
mortality shares and pursuit-loss ratios need conflict-specific evidence; no universal
majority or fixed ratio is endorsed here.

## Provenance and remaining work

This is a research-only pass. Primary full texts were opened or downloaded and read for
ISAAC's application, MANA, Janus's application, civil violence and the Kalyvas-related models.
The benchmark and bargaining checks were verified in the parallel literature pass at the
locators above. EINSTein and Fearon (1995) retain explicit abstract-only status. No source
here supplies calibrated combat coefficients or a historical equality-to-duration estimate.
The measured polarity and GeoSim handoffs in the parent program are unchanged.
