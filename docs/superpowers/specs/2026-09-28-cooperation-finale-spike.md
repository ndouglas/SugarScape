# Spike: the Cooperation finale (episode 8), "Why help a stranger?"

**Date:** 2026-09-28
**Status:** approved and built as `studio/episodes/stranger` (claims in its `measurements.md`: all eleven
hold; 78 s), after an independent fairness review of the first draft (below).
**Builds on:** the series plan and every episode's `measurements.md` (all claims held there, over 20
seeds, after the paper-versus-code reviews). The finale's `claims.py` reads the episodes'
measurements and fails if any row's source claim no longer holds; the one new measurement (Run 4
under the working paper's rule) it runs itself.

## The review of the first draft

An independent reviewer checked every row and caption against the papers. Changes made:

- **Huberman & Glance** moved from "one sentence read two ways" to "reproduces": updating one at a
  time is a model choice they argued for (p. 7716), not a reading of a sentence, and their takeover
  reproduces in the regime they ran; Nowak, Bonhoeffer & May's reply reproduces too.
- **Riolo, Cohen & Axelrod's tie rule** left the "read two ways" rows: the paper states its rule
  (p. 441's parenthetical and p. 442); only a reading that ignores it gives a coin flip. Listing it
  would undo the tags episode's correction.
- **Epstein** is not one sentence read two ways: his 1998 working paper and his 2006 book print two
  different rules beside the same 779, and matching it also needs founders who start with nothing,
  which neither states.
- **Hammond & Axelrod's 14 %** (color-blind, double cost) is dropped: it appears only in the text,
  without its setup, and their companion paper (TPB 2006), which may give it, is still unread. Their
  56 % (Table 1 c) stays: every seed here is above it (59–74 %).
- **Epstein's Run 4:** he describes outcomes that vary (coexistence, cooperators alone, extinction),
  with no frequencies, so the row quotes that. Run under the working paper's rule too (decision rule
  fixed first: "under either rule" only if extinct in 15+ of 20 under both): extinct by cycle 500 in
  19 of 20 (the book's rule), 18 (the working paper's) and 18 (its closest settings), with cooperators
  alone in 0, 1 and 2.
