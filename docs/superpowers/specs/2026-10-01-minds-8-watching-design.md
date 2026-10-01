# Minds 8: watching (design)

**Date:** 2026-10-01
**Program:** Minds (`docs/studies/2026-09-27-minds.md`), step 8. It is P2 of the pilfering campaign:

- P1: theft (Minds 6, done).
- P1b: the evolution of larder hoarding (Minds 7, done, as Vander Wall and Jenkins's non-spatial model).
- **P2: watching (this milestone).**
- P1b again, spatially (Minds 9, next): Minds 7's heritable larder hoarding in the Sugarscape world.
- P3: protection.
- P4: deception.

Behavior trees come after the campaign. This is our own experiment, numbered apart from the reproductions'
milestones.

**Builds on:** Minds 1–7, all binding where not changed here. In particular:
- Minds 2's sight (`World::sight`: the book's four lattice lines out to an agent's vision, stopped by opaque
  walls).
- Minds 5's caches, the carrying limit, the reserve R and hunger (holdings below R / 2), `join_caches`, and the
  winter world.
- Minds 6's theft (`theft.find`, `owner_memory`, `loot`, `cheaters` dealt by id), the fate log, `Agent.fed`,
  `caches_pilfered` and `pilfer_candidates`, and the one-take-per-arrival rule.
- The literal-default-plus-named-switch pattern, the titles rules, the Minds menu entry, and "judges before
  runs".

Model agents are "agents", never "Flumps".

**Out of scope:**
- **The cacher's side:** caching out of sight, re-caching when watched, guarding a cache, and an owner
  returning to chase a raider. These are P3.
- **The raider's tactical side:** approaching or following cachers, and waiting for the owner to leave before
  raiding. These matter only once owners defend, so they are P3 as well.
- **Misleading others:** P4.
- **Evolution:** Minds 9.

## Sources

Local copies are in `papers/caching/`.

- T. Bugnyar and K. Kotrschal, "Observational learning and the raiding of food caches in ravens, *Corvus
  corax*: is it 'tactical' deception?", *Animal Behaviour* 64 (2002), 185–195 (B&K).
- B. Heinrich and J. W. Pepper, "Influence of competitors on caching behaviour in the common raven, *Corvus
  corax*", *Animal Behaviour* 56 (1998), 1083–1090 (H&P).
- M. Andersson and J. Krebs, "On the evolution of hoarding behaviour", *Animal Behaviour* 26 (1978),
  707–711 (A&K).
- C. J. Barnard and R. M. Sibly, "Producers and scroungers: a general model and its application to captive
  flocks of house sparrows", *Animal Behaviour* 29 (1981), 543–550 (a scan with a text layer): "the
  hypothetical pay-off to scroungers increases with the number of producers", with a stable mix at the ESS
  point. Context: W. L. Vickery, L.-A. Giraldeau, J. J. Templeton, D. L. Kramer and C. A. Chapman, "Producers,
  scroungers, and group foraging", *American Naturalist* 137 (1991), 847–863.
- Context only: P. A. Bednekoff and R. P. Balda (1996a, b), *Behaviour* 133, 807–826 and *Animal Behaviour* 52,
  833–839, cited by B&K for observational spatial memory in pinyon jays, Clark's nutcrackers and Mexican jays.

## Source summary

**Seeing matters** (B&K, Experiment 1, pp. 187–189, Fig. 2):
- The demonstrators made 69 caches: 56 with one to three observers present, 13 with none.
- Of the 56, 27 were taken back by the demonstrator during the session. Of the other 29, "all were found
  during retrieval sessions by conspecifics that had been in the pathway during caching".
- Of the 13 caches made with no observer present, only one was found by a conspecific.
- "All ravens moved directly to the sites where they had observed the demonstrators caching. On average, they
  recovered the food within the first minute". Birds that had seen food handled but not the final caching
  "moved up and down the aviary, checking several locations", and searched significantly longer (Fig. 3).

