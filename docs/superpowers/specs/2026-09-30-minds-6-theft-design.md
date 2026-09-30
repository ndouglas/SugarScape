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
    then long-term scatter hoarding can persist". Loot that isn't recached (eaten, or put in a
    larder) fails that condition. (Corrected; see the amendments: "the most damaging form of
    pilferage" is p.664, about pilferage that isn't reciprocated.)
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

## Amendments (implementation)

The rulings made while building, and what measuring changed. The full record is the SDD ledger for
this plan.

- **Eaten loot feeds the thief (the stomach).** This replaces "`eat` … adds nothing to holdings",
  which starved thieves: under that rule a pilfer was strictly worse than the harvest it replaced.
  - Under `eat` the thief takes the whole cache, with no room cap, into `Agent.fed`, a stomach.
    Metabolism draws on the stomach before holdings. An agent starves only when holdings ≤ 0 and
    the stomach is empty.
  - Stomach sugar can't be buried, dug or traded. It doesn't count toward the carrying limit, the
    reserve, the surplus or hunger. What's left in it at death leaves counted (`fed_lost`), and it
    isn't inherited.
  - `loot_eaten` counts sugar as it enters the stomach, so it is a transfer, not a term in the ledger.
    The ledger is sites + holdings + caches + stomachs + eaten (metabolism and bury cost) + what
    left with the dead = start + growback.
  - `keep` is unchanged: the take is min(cache, room under the carrying limit), and a success
    with no room takes nothing, so the agent harvests as usual.
- **The citation, corrected.** The Source summary cites "the most damaging form of pilferage" as
  p.661, about eating or lardering. The phrase is on **p.664** and is about pilferage that isn't
  reciprocated: "Nonreciprocated pilferage, which is often caused by heterospecifics, appears to be
  the most damaging form of pilferage." What p.661 says is: "What is important is not how much
  pilfering occurs, but what the pilferer does with the food that it discovers. If the pilferer
  recaches the food, and if pilfering is reciprocal, then long-term scatter hoarding can persist
  unless some alternative form of food storage (larder hoarding or internal energy storage)
  provides more benefits." It also names the larder: "The cheating strategy that seems most likely
  to be damaging to a population of scatter hoarders is pilferage of scattered caches and storage
  of pilfered items in a defensible larder." So `eat` is loot that isn't recached, the case that
  fails p.661's condition. It is not the paper's "most damaging form".
- **Stumbling, exactly.**
  - An agent arriving on a site makes one draw per foreign cache there, in owner-id order, even
    after an earlier success, so the number of draws depends only on the caches present. The first
    success is taken: **one take per arrival**.
  - **The dig wins.** An owner that digs its own cache (Minds 5's rule, with `owner_memory` on)
    makes no draw. An owner that isn't digging draws only for the foreign caches.
  - Staying on a site counts as arriving, as it does for Minds 5's dig, so an agent standing on
    foreign caches draws again every tick.
  - Under `owner_memory: off` an owner draws for its own cache in owner-id order with the rest. A
    success is a dig (`owner_finds`, a `Dug` fate, into holdings under either loot rule), whether
    or not the owner is hungry: it stumbled on the cache rather than choosing it.
  - Pilfered sugar is kept apart from dug sugar (`Harvest.pilfered`). Like dug sugar, it isn't a
    harvest.
  - The index of caches by site is built on the first stumble that needs it and dropped when
    `find` returns to 0. With `find` 0 nothing is drawn or allocated.
- **The pilferage rate** is `caches_pilfered ÷ pilfer_candidates`. The numerator is the distinct
  caches that existed at the tick's start and lost any sugar to a thief this tick. The denominator
  is every cache in the world at the tick's start. Takes (`pilfers`) are kept beside it.
- **Fates.**
  - The cache log is recorded only when a caller asks for it (`World::record_fates`: the survey
    and the tests that read the log set it; no config, the app or a sweep does), and then only
    while theft is on. Every other world, Minds 5's included, allocates nothing. (Final review:
    the log grew without bound in long app runs, and nothing but the survey reads it.)
  - **Backfill:** at every touch of a cache under theft (bury, dig, pilfer, death), sugar the log
    doesn't hold, from before theft came on, is first logged as a record dated `cache_since`.
    From the tick theft comes on, Σ Dug = Σ dug events, Σ Lost = Σ `cache_lost`, and Σ Pilfered =
    Σ `pilfered`, every tick (tested).
  - At 1 000 000 records the log freezes. The survey skips a frozen run for ages; it skipped none.
  - The stats' fate shares come from the tick events and live caches, not the log. They start at 0
    when theft is turned on, so a mid-run switch leaves the shares summing to more than 1.
- **Refusals.** Theft in a lab or a central-place world is refused on `theft.find`, and cheaters
  alone (`find` 0) in a lab on `theft.cheaters`. A bury cost above 0 is refused there on
  `caching.bury_cost` ("a bury cost applies only in the field").
- **p_s and p_o are amount-weighted:** p_s = sugar dug by owners ÷ (dug + pilfered), and p_o =
  pilfered ÷ (dug + pilfered + lost). Splits and backfill can't shift them. Still-buried sugar has
  met no fate and is excluded.
- **`find` is a free parameter; 0.25 is an anchor.** The balance probe found no listed `find`
  inside the field's band (below), so the list was extended mechanically: the smallest of 0.25,
  0.3, 0.4 and 0.5 with a winter rate of at least 2 %. That is 0.25, and every preset uses it. After
  that, the survey reports every claim across `find` 0.02, 0.05, 0.1, 0.25, 0.5 and 1, and judges
  at 0.25.
  - **Claim 1 became a decomposition.** Pilferage = non-owner visits per cache per tick (v) ×
    `find`, less the one-take-per-arrival shortfall on stacked sites. 2–30 % a day is context only.
  - The cohort fit to (1 − r)^age is reported, not judged.
- **Stumbling can't reach the field's median.** The rate is at most v × `find`, and at most v at
  `find` 1. v is set by crowding (0.08 agents per open site in the winter world). Even at `find` 1
  the winter field loses 7.0 % of its caches a tick (10.9 % of its sugar; the survey, ticks 0–200),
  below the 9 % median. Animals that reach the field's rates search for caches. Watching others
  cache comes in P2.
- **The arena's vision scales with the room:** 1–2, 1–3 and 1–4 at n = 2, 4 and 8, half the room's
  side. A fixed 1–2 let the 2-agent room be seen whole and the 8-agent room only in part, so it
  tied sight to n.
- **Survey-only instrumentation.** `TickEvents::pilfer_draws` counts find draws on other agents'
  caches. `World::probe_dig_at_reserve` digs below R instead of R/2, and `World::record_fates` keeps the
  fate log; neither is config, hashed or shown in the docs, and each is false everywhere but the
  survey (and, for the log, the tests that read it). None changes any golden.
- **The page.** Inspect also shows the stomach when it isn't empty. Holdings get their own chart
  ("Hoarder and cheater holdings"), so that counts and sugar don't share an axis. The owner-memory
  box reads through the default, so a config without it shows checked.

**The balance, as measured** (Task 5; `cache-winter-even`'s world with theft, `even` hoarders only,
5 seeds; the rate is the mean of `pilferage_rate` over ticks 101–200; survival is alive at 200 ÷
alive at 100):

| find | winter rate | summer rate (ticks 1–100) | first-winter survival |
|---|---|---|---|
| 0 | 0 | 0 | 74.9 % |
| 0.01 | 0.12 % | 0.16 % | 78.4 % |
| 0.02 | 0.24 % | 0.32 % | 78.6 % |
| 0.05 | 0.55 % | 0.72 % | 84.2 % |
| 0.1 | 1.02 % | 1.23 % | 86.2 % |
| 0.2 | 1.83 % | 2.08 % | 91.0 % |
| **0.25** | **2.21 %** | 2.38 % | 91.7 % |
| 0.3 | 2.55 % | 2.71 % | 92.5 % |
| 0.5 | 3.93 % | 3.79 % | 94.8 % |
| 1 | 7.67 % | 5.77 % | 98.3 % |

The survey's figures (20 seeds, ticks 0–200, a summer and a winter) are 2.31 % at 0.25 and 6.96 %
at 1. Both windows are stated in `presets.rs`.

The arena, without theft (100 seeds per n): first-winter survival with nobody caching was 0.0 %,
0.5 % and 1.4 % at n = 2, 4 and 8, against 99.5 %, 95.8 % and 91.0 % for `even` hoarders. The gap
is at least 89.6 points at every n. At `find` 0.25 with half cheaters the rate over ticks 101–200
is 1.57 %, 1.89 % and 2.40 %, and 100 %, 99.8 % and 98.1 % survive.

**The values.**

| World | Values |
|---|---|
| winter theft (`theft-winter`, `-quarter`, `-half`) | `cache-winter-even` unchanged, plus `theft.find` 0.25 and cheaters 0, 0.25 or 0.5; owner memory on, loot kept, bury cost 0 |
| arena (`theft-arena-2`, `-4`, `-8`) | a k × k room (k = 4, 6, 8) on a (k + 1) × (k + 1) torus, opaque walls along row 0 and column 0; flat sugar, capacity 4 and growback 0.3 (× 8/9 at n = 4, where 6 × 6 gives 9 sites an agent), so each agent has 32 standing and 2.4 a tick of regrowth; vision 1 to k/2; otherwise the winter world's settings (walking, metabolism 1, global winter γ 100 and β 32, carrying limit 50, horizon 20, half remembering for 100 ticks, `even`); `find` 0.25 and half cheaters |
| survey sweeps | `find` 0.02, 0.05, 0.1, 0.25, 0.5, 1 (and 0 for the cheater baseline); cheater shares 0.1–0.9; arena C/G (bury cost) 0, 0.1, 0.25, 0.5, 1; β 2, 4, 8, 16, 32 |
