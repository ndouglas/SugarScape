# Cederman, *Emergent Actors in World Politics* (1997), and its lineage: reading notes

Research notes for the geopolitics lineage campaign (`2026-10-01-geopolitics-lineage-plan.md`).
Source: `papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (OCR text layer;
PDF page = book page + 15; equations and figures checked against rendered page images). Sections 1–4
cover the book chapter by chapter; section 5 surveys the predecessors, follow-ups, replications and
data. Each section was written by a separate reader on 2026-10-01; figure readings are rough
(110–220 dpi, by eye) until digitized by marker.


---------------------------------------------------------------------------------------------------

## 1. Cederman (1997), *Emergent Actors in World Politics*: Ch. 3 "An Introduction to the Models" + Ch. 4 "Emergent Polarity"

Source PDF: `papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf`.
**Page offset: PDF page = book page + 15** (book p. 69 = PDF 84; p. 108 = PDF 123).
All page refs below are BOOK pages. Figures were checked against rendered page images, not only the OCR text
(the OCR mangles Fig 4.7, Fig 4.8, and the trust equation in fn 23).

---

### 0. Ch. 3 section "An Introduction to the Models" (pp. 69-71) + Table 3.1

- Fig 3.4 (p. 69) is a 3x3 grid: states (absent/implicit, reified, emergent) by ethnic/national communities
  (absent, reified, emergent). The Emergent Polarity Model (EPM, ch. 4) and the Extended EPM (ch. 5) go in cell 3
  (emergent states, absent nations). The Mobilization Model (ch. 7) and Coordination Model (ch. 8) go in cell 8.
  Conventional IR models go in cell 2.
- **Table 3.1 "A Guide to the Four Modeling Chapters"** (p. 70). Key: E = emergent, R = reified,
  -- = absent/implicit, X = explicitly modeled, () = possible extension.

| Chap | Model | States (ext.) | States (int.) | Nations | Time | Space | Model type | Illustration |
|---|---|---|---|---|---|---|---|---|
| -- | Conventional IR models | R | -- | -- | -- | -- | rational choice | -- |
| 4 | Emergent Polarity Model | E | R | -- | X | X | CAS | Italy |
| 5 | Extended EPM | E | E | -- | X | X | CAS | -- |
| 7 | Mobilization Model | R | R | E | X | -- | modified Markov chain | Habsburg, USSR, EU |
| 8 | Coordination Model | R(E) | E | E | X | (x) | CAS | Yugoslavia |

- p. 70: in the EPM "states are viewed as a corporate actors consisting of microlevel agents and emerge
  spontaneously as a consequence of power competition. This setup explicitly captures the spatiotemporal
  environment as a network of agents locally interacting." The ch. 4 EPM models external sovereignty only. Internal
  sovereignty is reified (no civil war, absolute rule). Ch. 5 adds the "two-level version that allows for internal
  collective action".
- p. 71: the Coordination Model (ch. 8) is "technically implemented drawing on the results of the extended
  Emergent Polarity Model".

---

### 1. Model specification as stated

#### 1.1 Grid, topology, initial state
- **Square grid "of user-defined proportions"** (p. 83). **"All the findings presented below were generated in a
  ten-by-ten system"** (p. 83), so the experiments have 100 initial states. The illustrations (Figs 4.3-4.5) use a
  **20 x 20** system (400 actors) (p. 79).
- **Neighborhood: von Neumann, "each of them surrounded by up to four neighbors"** (p. 79). The phrase "up to"
  and fn 9 (p. 83-84) point to a **bounded (non-wrapping) grid**. Fn 9 reads: "Bremer and Mihalka (1977) and Cusack
  and Stoll (1990) use a hexagonal map consisting of ninety-eight states. Duffy's (1992, 244) rectangular lattice
  containing 128 actors is closer to the present configuration, although his system is open by allowing neighbor
  relations to wrap around the system boundaries." The "although" contrasts Duffy's torus with Cederman's lattice, so
  Cederman's is most likely **not a torus**. This is inferred, not stated outright.
- **Each unit gets initial resources drawn from a normal distribution, mean 50, sd 10** (p. 84).
- **Two actor types** (p. 79): "predator" and "prey". The text says they correspond "to status quo and revisionist
  states, the latter denoted by the shaded areas". The order is reversed: predator = revisionist (shaded in Fig 4.3),
  prey = status quo.
- **Predator frequency** = "the density of predator states at the outset of the analysis" (p. 78). Swept over
  0, 5, 10, 20, 40, 60, 80, 100 % (p. 92). The text says only that "the exact location of the predators varied
  across the replications" (p. 92). It does not say whether the count is exact (round(f*N) placed at random) or
  each cell is a Bernoulli draw.
- **Hierarchy limited to two levels**: a capital plus provinces (p. 84; fn 10: this excludes feudal or federal
  structures). Sovereignty is "direct and absolute" (Coleman 1990). Conquered units lose their actor capacity
  completely: the head handles foreign relations, and internal peace is guaranteed, with no revolts or civil war
  (p. 84; fn 11: civil war arrives in ch. 5).

#### 1.2 Neighbors and action scope
- "Interactions [are limited] to territorial neighbors. ... the capital 'inherits' as neighbors those of its
  provinces. This means that great powers may have an almost global action scope." (p. 87)
- Interaction is between pairs of **sovereign** actors (p. 87: "for each pair of sovereign actors"). "Only bilateral
  encounters are considered, so the strategic situation resembles a two-by-two game." (p. 85)
- Actors "have no global knowledge of the system" (p. 79).

#### 1.3 Simulation loop (Fig 4.6, p. 85; Fig 4.9, p. 91)
Basic loop (p. 84): **Decisions -> Laws of Interaction -> Structural Change**.
"First, all sovereign actors make their decisions. Second, the consequences for the resource distribution between
the actors are calculated according to the rules of interaction. Third, conquest transforms the system's structure."
With alliances (Fig 4.9): **Perception of Threat -> Decisions -> Laws of Interaction -> Structural Change -> Alliance
Formation**, repeated.

Synchronous design (pp. 82-83): "giving all sovereign states the chance to act simultaneously in every period."
Fn 7 (p. 83): "The disadvantage of the parallel design is the possibility of computational conflicts. However, it is
easy to circumvent this difficulty by locking the units that have already been modified in the same time period, a
common trick in time-sharing operation systems. To make the execution unbiased, the order is randomly determined in
every time period. See Duffy (1993, 14-15) for a similar solution."
The model allows "one move per state and time period", and move sequences "may span over several simulation cycles".
"Behavioral continuity is assured by endowing the actors with a short-term memory" (p. 83), which gives "protracted
warfare".

#### 1.4 Decision phase (Fig 4.7, p. 87; quoted exactly from the page image)
```
for each sovereign neighbor j,
        if j played D in the previous period then
                Play D
        else
                Play C (Tit For Tat)

if i is a predator and there is no action on any front then
        look for the weakest sovereign neighbor j*
        if R(i)/R(j*) > superiority_ratio then
                Randomly select own agent province and target in j*
                Play D (i.e. launch unprovoked attack against j*)

for each sovereign neighbor j,
        Allocate R(i)/n to front j
