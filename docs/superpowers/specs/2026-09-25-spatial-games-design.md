# SugarScape Milestone 12 — Spatial games (Nowak–May and its critics) — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–11 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds and milestone 11's conventions (portable math, named switches for unstated choices, measured descriptions).
**Sources (the user supplied the PDFs):**
- **NM92:** Nowak & May, "Evolutionary games and spatial chaos", *Nature* 359 (1992) 826–829.
- **HG93:** Huberman & Glance, "Evolutionary games and computer simulations", *PNAS* 90 (1993) 7716–7718.
- **NBM94:** Nowak, Bonhoeffer & May, "Spatial games and the maintenance of cooperation", *PNAS* 91 (1994) 4877–4881 (with a fuller account cited as *Int. J. Bifurcation Chaos* 4 (1994) 33–56, not available here).

## Goal

Add the spatial Prisoner's Dilemma as a sixth model kind covering the whole 1992–1994 exchange: Nowak and May's deterministic synchronous lattice game, Huberman and Glance's asynchronous updating, and Nowak, Bonhoeffer and May's probabilistic winning, continuous time, irregular arrays and 3D lattices — a full citizen of the playground (worker engine, Max speed, replay and links, keyframes and the timeline, stop rules, Compare, recording, Experiments, the CLI and the survey), with each paper's claims measured.

## Non-negotiable constraints

- **Earlier models unchanged.** Every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **Faithful where the papers are specific; stated where they are silent** (every silent point is listed under "Choices the papers leave open").
- **One engine path** over the `Model` trait.
- **Deterministic and portable.** A function of (config, seed); no platform `powf`/`exp`/`ln` in a rule; fingerprints identical native and WASM.
- **Truthful descriptions**, with measurements, including what does not reproduce.

## Source summary

- **NM92 rules:** payoffs "R = 1, T = b (b > 1), S = P = 0" ("none of our findings is qualitatively altered if we instead set P = ε, with ε positive but significantly below unity"); "the boundaries of the n × n matrix are fixed, so that players at the boundaries simply have fewer neighbours; the qualitative character of our results is unchanged if we instead choose periodic boundary conditions"; "the game is played with the eight neighbouring sites … and with one's own site"; "each lattice-site is occupied by the player with the highest score among the previous owner and the immediate neighbours"; purely deterministic; all sites updated each generation.
- **NM92 regimes:** "If b > 1.8, a 2 × 2 cluster of D will continue to grow … for b < 1.8, big D clusters shrink. Conversely, if b < 2, a 2 × 2 or larger cluster of C will continue to grow; for b > 2, C clusters do not grow"; for 2 > b > 1.8 "chaotically varying spatial arrays … the asymptotic overall fraction of sites occupied by C, f_C, fluctuates around 0.318 for almost all starting proportions and configurations". Fig. 1: 200 × 200, fixed boundaries, 10 % D, 200 generations; (a) 1.75 < b < 1.8 static network, equilibrium f_C "usually between 0.7 and 0.95"; "for lower b values (provided b > 5/3), D persists as line fragments … or as scattered small oscillators"; (b) 1.8 < b < 2 spatial chaos. Fig. 2a: 400 × 400, fixed, f_C(0) = 0.6, 300 generations, dashed line f_C = 12 log 2 − 8. Fig. 2b: a single D invading an infinite array of C, 2000 generations; "a crude approximation suggests … f_C = 12 log 2 − 8 = 0.318". Fig. 3: a single D at the centre of a 99 × 99 lattice of C, fixed boundaries, 1.8 < b < 2, generations 30, 217, 219, 221; "the pattern has reached the boundary (which happens at t = 49)"; "the initial symmetry is always maintained". Colour code: blue C after C, red D after D, yellow D after C, green C after D. Without self-interaction (eight neighbours): interesting region 5/3 > b > 8/5, f_C ≈ 0.299; four orthogonal neighbours: 2 > b > 5/3 with self-interaction, 5/3 > b > 4/3 without, f_C ≈ 0.374. Hexagonal arrays: qualitative only.
- **HG93:** the same 99 × 99 single-defector world, fixed boundaries; asynchronous updating: "at each step at most one individual entity is chosen at random to interact with its neighbors … the state of the rest of the system is held constant"; "at most one player is replaced by the highest-scoring player within its neighborhood"; the microstep size makes one generation equal on average to the synchronous one. Result: "Within a hundred generations or so, the array evolves into a fixed state in which all of the players are defecting. In fact, as long as there is at least one defector in the initial state, we observed that the matrix always evolved rapidly into a state of overall defection." **b is not stated.** A delayed-score variant is mentioned qualitatively only.
- **NBM94:** "the PD is played with the eight nearest neighbors and with one's own site"; n × n, fixed boundaries in the text, **Figs. 1–2 use an 80 × 80 lattice with periodic boundaries**, 200 generations. Probabilistic winning, Eq. 1: P_j = Σ_{i=1}^{v_j} A_i^m s_i / Σ_{i=1}^{v_j} A_i^m (s_i = 1 for C, 0 for D; the sum over site j and its neighbours); m → ∞ deterministic, m = 1 "proportional winning", m = 0 "random drift". Fig. 1 (discrete time) and Fig. 2 (continuous time: "In each elementary time step a cell is chosen at random, its payoff is compared to the neighbors' payoffs, and the cell is updated immediately"): rows m = ∞, 100, 20, 10, 1, 0.5, 0; columns b = 1.05, 1.13, 1.16, 1.35, 1.42, 1.55, 1.71, 1.77, 1.9, 2.01 (**the captions list nine values, omitting 1.77, which the figures show**). Claims: all C near b = 1, all D approaching 2, coexistence between, for all m and both time modes; "for deterministic winning and b between 1.8 and 2 … we find polymorphism for discrete time and all D for continuous time", so HG93's conclusion "drawn from this restricted analysis is misleading". Random arrays: a 200 × 200 lattice, ~5 % of cells occupied at random, interaction radius r = 2 … 11, deterministic winning; coexistence for intermediate b "provided r was not too big"; "for b = 1.6, r_c ~ 9"; patterns "relatively static", proportions dependent on initial configuration for small b. 3D: "the results are similar to the two-dimensional ones" (no figures). Self-interaction weight a: "polymorphisms can be maintained in the absence of self-interaction (a = 0), but only for relatively large values of m and for smaller values of b; for m = 1 (proportional winning), C cannot persist in the absence of self-interaction". Initial configuration for Figs. 1–2: **not stated**.

