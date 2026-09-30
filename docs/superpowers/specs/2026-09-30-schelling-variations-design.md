# Design: variations on Schelling (milestone 32)

**Date:** 2026-09-30
**Status:** proposed.
**Why:** episode 3 of Following the Crowd, "Variations on Schelling": what later researchers changed
in his checkerboard, and what each change does. Each variation keeps its authors' own rules as a
preset, with every difference from Schelling's a named switch on the `schelling` (and `line`) model.
Sources in `papers/schelling/`; rules and claims in `2026-09-29-schelling-reading-notes.md` §7, and
below for Zhang's JEBO paper.

## The variations

| Paper | Their rules (what differs from Schelling's) | Claims to test |
|---|---|---|
| Epstein & Axtell 1996 | four neighbors, torus, a random acceptable square, random order (already switches) | (episode 1's survey) |
| Pancs & Vriend 2007 | one agent at a time **best-responds** over every empty square, **content agents move too** (no inertia), ties at random; utilities over the unlike share: *flat* (Schelling's), *p50*, *p100* (single-peaked at half), *spiked*; empty neighborhoods least preferred; bounded, Moore; 5 × 5 with 10 + 10 and 100 × 100 with 40 % each; a **ring** in 1-D | 5 × 5: random boards average 7.82 clusters; after 100 000 periods flat 2.10 (91 % complete segregation), p50 2.04 (98 %), p100 4.99; the ring (10 + 10, four each side): every utility ends completely segregated |
| Gauvin, Vannimenus & Nadal 2009 | a **random agent, content or not**, moves to a random vacancy where it would be content; bounded, Moore; tolerance T the most unlike share allowed; vacancy ρ | a frozen phase (T below about 3/8–1/2 at low vacancy), a segregated one up to T about 3/4, and a mixed one above; abrupt below ρ ≈ 26 % |
| Singh, Vainchtein & Weiss 2009 | torus, Moore, an **absolute** threshold (T of the 8 squares alike, blanks counting against: a demand table), a **deleted-checkerboard** start, board sizes 8–200 | at T = 3 the "striking global aggregation … is strictly a small city phenomenon": two clusters on 8 × 8, 22 to 55 clusters on 100 × 100 as vacancy goes 24 % → 33 %; T = 4 aggregates city-wide at low vacancy |
| Zhang 2004 (JEBO) | **no vacancies**; each period a random pair from different neighborhoods **swaps** with logit probability on their summed utility (β); a **tent** utility: rising to a peak at half alike, falling to 0.6 at all alike (his Table 4); torus; checkerboard or random start | from a checkerboard (100 × 100, Moore, β = 10) segregation emerges and persists; for β < 2 never below his cutoff (black–white pairs under 600); eight neighbors segregate in under half the time of four, twelve faster still |

Left for later: Zhang's JMS rents model (prices and asymmetric preferences, a separate model's worth
of rules) and Bruch & Mare (waiting on van de Rijt, Siegel & Macy 2009).

## Engine changes (`schelling`, `line`)

New switches, each defaulting to Schelling's so existing presets and goldens are unchanged:

- `movers`: `discontent` (Schelling, Epstein & Axtell) | `anyone` (Gauvin et al.; Pancs & Vriend's
  "no inertia").
- `movement`: adds `best` (Pancs & Vriend: the empty square of highest utility, staying included,
  ties at random) and `swap` (Zhang: no empty squares needed; a random pair of occupied squares not
  neighbors, swapping with probability e^{βV}/(e^{βU}+e^{βV}), U and V their summed utilities
  before and after).
- `utility`: `flat` | `p50` | `p100` | `spiked` (Pancs & Vriend, over the unlike share, empty
  neighborhoods lowest) | `tent` (Zhang, over like neighbors of n, his Table 4), with `beta`.
- `start`: `random` | `checkerboard` (Zhang) | `deleted_checkerboard` (Singh et al.: a checkerboard
  with the vacancies removed at random, half from each color).
- Statistics: `clusters` (like-colored groups, 4-adjacent; Pancs & Vriend count through uncontended
  blanks, Singh et al. their own way: each claim names its measure), Gauvin et al.'s segregation
  coefficient s = 2Σn_c²/(L²(1−ρ))², and `mixed_pairs` (Zhang's ρ: neighboring pairs of different
  colors).
- `line`: `edges: ends | ring` and `movement: best` with the same utilities (Pancs & Vriend's 1-D).

Presets: `pv-flat`, `pv-p50`, `pv-p100`, `pv-spiked` (5 × 5), `pv-ring`, `gvn-frozen`,
`gvn-segregated`, `gvn-mixed`, `svw-small`, `svw-large`, `svw-t4`, `zhang-checkerboard`,
`zhang-random`; sweeps for the phase diagram (T × ρ), city size, β and neighborhood size.

## Beyond the engine

The survey claims above (decision rules from each paper's numbers, fixed before running), the page
(the new switches in the Rules panel, a Clusters chart), goldens, docs. Episode 3's spike and
storyboard follow the build.
