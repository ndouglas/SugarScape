# Design: Schelling's bounded neighborhood (milestone 31)

**Date:** 2026-09-29
**Status:** built. Amendments from the build: presets are `tipping-*` (`s71-minority` was taken);
`order` is `alternate` (Red then Blue), `blue_first` or `simultaneous`; Fig. 21 starts at 55 and 45
(at exactly 50 and 50 nobody moves); Inspect reports `agent: null` for the page's shared code.
**Why:** episode 2 of Following the Crowd, "The tipping point", films Schelling's second model
(1971, pp. 167–186; 1969, pp. 491–493): one bounded area, everyone inside or outside, each person
with a tolerance for the other color. The rules, results and page references are in
`2026-09-29-schelling-reading-notes.md` §4.

## The model, as Schelling states it

- Two groups, "whites" and "blacks" in his words (the page and the video say Red and Blue; the
  docs quote him). Each person has a **tolerance**: the largest ratio of the other color to their
  own inside the area that they will accept. There is no lower limit. "Absolute numbers do not
  matter, only ratios."
- A group's tolerances form a **tolerance schedule**: the n-th most tolerant person's limit. His
  worked schedules are straight lines (Fig. 18: from 2.0 down to 0, median 1.0; Fig. 19: from 5.0,
  median 2.5), a three-tier step (Fig. 26), a rectangular hyperbola (Fig. 27), and truncations (the
  least tolerant 60 % made intolerant; the least tolerant replaced by less tolerant ones).
- **Moves:** "People in the area move out if the ratio is not within their color limit; people
  outside move in if they see that it meets their requirements." "the least tolerant leave first
  and the most tolerant enter first." Nobody anticipates. **Relative speeds are left open**: "we
  can watch and see how they matter".
- **Limits** (Figs. 22–24): at most so many of a color inside (the most tolerant being "the first
  to enter and the last to leave"), or at most so many in all.

## The engine: a new model kind, `tipping` ("Schelling's tipping")

Config (every open point a named switch):

| Field | Default | Other values |
|---|---|---|
| `red`, `blue` | 100, 50 (1971's Fig. 18: 100 whites, 50 blacks) | any |
| `red_schedule`, `blue_schedule` | a straight line from 2.0 to 0 | `line {intercept}`, `tiers {…}`, `hyperbola {k}`, and `intolerant_share` (the least tolerant share set to 0) |
| `draws` | `schedule`: the n-th person's tolerance is the schedule's value at their rank, exactly | `random`: drawn from the schedule's distribution (for runs that differ by seed) |
| `start` | a given number of each inside | `random`: each person inside with a chance |
| `speed_red`, `speed_blue` | 1 person a step each | any: "relative speeds … we can watch and see how they matter" |
| `order` | `alternate` (a Red move, then a Blue) | `simultaneous`, `red_first`, `blue_first` |
| `limit_red`, `limit_blue`, `limit_total` | none | Figs. 22–24 |

A step: for each color in `order`, up to its speed of moves. A move is the least tolerant insider
who is discontent leaving, or failing that the most tolerant outsider who would be content (judged
on the ratio inside now, as "they see") entering, if the limits allow. A still state is an
equilibrium; the series record reds and blues inside, each color's discontented, and whether it is
still.

The page draws Schelling's own picture: the plane of reds inside against blues inside, each
color's tolerance curve (his parabolas: n people tolerate n·R(n) of the other), the path so far,
and beside it the area itself, its insiders and a queue outside.

## What gets tested (the survey, 20 seeds where seeds matter)

From §4.2, each with its decision rule fixed before running: Fig. 18 has only the two one-color
stable states and any mixture empties of one color; Fig. 19's stable 80–80 mix, reached from over
40 % of both, and needing "concerted entry of more than 25%" to reach from one color; Fig. 20's
2:1 numbers destroying it; Fig. 21's threshold (a stable mix for straight lines with equal numbers
only above an intercept of 3.0); Fig. 22's limit of 40 whites making a stable mix; the least
tolerant 60 % made intolerant giving a stable 40–40 (1969); "the minority must be the more
tolerant"; replacing the least tolerant two-thirds by less tolerant people helping, where making
everyone less tolerant does not; and no discontinuity at the typical tolerance. Sweeps: the plane
of starting points and where each ends; the intercept; relative speeds; the limit on one color.

## Out of scope

His subneighborhood and speculation discussions (pp. 184–186), which he leaves as conjecture; many
areas at once, which he says the model lacks.
