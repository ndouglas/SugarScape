# Minds 8, second round: tests that can tell the hypotheses apart (design)

**Date:** 2026-10-01
**Amends:** `docs/superpowers/specs/2026-10-01-minds-8-watching-design.md` (the first round), on the same
branch (`minds-8`). This round runs before the branch merges.

## Why a second round

The first round's mechanism, page and presets are sound. Four independent audits, run before merging, found
no bug:
- **Measurement:** re-derived from the definitions alone, it matched every per-seed value.
- **Rulings:** none was wrong, and no verdict flipped under any variant tried.
- **Sources:** checked the mechanism and the test targets against the papers.
- **Design:** judged whether each test could discriminate.

The audit reports are kept with this round's ledger.

The audits did find that three of the five judged results were foreseeable from Minds 6's numbers, so they
weren't tests:
- **Claim 1 (the field band) was a strawman.**
  - The 2–30 % a day in Vander Wall and Jenkins (2003, pp. 656, 658, 663) measures loss of *artificial*
    caches to *all* pilferers, mostly rodents, per item per day.
  - The same paper calls intraspecific pilferage "usually low" in jays and nutcrackers.
  - Our measure was a ratio over every standing cache, so the winter stock, which watching can't reach,
    diluted it.
- **Claim 2's ratio half was at a floor, and close to built in.**
  - In Minds 6 owners already recovered under 2 % of their caches.
  - Watching makes a thief's per-visit find of a seen cache 1, the same as an owner's.
- **Claim 3 didn't test Barnard and Sibly's game.**
  - The "pure scroungers" still harvested sugar themselves, so scrounging didn't exclude producing.
  - Their loot was capped at about 1.1 sugar a raid by the carrying limit.
  - "Watchers who also bury" are Vickery et al.'s cost-free opportunists, a different game.

Two unstated mechanism choices also mattered:
- **A raid on every arrival,** even where harvesting the site paid more. Letting agents choose the better
  of the two raised `watch-winter` survival 8 points and cut the watcher lead from 8 points to 1.
- **Seen caches valued with no regard to carrying room.** This explains the pure scroungers' tiny takes.

From the sources, the mechanism is generous to watchers:
- perfect attention and exact memory;
- cachers that never hide and owners that never defend (both P3);
- a span at the generous end of what's known.

Read the results as an upper bound on what watching can do.

**The first round stays on record.** Its verdicts are what the first design measured, reported with the
audit's explanation, not erased.

## What changes in the model

### Named switches

All of them are in `watching`:

| Field | Values | Default | Live? | Meaning |
|---|---|---|---|---|
| `raid_if` | `better` \| `always` | `better` | live | On arriving at a site with fresh entries, `better` raids only when the remembered amount there (summed over owners) is at least the site's welfare value as rule M counts it. Otherwise the agent harvests as usual and keeps its entries. `always` is the first round's rule. |
| `value` | `amount` \| `room` | `amount` | live | A seen cache's candidate value. `amount` is the remembered amount, summed over owners. `room` is that capped at the agent's room under the carrying limit (infinite with no limit, or under `loot: eat`). |
| `who` | `share` \| `hoarders` \| `cheaters` | `share` | reset-only | Which founders watch. `share` uses `watchers` and the id rule, as now. `hoarders` makes every non-cheater watch, and `cheaters` every cheater, whatever `watchers` says. |
| `scrounge` | `harvest` \| `forgo` | `harvest` | live | What a scrounger (an agent who watches and cheats) does on a tick when it holds a fresh entry. `harvest`: as now. `forgo`: its candidates are only its seen caches and staying put. On staying put it harvests nothing that tick. Holding no fresh entry, it forages as usual. |

**The default changes to `raid_if: better`.**
- The user agreed to this before this design was written.
- It's how a rule-M agent chooses. The first round's rule stays available as `always`.
- Every `watch-*` preset's numbers change, and their golden entries (branch-only) are re-recorded.
- With `watching.on` false, nothing changes anywhere. No existing golden entry from main may move.

