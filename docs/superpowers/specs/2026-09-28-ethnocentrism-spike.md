# Spike: ethnocentrism (Cooperation, episode 3)

**Date:** 2026-09-28
**Corrected after the review (2026-09-28):** the "blind" caption first set our 42 % against the
paper's 14 % alone. The paper's paired figure for Flumps that see color (56 %) doesn't reproduce either
(here 67 %), and cooperation at double cost is steep in the cost, so the caption now shows both pairs
and blames no one. A new "usual" beat measures what the title rests on: at the standard cost,
color-blind Flumps help more (81.5 % to 75.6 %, in 18 of 20 seeds). "Family" became "share
ancestors", "favor" ends at a period near the median, and the end card credits Jansson.
**Status:** built as `studio/episodes/ethno` (claims in its `measurements.md`: all eight hold as
proposed). The scattering is filmed as the standard case switched to offspring-anywhere at period
1,000, and measured that way. The land churns through about 250,000 Flumps a shot, so the board
gets a Flump per square, not per Flump (71 s).
**Question:** which of Hammond & Axelrod's claims hold over 20 seeds, and how can the Flumps show
two traits at once, a color and a strategy?
**Method:** a throwaway probe (not kept): CLI runs of the presets over seeds 1–20, 2,000 periods
each (the paper's length), averaging periods 1,901–2,000 as the paper does.

## The model in one paragraph

A 50 × 50 board that wraps around starts empty. Each period one newcomer arrives on a random empty
square, with random traits: one of four colors, and two yes-or-no rules, "help my own color" and
"help other colors". That makes four kinds: ethnocentric (help only your own), humanitarian (help
everyone), selfish (help no one) and traitorous (help only others). Every Flump's chance to have a
child starts at 12 %. Each decides, for each of its four neighbors, whether to help; helping costs
the helper 1 point and gives the neighbor 3. Then each has a child with that chance, on an empty
square next door (the child inherits color and rules, each mutating 0.5 % of the time), and each
dies with a 10 % chance.

## Can we film it?

Yes. The board holds about 1,560 Flumps (1,520–1,600), between the demographic PD's 900 and the
spatial games' 9,801. Flumps never move, so the build adds an `ethno` shot whose frames list each
Flump (id, square, color, kind, founding immigrant). Births and deaths are the differences between
consecutive frames, so the engine needs no new records.

Two traits, two channels:

- **Color:** the Flump's yarn (coral, teal, lilac, blue).
- **Kind:** the felt square under it, colored through the spatial board's per-square image.
  Ethnocentric is amber, humanitarian cream, selfish charcoal, traitorous magenta, and an empty
  square plain felt.

A board of ethnocentrics glows amber between the Flumps; one where nobody helps goes dark.

## Measured (seeds 1–20, periods 1,901–2,000)

| Candidate claim | Measured | Verdict |
|---|---|---|
| Ethnocentrism dominates (Table 1 a: 76.3 % ethnocentric, 74.2 % cooperation) | 76.7 % (66.2–80.3) ethnocentric, the largest kind in 20 of 20; 75.6 % cooperation | Reproduces |
| …because help goes to kin | 86.4 % (81.9–89.9) of helps go to relatives (a common founding immigrant); 75 % of neighboring pairs are relatives | Holds: the episode's "why" |
| Figure 1: mutation 0.25 %, 82.8 / 79.8 | 84.3 / 80.4 | Reproduces |
| From a full land of egoists, "just as dominant" | 78.6 % ethnocentric | Reproduces |
| Offspring placed anywhere (Jansson 2013): like the null model | cooperation 4.6 % (3.9–5.6); 88.8 % selfish; only 15 % of helps reach relatives | Reproduces: the headline |
| Color-blind, helping at double cost: "cooperation falls to 14 percent" | 41.6 % (31.7–55.3) | Doesn't reproduce, nor does the paired seeing figure (56 %; here 67 %) |
| The appendix's 5 % mutation | 35.7 % ethnocentric | A slip (the text, tables and code use 0.5 %) |

**For the finale:** color-blind cooperation at double cost; the appendix's mutation rate.

## Storyboard (proposed)

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | colors | "Each Flump wears one of four colors, and passes it on to its children." | close-up, a small board |
| 2 | rules | "Each carries two rules: help my own color? Help the others?" | close-up; a legend of the four kinds, the felt lighting up under each |
| 3 | kinds | "Help only your own, help everyone, help no one, or help only others." | the same; the legend |
| 4 | cost | "Helping a neighbor costs a little of your chance to have a child,\nand gives the neighbor three times as much." | close-up, one period |
| 5 | arrive | "The land starts empty. Newcomers arrive one at a time; children are born next door." | wide, periods 0–150 |
| 6 | favor | "Favoritism wins: about 3 in 4 Flumps help only their own color, in all 20 worlds." | wide, to period 2,000; a kinds tally |
| 7 | family | "Why? Their neighbors are family: over 8 in 10 of all helps go to relatives." | the same, closer; bars |
| 8 | anywhere | "Now put each child anywhere on the land." | jansson-offspring-anywhere |
| 9 | dark | "Favoritism collapses. Nine in ten Flumps help no one." | the same, fast; the land goes dark |
| 10 | blind | "The paper says color-blind Flumps, paying double to help, help 14% of the time.\nHere: 42%." | ha-cost-2-blind; bars |
| 11 | point | "Favoritism didn't make them helpful.\nFamily did." | title |
| 12 | end | "Ethnocentrism — after Hammond & Axelrod, 2006\nndouglas.github.io/SugarScape" | title card |

About 75 s. The decision rules are the verdicts above, fixed before the episode's `claims.py`
re-measures them on each shot.

## Tune (proposed): *Cradle Song for Four Colors*

An original lullaby in F major, 6/8, rocking like a cradle. There are four instruments, one per
color: music box, harp, clarinet and cello. Each plays its own variation of the tune and answers only
itself, as each color helps only its own.

- Beats 1–4: the music box alone, the tune plain.
- Beats 5–7, as families grow: the other three join, each echoing its own phrases, in close harmony.
- The scattering (beats 8–9): the phrases go to the wrong instruments in the wrong registers,
  fragments with rests between them, the harmony thinning to bare fifths, until the music box plays
  on alone, slowing (longer and longer notes), like a box winding down.
- "blind": a single clarinet line, the tune without its answers.
- The title: all four, the lullaby whole, ending on a soft F major chord in the harp.
