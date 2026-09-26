# SugarScape Milestone 21 — Image scoring (Nowak & Sigmund and its critics) — Design

**Date:** 2026-09-26
**Milestone number:** 21 (the `norms` worktree is taking 20); renumbered at merge if another lands first.
**Builds on:** the milestone 1–19 specs in `docs/superpowers/specs/`; all remain binding where not changed here, in particular milestone 9's model kinds and milestone 11's conventions (portable math, named switches for unstated choices, measured descriptions).
**Sources** (`papers/image-scoring/` and the planning scratch):
- **NS98:** Nowak & Sigmund, "Evolution of indirect reciprocity by image scoring", *Nature* 393 (1998) 573–577, read with its IIASA report IR-98-040 (identical numbers; the report adds "The Dynamics of Indirect Reciprocity", the draft of NS98b, *J. Theor. Biol.* 194:561–574).
- **LH01:** Leimar & Hammerstein, "Evolution of cooperation through indirect reciprocity", *Proc. R. Soc. Lond. B* 268 (2001) 745–753.
- **PB03:** Panchanathan & Boyd, "A tale of two defectors", *J. Theor. Biol.* 224 (2003) 115–126 (binary-score analysis; standing).
- **LFS99:** Lotem, Fishman & Stone, *Nature* 400 (1999) 226–227 and its appendix (no ESS without phenotypic defectors).
- **OI06:** Ohtsuki & Iwasa, "The leading eight", *J. Theor. Biol.* 239 (2006) 435–444 (image scoring is not among the stable norms).
- **FAIR23:** the Make Models FAIR NetLogo reimplementation (`imagescore.nlogo`, 2023): scores clamped at ±5, roulette reproduction, mutation uniform over k, observers Bernoulli per agent — but no payoff offset and a fixed visibility probability rather than NS98's ten observers.

## Goal

Add image scoring as a fourteenth model kind — NS98's simulations (Figs. 1–4), their analytic results, and LH01's island model, execution and perception errors, h strategies, q strategies and Sugden's standing strategy — a full citizen of the playground (worker engine, Max speed, replay and links, keyframes and the timeline, stop rules, Compare, recording, Experiments, the CLI and the survey), with every claim measured.

## Non-negotiable constraints

- **Earlier models unchanged.** Every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **The default is NS98's Fig. 1** (its stated rules, with LH01's statement of the payoff offset); each unstated choice or later reading is a named switch.
- **One engine path** over the `Model` trait.
- **Deterministic and portable.** A function of (config, seed); all draws from the world's seeded RNG with `u32` ranges; no platform transcendental functions; fingerprints identical native and WASM.
- **Truthful descriptions**, with measurements, including what does not reproduce.

## Source summary

