# SugarScape Milestone 24 — Ants and Recruitment — Design

**Date:** 2026-09-27
**Builds on:** the milestone 1–23 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–23, and the preset titles of `crates/sugarscape-core/src/titles.rs` (every preset gets a plain title saying what happens).
**Source text** (local copy in `papers/ants/`):
- Alan Kirman, "Ants, Rationality, and Recruitment", *Quarterly Journal of Economics* 108(1) (1993), 137–156 (Kirman below).

**Critique** (local copy, fetched for this milestone):
- Simone Alfarano and Mishael Milaković, "Should Network Structure Matter in Agent-Based Finance?", Warwick working paper WP07-02 (April 2007); later Kiel Economics Working Paper 2008-04, published as "Network structure and N-dependence in agent-based herding models", *Journal of Economic Dynamics and Control* 33(1) (2009), 78–92 (AM below; we have the 2007 version).

## Goal

Kirman's recruitment chain as one model kind, `ants` ("Ants and Recruitment"), a full citizen of the playground: Kirman's Figures I and II and his unfigured claims, the three extensions he names but never runs (Becker's majority externality, more than two sources, meetings over a network), and AM's microfoundation, N-dependence and network critique, as presets, sweeps and switches, with every unstated detail (how self-conversion and recruitment combine, where a self-converting ant goes among several sources, the starting state, the time unit, the form of the majority pull) a named switch or a stated choice, and every claim measured.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before. The graph builders moved out of `agreement` keep their exact draws.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only).
- **Truthful descriptions and titles:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **Kirman, the puzzle:** "Two identical food sources were placed equidistant from an ants' nest … the ants stabilized, for a while, in a very unbalanced situation, with some 80 percent at one source and 20 percent at the other"; "sometimes a 'flip' occurred … a change from 80 percent at source A to 80 percent at source B"; the model "predicts that both of these features must occur."
- **Kirman, the chain:** N ants, k at the black source. "Two ants meet at random … The first is converted to the second's color with probability (1 − δ) … There is also a small probability ε that the first will change his own color independently before meeting anyone." Equation (1): P(k, k + 1) = (1 − k/N)(ε + (1 − δ) k/(N − 1)); P(k, k − 1) = (k/N)(ε + (1 − δ)(N − k)/(N − 1)). Footnote 9: "an ant converts with probability ε. If no conversion takes place, then a second ant is drawn, and the first converts to the second's group with probability γ. Then δ … is given by δ = (1 − γ + γε)." Special cases: ε = ½, δ = 1 is Ehrenfest's urn with a binomial equilibrium; ε = δ = 0 is a martingale absorbed at N with probability k₀/N.
- **Kirman, the claims:** the equilibrium distribution (4)–(5); "The uniform distribution (Figure Ib) occurs when … ε = (1 − δ)/(N − 1)"; "if ε < (1 − δ)/(N − 1) … the distribution has the form shown in Figure Ia"; "no specific assumption about the size of δ is necessary … except that it be strictly less than one"; "the probability, a priori, that a majority, once established, will decrease, diminishes with the size of that majority"; Figure I (N 100: a ε 0.005, δ 0.01; b ε 0.01, δ 0.02; c ε 0.15, δ 0.3); Figure II (100 000 meetings, every fiftieth plotted: a ε 0.15, δ 0.3 "fluctuates around one-half"; b ε 0.002, δ 0.01 "spends little time around the value of one-half and a great deal of time in the extremes … Although the average value of the system over the period is about one-half"); the Proposition (ε = a/N, δ = 2a/N, N → ∞: a symmetric Beta(a, a)); "since the process is Markov, the expected time to switch from one extreme state to the other is unmodified by the length of time spent in such an extreme state"; "the switches are so rapid".
- **Kirman, the extensions:** Becker's externality "could be incorporated by having the probability, 1 − δ, of conversion to the majority increase with the size of the majority. This would make the process more extreme"; "Generalizing to a larger number of sources would not change the analysis"; "This approach could be extended to model explicitly the communication network … using the Ising model."
- **AM, the microfoundation:** each agent switches at rate a + λ·(neighbors in the other state); in mean field, Kirman's rates with β = λD, D the mean degree; "Kirman's original interpretation of random pairwise meetings corresponds to the special case where D is always equal to unity, which leads to extensive transition rates, and therefore suffers from the problem of N-dependence"; the equilibrium is Beta(α, α) with α = aN/λD and Var[z] = 1/(4(2α + 1)).
- **AM, the implementation:** "pᵢ = (a + λ n(i, j))/(a + λN)" per agent, "sequentially update the state of each agent … One 'sweep' … N steps"; regular networks with D = 10, small worlds as the ring plus shortcuts with "S/N … s = 0.1", scale-free by Barabási–Albert with 2m = 10, random graphs with p = 0.1.
- **AM, the claims:** Figure 3: after 100 000 sweeps the densities "are in agreement with the mean-field prediction of a symmetric beta distribution … irrespective of the underlying network structure" for α 0.5, 1, 2; Figure 4 (a 0.5, λ 1, N from 50 in steps of 500, 300 000 sweeps): Var[z] = 1/(0.4N + 4) for regular, small-world and scale-free networks, inverse-variance slopes 0.512 (regular), 0.507 (small world), 0.402 (scale-free), and for the random network "a slope that is not significantly different from zero, and the intercept is 46.81" (predicted 44): "the random network would appear to be the only structure capable of overcoming the problem of N-dependence"; Figure 6: independent agents (λᵢ = 0) on the network reduce the variance far more than the same agents outside it, roughly as Var[z_H] = (8a/λp + 4 + 4qK)⁻¹.

