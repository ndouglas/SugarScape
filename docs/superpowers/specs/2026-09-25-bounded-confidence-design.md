# SugarScape Milestone 16 — Bounded Confidence (Hegselmann & Krause) — Design

**Date:** 2026-09-25
**Builds on:** the milestone 1–15 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds and the literal-default-plus-named-switch pattern of milestones 11–15.
**Source text:** Rainer Hegselmann and Ulrich Krause, "Opinion Dynamics and Bounded Confidence: Models, Analysis, and Simulation", *Journal of Artificial Societies and Social Simulation* 5(3) (2002), 2 (HK below).
**Related:** Jan Lorenz, "Consensus Strikes Back in the Hegselmann-Krause Model of Continuous Opinion Dynamics Under Bounded Confidence", *JASSS* 9(1) (2006), 8.

The milestone number is provisional: another session is building ethnocentrism in parallel, and whichever merges second renumbers.

## Goal

HK's bounded-confidence model (their eq. BC) as a tenth model kind, `opinions` ("Bounded Confidence"), a full citizen of the playground: the paper's Section 4 figures as presets and sweeps, its symmetric, opinion-independent and opinion-dependent confidence as settings, and its two unfigured claims — random serial updating changes nothing, and local neighborhoods remove polarization — as named switches, with every claim measured over 20 seeds (50 where HK used 50).

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the source is specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical).
- **Truthful descriptions:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **The rule (BC, eq. 2.3):** "An agent i takes only those agents j into account whose opinions differ from his own not more than a certain confidence level εi … agent i puts an equal weight on all j ∈ I(i, x)": x_i(t+1) = |I(i, x(t))|⁻¹ Σ_{j ∈ I(i, x(t))} x_j(t). Opinions in [0, 1]; the set includes i.
- **Symmetric (§4.1):** "625 opinions. Updating is simultaneous." Fig. 2 (one start): ε = 0.01 — "exactly 38 different opinions survive"; 0.15 — "two camps"; 0.25 — "consensus"; "it takes less than 15 periods to get a stable pattern". Fig. 3: ε = 0.01 … 0.40, 50 random starts each, "each run is continued until the dynamics becomes stable": plurality → polarization (camps left and right of the center) → consensus, with a sudden change "at about step 25"; "for all values εl, εr > 0.4 the result is always conformity".
- **Regular starts:** Figs. 4–5 (50 opinions, ε = 0.2): "the ε-profile splits in t6"; "from period 7 to 8 onwards nothing changes anymore". Fig. 7: 100 opinions, ε = 0.05, "8 splits". Fig. 8: 100 opinions, ε = 0.25, "no split, total consensus". "Under simultaneous updating cracked profiles can never get connected again" (also §3 D II).
- **Opinion-independent asymmetry (§4.2.1):** j counts when x_i − εl ≤ x_j ≤ x_i + εr. Fig. 10 (625 random): εl/εr = 0.02/0.04, 0.03/0.15, 0.10/0.25. Fig. 11: walks εl = 0.9, 0.75, 0.5, 0.25, 0.1 × εr, εr to 0.4, 50 runs: the consensus "moves into the favoured … direction"; asymmetric polarization with a big camp at the favored border. Fig. 12: 25 runs on the grid εl, εr = 0, 0.02, …, 0.4: surviving opinions (plurality → mostly 2 → 1) and the final mean opinion, moving toward the favored side, "extreme if there is only little confidence in the non favoured direction". Fig. 13 (100 opinions, caption "εl = 0.8, εr = 0.24"): one-sided splits, which "can close again".
- **Opinion-dependent asymmetry (§4.2.2):** b_r(x) = m·x + (1 − m)/2, b_l = 1 − b_r, εr(x) = b_r(x)·ε, εl(x) = b_l(x)·ε; worked example x = 0.6, ε = 0.4, m = 0.5 → εl = 0.18, εr = 0.22. Fig. 17: ε = 0.2, 0.4, 0.6, m = 0 … 1 in 26 steps: bias polarizes, camps grow and move outward, "for an m = 1 the two camps occupy the most extreme positions 0 and 1"; at ε = 0.6 the consensus breaks "about step 11, i.e. m ≈ 0.4". Fig. 18 (50 regular, ε = 0.6, m = 0, 0.25, 0.5, 0.75, 0.99): at m = 0.5 "in period 4 the profile splits finally and two polarised opinion camps remain"; with rising m consensus takes longer.
- **Claims without figures (§4.3):** "Random serial updating gives extreme opinions a slightly better chance to survive. But none of the results stated above depends crucially on simultaneous updating." "First simulations show that this type of locality matters dramatically: If the neighbourhoods in which the agents interact are fairly small (though overlapping!), then the phase in between plurality and consensus, i.e. polarization, disappears."
- **Theory (§3 D, App. D):** the profile stabilizes in finite time (Theorem 6); a two-sided split is permanent.

## Architecture