```
Caption: "the decision phase of state i, with resources R_i and n neighbors" (p. 86).
- **Prey (status quo)**: pure TFT. "always respond with the same action as the opponent played in the last round and
  never initiate hostilities" (p. 85).
- **Predator**: TFT plus unprovoked attack under "prudence". (a) The **"Schlieffen plan" rule**: "a predator only
  considers an unprovoked attack if it is not already involved in warfare on another front" (p. 85). (b) It attacks
  only if the power ratio exceeds the **superiority ratio** (p. 85). "Should there be many potential victims, the
  aggressor selects the weakest of them." (p. 85). The superiority check uses **total** resources R(i)/R(j*) (p. 88:
  "the balance between the total resources of each side"). Fn 12: the strategic literature often uses 3:1
  (Mearsheimer 1983), and the Bremer/Mihalka family fixes it at 1 with a probabilistic misperception term.
- **Combat path** (p. 86): "randomly selecting a 'combat path' consisting of an agent and a target province. If both
  ... are primitive actors, they also serve as agent and target respectively. ... The current implementation lets
  the attacker randomly choose any of its provinces (including the capital) that are adjacent to the defending state
  as the agent. Thereafter, the attacker selects a target randomly from the provinces of the defending actor that
  border the agent province."
- **Resource allocation** (p. 86): "divides the total resources evenly among all territorial neighbors regardless of
  local need. For example, a sovereign primitive actor with four neighbors devotes one fourth of its resources to
  defense against each neighbor. ... great powers with long borders need more resources". n = number of
  (sovereign) neighboring states. Fn 14: the rule is admittedly unrealistic, and Proportional Resource Allocation
  (PRA) arrives in ch. 5.
- Fn 15: the predators resemble Cusack and Stoll's "primitive power-seekers", and the prey resemble their
  "collective security" style without alliances.

#### 1.5 Laws of interaction (Fig 4.8, p. 88; checked against the page image)
Rows: A_i (act of state i). Columns: A_j. Each cell gives (ΔR_i, ΔR_j):

|        | A_j = C        | A_j = D            |
|--------|----------------|--------------------|
| A_i = C | 0, 0          | -kR_j, 0           |
| A_i = D | 0, -kR_i      | -kR_j, -kR_i       |

"where A_i = act of state i; R_i = local resources of state i; k = rate of destruction".
- **k = 0.05** in all reported simulations (p. 87). "the resource level of an actor under attack is reduced by 5
  percent of the locally allocated resources of the attacking state" (p. 88).
- In the Richardson-like logic, an aggressor gains nothing immediately (it is not a Prisoner's Dilemma) (p. 88).
- **Ch. 5 fn 1 (p. 109)**: in all ch. 5 runs the CD/DC entries were set to 0,0, and "Tests indicated that this
  change did not alter the results in any significant way." In ch. 4 the asymmetric entries are active. This should
  be a named switch.
- **Victory ratio** (p. 88): "Each conflict must end in victory for one party. The outcome is governed by the victory
  ratio, which unlike the superiority ratio compares the locally allocated resources rather than the balance between
  the total resources of each side. Should the balance exceed the victory ratio in either direction, the stronger
  party has won the conflict."
- **Victory ratio = superiority ratio** in this study (p. 88). Fn 17 (p. 88): see ch. 5 for localized
  decision-making.
- **Harvest** (p. 88): "each actor reaps a 'harvest' drawn from a normal distribution with adjustable parameters. In
  the present study, mean was two and the standard deviation five. A corporate state receives the sum of the harvests
  from its capital province and all subordinate members." Fn 18: the distribution is common to all states, so there
  is no differential growth.
- The new resource level = old + combat differences (summed over fronts) + harvest (p. 88).

#### 1.6 Structural change (pp. 88-89)
"If the resource balance exceeds the victory ratio, the superior party conquers the target province." Three cases:
1. **Target is an independent primitive actor**: it is subordinated under the head of the conqueror (example: T11
   absorbs R6).
2. **Target is a province of a corporate actor**: the province is transferred to the attacker (example: B3 invades
   C4, which becomes a subject of B3 rather than G8).
3. **Target is the capital of a corporate actor**: "the entire corporate unit collapses into its primitive units when
   the capital falls." Fn 19: there is no immediate takeover of large states. The attacker may absorb the fragments
   later, in competition with others.
- **Detached provinces** (p. 89): "if there are provinces of this state that can no longer be reached from the
  capital province as a consequence of the loss of a newly invaded province ... the truncated provinces are made
  independent to exclude the possibility of enclaves." Fn 20: contiguity comes from the sovereignty assumption.
- **Resources on independence** (p. 89): "When a state loses its sovereignty, it abandons control of its own
  resources. Upon regaining independence, a province receives an equal share of the resources previously controlled
  by the center. For example, if a province breaks out of a multiprovince state consisting of five subordinated
  members and a center prior to the secession, it will receive a sixth of the total resources. It also regains its
  previous strategic orientation". Each cell therefore keeps a latent predator/prey type for life.
- p. 97: "conquest pays because the vanquished states delegate all their resources to the conqueror". This is
  "automatic-resource transfer to the capitals", which ch. 5 replaces with a tax.
- Hegemony (a single state) ends a run: "All simulations ran until hegemony occurred or one thousand time periods
  elapsed." (p. 92)

#### 1.7 Defensive alliances (pp. 89-91)
- Realist premises (p. 90): alliances counter external threats, are expedient, and are ad hoc. The model is
  "decentralized, defensive" only, with no bandwagoning. "alignments should be perfectly binding and credible"
  (an "optimal alliance mechanism"). Allies "coordinate their foreign-policy actions without directly redistributing
  or sharing their resources."
- **Phase 1, Perception of Threat** (runs before Decisions): "each state updates the trust score for each neighbor,
  as a result of their behavioral records. This score gradually increases in the absence of unprovoked attacks or
  conquest. Should the latter occur, however, relations deteriorate quickly. Thereafter the actor singles out one of
  the neighbors as the prime threat. If the state perceives no aggressive behavior, there is no threat." (pp. 90-91)
- **Trust update, fn 23 (pp. 90-91), exact from the page image**: "the trust score T depends on a sensitivity
  parameter s. The latter parameter determines the impact of the change dT = ±1000 on the new trust score:
  **T(t) = (1 − s) T(t − 1) + s dT**. The value of the sensitivity parameter differs depending on whether dT is
  negative or positive. In the former case, the perceptual change has a greater impact since negative experiences
  and threats make themselves felt more quickly than positive ones. Based on observations of the system, I
  calibrated the sensitivity parameter to **s = 0.5 and s = 0.01 for negative and positive updating** respectively."
  He calls the parameter choice "necessarily arbitrary" and rejects Bayes's rule as "too fast and symmetric".
- **Phase 2, Alliance Formation** (the last step of each iteration, p. 91): "every state that perceives a threat
  announces its willingness to counter the threatening actor. If there is more than one state that feels threatened
  by a given actor, an alliance automatically forms against that state. ... if there are no longer at least two
  states that perceive a common threat, the coalition immediately dissolves." Fn 24: alliances persist across
  periods only while members keep feeling threatened by the same aggressor.
- **Effects** (p. 91, after Cusack and Stoll 1990, 120):
  - *Deterrent*: "If such a [predator] state faces a defensive alliance, its decision to launch an unprovoked attack
    incorporates the sum of resources of the alliance members rather than the capabilities of the chosen victim
    alone. Should the potential attacker itself belong to an alliance, it can only count on support from the other
    members if the alliance is targeted against the selected victim."
  - *Defensive*: "the attack on an alliance member automatically creates an obligation to support the defending
    state ... each member of the alliance strikes against the aggressor, resulting in a collective defense. If the
    victim happens to be a member of the aggressor's alliance, the latter state is automatically excluded from the
    alliance".

#### 1.8 Parameter table (ch. 4 defaults)

| Parameter | Value | Page |
|---|---|---|
| Grid (experiments) | 10 x 10 (100 units) | 83 |
| Grid (illustrations) | 20 x 20 | 79-81 |
| Neighborhood | 4 (von Neumann), "up to four", most likely non-wrapping | 79, fn 9 p. 83-84 |
| Initial resources | N(50, 10) per unit | 84 |
| Harvest per unit per period | N(2, 5), summed over capital and provinces | 88 |
| Destruction rate k | 0.05 | 87 |
| Superiority ratio | 2 (offense-dominant) / 3 (defense-dominant) | 92 |
| Victory ratio | = superiority ratio | 88 |
| Predator frequency | 0, .05, .1, .2, .4, .6, .8, 1.0 | 92 |
| Replications | 20 per cell, different seeds | 92 |
| Run length | until hegemony or 1000 periods | 92 |
| Hierarchy depth | 2 | 84 |
| Trust dT | ±1000 | fn 23, 90-91 |
| Trust s (neg / pos) | 0.5 / 0.01 | fn 23, 91 |
| Alliance minimum size | at least 2 states threatened by the same actor | 91 |

#### 1.9 Random elements (as stated)
- Initial resources N(50, 10) (p. 84). Predator locations (p. 92). Harvest N(2, 5) each period (p. 88). Random
  agent province and random target province (p. 86). Random processing order each period for conflict locking
  (fn 7, p. 83). The replications use different seeds, which vary "the initial power distributions and the exact
  location of the predators" (p. 92). Fn 25 compares this to experimental randomization (Fig 3.3).

---

### 2. Experiments and claimed results

#### Fig 4.1 (p. 75): conceptual
Polarity against time: a contraction phase from N to a "low N", then consolidation, with the "power politics"
band above 2. "Power politics" is defined as polarity reduced to "at most 10 percent of the original number of
states" but without universal empire (pp. 74-75). The motivating fact: Europe went from about 500 units in 1500 to
about 20 in 1900 (Tilly 1975, 24).

#### Fig 4.2 (p. 76): static causal scheme
Defensive alliances, free competition, and defensive technology act through balance of power and predation on
polarity.

#### Fig 4.3 (p. 79): initial 20 x 20 grid with 20 % predators
Rows A-T, columns 1-20. Shaded = predator, dot = capital. A single sample, not data. It could be digitized as a
picture, but there is no need.

#### Fig 4.4 (p. 80): sample run, 20 x 20, polarity against time (0-~165)
Starts at 400 and drops nearly linearly. Text: "After fifty time periods, system polarity has shrunk to about a
tenth of the initial population" (about 40). The curve reaches about 20 or less by t of about 55-60. After that it
stabilizes with "occasional 'jumps'" from imperial collapses (visible spikes near t of about 95 and 125). **Digitize
(low priority):** the polarity curve shape (a single run, qualitative only).

#### Fig 4.5 (p. 81): 20 x 20 after an imperial collapse
Almost everything is shaded (predators), with a small cluster of newly independent single cells at lower left (rows
K-R, columns 1-8). **Naming inconsistency**: the caption says "collapse of actor R5", the text says capital "R4",
and on p. 89 "the R5 empire before it fell to T11" and "T11 absorbing R6". Claim (p. 81): the fragments "will be
absorbed by their more powerful neighbors" within a few periods. Trend: "a smaller number of large predator
states".

#### Qualitative claims from the overview (p. 80)
(i) The number of states decreases. (ii) Predators grow. (iii) "the surviving units will be predominantly
predators, the more peaceful prey states having been eliminated in the selection process."

#### Experiment A: no alliances (pp. 92-94), Figs 4.10, 4.11
Design: predator frequency {0, .05, .1, .2, .4, .6, .8, 1.0} x superiority ratio {2 = offense, 3 = defense} x 20
seeds. The grid is 10 x 10. Each run lasts until hegemony or t = 1000.

**Fig 4.10 (p. 93) "Average state survival after one thousand time periods"** (y = average polarity at time 1000,
0-100). My readings from a 220-dpi render; these need proper digitizing:

| pred | 0 | .05 | .1 | .2 | .4 | .6 | .8 | 1.0 |
|---|---|---|---|---|---|---|---|---|
| defense (SR = 3) | 100 | ~67 | ~53 | ~42 | ~35 | ~6 | ~12 | ~11 |
| offense (SR = 2) | 100 | ~6-7 | ~3 | ~3.5 | ~4 | ~4 | ~4 | ~4 |

Claims: "the number of states falls dramatically even for small numbers of predators", which is "particularly
pronounced in the offense-dominated system"; there is "less attrition" in defense. Hegemonic runs presumably count
as polarity 1 in the average (they stop early).

**Fig 4.11 (p. 93) "Emergent polarity without alliances"**: stacked areas, counts out of 20 per predator frequency.
Categories: unipolar (1), bipolar (2), "multipolar" = low multipolarity (3-10), ">10" = high multipolarity (11-90),
">90" = "virtually no integration" (91-100) (p. 92-94). My cumulative readings (unipolar / +bipolar / +multipolar /
+">10" / remainder >90):

Defense (SR = 3):

| pred | uni | ≤2 | ≤10 | ≤90 | >90 |
|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 | 20 |
| .05 | ~4 | ~4-5 | ~6 | ~6-7 | ~13-14 |
| .1 | ~4 | ~6 | ~6-7 | ~10 | ~10 |
| .2 | ~7 | ~9 | ~9 | ~12 | ~8 |
| .4 | ~6 | ~12 | ~13 | ~13 | ~7 |
| .6 | **5** | **14** | **19** | 19 | **1** (exact, from text p. 94: 5 uni, 9 bi, 5 multi, 1 >90) |
| .8 | ~5 | ~11 | ~17 | ~19 | ~1 |
| 1.0 | ~3 | ~13 | ~18 | ~18 | ~2 |

Offense (SR = 2):

| pred | uni | ≤2 | ≤10 | ≤90 | >90 |
|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 | 20 |
| .05 | ~0-8? (narrow dark spike) | ~17 | ~19 | ~20 | ~0 |
| .1 | ~0 | ~7 | 20 | 20 | 0 |
| .2 | ~1 | ~5 | 20 | 20 | 0 |
| .4 | ~2 | ~5 | 20 | 20 | 0 |
| .6 | ~2 | ~4 | 20 | 20 | 0 |
| .8 | ~3 | ~6 | 20 | 20 | 0 |
| 1.0 | ~3 | ~3 | 20 | 20 | 0 |

(The x = .05 column in the offense panel is a narrow spike and hard to read; the bipolar band peaks near 17 there.)

**Operational definition (p. 94):** "power politics as either bipolarity or multipolarity—that is, outcomes
featuring from two to ten sovereign states."

**Claims (p. 94):**
- "power politics completely dominates the outcomes in the offense-dominated system."
- In defense it is "a common outcome ... though only for high predator frequencies."
- P1 "seems to possess considerable validity", especially in offense: "power politics is the structural outcome
  regardless of the initial frequency of predators."
- **P2 contradicted**: "defensive attitudes tend to increase the chances of state survival, but at the same time the
  likelihood of unipolarity increases significantly, thus making power politics less likely in defense-dominated
  systems."
- No statistical tests are given, only counts out of 20. Fn 26 contrasts this with Cusack and Stoll's linear
  regressions, which cannot capture hegemonic takeoff.

#### Fig 4.12 (p. 95): dynamic causal model with positive feedback
Competition -> (-) predation -> polarity (integration) -> (-) back to competition, giving a positive-feedback,
unstable equilibrium (Kaplan 1957). Defense dominance has a direct (-) link to predation and an indirect (-) link to
competition. "If the indirect link is stronger than the direct one, the net effect of defense dominance will be
negative" (p. 96). "The only thing needed to trigger the process is a somewhat uneven initial distribution of
resources and unbiased stochastic growth rates." (p. 95)

#### Experiment B: with alliances (pp. 98-99), Fig 4.13
Same design plus the trust/alliance mechanism. **Fig 4.13 (p. 99)** readings (cumulative, as above):

Defense (SR = 3):

| pred | uni | ≤2 | ≤10 | ≤90 | >90 |
|---|---|---|---|---|---|
| 0, .05, .1, .2 | 0 | 0 | 0 | 0 | 20 |
| .4 | ~3 | ~3 | ~3 | ~12 | ~8 |
| .6 | ~7 | ~7 | ~7 | 20 | 0 |
| .8 | ~16 | ~17 | ~17-18 | ~19 | ~1 |
| 1.0 | ~16-17 | ~18 | ~19 | 20 | 0 |

Offense (SR = 2):

| pred | uni | ≤2 | ≤10 | ≤90 | >90 |
|---|---|---|---|---|---|
| 0 | 0 | 0 | 0 | 0 | 20 |
| .05 | ~1 | ~1-2 | ~2 | ~3-4 | ~16 |
| .1 | ~7 | ~7 | ~7-8 | ~13 | ~7 |
| .2 | ~9 | ~9-10 | ~10 | ~17 | ~3 |
| .4 | ~12 | ~17 | ~19 | 20 | 0 |
| .6 | ~11 | ~17 | ~20 | 20 | 0 |
| .8 | ~16 | ~20 | 20 | 20 | 0 |
| 1.0 | ~15 | ~19 | 20 | 20 | 0 |

**Claims (pp. 98-99):**
- Alliances "offer some protection at least for low predation frequencies". This is "particularly strong in the
  defense-dominated system where there is almost no reduction of polarity for predatory rates below 20 percent". It
  is weaker in offense.
- "a remarkable dominance of unipolarity at the expense of power politics. From having been the most common outcome
  in the offense-dominated system, power politics becomes less likely than universal empire. In the
  defense-oriented runs, power politics almost does not appear at all except for predator rates above 80 percent."
- **P3 is "questionable"**.
- Mechanism (pp. 99-100): alliances encircle predators, which reduces competition and staggers the timing of
  expansion. A predator that breaks out has more room and reaches the "threshold of hegemonic takeoff", then
  "overwhelms whole subsystems of locally stable alliances. At this point, unified defenses may be formed, but as a
  rule, this is much too late."
- p. 108: "the counterintuitive results pertaining to the influence of defensive alliances may well be reversed
  under a different specification of the alignment mechanism ... the exact results should be interpreted with
  caution."

#### To digitize
Fig 4.10 (2 curves x 8 points), Fig 4.11 (2 panels x 8 x 4 boundaries), Fig 4.13 (same), and optionally Fig 4.4.
The PNG renders made for the reading (not kept; `hi108-108.png`, `hi114-114.png`, crops `f410.png`, `f411d.png`,
`f411o.png`, `f413d.png`, `f413o.png`). The only exact number in the text is defense/no alliances/60 %: 5/9/5/1.

---

### 3. Ambiguities a reimplementation must decide (each should get a named switch)

1. **Topology**: bounded or torus. "up to four neighbors" (p. 79) and fn 9 (p. 83-84) point to bounded. Default:
   bounded.
2. **Predator placement**: exact count or Bernoulli per cell (p. 78, 92).
3. **What persists a war: the TFT echo problem.** Taken literally, Fig 4.7 (p. 87) gives: i attacks j (D vs C); next
   period j plays D (TFT), but i plays C because j played C last and the Schlieffen rule blocks a new unprovoked
   attack (there is now "action on a front"). The pair then **alternates DC/CD** instead of fighting DD. The text
   says "the result of an attack is always war" (p. 79), "Each conflict must end in victory for one party" (p. 88),
   and gives a "short-term memory" for "protracted warfare" (p. 83), which implies a war stays DD until resolved.
   Switch: `literal_tft_echo` vs `war_until_victory` (default probably the latter as intended; report both).
   Related: does a predator re-attack the same target on the same path each period? Is "previous period" D the
   *front* status?
4. **Combat path for the defender's retaliation and for wars that a third party starts**: which provinces form the
   front when j plays D back, or when two corporate states fight? Is the attacker's path fixed for the whole war or
   redrawn each period? (p. 86)
5. **What "conquers the target province" means when the *defender* wins**: does it take the attacker's agent
   province, or does the war just end? (p. 88: "the superior party conquers the target province", and "in either
   direction".)
6. **Timing of the victory check**: before or after this period's ΔR, and in the same period as the attack? With
   victory ratio = superiority ratio, a primitive predator whose total ratio > SR and with the same n as the victim
   has local ratio = total ratio, and so could win in the first period.
7. **Local resources**: n = number of sovereign neighbor *states* (not border cells) (p. 86). Is allocation computed
   before or after harvest? Is a front's R_local used both for damage and the victory check? Do prey with no
   conflicts still split their resources?
8. **Resources at conquest** (p. 89, p. 97): when a primitive actor is absorbed, does the conqueror get its remaining
   resources ("vanquished states delegate all their resources to the conqueror", p. 97)? When a province changes
   hands between corporate states, do resources move (does the loser lose 1/(members) and the winner gain it)? The
   text implies only future harvests move.
9. **Equal-share rule on independence** (p. 89): stated for secession or collapse ("a sixth of the total"). For
   detached enclaves, are shares computed from the pre-loss center total, and does the center keep the rest? On
   capital collapse, does the capital's own cell also get 1/(m+1)?
10. **Negative resources**: the N(2, 5) harvest can be negative, and combat losses can exceed resources. Is there a
    floor at 0, and what happens at R ≤ 0? Not stated.
11. **Weakest neighbor tie-breaking**: whether only the single weakest is tested (Fig 4.7 says so) and how ties break
    (random?). "weakest" means smallest total R.
12. **Locking and order** (fn 7, p. 83): which units lock (target province, or both agent and target), and what does
    a locked conquest do (skip or defer)? There is a random order per period.
13. **The "no action on any front" test**: does a front count if a neighbor played D at i last period, or only if i
    is currently at war? Does a war between allies and the aggressor count as action?
14. **Hegemony stop**: a run stops at 1 state. Is polarity at t = 1000 taken as is for other runs (Fig 4.10 average
    includes early stops as 1)?
15. **Trust model** (fn 23): initial T value; what event sets dT = −1000 (an unprovoked attack *on me*, on anyone,
    any conquest by the neighbor?); is dT = +1000 every period otherwise; trust per neighbor *state*; the **threshold
    for "threat"** ("If the state perceives no aggressive behavior, there is no threat", p. 91: T < 0? any negative
    event ever?); the prime threat as argmin T; tie-breaking. None of these is stated.
16. **Alliance mechanics**: whether an alliance requires members to be mutual neighbors (it is regional only because
    threats are neighbors); whether one state can be in several alliances (each state has a single prime threat, so
    at most one alliance as a member); the deterrent sum (members' total R or local R; does it include the victim's
    own R? yes "rather than the capabilities of the chosen victim alone"); "count on support from the other members
    if the alliance is targeted against the selected victim" (the attacker's side sums its own allies' R); whether
    collective defense means each member plays D against the aggressor on *its own* front (they must border it) and
    whether that opens wars that can conquer provinces; whether alliance-triggered fronts count for the Schlieffen
    rule; and the exclusion of an intra-alliance aggressor (p. 91).
17. **Superiority vs strict inequality**: ">" is stated (Fig 4.7). Does victory need ">" or "≥"? (p. 88 "exceed").
18. **Simultaneous attacks**: whether two predators can attack the same victim in one period, and whether several
    conquests can hit one state (locking handles conflicts).
19. **Capital inheritance of neighbors**: a corporate state's neighbor set = states bordering any of its cells
    (p. 87). Is a front per neighbor state (one front even with a long border)? Yes, by the R(i)/n rule.
20. **Strategic orientation of a corporate state** is the capital's type. Provinces keep a latent type that is
    restored on independence (p. 89).
21. **CD/DC entries** active in ch. 4 but zeroed in ch. 5 (ch. 5 fn 1, p. 109): switch `asymmetric_losses`.
22. **Text and figure inconsistencies**: R4/R5/T11/R6 naming (pp. 81, 89); "predator and prey ... corresponding to
    status quo and revisionist" in reversed order (p. 79); and Fig 4.4's polarity at t = 50 looks lower than "about
    a tenth" (40).

---

### 4. Original software, code, predecessors

- **Software** (fn 8, p. 83): "The code was developed in THINK Pascal on a Macintosh and subsequently ported to Unix
  running on Hewlett Packard and Sun workstations as well as Turbo Pascal for Windows on a Pentium platform." The
  algorithms are synchronous, "particularly well suited for future implementation on a parallel machine". **No code
  availability is stated** anywhere in the book (a grep of the whole text found nothing).
- **Predecessors** (pp. 82-83):
  - **Bremer and Mihalka (1977)**: a pioneering geopolitical simulation of realism, giving only "suggestive sample
    runs". It is sequential: initiator/target pairs are chosen with power-proportional opportunities (p. 309). It has
    fixed move/countermove campaign sequences, complicated spoils-division rules (316-17), a superiority ratio fixed
    at 1 with probabilistic misperception, offensive balancing with endogenously credible one-shot alliances, and a
    common growth distribution. It uses a hexagonal map of 98 states. It also collects realist quotations (Waltz
    1959, Wright 1965) (p. 96).
  - **Cusack and Stoll (1990)**: "closely related, though significantly more efficient" and studied with statistics
    (regression). Power-proportional action opportunities (78-79), war-outcome rules (81-91), "power management
    styles" (71) including "primitive power-seekers" and "collective security" (141-42), deterrent and defensive
    balancing (120), misperception (102), and Fig 4.5.6 (156) on collective-security states. Only highly
    offense-dominated (superiority ratio 1). The hex map has 98 states. Cederman calls it his "conceptual guide".
  - **Duffy (1992, 1993)**: a parallel-machine version with a 128-actor rectangular lattice *with wraparound*,
    several initiators per round but power-allocated (1992, 252-53), and learning plus reciprocity (1993). The
    conflict-locking solution (1993, 14-15) parallels Cederman's.
  - Also: Dacey 1974, Schrodt 1981, the Game of Life (Gardner 1970), von Neumann 1966, Burks 1970, and Wolfram 1986.
- **Three claimed innovations over Bremer/Mihalka** (pp. 82-83): (1) a parallel and simultaneous scheme (powerful
  states attacking more often is emergent); (2) one move per period with short-term memory instead of game-tree
  campaigns; (3) province-level combat instead of spoils-division rules, which makes contiguity easy.

---

### 5. Theoretical claims to test

Three propositions (p. 78), each relating an independent variable to the likelihood of power politics (2-10
states):
- **P1, "Anarchy implies power politics"** (Waltz 1979, 119): power politics follows regardless of predator
  frequency or defense dominance. *Claimed*: largely holds in offense-dominated runs ("regardless of the initial
  frequency of predators"), and fails in defense-dominated runs and with alliances (p. 94, p. 105).
- **P2, "Defense-dominance increases the likelihood of power politics"** (defensive realism: Snyder 1991, Van Evera
  1984, Jervis 1978). *Claimed contradicted*: defense raises survival but raises unipolarity (p. 94).
- **P3, "Defensive alliances increase the likelihood of power politics"** (Walt 1987). *Claimed contradicted*:
  alliances protect at low predator rates but make unipolarity dominant (pp. 98-99).

Mechanism claims:
- **Hegemonic takeoff** (pp. 95-98): conquest-induced, self-feeding exponential growth through the
  competition-predation-integration positive feedback, triggered only by "a somewhat uneven initial distribution of
  resources and unbiased stochastic growth rates" (p. 95). Testable: the time series of the largest state's size,
  and the gap between first and second; unipolar runs should show an accelerating leader.
- **"Paradoxically ... offense-dominated worlds tend to remain pluralist"** (p. 96): more competition means more
  even growth among aggressors, which gives a balance. Defense "widens the window of opportunity for potential
  hegemons by thwarting the attempts of predatory rivals to catch up."
- **Regional balancing** (pp. 99-104): regional alliances encircle local predators, and a breakout predator then
  overwhelms locally stable subsystems. Regional security complexes emerge (Buzan 1991). Regional balancing
  "automatically acquires a more global quality as further states are eliminated" because larger states have more
  neighbors (p. 103). In bipolarity regional and global concerns coincide (Waltz 1979, 171), with "the same result
  without making any assumptions about uncertainty" (p. 104). Illustration: Renaissance Italy (pp. 100-102), where
  regional balance among Italian city-states left them open to the French invasion of 1494.
- **Emergence of power politics** (pp. 73, 105): "in order for the structural core of neorealism to apply, the state
  system needs to remain highly competitive. Under these conditions, power politics ... emerges almost regardless of
  states' motives." It requires a self-feeding predatory process to reduce polarity, and is threatened by hegemonic
  takeoffs.
- **Selection** (p. 80): the survivors are predominantly predators.
- **Stability-instability paradox** (p. 105): defensive mechanisms (technology, strategic beliefs, alliances) undermine
  global balance like cartels in markets.
- **Post-settlement claim** (p. 106): "once a geopolitical system has settled, defense dominance is likely to
  stabilize it even further". This is not tested in ch. 4; it could be tested by switching SR mid-run.
- p. 108: he concedes that the alliance results "may well be reversed under a different specification", and ch. 5
  checks robustness (power balancing with T_min, PRA, tax, adaptation).

---------------------------------------------------------------------------------------------------

## 2. Cederman (1997), *Emergent Actors in World Politics*, Chapter 5 "Extending the Emergent Polarity Model" (book pp. 109–135)

Source PDF: `papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf`.
**Page offset: PDF page = book page + 15** (book p.109 = PDF p.124; book p.91 = PDF p.106).
Equations were checked against rendered page images (the OCR mangles them). Figures to digitize are on
PDF pp. 126 (5.1), 127 (5.2), 130 (5.3), 132 (5.4), 134 (5.5), 135 (5.6), 141 (5.8), 142 (5.9), 144 (5.10),
145 (5.11), 146–148 (5.12, 5.13a–g).

---

### 0. Base model that Chapter 5 modifies (Ch. 4, pp. 79–92), only what's needed

- **Grid**: square grid; "All the findings presented below were generated in a ten-by-ten system" (p.83). So N = 100 primitive units. Neighborhood: von Neumann, "each of them surrounded by up to four neighbors" (p.79). Boundaries are not wrapped (contrast with Duffy's torus, fn 9 p.84; "up to four" implies edges/corners have fewer).
- **Initial resources**: each unit ~ Normal(mean 50, sd 10) (p.84).
- **Hierarchy**: two levels only, capital + provinces (p.84). Capital "inherits" the neighbors of its provinces (p.87).
- **Loop** (fig 4.6, p.85): Decisions → Laws of interaction → Structural change. With alliances (fig 4.9, p.91): Perception of threat → Decisions → Laws of interaction → Structural change → Alliance formation.
- **Decision phase (fig 4.7, p.87, quoted)**:
  ```
  for each sovereign neighbor j,
      if j played D in the previous period then Play D
      else Play C (Tit For Tat)
  if i is a predator and there is no action on any front then
      look for the weakest sovereign neighbor j*
      if R(i)/R(j*) > superiority_ratio then
          Randomly select own agent province and target in j*
          Play D (i.e. launch unprovoked attack against j*)
  for each sovereign neighbor j,
      Allocate R(i)/n to front j
  ```
  - Prey = pure TFT; predator = TFT + unprovoked attack under the "Schlieffen" rule (no other active front) and superiority ratio on **total** resources (p.85). Agent province chosen randomly among own provinces adjacent to the target state; target province chosen randomly among the target's provinces bordering the agent (p.86).
  - Equal allocation R(i)/n across the n sovereign neighbors (p.86). Fn 14 (p.86) already announces PRA: "Proportional resource allocation sets the power level proportional to each opponent's power allocation in the previous period (see chap. 5 below)."
- **Laws of interaction (fig 4.8, p.88)**: payoff (resource change) matrix with k = destruction rate = 0.05:
  CC: 0, 0; C(i)/D(j): −kRj, 0 ... i.e. the side attacked loses k × attacker's **locally allocated** resources; DD: both lose. **Chapter 5 changes this — see §1.0.**
- **Victory**: "victory ratio" compares **locally allocated** resources; "Should the balance exceed the victory ratio in either direction, the stronger party has won" (p.88). "In the current study, the victory ratio coincided with the superiority ratio" (p.88).
- **Harvest**: each actor per period gets Normal(mean 2, sd 5); corporate state receives the sum over capital + all provinces (p.88).
- **Structural change** (p.89): conquered independent unit → province of conqueror; conquered province → transferred; conquered capital → whole state collapses into primitive units. Provinces cut off from the capital become independent (no enclaves). Regained independence: province gets equal share of center's resources (1/(provinces+1)), and "regains its previous strategic orientation" (p.89; p.89 says Ch5 has "a more realistic rule that allows the provinces to keep their resources after conquest" — that is the two-level extension).
- **Behavioral alliances (Ch4)**: trust update, fn 23 pp.90–91: dT = ±1000; "T(t) = (1 − s) T(t − 1) + s dT", s = 0.5 for negative dT, s = 0.01 for positive dT. Prime threat = one neighbor; alliance forms automatically if ≥2 states perceive same threat; dissolves when <2 (p.91). Deterrent balancing: predator's attack calculus uses sum of alliance members' resources; defensive balancing: every member attacks the aggressor; an aggressor that attacks a member of its own alliance is expelled (p.91).
- **Experiments (p.92)**: predator frequency ∈ {0, 5, 10, 20, 40, 60, 80, 100}%; superiority ratio 2 = "offense-dominated", 3 = "defense-dominated"; **20 replications** per setting with different seeds; run "until hegemony occurred or one thousand time periods elapsed".
- **Outcome categories** (p.92–94): unipolarity (1), bipolarity (2), low multipolarity (3–10, labelled "multipolar"), high multipolarity (11–90, ">10"), and "virtually no integration" (91–100, ">90"). **"Power politics" = 2 to 10 sovereign states** (p.94).
- Ch.4 headline findings that Ch.5 tests for robustness: (P2 contradicted) defense dominance makes power politics *less* likely (more unipolarity) (p.94); (P3 contradicted) defensive alliances make unipolarity *more* likely (fig 4.13, p.99).

---

### 1. The extensions, rule by rule

#### 1.0 Global change for ALL Chapter 5 runs (fn 1, p.109)

> "In all runs involving the extended versions of the Emergent Polarity Models, the same basic parameters were used as previously. The only change, except for those associated with the respective extensions, pertained to the laws of interaction. To prepare the ground for the PRA mechanism described below, the experiments were run without resource differences in the CD and DC exchanges (e.g., the entries in the corresponding boxes were 0,0 in fig. 4.8). Tests indicated that this change did not alter the results in any significant way."

So in Ch.5 only DD (mutual combat) destroys resources. Unprovoked attack against a not-yet-responding TFT state costs neither side in that period. (Implementation switch: `cd_dc_damage = false` default for Ch.5; `true` reproduces Ch.4.) The "tests" are not shown.

All other base parameters (10×10, N(50,10), harvest N(2,5), k=0.05, 20 reps, ≤1000 periods or hegemony, victory ratio = superiority ratio) are inherited by implication of "same basic parameters".

#### 1.1 Power balancing (simplified alliance mechanism) — pp.109–112

**What changes**: only the "perception of threat" module of fig 4.9. Alliance formation, deterrent balancing, and defensive balancing are unchanged (p.110: "thus leaving the actual implementation of the alliances unchanged"). The behavioral trust index (fn 23 Ch.4) is replaced by a resource-ratio score.

**Rules (p.110)**:
- Each period each state scans neighbors and computes a trust score "that is the negative ratio between each neighbor's resource level and their own."
- Perfect information about neighbors' strategic type: "all actors, whether predators or prey, consider balancing against predator states only. Prey states are never seen as threatening regardless of their strength."
- "States under attack count any attacking as an immediate, rather than potential, threat, and always balance against such threats before potential ones are considered."
- The most serious threat is the prime threat; if sufficiently serious, the state signals interest in an alliance against it.

**Equation 5.1 (p.110)**, exact:
> T(i, j) = − R(j)/R(i),   [5.1]
> "where R stands for the total resource level of each actor. The threat to balance against j* is the one that minimizes T(i, j), provided that T(i, j*) < T_min, where T_min stands for a predefined trust level over which alliance formation never takes place."

**Fn 2 (p.110)**: "If j is already attacking i, T(i, j) = −100 − R(j)/R(i), which ensures that direct threats are taken much more seriously than others."

**Worked example (p.110–111)**: R(1)=10, R(2)=25, R(3)=35, R(4)=40; all predators except 4; T_min = −3. T(1,2) = −2.5, T(1,3) = −3.5 < −3 → state 3 is prime threat. Without 3 as a neighbor, no threat. Combat with 2: T(1,2) = −102.5 → 2 is the prime threat. Combat with both 2 and 3: T(1,3) = −103.5 is lowest. (Good unit test.)

**Parameter T_min**:
- Fig 5.1: **T_min = −1** ("align themselves against predatory neighbors that are about as strong as they are", p.111).
- Fig 5.2: **T_min = −1.5** ("retarded power balancing"; "the threatening predators need to be one and a half times as powerful to trigger counterbalancing", p.111).

#### 1.2 Strategic adaptation (and strategic blurring) — pp.112–117

**What learns**: the *strategic type* of initially-prey states. Pure predators never change ("predators always remain strategically invariant", p.114). Alliances are replaced by adaptation: "Suppose that instead of forming an alliance as a response to threats, states change their own strategies in order to prepare for the worst" (p.114).

**How**: neither imitation nor payoff-based reinforcement. It is **threat-triggered stochastic type switching** (a two-state Markov chain per state, conditioned on perceived threat via eq. 5.1). No payoffs, no observation of others' success, no copying. Cederman calls it "socialization"/"simple behavioral adaptation" (p.113) and frames it with Waltz's emulation quote and Wendt's "bad apples", but the mechanism is not emulation.

**Rules (p.114)**:
- Third type: **conditional predator**. (Behavior when a conditional predator is not stated separately; implied same as predator: TFT + unprovoked attack under superiority ratio.)
- "p indicates the likelihood (per time period) of a threatened prey state switching to conditional predation; q stands for the probability of the reverse process" (q applies to "unthreatened conditional predators").
- Threat test: "Recall that T_min tunes the sensitivity of the threat mechanism. Thus, a state i is threatened if and only if there is no neighboring state j such that T(i, j) < T_min." **(Misprint: as written this is inverted; must mean "there is a neighboring state j such that T(i,j) < T_min" — see §3.)**
- "Again potential candidates for switching consider threats from predators and conditional predatory states only." (Conditional predators *are* perceived as threats — contagion.)
- T_min = −1 ("the same threat threshold as in figure 5.1 ... or simply parity", p.114).
- Base system: the **defense-dominated** system (superiority ratio 3) of fig 4.11 left panel (p.114). x-axis = pure predators' initial share; all others start prey (p.114).
- **Weak adaptation**: p = 0.5, q = 0.01 (p.115).
- **Strong adaptation**: p = 1, q = 0 (p.115): "the prey states immediately abandon their prey attitude and shift to predation, a strategic orientation they never abandon."

**Strategic blurring (p.116)**: Jervis's (1978) worse security dilemma when offensive and defensive orientations are indistinguishable. Implementation: "I made the prey states respond to threatening resource balances regardless of their neighbors' strategy." I.e., in eq 5.1 the candidate set j becomes **all** neighbors (prey included), not just predators/conditional predators. Fig 5.4 = fig 5.3 runs + blurring ("differing from those in figure 5.3 except for the addition of strategic blurring" — the wording is garbled; it means identical except for blurring).

#### 1.3 Proportional Resource Allocation (PRA) — pp.117–121

**What changes**: (a) the allocation step replaces R(i)/n with front-specific r(i,j); (b) predator attack decisions use front-level ratio r(i,j)/r(j,i) instead of total R(i)/R(j).

**Rules (p.118)**: "states earmark forces proportional to their neighbors' mobilization while paying close attention to each front's status. If combat occurred in the previous time period, the front is active, otherwise it is passive. For every time step, a state i announces how many resources r(i,j) it *could* commit to each front j in case of an attack, the combat status of other fronts being unchanged."

**Eq 5.2 (active front), exact**:
> r_new(i,j) = [ r(j,i) / Σ_k r(k,i) ] · R(i)   [5.2]
> "where the nonindexed right-hand side pertains to variables of the previous period, R(i) denoting the total resources to be distributed, r(j, i) the resource allocation of neighboring state j against state i, and Σ_k r(k,i) the sum of all resources currently targeted against state i in combat. Obviously, if j is the only active front, the ratio is one, implying that all resources R(i) go to this front."

**Eq 5.3 (passive front), exact**:
> r_new(i,j) = [ r(j,i) / ( r(j,i) + Σ_k r(k,i) ) ] · R(i).   [5.3]
> "Here the enemy's resources r(j,i), expected to be mobilized to prepare for the attack, have been added to the dominator [sic: denominator]."

Sum over k = active (in-combat) fronts only. With no active fronts, every passive front gets r = R(i) (every state can throw its entire resources at any single front; allocations are *conditional* commitments that sum to more than R(i)).

**Conquest/independence (p.120)**: "In case of conquest, the invading party inherits the previous, external resource levels of the conquered state. By the same token, a newly independent unit allocates its entire resource level to all fronts."

**Decision change (p.120)**: "a predator state i computes the power ratio ρ(i,j) = r(i,j)/r(j,i) for all neighbors j, singling out the state j* for which the ratio is minimized. If the ratio falls under the superiority threshold ... combat follows." **(Inverted — see §3; fig 5.7 has it right: weakest j*, attack if r(i,j*)/r(j*,i) > superiority_ratio.)**

**Worked example fig 5.5 (p.118–119)**: 2×2 world, R(i)=10, R(j)=30, R(k)=20, R(l)=40; i–j, i–k, j–l, k–l adjacent.
- Time 1, no combat: every front gets the full R ("r(i,j) = 30/(30+0) × 10").
- Time 2, k–l fighting: r(k,i) = 10/(10+40) × 20 = 4; r(l,j) = 30/(20+30) × 40 = 24.
- Time 3, i–k also fighting: r(k,l) = 40/(10+40) × 20 = 16; figure shows r(i,j) = 8.8 (= 30/(30+4) × 10 — consistent with 5.3).
- Time 4: text: "state l is ready to devote **more** resources to its front with state j: r(l,j) = 30/(16+40) × 40 = 21.4." **Arithmetic/eq error**: eq 5.3 gives r(l,j) = r(j,l)/(r(j,l) + r(k,l)) × R(l) = 30/(30+16) × 40 = **26.1**, which *is* "more" than 24. The printed 21.4 is *less* than 24 and uses a denominator (16+40) that mixes k's allocation against l with l's own allocation against k. The figure also prints 21.4. Use 26.1 (eq 5.3) as default; this is a unit test where the book's printed value is wrong.

**Experiments**: fig 5.6, offense (2) and defense (3) at the 8 predator frequencies, no alliances (fn 5 says alliances+PRA not reported).

#### 1.4 Two-level action (civil war + taxation) — pp.121–126

**What changes**:
1. "Diplomatic portfolio": a sovereign state's relations now include its own provinces as fronts; a province's only relation is the asymmetric one with its center (p.123).
2. Provinces are actors: "Each province can thus be seen as a predator contemplating an attack on the center as soon as the periphery-center power balance allows. The center ... plays a simple defensive tit-for-tat strategy" toward provinces (p.123). Predators never attack their own provinces unprovoked ("Predators consider unprovoked attacks against other states but never the regions", p.123).
3. PRA is used throughout ("the decision criterion use local, front-related resources r rather than total ones R, all according to the PRA scheme", p.123). Center must allocate to internal and external fronts; domestic conflict can block foreign unprovoked attacks (Schlieffen rule now counts domestic fronts).
4. Civil war uses the same combat laws as interstate war; domestic victory ratio "may or may not be the same" but is set equal here (p.123–124). Successful revolt → secession; failed → province stays (p.123).
5. **Tax replaces automatic resource transfer**: "the provinces retain their own resources after conquest, although they lose parts of them through taxation. ... the tax is drawn as a given share from the province's 'harvest' in each period, whether this income is negative or positive. ... the center may gain or lose from its tax imposition ... If the province does not dispose of any resources, however, the center receives nothing." (p.124)

**Fig 5.7, decision phase (p.124), exact**:
```
Center
for each sovereign neighbor or subordinated province j,
    if j played D in the previous period then
        Play D (respond to internal or external challenges to sovereignty)
    else
        Play C (Tit For Tat)
