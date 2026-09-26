# SugarScape Milestone 22 — Relative Agreement and Extremism (Deffuant et al.) — Design

**Date:** 2026-09-26
**Builds on:** the milestone 1–21 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, milestone 17's `opinions` (whose cluster counting and canvas helpers this model reuses) and the literal-default-plus-named-switch pattern of milestones 11–20.
**Source texts** (local copies in `papers/bounded-confidence/`):
- Guillaume Deffuant, David Neau, Frédéric Amblard and Gérard Weisbuch, "Mixing Beliefs Among Interacting Agents", *Advances in Complex Systems* 3 (2000), 87–98 (DNAW below), with the sample program `melsimp.c` published alongside the 2002 paper (jasss.org/5/4/1/melsimp.html).
- Guillaume Deffuant, Frédéric Amblard, Gérard Weisbuch and Thierry Faure, "How Can Extremism Prevail? A Study Based on the Relative Agreement Interaction Model", *JASSS* 5(4) 1 (2002) (DAWF below). The equations are images on the JASSS page; they were read from jasss.org/5/4/1/eq*.gif and eq1.jpg.
- Frédéric Amblard and Guillaume Deffuant, "The Role of Network Topology on Extremism Propagation with the Relative Agreement Opinion Dynamics", *Physica A* 343 (2004), 725–738 (arXiv cond-mat/0404574; AD below).
- Gérard Weisbuch, "Bounded Confidence and Social Networks", *European Physical Journal B* 38 (2004), 339–343 (arXiv cond-mat/0311279; W below).

**Replication and reply:** Michael Meadows and Dave Cliff, "Reexamining the Relative Agreement Model of Opinion Dynamics", *JASSS* 15(4) 4 (2012) (M&C below); Guillaume Deffuant, Frédéric Amblard and Gérard Weisbuch, "The Results of Meadows and Cliff Are Wrong Because They Compute Indicator y Before Model Convergence", *JASSS* 16(1) 11 (2013) (DAW below).

Image scoring holds Milestone 21 in its worktree, so this is Milestone 22.

## Goal