- **The norms beat** no longer says "for a while", which took Galán & Izquierdo's side; **image
  scoring** says "everyone ends up helping", not "helps those who help" (that is only the k = 0
  worlds); **living** is "neighbors, and children born next door" (Epstein's Flumps move every turn).
- **The closing line** "they help the ones they know" was wrong for three papers: Nowak & Sigmund's
  framework "does not require the same two individuals ever to meet again", Riolo, Cohen & Axelrod's
  pairs rarely meet again, and norms has no helping at all.
- **The end card:** seven episodes, ten papers.

## What the series found

| Episode | What made helping work | Measured (20 seeds) | Take it away | Measured |
|---|---|---|---|---|
| 1 Spatial games | neighbors | about a third help, in clusters (32.4 %) | — | — |
| 2 Living neighbors | neighbors, and children born next door | 728 helpers to 171 cheats | stirred into a soup | no helper left by cycle 12 in 19 of 20 |
| 3 Ethnocentrism | relatives next door | 86.4 % of helps go to relatives | children put anywhere | cooperation 4.9 % |
| 4 Tags | a shared look | 94.3 % of gifts go to an exact look-alike | look-alikes not made to help | giving 1.4 % |
| 5 Reputation | a good name | everyone ends up helping in 7 of 20 worlds | — | — |
| 6 Norms | a rule everyone enforces | the norm holds at generation 100 in 17 of 20 | milder metapunishment | collapses in 20 of 20 |
| 7 Friends and strangers | partners who stay | payoff 2.48 of 3 | new strangers every period | 1.08 |

## The ledger

**It reproduces** (the paper's figure; here, the 20-seed median or count):

| Paper | Claim | Paper | Here |
|---|---|---|---|
| Nowak & May 1992 | helpers, 1.8 < b < 2 (we use 1.9) | about a third (0.318, 400 × 400) | 32.4 % |
| Huberman & Glance 1993 | one at a time, one cheat takes the world | "within a hundred generations or so" | 20 of 20, by t = 56–149 |
| Nowak, Bonhoeffer & May 1994 | less temptation, and helpers survive either way | coexistence | 72 % one at a time, 88 % all at once |
| Hammond & Axelrod 2006 | help only their own color (Table 1 a) | 76.3 % | 76.7 % |
| Riolo, Cohen & Axelrod 2001 | meetings that end in a gift (Table 1, the tie rule p. 442 states) | 73.6 % | 73.7 % |
| Nowak & Sigmund 1998 | cooperation in groups of 20, 50, 100 (Fig. 3) | 90 / 47 / 18 % | close: 89 / 41 / 16 % |
| Axelrod 1986 | metanorms establish the norm, 100 generations | 5 of 5 runs | 17 of 20 (G&I's region) |
| Cohen, Riolo & Axelrod 2001 | Table 2's payoffs, six of seven structures | 1.09–2.56 | each within 0.09, same order |

**It depends on a detail the papers give two ways:**

| Paper | The detail | One way | The other |
|---|---|---|---|
| Epstein 1998 / 2006 | games a turn, and the founders' wealth | the book's each neighbor: 733 helpers | the working paper's one neighbor: 760; with founders who start with nothing (unstated): 784 (his 779) |
| Axelrod 1986 / Galán & Izquierdo 2005 | who has the children when all earn the same | G&I's (who flag it as ambiguous): the norm collapses by 10⁶ in 18 of 20 | Axelrod's words: it holds in 14 of 20 |

**It didn't come back** (the rules as written; the cause is not known):

| Paper | Claim | Paper | Here |
|---|---|---|---|
| Hammond & Axelrod 2006 | cooperation at double cost (Table 1 c) | 56 % | 67 % (every seed above) |
| Epstein 1998 / 2006 | Run 4: outcomes vary | coexistence, cooperators alone, or extinction | extinct by cycle 500 in 18–19 of 20, under either rule |

Left out, as not findings about the papers: Nowak & Sigmund's Fig. 1 is one run shown as an example;
the tags twins are reported by the paper itself; FRNE over 2DK is note 5's, and the paper calls the
difference unimportant; the 14 % (companion paper unread); dominance (not yet Axelrod's model here).

## Storyboard (about 85 s)

| # | Beat | Caption | Shot |
|---|---|---|---|
| 1 | question | "Seven episodes, one question:\nwhy would anyone help a stranger?" (title) | friends' strangers grid |
| 2 | neighbors | "Neighbors: about a third keep helping, in clusters." | spatial's kaleidoscope |
| 3 | relatives | "Relatives: most help goes to kin next door." | ethno's land |
| 4 | look | "A shared look: most gifts go to an exact look-alike." | tags' ring |
| 5 | name | "A good name: in 7 of 20 worlds, everyone ends up helping." | image's street |
| 6 | rule | "A rule everyone enforces: the norm holds.\nHow long depends on one sentence." | norms' plane, metanorms |
| 7 | faces | "Partners who stay: cooperation holds." | friends' fixed partners |
| 8 | strangers | "New strangers every period: it almost never takes hold." | friends' strangers, gauge at 1.08 |
| 9 | honest | "Now the honest part: what reproduced, and what didn't." | the same |
| 10 | held | "Most of it reproduces, closely." | ledger 1 (8 rows) |
| 11 | detail | "Two results depend on a detail the papers give two ways." | ledger 2 |
| 12 | didnt | "And two didn't come back, as written." | ledger 3 |
| 13 | point | "Why help a stranger?\nThe Flumps do, when something makes the stranger less strange." (title) | friends' fixed partners |
| 14 | end | "Why help a stranger? — seven episodes, ten papers, 1986–2006\nndouglas.github.io/SugarScape" | end card |

Montage shots reuse each episode's shot files, so every Flump still comes from a real run of its
model. Montage beats about 4.5 s each; ledgers 8–12 s.

## Tune (proposed): *Home Again*

A medley quoting each episode's tune in turn, in its own instruments, over one steady walking bass:
*Neighbors in Phase* (marimba), *A Round for Neighbors* (flute and clarinet), *Cradle Song for Four
Colors* (music box), *The Twins' Slip Jig* (whistle and fiddle in unison), *The Talk of the Town*
(clarinet and banjo), *A Fugue for Keeping Order* (string quartet), *Swing Your Partner* (fiddle and
banjo). "Strangers": the bass alone. The ledgers: a quiet reprise of the first theme on piano. The title:
every theme's opening bar in turn over one chord progression, ending on G major.

## Studio work

A finale episode (`studio/episodes/stranger/`) whose shots copy earlier episodes' shots (every board
already exists) and the existing two-column ledger (the paper's claim and figure; here).
