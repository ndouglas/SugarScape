# SugarScape Milestone 25 — Threshold Models — Design

**Date:** 2026-09-27
**Builds on:** the milestone 1–24 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–24, the preset titles of `crates/sugarscape-core/src/titles.rs`, and milestone 24's shared `crate::graph`.
**Source text** (local copy in `papers/thresholds/`, a scan read by OCR):
- Mark Granovetter, "Threshold Models of Collective Behavior", *American Journal of Sociology* 83(6) (1978), 1420–1443 (Granovetter below).

**Follow-up** (local copy, fetched for this milestone from the Internet Archive's copy of PNAS):
- Duncan J. Watts, "A Simple Model of Global Cascades on Random Networks", *PNAS* 99(9) (2002), 5766–5771 (Watts below).

## Goal

Granovetter's threshold model as one model kind, `thresholds` ("Threshold Models"), a full citizen of the playground: his uniform and perturbed crowds, Figure 1's cobweb, Figure 2's normal crowds, and the four extensions he sketches — friends and acquaintances, crowds sampled from a city, clusters with movement, and ceilings (Figure 3's net benefit crossing zero twice) — with Watts's cascades on random networks as the follow-up; every unstated detail (who counts in the denominator, how fractional thresholds become people, zero thresholds, the friends rule, the clusters rule, what counts as a global cascade) a named switch or a stated choice, and every claim measured.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only). **Threshold comparisons are exact:** an actor acts when (acting others, weighted) ≥ θ × (group, weighted), compared without dividing — the prototype's floating division stopped the uniform crowd at 29.
- **Truthful descriptions and titles:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **Granovetter, the model:** "A person's threshold for joining a riot is defined here as the proportion of the group he would have to see join before he would do so"; "Depending on the substantive situation, thresholds may usefully be described as either proportions or absolute numbers … numerical examples given in this paper use groups of 100 people"; the process "is described by the difference equation: r(t + 1) = F[r(t)]" with F the c.d.f. "the proportion of the population having threshold less than or equal to x"; "where no provision has been made for 'removal' of participants, oscillatory behavior of r(t) is not possible, and an equilibrium will always be reached"; Figure 1: the equilibrium is "the point where the c.d.f. first crosses the 45° line from above."
- **Granovetter, the crowds:** the uniform crowd (thresholds 0, 1, …, 99) — "The equilibrium is 100"; replace the person with threshold 1 by one with threshold 2 — "the riot ends at that point, with one rioter." Figure 2: 100 people, normal thresholds with mean 25 ("Those thresholds below zero may be regarded … as equivalent to zero … Similar comments apply to thresholds above 100"): "Up until a critical point, σc, the equilibrium number of rioters increases gradually to about six. Then after this point, approximately 12.2, the value of r∞ jumps to nearly 100, after which it declines. (The limiting value, as σ increases without bound, is 50.)"
- **Granovetter, friends:** "the influence of friends is twice that of strangers … if he knows 20 people in this crowd of whom 15 have already joined the riot … our subject 'sees' [(15 × 2) + (33 × 1)] rioters and [(5 × 2) + (47 × 1)] nonrioters, leading him to form a ratio not of 48/100 but of 63/120 = .525" (he counts himself among the 52 nonrioters); for the perturbed crowd, "the null hypothesis becomes increasingly improbable as the weight attached to friends' behavior increases"; "the largest effects occur where people know, on the average, about one-quarter of the rest of the group"; "the symmetry of ties has little effect on outcomes"; "the equilibrium changes very little and rarely exceeds five to 10 rioters"; for the true uniform crowd "its equilibrium of 100 rioters is unstable against almost any kind of social structural influence. For most combinations of weights and acquaintance volume tested, the modal equilibrium result is one rioter." The friendship graph, the weights and volumes tested and the denominator rule are not given (the simulations are "in an earlier draft of this paper, available from the author").
- **Granovetter, sampled crowds:** crowds of 100 drawn from a uniform city: "The probability of no successes in 100 trials is then (1 − P)^100 = .37 … the chance of drawing one zero percenter but no one percenters … = .14. This means that in over half the cases (.37 + .14 = .51) the equilibrium result is either no rioters or one rioter"; Spilerman: "If this probability is, say, .10 … in a larger city where 10 incidents occurred the chance of no riot falls to (.90)^10 = .35."
- **Granovetter, clusters:** "what level of movement among clusters would have the most incendiary effect … too much movement out of a cluster which had reached a high equilibrium may have the effect of deactivating some rioters … a reverse bandwagon. Thus, for some threshold distributions small movements among clusters may have greater effects than large ones." No rule is given.
- **Granovetter, ceilings (Fig. 3):** "Some cautious individuals might join a riot when 50% of the others had but leave when the total passed 90% … In the case of two [crossings], results can still be computed by forward recursion, but equilibria cannot be guaranteed … if we found that aggregate behavior oscillated … at least some participants had net-benefit curves which crossed the x-axis more than once."
- **Watts, the model:** "An individual agent observes the current states (either 0 or 1) of k other agents … and adopts state 1 if at least a threshold fraction φ of its k neighbors are in state 1"; thresholds from f(φ) on the unit interval; a random graph with degree distribution p_k and mean z; "initially all-off … perturbed at time t = 0 by a small fraction Φ0 ≪ 1 of vertices that are switched on … all vertices updating their states in random, asynchronous order … Once a vertex has switched on, it remains on."
- **Watts, the claims:** the cascade condition Σ k(k − 1)ρ_k p_k = z with ρ_k = F(1/k) (Eq. 5), for a uniform random graph "zQ(K* − 1, z) = 1"; Fig. 1 (n 10 000, single-node seeds): the cascade window in (φ*, z) with a lower and an upper boundary, the simulated window close to the analytic one; Fig. 2 (φ* 0.18): the frequency of global cascades ≈ the extended vulnerable cluster, their size ≈ the connected component S = 1 − e^(−zS); Fig. 3 (n 1 000): at z 1.05 the cumulative cascade-size distribution has slope ½, at z 6.14 it is bimodal, with "only a single cascade occurring in 1,000 random trials"; Fig. 4a: normal thresholds (σ 0.05, 0.1) "cause the system to be less stable, yielding cascades over a greater range of both φ and z"; Fig. 4b: power-law degrees p_k = Ck^(−τ)e^(−k/κ), τ 2.5, "κ0 has been adjusted to generate graphs with variable z", are "much less vulnerable"; targeting: "the most connected nodes are far more likely than average nodes to trigger cascades, but not in the second regime." A global cascade is "a cascade that occupies a finite fraction of an infinite network" ("in practice … more than a fixed fraction of large, but finite network"; the fraction is not given).

