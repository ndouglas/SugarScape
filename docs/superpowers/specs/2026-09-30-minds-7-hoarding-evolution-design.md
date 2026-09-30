# Minds 7: the evolution of larder hoarding (design)

**Date:** 2026-09-30
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 7. This is P1b of the pilfering campaign.
The campaign runs:

- P1: theft (Minds 6, done).
- **P1b: evolution (this milestone).**
- P2: watching.
- P3: protection.
- P4: deception.

**Approach:** reproduce the originator's model first. Vander Wall and Jenkins's genetic algorithm is built as
written, as its own non-spatial model kind, `hoard`. What Minds 6 taught us comes in as named switches. A
spatial version (the same evolution in our Sugarscape world) is a later step, and only if this one reproduces.

**Builds on:** the repository's model-kind pattern (for example `zi`, `punishment`, `retirement`), the
literal-default-plus-named-switch pattern, the titles rules, the Minds menu entry, and "judges before runs".
Model agents are "agents", never "Flumps".

**Source**, a local copy in `papers/caching/`: S. B. Vander Wall and S. H. Jenkins, "Reciprocal pilferage
and the evolution of food-hoarding behavior", *Behavioral Ecology* 14(5) (2003), 656–667. It is cited below
as V&J, with printed page numbers.

## Goal

Build V&J's genetic algorithm exactly and test the results they report:

- Larder hoarding is all-or-nothing.
- The switch between outcomes depends on how visible scattered caches are relative to larders.
- Larders lose food at higher and more variable rates.
- The best early larder, not the average, predicts takeover.
- Scatter hoarding withstands about 18 % loss a day.

Then vary what they held fixed: owners who don't recover their own caches for free, a cheater that never
caches, and cache visibility (their own untested prediction).

## Non-negotiable constraints

- **Originator first.** The defaults are V&J's model and parameters. Where the paper is silent, the plan's
  first task reads the Methods (pp. 664–666) and records each gap it fills as a stated choice in this
  spec's amendments. It never tunes a gap to reproduce a result.
- **Isolation.** A new model kind. Nothing in the Sugarscape world changes, and every existing golden entry
  and fingerprint stays unedited.
- **Determinism.** One seeded RNG per world, with draws in a fixed, documented order.
- **Judges before runs.** Survey judges and thresholds are committed before any survey run and are never
  tuned.
- **Honest reporting.** Causes are labeled "likely" unless isolated. Failures to reproduce are findings.

## Source summary (V&J)

- **Setting** (p.664): "a population of 20 individuals harvesting and storing food during a nonbreeding
  season of 100 days", with "20 bouts of foraging per day". Each individual "had to harvest one item,
  either from the public supply or from its own stores, to survive the day."
- **Food** (p.665): "1.05 × 20 individuals × 100 days = 2100; i.e., 82 items on day 1, 81 on day 2, 79 on
  day 3, ..., 2 on day 50". Days 1–5 offer "alternative nonstorable food items". Unharvested public food
  carries over.
- **Actions** (p.664): "consume one food item from its own stores, forage for consumption, forage and scatter
  hoard or larder hoard if successful at finding a food item, or remain in the burrow defending a larder. The
  first two options took priority".
- **Own stores** (p.665): individuals with stored items "were assumed to use one of these items to satisfy
  their daily food requirements". There is no search step for the owner.
- **Apparency** (pp.662, 665): app_scat < 1, swept over 0.05–0.90; app_lard between 1 and 3.
- **Search** (p.665):
  - k = forage_i × (−ln 0.01 / (20 × 82^1.5)) × (food available)^1.5.
  - P = 1 − e^(−k) per bout.
  - forage_i ~ N(1, 0.1).
  - Calibration: k = 0.23 gives 0.21 per bout, and a 0.01 chance of complete failure on day 1.
  - Food available to i is public food, plus others' scatter hoards × app_scat, plus others' larders ×
    app_lard. The item found is drawn in proportion to those pools.
- **Larder raids** (p.662): a pilferer "continues to do so until the end of 1 day of foraging or until the
  owner returns to defend the larder hoard".
- **Defense** (p.664): logistic in larder fullness, "0.5 when the amount of food stored in the burrow was
  half of the target amount". A heritable propensity in [0, 1] places the target between the minimum needed
  to survive and the maximum obtainable.
