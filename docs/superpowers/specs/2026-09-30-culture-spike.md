# Spike: Axelrod's culture (Following the Crowd, episode 4, "One culture or many")

**Date:** 2026-09-30
**Status:** approved and built as `studio/episodes/culture` (all fourteen captions hold; 84 s). The "ours" beat was dropped; beat 8 split in two ("a map of 50 by 50: a handful", measured).
**Sources:** Axelrod 1997 ("The Dissemination of Culture", *J. Conflict Resolution* 41; a scan),
his archived demo program CULTURE.P, and the docking paper (Axtell, Axelrod, Epstein & Cohen 1996),
all in `papers/culture/`; milestone 14 (`culture`) and its 2026-09-30 corrections
(`2026-09-25-culture-design.md`, reviewed independently). Every caption's number will be a claim in
the episode's `measurements.md` (20 seeds, rules fixed first, 1,000 where a claim is about his
samples); the numbers below are the survey's (`culture.*`, 27 of 27 holding).

## What the episode can say

| Axelrod (and followers) | Here | Survey |
|---|---|---|
| 100 villages, five features of ten traits; neighbors interact with probability equal to what they share, and one copies a feature | the rule, as his paper and his program give it | — |
| Local convergence: neighbors grow alike, until each pair is identical or shares nothing, and change stops | every run stops | holds |
| The sample setup ends with a few regions (mean 3.2 of 10 runs; median 3 of 100) | mean 4.3, median 4 over 1,000 seeds, within sampling of his runs | holds |
| More features in common: fewer regions; more traits to differ on: more (Table 2) | 1 region at 10–15 features; 23 at 15 traits | holds |
| Talking farther (8 or 12 neighbors) leaves fewer regions | 4.1, 2.1, 1.4 (his 3.4, 2.5, 1.5) | holds |
| The surprise: bigger maps end with fewer regions (a peak near 12 × 12) | 21 at 12 × 12, 4.9 at 50 × 50, 2.3 at 100 × 100 | holds |
| Castellano et al. 2000: with enough traits, big maps shatter instead | 207 regions at 20 × 20, 424 at 30 × 30 (25 traits) | holds |
| Klemm et al. 2003: a little drift melts the borders | 17 regions without drift, 5 with 1 in 10,000 | holds |
| AAEC 1996: let them wander a sugar mountain and one culture takes everyone | 1.15 cultures (their 1.1), on a broad or tall enough mountain | holds |

Everything reproduces.

## How to film it

A Flump on every site of the felt (the lattice from the Cooperation series), still: villages don't
move. Each culture that survives to the end of a shot wears its own yarn color, from the first frame
it appears; cultures that die out are cream, so the survivors grow out of a cream field. Axelrod's
Fig. 1 lanes run between neighbors, darker the less they share (a new `lanes` overlay), so borders
read even where two survivors share a color. The mobility beat films the Sugarscape itself (its
existing shot), agents colored by culture. New in the engine: a `culture` shot (built, tested).

## Storyboard (about 90 s)

| # | Beat | Caption | Shot |
|---|---|---|---|
| 1 | villages | "Robert Axelrod's villages: a hundred on a map.\nEach has five features, each one of ten traits." | 10 × 10, the start |
| 2 | rule | "Neighbors talk as often as they're alike.\nWhen they talk, one copies a feature from the other." | a few events, slowed |
| 3 | converge | "Neighbors grow alike, until each pair is\neither the same or shares nothing. Then nothing changes." | the sample run to stability, lanes |
| 4 | regions | "A few cultures survive, about four here;\nAxelrod's runs, about three." | the end |
| 5 | traits | "More traits to differ on, and more survive." | 15 traits, about 23 |
| 6 | features | "More features to share, and one culture wins." | 15 features |
| 7 | range | "Talk to more neighbors, and fewer survive." | 12 neighbors |
| 8 | territory | "The surprise: a bigger map ends with fewer cultures." | 12 × 12 then 50 × 50 |
| 9 | shatter | "Unless there are enough traits.\nThen a big map shatters." | 30 × 30, 25 traits |
| 10 | drift | "Let a trait change at random, now and then,\nand the borders melt." | 12 × 12 with drift |
| 11 | wander | "Let them wander a sugar mountain,\nand one culture takes everyone." | the Sugarscape, docked |
| 12 | reproduces | "Everything Axelrod reported, the Flumps reproduce." | the sample run at rest |
| 13 | point | "Neighbors grow alike, and the map stays divided." (title) | the sample run at rest, wide |
| 14 | end | "One culture or many — after Axelrod, 1997" | end card |

## Tune (proposed): *Round of Villages*

A round (a canon) in F for recorders, each voice a village: they enter one by one on the same
tune at different pitches; "converge": the voices drift into unison in groups; "regions": three
or four groups hold different phrases, never merging; "traits": more, shorter phrases; "shatter":
the round in fragments; "drift": a wandering fiddle nudges the groups until they join; "wander":
everyone in unison; the title: the round again, ending on three chords held apart.