**`scrounge: forgo` is Barnard and Sibly's assumption, scrounging excludes producing,** stated in Vickery et al.'s
words (p. 848). It is the literal default for the new scrounger presets only. The config default stays
`harvest`, so the first-round worlds are unchanged except for `raid_if`.

### New presets

Each preset gets a title written after the runs.

| Id | World |
|---|---|
| `watch-scroungers-forgo` | The winter field, `find` 0, everyone burying except the scroungers. Half the agents are scroungers (`cheaters` 0.5 and `watchers` 0.5, the same ids). The scroungers use `scrounge: forgo`, and the world uses `loot: eat`. |
| `watch-ak` | The world chosen by claim 2's calibration (below), with watching on for everyone. |

## The pre-mortem

This is the step the first round lacked. For each judge below, the table gives what each hypothesis
predicts, checked against Minds 6, the first round and the audits' numbers. Where a number was already
seen, it says so.

| Judge | If true | If false | Already seen? |
|---|---|---|---|
| 1a | Watching's fresh-cache hazard is above stumbling's. | It is at or below stumbling's. | Partly. Under `raid_if: always` the audit estimated about 4.9 %/day for watching against "about half" for stumbling. Under `better`, raids fall about 20 % (audit). The direction is likely but not certain. Disclosed. |
| 2a | Watching flips A&K's condition in a world where it held. | It doesn't flip. | No. No world with p_s > p_o has been run with watching. |
| 2b | The sign of the hoarders' advantage follows the sign of p_s/p_o − 1 across the span sweep. | The signs don't agree. | No. |
| 2c | Being watched harms hoarders. | It doesn't. | No, since the factorial hasn't been run. |
| 2d | Hoarders' own raiding harms them. | It doesn't. | No. |
| 3a | Scrounger advantage falls with share. | It is flat or rises. | Not for `forgo`. For watchers who also bury, the first round saw negative slopes at every span (not detected). |
| 3b | There is a stable mix: scroungers fitter when rare, less fit when common. | No crossing. | No for `forgo`. |
| 3c | A social dilemma: watchers individually ahead while the world's survival falls with share. | One part or both fail. | Yes, under `always`, by the design audit (survival 0.714 → 0.551). Under `better` the audit's variant cut the lead to 1 point. Judged, but flagged as partly seen. |

Floors, ceilings and caps checked:
- **Claim 2's world must have p_s > p_o before watching.** That's the precondition (below).
- **Survival at tick 200 saturates near 1 in arenas and floors near 0.46** where nobody caches. Fitness is now
  agent-ticks alive per founder over ticks 1–200, divided by 200, which is continuous.
- **The carrying limit capped scrounger loot.** The scrounger world uses `loot: eat`.
- **In an arena nearly every agent survives,** so ticks alive can't discriminate there. For claim 2 in an arena
  world, fitness is wealth per founder at tick 200, Minds 6's arena rule (holdings, caches and stomach of the
  living ÷ founders). In a field world it is ticks alive per founder.

## Questions and judges

**Setup.**
- 200 ticks: a summer (0–99) and the first winter (100–199).
- Seeds 1–20 for claims 1 and 2, and seeds 1–60 for claim 3 (power: a minimum detectable slope of about 0.045
  at the first round's per-seed SD of about 0.11).
- Span 7 is judged. Spans 1, 2, 3, 7 and 13 are reported; span 2 is Bednekoff and Balda's pinyon jays' exact
  recall.
- Thresholds are ≥ 80 % of seeds (16 of 20, 48 of 60).
- Verdicts are Holds, Fails, Untestable (a precondition not met) or Inconclusive (claim 3a only, defined
  there).

### Claim 1: fresh caches

**The measure: the cohort hazard.**
- Take each (site, owner) cache first created in ticks 1–90, a summer cohort that can be followed for 7 ticks
  before winter.
- P_d is the share of the cohort's caches that a non-owner takes any of within d ticks of creation (d = 1, 3
  and 7).
- The hazard is h = 1 − (1 − P_7)^(1/7).
- The fate log gives the times. A cache that its owner digs up, or that is lost, before any pilfer counts as
  not taken.

