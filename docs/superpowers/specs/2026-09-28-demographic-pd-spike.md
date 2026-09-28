# Spike: the demographic Prisoner's Dilemma (Cooperation, episode 2)

**Date:** 2026-09-28
**Corrected after the review (2026-09-28):** the "book" caption first said Epstein's count "needs rules
he never wrote down". His working paper prints the same Table 1 beside a rule he did write: one game
a turn. It now credits that rule. The "low" caption now says Run 4 keeps Run 2's 100-cycle lives, and
that Epstein allows extinction as one outcome; "half helpers" became "about half".
**Status:** built as `studio/episodes/living` (claims in its `measurements.md`: all nine hold as
proposed); "book" was lengthened to 10 s so the round's sections land on the story (72 s).
**Question:** which of Epstein's claims hold on the board we'd film, and what the studio needs to film
Flumps that move, clone themselves and die?
**Method:** a throwaway probe (not kept): CLI runs over seeds 1–20 of the presets, on Epstein's own
30 × 30 board.

## The model in one paragraph

A hundred Flumps, half helpers and half cheats, start at random on a 30 × 30 board that wraps
around. In turn, each steps to a random empty square next door and plays each of its four neighbors.
Two helpers earn 5 each; a cheat facing a helper earns 6 and the helper loses 6; two cheats lose 5
each. A Flump whose wealth reaches 11 clones itself onto an empty square next door, giving the clone
6; one whose wealth goes negative dies. Strategies never change: a helper's clone is a helper.

## Can we film it?

Yes, and cheaply: at most 900 Flumps, a tenth of the spatial games' board. The Sugarscape's crowd
already animates Flumps that step between squares, appear and die. The build adds a `dpd` shot to
the frame dump: each cycle's agents (id, square, wealth, age, strategy), deaths and births (with the
parent), loaded into the studio's existing `Dump`, with a strategy color mode (blue helpers, red
cheats) and a flat board without sugar.

## Measured (seeds 1–20)

| Candidate claim | Measured | Verdict |
|---|---|---|
| From 100 Flumps, the land fills, mostly with helpers (Table 1, dpd-run-1) | Cycle 500: 733 ± 18 helpers (710–778), 167 ± 18 cheats; the board full (899–900 of 900) in 20 of 20 | Holds: "about 730 to 170" |
| Epstein's Table 1: 779 ± 15 helpers, 121 ± 15 cheats | 733 against 779: 46 fewer helpers; no seed reaches 779 | **Fails** under the published rules |
| …with the working paper's own rule (one game a turn against a random neighbor, printed beside the same Table 1) and founders with no wealth | 784 ± 18 helpers (t 1.0 against Table 1) | Matches: only the founders' wealth is a detail no source gives this way |
| Five helpers to every cheat by cycle 50 | 4.5 (3.8–6.2) | Near: "about four and a half to one" if used |
| Cheats live on the edges of helper clusters | 96 % of cheats touch a helper, but 100 % would if the same Flumps were shuffled at random (the board is full) | Uninformative: dropped |
| Soup: pair at random instead, and cooperation dies (dpd-soup) | The last helper is gone by cycle 5–12 in 19 of 20; in the other, a lone helper outlives everyone | Holds: "in 19 of 20" |
| …then the cheats destroy each other | Cycle 500: nobody left in 7, one Flump in 13 (peak about 125 Flumps) | Holds: "one Flump is left, or none", 20 of 20 |
| R = 1: booms and busts, and cooperators "ultimately do better" (dpd-run-4) | Extinct by cycle 500 in 19 of 20 (cycles 93–500) | **Fails**: the worlds die out |

**For the finale:** Table 1 needs rules the paper never states; lowering the reward gives
extinction, not booms and busts.

## Storyboard (proposed)

Helpers blue, cheats red. The close-ups use a small board (8 × 8, a dozen Flumps) with wealth over
the followed Flumps.

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | walk | "Now the Flumps walk. Each turn, a Flump steps to an empty square and plays each neighbor." | close-up, one cycle slowed; lines to the neighbors |
| 2 | pay | "Two helpers earn 5 each. A cheat takes 6 from a helper.\nTwo cheats lose 5 each." | close-up; wealth over the followed Flumps |
| 3 | clone | "Reach 11, and a Flump splits in two. Go broke, and it's gone." | close-up; a clone appears beside its parent, a broke cheat vanishes |
| 4 | start | "Start with 100 Flumps: half helpers, half cheats." | 30 × 30, cycle 0 |
| 5 | fill | "The land fills up, mostly with helpers: about 730 to 170." | wide, cycles 0–500; a helpers and cheats counter |
| 6 | book | "Epstein counted 779 helpers. His stated rules give about 730; his count needs rules he never wrote down." | bars: Epstein, stated rules, unstated rules |
| 7 | soup | "Now let any Flump meet any other, anywhere." | the soup: Flumps leap across the board each cycle |
| 8 | gone | "The last helper is gone within 12 cycles, in 19 of 20 worlds." | the soup, slowed; counter |
| 9 | ruin | "Then the cheats ruin each other. By cycle 500, one Flump is left, or none." | the soup, fast |
| 10 | low | "Back on the land, make helping pay less. Epstein saw booms and busts; here, 19 of 20 worlds die out." | dpd-run-4 (R = 1), fast; counter |
| 11 | point | "Mixed with strangers, helpers vanish.\nAmong neighbors, they fill the world." | title |
| 12 | end | "Living neighbors — after Epstein, 1998\nndouglas.github.io/SugarScape" | title card |

About 75 s. The decision rules are the verdicts above, fixed before the episode's `claims.py`
re-measures them on each shot.

## Tune (proposed): *A Round for Neighbors*

An original round: every entering voice copies the tune, as every clone copies its parent. It's in
G major, 3/4, for a woodwind quartet (flute, clarinet, oboe, bassoon) over pizzicato strings.

- Beats 1–3: the tune alone, on the flute.
- Beats 4–6, as the land fills: the clarinet enters two bars behind, then the oboe, then the bassoon.
  The round is full by the end of "fill".
- The soup (beats 7–9): the voices lose their places and collide in G minor. One by one they drop
  out, until a lone muted trumpet plays the first phrase and stops.
- "low" (beat 10): the round tries again with two voices and thins out.
- The title: the full round once more, ending on a held G major chord.
