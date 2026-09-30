# SugarScape Milestone 30 — The Emergence of Firms — Design

**Date:** 2026-09-30
**Builds on:** the milestone 1–29 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–29, the preset titles of `crates/sugarscape-core/src/titles.rs`, and milestones 27–29's method (one fit rule fixed before measuring; where a source's text and its figures or tables disagree, both are reported; decision rules written before measuring).
**Source texts** (local copies):
- Robert L. Axtell, "The Emergence of Firms in a Population of Agents: Local Increasing Returns, Unstable Nash Equilibria, and Power Law Size Distributions," Brookings CSED Working Paper 3 (1999), `papers/firms/axtell-1999-emergence-of-firms.pdf` (A99 below; 108 pages, with a text layer; page numbers are the printed ones, PDF page = printed + 7).
- The follow-up: Robert L. Axtell, "Endogenous Dynamics of Firms and Labor with Large Numbers of Simple Agents" (LEM working paper; dated 2014, circulated 2013), `papers/firms/axtell-2013-lem-endogenous-dynamics-of-firms-and-labor.pdf` (A13 below; 51 pages). Its 2018 handbook chapter (*Handbook of Computational Economics* 4, ch. 3) is wanted but not in hand.
- For reference only: Furtado & Eberhardt's Python replication of A13 (CoMSES 4948, AFL-3.0) — not read for rules.

## Goal

Axtell's model of firm formation as one model kind, `firms` ("The Emergence of Firms"), a full citizen of the playground: agents with Cobb–Douglas preferences for income and leisure choose their effort in teams with increasing returns and equal shares, moving between their own firm, a start-up and their friends' firms; firms form, grow, fill with free riders and collapse. Every rule of A99 §3 and every variation of §4 (Tables 3–13) as a named switch, with A13's parameterization as a preset; the §2 analytics (Nash efforts, optimal and maximum stable sizes, Table 1) as tested functions; titled presets, one measured sweep per §4 table, and a survey of A99's claims — including where its own figures, tables and text disagree.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited.
- **Faithful where the source is specific**; where silent, the literal reading is the default and the alternatives are named switches; every choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only; the effort optimum by `+ − × ÷` and `sqrt`; non-integer powers via the project's portable helpers).
- **Truthful descriptions and titles**; the paper's numbers are quoted beside ours, never in place of them.
- **The survey's decision rules are written before measuring**, and every rule set after seeing a number says so.

## Source summary

- **Production and preferences (A99 §2.1, §3.1):** a firm's output is O = aE + bE^β over its members' total effort E (a = b = 1, β = 2 in the base case); output is shared equally; agent i's utility is (O/N)^θᵢ (1 − eᵢ)^(1−θᵢ), θ ~ U[0, 1]; effort eᵢ ∈ [0, 1].
- **Best effort (§2.2):** closed forms (5) for b > 0, β = 2 and (6) for b = 0, given the others' effort E₋ᵢ; the computational model uses "a line search over the feasible range of efforts" (fn 31), resolution unstated.
- **The computational model (§3.1, Table 2):** 1 000 agents, each with ν = 2 friends assigned at random at t = 0; "a time period defined as 1000 agents being active", agents drawn at random ("random activation", binomially distributed activations); an activated agent "looks up the size and output of its firm as well as its own previous period effort level" and chooses its utility-maximizing effort, then repeats the calculation for starting a firm alone and for joining each friend's firm, and takes the best; all agents start alone. Production and pay happen once a period.
- **The §2 analytics:** Nash equilibria in homogeneous groups; optimal and maximum stable sizes (for θ = 0.7: stable to 6, optimal 5, Table 1); instability beyond a size set by the most productive members.
- **The §3 results (one "typical" realization each):** a power-law firm-size distribution, p(s) ∝ s^−(1+µ) with µ = 1.28 (OLS on the log-log probability mass function, dropping size 1 and frequencies below 10⁻⁵; empirical comparisons 1.23 and 1.11); output exponent 0.88; productivity 0.58 s^1.15; Laplace growth rates; σ_r ∝ s^−γ, γ = 0.174 ± 0.004; mean lifetime 23.4 (sd 27.1) over about 10⁶ firms, lifetime linear in log rank (slope −70); time series of firm counts, sizes, effort, output, income and utility; the firm life cycle and agent welfare by θ.
- **§4 (Tables 3–13), µ only, single estimates, no standard errors:** β, b, preference distributions (including CES), fixed friends ν, random firms ν, loyalty λ, sticky effort, groping, seniority ("hyperbolic") shares, base pay plus bonus, hiring standards; the introduction's population size, uniform activation and initial conditions; §4.1's random behavior.
- **A13:** 120 million agents; per-firm a ~ U[0, ½], b ~ U[¾, 5/4], β ~ U[3/2, 2]; ν ~ U[2, 6] on an Erdős–Rényi graph; 4 % of agents activated a period (a month); effort from the previous period's E₋ᵢ; Zipf's exponent α ≈ 1.06.

