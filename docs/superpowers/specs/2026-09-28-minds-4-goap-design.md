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
