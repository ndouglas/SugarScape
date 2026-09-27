# War (episode 7): storyboard and tune

**Date:** 2026-09-27
**Builds on:** `2026-09-26-markets-and-war-spikes.md` (the War spike and the book check) and the
series plan. Approved storyboard and tune direction. Every number below comes from the spike, and
the episode's `claims.py` re-measures it over 20 seeds before it becomes a caption. If a claim
fails, the caption changes.

## The story: deterrence

Rule C lets a Flump attack only an enemy poorer than itself, and never where a richer enemy could see
it afterward. So nobody fights an equal. War waits until someone can't lose, and then it is quick and
one-sided (III-9, which reproduces the book's "stunning blitzkrieg"). Without deterrence, with the
loot capped and every death replaced (III-11), the war never ends. The book's battle front doesn't
appear under its own stated rules. III-12's missing collision and III-14's civil war stay for the
finale.

## Storyboard (about 80 s)

| # | Beat | Caption | Shot |
|---|---|---|---|
| 1 | fight | "Now Flumps can fight. A Flump may attack an enemy poorer than itself…" | close-up: a rich Blue beside a poorer Red |
| 2 | take | "…and takes everything it had." | close-up: the pounce; the victim's sugar arcs to the attacker |
| 3 | wary | "But never if a richer enemy could see it afterward." | close-up: a poorer Red within reach, a richer Red in sight; no attack |
| 4 | hills | "Two tribes of 200, each on its own hill." | wide III-9, a seed that conquers early |
| 5 | quiet | "For hundreds of ticks, almost nothing happens." | wide, fast; a kill counter barely moving |
| 6 | warlord | "Then one Flump gets rich enough that no one can stop it." | the top killer ringed; its wealth shown |
| 7 | fallen | "In 14 of 20 worlds, one tribe is wiped out, or nearly." | bars |
| 8 | one | "Most of the killing is done by a single Flump." | bars: the top killer's share |
| 9 | spoils | "The winners end up far richer than any Flump in a world without war." | bars: the richest Flump, war against none |
| 10 | standoff | "In the other worlds, each tribe keeps its hill, and nobody attacks." | wide, a seed without conquest |
| 11 | endless | "Now cap the loot, and replace every Flump that dies." | wide III-11 |
| 12 | nofront | "The book shows a battle front. Under its own rules, there isn't one: newcomers land among enemies and die." | wide III-11; where kills happen |
| 13 | question | "Nobody fights an equal. / War waits until someone can't lose." | title |
| 14 | end | "War — after Epstein & Axtell, 1996 / ndouglas.github.io/SugarScape" | title card |

**Claims to measure** (spike values in brackets):
- conquest by tick 2000 [≥ 90 % one tribe in 14 of 20];
- how quiet the start is [5 kills by tick 500; first kill at a median of tick 137];
- the top killer's share of the kills in conquest worlds [about 70 %];
- the richest Flump with war against without [1.0 × 10⁵ against 4.4 × 10³];
- standoffs [6 of 20];
- III-11:
  - both tribes survive [20 of 20, by replacement];
  - kills barely concentrate near the halfway line [26 % within 5 cells, against 12 % of Flumps];
  - victims are young [median age 2 ticks].

## Tune: "Poseidon's Horn"

An original march in 5/4, after the effect of King Crimson's "The Devil's Triangle" (itself after
Holst's "Mars"). The ostinato is our own figure, not Holst's.

- **The quiet** (beats 4–5): a soft 5/4 ostinato on low strings and timpani; a snare that almost
  starts.
- **The build:** trombones, then the snare in earnest, then a fife line above, climbing.
- **The horn** (beat 6): a sting. One enormous sustained chord stacked across the brass patches, choir
  and a low organ pedal, swelling on a timpani roll. The sting **ducks** the main track, so the march
  collapses beneath it; the sting's own figures fall apart as it rises.
- **After** (beats 7–10): the horn's echo fades to a low drone.
- **The endless war** (beats 11–12): the ostinato again, grinding, unresolved; the end card lands on
  an open fifth.
- The horn has its own MIDI tracks, so GarageBand can give it a bigger patch than FluidR3's General
  MIDI brass.

## New studio work

- **Engine:** kills in the frame dump: each combat death's attacker and victim and the loot taken.
  Today a death records only its cause.
- **Blender:**
  - a pounce;
  - the loot arc (after the bequest arcs);
  - a kill counter;
  - a ring on the top killer, with its wealth;
  - a map of where kills happen.
- **Music:** a sting can duck the main track by a set amount over its length.