The relative agreement model and the pairwise bounded-confidence models it extends as a model kind `agreement` ("Relative Agreement"), a full citizen of the playground: DAWF's extremist regimes, Fig. 9 map and §6 variants, DNAW's pairwise model and lattice, AD's lattice and small worlds, and W's scale-free networks as presets and sweeps; M&C's and DAW's readings, the unstated placement, cutoff, horizon and pair-update details, and eq. 11's printed window as named switches; every claim measured over many seeds.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before. `opinions` keeps its behavior; shared helpers move only if their outputs are unchanged.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical).
- **Truthful descriptions:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **Pairwise BC (DNAW §2.1):** "At each time step any two randomly chosen agents meet. They re-adjust their opinion when their difference of opinion is smaller in magnitude than a threshold d … x = x + μ·(x′ − x), x′ = x′ + μ·(x − x′)", μ in (0, 0.5], opinions uniform on [0, 1]. Figs. 1–2: N 1000 (Fig. 1's caption says 2000), d 0.5 and 0.2, μ 0.5, "one time unit corresponds to sampling 1000 pairs". Fig. 4 (250 samples, μ 0.5, N 1000, "wings are excluded"): the number of peaks falls with d; a rough bound gives about 1/(2d) peaks. `melsimp.c`: pairs drawn independently (an agent can meet itself), `cardis < d*d` (strict), both updated from the old values.
- **DNAW lattice (§3):** a 29 × 29 square lattice, four neighbors, "a pair is randomly selected among connected agents". Fig. 5 (d 0.3, μ 0.3, "after 100 000 iterations"): "a large majority of agents which have reached consensus … apart from isolated agents which have 'extremists' opinions closer to 0 or 1". Fig. 6 (d 0.15): a percolating cluster and "smaller non-percolating clusters with similar but not equal opinions".
- **RA (DAWF §2, eqs. 1–6):** opinion xᵢ, uncertainty uᵢ, opinions uniform on [−1, 1]. hᵢⱼ = min(xᵢ + uᵢ, xⱼ + uⱼ) − max(xᵢ − uᵢ, xⱼ − uⱼ); "If hᵢⱼ > uᵢ": xⱼ := xⱼ + μ(hᵢⱼ/uᵢ − 1)(xᵢ − xⱼ), uⱼ := uⱼ + μ(hᵢⱼ/uᵢ − 1)(uᵢ − uⱼ); "If hᵢⱼ ≤ uᵢ, there is no influence of i on j." "After agents i and j opinion and uncertainty updating, a new pair is randomly chosen and the same process is iterated until attractors of the dynamics with invariant opinions and uncertainties are reached." Fig. 4 (50 runs per point): clusters "close to w/2u (r² = 0.99), whereas it is close to the integer part of w/2u in the BC model".
- **Extremists (§3):** ue for extremists, U for moderates, pe the proportion, δ = |p₊ − p₋|/(p₊ + p₋). "We first randomly draw opinions of our population from a uniform distribution. Then we initialise the Np₊ most positive opinions and Np₋ most negative opinions with the ue uncertainty". Figs. 5–8 (μ 0.5, δ 0, ue 0.1, N 200): central (pe 0.2, U 0.4, 4 % join), both extremes (pe 0.25, U 1.2; 43 % and 56 %), single extreme (pe 0.1, U 1.4, 98.33 %), and central again for another sample at Fig. 7's parameters.
- **Indicator (§4):** "the proportions p′₊ and p′₋ of the initially moderate agents which became extremists", y = p′₊² + p′₋²; 0 central, 0.5 both extremes, 1 single extreme. "Which became extremists" is not defined, nor when y is computed beyond "after convergence".
- **Fig. 9:** mean and s.d. of y, 50 runs of 1000 agents per point, U 0.2–2 by 0.1, pe 0.025–0.3 by 0.0125, ue 0.1, μ 0.2, δ 0 and 0.1: two central zones (left, and a diagonal from the lower middle), a both-extremes triangle, single extreme at the bottom right. Fig. 10 (pe 0.125): unimodal y between 0 and 0.5 at small U, both extremes at 0.5 < U < 1, bimodal central-or-single at U > 1. §4.8 (10 runs per point, 84 000 runs): larger μ widens both-extremes and narrows single; larger δ widens single; ue has no significant effect.
- **BC variants (§6):** eq. 11 "If |x − x′| < u′ the influence of x′ on x is given by x = x + μ(x′ − x)", with "u′ the uncertainty of opinion x′"; eq. 12 adds u = u + μ(u′ − u); eqs. 13–14 (after Weisbuch et al. 2002) x = αx + (1 − α)x′, u² = αu² + α(1 − α)(x − x′)², condition not given. Claims: plain BC gives both extremes only near U 0.4–0.5, single extreme around U = 1 ("larger for delta = 0.1"), and only central above U 1.2 with the cluster fluctuating; averaging gives both extremes near 0.4, a central band near 0.8 and single or central above 1.1 (single when δ > 0); the variance rule gives no single extreme and very rare both extremes.
- **AD:** extremists placed differently: "we first randomly draw opinions of (1 − pe)·N agents … Then we initialize Np₊ agents to +1 and Np₋ most negative opinions to −1". On a Moore torus (Fig. 3, ue 0.1, μ 0.2, δ 0) "y is always below 0.6 … the single extreme convergence never occurs". Small worlds (Figs. 4–5, U 1.8, ue 0.1, N 1000, μ 0.1, δ 0, pe 0.05, 50 runs, k = 2 … 256 by powers of 2, p 0 … 1): "a transition from double extreme convergence to single extreme convergence case when the connectivity (k) increases … the transition takes place for higher connectivity when p decreases"; at β = 0.8 "the phase transition … occurs for values of connectivity around 8". Fig. 6: the same on a grid substrate with a generalized Moore neighborhood. Fig. 7: (U, pe) = (1.0, 0.05), (1.2, 0.05), (1.4, 0.05).
- **W:** DNAW's model on Barabási–Albert networks of 900 nodes grown from a triangle with two links per new node (mean degree 4; 8 for Fig. 3's second curve), "a random node is first chosen, and then one of its neighbours. But only the first node in the pair might update". The dispersion index Σsᵢ²/(Σsᵢ)² (Derrida and Flyvbjerg; W calls it y). Claims: well mixed, "two distinct steps at y = 0.5 and y = 0.33"; scale-free, "a continuous increase … with only a kink in the d = 0.25, y = 0.7 region"; the scale-free and lattice curves are similar; "Increasing the average connectivity by a factor 2 brings the scale free network results closer to those of the well-mixed case"; well-connected nodes end in the big cluster; many poorly connected nodes never move ("outlying").
- **M&C:** Java and Python reimplementations: moderates uniform on (−0.8, 0.8), extremists uniform in [0.8, 1] and [−1, −0.8], the extremist count rounded up to even, N 200, μ 0.2, 40 000 pair meetings (200 per agent), both agents updated from their old values, a moderate counted as extremist when beyond ±0.8. Result: mean y peaks near 0.5; "There appear to be no conditions under which single extreme convergence will occur in the majority of the simulations"; the single-extreme zone shrinks as N grows (§5.3).
- **DAW:** M&C "compute indicator y before model convergence". With 240 000 meetings (1200 per agent) and new extremists counted beyond ±0.7 ("a threshold … lower of 0.1 than the threshold for initial extremists"), M&C's own program gives "single extreme convergence … very frequent (often more than 80% of the simulations) for low values of pe and large values of U"; with N 1000 at 4000 meetings per agent it "is still significantly present", and "takes place with any large number of agents".

