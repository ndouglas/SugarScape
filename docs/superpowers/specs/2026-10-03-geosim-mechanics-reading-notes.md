# GeoSim mechanics reading notes

Date: 2026-10-03. Read-only research synthesis; no implementation or measurements.

Sources: Cederman (2003), *Modeling the Size of Wars*, APSR 97(1):135–150; Cederman (2002), *Modeling Self-Organized Criticality in War Size* working paper; Cederman (2002), *Endogenizing Geopolitical Boundaries*, PNAS 99 suppl.3:7296–7303. APSR/PNAS page numbers below are printed pages; working-paper citations are 1-based PDF pages. Local PDFs are under `papers/geopolitics/`. Inspected rendered APSR pp.146–148, WP PDF pp.10/14, and PNAS p.7298 in addition to extracted text. The WP PDF itself contains corrupted characters, missing figures, and overlapping table text.

## Recommendation

Add a separate `geosim` kind/world/config. Initialization, economy, strategy, combat, technology, period ordering and the observed unit all differ from `polarity`. An environment around that world would hide substantive model changes. Preserve existing polarity fixtures/fingerprints; reuse interfaces and small verified geometry utilities where appropriate. Do not build a generalized geopolitical framework before two independently faithful models exist.

Current `polarity` starts every cell sovereign with normal stocks and latent predator/status-quo types, harvests stochastic resources, chooses targets with EPM rules, supports equal/full PRA, alliances and domestic revolt, and tracks dyadic episodes. `PolarityWorld::period` decides and resolves damage before harvesting/claims. GeoSim requires state-owned technology, alert/campaign memory, capacity/damage buffers, hybrid commitments and multi-dyad war aggregation. Existing `territory.rs` embeds stock-transfer/capital-capture readings; reuse requires an assumption audit, not direct mechanical substitution. The October2 polarity design explicitly excludes later GeoSim technology/war-size fitting.

## APSR Table A1 defaults (p.147)

| Parameter | Base | Reported alternatives |
|---|---:|---:|
| nx, ny | 50,50 | 75,75 |
| initPolarity | 200 | 450 |
| initPeriod | 500 | |
| duration after initial period | 10,000 | |
| resChange | .01 | |
| propMobile | .5 | .9 |
| pDropCampaign | .2 | |
| pAttack | .01 | |
| pDeactivate | .1 | |
| supThresh | 3 | 2.5 |
| supSlope | 20 | |
| victThresh | 3 | 2.5 |
| victSlope | 20 | |
| propDamage | .1 | |
| distOffset | .1 | .2 |
| distThresh at time0 | 2 | |
| distSlope | 3 | 5 |
| pShock | .0001 | |
| shockSize final shift | 20 | 0,10 |
| warShadow | 20 | 10,40 |

Setup selects 200 randomly located founders, recursively grows composite states until the grid is filled, and leaves no nonfounder primitive actor sovereign (p.146). Frontier selection, growth collisions, initial stocks and initial front commitments are underspecified.

## Period and resource semantics

APSR p.146 orders a complete period as resource calculation, allocation, decisions, interaction, structural change. Decisions/allocation use double buffering and randomized order; the actor list is scrambled when structural changes occur (pp.146,148). Initialization lasts500 periods; observation/technology then run10,000 periods to10,500 (pp.140–141). Exactly whether counting starts on500 or501 needs an explicit reconstruction convention.

Each cell yields1, the capital undiscounted and provinces distance-discounted. The corporate capacity update is

`R_i(t) = .99 R_i(t-1) + .01 [1 + sum_provinces f(distance,t) - totalDamage_i]`.

This is relaxation toward resource capacity after damage, not EPM normal harvest or stock accumulation (pp.146–147). Previous-period damage naturally enters the next resource-update phase; record the exact buffer convention.

The intended distance function declines from near1 centrally toward .1 at long distance. **Printed p.146 uses exponent `-distSlope` with positive slope3, which instead increases.** This was visually verified. Fig.2 and pp.139–140 require decline; WP PDF p.14 uses a positive distance exponent. Declare a literal/prose reading, never silently correct the paper. Distance metric is not specified.

Technology frontier after initialization: `distThresh(t) = 2 + (t-500)*shockSize/10000`. Each state catches up to the contemporary frontier with probability .0001 each period independently of strategic context (p.147). Extraction and force projection both improve. This does not lower attack/victory thresholds. Clamping before500, inherited technology and collapse/reemergence technology remain unspecified.

Half of capacity is fixed and evenly spread over sovereign neighbors; half mobile. Fixed component is `.5 R_i / n_i`. Add mobile commitment as follows: full mobile capacity when no active enemy resources; active front `mobile_i * oldOpposingCommitment / activeEnemyTotal`; passive front `mobile_i * oldOpposingCommitment / (activeEnemyTotal + oldOpposingCommitment)`. Passive allocations are conditional plans, not an additive budget. The printed enemyRes summation appears to cover all relations whereas its no-attack branch and example imply active fronts only (pp.147–148). Name that reading.

## Decisions, combat and structural change

All actors use grim trigger: D if either party played D previously, otherwise C. New attacks select a random neighbor; campaigns retain a target and drop with probability .2. Campaign-drop sampling timing is not specified (p.148).

Normally contemplate attacks with probability .01; own or neighbor fighting automatically alerts a state. Alert states contemplate attacks each period; after neighborhood action disappears they deactivate with probability .1. Pseudocode can read `(no front action AND random activation) OR alerted OR campaign`; prose can suggest a global no-current-action guard. Freeze precedence explicitly (p.148).

Force ratio is `B = f_i(d_i)*r_ij / [f_j(d_j)*r_ji]`, discounting both capitals' commitments to the battle site. Attack probability is `1/[1+(B/3)^-20]`, .5 at3:1 (p.148).