- **Other parameters:**
  - Predation is 0.0001 per bout.
  - Heritability is 0.80.
  - Initial mean larder probability is about 0.15 (logistic distribution).
  - Initial mean defense propensity is about 0.50 (p.665).
- **Fitness and reproduction** (pp.662, 664–665): fitness is "directly proportional to the number of items
  remaining in an individual's stores". Starved individuals die, and survivors reproduce in proportion to
  their stores. Generations don't overlap, n is 20, and a run lasts 60 generations.
- **Inheritance** (pp.665–666): X′ = N[h²·X_p + (1 − h²)·X̄, V_seg] on the logit scale, where X_p is the
  midparent value.
- **Results:**
  1. p.663: "In 35 runs of the model, average probability of larder hoarding always remained less than 0.2
     or increased rapidly to more than 0.95, usually within 10 generations".
  2. p.663: "For ratios less than 0.2, high probabilities of larder hoarding never evolved; for ratios
     greater than 0.3, high probabilities of larder hoarding almost invariably evolved". The Fig. 2B fit
     is "logit = −8.00 + 36.59 × ratio; McFadden's ρ² = 0.61", with its 50 % point at 0.219.
  3. p.663: larder items were "lost at a much greater rate (mean = 186% per day) than were scattered caches
     (24%/day)". The 186 % can't be a daily proportion; it is reported as printed and flagged. The larder
     loss rate exceeded the scatter rate "in all 35 simulations". Larder loss has a CV of 57 % and scatter
     loss 33 %.
  4. p.663: "If one or more individuals in the first 10 generations lost items from larders at a lower rate
     than the average rate of loss of scattered caches in the population, then larder hoarding usually
     became established".
  5. p.663: "the average daily rate of loss of scatter hoards was 18% in cases in which larder hoarding did
     not become established".
  6. p.663, untested: "the prevalence of scatter hoarding in communities should depend on environmental
     conditions that affect the apparency of scattered caches and larders".
  7. p.661, untested: "under ideal conditions (e.g., mild winters), a nonhoarding cheater could survive and
     even flourish".
- V&J describe the model as "a computational tool", not a mechanistic description (p.662). A sensitivity
  analysis "will be reported elsewhere" (p.665).

## The model (`hoard`)

- **Time.** One tick is one foraging bout: 20 bouts a day, 100 days a season, and one season per
  generation. At the season's end, fitness is scored and the next generation is bred.
- **Agents.** Each of the 20 agents has:
  - scatter stores and larder stores (item counts);
  - heritable traits: larder probability L and defense propensity D;
  - a per-generation foraging efficiency, drawn from N(1, 0.1);
  - per-day state: whether it has eaten today, and whether it is defending.
- **Each bout, in the order the plan's first task fixes from the Methods:**
  - **An unfed agent** eats from its own stores if it has any, with no search. Otherwise it forages to eat.
  - **A fed agent with a larder** defends with probability logistic(fullness / target), where the target is
    placed by D. Otherwise it forages to store.
  - **Foraging:** detection with P = 1 − e^(−k). A found item is drawn from public food, others' scattered
    caches (weighted by app_scat) or others' larders (weighted by app_lard). A stored item goes into the
    agent's larder with probability L, otherwise into a scattered cache. An undefended larder, once found,
    is raided one item per bout for the rest of the day, unless its owner returns.
  - **Predation:** 0.0001 per bout.
- **End of season.** Agents that failed to eat on any day have died then. Fitness is the stores left.
  Parents are drawn in proportion to fitness, and offspring traits follow the logit-scale regression.
- **Measurements, per generation:**
  - mean L and mean D;
  - each agent's daily loss rates for larder and scattered items;
  - the share of stores in larders;
  - survivors;
  - "larders take over", meaning mean L > 0.95, and the generation it happened.

## Switches (reported as new ground, never judged against V&J)

| Field | Default | Meaning |
|---|---|---|
| `hoard.app_scat` | 0.5 | Apparency of scattered caches (0.05–0.9) |
| `hoard.app_lard` | 2.0 | Apparency of larders (1–3) |
| `hoard.owner_recovery` | 1.0 | Chance an owner finds each of its own scattered caches when it goes to eat from them. 1 is V&J's free recovery. |
| `hoard.cheaters` | 0 | Share of founders that never cache and eat what they find. V&J's untested non-hoarding cheater. |
| `hoard.generations` | 60 | Generations per run |
| `hoard.heritability` | 0.8 | h² |

