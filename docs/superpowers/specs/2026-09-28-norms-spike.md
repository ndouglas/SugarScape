# Spike: norms and metanorms (Cooperation, episode 6)

**Date:** 2026-09-28
**Status:** built as `studio/episodes/norms` (claims in its `measurements.md`: all ten hold, with the
counts the spike found; 77 s). Dominance stays out. Its engine fix is done (2026-09-29): Axelrod's rule is the default, ours a named
switch (see the README).
**Question:** what do Axelrod (1986) and Galán & Izquierdo (2005) each claim, which claims hold over 20
seeds, and how do we film a model of 20 Flumps whose traits drift over a million generations?
**Method:** an independent paper-versus-code review *before* the storyboard (see below), and CLI runs
of the presets over seeds 1–20 at generations 100, 1,000, 10⁴, 10⁵ and 10⁶, under both readings of the
one sentence the papers disagree on. The runs are throwaway probes, not kept.

## The model in one paragraph

Twenty Flumps each have a boldness and a vengefulness, eight levels from 0/7 to 7/7. Four times a
generation each gets a chance to cheat, seen by each other Flump with a random chance S; it cheats if
S is below its boldness, gaining 3 and costing everyone else 1. Whoever sees a cheat punishes it with
probability equal to their vengefulness: −9 to the cheat, −2 to the punisher. In the *metanorms* game,
whoever sees a cheat go unpunished can also punish the one who looked away, by the same rule. After
each generation, Flumps a standard deviation above the average payoff have two offspring, those a
standard deviation below have none, and the rest one; each trait's bits mutate 1 % of the time.

## What each paper claims

- **Axelrod (1986), the norms game:** outcomes vary over his five 100-generation runs, including "the
  partial establishment of a norm" (p. 1100); in the long run, "the norm completely collapsed … a
  stable outcome" (p. 1109). **Galán & Izquierdo agree** (§5.13: "the norm collapses almost always, as
  Axelrod concluded").
- **Axelrod, the metanorms game:** "In all five runs a norm against defection was established"
  (p. 1102); metanorms "promote and sustain cooperation" (p. 1109).
- **Galán & Izquierdo:** "while Axelrod's conclusions were correct in the short-term, the long-term
  behaviour … is significantly different … metanorms do not prevent defections most of the time in the
  long-term" (§2.3). They stress this "should not be understood as a critique" (§1.6): the computing
  power "was simply not available". Milder metapunishment reverses the result (§6.8), a temptation of
  10 keeps the norm (§6.10), and a lower mutation rate makes the collapse quicker (§6.6).
- **"Established" and "collapsed"** are Galán & Izquierdo's regions (§5.12): mean boldness ≤ 2/7 with
  vengefulness ≥ 5/7, and boldness ≥ 6/7 with vengefulness ≤ 1/7. Axelrod gives no thresholds. Captions
  that count them say so.
- **The sentence that decides:** when every payoff ties, nobody is a standard deviation from the
  average. Axelrod's text says to "give an average individual one offspring" (p. 1099): everyone keeps
  one. Galán & Izquierdo call the case ambiguous (their note 4) and give everyone two offspring, then
  remove a random half. The engine's default (`all_equal: drift`) is their reading; `keep` is his words.

## The review, before the storyboard

An independent reviewer read both papers against the engine and found:

- **The dominance variant isn't Axelrod's.** He has each group's defections hurt, and be punished by,
  only the other group, with metapunishment within groups (p. 1103). Our engine lets everyone punish
  everyone, a silent departure that makes his second dominance claim look like a failure. **Dominance
  stays out of the video**; fixing the engine (his rule as the default, ours as a named switch) is
  separate work.
- **The long-run collapse depends on the tie reading** (measured below). The repository's "Axelrod left
  two things unstated" should say his text is ambiguous where Galán & Izquierdo flag it.
- **Other readings taken from Galán & Izquierdo** (the step rule for offspring, the population standard
  deviation, who may catch a non-punisher, whether a metapunisher must have seen the cheat) are silent
  choices. They should be listed as readings, not presented as Axelrod's.
