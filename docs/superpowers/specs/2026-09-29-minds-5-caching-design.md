# Minds 5: caching for the future (design)

**Date:** 2026-09-29
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 5. This is our own experiment, numbered
apart from the reproductions' milestones.
**Builds on:** the milestone specs, and Minds 1–4. All remain binding where not changed here:
- the decision seam;
- walking and A* (Minds 2);
- memory and belief (Minds 3);
- GOAP and the marginal-value rule (Minds 4);
- the literal-default-plus-named-switch pattern;
- the titles rules;
- the Minds rule that every engine reduces to what came before.

**Out of scope** (a future campaign): pilfering, being watched, re-caching, and anything else that
needs one Flump to perceive another's caching. Also out: caches that rot, and GOAP over several
goods.

**Sources** (local copies in `papers/caching/` and `papers/foraging/`):
- N. J. Raby, D. M. Alexis, A. Dickinson and N. S. Clayton, "Planning for the future by western
  scrub-jays", *Nature* 445 (2007), 919–921.
- P. Amodio, J. Brea, B. G. Farrar, L. Ostojić and N. S. Clayton, "Testing two competing hypotheses
  for Eurasian jays' caching for the future", *Scientific Reports* 11 (2021), 835.
- D. W. Stephens and J. R. Krebs, *Foraging Theory* (1986): §3.5 (central-place foraging, after
  Orians and Pearson 1979) and §9.4–9.5 (Kacelnik and Houston 1984; Kacelnik and Cuthill 1986).
- The program document's verified caching sources (Vander Wall and Balda 1977; Balda and Kamil
  1992; Vander Wall and Jenkins 2003) for context only; no claim is judged on them.

## Goal

Give Flumps a carrying limit, caches they bury and dig, and a winter everywhere at once. Build three
caching rules, each the mechanism one hypothesis names. Run each rule through the two laboratory
protocols whose published results disagree, and ask whether the rules leave their predicted
signatures and which rule's signature the birds' data resemble. Then put the same rules in the
field and ask which one gets a Flump through winter. Test central-place foraging, which the same
mechanics make possible, against its textbook predictions.

## Non-negotiable constraints

- **Earlier work unchanged.** Every golden entry, legacy fixture and pinned fingerprint stays
  green and unedited. The new rules and switches are off by default and draw nothing new.
- **Reduction.** With `caching.rule: none` and no carrying limit, every world is its Minds 4
  self, bit for bit. Global winter with divisor 1 equals no seasons.
- **Deterministic.** The caching rules draw nothing. The lab harness draws only through the world's
  existing draws.
- **Oracles before claims.** Each rule must reproduce its hypothesis's hand-derived prediction in
  both protocols. That's a test; a rule that fails it is a bug, not a finding.
- **Judges before runs.** Survey judges and thresholds are committed before any survey result, and
  never tuned.
- **Truthful titles and descriptions.** Results are ours, and failures are reported plainly. Before
  crediting a mechanism, check how often it's used, and whether a result rests on who survives.

## Source summary

- **Raby et al. (2007), "planning for breakfast":**
  - Eight birds, housed in three adjoining compartments A, B, C. "each bird was shut in compartment
    A or C on alternate mornings for two hours". One compartment always had breakfast (powdered,
    non-cacheable pine nuts); the other had none.
  - After "three 'no-breakfast' training trials and three 'breakfast' training trials", they were
    unexpectedly given whole pine nuts in the evening, with access to all compartments.
  - They cached more in the no-breakfast compartment: 16.3 ± 1.8 against 5.4 ± 1.8 (mean ± s.e.m.;
    paired t₇ = 3.01, P = 0.02).
- **Amodio et al. (2021):**
  - Two hypotheses. The Compensatory Caching Hypothesis: birds "learn to cache more of a particular
    food in places where that food was less frequently available in the past". The Future Planning
    Hypothesis: birds "recall the 'what–when–where' features of specific past events to predict the
    future availability of food".
  - Their proposed compensatory mechanism: each location carries a weight; experiencing food at a
    location lowers that location's weight; caches are distributed by weight, "thus achieving a
    more uniform distribution of resources".
  - Experiment 2 (one food, present or absent): nine experience days. The compartment rotates on a
    3-day cycle (K1 on days 1, 4, 7; K2 on 2, 5, 8; K3 on 3, 6, 9); food is present or absent on a
    2-day cycle. Test on the evening of day 9, with caching allowed in all three.
    - Food-First group (food on odd days): "FPH 1 … will provision only for the next day, when it
      expects to be in compartment K1 with no food available; thus it would cache most in K1."
      "CCH … should cache more in compartment K2, where it has only once encountered the powdered
      food." FPH 2 (the next three days) "will distribute the caches across compartments K1 and
      K2". The Empty-First group's predictions are reversed.
  - Results: the compartment-independent (even) model had the highest posterior probability in
    both experiments: 0.997 in Experiment 1, 0.72 in Experiment 2 (CCH 0.16, FPH 0.002). Six
    birds were tested in each experiment.
