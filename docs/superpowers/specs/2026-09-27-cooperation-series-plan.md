# Cooperation: the second Flump series

**Date:** 2026-09-27
**Status:** in progress. Episodes 1 (Spatial games), 2 (Living neighbors,
`2026-09-28-demographic-pd-spike.md`) and 3 (Ethnocentrism, `2026-09-28-ethnocentrism-spike.md`) are
built.
**Follows:** the Sugarscape series (`2026-09-26-sugarscape-series-plan.md`, complete), and keeps its
rules.

## The question

Why would anyone help a stranger? Helping costs the helper and pays the one helped, so a cheat
should always do better. Seven reproduced papers each give an answer, and each episode asks what a
different fact about the Flumps does for cooperation:

- who they meet (space);
- who they're related to, and who they look like (kin and tags);
- what they know about each other (reputation);
- who enforces the rules (norms);
- who they keep meeting (social structure).

As in the first series, each episode shows where the paper's stated rules don't give the paper's
result, and a finale collects them.

## Rules (unchanged from the first series)

- Every Flump behavior comes from a real run: a frame dump of the model at the shot's own size.
- Every caption is measured over 20 seeds on the shot's config. If a claim fails, the caption
  changes, not the data. Decision rules are fixed before measuring.
- Each episode gets a new original tune in ABC.
- American spelling.
- The process: spike → storyboard and music proposal → approval → build → preview (960 × 540).

## Episodes

| # | Episode | Source | Layout | The finding |
|---|---|---|---|---|
| 1 | Spatial games ✅ | Nowak & May 1992; Huberman & Glance 1993; Nowak, Bonhoeffer & May 1994 | 99 × 99 lattice | Helpers survive in clusters: about a third help forever at b = 1.9. Move the Flumps one at a time and one cheat takes the world (20 of 20), but only above b = 1.8 |
| 2 | Living neighbors (demographic PD) ✅ | Epstein 1998 | 30 × 30 torus, moving agents with wealth | Space alone saves cooperators; mixed in a soup they are gone by cycle 8. Tables 1 and 2 need unstated rules |
| 3 | Ethnocentrism | Hammond & Axelrod 2006 | 50 × 50 torus, 4 colors | Favoritism works only because relatives live next door: scatter the babies and cooperation falls from 76% to 4.5%. Color-blind cooperation doesn't reproduce; the appendix's mutation rate is a slip |
| 4 | Tags | Riolo, Cohen & Axelrod 2001 | 100 agents, well mixed | The cooperation is identical twins forced to help each other; the tolerance the paper credits does nothing. Table 1 needs an unstated tie rule |
| 5 | Reputation (image scoring) | Nowak & Sigmund 1998; Leimar & Hammerstein 2001 | 100 agents, well mixed | The showcase run is the lucky one: discriminators win 1 run in 5. The universal constant reproduces to every digit; "two interactions per lifetime" doesn't suffice |
| 6 | Norms | Axelrod 1986; Galán & Izquierdo 2005 | 20 agents | Punishing those who don't punish establishes the norm (92 of 100), but run on, an unstated tie rule decides whether it lasts |
| 7 | Friends and strangers (social structure) | Cohen, Riolo & Axelrod 2001 | 256 agents: torus, fixed networks or strangers | The one that reproduces: fixed friends cooperate as well as neighbors (2.57 vs 2.55); strangers sit at 1.09 |
| 8 | What didn't reproduce | all | — | The ledger |

Numbers in this table are the repository's own records, at each model's seed count (10–100).
Every caption is re-measured over 20 seeds on its shot before it goes on screen.

The order builds the argument: space first (1–2), then what can stand in for space (kinship and
look-alikes, 3–4; reputation, 5; enforcement, 6), then structure without space (7).

## Studio work

The frame dump (`frames.rs`) runs the Sugarscape only. Each episode adds its model to it, the way
the Sugarscape episodes added fields:

- **Grid models** (spatial games, ethnocentrism, the demographic PD, social structure's torus): a
  Flump per player or agent on the felt, recolored as its strategy changes. The spike measured the
  cost: a 99 × 99 crowd of 9,801 Flumps renders at about 3.5 s a preview frame, 12 minutes for a
  7-second beat, and reads clearly from above.
- **Well-mixed models** (tags, image scoring, norms, social structure's networks): a small crowd
  that mills about, with each game drawn as a pair meeting. These need a new layout, first built
  for episode 4.