## Measured in planning

A throwaway prototype of the rules below (exact stationary distributions for the complete-graph chain; ant-level runs otherwise); seeds and horizons as stated; the survey reproduces each.

- **Equation (4) is exact, and is a beta-binomial** with α = ε(N − 1)/(1 − δ): simulated 2·10⁷ meetings match it within total variation 0.018, 0.014, 0.003 for Figure Ia, Ib, Ic. At ε = (1 − δ)/(N − 1) it is uniform to 10⁻¹⁶. Figure Ia (α 0.50) is U-shaped, Ib (α 1.01) nearly flat, Ic (α 21) centered. Kirman's Proposition holds.
- **No 80/20 plateaus.** Over every valid ε 0.001–0.199 and δ 0–0.99 at N 100, the distribution's only interior mode is at N/2: the chain is U-shaped (piled at 0 and N), flat, or centered, never peaked near 80/20. Ants held at a source that always self-converts (independent ants, q 0.1–0.4) move the mode to 50, not to 80/20.
- **Becker's pull does give 80/20.** With pull 1 at Figure Ic's ε and δ, the modes are at 15 and 85 (pull 2: 13 and 87): Kirman's own extension produces the plateaus the base chain cannot. At Figure IIb's ε and δ, pull 0.5 already locks the colony at one source (no flip in 2·10⁸ meetings): "more extreme" holds, all the way to lock-in.
- **Figure II:** IIa's time mean is 0.49–0.51 in 20 seeds of 100 000 meetings (a flip in 6 of 20 by the 80/20 thresholds). IIb spends 7 % of the time between 0.4 and 0.6 and flips 0–6 times, but its time mean over 100 000 meetings is between 0.4 and 0.6 in only 3 seeds of 20 (means 7 to 93). "About one-half" needs 10⁷ meetings (20 of 20); at 10⁶, 11 of 20.
- **Majorities:** at IIb's and Ia's ε and δ, P(k, k − 1) falls steadily with k above N/2, as Kirman says; at Ic's it first rises (from 0.253 at k 51), so the claim holds only in the herding regime.
- **Markov switching:** over 2·10⁹ IIb meetings (50 614 flips, 25 per 10⁶ meetings) a regime ends at a constant rate once it has lasted half the mean residence (hazard 0.34 per half-mean) but more often in its first half (0.45): newly entered extremes often fall back. Residence averages 36 800 meetings (coefficient of variation 1.18); a transit between extremes averages 2 750 (7.5 % of a residence): the switches are rapid.
- **More sources:** at IIb's ε and δ, one source holds 80 % or more of the colony 79 %, 78 %, 77 %, 77 % of the time with 2, 3, 4, 6 sources, and the leader changes about 245–252 times per 10⁷ meetings: "would not change the analysis" holds.
- **N-dependence (Kirman's rule):** at fixed ε 0.002, δ 0.01, α passes 1 at N 500: the colony is within 10 % of an extreme 82 %, 68 %, 49 %, 20 %, 6 %, 0.5 %, 0 % of the time at N 50, 100, 200, 500, 1 000, 2 000, 5 000. On a ring of degree 10 the variance falls faster still (0.136, 0.032, 0.015, 0.008 at N 100, 500, 1 000, 2 000 against 0.174, 0.084, 0.050, 0.028 on the complete graph).
- **AM Figure 3 at N 100–200:** the random graph matches Beta(α, α) (Var[z] 0.125, 0.078, 0.045 against 0.125, 0.083, 0.050 at N 100; the same at N 200 over 10⁶ sweeps), the scale-free network nearly (0.116, 0.080, 0.046); the ring and the small world fall well short (0.093, 0.058, 0.038 at N 100; 0.066, 0.047, 0.031 at N 200 over 10⁶ sweeps): the mean field fails on lattices. At N 1 000 and AM's 100 000 sweeps only the random graph has mixed (0.140, 0.090, 0.048); the lattices stay near one-half (Var[z] 0.005–0.019).
- **AM Figure 6** (random graph, K 1 000, p 0.1, a 0.5, λ 1): Var[z] 0.026, 0.017, 0.015, 0.0069, 0.0038, 0.0020 at q 0, 0.01, 0.02, 0.05, 0.1, 0.2 against 0.023, 0.022, 0.022, 0.021, 0.018, 0.015 for the same agents off the network and 0.023, 0.012, 0.008, 0.004, 0.002, 0.001 by eq. 27: the variance falls far faster than the core/periphery case, as AM say, and eq. 27 is within a factor of two.
- **AM Figure 4** (a 0.5, λ 1, N 50–2 050 in steps of 500, one seed each, 100 000 sweeps after 10 000; AM ran 300 000): inverse-variance slopes 0.570 (regular), 0.483 (small world), 0.362 (scale-free) against the mean field's 0.4 and AM's 0.512, 0.507, 0.402; the random graph's slope 0.005 with intercept 41.1 (AM 46.8, mean field 44). Reproduces: only the random graph holds its variance as N grows. Here the lattices fall only a little below the mean field (Var[z] 0.0009 against 0.0012 at N 2 050 on the ring): the mean field's failure on lattices is large when herding is strong (α ≤ 2, above) and small when it is weak (α 0.05N here).
- **Kirman's pairwise rule on the same networks** (IIb's ε and δ, N 100–2 000): Var[z] 0.174, 0.086, 0.053, 0.027 on a random graph with p 0.1, the same as the complete graph's: under Kirman's meetings a random graph does not cure N-dependence, since each meeting is one partner however many neighbors an ant has.

