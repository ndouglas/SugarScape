# Minds 4: GOAP and the marginal value theorem (design)

**Date:** 2026-09-28
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 4. This is our own experiment, numbered
apart from the reproductions' milestones.
**Builds on:** the milestone specs, and Minds 1–3. All remain binding where not changed here:
- the decision seam;
- walking and A* (Minds 2);
- memory, belief and truffles (Minds 3);
- the literal-default-plus-named-switch pattern;
- the titles rules;
- the Minds rule that every engine reduces to what came before.

**Sources** (local copies in `papers/planning/`, `papers/foraging/`, `papers/caching/`):
- J. Orkin, "Three States and a Plan: The A.I. of F.E.A.R.", GDC 2006.
- J. Orkin, "Applying Goal-Oriented Action Planning to Games", in *AI Game Programming Wisdom 2*
  (2004) (draft).
- J. Orkin, "Symbolic Representation of Game World State: Toward Real-Time Planning in Games", AAAI
  workshop WS-04-04 (2004).
- R. E. Fikes and N. J. Nilsson, "STRIPS: A New Approach to the Application of Theorem Proving to
  Problem Solving", *Artificial Intelligence* 2 (1971), 189–208.
- M. Helmert and R. Mattmüller, "Accuracy of Admissible Heuristic Functions in Selected Planning
  Domains", ICAPS 2007 workshop; M. Helmert and G. Röger, "How Good is Almost Perfect?", AAAI 2008.
- J. Slaney and S. Thiébaux, "Blocks World revisited", *Artificial Intelligence* 125 (2001), 119–153.
- S. M. Constantino and N. D. Daw, "Learning the opportunity cost of time in a patch-foraging task",
  *Cognitive, Affective, & Behavioral Neuroscience* 15(4) (2015), 837–853.
- The marginal value theorem's sources are as in Minds 3: Charnov 1976; Stephens and Krebs 1986;
  Hayden, Pearson and Platt 2011; Nonacs 2001.
- For Minds 5, not used here: Raby, Alexis, Dickinson and Clayton, "Planning for the future by
  western scrub-jays", *Nature* 445 (2007); Emery and Clayton 2001; Amodio et al. 2021.

## Goal

Add GOAP, the first planner in the program, and verify it against planning domains whose optimal
plan lengths are known. Give it a foraging domain in which travel costs the time it takes, so the
cheapest plan to gather enough food is the fastest one. With it, run the marginal value theorem
test that Minds 3 couldn't, and answer Minds 3's open question: does memory pay a mind that prices
travel?

## Non-negotiable constraints

- **Earlier work unchanged.** Every golden entry, legacy fixture and pinned fingerprint stays
  green and unedited. The new rules and switches are off by default and draw nothing new.
- **Deterministic.** The planner draws no random numbers. Its only draw is the one `choose` makes
  when two first steps tie.
- **Bounded cost.** Planning is limited by K (the sites it plans over) and an expansion limit. The
  cost per Flump per tick is reported.
- **Truthful titles and descriptions.** Results are ours, and failures are reported plainly. The
  lesson carried from Minds 3: before crediting a mechanism, check how often the treatment is
  actually used.

## Source summary

- **GOAP** (Orkin 2006):
  - "F.E.A.R. most closely resembles the STRIPS planning system from academia. … We added a cost
    per action, eliminated Add and Delete Lists for effects, and added procedural preconditions and
    effects."
  - "the nodes are states of the world, and we are searching to find a path to the goal state. The
    edges connecting different states of the world are the actions."
  - Replanning (2004 AAAI): "We only formulate a new plan when the current plan has been
    invalidated, or the most relevant goal has changed."
  - Regressive search (2004 draft): "A regressive search is more efficient and intuitive."
- **STRIPS** (Fikes and Nilsson): an action is preconditions, an add list and a delete list.
- **Known optimal plan lengths:**
  - Gripper (two grippers, n balls): 3n − 1 for even n, 3n for odd n (Helmert and Mattmüller:
    "h*(sₙ) = 3n − 1").
  - Logistics families: 4n (one truck per city, one package each; or one truck moving n packages
    in one city).
  - A blocks-world family (the 4-operator encoding): 4n − 2.
  - Blocks world in general: optimal planning is NP-hard, and near-optimal within a factor of 2 is
    tractable (Slaney and Thiébaux).
