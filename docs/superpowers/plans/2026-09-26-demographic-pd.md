# Demographic Prisoner's Dilemma Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Epstein's Demographic Prisoner's Dilemma (1998) as a model kind, `dpd`, with the working paper's and the published text's rules, Radax and Rengs' unstated timing choices, soup, metabolism and the coordination game as named switches, presets, sweeps and survey claims, and every claim measured.

**Architecture:** A new module `crates/sugarscape-core/src/dpd/` implementing the `Model` trait on the shared periodic square geometry (von Neumann for moves, play and births; Moore for the surrounded index), wired into `ModelKind`/`ModelConfig`/`ModelWorld` with keyframes, presets and golden entries; book-style tests and four sweeps measure it against Epstein and Radax & Rengs; the page gains its types, colour modes, charts, Inspect, Compare entries and Experiments default; the survey gains its claims.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm via wasm-pack, sugarscape-cli, the standalone survey crate), TypeScript (Vite, uPlot, Vitest).

**Spec:** `docs/superpowers/specs/2026-09-26-demographic-pd-design.md`

## Global Constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **The default is the sources' stated rule** (the published text, GSS Table 9.1, the CD's settings where the prose is silent); each disagreement or unstated choice is a named switch.
- **Deterministic and portable:** a function of (config, seed); all draws from the world's seeded `SimRng` with `u32` ranges; no platform transcendental functions; fingerprints identical native and WASM.
- **Truthful descriptions,** with measurements, including what does not reproduce; numbers only from Decision M (the measurements), never invented.
- **Verify with:** `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` and `cargo +1.98.1 clippy --all-targets -- -D warnings` (CI runs the newest stable clippy), `cargo test --workspace`, `cargo test -p sugarscape-core --release --test dpd -- --ignored`, `wasm-pack test --node crates/sugarscape-wasm`, `(cd web && npm run build && npm test)`, `(cd survey && cargo test)`.
- **Do not run `cargo fmt` inside `survey/`** (it would rewrite unrelated chapters; the survey is not a workspace member).
- **Commits:** stage only the files the task names (never `-A`/`.`, never `.claude/`, `.superpowers/`, `web/src/wasm-pkg`); messages are plain imperative sentences and end with a second `-m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"`.
- **Web:** TypeScript strict with `noUnusedLocals`/`noUnusedParameters`; build DOM with `h()`; typographic apostrophes (’) in test names.
- **If a measured number differs from the plan, stop and report; do not retune thresholds.**

## Review Focus

1. **A full lattice from the start** (`agents` = width²): nobody can move or clone, play still runs, the lattice stays consistent — Task 1, `a_full_lattice_starts_and_runs_with_nowhere_to_move_or_clone`.
2. **A lone agent** (after a die-off, or `agents: 1`), in space and in soup: it plays no one and its wealth is untouched, nothing panics — Task 1, `a_lone_agent_plays_no_one_in_space_or_soup`.
3. **An endowment above the fission wealth** (the Rules panel allows it): the rules as written — the parent pays and dies, the offspring lives — Task 1, `an_endowment_above_the_parents_wealth_kills_the_parent`.
4. **A fission wealth of 0** (every agent clones every turn): the lattice fills without breaking — Task 1, `a_zero_fission_wealth_fills_the_lattice_without_breaking_it`.
5. **A vision wider than the lattice** (vision 10 on 3 × 3): each site within reach counts once — Task 1, `a_vision_wider_than_the_lattice_reaches_each_site_once`.

## Why this task order

The core model (Task 1) carries everything later tasks read: the config and schema, the statistics, the inspection JSON and the golden fingerprints. Task 2 measures it and adds the sweeps, which need only the core. The page (Tasks 3–4) needs the core's JSON; the survey (Task 5) needs the presets and the measured tolerances; the docs (Task 6) quote everything.

## Decisions (where the spec leaves room)
Binding once the plan is written. Every rule below was implemented in the scratch copy and measured.
1. **Wiring.** `ModelKind::Dpd` last (`ALL: [ModelKind; 11]`), `"dpd"`; the catalog ends with the dpd presets; `pub mod dpd;` after `pub mod culture;`. The label "Demographic PD" lives only in `web/src/models.ts` (Rust has no kind labels) — a page-task item.
2. **Lattice.** Two `spatial::Geometry`s on the periodic square (von Neumann for moves, play and births; Moore for the surrounded index), built with a throwaway `rng::seeded(0)`, shared by `Arc`. Sites `x + y·w`; neighbours up, left, right, down. Vision: `reach(width, vision)` = the distinct wrapped displacements with |dx| + |dy| ≤ vision, ascending (dy, dx), duplicates dropped (a 3-wide torus has 8 other sites, not 12).
3. **Agents and the list.** `Agent { id, cooperator, wealth: f64, age: u32, site, dead, income, games }` held in the call list (`Vec<Agent>`); `at[site]` → list index; an unordered empty-site list with slots (as ethno). The dead stay in the list, flagged, until the cycle's end; `removal: immediate` vacates the site at death, `end_of_cycle` leaves the body on it (it blocks moves and births and is never played).
4. **The RNG contract.** Setup, per agent: site (`gen_range` over the empty list), strategy (`gen::<f64>() < initial_cooperators`), age (`gen_range(0..max) + 1`, only with a maximum). Turn: move = `gen_range` over the free reach sites (nothing drawn when none; soup: over the empty list); play: `each_neighbor` draws nothing, `random_neighbor` `gen_range` over living neighbours, soup rejection-samples `gen_range(0..len)` until another living agent; reproduce (only at wealth ≥ fission): target (`gen_range` over free neighbours; soup: the empty list), then mutation (`gen::<f64>()`, always drawn), then the newborn's age (random with a maximum). End: `swaps` = N/2 pairs of `gen_range(0..N)` (a pair may coincide), `full` = `SliceRandom::shuffle`.
5. **Within a turn.** Death checks after each game, mover first; a mover that dies stops playing. `own_turn`: the only check is the agent's own step 4, so it may recover on its turn (and the mover plays on while negative). Metabolism per game is charged after that game's payoff and before the check; per cycle in step 4 before the check.
6. **Ages.** Step 4 adds 1 and kills `age > max_age`; initial and random newborn ages uniform in 1 … max (so a random-aged agent lives 1 … max more cycles); `newborn_age: zero` is 0 (RR's FALSE is 1 with "age ≥ max" — we keep the spec's 0 and our order, so a zero-aged newborn dies on its (max + 1)th turn). With no maximum ages start at 0 and only count.
7. **Soup.** Partner uniform over the other living agents; movement and offspring uniform over the empty sites anywhere; `play` ignored (the schema hides it unless `pairing: space`).
8. **Statistics.** Taken after the cycle's removals; `cooperator_share`, `wealth_c`, `wealth_d` NaN with nobody to count; `births`, `deaths` this cycle; the t = 0 snapshot counts the initial agents.
9. **Validation beyond the spec.** `initial_wealth` and `metabolism` finite and ≥ 0; `agents ≤ width²` checked only when the width is valid (one error per bad field).
10. **`dpd-shifted` reads GSS's explicit settings** (max age 100, zero mutation — Run 2's), not Run 5's as the spec proposed: GSS states them; the working paper's context (after Run 5, "figure 12") cannot produce pure defection with 50% mutation. `dpd-metabolism` follows, so the Compare entry "Negative payoffs vs shifted with metabolism" should pair `dpd-run-2` with `dpd-metabolism` (spec: `dpd-run-5`).
11. **`metabolism_per` default `cycle`** as the spec; the per-game reading is the exact equivalence (measured) and is what the `dpd-metabolism` sweep's second series shows.
12. **Frames.** Strategy `BLUE`/`RED`; Wealth `lerp(COOL, HOT, wealth / (2·max(fission, 1)))`; Age `lerp(COOL, HOT, age / max_age)` (÷ 1,000 with no maximum); Surrounded: surrounded cooperators `BLUE`, everyone else `lerp(BACKGROUND, own colour, 0.35)`; empty `BACKGROUND`.
13. **Inspect.** The spec's "each neighbour with this cycle's payoff between them" is `payoff` / `their_payoff`: one game's payoffs between them under the current matrix; the agent's own `income` and `games` this cycle are its totals (per-partner totals are not stored — agents move). JSON `{ site: {x, y}, agent: { id, strategy: "C" | "D", wealth, age, max_age, surrounded, income, games, neighbors: [{x, y, id, strategy, payoff, their_payoff}] } | null }`. Agents CSV `id,x,y,strategy,wealth,age,surrounded`. Fingerprint: FNV-1a over tick, id counter, and each agent in call order (id, site | cooperator << 32, wealth bits, age) — the list order is state.
14. **Live and reset.** Reset: `width`, `agents`, `initial_cooperators`, `initial_wealth`, `vision`, `schedule`; all else live (changes land between cycles, when no body is on the lattice). `end` 0 = never; sweep error "the demographic PD stops at its last cycle, N in this config"; CLI "finished at tick N (its last cycle)".
15. **`dpd-payoffs` layout.** x = R 1–9 (each value sets `r` and `p = −r`), series T 2–10 (sets `t`, `s = −t`): 81 cells × 30 seeds = 2,430 runs (limits 64 x, 16 series, 10,000 points). The sweep format cannot skip cells and its sets are absolute, so x = T − R cannot be expressed without a new feature; the 36 cells with R ≥ T run and the description says so (cooperators 774–876 there). Smallest option: no code.
16. **Sweeps** (after `jansson-markers`): all 30 seeds, 500 ticks, `final` `cooperators`. `dpd-mutation` base `dpd-run-2`, x 0, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5; `dpd-metabolism` base `dpd-shifted`, x 0–6, series per cycle / per game; `dpd-max-age` base `dpd-run-1`, x 10, 20, 50, 100, 200, 500, 1,000.
17. **A thirteenth preset, `dpd-closest`** (not in the spec): the working paper's rule, no initial wealth, newborns this cycle — the finding below, so the page can show it; source string the working paper's.
18. **Test statistic.** RR's pooled t (identical to Welch's statistic at n = 30 each), |t| < 2.0017.
19. **Book-style tests** pin seed means to ±0.06 (one-decimal descriptions) and outcome counts exactly, with the source's number beside each; failing claims are pinned as measured.
20. **Spec corrections** (docs task): RR's Run 1 count is 1 of 128 (synchronous, set aside), not 0; `dpd-shifted` settings (Decision 10) and the Compare pairing; the metabolism claim holds exactly per game; add `dpd-closest`; footnote 27's R is 11 as printed but "hiked by ten" gives 15.

**M. The measurements** — `measurements.md` (every number, method and seed set). Re-measuring: `cargo test -p sugarscape-core --release --test dpd -- --ignored --nocapture` and `cargo run --release -p sugarscape-cli -- sweep --builtin <id> --quiet --summary-csv /dev/stdout --out /dev/null`. If an implementation following this plan gives different numbers, stop and report rather than retune.