## Architecture

Model kind `ants` ("Ants and Recruitment"): `ModelKind::Ants`, `ModelConfig::Ants(AntsConfig)` tagged `"model": "ants"`, an `AntsWorld` implementing `Model`, schema, `SERIES`, presets, titles and golden entries. Code in `crates/sugarscape-core/src/ants/` (`config.rs`, `world.rs` for both rules, `theory.rs` for the exact and mean-field distributions, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`).

**Shared graphs.** `agreement/network.rs`'s `Graph` (compressed adjacency lists) and its `ring` and `barabasi_albert` builders move to `crate::graph`, unchanged, and `agreement` uses them from there with the same draws (its golden entries prove it). `ants` adds `shortcuts` (AM's small world: the ring plus S = round(0.1·N) distinct random links, no rewiring) and `gnp` (a random graph with link probability p, drawn pair by pair in index order).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `ants` | 100 | reset | N (2–5 000) |
| `rule` | `kirman` | reset | `kirman` (random pairwise meetings, eq. 1) or `alfarano` (AM eq. 18: each ant in turn switches with probability (a + λ·n_opp)/(a + λN)) |
| `epsilon` | 0.002 | live | ε, self-conversion (Kirman) |
| `delta` | 0.01 | live | δ: a met ant of another source converts the first with probability 1 − δ (Kirman) |
| `conversion` | `kirman` | live | `kirman` (eq. 1 as written: the first ant switches with probability ε + (1 − δ)·[the second is elsewhere], clamped to 1) or `footnote` (footnote 9: switch with probability ε; otherwise meet, and convert with probability γ = (1 − δ)/(1 − ε), the γ that gives the same δ) |
| `meetings` | 50 | live | meetings per step under `kirman` (Figure II plots every fiftieth); a step under `alfarano` is one sweep of N updates |
| `sources` | 2 | reset | 2–6. A self-converting ant moves to a uniformly random other source (stated choice; with two sources, Kirman's "change his own color") |
| `pull` | 0 | live | Becker's externality: the recruiting probability becomes (1 − δ)·(1 + pull·(xⱼ − xᵢ)), clamped to 0–1, where xᵢ and xⱼ are the shares at the two ants' sources. 0 is Kirman's rule |
| `a`, `lambda` | 0.5, 1 | live | AM's a and λ (under `alfarano`) |
| `network` | `complete` | reset | `complete` (anyone meets anyone), `ring` (degree D), `small_world` (the ring plus 0.1·N shortcuts), `random` (link probability p), `scale_free` (Barabási–Albert, 2m = D). Under `kirman` the second ant is a uniformly random neighbor of the first (none: only self-conversion); under `alfarano`, n_opp counts the neighbors elsewhere (on `complete`: all other ants elsewhere) |
| `degree` | 10 | reset | D for `ring`, `small_world`, `scale_free` (even, 2–N − 1) |
| `link` | 0.1 | reset | p for `random` |
| `independent` | 0 | reset | q, the share of ants that never herd (λᵢ = 0 under `alfarano`; δᵢ = 1 under `kirman`), chosen at random |
| `start` | `random` | reset | `random` (each ant at a uniformly random source; Kirman does not say) or `one` (all at the first source, the martingale's k₀ = N) |
| `stop_at` | 0 | live | `finished()` at this step (0: never) |

Validation: `alfarano` takes `sources` 2 only (AM's model is binary); `pull` applies under `kirman`. The Rules panel shows Kirman's fields only under `kirman`, AM's only under `alfarano`, `degree` and `link` only under their networks.

## Step

- **Kirman** (`meetings` times): draw the first ant i uniformly. With probability ε it moves to another source (uniform among the others). Otherwise, unless it is independent, draw the second ant j (uniform among the others, or among i's neighbors); if j is at another source, i moves there with probability (1 − δ)·f, f = 1 + pull·(xⱼ − xᵢ) clamped so the probability is in 0–1. Under `conversion: kirman` one uniform u decides both (u < ε, else u < ε + (1 − δ)f), which is eq. (1) exactly; under `footnote`, two draws.
- **Alfarano** (one sweep): for i = 0 … N − 1 in order, i switches with probability (a + λᵢ·n_opp(i))/(a + λN), n_opp kept incrementally.

## Theory

`theory.rs` gives the exact stationary distribution of the two-source chain on the complete graph for any ε, δ and pull by detailed balance (eq. 4 with the pull-adjusted rates; the beta-binomial when pull is 0), and AM's Beta(α, α) with α = aN/λD. The histogram overlays the exact one under `kirman` on `complete` with two sources and no independent ants, and AM's under `alfarano`; otherwise none, and the view says so.

## Statistics

`SERIES`: `share` (the fraction at the first source, z), `top_share` (the largest source's share), `variance` (Var[z] over the run so far), `theory_variance` (the overlaid distribution's variance, or none), `flips` (regime switches so far: z crossing from 0.8 or more to 0.2 or less or back; with more sources, the source holding 80 % changing), `residence` (mean steps in a regime so far), `extreme` (share of steps with a source at 80 % or more).

## Views

- **Share × time** (left, 401 × 201): the last 400 steps of each source's share as lines, 0 at the bottom, 0.8 and 0.2 marked.
- **Histogram** (middle, 8-cell gap, 101 wide): the steps spent at each share so far in 101 bins, with the theory curve.
- **Ants** (right, 8-cell gap): a square grid of cells, one per ant in index order (so a ring reads row by row).
- **Color modes:** **Source** (the default), **Independent**, **Degree** (listed only on networks).
- **Inspect:** an ant — its source, whether it is independent, its degree and how many of its neighbors are elsewhere; the time panel — the step and the shares. `agent` stays null (cells are read where they are, as in `farol`).
- **Charts:** Share (`share`, or each source's share); Variance (`variance`, `theory_variance`); Flips (`flips`, `residence`); Extremes (`extreme`). Time axis: Steps.

## Presets

Titles follow `titles.rs`'s style; these are drafts.

| Preset | Title | Setup |
|---|---|---|
| `ants-1a` | Two identical sources, and most of the colony crowds one | N 100, ε 0.005, δ 0.01 (Figure Ia) |
| `ants-1b` | Every split of the colony is equally likely | ε 0.01, δ 0.02 (Ib) |
| `ants-1c` | Weak recruiting: the ants spread evenly | ε 0.15, δ 0.3 (Ic) |
| `ants-2a` | The colony wanders around half and half | ε 0.15, δ 0.3, the first 2 000 steps (Figure IIa) |
| `ants-2b` | Nearly all the ants at one source, then a sudden flip | ε 0.002, δ 0.01 (IIb; the kind's default) |
| `ants-crowd` | Ten times the ants, the same habits, and the herding is gone | IIb at N 1 000 |
| `ants-becker` | Following the crowd pays: the colony settles 85–15, as the real ants did | Ic with pull 1 |
| `ants-lock` | Following the crowd pays too well: one source forever | IIb with pull 0.5 |
| `ants-three` | Three sources: one wins for a while, then another | IIb with 3 sources |
| `am-ring`, `am-random`, `am-scale-free` | Herding over a ring / a random network / a network with hubs | `alfarano`, N 200, a and λ for α 0.5, D 10 / p 0.1 |
| `am-independent` | A few ants who ignore everyone calm the whole colony | `alfarano`, random, N 1 000, q 0.05 |

**Compare entry:** "Colony size — Ants (Compare)": `ants-2b` and `ants-crowd`.

## Experiments and CLI

Seeds and horizons measured to fit a browser run and recorded in each description; the survey runs longer.
- `ants-alpha`: final `variance` against ε at N 100, δ 0.01, with the exact line (`theory_variance`).
- `ants-n`: final `variance` against N (50–2 000), series Kirman complete, Kirman ring, AM ring, AM random (AM Figure 4 and our contrast).
- `ants-flips`: `flips` per 10⁶ meetings against ε.
- `ants-pull`: `extreme` and `residence` against pull, at IIb and at Ic.
- `ants-sources`: `extreme` against the number of sources.
- `am-independent`: `variance` against q (AM Figure 6).
The CLI names the stop `(its last step)`.

## Survey

An `ants` claims module:
- Kirman: eq. (4) against simulation; the uniform threshold; Figure I's three shapes; IIa near one-half; IIb at the extremes with flips; IIb's time mean "about one-half" over 100 000 meetings; shrinking majorities; the flat switching hazard; rapid switches; the Proposition's Beta limit; 80/20 plateaus (base chain, independent ants, Becker's pull); Becker's "more extreme"; more sources "would not change the analysis"; the Ehrenfest and martingale special cases (absorption at N with probability k₀/N).
- AM: Figure 3's Beta fits on all four networks; Figure 4's slopes and the random graph's flat variance; under Kirman's pairwise rule no network cures N-dependence; Figure 6's independent agents.
Claims that fail are reported, and the descriptions, titles and README say so.

## Page

The presets menu gains an **Ants and Recruitment** group (titled presets) and the Compare entry; the Rules panel is generated from the schema in groups Colony, Kirman, Alfarano & Milaković, Network and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged. Editing tools, overlays, trails, Follow and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries untouched (including `agreement`'s after the graph move); new entries for every `ants` preset; titles for every preset.
- **Core unit:** eq. (1)'s rates against hand values; the exact distribution (sums to 1, the beta-binomial, uniform at the threshold, Ehrenfest's binomial at ε ½, δ 1); a long run's histogram against it; the footnote rule's equivalence; self-conversion among several sources; pull (0 is Kirman's rule; clamping); independent ants never recruited; each network's degree and link count (AM's shortcuts, G(n, p), Barabási–Albert with 2m = D); `alfarano`'s probability and incremental n_opp against a recount; flips and residence on hand series; the martingale's absorption; the view and Inspect; keyframes; live and reset fields; degenerate configs (N 2, ε 0, δ 1, an isolated ant).
- **Web:** schema groups and visibility, charts, the Compare entry, a sweep over an `ants` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, Compare, recording, Experiments.

## Docs

README: an Ants and Recruitment section (the chain, the stated choices, switches, presets, sweeps, and the findings: the chain is exactly beta-binomial and never plateaus at 80/20, while Becker's pull does; IIb's time mean needs a hundred times the figure's run to reach one-half; herding fades with N at fixed ε, δ; AM's mean field holds on random graphs but not on lattices). `docs/papers.md`: the milestone's row with AM, and the Queue's first entry removed; roadmap: Milestone 24 done.

## Amendments (implementation planning)

The model was implemented in full while planning (`docs/superpowers/plans/2026-09-27-ants.md`) and measured with it; these change or extend the sections above.

- **Becker's pull peaks at 18 % and 82 %**, not 15 and 85: the prototype let ε plus the clamped recruiting probability exceed 1; the implementation caps the meeting's switching probability at 1 (eq. 1's single draw), so an ant joins with probability min(1, ε + (1 − δ)f) − ε. The 80–20 finding stands, closer.
- **Alfarano and Milaković's presets use 100 ants** (a 0.05, α ≈ 0.5 on all three networks), not 200: at 200 they flip only 1–5 times in 20 000 sweeps, too slowly for a browser session.
- **`ants-crowd` runs 500 meetings a step**, so each of its 1 000 ants meets as often as in `ants-2b`.
- **The footnote rule needs δ ≥ ε** (validation): footnote 9's δ = 1 − γ + γε is at least ε.
- **The Rules panel shows `degree` under every network** (its help names the three that use it; a schema condition tests one value); `link` only under `random`. The color modes are always listed, Degree included.
- **Charts:** Share shows `share` and `top_share` (no per-source series); `theory_variance` is NaN (null in JSON) when no theory applies. `residence` is the mean number of steps between successive holders of 80 %, counted from the first holder's arrival (0 before the first flip).
- **Shared graphs:** `crate::graph` holds `Graph` (with `from_lists` public), `ring`, `rewire`, `barabasi_albert`, and the new `shortcuts` and `gnp`; `agreement` keeps `Graph::new` and its torus, with the same draws.
- **`ants-alpha` has no exact line**: a sweep reads one series, so its description lists the exact variances beside the measured ones. `ants-n` runs 500 meetings a step at every N.
- **The time panel** draws one line (the first source) with two sources, one per source otherwise. Inspect reads a time column's `shares`, a histogram row's `share`, `count` and `theory`, and an ant as `member`.
- **Chart axes below 1 keep three significant digits** (the page's shared `compactNumber`): the browser check found the Variance chart's axis reading 0, 0, 0.
- **Measured with the implementation** (the survey, 19 claims): eq. (4) within total variation 0.011, 0.010, 0.004 (Ia, Ib, Ic); uniform at the threshold to 4·10⁻¹⁷; no peak between 65 % and 95 % in 17 820 settings, a peak at 53 % with a fifth of the ants never herding; IIa's time means 0.49–0.51; IIb 8 % of steps near one-half, 77 % held at 80 %, but 4 of 20 time means between 0.4 and 0.6 over 100 000 meetings (20 of 20 over 10⁷); majorities shrink less as they grow at Ia, Ib and IIb, not Ic (from k 51); 896 and 894 steps to the next flip after half and all of the mean regime (819); crossings 8.4 % of a regime; the Proposition's variances 0.1250 and 0.0500 at N 10 000; Ehrenfest's binomial and the martingale's absorption (0.51 against 0.50); Becker's variances rising with the pull at IIb and Ic, no flips at pull 0.5; 2, 3 and 6 sources equivalent; Var[z] 0.179 at N 100 against 0.048 at N 1 000; AM's Figure 3 fails on the ring (0.086, 0.055, 0.037 against 0.125, 0.083, 0.050) and small world, holds on scale-free and random networks; Figure 4's slopes 0.518, 0.428, 0.384 and 0.004; Kirman's rule on a random network 0.180 against 0.052; Figure 6's 0.0063 and 0.0041 against 0.0191 and 0.0171 off the network; the start does not matter.