if i is a predator and there is no action on any foreign or domestic front then
    look for the weakest sovereign neighbor j*
    if r(i,j*)/r(j*,i) > superiority_ratio then
        Randomly select own agent province and target in j*
        Play D (i.e., launch unprovoked attack against j*)
for each sovereign neighbor or subordinated province j,
    Allocate r(i,j) to front j

Province
if the center j played D in the previous period then
    Play D
else
    if R(i)/r(j,i) > domestic_superiority_ratio then
        Play D (i.e., revolt against the center)
    else
        Play C
Allocate R(i) to the revolutionary front
```
(p.123: "Since there is only one front, all of the province's resources can be used for this purpose.")

**Parameters (p.125)**: base = offense-dominated PRA system of fig 5.6 (superiority ratio 2); **domestic superiority ratio = 2**; **internal victory ratio = 2**; **initial predator rate = 100%**; x-axis = tax rate (0..1). 20 replications presumably; ≤1000 periods ("at least not within one thousand time periods", p.125).

**Consequence worth noting** (derivable, not stated): with PRA, if the center has no active fronts, its passive allocation toward province j is r(c,j) = R(j)/(R(j)+0) × R(c) = R(c). So a province revolts only if R(prov) > 2·R(center). Cederman confirms the effect: "Because of the deterministic decision criterion, secession almost never takes place" (p.126).

#### 1.5 Geopolitical overextension (stochastic decisions/combat + distance-decaying tax) — pp.126–135

**Two changes (p.126)**: (1) "both decision-making and combat feature a probabilistic component"; (2) "the center's resource extraction declines according to a distance gradient".

**Stochastic decision (p.126–127)**: applies to "the predator states' and secessionist provinces' decisions to engage in combat". Logistic in the front-specific power balance ρ(i,j) = r(i,j)/r(j,i). Exact (from page image, p.127):
> Pr(ρ(i,j)) = 1 / ( 1 + {ρ(i,j)/ρ0}^(−c) ).
"ρ0 the threshold value (i.e., the superiority ratio) and c a tunable parameter determining the step's slope". Deterministic rule = c → ∞.
- Fig 5.9 parameters: **ρ0 = 3, c = 5** (p.128); figure label "superiority threshold = 3.0".
- Text claims: "at a resource ratio of two, there is a 10 percent chance" (check: Pr(2) = 1/(1+1.5^5) = 0.116 ✓ approx); "The probability passes the 90 percent threshold when the attacker gains four times the power of its potential victim" (**check: Pr(4) = 1/(1+0.75^5) = 0.808; 90% is reached at ρ = 3·9^(1/5) ≈ 4.66** — text overstates; see §3).
- "Steepest slope for the threshold value" — in ρ the steepest point of this log-logistic is slightly below ρ0 for c=5, but at ρ0 in log ρ; minor.
- Fn: chapters 7 and 8 give the rationale for this logistic function (p.128).

**Stochastic combat (p.128)**: "similarly stochastic combat criteria based on exactly the same function (cf. figure 5.9). Thus, on average, an actor prevails if its field superiority surpasses the victory ratio, but since the mechanism is stochastic, victory may sometimes happen earlier or later than this threshold suggests." I.e., per-period victory probability = Pr(local ratio) with ρ0 = victory ratio, presumably same c.

**Distance-dependent tax (p.128–129)**: "For every unit's distance from the center, the tax contribution declines by a given fraction. ... a discount rate of 0.7, which means that provinces at one unit's distance from the center pay 70 percent of the nominal tax rate, and those at two units' distance pay 0.7 × 0.7 = 49 percent". Exact:
> τ(d) = τ0 s^d
with **s = 0.7, τ0 = 0.4** (p.129; fig 5.10 plots τ(d) for d = 0..10, starting at 0.4).

**Run (p.129–134)**: a single illustrative run, 10×10 grid, "for a duration of more than one thousand periods" (the figures show t ≈ 1350–3800, so it is ≥3800 periods). All states are predators (p.131: "Since all states are predators, the shading ... has been suppressed"). Superiority ratio for this run not stated (see §3).

---

### 2. Experiments, figures, claimed results

Format of the polarity figures (5.1, 5.2, 5.3, 5.4, 5.6, 5.8): stacked-area charts of the **count of replications (out of 20)** in each final-polarity category, stacked bottom→top: unipolar, bipolar, multipolar (3–10), ">10" (11–90), ">90" (91–100). The x-axis knots are the 8 predator frequencies 0, .05, .1, .2, .4, .6, .8, 1.0 (linear axis, so the kinks at 0.05/0.1 are crowded at the left). Each two-panel figure = 2 × 8 × 20 = 320 runs. Polarity measured at end of run (hegemony or t = 1000). Readings below are by eye from 110-dpi renders; **digitize at ≥300 dpi before use**.

| Fig | Page (book/PDF) | Setting | What it shows / claim |
|---|---|---|---|
| 5.1 | 111/126 | Power balancing, T_min = −1; defense (3) left, offense (2) right | "unipolar and massively multipolar outcomes again dominate almost all parameter combinations"; "very few differences" from fig 4.13 (behavioral alliances); defense: power politics "somewhat more common" than with behavioral alliances, "but unipolarity continues to dominate the lower polarity outcomes except for very high predator frequencies" (p.111). By eye: defense panel is >90 up to ~0.2–0.4 predators, unipolar grows to ~12/20 at 0.8; offense panel unipolar ≈12–13/20 from 0.2 upward, bipolar+multipolar ≈4–6. |
| 5.2 | 112/127 | Power balancing, T_min = −1.5 ("retarded") | "once alignments are delayed, power politics becomes more common. In the defense-dominated system, bipolarity and low-level multipolarity occur even for modest predator frequencies. In the offense-dominated setting, the domain of power politics is also expanded." Still, vs fig 4.11 (no alliances), "Alliances make unipolar and massively multipolar outcomes more frequent than in their absence" (p.112). By eye: offense unipolar still ≈11–13/20. |
| 5.3 | 115/130 | Adaptation, defense (3), T_min = −1, no alliances; left weak (p=.5,q=.01), right strong (p=1,q=0) | Power-politics "wedge" less pointed, less dependent on predator density; weak: power politics "extends into the below-20-percent predator area, though only to a limited extent", unipolar frequency down; strong: "a few bad apples have a tendency to generate power politics in many cases (roughly a third below 20 percent predation)" (p.115–116). |
| 5.4 | 117/132 | Same as 5.3 + strategic blurring | "power politics becomes even more common"; weak: "bi- or multipolarity in at least a third of the cases for low initial predation"; strong: "more than half of the outcomes fall into the category of power politics"; "closely resemble the offense-dominated system in figure 4.11" (p.116). By eye: strong panel bipolar band dominates (~6–14/20) across x ≥ 0.05. |
| 5.5 | 119/134 | PRA toy example (2×2, R = 10,30,20,40) | Arithmetic illustration; 4 snapshots. Time-4 value printed wrong (21.4 vs eq-5.3 26.1). |
| 5.6 | 120/135 | PRA + front-level decision; defense (3) left, offense (2) right; no alliances | "little impact on the general findings about offense and defense dominance (cf. fig. 4.11). Again, power politics becomes less common as the system turns more defense oriented. Moreover, a truly offense-dominated system approximates the structural invariance of power politics" (p.120). By eye: offense panel multipolar ≈ 14–18/20 for all x ≥ 0.1; defense panel unipolar ≈ 7–10/20 for x 0.2–0.6. |
| 5.7 | 124/139 | Pseudocode (two-level decision) | — |
| 5.8 | 126/141 | Two-level, 100% predators, offense PRA, x = tax rate | "power politics appear to depend positively on the tax rate"; lower tax → more unipolarity and more >10/>90; "Below a tax rate of 20 percent, power politics never occur, and below 10 percent the system does not converge to low-level polarity at all (at least not within one thousand time periods)"; tax 100% ≈ fig 5.6 offense at 100% predators, "almost complete dominance of multipolar systems" (p.125). Tax-rate grid not stated; kinks by eye at roughly 0, .05, .1, .2, .3, .4, .6, .8, 1.0 (verify). |
| 5.9 | 127/142 | Logistic decision curve, ρ0 = 3, c = 5 | Plot only. |
| 5.10 | 129/144 | τ(d) = 0.4·0.7^d, d = 0..10 | Plot only. |
| 5.11 | 130/145 | One overextension run, t ≈ 3450–3800; territorial size (number of provinces, scale 0–100) of each state over time | Narrative: "At the beginning of the recording, the system is unipolar. After about time period 3500, a challenger appears, creating a temporary bipolar regime. Decline affects this state at about time 3600, however, and about forty time periods later both leading powers suffer terminal fragmentation. In their place, five intermediate powers rise ... The new multipolar regime attains a considerable degree of stability" (p.130). Punctuated equilibrium analogy (Eldredge & Gould). By eye the largest state is only ~30–35 cells before ~3650 — "unipolar" is used loosely. |
| 5.12 | 131/146 | Same run, t ≈ 1350–1700 | "first major state collapse since the creation of the system. This event terminates a rather stable period of bipolarity at about time 1650" (p.131). |
| 5.13a–g | 131–133/146–148 | 10×10 resource maps at t = 1630, 1634, 1637, 1639, 1645, 1646, 1653 with state borders; bold = capitals | Narrative (p.131–133): left empire (center 3927) attacks a small state (center 3168) at t=1630; by 1634 center down to 3379 and a (not the strongest) province revolts; 1637 center 2802, another revolt (province 2428) and an external attack by the 5042 power; 1639 external attack repelled but center at 1812, lost two provinces (one secession, one external); 1645 four more provinces seceded (1133, 1904, 2110, 1291); 1646 two "strategically located rebellions cut off more than half of the territory"; center at 556, "lower than all its provinces"; 1653 "the final coup de grace is dealt by newly independent states". Resource levels ~1000–5000 per cell at t≈1630. |

**What to digitize**: 5.1, 5.2, 5.3, 5.4, 5.6, 5.8 (5 stacked boundaries × 8–9 x-knots per panel; 11 panels). Also the Ch.4 comparison baselines 4.11 and 4.13 (book pp.93, 99; PDF 108, 114), since every Ch.5 claim is phrased relative to them. 5.11/5.12 only qualitatively (one run; not reproducible seed-for-seed). 5.13a could serve as a sanity check for resource magnitudes after ~1600 periods.

No statistics, no confidence intervals, no tests; all comparisons are visual across 20-replicate stacked counts.

---

### 3. Ambiguities, misprints, unstated details (with page refs)

**Global**
1. **CD/DC damage set to 0,0** for all Ch.5 runs (fn 1, p.109) — easy to miss; "Tests indicated that this change did not alter the results" (not shown). Switch.
2. Run length/replications for Ch.5 not restated; assumed 20 reps, stop at hegemony or t = 1000 (p.92 via "same basic parameters", fn 1 p.109). p.125 confirms 1000 for fig 5.8.
3. Whether alliances are on in figs 5.3/5.4/5.6/5.8: implied off (adaptation is "instead of forming an alliance", p.114; PRA+alliances explicitly left to fn 5 p.121). Not stated for 5.8.
4. Whether figs 5.1–5.4 use equal allocation (yes by chapter order — PRA comes later) — not stated explicitly.
5. Victory ratio = superiority ratio throughout (p.88), so under defense dominance victory also needs 3:1 local superiority.

**Power balancing**
6. Fn 2 (p.110) "If j is already attacking i": does a *prey* neighbor that is retaliating (TFT D) count as "attacking"? Main text says balancing is against predators only, but fn 2 / "count any attacking as an immediate ... threat" suggests any attacker. Switch: `direct_threat_any_attacker` vs predators-only.
7. Strict "<" vs "≤" T_min: worked example uses strict (p.110). At T_min = −1, "about as strong" means strictly stronger in total resources.
8. R is total resources of the *state* (corporate R), and neighbors are state-level (capital inherits neighbors).
9. Only one prime threat per state; alliance forms when ≥2 states name the same prime threat (from Ch.4 p.91). Unchanged.

**Strategic adaptation**
10. **Misprint, p.114**: "a state i is threatened if and only if there is **no** neighboring state j such that T(i, j) < T_min." Logically inverted; must be "there is a neighboring state j". Implement the corrected rule (literal reading would make states threatened when nobody is threatening, which contradicts the whole section).
11. Conditional predators' behavior not specified; assume identical to predators (unprovoked attack when Schlieffen + superiority conditions hold). Do they attack while "threatened"? Unstated.
12. Timing/order of switching within the loop (before or after decisions; synchronous or sequential) unstated. Per-period Bernoulli with p, q.
13. Does q apply when unthreatened only (yes per p.114 "unthreatened conditional predators")? Does a threatened conditional predator stay (yes, implied)?
14. Does the -100 attacking offset (fn 2) also apply in adaptation? Unstated; under T_min = −1 any attack by a (conditional) predator would trigger switching.
15. Conquest/independence: does a newly independent province "regain its previous strategic orientation" (Ch.4 p.89) — including conditional status? Unstated for adapted types.
16. Strategic blurring (p.116): "made the prey states respond to threatening resource balances regardless of their neighbors' strategy" — unclear whether conditional predators' *switch-back* test also uses blurred threat. Also, with blurring prey can be "threats" so adaptation spreads through strong prey; implement candidate set = all neighbors for both tests by default, with a switch.
17. Adaptation is a *threat-response* rule, not learning from payoffs. The Waltz "emulation" framing (p.113) is not what is implemented; worth noting as a framing/mechanism mismatch.

**PRA**
18. **Eq 5.3 example error, p.119**: printed r(l,j) = 30/(16+40) × 40 = 21.4 ("more resources") contradicts eq 5.3 (= 30/(30+16) × 40 = 26.1, which *is* more than 24). Default to equation; switch to reproduce printed arithmetic is not meaningful (no consistent rule produces it).
19. **Decision rule inverted in text, p.120**: "singling out the state j* for which the ratio is minimized. If the ratio falls under the superiority threshold ... combat follows." With ρ = r(i,j)/r(j,i), attack should be on the *maximizing* j* when ρ *exceeds* threshold. Fig 5.7 (p.124) uses "weakest sovereign neighbor j*" and "r(i,j*)/r(j*,i) > superiority_ratio". Use fig 5.7. Also "weakest" — by R(j), by r(j,i), or by max ρ? Ambiguous; switch (`target = max_rho` default vs `min_total_R`).
20. Which r values are compared: the *announced* conditional allocations from the previous period (r(i,j) is what i "could" commit). Combat damage (k × local resources) presumably uses the actual allocated r on active fronts. Not explicit.
21. Does Σ_k in 5.2 include j itself? For active front j yes (j is in combat), so the ratio for the sole active front = 1, as stated. For 5.3 j is passive so not in Σ.
22. Initialization: first period r(i,j) = R(i) for all fronts (implied by time-1 example and "newly independent unit allocates its entire resource level to all fronts", p.120).
23. "In case of conquest, the invading party inherits the previous, external resource levels of the conquered state" (p.120) — i.e., the conqueror's r toward the conquered province's former neighbors is initialized from the conquered unit's r? Ambiguous.
24. Harvest/resources accounting under PRA: is R(i) the total (sum of r is not R) — damages subtract from R(i) proportionally? Unstated how front losses map back to R.
25. Fn 5 (p.121): "Preliminary runs confirm that the previous findings related to alliances are probably dependent on resource allocation in a nontrivial way." I.e. Cederman himself flags that the Ch.4 alliance result (P3 contradicted) may not survive PRA. Not shown — a prime target for us.

**Two-level**
26. What is the center's R(i) once provinces keep their resources? Implied: capital's own resources + tax receipts (p.124; the 5.13 narrative tracks "center's resources" separately from provinces). Harvest of the capital itself presumably untaxed and kept by the center.
27. Tax on negative harvest (p.124): center "may gain or lose" — so center's income = τ × harvest even if negative? And "If the province does not dispose of any resources, however, the center receives nothing" — for a province at ≤0 resources. Order of operations unstated.
28. Province combat: province allocates R(i) to the "revolutionary front"; does a province also fight external invaders itself, or does only the center (via its allocation r(center, foreign j))? Combat with foreign states is via randomly chosen agent/target provinces, but resources used are the state-level r. Unclear whether provinces' own resources are ever used in external defense — seemingly not.
29. Text p.125 "at left end of the axis, the rate comes close to the automatic resource transfer" — should be the **right** end (tax = 1.0). Misprint.
30. "The internal-victory ratio also coincides with its domestic counterpart" (p.125) — read as: internal victory ratio = domestic superiority ratio = 2.
31. Secession mechanics: successful revolt makes the province independent with its own resources (contrast Ch.4 equal share). Provinces cut off by the secession then become independent (no enclaves, p.89; implied by fig 5.13f "cut off more than half of the territory").
32. When a province revolts, the Schlieffen rule now blocks the center from foreign unprovoked attacks ("no action on any foreign or domestic front", fig 5.7).
33. Do provinces "regain previous strategic orientation"? All predators in these runs, so moot. Provinces' own type (predator/prey) irrelevant to the domestic rule — every province is a potential rebel.
34. Tax-rate grid for fig 5.8 not stated.

**Overextension**
35. **ρ0 for the overextension run not stated.** Fig 5.9 uses ρ0 = 3 ("to generate the curve"), but the preceding two-level runs use 2. Victory-function ρ0 likewise unstated. c = 5 presumably used for both.
36. **Numerical claim p.127**: "passes the 90 percent threshold when the attacker gains four times the power" — with ρ0 = 3, c = 5: Pr(4) = 0.81; 90% at ρ ≈ 4.66. With ρ0 = 2 (offense) the 10% claim at ρ=2 would fail (Pr(2) = 0.5). So text numbers are consistent only with ρ0 = 3 (approximately).
37. Stochastic combat: per-period probability of victory = Pr(local ratio)? For whom — evaluated for both sides each period (both could "win")? Unspecified. Per-period hazard vs single draw.
38. Does the stochastic criterion also replace the Schlieffen rule or the "weakest neighbor" targeting? Not stated; presumably only the threshold step is smoothed.
39. Distance d metric: "one unit's distance from the center" — Manhattan grid steps? Euclidean? path length within the state? Unstated. Switch.
40. Is τ(d) applied with nominal τ0 = 0.4 from the "two-level" section's flat tax? Yes, τ0 = 0.4 (p.129).
41. Run length: "more than one thousand periods" (p.129) but plotted times reach 3800. Seed/initial predator share not stated beyond "all states are predators".
42. Fig 5.11 calls the system "unipolar" while the largest state holds ~1/3 of cells (and others exist); polarity used informally here (vs Ch.4's count of sovereign states).

---

### 4. Theoretical claims each extension supports (testable)

**A. Power balancing (robustness of Ch.4 P3 rejection)** — "the alternative implementation of alliances should increase our confidence in the qualitative lesson already drawn. Contrary to the expectations of mainstream realist scholarship, coalition formation may in fact undermine the formation of balance-of-power systems rather than support it" (p.112).
- Test A1: with power balancing (T_min = −1), fraction of power-politics (2–10) outcomes ≤ no-alliance baseline (fig 4.11), and unipolar+>90 ≥ baseline, at each predator frequency × {2,3}.
- Test A2: monotonicity in T_min — less sensitive balancing (−1.5) → more power politics (p.111–112), yet still below no-alliance baseline.
- Test A3 (fn 5): does the alliance effect survive PRA? Cederman says probably not ("dependent on resource allocation in a nontrivial way").

**B. Strategic adaptation (Waltz socialization; Wendt "bad apples"; Jervis security dilemma)** — "Clearly, adaptation reinforces the competition mechanism by making power politics more likely" (p.116); "In conformance with neorealist expectations, threat-induced strategic adaptation reinforces the systemic invariance by boosting predatorial behavior. This effect becomes even more pronounced if strategic separation is impossible" (p.116).
- B1: in defense-dominated system, power politics share at predator frequency ≤ 0.2 increases with adaptation (weak < strong); strong ≈ one third below 20%.
- B2: dependence of power politics on initial predator share flattens ("wedge" → horizontal band).
- B3: blurring raises power politics further: weak ≥ 1/3 at low predation; strong > 1/2; strong+blurring ≈ offense-dominated fig 4.11.
- B4 (meta-claim, p.117): without security dilemma (adaptation + blurring), power politics depends on initial predator frequency in defensive systems — "undermines the theoretical power of structural realism"; pluralism "seems to hinge upon unsavory aspects of geopolitical competition."

**C. PRA (robustness of Ch.4 P2 rejection: defense dominance → unipolarity)** — "the modification introduced by PRA and localized decision criteria have little impact on the general findings about offense and defense dominance"; "power politics becomes less common as the system turns more defense oriented"; offense-dominated "approximates the structural invariance of power politics" (p.120); "the monopolistic tendencies in defense-dominated systems are not necessarily an artifact of any specific resource-mobilization process" (p.121), notable because PRA "should make hegemonic takeoffs more difficult" via multi-front entanglement.
- C1: under PRA, P(power politics | ratio 2) > P(power politics | ratio 3) across predator frequencies; unipolar share higher under ratio 3.
- C2: under PRA + ratio 2, power politics share roughly invariant to predator frequency (≥ ~0.1).

**D. Two-level action / taxation (state formation; Ardant, Liberman "conquest pays"; Putnam reverberation)** — "power politics appear to depend positively on the tax rate"; "Below a tax rate of 20 percent, power politics never occur, and below 10 percent the system does not converge to low-level polarity at all" (p.125); "the emergence of balancing is dependent on uninhibited predatory resource accumulation" (p.125); "efficient taxation may be a necessary condition of power politics" (p.126); interstate competition "depends crucially on mechanisms internal to states" (p.125).
- D1: power-politics share increasing in tax rate; zero for τ < 0.2; no convergence to ≤10 states within 1000 for τ < 0.1.
- D2: unipolarity and >10 outcomes increase as τ decreases (non-monotone mix to check).
- D3: τ = 1.0 ≈ fig 5.6 offense at 100% predators (mostly multipolar).
- D4 (implicit, p.126): deterministic secession is nearly absent — measurable (count secessions).

**E. Overextension / reverberation (Gilpin, Kennedy, Snyder; punctuated equilibrium)** — with stochastic decisions/combat and distance-decaying taxes, the system "does not necessarily settle in the long run ... geopolitical cycles of limited duration emerge ... interrupted periodically by crises, such as wars and revolutions" (p.129); "crises have a tendency to propagate from domestic to foreign policy and vice versa as deploying troops to any particular front threatens to thin out other fronts. Once set in motion, any sign of geopolitical decline has a tendency to invite further challenges, especially if protracted combat weakens the center" — a decentralizing positive feedback, the mirror of hegemonic takeoff (p.134); "the attack on the small neighbor in period 1630 was a mistake" = overextension, but only knowable in hindsight under limited foresight (p.134). Explicitly illustrative only: "a systematic exploration of overextension goes beyond the goal of the current project" (p.134).
- E1: long runs do not converge; polarity time series shows stasis punctuated by abrupt collapses (measure: distribution of regime durations, size of collapse events, fat tails).
- E2: revolts and external attacks cluster in time around a weakened center (cross-correlation of domestic and foreign war onsets per state; hazard of revolt as a function of center's active external fronts).
- E3: overextension: an unprovoked attack raises the attacker's subsequent collapse hazard (compare collapse probability after attack vs matched no-attack states).
- E4: role of each ingredient — switch off distance decay, stochastic decisions, or stochastic combat and see whether cycling persists (Cederman attributes non-settling to the probabilistic criterion + disintegration, p.129).

**Chapter-level closing caveats (p.134–135)**: prey/predator base strategies constant; "the harvest mechanism is wildly unrealistic and the initial conditions were never varied."

---------------------------------------------------------------------------------------------------

## 3. Cederman 1997, *Emergent Actors in World Politics*: Chapters 6 and 7

Source: `papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf`.
**Page offset: PDF page = book page + 15** (book 181 = PDF 196). Ch. 6 = book 136–150 (PDF 151–165); Ch. 7 = book 151–183 (PDF 166–198), Appendix book 177–183 (PDF 192–198).
The appendix equations were checked against renders of the pages, not only the OCR. The OCR is badly garbled there.

### 0. Headline: this is not an agent-based model

The Ch. 7 "Mobilization Model" is a **closed-form absorbing Markov chain**, read as population shares. It is not an ABM. There are no agents, no space, no network and no individual thresholds. It has two phases:

1. **Mobilization phase**: a 4-state constant-probability Markov chain (Deutsch 1953) over the minority population. You can read it as each individual moving independently, but the book only ever computes the deterministic expected shares S(t), N(t), C(t), U(t).
2. **Collective-action phase**: a 3-state chain {invasion, secession, status quo} with time-varying transition probabilities. The probabilities are logistic "step" functions of S(t) and N(t) from phase 1. Cederman computed it numerically as a probability product. He says explicitly that it is **not** simulated with repeated runs (fn 19, p. 170: "the figures report numerically calculated probabilities of the Markov states that only have to be computed once, rather than the outcome of repeated simulations").

The Granovetter/Kuran/Schelling threshold talk (p. 167) is only motivation for using a steep logistic. **No individual-level threshold model is implemented in Ch. 7.** The CAS (agent-based) nationalism model is Ch. 8, "Nationalist Coordination". Ch. 6 is purely conceptual.

Reproduction cost is trivial: a few dozen lines, deterministic, no RNG. My Python check (a throwaway `ced7.py`, not kept) reproduces every figure in kind and every number to about ±0.05 (see §2).

---

### 1. Chapter 6 framing (book pp. 136–150)

- **Motivation (pp. 136–138).** Neorealism (Waltz) treats states as cohesive nation-states and separates domestic and international politics. Responses to nationalism have bolted on "hypernationalism" (Mearsheimer 1990) or swapped ethnic groups in for states (Posen 1993; Fearon 1994, treating groups as unitary rational actors). Cederman rejects both and wants nationalism "as a process in its own right."
- **Taxonomy, Fig. 6.1 (p. 139): a 2×2 of "Common state?" (absent/present, horizontal) × "Common nation?" (absent/present, vertical).**
  - Case 1, anarchy: separate ethnic groups or small nation-states.
  - Case 2, multinational/imperial state: Habsburg, USSR, also a Swiss-style confederation.
  - Case 3, stateless/split nation: pre-unification Germany or Italy.
  - Case 4, nation-state: France, Japan.
  - Neorealism keeps only Cases 1 and 4 (the thick frames). It confuses 3 with 1 (missed German reunification) and 2 with 4 (missed the Soviet collapse) (p. 140).
- **Gellner's definition (p. 140).** Nationalism = "the political principle which holds that the political and the national unit should be congruent." Cases 1 and 4 are stable. Case 2 has a deficit of states and Case 3 a surplus. Culture changes slowly (Smith 1986), so nationalism produces mainly *horizontal* shifts: empires secede (2→1) and fragmented nations unify (3→4). So nationalism produces disintegration as readily as integration.
- **Three types, Fig. 6.2 (p. 141), after Schieder (1991) and Alter (1989):**
  - **State-initiated** (Western: England, France, Spain, Sweden). Conquest gives 1→2, then the center's slow cultural homogenization gives 2→4, core to periphery, over centuries (Tilly 1990; Weber 1976) (p. 142).
  - **Unification** (Central/Southern: Germany, Italy). Nation before state, 1→3. Napoleon spreads the "nationalist germ." Then a core state (Prussia/Bismarck, Piedmont/Cavour) drives 3→4 (pp. 142–143). Fn: "We have made Italy, now we have to make Italians."
  - **Separatist** (Eastern: Habsburg, Ottoman, Russian collapse). Peripheral nationalities secede, 2→1, sometimes followed by 3→4. **Irredentism** = 2→1 then 3→4 (Sudeten Germans; Serbia and Croatia) (pp. 143–144).
  - Reversals are possible: Reich → two Germanies + Austria (4→3); Brittany/Scotland fringe nationalism (4→2) (p. 144).
  - Outside Europe (pp. 145–146): anticolonial movements are mostly separatist (Wilsonian self-determination). Settler states and Zionism (and Palestinians, Kurds) resemble unification or "diaspora" nationalism. Most postcolonial states are multiethnic with aborted state-initiated nation-building (Mayall 1990).
- **Mobilization and coordination (pp. 146–150), Fig. 6.3 "Secession or assimilation?"**
  - Starting from Case 2 (center dominating a periphery), the outcome is either assimilation (2→4) or secession (2→1).
  - Successful secession needs two conditions (p. 148):
    - **mobilization**: the periphery becomes politically conscious, which requires literacy and modernization;
    - **coordination**: the periphery is not culturally fragmented. The "tribal underbrush" has to be cleared, possibly by myth-making (Renan).
  - Breuilly (1982) adds a third condition, external legitimacy (fn 11).
  - **Ch. 7 models mobilization with a single periphery. Ch. 8 models coordination** in a two-dimensional cultural space.

---

### 2. Chapter 7 model specification (book pp. 151–183)

#### 2.1 Setting (pp. 152–154)
- One center and one ethno-culturally distinct periphery, treated as a **single unit** (p. 153; relaxed in Ch. 8).
- Two Deutschian dichotomies: mobilized/unmobilized × assimilated/differentiated.
- "Assimilation" means *political* assimilation: barriers to political communication are removed and identity shifts to the center's supranational platform (p. 154).

#### 2.2 Phase 1: mobilization chain (Fig. 7.1 p. 155; Appendix pp. 177–181)

**States:**

| | unmobilized | mobilized |
|---|---|---|
| assimilated | C, nonethnic class | S, supranational identity (absorbing) |
| differentiated | U, unmobilized ethnic category | N, nationality |

**Parameters:**
- m: probability of mobilization ("wakes up") per period.
- a: probability of direct assimilation given mobilization from U. 1−a is "provocation."
- b: probability of indirect assimilation N→S per period. 1−b is "nationalist persistence."

**Transitions, verbatim matrix (p. 177).** M_ij is the probability of moving from i to j, with order (S, N, C, U):
```
      [ 1     0        0     0   ]