- **The marginal value theorem learned** (Constantino and Daw 2015):
  - People harvest while "κsᵢ ≥ ρh". Here ρ is a running average reward rate, updated as
    ρ ← ρ + [1 − (1 − α)^τ]·(r/τ − ρ). This rule beat temporal-difference learning in model
    comparison (exceedance probability .999).
  - Overharvesting was "qualitatively consistent with the MVT, though potentially with a slight
    bias to overstay". It was significant only with long travel times (t₉ = 2.3, p = .045).

## Architecture

- **`crates/sugarscape-core/src/minds/goap.rs`:** a generic GOAP planner.
  - A domain supplies states, actions (preconditions, effects, cost) and an admissible heuristic
    for its goal.
  - The planner runs Minds 2's `astar` over the state graph, with states hashed to indices by the
    domain.
  - Stated departure from Orkin: the search runs **forward**, not backward. Our foraging states
    hold amounts of sugar, and regression over numeric effects is awkward. Orkin's regression is an
    efficiency choice for symbolic states, not part of GOAP's definition.
- **`minds/goap/strips.rs`:** a STRIPS encoding (facts, add and delete lists, action costs) for
  verification: gripper, the logistics families and the blocks world.
- **`minds/goap/forage.rs`:** the foraging domain.
- **`minds/mvt.rs`:** the marginal-value rule.
- **The seam:** `DecisionRule` gains `Goap` and `Mvt`. Both require `movement.mode: walk`, and both
  end in `movement::arrive`.

## The foraging domain

