# Following the Crowd: the third Flump series

**Date:** 2026-09-29
**Status:** in progress. Episode 1 (Neighbors like me, `2026-09-29-schelling-spike.md`) is built on
milestone 30 (Schelling's own models); episode 2 (The tipping point, `2026-09-30-tipping-spike.md`) on
milestone 31 (his bounded neighborhood); episode 3 (the variations) needs engine work.
**Follows:** the Sugarscape series (`2026-09-26-sugarscape-series-plan.md`) and the Cooperation series
(`2026-09-27-cooperation-series-plan.md`), both complete, and keeps their rules.

## The question

What happens when each Flump watches what the others do? Every model here gives its agents one
simple rule about other agents (move if too few neighbors are like you; become more like a
neighbor you already resemble; listen only to those who think nearly as you do; join once enough
others have; go where the crowd isn't), and asks what the crowd as a whole does: sort itself,
agree, split into camps, stampede, herd, or coordinate without a word.

As in the earlier series, each episode shows where a paper's stated rules don't give the paper's
result, and a finale collects them. Nothing here touches the War or Trade-and-Inequality studies,
which the user is doing separately.

## Rules (unchanged)

- Every Flump behavior comes from a real run: a frame dump of the model at the shot's own size.
- Every caption is measured over 20 seeds on the shot's config. If a claim fails, the caption
  changes, not the data. Decision rules are fixed before measuring.
- Before the storyboard, an independent paper-versus-code review; before any caption calls
  something a mistake, check that the paper doesn't state it elsewhere, and quote what it claims.
  An example run is not a claim about typical outcomes.
- Each episode gets a new original tune in ABC.
- American spelling.
- The process: spike → storyboard and music proposal → approval → build → preview (960 × 540).

## Episodes

| # | Episode | Source | Model | What the repository's records suggest (to be re-checked) |
|---|---|---|---|---|
| 1 | Neighbors like me ✅ | Schelling 1969, 1971: the line and the checkerboard | `schelling` (his rules as the default) | Mild demands, exaggerated separation; his tabletop counts are hand-worked examples, to be tested as typical |
| 2 | The tipping point ✅ | Schelling 1971: the bounded neighborhood | new, small | Only the all-one-color mixes are stable unless tolerance is wide; greater tolerance can hurt |
| 3 | Variations on Schelling ✅ | Epstein & Axtell 1996; Pancs & Vriend 2007; Zhang 2004; Gauvin, Vannimenus & Nadal 2009; Singh, Vainchtein & Weiss 2009 | `schelling` switches | Wanting a mixed street still sorts the town; a frozen, a segregated and a mixed phase; clusters shrink on a big board |
| 4 | One culture or many ✅ | Axelrod 1997 | `culture` | Local convergence, global polarization; the docked mobility result reproduces on a broad or tall enough mountain |
| 5 | Listening to the like-minded | Hegselmann & Krause 2002 | `opinions` | Fig. 2b's two camps come up in about a third of runs; most keep a middle camp |
| 6 | How extremists win | Deffuant et al. 2000, 2002 | `agreement` | A few confident extremists and a moderate crowd |
| 7 | The riot that needs one person | Granovetter 1978; Watts 2002 | `thresholds` | One threshold decides the riot; cascades on networks |
| 8 | Ants at two food piles | Kirman 1993; Alfarano & Milaković | `ants` | The ants never rest at 80–20 |
| 9 | Nobody goes, it's too crowded | Arthur 1994; Challet & Zhang 1997 | `farol` | Coordination without communication |
| 10 | When to retire | Axtell & Epstein 1999 | `retirement` | A norm spreads through networks; footnote 5 is false |
| 11 | Finale | all | — | The ledger |

Episodes 1–3 are Schelling's: his own rules first, as the default, then later researchers' versions as named variations (see `2026-09-29-schelling-reading-notes.md`).

The order builds from space (1–4) through opinions (5–6) and joining (7–8) to coordination (9–10).

## Studio work

Each episode adds its model to the frame dump (`frames.rs`), as the Cooperation series did. Boards
the studio already has may serve: the lattice (Schelling, culture), the ring or street (opinions on
a line), the plane (two-trait models), the grid (networks).
