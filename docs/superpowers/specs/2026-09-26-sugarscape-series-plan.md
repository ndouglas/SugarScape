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
| 2 | Seasons ✅ (in production) | II-7 | Migration without planning; seasons cost a third of the Flumps and remove every hungry one; needing little matters far more than where or how rich a Flump starts | Frost, season card, rings, counter, survival panel |
| 3 | Pollution | II-8 | Harvesting and eating pollute; Flumps shun polluted sites; diffusion spreads it; carrying capacity falls | Pollution in the dump; stains on the felt |
| 4 | Inheritance | III-1–4 | Pairing, children, generations; vision rises and metabolism falls; inherited sugar raises the Gini | Sex, parents and children in the dump; births beside parents; sugar passing to heirs |
| 5 | Tribes | III-6 | Neighbors copy cultural tags; tribes form; each hill tends to one. Finding: global one-tribe dominance holds on only about half the seeds | Tags and tribe in the dump; yarn color by tribe |
| 6 | War | III-9–14 | Conquest; fronts that hold; conversion when culture and combat run together (the book's colliding waves don't reproduce as set up) | Attacks in the dump; a pounce |
| 7 | Markets | IV-1–6 | Spice on the opposite hills; trade; prices near one; trade raises carrying capacity on every seed at every vision | A second good; trade arcs; a price chart |
| 8 | Credit | IV-5 | Older Flumps lend, younger borrow to have children; a lending hierarchy | Loan lines; an age cue |
| 9 | Contagion | V | Immune systems learn; endemic disease; a novel disease sweeping an unprepared society (McNeill) | Infections in the dump; a sick tint |
| 10 | What didn't reproduce | VI + findings | Everything together, then the honest part: under the stated rules the book's VI-2 crash, VI-3 doubling, travelling waves, one-tribe dominance and falling foresight don't reproduce | Side-by-side Compare in the cut |

Episode order after Seasons is open; Inheritance follows on most directly from the pilot's question.

## Per episode

A short brainstorm (storyboard, the claims each caption rests on, the tune), then: shots checked
against their dumps, `claims.py` measured, visuals, beats, tune, a preview for review, and the
shareable cut in `~/Movies/Flump Studio/`.