**1a. Watching takes fresh caches faster than stumbling** (`watch-winter.fresh`):
- h(`watch-winter`) > h(`theft-winter`), paired on seeds.
- Holds in ≥ 16 of 20 seeds.

**Reported, not judged:**
- P_1, P_3, P_7 and h for `watch-winter`, `watch-winter-stumble` and `theft-winter`.
- Heinrich and Pepper's within-species next-day figure, as context for P_1: at most 15 of 42, which is 36 %.
- Vander Wall and Jenkins's band, cited for what it measures. It is not a target.
- **The bottleneck, classified by a rule fixed now:**
  - P(seen) = burials seen ÷ burials.
  - P(raid | seen) = caches seen that are raided within `span` ÷ caches seen.
  - "Knowledge-bound" iff P(seen) < P(raid | seen), else "action-bound".
- The first round's stock rate, kept as context.

### Claim 2: Andersson and Krebs under watching

**The precondition: a calibration whose rule is fixed now.**
- Take the first world from this list in which p_s > p_o holds in ≥ 16 of 20 seeds without watching:
  1. `theft-arena-2`;
  2. `theft-winter-half` at `find` 0.02;
  3. `theft-winter-half` at `find` 0.05;
  4. `theft-winter-half` at `find` 0.02, with owners digging below their whole reserve (the survey probe);
  5. the same at `find` 0.05.
- p_s and p_o are measured as in Minds 6: amount-weighted, with sugar still buried excluded.
- That world becomes `watch-ak` (with watching on for everyone).
- If no world qualifies, claims 2a–2d are **Untestable**, not failed.

**2a. Watching flips the condition** (`watch-ak.flip`):
- With watching on, p_s ÷ p_o < 1 in ≥ 16 of 20 seeds.

**2b. Fitness follows the condition** (`watch-ak.sign`):
- Runs over watching off and spans 1, 3, 7 and 13, on seeds 1–20.
- In each run, the sign of the hoarders' advantage (hoarder minus cheater fitness) should agree with the sign
  of p_s ÷ p_o − 1. A run with an undefined ratio is excluded.
- Holds when they agree in ≥ 80 % of runs.

**The attribution factorial,** in `watch-ak`'s world on the same seeds, with `who` set to none (watching off),
`hoarders`, `cheaters` and everyone:
- **2c. Being watched costs hoarders** (`watch-ak.watched`):
  - Holds when the hoarders' advantage under `who: cheaters` is below its value with watching off by at
    least 0.05.
  - Fitness units, paired, in ≥ 16 of 20 seeds.
- **2d. Their own raiding costs hoarders** (`watch-ak.raiding`):
  - Holds when hoarder fitness under `who: hoarders` is below its value with watching off by at least 0.05.
  - Paired, in ≥ 16 of 20 seeds.
  - 2c and 2d may both hold.

### Claim 3: producers and scroungers

**The worlds:**
- **Variant forgo:** `watch-scroungers-forgo` at scrounger shares s = 0.1, 0.2, …, 0.9 (`cheaters` =
  `watchers` = s).
- **Variant bury:** `watch-scroungers` at watcher shares s = 0.1–0.9, with no cheaters.
- Seeds 1–60.
- The advantage at a share is the scrounger's (or watcher's) fitness minus the others' fitness.

**3a. Frequency dependence** (`watch-scroungers.frequency`, two claim ids, one per variant):
- Take the OLS slope of the advantage on s per seed. Then compute the 95 % t CI of the mean slope, and the
  90 % CI.
- **Holds** when the 95 % CI lies wholly below 0.
- **Flat** (reported as Fails, labeled "flat") when the 90 % CI lies within ±0.05.
- **Inconclusive** otherwise.
- A 95 % CI wholly above 0 is Fails (rising), as the pre-mortem's 'flat or rises' states (clarified before any run).

**3b. A stable mix** (`watch-scroungers.mix`, variant forgo only):
- The advantage is above 0 at s = 0.1 and below 0 at s = 0.9.
- Each must hold in ≥ 48 of 60 seeds.
- The per-seed crossing is reported.