The exact defaults for app_scat and app_lard are chosen in the plan so that the default preset sits at the
threshold.

## Survey (`minds7` claims)

The judges are committed first. There are at least 35 runs per condition, matching V&J's count.

1. **All-or-nothing.** Final mean L < 0.2 or > 0.95 in every run, and the rise happens within 10 generations
   in most runs. Judged across a grid of ratios.
2. **Threshold.**
   - Fit a logistic of "larders take over" against app_scat / app_lard.
   - Judge the 50 % point against 0.219, with a tolerance stated in the judge commit.
   - Also judge no takeover below 0.2 and takeover almost always above 0.3.
3. **Larder loss.** The mean daily larder loss rate exceeds the scatter loss rate in every run. The CV of
   the larder rate is about twice the scatter CV, reported against 57 % and 33 %.
4. **Predictor.** "The minimum larder loss in the first 10 generations is below the mean scatter loss"
   predicts takeover better than a comparison of means does.
5. **Scatter withstands loss.** Mean daily scatter loss in runs without takeover is reported against 18 %.
6. **Visibility (new ground).** Sweep app_scat. Lower visibility should favor scatter hoarding.
7. **Owner recovery (new ground).** Takeover and survival as `owner_recovery` falls below 1.
8. **Non-hoarding cheater (new ground).** Cheater share over generations, and survival, against hoarders.

## Verification (tests)

- The food schedule sums to 2 100, with 82 items on day 1 and 2 on day 50.
- The search calibration gives k ≈ 0.23 and P ≈ 0.21 at the start, and a day-1 total-failure chance near
  0.01.
- Inheritance keeps traits in (0, 1). With V = 0, an offspring equals h²·midparent + (1 − h²)·mean on the
  logit scale.
- Fitness-proportional selection is right in expectation.
- Conservation: public food + stores + eaten + lost equals everything supplied.
- The same seed gives the same run.

## Page

- `hoard` joins the model list. Its presets name "Minds 7" in `source`, so they appear under the Minds entry
  as "Minds 7: evolution of hoarding".
- A population panel shows the 20 agents with their stores (larder and scattered) and their L.
- Charts:
  - mean L and mean D by generation;
  - daily loss rates, larder against scatter;
  - survivors.
- Inspect shows an agent's traits, stores and losses.

## Docs

- README: a Minds 7 section.
- The program document: status, results, and the campaign's next step (P2, watching, or the spatial
  version of P1b).
- The roadmap line.
- This spec's amendments.

## Amendments (implementation)

### Methods as read

The model's full description is the Appendix, pp. 664–666 ("General description of the model" and
"Implementation of the model"); p. 662 summarizes it. It is more specific than the Source summary above,
and where they differ the Appendix governs. Read in full with `pdftotext -layout`; the two equations were
checked against a 150 dpi render of pp. 665–666. What the Appendix adds:

- **A day, as the program runs it** (p. 665): "For each new day, the public supply was computed as the sum
  of carryover from the previous day (for all days) plus new production (for days 1–50). For days 6–100,
  the state of each living individual was set at −1 (hungry)." States are "satiated = 0, hungry = −1, or
  dead = 999".
- **Bout 1** (p. 665): "For the first foraging bout on each day, individuals with any items stored in
  larder hoards or scatter hoards were assumed to use one of these items to satisfy their daily food
  requirements, and their states were set at 0 (satiated). If individuals had only larder hoards or both
  larder hoards and scatter hoards, they used a larder-hoarded item and were assigned a probability of
  burrow defense of 1; if individuals had only scatter hoards, they used one of these and were assigned a
  probability of burrow defense of 0. Otherwise, individuals were identified with a flag that specified that
  the first item they harvested during the day would be used to satisfy their daily food requirement."
- **Later bouts** (p. 665): "For all bouts after the first daily bout, probabilities of burrow defense for
  individuals with state = 0 were determined by the ratio of food stored in the burrow to the target value".
- **Bout order** (p. 665): "For each bout, the program called a subroutine that determined whether each
  individual was preyed upon. Then individuals that were not defending their burrows were selected for
  foraging in random order. For individuals that had harvested an item from another animal's larder hoard
  during the previous bout, the program used a subroutine to determine if the individual was successful at
  the same burrow during the current bout, based on whether larder-hoarded items remained in the burrow and
  whether the owner was defending it. Finally, the program used a subroutine to simulate the searching
  process."
