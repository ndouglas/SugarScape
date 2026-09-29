# Spike: social structure (Cooperation, episode 7)

**Date:** 2026-09-28
**Status:** built as `studio/episodes/friends` (claims in its `measurements.md`: all ten hold; the
dial's 30 % caption was reworded when "it flickers" failed, 11 of 20). The repository wording below
is fixed.
**Question:** what do Cohen, Riolo & Axelrod (2001) claim, does it hold over 20 seeds, and how do we film
256 Flumps whose partners stay put, change, or partly change?
**Method:** an independent paper-versus-code review before the storyboard, and CLI runs of the presets
over seeds 1–20, 2,500 periods each (the paper's length). Throwaway probes, not kept.

## The model in one paragraph

256 Flumps each have three habits: how likely to help at the start of a game (y), after being helped
(p), and after being cheated (q). Each period, each plays four-move Prisoner's Dilemmas with four
partners (3 each for mutual help, 1 each for mutual cheating, 5 and 0 when one cheats the other).
Then each copies its best-scoring partner if that partner did strictly better, misjudging 10 % of the
time, with small random changes to each habit 10 % of the time. What varies is who the partners
are: fresh strangers every period (RWR), the four neighbors on a torus (2DK), fixed random partners
(FRN, or FRNE with both sides of each tie), or fixed partners each swapped for a stranger with some
chance each period (FFR-x).

## What the paper claims

- **The point:** "Context-preserving social structure suffices … without paired agents having
  correlated networks" (p. 12). Keeping the same partners sustains cooperation; geography isn't
  needed. They are careful: "We do not contend that clustering could never contribute" (p. 12).
- **Table 2:** mean payoff per move over the last 1,000 of 2,500 periods, whether high cooperation is
  ever attained, and the share of time it remains high once attained. RWR 1.091; 2DK 2.557; FRNE
  2.575; FRN 2.480; FFR-0.1 2.385; FFR-0.3 2.100; FFR-0.5 1.257.
- **The dial:** "Around a parameter value of 0.3 the dynamics shift … at levels of 0.5 and above, it
  collapses" (p. 13); their typical 0.3 history is "bi-stable" (p. 21).
- **FRNE and 2DK:** "nearly indistinguishable" (p. 11); FRNE is statistically higher (note 5), but "the
  magnitude of the difference is not important for our argument".
- **"High cooperation"** is not formally defined, but p. 20's "9 of our 30 histories reached an
  average score over 2.3" (Table 2's 0.30) implies 2.3, and fitting the Remain High column agrees.

## The review, before the storyboard

- **The engine follows the stated rules** (partners, games, payoffs, copying with misjudgment, noise,
  the structures, FFR's substitution, 2,500 periods, the last-1,000 mean). The paper's two
  descriptions of its own start and of when noise applies are named switches.
- **Repository wording to fix:** "the unstated threshold is 2.3" (p. 20 implies it); "only the
  Appendix's noise rule keeps FRNE above 2DK" (the other reading narrows the gap, it doesn't erase it,
  at 40 seeds); "bi-stable, as stated" (our 50-period test is ours, and lenient).
- **Caption risks:** don't headline FRNE over 2DK or "clustering doesn't matter"; "remain high" counts
  only runs that reached it; the "attain" column is too noisy at 20 seeds to caption.

## Measured (seeds 1–20)

Mean payoff over periods 1–2,500 in the spike (the claims will use the paper's last 1,000):

| Structure | Here | Paper | Remain high (here, once reached) | Paper |
|---|---|---|---|---|
| Strangers every period (RWR) | 1.098 | 1.091 | 1.2 % | 1.5 % |
| Torus neighbors (2DK) | 2.546 | 2.557 | 99.8 % | 99.7 % |
| Fixed random neighbors, both ways (FRNE) | 2.567 | 2.575 | 99.6 % | 99.5 % |
| Fixed random partners (FRN) | 2.454 | 2.480 | 93.6 % | 94.2 % |
| FRN, a tenth swapped each period | 2.372 | 2.385 | 84.2 % | 84.4 % |
| … 30 % swapped | 1.977 | 2.100 | 37.0 % | 40.2 % |
| … half swapped | 1.325 | 1.257 | 7.7 % | 6.1 % |

**Verdict:** it reproduces, closely: every row of Table 2 comes back, in order and within a few
hundredths (FFR-0.3 is the noisiest: a run-to-run s.d. of about 0.2).

## How to film it: 256 Flumps and their ties

The board is a 16 × 16 grid of Flumps, colored by how readily each helps after being helped (p):
green for friendly, red for wary. For a few highlighted Flumps, yarn lines show their four partners
this period: on the torus, the four next door; for fixed random partners, lines across the board that
never change; for strangers, lines that jump every period; on the dial, some lines jumping and some
holding. A gauge shows the mean payoff. The build adds a `structure` shot: each period's habits and
payoffs, and on request each Flump's partners.

## Storyboard (proposed)

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | habits | "256 Flumps, each with three habits: how often it helps first,\nafter being helped, and after being cheated." | the grid, colored by p |
| 2 | games | "Each period, each plays four rounds with four partners." | partner lines for a few Flumps |
| 3 | copy | "Then each copies its best-scoring partner, if it did better." | a period slowed |
| 4 | strangers | "Meet strangers every period, and cooperation almost never takes hold." | RWR; lines jumping; payoff gauge |
| 5 | neighbors | "Keep the same four neighbors, and it takes hold in every world, and stays." | 2DK |
| 6 | friends | "Keep the same partners, scattered anywhere: nearly as good.\nNo geography needed." | FRN; long fixed lines |
| 7 | dial | "Swap a tenth of them each period, and it mostly holds." / "Swap 30%, and it holds only about a third of the time." / "Half, and it collapses." | FFR-0.1, 0.3, 0.5, one beat each. ("It flickers" failed when measured: a long high and a long low stretch in only 11 of 20 runs.) |
| 8 | paper | "That's what Cohen, Riolo and Axelrod found in 2001,\nto within a few hundredths." | bars: paper vs here, each structure |
| 9 | point | "Cooperation doesn't need neighbors.\nIt needs the same faces." | title |
| 10 | end | "Friends and strangers — after Cohen, Riolo & Axelrod, 2001\nndouglas.github.io/SugarScape" | title card |

About 70 s. Each caption's numbers come from the episode's `claims.py` over 20 seeds, with the
paper's own measures (mean payoff over the last 1,000 periods; remain high once reached).

## Tune (proposed): *Swing Your Partner*

An original reel in G, as for a square dance, where who you dance with is the whole game: fiddle,
banjo, guitar, upright bass and spoons.

- Beats 1–3: fiddle and guitar, the tune plain.
- "strangers": the band loses the beat, each player a little off, the tune breaking up.
- "neighbors" and "friends": the reel in time, full band.
- "dial": the reel holds at a tenth, drops in and out of time every two bars at 30 %, and falls apart at
  half.
- "paper": the reel steady, the fiddle leading.
- The title: the full band, ending on a "shave and a haircut" tag.