M =   [ b     1-b      0     0   ]
      [ m     0        1-m   0   ]
      [ ma    m(1-a)   0     1-m ]
```
- p^(t) = p^(0) M^t, with p^(0) = [S0, N0, C0, U0] summing to 1.
- U→S happens in one step with probability m·a. U→N happens with probability m(1−a). U never goes to C.
- C is inert under Assumption 1 ("for our analytical purpose less relevant").
- "Each individual member's shift of loyalties" (p. 155) suggests individual-level application, but only expected shares are ever computed.

**Assumption 1 (p. 179):** S0 = 0, N0 = 0, C0 = 0, U0 = 1.

**Proposition 1 (pp. 177–178), as printed, for m ≠ b:**
```
S(t) = 1 − (N0 + c1 U0)(1 − b)^t − (C0 − c2 U0)(1 − m)^t
N(t) = (N0 + c2 U0)(1 − b)^t − c2 (1 − m)^t
C(t) = C0 (1 − m)^t
U(t) = U0 (1 − m)^t
c1 = (b − ma)/(m − b),   c2 = m(1 − a)/(m − b)
```
For m = b, as printed:
```
S(t) = 1 − (C0 + N0 + U0{1 − m(1−a)/(1−m) · t})(1 − m)^t
N(t) = {N0 − U0 · m(1−a)/(1−m) · t}(1 − m)^t
```

**Corollary 1 (p. 179), as printed:** S(t) = 1 − c1(1−b)^t + c2(1−m)^t; N(t) = c2{(1−b)^t − (1−m)^t}; C(t) = 0; U(t) = (1−m)^t.

**Corollary 2:** if m > 0 and b > 0, then S→1 and N, C, U→0.

**Proposition 2 (p. 179), "overshooting":** under Assumption 1, there exists t ≥ 1 with N(t) > S(t) **iff a < 1/2**. This is independent of m and b. The proof uses S(1) = ma and N(1) = m(1−a).

**Proposition 3 (p. 181):** N(t) peaks at
`t_max = ln{ln(1−b)/ln(1−m)} / {ln(1−m) − ln(1−b)}`.
Worked example: m = 0.1, b = 0.02 gives t_max = 19.4 and N(19) = 0.55.

**Typos found and verified algebraically and numerically.** Implement from M, not from the printed closed forms:
1. **Corollary 1, Proposition 1's S(t), and the bottom-left entry of M^t (p. 178) have c1 and c2 swapped.** As printed, S(0) = 1 − c1 + c2 = 2. The correct form is S(t) = 1 − c2(1−b)^t + c1(1−m)^t, or in general S(t) = 1 − (N0 + c2U0)(1−b)^t − (C0 − c1U0)(1−m)^t. The proof of Prop. 2 on p. 180 uses the correct form, with (1−a)m/(m−b) on (1−b) and (b−ma)/(m−b) on (1−m).
2. **The m = b case has a sign error.** It should read N(t) = {N0 + U0·m(1−a)/(1−m)·t}(1−m)^t, and S correspondingly 1 − (C0+N0+U0{1 + m(1−a)t/(1−m)})(1−m)^t. As printed, N goes negative from N0 = 0.
3. **N(1) on p. 180 has denominator "m − a"**, which should be m − b.
4. **The m = b limit on p. 179 is written "lim_{k→w}"**, which should be the limit as m→b.

#### 2.3 Phase 2: collective-action chain (Fig. 7.5 p. 166; Appendix pp. 181–183)

**States:** invasion (absorbing), secession (absorbing), status quo (transient). x^(0) = [0, 0, 1] is implied. The book says "the process starts in the first state," meaning status quo (first in the figure, though third in the vector).

The state vector evolves as x^(t) = x^(0) ∏_{τ=1}^{t} P^(τ), with order [invasion, secession, survival]:
```
        [ 1          0          0                     ]