## Measured in planning

A throwaway prototype (Python) of the rules below; the survey reproduces each with the implementation.

- **Uniform and perturbed crowds:** 100 and 1, exactly.
- **Figure 2, continuous** (r ← 100·Φ((r − 25)/σ)): the critical σ lies between 12.2 and 12.3; below it the equilibrium rises to 5.5 (σ 12.2; 4.0 at 12, 0.8 at 10); above, 100.0 (σ 12.3–15), then 99.3, 90.5, 65.9, 51.0 at σ 30, 50, 100, 1 000 — reproduces.
- **Figure 2, a crowd of 100 people:** with the normal's quantiles as thresholds, the crowd tips at σ 12.55 when thresholds are used as they are, 11.89 when rounded down to whole people, 12.22 when rounded to the nearest (Granovetter's figure) — the critical point depends on an unstated rounding. With crowds *sampled* from the normal there is no jump at all: at σ 10, 12, 12.2, 14, 16, 20 a riot of more than half comes in 1 %, 21 %, 28 %, 69 %, 94 %, 100 % of 2 000 crowds (median rioters at σ 12.2: 4).
- **Sampled crowds from the uniform city:** 0 rioters in 36.9 %, 1 in 13.3 % of 20 000 crowds (together 50.2 %; Granovetter .51); but everyone riots in only 2.6 %, fewer than 10 riot in 76 %, the mean is 12.2 — the "equilibrium of 100" is rare. Spilerman's arithmetic: .9^10 = 0.349.
- **Friends** (each pair friends with probability a, symmetric; friends count w; the actor counts himself as a nonrioting stranger; 300 crowds): the uniform crowd's modal result is 1 rioter at w 2 and 5 for every a from 0.05 to 0.5 (P(more than one) 0.06–0.50), 3–4 at a 0.9 — "modal … one rioter" holds for most settings. The perturbed crowd: P(more than one) 0 at w 2; at w 5, 0.20, 0.30, 0.50, 0, 0 at a 0.05, 0.1, 0.25, 0.5, 0.9 — rising with the weight and largest at a quarter, as stated; the 90th percentile 5 or fewer. One-way ties: 0.23 and 0.46 against 0.23 and 0.40 symmetric — little effect, as stated.
- **Clusters** (10 crowds of 100 from the uniform city; each step each actor moves to a random other crowd with probability m; actors act when the others acting in their crowd reach their threshold, and stop when they no longer do; 30 runs, the last 100 of 400 steps): 127, 222, 386, 466, 558, 510, 338 rioters of 1 000 at m 0, 0.001, 0.005, 0.01, 0.05, 0.2, 1 — an intermediate movement is the most incendiary, as Granovetter suggests.
- **Ceilings** (the uniform crowd; a share q leave when more than 90 of the others act): q 0.1 settles at 91 (both updates); q 0.3 under synchronous updating never settles — a slow build and a collapse, over and over — while asynchronous updating hovers at 89–92.
- **Watts, Fig. 1–2** (φ* 0.18, n 10 000, 100 single-node seeds): global cascades (over 1 %) in 2 %, 14 %, 57 %, 88 %, 65 %, 9 %, 2 %, 0 % of seeds at z 0.9, 1.05, 1.5, 3, 5, 6, 6.3, 6.6; their mean size 0.94 of the network at z 3 (S = 0.94). The analytic window (zQ(4, z) = 1, K* = 1/0.18) is z 1.02–5.75; the simulated upper edge is near 6.3.
- **Watts, Fig. 3** (n 1 000, 3 000 seeds): at z 1.05 the cumulative size distribution has slope −0.50 — reproduces; at z 6.14, 23 % of seeds set off a cascade of half the network or more — not "a single cascade occurring in 1,000 random trials". The upper edge moves with n (about 5 % at n 10 000).
- **Watts, Fig. 4a** (n 2 000, 150 seeds, cascades over 10 %): σ 0.1 extends the window from z 6 to beyond 12 (0.80 at z 12) — as stated for the upper side; but at the lower side fewer cascades (0.07 against 0.27 at z 1.2). (Nodes with thresholds of 0 or less were switched on only when reached — see `zero` below.)
- **Watts, Fig. 4b:** with τ 2.5 and k ≥ 1 the mean degree of p_k = Ck^(−2.5)e^(−k/κ) cannot exceed ζ(1.5)/ζ(2.5) = 1.95 for any κ, and at φ* 0.18 the cascade condition holds nowhere (Σ k(k − 1)ρ_k p_k/z at most 0.65, at z 1.54); a window exists only at small φ* (to 2.18× z at φ* 0.05, z 1.78). The figure's "variable z" beyond 1.95 cannot be built as stated.
- **Watts, targeting** (n 2 000, 300 seeds): the highest-degree node triggers a global cascade in 49 %, 93 % against a random node's 12 %, 35 % at z 1.1, 1.3 — and still 89 %, 53 % against 49 %, 23 % at z 5.5, 6.0: hubs remain about twice as likely in the second regime, where Watts says they are not.