**3c. A social dilemma** (`watch-scroungers.dilemma`, variant bury only). Holds when both hold:
- The watchers' advantage, pooled over shares (the per-seed mean over s), has a 95 % CI above 0.
- World fitness (all agents) falls with s: the per-seed OLS slope's 95 % CI lies below 0.

### Usage

Usage is no longer a claim. It is a check in the survey output: raids take sugar in every watching preset.
It is kept outside the claims table.

### Also reported, not judged

- Every claim under `raid_if: always`, `value: room` and spans 1, 2, 3, 7 and 13.
- The first round's probe (`probe_raid_harvests`).

## Corrections to the first round's text

The descriptions, the README, the program doc and the spec's amendments get corrected. These statements are
replaced:
- "watching can't reach field rates";
- "bounded by knowledge, not action" (the first round was action-bound: 0.88 seen, about 0.33 raided);
- "no frequency dependence" (it was not detected; the slopes were negative at every span);
- "pure scroungers did worse at every share" stated as a fact about scrounging;
- Minds 9's rationale where it cites those statements.

The first round's verdict table stays in the program doc. It is labeled as the first design's, with the
audits' explanation and a pointer to this round.

## Verification

- **Unit tests for each switch:**
  - `better` declines when the site is worth more, and keeps the entries;
  - `room` caps the value;
  - `who` assigns watchers;
  - `forgo` leaves only seen caches and staying put as candidates, and harvests nothing on staying put while
    holding a fresh entry.
- **Conservation** under each switch.
- **Defaults off reproduce main's worlds bit for bit.**
- **Golden entries:** `watch-*` re-recorded (branch-only), plus new entries for the new presets, natively
  and in WASM.
- **The survey judges are committed alone,** before any survey run, with this table's pre-mortem copied into
  the module doc.
- **The calibration for claim 2 runs first,** under its fixed rule, and is committed with its result before
  claims 2a–2d run.
- **A final whole-branch review.**

## Amendments (implementation)

Rulings made while building and surveying the second round, each with its reason. Only ruling 6 touches a
judge, and it was made and committed (55a8c23) after the judges (e4edce3) and before any survey run. No
threshold was changed. The results are in `survey/out/minds8b-results.md`, `minds8b-calibration.md`,
`minds8b-presets.md` and `minds8b-usage.md`.

**Mechanism.**

1. **The watcher statistics need a real founder split.** The core's watcher-against-others series were gated
   on 0 < `watchers` < 1, which ignores `who`. They now need some founders who watch and some who don't,
   counted with the same rule that deals watching (`who` and the cheater id rule). The page's gates (the
   charts and the Watching color mode) use the same split, exactly: the id rule deals ⌊n·s⌋ of n founders, so
   a split needs 0 < ⌊n·s⌋ < n. The survey computes from the agents, so only the page's series are affected.
2. **A forgoing scrounger that is fed, under `raid_when: hungry`, forages as usual.** Its seen caches aren't
   places to go while it is fed, and the literal "holding a fresh entry" would leave it no choice but to sit
   and starve. No preset combines `hungry` with `forgo`.
3. **In the forgo list, staying put is valued 0, and a forgoing scrounger skips the `better` test.** The site
   it forgoes is worth nothing to it, which follows from "scrounging excludes producing". The cost if wrong:
   forgoing scroungers raid a little more readily. It is also why variant forgo is identical under `raid_if:
   always`, and, with `loot: eat` making room infinite, under `value: room`.
4. **A forgoing scrounger harvests nothing on any tick it holds a fresh entry and doesn't raid,** including
   the walking ticks on the way to a seen cache and a wasted raid. The first build harvested on those walking
   steps. A reviewer's minor point was promoted to a fix because the spec's `forgo` is "no harvest" while an
   entry is fresh. Its own dig is unaffected, and under the survey probe a raid that took something still
   harvests.
5. **Minds 3's diagnostics apply the same cap under `value: room`.** A seen cache's true value is capped at
   the room, as its believed value is, so a capped choice isn't miscounted as a stale one.