Model kind `opinions` ("Bounded Confidence"): `ModelKind::Opinions`, `ModelConfig::Opinions(OpinionsConfig)` tagged `"model": "opinions"`, an `OpinionsWorld` implementing `Model`, schema, `SERIES`, presets and golden entries — the same wiring as the culture and classes models. Code in `crates/sugarscape-core/src/opinions/` (`config.rs`, `world.rs`, `stats.rs`, `view.rs` for the opinion × time frame, `presets.rs`, `mod.rs`).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `agents` | 625 | reset | n (2–2000); with `lattice`, width × height |
| `start` | `random` | reset | `random` (uniform on [0, 1]) or `regular` (x_i = i/(n − 1), both ends included) |
| `confidence` | `symmetric` | live | `symmetric` (εl = εr = ε), `asymmetric` (§4.2.1), `opinion_dependent` (§4.2.2) |
| `epsilon` | 0.15 | live | ε (0–1): the reach each way, or the total under `opinion_dependent` |
| `epsilon_left`, `epsilon_right` | 0.1, 0.2 | live | εl, εr (0–1) under `asymmetric` |
| `bias` | 0.5 | live | m (0–1) under `opinion_dependent` |
| `updating` | `simultaneous` | live | `simultaneous` (HK), `serial_shuffled` (each agent once per period, in a fresh random order, seeing opinions as they change), `serial_random` (n uniform draws with replacement per period) — HK's "random serial updating" does not say which |
| `interaction` | `all` | reset | `all` (HK) or `lattice`: a torus where an agent's candidates are itself and its neighbors |
| `lattice.width`, `lattice.height` | 25, 25 | reset | 3–44 each (at most 1 936 agents); `agents` = width × height |
| `lattice.neighborhood` | `moore` | reset | `moore` or `von_neumann` |
| `stop_when_stable` | true | live | `finished()` at the first stable period |

Under `symmetric` only `epsilon` is read; under `asymmetric` only `epsilon_left` and `epsilon_right`; under `opinion_dependent` `epsilon` and `bias`. The Rules panel shows all three and each help line says when it applies.

## Step (one period)

- **Reach:** agent i counts j when x_i − εl(i) ≤ x_j ≤ x_i + εr(i), with εl(i) = εr(i) = ε (symmetric), the fixed εl, εr (asymmetric), or εl(i) = (1 − b_r(x_i))·ε, εr(i) = b_r(x_i)·ε with b_r(x) = m·x + (1 − m)/2 (opinion-dependent). The comparison is exact on f64; the set always contains i.
- **Simultaneous:** every agent's new opinion is the mean of the reached opinions at t. With `all`, opinions are kept sorted with prefix sums, so each agent's reach is a contiguous run found by binary search (O(n log n) per period). The sum over a run is taken as a prefix-sum difference; stated below as a numeric choice.
- **Serial:** agents update one at a time against the current profile (O(n) each, direct summation).
- **Lattice:** candidates are the agent and its 8 (Moore) or 4 (von Neumann) torus neighbors; direct summation.
- **Numerics (stated):** means are computed by direct summation over the reached agents in index order for lattice and serial updating, and by sorted prefix sums otherwise; the golden entries pin whichever the build uses. A period is **stable** when no opinion moves more than 10⁻¹⁰; clusters are maximal runs of sorted opinions whose neighboring gaps are at most 10⁻¹⁰. (Averaging k identical doubles need not return the same double, so exact stillness is not guaranteed; the tolerance is far below any ε.)

## Statistics

`SERIES`: `clusters` (surviving opinions, as above), `largest`, `second` (shares of agents in the two biggest clusters), `mean_opinion`, `median_opinion`, `range` (max − min), `splits` (gaps between sorted neighbors x_{k+1} − x_k greater than both the lower one's εr and the upper one's εl), `one_sided_splits` (gaps within exactly one of those reaches), `max_change` (the largest |Δx| this period), `stable_at` (the first stable period, else the current one). Splits are counted on the sorted profile for `all`; under `lattice` they are reported on the sorted profile too (they then describe the opinion distribution, not who can reach whom — stated in the help).

## Views

- **Opinion × time** on the grid canvas: 240 × 200 cells; opinion 0 at the bottom, 1 at the top; 4 cells per period, the last 60 periods (older ones scroll off, as in the tag × time view). Each agent's line is drawn between consecutive periods; where sorted neighbors are within reach of each other in a period, the gap between them is filled gray (HK's Figs. 4, 7, 13). The history (opinions per retained period) is world state: keyframes and step-back restore it.
- **Color modes:** **Start** (each line by its starting opinion, red at 0 through magenta at 1, as in HK), **Opinion** (by the current opinion), and with `lattice` **Lattice** (the torus, one block per site colored by opinion).
- **Inspect:** a point gives the period, the opinion there, and the agents whose lines pass within one cell (id, start, current opinion, εl, εr, how many it reaches); on the Lattice mode, a site. `locate` returns nothing; Follow hidden.
- **Charts:** Clusters; Largest camps (`largest`, `second`); Mean and median; Splits (two-sided, one-sided); Change (`max_change`). Time axis: Periods.

