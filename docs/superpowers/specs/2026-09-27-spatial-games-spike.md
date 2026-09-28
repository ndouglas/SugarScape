# Spike: spatial games (Cooperation, episode 1)

**Date:** 2026-09-27
**Status:** built as `studio/episodes/spatial` (claims in its `measurements.md`); the beats were
retimed so the tune's sections land on "clock" and "below" (78 s).
**Question:** can the Flumps film Nowak & May's spatial Prisoner's Dilemma, and which of the papers'
claims hold on the board we would film?
**Method:** throwaway probes (not kept): a Blender timing test, and CLI runs over seeds 1–20 on a
99 × 99 board, the size we would film, not the papers' 200 × 200 and 400 × 400.

## The model in one paragraph

Every square holds a Flump that either helps or cheats. Each generation, every Flump plays each of
its eight neighbors and itself. Two helpers earn 1 each; a cheat facing a helper earns b (1.9
here), and the helper nothing; two cheats earn nothing. Then every Flump copies whichever of its
neighbors (itself included) earned the most. There is no memory, no strategy beyond help or cheat,
and no movement.

## Can we film it?

Yes. A 99 × 99 crowd of 9,801 Flumps, recolored every frame, renders at about 3.5 s a preview frame
(6 frames in 20.7 s; 49 × 49 takes 1 s a frame), about 12 minutes for a 7-second beat. From
above, the crowd reads as a clear pattern of blue and red.

## Measured (seeds 1–20, 99 × 99, fixed edges)