## Architecture

Model kind `thresholds` ("Threshold Models"): `ModelKind::Thresholds`, `ModelConfig::Thresholds(ThresholdsConfig)` tagged `"model": "thresholds"`, a `ThresholdsWorld` implementing `Model`, schema, `SERIES`, presets, titles and golden entries. Code in `crates/sugarscape-core/src/thresholds/` (`config.rs`, `crowd.rs` for drawing thresholds, `theory.rs` for Granovetter's continuous recursion and Watts's cascade condition, `world.rs`, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`). Networks use `crate::graph` (`gnp` by expected degree; a new `configuration` builder for power-law degrees).

## The world: episodes

A world runs **episodes**: one crowd (or network) and one trigger, updated step by step until nothing changes (an equilibrium) or `max_steps` pass. With `repeat` on, the finished episode's size is recorded and the next begins — a fresh sample, fresh friendships or network, a fresh seed; with `repeat` off, the world stays at its equilibrium. Clusters are one long episode (movement never stops).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `actors` | 100 | reset | N, per crowd (2–20 000) |
| `distribution` | `uniform` | reset | `uniform` (thresholds 0, 1/N, …, (N − 1)/N), `perturbed` (the one at 1/N moved to 2/N), `normal` (`mean`, `sd`, as fractions; below 0 is 0, above 1 never acts), `fixed` (everyone `mean`: Watts's φ*) |
| `mean`, `sd` | 0.25, 0.122 | reset | the normal's (and `fixed`'s) parameters |
| `crowd` | `quantiles` | reset | how a normal crowd is realized: `quantiles` (the normal's (i + ½)/N quantiles: Granovetter's population) or `sampled` (N independent draws) |
| `rounding` | `exact` | reset | thresholds as fractions (`exact`), or made whole people: `floor` or `nearest` (×N, rounded) |
| `population` | `fixed` | reset | `fixed` (the crowd as drawn) or `city` (each episode draws N from the city: uniform thresholds 0–99 %, Granovetter's sampling) |
| `network` | `everyone` | reset | `everyone` (Granovetter: the whole crowd), `random` (Poisson degrees, mean `degree`: Watts), `power_law` (τ 2.5, κ set for `degree`, which must be below 1.95) |
| `degree` | 3 | reset | z |
| `counts_self` | true | reset | under `everyone`: the actor counts as a nonacting member of the group he divides by (Granovetter's 63/120), or not (N − 1) |
| `friends` | off | reset | `friends.enabled`, `acquaintance` 0.25 (pair probability), `weight` 2, `symmetric` true (under `everyone`) |
| `trigger` | `instigators` | reset | `instigators` (those whose threshold is 0 act at once: Granovetter), `random` (one random actor switched on: Watts), `hub` (the highest-degree actor) |
| `zero` | `acts` | reset | an actor with threshold 0 or less: `acts` spontaneously (Watts's rule read literally: 0 ≥ 0) or only `when_reached` (at least one neighbor on) |
| `update` | `synchronous` | live | `synchronous` (r(t + 1) = F[r(t)]) or `asynchronous` (random order, each seeing the latest states: Watts) |
| `ceilings` | off | reset | `ceilings.share` q (chosen at random), `at` 0.9: those actors stop when the proportion of others acting exceeds `at` (and may start again) |
| `clusters` | off | reset | `clusters.count` K (10), `movement` m (0.05): K crowds of N from the city; each step each actor moves to a random other crowd with probability m; everyone's state is recomputed against its crowd (reversible) |
| `repeat` | false | live | start a new episode at each equilibrium |
| `global` | 0.1 | live | the share of actors that makes a cascade global (Watts gives none) |
| `max_steps` | 1 000 | live | an episode ends here if it has not settled |
| `stop_at` | 0 | live | `finished()` at this step (0: never) |

Validation: `friends` only under `everyone`; `hub` needs a network; `power_law` needs `degree` < 1.95; `clusters` need `population: city`; `sd` ≥ 0.

## Step

- **Acting:** under `everyone`, actor i acts when A_i ≥ θ_i·G_i in exact arithmetic, where A_i is the number of others acting (with friends, w × friends acting + strangers acting) and G_i the group (N, or N − 1 without `counts_self`; with friends, w × friends + strangers, counting i as a stranger when `counts_self`). On a network, A_i is the neighbors acting and G_i the degree (an isolated actor never acts unless `zero: acts` and θ ≤ 0). With ceilings, an actor also stops when A_i > at·G_i.
- **Synchronous:** everyone decides from the last step's states. **Asynchronous:** a random order each step; each actor decides from the current states.
- **Without ceilings or clusters**, nobody who acts ever stops (Granovetter's no-removal model); with them, states are recomputed every step.
- **Clusters:** movement first (each actor with probability m to a uniformly random other crowd), then the update within each crowd.
- An episode settles when a step changes nothing.

## Theory

`theory.rs` gives Granovetter's continuous equilibrium r∞ for the configured distribution (forward recursion of r ← F(r) from r = 0 over real r, F the c.d.f. with mass below 0 at 0) and the first crossing of F with the 45° line; and Watts's cascade condition value G₀″(1)/z for Poisson and power-law degrees with fixed or normal thresholds.

## Statistics

`SERIES`: `acting` (the share acting now), `step` (steps into the episode), `episodes` (finished so far), `last_size` (the last episode's final share), `mean_size` (over episodes), `global_share` (the share of episodes over `global`), `theory` (Granovetter's r∞/N, or null), `clusters_mean` (under clusters, the mean share over the last 100 steps).

## Views

- **Participation × time** (left, 401 × 201): the share acting over the last 400 steps (one line per crowd under clusters), episode starts marked.
- **Middle panel** (8-cell gap, 101 wide): with a fixed crowd under `everyone`, Granovetter's Figure 1 — F against the 45° line with the episode's staircase; otherwise, the histogram of episode sizes in 101 bins.
- **Actors** (right): a grid colored by the mode.
- **Color modes:** **State** (the default), **Threshold**, **Degree**, **Crowd**.
- **Inspect:** an actor (threshold, ceiling, degree or friends, what it perceives, acting); a step (its share); a histogram bin or Figure 1 point.
- **Charts:** Participation (`acting`, `theory`); Episodes (`mean_size`, `global_share`); Last cascade (`last_size`). Time axis: Steps.

## Presets

Titles follow `titles.rs`'s style; drafts.

| Preset | Title | Setup |
|---|---|---|
| `gr-uniform` | One instigator, and all 100 riot | uniform (the kind's default) |
| `gr-perturbed` | Move one person up one notch, and only the instigator riots | perturbed |
| `gr-normal-12` / `gr-normal-13` | Mean threshold 25, spread 12 / 13: a handful riot, or nearly everyone | normal, quantiles, nearest |
| `gr-normal-sampled` | The same crowd drawn from real people: no sharp tipping point | normal σ 0.122, sampled, repeat |
| `gr-city` | Crowds drawn from a city that should riot: half end with no rioter or one | city, repeat |
| `gr-friends` | Count friends double, and the crowd that should riot mostly doesn't | uniform, friends 0.25 × 2, repeat |
| `gr-friends-perturbed` | Close friends rescue the stalled crowd, now and then | perturbed, friends 0.25 × 5, repeat |
| `gr-ceilings` | Join a crowd, leave a mob: the riot builds and collapses | uniform, 30 % ceilings at 90 % |
| `gr-clusters` | Ten crowds with people drifting between them | clusters, m 0.05 |
| `w-lower`, `w-middle`, `w-upper` | Few links: small cascades, now and then a large one / A middling network: most sparks spread everywhere / Many links: almost never, then everything | random, fixed 0.18, n 10 000, trigger random, zero acts, asynchronous, repeat; z 1.05 / 3 / 6.14 |
| `w-hetero` | Varied thresholds keep the window open | normal 0.18 ± 0.1, z 8 |
| `w-hub` | Light the best-connected node | z 1.3, trigger hub |

**Compare entry:** "Uniform vs perturbed crowd — Threshold Models (Compare)": `gr-uniform` and `gr-perturbed`.

## Experiments and CLI

Seeds and horizons measured to fit a browser run and recorded in each description.
- `gr-sd`: `acting` at equilibrium against σ (0.08–0.2), series quantiles/nearest, quantiles/exact, sampled (Fig. 2 and the rounding finding).
- `gr-friends`: `global_share` over 200 episodes against acquaintance, series weight 2, 5 (perturbed crowd, global 0.02, i.e. more than one rioter).
- `gr-movement`: `clusters_mean` against m.
- `gr-ceilings`: the swing of `acting` (its range over the last 100 steps) against the share with ceilings, series synchronous, asynchronous.
- `w-window`: `global_share` against z, series φ* 0.14, 0.18, 0.24 (Fig. 1).
- `w-hetero`: `global_share` against z, series σ 0, 0.05, 0.1 (Fig. 4a).
- `w-targeting`: `global_share` against z, series random, hub.
The CLI names the stop `(its last step)`.

## Survey

A `thresholds` claims module: Granovetter's uniform and perturbed crowds; Figure 2 (continuous σc, "about six", "nearly 100", limit 50); the crowd-of-100 tipping points under each rounding and the sampled crowd's smooth transition; the .37, .14 and .51 of sampled crowds and the fuller distribution; Spilerman's .35; the friends claims (weight, a quarter, symmetry, the perturbed crowd's size, the uniform crowd's modal 1); no oscillation without removal; oscillation with ceilings; an intermediate movement most incendiary. Watts: the analytic window against simulation; global size ≈ S; Fig. 3's slope ½ and bimodality with its "single cascade in 1,000"; Fig. 4a on both sides; Fig. 4b's unreachable z; hubs in both regimes. Claims that fail are reported, and the descriptions, titles and README say so.

## Page

The presets menu gains a **Threshold Models** group and the Compare entry; the Rules panel is generated from the schema in groups Crowd, Thresholds, Friends, Network, Ceilings, Clusters, Episodes and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `thresholds` preset; titles for every preset.
- **Core unit:** each distribution and rounding on hand values; exact comparisons (the uniform crowd reaches 100 at every N); the perturbed crowd stops at 1; Granovetter's 63/120 example; symmetric and one-way friendships; the city sample; networks' degrees (Poisson mean, power-law cap); zero thresholds both ways; isolated actors; triggers; synchronous and asynchronous equilibria equal without ceilings; ceilings' oscillation under synchronous updating; clusters' movement and reversibility; episodes and `repeat`; `global`; theory against hand values (σc, the Poisson window); the view and Inspect; keyframes; live and reset fields; degenerate configs (N 2, sd 0, a 0 and 1, degree 0).
- **Web:** schema groups and visibility, charts, the Compare entry, a sweep over a `thresholds` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, Compare, recording, Experiments.

## Docs

README: a Threshold Models section (the model, the stated choices, switches, presets, sweeps, and the findings: the uniform and perturbed crowds and Figure 2 reproduce, but a real crowd of 100 has no sharp tipping point and its critical σ depends on rounding; "equilibrium 100" happens in 2.6 % of sampled crowds; the friends claims hold under our reading; intermediate movement is most incendiary; ceilings make riots pulse; Watts's window and slope ½ reproduce, his upper edge and "one in 1,000" depend on n, Fig. 4b cannot be built above z 1.95, and hubs still help in the dense regime). `docs/papers.md`: the milestone's row with Watts; roadmap: Milestone 25 done.

## Amendments (implementation planning)

The model was implemented in full while planning (`docs/superpowers/plans/2026-09-27-thresholds.md`) and measured with it; these change or extend the sections above.

- **Watts's presets and sweeps are `watts-*`**, not `w-*`: `w-scale-free` is already Weisbuch's (milestone 22).
- **Thresholds are exact fractions** (`num/den`): whole-people thresholds as k/N, real ones to six decimals; every comparison is a·den ≥ num·g in integers.
- **Normal thresholds are portable**: quantiles by Acklam's approximation and draws by the anasazi's polar method, both on `crate::portable::ln`, so native and WASM agree bit for bit (the WASM golden test covers sampled normals).
- **Networks** use a new sparse G(n, p) (geometric skipping, portable) and a configuration model for power-law degrees (degrees drawn from the table, self-links and repeats dropped); the power law's κ is solved by bisection with the portable exponential.
- **Friends need at most 2 000 actors** (one-way ties draw every ordered pair).
- **`gr-ceilings` gives 10 % ceilings**, not 30 %: whether a crowd pulses depends on who holds the ceilings (at 30 % seed 1 settles; at 10 %, 34 of 40 crowds pulse), and the swing is about the share holding them (83–92 at 10 %).
- **Statistics:** `recent_mean` and `swing` (the mean and range of the share acting over the last 100 steps) replace `clusters_mean`; `theory` is Granovetter's continuous equilibrium for a normal crowd seen whole, null otherwise.
- **The view** marks episode starts with a short tick at the top of the time panel; the Figure 1 panel shows the c.d.f. (blue), the 45° line and the episode's staircase (orange).
- **Charts:** Participation (`acting`, `theory`), Episodes (`mean_size`, `global_share`), Last cascade (`last_size`), Swing (`swing`).
- **Measured with the implementation** (the survey, 20 claims; 14 hold, 6 fail): the uniform and perturbed crowds 100 and 1; Fig. 2's continuous 5.48 at σ 12.2 and 100.0 at 12.3; the crowd of 100 tipping at 12.55, 11.89, 12.23 (fractions, rounded down, rounded); sampled crowds past half 0.15 at σ 12 and 0.25 at 12.5; the city's 0.369 + 0.137 = 0.505, everyone in 2.3 %, mean 12.4; friends' weight 0, 0, 0.43, 0.52 at w 1, 2, 5, 10; the quarter peak 0.19, 0.27, 0.43, 0.01, 0 at a 0.05–0.9; symmetry equivalent (0.43 against 0.41); the perturbed crowd's 95th percentile 7; the uniform crowd's mode one in 8 of 10 settings; no episode past 35 steps without removal; ceilings pulsing in 34, 29, 22 of 40 at shares 0.1, 0.3, 0.5; movement 0.39 at m 0.05 against 0.12 and 0.11; Watts's window 1.02–5.76 analytic, 0.59, 0.91, 0.76 global at z 1.5, 3, 5 and 0 at 0.8 and 7; global size 0.941 against S 0.940; the lower edge's slope −0.48; the upper edge 19.9 % global at n 1 000 (3.0 % at n 10 000); Fig. 4a 0.85 against 0 at z 8 but 0.10 against 0.23 at z 1.2; Fig. 4b's largest ratio 0.65 and no global cascades at z 1.5; hubs 0.95 against 0.37 at z 1.3 and 0.88 against 0.52 at z 5.5.