## Architecture

- **Model kind** `spatial` ("Spatial Games"): `ModelKind::Spatial`, `ModelConfig::Spatial(SpatialConfig)` tagged `"model": "spatial"`, `SpatialWorld` implementing `Model`, keyframes (`ModelWorld::checkpoint`/`restore`), a schema, `SERIES`, presets and golden entries. Code in `crates/sugarscape-core/src/spatial/` (`config.rs`, `geometry.rs` for the three lattices' neighbour tables, `world.rs`, `presets.rs`, `mod.rs`).
- **Shared portable math:** `anasazi::random::ln` and `civil::exp_neg` move to a crate-level `portable` module (`portable::{ln, exp_neg}`), re-exported from their old paths so nothing else changes; bit-for-bit identical results (golden entries unchanged).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `lattice` | `square` | reset | `square`, `cube` or `random` |
| `width`, `height` | 200, 200 | reset | square and random: the base lattice; cube: n × n × n with n = `width` (`height` ignored) |
| `neighborhood` | `moore` | reset | square: Moore (8) or von Neumann (4); cube: Moore (26) or von Neumann (6); ignored for random |
| `boundary` | `fixed` | reset | `fixed` or `periodic` |
| `occupancy` | 0.05 | reset | random: fraction of base cells holding a player |
| `radius` | 5 | reset | random: Euclidean interaction radius in base cells |
| `b` | 1.9 | live | temptation T |
| `epsilon` | 0 | live | P (0 ≤ ε < 1) |
| `self_weight` | 1 | live | a: weight of the payoff from playing oneself (0 = no self-interaction) |
| `update` | `synchronous` | live | `synchronous` or `asynchronous` |
| `winning` | `deterministic` | live | `deterministic` (m = ∞) or `probabilistic` |
| `m` | 1 | live | Eq. 1's exponent when probabilistic (≥ 0) |
| `start` | `random` | reset | `random` or `single_defector` |
| `defectors` | 0.1 | reset | random start: fraction of players that start D |
| `schedule` | `[]` | — | as milestone 11's (live paths only) |

## Rules

- **Neighbourhoods** (precomputed tables per site, like civil's `View`): square Moore/von Neumann; cube Moore (26) / von Neumann (6); random arrays: all other players within Euclidean distance ≤ r (on the torus with `periodic`). Fixed boundaries drop off-lattice neighbours ("players at the boundaries simply have fewer neighbours").
- **Scores:** A_i = Σ over neighbours j of π(s_i, s_j) + a · π(s_i, s_i), with π(C,C) = 1, π(D,C) = b, π(C,D) = 0, π(D,D) = ε. Computed in closed form (plan Decision 3): a cooperator with k cooperating neighbours scores k + a; a defector scores k·b + (n − k)·ε + a·ε (n its neighbours) — exact for integer k.
- **Candidates for site i:** i and its neighbours.
- **Deterministic winning:** the candidates with the highest score; if they all share one strategy, the site takes it; if both strategies are among them, the site keeps its strategy.
- **Probabilistic winning (Eq. 1):** P(C) = Σ_c A_c^m s_c / Σ_c A_c^m over the candidates; one uniform draw decides. A_c^m is computed as exp_neg(m · (ln A_c − ln A_max)) (A_max the largest candidate score; terms with A_c = 0 contribute 0 for m > 0); m = 0 gives every candidate weight 1 (0^0 = 1: the paper's "random drift"); if every weight is 0 (all scores 0 with m > 0) the site keeps its strategy.
- **Synchronous generation:** all scores from the current strategies, then every site's new strategy, then all switch at once. Probabilistic draws in site order.
- **Asynchronous generation:** N microsteps (N = players); each draws a uniformly random player (with replacement), rescores it and its neighbours from the current strategies, and updates that site immediately. The four-colour change classification compares each site with its strategy at the start of the generation.
- **Initial pattern:** `random`: exactly round(defectors × N) players chosen uniformly without replacement start D; `single_defector`: the player nearest the lattice centre ((w−1)/2, (h−1)/2[, (d−1)/2]; for even sizes the lower-left of the central cells) starts D, all others C.

## Choices the papers leave open

1. **Ties** (NM92 silent): C/D ties for the highest score occur only when both are 0 or when b sits exactly on a threshold ratio; the owner keeps its site.
2. **Eq. 1 with all-zero scores** (0/0): the site keeps its strategy. **m = 0:** 0^0 = 1.
3. **Asynchronous sampling:** with replacement (HG93's "chosen at random"); one generation = N microsteps.
4. **NBM94 Figs. 1–2 initial condition** (not stated): random, **50 % D** (plan Decision 10). Their m = 0 row is pure drift, which keeps its start, and it shows roughly even colours; a 10 % start would leave it ~90 % blue. `nbm-random-array` starts from 50 % too. The presets and sweeps say so.
5. **Random arrays:** the occupied cells are drawn once at setup (exactly round(occupancy × cells), uniformly); self-interaction included; distance measured between cell centres.
6. **3D:** no source detail; the cube uses the 26-neighbour Moore analogue with self-interaction; b and the start are chosen by measurement and stated.
7. **Hexagonal lattices:** not implemented (NM92 gives no numbers).

## Statistics

`SERIES`: `fraction_c`, `changed` (share of players whose strategy differs from the previous generation), `c_to_d`, `d_to_c` (counts), `mean_payoff_c`, `mean_payoff_d` (NaN with none of that strategy), `players`.

## Views

- **Colour modes:** **Change** (NM92's scheme: blue C→C, red D→D, yellow C→D, green D→C; the default), **Strategy** (blue C, red D), **Payoff** (low → high heat). Empty base cells (random arrays) dark.
- **Cube:** the frame is one z-slice, chosen by a **Slice** selector (the layer menu, `slice:<z>`); the run is unaffected.
- **Inspect:** the site's strategy (and previous), score, and each candidate's strategy and score with the winner marked (deterministic) or P(C) (probabilistic); cube: the cell in the current slice.
- **Charts:** **Cooperators** (`fraction_c`), **Changes** (`changed`), **Switches** (`c_to_d`, `d_to_c`; split from Changes because they are counts), **Payoffs** (`mean_payoff_c`, `mean_payoff_d`).

## Presets

| Preset | Setup | Paper |
|---|---|---|
| `nm-1a-static` | 200², fixed, 10 % D, b = 1.77 | NM92 Fig. 1a |
| `nm-1b-chaos` | as above, b = 1.9 | NM92 Fig. 1b |
| `nm-2a-universal` | 400², fixed, 40 % D, b = 1.9 | NM92 Fig. 2a |
| `nm-3-kaleidoscope` | 99², fixed, single D, b = 1.9 | NM92 Fig. 3 |
| `nm-no-self` | 200², fixed, 10 % D, a = 0, b = 1.62 | NM92 text |
| `nm-four-neighbors` | 200², fixed, von Neumann, 10 % D, b = 1.8 | NM92 text |
| `hg-async-kaleidoscope` | the kaleidoscope, asynchronous | HG93 Fig. 1 |
| `nbm-probabilistic` | 80², periodic, 50 % D, m = 1, b = 1.35 | NBM94 Fig. 1 |
| `nbm-discrete` | 80², periodic, 50 % D, deterministic, b = 1.71 | NBM94 Fig. 1 (added for the second Compare entry) |
| `nbm-continuous` | 80², periodic, 50 % D, asynchronous, b = 1.71 | NBM94 Fig. 2 |
| `nbm-random-array` | 200² at 5 %, r = 5, 50 % D, b = 1.6 | NBM94 text |
| `nbm-cube` | 30³, periodic, Moore, b = 1.6, 10 % D (by measurement) | NBM94 text |

**Compare entries:** "Synchronous vs asynchronous — Spatial Games (Compare)" (`nm-3-kaleidoscope` vs `hg-async-kaleidoscope`) and "Discrete vs continuous time — Spatial Games (Compare)" (`nbm-discrete` vs `nbm-continuous`: 80², 50 % D, b = 1.71, synchronous vs asynchronous; `nbm-discrete` was added as a preset for this entry).

## Experiments and CLI

- `nbm-grid-discrete`, `nbm-grid-continuous`: base 80² periodic 50 % D; x = b over NBM94's ten columns (including 1.77); series = m over ∞, 100, 20, 10, 1, 0.5, 0; metric final `fraction_c` at 200; seeds by measurement.
- `nm-universal`: base `nm-1b-chaos`; x = initial D fraction 0.05 … 0.95; metric window mean of `fraction_c` late in the run.
- `nbm-radius`: base `nbm-random-array`; x = r 2 … 11; series = a few b values; metric final `fraction_c`.
- `sugarscape presets | run | sweep` accept `spatial`.

## Claims to test (survey and book-style tests, 20 seeds; exact where deterministic)

- NM92: the thresholds (a 2 × 2 D cluster grows for b > 1.8 and shrinks below; a 2 × 2 C cluster grows for b < 2 and not above) — exact, on hand-built worlds; f_C ≈ 0.318 (= 12 ln 2 − 8) in 1.8 < b < 2 for almost all starting proportions; Fig. 1a's f_C between 0.7 and 0.95; the kaleidoscope's four-fold symmetry at every generation and its reaching the boundary at t = 49 — exact; f_C ≈ 0.299 without self-interaction and ≈ 0.374 with four neighbours.
- HG93: asynchronous updating ends in all D whenever a D is present — tested across b, not only at their unstated value.
- NBM94: the grids' regimes in both time modes; asynchronous updating removes only the 1.8 < b < 2 chaos; C cannot persist at m = 1 without self-interaction; random arrays keep C and D together up to r_c ≈ 9 at b = 1.6.

## Page

- A **Spatial Games** presets group; the schema panel in groups **Game** (b, ε, a), **Lattice** (geometry, size, neighbourhood, edges, occupancy and radius shown only for random arrays), **Update** (update, winning, m shown only when probabilistic), **Start** (start, defectors shown only for random); the three colour modes; the Slice selector for cubes; the four charts; Inspect; the two Compare entries; `defaultForm('spatial')` (x = b, final `fraction_c`).
- Keyframes, the timeline, stop rules, share links, sessions, recording and Compare work unchanged (asynchronous and probabilistic runs draw from the world's seeded RNG and replay exactly).

## Testing

- **Core unit:** scores (C with k cooperating candidates scores k; D scores b·k; a = 0 drops the self term; ε), neighbourhoods (fixed edges fewer neighbours; periodic wrap in 2D and 3D; random-array radius), deterministic winner and the tie rule, Eq. 1 (m = 0 drift, 0/0 keeps, large m approaches deterministic), asynchronous one-site microsteps rescoring from current state, initial counts and the single defector's site, the four-colour classification, portable A^m against `powf`.
- **Exact:** the kaleidoscope's symmetry at t = 30, 217, 219, 221 and the boundary at t = 49; the 2 × 2 cluster thresholds.
- **Golden:** new entries for every preset; earlier entries untouched; WASM equal to native.
- **Book-style (`#[ignore]`, release, 20 seeds):** the claims above, thresholds measured.
- **Web (Vitest):** schema fields and `show_if`; the Slice selector; charts; Inspect rows; the Compare entries; `defaultForm('spatial')`.
- **Browser (controller):** every preset, the cube slice, both Compare entries, the sweeps, Max speed at 400².

## Docs

README: a Spatial Games section in the civil-violence style — the rules, the choices above, what reproduces and what does not, the Compare entries and sweeps — crediting all three papers. Roadmap: Milestone 12 done; the Nowak–May entry marked done.
