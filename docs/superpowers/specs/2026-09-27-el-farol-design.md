# SugarScape Milestone 23 — El Farol and the Minority Game — Design

**Date:** 2026-09-27
**Builds on:** the milestone 1–22 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–22, and the preset titles of `crates/sugarscape-core/src/titles.rs` (every preset gets a plain title saying what happens).
**Source texts** (local copies in `papers/el-farol/`):
- W. Brian Arthur, "Inductive Reasoning and Bounded Rationality", *American Economic Review* 84(2) (1994), 406–411 (Arthur below).
- Damien Challet and Yi-Cheng Zhang, "Emergence of Cooperation and Organization in an Evolutionary Game", *Physica A* 246 (1997), 407–418 (arXiv adap-org/9708006; CZ97 below).

**Follow-ups and critique** (local copies, fetched from arXiv for this milestone):
- Robert Savit, Radu Manuca and Rick Riolo, "Adaptive Competition, Market Efficiency, and Phase Transitions", *Physical Review Letters* 82 (1999), 2203 (adap-org/9712006; SMR below).
- Damien Challet and Yi-Cheng Zhang, "On the Minority Game: Analytical and Numerical Studies", *Physica A* 256 (1998), 514 (cond-mat/9805084; CZ98 below).
- Damien Challet, Matteo Marsili and Gabriele Ottino, "Shedding Light on El Farol", *Physica A* 332 (2004), 469 (cond-mat/0306445; CMO below).

## Goal

