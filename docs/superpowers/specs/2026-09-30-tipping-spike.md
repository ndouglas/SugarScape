# Spike: Schelling's bounded neighborhood (Following the Crowd, episode 2, "The tipping point")

**Date:** 2026-09-30
**Status:** approved and built as `studio/episodes/tipping` (claims in its `measurements.md`: all thirteen
hold, the survey's ten among them, run by the claims; 86 s). The paradox beat became two beats (the
least tolerant made less tolerant; every Red made less tolerant); the capped run starts at 10 and 10
so the area fills on screen.
**Sources:** Schelling 1971 (pp. 167–186) and 1969 (pp. 491–493), read page by page
(`2026-09-29-schelling-reading-notes.md` §4); the engine is milestone 31 (`tipping`), reviewed
independently. Every caption's number is the survey's (`tipping.*`); with his exact tolerance
schedules the model is deterministic, so each result is one run, and the survey sweeps many starts
where a claim is about where things end.

## What reproduces: everything he states

| Schelling | Here | Claim |
|---|---|---|
| Fig. 18: "only two stable equilibria", each all one color | every start on a grid ends one-colored | holds |
| Fig. 19: "a stable mixture at 80 blacks and 80 whites" | 80 and 80 | holds |
| … reached with "slightly over 40%" of both present | every start with 41+ of each (400 starts) | holds |
| … from one color, "concerted entry of more than 25%" | 28 needed, 26 too few | holds |
| Fig. 20: two to one, and the mixture "disappears" | every start ends one-colored | holds |
| Fig. 21: no stable mix below an intercept of 3.0 | none up to 2.9; some from 2.95 | holds |
| Fig. 22: whites limited to 40: "40 whites and a comparable number of blacks" | 40 and 40 | holds |
| 1969: the least tolerant 60 % intolerant: "forty apiece" | 40 and 40 | holds |
| "the minority must be the more tolerant" | as tolerant, pushed out; five times as tolerant, a mix | holds |
| Making the least tolerant two-thirds less tolerant keeps a mix; making all less tolerant doesn't | 33 and 42; all Red | holds |

His model reproduces completely, unlike his hand-worked boards (episode 1): the episode can say so.

## How to film it

A fenced patch of felt, the area, with its insiders standing in it; outside, a queue for each color,
most tolerant nearest the gate. A Flump who leaves walks out to the back of its queue's tolerance
order; one who enters walks in. Tolerance is shown by where one stands in its queue (and a small
number on the close-up). Schelling's own picture, the plane (Red inside across, Blue inside up, each
color's contented region tinted), rides alongside as an overlay, its path drawn as the area changes.
The build adds a `tipping` shot: each step's insiders by color and rank.

## Storyboard (about 80 s)

| # | Beat | Caption | Shot |
|---|---|---|---|
| 1 | area | "Schelling's second model: one neighborhood everyone prefers.\nYou're in it, or you're not." | the area and queues |
| 2 | limit | "Each Flump has a limit: how many of the other color,\nfor each of its own, it will live with." | close-up, a few limits shown |
| 3 | rule | "The least tolerant leave first.\nThe most tolerant come in first." | a few steps slowed |
| 4 | fig18 | "100 Red, 50 Blue, most able to live with some of the other color.\nStart mixed, and one color leaves entirely." | Fig. 18 from 25 and 25 |
| 5 | plane | "Schelling drew every mix both colors would stay in.\nIn this one, no mix can last." | the plane overlay |
| 6 | fig19 | "Make them more tolerant, and a mix holds:\n80 and 80, from any start with enough of each." | Fig. 19 from 50 and 50 |
| 7 | entry | "Start all Red, and it takes 28 Blue\narriving together to get in." | Fig. 19, 100 Red and 28 Blue |
| 8 | fig20 | "Make one color twice as many,\nand the mix is lost." | Fig. 20 |
| 9 | cap | "Let in only the 40 most tolerant Red,\nand it holds at 40 and 40." | Fig. 22 |
| 10 | paradox | "Make the least tolerant even less tolerant, and it holds too.\nMake every Red less tolerant, and it doesn't." | the paradox, two runs |
| 11 | reproduces | "Every result Schelling reported here, the Flumps reproduce." | the plane |
| 12 | point | "Whether a neighborhood stays mixed depends not only\non how tolerant people are, but on who, and how many." (title) | Fig. 19's mix |
| 13 | end | "The tipping point — after Schelling, 1969, 1971" | end card |

## Tune (proposed): *A Waltz That Tips*

A waltz in G for two partners, a muted trumpet (Red) and a flute (Blue), over strings. "area",
"limit", "rule": the waltz plain, both partners. "fig18": the flute drops out bar by bar until the
trumpet dances alone. "fig19": both hold the waltz together. "entry": the flute tries to come in and
drops out, until it enters with the others (a chord). "fig20": the flute is crowded out again.
"cap" and "paradox": both return. The title: the two partners in canon, ending together.
