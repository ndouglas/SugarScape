# Spike: tags (Cooperation, episode 4)

**Date:** 2026-09-28
**Corrected after the review (2026-09-28):**

- **Tie rule:** the paper states it (p. 442: an agent "adopts the other's tag and tolerance if the
  other's score is higher than its own"); only its p. 441 wording reads as a coin flip. The docs had
  called it unstated.
- **Twins:** the paper reports them itself (p. 442): the twins beat now says "The paper saw it too", and
  the end card credits Edmonds & Hales, whose result "It was twins" is.
- **Takeovers:** one definition for both halves of the cycle caption (the most common exact tag
  changing while half the Flumps hold it): every 343 generations (294–483), 95.2 % of 565 steps within
  0.03 of the tag range, none over 0.075. The engine's 0.01-radius count had made the paper's cycle look
  "much slower than it describes"; it isn't. The caption is now "every few hundred generations, a new
  crowd takes over".
- **Narrow:** measured directly: 94.3 % of gifts go to an exact twin (89.7–96.2 %).
- **Clones:** compared with the same coin-flip ties (73.7 %), above it in 20 of 20.
**Status:** built as `studio/episodes/tags` (claims in its `measurements.md`; 74 s). Two changes
after the first cut:

- **The ring has a gap at the top.** Tags don't wrap around in the model, so 0 and 1 are as far
  apart as two shades can be; a closed ring put them side by side.
- **Takeovers only ever step next door.** The first cut showed the crowd creeping along the ring
  rather than a new shade rising elsewhere. Measured over seeds 1–20 (generations 100–6,000): each
  time the most common exact tag changes while holding 50+ Flumps, it moves a median 0.007 of the
  tag range, 98.5 % of 334 changes under 0.05, never more than 0.075. Plausibly a new cluster needs
  the old one's gifts to get started, so it must begin within the old one's tolerance; that
  mechanism is a guess, not measured. The caption became "a new shade takes
  over: always the one next door".
**Question:** which of Riolo, Cohen & Axelrod's claims hold over 20 seeds, and how do we film a
model with no space at all?
**Method:** a throwaway probe (not kept): CLI runs of the presets over seeds 1–20, 30,000
generations each (the paper's length).

## The model in one paragraph

A hundred Flumps each carry a tag (a number from 0 to 1: their shade) and a tolerance. Each
generation, each Flump meets three others at random and helps any whose tag is within its tolerance of
its own: helping costs the helper 0.1 and gives the other 1. Then each Flump faces a random other,
and the higher score has the offspring that takes its place. The offspring's tag is redrawn at random
10 % of the time, and its tolerance nudged 10 % of the time. When scores tie, the paper doesn't say
who wins; only "the current agent wins" matches its tables (Edmonds & Hales 2003).

## How to film it: the ring of shades

There is no land, so the tag becomes the place. The board is a ring of felt, and a Flump stands at
the point of the ring its tag names (tag 0 at the top, going round), in the yarn of that shade (a hue
wheel). Flumps with the same or nearly the same tag crowd together at one spot, packed outward from
the ring, so a cluster is a visible crowd.

The model has no lasting individuals: each generation is 100 offspring. The film keeps 100 Flumps,
one per place in the population's list. When a Flump's offspring copies another, the Flump leaps
across the ring to its role model's shade; when its tag mutates, it leaps to a random shade. A slowed
generation draws each gift as a yarn arc between giver and receiver.

The build adds a `tags` shot: each generation's agents (id, parent, tag, tolerance, gifts given and
received), and in slowed shots each gift. That's 100 agents a generation, a small dump even over
thousands of generations.

## Measured (seeds 1–20, 30,000 generations)

| Candidate claim | Measured | Verdict |
|---|---|---|
| Table 1: 73.6 % of meetings end in a gift (rca-published: ties to the current agent) | 73.7 % (72.3–74.0) | Reproduces, with the paper's p. 442 tie rule |
| …with the paper's literal coin-flip ties, at two pairings: 4.3 % | 42.0 % (34.5–48.3); ties to the current agent give 2.0 % | The tables need p. 442's rule; the coin-flip reading of p. 441 doesn't give them |
| Clusters: most Flumps share one shade | 85.3 % in the dominant cluster (within 0.02 of its modal tag); 97.1 % of that cluster hold the modal tag exactly | Holds: they're twins |
| Tolerance does the work | mean tolerance 0.018 (0.017–0.020): Flumps help almost no one but exact twins | The paper's mechanism is doing little |
| A new cluster rises and takes over, again and again | 29 takeovers a run (19–37): one every 1,030 generations (810–1,580) | Holds: "every thousand generations or so" |
| Stop twins from having to help each other (donate only when the tag difference is strictly below tolerance) | 1.4 % (1.3–1.9) | Cooperation collapses (Edmonds & Hales 2003 report none) |
| Take tolerance away entirely (fixed at 0: help exact twins only; coin-flip ties) | 75.3 % (75.0–75.5) | More giving than with tolerance |
| Give every child a tiny random tag nudge, so no two are exact twins | 1.4 % (1.3–2.1) | Cooperation collapses |

**For the finale:** the tie rule the tables need; the tolerance the paper credits does none of the
work.

## Storyboard (proposed)

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | ring | "No land this time: 100 Flumps, each with a shade, and a tolerance." | the ring, 100 Flumps spread round it |
| 2 | help | "A Flump meets three others, and helps any whose shade is within its tolerance of its own." | one generation slowed; gift arcs |
| 3 | pay | "Helping costs 0.1. Being helped earns 1." | the same; scores |
| 4 | copy | "Then each faces a random other, and the winner's child takes the loser's place." | Flumps leap across the ring to their role models |
| 5 | works | "It works: about 74% of meetings end in a gift, as the paper says." | faster; a gift gauge |
| 6 | twins | "But look where they stand: most Flumps share one exact shade." | a crowd at one spot; a panel of the cluster's share |
| 7 | narrow | "Their tolerance is tiny. They help almost no one but their twins." | close on the crowd; tolerance drawn as a sliver of arc |
| 8 | cycle | "Every thousand generations or so, a new shade rises and takes over." | fast; the crowd migrates round the ring |
| 9 | strict | "Stop twins from having to help each other, and giving collapses to 1.4%." | rca-strict; the gauge falls |
| 10 | clones | "Take tolerance away entirely, and they give even more: 75%." | eh-clones-only; bars against the published 74 % |
| 11 | point | "It wasn't tolerance.\nIt was twins." | title |
| 12 | end | "Tags — after Riolo, Cohen & Axelrod, 2001\nndouglas.github.io/SugarScape" | title card |

About 75 s. The decision rules are the verdicts above, fixed before the episode's `claims.py`
re-measures them on each shot.

## Tune (proposed): *The Twins' Slip Jig*

An original slip jig in 9/8, played in unison by tin whistle and fiddle, the twins. Unison is the
point: the same notes, doubled exactly. Each takeover modulates the jig up a step, a new shade, with
a new pair of twin instruments taking it (whistle and fiddle, then two mandolins, then accordion and
concertina).

- Beats 1–4: the whistle alone, over a bodhrán.
- Beats 5–8: the twins in unison, modulating at each takeover.
- "strict": the twins split. The fiddle plays a beat late and a step off, the unison fails, and the
  tune thins to the bodhrán alone.
- "clones": the unison returns, louder.
- The title: the jig once more, ending on a doubled D.

The build needs one small change to the music code: the tunes so far have been written in quarter-note
meters, and a slip jig's 9/8 needs its own meter line.