Arthur's bar and Challet and Zhang's minority game as one model kind, `farol` ("El Farol and the Minority Game"), a full citizen of the playground: Arthur's Fig. 1 and his unfigured claims, CZ97's figures and their Darwinian variant, SMR's and CZ98's memory transition, and CMO's critique and binary El Farol as presets, sweeps and switches, with every unstated detail (the predictor library, how predictors are scored, what an agent does at exactly the capacity, the N/x − 2 rounding, the evolution's timing and rates) a named switch or a stated choice, and every claim measured.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the sources are specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only).
- **Truthful descriptions and titles:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **Arthur, the bar:** "N people decide independently each week whether to go to a bar … N at 100 … the evening is enjoyable if things are not too crowded — specifically, if fewer than 60 percent of the possible 100 are present … a person or agent goes (deems it worth going) if he expects fewer than 60 to show up or stays home if he expects more than 60 to go … the only information available is the numbers who came in past weeks." Predictors are "functions that map the past d weeks' attendance figures into next week's", e.g. "the same as last week's", "a mirror image around 50 of last week's", "a (rounded) average of the last four weeks", "the trend in last 8 weeks, bounded by 0, 100", "the same as 2 weeks ago (2-period cycle detector)", "the same as 5 weeks ago". "I first create an 'alphabet soup' of predictors, in the form of several dozen focal predictors replicated many times. I then randomly ladle out k (6 or 12 or 23, say) of these to each of 100 agents." Each agent "decides to go or stay according to the currently most accurate predictor in his set … updates the accuracies of his monitored predictors."
- **Arthur, the claims (Fig. 1, the first 100 weeks):** "cycles are quickly 'arbitraged' away so there are no persistent cycles"; "mean attendance converges always to 60"; "of the active predictors … on average 40 percent are forecasting above 60, 60 percent below 60"; "These results appear throughout the experiments and are robust to changes in types of predictors created and in numbers assigned"; "the result would fail if all predictors could only predict below 60"; "The reader might ponder what would happen if all agents shared the same set of predictors."
- **CMO:** "the average convergence to optimality at the collective level is trivial and does not even require any intelligence on the side of agents … even zero-intelligence agents … are able to self-organize to the comfort level. … The really non-trivial question is whether agents are able or not to reduce stochastic fluctuations of the attendance A around L." Predictors "should not be rewarded depending on their precision but rather on the payoff they give … if A(t) = 59, a prediction of 5 is better than a prediction of 61": Uᵢ,ₛ(t + 1) = Uᵢ,ₛ(t) + Θ{[Aᵢ,ₛ − L][A(t) − L]}. The binary El Farol: strategies read the last m bits Θ[L − A(t − k)], each entry "attend" with probability ā; with L = 60, ā = 1/2 and m = 2, 3, 6 as N varies: ⟨A⟩ ≈ L "in a whole interval around Nā = L" for small m, shrinking as m grows; "in the region āN ≈ L adaptive agents behave less efficiently than random agents … stronger for small values of m"; "a small bias in the strategies, of either sign, is beneficial as it decreases the fluctuations"; "there is an intermediate memory length which is optimal".
- **CZ97, the game:** "N (odd) players … choose to be in side A or side B … those who are in the minority side win … all winners collect a point"; the signal is the last M winning sides; a strategy is a table from the M bits to a side, "We randomly draw S strategies for each player"; every strategy collects virtual points "as if it were used each time. The player uses the strategy having the highest accumulated points … he gets a real point only if the strategy used happens to win"; "currently a player switches immediately if another strategy has one virtual point more than that in use".
- **CZ97, the claims:** Fig. 1 (N 1001, M 6, 8, 10): "the fluctuation are indeed in decreasing order for ever increasingly 'intelligent' players"; Fig. 2 (mixed M 1–10, N 1001, S 5): larger brains win more and "above a certain size (M ≈ 6) the average performance … appears to saturate"; Fig. 3: a symmetric histogram around N/2; Fig. 4 (payoff N/x − 2, "these many (nearest integer values) points awarded to every player choosing the minority side", N 1001, M 4, S 5): "a histograph … with two peaks"; Fig. 5 (N 1001, M 5, S 2–9): "with increasing number of alternatives the players tend to perform worse"; Fig. 6: "the oftener one switches, less successful one would end up"; Fig. 9: with "the worst player … replaced by … a clone of the best player … virtual capitals reset to zero" and one strategy replaced by a new one, "Fluctuations are reduced and saturated"; Fig. 10: perfect cloning without mutation, "tremendous waste"; Fig. 11 (from M = 2, N 101 and 1001, S 5; "a bit of memory can be added or substracted for the cloned new player, with a small probability"): an "arm race" whose "saturation values are not universal, having to do with the time intervals of reproduction … Larger population … needs more powerful brains". Neither the replacement interval nor the mutation probabilities are given.
- **SMR, CZ98:** "σ²/N is a function only of 2^m/N"; its minimum is near 2^m/N ≈ 0.5 (SMR; CZ98's αc ≈ 0.34 for s = 2); below it σ ∝ N and no agent beats 50 %, above it σ ∝ N^½ and "some agents do win more than 50 % of the time".

## Measured in planning

A throwaway prototype of the rules below; seeds and horizons as stated; the survey reproduces each.

- **Arthur, with a 48-predictor library (four dozen) and accuracy scoring (decay λ 0.9), N 100, L 60, 20 seeds, 2 000 rounds after 200:**
  - mean attendance 59.1, 59.1, 59.4 at k 6, 12, 23 — the mean converges to 60;
  - σ²/N 5.2, 6.9, 11.4, against 0.24 for agents attending at random with probability 0.6 — adaptive agents fluctuate 20 to 50 times as much as coin-flippers (σ about 23–34 people against 5);
  - lag-1 autocorrelation −0.58, −0.43, −0.23 — a persistent two-period cycle; "no persistent cycles" does not hold under accuracy scoring;
  - forecasts above 60: 30 %, 31 %, 36 % of active predictors (Arthur: 40 %).
- **CMO's payoff scoring**, the same setups: mean 60.1, 60.2, 59.6; σ²/N 0.80, 2.70, 5.78 (still worse than random); lag-1 autocorrelation −0.04 to −0.08 (no cycle); above 60: 36 %, 38 %, 39 %.
- **The library matters:** with 25 predictors instead of 48, k 23 gives a mean of 52 and attendance swinging between near-empty and near-full (σ²/N 22); Arthur's "robust to changes in types of predictors created and in numbers assigned" holds for the mean only when the library is large enough. With every agent holding the whole library, σ²/N 26 and nobody is ever right.
- **SMR (N 101, S 2, 20 seeds, 10 000 rounds after 2 000):** σ²/N 2.06, 1.42, 0.76, 0.39, 0.10, 0.063, 0.10, 0.15, 0.22, 0.24 at m 1–8, 10, 12: the minimum at m 6 (2^m/N 0.63), random (0.25) again by m 12.
- **CZ97 Fig. 1 (N 1001, 10 seeds):** with S 5, σ²/N 6.9, 2.2, 0.36 at M 6, 8, 10 (0.24 at M 12) — decreasing, as CZ97 say; with S 2, 1.5, 0.16, 0.12. More strategies move the minimum to longer memories.
- **CZ97 Fig. 2 (mixed M 1–10, N 1001, S 5):** success 0.35, 0.40, 0.46, 0.48, 0.50, 0.50, 0.50, 0.50, 0.50, 0.50 by memory — saturating at M ≈ 6, as stated.
- **CZ97 Fig. 4 (N 1001, M 4, S 5):** with N/x − 2 rounded to the nearest integer, as stated, a near-even minority earns 0 points, strategies stay tied, and attendance keeps one random-width peak (σ²/N 0.25); unrounded, one central peak with small side lobes (σ²/N 22). Two peaks under neither reading.
- **CZ97 Fig. 5 (N 1001, M 5):** success 0.458 at S 2 down to 0.396 at S 9 — more strategies, worse players, as stated.
- **Random history (N 101, S 2):** σ²/N the same as with the true history at every m (1.34/1.37, 0.40/0.39, 0.062/0.062, 0.16/0.15 at m 2, 4, 6, 8).
- **CZ97 Fig. 11 (from M 2, S 5, 10 % strategy and memory mutation, 100 000 rounds):** mean memory over the run 3.4 (N 101) and 4.1 (N 1001) when the worst is replaced every 50 rounds; 3.2 and 2.5 every 200 rounds — memory rises, more for the larger population when replacement is frequent enough, and the level depends on the interval, as CZ97 say.
- **CMO's binary El Farol (L 60, ā 0.5, S 2, 10 seeds):** at m 2 the mean is within ±1.3 of 60 for N 90–150 while σ²/N is 1.3–1.7 near āN = L (random 0.25) and 0.12 at N 90 (a bias helps); at m 6, σ²/N 0.056–0.072 near āN = L (better than random) and the mean strays (−4.6 at N 90, +3.5 at N 150) — the interval where ⟨A⟩ ≈ L shrinks as m grows.

## Architecture

Model kind `farol` ("El Farol and the Minority Game"): `ModelKind::Farol`, `ModelConfig::Farol(FarolConfig)` tagged `"model": "farol"`, a `FarolWorld` implementing `Model`, schema, `SERIES`, presets, titles and golden entries. Code in `crates/sugarscape-core/src/farol/` (`config.rs`, `predictors.rs` for Arthur's library, `world.rs` for both games, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `game` | `el_farol` | reset | `el_farol` (Arthur) or `minority` (CZ97; with `capacity` ≠ (N − 1)/2, CMO's binary El Farol) |
| `agents` | 100 | reset | N (3–2 001) |
| `strategies` | 12 | reset | Arthur's k or CZ97's S (1–48 under `el_farol`, 1–16 under `minority`) |
| `behavior` | `inductive` | live | `inductive` or `random` (attend with probability `capacity`/N: CMO's zero-intelligence agents) |
| `capacity` | null | reset | L. El Farol: attendance of L or more is crowded. Minority: attending (side A) wins when A ≤ L. Null (amended in review): the game's own, 60 % of N rounded or (N − 1)/2, so editing N keeps the plain game |
| `scoring` | `error` | reset (amended in review: error scores and payoff points share one tally) | El Farol: `error` (Arthur's "most accurate": the predictor with the lowest decaying mean of \|prediction − A\|) or `payoff` (CMO: a point whenever the predictor's advice was right) |
| `decay` | 0.9 | live | λ in the error score s ← λs + (1 − λ)\|prediction − A\| (Arthur gives none) |
| `at_capacity` | `stay` | live | an agent whose predictor forecasts exactly L: `stay` (Arthur: go only if expecting "fewer than 60") or `go` |
| `shared` | false | reset | every agent holds the whole library (Arthur's "ponder … if all agents shared the same set") |
| `memory` | 3 | reset | M (1–16, with N·S·2^M ≤ 2^28 bits) |
| `mixed_memory` | off | reset | `mixed_memory.enabled`, `min` 1, `max` 10: memories uniform in the range (CZ97 Fig. 2) |
| `payoff` | `step` | live | `step` (a point per win) or `inverse` (N/x − 2 for x winners) |
| `rounding` | `nearest` | live | under `inverse`: `nearest` (CZ97: "nearest integer values") or `exact` |
| `bias` | 0.5 | reset | ā, the chance a strategy's entry says attend (CMO) |
| `information` | `true` | live | `true` (the real history) or `random` (a uniform random history each round) |
| `evolution` | off | reset | `evolution.enabled`, `every` 100 (rounds between replacements), `strategy_mutation` 0.1, `memory_mutation` 0 (CZ97 give neither interval nor rates) |
| `stop_at` | 0 | live | `finished()` at this round (0: never) |

The Rules panel shows the El Farol fields only under `el_farol` and the minority fields only under `minority`; `rounding` only under `inverse`; `decay` only under `error`.

## The library (El Farol)

48 focal predictors, each an integer forecast of next week's attendance from the last 12 weeks, clamped to 0–N: the same as k weeks ago (k 1–12; k ≥ 2 are Arthur's cycle detectors); the mirror image around N/2 of k weeks ago (k 1–8); the rounded mean of the last k weeks (k 2–12); the least-squares trend over the last k weeks extrapolated one week, rounded (k 3–12); the mirror of the rounded mean of the last k (k 2–8). Each agent draws k distinct predictors uniformly at random. The first 12 weeks of history are uniform on 0–N from the seed.

## Step (one round)

- **El Farol:** every predictor forecasts from the history; each inductive agent acts on its best predictor (lowest error score, or highest payoff score; ties broken uniformly at random), going when the forecast is below L (`at_capacity: go`: at or below). A = the number going. Crowded when A ≥ L. Every agent's predictors are rescored: `error` as above; `payoff`: a point when (forecast ≥ L) = (A ≥ L). A goer gains 1 when not crowded.
- **Minority:** the round's information μ is the last M outcomes (or a random draw), each agent's own M bits of it under mixed memories. Each inductive agent plays its strategy with the most virtual points (ties uniform) and chooses A or B from the table. A wins when A ≤ L. Winners gain the payoff (step: 1; inverse: N/x − 2, rounded or exact); every strategy that would have chosen the winning side gains the same virtual payoff. The outcome bit (A won) is appended to the history.
- **Evolution** (minority): after every `every` rounds, the agent with the least gain over those rounds is replaced by a copy of the agent with the most (its strategies and memory; its virtual points and gain reset to 0). With probability `memory_mutation` the copy's memory moves one up or down (1–16) and all its strategies are redrawn at the new memory; with probability `strategy_mutation` one of its strategies is redrawn. Ties for worst or best go to the lower index.

## Statistics

`SERIES`: `attendance` (A), `crowded` (El Farol: A ≥ L; minority: A > L), `fluctuation` ((A − c)² averaged over the last 100 rounds, divided by N; c is L, or N/2 when L = (N − 1)/2), `random_fluctuation` (p(1 − p) with p = L/N, or 0.25 — the coin-flippers' baseline), `success` (the share of agents whose choice was right this round), `mean_gain` (per agent per round so far), `switching` (the share of agents whose active strategy changed), `forecast_above` (El Farol: the share of active predictors forecasting above L), `mean_memory`.

## Views

- **Attendance × time** (left, 241 × 201): the last 240 rounds as a line from 0 (bottom) to N (top), L as a line across, crowded rounds shaded.
- **Histogram** (middle, 8-cell gap, 101 wide): attendance over all rounds so far in 101 bins, L marked.
- **Agents** (right, 8-cell gap): a square grid of cells, one per agent.
- **Color modes:** **Choice** (went or stayed this round; the default), **Gain** (cumulative gain, poor to rich), **Strategy** (El Farol: the active predictor's family — same, mirror, mean, trend; minority: switched this round or not), **Memory** (listed only under mixed memories or evolution).
- **Inspect:** an agent cell — its strategies or predictors with their scores (the active one marked), in El Farol what each forecasts now, its gain, switches and memory; the time panel — the round and its attendance. `agent` stays null and `locate` returns nothing (cells are read where they are, as in `opinions` and `agreement`).
- **Charts:** Attendance (`attendance` with the capacity); Fluctuations (`fluctuation`, `random_fluctuation`); Success (`success`, `mean_gain`); Forecasts (`forecast_above`, El Farol only); Switching; Memory (under mixed memories or evolution). Time axis: Rounds.

## Presets

Titles follow `titles.rs`'s style; these are drafts.

| Preset | Title | Setup |
|---|---|---|
| `ef-arthur` | Who goes to the bar? Attendance averages 60 but swings far more than chance | N 100, L 60, k 12, error scoring (Arthur Fig. 1) |
| `ef-payoff` | Score predictors by their advice, not their accuracy: the cycles go | the same, payoff scoring (CMO) |
| `ef-random` | Coin-flippers also average 60, with far smaller swings | random attendance, p 0.6 (CMO) |
| `ef-shared` | Everyone holds the same predictors, and nobody is ever right | `shared` |
| `mg-m6`, `mg-m8`, `mg-m10` | Two sides, the minority wins: short, longer and long memories | N 1001, S 5, M 6, 8, 10 (CZ97 Fig. 1) |
| `mg-mixed` | Long and short memories play together: the longer win, up to about six | mixed M 1–10, N 1001, S 5 (Fig. 2) |
| `mg-inverse` | Win more the smaller the minority: the paper's two peaks don't appear | inverse payoff, M 4, S 5, N 1001 (Fig. 4) |
| `mg-evolution` | The worst player is replaced by a mutated copy of the best | N 1001, M 6, S 5, evolution (Fig. 9) |
| `mg-inbred` | Perfect copies of the best player and no mutation | the same, no mutation (Fig. 10) |
| `mg-arms-race` | Memories that can grow: an arms race that levels off | from M 2, N 101, S 5, memory mutation 0.1, every 50 (Fig. 11) |
| `mg-crowded`, `mg-critical`, `mg-random-like` | Too little memory: worse than coin flips / Just enough memory: the best coordination / Too much memory: no better than chance | N 101, S 2, M 2, 6, 12 (SMR) |
| `cmo-binary` | El Farol as a yes-or-no game: 60 seats, two bits of memory | minority, L 60, N 120, ā 0.5, M 2 (CMO) |

**Compare entry:** "Accuracy vs payoff scoring — El Farol (Compare)": `ef-arthur` and `ef-payoff`.

## Experiments and CLI

Seeds and horizons measured to fit a browser run and recorded in each description; the survey runs longer.
- `ef-predictors`: final `fluctuation` against k (1–48), series error, payoff, random.
- `ef-capacity`: mean attendance against L (20–80), series inductive and random (CMO's trivial convergence).
- `mg-memory`: `fluctuation` against 2^M/N, series N 51, 101, 201 (SMR Fig. 6).
- `mg-fig-1`: `fluctuation` against M 4–12 at N 1001, series S 2, 5.
- `mg-strategies`: `success` against S 2–9 (CZ97 Fig. 5).
- `mg-information`: `fluctuation` against M, series true and random history.
- `cmo-bias`: `fluctuation` against N at L 60, series M 2, 3, 6 (CMO Fig. 1).
The CLI names the stop `(its last round)`.

## Survey

A `farol` claims module:
- Arthur: the mean converges to 60 at k 6, 12, 23; no persistent cycles (lag-1 autocorrelation small in magnitude); active predictors 40 % above 60; robust to the library.
- CMO: random agents' mean is also 60; adaptive agents fluctuate more than random ones in El Farol (both scorings) and near āN = L in the binary version; a small bias lowers fluctuations; an intermediate memory coordinates best.
- CZ97: Fig. 1's decreasing fluctuations; Fig. 2's saturation near M 6; Fig. 4's two peaks (both roundings); Fig. 5's worse players with more strategies; Fig. 6's switchers doing worse; Fig. 9's reduced fluctuations under evolution; Fig. 11's rising, saturating memory, higher for N 1001.
- SMR/CZ98: σ²/N depends only on 2^M/N (curves for N 51, 101, 201 overlap); the minimum near 0.3–0.7; σ ∝ N below and ∝ N^½ above; better-than-random agents only above.
Claims that fail are reported, and the descriptions, titles and README say so.

## Page

The presets menu gains an **El Farol and the Minority Game** group (titled presets) and the Compare entry; the Rules panel is generated from the schema in groups Game, El Farol, Minority game, Evolution and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged. Editing tools, overlays, trails, Follow and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `farol` preset; titles for every preset (the titles test).
- **Core unit:** each predictor family on hand histories (lags, mirrors, means, trend and clamping); the library's size and distinct draws; error and payoff scoring and the tie rule; `at_capacity` both ways; crowded at A = L; the minority rule at A = L; strategy tables and bias; true and random information; mixed memories reading their own bits; step and inverse payoffs, both roundings; switching counts; evolution (the worst replaced by the best, reset scores, both mutations, memory bounds); `random` behavior; statistics on hand series; the view and Inspect; keyframes; live and reset fields; degenerate configs (N 3, S 1, M 1, L 0 and N).
- **Web:** schema groups and visibility, charts, the Compare entry, a sweep over a `farol` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, Compare, recording, Experiments.

## Docs

README: an El Farol and the Minority Game section (both games, the stated library and choices, switches, presets, sweeps, and the findings: the mean at 60 is trivial and adaptive agents fluctuate far more than coin-flippers; accuracy scoring keeps a two-period cycle; Arthur's robustness needs a large library; CZ97's Fig. 1 and 2 hold; the two peaks of Fig. 4 do not appear; the memory transition). `docs/papers.md`: the milestone's row with SMR, CZ98 and CMO; roadmap: Milestone 23 done.

## Amendments (implementation planning)

The model was implemented in full while planning (`docs/superpowers/plans/2026-09-27-el-farol.md`) and measured with it; these change or extend the sections above.

- **Random agents in the plain minority game flip a fair coin** (p = ½) rather than attending with probability L/N = (N − 1)/2N: the "random guessing" Savit et al. and Challet and Zhang compare with, and a baseline of exactly 0.25. With any other capacity p = L/N, as specified.
- **Scores are the library's, not each agent's** (El Farol): every agent that holds a predictor rates it against the same history, so its score is kept once. The dynamics are unchanged.
- **The evolution presets replace the worst player every 10 rounds** (`every` still defaults to 100): at 100 the decline of Fig. 9 barely shows within a browser session.
- **Inspect:** a grid cell's agent is `member`; `agent` stays null.
- **Measured with the implementation** (the survey): Arthur's mean 59.1–59.5 at k 6, 12, 23 and 58.1–59.9 across k 2–32 and the rule at exactly 60; lag-1 autocorrelation −0.57, −0.43, −0.23 (−0.06 rated by payoff); forecasts above 60 in use 30 %, 32 %, 36 %; CMO's binary El Farol σ²/N 1.66 at N 120, m 2 against 0.25, a bias helping at N 90 (0.08) and less clearly at N 150 (1.41 against 1.74); CZ97 Fig. 1 7.17, 2.11, 0.40 (S 5) and 1.60, 0.19, 0.18 (S 2); Fig. 2 0.351 to 0.500 by M 6, level after; Fig. 4 all rounds within 5 % of N/2 when rounded, 72 % unrounded; Fig. 5 0.457 at S 2 against 0.397 at S 9; Fig. 6 correlations of switches with wins −0.73 to −0.86; Fig. 9 σ²/N 5.6 to 2.2 over 40 000 rounds; Fig. 10 without mutation 2.39 against 2.16 (N 1001) and 0.32 against 0.25 (N 101), not significant; Fig. 11 final mean memory 2.92 (N 101) and 4.95 (N 1001); SMR's minimum at M 5, 6, 7 for N 51, 101, 201 (2^M/N 0.63, 0.63, 0.64), σ²/N scaling 3.88 at m 2 and 0.98 at m 14 from N 51 to 201; the best of 101 agents wins 48.7 % at m 3, 54.2 % at m 6, 52.5 % at m 8 and 49.0 % at m 10 — above the transition only near it.