APSR p.148 describes target-first then adjacent attacker selection. PNAS p.7298 describes attacker-border-cell-first then target. These two-stage samplers differ from each other and from uniform-border-edge selection; preserve a named path reading.

Attacker victory uses threshold3/slope20. Defender uses reciprocal threshold1/3; defender victory stops fighting **without annexing attacker territory**. Attacker victory generates a claim. Neither, one or both parties may claim victory, but shared/independent draws and simultaneous-victory handling are unspecified (p.148). Reciprocal defender threshold needs an explicit ratio orientation so defender success does not increase with attacker advantage.

Damage text p.148 says an attacking state incurs .1 times its own commitment; pp.139–140 describe damage incurred by all parties as10% allocated resources, while the resource formula sums damage(j,i). Own-commitment versus opponent-inflicted accounting and C/D incidence need explicit readings.

Claims execute in random order and lock involved units; contiguity is enforced (p.149). Primitive targets are absorbed; ordinary provinces are absorbed; disconnected provinces individually regain sovereignty; composite-capital targets cause collapse. APSR's capital bullet does not explicitly annex the center. PNAS p.7298 explicitly absorbs the center while freeing all other provinces: attributed support for capture-and-fragment. Lock scope and stale-claim handling remain reconstruction choices. APSR excludes deliberate secession and voluntary unification (pp.139,148–149).

## Cluster aggregation and censoring

APSR pp.140–141 defines an active state as currently fighting or having fought in the last20 periods. Spatial clusters comprise adjacent fighting states bound through conflictual interaction. Continuing fighting states retain cluster identity; conflicts may merge. Complete a war only after no active member remains. Severity accumulates battle damage over all relevant participants/fronts.

A polarity dyadic episode can feed this aggregation but is not itself the GeoSim war. Shadow persistence can connect separated engagements; merges combine multiple histories. The paper does not settle cluster splits, exact shadow-edge memory, sovereignty changes, merge bookkeeping, observation-boundary initialization, or horizon-open wars. Retain period-level conflict graphs, state identities, damage ledger and merge records. Report open clusters as censored; excluding them from completed-war fits is a labeled reconstruction, not a recovered author censoring rule.

Figure6 excludes events below **log10 severity2.5**, about316 damage units (p.141 footnote8). Tail cumulative frequency is fit against severity by log/log OLS. The cutoff is not duration/cell count. Preserve source regression as a reproduction diagnostic without equating high R² with proof of a power law.

## APSR Table1 exact targets (p.143)

Fifteen independent complete histories per arm, seeds affecting setup and later randomness. Illustrative run selected for median R²; its slope is the lowest base-run slope (pp.143–144).

| Arm | Slope min | Median | Max | R² min | Median | Max | Median range | Median wars |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Base | -.64 | -.55 | -.49 | .975 | .991 | .996 | 4.2 | 204 |
| Shock10 | -.71 | -.62 | -.56 | .968 | .980 | .993 | 3.7 | 267 |
| No shocks | -1.43 | -1.32 | -1.17 | .878 | .941 | .975 | 1.4 | 132 |
| No context activation | -1.52 | -1.34 | -1.20 | .835 | .882 | .934 | 1.6 | 696 |
| Shadow10 | -.69 | -.60 | -.53 | .966 | .990 | .996 | 4.2 | 325 |
| Shadow40 | -.60 | -.50 | -.45 | .970 | .989 | .997 | 4.2 | 148 |
| Thresholds2.5, shock10 | -.62 | -.53 | -.46 | .965 | .984 | .991 | 4.3 | 210 |
| Mobile.9 | -.65 | -.58 | -.53 | .954 | .987 | .992 | 4.3 | 250 |
| Distance offset.2 | -.72 | -.52 | -.43 | .908 | .990 | .995 | 4.4 | 211 |
| Distance slope5 | -.60 | -.53 | -.46 | .974 | .986 | .991 | 4.3 | 217 |
| 75×75/450 | -.67 | -.59 | -.54 | .987 | .993 | .996 | 4.6 | 502 |

**Row7 is resolved by p.149: both supThresh and victThresh are2.5, and shockSize is10.** It is not a single-factor threshold experiment. No-context activation's exact handling of baseline activation/campaign behavior remains unclear. Author acknowledges extensive calibration and failure under extremes; full SOC parameter insensitivity is not achieved (pp.144–145).

## Working-paper predecessor boundaries

WP PDF pp.7–10,14–16 differs substantially:

- 30×30 or45×45 primitive sovereign grid;5% receive10 initial resources, others0.
- Yield10/cell; distance offset .2, threshold3/slope10; capacity is `10 + discounted province yield`, without APSR relaxation.
- Mobile share .85; superiority threshold2.2/slope50; victory threshold2.2/slope10; stalemate probability .2 if undecided; no battle-damage economy.
- Technology lowers attack and victory thresholds by5%,10% or15%, adopted once with probability .001 per state/period after500.
- Shadow5; adjacent fighting states merge **whether or not they fight each other**.
- Severity proxy is fighting **dyad-periods**, not resource damage.
- Thirty continuations from an identical66-state configuration at500 for each shock, ending10,000; pooled slopes roughly -.75/-.53/-.41 (PDF p.10).
- Ten45×45 histories with131 states at500, ending15,000 (PDF p.10).

Its printed attack/victory exponents are positive despite prose requiring increasing probability with superiority. Its sample says20 wars but lists21 magnitudes (PDF p.9). These discrepancies are evidence limits, not invitations to tune. PNAS p.7298 supplies lineage clarification of capital annexation, two-stage paths, parallel claim processing, defensive alliances and optional secession; it is not an APSR2003 default parameter source.
