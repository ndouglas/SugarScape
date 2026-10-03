# Cultures, construction and underworlds

**Date:** 2026-10-03

**Status:** living research programme; campaign outline preserved for refinement.

**Selected first direction:** collective burrow construction.

**Scope:** campaign families and research questions, not an approved implementation spec or run protocol.

## Purpose and motivation

Develop speculative cultures from inspectable biological, behavioral, social and environmental
components, then investigate how they inhabit, build, communicate and change their worlds across
generations. Individual components should have empirical support where possible. Their combinations
are hypotheses to investigate, not automatically validated explanations of human or animal societies.

The user's motivating examples include goblin villages of hovels and the placement of a chieftain's
or shaman's residence; kobold and orc settlements; rooms, homes, temples, fortresses, castles,
districts, neighborhoods, communities and cities. Cultures might differ in beliefs, behavioral
components, bodies, ecological circumstances and historical inheritance. Salient astronomical
phenomena, including a moonless sky, binary suns or a tidally locked world, could influence learned
calendars, practices, institutions and linguistic evolution through explicit intermediate mechanisms.

The selected first direction is **collective burrow construction**. The user's love of *Zork* and
*Dungeons & Dragons*, and the desire to create rich, interesting, diverse, complex, individual
Underworlds or Underdarks, are major reasons for starting Hornvale. Preserve that creative target
while developing small experiments whose causes and limitations we can explain.

The long-term underworld should acquire character through its inhabitants and history: excavation,
resource movement, dwelling, repair, access control, abandonment, reuse, cultural transmission and
conflicting purposes. Geography, biology, customs and past events can contribute independently.
This is an aspiration, not a claim about current SugarScape capabilities.

## Relationship to Minds and Hornvale

[Minds](2026-09-27-minds.md) develops individual decision mechanisms. The
[collective-agency programme](../superpowers/specs/2026-10-02-minds-collective-agency-program-design.md)
provides related campaigns: C2 affordances and controls, C3 communication, C4 learning, C5 social
knowledge, C6 organization, C7 costly assistance, C8 research, C9 oversight and C10 culture and
environmental inheritance. The present programme elaborates C10 and supplies environments and
questions to several other campaigns. Its A–G labels are local research-family labels, not new
Minds milestone numbers.

Minds' existing sequence continues. P3 protection implementation is merged, but scientific
execution still awaits its manifest/opportunity review; P4 deception and subsequent decision-engine
comparisons keep their existing place. Hornvale's repaired exchange request remains a separate
design branch. This programme does not displace those commitments or require their completion
before literature and environment design can begin.

The user reports that Hornvale has constraint/pattern-based floorplan code. Its interfaces and
assumptions have not been audited for this programme. Inspect it before assigning integration
responsibilities. A possible division is SugarScape for controlled mechanism experiments and
Hornvale for richer world generation, presentation and use, but that boundary remains open.

## Programme organization

The recommended arrangement is **parallel branches with staged integration**. Each branch gets a
small discriminating experiment; combinations follow understanding of the components. Separate
sessions can develop literature, requirements, generators and protocols. Agent experiments join
when their required capabilities have been designed and verified. Shared runtime changes still
need coordinated ownership and integration; separate sessions alone do not provide isolation.

Two alternatives remain available: defer all work until later Minds milestones, reducing competing
demands but delaying independent environmental research; or build an integrated fantasy settlement
immediately, obtaining an evocative demonstration at the cost of many simultaneous assumptions.
The latter could be a useful creative demonstration, but should be labeled accordingly.

The earlier recommended shared setting was a small settlement with shelter, storage, entrances
and a gathering place. The user's burrow choice moves the first setting underground while retaining
those functions as later additions. We can begin with excavation and transport before households,
institutions or a complete cultural model exist.

## Components and evidence