- **Search** (p. 665): the rate is printed as λ (the spec's k), λ = forage_i × (−ln(0.01) / (20 × 82^1.5)) ×
  (food available)^1.5. The `-layout` text garbles the minus sign as "2"; the render confirms "−ln(0.01)".
  At 82 items, λ = 0.2303, P = 0.2057, and (1 − P)^20 = 0.0100, as printed.
- **Breeding** (pp. 665–666): "the program randomly picked a mother and father from among surviving
  individuals for each of 20 offspring. The probabilities of being picked were proportional to the amount of
  leftover stores (both scatter hoards and larder hoards)". Inheritance is X′ = N[h²X_p + (1 − h²)X̄, V_seg],
  and "We applied the logit transformation to these values to restrict them to the range between 0 and 1."
- **Food schedule** (p. 665): "82 items on day 1, 81 on day 2, 79 on day 3, . . ., 2 on day 50 (these
  numbers were based on a linear function but rounded off to integers)". The function isn't given. Five
  distinct rounded sequences fit all four printed values and sum to 2 100. **Choice:** day d gets
  ⌊82.3 − 1.645(d − 1) + 0.5⌋ items (d = 1..50). Reason: the line through the printed endpoints, (1, 82) and
  (50, 2), gives 80 on day 2, so it can't be V&J's. This one is a line that matches every printed value, with
  round-half-up.

**Corrections to the sections above.**

- *The model* says that an unfed agent eats from its stores "if it has any". It does, but only in bout 1
  (see item 2).
- *The model* gives the defense chance as logistic(fullness / target). Its midpoint is at *half* the
  target, not at the target (item 3).
- *The model* says "A fed agent with a larder defends". In fact every satiated agent makes a defense draw
  (item 3).
- *The model* says an agent "forages to eat". That applies to days 6–100 only. On days 1–5 every agent is
  satiated all day (item 5).

**Contradictions and errors in the paper, and how we handle them.**

1. **Clark & Mangel.** V&J write (p. 666): "note that the version of this equation in Clark and Mangel
   [2000, equation 10.35] is incorrect". We implement V&J's printed equation and don't consult Clark &
   Mangel. Their version is not a switch.
2. **Larder apparency: per burrow or per item?** P. 662 defines app_lard as "the relative probability of
   detecting a burrow of another individual while searching for food". The Appendix (p. 665) weights items,
   not burrows: "all larder hoards of other individuals weighted by apparency of larder hoards". Only the
   Appendix gives a formula. **Default:** per item, following the Appendix. **Possible switch:**
   `hoard.larder_weight = per_burrow`, which weights each non-empty larder once. This one could decide the
   threshold (see Concerns).
3. **"186 % per day"** can't be a daily proportion. See item 10.
4. **"Mean of about 0.15" versus a "logistic distribution".** A logit-normal distribution's mean isn't the
   inverse logit of its center. See item 8.
5. **The calibration day.** P. 665 calibrates the search on "the first day" and says that early starvation
   "happened occasionally by failure to find food early in the hoarding season". But no one needs storable
   food on days 1–5 (item 5), so a day-1 total failure kills no one. We implement both statements as
   printed. Early starvation can only start on day 6.

### 1. Order within a bout

(a) p. 665: "individuals that were not defending their burrows were selected for foraging in random order".
Predation comes first: "For each bout, the program called a subroutine that determined whether each
individual was preyed upon. Then …". The order of the defense draws is silent.

(b) **Choice:** within each bout, in this order:
1. bout-1 eating (first bout of the day only), with each agent in index order;
2. defense draws for every living, satiated agent, in index order;
3. predation draws for every living agent, in index order;
4. a fresh shuffle of the living agents that aren't defending, each of which then takes one foraging turn.

Every search sees the pools as the agents before it in the shuffle left them. Reason: this is the text's
order, with the unordered steps fixed to index order for determinism. The defense and predation draws are
independent, so their relative order doesn't change the distribution.

### 2. "The first two options took priority"

(a) p. 664: "The first two options took priority; i.e., until an individual fed once each day, it did not
store additional food." p. 664: "Once an individual had met its daily requirement, which often happened in
the first bout early in the hoarding season, then either it stored new items found while foraging on
subsequent bouts during the same day or it remained in its burrow." For bout 1 and later bouts, see "Methods
as read".

