# Minds 6: theft (design)

**Date:** 2026-09-30
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 6. It is the first step of the pilfering
campaign:
- **P1, theft** (this milestone, Minds 6);
- **P1b, evolution of larder hoarding** (Minds 7);
- **P2, watching;**
- **P3, protection;**
- **P4, deception.**

Behavior trees move after the campaign. This is our own experiment, numbered apart from the
reproductions' milestones.

**Builds on:** Minds 1–5, all binding where not changed here. That covers the decision seam, walking,
memory, GOAP and the marginal-value rule. From Minds 5 it covers caches, the carrying limit, dig
hysteresis, the caching rules, `caching.mixed`, global winter and the winter world. It also covers the
literal-default-plus-named-switch pattern, the titles rules, and the rule that every engine reduces
to what came before. Model agents are "agents", never "Flumps".

**Out of scope:**
- **Evolution** (heritable strategies and generations): P1b.
- **Seeing others cache, and remembering where:** P2.
- **Re-caching when watched, and caching out of sight:** P3.
- **Misleading others:** P4.

**Sources**, local copies in `papers/caching/`:
- S. B. Vander Wall and S. H. Jenkins, "Reciprocal pilferage and the evolution of food-hoarding
  behavior", *Behavioral Ecology* 14(5) (2003), 656–667.
- M. Andersson and J. Krebs, "On the evolution of hoarding behaviour", *Animal Behaviour* 26 (1978),
  707–711.
- For context only: Vander Wall and Balda, *Ecological Monographs* 47 (1977), 89–111; Balda and Kamil,
  *Animal Behaviour* 44 (1992), 761–769.

## Goal

Let agents steal each other's caches and measure what theft does to hoarding. We test Andersson and
Krebs's condition for hoarding to pay in a group, their claim that it doesn't depend on how many
hoard, and their claim that without an owner's advantage cheaters win. We also test Vander Wall and
Jenkins's claim that reciprocal pilferage keeps hoarding worthwhile, and their untested suggestion
that a cheater that never caches could flourish in mild winters. All of this uses fixed strategy
mixes. The evolutionary claims wait for P1b.

## Non-negotiable constraints

- **Earlier work unchanged.** Every golden entry, legacy fixture and pinned fingerprint stays green
  and unedited. Theft is off by default.
- **Reduction.** With `theft.find` 0, `theft.cheaters` 0 and `caching.bury_cost` 0, every world is
  its Minds 5 self, bit for bit.
- **Draws.** Theft's find draw happens only when theft is on. Cheaters are assigned by agent id, with
  no draw.
- **Conservation.** Sites + holdings + caches + eaten + lost is conserved through theft, with either
  loot rule. Bury cost counts as eaten.
- **Judges before runs.** Survey judges and thresholds are committed before any survey run, and never
  tuned afterward.
- **Honest reporting.** Causes are labeled "likely" unless isolated. Before a mechanism is credited,
  report how often it acts and whether a result rests on who survives. Every survival or wealth
  figure is also reported per founding agent, with the dead counted as 0.

## Source summary

- **Andersson and Krebs (1978):**
  - Alone: F_H = Gp − C and F_N = p_r·m·G. Hoarding pays if p > C/G + p_r·m (condition (1), p.707).
  - In a group sharing a feeding area, hoarders beat non-hoarders if p_s/p_o > (C/G)(n − 1) + 1
    (condition (3), p.708). Here p_s is the probability the hoarder recovers its own item, p_o the
    probability another individual finds it first, C the "fitness cost of hoarding one item", G the
    gain from eating a stored item, and n the group size.
  - Frequency independence (p.708): "If a hoarder is fitter than a non-hoarder, this applies
    irrespective of the proportion of hoarders in the group."
  - Equal recovery (p.710): "if … (p_s = p_o) then hoarders will have a lower fitness than
    'cheaters'."
  - A stable mixture (p.708), with no derivation shown: "a stable mixture of hoarders and non-hoarders
    may result … hoarders are fitter than non-hoarders when at a low proportion".
  - The conditions are necessary, not sufficient. The paper gives no numbers and no simulation.