## The paper against itself (read in planning)

A99's figures and text disagree on basic levels, and several tables contradict their text:

- **Firm counts, mean size and lifetime (X9):** figure 10 shows about 150–170 firms (mean size about 6); figure 12's inset about 3; the text "about 4"; a pure s^−2.28 distribution gives 2.5–2.8; Little's law (about 45 births a period × 23.4) gives about 1 050 firms, impossible with 1 000 agents.
- **Levels (X14):** with Table 2's parameters the all-alone start already gives total output ≈ 934, income ≈ 0.93 and utility ≈ 0.83; figures 14–16 start near 450, 0.5 and 0.6.
- **Table 11's noise (X3):** shares are normalized, so 2^−(i+k) gives the same shares as 2^−i; the table reports four different µ (0.89–1.07) for one model — an estimate of run-to-run noise.
- **Text against tables:** µ "increases" with β (Table 3: it falls); groping "more pronounced" than stickiness (Table 10: less, at every β); the conclusion's homogeneous case (not reported; the one reported, θ = 0.75, gives more large firms); Table 13's caption ("target output") describes no rule in the paper — the table is hiring standards; the text's eigenvalue −0.552 is not Table 1's −0.368; the count of "195,955,200" models is 65,318,400.

## Measured in planning

A throwaway prototype (Python) of the base case, 1 000 agents, 3 000 periods (500 burn-in), the paper's OLS fit:

- **µ does not come out at 1.28 under any reading:** 1.70, 1.90, 1.91 (seeds 1–3; others' effort live, random activation); 2.24 with others' effort from last period's output (the literal reading); 1.54 with uniform activation; 1.45–1.64 with a coarse effort grid (10 steps); a 100-step grid equals the closed form (1.70). Seeds alone move µ by about 0.2.
- **Firms churn far faster than the paper's:** about 110 births a period (the paper's figure 10: about 45); mean lifetime 2–3 periods under every counting (all firms 3.2; firms that ever had two members 3.8; three 4.6) against 23.4; mean size 2.6–3.5; about half the firms are singletons; the largest firm reaches 150–350.
- **Levels:** total output 800–930, consistent with the all-alone start (934), not with figures 14–16.
- **The model's qualitative story holds:** firms form, grow to a few hundred members, and collapse as free riders accumulate.

The survey reproduces each of these with the implementation, over more seeds and longer runs.

## Architecture

