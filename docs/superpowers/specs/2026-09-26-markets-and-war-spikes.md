# Markets and War: spikes, and the Markets storyboard

**Date:** 2026-09-26
**Builds on:** `2026-09-26-sugarscape-series-plan.md`. Spikes only: nothing here is rendered, and every
figure is re-measured by the episode's own `claims.py` before it becomes a caption.

Seeds 1–20 throughout (the studio's convention), medians with IQRs. The scripts that produced these
numbers were throwaway spike scripts outside the repository. Two results were re-derived
independently with the CLI on main (9c3deb0): Figure IV-6 over 20 seeds, and III-9 seeds 20 and 3.

## War (III-9, III-11, III-12, III-14)

The survey's combat checks only ask whether anything happens ("at least one combat death"), so they
said nothing a caption could rest on. Measured properly:

| Candidate claim | Measured | Verdict |
|---|---|---|
| III-9: one tribe conquers the other | ≥ 90 % one tribe in 14/20 by t = 2000 (8/20 by t = 1000; median t = 882); the other tribe extinct in 10/20 | **Holds, at 2000 ticks** |
| III-9: "tribes rarely meet" (the survey's wording) | 5 kills by t = 500 (IQR 3–20), but 120 by t = 2000; enemies within sight by about t = 100 on 20/20 | **Reword**: quiet, then sudden. The survey's "median 5 kills" is only true at 500 ticks |
| III-9: all or nothing | 14 seeds end with 111–135 kills (a whole tribe); 6 stay at 11–21 with both tribes near 110 | Holds |
| III-9: one warlord | in conquest seeds the top killer makes about 70 % of kills (51–95 %); 88 % of victims die on their own hill | Holds |
| III-9: winners get rich | richest Flump 1.0 × 10⁵ sugar vs 4.4 × 10³ without combat; Gini 0.46 vs 0.32; top 10 % hold 35 % vs 17 % | Holds |
| III-11: "prolonged battle fronts" | about 1,000 kills per 100 ticks forever, both tribes at 200 (by replacement), but kills spread evenly over the map: 26 % within 5 cells of the halfway line vs 12 % of Flumps | **Fails: no front.** Victims are a median 2 ticks old: replacements land among enemies and die at once |
| III-14: conquest and conversion together | 352 kills in about 100 ticks; 400 → 5 Flumps (vs 238 with culture alone); 104 tribe changes vs 11,500 without combat | **Fails.** Random tags leave many Flumps one flip from the other tribe; converts fight their own block. Combat suppresses conversion |
| III-12: colliding waves | each hill ≥ 96.5 % its own tribe on 20/20; enemies touch only at the hill edges | **Fails** (already known) |

Spot check with the CLI: III-9 seed 20 has one tribe gone by t = 500 (123 combat deaths). Seed 3 has
both tribes alive at t = 2000 (114 and 112, 13 combat deaths).

**So:** there is a War episode, but not the one the preset descriptions promise. Its story is "for
hundreds of ticks nothing happens; then one Flump starts, and one tribe is gone", with a winner about
twenty times richer. That returns to the inequality thread. III-11's endless war with no front can
close the episode honestly ("with replacements, nobody ever wins"). III-12's missing collision and
III-14's civil war go to the finale. Footage: seeds 20, 12 and 8 conquer early (the killing starts
around ticks 104, 322 and 295); seeds 3, 7, 15, 17 and 18 never do.

**The book, checked** (`papers/sugarscape/`, pp. 83–92 and Appendix B):
- **III-9 reproduces.** The book calls it "the stunning blitzkrieg", explains it as "increasing
  returns … the bigger you are the faster you grow", and lists three outcomes: Blue wins, Red wins, or
  small colonies coexist, each on its own peak. That matches our 14 conquests and 6 standoffs in 20.
- **III-11 does not.** The book's replacements are "random agent[s] of the same tribe", and rule R
  (p. 32–33 and Appendix B) states a "random position on the sugarscape". Our engine does exactly
  that. The book reports "coherent battle fronts. Penetration is minimal … a prolonged war of
  attrition". Under its stated rule, there is no front.
- **III-14 does not.** The book says "everything exactly as in animation III-9 … except that cultural
  processes are unfolding" and tells of invaders converted before they conquer. With the stated random
  tags we get a civil war instead. The book mentions unanimous tags (all 0s against all 1s) only in a
  footnote, as an alternative (n. 20).
- **Framing this suggests: deterrence.** Rule C never lets a Flump attack an equal, or attack when a
  richer enemy would see it, so equals never fight. War comes only when it is lopsided, and then it is
  quick. The long war of attrition is the part the book claims and its stated rules don't produce.

Next: named switches, with the literal rules as the default: replacements placed near their own tribe
(III-11), and unanimous starting tags (III-14). Then a 20-seed sweep of each, to see whether the
book's fronts and conversions come back.

Engine follow-ups (not studio work): the `iii-11-combat-fixed` and `iii-14-combat-culture` preset
descriptions claim fronts and "conquest and conversion together", and both fail. Their descriptions
should say so, as `iii-12-collision`'s does. The survey's III-9 sentence needs "by t = 500".

## Markets (IV-1 to IV-6, IV-13)

> **Corrected after the review (2026-09-27).** The shuttling numbers below counted a Flump as
> changing hills whenever it crossed the midline, so walks along the border counted too. Counting
> only walks between sites deep in each hill (`markets.DEEP`), 28 % shuttle without trade and 29 %
> with it. The episode says "about a quarter" and "about as many Flumps still shuttle", not "about
> half" and "more, not fewer". The rest of this record is kept as measured at the time.

| Candidate claim | Measured | Verdict |
|---|---|---|
| Without trade, Flumps shuttle between sugar and spice (iv-1) | 49 % (46–52 %) cross sides at least twice in t = 100–200; ≥ 50 % on 9/20 | **Reword**: "about half" |
| Two ways to starve | 400 → 120 by t = 1000; deaths: 148 short of sugar, 132 short of spice | Holds. It contradicts the book's "most agents never suffer this fate" |
| Neighbors trade | 342 exchanges at t = 1, about 25 a tick from t = 100; 97 % of Flumps trade at some point | Holds |
| The sugar-hill Flump trades with the spice-hill Flump | 1.5 % of exchanges cross hills: partners are neighbors | **Fails.** Say "neighbors swap what they have too much of" |
| Nobody sets the price, yet it settles near one | geometric mean 1.00 in every 50-tick block; mean ln price over t = 500–1000 within ±0.05 on 20/20 | Holds |
| …and prices agree more and more | spread (sd ln price) 0.455 → 0.026 (IQR 0.023–0.034) by t = 950–1000; falls on 20/20 | Holds (the book's "about 0.05" is twice ours) |
| Trade feeds more Flumps (book settings) | 62.6 vs 54.1 over t = 200–300; higher on 20/20 | Holds |
| …"on every seed at every vision" (the series plan and survey) | trade higher in 118 of 120 seed–vision pairs; seed 10 at mean vision 2 goes the other way (42.3 vs 43.9), seed 11 at 5 ties | **Fails as worded.** Re-derived with `sugarscape sweep --builtin fig-iv-6 --seeds 20`: 59 of 60 even in the sweep's own seeds 1–10, so the survey's "all 10 seeds" no longer holds on main |
| Trade does not replace travel | with trade 66 % shuttle (vs 49 %); ≥ 50 % on 20/20 | Holds, and it's a surprise |
| Trade makes wealth less equal (Figure IV-13's setup: lifetimes 60–100, replacement) | Gini of sugar + spice higher with trade at every mean vision 1–13, on 20/20 each (0.365 vs 0.320 at vision 3) | **Holds**: the thread's caption |
| The market never clears | Flumps trade 9.7 % (8.3–11.8 %) of what a clearing auction would move, on 20/20 | Holds |

**Where the book does not reproduce** (for the finale):
- **Trade volume** is about 5× lower: 31,300 trades over 1000 ticks against the book's "nearly
  150,000".
- **Figure IV-6's carrying capacity** is about half the book's (36 → 71 without trade and 44 → 76
  with, vs roughly 80 → 175 and 95 → 180). Starting with 800 Flumps instead of 200 recovers the book's
  trade volume. That is a hypothesis about the book's setup, not a finding.
- **Figure IV-13's shape** doesn't match. The direction ("trade raises inequality") reproduces, but the
  lines don't cross and the gap narrows instead of widening.

## Order

Markets becomes episode 6: its claims are the strongest, and its studio work (a second good, trade
arcs, a price chart) is what Credit and the finale's VI-2 and VI-3 scenes need too. War follows as
episode 7 with the story above, and its winner's wealth keeps the inequality thread going.

## Markets storyboard (proposal, about 80 s)

Shots: `spice` (a close-up world of iv-1), `shuttle` (iv-1, 400 Flumps, no trade), `swap` (a close
pair trading), `market` (iv-3-trade), `apart` (the same seed without trade), `unequal` (Figure
IV-13's setup with and without trade).

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | spice | "Now there's a second food: spice." | close-up; spice gumdrops (a new color) on the far hill |
| 2 | two | "A Flump needs both. Run out of either, and it's gone." | close-up; a two-good belly meter |
| 3 | shuttle | "Without trade, about half the Flumps walk back and forth between the hills." | wide `shuttle`; rings on the shuttlers |
| 4 | cost | "Of 400 Flumps, about 120 survive." | wide; counter |
| 5 | swap | "Now let neighbors swap what they have too much of for what they lack." | close-up `swap`; a gumdrop arcs each way |
| 6 | haggle | "They settle on a price between what each thinks it's worth." | close-up; a price tag over the pair |
| 7 | busy | "Hundreds of trades a tick." | wide `market`, early ticks; trade arcs |
| 8 | price | "Nobody sets the price. It settles near one sugar per spice…" | wide; price chart (from `stats`) |
| 9 | agree | "…and the prices agree more and more." | wide, fast; the chart's spread narrowing |
| 10 | more | "Trade feeds more Flumps: in 118 of 120 worlds." | bars, trade vs none |
| 11 | travel | "But it doesn't stop the walking: more Flumps shuttle, not fewer." | wide; rings |
| 12 | unequal | "And it makes them less equal, in all 20 worlds." | bars: Gini with and without |
| 13 | question | "Trade made the pie bigger. / It didn't share it out." | title |
| 14 | end | "Markets — after Epstein & Axtell, 1996 / ndouglas.github.io/SugarScape" | title card |

"Never clears" and the book's missing volume and capacity are held back for the finale.

**New studio work:**
- **Dump:** today it records only good 0. It needs spice per cell and spice capacity, each Flump's
  spice and spice metabolism, and trades per frame as `(buyer, seller, price, amount)`, merged per pair
  per tick. Starvation should record which good ran out. The price chart needs nothing new: `stats`
  already has `mean_log_price`, `sd_log_price` and `trade_volume`.
- **Blender:** a spice gumdrop and a spice felt tint, a two-good belly, trade arcs, a price chart
  panel, and a `rings` variant for shuttlers.

## Markets tune (proposal)

*The Spice Rag*, a slow rag in F ("not fast", after Joplin):
- **Voices:** honky-tonk piano, banjo and tuba, with a clarinet taking the melody in the B strain.
- **A strain:** the tuba's oom-pah bass alternates between two chord roots, the walk between two hills.
- **B strain:** moves to D minor for "less equal".
- **Trio:** in B♭, for the market settling.
- **Stings:** a small bell as the price settles (beat 8), and a clarinet drop on beat 12.

It's a new character after the gånglåt, waltz, chorale, passacaglia and canzona. War keeps its
suggested 5/4.