### Surprises (for the README and the survey)
1. **The published rule never reproduces Table 1, under any of RR's timing settings (0/64), and Table 2 only once (1/64, not RR's best fit).** The defaults give 729 / 171 against 779 / 121.
2. **Table 1 needs the initial agents to start with no wealth.** In 512 readings (rule × newborn timing × initial wealth × RR's 64), all 46 that reproduce Run 1 have initial wealth 0 — never the CD's 6 — and all 7 that reproduce both tables also use the working paper's random-neighbour rule. The text's own timing with those two changes and newborns acting at once (`dpd-closest`) passes both tables on seeds 1–30 (Run 2's defectors drift high on other seeds). So the 1998 tables look like the working paper's rule, not the published one, with unfunded founders.
3. **The shifted payoffs (12, 11, 1, 0) never converge to pure defection** (30/30 coexist under Run 2's, Run 5's and Run 1's settings, to 2,000 cycles). Fig. 13 does not reproduce, and with it the premise of the metabolism argument ("in which cooperators are annihilated").
4. **The metabolism "equivalence" is exact per game and only approximate per cycle** — the chapter defines metabolism per cycle (note 29) but its sentence says "after every interaction", and only that reading is "equivalent mathematically".
5. **Run 4 (R = 1) dies out** (26/30 by 500, 30/30 by 2,000) after at most two swings: no predator–prey cycles, no cooperator monopoly, so no R = 1 paradox (0/30 monopolies at R = 1 and at R = 5). No timing setting produces sustained cycles.
6. **Epstein's all-zero cells are impossible under the stated rules**: once cooperators are gone, the last 1–5 defectors wander forever with nobody to play (no maximum age, no metabolism); his (0, 0) ranges need every defector to die.
7. **Footnote 27's monopoly is rare** (1/30 by 2,000, 0/30 by 500) and its R = 11 contradicts "hiked by ten" (R = 15); both give the same.
8. **RR's best Run 2 fit does not transfer** (703 / 164 here vs their 780 / 97) — the same six switches in two implementations give different models; their pseudo-code fixes details (neighbours in random order, deaths checked after all games, "age ≥ max") the text does not.
9. **Soup and Run 5 reproduce**; the coordination game's norm maps persist in only about half the runs and erode (17/30 at 500, 11/30 at 5,000).
10. **Smaller slips confirmed**: "approximately 5 to 1" is 4.3 to 1 here; Table 9.3's (4, 2) defector interval is misprinted; the metabolism passage's "figure 12" means Fig. 13; RR's own text counts 1 of 128 Run 1 fits.


21. **A fourth Compare entry, "Published rule vs closest reading — Demographic PD (Compare)"** (`dpd-published-vs-closest`: `dpd-run-1` vs `dpd-closest`), after the spec's three. The closest reading is the milestone's main finding (Decision 17, Surprise 2): side by side the page shows the same demography giving 729 / 171 and 786 / 114. The three spec entries keep their labels; the metabolism entry pairs `dpd-run-2` with `dpd-metabolism` (Decision 10), not `dpd-run-5`.
22. **Charts.** The five charts in the spec's order (Population, Cooperator share, Surrounded cooperators, Wealth, Births and deaths), cooperators `--blue` and defectors `--red` (the frame's colours), `cooperator_share` on [0, 1]; the others unranged (populations reach 900, wealths are unbounded). The time axis reads **"Cycle"** (Epstein's word throughout: "cycles", "the cycle"), like ethno's "Period". **No phase diagram:** `ModelChart` is a time chart (x = tick) only, so a cooperators-vs-defectors plot would be a new chart type; the spec makes it conditional on not needing one.
23. **`defaultForm('dpd')`**: x = `r` over `1:5:1` (Table 9.3's row at T = 6, the defaults' own T; R = 1 dies out, R = 5 is Run 1), 500 cycles, the final `cooperators`, and the form's usual 3 seeds (the spec said 30: every other model's default form uses 3, and 5 × 30 runs would be the heaviest default; the built-in `dpd-payoffs` runs Epstein's 30).
24. **Survey.** Every dpd claim runs Epstein's and RR's seeds 1–30 whatever `--seeds` says (as HKS13's claims run their own 50), so each verdict's numbers are Decision M's. Judges are the survey's own: `range` (per run inside the source's range; per cell or per setting as 0/1 indicators where the claim counts cells or settings), `greater`, `equivalent` (default 10 % margin), `all_of`; RR's pooled t is reported in each detail, not judged. The factorial: `dpd-rr.run-1` runs the 32 asynchronous settings against Table 1 and `dpd-rr.run-2` RR's own Run 2 fits (Table 7's seven, six once the RNG column is dropped); the full 64 × 2 stays pinned in the core's `radax_and_rengs_factorial_as_measured`. 21 claims, 20 s in all on 10 threads, the slowest 5.1 s. Memoized per process by config and length.
25. **The view guard.** `isDpdView(v, model)` = `model === 'dpd' && (agent === null || 'surrounded' in agent)`: an empty dpd site is `{ site: { x, y }, agent: null }`, the shape of an empty Schelling or ethnocentrism site, so the model decides; `surrounded` appears in no other model's agent view. In `inspect-panel.ts` it is tested second, right after `isEthnoView` and before every shape-only guard.
26. **The Rules panel needs no schema or panel change.** The core's schema already gives the payoffs sliders on [−20, 20] (the coordination game's −3, footnote 27's 16), metabolism [0, 20], max age [0, 100,000], and `play` shown only while `pairing` is `space`; `paramShown` compares a choice's string, `paramEdit` sends a typed negative number as is. Tests pin it: a negative payoff round-trips through the form; the engine test checks the group order, the payoff bounds, `play`'s `show_if`, and that every dpd preset's numbers sit on their sliders; a live payoff edit, a live switch to soup and a live out-of-range mutation (the core names the field) go through the engine.
27. **Inspect rows** (`dpdRows`): Agent (`#id · cooperator`), Wealth, Age (`37 cycles (maximum 100)` or `(no maximum)`), Surrounded (`yes: all eight neighbors (Moore) cooperate` or `no`), This cycle (`4 from 2 games`), then each neighbour `(x, y)`: `#id · strategy · a game pays this agent P and the neighbor Q`; `Neighbors: none` for a lone agent. Numbers whole or to two decimals, negatives with an ASCII minus (as the Rules panel's boxes). An empty site: `Agent: none (an empty site)`; a followed agent that dies: "Agent #N has died."
28. **Finished notice**: "This run has reached its last cycle (N) — Reset to run it again"; `ticksLeft` counts to `end` when set (the default 0 never finishes).
29. **Docs.** README section after Ethnocentrism; the presets menu line ends "**Ethnocentrism** and **Demographic PD**"; roadmap Milestone 17 after 16 and an Experiments line; the spec's notes as Task 6 Step 1 (Decisions 10, 15, 17, 20–24).

### Survey results (Decision 24; `survey-results.md` has the full rows)

**Holds 6:** `dpd-payoffs.collapse` (45/45 cells die out where Epstein's do), `dpd-run-5.persists` (27/30), `dpd-soup.pure-defection` (29/30), `dpd-metabolism.per-interaction` (30/30 runs identical), `dpd-metabolism.necessity` (p = 5e-11), `dpd-rr.run-1` (no asynchronous setting reproduces Table 1: 32/32). **Weak 4:** `dpd-metabolism.per-cycle` (644 ± 97 against 695 ± 29; TOST p = 0.19), `dpd-coordination.regions` (both conventions in 16/30 at 2,000; regions 16/16), `dpd-working-paper.table-1` (22/30 in each range), `dpd-closest.both-tables` (Table 1 21/30 per run; Table 2 30/30). **Fails 11:** `dpd-run-1.table-1` (2/30 cooperators in range), `dpd-run-2.table-2` (11/30, 3/30), `dpd-run-1.five-to-one` (10/30; 4.34 at t = 50), `dpd-payoffs.table-9-3` (22/90 in CI, 51/90 in range), `dpd-payoffs.all-zero` (5/330 runs empty), `dpd-run-4.cycles` (0/30 with 3 swings; none alive at 2,000), `dpd-run-4.paradox` (0 against 677), `dpd-shifted.pure-defection` (0/30), `dpd-footnote-27.monopoly` (1/30), `dpd-rr.run-2` (0/6 of RR's fits), `dpd-rr-best.table-2` (12/30, 13/30).

## Decision M: the measurements (planning dry run, recorded 2026-09-26)

### Tables 1 and 2 (the defaults: the published rule, CD wealths, the text's timing)

| | Cooperators | Defectors | Epstein C | Epstein D | t (C / D) |
|---|---|---|---|---|---|
| Run 1 (no max age) | 729.2 ± 17.4 (698, 778) | 170.5 ± 17.3 (122, 201) | 779 ± 15 (752, 806) | 121 ± 15 (93, 148) | 11.86 / −11.83 |
| Run 2 (max age 100) | 694.8 ± 29.4 (628, 741) | 195.9 ± 28.2 (152, 263) | 784 ± 29 (708, 846) | 99 ± 25 (45, 160) | 11.83 / −14.09 |

Population: Run 1 899.7 ± 0.6 (the lattice fills), Run 2 890.8 ± 3.7. Surrounded cooperators at 500: Run 1 380 ± 45, Run 2 320 ± 74. **Not reproduced**: cooperation dominates, but 50–90 fewer cooperators and 50–100 more defectors than Epstein; both rejected.

**"About 5 to 1 by t = 50" (Run 1).** Ratio of the means C/D: t = 15: 3.75; 30: 4.14; **50: 4.34** (median seed 4.28, range 3.48–6.23); 100: 4.36; 500: 4.28. A stable ratio from about t = 30, as Epstein says, but about 4.3 to 1, not 5. (Run 2: 4.38 at 50, falling to 3.55 by 500.)

### Table 9.3 (45 cells, Run 1's other settings)

Method: T = 2 … 10, R = 1 … T − 1, S = −T, P = −R; 30 runs; counts at 500. A mean counts as inside one of Epstein's (rounded) intervals if it rounds into it (lo − 0.5 ≤ m < hi + 0.5). "t-pass" = both t-tests pass at 2.0017.

**Tally: of 90 means (45 cooperator, 45 defector), 22 fall inside Epstein's 95% CI and 51 inside his range. Cells with both means inside the CI: 1 of 45 ((8, 3)). Cells with both means inside the range: 14 of 45. Cells passing both t-tests: 5 of 45 ((9, 3), (8, 3), (6, 2), (3, 1), (2, 1)).** Of the 22 in-CI means, 12 are cooperator zeros in cells where both die out, 6 are cooperators on the high-variance edge of the collapse ((9, 3), (8, 3), (7, 3), (6, 2), (5, 2), (3, 1)), 1 is (3, 2)'s cooperators, and 3 are defectors ((10, 4), (8, 3), and (4, 2) only by its misprinted interval). Along the diagonal (R = T − 1) our cooperators run 13–92 below and our defectors 33–106 above (bar T = 2). The qualitative pattern holds exactly: cooperators die out at the same R in every row (R ≤ 3 at T = 10, R ≤ 2 at T = 7–9, R = 1 at T ≤ 6). **Epstein's all-zero cells are never all-zero here: 1–5 lone defectors survive** (mean 2.2–3.1 at t = 500; still ≥ 1 on average at 2,000 for (10, 1)) — with nobody to play, no maximum age and no metabolism nothing can kill them; his ranges (0, 0) require every last defector to die. Row (4, 2)'s defector CI "(254, 376)" is a misprint (mean 265, s.d. 32 → (253, 277)); our 339 is inside the misprinted interval and outside the corrected one (the tally counts it as printed).

Full table (ours: mean ± s.d. (range) | Epstein: mean [95% CI] (range) | C in CI, C in range, D in CI, D in range | t_C t_D):

```
T R | ours C mean±sd (lo,hi) | E C mean [ci] (range) | ours D | E D | C in CI, C in range, D in CI, D in range | t_C t_D
10 9 | C 750.2 ± 25.4 (699, 814) | E 809 [802, 816] (772, 845) | D 149.7 ± 25.4 (86, 201) | E 77 [71, 83] (43, 109) | false false false false | 10.16 -13.03 
10 8 | C 699.6 ± 26.1 (643, 759) | E 748 [738, 757] (707, 802) | D 198.7 ± 25.7 (141, 255) | E 132 [124, 140] (82, 169) | false false false false | 7.06 -10.60 
10 7 | C 630.6 ± 50.4 (496, 700) | E 654 [641, 668] (568, 717) | D 262.8 ± 45.0 (198, 378) | E 198 [188, 209] (154, 280) | false true false true | 2.03 -6.56 
10 6 | C 423.8 ± 69.0 (323, 580) | E 469 [447, 490] (312, 604) | D 409.0 ± 41.9 (301, 477) | E 274 [264, 285] (191, 329) | false true false false | 2.71 -14.35 
10 5 | C 177.5 ± 68.1 (0, 326) | E 258 [240, 277] (150, 383) | D 353.5 ± 84.8 (0, 493) | E 270 [261, 279] (202, 319) | false true false false | 5.18 -5.16 
10 4 | C 102.6 ± 83.4 (0, 299) | E 231 [177, 285] (0, 598) | D 222.5 ± 126.9 (0, 462) | E 199 [172, 225] (0, 287) | false true true true | 4.08 -0.87 
10 3 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 1) | D 2.8 ± 3.6 (0, 21) | E 0 [0, 0] (0, 1) | true true false false | 0.00 -4.22 
10 2 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 2.2 ± 1.0 (1, 4) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -12.59 
10 1 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 2.7 ± 1.1 (0, 5) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -13.60 
 9 8 | C 746.4 ± 21.8 (711, 797) | E 806 [799, 814] (766, 850) | D 153.3 ± 21.5 (103, 188) | E 81 [74, 88] (39, 117) | false false false false | 10.79 -13.80 
 9 7 | C 692.2 ± 32.3 (628, 750) | E 728 [716, 740] (669, 793) | D 205.4 ± 31.5 (149, 267) | E 146 [136, 156] (86, 190) | false true false false | 4.18 -7.72 
 9 6 | C 525.7 ± 43.3 (463, 637) | E 604 [583, 625] (490, 768) | D 351.5 ± 35.8 (255, 402) | E 225 [212, 239] (102, 303) | false true false false | 5.92 -13.27 
 9 5 | C 273.8 ± 50.9 (171, 381) | E 374 [358, 390] (268, 460) | D 419.1 ± 41.7 (308, 487) | E 290 [283, 297] (252, 323) | false true false false | 8.15 -15.29 
 9 4 | C 132.4 ± 69.3 (56, 371) | E 203 [175, 231] (98, 442) | D 275.4 ± 85.7 (178, 489) | E 235 [222, 248] (159, 296) | false true false true | 3.71 -2.38 
 9 3 | C 33.5 ± 76.6 (0, 391) | E 35 [3, 67] (0, 382) | D 71.1 ± 96.2 (0, 349) | E 38 [8, 68] (0, 284) | true true false true | 0.07 -1.42 T-PASS
 9 2 | C 0.0 ± 0.2 (0, 1) | E 0 [0, 0] (0, 0) | D 2.3 ± 0.8 (1, 4) | E 0 [0, 0] (0, 0) | true true false false | -1.00 -15.82 
 9 1 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 2.7 ± 1.1 (1, 5) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -13.85 
 8 7 | C 737.6 ± 23.9 (692, 791) | E 807 [796, 818] (744, 879) | D 162.0 ± 23.9 (109, 208) | E 80 [70, 89] (14, 142) | false false false false | 9.90 -12.45 
 8 6 | C 631.4 ± 39.3 (563, 716) | E 721 [708, 734] (626, 787) | D 263.6 ± 37.5 (180, 327) | E 153 [142, 163] (98, 231) | false true false false | 9.21 -12.62 
 8 5 | C 476.3 ± 59.3 (376, 605) | E 530 [516, 544] (460, 604) | D 386.6 ± 42.9 (285, 451) | E 263 [255, 271] (209, 312) | false true false false | 4.14 -14.05 
 8 4 | C 194.9 ± 58.0 (92, 328) | E 259 [239, 279] (128, 387) | D 375.6 ± 61.0 (234, 497) | E 271 [263, 279] (227, 313) | false true false false | 4.32 -8.83 
 8 3 | C 70.9 ± 85.2 (0, 274) | E 93 [53, 134] (0, 513) | D 139.6 ± 121.9 (2, 346) | E 113 [77, 148] (0, 279) | true true true true | 0.85 -0.93 T-PASS
 8 2 | C 0.0 ± 0.2 (0, 1) | E 0 [0, 0] (0, 0) | D 2.2 ± 0.9 (0, 4) | E 0 [0, 0] (0, 0) | true true false false | -1.00 -13.63 
 8 1 | C 0.0 ± 0.2 (0, 1) | E 0 [0, 0] (0, 0) | D 2.5 ± 1.2 (0, 5) | E 0 [0, 0] (0, 0) | true true false false | -1.00 -11.30 
 7 6 | C 704.8 ± 24.7 (663, 764) | E 797 [787, 807] (739, 852) | D 194.3 ± 24.1 (136, 235) | E 88 [79, 97] (43, 133) | false false false false | 13.54 -16.76 
 7 5 | C 582.6 ± 33.6 (504, 645) | E 668 [652, 684] (542, 782) | D 305.9 ± 29.9 (247, 375) | E 187 [175, 199] (101, 271) | false true false false | 8.45 -14.64 
 7 4 | C 357.3 ± 95.5 (0, 587) | E 430 [410, 449] (323, 547) | D 407.1 ± 84.1 (2, 493) | E 286 [277, 295] (244, 345) | false true false false | 3.61 -7.54 
 7 3 | C 128.8 ± 60.6 (12, 366) | E 126 [90, 162] (0, 370) | D 267.0 ± 71.1 (43, 412) | E 153 [126, 180] (0, 267) | true true false true | -0.13 -6.00 
 7 2 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 3.1 ± 2.7 (1, 16) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -6.24 
 7 1 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 2.4 ± 0.9 (1, 4) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -15.53 
 6 5 | C 729.2 ± 17.4 (698, 778) | E 779 [773, 784] (752, 806) | D 170.5 ± 17.3 (122, 201) | E 121 [115, 126] (93, 148) | false false false false | 11.86 -11.83 
 6 4 | C 633.9 ± 30.3 (581, 717) | E 587 [576, 599] (524, 658) | D 260.6 ± 29.3 (178, 311) | E 241 [233, 248] (193, 278) | false true false true | -5.73 -2.92 
 6 3 | C 367.5 ± 39.5 (316, 481) | E 266 [247, 285] (120, 344) | D 419.6 ± 25.1 (359, 463) | E 280 [270, 291] (199, 320) | false false false false | -8.41 -19.92 
 6 2 | C 11.9 ± 28.0 (0, 89) | E 24 [0, 57] (0, 482) | D 34.9 ± 75.4 (1, 283) | E 13 [0, 27] (0, 166) | true true false true | 0.69 -1.43 T-PASS
 6 1 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 2.4 ± 0.9 (1, 4) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -14.25 
 5 4 | C 726.4 ± 21.3 (673, 758) | E 741 [729, 752] (689, 810) | D 173.0 ± 21.0 (140, 224) | E 136 [126, 146] (79, 179) | false true false true | 2.04 -5.79 
 5 3 | C 551.5 ± 32.0 (497, 605) | E 473 [456, 489] (379, 574) | D 331.2 ± 28.5 (288, 380) | E 283 [274, 291] (243, 329) | false true false false | -7.68 -7.08 
 5 2 | C 93.9 ± 79.0 (0, 260) | E 125 [70, 180] (0, 589) | D 194.0 ± 146.8 (2, 463) | E 114 [82, 145] (0, 260) | true true false true | 0.98 -2.55 
 5 1 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 2.6 ± 1.0 (1, 5) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -14.47 
 4 3 | C 688.6 ± 22.7 (631, 723) | E 710 [693, 727] (613, 814) | D 209.4 ± 22.5 (175, 268) | E 159 [146, 172] (76, 231) | false true false true | 2.24 -6.50 
 4 2 | C 414.2 ± 146.0 (0, 553) | E 282 [262, 302] (171, 380) | D 338.9 ± 117.5 (0, 417) | E 265 [254, 376] (176, 322) | false false true false | -4.63 -3.32 
 4 1 | C 0.0 ± 0.0 (0, 0) | E 0 [0, 0] (0, 0) | D 2.7 ± 1.1 (0, 6) | E 0 [0, 0] (0, 0) | true true false false | 0.00 -13.10 
 3 2 | C 611.3 ± 117.7 (0, 680) | E 624 [610, 637] (516, 699) | D 254.0 ± 52.3 (2, 306) | E 221 [210, 232] (164, 284) | true true false true | 0.56 -3.00 
 3 1 | C 9.1 ± 26.2 (0, 119) | E 4 [0, 12] (0, 123) | D 29.2 ± 74.7 (1, 345) | E 7 [0, 20] (0, 197) | true true false true | -0.81 -1.48 T-PASS
 2 1 | C 332.1 ± 240.4 (0, 558) | E 361 [344, 377] (256, 471) | D 241.3 ± 172.6 (2, 404) | E 280 [273, 288] (234, 337) | false true false true | 0.65 1.22 T-PASS
means inside CI: 22/90; inside range: 51/90; cells with both in CI: 1/45; both in range: 14/45; cells passing both t-tests: 5/45
```

### Runs 3–5

**Run 3 (R = 2, max age 100).** 339.0 ± 161.8 (0, 543) cooperators, 209.3 ± 97.7 (0, 283) defectors; 25 of 30 coexist and 5 populations die out by 500 (same counts at 2,000 and 5,000). Epstein gives no numbers ("cooperators do worse … defectors do better, and the oscillations are now more evident"); RR report 250–450 cooperators and about 200 defectors — consistent.

**Run 4 (R = 1, max age 100).** Outcomes (coexistence / cooperator monopoly / defectors only / extinction): **t = 500: 4 / 0 / 0 / 26; t = 2,000: 0 / 0 / 0 / 30; t = 5,000: 0 / 0 / 0 / 30.** The populations collapse within about 100 cycles (seed 1: 49 C / 51 D at 0, 4 / 22 at 20, 0 / 11 at 40, 0 / 0 at 100; seeds 2 and 3 alike).
- *Cycle count* (defined here): a swing = the cooperator count rising above 400 and later falling below 100 (hysteresis, so noise near one threshold does not count); counted over cycles 0–2,000. **Mean 0.6 ± 0.8 swings per run (0–2)**; with thresholds 300/150, 0.9 ± 1.2 (0–4). Epstein's Fig. 9.8 shows a cycle about every 300–500 cycles (panels at 40, 60, 160, 240, 560, 625, 700) — at least three swings in 1,000 cycles. No run here sustains predator–prey cycles; at most two swings before extinction.
- *Seed-dependent outcomes*: only "coexistence then extinction" occurs; no cooperator monopoly at R = 1 in any seed.
- *The R = 1 paradox* ("cooperators ultimately do better with a low payoff (R = 1) than with a high one (R = 5)"): cooperator monopolies at t = 2,000 — **R = 1: 0/30; R = 5: 0/30** (all 30 coexist at R = 5, C 678 ± 30, D 212 ± 28). Not reproduced; there is nothing to compare.
- *Across the six timing switches* (64 combinations × 30 seeds, R = 1, max age 100, `out-run4f.txt`): asynchronous settings mostly die out (0–6 of 30 coexist at 500); synchronous ones mostly coexist (10–29 of 30) but without cycles (mean 0.0–1.4 swings); cooperator monopolies occur in at most 2 of 30 runs in any combination (e.g. F F T T T F: 2 at 500). With no maximum age (R = 1, Run 1's settings) 16 coexist, 2 monopolies, 12 defectors-only at 500; at 2,000: 2 monopolies, 19 lone-defector runs, 9 extinct. With the working paper's rule: 1 monopoly, 29 extinct by 2,000.

**Run 5 (Run 2 + 50% mutation), 10,000 cycles.** Cooperators at 10,000 > 0 in **27 of 30** runs; the other 3 populations die out entirely (population 0). Minimum cooperator count over each run 42 ± 21 (0, 73); at 10,000: 250 ± 94 cooperators, 289 ± 102 defectors; mean over 5,001–10,000: 260 ± 88 / 295 ± 100. At 500: 267.6 ± 87.2 (0, 362) / 306.3 ± 91.3 (0, 385). "Persistence through 10,000 cycles": **reproduced in 27/30** (Epstein shows one run). Cooperators are fewer than defectors (Epstein: "the cooperator-defector ratio closer to 1"). 25% mutation, mean over cycles 1,001–2,000: **420 ± 5 cooperators, 393 ± 2 defectors** (Epstein: "around 350 and … around 400").

### Soup, the shifted payoffs, metabolism, footnote 27

**Soup** (`pairing: soup`, Run 1): the last cooperator dies at cycle 7.6 ± 2.6 (4–14) in 29 of 30 runs; at 500, 19 runs hold one lone defector, 10 are empty, 1 holds one lone cooperator (the last agent standing). With max age 100: 30/30 empty by 500, cooperators gone by 7.7 ± 2.3 (4–15). **Reproduced** ("runs to pure defection"), then extinction as Epstein's negative-payoff argument predicts.

**Shifted payoffs (12, 11, 1, 0).** Which settings? GSS p. 216 states them: "with maximum age of 100, zero mutation, and all payoffs shifted up by 6" — Run 2's settings. The working paper (p. 11) says only "with all payoffs shifted up by 6", right after Run 5, and the metabolism passage calls the shifted run "the immediately preceding run (figure 12)" — Fig. 12 is Run 5's 50% mutation. With 50% mutation pure defection is impossible (half of every defector's offspring cooperate), so that reading cannot produce Fig. 13's "converges to pure defection". **Reading chosen: GSS's explicit Run 2 settings** (`dpd-shifted` = max age 100, mutation 0; the spec said Run 5's — see Decision 10). Measured, all three readings:
- Run 2 settings (GSS): coexistence in **30/30** at 500 (C 418.1 ± 77.5 (264, 574), D 478.1 ± 77.3) and 30/30 at 2,000 (C 409 ± 106, D 487 ± 106); cooperators never reach 0.
- Run 5 settings (WP context): 30/30 coexist, C 448.2 ± 13.1, D 448.5 ± 13.2.
- Run 1 settings (no max age): 30/30 coexist, C 423.5 ± 74.4, D 476.5 ± 74.4.
**Not reproduced in any reading.** Why: with no negative payoff only old age kills; the lattice fills (≈ 896 agents) and every agent's wealth quickly exceeds the fission threshold (a cooperator among cooperators earns up to 88 a cycle, an offspring costs 6), so a vacated site goes to whichever neighbour takes its turn first — near-neutral drift, not selection for defection.

**Metabolism** (the "equivalent mathematically" claim). Shifted payoffs, Run 2 settings, cooperators / defectors at 500:

| metabolism | per cycle | per game |
|---|---|---|
| 0 | 418.1 / 478.1 | 418.1 / 478.1 |
| 1 | 414.1 / 482.7 | 339.1 / 549.9 |
| 2 | 435.4 / 461.2 | 380.3 / 498.1 |
| 3 | 494.6 / 401.7 | 514.0 / 378.6 |
| 4 | 611.5 / 285.2 | 676.7 / 217.8 |
| 5 | 661.1 / 235.0 | 713.9 / 179.4 |
| 6 | 644.2 ± 97.4 / 252.6 ± 97.0 | **694.8 / 195.9 = Run 2 exactly** |

Per game at 6 is Run 2 bit for bit (all 30 fingerprints equal at 500: each game's +payoff − 6 is the original payoff, and nothing else draws or differs). Per cycle (WP note 29's definition, and the default) at 6: cooperative, but a different model (t = 2.7 against Run 2's cooperators, with a much wider spread: s.d. 97 against 29). So the claim holds exactly for the text's other phrase ("imposed on all agents after every interaction") and only approximately for its note's "per cycle". Epstein's premise that at metabolism 0 "cooperators are annihilated" fails (418 cooperators). The "necessity is the mother of cooperation" trend (more metabolism, more cooperators) does hold from 1 to 5. On Run 5's settings (50% mutation) nothing changes much: per cycle 426–448 cooperators at every metabolism; per game at 6 = Run 5 exactly (267.6 / 306.3).

**Footnote 27** ("payoffs hiked by ten (so that T = 16, R = 11, P = 5, S = 4) and the maximum lifetime reduced from 100 to 10 cycles … an evolution to cooperative monopoly"). Hiked by ten, R would be 15: a slip in the text; both measured. Max age 10: (16, 11, 5, 4): **30/30 coexist at 500** (C 384.5 ± 157.7 (101, 719), D 484.6 ± 158.1); at 2,000: 27 coexist, **1 cooperative monopoly**, 2 defectors-only. (16, 15, 5, 4): 30/30 coexist at 500 (C 395 ± 172); at 2,000: 28 coexist, 1 monopoly, 1 defectors-only. Max age 100 (for contrast): 30/30 coexist at 500 and 2,000 in both. "Using the same random seed" suggests one run: 1 of 30 seeds (by 2,000) gives a monopoly. **Not reproduced as a typical outcome.**

### The coordination game (GSS appendix)

Payoffs [1, −3, −3, 1] as (CC, CD, DC, DD): R 1, S −3, T −3, P 1; max age 1,000; mutation 0; other settings Run 1's. The "cooperator" strategy is one convention, "defector" the other. **Measure** (defined here): (a) whether both conventions are present at cycle t; (b) among runs where both are, the share of neighbouring (von Neumann) occupied pairs that differ, per mille, against the share expected if the same counts were mixed at random (2pq). Regions = both present with few unlike neighbours.

| t | both present | minority ≥ 10% | unlike pairs ‰ (mixed runs) | if mixed at random ‰ | minority share % |
|---|---|---|---|---|---|
| 100 | 18/30 | 16/30 | 36.0 ± 9.4 (14, 48) | 346 ± 126 | 25.1 ± 12.5 |
| 500 | 17/30 | 15/30 | 36.5 ± 8.7 (19, 48) | 347 ± 117 | 25.1 ± 12.5 |
| 2,000 | 16/30 | 11/30 | 34.5 ± 9.6 (13, 47) | 307 ± 147 | 22.6 ± 15.0 |
| 5,000 | 11/30 | 6/30 | 29.2 ± 12.7 (9, 43) | 279 ± 203 | 22.6 ± 19.7 |

Population ≈ 899.5 (the lattice is full). **Partly reproduced**: where both conventions persist they form regions (unlike neighbours ~3.5% against ~30–35% if mixed); but about half the runs settle on one convention by 500, and the mixed runs keep being lost (18 → 11 of 30 from 100 to 5,000) — persistent over hundreds of cycles, eroding over thousands.

### Radax & Rengs' factorial (the six model switches × 30 seeds, Runs 1 and 2)

RR's switches in their order (TRUE first): remove dead immediately (`removal: immediate` / `end_of_cycle`), die immediately (`death_timing: immediate` / `own_turn`), initial endowment inherited (`endowment_from: parent` / `granted`), random birth age (`newborn_age: random` / `zero`; RR's FALSE is age 1), asynchronous (`updating`), Repast list shuffle (`shuffle: full` / `swaps`). RNG library excluded. Row n here = RR's rows with their CERN column dropped (our 1 = TTTTTT). Test: RR's pooled t at 2.0017 on both cooperators and defectors.

**Result: Run 1: 0 of 64; Run 2: 1 of 64 (T F T T T T: removal at once, death on the agent's own turn, endowment from the parent, random newborn age, asynchronous, full shuffle: 785 ± 23 / 110 ± 22, t = −0.09 / −1.85); both: 0.** Welch's df gives the same counts. RR (MPRA text, p. 11): **1 of 128 reproduces Run 1** (a synchronous setting, 785 (16) / 114 (16), which they set aside as "radically different from the original model") — the spec's summary "0 of 128" follows their dismissal, not their count; **7 of 128 reproduce Run 2; none both.** Ours: 0 / 1 / none. RR's best Run 2 fit (F T F T T F: 780 (25) / 97 (22) in Repast) gives **703.0 ± 26.3 / 163.5 ± 23.2 here (t = 11.3 / −10.4)** — it does not carry over; our only Run 2 fit (T F T T T T) is not among their seven. Our best Run 1 fits (max |t| = 4.7) are synchronous with own-turn death (e.g. T F F F F F: 796 / 104): synchronous settings overshoot cooperators (796–812) unless death and removal are both immediate (678–719); asynchronous ones undershoot (671–732); Epstein's 779 falls in the gap. Our model and RR's differ in details RR's pseudo-code fixes and the spec does not (RR play neighbours "in random order" and check deaths only after all games; they age everyone at the end of the period and kill at "age ≥ maximum age"; whether their dead bodies are played against is not stated).

Full table (R1 / R2: cooperators (s.d.) t, defectors (s.d.) t; PASS = both t pass):

```
 1 TTTTTT | R1 C 730 (19) t 11.12 D 169 (19) t -11.07  | R2 C 684 (29) t 13.42 D 209 (28) t -15.95 
 2 TTTTTF | R1 C 729 (17) t 11.86 D 170 (17) t -11.83  | R2 C 695 (29) t 11.83 D 196 (28) t -14.09 
 3 TTTTFT | R1 C 719 (30) t 9.91 D 181 (30) t -9.90  | R2 C 460 (22) t 49.13 D 407 (17) t -56.29 
 4 TTTTFF | R1 C 712 (25) t 12.72 D 188 (25) t -12.73  | R2 C 453 (23) t 48.83 D 410 (17) t -56.14 
 5 TTTFTT | R1 C 730 (19) t 11.12 D 169 (19) t -11.07  | R2 C 658 (25) t 18.11 D 237 (25) t -21.48 
 6 TTTFTF | R1 C 729 (17) t 11.86 D 170 (17) t -11.83  | R2 C 670 (27) t 15.70 D 225 (27) t -18.77 
 7 TTTFFT | R1 C 719 (30) t 9.91 D 181 (30) t -9.90  | R2 C 473 (26) t 44.04 D 413 (22) t -52.00 
 8 TTTFFF | R1 C 712 (25) t 12.72 D 188 (25) t -12.73  | R2 C 468 (23) t 46.84 D 414 (20) t -53.98 
 9 TTFTTT | R1 C 707 (18) t 16.65 D 193 (18) t -16.60  | R2 C 661 (29) t 16.52 D 230 (27) t -19.38 
10 TTFTTF | R1 C 709 (19) t 15.99 D 191 (19) t -16.01  | R2 C 676 (29) t 14.33 D 213 (27) t -16.93 
11 TTFTFT | R1 C 687 (34) t 13.72 D 213 (34) t -13.70  | R2 C 418 (24) t 53.53 D 442 (16) t -63.63 
12 TTFTFF | R1 C 678 (28) t 17.56 D 222 (28) t -17.45  | R2 C 415 (21) t 56.22 D 448 (21) t -59.01 
13 TTFFTT | R1 C 707 (18) t 16.65 D 193 (18) t -16.60  | R2 C 629 (20) t 24.12 D 266 (20) t -28.63 
14 TTFFTF | R1 C 709 (19) t 15.99 D 191 (19) t -16.01  | R2 C 639 (22) t 21.70 D 254 (21) t -26.30 
15 TTFFFT | R1 C 687 (34) t 13.72 D 213 (34) t -13.70  | R2 C 430 (22) t 52.80 D 449 (20) t -59.78 
16 TTFFFF | R1 C 678 (28) t 17.56 D 222 (28) t -17.45  | R2 C 420 (22) t 54.60 D 457 (19) t -62.64 
17 TFTTTT | R1 C 732 (17) t 11.34 D 168 (17) t -11.32  | R2 C 785 (23) t -0.09 D 110 (22) t -1.85 PASS
18 TFTTTF | R1 C 716 (18) t 14.86 D 184 (18) t -14.76  | R2 C 756 (31) t 3.60 D 136 (29) t -5.25 
19 TFTTFT | R1 C 812 (15) t -8.47 D 88 (15) t 8.51  | R2 C 853 (14) t -11.79 D 29 (14) t 13.54 
20 TFTTFF | R1 C 808 (12) t -8.39 D 92 (12) t 8.44  | R2 C 853 (20) t -10.69 D 28 (20) t 11.99 
21 TFTFTT | R1 C 732 (17) t 11.34 D 168 (17) t -11.32  | R2 C 700 (30) t 11.02 D 196 (29) t -14.02 
22 TFTFTF | R1 C 716 (18) t 14.86 D 184 (18) t -14.76  | R2 C 678 (26) t 15.01 D 216 (25) t -18.33 
23 TFTFFT | R1 C 812 (15) t -8.47 D 88 (15) t 8.51  | R2 C 864 (11) t -14.16 D 35 (10) t 12.98 
24 TFTFFF | R1 C 808 (12) t -8.39 D 92 (12) t 8.44  | R2 C 862 (14) t -13.17 D 37 (14) t 11.80 
25 TFFTTT | R1 C 710 (16) t 17.39 D 190 (16) t -17.26  | R2 C 755 (38) t 3.37 D 139 (36) t -4.98 
26 TFFTTF | R1 C 702 (18) t 18.28 D 198 (18) t -18.21  | R2 C 744 (38) t 4.55 D 148 (36) t -6.11 
27 TFFTFT | R1 C 799 (15) t -5.28 D 101 (15) t 5.29  | R2 C 851 (16) t -11.02 D 30 (17) t 12.53 
28 TFFTFF | R1 C 796 (14) t -4.67 D 104 (14) t 4.70  | R2 C 853 (16) t -11.44 D 30 (15) t 13.13 
29 TFFFTT | R1 C 710 (16) t 17.39 D 190 (16) t -17.26  | R2 C 671 (22) t 17.01 D 225 (22) t -20.85 
30 TFFFTF | R1 C 702 (18) t 18.28 D 198 (18) t -18.21  | R2 C 649 (25) t 19.12 D 242 (23) t -22.86 
31 TFFFFT | R1 C 799 (15) t -5.28 D 101 (15) t 5.29  | R2 C 850 (19) t -10.37 D 49 (19) t 8.84 
32 TFFFFF | R1 C 796 (14) t -4.67 D 104 (14) t 4.70  | R2 C 855 (16) t -11.74 D 43 (16) t 10.35 
33 FTTTTT | R1 C 727 (23) t 10.34 D 171 (22) t -10.15  | R2 C 731 (35) t 6.42 D 140 (31) t -5.60 
34 FTTTTF | R1 C 710 (21) t 14.63 D 187 (20) t -14.31  | R2 C 734 (31) t 6.51 D 135 (27) t -5.44 
35 FTTTFT | R1 C 811 (12) t -9.04 D 89 (12) t 9.06  | R2 C 868 (11) t -14.75 D 14 (11) t 17.09 
36 FTTTFF | R1 C 812 (13) t -9.13 D 87 (13) t 9.17  | R2 C 872 (11) t -15.42 D 10 (10) t 18.16 
37 FTTFTT | R1 C 727 (23) t 10.34 D 171 (22) t -10.15  | R2 C 651 (34) t 16.16 D 226 (29) t -18.02 
38 FTTFTF | R1 C 710 (21) t 14.63 D 187 (20) t -14.31  | R2 C 642 (27) t 19.64 D 231 (22) t -21.66 
39 FTTFFT | R1 C 811 (12) t -9.04 D 89 (12) t 9.06  | R2 C 874 (12) t -15.76 D 25 (11) t 14.86 
40 FTTFFF | R1 C 812 (13) t -9.13 D 87 (13) t 9.17  | R2 C 871 (11) t -15.41 D 28 (10) t 14.40 
41 FTFTTT | R1 C 696 (21) t 17.50 D 201 (21) t -17.13  | R2 C 708 (35) t 9.28 D 160 (31) t -8.44 
42 FTFTTF | R1 C 687 (26) t 16.80 D 209 (25) t -16.65  | R2 C 703 (26) t 11.34 D 163 (23) t -10.35 
43 FTFTFT | R1 C 804 (13) t -6.95 D 96 (13) t 7.02  | R2 C 867 (10) t -14.88 D 15 (9) t 17.35 
44 FTFTFF | R1 C 804 (15) t -6.37 D 96 (15) t 6.45  | R2 C 870 (13) t -14.79 D 14 (11) t 17.06 
45 FTFFTT | R1 C 696 (21) t 17.50 D 201 (21) t -17.13  | R2 C 623 (25) t 22.98 D 249 (21) t -25.35 
46 FTFFTF | R1 C 687 (26) t 16.80 D 209 (25) t -16.65  | R2 C 603 (26) t 25.65 D 265 (19) t -28.77 
47 FTFFFT | R1 C 804 (13) t -6.95 D 96 (13) t 7.02  | R2 C 868 (13) t -14.49 D 31 (13) t 13.37 
48 FTFFFF | R1 C 804 (15) t -6.37 D 96 (15) t 6.45  | R2 C 870 (11) t -15.10 D 29 (11) t 13.99 
49 FFTTTT | R1 C 700 (22) t 16.08 D 197 (21) t -15.87  | R2 C 690 (36) t 11.18 D 177 (30) t -10.87 
50 FFTTTF | R1 C 689 (25) t 16.93 D 206 (23) t -16.84  | R2 C 702 (38) t 9.39 D 166 (33) t -8.83 
51 FFTTFT | R1 C 812 (15) t -8.47 D 88 (15) t 8.51  | R2 C 853 (14) t -11.79 D 29 (14) t 13.54 
52 FFTTFF | R1 C 808 (12) t -8.39 D 92 (12) t 8.44  | R2 C 853 (20) t -10.69 D 28 (20) t 11.99 
53 FFTFTT | R1 C 700 (22) t 16.08 D 197 (21) t -15.87  | R2 C 617 (31) t 21.64 D 257 (28) t -23.07 
54 FFTFTF | R1 C 689 (25) t 16.93 D 206 (23) t -16.84  | R2 C 604 (27) t 24.74 D 266 (24) t -26.66 
55 FFTFFT | R1 C 812 (15) t -8.47 D 88 (15) t 8.51  | R2 C 864 (11) t -14.16 D 35 (10) t 12.98 
56 FFTFFF | R1 C 808 (12) t -8.39 D 92 (12) t 8.44  | R2 C 862 (14) t -13.17 D 37 (14) t 11.80 
57 FFFTTT | R1 C 674 (21) t 21.91 D 222 (21) t -21.04  | R2 C 678 (38) t 12.11 D 189 (33) t -11.96 
58 FFFTTF | R1 C 671 (23) t 21.37 D 223 (23) t -20.40  | R2 C 671 (29) t 15.01 D 194 (26) t -14.29 
59 FFFTFT | R1 C 799 (15) t -5.28 D 101 (15) t 5.29  | R2 C 851 (16) t -11.02 D 30 (17) t 12.53 
60 FFFTFF | R1 C 796 (14) t -4.67 D 104 (14) t 4.70  | R2 C 853 (16) t -11.44 D 30 (15) t 13.13 
61 FFFFTT | R1 C 674 (21) t 21.91 D 222 (21) t -21.04  | R2 C 586 (24) t 28.55 D 284 (20) t -31.58 
62 FFFFTF | R1 C 671 (23) t 21.37 D 223 (23) t -20.40  | R2 C 577 (27) t 28.77 D 290 (22) t -31.29 
63 FFFFFT | R1 C 799 (15) t -5.28 D 101 (15) t 5.29  | R2 C 850 (19) t -10.37 D 49 (19) t 8.84 
64 FFFFFF | R1 C 796 (14) t -4.67 D 104 (14) t 4.70  | R2 C 855 (16) t -11.74 D 43 (16) t 10.35 
Run 1 pass (pooled): 0/64; Run 2 pass: 1/64; both: 0/64; Welch: R1 0 R2 1
Best Run 2 fits (max |t|): [("TFTTTT", 11.337322585768948, 1.8463939868106334), ("TFFTTT", 17.393834106900506, 4.984538703867131), ("TFTTTF", 14.861841249071597, 5.248352326932929), ("TFFTTF", 18.283965624064727, 6.1079142620762665), ("FTTTTT", 10.336020848727825, 6.422851274685204)]
Best Run 1 fits (max |t|): [("TFFFFF", 4.69523823121254, 11.742101790737243), ("FFFFFF", 4.69523823121254, 11.742101790737243), ("TFFTFF", 4.69523823121254, 13.132819288817204), ("FFFTFF", 4.69523823121254, 13.132819288817204), ("TFFFFT", 5.289988038235253, 10.365935652481848)]
```

### The working paper's rule against the published rule

`play: random_neighbor` (one game with one random occupied neighbour a turn), everything else default:

| | Cooperators | Defectors | t (C / D) |
|---|---|---|---|
| Run 1, published rule | 729.2 ± 17.4 | 170.5 ± 17.3 | 11.86 / −11.83 |
| Run 1, working paper | 758.9 ± 17.5 (711, 784) | 141.0 ± 17.6 (116, 189) | 4.78 / −4.74 |
| Run 2, published rule | 694.8 ± 29.4 | 195.9 ± 28.2 | 11.83 / −14.09 |
| Run 2, working paper | 737.0 ± 21.2 (692, 792) | 156.2 ± 20.8 (102, 200) | 7.16 / −9.63 |

The working paper's rule is nearer both tables, but still rejected alone.

### Beyond RR: the choices no source settles (512 readings)

One-at-a-time from the defaults (Run 1 | Run 2, cooperators / defectors):
- newborns act this cycle: 736 / 163 | 756 / 132
- full shuffle: 731 / 169 | 684 / 209
- synchronous: 712 / 188 | 453 / 410
- end-of-cycle removal: 710 / 187 | 734 / 135
- own-turn death: 716 / 184 | 756 / 136
- granted endowment: 709 / 191 | 676 / 213
- **initial wealth 0**: **758 / 141** | 687 ± 190 / 147 (some runs die out)
- fission at 10 instead of 11: 737 / 163 | 695 / 196

Full search (`out-search.txt`): play (each / random neighbour) × newborns act (next / this cycle) × initial wealth (6 / 0) × RR's 64 timing settings = 512 readings, 30 seeds each, RR's test. **46 reproduce Run 1, 24 Run 2, 7 both. Every one of the 46 Run 1 fits has initial wealth 0 (none with the CD's 6), and every joint fit also has the working paper's rule.** The joint fits: random neighbour, next cycle: F T F F T T, F T F F T F, F F F F T T; random neighbour, this cycle: T T T T T T, T T T T T F (the text's timing), F T F F T F, F F F F T T. Best: random neighbour, this cycle, wealth 0, F T F F T F: Run 1 777.9 ± 18.3 / 121.6 ± 18.2 (t 0.25 / −0.15); Run 2 763.8 ± 146.4 / 99.4 ± 30.4 (t 0.74 / −0.06; one run dies out). Robustness on fresh seeds (`out-joint.txt`), the text's timing (preset `dpd-closest`):

| seeds | Run 1 C / D (t) | Run 2 C / D (t) |
|---|---|---|
| 1–30 | 786.1 ± 21.1 / 113.7 ± 21.2 (−1.50 / 1.55) | 792.2 ± 24.1 / 101.3 ± 23.1 (−1.20 / −0.38) |
| 31–60 | 781.8 ± 14.0 / 118.0 ± 14.0 (−0.75 / 0.81) | 776.7 ± 27.6 / 115.8 ± 25.4 (0.99 / **−2.58**) |
| 61–90 | 760.9 ± 144.5 / 108.8 ± 25.2 (0.68 / **2.28**; one run dies out) | 771.6 ± 28.5 / 120.5 ± 26.7 (1.67 / **−3.23**) |

So Run 1 is reproduced by these readings robustly, Run 2 only marginally (its defectors run 2–20 high). With no initial wealth an agent's first game decides its life (a defector met first kills it), which thins the start; it may be what Epstein's C++ did — the CD (a 2006 Ascape reimplementation, not the original) gives 6.

### Golden fingerprints (200 cycles, seed 1; `MODEL_GOLDEN`, after `hks-no-ethnocentrics`)

dpd-run-1 0x3d64b053fbfee4f6; dpd-run-2 0xe0198124ac5f4789; dpd-run-3 0x1c819cc85b7bb351; dpd-run-4 0xe6b5d66dee22ce07; dpd-run-5 0x2f5ae2bdc6bd257a; dpd-working-paper 0xaba834120f15810b; dpd-closest 0xd1c7475297f9864c; dpd-soup 0xdb64ad16a49146b; dpd-shifted 0x39b59d546b2bb2e8; dpd-metabolism 0x7b580a83419a3ea4; dpd-footnote-27 0xd1adeadce6881068; dpd-rr-best 0xe8fdc4ce027dd236; dpd-coordination 0x47f8c68504a9157a. WASM (`wasm-pack test --node`): dpd-run-1, dpd-run-5, dpd-rr-best equal native. Every earlier golden entry unchanged. Keyframes: `tests/checkpoint.rs` adds `dpd-rr-best` (end-of-cycle removal: bodies, the list order and the empty list restore exactly).

## File Structure
**Task 1:** create `crates/sugarscape-core/src/dpd/{mod,config,stats,world,presets}.rs`; modify `crates/sugarscape-core/src/lib.rs`, `model.rs` (kind, config, world, keyframe arms, the unknown-model message, `max_ticks`, series, the round-trip test), `presets.rs` (the catalog), `sweep.rs` (the ticks message only), `crates/sugarscape-cli/src/main.rs` (the finish message), `crates/sugarscape-core/tests/golden.rs` (13 entries), `crates/sugarscape-core/tests/checkpoint.rs` (`dpd-rr-best`).
**Task 2:** create `crates/sugarscape-core/tests/dpd.rs` (14 ignored tests) and `sweeps/{dpd-payoffs,dpd-mutation,dpd-metabolism,dpd-max-age}.json`; modify `crates/sugarscape-core/src/sweep.rs` (BUILTINS and the id list), `crates/sugarscape-cli/tests/cli.rs` and `crates/sugarscape-wasm/tests/web.rs` (ids and a fingerprint test).

---

### Task 1: The demographic Prisoner's Dilemma in the core

**Files:**
- Create: `crates/sugarscape-core/src/dpd/{mod,config,stats,world,presets}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/sweep.rs` (the ticks message only), `crates/sugarscape-cli/src/main.rs` (the finish message only), `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-core/tests/checkpoint.rs`

**Interfaces:**
- Consumes: `crate::spatial::{self, Geometry}` (`SpatialConfig`, `Neighborhood::{VonNeumann, Moore}`, `Boundary::Periodic`); `crate::config::{FieldError, ScheduledChange}`; `crate::model::{wrong_model, Model, ModelConfig, ModelKind}`; `crate::presets::ModelPreset`; `crate::render::{lerp, Rgb, BACKGROUND, BLUE, COOL, HOT, RED}`; `crate::rng::{self, SimRng}`; `crate::schema::{Apply, Param}` (`with_help`, `shown_if`); `crate::stats::{Series, Stats}`; `crate::export::history_csv`; `crate::schema::check_schema` (tests).
- Produces: `dpd::{DpdConfig, MetabolismPer, Play, Pairing, DeathTiming, Removal, EndowmentFrom, NewbornAge, Updating, Shuffle, NewbornsAct, LIVE, schema, DpdSnapshot, SERIES, DpdWorld, DpdMode, Agent, DpdInspection, AgentView, NeighborView, Site, COOPERATOR, DEFECTOR, presets}`; `DpdConfig::{payoff, validate, changes}`; `DpdWorld::{new, step, run, sites, agents, agent_at, population, is_finished, surrounded, inspect, geometry, stats, config, tick}`; `ModelKind::Dpd` (`"dpd"`), `ModelConfig::Dpd`, `ModelWorld::Dpd` with keyframes; `MODEL_GOLDEN` gains thirteen entries.

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/dpd/config.rs` with only its tests for now (the implementation goes above them in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &DpdConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_table_9_1_with_the_cds_wealths_and_validates() {
        let c = DpdConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!((c.width, c.agents, c.vision, c.max_age), (30, 100, 1, 0));
        assert_eq!((c.t, c.r, c.p, c.s), (6.0, 5.0, -5.0, -6.0));
        assert_eq!(
            (c.fission_wealth, c.endowment, c.initial_wealth),
            (11.0, 6.0, 6.0)
        );
        assert_eq!((c.mutation, c.metabolism, c.end), (0.0, 0.0, 0));
        let v = serde_json::to_value(&c).unwrap();
        assert_eq!(v["play"], "each_neighbor");
        assert_eq!(v["death_timing"], "immediate");
        assert_eq!(v["removal"], "immediate");
        assert_eq!(v["endowment_from"], "parent");
        assert_eq!(v["newborn_age"], "random");
        assert_eq!(v["updating"], "asynchronous");
        assert_eq!(v["shuffle"], "swaps");
        assert_eq!(v["newborns_act"], "next_cycle");
        assert_eq!(v["metabolism_per"], "cycle");
        assert_eq!(v["pairing"], "space");
    }

    #[test]
    fn payoffs_follow_the_matrix() {
        let c = DpdConfig::default();
        assert_eq!(c.payoff(true, true), 5.0);
        assert_eq!(c.payoff(true, false), -6.0);
        assert_eq!(c.payoff(false, true), 6.0);
        assert_eq!(c.payoff(false, false), -5.0);
    }

    #[test]
    fn validation_names_the_field() {
        let bad = |edit: &dyn Fn(&mut DpdConfig)| {
            let mut c = DpdConfig::default();
            edit(&mut c);
            fields(&c)
        };
        assert_eq!(bad(&|c| c.width = 2), ["width"]);
        assert_eq!(bad(&|c| c.width = 201), ["width"]);
        assert_eq!(bad(&|c| c.agents = 901), ["agents"]);
        assert!(bad(&|c| c.agents = 900).is_empty());
        assert_eq!(
            bad(&|c| c.initial_cooperators = 1.5),
            ["initial_cooperators"]
        );
        assert_eq!(bad(&|c| c.initial_wealth = -1.0), ["initial_wealth"]);
        assert_eq!(bad(&|c| c.t = f64::NAN), ["t"]);
        assert_eq!(bad(&|c| c.r = f64::INFINITY), ["r"]);
        assert_eq!(bad(&|c| c.p = f64::NAN), ["p"]);
        assert_eq!(bad(&|c| c.s = f64::NEG_INFINITY), ["s"]);
        assert_eq!(bad(&|c| c.fission_wealth = -1.0), ["fission_wealth"]);
        assert_eq!(bad(&|c| c.endowment = -0.5), ["endowment"]);
        assert_eq!(bad(&|c| c.metabolism = -1.0), ["metabolism"]);
        assert_eq!(bad(&|c| c.mutation = 1.1), ["mutation"]);
        assert_eq!(bad(&|c| c.vision = 0), ["vision"]);
        assert_eq!(bad(&|c| c.vision = 11), ["vision"]);
        // No ordering or sign is imposed on the payoffs.
        assert!(bad(&|c| {
            c.t = -3.0;
            c.r = 1.0;
            c.p = 1.0;
            c.s = -3.0;
        })
        .is_empty());
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = DpdConfig {
            schedule: vec![entry("r", json!(1.0))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("width", json!(40))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("mutation", json!(2))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = DpdConfig::default();
        let mut b = a.clone();
        b.r = 1.0;
        b.max_age = 100;
        b.updating = Updating::Synchronous;
        assert!(a.changes(&b).is_empty());
        b.width = 40;
        b.vision = 2;
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["vision", "width"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: DpdConfig =
            serde_json::from_str(r#"{"max_age": 100, "removal": "end_of_cycle"}"#).unwrap();
        assert_eq!(
            (c.max_age, c.removal, c.width),
            (100, Removal::EndOfCycle, 30)
        );
        assert!(serde_json::from_str::<DpdConfig>(r#"{"threshold": 10}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Dpd(DpdConfig::default());
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

Create `crates/sugarscape-core/src/dpd/world.rs` with only its tests (they use the private `add`, `move_agent`, `play`, `reproduce`, `age_and_die`, `turn`, `begin_cycle`, `end_cycle`, `record`, `relocate`, `vacate`, `newborn_strategy`, `reach` and the `agents`, `at`, `empty`, `slot`, `living`, `births`, `deaths` fields):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduledChange;
    use serde_json::json;

    /// An empty `w` × `w` world (no initial agents), after `edit`.
    fn world(w: u32, edit: impl FnOnce(&mut DpdConfig)) -> DpdWorld {
        let mut c = DpdConfig {
            width: w,
            agents: 0,
            ..Default::default()
        };
        edit(&mut c);
        DpdWorld::new(c, 1).unwrap()
    }

    /// Puts an agent aged 0 at (x, y) at the end of the list; returns its index.
    fn put(w: &mut DpdWorld, (x, y): (u32, u32), cooperator: bool, wealth: f64) -> usize {
        let s = w.geometry.at(x, y, 0).unwrap();
        w.add(s, cooperator, wealth, 0);
        w.agents.len() - 1
    }

    fn site(w: &DpdWorld, (x, y): (u32, u32)) -> usize {
        w.geometry.at(x, y, 0).unwrap()
    }

    fn xy(w: &DpdWorld, k: usize) -> (u32, u32) {
        let (x, y, _) = w.geometry.xyz(w.agents[k].site as usize);
        (x, y)
    }

    /// The empty-site list holds exactly the unoccupied sites, each at its
    /// slot, and every living agent's site points back at it.
    fn empty_list_is_consistent(w: &DpdWorld) {
        let unoccupied = w.at.iter().filter(|&&k| k == NONE).count();
        assert_eq!(w.empty.len(), unoccupied);
        for (k, &s) in w.empty.iter().enumerate() {
            assert_eq!(w.at[s as usize], NONE, "listed site {s} is occupied");
            assert_eq!(w.slot[s as usize], k as u32);
        }
        for (k, a) in w.agents.iter().enumerate() {
            if !a.dead {
                assert_eq!(w.at[a.site as usize], k as u32, "agent {}", a.id);
            }
        }
    }

    #[test]
    fn vision_reaches_every_distinct_site_within_its_von_neumann_distance() {
        assert_eq!(reach(30, 1), [(0, 29), (29, 0), (1, 0), (0, 1)]);
        assert_eq!(reach(30, 2).len(), 12);
        assert_eq!(reach(30, 10).len(), 220);
        // On a 3 × 3 torus distance 2 reaches all eight other sites once.
        assert_eq!(reach(3, 2).len(), 8);
    }

    #[test]
    fn movers_go_to_an_unoccupied_site_within_vision_or_stay() {
        let mut w = world(5, |_| {});
        let a = put(&mut w, (2, 2), true, 100.0);
        for p in [(2, 1), (1, 2), (3, 2)] {
            put(&mut w, p, true, 100.0);
        }
        w.move_agent(a);
        assert_eq!(xy(&w, a), (2, 3), "the one unoccupied neighbour");
        let mut w = world(5, |_| {});
        let a = put(&mut w, (2, 2), true, 100.0);
        for p in [(2, 1), (1, 2), (3, 2), (2, 3)] {
            put(&mut w, p, true, 100.0);
        }
        w.move_agent(a);
        assert_eq!(xy(&w, a), (2, 2), "blocked: stays");
        empty_list_is_consistent(&w);
        // Vision 2 with the four neighbours taken: one of the eight sites at
        // distance 2, each in turn.
        let mut w = world(7, |c| c.vision = 2);
        let a = put(&mut w, (3, 3), true, 100.0);
        for p in [(3, 2), (2, 3), (4, 3), (3, 4)] {
            put(&mut w, p, true, 100.0);
        }
        let home = site(&w, (3, 3));
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..400 {
            w.move_agent(a);
            let (x, y) = xy(&w, a);
            assert_eq!(x.abs_diff(3) + y.abs_diff(3), 2, "({x}, {y})");
            seen.insert((x, y));
            w.relocate(a, home);
        }
        assert_eq!(seen.len(), 8);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_mover_plays_each_neighbour_or_one_and_both_players_are_paid() {
        let mut w = world(5, |_| {});
        let me = put(&mut w, (2, 2), true, 100.0);
        let up = put(&mut w, (2, 1), true, 100.0);
        let left = put(&mut w, (1, 2), false, 100.0);
        let right = put(&mut w, (3, 2), false, 100.0);
        w.play(me);
        let wealth = |k: usize| w.agents[k].wealth;
        assert_eq!(wealth(me), 100.0 + 5.0 - 6.0 - 6.0);
        assert_eq!(
            (wealth(up), wealth(left), wealth(right)),
            (105.0, 106.0, 106.0)
        );
        assert_eq!((w.agents[me].games, w.agents[me].income), (3, -7.0));
        // The working paper: one random occupied neighbour.
        let mut w = world(5, |c| c.play = Play::RandomNeighbor);
        let me = put(&mut w, (2, 2), true, 100.0);
        for (p, c) in [((2, 1), true), ((1, 2), false), ((3, 2), false)] {
            put(&mut w, p, c, 100.0);
        }
        w.play(me);
        assert_eq!(w.agents[me].games, 1);
        assert_eq!(w.agents.iter().map(|a| a.games).sum::<u32>(), 2);
    }

    #[test]
    fn a_mover_that_dies_stops_playing() {
        let mut w = world(5, |_| {});
        let me = put(&mut w, (2, 2), true, 1.0);
        let left = put(&mut w, (1, 2), false, 0.0);
        let right = put(&mut w, (3, 2), false, 0.0);
        w.play(me);
        assert!(w.agents[me].dead);
        assert_eq!((w.agents[left].wealth, w.agents[right].wealth), (6.0, 0.0));
        assert!(w.agent_at(site(&w, (2, 2))).is_none());
    }

    #[test]
    fn eleven_clones_and_the_endowment_comes_from_the_parent_or_is_granted() {
        let mut w = world(5, |_| {});
        let p = put(&mut w, (2, 2), true, 10.5);
        w.reproduce(p);
        assert_eq!(w.population(), 1, "10.5 < 11");
        w.agents[p].wealth = 11.0;
        w.reproduce(p);
        assert_eq!((w.population(), w.births), (2, 1));
        let child = w.agents.last().unwrap();
        assert_eq!((child.wealth, child.cooperator), (6.0, true));
        let (cx, cy, _) = w.geometry.xyz(child.site as usize);
        assert_eq!(cx.abs_diff(2) + cy.abs_diff(2), 1, "a neighbouring site");
        assert_eq!(w.agents[p].wealth, 5.0);
        let mut g = world(5, |c| c.endowment_from = EndowmentFrom::Granted);
        let p = put(&mut g, (2, 2), false, 11.0);
        g.reproduce(p);
        assert_eq!(
            (g.population(), g.agents[p].wealth, g.agents[1].wealth),
            (2, 11.0, 6.0)
        );
        // No unoccupied neighbour: no offspring.
        let mut full = world(5, |_| {});
        let p = put(&mut full, (2, 2), true, 50.0);
        for q in [(2, 1), (1, 2), (3, 2), (2, 3)] {
            put(&mut full, q, true, 0.0);
        }
        full.reproduce(p);
        assert_eq!((full.population(), full.agents[p].wealth), (5, 50.0));
    }

    #[test]
    fn negative_wealth_kills_at_once_and_the_site_empties_at_once() {
        let mut w = world(5, |_| {});
        let d = put(&mut w, (2, 2), false, 100.0);
        let c = put(&mut w, (2, 1), true, 1.0);
        w.play(d);
        assert!(w.agents[c].dead);
        assert_eq!((w.population(), w.deaths), (1, 1));
        assert_eq!(w.at[site(&w, (2, 1))], NONE);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn end_of_cycle_removal_leaves_a_body_that_blocks_and_takes_no_part() {
        let mut w = world(5, |c| c.removal = Removal::EndOfCycle);
        let d = put(&mut w, (2, 2), false, 100.0);
        let c = put(&mut w, (2, 1), true, 1.0);
        let other = put(&mut w, (1, 1), false, 100.0);
        w.play(d);
        assert!(w.agents[c].dead);
        let body = site(&w, (2, 1));
        assert_eq!(w.at[body], c as u32, "the body still holds its site");
        assert!(w.agent_at(body).is_none());
        w.play(other);
        assert_eq!(w.agents[other].games, 0, "nobody plays the dead");
        w.end_cycle();
        assert_eq!(w.at[body], NONE);
        assert_eq!(w.agents.len(), 2);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn own_turn_death_waits_for_the_agents_turn_where_it_may_recover() {
        let mut w = world(5, |c| c.death_timing = DeathTiming::OwnTurn);
        // A cooperator at (2, 1) boxed in by three cooperators and a defector.
        let c = put(&mut w, (2, 1), true, 1.0);
        for p in [(2, 0), (1, 1), (3, 1)] {
            put(&mut w, p, true, 100.0);
        }
        let d = put(&mut w, (2, 2), false, 100.0);
        w.play(d);
        assert_eq!(w.agents[c].wealth, -5.0);
        assert!(!w.agents[c].dead, "not on its own turn");
        w.turn(c);
        // Blocked from moving, it plays its four neighbours: −5 + 3 × 5 − 6.
        assert_eq!(w.agents[c].wealth, 4.0);
        assert!(!w.agents[c].dead);
        // Still negative at the end of its turn: it dies.
        w.agents[c].wealth = -20.0;
        w.turn(c);
        assert!(w.agents[c].dead);
    }

    #[test]
    fn initial_and_newborn_ages_are_uniform_up_to_the_maximum() {
        let c = DpdConfig {
            agents: 900,
            max_age: 100,
            ..Default::default()
        };
        let w = DpdWorld::new(c, 3).unwrap();
        let ages: Vec<u32> = w.agents().map(|a| a.age).collect();
        assert!(ages.iter().all(|a| (1..=100).contains(a)));
        assert!(ages.contains(&1) && ages.contains(&100));
        let none = DpdWorld::new(DpdConfig::default(), 3).unwrap();
        assert!(none.agents().all(|a| a.age == 0));
        let mut w = world(5, |c| c.max_age = 10);
        let p = put(&mut w, (2, 2), true, 0.0);
        let mut born = std::collections::BTreeSet::new();
        for _ in 0..300 {
            w.agents[p].wealth = 11.0;
            w.reproduce(p);
            let child = w.agents.pop().unwrap();
            w.vacate(child.site as usize);
            w.living -= 1;
            assert!((1..=10).contains(&child.age), "{}", child.age);
            born.insert(child.age);
        }
        assert_eq!(born.len(), 10);
        w.config.newborn_age = NewbornAge::Zero;
        w.agents[p].wealth = 11.0;
        w.reproduce(p);
        assert_eq!(w.agents.last().unwrap().age, 0);
    }

    #[test]
    fn agents_age_each_turn_and_die_past_the_maximum_age() {
        let mut w = world(5, |c| c.max_age = 100);
        let old = put(&mut w, (0, 0), true, 10.0);
        let young = put(&mut w, (3, 3), true, 10.0);
        w.agents[old].age = 100;
        w.agents[young].age = 99;
        w.age_and_die(old);
        w.age_and_die(young);
        assert!(w.agents[old].dead && !w.agents[young].dead);
        assert_eq!(w.agents[young].age, 100);
        let mut w = world(5, |_| {});
        let a = put(&mut w, (0, 0), true, 10.0);
        w.agents[a].age = 1_000_000;
        w.age_and_die(a);
        assert!(!w.agents[a].dead, "no maximum: age only counts");
    }

    #[test]
    fn metabolism_is_charged_per_cycle_or_per_game() {
        let mut w = world(5, |c| c.metabolism = 3.0);
        let a = put(&mut w, (2, 2), true, 10.0);
        let b = put(&mut w, (2, 1), true, 10.0);
        w.play(a);
        assert_eq!((w.agents[a].wealth, w.agents[b].wealth), (15.0, 15.0));
        w.age_and_die(a);
        assert_eq!(w.agents[a].wealth, 12.0);
        let mut w = world(5, |c| {
            c.metabolism = 3.0;
            c.metabolism_per = MetabolismPer::Interaction;
        });
        let a = put(&mut w, (2, 2), true, 10.0);
        let b = put(&mut w, (2, 1), true, 10.0);
        w.play(a);
        assert_eq!((w.agents[a].wealth, w.agents[b].wealth), (12.0, 12.0));
        w.age_and_die(a);
        assert_eq!(w.agents[a].wealth, 12.0);
        // Metabolism kills: 1 − 3 < 0.
        w.agents[b].wealth = 1.0;
        w.config.metabolism_per = MetabolismPer::Cycle;
        w.age_and_die(b);
        assert!(w.agents[b].dead);
    }

    #[test]
    fn shifted_payoffs_less_a_per_game_metabolism_replay_the_negative_payoffs() {
        let neg = DpdConfig {
            max_age: 100,
            ..Default::default()
        };
        let shifted = DpdConfig {
            t: 12.0,
            r: 11.0,
            p: 1.0,
            s: 0.0,
            metabolism: 6.0,
            metabolism_per: MetabolismPer::Interaction,
            ..neg.clone()
        };
        let mut a = DpdWorld::new(neg, 5).unwrap();
        let mut b = DpdWorld::new(shifted.clone(), 5).unwrap();
        a.run(150);
        b.run(150);
        assert_eq!(a.fingerprint(), b.fingerprint());
        let per_cycle = DpdConfig {
            metabolism_per: MetabolismPer::Cycle,
            ..shifted
        };
        let mut c = DpdWorld::new(per_cycle, 5).unwrap();
        c.run(150);
        assert_ne!(
            a.fingerprint(),
            c.fingerprint(),
            "per cycle is another model"
        );
    }

    #[test]
    fn soup_pairs_random_agents_and_places_anywhere() {
        let mut w = world(9, |c| c.pairing = Pairing::Soup);
        let a = put(&mut w, (0, 0), true, 100.0);
        let b = put(&mut w, (4, 4), false, 100.0);
        w.play(a);
        assert_eq!(
            (w.agents[a].games, w.agents[b].games),
            (1, 1),
            "strangers play"
        );
        assert_eq!((w.agents[a].wealth, w.agents[b].wealth), (94.0, 106.0));
        let mut lone = world(9, |c| c.pairing = Pairing::Soup);
        let x = put(&mut lone, (0, 0), true, 100.0);
        lone.play(x);
        assert_eq!(lone.agents[x].games, 0, "nobody to play");
        // Boxed in, it still clones and moves: anywhere.
        let mut w = world(9, |c| c.pairing = Pairing::Soup);
        let p = put(&mut w, (4, 4), true, 50.0);
        for q in [(4, 3), (3, 4), (5, 4), (4, 5)] {
            put(&mut w, q, true, 0.0);
        }
        w.reproduce(p);
        assert_eq!(w.population(), 6);
        let far = |w: &DpdWorld, k: usize| {
            let (x, y) = xy(w, k);
            x.abs_diff(4) + y.abs_diff(4) > 1
        };
        assert!(far(&w, 5), "the offspring is not next door");
        let mut moved_far = false;
        for _ in 0..20 {
            w.move_agent(p);
            moved_far |= far(&w, p);
        }
        assert!(moved_far);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn mutation_flips_an_offspring_strategy_at_its_rate() {
        let mut w = world(5, |c| c.mutation = 1.0);
        assert!(!w.newborn_strategy(true) && w.newborn_strategy(false));
        w.config.mutation = 0.0;
        assert!(w.newborn_strategy(true) && !w.newborn_strategy(false));
        w.config.mutation = 0.5;
        let flips = (0..4000).filter(|_| !w.newborn_strategy(true)).count();
        assert!((1800..=2200).contains(&flips), "{flips}");
    }

    #[test]
    fn the_call_order_takes_n_over_2_swaps_or_a_full_shuffle() {
        let unmoved = |shuffle: Shuffle| {
            let mut w = world(30, |c| c.shuffle = shuffle);
            for k in 0..400u32 {
                put(&mut w, (k % 30, k / 30), true, 0.0);
            }
            let before: Vec<u64> = w.agents.iter().map(|a| a.id).collect();
            w.end_cycle();
            let after: Vec<u64> = w.agents.iter().map(|a| a.id).collect();
            let mut sorted = after.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, before, "a permutation");
            empty_list_is_consistent(&w);
            before.iter().zip(&after).filter(|(a, b)| a == b).count()
        };
        // 200 random swaps leave about 1/e of 400 in place; a full shuffle about one.
        let swaps = unmoved(Shuffle::Swaps);
        assert!((110..=190).contains(&swaps), "{swaps}");
        assert!(unmoved(Shuffle::Full) < 10);
        let mut one = world(5, |_| {});
        put(&mut one, (0, 0), true, 0.0);
        one.end_cycle();
        assert_eq!(one.agents.len(), 1);
    }

    #[test]
    fn a_cycle_runs_turns_in_list_order_or_each_phase_for_everyone() {
        let phases: [fn(&mut DpdWorld, usize); 4] = [
            DpdWorld::move_agent,
            DpdWorld::play,
            DpdWorld::reproduce,
            DpdWorld::age_and_die,
        ];
        for updating in [Updating::Asynchronous, Updating::Synchronous] {
            let c = DpdConfig {
                updating,
                ..Default::default()
            };
            let mut w = DpdWorld::new(c, 4).unwrap();
            w.run(20);
            let mut by_hand = w.clone();
            let before = w.clone();
            w.step();
            by_hand.begin_cycle();
            let n = by_hand.agents.len();
            match updating {
                Updating::Asynchronous => (0..n).for_each(|i| by_hand.turn(i)),
                Updating::Synchronous => {
                    for phase in phases {
                        for i in 0..n {
                            if !by_hand.agents[i].dead {
                                phase(&mut by_hand, i);
                            }
                        }
                    }
                }
            }
            by_hand.end_cycle();
            by_hand.tick += 1;
            assert_eq!(w.fingerprint(), by_hand.fingerprint(), "{updating:?}");
            let mut other = before;
            other.config.updating = match updating {
                Updating::Asynchronous => Updating::Synchronous,
                Updating::Synchronous => Updating::Asynchronous,
            };
            other.step();
            assert_ne!(w.fingerprint(), other.fingerprint());
        }
    }

    #[test]
    fn newborns_act_from_the_next_cycle_unless_set_to_this_one() {
        for (act, age) in [(NewbornsAct::NextCycle, 0), (NewbornsAct::ThisCycle, 1)] {
            let mut w = world(7, |c| c.newborns_act = act);
            put(&mut w, (3, 3), false, 20.0);
            w.step();
            assert_eq!(w.population(), 2);
            let child = w.agents().find(|a| a.id == 2).unwrap();
            assert_eq!(child.age, age, "{act:?}");
        }
    }

    #[test]
    fn a_surrounded_cooperator_has_eight_cooperating_moore_neighbours() {
        let mut w = world(8, |_| {});
        // A 4 × 3 block of cooperators has two surrounded (as Figure 9.10).
        for y in 1..4 {
            for x in 1..5 {
                put(&mut w, (x, y), true, 0.0);
            }
        }
        w.record();
        assert_eq!(w.stats.latest().unwrap().surrounded, 2);
        assert!(w.surrounded(site(&w, (2, 2))) && w.surrounded(site(&w, (3, 2))));
        let k = w.at[site(&w, (1, 1))] as usize;
        w.agents[k].cooperator = false;
        w.record();
        assert_eq!(w.stats.latest().unwrap().surrounded, 1);
        assert!(!w.surrounded(site(&w, (2, 2))));
    }

    #[test]
    fn a_world_that_empties_reports_nan_and_still_draws() {
        let mut w = world(10, |c| c.metabolism = 1.0);
        put(&mut w, (2, 2), true, 0.5);
        w.step();
        assert_eq!(w.population(), 0);
        let last = w.stats.latest().unwrap();
        assert!(last.cooperator_share.is_nan() && last.wealth_c.is_nan() && last.wealth_d.is_nan());
        assert_eq!((last.deaths, last.births, last.population), (1, 0, 0));
        assert!(w.latest_json().contains("\"cooperator_share\":null"));
        let mut buf = Vec::new();
        for mode in ["strategy", "wealth", "age", "surrounded"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 10 * 10 * 4);
        }
        assert!(w.inspect_json(2, 2).unwrap().contains("\"agent\":null"));
        w.run(3);
        assert_eq!(w.tick, 4);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn frames_draw_each_mode_and_inspect_names_neighbours_and_payoffs() {
        let mut w = world(4, |c| c.max_age = 100);
        put(&mut w, (1, 0), true, 11.0);
        put(&mut w, (2, 0), false, 0.0);
        let mut buf = Vec::new();
        w.render("strategy", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 4 * 4 * 4);
        assert_eq!(&buf[0..3], &BACKGROUND);
        assert_eq!(&buf[4..7], &COOPERATOR);
        assert_eq!(&buf[8..11], &DEFECTOR);
        w.render("wealth", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &lerp(COOL, HOT, 0.5));
        assert_eq!(&buf[8..11], &COOL);
        w.render("age", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &COOL);
        w.render("surrounded", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &lerp(BACKGROUND, COOPERATOR, 0.35));
        assert!(w.render("nope", "", &mut buf).is_err());
        let v = w.inspect(1, 0).unwrap().agent.unwrap();
        assert_eq!(
            (v.strategy, v.wealth, v.max_age, v.surrounded),
            ("C", 11.0, 100, false)
        );
        assert_eq!(v.neighbors.len(), 1);
        let n = &v.neighbors[0];
        assert_eq!(
            (n.x, n.y, n.strategy, n.payoff, n.their_payoff),
            (2, 0, "D", -6.0, 6.0)
        );
        assert!(w.inspect(0, 0).unwrap().agent.is_none());
        assert!(w.inspect(4, 0).is_err());
        let json: serde_json::Value = serde_json::from_str(&w.inspect_json(1, 0).unwrap()).unwrap();
        assert_eq!(json["agent"]["neighbors"][0]["strategy"], "D");
    }

    #[test]
    fn runs_stop_at_the_end_and_follow_their_seed() {
        let c = DpdConfig {
            end: 30,
            ..Default::default()
        };
        let mut a = DpdWorld::new(c.clone(), 7).unwrap();
        let mut b = DpdWorld::new(c.clone(), 7).unwrap();
        let mut other = DpdWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(40);
        }
        assert_eq!(a.tick, 30);
        assert!(a.is_finished() && Model::finished(&a));
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
        assert_eq!(a.stats.history().len(), 31);
        let first = a.agents().next().unwrap().id;
        assert!(a.locate(first).is_some() && a.locate(0).is_none());
        assert_eq!(a.agents_csv().lines().count(), a.population() + 1);
        let s = a.stats.latest().unwrap();
        assert_eq!(s.cooperators + s.defectors, s.population);
        assert_eq!(s.population as usize, a.population());
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(10, |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("r".to_string(), json!(1.0))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.r, 5.0);
        w.step();
        assert_eq!(w.config.r, 1.0);
    }

    #[test]
    fn the_lattice_stays_consistent_under_every_switch_and_keyframes_replay() {
        let c = DpdConfig {
            max_age: 50,
            mutation: 0.1,
            ..Default::default()
        };
        let mut w = DpdWorld::new(c, 2).unwrap();
        type Edit = fn(&mut DpdConfig);
        let edits: [Edit; 9] = [
            |c| c.removal = Removal::EndOfCycle,
            |c| c.death_timing = DeathTiming::OwnTurn,
            |c| c.endowment_from = EndowmentFrom::Granted,
            |c| c.newborn_age = NewbornAge::Zero,
            |c| c.updating = Updating::Synchronous,
            |c| c.shuffle = Shuffle::Full,
            |c| c.newborns_act = NewbornsAct::ThisCycle,
            |c| c.pairing = Pairing::Soup,
            |c| {
                c.pairing = Pairing::Space;
                c.play = Play::RandomNeighbor;
                c.metabolism = 1.0;
                c.metabolism_per = MetabolismPer::Interaction;
            },
        ];
        for edit in edits {
            edit(&mut w.config);
            w.run(40);
            empty_list_is_consistent(&w);
            assert_eq!(w.population(), w.agents.len());
        }
        let kept = w.clone();
        let mut back = kept.clone();
        back.run(50);
        let mut again = kept;
        again.run(50);
        assert_eq!(back.fingerprint(), again.fingerprint());
    }
}
```

The golden entries and the keyframe preset:

```diff
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -130,6 +130,20 @@
     ("jansson-kin", 0x265998639eacfbd0),
     ("jansson-kin-fixed", 0x3fac090571612879),
     ("hks-no-ethnocentrics", 0xbe867e7210bad2d2),
+    // Milestone 17: the demographic Prisoner's Dilemma.
+    ("dpd-run-1", 0x3d64b053fbfee4f6),
+    ("dpd-run-2", 0xe0198124ac5f4789),
+    ("dpd-run-3", 0x1c819cc85b7bb351),
+    ("dpd-run-4", 0xe6b5d66dee22ce07),
+    ("dpd-run-5", 0x2f5ae2bdc6bd257a),
+    ("dpd-working-paper", 0xaba834120f15810b),
+    ("dpd-closest", 0xd1c7475297f9864c),
+    ("dpd-soup", 0xdb64ad16a49146b),
+    ("dpd-shifted", 0x39b59d546b2bb2e8),
+    ("dpd-metabolism", 0x7b580a83419a3ea4),
+    ("dpd-footnote-27", 0xd1adeadce6881068),
+    ("dpd-rr-best", 0xe8fdc4ce027dd236),
+    ("dpd-coordination", 0x47f8c68504a9157a),
 ];
 
 fn fingerprint(id: &str) -> u64 {
```
```diff
--- a/crates/sugarscape-core/tests/checkpoint.rs
+++ b/crates/sugarscape-core/tests/checkpoint.rs
@@ -14,6 +14,7 @@
     "nbm-probabilistic",
     "hg-async-kaleidoscope",
     "jansson-kin",
+    "dpd-rr-best",
 ];
 
 fn world(id: &str) -> ModelWorld {
```

Run: `cargo test -p sugarscape-core --lib dpd` — Expected: compile errors (no `dpd` module).

- [ ] **Step 2: Wire the model kind**

`lib.rs` (`pub mod dpd;` after `pub mod culture;`), `model.rs` (every arm beside the ethno model's, `ALL: [ModelKind; 11]`, the unknown-model message, `max_ticks` from `end`, keyframes, the round-trip test), `presets.rs` (the catalog ends with `crate::dpd::presets()`), `sweep.rs` (the ticks error names the model) and the CLI's finish message:

```diff
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -10,6 +10,7 @@
 pub mod classes;
 pub mod config;
 pub mod culture;
+pub mod dpd;
 pub mod econ;
 pub mod edit;
 pub mod ethno;
```
```diff
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -10,6 +10,7 @@
 use crate::classes::{ClassesConfig, ClassesWorld};
 use crate::config::{Config, FieldError};
 use crate::culture::{CultureConfig, CultureWorld};
+use crate::dpd::{DpdConfig, DpdWorld};
 use crate::ethno::{EthnoConfig, EthnoWorld};
 use crate::render::{self, ColorMode, Layer};
 use crate::ring::{RingConfig, RingWorld};
@@ -19,7 +20,7 @@
 use crate::tags::{TagsConfig, TagsWorld};
 use crate::world::World;
 use crate::{
-    anasazi, civil, classes, culture, ethno, export, ring, schelling, spatial, stats, tags,
+    anasazi, civil, classes, culture, dpd, ethno, export, ring, schelling, spatial, stats, tags,
 };
 
 /// Which model a config or world is.
@@ -36,10 +37,11 @@
     Culture,
     Classes,
     Ethno,
+    Dpd,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 10] = [
+    pub const ALL: [ModelKind; 11] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -50,6 +52,7 @@
         ModelKind::Culture,
         ModelKind::Classes,
         ModelKind::Ethno,
+        ModelKind::Dpd,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -64,6 +67,7 @@
             ModelKind::Culture => "culture",
             ModelKind::Classes => "classes",
             ModelKind::Ethno => "ethno",
+            ModelKind::Dpd => "dpd",
         }
     }
 
@@ -81,6 +85,7 @@
             ModelKind::Culture => culture::schema(),
             ModelKind::Classes => classes::schema(),
             ModelKind::Ethno => ethno::schema(),
+            ModelKind::Dpd => dpd::schema(),
         }
     }
 }
@@ -104,6 +109,7 @@
     Culture(CultureConfig),
     Classes(ClassesConfig),
     Ethno(EthnoConfig),
+    Dpd(DpdConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -119,6 +125,7 @@
     Culture(&'a CultureConfig),
     Classes(&'a ClassesConfig),
     Ethno(&'a EthnoConfig),
+    Dpd(&'a DpdConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -141,6 +148,7 @@
             ModelConfig::Culture(c) => Tagged::Culture(c).serialize(s),
             ModelConfig::Classes(c) => Tagged::Classes(c).serialize(s),
             ModelConfig::Ethno(c) => Tagged::Ethno(c).serialize(s),
+            ModelConfig::Dpd(c) => Tagged::Dpd(c).serialize(s),
         }
     }
 }
@@ -158,6 +166,7 @@
             ModelConfig::Culture(_) => ModelKind::Culture,
             ModelConfig::Classes(_) => ModelKind::Classes,
             ModelConfig::Ethno(_) => ModelKind::Ethno,
+            ModelConfig::Dpd(_) => ModelKind::Dpd,
         }
     }
 
@@ -222,10 +231,13 @@
             "ethno" => serde_json::from_value(value)
                 .map(ModelConfig::Ethno)
                 .map_err(|e| FieldError::new("config", e.to_string())),
+            "dpd" => serde_json::from_value(value)
+                .map(ModelConfig::Dpd)
+                .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes or ethno)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags, culture, classes, ethno or dpd)"
                 ),
             )),
         }
@@ -243,6 +255,7 @@
             ModelConfig::Culture(c) => c.validate(),
             ModelConfig::Classes(c) => c.validate(),
             ModelConfig::Ethno(c) => c.validate(),
+            ModelConfig::Dpd(c) => c.validate(),
         }
     }
 
@@ -260,6 +273,7 @@
             ModelConfig::Culture(c) => set_path(c, path, value).map(ModelConfig::Culture),
             ModelConfig::Classes(c) => set_path(c, path, value).map(ModelConfig::Classes),
             ModelConfig::Ethno(c) => set_path(c, path, value).map(ModelConfig::Ethno),
+            ModelConfig::Dpd(c) => set_path(c, path, value).map(ModelConfig::Dpd),
         }
     }
 
@@ -270,6 +284,7 @@
             ModelConfig::Anasazi(c) => Some(c.end_year.saturating_sub(c.start_year)),
             ModelConfig::Tags(c) => (c.end > 0).then_some(c.end),
             ModelConfig::Ethno(c) => (c.end > 0).then_some(c.end),
+            ModelConfig::Dpd(c) => (c.end > 0).then_some(c.end),
             ModelConfig::Sugarscape(_)
             | ModelConfig::Schelling(_)
             | ModelConfig::Ring(_)
@@ -293,6 +308,7 @@
             ModelConfig::Culture(_) => culture::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Classes(_) => classes::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Ethno(_) => ethno::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Dpd(_) => dpd::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -472,6 +488,7 @@
     Culture(Box<CultureWorld>),
     Classes(Box<ClassesWorld>),
     Ethno(Box<EthnoWorld>),
+    Dpd(Box<DpdWorld>),
 }
 
 impl ModelWorld {
@@ -501,6 +518,7 @@
             ModelConfig::Culture(c) => ModelWorld::Culture(Box::new(CultureWorld::new(c, seed)?)),
             ModelConfig::Classes(c) => ModelWorld::Classes(Box::new(ClassesWorld::new(c, seed)?)),
             ModelConfig::Ethno(c) => ModelWorld::Ethno(Box::new(EthnoWorld::new(c, seed)?)),
+            ModelConfig::Dpd(c) => ModelWorld::Dpd(Box::new(DpdWorld::new(c, seed)?)),
         })
     }
 
@@ -516,6 +534,7 @@
             ModelWorld::Culture(_) => ModelKind::Culture,
             ModelWorld::Classes(_) => ModelKind::Classes,
             ModelWorld::Ethno(_) => ModelKind::Ethno,
+            ModelWorld::Dpd(_) => ModelKind::Dpd,
         }
     }
 
@@ -531,6 +550,7 @@
             ModelWorld::Culture(w) => w.as_ref(),
             ModelWorld::Classes(w) => w.as_ref(),
             ModelWorld::Ethno(w) => w.as_ref(),
+            ModelWorld::Dpd(w) => w.as_ref(),
         }
     }
 
@@ -546,6 +566,7 @@
             ModelWorld::Culture(w) => w.as_mut(),
             ModelWorld::Classes(w) => w.as_mut(),
             ModelWorld::Ethno(w) => w.as_mut(),
+            ModelWorld::Dpd(w) => w.as_mut(),
         }
     }
 
@@ -629,6 +650,7 @@
             ModelWorld::Culture(w) => copy_without_history!(Culture, w),
             ModelWorld::Classes(w) => copy_without_history!(Classes, w),
             ModelWorld::Ethno(w) => copy_without_history!(Ethno, w),
+            ModelWorld::Dpd(w) => copy_without_history!(Dpd, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -654,6 +676,7 @@
             (ModelWorld::Culture(live), ModelWorld::Culture(kept)) => restore_into!(live, kept),
             (ModelWorld::Classes(live), ModelWorld::Classes(kept)) => restore_into!(live, kept),
             (ModelWorld::Ethno(live), ModelWorld::Ethno(kept)) => restore_into!(live, kept),
+            (ModelWorld::Dpd(live), ModelWorld::Dpd(kept)) => restore_into!(live, kept),
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -907,6 +930,32 @@
     }
 
     #[test]
+    fn dpd_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "dpd", "max_age": 100, "removal": "end_of_cycle"}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Dpd);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(json["model"], "dpd");
+        assert_eq!(json["removal"], "end_of_cycle");
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["cooperators", "defectors"]);
+        assert_eq!(c.max_ticks(), None);
+        let next = c.with_path("end", &json!(500)).unwrap();
+        assert_eq!(next.max_ticks(), Some(500));
+        let e = ModelConfig::from_json(r#"{"model": "dpd", "mutation": 2}"#).unwrap_err();
+        assert_eq!(e[0].field, "mutation");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!((w.kind(), w.model().size()), (ModelKind::Dpd, (30, 30)));
+        assert_eq!(w.model().population(), 100);
+        let cp = w.checkpoint().expect("dpd worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
+    }
+
+    #[test]
     fn only_the_anasazi_finishes() {
         let mut w = ModelWorld::new(
             ModelConfig::Anasazi(crate::anasazi::AnasaziConfig {
@@ -939,7 +988,8 @@
                 "tags",
                 "culture",
                 "classes",
-                "ethno"
+                "ethno",
+                "dpd"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
```
```diff
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -673,7 +673,7 @@
 
 /// Every model's presets: the sugarscape's (`all`), then Schelling's, Ring
 /// World's, the anasazi's, civil violence's, the tags model's, the spatial
-/// games' and the ethnocentrism model's.
+/// games', the ethnocentrism model's and the demographic PD's.
 pub fn catalog() -> Vec<ModelPreset> {
     let mut out: Vec<ModelPreset> = all().into_iter().map(ModelPreset::from).collect();
     out.extend(crate::schelling::presets());
@@ -685,6 +685,7 @@
     out.extend(crate::classes::presets());
     out.extend(crate::spatial::presets());
     out.extend(crate::ethno::presets());
+    out.extend(crate::dpd::presets());
     out
 }
```
```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -398,6 +398,9 @@
                     format!(
                         "the ethnocentrism model stops at its last period, {max} in this config"
                     )
+                }
+                crate::model::ModelKind::Dpd => {
+                    format!("the demographic PD stops at its last cycle, {max} in this config")
                 }
                 _ => format!(
                     "the Long House Valley stops at its end year, \
```
```diff
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -182,7 +182,7 @@
         // The anasazi stops at its end year; civil violence when a group is gone;
         // the tags model at its last generation; Axelrod's culture once stable,
         // a sugarscape under his rule once its cultures settle; ethnocentrism at its
-        // last period.
+        // last period; the demographic PD at its last cycle.
         let why = match config.kind() {
             ModelKind::Civil => "a group has died out",
             ModelKind::Tags => "its last generation",
@@ -190,6 +190,7 @@
             ModelKind::Classes => "equity reached",
             ModelKind::Sugarscape => "the cultures have settled",
             ModelKind::Ethno => "its last period",
+            ModelKind::Dpd => "its last cycle",
             _ => "its end year",
         };
         eprintln!("finished at tick {} ({why})", world.tick());
```

- [ ] **Step 3: Implement**

`mod.rs`:

```rust
//! The demographic Prisoner's Dilemma (milestone 17): Epstein, "Zones of
//! Cooperation in Demographic Prisoner's Dilemma" (SFI WP 97-12-094;
//! Complexity 4(2), 1998; Generative Social Science, 2006, ch. 9 and its
//! appendix), with the working paper's rule, Radax & Rengs' (2009) unstated
//! timing choices, soup, metabolism and the coordination game as named
//! switches. See docs/superpowers/specs/2026-09-26-demographic-pd-design.md.

mod config;
mod presets;
mod stats;
mod world;

pub use config::{
    schema, DeathTiming, DpdConfig, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct, Pairing,
    Play, Removal, Shuffle, Updating, LIVE,
};
pub use presets::presets;
pub use stats::{DpdSnapshot, SERIES};
pub use world::{
    Agent, AgentView, DpdInspection, DpdMode, DpdWorld, NeighborView, Site, COOPERATOR, DEFECTOR,
};
```

`config.rs`, above its tests (Decisions 9, 14):

```rust
//! The demographic Prisoner's Dilemma's parameters: the published text (GSS
//! Table 9.1, the CD's settings where the prose is silent) by default, with
//! the working paper's rule, Radax and Rengs' timing choices, soup and
//! metabolism as named switches.

use serde::{Deserialize, Serialize};

use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// When `metabolism` is charged.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetabolismPer {
    /// Once per cycle, on the agent's own turn (WP note 29).
    #[default]
    Cycle,
    /// To both players after every game (the text's "after every interaction").
    Interaction,
}

/// Whom a mover plays.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Play {
    /// One game with each occupied von Neumann neighbour (GSS).
    #[default]
    EachNeighbor,
    /// One game with one random occupied neighbour (the working paper).
    RandomNeighbor,
}

/// Where agents meet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pairing {
    /// On the lattice: neighbours play, offspring go next door.
    #[default]
    Space,
    /// "Equiprobable random agent pairings": a random partner, random
    /// placement anywhere.
    Soup,
}

/// When an agent whose wealth goes negative dies (RR's "die immediately").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeathTiming {
    /// At once, even on another agent's turn.
    #[default]
    Immediate,
    /// At the end of its own next turn, if still negative.
    OwnTurn,
}

/// When a dead agent leaves its site (RR's "remove dead agents immediately").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Removal {
    /// At once.
    #[default]
    Immediate,
    /// At the cycle's end; until then it blocks its site and takes no part.
    EndOfCycle,
}

/// Where an offspring's endowment comes from (RR's "initial endowment inherited").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndowmentFrom {
    /// Subtracted from the parent's wealth (the text).
    #[default]
    Parent,
    /// Granted without cost to the parent.
    Granted,
}

/// An offspring's starting age (RR's "random birth age").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewbornAge {
    /// Uniform in 1 … `max_age`, like the initial agents (0 with no maximum).
    #[default]
    Random,
    /// 0.
    Zero,
}

/// How the cycle is scheduled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Updating {
    /// Each agent moves, plays, reproduces and ages in its turn.
    #[default]
    Asynchronous,
    /// All move, then all play, then all reproduce, then all age (RR's control).
    Synchronous,
}

/// How the agent list is reordered after each cycle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shuffle {
    /// N/2 swaps of two random agents (GSS p. 206).
    #[default]
    Swaps,
    /// A full Fisher–Yates shuffle (RR's Repast list shuffle).
    Full,
}

/// When an offspring first takes a turn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewbornsAct {
    /// From the next cycle.
    #[default]
    NextCycle,
    /// In this cycle, after the agents already in the list.
    ThisCycle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DpdConfig {
    /// The torus is `width` × `width`.
    pub width: u32,
    /// Initial agents, on random empty sites.
    pub agents: u32,
    /// The chance an initial agent cooperates.
    pub initial_cooperators: f64,
    /// The initial agents' wealth (the CD's `Initial Wealth`).
    pub initial_wealth: f64,
    /// Payoffs: T to a defector against a cooperator, R to mutual
    /// cooperators, P to mutual defectors, S to a cooperator against a
    /// defector.
    pub t: f64,
    pub r: f64,
    pub p: f64,
    pub s: f64,
    /// An agent with at least this wealth may clone (the CD's `Fission Wealth`).
    pub fission_wealth: f64,
    /// An offspring's starting wealth.
    pub endowment: f64,
    /// The maximum age (0: none).
    pub max_age: u32,
    /// Wealth charged per cycle or per game.
    pub metabolism: f64,
    pub metabolism_per: MetabolismPer,
    /// The chance an offspring's strategy is the other one.
    pub mutation: f64,
    /// The movement radius (von Neumann distance).
    pub vision: u32,
    pub play: Play,
    pub pairing: Pairing,
    pub death_timing: DeathTiming,
    pub removal: Removal,
    pub endowment_from: EndowmentFrom,
    pub newborn_age: NewbornAge,
    pub updating: Updating,
    pub shuffle: Shuffle,
    pub newborns_act: NewbornsAct,
    /// The last cycle; the run stops there (0: never).
    pub end: u32,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for DpdConfig {
    /// GSS Table 9.1 (Run 1) with the CD's initial wealth 6 and fission
    /// wealth 11: a 30 × 30 torus, 100 agents, payoffs 6, 5, −5, −6, no
    /// maximum age, no mutation, no metabolism, vision 1.
    fn default() -> Self {
        DpdConfig {
            width: 30,
            agents: 100,
            initial_cooperators: 0.5,
            initial_wealth: 6.0,
            t: 6.0,
            r: 5.0,
            p: -5.0,
            s: -6.0,
            fission_wealth: 11.0,
            endowment: 6.0,
            max_age: 0,
            metabolism: 0.0,
            metabolism_per: MetabolismPer::Cycle,
            mutation: 0.0,
            vision: 1,
            play: Play::EachNeighbor,
            pairing: Pairing::Space,
            death_timing: DeathTiming::Immediate,
            removal: Removal::Immediate,
            endowment_from: EndowmentFrom::Parent,
            newborn_age: NewbornAge::Random,
            updating: Updating::Asynchronous,
            shuffle: Shuffle::Swaps,
            newborns_act: NewbornsAct::NextCycle,
            end: 0,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 20] = [
    "t",
    "r",
    "p",
    "s",
    "fission_wealth",
    "endowment",
    "max_age",
    "metabolism",
    "metabolism_per",
    "mutation",
    "play",
    "pairing",
    "death_timing",
    "removal",
    "endowment_from",
    "newborn_age",
    "updating",
    "shuffle",
    "newborns_act",
    "end",
];

impl DpdConfig {
    /// The payoff to a player with strategy `me` against `other` (true:
    /// cooperate).
    pub fn payoff(&self, me: bool, other: bool) -> f64 {
        match (me, other) {
            (true, true) => self.r,
            (true, false) => self.s,
            (false, true) => self.t,
            (false, false) => self.p,
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
        let nonneg = |x: f64| x.is_finite() && x >= 0.0;
        let width_ok = (3..=200).contains(&self.width);
        check(width_ok, "width", "must be between 3 and 200");
        check(
            !width_ok || self.agents <= self.width * self.width,
            "agents",
            "must be at most width × width",
        );
        check(
            unit(self.initial_cooperators),
            "initial_cooperators",
            "must be between 0 and 1",
        );
        check(
            nonneg(self.initial_wealth),
            "initial_wealth",
            "must be a number ≥ 0",
        );
        for (field, v) in [("t", self.t), ("r", self.r), ("p", self.p), ("s", self.s)] {
            check(v.is_finite(), field, "must be a number");
        }
        check(
            nonneg(self.fission_wealth),
            "fission_wealth",
            "must be a number ≥ 0",
        );
        check(nonneg(self.endowment), "endowment", "must be a number ≥ 0");
        check(
            nonneg(self.metabolism),
            "metabolism",
            "must be a number ≥ 0",
        );
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        check(
            (1..=10).contains(&self.vision),
            "vision",
            "must be between 1 and 10",
        );
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
                match ModelConfig::Dpd(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Dpd(next)) => {
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
    pub fn changes(&self, next: &DpdConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !LIVE.contains(&k.as_str()))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    let payoff = (-20.0, 20.0, 1.0);
    vec![
        Param::number("Game", "t", "T (defect against a cooperator)", payoff, Live)
            .with_help("The temptation: a defector's payoff against a cooperator"),
        Param::number("Game", "r", "R (both cooperate)", payoff, Live)
            .with_help("The reward to each of two cooperators"),
        Param::number("Game", "p", "P (both defect)", payoff, Live)
            .with_help("The punishment to each of two defectors"),
        Param::number(
            "Game",
            "s",
            "S (cooperate against a defector)",
            payoff,
            Live,
        )
        .with_help("The sucker's payoff: a cooperator's against a defector"),
        Param::integer(
            "Population",
            "width",
            "Width (a square torus)",
            (3, 200),
            Reset,
        ),
        Param::integer("Population", "agents", "Initial agents", (0, 40_000), Reset),
        Param::number(
            "Population",
            "initial_cooperators",
            "Initial cooperators",
            (0.0, 1.0, 0.05),
            Reset,
        )
        .with_help("The chance an initial agent cooperates"),
        Param::number(
            "Population",
            "initial_wealth",
            "Initial wealth",
            (0.0, 50.0, 1.0),
            Reset,
        ),
        Param::number(
            "Population",
            "fission_wealth",
            "Wealth to clone",
            (0.0, 100.0, 1.0),
            Live,
        )
        .with_help("An agent with at least this much has an offspring (the CD's 11)"),
        Param::number(
            "Population",
            "endowment",
            "Offspring's endowment",
            (0.0, 50.0, 1.0),
            Live,
        ),
        Param::integer(
            "Population",
            "max_age",
            "Maximum age (0: none)",
            (0, 100_000),
            Live,
        ),
        Param::number(
            "Population",
            "metabolism",
            "Metabolism",
            (0.0, 20.0, 1.0),
            Live,
        )
        .with_help("Wealth charged every cycle (or every game)"),
        Param::choice(
            "Population",
            "metabolism_per",
            "Metabolism charged",
            &[
                ("cycle", "Per cycle (the text's note)"),
                ("interaction", "Per game"),
            ],
            Live,
        ),
        Param::integer("Population", "vision", "Vision", (1, 10), Reset)
            .with_help("How far an agent can move (von Neumann)"),
        Param::number(
            "Evolution",
            "mutation",
            "Mutation rate",
            (0.0, 1.0, 0.05),
            Live,
        )
        .with_help("The chance an offspring's strategy is not its parent's"),
        Param::choice(
            "Timing",
            "death_timing",
            "Death",
            &[
                ("immediate", "As wealth goes negative"),
                ("own_turn", "At the end of its own turn"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "removal",
            "The dead leave",
            &[
                ("immediate", "At once"),
                ("end_of_cycle", "At the cycle's end"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "endowment_from",
            "Endowment",
            &[("parent", "Taken from the parent"), ("granted", "Granted")],
            Live,
        ),
        Param::choice(
            "Timing",
            "newborn_age",
            "Newborns' age",
            &[("random", "Random, 1 to the maximum age"), ("zero", "Zero")],
            Live,
        ),
        Param::choice(
            "Timing",
            "newborns_act",
            "Newborns act",
            &[
                ("next_cycle", "From the next cycle"),
                ("this_cycle", "In this cycle"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "updating",
            "Updating",
            &[
                ("asynchronous", "Asynchronous (each agent in turn)"),
                ("synchronous", "Synchronous (all move, all play, …)"),
            ],
            Live,
        ),
        Param::choice(
            "Timing",
            "shuffle",
            "Call order",
            &[
                ("swaps", "N/2 random swaps (Epstein)"),
                ("full", "A full shuffle"),
            ],
            Live,
        ),
        Param::choice(
            "Interaction",
            "pairing",
            "Pairing",
            &[
                ("space", "Space: neighbours"),
                ("soup", "Soup: random partners"),
            ],
            Live,
        ),
        Param::choice(
            "Interaction",
            "play",
            "Play",
            &[
                ("each_neighbor", "Each neighbour (the published text)"),
                (
                    "random_neighbor",
                    "One random neighbour (the working paper)",
                ),
            ],
            Live,
        )
        .shown_if("pairing", "space"),
        Param::integer("Run", "end", "Last cycle (0: never)", (0, 100_000), Live),
    ]
}
```

`stats.rs` (Decision 8):

```rust
//! The demographic Prisoner's Dilemma's statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 9] = [
    "cooperators",
    "defectors",
    "population",
    "cooperator_share",
    "surrounded",
    "wealth_c",
    "wealth_d",
    "births",
    "deaths",
];

/// One cycle's statistics, of the agents alive at its end. A share or mean
/// with nobody to count is NaN (JSON null).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct DpdSnapshot {
    pub tick: u64,
    pub cooperators: u32,
    pub defectors: u32,
    pub population: u32,
    pub cooperator_share: f64,
    /// Cooperators all eight of whose Moore neighbours are cooperators.
    pub surrounded: u32,
    /// Mean wealth of cooperators and of defectors.
    pub wealth_c: f64,
    pub wealth_d: f64,
    /// Offspring born and agents dead this cycle.
    pub births: u32,
    pub deaths: u32,
}

impl Series for DpdSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "cooperators" => f64::from(self.cooperators),
            "defectors" => f64::from(self.defectors),
            "population" => f64::from(self.population),
            "cooperator_share" => self.cooperator_share,
            "surrounded" => f64::from(self.surrounded),
            "wealth_c" => self.wealth_c,
            "wealth_d" => self.wealth_d,
            "births" => f64::from(self.births),
            "deaths" => f64::from(self.deaths),
            _ => return None,
        })
    }
}

/// `a / b`, or NaN when `b` is 0.
pub fn ratio(a: f64, b: u32) -> f64 {
    if b == 0 {
        f64::NAN
    } else {
        a / f64::from(b)
    }
}
```

`world.rs`, above its tests (Decisions 2–7, 12, 13):

```rust
//! The demographic Prisoner's Dilemma's world: agents with a fixed strategy
//! (cooperate or defect), wealth and age on a von Neumann torus. In each
//! cycle every agent, in the list's order, moves to a random unoccupied site
//! within its vision, plays each neighbour, clones itself onto an empty
//! neighbouring site once rich enough, ages, and dies when its wealth goes
//! negative or it outlives the maximum age.

use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{
    DeathTiming, DpdConfig, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct, Pairing, Play,
    Removal, Shuffle, Updating,
};
use super::stats::{ratio, DpdSnapshot, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, COOL, HOT, RED};
use crate::rng::{self, SimRng};
use crate::spatial::{self, Geometry};
use crate::stats::Stats;

/// The Strategy view's colours (GSS: cooperators blue, defectors red).
pub const COOPERATOR: Rgb = BLUE;
pub const DEFECTOR: Rgb = RED;

/// No agent on a site.
const NONE: u32 = u32::MAX;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DpdMode {
    Strategy,
    Wealth,
    Age,
    Surrounded,
}

impl std::str::FromStr for DpdMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "strategy" => Self::Strategy,
            "wealth" => Self::Wealth,
            "age" => Self::Age,
            "surrounded" => Self::Surrounded,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// An agent in the call list.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: u64,
    pub cooperator: bool,
    pub wealth: f64,
    pub age: u32,
    pub site: u32,
    /// Dead this cycle: it leaves the list at the cycle's end (and its site
    /// at once or then, per `removal`).
    pub dead: bool,
    /// Payoffs received and games played this cycle.
    pub income: f64,
    pub games: u32,
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DpdInspection {
    pub site: Site,
    /// The agent on the site, or none.
    pub agent: Option<AgentView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Site {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct AgentView {
    pub id: u64,
    /// "C" or "D".
    pub strategy: &'static str,
    pub wealth: f64,
    pub age: u32,
    /// The maximum age (0: none).
    pub max_age: u32,
    pub surrounded: bool,
    /// Payoffs received and games played this cycle.
    pub income: f64,
    pub games: u32,
    /// The occupied neighbours: up, left, right, down.
    pub neighbors: Vec<NeighborView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NeighborView {
    pub x: u32,
    pub y: u32,
    pub id: u64,
    pub strategy: &'static str,
    /// One game's payoff to the agent against this neighbour, and to the
    /// neighbour, under the current payoffs.
    pub payoff: f64,
    pub their_payoff: f64,
}

#[derive(Clone)]
pub struct DpdWorld {
    pub config: DpdConfig,
    /// The periodic von Neumann lattice (movement, play, birth), and the
    /// Moore one (the surrounded index); fixed once built, so keyframes
    /// share them.
    pub geometry: Arc<Geometry>,
    moore: Arc<Geometry>,
    /// The distinct displacements (dx, dy mod width) within `vision`.
    reach: Arc<Vec<(u32, u32)>>,
    /// Completed cycles.
    pub tick: u64,
    /// The call list.
    agents: Vec<Agent>,
    /// Each site's agent (an index into `agents`), or `NONE`.
    at: Vec<u32>,
    /// The unoccupied sites (in no particular order) and each site's index
    /// in it (`u32::MAX` when occupied). A dead agent awaiting removal
    /// occupies its site.
    empty: Vec<u32>,
    slot: Vec<u32>,
    next_id: u64,
    /// Agents in the list not yet dead.
    living: u32,
    births: u32,
    deaths: u32,
    rng: SimRng,
    pub stats: Stats<DpdSnapshot>,
}

/// The distinct displacements within von Neumann distance `vision` on a
/// `width` torus, excluding none: ascending by (dy, dx), wrapped, the first
/// of any that coincide kept.
fn reach(width: u32, vision: u32) -> Vec<(u32, u32)> {
    let (w, v) = (i64::from(width), i64::from(vision));
    let mut out = Vec::new();
    for dy in -v..=v {
        let span = v - dy.abs();
        for dx in -span..=span {
            let d = (dx.rem_euclid(w) as u32, dy.rem_euclid(w) as u32);
            if d != (0, 0) && !out.contains(&d) {
                out.push(d);
            }
        }
    }
    out
}

impl DpdWorld {
    pub fn new(config: DpdConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let lattice = |neighborhood| spatial::SpatialConfig {
            width: config.width,
            height: config.width,
            neighborhood,
            boundary: spatial::Boundary::Periodic,
            ..Default::default()
        };
        // A square lattice draws nothing; the world's stream starts untouched.
        let geometry = Geometry::new(
            &lattice(spatial::Neighborhood::VonNeumann),
            &mut rng::seeded(0),
        );
        let moore = Geometry::new(&lattice(spatial::Neighborhood::Moore), &mut rng::seeded(0));
        let n = geometry.len();
        let reach = reach(config.width, config.vision);
        let mut w = DpdWorld {
            config,
            geometry: Arc::new(geometry),
            moore: Arc::new(moore),
            reach: Arc::new(reach),
            tick: 0,
            agents: Vec::new(),
            at: vec![NONE; n],
            empty: (0..n as u32).collect(),
            slot: (0..n as u32).collect(),
            next_id: 1,
            living: 0,
            births: 0,
            deaths: 0,
            rng: rng::seeded(seed),
            stats: Stats::default(),
        };
        for _ in 0..w.config.agents {
            let site = w.random_empty().expect("validation keeps agents ≤ sites");
            let cooperator = w.rng.gen::<f64>() < w.config.initial_cooperators;
            let age = w.random_age();
            w.add(site, cooperator, w.config.initial_wealth, age);
        }
        w.record();
        Ok(w)
    }

    pub fn sites(&self) -> usize {
        self.at.len()
    }

    /// The living agents, in call order.
    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.agents.iter().filter(|a| !a.dead)
    }

    /// The living agent on `site`, if any.
    pub fn agent_at(&self, site: usize) -> Option<&Agent> {
        self.living_at(site).map(|k| &self.agents[k])
    }

    pub fn population(&self) -> usize {
        self.living as usize
    }

    /// Whether the run has reached its last cycle.
    pub fn is_finished(&self) -> bool {
        self.config.end > 0 && self.tick >= u64::from(self.config.end)
    }

    /// A uniform age in 1 … `max_age`, or 0 with no maximum.
    fn random_age(&mut self) -> u32 {
        match self.config.max_age {
            0 => 0,
            m => self.rng.gen_range(0..m) + 1,
        }
    }

    /// Adds an agent to the end of the list on the empty `site`.
    fn add(&mut self, site: usize, cooperator: bool, wealth: f64, age: u32) {
        let id = self.next_id;
        self.next_id += 1;
        let k = self.agents.len();
        self.agents.push(Agent {
            id,
            cooperator,
            wealth,
            age,
            site: site as u32,
            dead: false,
            income: 0.0,
            games: 0,
        });
        self.place(site, k);
        self.living += 1;
    }

    fn place(&mut self, site: usize, k: usize) {
        debug_assert_eq!(self.at[site], NONE, "site {site} is occupied");
        let j = self.slot[site] as usize;
        let last = *self.empty.last().expect("an empty site");
        self.empty.swap_remove(j);
        if last as usize != site {
            self.slot[last as usize] = j as u32;
        }
        self.slot[site] = u32::MAX;
        self.at[site] = k as u32;
    }

    fn vacate(&mut self, site: usize) {
        if self.at[site] != NONE {
            self.at[site] = NONE;
            self.slot[site] = self.empty.len() as u32;
            self.empty.push(site as u32);
        }
    }

    /// A uniformly chosen unoccupied site, if any.
    fn random_empty(&mut self) -> Option<usize> {
        if self.empty.is_empty() {
            None
        } else {
            let k = self.rng.gen_range(0..self.empty.len() as u32) as usize;
            Some(self.empty[k] as usize)
        }
    }

    /// A uniformly chosen member of `from`, if any.
    fn pick(&mut self, from: &[usize]) -> Option<usize> {
        if from.is_empty() {
            None
        } else {
            Some(from[self.rng.gen_range(0..from.len() as u32) as usize])
        }
    }

    /// The index of the living agent on `site`, if any.
    fn living_at(&self, site: usize) -> Option<usize> {
        let k = self.at[site];
        (k != NONE && !self.agents[k as usize].dead).then_some(k as usize)
    }

    /// Agent `k` moves to the empty `site`.
    fn relocate(&mut self, k: usize, site: usize) {
        let from = self.agents[k].site as usize;
        self.place(site, k);
        self.vacate(from);
        self.agents[k].site = site as u32;
    }

    /// One cycle.
    pub fn step(&mut self) {
        self.apply_schedule();
        self.begin_cycle();
        let start = self.agents.len();
        match self.config.updating {
            Updating::Asynchronous => {
                let mut i = 0;
                while i < self.bound(start) {
                    self.turn(i);
                    i += 1;
                }
            }
            Updating::Synchronous => {
                let phases: [fn(&mut Self, usize); 4] = [
                    Self::move_agent,
                    Self::play,
                    Self::reproduce,
                    Self::age_and_die,
                ];
                for phase in phases {
                    let mut i = 0;
                    while i < self.bound(start) {
                        if !self.agents[i].dead {
                            phase(self, i);
                        }
                        i += 1;
                    }
                }
            }
        }
        self.end_cycle();
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
            let next = ModelConfig::Dpd(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Dpd(next)) = next {
                self.config = next;
            }
        }
    }

    fn begin_cycle(&mut self) {
        self.births = 0;
        self.deaths = 0;
        for a in &mut self.agents {
            a.income = 0.0;
            a.games = 0;
        }
    }

    /// How far down the list this cycle reaches: the agents present at its
    /// start, or (`this_cycle`) newborns too.
    fn bound(&self, start: usize) -> usize {
        match self.config.newborns_act {
            NewbornsAct::NextCycle => start,
            NewbornsAct::ThisCycle => self.agents.len(),
        }
    }

    /// Agent `i`'s turn (asynchronous): move, play, reproduce, age and die.
    fn turn(&mut self, i: usize) {
        if self.agents[i].dead {
            return;
        }
        self.move_agent(i);
        self.play(i);
        if self.agents[i].dead {
            return;
        }
        self.reproduce(i);
        self.age_and_die(i);
    }

    /// Rule 1: to a uniformly chosen unoccupied site within vision (soup:
    /// anywhere); none → stay.
    fn move_agent(&mut self, i: usize) {
        let target = match self.config.pairing {
            Pairing::Space => {
                let w = self.config.width;
                let s = self.agents[i].site;
                let (x, y) = (s % w, s / w);
                let free: Vec<usize> = self
                    .reach
                    .iter()
                    .map(|&(dx, dy)| ((x + dx) % w + (y + dy) % w * w) as usize)
                    .filter(|&j| self.at[j] == NONE)
                    .collect();
                self.pick(&free)
            }
            Pairing::Soup => self.random_empty(),
        };
        if let Some(site) = target {
            self.relocate(i, site);
        }
    }

    /// Rule 2: one game with each occupied neighbour in direction order (up,
    /// left, right, down), stopping if the mover dies; or with one random
    /// occupied neighbour; or (soup) with one random other living agent.
    fn play(&mut self, i: usize) {
        match (self.config.pairing, self.config.play) {
            (Pairing::Soup, _) => {
                if self.living < 2 {
                    return;
                }
                let n = self.agents.len() as u32;
                let k = loop {
                    let k = self.rng.gen_range(0..n) as usize;
                    if k != i && !self.agents[k].dead {
                        break k;
                    }
                };
                self.game(i, k);
            }
            (Pairing::Space, Play::EachNeighbor) => {
                let geometry = Arc::clone(&self.geometry);
                for &j in geometry.neighbors(self.agents[i].site as usize) {
                    if let Some(k) = self.living_at(j as usize) {
                        self.game(i, k);
                        if self.agents[i].dead {
                            break;
                        }
                    }
                }
            }
            (Pairing::Space, Play::RandomNeighbor) => {
                let occupied: Vec<usize> = self
                    .geometry
                    .neighbors(self.agents[i].site as usize)
                    .iter()
                    .filter_map(|&j| self.living_at(j as usize))
                    .collect();
                if let Some(k) = self.pick(&occupied) {
                    self.game(i, k);
                }
            }
        }
    }

    /// One game between `i` and `k`: both add their payoff (less the
    /// metabolism when charged per game); with immediate death, either whose
    /// wealth goes negative dies.
    fn game(&mut self, i: usize, k: usize) {
        let (a, b) = (self.agents[i].cooperator, self.agents[k].cooperator);
        let per_game = self.config.metabolism_per == MetabolismPer::Interaction;
        let m = self.config.metabolism;
        for (me, pay) in [(i, self.config.payoff(a, b)), (k, self.config.payoff(b, a))] {
            let agent = &mut self.agents[me];
            agent.wealth += pay;
            if per_game {
                agent.wealth -= m;
            }
            agent.income += pay;
            agent.games += 1;
        }
        if self.config.death_timing == DeathTiming::Immediate {
            for me in [i, k] {
                if self.agents[me].wealth < 0.0 {
                    self.die(me);
                }
            }
        }
    }

    fn die(&mut self, k: usize) {
        self.agents[k].dead = true;
        self.living -= 1;
        self.deaths += 1;
        if self.config.removal == Removal::Immediate {
            self.vacate(self.agents[k].site as usize);
        }
    }

    /// The strategy of an offspring of a parent with strategy `parent`.
    fn newborn_strategy(&mut self, parent: bool) -> bool {
        let flip = self.rng.gen::<f64>() < self.config.mutation;
        parent != flip
    }

    /// Rule 3: with wealth ≥ `fission_wealth` and an unoccupied neighbouring
    /// site (soup: anywhere), an offspring there with the endowment.
    fn reproduce(&mut self, i: usize) {
        if self.agents[i].wealth < self.config.fission_wealth {
            return;
        }
        let target = match self.config.pairing {
            Pairing::Space => {
                let free: Vec<usize> = self
                    .geometry
                    .neighbors(self.agents[i].site as usize)
                    .iter()
                    .map(|&j| j as usize)
                    .filter(|&j| self.at[j] == NONE)
                    .collect();
                self.pick(&free)
            }
            Pairing::Soup => self.random_empty(),
        };
        let Some(site) = target else {
            return;
        };
        let endowment = self.config.endowment;
        if self.config.endowment_from == EndowmentFrom::Parent {
            self.agents[i].wealth -= endowment;
        }
        let cooperator = self.newborn_strategy(self.agents[i].cooperator);
        let age = match self.config.newborn_age {
            NewbornAge::Random => self.random_age(),
            NewbornAge::Zero => 0,
        };
        self.add(site, cooperator, endowment, age);
        self.births += 1;
    }

    /// Rule 4: age by one, pay the per-cycle metabolism, and die with
    /// negative wealth or past the maximum age.
    fn age_and_die(&mut self, i: usize) {
        let c = &self.config;
        let (per_cycle, m, max_age) = (
            c.metabolism_per == MetabolismPer::Cycle,
            c.metabolism,
            c.max_age,
        );
        let a = &mut self.agents[i];
        a.age = a.age.saturating_add(1);
        if per_cycle {
            a.wealth -= m;
        }
        if a.wealth < 0.0 || (max_age > 0 && a.age > max_age) {
            self.die(i);
        }
    }

    /// After the list: the dead leave their sites and the list, and the
    /// list is reordered.
    fn end_cycle(&mut self) {
        for k in 0..self.agents.len() {
            let site = self.agents[k].site as usize;
            if self.agents[k].dead && self.at[site] == k as u32 {
                self.vacate(site);
            }
        }
        self.agents.retain(|a| !a.dead);
        match self.config.shuffle {
            Shuffle::Swaps => {
                let n = self.agents.len() as u32;
                for _ in 0..n / 2 {
                    let a = self.rng.gen_range(0..n) as usize;
                    let b = self.rng.gen_range(0..n) as usize;
                    self.agents.swap(a, b);
                }
            }
            Shuffle::Full => self.agents.shuffle(&mut self.rng),
        }
        for (k, a) in self.agents.iter().enumerate() {
            self.at[a.site as usize] = k as u32;
        }
        debug_assert_eq!(self.living as usize, self.agents.len());
    }

    /// Whether the living agent on `site` is a cooperator all eight of whose
    /// Moore neighbours are living cooperators.
    pub fn surrounded(&self, site: usize) -> bool {
        self.living_at(site)
            .is_some_and(|k| self.agents[k].cooperator)
            && self.moore.neighbors(site).iter().all(|&j| {
                self.living_at(j as usize)
                    .is_some_and(|k| self.agents[k].cooperator)
            })
    }

    fn record(&mut self) {
        let (mut c, mut d, mut wc, mut wd, mut surrounded) = (0u32, 0u32, 0.0, 0.0, 0u32);
        for a in self.agents.iter().filter(|a| !a.dead) {
            if a.cooperator {
                c += 1;
                wc += a.wealth;
                surrounded += u32::from(self.surrounded(a.site as usize));
            } else {
                d += 1;
                wd += a.wealth;
            }
        }
        let s = DpdSnapshot {
            tick: self.tick,
            cooperators: c,
            defectors: d,
            population: c + d,
            cooperator_share: ratio(f64::from(c), c + d),
            surrounded,
            wealth_c: ratio(wc, c),
            wealth_d: ratio(wd, d),
            births: self.births,
            deaths: self.deaths,
        };
        self.stats.push(s);
    }

    fn color(&self, a: &Agent, mode: DpdMode) -> Rgb {
        let own = if a.cooperator { COOPERATOR } else { DEFECTOR };
        match mode {
            DpdMode::Strategy => own,
            DpdMode::Wealth => {
                let top = 2.0 * self.config.fission_wealth.max(1.0);
                lerp(COOL, HOT, a.wealth / top)
            }
            DpdMode::Age => {
                let top = match self.config.max_age {
                    0 => 1000.0,
                    m => f64::from(m),
                };
                lerp(COOL, HOT, f64::from(a.age) / top)
            }
            DpdMode::Surrounded => {
                if self.surrounded(a.site as usize) {
                    COOPERATOR
                } else {
                    lerp(BACKGROUND, own, 0.35)
                }
            }
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<DpdInspection, String> {
        let site = self
            .geometry
            .at(x, y, 0)
            .ok_or_else(|| format!("({x}, {y}) is outside the lattice"))?;
        let letter = |c: bool| if c { "C" } else { "D" };
        let agent = self.agent_at(site).map(|a| {
            let neighbors = self
                .geometry
                .neighbors(site)
                .iter()
                .filter_map(|&j| {
                    let b = self.agent_at(j as usize)?;
                    let (x, y, _) = self.geometry.xyz(j as usize);
                    Some(NeighborView {
                        x,
                        y,
                        id: b.id,
                        strategy: letter(b.cooperator),
                        payoff: self.config.payoff(a.cooperator, b.cooperator),
                        their_payoff: self.config.payoff(b.cooperator, a.cooperator),
                    })
                })
                .collect();
            AgentView {
                id: a.id,
                strategy: letter(a.cooperator),
                wealth: a.wealth,
                age: a.age,
                max_age: self.config.max_age,
                surrounded: self.surrounded(site),
                income: a.income,
                games: a.games,
                neighbors,
            }
        });
        Ok(DpdInspection {
            site: Site { x, y },
            agent,
        })
    }
}

impl Model for DpdWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Dpd(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        DpdWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        DpdWorld::population(self)
    }

    /// FNV-1a over the tick, the id counter and every agent in call order.
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
            eat(u64::from(a.site) | u64::from(a.cooperator) << 32);
            eat(a.wealth.to_bits());
            eat(u64::from(a.age));
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.width)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: DpdMode = mode.parse()?;
        buf.clear();
        buf.resize(self.at.len() * 4, 0);
        for (s, px) in buf.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let rgb = self.agent_at(s).map_or(BACKGROUND, |a| self.color(a, mode));
            px.copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
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
        let mut out = String::from("id,x,y,strategy,wealth,age,surrounded\n");
        for a in self.agents() {
            let (x, y, _) = self.geometry.xyz(a.site as usize);
            writeln!(
                out,
                "{},{x},{y},{},{},{},{}",
                a.id,
                if a.cooperator { "C" } else { "D" },
                a.wealth,
                a.age,
                self.surrounded(a.site as usize)
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
        let a = self.agents().find(|a| a.id == id)?;
        let (x, y, _) = self.geometry.xyz(a.site as usize);
        Some((x, y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Dpd(next) = next else {
            return Err(wrong_model(ModelKind::Dpd, &next));
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

`presets.rs` (Decisions 10, 17; descriptions carry Decision M's numbers):

```rust
//! Epstein's runs, the working paper's rule, soup, the shifted payoffs and
//! metabolism, footnote 27, Radax & Rengs' best fit and the coordination
//! game. Descriptions quote measurements (release, seeds 1–30, the value at
//! cycle 500 unless noted, mean ± s.d. (range), recorded 2026-09-26).

use super::config::{DpdConfig, EndowmentFrom, NewbornsAct, Pairing, Play, Removal};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut DpdConfig),
) -> ModelPreset {
    let mut c = DpdConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Dpd(c),
    }
}

const GSS: &str = "Epstein, Generative Social Science (2006), ch. 9";
const WP: &str = "Epstein, SFI Working Paper 97-12-094 (1997)";
const APPENDIX: &str = "Epstein, Generative Social Science (2006), ch. 9 appendix";
const RR: &str = "Radax & Rengs 2009, MPRA 14419";

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "dpd-run-1",
            "Run 1: no maximum age",
            GSS,
            "Epstein's Table 9.1: 100 agents with random fixed strategies on a 30 × 30 torus, wealth 6 (the CD's). In turn each moves to a random unoccupied neighbouring site, plays each neighbour (T 6, R 5, P −5, S −6), clones onto an empty neighbouring site once its wealth reaches 11 (giving the offspring 6), and dies if its wealth goes negative. Table 1: 779 ± 15 cooperators, 121 ± 15 defectors. Measured: 729 ± 17 (698–778) and 171 ± 17 (122–201) — cooperation dominates, but 50 fewer cooperators than Epstein (t = 11.9: not reproduced), and about 4.3 to 1 by t = 50, not 5 to 1.",
            |_| {},
        ),
        preset(
            "dpd-run-2",
            "Run 2: maximum age 100",
            GSS,
            "Run 1 with a maximum age of 100 (initial and newborn ages random in 1–100). Table 2: 784 ± 29 cooperators, 99 ± 25 defectors. Measured: 695 ± 29 (628–741) and 196 ± 28 (152–263) — twice Epstein's defectors (not reproduced). Radax & Rengs' factorial of the six timing switches finds one setting of 64 that matches (removal at once, death on the agent's own turn, a full shuffle): 785 ± 23 / 110 ± 22.",
            |c| c.max_age = 100,
        ),
        preset(
            "dpd-run-3",
            "Run 3: R = 2",
            GSS,
            "Run 2 with the reward for mutual cooperation cut from 5 to 2: \"cooperators do worse … defectors do better, and the oscillations are now more evident.\" Measured: 339 ± 162 cooperators, 209 ± 98 defectors; 5 of 30 populations die out by cycle 500 (Radax & Rengs saw 250–450 cooperators and about 200 defectors).",
            |c| {
                c.max_age = 100;
                c.r = 2.0;
            },
        ),
        preset(
            "dpd-run-4",
            "Run 4: R = 1",
            GSS,
            "R cut to 1: Epstein describes predator–prey cycles of cooperative zones, with outcomes that differ by seed — coexistence, cooperator monopoly or extinction — and cooperators that \"ultimately do better with a low payoff (R = 1) than with a high one (R = 5)\". Measured: 26 of 30 populations are extinct by cycle 500 and all 30 by 2,000, after at most two swings (cooperators above 400 then below 100; 0.6 per run); no cooperator monopoly at R = 1 (and none at R = 5, where all 30 coexist). Not reproduced.",
            |c| {
                c.max_age = 100;
                c.r = 1.0;
            },
        ),
        preset(
            "dpd-run-5",
            "Run 5: 50% mutation",
            GSS,
            "Run 2 with a 50% chance that an offspring takes the other strategy: cooperation \"is intact through 10 thousand cycles\". Measured: 268 ± 87 cooperators and 306 ± 91 defectors at cycle 500; cooperators persist through 10,000 cycles in 27 of 30 runs (the other three populations die out entirely), averaging 260 against 295 defectors over cycles 5,001–10,000. At 25% mutation (Epstein: about 350 and 400): 420 and 393.",
            |c| {
                c.max_age = 100;
                c.mutation = 0.5;
            },
        ),
        preset(
            "dpd-working-paper",
            "The working paper's rule",
            WP,
            "The 1997 working paper's rule: \"Choose a random site within your vision; go there and play your strategy against a random neighbor\" — one game a turn, where the published text plays each neighbour. On Run 1's settings: 759 ± 18 cooperators, 141 ± 18 defectors — nearer Table 1 (779/121) than the published rule (729/171), but still rejected (t = 4.8); on Run 2's: 737 ± 21 and 156 ± 21.",
            |c| c.play = Play::RandomNeighbor,
        ),
        preset(
            "dpd-closest",
            "Closest to Tables 1 and 2",
            WP,
            "Three choices no source settles, changed from the defaults: the working paper's one random neighbour a turn, initial agents with no wealth (the prose never gives one; 6 is the 2006 CD's), and newborns acting in the cycle they are born. On Run 1's settings: 786 ± 21 cooperators, 114 ± 21 defectors (Table 1: 779 ± 15, 121 ± 15; t = −1.5); with a maximum age of 100, 792 ± 24 and 101 ± 23 (Table 2: 784 ± 29, 99 ± 25; t = −1.2) — both tables at once, which none of Radax & Rengs' 64 timing settings does with the published rule. It is fragile: over seeds 31–60 and 61–90 Run 1 still holds but Run 2's defectors come out 116 and 121 (t = −2.6, −3.2); of 512 combinations of these three choices with the six timing switches, 46 reproduce Run 1 (every one with no initial wealth), 24 Run 2 and 7 both.",
            |c| {
                c.play = Play::RandomNeighbor;
                c.initial_wealth = 0.0;
                c.newborns_act = NewbornsAct::ThisCycle;
            },
        ),
        preset(
            "dpd-soup",
            "Soup",
            GSS,
            "\"If we replace space and local interactions with soup (equiprobable random agent pairings), then the system runs to pure defection.\" Each agent moves to a random empty site anywhere, plays one random other agent and places offspring anywhere. Measured: the last cooperator dies by cycle 8 (4–14) in 29 of 30 runs; the defectors then kill one another, leaving a lone agent or nobody by cycle 500 — reproduced.",
            |c| c.pairing = Pairing::Soup,
        ),
        preset(
            "dpd-shifted",
            "Payoffs shifted by 6",
            GSS,
            "GSS: \"with maximum age of 100, zero mutation, and all payoffs shifted up by 6, so that T = 12, R = 11, P = 1, and S = 0, the spatial system again converges to pure defection\" (Fig. 9.13). Measured: no run converges — all 30 coexist at cycle 500 (418 ± 78 cooperators, 478 ± 77 defectors) and at 2,000. With nothing negative only old age kills, the lattice stays full and everyone can afford to clone, so births go to whoever finds an empty site. Run 5's 50% mutation (the working paper's context) or Run 1's unlimited lives do not converge either. Not reproduced.",
            shifted,
        ),
        preset(
            "dpd-metabolism",
            "Shifted payoffs, metabolism 6",
            GSS,
            "Epstein: the shifted payoffs with a metabolism of 6 are \"equivalent mathematically\" to the negative payoffs, metabolism being \"a fixed decrement to accumulated payoff per cycle\". Charged per cycle (this preset): 644 ± 97 cooperators, 253 ± 97 defectors against Run 2's 695 and 196 — cooperative, but another model. Charged per game (metabolism_per: interaction, the text's \"after every interaction\"), the run is Run 2 exactly, to the fingerprint.",
            |c| {
                shifted(c);
                c.metabolism = 6.0;
            },
        ),
        preset(
            "dpd-footnote-27",
            "Footnote 27",
            GSS,
            "Footnote 27: \"with all payoffs hiked by ten (so that T = 16, R = 11, P = 5, S = 4) and the maximum lifetime reduced from 100 to 10 cycles … we again see an evolution to cooperative monopoly.\" (Hiked by ten, R would be 15.) Measured: all 30 runs coexist at cycle 500 (385 ± 158 cooperators, 485 ± 158 defectors); by 2,000 one run of 30 is a cooperative monopoly. R = 15 gives the same. Not reproduced.",
            |c| {
                (c.t, c.r, c.p, c.s) = (16.0, 11.0, 5.0, 4.0);
                c.max_age = 10;
            },
        ),
        preset(
            "dpd-rr-best",
            "Radax & Rengs' best fit",
            RR,
            "Radax & Rengs' best fit to Table 2 (their Repast replication: 780 ± 25 cooperators, 97 ± 22 defectors): the dead leave at the end of the cycle, death as wealth goes negative, the endowment granted rather than taken from the parent, random newborn ages, asynchronous updating, Epstein's swaps. Here: 703 ± 26 and 164 ± 23 (t = 11.3 against Table 2) — their best fit does not carry over to this implementation of the same switches.",
            |c| {
                c.max_age = 100;
                c.removal = Removal::EndOfCycle;
                c.endowment_from = EndowmentFrom::Granted;
            },
        ),
        preset(
            "dpd-coordination",
            "The coordination game",
            APPENDIX,
            "GSS's appendix: the same demography with the CD's coordination payoffs [1, −3, −3, 1] (two conventions — say, driving on the left or the right — paying 1 when neighbours match and −3 when they do not), maximum age 1,000, no mutation: persistent \"norm maps\", regions of each convention with accidents at their borders. Measured: both conventions persist in 17 of 30 runs at cycle 500, 16 at 2,000 and 11 at 5,000 (the rest settle on one); where both persist 3.6% of neighbouring pairs differ, against 35% if mixed at random.",
            |c| {
                (c.r, c.s, c.t, c.p) = (1.0, -3.0, -3.0, 1.0);
                c.max_age = 1000;
            },
        ),
    ]
}

/// GSS p. 216: "maximum age of 100, zero mutation, and all payoffs shifted
/// up by 6" (12, 11, 1, 0).
fn shifted(c: &mut DpdConfig) {
    c.max_age = 100;
    (c.t, c.r, c.p, c.s) = (12.0, 11.0, 1.0, 0.0);
}
```

- [ ] **Step 3b: The Review Focus tests**

Append inside `crates/sugarscape-core/src/dpd/world.rs`'s `mod tests`, before its closing brace (they use the module's `world`, `put`, `xy` and `empty_list_is_consistent` helpers):

```rust
    #[test]
    fn a_full_lattice_starts_and_runs_with_nowhere_to_move_or_clone() {
        let mut w = world(4, |c| c.agents = 16);
        assert_eq!(w.population(), 16);
        empty_list_is_consistent(&w);
        w.run(20);
        assert!(w.population() <= 16);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_lone_agent_plays_no_one_in_space_or_soup() {
        for pairing in [Pairing::Space, Pairing::Soup] {
            let mut w = world(5, |c| c.pairing = pairing);
            let a = put(&mut w, (2, 2), false, 5.0);
            w.run(3);
            assert_eq!(w.population(), 1, "{pairing:?}");
            assert_eq!(w.agents[a].wealth, 5.0, "{pairing:?}: nobody to play");
            empty_list_is_consistent(&w);
        }
    }

    #[test]
    fn an_endowment_above_the_parents_wealth_kills_the_parent() {
        // The rules as written: the endowment comes from the parent, so a
        // parent at the threshold that gives more than it has goes negative.
        let mut w = world(5, |c| c.endowment = 20.0);
        put(&mut w, (2, 2), true, 11.0);
        w.step();
        assert_eq!(w.population(), 1, "the offspring lives, the parent dies");
        assert_eq!(w.agents[0].wealth, 20.0);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_zero_fission_wealth_fills_the_lattice_without_breaking_it() {
        let mut w = world(6, |c| {
            c.agents = 3;
            c.fission_wealth = 0.0;
            c.endowment = 0.0;
        });
        w.run(30);
        assert!(w.population() <= 36);
        empty_list_is_consistent(&w);
    }

    #[test]
    fn a_vision_wider_than_the_lattice_reaches_each_site_once() {
        let mut w = world(3, |c| c.vision = 10);
        let a = put(&mut w, (1, 1), true, 5.0);
        w.step();
        assert_ne!(xy(&w, a), (1, 1), "eight empty sites are within reach");
        empty_list_is_consistent(&w);
    }
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --lib dpd && cargo test -p sugarscape-core --lib model:: && cargo test -p sugarscape-core --test golden && cargo test -p sugarscape-core --test checkpoint`
Expected: PASS — 31 tests matching `dpd` (7 config, 23 world, and `model::tests::dpd_configs_round_trip_with_their_tag`), the model tests, every golden entry (the thirteen new ones as listed; if one differs, stop and report), the keyframe tests. Then `cargo test --workspace` — PASS (the core's lib: 589 tests).

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/dpd crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/src/main.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/checkpoint.rs
git commit -m "Add Epstein's demographic Prisoner's Dilemma as the dpd model kind" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 2: Measured against the sources — book-style tests, four sweeps, WASM

**Files:**
- Create: `crates/sugarscape-core/tests/dpd.rs`, `sweeps/dpd-payoffs.json`, `sweeps/dpd-mutation.json`, `sweeps/dpd-metabolism.json`, `sweeps/dpd-max-age.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets, `DpdWorld`, `DpdConfig` and its switch enums, `Model::fingerprint`.
- Produces: built-in sweep ids `dpd-payoffs`, `dpd-mutation`, `dpd-metabolism`, `dpd-max-age` (in that order, after `jansson-markers`).

- [ ] **Step 1: The book-style tests**

Create `crates/sugarscape-core/tests/dpd.rs` (every test `#[ignore]`, release; the claims and Decision M's numbers):

```rust
//! The demographic Prisoner's Dilemma (milestone 17) against Epstein's
//! working paper (1997) and chapter (GSS 2006, ch. 9 and its appendix) and
//! Radax & Rengs' replication (RR, 2009). Every claim runs over seeds 1–30
//! (Epstein's and RR's 30 runs) in release: `cargo test -p sugarscape-core
//! --release --test dpd -- --ignored --nocapture`. A run's value is its
//! count at cycle 500 unless noted. Thresholds come from the measurements
//! recorded 2026-09-26 (docs/superpowers/plans/2026-09-26-demographic-pd.md,
//! the measurements decision); claims that do not hold are pinned as
//! measured.

use std::thread;

use sugarscape_core::dpd::{
    DeathTiming, DpdConfig, DpdWorld, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct,
    Pairing, Play, Removal, Shuffle, Updating,
};
use sugarscape_core::model::Model;

/// Runs `c` for `ticks` from seeds 1–30 in parallel and measures each run.
fn each_seed<T: Send>(c: &DpdConfig, ticks: u32, f: impl Fn(&DpdWorld) -> T + Sync) -> Vec<T> {
    thread::scope(|s| {
        let handles: Vec<_> = (1..=30u64)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = DpdWorld::new(c.clone(), seed).unwrap();
                    w.run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn last(w: &DpdWorld, name: &str) -> f64 {
    *w.stats.series(name).unwrap().last().unwrap()
}

/// Mean and sample standard deviation.
fn mean_sd(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0);
    (m, var.sqrt())
}

/// (cooperators, defectors) at cycle `ticks`: each (mean, s.d.).
fn counts(c: &DpdConfig, ticks: u32) -> ((f64, f64), (f64, f64)) {
    let v = each_seed(c, ticks, |w| (last(w, "cooperators"), last(w, "defectors")));
    let cs: Vec<f64> = v.iter().map(|x| x.0).collect();
    let ds: Vec<f64> = v.iter().map(|x| x.1).collect();
    (mean_sd(&cs), mean_sd(&ds))
}

/// RR's t statistic for the equality of two means of 30 runs with the same
/// unknown variance, (source − ours) / s.e.; |t| < 2.0017 (df 58, α = 0.05)
/// is indistinguishable. With equal samples it is also Welch's statistic.
fn t_stat(source: (f64, f64), ours: (f64, f64)) -> f64 {
    let pooled = (source.1 * source.1 + ours.1 * ours.1) / 2.0;
    (source.0 - ours.0) / (pooled * 2.0 / 30.0).sqrt()
}

const T_CRIT: f64 = 2.0017;

fn near(ours: f64, target: f64, within: f64) -> bool {
    (ours - target).abs() <= within
}

fn run_1() -> DpdConfig {
    DpdConfig::default()
}

fn run_2() -> DpdConfig {
    DpdConfig {
        max_age: 100,
        ..Default::default()
    }
}

/// How each run ends: (coexistence, cooperators only, defectors only, nobody).
fn outcomes(c: &DpdConfig, ticks: u32) -> (usize, usize, usize, usize) {
    let v = each_seed(c, ticks, |w| (last(w, "cooperators"), last(w, "defectors")));
    let count = |f: fn(f64, f64) -> bool| v.iter().filter(|x| f(x.0, x.1)).count();
    (
        count(|c, d| c > 0.0 && d > 0.0),
        count(|c, d| c > 0.0 && d == 0.0),
        count(|c, d| c == 0.0 && d > 0.0),
        count(|c, d| c == 0.0 && d == 0.0),
    )
}

#[test]
#[ignore]
fn tables_1_and_2_are_not_reproduced() {
    // (run, config, ours C, ours D, Epstein C (s.d.), Epstein D (s.d.)).
    for (name, c, oc, od, ec, ed) in [
        ("Run 1", run_1(), 729.2, 170.5, (779.0, 15.0), (121.0, 15.0)),
        ("Run 2", run_2(), 694.8, 195.9, (784.0, 29.0), (99.0, 25.0)),
    ] {
        let (co, de) = counts(&c, 500);
        println!("{name}: C {co:.1?} D {de:.1?} (Epstein {ec:?} {ed:?})");
        assert!(near(co.0, oc, 0.06) && near(de.0, od, 0.06), "{name}");
        // Cooperation dominates, as Epstein's, but both t-tests reject.
        assert!(co.0 > 3.0 * de.0);
        assert!(t_stat(ec, co).abs() > T_CRIT && t_stat(ed, de).abs() > T_CRIT);
    }
}

#[test]
#[ignore]
fn run_1_is_about_4_to_1_by_t_50_not_5_to_1() {
    let (co, de) = counts(&run_1(), 50);
    let ratio = co.0 / de.0;
    println!("t = 50: {:.1} / {:.1} = {ratio:.2}", co.0, de.0);
    // Epstein: "a stable ratio of cooperators to defectors (approximately 5
    // to 1)" by t = 50. Measured 4.34.
    assert!(near(ratio, 4.34, 0.005), "{ratio}");
}

/// GSS Table 9.3 as printed: (T, R, cooperators' mean, 95% CI, range,
/// defectors' mean, 95% CI, range).
type Cell = (
    u32,
    u32,
    f64,
    (f64, f64),
    (f64, f64),
    f64,
    (f64, f64),
    (f64, f64),
);
const TABLE_9_3: [Cell; 45] = [
    (
        10,
        9,
        809.,
        (802., 816.),
        (772., 845.),
        77.,
        (71., 83.),
        (43., 109.),
    ),
    (
        10,
        8,
        748.,
        (738., 757.),
        (707., 802.),
        132.,
        (124., 140.),
        (82., 169.),
    ),
    (
        10,
        7,
        654.,
        (641., 668.),
        (568., 717.),
        198.,
        (188., 209.),
        (154., 280.),
    ),
    (
        10,
        6,
        469.,
        (447., 490.),
        (312., 604.),
        274.,
        (264., 285.),
        (191., 329.),
    ),
    (
        10,
        5,
        258.,
        (240., 277.),
        (150., 383.),
        270.,
        (261., 279.),
        (202., 319.),
    ),
    (
        10,
        4,
        231.,
        (177., 285.),
        (0., 598.),
        199.,
        (172., 225.),
        (0., 287.),
    ),
    (10, 3, 0., (0., 0.), (0., 1.), 0., (0., 0.), (0., 1.)),
    (10, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (10, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        9,
        8,
        806.,
        (799., 814.),
        (766., 850.),
        81.,
        (74., 88.),
        (39., 117.),
    ),
    (
        9,
        7,
        728.,
        (716., 740.),
        (669., 793.),
        146.,
        (136., 156.),
        (86., 190.),
    ),
    (
        9,
        6,
        604.,
        (583., 625.),
        (490., 768.),
        225.,
        (212., 239.),
        (102., 303.),
    ),
    (
        9,
        5,
        374.,
        (358., 390.),
        (268., 460.),
        290.,
        (283., 297.),
        (252., 323.),
    ),
    (
        9,
        4,
        203.,
        (175., 231.),
        (98., 442.),
        235.,
        (222., 248.),
        (159., 296.),
    ),
    (9, 3, 35., (3., 67.), (0., 382.), 38., (8., 68.), (0., 284.)),
    (9, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (9, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        8,
        7,
        807.,
        (796., 818.),
        (744., 879.),
        80.,
        (70., 89.),
        (14., 142.),
    ),
    (
        8,
        6,
        721.,
        (708., 734.),
        (626., 787.),
        153.,
        (142., 163.),
        (98., 231.),
    ),
    (
        8,
        5,
        530.,
        (516., 544.),
        (460., 604.),
        263.,
        (255., 271.),
        (209., 312.),
    ),
    (
        8,
        4,
        259.,
        (239., 279.),
        (128., 387.),
        271.,
        (263., 279.),
        (227., 313.),
    ),
    (
        8,
        3,
        93.,
        (53., 134.),
        (0., 513.),
        113.,
        (77., 148.),
        (0., 279.),
    ),
    (8, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (8, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        7,
        6,
        797.,
        (787., 807.),
        (739., 852.),
        88.,
        (79., 97.),
        (43., 133.),
    ),
    (
        7,
        5,
        668.,
        (652., 684.),
        (542., 782.),
        187.,
        (175., 199.),
        (101., 271.),
    ),
    (
        7,
        4,
        430.,
        (410., 449.),
        (323., 547.),
        286.,
        (277., 295.),
        (244., 345.),
    ),
    (
        7,
        3,
        126.,
        (90., 162.),
        (0., 370.),
        153.,
        (126., 180.),
        (0., 267.),
    ),
    (7, 2, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (7, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        6,
        5,
        779.,
        (773., 784.),
        (752., 806.),
        121.,
        (115., 126.),
        (93., 148.),
    ),
    (
        6,
        4,
        587.,
        (576., 599.),
        (524., 658.),
        241.,
        (233., 248.),
        (193., 278.),
    ),
    (
        6,
        3,
        266.,
        (247., 285.),
        (120., 344.),
        280.,
        (270., 291.),
        (199., 320.),
    ),
    (6, 2, 24., (0., 57.), (0., 482.), 13., (0., 27.), (0., 166.)),
    (6, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        5,
        4,
        741.,
        (729., 752.),
        (689., 810.),
        136.,
        (126., 146.),
        (79., 179.),
    ),
    (
        5,
        3,
        473.,
        (456., 489.),
        (379., 574.),
        283.,
        (274., 291.),
        (243., 329.),
    ),
    (
        5,
        2,
        125.,
        (70., 180.),
        (0., 589.),
        114.,
        (82., 145.),
        (0., 260.),
    ),
    (5, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        4,
        3,
        710.,
        (693., 727.),
        (613., 814.),
        159.,
        (146., 172.),
        (76., 231.),
    ),
    // The printed defectors' CI "(254, 376)" contradicts mean 265, s.d. 32.
    (
        4,
        2,
        282.,
        (262., 302.),
        (171., 380.),
        265.,
        (254., 376.),
        (176., 322.),
    ),
    (4, 1, 0., (0., 0.), (0., 0.), 0., (0., 0.), (0., 0.)),
    (
        3,
        2,
        624.,
        (610., 637.),
        (516., 699.),
        221.,
        (210., 232.),
        (164., 284.),
    ),
    (3, 1, 4., (0., 12.), (0., 123.), 7., (0., 20.), (0., 197.)),
    (
        2,
        1,
        361.,
        (344., 377.),
        (256., 471.),
        280.,
        (273., 288.),
        (234., 337.),
    ),
];

#[test]
#[ignore]
fn table_9_3_cell_by_cell() {
    // A mean counts as inside an interval Epstein printed rounded if it
    // rounds into it.
    let inside = |m: f64, (lo, hi): (f64, f64)| m >= lo - 0.5 && m < hi + 0.5;
    let (mut ci, mut range, mut both_ci, mut lone_defectors) = (0, 0, 0, 0);
    for (t, r, ec, eci, erange, ed, edci, edrange) in TABLE_9_3 {
        let c = DpdConfig {
            t: f64::from(t),
            r: f64::from(r),
            p: -f64::from(r),
            s: -f64::from(t),
            ..Default::default()
        };
        let (co, de) = counts(&c, 500);
        let (c_ci, d_ci) = (inside(co.0, eci), inside(de.0, edci));
        ci += usize::from(c_ci) + usize::from(d_ci);
        range += usize::from(inside(co.0, erange)) + usize::from(inside(de.0, edrange));
        both_ci += usize::from(c_ci && d_ci);
        // Where Epstein's whole population dies out, lone defectors remain.
        if ec == 0.0 && ed == 0.0 && erange.1 == 0.0 && de.0 > 0.0 {
            lone_defectors += 1;
        }
        println!(
            "({t}, {r}): C {:.1} [{ec}] D {:.1} [{ed}] {c_ci} {d_ci}",
            co.0, de.0
        );
    }
    println!("in CI {ci}/90, in range {range}/90, both in CI {both_ci}/45, lone defectors in {lone_defectors} cells");
    assert_eq!((ci, range, both_ci), (22, 51, 1));
    assert_eq!(
        lone_defectors, 11,
        "every all-zero cell keeps a few defectors"
    );
}

#[test]
#[ignore]
fn collapsed_populations_leave_lone_defectors_epstein_counts_none() {
    // Table 9.3 (10, 1): (0, 0) cooperators and defectors in every run. Here
    // the last defectors have nobody to play and no maximum age: they stay.
    let c = DpdConfig {
        t: 10.0,
        r: 1.0,
        p: -1.0,
        s: -10.0,
        ..Default::default()
    };
    let (co, de) = counts(&c, 500);
    assert_eq!(co.0, 0.0);
    assert!(near(de.0, 2.7, 0.06), "{de:?}");
    let (_, de) = counts(&c, 2000);
    assert!(de.0 >= 1.0, "{de:?}");
}

/// Swings of the cooperator count from above 400 to below 100.
fn swings(w: &DpdWorld) -> u32 {
    let (mut high, mut n) = (false, 0);
    for x in w.stats.series("cooperators").unwrap() {
        if x > 400.0 {
            high = true;
        } else if x < 100.0 && high {
            high = false;
            n += 1;
        }
    }
    n
}

#[test]
#[ignore]
fn run_4_dies_out_instead_of_cycling_and_the_paradox_does_not_appear() {
    let r1 = DpdConfig { r: 1.0, ..run_2() };
    // Epstein: predator–prey cycles; coexistence, cooperator monopoly or
    // extinction by seed; monopolies at R = 1 but not at R = 5.
    assert_eq!(outcomes(&r1, 500), (4, 0, 0, 26));
    assert_eq!(outcomes(&r1, 2000), (0, 0, 0, 30));
    let s: Vec<f64> = each_seed(&r1, 2000, |w| f64::from(swings(w)));
    let (m, _) = mean_sd(&s);
    assert!(near(m, 0.6, 0.02), "{m}");
    assert!(s.iter().all(|&x| x <= 2.0));
    assert_eq!(outcomes(&run_2(), 2000), (30, 0, 0, 0));
}

#[test]
#[ignore]
fn run_5_cooperation_persists_through_10000_cycles_in_27_of_30() {
    let c = DpdConfig {
        mutation: 0.5,
        ..run_2()
    };
    let v = each_seed(&c, 10_000, |w| {
        (last(w, "cooperators"), last(w, "population"))
    });
    let alive = v.iter().filter(|x| x.0 > 0.0).count();
    let empty = v.iter().filter(|x| x.1 == 0.0).count();
    assert_eq!((alive, empty), (27, 3), "the others die out entirely");
}

#[test]
#[ignore]
fn soup_runs_to_pure_defection() {
    let c = DpdConfig {
        pairing: Pairing::Soup,
        ..run_1()
    };
    let v = each_seed(&c, 500, |w| {
        let cs = w.stats.series("cooperators").unwrap();
        (cs.iter().position(|&x| x == 0.0), last(w, "population"))
    });
    let gone: Vec<usize> = v.iter().filter_map(|x| x.0).collect();
    assert_eq!(gone.len(), 29);
    assert!(gone.iter().all(|&t| t <= 14));
    assert!(v.iter().all(|x| x.1 <= 1.0), "a lone agent at most");
}

#[test]
#[ignore]
fn shifted_payoffs_do_not_converge_to_pure_defection() {
    let shifted = |c: DpdConfig| DpdConfig {
        t: 12.0,
        r: 11.0,
        p: 1.0,
        s: 0.0,
        ..c
    };
    // GSS: "maximum age of 100, zero mutation": all 30 coexist.
    assert_eq!(outcomes(&shifted(run_2()), 500), (30, 0, 0, 0));
    assert_eq!(outcomes(&shifted(run_2()), 2000), (30, 0, 0, 0));
    let (co, de) = counts(&shifted(run_2()), 500);
    assert!(near(co.0, 418.1, 0.06) && near(de.0, 478.1, 0.06));
    // Run 5's settings (the working paper's context) and Run 1's: the same.
    let run_5 = DpdConfig {
        mutation: 0.5,
        ..run_2()
    };
    assert_eq!(outcomes(&shifted(run_5), 500), (30, 0, 0, 0));
    assert_eq!(outcomes(&shifted(run_1()), 500), (30, 0, 0, 0));
}

#[test]
#[ignore]
fn metabolism_recovers_the_negative_payoffs_per_game_not_per_cycle() {
    let shifted = |per: MetabolismPer| DpdConfig {
        t: 12.0,
        r: 11.0,
        p: 1.0,
        s: 0.0,
        metabolism: 6.0,
        metabolism_per: per,
        ..run_2()
    };
    let per_game = each_seed(&shifted(MetabolismPer::Interaction), 500, |w| {
        w.fingerprint()
    });
    let negative = each_seed(&run_2(), 500, |w| w.fingerprint());
    assert_eq!(per_game, negative, "per game: Run 2 exactly");
    let (co, de) = counts(&shifted(MetabolismPer::Cycle), 500);
    assert!(near(co.0, 644.2, 0.06) && near(de.0, 252.6, 0.06));
    // Cooperative, but distinguishable from Run 2 (694.8 ± 29.4).
    assert!(t_stat((694.8, 29.4), co).abs() > T_CRIT);
}

#[test]
#[ignore]
fn footnote_27_does_not_reach_cooperative_monopoly() {
    for r in [11.0, 15.0] {
        let c = DpdConfig {
            t: 16.0,
            r,
            p: 5.0,
            s: 4.0,
            max_age: 10,
            ..Default::default()
        };
        assert_eq!(outcomes(&c, 500), (30, 0, 0, 0), "R = {r}");
        assert_eq!(outcomes(&c, 2000).1, 1, "R = {r}: one monopoly by 2,000");
    }
}

#[test]
#[ignore]
fn coordination_regions_persist_in_about_half_the_runs() {
    let c = DpdConfig {
        r: 1.0,
        s: -3.0,
        t: -3.0,
        p: 1.0,
        max_age: 1000,
        ..Default::default()
    };
    // Both conventions present, and the share of neighbouring pairs that
    // differ (per mille).
    let measure = |w: &DpdWorld| {
        let (mut unlike, mut pairs) = (0u32, 0u32);
        for s in 0..w.sites() {
            let Some(a) = w.agent_at(s) else { continue };
            for &j in w.geometry.neighbors(s) {
                if let Some(b) = w.agent_at(j as usize) {
                    pairs += 1;
                    unlike += u32::from(a.cooperator != b.cooperator);
                }
            }
        }
        let c = last(w, "cooperators");
        let mixed = c > 0.0 && c < last(w, "population");
        (mixed, 1000.0 * f64::from(unlike) / f64::from(pairs.max(1)))
    };
    for (ticks, persist) in [(500, 17), (2000, 16), (5000, 11)] {
        let v = each_seed(&c, ticks, measure);
        let mixed: Vec<f64> = v.iter().filter(|x| x.0).map(|x| x.1).collect();
        assert_eq!(mixed.len(), persist, "t = {ticks}");
        // Where both persist they are regions: few unlike neighbours.
        assert!(mean_sd(&mixed).0 < 50.0, "t = {ticks}");
    }
}

#[test]
#[ignore]
fn radax_and_rengs_factorial_as_measured() {
    // RR's six model switches (their RNG library excluded), each TRUE/FALSE
    // in their order: remove dead immediately, die immediately, endowment
    // inherited, random birth age, asynchronous updating, Repast shuffle.
    // RR: 1 of 128 settings reproduces Run 1 (synchronous, which they set
    // aside), 7 reproduce Run 2, none both.
    let (mut run1, mut run2, mut both, mut fits2) = (0, 0, 0, Vec::new());
    for bits in 0..64u32 {
        let on = |k: u32| bits & (1 << (5 - k)) == 0;
        let edit = |mut c: DpdConfig| {
            c.removal = if on(0) {
                Removal::Immediate
            } else {
                Removal::EndOfCycle
            };
            c.death_timing = if on(1) {
                DeathTiming::Immediate
            } else {
                DeathTiming::OwnTurn
            };
            c.endowment_from = if on(2) {
                EndowmentFrom::Parent
            } else {
                EndowmentFrom::Granted
            };
            c.newborn_age = if on(3) {
                NewbornAge::Random
            } else {
                NewbornAge::Zero
            };
            c.updating = if on(4) {
                Updating::Asynchronous
            } else {
                Updating::Synchronous
            };
            c.shuffle = if on(5) { Shuffle::Full } else { Shuffle::Swaps };
            c
        };
        let (c1, d1) = counts(&edit(run_1()), 500);
        let (c2, d2) = counts(&edit(run_2()), 500);
        let r1 =
            t_stat((779.0, 15.0), c1).abs() < T_CRIT && t_stat((121.0, 15.0), d1).abs() < T_CRIT;
        let r2 =
            t_stat((784.0, 29.0), c2).abs() < T_CRIT && t_stat((99.0, 25.0), d2).abs() < T_CRIT;
        run1 += usize::from(r1);
        run2 += usize::from(r2);
        both += usize::from(r1 && r2);
        if r2 {
            let flags: String = (0..6).map(|k| if on(k) { 'T' } else { 'F' }).collect();
            fits2.push(flags);
        }
    }
    assert_eq!((run1, run2, both), (0, 1, 0));
    assert_eq!(fits2, ["TFTTTT"]);
    // RR's best Run 2 fit (F T F T T F: 780 ± 25 / 97 ± 22 in Repast) here.
    let best = DpdConfig {
        removal: Removal::EndOfCycle,
        endowment_from: EndowmentFrom::Granted,
        ..run_2()
    };
    let (co, de) = counts(&best, 500);
    assert!(
        near(co.0, 703.0, 0.06) && near(de.0, 163.5, 0.06),
        "{co:?} {de:?}"
    );
}

#[test]
#[ignore]
fn the_working_papers_rule_is_nearer_table_1_but_still_rejected() {
    let wp = |c: DpdConfig| DpdConfig {
        play: Play::RandomNeighbor,
        ..c
    };
    let (co, de) = counts(&wp(run_1()), 500);
    assert!(
        near(co.0, 758.9, 0.06) && near(de.0, 141.0, 0.06),
        "{co:?} {de:?}"
    );
    assert!(t_stat((779.0, 15.0), co).abs() > T_CRIT);
    let (co, de) = counts(&wp(run_2()), 500);
    assert!(
        near(co.0, 737.0, 0.06) && near(de.0, 156.2, 0.06),
        "{co:?} {de:?}"
    );
}

#[test]
#[ignore]
fn three_unsettled_choices_reproduce_both_tables_on_seeds_1_to_30() {
    // The working paper's rule, no initial wealth, newborns acting at once
    // (dpd-closest): both tables pass RR's test, where no timing setting of
    // the published rule does. Over seeds 31–60 Run 2's defectors fail.
    let closest = |c: DpdConfig| DpdConfig {
        play: Play::RandomNeighbor,
        initial_wealth: 0.0,
        newborns_act: NewbornsAct::ThisCycle,
        ..c
    };
    let (c1, d1) = counts(&closest(run_1()), 500);
    let (c2, d2) = counts(&closest(run_2()), 500);
    println!("Run 1 {c1:.1?} {d1:.1?}; Run 2 {c2:.1?} {d2:.1?}");
    assert!(near(c1.0, 786.1, 0.06) && near(d1.0, 113.7, 0.06));
    assert!(near(c2.0, 792.2, 0.06) && near(d2.0, 101.3, 0.06));
    for (source, ours) in [
        ((779.0, 15.0), c1),
        ((121.0, 15.0), d1),
        ((784.0, 29.0), c2),
        ((99.0, 25.0), d2),
    ] {
        assert!(t_stat(source, ours).abs() < T_CRIT, "{source:?} {ours:?}");
    }
}
```

Run: `cargo test -p sugarscape-core --release --test dpd -- --ignored --nocapture`
Expected: PASS — 14 tests in about 35 s on 10 threads. If a pinned number differs, stop and report.

- [ ] **Step 2: The sweeps**

`sweeps/dpd-payoffs.json`:

```json
{
  "name": "Demographic PD: Table 9.3's payoffs (Epstein)",
  "description": "Epstein's Table 9.3: T from 2 to 10 (S = −T) and R from 1 to T − 1 (P = −R), Run 1's other settings, cooperators at cycle 500 over 30 runs. The sweep format cannot skip a cell, so the 36 cells with R ≥ T also run, at the right of each line; they are not Prisoner's Dilemmas (cooperating against a cooperator pays at least as much as defecting) and cooperators take 774–876 of the 900 sites there. Measured (release, seeds 1–30, recorded 2026-09-26), cooperators for R = T − 1 down to 1 — T 10: 750, 700, 631, 424, 177, 103, 0, 0, 0; T 9: 746, 692, 526, 274, 132, 34, 0, 0; T 8: 738, 631, 476, 195, 71, 0, 0; T 7: 705, 583, 357, 129, 0, 0; T 6: 729, 634, 368, 12, 0; T 5: 726, 552, 94, 0; T 4: 689, 414, 0; T 3: 611, 9; T 2: 332. Epstein's pattern holds — cooperators dominate when R is near T and die out when R falls far enough, at the same R in every row — but of the 90 means (cooperators and defectors) only 22 fall inside his 95% confidence intervals and 51 inside his ranges; in just 1 cell of 45 do both means fall inside the intervals. Along the diagonal (R = T − 1) our cooperators run 13–92 below his and our defectors 33–106 above (bar T = 2); where his populations die out, 1–5 lone defectors survive here, with nobody to play and no maximum age to kill them.",
  "base": {"preset": "dpd-run-1"},
  "x": {
    "label": "R (P = −R)",
    "values": [
      {"at": 1, "name": "R = 1", "set": {"r": 1, "p": -1}},
      {"at": 2, "name": "R = 2", "set": {"r": 2, "p": -2}},
      {"at": 3, "name": "R = 3", "set": {"r": 3, "p": -3}},
      {"at": 4, "name": "R = 4", "set": {"r": 4, "p": -4}},
      {"at": 5, "name": "R = 5", "set": {"r": 5, "p": -5}},
      {"at": 6, "name": "R = 6", "set": {"r": 6, "p": -6}},
      {"at": 7, "name": "R = 7", "set": {"r": 7, "p": -7}},
      {"at": 8, "name": "R = 8", "set": {"r": 8, "p": -8}},
      {"at": 9, "name": "R = 9", "set": {"r": 9, "p": -9}}
    ]
  },
  "series": {
    "label": "T (S = −T)",
    "values": [
      {"at": 2, "name": "T = 2", "set": {"t": 2, "s": -2}},
      {"at": 3, "name": "T = 3", "set": {"t": 3, "s": -3}},
      {"at": 4, "name": "T = 4", "set": {"t": 4, "s": -4}},
      {"at": 5, "name": "T = 5", "set": {"t": 5, "s": -5}},
      {"at": 6, "name": "T = 6", "set": {"t": 6, "s": -6}},
      {"at": 7, "name": "T = 7", "set": {"t": 7, "s": -7}},
      {"at": 8, "name": "T = 8", "set": {"t": 8, "s": -8}},
      {"at": 9, "name": "T = 9", "set": {"t": 9, "s": -9}},
      {"at": 10, "name": "T = 10", "set": {"t": 10, "s": -10}}
    ]
  },
  "seeds": {"from": 1, "count": 30},
  "ticks": 500,
  "metric": {"kind": "final", "series": "cooperators"}
}
```

`sweeps/dpd-mutation.json`:

```json
{
  "name": "Demographic PD: cooperation against mutation (Epstein's Run 5)",
  "description": "Epstein's Run 5: Run 2 with a 50% chance that an offspring takes the other strategy, and cooperation persists (at 25%, \"cooperators averaging around 350 and defectors around 400\"). Cooperators at cycle 500 against the mutation rate. Measured (release, seeds 1–30, recorded 2026-09-26): 0: 695; 5%: 551; 10%: 512; 20%: 446; 30%: 386; 40%: 342; 50%: 268 (the lowest run 0). Cooperation falls steadily with mutation but persists; at 25% the means over cycles 1,001–2,000 are 420 cooperators and 393 defectors.",
  "base": {"preset": "dpd-run-2"},
  "x": {"label": "Mutation rate", "path": "mutation", "values": [0, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5]},
  "seeds": {"from": 1, "count": 30},
  "ticks": 500,
  "metric": {"kind": "final", "series": "cooperators"}
}
```

`sweeps/dpd-metabolism.json`:

```json
{
  "name": "Demographic PD: shifted payoffs against metabolism, per cycle or per game (Epstein)",
  "description": "Epstein: the shifted payoffs (12, 11, 1, 0) with a metabolism of 6 are \"equivalent mathematically\" to the original (6, 5, −5, −6), metabolism being a fixed decrement \"per cycle\" (the text also says \"after every interaction\"). Cooperators at cycle 500 against the metabolism, charged per cycle or per game. Measured (release, seeds 1–30, recorded 2026-09-26), per cycle: 0: 418; 1: 414; 2: 435; 3: 495; 4: 612; 5: 661; 6: 644; per game: 0: 418; 1: 339; 2: 380; 3: 514; 4: 677; 5: 714; 6: 695. Per game at 6 is Run 2 exactly (the same runs, to the fingerprint); per cycle at 6 is another model with a similar outcome. Metabolism does raise cooperation, as Epstein's \"necessity is the mother of cooperation\" conjecture says — but from coexistence (418 at no metabolism), not from pure defection.",
  "base": {"preset": "dpd-shifted"},
  "x": {"label": "Metabolism", "path": "metabolism", "values": [0, 1, 2, 3, 4, 5, 6]},
  "series": {
    "label": "Metabolism charged",
    "values": [
      {"at": 0, "name": "Per cycle", "set": {"metabolism_per": "cycle"}},
      {"at": 1, "name": "Per game", "set": {"metabolism_per": "interaction"}}
    ]
  },
  "seeds": {"from": 1, "count": 30},
  "ticks": 500,
  "metric": {"kind": "final", "series": "cooperators"}
}
```

`sweeps/dpd-max-age.json`:

```json
{
  "name": "Demographic PD: cooperation against the maximum age (Epstein's Runs 1–2)",
  "description": "Epstein's Runs 1 and 2 differ only in the maximum age (none, then 100): Tables 1 and 2 give 779 and 784 cooperators. Cooperators at cycle 500 against the maximum age (initial and newborn ages random up to it). Measured (release, seeds 1–30, recorded 2026-09-26): 10: 666; 20: 705; 50: 711; 100: 695; 200: 710; 500: 721; 1,000: 735 (no maximum: 729). The maximum age barely matters, and every value stays 45–120 below Epstein's.",
  "base": {"preset": "dpd-run-1"},
  "x": {"label": "Maximum age", "path": "max_age", "values": [10, 20, 50, 100, 200, 500, 1000]},
  "seeds": {"from": 1, "count": 30},
  "ticks": 500,
  "metric": {"kind": "final", "series": "cooperators"}
}
```

Register them and list the ids (core, CLI, WASM), and add the WASM fingerprint test:

```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -959,7 +962,7 @@
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 39] = [
+const BUILTINS: [Builtin; 43] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1116,6 +1119,22 @@
         id: "jansson-markers",
         json: include_str!("../../../sweeps/jansson-markers.json"),
     },
+    Builtin {
+        id: "dpd-payoffs",
+        json: include_str!("../../../sweeps/dpd-payoffs.json"),
+    },
+    Builtin {
+        id: "dpd-mutation",
+        json: include_str!("../../../sweeps/dpd-mutation.json"),
+    },
+    Builtin {
+        id: "dpd-metabolism",
+        json: include_str!("../../../sweeps/dpd-metabolism.json"),
+    },
+    Builtin {
+        id: "dpd-max-age",
+        json: include_str!("../../../sweeps/dpd-max-age.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -1977,7 +1996,11 @@
                 "ha-immigration",
                 "ha-lattice",
                 "jansson-tag-mutation",
-                "jansson-markers"
+                "jansson-markers",
+                "dpd-payoffs",
+                "dpd-mutation",
+                "dpd-metabolism",
+                "dpd-max-age"
             ]
         );
         for b in builtins() {
```
```diff
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -87,6 +87,10 @@
         "ha-lattice",
         "jansson-tag-mutation",
         "jansson-markers",
+        "dpd-payoffs",
+        "dpd-mutation",
+        "dpd-metabolism",
+        "dpd-max-age",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
```
```diff
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -289,7 +289,11 @@
             "ha-immigration",
             "ha-lattice",
             "jansson-tag-mutation",
-            "jansson-markers"
+            "jansson-markers",
+            "dpd-payoffs",
+            "dpd-mutation",
+            "dpd-metabolism",
+            "dpd-max-age"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -744,6 +748,22 @@
     ] {
         let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
         assert_eq!(sim.model_kind(), "ethno");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
+#[wasm_bindgen_test]
+fn dpd_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: wealth sums and
+    // the swap shuffle's draws are the same bits here as natively.
+    for (id, fp) in [
+        ("dpd-run-1", "0x3d64b053fbfee4f6"),
+        ("dpd-run-5", "0x2f5ae2bdc6bd257a"),
+        ("dpd-rr-best", "0xe8fdc4ce027dd236"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "dpd");
         sim.step(200);
         assert_eq!(sim.fingerprint(), fp, "{id}");
     }
```

- [ ] **Step 3: Run the sweeps and check the descriptions**

```bash
for s in dpd-payoffs dpd-mutation dpd-metabolism dpd-max-age; do
  cargo run --release -p sugarscape-cli -- sweep --builtin $s --quiet --summary-csv /dev/stdout --out /dev/null
done
```
Expected (means, 30 seeds): `dpd-mutation` 694.8, 551.0, 512.1, 446.3, 386.5, 342.1, 267.6; `dpd-metabolism` per cycle 418.1, 414.1, 435.4, 494.6, 611.5, 661.1, 644.2 and per game 418.1, 339.1, 380.3, 514.0, 676.7, 713.9, 694.8; `dpd-max-age` 665.7, 704.5, 710.9, 694.8, 709.8, 720.6, 734.9; `dpd-payoffs` as its description (e.g. T = 6: 0.0, 11.9, 367.5, 633.9, 729.2, then 780–834 for R ≥ 6). About 7, 1, 2 and 1 s.

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --lib sweep && cargo test -p sugarscape-cli && wasm-pack test --node crates/sugarscape-wasm`
Expected: PASS — `builtin_sweeps_parse_and_validate` with the four new ids, the CLI's listing test, and 38 WASM tests including `dpd_sims_match_the_native_golden_entries`.

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/tests/dpd.rs sweeps/dpd-payoffs.json sweeps/dpd-mutation.json sweeps/dpd-metabolism.json sweeps/dpd-max-age.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure the demographic Prisoner's Dilemma against Epstein and Radax and Rengs" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 3: The demographic PD on the page — types, the Rules panel, charts and Inspect

**Files:**
- Create: `web/src/dpd.ts`, `web/src/dpd.test.ts`
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`, `web/src/engine.ts`
- Test: `web/src/models.test.ts`, `web/src/schema-form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/engine.test.ts`

**Interfaces:**
- Consumes: Tasks 1–2's `ModelKind::Dpd` (`"dpd"`), its schema (groups Game, Population, Evolution, Timing, Interaction, Run; `play` `shown_if("pairing", "space")`), `DpdInspection` JSON (Decision 13), `SERIES`, the colour modes `strategy`, `wealth`, `age`, `surrounded`.
- Produces: `DpdConfig`, `DpdStats`, `DpdNeighborView`, `DpdAgentView`, `DpdInspection` (types.ts); `ModelKind` gains `'dpd'`, `ColorMode` gains `'surrounded'`; `isDpdView(v, model)`, `MODELS`/`MODEL_LABELS`/`COLOR_MODES`/`MODEL_OVERLAYS` entries, `ticksLeft` for dpd (models.ts); `MODEL_CHARTS.dpd`, `timeAxisLabel('dpd') === 'Cycle'` (series-data.ts); `dpdRows(a: DpdAgentView): [string, string][]` (dpd.ts); `finishedNotice` for dpd.

- [ ] **Step 1: Write the failing tests**

`web/src/models.test.ts`: import `isDpdView` (after `isCultureView`) and append:

```ts
describe('the demographic PD', () => {
  const dpd = (end: number) => ({ model: 'dpd', end }) as unknown as ModelConfig;

  it('is read by its tag, and its inspections by the model (an empty site is shaped like Schelling’s and ethnocentrism’s)', () => {
    expect(modelOf(dpd(0))).toBe('dpd');
    expect(isSugar(dpd(0))).toBe(false);
    const empty = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    const agent = { site: { x: 1, y: 2 }, agent: { id: 3, strategy: 'C', surrounded: false, neighbors: [] } } as unknown as AnyInspection;
    const ethno = { site: { x: 1, y: 2 }, agent: { id: 3, kin_marker: 3, neighbors: [] } } as unknown as AnyInspection;
    expect([empty, agent].map((v) => isDpdView(v, 'dpd'))).toEqual([true, true]);
    expect([empty, agent].map((v) => isDpdView(v, 'ethno'))).toEqual([false, false]);
    expect([empty, agent].map((v) => isDpdView(v, 'schelling'))).toEqual([false, false]);
    expect(isDpdView(ethno, 'dpd')).toBe(false);
    expect([empty, agent].map((v) => isEthnoView(v, 'dpd'))).toEqual([false, false]);
    expect(isTagsView(agent) || isRingView(agent) || isSugarView(agent) || isValleyView(agent) || isCivilView(agent) || isSpatialView(agent)).toBe(false);
    expect(isClassesView(agent) || isCultureView(agent)).toBe(false);
  });

  it('offers strategy, wealth, age and surrounded colors and no overlays, and is grouped last', () => {
    expect(COLOR_MODES.dpd).toEqual([
      ['strategy', 'Strategy'],
      ['wealth', 'Wealth'],
      ['age', 'Age'],
      ['surrounded', 'Surrounded'],
    ]);
    expect(MODEL_OVERLAYS.dpd).toEqual([]);
    const p = (id: string, config: object) => ({ id, name: id, source: '', description: '', config }) as unknown as Preset;
    expect(presetGroups([p('dpd', { model: 'dpd' }), p('ha', { model: 'ethno' })]).map((g) => g.label)).toEqual(['Ethnocentrism', 'Demographic PD']);
  });

  it('counts down to its last cycle, or never with none (the default)', () => {
    expect(ticksLeft(dpd(500), 490)).toBe(10);
    expect(ticksLeft(dpd(500), 505)).toBe(0);
    expect(ticksLeft(dpd(0), 5)).toBe(Infinity);
    expect(finishesUnpredictably(dpd(500))).toBe(false);
    expect(calendarYear(dpd(500), 5)).toBeNull();
  });
});
```

`web/src/schema-form.test.ts`: the type import gains `DpdConfig`:

```ts
import type { AnasaziConfig, DpdConfig, EthnoConfig, ModelConfig, Param, RingConfig, SchellingConfig } from './types';
```

and inside `describe('the schema form', …)`, after the nullable-slider test:

```ts
  it('reads and writes negative payoffs, and shows the play rule only in space (the demographic PD)', () => {
    const s = param({ path: 's', kind: 'number', min: -20, max: 20, step: 1 });
    const c = { model: 'dpd', t: 6, r: 5, p: -5, s: -6, pairing: 'space' } as unknown as DpdConfig;
    expect(paramInput(s, c)).toBe('-6');
    expect(paramSlider(s, c)).toBe('-6');
    paramEdit(s, '-3')(c);
    expect(c.s).toBe(-3);
    paramEdit(s, '-2.5')(c);
    expect(c.s).toBe(-2.5);
    const play = { path: 'play', label: 'Play', kind: 'choice', apply: 'live', group: 'Interaction', show_if: { path: 'pairing', equals: 'space' } } as Param;
    expect(paramShown(play, c)).toBe(true);
    paramEdit(param({ path: 'pairing', kind: 'choice' }), 'soup')(c);
    expect(paramShown(play, c)).toBe(false);
  });
```

`web/src/ui/series-data.test.ts`, appended:

```ts
describe('the demographic PD’s charts', () => {
  it('charts population, cooperator share, surrounded cooperators, wealth and births and deaths against the cycle', () => {
    expect(MODEL_CHARTS.dpd.map((c) => c.title)).toEqual(['Population', 'Cooperator share', 'Surrounded cooperators', 'Wealth', 'Births and deaths']);
    expect(MODEL_CHARTS.dpd.map((c) => c.lines.map((l) => [l.key, l.color]))).toEqual([
      [
        ['cooperators', '--blue'],
        ['defectors', '--red'],
      ],
      [['cooperator_share', '--blue']],
      [['surrounded', '--c1']],
      [
        ['wealth_c', '--blue'],
        ['wealth_d', '--red'],
      ],
      [
        ['births', '--c2'],
        ['deaths', '--muted'],
      ],
    ]);
    expect(MODEL_CHARTS.dpd[1].range).toEqual([0, 1]);
    expect(MODEL_CHARTS.dpd.every((c) => !c.shown)).toBe(true);
    expect(timeAxisLabel('dpd')).toBe('Cycle');
  });
});
```

`web/src/engine.test.ts`, after the ethnocentrism model's `finishedNotice` expectation:

```ts
    expect(finishedNotice({ model: 'dpd' } as unknown as ModelConfig, 500)).toBe('This run has reached its last cycle (500) — Reset to run it again');
```

Create `web/src/dpd.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { dpdRows } from './dpd';
import type { DpdAgentView } from './types';

const agent = (c: Partial<DpdAgentView>): DpdAgentView => ({
  id: 57,
  strategy: 'C',
  wealth: 14,
  age: 37,
  max_age: 100,
  surrounded: false,
  income: 4,
  games: 2,
  neighbors: [
    { x: 5, y: 3, id: 12, strategy: 'C', payoff: 5, their_payoff: 5 },
    { x: 4, y: 4, id: 80, strategy: 'D', payoff: -6, their_payoff: 6 },
  ],
  ...c,
});

describe('dpdRows', () => {
  it('shows the strategy, wealth, age against the maximum, this cycle’s payoffs, and each neighbor with what a game pays', () => {
    expect(dpdRows(agent({}))).toEqual([
      ['Agent', '#57 · cooperator'],
      ['Wealth', '14'],
      ['Age', '37 cycles (maximum 100)'],
      ['Surrounded', 'no'],
      ['This cycle', '4 from 2 games'],
      ['(5, 3)', '#12 · cooperator · a game pays this agent 5 and the neighbor 5'],
      ['(4, 4)', '#80 · defector · a game pays this agent -6 and the neighbor 6'],
    ]);
  });

  it('says when a cooperator is surrounded, with no maximum age, fractional wealth and a lone agent', () => {
    const a = agent({ surrounded: true, max_age: 0, age: 1, wealth: 7.25, income: -0.5, games: 1, neighbors: [] });
    expect(dpdRows(a)).toEqual([
      ['Agent', '#57 · cooperator'],
      ['Wealth', '7.25'],
      ['Age', '1 cycle (no maximum)'],
      ['Surrounded', 'yes: all eight neighbors (Moore) cooperate'],
      ['This cycle', '-0.50 from 1 game'],
      ['Neighbors', 'none'],
    ]);
    const d = agent({ strategy: 'D', neighbors: [{ x: 0, y: 29, id: 3, strategy: 'C', payoff: 6, their_payoff: -6 }] });
    expect(dpdRows(d)[0]).toEqual(['Agent', '#57 · defector']);
    expect(dpdRows(d).at(-1)).toEqual(['(0, 29)', '#3 · cooperator · a game pays this agent 6 and the neighbor -6']);
  });
});
```

Run: `(cd web && npm run build)` — Expected: FAIL at `tsc --noEmit` (`'"./dpd"'` cannot be found, `isDpdView` is not exported from `./models`, `DpdConfig`/`DpdAgentView` are not exported from `./types`, `'dpd'` is not assignable to `ModelKind`, `MODEL_CHARTS.dpd` does not exist). (Stated, not observed: only the final state was run.)

- [ ] **Step 2: Types and models**

`web/src/types.ts`: `ModelKind` and its comment:

```ts
/** The models the playground runs (milestones 9–17). */
export type ModelKind = 'sugarscape' | 'schelling' | 'ring' | 'anasazi' | 'civil' | 'spatial' | 'tags' | 'culture' | 'classes' | 'ethno' | 'dpd';
```

after `EthnoConfig`:

```ts
/**
 * Epstein's demographic Prisoner's Dilemma (milestone 17): agents with fixed strategies move, play
 * their neighbors, clone and die by their accumulated payoffs, with the working paper's rule, Radax
 * and Rengs' timing choices, soup and metabolism as switches. A tick is a cycle.
 */
export interface DpdConfig {
  model: 'dpd';
  width: number;
  agents: number;
  initial_cooperators: number;
  initial_wealth: number;
  /** Payoffs: T to a defector against a cooperator, R to two cooperators, P to two defectors, S to a cooperator against a defector (any finite values). */
  t: number;
  r: number;
  p: number;
  s: number;
  fission_wealth: number;
  endowment: number;
  /** 0: no maximum. */
  max_age: number;
  metabolism: number;
  metabolism_per: 'cycle' | 'interaction';
  mutation: number;
  vision: number;
  play: 'each_neighbor' | 'random_neighbor';
  pairing: 'space' | 'soup';
  death_timing: 'immediate' | 'own_turn';
  removal: 'immediate' | 'end_of_cycle';
  endowment_from: 'parent' | 'granted';
  newborn_age: 'random' | 'zero';
  updating: 'asynchronous' | 'synchronous';
  shuffle: 'swaps' | 'full';
  newborns_act: 'next_cycle' | 'this_cycle';
  /** The last cycle (0: never). */
  end: number;
  schedule: ScheduledChange[];
}
```

`ModelConfig` ends `| EthnoConfig | DpdConfig;`. Before the `/** The latest statistics of a world of any model. */` comment above `CultureStats`:

```ts
/** A cycle's statistics, of the agents alive at its end. The share and mean wealths are null with nobody to count. */
export interface DpdStats {
  tick: number;
  cooperators: number;
  defectors: number;
  population: number;
  cooperator_share: number | null;
  /** Cooperators all eight of whose Moore neighbors are cooperators. */
  surrounded: number;
  wealth_c: number | null;
  wealth_d: number | null;
  births: number;
  deaths: number;
}
```

and `ModelStats` ends `| EthnoStats | DpdStats;`. Before the `/** What a world of any model says about a site. */` comment:

```ts
/** An occupied neighbor of a demographic PD agent (up, left, right, down), with one game's payoff to each under the current payoffs. */
export interface DpdNeighborView { x: number; y: number; id: number; strategy: 'C' | 'D'; payoff: number; their_payoff: number }
/** A demographic PD agent: its strategy, wealth and age, whether it is surrounded, this cycle's income and games, and its neighbors. */
export interface DpdAgentView {
  id: number;
  strategy: 'C' | 'D';
  wealth: number;
  age: number;
  /** The maximum age (0: none). */
  max_age: number;
  /** A cooperator all eight of whose Moore neighbors are cooperators. */
  surrounded: boolean;
  /** Payoffs received and games played this cycle. */
  income: number;
  games: number;
  neighbors: DpdNeighborView[];
}
/** A demographic PD site. Empty, it looks exactly like an empty Schelling site: `isDpdView` asks the model. */
export interface DpdInspection { site: { x: number; y: number }; agent: DpdAgentView | null }
```

and `AnyInspection` ends `| EthnoInspection | DpdInspection;`. `ColorMode`'s comment ends "or (ethnocentrism) `strategy`, `tag`, `lineage`, `ptr`, or (the demographic PD) `strategy`, `wealth`, `age`, `surrounded`." and its union ends `| 'payoff' | 'surrounded';` (`strategy`, `wealth` and `age` are already there).

`web/src/models.ts`: the header comment says "milestones 9–17"; import types `DpdConfig`, `DpdInspection` (after `Config`); `MODELS` ends `'classes', 'ethno', 'dpd'`; `MODEL_LABELS.dpd = 'Demographic PD'`; `modelOf` accepts `|| tag === 'dpd'`; after `isEthnoView`:

```ts
/**
 * A demographic PD site's inspection (its agent says whether it is surrounded). An empty one is
 * exactly an empty Schelling or ethnocentrism site, so the world's model decides as well as the shape.
 */
export function isDpdView(v: AnyInspection, model: ModelKind): v is DpdInspection {
  return model === 'dpd' && (v.agent === null || 'surrounded' in v.agent);
}
```

`ticksLeft`'s comment adds "the demographic PD's last cycle" and, after the ethno line:

```ts
  if (modelOf(c) === 'dpd' && (c as DpdConfig).end > 0) return Math.max(0, (c as DpdConfig).end - tick);
```

`COLOR_MODES.dpd` (after `ethno`) and `MODEL_OVERLAYS.dpd = []`:

```ts
  // Epstein's colors first (cooperators blue, defectors red); the core's mode names.
  dpd: [
    ['strategy', 'Strategy'],
    ['wealth', 'Wealth'],
    ['age', 'Age'],
    ['surrounded', 'Surrounded'],
  ],
```

`web/src/engine.ts`, `finishedNotice`, after the ethno line:

```ts
  if (modelOf(config) === 'dpd') return `This run has reached its last cycle (${tick}) — Reset to run it again`;
```

- [ ] **Step 3: Charts and Inspect** (Decisions 22, 25, 27; the Rules panel needs no code: Decision 26)

`web/src/ui/series-data.ts`: `MODEL_CHARTS`'s doc comment ends "…the ethnocentrism model's strategies (in the frame's colors), cooperation, population and kin; the demographic PD's cooperators and defectors (in the frame's colors), cooperator share, surrounded cooperators, mean wealths, and births and deaths."; after `ethno`:

```ts
  dpd: [
    {
      title: 'Population',
      lines: [
        { key: 'cooperators', label: 'Cooperators', color: '--blue' },
        { key: 'defectors', label: 'Defectors', color: '--red' },
      ],
    },
    { title: 'Cooperator share', lines: [{ key: 'cooperator_share', label: 'Cooperators ÷ agents', color: '--blue' }], range: [0, 1] },
    { title: 'Surrounded cooperators', lines: [{ key: 'surrounded', label: 'All eight neighbors cooperate', color: '--c1' }] },
    {
      title: 'Wealth',
      lines: [
        { key: 'wealth_c', label: 'Mean, cooperators', color: '--blue' },
        { key: 'wealth_d', label: 'Mean, defectors', color: '--red' },
      ],
    },
    {
      title: 'Births and deaths',
      lines: [
        { key: 'births', label: 'Births', color: '--c2' },
        { key: 'deaths', label: 'Deaths', color: '--muted' },
      ],
    },
  ],
```

and `timeAxisLabel` becomes:

```ts
/**
 * A model's time charts count calendar years (the anasazi's), generations (tags), periods
 * (ethnocentrism, HA06's word), cycles (the demographic PD, Epstein's word) or ticks.
 */
export function timeAxisLabel(model: ModelKind): string {
  return model === 'anasazi'
    ? 'Year'
    : model === 'tags'
      ? 'Generation'
      : model === 'culture'
        ? 'Events per site'
        : model === 'classes'
          ? 'Periods'
          : model === 'ethno'
            ? 'Period'
            : model === 'dpd'
              ? 'Cycle'
              : 'Tick';
}
```

Create `web/src/dpd.ts`:

```ts
// The demographic Prisoner's Dilemma's pure page helpers (milestone 17): Inspect's rows.
import type { DpdAgentView } from './types';

/** A strategy's name. */
const STRATEGY: Record<'C' | 'D', string> = { C: 'cooperator', D: 'defector' };

/** Whole numbers as they are, others to two decimals. */
const num = (n: number): string => (Number.isInteger(n) ? String(n) : n.toFixed(2));

/** "1 game", "3 games" (and cycles alike). */
const count = (n: number, one: string): string => `${n} ${n === 1 ? one : `${one}s`}`;

/**
 * An agent's Inspect rows: its strategy, wealth and age (against its maximum), whether it is a
 * surrounded cooperator, this cycle's payoffs and games, then each occupied neighbor (up, left,
 * right, down) with what one game between them pays each under the current payoffs.
 */
export function dpdRows(a: DpdAgentView): [string, string][] {
  const rows: [string, string][] = [
    ['Agent', `#${a.id} · ${STRATEGY[a.strategy]}`],
    ['Wealth', num(a.wealth)],
    ['Age', `${count(a.age, 'cycle')} (${a.max_age === 0 ? 'no maximum' : `maximum ${a.max_age}`})`],
    ['Surrounded', a.surrounded ? 'yes: all eight neighbors (Moore) cooperate' : 'no'],
    ['This cycle', `${num(a.income)} from ${count(a.games, 'game')}`],
  ];
  if (a.neighbors.length === 0) return [...rows, ['Neighbors', 'none']];
  for (const n of a.neighbors) {
    rows.push([`(${n.x}, ${n.y})`, `#${n.id} · ${STRATEGY[n.strategy]} · a game pays this agent ${num(n.payoff)} and the neighbor ${num(n.their_payoff)}`]);
  }
  return rows;
}
```

`web/src/ui/inspect-panel.ts`: import `dpdRows` from `../dpd` (first), `isDpdView` from `../models`, type `DpdInspection`; after `ethnoSiteRows`:

```ts
  /**
   * A demographic PD site and its agent: strategy, wealth, age, whether surrounded, this cycle's
   * payoffs, and its neighbors with what a game between them pays. An agent that died leaves the
   * site's rows alone.
   */
  private dpdSiteRows(view: DpdInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
    if (gone) return rows;
    if (!view.agent) return [...rows, row('Agent', 'none (an empty site)')];
    return [...rows, ...dpdRows(view.agent).map(([k, v]) => row(k, v))];
  }
```

and in `render` the note and the chain begin:

```ts
      const left = isValleyView(view)
        ? `Household #${shown.agentId} is gone: it died or left the valley.`
        : isCivilView(view)
          ? `Agent #${shown.agentId} is gone: killed, or dead of old age.`
          : isEthnoView(view, this.engine.model) || isDpdView(view, this.engine.model)
            ? `Agent #${shown.agentId} has died.`
            : `Agent #${shown.agentId} has left.`;
      const note = gone ? [h('p', { class: 'error' }, left)] : [];
      // First: an empty ethnocentrism or demographic PD site is shaped like an empty Schelling site.
      const rows = isEthnoView(view, this.engine.model)
        ? this.ethnoSiteRows(view, gone)
        : isDpdView(view, this.engine.model)
          ? this.dpdSiteRows(view, gone)
          : isClassesView(view)
            ? this.classesRows(view)
```

(the rest of the chain unchanged, one level deeper).

- [ ] **Step 4: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS. (determinism.test's "model charts draw only series their model records" now covers `MODEL_CHARTS.dpd` against `config_series_names`.)

```bash
git add web/src/types.ts web/src/models.ts web/src/models.test.ts web/src/schema-form.test.ts web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/dpd.ts web/src/dpd.test.ts web/src/ui/inspect-panel.ts web/src/engine.ts web/src/engine.test.ts
git commit -m "Show the demographic PD on the page: its colors, charts and Inspect" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): every preset of the **Demographic PD** group renders in Strategy, Wealth, Age and Surrounded; the Rules panel's six groups, the payoffs taking −3 and 16, Play hidden under soup; Inspect on an agent lists its neighbours with what a game pays each, on an empty site says "none (an empty site)", and says "Agent #N has died." when a followed agent dies; the five charts against "Cycle".

---

### Task 4: Compare, Experiments and the engine-level checks

**Files:**
- Modify: `web/src/compare-presets.ts`, `web/src/experiments/form.ts`
- Test: `web/src/compare-presets.test.ts`, `web/src/experiments/form.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: Task 3's types and `isDpdView`; Task 1's presets and golden fingerprints (Decision M); `model_schemas_json`, `sweep_points` (WASM).
- Produces: the four Compare entries (`dpd-wp-vs-published`, `dpd-negative-vs-metabolism`, `dpd-space-vs-soup`, `dpd-published-vs-closest`); `defaultForm('dpd')`; thirteen `GOLDEN_MODELS` entries.

- [ ] **Step 1: Write the failing tests**

`web/src/compare-presets.test.ts`, inside the top `describe`, after the ethnocentrism test:

```ts
  it('pairs the demographic PD runs the sources and readings disagree on', () => {
    const ids = COMPARE_PRESETS.filter((c) => c.id.startsWith('dpd-')).map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toEqual([
      ['dpd-wp-vs-published', 'dpd-run-1', 'dpd-working-paper', 'Working paper vs published rule — Demographic PD (Compare)'],
      ['dpd-negative-vs-metabolism', 'dpd-run-2', 'dpd-metabolism', 'Negative payoffs vs shifted with metabolism — Demographic PD (Compare)'],
      ['dpd-space-vs-soup', 'dpd-run-1', 'dpd-soup', 'Space vs soup — Demographic PD (Compare)'],
      ['dpd-published-vs-closest', 'dpd-run-1', 'dpd-closest', 'Published rule vs closest reading — Demographic PD (Compare)'],
    ]);
  });
```

`web/src/experiments/form.test.ts`, after the ethno expectation:

```ts
    expect(defaultForm('dpd')).toMatchObject({
      x: { path: 'r', values: '1:5:1' },
      series: null,
      seeds: 3,
      ticks: 500,
      metric: { kind: 'final', series: 'cooperators' },
    });
```

`web/src/determinism.test.ts`: `isDpdView` beside `isEthnoView`; types `DpdConfig`, `DpdInspection`, `DpdStats` (after `CultureStats`). Append to `GOLDEN_MODELS` after `hks-no-ethnocentrics` (Decision M; the WASM prints 16 hex digits, so `dpd-soup` keeps its leading zero):

```ts
    ['dpd-run-1', '0x3d64b053fbfee4f6'],
    ['dpd-run-2', '0xe0198124ac5f4789'],
    ['dpd-run-3', '0x1c819cc85b7bb351'],
    ['dpd-run-4', '0xe6b5d66dee22ce07'],
    ['dpd-run-5', '0x2f5ae2bdc6bd257a'],
    ['dpd-working-paper', '0xaba834120f15810b'],
    ['dpd-closest', '0xd1c7475297f9864c'],
    ['dpd-soup', '0x0db64ad16a49146b'],
    ['dpd-shifted', '0x39b59d546b2bb2e8'],
    ['dpd-metabolism', '0x7b580a83419a3ea4'],
    ['dpd-footnote-27', '0xd1adeadce6881068'],
    ['dpd-rr-best', '0xe8fdc4ce027dd236'],
    ['dpd-coordination', '0x47f8c68504a9157a'],
```

and before `describe('civil violence through the engine', …)`:

```ts
describe('the demographic PD through the engine', () => {
  const preset = (id: string) => presets.find((p) => p.id === id)!;
  const dpdPresets = presets.filter((p) => modelOf(p.config) === 'dpd');
  const create = (id: string, edit: (c: DpdConfig) => void = () => {}) => {
    const config = structuredClone(preset(id).config) as DpdConfig;
    edit(config);
    return Engine.create({ config, seed: 1 }, { presets, transport: inline() });
  };
  const schema = (JSON.parse(model_schemas_json()) as Record<string, Param[]>).dpd;
  const field = (path: string) => schema.find((p) => p.path === path)!;

  it('builds a Rules panel in the spec’s groups, with negative payoffs on the sliders and the play rule only in space', () => {
    expect([...new Set(schema.map((p) => p.group))]).toEqual(['Game', 'Population', 'Evolution', 'Timing', 'Interaction', 'Run']);
    for (const path of ['t', 'r', 'p', 's']) expect([field(path).min, field(path).max]).toEqual([-20, 20]);
    expect(field('play').show_if).toEqual({ path: 'pairing', equals: 'space' });
    expect([paramShown(field('play'), preset('dpd-run-1').config), paramShown(field('play'), preset('dpd-soup').config)]).toEqual([true, false]);
    // Every preset's numbers sit on their sliders (the coordination game's −3, footnote 27's 16, max age 1,000).
    expect(dpdPresets).toHaveLength(13);
    for (const p of dpdPresets) {
      for (const f of schema.filter((f) => f.kind === 'number' || f.kind === 'integer')) {
        const v = (p.config as unknown as Record<string, number>)[f.path];
        expect(v, `${p.id}: ${f.path}`).toBeGreaterThanOrEqual(f.min!);
        expect(v, `${p.id}: ${f.path}`).toBeLessThanOrEqual(f.max!);
      }
    }
  });

  it('takes live edits of the payoffs and the pairing, and replays them through keyframes and a share link', async () => {
    const e = await create('dpd-rr-best');
    await e.advance(40);
    expect(await e.applyModelConfig((c) => void ((c as DpdConfig).s = -3))).toBeNull();
    await e.advance(40);
    expect(await e.applyModelConfig((c) => void ((c as DpdConfig).pairing = 'soup'))).toBeNull();
    await e.advance(20);
    const want = await e.fingerprint();
    expect(e.tick).toBe(100);
    await e.seek(30);
    await e.seek(100);
    expect(await e.fingerprint()).toBe(want);
    const { session, tick } = await e.session();
    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await opened.advance(tick);
    expect((opened.config as DpdConfig).pairing).toBe('soup');
    expect(await opened.fingerprint()).toBe(want);
    const errors = await e.applyModelConfig((c) => void ((c as DpdConfig).mutation = 2));
    expect(errors?.map((f) => f.field)).toEqual(['mutation']);
  });

  it('stops at its last cycle, once, and inspects agents, surrounded cooperators and empty sites', async () => {
    const e = await create('dpd-run-1', (c) => (c.end = 120));
    e.setDisplay({ colorMode: 'surrounded' });
    // At the start 100 agents hold 900 sites: an empty one is an empty demographic PD site.
    const empties: number[] = [];
    for (let x = 0; x < 30; x++) {
      await e.select(x, 0);
      const v = e.inspection!.view;
      expect(isDpdView(v, e.model)).toBe(true);
      if ((v as DpdInspection).agent === null) empties.push(x);
    }
    expect(empties.length).toBeGreaterThan(20);
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(200);
    expect([e.tick, e.finished, ends]).toEqual([120, true, 1]);
    const stats = e.latest as DpdStats;
    expect(stats.cooperators + stats.defectors).toBe(stats.population);
    const seen = { agent: 0, surrounded: 0, empty: 0 };
    for (let y = 0; y < 30; y++) {
      for (let x = 0; x < 30; x += 3) {
        await e.select(x, y);
        const v = e.inspection!.view;
        expect(isDpdView(v, e.model)).toBe(true);
        const a = (v as DpdInspection).agent;
        if (!a) {
          seen.empty++;
          continue;
        }
        seen.agent++;
        expect(e.inspection!.agentId).toBe(a.id);
        expect(a.neighbors.length).toBeLessThanOrEqual(4);
        if (a.surrounded) {
          seen.surrounded++;
          expect(a.strategy).toBe('C');
          expect(a.neighbors.map((n) => n.strategy)).toEqual(['C', 'C', 'C', 'C']);
        }
      }
    }
    expect(seen.agent).toBeGreaterThan(0);
    expect(seen.surrounded).toBeGreaterThan(0);
    // By then the lattice is all but full (Run 1 holds about 900 agents).
    expect(seen.agent).toBeGreaterThan(290);
  });

  it('opens its four Compare entries and sweeps its default form', () => {
    for (const id of ['dpd-wp-vs-published', 'dpd-negative-vs-metabolism', 'dpd-space-vs-soup', 'dpd-published-vs-closest']) {
      const entry = COMPARE_PRESETS.find((c) => c.id === id)!;
      expect(comparePresetStates(presets, entry, 1), id).not.toBeNull();
    }
    const { sweep, errors } = formToSweep(defaultForm('dpd'), { preset: 'dpd-run-1' });
    expect(errors).toEqual([]);
    // Five rewards × the form's three seeds.
    expect(JSON.parse(sweep_points(JSON.stringify(sweep)))).toHaveLength(15);
  });
});
```

Run: `(cd web && npm run build && npm test)` — Expected: FAIL (the Compare entries, `defaultForm('dpd')`, and the engine test's Compare/sweep case; the golden entries, Rules panel, live-edit and Inspect tests already pass on Task 3's code).

- [ ] **Step 2: Implement**

`web/src/compare-presets.ts`, at the end of `COMPARE_PRESETS`:

```ts
  {
    id: 'dpd-wp-vs-published',
    label: 'Working paper vs published rule — Demographic PD (Compare)',
    a: 'dpd-run-1',
    b: 'dpd-working-paper',
  },
  {
    id: 'dpd-negative-vs-metabolism',
    label: 'Negative payoffs vs shifted with metabolism — Demographic PD (Compare)',
    a: 'dpd-run-2',
    b: 'dpd-metabolism',
  },
  {
    id: 'dpd-space-vs-soup',
    label: 'Space vs soup — Demographic PD (Compare)',
    a: 'dpd-run-1',
    b: 'dpd-soup',
  },
  {
    id: 'dpd-published-vs-closest',
    label: 'Published rule vs closest reading — Demographic PD (Compare)',
    a: 'dpd-run-1',
    b: 'dpd-closest',
  },
];
```

`web/src/experiments/form.ts`, `defaultForm`, after the ethno branch:

```ts
  if (model === 'dpd') {
    // Table 9.3's axis at T = 6: cooperators after 500 cycles against the reward R (R = 1 dies out).
    return { ...form, x: { path: 'r', values: '1:5:1' }, ticks: 500, metric: { ...form.metric, kind: 'final', series: 'cooperators' } };
  }
```

- [ ] **Step 3: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS (45 files, 579 tests): the thirteen dpd fingerprints through the engine, the Rules panel's schema (six groups, payoffs on [−20, 20], `play` shown only in space, every preset on its sliders), live payoff and pairing edits through keyframes and a share link, a bad live mutation named on its field, the stop at the last cycle with Inspect on empty sites, agents and surrounded cooperators, the four Compare entries resolving, and the default sweep's 15 points.

```bash
git add web/src/compare-presets.ts web/src/compare-presets.test.ts web/src/experiments/form.ts web/src/experiments/form.test.ts web/src/determinism.test.ts
git commit -m "Pair the demographic PD runs the sources disagree on in Compare, and sweep the reward by default" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): the four Compare entries open side by side (space vs soup: B's cooperators are gone within about ten cycles); Experiments from a dpd world offers R 1–5 and runs; keyframes and the timeline restore a `dpd-rr-best` run (end-of-cycle removal) exactly; Max speed on `dpd-run-1`.

---

### Task 5: The survey's demographic PD claims

**Files:**
- Create: `survey/src/claims/dpd.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `crate::runner::model_after`, `crate::claim::{all_of, equivalent, greater, range, Claim, Outcome, Source}`; `sugarscape_core::dpd::{DpdConfig, DpdWorld, …}` (`stats.series`, `sites`, `agent_at`, `geometry`) and `ModelWorld::Dpd`.
- Produces: 21 claims (`dpd-…`), Decision 24.

- [ ] **Step 1: The claims and their unit tests**

Create `survey/src/claims/dpd.rs`:

```rust
//! The demographic Prisoner's Dilemma (milestone 17): Epstein's working
//! paper (1997) and chapter (GSS 2006, ch. 9 and its appendix) and Radax &
//! Rengs' replication (RR, 2009), each claim in its source's words. Every
//! claim runs Epstein's and RR's 30 runs (seeds 1–30, whatever `--seeds`
//! says) and reads each run's counts at cycle 500 unless it says otherwise,
//! so its numbers are the plan's measurements. Runs that several claims
//! share are memoized per process. RR's statistic (a pooled two-sample t
//! against the source's mean and s.d., 30 runs each) is reported in the
//! details; the verdicts use the survey's judges.

use std::sync::{Arc, Mutex};

use sugarscape_core::dpd::{
    DeathTiming, DpdConfig, DpdWorld, EndowmentFrom, MetabolismPer, NewbornAge, NewbornsAct,
    Pairing, Play, Removal, Shuffle, Updating,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, range, Claim, Outcome, Source};
use crate::runner::model_after;

const GSS: &str = "Epstein, Generative Social Science (2006), ch. 9";
const GSS_APPENDIX: &str = "Epstein, Generative Social Science (2006), ch. 9 appendix";
const WP: &str = "Epstein, SFI Working Paper 97-12-094 (1997)";
const RR: &str = "Radax & Rengs 2009, MPRA 14419";
const OURS: &str = "spec 2026-09-26-demographic-pd-design.md; plan Decision 17";

/// Epstein's and RR's 30 runs.
const SEEDS: std::ops::RangeInclusive<u64> = 1..=30;

/// RR's critical |t| (df 58, α = 0.05).
const T_CRIT: f64 = 2.0017;

fn dpd(w: &ModelWorld) -> &DpdWorld {
    match w {
        ModelWorld::Dpd(w) => w,
        _ => unreachable!("a demographic PD world"),
    }
}

/// One run: its cooperators and defectors at every cycle (index t is cycle t).
struct Run {
    c: Vec<f64>,
    d: Vec<f64>,
}

impl Run {
    fn c_at(&self, t: usize) -> f64 {
        self.c[t]
    }
    fn d_at(&self, t: usize) -> f64 {
        self.d[t]
    }
}

/// `c` run `ticks` cycles from seeds 1–30 (memoized by config and length).
fn runs(c: &DpdConfig, ticks: u32) -> Arc<Vec<Run>> {
    type Cache = Mutex<Vec<(String, u32, Arc<Vec<Run>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let key = serde_json::to_string(c).expect("configs serialize");
    if let Some((.., v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, t, _)| *k == key && *t == ticks)
    {
        return v.clone();
    }
    let seeds: Vec<u64> = SEEDS.collect();
    let v = Arc::new(model_after(
        &ModelConfig::Dpd(c.clone()),
        &seeds,
        ticks,
        |w| {
            let s = &dpd(w).stats;
            Run {
                c: s.series("cooperators").expect("a dpd series"),
                d: s.series("defectors").expect("a dpd series"),
            }
        },
    ));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, ticks, v.clone()));
    v
}

/// Each run's cooperators and defectors at cycle `t` of a `ticks`-cycle run.
fn counts(c: &DpdConfig, ticks: u32, t: usize) -> (Vec<f64>, Vec<f64>) {
    let v = runs(c, ticks);
    (
        v.iter().map(|r| r.c_at(t)).collect(),
        v.iter().map(|r| r.d_at(t)).collect(),
    )
}

/// Mean and sample standard deviation.
fn mean_sd(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    (
        m,
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0)).sqrt(),
    )
}

/// RR's t for two samples of 30 given as (mean, s.d.): (source − ours) over
/// the pooled standard error.
fn t_from(source: (f64, f64), ours: (f64, f64)) -> f64 {
    (source.0 - ours.0) / ((source.1 * source.1 + ours.1 * ours.1) / 2.0 * 2.0 / 30.0).sqrt()
}

/// RR's t of our 30 runs against the source's (mean, s.d.).
fn t_stat(source: (f64, f64), ours: &[f64]) -> f64 {
    t_from(source, mean_sd(ours))
}

/// A source's number as printed: whole, or to one decimal.
fn printed(x: f64) -> String {
    if x.fract() == 0.0 {
        format!("{x}")
    } else {
        format!("{x:.1}")
    }
}

/// "cooperators 729.2 ± 17.4 against 779 ± 15 (RR's t 11.86)".
fn against(name: &str, v: &[f64], source: (f64, f64)) -> String {
    let (m, sd) = mean_sd(v);
    format!(
        "{name} {m:.1} ± {sd:.1} against {} ± {} (RR's t {:.2})",
        printed(source.0),
        printed(source.1),
        t_stat(source, v)
    )
}

/// Whether both means pass RR's test against a table's (mean, s.d.) pairs.
fn passes(c: &[f64], d: &[f64], table: [(f64, f64); 2]) -> bool {
    t_stat(table[0], c).abs() < T_CRIT && t_stat(table[1], d).abs() < T_CRIT
}

/// Each run's counts inside a table's ranges (cooperators, defectors).
fn in_ranges(c: &[f64], d: &[f64], rc: (f64, f64), rd: (f64, f64)) -> Outcome {
    all_of(vec![
        ("cooperators".into(), range(c, rc.0, rc.1, false)),
        ("defectors".into(), range(d, rd.0, rd.1, false)),
    ])
}

const TABLE_1: [(f64, f64); 2] = [(779.0, 15.0), (121.0, 15.0)];
const TABLE_2: [(f64, f64); 2] = [(784.0, 29.0), (99.0, 25.0)];

fn run_1() -> DpdConfig {
    DpdConfig::default()
}

fn run_2() -> DpdConfig {
    DpdConfig {
        max_age: 100,
        ..DpdConfig::default()
    }
}

fn shifted(c: DpdConfig) -> DpdConfig {
    DpdConfig {
        t: 12.0,
        r: 11.0,
        p: 1.0,
        s: 0.0,
        ..c
    }
}

fn metabolism(m: f64, per: MetabolismPer) -> DpdConfig {
    DpdConfig {
        metabolism: m,
        metabolism_per: per,
        ..shifted(run_2())
    }
}

fn closest(c: DpdConfig) -> DpdConfig {
    DpdConfig {
        play: Play::RandomNeighbor,
        initial_wealth: 0.0,
        newborns_act: NewbornsAct::ThisCycle,
        ..c
    }
}

/// 1 where `f` holds of a run, else 0.
fn indicator(v: &[Run], t: usize, f: impl Fn(f64, f64) -> bool) -> Vec<f64> {
    v.iter()
        .map(|r| f64::from(u8::from(f(r.c_at(t), r.d_at(t)))))
        .collect()
}

/// GSS Table 9.3 as printed: (T, R, cooperators' mean, s.d., 95% CI, range,
/// defectors' mean, s.d., 95% CI, range). Row (4, 2)'s defectors' CI is
/// printed "(254, 376)" (its mean and s.d. give (253, 277)); kept as printed.
type Cell = (u32, u32, [f64; 6], [f64; 6]);
#[rustfmt::skip]
const TABLE_9_3: [Cell; 45] = [
    (10, 9, [809., 19., 802., 816., 772., 845.], [77., 17., 71., 83., 43., 109.]),
    (10, 8, [748., 27., 738., 757., 707., 802.], [132., 23., 124., 140., 82., 169.]),
    (10, 7, [654., 38., 641., 668., 568., 717.], [198., 30., 188., 209., 154., 280.]),
    (10, 6, [469., 60., 447., 490., 312., 604.], [274., 30., 264., 285., 191., 329.]),
    (10, 5, [258., 51., 240., 277., 150., 383.], [270., 26., 261., 279., 202., 319.]),
    (10, 4, [231., 151., 177., 285., 0., 598.], [199., 74., 172., 225., 0., 287.]),
    (10, 3, [0., 0., 0., 0., 0., 1.], [0., 0., 0., 0., 0., 1.]),
    (10, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (10, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (9, 8, [806., 21., 799., 814., 766., 850.], [81., 19., 74., 88., 39., 117.]),
    (9, 7, [728., 34., 716., 740., 669., 793.], [146., 28., 136., 156., 86., 190.]),
    (9, 6, [604., 58., 583., 625., 490., 768.], [225., 38., 212., 239., 102., 303.]),
    (9, 5, [374., 44., 358., 390., 268., 460.], [290., 20., 283., 297., 252., 323.]),
    (9, 4, [203., 78., 175., 231., 98., 442.], [235., 36., 222., 248., 159., 296.]),
    (9, 3, [35., 89., 3., 67., 0., 382.], [38., 84., 8., 68., 0., 284.]),
    (9, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (9, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (8, 7, [807., 30., 796., 818., 744., 879.], [80., 27., 70., 89., 14., 142.]),
    (8, 6, [721., 36., 708., 734., 626., 787.], [153., 30., 142., 163., 98., 231.]),
    (8, 5, [530., 39., 516., 544., 460., 604.], [263., 22., 255., 271., 209., 312.]),
    (8, 4, [259., 57., 239., 279., 128., 387.], [271., 22., 263., 279., 227., 313.]),
    (8, 3, [93., 113., 53., 134., 0., 513.], [113., 99., 77., 148., 0., 279.]),
    (8, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (8, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (7, 6, [797., 28., 787., 807., 739., 852.], [88., 25., 79., 97., 43., 133.]),
    (7, 5, [668., 44., 652., 684., 542., 782.], [187., 33., 175., 199., 101., 271.]),
    (7, 4, [430., 55., 410., 449., 323., 547.], [286., 26., 277., 295., 244., 345.]),
    (7, 3, [126., 101., 90., 162., 0., 370.], [153., 76., 126., 180., 0., 267.]),
    (7, 2, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (7, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (6, 5, [779., 15., 773., 784., 752., 806.], [121., 15., 115., 126., 93., 148.]),
    (6, 4, [587., 33., 576., 599., 524., 658.], [241., 22., 233., 248., 193., 278.]),
    (6, 3, [266., 53., 247., 285., 120., 344.], [280., 29., 270., 291., 199., 320.]),
    (6, 2, [24., 92., 0., 57., 0., 482.], [13., 37., 0., 27., 0., 166.]),
    (6, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (5, 4, [741., 33., 729., 752., 689., 810.], [136., 28., 126., 146., 79., 179.]),
    (5, 3, [473., 46., 456., 489., 379., 574.], [283., 24., 274., 291., 243., 329.]),
    (5, 2, [125., 155., 70., 180., 0., 589.], [114., 89., 82., 145., 0., 260.]),
    (5, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (4, 3, [710., 47., 693., 727., 613., 814.], [159., 36., 146., 172., 76., 231.]),
    (4, 2, [282., 56., 262., 302., 171., 380.], [265., 32., 254., 376., 176., 322.]),
    (4, 1, [0., 0., 0., 0., 0., 0.], [0., 0., 0., 0., 0., 0.]),
    (3, 2, [624., 38., 610., 637., 516., 699.], [221., 30., 210., 232., 164., 284.]),
    (3, 1, [4., 22., 0., 12., 0., 123.], [7., 35., 0., 20., 0., 197.]),
    (2, 1, [361., 47., 344., 377., 256., 471.], [280., 21., 273., 288., 234., 337.]),
];

/// Table 9.3's cell (T, R): S = −T, P = −R, Run 1's other settings.
fn cell(t: u32, r: u32) -> DpdConfig {
    DpdConfig {
        t: f64::from(t),
        r: f64::from(r),
        p: -f64::from(r),
        s: -f64::from(t),
        ..DpdConfig::default()
    }
}

/// A mean inside an interval Epstein printed rounded: it rounds into it.
fn rounds_into(m: f64, lo: f64, hi: f64) -> bool {
    m >= lo - 0.5 && m < hi + 0.5
}

/// Per cell: our (cooperators, defectors) means at 500.
fn table_9_3() -> Vec<(f64, f64)> {
    TABLE_9_3
        .iter()
        .map(|&(t, r, ..)| {
            let (c, d) = counts(&cell(t, r), 500, 500);
            (mean_sd(&c).0, mean_sd(&d).0)
        })
        .collect()
}

/// Swings of the cooperator count from above 400 to below 100 (Decision M's
/// cycle count: hysteresis, so noise near one threshold does not count).
fn swings(r: &Run) -> f64 {
    let (mut high, mut n) = (false, 0.0);
    for &x in &r.c {
        if x > 400.0 {
            high = true;
        } else if x < 100.0 && high {
            high = false;
            n += 1.0;
        }
    }
    n
}

/// The coordination game at cycle `ticks`: per run, whether both
/// conventions are present, and the share of occupied von Neumann neighbour
/// pairs that differ over the share if the same counts were mixed at random
/// (2pq).
fn coordination(ticks: u32) -> Vec<(bool, f64)> {
    let c = DpdConfig {
        r: 1.0,
        s: -3.0,
        t: -3.0,
        p: 1.0,
        max_age: 1000,
        ..DpdConfig::default()
    };
    let seeds: Vec<u64> = SEEDS.collect();
    model_after(&ModelConfig::Dpd(c), &seeds, ticks, |w| {
        let w = dpd(w);
        let (mut unlike, mut pairs, mut coop, mut all) = (0u32, 0u32, 0u32, 0u32);
        for s in 0..w.sites() {
            let Some(a) = w.agent_at(s) else { continue };
            all += 1;
            coop += u32::from(a.cooperator);
            for &j in w.geometry.neighbors(s) {
                if let Some(b) = w.agent_at(j as usize) {
                    pairs += 1;
                    unlike += u32::from(a.cooperator != b.cooperator);
                }
            }
        }
        let p = f64::from(coop) / f64::from(all.max(1));
        let mixed = coop > 0 && coop < all;
        let observed = f64::from(unlike) / f64::from(pairs.max(1));
        (
            mixed,
            observed / (2.0 * p * (1.0 - p)).max(f64::MIN_POSITIVE),
        )
    })
}

/// RR's six model switches in their order, TRUE first: remove dead
/// immediately, die immediately, endowment inherited, random birth age,
/// asynchronous updating, Repast list-shuffle (their RNG column dropped).
fn rr_setting(flags: &str, c: DpdConfig) -> DpdConfig {
    let on: Vec<bool> = flags.chars().map(|f| f == 'T').collect();
    DpdConfig {
        removal: if on[0] {
            Removal::Immediate
        } else {
            Removal::EndOfCycle
        },
        death_timing: if on[1] {
            DeathTiming::Immediate
        } else {
            DeathTiming::OwnTurn
        },
        endowment_from: if on[2] {
            EndowmentFrom::Parent
        } else {
            EndowmentFrom::Granted
        },
        newborn_age: if on[3] {
            NewbornAge::Random
        } else {
            NewbornAge::Zero
        },
        updating: if on[4] {
            Updating::Asynchronous
        } else {
            Updating::Synchronous
        },
        shuffle: if on[5] { Shuffle::Full } else { Shuffle::Swaps },
        ..c
    }
}

/// RR's Table 7: the seven settings that reproduce Run 2 in Repast, six once
/// their RNG column is dropped (rows 3 and 4 differ only there).
const RR_RUN_2_FITS: [&str; 6] = ["TTFTTT", "TFTFTT", "FTTTTF", "FTFTTF", "FFTTTT", "FFFTTT"];

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "dpd-run-1.table-1",
            item: "dpd-run-1",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.2 (Run 1, 30 runs, t = 500): cooperators range (752, 806), mean 779, s.d. 15; defectors range (93, 148), mean 121, s.d. 15",
            check: |_| {
                let (c, d) = counts(&run_1(), 500, 500);
                in_ranges(&c, &d, (752.0, 806.0), (93.0, 148.0)).with(&format!(
                    "{}; {}. Cooperation dominates, but with 50 fewer cooperators than Epstein's.",
                    against("cooperators", &c, TABLE_1[0]),
                    against("defectors", &d, TABLE_1[1])
                ))
            },
        },
        Claim {
            id: "dpd-run-2.table-2",
            item: "dpd-run-2",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.4 (Run 2, maximum age 100): cooperators range (708, 846), mean 784, s.d. 29; defectors range (45, 160), mean 99, s.d. 25",
            check: |_| {
                let (c, d) = counts(&run_2(), 500, 500);
                in_ranges(&c, &d, (708.0, 846.0), (45.0, 160.0)).with(&format!(
                    "{}; {}.",
                    against("cooperators", &c, TABLE_2[0]),
                    against("defectors", &d, TABLE_2[1])
                ))
            },
        },
        Claim {
            id: "dpd-run-1.five-to-one",
            item: "dpd-run-1",
            source: Source::Book,
            citation: GSS,
            text: "Run 1: by t = 50 a stable ratio of cooperators to defectors, approximately 5 to 1 (each run's ratio within 10% of 5)",
            check: |_| {
                let v = runs(&run_1(), 500);
                let ratio = |t: usize| -> Vec<f64> { v.iter().map(|r| r.c_at(t) / r.d_at(t)).collect() };
                let mean = |t: usize| {
                    let c: f64 = v.iter().map(|r| r.c_at(t)).sum();
                    c / v.iter().map(|r| r.d_at(t)).sum::<f64>()
                };
                range(&ratio(50), 5.0, 5.0, true).with(&format!(
                    "Ratio of the means: {:.2} at t = 30, {:.2} at 50, {:.2} at 100, {:.2} at 500 — stable from about t = 30, as Epstein says, but about 4.3 to 1.",
                    mean(30),
                    mean(50),
                    mean(100),
                    mean(500)
                ))
            },
        },
        Claim {
            id: "dpd-payoffs.table-9-3",
            item: "dpd-payoffs",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.3 (45 payoff vectors, S = −T, P = −R, 30 runs each at t = 500): our mean cooperators and defectors in each cell inside Epstein's 95% confidence interval and range (one value per mean: 90 of each)",
            check: |_| {
                let ours = table_9_3();
                let (mut ci, mut rg, mut both, mut t_pass) = (vec![], vec![], 0, 0);
                for (&(t, r, ec, ed), &(c, d)) in TABLE_9_3.iter().zip(&ours) {
                    let (cc, dc) = (rounds_into(c, ec[2], ec[3]), rounds_into(d, ed[2], ed[3]));
                    ci.extend([f64::from(u8::from(cc)), f64::from(u8::from(dc))]);
                    rg.push(f64::from(u8::from(rounds_into(c, ec[4], ec[5]))));
                    rg.push(f64::from(u8::from(rounds_into(d, ed[4], ed[5]))));
                    both += usize::from(cc && dc);
                    let (cs, ds) = counts(&cell(t, r), 500, 500);
                    t_pass += usize::from(passes(&cs, &ds, [(ec[0], ec[1]), (ed[0], ed[1])]));
                }
                all_of(vec![
                    ("means in the 95% CI".into(), range(&ci, 1.0, 1.0, false)),
                    ("means in the range".into(), range(&rg, 1.0, 1.0, false)),
                ])
                .with(&format!(
                    "Cells with both means in the CI: {both} of 45; passing RR's two t-tests: {t_pass} of 45. Our cooperators run 13–92 below Epstein's along the diagonal (R = T − 1) and our defectors 33–106 above. Row (4, 2)'s defectors' CI is counted as printed, (254, 376); its mean and s.d. give (253, 277)."
                ))
            },
        },
        Claim {
            id: "dpd-payoffs.collapse",
            item: "dpd-payoffs",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.3: cooperators die out (mean 0) at R ≤ 3 when T = 10, R ≤ 2 when T = 7–9 and R = 1 when T ≤ 6, and survive in every other cell",
            check: |_| {
                let ours = table_9_3();
                let agree: Vec<f64> = TABLE_9_3
                    .iter()
                    .zip(&ours)
                    .map(|(&(_, _, ec, _), &(c, _))| f64::from(u8::from((ec[0] == 0.0) == (c < 0.5))))
                    .collect();
                range(&agree, 1.0, 1.0, false)
            },
        },
        Claim {
            id: "dpd-payoffs.all-zero",
            item: "dpd-payoffs",
            source: Source::Book,
            citation: GSS,
            text: "Table 9.3: where cooperation collapses, the whole population dies out: 0 cooperators and 0 defectors in every run (ranges (0, 0))",
            check: |_| {
                let mut d = Vec::new();
                for &(t, r, ec, ed) in &TABLE_9_3 {
                    if ec[5] == 0.0 && ed[5] == 0.0 {
                        d.extend(counts(&cell(t, r), 500, 500).1);
                    }
                }
                range(&d, 0.0, 0.0, false).with(&format!(
                    "Defectors at 500 over the {} runs of the 11 all-zero cells: mean {:.1}. Once cooperators are gone the last defectors have nobody to play, and with no maximum age and no metabolism nothing kills them.",
                    d.len(),
                    mean_sd(&d).0
                ))
            },
        },
        Claim {
            id: "dpd-run-4.cycles",
            item: "dpd-run-4",
            source: Source::Book,
            citation: GSS,
            text: "Run 4 (R = 1, maximum age 100): predator–prey cycles — at least three swings of the cooperators (above 400, then below 100) in 2,000 cycles, and populations that live on",
            check: |_| {
                let c = DpdConfig {
                    r: 1.0,
                    ..run_2()
                };
                let v = runs(&c, 2000);
                let s: Vec<f64> = v.iter().map(swings).collect();
                let alive = indicator(&v, 2000, |c, d| c + d > 0.0);
                let at_500 = indicator(&v, 500, |c, d| c + d > 0.0);
                all_of(vec![
                    ("swings".into(), range(&s, 3.0, f64::INFINITY, false)),
                    ("alive at 2,000".into(), range(&alive, 1.0, 1.0, false)),
                ])
                .with(&format!(
                    "Mean {:.1} swings, at most {}; {} of 30 populations alive at 500, none at 2,000 — no run sustains cycles, and no seed ends in cooperator monopoly.",
                    mean_sd(&s).0,
                    s.iter().copied().fold(0.0, f64::max),
                    at_500.iter().sum::<f64>()
                ))
            },
        },
        Claim {
            id: "dpd-run-4.paradox",
            item: "dpd-run-4",
            source: Source::Book,
            citation: GSS,
            text: "\"Cooperators ultimately do better with a low payoff (R = 1) than with a high one (R = 5)!\" — more cooperators at 2,000 cycles with R = 1 than with R = 5 (maximum age 100)",
            check: |_| {
                let low = DpdConfig {
                    r: 1.0,
                    ..run_2()
                };
                let (cl, dl) = counts(&low, 2000, 2000);
                let (ch, dh) = counts(&run_2(), 2000, 2000);
                let monopolies = |c: &[f64], d: &[f64]| c.iter().zip(d).filter(|(c, d)| **c > 0.0 && **d == 0.0).count();
                greater(&cl, &ch, "R = 1", "R = 5").with(&format!(
                    "Cooperator monopolies at 2,000: {} of 30 at R = 1, {} of 30 at R = 5; every R = 1 population dies out.",
                    monopolies(&cl, &dl),
                    monopolies(&ch, &dh)
                ))
            },
        },
        Claim {
            id: "dpd-run-5.persists",
            item: "dpd-run-5",
            source: Source::Book,
            citation: GSS,
            text: "Run 5 (Run 2 with 50% mutation): cooperation persists through 10,000 cycles",
            check: |_| {
                let c = DpdConfig {
                    mutation: 0.5,
                    ..run_2()
                };
                let v = runs(&c, 10_000);
                let (co, de) = counts(&c, 10_000, 10_000);
                range(&co, 1.0, f64::INFINITY, false).with(&format!(
                    "{} of 30 populations die out entirely; at 10,000: {:.0} cooperators, {:.0} defectors (means); the lowest cooperator count in a surviving run: {:.0}.",
                    v.iter().filter(|r| r.c_at(10_000) + r.d_at(10_000) == 0.0).count(),
                    mean_sd(&co).0,
                    mean_sd(&de).0,
                    v.iter()
                        .filter(|r| r.c_at(10_000) > 0.0)
                        .map(|r| r.c.iter().copied().fold(f64::INFINITY, f64::min))
                        .fold(f64::INFINITY, f64::min)
                ))
            },
        },
        Claim {
            id: "dpd-soup.pure-defection",
            item: "dpd-soup",
            source: Source::Book,
            citation: GSS,
            text: "Soup (equiprobable random agent pairings): the population runs to pure defection — no cooperators at t = 500",
            check: |_| {
                let c = DpdConfig {
                    pairing: Pairing::Soup,
                    ..run_1()
                };
                let v = runs(&c, 500);
                let (co, _) = counts(&c, 500, 500);
                let gone: Vec<f64> = v
                    .iter()
                    .filter_map(|r| r.c.iter().position(|&x| x == 0.0))
                    .map(|t| t as f64)
                    .collect();
                range(&co, 0.0, 0.0, false).with(&format!(
                    "The last cooperator dies at cycle {:.1} on average ({} runs); then the defectors starve each other: {} of 30 populations are empty at 500, the rest one lone agent.",
                    mean_sd(&gone).0,
                    gone.len(),
                    indicator(&v, 500, |c, d| c + d == 0.0).iter().sum::<f64>()
                ))
            },
        },
        Claim {
            id: "dpd-shifted.pure-defection",
            item: "dpd-shifted",
            source: Source::Book,
            citation: GSS,
            text: "Payoffs shifted up by 6 (12, 11, 1, 0), maximum age 100, zero mutation: the population converges to pure defection (Fig. 13) — no cooperators at t = 500",
            check: |_| {
                let (c, d) = counts(&shifted(run_2()), 500, 500);
                let (c5, _) = counts(
                    &shifted(DpdConfig {
                        mutation: 0.5,
                        ..run_2()
                    }),
                    500,
                    500,
                );
                let (c1, _) = counts(&shifted(run_1()), 500, 500);
                range(&c, 0.0, 0.0, false).with(&format!(
                    "{:.1} cooperators, {:.1} defectors (means); every run coexists. With Run 5's settings (the working paper's context) {:.1} cooperators, with Run 1's {:.1}. With no negative payoff only old age kills; the full lattice's vacancies go to whichever neighbour moves first.",
                    mean_sd(&c).0,
                    mean_sd(&d).0,
                    mean_sd(&c5).0,
                    mean_sd(&c1).0
                ))
            },
        },
        Claim {
            id: "dpd-metabolism.per-cycle",
            item: "dpd-metabolism",
            source: Source::Book,
            citation: WP,
            text: "The shifted payoffs with a metabolism of 6 \"recover, in effect, our initial payoffs\" (metabolism \"a fixed decrement to accumulated payoff per cycle\", note 29): cooperators at 500 as in Run 2",
            check: |_| {
                let (c, _) = counts(&metabolism(6.0, MetabolismPer::Cycle), 500, 500);
                let (base, _) = counts(&run_2(), 500, 500);
                equivalent(&c, &base, None, "metabolism 6 per cycle", "Run 2").with(&format!(
                    "{}. Cooperative, but a different model: its spread is three times Run 2's.",
                    against("cooperators", &c, mean_sd(&base))
                ))
            },
        },
        Claim {
            id: "dpd-metabolism.per-interaction",
            item: "dpd-metabolism",
            source: Source::Book,
            citation: GSS,
            text: "\"Equivalent mathematically\": the shifted payoffs with a metabolism of 6 \"imposed on all agents after every interaction\" give Run 2",
            check: |_| {
                let per_game = runs(&metabolism(6.0, MetabolismPer::Interaction), 500);
                let base = runs(&run_2(), 500);
                let same = per_game
                    .iter()
                    .zip(base.iter())
                    .filter(|(a, b)| a.c == b.c && a.d == b.d)
                    .count();
                let (c, _) = counts(&metabolism(6.0, MetabolismPer::Interaction), 500, 500);
                let (b, _) = counts(&run_2(), 500, 500);
                equivalent(&c, &b, None, "metabolism 6 per game", "Run 2").with(&format!(
                    "Identical counts at every cycle in {same} of 30 runs: each game's payoff less 6 is the original payoff."
                ))
            },
        },
        Claim {
            id: "dpd-metabolism.necessity",
            item: "dpd-metabolism",
            source: Source::Book,
            citation: GSS,
            text: "\"Necessity is the mother of cooperation\": on the shifted payoffs a higher metabolism (5 per cycle) leaves more cooperators than a low one (1)",
            check: |_| {
                let (hi, _) = counts(&metabolism(5.0, MetabolismPer::Cycle), 500, 500);
                let (lo, _) = counts(&metabolism(1.0, MetabolismPer::Cycle), 500, 500);
                greater(&hi, &lo, "metabolism 5", "metabolism 1")
            },
        },
        Claim {
            id: "dpd-footnote-27.monopoly",
            item: "dpd-footnote-27",
            source: Source::Book,
            citation: GSS,
            text: "Footnote 27: payoffs T 16, R 11, P 5, S 4 and a maximum lifetime of 10 cycles give an evolution to cooperative monopoly (at 2,000 cycles)",
            check: |_| {
                let c = DpdConfig {
                    t: 16.0,
                    r: 11.0,
                    p: 5.0,
                    s: 4.0,
                    max_age: 10,
                    ..DpdConfig::default()
                };
                let v = runs(&c, 2000);
                let mono = indicator(&v, 2000, |c, d| c > 0.0 && d == 0.0);
                range(&mono, 1.0, 1.0, false).with(&format!(
                    "{} of 30 runs reach cooperative monopoly by 2,000, none by 500. \"Hiked by ten\" would make R 15; that gives the same.",
                    mono.iter().sum::<f64>()
                ))
            },
        },
        Claim {
            id: "dpd-coordination.regions",
            item: "dpd-coordination",
            source: Source::Book,
            citation: GSS_APPENDIX,
            text: "The coordination game (payoffs [1, −3, −3, 1], death age 1,000): persistent norm maps — both conventions still present at 2,000 cycles, in regions (unlike neighbours under half as common as if mixed at random)",
            check: |_| {
                let v = coordination(2000);
                let present: Vec<f64> = v.iter().map(|x| f64::from(u8::from(x.0))).collect();
                let clumped: Vec<f64> = v.iter().filter(|x| x.0).map(|x| x.1).collect();
                all_of(vec![
                    ("both present".into(), range(&present, 1.0, 1.0, false)),
                    ("regions".into(), range(&clumped, 0.0, 0.5, false)),
                ])
                .with("Measured at 100, 500, 2,000 and 5,000 cycles: both conventions persist in 18, 17, 16 and 11 of 30 runs — the maps persist over hundreds of cycles and erode over thousands.")
            },
        },
        Claim {
            id: "dpd-rr.run-1",
            item: "dpd-run-1",
            source: Source::Book,
            citation: RR,
            text: "Radax & Rengs: of their timing settings only one reproduces Run 1, a synchronous one they set aside — no asynchronous setting of the six model switches passes their t-tests against Table 1",
            check: |_| {
                let mut fails = Vec::new();
                for bits in 0..32u32 {
                    // Asynchronous (the fifth switch) and every setting of the other five.
                    let flags: String = [0, 1, 2, 3, 99, 4]
                        .map(|k| if k == 99 || bits & (1 << k) == 0 { 'T' } else { 'F' })
                        .iter()
                        .collect();
                    let (c, d) = counts(&rr_setting(&flags, run_1()), 500, 500);
                    fails.push(f64::from(u8::from(!passes(&c, &d, TABLE_1))));
                }
                range(&fails, 1.0, 1.0, false).with(
                    "Of all 64 settings none passes here, synchronous ones included (the core's radax_and_rengs_factorial_as_measured pins the full factorial).",
                )
            },
        },
        Claim {
            id: "dpd-rr.run-2",
            item: "dpd-run-2",
            source: Source::Book,
            citation: RR,
            text: "Radax & Rengs' Table 7: seven settings of their switches (six once their RNG column is dropped) reproduce Run 2 — each passes their t-tests against Table 2",
            check: |_| {
                let mut pass = Vec::new();
                let mut notes = Vec::new();
                for flags in RR_RUN_2_FITS {
                    let (c, d) = counts(&rr_setting(flags, run_2()), 500, 500);
                    pass.push(f64::from(u8::from(passes(&c, &d, TABLE_2))));
                    notes.push(format!(
                        "{flags}: {:.0} / {:.0} (t {:.1} / {:.1})",
                        mean_sd(&c).0,
                        mean_sd(&d).0,
                        t_stat(TABLE_2[0], &c),
                        t_stat(TABLE_2[1], &d)
                    ));
                }
                range(&pass, 1.0, 1.0, false).with(&format!(
                    "{}. Our only Run 2 fit of the 64 is TFTTTT (785 / 110), not one of theirs.",
                    notes.join("; ")
                ))
            },
        },
        Claim {
            id: "dpd-rr-best.table-2",
            item: "dpd-rr-best",
            source: Source::Book,
            citation: RR,
            text: "Radax & Rengs' best Run 2 fit (dead removed at the end of the cycle, death at once, endowment granted, random newborn age, asynchronous, Epstein's swaps): 780 (25) cooperators, 97 (22) defectors, inside Table 2's ranges (708, 846) and (45, 160)",
            check: |_| {
                let (c, d) = counts(&rr_setting("FTFTTF", run_2()), 500, 500);
                in_ranges(&c, &d, (708.0, 846.0), (45.0, 160.0)).with(&format!(
                    "{}; {}. The same six switches in two implementations give different models.",
                    against("cooperators", &c, TABLE_2[0]),
                    against("defectors", &d, TABLE_2[1])
                ))
            },
        },
        Claim {
            id: "dpd-working-paper.table-1",
            item: "dpd-working-paper",
            source: Source::Book,
            citation: WP,
            text: "Table 1 of the working paper, whose rule plays one random neighbour a turn: cooperators range (752, 806), mean 779; defectors range (93, 148), mean 121",
            check: |_| {
                let wp = |c: DpdConfig| DpdConfig {
                    play: Play::RandomNeighbor,
                    ..c
                };
                let (c, d) = counts(&wp(run_1()), 500, 500);
                let (c2, d2) = counts(&wp(run_2()), 500, 500);
                in_ranges(&c, &d, (752.0, 806.0), (93.0, 148.0)).with(&format!(
                    "{}; {} — nearer Table 1 than the published rule's 729 / 171, but still rejected. Run 2 with this rule: {}; {}.",
                    against("cooperators", &c, TABLE_1[0]),
                    against("defectors", &d, TABLE_1[1]),
                    against("cooperators", &c2, TABLE_2[0]),
                    against("defectors", &d2, TABLE_2[1])
                ))
            },
        },
        Claim {
            id: "dpd-closest.both-tables",
            item: "dpd-closest",
            source: Source::App,
            citation: OURS,
            text: "dpd-closest (the working paper's rule, no initial wealth, newborns acting at once) reproduces both tables: each run inside Tables 9.2's and 9.4's ranges",
            check: |_| {
                let (c1, d1) = counts(&closest(run_1()), 500, 500);
                let (c2, d2) = counts(&closest(run_2()), 500, 500);
                all_of(vec![
                    ("Table 1".into(), in_ranges(&c1, &d1, (752.0, 806.0), (93.0, 148.0))),
                    ("Table 2".into(), in_ranges(&c2, &d2, (708.0, 846.0), (45.0, 160.0))),
                ])
                .with(&format!(
                    "{}; {}; {}; {}. On seeds 31–60 and 61–90 Run 2's defectors fail RR's test (t −2.6, −3.2).",
                    against("Run 1 cooperators", &c1, TABLE_1[0]),
                    against("Run 1 defectors", &d1, TABLE_1[1]),
                    against("Run 2 cooperators", &c2, TABLE_2[0]),
                    against("Run 2 defectors", &d2, TABLE_2[1])
                ))
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_9_3_rows_are_the_printed_ones() {
        assert_eq!(TABLE_9_3.len(), 45);
        let mut cells = Vec::new();
        for t in (2..=10).rev() {
            for r in (1..t).rev() {
                cells.push((t, r));
            }
        }
        let ours: Vec<(u32, u32)> = TABLE_9_3.iter().map(|&(t, r, ..)| (t, r)).collect();
        assert_eq!(ours, cells);
        // Run 1 is the cell (6, 5).
        let run_1 = TABLE_9_3.iter().find(|c| (c.0, c.1) == (6, 5)).unwrap();
        assert_eq!(
            (run_1.2[0], run_1.2[1], run_1.3[0], run_1.3[1]),
            (779.0, 15.0, 121.0, 15.0)
        );
    }

    #[test]
    fn rr_flags_set_the_switches_in_their_order() {
        let c = rr_setting("FTFTTF", run_2());
        assert_eq!(
            (
                c.removal,
                c.death_timing,
                c.endowment_from,
                c.newborn_age,
                c.updating,
                c.shuffle
            ),
            (
                Removal::EndOfCycle,
                DeathTiming::Immediate,
                EndowmentFrom::Granted,
                NewbornAge::Random,
                Updating::Asynchronous,
                Shuffle::Swaps
            )
        );
        assert_eq!(rr_setting("TTTTTT", run_1()).shuffle, Shuffle::Full);
        assert_eq!(rr_setting("TTTTTF", run_1()), run_1());
    }

    #[test]
    fn rr_t_matches_their_appendix() {
        // RR's appendix row 1: 779 (15) against 755 (16) prints t = 6.01
        // (from unrounded means).
        assert!((t_from((779.0, 15.0), (755.0, 16.0)) - 5.99).abs() < 0.01);
        let v: Vec<f64> = (0..30)
            .map(|i| if i % 2 == 0 { 771.0 } else { 739.0 })
            .collect();
        assert_eq!(t_stat((755.0, 15.0), &v), 0.0);
        assert!(passes(&[779.0; 30], &[121.0; 30], TABLE_1));
        assert!(!passes(&[729.0; 30], &[121.0; 30], TABLE_1));
    }

    #[test]
    fn swings_need_both_thresholds() {
        let run = |c: Vec<f64>| Run {
            d: vec![0.0; c.len()],
            c,
        };
        assert_eq!(swings(&run(vec![500.0, 50.0, 500.0, 50.0])), 2.0);
        assert_eq!(swings(&run(vec![500.0, 300.0, 500.0, 50.0])), 1.0);
        assert_eq!(swings(&run(vec![50.0, 500.0, 300.0])), 0.0);
    }
}
```

In `survey/src/claims/mod.rs` add `mod dpd;` after `mod culture;` and `dpd::claims(),` after `culture::claims(),`.

Do not run `cargo fmt` over the survey crate (it is not fmt-clean); check the new file alone.

Run: `(cd survey && rustfmt --edition 2021 --check src/claims/dpd.rs && cargo test)` — Expected: PASS (20 tests, 4 of them `claims::dpd::tests`). (Its RED — the four unit tests before `dpd.rs` exists — is a compile error: file not found for module `dpd`.)

- [ ] **Step 2: Run it**

Run: `(cd survey && cargo run --release -- --only dpd-)` — Expected (seeds 1–30 regardless of `--seeds`; about 20 s on 10 threads, the slowest claims `dpd-payoffs.table-9-3` and `dpd-rr.run-1` about 5 s): **Holds 6** — `dpd-payoffs.collapse`, `dpd-run-5.persists`, `dpd-soup.pure-defection`, `dpd-metabolism.per-interaction`, `dpd-metabolism.necessity`, `dpd-rr.run-1`; **Weak 4** — `dpd-metabolism.per-cycle`, `dpd-coordination.regions`, `dpd-working-paper.table-1`, `dpd-closest.both-tables`; **Fails 11** — `dpd-run-1.table-1`, `dpd-run-2.table-2`, `dpd-run-1.five-to-one`, `dpd-payoffs.table-9-3`, `dpd-payoffs.all-zero`, `dpd-run-4.cycles`, `dpd-run-4.paradox`, `dpd-shifted.pure-defection`, `dpd-footnote-27.monopoly`, `dpd-rr.run-2`, `dpd-rr-best.table-2`. The measured values are Decision M's (e.g. Table 1: 729.2 ± 17.4 / 170.5 ± 17.3, t 11.86 / −11.83; Table 9.3: 22/90 in CI, 51/90 in range, 1 cell both, 5 cells pass). Delete `survey/out/results-dpd-.json` (not committed).

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/dpd.rs survey/src/claims/mod.rs
git commit -m "Survey the demographic PD claims: six hold, four are weak, eleven fail" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 6: README, roadmap, spec notes and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/superpowers/specs/2026-09-26-demographic-pd-design.md`

- [ ] **Step 1: The spec's notes** (Decisions 10, 15, 17, 20–24)

Apply to the spec (each hunk a note that changes it):

```diff
--- a/docs/superpowers/specs/2026-09-26-demographic-pd-design.md
+++ b/docs/superpowers/specs/2026-09-26-demographic-pd-design.md
@@ -31,7 +31,7 @@
 - **Runs:** Run 1 (no maximum age): about 5 to 1 by t = 50; Table 1 / 9.2 (30 runs, t = 500): cooperators range (752, 806), mean 779, s.d. 15; defectors (93, 148), 121, 15. Run 2 (maximum age 100): Table 2 / 9.4: cooperators (708, 846), 784, 29; defectors (45, 160), 99, 25. Run 3 (R = 2): more oscillatory. Run 4 (R = 1): predator–prey cycles; outcomes differ by seed (coexistence, cooperator monopoly or extinction); "cooperators ultimately do better with a low payoff (R = 1) than with a high one (R = 5)!… But it is not robust." Run 5 (Run 2 with 50 % mutation): cooperation persists through 10,000 cycles. Soup ("equiprobable random agent pairings") runs to pure defection. Payoffs shifted by 6 (12, 11, 1, 0) run to pure defection (Fig. 13). "Equivalent mathematically": the shifted payoffs with a metabolism of 6 "recover, in effect, our initial payoffs" (metabolism = "a fixed decrement to accumulated payoff per cycle", WP note 29); the passage calls the pure-defection run "figure 12" (the 50 %-mutation run; it means Fig. 13). Footnote 27: payoffs 16, 11, 5, 4 with maximum lifetime 10 evolve to cooperative monopoly. Surrounded cooperators: cooperators all eight of whose Moore neighbours are cooperators (Fig. 10–11).
 - **GSS Table 9.3:** 45 payoff vectors (T = 2 … 10, R = 1 … T − 1, S = −T, P = −R), 30 runs each at t = 500, Run 1's other settings: range, mean, s.d. and 95 % CI for cooperators and defectors (e.g. (10, 9): 809 / 77; (6, 5): 779 / 121; (4, 1): 0 / 0). Row (4, 2)'s defector CI "(254, 376)" contradicts its mean 265 and s.d. 32 (≈ (253, 277)).
 - **GSS appendix:** the same mechanics with payoffs [1, −3, −3, 1], death age 1,000, mutation 0 give persistent "norm maps" (regions of each convention, accidents at their borders).
-- **RR:** seven unstated binary choices — dead removed immediately vs at the end of the cycle; death immediately (even on another's turn) vs on one's own turn; endowment taken from the parent vs granted; newborns' age random vs 0; asynchronous vs synchronous (all move, then all play, then all reproduce); RNG library; Repast's shuffle vs Epstein's swaps — a 2⁷ factorial × 30 runs, Welch t-tests against Tables 1 and 2: **0 of 128 settings reproduce Run 1, 7 of 128 reproduce Run 2, none both**; best Run 2 fit: removal at end of cycle, immediate death, endowment granted, random newborn age, asynchronous, Epstein's swaps → 780 (25) / 97 (22).
+- **RR:** seven unstated binary choices — dead removed immediately vs at the end of the cycle; death immediately (even on another's turn) vs on one's own turn; endowment taken from the parent vs granted; newborns' age random vs 0; asynchronous vs synchronous (all move, then all play, then all reproduce); RNG library; Repast's shuffle vs Epstein's swaps — a 2⁷ factorial × 30 runs, t-tests against Tables 1 and 2: **1 of 128 settings reproduces Run 1 (a synchronous one, which they set aside as "radically different from the original model"; their summary counts none), 7 of 128 reproduce Run 2, none both**; best Run 2 fit: removal at end of cycle, immediate death, endowment granted, random newborn age, asynchronous, Epstein's swaps → 780 (25) / 97 (22).
 
 ## Architecture
 
@@ -97,8 +97,8 @@
 ## Views
 
 - **Colour modes:** **Strategy** (default; cooperators blue, defectors red, GSS's colours), **Wealth** (heat), **Age** (heat), **Surrounded** (surrounded cooperators highlighted). Empty sites dark.
-- **Inspect:** an agent's strategy, wealth, age and maximum age, whether surrounded, and each neighbour with this cycle's payoff between them; an empty site says so.
-- **Charts:** **Population** (cooperators, defectors), **Cooperator share**, **Surrounded cooperators**, **Wealth** (mean C, mean D), **Births and deaths**. (A cooperator-vs-defector phase diagram, Figs. 4 and 9, only if the chart library supports it without a new chart type.)
+- **Inspect:** an agent's strategy, wealth, age and maximum age, whether surrounded, this cycle's payoffs and games, and each neighbour with what one game between them pays each under the current payoffs (agents move, so per-partner totals are not kept: plan Decision 13); an empty site says so.
+- **Charts:** **Population** (cooperators, defectors), **Cooperator share**, **Surrounded cooperators**, **Wealth** (mean C, mean D), **Births and deaths**, against the **cycle** (Epstein's word). No cooperator-vs-defector phase diagram (Figs. 4 and 9): the model charts are time charts only, and a phase diagram would be a new chart type (plan Decision 22).
 
 ## Presets
 
@@ -110,37 +110,38 @@
 | `dpd-run-4` | max age 100, R 1 | Run 4 |
 | `dpd-run-5` | max age 100, mutation 0.5 | Run 5 |
 | `dpd-working-paper` | `play: random_neighbor` | WP's rule |
+| `dpd-closest` | `play: random_neighbor`, `initial_wealth: 0`, `newborns_act: this_cycle` | the readings that reproduce Tables 1 and 2 together (plan Decision 17) |
 | `dpd-soup` | `pairing: soup` | the soup variant |
-| `dpd-shifted` | max age 100, mutation 0.5, payoffs 12, 11, 1, 0 | Fig. 13 |
+| `dpd-shifted` | max age 100, payoffs 12, 11, 1, 0 (GSS: "maximum age of 100, zero mutation" — Run 2's settings; plan Decision 10) | Fig. 13 |
 | `dpd-metabolism` | `dpd-shifted` with metabolism 6 | the "equivalent" run |
 | `dpd-footnote-27` | payoffs 16, 11, 5, 4, max age 10 | footnote 27 |
 | `dpd-rr-best` | RR's best Run 2 fit | RR |
 | `dpd-coordination` | CD's payoffs [1, −3, −3, 1] as (CC, CD, DC, DD): R 1, S −3, T −3, P 1; max age 1,000 | GSS appendix |
 
-(The runs following Run 5 — `dpd-shifted`, `dpd-metabolism` — take Run 5's settings as "all else as before"; the dry run checks this against the figures' descriptions and records the reading.)
+(`dpd-shifted` and `dpd-metabolism` take Run 2's settings, as GSS states them — not Run 5's, which the working paper's context and its "figure 12" suggest: with 50 % mutation pure defection is impossible. No reading converges to pure defection; plan Decision 10 and the measurements.)
 
-**Compare entries:** "Working paper vs published rule — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-working-paper`), "Negative payoffs vs shifted with metabolism — Demographic PD (Compare)" (`dpd-run-5` vs `dpd-metabolism`), "Space vs soup — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-soup`).
+**Compare entries:** "Working paper vs published rule — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-working-paper`), "Negative payoffs vs shifted with metabolism — Demographic PD (Compare)" (`dpd-run-2` vs `dpd-metabolism`: `dpd-metabolism` has Run 2's settings), "Space vs soup — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-soup`), "Published rule vs closest reading — Demographic PD (Compare)" (`dpd-run-1` vs `dpd-closest`; plan Decision 21).
 
 ## Experiments and CLI
 
 Thirty seeds, 500 cycles, metric the final value unless noted.
 
-- `dpd-payoffs`: Table 9.3's 45 cells (T = 2 … 10 as series, R as x; cells with R ≥ T skipped or laid out as the sweep format allows — the plan settles it); metrics `cooperators` (and the survey reads `defectors`).
+- `dpd-payoffs`: Table 9.3's 45 cells (T = 2 … 10 as series, R = 1 … 9 as x, each value setting `p = −r` and `s = −t`); the sweep format cannot skip a cell, so the 36 cells with R ≥ T also run and the description says so (plan Decision 15); metric `cooperators` (the survey reads `defectors` itself).
 - `dpd-mutation`: base Run 2; x = mutation 0 … 0.5.
 - `dpd-metabolism`: base `dpd-shifted`; x = metabolism 0 … 6; series `metabolism_per` cycle vs interaction; metric `cooperators`.
-- `dpd-max-age`: x = max age 10 … 1,000.
+- `dpd-max-age`: base Run 1; x = max age 10 … 1,000.
 - `sugarscape presets | run | sweep` accept `dpd`.
 
 ## Claims to test (survey and book-style tests, 30 seeds as Epstein)
 
-- **Epstein:** Table 1 and Table 2 (means and ranges); Table 9.3 cell by cell (our mean inside Epstein's 95 % CI, and within his range); Run 1's ~5 : 1 by t = 50; Run 4's oscillation (a measured cycle count) and seed-dependent outcomes; the R = 1 paradox (share of seeds ending in cooperator monopoly at R = 1 vs R = 5); Run 5's persistence through 10,000 cycles; soup → pure defection; shifted payoffs → pure defection; the metabolism equivalence (per cycle, per interaction); footnote 27's monopoly; the coordination game's persistent regions.
-- **RR:** the 64 combinations of the six model switches (RNG excluded) × 30 seeds against Tables 1 and 2 (RR: 0 / 7 of 128, none both); RR's best fit.
+- **Epstein:** Table 1 and Table 2 (means and ranges); Table 9.3 cell by cell (our mean inside Epstein's 95 % CI, and within his range); Run 1's ~5 : 1 by t = 50; Run 4's oscillation (a measured cycle count) and seed-dependent outcomes; the R = 1 paradox (share of seeds ending in cooperator monopoly at R = 1 vs R = 5); Run 5's persistence through 10,000 cycles; soup → pure defection; shifted payoffs → pure defection; the metabolism equivalence (per cycle, per interaction: exact per interaction, measured); footnote 27's monopoly (printed R = 11, though "hiked by ten" gives 15: both measured); the coordination game's persistent regions.
+- **RR:** the 64 combinations of the six model switches (RNG excluded) × 30 seeds against Tables 1 and 2 (RR: 1 / 7 of 128, none both); RR's best fit. The core's book-style test runs all 64; the survey runs subsets (the 32 asynchronous settings for Run 1, RR's own seven Run 2 fits — six without the RNG column), plan Decision 24.
 
 Tolerances come from the measurements (as milestones 11–16).
 
 ## Page
 
-- A **Demographic PD** presets group; the schema panel in groups **Game** (T, R, P, S), **Population** (width, agents, initial cooperators, initial wealth, fission wealth, endowment, max age, metabolism and its timing, vision), **Evolution** (mutation), **Timing** (death timing, removal, endowment from, newborn age, newborns act, updating, shuffle), **Interaction** (play, pairing), **Run** (end); the four colour modes; the charts; Inspect; the three Compare entries; `defaultForm('dpd')` (x = R, 30 seeds, final `cooperators` at 500).
+- A **Demographic PD** presets group; the schema panel in groups **Game** (T, R, P, S), **Population** (width, agents, initial cooperators, initial wealth, fission wealth, endowment, max age, metabolism and its timing, vision), **Evolution** (mutation), **Timing** (death timing, removal, endowment from, newborn age, newborns act, updating, shuffle), **Interaction** (play, pairing), **Run** (end); the four colour modes; the charts; Inspect; the four Compare entries; `defaultForm('dpd')` (x = R 1–5, the form's usual 3 seeds, final `cooperators` at 500; plan Decision 23).
 - Keyframes, the timeline, stop rules, share links, sessions, recording and Compare work unchanged.
 
 ## Testing
```

- [ ] **Step 2: README**

In **Other artificial societies**, the presets menu's groups end "**Emergence of Classes**, **Ethnocentrism** and **Demographic PD**." After the Ethnocentrism section (before `## Experiments`), write:

```markdown
### Demographic Prisoner's Dilemma (Epstein 1998, and its replication)

Epstein's demographic Prisoner's Dilemma: 100 agents on a 30 × 30 torus, each with a fixed strategy,
cooperate or defect, and a wealth of 6. In a random order each agent in turn moves to a random
unoccupied site within its vision (one site, von Neumann), plays the Prisoner's Dilemma with each
neighbor (T 6, R 5, P −5, S −6; both players are paid), has an offspring on a free neighboring site
once its wealth reaches 11 (giving it 6 from its own wealth; the offspring keeps the strategy), and
dies when its wealth goes negative. After every cycle N/2 random pairs of agents swap places in the
list. Cooperators find each other, clone into zones of cooperation, and dominate: the published tables
give 779 cooperators and 121 defectors after 500 cycles (Table 9.2, Run 1), and 784 and 99 with a
maximum age of 100 (Table 9.4, Run 2).

**The working paper and the published text disagree** on the rule, and the sources leave much open;
each choice is a switch or a preset. The 1997 working paper moves to "a random site within your
vision" and plays "a random neighbor", once; the 1998 article and *Generative Social Science* (2006)
move to an unoccupied site and play each neighbor (`play`, `dpd-working-paper`). The prose never gives
the initial agents' wealth or states the cloning threshold as a number to reach: the defaults take
the 2006 CD's `Initial Wealth = 6` and `Fission Wealth = 11` ("exceeds 10"). Radax and Rengs (2009)
list six more choices the text leaves open and the defaults read literally: death as wealth goes
negative, even on another agent's turn (`death_timing`), removal at once (`removal`), the endowment
taken from the parent (`endowment_from`), every newborn's age random up to the maximum (`newborn_age`),
asynchronous updating (`updating`) and Epstein's swaps (`shuffle`). No source says whether a newborn
acts in the cycle it is born (`newborns_act`: next cycle), or whether metabolism, "a fixed decrement to
accumulated payoff per cycle", is charged per cycle or, as another sentence says, "after every
interaction" (`metabolism_per`: per cycle). Soup (`pairing: soup`) pairs each agent with a random
other agent and places moves and offspring anywhere.

What reproduces, measured (release, seeds 1–30, Epstein's 30 runs; each run's count at cycle 500
unless noted; t is Radax and Rengs' two-sample test against the source's mean and s.d., |t| < 2.0017
to pass):

- **Cooperation dominates.** Run 1: 729 ± 17 cooperators against 171 ± 17 defectors; Run 2: 695 ± 29
  against 196 ± 28. The ratio stabilizes from about cycle 30, as Epstein says.
- **Soup runs to pure defection** (`dpd-soup`): the last cooperator dies by cycle 8 on average (4–14)
  in 29 of 30 runs; the defectors then kill one another, leaving one agent or nobody.
- **Run 5's cooperation persists through 10,000 cycles** (`dpd-run-5`, 50 % mutation) in 27 of 30
  runs (the other three populations die out entirely): 260 cooperators against 295 defectors over
  cycles 5,001–10,000. At 25 % mutation the means over cycles 1,001–2,000 are 420 and 393 (Epstein:
  "around 350 and … around 400").
- **Table 9.3's pattern**: cooperators dominate when R is near T and die out as R falls, at the same R
  in every row (R ≤ 3 at T = 10, R ≤ 2 at T = 7–9, R = 1 at T ≤ 6).
- **The metabolism equivalence, charged per game.** The payoffs shifted up by 6 with a metabolism of 6
  charged after every game are Run 2 exactly, run for run (`metabolism_per: interaction`); more
  metabolism does mean more cooperators, from 414 at 1 to 661 at 5 per cycle.

What does not, or only partly:

- **Tables 1 and 2.** Both counts are rejected in both runs: 729 / 171 against 779 / 121 (t = 11.9 and
  −11.8), and 695 / 196 against 784 / 99 (t = 11.8 and −14.1) — 50–90 fewer cooperators and 50–100
  more defectors than Epstein's. By cycle 50 the ratio is 4.3 to 1, not "approximately 5 to 1".
- **Radax and Rengs' factorial**, repeated over the six switches (their random-number library
  excluded): no setting reproduces Run 1 and one of 64 reproduces Run 2 (removal at once, death on the
  agent's own turn, a full shuffle: 785 / 110), none both. They found 1 of 128 for Run 1 (a
  synchronous setting they set aside) and 7 for Run 2; none of their seven fits here, and their best
  (`dpd-rr-best`: 780 / 97 in Repast) gives 703 / 164. The same switches in two implementations give
  different models; their pseudo-code fixes details the text does not (neighbors played in random
  order, deaths checked after all games, "age ≥ maximum").
- **What does reproduce both tables** is three choices no source settles: the working paper's one
  random neighbor a turn, initial agents with no wealth, and newborns acting at once (`dpd-closest`):
  786 / 114 (t = −1.5) and 792 / 101 (t = −1.2). Of 512 combinations of those three choices with the six
  switches, 46 reproduce Run 1 — every one with no initial wealth — 24 Run 2 and 7 both, all 7 with the
  working paper's rule. It is fragile: over seeds 31–60 and 61–90 Run 2's defectors fail (t = −2.6,
  −3.2). The working paper's rule alone (`dpd-working-paper`) is nearer Table 1 but still rejected:
  759 / 141 (t = 4.8).
- **Table 9.3 cell by cell**: of the 90 means (45 payoff vectors, cooperators and defectors) 22 fall
  inside Epstein's 95 % confidence intervals and 51 inside his ranges; both means are inside the
  intervals in 1 cell of 45, and 5 cells pass both t-tests. Along the diagonal (R = T − 1) our
  cooperators run 13–92 below his and our defectors 33–106 above (bar T = 2). Where his populations die
  out entirely (ranges (0, 0)), 1–5 lone defectors survive here: once the cooperators are gone they have
  nobody to play, and with no maximum age and no metabolism nothing kills them. Row (4, 2)'s defector
  interval, "(254, 376)", is a misprint: its mean 265 and s.d. 32 give (253, 277).
- **Run 4 (R = 1) dies out** instead of cycling (`dpd-run-4`): 26 of 30 populations are extinct by
  cycle 500 and all 30 by 2,000, after at most two swings of the cooperators (above 400, then below
  100: 0.6 a run). Epstein's figure shows a cycle every 300–500 cycles. There is no cooperator monopoly
  at R = 1 (nor at R = 5, where all 30 coexist), so the paradox that "cooperators ultimately do better
  with a low payoff (R = 1) than with a high one (R = 5)" has nothing to stand on. No timing setting
  gives sustained cycles.
- **The shifted payoffs (12, 11, 1, 0) never converge to pure defection** (`dpd-shifted`, Fig. 13):
  all 30 runs coexist at 500 and 2,000 (418 cooperators, 478 defectors). GSS gives the settings —
  "maximum age of 100, zero mutation" — and Run 5's 50 % mutation or Run 1's unlimited lives do not
  converge either. With no negative payoff only old age kills, the lattice stays full and everyone can
  afford to clone. So the premise of the metabolism argument ("in which cooperators are annihilated")
  fails, and charged per cycle, as the chapter's note defines it, a metabolism of 6 gives another model
  (`dpd-metabolism`: 644 ± 97 cooperators, 253 ± 97 defectors against Run 2's 695 ± 29, t = 2.7). The
  passage calls the pure-defection run "figure 12", which is Run 5's; it means Fig. 13.
- **Footnote 27** (T 16, R 11, P 5, S 4, maximum lifetime 10: "an evolution to cooperative monopoly")
  gives a monopoly in 1 of 30 runs by cycle 2,000 and none by 500 (`dpd-footnote-27`); "hiked by ten"
  would make R 15, which gives the same.
- **The coordination game's norm maps** (GSS appendix, `dpd-coordination`: payoffs [1, −3, −3, 1],
  death age 1,000) form where both conventions persist — at cycle 500, 3.6 % of neighboring pairs
  differ, against 35 % if mixed at random — but both persist in only 17 of 30 runs at cycle 500, 16 at
  2,000 and 11 at 5,000.

The survey measures 21 of these claims: 6 hold, 4 are weak and 11 fail. Four built-in sweeps (30
seeds, cooperators at cycle 500): `dpd-payoffs` (Table 9.3: T 2–10 as lines, R 1–9 on the x axis; the
sweep format cannot skip the 36 cells with R ≥ T, which are not Prisoner's Dilemmas and fill with
774–876 cooperators), `dpd-mutation` (Run 2 with mutation 0–50 %: 695 down to 268), `dpd-metabolism`
(the shifted payoffs with metabolism 0–6 per cycle and per game: 418 at none, 644 and 695 at 6) and
`dpd-max-age` (maximum age 10–1,000: 666–735, barely mattering).

Agents are drawn by **Strategy** (the default: cooperators blue, defectors red, as Epstein's),
**Wealth** and **Age** (heat) or **Surrounded** (cooperators all eight of whose neighbors cooperate, in
blue; everyone else dimmed), with empty sites dark. Inspect shows an agent's strategy, wealth, age and
maximum age, whether it is surrounded, this cycle's payoffs and games, and each neighbor with what one
game between them pays each; an empty site says so. Charts: **Population** (cooperators and
defectors), **Cooperator share**, **Surrounded cooperators**, **Wealth** (the mean of each strategy)
and **Births and deaths**, against the cycle. A run never stops by default (`end`: a last cycle, 0
for never); the payoffs, the demography, the timing switches and the pairing apply to the running
world. **Compare** entries: "Working paper vs published rule — Demographic PD (Compare)" (`dpd-run-1`
and `dpd-working-paper`), "Negative payoffs vs shifted with metabolism — Demographic PD (Compare)"
(`dpd-run-2` and `dpd-metabolism`), "Space vs soup — Demographic PD (Compare)" (`dpd-run-1` and
`dpd-soup`) and "Published rule vs closest reading — Demographic PD (Compare)" (`dpd-run-1` and
`dpd-closest`). Credit: Joshua M. Epstein, "Zones of Cooperation in Demographic Prisoner's Dilemma,"
Santa Fe Institute Working Paper 97-12-094 (1997), *Complexity* 4(2) (1998), 36–48, and *Generative
Social Science* (Princeton, 2006), chapter 9 and its appendix; and Andreas Radax and Bernhard Rengs,
"Replication of the Demographic Prisoner's Dilemma," MPRA 14419 (2009), published as "Prospects and
Pitfalls of Statistical Testing: Insights from Replicating the Demographic Prisoner's Dilemma,"
*JASSS* 13(4) 1 (2010). See `docs/superpowers/specs/2026-09-26-demographic-pd-design.md`.
```

- [ ] **Step 3: Roadmap**

After Milestone 16:

```markdown
## Milestone 17: The demographic Prisoner's Dilemma (done)

Epstein's demographic Prisoner's Dilemma (1998) as an eleventh model kind, with the working paper's rule,
Radax and Rengs' six unstated timing choices, soup, metabolism and the coordination game as switches and
presets. Cooperation dominates and soup runs to pure defection, as Epstein says, but Tables 1 and 2 are
rejected (729 / 171 cooperators and defectors against 779 / 121) under every timing setting but one of 64
for Run 2; they reproduce together only with the working paper's rule, founders with no wealth and newborns
acting at once, none of which the published text says. Run 4 dies out instead of cycling, the shifted
payoffs never converge to pure defection, footnote 27's monopoly comes in 1 run of 30, and the metabolism
"equivalence" is exact only when metabolism is charged per game. See
`docs/superpowers/specs/2026-09-26-demographic-pd-design.md`.
```

and under **Experiments and science**, after the Hammond–Axelrod line:

```markdown
- **Epstein's demographic Prisoner's Dilemma** (and Radax & Rengs' replication): done (Milestone 17).
```

- [ ] **Step 4: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo +1.98.1 clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test dpd -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && rustfmt --edition 2021 --check src/claims/dpd.rs && cargo test)
```

Expected: all PASS (workspace: 589 core unit tests among the rest; wasm-pack: 38; web: 45 files, 579 tests; survey: 20 tests). Browser (controller): every dpd preset, the four colour modes, Inspect (agents, neighbours, empty sites, an agent that dies), the Rules panel's groups and hidden Play, the four Compare entries, the four sweeps, keyframes and the timeline on `dpd-rr-best`, share links and sessions with a live payoff change, recording, Max speed, and every existing scenario.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/roadmap.md docs/superpowers/specs/2026-09-26-demographic-pd-design.md
git commit -m "Document the demographic PD and mark milestone 17 done" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

## Self-review (planning)

- **Spec coverage:** Architecture, Config, Rules, Statistics — Task 1 (Decisions 1–20); the choices the sources leave open — Task 1's code and Decisions, Task 6's spec notes; Views — frames and Inspect JSON in Task 1, colour modes, charts and Inspect rows in Task 3; Presets and Compare — Tasks 1 and 4; Experiments and CLI — Task 2 and Task 4's `defaultForm`; Claims — Task 2's book-style tests and Task 5's survey; Page — Tasks 3–4; Testing — every task; Docs — Task 6.
- **Placeholders:** none; every code block is the planning dry runs' code, which passed fmt, clippy (stable and 1.98.1), the workspace tests, the ignored dpd tests, wasm-pack, the web build and tests, and the survey's tests (the Review Focus tests were added to the dry run and pass).
- **Review Focus:** each item names its test and task.
- **Caveat:** only the final state of each part was run; the RED expectations of intermediate steps are stated, not observed.

- **Spec coverage:** Views — colour modes (`COLOR_MODES.dpd`), Inspect (Task 3), charts (Task 3; no phase diagram, Decision 22); Page — the presets group (`MODELS`/`MODEL_LABELS`), the schema panel's groups and `show_if` (the core's schema, tested in Tasks 3–4), Compare entries and `defaultForm` (Task 4); Testing — Vitest for schema fields, charts, Inspect rows, Compare entries, `defaultForm`, determinism fingerprints (Tasks 3–4); Claims to test — every measured item has a survey claim (Task 5; Run 3 has no claim, since Epstein gives no number); Docs (Task 6).
- **Placeholders:** none; every block is `dpd-web`'s code.
- **Type consistency:** `DpdInspection` / `DpdAgentView` / `DpdNeighborView` mirror the Rust `DpdInspection` / `AgentView` / `NeighborView` field for field (`strategy` "C" | "D", `max_age`, `surrounded`, `income`, `games`, `payoff`, `their_payoff`); `DpdStats` mirrors `DpdSnapshot` (NaN → null for `cooperator_share`, `wealth_c`, `wealth_d`); `DpdConfig` mirrors the Rust config and its snake_case enums.
- **Caveat:** only the final state was run; the RED expectations are stated, not observed.
