# Democratic peace — primary-paper mechanics reading

Date: 2026-10-03. Status: source-first research proposal; not an approved implementation specification, preregistration, or measured finding. No product code, builds, dependencies, or registered simulations were involved in this reading.

## Source and visual verification

Lars-Erik Cederman, “Modeling the Democratic Peace as a Kantian Selection Process,” *Journal of Conflict Resolution* 45(4), 2001, pp.470–502.

Local source: `papers/geopolitics/cederman-2001-jcr-democratic-peace-kantian-selection-process.pdf`. SHA-256: `069ad22da938727b2020c5d2c3a52aa383687c74085f5bc0af8cea74817b5215`.

The adjacent `.ocr.txt` was used for navigation, not as sole authority for equations or numerical defaults. The scan has 33 pages; one-based PDF page = printed page −469. Visually inspected rendered printed pp.486–491 and493–499, including Table A1, Figures9–11, and the attack, victory, extraction and alliance equations.

Ignored visual evidence is in the main checkout, not this research worktree:

`/Users/nathan/Projects/ndouglas/SugarScape/survey/out/geosim-democratic-peace-research/reader/page-17.png` through `page-22.png` (printed pp.486–491), and `page-24.png` through `page-30.png` (printed pp.493–499).

Rendering used `pdftoppm -scale-to 1800 -png` on the respective page ranges. Exact illustrative maps and seed7 outcomes were read from OCR/prose but were not independently digitized or docked to author code.

## Literal defaults

| Parameter | Source value | Printed page |
|---|---:|---|
| Grid |15×15 bounded cardinal lattice|493–494, footnote14|
| Initial actors |225 unitary primitive actors|493|
| Democratic share |Independent variable|494|
| Initial resourced share |`propHegemon=.05`; selected actors10, others0|493–494|
| Mobile share |.5; sensitivity .15 and .85|494|
| Attack threshold/exponent |2.2 /30|494,496|
| Victory threshold/exponent |2.2 /10|494,497|
| Stalemate probability |.2|494,497|
| Province extraction share |1|494,497|
| Distance discount |.95|494,497|
| Alliance threat threshold |2|494,498|
| Horizon |1,000 source periods|486|
| Replications |30 per initial democratic frequency|486|
| Illustrative seed |7 for Figures1–8|478 footnote5|

The stated mean11.25 initially resourced actors supports independent probabilistic assignment rather than an exact fixed count, but the source does not provide the sampling algorithm or RNG. Democratic assignment likewise describes proportions without specifying Bernoulli versus rounded fixed counts. Seed equality alone cannot establish trajectory agreement.

## Phase clock and resources

The base clock is allocation → decision → interaction → resource updating → structural change. The appendix explicitly places resource updating after combat and before conquest. Allocation and decisions use quasi-parallel double buffering, and structural execution randomizes sovereign actor order. Actors remember one previous step. (p.493.)

Threat perception and alliance formation occur immediately after relational allocation. Collective security follows alliance formation before modified decisions. (pp.498–499.) The precise observation buffer for attacked allies and pariahs remains unspecified; see ambiguities below.

For an actor with resources `res` and `n` eligible external fronts, fixed commitment is `(1-propMobile)*res/n`. Democracies mobilize only toward nondemocratic neighbors, so their `n` excludes democratic fronts. Mobile commitments depend on old opposing commitments and active enemy resources; when enemy total is zero, each contemplated front receives the whole mobile pool. These are conditional commitments, not simultaneously spent resource accounts: summing all contemplated fronts can exceed the mobile pool. (p.495.)

The extraction equation is visibly printed with exponentiation:

`res(i) = 10 + sum(provinces j: taxRate * 10 * grad^dist(i,j))`.

There is no capacity relaxation, damage subtraction, technology shock, or resource accumulation stock. All sites become worth10 for extraction, despite the initial10/0 competition trigger. The source does not identify the distance metric. Resources update before structural claims, so territory acquired in that structural phase contributes only in a later resource update. (p.497.)

## Decisions, combat and selection

