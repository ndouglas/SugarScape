# The core Sugarscape series — plan

**Date:** 2026-09-26
**Builds on:** `2026-09-25-flump-studio-design.md` (the studio and its constraints, which bind every
episode: every Flump behavior is a real run; every captioned claim is measured over 20 seeds before
it is rendered, and a caption changes if its claim does not hold).

## Goal

A series of short explainers (60–100 s each) covering *Growing Artificial Societies*' sugarscape,
Chapters II–VI, one phenomenon per episode, ending with an episode on the book's results this
engine does not reproduce. The other models (Schelling, Ring World, the Anasazi, civil violence,
tag cooperation, spatial games, N goods) are a separate series.

## Conventions

- **Honest captions.** Each episode has a `claims.py`; `measurements.md` records every verdict.
  Where the book's story and the measured model differ, the video follows the measurements and may
  say so (the Seasons episode's lesson — seasons select on need, not sight — came this way).
- **A tune per episode**, original, in ABC (`tune.py`), with stings cued to beats. Each episode's
  music has its own character (5/4 for war has been suggested).
- **Preview size (960 × 540) is the shareable size**; everything needed to re-render at 1080p stays
  in the repo and runs deterministically.
- **The thread:** the pilot asks where inequality comes from; later episodes return to it
  (inheritance, tribes, markets, credit) before the finale asks how much of the book's story is in
  its stated rules.

## Episodes

| # | Episode | Book | What emerges (holds over 20 seeds in the survey) | New studio work |
|---|---|---|---|---|
| 1 | Sugarscape ✅ | II-1–5 | Selection on sight and thrift; crowding on the hills; skewed wealth | — |
| 2 | Seasons ✅ | II-7 | Migration without planning; seasons cost a third of the Flumps and remove every hungry one; needing little matters far more than where or how rich a Flump starts | Frost, season card, rings, counter, survival panel |
| 3 | Pollution ✅ | II-8 | The best land gets the dirtiest; Flumps flee the hills but the mess follows them; a quarter of the Flumps are lost, the hungry and then the far-sighted first; survivors end up more equal because there are fewer of them | Pollution in the dump; soot on the felt; a hills gauge; bar panels |
| 4 | Inheritance ✅ | III-1–4 | Pairing, children, ~60 generations; kept in the family, sugar feeds nearly four times as many Flumps, but the Gini more than doubles and fortunes persist across generations | Sex and parents in the dump; family-line colors; bequests; twin close-ups |
| 5 | Tribes ✅ | III-6 | Local convergence, global polarization: each hill becomes one tribe (18 of 20), but one tribe takes the world only half the time (10 of 20; the hills split in 9); tribes change nothing else | Tags and tribe in the dump; live tribe colors; traits pill; neighbors-alike gauge |
| 6 | Markets | IV-1–6, IV-13 | Spice on the far hills; about half the Flumps shuttle; neighbors trade; the price settles near one and prices agree more and more; trade feeds more Flumps (118 of 120 runs, not every one) but doesn't stop the walking, and makes them less equal (20 of 20) | A second good in the dump; trades; trade arcs; a price chart |
| 7 | War | III-9, III-11 | Quiet for hundreds of ticks, then one warlord: a tribe is wiped out or nearly (≥ 90 % one tribe in 14 of 20 by t = 2000); the winner ends about 20 times richer; with replacement the war never ends, but there is no front | Attacks in the dump; a pounce |
| 8 | Credit | IV-5 | Older Flumps lend, younger borrow to have children; a lending hierarchy | Loan lines; an age cue |
| 9 | Contagion | V | Immune systems learn; endemic disease; a novel disease sweeping an unprepared society (McNeill) | Infections in the dump; a sick tint |
| 10 | What didn't reproduce | VI + findings | Everything together, then the honest part: under the stated rules the book's VI-2 crash, VI-3 doubling, travelling waves, one-tribe dominance (already shown in Tribes), falling foresight, III-11's fronts, III-14's conquest and conversion (a civil war instead), III-12's collision, and Chapter IV's trade volume and carrying capacity don't reproduce | Side-by-side Compare in the cut |

Episode order after Seasons is open; Inheritance follows on most directly from the pilot's question. Markets moved ahead of War after both were spiked (`2026-09-26-markets-and-war-spikes.md`): its claims are the strongest and its studio work serves Credit and the finale too.

## Per episode

A short brainstorm (storyboard, the claims each caption rests on, the tune), then: shots checked
against their dumps, `claims.py` measured, visuals, beats, tune, a preview for review, and the
shareable cut in `~/Movies/Flump Studio/`.
