# SugarScape Milestone 27 — Altruistic Punishment — Design

**Date:** 2026-09-28
**Builds on:** the milestone 1–26 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds, the literal-default-plus-named-switch pattern of milestones 11–26, and the preset titles of `crates/sugarscape-core/src/titles.rs`.
**Source texts** (local copies):
- Robert Boyd, Herbert Gintis, Samuel Bowles and Peter J. Richerson, "The evolution of altruistic punishment", *PNAS* 100(6): 3531–3535 (2003), `papers/punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` (BGBR below).
- The critique: Daniel B. Cooney, "Exploring the Evolution of Altruistic Punishment with a PDE Model of Cultural Multilevel Selection", arXiv:2405.18419v3 (2024; *Bull. Math. Biol.* 2025), `papers/punishment/cooney-2024-arxiv-altruistic-punishment-pde-multilevel-selection.pdf` (Cooney below).
- Reference only: Marco Janssen's NetLogo replication, CoMSES 2223 v1.2.0 (GPL-3.0), `papers/punishment/janssen-comses-2223-netlogo/`. Read for its readings of the unstated rules; no code or text is copied.

## Goal

Boyd, Gintis, Bowles and Richerson's model of cultural group selection with altruistic punishment as one model kind, `punishment` ("Altruistic Punishment"), a full citizen of the playground: Figs. 1–4 (cooperation against group size, with and without punishment, for conflict rates, mixing rates, the cost of being punished, and fixed against variable punishing costs), the sensitivity paragraph (mutation, error, number of groups), the three structural variants the text describes (a per-capita benefit with conflict decided by payoffs, continuous traits, a ring of groups without extinction), and Cooney's claims about payoff-decided conflict; every detail the text leaves open — above all the payoff baseline that imitation needs — a named switch or a stated choice, and every claim measured.

## Non-negotiable constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session and sweep reads and runs as before.
- **Faithful where the source is specific** (quoted below); where silent, the choice is stated here and in the module docs.
- **One engine path; deterministic; portable** (native and WASM fingerprints identical; `u32` ranges and `f64` samples only). The tanh victory rule is computed with `crate::portable::exp_neg`, never `f64::tanh`.
- **Truthful descriptions and titles:** each preset and sweep says what it measurably reproduces and what it does not.

## Source summary