Either actor's previous D triggers D on their front. Only an actor without action on any front may choose one random new victim and test attack probability. Democratic actors exclude democratic victims. An approved attack chooses a victim border province, then an attacker province adjacent to that target. (pp.495–496.) Battle memory resets when the battle ends; a further campaign requires a new unprovoked attack. (p.478 footnote6.)

Combat has consequences only for mutual D and produces no damage. Each side may claim victory; if neither claims, a .2 draw ends the battle, otherwise the grim trigger carries it forward. A unilateral attack can therefore become mutual combat later. Draw independence, opposing-claim resolution, and exact action-reset handling are not fully specified. (p.497.)

For conquest: a unitary victim is absorbed; capturing a compound capital collapses the state and releases its provinces; capturing an ordinary province transfers it and releases any provinces disconnected from their capital. The compound capital's own disposition is not explicit. (pp.497–498.)

Evolution operates through conquest and survival. States retain regimes except when invaded; learning and socialization are excluded. The paper does not precisely specify occupied primitive tags or the regimes of released provinces. (p.491.) Democratic dominance is consequently a selection result under hardwired dyadic avoidance, not learned democratic cooperation.

## Alliances and collective security

An actor identifies its strongest neighbor above local superiority2 as its primary threat. Democracies exclude democracies as threats. Two or more actors sharing a primary threat form a defensive alliance; mixed regimes are allowed. Each actor has one alliance membership and each alliance one threat. Membership below two cancels the alliance. (pp.482,498.)

An aggressor contemplating a member of an alliance against itself substitutes pooled relational resources of that alliance for the victim's individual defensive resources. These alliances also induce D on local fronts against a state attacking an ally. The source supplies pooled deterrence and additional-front obligations; it does not provide a separate fully specified pooled combat equation. (pp.498–499.)

A nondemocracy fighting a democracy becomes a pariah for the combat's duration, including when the democracy initiated it. Democracies then open local fronts against pariahs. Predators consider local alliance deterrence but ignore the deterrent potential of collective security. (pp.484,499.)

## Named ambiguities requiring source or design resolution

1. **Printed probability versus superiority prose.** Visually printed equations on pp.496,497,499 are `1/(1+(bal/t)^c)`, with positive exponents30/10 and `bal=res(i,j)/res(j,i)`. They decrease as own superiority rises, whereas prose requires increasing attack and victory probability. Name `printed_decreasing` and `prose_increasing`; do not silently import GeoSim's increasing helper.
2. **Collective-security scope.** The p.499 paragraph restricts assistance to members of the same alliance; its pseudocode and final paragraph, plus p.484's global security community, cover all democracies regardless of other alignments. Name `same_alliance` and `all_democracies`.
3. **Enemy-total denominator.** The p.495 sum does not explicitly filter active fronts, whereas the explanation does. Its inactive-front worked example repeats denominator `15+25+10` for opponents20 and30; the formal formula supplies each front's own opponent term. Keep `active_fronts` versus `all_fronts` and formula versus literal-example evidence distinct.
4. **Initial zero ratios.**10/0 setup makes zero/zero and positive/zero relations possible before the first resource update. No numerical policy is supplied; this cannot be dismissed as an irrelevant edge case.
5. **Assignment granularity and RNG.** Bernoulli versus fixed-count regimes/resources, rounding, random draw order, target sampling distribution and RNG/runtime are unreported.
6. **Distance metric.** Euclidean, Manhattan and territorial-path distance are not identified.
7. **Conquest and regime inheritance.** Compound-capital transfer, occupied primitive tag persistence, disconnected provinces' regimes, simultaneous claims, actor-versus-claim shuffling and locking are incomplete.
8. **Alliance evolution.** Reassessment, membership persistence, switching primary threats, ties and reentry are incomplete.
9. **Obligation timing.** Attacked-ally and pariah detection may inspect previous actions or buffered current decisions. Current decisions would require an explicit resolution rule for cascades.
10. **Clustering estimator.** The paper gives size weighting and survivor conditioning, but not unique-neighbor versus border-length exposure, isolated actors, or exact averaging order.

These are research questions; the notes do not approve defaults or register sensitivity arms.

