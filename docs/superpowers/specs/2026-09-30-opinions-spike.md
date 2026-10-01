# Spike: bounded confidence (Following the Crowd, episode 5, "Listening to the like-minded")

**Date:** 2026-09-30
**Status:** proposed.
**Sources:** Hegselmann & Krause 2002 (*JASSS* 5(3) 2, in `papers/bounded-confidence/`), Lorenz 2006
(*JASSS* 9(1) 8, also there); milestone 17 (`opinions`), rechecked 2026-09-30: Fig. 3 rebuilt from 2,000
runs and set beside theirs, and ε = 0.15 cross-checked with a separate implementation. Every caption's
number will be a claim in the episode's `measurements.md` (20 seeds, 50 where the paper used 50,
rules fixed first); the numbers below are the survey's (`opinions.*`, 18 of 19 holding).

## What the episode can say

| Hegselmann & Krause (and Lorenz) | Here | Survey |
|---|---|---|
| Each agent moves to the mean of the opinions within its confidence ε (625 agents, all at once) | the rule | — |
| ε = 0.01: "exactly 38 different opinions survive" | median 37.5 | holds |
| ε = 0.25: consensus | 20 of 20 | holds |
| ε = 0.15: "two camps … a fairly typical result" | about a third of runs (16 of 50); most keep a middle camp of about 30 %, which their own Fig. 3 shows as a central ridge | fails |
| Plurality, then polarization, then consensus as ε grows; consensus takes over near 0.25 | 13, 30 and 50 of 50 in consensus at 0.21, 0.22, 0.25 | holds |
| Splits at the edges drive it: the outermost hear only one side and move inward (Fig. 4: 50 evenly spaced at 0.2 split in period 6) | exactly | holds |
| Asymmetric confidence: the crowd drifts toward the side it listens to | mean 0.53 → 0.94 | holds |
| Confidence leaning with one's opinion: camps pushed to 0 and 1 | final range ≥ 0.95 at m = 1 | holds |
| Only neighbors on a map: "polarization disappears" | 9 of 200 lattice runs polarized, 57 of 100 among everyone | holds |
| Lorenz 2006: consensus depends on how many there are | at ε = 0.22, more often with 1,000 than 50 | holds |

## How to film it

A felt ruler from 0 to 1 across the table; each Flump stands at its opinion, colored by where it
started (red at 0 to magenta at 1, as their figures are), spread in depth so a camp reads as a
crowd. Each period every Flump hops to its new place; camps pile up. A panel draws their own picture
(opinion against time, a line per agent, a sample of 120) as it goes. New: an `opinions` shot (each
agent's opinion per period), its loader and the ruler board.

## Storyboard (about 90 s)

| # | Beat | Caption | Shot |
|---|---|---|---|
| 1 | crowd | "625 Flumps, each with an opinion between 0 and 1." | ε 0.15, the start |
| 2 | rule | "Each listens only to those close enough,\nand moves to their average." | a few periods, slowed |
| 3 | few | "Listen to very few, and dozens of opinions survive." | ε 0.01 |
| 4 | many | "Listen widely, and everyone agrees." | ε 0.25 |
| 5 | middle | "In between, Hegselmann and Krause's run ends in two camps." | ε 0.15, a two-camp seed |
| 6 | three | "Here that happens about a third of the time.\nMore often, a third camp holds the middle." | ε 0.15, a typical seed |
| 7 | edges | "Camps form at the edges first: those at the ends\nhear only one side, and move in." | Fig. 4, 50 evenly spaced |
| 8 | lean | "Listen more to one side, and the whole crowd drifts there." | asymmetric |
| 9 | extremes | "Let each lean toward its own side,\nand the camps are pushed to the ends." | bias m = 1 |
| 10 | neighbors | "Hear only your neighbors on a map, and the camps dissolve:\none crowd, with stranded minorities." | lattice |
| 11 | size | "And numbers matter: a big crowd agrees\nwhere a small one splits." | ε 0.22, 1,000 against 50 |
| 12 | reproduces | "Nearly all they reported, the Flumps reproduce." | the panel |
| 13 | point | "Who you'll listen to decides whether a crowd agrees,\nsplits, or shatters." (title) | ε 0.15 at rest |
| 14 | end | "Listening to the like-minded — after Hegselmann & Krause, 2002" | end card |

## Tune (proposed): *Converging Lines*

A fugue-like piece in E minor for a string quartet: voices start scattered across the range (each
entering on a different pitch of the subject), then close in by steps; "few": many short motifs
that never meet; "many": all four voices converge to one held E; "middle" and "three": the voices
pair off into two (then three) registers; "lean": the whole texture rises; "extremes": the outer
voices pushed to their highest and lowest; "neighbors": a single unison line with stray notes;
the title: the subject once more, ending on an open fifth.