- Everything else (the payoffs, S, the metanorm's "same S" and "same vengefulness", mutation) matches.

## Measured (seeds 1–20; established / collapsed at each generation)

| Game | Tie reading | 100 | 1,000 | 10⁴ | 10⁵ | 10⁶ |
|---|---|---|---|---|---|---|
| Norms | G&I (everyone two, half removed) | 1 / 5 | 0 / 20 | 0 / 19 | 0 / 20 | 0 / 20 |
| Norms | Axelrod's words (everyone one) | 0 / 5 | 0 / 20 | 0 / 20 | 0 / 19 | 0 / 20 |
| Metanorms | G&I | 17 / 0 | 17 / 0 | 16 / 1 | 11 / 9 | 2 / 18 |
| Metanorms | Axelrod's words | 15 / 0 | 20 / 0 | 19 / 0 | 19 / 0 | 14 / 5 |
| Mutation 0.001 | G&I | 18 / 0 | 14 / 2 | 3 / 16 | 0 / 20 | 0 / 20 |
| Mutation 0.001 | Axelrod's words | 18 / 0 | 17 / 0 | 19 / 0 | 20 / 0 | 7 / 13 |
| Milder metapunishment | G&I | 3 / 3 | 0 / 18 | 0 / 20 | 0 / 20 | 0 / 20 |
| Milder metapunishment | Axelrod's words | 1 / 4 | 0 / 17 | 0 / 20 | 0 / 20 | 0 / 20 |
| Temptation 10 | G&I | 20 / 0 | 20 / 0 | 20 / 0 | 20 / 0 | 20 / 0 |
| Random tournament | G&I | 8 / 0 | 2 / 15 | 0 / 18 | 0 / 20 | 0 / 20 |

**Verdicts:**

- The norms game collapses: both authors say so, and it does (20 of 20 by generation 1,000).
- Metanorms establish the norm at 100 generations, as Axelrod reported (17 of 20; 15 under his tie
  reading).
- Under Galán & Izquierdo's implementation, the norm usually collapses over a million generations
  (18 of 20), as they showed.
- Under Axelrod's own words for ties, it usually doesn't (14 of 20 still established at 10⁶).
- Milder metapunishment reverses the result under either reading; temptation 10 keeps the norm.

**For the finale:** not a failure of either paper, but a sentence: one ambiguous rule decides the
long-run result.

## How to film it: the plane of temperaments

The board is an 8 × 8 felt plane: boldness from left (0) to right (7/7), vengefulness from front (0)
to back (7/7). Each Flump stands on its square; Flumps on the same square crowd together. The
"established" corner (Galán & Izquierdo's region, back left) is tinted green, the "collapsed" corner
(front right) red, with labels. A slowed generation shows a cheat (the Flump lifts a stolen gumdrop),
the punishments (a red flash from each punisher), and in the metanorms game the punishment of those who
looked away.

The build adds a `norms` shot: each generation's agents (boldness, vengefulness, payoff) and, in slowed
shots, the round's events. A new `every` field records every *n*th generation, so a million generations
can be filmed at a few thousand frames; the on-screen counter shows the true generation.

## Storyboard (proposed)

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | traits | "Each Flump has a boldness, and a vengefulness." | the plane, labeled |
| 2 | cheat | "A bold Flump cheats when it thinks nobody is looking:\nit gains 3, and everyone else loses 1." | one generation slowed |
| 3 | punish | "Anyone who sees it may punish it, if vengeful enough:\n−9 to the cheat, −2 to the punisher." | the same |
| 4 | norms | "Axelrod, 1986: on their own, punishments fade.\nThe cheats take over, in 20 of 20 worlds." | the norms game, fast; the Flumps drift to the red corner |
| 5 | meta | "So he added one rule: punish anyone who looks away." | the metanorms game, one generation slowed |
| 6 | holds | "The norm holds, in 17 of 20 worlds, as he found." | metanorms to generation 100; the Flumps gather in the green corner |
| 7 | long | "In 2005, Galán and Izquierdo ran it a million generations.\nThe norm collapsed, in 18 of 20." | sampled every 5,000 generations; a counter |
| 8 | sentence | "But when every Flump earns the same, who has the children?\nAxelrod: each has one. They: each has two, and half are removed." | a still; the two readings side by side |
| 9 | axelrod | "Read his way, the norm still holds after a million generations,\nin 14 of 20." | the same million, his reading |
| 10 | mild | "Either way, make metapunishment milder, and the norm collapses." | milder metapunishment, fast |
| 11 | point | "Both were right.\nOne sentence decides the long run." | title |
| 12 | end | "Norms — after Axelrod, 1986; Galán & Izquierdo, 2005\nndouglas.github.io/SugarScape" | title card |

About 80 s. The decision rules are the verdicts above, re-measured by the episode's `claims.py` on each
shot; the counts in the captions are whatever those runs give.

## Tune (proposed): *A Fugue for Keeping Order*

An original fugue in C minor for string quartet: a rule that every voice must follow, and must enforce
in the others.

- Beats 1–3: the subject alone on the viola.
- The norms game: the voices enter but don't answer each other, and the fugue frays into single
  unaccompanied lines.
- Metanorms: a proper exposition, each voice answering the last; the fugue holds.
- The long run: the same exposition, faster, the answers slipping out of step one by one until
  the fugue falls apart.
- "sentence": a single phrase on the cello, alone.
- "axelrod": the fugue resumes and holds, in stretto.
- "mild": it frays again.
- The title: all four voices together on the subject, ending on a C major chord (a Picardy third).