**Not by smell or by chance** (H&P, Experiment 2, p. 1086):
- "The ravens recovered none of 40 artificial caches."

**Observational memory is short** (H&P, Experiment 2, p. 1086, Fig. 4):
- Of 85 caches, 58 (68 %) were recovered "during a 1-h trial the next day". Fifteen of those 58 were
  recovered "by birds other than those that had made the caches but that were present during caching".
- At 14 days, 6 of 19 caches (32 %) were recovered. At 28 days, 1 of 13 (7.7 %).
- "No caches other than the birds' own were recovered during the 14- and 28-day trials."

**The cacher–raider game** (B&K, pp. 189–193, Table 3). Context for P3, not modeled here:
- Cachers withdrew from others and cached behind objects that blocked the view (about 80 % of caches).
- Raiders kept their distance and raided only after the cacher left, at a first attempt of about 1.5–2
  minutes.
- In the field, raids succeeded in 3 of 12 attempts (25 %) with the cacher within 10 m, and in 15 of 24
  (63 %) with the cacher further away.

**The condition for caching** (A&K): with no cost of burying, hoarding pays when owners recover more of
their caches than others do (p_s > p_o; with a cost C per unit G among n agents, p_s ÷ p_o > (C/G)(n − 1) + 1,
as in Minds 6).

**Producers and scroungers** (Barnard and Sibly, as above): when scrounging excludes producing, a scrounger's
payoff falls as scroungers become common, so the two strategies settle at a mixed equilibrium.

## Goal

Give agents observational spatial memory: an agent who sees another bury remembers the cache, and goes to
take it on purpose. Then ask three questions:

1. Does watching reach the field's pilferage rates, which Minds 6's stumbling couldn't?
2. Does watching break Andersson and Krebs's condition, so that caching stops paying?
3. Is watching a scrounger strategy (worth less as it becomes common), and does that depend on whether
   watchers also cache?

## Non-negotiable constraints

