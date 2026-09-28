# Spike: image scoring (Cooperation, episode 5)

**Date:** 2026-09-28
**Corrected after the review (2026-09-28):** NS98 show one run and give no frequency; their text
says "if the value is 1 or more then defection has won", more likely with fewer meetings. The spike
first marked Fig. 1 "fails as a typical outcome", a claim they never made. The captions now say the
paper shows one run, and count outcomes with the paper's own split (k ≤ 0 cooperation): everyone ends
up helping in 7 of 20 worlds, nobody in 13.
**Status:** built as `studio/episodes/image` (claims in its `measurements.md`: all eight hold;
over generations 1,001–20,000 Fig. 3 comes out 89 / 41 / 16 %, close to the paper's 90 / 47 / 18; 61 s).
**Question:** which of Nowak & Sigmund's claims hold over 20 seeds, and how do we film a reputation?
**Method:** a throwaway probe (not kept): CLI runs of the presets over seeds 1–20.

## The model in one paragraph

A hundred Flumps each carry a public score, from −5 to +5, starting at 0. Each generation, 125
times, a random donor meets a random recipient: the donor helps (cost 0.1, benefit 1) if the
recipient's score is at least the donor's own threshold k, and helping raises the donor's score by one
while refusing lowers it. Thresholds run from k = −5 (help everyone) to k = +6 (help no one); k = 0,
"help anyone who hasn't been refusing", is the paper's discriminator. At the generation's end each
Flump's children are in proportion to its payoff.

## How to film it: a street of thresholds

There is no land, so the strategy becomes the place: twelve columns along a street, one per
threshold, from "help everyone" (k = −5) on the left to "help no one" (k = +6) on the right. Each
Flump stands in its threshold's column, and the felt under it shows its score, from red (−5) through
plain felt (0) to green (+5). Each generation's children take their places in their parents' columns,
so a winning strategy's column fills while others empty. A slowed generation draws each meeting as an
arc from donor to recipient, yarn for a gift and a red flash for a refusal.

The build adds an `image` shot: each generation's agents (id, threshold, score, payoff), and in
slowed shots each meeting (donor, recipient, helped or not).

## Measured (seeds 1–20)

| Candidate claim | Measured | Verdict |
|---|---|---|
| Fig. 1: "cooperation wins": in the paper's run the discriminators (k = 0) take over by generation 166 | k = 0 takes over in 5 of 20; some helping threshold (k ≤ 0) in 7; a threshold that helps no one (k > 0) in 13, by generation 1,000 | The paper gives no frequency, and warns defection can win: here k ≤ 0 wins 7 of 20 (the repository's 100-seed record: 40 of 100) |
| Fig. 2: with mutation, "endless cycles" of cooperation and collapse | 20,000 generations: cooperation collapses (from 90 %+ cooperative to 10 % or less) 2–8 times in every run, and recovers 1–8 times; cooperative 73 % of the time (43–88 %) | Reproduces |
| Fig. 3: the bigger the group, the less anyone helps (each interaction seen by ten others; 90 / 47 / 18 % at n = 20 / 50 / 100) | cooperative strategies over generations 1,001–6,000: 94 %, 39 % and 26 % (medians; each spans nearly 0–100 %) | The ordering holds; the numbers, over this shorter window, are noisy |
| The universal constant 0.7380294688… | to every printed digit (analytic; the repository's record) | Reproduces |
| "About two interactions per lifetime suffice" | cooperative strategies hold 18 % at m = 100; half needs m = 200 (the repository's record) | Fails |
| LH01 Fig. 2b: on islands, image scoring fades to 9 % | 44 % (the repository's 10-seed record) | Fails |

**For the finale:** the showcase run; "two interactions suffice"; LH01's islands.

## Storyboard (proposed)

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | score | "Now every Flump carries a reputation: a score from −5 to 5." | the street, generation 0; the felt shows each score |
| 2 | earn | "Helping raises your score. Refusing lowers it." | one generation slowed; gift arcs and refusals |
| 3 | rule | "Each Flump has a rule: help anyone whose score is at least k." | the columns, labeled from "help everyone" to "help no one" |
| 4 | pay | "Helping costs 0.1 and gives 1. The more a Flump earns, the more children it has." | a generation's end; columns reshuffle |
| 5 | paper | "The paper's run ends with everyone helping those who help." | a world where k = 0 takes over |
| 6 | here | "Here that happens in only 5 of 20 worlds.\nIn 13, nobody ends up helping anyone." | a world where k = 6 takes over; a tally |
| 7 | cycles | "Let rules mutate, and helping rises and collapses, again and again." | Fig. 2's settings, fast; a helping gauge swinging |
| 8 | crowd | "And it works best in small groups:\nthe bigger the group, the less anyone helps." | bars: group sizes 20, 50 and 100 |
| 9 | point | "A good name helps only\nwhere people can keep track." | title |
| 10 | end | "Reputation — after Nowak & Sigmund, 1998\nndouglas.github.io/SugarScape" | title card |

About 65 s. The decision rules are the verdicts above, fixed before the episode's `claims.py`
re-measures them on each shot (Fig. 3's ordering over a longer window, so its numbers settle).

## Tune (proposed): *The Talk of the Town*

An original Charleston in B♭, 4/4: gossip set to a dance. A clarinet sings each phrase and a muted
trumpet repeats it round the band, as reputations get passed along, over banjo, tuba and brushed snare.

- Beats 1–4: clarinet and banjo, the tune plain.
- "paper": the full band, the trumpet answering.
- "here": the band drops out one by one until the tuba plays alone.
- "cycles": stop-time: the band hits and falls silent in turn, rising and collapsing.
- "crowd": the tune passes through more and more instruments, each echo quieter, until it can't be
  heard.
- The title: the band together once more, ending on a Charleston "hit".