- **NS98 model:** n players; each generation starts with all image scores 0; m donor–recipient pairs drawn at random; a donor with strategy k helps iff the recipient's score ≥ k; helping costs c, gives b, and raises the donor's score by 1, refusing lowers it by 1; the recipient's score does not change; "players leave offspring in proportion to their fitness"; mutation: "a probability of 0.001 that an offspring … uses another randomly chosen strategy". Fig. 1: n = 100, scores −5 … +5, k −5 … +6 (−5 unconditional cooperators, +6 defectors), m = 125, b = 1, c = 0.1, "to avoid negative payoffs we add 0.1 in each interaction"; k = 0 fixed after t = 166. Fig. 2: m = 300, mutation 0.001; endless cycles; unconditional cooperators let defectors in. Fig. 3: observers — "each interaction is observed, on average, by 10 randomly chosen players"; perception matrix sᵢⱼ, 0 when unknown; m = 10n; cooperative strategies (k ≤ 0) 90 %, 47 %, 18 % at n = 20, 50, 100. Fig. 4: AND (help if recipient ≥ k and own < h) and OR (… or own < h) strategies; most frequent (k 0, h 1) with perfect information, (k 0, h 4) with observers; cooperation 55 %, 57 %, 70 %, 80 % in (a)–(d) (m = 500; (b, d) n = 20 with observers); own-score-only strategies < 0.1 %. Text: "it suffices that each player is chosen only for about 2 interactions per life-time".
- **NS98 analytics (Methods):** two scores (0, 1), discriminators and defectors, information probability q, prior p = 1; difference equations x₀′, x₁′, y₀′, y₁′; payoff expressions; discriminators stable iff q > c/b and rounds 1/(1 − w) > (bq + c)/(bq − c) ("about 1.2 rounds" at b = 1, c = 0.1, q = 1); with cooperators, equilibrium x = c(2 − w)/(bwq). "Universal constant": all players k = 0, unbounded scores, xᵢ′ = [xᵢ + xᵢ₋₁φ + xᵢ₊₁(1 − φ)]/2 with φ = Σ_{i≥0} xᵢ; the maximum initial fraction below 0 converging to all-out cooperation "is 0.7380294688360…" — the initial distribution below 0 is not stated (planning check: 0.5 with the negatives at −1 and the rest at 0, 0.642 with the rest at +1, 0.688 at +2).
- **LH01:** g groups of n; m rounds per generation, one random donor–recipient pair per round; "we also add the amount c in each round to both the donor and recipient, as did Nowak & Sigmund"; initial payoff u₀; scores constrained −5 … +5; reproduction: summed payoffs per genotype, local with probability p, global otherwise; mutation ν uniform over genotypes; execution error e; perception error ε; random number of rounds (end probability 1/m) for the standing analysis. Figs.: 1a h = 1 invades k = 0 (e = 0); 1b h = 1 invades (k 0, h 1) with e = 0.05 (n 100, g 100, m 500, b 1, c 0.25, u₀ 0, p 0.9, ν 0); 2 help 39 % (g 1, e 0), 9 % (g 100, p 0.9, e 0.02), 2 % (p 0.5) at c 0.25, ν 0.001; 3a help 45 % (c 0.1, u₀ 5, p 0.5, e 0.02); 3b–c with q strategies (Δq 0.01 … 0.99) help 15 %, q strategies 12 %; standing: good at start, lost by refusing a recipient in good standing, regained by helping anyone; best reply iff rb − c > 0, r = (m − 1)/(n + m − 1); with perception errors vrb < c < rb, v = ε/(e + ε); Fig. 4: standing invades binary discriminators (k 0 with scores {0, −1}) with e 0.05, and with e = ε = 0.025, and persists with cooperators over 10⁵ generations (ν 0.0001).

## Architecture