- **Candidates.** The Flump's best **K** known sites by believed value (default 8). They're drawn
  from sight and, for rememberers, from memory (Minds 3's believed values). Its own site is always
  included. Ties go to the nearer site, then the lower site index.
- **State.** Where the Flump is (its own site or one of the K), which of the K it has harvested
  (a bitmask), and the sugar gathered so far. Cost is ticks.
- **Actions.**
  - `Harvest(i)`: walk to site i and harvest it.
    - Cost = the lattice (torus Manhattan) distance + 1.
    - Precondition: i isn't yet harvested in this plan.
    - Effect: position i, i marked harvested, gathered += its believed value.
  - `Harvest(here)` costs 1.
  - Regrowth during a plan is ignored: plans are short against regrowth. This is a stated choice.
- **Goal.** Gathered ≥ G, where G = metabolism × `goap.horizon` (H ticks of food; default H = 10).
  - Heuristic: ⌈(G − gathered) / v_max⌉, where v_max is the most valuable unharvested candidate.
    Each action costs at least 1 tick and gains at most v_max, so this is admissible.
  - If no plan reaches G (not enough known sugar), the Flump takes the single candidate with the
    best value ÷ (distance + 1) instead. That's a rate choice, stated.
- **Executing.** The plan's first step is the target, and the Flump walks there with `arrive`. It
  keeps its plan (`Agent.goap_plan`, observational for Inspect but also behavioral) until the plan
  is invalidated:
  - the next target is occupied, found emptier than believed on arrival, or unreachable;
  - or the plan is finished.

  Then it replans, following Orkin.
- **Expansion limit.** 4 096. Beyond it the plan falls back to the rate choice above.
- **Ties.** Among equal-cost plans, the first steps are passed to `choose` (rule M's tie rule and
  draw).

## The marginal-value rule (`decision.rule: mvt`)

Constantino and Daw's rule at site level:
- Each Flump keeps ρ, a running mean of its gain per tick. It's updated every tick, travel ticks
  counting as 0: ρ ← ρ + α(gain − ρ), with `mvt.alpha` defaulting to 0.05.
- **Stay** while the best site within distance 1 of it (including its own) believes to yield ≥ ρ.
  It harvests there.
- **Otherwise leave:** walk to the best known site by believed value, committed until arrival.
- A new Flump starts with ρ equal to its metabolism.

This is the theorem's rule, learned; it's the baseline GOAP is compared against.

## Knowing the map (`memory.prior`)

`memory.prior: none` (the default) or `map`. Under `map`, rememberers start with every non-wall site
in memory, seen at tick 0 with its starting levels: the theorem's "ideal" forager. This needs
`memory.span > 0`, and the span still applies (set it long).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `decision.rule` | `book` | reset | adds `goap` and `mvt` (both need walking) |
| `goap.k` | 8 | live | known sites a plan considers (1–12) |
| `goap.horizon` | 10 | live | ticks of food the goal asks for (1–100) |
| `mvt.alpha` | 0.05 | live | the rate estimate's learning rate (0–1, exclusive of 0) |
| `memory.prior` | `none` | reset | `none` or `map` |

## Statistics

When the rule is `goap` or `mvt`, add these series:
- `replans`: the share of Flumps that planned this tick (GOAP), or that left (MVT);
- `mean_plan_length`: GOAP only;
- `mean_rate`: MVT only, the mean ρ.

Residence and overstaying are computed in the survey, as in Minds 3.

## Presets

Titles are drafts.

| Preset | Setup |
|---|---|
| `goap-mvt` | The marginal value theorem's world. 9 patches (peaks, radius 3, height 4) on a 3s × 3s torus with s = 20. 5 Flumps with metabolism 1, walking, GOAP, memory (span 1 000, share 1, `project`, prior `map`), vision 1–6, growback 0.05. The design principle: each patch runs out under one forager, and total regrowth exceeds the population's need. The plan's first task measures and records both. |
| `mvt-rule` | The same world under the marginal-value rule. |
| `goap-open` | Minds 3's `mem-open` (share 0.5) under GOAP. |
| `goap-truffles` | Minds 3's `mem-truffles` under GOAP. |
| `goap-walled` | Minds 3's `mem-walled` under GOAP. |

## Sweeps

- `goap-horizon`: `ii-2-unit`'s population (walking, GOAP, ticks 300–500) against H (2, 5, 10, 20, 40).
- `goap-k`: the same against K (2, 4, 8, 12).
- `goap-memory`: `wealth_advantage` in `goap-truffles` against memory share (0.1–0.9), for comparison
  with Minds 3's `mem-share`.

## Survey

A `minds4` claims module, with pairing as in Minds 3.

1. **Travel time** (Charnov; Stephens and Krebs; Hayden et al.): on the 3s × 3s tori with
   s = 12, 16, 20, 24, the mean patch residence rises with s.
   - Judged per seed as a slope > 0, for GOAP and for the marginal-value rule.
   - Rule M with the same knowledge is reported.
   - Every seed with no fit is reported, with its alive count.
2. **Overstaying** (Nonacs; Constantino and Daw): at each departure, the gain on the Flump's last tick
   in the patch against its long-run mean gain rate.
   - The share that overstays is reported for GOAP and the marginal-value rule.
   - Direction and size are compared with the rule and with the travel time.
3. **Minds 3's open question:** does memory pay a mind that prices travel?
   - The rememberers' wealth advantage under GOAP in `goap-open`, `goap-truffles` and `goap-walled`,
     judged paired within each world.
   - Reported beside Minds 3's rule M values.
   - The share of GOAP rememberers' plans that include a remembered out-of-sight site is reported
     too, the usage check Minds 3 taught.
4. **Horizon and K:** how the population depends on H and K. Reported, not judged.

## Verification (tests, not claims)

- **STRIPS domains:** GOAP's optimal plan lengths equal:
  - gripper, 3n − 1 or 3n, for n = 1–8;
  - the logistics families, 4n, for n = 1–6;
  - the blocks-world family, 4n − 2, for n = 1–6.
- **Exhaustive search:** GOAP's plan cost equals breadth-first search's (unit costs) or Dijkstra's
  (weighted) on 500 random small STRIPS instances.
- **The foraging domain:**
  - the heuristic is admissible (checked against an uninformed search on random small instances);
  - hand-built plans;
  - invalidation and replanning;
  - the fallback.
- **Reduction:** with K covering the sight candidates, H = 1 and the travel cost set to 0 through a
  test-only hook, GOAP's chosen site is rule M's, with the same tie draw.
- **The marginal-value rule:** the ρ update, stay against leave, and committing to the target.
- **`memory.prior: map`** fills memory at tick 0.
- **Golden and legacy:** unchanged, plus new entries. A WASM pin for `goap-mvt`.

## Page

- **Decision group:** GOAP and "Marginal value" options; K and H numbers; `mvt.alpha`.
- **Memory group:** the prior.
- **Inspect:** the plan, "Plan: n steps, gathers ~x of G", the next target, and ρ for the
  marginal-value rule.
- **Map:** the inspected Flump's plan drawn as a route through its sites (dashed, like Minds 2's
  path).