## Measured in planning

A throwaway prototype of the rules below (it gave an odd extremist count's extra agent to the positive side, a lean of 1/nₑ at δ 0 that the coin flip below removes; the survey re-measures); 50 seeds unless stated, N 1000 for Fig. 9 points, μ 0.2 and ue 0.1 unless stated. The survey reproduces each.

- **M&C against DAW** (N 200, pe 0.05, U 1.4, 100 seeds): M&C's reading (band placement, cutoff 0.8, 200 periods) y 0.01, single extreme 1; 1200 periods with cutoff 0.8, y 0.48; 200 periods with cutoff 0.7, y 0.16; DAW's reading (1200 periods, cutoff 0.7) y 0.94, single 94. Both fixes are needed: the drifted majority settles just inside 0.8. The literal reading (drawn placement, margin 0.1, run to stability) gives y 0.88 at N 200, stable by a median of period 280.
- **Fig. 9, literal reading, δ 0:** the layout reproduces — y ≈ 0.02–0.18 at U ≤ 0.4; the both-extremes triangle (y 0.49–0.51 at U 0.8–1.2 for pe ≥ 0.1, widening upward); the central diagonal (y ≤ 0.1 at pe 0.025 U 0.8–1.0, pe 0.1 U 1.2–1.4, pe 0.15 U 1.4–2.0); single extreme bottom right (y 0.98–1.00 at pe 0.025, U ≥ 1.2; 0.78–0.80 at pe 0.05, U ≥ 1.4). It fades faster in pe than the figure: at pe 0.1 and 0.125, U ≥ 1.4, y 0.08–0.20 and single extreme in 4–11 of 50 runs, where Fig. 10 shows many runs near y = 1.
- **δ 0.1:** single extreme y ≥ 0.93 for U ≥ 1.6 up to pe 0.125, 0.52–0.69 at pe 0.25 — Fig. 9's bottom panel.
- **Fig. 10 (pe 0.125):** unimodal around 0.1–0.3 at U 0.4–0.6, all at 0.5 at U 0.8–1.0, bimodal from U 1.2 (at 1.2 central 12, both 14–25, single 0; from 1.4 central 45–46, single 4–5).
- **Population size (δ 0, U 1.6, 100 seeds):** single extreme at pe 0.05: 99, 90, 88, 78, 75, 57 runs for N 100, 200, 400, 1000, 2000, 4000; at pe 0.1: 79, 62, 50, 16, 6, 0; at pe 0.15: 67, 50, 33, 9, 3, 0. With δ = 0, single extreme is a finite-size effect that vanishes with N except at the smallest pe — M&C's §5.3 holds, DAW's "any large number of agents" holds only at pe ≈ 0.05. N 200 in the crossover (pe 0.1, U 1.4) gives y 0.47, s.d. 0.49 — Fig. 9's dark-blue band — where the stated N 1000 gives 0.10.
- **Placement and update order** (N 1000, pe 0.075–0.125, U 1.4–1.8, 100 seeds): band placement raises single extreme (pe 0.075: 81 against 31 drawn, 29 at ±1); sequential updating lowers it slightly. None changes the layout.
- **Eq. 11's window:** as printed (|x − x′| < u′, the influencer's uncertainty) plain BC and BC with averaging give y = 0.00 everywhere (U 0.2–1.8, pe 0.05–0.3, δ 0 and 0.1): the uncertain moderates reach the confident extremists and pull them in. With the listener's uncertainty (|x − x′| < u, the analogue of RA's hᵢⱼ > uᵢ when uᵢ < uⱼ) plain BC gives single extreme in 50 of 50 at U 1.0 (δ 0) and U 1.0–1.2 (δ 0.1), both extremes only at U 0.4–0.6, and central from U 1.2 with runs that never go still (20 000 periods); BC with averaging gives a central band at U 0.8, both extremes near 0.5–0.6, and single extreme above U 1.1 (δ 0.1: 0.68–1.00; δ 0: 0.30–0.48). §6's text is reproduced by the listener's reading only.
- **Variance rule** (α 0.8): y ≤ 0.14 everywhere under either window; single extreme in at most 5 of 50 runs — as §6 says.
- **Clusters (pe 0):** RA gives 1.0, 2.0, 2.7, 3.4, 4.1, 5.3, 6.9, 10.6 major clusters at w/2u 1.25, 1.67–2, 2.5, 3.33, 4, 5, 6.67, 10 (N 1000, μ 0.5; clusters of at least 1 % of agents): about w/2u, rounded up. BC gives 1.0, 1.0, 2.0, 2.0, 2.9, 3.4, 4.4, 6.0, 8.8: the integer part up to w/2u ≈ 3, then below it. Counting every non-isolated cluster, BC gives about w/2u (2.98 at 2.5). "Integer part" depends on excluding small clusters.
- **Moore torus** (30 × 30, pe 0.05–0.3, U 0.4–1.8): y ≤ 0.50 and no single extreme in any run; many runs not still within 20 000 periods. With extremists at ±1 and margin 0.1, y ≈ 0.
- **Small worlds** (N 1000, U 1.8, pe 0.05, μ 0.1, extremists at ±1, 30 seeds): single extreme appears from k 32 at p 0.2 (11 of 30), k 16–32 at p 0.8 (16–24 of 30 drawn; 12 at k 32 with ±1) and p 1, reaching 28–30 of 30 at k 128–256 — a critical k that falls as p rises, as AD say, though above their "around 8". Below it the outcome depends on the cutoff: with margin 0.1 (cutoff 0.9) y ≈ 0 (central); with margin 0.3 (cutoff 0.7) y 0.48–0.49 at k 2–4 (both extremes, AD's reading). The local clusters settle between 0.7 and 0.9.
- **Scale-free (W, d on [0, 1], μ 0.5, 100 seeds):** dispersion well mixed 0.24, 0.36, 0.50, 0.50, 0.53, 0.83, 0.99 at d 0.10, 0.15, 0.20, 0.22, 0.25, 0.28, 0.30 (steps at ⅓ and ½); scale-free (m 2, node pairing, one-way) 0.04, 0.19, 0.34, 0.42, 0.58, 0.77, 0.85 — no steps, a rise through 0.6–0.8 at d 0.25–0.28; 30 × 30 von Neumann lattice 0.01, 0.02, 0.26, 0.44, 0.68, 0.80, 0.86, close to it.
- **DNAW lattice** (29 × 29, μ 0.3, 20 seeds): Fig. 5's "100 000 iterations" are 119 meetings per agent; there, at d 0.3, the largest cluster (gaps ≤ 10⁻³) holds 34 %; run to stability (median period 1 848) it holds 92 %, with 27 isolated agents — the figure's picture. At d 0.15 many clusters of similar opinions remain (largest group within gaps of 0.02: 35 % at 119 periods, 24 % at stability).
- **Speed:** 50 runs of N 1000 to stability take under half a second natively on 10 cores; the survey's full Fig. 9 grid (437 points × 50 × 2) is minutes.

## Architecture

Model kind `agreement` ("Relative Agreement"): `ModelKind::Agreement`, `ModelConfig::Agreement(AgreementConfig)` tagged `"model": "agreement"`, an `AgreementWorld` implementing `Model`, schema, `SERIES`, presets and golden entries — the same wiring as the other models. Code in `crates/sugarscape-core/src/agreement/` (`config.rs`, `world.rs` for the meetings, `network.rs` for the graphs, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`). It reuses `opinions`' cluster helper (generalized to take a gap, with `opinions` passing its own) and its canvas helpers.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `agents` | 200 | reset | N (2–4000); ignored under `lattice` and the grid substrate (width × height) |
| `rule` | `ra` | live | `ra` (eqs. 1–6), `bc` (eq. 11; DNAW), `bc_averaging` (eqs. 11–12), `bc_variance` (eqs. 13–14) |
| `window` | `influencer` | live | the BC rules' condition: `influencer` (eq. 11 as printed: \|x − x′\| < u′) or `listener` (\|x − x′\| < u). `bc_variance` reads it too (the source gives no condition) |
| `mu` | 0.2 | live | μ (0–1) |
| `alpha` | 0.8 | live | α (0–1) under `bc_variance` (unstated in DAWF; 0.8 = 1 − μ) |
| `uncertainty` | 1.0 | reset | U, the moderates' starting uncertainty (0–4) |
| `extremists` | 0.1 | reset | pe (0–1) |
| `extremist_uncertainty` | 0.1 | reset | ue |
| `delta` | 0 | reset | δ (0–1) |
| `placement` | `drawn` | reset | `drawn` (DAWF: N uniform draws on [−1, 1], the most extreme take ue), `bounds` (AD: the same agents, set to ±1), `band` (M&C: extremists uniform in [b, 1] and [−1, −b], moderates uniform on (−b, b), at random positions) |
| `band` | 0.8 | reset | b under `band` |
| `extreme_margin` | 0.1 | live | a moderate counts as a new extremist when beyond the side's boundary minus this; the boundary is the innermost initial extremist's opinion (`drawn`), 1 (`bounds`) or b (`band`). 0.1 is DAW; 0 with `band` is M&C |
| `pair_update` | `simultaneous` | live | `simultaneous` (both from the old values: `melsimp.c`, M&C), `sequential` (i acts on j, then the new j on i), `one_way` (only the first agent updates: W) |
| `network` | `all` | reset | `all`, `lattice`, `small_world`, `scale_free` |
| `lattice.width`, `lattice.height` | 29, 29 | reset | 3–64 each |
| `lattice.neighborhood` | `von_neumann` | reset | `von_neumann` or `moore` |
| `small_world.substrate` | `ring` | reset | `ring` (k/2 each side) or `grid` (the lattice's size, a generalized Moore neighborhood of radius r: k = (2r + 1)² − 1) |
| `small_world.degree` | 8 | reset | k (even, 2–256, below N) |
| `small_world.rewire` | 0.1 | reset | p (0–1) |
| `scale_free.links` | 2 | reset | m per new node (mean degree 2m) |
| `pairing` | `edge` | live | on a network: `edge` (a uniform link, its order uniform: DNAW, AD) or `node` (a uniform agent, then a uniform neighbor: W) |
| `stop_when_stable` | true | live | `finished()` at the first stable period |
| `stop_at` | 20 000 | live | `finished()` at this period (0: never) — the cap under stability, the horizon otherwise |

The Rules panel shows `window` only for the BC rules, `alpha` only under `bc_variance`, the extremist fields only when `extremists` > 0, `band` only under `band`, and each network's fields only with it.

**DNAW's [0, 1] scale:** the dynamics are invariant under x ↦ 2x − 1 with u = 2d and μ unchanged, so DNAW and W run on [−1, 1] with `uncertainty` = 2d; descriptions give d.

## Setup

- **Extremist counts:** nₑ = round(N·pe); n₊ = round(nₑ·(1 + δ)/2), n₋ = nₑ − n₊; a tie (nₑ odd at δ 0) goes to a side chosen by a coin flip, so δ 0 has no built-in lean. (M&C round nₑ up to even; the odd extremist is a real asymmetry at small N, M&C Fig. 13.)
- **`drawn`:** N opinions uniform on [−1, 1); sorted, the lowest n₋ and highest n₊ get ue, the rest U. **`bounds`:** as `drawn`, then those agents' opinions are set to −1 and +1. **`band`:** a random permutation assigns n₊, n₋ and the moderates; draws as above.
- **Networks** are built from the seed before opinions. Watts–Strogatz: each substrate link (i, j) in index order is, with probability p, replaced by (i, c) for a uniform c that is not i and not already linked to i. Barabási–Albert: a triangle, then each new node links to m distinct existing nodes chosen with probability ∝ degree. A node with no neighbor is skipped when drawn under `node` pairing.

## Step (one meeting) and period

A tick is one **period** of N meetings (DAWF's "average number of iterations per agent"; DNAW's time unit). Each meeting draws (i, j): under `all`, j uniform and i uniform among the others (M&C's order; `melsimp.c`'s self-meetings are left out: they only rescale time); otherwise by `pairing`. Then:
- **`ra`:** i acts on j when hᵢⱼ > uᵢ, j on i when hⱼᵢ > uⱼ, by eqs. 5–6.
- **`bc`:** j moves by μ(xᵢ − xⱼ) when |xᵢ − xⱼ| < uᵢ (`influencer`) or < uⱼ (`listener`); `bc_averaging` also moves uⱼ by μ(uᵢ − uⱼ); `bc_variance` sets xⱼ = αxⱼ + (1 − α)xᵢ and uⱼ = √(αuⱼ² + α(1 − α)(xⱼ − xᵢ)²).
- **`pair_update`** decides whether both act from the old values, in sequence, or only i updates (j acting on i).

The comparisons are exact on f64. Random draws follow this order, so runs are deterministic.

A period is **stable** when no opinion or uncertainty moved more than 10⁻⁶ in it (planning: 10⁻⁴, 10⁻⁶ and 10⁻⁸ give the same y; at stability y equals y at period 5 000 for every reading tested).

## Statistics

`SERIES`:
- `y` (DAWF's indicator), `p_plus`, `p_minus` (shares of the initial moderates that became extremists), `outcome` (0 central: both below 0.15; 1 both extremes: both at least 0.25; 2 single extreme: one at least 0.7 and the other below 0.1; 3 intermediate) — y alone misreads p′₊ 0.7, p′₋ 0 (M&C ¶4.10);
- `clusters` (groups of sorted opinions with gaps ≤ 10⁻³ and at least two agents), `major` (groups holding at least 1 % of agents; DNAW's wings excluded), `isolated` (groups of one), `largest`, `second` (shares);
- `dispersion` (W's Σsᵢ²/N² over all groups);
- `unmoved` (share of agents never moved; W's outlying nodes);
- `mean_opinion`, `mean_uncertainty`, `max_change`, `stable_at`.

## Views

- **Opinion × time** (left): 241 × 201 cells, +1 at the top. The history keeps at most 241 columns: when full, every other kept period is dropped and later periods are kept at the doubled interval, so a run of any length stays in view. The history is world state: keyframes and step-back restore it.
- **Start vs now** (right, 8-cell gap): 201 × 201, each agent a dot at (start, current opinion); the diagonal marks agents that never moved (DNAW Fig. 3, W Figs. 4–5).
- **Torus** (further right, 8-cell gap) under `lattice` and the grid substrate: one cell per site, colored by current opinion in every mode.
- **Color modes:** **Uncertainty** (default; confident to uncertain, as DAWF's figures), **Role** (initial extremists by side; moderates by current opinion), **Start** (by starting opinion).
- **Inspect:** a column, dot or site — the agents there, each with its id, role, start, opinion and uncertainty (in that period), degree, meetings and moves. `agent` stays null and `locate` returns nothing, as in `opinions`: the page would otherwise re-read a tracked agent at its dot, where neighbors crowd it (amended in planning).
- **Charts:** Convergence (`y`, `p_plus`, `p_minus`); Clusters (`clusters`, `major`, `isolated`); Dispersion (`dispersion`, `unmoved`); Opinion and uncertainty (`mean_opinion`, `mean_uncertainty`); Change (`max_change`). Time axis: Periods.

## Presets

| Preset | Setup | Source |
|---|---|---|
| `dnaw-consensus`, `dnaw-clusters` | bc, pe 0, μ 0.5, N 1000, u 1.0 and 0.4 (d 0.5, 0.2) | DNAW Figs. 1–2 |
| `dnaw-lattice`, `dnaw-lattice-clusters` | bc, pe 0, 29 × 29 von Neumann, μ 0.3, u 0.6 and 0.3 (d 0.3, 0.15) | DNAW Figs. 5–6 |
| `ra-uniform` | ra, pe 0, u 0.4, N 200, μ 0.5 | DAWF Fig. 3 |
| `ra-central`, `ra-both`, `ra-single` | ra, μ 0.5, N 200: pe 0.2 U 0.4; pe 0.25 U 1.2; pe 0.1 U 1.4 | Figs. 5–7 |
| `ra-literal` | ra, N 200, μ 0.2, pe 0.05, U 1.4 | Fig. 9's μ and ue at M&C's N 200, the literal reading |
| `ra-meadows-cliff` | `ra-literal` with band, margin 0, stop at 200, not at stability | M&C |
| `ra-deffuant-2013` | `ra-literal` with band, margin 0.1, stop at 1200, not at stability | DAW |
| `ra-bc-extremists` | bc, listener, N 1000, pe 0.05, U 1.0 | Fig. 20 |
| `ra-bc-printed` | `ra-bc-extremists` with the printed window | eq. 11 |
| `ad-moore` | ra, 30 × 30 Moore, μ 0.2, pe 0.2, U 1.4, bounds | AD Fig. 3c |
| `ad-small-world` | ra, ring, k 32, p 0.8, N 1000, μ 0.1, U 1.8, pe 0.05, bounds | AD Figs. 4–5 |
| `w-scale-free` | bc, pe 0, BA m 2, N 900, node pairing, one-way, μ 0.5, u 0.4 (d 0.2) | W Fig. 4 |

**Compare entry:** "Meadows and Cliff vs Deffuant et al.'s reply — Relative Agreement (Compare)": `ra-meadows-cliff` and `ra-deffuant-2013`.

## Experiments and CLI

Seeds and ranges measured to fit a browser run and recorded in each description; the survey runs the papers' counts.
- `ra-clusters`: final `major` against w/2u (u 1.0 … 0.1), series `ra`, `bc` (pe 0; DAWF Fig. 4).
- `ra-map`: final `y` against U 0.2 … 2, series pe 0.025, 0.05, 0.1, 0.2 (Fig. 9, N 200 for the browser; the survey runs N 1000).
- `ra-readings`: final `y` against U at pe 0.05, series M&C, horizon only, cutoff only, DAW, literal.
- `ra-population`: final `y` against N 100 … 2000 at pe 0.1, U 1.6, series δ 0 and 0.1.
- `ra-rules`: final `y` against U, series `ra`, `bc`/`bc_averaging` × window, `bc_variance` (pe 0.05, δ 0.1).
- `ra-delta`: final `y` against δ at pe 0.1, U 1.4.
- `ad-connectivity`: final `y` against k 2 … 256, series p 0.2, 0.8, 1 (AD Fig. 4; N 500 for the browser).
- `w-dispersion`: final `dispersion` against d, series all, 30 × 30 lattice, scale-free m 2 and 4 (W Fig. 3).
The CLI names the stop `(stable)` or `(its last period)`. A stopped run holds its values (`holds_when_finished`).

## Survey

An `agreement` claims module, the papers' seed counts:
- DNAW: consensus at d 0.5 and two major clusters at d 0.2; the peak count falls with d; on the lattice at d 0.3, one cluster holds most agents with isolated agents near the ends (at stability, and whether at 119 periods).
- DAWF: RA clusters ≈ w/2u and BC's ≈ its integer part (major clusters); Figs. 5–7's three regimes and Fig. 8's other sample at Fig. 7's parameters; Fig. 9's four zones at N 1000 (the full 19 × 23 grid, δ 0 and 0.1) and the single-extreme zone's extent against the figure's; Fig. 10's unimodal and bimodal regions; §4.8's μ, δ and ue effects; §6 under both windows.
- M&C and DAW: which reading reproduces Fig. 9; both fixes are needed; the literal stability rule agrees with DAW's 1200 periods; the single-extreme share against N (M&C §5.3 against DAW's "any large number").
- AD: no single extreme on the Moore torus; a critical k that falls with p; the low-k regime under both cutoffs; the grid substrate.
- W: well-mixed dispersion steps near ⅓ and ½; scale-free and lattice curves without steps and close to each other; m 4 closer to well mixed; hubs end in the big cluster; unmoved agents on the scale-free network.

Claims that fail are reported, and the descriptions and README say so.

## Page

The presets menu gains a **Relative Agreement** group and the Compare entry; the Rules panel is generated from the schema in groups Population, Interaction, Extremists, Network and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged; `finished()` pauses at stability or the stop when asked. Editing tools, overlays, trails and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries untouched (including `opinions` after the cluster helper takes a gap); new entries for every `agreement` preset.
- **Core unit:** hᵢⱼ and the RA update on hand values (no influence at hᵢⱼ = uᵢ; asymmetry with unequal uncertainties); each BC rule under both windows; `bc_variance`'s uncertainty; the three pair updates (sequential sees the new j; one-way leaves j); extremist counts (rounding, δ, the coin flip), each placement and its boundaries; the margin and y, p′±, outcome on hand profiles; lattice, ring, grid-substrate and BA degrees and determinism; rewiring keeps links simple; edge and node pairing; stability and the cap; clusters, major, isolated, dispersion, unmoved; history compression and keyframes; the views and Inspect; live and reset fields; degenerate configs (N 2, pe 0 and 1, U = ue, μ 0 and 1, k 2, p 0 and 1).
- **Web:** schema groups and visibility, charts, the Compare entry, a sweep over an `agreement` base, determinism through the engine (presets that run past 200 periods in the golden list; `ra-meadows-cliff` run to its stop at 200 and inspected).
- **Browser (controller):** every preset's view and charts, Inspect, the stops, Compare, recording, Experiments, every existing scenario.

## Docs

README: a Relative Agreement section (the rules, the stated choices and switches, the readings and what each reproduces, presets, sweeps, and the findings: both fixes are needed; single extreme is a finite-size effect at δ 0; eq. 11's printed window reproduces none of §6; the cutoff decides AD's low-k regime). `docs/papers.md`: the milestone's row, with M&C and DAW as the critique and reply; roadmap: Milestone 22 done.

## Amendments (implementation planning)

The model was implemented in full while planning (`docs/superpowers/plans/2026-09-26-relative-agreement.md`) and measured with it; these change or extend the sections above.

- **Inspect reads cells** (see Views): `agent` is always null and `locate` returns nothing.
- **Weisbuch's lattice line** in `w-dispersion` uses his pairing and one-way updating, like the scale-free lines; the well-mixed line uses DNAW's symmetric meetings.
- **The web golden list** holds the nine presets still running at period 200 on seed 1 (`dnaw-lattice`, `dnaw-lattice-clusters`, `ra-central`, `ra-literal`, `ra-deffuant-2013`, `ra-bc-extremists`, `ad-moore`, `ad-small-world`, `w-scale-free`); the rest settle sooner. An engine test runs `ra-single` to its stop at 71 and `ra-meadows-cliff` to 200.
- **Measured with the implementation** (20–50 seeds; the survey's numbers):
  - Fig. 5 (pe 0.2, U 0.4, μ 0.5): 48 % of moderates become extremists (22 % with extremists at ±1) against the caption's 4 %.
  - Figs. 7–8 (pe 0.1, U 1.4, μ 0.5): both extremes in 39 of 40 runs, under every placement and update order; at Fig. 9's μ 0.2, single 24 and central 16 of 40. The figures' μ looks misstated (§4.8: larger μ widens both extremes).
  - Fig. 9 at N 1000: the single-extreme zone (y ≥ 0.75, pe ≤ 0.075, U ≥ 1.4) holds in 14 of 35 cells; at N 200, 35 of 35. With δ 0.1 it holds in all 55 cells at U ≥ 1.6, pe ≤ 0.15.
  - §4.8's ue: at N 1000, pe 0.05, U 1.4, the population drifts to one extreme in far fewer runs at ue 0.05 than at 0.2 (median |mean opinion| 0.03 against 0.75); and at ue 0.2 the extreme cluster settles at ±0.75, inside the reply's cutoff (y 0.00; 1.00 counted 0.3 inside the innermost extremist).
  - The reply's "any large number of agents": single extreme in 37 of 50 runs at N 2000 for pe 0.05, 1 of 50 for pe 0.1 (δ 0). With δ 0.1 y rises with N (0.84 at N 100 to 0.99 at N 2000).
  - Fig. 4 at N 1000: relative agreement 2.00, 2.70, 3.40, 5.36, 10.54 clusters at w/2u 2, 2.5, 3.33, 5, 10 (within a fifth of w/2u; 2.10, 3.04, 3.70, 5.90, 11.28 at N 200); bounded confidence 2.00, 2.98 at 2.5, 3.33, then 3.44, 4.32, 8.92 at 4, 5, 10 — the integer part only up to about 3.
  - AD (N 1000, 20 runs): most runs single from k 32 at p 0.8, 64 at p 1, 256 at p 0.2; none single on the Moore torus (largest y 0.51 over 160 runs); at k 2–4 central with the 0.9 cutoff, both extremes in 40 of 40 with 0.7; the grid substrate's y rises from 0.01 at k 8 to 1.00 at k 120.
  - Weisbuch (N 900, 50 runs): well-mixed dispersion 0.36, 0.50, 0.51 at d 0.15, 0.2, 0.25; scale-free 0.17, 0.34, 0.60, 0.84 and the lattice 0.02, 0.30, 0.70, 0.86 at d 0.15–0.3; 8 links 0.22 from well mixed against 0.57 for 4; 62 % of the ten best-connected agents in the largest cluster (which holds 49 %); 15.9 % never move (0.1 % well mixed).
  - DNAW's lattice at d 0.3: at period 119 (the caption's 100 000 iterations) the largest cluster holds 39 % on average; at stability 92 %, with a median of 28 isolated agents.