| Candidate claim | Measured | Verdict |
|---|---|---|
| One cheat among helpers makes a kaleidoscope (NM92 Fig. 3) | Identical in every seed (the start has no randomness); four-fold symmetric; reaches the edges at t = 49 | Holds |
| …that never settles | Over t = 50–1000, 22 % of Flumps change side each generation on average; no generation without change; f_C wanders 0.18–0.55 | Holds |
| About a third end up helpers (NM92's 0.318) | From 10 % or 40 % cheats at b = 1.9: f_C 0.325 (0.320–0.331) over t = 201–300, 20 of 20 | Holds as "about a third"; 0.318 is the 400 × 400 value |
| "for almost all starting proportions" | From 70 % cheats: 18 of 20 near a third, 1 all cheats, and 1 (seed 17) all helpers: its 71 surviving helpers grow until they fill the board. From 90 % cheats: all cheats in 14 of 20 | Holds up to about half cheats on this board |
| Helpers earn more on average | Mean payoff of helpers above that of cheats in every generation of t = 100–300, 20 of 20 | Holds. This is the episode's "why" |
| Less temptation freezes the pattern (NM92 Fig. 1a) | b = 1.77: f_C 0.749 (0.740–0.758), 3 % changing each generation | Holds: "three in four help, and the world nearly freezes" |
| Much more temptation: cheats win | b = 2.1: f_C 0.035 (0.021–0.046) at t = 300, but all cheats in 0 of 20 | Cheats win almost everything; a few helper clusters survive |
| One at a time, one cheat takes the world (HG93) | Asynchronous kaleidoscope, b = 1.9: all cheats in 20 of 20, at t = 56–149 | Holds |
| …at any temptation (HG93 never give b) | Asynchronous kaleidoscope, b = 1.7: cheats persist as a small stuck cluster (31–225 of 9,801) in 20 of 20; f_C 0.974–0.999 at t = 400 | Fails below 1.8 |
| Continuous time keeps coexistence below the chaos (NBM94) | 50 % cheats, periodic edges, b = 1.71: one at a time f_C 0.721 (0.713–0.730), all at once 0.884; 0 of 20 all cheats either way | Holds |
| …and only the chaos needs the clock | b = 1.9: one at a time, all cheats in 20 of 20; all at once, 1 of 20 | Holds |

**For the finale:** the chaos, and "about a third", exist only because every Flump moves at once.
"Almost all starting proportions" means up to about half cheats on a board this size. HG93's
"always" holds only above b = 1.8, which they never state.

**Preset fix (engine, not studio):** `hg-async-kaleidoscope`'s description says that at b = 1.7
"the lone defector dies out (f_C 0.974–0.999)". It doesn't: those numbers mean 31–225 cheats
remain. The description should say the cheats stay a small cluster.

## The frame dump

`frames.rs` rejects every model but the Sugarscape. The build adds a spatial shot and dump:

- A shot names a spatial preset or config, as now. It may also give `cells`: a row-major string of
  `C` and `D` for a close-up's hand-made start, replacing the config's start.
- Each frame carries `strategies`, a row-major string of `C`, `D` and `.` (no player). With
  `"scores": true` in the shot, each frame also carries every player's score, for close-ups. Wide
  shots leave the scores out: 9,801 floats a frame would make the dump 20 MB for 200 ticks.
- The stats are the model's series, as for the Sugarscape.

The studio's `dump.py` gains a `Lattice` loader, and the scene gains a lattice board: plain felt,
one Flump per square. Nowak & May's change colors light the felt under each Flump that switched
this generation: green for cheat → helper, yellow for helper → cheat. A Flump that switches sides
hops and is re-knitted at the top of the hop.

## Storyboard (proposed)

Helpers are blue, cheats red (Nowak & May's colors, and the yarn the Sugarscape tribes wore).

| # | Beat | Caption | Shot and overlays |
|---|---|---|---|
| 1 | play | "Every Flump plays a game with each of its neighbors." | close-up, a 7 × 7 board (hand-made start), one generation slowed; a line to each neighbor |
| 2 | pay | "Two helpers earn 1 each. A cheat facing a helper earns almost 2, and the helper nothing." | close-up; scores over each Flump |
| 3 | copy | "Then every Flump copies whichever neighbor earned the most." | close-up; the losers hop and turn red or blue |
| 4 | one | "One cheat, in a world of 9,800 helpers." | 99 × 99 kaleidoscope, t = 0–3 |
| 5 | bloom | "Cheating spreads…" | wide, t = 3–49; change colors on the felt |
| 6 | never | "…but never wins, and never settles." | wide, t = 49–250, faster |
| 7 | third | "Scatter the cheats anywhere: about a third of the Flumps end up helping." | 99 × 99, 10 % cheats, b = 1.9; a helpers gauge |
| 8 | why | "On average, helpers earn more than cheats, every generation." | the same, lower; live bars of mean earnings. The storyboard's first sentence, "a cheat beside helpers out-earns them", failed when measured (the cheat wins 49 % of the frontier's pairs, 48–50 %) and was dropped |
| 9 | tempt | "Make cheating a little less tempting, and three in four help." | b = 1.77; the pattern freezes into walls of cheats |
| 10 | clock | "But here, everyone moves at once. In 1993, Huberman and Glance let the Flumps move one at a time." | asynchronous kaleidoscope, b = 1.9 |
| 11 | lost | "One cheat takes the whole world, in 20 of 20." | wide, fast; the helpers gauge falls to 0 |
| 12 | below | "Unless cheating is less tempting: then helpers survive either way." | asynchronous, b = 1.71, 50 % cheats; about 72 % help |
| 13 | point | "Nobody has to be nice.\nThey just have to be neighbors." | title |
| 14 | end | "Spatial games — after Nowak & May, 1992\nndouglas.github.io/SugarScape" | title card |

About 80 s. The decision rules for each caption are the verdicts above, fixed before the episode's
`claims.py` re-measures them on each shot.

## Tune (proposed): *Neighbors in Phase*

A marimba and a vibraphone play the same twelve-note figure, after Steve Reich's *Piano Phase*.
Every eight bars one voice slips ahead by one eighth note, so the figure keeps making new patterns
out of the same notes, as the kaleidoscope does. The piece is in 12/8, around 100 beats a minute,
in D Dorian, with a soft bowed bass drone.

- Beats 1–3: the figure alone, in unison.
- Beats 4–9: the phasing, with a pizzicato bass joining for the chaos.
- At "clock" (beat 10): the second voice falls out of step into triplets against the eighths, and
  the lock is lost. The bass drops a half step, darker, as red takes the board.
- At "below" (beat 12): the voices find unison again.
- The title: the figure, once, in unison, ending on the open fifth.

Each phase shift is a discrete one-eighth step, which ABC and our MIDI renderer can play
(continuous phasing is out of reach).
