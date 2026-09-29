# Design: Schelling's own models (milestone 29)

**Date:** 2026-09-29
**Status:** proposed.
**Why:** the playground's `schelling` model is Epstein & Axtell's 1996 variant. Schelling's 1969
and 1971 papers are now in `papers/schelling/`, and the user wants his model treated as his: his
rules as the default, later researchers' versions as named variations. The rules, what he left
open, and his claims are in `2026-09-29-schelling-reading-notes.md`.

## Three pieces

### 1. The checkerboard: `schelling` generalized, Schelling's rules the default

New config, each field a named switch; the defaults are Schelling 1971's, and every `vi-*`
preset sets Epstein & Axtell's values explicitly (their golden fingerprints must not move).

| Field | Schelling (default) | Epstein & Axtell | Other values |
|---|---|---|---|
| `width` × `height` | 16 × 13 | 50 × 50 | any |
| `population`, `red_share`, `exact` | 138, ½, exact counts (69 + 69, 70 blank) | 2,000, ½, a coin per agent | minorities (Fig. 13's 2:1, Fig. 14's 5:1) |
| `neighborhood`, `radius` | Moore (8 around), 1 | von Neumann (4), 1 | radius 2: the 24 squares of p. 154 |
| `edges` | bounded ("a square has only five neighboring squares, and in a corner but three") | torus | |
| neighbors counted | occupied squares only (his figures) | the same | |
| demand | "no fewer than half" alike | a share (25 %, 50 %, 25–50 %) | per-color **demand tables**: for each number of neighbors, the least and most alike wanted (his "eight denominators … eight numerators", p. 155) |
| `movement` | nearest satisfying vacancy, by squares traversed horizontally and vertically | a random satisfying vacancy | |
| `order` | rounds: the discontented at the round's start, in reading order from the top left (Fig. 8); anyone content when their turn comes stays; the newly discontented wait for the next round | random order every turn | `center_out` (Fig. 9) |
| ties (nearest) | **unstated**: random | — | |
| nowhere acceptable | **unstated** ("it usually turns out that he can"): stay | stay | |
| no neighbors | **unstated**: content (his "less than half" reading) | content | |
| residence and newcomers | off | VI-5 to VI-7 | plus the planned `colorblind` and `newcomers` switches |

Demand tables express every figure: Fig. 11's "one like neighbor out of four or fewer, two out of
five or more"; Fig. 12's unequal demands; Fig. 13's "two neighbors of like color"; Fig. 16's
congregationists ("three … out of eight … indifferent to the presence of the opposite color");
Fig. 17's integrationist bands ("at least three and at most six like oneself …"). A share is the
table it implies. Fig. 17's ranked preference (a first, second and third choice) is a utility rule,
left to the variations milestone.

The nearest search is a breadth-first walk in Manhattan distance (wrapping on the torus); the
random search keeps the existing class pools, so Epstein & Axtell's runs draw the same numbers.

New series: like share by color, the share with no opposite neighbor, like-to-unlike ratio,
moves, rounds to equilibrium; and for his density claims, each color's mean number of neighbors.

### 2. The line: a new model kind, `line`

"Linear Distribution" (1971 pp. 149–154): a row of stars and zeros with no gaps, a coin or exact
counts for each; each counts "the four nearest neighbors on either side" (radius a switch; ends
truncated to 4–7) and wants at least half alike; the discontented, a round at a time and "counting
from left to right", each move to "the nearest point that meets his minimum demand … 'Nearest'
means the point reached by passing the smallest number of neighbors", inserting themselves between
two others. Unstated, and switches: left–right ties (random), nowhere acceptable (stay). His
variation "restricted movement": a `reach` (people passed) with a fallback demand ("the nearest
place where three out of eight occur"). Series: groups (runs of one color), mean group size, like
share, share with no opposite neighbor, discontented, moves. The page draws the row wrapped onto
lines like a typewriter's, as he printed it.

### 3. The bounded neighborhood (for episode 2), designed after episode 1

A small model of one area and a pool outside, with tolerance schedules (pp. 167–181). Its design
waits for its own spike.

## Beyond the engine

Presets for each figure and claim (`s71-*`), preset titles, survey claims (the notes' §8, decision
rules fixed before running), sweeps (demand, vacancy, radius, order), the page (both models, the
new switches in the Rules panel), goldens, and `docs/papers.md` (milestone 29). The studio adds
`schelling` and `line` shots.

## Out of scope here

Pancs & Vriend's utility movers, Zhang's swaps and rents, Gauvin et al.'s phase diagram and Singh et
al.'s scaling belong to the variations milestone (episode 3). Bruch & Mare waits for van de Rijt,
Siegel & Macy (2009).