P(t) =  [ 0          1          0                     ]
        [ p_inv(t)   p_sec(t)   1 − p_inv(t) − p_sec(t)]
```

Transition probabilities, verbatim (p. 182):
```
p_inv(t) = q_inv / (1 + (F / S(t))^(−c_inv))
p_sec(t) = q_sec / (1 + (N(t) / G)^(−c_sec))
```
Equivalently, p_inv = q_inv/(1 + (S/F)^c), which is near q_inv when S ≪ F and drops when S > F. And p_sec = q_sec·(N/G)^c/(1 + (N/G)^c).

**Interpretation:**
- F = exogenous external force.
- G = the government's resources.
- The c values set the slope of the step.
- The q values are the maximum per-step probabilities.

**Parameter values (p. 182), "in the particular runs reported in figures 7.6 and 7.7":**
- F = G = 0.5
- c_inv = c_sec = 10
- q_inv = 0.005, q_sec = 0.05

The survival probability is P(t) = ∏_{τ=1}^{t} {1 − p_inv(τ) − p_sec(τ)}. The long-run P = lim P(t) "converges" (stated from the figure, not proved) (p. 183).

**Default parameter table:**

| param | meaning | values used | page |
|---|---|---|---|
| m | mobilization rate | 0.1 (Figs 7.2–7.4, 7.6); swept 0.001–1 on a log axis (7.7) | 158, 161, 168, 169 |
| a | direct assimilation | 0.8 (assim., delayed); 0.2 (provocation) | 158, 161, 168 |
| b | indirect assimilation | 0.2 (assim.); 0.02 (delayed, provocation) | 158, 161, 168 |
| S0,N0,C0,U0 | initial shares | 0,0,0,1 | 179 |
| F | external threat level | 0.5 | 182 |
| G | center's resources | 0.5 | 182 |
| c_inv, c_sec | logistic steepness | 10, 10 | 182 |
| q_inv, q_sec | max per-step hazards | 0.005, 0.05 | 182 |

**Update order and random elements.** Deterministic, with no RNG. The only open choice is whether step τ's hazard uses S(τ),N(τ) (after τ mobilization steps; the literal reading of ∏_{τ=1}) or S(τ−1). That choice changes results by less than 0.005.

#### 2.4 Table 7.1 (p. 157): "A Comparison of Three Basic Hypotheses" (after Horowitz 1985, 96–97)

| | Assimilation | Delayed Assimilation | Provocation |
|---|---|---|---|
| Mobilization m | high | high | high |
| Direct assimilation a | high | high | low |
| Indirect assimilation b | high | low | low |

Numerical stand-ins: high m = 0.1, high a = 0.8, low a = 0.2, high b = 0.2, low b = 0.02.

---

### 3. Experiments and claimed results, with my reproduction check

All curves come from the closed forms. There is no replication count: each is computed once, analytically or numerically.

| Fig | page | params | claim in text | my computation |
|---|---|---|---|---|
| 7.1 | 155 | — | flow diagram | — |
| 7.2 assimilation | 159 | m=.1 a=.8 b=.2, t=0–100 | U falls; S surges; "within fifty time periods… almost the entire population"; N "no more than 5 percent" at peak, then declines fast | S(50)=0.994; **N peak 0.054 at t=6**, so slightly over the stated 5% |
| 7.3 delayed | 161 | m=.1 a=.8 b=.02 | N "rises above 10 percent and persists much longer" | N peak 0.137 at t=19; N(50)=0.090, N(100)=0.033 ✓ |
| 7.4 provocation | 163 | m=.1 a=.2 b=.02 (from p.168 and the p.181 example) | nationalists initially outpace assimilation, but S eventually wins | N peak 0.546 at t=19 ✓ (book: N(19)=0.55); N>S for t=1..32 |
| 7.5 | 166 | — | collective-action flow diagram | — |
| 7.6 | 168 | provocation + F=G=.5, c=10, q=.005/.05, t=0–100 | survival falls dramatically, stabilizes after ~50 steps; long run: invasion "in a tenth," secession "more than half," survival "about 35 percent" | t=50: inv 0.123, sec 0.533, sq 0.344; limit: **inv 0.133, sec 0.536, sq 0.332** ✓ (roughly; invasion is nearer 13%) |
| 7.7a | 169 | delayed (a=.8, b=.02), m on log axis 0.001–1 | invasion risk falls steadily with m; no secession; the center should speed mobilization | m=.001: sq 0.024; .01: 0.645; .1: 0.932; 1: 0.975; secession ≈ 0 everywhere ✓ (N never nears G=0.5) |
| 7.7b | 169 | provocation (a=.2, b=.02), same axis | secession appears at about m=0.03 and dominates from m=0.1; **optimum survival at about m=0.04**; survival curvilinear | secession 0.005 at m=.018, 0.057 at .032, 0.28 at .056, 0.54 at .1, 0.67 at 1; **sq max 0.689 at m≈0.0275** (sq at m=0.04 is 0.641); sq 0.25 at m=1 |

Fn 18 (p. 169): Fig. 7.6's limit equals Fig. 7.7b at m=0.1 ✓.

**Notes on the Fig. 7.7 sweep:**
- The long run needs long horizons at small m: at m=0.001, the status quo is 0.61 at T=100 and 0.024 at T=5000. The figure must be using a long horizon or the limit. I used T=5000.
- Fn 15 (p. 165), numerical claim: N's maximum comes earlier and is higher as m rises. ✓ For a=.2, b=.02: m=.01 peaks at 0.20 at t=69; m=.1 at 0.55 at t=19; m=.9 at 0.78 at t=2.

**Prop. 2 check.** The overshooting-iff-a<1/2 claim holds on a grid of m and b from 0.01 to 0.99, with a = .3–.9 and 400 steps (0 violations).

**Other textual claims:**
- "What is less intuitive… overshooting does not vary with the rate of mobilization m" (p. 164).
- The policy implication is a narrow band: too slow invites invasion, too fast provokes secession (pp. 169–170, 175).

**What to digitize.** Nothing is strictly needed: every figure is a deterministic function of stated parameters. Optionally, digitize the area boundaries of Fig. 7.7a/b (status-quo and secession edges at the decade ticks) and Fig. 7.6 at t = 20, 40, 60, so we can pin the horizon and timing convention and test the "optimum ≈ 0.04" reading. My visual read of 7.7b at m=1 is sq ≈ 25% and secession ≈ 70%, which matches.

#### Divergences to report (per "findings are the point")
1. **The optimum mobilization rate.** The text says about 0.04. The stated equations give about 0.028 (survival 0.69 against 0.64 at 0.04). This is minor and possibly a coarse sweep grid or a reading off a log axis.
2. **Fig. 7.2's peak N** is 5.4%, against the text's "no more than 5 percent."
3. **Fig. 7.6's invasion share** is 13%, against "a tenth."
4. **The p. 163 text** says Fig. 7.4 differs from 7.3 by "a reduction of direct assimilation from 20 percent to 2 percent." That is wrong. Figs 7.3 and 7.4 actually use a = 0.8 → 0.2 (confirmed by p. 168, "m = 0.1, a = 0.2, b = 0.02", and by N(19) = 0.55, which needs a = 0.2; a = 0.02 would give a peak of 0.67). The literal text should be a named switch or just noted. 0.02 is b, not a.
5. **Closed-form typos** (c1/c2 swap; the m=b sign; "m − a"), listed in §2.2. Use the matrix.
6. **Labels.** The status quo is called both "survival" and "status quo." The vector order [inv, sec, sq] differs from the figure's order.

---

### 4. Historical illustrations (pp. 170–174) and what they are claimed to show

- **Habsburg monarchy (pp. 170–171).** After 1848 and the defeats of the 1860s (Ausgleich 1867), the empire deliberately reduced central authority and avoided mobilization (McCagg 1991). Low m prevented secession but left the army inferior in WWI. It fell to "invasion," i.e., losing the war (Sked 1989). Placement: **m well left of the optimum on Fig. 7.7b.**
- **Soviet Union (pp. 171–173).** Mobilized mainly against external threats. Gorbachev's perestroika and glasnost were "massive and speedy political mobilization" (Dawisha & Parrott 1994). Expecting assimilation (an assimilation-theory bias), they provoked nationalism: Central Asia, the Caucasus, the Baltic secession claims. Placement: **provocation world, m right of the optimum, giving secession.**
- **Fn 20 (p. 173), an informal threshold sketch:** "the center-periphery balance is the ratio between the resources R controlled by the center and those controlled by the regional elite Nr, where N is the number of nationalist adherents and r the power of each individual… revolutionary collective action becomes possible if the ratio of forces exceeds some threshold ρ—i.e., if Nr/R > ρ." The totalitarian state has r/R ≪ ρ, which reverses for large N. The decline of R mattered too (fn 25 maps this to a decrease in G).
  - The OCR renders ρ as "r". The image shows "> r." in the inequality but ρ in the text, so this is probably a print typo for ρ.
- **European Union (pp. 173–174).** Delors's Single European Act and Maastricht push, a response to "Eurosclerosis" and to Japanese and US competition, provoked nationalist backlash: the Danish no (1992), the French "petit oui," opt-outs. This was a non-fatal provocation.
- **Conclusion (pp. 174–177).** Mobilization must be "carefully tuned" (Motyl 1991); "use both the gas and the brake"; integration may produce disintegration (Wæver & Kelstrup 1993).
- **Self-stated limitations:**
  - a single cultural cleavage;
  - exogenous parameters with no feedback (fn 24: "make the direct assimilation a a decreasing function of the already acquired strength of the nationalist movement N", citing Kuran 1991 multiplier effects);
  - internal and external threats that do not interact.

These are good candidate switches.

---

### 5. Ambiguities and unstated details

1. **Individual vs. aggregate (p. 155 vs. Appendix).** The text says probabilities govern "each individual member's shift of loyalties." Only expected shares are computed. An agent-based stochastic version with a finite population is a variation that would add variance. Its name and population size are unstated.
2. **Hazard timing.** Whether τ uses S(τ) or S(τ−1) is unstated. The difference is under 0.005.
3. **Horizon for the "long-run" Fig. 7.7.** Unstated. It matters a lot at m ≤ 0.003. Fig. 7.6 is shown to t=100.
4. **The m grid for Fig. 7.7** is unstated. The optimum of about 0.04 against my 0.028 may come from the grid.
5. **The logistic form** is stated with a negative exponent. That is fine, but note p_inv(0) = q_inv when S = 0 (0^−c is ∞ in the denominator's power term; handle it as the limit).
6. **"Beyond a certain step F"** (p. 182). F and G are on the same scale as population shares, so they mean "half the minority," but the minority's size relative to the whole state is never modeled. The substantive meaning of F = G = 0.5 is unstated.
7. **q values are "maximum probability per time step".** The time unit is unspecified (years?).
8. **"High" and "low" in Table 7.1** are given numbers only implicitly through the figures. Is m = 0.1 "high," and is 0.03 the secession onset?
9. **Fig. 7.4's direct-assimilation value.** The text on p. 163 contradicts p. 168 (see §3, item 4).
10. **The invasion threshold.** The chapter references Ch. 4's 3:1 or 2:1 superiority ratio (p. 168), but Ch. 7 doesn't use it. F is a free constant.
11. **C (nonethnic class)** is defined but plays no role under Assumption 1.
12. **Prop. 1 assumes 0 < m, a, b < 1.** Behavior at a = 0 or 1 isn't treated, but the matrix handles it.

---

### 6. Links to known models

| Cited model | Where | How it is used | Overlap with our repo |
|---|---|---|---|
| Deutsch 1953, 1969 (mobilization vs. assimilation) | pp. 152–155, 164 | **The actual core**: the 2×2 Deutsch categories and the "assimilation must keep abreast of mobilization" criterion. Deutsch's crude indicator, the share of mobilized-but-differentiated, is max N(t) (Prop. 3) | New. Not in the repo. |
| Horowitz 1985 (three theories) | pp. 156–157 | Table 7.1 taxonomy | — |
| Granovetter 1978; Schelling 1978 | p. 167 | Motivation only: "secessionist chain reactions," critical point, self-reinforcing. **Not implemented**; replaced by a steep logistic in N with c = 10 | We have both. One could swap p_sec's logistic for an explicit Granovetter cascade among the N population: a named variation, not the default. |
| Kuran 1989, 1991 (preference falsification, revolutionary thresholds) | p. 167; fn 24 p. 176 | Same as above. Fn 24 suggests making a decrease with N (positive feedback), a Kuran-style multiplier | Not reproduced. A candidate variation would be `a(N)`. |
| Lohmann 1994; Macy 1990, 1991; Goertz 1994 | p. 167 | cited for threshold collective action | Macy, if in the queue |
| Hardin 1982 ("step good") | fn 17 p. 167 | justifies the step-shaped benefit | — |
| Olson / Tilly 1978 / Taylor 1988 | p. 167 | collective-action dilemma framing | — |
| Axelrod 1997 culture | not cited in Ch. 6–7 | Ch. 8's "two-dimension cultural space" is the CAS follow-up; check Ch. 8 for Axelrod links | We have Axelrod culture, which is probably relevant to Ch. 8, not Ch. 7. |
| Hegselmann–Krause, Watts 2002 | not cited | — | No direct overlap |
| Emergent Polarity Model (Ch. 4–5) | pp. 166, 168, 177 | the collective-action phase is "an abstraction of the more involved two-level design" of extended EPM; invasion threshold analog | Same book, other sections |
| Gellner 1983; Schieder 1991; Alter 1989 | Ch. 6 | definitions and typology | — |

**Bottom line for implementation.** Ch. 7 is a small deterministic Markov model: a good, cheap "preset" with analytic checks (Props 2 and 3, Cor. 2). Its natural ABM variations are:
- a finite-agent stochastic version;
- a Granovetter/Kuran threshold cascade replacing the logistic p_sec;
- a(N) feedback (fn 24);
- G declining over time (fn 25).

The literal default is fully specified apart from the horizon and timing.

---------------------------------------------------------------------------------------------------

## 4. Cederman (1997), *Emergent Actors in World Politics*: Ch. 8 "Nationalist Coordination" and Ch. 9 "Conclusions"

Source: `papers/geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf`.
**Page offset: PDF page = book page + 15** (book p.184 = PDF 199; book p.213 = PDF 228).
Ch. 8 = book pp.184–212 (PDF 199–227). Ch. 9 = book pp.213–231 (PDF 228–246). Bibliography starts p.233.
The book calls it the **"Coordination Model"** (p.192). Chapter 8 was first published as Cederman (1995), "Competing Identities: An Ecological Model of Nationality Formation," *European Journal of International Relations* 1:331–65 (Preface, p.xiv). **We don't have that article. It may give more spec detail, so it's worth fetching.** Theoretical source: Hannan (1979), "The Dynamics of Ethnic Boundaries in Modern States" (in Meyer & Hannan, eds., *National Development and the World System*).

I checked every equation, Table 8.1 and Figs 8.1–8.7 against rendered page images, because the OCR is garbled for all of them.

---

### 1. Model specification as stated

#### 1.1 Lineage and simplifications
- "The Coordination Model can be seen as an extension of the two-level version of the [Emergent Polarity] Model. In addition to using the collective-action mechanism, the current extension enables regions to coordinate their opposition to the center by forging cultural alliances. ... the Coordination Model operates with simplified combat rules and without a harvest mechanism." (pp.191–92)
- Ch. 7's Mobilization Model differs in two ways (fn 8, p.188): (a) here communities are always conscious of their culture; in Ch. 7 they started with the wild-card identity "?"; (b) "**assimilation is ruled out by assumption**. Thus, while transcommunal identities are free to vary, the cultural traits of the communities remain constant."
- Rule 2's S-curve reuses the logistic from Ch. 5's two-level extension (fn 17, p.195; Fig. 5.9, p.127). There it is Pr = 1/(1+(ρ/ρ0)^−c), with no scale q.

#### 1.2 Agents and identities
- **Communal actors (provinces / "primitive units" / "social atoms")** are small, culturally homogeneous communities (p.188). Each has an identity string of **two dichotomous traits** (00, 01, 10, 11). "Reflecting Barth's formalism, it is immaterial if these traits stand for religion, language, or other cultural attributes" (p.188). The traits are fixed for the whole run (fn 8).
- **Transcommunal identities (movements / "social molecules")** are trait vectors that may contain wild cards "?" meaning indifference. With two traits there are 8: "00, 01, 10, 11, ?0, ?1, 0?, 1? (not counting ??)" (p.188). "It is helpful to think of a transcommunal identity as the program of a political movement" (p.188).
- **Compatibility:** "a primitive community can only share a transcommunal identity to the extent that it does not violate its own identity, although it does not have to match it fully. For example, an atomic actor of type 00 can adopt the transcommunal identities 00, ?0, and 0? only, whereas one of type 01 is restricted to the identities 01, 0?, and ?1" (pp.188–89).
- **Population used:** the periphery consists only of 00 and 01 types. **The center has identity 1?** (p.189). The first trait is the ethnic marker that separates the center from the periphery ("a one denotes the status of foreign ruler"). The second trait is a "tribal" idiosyncrasy, and the center's "?" there means indifference to it.
- The five relevant movement types are therefore 00, ?0, 0?, ?1 and 01. 0? is the only shared, inclusive one.

#### 1.3 Fit index (Table 8.1, p.192), quoted exactly
"The fit is defined as the difference between the number of corresponding traits minus the number of clashing ones, not counting the indifferent traits of the transcommunal identity (i.e., those with wild cards)." (p.192)

TABLE 8.1 "The Fit of Transcommunal Identities" (read from the image; the OCR drops cells):

| Communal \ Transcommunal | 00 | ?0 | 0? | ?1 | 01 |
|---|---|---|---|---|---|
| 00 | 2 | 1 | 1 | −1 | 0 |
| 01 | 0 | −1 | 1 | 1 | 2 |

"The decision to join a national movement presupposes that the fit is positive. In other words, the 00 regions cannot adhere to the ?1 and 01 transcommunal identities since they both violate the second trait. By the same token, the 01 actors reject the 00 and ?0 identities." (p.192). So eligibility is fit > 0, and fit 0 (00 vs 01) also excludes a region.

#### 1.4 Space (Fig. 8.1, p.189)
- A **5×5 grid** with the center (1?) in the middle cell, giving **24 provinces**. For the equal split there are **12 of type 00 and 12 of type 01**.
- Fig. 8.1 layout, row by row from the top (center marked C):
  - `00 00 01 00 01`
  - `00 00 01 01 01`
  - `01 00  C 01 01`
  - `00 00 01 01 01`
  - `00 00 00 01 00`
- "Although the system is explicitly territorial, **the location of each region is immaterial** for the purposes of the simulations" (p.189). Fn 9 says geography matters only after independence; runs stop at the revolution.
- "it is assumed that the center is strong enough to eliminate any interregional conflicts. Thus, each subordinate province interacts directly with the center only." (p.190)
- For Fig. 8.7 there are "twice as many 00 types as 01 types", i.e. **16 × 00 and 8 × 01** (p.201: "two-thirds of the population, or sixteen regions, are of type 00"). The spatial arrangement for this case is not given, and it doesn't matter.

#### 1.5 Resources and the revolutionary threshold (Fig. 8.2, p.193)
- Each region controls **r = 1** resource unit. The center controls **R**, which is the independent variable. A movement with N members mobilizes **Nr**.
- "it is assumed that the nationalist opposition needs to attain the **same resource level** as the center if it wants to challenge the center's hegemony" (p.193; fn 14 calls this "an arbitrary assumption that can easily be changed depending on which side is presumed to possess an offensive advantage").
- Example: R = 12 means a coalition of at least twelve regions is needed; R = 24 needs all 24 (p.193). At R = 8, "a nationalist movement only needs eight provinces, or a third" (p.197). So the threshold is **Nr ≥ R** (a tie wins).
- The x-axis "resource balance" in Figs 8.6–8.7 is R/r = R, running 1..20.

#### 1.6 Dynamics: the three rules, quoted
Per-period overview (p.194): "**In each time period, each communal actor decides on its transcommunal identity. If it does not already belong to a movement, it may decide to create a new one (Rule 1) or to join an existing one (Rule 2). At the same time, the center scans through all its provinces checking whether any of its subjects is disloyal. Should this be the case, it randomly singles out one such region, and decides if enforcement should be enacted (Rule 3).** These three rules fully define the dynamics of the system."

**Rule 1, creation (dissidents), pp.194–95**
- "The probability that such an actor creates a nationalist movement is set to a low, fixed probability, in our case **0.01 per period and unit**." It is a flat line in Fig. 8.3 and does not depend on power.
- The platform is "selecting randomly from the transcommunal identities that do not violate the dissident's communal identity (cf. table 8.1). The dissidents attach an equal probability to each of the three possible transcommunal identities" (00 → {00, 0?, ?0} each with probability 1/3; 01 → {01, 0?, ?1}).
- Implied creation mix, with p = the fraction of 01 types: "Pr(01) = Pr(?1) = p/3 and Pr(00) = Pr(?0) = (1 − p)/3 and of the shared transcommunal identity Pr(0?) = p/3 + (1 − p)/3 = 1/3." For p = 1/2: 1/6, 1/6, 1/6, 1/6 and Pr(0?) = 1/3, so "0? is twice as likely as any of the other identities" (p.195).
- Fn 15: "The process of cultural innovation may actually produce a transcommunal identity that already exists. In this case, **the effect is the same as joining the already existing movement**, although the motive must be idealistic rather than opportunistic." So **there is at most one movement per identity string.** This is contradicted by Fig. 8.4's narrative; see §3.

**Rule 2, joining (opportunistic masses), pp.195–96**
- "The regional actor scans through all available movement[s] and judges whether it would be safe to join."
- Equation (p.195), exact: **ρ = R / ((N + 1) r)**, where N is the current membership of the movement. Fn 17: "The peripheral unit bases its power comparison on the hypothetical case, presuming that it has already joined the movement (i.e., the denominator contains the factor N + 1 and not just N). This is not a mere technicality because it generates the type of 'bandwagoning' self-feeding process."
- Equation (p.196), exact: **Pr(Join | ρ) = q / (1 + (ρ/ρ0)^c)**
- "where ρ0 stands for the resource balance that makes the fall in probability the steepest. I set the value to **one** for all the simulation runs below. The parameter c controls the overall slope ... The numerical value **c = 3.17** was calibrated to make the probability of joining 0.01 for a resource balance of two. Finally, the probability q scales the entire function ... The value used in the simulations was **0.1**. This means that the probability of considering a movement safe never exceeds this value." (p.196)
- Checks (I verified the arithmetic): ρ = 1 gives 0.05 ("drops below 0.05" for ρ > 1); ρ = 2 gives 0.1/(1 + 2^3.17) = 0.0100; ρ = 8 gives 0.1/(1 + 8^3.17 ≈ 729) = 0.000137, matching the text's "(0.000137)".
- "Should there be more than one 'safe' movement, the peripheral actors pick the one with the **highest identity fit**. For the parameter values chosen here, however, the bias in favor of better fit plays a subordinate role since the likelihood of finding any safe movement is quite low." (p.196)
- Only regions with fit > 0 can join (p.192).
- No leaving: "The present version of the model excludes the possibility of switching from one identity to another as soon as a communal actor commits itself to a movement" (p.209).

**Rule 3, collapse (center enforcement), pp.196–97**
- Equation (p.197), exact: **Pr(Enforce | ρ) = q / (1 + (ρ/ρ0)^(−c))**, "where **ρ = R/(Nr)**, that is the power ratio in the center's favor. As in the periphery's function, we use the same parameter values: **q = 1, ρ0 = 1, and c = 3.17**." (Note that this ρ has **no +1**.)
- "In each time period, the center checks whether any of the provinces belongs to a nationalist organization. Should this be the case, it applies the probabilistic criterion to decide whether repression is necessary. Since the rule depends on the power balance, large nationalities will be persecuted less often than small and defenseless dissident cells." (p.197). Fn 18 defends this slope over the alternative of targeting big movements first.
- "In the event of repression, **all members of the same nationality as the region under attack respond** to the center's challenge. As long as the center is more powerful than the group in question, the outcome is a quick victory for the government. Police repression **always results in the collapse of the oppositional movement as long as the latter is in an inferior position**. If the periphery manages to gather superior resources, however, a nationalist revolution takes place that ends the simulation run. **The strongest movement is recorded as the 'winner'** and the simulation ends." (p.197)
- Earlier wording (p.191): "each attempt to repress succeeds in eliminating the campaign completely as long as it does not possess resources comparable to those of the center."

#### 1.7 Parameter table

| Parameter | Value | Page |
|---|---|---|
| Grid | 5×5, center at the middle cell, 24 provinces | 189 (Fig. 8.1) |
| Traits | 2 binary; periphery 00/01; center 1? | 188–89 |
| Population split | 12/12 (Figs 8.4–8.6); 16 × 00 / 8 × 01 (Fig. 8.7) | 189, 200–201 |
| r (province resources) | 1 | 193 |
| R (center resources) | swept 1..20; single runs at 8 and 12 | 197–200 |
| Revolutionary threshold | Nr ≥ R ("same resource level") | 193, 197 |
| Creation probability | 0.01 per period per unaffiliated unit | 194 |
| Platform choice | uniform over the 3 compatible identities | 194–95 |
| Join: q, ρ0, c | 0.1, 1, 3.17; ρ = R/((N+1)r) | 195–96 |
| Enforce: q, ρ0, c | **1** (text) or 0.1 (Fig. 8.3), 1, 3.17; ρ = R/(Nr) | 197; Fig. 8.3 p.194 |
| Join tie-break among safe movements | highest fit | 196 |
| Replications per R | 20, different seeds | 199 |
| Run cap | 10,000 periods, then "no secession" | 199 |
| Initial state | no movements ("As the simulation starts, there is no nationalist activity") | 198 |

#### 1.8 Random elements
Creation draw per unaffiliated unit per period; platform choice (uniform over 3); per-movement safety draws in Rule 2 (the exact draw structure is unstated); the center's uniform choice of one disloyal region; the enforcement draw. "Each replication started with a different random seed generating a distinct series of probabilistic decisions" (p.199).

#### 1.9 Interpretive framing (for captions)
- Variation and selection follow Hroch's (1985) phases: A (scholarly interest) is variation; B (patriotic agitation) and C (mass movement) are selection (p.190). Rule 1 and Rule 3 supply variation; Rule 3 also selects; "Rule 2 amplifies the selective logic of rule 3 by favoring stronger movements" (p.197).
- Dissidents are like Calhoun's "Bravery to the point of apparent foolishness"; the masses are rational and power-sensitive (p.191). This is presented as a middle course between rational choice (Hechter, Meadwell) and emotional accounts (Smith, Connor).
- Bandwagoning links to Granovetter 1978 and Kuran 1989/1991 thresholds (p.196), Macy 1990/91 critical mass (fn 11, p.191), Hardin's "lumpy good" (fn 17), and Therborn 1991.
- **Headline hypothesis (p.185): "the more powerful the center compared to the periphery, the more inclusive the emergent identity of the new political unit."**

---

### 2. Experiments and claimed results

#### Fig. 8.1 (p.189): 5×5 imperial system
Initial layout only (see §1.4).

#### Fig. 8.2 (p.193): power balance diagram
A threshold line Nr = R. Point A is a single region at R = 12; point B is a 12-region coalition. Conceptual, nothing to digitize. (The text says "despite possessing only half of the opposition's total power (i.e., Nr = 24, see point A)". Point A actually sits at Nr = 1, so the parenthetical is sloppy.)

#### Fig. 8.3 (p.194): probability curves against ρ ∈ [0, 8]
- Flat creation line at 0.01.
- Join curve starts at 0.1 at ρ = 0 and falls.
- Enforcement curve rises from 0 and **saturates at 0.1**.
- A vertical dotted line at ρ = 1 is labeled "nationalist revolution".
- This is a useful check plot to regenerate. **The plotted enforcement ceiling of 0.1 contradicts the text's q = 1** (see §3).

#### Fig. 8.4 (p.198): one run, R = 8 (an "8:1 power balance"), equal split
- y = movement strength (members, 0–24); x = time 0–350.
- Narrative (p.198):
  - At ~t5, "two peripheral actors independently decide to launch a campaign of type 00".
  - One 00 crumbles and is replaced by ?1 and ?0 (short-lived), then attempts by 01 and 0?.
  - At t125 a ?1 movement grows. At ~t175, after reaching six members, it is repressed.
  - Next, a campaign reaches four members and vanishes.
  - Around t200 several platforms contend. **?1 wins: it is the first to reach 8 members, at ~t250–260** (read from the figure).
- Single trajectory. Reproduce qualitatively only: repeated small movements, repression, then a takeoff.

#### Fig. 8.5 (p.199): one run, R = 12, same seed as Fig. 8.4
- "After starting off with exactly the same conditions as in the previous run, the new simulation starts to differ after time 150."
- 0? dominates. One 00 attempt reaches **11 members** and nearly wins (12 are needed). 0? wins "on its fifth attempt" with **12 members**, at ~t410–420 (pp.198–99).
- The x-axis tick labels read 100, 150, ..., 400 starting at the origin, which looks like a labeling oddity (see §3).

#### Fig. 8.6 (p.200): main result, 20 replications per R = 1..20, equal split
- A stacked-area chart of winner identity: 0? at the bottom, then ?0, ?1, 00, 01, with "no secession" on top.
- Claims (p.200):
  - At R = 1 (= r, so ρ = 1), "a revolution takes place as soon as a new movement crops up", so the shares should match the creation mix: 1/3 for 0? and 1/6 for each other type. "This seems to be the case."
  - As R rises, 0? becomes "increasingly more successful at the expense of the more parochial movements. This trend culminates at power balance 12, **beyond which it becomes impossible for the other movements to reach the revolutionary threshold**" (only 12 regions are compatible with each non-0? identity).
  - After that point 0? dominates completely, "but their effectiveness falls sharply"; at R = 16 "collective action becomes practically impossible, at least within the time limit."
  - "**Asymmetry in power apparently breeds homogeneity in political culture.**"
- **Digitization (my eyeball reading at integer R off a gridded overlay, ±~1 replication; redo with a proper digitizer before using as golden targets).** Cumulative stack tops (0? | +?0 | +?1 | +00 | total):

| R | 0? | ≤?0 | ≤?1 | ≤00 | total |
|---|---|---|---|---|---|
| 1 | 7.5 | 10 | 13.4 | 18.3 | 20 |
| 2 | 8 | 10.5 | 13 | 17.6 | 20 |
| 3 | 9.5 | 12.5 | 14.4 | 17.3 | 20 |
| 4 | 9 | 12.4 | 14.4 | 18 | 20 |
| 5 | 10 | 11 | 15 | 19 | 20 |
| 6 | 6.5 | 10.5 | 13 | 17 | 20 |
| 7 | 12 | 13 | 18 | 19 | 20 |
| 8 | 10 | 11 | 16 | 19 | 20 |
| 9 | 8 | 9 | 13 | 16 | 20 |
| 10 | 11.5 | 12.5 | 16 | 18 | 20 |
| 11 | 14.5 | 16 | 19 | 19 | 20 |
| 12 | 16 | 17 | 19 | 19 | 20 |
| 13 | 18.5 | – | – | – | 19 |
| 14 | 9 | | | | 9 |
| 15 | 7 | | | | 7 |
| 16 | ~2 | | | | ~2 |
| 17–20 | 0 | | | | 0 |

- Testable shape: the 0? share trends up over R = 1..12 (noisy, n = 20); non-0? winners are at or near zero for R ≥ 13; seceded runs are about 19–20/20 up to R = 13, then collapse over 14–16 and reach zero by 17.

#### Fig. 8.7 (p.201): same sweep, 16 × 00 / 8 × 01
- Claims (p.201):
  - "no longer any strong tendency for the wider identity 0? to dominate as the balance shifts in favor of the center."
  - With 16 regions of type 00, "any of the three identities 0?, ?0, and 00 provide enough support to topple the imperial government."
  - "the 01 and ?1 type movements disappear almost entirely since their more limited popular support does not suffice."
  - Summary: "If one group dominates, overarching identities become less central to the periphery's struggle against the center. A more fragmented culture map, however, requires more coordination ... the center-periphery power balance is likely to influence the inclusiveness of the revolutionary movement."
- Eyeball digitization (±~1):

| R | 0? | ≤?0 | ≤?1 | ≤00 | total |
|---|---|---|---|---|---|
| 1 | 8 | 12.8 | 13 | 17 | 20 |
| 2 | 7.7 | 12.5 | 12.5 | 18.5 | 20 |
| 3 | 7 | 13 | 13 | 20 | 20 |
| 4 | 6 | 11.5 | 11.5 | 20 | 20 |
| 5 | 7.5 | 15.3 | 15.3 | 19 | 20 |
| 6 | 6.5 | 9.5 | 9.5 | 20 | 20 |
| 7 | 9 | 14 | 14 | 20 | 20 |
| 8 | 9 | 16 | 16 | 20 | 20 |
| 9 | 9 | 14.5 | 14.5 | 20 | 20 |
| 10 | 8.5 | 16 | 16 | 20 | 20 |
| 11 | 9 | 17 | 17 | 20 | 20 |
| 12 | 9 | 18 | 18 | 20 | 20 |
| 13 | 10 | 18 | 18 | 20 | 20 |
| 14 | 9 | 15 | 15 | 19 | 19 |
| 15 | 10 | 15 | 15 | 18 | 18 |
| 16 | ~5 | ~6.5 | | ~7 | ~7 |
| 17 | ~4 | | | | ~4.5 |
| 18–20 | 0 | | | | 0 |

- ?1 is a sliver almost everywhere, even at R = 1, where the creation mix predicts 1/9 (≈ 2.2 of 20). 01 shows about 3/20 at R = 1 and almost nothing for R ≥ 3.
- **Check for our runs:** at R = 1 the creation mix with p = 1/3 is 0? 1/3, 00 2/9, ?0 2/9, 01 1/9, ?1 1/9. The figure's ?1 ≈ 0 is a 1-in-10 event under that mix (P(0 of 20) = (8/9)^20 ≈ 0.095). That's worth noting if we reproduce it.
- An arithmetic boundary: 00 and ?0 can only win while R ≤ 16, so for R ≥ 17 only 0? remains.

#### What to digitize
Figs 8.6 and 8.7 (the stacked areas, per integer R, cumulative bands) are the main targets. Figs 8.4/8.5 only as qualitative single runs: winner identity, winner time and peak memberships. Fig. 8.3's curves can be regenerated analytically. The gridded overlays made for the reading (not kept) were (`grid-215.png`, `grid-216.png`; 300-dpi crops `crop-215.png`, `crop-216.png`).

#### Yugoslavia application (pp.201–10): qualitative only, **no data and no quantitative fit**
- Coding (p.203):
  - Trait 1: Slav = 0, non-Slav = 1 (Germans and Magyars of the post-1867 dual monarchy).
  - Trait 2: Orthodox / "Eastern" = 0, Catholic / "Western" = 1.
  - **Habsburg Monarchy = 1?** (fn 26: not 11, because the Habsburgs didn't impose Catholicism). **Croats (and Slovenes) = 01. Serbs = 00.**
- Mapping of movements (pp.204–206):
  - **0? = Yugoslavism / Illyrianism**: Illyrian Provinces 1809–1814; the Illyrian movement against Magyarization; Strossmayer, Rački.
  - **01 = Croatian nationalism**: Starčević's Party of Rights; Frank's Party of Pure Right after 1896.
  - **?1 = Catholic clericalism**: the Slovene People's Party, trialism.
  - **00 = Serbian nationalism**: Karadžić, Orthodoxy, Cyrillic, "Piedmontization".
  - **?0 = Orthodox clericalism**: the Sremski Karlovci metropolitanate; the church opposed ethnic parochialism.
- Claims (pp.206–208):
  - Variation is evident: conspiracies, Princip, Kvaternik's 1871 uprising.
  - Repression existed: Hungarian control of Croatia after the Ausgleich; the annexation of Bosnia (the book says 1909; historically it was 1908).
  - "when the pendulum swung in favor of the center, less particularistic nationalist programs prevailed. Due to Croatia's weaker position, **Yugoslavism gained more support in Croatia than in Serbia**", strengthening under Khuen-Héderváry and Italian pressure.
  - Serbs accepted Yugoslavism only when weak and in exile: the **Corfu Declaration 1917**, the army defeated, Russia's revolution removing their ally.
  - Fn 30: the late-1930s threat brought Serb concessions to the Croats (the 1939 Sporazum), too late.
- **Self-stated mismatches (p.208):**
  - "the model tends to **exaggerate the frequency** of these attempts."
  - "In reality, the struggle did not usually assume the life-and-death character suggested by the model."
  - Actors "learned from past experiences": the process is Lamarckian (Harré 1979), so variation is not blind.
  - Rejects Friedman's "as-if" defense, since organizational turnover is too slow (Elster 1985).
- Proposed extensions (pp.209–10):
  1. Biased, non-random innovation (leaders' deliberate use of myths).
  2. Leaving or switching movements.
  3. Mergers and splits of whole movements (e.g. the Party of Right split strengthened Yugoslavism).
- Serbia was formally independent from 1878. He treats it as dominated by Austria-Hungary (fn 20, p.202). That's an empirical modeling choice worth flagging in any caption.

---

### 3. Ambiguities and unstated details (each a candidate named switch)

1. **Enforcement scale q: text 1 vs figure 0.1.** p.197 says "we use the same parameter values: q = 1, ρ0 = 1, and c = 3.17". But Rule 2 used q = 0.1, so they are *not* "the same", and Fig. 8.3 plots enforcement saturating at 0.1. The literal text default is q = 1; the switch is q = 0.1 (matching the figure and "same values"). With q = 1, enforcement at parity is 0.5 per period, which changes selection pressure a lot. **This is the top-priority switch.**
2. **What triggers a revolution.**
   - p.197 makes it the outcome of a repression attempt ("In the event of repression ... If the periphery manages to gather superior resources ... a nationalist revolution takes place").
   - But p.198 ("the first movement to win support from the necessary eight regions, allowing it to crush the center") and p.199 ("If ... ρ = 1, a revolution takes place **as soon as a new movement crops up**") imply an immediate check when Nr ≥ R.
   - Under the repression-triggered reading, a movement with Nr ≥ R is attacked only with probability Pr(Enforce | ρ ≤ 1) ≤ q/2, so the takeoff is delayed and other movements can keep growing.
   - Switch: `revolution_on_threshold` (immediate) vs `revolution_on_enforcement`.
3. **Threshold equality.** "same resource level" and the examples (8 of 8, 12 of 12, R = 1 with N = 1) imply ≥. But "superior resources" (p.197) and "as long as the center is more powerful" suggest ties go to the periphery anyway. Use ≥; strict > is a switch.
4. **Which movement wins.** "The strongest movement is recorded as the winner", which is not necessarily the one that triggered the revolution. Ties are unstated.
5. **Duplicate identities.** Fn 15 says creating an existing identity equals joining it, so one movement per identity. **But the Fig. 8.4 narrative has "two peripheral actors independently decide to launch a campaign of type 00" and "one of the 00 movements crumbles".** This could be same-period simultaneous creation under synchronous updating, or a contradiction. Switch: `unique_identity_movements` vs allowing duplicates.
6. **Center's target selection.** "randomly singles out one such region": uniform over disloyal *regions* means a movement is picked with probability proportional to its size, which partly offsets the size-protective enforcement curve. The alternative is uniform over *movements*. Only one enforcement check per period. Switch.
7. **Rule 2 draw structure.**
   - Is each movement judged "safe" by an independent Bernoulli(Pr(Join | ρ_m)), then the highest-fit safe one chosen? Or one draw?
   - The tie-break among equal-fit safe movements is unstated (e.g. 00 actor: 00 has fit 2; ?0 and 0? both have fit 1).
   - Does a dissident whose new platform duplicates an existing one count as "joining" without the safety test (fn 15 says yes)?
8. **Order within a period and synchronous vs sequential updating.**
   - Can an unaffiliated unit both try Rule 1 and Rule 2 in one period? Which comes first?
   - Is the actor order random? (Ch. 4 fn 7, p.83, used parallel updating with random order and locking.)
   - Does the center act before or after joins ("At the same time")?
   - Does membership N used in ρ update as units join during the same period?
9. **Can members be re-recruited after a collapse?** Presumably all members return to unaffiliated. Stated only as "collapse of the oppositional movement".
10. **Rule 1 eligibility.** "per period and unit", but only for units "not already belong[ing] to a movement" (p.194). So the creation hazard falls as membership grows.
11. **ρ asymmetry is stated**: joining uses N+1 and enforcement uses N. Keep it literal. A switch could symmetrize it.
12. **R grid.** Figs 8.6/8.7 x-axis 1..20; I assume integer R = 1..20 with r = 1. Not stated outright.
13. **Fig. 8.5 x-axis.** The labels start at "100" at the origin and run to 400, while the text says the run starts identically to Fig. 8.4 at t = 0. Likely a labeling slip (the ticks are probably shifted by 50) or a truncated window. Don't use the absolute times.
14. **Fig. 8.4/8.5 coupling.** The same seed gives an identical history until t ≈ 150, despite different R. That implies a fixed sequence of uniform draws compared against thresholds (common random numbers). Reproducible in spirit only.
15. **Fig. 8.6's "practically impossible" at R = 16** vs the figure showing ~2/20 at 16 and 0 at 17. Minor.
16. **Fig. 8.7 at R = 1**: ?1 ≈ 0 vs an expected ~2.2/20 (see §2).
17. **Space unused.** The grid is decorative; the model is non-spatial (mean-field). No neighbor interaction at all.
18. Two-trait restriction acknowledged (fn 7, p.188): "the current model may therefore downplay the difficulties of cultural coordination." A natural extension switch is more traits.
19. The fit for 00 vs the 01 platform is 0, excluded by "positive". Make sure the code uses `> 0`, not `>= 0`.

---

### 4. Chapter 9: substantive findings and policy claims, as testable claims

Findings tied to models in the book (Ch. 9 restates them; each is checkable in the relevant reproduction):

- **C9.1** (p.213) Actors should be dependent variables, and state and nation are analytically distinct (formal hierarchy vs informal community). This is a framing claim and not testable.
- **C9.2** (pp.214–15, Emergent Polarity, Ch. 4–5) Unit-level factors are subordinate only under violent, persistent interstate competition. Positive feedback ("hegemonic takeoffs") makes unipolarity likely without balancing.
- **C9.3** (p.215) **Defensive technology and alliances can increase the chance of unipolarity**, by blocking competing great powers from catching up while the leader absorbs small states. Testable: unipolarity frequency vs defense dominance / alliance switches in the Polarity Model.
- **C9.4** (p.215) The security dilemma bolsters geopolitical pluralism: it forces defenders to be aggressive, which promotes multipolar balance. Strategic uncertainty (offense/defense blurring) promotes power politics.
- **C9.5** (p.215; Fig. 5.8 p.125) Resource absorption / fiscal centralization (tax rate) is crucial for balance-of-power systems. Low taxation leads to no power politics: below a 20% tax never; below 10% no convergence in 1,000 periods.
- **C9.6** (pp.216–17) Nationalism unifies culturally homogeneous, politically fragmented regions (Germany, Italy) and disintegrates culturally fragmented, politically unified ones (the Habsburg/Ottoman empires, decolonization, communist federations). This "double convergence" is not directly modeled.
- **C9.7** (p.217, Mobilization Model, Ch. 7) Three regimes: assimilation theory (fast direct and indirect assimilation), delayed assimilation (low indirect rate, assimilation wins eventually), and provocation theory (low both). Under provocation, a **short-term surge in mobilization outpaces assimilation**, can strengthen the movement and lead to secession.
- **C9.8** (pp.217–18) **Imperial dilemma**: external threats force mobilization, which (under provocation) risks secession, so there is an interior optimum of mobilization speed. "too fast a transformation risks provoking secession, too conservative a policy exposes the state to great external dangers." Fn 1: concessions to powerful nationalities give temporary relief but entrench peripheral power brokers and raise the long-run chance of successful challenge.
- **C9.9** (p.218, Coordination Model) **"the more powerful the center, the more inclusive the nationalist identities"**, and "the center's attempts to stop revolutionary activity may accelerate the shift away from parochial loyalties toward more inclusive identities, a transformation that could promote collective action on the periphery in the long run." Testable: the 0? share of wins rises with R (Fig. 8.6). The hedge "could promote collective action" is not shown; the success rate falls at high R.
- **C9.10** (pp.218–19) **Post-independence fragmentation**: "The same factors that lead to revolutionary success cause nation-building failure." Once imperial repression disappears, the selection that favored inclusive identities is gone and old cleavages reappear. **Not modeled**: runs stop at the revolution. This is a natural extension test: continue the run after independence with the center removed and see whether 0? persists or fragments.
- **C9.11** (p.218) Geopolitical and cultural conditions co-determine outcomes. Testable: the interaction of R × population split (Fig. 8.6 vs 8.7).
- **C9.12** (pp.220–21) "Rerun the tape" counterfactual simulation filters out historical accidents and estimates causal effects. Methodological.
- **C9.13** (p.221) Collective identities let latent groups overcome collective-action dilemmas that selective-incentive theories can't explain. Not directly tested.

Future research (pp.221–22): (1) richer internal models / learning; (2) systematic tests of internal-external (two-level) interaction, which Ch. 5 didn't test systematically; (3) simultaneous emergence of states and nations, including carrying the Coordination Model past the "artificially imposed endpoint of successful revolution" (e.g. Hindu–Muslim violence after the British left India).

Policy claims (pp.222–31):

- **P1** (pp.223–26) Distinguish state from nation. Realist analogies (Posen 1993's ethnic security dilemma) applied to ethnic groups obscure the difference between decentralized and centralized power. A state-centric lens **overestimates anarchy among states and cohesion within them**. Examples:
  - The West's focus on territorial over popular sovereignty delayed its Yugoslav response.
  - Germany's early recognition of Croatia lacked safeguards for Serb minorities.
  - The democratic-peace literature ignores the risks of *democratization* (Russett's "Sleeping Beauty" view).
  - "let the people decide" presupposes an existing collective identity (Jennings, Mayall).
  - Recommendation: attend to minorities, not only democracy; Gottlieb's (1994) intermediate statuses between autonomy and sovereignty.
- **P2** (pp.226–28) Emergent identities avoid both reification (East/West "good vs bad nationalism": Kohn, Kedourie, Greenfeld; Tudjman's "clash of civilizations" use of Huntington) and pure instrumentalist relativism (Hechter; Haas-style functionalism surprised by integration refusals). Identities have inertia (Gellner, Smith). **State-led acceleration of modernization in multiethnic states often backfires.** Caution about shock therapy in post-communist states (Motyl 1993) and about rushing EU deepening; a common market is not a pan-European nation.
- **P3** (pp.229–31) CAS models test the robustness of "deductive" policy advice. Against Mearsheimer (1990, 1993):
  - Nuclear proliferation to Germany and Ukraine, stability inferred from "running the tape once" (the drunk-driver analogy).
  - The SDI / Reagan-buildup counterfactual (Lebow & Stein 1994).
  - Laissez-faire balance-of-power claims, where his models show unipolarity "emerges as a serious threat".
  - Conclusion: "social scientists will have to give up the dream of positivist predictability" but "will know more precisely the limits of their knowledge" (p.231).

---

### 5. Links to cited models and overlap with our work

- **Axelrod culture.** The book cites **Axelrod (1995), "The Convergence and Stability of Cultures: Local Convergence and Global Polarization"**, U. Michigan IPPS Discussion Paper No. 375 (Bibliography, p.234), which is the working-paper version of Axelrod (1997) "Dissemination of Culture". It is cited on **pp.61–62 (Ch. 3)**, together with Axelrod 1987, Arthur 1991, March 1991 and Schrodt 1993, as examples of "culture vectors". Cederman distinguishes his approach: culture = Holland *chromosome* (fixed trait string); identity = Holland *schema* with wild cards (pp.61–62). Ch. 8 fn 13 (p.192) cites Holland's (1992a) schemas, Axelrod 1987 strategy strings and March 1991 bit-vectors.
  - **Overlap with our Axelrod 1997 reproduction is representational, not dynamic.** Shared: trait-vector culture and a similarity count (Axelrod uses the share of matching features; Cederman's fit = matches − clashes, ignoring wild cards). Different:
    - Cederman has **no social influence and no assimilation**: traits are fixed (fn 8, p.188).
    - No lattice neighbors: space is immaterial.
    - The dynamics are movement creation, power-dependent joining and repression.
  - Possible reuse: our trait-vector types, and a "schema with wild cards" match operator. An Axelrod-style cultural-drift switch would be our own variation, not Cederman's.
- **Deutsch (1953) *Nationalism and Social Communication*.** It underlies the **Ch. 7 Mobilization Model**: the assimilated/mobilized dichotomies, direct and indirect assimilation, and provocation theory (pp.152–56, 168–70; Ch. 9 p.217). Not used in Ch. 8. Ch. 9 cites "Karl Deutsch's view of nationalist mobilization" for C9.7.
- **Gellner (1983).** Cited in Ch. 8 for "many more nations than there are viable states" (p.184) and critiqued as functionalist/demand-driven (p.210: "a modern, industrial state can only function with a mobile, literate, culturally standardized, interchangeable population"; via Mann: does industrialization *require* nationalism?). The ecological model is offered as a mechanism-based alternative. Gellner (1964) is used in Ch. 3/7 for politicization of distinctions. There is no Gellner model to reproduce.
- **Hannan (1979)** supplies the core ecological theory (pp.186–87): center penetration favors large-scale identities. "If an ethnic stand is to be made against the center, it must be on the basis of some identity larger than that of the premodern ethnic identities" (270). Also Barth (1969) boundaries; Horowitz (1975); Hroch (1985).
- **Threshold / critical-mass models**: Granovetter 1978, Kuran 1989/1991, Macy 1990/1991. These overlap with our Crowd series (Granovetter thresholds) if reproduced. Rule 2 is a power-weighted Granovetter-like bandwagon.
- **Ch. 5 Fig. 5.9 (p.127)**: the same logistic, Pr = 1/(1+(ρ/ρ0)^−c), used for attack decisions there (10% at ratio 2, >90% at ratio 4). It should share code with Ch. 8's Rule 2 and Rule 3.

---

### 6. Implementation language, software and code availability

- **Ch. 4 fn 8, book p.83 (PDF 98):** "The code was developed in **THINK Pascal on a Macintosh** and subsequently ported to **Unix running on Hewlett Packard and Sun workstations** as well as **Turbo Pascal for Windows** on a Pentium platform. Despite the conventional implementations, the computational performance under Unix was sufficient for the purposes of this study. Nevertheless, the synchronous algorithms are particularly well suited for future implementation on a parallel machine." (It cites Duffy 1992/1993 parallelization.)
- Ch. 4 fn 7 (p.83): parallel/synchronous design, with "locking the units that have already been modified in the same time period"; "the order is randomly determined in every time period." This is presumably the convention for Ch. 8 as well, but it isn't restated there.
- **Ch. 8 itself names no software. No code-availability statement anywhere** (Preface, Ch. 8, Ch. 9).
- **Preface (pp.xiii–xiv):** no software mention. Credits Axelrod (adviser), Michael Cohen ("computer modeling and simulation"), John Holland (CAS), the Santa Fe Institute Summer School 1992 and the Michigan CAS study group. Notes prior publication: Ch. 4 as "Emergent Polarity," *ISQ* 38 (1994): 501–33; **Ch. 8 as "Competing Identities: An Ecological Model of Nationality Formation," *EJIR* 1 (1995): 331–65.**
- Our papers dir has `cederman-icr-2004-geosim0-repast-source.zip`. That is later GeoSim (Repast/Java) code for the Ch. 4–5 lineage, not the Coordination Model.

### Wanted papers (for the unfound list)
- Cederman, L.-E. 1995. "Competing Identities: An Ecological Model of Nationality Formation." *European Journal of International Relations* 1(3):331–65. May contain the fuller spec, the enforcement q, update order and revolution trigger.
- Hannan, M. T. 1979. "The Dynamics of Ethnic Boundaries in Modern States." In Meyer & Hannan, eds., *National Development and the World System*, Univ. of Chicago Press.

---------------------------------------------------------------------------------------------------

## 5. Literature around Cederman, *Emergent Actors in World Politics* (1997)

Notes compiled 2026-10-01 for planning a geopolitics reproduction campaign.
All saved files live in `papers/geopolitics/` (gitignored), shown below as `G/`.

### Headline findings

- **Public code exists for GeoSim.** ETH ICR's old teaching archive still serves it:
  - `G/cederman-icr-2004-geosim0-repast-source.zip`: GeoSim0 (Girardin & Cederman, Repast 1.x, Java, GPL-2+). Defaults: 30x30 grid, initPolarity 200, probAttack 0.2, superiorityThreshold 3.0, propMobileResources 1.0, runUntil 10000. It has **no technology shocks**, so it is the base conquest model without the APSR 2003 sandpile driver. Source: http://www.cederman.ethz.ch/teaching/archive/compmodels/ws2004/models/geosim0.zip
  - `G/cederman-girardin-weidmann-icr-2017-growlab-0.9.5-source-geosim-geocontest.zip`: the GROWLab framework source (2017 snapshot, 1156 files, 76 MB uncompressed). Under `models/` it bundles:
    - **geosim0**
    - **geosim2**: header says it "corresponds to" APSR 2003, and also does Cederman & Gleditsch democracy/regime change/collective security. Parameters: shockSize, pShock=0.0001, distSlope=3, distOffset=0.1, collSec, propDem, alliances.
    - **geosim4/geosim5**: nationalism and terrain (Cederman 2002–2004), with Nation/Identity classes.
    - **geocontest**: Weidmann & Cederman 2008, a pluggable strategy API.
    - Source: http://www.cederman.ethz.ch/growlab/ (also has a Modeller's Guide, saved as `G/icr-growlab-modellers-guide.pdf`, and a TechnicalGuide.pdf, not saved).
  - All of this is GPL, so treat it like the Anasazi data: use it to check our build against theirs, and decide on licensing before copying anything.
- **The 1997 models themselves have no public code.** They were written in THINK Pascal on a Mac and ported to Unix and Turbo Pascal (book p. 83, note). The book's appendix and the parameter tables in the ISQ 1994 and JCR 2001 papers are the spec. For the 1997 polarity model, GeoSim0 is the closest code we have.
- **The only replication of a 1997 model I found** is Störmer 2018, a BA thesis at HAW Hamburg (in German). She reimplemented the Emergent Polarity model in Python and sampled parameters at random within "reasonable" ranges.
  - About 3/4 of the random configurations ended with **nothing happening**: no power struggles and no change in the number of states.
  - No single parameter explains which runs work.
  - She concludes the model "produces meaningful results only under particular circumstances" and that Cederman tested few parameters. This is a ready-made "where it fails" lead for us.
  - Saved: `G/stoermer-2018-haw-ba-thesis-emergent-polarity-parameter-variation.pdf`. The 123 pages include the code description and the per-configuration appendix.
- Nothing on CoMSES/OpenABM or in NetLogo reimplements Cederman. I also found no docking study in JASSS.
  - The JASSS review of the book (Lazer 2001) is saved as `G/lazer-2001-jasss-review-emergent-actors.html`.
  - Cioffi-Revilla & Gotts 2003 (JASSS 6(4)/10) compares GeoSim-like models qualitatively. Saved as `G/cioffi-revilla-gotts-2003-jasss-comparative-analysis-agent-based-social-simulations.html`.

---

### 1. Predecessors

#### Bremer & Mihalka 1977
- **Citation:** Bremer, S. A. & Mihalka, M. (1977). "Machiavelli in Machina: Or Politics Among Hexagons." In K. W. Deutsch, B. Fritsch, H. Jaguaribe & A. S. Markovits (eds.), *Problems of World Modeling: Political and Social Implications*, pp. 303–337. Cambridge, MA: Ballinger.
- **Model:** about 30 states on a hexagonal map; power-based conquest and alliances; the first territorial conquest simulation. Cederman (1997 ch. 4, 1994) and Cusack & Stoll describe it at second hand.
- **Spec:** partial. It is a book chapter whose results are sample runs.
- **Status: NOT FOUND.** The volume is out of print, isn't on sci-hub, and I found no scan. Duffy 1992 and Cusack & Stoll 1990/1994 summarize its rules.

#### Cusack & Stoll 1990 (REALPOLITIK / EARTH)
- **Citation:** Cusack, T. R. & Stoll, R. J. (1990). *Exploring Realpolitik: Probing International Relations Theory with Computer Simulation.* Boulder: Lynne Rienner. DOI 10.1515/9781685855871 (De Gruyter reissue).
- **Model:** extends Bremer–Mihalka. A hexagonal world of states; "realpolitik" decision rules; alliance and war; state survival across rule sets. The system is named EARTH ("Exploring Alternative Realpolitik THeses").
- **Book status: NOT FOUND.** It isn't on sci-hub, and the De Gruyter edition is paywalled.
- **Saved substitutes:**
  - `G/cusack-stoll-1994-isq-collective-security-and-state-survival.pdf`. Cusack & Stoll (1994), "Collective Security and State Survival in the Interstate System," *ISQ* 38(1): 33–59, doi 10.2307/2600871. The same simulation with collective-security rules; it states the rules and gives survival numbers, so it is a usable spec.
  - `G/stoll-2005-cmps-civil-reality-simulation-civil-war-realist-world.pdf`. Stoll (2005), "Civil Reality? Simulation Experiments on the Impact of Civil War in a Realist World," *CMPS* 22(1), doi 10.1080/07388940590915309.
  - `G/stoll-2011-simgaming-civil-engineering-realist-world-civil-wars.pdf`. Stoll (2011), "Civil Engineering: Does a Realist World Influence the Onset of Civil Wars?" *Simulation & Gaming* 42(6): 747–770, doi 10.1177/1046878109341765. It uses EARTH again.
- **A critique worth reading first:** Duffy, G. (1992), "Concurrent Interstate Conflict Simulations: Testing the Effects of the Serial Assumption," *Mathematical and Computer Modelling* 16(8/9): 241–270, doi 10.1016/0895-7177(92)90099-7.
  - Duffy re-ran a Bremer–Mihalka/Cusack–Stoll-type world with parallel updating and showed that the serial-update assumption changes the results. This is a classic early "scheduling artifact" finding.
  - Cederman 1997 adopted synchronous updating with unit locking because of it.
  - Saved: `G/duffy-1992-mcm-concurrent-interstate-conflict-simulations-serial-assumption.pdf`.
  - Not found: Duffy 1993, "Historical Reflection and the Outcomes of War: A Massively Parallel Computer Simulation," an ISA conference paper.

#### Axelrod 1995/1997, tribute model
- **Citation:** Axelrod, R. (1995). "A Model of the Emergence of New Political Actors." In N. Gilbert & R. Conte (eds.), *Artificial Societies*, pp. 19–39. London: UCL Press. It was earlier SFI Working Paper 93-11-068 and was reprinted as ch. 6, "Building New Political Actors," of *The Complexity of Cooperation* (Princeton 1997).
- **Model:**
  - 10 actors on a ring, initial wealth U(300,500), 1000 "years", 3 active actors per year.
  - An active actor demands 250 from a neighbor, who pays or fights.
  - Fights destroy 25% of the opponent's wealth, Lanchester-style.
  - Each actor gains +20 wealth per year.
  - Commitments (loyalty) grow ±10% through tribute and fighting together, and alliances form along contiguous chains of commitment.
  - The emergent "new actors" are clusters of high mutual commitment.
- **Claims:**
  - Stable clusters of commitment emerge, with a dominant "leader".
  - Some clusters suffer "imperial overstretch" and civil war inside the cluster.
  - The paper reports runs of 1000 years and 4 population-level case histories.
- **Spec:** very reproducible. The rules are fully stated, the paper has numbers and figures, and the **original THINK Pascal source survives**.
- **Saved:**
  - `G/axelrod-1995-artificial-societies-model-emergence-new-political-actors.pdf`: 21-page image scan cut from the open-access book, plus a `.ocr.txt` sidecar.
  - `G/gilbert-conte-1995-artificial-societies-book-scan.pdf`: the whole OA book, a 320-page image scan. Its other chapters include Doran's EOS project.
  - `G/axelrod-1997-tribute40-pascal-source.txt`: Tribute40.p, v4.0. Constants: Imax=10, Demand_phase_max=3, W_base=400, Standard_Demand=250, Destructiveness=0.25, plus options for rules A1–A4 and B1–B6, contiguity and loyalty.
  - `G/axelrod-1997-tribute-readme.txt`.
  - Both source files came from the Wayback copy of pscs.physics.lsa.umich.edu/Software/CC/CC6/.
- **Follow-ups:**
  - Walbert, Caton & Norgaard (2018), "Countries as Agents in a Global-Scale Computational Model," *JASSS* 21(3)/4. Tribute dynamics on real Correlates of War alliance networks. Code is on GitHub at hwalbert/Countries-as-Agents-in-a-Global-Scale-Computational-Model (NetLogo/R/Python). Saved as `G/walbert-caton-norgaard-2018-jasss-countries-as-agents.html`.
  - Poulshock (2019), "The Foundations of Political Realism," arXiv:1910.04785. A tribute-like realism model. Saved as `G/poulshock-2019-arxiv-foundations-political-realism.pdf`.
  - Antunes et al. (2002), "BVG Choice in Axelrod's Tribute Model," MABS 2002, LNAI 2581, doi 10.1007/3-540-36483-8_2. It changes the decision rules. **Not found** (not on sci-hub).

#### Cederman 1994 (the 1997 book's direct precursor)
- **Citation:** Cederman, L.-E. (1994). "Emergent Polarity: Analyzing State-Formation and Power Politics." *International Studies Quarterly* 38(4): 501–533. doi 10.2307/2600863.
- **Model:** the Emergent Polarity model.
  - A 10x10 grid (20x20 in an illustration) of primitive units.
  - Predator states conquer neighbors, alliances form, and the number of sovereign states (polarity) emerges.
- **Results:** 20 replications per setting, run for 1000 periods or until hegemony. It reports state survival against predator frequency and offense/defense dominance. One cell's 20 runs gave 5 unipolar, 9 bipolar and 5 multipolar outcomes.
- **Spec:** good. It gives rules and replication counts, and results as distributions over runs.
- **Saved:** `G/cederman-1994-isq-emergent-polarity.pdf`.

---

### 2. Cederman's follow-ups

#### Cederman 2001 (JCR), democratic peace
- **Citation:** Cederman, L.-E. (2001). "Modeling the Democratic Peace as a Kantian Selection Process." *Journal of Conflict Resolution* 45(4): 470–502. doi 10.1177/0022002701045004004.
- **Model:**
  - Built on the 1997 framework: a 15x15 grid of 225 primitive units, 1000 periods, 5% "hegemons".
  - Democracies use tags, conditional cooperation and ideological alliances.
- **Claims:**
  - Tags alone leave the democratic share at about 20–23%.
  - Alliances bring it to 28% in the sample run.
  - Ideological alignment raises the democratic success rate to over 80%.
  - Above about 20% initial democracies, the democratic territorial share exceeds 90%.
- **Replication design:** 30 replications at each initial democracy share of 0, .05, .1, .15, .2, .25, .3, .4, .5, .6, .7 and 1.
- **Spec:** very reproducible. The appendix has a full parameter table.
- **Saved:** `G/cederman-2001-jcr-democratic-peace-kantian-selection-process.pdf`, an image scan with OCR in `.ocr.txt`.
- **Companion:** Cederman & Rao (2001), "Exploring the Dynamics of the Democratic Peace," *JCR* 45(6): 818–833, doi 10.1177/0022002701045006006. Empirical, with time-varying effects; not an agent-based model. Saved as `G/cederman-rao-2001-jcr-exploring-dynamics-democratic-peace.pdf` (scan).

#### Cederman 2002 (PNAS)
- **Citation:** Cederman, L.-E. (2002). "Endogenizing Geopolitical Boundaries with Agent-Based Modeling." *PNAS* 99(suppl 3): 7296–7303. doi 10.1073/pnas.082081099.
- **Content:** an overview of the "finite-agent method".
  - The first half is GeoSim, with states as networks on a grid.
  - The second half is a nationalism model in which national identities are cultural strings over the grid ("1 of 25 values").
  - Table 2 gives four types of macro validation.
- **Spec:** largely conceptual. The mechanisms are spelled out in more detail elsewhere.
- **Saved:** `G/cederman-2002-pnas-endogenizing-geopolitical-boundaries.pdf`.

#### Cederman 2003 (APSR), war sizes ★ the best target
- **Citation:** Cederman, L.-E. (2003). "Modeling the Size of Wars: From Billiard Balls to Sandpiles." *APSR* 97(1): 135–150. doi 10.1017/S0003055403000571.
- **Model:** GeoSim.
  - 50x50 lattice, about 200 initial states.
  - Half of each state's resources are spread evenly across its fronts; the other half form a fungible pool.
  - States play grim trigger. They attack unprovoked when superiority passes a 3:1 threshold, probabilistically, with probability 0.01 per period, rising to every period on "alert" when a neighbor is fighting (context activation).
  - Capitals tax provinces through a logistic loss-of-strength gradient, flattening to about 10% far away.
  - Technological change shifts that gradient.
  - War clusters are tracked with a "warShadow" of 20 periods, and battle damage is 10% of the resources allocated to a front.
  - Periods 0–500 have no technology; technology diffuses from then until period 10,500.
- **Empirical target:** COW/Levy severity data from 1820 to 1997. P(S>s) follows a power law with slope -0.41 and R² 0.985; Levy's great-power data give slope -0.57.
- **Claims (Table 1, 15 runs per row; slopes are min / median / max):**

  | Row | Setting | Slope min / median / max | Median R² |
  |---|---|---|---|
  | 1 | Base runs | -0.64 / -0.55 / -0.49 | 0.991 |
  | 2 | Smaller shocks | -0.71 / -0.62 / -0.56 | |
  | 3 | No shocks | -1.17 / -1.43 / -1.32 | lower |
  | 4 | No context activation | -1.52 / -1.34 / -1.20 | lower |

  - Table 1 also has a sensitivity sweep over warShadow (10 and 40), supThresh 2.5, propMobile 0.9, distOffset, distSlope, and a 75x75 grid.
  - Sample run: 218 wars with log P = 1.68 - 0.64 log s (R² 0.991). 35 states remain at t=10,500.
- **Spec:** excellent. There is a parameter table in the appendix, the claims are quantitative, and **code exists** (GROWLab geosim2).
- **Things to watch:**
  - Shallow simulated slopes (-0.55) against the empirical -0.41.
  - Small-event truncation below log s 2.5.
  - Fitting by OLS on the CCDF. Clauset, Shalizi & Newman would reject this; refit with MLE.
- **Saved:**
  - `G/cederman-2003-apsr-modeling-the-size-of-wars.pdf`.
  - Working paper version: Cederman (2002), "Modeling the Self-Organized Criticality of War Size," saved as `G/cederman-2002-wp-modeling-self-organized-criticality-war-size.pdf`.

#### Cederman 2003, state-size distributions
- **Citation:** Cederman, L.-E. (2003). "Generating State-Size Distributions: A Geopolitical Model." Paper for the Agent 2003 conference, Chicago. Unpublished.
- **Empirical claim:** state territorial sizes from 1815 to 1998 are **log-normal**, not power law. The paper estimates μ and σ by year and reports MAE by year.
- **Model claim:** GeoSim needs an addition to produce log-normal sizes.
- **Spec:** good, using the same GeoSim spec.
- **Saved:** `G/cederman-2003-generating-state-size-distributions.pdf` (29 pp).

#### Cederman & Gleditsch 2004 (ISQ), democracy
- **Citation:** Cederman, L.-E. & Gleditsch, K. S. (2004). "Conquest and Regime Change: An Evolutionary Model of the Spread of Democracy and Peace." *ISQ* 48(3): 603–629. doi 10.1111/j.0020-8833.2004.00317.x.
- **Model:** GeoSim (50x50, about 200 states) with 10% initial democracies. It adds S-shaped neighborhood-dependent regime change and collective security among democracies.
- **Claims (Table 1, 20 runs each):**

  | Setting | Mean final democratic share | Runs above 10% | Runs above 50% |
  |---|---|---|---|
  | Base | .072 | 15% | 0% |
  | + regime change | .064 | 15% | 5% |
  | + collective security | .636 | 80% | 65% |

  - Target: the empirical democratic share rises to about 50%.
- **Spec:** very reproducible, and the code exists (geosim2 with collSec).
- **Saved:** `G/cederman-gleditsch-2004-isq-conquest-and-regime-change.pdf`.

#### Cederman 2004/2008, nationalist insurgency
- **Citation:** Cederman, L.-E. (2008). "Articulating the Geo-Cultural Logic of Nationalist Insurgency." In S. N. Kalyvas, I. Shapiro & T. Masoud (eds.), *Order, Conflict, and Violence*, pp. 242–270. Cambridge UP.
- **Model:** GeoSim plus nationalism, cultural maps and terrain, used to study civil war onset. This is GROWLab geosim4/5 ("Nov 30 2002 extending geosim2 to include nationalism", "Dec 8 2003 nationalism & terrain").
- **Spec:** moderately reproducible. Saved is the 2004 working version (40 pp): `G/cederman-2004-articulating-geo-cultural-logic-nationalist-insurgency.pdf`.

#### Weidmann & Cederman 2008, GeoContest
- **Citation:** Weidmann, N. B. & Cederman, L.-E. (2008). "GeoContest: Modeling Strategic Competition in Geopolitical Systems." *Social Science Computer Review* 26(4): 510–518. doi 10.1177/0894439307313516.
- **Model:** simplified GeoSim with pluggable state strategies, used to test strategies such as overconfidence. The code is in GROWLab (models/geocontest).
- **Saved:** `G/weidmann-cederman-2008-sscr-geocontest.pdf`.

#### Cederman & Girardin 2010 (ISQ), sovereignty
- **Citation:** Cederman, L.-E. & Girardin, L. (2010). "Growing Sovereignty: Modeling the Shift from Indirect to Direct Rule." *ISQ* 54(1): 27–48. doi 10.1111/j.1468-2478.2009.00576.x. (The task's alternative "Weidmann & Cederman" attribution is wrong; it is Cederman & Girardin.)
- **Model:**
  - First a 1-D analytic model of organizational versus geographic distance, with propositions proved in an appendix.
  - Then an agent-based model: a 30x30 lattice, 200 states, organizational levels, and "bypass" of intermediate layers.
- **Claim:** technological change (shifting the loss-of-strength gradient) is enough to make direct rule take over. The share of territory under direct rule is shown in Fig. 12.
- **Twist:** with an exponential instead of a logistic gradient, the shift does not happen. The direct share never exceeds 0.25 across 30 runs. A built-in robustness failure worth reproducing.
- **Spec:** good. I found no code; GROWLab has no "sovereignty" model.
- **Saved:** `G/cederman-girardin-2010-isq-growing-sovereignty.pdf`.

#### Other related work
- **Cederman, Warren & Sornette (2011).** "Testing Clausewitz: Nationalism, Mass Mobilization, and the Severity of War." *International Organization* 65(4): 605–638. doi 10.1017/S0020818311000245.
  - Empirical: a power-law shift in war severity after the French Revolution.
  - It gives a target for "with nationalism" GeoSim variants.
  - Saved as `G/cederman-warren-sornette-2011-io-testing-clausewitz.pdf`.
- **Cederman, Girardin & Müller-Crepon (2023).** "Nationalism and the Puzzle of Reversing State Size." *World Politics* 75(4): 692–734. doi 10.1353/wp.2023.a908773.
  - Empirical: average state size rose for centuries and fell after about 1880.
  - **NOT FOUND** (not on sci-hub; Project MUSE is paywalled).
- **Cederman & Daase (2003).** "Endogenizing Corporate Identities." *EJIR* 9(1): 5–35. Theory only; not saved.
- **Cederman (2007).** Slides, "Complexity" talk at the Stockholm Resilience Centre. Saved as `G/cederman-2007-slides-complexity-stockholm.pdf`.

#### Lustick's PS-I
- Lustick, I. S. (2002). "PS-I: A User-Friendly Agent-Based Modeling Platform for Testing Theories of Political Identity and Political Stability." *JASSS* 5(3)/7. Saved as `G/lustick-2002-jasss-ps-i-platform.html`.
- Lustick, Miodownik & Eidelson (2004). "Secessionism in Multicultural States: Does Sharing Power Prevent or Encourage It?" *APSR* 98(2): 209–229. doi 10.1017/S0003055404001108. Saved as `G/lustick-miodownik-eidelson-2004-apsr-secessionism-multicultural-states.pdf`.
- These are identity-repertoire models on a grid rather than conquest models. They are adjacent to Cederman's nationalism work, and PS-I software was distributed from Penn (polisci.upenn.edu/ps-i).

---

### 3. Replications, dockings and critiques

| Item | What it is | Status |
|---|---|---|
| Störmer 2018 BA thesis, HAW Hamburg: "Eine Analyse politikwissenschaftlicher Modellbildung: Parametervariation am Beispiel des Emergent Polarity Model" | Python reimplementation of the 1997 model plus random parameter sampling. About 3/4 of runs are inert, and she finds a weak negative link between the number of states and the number of fights. | saved |
| Duffy 1992 MCM | serial vs. concurrent updating in Bremer–Mihalka-type worlds | saved |
| Lazer 2001 JASSS review of the book | review | saved (html) |
| Cioffi-Revilla & Gotts 2003 JASSS 6(4)/10 | comparative analysis of GeoSim-like models | saved (html) |
| Kuperman 2010, arXiv:1007.0229, "A model for the emergence of geopolitical division" | independent physics-style conquest model on a lattice (Collins geopolitics); finds capitals near barycenters | saved `G/kuperman-2010-arxiv-model-emergence-geopolitical-division.pdf` |
| GeoSim code (geosim0 2004; GROWLab geosim0/2/4/5 and geocontest) | original authors' code, GPL | saved (zips) |
| CoMSES / OpenABM / NetLogo ports of the 1997 models or GeoSim | none found | — |

---

### 4. Empirical regularities (targets)

- **Richardson's law.**
  - Richardson, L. F. (1948). "Variation of the Frequency of Fatal Quarrels with Magnitude." *JASA* 43: 523–546. doi 10.1080/01621459.1948.10483278. Saved as `G/richardson-1948-jasa-frequency-of-fatal-quarrels-with-magnitude.pdf`.
  - Richardson's *Statistics of Deadly Quarrels* (1960) is a book and was not obtained.
- **Modern statistical treatment.**
  - Clauset, A. (2018). "Trends and Fluctuations in the Severity of Interstate Wars." *Science Advances* 4: eaao3580. MLE power-law fit to COW wars 1823–2003 with α ≈ 1.53 (density exponent) and x_min ≈ 7061 deaths. It finds no statistically significant trend toward peace, and gives the right fitting method for checking Cederman's slopes. Saved as `G/clauset-2018-sciadv-trends-fluctuations-severity-interstate-wars.pdf`.
  - Clauset (2019/2020), "On the Frequency and Severity of Interstate Wars," arXiv:1901.05086. Saved as `G/clauset-2019-arxiv-frequency-and-severity-of-interstate-wars.pdf`.
- **Levy (1983).** *War in the Modern Great Power System, 1495–1975*. A book, not obtained. It is the source of the slope -0.57 in Cederman 2003. Its data tables are reproduced in many places.
- **Data, not downloaded.** Correlates of War Inter-State War data v4 (correlatesofwar.org) is free, and so is its State System Membership list (for polarity and number of states).
- **State death.**
  - Fazal, T. M. (2004). "State Death in the International System." *International Organization* 58(2): 311–344. doi 10.1017/S0020818304582048. Saved as `G/fazal-2004-io-state-death-in-the-international-system.pdf`.
  - The book (*State Death*, Princeton 2007) was not obtained. Its finding: 66 of 207 states died between 1816 and 2000, mostly buffer states, and violent state death nearly stopped after 1945.
- **Number of European states over time.**
  - Tilly, C. (1990), *Coercion, Capital, and European States, AD 990–1990*. A book, not obtained. It is the source of "about 500 state-like units in 1500, about 25 in 1900". The figure is quoted in Gennaioli & Voth, who give their source as Tilly 1990.
  - Abramson, S. F. (2017). "The Economic Origins of the Territorial State." *International Organization* 71(1): 97–130. doi 10.1017/S0020818316000308. **New data on all European polities from 1100 to 1790 at 5-year intervals.** It shows small units persisting and the number of states rising until the 13th century. This is the best quantitative target for "the number of states shrinking". Saved as `G/abramson-2017-io-economic-origins-territorial-state.pdf`. The replication data should be on the IO Dataverse; I didn't fetch it.
  - Gennaioli, N. & Voth, H.-J. (2015). "State Capacity and Military Conflict." *Review of Economic Studies* 82(4): 1409–1448. A bellicist formal model plus European data: the number of states falls and fiscal capacity rises after 1500. Saved as the working-paper version, `G/gennaioli-voth-2015-restud-state-capacity-and-military-conflict-wp.pdf`.
- **State-size distribution:** log-normal, 1815–1998 (Cederman 2003 conference paper, above).
- **Polarity and great-power counts:** Levy 1983 and the COW major-power list; not saved.

---

### 5. Related models for extensions

- **Artzrouni, M. & Komlos, J. (1996).** "The Formation of the European State System: A Spatial 'Predatory' Model." *Historical Methods* 29(3): 126–134. doi 10.1080/01615440.1996.10112734.
  - A map of Europe on a grid of cells about 40 km across.
  - It starts in 500 AD with equal 5x5-cell states.
  - Power grows with size and falls with distance from the capital (overextension).
  - It runs 500–1800 AD, and some runs yield recognizable Spain, Italy and Britain.
  - The spec is short and analytic, so it is very reproducible on a real map, which would pair well with our terrain/geography work.
  - Saved as `G/artzrouni-komlos-1996-histmethods-formation-european-state-system-predatory-model.pdf`.
- **Alesina, A. & Spolaore, E. (1997).** "On the Number and Size of Nations." *QJE* 112(4): 1027–1056.
  - The analytic size-of-nations model: heterogeneity costs traded against economies of scale. It predicts the equilibrium number of nations under democracy versus a Leviathan.
  - Saved is the NBER WP 5050 version (1995): `G/alesina-spolaore-1995-nber-number-and-size-of-nations.pdf`. The QJE version is not on sci-hub.
- **Turchin, Currie, Turner & Gavrilets (2013), PNAS.** Already queued; files in `papers/archaeology/`.
- **Gavrilets, Anderson & Turchin (2010), chiefdom cycling.** Already queued; in `papers/archaeology/`.
- **Tilly-style bellicist models.** Gennaioli & Voth 2015 (saved). Other options I didn't fetch:
  - Hoffman (2015), *Why Did Europe Conquer the World?* (tournament model).
  - Konrad & Skaperdas, "The Market for Protection and the Origin of the State" (free at sites.socsci.uci.edu/~sskaperd/konradskaperdas0306.pdf).
- **Cioffi-Revilla, Honeychurch & Rogers (2015).** "MASON Hierarchies: A Long-Range Agent Model of Power, Conflict, and Environment in Inner Asia." A polity rise-and-fall agent-based model on a steppe landscape. Not fetched (Bonn UP chapter).
- **Kuperman (2010).** See section 3.

### Not obtained (wanted list)

1. Bremer & Mihalka (1977), "Machiavelli in Machina" (Ballinger chapter).
2. Cusack & Stoll (1990), *Exploring Realpolitik* (book; 1994 ISQ article used instead).
3. Duffy (1993), ISA paper on massively parallel simulation.
4. Antunes et al. (2002), "BVG Choice in Axelrod's Tribute Model" (LNAI 2581).
5. Cederman, Girardin & Müller-Crepon (2023), *World Politics* 75(4).
6. Alesina & Spolaore (1997), QJE published version (NBER WP saved instead).
7. Tilly (1990), *Coercion, Capital, and European States* (book).
8. Levy (1983), *War in the Modern Great Power System* (book).
9. Fazal (2007), *State Death* (book; 2004 IO article saved).
10. Richardson (1960), *Statistics of Deadly Quarrels* (book).
11. Cioffi-Revilla, Honeychurch & Rogers (2015), MASON Hierarchies chapter.
12. Cioffi-Revilla & Midlarsky (2004), "Power Laws, Scaling, and Fractals in the Most Lethal International and Civil Wars" (chapter in *The Scourge of War*).
13. GeoContest TechnicalGuide.pdf, and the Abramson 2017 replication data. Both are available but not fetched.