- **Vander Wall and Jenkins (2003):**
  - "Most pilferage rates for long-term hoarders fall between 2–30% per day" (p.656), with an
    empirical median of 9% a day (p.663). Their rates assume "the rate of removal over the duration
    of a study was constant" (p.658).
  - Reciprocal pilferage (p.661): "If the pilferer recaches the food, and if pilfering is reciprocal,
    then long-term scatter hoarding can persist". Eating or lardering stolen food is "the most
    damaging form of pilferage".
  - Their model is a genetic algorithm: 20 animals, a 100-day season, 20 foraging bouts a day, 2 100
    food items, heritability 0.8 and 60 generations. Owners recover their own caches at no cost; only
    others search by apparency.
  - "the average daily rate of loss of scatter hoards was 18% in cases in which larder hoarding did
    not become established" (p.663).
  - The larder results (the bistability and the apparency-ratio threshold 0.2–0.3) are P1b's.
  - Untested in the paper (p.661): "under ideal conditions (e.g., mild winters), a nonhoarding
    cheater could survive and even flourish".
- **Context, checked:**
  - Vander Wall and Balda (1977) estimate that a flock of 150 nutcrackers cached 3.3–5.0 × 10⁶ piñon
    seeds, 2.2–3.3 times their needs. This is an estimate, not a count.
  - Balda and Kamil (1992) found recovery better than chance after 285 days, in 7 birds in a lab
    room.

## Mechanics

- **Finding others' caches** (`theft.find`, 0 to 1, default 0). An agent that arrives on a site
  holding caches of other agents finds each one with probability `find` that tick. This is one draw
  per foreign cache, in the site's owner-id order, and only when `find > 0`. A found cache is
  **pilfered** whole, up to the carrying limit's room. Any remainder stays as the owner's cache.
  Pilfering replaces that tick's harvest of the site, as a dig does.
- **The thief's choice.** Foreign caches the agent doesn't know about aren't candidates; theft is
  stumbled on, never planned. That is P2's job. Pilfering happens only where the agent's own rule
  took it.
- **The owner's advantage** (`theft.owner_memory: on | off`, default on).
  - **On:** Minds 5 applies, and owners know their own caches.
  - **Off:** own caches aren't candidates. An owner arriving on one finds it with probability
    `find`, like any thief. This is Andersson and Krebs's p_s = p_o case.
- **Loot** (`theft.loot: eat | keep`, default `keep`).
  - `eat`: the pilfered amount is eaten on the spot. It's counted as eaten and adds nothing to
    holdings.
  - `keep`: it goes into the thief's holdings, where a hoarding thief's rule may bury it again. That
    is reciprocal pilferage, arising, not scripted.
- **Cheaters** (`theft.cheaters`, 0 to 1, default 0). This is the share of founders who never bury.
  An agent with id i is a cheater iff ⌊i·s⌋ > ⌊(i − 1)·s⌋. That gives an exact proportion with no
  draw.
  - A cheater's caching rule is `none`, whatever `caching.rule` says. Everyone else follows
    `caching.rule`, or `caching.mixed`.
  - Children take the rule of the parent whose turn it is, as in Minds 5.
- **Bury cost** (`caching.bury_cost`, default 0). This is sugar lost per unit buried: burying q
  costs q × bury_cost from holdings, counted as eaten. It is Andersson and Krebs's C, with G the unit
  recovered.
- **Every cache's fate is recorded:** dug by its owner, pilfered (by whom), lost with a dead owner,
  or still buried. Each record carries the tick it was buried and the tick it ended.

## Worlds

- **Winter field:** Minds 5's winter world, unchanged:
  - 175 agents, metabolism 1;
  - global winter with γ 100 and β 32;
  - carrying limit 50 and horizon 20;
  - `even` hoarders.
  
  Theft is on at a `find` the balance probe chooses, so that daily pilferage lands inside 2–30 %,
  measured, not assumed. `plan` hoarders are reported alongside.