| Component | Examples | Proposed consequences to investigate |
|---|---|---|
| Embodiment | Size, climbing, carrying, sensory range, light sensitivity, temperature tolerance | Passage dimensions, entrances, nesting sites, accessible terrain |
| Subsistence | Resource distribution, storage, spoilage, seasonal scarcity | Stores, kitchens, dwelling locations, defended resources |
| Social organization | Household membership, inheritance, authority, ownership | Boundaries, gathering places, privileged access |
| Learning | Individual exploration, copying success, conformity, teaching, innovation | Traditions, specialization, regional differences |
| Beliefs and norms | Predictive claims, obligations, prohibitions, symbolic associations | Orientation, restricted spaces, ritual routes, preservation |
| Communication | Distinctions needed for coordination and transmission | Signals, names, conventions, specialist vocabulary |
| Historical inheritance | Existing passages, buildings, customs, remembered disasters | Persistent arrangements despite changed conditions |

These are modeling dimensions, not universal laws determining architecture. A named fantasy
population is a configured combination, not an architectural destiny. Hold bodies constant while
varying institutions, and hold institutions constant while varying environment or embodiment.

For each proposed mechanism record: the source and evidence type; the original setting and
measured behavior; the computational interpretation; departures and uncertainties; the simplest
competing explanation; and the observations that would discriminate between them. Distinguish
published model reproduction, empirical regularity, speculative composition and aesthetic goals.

Alexander's *Notes on the Synthesis of Form* supplies a useful vocabulary of interacting potential
misfits between form and use, and their decomposition into groups of requirements. Its 1971
paperback preface shifts emphasis toward reusable diagrams/patterns and away from obligatory use
of the elaborate mathematical method. Treat patterns as candidate spatial relationships and
functional hypotheses. Do not treat an entire pattern catalogue as established universal science.

## A. Bodies, habitats and usable space

**Question:** what environments can different bodies actually use, and how does that shape habitation?

Start with supplied shelters and vary one capability: body size, climbing, carrying or light
sensitivity. Measure accessible resources, travel costs, congestion, exposure and mistaken attempts.
Later studies introduce temperature, ventilation, water, slopes and materials. These provide
physical foundations for burrows, hovels, towers and underground settlements.

Compare the same social organization under different embodiment, and the same embodiment under
different environments. This joins C2 directly. Distinguish actual capability from the agent's
estimate of it. Coarse traversal constraints do not reproduce detailed biomechanics.

For the first burrow study, choose the minimum physical rules necessary to explain excavation,
movement and material transport. Verticality, support, gases, flooding and collapse are valuable
later candidates, not prerequisites to an initial controlled experiment.

## B. Construction and architectural patterns

**Question:** which structures follow from requirements, and which can arise from local building behavior?

Keep two sub-branches:

- **Requirement-based generation:** constraints and reusable patterns produce candidate layouts.
  Compare layouts with matched space, materials and functions. A first study can compare access
  arrangements around storage, sleeping and gathering spaces.
- **Agent construction:** agents choose sites, transport materials, build, repair and remodel at
  explicit cost. A first study can investigate whether local rules produce useful structures.

Shape grammars and residential-layout optimization support the first approach; experimental insect
construction supports the second. They make different explanatory claims. A generated settlement
does not show that its inhabitants designed it; an excavation simulation does not show the agents
held an architectural plan. Hornvale's generator is a candidate connection after interface review.

The selected first campaign belongs to the agent-construction sub-branch. Underground excavation
must be distinguished from material deposition: ant roof/pillar construction is a useful mechanism
lead, but not direct validation of a digging model.

## C. Households, authority and institutions

**Question:** how do membership, ownership, obligations and authority influence settlement organization?

Begin with a small settlement containing stores and households. Supply alternative arrangements:
household ownership, communal storage, or a distributor with privileged access. Measure
distribution, exclusion, assistance, conflict and access concentration.

Later compare authority based on resource distribution, defensive coordination, expertise or
inheritance. Initially these institutions are supplied treatments. A separate, harder campaign
investigates whether roles and authority emerge, persist and survive succession. This connects
to exchange and C5–C7.

The chieftain's location is an experimental question: proximity to a granary, a defended approach,
an ancestral compound, controlled thresholds or a shared hall might follow from different supplied
roles. A shaman/observer might need horizon access, privacy for instruction, a gathering place or
custody of inherited objects. These are speculative scenarios whose requirements can conflict.
Underground analogues include entrance control, shared stores and chambers used for instruction.

## D. Traditions, transmission and historical inheritance

**Question:** how do practices become traditions, and when do traditions help or hinder?