- **Originator first, where there is one.** The mechanism follows B&K and H&P: seen caches are found (B&K's 29
  of 29), unseen ones are not found by sight (H&P's 0 of 40). Stumbling stays Minds 6's separate `find`.
  Every gap the papers leave is a stated choice below or in the amendments, never tuned to a result.
- **Isolation.** With `watching.on: false` (the default) nothing runs: no observation, no allocation, no rng
  draw. Every existing golden entry and fingerprint stays unedited, natively and in WASM.
- **Determinism.** Watching draws no rng. Observers are visited in a fixed, documented order.
- **Judges before runs.** Survey judges and thresholds are committed before any survey run and are never
  tuned.
- **Honest reporting.** Causes are labeled "likely" unless isolated. Failures to confirm are findings.

## The mechanism

### Configuration

`watching` is a new config section:

| Field | Type | Default | Live? | Meaning |
|---|---|---|---|---|
| `on` | bool | false | live | Watching at all. |
| `span` | u32 | 7 | live | Ticks a seen cache stays remembered. |
| `watchers` | f64 in [0, 1] | 1.0 | reset-only | Share of founders who watch, dealt by id. |
| `raid_when` | `always` \| `hungry` | `always` | live | When a seen cache is a place to go. |

- **`watchers` is dealt by Minds 6's id rule:** the founder with id i watches when ⌊i·s⌋ > ⌊(i − 1)·s⌋. Over
  ids 1..=n that is ⌊n·s⌋ watchers, an exact share, with no draw. An agent born later doesn't
  watch. The trait is `Agent.watches`.
- **Watchers and cheaters overlap by design.** Both are dealt by the same rule, so at equal shares they are
  the same agents. That's how `watch-scroungers-only` makes pure scroungers (below): agents who watch and never
  bury. At unequal shares the overlap is whatever the rule gives, and the page says so where both are set.
- **`span` 7 is a gap choice.** H&P show observers recovering seen caches the next day and none at 14 days,
  and nothing between. We take 7, about midway, and every claim is reported across span 1, 3, 7 and 13. A
  tick stands for a day, as in Minds 6.
- **`raid_when: always` is the default.** Ravens raided within minutes (B&K), and Minds 6's thieves take
  caches all year. `hungry` puts raiders on the owners' terms: a seen cache is a place to go only while the
  watcher holds less than R / 2. This is the same threshold as Minds 5's hunger, but it doesn't require the
  watcher to have caches of its own.

### Seeing a burial

- When an agent buries at a site, every living agent who watches, other than the owner, whose sight covers that
  site at that moment remembers the cache. "Sight covers" means the site is in `World::sight(w.pos,
  w.vision)`: on one of the four lattice lines from the watcher, within its vision, and not behind an opaque
  wall.
- The memory is an entry keyed by (site, owner) in `Agent.seen`, a `BTreeMap`. It holds the amount the
  watcher saw buried and the tick. A second burial by the same owner at the same site, seen again, adds its
  amount and refreshes the tick. A burial not seen leaves the entry as it was.
- Watchers observe in id order. The order changes nothing that is drawn, since no draw is made, but it is
  documented for the counters.
- The owner never knows it was seen (P3).
- Loot buried again by its thief (under `loot: keep`) is a burial like any other and can be seen.
- An entry is forgotten when it is older than `span`, when the watcher arrives on its site (taken or not), or
  when the watcher dies. Forgetting by age happens lazily, wherever entries are read, and also in a sweep at
  each tick's start so that memory stays bounded.

### Going to a seen cache

- A watcher's remembered caches join its candidate list the way Minds 5's own caches do (`join_caches`). They
  come after the sight list and after its own caches, before Minds 3's remembered sites. Each is valued at the
  amount remembered, summed over owners at that site, and sits at its torus Manhattan distance. The same sites
  are skipped as for its own caches: a wall, one walled apart from it, one another agent stands on, and the
  target its last walk found no path to.
- Under `raid_when: hungry` they join only while the watcher holds less than R / 2.
- The value is what the watcher believes. A cache dug or taken since then still looks full until the watcher
  arrives.
- Rule M, GOAP and the marginal-value rule see the same list, as with own caches.

### Arriving

On arrival (`movement::go_and_gather`), in this order, and with at most one take an arrival as in Minds 6:

1. **The dig wins:** an owner who would dig its own cache there digs it (Minds 5 and 6, unchanged).
2. **A raid:** if the watcher remembers caches at the site, it takes from the first in owner-id order that is
   still there, with no draw. B&K's observers found every seen cache still in place. The take follows Minds 6's
   `loot` and carrying-limit rules exactly (`theft::pilfer`), including its fates, `pilfered`, `pilfers`
   and `caches_pilfered`. All of its entries at the site are then forgotten.
   - If none of the remembered caches is still there, it is a **wasted raid**, counted once per arrival, and
     the agent goes on to step 3.
3. **Stumbling:** Minds 6's `find` draws for the remaining caches, unchanged. With `find` 0 nothing is drawn.
4. Otherwise the agent harvests the site as usual.

A raid replaces the tick's harvest, as a pilfer does.

### Counting

These are new tick events and stats, all zero and unallocated with watching off:

- `burials_seen`: burials seen by at least one watcher.
- `sightings`: (watcher, burial) pairs.
- `raids`: takes from a seen cache, also counted in `pilfers`.
- `raided`: the sugar raids took, also counted in `pilfered`.
- `raids_wasted`: arrivals whose remembered caches were all gone.
- `seen_entries`: entries held after the tick-start sweep, summed over agents (see the amendments).
- Pilfers split by source: seen (`raids`) and stumbled (`pilfers − raids`).
- Group stats for watchers against everyone else, the pattern of Minds 6's `CheaterStats`: alive, survival,
  and wealth per founder (holdings, caches and stomach of the living; the dead count as 0).
- The fate log's `Pilfered { by }` gains nothing. Whether a pilfer was a raid is in the events, not the log.

Sugar is conserved: a raid is a pilfer.

## Questions and judges

**Setup.** Minds 6's winter field (`theft-winter`), seeds 1–20, 200 ticks each: a summer (ticks 0–99) and the
first winter (100–199). A tick stands for a day. Measures are Minds 6's unless said:
- the pilferage rate is Σ `caches_pilfered` ÷ Σ `pilfer_candidates` over ticks 1–200;
- p_s and p_o are amount-weighted, and sugar still buried at 200 is excluded;
- survival per founder is alive at 200 ÷ founders, the dead counting as 0.

Every claim is judged at span 7 and reported across span 1, 3, 7 and 13. The 9 % median and 2–30 % band
come from the field studies Minds 6 used.

1. **The field band** (`watch-winter.pilferage`).
   - World: `watch-winter` (theft-winter with watching on, every agent a watcher, `find` 0, no cheaters).
   - Holds when the pilferage rate is at least 9 % in at least 80 % of seeds (`range` on flags); fails
     otherwise.
   - Reported beside it:
     - a decomposition (burials seen per burial, raids per sighting, wasted raids per sighting);
     - the same world with stumbling added back (`watch-winter-stumble`, `find` 0.25);
     - Minds 6's stumbling-only rate for comparison.
2. **Andersson and Krebs under watching** (`watch-half.threshold`).
   - Worlds: `theft-winter-half` (half cheaters, `find` 0.25) against `watch-half` (the same with watching on
     and every agent a watcher), paired on seeds.
   - Holds when both hold in at least 80 % of seeds (`all_of`):
     - p_s ÷ p_o is lower under watching. A seed with p_o = 0 under both has no value.
     - The hoarders' advantage (hoarder survival per founder minus cheater survival per founder) is lower
       under watching.
   - Reported: whether p_s > p_o holds at all in either world. In Minds 6 it already failed: owners dug 1.5 %
     and thieves took 98 %.
3. **Producers and scroungers** (`watch-scroungers.frequency`).
   - Worlds: the winter field with `find` 0, everyone burying, and the watcher share s = 0.1, 0.2, …, 0.9.
     - **Watchers who also bury:** `watch-scroungers` at each s, with no cheaters.
     - **Pure scroungers:** `watch-scroungers-only` at each s, with cheaters at the same share s, so the
       watchers never bury.
   - The watcher advantage is watcher survival per founder minus non-watcher survival per founder.
   - Per seed, take the OLS slope of the advantage on s.
   - Theory predicts negative frequency dependence. The judge holds for a variant when the 95 % t confidence
     interval of the mean per-seed slope lies wholly below 0. It fails when the interval includes 0 or lies
     above it. Each variant is judged and reported separately.
   - The per-seed crossing (the advantage's first sign change from above 0 to at most 0, interpolated) is
     reported where one exists.
   - Our expectation, recorded before any run: pure scroungers show the prediction, and watchers who also
     bury don't, because watching costs them nothing. Either way it's reported.
4. **Usage** (`watch-winter.usage`).
   - In each watching preset except `watch-arena`, where the check is a test, some sugar is taken by a raid
     in at least 80 % of seeds (`range` on flags, `all_of` over presets).

**Reported, not judged:**
- Raid freshness: the age at raiding of the sugar raids take, and raids and wasted raids against span. This
  is our version of H&P's recovery by delay.
- Pilfers by source, seen against stumbled, in every world.
- The usage of the switch: the share of moves that targeted a seen cache.

**A check, not a claim** (`watch-arena`):
- This is Minds 6's four-agent arena with no cheaters, `find` 0.25, and half the agents watchers.
- It plays B&K's comparison of observers and non-observers in a closed room.
- Rust tests pin that a seen cache still in place is always taken on arrival, that an unseen cache is taken
  only through `find`, and that a wall between a burial and a watcher stops the sighting.

## Presets and page

**Presets.** Titles come from measured results, written after the runs:

| Id | World |
|---|---|
| `watch-winter` | theft-winter, watching on, all watchers, `find` 0 |
| `watch-winter-stumble` | the same with `find` 0.25 |
| `watch-half` | theft-winter-half (half cheaters, `find` 0.25) with watching on, all watchers |
| `watch-scroungers` | the winter field, `find` 0, half the agents watchers, no cheaters |
| `watch-scroungers-only` | the same with half cheaters (the watchers, so they never bury) |
| `watch-arena` | the four-agent arena, no cheaters, `find` 0.25, half the agents watchers |

- Source: "Bugnyar & Kotrschal 2002; Heinrich & Pepper 1998; Minds 8". Add Andersson & Krebs and Barnard &
  Sibly where relevant.
- The presets group under "Minds 8: watching" in the Minds menu.

**Page:**
- **Color mode "Watching":** watchers who also bury, pure scroungers (watchers who never bury), and
  everyone else. The default color mode picks it when watching is on.
- **Inspector:** the selected agent's remembered caches (site, owner, amount, age).
- **Charts:** pilfers by source (seen against stumbled), wasted raids, and watcher and non-watcher survival.
- **Live toggles:** `on`, `span` and `raid_when` are live, like `theft.find`. The CSV export handles a missing
  group (no watchers, or no non-watchers) without NaN, as Minds 6's fix did.
- **WASM:** goldens repeat the new native fingerprints in `crates/sugarscape-wasm/tests/web.rs`. No existing
  fingerprint moves.

## Verification

- **Unit tests:**
  - sighting along each lattice line, at vision's edge, and stopped by an opaque wall (not by a fence);
  - no self-sighting;
  - the span boundary (remembered at exactly `span`, forgotten after);
  - a raid's certainty;
  - a wasted raid falls through to stumbling;
  - the dig wins over a raid;
  - one take per arrival;
  - `raid_when: hungry` gating;
  - conservation of sugar with watching on;
  - nothing allocated or drawn with watching off (a run's rng state matches the same run without the
    section).
- **Golden fingerprints** for `watch-winter` and `watch-arena`, natively and in WASM.
- **The survey's claims** in `survey/src/claims/minds8.rs`, committed before any survey run.
- **A final whole-branch review,** as in every milestone.

## Docs

- A README section.
- The program document: status, results, and the next step (Minds 9, spatial P1b).
- A roadmap line.
- Spec amendments for every ruling made in planning or execution.

## Amendments (implementation)

Rulings made while building and surveying Minds 8, each with its reason. None changed a survey judge
or threshold.

**Mechanism.**

1. **`seen_entries` is counted after the tick-start sweep**, not at the tick's end. The plan placed
   the count there, where the sweep already visits every agent. The two differ only by that tick's
   expiries. It is a reported measure, never judged.
2. **Chapter II replacement agents are dealt `watches` by id, like founders.** A replacement has no
   parents, and Minds 6 deals `cheater` to it the same way. Consistency matters because watchers
   and cheaters must be the same agents at equal shares (pure scroungers). No watching preset uses
   replacement. A child born to parents still never watches.
3. **A dig also forgets the digger's seen entries at the site**, and counts in `seen_arrivals` if
   one was fresh. The spec forgets entries on any arrival on the site, and a dig is an arrival. So
   `seen_arrivals` can exceed raids + wasted raids + takes of 0 slightly.
4. **`raid_when: hungry` gates the raid, not only the going.** Under `hungry`, an arrival at or above
   R / 2 doesn't raid: it forgets nothing, counts nothing, and stumbles and harvests as if it had no
   entries. The spec puts `hungry` raiders "on the owners' terms", and owners dig only when hungry.
   As first built (by a dispatch that contradicted the spec), a fed watcher still raided whenever it
   arrived on a seen cache for the site's sugar. A 5-seed investigation counted 2 955 such summer
   raids a seed under `hungry`, each replacing a harvest, so `hungry` was not a no-summer-raiding
   switch. No preset uses `hungry`, so no preset moved.
5. **A seen cache the watcher's walk can't reach is given up.** Where a walk finds no path
   (`movement::arrive`), the walker forgets every owner's entry at the target, fresh or not, and
   counts nothing. The same investigation found 15 % of summer moves in `watch-winter` (2 508 a
   seed) targeting a seen cache with no path, mostly in pockets sealed off by crowds, since caches
   are buried where their owners stand. `join_sites` skips only the last failed target, so agents
   chained from one unreachable seen cache to the next. Own caches keep Minds 5's rule. The cost:
   a watcher gives up on a cache a later path might reach, and it gives up whenever a walk to a site
   holding its entries fails, even when the site was chosen for its sugar. Survival in the six
   presets moved by −2.1 to +5.0 points (5 seeds), mostly within noise.
6. **Minds 3's true value of a seen site** is the sum of the remembered (fresh) owners' caches
   still at the site, under the same gates as the believed value. The plan said the believed value,
   which would have made the stale-choice diagnostic blind to exactly what the spec stresses: an
   emptied cache still looks full.
7. **A survey-only probe, `World::probe_raid_harvests`** (hidden, not config, not hashed, off by
   default, like Minds 6's `probe_dig_at_reserve`). A raid that took something also harvests the
   site that tick, under the carrying limit with kept loot counted against it, and still draws no
   stumble. It is reported, never judged, and exists to measure what replacing the harvest costs.
   Under it, `Harvest::pilfered` and `gathered` can both be positive in one tick.

**Measures, tests and fixtures.**

8. **A `watcher_advantage` series** (watcher survival per founder minus everyone else's, founders by
   the id rule, the dead counting as 0, a group with no founders as 0). The `watch-scroungers`
   sweep reads it, because a sweep takes one metric and the raw count of watchers alive rises with
   the share by construction. It is claim 3's measure. A second sweep, `watch-span`, reads the
   first winter's pilferage rate against the span.
9. **Claim 3 is two claim ids**, `watch-scroungers.frequency` (watchers who bury) and
   `watch-scroungers-only.frequency` (pure scroungers), since the spec judges each variant
   separately.
10. **Reading the judges.** Under claim 2, a seed with p_o = 0 in only one world makes that world's
    ratio infinite; only p_o = 0 in both has no value (no seed had either). For the `range` claims
    (1 and 4), Weak (50–80 % of seeds) means the claim does not hold.
11. **The no-trajectory-change test was narrowed.** With watching on, a world is identical to the
    same world with watching off only until the first entry is remembered, since raids now change
    trajectories by design. Every with-watching-off identity (fingerprints, rng state, no
    allocation) still holds.
12. **Golden entries for all six presets**, not two, because `every_preset_has_a_golden_entry`
    requires one per preset. All are new; no existing entry moved. WASM pins repeat `watch-winter` and
    `watch-arena` as specified. The `watch-*` entries, which never existed on main, were re-recorded
    after rulings 4 and 5, natively and in `web.rs`.

**The page.**

13. **The Watching color mode wins over Strategy** in the default color mode. Pure scroungers are
    cheaters, so Strategy would hide the three-way split. A world with theft and watching opens in
    Watching colors.
14. **No TypeScript mirrors of the watching stats.** The page reads series by name, and has no mirror
    of Minds 6's theft stats either.
15. **No browser check was made** (none was available in the session). Unit tests cover the Rules
    group, the color mode, the legend, the inspector rows and the charts; a visual problem would
    show first on the deployed page.

**Sources.**

16. **Barnard and Sibly (1981)** are cited, in the sources of `watch-scroungers` and
    `watch-scroungers-only` and in claim 3's two claims, for "the hypothetical pay-off to scroungers
    increases with the number of producers" (p.543), with a stable mix at the ESS point. Andersson
    and Krebs (1978) are added to `watch-half`'s source. **Vickery et al. (1991)** stayed context
    and are cited nowhere in the model. Their "opportunistic forager that can both produce and
    scrounge but with reduced efficiency" (p.847) is the closest the theory comes to our watcher who
    also buries. Their finding that "when there is little incompatibility between producing and
    scrounging, opportunists will always be present, unless the producer is able to consume most of
    the patch without sharing" (p.847) fits our pre-run reason for expecting no fall in that variant
    (watching costs them nothing); it was not tested.

**The order of runs, disclosed.** The survey's judges and thresholds were in this spec from its
first commit (57ec71c), and the survey module transcribed them (8b1bf8c) before its first run. But
sanity runs came before that module: Task 5's (seeds 1–5, 200 ticks: pilferage, raids, wasted raids
and first-winter survival in each preset, e.g. `watch-winter` 0.90 % and 54.7 %), a mechanism
investigation (5 seeds) and Task 5b's re-run after rulings 4 and 5. Those numbers were seen before the
survey code was written. No judge or threshold was changed because of them.

**What the mechanism findings changed in how claims are read** (not their thresholds).

17. **The 5-seed investigation found no bug.** Sugar was conserved per agent turn (largest error
    7e-12 over about 154 000 turns). `watchers` 0, and `theft-winter` with `find` 0, are bit-identical
    to `cache-winter-even`. Every arrival with an entry forgets it, and the tick-start sweep keeps
    memory bounded.
18. **Watching's survival cost has two causes, each read as isolated by a switch.**
    - **Raids replace the summer harvest.** Seen caches out-rank every site (62 % of summer moves
      head for one), and a raid gathers nothing from its site. Summer gathering falls to 29 700 a
      seed against 42 300 without theft (a ledger estimate, within 83 a seed). The probe of ruling 7
      recovers it only to 34 800, so the replaced harvest is part of the gap, not all of it; the
      rest is likely the moves spent walking to seen caches. (Before rulings 4 and 5, the
      investigation recovered all of it on 5 seeds by also turning off summer targeting with
      `hungry`; that switch now behaves differently, so the rest is not claimed as isolated.)
    - **Watching gives little winter pooling.** Raids need a burial seen within the span, and few
      agents bury in winter. Turning stumbling on at tick 100 lifts `watch-winter`'s survival per
      founder from 0.526 to 0.817, and turning it off at tick 100 drops `theft-winter`'s from 0.857
      to 0.586, below no theft (0.700).

    So survival-based claims (2 and 3) are read as the net of what raids bring and what they
    displace, not as the value of information alone.
19. **Claim 1 is read as bounded by knowledge, not by action.** Almost every arrival with a fresh
    entry raids; the one-take rule and the carrying limit hardly ever bind (5 seeds). What bounds the
    rate is the share of caches seen buried within the span (19.6 % in summer, 3.6 % in winter at
    span 7), so the rate is reported across span (0.09–1.20 % at span 1–13).
20. **Claim 2's ratio half had a floor.** p_s ÷ p_o was already near 0 without watching (0.016), so
    it could hardly fall. The advantage half carries the information, and claim 2 is read as
    watching hurting hoarders through survival.
21. **Claim 3's pure scroungers take little.** A pure scrounger's raid takes 1.1 sugar on average,
    against 5.2 for a watcher who buries, likely because agents that never bury carry close to their
    limit. So that variant likely measures watchers with little room, not watching alone. No switch
    isolates it. The expectation recorded before the run (pure scroungers show the prediction) was
    refuted, and is reported as refuted.