(b) **Choice**, for days 6–100:

- **Bout 1.** An agent with any stores eats one and is satiated. If it has a larder item, it eats that and
  defends for the whole bout, with probability 1. If it has only scatter items, it eats one and forages to
  store (defense probability 0).
- **An agent with no stores** stays hungry and forages. The first item it finds, by search or raid, is
  eaten.
- **Bouts 2–20.** A hungry agent always forages; it never defends. A satiated agent makes a defense draw
  (item 3). If it doesn't defend, it forages, and anything it finds is stored.

A hungry agent can't hold stores after bout 1, because it eats its first find. So "eat from stores"
happens only in bout 1.

On days 1–5 (item 5), every agent is satiated all day and never eats from stores. Bout 1 uses the ordinary
defense draw.

Reason: this is the Appendix's own procedure, taken literally.

### 3. Defense

(a) What the paper says:

- p. 664: "We defined probability of burrow defense as an increasing logistic function of how full the
  burrow was. Probability of burrow defense was 0.5 when the amount of food stored in the burrow was half
  of the target amount of food larder hoarded, where the target value was between the minimum amount needed
  to survive the rest of the hoarding season without further foraging and the maximum amount available if
  the animal foraged successfully in all its remaining foraging bouts."
- p. 665: "0 implied that the target was the minimum amount needed to survive, and 1 implied that the
  target was the maximum amount available".
- It's per bout. P. 665 gives defense probabilities "for all bouts after the first daily bout", and
  defending agents are excluded from foraging "for each bout".
- The logistic's slope is **silent**.

(b) **Choices:**

- **The logistic.** p = 1 / (1 + e^(−s·(x − 0.5))), where x = (larder items) / target. The slope is
  s = 10, so p is 0.007 when the larder is empty, 0.5 at half the target and 0.993 at the target. Reason:
  s = 10 is the round slope that takes an empty larder to "almost never" and a full one to "almost always".
  It's exposed as `hoard.defense_slope`, a named switch that isn't judged.
- **The minimum on day d** (satiated today): min = 100 − d, one item for each remaining day.
- **The maximum:** max = the foraging bouts left in the season, counting the current one:
  (20 − b + 1) + 20·(100 − d) for bout b. Reason: "all its remaining foraging bouts", with one item per
  successful bout.
- **The target:** target = max(1, min + D·(max − min)). The floor of 1 avoids dividing by zero in the
  season's last bout. That's a stated choice, not a result lever.
- **When the draw is made.** Every living, satiated agent draws once per bout (bouts 2–20, and every bout
  on days 1–5), including agents with an empty larder (p ≈ 0.007). Reason: the text gives the rule for
  "individuals with state = 0", without exception.
- **What defense does.** A defending agent doesn't forage, and its larder can't be pilfered that bout. A
  forager whose search draws a defended larder gets nothing that bout. Reason: the text never says what
  happens when a search lands on a defended burrow. Defense has to stop theft, and the Appendix already
  applies that rule to raid continuations.