6. **`caching.dig_below: half | reserve`** (default `half`, live). The calibration's first qualifying world
   was item 4, which uses the survey probe (`probe_dig_at_reserve`) to make owners dig below their whole
   reserve. A probe is not config, so a preset couldn't express that world, and the app's `watch-ak` would
   have differed from the survey's. `dig_below: reserve` makes `hungry()` use R, as the probe and
   central-place foraging already did. The probe is kept, and `raiding()` stays at R / 2 under both values.
   Main's worlds are unchanged at the default. `theft-winter-half`'s description names the setting, and the
   Rules panel offers it as **Dig below**.

**Judges and measures.**

7. **3a: a 95 % CI wholly above 0 reads Fails ("rising"), not Inconclusive.** The pre-mortem lists "flat or
   rises" as the outcome if the claim is false, so a clear contradiction of the prediction is a failure. The
   judge's text has been amended to say so. The order of precedence is Holds, then flat, then rising, then
   Inconclusive, so an interval inside (0, 0.05] reads flat. Ruled before any run.
8. **Untestable when fewer than 5 seeds (or runs) have a value,** the convention of Minds 6–8, as well as when
   the precondition isn't met. No claim read Untestable.
9. **"Raided within span" is counted from the raider's sighting,** when its memory starts, not from the
   cache's creation. The span runs from the sighting, and sightings happen at burial, so the two are nearly
   identical. It affects only the reported P(raid | seen).
10. **The bottleneck compares the medians of the per-seed P(seen) and P(raid | seen).** The class is printed
    beside a count of the seeds that are knowledge-bound (0 of 20 in every world and setting).
11. **The cohort counts cache instances.** A (site, owner) cache runs from the burial that creates it until it
    is emptied, and a later cache at the same site is a new one. A cache emptied and buried again on the same
    tick reads as one, which is rare.
12. **The usage check needs raids to take sugar in at least 16 of 20 seeds** in each watching preset. The
    implementer set this threshold, and it is disclosed here. Every preset passed in 20 of 20.
13. **The calibration ran once, under its committed rule** (9ec279e, result b966312), before claims 2a–2d ran.
    The judges read its result as a constant (`WATCH_AK_ITEM`, item 4), and the claim run reproduced it (20 of
    20).
14. **A reported-only `--presets` section was added after the run.** It measures each watching preset as set,
    beside the same world with one switch changed, so that the presets' descriptions rest on measured figures.
    It touches no judge. Survival counts at ticks 100 and 200 were added to the survey's per-run record for
    it.
15. **The first round's five claims were rerun under the new defaults,** as context. Their judges are
    unchanged.

**What the results changed in how claims are read** (not their verdicts).

16. **Fitness in the field is compressed.** Ticks alive per founder ÷ 200 sits near 0.9, because most agents
    who die do so in the winter (in `watch-winter`, at about tick 150 on average). So 2c's and 2d's 0.05 per
    seed was a large effect on this scale, and survival at 200 moves several times as much. The paired means
    (2c 0.012, 95 % interval 0.008 to 0.017; 2d 0.003, −0.002 to 0.007) are reported beside the verdicts.
17. **2b's 62 of 100 under all three span-7 settings is a coincidence of totals.** The runs were recounted,
    and the per-run-set counts differ.
18. **Spans 1 and 2 reverse four results.** Watching's fresh-cache hazard falls below stumbling's (1a), the
    condition doesn't flip (2a), variant forgo's shortfall shrinks with share (3a, rising), and variant bury's
    watcher lead isn't detected (3c fails at spans 1, 2 and 3: pooled 0.0007, −0.0015, −0.0027). Span 7 sits
    at the generous end of what the sources support, so the docs present the judged results as an upper bound.
19. **`watch-scroungers-only`'s shortfall predates watching.** With watching off, the same agents survive 48 %
    against the hoarders' 70 %, and with it 49 % against 65 %. The first round's "pure scroungers trail at
    every share" describes agents who never cache, not scrounging. The preset's title and description say so.

**Not re-timed.** The second round measured no costs. The first round's cost table stands, labeled as the
first design's (`raid_if: always`), and `watch-scroungers-forgo` and `watch-ak` have no cost figure.
