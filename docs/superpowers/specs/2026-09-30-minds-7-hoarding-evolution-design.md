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
