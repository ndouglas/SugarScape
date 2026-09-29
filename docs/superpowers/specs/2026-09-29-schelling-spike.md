# Spike: Schelling segregation (Following the Crowd, episode 1, "Neighbors like me")

**Date:** 2026-09-29
**Status:** storyboard proposed.
**Question:** which of Epstein & Axtell's claims about their variant of Schelling's model (1996,
pp. 165–171, animations VI-4 to VI-7) hold over 20 seeds, and what can the episode add by
answering the questions the book poses but leaves open?
**Method:** CLI runs of the presets over seeds 1–20 (throwaway probes), and an independent
book-versus-code review before the storyboard.

## The model in one paragraph

2,000 Red and Blue Flumps on a 50 × 50 torus (a fifth of the squares empty). Each wants at least a
share of its four neighbors (north, east, south, west) to be its own color. Taking turns in random
order, an unhappy Flump moves to a random empty square where it would be happy, or stays if there is
none. With a maximum residence (VI-5 to VI-7), each leaves after 80–100 turns and a newcomer of
random color takes a random square where it would be happy.

## The review, before the storyboard

- **The engine follows every stated rule.** The p. 165 box is "Schelling's agent movement rule"
  (nearest site); p. 166 says the variant differs: von Neumann neighbors, "our agents simply select
  an acceptable site at random", a torus. The engine does the variant. This is not an inconsistency
  in the book, and not a finding.
- **Choices the book leaves open,** filled by the engine and to be named as ours: a Flump with no
  neighbors is happy; with nowhere to go it stays; a newcomer with no acceptable square takes any;
  everyone starts at age 0 (so VI-5 to VI-7 sit still from about turn 3 to 80, then turn over). The
  book defines no measure of segregation; the engine's (the mean share of like neighbors) ranks the
  presets as three other measures do.
- **Every animation is "Typical Evolution"**, so the qualitative features the text names are fair
  to test as typical; its numbers aren't claims.
- **Four neighbors make preferences coarse:** 25 % means "at least one like neighbor"; anything in
  (¼, ⅓] acts as ⅓ and (⅓, ½] as ½. VI-7's "25–50 %" is a third of Flumps at ⅓ and two-thirds at ½.
  The steps, and any number, belong to Epstein & Axtell's variant, not to Schelling's 8-neighbor
  board; only "a small change in preferences, a large change in segregation" is Schelling's, and
  the book credits it to him.
- **Repository wording to fix:** "the book calls the two comparable" beside 0.63 and 0.76 is mildly
  unfair: the book gives no measure, VI-5 lands much nearer VI-4 than VI-6, and p. 170 itself calls
  VI-5 "modestly segregated".
- **Don't use the `quiet` column for "never settles"**: it counts moves, not replacements.

## Measured (seeds 1–20)