Use several viable construction practices. Compare individual experimentation, copying observed
success, conformity and teaching. Replace cohorts and change environmental conditions. Measure
retention, diversity, adaptation lag, innovation and dependence on particular teachers.

Independently preserve or remove structures so we can distinguish inherited knowledge from
inherited infrastructure. This is where settlements acquire histories: later agents encounter
arrangements their predecessors created. Cultural-evolution research supplies candidate
transmission mechanisms; their architectural consequences remain experimental questions.

In an underworld, inherited routes, spoil piles, markers and abandoned chambers can change later
construction even without explicit teaching. Separate that effect from copied habits or transmitted
plans. Define cohort replacement, migration and biological inheritance independently.

## E. Beliefs, norms, ritual and sacred places

**Question:** how do shared interpretations and obligations influence action and construction?

Split this into predictive beliefs, normative rules and symbolic associations. They require
different representations and tests. A tractable first experiment supplies a prohibition concerning
a location or resource, varies whether it predicts danger, and compares learning and transmission
under environmental change.

Later investigate recurring gatherings, restricted instruction, specialist authority, sacred
orientation and preservation of costly structures. Calling a building a temple describes its use
and institutional role. Demonstrating the emergence of religion requires a substantially more
demanding account.

An underworld could contain restricted passages, ancestral chambers or recurring gatherings,
initially as supplied scenarios. Test their coordination, exclusion and preservation effects
separately from any claim that agents invented sacred meanings.

## F. Skies, calendars and ecological knowledge

**Question:** how do observable cycles become useful knowledge, schedules and cultural practices?

Start with a simple visible cue that imperfectly predicts resource availability. Compare learners,
observers and agents receiving transmitted knowledge. Measure forecasting accuracy, harvest timing,
coordination and specialist dependence. Then remove the lunar cue, introduce overlapping cycles,
change visibility or provide persistent illumination.

The astronomical environment supplies observations and physical consequences. Cultural
interpretations are separate mechanisms. Use the causal sequence: observable phenomena, learned
regularities, interpretations, shared practices, institutions and construction. Two suns do not
automatically imply dual gods. Moonless conditions do not remove every possible calendar cue.
Physical consequences depend on orbital, atmospheric and ecological assumptions.

Underground inhabitants may obtain sky information through entrances, shafts, visits or specialist
reports. Lack of direct visibility provides a later test of information dependence. Environmental
model preparation can proceed independently; learned calendars connect to C3, C4, D and E.

## G. Signals, naming and linguistic evolution

**Question:** how do communication needs and transmission shape shared conventions?

Begin with a small signaling task: distinguish resource locations or coordinate access. Compare
supplied signals, negotiated conventions and conventions passed to newcomers. Then add construction
instructions, temporal distinctions, specialist vocabulary, migration and network separation.
Measure comprehension, expressivity, learning cost, divergence and recovery after contact.

Experimental work on transmission and communication pressures provides foundations. Start with
finite meanings and signals, without per-agent LLMs. Underworld tasks could later require route
names, hazard warnings, excavation requests or material distinctions. Their meanings should first
be supplied and tested; learned conventions are a separate treatment.

## First branch refinement: collective burrow construction

The following is a proposed sequence for refinement, not a frozen implementation plan. The user
selected the branch, but the first mechanism, species/model target and judged claims remain open.
Each selected study needs its own literature review, design and protocol.

### B1. Establish a bounded excavation world

**Question:** what minimum world can represent paid excavation, traversal and spoil transport honestly?

A candidate first world is a finite horizontal substrate with an entrance, solid and excavated
cells, and one explicit excavation/transport model. Decide whether spoil is transported as an
agent load, moved locally or abstracted into a paid disposal action. State the consequences of
that choice. Specify action order, occupancy, collision, costs, sensing, and volume accounting.
Begin without collapse or ventilation unless the selected empirical target requires them.

Verification should include hand-checkable excavation, material accounting, blocked movement,
transport, simultaneous-action arbitration and seeded replay. Select small exact-answer cases
before scaling. These establish world correctness, not biological realism.

### B2. Compare local construction mechanisms

**Question:** can local interactions generate organized connected structures, and what information is needed?