- **The game:** "a large population is divided into groups of size n … contributors incur a cost c … punishers who cooperate and then punish each defector in their group, reducing each defector's payoff by p/n at a cost k/n to the punisher."
- **The model:** "There are N groups. Local density-dependent competition maintains each group at a constant population size n. … During the first stage, contributors and punishers cooperate with probability 1 − e and defect with probability e. Cooperation reduces the payoff of cooperators by an amount c … cooperation has no effect on the individual payoffs of others, but does reduce the probability of group extinction. Defectors always defect. During the second stage, punishers punish each individual who defected during the first stage. After the second stage, individuals encounter another individual from their own group with probability 1 − m and an individual from another randomly chosen group with probability m. An individual i who encounters an individual j imitates j with probability Wj/(Wj + Wi), where Wx is the payoff of individual x in the game, including the costs of any punishment received or delivered."
- **Conflict:** "In each time period, groups are paired at random, and with probability ε, intergroup conflict results in one group defeating and replacing the other group. The probability that group i defeats group j is 1/2(1 + (dj − di)), where dq is the frequency of defectors in group q."
- **Mutation:** "with probability μ individuals of each type spontaneously switch into one of the two other types."
- **Methods:** "In all simulations there were 128 groups. Initially one group consisted of all altruistic punishers and the other 127 groups were all defectors. … Simulations were run for 2,000 time periods. The long run average results plotted in Figs. 1–4 represent the average of frequencies over the last 1,000 time periods of 10 simulations."
- **Parameters and their calibration:** c = k = 0.2 "so that traits with this cost advantage would spread in 50 time periods"; p = 0.8; e = 0.02; m = 0.01 "so that … passive diffusion will cause two neighboring groups that are initially as different as possible to achieve the same trait frequencies in ≈50 time periods"; μ = 0.01 "so that the long run average frequency of an ordinary adaptive trait with payoff advantage c is ≈0.9"; ε = 0.015 (an extinction rate of ≈0.0075, "because only one of the two groups entering into a conflict becomes extinct").
- **Figures** (group sizes 4, 8, 16, 32, 64, 128, 256): Fig. 1 — cooperation (contributors plus punishers) without punishment (p = k = 0; a) and with (b), for three conflict rates. The legend reads 0.0075, 0.015, 0.03; **the caption reads 0.075, 0.015, 0.003.** "Group selection is ineffective unless groups are quite small … group selection can maintain cooperation in substantially larger groups." Fig. 2 — mixing rates 0.002, 0.01, 0.05: "when the migration rate increases, levels of cooperation fall precipitously … at higher rates of mixing, cooperation does not persist in the largest groups." Fig. 3 — p = 0.8 and 0.4: "Lower values of p result in much lower levels of cooperation." Fig. 4 — no punishment, a fixed punishing cost "equal to the cost of cooperating (c)", and the variable cost: "Punishment does not aid in the evolution of cooperation when the costs born by punishers are fixed."
- **Sensitivity:** "Decreasing the mutation rate substantially increases the long run average levels of cooperation … Increasing e, the error rate, reduces the long run average amount of cooperation. Reducing the number of groups, N, adds random noise to the results."
- **Variants:** a per-capita benefit "b/n for each other group member" with extinction "proportional to the difference between warring groups in average payoffs including the costs of punishment … For reasonable values of b (2c, 4c, and 8c), the results of this model are qualitatively similar"; continuous traits ("An individual with cooperation value x behaves like a cooperator with probability x … New mutants are uniformly distributed. The steady-state mean levels of cooperation in this model are similar to the base model"); a ring ("Populations are arranged in a ring, and individuals imitate only individuals drawn from the neighboring two groups. Cooperative acts produce a per capita benefit b/n … We could find no reasonable parameter combination that led to significant long run average levels of cooperation in this last model").
- **Discussion:** "cooperation is sustained in groups on the order of 100 individuals."
- **Never stated:** the payoff to which the game's costs are added (imitation needs Wx > 0, and a defector among punishers loses up to p); whether a punisher who errs punishes, or punishes itself; whether imitation updates at once or in turn; what replaces a defeated group; whether d counts defector types or defecting acts; the form of the payoff-decided victory probability.
- **Cooney:** under victory by the "difference in fraction of cooperative individuals (as assumed by Boyd and coauthors)", results match the stochastic model's dependence on costs and punishment; under victory by average payoff normalized by the largest possible difference (his eq. 3.18), "increasing the cost of punishing defectors can increase the level of altruistic punishment at steady state (albeit at a lower average payoff)" and "a non-monotonic dependence of long-time average payoff on the strength of punishment" — which, by his Remark 6.1, appears only for the globally normalized and Tullock rules, not the Fermi (tanh) rule.
- **Janssen's readings** (recorded, not copied): payoff 1 plus a benefit 0.5 × the share cooperating; every group challenges a random opponent with probability ε (about twice the stated conflict rate); d counts acts; imitation in turn, a model possibly itself; erring punishers punish, themselves included; a defeated group is replaced one for one by copies of the winners.

## Measured in planning

A throwaway prototype (Rust) of the rules below; 10 runs each, 128 groups, 2 000 periods, the mean cooperation over the last 1 000; group sizes 4, 8, 16, 32, 64, 128, 256. The survey reproduces each with the implementation.