Model kind `firms` ("The Emergence of Firms"): `ModelKind::Firms`, `ModelConfig::Firms(FirmsConfig)` tagged `"model": "firms"`, a `FirmsWorld` implementing `Model`. Code in `crates/sugarscape-core/src/firms/`: `config.rs` (parameters, reading enums, validation, schema), `effort.rs` (the optimum — closed form, line search, sticky window, groping — and the §2 analytics: Nash efforts, optimal and maximum stable sizes, the Jacobian's k), `pay.rs` (equal shares, seniority shares, base pay plus bonus, and a firm's output), `world.rs` (`FirmsWorld`: agents, firms, activation, choice, production, statistics, rendering, Inspect), `fit.rs` (the paper's OLS µ, a maximum-likelihood µ, the Laplace growth fit, γ, lifetimes), `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`. One tick is one period.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `agents` | 1 000 | reset | A |
| `a`, `b`, `beta` with `a_max`, `b_max`, `beta_max` | 1, 1, 2; maxima 0 | live | fixed, or (a maximum above the value) drawn uniformly per firm at founding between the two; A99 §4.2 draws b per firm; β per firm is our reading |
| `preferences` | `uniform` | reset | θ: `uniform` [0, 1], `middle` U[0.25, 0.75], `triangular` (mode 0.5), `triangular_high` (mode 0.75), `normal` (truncated, mean 0.5, variance ½), `beta` (Beta(1, 2) — the order is our reading), `fixed` (all `theta`), `ces` (CES with δ ~ U[0, 1] and ρ ~ U[`rho`, `rho_max`]) |
| `theta`, `rho`, `rho_max`, `ces_sign` | 0.75, −1, 0, `text` | reset | the fixed θ; CES's ρ range; `text` or `printed` (the printed CES contradicts the text's limits) |
| `network` | `friends` | reset | `friends` (ν fixed random agents, their current firms) or `random_firms` (ν firms drawn uniformly each activation, not one's own) |
| `neighbors`, `neighbors_max` | 2, 0 | reset | ν, or drawn per agent between the two |
| `activation` | `random` | live | `random` (with replacement) or `uniform` (each agent once, in random order) |
| `activation_rate` | 1.0 | live | activations a period ÷ agents (A13: 0.04) |
| `others_effort` | `last_period` | live | what an agent takes as the others' effort: `last_period` (inferred from last period's output: the text's reading, and A13's) or `live` (their current efforts) |
| `effort_search` | `exact` | live | `exact` (the optimum; the limit of a fine line search) or `grid` with `grid_steps` |
| `effort_window` | 1.0 | live | sticky effort: new effort within ±window/2 of the current (A99: 0.10) |
| `groping` | false | live | sample one effort U[0, 1]; keep it if it does better |
| `loyalty`, `loyalty_max` | 0, 0 | reset | λ, or drawn per agent between the two: moves only after wanting to more than λ times (the counter resets on a move) |
| `pay` | `equal` | live | `equal`, `seniority` (shares ∝ `seniority_base`^−rank; rank 1 is the longest-serving), or `base` (base pay per `base_pay` plus an equal bonus max(0, (O − ΣΦ)/N)) |
| `seniority_base`, `base_pay`, `base_share` | 2, `own`, 0.5 | live | p; base pay from each agent's own singleton income, the median agent's (θ = ½) or the mean agent's, × share |
| `hiring`, `hiring_max` | 0, 0 | live | φ, or drawn per firm at founding between the two: a firm admits only θ ≥ φ·θ of its longest-serving member; a rejected option is unavailable |
| `random_behavior` | `none` | live | §4.1: `none`, `choices` (stay, move or start up at random, a random friend's firm, then the best effort) or `effort` (the best option at a random effort) |
| `initial` | `alone` | reset | `alone`, `random_groups` (sizes drawn from a geometric distribution with mean 4 — our reading) or `one_firm` |
| `burn_in`, `sample_every` | 500, 1 | live | the size and lifetime statistics' start and sampling interval (A99: longer than the longest lifetime) |
| `stop_at` | 5 000 | live | `finished()` after this many periods (0: never) |

## Step (one period)

`activation_rate × agents` activations. An activated agent computes, with the others' effort as `others_effort` says: its best effort and utility if it stays; starting alone (N = 1, E₋ᵢ = 0; the firm's a, b and β drawn before deciding); and joining each option of its network (N + 1, the target's effort) that admits it. It takes the best, staying on ties; loyalty may keep it. At the end of the period each firm produces and pays; firms without members are gone. Lifetimes are counted from founding to the last member's departure, the initial firms from period 0; the survey reports lifetimes with and without the initial firms and the firms that never had a second member.

## Statistics

`SERIES`: `firms`, `births`, `deaths`, `mean_size`, `largest`, `singletons` (share of firms of one), `effort`, `output`, `income`, `utility` (means over agents), `largest_output_share`, `mu` (the paper's OLS on the sizes sampled since `burn_in`), `mu_mle` (the discrete maximum-likelihood exponent for sizes ≥ 2), `lifetime` (mean lifetime of firms that died since `burn_in`), `period`. The growth-rate distribution, σ_r(s), γ, the output exponent and productivity are computed by `fit.rs` from the world's records (for the survey and Inspect), not as per-tick series.

## Views

- **Firms (A99's Animation 1):** each firm a row of cells, its longest-serving member first, rows sorted by founding, so a long row is a big firm; rows wider than the frame are truncated at its edge, as in the paper.
- **Beside it:** the firm-size distribution on log-log axes, with the current OLS line.
- **Color modes:** **Founder** (the paper's red and blue), **θ**, **Effort**, **Income**.
- **Inspect:** a firm (size, output, age, members' θ and efforts, free riders) or an agent (θ, effort, utility, income, tenure, friends' firms).
- **Charts:** Firms (`firms`, `births`, `deaths`); Sizes (`mean_size`, `largest`); Effort and pay (`effort`, `income`, `utility`, `output`); Scaling (`mu`, `mu_mle`). Time axis: Periods.
- **Compare:** "Last period's effort vs live effort — The Emergence of Firms (Compare)".
- **Experiments default:** `mu` against `beta` 1.7–2.1.

## Presets

About 18, measured and titled: `firms-base` (the literal base case), `firms-live` (others' effort live), `firms-uniform` (uniform activation), `firms-beta-17`, `firms-beta-21`, `firms-b-15`, `firms-b-random`, `firms-theta-075`, `firms-friends-10`, `firms-random-firms-10`, `firms-loyal-10`, `firms-sticky`, `firms-groping`, `firms-seniority-5`, `firms-base-pay-80`, `firms-hiring-100`, `firms-random-choices`, `firms-2013` (A13's parameterization at 10 000 agents).

## Experiments and CLI

Sweeps, each 10 seeds with seeds and horizons in its description: `firms-beta` (Table 3), `firms-b` (Table 4), `firms-preferences` (Table 5), `firms-friends` (Table 6), `firms-random-firms` (Table 7), `firms-loyalty` (Table 8), `firms-sticky` (Table 9, by β), `firms-groping` (Table 10, by β), `firms-seniority` (Table 11, including the paper's four equivalent rows), `firms-base-pay` (Table 12), `firms-hiring` (Table 13), `firms-readings` (activation × others' effort × effort search), `firms-population` (1 000 to 100 000 agents). The CLI names the stop `(its last period)`.

## Survey

A `firms` claims module; every decision rule written before measuring:

- **§2 (exact):** Table 1 to 3 dp; the optimal and maximum stable sizes by θ; (14)'s bound.
- **§3:** µ = 1.28 (the paper's fit; within 0.15, our estimate of its noise); µ against the empirical 1.23 and 1.11; the output exponent 0.88; productivity near constant returns; Laplace beats Gaussian for growth rates; γ = 0.174 within its reported ± twice; mean lifetime 23.4; lifetimes linear in log rank; mean size about 4; the time-series levels (firm counts, the largest firm near 200, output, income, utility).
- **§4:** each table as a claim about direction and order over 10 seeds, with the paper's absolute numbers reported beside ours (µ falls as β, b, ν and λ rise; stickiness and groping lower µ, stickiness more; heterogeneous b behaves like b = 1.25; homogeneous θ = 0.75 gives more large firms; the introduction's invariances — population size, uniform activation, initial conditions).
- **Ours:** Table 11's four equivalent rows are one model (identical runs); the paper's internal contradictions (X9, X14) against our levels; the literal reading of others' effort against the live one; the seed spread of µ; the maximum-likelihood µ beside the OLS.

## Page

The presets menu gains **The Emergence of Firms**; the Rules panel is generated from the schema in groups Agents, Production, Preferences, Network, Decisions, Pay and Measurement.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `firms` preset.
- **Core unit:** the closed form against a brute-force line search; Table 1 and the §2 sizes; the Jacobian's k; the two worked examples of A99 pp. 11–14; each pay rule by hand; hiring, loyalty, stickiness and groping on small hand-built worlds; activation counts; birth, death and lifetime accounting; the OLS and maximum-likelihood fits on synthetic power laws; the Laplace fit; degenerate configs; the view and Inspect; keyframes.
- **Web and browser:** as for the earlier models.

## Docs

README: an Emergence of Firms section (the model, how the paper was read, its contradictions, switches, presets, sweeps, findings). `docs/papers.md`: the milestone's row; the Queue's first entry removed; roadmap: Milestone 30 done.