## Presets

| Preset | Setup | Source |
|---|---|---|
| `hk-plurality` | 625 random, ε 0.01 | Fig. 2a |
| `hk-polarisation` | 625 random, ε 0.15 | Fig. 2b |
| `hk-consensus` | 625 random, ε 0.25 | Fig. 2c |
| `hk-regular-50` | 50 regular, ε 0.2 | Figs. 4–5 |
| `hk-regular-plurality` | 100 regular, ε 0.05 | Fig. 7 |
| `hk-regular-consensus` | 100 regular, ε 0.25 | Fig. 8 |
| `hk-asym-a`, `hk-asym-b`, `hk-asym-c` | 625 random, εl/εr 0.02/0.04, 0.03/0.15, 0.10/0.25 | Fig. 10 |
| `hk-one-sided` | 100 regular, εl 0.08, εr 0.24 | Fig. 13 (caption's 0.8 read as 0.08: εl = 0.8 would reach every opinion below) |
| `hk-bias` | 50 regular, opinion-dependent, ε 0.6, m 0.5 | Fig. 18c |
| `hk-serial` | `hk-polarisation`, `serial_shuffled` | §4.3 |
| `hk-lattice` | 25 × 25 Moore torus, ε 0.15 | §4.3 |

**Compare entry:** "Simultaneous vs serial updating — Bounded Confidence (Compare)": `hk-polarisation` and `hk-serial`.

## Experiments and CLI

Seeds and ranges measured to fit a browser run, recorded in each description; the survey runs the paper's counts. Runs stop when stable; sweeps read a stopped world at its last values (the culture model's padding).
- `hk-diagonal`: final `clusters` against ε = 0.01 … 0.40 (Fig. 3; and `largest`).
- `hk-asymmetry`: final `mean_opinion` against εr = 0.02 … 0.40, series εl = 0.9, 0.5, 0.1 × εr (Figs. 11, 12c).
- `hk-bias`: final `clusters` (and `range`) against m = 0 … 1, series ε = 0.2, 0.4, 0.6 (Fig. 17).
- `hk-updating`: final `clusters` against ε, series the three updating modes.
- `hk-lattice`: final `clusters` and `largest` + `second` against ε, series `all`, Moore, von Neumann.
- `hk-population`: final `largest` against n = 25 … 2000 at ε = 0.2 and 0.25 (Lorenz 2006: the consensus threshold moves with n).
The CLI names the stop `(stable)`.

## Survey

An `opinions` claims module:
- Fig. 2: surviving opinions at ε 0.01 (the paper's single run: 38); two camps at 0.15; consensus at 0.25; stable within 15 periods.
- Fig. 3 (50 seeds): plurality → polarization → consensus in that order as ε grows, the consensus change near 0.25; consensus for every ε > 0.4.
- Regular starts: still from period 8 (50, ε 0.2); 8 splits (100, ε 0.05); consensus (100, ε 0.25).
- §4.2.1: the final mean moves toward the favored side and more as εl/εr falls (Fig. 12c); one-sided splits close again (some one-sided split disappears in a run).
- §4.2.2: at ε 0.6 consensus holds at small m and breaks near m ≈ 0.4; the camps reach 0 and 1 at m = 1; the camp distance grows with m; consensus takes longer as m grows.
- §4.3: serial updating (both readings) leaves the Fig. 3 phases in place; on a lattice polarization disappears (the share of runs with two major camps at the ε that polarizes all-to-all).
- Lorenz 2006: the ε at which consensus becomes typical depends on n.

Claims that fail are reported, and the descriptions and README say so.

## Page

The presets menu gains a **Bounded Confidence** group and the Compare entry; the Rules panel is generated from the schema in groups Population, Confidence, Updating and Lattice. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged; `finished()` pauses at stability when asked. Editing tools, overlays, trails, Follow and the Credit tab stay hidden.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `opinions` preset.
- **Core unit:** the BC mean on hand profiles (symmetric, asymmetric, opinion-dependent with HK's worked example); the prefix-sum path agrees with direct summation; the reach always includes self; simultaneous vs serial (serial sees changed opinions); lattice neighbors on the torus; regular start endpoints; stability and cluster tolerance; splits and one-sided splits on hand profiles; `stable_at` and the stop; the view (lines, gray bands, scrolling) and Inspect; keyframes restore the history; live and reset fields; degenerate configs (n = 2, ε 0 and 1, εl = 0, m 0 and 1).
- **Web:** schema groups, charts, the Compare entry, a sweep over an `opinions` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, the stop, Compare, recording, Experiments, every existing scenario.

## Docs

README: a Bounded Confidence section (the rule, stated choices, switches and sources, presets and what they reproduce, sweeps); roadmap: Milestone 16 done.
