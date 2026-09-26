# Image Scoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Nowak & Sigmund's image scoring (1998) as a model kind, `image`, with Leimar & Hammerstein's island model, errors, h and q strategies and Sugden's standing strategy as named switches, presets, sweeps and survey claims, NS98's Methods as a small analytic module, and every claim measured.

**Architecture:** A new module `crates/sugarscape-core/src/image/` implementing the `Model` trait (a tick is a generation; groups of agents, rounds of donor–recipient pairs, payoff-proportional reproduction with an island draw), with strategy classes in `strategy.rs` and NS98's difference equations and "universal constant" in `analytic.rs`; wired into `ModelKind`/`ModelConfig`/`ModelWorld` with keyframes, presets and golden entries; book-style tests and sweeps measure it against NS98 and LH01; the page gains its types, colour modes, charts, Inspect, Compare entries and Experiments default; the survey gains its claims.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm via wasm-pack, sugarscape-cli, the standalone survey crate), TypeScript (Vite, uPlot, Vitest).

**Spec:** `docs/superpowers/specs/2026-09-26-image-scoring-design.md`

## Global Constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **The default is NS98's Fig. 1** (its stated rules, with LH01's statement of the payoff offset); each unstated choice or later reading is a named switch.
- **Deterministic and portable:** a function of (config, seed); all draws from the world's seeded `SimRng` with `u32` ranges; no platform transcendental functions; fingerprints identical native and WASM.
- **Truthful descriptions,** with measurements, including what does not reproduce; numbers only from Decision M (the measurements), never invented.
- **Verify with:** `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and `cargo +1.98.1 clippy --all-targets -- -D warnings` (CI runs the newest stable clippy), `cargo test --workspace`, `cargo test -p sugarscape-core --release --test image -- --ignored`, `wasm-pack test --node crates/sugarscape-wasm`, `(cd web && npm run build && npm test)`, `(cd survey && cargo test)`.
- **Do not run `cargo fmt` inside `survey/`** (it would rewrite unrelated chapters; the survey is not a workspace member).
- **Commits:** stage only the files the task names (never `-A`/`.`, never `.claude/`, `.superpowers/`, `web/src/wasm-pkg`); messages are plain imperative sentences and end with a second `-m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"`.
- **Web:** TypeScript strict with `noUnusedLocals`/`noUnusedParameters`; build DOM with `h()`; typographic apostrophes (’) in test names.
- **If a measured number differs from the plan, stop and report; do not retune thresholds.**

## Review Focus

1. **A group of two with observers** (no bystanders): only the recipient watches, nothing divides by zero — Task 1, `a_group_of_two_with_observers_has_only_the_recipient_watching`.
2. **More observers than bystanders** (observers 10 in a group of 5): everyone watches, the chance is capped at 1 — Task 1, `more_observers_than_bystanders_means_everyone_watches`.
3. **Unbounded scores** (`clamp: 0`) over many rounds: the run and every frame mode still work — Task 1, `unbounded_scores_run_long_and_still_draw`.
4. **A generation where nobody earns** (defectors, no offset, one round): the empty payoff pool falls back to a uniform draw and the population stays whole — Task 1, `a_single_round_generation_with_all_payoffs_zero_still_reproduces`.
5. **A generation at the validation limits** (40,000 agents, 20,000 rounds per group, observers): the work per generation is rounds × groups × group size; the final review should judge whether the limits keep a generation responsive in the worker (no test pins wall-clock time).

## Why this task order

The core model (Task 1) carries everything later tasks read: the config and schema, the statistics, the inspection JSON and the golden fingerprints. Task 2 measures it and adds the sweeps, which need only the core. The page (Tasks 3–4) needs the core's JSON; the survey (Task 5) needs the presets and the measured tolerances; the docs (Task 6) quote everything.

## Decisions (where the spec leaves room)
Binding once the plan is written. Every rule below was implemented in the scratch copy and measured.

1. **Wiring.** `ModelKind::Image` last (`ALL: [ModelKind; 14]`), `"image"`; `pub mod image;` after `pub mod geometry;`; the catalog ends with the image presets; the unknown-model message ends "…, dpd or image". The label "Image Scoring" lives only in `web/src/models.ts` (Rust has no kind labels) — a page-task item. Sweep error "image scoring stops at its last generation, N in this config"; CLI "finished at tick N (its last generation)".
2. **A tick is a generation, and the world holds the generation that last played.** `step`: scheduled changes; offspring of the previous generation (from the second tick on, by the payoffs it kept); then play. So the frame, Inspect and the tick-t snapshot all describe the same generation; tick 0 is the first generation before it plays (help rate NaN).
3. **The RNG contract.** Setup: a uniform start draws `gen_range(0..allowed)` per agent, a seeded start draws nothing. A round: donor `gen_range(0..n)`, recipient `gen_range(0..n − 1)` shifted past the donor; execution error `gen::<f64>() < e` only when e > 0; observers in member order, the donor skipped, the recipient always, each other member `gen::<f64>() < p` only when p < 1; perception error per observer only when ε > 0; random rounds `gen::<f64>() < 1/m` after each round. Offspring, in group and slot order: `gen::<f64>() < p` only when g > 1 and 0 < p < 1; roulette `gen::<f64>() × total` over cumulative weights (binary search; `gen_range` when the pool has no weight); mutation `gen::<f64>() < ν` only when ν > 0, then `gen_range` over the allowed set.
4. **A new switch, `records`, default `tally`** (the spec says an observer "records the donor's new score"). NS98 says only that onlookers "update their perception of the donor's image score". `tally`: the observer moves its own record one up or down by the action it saw (FAIR23's `imagescoreothers`); `score`: it records the donor's true previous score ± 1, so one sighting reveals the whole score. Measured, Fig. 3 reproduces only with `tally` (86 / 44 / 20% against 90 / 47 / 18%; `score`: 97 / 93 / 92%). Live.
5. **Observers.** `observers` is a number (mean others per interaction); each member besides the pair sees with probability min(1, observers/(n − 2)); n = 2: only the recipient. FAIR23's visibility v is `observers = v(n − 2)`.
6. **Private records** only with `information: observers` or ε > 0: per group, dense n × n `i16` records of scores and `u8` marks (bit 0: believes the other in good standing; bit 1: has seen it act this generation). A donor uses its true own score, its own belief about its own standing (judged, like everyone's, by its own record of the recipient), and its records of the recipient. The world also keeps each agent's true score and standing (an omniscient judge) for statistics, frames and Inspect. With perfect information and ε > 0 every member observes, each misperceiving independently.
7. **Standing** (LH01's second variant): good at the start; lost by refusing a recipient believed good; regained by helping anyone; refusing one believed bad leaves it unchanged.
8. **q strategies.** One tally set per group, public: updated from the recipient's true score and the true action (no perception error), with LH01's prior; a donor at score s helps iff q(s + 1) − q(s − 1) > Δq, the neighbours clamped to the score range. Δq in hundredths on the wire (`{"q": 25}`). Validation: q needs bounded scores (clamp ≥ 1).
9. **Binary scorers.** Whenever `binary` is allowed every score lives in {−1, 0}; `binary` combines only with `standing` (validation). Their decision is the k rule.
10. **"Cooperative"** for every class = helps at a generation's start (own and recipient's scores 0, both good, the q prior) — NS98's k ≤ 0 generalised. `SERIES` adds `own_only` (the spec's list has no share for it) and `helps` (a count) to the spec's fifteen.
11. **A seeded start** (`{"only", "invader", "share"}`): the first round(share × n) members of every group play the invader; both must be of allowed classes. LH01's invasion presets use 1%.
12. **Reproduction.** Individual roulette (equivalent to LH01's per-genotype sums); the global pool is every agent's payoff. Negative payoffs (possible only without the offset) weigh 0; a pool with no weight is drawn uniformly.
13. **Live and reset.** Reset: `groups`, `group_size`, `information`, `strategies`, `initial`, `schedule`; all else live, applied from the next generation (records and tallies are rebuilt each generation, so `clamp`, `records` and ε can change live).
14. **Validation beyond the spec.** group size 2–500, groups 1–400, g × n ≤ 40,000; rounds 1–20,000 (records are `i16`); b, c, u₀ in [0, 1,000]; clamp 0–100; observers 0–500; errors and mutation in [0, 1]; strategies non-empty, no repeats.
15. **Frames and Inspect.** Each group a square tile of side ⌈√n⌉, tiles in ⌈√g⌉ columns with one-cell gaps (`BACKGROUND`). Strategy: k-bearing classes on `lerp(BLUE, RED, (k + 5)/11)` (binary: (k + 1)/2), h `POLLUTION`, own-only `NEUTRAL`, standing `SUGAR`, q `FEMALE`; Score: `NEUTRAL`→`BLUE` above 0, →`RED` below, over ±min(clamp, 5); Payoff: `lerp(COOL, HOT, payoff/max)`. Inspect JSON `{ cell: {x, y}, group: n | null, agent: { id, group, strategy (label), class, cooperative, score, standing, known, mean_view, payoff, given, received } | null }`, `known`/`mean_view` null with perfect information. Agents CSV `id,group,x,y,strategy,score,standing,payoff,given,received` (names without commas). Fingerprint: FNV-1a over tick, the id counter and each agent (id, strategy code, score and standing, payoff bits, helps given and received).
16. **Presets that differ from the spec's table.** `fair-no-offset` becomes **`ns-no-offset`**: FAIR23 does add c to both players each round (its code says so), so the offset-free run is nobody's reading — it is our ablation. **`ns-fig-4b` and `ns-fig-4d` use m = 200** (NS98: "(b, d) as in figure 3 with n = 20", i.e. m = 10n), not 500 (4b 53% / 4d 85% at 200; 55% / 94% at 500). LH01's Fig. 4 presets keep fixed rounds (the random rounds are for their analysis); `lh-fig-4c` starts uniform. The Compare entry "With vs without the offset" pairs `ns-fig-1` with `ns-no-offset`.
17. **Golden.** A separate `IMAGE_GOLDEN` of (id, generations, fingerprint): one-group presets 200 generations, island presets 20 (a 100 × 100 island generation is 50,000 rounds, and with ε > 0 five million record updates — 200 would take minutes in debug). `every_model_preset_has_a_golden_entry` and `print_golden` include it. Keyframes: `ns-fig-4b` in `tests/checkpoint.rs` (records, mutation).
18. **Sweeps** (after `dpd-max-age`): `ns-rounds` (base `ns-fig-2`, m 25 … 500, 20,000 generations, window mean `cooperative` from 1,001), `ns-group-size` (base `ns-fig-3-n50`, n 20 … 100 with `rounds = 10n` in each x value's `set`), `lh-cost` (base `lh-fig-2b`, c 0.05 … 0.5, series one group / 100 groups, 5,000 generations, `help_rate`), `lh-gene-flow` (base `lh-fig-2b`, p 0.5 … 1). Seeds 1–10 each.
19. **The universal constant.** `analytic::Start` covers negatives at −1 with the rest at a score or out of reach (`rest: None`, mass that never falls below 0), negatives spread over −1 … −b with the rest at 0, and both spread. Bisection (45 steps); cooperation when φ > 1 − 10⁻¹², defection when after round 1 less than 10⁻¹² of the movable mass is at or above 0; tails below 10⁻⁴⁰ dropped.
20. **Analytic limits.** At q = x = 1, Di(k) − De(k) is its limit [(b − c) − b·2^−(k−1)]/2 and 2(Di − De) at x = 1 is (bq − c)/(1 − w) − 2qb/(2 − w) (both limits agree). Powers by repeated multiplication (no `powf`).
21. **Book-style tests** pin seed means to ±0.0005 (worlds are deterministic; four-decimal pins) and counts exactly, with the source's number beside each; runs are 10–100× shorter than the papers' and say so; failing claims are pinned as measured. Slowest 55 s.
22. **Spec corrections** (docs task): FAIR23 adds the offset (and with visibility < 1 its donor reads the recipient's record of itself — a bug); FAIR23 is Janssen's 2010 NetLogo 4.1.1 model, republished 2023; `ns-fig-4b/4d` m = 200; the `records` switch; `ns-no-offset`; `own_only` and `helps` series; `IMAGE_GOLDEN`.

**M. The measurements** — `measurements.md` (every number, method and seed set). Re-measuring: `cargo test -p sugarscape-core --release --test image -- --ignored --nocapture` and `cargo run --release -p sugarscape-cli -- sweep --builtin <id> --quiet --summary-csv /dev/stdout --out /dev/null`. If an implementation following this plan gives different numbers, stop and report rather than retune.

### Surprises (for the README and the survey)
1. **NS98's universal constant reproduces to every printed digit, and its unstated start is found:** all the negatives at −1 and the rest out of reach gives 0.7380294688360038 (NS98: 0.7380294688360…). Rest at 0, +1, +2: 0.5, 0.642, 0.688; the limit is reached by about +80.
2. **Fig. 3's group-size effect depends on how an observer records:** own tallies 86 / 44 / 20% (NS98 90 / 47 / 18%); the whole score at one sighting 97 / 93 / 92%. FAIR23's fixed visibility (0.1) flattens it too: 28 / 16 / ≈ 20%.
3. **The spec misreads FAIR23:** it adds the offset; its donor (with visibility < 1) reads the recipient's own record of itself.
4. **Fig. 1's k = 0 fixation is a minority outcome:** 20 of 100 runs (median generation 56); defection wins 60.
5. **"About 2 interactions per lifetime" does not suffice:** at m = n cooperative strategies hold 18% of the time; half needs m = 2n.
6. **LH01's drift argument does not reproduce:** Fig. 2b 44% help (LH01 9%), 2c 15% (2%); help is not monotone in gene flow (isolated groups 27%, p = 0.8 50%); below c = 0.25 islands help more than one group.
7. **LH01 Fig. 1b's invasion is about ten times slower** (h = 1 at 1.8% by generation 150, 42% by 1,000).
8. **Fig. 4c/4d's most frequent strategy is cooperative OR, not the defector** ((k 3, h 4) and (k 2, h 5); (6, −5) second and third); help 78% and 85% against 70% and 80%.
9. **Own-score strategies help 0.19%,** not < 0.1% — the mutation floor.
10. **The offset lowers cooperation:** Fig. 1 some k ≤ 0 fixes 40/100 with it, 63/100 without; Fig. 2 67% against 78%.
11. **The Methods' x_min (0.123 over five rounds) is below the simulated 0.16** — its rounds (everyone plays once) are not random pairs.
12. **Reproduced:** Fig. 2's cycles (172 collapses and 167 recoveries in 10⁶ generations; k ≤ −4 at 68% before a collapse against 8% in cooperative phases), Fig. 3 (with tallies), Fig. 4a, "about 1.2 rounds" and q > c/b, LH01 Figs. 1a, 2a, 3 (help), 4a–c, and the standing condition's failure at Fig. 4b's parameters.


23. **The kind on the page.** `ModelKind` gains `'image'`, labelled **Image Scoring**, and `MODELS` ends with it — after `'dpd'`, not straight after `'ethno'` (the brief's wording dates from when ethnocentrism was last; the presets menu follows the Rust catalog, which ends with the image presets). `ColorMode` gains `'score'` (`strategy` and `payoff` exist); `COLOR_MODES.image` is Strategy, Score, Payoff (the core's names); `MODEL_OVERLAYS.image = []`. `finishedNotice` shares tags' "last generation" text; `ticksLeft` counts to `end`.
24. **Compare entries.** `image-one-vs-island` (`lh-fig-2a` vs `lh-fig-2b`) and `image-offset` (`ns-fig-1` vs `ns-no-offset`) as the brief says. `image-scoring-vs-standing` is `lh-fig-2b` vs `lh-fig-4c`: Compare pairs preset ids, and no `lh-fig-4*` preset has discriminators without standing (4a and 4b both seed 1% standing; 4c starts uniform over all four), so the cleanest pair is the same island model (100 × 100, p 0.9, c 0.25, m 500, errors about 0.02) with image-scoring AND strategies (help 44%) against binary scorers with standing (help 94%). A fourth, `image-group-size` ("Small vs large groups with observers": `ns-fig-3-n20` vs `ns-fig-3-n100`), shows the claim that reproduces; `records: tally` vs `score` has no preset for B, and the core adds none.
25. **`defaultForm('image')`** follows the spec, not the brief's copied `r`: x = `rounds` over `50:500:50` (NS98's rounds axis, the built-in `ns-rounds` shortened), 2,000 generations, the window mean of `cooperative` from generation 1,001, the form's 3 seeds (30 points).
26. **Charts** (against **Generation**): **Help rate** (`help_rate` with `cooperative`, both shares on [0, 1]: Fig. 3 plots the one, Fig. 4 the other); **Mean k** on [−5, 6], shown only when a class with a k is allowed; **Strategy shares** (k ≤ 0, k > 0, h, own only, AND, OR, standing, q) unless binary scorers are allowed, when **Binary scorers and standing** (cooperators, discriminators, defectors, standing) shows instead; **Mean payoff**. `helps` (a count, = help rate × rounds) and `mean_score` get no chart.
27. **Inspect.** The core's inspection is `{cell, group, agent}` with no `site`, so `isImageView` is shape-only (`'cell' in v && 'group' in v`; no other model has a `cell`), and the four guards that read `v.site` (`isSugarView`, `isRingView`, `isValleyView`, `isSpatialView`) first check `'site' in v` (TypeScript requires it once the union has a member without `site`; `sim-host.test.ts` reads `site` through `toMatchObject`). `render` handles an image cell first. The helper lives in **`web/src/image-scoring.ts`** (`web/src/image.ts` is the map import's `capacitiesFromPixels`). Rows: cell, group ("42 of 100", only with more than one), agent id, strategy with "helps/refuses at a generation's start", score, "Seen by" (private records only), standing (only when standing is allowed), payoff, this generation's help; a gap and a tile's unused cell say so. There are no neighbours to list (partners are random each round, per the spec's Inspect). An agent lives one generation (offspring get new ids), so a followed agent is gone at the next tick: the note says "Agent #N's generation has passed: each agent lives one generation."
28. **Survey.** `survey/src/claims/image.rs`, 34 claims with ids under `ns-` and `lh-` (their items are presets and sweeps; `ns-methods` and `ns-universal` name the Methods), so `--only ns-` and `--only lh-` run them. Each claim uses its own seeds (1–10; Fig. 1 1–100; the Methods' simulation 1–2,000; Fig. 4b's invasion 1–5) whatever `--seeds` says, and the measurements' windows, except where a claim would pass about 30 s: Fig. 2b/2c and their comparison over generations 1,001–3,000 (not 5,000), Fig. 4b's invasion to generation 500 (not 1,000), and the records claim at n = 20 against n = 50 (not 100). A source's percentage is judged by `equivalent` of our seeds against the constant (a one-sample TOST) with a margin of 5 points — the most our own estimates move between the survey's windows and the measurements' longer ones (Fig. 3 n = 20: 0.863 → 0.912; LH01 3a: 0.522 → 0.468) — so noisy seeds around the source are Weak, not Holds. Deterministic analytic claims are judged over a grid (rounds 1.00–2.00, q 0.01–1, w 0.5–0.95, costs 0.05–0.25, five far starts), each point an indicator. **`lh-fig-4c.long-run` is Untestable in the survey and pinned by the book test instead**: from a uniform start standing needs about 1,000 generations, and five seeds to 600 took 33 s with standing still at 0.31. Figs. 1, 2, 3 and the fixation runs are memoized, so later claims reuse them.
29. **Web golden.** `determinism.test.ts` gains its own `IMAGE_GOLDEN` of (id, generations, fingerprint), Decision 17's table (200 generations one group, 20 islands), checked in three chunks with charts, a colour mode and a selection watched; the WASM prints 16 hex digits, so `ns-fig-2` and `ns-fig-4b` gain a leading zero. A test checks the table lists every image preset in catalog order.
30. **The Rules panel needs no code.** The schema's seven groups, `observers` shown only under `information: observers`, and every number within its slider (b, c 0–10, u₀ 0–50, mutation to 0.1, rounds to 20,000) work through the existing panel; no field is negative. `records` is always shown (it also matters with perfect information and ε > 0). The tests check the groups, the `show_if`, that `strategies` and `initial` are not fields, and every preset on its sliders.
31. **Docs.** The milestone is **21** in the roadmap (the spec's number; the norms milestone takes 20), not the brief's copied "17". `docs/papers.md` gains the row and loses image scoring from the Queue (the rows below renumber). The spec notes record Decisions 4, 10, 16, 17, 22 and 24–28.

## Decision M: the measurements (planning dry run, recorded 2026-09-26)

### NS98

### Fig. 1 (defaults: n 100, m 125, k −5…+6 uniform, offset both, ±5)
Seeds 1–100, each run until one strategy is fixed (max 5,000 generations; all fixed).

| | k = 0 fixed | some k ≤ 0 fixed | median generation k = 0 fixed |
|---|---|---|---|
| defaults | **20/100** | 40/100 | 56 (range 24–174) |
| m = 50 | 10 | 19 | 39 |
| m = 300 | 14 | 91 | 103 |
| m = 1,000 | 14 | 100 | 177 |
| offset none | 10 | 63 | 93 |

NS98 show one run (k = 0 fixed at t = 166). k = 0 is the most common single winner, but defection (k ≥ 1) wins 60% of runs at m = 125. "Cooperation is more likely to win the greater the number m": reproduced.

### Fig. 2 (m 300, ν 0.001) — cycles
Seeds 1–10 × 10⁵ generations. A **collapse** = the share of k ≤ 0 falling from ≥ 0.9 to ≤ 0.1; a **recovery** the reverse.
- 172 collapses, 167 recoveries (1.72 per 10⁴ generations); every seed 12–24 collapses. Endless cycles: reproduced.
- Share of k ≤ −4: 0.081 averaged over cooperative generations (k ≤ 0 ≥ 0.9); 0.682 over the 51 generations up to each collapse's last cooperative generation; above the cooperative mean before 132/172 collapses. Unconditional cooperators rise before defector invasions: reproduced.
- Cooperative share (window 1,001–100,000): 0.672 (0.55–0.78 by seed). Without the offset: 0.777; collapses 156, recoveries 155.

### Fig. 3 (observers 10, m 10n, ν 0.001) — group size
Seeds 1–10, window 1,001–20,000 (the book test). NS98: 90 / 47 / 18%.

| records | n 20 | n 50 | n 100 |
|---|---|---|---|
| **tally** (default; FAIR23) | **0.863** | **0.440** | **0.201** |
| score (spec's "donor's new score") | 0.968 | 0.928 | 0.923 |
| tally, FAIR23 visibility 0.1 (observers 1.8 / 4.8 / 9.8) | 0.282 | 0.155 | ≈ 0.20 (9.8 ≈ 10) |

Longer (scratch): tally, window 1,001–100,000 at n 20: 0.912 ± 0.056; n 50 (1,001–50,000): 0.459 ± 0.131; n 100 (1,001–30,000): 0.215 ± 0.131. Score, 10⁵ generations: 0.967 / 0.954 / 0.914. Visibility 0.1 (1,001–50,000): n 20 0.289, n 50 0.201.
Sweep `ns-group-size` (same window): n 20 0.863, 30 0.826, 50 0.440, 70 0.347, 100 0.201.

### Fig. 4 (m 500 perfect / n 20 m 200 observers; ν 0.001)
Seeds 1–10, window 1,001–50,000; strategy frequencies summed over the same window.

| | help rate | NS98 | most frequent (share) | NS98 most frequent |
|---|---|---|---|---|
| 4a AND perfect | 0.5316 | 55% | (k 0, h 1) 22.2%; (−1, 0) 15.3%; (−1, 1) 11.3% | (0, 1) |
| 4b AND observers | 0.5344 | 57% | (k 0, h 5) 10.7%; (−1, 5) 6.0%; (0, 3) 5.6% | (0, 4) |
| 4c OR perfect | 0.7843 | 70% | (k 3, h 4) 10.0%; **(6, −5) 6.0%**; (6, −4) 4.9% | (6, −5) |
| 4d OR observers | 0.8545 | 80% | (k 2, h 5) 4.5%; (6, 5) 3.5%; **(6, −5) 3.1%** | (6, −5) |

Scratch: 4b/4d with `records: score`: 0.538 / 0.854; with m = 500 (the spec's reading) and tally: 0.550 / 0.935. Over 1,001–20,000 only, 4a's top two swap ((−1, 0) 16.0%, (0, 1) 14.3%) — (k −1, h 0) behaves like (k 0, h 1) on scores {−1, 0}.

### Own score only (m 500, ν 0.001)
Help rate, seeds 1–10, window 1,001–20,000: **0.0019** (scratch 1,001–50,000: 0.0019 ± 0.0001). NS98: < 0.1%. The floor is mutation: uniform mutants, 5 of 12 of which (h ≥ 1) help at a generation's start.

### "About 2 interactions per life-time" (m ≈ n)
Fig. 2's settings, cooperative share, seeds 1–10, window 1,001–20,000 (sweep `ns-rounds`): m 25: 0.013; 50: 0.100; 75: 0.084; **100: 0.182**; 125: 0.154; 150: 0.231; 200: 0.500; 300: 0.652; 500: 0.911. Not reproduced: two interactions per lifetime give 18%; half the time needs m = 200 (four per lifetime).

### Methods (analytic.rs)
- Minimum mean rounds (bq + c)/(bq − c) at b 1, c 0.1, q 1: **1.2222** ("about 1.2"). Stability flips exactly at w* = 1 − 1/1.2222 (unit test, ±1e-9). 1.2 rounds: not stable; 1.25: stable.
- q > c/b: at q = 0.1 or 0.05 (b 1, c 0.1) min rounds = ∞ and no w < 1 is stable; at q 0.2: 3 rounds.
- Closed-form Di(k) − De(k) equals the difference equations iterated, to 1e-14 (k 1–12, two parameter sets); the w-weighted sum of 3,000 rounds equals the closed 2(Di − De) to 1e-12.
- q = 1: x_min = c(2 − w)/(bw) (= the with-cooperators equilibrium at q = 1): 0.3 at w 0.5.
- Equilibrium with cooperators x = c(2 − w)/(bwq): Dc − De = 0 there, sign change either side (unit test); at b 1, c 0.1, q 1, w 0.9: 0.1222.
- **Simulation vs x_min:** binary discriminators (k 0) vs defectors (k 1), no offset, perfect information, n 100, m 250 (= 5 Methods rounds); one generation's mean payoff gap, 2,000 seeds per share. Analytic x_min (5 fixed rounds): **0.1228**. Simulated sign change: between 0.15 (gap −0.0044) and **0.16** (first positive at 0.01 steps); scratch, 4,000 seeds per 0.05 step: gap −0.0044 at 0.15, +0.037 at 0.20; the simulated gap runs 0.03–0.05 below the analytic one at low x and meets it near x 0.9. The Methods' rounds (everyone plays once, half as donor) are not NS98's random pairs.

### Universal constant (k = 0, unbounded scores, map xᵢ′ = [xᵢ + xᵢ₋₁φ + xᵢ₊₁(1 − φ)]/2)
Bisection (45 steps) on the fraction f below 0; cooperation when φ > 1 − 1e-12, defection when (after round 1) the movable mass at ≥ 0 falls below 1e-12.

| start | threshold |
|---|---|
| f at −1, rest at 0 | 0.4999999999999716 |
| f at −1, rest at +1 | 0.642004504966053 |
| f at −1, rest at +2 | 0.6878695524337104 |
| f at −1, rest at +5 | 0.7263483514787481 |
| f at −1, rest at +10 | 0.7363132493855 |
| f at −1, rest at +20 | 0.737966029371222 |
| f at −1, rest at +40 | 0.7380292624026 |
| f at −1, rest at +80 | 0.7380294688227309 |
| **f at −1, rest never falling below 0 (+∞)** | **0.7380294688360038** |
| f uniform over −1, −2; rest at 0 | 0.42091451472646213 |
| f uniform over −1 … −5; rest at 0 | 0.3384593809258831 |
| f uniform over −1 … −5; rest uniform over 0 … 5 | 0.515790224230301 |

NS98: 0.7380294688360…. **Reproduced to every printed digit by the negatives at −1 and the rest out of reach** (the limit of "rest at +r" as r → ∞; +80 already agrees to 1e-11). Python cross-check (numpy, fixed window) agreed for rest 0–80.

### LH01 (100 groups of 100, m 500, b 1, u₀ 0, unless noted)

### Fig. 1 (c 0.25, p 0.9, ν 0; invader at 1% of each group)
Seeds 1–10. 1a (k 0 vs h 1, e 0): h = 1 at 50 gens **0.370** (0/10 above ½), at 100 0.993, at 150 **1.000**, fixed in all. LH01: ≈ 0.8 at 150 — reproduced, faster. 1b ((0, 1) vs h 1, e 0.05): **0.018** at 150, **0.132** at 500, **0.423** at 1,000 (5/10 above ½), 0.645 at 2,000 (8/10) — invades, ~10× slower than LH01's figure.

### Fig. 2 (AND; c 0.25, ν 0.001)
- 2a (g 1, e 0): help **0.3742** (seeds 1–10, window 1,001–100,000; 0.20–0.45 by seed); most frequent (k 0, h 1) 37.7%, top in 8/10 seeds. LH01 39%: reproduced. With e 0.02: 0.351 (LH01 ≈ 30%).
- 2b (p 0.9, e 0.02): help **0.4413** (seeds 1–10, window 1,001–5,000); scratch 1,001–20,000: 0.345 ± 0.197 (0.02–0.54); 10,001–50,000: 0.311 ± 0.160 (0.02–0.53). LH01 9%: **not reproduced**.
- 2c (p 0.5, e 0.02): help **0.1544** (1,001–5,000); 1,001–20,000: 0.094 ± 0.125 (0.02–0.37). LH01 2%: not reproduced.
- Sweep `lh-gene-flow` (1,001–5,000): p 0.5 0.154; 0.6 0.347; 0.7 0.390; 0.8 0.500; 0.9 0.441; 1.0 0.269.
- Sweep `lh-cost` (1,001–5,000): one group c 0.05 0.584, 0.1 0.436, 0.15 0.423, 0.25 0.351, 0.35 0.086, 0.5 0.061; islands 0.533, 0.504, 0.509, 0.441, 0.0219, 0.0213.

### Fig. 3 (c 0.1, u₀ 5, p 0.5, e 0.02, ν 0.001)
Seeds 1–10, window 1,001–3,000: 3a help **0.5220** (LH01 45%); 3b help **0.1653** (15%), q share **0.2579** (12%). Scratch 1,001–10,000: 3a 0.468 ± 0.060; 3b 0.148 ± 0.076, q 0.177 ± 0.111.

### Fig. 4 (binary scorers {−1, 0} + standing; c 0.25, p 0.9)
- 4a (only discriminators + 1% standing, e 0.05, ν 0): standing at 250 0.189, **500 0.694**, **1,000 0.984** (seeds 1–10). Reproduced.
- 4b (e = ε = 0.025): **500 0.352**, **1,000 0.746** (seeds 1–5); seeds 1–10: 0.377 / 0.767 (8/10 above ½). Reproduced.
- 4c (uniform start, e = ε = 0.025, ν 0.0001): seeds 1–3, window 1,001–1,500: standing **0.531**, cooperators **0.389**, discriminators **0.080**, defectors **0.0002**. Scratch seeds 1–10, window 1,001–3,000: 0.775 ± 0.051, 0.198, 0.027, 0.0001; help 0.938. LH01 (10⁵): standing dominant, cooperators appreciable, discriminators occasionally 5%, defectors < 1%: reproduced (shorter runs).
- Condition: r = (m − 1)/(n + m − 1) = 0.8331; v = ε/(e + ε) = 0.5; vrb = 0.417 > c = 0.25 → not met (LH01: "not fulfilled"). e 0.04, ε 0.01: met.

### Timing (release, 10 threads)
One group at m 300 (ns-fig-2): ~36 µs/generation (10⁵ generations × 10 seeds in 3.6 s on 10 threads). ns-fig-3-n100: ~0.6 ms/gen. Island (lh-fig-2b): 3.8 ms/gen alone. Island with ε > 0 (lh-fig-4b): 26 ms/gen alone, ~110 ms/gen with 10 in parallel (memory-bound records). Book tests: 18 tests, the slowest lh01_fig_4c 55 s, ns98_fig_3 47 s, all others ≤ 30 s (total ≈ 4.5 min). `cargo test --workspace` (debug): golden 62 s (image entries included).

### Golden fingerprints (seed 1; `IMAGE_GOLDEN`: one group 200 generations, islands 20)
ns-fig-1 200 0x98875bd71738cf05; ns-fig-2 200 0x91995405bae5fd9; ns-fig-3-n20 200 0xb4bd11bc229673c4; ns-fig-3-n50 200 0x5907ce5cb47602cf; ns-fig-3-n100 200 0xaf771beff4611d0d; ns-fig-4a 200 0xe7d6cefc18bd8d6a; ns-fig-4b 200 0xa4c19c5fa5fcfe3; ns-fig-4c 200 0x64755eafb526a816; ns-fig-4d 200 0x72ca0b1d80947f44; ns-own-only 200 0x20d6768b1fbd7081; ns-no-offset 200 0xc595349de0c0bf65; lh-fig-1a 20 0x46669de796924497; lh-fig-1b 20 0x2c3d0dea6af0c894; lh-fig-2a 200 0x56250416634c6ed9; lh-fig-2b 20 0x566a697764e9cdd9; lh-fig-2c 20 0x963ec9f09d2fadbe; lh-fig-3a 20 0x46059fbe6bab2056; lh-fig-3b 20 0x4d6575c59b0da89a; lh-fig-4a 20 0xd5654fa4b600eb57; lh-fig-4b 20 0x4bfe8ea9f3e8598b; lh-fig-4c 20 0x6371653b544f6387.
WASM (`wasm-pack test --node crates/sugarscape-wasm`, 42 passed): ns-fig-1 (200), ns-fig-4b (200), lh-fig-3b (20) equal native. Every earlier golden entry unchanged. Keyframes: `tests/checkpoint.rs` adds `ns-fig-4b`.

## File Structure
**Task 1:** create `crates/sugarscape-core/src/image/{mod,strategy,config,stats,world,analytic,presets}.rs`; modify `crates/sugarscape-core/src/lib.rs`, `model.rs`, `presets.rs`, `sweep.rs` (the ticks message only), `crates/sugarscape-cli/src/main.rs` (the finish message), `crates/sugarscape-core/tests/golden.rs` (`IMAGE_GOLDEN`), `crates/sugarscape-core/tests/checkpoint.rs` (`ns-fig-4b`).
**Task 2:** create `crates/sugarscape-core/tests/image.rs` (18 ignored tests) and `sweeps/{ns-rounds,ns-group-size,lh-cost,lh-gene-flow}.json`; modify `crates/sugarscape-core/src/sweep.rs` (BUILTINS and the id list), `crates/sugarscape-cli/tests/cli.rs` and `crates/sugarscape-wasm/tests/web.rs` (ids and a fingerprint test).


- Create: `web/src/image-scoring.ts`, `web/src/image-scoring.test.ts`, `survey/src/claims/image.rs`
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `survey/src/claims/mod.rs`, `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-26-image-scoring-design.md`
- Test: `web/src/models.test.ts`, `web/src/schema-form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/engine.test.ts`, `web/src/sim-host.test.ts`, `web/src/compare-presets.test.ts`, `web/src/experiments/form.test.ts`, `web/src/determinism.test.ts`

---

### Task 1: Image scoring in the core

**Files:**
- Create: `crates/sugarscape-core/src/image/{mod,strategy,config,stats,world,analytic,presets}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs` (kind, config, world, keyframe arms, the unknown-model message, `max_ticks`, series, the round-trip and kind-list tests), `crates/sugarscape-core/src/presets.rs` (the catalog), `crates/sugarscape-core/src/sweep.rs` (the ticks message only), `crates/sugarscape-cli/src/main.rs` (the finish message only), `crates/sugarscape-core/tests/golden.rs` (`IMAGE_GOLDEN`, 21 entries), `crates/sugarscape-core/tests/checkpoint.rs` (`ns-fig-4b`)

**Interfaces:**
- Consumes: `crate::config::{FieldError, ScheduledChange}`; `crate::model::{wrong_model, Model, ModelConfig, ModelKind}`; `crate::presets::ModelPreset`; `crate::render::{lerp, Rgb, BACKGROUND, BLUE, COOL, FEMALE, HOT, NEUTRAL, POLLUTION, RED, SUGAR}`; `crate::rng::{self, SimRng}`; `crate::schema::{Apply, Param}` (`with_help`, `shown_if`); `crate::stats::{Series, Stats}`; `crate::export::history_csv`; `crate::schema::check_schema` (tests).
- Produces: `image::{ImageConfig, RoundsKind, Offset, Information, Records, Initial, Seeded, LIVE, schema, ImageSnapshot, SERIES, Class, Strategy, Situation, Tallies, K_MIN, K_MAX, ImageWorld, ImageMode, Agent, AgentView, Cell, ImageInspection, presets}`, `image::analytic::{Binary, State, Start, Scores, cooperates, universal_threshold, standing_r, standing_v, standing_stable}`; `ImageConfig::{allowed, score_range, watch_probability, validate, changes}`; `ImageWorld::{new, step, run, agents, group, population, allowed, is_finished, helps, private, cell, inspect, config, tick, stats}`; `ModelKind::Image` (`"image"`), `ModelConfig::Image`, `ModelWorld::Image` with keyframes; `IMAGE_GOLDEN` (21 entries).

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/image/mod.rs`:

```rust
//! Image scoring (milestone 21): Nowak & Sigmund, "Evolution of indirect
//! reciprocity by image scoring" (Nature 393, 1998) and its Methods, with
//! Leimar & Hammerstein's (Proc. R. Soc. B 268, 2001) island model, errors,
//! own-score, standing and q strategies, and each unstated choice as a named
//! switch. See docs/superpowers/specs/2026-09-26-image-scoring-design.md.

pub mod analytic;
mod config;
mod presets;
mod stats;
mod strategy;
mod world;

pub use config::{
    schema, ImageConfig, Information, Initial, Offset, Records, RoundsKind, Seeded, LIVE,
};
pub use presets::presets;
pub use stats::{ImageSnapshot, SERIES};
pub use strategy::{Class, Situation, Strategy, Tallies, K_MAX, K_MIN};
pub use world::{Agent, AgentView, Cell, ImageInspection, ImageMode, ImageWorld};
```
Add `pub mod image;` to `crates/sugarscape-core/src/lib.rs` (after `pub mod geometry;`):

```diff
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -17,6 +17,7 @@
 pub mod export;
 pub mod frames;
 pub mod geometry;
+pub mod image;
 pub mod landscape;
 mod legacy;
 pub mod model;
```
Create `crates/sugarscape-core/src/image/strategy.rs` with only its tests for now (the decisions, the q tallies, the wire form; the implementation goes above them in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn at(own: i32, seen: i32) -> Situation<'static> {
        Situation {
            own,
            seen,
            own_good: true,
            their_good: true,
            tallies: None,
        }
    }

    #[test]
    fn k_helps_a_recipient_seen_at_k_or_above() {
        assert!(Strategy::K(0).helps(&at(-5, 0)));
        assert!(!Strategy::K(0).helps(&at(5, -1)));
        assert!(Strategy::K(-5).helps(&at(0, -5)), "−5 always helps");
        assert!(!Strategy::K(6).helps(&at(0, 5)), "+6 never helps");
    }

    #[test]
    fn h_and_own_only_look_only_at_the_donors_score() {
        for s in [Strategy::H(1), Strategy::OwnOnly(1)] {
            assert!(s.helps(&at(0, -5)));
            assert!(!s.helps(&at(1, 5)));
        }
        assert_eq!(Strategy::OwnOnly(1).class(), Class::OwnOnly);
    }

    #[test]
    fn and_needs_both_and_or_either() {
        let and = Strategy::And { k: 0, h: 1 };
        let or = Strategy::Or { k: 0, h: 1 };
        assert!(and.helps(&at(0, 0)));
        assert!(!and.helps(&at(1, 0)) && !and.helps(&at(0, -1)));
        assert!(or.helps(&at(1, 0)) && or.helps(&at(0, -1)));
        assert!(!or.helps(&at(1, -1)));
    }

    #[test]
    fn binary_scorers_are_cooperators_discriminators_and_defectors() {
        let [c, x, d] = [-1, 0, 1].map(Strategy::Binary);
        for seen in [-1, 0] {
            assert!(c.helps(&at(0, seen)) && !d.helps(&at(0, seen)));
        }
        assert!(x.helps(&at(-1, 0)) && !x.helps(&at(0, -1)));
    }

    #[test]
    fn standing_helps_the_good_or_when_itself_bad() {
        let s = |own_good, their_good| {
            Strategy::Standing.helps(&Situation {
                own_good,
                their_good,
                ..at(0, 0)
            })
        };
        assert!(s(true, true) && s(false, true) && s(false, false));
        assert!(!s(true, false));
    }

    #[test]
    fn q_strategies_follow_the_tallies() {
        let mut t = Tallies::new(-5, 5);
        // The prior: q = 1 at s ≥ 0, 0 below; at 0 the gain is 1, at 1 it is 0.
        assert_eq!(
            (t.q(0), t.q(-1), t.gain(0), t.gain(1)),
            (1.0, 0.0, 1.0, 0.0)
        );
        let q = |d, own, t: &Tallies| {
            Strategy::Q(d).helps(&Situation {
                tallies: Some(t),
                ..at(own, 0)
            })
        };
        assert!(q(99, 0, &t) && !q(1, 1, &t) && !q(1, -3, &t));
        // A recipient at −2 is helped: x counts for s ≥ −2.
        t.record(-2, true);
        assert_eq!((t.x[3], t.x[2], t.y[3]), (1, 0, 1));
        assert_eq!(t.q(-2), 0.5);
        // A recipient at 3 is refused: y counts for s ≤ 3.
        t.record(3, false);
        assert_eq!((t.y[8], t.y[9], t.x[8]), (1, 0, 2));
        assert_eq!(t.q(3), 2.0 / 3.0);
        // At the ends the neighbours clamp: at 5, q(6) is q(5).
        assert_eq!(t.gain(5), t.q(5) - t.q(4));
        assert!(!Strategy::Q(50).helps(&at(0, 0)), "no tallies, no help");
        t.reset();
        assert_eq!(t, Tallies::new(-5, 5));
    }

    #[test]
    fn cooperative_means_helping_at_the_start() {
        assert!(Strategy::K(0).cooperative() && !Strategy::K(1).cooperative());
        assert!(Strategy::H(1).cooperative() && !Strategy::H(0).cooperative());
        assert!(Strategy::And { k: 0, h: 1 }.cooperative());
        assert!(!Strategy::And { k: 0, h: 0 }.cooperative());
        assert!(Strategy::Or { k: 1, h: 1 }.cooperative());
        assert!(Strategy::Binary(0).cooperative() && !Strategy::Binary(1).cooperative());
        assert!(Strategy::Standing.cooperative() && Strategy::Q(99).cooperative());
    }

    #[test]
    fn classes_list_their_members_and_the_wire_form_is_stable() {
        let sizes: Vec<usize> = Class::ALL.iter().map(|c| c.members().len()).collect();
        assert_eq!(sizes, [12, 12, 144, 144, 12, 3, 1, 99]);
        for c in Class::ALL {
            assert!(c.members().iter().all(|s| s.is_valid() && s.class() == c));
        }
        assert!(!Strategy::K(7).is_valid() && !Strategy::Q(0).is_valid());
        assert!(!Strategy::Binary(2).is_valid());
        let json = serde_json::to_string(&[
            Strategy::K(0),
            Strategy::And { k: 0, h: 1 },
            Strategy::Standing,
            Strategy::Q(25),
        ])
        .unwrap();
        assert_eq!(
            json,
            r#"[{"k":0},{"and":{"k":0,"h":1}},"standing",{"q":25}]"#
        );
        let codes: std::collections::HashSet<u64> = Class::ALL
            .iter()
            .flat_map(|c| c.members())
            .map(Strategy::code)
            .collect();
        assert_eq!(codes.len(), 12 * 3 + 144 * 2 + 3 + 1 + 99);
        assert_eq!(Strategy::And { k: 0, h: 1 }.label(), "k = 0, h = 1 (AND)");
        assert_eq!(Strategy::Q(25).label(), "Δq = 0.25");
    }
}
```
Create `crates/sugarscape-core/src/image/config.rs` with only its tests for now (defaults, `initial`, validation, schedules, the schema; the implementation goes above them in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &ImageConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_ns98_fig_1_and_validates() {
        let c = ImageConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!(
            (c.groups, c.group_size, c.rounds, c.clamp),
            (1, 100, 125, 5)
        );
        assert_eq!((c.b, c.c, c.u0, c.mutation), (1.0, 0.1, 0.0, 0.0));
        assert_eq!((c.local, c.observers), (1.0, 10.0));
        assert_eq!(c.strategies, [Class::K]);
        assert_eq!(c.allowed().len(), 12);
        assert_eq!(c.score_range(), (-5, 5));
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["offset"], "both");
        assert_eq!(v["rounds_kind"], "fixed");
        assert_eq!(v["information"], "perfect");
        assert_eq!(v["records"], "tally");
        assert_eq!(v["initial"], "uniform");
        assert_eq!(v["strategies"], json!(["k"]));
    }

    #[test]
    fn initial_reads_uniform_or_a_seeded_start() {
        let c: ImageConfig = serde_json::from_value(json!({
            "strategies": ["k", "h"],
            "initial": {"only": {"k": 0}, "invader": {"h": 1}, "share": 0.01}
        }))
        .unwrap();
        let Initial::Seeded(s) = &c.initial else {
            panic!("{:?}", c.initial)
        };
        assert_eq!(
            (s.only, s.invader, s.share),
            (Strategy::K(0), Some(Strategy::H(1)), 0.01)
        );
        assert!(c.validate().is_ok());
        let back = serde_json::to_value(&c).unwrap();
        assert_eq!(
            back["initial"],
            json!({"only": {"k": 0}, "invader": {"h": 1}, "share": 0.01})
        );
        let c: ImageConfig = serde_json::from_value(json!({"initial": "uniform"})).unwrap();
        assert_eq!(c.initial, Initial::Uniform);
        assert!(serde_json::from_value::<ImageConfig>(json!({"initial": "random"})).is_err());
        assert!(serde_json::from_value::<ImageConfig>(
            json!({"initial": {"only": {"k": 0}, "extra": 1}})
        )
        .is_err());
    }

    #[test]
    fn validation_names_the_field() {
        let bad = |edit: &dyn Fn(&mut ImageConfig)| {
            let mut c = ImageConfig::default();
            edit(&mut c);
            fields(&c)
        };
        assert_eq!(bad(&|c| c.group_size = 1), ["group_size"]);
        assert_eq!(bad(&|c| c.group_size = 501), ["group_size"]);
        assert_eq!(bad(&|c| c.groups = 0), ["groups"]);
        assert_eq!(bad(&|c| c.groups = 401), ["groups"]);
        assert_eq!(
            bad(&|c| {
                c.groups = 100;
                c.group_size = 401;
            }),
            ["groups"]
        );
        assert!(bad(&|c| c.groups = 400).is_empty());
        assert_eq!(bad(&|c| c.local = 1.5), ["local"]);
        assert_eq!(bad(&|c| c.rounds = 0), ["rounds"]);
        assert_eq!(bad(&|c| c.rounds = 20_001), ["rounds"]);
        assert_eq!(bad(&|c| c.b = f64::NAN), ["b"]);
        assert_eq!(bad(&|c| c.c = -0.1), ["c"]);
        assert_eq!(bad(&|c| c.u0 = f64::INFINITY), ["u0"]);
        assert_eq!(bad(&|c| c.clamp = 101), ["clamp"]);
        assert_eq!(bad(&|c| c.observers = -1.0), ["observers"]);
        assert_eq!(bad(&|c| c.execution_error = 2.0), ["execution_error"]);
        assert_eq!(bad(&|c| c.perception_error = -0.1), ["perception_error"]);
        assert_eq!(bad(&|c| c.mutation = 1.1), ["mutation"]);
        assert_eq!(bad(&|c| c.strategies = vec![]), ["strategies"]);
        assert_eq!(
            bad(&|c| c.strategies = vec![Class::K, Class::K]),
            ["strategies"]
        );
        assert_eq!(
            bad(&|c| c.strategies = vec![Class::Binary, Class::K]),
            ["strategies"]
        );
        assert!(bad(&|c| c.strategies = vec![Class::Binary, Class::Standing]).is_empty());
        assert_eq!(
            bad(&|c| {
                c.strategies = vec![Class::Q];
                c.clamp = 0;
            }),
            ["strategies"]
        );
        let seeded = |only, invader, share| {
            Initial::Seeded(Seeded {
                only,
                invader,
                share,
            })
        };
        assert_eq!(
            bad(&|c| c.initial = seeded(Strategy::H(1), None, 0.0)),
            ["initial"]
        );
        assert_eq!(
            bad(&|c| c.initial = seeded(Strategy::K(9), None, 0.0)),
            ["initial"]
        );
        assert_eq!(
            bad(&|c| c.initial = seeded(Strategy::K(0), Some(Strategy::K(1)), 1.5)),
            ["initial"]
        );
        assert!(bad(&|c| c.initial = seeded(Strategy::K(0), Some(Strategy::K(1)), 0.5)).is_empty());
    }

    #[test]
    fn binary_scores_are_zero_and_minus_one_and_the_watch_chance_follows_n() {
        let mut c = ImageConfig {
            strategies: vec![Class::Binary, Class::Standing],
            ..Default::default()
        };
        assert_eq!(c.score_range(), (-1, 0));
        assert_eq!(c.allowed().len(), 4);
        c.strategies = vec![Class::K];
        c.clamp = 0;
        assert_eq!(c.score_range(), (-1_000_000, 1_000_000));
        assert_eq!(c.watch_probability(), 1.0);
        c.information = Information::Observers;
        c.group_size = 20;
        assert_eq!(c.watch_probability(), 10.0 / 18.0);
        c.group_size = 10;
        assert_eq!(c.watch_probability(), 1.0);
        c.group_size = 2;
        assert_eq!(c.watch_probability(), 0.0);
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = ImageConfig {
            schedule: vec![entry("c", json!(0.25))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("group_size", json!(40))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("mutation", json!(2))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = ImageConfig::default();
        let mut b = a.clone();
        b.c = 0.25;
        b.records = Records::Score;
        b.execution_error = 0.02;
        assert!(a.changes(&b).is_empty());
        b.groups = 10;
        b.strategies = vec![Class::And];
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["groups", "strategies"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: ImageConfig = serde_json::from_str(r#"{"rounds": 300, "offset": "none"}"#).unwrap();
        assert_eq!((c.rounds, c.offset, c.group_size), (300, Offset::None, 100));
        assert!(serde_json::from_str::<ImageConfig>(r#"{"visibility": 0.1}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Image(ImageConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```
Create `crates/sugarscape-core/src/image/world.rs` with only its tests for now (a round, scores, each class, standing, q, errors, observers, records, random rounds, roulette, mutation, `initial`, statistics, frames, runs, keyframes; they use the private `reset_results`, `interact`, `play`, `reproduce`, `record`, `at`, `n` and the `agents`, `views`, `marks`, `tallies`, `helps`, `rounds_played`, `rng` fields; the implementation goes above them in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduledChange;
    use crate::image::config::Seeded;
    use serde_json::json;

    /// A world of `groups` × `n` all playing `only`, after `edit`, with its
    /// results reset (a generation about to play).
    fn world(
        groups: u32,
        n: u32,
        only: Strategy,
        edit: impl FnOnce(&mut ImageConfig),
    ) -> ImageWorld {
        let mut c = ImageConfig {
            groups,
            group_size: n,
            strategies: vec![only.class()],
            initial: Initial::Seeded(Seeded {
                only,
                invader: None,
                share: 0.0,
            }),
            ..Default::default()
        };
        edit(&mut c);
        let mut w = ImageWorld::new(c, 1).unwrap();
        w.reset_results();
        w
    }

    fn payoffs(w: &ImageWorld) -> Vec<f64> {
        w.agents.iter().map(|a| a.payoff).collect()
    }

    #[test]
    fn a_round_pays_b_and_c_with_or_without_the_offset() {
        // Helping: donor −c (+c offset), recipient +b (+c).
        let mut w = world(1, 3, Strategy::K(-5), |c| c.u0 = 2.0);
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [2.0, 3.1, 2.0]);
        assert_eq!((w.agents[0].given, w.agents[1].received), (1, 1));
        assert_eq!(w.helps(), (1, 1));
        let mut w = world(1, 3, Strategy::K(-5), |c| c.offset = Offset::None);
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [-0.1, 1.0, 0.0]);
        // Refusing: only the offset.
        let mut w = world(1, 3, Strategy::K(6), |_| {});
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [0.1, 0.1, 0.0]);
        assert_eq!(w.helps(), (0, 1));
        let mut w = world(1, 3, Strategy::K(6), |c| c.offset = Offset::None);
        w.interact(0, 0, 1);
        assert_eq!(payoffs(&w), [0.0, 0.0, 0.0]);
    }

    #[test]
    fn a_donors_score_moves_one_step_within_the_clamp_and_the_recipients_does_not() {
        let mut w = world(1, 2, Strategy::K(-5), |_| {});
        for i in 1..=8 {
            w.interact(0, 0, 1);
            assert_eq!(w.agents[0].score, i.min(5));
        }
        assert_eq!(w.agents[1].score, 0);
        let mut w = world(1, 2, Strategy::K(6), |c| c.clamp = 2);
        for _ in 0..5 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, -2);
        let mut w = world(1, 2, Strategy::K(-5), |c| c.clamp = 0);
        for _ in 0..30 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, 30, "unbounded");
        // Binary scorers live in {−1, 0}.
        let mut w = world(1, 2, Strategy::Binary(1), |_| {});
        for _ in 0..3 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, -1);
        w.agents[0].strategy = Strategy::Binary(-1);
        for _ in 0..3 {
            w.interact(0, 0, 1);
        }
        assert_eq!(w.agents[0].score, 0);
    }

    #[test]
    fn each_class_decides_from_the_scores_it_reads() {
        // A k = 0 donor refuses a recipient at −1, helps one at 0.
        let mut w = world(1, 3, Strategy::K(0), |_| {});
        w.agents[1].score = -1;
        w.interact(0, 0, 1);
        w.interact(0, 0, 2);
        assert_eq!((w.agents[1].received, w.agents[2].received), (0, 1));
        // AND (k 0, h 1): not once its own score is 1.
        let mut w = world(1, 2, Strategy::And { k: 0, h: 1 }, |_| {});
        w.interact(0, 0, 1);
        w.interact(0, 0, 1);
        assert_eq!((w.agents[0].score, w.agents[0].given), (0, 1));
        // OR (k 1, h 1): helps a recipient at 0 only while its own score is below 1.
        let mut w = world(1, 2, Strategy::Or { k: 1, h: 1 }, |_| {});
        for _ in 0..3 {
            w.interact(0, 0, 1);
        }
        assert_eq!((w.agents[0].score, w.agents[0].given), (1, 2));
        // h = 1 keeps its score at 0 or 1 whatever the recipient's.
        let mut w = world(1, 2, Strategy::H(1), |_| {});
        w.agents[1].score = -5;
        for _ in 0..4 {
            w.interact(0, 0, 1);
        }
        assert_eq!((w.agents[0].score, w.agents[0].given), (0, 2));
    }

    #[test]
    fn standing_is_lost_by_refusing_the_good_and_regained_by_helping_anyone() {
        let mut w = world(1, 3, Strategy::Standing, |_| {});
        // A defector refusing a good recipient loses standing …
        w.agents[0].strategy = Strategy::Binary(1);
        w.interact(0, 0, 1);
        assert!(!w.agents[0].good);
        // … and a standing donor refuses it without losing its own.
        w.interact(0, 2, 0);
        assert_eq!(w.agents[0].received, 0);
        assert!(w.agents[2].good && w.agents[2].score == -1);
        // A standing agent in bad standing helps even a bad recipient, and is good again.
        w.agents[2].good = false;
        w.interact(0, 2, 0);
        assert_eq!(w.agents[0].received, 1);
        assert!(w.agents[2].good);
    }

    #[test]
    fn q_strategies_read_and_feed_their_groups_tallies() {
        let mut w = world(2, 3, Strategy::Q(50), |_| {});
        assert_eq!(w.tallies.len(), 2);
        // At score 0 the prior's gain is 1: help; at 1 it is 0: refuse.
        w.interact(0, 0, 1);
        w.interact(0, 0, 2);
        assert_eq!((w.agents[0].given, w.agents[0].score), (1, 0));
        // Group 0 tallied a help at 0 and a refusal at 0; group 1 nothing.
        let t = &w.tallies[0];
        assert_eq!((t.x[5], t.y[5]), (2, 1));
        assert_eq!(w.tallies[1], Tallies::new(-5, 5));
        w.reset_results();
        assert_eq!(w.tallies[0], Tallies::new(-5, 5));
    }

    #[test]
    fn execution_errors_flip_the_action_at_their_rate() {
        let mut w = world(1, 2, Strategy::K(-5), |c| c.execution_error = 0.25);
        for _ in 0..20_000 {
            w.interact(0, 0, 1);
        }
        let rate = w.helps as f64 / w.rounds_played as f64;
        assert!((rate - 0.75).abs() < 0.01, "{rate}");
    }

    /// The records of member `i` held by the others who have seen it act.
    fn records_of(w: &ImageWorld, i: usize) -> Vec<i16> {
        let n = w.n();
        (0..n)
            .filter(|&j| j != i && w.marks[j * n + i] & SEEN != 0)
            .map(|j| w.views[j * n + i])
            .collect()
    }

    #[test]
    fn perception_errors_flip_what_each_observer_records() {
        let mut w = world(1, 50, Strategy::K(-5), |c| c.perception_error = 0.2);
        assert!(w.private());
        let (mut up, mut down) = (0, 0);
        for _ in 0..400 {
            w.reset_results();
            w.interact(0, 0, 1);
            let r = records_of(&w, 0);
            assert_eq!(r.len(), 49, "perfect information: all see");
            up += r.iter().filter(|&&v| v == 1).count();
            down += r.iter().filter(|&&v| v == -1).count();
            assert_eq!(w.agents[0].score, 1, "the true score is untouched");
        }
        let wrong = down as f64 / (up + down) as f64;
        assert!((wrong - 0.2).abs() < 0.01, "{wrong}");
    }

    #[test]
    fn the_recipient_always_sees_and_others_see_observers_over_n_minus_2() {
        let mut w = world(1, 50, Strategy::K(-5), |c| {
            c.information = Information::Observers;
            c.observers = 10.0;
        });
        let mut seen = 0;
        for _ in 0..2000 {
            w.reset_results();
            w.interact(0, 0, 1);
            assert!(w.marks[w.n()] & SEEN != 0, "the recipient saw");
            seen += records_of(&w, 0).len() - 1;
        }
        let mean = seen as f64 / 2000.0;
        assert!((mean - 10.0).abs() < 0.2, "{mean}");
        // No observers: only the recipient.
        let mut w = world(1, 50, Strategy::K(-5), |c| {
            c.information = Information::Observers;
            c.observers = 0.0;
        });
        w.interact(0, 0, 1);
        assert_eq!(records_of(&w, 0).len(), 1);
    }

    #[test]
    fn unknown_scores_read_as_zero_and_records_follow_the_switch() {
        let edit = |records| {
            move |c: &mut ImageConfig| {
                c.information = Information::Observers;
                c.observers = 0.0;
                c.records = records;
            }
        };
        let mut w = world(1, 4, Strategy::K(-5), edit(Records::Score));
        // 0 helps 1, 2 and 3 in turn: each recipient sees one act.
        for r in 1..=3 {
            w.interact(0, 0, r);
        }
        assert_eq!(w.agents[0].score, 3);
        // Score: each records the new score; tally: each its own record + 1.
        assert_eq!(records_of(&w, 0), [1, 2, 3]);
        let mut w = world(1, 4, Strategy::K(-5), edit(Records::Tally));
        for r in 1..=3 {
            w.interact(0, 0, r);
        }
        assert_eq!(records_of(&w, 0), [1, 1, 1]);
        // A k = 1 donor that never saw 3 act reads its score as 0: refuses.
        w.agents[2].strategy = Strategy::K(1);
        w.agents[3].strategy = Strategy::K(1);
        w.interact(0, 2, 3);
        assert_eq!(w.agents[3].received, 1, "only 0's help");
        // One that saw 0 act (record 1) helps it.
        w.interact(0, 3, 0);
        assert_eq!(w.agents[0].received, 1);
    }

    #[test]
    fn a_donor_judges_its_own_standing_by_its_own_record_of_the_recipient() {
        // Only the recipient sees; 1 believes 2 bad, the others believe it good.
        let mut w = world(1, 4, Strategy::Standing, |c| {
            c.information = Information::Observers;
            c.observers = 0.0;
        });
        let n = w.n();
        w.marks[n + 2] &= !GOOD;
        w.interact(0, 1, 2);
        // 1 refuses and still believes itself good; 2 saw it refuse someone
        // 2 believes good (itself) and marks it bad; so does the truth.
        assert_eq!(w.agents[2].received, 0);
        assert!(w.marks[n + 1] & GOOD != 0);
        assert!(w.marks[2 * n + 1] & GOOD == 0);
        assert!(!w.agents[1].good);
    }

    #[test]
    fn random_rounds_end_with_chance_one_over_m() {
        let mut w = world(4, 10, Strategy::K(0), |c| {
            c.rounds = 10;
            c.rounds_kind = RoundsKind::Random;
        });
        let mut total = 0;
        for _ in 0..500 {
            w.play();
            total += w.rounds_played;
        }
        let mean = total as f64 / 2000.0;
        assert!((mean - 10.0).abs() < 0.5, "{mean}");
        w.config.rounds = 1;
        w.play();
        assert_eq!(w.rounds_played, 4, "one round per group");
        w.config.rounds_kind = RoundsKind::Fixed;
        w.config.rounds = 7;
        w.play();
        assert_eq!(w.rounds_played, 28);
    }

    #[test]
    fn parents_are_drawn_by_payoff_from_the_group_or_the_whole_population() {
        let setup = |local: f64| {
            let mut w = world(2, 4, Strategy::K(0), |c| c.local = local);
            for (i, a) in w.agents.iter_mut().enumerate() {
                a.strategy = Strategy::K(i as i8 - 4);
                a.payoff = 0.0;
            }
            w.agents[0].payoff = 1.0;
            w.agents[3].payoff = 3.0;
            w
        };
        // Local: group 0 copies agents 0 and 3, one to three; group 1, with
        // no payoff, copies its own members uniformly.
        let mut counts = [0u32; 8];
        let mut w = setup(1.0);
        for _ in 0..4000 {
            let mut v = w.clone();
            v.reproduce();
            for a in &v.agents[..4] {
                counts[(a.strategy.k().unwrap() + 4) as usize] += 1;
            }
            assert!(v.agents[4..].iter().all(|a| a.strategy.k().unwrap() >= 0));
            w.rng = v.rng;
        }
        assert_eq!((counts[1], counts[2]), (0, 0));
        let r = f64::from(counts[3]) / f64::from(counts[0]);
        assert!((r - 3.0).abs() < 0.15, "{counts:?}");
        // Global: every offspring, in either group, is a child of 0 or 3.
        let mut w = setup(0.0);
        w.reproduce();
        assert!(w
            .agents
            .iter()
            .all(|a| matches!(a.strategy, Strategy::K(-4) | Strategy::K(-1))));
        assert_eq!(w.agents[0].id, 9, "offspring are new agents");
    }

    #[test]
    fn negative_payoffs_weigh_nothing_and_an_empty_pool_is_uniform() {
        let cum = [0.0, 0.0, 2.0, 2.0, 3.0];
        assert_eq!(roulette(&cum, 0.0), Some(2));
        assert_eq!(roulette(&cum, 1.999), Some(2));
        assert_eq!(roulette(&cum, 2.0), Some(4));
        assert_eq!(
            roulette(&cum, 3.0),
            Some(4),
            "a draw rounded up to the total"
        );
        assert_eq!(roulette(&[0.0, 0.0], 0.0), None);
        let mut w = world(1, 4, Strategy::K(0), |c| c.offset = Offset::None);
        for (i, a) in w.agents.iter_mut().enumerate() {
            a.strategy = Strategy::K(i as i8);
            a.payoff = -0.1;
        }
        w.agents[2].payoff = 0.0;
        w.reproduce();
        assert_eq!(w.agents.len(), 4, "a uniform draw when nobody has weight");
    }

    #[test]
    fn mutation_redraws_uniformly_over_the_allowed_set() {
        let mut w = world(1, 100, Strategy::K(0), |c| {
            c.strategies = vec![Class::K, Class::Standing];
            c.mutation = 1.0;
        });
        let mut counts = std::collections::HashMap::new();
        for _ in 0..130 {
            w.reproduce();
            for a in &w.agents {
                *counts.entry(a.strategy).or_insert(0u32) += 1;
            }
        }
        assert_eq!(counts.len(), 13);
        for (s, c) in counts {
            assert!(
                (f64::from(c) / 13_000.0 - 1.0 / 13.0).abs() < 0.01,
                "{s:?}: {c}"
            );
        }
    }

    #[test]
    fn a_seeded_start_puts_the_invaders_first_in_each_group() {
        let w = world(3, 10, Strategy::K(0), |c| {
            c.strategies = vec![Class::K, Class::H];
            c.initial = Initial::Seeded(Seeded {
                only: Strategy::K(0),
                invader: Some(Strategy::H(1)),
                share: 0.2,
            });
        });
        for g in 0..3 {
            let s: Vec<Strategy> = w.group(g).iter().map(|a| a.strategy).collect();
            assert_eq!(s[..2], [Strategy::H(1); 2]);
            assert!(s[2..].iter().all(|&x| x == Strategy::K(0)));
        }
        let s = w.stats.latest().unwrap();
        assert!((s.h - 0.2).abs() < 1e-12 && (s.k_cooperative - 0.8).abs() < 1e-12);
        // A uniform start draws from every allowed strategy.
        let w = ImageWorld::new(
            ImageConfig {
                strategies: vec![Class::Or],
                group_size: 500,
                ..Default::default()
            },
            3,
        )
        .unwrap();
        let kinds: std::collections::HashSet<_> = w.agents.iter().map(|a| a.strategy).collect();
        assert!(kinds.len() > 120 && kinds.iter().all(|s| s.class() == Class::Or));
    }

    #[test]
    fn the_first_generation_plays_its_starting_strategies() {
        let c = ImageConfig {
            mutation: 0.5,
            ..Default::default()
        };
        let mut w = ImageWorld::new(c, 4).unwrap();
        let start: Vec<Strategy> = w.agents.iter().map(|a| a.strategy).collect();
        assert!(w.stats.latest().unwrap().help_rate.is_nan());
        w.step();
        let played: Vec<Strategy> = w.agents.iter().map(|a| a.strategy).collect();
        assert_eq!(start, played);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.tick, w.rounds_played), (1, 125));
        assert!((s.help_rate - s.helps as f64 / 125.0).abs() < 1e-15);
        w.step();
        assert_ne!(
            start,
            w.agents.iter().map(|a| a.strategy).collect::<Vec<_>>()
        );
    }

    #[test]
    fn statistics_count_each_class() {
        let mut w = world(1, 10, Strategy::K(0), |c| {
            c.strategies = vec![
                Class::K,
                Class::H,
                Class::And,
                Class::Or,
                Class::OwnOnly,
                Class::Standing,
                Class::Q,
            ];
        });
        let s = [
            Strategy::K(-2),
            Strategy::K(3),
            Strategy::H(1),
            Strategy::OwnOnly(0),
            Strategy::And { k: 0, h: 1 },
            Strategy::Or { k: 4, h: 0 },
            Strategy::Standing,
            Strategy::Q(10),
            Strategy::Q(90),
            Strategy::K(0),
        ];
        for (a, s) in w.agents.iter_mut().zip(s) {
            a.strategy = s;
            a.score = 1;
        }
        w.record();
        let t = w.stats.latest().unwrap().clone();
        let near = |a: f64, b: f64| (a - b).abs() < 1e-12;
        assert!(near(t.k_cooperative, 0.2) && near(t.k_defective, 0.1));
        assert!(near(t.h, 0.1) && near(t.own_only, 0.1));
        assert!(near(t.and, 0.1) && near(t.or, 0.1));
        assert!(near(t.standing, 0.1) && near(t.q, 0.2));
        // k ≤ 0 twice, h 1, AND (0, 1), standing and both q: cooperative.
        assert!(near(t.cooperative, 0.7), "{}", t.cooperative);
        assert!(near(t.mean_k, (-2.0 + 3.0 + 0.0 + 4.0 + 0.0) / 5.0));
        assert_eq!((t.mean_score, t.binary_c), (1.0, 0.0));
        let mut w = world(1, 3, Strategy::Binary(0), |_| {});
        w.agents[0].strategy = Strategy::Binary(-1);
        w.agents[2].strategy = Strategy::Binary(1);
        w.record();
        let t = w.stats.latest().unwrap();
        assert!(near(t.binary_c, 1.0 / 3.0) && near(t.binary_x, 1.0 / 3.0));
        assert!(near(t.binary_d, 1.0 / 3.0) && near(t.mean_k, 0.0));
    }

    #[test]
    fn frames_tile_the_groups_and_inspect_reports_the_agent() {
        let mut w = world(5, 7, Strategy::K(0), |c| {
            c.information = Information::Observers;
        });
        // Tiles of 3 × 3 in three columns and two rows, with one-cell gaps.
        assert_eq!(Model::size(&w), (11, 7));
        assert_eq!(w.cell(0), (0, 0));
        assert_eq!(w.cell(6), (0, 2));
        assert_eq!(w.cell(7), (4, 0));
        assert_eq!(w.cell(4 * 7), (4, 4));
        for i in 0..w.agents.len() {
            let (x, y) = w.cell(i);
            assert_eq!(w.at(x, y).1, Some(i));
        }
        assert_eq!(w.at(3, 0), (None, None), "a gap");
        assert_eq!(w.at(2, 2), (Some(0), None), "a tile's unused cell");
        assert_eq!(w.at(8, 4), (None, None), "no sixth group");
        w.play();
        let mut buf = Vec::new();
        for mode in ["strategy", "score", "payoff"] {
            Model::render(&w, mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 11 * 7 * 4);
        }
        assert!(Model::render(&w, "wealth", "", &mut buf).is_err());
        let v: serde_json::Value = serde_json::from_str(&w.inspect_json(4, 0).unwrap()).unwrap();
        assert_eq!(v["group"], 1);
        assert_eq!(v["agent"]["strategy"], "k = 0");
        assert_eq!(v["agent"]["class"], "k");
        assert_eq!(v["agent"]["cooperative"], true);
        assert!(v["agent"]["known"].is_u64());
        let v: serde_json::Value = serde_json::from_str(&w.inspect_json(3, 0).unwrap()).unwrap();
        assert_eq!((v["group"].is_null(), v["agent"].is_null()), (true, true));
        assert!(w.inspect(11, 0).is_err());
        let mut w = world(1, 100, Strategy::K(0), |_| {});
        w.play();
        let v: serde_json::Value = serde_json::from_str(&w.inspect_json(0, 0).unwrap()).unwrap();
        assert!(v["agent"]["known"].is_null(), "perfect information");
    }

    #[test]
    fn runs_stop_at_the_end_and_follow_their_seed() {
        let c = ImageConfig {
            end: 30,
            mutation: 0.01,
            ..Default::default()
        };
        let mut a = ImageWorld::new(c.clone(), 7).unwrap();
        let mut b = ImageWorld::new(c.clone(), 7).unwrap();
        let mut other = ImageWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(40);
        }
        assert_eq!(a.tick, 30);
        assert!(a.is_finished() && Model::finished(&a));
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
        assert_eq!(a.stats.history().len(), 31);
        let first = a.agents[0].id;
        assert!(a.locate(first).is_some() && a.locate(0).is_none());
        assert_eq!(a.agents_csv().lines().count(), 101);
        assert_eq!(a.series_csv().lines().count(), 32);
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(1, 10, Strategy::K(0), |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("c".to_string(), json!(0.25))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.c, 0.1);
        w.step();
        assert_eq!(w.config.c, 0.25);
    }

    #[test]
    fn keyframes_replay_under_every_switch() {
        let c = ImageConfig {
            groups: 3,
            group_size: 20,
            local: 0.7,
            mutation: 0.05,
            strategies: vec![Class::And, Class::Standing, Class::Q],
            ..Default::default()
        };
        let mut w = ImageWorld::new(c, 2).unwrap();
        type Edit = fn(&mut ImageConfig);
        let edits: [Edit; 6] = [
            |c| c.execution_error = 0.05,
            |c| c.perception_error = 0.05,
            |c| c.rounds_kind = RoundsKind::Random,
            |c| c.offset = Offset::None,
            |c| c.records = Records::Tally,
            |c| c.clamp = 3,
        ];
        for edit in edits {
            edit(&mut w.config);
            w.run(10);
        }
        let kept = w.clone();
        let mut back = kept.clone();
        back.run(20);
        let mut again = kept;
        again.run(20);
        assert_eq!(back.fingerprint(), again.fingerprint());
    }
}
```
Create `crates/sugarscape-core/src/image/analytic.rs` with only its tests for now (the Methods' equations and thresholds, the universal map, LH01's condition; the implementation goes above them in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn the_difference_equations_conserve_each_type_and_match_the_closed_forms() {
        for m in [
            Binary::ns98(),
            Binary {
                b: 1.0,
                c: 0.3,
                q: 0.6,
            },
        ] {
            for x in [0.1, 0.5, 0.9] {
                let mut s = State::start(x);
                for _ in 0..10 {
                    s = m.round(s);
                    assert!(close(s.x0 + s.x1, x, 1e-15) && close(s.y0 + s.y1, 1.0 - x, 1e-15));
                }
                for k in 1..=12 {
                    let a = m.di_minus_de(k, x);
                    let b = m.di_minus_de_iterated(k, x);
                    assert!(close(a, b, 1e-14), "{m:?} x {x} k {k}: {a} vs {b}");
                }
                // Round 1: De = bx/2, and a discriminator pays c half the time.
                assert!(close(m.de(1, x), m.b * x / 2.0, 1e-15));
                assert!(close(m.di_minus_de(1, x), -m.c / 2.0, 1e-15));
            }
        }
    }

    #[test]
    fn the_random_rounds_total_sums_the_rounds() {
        let m = Binary {
            b: 1.0,
            c: 0.2,
            q: 0.8,
        };
        for (w, x) in [(0.5, 0.3), (0.9, 0.7), (0.95, 0.99)] {
            let mut sum = 0.0;
            let mut wk = 1.0;
            for k in 1..=3000 {
                sum += 2.0 * wk * m.di_minus_de(k, x);
                wk *= w;
            }
            assert!(close(sum, m.advantage(w, x), 1e-12), "w {w} x {x}");
        }
        // The x = 1 limit agrees with x just below it.
        assert!(close(
            m.advantage(0.9, 1.0),
            m.advantage(0.9, 1.0 - 1e-9),
            1e-6
        ));
    }

    #[test]
    fn stability_needs_q_above_c_over_b_and_about_1_2_rounds() {
        let m = Binary::ns98();
        assert!(close(m.min_rounds(), 1.1 / 0.9, 1e-15));
        // 1/(1 − w) = 1.2 rounds falls just short; 1.25 is enough.
        assert!(!m.stable(1.0 - 1.0 / 1.2) && m.stable(1.0 - 1.0 / 1.25));
        let w_star = 1.0 - 1.0 / m.min_rounds();
        assert!(!m.stable(w_star - 1e-9) && m.stable(w_star + 1e-9));
        for q in [0.05, 0.1] {
            let m = Binary { q, ..m };
            assert_eq!(m.min_rounds(), f64::INFINITY);
            assert!(!m.stable(0.999_999));
        }
        let m = Binary { q: 0.2, ..m };
        assert!(close(m.min_rounds(), 3.0, 1e-12) && m.stable(1.0 - 1.0 / 3.1));
        // x_min falls below 1 exactly when stable, with q = 1 at c(2 − w)/(bw).
        let m = Binary::ns98();
        assert_eq!(m.x_min(1.0 - 1.0 / 1.2), None);
        let x = m.x_min(0.5).unwrap();
        assert!(close(x, 0.1 * 1.5 / 0.5, 1e-12), "{x}");
        assert!(close(m.advantage(0.5, x), 0.0, 1e-12));
    }

    #[test]
    fn cooperators_hold_even_with_defectors_at_c_2_minus_w_over_bwq() {
        let m = Binary {
            b: 1.0,
            c: 0.1,
            q: 0.9,
        };
        for w in [0.5, 0.8, 0.95] {
            let x = m.cooperator_equilibrium(w);
            assert!(close(m.cooperators_over_defectors(w, x), 0.0, 1e-15));
            assert!(m.cooperators_over_defectors(w, x * 1.01) > 0.0);
            assert!(m.cooperators_over_defectors(w, x * 0.99) < 0.0);
        }
        // Without cooperators the three-type formula is half the two-type one.
        for (w, x) in [(0.6, 0.4), (0.9, 0.8)] {
            let three = m.discriminators_over_defectors(w, x, 1.0 - x, 0.0);
            assert!(close(2.0 * three, m.advantage(w, x), 1e-12));
        }
    }

    #[test]
    fn fixed_rounds_thresholds_fall_with_more_rounds() {
        let m = Binary::ns98();
        assert_eq!(m.x_min_fixed(1), None, "one round: helping only costs");
        let xs: Vec<f64> = [2, 3, 5, 10]
            .iter()
            .map(|&k| m.x_min_fixed(k).unwrap())
            .collect();
        assert!(xs.windows(2).all(|p| p[1] < p[0]), "{xs:?}");
    }

    #[test]
    fn the_universal_map_conserves_mass_and_its_ends_are_absorbing() {
        let mut s = Scores::new(Start::Spread { below: 3 }, 0.4);
        for _ in 0..50 {
            s.round();
            assert!(close(s.x.iter().sum::<f64>(), 1.0, 1e-12));
        }
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 0.0, 10),
            Some(true)
        );
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 1.0, 10),
            Some(false)
        );
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 0.49, 100_000),
            Some(true)
        );
        assert_eq!(
            cooperates(Start::AtMinusOne { rest: Some(0) }, 0.51, 100_000),
            Some(false)
        );
        let s = Scores::new(Start::AtMinusOne { rest: None }, 0.3);
        assert!(close(s.phi(), 0.7, 1e-15) && s.x == [0.3]);
    }

    #[test]
    fn standing_is_a_best_reply_when_vrb_is_below_c_below_rb() {
        // LH01 Fig. 4b: e = ε = 0.025 gives v = 0.5, r = 0.833 — not met at c = 0.25.
        assert!(close(standing_r(100, 500), 499.0 / 599.0, 1e-15));
        assert_eq!(standing_v(0.025, 0.025), 0.5);
        assert!(!standing_stable(1.0, 0.25, 100, 500, 0.025, 0.025));
        // e = 0.04, ε = 0.01 (v = 0.2) meets it; so does ε = 0.
        assert!(standing_stable(1.0, 0.25, 100, 500, 0.04, 0.01));
        assert!(standing_stable(1.0, 0.25, 100, 500, 0.05, 0.0));
        assert!(
            !standing_stable(1.0, 0.9, 100, 500, 0.05, 0.0),
            "c above rb"
        );
    }
}
```
- [ ] **Step 2: Run the tests to see them fail**

Run: `cargo test -p sugarscape-core --lib image`
Expected: FAIL to compile — `file not found for module `presets`` and `stats`, then `cannot find type `ImageConfig` in this scope` and the like (the modules hold only tests).

- [ ] **Step 3: Implement the model**

Create `crates/sugarscape-core/src/image/stats.rs`:

```rust
//! Image scoring's statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 17] = [
    "help_rate",
    "mean_k",
    "cooperative",
    "mean_payoff",
    "mean_score",
    "k_cooperative",
    "k_defective",
    "h",
    "own_only",
    "and",
    "or",
    "standing",
    "binary_c",
    "binary_x",
    "binary_d",
    "q",
    "helps",
];

/// One generation's statistics: the rounds it played and the strategies
/// that played them (tick 0: the first generation before it plays). A
/// share or mean with nobody to count is NaN (JSON null).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ImageSnapshot {
    pub tick: u64,
    /// Helps ÷ rounds played (all groups).
    pub help_rate: f64,
    /// The mean k over agents whose strategy has one (k, AND, OR, binary).
    pub mean_k: f64,
    /// The share whose strategy helps at a generation's start
    /// (`Strategy::cooperative`: k ≤ 0 for the k strategies).
    pub cooperative: f64,
    pub mean_payoff: f64,
    pub mean_score: f64,
    /// Shares: k strategies with k ≤ 0 and k > 0; each other class; the
    /// binary cooperators, discriminators and defectors.
    pub k_cooperative: f64,
    pub k_defective: f64,
    pub h: f64,
    pub own_only: f64,
    pub and: f64,
    pub or: f64,
    pub standing: f64,
    pub binary_c: f64,
    pub binary_x: f64,
    pub binary_d: f64,
    pub q: f64,
    /// Helps given this generation.
    pub helps: u64,
}

impl Series for ImageSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "help_rate" => self.help_rate,
            "mean_k" => self.mean_k,
            "cooperative" => self.cooperative,
            "mean_payoff" => self.mean_payoff,
            "mean_score" => self.mean_score,
            "k_cooperative" => self.k_cooperative,
            "k_defective" => self.k_defective,
            "h" => self.h,
            "own_only" => self.own_only,
            "and" => self.and,
            "or" => self.or,
            "standing" => self.standing,
            "binary_c" => self.binary_c,
            "binary_x" => self.binary_x,
            "binary_d" => self.binary_d,
            "q" => self.q,
            "helps" => self.helps as f64,
            _ => return None,
        })
    }
}

/// `a / b`, or NaN when `b` is 0.
pub fn ratio(a: f64, b: u64) -> f64 {
    if b == 0 {
        f64::NAN
    } else {
        a / b as f64
    }
}
```
Put above the tests in `crates/sugarscape-core/src/image/strategy.rs`:

```rust
//! The strategy classes and their decisions: NS98's k (recipient's score at
//! least k), AND and OR (with the donor's own score below h); LH01's h (own
//! score below h), binary scorers, Sugden's standing and the q strategies.

use serde::{Deserialize, Serialize};

/// The lowest and highest k and h (NS98: k −5 … +6; FAIR23 draws h alike).
pub const K_MIN: i8 = -5;
pub const K_MAX: i8 = 6;

/// A class of strategies a run allows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    /// Help when the recipient's score is at least k (NS98).
    K,
    /// Help when one's own score is below h (LH01).
    H,
    /// Help when the recipient's score is at least k and one's own is below h.
    And,
    /// Help when the recipient's score is at least k or one's own is below h.
    Or,
    /// NS98's strategies that "only consider their own image": the same rule
    /// as `h`, counted apart.
    OwnOnly,
    /// LH01 Fig. 4's scorers with scores 0 and −1: cooperators (k −1),
    /// discriminators (k 0) and defectors (k 1).
    Binary,
    /// Sugden's standing strategy (LH01 §3).
    Standing,
    /// LH01's q strategies: help when helping is estimated to raise the
    /// chance of being helped by more than Δq.
    Q,
}

impl Class {
    pub const ALL: [Class; 8] = [
        Class::K,
        Class::H,
        Class::And,
        Class::Or,
        Class::OwnOnly,
        Class::Binary,
        Class::Standing,
        Class::Q,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Class::K => "k",
            Class::H => "h",
            Class::And => "and",
            Class::Or => "or",
            Class::OwnOnly => "own_only",
            Class::Binary => "binary",
            Class::Standing => "standing",
            Class::Q => "q",
        }
    }

    /// Every strategy of the class, in a fixed order (AND and OR: k outer, h inner).
    pub fn members(self) -> Vec<Strategy> {
        let ks = || K_MIN..=K_MAX;
        match self {
            Class::K => ks().map(Strategy::K).collect(),
            Class::H => ks().map(Strategy::H).collect(),
            Class::OwnOnly => ks().map(Strategy::OwnOnly).collect(),
            Class::And => ks()
                .flat_map(|k| ks().map(move |h| Strategy::And { k, h }))
                .collect(),
            Class::Or => ks()
                .flat_map(|k| ks().map(move |h| Strategy::Or { k, h }))
                .collect(),
            Class::Binary => (-1..=1).map(Strategy::Binary).collect(),
            Class::Standing => vec![Strategy::Standing],
            Class::Q => (1..=99).map(Strategy::Q).collect(),
        }
    }
}

/// One genotype. On the wire: `{"k": 0}`, `{"h": 1}`, `{"and": {"k": 0,
/// "h": 1}}`, `{"or": {…}}`, `{"own_only": 1}`, `{"binary": 0}`,
/// `"standing"`, `{"q": 25}` (Δq in hundredths).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    K(i8),
    H(i8),
    And { k: i8, h: i8 },
    Or { k: i8, h: i8 },
    OwnOnly(i8),
    Binary(i8),
    Standing,
    Q(u8),
}

/// What a donor knows when it decides.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Situation<'a> {
    /// The donor's own score.
    pub own: i32,
    /// The recipient's score as the donor sees it (0 when unknown).
    pub seen: i32,
    /// Whether the donor believes itself, and the recipient, in good standing.
    pub own_good: bool,
    pub their_good: bool,
    /// The group's q tallies, when q strategies play: x and y by score from `lo`.
    pub tallies: Option<&'a Tallies>,
}

impl Strategy {
    pub fn class(self) -> Class {
        match self {
            Strategy::K(_) => Class::K,
            Strategy::H(_) => Class::H,
            Strategy::And { .. } => Class::And,
            Strategy::Or { .. } => Class::Or,
            Strategy::OwnOnly(_) => Class::OwnOnly,
            Strategy::Binary(_) => Class::Binary,
            Strategy::Standing => Class::Standing,
            Strategy::Q(_) => Class::Q,
        }
    }

    /// The strategy's k, for the classes that have one.
    pub fn k(self) -> Option<i8> {
        match self {
            Strategy::K(k)
            | Strategy::Binary(k)
            | Strategy::And { k, .. }
            | Strategy::Or { k, .. } => Some(k),
            _ => None,
        }
    }

    /// Whether the strategy is one of its class's.
    pub fn is_valid(self) -> bool {
        let r = K_MIN..=K_MAX;
        match self {
            Strategy::K(v) | Strategy::H(v) | Strategy::OwnOnly(v) => r.contains(&v),
            Strategy::And { k, h } | Strategy::Or { k, h } => r.contains(&k) && r.contains(&h),
            Strategy::Binary(k) => (-1..=1).contains(&k),
            Strategy::Standing => true,
            Strategy::Q(d) => (1..=99).contains(&d),
        }
    }

    /// Whether the donor helps.
    pub fn helps(self, s: &Situation) -> bool {
        match self {
            Strategy::K(k) | Strategy::Binary(k) => s.seen >= i32::from(k),
            Strategy::H(h) | Strategy::OwnOnly(h) => s.own < i32::from(h),
            Strategy::And { k, h } => s.seen >= i32::from(k) && s.own < i32::from(h),
            Strategy::Or { k, h } => s.seen >= i32::from(k) || s.own < i32::from(h),
            Strategy::Standing => !s.own_good || s.their_good,
            Strategy::Q(d) => s
                .tallies
                .is_some_and(|t| t.gain(s.own) > f64::from(d) / 100.0),
        }
    }

    /// Whether it helps at the start of a generation: own and recipient's
    /// scores 0, both in good standing, the q tallies at their prior. NS98's
    /// "cooperative" (k ≤ 0: "they cooperate with individuals that have not
    /// had an interaction"), for every class.
    pub fn cooperative(self) -> bool {
        let prior = Tallies::new(-5, 5);
        self.helps(&Situation {
            own: 0,
            seen: 0,
            own_good: true,
            their_good: true,
            tallies: Some(&prior),
        })
    }

    /// A short label: "k = 0", "k = 0, h = 1 (AND)", "standing", "Δq = 0.25".
    pub fn label(self) -> String {
        match self {
            Strategy::K(k) => format!("k = {k}"),
            Strategy::H(h) => format!("h = {h}"),
            Strategy::OwnOnly(h) => format!("h = {h} (own score only)"),
            Strategy::And { k, h } => format!("k = {k}, h = {h} (AND)"),
            Strategy::Or { k, h } => format!("k = {k}, h = {h} (OR)"),
            Strategy::Binary(-1) => "cooperator (k = −1)".to_string(),
            Strategy::Binary(0) => "discriminator (k = 0)".to_string(),
            Strategy::Binary(k) => format!("defector (k = {k})"),
            Strategy::Standing => "standing".to_string(),
            Strategy::Q(d) => format!("Δq = {:.2}", f64::from(d) / 100.0),
        }
    }

    /// A number unique to the strategy (the fingerprint's).
    pub fn code(self) -> u64 {
        let v = |x: i8| u64::from(x as u8);
        match self {
            Strategy::K(k) => v(k),
            Strategy::H(h) => 1 << 16 | v(h),
            Strategy::And { k, h } => 2 << 16 | v(k) << 8 | v(h),
            Strategy::Or { k, h } => 3 << 16 | v(k) << 8 | v(h),
            Strategy::OwnOnly(h) => 4 << 16 | v(h),
            Strategy::Binary(k) => 5 << 16 | v(k),
            Strategy::Standing => 6 << 16,
            Strategy::Q(d) => 7 << 16 | u64::from(d),
        }
    }
}

/// LH01's q-strategy estimates for one group: `x[s]` counts rounds in which a
/// recipient with score s or lower was helped, `y[s]` rounds in which one
/// with score s or higher was not, over scores `lo` … `lo + len − 1`; the
/// prior is x = 1, y = 0 for s ≥ 0 and x = 0, y = 1 for s < 0.
#[derive(Clone, Debug, PartialEq)]
pub struct Tallies {
    pub lo: i32,
    pub x: Vec<u32>,
    pub y: Vec<u32>,
}

impl Tallies {
    pub fn new(lo: i32, hi: i32) -> Self {
        let len = (hi - lo + 1) as usize;
        let mut t = Tallies {
            lo,
            x: vec![0; len],
            y: vec![0; len],
        };
        t.reset();
        t
    }

    /// Back to the prior.
    pub fn reset(&mut self) {
        for (i, (x, y)) in self.x.iter_mut().zip(&mut self.y).enumerate() {
            let good = self.lo + i as i32 >= 0;
            (*x, *y) = (u32::from(good), u32::from(!good));
        }
    }

    fn hi(&self) -> i32 {
        self.lo + self.x.len() as i32 - 1
    }

    /// q_s = x_s / (x_s + y_s), the score clamped to the range.
    pub fn q(&self, s: i32) -> f64 {
        let i = (s.clamp(self.lo, self.hi()) - self.lo) as usize;
        f64::from(self.x[i]) / f64::from(self.x[i] + self.y[i])
    }

    /// q_{s+1} − q_{s−1}: the estimated gain from helping at score `s`.
    pub fn gain(&self, s: i32) -> f64 {
        self.q(s + 1) - self.q(s - 1)
    }

    /// Records a round: a recipient with score `score` was helped or not.
    pub fn record(&mut self, score: i32, helped: bool) {
        let i = (score.clamp(self.lo, self.hi()) - self.lo) as usize;
        if helped {
            self.x[i..].iter_mut().for_each(|v| *v += 1);
        } else {
            self.y[..=i].iter_mut().for_each(|v| *v += 1);
        }
    }
}
```
Put above the tests in `crates/sugarscape-core/src/image/config.rs`:

```rust
//! Image scoring's parameters: NS98's Fig. 1 by default (with LH01's
//! statement of the payoff offset), LH01's island model, errors, standing
//! and q strategies, and each unstated choice as a named switch.

use serde::{Deserialize, Serialize};

use super::strategy::{Class, Strategy};
use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// How many rounds a group plays in a generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoundsKind {
    /// Exactly `rounds` (NS98, LH01 §2).
    #[default]
    Fixed,
    /// Each round is the last with probability 1/`rounds` (LH01 §3's
    /// stability analysis): at least one, `rounds` on average.
    Random,
}

/// NS98's "to avoid negative payoffs we add 0.1 in each interaction".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Offset {
    /// c to both donor and recipient every round (LH01: "as did Nowak &
    /// Sigmund"; FAIR23 does the same).
    #[default]
    Both,
    /// Nothing added.
    None,
}

/// What a donor knows of a recipient's score.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Information {
    /// Everyone sees every interaction (NS98 Figs. 1–2, LH01).
    #[default]
    Perfect,
    /// Each interaction is seen by the recipient and on average `observers`
    /// others; each member keeps its own record, 0 when unknown (NS98 Fig. 3).
    Observers,
}

/// What an observer writes into its record of the donor.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Records {
    /// Its own record of the donor, one up or down by the action seen: what
    /// it has itself seen (FAIR23's `imagescoreothers`; NS98's Fig. 3
    /// reproduces with it).
    #[default]
    Tally,
    /// The donor's new score: its true score before the round, one up or
    /// down by the action seen (one sighting reveals the whole score).
    Score,
}

/// The first generation's strategies.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Initial {
    /// `"uniform"`: each agent's strategy uniform over the allowed set.
    #[default]
    #[serde(with = "uniform")]
    Uniform,
    /// `{"only": s}`: everyone plays `s`, but in each group the first
    /// round(`share` × n) members play `invader`.
    Seeded(Seeded),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Seeded {
    pub only: Strategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub invader: Option<Strategy>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub share: f64,
}

fn is_zero(x: &f64) -> bool {
    *x == 0.0
}

/// `Initial::Uniform` as the string "uniform".
mod uniform {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str("uniform")
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<(), D::Error> {
        let s = String::deserialize(d)?;
        if s == "uniform" {
            Ok(())
        } else {
            Err(serde::de::Error::custom(format!(
                "expected \"uniform\", not {s:?}"
            )))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ImageConfig {
    /// g groups of n.
    pub groups: u32,
    pub group_size: u32,
    /// p: the chance an offspring's parent is drawn from its own group
    /// rather than the whole population.
    pub local: f64,
    /// m: rounds per group per generation (fixed, or the mean).
    pub rounds: u32,
    pub rounds_kind: RoundsKind,
    /// Benefit to the recipient, cost to the donor, payoff at the start.
    pub b: f64,
    pub c: f64,
    pub u0: f64,
    pub offset: Offset,
    /// Scores stay in −clamp … +clamp (0: unbounded). Binary scorers
    /// always live in {−1, 0}.
    pub clamp: u32,
    pub information: Information,
    /// With `observers`: the mean number of members besides the pair who
    /// see an interaction (each with probability observers / (n − 2)).
    pub observers: f64,
    pub records: Records,
    /// e: a donor does the other action.
    pub execution_error: f64,
    /// ε: an observer sees the other action.
    pub perception_error: f64,
    /// ν: an offspring's strategy is redrawn uniformly from the allowed set.
    pub mutation: f64,
    /// The classes allowed (mutation and a uniform start draw from them).
    pub strategies: Vec<Class>,
    pub initial: Initial,
    /// The last generation; the run stops there (0: never).
    pub end: u32,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for ImageConfig {
    /// NS98 Fig. 1: one group of 100, k −5 … +6 uniform, m = 125, b = 1,
    /// c = 0.1 added to both players each round, scores clamped at ±5,
    /// perfect information, no errors, no mutation.
    fn default() -> Self {
        ImageConfig {
            groups: 1,
            group_size: 100,
            local: 1.0,
            rounds: 125,
            rounds_kind: RoundsKind::Fixed,
            b: 1.0,
            c: 0.1,
            u0: 0.0,
            offset: Offset::Both,
            clamp: 5,
            information: Information::Perfect,
            observers: 10.0,
            records: Records::Tally,
            execution_error: 0.0,
            perception_error: 0.0,
            mutation: 0.0,
            strategies: vec![Class::K],
            initial: Initial::Uniform,
            end: 0,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world (from the next generation);
/// every other field rebuilds it.
pub const LIVE: [&str; 14] = [
    "local",
    "rounds",
    "rounds_kind",
    "b",
    "c",
    "u0",
    "offset",
    "clamp",
    "observers",
    "records",
    "execution_error",
    "perception_error",
    "mutation",
    "end",
];

/// The most agents in a world, and the most in a group.
pub const MAX_AGENTS: u32 = 40_000;
pub const MAX_GROUP: u32 = 500;
pub const MAX_ROUNDS: u32 = 20_000;

impl ImageConfig {
    /// Every strategy the allowed classes contain, in `strategies` order.
    pub fn allowed(&self) -> Vec<Strategy> {
        self.strategies.iter().flat_map(|c| c.members()).collect()
    }

    /// The score range: {−1, 0} with binary scorers, else ±clamp (a
    /// million when unbounded, beyond any run's reach).
    pub fn score_range(&self) -> (i32, i32) {
        if self.strategies.contains(&Class::Binary) {
            (-1, 0)
        } else if self.clamp == 0 {
            (-1_000_000, 1_000_000)
        } else {
            (-(self.clamp as i32), self.clamp as i32)
        }
    }

    /// The chance each member besides the pair sees an interaction.
    pub fn watch_probability(&self) -> f64 {
        match self.information {
            Information::Perfect => 1.0,
            Information::Observers if self.group_size > 2 => {
                (self.observers / f64::from(self.group_size - 2)).min(1.0)
            }
            Information::Observers => 0.0,
        }
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut e = self.validate_fields();
        e.extend(self.validate_schedule());
        if e.is_empty() {
            Ok(())
        } else {
            Err(e)
        }
    }

    fn validate_fields(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        let mut check = |ok: bool, field: &str, message: &str| {
            if !ok {
                e.push(FieldError::new(field, message));
            }
        };
        let unit = |x: f64| (0.0..=1.0).contains(&x);
        let size_ok = (2..=MAX_GROUP).contains(&self.group_size);
        check(size_ok, "group_size", "must be between 2 and 500");
        let groups_ok = (1..=400).contains(&self.groups);
        check(groups_ok, "groups", "must be between 1 and 400");
        check(
            !(size_ok && groups_ok) || self.groups * self.group_size <= MAX_AGENTS,
            "groups",
            "groups × group size must be at most 40,000",
        );
        check(unit(self.local), "local", "must be between 0 and 1");
        check(
            (1..=MAX_ROUNDS).contains(&self.rounds),
            "rounds",
            "must be between 1 and 20,000",
        );
        for (field, v) in [("b", self.b), ("c", self.c), ("u0", self.u0)] {
            check(
                v.is_finite() && (0.0..=1000.0).contains(&v),
                field,
                "must be a number between 0 and 1,000",
            );
        }
        check(self.clamp <= 100, "clamp", "must be between 0 and 100");
        check(
            self.observers.is_finite() && (0.0..=f64::from(MAX_GROUP)).contains(&self.observers),
            "observers",
            "must be a number between 0 and 500",
        );
        check(
            unit(self.execution_error),
            "execution_error",
            "must be between 0 and 1",
        );
        check(
            unit(self.perception_error),
            "perception_error",
            "must be between 0 and 1",
        );
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        e.extend(self.validate_strategies());
        e
    }

    fn validate_strategies(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        let s = &self.strategies;
        let mut sorted = s.clone();
        sorted.sort();
        sorted.dedup();
        if s.is_empty() {
            e.push(FieldError::new(
                "strategies",
                "must list at least one class",
            ));
        } else if sorted.len() != s.len() {
            e.push(FieldError::new("strategies", "must not repeat a class"));
        } else if s.contains(&Class::Binary)
            && s.iter()
                .any(|c| !matches!(c, Class::Binary | Class::Standing))
        {
            e.push(FieldError::new(
                "strategies",
                "binary scorers (scores 0 and −1) combine only with standing",
            ));
        } else if s.contains(&Class::Q) && self.clamp == 0 {
            e.push(FieldError::new(
                "strategies",
                "q strategies need bounded scores (clamp at least 1)",
            ));
        }
        if let Initial::Seeded(seed) = &self.initial {
            let allowed = |x: &Strategy| x.is_valid() && s.contains(&x.class());
            if !allowed(&seed.only) || !seed.invader.as_ref().is_none_or(allowed) {
                e.push(FieldError::new(
                    "initial",
                    "the starting strategies must be of the allowed classes",
                ));
            } else if !(0.0..=1.0).contains(&seed.share) {
                e.push(FieldError::new("initial", "share must be between 0 and 1"));
            }
        }
        e
    }

    /// Schedule entries: live paths only, values that validate.
    fn validate_schedule(&self) -> Vec<FieldError> {
        let mut e = Vec::new();
        for change in &self.schedule {
            for (path, value) in &change.set {
                if !LIVE.contains(&path.as_str()) {
                    e.push(FieldError::new(
                        "schedule",
                        format!("{path} changes only on reset and cannot be scheduled"),
                    ));
                    continue;
                }
                match ModelConfig::Image(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Image(next)) => {
                        for f in next.validate_fields() {
                            e.push(FieldError::new(
                                "schedule",
                                format!("tick {}: {}: {}", change.tick, f.field, f.message),
                            ));
                        }
                    }
                    Ok(_) => unreachable!("with_path keeps the model"),
                    Err(f) => e.push(f),
                }
            }
        }
        e
    }

    /// The reset-only fields that differ from `next`.
    pub fn changes(&self, next: &ImageConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !LIVE.contains(&k.as_str()))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields (`strategies` and `initial` come from presets,
/// files and links).
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::number("Game", "b", "Benefit b", (0.0, 10.0, 0.05), Live)
            .with_help("What the recipient gains when helped"),
        Param::number("Game", "c", "Cost c", (0.0, 10.0, 0.05), Live)
            .with_help("What helping costs the donor"),
        Param::number("Game", "u0", "Initial payoff u₀", (0.0, 50.0, 0.5), Live),
        Param::choice(
            "Game",
            "offset",
            "Payoff offset",
            &[
                ("both", "c to donor and recipient each round (LH01)"),
                ("none", "None"),
            ],
            Live,
        ),
        Param::integer("Population", "groups", "Groups", (1, 400), Reset),
        Param::integer("Population", "group_size", "Group size", (2, 500), Reset),
        Param::number(
            "Population",
            "local",
            "Local parents",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("The chance an offspring's parent comes from its own group (LH01's p)"),
        Param::integer(
            "Rounds",
            "rounds",
            "Rounds per generation",
            (1, 20_000),
            Live,
        ),
        Param::choice(
            "Rounds",
            "rounds_kind",
            "Number of rounds",
            &[
                ("fixed", "Fixed"),
                ("random", "Random (each the last with chance 1/m)"),
            ],
            Live,
        ),
        Param::choice(
            "Information",
            "information",
            "Information",
            &[
                ("perfect", "Perfect: everyone sees everything"),
                ("observers", "Observers: each keeps its own record"),
            ],
            Reset,
        ),
        Param::number(
            "Information",
            "observers",
            "Observers per interaction",
            (0.0, 100.0, 1.0),
            Live,
        )
        .with_help("Besides the recipient, who always sees (NS98: 10)")
        .shown_if("information", "observers"),
        Param::choice(
            "Information",
            "records",
            "An observer records",
            &[
                ("tally", "Its own record ± 1 (FAIR23)"),
                ("score", "The donor's new score"),
            ],
            Live,
        ),
        Param::integer(
            "Information",
            "clamp",
            "Score limit (0: none)",
            (0, 100),
            Live,
        ),
        Param::number(
            "Errors",
            "execution_error",
            "Execution error",
            (0.0, 1.0, 0.005),
            Live,
        )
        .with_help("The chance a donor does the other action"),
        Param::number(
            "Errors",
            "perception_error",
            "Perception error",
            (0.0, 1.0, 0.005),
            Live,
        )
        .with_help("The chance an observer sees the other action"),
        Param::number(
            "Evolution",
            "mutation",
            "Mutation rate",
            (0.0, 0.1, 0.0001),
            Live,
        )
        .with_help("The chance an offspring's strategy is redrawn from all allowed"),
        Param::integer(
            "Run",
            "end",
            "Last generation (0: never)",
            (0, 100_000),
            Live,
        ),
    ]
}
```
Put above the tests in `crates/sugarscape-core/src/image/world.rs`:

```rust
//! Image scoring's world: g groups of n agents with fixed strategies. A tick
//! is a generation: the agents' scores and payoffs start afresh, each group
//! plays its rounds of one random donor and one random recipient, and the
//! next generation's parents are drawn in proportion to payoff, from the
//! offspring's own group or (LH01's island model) the whole population. The
//! world keeps the generation that last played, its scores, payoffs and
//! records, for the frame and Inspect; its offspring are drawn when the next
//! tick begins.

use std::fmt::Write;
use std::sync::Arc;

use rand::Rng;
use serde::Serialize;

use super::config::{ImageConfig, Information, Initial, Offset, Records, RoundsKind};
use super::stats::{ratio, ImageSnapshot, SERIES};
use super::strategy::{Class, Situation, Strategy, Tallies};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{
    lerp, Rgb, BACKGROUND, BLUE, COOL, FEMALE, HOT, NEUTRAL, POLLUTION, RED, SUGAR,
};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

/// A mark's bits: the member believes the other in good standing; it has
/// seen the other act this generation.
const GOOD: u8 = 1;
const SEEN: u8 = 2;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageMode {
    Strategy,
    Score,
    Payoff,
}

impl std::str::FromStr for ImageMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "strategy" => Self::Strategy,
            "score" => Self::Score,
            "payoff" => Self::Payoff,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// One member of a group, with this generation's results.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: u64,
    pub strategy: Strategy,
    /// The true image score (its own record) and standing.
    pub score: i32,
    pub good: bool,
    pub payoff: f64,
    /// Helps given and received this generation.
    pub given: u32,
    pub received: u32,
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ImageInspection {
    pub cell: Cell,
    /// The group the cell's tile belongs to, or none (a gap).
    pub group: Option<u32>,
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Cell {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    pub group: u32,
    /// "k = 0", "k = 0, h = 1 (AND)", "standing" …
    pub strategy: String,
    pub class: &'static str,
    /// Whether the strategy helps at a generation's start.
    pub cooperative: bool,
    pub score: i32,
    /// Good standing (as everyone would judge it without perception errors).
    pub standing: bool,
    /// With private records: the members (besides itself) who have seen it
    /// act this generation, and the mean of their records of its score;
    /// none with perfect information (everyone knows it).
    pub known: Option<u32>,
    pub mean_view: Option<f64>,
    pub payoff: f64,
    pub given: u32,
    pub received: u32,
}

#[derive(Clone)]
pub struct ImageWorld {
    pub config: ImageConfig,
    /// Generations played.
    pub tick: u64,
    /// Group-major: group g's members are `g·n .. (g + 1)·n`.
    agents: Vec<Agent>,
    /// The allowed strategies (fixed once built; keyframes share them).
    allowed: Arc<Vec<Strategy>>,
    /// Private records, kept only with observers or perception errors:
    /// `views[g·n² + j·n + i]` is member j's record of member i's score in
    /// group g, `marks` at the same index its belief in i's standing and
    /// whether it has seen i act.
    views: Vec<i16>,
    marks: Vec<u8>,
    /// The q strategies' tallies, one per group (empty without them).
    tallies: Vec<Tallies>,
    /// Helps and rounds in the generation that last played.
    helps: u64,
    rounds_played: u64,
    next_id: u64,
    rng: SimRng,
    pub stats: Stats<ImageSnapshot>,
}

/// The smallest s with s² ≥ v.
fn ceil_sqrt(v: u32) -> u32 {
    let mut s = 0;
    while s * s < v {
        s += 1;
    }
    s
}

/// The index of a draw `x` in `[0, total)` from cumulative weights `cum`
/// (the first entry above `x`), or `None` when every weight is 0.
fn roulette(cum: &[f64], x: f64) -> Option<usize> {
    let total = *cum.last()?;
    if total <= 0.0 {
        return None;
    }
    let i = cum.partition_point(|&v| v <= x);
    // x rounded up to the total: the last member with any weight.
    Some(if i < cum.len() {
        i
    } else {
        cum.partition_point(|&v| v < total)
    })
}

impl ImageWorld {
    pub fn new(config: ImageConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let allowed = config.allowed();
        let mut w = ImageWorld {
            config,
            tick: 0,
            agents: Vec::new(),
            allowed: Arc::new(allowed),
            views: Vec::new(),
            marks: Vec::new(),
            tallies: Vec::new(),
            helps: 0,
            rounds_played: 0,
            next_id: 1,
            rng: rng::seeded(seed),
            stats: Stats::default(),
        };
        let n = w.config.group_size as usize;
        for _ in 0..w.config.groups {
            for slot in 0..n {
                let strategy = match &w.config.initial {
                    Initial::Uniform => w.random_strategy(),
                    Initial::Seeded(s) => {
                        let invaders = (s.share * n as f64).round() as usize;
                        match s.invader {
                            Some(x) if slot < invaders => x,
                            _ => s.only,
                        }
                    }
                };
                w.add(strategy);
            }
        }
        w.reset_results();
        w.record();
        Ok(w)
    }

    fn random_strategy(&mut self) -> Strategy {
        let k = self.rng.gen_range(0..self.allowed.len() as u32) as usize;
        self.allowed[k]
    }

    fn add(&mut self, strategy: Strategy) {
        self.agents.push(Agent {
            id: self.next_id,
            strategy,
            score: 0,
            good: true,
            payoff: self.config.u0,
            given: 0,
            received: 0,
        });
        self.next_id += 1;
    }

    fn n(&self) -> usize {
        self.config.group_size as usize
    }

    /// Every agent, group by group.
    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    /// Group `g`'s members.
    pub fn group(&self, g: usize) -> &[Agent] {
        let n = self.n();
        &self.agents[g * n..(g + 1) * n]
    }

    pub fn population(&self) -> usize {
        self.agents.len()
    }

    /// The allowed strategies, in the order a mutation or uniform start draws them.
    pub fn allowed(&self) -> &[Strategy] {
        &self.allowed
    }

    /// Whether the run has reached its last generation.
    pub fn is_finished(&self) -> bool {
        self.config.end > 0 && self.tick >= u64::from(self.config.end)
    }

    /// Helps and rounds in the generation that last played.
    pub fn helps(&self) -> (u64, u64) {
        (self.helps, self.rounds_played)
    }

    /// Whether this generation keeps private records.
    pub fn private(&self) -> bool {
        !self.views.is_empty()
    }

    /// One generation: offspring of the last (after the first), then play.
    pub fn step(&mut self) {
        self.apply_schedule();
        if self.tick > 0 {
            self.reproduce();
        }
        self.play();
        self.tick += 1;
        self.record();
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.is_finished() {
                break;
            }
            self.step();
        }
    }

    /// Applies schedule entries due at the tick about to run.
    fn apply_schedule(&mut self) {
        let t = self.tick;
        let due: Vec<_> = self
            .config
            .schedule
            .iter()
            .filter(|c| c.tick == t)
            .flat_map(|c| c.set.clone())
            .collect();
        for (path, value) in due {
            let next = ModelConfig::Image(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Image(next)) = next {
                self.config = next;
            }
        }
    }

    /// Rule 1: scores 0, standing good, payoffs u₀, records unknown, the q
    /// tallies at their prior.
    fn reset_results(&mut self) {
        let u0 = self.config.u0;
        for a in &mut self.agents {
            (a.score, a.good, a.payoff, a.given, a.received) = (0, true, u0, 0, 0);
        }
        let private =
            self.config.information == Information::Observers || self.config.perception_error > 0.0;
        let (g, n) = (self.config.groups as usize, self.n());
        self.views.clear();
        self.marks.clear();
        if private {
            self.views.resize(g * n * n, 0);
            self.marks.resize(g * n * n, GOOD);
        }
        self.tallies.clear();
        if self.config.strategies.contains(&Class::Q) {
            let (lo, hi) = self.config.score_range();
            self.tallies.resize(g, Tallies::new(lo, hi));
        }
        self.helps = 0;
        self.rounds_played = 0;
    }

    /// Rule 2: each group plays its rounds.
    fn play(&mut self) {
        self.reset_results();
        let n = self.n() as u32;
        let m = self.config.rounds;
        for g in 0..self.config.groups as usize {
            let mut played = 0;
            loop {
                let d = self.rng.gen_range(0..n);
                let mut r = self.rng.gen_range(0..n - 1);
                if r >= d {
                    r += 1;
                }
                self.interact(g, d as usize, r as usize);
                played += 1;
                let last = match self.config.rounds_kind {
                    RoundsKind::Fixed => played >= m,
                    RoundsKind::Random => self.rng.gen::<f64>() < 1.0 / f64::from(m),
                };
                if last {
                    break;
                }
            }
        }
    }

    /// One round in group `g`: member `d` may help member `r`.
    fn interact(&mut self, g: usize, d: usize, r: usize) {
        let n = self.n();
        let (di, ri) = (g * n + d, g * n + r);
        let row = g * n * n + d * n;
        let private = self.private();
        let (seen, own_good, their_good) = if private {
            (
                i32::from(self.views[row + r]),
                self.marks[row + d] & GOOD != 0,
                self.marks[row + r] & GOOD != 0,
            )
        } else {
            (
                self.agents[ri].score,
                self.agents[di].good,
                self.agents[ri].good,
            )
        };
        let situation = Situation {
            own: self.agents[di].score,
            seen,
            own_good,
            their_good,
            tallies: self.tallies.get(g),
        };
        let mut help = self.agents[di].strategy.helps(&situation);
        let e = self.config.execution_error;
        if e > 0.0 && self.rng.gen::<f64>() < e {
            help = !help;
        }
        let (b, c) = (self.config.b, self.config.c);
        let bonus = match self.config.offset {
            Offset::Both => c,
            Offset::None => 0.0,
        };
        let (old, r_score, r_good) = (
            self.agents[di].score,
            self.agents[ri].score,
            self.agents[ri].good,
        );
        let new = self.moved(old, help);
        let donor = &mut self.agents[di];
        if help {
            donor.payoff -= c;
            donor.given += 1;
        }
        donor.payoff += bonus;
        donor.score = new;
        if help {
            donor.good = true;
        } else if r_good {
            donor.good = false;
        }
        let recipient = &mut self.agents[ri];
        if help {
            recipient.payoff += b;
            recipient.received += 1;
        }
        recipient.payoff += bonus;
        if let Some(t) = self.tallies.get_mut(g) {
            t.record(r_score, help);
        }
        self.helps += u64::from(help);
        self.rounds_played += 1;
        if private {
            self.observe(g, d, r, old, help);
        }
    }

    /// A score one up (helped) or down, within the range.
    fn moved(&self, score: i32, up: bool) -> i32 {
        let (lo, hi) = self.config.score_range();
        (score + if up { 1 } else { -1 }).clamp(lo, hi)
    }

    /// Updates the private records after `d` helped `r` or not: the donor
    /// judges its own standing by its own record of the recipient; the
    /// recipient always, and each other member with the watch probability,
    /// sees the action (the other one with probability ε) and records it.
    fn observe(&mut self, g: usize, d: usize, r: usize, old: i32, help: bool) {
        let n = self.n();
        let base = g * n * n;
        let judge = |marks: &[u8], row: usize, help: bool| {
            let mut mark = marks[row + d] | SEEN;
            if help {
                mark |= GOOD;
            } else if marks[row + r] & GOOD != 0 {
                mark &= !GOOD;
            }
            mark
        };
        let own = base + d * n;
        self.marks[own + d] = judge(&self.marks, own, help);
        let p = self.config.watch_probability();
        let eps = self.config.perception_error;
        for j in 0..n {
            if j == d || (j != r && p < 1.0 && self.rng.gen::<f64>() >= p) {
                continue;
            }
            let mut saw = help;
            if eps > 0.0 && self.rng.gen::<f64>() < eps {
                saw = !saw;
            }
            let row = base + j * n;
            let from = match self.config.records {
                Records::Score => old,
                Records::Tally => i32::from(self.views[row + d]),
            };
            self.views[row + d] = self.moved(from, saw) as i16;
            self.marks[row + d] = judge(&self.marks, row, saw);
        }
    }

    /// Rule 3: each new member's parent by payoff-proportional roulette from
    /// its own group (probability p) or the whole population, its strategy
    /// redrawn with probability ν. Negative payoffs (only possible without
    /// the offset) weigh 0; a pool with no weight at all is drawn uniformly.
    fn reproduce(&mut self) {
        let (g, n) = (self.config.groups as usize, self.n());
        let mut local = Vec::with_capacity(self.agents.len());
        let mut global = Vec::with_capacity(self.agents.len());
        let (mut in_group, mut all) = (0.0, 0.0);
        for (i, a) in self.agents.iter().enumerate() {
            if i % n == 0 {
                in_group = 0.0;
            }
            let w = a.payoff.max(0.0);
            in_group += w;
            all += w;
            local.push(in_group);
            global.push(all);
        }
        let p = self.config.local;
        let nu = self.config.mutation;
        let mut next = Vec::with_capacity(self.agents.len());
        for group in 0..g {
            for _ in 0..n {
                let home = g == 1 || p >= 1.0 || (p > 0.0 && self.rng.gen::<f64>() < p);
                let (pool, offset) = if home {
                    (&local[group * n..(group + 1) * n], group * n)
                } else {
                    (&global[..], 0)
                };
                let x = self.rng.gen::<f64>() * pool.last().copied().unwrap_or(0.0);
                let parent = match roulette(pool, x) {
                    Some(i) => i,
                    None => self.rng.gen_range(0..pool.len() as u32) as usize,
                } + offset;
                let mut strategy = self.agents[parent].strategy;
                if nu > 0.0 && self.rng.gen::<f64>() < nu {
                    strategy = self.random_strategy();
                }
                next.push(strategy);
            }
        }
        self.agents.clear();
        for s in next {
            self.add(s);
        }
    }

    fn record(&mut self) {
        let (mut coop, mut k_sum, mut k_count) = (0u64, 0.0, 0u64);
        let (mut payoff, mut score) = (0.0, 0.0);
        let mut class = [0u64; 8];
        let (mut k_coop, mut k_def) = (0u64, 0u64);
        let mut binary = [0u64; 3];
        for a in &self.agents {
            let s = a.strategy;
            coop += u64::from(s.cooperative());
            if let Some(k) = s.k() {
                k_sum += f64::from(k);
                k_count += 1;
            }
            payoff += a.payoff;
            score += f64::from(a.score);
            class[Class::ALL.iter().position(|&c| c == s.class()).unwrap()] += 1;
            match s {
                Strategy::K(k) if k <= 0 => k_coop += 1,
                Strategy::K(_) => k_def += 1,
                Strategy::Binary(k) => binary[(k + 1) as usize] += 1,
                _ => {}
            }
        }
        let total = self.agents.len() as u64;
        let share = |c: u64| ratio(c as f64, total);
        let s = ImageSnapshot {
            tick: self.tick,
            help_rate: ratio(self.helps as f64, self.rounds_played),
            mean_k: ratio(k_sum, k_count),
            cooperative: share(coop),
            mean_payoff: ratio(payoff, total),
            mean_score: ratio(score, total),
            k_cooperative: share(k_coop),
            k_defective: share(k_def),
            h: share(class[1]),
            and: share(class[2]),
            or: share(class[3]),
            own_only: share(class[4]),
            standing: share(class[6]),
            binary_c: share(binary[0]),
            binary_x: share(binary[1]),
            binary_d: share(binary[2]),
            q: share(class[7]),
            helps: self.helps,
        };
        self.stats.push(s);
    }

    /// The frame's tiles: each group a square of side `tile`, groups in
    /// `cols` columns with one-cell gaps.
    fn layout(&self) -> (u32, u32, u32) {
        let tile = ceil_sqrt(self.config.group_size);
        let cols = ceil_sqrt(self.config.groups);
        let rows = self.config.groups.div_ceil(cols);
        (tile, cols, rows)
    }

    /// Agent `i`'s cell.
    pub fn cell(&self, i: usize) -> (u32, u32) {
        let (tile, cols, _) = self.layout();
        let n = self.n();
        let (g, s) = ((i / n) as u32, (i % n) as u32);
        let (gx, gy) = (g % cols, g / cols);
        (gx * (tile + 1) + s % tile, gy * (tile + 1) + s / tile)
    }

    /// The group and agent (index) at a cell, if any.
    fn at(&self, x: u32, y: u32) -> (Option<u32>, Option<usize>) {
        let (tile, cols, rows) = self.layout();
        let (gx, ox, gy, oy) = (
            x / (tile + 1),
            x % (tile + 1),
            y / (tile + 1),
            y % (tile + 1),
        );
        if ox == tile || oy == tile || gx >= cols || gy >= rows {
            return (None, None);
        }
        let g = gy * cols + gx;
        if g >= self.config.groups {
            return (None, None);
        }
        let s = oy * tile + ox;
        let agent = (s < self.config.group_size).then(|| (g * self.config.group_size + s) as usize);
        (Some(g), agent)
    }

    fn color(&self, a: &Agent, mode: ImageMode, top: f64) -> Rgb {
        match mode {
            ImageMode::Strategy => match a.strategy {
                Strategy::Binary(k) => lerp(BLUE, RED, f64::from(k + 1) / 2.0),
                Strategy::H(_) => POLLUTION,
                Strategy::OwnOnly(_) => NEUTRAL,
                Strategy::Standing => SUGAR,
                Strategy::Q(_) => FEMALE,
                s => lerp(BLUE, RED, f64::from(s.k().unwrap_or(0) + 5) / 11.0),
            },
            ImageMode::Score => {
                let (lo, hi) = self.config.score_range();
                let (lo, hi) = (f64::from(lo.max(-5)), f64::from(hi.min(5)));
                let s = f64::from(a.score);
                if s >= 0.0 {
                    lerp(NEUTRAL, BLUE, if hi > 0.0 { s / hi } else { 0.0 })
                } else {
                    lerp(NEUTRAL, RED, s / lo)
                }
            }
            ImageMode::Payoff => lerp(COOL, HOT, if top > 0.0 { a.payoff / top } else { 0.0 }),
        }
    }

    /// Member `i`'s record among the others in its group: how many have
    /// seen it act, and their mean record of its score.
    fn known(&self, i: usize) -> (u32, Option<f64>) {
        let n = self.n();
        let (g, s) = (i / n, i % n);
        let (mut count, mut sum) = (0u32, 0.0);
        for j in (0..n).filter(|&j| j != s) {
            let at = g * n * n + j * n + s;
            if self.marks[at] & SEEN != 0 {
                count += 1;
                sum += f64::from(self.views[at]);
            }
        }
        (count, (count > 0).then(|| sum / f64::from(count)))
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<ImageInspection, String> {
        let (w, h) = Model::size(self);
        if x >= w || y >= h {
            return Err(format!("({x}, {y}) is outside the frame"));
        }
        let (group, index) = self.at(x, y);
        let agent = index.map(|i| {
            let a = &self.agents[i];
            let (known, mean_view) = if self.private() {
                let (k, m) = self.known(i);
                (Some(k), m)
            } else {
                (None, None)
            };
            AgentView {
                id: a.id,
                group: (i / self.n()) as u32,
                strategy: a.strategy.label(),
                class: a.strategy.class().as_str(),
                cooperative: a.strategy.cooperative(),
                score: a.score,
                standing: a.good,
                known,
                mean_view,
                payoff: a.payoff,
                given: a.given,
                received: a.received,
            }
        });
        Ok(ImageInspection {
            cell: Cell { x, y },
            group,
            agent,
        })
    }
}

/// A strategy's name in the agents CSV (no commas).
fn csv_name(s: Strategy) -> String {
    match s {
        Strategy::K(k) => format!("k={k}"),
        Strategy::H(h) => format!("h={h}"),
        Strategy::OwnOnly(h) => format!("own_only h={h}"),
        Strategy::And { k, h } => format!("and k={k} h={h}"),
        Strategy::Or { k, h } => format!("or k={k} h={h}"),
        Strategy::Binary(k) => format!("binary k={k}"),
        Strategy::Standing => "standing".to_string(),
        Strategy::Q(d) => format!("q {:.2}", f64::from(d) / 100.0),
    }
}

impl Model for ImageWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Image(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        ImageWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        ImageWorld::population(self)
    }

    /// FNV-1a over the tick, the id counter and every agent in order.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        eat(self.next_id);
        for a in &self.agents {
            eat(a.id);
            eat(a.strategy.code());
            eat(i64::from(a.score) as u64 ^ u64::from(a.good) << 40);
            eat(a.payoff.to_bits());
            eat(u64::from(a.given) | u64::from(a.received) << 32);
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        let (tile, cols, rows) = self.layout();
        (cols * (tile + 1) - 1, rows * (tile + 1) - 1)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: ImageMode = mode.parse()?;
        let (w, h) = Model::size(self);
        buf.clear();
        buf.resize((w * h * 4) as usize, 0);
        let px = buf.as_chunks_mut::<4>().0;
        for p in px.iter_mut() {
            p.copy_from_slice(&[BACKGROUND[0], BACKGROUND[1], BACKGROUND[2], 255]);
        }
        let top = self.agents.iter().map(|a| a.payoff).fold(0.0, f64::max);
        for (i, a) in self.agents.iter().enumerate() {
            let (x, y) = self.cell(i);
            let rgb = self.color(a, mode, top);
            px[(y * w + x) as usize].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
        Ok(())
    }

    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).expect("snapshot serializes")
    }

    fn series_names(&self) -> Vec<String> {
        SERIES.iter().map(|s| s.to_string()).collect()
    }

    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }

    fn series_csv(&self) -> String {
        export::history_csv(&self.series_names(), self.stats.history())
    }

    fn agents_csv(&self) -> String {
        let mut out = String::from("id,group,x,y,strategy,score,standing,payoff,given,received\n");
        for (i, a) in self.agents.iter().enumerate() {
            let (x, y) = self.cell(i);
            writeln!(
                out,
                "{},{},{x},{y},{},{},{},{},{},{}",
                a.id,
                i / self.n(),
                csv_name(a.strategy),
                a.score,
                a.good,
                a.payoff,
                a.given,
                a.received
            )
            .unwrap();
        }
        out
    }

    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y)?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        let i = self.agents.iter().position(|a| a.id == id)?;
        Some(self.cell(i))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Image(next) = next else {
            return Err(wrong_model(ModelKind::Image, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        Ok(())
    }

    fn finished(&self) -> bool {
        self.is_finished()
    }
}
```
Put above the tests in `crates/sugarscape-core/src/image/analytic.rs`:

```rust
//! NS98's Methods, computed: the binary-score model of discriminators and
//! defectors (its difference equations, payoffs, the threshold x_min, the
//! stability conditions q > c/b and 1/(1 − w) > (bq + c)/(bq − c)), the
//! equilibrium with unconditional cooperators, and the "universal
//! constant" map with a threshold finder for several starting
//! distributions; and LH01's condition for the standing strategy. Only
//! `+ − × ÷` (no platform `powf`), so results are the same everywhere.

/// NS98's binary model: benefit b, cost c, the chance q that a
/// discriminator knows the recipient's image (and the prior p = 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Binary {
    pub b: f64,
    pub c: f64,
    pub q: f64,
}

/// The frequencies of discriminators with image 0 and 1, and of defectors
/// with image 0 and 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct State {
    pub x0: f64,
    pub x1: f64,
    pub y0: f64,
    pub y1: f64,
}

impl State {
    /// A generation's first round: everyone's image is 1 (the payoffs in
    /// round 1, De(1) = bx/2, say so).
    pub fn start(x: f64) -> Self {
        State {
            x0: 0.0,
            x1: x,
            y0: 0.0,
            y1: 1.0 - x,
        }
    }
}

/// `v`^k by repeated multiplication.
fn power(v: f64, k: u32) -> f64 {
    (0..k).fold(1.0, |acc, _| acc * v)
}

impl Binary {
    /// NS98's example (Figs. 1–2): b = 1, c = 0.1, q = 1.
    pub fn ns98() -> Self {
        Binary {
            b: 1.0,
            c: 0.1,
            q: 1.0,
        }
    }

    /// One round of the difference equations: x₀′ = [x₀ + x(1 − φ)q]/2,
    /// x₁′ = [x₁ + x(1 − q + qφ)]/2, y₀′ = [y₀ + y]/2, y₁′ = y₁/2, with
    /// φ = x₁ + y₁.
    pub fn round(&self, s: State) -> State {
        let (x, y, phi, q) = (s.x0 + s.x1, s.y0 + s.y1, s.x1 + s.y1, self.q);
        State {
            x0: (s.x0 + x * (1.0 - phi) * q) / 2.0,
            x1: (s.x1 + x * (1.0 - q + q * phi)) / 2.0,
            y0: (s.y0 + y) / 2.0,
            y1: s.y1 / 2.0,
        }
    }

    /// The round's expected payoffs to discriminators of image 0 and 1 and
    /// to defectors of image 0 and 1.
    pub fn payoffs(&self, s: State) -> [f64; 4] {
        let (b, c, q) = (self.b, self.c, self.q);
        let (x, phi) = (s.x0 + s.x1, s.x1 + s.y1);
        let cost = -c * (1.0 - q + q * phi);
        [
            (cost + b * x * (1.0 - q)) / 2.0,
            (cost + b * x) / 2.0,
            b * x * (1.0 - q) / 2.0,
            b * x / 2.0,
        ]
    }

    /// De(k): a defector's expected payoff in round k (from 1).
    pub fn de(&self, k: u32, x: f64) -> f64 {
        let (b, q) = (self.b, self.q);
        b * x * (1.0 - q + q * power(0.5, k - 1)) / 2.0
    }

    /// Di(k) − De(k), NS98's closed form; at qx = 1 (q = x = 1) its limit
    /// [(b − c) − b·2^−(k−1)]/2.
    pub fn di_minus_de(&self, k: u32, x: f64) -> f64 {
        let (b, c, q) = (self.b, self.c, self.q);
        let qx = 1.0 - q * x;
        if qx == 0.0 {
            return (b - c - b * power(0.5, k - 1)) / 2.0;
        }
        ((1.0 - q) * (b * q * x - c) / qx - b * q * power(0.5, k - 1)
            + q * (b - c) * (1.0 - x) / qx * power((1.0 + q * x) / 2.0, k - 1))
            / 2.0
    }

    /// Di(k) − De(k) by iterating the difference equations and averaging
    /// the payoffs over each type's frequencies.
    pub fn di_minus_de_iterated(&self, k: u32, x: f64) -> f64 {
        let mut s = State::start(x);
        for _ in 1..k {
            s = self.round(s);
        }
        let p = self.payoffs(s);
        let di = (s.x0 * p[0] + s.x1 * p[1]) / (s.x0 + s.x1);
        let de = (s.y0 * p[2] + s.y1 * p[3]) / (s.y0 + s.y1);
        di - de
    }

    /// 2(Di − De) over a random number of rounds (another with probability
    /// w), NS98's closed form; at x = 1 its limit (bq − c)/(1 − w) − 2qb/(2 − w).
    pub fn advantage(&self, w: f64, x: f64) -> f64 {
        let (b, c, q) = (self.b, self.c, self.q);
        if x >= 1.0 {
            return (b * q - c) / (1.0 - w) - 2.0 * q * b / (2.0 - w);
        }
        let qx = 1.0 - q * x;
        (1.0 - q) * (b * q * x - c) / ((1.0 - w) * qx)
            + 2.0 * q * ((b - c) * (1.0 - x) / (qx * (2.0 - w - w * q * x)) - b / (2.0 - w))
    }

    /// 2(Di − De) over exactly `rounds` rounds.
    pub fn advantage_fixed(&self, rounds: u32, x: f64) -> f64 {
        (1..=rounds).map(|k| 2.0 * self.di_minus_de(k, x)).sum()
    }

    /// The threshold x_min in (0, 1) where Di = De (discriminators win
    /// above it), by bisection on `f`; `None` when discriminators lose
    /// even at x = 1.
    fn threshold(f: impl Fn(f64) -> f64) -> Option<f64> {
        if f(1.0) <= 0.0 {
            return None;
        }
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if mid == lo || mid == hi {
                break;
            }
            if f(mid) > 0.0 {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Some(hi)
    }

    /// x_min with random rounds (continuation probability w).
    pub fn x_min(&self, w: f64) -> Option<f64> {
        Self::threshold(|x| self.advantage(w, x))
    }

    /// x_min with exactly `rounds` rounds.
    pub fn x_min_fixed(&self, rounds: u32) -> Option<f64> {
        Self::threshold(|x| self.advantage_fixed(rounds, x))
    }

    /// The fewest mean rounds for discriminators to be stable,
    /// (bq + c)/(bq − c); infinite unless q > c/b.
    pub fn min_rounds(&self) -> f64 {
        let (bq, c) = (self.b * self.q, self.c);
        if bq > c {
            (bq + c) / (bq - c)
        } else {
            f64::INFINITY
        }
    }

    /// Whether discriminators are evolutionarily stable (Di > De at x = 1).
    pub fn stable(&self, w: f64) -> bool {
        self.advantage(w, 1.0) > 0.0
    }

    /// With unconditional cooperators: the discriminator frequency
    /// c(2 − w)/(bwq) below which defectors win.
    pub fn cooperator_equilibrium(&self, w: f64) -> f64 {
        self.c * (2.0 - w) / (self.b * w * self.q)
    }

    /// Dc − De = [−c + bwqx/(2 − w)] / [2(1 − w)].
    pub fn cooperators_over_defectors(&self, w: f64, x: f64) -> f64 {
        (-self.c + self.b * w * self.q * x / (2.0 - w)) / (2.0 * (1.0 - w))
    }

    /// Di − De with discriminators x, defectors y and cooperators z.
    pub fn discriminators_over_defectors(&self, w: f64, x: f64, y: f64, z: f64) -> f64 {
        let (b, c, q) = (self.b, self.c, self.q);
        let qx = 1.0 - q * x;
        (b * q * x - c) * (1.0 - q + q * z) / (2.0 * (1.0 - w) * qx) - b * q * (x + y) / (2.0 - w)
            + q * y * (b - c) / (qx * (2.0 - w - w * q * x))
    }
}

/// A starting distribution for the universal-constant map: a fraction f
/// below 0 and the rest at or above it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Start {
    /// f at −1; the rest at score `rest` (`None`: so high that it never
    /// falls below 0).
    AtMinusOne { rest: Option<u32> },
    /// f spread evenly over −1 … −`below`; the rest at 0.
    Spread { below: u32 },
    /// f spread evenly over −1 … −`below`; the rest evenly over 0 … `above`.
    SpreadBoth { below: u32, above: u32 },
}

/// Score frequencies over `lo …`, plus a mass that never falls below 0.
#[derive(Clone, Debug, PartialEq)]
pub struct Scores {
    pub lo: i64,
    pub x: Vec<f64>,
    pub aloft: f64,
}

impl Scores {
    pub fn new(start: Start, f: f64) -> Self {
        let mut s = Scores {
            lo: 0,
            x: Vec::new(),
            aloft: 0.0,
        };
        match start {
            Start::AtMinusOne { rest } => {
                s.add(-1, f);
                match rest {
                    Some(r) => s.add(i64::from(r), 1.0 - f),
                    None => s.aloft = 1.0 - f,
                }
            }
            Start::Spread { below } => {
                for i in 1..=below {
                    s.add(-i64::from(i), f / f64::from(below));
                }
                s.add(0, 1.0 - f);
            }
            Start::SpreadBoth { below, above } => {
                for i in 1..=below {
                    s.add(-i64::from(i), f / f64::from(below));
                }
                for i in 0..=above {
                    s.add(i64::from(i), (1.0 - f) / f64::from(above + 1));
                }
            }
        }
        s
    }

    fn add(&mut self, score: i64, mass: f64) {
        if self.x.is_empty() {
            self.lo = score;
        }
        while score < self.lo {
            self.x.insert(0, 0.0);
            self.lo -= 1;
        }
        let i = (score - self.lo) as usize;
        if i >= self.x.len() {
            self.x.resize(i + 1, 0.0);
        }
        self.x[i] += mass;
    }

    /// φ: the frequency with score ≥ 0.
    pub fn phi(&self) -> f64 {
        let first = (-self.lo).max(0) as usize;
        self.aloft + self.x.iter().skip(first).sum::<f64>()
    }

    /// One round: xᵢ′ = [xᵢ + xᵢ₋₁φ + xᵢ₊₁(1 − φ)]/2 (everyone k = 0,
    /// unbounded scores). Frequencies below 10⁻⁴⁰ at the ends are dropped.
    pub fn round(&mut self) {
        let phi = self.phi();
        let len = self.x.len();
        let mut next = vec![0.0; len + 2];
        for (i, &v) in self.x.iter().enumerate() {
            next[i + 1] += v / 2.0;
            next[i + 2] += v * phi / 2.0;
            next[i] += v * (1.0 - phi) / 2.0;
        }
        self.lo -= 1;
        let first = next.iter().position(|&v| v > 1e-40).unwrap_or(0);
        let last = next.iter().rposition(|&v| v > 1e-40).unwrap_or(0);
        self.x = next[first..=last].to_vec();
        self.lo += first as i64;
    }
}

/// Where the map goes from a start: all-out cooperation (φ → 1) or
/// defection (after the first round, less than 10⁻¹² of the scores that can
/// still fall at or above 0), within `max_rounds`.
pub fn cooperates(start: Start, f: f64, max_rounds: u32) -> Option<bool> {
    let mut s = Scores::new(start, f);
    for round in 0..max_rounds {
        let phi = s.phi();
        if phi > 1.0 - 1e-12 {
            return Some(true);
        }
        if (round > 0 || s.aloft == 0.0) && phi - s.aloft < 1e-12 {
            return Some(false);
        }
        s.round();
    }
    None
}

/// The largest fraction below 0 from which the map still reaches all-out
/// cooperation (NS98: "0.7380294688360…", start unstated), by bisection;
/// an undecided run counts as not cooperating.
pub fn universal_threshold(start: Start) -> f64 {
    let (mut lo, mut hi) = (0.0, 1.0);
    for _ in 0..45 {
        let mid = (lo + hi) / 2.0;
        if cooperates(start, mid, 1_000_000) == Some(true) {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// LH01's r = (m − 1)/(n + m − 1): the expected rounds as a recipient
/// before next being a donor or the game's end (random rounds, mean m).
pub fn standing_r(n: u32, m: u32) -> f64 {
    f64::from(m - 1) / f64::from(n + m - 1)
}

/// LH01's v = ε/(e + ε): the chance a perceived refusal was misperceived.
pub fn standing_v(e: f64, eps: f64) -> f64 {
    if e + eps > 0.0 {
        eps / (e + eps)
    } else {
        0.0
    }
}

/// LH01's condition for standing to be a strict best reply to itself:
/// vrb < c < rb (with no perception errors, v = 0: rb − c > 0).
pub fn standing_stable(b: f64, c: f64, n: u32, m: u32, e: f64, eps: f64) -> bool {
    let r = standing_r(n, m);
    let v = standing_v(e, eps);
    v * r * b < c && c < r * b
}
```
Create `crates/sugarscape-core/src/image/presets.rs` (descriptions quote Decision M):

```rust
//! NS98's Figs. 1–4 and own-score strategies, LH01's Figs. 1–4, and the
//! run without the payoff offset. Descriptions quote measurements (release,
//! recorded 2026-09-26; seeds and run lengths in each).

use super::config::{ImageConfig, Information, Initial, Offset, Seeded};
use super::strategy::{Class, Strategy};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut ImageConfig),
) -> ModelPreset {
    let mut c = ImageConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Image(c),
    }
}

const NS98: &str = "Nowak & Sigmund, Nature 393 (1998) 573–577";
const LH01: &str = "Leimar & Hammerstein, Proc. R. Soc. B 268 (2001) 745–753";

/// NS98 Fig. 3: observers 10, m = 10n, ν 0.001, a group of n.
fn fig_3(c: &mut ImageConfig, n: u32) {
    c.information = Information::Observers;
    c.group_size = n;
    c.rounds = 10 * n;
    c.mutation = 0.001;
}

/// LH01's island model: 100 groups of 100, m 500, c 0.25, p 0.9.
fn island(c: &mut ImageConfig) {
    c.groups = 100;
    c.rounds = 500;
    c.c = 0.25;
    c.local = 0.9;
}

fn seeded(only: Strategy, invader: Option<Strategy>, share: f64) -> Initial {
    Initial::Seeded(Seeded {
        only,
        invader,
        share,
    })
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ns-fig-1",
            "Fig. 1: cooperation wins",
            NS98,
            "NS98 Fig. 1: 100 players with k from −5 (always help) to +6 (never), drawn uniformly; in each generation 125 random donor–recipient pairs; a donor helps if the recipient's score is at least its k (cost 0.1, benefit 1, 0.1 added to both players every round), helping raises its score by one and refusing lowers it, within ±5; offspring in proportion to payoff. NS98 show one run in which k = 0 is fixed after 166 generations. Measured (seeds 1–100, run to fixation): k = 0 fixes in 20 runs (median generation 56) and some k ≤ 0 in 40 — defection wins the other 60. With m = 300, some k ≤ 0 wins 91.",
            |_| {},
        ),
        preset(
            "ns-fig-2",
            "Fig. 2: cycles under mutation",
            NS98,
            "Fig. 1 with m = 300 and mutation 0.001 (uniform over k): \"endless cycles\", with k = −4 or −5 drifting in and letting defectors back. Measured (seeds 1–10, 10⁵ generations each): 172 collapses of cooperation (k ≤ 0 falling from at least 90% to at most 10%) and 167 recoveries, 1.7 per 10,000 generations; k ≤ −4 averages 8% of cooperative populations but 68% over the 50 generations before a collapse; cooperative strategies 67% of the time. Reproduced.",
            |c| {
                c.rounds = 300;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-fig-3-n20",
            "Fig. 3: observers, n = 20",
            NS98,
            "NS98 Fig. 3: each interaction is seen by the recipient and on average ten others (each other member with probability 10/(n − 2)); each member keeps its own tally of what it has seen, 0 until it sees someone act; m = 10n, mutation 0.001. NS98 (10⁷ generations): cooperative strategies 90%, 47% and 18% at n = 20, 50 and 100. Measured (seeds 1–10, generations 1,001–20,000): 86%, 44% and 20% — reproduced. If one sighting revealed the donor's whole score (records: score) the group-size effect all but vanishes: 97%, 93%, 92%. With FAIR23's fixed visibility of 0.1 (1.8 and 4.8 observers at n = 20 and 50): 28% and 16%.",
            |c| fig_3(c, 20),
        ),
        preset(
            "ns-fig-3-n50",
            "Fig. 3: observers, n = 50",
            NS98,
            "NS98 Fig. 3: each interaction is seen by the recipient and on average ten others (each other member with probability 10/(n − 2)); each member keeps its own tally of what it has seen, 0 until it sees someone act; m = 10n, mutation 0.001. NS98 (10⁷ generations): cooperative strategies 90%, 47% and 18% at n = 20, 50 and 100. Measured (seeds 1–10, generations 1,001–20,000): 86%, 44% and 20% — reproduced. If one sighting revealed the donor's whole score (records: score) the group-size effect all but vanishes: 97%, 93%, 92%. With FAIR23's fixed visibility of 0.1 (1.8 and 4.8 observers at n = 20 and 50): 28% and 16%.",
            |c| fig_3(c, 50),
        ),
        preset(
            "ns-fig-3-n100",
            "Fig. 3: observers, n = 100",
            NS98,
            "NS98 Fig. 3: each interaction is seen by the recipient and on average ten others (each other member with probability 10/(n − 2)); each member keeps its own tally of what it has seen, 0 until it sees someone act; m = 10n, mutation 0.001. NS98 (10⁷ generations): cooperative strategies 90%, 47% and 18% at n = 20, 50 and 100. Measured (seeds 1–10, generations 1,001–20,000): 86%, 44% and 20% — reproduced. If one sighting revealed the donor's whole score (records: score) the group-size effect all but vanishes: 97%, 93%, 92%. With FAIR23's fixed visibility of 0.1 (1.8 and 4.8 observers at n = 20 and 50): 28% and 16%.",
            |c| fig_3(c, 100),
        ),
        preset(
            "ns-fig-4a",
            "Fig. 4a: AND, perfect information",
            NS98,
            "NS98 Fig. 4a: AND strategies (help if the recipient's score is at least k and one's own is below h; k and h −5 … +6), perfect information, m = 500, mutation 0.001. NS98: 55% of interactions cooperative, (k 0, h 1) the most frequent strategy. Measured (seeds 1–10, generations 1,001–50,000): 53%, (0, 1) the most frequent (22%). Reproduced.",
            |c| {
                c.strategies = vec![Class::And];
                c.rounds = 500;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-fig-4b",
            "Fig. 4b: AND, observers",
            NS98,
            "NS98 Fig. 4b: AND strategies with observers, \"as in figure 3 with n = 20\" (m = 10n = 200). NS98: 57%, (k 0, h 4) the most frequent. Measured (seeds 1–10, generations 1,001–50,000): 53%, (k 0, h 5) the most frequent (11%) — a high own threshold, as NS98 argue.",
            |c| {
                fig_3(c, 20);
                c.strategies = vec![Class::And];
            },
        ),
        preset(
            "ns-fig-4c",
            "Fig. 4c: OR, perfect information",
            NS98,
            "NS98 Fig. 4c: OR strategies (help if the recipient's score is at least k or one's own is below h), perfect information, m = 500. NS98: 70%, with the defectors (k 6, h −5) the most frequent single strategy. Measured (seeds 1–10, generations 1,001–50,000): 78%; the most frequent is (k 3, h 4) (10%), the defectors second (6%).",
            |c| {
                c.strategies = vec![Class::Or];
                c.rounds = 500;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-fig-4d",
            "Fig. 4d: OR, observers",
            NS98,
            "NS98 Fig. 4d: OR strategies with observers (n = 20, m = 200). NS98: 80%, the defectors most frequent. Measured (seeds 1–10, generations 1,001–50,000): 85%; the most frequent is (k 2, h 5) (4.5%), the defectors (k 6, h −5) third (3.1%).",
            |c| {
                fig_3(c, 20);
                c.strategies = vec![Class::Or];
            },
        ),
        preset(
            "ns-own-only",
            "Own score only",
            NS98,
            "Strategies that consider only their own score (help while it is below h, h −5 … +6), m = 500, mutation 0.001. NS98: \"less than 0.1% cooperation\". Measured (seeds 1–10, generations 1,001–20,000): 0.19% — the mutants (5 of 12 help at a generation's start) keep it above 0.1%.",
            |c| {
                c.strategies = vec![Class::OwnOnly];
                c.rounds = 500;
                c.mutation = 0.001;
            },
        ),
        preset(
            "ns-no-offset",
            "Fig. 1 without the offset",
            NS98,
            "Fig. 1 with nothing added to payoffs: a helping donor ends the round 0.1 down, and negative payoffs weigh nothing in reproduction. (The FAIR23 NetLogo model, said to omit the offset, in fact adds c to both players every round, as LH01 say NS98 did.) Measured (seeds 1–100): k = 0 fixes in 10 runs (median generation 93) and some k ≤ 0 in 63, against 40 with the offset; with Fig. 2's settings, 78% cooperative against 67%. The offset weakens selection, and cooperation loses by it.",
            |c| c.offset = Offset::None,
        ),
        preset(
            "lh-fig-1a",
            "Fig. 1a: h = 1 invades k = 0",
            LH01,
            "LH01 Fig. 1a: 100 groups of 100 (a parent from the offspring's own group with probability 0.9), m = 500, c = 0.25, no mutation or errors; everyone plays k = 0 except 1% of each group playing h = 1 (help while one's own score is below 1, whatever the recipient's). LH01: h = 1 invades (about 80% by generation 150). Measured (seeds 1–10): 37% at generation 50, fixed in every run by 150. Reproduced, faster.",
            |c| {
                island(c);
                c.strategies = vec![Class::K, Class::H];
                c.initial = seeded(Strategy::K(0), Some(Strategy::H(1)), 0.01);
            },
        ),
        preset(
            "lh-fig-1b",
            "Fig. 1b: h = 1 invades (k 0, h 1) with errors",
            LH01,
            "LH01 Fig. 1b: the same with (k 0, h 1) and execution errors 0.05: h = 1 invades but does not wipe out (k 0, h 1). Measured (seeds 1–10): 1.8% at generation 150, 13% at 500, 42% at 1,000 — it invades, an order of magnitude more slowly than LH01 show.",
            |c| {
                island(c);
                c.strategies = vec![Class::And, Class::H];
                c.initial = seeded(Strategy::And { k: 0, h: 1 }, Some(Strategy::H(1)), 0.01);
                c.execution_error = 0.05;
            },
        ),
        preset(
            "lh-fig-2a",
            "Fig. 2a: one group",
            LH01,
            "LH01 Fig. 2a: AND strategies in one group of 100 (NS98's setting), m = 500, c = 0.25, mutation 0.001, no errors. LH01: help in 39% of rounds over 10⁶ generations, (k 0, h 1) dominant. Measured (seeds 1–10, generations 1,001–100,000): 37%, (0, 1) the most frequent (38%). Reproduced.",
            |c| {
                c.strategies = vec![Class::And];
                c.rounds = 500;
                c.c = 0.25;
                c.mutation = 0.001;
            },
        ),
        preset(
            "lh-fig-2b",
            "Fig. 2b: island model, p = 0.9",
            LH01,
            "LH01 Fig. 2b: the island model (100 groups, p = 0.9) with execution errors 0.02. LH01: with drift limited, image scoring fades — help in 9% of rounds over 10⁵ generations. Measured (seeds 1–10, generations 1,001–5,000): 44% (to 20,000: 34%, runs from 2% to 54%; 10,001–50,000: 31%). Not reproduced: cooperative AND strategies persist in most runs.",
            |c| {
                island(c);
                c.strategies = vec![Class::And];
                c.mutation = 0.001;
                c.execution_error = 0.02;
            },
        ),
        preset(
            "lh-fig-2c",
            "Fig. 2c: island model, p = 0.5",
            LH01,
            "LH01 Fig. 2c: stronger gene flow (p = 0.5). LH01: 2% help, \"mainly a consequence of execution errors\". Measured (seeds 1–10, generations 1,001–5,000): 15% (to 20,000: 9%). Not reproduced.",
            |c| {
                island(c);
                c.strategies = vec![Class::And];
                c.mutation = 0.001;
                c.execution_error = 0.02;
                c.local = 0.5;
            },
        ),
        preset(
            "lh-fig-3a",
            "Fig. 3a: a small cost",
            LH01,
            "LH01 Fig. 3a: a small cost (c = 0.1), initial payoff 5, p = 0.5, execution errors 0.02. LH01: help in 45% of rounds over 2 × 10⁵ generations. Measured (seeds 1–10, generations 1,001–3,000): 52% (to 10,000: 47%).",
            |c| {
                island(c);
                c.strategies = vec![Class::And];
                c.c = 0.1;
                c.u0 = 5.0;
                c.local = 0.5;
                c.execution_error = 0.02;
                c.mutation = 0.001;
            },
        ),
        preset(
            "lh-fig-3b",
            "Fig. 3b: with q strategies",
            LH01,
            "LH01 Fig. 3b: Fig. 3a with LH01's q strategies (help when helping is estimated to raise the chance of being helped by more than Δq, 0.01 … 0.99, from group tallies of who was helped). LH01: help 15%, q strategies 12% of the population. Measured (seeds 1–10, generations 1,001–3,000): help 17%, q strategies 26% (to 10,000: 15% and 18%).",
            |c| {
                island(c);
                c.strategies = vec![Class::And, Class::Q];
                c.c = 0.1;
                c.u0 = 5.0;
                c.local = 0.5;
                c.execution_error = 0.02;
                c.mutation = 0.001;
            },
        ),
        preset(
            "lh-fig-4a",
            "Fig. 4a: standing invades discriminators",
            LH01,
            "LH01 Fig. 4a: binary scorers (scores 0 and −1): everyone a discriminator (k 0) but 1% of each group playing standing (help when in bad standing or when the recipient is in good standing; standing is lost only by refusing a recipient in good standing), execution errors 0.05, no mutation. LH01: standing takes over. Measured (seeds 1–10): standing 69% at generation 500, 98% at 1,000. Reproduced.",
            |c| {
                island(c);
                c.strategies = vec![Class::Binary, Class::Standing];
                c.initial = seeded(Strategy::Binary(0), Some(Strategy::Standing), 0.01);
                c.execution_error = 0.05;
            },
        ),
        preset(
            "lh-fig-4b",
            "Fig. 4b: … with perception errors",
            LH01,
            "LH01 Fig. 4b: Fig. 4a with execution and perception errors 0.025 (each member keeps its own view of the others' scores and standing). LH01: standing still invades. Measured (seeds 1–5): 35% at generation 500, 75% at 1,000. Reproduced.",
            |c| {
                island(c);
                c.strategies = vec![Class::Binary, Class::Standing];
                c.initial = seeded(Strategy::Binary(0), Some(Strategy::Standing), 0.01);
                c.execution_error = 0.025;
                c.perception_error = 0.025;
            },
        ),
        preset(
            "lh-fig-4c",
            "Fig. 4c: standing in the long run",
            LH01,
            "LH01 Fig. 4c: cooperators, discriminators, defectors and standing from a uniform start, errors 0.025, mutation 0.0001. LH01 (10⁵ generations): standing dominates, cooperators stay appreciable, discriminators occasionally reach 5%, defectors below 1% — though their condition vrb < c < rb fails here (v = 0.5, r = 0.833). Measured (seeds 1–3, generations 1,001–1,500): standing 53%, cooperators 39%, discriminators 8%, defectors 0.02% (seeds 1–10 to 3,000: 77%, 20%, 3%, 0.01%).",
            |c| {
                island(c);
                c.strategies = vec![Class::Binary, Class::Standing];
                c.execution_error = 0.025;
                c.perception_error = 0.025;
                c.mutation = 0.0001;
            },
        ),
    ]
}
```
- [ ] **Step 4: Wire the kind**

`crates/sugarscape-core/src/model.rs` (every `Dpd` arm gains an `Image` twin; the round-trip test and the kind list):

```diff
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -12,6 +12,7 @@
 use crate::culture::{CultureConfig, CultureWorld};
 use crate::dpd::{DpdConfig, DpdWorld};
 use crate::ethno::{EthnoConfig, EthnoWorld};
+use crate::image::{ImageConfig, ImageWorld};
 use crate::opinions::{OpinionsConfig, OpinionsWorld};
 use crate::render::{self, ColorMode, Layer};
 use crate::ring::{RingConfig, RingWorld};
@@ -22,8 +23,8 @@
 use crate::tags::{TagsConfig, TagsWorld};
 use crate::world::World;
 use crate::{
-    anasazi, civil, classes, culture, dpd, ethno, export, opinions, ring, schelling, spatial,
-    stats, structure, tags,
+    anasazi, civil, classes, culture, dpd, ethno, export, image, opinions, ring, schelling,
+    spatial, stats, structure, tags,
 };
 
 /// Which model a config or world is.
@@ -43,10 +44,11 @@
     Opinions,
     Structure,
     Dpd,
+    Image,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 13] = [
+    pub const ALL: [ModelKind; 14] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -60,6 +62,7 @@
         ModelKind::Opinions,
         ModelKind::Structure,
         ModelKind::Dpd,
+        ModelKind::Image,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -77,6 +80,7 @@
             ModelKind::Opinions => "opinions",
             ModelKind::Structure => "structure",
             ModelKind::Dpd => "dpd",
+            ModelKind::Image => "image",
         }
     }
 
@@ -97,6 +101,7 @@
             ModelKind::Opinions => opinions::schema(),
             ModelKind::Structure => structure::schema(),
             ModelKind::Dpd => dpd::schema(),
+            ModelKind::Image => image::schema(),
         }
     }
 }
@@ -123,6 +128,7 @@
     Opinions(OpinionsConfig),
     Structure(StructureConfig),
     Dpd(DpdConfig),
+    Image(ImageConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -141,6 +147,7 @@
     Opinions(&'a OpinionsConfig),
     Structure(&'a StructureConfig),
     Dpd(&'a DpdConfig),
+    Image(&'a ImageConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -166,6 +173,7 @@
             ModelConfig::Opinions(c) => Tagged::Opinions(c).serialize(s),
             ModelConfig::Structure(c) => Tagged::Structure(c).serialize(s),
             ModelConfig::Dpd(c) => Tagged::Dpd(c).serialize(s),
+            ModelConfig::Image(c) => Tagged::Image(c).serialize(s),
         }
     }
 }
@@ -186,6 +194,7 @@
             ModelConfig::Opinions(_) => ModelKind::Opinions,
             ModelConfig::Structure(_) => ModelKind::Structure,
             ModelConfig::Dpd(_) => ModelKind::Dpd,
+            ModelConfig::Image(_) => ModelKind::Image,
         }
     }
 
@@ -259,10 +268,13 @@
             "dpd" => serde_json::from_value(value)
                 .map(ModelConfig::Dpd)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "image" => serde_json::from_value(value)
+                .map(ModelConfig::Image)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure or dpd)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno, opinions, structure, dpd or image)"
                 ),
             )),
         }
@@ -283,6 +295,7 @@
             ModelConfig::Opinions(c) => c.validate(),
             ModelConfig::Structure(c) => c.validate(),
             ModelConfig::Dpd(c) => c.validate(),
+            ModelConfig::Image(c) => c.validate(),
         }
     }
 
@@ -303,6 +316,7 @@
             ModelConfig::Opinions(c) => set_path(c, path, value).map(ModelConfig::Opinions),
             ModelConfig::Structure(c) => set_path(c, path, value).map(ModelConfig::Structure),
             ModelConfig::Dpd(c) => set_path(c, path, value).map(ModelConfig::Dpd),
+            ModelConfig::Image(c) => set_path(c, path, value).map(ModelConfig::Image),
         }
     }
 
@@ -314,6 +328,7 @@
             ModelConfig::Tags(c) => (c.end > 0).then_some(c.end),
             ModelConfig::Ethno(c) => (c.end > 0).then_some(c.end),
             ModelConfig::Dpd(c) => (c.end > 0).then_some(c.end),
+            ModelConfig::Image(c) => (c.end > 0).then_some(c.end),
             ModelConfig::Sugarscape(_)
             | ModelConfig::Schelling(_)
             | ModelConfig::Ring(_)
@@ -342,6 +357,7 @@
             ModelConfig::Opinions(_) => opinions::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Structure(_) => structure::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Dpd(_) => dpd::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Image(_) => image::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -524,6 +540,7 @@
     Opinions(Box<OpinionsWorld>),
     Structure(Box<StructureWorld>),
     Dpd(Box<DpdWorld>),
+    Image(Box<ImageWorld>),
 }
 
 impl ModelWorld {
@@ -560,6 +577,7 @@
                 ModelWorld::Structure(Box::new(StructureWorld::new(c, seed)?))
             }
             ModelConfig::Dpd(c) => ModelWorld::Dpd(Box::new(DpdWorld::new(c, seed)?)),
+            ModelConfig::Image(c) => ModelWorld::Image(Box::new(ImageWorld::new(c, seed)?)),
         })
     }
 
@@ -578,6 +596,7 @@
             ModelWorld::Opinions(_) => ModelKind::Opinions,
             ModelWorld::Structure(_) => ModelKind::Structure,
             ModelWorld::Dpd(_) => ModelKind::Dpd,
+            ModelWorld::Image(_) => ModelKind::Image,
         }
     }
 
@@ -596,6 +615,7 @@
             ModelWorld::Opinions(w) => w.as_ref(),
             ModelWorld::Structure(w) => w.as_ref(),
             ModelWorld::Dpd(w) => w.as_ref(),
+            ModelWorld::Image(w) => w.as_ref(),
         }
     }
 
@@ -614,6 +634,7 @@
             ModelWorld::Opinions(w) => w.as_mut(),
             ModelWorld::Structure(w) => w.as_mut(),
             ModelWorld::Dpd(w) => w.as_mut(),
+            ModelWorld::Image(w) => w.as_mut(),
         }
     }
 
@@ -700,6 +721,7 @@
             ModelWorld::Opinions(w) => copy_without_history!(Opinions, w),
             ModelWorld::Structure(w) => copy_without_history!(Structure, w),
             ModelWorld::Dpd(w) => copy_without_history!(Dpd, w),
+            ModelWorld::Image(w) => copy_without_history!(Image, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -728,6 +750,7 @@
             (ModelWorld::Opinions(live), ModelWorld::Opinions(kept)) => restore_into!(live, kept),
             (ModelWorld::Structure(live), ModelWorld::Structure(kept)) => restore_into!(live, kept),
             (ModelWorld::Dpd(live), ModelWorld::Dpd(kept)) => restore_into!(live, kept),
+            (ModelWorld::Image(live), ModelWorld::Image(kept)) => restore_into!(live, kept),
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -1007,6 +1030,32 @@
     }
 
     #[test]
+    fn image_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "image", "rounds": 300, "strategies": ["and"], "offset": "none"}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Image);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(json["model"], "image");
+        assert_eq!(json["offset"], "none");
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["help_rate", "mean_k"]);
+        assert_eq!(c.max_ticks(), None);
+        let next = c.with_path("end", &json!(500)).unwrap();
+        assert_eq!(next.max_ticks(), Some(500));
+        let e = ModelConfig::from_json(r#"{"model": "image", "mutation": 2}"#).unwrap_err();
+        assert_eq!(e[0].field, "mutation");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!((w.kind(), w.model().size()), (ModelKind::Image, (10, 10)));
+        assert_eq!(w.model().population(), 100);
+        let cp = w.checkpoint().expect("image worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
+    #[test]
     fn only_the_anasazi_finishes() {
         let mut w = ModelWorld::new(
             ModelConfig::Anasazi(crate::anasazi::AnasaziConfig {
@@ -1042,7 +1091,8 @@
                 "ethno",
                 "opinions",
                 "structure",
-                "dpd"
+                "dpd",
+                "image"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
```
`crates/sugarscape-core/src/presets.rs`:

```diff
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -673,7 +673,8 @@
 
 /// Every model's presets: the sugarscape's (`all`), then Schelling's, Ring
 /// World's, the anasazi's, civil violence's, the tags model's, the spatial
-/// games', the ethnocentrism model's and the demographic PD's.
+/// games', the ethnocentrism model's, the demographic PD's and image
+/// scoring's.
 pub fn catalog() -> Vec<ModelPreset> {
     let mut out: Vec<ModelPreset> = all().into_iter().map(ModelPreset::from).collect();
     out.extend(crate::schelling::presets());
@@ -688,6 +689,7 @@
     out.extend(crate::spatial::presets());
     out.extend(crate::ethno::presets());
     out.extend(crate::dpd::presets());
+    out.extend(crate::image::presets());
     out
 }
```
`crates/sugarscape-core/src/sweep.rs` (Task 1 takes only the first hunk, the ticks message; the BUILTINS and id-list hunks are Task 2's):

```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -402,6 +402,9 @@
                 crate::model::ModelKind::Dpd => {
                     format!("the demographic PD stops at its last cycle, {max} in this config")
                 }
+                crate::model::ModelKind::Image => {
+                    format!("image scoring stops at its last generation, {max} in this config")
+                }
                 _ => format!(
                     "the Long House Valley stops at its end year, \
                      {max} ticks after its start year in this config"
```
`crates/sugarscape-cli/src/main.rs`:

```diff
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -210,7 +210,7 @@
         // the tags model at its last generation; Axelrod's culture and bounded
         // confidence once stable, a sugarscape under his rule once its cultures
         // settle; ethnocentrism at its last period; the demographic PD at its
-        // last cycle.
+        // last cycle; image scoring at its last generation.
         let why = match config.kind() {
             ModelKind::Civil => "a group has died out",
             ModelKind::Tags => "its last generation",
@@ -221,6 +221,7 @@
             ModelKind::Sugarscape => "the cultures have settled",
             ModelKind::Ethno => "its last period",
             ModelKind::Dpd => "its last cycle",
+            ModelKind::Image => "its last generation",
             _ => "its end year",
         };
         eprintln!("finished at tick {} ({why})", world.tick());
```
- [ ] **Step 4b: The Review Focus tests**

Append inside `crates/sugarscape-core/src/image/world.rs`'s `mod tests`, before its closing brace (they use the module's `world` helper):

```rust
    #[test]
    fn a_group_of_two_with_observers_has_only_the_recipient_watching() {
        let mut w = world(1, 2, Strategy::K(0), |c| {
            c.information = Information::Observers;
            c.observers = 10.0;
        });
        assert_eq!(w.config.watch_probability(), 0.0);
        w.run(5);
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn more_observers_than_bystanders_means_everyone_watches() {
        let w = world(1, 5, Strategy::K(0), |c| {
            c.information = Information::Observers;
            c.observers = 10.0;
        });
        assert_eq!(w.config.watch_probability(), 1.0);
    }

    #[test]
    fn unbounded_scores_run_long_and_still_draw() {
        let mut w = world(2, 10, Strategy::K(-5), |c| {
            c.clamp = 0;
            c.rounds = 2000;
        });
        w.run(2);
        let mut buf = Vec::new();
        for mode in ["strategy", "score", "payoff"] {
            w.render(mode, "", &mut buf).unwrap();
            assert!(!buf.is_empty(), "{mode}");
        }
        assert_eq!(w.population(), 20);
    }

    #[test]
    fn a_single_round_generation_with_all_payoffs_zero_still_reproduces() {
        // Defectors with no offset: nobody earns, the pool is empty, and the
        // next generation is drawn uniformly (Decision on empty pools).
        let mut w = world(3, 4, Strategy::K(6), |c| {
            c.rounds = 1;
            c.offset = Offset::None;
            c.mutation = 0.5;
        });
        w.run(10);
        assert_eq!(w.population(), 12);
    }
```

- [ ] **Step 5: Run the tests to see them pass**

Run: `cargo test -p sugarscape-core --lib image`
Expected: `test result: ok. 45 passed; 0 failed`

Run: `cargo test -p sugarscape-core --lib model::`
Expected: `test result: ok. 20 passed` (including `image_configs_round_trip_with_their_tag` and `every_kind_names_itself_and_only_other_models_have_schemas`)

- [ ] **Step 6: Golden entries and keyframes**

`crates/sugarscape-core/tests/golden.rs` (a separate table: the island presets run 20 generations, not 200 — Decision 17):

```diff
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -1,7 +1,7 @@
 //! Earlier runs are unchanged: with disease off, the milestone-1 and
 //! Chapter IV presets evolve exactly as they did before Chapter V was added.
 
-use sugarscape_core::model::ModelWorld;
+use sugarscape_core::model::{ModelConfig, ModelKind, ModelWorld};
 use sugarscape_core::presets;
 use sugarscape_core::world::World;
 
@@ -201,7 +201,54 @@
     world.model().fingerprint()
 }
 
+/// Image scoring's presets: (id, generations, fingerprint from seed 1). The
+/// island presets (100 groups of 100, 50,000 rounds a generation, and with
+/// perception errors a record per pair of members) run 20 generations, the
+/// one-group presets 200.
+const IMAGE_GOLDEN: &[(&str, u32, u64)] = &[
+    // Milestone 21: image scoring.
+    ("ns-fig-1", 200, 0x98875bd71738cf05),
+    ("ns-fig-2", 200, 0x91995405bae5fd9),
+    ("ns-fig-3-n20", 200, 0xb4bd11bc229673c4),
+    ("ns-fig-3-n50", 200, 0x5907ce5cb47602cf),
+    ("ns-fig-3-n100", 200, 0xaf771beff4611d0d),
+    ("ns-fig-4a", 200, 0xe7d6cefc18bd8d6a),
+    ("ns-fig-4b", 200, 0xa4c19c5fa5fcfe3),
+    ("ns-fig-4c", 200, 0x64755eafb526a816),
+    ("ns-fig-4d", 200, 0x72ca0b1d80947f44),
+    ("ns-own-only", 200, 0x20d6768b1fbd7081),
+    ("ns-no-offset", 200, 0xc595349de0c0bf65),
+    ("lh-fig-1a", 20, 0x46669de796924497),
+    ("lh-fig-1b", 20, 0x2c3d0dea6af0c894),
+    ("lh-fig-2a", 200, 0x56250416634c6ed9),
+    ("lh-fig-2b", 20, 0x566a697764e9cdd9),
+    ("lh-fig-2c", 20, 0x963ec9f09d2fadbe),
+    ("lh-fig-3a", 20, 0x46059fbe6bab2056),
+    ("lh-fig-3b", 20, 0x4d6575c59b0da89a),
+    ("lh-fig-4a", 20, 0xd5654fa4b600eb57),
+    ("lh-fig-4b", 20, 0x4bfe8ea9f3e8598b),
+    ("lh-fig-4c", 20, 0x6371653b544f6387),
+];
+
+fn image_fingerprint(id: &str, ticks: u32) -> u64 {
+    let preset = presets::find(id).unwrap_or_else(|| panic!("unknown preset {id}"));
+    let mut world = ModelWorld::new(preset.config, 1).unwrap();
+    world.model_mut().run(ticks);
+    world.model().fingerprint()
+}
+
 #[test]
+fn image_presets_are_unchanged() {
+    for &(id, ticks, expected) in IMAGE_GOLDEN {
+        assert_eq!(
+            image_fingerprint(id, ticks),
+            expected,
+            "preset {id} changed"
+        );
+    }
+}
+
+#[test]
 fn other_models_are_unchanged() {
     for &(id, expected) in MODEL_GOLDEN {
         assert_eq!(model_fingerprint(id), expected, "preset {id} changed");
@@ -212,14 +259,15 @@
 fn every_model_preset_has_a_golden_entry() {
     for p in presets::catalog() {
         assert!(
-            GOLDEN.iter().chain(MODEL_GOLDEN).any(|&(id, _)| id == p.id),
+            GOLDEN.iter().chain(MODEL_GOLDEN).any(|&(id, _)| id == p.id)
+                || IMAGE_GOLDEN.iter().any(|&(id, _, _)| id == p.id),
             "record a golden fingerprint for {} (run print_golden)",
             p.id
         );
     }
 }
 
-/// Prints `GOLDEN` entries, then `MODEL_GOLDEN`'s:
+/// Prints `GOLDEN` entries, then `MODEL_GOLDEN`'s, then `IMAGE_GOLDEN`'s:
 /// `cargo test -p sugarscape-core --test golden -- --ignored --nocapture`.
 #[test]
 #[ignore]
@@ -228,10 +276,23 @@
         println!("    (\"{}\", {:#x}),", p.id, fingerprint(p.id));
     }
     println!("MODEL_GOLDEN:");
+    let image = |p: &presets::ModelPreset| p.config.kind() == ModelKind::Image;
     for p in presets::catalog()
         .iter()
-        .filter(|p| p.config.sugarscape().is_none())
+        .filter(|p| p.config.sugarscape().is_none() && !image(p))
     {
         println!("    (\"{}\", {:#x}),", p.id, model_fingerprint(p.id));
+    }
+    println!("IMAGE_GOLDEN:");
+    for p in presets::catalog().iter().filter(|p| image(p)) {
+        let ModelConfig::Image(c) = &p.config else {
+            unreachable!()
+        };
+        let ticks = if c.groups > 1 { 20 } else { 200 };
+        println!(
+            "    (\"{}\", {ticks}, {:#x}),",
+            p.id,
+            image_fingerprint(p.id, ticks)
+        );
     }
 }
```
`crates/sugarscape-core/tests/checkpoint.rs`:

```diff
--- a/crates/sugarscape-core/tests/checkpoint.rs
+++ b/crates/sugarscape-core/tests/checkpoint.rs
@@ -5,6 +5,7 @@
 use sugarscape_core::presets;
 
 /// One preset per model kind, and a second for the spatial games' asynchronous updating.
+/// Image scoring's is AND strategies with observers (private records, mutation).
 const IDS: &[&str] = &[
     "vi-1-everything",
     "vi-4-schelling-25",
@@ -15,6 +16,7 @@
     "hg-async-kaleidoscope",
     "jansson-kin",
     "dpd-rr-best",
+    "ns-fig-4b",
 ];
 
 fn world(id: &str) -> ModelWorld {
```
Run: `cargo test -p sugarscape-core --test golden -- --ignored --nocapture print_golden` and compare the `IMAGE_GOLDEN:` block with the table above (Decision M lists the same 21 values). If any differs, stop and report.
Run: `cargo test -p sugarscape-core --test golden --test checkpoint`
Expected: `test result: ok. 5 passed; 0 failed; 1 ignored` (golden, about 60 s in debug) and `test result: ok. 4 passed` (checkpoint).

- [ ] **Step 7: Format and lint**

Run: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo +1.98.1 clippy --all-targets -- -D warnings`
Expected: no output from fmt; both clippies `Finished` with no warnings.
Run: `cargo test --workspace`
Expected: every suite `ok` (core lib 706 passed; golden 5 passed).

- [ ] **Step 8: Commit**

```bash
git add crates/sugarscape-core/src/image/mod.rs crates/sugarscape-core/src/image/strategy.rs crates/sugarscape-core/src/image/config.rs crates/sugarscape-core/src/image/stats.rs crates/sugarscape-core/src/image/world.rs crates/sugarscape-core/src/image/analytic.rs crates/sugarscape-core/src/image/presets.rs crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/src/main.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/checkpoint.rs
git commit -m "Add image scoring as a model kind" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 2: Measured against the sources — book-style tests, four sweeps, WASM

**Files:**
- Create: `crates/sugarscape-core/tests/image.rs` (18 ignored tests), `sweeps/{ns-rounds,ns-group-size,lh-cost,lh-gene-flow}.json`
- Modify: `crates/sugarscape-core/src/sweep.rs` (BUILTINS and the id list), `crates/sugarscape-cli/tests/cli.rs` (ids), `crates/sugarscape-wasm/tests/web.rs` (ids and a fingerprint test)

**Interfaces:**
- Consumes: Task 1's `image::*` and `image::analytic::*`; `presets::find`; `model::{Model, ModelConfig}`; the CLI's `sweep --builtin`.
- Produces: `sweep::builtin("ns-rounds" | "ns-group-size" | "lh-cost" | "lh-gene-flow")`; the ignored tests pinning Decision M.

- [ ] **Step 1: The book-style tests**

Create `crates/sugarscape-core/tests/image.rs` (each pin is Decision M's number; they pass on Task 1's model as written — a failure means the model differs from the dry run: stop and report, do not retune):

```rust
//! Image scoring against its sources: NS98's Figs. 1–4 and Methods, and
//! LH01's Figs. 1–4 and standing condition. Ignored by default (release:
//! `cargo test -p sugarscape-core --release --test image -- --ignored
//! --nocapture`). NS98 averaged over 10⁷ generations and LH01 over 10⁵–10⁶;
//! these runs are shorter, averaged over seeds, as each test says. Every
//! world is a function of (config, seed), so the numbers pinned here are the
//! measured ones (recorded 2026-09-26), each with the source's beside it;
//! claims that fail are pinned as measured.

use std::sync::Mutex;

use sugarscape_core::image::analytic::{self, Binary, Start};
use sugarscape_core::image::{
    Class, ImageConfig, ImageWorld, Initial, Offset, Records, Seeded, Strategy,
};
use sugarscape_core::model::{Model, ModelConfig};
use sugarscape_core::presets;

fn config(id: &str, edit: impl FnOnce(&mut ImageConfig)) -> ImageConfig {
    let ModelConfig::Image(mut c) = presets::find(id).unwrap().config else {
        panic!("{id} is not an image preset")
    };
    edit(&mut c);
    c
}

/// `f(seed)` for each seed, on up to ten threads, in seed order.
fn par<T: Send>(seeds: impl IntoIterator<Item = u64>, f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let next = Mutex::new(seeds.into_iter().collect::<Vec<_>>().into_iter());
    let out = Mutex::new(Vec::new());
    std::thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| loop {
                let Some(seed) = next.lock().unwrap().next() else {
                    break;
                };
                let v = f(seed);
                out.lock().unwrap().push((seed, v));
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort_by_key(|p| p.0);
    v.into_iter().map(|p| p.1).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The mean of `series` over generations `from..=ticks`, one per seed.
fn window(c: &ImageConfig, series: &str, seeds: u64, ticks: u32, from: usize) -> Vec<f64> {
    windows(c, &[series], seeds, ticks, from)
        .into_iter()
        .map(|v| v[0])
        .collect()
}

/// The means of several series over generations `from..=ticks`, per seed.
fn windows(c: &ImageConfig, series: &[&str], seeds: u64, ticks: u32, from: usize) -> Vec<Vec<f64>> {
    par(1..=seeds, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        w.run(ticks);
        series
            .iter()
            .map(|s| mean(&w.series(s).unwrap()[from..]))
            .collect()
    })
}

/// Asserts a measured value (printing it beside the source's).
fn pin(what: &str, got: f64, want: f64, tol: f64, source: &str) {
    println!("{what}: {got} (pinned {want}; source: {source})");
    assert!((got - want).abs() <= tol, "{what}: {got} vs pinned {want}");
}

fn share(w: &ImageWorld, f: impl Fn(Strategy) -> bool) -> f64 {
    let a = w.agents();
    a.iter().filter(|x| f(x.strategy)).count() as f64 / a.len() as f64
}

/// The strategy every agent plays, if they all play one.
fn fixed(w: &ImageWorld) -> Option<Strategy> {
    let a = w.agents();
    a.iter()
        .all(|x| x.strategy == a[0].strategy)
        .then(|| a[0].strategy)
}

/// Seeds 1–100 of `c` to fixation (at most 5,000 generations): how many fix
/// k = 0, how many any k ≤ 0, and the median generation k = 0 fixed at.
fn fixation(c: &ImageConfig) -> (usize, usize, u64) {
    let r = par(1..=100, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        for _ in 0..5000 {
            w.step();
            if let Some(s) = fixed(&w) {
                return Some((s, w.tick));
            }
        }
        None
    });
    let mut t0: Vec<u64> = r
        .iter()
        .flatten()
        .filter(|(s, _)| *s == Strategy::K(0))
        .map(|p| p.1)
        .collect();
    t0.sort();
    let coop = r.iter().flatten().filter(|(s, _)| s.cooperative()).count();
    (t0.len(), coop, t0.get(t0.len() / 2).copied().unwrap_or(0))
}

#[test]
#[ignore]
fn ns98_fig_1_k_0_fixes_but_only_in_a_fifth_of_runs() {
    // NS98 Fig. 1: one run in which k = 0 is fixed after t = 166.
    let (k0, coop, median) = fixation(&config("ns-fig-1", |_| {}));
    println!("k = 0 fixed in {k0}/100 (median generation {median}); some k ≤ 0 in {coop}/100");
    assert_eq!((k0, coop, median), (20, 40, 56));
    // More rounds, more cooperation ("more likely to win the greater the number m").
    let (k0, coop, _) = fixation(&config("ns-fig-1", |c| c.rounds = 300));
    println!("m = 300: k = 0 in {k0}/100, some k ≤ 0 in {coop}/100");
    assert_eq!((k0, coop), (14, 91));
}

#[test]
#[ignore]
fn ns98_fig_1_without_the_offset() {
    // Ours: the offset weakens selection; without it cooperation wins more often.
    let (k0, coop, median) = fixation(&config("ns-no-offset", |_| {}));
    println!("no offset: k = 0 in {k0}/100 (median {median}), some k ≤ 0 in {coop}/100");
    assert_eq!((k0, coop, median), (10, 63, 93));
}

/// Collapses of cooperation in `c` over `ticks` generations, seeds 1–10: a
/// collapse is the share of k ≤ 0 falling from at least 0.9 to at most 0.1,
/// a recovery the reverse. Also the mean share of k ≤ −4 in cooperative
/// generations (≥ 0.9) and over the 51 generations up to each collapse's
/// last cooperative one.
fn cycles(c: &ImageConfig, ticks: u64) -> (usize, usize, f64, f64) {
    let r = par(1..=10, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        let (mut coop, mut unc) = (Vec::new(), Vec::new());
        for _ in 0..ticks {
            w.step();
            coop.push(share(&w, |s| s.k().is_some_and(|k| k <= 0)));
            unc.push(share(&w, |s| s.k().is_some_and(|k| k <= -4)));
        }
        let (mut state, mut last_hi, mut collapses, mut recoveries) = (0, 0, Vec::new(), 0);
        for (t, &x) in coop.iter().enumerate() {
            if x >= 0.9 {
                recoveries += usize::from(state == -1);
                (state, last_hi) = (1, t);
            } else if x <= 0.1 {
                if state == 1 {
                    collapses.push(last_hi);
                }
                state = -1;
            }
        }
        let in_coop: Vec<f64> = (0..coop.len())
            .filter(|&t| coop[t] >= 0.9)
            .map(|t| unc[t])
            .collect();
        let before: Vec<f64> = collapses
            .iter()
            .map(|&t| mean(&unc[t.saturating_sub(50)..=t]))
            .collect();
        (collapses.len(), recoveries, mean(&in_coop), before)
    });
    let collapses = r.iter().map(|p| p.0).sum();
    let recoveries = r.iter().map(|p| p.1).sum();
    let base = mean(&r.iter().map(|p| p.2).collect::<Vec<_>>());
    let before: Vec<f64> = r.iter().flat_map(|p| p.3.clone()).collect();
    (collapses, recoveries, base, mean(&before))
}

#[test]
#[ignore]
fn ns98_fig_2_cycles_and_unconditional_cooperators_come_first() {
    // NS98 Fig. 2: "endless cycles of cooperation and defection"; k = −4 or
    // −5 undermine cooperative populations, then defectors invade. 10⁶
    // generations in all (seeds 1–10 × 10⁵).
    let (collapses, recoveries, base, before) = cycles(&config("ns-fig-2", |_| {}), 100_000);
    println!("collapses {collapses}, recoveries {recoveries}; k ≤ −4: {base:.3} in cooperative generations, {before:.3} before collapses");
    assert_eq!((collapses, recoveries), (172, 167));
    pin("k ≤ −4 in cooperative generations", base, 0.081, 0.001, "—");
    pin(
        "k ≤ −4 before collapses",
        before,
        0.682,
        0.001,
        "rises first",
    );
    let c = config("ns-fig-2", |_| {});
    pin(
        "Fig. 2 cooperative (k ≤ 0), gens 1,001–100,000",
        mean(&window(&c, "cooperative", 10, 100_000, 1001)),
        0.672,
        0.001,
        "not given",
    );
    // Ours: without the offset.
    let c = config("ns-fig-2", |c| c.offset = Offset::None);
    pin(
        "Fig. 2 without the offset",
        mean(&window(&c, "cooperative", 10, 100_000, 1001)),
        0.777,
        0.001,
        "—",
    );
}

fn fig_3(n: u32, edit: impl FnOnce(&mut ImageConfig)) -> f64 {
    let id = match n {
        20 => "ns-fig-3-n20",
        50 => "ns-fig-3-n50",
        _ => "ns-fig-3-n100",
    };
    mean(&window(&config(id, edit), "cooperative", 10, 20_000, 1001))
}

#[test]
#[ignore]
fn ns98_fig_3_group_size_with_ten_observers() {
    // NS98 Fig. 3: cooperative strategies (k ≤ 0) 90%, 47%, 18% at n = 20,
    // 50, 100 (10⁷ generations). Here seeds 1–10, generations 1,001–20,000,
    // each observer keeping its own tally (the default).
    pin("n = 20", fig_3(20, |_| {}), 0.8629, 0.0005, "0.90");
    pin("n = 50", fig_3(50, |_| {}), 0.4397, 0.0005, "0.47");
    pin("n = 100", fig_3(100, |_| {}), 0.2008, 0.0005, "0.18");
}

#[test]
#[ignore]
fn ns98_fig_3_when_an_observer_learns_the_whole_score() {
    // `records: score`: one sighting reveals the donor's score. The group-size
    // effect all but vanishes.
    let score = |c: &mut ImageConfig| c.records = Records::Score;
    pin("n = 20, score", fig_3(20, score), 0.9676, 0.0005, "0.90");
    pin("n = 50, score", fig_3(50, score), 0.9279, 0.0005, "0.47");
    pin("n = 100, score", fig_3(100, score), 0.9234, 0.0005, "0.18");
}

#[test]
#[ignore]
fn ns98_fig_3_with_fair23s_fixed_visibility() {
    // FAIR23 sees each interaction with a fixed probability (0.1 by
    // default), not ten observers: 1.8, 4.8 and 9.8 observers at n = 20, 50,
    // 100. The group-size effect goes with it.
    pin(
        "n = 20, visibility 0.1",
        fig_3(20, |c| c.observers = 1.8),
        0.2824,
        0.0005,
        "0.90",
    );
    pin(
        "n = 50, visibility 0.1",
        fig_3(50, |c| c.observers = 4.8),
        0.1551,
        0.0005,
        "0.47",
    );
}

/// The most frequent strategies of `c` over generations 1,001–`ticks`,
/// seeds 1–10, with their shares.
fn most_frequent(c: &ImageConfig, ticks: u64) -> Vec<(Strategy, f64)> {
    let r = par(1..=10, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        let mut counts = std::collections::HashMap::new();
        for t in 1..=ticks {
            w.step();
            if t > 1000 {
                for a in w.agents() {
                    *counts.entry(a.strategy).or_insert(0u64) += 1;
                }
            }
        }
        counts
    });
    let mut all = std::collections::HashMap::new();
    for c in r {
        for (s, n) in c {
            *all.entry(s).or_insert(0u64) += n;
        }
    }
    let total: u64 = all.values().sum();
    let mut v: Vec<(Strategy, f64)> = all
        .into_iter()
        .map(|(s, n)| (s, n as f64 / total as f64))
        .collect();
    v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.code().cmp(&b.0.code())));
    v.truncate(3);
    v
}

#[test]
#[ignore]
fn ns98_fig_4_and_and_or_strategies() {
    // NS98 Fig. 4: cooperative interactions 55%, 57%, 70%, 80% in (a)–(d);
    // most frequent (k 0, h 1) in (a), (k 0, h 4) in (b), and the defectors
    // (k 6, h −5) in (c) and (d). Seeds 1–10, generations 1,001–50,000.
    let rows = [
        ("ns-fig-4a", 0.5316, "0.55", Strategy::And { k: 0, h: 1 }),
        ("ns-fig-4b", 0.5344, "0.57", Strategy::And { k: 0, h: 5 }),
        ("ns-fig-4c", 0.7843, "0.70", Strategy::Or { k: 3, h: 4 }),
        ("ns-fig-4d", 0.8545, "0.80", Strategy::Or { k: 2, h: 5 }),
    ];
    for (id, want, source, first) in rows {
        let c = config(id, |_| {});
        let help = mean(&window(&c, "help_rate", 10, 50_000, 1001));
        pin(&format!("{id} help rate"), help, want, 0.0005, source);
        let top = most_frequent(&c, 50_000);
        println!("{id} most frequent: {top:?}");
        assert_eq!(top[0].0, first, "{id}");
    }
}

#[test]
#[ignore]
fn ns98_own_score_strategies_barely_cooperate() {
    // NS98: "less than 0.1% cooperation" with strategies that only consider
    // their own score. Mutation (0.001, uniform) keeps a floor above that.
    let c = config("ns-own-only", |_| {});
    pin(
        "own-only help rate",
        mean(&window(&c, "help_rate", 10, 20_000, 1001)),
        0.0019,
        0.0001,
        "< 0.001",
    );
}

#[test]
#[ignore]
fn ns98_two_interactions_per_lifetime() {
    // NS98: "it suffices that each player is chosen only for about 2
    // interactions per life-time" — m ≈ n (each player takes part in 2m/n).
    // Fig. 2's settings, seeds 1–10, generations 1,001–20,000.
    for (m, want) in [(25, 0.0134), (50, 0.1004), (100, 0.1815), (200, 0.5003)] {
        let c = config("ns-fig-2", |c| c.rounds = m);
        pin(
            &format!("m = {m} cooperative"),
            mean(&window(&c, "cooperative", 10, 20_000, 1001)),
            want,
            0.0005,
            "prevails from m ≈ n",
        );
    }
}

#[test]
#[ignore]
fn ns98_methods_thresholds() {
    let m = Binary::ns98();
    // "about 1.2 rounds per generation" at b = 1, c = 0.1, q = 1.
    pin(
        "minimum rounds (bq + c)/(bq − c)",
        m.min_rounds(),
        1.2222,
        0.0001,
        "about 1.2",
    );
    // q > c/b: at q = 0.1 no number of rounds suffices.
    assert_eq!(Binary { q: 0.1, ..m }.min_rounds(), f64::INFINITY);
    // The equilibrium with cooperators, x = c(2 − w)/(bwq), at w = 0.9.
    pin(
        "x with cooperators (w 0.9)",
        m.cooperator_equilibrium(0.9),
        0.1222,
        0.0001,
        "c(2 − w)/(bwq)",
    );
}

#[test]
#[ignore]
fn ns98_methods_x_min_against_a_simulation() {
    // Binary discriminators (k 0) against defectors (k 1), no offset,
    // perfect information, n = 100, m = 250 (5 rounds of the Methods'
    // everyone-plays-once rounds): one generation's payoff gap, seeds
    // 1–2,000 at each starting share.
    let a = Binary::ns98();
    let x_min = a.x_min_fixed(5).unwrap();
    let gap = |x: f64| {
        let c = ImageConfig {
            rounds: 250,
            offset: Offset::None,
            strategies: vec![Class::Binary],
            initial: Initial::Seeded(Seeded {
                only: Strategy::Binary(1),
                invader: Some(Strategy::Binary(0)),
                share: x,
            }),
            ..Default::default()
        };
        mean(&par(1..=2000, |seed| {
            let mut w = ImageWorld::new(c.clone(), seed).unwrap();
            w.step();
            let (mut d, mut e) = (Vec::new(), Vec::new());
            for a in w.agents() {
                if a.strategy == Strategy::Binary(0) {
                    d.push(a.payoff)
                } else {
                    e.push(a.payoff)
                }
            }
            mean(&d) - mean(&e)
        }))
    };
    // The share where the gap changes sign, to 0.01.
    let mut crossing = 0.0;
    for i in 10..=20 {
        let x = f64::from(i) / 100.0;
        if gap(x) > 0.0 {
            crossing = x;
            break;
        }
    }
    pin(
        "analytic x_min (5 rounds)",
        x_min,
        0.1228,
        0.0001,
        "Methods",
    );
    pin("simulated crossing", crossing, 0.16, 0.0005, "x_min");
}

#[test]
#[ignore]
fn ns98_universal_constant_under_each_start() {
    // "0.7380294688360…": the largest fraction below 0 from which everyone
    // at k = 0 reaches all-out cooperation; the start is not stated.
    let rows = [
        (Start::AtMinusOne { rest: Some(0) }, 0.5),
        (Start::AtMinusOne { rest: Some(1) }, 0.6420045049661),
        (Start::AtMinusOne { rest: Some(2) }, 0.6878695524337),
        (Start::AtMinusOne { rest: Some(5) }, 0.7263483514787),
        (Start::AtMinusOne { rest: Some(20) }, 0.7379660293712),
        (Start::AtMinusOne { rest: Some(80) }, 0.7380294688227),
        (Start::AtMinusOne { rest: None }, 0.7380294688360),
        (Start::Spread { below: 2 }, 0.4209145147265),
        (Start::Spread { below: 5 }, 0.3384593809259),
        (Start::SpreadBoth { below: 5, above: 5 }, 0.5157902242303),
    ];
    let got = par(0..rows.len() as u64, |i| {
        analytic::universal_threshold(rows[i as usize].0)
    });
    for ((start, want), got) in rows.iter().zip(got) {
        pin(&format!("{start:?}"), got, *want, 1e-12, "0.7380294688360");
    }
}

/// The share of strategies matching `is` at each generation in `at`, per seed.
fn invasion(
    c: &ImageConfig,
    seeds: u64,
    at: &[u64],
    is: impl Fn(Strategy) -> bool + Sync,
) -> Vec<Vec<f64>> {
    par(1..=seeds, |seed| {
        let mut w = ImageWorld::new(c.clone(), seed).unwrap();
        at.iter()
            .map(|&t| {
                while w.tick < t {
                    w.step();
                }
                share(&w, &is)
            })
            .collect()
    })
}

fn column(v: &[Vec<f64>], i: usize) -> Vec<f64> {
    v.iter().map(|r| r[i]).collect()
}

#[test]
#[ignore]
fn lh01_fig_1_h_1_invades() {
    // LH01 Fig. 1a: h = 1 invades k = 0 (e = 0); 1b: it invades (k 0, h 1)
    // with e = 0.05, "but does not wipe out". Invaders start at 1% of each
    // group; seeds 1–10.
    let v = invasion(&config("lh-fig-1a", |_| {}), 10, &[50, 100, 150], |s| {
        s == Strategy::H(1)
    });
    pin(
        "1a h = 1 at 50",
        mean(&column(&v, 0)),
        0.3704,
        0.0005,
        "rising",
    );
    pin(
        "1a h = 1 at 150",
        mean(&column(&v, 2)),
        1.0,
        0.0005,
        "about 0.8",
    );
    let v = invasion(&config("lh-fig-1b", |_| {}), 10, &[150, 500, 1000], |s| {
        s == Strategy::H(1)
    });
    pin(
        "1b h = 1 at 150",
        mean(&column(&v, 0)),
        0.0183,
        0.0005,
        "invading",
    );
    pin("1b h = 1 at 500", mean(&column(&v, 1)), 0.1317, 0.0005, "—");
    pin(
        "1b h = 1 at 1,000",
        mean(&column(&v, 2)),
        0.4232,
        0.0005,
        "—",
    );
}

#[test]
#[ignore]
fn lh01_fig_2a_one_group() {
    // LH01 Fig. 2a: help in 39% of rounds (10⁶ generations), (k 0, h 1)
    // dominant. Seeds 1–10, generations 1,001–100,000.
    let c = config("lh-fig-2a", |_| {});
    pin(
        "2a help",
        mean(&window(&c, "help_rate", 10, 100_000, 1001)),
        0.3742,
        0.0005,
        "0.39",
    );
    let top = most_frequent(&c, 100_000);
    println!("2a most frequent: {top:?}");
    assert_eq!(top[0].0, Strategy::And { k: 0, h: 1 });
}

#[test]
#[ignore]
fn lh01_fig_2bc_the_island_model() {
    // LH01 Fig. 2b: 9% help (p 0.9, 10⁵ generations); 2c: 2% (p 0.5).
    // Seeds 1–10, generations 1,001–5,000.
    let c = config("lh-fig-2b", |_| {});
    pin(
        "2b help",
        mean(&window(&c, "help_rate", 10, 5000, 1001)),
        0.4413,
        0.0005,
        "0.09",
    );
    let c = config("lh-fig-2c", |_| {});
    pin(
        "2c help",
        mean(&window(&c, "help_rate", 10, 5000, 1001)),
        0.1544,
        0.0005,
        "0.02",
    );
}

#[test]
#[ignore]
fn lh01_fig_3_a_small_cost_and_q_strategies() {
    // LH01 Fig. 3a: 45% help (c 0.1, u₀ 5, p 0.5, e 0.02; 2 × 10⁵
    // generations); 3b–c with q strategies: 15% help, 12% q strategies.
    // Seeds 1–10, generations 1,001–3,000.
    let c = config("lh-fig-3a", |_| {});
    pin(
        "3a help",
        mean(&window(&c, "help_rate", 10, 3000, 1001)),
        0.522,
        0.0005,
        "0.45",
    );
    let c = config("lh-fig-3b", |_| {});
    let v = windows(&c, &["help_rate", "q"], 10, 3000, 1001);
    pin("3b help", mean(&column(&v, 0)), 0.1653, 0.0005, "0.15");
    pin("3b q share", mean(&column(&v, 1)), 0.2579, 0.0005, "0.12");
}

#[test]
#[ignore]
fn lh01_fig_4ab_standing_invades_discriminators() {
    // LH01 Fig. 4a (e 0.05) and 4b (e = ε = 0.025): standing takes over a
    // population of binary discriminators within 1,000 generations. Standing
    // starts at 1% of each group; seeds 1–10 (4a), 1–5 (4b).
    let standing = |s: Strategy| s == Strategy::Standing;
    let v = invasion(
        &config("lh-fig-4a", |_| {}),
        10,
        &[250, 500, 1000],
        standing,
    );
    pin(
        "4a standing at 500",
        mean(&column(&v, 1)),
        0.6939,
        0.0005,
        "rising",
    );
    pin(
        "4a standing at 1,000",
        mean(&column(&v, 2)),
        0.9841,
        0.0005,
        "near 1",
    );
    let v = invasion(&config("lh-fig-4b", |_| {}), 5, &[500, 1000], standing);
    pin(
        "4b standing at 500",
        mean(&column(&v, 0)),
        0.3523,
        0.0005,
        "rising",
    );
    pin(
        "4b standing at 1,000",
        mean(&column(&v, 1)),
        0.7459,
        0.0005,
        "near 1",
    );
}

#[test]
#[ignore]
fn lh01_fig_4c_standing_persists_with_cooperators() {
    // LH01 Fig. 4c: over 10⁵ generations standing dominates, cooperators
    // stay at appreciable frequencies, discriminators occasionally reach 5%,
    // defectors stay below 1%. Here seeds 1–3, generations 1,001–1,500 (a
    // uniform start).
    let c = config("lh-fig-4c", |_| {});
    let v = windows(
        &c,
        &["standing", "binary_c", "binary_x", "binary_d"],
        3,
        1500,
        1001,
    );
    pin(
        "4c standing",
        mean(&column(&v, 0)),
        0.5309,
        0.0005,
        "dominant",
    );
    pin(
        "4c cooperators",
        mean(&column(&v, 1)),
        0.3886,
        0.0005,
        "appreciable",
    );
    pin(
        "4c discriminators",
        mean(&column(&v, 2)),
        0.0803,
        0.0005,
        "occasionally 0.05",
    );
    pin(
        "4c defectors",
        mean(&column(&v, 3)),
        0.0002,
        0.0001,
        "< 0.01",
    );
    // The condition vrb < c < rb: v = 0.5, r = 0.833 — not met at c = 0.25.
    assert!(!analytic::standing_stable(
        1.0, 0.25, 100, 500, 0.025, 0.025
    ));
    pin("r", analytic::standing_r(100, 500), 0.833, 0.001, "0.833");
}
```
Run: `cargo test -p sugarscape-core --release --test image -- --ignored --nocapture --test-threads=1`
Expected: `test result: ok. 18 passed` in about 4.5 minutes (the slowest, `lh01_fig_4c_standing_persists_with_cooperators`, about 55 s; `ns98_fig_3_group_size_with_ten_observers` about 47 s). Each test prints its measurements beside the sources'.

- [ ] **Step 2: The sweeps — failing lists first**

Add the four ids to the id lists. `crates/sugarscape-core/src/sweep.rs` (the id-list hunk):

```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -2055,7 +2074,11 @@
                 "dpd-payoffs",
                 "dpd-mutation",
                 "dpd-metabolism",
-                "dpd-max-age"
+                "dpd-max-age",
+                "ns-rounds",
+                "ns-group-size",
+                "lh-cost",
+                "lh-gene-flow"
             ]
         );
         for b in builtins() {
```
`crates/sugarscape-cli/tests/cli.rs`:

```diff
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -102,6 +102,10 @@
         "dpd-mutation",
         "dpd-metabolism",
         "dpd-max-age",
+        "ns-rounds",
+        "ns-group-size",
+        "lh-cost",
+        "lh-gene-flow",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
```
Run: `cargo test -p sugarscape-core --lib builtin_sweeps_parse_and_validate`
Expected: FAIL — `assertion `left == right` failed` (the list ends at `dpd-max-age`).

- [ ] **Step 3: The sweeps**

Create `sweeps/ns-rounds.json`:

```json
{
  "name": "Image scoring: cooperation against rounds per generation (NS98 Fig. 2)",
  "description": "NS98: \"Cooperation is more likely to win the greater the number m of interactions per generation\", and \"it suffices that each player is chosen only for about 2 interactions per life-time\" (m ≈ n). Fig. 2's settings (n = 100, mutation 0.001), the share of cooperative strategies (k ≤ 0) averaged over generations 1,001–20,000. Measured (release, seeds 1–10, recorded 2026-09-26): m 25: 1%; 50: 10%; 75: 8%; 100: 18%; 125: 15%; 150: 23%; 200: 50%; 300: 65%; 500: 91%. More rounds, more cooperation — but at two interactions per lifetime (m = 100) cooperative strategies hold 18% of the time; they reach half only at m = 200 (four per lifetime). Not reproduced.",
  "base": {
    "preset": "ns-fig-2"
  },
  "x": {
    "label": "Rounds per generation (m)",
    "path": "rounds",
    "values": [
      25,
      50,
      75,
      100,
      125,
      150,
      200,
      300,
      500
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 20000,
  "metric": {
    "kind": "window_mean",
    "series": "cooperative",
    "from": 1001
  }
}
```
Create `sweeps/ns-group-size.json`:

```json
{
  "name": "Image scoring: cooperation against group size with ten observers (NS98 Fig. 3)",
  "description": "NS98 Fig. 3: with ten observers per interaction, cooperation gets harder as groups grow (90%, 47%, 18% at n = 20, 50, 100). m = 10n, mutation 0.001, each member keeping its own tally; cooperative strategies (k ≤ 0) averaged over generations 1,001–20,000. Measured (release, seeds 1–10, recorded 2026-09-26): n 20: 86%; 30: 83%; 50: 44%; 70: 35%; 100: 20%. Reproduced.",
  "base": {
    "preset": "ns-fig-3-n50"
  },
  "x": {
    "label": "Group size (n; m = 10n)",
    "values": [
      {
        "at": 20,
        "name": "n = 20",
        "set": {
          "group_size": 20,
          "rounds": 200
        }
      },
      {
        "at": 30,
        "name": "n = 30",
        "set": {
          "group_size": 30,
          "rounds": 300
        }
      },
      {
        "at": 50,
        "name": "n = 50",
        "set": {
          "group_size": 50,
          "rounds": 500
        }
      },
      {
        "at": 70,
        "name": "n = 70",
        "set": {
          "group_size": 70,
          "rounds": 700
        }
      },
      {
        "at": 100,
        "name": "n = 100",
        "set": {
          "group_size": 100,
          "rounds": 1000
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 20000,
  "metric": {
    "kind": "window_mean",
    "series": "cooperative",
    "from": 1001
  }
}
```
Create `sweeps/lh-cost.json`:

```json
{
  "name": "Image scoring: help against the cost of helping, one group or islands (LH01)",
  "description": "LH01: image scoring \"does very badly\" when c/b = 0.5 and needs a small cost to persist when drift is limited. AND strategies, m = 500, execution errors 0.02, mutation 0.001; help rate over generations 1,001–5,000, one group of 100 against 100 groups with p = 0.9. Measured (release, seeds 1–10, recorded 2026-09-26) — one group: c 0.05: 58%; 0.1: 44%; 0.15: 42%; 0.25: 35%; 0.35: 9%; 0.5: 6%. Islands: 53%, 50%, 51%, 44%, 2.2%, 2.1%. Both collapse above c = 0.25 (the islands to the 2% that execution errors give); below it the island model helps more, not less, than one group.",
  "base": {
    "preset": "lh-fig-2b"
  },
  "x": {
    "label": "Cost c (b = 1)",
    "path": "c",
    "values": [
      0.05,
      0.1,
      0.15,
      0.25,
      0.35,
      0.5
    ]
  },
  "series": {
    "label": "Population",
    "values": [
      {
        "at": 1,
        "name": "One group of 100",
        "set": {
          "groups": 1
        }
      },
      {
        "at": 100,
        "name": "100 groups, p = 0.9",
        "set": {
          "groups": 100
        }
      }
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 5000,
  "metric": {
    "kind": "window_mean",
    "series": "help_rate",
    "from": 1001
  }
}
```
Create `sweeps/lh-gene-flow.json`:

```json
{
  "name": "Image scoring: help against gene flow in the island model (LH01 Fig. 2)",
  "description": "LH01 Fig. 2: less drift, less image scoring — 9% help at p = 0.9 and 2% at p = 0.5 (10⁵ generations). Fig. 2b's settings (100 groups of 100, AND strategies, c 0.25, errors 0.02), the help rate over generations 1,001–5,000 against the chance p that a parent is local. Measured (release, seeds 1–10, recorded 2026-09-26): p 0.5: 15%; 0.6: 35%; 0.7: 39%; 0.8: 50%; 0.9: 44%; 1 (isolated groups): 27%. Not monotone and not reproduced: some gene flow raises help above isolated groups, and only at p = 0.5 does it fall toward LH01's levels.",
  "base": {
    "preset": "lh-fig-2b"
  },
  "x": {
    "label": "Local parents (p)",
    "path": "local",
    "values": [
      0.5,
      0.6,
      0.7,
      0.8,
      0.9,
      1.0
    ]
  },
  "seeds": {
    "from": 1,
    "count": 10
  },
  "ticks": 5000,
  "metric": {
    "kind": "window_mean",
    "series": "help_rate",
    "from": 1001
  }
}
```
`crates/sugarscape-core/src/sweep.rs` (BUILTINS):

```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -962,7 +965,7 @@
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 54] = [
+const BUILTINS: [Builtin; 58] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1179,6 +1182,22 @@
         id: "dpd-max-age",
         json: include_str!("../../../sweeps/dpd-max-age.json"),
     },
+    Builtin {
+        id: "ns-rounds",
+        json: include_str!("../../../sweeps/ns-rounds.json"),
+    },
+    Builtin {
+        id: "ns-group-size",
+        json: include_str!("../../../sweeps/ns-group-size.json"),
+    },
+    Builtin {
+        id: "lh-cost",
+        json: include_str!("../../../sweeps/lh-cost.json"),
+    },
+    Builtin {
+        id: "lh-gene-flow",
+        json: include_str!("../../../sweeps/lh-gene-flow.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
```
Run: `cargo test -p sugarscape-core --lib sweep:: && cargo test -p sugarscape-cli`
Expected: `ok` (`builtin_sweeps_parse_and_validate` passes; the CLI lists the four ids).
Run each: `cargo run --release -p sugarscape-cli -- sweep --builtin <id> --quiet --summary-csv /dev/stdout --out /dev/null`
Expected: the means in each description (Decision M): `ns-rounds` about 3 s, `ns-group-size` about 31 s, `lh-gene-flow` about 82 s, `lh-cost` about 94 s on 10 threads.

- [ ] **Step 4: WASM**

`crates/sugarscape-wasm/tests/web.rs` (the id list and a fingerprint test):

```diff
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -304,7 +304,11 @@
             "dpd-payoffs",
             "dpd-mutation",
             "dpd-metabolism",
-            "dpd-max-age"
+            "dpd-max-age",
+            "ns-rounds",
+            "ns-group-size",
+            "lh-cost",
+            "lh-gene-flow"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -806,6 +810,24 @@
         let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
         assert_eq!(sim.model_kind(), "dpd");
         sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
+#[wasm_bindgen_test]
+fn image_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, IMAGE_GOLDEN: payoff sums,
+    // roulette draws and the observers' records are the same bits here as
+    // natively (perfect information, observers with AND strategies, and the
+    // island model with q strategies).
+    for (id, ticks, fp) in [
+        ("ns-fig-1", 200, "0x98875bd71738cf05"),
+        ("ns-fig-4b", 200, "0x0a4c19c5fa5fcfe3"),
+        ("lh-fig-3b", 20, "0x4d6575c59b0da89a"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "image");
+        sim.step(ticks);
         assert_eq!(sim.fingerprint(), fp, "{id}");
     }
 }
```
Run: `wasm-pack test --node crates/sugarscape-wasm`
Expected: `test result: ok. 42 passed` (including `image_sims_match_the_native_golden_entries`).

- [ ] **Step 5: Verify and commit**

Run: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo +1.98.1 clippy --all-targets -- -D warnings && cargo test --workspace`
Expected: clean; every suite `ok`.

```bash
git add crates/sugarscape-core/tests/image.rs sweeps/ns-rounds.json sweeps/ns-group-size.json sweeps/lh-cost.json sweeps/lh-gene-flow.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure image scoring against Nowak and Sigmund and Leimar and Hammerstein" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 3: Image scoring on the page — types, the Rules panel, charts and Inspect

**Files:**
- Create: `web/src/image-scoring.ts`, `web/src/image-scoring.test.ts`
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/engine.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`
- Test: `web/src/models.test.ts`, `web/src/schema-form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/engine.test.ts`, `web/src/sim-host.test.ts`

**Interfaces:**
- Consumes: Tasks 1–2's `ModelKind::Image` (`"image"`), its schema (groups Game, Population, Rounds, Information, Errors, Evolution, Run; `observers` `shown_if("information", "observers")`), the inspection JSON `{cell, group, agent}` (Decision 15), `SERIES` (17 names), the colour modes `strategy`, `score`, `payoff`.
- Produces: `ImageClass`, `ImageStrategy`, `ImageConfig`, `ImageStats`, `ImageAgentView`, `ImageInspection` (types.ts); `ModelKind` gains `'image'`, `ColorMode` gains `'score'`; `isImageView(v)`, the `MODELS`/`MODEL_LABELS`/`COLOR_MODES`/`MODEL_OVERLAYS` entries, `ticksLeft` for image (models.ts); `MODEL_CHARTS.image`, `timeAxisLabel('image') === 'Generation'` (series-data.ts); `imageRows(view, config): [string, string][]` (image-scoring.ts); `finishedNotice` for image.

- [ ] **Step 1: Write the failing tests**

```diff
--- a/web/src/models.test.ts
+++ b/web/src/models.test.ts
@@ -10,6 +10,7 @@
   isCultureView,
   isDpdView,
   isEthnoView,
+  isImageView,
   isRingView,
   isSpatialView,
   isSugar,
@@ -355,3 +356,47 @@
     expect(calendarYear(dpd(500), 5)).toBeNull();
   });
 });
+
+describe('image scoring', () => {
+  const image = (end: number) => ({ model: 'image', end }) as unknown as ModelConfig;
+
+  it('is read by its tag, and its cells by their shape, which no other model’s inspection shares', () => {
+    expect(modelOf(image(0))).toBe('image');
+    expect(isSugar(image(0))).toBe(false);
+    const gap = { cell: { x: 10, y: 0 }, group: null, agent: null } as AnyInspection;
+    const agent = { cell: { x: 1, y: 2 }, group: 0, agent: { id: 3, group: 0, strategy: 'k = 0', class: 'k', score: 1 } } as unknown as AnyInspection;
+    expect([gap, agent].map(isImageView)).toEqual([true, true]);
+    // The guards that read `site` check for one first (an image cell has none).
+    for (const v of [gap, agent]) {
+      expect([isSugarView(v), isRingView(v), isValleyView(v), isSpatialView(v)]).toEqual([false, false, false, false]);
+      expect([isCivilView(v), isTagsView(v), isClassesView(v), isCultureView(v), isStructureView(v), isOpinionsView(v)]).toEqual([false, false, false, false, false, false]);
+      expect([isEthnoView(v, 'image'), isDpdView(v, 'image')]).toEqual([false, false]);
+    }
+    const empty = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
+    const sugar = { site: { x: 1, y: 2, resources: [] }, agent: null } as unknown as AnyInspection;
+    expect([empty, sugar].map(isImageView)).toEqual([false, false]);
+  });
+
+  it('offers strategy, score and payoff colors and no overlays, and is grouped last', () => {
+    expect(COLOR_MODES.image).toEqual([
+      ['strategy', 'Strategy'],
+      ['score', 'Score'],
+      ['payoff', 'Payoff'],
+    ]);
+    expect(MODEL_OVERLAYS.image).toEqual([]);
+    const p = (id: string, config: object) => ({ id, name: id, source: '', description: '', config }) as unknown as Preset;
+    expect(presetGroups([p('ns', { model: 'image' }), p('dpd', { model: 'dpd' }), p('ha', { model: 'ethno' })]).map((g) => g.label)).toEqual([
+      'Ethnocentrism',
+      'Demographic PD',
+      'Image Scoring',
+    ]);
+  });
+
+  it('counts down to its last generation, or never with none (the default)', () => {
+    expect(ticksLeft(image(500), 490)).toBe(10);
+    expect(ticksLeft(image(500), 505)).toBe(0);
+    expect(ticksLeft(image(0), 5)).toBe(Infinity);
+    expect(finishesUnpredictably(image(500))).toBe(false);
+    expect(calendarYear(image(500), 5)).toBeNull();
+  });
+});
```

```diff
--- a/web/src/schema-form.test.ts
+++ b/web/src/schema-form.test.ts
@@ -1,6 +1,6 @@
 import { describe, expect, it } from 'vitest';
 import { describedBy, groupParams, paramEdit, paramInput, paramShown, paramSlider } from './schema-form';
-import type { AnasaziConfig, DpdConfig, EthnoConfig, ModelConfig, Param, RingConfig, SchellingConfig } from './types';
+import type { AnasaziConfig, DpdConfig, EthnoConfig, ImageConfig, ModelConfig, Param, RingConfig, SchellingConfig } from './types';
 
 const ring = (): RingConfig => ({
   model: 'ring',
@@ -148,4 +148,22 @@
     paramEdit(param({ path: 'pairing', kind: 'choice' }), 'soup')(c);
     expect(paramShown(play, c)).toBe(false);
   });
+
+  it('reads and writes small rates and whole limits, and shows the observers only with observers (image scoring)', () => {
+    const c = { model: 'image', mutation: 0.0001, clamp: 5, observers: 10, information: 'perfect' } as unknown as ImageConfig;
+    const mutation = param({ path: 'mutation', kind: 'number', min: 0, max: 0.1, step: 0.0001 });
+    expect([paramInput(mutation, c), paramSlider(mutation, c)]).toEqual(['0.0001', '0.0001']);
+    paramEdit(mutation, '0.001')(c);
+    expect(c.mutation).toBe(0.001);
+    // Clamp 0 means unbounded scores; an integer box rounds.
+    const clamp = param({ path: 'clamp', kind: 'integer', min: 0, max: 100 });
+    paramEdit(clamp, '0')(c);
+    expect(c.clamp).toBe(0);
+    paramEdit(clamp, '2.6')(c);
+    expect(c.clamp).toBe(3);
+    const observers = { path: 'observers', label: 'Observers per interaction', kind: 'number', apply: 'live', group: 'Information', show_if: { path: 'information', equals: 'observers' } } as Param;
+    expect(paramShown(observers, c)).toBe(false);
+    paramEdit(param({ path: 'information', kind: 'choice' }), 'observers')(c);
+    expect(paramShown(observers, c)).toBe(true);
+  });
 });
```

```diff
--- a/web/src/ui/series-data.test.ts
+++ b/web/src/ui/series-data.test.ts
@@ -337,3 +337,45 @@
     expect(timeAxisLabel('dpd')).toBe('Cycle');
   });
 });
+
+describe('image scoring’s charts', () => {
+  const image = (strategies: string[]) => ({ model: 'image', strategies }) as unknown as ModelConfig;
+
+  it('charts the help rate with cooperative strategies, mean k, strategy shares and mean payoff against the generation', () => {
+    expect(MODEL_CHARTS.image.map((c) => c.title)).toEqual(['Help rate', 'Mean k', 'Strategy shares', 'Binary scorers and standing', 'Mean payoff']);
+    expect(MODEL_CHARTS.image.map((c) => c.lines.map((l) => [l.key, l.color]))).toEqual([
+      [
+        ['help_rate', '--c1'],
+        ['cooperative', '--blue'],
+      ],
+      [['mean_k', '--c4']],
+      [
+        ['k_cooperative', '--blue'],
+        ['k_defective', '--red'],
+        ['h', '--c3'],
+        ['own_only', '--muted'],
+        ['and', '--c1'],
+        ['or', '--c4'],
+        ['standing', '--c2'],
+        ['q', '--lender'],
+      ],
+      [
+        ['binary_c', '--blue'],
+        ['binary_x', '--c3'],
+        ['binary_d', '--red'],
+        ['standing', '--c2'],
+      ],
+      [['mean_payoff', '--c2']],
+    ]);
+    expect(MODEL_CHARTS.image.map((c) => c.range)).toEqual([[0, 1], [-5, 6], [0, 1], [0, 1], undefined]);
+    expect(timeAxisLabel('image')).toBe('Generation');
+  });
+
+  it('shows mean k only with a class that has a k, and the binary scorers’ shares instead of the others', () => {
+    const shown = (strategies: string[]) => MODEL_CHARTS.image.map((c) => !c.shown || c.shown(image(strategies)));
+    expect(shown(['k'])).toEqual([true, true, true, false, true]);
+    expect(shown(['and', 'q'])).toEqual([true, true, true, false, true]);
+    expect(shown(['own_only'])).toEqual([true, false, true, false, true]);
+    expect(shown(['binary', 'standing'])).toEqual([true, true, false, true, true]);
+  });
+});
```

```diff
--- a/web/src/engine.test.ts
+++ b/web/src/engine.test.ts
@@ -1168,6 +1168,7 @@
     );
     expect(finishedNotice({ model: 'ethno' } as unknown as ModelConfig, 2000)).toBe('This run has reached its last period (2000) — Reset to run it again');
     expect(finishedNotice({ model: 'dpd' } as unknown as ModelConfig, 500)).toBe('This run has reached its last cycle (500) — Reset to run it again');
+    expect(finishedNotice({ model: 'image' } as unknown as ModelConfig, 1000)).toBe('This run has reached its last generation (1000) — Reset to run it again');
     let ends = 0;
     e.on('finished', () => ends++);
     e.setSpeed(4);
```

`web/src/sim-host.test.ts` (Decision 27: `AnyInspection` now has a member without `site`):

```diff
--- a/web/src/sim-host.test.ts
+++ b/web/src/sim-host.test.ts
@@ -97,7 +97,7 @@
       diseaseList: true,
     };
     const full = t.snap(t.send({ type: 'step', n: 1 }, { wants: all }));
-    expect(full.inspection?.view.site).toMatchObject({ x: 0, y: 0 });
+    expect(full.inspection?.view).toMatchObject({ site: { x: 0, y: 0 } });
     expect(full.trail).toBeDefined();
     expect(full.networks?.trade).toEqual(Uint32Array.of(0, 0, 1, 1));
     expect(Object.keys(full.charts ?? {})).toEqual(['population']);
```

Create `web/src/image-scoring.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { imageRows } from './image-scoring';
import type { ImageAgentView, ImageClass, ImageInspection } from './types';

const agent = (a: Partial<ImageAgentView>): ImageAgentView => ({
  id: 57,
  group: 0,
  strategy: 'k = 0',
  class: 'k',
  cooperative: true,
  score: 3,
  standing: true,
  known: null,
  mean_view: null,
  payoff: 1.5,
  given: 2,
  received: 1,
  ...a,
});

const cell = (a: ImageAgentView | null, group: number | null = 0): ImageInspection => ({ cell: { x: 4, y: 1 }, group, agent: a });
const one = { groups: 1, strategies: ['k'] as ImageClass[] };

describe('imageRows', () => {
  it('shows the strategy, whether it helps at the start, the score, the payoff and this generation’s help (NS98, one group)', () => {
    expect(imageRows(cell(agent({})), one)).toEqual([
      ['Cell', '(4, 1)'],
      ['Agent', '#57'],
      ['Strategy', 'k = 0 · helps at a generation’s start'],
      ['Score', '3'],
      ['Payoff', '1.50'],
      ['This generation', 'helped 2 times, was helped 1 time'],
    ]);
  });

  it('names the group of an island model, and with observers how many others have seen the agent act', () => {
    const a = agent({ id: 9, group: 41, strategy: 'k = 2, h = 5 (AND)', class: 'and', cooperative: false, score: -1, known: 4, mean_view: -0.75, payoff: 0, given: 1, received: 0 });
    expect(imageRows(cell(a, 41), { groups: 100, strategies: ['and'] })).toEqual([
      ['Cell', '(4, 1)'],
      ['Group', '42 of 100'],
      ['Agent', '#9'],
      ['Strategy', 'k = 2, h = 5 (AND) · refuses at a generation’s start'],
      ['Score', '-1'],
      ['Seen by', '4 others, whose mean record is -0.75'],
      ['Payoff', '0'],
      ['This generation', 'helped 1 time, was helped 0 times'],
    ]);
    expect(imageRows(cell(agent({ known: 1, mean_view: 2 })), one)[4]).toEqual(['Seen by', '1 other, whose mean record is 2']);
    expect(imageRows(cell(agent({ known: 0, mean_view: null })), one)[4]).toEqual(['Seen by', 'nobody yet: the others record it as 0']);
  });

  it('shows standing when standing plays', () => {
    const a = agent({ strategy: 'standing', class: 'standing', standing: false, score: -1 });
    expect(imageRows(cell(a), { groups: 1, strategies: ['binary', 'standing'] })).toContainEqual(['Standing', 'bad']);
    expect(imageRows(cell(agent({})), one).map(([k]) => k)).not.toContain('Standing');
  });

  it('says when a cell is a gap between groups or an unused cell of a tile', () => {
    expect(imageRows(cell(null, null), { groups: 4, strategies: ['k'] })).toEqual([
      ['Cell', '(4, 1)'],
      ['Agent', 'none (a gap between groups)'],
    ]);
    expect(imageRows(cell(null, 2), { groups: 4, strategies: ['k'] })).toEqual([
      ['Cell', '(4, 1)'],
      ['Group', '3 of 4'],
      ['Agent', 'none (an unused cell of the group’s tile)'],
    ]);
  });
});
```

Run: `(cd web && npm run build)` — Expected: FAIL at `tsc --noEmit` (`'./image-scoring'` cannot be found; `isImageView` is not exported from `./models`; `ImageConfig`, `ImageAgentView`, `ImageClass`, `ImageInspection` are not exported from `./types`; `'image'` is not assignable to `ModelKind`; `MODEL_CHARTS.image` does not exist).

- [ ] **Step 2: Types and models** (Decisions 23, 27)

```diff
--- a/web/src/types.ts
+++ b/web/src/types.ts
@@ -80,8 +80,8 @@
   schedule: ScheduledChange[];
 }
 
-/** The models the playground runs (milestones 9–13). */
-export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd';
+/** The models the playground runs (milestones 9–21). */
+export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'opinions' | 'structure' | 'dpd' | 'image';
 
 /** A fraction range (Schelling's preferences). */
 export interface FRange { min: number; max: number }
@@ -294,6 +294,61 @@
   schedule: ScheduledChange[];
 }
 
+/** An image-scoring strategy class (milestone 21). */
+export type ImageClass = 'k' | 'h' | 'and' | 'or' | 'own_only' | 'binary' | 'standing' | 'q';
+
+/**
+ * One image-scoring strategy as the core writes it: `{k: 0}`, `{h: 1}`, `{and: {k, h}}`,
+ * `{or: {k, h}}`, `{own_only: h}`, `{binary: k}`, `"standing"` or `{q: Δq in hundredths}`.
+ */
+export type ImageStrategy =
+  | { k: number }
+  | { h: number }
+  | { and: { k: number; h: number } }
+  | { or: { k: number; h: number } }
+  | { own_only: number }
+  | { binary: number }
+  | 'standing'
+  | { q: number };
+
+/**
+ * Nowak and Sigmund's image scoring (milestone 21), with Leimar and Hammerstein's island model,
+ * errors, standing and q strategies: g groups of n, each generation m random donor–recipient pairs
+ * per group, offspring in proportion to payoff. A tick is a generation.
+ */
+export interface ImageConfig {
+  model: 'image';
+  groups: number;
+  group_size: number;
+  /** p: the chance an offspring's parent comes from its own group. */
+  local: number;
+  /** m: rounds per group per generation (fixed, or the mean). */
+  rounds: number;
+  rounds_kind: 'fixed' | 'random';
+  b: number;
+  c: number;
+  u0: number;
+  /** `both`: c added to donor and recipient each round (LH01 on NS98). */
+  offset: 'both' | 'none';
+  /** Scores stay in −clamp … +clamp (0: unbounded). */
+  clamp: number;
+  information: 'perfect' | 'observers';
+  /** With observers: the mean number of members besides the pair who see an interaction. */
+  observers: number;
+  /** What an observer writes: its own tally ± 1 (FAIR23), or the donor's new score. */
+  records: 'tally' | 'score';
+  execution_error: number;
+  perception_error: number;
+  mutation: number;
+  /** The classes allowed (presets, files and links set them; the Rules panel does not). */
+  strategies: ImageClass[];
+  /** `"uniform"`, or everyone playing `only` but the first round(share × n) of each group playing `invader`. */
+  initial: 'uniform' | { only: ImageStrategy; invader?: ImageStrategy; share?: number };
+  /** The last generation (0: never). */
+  end: number;
+  schedule: ScheduledChange[];
+}
+
 
 /**
  * Axelrod's culture model (milestone 14): sites with F features of q traits copying a neighbor's
@@ -373,7 +428,7 @@
   stop_at: number;
 }
 
-export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig;
+export type ModelConfig = Config | SchellingConfig | RingConfig | AnasaziConfig | CivilConfig | TagsConfig | SpatialConfig | CultureConfig | ClassesConfig | EthnoConfig | OpinionsConfig | StructureConfig | DpdConfig | ImageConfig;
 
 export interface Preset { id: string; name: string; source: string; description: string; config: ModelConfig }
 
@@ -533,6 +588,36 @@
   relative_given_tag: number | null;
 }
 
+/**
+ * A generation's statistics: the rounds it played and the strategies that played them (tick 0: the
+ * first generation before it plays, so its help rate is null). A share or mean with nobody to count
+ * is null.
+ */
+export interface ImageStats {
+  tick: number;
+  /** Helps ÷ rounds played. */
+  help_rate: number | null;
+  /** The mean k over agents whose strategy has one (k, AND, OR, binary). */
+  mean_k: number | null;
+  /** The share whose strategy helps at a generation's start (k ≤ 0 for the k strategies). */
+  cooperative: number | null;
+  mean_payoff: number | null;
+  mean_score: number | null;
+  k_cooperative: number | null;
+  k_defective: number | null;
+  h: number | null;
+  own_only: number | null;
+  and: number | null;
+  or: number | null;
+  standing: number | null;
+  binary_c: number | null;
+  binary_x: number | null;
+  binary_d: number | null;
+  q: number | null;
+  /** Helps given this generation. */
+  helps: number;
+}
+
 /** A cycle's statistics, of the agents alive at its end. The share and mean wealths are null with nobody to count. */
 export interface DpdStats {
   tick: number;
@@ -610,7 +695,7 @@
   partner_p_slope: number;
 }
 
-export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats;
+export type ModelStats = Snapshot | SchellingStats | RingStats | AnasaziStats | CivilStats | TagsStats | SpatialStats | CultureStats | ClassesStats | EthnoStats | OpinionsStats | StructureStats | DpdStats | ImageStats;
 
 export interface SiteView { x: number; y: number; resources: number[]; capacities: number[]; pollution: number[] }
 export interface LinkView { id: number; alive: boolean }
@@ -789,6 +874,36 @@
 /** A demographic PD site. Empty, it looks exactly like an empty Schelling site: `isDpdView` asks the model. */
 export interface DpdInspection { site: { x: number; y: number }; agent: DpdAgentView | null }
 
+/** An image-scoring agent in the generation that last played. */
+export interface ImageAgentView {
+  id: number;
+  group: number;
+  /** "k = 0", "k = 0, h = 1 (AND)", "standing", "Δq = 0.25" … */
+  strategy: string;
+  class: ImageClass;
+  /** Whether the strategy helps at a generation's start. */
+  cooperative: boolean;
+  score: number;
+  /** Good standing (as everyone would judge it without perception errors). */
+  standing: boolean;
+  /**
+   * With private records: the members who have seen it act this generation, and their mean record
+   * of its score (null when none has); both null with perfect information.
+   */
+  known: number | null;
+  mean_view: number | null;
+  payoff: number;
+  /** Helps given and received this generation. */
+  given: number;
+  received: number;
+}
+/**
+ * A cell of the image-scoring frame: its group's tile (null in a gap) and the agent there (null in a
+ * gap or a tile's unused cell). It has `cell`, not `site`: the guards that read `site` check that
+ * there is one.
+ */
+export interface ImageInspection { cell: { x: number; y: number }; group: number | null; agent: ImageAgentView | null }
+
 /** What a world of any model says about a site. */
 /** A culture site: its position, traits, and the sizes of its region and zone. */
 export interface CultureSiteView { x: number; y: number; traits: number[]; region_size: number; zone_size: number }
@@ -858,13 +973,14 @@
   agent: StructureAgentView | null;
 }
 
-export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection;
+export type AnyInspection = Inspection | SchellingInspection | RingInspection | AnasaziInspection | CivilInspection | TagsInspection | SpatialInspection | CultureInspection | ClassesInspection | EthnoInspection | OpinionsInspection | StructureInspection | DpdInspection | ImageInspection;
 
 /**
  * A sugarscape color mode, or (Schelling) `color`, `satisfaction`, `preference`, or (the anasazi)
  * `occupation`, `zones`, `yield`, or (civil violence) `action`, `grievance`, `group`, or (tags)
  * `count`, `tolerance`, `clones`, or (ethnocentrism) `strategy`, `tag`, `lineage`, `ptr`, or (the
- * demographic PD) `strategy`, `wealth`, `age`, `surrounded`.
+ * demographic PD) `strategy`, `wealth`, `age`, `surrounded`, or (image scoring) `strategy`, `score`,
+ * `payoff`.
  */
 export type ColorMode =
   | 'tribe'
@@ -902,7 +1018,8 @@
   | 'friendliness'
   | 'provocability'
   | 'strategy'
-  | 'surrounded';
+  | 'surrounded'
+  | 'score';
 export type Layer = `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}`;
 
 /** WASM calls throw a JSON string of FieldError[]; anything else becomes one error. */
```

```diff
--- a/web/src/models.ts
+++ b/web/src/models.ts
@@ -1,4 +1,4 @@
-// Which model a config is (milestones 9–17), and what each model offers the page.
+// Which model a config is (milestones 9–21), and what each model offers the page.
 import { NETWORKS, VALLEY_OVERLAYS, type Overlay } from './protocol';
 import type {
   AnasaziInspection,
@@ -19,6 +19,8 @@
   DpdInspection,
   EthnoConfig,
   EthnoInspection,
+  ImageConfig,
+  ImageInspection,
   Inspection,
   ModelConfig,
   ModelKind,
@@ -29,7 +31,7 @@
   TagsInspection,
 } from './types';
 
-export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd'];
+export const MODELS: ModelKind[] = ['sugarscape', 'schelling', 'ring', 'anasazi', 'civil', 'tags', 'spatial', 'culture', 'classes', 'ethno', 'opinions', 'structure', 'dpd', 'image'];
 
 /** The presets menu's group labels. */
 export const MODEL_LABELS: Record<ModelKind, string> = {
@@ -46,12 +48,13 @@
   opinions: 'Bounded Confidence',
   structure: 'Social Structure',
   dpd: 'Demographic PD',
+  image: 'Image Scoring',
 };
 
 /** A config without a `model` key (or with `"sugarscape"`) is a sugarscape config. */
 export function modelOf(c: ModelConfig): ModelKind {
   const tag = (c as { model?: unknown }).model;
-  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd'
+  return tag === 'schelling' || tag === 'ring' || tag === 'anasazi' || tag === 'civil' || tag === 'spatial' || tag === 'tags' || tag === 'culture' || tag === 'classes' || tag === 'ethno' || tag === 'opinions' || tag === 'structure' || tag === 'dpd' || tag === 'image'
     ? tag
     : 'sugarscape';
 }
@@ -62,17 +65,17 @@
 
 /** A sugarscape site's inspection (it lists the site's resources). */
 export function isSugarView(v: AnyInspection): v is Inspection {
-  return 'resources' in v.site;
+  return 'site' in v && 'resources' in v.site;
 }
 
 /** A Ring World site's inspection (it has sugar but no resources list). */
 export function isRingView(v: AnyInspection): v is RingInspection {
-  return 'sugar' in v.site;
+  return 'site' in v && 'sugar' in v.site;
 }
 
 /** A Long House Valley cell's inspection (it names its zone). */
 export function isValleyView(v: AnyInspection): v is AnasaziInspection {
-  return 'zone' in v.site;
+  return 'site' in v && 'zone' in v.site;
 }
 
 /** A civil violence site's inspection (it lists the agents jailed after arrest there). */
@@ -82,7 +85,7 @@
 
 /** A spatial games cell's inspection (it names its z). */
 export function isSpatialView(v: AnyInspection): v is SpatialInspection {
-  return 'z' in v.site;
+  return 'site' in v && 'z' in v.site;
 }
 
 /** A cell of the tags model's diagram (it names its generation). */
@@ -127,6 +130,14 @@
   return model === 'dpd' && (v.agent === null || 'surrounded' in v.agent);
 }
 
+/**
+ * A cell of the image-scoring frame (it names its cell and group; no other model's inspection has a
+ * `cell`). It has no `site`, so the guards above check for one before reading it.
+ */
+export function isImageView(v: AnyInspection): v is ImageInspection {
+  return 'cell' in v && 'group' in v;
+}
+
 /** The calendar year a world of `c` is in at `tick` (the anasazi's), or null for a model without one. */
 export function calendarYear(c: ModelConfig, tick: number): number | null {
   return 'model' in c && c.model === 'anasazi' ? c.start_year + tick : null;
@@ -134,8 +145,8 @@
 
 /**
  * Ticks until a world of `c` at `tick` is finished (the anasazi's end year, the tags model's last
- * generation, the ethnocentrism model's last period, the demographic PD's last cycle); Infinity for
- * a model that never finishes.
+ * generation, the ethnocentrism model's last period, the demographic PD's last cycle, image scoring's
+ * last generation); Infinity for a model that never finishes.
  */
 export function ticksLeft(c: ModelConfig, tick: number): number {
   if ('model' in c && c.model === 'anasazi') return Math.max(0, c.end_year - c.start_year - tick);
@@ -143,6 +154,7 @@
   if (modelOf(c) === 'ethno' && (c as EthnoConfig).end > 0) return Math.max(0, (c as EthnoConfig).end - tick);
   if (modelOf(c) === 'structure' && (c as StructureConfig).stop_at > 0) return Math.max(0, (c as StructureConfig).stop_at - tick);
   if (modelOf(c) === 'dpd' && (c as DpdConfig).end > 0) return Math.max(0, (c as DpdConfig).end - tick);
+  if (modelOf(c) === 'image' && (c as ImageConfig).end > 0) return Math.max(0, (c as ImageConfig).end - tick);
   return Infinity;
 }
 
@@ -252,6 +264,12 @@
     ['wealth', 'Wealth'],
     ['age', 'Age'],
     ['surrounded', 'Surrounded'],
+  ],
+  // The strategies first (k on a blue–red scale, each other class its own color); the core's mode names.
+  image: [
+    ['strategy', 'Strategy'],
+    ['score', 'Score'],
+    ['payoff', 'Payoff'],
   ],
 };
 
@@ -270,4 +288,5 @@
   opinions: [],
   structure: [],
   dpd: [],
+  image: [],
 };
```

```diff
--- a/web/src/engine.ts
+++ b/web/src/engine.ts
@@ -52,7 +52,7 @@
 /** What the page says when a world has run its course (the engine pauses and fires 'finished'). */
 export function finishedNotice(config: ModelConfig, tick: number): string {
   if (modelOf(config) === 'civil') return `A group has died out at t = ${tick} — Reset to run it again`;
-  if (modelOf(config) === 'tags') return `This run has reached its last generation (${tick}) — Reset to run it again`;
+  if (modelOf(config) === 'tags' || modelOf(config) === 'image') return `This run has reached its last generation (${tick}) — Reset to run it again`;
   if (modelOf(config) === 'classes') return `Equity reached at t = ${tick}: every agent remembers mostly M — Reset to run it again`;
   if (modelOf(config) === 'opinions')
     return `Stable at t = ${tick}: no opinion moves any more — Reset, or change confidence or updating, to run it again`;
```

- [ ] **Step 3: Charts and Inspect** (Decisions 26, 27; the Rules panel needs no code: Decision 30)

```diff
--- a/web/src/ui/series-data.ts
+++ b/web/src/ui/series-data.ts
@@ -120,6 +120,12 @@
 
 export const isEthnic = (c: ModelConfig): boolean => 'variant' in c && c.variant === 'ethnic';
 
+/** Whether an image-scoring config allows LH01's binary scorers (who have their own shares chart). */
+const hasBinary = (c: ModelConfig): boolean => 'strategies' in c && (c.strategies as string[]).includes('binary');
+
+/** Whether an image-scoring config allows a class with a k (k, AND, OR, binary). */
+const hasK = (c: ModelConfig): boolean => 'strategies' in c && (c.strategies as string[]).some((s) => s === 'k' || s === 'and' || s === 'or' || s === 'binary');
+
 /**
  * The other models' charts (Decision 13), each a time chart of the model's own series:
  * Schelling's segregation, share unsatisfied, moves and Red share; Ring World's flocks, flock
@@ -129,7 +135,9 @@
  * kills; the spatial games' cooperators, changes, switches and payoffs; the tags model's donation,
  * tolerance, clusters, tags and takeovers; the ethnocentrism model's strategies (in the frame's
  * colors), cooperation, population and kin; the demographic PD's cooperators and defectors (in the
- * frame's colors), cooperator share, surrounded cooperators, mean wealths, and births and deaths.
+ * frame's colors), cooperator share, surrounded cooperators, mean wealths, and births and deaths;
+ * image scoring's help rate and cooperative strategies, mean k, strategy shares (the binary scorers
+ * apart) and mean payoff.
  */
 export const MODEL_CHARTS: Record<Exclude<ModelKind, 'sugarscape'>, ModelChart[]> = {
   schelling: [
@@ -386,14 +394,52 @@
       ],
     },
   ],
+  image: [
+    {
+      title: 'Help rate',
+      lines: [
+        { key: 'help_rate', label: 'Helps ÷ rounds', color: '--c1' },
+        { key: 'cooperative', label: 'Cooperative strategies', color: '--blue' },
+      ],
+      range: [0, 1],
+    },
+    { title: 'Mean k', lines: [{ key: 'mean_k', label: 'Over strategies with a k', color: '--c4' }], range: [-5, 6], shown: hasK },
+    {
+      title: 'Strategy shares',
+      lines: [
+        { key: 'k_cooperative', label: 'k ≤ 0', color: '--blue' },
+        { key: 'k_defective', label: 'k > 0', color: '--red' },
+        { key: 'h', label: 'h (own score)', color: '--c3' },
+        { key: 'own_only', label: 'Own score only', color: '--muted' },
+        { key: 'and', label: 'AND', color: '--c1' },
+        { key: 'or', label: 'OR', color: '--c4' },
+        { key: 'standing', label: 'Standing', color: '--c2' },
+        { key: 'q', label: 'q strategies', color: '--lender' },
+      ],
+      range: [0, 1],
+      shown: (c) => !hasBinary(c),
+    },
+    {
+      title: 'Binary scorers and standing',
+      lines: [
+        { key: 'binary_c', label: 'Cooperators', color: '--blue' },
+        { key: 'binary_x', label: 'Discriminators', color: '--c3' },
+        { key: 'binary_d', label: 'Defectors', color: '--red' },
+        { key: 'standing', label: 'Standing', color: '--c2' },
+      ],
+      range: [0, 1],
+      shown: hasBinary,
+    },
+    { title: 'Mean payoff', lines: [{ key: 'mean_payoff', label: 'Per agent, this generation', color: '--c2' }] },
+  ],
 };
 
 /**
- * A model's time charts count calendar years (the anasazi's), generations (tags), periods
- * (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
+ * A model's time charts count calendar years (the anasazi's), generations (tags, image scoring),
+ * periods (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
  */
 export function timeAxisLabel(model: ModelKind): string {
-  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : 'Tick';
+  return model === 'anasazi' ? 'Year' : model === 'tags' || model === 'image' ? 'Generation' : model === 'culture' ? 'Events per site' : model === 'classes' || model === 'opinions' || model === 'structure' ? 'Periods' : model === 'ethno' ? 'Period' : model === 'dpd' ? 'Cycle' : 'Tick';
 }
 
 /** A calendar-year axis's tick labels: plain years (`1000`, not `1,000`), up to 3 decimals when zoomed in. */
```

Create `web/src/image-scoring.ts`:

```ts
// Image scoring's pure page helpers (milestone 21): Inspect's rows. (`image.ts` is the map import's.)
import type { ImageConfig, ImageInspection } from './types';

/** Whole numbers as they are, others to two decimals. */
const num = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(2));

/** "1 time", "3 times". */
const times = (n: number): string => `${n} ${n === 1 ? 'time' : 'times'}`;

/**
 * A cell's Inspect rows: the cell, its group (with more than one), then the agent of the generation
 * that last played — its strategy and whether it helps at a generation's start, its score (and, with
 * private records, how many others have seen it act and their mean record of it), its standing
 * (only when standing plays), its payoff and the help it gave and received. A gap or a tile's unused
 * cell says so.
 */
export function imageRows(view: ImageInspection, config: Pick<ImageConfig, 'groups' | 'strategies'>): [string, string][] {
  const rows: [string, string][] = [['Cell', `(${view.cell.x}, ${view.cell.y})`]];
  if (view.group === null) return [...rows, ['Agent', 'none (a gap between groups)']];
  if (config.groups > 1) rows.push(['Group', `${view.group + 1} of ${config.groups}`]);
  const a = view.agent;
  if (!a) return [...rows, ['Agent', 'none (an unused cell of the group’s tile)']];
  rows.push(
    ['Agent', `#${a.id}`],
    ['Strategy', `${a.strategy} · ${a.cooperative ? 'helps' : 'refuses'} at a generation’s start`],
    ['Score', String(a.score)],
  );
  if (a.known !== null) {
    const seen = a.known === 0 || a.mean_view === null ? 'nobody yet: the others record it as 0' : `${a.known} ${a.known === 1 ? 'other' : 'others'}, whose mean record is ${num(a.mean_view)}`;
    rows.push(['Seen by', seen]);
  }
  if (config.strategies.includes('standing')) rows.push(['Standing', a.standing ? 'good' : 'bad']);
  rows.push(['Payoff', num(a.payoff)], ['This generation', `helped ${times(a.given)}, was helped ${times(a.received)}`]);
  return rows;
}
```

```diff
--- a/web/src/ui/inspect-panel.ts
+++ b/web/src/ui/inspect-panel.ts
@@ -2,7 +2,8 @@
 import { dpdRows } from '../dpd';
 import type { Engine } from '../engine';
 import { ethnoRows } from '../ethno';
-import { isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
+import { imageRows } from '../image-scoring';
+import { isCivilView, isClassesView, isCultureView, isDpdView, isEthnoView, isImageView, isOpinionsView, isRingView, isStructureView, isSpatialView, isSugarView, isTagsView, isValleyView } from '../models';
 import { playerRows } from '../spatial';
 import type {
   AgentView,
@@ -16,6 +17,8 @@
   DpdInspection,
   EthnoConfig,
   EthnoInspection,
+  ImageConfig,
+  ImageInspection,
   LinkView,
   RingInspection,
   SchellingInspection,
@@ -171,6 +174,16 @@
     return [...rows, ...dpdRows(view.agent).map(([k, v]) => row(k, v))];
   }
 
+  /**
+   * An image-scoring cell and the agent of the generation that last played there. A followed agent
+   * lives one generation (its offspring have new ids), so once it is gone only the cell shows.
+   */
+  private imageCellRows(view: ImageInspection, gone: boolean): HTMLElement[] {
+    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
+    if (gone) return [row('Cell', `(${view.cell.x}, ${view.cell.y})`)];
+    return imageRows(view, this.engine.config as ImageConfig).map(([k, v]) => row(k, v));
+  }
+
   /** A civil site: its cop, the agent shown there (followed into jail), and others jailed after arrest here. */
   private civilRows(view: CivilInspection, followed: number | null, gone: boolean): HTMLElement[] {
     const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
@@ -336,6 +349,12 @@
     // The host tracks a selected agent while it lives (Decision 3).
     const gone = shown.agentId !== null && !shown.alive;
     const view = shown.view;
+    // First: an image-scoring cell has no `site` for the guards below to read.
+    if (isImageView(view)) {
+      const note = gone ? [h('p', { class: 'error' }, `Agent #${shown.agentId}’s generation has passed: each agent lives one generation.`)] : [];
+      this.el.replaceChildren(...note, h('table', {}, ...this.imageCellRows(view, gone)));
+      return;
+    }
     if (!isSugarView(view)) {
       // A Schelling agent that reached its maximum residence has left the landscape; a household
       // dies or leaves the valley; a civil agent dies, is released, or (Model II) is killed.
```

- [ ] **Step 4: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS. (determinism.test's "model charts draw only series their model records" now covers `MODEL_CHARTS.image` against `config_series_names` of `ns-fig-1`.)

```bash
git add web/src/types.ts web/src/models.ts web/src/models.test.ts web/src/engine.ts web/src/engine.test.ts web/src/schema-form.test.ts web/src/sim-host.test.ts web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/image-scoring.ts web/src/image-scoring.test.ts web/src/ui/inspect-panel.ts
git commit -m "Show image scoring on the page: its colors, charts and Inspect" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): every preset of the **Image Scoring** group renders in Strategy, Score and Payoff (islands as a 10 × 10 grid of tiles); the Rules panel's seven groups, Observers per interaction appearing only under Observers; Inspect on an agent (with observers: "Seen by"), on a gap and on a tile's unused cell, and "Agent #N's generation has passed" at the next tick; the charts against "Generation", Mean k hidden for `ns-own-only`, the binary chart for `lh-fig-4*`.

---

### Task 4: Compare, Experiments and the engine-level checks

**Files:**
- Modify: `web/src/compare-presets.ts`, `web/src/experiments/form.ts`
- Test: `web/src/compare-presets.test.ts`, `web/src/experiments/form.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: Task 3's types and `isImageView`; Task 1's presets and `IMAGE_GOLDEN` (Decision 17); `model_schemas_json`, `sweep_points` (WASM).
- Produces: the four Compare entries (`image-one-vs-island`, `image-scoring-vs-standing`, `image-offset`, `image-group-size`); `defaultForm('image')`; the web `IMAGE_GOLDEN` (21 entries).

- [ ] **Step 1: Write the failing tests**

```diff
--- a/web/src/compare-presets.test.ts
+++ b/web/src/compare-presets.test.ts
@@ -110,4 +110,14 @@
       ['dpd-published-vs-closest', 'dpd-run-1', 'dpd-closest', 'Published rule vs closest reading — Demographic PD (Compare)'],
     ]);
   });
+
+  it('pairs the image-scoring runs the sources and readings disagree on', () => {
+    const ids = COMPARE_PRESETS.filter((c) => c.id.startsWith('image-')).map((c) => [c.id, c.a, c.b, c.label]);
+    expect(ids).toEqual([
+      ['image-one-vs-island', 'lh-fig-2a', 'lh-fig-2b', 'One group vs the island model — Image Scoring (Compare)'],
+      ['image-scoring-vs-standing', 'lh-fig-2b', 'lh-fig-4c', 'Image scoring vs standing — Image Scoring (Compare)'],
+      ['image-offset', 'ns-fig-1', 'ns-no-offset', 'With vs without the offset — Image Scoring (Compare)'],
+      ['image-group-size', 'ns-fig-3-n20', 'ns-fig-3-n100', 'Small vs large groups with observers — Image Scoring (Compare)'],
+    ]);
+  });
 });
```

```diff
--- a/web/src/experiments/form.test.ts
+++ b/web/src/experiments/form.test.ts
@@ -194,6 +194,13 @@
       seeds: 3,
       ticks: 500,
       metric: { kind: 'final', series: 'cooperators' },
+    });
+    expect(defaultForm('image')).toMatchObject({
+      x: { path: 'rounds', values: '50:500:50' },
+      series: null,
+      seeds: 3,
+      ticks: 2000,
+      metric: { kind: 'window_mean', series: 'cooperative', from: 1001, to: null },
     });
   });
 
```

```diff
--- a/web/src/determinism.test.ts
+++ b/web/src/determinism.test.ts
@@ -4,7 +4,7 @@
 import { copyWorld, Lockstep } from './compare/lockstep';
 import { defaultForm, formToSweep } from './experiments/form';
 import { Engine, type Speed } from './engine';
-import { isDpdView, isEthnoView, modelOf } from './models';
+import { isDpdView, isEthnoView, isImageView, modelOf } from './models';
 import type { NetworkOverlay } from './protocol';
 import { paramShown } from './schema-form';
 import { SimHost } from './sim-host';
@@ -26,6 +26,9 @@
   EthnoConfig,
   EthnoInspection,
   EthnoStats,
+  ImageConfig,
+  ImageInspection,
+  ImageStats,
   OpinionsConfig,
   OpinionsInspection,
   OpinionsStats,
@@ -474,6 +477,49 @@
   });
 });
 
+describe('image scoring’s golden fingerprints', () => {
+  // crates/sugarscape-core/tests/golden.rs IMAGE_GOLDEN (Decision 17): one group 200 generations,
+  // islands 20. The WASM prints 16 hex digits, so ns-fig-2 and ns-fig-4b keep their leading zero.
+  const IMAGE_GOLDEN: [string, number, string][] = [
+    ['ns-fig-1', 200, '0x98875bd71738cf05'],
+    ['ns-fig-2', 200, '0x091995405bae5fd9'],
+    ['ns-fig-3-n20', 200, '0xb4bd11bc229673c4'],
+    ['ns-fig-3-n50', 200, '0x5907ce5cb47602cf'],
+    ['ns-fig-3-n100', 200, '0xaf771beff4611d0d'],
+    ['ns-fig-4a', 200, '0xe7d6cefc18bd8d6a'],
+    ['ns-fig-4b', 200, '0x0a4c19c5fa5fcfe3'],
+    ['ns-fig-4c', 200, '0x64755eafb526a816'],
+    ['ns-fig-4d', 200, '0x72ca0b1d80947f44'],
+    ['ns-own-only', 200, '0x20d6768b1fbd7081'],
+    ['ns-no-offset', 200, '0xc595349de0c0bf65'],
+    ['lh-fig-1a', 20, '0x46669de796924497'],
+    ['lh-fig-1b', 20, '0x2c3d0dea6af0c894'],
+    ['lh-fig-2a', 200, '0x56250416634c6ed9'],
+    ['lh-fig-2b', 20, '0x566a697764e9cdd9'],
+    ['lh-fig-2c', 20, '0x963ec9f09d2fadbe'],
+    ['lh-fig-3a', 20, '0x46059fbe6bab2056'],
+    ['lh-fig-3b', 20, '0x4d6575c59b0da89a'],
+    ['lh-fig-4a', 20, '0xd5654fa4b600eb57'],
+    ['lh-fig-4b', 20, '0x4bfe8ea9f3e8598b'],
+    ['lh-fig-4c', 20, '0x6371653b544f6387'],
+  ];
+
+  it('covers every image-scoring preset', () => {
+    expect(IMAGE_GOLDEN.map(([id]) => id)).toEqual(presets.filter((p) => modelOf(p.config) === 'image').map((p) => p.id));
+  });
+
+  it.each(IMAGE_GOLDEN)('%s reproduces its golden fingerprint after %i generations, whatever is watched', async (id, ticks, golden) => {
+    const preset = presets.find((p) => p.id === id)!;
+    const e = await Engine.create({ config: structuredClone(preset.config), seed: 1 }, { presets, transport: inline() });
+    e.want(() => ({ charts: { groups: [['help_rate', 'cooperative']], max: 50 }, lorenz: true, networks: ['trade'] }));
+    e.setDisplay({ colorMode: 'score' });
+    await e.select(0, 0);
+    for (const n of [1, ticks / 2 - 1, ticks / 2]) await e.advance(n);
+    expect(e.tick).toBe(ticks);
+    expect(await e.fingerprint()).toBe(golden);
+  });
+});
+
 describe('model charts', () => {
   it('draw only series their model records', () => {
     for (const [model, charts] of Object.entries(MODEL_CHARTS)) {
@@ -903,6 +949,105 @@
   });
 });
 
+describe('image scoring through the engine', () => {
+  const preset = (id: string) => presets.find((p) => p.id === id)!;
+  const imagePresets = presets.filter((p) => modelOf(p.config) === 'image');
+  const create = (id: string, edit: (c: ImageConfig) => void = () => {}) => {
+    const config = structuredClone(preset(id).config) as ImageConfig;
+    edit(config);
+    return Engine.create({ config, seed: 1 }, { presets, transport: inline() });
+  };
+  const schema = (JSON.parse(model_schemas_json()) as Record<string, Param[]>).image;
+  const field = (path: string) => schema.find((p) => p.path === path)!;
+
+  it('builds a Rules panel in the spec’s groups, with observers only under observers and every preset on its sliders', () => {
+    expect([...new Set(schema.map((p) => p.group))]).toEqual(['Game', 'Population', 'Rounds', 'Information', 'Errors', 'Evolution', 'Run']);
+    // Presets, files and links set the strategies and the start (as ethnocentrism's allowed strategies).
+    expect(schema.map((p) => p.path)).not.toContain('strategies');
+    expect(schema.map((p) => p.path)).not.toContain('initial');
+    expect(field('observers').show_if).toEqual({ path: 'information', equals: 'observers' });
+    expect([paramShown(field('observers'), preset('ns-fig-1').config), paramShown(field('observers'), preset('ns-fig-3-n20').config)]).toEqual([false, true]);
+    expect(imagePresets).toHaveLength(21);
+    for (const p of imagePresets) {
+      for (const f of schema.filter((f) => f.kind === 'number' || f.kind === 'integer')) {
+        const v = (p.config as unknown as Record<string, number>)[f.path];
+        expect(v, `${p.id}: ${f.path}`).toBeGreaterThanOrEqual(f.min!);
+        expect(v, `${p.id}: ${f.path}`).toBeLessThanOrEqual(f.max!);
+      }
+    }
+  });
+
+  it('takes live edits of the cost, the records and the observers, and replays them through keyframes and a share link', async () => {
+    const e = await create('ns-fig-3-n20');
+    await e.advance(30);
+    expect(await e.applyModelConfig((c) => void ((c as ImageConfig).c = 0.2))).toBeNull();
+    await e.advance(30);
+    expect(await e.applyModelConfig((c) => void ((c as ImageConfig).records = 'score'))).toBeNull();
+    expect(await e.applyModelConfig((c) => void ((c as ImageConfig).observers = 5))).toBeNull();
+    await e.advance(20);
+    const want = await e.fingerprint();
+    expect(e.tick).toBe(80);
+    await e.seek(20);
+    await e.seek(80);
+    expect(await e.fingerprint()).toBe(want);
+    const { session, tick } = await e.session();
+    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
+    await opened.advance(tick);
+    expect([(opened.config as ImageConfig).records, (opened.config as ImageConfig).observers]).toEqual(['score', 5]);
+    expect(await opened.fingerprint()).toBe(want);
+    const errors = await e.applyModelConfig((c) => void ((c as ImageConfig).mutation = 2));
+    expect(errors?.map((f) => f.field)).toEqual(['mutation']);
+  });
+
+  it('stops at its last generation, once, and inspects agents, a tile’s unused cell and a gap', async () => {
+    // Two groups of 20 with observers: tiles of 5 × 5 side by side, a one-cell gap at x = 5.
+    const e = await create('ns-fig-3-n20', (c) => {
+      c.groups = 2;
+      c.end = 30;
+    });
+    let ends = 0;
+    e.on('finished', () => ends++);
+    await e.advance(100);
+    expect([e.tick, e.finished, ends]).toEqual([30, true, 1]);
+    const at = async (x: number, y: number): Promise<ImageInspection> => {
+      await e.select(x, y);
+      const v = e.inspection!.view;
+      expect(isImageView(v)).toBe(true);
+      return v as ImageInspection;
+    };
+    expect(await at(5, 0)).toEqual({ cell: { x: 5, y: 0 }, group: null, agent: null });
+    expect(await at(0, 4)).toEqual({ cell: { x: 0, y: 4 }, group: 0, agent: null });
+    let given = 0;
+    let received = 0;
+    for (const x0 of [0, 6]) {
+      for (let i = 0; i < 20; i++) {
+        const v = await at(x0 + (i % 5), Math.floor(i / 5));
+        const a = v.agent!;
+        expect(v.group).toBe(x0 === 0 ? 0 : 1);
+        expect(e.inspection!.agentId).toBe(a.id);
+        // With observers each agent's record among the others is known.
+        expect(a.known).toBeGreaterThanOrEqual(0);
+        expect(a.known).toBeLessThanOrEqual(19);
+        given += a.given;
+        received += a.received;
+      }
+    }
+    const stats = e.latest as ImageStats;
+    expect([given, received]).toEqual([stats.helps, stats.helps]);
+  });
+
+  it('opens its Compare entries and sweeps its default form', () => {
+    for (const id of ['image-one-vs-island', 'image-scoring-vs-standing', 'image-offset', 'image-group-size']) {
+      const entry = COMPARE_PRESETS.find((c) => c.id === id)!;
+      expect(comparePresetStates(presets, entry, 1), id).not.toBeNull();
+    }
+    const { sweep, errors } = formToSweep(defaultForm('image'), { preset: 'ns-fig-2' });
+    expect(errors).toEqual([]);
+    // Ten values of m × the form's three seeds.
+    expect(JSON.parse(sweep_points(JSON.stringify(sweep)))).toHaveLength(30);
+  });
+});
+
 describe('civil violence through the engine', () => {
   it('stops run 7 when a group is gone, once', async () => {
     const run7 = presets.find((p) => p.id === 'cv-run-7-cleansing')!;
```

Run: `(cd web && npm run build && npm test)` — Expected: FAIL (the Compare entries, `defaultForm('image')`, and the engine test's Compare/sweep case; the golden fingerprints, Rules panel, live-edit and Inspect tests already pass on Task 3's code).

- [ ] **Step 2: Implement** (Decisions 24, 25)

```diff
--- a/web/src/compare-presets.ts
+++ b/web/src/compare-presets.ts
@@ -116,6 +116,30 @@
     a: 'dpd-run-1',
     b: 'dpd-closest',
   },
+  {
+    id: 'image-one-vs-island',
+    label: 'One group vs the island model — Image Scoring (Compare)',
+    a: 'lh-fig-2a',
+    b: 'lh-fig-2b',
+  },
+  {
+    id: 'image-scoring-vs-standing',
+    label: 'Image scoring vs standing — Image Scoring (Compare)',
+    a: 'lh-fig-2b',
+    b: 'lh-fig-4c',
+  },
+  {
+    id: 'image-offset',
+    label: 'With vs without the offset — Image Scoring (Compare)',
+    a: 'ns-fig-1',
+    b: 'ns-no-offset',
+  },
+  {
+    id: 'image-group-size',
+    label: 'Small vs large groups with observers — Image Scoring (Compare)',
+    a: 'ns-fig-3-n20',
+    b: 'ns-fig-3-n100',
+  },
 ];
 
 /**
```

```diff
--- a/web/src/experiments/form.ts
+++ b/web/src/experiments/form.ts
@@ -87,6 +87,15 @@
   if (model === 'dpd') {
     // Table 9.3's axis at T = 6: cooperators after 500 cycles against the reward R (R = 1 dies out).
     return { ...form, x: { path: 'r', values: '1:5:1' }, ticks: 500, metric: { ...form.metric, kind: 'final', series: 'cooperators' } };
+  }
+  if (model === 'image') {
+    // NS98's rounds axis (the built-in ns-rounds, shortened): cooperative strategies against m.
+    return {
+      ...form,
+      x: { path: 'rounds', values: '50:500:50' },
+      ticks: 2000,
+      metric: { ...form.metric, kind: 'window_mean', series: 'cooperative', from: 1001, to: null },
+    };
   }
   return form;
 }
```

- [ ] **Step 3: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS (46 files, 638 tests): the 21 image fingerprints through the engine (island presets at 20 generations), the Rules panel's schema (seven groups, observers only under observers, every preset on its sliders), live edits of the cost, the records and the observers through keyframes and a share link, a bad live mutation named on its field, the stop at the last generation with Inspect on agents, a gap and a tile's unused cell (helps given and received summing to the generation's `helps`), the four Compare entries resolving, and the default sweep's 30 points.

```bash
git add web/src/compare-presets.ts web/src/compare-presets.test.ts web/src/experiments/form.ts web/src/experiments/form.test.ts web/src/determinism.test.ts
git commit -m "Pair the image-scoring runs the sources disagree on in Compare, and sweep the rounds by default" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): the four Compare entries open side by side (one group's 10 × 10 frame beside the islands' 109 × 109); Experiments from an image world offers rounds 50–500 and runs; keyframes and the timeline restore an `ns-fig-3-n20` run with a live records change exactly; Max speed on `lh-fig-2b` (g 100 × n 100).

---

### Task 5: The survey's image-scoring claims

**Files:**
- Create: `survey/src/claims/image.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `crate::runner::{model_after, model_preset}`, `crate::claim::{all_of, equivalent, greater, range, untestable, Claim, Outcome, Source}`; `sugarscape_core::image::{ImageWorld, ImageConfig, Strategy, …}` (`step`, `agents`, `stats.series`, `tick`) and `image::analytic`; `ModelWorld::Image`.
- Produces: 34 claims (`ns-…`, `lh-…`), Decision 28.

- [ ] **Step 1: The claims and their unit tests**

Create `survey/src/claims/image.rs`:

```rust
//! Image scoring (milestone 21): Nowak & Sigmund 1998 (NS98, its figures
//! and Methods) and Leimar & Hammerstein 2001 (LH01), each claim in its
//! source's words, and our own switches (the offset, FAIR23's visibility,
//! how an observer records). NS98 averaged over 10⁷ generations and LH01
//! over 10⁵–10⁶; these runs are shorter, each claim says how long, and use
//! their own seeds (seeds 1–10 unless they say otherwise, whatever `--seeds`
//! says), so their numbers are the plan's measurements. A source's
//! percentage is judged by a one-sample equivalence test of our seeds'
//! window means against it (`equivalent` against the constant), with a
//! margin of 5 points: the most our own estimates move between these
//! windows and the measurements' longer ones (NS98 Fig. 3 at n = 20: 0.863
//! to 0.912; LH01 Fig. 3a: 0.522 to 0.468). Runs that several claims share
//! are memoized per process.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sugarscape_core::image::analytic::{self, Binary, Start};
use sugarscape_core::image::{
    Class, ImageConfig, ImageWorld, Initial, Offset, Records, Seeded, Strategy,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, range, untestable, Claim, Outcome, Source};
use crate::runner::{model_after, model_preset};

const NS98: &str = "Nowak & Sigmund, Nature 393 (1998) 573–577";
const NS98_METHODS: &str = "Nowak & Sigmund, Nature 393 (1998), Methods";
const LH01: &str = "Leimar & Hammerstein, Proc. R. Soc. B 268 (2001) 745–753";
const OURS: &str = "spec 2026-09-26-image-scoring-design.md; plan Decisions 4, 16 and 22";

/// The equivalence margin for a source's share (see the module comment).
const MARGIN: f64 = 0.05;

fn image(w: &ModelWorld) -> &ImageWorld {
    match w {
        ModelWorld::Image(w) => w,
        _ => unreachable!("an image-scoring world"),
    }
}

fn preset(id: &str, edit: impl FnOnce(&mut ImageConfig)) -> ImageConfig {
    let ModelConfig::Image(mut c) = model_preset(id) else {
        panic!("{id} is not an image-scoring preset")
    };
    edit(&mut c);
    c
}

fn seeds(n: u64) -> Vec<u64> {
    (1..=n).collect()
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// Each seed's `f` of a fresh world of `c`, stepped by `f` itself.
fn each<T: Send>(c: &ImageConfig, seeds: &[u64], f: impl Fn(ImageWorld) -> T + Sync) -> Vec<T> {
    model_after(&ModelConfig::Image(c.clone()), seeds, 0, |w| {
        f(image(w).clone())
    })
}

/// Each seed's mean of each of `names` over generations `from..=ticks` (memoized).
fn windows(c: &ImageConfig, names: &[&str], n: u64, ticks: u32, from: usize) -> Arc<Vec<Vec<f64>>> {
    type Cache = Mutex<Vec<(String, u32, usize, Arc<Vec<Vec<f64>>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let key = format!(
        "{}|{names:?}|{n}",
        serde_json::to_string(c).expect("configs serialize")
    );
    if let Some((.., v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, t, f, _)| *k == key && *t == ticks && *f == from)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(
        &ModelConfig::Image(c.clone()),
        &seeds(n),
        ticks,
        |w| {
            let w = image(w);
            names
                .iter()
                .map(|s| mean(&w.stats.series(s).expect("an image series")[from..]))
                .collect()
        },
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, ticks, from, v.clone()));
    v
}

/// Each seed's mean of `name` over generations `from..=ticks`.
fn window(c: &ImageConfig, name: &str, n: u64, ticks: u32, from: usize) -> Vec<f64> {
    windows(c, &[name], n, ticks, from)
        .iter()
        .map(|v| v[0])
        .collect()
}

/// Our seeds against a source's share: equivalent within `MARGIN`.
fn about(ours: &[f64], source: f64, who: &str) -> Outcome {
    equivalent(ours, &vec![source; ours.len()], Some(MARGIN), "ours", who)
}

fn share(w: &ImageWorld, f: impl Fn(Strategy) -> bool) -> f64 {
    let a = w.agents();
    a.iter().filter(|x| f(x.strategy)).count() as f64 / a.len() as f64
}

/// 1 where `ok`, else 0.
fn ind(ok: bool) -> f64 {
    f64::from(u8::from(ok))
}

/// Seeds 1–100 of `c`, each until one strategy is fixed (at most 5,000
/// generations): the fixed strategy and its generation (memoized).
fn fixation(c: &ImageConfig) -> Arc<Vec<Option<(Strategy, u64)>>> {
    type Cache = Mutex<Vec<(String, Arc<Vec<Option<(Strategy, u64)>>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let key = serde_json::to_string(c).expect("configs serialize");
    if let Some((_, v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, _)| *k == key)
    {
        return v.clone();
    }
    let v = Arc::new(each(c, &seeds(100), |mut w| {
        for _ in 0..5000 {
            w.step();
            let a = w.agents();
            if a.iter().all(|x| x.strategy == a[0].strategy) {
                return Some((a[0].strategy, w.tick));
            }
        }
        None
    }));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, v.clone()));
    v
}

/// Per seed, 1 where some k ≤ 0 was fixed (cooperation won), else 0.
fn cooperation_won(c: &ImageConfig) -> Vec<f64> {
    fixation(c)
        .iter()
        .map(|r| ind(r.is_some_and(|(s, _)| s.cooperative())))
        .collect()
}

/// "k = 0 fixed in 20/100 (median generation 56); some k ≤ 0 in 40/100".
fn fixation_text(c: &ImageConfig) -> String {
    let r = fixation(c);
    let mut t0: Vec<u64> = r
        .iter()
        .flatten()
        .filter(|(s, _)| *s == Strategy::K(0))
        .map(|p| p.1)
        .collect();
    t0.sort_unstable();
    let coop = r.iter().flatten().filter(|(s, _)| s.cooperative()).count();
    format!(
        "k = 0 fixed in {}/100 (median generation {}); some k ≤ 0 in {coop}/100",
        t0.len(),
        t0.get(t0.len() / 2).copied().unwrap_or(0)
    )
}

/// Per seed of Fig. 2 over 10⁵ generations: collapses (the share of k ≤ 0
/// falling from ≥ 0.9 to ≤ 0.1), recoveries, the mean share of k ≤ −4 in
/// cooperative generations (≥ 0.9) and over the 51 generations up to each
/// collapse's last cooperative one (memoized).
fn cycles() -> Arc<Vec<(f64, f64, f64, f64)>> {
    static CACHE: Mutex<Option<Arc<Vec<(f64, f64, f64, f64)>>>> = Mutex::new(None);
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(v) = cache.as_ref() {
        return v.clone();
    }
    let v = Arc::new(each(&preset("ns-fig-2", |_| {}), &seeds(10), |mut w| {
        let (mut coop, mut unc) = (Vec::new(), Vec::new());
        for _ in 0..100_000 {
            w.step();
            coop.push(share(&w, |s| s.k().is_some_and(|k| k <= 0)));
            unc.push(share(&w, |s| s.k().is_some_and(|k| k <= -4)));
        }
        let (mut state, mut last_hi, mut collapses, mut recoveries) = (0, 0, Vec::new(), 0);
        for (t, &x) in coop.iter().enumerate() {
            if x >= 0.9 {
                recoveries += usize::from(state == -1);
                (state, last_hi) = (1, t);
            } else if x <= 0.1 {
                if state == 1 {
                    collapses.push(last_hi);
                }
                state = -1;
            }
        }
        let base: Vec<f64> = (0..coop.len())
            .filter(|&t| coop[t] >= 0.9)
            .map(|t| unc[t])
            .collect();
        let before: Vec<f64> = collapses
            .iter()
            .map(|&t| mean(&unc[t.saturating_sub(50)..=t]))
            .collect();
        (
            collapses.len() as f64,
            recoveries as f64,
            mean(&base),
            mean(&before),
        )
    }));
    *cache = Some(v.clone());
    v
}

/// Fig. 4: each seed's help rate over generations 1,001–50,000, its most
/// frequent strategy over them, and the three most frequent pooled over
/// the seeds with their shares.
struct Fig4 {
    help: Vec<f64>,
    top: Vec<Strategy>,
    pooled: Vec<(Strategy, f64)>,
}

fn fig_4(id: &str) -> Fig4 {
    let r = each(&preset(id, |_| {}), &seeds(10), |mut w| {
        let mut counts = HashMap::new();
        for t in 1..=50_000u32 {
            w.step();
            if t > 1000 {
                for a in w.agents() {
                    *counts.entry(a.strategy).or_insert(0u64) += 1;
                }
            }
        }
        (
            counts,
            mean(&w.stats.series("help_rate").expect("an image series")[1001..]),
        )
    });
    let top = |counts: &HashMap<Strategy, u64>| -> Vec<(Strategy, f64)> {
        let total: u64 = counts.values().sum();
        let mut v: Vec<(Strategy, f64)> = counts
            .iter()
            .map(|(s, n)| (*s, *n as f64 / total as f64))
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.code().cmp(&b.0.code())));
        v.truncate(3);
        v
    };
    let mut all = HashMap::new();
    for (c, _) in &r {
        for (s, n) in c {
            *all.entry(*s).or_insert(0u64) += n;
        }
    }
    Fig4 {
        help: r.iter().map(|p| p.1).collect(),
        top: r.iter().map(|p| top(&p.0)[0].0).collect(),
        pooled: top(&all),
    }
}

fn pooled_text(v: &[(Strategy, f64)]) -> String {
    v.iter()
        .map(|(s, x)| format!("{} {:.1}%", s.label(), 100.0 * x))
        .collect::<Vec<_>>()
        .join("; ")
}

/// A Fig. 4 claim: the help rate against NS98's, and each seed's most
/// frequent strategy against NS98's.
fn fig_4_claim(id: &str, help: f64, first: Strategy) -> Outcome {
    let f = fig_4(id);
    let firsts: Vec<f64> = f.top.iter().map(|s| ind(*s == first)).collect();
    all_of(vec![
        ("help rate".into(), about(&f.help, help, "NS98")),
        (
            format!("most frequent {}", first.label()),
            range(&firsts, 1.0, 1.0, false),
        ),
    ])
    .with(&format!(
        "Help rate {:.4} (seeds 1–10, generations 1,001–50,000); most frequent over them: {}.",
        mean(&f.help),
        pooled_text(&f.pooled)
    ))
}

/// Each seed's share of strategies matching `is` at each generation of `at`.
fn invasion(
    c: &ImageConfig,
    n: u64,
    at: &[u64],
    is: impl Fn(Strategy) -> bool + Sync,
) -> Vec<Vec<f64>> {
    each(c, &seeds(n), |mut w| {
        at.iter()
            .map(|&t| {
                while w.tick < t {
                    w.step();
                }
                share(&w, &is)
            })
            .collect()
    })
}

fn column(v: &[Vec<f64>], i: usize) -> Vec<f64> {
    v.iter().map(|r| r[i]).collect()
}

/// NS98's Methods game in the simulation: binary discriminators (k 0)
/// against defectors (k 1) at share `x`, no offset, perfect information,
/// n = 100, m = 250 (five of the Methods' rounds, in which everyone plays
/// once, half as donor): each seed's payoff gap after one generation.
fn methods_gap(x: f64, n: u64) -> Vec<f64> {
    let c = ImageConfig {
        rounds: 250,
        offset: Offset::None,
        strategies: vec![Class::Binary],
        initial: Initial::Seeded(Seeded {
            only: Strategy::Binary(1),
            invader: Some(Strategy::Binary(0)),
            share: x,
        }),
        ..Default::default()
    };
    each(&c, &seeds(n), |mut w| {
        w.step();
        let (mut d, mut e) = (Vec::new(), Vec::new());
        for a in w.agents() {
            if a.strategy == Strategy::Binary(0) {
                d.push(a.payoff);
            } else {
                e.push(a.payoff);
            }
        }
        mean(&d) - mean(&e)
    })
}

/// Fig. 3's cooperative share at group size `n` (seeds 1–10, generations
/// 1,001–20,000).
fn fig_3(n: u32, edit: impl FnOnce(&mut ImageConfig)) -> Vec<f64> {
    let id = match n {
        20 => "ns-fig-3-n20",
        50 => "ns-fig-3-n50",
        _ => "ns-fig-3-n100",
    };
    window(&preset(id, edit), "cooperative", 10, 20_000, 1001)
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ns-fig-1.cooperation-wins",
            item: "ns-fig-1",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 1 (n 100, m 125, k −5 … +6 at random): the discriminating strategy k = 0 is fixed (after 166 generations in the run shown) — cooperation wins (some k ≤ 0 fixed in each run)",
            check: |_| {
                let c = preset("ns-fig-1", |_| {});
                range(&cooperation_won(&c), 1.0, 1.0, false).with(&format!(
                    "Seeds 1–100, each to fixation (at most 5,000 generations; all fixed): {}. k = 0 is the most common single winner, but defection (k ≥ 1) wins the rest.",
                    fixation_text(&c)
                ))
            },
        },
        Claim {
            id: "ns-fig-1.more-rounds",
            item: "ns-fig-1",
            source: Source::Book,
            citation: NS98,
            text: "Cooperation is more likely to win the greater the number m of interactions per generation (runs won by some k ≤ 0 at m = 300 against m = 125)",
            check: |_| {
                let more = preset("ns-fig-1", |c| c.rounds = 300);
                let base = preset("ns-fig-1", |_| {});
                greater(&cooperation_won(&more), &cooperation_won(&base), "m 300", "m 125")
                    .with(&format!("m = 300: {}.", fixation_text(&more)))
            },
        },
        Claim {
            id: "ns-fig-2.cycles",
            item: "ns-fig-2",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 2 (m 300, mutation 0.001): endless cycles of cooperation and defection (each run's collapses — the share of k ≤ 0 falling from at least 90% to at most 10% — over 10⁵ generations, at least one)",
            check: |_| {
                let v = cycles();
                let collapses: Vec<f64> = v.iter().map(|r| r.0).collect();
                let recoveries: f64 = v.iter().map(|r| r.1).sum();
                range(&collapses, 1.0, f64::INFINITY, false).with(&format!(
                    "Seeds 1–10 × 10⁵ generations: {} collapses and {recoveries} recoveries (per seed {}–{}).",
                    collapses.iter().sum::<f64>(),
                    collapses.iter().copied().fold(f64::INFINITY, f64::min),
                    collapses.iter().copied().fold(0.0, f64::max)
                ))
            },
        },
        Claim {
            id: "ns-fig-2.cooperators-first",
            item: "ns-fig-2",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 2: unconditional cooperators (k −4, −5) spread in a cooperative population before defectors invade (each seed's share of k ≤ −4 over the 51 generations up to a collapse, against its share in cooperative generations)",
            check: |_| {
                let v = cycles();
                let before: Vec<f64> = v.iter().map(|r| r.3).collect();
                let base: Vec<f64> = v.iter().map(|r| r.2).collect();
                greater(&before, &base, "before collapses", "cooperative generations")
            },
        },
        Claim {
            id: "ns-fig-3-n20.cooperative",
            item: "ns-fig-3-n20",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3 (ten observers per interaction, m = 10n, mutation 0.001): cooperative strategies (k ≤ 0) 90% of the time at n = 20",
            check: |_| {
                about(&fig_3(20, |_| {}), 0.90, "NS98")
                    .with("Seeds 1–10, generations 1,001–20,000 (NS98: 10⁷); to 100,000 our mean is 0.912 ± 0.056.")
            },
        },
        Claim {
            id: "ns-fig-3-n50.cooperative",
            item: "ns-fig-3-n50",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3: cooperative strategies 47% of the time at n = 50",
            check: |_| {
                about(&fig_3(50, |_| {}), 0.47, "NS98")
                    .with("Seeds 1–10, generations 1,001–20,000; the seeds spread widely (to 50,000: 0.459 ± 0.131).")
            },
        },
        Claim {
            id: "ns-fig-3-n100.cooperative",
            item: "ns-fig-3-n100",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3: cooperative strategies 18% of the time at n = 100",
            check: |_| {
                about(&fig_3(100, |_| {}), 0.18, "NS98")
                    .with("Seeds 1–10, generations 1,001–20,000 (to 30,000: 0.215 ± 0.131).")
            },
        },
        Claim {
            id: "ns-fig-3.falls-with-n",
            item: "ns-fig-3",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 3: with a fixed number of observers, cooperation falls as the group grows (n = 20 above n = 50 above n = 100)",
            check: |_| {
                let (a, b, c) = (fig_3(20, |_| {}), fig_3(50, |_| {}), fig_3(100, |_| {}));
                all_of(vec![
                    ("20 > 50".into(), greater(&a, &b, "n 20", "n 50")),
                    ("50 > 100".into(), greater(&b, &c, "n 50", "n 100")),
                ])
            },
        },
        Claim {
            id: "ns-fig-4a.help-and-mode",
            item: "ns-fig-4a",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4a (AND strategies, perfect information, m 500): 55% of interactions cooperative; (k 0, h 1) the most frequent strategy",
            check: |_| fig_4_claim("ns-fig-4a", 0.55, Strategy::And { k: 0, h: 1 }),
        },
        Claim {
            id: "ns-fig-4b.help-and-mode",
            item: "ns-fig-4b",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4b (AND, observers, n 20, m 200): 57% cooperative; (k 0, h 4) the most frequent",
            check: |_| fig_4_claim("ns-fig-4b", 0.57, Strategy::And { k: 0, h: 4 }),
        },
        Claim {
            id: "ns-fig-4c.help-and-mode",
            item: "ns-fig-4c",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4c (OR strategies, perfect information, m 500): 70% cooperative; the defectors (k 6, h −5) the most frequent single strategy",
            check: |_| fig_4_claim("ns-fig-4c", 0.70, Strategy::Or { k: 6, h: -5 }),
        },
        Claim {
            id: "ns-fig-4d.help-and-mode",
            item: "ns-fig-4d",
            source: Source::Book,
            citation: NS98,
            text: "Fig. 4d (OR, observers, n 20, m 200): 80% cooperative; the defectors (k 6, h −5) the most frequent",
            check: |_| fig_4_claim("ns-fig-4d", 0.80, Strategy::Or { k: 6, h: -5 }),
        },
        Claim {
            id: "ns-own-only.help",
            item: "ns-own-only",
            source: Source::Book,
            citation: NS98,
            text: "Strategies that only consider their own image give less than 0.1% cooperation (each seed's help rate below 0.001)",
            check: |_| {
                let c = preset("ns-own-only", |_| {});
                range(&window(&c, "help_rate", 10, 20_000, 1001), 0.0, 0.001, false).with(
                    "Seeds 1–10, generations 1,001–20,000. The floor is mutation: mutants are uniform over h −5 … +6, and the 5 of 12 with h ≥ 1 help at a generation's start.",
                )
            },
        },
        Claim {
            id: "ns-rounds.two-interactions",
            item: "ns-rounds",
            source: Source::Book,
            citation: NS98,
            text: "It suffices that each player is chosen only for about 2 interactions per life-time (Fig. 2's settings at m = n = 100: cooperative strategies prevail, above half the time in each seed)",
            check: |_| {
                let at = |m: u32| window(&preset("ns-fig-2", |c| c.rounds = m), "cooperative", 10, 20_000, 1001);
                range(&at(100), 0.5, 1.0, false).with(&format!(
                    "Seeds 1–10, generations 1,001–20,000: cooperative strategies {:.3} at m = 100 and {:.3} at m = 200 (four interactions per lifetime); the sweep ns-rounds has the whole curve.",
                    mean(&at(100)),
                    mean(&at(200))
                ))
            },
        },
        Claim {
            id: "ns-methods.rounds",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Discriminators are stable against defectors if the mean number of rounds 1/(1 − w) exceeds (bq + c)/(bq − c): about 1.2 rounds at b = 1, c = 0.1, q = 1 (stability, from the w-weighted payoff sums, at mean rounds 1.00 … 2.00 in steps of 0.01, agreeing with 1.2 away from 1.20–1.25)",
            check: |_| {
                let m = Binary::ns98();
                let agree: Vec<f64> = (100..=200)
                    .map(|i| f64::from(i) / 100.0)
                    .filter(|r| !(1.2..=1.25).contains(r))
                    .map(|r| ind(m.stable(1.0 - 1.0 / r) == (r > 1.2)))
                    .collect();
                range(&agree, 1.0, 1.0, false).with(&format!(
                    "The exact threshold (bq + c)/(bq − c) = {:.4}.",
                    m.min_rounds()
                ))
            },
        },
        Claim {
            id: "ns-methods.q",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Discriminators can be stable only if q > c/b (at b = 1, c = 0.1, q = 0.01 … 1: some continuation w < 1 stabilizes them exactly when q > 0.1)",
            check: |_| {
                let agree: Vec<f64> = (1..=100)
                    .map(|i| {
                        let m = Binary {
                            q: f64::from(i) / 100.0,
                            ..Binary::ns98()
                        };
                        let some_w = (1..1000).any(|j| m.stable(f64::from(j) / 1000.0));
                        ind(some_w == (m.q > 0.1 + 1e-12))
                    })
                    .collect();
                range(&agree, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "ns-methods.cooperators",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "With unconditional cooperators, defectors win below the discriminator frequency x = c(2 − w)/(bwq) and cooperators do better above it (Dc − De changes sign there, at w = 0.5 … 0.95)",
            check: |_| {
                let m = Binary::ns98();
                let agree: Vec<f64> = (10..=19)
                    .map(|i| {
                        let w = f64::from(i) / 20.0;
                        let x = m.cooperator_equilibrium(w);
                        let d = |x: f64| m.cooperators_over_defectors(w, x);
                        ind(d(x).abs() < 1e-12 && d(x - 0.01) < 0.0 && d(x + 0.01) > 0.0)
                    })
                    .collect();
                range(&agree, 1.0, 1.0, false).with(&format!(
                    "At w = 0.9: x = {:.4}.",
                    m.cooperator_equilibrium(0.9)
                ))
            },
        },
        Claim {
            id: "ns-methods.threshold-in-simulation",
            item: "ns-methods",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Discriminators do better than defectors once their frequency exceeds x_min (0.123 over five rounds): in the simulation at x = 0.14, discriminators' mean payoff above defectors' (one generation, n 100, m 250, no offset, seeds 1–2,000)",
            check: |_| {
                let gap = methods_gap(0.14, 2000);
                let zero = vec![0.0; gap.len()];
                greater(&gap, &zero, "discriminators − defectors", "0").with(&format!(
                    "Analytic x_min over five rounds {:.4}; the simulated mean gap {:+.4} at 0.14, {:+.4} at 0.20. The Methods' rounds (everyone plays once, half as donor) are not NS98's random pairs, and the simulated threshold is higher (the first positive gap at 0.16 in 0.01 steps).",
                    Binary::ns98().x_min_fixed(5).unwrap_or(f64::NAN),
                    mean(&gap),
                    mean(&methods_gap(0.20, 2000))
                ))
            },
        },
        Claim {
            id: "ns-universal.constant",
            item: "ns-universal",
            source: Source::Book,
            citation: NS98_METHODS,
            text: "Everyone k = 0 with unbounded scores: the maximum initial fraction below 0 that still converges to all-out cooperation is 0.7380294688360… (a universal constant; the start is not stated) — to every printed digit with the negatives at −1 and the rest never falling below 0, and within 10⁻¹⁰ with the rest at +100 … +300",
            check: |_| {
                let rests = [None, Some(100), Some(150), Some(200), Some(300)];
                let t: Vec<f64> = rests
                    .iter()
                    .map(|&rest| analytic::universal_threshold(Start::AtMinusOne { rest }))
                    .collect();
                let others = [
                    ("rest at 0", Start::AtMinusOne { rest: Some(0) }),
                    ("rest at +1", Start::AtMinusOne { rest: Some(1) }),
                    ("rest at +2", Start::AtMinusOne { rest: Some(2) }),
                    ("negatives over −1 … −5", Start::Spread { below: 5 }),
                ];
                let others = others
                    .iter()
                    .map(|(name, s)| format!("{name} {:.4}", analytic::universal_threshold(*s)))
                    .collect::<Vec<_>>()
                    .join(", ");
                all_of(vec![
                    ("out of reach".into(), range(&[t[0]; 5], 0.738_029_468_836_0, 0.738_029_468_836_1, false)),
                    ("+100 … +300".into(), range(&t[1..].to_vec().repeat(2), 0.738_029_468_7, 0.738_029_468_9, false)),
                ])
                .with(&format!(
                    "Out of reach: {:.16}. Other starts give other constants: {others}.",
                    t[0]
                ))
            },
        },
        Claim {
            id: "lh-fig-1a.h-invades",
            item: "lh-fig-1a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 1a (100 groups of 100, m 500, c 0.25, p 0.9): h = 1 invades a population of k = 0 (from 1% of each group; above half by generation 150)",
            check: |_| {
                let v = invasion(&preset("lh-fig-1a", |_| {}), 10, &[50, 150], |s| s == Strategy::H(1));
                range(&column(&v, 1), 0.5, 1.0, false).with(&format!(
                    "Seeds 1–10: h = 1 at {:.3} by generation 50 and {:.3} by 150 — faster than LH01's figure (about 0.8 at 150).",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1))
                ))
            },
        },
        Claim {
            id: "lh-fig-1b.h-invades",
            item: "lh-fig-1b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 1b (execution errors 0.05): h = 1 invades (k 0, h 1) within the figure's 150 generations but does not wipe it out (h = 1 between 2% and 99% at generation 150)",
            check: |_| {
                let v = invasion(&preset("lh-fig-1b", |_| {}), 10, &[150, 500, 1000], |s| s == Strategy::H(1));
                range(&column(&v, 0), 0.02, 0.99, false).with(&format!(
                    "Seeds 1–10: h = 1 at {:.3} by generation 150, {:.3} by 500, {:.3} by 1,000 — it invades, an order of magnitude more slowly than LH01 show.",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1)),
                    mean(&column(&v, 2))
                ))
            },
        },
        Claim {
            id: "lh-fig-2a.help",
            item: "lh-fig-2a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 2a (AND strategies, one group of 100, m 500, c 0.25, mutation 0.001, no errors): help in 39% of rounds",
            check: |_| {
                let c = preset("lh-fig-2a", |_| {});
                about(&window(&c, "help_rate", 10, 100_000, 1001), 0.39, "LH01")
                    .with("Seeds 1–10, generations 1,001–100,000 (LH01: 10⁶).")
            },
        },
        Claim {
            id: "lh-fig-2b.help",
            item: "lh-fig-2b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 2b (the island model: 100 groups, p 0.9, execution errors 0.02): with drift limited, image scoring fades — help in 9% of rounds",
            check: |_| {
                let c = preset("lh-fig-2b", |_| {});
                about(&window(&c, "help_rate", 10, 3000, 1001), 0.09, "LH01").with(
                    "Seeds 1–10, generations 1,001–3,000 (LH01: 10⁵); to 5,000: 0.441; to 20,000: 0.345 ± 0.197, runs from 0.02 to 0.54. Cooperative AND strategies persist in most runs.",
                )
            },
        },
        Claim {
            id: "lh-fig-2c.help",
            item: "lh-fig-2c",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 2c (p 0.5): help in 2% of rounds, mainly a consequence of execution errors",
            check: |_| {
                let c = preset("lh-fig-2c", |_| {});
                about(&window(&c, "help_rate", 10, 3000, 1001), 0.02, "LH01")
                    .with("Seeds 1–10, generations 1,001–3,000; to 5,000: 0.154; to 20,000: 0.094 ± 0.125.")
            },
        },
        Claim {
            id: "lh-fig-2b.below-one-group",
            item: "lh-fig-2b",
            source: Source::Book,
            citation: LH01,
            text: "Limited dispersal undoes image scoring: with the same errors (0.02), one group of 100 helps more than the island model with p = 0.9",
            check: |_| {
                let one = preset("lh-fig-2a", |c| c.execution_error = 0.02);
                let islands = preset("lh-fig-2b", |_| {});
                greater(
                    &window(&one, "help_rate", 10, 3000, 1001),
                    &window(&islands, "help_rate", 10, 3000, 1001),
                    "one group",
                    "islands",
                )
                .with("Seeds 1–10, generations 1,001–3,000 (to 5,000: 0.351 against 0.441). Help is not monotone in gene flow either (the sweep lh-gene-flow: p = 1, isolated groups, 0.27; p = 0.8, 0.50).")
            },
        },
        Claim {
            id: "lh-fig-3a.help",
            item: "lh-fig-3a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 3a (c 0.1, u₀ 5, p 0.5, execution errors 0.02): help in 45% of rounds",
            check: |_| {
                let c = preset("lh-fig-3a", |_| {});
                about(&window(&c, "help_rate", 10, 3000, 1001), 0.45, "LH01")
                    .with("Seeds 1–10, generations 1,001–3,000 (LH01: 2 × 10⁵); to 10,000: 0.468 ± 0.060.")
            },
        },
        Claim {
            id: "lh-fig-3b.help-and-q",
            item: "lh-fig-3b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 3b–c (with q strategies): help in 15% of rounds, q strategies 12% of the population",
            check: |_| {
                let c = preset("lh-fig-3b", |_| {});
                let v = windows(&c, &["help_rate", "q"], 10, 3000, 1001);
                all_of(vec![
                    ("help".into(), about(&column(&v, 0), 0.15, "LH01")),
                    ("q share".into(), about(&column(&v, 1), 0.12, "LH01")),
                ])
                .with("Seeds 1–10, generations 1,001–3,000; to 10,000: help 0.148 ± 0.076, q 0.177 ± 0.111.")
            },
        },
        Claim {
            id: "lh-fig-4a.standing-invades",
            item: "lh-fig-4a",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 4a (binary discriminators, 1% standing, execution errors 0.05): standing invades and takes over (above half by generation 1,000)",
            check: |_| {
                let v = invasion(&preset("lh-fig-4a", |_| {}), 10, &[500, 1000], |s| s == Strategy::Standing);
                range(&column(&v, 1), 0.5, 1.0, false).with(&format!(
                    "Seeds 1–10: standing {:.3} at generation 500, {:.3} at 1,000.",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1))
                ))
            },
        },
        Claim {
            id: "lh-fig-4b.standing-invades",
            item: "lh-fig-4b",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 4b (execution and perception errors 0.025): standing still invades (from 1% of each group to above 10% by generation 500)",
            check: |_| {
                let v = invasion(&preset("lh-fig-4b", |_| {}), 5, &[250, 500], |s| s == Strategy::Standing);
                range(&column(&v, 1), 0.1, 1.0, false).with(&format!(
                    "Seeds 1–5: standing {:.3} at generation 250, {:.3} at 500 (the book test: 0.746 at 1,000; seeds 1–10: 0.767, 8 of 10 above half). A generation with perception errors updates five million records, so the survey stops at 500.",
                    mean(&column(&v, 0)),
                    mean(&column(&v, 1))
                ))
            },
        },
        Claim {
            id: "lh-fig-4b.condition",
            item: "lh-fig-4b",
            source: Source::Book,
            citation: LH01,
            text: "With perception errors standing is a strict best reply to itself only if vrb < c < rb (v = ε/(e + ε), r = (m − 1)/(n + m − 1)); at Fig. 4's parameters (n 100, m 500, b 1, e = ε = 0.025) the condition is not fulfilled for costs up to c = 0.25",
            check: |_| {
                let unmet: Vec<f64> = [0.05, 0.1, 0.15, 0.2, 0.25]
                    .iter()
                    .map(|&c| ind(!analytic::standing_stable(1.0, c, 100, 500, 0.025, 0.025)))
                    .collect();
                range(&unmet, 1.0, 1.0, false).with(&format!(
                    "r = {:.4}, v = {:.2}, vrb = {:.3}: the condition needs c above vrb. With e = 0.04, ε = 0.01 it holds at c = 0.25 ({}). Standing invades anyway (lh-fig-4b.standing-invades).",
                    analytic::standing_r(100, 500),
                    analytic::standing_v(0.025, 0.025),
                    analytic::standing_v(0.025, 0.025) * analytic::standing_r(100, 500),
                    analytic::standing_stable(1.0, 0.25, 100, 500, 0.04, 0.01)
                ))
            },
        },
        Claim {
            id: "lh-fig-4c.long-run",
            item: "lh-fig-4c",
            source: Source::Book,
            citation: LH01,
            text: "Fig. 4c (all four binary strategies, errors 0.025, mutation 0.0001): standing dominates, cooperators stay at appreciable frequencies, discriminators reach 5% only occasionally, defectors stay below 1%",
            check: |_| {
                untestable("Pinned by the book test lh01_fig_4c_standing_persists_with_cooperators instead (55 s): from a uniform start standing needs about 1,000 generations to dominate, and a generation with perception errors updates five million records, so no subset stays near 30 s (seeds 1–5, generations 401–600 took 33 s and standing was still at 0.31). The book test (seeds 1–3, generations 1,001–1,500): standing 0.531, cooperators 0.389, discriminators 0.080, defectors 0.0002 — each as LH01 describe.")
            },
        },
        Claim {
            id: "ns-no-offset.cooperation-wins-more",
            item: "ns-no-offset",
            source: Source::App,
            citation: OURS,
            text: "Ours: NS98's \"we add 0.1 in each interaction\" (to both players, LH01 and FAIR23) weakens selection, and cooperation wins Fig. 1 more often without it (runs won by some k ≤ 0)",
            check: |_| {
                let none = preset("ns-no-offset", |_| {});
                let both = preset("ns-fig-1", |_| {});
                greater(&cooperation_won(&none), &cooperation_won(&both), "no offset", "offset")
                    .with(&format!("Seeds 1–100 to fixation, without the offset: {}. With Fig. 2's settings: 0.777 cooperative against 0.672.", fixation_text(&none)))
            },
        },
        Claim {
            id: "ns-fig-3.fair23-visibility",
            item: "ns-fig-3",
            source: Source::App,
            citation: OURS,
            text: "Ours: FAIR23's fixed visibility (each member sees an interaction with probability 0.1) is not NS98's ten observers: at n = 20 and 50 (1.8 and 4.8 observers) it gives less cooperation than ten",
            check: |_| {
                let (a20, f20) = (fig_3(20, |_| {}), fig_3(20, |c| c.observers = 1.8));
                let (a50, f50) = (fig_3(50, |_| {}), fig_3(50, |c| c.observers = 4.8));
                all_of(vec![
                    ("n 20".into(), greater(&a20, &f20, "ten observers", "visibility 0.1")),
                    ("n 50".into(), greater(&a50, &f50, "ten observers", "visibility 0.1")),
                ])
                .with(&format!(
                    "Seeds 1–10, generations 1,001–20,000: visibility 0.1 gives {:.3} at n = 20 and {:.3} at n = 50 (ten observers: {:.3}, {:.3}); at n = 100 it is 9.8 observers, NS98's ten.",
                    mean(&f20),
                    mean(&f50),
                    mean(&a20),
                    mean(&a50)
                ))
            },
        },
        Claim {
            id: "ns-fig-3.records",
            item: "ns-fig-3",
            source: Source::App,
            citation: OURS,
            text: "Ours: Fig. 3's group-size effect needs each observer's own tally (records: tally, FAIR23's); if one sighting revealed the donor's whole score (records: score, the spec's first reading) n = 20 and n = 50 would cooperate alike",
            check: |_| {
                let score = |c: &mut ImageConfig| c.records = Records::Score;
                let (s20, s50) = (fig_3(20, score), fig_3(50, score));
                all_of(vec![
                    ("tally: 20 > 50".into(), greater(&fig_3(20, |_| {}), &fig_3(50, |_| {}), "n 20", "n 50")),
                    ("score: 20 ≈ 50".into(), equivalent(&s20, &s50, Some(MARGIN), "n 20", "n 50")),
                ])
                .with(&format!(
                    "Seeds 1–10, generations 1,001–20,000, records: score: {:.3} at n = 20, {:.3} at n = 50 (0.923 at n = 100).",
                    mean(&s20),
                    mean(&s50)
                ))
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claim::Verdict;

    #[test]
    fn about_is_an_equivalence_test_against_the_source() {
        let near: Vec<f64> = (0..10).map(|i| 0.88 + 0.004 * f64::from(i)).collect();
        assert_eq!(about(&near, 0.90, "NS98").verdict, Verdict::Holds);
        let far: Vec<f64> = near.iter().map(|x| x - 0.3).collect();
        assert_eq!(about(&far, 0.90, "NS98").verdict, Verdict::Fails);
        // Noisy seeds around the source: neither shown equal nor different.
        let noisy: Vec<f64> = (0..10)
            .map(|i| 0.47 + if i % 2 == 0 { 0.2 } else { -0.2 })
            .collect();
        assert_eq!(about(&noisy, 0.47, "NS98").verdict, Verdict::Weak);
    }

    #[test]
    fn presets_and_seeded_starts_are_image_configs() {
        let c = preset("lh-fig-4a", |_| {});
        assert_eq!(c.groups, 100);
        assert!(
            matches!(c.initial, Initial::Seeded(ref s) if s.invader == Some(Strategy::Standing))
        );
        assert_eq!(preset("ns-fig-2", |c| c.rounds = 100).rounds, 100);
        assert_eq!(seeds(3), vec![1, 2, 3]);
    }

    #[test]
    fn a_short_run_of_every_window_is_finite() {
        let c = preset("ns-fig-1", |c| c.mutation = 0.001);
        let v = windows(&c, &["help_rate", "cooperative"], 5, 20, 1);
        assert_eq!(v.len(), 5);
        assert!(v
            .iter()
            .flatten()
            .all(|x| x.is_finite() && (0.0..=1.0).contains(x)));
    }
}
```

```diff
--- a/survey/src/claims/mod.rs
+++ b/survey/src/claims/mod.rs
@@ -8,6 +8,7 @@
 mod culture;
 mod dpd;
 mod ethno;
+mod image;
 mod opinions;
 mod spatial;
 mod structure;
@@ -27,6 +28,7 @@
         culture::claims(),
         dpd::claims(),
         ethno::claims(),
+        image::claims(),
         opinions::claims(),
         spatial::claims(),
         structure::claims(),
```

Do not run `cargo fmt` over the survey crate (it is not fmt-clean); check the new file alone.

Run: `(cd survey && rustfmt --edition 2021 --check src/claims/image.rs && cargo test)` — Expected: PASS (23 tests, 3 of them `claims::image::tests`). (Its RED — the three unit tests before `image.rs` exists — is a compile error: file not found for module `image`.)

- [ ] **Step 2: Run it**

Run: `(cd survey && cargo run --release -- --only ns- && cargo run --release -- --only lh-)` — Expected (each claim's own seeds; about 4 minutes on 10 threads, the slowest claims `ns-fig-3-n100.cooperative` (23–35 s) and `lh-fig-3b.help-and-q`, `lh-fig-4b.standing-invades`, `lh-fig-2b.help` (about 26–28 s)): **Holds 14** — `ns-fig-1.more-rounds`, `ns-fig-2.cycles`, `ns-fig-2.cooperators-first`, `ns-fig-3.falls-with-n`, `ns-methods.rounds`, `ns-methods.q`, `ns-methods.cooperators`, `ns-universal.constant`, `ns-no-offset.cooperation-wins-more`, `ns-fig-3.fair23-visibility`, `lh-fig-1a.h-invades`, `lh-fig-4a.standing-invades`, `lh-fig-4b.standing-invades`, `lh-fig-4b.condition`; **Weak 8** — `ns-fig-3-n20.cooperative`, `ns-fig-3-n50.cooperative`, `ns-fig-3-n100.cooperative`, `ns-fig-4a.help-and-mode`, `ns-fig-3.records`, `lh-fig-1b.h-invades`, `lh-fig-2a.help`, `lh-fig-3b.help-and-q`; **Fails 11** — `ns-fig-1.cooperation-wins`, `ns-fig-4b.help-and-mode`, `ns-fig-4c.help-and-mode`, `ns-fig-4d.help-and-mode`, `ns-own-only.help`, `ns-rounds.two-interactions`, `ns-methods.threshold-in-simulation`, `lh-fig-2b.help`, `lh-fig-2c.help`, `lh-fig-2b.below-one-group`, `lh-fig-3a.help`; **Untestable 1** — `lh-fig-4c.long-run`; The measured values are in `survey-results.md` (e.g. Fig. 3 at n = 20, 50, 100: means 0.863, 0.440, 0.201 against 90 / 47 / 18%, Weak each; LH01 Fig. 2b: median 0.538 over generations 1,001–3,000 against 9%, Fails). Delete `survey/out/results-ns-.json` and `survey/out/results-lh-.json` (not committed).

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/image.rs survey/src/claims/mod.rs
git commit -m "Survey the image-scoring claims: fourteen hold, eight are weak, eleven fail" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 6: README, roadmap, papers, spec notes and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/papers.md`, `docs/superpowers/specs/2026-09-26-image-scoring-design.md`

- [ ] **Step 1: The spec's notes** (Decisions 4, 10, 16, 17, 22, 24–28)

```diff
--- a/docs/superpowers/specs/2026-09-26-image-scoring-design.md
+++ b/docs/superpowers/specs/2026-09-26-image-scoring-design.md
@@ -9,7 +9,7 @@
 - **PB03:** Panchanathan & Boyd, "A tale of two defectors", *J. Theor. Biol.* 224 (2003) 115–126 (binary-score analysis; standing).
 - **LFS99:** Lotem, Fishman & Stone, *Nature* 400 (1999) 226–227 and its appendix (no ESS without phenotypic defectors).
 - **OI06:** Ohtsuki & Iwasa, "The leading eight", *J. Theor. Biol.* 239 (2006) 435–444 (image scoring is not among the stable norms).
-- **FAIR23:** the Make Models FAIR NetLogo reimplementation (`imagescore.nlogo`, 2023): scores clamped at ±5, roulette reproduction, mutation uniform over k, observers Bernoulli per agent — but no payoff offset and a fixed visibility probability rather than NS98's ten observers.
+- **FAIR23:** the Make Models FAIR NetLogo reimplementation (`imagescore.nlogo`, 2023; Marco Janssen's 2010 NetLogo 4.1.1 model, republished): scores clamped at ±5, roulette reproduction, mutation uniform over k, observers Bernoulli per agent, c added to donor and recipient each round ("as stated in Nowak and Sigmund c is added…") — but a fixed visibility probability rather than NS98's ten observers, and with visibility below 1 its donor reads the recipient's record of itself (`[item ([who] of receiver) imagescoreothers] of receiver`), not its own record of the recipient: a bug (plan Decision 22).
 
 ## Goal
 
@@ -42,10 +42,11 @@
 | `local` | 1 | live | p: chance a parent is drawn from the offspring's own group |
 | `rounds`, `rounds_kind` | 125, `fixed` | live | m; `random`: each round is the last with probability 1/m |
 | `b`, `c`, `u0` | 1, 0.1, 0 | live | benefit, cost, initial payoff |
-| `offset` | `both` | live | `both`: add c to donor and recipient each round (LH01 on NS98); `none` (FAIR23) |
+| `offset` | `both` | live | `both`: add c to donor and recipient each round (LH01 on NS98, and FAIR23); `none` (our ablation: nobody's reading; plan Decision 16) |
 | `clamp` | 5 | live | scores stay in −clamp … +clamp (0: unbounded) |
 | `information` | `perfect` | reset | `perfect` or `observers` |
 | `observers` | 10 | live | mean observers per interaction (each other group member except the pair sees with probability observers/(n − 2); the recipient always sees) |
+| `records` | `tally` | live | what an observer writes: `tally` (its own record one up or down by the action seen: FAIR23's `imagescoreothers`) or `score` (the donor's true previous score ± 1: one sighting reveals the whole score); NS98 say only that onlookers "update their perception" (plan Decision 4) |
 | `execution_error` | 0 | live | e: a donor does the other action |
 | `perception_error` | 0 | live | ε: an observer records the other action |
 | `mutation` | 0 | live | ν: an offspring's strategy is redrawn uniformly from the allowed set |
@@ -59,12 +60,12 @@
 A generation (one tick), per group:
 
 1. Scores 0 (standing good; binary scores 0; q-strategy tallies at LH01's prior); payoffs `u0`; with `observers`, every perception entry unknown (0).
-2. For each round (m fixed, or until the 1/m stop): a uniformly random donor and a different uniformly random recipient in the group. The donor's view of the recipient (perfect: the score; observers: its own record) and its own score decide per its strategy; with probability e the action flips. Helping: donor −c, recipient +b; `offset: both` adds c to both. The donor's score ±1 (clamped); standing and q tallies update; observers (the recipient always, each other member with probability observers/(n − 2)) record the donor's new score (flipped action with probability ε).
+2. For each round (m fixed, or until the 1/m stop): a uniformly random donor and a different uniformly random recipient in the group. The donor's view of the recipient (perfect: the score; observers: its own record) and its own score decide per its strategy; with probability e the action flips. Helping: donor −c, recipient +b; `offset: both` adds c to both. The donor's score ±1 (clamped); standing and q tallies update; observers (the recipient always, each other member with probability observers/(n − 2)) record the action (flipped with probability ε): by default each moves its own record one up or down (`records: tally`), or it writes the donor's new score (`records: score`). Measured, Fig. 3 reproduces only with `tally` (86 / 44 / 20 % against 90 / 47 / 18 %; `score`: 97 / 93 / 92 %).
 3. After all groups: each new agent's parent is drawn by payoff-proportional roulette from its group (probability p) or from the whole population (1 − p); with probability ν its strategy is redrawn uniformly from the allowed set.
 
 ## Choices the sources leave open
 
-1. **The offset** — "we add 0.1 in each interaction": LH01 say to both players each round: default `both`; FAIR23 omits it (`none`).
+1. **The offset** — "we add 0.1 in each interaction": LH01 say to both players each round: default `both`; FAIR23 does the same (its code says so; this spec first said it omits it), so `none` is our ablation (`ns-no-offset`).
 2. **Score range** — Fig. 1's legend: ±5, clamped (FAIR23, LH01).
 3. **Reproduction** — "in proportion to their fitness": roulette with replacement, fixed n (FAIR23, LH01).
 4. **Mutation** — "another randomly chosen strategy": uniform over the allowed set (FAIR23).
@@ -77,13 +78,13 @@
 
 ## Statistics
 
-`SERIES`: `help_rate` (helps ÷ rounds), `mean_k` (over k-bearing strategies), `cooperative` (share with k ≤ 0 or a cooperative class), `mean_payoff`, `mean_score`, shares `k_cooperative`, `k_defective`, `h`, `and`, `or`, `standing`, `binary_c`, `binary_d`, `binary_x` (discriminators), `q`.
+`SERIES`: `help_rate` (helps ÷ rounds), `mean_k` (over k-bearing strategies), `cooperative` (share whose strategy helps at a generation's start: k ≤ 0, generalised to every class; plan Decision 10), `mean_payoff`, `mean_score`, shares `k_cooperative`, `k_defective`, `h`, `own_only`, `and`, `or`, `standing`, `binary_c`, `binary_x` (discriminators), `binary_d`, `q`, and `helps` (a count). (`own_only` and `helps` were added in the core: the list had no share for own-score strategies.)
 
 ## Views
 
 - **Colour modes:** **Strategy** (k on a blue–red scale; each other class its own colour), **Score** (diverging −5 … +5), **Payoff** (heat). Each group is a tile of cells, groups in a grid.
-- **Inspect:** an agent's strategy, score (with observers: how many members know it, and their mean view), payoff, helps given and received this generation.
-- **Charts:** **Help rate**, **Mean k**, **Strategy shares**, **Mean payoff**.
+- **Inspect:** a cell's group (with more than one); its agent's strategy and whether it helps at a generation's start, score (with private records: how many others have seen it act, and their mean record), standing (when standing plays), payoff, helps given and received this generation; a gap or a tile's unused cell says so. There are no neighbours to list (partners are random each round). An agent lives one generation, so a followed agent is gone at the next tick (plan Decision 27).
+- **Charts** (against the **generation**): **Help rate** (with the cooperative share), **Mean k** (only with a class that has a k), **Strategy shares** (k ≤ 0, k > 0, h, own only, AND, OR, standing, q; or, with binary scorers, **Binary scorers and standing** instead), **Mean payoff** (plan Decision 26).
 
 ## Presets
 
@@ -92,21 +93,21 @@
 | `ns-fig-1` | defaults | NS98 Fig. 1 |
 | `ns-fig-2` | m 300, ν 0.001 | NS98 Fig. 2 |
 | `ns-fig-3-n20`, `ns-fig-3-n50`, `ns-fig-3-n100` | observers 10, m 10n, ν 0.001 | NS98 Fig. 3 |
-| `ns-fig-4a` … `ns-fig-4d` | AND / OR, perfect / observers (n 20), m 500, ν 0.001 | NS98 Fig. 4 |
+| `ns-fig-4a` … `ns-fig-4d` | AND / OR, perfect / observers (n 20), ν 0.001; m 500 with perfect information, m 200 with observers ("as in figure 3 with n = 20": m = 10n; plan Decision 16) | NS98 Fig. 4 |
 | `ns-own-only` | `own_only`, m 500, ν 0.001 | NS98 Fig. 4 text |
 | `lh-fig-1a`, `lh-fig-1b` | k 0 (or k 0 & h 1) with an h 1 invader; g 100, m 500, c 0.25, p 0.9 (e 0, 0.05) | LH01 Fig. 1 |
 | `lh-fig-2a`, `lh-fig-2b`, `lh-fig-2c` | AND strategies; g 1 / g 100 p 0.9 / p 0.5; c 0.25, ν 0.001, e 0 / 0.02 / 0.02 | LH01 Fig. 2 |
 | `lh-fig-3a`, `lh-fig-3b` | c 0.1, u₀ 5, p 0.5, e 0.02; without / with q strategies | LH01 Fig. 3 |
 | `lh-fig-4a`, `lh-fig-4b`, `lh-fig-4c` | standing vs binary strategies; e 0.05 / e = ε = 0.025 / long run ν 0.0001 | LH01 Fig. 4 |
-| `fair-no-offset` | `offset: none` | FAIR23's reading |
+| `ns-no-offset` | `offset: none` | our ablation (FAIR23 adds the offset; plan Decision 16) |
 
-**Compare entries:** "One group vs the island model — Image Scoring (Compare)" (`lh-fig-2a` vs `lh-fig-2b`), "Image scoring vs standing — Image Scoring (Compare)" (`lh-fig-4a` with standing vs without), "With vs without the offset — Image Scoring (Compare)" (`ns-fig-1` vs `fair-no-offset`).
+**Compare entries:** "One group vs the island model — Image Scoring (Compare)" (`lh-fig-2a` vs `lh-fig-2b`), "Image scoring vs standing — Image Scoring (Compare)" (`lh-fig-2b` vs `lh-fig-4c`: the island model with AND strategies against standing among binary scorers; no preset has discriminators without standing, and Compare pairs presets; plan Decision 24), "With vs without the offset — Image Scoring (Compare)" (`ns-fig-1` vs `ns-no-offset`), "Small vs large groups with observers — Image Scoring (Compare)" (`ns-fig-3-n20` vs `ns-fig-3-n100`).
 
 ## Experiments and CLI
 
 - `ns-rounds`: base `ns-fig-2`; x = m 25 … 500; metric `cooperative`.
 - `ns-group-size`: base `ns-fig-3-n50`; x = n 20 … 100 (m = 10n per point via the sweep's series or paired paths); metric `cooperative`.
-- `lh-cost`: x = c 0.05 … 0.5; series one group vs island; metric `help_rate`.
+- `lh-cost`: base `lh-fig-2b`; x = c 0.05 … 0.5; series one group vs island; metric `help_rate`.
 - `lh-gene-flow`: base `lh-fig-2b`; x = p 0.5 … 1; metric `help_rate`.
 - `sugarscape presets | run | sweep` accept `image`.
 
@@ -114,19 +115,19 @@
 
 - **NS98:** Fig. 1 fixation of k = 0 (and when); Fig. 2 cycles (a measured count) and unconditional cooperators preceding defector invasions; Fig. 3's 90 / 47 / 18 %; Fig. 4's 55 / 57 / 70 / 80 % and most frequent strategies; own-only < 0.1 %; "about 2 interactions per lifetime"; the Methods' analytics (q > c/b; ~1.2 rounds; the equilibrium with cooperators) by iterating their equations and by simulation; 0.738… under each tried start.
 - **LH01:** Fig. 1a/1b invasions; Fig. 2's 39 / 9 / 2 %; Fig. 3's 45 / 15 % and 12 % q strategies; the standing condition and Fig. 4's invasions and long run.
-- **Ours:** the offset's effect on Fig. 1; FAIR23's fixed visibility against NS98's ten observers.
+- **Ours:** the offset's effect on Fig. 1; FAIR23's fixed visibility against NS98's ten observers; `records: tally` against `score` in Fig. 3.
 
-Tolerances come from the measurements (as milestones 11–19); runs shorter than the papers' are stated.
+Tolerances come from the measurements (as milestones 11–19); runs shorter than the papers' are stated. The survey judges a source's percentage by equivalence within 5 points and runs `lh-fig-4c` on a subset (plan Decision 28).
 
 ## Page
 
-- An **Image Scoring** presets group; the schema panel in groups **Game** (b, c, u₀, offset), **Population** (groups, group size, local), **Rounds** (m, kind), **Information** (mode, observers shown only with `observers`, clamp), **Errors** (execution, perception), **Evolution** (mutation), **Run** (end); `strategies` and `initial` set by presets, files and links (as ethno's `allowed`); the colour modes; charts; Inspect; the Compare entries; `defaultForm('image')` (x = rounds, `cooperative`).
+- An **Image Scoring** presets group; the schema panel in groups **Game** (b, c, u₀, offset), **Population** (groups, group size, local), **Rounds** (m, kind), **Information** (mode, observers shown only with `observers`, records, clamp), **Errors** (execution, perception), **Evolution** (mutation), **Run** (end); `strategies` and `initial` set by presets, files and links (as ethno's `allowed`); the colour modes; charts; Inspect; the Compare entries; `defaultForm('image')` (x = rounds 50 … 500, 2,000 generations, the mean `cooperative` from generation 1,001, the form's 3 seeds; plan Decision 25).
 - Keyframes, the timeline, stop rules, share links, sessions, recording and Compare work unchanged.
 
 ## Testing
 
 - **Core unit:** a round's payoffs with and without the offset; score update and clamp; each strategy class's decision (k, h, and, or, binary, standing, q) on hand-built states; execution and perception errors; observers (the recipient always sees; expected count); unknown scores as 0; random rounds; roulette reproduction and the island draw; mutation uniform over the allowed set; `initial` with an invader; `analytic.rs` against the paper's formulas; validation errors on the named field.
-- **Golden:** entries for every preset; earlier entries untouched; WASM equal to native.
+- **Golden:** entries for every preset in a separate `IMAGE_GOLDEN` of (id, generations, fingerprint): one-group presets 200 generations, island presets 20 (a 100 × 100 island generation is 50,000 rounds; plan Decision 17); earlier entries untouched; WASM equal to native, and the page's determinism test the same table.
 - **Book-style (`#[ignore]`, release):** the claims above, thresholds measured.
 - **Web (Vitest):** schema fields; charts; Inspect rows; Compare entries; `defaultForm('image')`; determinism fingerprints.
 - **Browser (controller):** every preset, colour modes, Inspect, the Compare entries, sweeps, Max speed at g 100 × n 100.
```

- [ ] **Step 2: README**

The presets menu's groups end "**Demographic PD** and **Image Scoring**"; the Image Scoring section follows the Demographic PD section (before `## Experiments`). Every number is from the measurements (Decision M of the core) and the survey:

```diff
--- a/README.md
+++ b/README.md
@@ -141,8 +141,8 @@
 
 The presets menu groups its presets by model: **Sugarscape**, **Schelling**, **Ring World**,
 **Artificial Anasazi**, **Civil Violence**, **Tag Cooperation**, **Spatial Games**, **Axelrod Culture**,
-**Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence**, **Social Structure**
-and **Demographic PD**.
+**Emergence of Classes**, **Ethnocentrism**, **Bounded Confidence**, **Social Structure**,
+**Demographic PD** and **Image Scoring**.
 Choosing a preset of another model rebuilds the world as that model; the toolbar, every speed
 (Max included), Share, Export, Record, Compare, Experiments and the CLI work the same for every
 model. A config without a `model` key is a sugarscape config, so every older config, link, session
@@ -986,6 +986,138 @@
 Pitfalls of Statistical Testing: Insights from Replicating the Demographic Prisoner's Dilemma,"
 *JASSS* 13(4) 1 (2010). See `docs/superpowers/specs/2026-09-26-demographic-pd-design.md`.
 
+### Image Scoring (Nowak & Sigmund 1998, and its critics)
+
+Nowak and Sigmund's indirect reciprocity by image scoring: 100 players, each with an image score that
+starts every generation at 0. In each generation 125 random donor–recipient pairs are drawn; a donor
+with strategy k helps if the recipient's score is at least k (cost 0.1 to the donor, benefit 1 to the
+recipient), and helping raises the donor's score by one and refusing lowers it, within −5 … +5. k runs
+from −5 (always help) to +6 (never), drawn at random at the start; players leave offspring in
+proportion to their payoffs, and an offspring may mutate to another strategy. A tick is a generation.
+Leimar and Hammerstein (2001) put the same game in an island model (100 groups of 100, a parent drawn
+from the offspring's own group with probability p), add execution and perception errors, strategies
+that look only at their own score (h), both scores (AND, OR), q strategies and Sugden's standing, and
+argue that image scoring is not evolutionarily stable.
+
+**The sources leave choices open;** each is a switch or a preset. "To avoid negative payoffs we add 0.1
+in each interaction": Leimar and Hammerstein say c is added to donor and recipient every round, as does
+FAIR23's NetLogo model, and the default follows them (`offset: none` is our ablation, `ns-no-offset`).
+Scores are clamped at ±5 (`clamp`, 0 for none), reproduction is payoff-proportional roulette and
+mutation is uniform over the allowed strategies. With observers (Fig. 3: "each interaction is
+observed, on average, by 10 randomly chosen players"), the recipient always sees and each other member
+with probability 10/(n − 2) (`observers`), and each keeps its own record, 0 until it has seen someone
+act; NS98 say only that onlookers "update their perception", so how an observer records is a switch:
+its own tally one up or down (`records: tally`, FAIR23's, the default) or the donor's new score, the
+whole score at one sighting (`records: score`). Rounds are fixed or random (`rounds_kind`: each the
+last with probability 1/m, for Leimar and Hammerstein's analysis). The classes a run allows
+(`strategies`: k, h, AND, OR, own score only, binary scorers, standing, q) and its start (uniform, or
+everyone on one strategy with an invader at a share of each group) come from presets, files and links.
+
+What reproduces, measured (release, seeds 1–10 unless noted; a window is the mean over the stated
+generations, per seed, then over seeds; NS98 averaged over 10⁷ generations and Leimar and Hammerstein
+over 10⁵–10⁶, so these runs are 10–100 times shorter):
+
+- **The universal constant, to every printed digit, and its unstated start.** With everyone at k = 0
+  and unbounded scores, NS98 give 0.7380294688360… as the largest fraction below 0 from which the
+  population still reaches all-out cooperation, without saying how the scores start. All the negatives
+  at −1 and the rest so high that they never fall below 0 gives 0.7380294688360038. Any finite start
+  lower than about +80 gives less: 0.5 with the rest at 0, 0.642 at +1, 0.688 at +2; spreading the
+  negatives over −1 … −5 gives 0.338.
+- **Fig. 2's endless cycles** (`ns-fig-2`: m 300, mutation 0.001): 172 collapses of cooperation (k ≤ 0
+  falling from at least 90 % to at most 10 %) and 167 recoveries in 10⁶ generations (seeds 1–10 ×
+  10⁵), 12–24 per seed; the unconditional cooperators (k ≤ −4) hold 68 % of the population over the 50
+  generations before a collapse against 8 % in cooperative phases, as NS98 describe. Cooperative
+  strategies hold 67 % of the time.
+- **Fig. 3's group-size effect, with each observer's own tally** (`ns-fig-3-n20`, `-n50`, `-n100`: ten
+  observers, m = 10n): cooperative strategies 86 %, 44 % and 20 % of generations 1,001–20,000 at n =
+  20, 50 and 100, against NS98's 90 %, 47 % and 18 % (to 100,000 at n = 20: 91 %).
+- **Fig. 4a** (`ns-fig-4a`, AND strategies): 53 % of interactions cooperative (NS98 55 %) over
+  generations 1,001–50,000, with (k 0, h 1) the most frequent strategy (22 %).
+- **The Methods' thresholds**: discriminators are stable against defectors above (bq + c)/(bq − c) =
+  1.2222 rounds ("about 1.2"), and only when q > c/b; with cooperators, defectors win below c(2 −
+  w)/(bwq) (0.1222 at w = 0.9).
+- **Leimar and Hammerstein's Fig. 1a**: h = 1 invades a population of k = 0 (`lh-fig-1a`: 37 % by
+  generation 50, fixed by 150 in every run; faster than their 80 % at 150). **Fig. 2a** (one group, AND
+  strategies, c 0.25): help in 37 % of rounds (they report 39 %), (k 0, h 1) the most frequent (38 %).
+  **Fig. 3's help rates**: 52 % (45 %) without and 17 % (15 %) with q strategies over generations
+  1,001–3,000 (to 10,000: 47 % and 15 %).
+- **Standing** (Fig. 4): it invades binary discriminators (`lh-fig-4a`: 69 % by generation 500, 98 % by
+  1,000) and still does with perception errors (`lh-fig-4b`: 35 % and 75 %, seeds 1–5), and from a
+  uniform start dominates with cooperators beside it (`lh-fig-4c`, seeds 1–3, generations 1,001–1,500:
+  standing 53 %, cooperators 39 %, discriminators 8 %, defectors 0.02 %) — although their condition for
+  standing with perception errors, vrb < c < rb, is not met at these parameters (v = 0.5, r = 0.833, vrb
+  = 0.417 > c = 0.25), as they say.
+
+What does not, or only partly:
+
+- **Fig. 1's victory of k = 0 is a minority outcome.** NS98 show one run in which k = 0 is fixed after
+  166 generations. Of seeds 1–100 run to fixation, k = 0 wins 20 (median generation 56) and some k ≤ 0
+  40; defection wins the other 60. More rounds do help, as NS98 say: some k ≤ 0 wins 91 of 100 at m =
+  300 and all 100 at m = 1,000.
+- **"It suffices that each player is chosen only for about 2 interactions per life-time"** (m ≈ n):
+  with Fig. 2's settings, cooperative strategies hold 18 % of generations 1,001–20,000 at m = 100 and
+  reach half only at m = 200, four interactions per lifetime (the sweep `ns-rounds`).
+- **Fig. 3 depends on how an observer records.** If one sighting revealed the donor's whole score
+  (`records: score`), n = 20, 50 and 100 would cooperate alike: 97 %, 93 %, 92 %. And FAIR23's fixed
+  visibility (each member sees with probability 0.1: 1.8 and 4.8 observers at n = 20 and 50, not ten)
+  flattens it the other way: 28 % and 16 %.
+- **Fig. 4's other panels**: 53 % (NS98 57 %) in 4b, with (k 0, h 5) most frequent (NS98 (k 0, h 4));
+  78 % and 85 % (70 %, 80 %) in 4c and 4d, where the most frequent strategies are cooperative ORs, (k 3,
+  h 4) and (k 2, h 5), and NS98's defectors (k 6, h −5) come second and third. 4b and 4d are "as in
+  figure 3 with n = 20", so m = 200 (at m = 500, 55 % and 94 %).
+- **Own-score strategies help in 0.19 % of rounds** (`ns-own-only`), not "less than 0.1 %": uniform
+  mutants keep a floor, 5 of 12 of them (h ≥ 1) helping at a generation's start.
+- **The Methods' x_min in a simulation**: over five of the Methods' rounds discriminators should beat
+  defectors from a share of 0.123; in NS98's random pairs (n 100, m 250, 2,000 seeds a share) the
+  payoff gap turns positive only at 0.16. The Methods' rounds, in which everyone plays once, are not
+  random pairs.
+- **Leimar and Hammerstein's island model does not undo image scoring here.** Fig. 2b (`lh-fig-2b`: p
+  0.9, execution errors 0.02): help in 44 % of rounds over generations 1,001–5,000 (to 20,000: 34 %,
+  runs from 2 % to 54 %) against their 9 %; Fig. 2c (p 0.5): 15 % against 2 %. Help is not even
+  monotone in gene flow (the sweep `lh-gene-flow`: 27 % in isolated groups, p = 1; 50 % at p = 0.8),
+  and below c = 0.25 the island model helps more than one group (`lh-cost`).
+- **Fig. 1b's invasion is ten times slower**: h = 1 invades (k 0, h 1) with execution errors 0.05, but
+  holds 1.8 % by generation 150, 13 % by 500 and 42 % by 1,000 (`lh-fig-1b`). **Fig. 3b's q
+  strategies** hold 26 % of the population over generations 1,001–3,000 (18 % to 10,000), against their
+  12 %.
+- **The offset lowers cooperation.** Without it (`ns-no-offset`) some k ≤ 0 wins Fig. 1 in 63 runs of
+  100 (k = 0 in 10) against 40, and Fig. 2's cooperative strategies hold 78 % of the time against 67 %:
+  adding c to both players weakens selection, and cooperation loses by it.
+
+The survey measures 34 of these claims: 14 hold, 8 are weak, 11 fail, and one (Fig. 4c's long run, too slow
+for it) is left to the book test. Four built-in sweeps (seeds 1–10):
+`ns-rounds` (Fig. 2's settings, m 25–500, cooperative strategies over generations 1,001–20,000: 1 %,
+10 %, 8 %, 18 %, 15 %, 23 %, 50 %, 65 %, 91 %), `ns-group-size` (Fig. 3, n 20–100 with m = 10n: 86 %,
+83 %, 44 %, 35 %, 20 %), `lh-cost` (c 0.05–0.5, one group against 100 groups, help over generations
+1,001–5,000: 58 % and 53 % at 0.05, 35 % and 44 % at 0.25, 6 % and 2 % at 0.5) and `lh-gene-flow` (p
+0.5–1: 15 %, 35 %, 39 %, 50 %, 44 %, 27 %).
+
+Each group is a tile of cells, the groups in a grid with a gap between tiles. Agents are drawn by
+**Strategy** (the default: k on a blue–red scale from −5 to +6, h, own-score, standing and q
+strategies each their own color), **Score** (below 0 red, above blue) or **Payoff** (heat). Inspect shows
+a cell's group, its agent's strategy and whether it helps at a generation's start, its score (with
+observers, how many others have seen it act and their mean record of it), its standing when standing
+plays, its payoff and the help it gave and received; a gap or a tile's unused cell says so. An agent
+lives one generation, so a followed agent's offspring are new agents. Charts, against the generation:
+**Help rate** (with the share of cooperative strategies), **Mean k**, **Strategy shares** (the binary
+scorers and standing in their own chart) and **Mean payoff**. A run never stops by default (`end`: a
+last generation, 0 for never); the game, the rounds, the observers, the records, the errors, mutation
+and `local` apply to the running world from the next generation. **Compare** entries: "One group vs
+the island model — Image Scoring (Compare)" (`lh-fig-2a` and `lh-fig-2b`), "Image scoring vs standing
+— Image Scoring (Compare)" (`lh-fig-2b` and `lh-fig-4c`: the island model with AND strategies or with
+standing), "With vs without the offset — Image Scoring (Compare)" (`ns-fig-1` and `ns-no-offset`) and
+"Small vs large groups with observers — Image Scoring (Compare)" (`ns-fig-3-n20` and
+`ns-fig-3-n100`). Credit: Martin A. Nowak and Karl Sigmund, "Evolution of indirect reciprocity by
+image scoring," *Nature* 393 (1998), 573–577 (IIASA IR-98-040, with "The Dynamics of Indirect
+Reciprocity," *J. Theor. Biol.* 194 (1998), 561–574); Olof Leimar and Peter Hammerstein, "Evolution of
+cooperation through indirect reciprocity," *Proc. R. Soc. Lond. B* 268 (2001), 745–753; Karthik
+Panchanathan and Robert Boyd, "A tale of two defectors," *J. Theor. Biol.* 224 (2003), 115–126; Arnon
+Lotem, Michael A. Fishman and Lewi Stone, "Evolution of cooperation between individuals," *Nature* 400
+(1999), 226–227; Hisashi Ohtsuki and Yoh Iwasa, "The leading eight," *J. Theor. Biol.* 239 (2006),
+435–444; and Marco Janssen's NetLogo model of image scoring (2010), republished by Make Models FAIR
+(2023), whose donor, with visibility below 1, reads the recipient's record of itself. See
+`docs/superpowers/specs/2026-09-26-image-scoring-design.md`.
+
 ## Experiments
 
 The header's **Experiments** switch replaces the grid with a sweep runner (the playground's
```

- [ ] **Step 3: Roadmap and papers** (Decision 31)

```diff
--- a/docs/roadmap.md
+++ b/docs/roadmap.md
@@ -174,6 +174,18 @@
 payoffs never converge to pure defection, footnote 27's monopoly comes in 1 run of 30, and the metabolism
 "equivalence" is exact only when metabolism is charged per game. See
 `docs/superpowers/specs/2026-09-26-demographic-pd-design.md`.
+
+## Milestone 21: Image scoring (done)
+
+Nowak and Sigmund's image scoring (1998) as a fourteenth model kind, with Leimar and Hammerstein's island
+model, errors, own-score, standing and q strategies, the payoff offset and how an observer records as
+switches. (Numbered 21: the norms milestone takes 20.) NS98's universal constant reproduces to every
+printed digit, and its unstated start is found (the negatives at −1, the rest out of reach); Fig. 2's
+endless cycles and Fig. 3's group-size effect reproduce, the latter only when each observer keeps its own
+tally. But Fig. 1's victory of k = 0 comes in 20 runs of 100 (defection wins 60), two interactions per
+lifetime give cooperation 18 % of the time, not most, and Leimar and Hammerstein's island model does not
+undo image scoring (44 % help against their 9 %), though standing invades as they say. See
+`docs/superpowers/specs/2026-09-26-image-scoring-design.md`.
 
 ## Experiments and science
 
@@ -190,6 +202,7 @@
 - **Hegselmann & Krause's bounded confidence**: done (Milestone 17).
 - **Cohen, Riolo & Axelrod's social structure**: done (Milestone 18).
 - **Epstein's demographic Prisoner's Dilemma** (and Radax & Rengs' replication): done (Milestone 19).
+- **Nowak & Sigmund's image scoring** (and Leimar & Hammerstein's critique): done (Milestone 21).
 - **Credit hierarchy view**: done (Milestone 6).
 
 ## Model extensions
```

```diff
--- a/docs/papers.md
+++ b/docs/papers.md
@@ -27,6 +27,7 @@
 | 17 | `opinions` | `bounded-confidence/hegselmann-krause-2002-…` | Fig. 2b's two camps are the exception; the lattice claim holds |
 | 18 | `structure` | `social-structure/cohen-riolo-axelrod-2001-role-of-social-structure.pdf` | reproduces closely; the unstated threshold is 2.3; only the Appendix's noise rule keeps FRNE above 2DK |
 | 19 | `dpd` | `demographic-pd/epstein-1998-zones-of-cooperation-in-demographic-pd.pdf` (the working paper), `demographic-pd/epstein-2006-generative-social-science.pdf` (ch. 9: the published rule, Tables 9.1 and 9.3), `demographic-pd/radax-rengs-2009-mpra-replication-of-the-demographic-prisoners-dilemma.pdf` (published as JASSS 13(4) 1, 2010) | Tables 1 and 2 do not reproduce under the published rule; only unstated readings (founders with no wealth, the working paper's rule) come close; the metabolism "equivalence" holds only per game |
+| 21 | `image` | `image-scoring/nowak-sigmund-1998-iiasa-indirect-reciprocity-by-image-scoring.pdf` (with NS98b's draft), `image-scoring/leimar-hammerstein-2001-prsb-cooperation-through-indirect-reciprocity.pdf`; Panchanathan & Boyd 2003, Lotem, Fishman & Stone 1999, Ohtsuki & Iwasa 2006 and FAIR23's `imagescore.nlogo` (*not in `papers/`*) | the universal constant reproduces to every digit; Fig. 1's k = 0 wins 1 run in 5; two interactions per lifetime do not suffice; LH01's island model does not undo image scoring |
 
 ## Queue
 
@@ -37,16 +38,15 @@
 |---|---|---|---|---|---|
 | 1 | Axelrod's norms and metanorms | `norms/axelrod-1986-apsr-evolutionary-approach-to-norms.pdf` | `norms/galan-izquierdo-2005-jasss-appearances-can-be-deceiving.html` | small | new kind (20 agents, boldness and vengefulness, a genetic algorithm) |
 | 2 | Relative agreement and extremism | `bounded-confidence/deffuant-neau-amblard-weisbuch-2000-acs-mixing-beliefs.pdf` (the pairwise original), `bounded-confidence/deffuant-amblard-weisbuch-faure-2002-jasss-how-can-extremism-prevail.html` | `bounded-confidence/amblard-deffuant-2004-network-topology-and-extremism.pdf`, `bounded-confidence/weisbuch-2003-bounded-confidence-and-social-networks.pdf` | medium | extends `opinions` (pairwise updating, uncertainty, networks) |
-| 3 | Image scoring | `image-scoring/nowak-sigmund-1998-iiasa-indirect-reciprocity-by-image-scoring.pdf` | `image-scoring/leimar-hammerstein-2001-prsb-cooperation-through-indirect-reciprocity.pdf` | medium | new kind |
-| 4 | El Farol and the minority game | `el-farol/arthur-1994-aer-inductive-reasoning-and-bounded-rationality.pdf` | `el-farol/challet-zhang-1997-emergence-of-cooperation-minority-game.pdf` | small | new kind (predictor pools, the memory transition) |
-| 5 | Ants and recruitment (herding) | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf` | — | small | new kind (N agents, two sources, random recruitment and switching; the bimodal regime) |
-| 6 | Threshold models | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*) | — | small | new kind, or a Sugarscape rule |
-| 7 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
-| 8 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
-| 9 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
-| 10 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
-| 11 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
-| 12 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
+| 3 | El Farol and the minority game | `el-farol/arthur-1994-aer-inductive-reasoning-and-bounded-rationality.pdf` | `el-farol/challet-zhang-1997-emergence-of-cooperation-minority-game.pdf` | small | new kind (predictor pools, the memory transition) |
+| 4 | Ants and recruitment (herding) | `ants/kirman-1993-qje-ants-rationality-and-recruitment.pdf` | — | small | new kind (N agents, two sources, random recruitment and switching; the bimodal regime) |
+| 5 | Threshold models | `thresholds/granovetter-1978-ajs-threshold-models-of-collective-behavior.pdf` (*scan*) | — | small | new kind, or a Sugarscape rule |
+| 6 | The timing of retirement | `retirement/axtell-epstein-1999-coordination-in-transient-social-networks-retirement.pdf` | — | small | new kind (age cohorts, rational and imitating agents, a social network) |
+| 7 | Altruistic punishment | `punishment/boyd-gintis-bowles-richerson-2003-pnas-altruistic-punishment.pdf` | — | medium | new kind (groups, migration, conflict) |
+| 8 | Zero-intelligence traders | `zi-traders/gode-sunder-1993-jpe-zero-intelligence-traders.pdf` (*scan*) | `zi-traders/cliff-1997-hp-minimal-intelligence-agents-for-bargaining.pdf` (Cliff's HP Labs report: where ZI-C fails, and ZIP traders) | medium | new kind: a double auction with values and costs (Sugarscape's `PriceRule::Random` is only the bilateral analog) |
+| 9 | Bali water temples | `bali/lansing-kremer-1993-am-anthropologist-balinese-water-temples.pdf` (*scan*) | `bali/janssen-2007-agricultural-systems-coordination-in-irrigation.pdf`; github.com/mars0i/bali (NetLogo, no license: reference only); Janssen's CoMSES model 2221 for the watershed data (check its license first) | large | new kind on a watershed: subaks, dams, rain, pests |
+| 10 | Emergence of firms | `firms/axtell-1999-emergence-of-firms.pdf` (108 pp) | — | large | new kind (team formation, power-law firm sizes) |
+| 11 | Emergent actors in world politics | `geopolitics/cederman-1997-emergent-actors-in-world-politics.pdf` (274-page book) | — | very large | new kind (states on a grid, conquest, nationalism) |
 
 ## Wanted
 
```

- [ ] **Step 4: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo +1.98.1 clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test image -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && rustfmt --edition 2021 --check src/claims/image.rs && cargo test)
```

Expected: all PASS (wasm-pack: 42; web: 46 files, 638 tests; survey: 23 tests; the 18 ignored image tests about 4.5 minutes). Browser (controller): every image preset, the three colour modes, Inspect (agents, "Seen by" with observers, gaps, unused cells, a followed agent's generation passing), the Rules panel's groups and hidden observers, the four Compare entries, the four sweeps, keyframes and the timeline, share links and sessions with a live change, recording, Max speed at g 100 × n 100, and every existing scenario.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/roadmap.md docs/papers.md docs/superpowers/specs/2026-09-26-image-scoring-design.md
git commit -m "Document image scoring and mark milestone 21 done" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

## Self-review (planning)

- **Spec coverage:** Architecture, Config, Rules, Statistics and the analytics — Task 1 (Decisions 1–22); the choices the sources leave open — Task 1's code and Decisions, Task 6's spec notes; Views — frames and Inspect JSON in Task 1, colour modes, charts and Inspect rows in Task 3; Presets and Compare — Tasks 1 and 4; Experiments and CLI — Task 2 and Task 4's `defaultForm`; Claims — Task 2's book-style tests and Task 5's survey; Page — Tasks 3–4; Testing — every task; Docs, papers index — Task 6.
- **Placeholders:** none; every code block is the planning dry runs' code, which passed fmt, clippy (stable and 1.98.1), the workspace tests, the ignored image tests, wasm-pack, the web build and tests, and the survey's tests (the Review Focus tests were added to the dry run and pass).
- **Review Focus:** items 1–4 name their tests (Task 1); item 5 is for the final review.
- **Caveat:** only the final state of each part was run; the RED expectations of intermediate steps are stated, not observed.