## Source experiments and own statistics

The specified initial democratic frequencies are `0,.05,.1,.15,.2,.25,.3,.4,.5,.6,.7,1`, with30 histories each. The plotted mechanism treatments are cumulative: tagging; tagging+alliances; tagging+alliances+collective security. The paper also describes the same mechanisms at mobile shares.15 and.85. (pp.486,488–489.)

A full reconstruction might use `12 frequencies ×3 mechanisms ×3 mobile shares ×30 histories =3,240` attempts. This is a **tentative population assumption, not a registered experiment**: thirty histories are stated for the main sweep but not explicitly repeated for the alternative resource-share sweeps. Precision extensions, reconstruction controls and follow-up experiments are not part of that count.

| Source target | Definition and availability |
|---|---|
| Figure9, p.486 |Mean final democratic territory share at period1,000, mobile.5|
| Figure10, p.488 |Mean size-weighted exposure of democracies to democratic neighbors, divided by initial democratic frequency, mobile.5; only histories with a surviving democracy enter (definition p.487)|
| Figure11, p.489 |Mean final democratic territory share at period1,000, mobile.85|
| Mobile.15 |Textual comparison, no standalone plotted curve|
| Collective security without alliances |Qualitative footnote9 p.487: weaker than combined security, stronger than alliances alone|

Final democratic territory share can be defined exactly in our implementation as democratic-owned cells /225. For Figure10, a proposed unique-neighbor estimator would weight each surviving democratic state's fraction of democratic sovereign neighbors by its cell count, divide by surviving democratic cell count, then by initial share. This is an **inferred estimator**, not an extracted formula. Border-length exposure must remain a named competing interpretation. Extinction histories are excluded from the source estimator, not assigned zero; the ratio is undefined at initial share0. Report eligibility and extinction denominators separately.

Useful supplementary, explicitly our-own statistics include democratic extinction probability; all-democratic occupation probability and first-passage period; regime-specific surviving-state counts; final conflict-front count; alliance participation; and pariah prevalence. The paper's absorbing perpetual peace is all-democratic occupation, not merely an uneventful final period. (p.484.)

Figures9–11 require calibrated digitization with pixel uncertainty. They provide no raw source means, seed lists or uncertainty bands. Quantitative source comparisons therefore need extraction envelopes and a frozen reconstruction interpretation before measurement; compatibility would not establish exact source equivalence.

Decision targets supported by source prose are the tagging intermediate-density dip; alliances' large advantage near initial.2–.4; collective security's low-density advantage; clustering factors above5, around7 and above10; and mobile.85 undermining tagging while collective security remains strong. (pp.487–489.) Seed7 pictures are descriptive until the original runtime and RNG are recovered; identical seed labels are insufficient.

## Architecture recommendation

Recommend a separate `democratic_peace` model kind that reuses narrow territory/front helpers and host infrastructure. This recommendation is provisional research, not authorization to implement.

The existing `crates/sugarscape-core/src/geosim/world.rs::step` updates resources before allocation and then executes decision/combat/claims, WarTracker and technology shocks. GeoSim capacity is a relaxation recurrence with damage; actor state includes technology threshold, alert and campaign; structural release initializes technology-related state. The2001 paper instead requires after-combat instantaneous extraction, mutual-only costless battles,225 initially independent actors with10/0 resources, separate alliances/pariahs, and no technology, alert, campaign or war-shadow rules.

Bounded cardinal adjacency, generation-aware identity concepts, contiguity traversal, deterministic buffering patterns and native/WASM host conventions can be reused. A small neutral conditional-commitment helper may also be reusable after config decoupling. Do not carry the existing WarTracker, phase clock, damage or capacity recurrence into the2001 engine merely because both models share Cederman lineage.

`docs/papers.md` Queue#1 describes “regime types on the GeoSim engine.” Treat that as a conceptual lineage note rather than proven config-only compatibility. The2001 article predates the2003 war-size model. No historical validation, exact paper reproduction or author-code docking follows from reuse. Main-checkout source identity and existing GeoSim measured pre-fix identity must remain distinct in any later provenance record.