Candidate comparisons include cost-matched undirected digging, local geometry-sensitive rules,
traces or recruitment, and an explicit-plan benchmark where appropriate. A candidate testable
claim is that responding to existing geometry or local work sites changes connected usable space
per unit cost relative to a controller lacking that response. Choose the actual rule and empirical
target after reviewing primary construction studies; do not assume pheromones are universal.

Measure excavated volume, connected traversable volume, passage widths, chamber structure,
branching, dead ends, loops, travel, congestion and work allocation. Define how chambers and
passages are recognized before judged runs. Investigate how much apparent structure comes from
lattice geometry, boundary conditions, update order or site-selection bias.

### B3. Add inhabitation and competing functions

**Question:** when does a constructed burrow serve its inhabitants, and who benefits?

Introduce one function at a time: reaching resources, storing food, sheltering, or moving loads.
Hold resources and budgets comparable across alternatives. A chamber's name should follow its
supplied or observed use; an empty cavity alone is not a pantry, home or temple.

Add shared labor and conflicting requirements only when individual construction and use are
understood. Compare increased workforce with a lone builder given matched total action resources
where feasible. Report coordination cost, interference, distribution of benefits and unfinished work.

### B4. Preserve, repair and inherit the burrow

**Question:** how do existing structures and transmitted practices affect later builders?

Cross structure persistence with practice transmission. Compare reset environments, preserved
burrows without teaching, transmitted practices without inherited structures, and both together.
Introduce an environmental change so persistence and adaptation can be distinguished. Later add
damage, maintenance, abandonment, migration and reuse.

Measure reuse, repair, adaptation, accumulated benefit or degradation and dependence on founder
choices. This joins D and C10 and begins to give individual underworlds histories.

### B5. Diversify and integrate underworlds

**Question:** which combinations produce distinct, functional and historically contingent underworlds?

Vary one axis at first: bodies, substrate, resource geography, household organization or a learned
building practice. Then study interactions. Evaluate repeated seeds and unfamiliar conditions;
separate differences between runs from systematic differences between treatments.

Vertical levels, shafts, support, water, gases, collapse, doors, locks, ramps, traps, fortification,
settlement networks and sacred spaces become individually designed extensions. Add complexity
when a question requires it. A richly presented Hornvale world can eventually combine mechanisms,
but visual complexity alone is not evidence of sophisticated construction or culture.

## Integration horizon

The broader synthesis can proceed through household, village, connected settlements, districts
and cities. Underground equivalents include a dwelling chamber, occupied burrow, linked colonies,
specialized districts and extensive inhabited passage networks. Scale introduces new questions:
transport, infrastructure, specialization, governance, defense and maintenance.

Persistent access controls, private communication, shared stores and specialist knowledge also
connect to the safety programme. Later experiments can ask how architecture affects visibility,
permission, coordination and circumvention. Those links do not make oversight a prerequisite for
the first excavation studies.

## Research anchors and reading queue

The following sources were identified during the conversation. Their stated contribution is
narrower than validating this entire programme. Except for the Alexander section reads noted
below, full methods, parameters, data and code still require campaign-specific review.

- **Alexander:** *Notes on the Synthesis of Form* (1964), *A Pattern Language* (1977, with
  Ishikawa, Silverstein and collaborators), and *The Timeless Way of Building* (1979).
  Local copies are indexed in [the paper inventory](../papers.md). The paperback preface and
  selected requirements/decomposition sections of *Notes* were inspected; the other books
  have been collected, not reviewed in full.