- **Arena:** a small walled patch with n agents (n = 2, 4, 8) through one winter:
  - sites and regrowth scaled so that each agent's share is the same at every n;
  - Andersson and Krebs's n is exact, since the group shares one area.
  
  The plan's first task measures and records the arena's balance before any preset is recorded.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `theft.find` | 0 | live | per-tick chance of finding each foreign cache on a site one arrives on |
| `theft.owner_memory` | `on` | reset | owners know their own caches |
| `theft.loot` | `keep` | live | `eat` or `keep` |
| `theft.cheaters` | 0 | reset | share of founders who never bury |
| `caching.bury_cost` | 0 | live | sugar lost per unit buried |

Validation:
- theft needs caching on;
- `find` is in [0, 1] and `cheaters` in [0, 1];
- `bury_cost` ≥ 0;
- theft is not allowed in a lab (`lab`) or a central-place world.

## Statistics

When theft is on, the stats record:
- `pilfered` per tick;
- `pilferage_rate`: caches pilfered this tick ÷ caches existing at the tick's start (a tick stands for a day);
- `cache_fates`: cumulative shares dug by owner, pilfered, lost and still buried;
- the mean holdings and survival of hoarders and of cheaters, when there are cheaters.

## Survey (`minds6` claims)

The judges are committed first.
1. **Constant pilferage.** Caches disappear at a constant per-tick rate. The cohort survival fits
   (1 − r)^t, judged per seed on the fit's residuals within a stated tolerance. r is reported against
   2–30 % a day, with a median of 9 %.
2. **Andersson and Krebs's threshold.**
   - Across arena runs sweeping `find`, `bury_cost` and n, hoarders beat cheaters (per founder, dead
     as 0) where p_s/p_o > (C/G)(n − 1) + 1 holds.
   - p_s and p_o are measured from cache fates, not set.
   - The judge is the share of runs where the sign of the advantage agrees with the condition.
3. **Frequency independence.** In the winter field, the hoarder advantage stays flat across cheater
   shares from 0.1 to 0.9. The judge is that the fitted slope's confidence interval includes 0. Any
   slope is reported with its likely cause.
4. **Equal recovery.** With `owner_memory: off`, cheaters beat hoarders at every cheater share,
   judged per seed.
5. **The mixed equilibrium.** Where hoarders are poorer thieves, the advantage crosses zero at an
   interior cheater share. The crossing is located and reported; its stability is P1b's.
6. **Reciprocity.** At the same `find`, `keep` gives hoarders more stored food and higher survival
   than `eat`, paired on seeds.
7. **Loss that hoarding withstands.** The daily loss per cache in worlds where hoarders still beat
   cheaters, reported against Vander Wall and Jenkins's 18 %.
8. **The mild-winter cheater** (new ground). The cheater's advantage against winter severity (β
   swept). The prediction is that cheaters gain as winter softens. This is judged per seed as a slope.
9. **Usage.** How often theft acts, per world, and the shares of cache fates.

## Verification (tests, not claims)

- **Reductions:** the defaults reproduce Minds 5, and golden is unchanged.
- **Conservation** through pilfering, with either loot rule, and through bury cost.
- **The find draw:** over many trials, the pilfer frequency matches `find` (a binomial test).
- **Owner memory off:** owners find their own caches only at rate `find`.
- **Cheaters:** exact proportions by id, with no draws.
- **Fates:** every cache ends in exactly one fate.

## Page

- A "Theft (Minds 6)" group under the Minds entry: find, owner memory, loot, cheaters and bury cost.
- In the menu, `usesMinds` covers theft, and `MINDS_TITLES['6'] = 'Minds 6: theft'`.
- Inspect: whether the agent is a cheater, plus caches stolen by it and from it.
- Charts: "Pilferage" (the rate) and "Cache fates" (0 to 1); "Hoarders vs cheaters" when there are
  cheaters.

## Docs

The README gets a Minds 6 section. The program document gets its status, results, the campaign
outline (P1–P4) and the next target, P1b. The roadmap line is updated, and this spec gets its
amendments.