- **Charts:** `replans`, `mean_plan_length`, `mean_rate`.

## Docs

- **README:** a Minds 4 section.
- **The program document:**
  - results;
  - the cost table;
  - Minds 5's target: caching (winter, a carrying limit, burying and digging, pilfering; Raby et al.
    2007: jays cached 16.3 ± 1.8 against 5.4 ± 1.8 nuts, t₇ = 3.01, P = 0.02; Amodio et al.'s
    counterpoint).
- **Roadmap:** the Minds line.
- **The spec's amendments.**

## Amendments (implementation)

These change or extend the sections above. The measured values are the survey's (20 seeds) and the
sweeps' (20 seeds), with the source named. The cost figures are the release CLI's.

- **Forward search, as built.** The planner lives in `minds/goap/` (`mod.rs`, `strips.rs`,
  `forage.rs`). It interns each state to an index for Minds 2's `astar`, computing the state's
  heuristic and goal flag once. A goal state's only edge goes to a sentinel at cost 0, and the
  sentinel is A\*'s goal, so reaching a goal costs one extra expansion; the limit of 4 096 counts
  it. A start that is already a goal returns the empty plan without searching. Actions aren't
  stored per edge: the path's actions are rebuilt by regenerating each state's actions and taking
  the first cheapest one that reaches the next state.
- **STRIPS, as built.** Fact sets are `u128`, not `u64`: blocks at n = 6 has 81 facts and one-truck
  logistics at n = 6 has 97. The heuristic is (unsatisfied goal facts) × (cheapest action cost) when
  every action adds at most one goal fact, otherwise 0. The families are recovered from Helmert and
  Mattmüller: `logistics_trucks` (one truck per city, grounded over reachable facts), n = 1–6;
  `logistics_one_truck`, n = 1–5, with n = 6 (24 = 4·6, about 62 s and 19.3 M expansions) behind
  `#[ignore]`; gripper, n = 1–8; blocks, n = 1–6. The 500 random instances (6–12 facts, 4–12
  actions, costs 1–3) give 130 non-empty plans, 107 starts at a goal and 263 unsolvable; every one
  matches Dijkstra.
- **The foraging state, as built.** (slot, harvested mask), with gathered sugar derived from the
  mask, not stored, so a plan has at most (K + 1)·2^K states. The heuristic is ∞ when nothing
  unharvested is worth anything. If harvesting every slot still falls short of G, the Flump goes
  straight to the fallback without searching, so a Flump with no known sugar never searches.
- **Ruling: ties, as built.** Equal-cost plans take A\*'s deterministic order (lowest f, then h, then
  push order), and only the fallback's maxima go through `choose`. The reduction test asserts that
  GOAP's first target has rule M's maximal value and minimal distance, on 200 random neighborhoods
  where one harvest meets G, not the same draw. The exact-draw variant needs a search per first
  action. So GOAP's choices among equals can differ from rule M's random ones.
- **Ruling: the shortlist ranks by rate.** The K candidates are the others ranked by value ÷
  (distance + 1), the fallback's rate, then distance, then site index. The spec's ranking by value is
  kept as `goap.shortlist: value` (live). Why: with the map known, the value ranking shortlists only
  the far peak centers, so the option set, not the planner, decides travel. The planner shuttles
  between centers and starves: at s = 20, 4 of 60 Flumps are alive at tick 1000 under value against
  28 of 60 under rate, and under value the travel test reverses (residence falls with spacing in 20
  of 20 seeds). The finding, that what a planner considers matters more than how it plans, stays
  visible through the switch.
