# Spike: Schelling's own models (Following the Crowd, episode 1, "Neighbors like me")

**Date:** 2026-09-29 (rewritten after milestone 30; the first version filmed Epstein & Axtell's variant)
**Status:** approved and built as `studio/episodes/neighbors` (claims in its `measurements.md`: all ten
hold; 79 s). Changes in the build: beat 1 says Schelling "worked it out by hand" (his p. 155–156:
"done by hand and eye"; the pennies and nickels were his invitation to the reader, the dimes Epstein
& Axtell's phrase); beat 8 compares his Figs. 8 and 9 with the 20-seed median in a bar panel, rather
than redrawing his board; the line is filmed along its length at an angle.
**Sources:** Schelling 1969 (AER) and 1971 (J. Math. Sociol. 1: 143–186), read page by page
(`2026-09-29-schelling-reading-notes.md`); the engine is milestone 30 (`schelling` with his rules as
the default, and `line`), reviewed independently; every number below is the survey's (`s71.*`,
20 seeds) or a sweep's, and each caption's decision rule is the survey claim named beside it.

## What reproduces, and what doesn't (20 seeds)

| Schelling | Here | Claim |
|---|---|---|
| The line: "about five groupings … to seven or eight", groups of 9–14 | 7 groups of 10 | `s71-line.groups` holds |
| The line, 3 each side: "7 or 8 per cluster", 75–80 % own | groups of 8.75, 79 % | fails narrowly |
| A halved minority "tends to become more segregated" | 0.77 alike against 0.78 | fails |
| Restricted travel: "everybody achieves his desired neighborhood" | 6 % unsatisfied | fails |
| The board, half alike: Fig. 9's four-fifths to five-sixths and 40 % unmixed | 0.80 and 38 % | fails narrowly |
| "upwards of four to one" | 3.6 | fails |
| A third: "slight", a ratio under 1.5 | 1.35 | holds |
| "the character of the outcome not very much" on the order of moves | 0.80, 0.80, 0.79 | holds |
| Congregationists: "just over 75%" alike | 0.79 | holds |
| Integrationists: many more moves, some unsatisfiable | 96 against 46 moves; 9 % | holds |
| Unequal demands: more demanding, "not much" more alike, denser | 0.67 each; 5.6 against 4.8 neighbors | holds |
| A wider neighborhood "attenuates … for moderate demands" | at a third 0.55 against 0.66; at half 0.86 against 0.80 | holds (and reverses at half, as he hedged) |

The fair summary: his direction holds on the board; his hand-worked boards, which he called too
few "to allow serious generalizations", sit at the sorted end of what his rules give.

## Storyboard (about 80 s)

| # | Beat | Caption | Shot |
|---|---|---|---|
| 1 | pennies | "In 1969, Thomas Schelling sorted pennies and dimes\nto ask how neighborhoods divide." (title) | the line, still |
| 2 | line | "70 Flumps in a row. Each wants at least half\nof its eight nearest neighbors like itself." | `s71-line`, round 0 |
| 3 | squeeze | "The unhappy, from left to right, squeeze in\nat the nearest spot that suits them." | one round, slowed |
| 4 | clusters | "A few rounds later: about seven clusters of ten.\nNobody asked for more than half." | to rest |
| 5 | board | "Then a checkerboard: 138 Flumps and 70 empty squares." | `s71-board`, round 0 |
| 6 | nearest | "Each wants no fewer than half its neighbors alike,\nand moves to the nearest square that suits." | round 1, slowed |
| 7 | sorted | "Four in five neighbors end up alike.\nNearly two in five Flumps see no one of the other color." | to rest |
| 8 | hand | "Schelling worked his boards by hand, and said they were\ntoo few to generalize. His came out a little more sorted." | his Fig. 8 beside ours (a still) |
| 9 | third | "Ask for only a third, and the sorting is slight." | `s71-third` |
| 10 | company | "Ask only for company — three of your own —\nand the town sorts anyway." | `s71-congregate` |
| 11 | mixed | "Even Flumps who want a mixed street move more,\nand some are never satisfied." | `s71-integrate` |
| 12 | point | "Nobody wanted a divided town.\nThey just didn't want to be outnumbered." (title) | the sorted board |
| 13 | end | "Neighbors like me — after Schelling, 1969, 1971" | end card |

The line's minority and restricted-travel misses, and the demand curve's shape, go to the finale's
ledger. Episode 2 is his tipping model; episode 3 the later variations (Epstein & Axtell among them).

## Studio work

A `schelling` shot on the existing squares board (a Flump per chip, Red and Blue yarn, empty squares
bare; a hop to the new square), and a `line` shot: the row laid out like a typed line, Flumps
shuffling aside as one squeezes in. The frame dump records each round's agents and, when slowed,
each move.

## Tune (proposed): *Hocket for Two Colors*

Oboe (Red) and clarinet (Blue) share one melody note by note, interleaved like the starting row;
over the rounds the notes gather into runs of each color, then into separate phrases in separate
registers ("sorted"). "third": the hocket barely changes. "company": runs again, though nobody
asked. "mixed": the two keep trading places, restless. The title: the hocket returns, both colors
in turn, ending in unison.