- **Central-place foraging** (Stephens and Krebs §3.5, §9.5):
  - Multiple-prey loaders face a negatively accelerated loading curve; "patch residence time
    determines load size, and the marginal-value theorem can be applied". "The chief prediction is
    that increasing average distance from the central place should be matched by increasing load
    size."
  - Lima: the prediction applies to the average patch distance in the habitat, not to individual
    trips. A forager alternating between a near and a far patch with the same loading curve should
    take the same load from both.
  - With a linear loading curve the theorem predicts no effect of travel on load size, yet
    Kacelnik and Cuthill's starlings "take larger loads from more distant patches even when the
    gain function is linear".

## Architecture

- **`minds/caching/mod.rs`:** caches (per Flump), burying and digging, the carrying limit, and the
  `CachingRule` seam.
- **`minds/caching/episodes.rs`:** episodic memory (what, where, when) and the cycle finder.
- **`minds/caching/rules.rs`:** the three caching rules.
- **`minds/caching/lab.rs`:** the protocol harness (Raby; Amodio Experiment 2).
- **`minds/central.rs`:** central-place provisioning under the marginal-value rule.
- **Seasons:** `seasons.mode` gains `global`; the book's `hemispheres` stays the default.
- **The seam:** caching runs after the Flump's move and harvest in rule M's step, under any
  decision rule. Digging happens through foraging: a Flump's own caches join its candidate sites.

## Mechanics

- **Carrying limit** (`caching.capacity`, 0 = none, the default). A Flump never holds more than the
  limit; harvest beyond it stays on the site. Only good 0 (sugar) is cached; caching requires
  exactly one good, like GOAP and the marginal-value rule.
- **Caches.** Each Flump owns a map from site to amount (`BTreeMap<u32, f64>`, deterministic
  order). Nobody else sees or takes them. They don't decay. A Flump's caches die with it.
- **Bury(q)** at the Flump's current site: holdings −= q, cache += q. It costs no tick; a Flump may
  bury after harvesting in the same tick.
- **Dig** at a site with the Flump's cache: takes min(cache, room under the limit). It is the
  harvest of that tick (it replaces harvesting the site).
- **Caches as candidates.** When holdings are below the reserve (below), each of the Flump's caches
  joins its candidate sites (rule M, utility, GOAP, the marginal-value rule), valued at its amount.
  Arriving at one digs it.