| Claim (quote) | Decision rule | Measured | Verdict |
|---|---|---|---|
| VI-4: "an equilibrium state is reached" | quiet, nobody unhappy, in 20 of 20 | still by turn 2–3, 0 unhappy, 20 of 20 | Holds |
| VI-4: "significantly … more segregated than the initial one" | end above start in 19+ of 20 | 0.50 → 0.62 (0.61–0.64), 20 of 20 | Holds |
| VI-5: "changes perpetually, never settling down" | squares keep changing color after turn 500 | (to measure in the build) | — |
| VI-5: "the degree of segregation is comparable" to VI-4 | VI-5 nearer VI-4 than VI-6, in 19+ of 20 | 0.76 against 0.62 and 0.94 | Holds |
| VI-6: "far higher than in the previous runs" | VI-6 above VI-5 above VI-4 in every seed | 0.94 (0.93–0.96) | Holds |
| VI-7: "a highly segregated pattern endures" | VI-7 nearer VI-6 than VI-5, 19+ of 20 | 0.93 (0.92–0.94) | Holds |
| "a relatively small change in individual preferences leads to a large change" (Schelling's) | 25 % → 50 %, no residence: 0.2+ more | 0.62 → 0.83 | Holds |

**Beyond the book** (its questions, our answers; framed as ours):

| Preference, everyone the same, no residence | 0 % | 20–25 % | 30 % | 34–50 % | 60 % | 67–75 % |
|---|---|---|---|---|---|---|
| Segregation (median of 20) | 0.50 | 0.62 | 0.73 | 0.83 | 0.94 | 0.78 |
| Unhappy at the end | 0 | 0 | 0 | 0 | 0 | 31 % |

- **"How little racism is enough to 'tip' a society?"** Any at all moves it; then it climbs in steps
  at a quarter, a third, a half and two-thirds, because a Flump has four neighbors.
- **Too picky, and the town freezes:** at two-thirds, a third of the Flumps are unhappy with nowhere
  acceptable to go; moving stops, and the town is *less* sorted (0.78) than at 60 %.
- **"Is racial segregation reversible through 'invasion' by a handful of 'color-blind'
  individuals?"** Not yet testable. **Proposed switch** `colorblind`: the chance each newcomer is
  color-blind (happy anywhere, placed anywhere). Run from VI-6 (50 %, residence) with 0, 10, 25, 50
  and 100 % color-blind newcomers, to turn 1,000. Decision rule, fixed now: the caption reports the
  smallest share whose median segregation at turn 1,000 falls below VI-5's level (0.76), or says no
  share short of all of them does.
- **Why churn sorts further (0.62 → 0.76 at 25 %)**: probably because newcomers only take squares
  they like. **Proposed switch** `newcomers: satisfied | anywhere`. Decision rule: if VI-5 with
  newcomers placed anywhere comes out within 0.03 of VI-4 (median), the caption says newcomers
  choosing do the sorting; otherwise it just says churn sorts further.

## How to film it

The studio's squares board: 50 × 50 felt, a Flump per agent in red or blue yarn, empty squares
bare. The unhappy frown (or glow yellow). A move is a hop to its new square. The build adds a
`schelling` shot: each turn's agents (square, color, preference, happy) and, in slowed shots, each
move.

## Storyboard (proposed, about 75 s)

| # | Beat | Caption | Shot |
|---|---|---|---|
| 1 | want | "Each Flump wants at least one neighbor its own color." | close-up, 25 % |
| 2 | move | "If it has none, it moves to a random empty square where it would." | a turn slowed |
| 3 | settle | "In three turns, everyone is content, and the town is more sorted:\nhalf of neighbors alike becomes 62 %." | VI-4 |
| 4 | churn | "Let Flumps move away and newcomers arrive, and it never settles." | VI-5 |
| 5 | why | (depends on the newcomers switch) "It sorts a little further (76 %), because newcomers choose." | VI-5 |
| 6 | half | "Ask for half, and the town all but splits: 94 %." | VI-6 |
| 7 | mixed | "Mix in Flumps asking for only a third, and it barely helps: 93 %." | VI-7 |
| 8 | steps | "How little is enough? Any at all. Then it climbs in steps:\na quarter, a third, a half, with four neighbors." | the tipping chart |
| 9 | freeze | "Ask for two-thirds, and the town freezes:\na third of the Flumps are unhappy, with nowhere to go." | 67 % |
| 10 | invade | (depends on the measurement) "Can color-blind newcomers undo it? …" | VI-6 with color-blind newcomers |
| 11 | point | "Nobody asked for a divided town.\nMost asked for one neighbor like them." | title |
| 12 | end | "Neighbors like me — after Schelling, 1969–78; Epstein & Axtell, 1996" | end card |

## Tune (proposed): *Hocket for Two Colors*

An original piece in which two instruments, oboe (red) and clarinet (blue), share one melody note by
note (a hocket), interleaved like the starting town.

- "want", "move": the melody plain, hocketed.
- "settle": the notes drift into runs of each color: two, then three at a time.
- "churn": the runs keep reshuffling, never quite repeating.
- "half", "mixed": the voices split into separate phrases in separate registers.
- "steps": the melody rises in four terraced steps.
- "freeze": a held chord, nobody moving.
- "invade": a third timbre (a flute, color-blind) threads between them.
- The title: the hocket returns, both colors in turn, ending in unison.