- **Comparative domestic space:** Susan Kent, ed., *Domestic Architecture and the Use of Space*;
  [publisher excerpt](https://assets.cambridge.org/97805214/45771/excerpt/9780521445771_excerpt.pdf).
  Low and Chambers, eds., [*Housing, Culture, and Design*](https://www.jstor.org/stable/j.ctv5135zb).
  These are comparative sources for situated requirements, not universal house grammars.
- **Access and social organization:** Hillier and Hanson,
  [*The Social Logic of Space*](https://www.cambridge.org/core/books/abs/social-logic-of-space/buildings-and-their-genotypes/16840510DF4E41F19738CD6FE9E57493);
  Hillier, Hanson and Graham, [Normandy farmhouse analysis](https://journals.sagepub.com/doi/10.1068/b140363).
  Matthew Johnson, [*Behind the Castle Gate*](https://www.routledge.com/Behind-the-Castle-Gate-From-the-Middle-Ages-to-the-Renaissance/Johnson/p/book/9780415258876).
- **Generation methods:** Stiny and Mitchell, [The Palladian Grammar](https://journals.sagepub.com/doi/10.1068/b050005);
  Merrell, Schkufza and Koltun,
  [Computer-Generated Residential Building Layouts](https://vladlen.info/publications/computer-generated-residential-building-layouts/).
  Recognizable generated forms do not establish designers' cognitive processes or social fitness.
- **Construction behavior:** Khuong et al. (2016),
  [Stigmergic construction and topochemical information shape ant nest architecture](https://pubmed.ncbi.nlm.nih.gov/26787857/).
  Weber et al. (2013), [genetic modules in Peromyscus burrow evolution](https://pubmed.ncbi.nlm.nih.gov/23325221/).
  Deeming (2023), [mammalian nest construction review](https://pubmed.ncbi.nlm.nih.gov/37427481/).
  Excavation, deposition, individual burrowing and collective building need separate evidence.
- **Cultural transmission:** [Creanza et al. research review](https://doi.org/10.1073/pnas.1620732114);
  [Henrich and Broesch's Fijian learning-network study](https://pubmed.ncbi.nlm.nih.gov/21357236/).
- **Cultural astronomy:** [Hamacher's Kaurna seasonal-stars study](https://research.monash.edu/en/publications/identifying-seasonal-stars-in-kaurna-astronomical-traditions-2/);
  [Ruggles on interpretation and methodology](https://www.cambridge.org/core/journals/proceedings-of-the-international-astronomical-union/article/pushing-back-the-frontiers-or-still-running-around-the-same-circles-interpretative-archaeoastronomy-thirty-years-on/75CF1CC7D6956A22A5F269FA2A357EFC).
- **Alternate physical worlds:** [circumbinary climate modeling](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2020JE006576);
  [tidally locked atmospheric simulations](https://arxiv.org/abs/1001.5117).
  These support physical modeling, not predictions of particular cultural interpretations.
- **Language:** [Kirby et al. transmission experiments](https://doi.org/10.1073/pnas.0707835105);
  [compression and communication](https://www.sciencedirect.com/science/article/pii/S0010027715000815).
- **Settlement scale and environmental inheritance:**
  [Ortman et al. archaeological settlement scaling](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0087902);
  [niche construction overview](https://pmc.ncbi.nlm.nih.gov/articles/PMC4922671/).

Priority gaps for burrow refinement are primary collective-excavation studies, substrate and spoil
constraints, quantitative nest geometry, observational data or images suitable for comparison,
and any reusable published construction model. Also inspect the existing archaeology, culture,
belief, language and coordination collections before acquiring duplicate sources.

## Standing requirements and next refinement

Preserve Minds' named treatments, defaults where applicable, seeded replay, exact-answer checks
and native/WASM agreement where a runtime extension uses that seam. Register claims, extraction
methods, comparisons, budgets and stopping criteria before judged runs. Report failures,
inconclusive outcomes and computational costs alongside useful structures.

For every run record what was supplied, discovered, learned, constructed, negotiated and inherited.
Separate agent observations and beliefs from researcher ground truth. Ablate mechanisms, compare
simple controllers, test changed conditions, and distinguish aesthetic diversity from functional
benefit. Do not credit supplied roles, room purposes, protocols or beliefs as emergent.

The next refinement should select one collective-excavation question and primary empirical/model
anchor, inspect relevant SugarScape and Hornvale code, and present a bounded first-world design.
Resolve dimensionality, material accounting, sensing, work allocation and the comparison target
there. This document deliberately preserves those choices as open research decisions rather than
inventing implementation commitments during archival work.

## Decision record

- **2026-10-03:** preserve all seven campaign families as a living research programme.
- **2026-10-03:** select collective burrow construction as the first branch to refine; retain
  rich, diverse, individual underworlds as the long-term creative target.
- **2026-10-03:** collect local Alexander books under `papers/architecture/`; keep them
  gitignored under the repository's existing personal-source-library convention.
- Future refinements should record changed priorities and their reasons here. Keep empirical
  results and approved implementation specs separate from this programme outline.