- **Reserve.** R = metabolism × `goap.horizon` (H ticks of food, as in GOAP's goal). Surplus is
  holdings above R.
- **Conservation.** Sites + holdings + caches + eaten is conserved across bury and dig exactly.
- **Inheritance.** None; children start with no caches.

## The caching rules (`caching.rule`)

Every rule decides only when to bury, how much, and where. Digging is the same for all.

- **`none`** (default): never buries.
- **`even`** (the compartment-independent model): buries a fixed share of its surplus,
  `caching.share` (default 0.5), at its current site, every tick it has surplus. Over a path this
  spreads caches evenly across the places it passes.
- **`compensate`** (Amodio's weights): each known site (or compartment) has a weight w, starting at
  1. Each time the Flump finds food there (harvests > 0 on arrival), w ← w × (1 − λ), with
  `caching.lambda` default 0.5. It buries surplus × share × w / w̄ at its current site, where w̄ is
  the mean weight over the sites it knows (capped at the surplus).
- **`plan`** (the Future Planning Hypothesis): recall what-where-when and provision for predicted
  need.
  - Episodic memory records, per day (or per tick in the field), where the Flump was and whether it
    found food. Capacity bounded as Minds 3's memory is.
  - **The cycle finder:** for a sequence, the smallest period p ≥ 1 such that the sequence equals
    itself shifted by p, over the observed span. It predicts the next value by extrapolation. With
    no period shorter than the span, it predicts nothing.
  - **Prediction:** for the next `caching.lookahead` days (default 1, FPH 1; 3 gives FPH 2), where
    the Flump will be and whether that place will have food.
  - **Provision:** buries at each predicted place without food its need for that day (metabolism ×
    day length), up to its surplus, nearest predicted day first. In the lab it caches in the
    compartments; in the field it caches at sites it predicts it will occupy in winter (below).
  - When it predicts nothing, it buries nothing. This is a stated choice: planning needs
    experience.

## The labs (`minds/caching/lab.rs`)

Scripted trials on a small walled rig: three compartments K1–K3 opening onto a hall (Minds 2 walls).
The harness places the Flump and sets each compartment's sugar; no world rule is added. A lab day
is one morning (the Flump shut in one compartment, which holds food or none) and one evening; the
test evening gives the Flump F units of sugar in the hall and lets it cache in any compartment.

- **The test evening's allocation.** The Flump caches all F (its reserve is 0 in the lab) across
  the open compartments, walking to each:
  - `even`: F / k in each of the k compartments;
  - `compensate`: F × w_k / Σw;
  - `plan`: F split equally over the compartments it predicts will lack food on the days it looks
    ahead to; with no prediction, nothing (the stated choice above).

  Whole units: remainders go to compartments in K order. These make the oracles exact.
- **Raby:** two compartments (K1, K3) on alternate mornings for 6 days, three with breakfast and
  three without, counterbalanced for which comes first. F = 30.
  - Predictions: `plan` and `compensate` cache more in the no-breakfast compartment; `even` ties.
- **Amodio Experiment 2:** the schedule above, Food-First and Empty-First groups. F = 30.
  - Food-First predictions: `plan` (lookahead 1) caches most in K1; `compensate` caches most in K2;
    `even` ties.
  - `plan` with lookahead 3: derived by the cycle finder. If our derivation disagrees with the
    paper's stated FPH 2 pattern ("across compartments K1 and K2"), the spec is amended with both,
    and the disagreement is reported, not resolved by editing either.
- **Individual differences.** A lab population is N Flumps (default 8 for Raby, 6 for Amodio, as
  in the papers) with rule parameters drawn per Flump from narrow ranges (share, λ), through the
  world's existing draws. This gives the per-bird spread the observers saw.
- **Page:** one read-only preset per protocol showing the test evening, so the three rules' caches
  can be seen side by side.

## The field

- **Winter.** `seasons.mode: global` makes the whole map winter at once (growback ÷
  `winter_divisor`); `hemispheres` is the book's rule and the default.
- **The calendar is known.** Flumps know the season phase (day length stands in for it); `plan`
  uses it. Knowing the calendar is not knowing winter's yield: `plan` forecasts winter intake from
  its episodic record of its own last winter's intake per tick. Before its first winter it assumes
  zero winter intake (worst case). This is a stated choice, reported.
- **`plan` in the field:** at each tick of summer, the forecast shortfall is metabolism × winter
  length − forecast winter intake − current caches. It buries up to its surplus toward that
  shortfall, at its current site if that site is in its winter range (sites it occupied last
  winter), else anywhere before its first winter.
- **`even` and `compensate` in the field:** as defined above, every tick of either season. They
  have no forecast and bury surplus by share.
- **Worlds:**
  - one rule per world (`none`, `even`, `compensate`, `plan`), paired seeds;
  - a mixed world with a quarter of the population on each rule.
- **Balance.** Measured before any preset is recorded (as Minds 4 did): summer surplus per Flump
  ≥ 1.5 × its winter need, and winter regrowth < 0.5 × the population's winter need, so winter is
  lethal without caching and survivable with it. Adjust mechanically (winter divisor, then
  population) and record the numbers.

## Central-place foraging (`minds/central.rs`)

- **Home.** In a central-place world each Flump has a home site (where it is placed) and a larder
  there. It forages and brings loads home; delivering buries the load in the larder. It eats from
  what it carries and, at home, from the larder.
- **The rule.** The marginal-value rule over round trips: ρ is the long-run delivery rate (delivered
  per tick, travel counted). In a patch, the Flump leaves for home when its last tick's gain falls
  below ρ or its load reaches the carrying limit. Load size is what it delivers per trip.
- **Worlds:**
  - curved loading: patches whose sites run down as harvested (no regrowth during a visit), at mean
    distances d from home, one world per d;
  - near and far: two patches, one near and one far, in one habitat;
  - linear loading: sites that yield a fixed amount per tick without running down.
- **Predictions:** load size rises with d (curved); near and far get the same load (Lima); linear
  loading shows no distance effect below the limit (the theorem), which Kacelnik and Cuthill's
  starlings contradicted.
- **The planner.** GOAP with the goal "deliver G" on the same worlds; its load sizes are reported,
  not judged.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `caching.rule` | `none` | reset | `none`, `even`, `compensate`, `plan` |
| `caching.capacity` | 0 | reset | carrying limit (0 = none) |
| `caching.share` | 0.5 | live | share of surplus buried (`even`, `compensate`) |
| `caching.lambda` | 0.5 | live | weight lowered per food found (`compensate`) |
| `caching.lookahead` | 1 | live | days `plan` provisions for (1–10) |
| `seasons.mode` | `hemispheres` | reset | `hemispheres` or `global` |
| `central.enabled` | false | reset | homes, larders and delivery |

Validation: caching needs exactly one good; `central.enabled` needs `decision.rule: mvt` or `goap`
and a carrying limit.

## Statistics

When caching is on: `cached` (total in caches), `buried` and `dug` per tick, `recovery` (share of
all sugar ever buried that has been dug), and `mean_cache_age` at digging. Central-place worlds add
`mean_load` and `trips`.

## Presets

Titles are drafts.

| Preset | Setup |
|---|---|
| `cache-raby` | Raby's test evening, 8 Flumps, one per rule shown by tribe. |
| `cache-amodio` | Amodio Experiment 2's test evening (Food-First). |
| `cache-winter-none`, `-even`, `-compensate`, `-plan` | The field world, one rule each. |
| `cache-winter-mixed` | The field world, a quarter on each rule. |
| `central-near`, `central-far` | Curved loading at two mean distances. |
| `central-linear` | Linear loading. |

## Sweeps

- `cache-capacity`: survival through winter against the carrying limit, per rule.
- `cache-winter`: survival against winter length (the season period).
- `central-distance`: mean load against mean patch distance.

## Survey (`minds5` claims)

1. **Raby.** Per rule, in a lab population: caches in the no-breakfast compartment exceed the
   breakfast compartment's (paired per Flump). Reported beside 16.3 against 5.4.
2. **Amodio.** Per rule, the pattern (K1, K2 or even) matches its hypothesis's prediction for both
   groups. Then: fitting the three patterns (and the compartment-independent one) to the pooled
   Flump data, which one wins, and by how much. If the paper's Bayesian comparison can be written
   down exactly from its text, it's used; otherwise a stated multinomial likelihood comparison.
   Reported beside 0.997 and 0.72.
3. **Winter survival.** Per rule against `none`, paired on seeds: survival through the first
   winter and after five. `plan` judged against `even` and `compensate`. Recovery and cache age
   reported. The mixed world reported, with each rule's share of survivors.
4. **Central place.** Load rises with mean distance (per-seed slope > 0, as Minds 4's travel
   claim). Near and far loads equal within a habitat (paired difference judged against a stated
   tolerance). Linear loading reported.
5. **Usage.** How often each rule buries and digs, per world, and how much buried sugar is never
   dug. Survivorship: every survival or wealth claim also reported per founding Flump, dead as 0.

## Verification (tests, not claims)

- **Reductions:** caching off with no limit reproduces Minds 4's fingerprints; global winter with
  divisor 1 equals no seasons; hemispheres mode leaves the book's golden unchanged.
- **Mechanics:** conservation across bury and dig; the carrying limit caps harvest and leaves the
  rest on the site.
- **Lab oracles:** each rule's test-evening caches match the hand-derived prediction in both
  protocols and both Amodio groups.
- **Cycle finder:** the period found is minimal and consistent, checked against brute force on
  random sequences.
- **Central place, analytic:** on one patch with a known loading curve, the rule's load equals the
  tangent construction worked by hand.

## Page

- Caching controls: rule, capacity, share, λ, lookahead; seasons mode; central-place on/off.
- Inspect: holdings against the limit, the Flump's caches (count and total), `plan`'s forecast, and
  its home and last load in central-place worlds.
- The map draws the inspected Flump's caches.
- Charts: caching (cached, buried, dug, recovery) and central place (mean load, trips).

## Docs

README (a Minds 5 section), the program document (status, results, cost table, and the next
target), the roadmap line, and this spec's amendments.

## Amendments (implementation)

- **The lab is a world mode** (`lab: raby | amodio`, reset-only, validated to the rig), not a harness
  outside the world: one code path serves tests, survey and presets. It draws nothing new.
- **Amodio's FPH 2 doesn't follow from the schedule as we read it.** For the Food-First group the
  next three days are K1 without food (day 10), K2 with food (11) and K3 without food (12), so a
  planner provisioning for them caches in K1 and K3 (ours: 15/0/15). The paper states FPH 2 as
  "distribute the caches across compartments K1 and K2". We keep our derivation and score both
  patterns in the survey. For the Empty-First group ours (0/30/0) matches the paper's statement.
- **Provisioning for tomorrow caches nothing when tomorrow has food.** `plan` with lookahead 1 does
  that for Amodio's Empty-First group and for Raby's breakfast-first counterbalancing. The paper's
  FPH 1 reads as "the next day without food"; the survey scores the paper's statement beside ours.
- **Whole units:** remainders go in K order, so Raby's `compensate` is 27/3 with breakfast in K3
  and 4/26 with breakfast in K1; the survey pools both counterbalancings.
- **Individual differences** come only from `compensate`'s λ in the lab (all F is cached, so
  `share` is unused); `even` and `plan` populations have no spread.