- **Model kind** `image` ("Image Scoring"): `ModelKind::Image`, `ModelConfig::Image(ImageConfig)` tagged `"model": "image"`, `ImageWorld` implementing `Model`, keyframes (`Clone`), schema, `SERIES`, presets, golden entries. Code in `crates/sugarscape-core/src/image/` (`config.rs`, `world.rs`, `strategy.rs` (the strategy classes and their decisions), `stats.rs`, `analytic.rs` (NS98's Methods), `presets.rs`, `mod.rs`).
- **A tick is a generation.** The world keeps the last generation's per-agent results for the frame and Inspect.

## Config

| Field | Default | Apply | Meaning |
|---|---|---|---|
| `groups`, `group_size` | 1, 100 | reset | g groups of n |
| `local` | 1 | live | p: chance a parent is drawn from the offspring's own group |
| `rounds`, `rounds_kind` | 125, `fixed` | live | m; `random`: each round is the last with probability 1/m |
| `b`, `c`, `u0` | 1, 0.1, 0 | live | benefit, cost, initial payoff |
| `offset` | `both` | live | `both`: add c to donor and recipient each round (LH01 on NS98); `none` (FAIR23) |
| `clamp` | 5 | live | scores stay in −clamp … +clamp (0: unbounded) |
| `information` | `perfect` | reset | `perfect` or `observers` |
| `observers` | 10 | live | mean observers per interaction (each other group member except the pair sees with probability observers/(n − 2); the recipient always sees) |
| `execution_error` | 0 | live | e: a donor does the other action |
| `perception_error` | 0 | live | ε: an observer records the other action |
| `mutation` | 0 | live | ν: an offspring's strategy is redrawn uniformly from the allowed set |
| `strategies` | `["k"]` | reset | classes allowed: `k` (−5 … +6), `h` (own < h, h −5 … +6), `and` (k, h), `or` (k, h), `own_only` (same as `h`, reported apart for Fig. 4's claim), `binary` (cooperator k −1, discriminator k 0, defector k 1 with scores {−1, 0}), `standing`, `q` (Δq 0.01 … 0.99) |
| `initial` | `uniform` | reset | `uniform` over the allowed set, or `{ "only": <strategy> }` with an invader `{ "invader": <strategy>, "share": x }` |
| `end` | 0 | live | the last generation (0: never) |
| `schedule` | `[]` | — | as milestone 11's (live paths only) |

## Rules

A generation (one tick), per group:

1. Scores 0 (standing good; binary scores 0; q-strategy tallies at LH01's prior); payoffs `u0`; with `observers`, every perception entry unknown (0).
2. For each round (m fixed, or until the 1/m stop): a uniformly random donor and a different uniformly random recipient in the group. The donor's view of the recipient (perfect: the score; observers: its own record) and its own score decide per its strategy; with probability e the action flips. Helping: donor −c, recipient +b; `offset: both` adds c to both. The donor's score ±1 (clamped); standing and q tallies update; observers (the recipient always, each other member with probability observers/(n − 2)) record the donor's new score (flipped action with probability ε).
3. After all groups: each new agent's parent is drawn by payoff-proportional roulette from its group (probability p) or from the whole population (1 − p); with probability ν its strategy is redrawn uniformly from the allowed set.

## Choices the sources leave open

1. **The offset** — "we add 0.1 in each interaction": LH01 say to both players each round: default `both`; FAIR23 omits it (`none`).
2. **Score range** — Fig. 1's legend: ±5, clamped (FAIR23, LH01).
3. **Reproduction** — "in proportion to their fitness": roulette with replacement, fixed n (FAIR23, LH01).
4. **Mutation** — "another randomly chosen strategy": uniform over the allowed set (FAIR23).
5. **Observers** — "on average by 10": each other member independently with probability 10/(n − 2), the recipient always (FAIR23 uses a fixed visibility, which cannot hold ten across n).
6. **Unknown scores** — "if j has no information on i then sᵢⱼ = 0".
7. **AND/OR ranges** — k and h both −5 … +6 (FAIR23).
8. **Initial strategies** — uniform over the allowed set (Fig. 1: "a random distribution of strategies").
9. **Universal constant's start** — unstated; `analytic.rs` computes the threshold for negatives at −1 with the rest at 0, +1, +2, and spread uniformly, and reports which (if any) gives 0.738….
10. **Long runs** — NS98 average over 10⁷ generations, LH01 over 10⁵–10⁶; claims use shorter runs averaged over seeds, stated in each claim.

## Statistics

`SERIES`: `help_rate` (helps ÷ rounds), `mean_k` (over k-bearing strategies), `cooperative` (share with k ≤ 0 or a cooperative class), `mean_payoff`, `mean_score`, shares `k_cooperative`, `k_defective`, `h`, `and`, `or`, `standing`, `binary_c`, `binary_d`, `binary_x` (discriminators), `q`.

## Views

- **Colour modes:** **Strategy** (k on a blue–red scale; each other class its own colour), **Score** (diverging −5 … +5), **Payoff** (heat). Each group is a tile of cells, groups in a grid.
- **Inspect:** an agent's strategy, score (with observers: how many members know it, and their mean view), payoff, helps given and received this generation.
- **Charts:** **Help rate**, **Mean k**, **Strategy shares**, **Mean payoff**.

## Presets

| Preset | Setup | Source |
|---|---|---|
| `ns-fig-1` | defaults | NS98 Fig. 1 |
| `ns-fig-2` | m 300, ν 0.001 | NS98 Fig. 2 |
| `ns-fig-3-n20`, `ns-fig-3-n50`, `ns-fig-3-n100` | observers 10, m 10n, ν 0.001 | NS98 Fig. 3 |
| `ns-fig-4a` … `ns-fig-4d` | AND / OR, perfect / observers (n 20), m 500, ν 0.001 | NS98 Fig. 4 |
| `ns-own-only` | `own_only`, m 500, ν 0.001 | NS98 Fig. 4 text |
| `lh-fig-1a`, `lh-fig-1b` | k 0 (or k 0 & h 1) with an h 1 invader; g 100, m 500, c 0.25, p 0.9 (e 0, 0.05) | LH01 Fig. 1 |
| `lh-fig-2a`, `lh-fig-2b`, `lh-fig-2c` | AND strategies; g 1 / g 100 p 0.9 / p 0.5; c 0.25, ν 0.001, e 0 / 0.02 / 0.02 | LH01 Fig. 2 |
| `lh-fig-3a`, `lh-fig-3b` | c 0.1, u₀ 5, p 0.5, e 0.02; without / with q strategies | LH01 Fig. 3 |
| `lh-fig-4a`, `lh-fig-4b`, `lh-fig-4c` | standing vs binary strategies; e 0.05 / e = ε = 0.025 / long run ν 0.0001 | LH01 Fig. 4 |
| `fair-no-offset` | `offset: none` | FAIR23's reading |

**Compare entries:** "One group vs the island model — Image Scoring (Compare)" (`lh-fig-2a` vs `lh-fig-2b`), "Image scoring vs standing — Image Scoring (Compare)" (`lh-fig-4a` with standing vs without), "With vs without the offset — Image Scoring (Compare)" (`ns-fig-1` vs `fair-no-offset`).

## Experiments and CLI

- `ns-rounds`: base `ns-fig-2`; x = m 25 … 500; metric `cooperative`.
- `ns-group-size`: base `ns-fig-3-n50`; x = n 20 … 100 (m = 10n per point via the sweep's series or paired paths); metric `cooperative`.
- `lh-cost`: x = c 0.05 … 0.5; series one group vs island; metric `help_rate`.
- `lh-gene-flow`: base `lh-fig-2b`; x = p 0.5 … 1; metric `help_rate`.
- `sugarscape presets | run | sweep` accept `image`.

## Claims to test (survey and book-style tests)

- **NS98:** Fig. 1 fixation of k = 0 (and when); Fig. 2 cycles (a measured count) and unconditional cooperators preceding defector invasions; Fig. 3's 90 / 47 / 18 %; Fig. 4's 55 / 57 / 70 / 80 % and most frequent strategies; own-only < 0.1 %; "about 2 interactions per lifetime"; the Methods' analytics (q > c/b; ~1.2 rounds; the equilibrium with cooperators) by iterating their equations and by simulation; 0.738… under each tried start.
- **LH01:** Fig. 1a/1b invasions; Fig. 2's 39 / 9 / 2 %; Fig. 3's 45 / 15 % and 12 % q strategies; the standing condition and Fig. 4's invasions and long run.
- **Ours:** the offset's effect on Fig. 1; FAIR23's fixed visibility against NS98's ten observers.

Tolerances come from the measurements (as milestones 11–19); runs shorter than the papers' are stated.

## Page

- An **Image Scoring** presets group; the schema panel in groups **Game** (b, c, u₀, offset), **Population** (groups, group size, local), **Rounds** (m, kind), **Information** (mode, observers shown only with `observers`, clamp), **Errors** (execution, perception), **Evolution** (mutation), **Run** (end); `strategies` and `initial` set by presets, files and links (as ethno's `allowed`); the colour modes; charts; Inspect; the Compare entries; `defaultForm('image')` (x = rounds, `cooperative`).
- Keyframes, the timeline, stop rules, share links, sessions, recording and Compare work unchanged.

## Testing

- **Core unit:** a round's payoffs with and without the offset; score update and clamp; each strategy class's decision (k, h, and, or, binary, standing, q) on hand-built states; execution and perception errors; observers (the recipient always sees; expected count); unknown scores as 0; random rounds; roulette reproduction and the island draw; mutation uniform over the allowed set; `initial` with an invader; `analytic.rs` against the paper's formulas; validation errors on the named field.
- **Golden:** entries for every preset; earlier entries untouched; WASM equal to native.
- **Book-style (`#[ignore]`, release):** the claims above, thresholds measured.
- **Web (Vitest):** schema fields; charts; Inspect rows; Compare entries; `defaultForm('image')`; determinism fingerprints.
- **Browser (controller):** every preset, colour modes, Inspect, the Compare entries, sweeps, Max speed at g 100 × n 100.

## Docs

README: an Image Scoring section in the voice of the others — the rules, the choices above, what reproduces and what does not, the critics' variants, the Compare entries and sweeps — crediting NS98 (and NS98b), LH01, PB03, LFS99, OI06 and FAIR23. `docs/papers.md`: the milestone's row. Roadmap: this milestone done; the image-scoring entry marked done.