- **Invalidation, as built.** The next target holds while it's still among the candidates (in sight,
  that means unoccupied), holds at least half its planned value when in sight, and can be reached.
  A remembered target out of sight can't be checked for occupancy; `arrive` stops one step short and
  the next tick finds it taken. A site walked over is harvested on the way, so a later plan step
  there fails the half-value check and the Flump replans.
- **Ruling: unreachable candidates.** GOAP drops `walled_apart` candidates and the target the last
  walk couldn't reach, in `forage.rs`, before the plan check, the slots and the fallback (not in
  `candidates_with_memory`, so other rules and golden values are unchanged). The exclusion covers only
  the last walk: a boxed-in Flump alternates between planning and the fallback and never moves.
- **Ruling: one good.** `decision.rule: goap` with other than exactly one good is a validation error
  ("planning (GOAP) needs exactly one good"): welfare over several goods isn't sugar. G is the
  effective sugar metabolism (disease fee included) × H. `mvt` isn't restricted.
- **Ruling: counting fallbacks.** `TickEvents` counts `plans`, `plan_steps_sum`,
  `plans_with_remembered`, `plans_by_rememberers` (for the usage check, which asks about
  rememberers' plans), `fallback_short` and `fallback_limit`. Only found, non-empty plans count as
  plans.
- **The marginal-value rule, as built.** The leave target is the best known site by value, then
  nearer, then lower index (no draw). A commitment is dropped and redecided the same tick when the
  target leaves the reachable candidates. ρ is clamped at 0 (`max`, which also absorbs NaN). **Ruling:**
  a Flump whose best known site is its own doesn't count or set a leave. ρ starts at the metabolism
  and is set after the agent's draws, so the draw order is unchanged.
- **`memory.prior: map`, as built.** Only founding rememberers get the map; children and
  replacements start empty. Past `MEMORY_CAP` (4 096) the richest sites are kept, ties to the lower
  index. It draws nothing.
- **Statistics, as built. Ruling:** GOAP adds `fallbacks` and `plans_remembered` beside `replans` and
  `mean_plan_length`. MVT's `replans` is the share that left. **Ruling:** the page's **Planning**
  chart holds `mean_plan_length` alone, and **Plan use** holds the three shares on 0–1; under MVT,
  **Average rate** (`mean_rate`) and **Leaving** (`replans`). The page adds `goap.shortlist` and
  Inspect's ρ (`AgentView.rate`, MVT only). **Ruling:** choosing GOAP or MVT doesn't switch movement
  to walk; the validation error and the group note say so, as for Minds 2's walk-only rules.
- **The theorem's world, balanced as measured.** Each radius-3, height-4 patch has 25 sites and 56
  sugar (225 sites in all). At the spec's growback 0.05 a patch regrows 25 × 0.05 = 1.25 a tick,
  at least one forager's need, so it never runs out, and total regrowth is 2.25 times the need of 5.
  By the rule set before measuring, growback became **0.02** (0.5 a patch) and the population **3**
  (1.5 times the need). Endowment 50, which the spec left open, is `mem-mvt`'s. Under the rate
  shortlist a lone planner empties its patch (no site holding 1 or more) by tick 33 and leaves at
  36; the 5-seed check left 1–2 of 3 alive at tick 1000. `mvt-rule` uses the same world.
- **The survey's worlds, as built.** The spacing tori are built survey-side from `goap-mvt`
  reshaped to 3s × 3s with peaks at s/2 + s·i, the same patches, growback 0.02 and 3 Flumps at every
  s, the map known; a test pins s = 20 to the presets. The reduction `walking_at_vision_one_is_jumping`
  skips GOAP and MVT presets, which can't be forced to jump.
- **Ruling: overstaying without transit.** The judge stays the pre-registered literal measure (the
  last in-patch tick's gain against the realized mean gain per tick). A reported row excludes transit
  ticks (MVT: `leaving` set; GOAP: the landing site wasn't a plan target, or the fallback) and visits
  that were all transit. GOAP fallback ticks that stay put count as transit (0.1–0.7 % of ticks).
- **Sweeps, as built.** `goap-horizon` and `goap-k` base on `ii-2-unit` with walk and GOAP set;
  `goap-memory` bases on `goap-truffles` and uses `mem-share`'s five shares.
- **Measured (the survey, 20 seeds; ticks 200–500 for the memory worlds):**
  - `goap-mvt.travel` **Holds**: slope > 0 in 20 of 20 (median 0.413); residence 18.6, 22.6, 25.7,
    22.8 at s = 12–24. `mvt-rule.travel` **Holds**: 19 of 20 (median 0.096); 11.8, 12.4, 12.7,
    12.9. Rule M with the same knowledge: negative in 20 of 20 (6.3, 4.9, 4.9, 5.0). Value
    shortlist: negative in 20 of 20 (7.6, 8.6, 6.8, 5.9). Survival falls with spacing under every
    rule.
  - `goap-mvt.overstay` **Holds**: median 0.571, 18 of 20 above 0.5. `mvt-rule.overstay` **Fails**:
    0.392, 0 of 20. By spacing, GOAP 0.614, 0.594, 0.571, 0.458; MVT 0.555, 0.452, 0.392, 0.348;
    rule M 0.089–0.140. Without transit: GOAP 0.325, 0.292, 0.303, 0.201; MVT 0.074, 0.032, 0.021,
    0.043; no seed above 0.5 at any spacing. GOAP's literal overstaying is likely the walk out, and
    overstaying falls with travel under both (per-seed slope positive in 1 and 0 of 20), opposite to
    Constantino and Daw.
  - `goap-open.advantage` **Holds** +69.4 (19 of 20), `goap-truffles` +98.7 (20 of 20),
    `goap-walled` +32.8 (20 of 20); rule M −112.6, −82.0, −114.1; Minds 3's travel-priced utility
    mind +7.4, +1.5, −55. Reported: the dead as 0, +35.1 (18 of 20), +26.3 (14 of 20), +10.2 (15 of
    20); rememberers' plans using a remembered site out of sight 99.0 %, 97.7 %, 99.3 %; new plans
    on 21–33 % of Flump-ticks, the short fallback on 30–40 %, the limit fallback on 0 %. GOAP's
    others are poorer than rule M's (240 against 438 open) and GOAP's population higher (200.5
    against 176.5). The "still a candidate" rule, which drops a non-rememberer's off-axis target
    sooner, may bias plan survival toward rememberers; not isolated.
  - Sweeps (untested): `goap-horizon` 143.3, 159.0, 206.4, 206.3, 195.7 at H = 2, 5, 10, 20, 40
    (walking rule M 181, jump 224); `goap-k` 196.5, 207.9, 206.4, 204.9 at K = 2, 4, 8, 12;
    `goap-memory` +90.1, +100.2, +104.4, +99.8, +113.0 at shares 0.1–0.9 (rule M's `mem-share`:
    −88.5, −73.2, −76.2, −86.1, −98.1).
  - Usage on `ii-2-unit` (one seed, 200 ticks, a scratch count): planned on 20.8 % of Flump-ticks,
    the short fallback on 52.3 %, the limit fallback on 0 %.
- **Cost** (µs per Flump-tick, release CLI, 2 000 ticks, seeds 1–5, run twice, within 6 %):
  `goap-mvt` 80.7, `mvt-rule` 54.2, `goap-open` 82.2, `goap-truffles` 74.7, `goap-walled` 149.0;
  re-timed `mem-open` 14.2, `mem-truffles` 24.3, `mem-mvt` 14.6, `mem-walled` 9.1, `walk-capacity`
  5.58, `ii-2-unit` 1.20. GOAP costs 5.8, 3.1 and 16 times rule M in the same worlds.
- **Titles** follow the measurements: `goap-mvt` "Planners who price travel stay longer when patches
  are farther apart"; `mvt-rule` "Leave when a patch falls below your average: stays lengthen only
  slightly with travel"; `goap-open` "Planners who remember end up richer on the open sugarscape";
  `goap-truffles` "Planners who remember hidden truffles end up richer while they live";
  `goap-walled` "Planners who remember what lies beyond the wall end up richer, and most survive".
- **Minds 5's target** is caching, with Raby et al. (2007) and Amodio et al.'s counterpoint; see the
  program document.
