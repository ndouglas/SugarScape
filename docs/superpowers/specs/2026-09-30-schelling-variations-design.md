# Design: variations on Schelling (milestone 32)

**Date:** 2026-09-30
**Status:** built (milestone 32); amendments below.
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
| Gauvin, Vannimenus & Nadal 2009 | a **random agent, content or not**, moves to a random vacancy where it would be content; bounded, Moore; tolerance T the most unlike share allowed; vacancy ρ | a frozen phase (T below 1/2 at 2–4 % vacant, 2/5–1/2 at 6 %), a segregated one up to T about 3/4, and a mixed one above; abrupt below ρ ≈ 26 % |
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

## Amendments (as built)

- **Each paper's own cluster count.** `clusters` (side by side) is ours; `pv_clusters` is Pancs &
  Vriend's (§4.2.1: agents joined side by side or through a zone of blanks bordered by one color
  only), and `clusters8` is Singh et al.'s N_C (joined at a side or a corner). On the side-by-side
  count Pancs & Vriend's boards looked less segregated than they report (flat 2.8, p50 2.6); on
  their own count they match (2.12 and 2.02 against 2.10 and 2.04).
- **Mixed pairs on Moore's eight.** Zhang measures his potential on the Moore neighborhood whatever
  the agents' neighborhood, so `mixed_pairs` always counts the eight around. A test pins his §2.2
  identity: what two traders gain together is 0.6/8 for each mixed pair their trade removes.
- **The ring's groups.** On `edges: ring` a run across the join counts once.
- **Goldens.** The 50 × 50 and 100 × 100 presets are fingerprinted after 20 ticks (`BIG_GOLDEN`),
  as image scoring's island presets are, to keep the debug test run short.
- **Zhang's cutoff of 600.** As mixed pairs (his ρ), 600 is exactly the least a 100 × 100 Moore
  torus split in half can have (two straight bands, 300 pairs each), so "below 600" cannot be
  literal; our runs round into blobs near 1,200–2,000. The claims read it as his scaled potential
  (0.075 ρ, so 8,000 pairs), reached in about 100,000 draws at β = 10, some 400 times sooner than
  his Fig. 8.
  His trades are a conserved Ising model at 0.0375 β per bond, whose ordering transition lies
  near β ≈ 5, not 2. The claims report both readings, and the ordering of neighborhoods is tested on
  the scaled one.
- **Pancs & Vriend p100** is tested over 500 seeds (a single run's count ranges from 2 to 10), with
  their share of strict equilibria; spiked, which their footnote 23 calls "very similar" to p100,
  is a claim of its own (it isn't: 7.0 clusters against 4.7). Empty neighborhoods score −1, strictly
  lowest, as their footnote 13 says; their formula would score them 0, tied with an unacceptable
  neighborhood (no measurable effect).
- **Gauvin et al.'s move rule** reads two ways: a random vacancy among those that suit (`random`,
  the default) or one vacancy tried at random, moving back if it doesn't suit (`try`). The frozen
  line is the same under both, and within one step of T of their Table 1.
- **Not built:** Singh et al.'s T = 5 swaps and their scale L, and their start's two permuted 3 × 3
  blocks (and with an odd number of blanks the deleted checkerboard's colors differ by one);
  Pancs & Vriend's 100 × 100 runs.
