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
  flocks of house sparrows", *Animal Behaviour* 29 (1981), 543–550. **To fetch.** If it can't be found, claim 3
  cites producer–scrounger theory through whichever source we do find (Giraldeau and Caraco's *Social
  Foraging Theory*, 2000, or a review), and the spec's amendments say which.
- To look for, as context only: P. A. Bednekoff and R. P. Balda (1996a, b), cited by B&K for
  observational spatial memory in pinyon jays, Clark's nutcrackers and Mexican jays.

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
- `seen_entries`: entries held at the tick's end, summed over agents.
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