- **Predation.** It is the same in or out of the burrow, 0.0001 per bout (p. 665: "set at 0.0001 in the
  open and in the burrow"), so defending carries no predation benefit.

Note that with this target the early-season targets are large. On day 6, with D = 0.5, the target is
about 1 000 items. So satiated larder owners rarely defend except in bout 1 until their larders are large or
the season is late. That follows from the text as written (see Concerns).

### 4. Raids

(a) What the paper says:

- p. 662: a pilferer "continues to do so until the end of 1 day of foraging or until the owner returns to
  defend the larder hoard".
- p. 665: raiders from "the previous bout" are checked for success "at the same burrow during the current
  bout, based on whether larder-hoarded items remained in the burrow and whether the owner was defending
  it".
- Items per bout: silent. Several raiders at one burrow: silent.

(b) **Choices:**

- **Items per bout.** A raid takes one item per bout: the search takes one item, and each continuation
  takes one.
- **Continuing.** An agent that took a larder item in bout b goes back to that burrow in bout b + 1, if it
  is foraging then. The continuation succeeds, with no detection draw, if the larder is non-empty and its
  owner isn't defending that bout.
  - If it fails, the raid ends and the agent searches normally in the same turn.
  - If the raider itself defends or dies, the raid ends.
  - Raids end at the end of the day.
- **"The owner returns"** means the owner's defense draw comes up "defend" in some bout.
- **Several raiders** may raid one larder at once, each taking one item per bout in shuffle order, until
  it's empty. Nothing in the text forbids it.
- **The item** goes to the raider: eaten if the raider is hungry, otherwise stored by its L draw.

Reasons: "harvested an item … during the previous bout" means one item per bout, with the check at the
start of the raider's turn. A failed check leaving the agent free to search matches "Finally, the program
used a subroutine to simulate the searching process". **Possible switch:** a failed continuation uses up
the bout.

### 5. Food available

(a) What the paper says:

- p. 665: "The total number of food items available to individual, i, was the sum of public food plus all
  scatter hoards of other individuals weighted by apparency of scatter hoards plus all larder hoards of
  other individuals weighted by apparency of larder hoards." The item is chosen from "public items, scatter
  hoards of individual j for all j ≠ i, and larder hoards of individual j for all j ≠ i, with the latter
  categories weighted by apparencies".
- p. 665: "individuals had alternative nonstorable food items (e.g., insects) available for days 1–5 but
  relied on storable items (e.g., seeds) for immediate consumption as well as storage from days 6–100".
- p. 665: states are reset to hungry only "For days 6–100".
- The stores of dead agents: silent.

(b) **Choices:**

- **Weighting.** Food available = public + app_scat·Σ_{j≠i} scatter_j + app_lard·Σ_{j≠i} larder_j, in
  weighted items. It sets λ and the draw. The agent's own stores are excluded.
- **Defended larders stay in the pool.** They are still "larder hoards of other individuals". Drawing one
  yields nothing (item 3).
- **Nonstorable food isn't modeled** as items. On days 1–5, every agent is satiated from the day's start,
  never eats from stores and stores everything it finds. The public pool holds only storable items. Reason:
  that is what "set at −1 … for days 6–100" does.
- **The dead.** A dead agent's stores stay in the world and stay pilferable. Reason: they are still hoards
  "of other individuals", and nothing removes them. The dead never defend, so a dead agent's larder is
  always open. **Possible switch:** `hoard.dead_stores = remove`.
- **A new generation** starts with empty stores and an empty public pool.

### 6. Eating from stores

(a) p. 665: "If individuals had only larder hoards or both larder hoards and scatter hoards, they used
a larder-hoarded item …; if individuals had only scatter hoards, they used one of these." No search step is
described for either kind. They "were assumed to use one of these items".

(b) **Choice:** larder first, then scatter. Recovery is free and certain for both. Reason: that's the text.
`hoard.owner_recovery` (below 1) is our switch, and it applies to scatter only. Under it, an owner with only
scatter items succeeds with probability `owner_recovery`, and on failure it stays hungry and forages. Larder
recovery stays free, because the food is in its own burrow.

### 7. Starvation

(a) p. 665: individuals died "because they were not able to acquire a meal each day". The timing is
silent.

(b) **Choice:** at the end of each day from day 6 on, every living agent still hungry (state −1) dies
(state 999). Its stores stay (item 5). There's no starvation on days 1–5. Predation kills at the moment it's
drawn. Reason: "a meal each day" is judged when the day ends.

### 8. Reproduction

(a) What the paper says:

- Pairing (p. 665): "randomly picked a mother and father from among surviving individuals for each of 20
  offspring. The probabilities of being picked were proportional to the amount of leftover stores".
- p. 666: "X̄ was the mean value in the parental population".
- The logit, p. 666: "We applied the logit transformation to these values".
- p. 665: the run's inputs include "average probability of larder hoarding, average propensity for burrow
  defense in the starting population of 20 individuals, and segregation variances for these last two
  parameters". L "followed a logistic distribution with a mean of about 0.15", and D one "with a mean of
  about 0.50".
- V_seg's value: silent. Whether one agent can be both parents: silent.

(b) **Choices:**

- **Pairing.** Mother and father are drawn independently, with replacement, in proportion to leftover
  stores (scatter + larder) among survivors. The same agent may be drawn twice. Reason: that's the simplest
  reading, and it works when only one agent survives. **Possible switch:** distinct parents.
- **The logit scale.** Inheritance is computed on it: logit X′ ~ N(h²·mean(logit X_mother, logit X_father)
  + (1 − h²)·X̄, V_seg), where X̄ is the mean logit over all 20 members of generation t, dead ones included.
  Reason: "parental population" means the whole generation. The pre-selection mean is what the breeder's
  equation regresses to.
  **Possible switch:** the survivors' mean.
- **V_seg** = 0.5 on the logit scale (SD ≈ 0.71) for both traits. Reason: the paper says only "a moderate
  amount of genetic variation". 0.5 is a middling round value, not fitted to anything. It is exposed as
  `hoard.v_seg` and should be swept (see Concerns).
- **Founders** are drawn as logit L ~ N(logit 0.15, V_seg) and logit D ~ N(logit 0.50, V_seg). So founders
  use the same segregation variance, which the paper lists alongside the averages. "Logistic distribution"
  is read as logit-normal, and the center is placed at logit(0.15). Its arithmetic mean comes out near 0.17,
  within "about 0.15". The realized founder mean is reported.
- **The clamp.** Values are clamped to (ε, 1 − ε), with ε = 1e−6, before taking the logit (Task 4).

### 9. Traits

(a) Two things are heritable. P. 665: "Heritability was fixed at 0.80 for both probability of larder
hoarding and propensity to defend the burrow". P. 664: "Each individual was characterized by two traits".
Foraging efficiency is drawn fresh. P. 665: "The model began each generation by initializing a set of arrays
for … the foraging efficiency of each individual", "drawn from a normal distribution with mean = 1 and
SD = 0.1".

(b) **Confirmed:** L and D are heritable. forage_i ~ N(1, 0.1) is drawn fresh for each agent in each
generation and not inherited. It is floored at 0, since λ can't be negative. That's a stated guard, and it
is essentially never hit.

### 10. Loss rates (the 186 % figure) and the CV

(a) What the paper says:

- p. 663: "we examined the rate of loss of scatter hoards and larder hoards within this time" (the first 10
  generations). Larder items "were lost at a much greater rate (mean = 186% per day) than were scattered
  caches (24%/day)".
- p. 663: "the coefficient of variation in rate of loss of larder-hoarded items (57%) was almost twice that
  of scattered caches (33%). This was a consequence of the fact that an individual's larder was vulnerable
  to catastrophic loss".
- The definition of "daily rate of loss" is **silent**. Table 1's percentages are geometric daily
  proportions ("the rate of removal over the duration of a study was constant"). For example, 423 of 500
  over 188 days gives 1 − (77/500)^(1/188) = 1.0 %, as printed. That kind of rate can't exceed 100 %, so
  the model's 186 % uses some other definition.

**Definitions that could give 186 % for larders and 24 % for scatter.** Given this model:

- **Items lost ÷ mean items held,** per day (a per-item hazard, with the stock averaged over bouts or days).
  It exceeds 100 % when items move through the larder faster than they sit in it. Larder items are added
  and stolen within a day, because the raider takes one per bout, while the stock averages only a few
  items. Scatter stock is large and loses slowly, so it would sit well below 100 %.
- **Losses ÷ the store at the start of the day** (or at the start of a raid), averaged over days. This
  exceeds 100 % whenever a larder that was small in the morning is refilled and stripped during the day.
  Days with a tiny denominator make the mean explode and skew it right. That fits a CV of 57 %.
- **Per-bout percentages summed over a raid.** A raid that empties n items one per bout sums
  1/n + 1/(n − 1) + … + 1 = H_n: 228 % for n = 5, 293 % for n = 10. Emptied larders alone would put the
  mean above 100 %.
- **An instantaneous (exponential) rate,** −ln(fraction surviving a day). 186 % would mean 84 % of a
  larder lost a day, and 24 % would mean 21 % for scatter. It is undefined for a larder emptied in a day
  unless capped.

**The geometric proportion** (Table 1's definition) is the only common definition that is ruled out.

(b) **Choice: per-item daily hazard.** For each agent and each store type:

- rate = (items of that type taken from it by other agents while it is alive) / (mean items of that type
  it held at the start of each bout while alive × days alive).
- Equivalently, rate = losses ÷ (Σ over bouts of the stock at the bout's start ÷ 20).
- It is computed only for agents whose mean stock of that type is above zero. Agents that lost items but
  held none at any bout start are counted and reported, not dropped silently.
- Own consumption isn't loss.

Reasons:

- It is the literal "items lost per day ÷ items stored".
- It is defined on a bout grid, so it doesn't blow up on tiny morning stocks.
- It exceeds 100 % exactly when items move through the store faster than they sit, which is the mechanism
  the paper names ("catastrophic loss").

The start-of-day ratio and the instantaneous rate may be logged as unjudged diagnostics.

**Aggregation.**

- The run's "mean larder loss" is the mean of the per-agent rates over all agents, pooled across
  generations 1–10, and likewise for scatter.
- **CV** = SD / mean of those per-agent rates within a run (population SD), averaged across runs. Reason:
  the paper ties the CV to "an individual's larder", so the variation is among individuals.
  **Alternative:** the CV across runs of the run means.
- **Minimum larder rate** (claim 4) = the minimum per-agent larder rate in generations 1–10.
- **Average scatter rate** (claim 4) = the pooled mean above.

**How the survey judges it** is separate from how it's measured. The survey judges only:

- that larder loss is above scatter loss in every run;
- the CV ratio (larder CV / scatter CV) against 57/33 ≈ 1.7, "almost twice".

186 % and 24 % are reported beside our values, and never judged, because the paper's definition is unknown.

### 11. "Larders take over"

(a) What the paper says:

- The Fig. 2B caption, p. 662: "Filled circles represent cases in which larder hoarding became
  established by the last 10 generations".
- p. 663: "average probability of larder hoarding always remained less than 0.2 or increased rapidly to
  more than 0.95, usually within 10 generations".
- The exact statistic and generation: silent.

(b) **Choices:**

- **Takeover** is when the population's mean L (over the agents born into the generation), averaged over
  generations 51–60, is above 0.95.
- **The rise generation** is the first generation whose mean L is above 0.95.
- **"Stayed low"** means mean L < 0.2 in every one of generations 51–60.
- Runs meeting neither condition are "intermediate", which counts against claim 1.

Reason: the caption's "by the last 10 generations" names the window, and the text's 0.95 names the level.
**Possible switch:** judge only generation 60. A run whose population dies out is recorded as extinct, with
the generation it happened (Task 4 decides what "dies out" does). It isn't counted as a takeover.

### 12. Default apparencies

(a) p. 662: "we used values between 1 and 3", for app_lard. p. 665: "We varied apparencies of scatter hoards
from 0.05–0.90 and apparencies of larder hoards from 1–3". The Fig. 2B fit, logit = −8.00 + 36.59 × ratio,
has its 50 % point at 8.00 / 36.59 = 0.2186. The Fig. 2A examples used app_lard = 1.0. The runs'
combinations are silent.

(b) **Choice:** the default preset uses `app_lard` = 2.0 and `app_scat` = 0.44, for a ratio of 0.22. Reason:
it sits at the reported 50 % point, and app_lard is mid-range. This replaces the table's placeholder
app_scat of 0.5 (ratio 0.25). The survey's ratio grid spans app_lard over {1, 2, 3}, so the threshold is
tested at more than one absolute apparency.

### New fields from these choices

These are all named switches whose defaults are the choices above. None is judged against V&J.

| Field | Default |
|---|---|
| `hoard.defense_slope` | 10 |
| `hoard.v_seg` | 0.5 |
| `hoard.larder_weight` | `per_item` (alternative `per_burrow`) |
| `hoard.dead_stores` | `remain` (alternative `remove`) |

### Concerns: gaps likely to decide the results

1. **Larder weighting: per item or per burrow** (contradiction 2).
   - Weighting per item makes a big larder more findable in proportion to its size.
   - Weighting per burrow doesn't.

   That changes the larder-to-scatter detection ratio, which is exactly what the threshold measures. The
   default follows the Appendix. The switch should be run before any reading of claim 2.
2. **The defense target's scale.** A literal "minimum needed to survive" and "maximum available" give
   targets in the hundreds early in the season. So defense after bout 1 is rare until late in the season.
   Larder security then rests mostly on the bout-1 rule. That rule forces a defense only for agents with a
   larder item.
3. **V_seg** is unstated. It sets the founders' spread of L, and so the best early larder that claim 4 is
   about. It should be swept, and the answer reported as a sensitivity, not tuned.
4. **The stores of the dead.** Leaving them (the default) gives pilferers undefended larders late in the
   season. Removing them doesn't. That may shift larder loss rates and takeover.