- **Fig. 1b (base, baseline 1):** ε 0.0075: 0.76, 0.76, 0.66, 0.42, 0.09, 0.07, 0.07; ε 0.015: 0.81, 0.83, 0.78, 0.71, 0.46, 0.15, 0.08; ε 0.03: 0.88, 0.89, 0.87, 0.83, 0.78, 0.71, 0.43. The paper's (read from the figure): 0.78 … 0.47, 0.18, 0.10; 0.83 … 0.76, 0.61, 0.58; 0.87 … 0.85, 0.78, 0.66. The shape holds; cooperation collapses a group size or two sooner.
- **Fig. 1a:** ε 0.0075, 0.015, 0.03 at n 4: 0.48, 0.59, 0.74 (paper about 0.63, 0.76, 0.88), all 0.09 from n 32 — the shape and floor hold, a little lower in small groups.
- **The caption's rates** (0.075, 0.003): 1a 0.89 and 0.41 at n 4; 1b 0.94 … 0.77 and 0.71 … 0.07 — farther from the figure than the legend's.
- **The unstated baseline decides how far punishment reaches.** Fig. 1b at baseline 0.5, 1, 2, 3, 4, 5, at n 256: 0.03, 0.08, 0.19, 0.50, 0.64, 0.67. But Fig. 1a's floor rises with it (0.09, 0.18, 0.33 at n 256 for baselines 1, 2, 4) and ε stops mattering (baseline 4: 0.59, 0.64, 0.72 at n 256 for the three rates). No baseline reproduces Figs. 1a and 1b together. Baseline 1 is the one their calibration implies: a trait with advantage c = 0.2 goes from 10 % to 90 % in about 48 periods (the logistic rate c/(2W + c)).
- **The mixing calibration is off.** With m = 0.01, two groups' difference shrinks by about 1 % a period: 60 % remains after 50 periods, 10 % after 230. Reaching "the same trait frequencies in ≈50 periods" needs m ≈ 0.05.
- **Readings:** sequential imitation 0.61, 0.24, 0.07 at n 64, 128, 256 (against 0.46, 0.15, 0.08); d by acts, the split refill, erring punishers that do not punish or punish themselves, and an all-defector start change nothing beyond noise. Every group challenging (Janssen's pairing, about twice the conflict): 0.88, 0.89, 0.86, 0.83, 0.79, 0.53, 0.54 — near the paper's ε 0.015 curve. Janssen's readings together: 1b 0.83, 0.84, 0.84, 0.84, 0.81, 0.77, 0.45; 1a 0.76, 0.66, 0.43, 0.14, 0.09 … — the closest to the figure of any reading.
- **Fig. 2:** 2a m 0.002, 0.05 at n 4: 0.66, 0.31; 2b at n 32: 0.71 (m 0.002), 0.71 (0.01), 0.26 (0.05); at n 256: 0.26, 0.08, 0.07. As stated, but the paper's 2b keeps 0.70 at n 256 for m 0.002.
- **Fig. 3:** p 0.4: 0.67, 0.51, 0.22, 0.08 … — much lower, as stated.
- **Fig. 4:** fixed cost c: 0.69, 0.49, 0.21, 0.09 … — no better than no punishment in large groups, as stated (the paper's is higher than none in small groups; here too).
- **Sensitivity (n 32):** μ 0.001, 0.005, 0.01, 0.02, 0.05: 0.96, 0.82, 0.71, 0.56, 0.64 — lower μ raises it substantially, as stated. e 0, 0.02, 0.05, 0.1, 0.2: 0.71, 0.71, 0.65, 0.56, 0.38 — as stated. N 8, 16, 32, 64, 128: 0.59, 0.61, 0.67, 0.71, 0.71 — fewer groups lower the mean, not only add noise.
- **The benefit with payoff conflict** (normalized, b 0.4, 0.8, 1.6): 0.58, 0.60, 0.60, 0.40, 0.11, 0.08, 0.07; 0.57 … 0.62, 0.39, 0.13, 0.08; 0.47 … 0.60, 0.55, 0.19, 0.10 — cooperation in groups up to about 32–64, falling at 128; qualitatively like the base model.
- **Continuous traits are not similar:** with punishment 0.70, 0.84, 0.91, 0.95, 0.95, 0.95, 0.93 — rising with group size, where the base model falls to 0.08. Uniform mutants keep the mean punishment near 0.5, and p × 0.5 exceeds c: cooperation pays within groups. Starting from all defectors it collapses from n 128 (0.06). Without conflict: 0.58, 0.71, 0.78, 0.46, 0.09 …
- **The ring:** b 0.4, m 0.01: 0.52, 0.48, 0.37, 0.18, 0.09 …; b 1.6: 0.29 …; b 4: 0.15 … — half cooperating in groups of four; more benefit lowers it.
- **Cooney:** under the normalized rule (b 2c, n 32), mean payoff falls slightly from p 0 to 0.5 (1.010 to 1.002) and then rises (1.13 at 2.4) — a shallow dip, of the kind he describes; with a fixed cost 0.1, 1.012 to 0.999 at p 0.6, then up. Under tanh (s 10, b 4c) the payoff is flat to rising. A higher k lowers the punisher share at every setting tried (0.50, 0.44, 0.24, 0.02, 0.01 for k 0.05–0.8) — his "increasing the cost of punishing … can increase the level of altruistic punishment" does not appear.

## Architecture

Model kind `punishment` ("Altruistic Punishment"): `ModelKind::Punishment`, `ModelConfig::Punishment(PunishmentConfig)` tagged `"model": "punishment"`, a `PunishmentWorld` implementing `Model`, schema, `SERIES`, presets, titles and golden entries. Code in `crates/sugarscape-core/src/punishment/` (`config.rs`, `world.rs`, `stats.rs`, `view.rs`, `presets.rs`, `mod.rs`).

Each agent carries two traits, `cooperate` and `punish`, in [0, 1] (stored as `f64`). Discrete agents are the corners: a defector (0, 0), a contributor (1, 0), a punisher (1, 1).

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `groups` | 128 | reset | N (2–512) |
| `size` | 32 | reset | n (2–512); the paper sweeps 4–256 and gives no base value |
| `cost` | 0.2 | live | c: the cost of cooperating |
| `punish_cost` | 0.2 | live | k: a punisher pays k/n per defector |
| `fine` | 0.8 | live | p: a defector pays p/n per punisher |
| `punishing` | `variable` | live | `variable` (k/n per defector), or `fixed` (a flat `fixed_cost` each period, whatever the group; Fig. 4) |
| `fixed_cost` | 0.2 | live | the fixed cost (Fig. 4: c) |
| `benefit` | 0 | live | b: each cooperative act gives b/n to every other member (0 in the base model) |
| `baseline` | 1 | live | the payoff the game's costs and benefits are added to; payoffs below 0 count as 0 |
| `error` | 0.02 | live | e |
| `mixing` | 0.01 | live | m |
| `mutation` | 0.01 | live | μ |
| `conflict` | 0.015 | live | ε |
| `pairing` | `paired` | live | `paired` (groups paired at random, each pair fights with probability ε: the text), or `challenge` (each group, in random order, challenges a random group not yet fighting with probability ε: Janssen) |
| `victory` | `defectors` | live | `defectors` (½(1 + dⱼ − dᵢ): the text), `payoff` (½ + ½(Ḡᵢ − Ḡⱼ)/(G_max − G_min): Cooney 3.18), or `tanh` (½ + ½ tanh(s(Ḡᵢ − Ḡⱼ)): Cooney's Fermi rule) |
| `sensitivity` | 10 | live | s for `tanh` |
| `counted` | `types` | live | d counts defector types (1 − mean `cooperate`) or defecting `acts` this period (Janssen) |
| `erring` | `others` | live | a punisher who defects: still punishes the other defectors (`others`), punishes nobody (`none`), or punishes itself too (`self`: Janssen) |
| `imitation` | `together` | live | all agents imitate from the period's starting traits (`together`), or in turn, later agents seeing earlier changes (`in_turn`: Janssen) |
| `refill` | `copy` | live | a defeated group becomes a copy of the winners (`copy`), or both groups are refilled by drawing the winners' members with replacement (`split`) |
| `traits` | `discrete` | reset | `discrete` (three types), or `continuous` (traits in [0, 1]; mutants uniform) |
| `structure` | `groups` | reset | `groups`, or `ring` (no conflict; migrants drawn only from the two neighboring groups) |
| `start` | `one_punisher_group` | reset | `one_punisher_group` (the text), or `all_defectors` |
| `window` | 1000 | live | `long_run` averages cooperation over periods after `stop_at` − `window` (all periods when `stop_at` is 0 or ≤ `window`) |
| `stop_at` | 2000 | live | `finished()` at this period (0: never) |

G_max − G_min is the range of the expected group payoff over all compositions of contributors, defectors and punishers (Cooney's eq. 2.1 with the current c, k, p, b and cost rule), computed exactly from its vertices and edges (it is quadratic in the composition) whenever the config changes; a range of 0 counts as a draw (½).

## Step (one period)

1. **Acts.** Each agent cooperates with probability `cooperate` × (1 − e) (discrete: contributors and punishers with 1 − e; defectors never), and is a punisher this period with probability `punish` (discrete: punishers always; under `erring: none` only if it cooperated).
2. **Payoffs.** W = baseline − c (if it cooperated) + b × (cooperators other than itself)/n − p × (punishers other than itself; with `self`, including itself)/n (if it defected) − k × (defectors other than itself; with `self`, including itself)/n (a punisher, `variable`) or − `fixed_cost` (a punisher, `fixed`); floored at 0.
3. **Imitation.** Each agent picks a model: with probability 1 − m a random other member of its own group, otherwise a random member of a random other group (`ring`: of one of the two neighboring groups, each with probability ½). It copies the model's traits with probability Wⱼ/(Wⱼ + Wᵢ) (½ when both are 0).
4. **Conflict** (`groups` only). Pairs are formed (`paired` or `challenge`); each fighting pair's winner is drawn by `victory`, with d and Ḡ from this period's acts and payoffs; the loser is refilled (`copy` or `split`).
5. **Mutation.** With probability μ an agent switches to one of the two other types, each with probability ½ (`continuous`: new traits drawn uniformly from [0, 1]²).
6. **Statistics** are recorded after mutation.

## Statistics

`SERIES`: `cooperation` (the mean `cooperate`: contributors plus punishers), `contributors`, `punishers`, `defectors` (discrete shares; continuous: the mean of `cooperate` × (1 − `punish`), `cooperate` × `punish`, 1 − `cooperate`), `punishment` (the mean `punish`), `acts` (the share who cooperated this period), `payoff` (the mean payoff), `conflicts` and `extinctions` (this period), `spread` (the standard deviation of groups' cooperation), and `long_run` (the mean `cooperation` over the window so far; NaN before it).

## Views

- **Groups** (the mosaic): the N groups in a grid of ⌈√(2N)⌉ columns, each group a near-square block of its n agents, one cell each; a thin frame around a group that lost a conflict this period.
- **Cooperation over time** (below): the last 300 periods of `cooperation` and `punishment`.
- **Color modes:** **Type** (contributors blue, punishers green, defectors red; continuous: blended by the two traits), **Acts** (cooperated, defected; punishers marked), **Payoff**, **Group** (each group shaded by its defector share).
- **Inspect:** an agent's traits, act, payoff and group; a group's composition, mean payoff and last conflict; a period of the time strip.
- **Charts:** Types (`contributors`, `punishers`, `defectors`); Cooperation (`cooperation`, `long_run`, `acts`); Payoff (`payoff`); Conflict (`conflicts`, `extinctions`, `spread`). Time axis: Periods.

## Presets

Titles follow `titles.rs`'s style; drafts, to be measured.

| Preset | Title | Setup |
|---|---|---|
| `bg-base` | Punishers keep most of a population of groups of 32 cooperating | the kind's default |
| `bg-none` | Without punishment, groups of 32 fall to defection | p = k = 0 (Fig. 1a) |
| `bg-large` | Groups of 128: punishment no longer holds | n 128 |
| `bg-weak` | A fine only twice the cost, and cooperation fades | p 0.4 (Fig. 3) |
| `bg-fixed` | Punishers who pay whether or not anyone defects | `fixed` (Fig. 4) |
| `bg-mixing` | More mixing between groups, and cooperation falls | m 0.05 (Fig. 2) |
| `bg-benefit` | Cooperation benefits the group, and groups fight over payoffs | b 4c, `payoff` |
| `bg-continuous` | Cooperate and punish by degrees | `continuous` |
| `bg-ring` | A ring of groups with no wars | `ring`, b 2c |
| `bg-janssen` | Janssen's readings of the gaps together | benefit 0.5, `challenge`, `acts`, `in_turn`, `self` |

**Compare entry:** "With vs without punishment — Altruistic Punishment (Compare)": `bg-base` and `bg-none`.

## Experiments and CLI

Seeds 10 and 2 000 periods unless recorded otherwise; the metric is the final `long_run`; the x-axis is the group size (4, 8, 16, 32, 64, 128, 256) unless stated.
- `bg-fig1a`, `bg-fig1b`: without and with punishment, series ε 0.0075, 0.015, 0.03 (the legend).
- `bg-fig1-caption`: series ε 0.075, 0.003 (the caption), with and without punishment.
- `bg-fig2a`, `bg-fig2b`: series m 0.002, 0.01, 0.05.
- `bg-fig3`: series p 0.4, 0.8.
- `bg-fig4`: series none, fixed, variable.
- `bg-baseline`: series baseline 1, 2, 4, with and without punishment.
- `bg-readings`: series `paired`, `challenge`, Janssen's readings together.
- `bg-sensitivity`: against μ, e and N at n 32 (three sweeps: `bg-mutation`, `bg-error`, `bg-groups`).
- `bg-benefit`: series b 2c, 4c, 8c under `payoff`.
- `bg-continuous`: series with and without punishment, and from all defectors.
- `bg-ring`: series b 2c, 8c.
- `bg-cooney-fine`: the final mean `payoff` (over the window) against p, series `payoff` and `tanh`, b 2c, n 32; `bg-cooney-cost`: the final `punishment` against k.
The CLI names the stop `(its last period)`.

## Survey

A `punishment` claims module, each claim with its decision rule written before measuring: Fig. 1a (cooperation only in small groups), Fig. 1b (punishment sustains it in much larger groups), more conflict more cooperation, the legend's against the caption's rates, Fig. 2 (more mixing, less), Fig. 3, Fig. 4, the calibrations (a trait with advantage c spreading in 50 periods at baseline 1; two groups equalizing in 50 periods at m 0.01), the sensitivity paragraph (μ, e, N), the three variants, "groups on the order of 100", Cooney's two claims, the baseline (no one value reproduces Figs. 1a and 1b together), and Janssen's readings. Claims that fail are reported, and the descriptions, titles and README say so.

## Page

The presets menu gains an **Altruistic Punishment** group and the Compare entry; the Rules panel is generated from the schema in groups Groups, Game, Imitation, Conflict, Variants and Stopping. Worker host, Max speed, timeline, links, sessions, Compare, recording and Experiments work unchanged.

## Testing

- **Golden/legacy:** existing entries untouched; new entries for every `punishment` preset; titles for every preset.
- **Core unit:** the start; acts with and without error; payoffs on hand-built groups for every `erring` and `punishing` rule and the benefit; the floor at 0; imitation's probability (a two-agent group, many draws); `together` against `in_turn`; migrants from other groups only, and from neighbors on the ring; pairing (each group at most one fight; `paired`'s rate ε/2 per group, `challenge`'s about ε); each victory rule on hand-built groups, including a zero payoff range; both refills; mutation to the other two types; continuous traits in range; statistics and `long_run`'s window; the view and Inspect; keyframes; live and reset fields; degenerate configs (2 groups, size 2, all rates 0, all rates 1, fine 0, baseline 0).
- **Web:** schema groups, charts, the Compare entry, a sweep over a `punishment` base, determinism through the engine.
- **Browser (controller):** every preset's view and charts, Inspect, Compare, recording, Experiments.

## Docs

README: an Altruistic Punishment section (the model, the stated choices, switches, presets, sweeps, and the findings). `docs/papers.md`: the milestone's row (with Cooney as the critique and Janssen's code as reference); the Queue's first entry removed; roadmap: Milestone 27 done.

## Amendments (implementation planning)

Found while implementing and measuring, in the plan `docs/superpowers/plans/2026-09-28-punishment.md`:

- **A fourth pairing, `either`**, the reading the figures fit: groups are paired at random and either group of a pair can start the conflict, each with probability ε (a pair fights with probability 2ε − ε²). Measured with the implementation, it reproduces all six of Fig. 1's curves (mean gaps 0.010–0.033) and Figs. 2 and 3 (0.006–0.022), which it was not found from; Fig. 4's fixed cost misses at 0.051. The preset `bg-either` and the sweep `bg-fig1-either` show it. Every "reproduces the figure" claim uses one rule, fixed before measuring: a mean gap of at most 0.05 over n 4–256.
- **The figures' values** were read from the PDF at 300 dpi by marker centers (a marker hidden under another curve takes that curve's value).
- **Presets:** eleven, `bg-either` added; the titles as measured (`bg-base` "Punishers keep about 70 % of groups of 32 cooperating"; `bg-fixed`, `bg-continuous` and `bg-ring` say what happens).
- **Sweeps:** eighteen. `bg-sensitivity` is `bg-mutation`, `bg-error` and `bg-groups`; `bg-readings` compares the text, `challenge`, `in_turn` and Janssen's readings with and without punishment; `bg-continuous` adds a run without conflict; `bg-cooney-fine` and `bg-cooney-cost` add the `defectors` rule as a third series and read window means of `payoff` and `punishment`.
- **The Conflict chart** shows `conflicts` and `spread`: every conflict replaces a group, so `extinctions` always equals `conflicts` (it stays a series).
- **The mosaic:** ⌈√(2N)⌉ groups a row; cells of 480/(groups a row × ⌈√n⌉) pixels, from 1 to 8.
- **The final review's amendment:** what the figures fit is about twice the stated conflict rate, not the `either` mechanism in particular: three claims score all 14 curves of Figs. 1–4 by one rule with each curve's worst point — the text's rate 2 of 14, `either` 13 (Fig. 4 at 0.051), pairs at 2ε 14 (Fig. 4 at 0.049). They replace the two `either` claims below.
- **Measured with the implementation** (the survey, 25 claims: 12 hold, 1 weak, 12 fail; after the final review 26: 12 hold, 1 weak, 13 fail): Figs. 1a, 2, 3 and 4's shapes, punishment helping at n 32, conflict raising cooperation, the spread calibration (40 periods), mutation, errors, the benefit variant and the start all hold; Fig. 1b's reach, Fig. 2b's reach, "groups on the order of 100" (0.17 at n 128), the caption's rates, the mixing calibration (0.58 of the difference left after 50 periods), continuous traits (0.94 against 0.69 at n 32), the ring (0.51 at n 4), every baseline, Janssen's readings (1a's mean gap 0.056), the Fig. 2–4 generalization of `either` (Fig. 4 at 0.051) and both of Cooney's claims (the dip appears under tanh too; a higher k never raises punishment) fail; fewer groups is weak (a lower mean at 8, not only noise).
