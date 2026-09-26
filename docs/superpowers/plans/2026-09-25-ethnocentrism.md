# Ethnocentrism Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Hammond and Axelrod's evolution of ethnocentrism (2006) as a model kind, `ethno`, with its appendix's and archived code's departures and the variants of Hartshorn, Kaznatcheev & Shultz (2013) and Jansson (2013) as named switches, presets, sweeps and survey claims, and every source's claims measured.

**Architecture:** A new module `crates/sugarscape-core/src/ethno/` implementing the `Model` trait on the spatial games' periodic von Neumann lattice (shared by `Arc`), wired into `ModelKind`/`ModelConfig`/`ModelWorld` with keyframes, presets and golden entries; book-style tests and seven sweeps measure it against the sources; the page gains its types, colour modes, charts, Inspect, Rules-panel support (a nullable number, `show_if` on a boolean), Compare entries and Experiments default; the survey gains 38 claims.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm via wasm-pack, sugarscape-cli, the standalone survey crate), TypeScript (Vite, uPlot, Vitest).

**Spec:** `docs/superpowers/specs/2026-09-25-ethnocentrism-design.md`

## Global Constraints

- **Earlier models unchanged:** every golden entry and legacy fixture stays green and unedited; every existing config, link, session file and sweep reads as before.
- **The default is HA06's text**, not its appendix or its code; each disagreement is a named switch or preset.
- **Deterministic and portable:** a function of (config, seed); all draws from the world's seeded `SimRng` with `u32` ranges; no platform transcendental functions; fingerprints identical native and WASM.
- **Truthful descriptions,** with measurements, including what does not reproduce; numbers only from Decision 12 (`measurements`), never invented.
- **Verify with:** `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings` (CI runs the newest stable clippy: prefer `as_chunks` over `chunks_exact` with constant sizes), `cargo test --workspace`, `cargo test -p sugarscape-core --release --test ethno -- --ignored`, `wasm-pack test --node crates/sugarscape-wasm`, `(cd web && npm run build && npm test)`, `(cd survey && cargo test)`.
- **Do not run `cargo fmt` inside `survey/`** (it would rewrite unrelated chapters; the survey is not a workspace member).
- **Commits:** stage only the files the task names (never `-A`/`.`, never `.claude/`, `.superpowers/`, `web/src/wasm-pkg`); messages are plain imperative sentences and end with a second `-m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"`.
- **Web:** TypeScript strict with `noUnusedLocals`/`noUnusedParameters`; build DOM with `h()`; typographic apostrophes (’) in test names.
- **If a measured number differs from Decision 12, stop and report; do not retune thresholds.**

## Review Focus

1. **A world that empties** (`death: 1`, or immigration 0 after a die-off): statistics become NaN (null on the page), frames draw every mode, Inspect says the site is empty, nothing panics — Task 1, `a_world_that_empties_reports_nan_and_still_draws`.
2. **Extreme colour counts in every discrimination mode** (1, 2 and 40 colours × `same_other`, `none`, `each_color`): runs stay valid, tags stay in range, help bits never exceed the mode's width — Task 1, `extreme_colour_counts_run_in_every_discrimination`.
3. **PTR outside [0, 1]** (the Rules panel accepts any cost, benefit and base PTR ≥ 0): negative or zero PTR never reproduces, PTR above 1 always does — Task 1, `ptr_outside_zero_to_one_only_saturates_the_chance`.
4. **The empty-site list after long runs, live switches and keyframes** (it drives immigration and `anywhere` placement, so a stale entry would place two agents on a site): it matches the lattice exactly, and a keyframe replays identically — Task 1, `the_empty_list_survives_long_runs_live_changes_and_keyframes`.
5. **`allowed` through the page** (not on the panel, so a live edit or reset could silently drop it): it survives live edits, resets, links and sessions — Task 3's `hks-no-ethnocentrics` round-trip test (Decision 26).

## Why this task order

The core model (Task 1) carries everything later tasks read: the config and schema, the statistics, the inspection JSON and the golden fingerprints. Task 2 measures it and adds the sweeps, which need only the core. The page (Tasks 3–4) needs the core's JSON; the survey (Task 5) needs the presets and the measured tolerances; the docs (Task 6) quote everything.

## Decisions (where the spec leaves room)

Binding. Every rule below was implemented in a scratch copy during planning and measured; the Rust code in Tasks 1–2 is that code and passes `cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --workspace`, the ignored ethno tests in release and `wasm-pack test --node crates/sugarscape-wasm`.

1. **Module layout.** `crates/sugarscape-core/src/ethno/` with `config.rs`, `stats.rs`, `world.rs`, `presets.rs`, `mod.rs`; `pub mod ethno;` after `pub mod edit;` in `lib.rs`. `ModelKind::Ethno` is last (`ALL: [ModelKind; 8]`); the ethno presets end the catalog (after the spatial games').
2. **Lattice.** `spatial::Geometry::new` on a `SpatialConfig { width, height: width, neighborhood: VonNeumann, boundary: Periodic, .. }` (a square draws nothing; it is handed a throwaway `rng::seeded(0)` so the world's stream is untouched), shared by `Arc`. Sites are numbered `x + y·w`; a site's neighbors come in the lattice's offset order — up, left, right, down — and the neighbor in direction `d` sees the site in direction `3 − d`. Width 3–200 (3 keeps the four neighbors distinct).
3. **Agents.** `Agent { id, tag, help: u64, kin_basis, lineage, family, born, ptr, given, received, gave: [u8; 4] }`. Help bits: `same_other` bit 0 = help same (or kin), bit 1 = help other; `none` bit 0 = help everyone; `each_color` bit k = help colour k. Ids come from one counter starting at 1; an immigrant's `lineage` and `family` are its own id; an offspring inherits `lineage`, and `family` unless the kin marker mutates (then its own id). The kin marker mutates only with `kin_strategies` (it is otherwise inert). `born` is the period of arrival or birth (0 for a full start); age = tick − born.
4. **Empty sites.** An unordered list of empty sites with each site's index (swap-remove). "A uniformly chosen empty site" is `empty[gen_range(0..len as u32)]`; the list's order depends on history but is deterministic.
5. **Draw order (the RNG contract).** Immigration: one `gen::<f64>()` only when `immigration` has a fractional part; per immigrant (none when the lattice is full): the site, then `gen_range(0..colors)` for the tag, then `gen::<u64>() & mask(bits)` redrawn until allowed, then (kin strategies only) `gen::<bool>()` for the basis. Interaction: one `gen::<f64>()` per decision only when `discrimination` is `same_other` and `misperception > 0`. Reproduction: the occupied sites ascending, shuffled (`SliceRandom::shuffle`); per agent one `gen::<f64>()` (always, even with no room — the Java also draws first), then the target (`gen_range` over the empty neighbors in direction order, or over the empty list); the offspring's draws: each strategy bit in order (always drawn), the basis bit (kin strategies with `kin_basis: mutates` only), the tag (always drawn; a mutated tag is `gen_range(0..colors − 1)` skipping the parent's, so uniform over the others; none with one colour), the kin marker (kin only). Death: one `gen::<f64>()` per occupied site in site order.
6. **Allowed strategies.** Immigrants and full `random` starts redraw both bits until the strategy is allowed (HKS13's "aborted"); an offspring's bit flip that yields a disallowed strategy is ignored, bits in order (same, then other). Only with `same_other` and no kin strategies (validation).
7. **Offspring and Inspect.** Offspring get PTR = `base_ptr`, zero helps (they did not interact); when one is placed, its neighbors' `gave` toward that site is cleared so Inspect never credits the newborn with help it did not receive.
8. **Full starts.** `random`: every site in order gets a newcomer (founding its own lineage and family, `born` 0); `selfish`: the same draws, then help bits 0 and basis tag. Validation: a selfish start needs S among the allowed strategies (field `start`).
9. **Classification.** `same_other`: the two bits give E/H/S/T; with a kin basis E becomes `kin` and T `nonkin` (H and S are counted once, J13's "all" and "none"). `none`: H or S. `each_color`, in order: no bits S, all bits H, own only E, all but own T, else `mixed` (one colour: all = own gives H).
10. **Statistics.** Shares (of the 7 strategies) are of the population at the period's end, after death; `cooperation`, `same_tag`, `relatives`, `kin_help`, `tag_given_relative`, `relative_given_tag` come from this period's interaction step. Pairs are counted ordered (each neighboring pair twice; ratios unchanged), once per pair even with `twice`; "related" = same lineage. Zero denominators give NaN (JSON null); an empty lattice's shares are NaN. The t = 0 snapshot has NaN interaction statistics. HA-Java pools the last 100 periods' counts before dividing; the window mean of per-period ratios differs negligibly (Decision 12).
11. **Validation** (on the named field): `width` 3–200; `colors` 1–40; `immigration` 0–100; `base_ptr`, `cost`, `benefit` finite and ≥ 0; `death`, `mutation`, `tag_mutation`, `misperception`, `kin_mutation` in [0, 1]; `each_color` with kin strategies → `kin_strategies`, with misperception > 0 → `misperception`; `allowed` empty, holding anything but E/H/S/T, or restricted without `same_other` or with kin strategies → `allowed`; schedules as milestone 11's (live paths only, values that validate).
12. **Measured results** — "Decision 12: the measurements" below (every number, method and seed set). Re-measuring: `cargo test -p sugarscape-core --release --test ethno -- --ignored --nocapture` and `cargo run --release -p sugarscape-cli -- sweep --builtin <id> --quiet --summary-csv /dev/stdout --out /dev/null`. If an implementation following this plan gives different numbers, stop and report rather than retune.
13. **Golden entries** (200 ticks, seed 1, `MODEL_GOLDEN`, after `rca-adopt-p1`): `ha-standard` 0xf07433e56417f07c, `ha-figure-1` 0x843632b62ddf7a6b, `ha-appendix-mutation` 0xae8c7eda9113dae8, `ha-appendix-double-play` 0xac2c2167fec9c326, `ha-java-five-colors` 0xdc78c1e27b9ab453, `ha-java-archive` 0xde2cff652c758fe7, `ha-egoist-start` 0xabfdf5c9e1ccdb45, `ha-cost-2` 0x41ba53998a8ee613, `ha-cost-2-blind` 0x699aa05497139005, `ha-misperception` 0x5567187174fd1c15, `ha-each-color` 0x9ad570c3ea183419, `jansson-offspring-anywhere` 0xcad22f8e7abafbfe, `jansson-tag-mutation-30` 0xf9dbf338238a8f1b, `jansson-kin` 0x265998639eacfbd0, `jansson-kin-fixed` 0x3fac090571612879, `hks-no-ethnocentrics` 0xbe867e7210bad2d2. `wasm-pack test` reproduces `ha-standard`, `ha-misperception` and `jansson-kin` (checked during planning). Keyframes: `tests/checkpoint.rs` adds `jansson-kin` (kin markers, basis bits and the empty list restore exactly).
14. **Live and reset fields.** Live: `immigration`, `base_ptr`, `cost`, `benefit`, `death`, `mutation`, `tag_mutation`, `pair_play`, `misperception`, `offspring`, `kin_mutation`, `end`. Reset: `width`, `colors`, `start`, `discrimination`, `allowed`, `kin_strategies`, `kin_basis`, `schedule`. `end` (default 2,000; 0 never) stops `run` like the tags model's (`max_ticks`, `finished`); the sweep error says "the ethnocentrism model stops at its last period, N in this config" and the CLI "finished at tick N (its last period)".
15. **Schema.** Groups **Game** (cost, benefit, base PTR, pair play), **Population** (width, start, immigration, death, offspring), **Traits** (colours, discrimination, misperception `shown_if discrimination = same_other`, kin strategies, kin basis and kin mutation both `shown_if kin_strategies = "true"`), **Mutation** (mutation, tag mutation), **Run** (end). `allowed` is not on the panel (a list is not a panel kind; presets, files and links set it — as the tags model's `initial_tolerance`), so the spec's "allowed shown only for same_other without kin" has no field to hide. Two web consequences for the page task: `tag_mutation` is `null` by default (the panel must show null as empty and send a number or null), and `paramShown` must compare `String(value)` so `kin_strategies: true` matches `"true"` (today it compares `===` to a string).
16. **Frames.** One pixel a site (`size()` = width × width); empty sites `BACKGROUND`. Strategy: E `LENDER` (green), H `BLUE`, S `RED`, T `BOTH` (yellow), kin `POLLUTION` (purple), nonkin `SPICE` (orange), mixed `NEUTRAL`. Tag: tags 0–3 in the Java's blue, red, green, yellow, then hues a golden angle apart (40 distinct). Lineage: splitmix64 of the lineage id, each channel in 64–255. PTR: `lerp(COOL, HOT, (ptr − lo)/(hi − lo))` with lo/hi = base ∓ 4·cost/benefit (8 with `twice`).
17. **Inspect, CSV, fingerprint.** Inspect JSON `{ site: {x, y}, agent: { id, tag, strategy ("E" … "mixed"), basis ("tag" | "kin"), ptr, given, received, lineage, kin_marker, age, neighbors: [{x, y, tag, strategy, related, helped, helped_by}] (occupied only, up/left/right/down) } | null }`. `locate(id)` scans the sites. Agents CSV `id,x,y,tag,strategy,basis,lineage,kin_marker,age,ptr,given,received`. Fingerprint: FNV-1a over the tick, the id counter, and each site (empty: `u64::MAX`; else id, help, tag | basis << 32, lineage, family, born, PTR bits, given << 32 | received).
18. **Presets** (the spec's table, in its order, plus `jansson-kin-fixed` after `jansson-kin`; source strings `Hammond & Axelrod 2006, J. Conflict Resolution 50`, `Hammond & Axelrod 2006, appendix`, `Hammond & Axelrod's Java code (2003)`, `Hartshorn, Kaznatcheev & Shultz 2013, JASSS 16(3)`, `Jansson 2013, JASSS 16(3)`); descriptions carry Decision 12's numbers.
19. **Sweeps** (after `rca-population`, in the spec's order): ten seeds, 2,000 periods, `window_mean` from 1,901. `ha-cost` x 0.005–0.03 by 0.005, series seeing/blind, metric `cooperation`; `ha-colors` x 2–9; `ha-mutation` x 0.0025, 0.005, 0.01, 0.02, 0.05, series once/twice; `ha-immigration` x 0.5, 1, 1.5, 2; `ha-lattice` x 25, 50, 75, 100; `jansson-tag-mutation` x 0.005, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.75, 0.9; `jansson-markers` base `jansson-kin`, x 4–40 by 4, series `kin_basis` mutates/fixed, metric `kin`.
20. **Book-style tests** pin each measured seed mean to ±0.06 points (the runs are deterministic and descriptions quote one decimal) and state the source's number beside it; claims that fail are pinned as measured, with the failure in the comment.
21. **HKS13's dominance** (Study 3 gives only the tests): at each cycle a strategy dominates when the 3-df chi-square against a uniform split exceeds 11.345 and the 1-df test of the leader against the runner-up, (a − b)²/(a + b), exceeds 6.635 (both p < .01), on counts = round(share × population). **Onset** of ethnocentric dominance: the first cycle from which E dominates 100 cycles running. **Early patterns** (cycles 1–300): humanitarian if H dominates 50 cycles running and in more cycles than E; ethnocentric if E dominates at least 150 cycles and more than H; else strong competition. (HKS13's World 4, "strong competition", had a 24-cycle H spell, so a short spell must not count; 50 is our line.)
22. **Kin basis inheritance** (J13 does not say; §2.5's "a probability of 0.005 per trait" is the only hint, and §5.2 says only that "agents are either group or kin discriminators"): `kin_basis: "mutates"` (default — the basis is one more trait mutating at `mutation`) or `"fixed"` (drawn at immigration, never mutates: no draw). Reset-only. Fixed matches Table 5 better (kin 65.5 vs 76.2, ingroup 12.6 vs 16.4; mutating 52.1 / 26.7) and its markers gap closes later (near 40), but nothing in the sources settles it, so the default stays `mutates`; preset `jansson-kin-fixed` and the `jansson-markers` series show both.
23. **HA06's colour-blind 14%** has no switch: no reading reproduces the 56%/14% pair (measurements.md). `discrimination: none` stays the blind reading.

24. **A nullable number on the Rules panel.** The core's `schema::Param` gains `nullable: bool` (serialized only when true) and a `.nullable()` builder; the ethno schema marks `tag_mutation` (null = "the mutation rate"). The page shows a nullable field's null as an empty box and writes an empty box as null (`paramInput`, `paramEdit`); every other number field still reads an empty box as a number, as before, so no other model changes. (A page-only rule — "an empty box is null" for every field — would turn an emptied box of another model into a whole-config deserialization error instead of 0; the flag keeps the change to the one field that needs it.) `check_schema` is unchanged: the ethno schema test already starts from `tag_mutation: Some(0.005)`.
25. **`show_if` on a bool.** `paramShown` compares `String(value)` with `equals`, so `kin_strategies: true` matches the core's `"true"` (Decision 15); string fields compare as before. The Rust doc comments on `ShowIf` and `shown_if` say so.
26. **`allowed` on the page.** Not on the panel (Decision 15); `EthnoConfig.allowed` is typed and carried untouched: a Vitest drives `hks-no-ethnocentrics` through a live edit, a reset-requiring edit, a live `tag_mutation` set and cleared, and a share link, and checks `allowed` stays `["H", "S", "T"]` and the replay's fingerprint matches.
27. **Telling an ethno inspection apart.** An empty ethno site (`{ site: { x, y }, agent: null }`) is exactly an empty Schelling site, so no shape test can separate them: `isEthnoView(v, model)` takes the world's model (`model === 'ethno'` and the agent, if any, names a `kin_marker`, which no other model's agent has; Schelling's agent has a numeric `neighbors`). It is the first test in the Inspect panel's chain. An ethno agent that died reads "Agent #N has died." Empty sites say "none (an empty site)".
28. **Charts.** Strategies uses the frame's palette through the page's CSS variables (E `--lender` green, H `--blue`, S `--red`, T `--both` yellow, kin `--c4` purple, non-kin `--c2` orange, mixed `--muted` gray; all seven lines always, 0 where a strategy cannot occur). The Kin chart adds `relatives` to the spec's three (J13 Table 4's first number; otherwise no chart shows it). Every ethno chart's y range is [0, 1] except Population. The time axis is **Period** (HA06's word throughout; HKS13 say "cycle").
29. **Inspect rows** (`ethnoRows` in `web/src/ethno.ts`): Agent (id, tag), Strategy (name and what it does), Judges by (only with kin strategies), PTR (3 decimals), Helps (given, received), Lineage and Kin marker (noting when the agent founded it), Age (periods), then one row per occupied neighbor, keyed by its coordinates: tag, strategy, related or not, and who helped whom ("helped it twice" under `pair_play: twice`). The page's own strings say "color" (as the rest of the page); the core's schema labels keep "colour".
30. **Experiments' default form** for ethno: x = `cost` over `0.005:0.03:0.0025` (eleven values), 2,000 ticks, `window_mean` of `ethnocentric` from 1,901 to the end, and the form's usual three seeds (33 runs; the built-in `ha-cost` runs ten).
31. **The end.** `ticksLeft` counts down to `end` (0 never) and `finishedNotice` says "This run has reached its last period (N) — Reset to run it again".
32. **Survey.** `survey/src/claims/ethno.rs`, 38 claims (Task 5). Tolerances are per world, two per-world standard deviations of the measured spread (s.e. over ten seeds × √10, from Decision 12): Table 1 ±7 points ethnocentric and ±3 cooperative; HKS13's shares ±4 (S), ±1.4 (T), ±10 (E, H); J13 Table 4 ±3.8, ±2, ±4.4 and kin help ±4.4; Table 5 ±14. A Table 1 row is one claim judged on both columns, the worse verdict winning (`all_of`, moved from `ch6.rs` to `claim.rs` and made public, unchanged). HKS13 Studies 1 and 3 run their own 50 worlds (seeds 1–50, 1,000 periods, once per process); Study 3's early-pattern counts use a local count judge (each count within two binomial s.d. of the source's holds, within three is weak) — the existing judges compare per-world values, not counts. The survey's default 20 seeds: 15 hold, 11 weak, 12 fail (`survey-results.md`); most Table 1 rows are weak because cooperation runs ~2 points high.
33. **Spec corrections** (Task 6): the model is the eighth kind, not the seventh (tags and the spatial games came first); `kin_basis` and `jansson-kin-fixed` join the config table, choice 12 and the presets; `jansson-markers` has the `kin_basis` series; the Page section drops "allowed shown only …" and states Decisions 24–26 and 30; the Kin chart names `relatives`.

## Decision 12: the measurements (planning dry run, recorded 2026-09-25)

**Method.** Release builds of the scratch copy (`eth-dry`). Unless noted: seeds 1–10, 2,000 periods, each run summarized by the mean of the per-period series over periods 1,901–2,000 (NaN periods skipped), then mean ± s.e. over seeds. Percentages. The numbers come from two independent paths that agree to the decimal: a scratch harness (`measure/src/main.rs`, output in `measure/out-*.txt`) and the committed tests (`crates/sugarscape-core/tests/ethno.rs`, `--ignored --nocapture`) and CLI sweeps (`sweep-out/*.csv`). Performance: 50 × 50 for 2,000 periods 0.23 s, 100 × 100 0.9 s (one thread); each sweep 1–6 s on 10 threads; the 17 ignored tests 21–27 s.

### HA06 Table 1 (E = ethnocentric share, C = cooperation = helps ÷ decisions)

| Row | Setting | HA06 E / C | Ours, 4 colours (e: 8) | Ours, 5 colours (e: 9) |
|---|---|---|---|---|
| a | standard | 76.3 / 74.2 | 75.9 ± 1.1 / 76.0 ± 0.5 | 75.4 ± 1.6 / 76.1 ± 0.5 |
| b | cost 0.5% | 76.0 / 77.8 | 76.0 ± 1.3 / 78.5 ± 0.6 | 73.9 ± 1.3 / 79.7 ± 0.8 |
| c | cost 2% | 61.8 / 56.1 | 63.5 ± 2.0 / **64.7 ± 1.1** | 68.6 ± 1.1 / 67.3 ± 1.3 |
| d | 2 colours | 69.4 / 78.1 | 68.9 ± 1.4 / 78.9 ± 0.7 | (same) |
| e | 8 colours | 79.1 / 71.7 | 77.9 ± 1.4 / 74.4 ± 0.4 | 81.8 ± 1.2 / 73.7 ± 0.7 |
| f | mutation 0.25% | 82.8 / 79.8 | 83.4 ± 1.0 / 80.2 ± 0.4 | 84.5 ± 1.3 / 79.3 ± 0.9 |
| g | mutation 1% | 67.1 / 69.0 | **63.0 ± 0.9** / 70.8 ± 0.6 | 64.0 ± 0.7 / 71.5 ± 0.7 |
| h | immigration 0.5 | 77.5 / 75.5 | 77.9 ± 1.2 / 77.8 ± 0.6 | 77.5 ± 1.6 / 75.7 ± 0.7 |
| i | immigration 2 | 74.4 / 71.4 | **70.5 ± 1.2** / 73.9 ± 0.6 | 74.5 ± 0.8 / 72.2 ± 0.6 |
| j | 25 × 25 | 70.5 / 69.9 | **64.4 ± 2.3** / 70.1 ± 1.2 | 68.3 ± 2.1 / 66.9 ± 1.4 |
| k | 100 × 100 | 78.2 / 76.0 | 76.5 ± 0.8 / 79.1 ± 0.2 | 77.6 ± 0.7 / 78.3 ± 0.3 |
| l | 500 periods (401–500) | 73.9 / 73.4 | **57.3 ± 1.9** / 78.6 ± 0.6 | 55.3 ± 3.2 / 80.4 ± 0.6 |
| m | 4,000 periods (3,901–4,000) | 77.3 / 74.4 | 77.1 ± 1.1 / 75.9 ± 0.5 | 73.8 ± 1.1 / 75.4 ± 0.6 |
| m′ | "2,000" read literally (= a) | 77.3 / 74.4 | 75.9 / 76.0 | 75.4 / 76.1 |

- Within 3 points of HA06 in both columns (4 colours): a, b, d, e, f, h, m (pinned by `table_1_as_measured`). Not reproduced: c (cooperation +8.6), g, i, j (ethnocentric −4 to −6), k (cooperation +3.1), l (ethnocentric −16.6).
- **4 vs 5 colours:** RMS error over all 13 rows — E 5.2 (4) vs 5.8 (5); C 3.3 vs 4.0. Without row l — E 2.5 vs 2.7; C 3.1 vs 3.7. Rows d/a/e only — E 0.8 vs 1.7; C 1.9 vs 1.7. The `ha-colors` sweep: 2: 68.9, 3: 76.2, 4: 75.9, 5: 75.4, 6: 76.1, 7: 78.6, 8: 77.9, 9: 81.8. **Verdict: Table 1 cannot tell four from five colours** (the share is flat from 3 to 6); 4/8 fits rows a/e marginally better (e: 77.9 vs 79.1 against 81.8), 5 fits a few cooperation cells marginally better. Neither reading is excluded at ten seeds.
- Row m: "run length 2,000" in a table whose standard is 2,000 is presumably 4,000 periods; at 4,000 we get 77.1 / 75.9 (HA06 77.3 / 74.4) — fits.
- Cooperation runs ~2 points above HA06 in most rows (mean +2.4). HA-Java pools the last 100 periods' counts before dividing; our window mean of per-period ratios differs from that by far less than 2 points, so this is not the definition.

### HA06 text, appendix and code

| Claim | Source | Ours |
|---|---|---|
| Appendix `MutationRate = 0.05` | 76.3 / 74.2 expected if it were Table 1 a | **36.0 ± 1.0 E**, 25.8 H, 22.1 S, 16.0 T; C 56.4 ± 0.4 — the appendix's 5% is not what ran |
| Appendix loop read literally (`twice`) | — | 80.9 ± 0.8 E, C 77.3 ± 0.7, population 1,857 (once: 75.9, 1,560) — five points above Table 1 a |
| `ha-mutation` sweep (once / twice) | — | 0.25%: 83.4 / 86.6; 0.5%: 75.9 / 80.9; 1%: 63.0 / 74.6; 2%: 52.4 / 62.9; 5%: 36.0 / 42.7 |
| Archived Java as it runs (5 colours, full random, no immigration) | — | 77.7 ± 1.4 E, C 77.6 ± 0.7; same outcome as the paper's start; 43.5% E by t = 100 (standard 35.3%), 59.6% by 200 |
| Archive with 4 colours | — | 80.1 ± 1.5 E, C 78.8 |
| Random full start with immigration 1 | — | 76.1 ± 1.4 E |
| Egoist start, no immigration: "just as dominant" | HA06 text | **78.6 ± 1.3 E**, C 79.9 — reproduced; E 7.2% at t = 100, 25.5% at 200, 69.5% at 500, 77.4% at 1,000 (population dips to 786 at t = 200) |
| Egoist start with immigration 1 | — | 76.8 ± 1.1 |
| Each colour: "80 percent ethnocentric strategies" | HA06 text | helps own colour only: **27.0 ± 2.3**; helps own and refuses ≥ 1 other: **84.3 ± 1.0**; helps own at all: 87.4; `mixed` 66.8. HA06's 80% matches the loose reading only. |
| Misperception 10%: "more than two-thirds" | HA06 text | 71.7 ± 1.3 E (C 71.1) — reproduced |
| Cost 2%: cooperation 56% seeing vs 14% blind | HA06 text | seeing **64.7 ± 1.1**, blind **41.8 ± 2.3** (40.4 H / 59.6 S) — the blind 14% is **not reproduced** |
| Other "blind" readings at cost 2% | — | one colour 40.2; misperception 0.5 44.9; blind with 5 colours 42.2; blind + `twice` 24.5; blind at cost 2.5% 19.8, at 3% 12.7 |
| `ha-cost` sweep, seeing / blind cooperation | — | 0.5%: 78.5 / 89.0; 1%: 76.0 / 81.2; 1.5%: 73.7 / 66.9; 2%: 64.7 / 41.8; 2.5%: 49.6 / 19.8; 3%: 29.8 / 12.7 |
| `ha-immigration` (E) | h 77.5, a 76.3, i 74.4 | 0.5: 77.9; 1: 75.9; 1.5: 73.4; 2: 70.5 |
| `ha-lattice` (E) | j 70.5, a 76.3, k 78.2 | 25: 64.4 (seeds 54–75); 50: 75.9; 75: 76.6; 100: 76.5 |
| Standard, 20 seeds | — | E 75.9 ± 0.8, C 75.9 ± 0.4 |
| E share over time (standard) | row l 73.9 at 500 | last-100 windows ending 300: 42.8; 500: 57.3; 750: 65.6; 1,000: 70.3; 1,500: 72.4; 2,000: 75.9 |

### HKS13 (50 seeds × 1,000 periods for Studies 1 and 3; Study 2: 10 seeds × 2,000)

- **Final shares** (901–1,000): selfish 7.7 ± 0.3, traitorous 2.6 ± 0.1, ethnocentric 72.4 ± 0.7, humanitarian 17.3 ± 0.7 (HKS13 .08 / .02 / .73 / .17) — **reproduced**. Population 1,568 ± 3 ("saturates just under 1,600").
- **Timing.** Mean population reaches 95% of its final level at period 303 (HKS13: ~300). Out-group share of interactions: 0.185 at 300, ~0.205–0.21 after (HKS13: "stagnates at just under .2"). The mean curves: E > H from period 85 on (E 0.48 / H 0.38 at 300, 0.60 / 0.29 at 500).
- **Period of ethnocentric dominance** (Decision 21: chi-square p < .01 both tests; first cycle from which E dominates 100 cycles running): all 50 worlds; mean 272 ± 23 (s.e.), **median 282**, range 21–596 — "around 300" holds for the median, but worlds vary enormously (11 of 50 settle before period 100, 11 after 400).
- **Early humanitarian dominance** (Decision 21 rule): **H-early 17, E-early 18, competition 15** of 50 (HKS13: 16 / 16 / 18) — reproduced. Sensitivity: worlds with an H-dominance spell before 300 of ≥ 10 / 20 / 50 / 100 cycles: 23 / 21 / 17 / 13. HKS13 Fig. 10's index (mean H − E share over cycles 1–200): −0.021 ± 0.030, positive in 24 of 50 (HKS13: normal, centred near 0) — consistent.
- **Study 2** (mean agents, last 100 of 2,000; ours vs Table 3):

| Allowed | Ours E / H / S / T | Table 3 | Order |
|---|---|---|---|
| EHST | 1184 / 211 / 127 / 38 | 1183 / 229 / 123 / 47 | E>H>S>T ✓ |
| HST | – / 1384 / 114 / 136 | – / 1368 / 115 / 150 | H>T>S ✓ (the reversal) |
| EHT | 1308 / 239 / – / 46 | 1334 / 247 / – / 39 | ✓ |
| EHS | 1144 / 289 / 135 / – | 1183 / 271 / 124 / – | ✓ |
| EST | 1344 / – / 157 / 38 | 1395 / – / 139 / 32 | ✓ |
| ST | – / – / 505 / 294 | 482 / 247 | ✓ |
| HT | – / 1478 / – / 189 | 1479 / 188 | ✓ |
| HS | – / 1519 / 133 / – | 1517 / 143 | ✓ |
| ET | 1558 / – / – / 38 | 1587 / 34 | ✓ |
| ES | 1411 / – / 141 / – | 1414 / 157 | ✓ |
| EH | 1338 / 275 / – / – | 1369 / 257 | ✓ |
| E, H, S, T alone | 1604, 1697, 657, 998 | 1615, 1707, 659, 966 | T > S, H > E ✓ |

All 15 orderings hold (`hks13_study_2_orders_hold_in_every_subset`); counts mostly within 5% (traitors in EHST 20% short).

### J13

- **Offspring anywhere** (J13 §3.6, "similar to the null model": 12% cooperators with one partner, 3.4% with four): cooperation **4.5 ± 0.1**, selfish 88.7, ethnocentric 8.3, traitorous 2.5, humanitarian 0.5; population 2,248 (20 seeds: identical to one decimal) — reproduced (four partners a period here).
- **Tag mutation** (E / H / T / S): 0.5%: 75.9 / 13.5 / 2.4 / 8.1; 5%: 76.0 / 13.8 / 1.9 / 8.2; 10%: 68.8 / 21.2 / 3.2 / 6.8; 20%: 54.5 / 32.2 / 5.6 / 7.7; 25%: 48.8 / 37.0 / 7.1 / 7.2; **30%: 39.9 / 44.9** / 7.0 / 8.2; 35%: 35.8 / 46.4; 40%: 30.1 / 50.6 / 12.2; 50%: 22.8 / 53.0 / 15.7; **60%: 21.8 / 49.5 / 20.8**; 75%: 14.7 / 47.6 / 29.8; **90%: 8.3 / 43.6 / 39.8**. Humanitarians pass ethnocentrics between 25% and 30% (J13: "at 30%") — reproduced. Traitors draw level with ethnocentrics at 60% (J13: pass them at 60%) and pass them by 75%. At 90% traitors do **not** outnumber humanitarians (39.8 vs 43.6; J13 says they do).
- **Table 4** (standard case): relatives 75.4 ± 0.6% of neighboring pairs (J13 RO + RI 74.7); P(same tag | related) **95.1 ± 0.3** (95.3); P(related | same tag) **90.1 ± 0.7** (89.2); `kin_help` 86.8 ± 0.7 of all helps (J13: 89% of an ethnocentric's donations); same-tag decisions 79.6. With 20 seeds: 75.5 / 94.8 / 89.6 / 86.5. Reproduced.
- **Kin strategies** (Table 5: none 2.0, outgroup 1.3, ingroup 16.4, nonkin 1.3, kin 76.2, all 2.8): **kin 52.1 ± 2.2**, ingroup (E) 26.7 ± 2.2, all (H) 12.0, none (S) 7.4, nonkin 1.0, outgroup (T) 0.7. Kin discriminators win but far less decisively — **not reproduced** with the basis as a mutating bit (J13 does not say how the basis is inherited; see Surprises).
- **Markers** (kin / E / gap, `jansson-markers` base): 2: 61.2 / 16.8 / 44.4; 4: 52.1 / 26.7 / 25.4; 8: 46.7 / 35.1 / 11.6; 12: 42.7 / 35.6 / 7.0; 16: 47.0 / 32.6 / 14.4; 20: 41.3 / 39.6 / 1.7; 24: 43.5 / 37.3 / 6.2; 28: 41.5 / 39.7 / 1.8; 32: 40.1 / 41.4 / −1.3; 36: 44.7 / 37.9 / 6.8; 40: 39.4 / 40.5 / −1.1 (gap s.e. 3–6). Gap below 10 from 12 colours (bar 16), not from 36 as J13 says — the gap closes sooner because it starts smaller; J13's shape (the gap shrinking with markers) holds.

### Follow-up 1: kin basis inheritance (`kin_basis`)

Sources: J13 never says how the tag-or-kin basis is inherited. §2.5 ("may mutate into another group or another in- or outgroup strategy, with a probability of 0.005 per trait") treats traits generically; §5.2 says only "agents are either group or kin discriminators". Nothing settles it; default stays `mutates`.

| Basis | none | outgroup | ingroup | nonkin | kin | all | coop |
|---|---|---|---|---|---|---|---|
| J13 Table 5 | 2.0 | 1.3 | 16.4 | 1.3 | 76.2 | 2.8 | – |
| mutates, 10 seeds | 7.4 ± 0.5 | 0.7 ± 0.2 | 26.7 ± 2.2 | 1.0 ± 0.1 | 52.1 ± 2.2 | 12.0 ± 0.6 | 73.2 |
| mutates, 20 seeds | 7.2 | 0.7 | 25.3 ± 1.5 | 1.1 | 53.7 ± 1.4 | 12.0 | 72.9 |
| **fixed**, 10 seeds | 6.2 ± 0.4 | 0.8 ± 0.1 | 12.6 ± 2.4 | 1.1 ± 0.2 | 65.5 ± 2.8 | 13.7 ± 1.5 | 74.2 |
| **fixed**, 20 seeds | 6.3 | 0.7 | 10.6 ± 1.4 | 1.1 | 68.9 ± 1.8 | 12.5 | 73.9 |

**Fixed matches Table 5 better** on kin (69 vs 76) and puts ingroup on the other side of 16.4 (11–13); both readings leave humanitarians (12–14) and selfish (6–7) far above Jansson's 2.8 and 2.0.

Markers (gap = kin − E, points; s.e. in brackets):

| Colours | 4 | 8 | 12 | 16 | 20 | 24 | 28 | 32 | 36 | 40 |
|---|---|---|---|---|---|---|---|---|---|---|
| mutates, 10 seeds | 25.4 | 11.6 | 7.0 | 14.4 | 1.7 | 6.2 | 1.8 | −1.3 | 6.7 | −1.1 |
| mutates, 20 seeds | 28.3 | 17.6 | 6.2 | 8.5 | 5.5 | 6.8 | 2.0 | 0.8 | 4.4 | 3.1 (3–4) |
| fixed, 10 seeds (sweep) | 52.9 | 25.5 | 16.8 | 8.9 | 13.4 | −1.2 | 19.2 | 21.2 | 1.9 | −15.3 (5–11) |
| fixed, 20 seeds | 58.3 | 28.2 | 24.9 | 10.1 | 13.8 | 1.5 | 16.2 | 13.7 | 14.7 | −3.7 (3–8) |

Crossing below 10 points: mutating at 12 colours and stays there (20 seeds); fixed hovers at 10–16 from 16 to 36 (one dip at 24) and closes clearly only at 40. J13's 36 lies between; **fixed is nearer J13's shape**, but the fixed gap's seed spread (s.d. 10–18 points) is too wide to pin a crossing at ten seeds. The ten-seed fixed run crosses (for good) at 36, which is noise.

### Follow-up 2: HA06's colour-blind 14% (cost 0.02, 10 seeds, 1,901–2,000; whole-run mean in brackets)

| Reading | Cooperation | Notes |
|---|---|---|
| HA06 | seeing 56, blind 14 | |
| (a) `discrimination: none` | 41.8 ± 2.3 (50.4) | 40 H / 60 S |
| (b) `colors: 1`, two bits, only the same bit expressed (Java `useOneTagOnly`) | 40.2 ± 2.6 (47.9) | |
| (c) `colors: 1`, mutation on both bits | 40.2 (identical to b) | in this model both bits always mutate and the other bit is never expressed, so (b) = (c) |
| (d1) a coin per decision (`misperception: 0.5`) | 44.9 ± 1.8 (50.3) | |
| (d2) blind, benefit halved (0.015) | **11.6 ± 1.1** (18.1) | but seeing agents then give 17.7, not 56 |
| (d3) seeing, benefit halved | 17.7 ± 0.8 | |
| (d4) blind, every decision twice | 24.5 ± 2.0 (34.2) | |
| (d5) blind, mutation 0.05 (appendix) | 39.3 ± 0.8 | |
| (d6) blind, cost 0.03 | **12.7 ± 1.7** (21.5) | seeing at 0.03: 29.8 |
| (d7) blind, offspring anywhere | 1.1 | |
| (d8) blind, egoist start, no immigration | 51.7 ± 3.7 | |
| (d9) blind, 500 periods | 59.0 ± 1.5 | |
| seeing, 2 colours | 65.1 ± 1.4 | |

**No reading reproduces the pair.** Blind cooperation near 14% needs a harsher game (benefit halved: 11.6, or cost 3%: 12.7), and in both the seeing world falls far below 56 (17.7, 29.8). The nearest joint fit is cost 2.5% (seeing 49.6, blind 19.8). No switch added.

### HKS13 no ethnocentrics preset (HST)

84.7 H / 7.0 S / 8.4 T %, cooperation 87.8, population 1,634.

### Golden fingerprints (200 ticks, seed 1)

ha-standard 0xf07433e56417f07c; ha-figure-1 0x843632b62ddf7a6b; ha-appendix-mutation 0xae8c7eda9113dae8; ha-appendix-double-play 0xac2c2167fec9c326; ha-java-five-colors 0xdc78c1e27b9ab453; ha-java-archive 0xde2cff652c758fe7; ha-egoist-start 0xabfdf5c9e1ccdb45; ha-cost-2 0x41ba53998a8ee613; ha-cost-2-blind 0x699aa05497139005; ha-misperception 0x5567187174fd1c15; ha-each-color 0x9ad570c3ea183419; jansson-offspring-anywhere 0xcad22f8e7abafbfe; jansson-tag-mutation-30 0xf9dbf338238a8f1b; jansson-kin 0x265998639eacfbd0; jansson-kin-fixed 0x3fac090571612879; hks-no-ethnocentrics 0xbe867e7210bad2d2. WASM (`wasm-pack test --node`): ha-standard, ha-misperception, jansson-kin equal native. Every earlier golden entry unchanged.

### Surprises (findings)

1. **HA06's colour-blind 14% does not reproduce.** At cost 2% blind agents cooperate 41.8% (three times 14%); no reading of "unable to distinguish" gets there (follow-up 2: one colour 40.2, coin-flip perception 44.9, deciding twice 24.5); only a harsher game does (benefit halved 11.6, cost 3% 12.7), and then seeing agents fall far below 56. Its companion number, 56% cooperation for seeing agents at cost 2%, is also high here (64.7). And **blind agents cooperate more than seeing ones at costs up to 1%** (81.2 vs 76.0 at the standard cost; 89.0 vs 78.5 at 0.5%) — seeing colour helps cooperation only when helping is expensive, which the paper's framing does not suggest.
2. **Ethnocentrics dominate later than Table 1 row l says.** After 500 periods 57% are ethnocentric, not 73.9%; the last-100 mean is 72.4% by period 1,500 and 75.9% by 2,000. HKS13's "around 300" holds for the median world (282) but individual worlds range from 21 to 596.
3. **"80 percent ethnocentric" with each-colour strategies is a loose count.** Only 27% help their own colour alone; 84% help their own colour and refuse at least one other — HA06's figure matches that looser sense.
4. **The appendix's 5% mutation is a slip** (36% ethnocentric, 56% cooperation), and its double-play loop, read literally, gives 81% — five points above Table 1 a. The text's 0.5%, played once, fits.
5. **The Java's five-colour draw is invisible in Table 1**: the ethnocentric share is flat from 3 to 6 colours; rows d/a/e fit 2/4/8 and 2/5/9 about equally.
6. **The archived Java (full random start, no immigration) lands on the same outcome** (77.7%), and faster (43% by period 100).
7. **HKS13 reproduces almost exactly**: final shares within 1 point, Study 2's fifteen orders all hold with counts mostly within 5%, early-pattern counts 17/18/15 vs 16/16/18.
8. **Jansson's kin discriminators win by far less** (52% vs 76%; ingroup 27% vs 16%) with the basis as a mutating bit, and the gap closes by 12 colours, not 36. With the basis fixed at immigration (follow-up 1) kin take 65–69% and the gap closes only near 40 — nearer J13 on both counts, though not settled by the sources.
9. **Rows g, i and j come out 4–6 points short of ethnocentrics** (and cooperation runs ~2 points high in most rows) — small but systematic; HA06's s.e. was about 1–2.
10. At 90% tag mutation traitors do not outnumber humanitarians (J13 says they do); everything else in J13 §4.4 reproduces.

## Why this split

Widening `AnyInspection` with `EthnoInspection` breaks the Inspect panel's fall-through to `schellingRows` (the remaining union is no longer Schelling's alone), so Inspect lands with the types in Task 3 rather than behind a stub; Task 4 then holds Compare, Experiments and the engine-level tests.

## File Structure

**Task 1 (core):** create `crates/sugarscape-core/src/ethno/{mod,config,stats,world,presets}.rs` (config and schema; statistics; the world, its rules, frames, Inspect and fingerprint; the presets); modify `lib.rs`, `model.rs` (the kind, config, world and keyframe arms), `presets.rs` (the catalog), `sweep.rs` (the ticks message), `crates/sugarscape-cli/src/main.rs` (the finish message), `tests/golden.rs` (16 entries), `tests/checkpoint.rs` (`jansson-kin`).
**Task 2 (measurement):** create `crates/sugarscape-core/tests/ethno.rs` (17 ignored book-style tests) and `sweeps/{ha-cost,ha-colors,ha-mutation,ha-immigration,ha-lattice,jansson-tag-mutation,jansson-markers}.json`; modify `sweep.rs` (BUILTINS and ids), `crates/sugarscape-cli/tests/cli.rs` and `crates/sugarscape-wasm/tests/web.rs` (ids and a fingerprint test).
**Task 3 (page):** create `web/src/ethno.ts` (+ test); modify `crates/sugarscape-core/src/schema.rs` and `ethno/config.rs` (a nullable field), `web/src/{types,models,schema-form,engine}.ts`, `web/src/ui/{series-data,inspect-panel}.ts` and their tests.
**Task 4 (Compare, Experiments):** modify `web/src/compare-presets.ts`, `web/src/experiments/form.ts`, `web/src/determinism.test.ts`, `web/src/engine.test.ts` and tests.
**Task 5 (survey):** create `survey/src/claims/ethno.rs`; modify `survey/src/claim.rs` (a shared `all_of`), `survey/src/claims/{mod,ch6}.rs`.
**Task 6 (docs):** modify `README.md`, `docs/roadmap.md`, the spec.

---

### Task 1: The ethnocentrism model in the core

**Files:**
- Create: `crates/sugarscape-core/src/ethno/{mod,config,stats,world,presets}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/src/sweep.rs` (the ticks message only), `crates/sugarscape-cli/src/main.rs` (the finish message only), `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-core/tests/checkpoint.rs`

**Interfaces:**
- Consumes: `crate::spatial::{Geometry, SpatialConfig, Neighborhood, Boundary}` (the periodic von Neumann square, Decision 2); `crate::config::{FieldError, ScheduledChange}`; `crate::model::{wrong_model, Model, ModelConfig, ModelKind}`; `crate::presets::ModelPreset`; `crate::render::{lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, LENDER, NEUTRAL, POLLUTION, RED, SPICE}`; `crate::rng::{self, SimRng}`; `crate::schema::{Apply, Param}` (with `shown_if`, `with_help`); `crate::stats::{Series, Stats}`; `crate::export::history_csv`; `crate::schema::check_schema` (tests).
- Produces: `ethno::{EthnoConfig, KinBasis, Start, PairPlay, Discrimination, Offspring, Strategy, LIVE, schema, EthnoSnapshot, SERIES, EthnoWorld, EthnoMode, Agent, EthnoInspection, AgentView, NeighborView, Site, tag_color, lineage_color, ETHNOCENTRIC, HUMANITARIAN, SELFISH, TRAITOROUS, KIN, NONKIN, MIXED, presets}`; `EthnoConfig::{validate, changes, tag_rate, allows_all}`; `EthnoWorld::{new, step, run, sites, agent, agents, population, is_finished, strategy, inspect, geometry, stats, config, tick}`; `Strategy::{FOUR, of_bits, letter}`; `ModelKind::Ethno` (`"ethno"`), `ModelConfig::Ethno`, `ModelWorld::Ethno` with keyframes; `MODEL_GOLDEN` gains sixteen entries.

- [ ] **Step 1: Write the failing tests**

Create `crates/sugarscape-core/src/ethno/config.rs` with only its tests for now (the implementation goes above them in Step 3):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &EthnoConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_ha06s_text_and_validates() {
        let c = EthnoConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!(
            (c.width, c.colors, c.immigration, c.mutation, c.end),
            (50, 4, 1.0, 0.005, 2000)
        );
        assert_eq!(
            (c.base_ptr, c.cost, c.benefit, c.death),
            (0.12, 0.01, 0.03, 0.1)
        );
        assert_eq!(c.tag_rate(), 0.005);
        assert_eq!(c.kin_basis, KinBasis::Mutates);
        assert_eq!(serde_json::to_value(&c).unwrap()["kin_basis"], "mutates");
        assert!(c.allows_all());
        assert_eq!(
            serde_json::to_value(&c).unwrap()["allowed"],
            json!(["E", "H", "S", "T"])
        );
    }

    #[test]
    fn validation_names_the_field() {
        let bad = |edit: &dyn Fn(&mut EthnoConfig)| {
            let mut c = EthnoConfig::default();
            edit(&mut c);
            fields(&c)
        };
        assert_eq!(bad(&|c| c.width = 2), ["width"]);
        assert_eq!(bad(&|c| c.width = 201), ["width"]);
        assert_eq!(bad(&|c| c.colors = 0), ["colors"]);
        assert_eq!(bad(&|c| c.colors = 41), ["colors"]);
        assert_eq!(bad(&|c| c.cost = -0.01), ["cost"]);
        assert_eq!(bad(&|c| c.benefit = f64::NAN), ["benefit"]);
        assert_eq!(bad(&|c| c.base_ptr = -1.0), ["base_ptr"]);
        assert_eq!(bad(&|c| c.death = 1.5), ["death"]);
        assert_eq!(bad(&|c| c.mutation = -0.1), ["mutation"]);
        assert_eq!(bad(&|c| c.tag_mutation = Some(2.0)), ["tag_mutation"]);
        assert_eq!(bad(&|c| c.kin_mutation = 2.0), ["kin_mutation"]);
        assert_eq!(bad(&|c| c.immigration = -1.0), ["immigration"]);
        assert_eq!(
            bad(&|c| {
                c.discrimination = Discrimination::EachColor;
                c.kin_strategies = true;
            }),
            ["kin_strategies"]
        );
        assert_eq!(
            bad(&|c| {
                c.discrimination = Discrimination::EachColor;
                c.misperception = 0.1;
            }),
            ["misperception"]
        );
        assert_eq!(bad(&|c| c.allowed.clear()), ["allowed"]);
        assert_eq!(bad(&|c| c.allowed = vec![Strategy::Kin]), ["allowed"]);
        assert_eq!(
            bad(&|c| {
                c.allowed = vec![Strategy::Humanitarian];
                c.discrimination = Discrimination::None;
            }),
            ["allowed"]
        );
        assert_eq!(
            bad(&|c| {
                c.allowed = vec![Strategy::Humanitarian];
                c.kin_strategies = true;
            }),
            ["allowed"]
        );
        assert_eq!(
            bad(&|c| {
                c.allowed = vec![Strategy::Humanitarian];
                c.start = Start::Selfish;
            }),
            ["start"]
        );
        assert!(bad(&|c| c.allowed = vec![Strategy::Humanitarian, Strategy::Selfish]).is_empty());
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = EthnoConfig {
            schedule: vec![entry("cost", json!(0.02))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("colors", json!(5))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("death", json!(2))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = EthnoConfig::default();
        let mut b = a.clone();
        b.cost = 0.02;
        b.tag_mutation = Some(0.3);
        b.offspring = Offspring::Anywhere;
        assert!(a.changes(&b).is_empty());
        b.colors = 5;
        b.start = Start::Random;
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["colors", "start"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: EthnoConfig =
            serde_json::from_str(r#"{"start": "random", "allowed": ["H", "S"]}"#).unwrap();
        assert_eq!(
            (c.start, c.allowed.clone(), c.width),
            (
                Start::Random,
                vec![Strategy::Humanitarian, Strategy::Selfish],
                50
            )
        );
        assert!(serde_json::from_str::<EthnoConfig>(r#"{"tags": 3}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        // The panel's tag mutation is a number; null (the default) is
        // "the mutation rate", so the check starts from a set value.
        let config = ModelConfig::Ethno(EthnoConfig {
            width: 10,
            tag_mutation: Some(0.005),
            ..Default::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

`crates/sugarscape-core/src/ethno/world.rs`, tests only:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduledChange;
    use serde_json::json;

    /// An empty `w` × `w` world without immigration, after `edit`.
    fn world(w: u32, edit: impl FnOnce(&mut EthnoConfig)) -> EthnoWorld {
        let mut c = EthnoConfig {
            width: w,
            immigration: 0.0,
            ..Default::default()
        };
        edit(&mut c);
        EthnoWorld::new(c, 1).unwrap()
    }

    /// Puts an agent with `tag` and help bits `help` at (x, y), founding
    /// lineage and family `lineage`; returns its site.
    fn put(w: &mut EthnoWorld, (x, y): (u32, u32), tag: u32, help: u64, lineage: u64) -> usize {
        let s = w.geometry.at(x, y, 0).unwrap();
        let id = w.next_id;
        w.next_id += 1;
        w.place(
            s,
            Agent {
                id,
                tag,
                help,
                kin_basis: false,
                lineage,
                family: lineage,
                born: 0,
                ptr: 0.0,
                given: 0,
                received: 0,
                gave: [0; DIRECTIONS],
            },
        );
        s
    }

    const E: u64 = 0b01;
    const H: u64 = 0b11;
    const S: u64 = 0b00;
    const T: u64 = 0b10;

    fn ptr(w: &EthnoWorld, s: usize) -> f64 {
        w.agent(s).unwrap().ptr
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn each_strategy_pair_moves_ptr_by_cost_and_benefit() {
        // (a's bits, b's bits, same tag) → a's and b's PTR after one period's
        // interaction, as a pair alone on the lattice.
        let cases = [
            (E, E, true, 0.12 - 0.01 + 0.03, 0.12 - 0.01 + 0.03),
            (E, E, false, 0.12, 0.12),
            (E, H, false, 0.12 + 0.03, 0.12 - 0.01),
            (H, H, false, 0.14, 0.14),
            (S, H, true, 0.15, 0.11),
            (T, T, false, 0.14, 0.14),
            (T, E, false, 0.11, 0.15),
            (S, S, true, 0.12, 0.12),
        ];
        for (a_bits, b_bits, same, pa, pb) in cases {
            let mut w = world(5, |_| {});
            let a = put(&mut w, (1, 1), 0, a_bits, 1);
            let b = put(&mut w, (2, 1), if same { 0 } else { 1 }, b_bits, 2);
            w.interact();
            assert!(
                close(ptr(&w, a), pa) && close(ptr(&w, b), pb),
                "{a_bits:b} vs {b_bits:b}, same {same}: {} {}",
                ptr(&w, a),
                ptr(&w, b)
            );
        }
    }

    #[test]
    fn an_agent_pays_for_each_neighbor_it_helps() {
        let mut w = world(5, |_| {});
        let mid = put(&mut w, (2, 2), 0, H, 1);
        for (k, xy) in [(2, 1), (1, 2), (3, 2), (2, 3)].into_iter().enumerate() {
            put(&mut w, xy, k as u32, S, 2);
        }
        w.interact();
        assert!(close(ptr(&w, mid), 0.12 - 4.0 * 0.01));
        let a = w.agent(mid).unwrap();
        assert_eq!((a.given, a.received, a.gave), (4, 0, [1; 4]));
        let up = w.geometry.at(2, 1, 0).unwrap();
        assert!(close(ptr(&w, up), 0.15));
    }

    #[test]
    fn twice_decides_every_direction_twice() {
        let mut w = world(5, |c| c.pair_play = PairPlay::Twice);
        let a = put(&mut w, (1, 1), 0, H, 1);
        let b = put(&mut w, (1, 2), 0, S, 2);
        w.interact();
        assert!(close(ptr(&w, a), 0.12 - 2.0 * 0.01));
        assert!(close(ptr(&w, b), 0.12 + 2.0 * 0.03));
        assert_eq!((w.tally.decisions, w.tally.helps, w.tally.pairs), (4, 2, 2));
    }

    #[test]
    fn misperception_one_inverts_every_judgment() {
        let mut w = world(5, |c| c.misperception = 1.0);
        let a = put(&mut w, (1, 1), 0, E, 1);
        let b = put(&mut w, (2, 1), 0, T, 2);
        w.interact();
        // Each sees the other as another colour: the E defects, the T helps.
        assert!(close(ptr(&w, a), 0.15) && close(ptr(&w, b), 0.11));
    }

    #[test]
    fn blind_and_each_colour_agents_decide_by_their_bits() {
        let mut blind = world(5, |c| c.discrimination = Discrimination::None);
        let a = put(&mut blind, (1, 1), 0, 1, 1);
        let b = put(&mut blind, (2, 1), 1, 0, 2);
        blind.interact();
        assert!(close(ptr(&blind, a), 0.11) && close(ptr(&blind, b), 0.15));
        assert_eq!(
            blind.strategy(blind.agent(a).unwrap()),
            Strategy::Humanitarian
        );
        assert_eq!(blind.strategy(blind.agent(b).unwrap()), Strategy::Selfish);
        let mut each = world(5, |c| {
            c.discrimination = Discrimination::EachColor;
            c.colors = 3;
        });
        // Helps colours 0 and 2; its neighbors are colours 1 and 2.
        let a = put(&mut each, (1, 1), 0, 0b101, 1);
        let b = put(&mut each, (2, 1), 1, 0, 2);
        let c = put(&mut each, (1, 2), 2, 0, 3);
        each.interact();
        assert!(close(ptr(&each, a), 0.11));
        assert!(close(ptr(&each, b), 0.12) && close(ptr(&each, c), 0.15));
        let kind = |w: &EthnoWorld, tag: u32, help: u64| {
            let mut probe = w.agent(a).unwrap().clone();
            (probe.tag, probe.help) = (tag, help);
            w.strategy(&probe)
        };
        assert_eq!(kind(&each, 0, 0b101), Strategy::Mixed);
        assert_eq!(kind(&each, 1, 0b010), Strategy::Ethnocentric);
        assert_eq!(kind(&each, 1, 0b101), Strategy::Traitorous);
        assert_eq!(kind(&each, 2, 0b111), Strategy::Humanitarian);
        assert_eq!(kind(&each, 2, 0), Strategy::Selfish);
    }

    #[test]
    fn kin_strategies_judge_by_the_kin_marker() {
        let mut w = world(5, |c| c.kin_strategies = true);
        let a = put(&mut w, (1, 1), 0, E, 1);
        put(&mut w, (2, 1), 1, S, 1);
        w.sites[a].as_mut().unwrap().kin_basis = true;
        w.interact();
        assert!(
            close(ptr(&w, a), 0.11),
            "the same family, another tag: helps"
        );
        assert_eq!(w.strategy(w.agent(a).unwrap()), Strategy::Kin);
        w.sites[a].as_mut().unwrap().help = T;
        assert_eq!(w.strategy(w.agent(a).unwrap()), Strategy::Nonkin);
        w.sites[a].as_mut().unwrap().help = H;
        assert_eq!(w.strategy(w.agent(a).unwrap()), Strategy::Humanitarian);
    }

    #[test]
    fn a_mutated_tag_is_never_the_parents_and_is_uniform_over_the_rest() {
        let mut w = world(5, |c| {
            c.colors = 5;
            c.tag_mutation = Some(1.0);
            c.mutation = 0.0;
        });
        let p = put(&mut w, (0, 0), 2, E, 1);
        let parent = w.agent(p).unwrap().clone();
        let mut counts = [0u32; 5];
        for _ in 0..20_000 {
            let child = w.offspring_of(&parent);
            counts[child.tag as usize] += 1;
            assert_eq!(child.help, E, "strategy bits do not mutate at rate 0");
        }
        assert_eq!(counts[2], 0);
        for (t, &n) in counts.iter().enumerate() {
            if t != 2 {
                assert!((4700..=5300).contains(&n), "tag {t}: {n}");
            }
        }
    }

    #[test]
    fn allowed_strategies_bind_immigrants_and_offspring() {
        let mut w = world(10, |c| {
            c.allowed = vec![Strategy::Humanitarian, Strategy::Selfish];
            c.immigration = 3.0;
            c.mutation = 0.3;
            c.base_ptr = 0.5;
        });
        for _ in 0..200 {
            w.step();
            for a in w.agents() {
                assert!(
                    matches!(w.strategy(a), Strategy::Humanitarian | Strategy::Selfish),
                    "{a:?}"
                );
            }
        }
        assert!(w.population() > 20);
        // One allowed strategy: its bits never change.
        let mut solo = world(10, |c| {
            c.allowed = vec![Strategy::Traitorous];
            c.immigration = 2.0;
            c.mutation = 1.0;
        });
        solo.run(50);
        assert!(solo.agents().all(|a| a.help == T));
    }

    #[test]
    fn lineage_is_inherited_and_the_kin_marker_can_found_a_family() {
        let mut w = world(5, |c| {
            c.kin_strategies = true;
            c.kin_mutation = 1.0;
        });
        let immigrant = w.newcomer(1);
        assert_eq!(
            (immigrant.lineage, immigrant.family),
            (immigrant.id, immigrant.id)
        );
        let child = w.offspring_of(&immigrant);
        assert_eq!(child.lineage, immigrant.lineage);
        assert_eq!(child.family, child.id, "a new family");
        w.config.kin_mutation = 0.0;
        let grandchild = w.offspring_of(&child);
        assert_eq!(
            (grandchild.lineage, grandchild.family),
            (immigrant.id, child.id)
        );
        // The basis bit flips at the mutation rate, unless it is fixed.
        w.config.mutation = 1.0;
        assert!(w.offspring_of(&grandchild).kin_basis != grandchild.kin_basis);
        w.config.kin_basis = KinBasis::Fixed;
        assert_eq!(w.offspring_of(&grandchild).kin_basis, grandchild.kin_basis);
        // Without kin strategies the marker never mutates.
        let mut plain = world(5, |c| c.kin_mutation = 1.0);
        let a = plain.newcomer(1);
        assert_eq!(plain.offspring_of(&a).family, a.family);
    }

    #[test]
    fn fractional_immigration_is_a_chance_of_one_more() {
        let mut w = world(100, |c| c.immigration = 0.5);
        for _ in 0..2000 {
            w.immigrate();
        }
        assert!((900..=1100).contains(&w.population()), "{}", w.population());
        let mut two = world(10, |c| c.immigration = 2.0);
        two.immigrate();
        assert_eq!(two.population(), 2);
        let mut full = world(5, |c| {
            c.start = Start::Random;
            c.immigration = 3.0;
        });
        let next = full.next_id;
        full.immigrate();
        assert_eq!(
            (full.population(), full.next_id),
            (25, next),
            "immigrants are lost"
        );
    }

    #[test]
    fn offspring_go_to_an_empty_neighbor_or_anywhere() {
        // A parent at (3, 3) certain to reproduce, with `taken` neighbors
        // that cannot.
        let parent_with = |offspring: Offspring, taken: &[(u32, u32)]| {
            let mut w = world(7, |c| {
                c.offspring = offspring;
                c.mutation = 0.0;
            });
            put(&mut w, (3, 3), 0, E, 1);
            for &xy in taken {
                put(&mut w, xy, 0, S, 2);
            }
            for a in w.sites.iter_mut().flatten() {
                a.ptr = if a.lineage == 1 { 1.0 } else { 0.0 };
            }
            w.reproduce();
            w
        };
        let three = [(3, 2), (2, 3), (4, 3)];
        let w = parent_with(Offspring::Adjacent, &three);
        assert_eq!(w.population(), 5);
        let open = w.geometry.at(3, 4, 0).unwrap();
        assert_eq!(
            w.agent(open).map(|a| a.lineage),
            Some(1),
            "the one empty neighbor"
        );
        let four = [(3, 2), (2, 3), (4, 3), (3, 4)];
        assert_eq!(
            parent_with(Offspring::Adjacent, &four).population(),
            5,
            "no room"
        );
        let far = parent_with(Offspring::Anywhere, &four);
        assert_eq!(far.population(), 6, "anywhere finds room");
        let child = far.agents().find(|a| a.id == 6).unwrap();
        assert_eq!(child.lineage, 1);
    }

    #[test]
    fn offspring_do_not_reproduce_in_the_period_they_are_born() {
        let mut w = world(9, |c| {
            c.base_ptr = 1.0;
            c.cost = 0.0;
            c.death = 0.0;
            c.mutation = 0.0;
        });
        put(&mut w, (4, 4), 0, S, 1);
        w.step();
        assert_eq!(w.population(), 2, "one parent, one child");
        w.step();
        assert_eq!(w.population(), 4);
        assert!(w.agents().all(|a| a.lineage == 1));
    }

    #[test]
    fn death_takes_newcomers_too() {
        let mut w = world(9, |c| {
            c.immigration = 1.0;
            c.base_ptr = 1.0;
            c.death = 1.0;
        });
        put(&mut w, (4, 4), 0, S, 1);
        w.step();
        assert_eq!(
            w.population(),
            0,
            "the parent, its child and the immigrant all die"
        );
        assert!(w.next_id >= 4, "the immigrant and a child were made");
    }

    #[test]
    fn the_statistics_count_decisions_pairs_and_shares() {
        let mut w = world(6, |c| c.death = 0.0);
        // A row: E(tag 0, lineage 1), E(tag 0, lineage 1), T(tag 1, lineage 2),
        // H(tag 0, lineage 3); no other neighbors.
        put(&mut w, (0, 0), 0, E, 1);
        put(&mut w, (1, 0), 0, E, 1);
        put(&mut w, (2, 0), 1, T, 2);
        put(&mut w, (3, 0), 0, H, 3);
        w.interact();
        w.tick += 1;
        w.record();
        let s = w.stats.latest().unwrap().clone();
        // Ordered pairs: (0,1) (1,0) (1,2) (2,1) (2,3) (3,2) = 6.
        // Helps: 0→1, 1→0 (same), 2→1, 2→3 (T to others), 3→2 (H) = 5.
        assert_eq!(s.population, 4);
        assert!(close(s.ethnocentric, 0.5) && close(s.traitorous, 0.25));
        assert!(close(s.humanitarian, 0.25) && s.selfish == 0.0 && s.kin == 0.0);
        assert!(close(s.cooperation, 5.0 / 6.0));
        assert!(close(s.same_tag, 2.0 / 6.0));
        assert!(close(s.relatives, 2.0 / 6.0));
        assert!(close(s.kin_help, 2.0 / 5.0));
        assert!(close(s.tag_given_relative, 1.0));
        assert!(close(s.relative_given_tag, 1.0));
        let empty = world(5, |_| {});
        let s0 = empty.stats.latest().unwrap();
        assert!(s0.cooperation.is_nan() && s0.ethnocentric.is_nan());
    }

    #[test]
    fn full_starts_fill_the_lattice() {
        let w = world(10, |c| c.start = Start::Random);
        assert_eq!(w.population(), 100);
        let lineages: std::collections::BTreeSet<u64> = w.agents().map(|a| a.lineage).collect();
        assert_eq!(lineages.len(), 100, "each founds its own lineage");
        let s = world(10, |c| c.start = Start::Selfish);
        assert!(s.agents().all(|a| s.strategy(a) == Strategy::Selfish));
        let tags: std::collections::BTreeSet<u32> = s.agents().map(|a| a.tag).collect();
        assert_eq!(tags.len(), 4);
        let five = world(20, |c| {
            c.start = Start::Random;
            c.colors = 5;
        });
        assert!(five.agents().any(|a| a.tag == 4), "five colours draw tag 4");
    }

    #[test]
    fn inspect_describes_the_agent_and_who_helped_whom() {
        let mut w = world(5, |_| {});
        put(&mut w, (1, 1), 0, E, 1);
        put(&mut w, (2, 1), 0, H, 1);
        put(&mut w, (1, 2), 1, H, 2);
        w.interact();
        let v = w.inspect(1, 1).unwrap().agent.unwrap();
        assert_eq!(
            (v.strategy, v.basis, v.given, v.received),
            (Strategy::Ethnocentric, "tag", 1, 2)
        );
        assert_eq!(v.neighbors.len(), 2);
        let right = &v.neighbors[0];
        assert_eq!(
            (
                right.x,
                right.y,
                right.related,
                right.helped,
                right.helped_by
            ),
            (2, 1, true, 1, 1)
        );
        let down = &v.neighbors[1];
        assert_eq!(
            (down.tag, down.related, down.helped, down.helped_by),
            (1, false, 0, 1)
        );
        assert!(w.inspect(0, 0).unwrap().agent.is_none());
        assert!(w.inspect(5, 0).is_err());
        let json: serde_json::Value = serde_json::from_str(&w.inspect_json(1, 1).unwrap()).unwrap();
        assert_eq!(json["agent"]["strategy"], "E");
        assert_eq!(json["agent"]["neighbors"][1]["strategy"], "H");
    }

    #[test]
    fn frames_draw_empty_sites_dark_and_every_mode() {
        let mut w = world(4, |_| {});
        put(&mut w, (1, 0), 2, T, 1);
        let mut buf = Vec::new();
        w.render("strategy", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 4 * 4 * 4);
        assert_eq!(&buf[0..3], &BACKGROUND);
        assert_eq!(&buf[4..7], &TRAITOROUS);
        w.render("tag", "", &mut buf).unwrap();
        assert_eq!(&buf[4..7], &LENDER, "tag 2 is the Java's green");
        for mode in ["lineage", "ptr"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_ne!(&buf[4..7], &BACKGROUND);
        }
        assert!(w.render("nope", "", &mut buf).is_err());
        let distinct: std::collections::BTreeSet<Rgb> = (0..40).map(tag_color).collect();
        assert_eq!(distinct.len(), 40);
    }

    #[test]
    fn runs_stop_at_the_end_and_follow_their_seed() {
        let c = EthnoConfig {
            width: 20,
            end: 30,
            ..Default::default()
        };
        let mut a = EthnoWorld::new(c.clone(), 7).unwrap();
        let mut b = EthnoWorld::new(c.clone(), 7).unwrap();
        let mut other = EthnoWorld::new(c, 8).unwrap();
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
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(10, |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("cost".to_string(), json!(0.02))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.cost, 0.01);
        w.step();
        assert_eq!(w.config.cost, 0.02);
    }
}
```

`crates/sugarscape-core/src/model.rs` gains a round-trip test beside the others, and the expected kinds end with `"tags", "ethno"` (full hunk in Step 2). In `crates/sugarscape-core/tests/golden.rs` append to `MODEL_GOLDEN` after `("rca-adopt-p1", 0xc3f75d53d237a89),` the sixteen entries of Decision 13, in that order; in `crates/sugarscape-core/tests/checkpoint.rs` add `"jansson-kin",` to `IDS` after `"hg-async-kaleidoscope",`:

```diff
--- a/crates/sugarscape-core/tests/golden.rs
+++ b/crates/sugarscape-core/tests/golden.rs
@@ -93,6 +93,22 @@
     ("eh-clones-only", 0x1bd933620c915c0e),
     ("eh-no-exact-clones", 0xafe94d6c0bb3b8ab),
     ("rca-adopt-p1", 0xc3f75d53d237a89),
+    ("ha-standard", 0xf07433e56417f07c),
+    ("ha-figure-1", 0x843632b62ddf7a6b),
+    ("ha-appendix-mutation", 0xae8c7eda9113dae8),
+    ("ha-appendix-double-play", 0xac2c2167fec9c326),
+    ("ha-java-five-colors", 0xdc78c1e27b9ab453),
+    ("ha-java-archive", 0xde2cff652c758fe7),
+    ("ha-egoist-start", 0xabfdf5c9e1ccdb45),
+    ("ha-cost-2", 0x41ba53998a8ee613),
+    ("ha-cost-2-blind", 0x699aa05497139005),
+    ("ha-misperception", 0x5567187174fd1c15),
+    ("ha-each-color", 0x9ad570c3ea183419),
+    ("jansson-offspring-anywhere", 0xcad22f8e7abafbfe),
+    ("jansson-tag-mutation-30", 0xf9dbf338238a8f1b),
+    ("jansson-kin", 0x265998639eacfbd0),
+    ("jansson-kin-fixed", 0x3fac090571612879),
+    ("hks-no-ethnocentrics", 0xbe867e7210bad2d2),
 ];
 
 fn fingerprint(id: &str) -> u64 {
--- a/crates/sugarscape-core/tests/checkpoint.rs
+++ b/crates/sugarscape-core/tests/checkpoint.rs
@@ -13,6 +13,7 @@
     "cv-run-8-nasty-regime",
     "nbm-probabilistic",
     "hg-async-kaleidoscope",
+    "jansson-kin",
 ];
 
 fn world(id: &str) -> ModelWorld {
```

Run: `cargo test -p sugarscape-core --lib ethno` — Expected: compile errors (no `ethno` module).

- [ ] **Step 2: Wire the model kind**

`lib.rs` (`pub mod ethno;` after `pub mod edit;`), `model.rs` (every arm beside the tags model's, `ALL: [ModelKind; 8]`, the unknown-model message, `max_ticks` from `end` as the tags model's, keyframes, the round-trip test), `presets.rs` (the catalog ends with `crate::ethno::presets()`), `sweep.rs` (the ticks error names the model) and the CLI's finish message:

```diff
--- a/crates/sugarscape-core/src/lib.rs
+++ b/crates/sugarscape-core/src/lib.rs
@@ -10,6 +10,7 @@
 pub mod config;
 pub mod econ;
 pub mod edit;
+pub mod ethno;
 pub mod export;
 pub mod geometry;
 pub mod landscape;
```

```diff
--- a/crates/sugarscape-core/src/model.rs
+++ b/crates/sugarscape-core/src/model.rs
@@ -8,6 +8,7 @@
 use crate::anasazi::{AnasaziConfig, AnasaziWorld};
 use crate::civil::{CivilConfig, CivilWorld};
 use crate::config::{Config, FieldError};
+use crate::ethno::{EthnoConfig, EthnoWorld};
 use crate::render::{self, ColorMode, Layer};
 use crate::ring::{RingConfig, RingWorld};
 use crate::schelling::{SchellingConfig, SchellingWorld};
@@ -15,7 +16,7 @@
 use crate::spatial::{SpatialConfig, SpatialWorld};
 use crate::tags::{TagsConfig, TagsWorld};
 use crate::world::World;
-use crate::{anasazi, civil, export, ring, schelling, spatial, stats, tags};
+use crate::{anasazi, civil, ethno, export, ring, schelling, spatial, stats, tags};
 
 /// Which model a config or world is.
 #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
@@ -28,10 +29,11 @@
     Civil,
     Spatial,
     Tags,
+    Ethno,
 }
 
 impl ModelKind {
-    pub const ALL: [ModelKind; 7] = [
+    pub const ALL: [ModelKind; 8] = [
         ModelKind::Sugarscape,
         ModelKind::Schelling,
         ModelKind::Ring,
@@ -39,6 +41,7 @@
         ModelKind::Civil,
         ModelKind::Spatial,
         ModelKind::Tags,
+        ModelKind::Ethno,
     ];
 
     pub fn as_str(self) -> &'static str {
@@ -50,6 +53,7 @@
             ModelKind::Civil => "civil",
             ModelKind::Spatial => "spatial",
             ModelKind::Tags => "tags",
+            ModelKind::Ethno => "ethno",
         }
     }
 
@@ -64,6 +68,7 @@
             ModelKind::Civil => civil::schema(),
             ModelKind::Spatial => spatial::schema(),
             ModelKind::Tags => tags::schema(),
+            ModelKind::Ethno => ethno::schema(),
         }
     }
 }
@@ -84,6 +89,7 @@
     Civil(CivilConfig),
     Spatial(SpatialConfig),
     Tags(TagsConfig),
+    Ethno(EthnoConfig),
 }
 
 /// Another model's config on the wire: its fields and `"model": "<kind>"`.
@@ -96,6 +102,7 @@
     Civil(&'a CivilConfig),
     Spatial(&'a SpatialConfig),
     Tags(&'a TagsConfig),
+    Ethno(&'a EthnoConfig),
 }
 
 impl From<Config> for ModelConfig {
@@ -115,6 +122,7 @@
             ModelConfig::Civil(c) => Tagged::Civil(c).serialize(s),
             ModelConfig::Spatial(c) => Tagged::Spatial(c).serialize(s),
             ModelConfig::Tags(c) => Tagged::Tags(c).serialize(s),
+            ModelConfig::Ethno(c) => Tagged::Ethno(c).serialize(s),
         }
     }
 }
@@ -129,6 +137,7 @@
             ModelConfig::Civil(_) => ModelKind::Civil,
             ModelConfig::Spatial(_) => ModelKind::Spatial,
             ModelConfig::Tags(_) => ModelKind::Tags,
+            ModelConfig::Ethno(_) => ModelKind::Ethno,
         }
     }
 
@@ -183,11 +192,14 @@
                 .map_err(|e| FieldError::new("config", e.to_string())),
             "tags" => serde_json::from_value(value)
                 .map(ModelConfig::Tags)
+                .map_err(|e| FieldError::new("config", e.to_string())),
+            "ethno" => serde_json::from_value(value)
+                .map(ModelConfig::Ethno)
                 .map_err(|e| FieldError::new("config", e.to_string())),
             _ => Err(FieldError::new(
                 "model",
                 format!(
-                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial or tags)"
+                    "unknown model {tag:?} (expected sugarscape, schelling, ring, anasazi, civil, spatial, tags or ethno)"
                 ),
             )),
         }
@@ -202,6 +214,7 @@
             ModelConfig::Civil(c) => c.validate(),
             ModelConfig::Spatial(c) => c.validate(),
             ModelConfig::Tags(c) => c.validate(),
+            ModelConfig::Ethno(c) => c.validate(),
         }
     }
 
@@ -216,6 +229,7 @@
             ModelConfig::Civil(c) => set_path(c, path, value).map(ModelConfig::Civil),
             ModelConfig::Spatial(c) => set_path(c, path, value).map(ModelConfig::Spatial),
             ModelConfig::Tags(c) => set_path(c, path, value).map(ModelConfig::Tags),
+            ModelConfig::Ethno(c) => set_path(c, path, value).map(ModelConfig::Ethno),
         }
     }
 
@@ -225,6 +239,7 @@
         match self {
             ModelConfig::Anasazi(c) => Some(c.end_year.saturating_sub(c.start_year)),
             ModelConfig::Tags(c) => (c.end > 0).then_some(c.end),
+            ModelConfig::Ethno(c) => (c.end > 0).then_some(c.end),
             ModelConfig::Sugarscape(_)
             | ModelConfig::Schelling(_)
             | ModelConfig::Ring(_)
@@ -243,6 +258,7 @@
             ModelConfig::Civil(_) => civil::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Spatial(_) => spatial::SERIES.iter().map(|s| s.to_string()).collect(),
             ModelConfig::Tags(_) => tags::SERIES.iter().map(|s| s.to_string()).collect(),
+            ModelConfig::Ethno(_) => ethno::SERIES.iter().map(|s| s.to_string()).collect(),
         }
     }
 }
@@ -404,6 +420,7 @@
     Civil(Box<CivilWorld>),
     Spatial(Box<SpatialWorld>),
     Tags(Box<TagsWorld>),
+    Ethno(Box<EthnoWorld>),
 }
 
 impl ModelWorld {
@@ -430,6 +447,7 @@
             ModelConfig::Civil(c) => ModelWorld::Civil(Box::new(CivilWorld::new(c, seed)?)),
             ModelConfig::Spatial(c) => ModelWorld::Spatial(Box::new(SpatialWorld::new(c, seed)?)),
             ModelConfig::Tags(c) => ModelWorld::Tags(Box::new(TagsWorld::new(c, seed)?)),
+            ModelConfig::Ethno(c) => ModelWorld::Ethno(Box::new(EthnoWorld::new(c, seed)?)),
         })
     }
 
@@ -442,6 +460,7 @@
             ModelWorld::Civil(_) => ModelKind::Civil,
             ModelWorld::Spatial(_) => ModelKind::Spatial,
             ModelWorld::Tags(_) => ModelKind::Tags,
+            ModelWorld::Ethno(_) => ModelKind::Ethno,
         }
     }
 
@@ -454,6 +473,7 @@
             ModelWorld::Civil(w) => w.as_ref(),
             ModelWorld::Spatial(w) => w.as_ref(),
             ModelWorld::Tags(w) => w.as_ref(),
+            ModelWorld::Ethno(w) => w.as_ref(),
         }
     }
 
@@ -466,6 +486,7 @@
             ModelWorld::Civil(w) => w.as_mut(),
             ModelWorld::Spatial(w) => w.as_mut(),
             ModelWorld::Tags(w) => w.as_mut(),
+            ModelWorld::Ethno(w) => w.as_mut(),
         }
     }
 
@@ -546,6 +567,7 @@
             ModelWorld::Civil(w) => copy_without_history!(Civil, w),
             ModelWorld::Spatial(w) => copy_without_history!(Spatial, w),
             ModelWorld::Tags(w) => copy_without_history!(Tags, w),
+            ModelWorld::Ethno(w) => copy_without_history!(Ethno, w),
             _ => return None,
         };
         Some(Checkpoint { world, tick })
@@ -568,6 +590,7 @@
             (ModelWorld::Civil(live), ModelWorld::Civil(kept)) => restore_into!(live, kept),
             (ModelWorld::Spatial(live), ModelWorld::Spatial(kept)) => restore_into!(live, kept),
             (ModelWorld::Tags(live), ModelWorld::Tags(kept)) => restore_into!(live, kept),
+            (ModelWorld::Ethno(live), ModelWorld::Ethno(kept)) => restore_into!(live, kept),
             _ => return Err("the keyframe is of another model".into()),
         }
         Ok(())
@@ -790,6 +813,34 @@
         let w = ModelWorld::new(c, 1).unwrap();
         assert_eq!(w.kind(), ModelKind::Tags);
         assert_eq!(w.model().size(), (100, 200));
+    }
+
+    #[test]
+    fn ethno_configs_round_trip_with_their_tag() {
+        let c = ModelConfig::from_json(
+            r#"{"model": "ethno", "colors": 5, "allowed": ["H", "S", "T"]}"#,
+        )
+        .unwrap();
+        assert_eq!(c.kind(), ModelKind::Ethno);
+        let json = serde_json::to_value(&c).unwrap();
+        assert_eq!(json["model"], "ethno");
+        assert_eq!(json["tag_mutation"], serde_json::Value::Null);
+        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
+        assert_eq!(c.series_names()[..2], ["population", "ethnocentric"]);
+        assert_eq!(c.max_ticks(), Some(2000));
+        let next = c.with_path("tag_mutation", &json!(0.3)).unwrap();
+        let ModelConfig::Ethno(e) = &next else {
+            unreachable!()
+        };
+        assert_eq!(e.tag_mutation, Some(0.3));
+        let e = ModelConfig::from_json(r#"{"model": "ethno", "cost": -1}"#).unwrap_err();
+        assert_eq!(e[0].field, "cost");
+        let mut w = ModelWorld::new(c, 1).unwrap();
+        assert_eq!((w.kind(), w.model().size()), (ModelKind::Ethno, (50, 50)));
+        let cp = w.checkpoint().expect("ethno worlds have keyframes");
+        w.model_mut().run(3);
+        w.restore(&cp).unwrap();
+        assert_eq!(w.model().tick(), 0);
     }
 
     #[test]
@@ -822,7 +873,8 @@
                 "anasazi",
                 "civil",
                 "spatial",
-                "tags"
+                "tags",
+                "ethno"
             ]
         );
         assert!(ModelKind::Sugarscape.schema().is_empty());
```

```diff
--- a/crates/sugarscape-core/src/presets.rs
+++ b/crates/sugarscape-core/src/presets.rs
@@ -634,7 +634,8 @@
 }
 
 /// Every model's presets: the sugarscape's (`all`), then Schelling's, Ring
-/// World's, the anasazi's, civil violence's and the spatial games'.
+/// World's, the anasazi's, civil violence's, the tags model's, the spatial
+/// games' and the ethnocentrism model's.
 pub fn catalog() -> Vec<ModelPreset> {
     let mut out: Vec<ModelPreset> = all().into_iter().map(ModelPreset::from).collect();
     out.extend(crate::schelling::presets());
@@ -643,6 +644,7 @@
     out.extend(crate::civil::presets());
     out.extend(crate::tags::presets());
     out.extend(crate::spatial::presets());
+    out.extend(crate::ethno::presets());
     out
 }
```

```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -394,6 +394,11 @@
                 crate::model::ModelKind::Tags => {
                     format!("the tags model stops at its last generation, {max} in this config")
                 }
+                crate::model::ModelKind::Ethno => {
+                    format!(
+                        "the ethnocentrism model stops at its last period, {max} in this config"
+                    )
+                }
                 _ => format!(
                     "the Long House Valley stops at its end year, \
                      {max} ticks after its start year in this config"
```

```diff
--- a/crates/sugarscape-cli/src/main.rs
+++ b/crates/sugarscape-cli/src/main.rs
@@ -180,10 +180,11 @@
     let world = world.model();
     if world.finished() && world.tick() < u64::from(args.ticks) {
         // The anasazi stops at its end year; civil violence when a group is gone;
-        // the tags model at its last generation.
+        // the tags model at its last generation; ethnocentrism at its last period.
         let why = match config.kind() {
             ModelKind::Civil => "a group has died out",
             ModelKind::Tags => "its last generation",
+            ModelKind::Ethno => "its last period",
             _ => "its end year",
         };
         eprintln!("finished at tick {} ({why})", world.tick());
```

- [ ] **Step 3: Implement**

`config.rs`, above its tests (Decisions 3, 6, 8, 11–13):

```rust
//! The ethnocentrism model's parameters: HA06's text by default, with its
//! appendix, its authors' code and its critics' variants as named switches.

use serde::{Deserialize, Serialize};

use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// How the lattice starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// Empty; immigration fills it (HA06).
    #[default]
    Empty,
    /// Every site holds an agent with random traits (the archived Java).
    Random,
    /// Every site holds an agent with a random tag that helps no one (HA06's
    /// "full lattice of egoists").
    Selfish,
}

/// How often each agent decides whether to help each neighbor in a period.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PairPlay {
    /// Once per neighbor (HA-Java, NetLogo).
    #[default]
    Once,
    /// Twice: HA06's appendix loop read literally.
    Twice,
}

/// What an agent's help decision can depend on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Discrimination {
    /// Two bits: help the same colour, help other colours.
    #[default]
    SameOther,
    /// One bit: help everyone or no one (HA06's "unable to distinguish").
    None,
    /// One bit per colour (HA06's "distinguish all four colors").
    EachColor,
}

/// Where an offspring goes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Offspring {
    /// A random empty neighbor, or nowhere (HA06).
    #[default]
    Adjacent,
    /// A random empty site anywhere (J13 §3.6).
    Anywhere,
}

/// How an offspring inherits the kin-strategies basis bit (tag or kin
/// marker). J13 does not say.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KinBasis {
    /// Like the strategy bits: it flips with probability `mutation`.
    #[default]
    Mutates,
    /// Drawn at immigration and never mutates.
    Fixed,
}

/// A strategy as the statistics and Inspect name it. `allowed` lists only
/// the first four.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Strategy {
    /// Helps its own colour (or kin) only.
    #[serde(rename = "E")]
    Ethnocentric,
    /// Helps everyone.
    #[serde(rename = "H")]
    Humanitarian,
    /// Helps no one.
    #[serde(rename = "S")]
    Selfish,
    /// Helps other colours only.
    #[serde(rename = "T")]
    Traitorous,
    /// Helps its kin (same kin marker) only (J13 §5.2).
    #[serde(rename = "kin")]
    Kin,
    /// Helps non-kin only.
    #[serde(rename = "nonkin")]
    Nonkin,
    /// Each colour: helps some other pattern of colours.
    #[serde(rename = "mixed")]
    Mixed,
}

impl Strategy {
    /// The four HA06 strategies, as `allowed` lists them by default.
    pub const FOUR: [Strategy; 4] = [
        Strategy::Ethnocentric,
        Strategy::Humanitarian,
        Strategy::Selfish,
        Strategy::Traitorous,
    ];

    /// The strategy of the two bits (help same, help other).
    pub fn of_bits(same: bool, other: bool) -> Strategy {
        match (same, other) {
            (true, false) => Strategy::Ethnocentric,
            (true, true) => Strategy::Humanitarian,
            (false, false) => Strategy::Selfish,
            (false, true) => Strategy::Traitorous,
        }
    }

    pub fn letter(self) -> &'static str {
        match self {
            Strategy::Ethnocentric => "E",
            Strategy::Humanitarian => "H",
            Strategy::Selfish => "S",
            Strategy::Traitorous => "T",
            Strategy::Kin => "kin",
            Strategy::Nonkin => "nonkin",
            Strategy::Mixed => "mixed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct EthnoConfig {
    /// The torus is `width` × `width`.
    pub width: u32,
    /// The number of tags.
    pub colors: u32,
    pub start: Start,
    /// Immigrants per period: ⌊r⌋, plus one more with probability r − ⌊r⌋.
    pub immigration: f64,
    /// PTR at the start of each interaction step.
    pub base_ptr: f64,
    /// PTR a helper loses.
    pub cost: f64,
    /// PTR the helped gains.
    pub benefit: f64,
    /// Per-period death probability.
    pub death: f64,
    /// Per strategy trait (and per tag unless `tag_mutation` is set).
    pub mutation: f64,
    /// The tag's own mutation rate (J13 §4.4); `None` = `mutation`.
    pub tag_mutation: Option<f64>,
    pub pair_play: PairPlay,
    pub discrimination: Discrimination,
    /// Probability a decision misjudges same/other (`same_other` only).
    pub misperception: f64,
    pub offspring: Offspring,
    /// The strategies immigrants and mutations may produce (HKS13 Study 2).
    pub allowed: Vec<Strategy>,
    /// Adds a basis bit: discriminate on the tag or on the kin marker (J13 §5.2).
    pub kin_strategies: bool,
    /// Whether the basis bit mutates (`kin_strategies` only).
    pub kin_basis: KinBasis,
    /// The kin marker's mutation rate: an offspring founds a new family.
    pub kin_mutation: f64,
    /// The last period; the run stops there (0: never).
    pub end: u32,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for EthnoConfig {
    /// HA06's text: an empty 50 × 50 torus, four colours, one immigrant a
    /// period, PTR 0.12, cost 0.01, benefit 0.03, death 0.10, mutation
    /// 0.005, 2,000 periods.
    fn default() -> Self {
        EthnoConfig {
            width: 50,
            colors: 4,
            start: Start::Empty,
            immigration: 1.0,
            base_ptr: 0.12,
            cost: 0.01,
            benefit: 0.03,
            death: 0.10,
            mutation: 0.005,
            tag_mutation: None,
            pair_play: PairPlay::Once,
            discrimination: Discrimination::SameOther,
            misperception: 0.0,
            offspring: Offspring::Adjacent,
            allowed: Strategy::FOUR.to_vec(),
            kin_strategies: false,
            kin_basis: KinBasis::Mutates,
            kin_mutation: 0.005,
            end: 2000,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 12] = [
    "immigration",
    "base_ptr",
    "cost",
    "benefit",
    "death",
    "mutation",
    "tag_mutation",
    "pair_play",
    "misperception",
    "offspring",
    "kin_mutation",
    "end",
];

impl EthnoConfig {
    /// The tag's mutation rate.
    pub fn tag_rate(&self) -> f64 {
        self.tag_mutation.unwrap_or(self.mutation)
    }

    /// Whether `allowed` holds all four strategies.
    pub fn allows_all(&self) -> bool {
        Strategy::FOUR.iter().all(|s| self.allowed.contains(s))
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
        check(
            (3..=200).contains(&self.width),
            "width",
            "must be between 3 and 200",
        );
        check(
            (1..=40).contains(&self.colors),
            "colors",
            "must be between 1 and 40",
        );
        check(
            nonneg(self.immigration) && self.immigration <= 100.0,
            "immigration",
            "must be between 0 and 100",
        );
        check(nonneg(self.base_ptr), "base_ptr", "must be a number ≥ 0");
        check(nonneg(self.cost), "cost", "must be a number ≥ 0");
        check(nonneg(self.benefit), "benefit", "must be a number ≥ 0");
        check(unit(self.death), "death", "must be between 0 and 1");
        check(unit(self.mutation), "mutation", "must be between 0 and 1");
        if let Some(t) = self.tag_mutation {
            check(unit(t), "tag_mutation", "must be between 0 and 1");
        }
        check(
            unit(self.misperception),
            "misperception",
            "must be between 0 and 1",
        );
        check(
            unit(self.kin_mutation),
            "kin_mutation",
            "must be between 0 and 1",
        );
        let each = self.discrimination == Discrimination::EachColor;
        check(
            !(each && self.kin_strategies),
            "kin_strategies",
            "each-colour strategies have no kin variant",
        );
        check(
            !(each && self.misperception > 0.0),
            "misperception",
            "applies only to same/other discrimination",
        );
        check(
            !self.allowed.is_empty(),
            "allowed",
            "must allow at least one strategy",
        );
        check(
            self.allowed.iter().all(|s| Strategy::FOUR.contains(s)),
            "allowed",
            "may list only E, H, S and T",
        );
        check(
            self.allows_all()
                || (self.discrimination == Discrimination::SameOther && !self.kin_strategies),
            "allowed",
            "restricting strategies needs same/other discrimination without kin strategies",
        );
        check(
            self.start != Start::Selfish || self.allowed.contains(&Strategy::Selfish),
            "start",
            "a selfish start needs S among the allowed strategies",
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
                match ModelConfig::Ethno(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Ethno(next)) => {
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
    pub fn changes(&self, next: &EthnoConfig) -> Vec<FieldError> {
        let a = serde_json::to_value(self).expect("config serializes");
        let b = serde_json::to_value(next).expect("config serializes");
        let (a, b) = (a.as_object().unwrap(), b.as_object().unwrap());
        a.keys()
            .filter(|k| a[*k] != b[*k] && !LIVE.contains(&k.as_str()))
            .map(|k| FieldError::new(k.as_str(), "changes only on reset"))
            .collect()
    }
}

/// The Rules panel's fields. `allowed` is not on the panel (a list is not a
/// panel kind); presets, files and links set it.
pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::number("Game", "cost", "Cost of helping", (0.0, 0.1, 0.001), Live)
            .with_help("PTR the helper loses"),
        Param::number(
            "Game",
            "benefit",
            "Benefit of help",
            (0.0, 0.1, 0.001),
            Live,
        )
        .with_help("PTR the helped agent gains"),
        Param::number("Game", "base_ptr", "Base PTR", (0.0, 0.5, 0.01), Live)
            .with_help("Each period's potential to reproduce before any help"),
        Param::choice(
            "Game",
            "pair_play",
            "Decisions per neighbor",
            &[
                ("once", "Once (the code)"),
                ("twice", "Twice (the appendix, literally)"),
            ],
            Live,
        ),
        Param::integer(
            "Population",
            "width",
            "Width (a square torus)",
            (3, 200),
            Reset,
        ),
        Param::choice(
            "Population",
            "start",
            "Start",
            &[
                ("empty", "Empty (HA06)"),
                ("random", "Full, random traits (the archived code)"),
                ("selfish", "Full of egoists"),
            ],
            Reset,
        ),
        Param::number(
            "Population",
            "immigration",
            "Immigrants per period",
            (0.0, 5.0, 0.5),
            Live,
        )
        .with_help("A fraction is the chance of one more"),
        Param::number("Population", "death", "Death rate", (0.0, 0.5, 0.01), Live),
        Param::choice(
            "Population",
            "offspring",
            "Offspring",
            &[
                ("adjacent", "Next to the parent (HA06)"),
                ("anywhere", "Anywhere (Jansson)"),
            ],
            Live,
        ),
        Param::integer("Traits", "colors", "Colours (tags)", (1, 40), Reset),
        Param::choice(
            "Traits",
            "discrimination",
            "Agents see",
            &[
                ("same_other", "Same or other colour"),
                ("none", "Nothing (help all or none)"),
                ("each_color", "Each colour"),
            ],
            Reset,
        ),
        Param::number(
            "Traits",
            "misperception",
            "Misperception",
            (0.0, 0.5, 0.01),
            Live,
        )
        .with_help("The chance a decision mistakes same for other colour")
        .shown_if("discrimination", "same_other"),
        Param::bool(
            "Traits",
            "kin_strategies",
            "Kin strategies (Jansson)",
            Reset,
        )
        .with_help("Agents discriminate on the tag or on a kin marker"),
        Param::choice(
            "Traits",
            "kin_basis",
            "Tag or kin basis",
            &[
                ("mutates", "Mutates like a strategy bit"),
                ("fixed", "Fixed at immigration"),
            ],
            Reset,
        )
        .shown_if("kin_strategies", "true"),
        Param::number(
            "Traits",
            "kin_mutation",
            "Kin marker mutation",
            (0.0, 0.1, 0.001),
            Live,
        )
        .shown_if("kin_strategies", "true"),
        Param::number(
            "Mutation",
            "mutation",
            "Mutation rate",
            (0.0, 0.1, 0.0005),
            Live,
        )
        .with_help("Per trait and offspring"),
        Param::number(
            "Mutation",
            "tag_mutation",
            "Tag mutation rate",
            (0.0, 1.0, 0.005),
            Live,
        )
        .with_help("Empty: the mutation rate"),
        Param::integer("Run", "end", "Last period (0: never)", (0, 100_000), Live),
    ]
}
```

`stats.rs` (Decision 10):

```rust
//! The ethnocentrism model's statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 14] = [
    "population",
    "ethnocentric",
    "humanitarian",
    "selfish",
    "traitorous",
    "kin",
    "nonkin",
    "mixed",
    "cooperation",
    "same_tag",
    "relatives",
    "kin_help",
    "tag_given_relative",
    "relative_given_tag",
];

/// One period's statistics. Shares are of the population at the end of the
/// period; the interaction statistics are of this period's decisions and
/// neighboring pairs (at the interaction step). A zero denominator gives NaN
/// (JSON null).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct EthnoSnapshot {
    pub tick: u64,
    pub population: u32,
    pub ethnocentric: f64,
    pub humanitarian: f64,
    pub selfish: f64,
    pub traitorous: f64,
    pub kin: f64,
    pub nonkin: f64,
    pub mixed: f64,
    /// Helps ÷ decisions (HA06's "percent cooperative behavior").
    pub cooperation: f64,
    /// The share of decisions toward an agent of the same tag.
    pub same_tag: f64,
    /// The share of neighboring pairs with a common founding immigrant.
    pub relatives: f64,
    /// The share of helps given to relatives.
    pub kin_help: f64,
    /// P(same tag | related pair) (J13 Table 4's p(i|r)).
    pub tag_given_relative: f64,
    /// P(related | same-tag pair) (J13 Table 4's p(r|i)).
    pub relative_given_tag: f64,
}

impl Series for EthnoSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "population" => f64::from(self.population),
            "ethnocentric" => self.ethnocentric,
            "humanitarian" => self.humanitarian,
            "selfish" => self.selfish,
            "traitorous" => self.traitorous,
            "kin" => self.kin,
            "nonkin" => self.nonkin,
            "mixed" => self.mixed,
            "cooperation" => self.cooperation,
            "same_tag" => self.same_tag,
            "relatives" => self.relatives,
            "kin_help" => self.kin_help,
            "tag_given_relative" => self.tag_given_relative,
            "relative_given_tag" => self.relative_given_tag,
            _ => return None,
        })
    }
}

/// This period's interaction counts, from which the snapshot's ratios come.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub decisions: u64,
    pub helps: u64,
    /// Decisions toward an agent of the same tag.
    pub same_tag: u64,
    /// Helps given to a relative.
    pub helps_to_relatives: u64,
    /// Ordered neighboring pairs (each unordered pair twice): all, related,
    /// same-tag, and both.
    pub pairs: u64,
    pub related: u64,
    pub same: u64,
    pub related_same: u64,
}

/// `a / b`, or NaN when `b` is 0.
pub fn ratio(a: u64, b: u64) -> f64 {
    if b == 0 {
        f64::NAN
    } else {
        a as f64 / b as f64
    }
}
```

`world.rs`, above its tests (Decisions 2–10, 14–17):

```rust
//! The ethnocentrism world: agents with a tag and help bits on a von Neumann
//! torus. Each period immigrants arrive, every agent decides whether to help
//! each neighbor (paying `cost` of its potential to reproduce, the neighbor
//! gaining `benefit`), agents reproduce with probability PTR into an empty
//! site near them, and each dies with probability `death`.

use std::fmt::Write;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{Discrimination, EthnoConfig, KinBasis, Offspring, PairPlay, Start, Strategy};
use super::stats::{ratio, EthnoSnapshot, Tally, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::render::{
    lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, LENDER, NEUTRAL, POLLUTION, RED, SPICE,
};
use crate::rng::{self, SimRng};
use crate::spatial::{self, Geometry};
use crate::stats::Stats;

/// The Strategy view's colours.
pub const ETHNOCENTRIC: Rgb = LENDER;
pub const HUMANITARIAN: Rgb = BLUE;
pub const SELFISH: Rgb = RED;
pub const TRAITOROUS: Rgb = BOTH;
pub const KIN: Rgb = POLLUTION;
pub const NONKIN: Rgb = SPICE;
pub const MIXED: Rgb = NEUTRAL;

/// The four neighbors of a site, in the lattice's order: up, left, right,
/// down. The neighbor in direction `d` sees this site in direction `3 − d`.
const DIRECTIONS: usize = 4;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EthnoMode {
    Strategy,
    Tag,
    Lineage,
    Ptr,
}

impl std::str::FromStr for EthnoMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "strategy" => Self::Strategy,
            "tag" => Self::Tag,
            "lineage" => Self::Lineage,
            "ptr" => Self::Ptr,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// An agent on the lattice.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    pub id: u64,
    pub tag: u32,
    /// Help bits. `same_other`: bit 0 helps the same colour (or kin), bit 1
    /// other colours; `none`: bit 0 helps everyone; `each_color`: bit k
    /// helps colour k.
    pub help: u64,
    /// With kin strategies: same/other is judged by the kin marker.
    pub kin_basis: bool,
    /// The founding immigrant's id (never mutates).
    pub lineage: u64,
    /// The kin marker: the id of the family's founder (an immigrant or a
    /// mutated offspring).
    pub family: u64,
    /// The period it arrived or was born in (0: a full start).
    pub born: u64,
    /// This period's potential to reproduce.
    pub ptr: f64,
    /// Helps given and received this period.
    pub given: u32,
    pub received: u32,
    /// Helps given this period to the neighbor in each direction.
    pub gave: [u8; DIRECTIONS],
}

/// What Inspect shows for a site.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EthnoInspection {
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
    pub tag: u32,
    pub strategy: Strategy,
    /// "tag" or "kin": what same/other is judged by.
    pub basis: &'static str,
    pub ptr: f64,
    pub given: u32,
    pub received: u32,
    pub lineage: u64,
    pub kin_marker: u64,
    pub age: u64,
    /// The occupied neighbors: up, left, right, down.
    pub neighbors: Vec<NeighborView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct NeighborView {
    pub x: u32,
    pub y: u32,
    pub tag: u32,
    pub strategy: Strategy,
    /// A common founding immigrant.
    pub related: bool,
    /// Helps from the agent to this neighbor this period, and back.
    pub helped: u8,
    pub helped_by: u8,
}

#[derive(Clone)]
pub struct EthnoWorld {
    pub config: EthnoConfig,
    /// The periodic von Neumann lattice, fixed once built, so keyframes share it.
    pub geometry: Arc<Geometry>,
    /// Completed periods.
    pub tick: u64,
    sites: Vec<Option<Agent>>,
    /// The empty sites (in no particular order) and each site's index in it
    /// (`u32::MAX` when occupied).
    empty: Vec<u32>,
    slot: Vec<u32>,
    next_id: u64,
    /// This period's interaction counts.
    tally: Tally,
    rng: SimRng,
    pub stats: Stats<EthnoSnapshot>,
}

/// The lowest `bits` bits.
fn mask(bits: u32) -> u64 {
    if bits >= 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    }
}

impl EthnoWorld {
    pub fn new(config: EthnoConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let lattice = spatial::SpatialConfig {
            width: config.width,
            height: config.width,
            neighborhood: spatial::Neighborhood::VonNeumann,
            boundary: spatial::Boundary::Periodic,
            ..Default::default()
        };
        // A square lattice draws nothing; the world's stream starts untouched.
        let geometry = Geometry::new(&lattice, &mut rng::seeded(0));
        let n = geometry.len();
        let mut w = EthnoWorld {
            config,
            geometry: Arc::new(geometry),
            tick: 0,
            sites: vec![None; n],
            empty: (0..n as u32).collect(),
            slot: (0..n as u32).collect(),
            next_id: 1,
            tally: Tally::default(),
            rng: rng::seeded(seed),
            stats: Stats::default(),
        };
        match w.config.start {
            Start::Empty => {}
            Start::Random => {
                for s in 0..n {
                    let a = w.newcomer(0);
                    w.place(s, a);
                }
            }
            Start::Selfish => {
                for s in 0..n {
                    let mut a = w.newcomer(0);
                    a.help = 0;
                    a.kin_basis = false;
                    w.place(s, a);
                }
            }
        }
        w.record();
        Ok(w)
    }

    pub fn sites(&self) -> usize {
        self.sites.len()
    }

    pub fn agent(&self, site: usize) -> Option<&Agent> {
        self.sites[site].as_ref()
    }

    pub fn agents(&self) -> impl Iterator<Item = &Agent> {
        self.sites.iter().flatten()
    }

    pub fn population(&self) -> usize {
        self.sites.len() - self.empty.len()
    }

    /// Whether the run has reached its last period.
    pub fn is_finished(&self) -> bool {
        self.config.end > 0 && self.tick >= u64::from(self.config.end)
    }

    /// The number of help bits a strategy has.
    fn help_bits(&self) -> u32 {
        match self.config.discrimination {
            Discrimination::SameOther => 2,
            Discrimination::None => 1,
            Discrimination::EachColor => self.config.colors,
        }
    }

    /// Whether help bits `help` make an allowed strategy.
    fn permitted(&self, help: u64) -> bool {
        self.config.allows_all()
            || self
                .config
                .allowed
                .contains(&Strategy::of_bits(help & 1 != 0, help & 2 != 0))
    }

    /// The strategy `a` plays.
    pub fn strategy(&self, a: &Agent) -> Strategy {
        match self.config.discrimination {
            Discrimination::SameOther => {
                match (
                    Strategy::of_bits(a.help & 1 != 0, a.help & 2 != 0),
                    a.kin_basis,
                ) {
                    (Strategy::Ethnocentric, true) => Strategy::Kin,
                    (Strategy::Traitorous, true) => Strategy::Nonkin,
                    (s, _) => s,
                }
            }
            Discrimination::None => {
                if a.help & 1 != 0 {
                    Strategy::Humanitarian
                } else {
                    Strategy::Selfish
                }
            }
            Discrimination::EachColor => {
                let all = mask(self.config.colors);
                let own = 1u64 << a.tag;
                match a.help {
                    0 => Strategy::Selfish,
                    h if h == all => Strategy::Humanitarian,
                    h if h == own => Strategy::Ethnocentric,
                    h if h == all & !own => Strategy::Traitorous,
                    _ => Strategy::Mixed,
                }
            }
        }
    }

    /// An agent arriving in period `born` (0: a full start) with a uniformly
    /// random tag, random help bits redrawn until allowed, and (with kin
    /// strategies) a random basis bit; it founds its own lineage and family.
    fn newcomer(&mut self, born: u64) -> Agent {
        let tag = self.rng.gen_range(0..self.config.colors);
        let bits = mask(self.help_bits());
        let help = loop {
            let h = self.rng.gen::<u64>() & bits;
            if self.permitted(h) {
                break h;
            }
        };
        let kin_basis = self.config.kin_strategies && self.rng.gen::<bool>();
        let id = self.next_id;
        self.next_id += 1;
        Agent {
            id,
            tag,
            help,
            kin_basis,
            lineage: id,
            family: id,
            born,
            ptr: self.config.base_ptr,
            given: 0,
            received: 0,
            gave: [0; DIRECTIONS],
        }
    }

    /// `parent`'s offspring: a copy whose strategy bits (in order), basis
    /// bit and tag may mutate, and which may found a new family.
    fn offspring_of(&mut self, parent: &Agent) -> Agent {
        let c = &self.config;
        let (mutation, tag_rate, kin_rate) = (c.mutation, c.tag_rate(), c.kin_mutation);
        let (colors, kin) = (c.colors, c.kin_strategies);
        let basis_mutates = kin && c.kin_basis == KinBasis::Mutates;
        let mut help = parent.help;
        for k in 0..self.help_bits() {
            if self.rng.gen::<f64>() < mutation {
                let next = help ^ (1 << k);
                // HKS13: "that mutation is ignored".
                if self.permitted(next) {
                    help = next;
                }
            }
        }
        let mut kin_basis = parent.kin_basis;
        if basis_mutates && self.rng.gen::<f64>() < mutation {
            kin_basis = !kin_basis;
        }
        let mut tag = parent.tag;
        if self.rng.gen::<f64>() < tag_rate && colors > 1 {
            // Uniform over the other colours (HA-Java redraws until different).
            let t = self.rng.gen_range(0..colors - 1);
            tag = if t >= parent.tag { t + 1 } else { t };
        }
        let id = self.next_id;
        self.next_id += 1;
        let family = if kin && self.rng.gen::<f64>() < kin_rate {
            id
        } else {
            parent.family
        };
        Agent {
            id,
            tag,
            help,
            kin_basis,
            lineage: parent.lineage,
            family,
            born: self.tick + 1,
            ptr: self.config.base_ptr,
            given: 0,
            received: 0,
            gave: [0; DIRECTIONS],
        }
    }

    fn place(&mut self, site: usize, a: Agent) {
        debug_assert!(self.sites[site].is_none(), "site {site} is occupied");
        let k = self.slot[site] as usize;
        let last = *self.empty.last().expect("an empty site");
        self.empty.swap_remove(k);
        if last as usize != site {
            self.slot[last as usize] = k as u32;
        }
        self.slot[site] = u32::MAX;
        self.sites[site] = Some(a);
    }

    fn vacate(&mut self, site: usize) {
        if self.sites[site].take().is_some() {
            self.slot[site] = self.empty.len() as u32;
            self.empty.push(site as u32);
        }
    }

    /// A uniformly chosen empty site, if any.
    fn random_empty(&mut self) -> Option<usize> {
        if self.empty.is_empty() {
            None
        } else {
            let k = self.rng.gen_range(0..self.empty.len() as u32) as usize;
            Some(self.empty[k] as usize)
        }
    }

    /// One period.
    pub fn step(&mut self) {
        self.apply_schedule();
        self.immigrate();
        self.interact();
        self.reproduce();
        self.die();
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
            let next = ModelConfig::Ethno(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Ethno(next)) = next {
                self.config = next;
            }
        }
    }

    /// Step 1: ⌊r⌋ immigrants, one more with probability r − ⌊r⌋, each at a
    /// uniformly chosen empty site (lost when the lattice is full).
    fn immigrate(&mut self) {
        let r = self.config.immigration;
        let whole = r.floor();
        let mut count = whole as u32;
        if r > whole && self.rng.gen::<f64>() < r - whole {
            count += 1;
        }
        for _ in 0..count {
            let Some(site) = self.random_empty() else {
                break;
            };
            let a = self.newcomer(self.tick + 1);
            self.place(site, a);
        }
    }

    /// Whether `a` helps `b` in one decision (drawing misperception).
    fn helps(&mut self, a: &Agent, b: &Agent) -> bool {
        match self.config.discrimination {
            Discrimination::SameOther => {
                let mut same = if a.kin_basis {
                    a.family == b.family
                } else {
                    a.tag == b.tag
                };
                let p = self.config.misperception;
                if p > 0.0 && self.rng.gen::<f64>() < p {
                    same = !same;
                }
                a.help & if same { 1 } else { 2 } != 0
            }
            Discrimination::None => a.help & 1 != 0,
            Discrimination::EachColor => a.help & (1 << b.tag) != 0,
        }
    }

    /// Step 2: every PTR set to the base, then in site order each agent
    /// decides for each occupied neighbor (up, left, right, down) whether to
    /// help it — twice with `pair_play: twice`.
    fn interact(&mut self) {
        let base = self.config.base_ptr;
        for a in self.sites.iter_mut().flatten() {
            a.ptr = base;
            a.given = 0;
            a.received = 0;
            a.gave = [0; DIRECTIONS];
        }
        let times = match self.config.pair_play {
            PairPlay::Once => 1,
            PairPlay::Twice => 2,
        };
        let (cost, benefit) = (self.config.cost, self.config.benefit);
        let geometry = Arc::clone(&self.geometry);
        let mut t = Tally::default();
        for i in 0..self.sites.len() {
            if self.sites[i].is_none() {
                continue;
            }
            for (d, &j) in geometry.neighbors(i).iter().enumerate() {
                let j = j as usize;
                let (Some(a), Some(b)) = (&self.sites[i], &self.sites[j]) else {
                    continue;
                };
                let (a, b) = (a.clone(), b.clone());
                let related = a.lineage == b.lineage;
                let same = a.tag == b.tag;
                t.pairs += 1;
                t.related += u64::from(related);
                t.same += u64::from(same);
                t.related_same += u64::from(related && same);
                for _ in 0..times {
                    t.decisions += 1;
                    t.same_tag += u64::from(same);
                    if self.helps(&a, &b) {
                        t.helps += 1;
                        t.helps_to_relatives += u64::from(related);
                        let giver = self.sites[i].as_mut().unwrap();
                        giver.ptr -= cost;
                        giver.given += 1;
                        giver.gave[d] += 1;
                        let taker = self.sites[j].as_mut().unwrap();
                        taker.ptr += benefit;
                        taker.received += 1;
                    }
                }
            }
        }
        self.tally = t;
    }

    /// Step 3: the agents present, in a random order, each reproduce with
    /// probability PTR into a random empty neighbor (or, `anywhere`, a
    /// random empty site). Offspring do not reproduce this period.
    fn reproduce(&mut self) {
        let mut order: Vec<usize> = (0..self.sites.len())
            .filter(|&s| self.sites[s].is_some())
            .collect();
        order.shuffle(&mut self.rng);
        let geometry = Arc::clone(&self.geometry);
        for s in order {
            let u = self.rng.gen::<f64>();
            let parent = self.sites[s].clone().expect("parents stay put");
            if u >= parent.ptr {
                continue;
            }
            let target = match self.config.offspring {
                Offspring::Adjacent => {
                    let free: Vec<usize> = geometry
                        .neighbors(s)
                        .iter()
                        .map(|&j| j as usize)
                        .filter(|&j| self.sites[j].is_none())
                        .collect();
                    if free.is_empty() {
                        continue;
                    }
                    free[self.rng.gen_range(0..free.len() as u32) as usize]
                }
                Offspring::Anywhere => match self.random_empty() {
                    Some(site) => site,
                    None => continue,
                },
            };
            let child = self.offspring_of(&parent);
            self.place(target, child);
            // Nobody helped the newcomer this period.
            for (d, &j) in geometry.neighbors(target).iter().enumerate() {
                if let Some(n) = self.sites[j as usize].as_mut() {
                    n.gave[DIRECTIONS - 1 - d] = 0;
                }
            }
        }
    }

    /// Step 4: in site order every agent, newcomers included, dies with
    /// probability `death`.
    fn die(&mut self) {
        let death = self.config.death;
        for s in 0..self.sites.len() {
            if self.sites[s].is_some() && self.rng.gen::<f64>() < death {
                self.vacate(s);
            }
        }
    }

    fn record(&mut self) {
        // Indexed by `Strategy`'s declaration order.
        let mut counts = [0u32; 7];
        for a in self.agents() {
            counts[self.strategy(a) as usize] += 1;
        }
        let n = self.population() as u64;
        let share = |s: Strategy| ratio(u64::from(counts[s as usize]), n);
        let t = self.tally;
        let s = EthnoSnapshot {
            tick: self.tick,
            population: n as u32,
            ethnocentric: share(Strategy::Ethnocentric),
            humanitarian: share(Strategy::Humanitarian),
            selfish: share(Strategy::Selfish),
            traitorous: share(Strategy::Traitorous),
            kin: share(Strategy::Kin),
            nonkin: share(Strategy::Nonkin),
            mixed: share(Strategy::Mixed),
            cooperation: ratio(t.helps, t.decisions),
            same_tag: ratio(t.same_tag, t.decisions),
            relatives: ratio(t.related, t.pairs),
            kin_help: ratio(t.helps_to_relatives, t.helps),
            tag_given_relative: ratio(t.related_same, t.related),
            relative_given_tag: ratio(t.related_same, t.same),
        };
        self.stats.push(s);
    }

    fn color(&self, a: &Agent, mode: EthnoMode) -> Rgb {
        match mode {
            EthnoMode::Strategy => match self.strategy(a) {
                Strategy::Ethnocentric => ETHNOCENTRIC,
                Strategy::Humanitarian => HUMANITARIAN,
                Strategy::Selfish => SELFISH,
                Strategy::Traitorous => TRAITOROUS,
                Strategy::Kin => KIN,
                Strategy::Nonkin => NONKIN,
                Strategy::Mixed => MIXED,
            },
            EthnoMode::Tag => tag_color(a.tag),
            EthnoMode::Lineage => lineage_color(a.lineage),
            EthnoMode::Ptr => {
                let times = match self.config.pair_play {
                    PairPlay::Once => 4.0,
                    PairPlay::Twice => 8.0,
                };
                let c = &self.config;
                let (lo, hi) = (c.base_ptr - times * c.cost, c.base_ptr + times * c.benefit);
                let t = if hi > lo {
                    (a.ptr - lo) / (hi - lo)
                } else {
                    0.5
                };
                lerp(COOL, HOT, t)
            }
        }
    }

    pub fn inspect(&self, x: u32, y: u32) -> Result<EthnoInspection, String> {
        let site = self
            .geometry
            .at(x, y, 0)
            .ok_or_else(|| format!("({x}, {y}) is outside the lattice"))?;
        let agent = self.sites[site].as_ref().map(|a| {
            let neighbors = self
                .geometry
                .neighbors(site)
                .iter()
                .enumerate()
                .filter_map(|(d, &j)| {
                    let b = self.sites[j as usize].as_ref()?;
                    let (x, y, _) = self.geometry.xyz(j as usize);
                    Some(NeighborView {
                        x,
                        y,
                        tag: b.tag,
                        strategy: self.strategy(b),
                        related: a.lineage == b.lineage,
                        helped: a.gave[d],
                        helped_by: b.gave[DIRECTIONS - 1 - d],
                    })
                })
                .collect();
            AgentView {
                id: a.id,
                tag: a.tag,
                strategy: self.strategy(a),
                basis: if a.kin_basis { "kin" } else { "tag" },
                ptr: a.ptr,
                given: a.given,
                received: a.received,
                lineage: a.lineage,
                kin_marker: a.family,
                age: self.tick - a.born.min(self.tick),
                neighbors,
            }
        });
        Ok(EthnoInspection {
            site: Site { x, y },
            agent,
        })
    }
}

/// Tags 0–3 in the Java's colours (blue, red, green, yellow), then hues a
/// golden angle apart.
pub fn tag_color(tag: u32) -> Rgb {
    const FIRST: [Rgb; 4] = [BLUE, RED, LENDER, BOTH];
    if let Some(&c) = FIRST.get(tag as usize) {
        return c;
    }
    let hue = (f64::from(tag) * 0.618_033_988_75).fract() * 6.0;
    let (sector, f) = (hue.floor() as u32, hue.fract());
    let (hi, lo) = (230.0, 70.0);
    let up = lo + (hi - lo) * f;
    let down = hi - (hi - lo) * f;
    let [r, g, b] = match sector {
        0 => [hi, up, lo],
        1 => [down, hi, lo],
        2 => [lo, hi, up],
        3 => [lo, down, hi],
        4 => [up, lo, hi],
        _ => [hi, lo, down],
    };
    [r as u8, g as u8, b as u8]
}

/// A colour hashed from a lineage id (splitmix64), kept away from black.
pub fn lineage_color(lineage: u64) -> Rgb {
    let mut z = lineage.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^= z >> 31;
    std::array::from_fn(|k| 64 + ((z >> (8 * k)) as u8 as u32 * 191 / 255) as u8)
}

impl Model for EthnoWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Ethno(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        EthnoWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        EthnoWorld::population(self)
    }

    /// FNV-1a over the tick, the id counter and every site's agent.
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
        for site in &self.sites {
            match site {
                None => eat(u64::MAX),
                Some(a) => {
                    eat(a.id);
                    eat(a.help);
                    eat(u64::from(a.tag) | u64::from(a.kin_basis) << 32);
                    eat(a.lineage);
                    eat(a.family);
                    eat(a.born);
                    eat(a.ptr.to_bits());
                    eat(u64::from(a.given) << 32 | u64::from(a.received));
                }
            }
        }
        h
    }

    fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.width)
    }

    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: EthnoMode = mode.parse()?;
        buf.clear();
        buf.resize(self.sites.len() * 4, 0);
        for (px, site) in buf.as_chunks_mut::<4>().0.iter_mut().zip(&self.sites) {
            let rgb = site.as_ref().map_or(BACKGROUND, |a| self.color(a, mode));
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
        let mut out =
            String::from("id,x,y,tag,strategy,basis,lineage,kin_marker,age,ptr,given,received\n");
        for (s, a) in self.sites.iter().enumerate() {
            let Some(a) = a else { continue };
            let (x, y, _) = self.geometry.xyz(s);
            writeln!(
                out,
                "{},{x},{y},{},{},{},{},{},{},{},{},{}",
                a.id,
                a.tag,
                self.strategy(a).letter(),
                if a.kin_basis { "kin" } else { "tag" },
                a.lineage,
                a.family,
                self.tick - a.born.min(self.tick),
                a.ptr,
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
        let s = self
            .sites
            .iter()
            .position(|a| a.as_ref().is_some_and(|a| a.id == id))?;
        let (x, y, _) = self.geometry.xyz(s);
        Some((x, y))
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Ethno(next) = next else {
            return Err(wrong_model(ModelKind::Ethno, &next));
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

`presets.rs` (Decision 21; the descriptions carry Decision 12's numbers):

```rust
//! HA06's runs, its appendix's and its code's readings, and its critics'
//! variants. Descriptions quote measurements (release, seeds 1–10, 2,000
//! periods, the mean over periods 1,901–2,000, recorded 2026-09-25).

use super::config::{Discrimination, EthnoConfig, KinBasis, Offspring, PairPlay, Start, Strategy};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut EthnoConfig),
) -> ModelPreset {
    let mut c = EthnoConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Ethno(c),
    }
}

const HA06: &str = "Hammond & Axelrod 2006, J. Conflict Resolution 50";
const HA_APPENDIX: &str = "Hammond & Axelrod 2006, appendix";
const HA_JAVA: &str = "Hammond & Axelrod's Java code (2003)";
const HKS13: &str = "Hartshorn, Kaznatcheev & Shultz 2013, JASSS 16(3)";
const J13: &str = "Jansson 2013, JASSS 16(3)";

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "ha-standard",
            "The standard case",
            HA06,
            "HA06's standard case: an empty 50 × 50 torus, four neighbors each. Each period an immigrant with random traits — one of four colours, a bit for helping its own colour and one for helping others — arrives at a random empty site; every agent's potential to reproduce (PTR) is reset to 12%, and each decides once for each neighbor whether to help it (costing it 1% of PTR, giving the neighbor 3%); in random order each reproduces with probability PTR into an empty neighbor (0.5% mutation per trait); each dies with probability 10%. HA06 Table 1 a: 76.3% ethnocentric, 74.2% cooperation. Measured (seeds 1–10, periods 1,901–2,000): 75.9% ethnocentric (13.5% humanitarian, 8.1% selfish, 2.4% traitorous), 76.0% cooperation, about 1,560 agents. Ethnocentrics take longer to dominate than HA06 say: 57% after 500 periods against their 73.9%.",
            |_| {},
        ),
        preset(
            "ha-figure-1",
            "Figure 1: mutation 0.25%",
            HA06,
            "HA06's Figure 1 and Table 1 f: the standard case with half the mutation rate (0.25%). HA06: 82.8% ethnocentric, 79.8% cooperation. Measured: 83.4% and 80.2% — reproduced.",
            |c| c.mutation = 0.0025,
        ),
        preset(
            "ha-appendix-mutation",
            "The appendix's 5% mutation",
            HA_APPENDIX,
            "HA06's appendix gives MutationRate = 0.05; its text, Table 1, the authors' code, NetLogo and every later paper use 0.005. At 5% the lattice never sorts: 36.0% ethnocentric, 25.8% humanitarian, 22.1% selfish, 16.0% traitorous, 56.4% cooperation — nothing like Table 1 a's 76.3/74.2, so the appendix's figure is a slip.",
            |c| c.mutation = 0.05,
        ),
        preset(
            "ha-appendix-double-play",
            "The appendix's loop: every decision twice",
            HA_APPENDIX,
            "HA06's appendix loops \"for each adjacent neighboring agent N of each existing agent A: A decides whether to donate to N … N decides whether to donate to A\" — read literally, every direction is decided twice a period. The authors' code and NetLogo decide once. Twice: 80.9% ethnocentric, 77.3% cooperation, 1,860 agents — five points above Table 1 a (76.3%), where once gives 75.9%.",
            |c| c.pair_play = PairPlay::Twice,
        ),
        preset(
            "ha-java-five-colors",
            "Five colours (the code's draw)",
            HA_JAVA,
            "The authors' Java draws a tag with Ascape's randomInRange(0, numColors), which includes both ends: \"four\" colours are five (tag 4 drawn black, \"if appears means error\"), \"eight\" are nine. Five colours: 75.4% ethnocentric, 76.1% cooperation — indistinguishable from four (75.9/76.0). The sweep ha-colors runs 2 to 9.",
            |c| c.colors = 5,
        ),
        preset(
            "ha-java-archive",
            "The archived code as it runs",
            HA_JAVA,
            "The authors' archived Java (July 2003) as it runs, not as its memo describes it: the immigration block is commented out and setup fills every site with a random agent, with the five-colour draw. Measured: 77.7% ethnocentric, 77.6% cooperation — the same outcome as the paper's empty start (and 43% ethnocentric by period 100, where the standard case has 35%).",
            |c| {
                c.colors = 5;
                c.start = Start::Random;
                c.immigration = 0.0;
            },
        ),
        preset(
            "ha-egoist-start",
            "A full lattice of egoists",
            HA06,
            "HA06: starting from a full lattice of egoists (help no one) with no immigration, ethnocentrism becomes \"just as dominant\". Measured: 78.6% ethnocentric, 79.9% cooperation — reproduced; mutation alone brings it in: 7% at period 100, 26% at 200, 70% at 500.",
            |c| {
                c.start = Start::Selfish;
                c.immigration = 0.0;
            },
        ),
        preset(
            "ha-cost-2",
            "Cost 2%",
            HA06,
            "Table 1 c: helping costs 2% of PTR. HA06: 61.8% ethnocentric, 56.1% cooperation. Measured: 63.5% ethnocentric (21.6% selfish), but 64.7% cooperation — 8.6 points above HA06's. Compare the colour-blind preset.",
            |c| c.cost = 0.02,
        ),
        preset(
            "ha-cost-2-blind",
            "Cost 2%, colour-blind",
            HA06,
            "HA06: \"when agents are unable to distinguish their own color from others, cooperation in the doubled-cost case falls to 14 percent.\" Blind agents have one bit (help everyone or no one). Measured: 41.8% cooperation (40% humanitarian, 60% selfish) — three times HA06's 14%, which blind agents reach only at a cost of 3% (12.7%). Not reproduced; the ha-cost sweep compares blind and seeing agents at every cost.",
            |c| {
                c.cost = 0.02;
                c.discrimination = Discrimination::None;
            },
        ),
        preset(
            "ha-misperception",
            "10% misperception",
            HA06,
            "HA06: with a 10% chance of misperceiving whether the other has the same colour, \"more than two-thirds\" are ethnocentric. Each decision uses the agent's other bit with probability 0.1 (the authors' noise). Measured: 71.7% ethnocentric, 71.1% cooperation — reproduced.",
            |c| c.misperception = 0.1,
        ),
        preset(
            "ha-each-color",
            "Strategies for each colour",
            HA06,
            "HA06: when agents distinguish all four colours, \"80 percent ethnocentric strategies\". Here each agent has one help bit per colour. Helping one's own colour only: 27.0%; helping one's own colour and refusing at least one other: 84.3% — HA06's 80% matches the looser reading. The Strategy view shows the rest as mixed (67%).",
            |c| c.discrimination = Discrimination::EachColor,
        ),
        preset(
            "jansson-offspring-anywhere",
            "Offspring anywhere",
            J13,
            "Jansson 2013 §3.6: put each offspring on a random empty site instead of next to its parent and the results are \"similar to the null model\" (12% cooperators with one partner a round, 3.4% with four). Measured: 4.5% cooperation; 88.7% selfish, 8.3% ethnocentric, 2.5% traitorous — reproduced: without kin next door, discrimination buys nothing.",
            |c| c.offspring = Offspring::Anywhere,
        ),
        preset(
            "jansson-tag-mutation-30",
            "Tag mutation 30%",
            J13,
            "Jansson 2013 §4.4: raise only the tag's mutation rate, so the tag becomes a deceptive marker of kinship: \"At a marker mutation rate of 30%, altruists surpass ethnocentrics.\" Measured: 44.9% humanitarian against 39.9% ethnocentric — reproduced (at 20%, 32.2 against 54.5). The sweep jansson-tag-mutation runs 0.5% to 90%.",
            |c| c.tag_mutation = Some(0.3),
        ),
        preset(
            "jansson-kin",
            "Kin strategies",
            J13,
            "Jansson 2013 §5.2: agents may discriminate on the tag or on a kin marker naming their family's founder (the marker mutates at 0.5%, the mutant founding a new family); the basis is a bit that mutates like the others (Jansson does not say). Table 5: kin 76.2%, ingroup 16.4%, all 2.8, none 2.0, outgroup 1.3, nonkin 1.3. Measured: kin 52.1%, ingroup (ethnocentric) 26.7%, all 12.0, none 7.4, nonkin 1.0, outgroup 0.7 — kin discriminators win, by far less. With the basis fixed at immigration (jansson-kin-fixed) kin take 65.5% and ingroup 12.6%, nearer Table 5.",
            |c| c.kin_strategies = true,
        ),
        preset(
            "jansson-kin-fixed",
            "Kin strategies, basis fixed",
            J13,
            "Jansson 2013 §5.2's kin discriminators with the other reading of an unstated rule: an agent's basis (tag or kin marker) is drawn at immigration and never mutates, so a lineage stays tag- or kin-discriminating. Table 5: kin 76.2%, ingroup 16.4%, all 2.8, none 2.0, outgroup 1.3, nonkin 1.3. Measured (seeds 1–10): kin 65.5%, ingroup 12.6%, all 13.7, none 6.2, nonkin 1.1, outgroup 0.8 (20 seeds: kin 68.9, ingroup 10.6) — closer to Table 5 than the mutating basis (52.1 / 26.7), though humanitarians stay far above Jansson's 2.8%. The jansson-markers sweep runs both readings.",
            |c| {
                c.kin_strategies = true;
                c.kin_basis = KinBasis::Fixed;
            },
        ),
        preset(
            "hks-no-ethnocentrics",
            "No ethnocentrics (H, S, T)",
            HKS13,
            "HKS13 Study 2: immigrants and mutations may produce only humanitarian, selfish and traitorous strategies (a disallowed immigrant is redrawn, a disallowed mutation ignored). HKS13 Table 3: 1,368 humanitarians, 115 selfish, 150 traitors — the one subset where traitors beat the selfish. Measured: 84.7% humanitarian, 7.0% selfish, 8.4% traitorous (1,384, 114, 136 agents) — reproduced; humanitarians do as well without ethnocentrics as ethnocentrics do with them.",
            |c| {
                c.allowed = vec![
                    Strategy::Humanitarian,
                    Strategy::Selfish,
                    Strategy::Traitorous,
                ]
            },
        ),
    ]
}
```

`mod.rs`:

```rust
//! Ethnocentrism (milestone 14): Hammond & Axelrod, "The Evolution of
//! Ethnocentrism", J. Conflict Resolution 50 (2006), with its appendix's
//! and its archived code's departures, and the variants of Hartshorn,
//! Kaznatcheev & Shultz (JASSS 2013) and Jansson (JASSS 2013), as named
//! switches. See docs/superpowers/specs/2026-09-25-ethnocentrism-design.md.

mod config;
mod presets;
mod stats;
mod world;

pub use config::{
    schema, Discrimination, EthnoConfig, KinBasis, Offspring, PairPlay, Start, Strategy, LIVE,
};
pub use presets::presets;
pub use stats::{EthnoSnapshot, SERIES};
pub use world::{
    lineage_color, tag_color, Agent, AgentView, EthnoInspection, EthnoMode, EthnoWorld,
    NeighborView, Site, ETHNOCENTRIC, HUMANITARIAN, KIN, MIXED, NONKIN, SELFISH, TRAITOROUS,
};
```

- [ ] **Step 3b: The Review Focus tests**

Append inside `crates/sugarscape-core/src/ethno/world.rs`'s `mod tests`, before its closing brace (they use the module's `world`, `put`, `E`, `H`, `mask` helpers and the private `sites`, `empty`, `slot` fields):

```rust
    /// The empty-site list holds exactly the unoccupied sites, each at its slot.
    fn empty_list_is_consistent(w: &EthnoWorld) {
        let unoccupied = w.sites.iter().filter(|a| a.is_none()).count();
        assert_eq!(w.empty.len(), unoccupied);
        for (k, &s) in w.empty.iter().enumerate() {
            assert!(w.sites[s as usize].is_none(), "listed site {s} is occupied");
            assert_eq!(w.slot[s as usize], k as u32);
        }
    }

    #[test]
    fn a_world_that_empties_reports_nan_and_still_draws() {
        let mut w = world(10, |c| c.death = 1.0);
        put(&mut w, (2, 2), 0, H, 1);
        put(&mut w, (2, 3), 0, E, 2);
        w.run(3);
        assert_eq!(w.population(), 0);
        let last = w.stats.latest().unwrap();
        assert!(last.cooperation.is_nan() && last.ethnocentric.is_nan());
        assert!(w.latest_json().contains("\"cooperation\":null"));
        let mut buf = Vec::new();
        for mode in ["strategy", "tag", "lineage", "ptr"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 10 * 10 * 4);
        }
        assert!(w.inspect_json(2, 2).unwrap().contains("\"agent\":null"));
        empty_list_is_consistent(&w);
    }

    #[test]
    fn extreme_colour_counts_run_in_every_discrimination() {
        for colors in [1, 2, 40] {
            for discrimination in [
                Discrimination::SameOther,
                Discrimination::None,
                Discrimination::EachColor,
            ] {
                let mut w = world(12, |c| {
                    c.colors = colors;
                    c.discrimination = discrimination;
                    c.immigration = 3.0;
                    c.mutation = 0.2;
                });
                w.run(60);
                assert!(w.population() > 0, "{colors} {discrimination:?}");
                assert!(w.agents().all(|a| a.tag < colors));
                let bits = w.help_bits();
                assert!(w.agents().all(|a| a.help & !mask(bits) == 0));
                empty_list_is_consistent(&w);
            }
        }
    }

    #[test]
    fn ptr_outside_zero_to_one_only_saturates_the_chance() {
        // PTR 0 never reproduces; PTR above 1 always does (if there is room).
        let mut w = world(10, |c| {
            c.base_ptr = 0.0;
            c.cost = 0.5;
            c.benefit = 0.5;
            c.death = 0.0;
        });
        put(&mut w, (4, 4), 0, H, 1);
        put(&mut w, (4, 5), 0, H, 2);
        w.run(5);
        assert_eq!(w.population(), 2, "PTR 0 − 0.5 + 0.5 = 0: no offspring");
        let mut w = world(10, |c| {
            c.base_ptr = 0.0;
            c.benefit = 5.0;
            c.cost = 0.0;
            c.death = 0.0;
        });
        put(&mut w, (4, 4), 0, H, 1);
        put(&mut w, (4, 5), 0, H, 2);
        w.step();
        assert_eq!(w.population(), 4, "PTR 5 > 1: both reproduce");
        empty_list_is_consistent(&w);
    }

    #[test]
    fn the_empty_list_survives_long_runs_live_changes_and_keyframes() {
        let mut w = world(15, |c| {
            c.immigration = 2.5;
            c.death = 0.2;
        });
        w.run(80);
        empty_list_is_consistent(&w);
        let kept = w.clone();
        w.config.offspring = Offspring::Anywhere;
        w.config.pair_play = PairPlay::Twice;
        w.run(80);
        empty_list_is_consistent(&w);
        let mut back = kept.clone();
        back.run(80);
        let mut again = kept;
        again.run(80);
        assert_eq!(back.fingerprint(), again.fingerprint());
        empty_list_is_consistent(&back);
    }
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --lib ethno && cargo test -p sugarscape-core --lib model:: && cargo test -p sugarscape-core --test golden && cargo test -p sugarscape-core --test checkpoint`
Expected: PASS — 25 ethno tests (6 config, 19 world), the model tests (including `ethno_configs_round_trip_with_their_tag`), every golden entry (the sixteen new ones as Decision 13; if one differs, stop and report), the keyframe tests. Then `cargo test --workspace` — PASS.

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/ethno crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/src/main.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/checkpoint.rs
git commit -m "Evolve Hammond and Axelrod's ethnocentrism as the ethno model kind" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 2: Measured against the sources — book-style tests, seven sweeps, WASM

**Files:**
- Create: `crates/sugarscape-core/tests/ethno.rs`, `sweeps/ha-cost.json`, `sweeps/ha-colors.json`, `sweeps/ha-mutation.json`, `sweeps/ha-immigration.json`, `sweeps/ha-lattice.json`, `sweeps/jansson-tag-mutation.json`, `sweeps/jansson-markers.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 1's presets and `EthnoWorld`, `EthnoConfig`, `Strategy::FOUR`.
- Produces: built-in sweep ids `ha-cost`, `ha-colors`, `ha-mutation`, `ha-immigration`, `ha-lattice`, `jansson-tag-mutation`, `jansson-markers` (in that order, after `rca-population`).

- [ ] **Step 1: The book-style tests**

Create `crates/sugarscape-core/tests/ethno.rs` (every test `#[ignore]`, release; the claims and Decision 12's numbers):

```rust
//! The ethnocentrism model (milestone 14) against Hammond & Axelrod 2006
//! (HA06), its appendix and archived code, Hartshorn, Kaznatcheev & Shultz
//! 2013 (HKS13) and Jansson 2013 (J13). Every claim runs over seeds 1–10
//! (HKS13's Study 1 over 50) in release: `cargo test -p sugarscape-core
//! --release --test ethno -- --ignored --nocapture`. A run's summary is
//! the mean over its last 100 periods (1,901–2,000), averaged over seeds.
//! Thresholds come from the measurements recorded 2026-09-25
//! (docs/superpowers/plans/2026-09-25-ethnocentrism.md, Decision 12); claims
//! that do not hold are pinned as measured.

use std::thread;

use sugarscape_core::ethno::{
    Discrimination, EthnoConfig, EthnoWorld, KinBasis, Offspring, PairPlay, Start, Strategy,
};

/// Runs `c` for `ticks` from seeds 1..=`seeds` in parallel and measures each run.
fn each_seed<T: Send>(
    c: &EthnoConfig,
    seeds: u64,
    ticks: u32,
    f: impl Fn(&EthnoWorld) -> T + Sync,
) -> Vec<T> {
    let mut c = c.clone();
    c.end = c.end.max(ticks);
    let c = &c;
    thread::scope(|s| {
        let handles: Vec<_> = (1..=seeds)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = EthnoWorld::new(c.clone(), seed).unwrap();
                    w.run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

/// The mean of `name` over the run's last 100 periods, skipping undefined ones.
fn last_100(w: &EthnoWorld, name: &str) -> f64 {
    let s = w.stats.series(name).unwrap();
    let v: Vec<f64> = s[s.len() - 100..]
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    v.iter().sum::<f64>() / v.len() as f64
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The seed mean of `name`'s last-100 mean, in percent.
fn pct(c: &EthnoConfig, ticks: u32, name: &str) -> f64 {
    100.0 * mean(&each_seed(c, 10, ticks, |w| last_100(w, name)))
}

/// (ethnocentric %, cooperation %) over seeds 1–10.
fn row(c: &EthnoConfig, ticks: u32) -> (f64, f64) {
    let v = each_seed(c, 10, ticks, |w| {
        (last_100(w, "ethnocentric"), last_100(w, "cooperation"))
    });
    let e = mean(&v.iter().map(|x| x.0).collect::<Vec<_>>());
    let k = mean(&v.iter().map(|x| x.1).collect::<Vec<_>>());
    (100.0 * e, 100.0 * k)
}

fn standard() -> EthnoConfig {
    EthnoConfig::default()
}

fn near(ours: f64, target: f64, within: f64) -> bool {
    (ours - target).abs() <= within
}

#[test]
#[ignore]
fn table_1_as_measured() {
    // (row, config, periods, ours E, ours C, HA06 E, HA06 C). Ours are the
    // seed means (seeds 1–10), recorded 2026-09-25; HA06's s.e. is ~1–2.
    type Edit = fn(&mut EthnoConfig);
    let rows: [(&str, Edit, u32, f64, f64, f64, f64); 13] = [
        ("a standard", |_| {}, 2000, 75.9, 76.0, 76.3, 74.2),
        (
            "b cost 0.5%",
            |c| c.cost = 0.005,
            2000,
            76.0,
            78.5,
            76.0,
            77.8,
        ),
        ("c cost 2%", |c| c.cost = 0.02, 2000, 63.5, 64.7, 61.8, 56.1),
        (
            "d 2 colours",
            |c| c.colors = 2,
            2000,
            68.9,
            78.9,
            69.4,
            78.1,
        ),
        (
            "e 8 colours",
            |c| c.colors = 8,
            2000,
            77.9,
            74.4,
            79.1,
            71.7,
        ),
        (
            "f mutation 0.25%",
            |c| c.mutation = 0.0025,
            2000,
            83.4,
            80.2,
            82.8,
            79.8,
        ),
        (
            "g mutation 1%",
            |c| c.mutation = 0.01,
            2000,
            63.0,
            70.8,
            67.1,
            69.0,
        ),
        (
            "h immigration 0.5",
            |c| c.immigration = 0.5,
            2000,
            77.9,
            77.8,
            77.5,
            75.5,
        ),
        (
            "i immigration 2",
            |c| c.immigration = 2.0,
            2000,
            70.5,
            73.9,
            74.4,
            71.4,
        ),
        ("j 25 × 25", |c| c.width = 25, 2000, 64.4, 70.1, 70.5, 69.9),
        (
            "k 100 × 100",
            |c| c.width = 100,
            2000,
            76.5,
            79.1,
            78.2,
            76.0,
        ),
        ("l 500 periods", |_| {}, 500, 57.3, 78.6, 73.9, 73.4),
        ("m 4,000 periods", |_| {}, 4000, 77.1, 75.9, 77.3, 74.4),
    ];
    let mut within_3 = Vec::new();
    for (name, edit, ticks, our_e, our_c, ha_e, ha_c) in rows {
        let mut c = standard();
        edit(&mut c);
        let (e, k) = row(&c, ticks);
        println!("{name}: E {e:.1} (HA06 {ha_e}), C {k:.1} (HA06 {ha_c})");
        assert!(
            near(e, our_e, 0.06) && near(k, our_c, 0.06),
            "{name}: {e} {k}"
        );
        if near(e, ha_e, 3.0) && near(k, ha_c, 3.0) {
            within_3.push(&name[..1]);
        }
    }
    // Both columns within 3 points of HA06's: a, b, d, e, f, h and m. Not
    // reproduced: c (cooperation 64.7 against 56.1), g, i and j (too few
    // ethnocentrics: 63.0, 70.5, 64.4 against 67.1, 74.4, 70.5), k
    // (cooperation 79.1 against 76.0) and l (57.3% ethnocentric after 500
    // periods against 73.9%: our ethnocentrics take longer to dominate).
    assert_eq!(within_3, ["a", "b", "d", "e", "f", "h", "m"]);
}

#[test]
#[ignore]
fn table_1_cannot_tell_four_colours_from_five() {
    // The Java draws five tags for "four" (and nine for "eight"). Rows d/a/e
    // against 2/4/8 colours: 68.9/75.9/77.9; against 2/5/9: 68.9/75.4/81.8;
    // HA06: 69.4/76.3/79.1. Both within 2.7 points of every row.
    let e = |colors| {
        pct(
            &EthnoConfig {
                colors,
                ..standard()
            },
            2000,
            "ethnocentric",
        )
    };
    let (two, four, five, eight, nine) = (e(2), e(4), e(5), e(8), e(9));
    println!("2: {two:.1} 4: {four:.1} 5: {five:.1} 8: {eight:.1} 9: {nine:.1}");
    for (ours, ha) in [
        (two, 69.4),
        (four, 76.3),
        (five, 76.3),
        (eight, 79.1),
        (nine, 79.1),
    ] {
        assert!(near(ours, ha, 2.8), "{ours} vs {ha}");
    }
    assert!(near(five, 75.4, 0.06) && near(nine, 81.8, 0.06));
}

#[test]
#[ignore]
fn the_appendixs_mutation_rate_is_not_table_1s() {
    // HA06's appendix: MutationRate = 0.05. It gives 36% ethnocentric and
    // 56% cooperation, far from Table 1 a's 76.3/74.2; 0.005 gives 75.9/76.0.
    let (e, k) = row(
        &EthnoConfig {
            mutation: 0.05,
            ..standard()
        },
        2000,
    );
    println!("mutation 0.05: E {e:.1} C {k:.1}");
    assert!(near(e, 36.0, 0.06) && near(k, 56.4, 0.06));
}

#[test]
#[ignore]
fn the_appendixs_double_play_raises_ethnocentrism() {
    // Every decision twice: 80.9% ethnocentric (77.3% cooperation), against
    // 75.9 once — five points above Table 1 a, so the text's once fits better.
    let (e, k) = row(
        &EthnoConfig {
            pair_play: PairPlay::Twice,
            ..standard()
        },
        2000,
    );
    println!("twice: E {e:.1} C {k:.1}");
    assert!(near(e, 80.9, 0.06) && near(k, 77.3, 0.06));
}

#[test]
#[ignore]
fn the_archived_code_lands_where_the_paper_does() {
    // The archive as it runs — five colours, a full random start, no
    // immigration — 77.7% ethnocentric, 77.6% cooperation: the same regime.
    let c = EthnoConfig {
        colors: 5,
        start: Start::Random,
        immigration: 0.0,
        ..standard()
    };
    let (e, k) = row(&c, 2000);
    println!("archive: E {e:.1} C {k:.1}");
    assert!(near(e, 77.7, 0.06) && near(k, 77.6, 0.06));
}

#[test]
#[ignore]
fn a_lattice_of_egoists_becomes_just_as_ethnocentric() {
    // HA06: "just as dominant". Measured: 78.6% (7% at period 100, 26% at 200, 70% at 500).
    let c = EthnoConfig {
        start: Start::Selfish,
        immigration: 0.0,
        ..standard()
    };
    let e = pct(&c, 2000, "ethnocentric");
    println!("egoists: E {e:.1}");
    assert!(near(e, 78.6, 0.06) && e > 70.0);
}

#[test]
#[ignore]
fn each_colour_strategies_are_ethnocentric_only_in_a_loose_sense() {
    // HA06: "80 percent ethnocentric strategies". Helping one's own colour
    // only: 27.0%. Helping one's own colour and refusing at least one
    // other: 84.3%, which is HA06's number.
    let c = EthnoConfig {
        discrimination: Discrimination::EachColor,
        ..standard()
    };
    let v = each_seed(&c, 10, 1900, |w| {
        let mut w = w.clone();
        let (mut strict, mut loose) = (0.0, 0.0);
        for _ in 0..100 {
            w.step();
            let n = w.population() as f64;
            let all = (1u64 << w.config.colors) - 1;
            strict += w.agents().filter(|a| a.help == 1 << a.tag).count() as f64 / n;
            loose += w
                .agents()
                .filter(|a| a.help & (1 << a.tag) != 0 && a.help != all)
                .count() as f64
                / n;
        }
        (strict / 100.0, loose / 100.0)
    });
    let strict = 100.0 * mean(&v.iter().map(|x| x.0).collect::<Vec<_>>());
    let loose = 100.0 * mean(&v.iter().map(|x| x.1).collect::<Vec<_>>());
    println!("each colour: own only {strict:.1}, own and not all {loose:.1}");
    assert!(near(strict, 27.0, 0.06) && near(loose, 84.3, 0.06));
}

#[test]
#[ignore]
fn misperception_keeps_two_thirds_ethnocentric() {
    // HA06: "more than two-thirds". Measured 71.7%.
    let e = pct(
        &EthnoConfig {
            misperception: 0.1,
            ..standard()
        },
        2000,
        "ethnocentric",
    );
    println!("misperception: E {e:.1}");
    assert!(near(e, 71.7, 0.06) && e > 200.0 / 3.0);
}

#[test]
#[ignore]
fn blind_agents_at_doubled_cost_cooperate_far_more_than_14_percent() {
    // HA06: 56% cooperation discriminating, 14% colour-blind, at cost 2%.
    // Measured: 64.7% and 41.8%; blind cooperation reaches 14% only at cost 3% (12.7%).
    // Other readings at cost 2% (recorded, not switches): one colour with
    // two bits 40.2; a coin per decision (misperception 0.5) 44.9; blind
    // deciding twice 24.5; blind with benefit halved 11.6 (but seeing agents
    // then cooperate 17.7, not 56).
    let c = |discrimination, cost| EthnoConfig {
        discrimination,
        cost,
        ..standard()
    };
    let seeing = pct(&c(Discrimination::SameOther, 0.02), 2000, "cooperation");
    let blind = pct(&c(Discrimination::None, 0.02), 2000, "cooperation");
    let blind_3 = pct(&c(Discrimination::None, 0.03), 2000, "cooperation");
    println!("cost 2%: seeing {seeing:.1}, blind {blind:.1}; blind at 3%: {blind_3:.1}");
    assert!(near(seeing, 64.7, 0.06) && near(blind, 41.8, 0.06) && near(blind_3, 12.7, 0.06));
    assert!(blind < seeing && blind > 30.0);
}

/// HKS13's chi-square dominance at one cycle (p < .01): the index (E, H, S,
/// T) of a strategy that both beats uniform (3 df) and beats the runner-up
/// (1 df).
fn dominant(counts: [f64; 4]) -> Option<usize> {
    let n: f64 = counts.iter().sum();
    if n < 1.0 {
        return None;
    }
    let e = n / 4.0;
    let chi3: f64 = counts.iter().map(|o| (o - e).powi(2) / e).sum();
    let mut order = [0, 1, 2, 3];
    order.sort_by(|&a, &b| counts[b].total_cmp(&counts[a]));
    let (a, b) = (counts[order[0]], counts[order[1]]);
    let chi1 = if a + b > 0.0 {
        (a - b).powi(2) / (a + b)
    } else {
        0.0
    };
    (chi3 > 11.345 && chi1 > 6.635).then_some(order[0])
}

/// Each cycle's dominant strategy (index 0 = the start).
fn dominance(w: &EthnoWorld) -> Vec<Option<usize>> {
    let names = ["ethnocentric", "humanitarian", "selfish", "traitorous"];
    let pop = w.stats.series("population").unwrap();
    let shares: Vec<Vec<f64>> = names.iter().map(|n| w.stats.series(n).unwrap()).collect();
    (0..pop.len())
        .map(|t| {
            if pop[t] == 0.0 {
                return None;
            }
            dominant(std::array::from_fn(|k| (shares[k][t] * pop[t]).round()))
        })
        .collect()
}

#[test]
#[ignore]
fn hks13_ethnocentrics_dominate_from_about_period_300() {
    // HKS13 Study 1 (50 worlds, 1,000 cycles): final shares .08 selfish,
    // .02 traitorous, .73 ethnocentric, .17 humanitarian; ethnocentric
    // dominance "at around 300" as the population saturates just under
    // 1,600. Measured: 7.7/2.6/72.4/17.3; population 1,568; E dominant
    // (chi-square, p < .01) for 100 cycles running from a median of cycle 282.
    let v = each_seed(&standard(), 50, 1000, |w| {
        let dom = dominance(w);
        let onset = (1..=901).find(|&t| (t..t + 100).all(|u| dom[u] == Some(0)));
        let shares = [
            "selfish",
            "traitorous",
            "ethnocentric",
            "humanitarian",
            "population",
        ]
        .map(|n| last_100(w, n));
        (onset, shares)
    });
    let share = |k: usize| mean(&v.iter().map(|x| x.1[k]).collect::<Vec<_>>());
    let (s, t, e, h, pop) = (share(0), share(1), share(2), share(3), share(4));
    println!("final S {s:.3} T {t:.3} E {e:.3} H {h:.3}, population {pop:.0}");
    for (ours, hks) in [(s, 0.08), (t, 0.02), (e, 0.73), (h, 0.17)] {
        assert!(near(ours, hks, 0.01), "{ours} vs {hks}");
    }
    assert!(pop > 1500.0 && pop < 1600.0);
    let mut onsets: Vec<usize> = v.iter().map(|x| x.0.expect("every world")).collect();
    onsets.sort_unstable();
    println!("onsets {onsets:?}");
    assert_eq!(onsets[25], 282);
}

#[test]
#[ignore]
fn hks13_early_humanitarian_dominance_in_a_third_of_worlds() {
    // HKS13 Study 3: of 50 worlds, 16 early humanitarian dominance, 16 early
    // ethnocentric, 18 strong competition. Our rule (their chi-square
    // tests, cycles 1–300): humanitarian if H dominates for 50 cycles
    // running and in more cycles than E; ethnocentric if E dominates in at
    // least 150 cycles and more than H; else competition. Measured 17/18/15.
    let v = each_seed(&standard(), 50, 300, |w| {
        let dom = dominance(w);
        let (mut run, mut best) = (0, 0);
        for d in &dom[1..=300] {
            run = if *d == Some(1) { run + 1 } else { 0 };
            best = best.max(run);
        }
        let h = dom[1..=300].iter().filter(|d| **d == Some(1)).count();
        let e = dom[1..=300].iter().filter(|d| **d == Some(0)).count();
        if best >= 50 && h > e {
            0
        } else if e >= 150 && e > h {
            1
        } else {
            2
        }
    });
    let count = |k| v.iter().filter(|&&c| c == k).count();
    println!("H {} E {} competition {}", count(0), count(1), count(2));
    assert_eq!((count(0), count(1), count(2)), (17, 18, 15));
}

#[test]
#[ignore]
fn hks13_study_2_orders_hold_in_every_subset() {
    // Table 3 (10 worlds, last 100 of 2,000): E > H > S > T in every subset
    // but HST, where T beats S. Measured: every order holds; the full set's
    // counts 1184/211/127/38 against 1183/229/123/47 (traitors 20% short).
    let four = Strategy::FOUR;
    let names = ["ethnocentric", "humanitarian", "selfish", "traitorous"];
    for mask in 1u32..16 {
        let allowed: Vec<Strategy> = (0..4)
            .filter(|k| mask & (1 << k) != 0)
            .map(|k| four[k])
            .collect();
        let c = EthnoConfig {
            allowed: allowed.clone(),
            ..standard()
        };
        let v = each_seed(&c, 10, 2000, |w| {
            let pop = last_100(w, "population");
            names.map(|n| last_100(w, n) * pop)
        });
        let counts: Vec<f64> = (0..4)
            .map(|k| mean(&v.iter().map(|x| x[k]).collect::<Vec<_>>()))
            .collect();
        let mut present: Vec<usize> = (0..4).filter(|k| mask & (1 << k) != 0).collect();
        present.sort_by(|&a, &b| counts[b].total_cmp(&counts[a]));
        let order: String = present.iter().map(|&k| four[k].letter()).collect();
        let expected: String = if mask == 0b1110 {
            "HTS".into()
        } else {
            (0..4)
                .filter(|k| mask & (1 << k) != 0)
                .map(|k| four[k].letter())
                .collect()
        };
        println!("{expected}: {order} {counts:.0?}");
        assert_eq!(order, expected);
        if mask == 0b1111 {
            for (ours, table) in counts.iter().zip([1183.0, 229.0, 123.0, 47.0]) {
                assert!((ours - table).abs() <= 0.25 * table, "{ours} vs {table}");
            }
        }
    }
}

#[test]
#[ignore]
fn j13_offspring_anywhere_is_the_null_model() {
    // J13 §3.6: with offspring on a random site the results are "similar to
    // the null model" (12% cooperators with one partner, 3.4% with four).
    // Measured: 4.5% cooperation; 88.7% selfish, 8.3% ethnocentric.
    let c = EthnoConfig {
        offspring: Offspring::Anywhere,
        ..standard()
    };
    let (k, s) = (pct(&c, 2000, "cooperation"), pct(&c, 2000, "selfish"));
    println!("anywhere: cooperation {k:.1}, selfish {s:.1}");
    assert!(near(k, 4.5, 0.06) && near(s, 88.7, 0.06));
}

#[test]
#[ignore]
fn j13_humanitarians_pass_ethnocentrics_near_30_percent_tag_mutation() {
    // J13 §4.4: "At a marker mutation rate of 30%, altruists surpass
    // ethnocentrics"; at 60% traitors pass ethnocentrics. Measured E/H:
    // 54.5/32.2 at 20%, 39.9/44.9 at 30%; E/T 21.8/20.8 at 60%, 14.7/29.8 at 75%.
    let at = |t| {
        let c = EthnoConfig {
            tag_mutation: Some(t),
            ..standard()
        };
        (
            pct(&c, 2000, "ethnocentric"),
            pct(&c, 2000, "humanitarian"),
            pct(&c, 2000, "traitorous"),
        )
    };
    let (e2, h2, _) = at(0.2);
    let (e3, h3, _) = at(0.3);
    let (e75, _, t75) = at(0.75);
    println!("20%: {e2:.1}/{h2:.1}; 30%: {e3:.1}/{h3:.1}; 75%: E {e75:.1} T {t75:.1}");
    assert!(e2 > h2 && h3 > e3 && t75 > e75);
    assert!(near(e3, 39.9, 0.06) && near(h3, 44.9, 0.06));
}

#[test]
#[ignore]
fn j13_markers_track_kinship() {
    // J13 Table 4: relatives are 74.7% of neighbouring pairs; P(same marker
    // | relative) 95.3%, P(relative | same marker) 89.2%; 89% of an
    // ethnocentric's help goes to kin. Measured 75.4, 95.1, 90.1 and 86.8
    // (all helps).
    let c = standard();
    let m = |n| pct(&c, 2000, n);
    let (r, tr, rt, kh) = (
        m("relatives"),
        m("tag_given_relative"),
        m("relative_given_tag"),
        m("kin_help"),
    );
    println!("relatives {r:.1} p(i|r) {tr:.1} p(r|i) {rt:.1} kin help {kh:.1}");
    assert!(
        near(r, 74.7, 1.0) && near(tr, 95.3, 1.0) && near(rt, 89.2, 1.0) && near(kh, 89.0, 2.5)
    );
}

#[test]
#[ignore]
fn j13_kin_discriminators_win_by_less_than_table_5_unless_the_basis_is_fixed() {
    // J13 Table 5: kin 76.2%, ingroup (E) 16.4%. J13 does not say how the
    // basis is inherited. Mutating like the other bits: kin 52.1%, E 26.7%.
    // Fixed at immigration: kin 65.5%, E 12.6% — nearer Table 5.
    let at = |kin_basis| {
        let c = EthnoConfig {
            kin_strategies: true,
            kin_basis,
            ..standard()
        };
        (pct(&c, 2000, "kin"), pct(&c, 2000, "ethnocentric"))
    };
    let (kin, e) = at(KinBasis::Mutates);
    let (kin_f, e_f) = at(KinBasis::Fixed);
    println!("mutates: kin {kin:.1} E {e:.1}; fixed: kin {kin_f:.1} E {e_f:.1}");
    assert!(near(kin, 52.1, 0.06) && near(e, 26.7, 0.06));
    assert!(near(kin_f, 65.5, 0.06) && near(e_f, 12.6, 0.06));
    assert!((kin_f - 76.2).abs() < (kin - 76.2).abs());
}

#[test]
#[ignore]
fn j13_more_markers_close_the_gap_sooner_than_36() {
    // J13 §5.3: the kin–ingroup gap falls below ten points at 36 markers.
    // Measured (kin − E, seeds 1–10), mutating basis: 25.4 at 4, 11.6 at 8,
    // 7.0 at 12, 14.4 at 16, then −1.3 to 6.7 from 20 to 40 — below ten from
    // 12 or 20, not 36. Fixed basis: 52.9 at 4, 25.5 at 8, then between −1.2
    // and 21.2 (s.e. 8–11) from 12 to 36, and −15.3 at 40; over 20 seeds the
    // gap stays 10–16 at 16–36 but for 1.5 at 24, and is −3.7 at 40.
    let gap = |colors, kin_basis| {
        let c = EthnoConfig {
            kin_strategies: true,
            kin_basis,
            colors,
            ..standard()
        };
        pct(&c, 2000, "kin") - pct(&c, 2000, "ethnocentric")
    };
    let (g4, g20, g36) = (
        gap(4, KinBasis::Mutates),
        gap(20, KinBasis::Mutates),
        gap(36, KinBasis::Mutates),
    );
    let (f4, f40) = (gap(4, KinBasis::Fixed), gap(40, KinBasis::Fixed));
    println!("mutates: 4 {g4:.1} 20 {g20:.1} 36 {g36:.1}; fixed: 4 {f4:.1} 40 {f40:.1}");
    assert!(g4 > 20.0 && g20 < 10.0 && g36 < 10.0);
    assert!(near(f4, 52.9, 0.1) && f40 < 0.0);
}
```

Run: `cargo test -p sugarscape-core --release --test ethno -- --ignored --nocapture`
Expected: PASS, 17 tests, about 25 s on 10 threads; the printed numbers are Decision 12's. If a pinned number differs, stop and report rather than retune.

- [ ] **Step 2: The sweeps**

Create the seven sweep files:

`sweeps/ha-cost.json`:

```json
{
  "name": "Ethnocentrism: the cost of helping, with and without seeing colour (HA06)",
  "description": "HA06's text: at double cost (2%) cooperation is 56% when agents see colour but \"falls to 14 percent\" when they cannot distinguish their own colour from others. The mean cooperation (helps ÷ decisions) over periods 1,901–2,000 against the cost of helping, seeing colour (two bits) and colour-blind (one bit: help all or none). Measured (release, seeds 1–10, recorded 2026-09-25), seeing / blind: 0.5%: 78.5 / 89.0%; 1%: 76.0 / 81.2%; 1.5%: 73.7 / 66.9%; 2%: 64.7 / 41.8%; 2.5%: 49.6 / 19.8%; 3%: 29.8 / 12.7%. Seeing colour helps cooperation only above a cost of about 1.25%; the blind world reaches HA06's 14% only at 3%, and the seeing world's 64.7% at 2% is well above their 56%.",
  "base": {"preset": "ha-standard"},
  "x": {"label": "Cost of helping", "path": "cost", "values": [0.005, 0.01, 0.015, 0.02, 0.025, 0.03]},
  "series": {
    "label": "Agents see",
    "values": [
      {"at": 0, "name": "Same or other colour", "set": {"discrimination": "same_other"}},
      {"at": 1, "name": "Nothing (colour-blind)", "set": {"discrimination": "none"}}
    ]
  },
  "seeds": {"from": 1, "count": 10},
  "ticks": 2000,
  "metric": {"kind": "window_mean", "series": "cooperation", "from": 1901}
}
```

`sweeps/ha-colors.json`:

```json
{
  "name": "Ethnocentrism: how many colours? Table 1 d/a/e against 2/4/8 and 2/5/9",
  "description": "Table 1 d/a/e (2, 4 and 8 colours: 69.4, 76.3 and 79.1% ethnocentric), and the question whether HA-Java's colour draw — randomInRange(0, numColors), inclusive, so five tags for \"four\" and nine for \"eight\" — produced them. The mean ethnocentric share over periods 1,901–2,000. Measured (release, seeds 1–10, recorded 2026-09-25): 2: 68.9%; 3: 76.2; 4: 75.9; 5: 75.4; 6: 76.1; 7: 78.6; 8: 77.9; 9: 81.8. Rows d/a/e fit 2/4/8 (68.9/75.9/77.9) and 2/5/9 (68.9/75.4/81.8) about equally well: the share is flat from 3 to 6 colours and ten seeds cannot tell the two readings apart.",
  "base": {"preset": "ha-standard"},
  "x": {"label": "Colours", "path": "colors", "values": [2, 3, 4, 5, 6, 7, 8, 9]},
  "seeds": {"from": 1, "count": 10},
  "ticks": 2000,
  "metric": {"kind": "window_mean", "series": "ethnocentric", "from": 1901}
}
```

`sweeps/ha-mutation.json`:

```json
{
  "name": "Ethnocentrism: the mutation rate, the text's and the appendix's (HA06)",
  "description": "HA06's text and Table 1 use a mutation rate of 0.5% (0.25% and 1% in rows f and g); its appendix says 5%, and its appendix's interaction loop, read literally, decides every direction twice. The mean ethnocentric share over periods 1,901–2,000 against the mutation rate, deciding once (HA-Java, NetLogo) or twice. Measured (release, seeds 1–10, recorded 2026-09-25), once / twice: 0.25%: 83.4 / 86.6%; 0.5%: 75.9 / 80.9; 1%: 63.0 / 74.6; 2%: 52.4 / 62.9; 5%: 36.0 / 42.7. Table 1 a (76.3%) is the text's 0.5% played once; the appendix's 5% gives about 36–43% and cannot be what the paper ran.",
  "base": {"preset": "ha-standard"},
  "x": {"label": "Mutation rate", "path": "mutation", "values": [0.0025, 0.005, 0.01, 0.02, 0.05]},
  "series": {
    "label": "Decisions per neighbor",
    "values": [
      {"at": 0, "name": "Once (the code)", "set": {"pair_play": "once"}},
      {"at": 1, "name": "Twice (the appendix)", "set": {"pair_play": "twice"}}
    ]
  },
  "seeds": {"from": 1, "count": 10},
  "ticks": 2000,
  "metric": {"kind": "window_mean", "series": "ethnocentric", "from": 1901}
}
```

`sweeps/ha-immigration.json`:

```json
{
  "name": "Ethnocentrism: immigrants per period (Table 1 h/a/i)",
  "description": "Table 1 h/a/i: 0.5, 1 and 2 immigrants a period give 77.5, 76.3 and 74.4% ethnocentric. The mean ethnocentric share over periods 1,901–2,000; half an immigrant is a 50% chance of one (HA-Java's halfImmigrant). Measured (release, seeds 1–10, recorded 2026-09-25): 0.5: 77.9%; 1: 75.9; 1.5: 73.4; 2: 70.5. The trend reproduces; row i comes out four points low.",
  "base": {"preset": "ha-standard"},
  "x": {"label": "Immigrants per period", "path": "immigration", "values": [0.5, 1, 1.5, 2]},
  "seeds": {"from": 1, "count": 10},
  "ticks": 2000,
  "metric": {"kind": "window_mean", "series": "ethnocentric", "from": 1901}
}
```

`sweeps/ha-lattice.json`:

```json
{
  "name": "Ethnocentrism: the lattice's size (Table 1 j/a/k)",
  "description": "Table 1 j/a/k: a 25 × 25, 50 × 50 and 100 × 100 torus give 70.5, 76.3 and 78.2% ethnocentric. The mean ethnocentric share over periods 1,901–2,000. Measured (release, seeds 1–10, recorded 2026-09-25): 25: 64.4%; 50: 75.9; 75: 76.6; 100: 76.5. The small lattice's share is six points below HA06's, with seeds spread from 54 to 75%.",
  "base": {"preset": "ha-standard"},
  "x": {"label": "Width", "path": "width", "values": [25, 50, 75, 100]},
  "seeds": {"from": 1, "count": 10},
  "ticks": 2000,
  "metric": {"kind": "window_mean", "series": "ethnocentric", "from": 1901}
}
```

`sweeps/jansson-tag-mutation.json`:

```json
{
  "name": "Ethnocentrism: a deceptive marker — the tag's mutation rate (Jansson 2013)",
  "description": "Jansson 2013 §4.4: raise the tag's mutation rate alone, making it a deceptive marker of kinship: \"At a marker mutation rate of 30%, altruists surpass ethnocentrics\"; at 60% traitors pass ethnocentrics; at 90% they outnumber altruists. The mean ethnocentric share over periods 1,901–2,000 (the humanitarian share comes from the same runs). Measured (release, seeds 1–10, recorded 2026-09-25), ethnocentric / humanitarian / traitorous: 0.5%: 75.9 / 13.5 / 2.4; 5%: 76.0 / 13.8 / 1.9; 10%: 68.8 / 21.2 / 3.2; 20%: 54.5 / 32.2 / 5.6; 30%: 39.9 / 44.9 / 7.0; 40%: 30.1 / 50.6 / 12.2; 50%: 22.8 / 53.0 / 15.7; 60%: 21.8 / 49.5 / 20.8; 75%: 14.7 / 47.6 / 29.8; 90%: 8.3 / 43.6 / 39.8. Humanitarians pass ethnocentrics between 20% and 30%, as Jansson reports; traitors draw level with ethnocentrics at 60% and pass them by 75%, but still trail humanitarians at 90%.",
  "base": {"preset": "ha-standard"},
  "x": {"label": "Tag mutation rate", "path": "tag_mutation", "values": [0.005, 0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.75, 0.9]},
  "seeds": {"from": 1, "count": 10},
  "ticks": 2000,
  "metric": {"kind": "window_mean", "series": "ethnocentric", "from": 1901}
}
```

`sweeps/jansson-markers.json`:

```json
{
  "name": "Ethnocentrism: kin discriminators against more colours (Jansson 2013)",
  "description": "Jansson 2013 §5.3: with kin discriminators (a kin marker naming the family's founder, mutating at 0.5%), more colours make tags a better proxy for kinship, and the kin–ingroup gap \"goes below ten percentage units at 36 markers\". The mean kin-discriminator share over periods 1,901–2,000 against the number of colours (the ethnocentric share comes from the same runs), with the tag-or-kin basis mutating like a strategy bit or fixed at immigration (Jansson does not say which). Measured (release, seeds 1–10, recorded 2026-09-25), kin / ethnocentric — mutating: 4: 52.1 / 26.7%; 8: 46.7 / 35.1; 12: 42.7 / 35.6; 16: 47.0 / 32.6; 20: 41.3 / 39.6; 24: 43.5 / 37.3; 28: 41.5 / 39.7; 32: 40.1 / 41.4; 36: 44.7 / 37.9; 40: 39.4 / 40.5; fixed: 4: 65.5 / 12.6; 8: 52.1 / 26.6; 12: 48.1 / 31.3; 16: 44.6 / 35.6; 20: 47.1 / 33.8; 24: 41.0 / 42.2; 28: 50.2 / 31.0; 32: 51.3 / 30.1; 36: 40.1 / 38.2; 40: 33.1 / 48.5. Mutating, the gap is below ten points from 12 colours (bar 16) — much sooner than 36. Fixed, it starts twice as wide and drifts down noisily (seed spread 10–18 points): over 20 seeds it stays 10–16 points from 16 to 36 colours (1.5 at 24) and closes only at 40, nearer Jansson's shape; neither reading gives a clean crossing at 36.",
  "base": {"preset": "jansson-kin"},
  "x": {"label": "Colours", "path": "colors", "values": [4, 8, 12, 16, 20, 24, 28, 32, 36, 40]},
  "series": {
    "label": "Tag or kin basis",
    "values": [
      {"at": 0, "name": "Mutates like a strategy bit", "set": {"kin_basis": "mutates"}},
      {"at": 1, "name": "Fixed at immigration", "set": {"kin_basis": "fixed"}}
    ]
  },
  "seeds": {"from": 1, "count": 10},
  "ticks": 2000,
  "metric": {"kind": "window_mean", "series": "kin", "from": 1901}
}
```

In `crates/sugarscape-core/src/sweep.rs`, `BUILTINS` becomes `[Builtin; 27]` with the seven after `rca-population`, `builtin_sweeps_parse_and_validate` lists them, and a test pins the ticks message:

```diff
--- a/crates/sugarscape-core/src/sweep.rs
+++ b/crates/sugarscape-core/src/sweep.rs
@@ -946,7 +951,7 @@
     pub json: &'static str,
 }
 
-const BUILTINS: [Builtin; 20] = [
+const BUILTINS: [Builtin; 27] = [
     Builtin {
         id: "fig-ii-5",
         json: include_str!("../../../sweeps/fig-ii-5.json"),
@@ -1026,7 +1031,35 @@
     Builtin {
         id: "rca-population",
         json: include_str!("../../../sweeps/rca-population.json"),
+    },
+    Builtin {
+        id: "ha-cost",
+        json: include_str!("../../../sweeps/ha-cost.json"),
+    },
+    Builtin {
+        id: "ha-colors",
+        json: include_str!("../../../sweeps/ha-colors.json"),
+    },
+    Builtin {
+        id: "ha-mutation",
+        json: include_str!("../../../sweeps/ha-mutation.json"),
+    },
+    Builtin {
+        id: "ha-immigration",
+        json: include_str!("../../../sweeps/ha-immigration.json"),
+    },
+    Builtin {
+        id: "ha-lattice",
+        json: include_str!("../../../sweeps/ha-lattice.json"),
+    },
+    Builtin {
+        id: "jansson-tag-mutation",
+        json: include_str!("../../../sweeps/jansson-tag-mutation.json"),
     },
+    Builtin {
+        id: "jansson-markers",
+        json: include_str!("../../../sweeps/jansson-markers.json"),
+    },
 ];
 
 /// The built-in sweeps, in display order.
@@ -1785,6 +1818,16 @@
         assert!(e[0].message.starts_with("must be ≤ 30000"), "{e:?}");
         assert!(e[0].message.contains("last generation"), "{e:?}");
         assert!(!e[0].message.contains("Long House Valley"), "{e:?}");
+    }
+
+    #[test]
+    fn an_ethno_sweep_past_the_last_period_names_the_model() {
+        let mut s = builtin("ha-cost").unwrap();
+        s.ticks = 2001;
+        let e = s.points().unwrap_err();
+        assert_eq!(e[0].field, "ticks");
+        assert!(e[0].message.starts_with("must be ≤ 2000"), "{e:?}");
+        assert!(e[0].message.contains("ethnocentrism"), "{e:?}");
     }
 
     #[test]
@@ -1812,7 +1855,14 @@
                 "rca-pairings",
                 "rca-cost",
                 "rca-clones",
-                "rca-population"
+                "rca-population",
+                "ha-cost",
+                "ha-colors",
+                "ha-mutation",
+                "ha-immigration",
+                "ha-lattice",
+                "jansson-tag-mutation",
+                "jansson-markers"
             ]
         );
         for b in builtins() {
```

The CLI's and the WASM's built-in lists gain the same seven ids:

```diff
--- a/crates/sugarscape-cli/tests/cli.rs
+++ b/crates/sugarscape-cli/tests/cli.rs
@@ -68,6 +68,13 @@
         "rca-cost",
         "rca-clones",
         "rca-population",
+        "ha-cost",
+        "ha-colors",
+        "ha-mutation",
+        "ha-immigration",
+        "ha-lattice",
+        "jansson-tag-mutation",
+        "jansson-markers",
     ] {
         assert!(
             text.lines().any(|l| l.starts_with(&format!("{id}\t"))),
```

Run each and compare with the descriptions (Decision 12):

```bash
for id in ha-cost ha-colors ha-mutation ha-immigration ha-lattice jansson-tag-mutation jansson-markers; do cargo run --release -q -p sugarscape-cli -- sweep --builtin $id --quiet --summary-csv /dev/stdout --out /dev/null; done
```

Expected: the means in the descriptions (1–6 s each on 10 threads).

- [ ] **Step 3: WASM portability**

In `crates/sugarscape-wasm/tests/web.rs`, the built-in list and, before `anasazi_overlays_and_inspection`, the ethno fingerprints:

```diff
--- a/crates/sugarscape-wasm/tests/web.rs
+++ b/crates/sugarscape-wasm/tests/web.rs
@@ -270,7 +270,14 @@
             "rca-pairings",
             "rca-cost",
             "rca-clones",
-            "rca-population"
+            "rca-population",
+            "ha-cost",
+            "ha-colors",
+            "ha-mutation",
+            "ha-immigration",
+            "ha-lattice",
+            "jansson-tag-mutation",
+            "jansson-markers"
         ]
     );
     assert!(list[0]["sweep"]["name"]
@@ -685,6 +692,22 @@
 }
 
 #[wasm_bindgen_test]
+fn ethno_sims_match_the_native_golden_entries() {
+    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: PTR sums and
+    // uniform draws are the same bits here as natively.
+    for (id, fp) in [
+        ("ha-standard", "0xf07433e56417f07c"),
+        ("ha-misperception", "0x5567187174fd1c15"),
+        ("jansson-kin", "0x265998639eacfbd0"),
+    ] {
+        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
+        assert_eq!(sim.model_kind(), "ethno");
+        sim.step(200);
+        assert_eq!(sim.fingerprint(), fp, "{id}");
+    }
+}
+
+#[wasm_bindgen_test]
 fn anasazi_overlays_and_inspection() {
     let sim = Sim::new(&preset_json("lhv-published-defaults"), 2, JsValue::NULL).unwrap();
     let n = sim.population() as usize;
```

Run: `cargo test -p sugarscape-core --lib sweep && cargo test -p sugarscape-cli && wasm-pack test --node crates/sugarscape-wasm` — Expected: PASS (35 WASM tests, including `ethno_sims_match_the_native_golden_entries`).

- [ ] **Step 4: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/tests/ethno.rs sweeps/ha-cost.json sweeps/ha-colors.json sweeps/ha-mutation.json sweeps/ha-immigration.json sweeps/ha-lattice.json sweeps/jansson-tag-mutation.json sweeps/jansson-markers.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure ethnocentrism against the paper, its code and its critics, with seven sweeps" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 3: The ethnocentrism model on the page — types, the Rules panel, charts and Inspect

**Files:**
- Create: `web/src/ethno.ts`, `web/src/ethno.test.ts`
- Modify: `crates/sugarscape-core/src/schema.rs`, `crates/sugarscape-core/src/ethno/config.rs`, `web/src/types.ts`, `web/src/models.ts`, `web/src/schema-form.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts`, `web/src/engine.ts`
- Test: `web/src/models.test.ts`, `web/src/schema-form.test.ts`, `web/src/ui/series-data.test.ts`, `web/src/engine.test.ts`

**Interfaces:**
- Consumes: Task 1's schema (`shown_if("kin_strategies", "true")`, `tag_mutation: Option<f64>`), `EthnoInspection` JSON (Decision 17), `SERIES`, the colour modes `strategy`, `tag`, `lineage`, `ptr`.
- Produces: `Param::nullable` (Rust field and builder); `EthnoStrategy`, `EthnoConfig`, `EthnoStats`, `EthnoNeighborView`, `EthnoAgentView`, `EthnoInspection` (types.ts); `ModelKind` gains `'ethno'`, `ColorMode` gains `'tag' | 'ptr'`, `Param.nullable?: true`; `isEthnoView(v, model)`, `MODELS`/`MODEL_LABELS`/`COLOR_MODES`/`MODEL_OVERLAYS` entries, `ticksLeft` for ethno (models.ts); `MODEL_CHARTS.ethno`, `timeAxisLabel('ethno') === 'Period'`; `STRATEGY_TEXT`, `ethnoRows(a: EthnoAgentView, kinStrategies: boolean): [string, string][]` (ethno.ts); `finishedNotice` for ethno.

- [ ] **Step 1: Write the failing tests**

In `crates/sugarscape-core/src/ethno/config.rs`'s tests, before `schema_paths_exist_and_match_what_set_config_allows`:

```rust
    #[test]
    fn only_the_tag_mutation_may_be_empty_on_the_panel() {
        let params = schema();
        let nullable: Vec<_> = params
            .iter()
            .filter(|p| p.nullable)
            .map(|p| p.path)
            .collect();
        assert_eq!(nullable, ["tag_mutation"]);
        let json = serde_json::to_value(&params).unwrap();
        let flagged = json
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p.get("nullable").is_some());
        assert_eq!(flagged.count(), 1);
    }
```

`web/src/models.test.ts`: import `isEthnoView` (after `isCivilView`) and append:

```ts
describe('the ethnocentrism model', () => {
  const ethno = (end: number) => ({ model: 'ethno', end }) as unknown as ModelConfig;

  it('is read by its tag, and its inspections by the model (an empty site is shaped like Schelling’s)', () => {
    expect(modelOf(ethno(2000))).toBe('ethno');
    expect(isSugar(ethno(2000))).toBe(false);
    const empty = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    const agent = { site: { x: 1, y: 2 }, agent: { id: 3, kin_marker: 3, neighbors: [] } } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: { id: 3, neighbors: 2 } } as unknown as AnyInspection;
    expect([empty, agent].map((v) => isEthnoView(v, 'ethno'))).toEqual([true, true]);
    expect([empty, agent, schelling].map((v) => isEthnoView(v, 'schelling'))).toEqual([false, false, false]);
    expect(isEthnoView(schelling, 'ethno')).toBe(false);
    expect(isTagsView(agent) || isRingView(agent) || isSugarView(agent) || isValleyView(agent) || isCivilView(agent) || isSpatialView(agent)).toBe(false);
  });

  it('offers strategy, tag, lineage and PTR colors and no overlays, and is grouped last', () => {
    expect(COLOR_MODES.ethno).toEqual([
      ['strategy', 'Strategy'],
      ['tag', 'Tag'],
      ['lineage', 'Lineage'],
      ['ptr', 'PTR'],
    ]);
    expect(MODEL_OVERLAYS.ethno).toEqual([]);
    const p = (id: string, config: object) => ({ id, name: id, source: '', description: '', config }) as unknown as Preset;
    expect(presetGroups([p('ha', { model: 'ethno' }), p('nm', { model: 'spatial' })]).map((g) => g.label)).toEqual(['Spatial Games', 'Ethnocentrism']);
  });

  it('counts down to its last period, or never with none', () => {
    expect(ticksLeft(ethno(2000), 1990)).toBe(10);
    expect(ticksLeft(ethno(2000), 2005)).toBe(0);
    expect(ticksLeft(ethno(0), 5)).toBe(Infinity);
    expect(finishesUnpredictably(ethno(2000))).toBe(false);
    expect(calendarYear(ethno(2000), 5)).toBeNull();
  });
});
```

`web/src/schema-form.test.ts`: import type `EthnoConfig`; append inside `describe('the schema form', …)`:

```ts
  it('shows a field while a bool field is on (the ethnocentrism model’s kin fields)', () => {
    const p = { path: 'kin_basis', label: 'Tag or kin basis', kind: 'choice', apply: 'reset', group: 'Traits', show_if: { path: 'kin_strategies', equals: 'true' } } as Param;
    const kin = (on: boolean) => ({ model: 'ethno', kin_strategies: on }) as unknown as ModelConfig;
    expect([paramShown(p, kin(true)), paramShown(p, kin(false))]).toEqual([true, false]);
  });

  it('shows a nullable field’s null as an empty box and writes an empty box as null', () => {
    const p = param({ path: 'tag_mutation', kind: 'number', step: 0.005, nullable: true });
    const c = { model: 'ethno', tag_mutation: null, mutation: 0.005 } as unknown as EthnoConfig;
    expect(paramInput(p, c)).toBe('');
    paramEdit(p, '0.3')(c);
    expect(c.tag_mutation).toBe(0.3);
    expect(paramInput(p, c)).toBe('0.3');
    paramEdit(p, ' ')(c);
    expect(c.tag_mutation).toBeNull();
    // A field that is not nullable reads an empty box as a number, as before.
    const r = ring();
    paramEdit(param({ path: 'growback', kind: 'number' }), '')(r);
    expect(r.growback).toBe(0);
  });
```

`web/src/ui/series-data.test.ts`, appended:

```ts
describe('the ethnocentrism model’s charts', () => {
  it('charts strategies, cooperation, population and kin against the period', () => {
    expect(MODEL_CHARTS.ethno.map((c) => c.title)).toEqual(['Strategies', 'Cooperation', 'Population', 'Kin']);
    expect(MODEL_CHARTS.ethno[0].lines.map((l) => [l.key, l.color])).toEqual([
      ['ethnocentric', '--lender'],
      ['humanitarian', '--blue'],
      ['selfish', '--red'],
      ['traitorous', '--both'],
      ['kin', '--c4'],
      ['nonkin', '--c2'],
      ['mixed', '--muted'],
    ]);
    expect(MODEL_CHARTS.ethno.every((c) => !c.shown)).toBe(true);
    expect(timeAxisLabel('ethno')).toBe('Period');
  });
});
```

`web/src/engine.test.ts`, after the tags model's `finishedNotice` expectation:

```ts
    expect(finishedNotice({ model: 'ethno' } as unknown as ModelConfig, 2000)).toBe('This run has reached its last period (2000) — Reset to run it again');
```

Create `web/src/ethno.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { ethnoRows } from './ethno';
import type { EthnoAgentView } from './types';

const agent = (c: Partial<EthnoAgentView>): EthnoAgentView => ({
  id: 40,
  tag: 2,
  strategy: 'E',
  basis: 'tag',
  ptr: 0.14,
  given: 1,
  received: 1,
  lineage: 7,
  kin_marker: 40,
  age: 12,
  neighbors: [
    { x: 5, y: 4, tag: 2, strategy: 'H', related: true, helped: 1, helped_by: 1 },
    { x: 4, y: 5, tag: 0, strategy: 'S', related: false, helped: 0, helped_by: 0 },
  ],
  ...c,
});

describe('ethnoRows', () => {
  it('shows the traits, this period’s PTR and helps, the lineage, and each neighbor with who helped whom', () => {
    expect(ethnoRows(agent({}), false)).toEqual([
      ['Agent', '#40 · tag 2'],
      ['Strategy', 'ethnocentric: helps its own color only'],
      ['PTR', '0.140'],
      ['Helps', 'gave 1 · received 1'],
      ['Lineage', '#7'],
      ['Kin marker', '#40 (founded the family)'],
      ['Age', '12 periods'],
      ['(5, 4)', 'tag 2 · humanitarian · related · helped it, helped by it'],
      ['(4, 5)', 'tag 0 · selfish · unrelated · no help either way'],
    ]);
  });

  it('says what a kin strategist judges by, counts a pair played twice, and names a lone founder', () => {
    const a = agent({
      id: 7,
      strategy: 'kin',
      basis: 'kin',
      lineage: 7,
      kin_marker: 7,
      age: 1,
      neighbors: [{ x: 5, y: 4, tag: 1, strategy: 'nonkin', related: false, helped: 2, helped_by: 0 }],
    });
    expect(ethnoRows(a, true)).toEqual([
      ['Agent', '#7 · tag 2'],
      ['Strategy', 'kin: helps its kin only'],
      ['Judges by', 'its kin marker (kin or not)'],
      ['PTR', '0.140'],
      ['Helps', 'gave 1 · received 1'],
      ['Lineage', '#7 (founded it)'],
      ['Kin marker', '#7 (founded the family)'],
      ['Age', '1 period'],
      ['(5, 4)', 'tag 1 · non-kin · unrelated · helped it twice'],
    ]);
    expect(ethnoRows(agent({ neighbors: [] }), false).at(-1)).toEqual(['Neighbors', 'none']);
    expect(ethnoRows(agent({ basis: 'tag' }), true)[2]).toEqual(['Judges by', 'its tag (same color or not)']);
  });
});
```

Run: `cargo test -p sugarscape-core --lib ethno::config` — Expected: FAIL to compile (`no field nullable on type &Param`).
Run: `(cd web && npm run build && npm test)` — Expected: FAIL (tsc: `'ethno'` is not a `ModelKind`, no `./ethno`, `isEthnoView` missing).

- [ ] **Step 2: The core's nullable field**

`crates/sugarscape-core/src/schema.rs`: after `show_if` in `Param`:

```rust
    /// A number field that may be empty (JSON null): the panel shows null
    /// as an empty box and sends null for one (milestone 14: the
    /// ethnocentrism model's tag mutation).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub nullable: bool,
```

`Param::new` sets `nullable: false,` after `show_if: None,`; before `fn bounded`:

```rust
    /// The same field, which may be empty (null).
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }
```

and the two doc comments become:

```rust
    /// The same field, shown only while the field `path` is `equals` (a
    /// bool as `"true"` or `"false"`).
```

```rust
/// A condition on the config: the field at `path` (a string, or a bool
/// compared as `"true"`/`"false"`) equals `equals`.
```

In `crates/sugarscape-core/src/ethno/config.rs`'s `schema()`, the tag mutation param ends:

```rust
            "Tag mutation rate",
            (0.0, 1.0, 0.005),
            Live,
        )
        .with_help("Empty: the mutation rate")
        .nullable(),
```

Run: `cargo test -p sugarscape-core --lib ethno::config` — Expected: PASS.

- [ ] **Step 3: Types and models**

`web/src/types.ts`: `ModelKind` ends `| 'tags' | 'ethno'`; before `ModelConfig`:

```ts
/** A strategy as Inspect and `allowed` name it (`allowed` lists only E, H, S and T). */
export type EthnoStrategy = 'E' | 'H' | 'S' | 'T' | 'kin' | 'nonkin' | 'mixed';

/** Hammond & Axelrod's ethnocentrism model and its critics' variants (milestone 14). A tick is a period. */
export interface EthnoConfig {
  model: 'ethno';
  width: number;
  colors: number;
  start: 'empty' | 'random' | 'selfish';
  immigration: number;
  base_ptr: number;
  cost: number;
  benefit: number;
  death: number;
  mutation: number;
  /** The tag's own mutation rate; null = `mutation`. */
  tag_mutation: number | null;
  pair_play: 'once' | 'twice';
  discrimination: 'same_other' | 'none' | 'each_color';
  misperception: number;
  offspring: 'adjacent' | 'anywhere';
  /** Not on the Rules panel: presets, files and links set it. */
  allowed: ('E' | 'H' | 'S' | 'T')[];
  kin_strategies: boolean;
  kin_basis: 'mutates' | 'fixed';
  kin_mutation: number;
  /** The last period (0: never). */
  end: number;
  schedule: ScheduledChange[];
}
```

`ModelConfig` ends `| SpatialConfig | EthnoConfig`. In `Param`, `show_if`'s comment and the new field:

```ts
  /** Shown only while the field `path` equals `equals` (a bool as `'true'`/`'false'`): Model II's population fields, the kin fields. */
  show_if?: { path: string; equals: string };
  /** A number field that may be empty: null shows as an empty box, and an empty box sends null. */
  nullable?: true;
```

Before `ModelStats` (which ends `| SpatialStats | EthnoStats`):

```ts
/** A period's statistics. Shares are null on an empty lattice; the interaction ratios null with a zero denominator (and at t = 0). */
export interface EthnoStats {
  tick: number;
  population: number;
  ethnocentric: number | null;
  humanitarian: number | null;
  selfish: number | null;
  traitorous: number | null;
  kin: number | null;
  nonkin: number | null;
  mixed: number | null;
  cooperation: number | null;
  same_tag: number | null;
  relatives: number | null;
  kin_help: number | null;
  tag_given_relative: number | null;
  relative_given_tag: number | null;
}
```

Before `AnyInspection`, and `AnyInspection` itself:

```ts
/** An occupied neighbor of an ethnocentrism agent (up, left, right, down): whether they share a founding immigrant, and this period's helps each way. */
export interface EthnoNeighborView { x: number; y: number; tag: number; strategy: EthnoStrategy; related: boolean; helped: number; helped_by: number }
/** An ethnocentrism agent: its traits, this period's PTR and helps, its lineage, kin marker and age, and its neighbors. */
export interface EthnoAgentView {
  id: number;
  tag: number;
  strategy: EthnoStrategy;
  /** What same/other is judged by: the tag, or (kin strategies) the kin marker. */
  basis: 'tag' | 'kin';
  ptr: number;
  given: number;
  received: number;
  /** The founding immigrant's id. */
  lineage: number;
  /** The family founder's id. */
  kin_marker: number;
  age: number;
  neighbors: EthnoNeighborView[];
}
/** An ethnocentrism site. Empty, it looks exactly like an empty Schelling site: `isEthnoView` asks the model. */
export interface EthnoInspection { site: { x: number; y: number }; agent: EthnoAgentView | null }

/** What a world of any model says about a site. */
export type AnyInspection =
  | Inspection
  | SchellingInspection
  | RingInspection
  | AnasaziInspection
  | CivilInspection
  | TagsInspection
  | SpatialInspection
  | EthnoInspection;
```

`ColorMode`'s comment ends "`count`, `tolerance`, `clones`, or (ethnocentrism) `strategy`, `tag`, `lineage`, `ptr`." and the union ends `| 'clones' | 'tag' | 'ptr';` (`strategy` and `lineage` are already there).

`web/src/models.ts`: the header comment says "milestones 9–14"; import types `EthnoConfig`, `EthnoInspection`; `MODELS` ends `'tags', 'spatial', 'ethno'`; `MODEL_LABELS.ethno = 'Ethnocentrism'`; `modelOf` accepts `|| tag === 'ethno'`; after `isTagsView`:

```ts
/**
 * An ethnocentrism site's inspection (its agent names a kin marker). An empty one
 * (`{ site: { x, y }, agent: null }`) is exactly an empty Schelling site, so the world's model
 * decides as well as the shape.
 */
export function isEthnoView(v: AnyInspection, model: ModelKind): v is EthnoInspection {
  return model === 'ethno' && (v.agent === null || 'kin_marker' in v.agent);
}
```

`ticksLeft`'s comment adds "the ethnocentrism model's last period" and its body, after the tags line:

```ts
  if (modelOf(c) === 'ethno' && (c as EthnoConfig).end > 0) return Math.max(0, (c as EthnoConfig).end - tick);
```

`COLOR_MODES.ethno` and `MODEL_OVERLAYS.ethno`:

```ts
  // Strategy first (the paper's question); the core's mode names.
  ethno: [
    ['strategy', 'Strategy'],
    ['tag', 'Tag'],
    ['lineage', 'Lineage'],
    ['ptr', 'PTR'],
  ],
```

```ts
  ethno: [],
```

`web/src/engine.ts`, `finishedNotice`, after the tags line:

```ts
  if (modelOf(config) === 'ethno') return `This run has reached its last period (${tick}) — Reset to run it again`;
```

- [ ] **Step 4: The Rules panel, charts and Inspect**

`web/src/schema-form.ts` (Decisions 24–25): `paramEdit`'s comment ends "An empty box of a `nullable` field sets null." and its chain gains, before the final `else`:

```ts
    } else if (p.nullable && String(input).trim() === '') {
      setPath(c, p.path, null);
```

`paramShown`:

```ts
/**
 * Whether a field shows in `config`: always, or while its `show_if` field equals its value (a bool
 * field compared as `'true'` or `'false'`).
 */
export function paramShown(p: Param, config: ModelConfig): boolean {
  return !p.show_if || String(getPath(config, p.show_if.path)) === p.show_if.equals;
}
```

and in `paramInput`, after the bool line:

```ts
  // A nullable field's null (the ethnocentrism model's tag mutation: "the mutation rate") is an empty box.
  if (v === null && p.nullable) return '';
```

`web/src/ui/series-data.ts`: `MODEL_CHARTS`'s doc comment ends "the tags model's donation, tolerance, clusters, tags and takeovers; the ethnocentrism model's strategies (in the frame's colors), cooperation, population and kin."; after `tags`:

```ts
  ethno: [
    {
      title: 'Strategies',
      lines: [
        { key: 'ethnocentric', label: 'Ethnocentric', color: '--lender' },
        { key: 'humanitarian', label: 'Humanitarian', color: '--blue' },
        { key: 'selfish', label: 'Selfish', color: '--red' },
        { key: 'traitorous', label: 'Traitorous', color: '--both' },
        { key: 'kin', label: 'Kin', color: '--c4' },
        { key: 'nonkin', label: 'Non-kin', color: '--c2' },
        { key: 'mixed', label: 'Mixed', color: '--muted' },
      ],
      range: [0, 1],
    },
    {
      title: 'Cooperation',
      lines: [
        { key: 'cooperation', label: 'Helps per decision', color: '--c1' },
        { key: 'same_tag', label: 'Decisions toward the same tag', color: '--c3' },
      ],
      range: [0, 1],
    },
    { title: 'Population', lines: [{ key: 'population', label: 'Agents', color: '--c2' }] },
    {
      title: 'Kin',
      lines: [
        { key: 'relatives', label: 'Neighbors related', color: '--muted' },
        { key: 'kin_help', label: 'Helps to relatives', color: '--c4' },
        { key: 'tag_given_relative', label: 'Same tag if related', color: '--c1' },
        { key: 'relative_given_tag', label: 'Related if same tag', color: '--c3' },
      ],
      range: [0, 1],
    },
  ],
```

and

```ts
/** A model's time charts count calendar years (the anasazi's), generations (tags), periods (ethnocentrism, HA06's word) or ticks. */
export function timeAxisLabel(model: ModelKind): string {
  return model === 'anasazi' ? 'Year' : model === 'tags' ? 'Generation' : model === 'ethno' ? 'Period' : 'Tick';
}
```

Create `web/src/ethno.ts`:

```ts
// The ethnocentrism model's pure page helpers (milestone 14): Inspect's rows.
import type { EthnoAgentView, EthnoStrategy } from './types';

/** Each strategy's name and what it does. */
export const STRATEGY_TEXT: Record<EthnoStrategy, [string, string]> = {
  E: ['ethnocentric', 'helps its own color only'],
  H: ['humanitarian', 'helps everyone'],
  S: ['selfish', 'helps no one'],
  T: ['traitorous', 'helps other colors only'],
  kin: ['kin', 'helps its kin only'],
  nonkin: ['non-kin', 'helps non-kin only'],
  mixed: ['mixed', 'helps some colors and not others'],
};

/** "helped it", "helped it twice" (pair play `twice`), or nothing. */
function times(n: number, verb: string): string | null {
  return n === 0 ? null : n === 1 ? verb : n === 2 ? `${verb} twice` : `${verb} ${n} times`;
}

/**
 * An agent's Inspect rows: its tag and strategy (and, with kin strategies, what it judges same and
 * other by), this period's PTR and helps, its lineage, kin marker and age, then each occupied
 * neighbor (up, left, right, down) with its tag, strategy, kinship and who helped whom.
 */
export function ethnoRows(a: EthnoAgentView, kinStrategies: boolean): [string, string][] {
  const [name, does] = STRATEGY_TEXT[a.strategy];
  const rows: [string, string][] = [
    ['Agent', `#${a.id} · tag ${a.tag}`],
    ['Strategy', `${name}: ${does}`],
  ];
  if (kinStrategies) rows.push(['Judges by', a.basis === 'kin' ? 'its kin marker (kin or not)' : 'its tag (same color or not)']);
  rows.push(
    ['PTR', a.ptr.toFixed(3)],
    ['Helps', `gave ${a.given} · received ${a.received}`],
    ['Lineage', a.lineage === a.id ? `#${a.lineage} (founded it)` : `#${a.lineage}`],
    ['Kin marker', a.kin_marker === a.id ? `#${a.kin_marker} (founded the family)` : `#${a.kin_marker}`],
    ['Age', `${a.age} ${a.age === 1 ? 'period' : 'periods'}`],
  );
  if (a.neighbors.length === 0) return [...rows, ['Neighbors', 'none']];
  for (const n of a.neighbors) {
    const help = [times(n.helped, 'helped it'), times(n.helped_by, 'helped by it')].filter((t) => t !== null);
    const parts = [`tag ${n.tag}`, STRATEGY_TEXT[n.strategy][0], n.related ? 'related' : 'unrelated', help.length > 0 ? help.join(', ') : 'no help either way'];
    rows.push([`(${n.x}, ${n.y})`, parts.join(' · ')]);
  }
  return rows;
}
```

`web/src/ui/inspect-panel.ts`: import `ethnoRows` from `../ethno`, `isEthnoView` from `../models`, types `EthnoConfig`, `EthnoInspection`; before `civilRows`:

```ts
  /**
   * An ethnocentrism site and its agent: tag, strategy (and basis), PTR and helps, lineage, kin
   * marker and age, and its neighbors. An agent that died leaves the site's rows alone.
   */
  private ethnoRows(view: EthnoInspection, gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const rows = [row('Site', `(${view.site.x}, ${view.site.y})`)];
    if (gone) return rows;
    if (!view.agent) return [...rows, row('Agent', 'none (an empty site)')];
    const kin = (this.engine.config as EthnoConfig).kin_strategies;
    return [...rows, ...ethnoRows(view.agent, kin).map(([k, v]) => row(k, v))];
  }
```

and in `render` the note and the chain become:

```ts
      const left = isValleyView(view)
        ? `Household #${shown.agentId} is gone: it died or left the valley.`
        : isCivilView(view)
          ? `Agent #${shown.agentId} is gone: killed, or dead of old age.`
          : isEthnoView(view, this.engine.model)
            ? `Agent #${shown.agentId} has died.`
            : `Agent #${shown.agentId} has left.`;
      const note = gone ? [h('p', { class: 'error' }, left)] : [];
      // First: an empty ethnocentrism site is shaped like an empty Schelling site.
      const rows = isEthnoView(view, this.engine.model)
        ? this.ethnoRows(view, gone)
        : isTagsView(view)
          ? this.tagsRows(view)
          : isRingView(view)
            ? this.ringRows(view, gone)
            : isValleyView(view)
              ? this.valleyRows(view, gone)
              : isCivilView(view)
                ? this.civilRows(view, shown.agentId, gone)
                : isSpatialView(view)
                  ? this.spatialRows(view)
                  : this.schellingRows(view, gone);
```

- [ ] **Step 5: Run the tests and commit**

Run: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test -p sugarscape-core --lib` — Expected: PASS.
Run: `(cd web && npm run build && npm test)` — Expected: PASS (determinism.test's "model charts draw only series their model records" now covers `MODEL_CHARTS.ethno` against `config_series_names`).

```bash
git add crates/sugarscape-core/src/schema.rs crates/sugarscape-core/src/ethno/config.rs web/src/types.ts web/src/models.ts web/src/models.test.ts web/src/schema-form.ts web/src/schema-form.test.ts web/src/ui/series-data.ts web/src/ui/series-data.test.ts web/src/ethno.ts web/src/ethno.test.ts web/src/ui/inspect-panel.ts web/src/engine.ts web/src/engine.test.ts
git commit -m "Show the ethnocentrism model on the page: its Rules panel, colors, charts and Inspect" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): every preset of the **Ethnocentrism** group renders in Strategy, Tag, Lineage and PTR; the Rules panel hides misperception under `each_color` and shows Tag or kin basis and Kin marker mutation only with Kin strategies on; Tag mutation shows empty until set, and clearing it restores the mutation rate; `hks-no-ethnocentrics` keeps its restriction through a live cost change; Inspect on an agent lists its neighbors with who helped whom, and on an empty site says "none (an empty site)"; the four charts show against "Period"; a run stops at 2,000 with the last-period notice.

---

### Task 4: Compare, Experiments and the engine-level checks

**Files:**
- Modify: `web/src/compare-presets.ts`, `web/src/experiments/form.ts`
- Test: `web/src/compare-presets.test.ts`, `web/src/experiments/form.test.ts`, `web/src/determinism.test.ts`

**Interfaces:**
- Consumes: Task 3's types and `isEthnoView`; Task 1's presets and golden fingerprints (Decision 13); `model_schemas_json`, `sweep_points` (WASM).
- Produces: the three Compare entries (`ha-four-vs-five`, `ha-adjacent-vs-anywhere`, `ha-tags-vs-kin`); `defaultForm('ethno')`; sixteen `GOLDEN_MODELS` entries.

- [ ] **Step 1: Write the failing tests**

`web/src/compare-presets.test.ts`, inside the top `describe`:

```ts
  it('pairs the ethnocentrism runs the sources disagree on', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['ha-four-vs-five', 'ha-standard', 'ha-java-five-colors', 'Four colors vs five (the Java’s draw) — Ethnocentrism (Compare)']);
    expect(ids).toContainEqual(['ha-adjacent-vs-anywhere', 'ha-standard', 'jansson-offspring-anywhere', 'Next to the parent vs anywhere — Ethnocentrism (Compare)']);
    expect(ids).toContainEqual(['ha-tags-vs-kin', 'ha-standard', 'jansson-kin', 'Tags vs kin — Ethnocentrism (Compare)']);
  });
```

`web/src/experiments/form.test.ts`, after the spatial expectation:

```ts
    expect(defaultForm('ethno')).toMatchObject({
      x: { path: 'cost', values: '0.005:0.03:0.0025' },
      ticks: 2000,
      metric: { kind: 'window_mean', series: 'ethnocentric', from: 1901, to: null },
    });
```

`web/src/determinism.test.ts`: imports — `comparePresetStates, COMPARE_PRESETS` from `./compare-presets`, `defaultForm, formToSweep` from `./experiments/form`, `isEthnoView` beside `modelOf`, `paramShown` from `./schema-form`, types `EthnoConfig`, `EthnoInspection`, `EthnoStats`, `Param`, and `model_schemas_json` from the WASM package. Append to `GOLDEN_MODELS` after `rca-adopt-p1` (Decision 13):

```ts
    ['ha-standard', '0xf07433e56417f07c'],
    ['ha-figure-1', '0x843632b62ddf7a6b'],
    ['ha-appendix-mutation', '0xae8c7eda9113dae8'],
    ['ha-appendix-double-play', '0xac2c2167fec9c326'],
    ['ha-java-five-colors', '0xdc78c1e27b9ab453'],
    ['ha-java-archive', '0xde2cff652c758fe7'],
    ['ha-egoist-start', '0xabfdf5c9e1ccdb45'],
    ['ha-cost-2', '0x41ba53998a8ee613'],
    ['ha-cost-2-blind', '0x699aa05497139005'],
    ['ha-misperception', '0x5567187174fd1c15'],
    ['ha-each-color', '0x9ad570c3ea183419'],
    ['jansson-offspring-anywhere', '0xcad22f8e7abafbfe'],
    ['jansson-tag-mutation-30', '0xf9dbf338238a8f1b'],
    ['jansson-kin', '0x265998639eacfbd0'],
    ['jansson-kin-fixed', '0x3fac090571612879'],
    ['hks-no-ethnocentrics', '0xbe867e7210bad2d2'],
```

and before `describe('civil violence through the engine', …)`:

```ts
describe('the ethnocentrism model through the engine', () => {
  const preset = (id: string) => presets.find((p) => p.id === id)!;
  const create = (id: string, edit: (c: EthnoConfig) => void = () => {}) => {
    const config = structuredClone(preset(id).config) as EthnoConfig;
    edit(config);
    return Engine.create({ config, seed: 1 }, { presets, transport: inline() });
  };
  const schema = (JSON.parse(model_schemas_json()) as Record<string, Param[]>).ethno;
  const field = (path: string) => schema.find((p) => p.path === path)!;

  it('builds a Rules panel without `allowed`, with a nullable tag mutation and the kin fields shown only with kin strategies', () => {
    expect(schema.some((p) => p.path === 'allowed')).toBe(false);
    expect(schema.filter((p) => p.nullable).map((p) => p.path)).toEqual(['tag_mutation']);
    expect(field('misperception').show_if).toEqual({ path: 'discrimination', equals: 'same_other' });
    for (const path of ['kin_basis', 'kin_mutation']) {
      const p = field(path);
      expect([paramShown(p, preset('jansson-kin').config), paramShown(p, preset('ha-standard').config)]).toEqual([true, false]);
    }
  });

  it('keeps a restricted `allowed` through live edits, resets and a share link', async () => {
    const e = await create('hks-no-ethnocentrics');
    const allowed = () => (e.config as EthnoConfig).allowed;
    expect(allowed()).toEqual(['H', 'S', 'T']);
    await e.advance(20);
    expect(await e.applyModelConfig((c) => void ((c as EthnoConfig).cost = 0.012))).toBeNull();
    expect(allowed()).toEqual(['H', 'S', 'T']);
    expect(await e.resetModelWith((c) => void ((c as EthnoConfig).colors = 3))).toBeNull();
    expect(allowed()).toEqual(['H', 'S', 'T']);
    await e.advance(60);
    expect(await e.applyModelConfig((c) => void ((c as EthnoConfig).tag_mutation = 0.3))).toBeNull();
    await e.advance(20);
    expect(await e.applyModelConfig((c) => void ((c as EthnoConfig).tag_mutation = null))).toBeNull();
    expect((e.config as EthnoConfig).tag_mutation).toBeNull();
    await e.advance(20);
    expect((e.latest as EthnoStats).ethnocentric).toBe(0);
    const { session, tick } = await e.session();
    const opened = await Engine.create(await decodeShare(await encodeShare(session)), { presets, transport: inline() });
    await opened.advance(tick);
    expect((opened.config as EthnoConfig).allowed).toEqual(['H', 'S', 'T']);
    expect(await opened.fingerprint()).toBe(await e.fingerprint());
  });

  it('stops at its last period, once, and inspects agents and empty sites', async () => {
    const e = await create('jansson-kin', (c) => (c.end = 300));
    e.setDisplay({ colorMode: 'ptr' });
    let ends = 0;
    e.on('finished', () => ends++);
    await e.advance(400);
    expect([e.tick, e.finished, ends]).toEqual([300, true, 1]);
    expect((e.latest as EthnoStats).population).toBeGreaterThan(0);
    const seen = { agent: 0, empty: 0 };
    for (let x = 0; x < 50; x++) {
      await e.select(x, 25);
      const v = e.inspection!.view;
      expect(isEthnoView(v, e.model)).toBe(true);
      const a = (v as EthnoInspection).agent;
      if (a) {
        seen.agent++;
        expect(e.inspection!.agentId).toBe(a.id);
        expect(a.neighbors.length).toBeLessThanOrEqual(4);
      } else seen.empty++;
    }
    expect(seen.agent).toBeGreaterThan(0);
    expect(seen.empty).toBeGreaterThan(0);
  });

  it('opens its three Compare entries and sweeps its default form', () => {
    for (const id of ['ha-four-vs-five', 'ha-adjacent-vs-anywhere', 'ha-tags-vs-kin']) {
      const entry = COMPARE_PRESETS.find((c) => c.id === id)!;
      expect(comparePresetStates(presets, entry, 1), id).not.toBeNull();
    }
    const { sweep, errors } = formToSweep(defaultForm('ethno'), { preset: 'ha-standard' });
    expect(errors).toEqual([]);
    // Eleven costs × the form's three seeds (the built-in ha-cost runs ten).
    expect(JSON.parse(sweep_points(JSON.stringify(sweep)))).toHaveLength(33);
  });
});
```

Run: `(cd web && npm run build && npm test)` — Expected: FAIL (the Compare entries and `defaultForm('ethno')`; the golden entries and the engine tests already pass on Task 3's code).

- [ ] **Step 2: Implement**

`web/src/compare-presets.ts`, at the end of `COMPARE_PRESETS`:

```ts
  {
    id: 'ha-four-vs-five',
    label: 'Four colors vs five (the Java’s draw) — Ethnocentrism (Compare)',
    a: 'ha-standard',
    b: 'ha-java-five-colors',
  },
  {
    id: 'ha-adjacent-vs-anywhere',
    label: 'Next to the parent vs anywhere — Ethnocentrism (Compare)',
    a: 'ha-standard',
    b: 'jansson-offspring-anywhere',
  },
  {
    id: 'ha-tags-vs-kin',
    label: 'Tags vs kin — Ethnocentrism (Compare)',
    a: 'ha-standard',
    b: 'jansson-kin',
  },
```

`web/src/experiments/form.ts`, `defaultForm`, after the spatial branch:

```ts
  if (model === 'ethno') {
    // HA06's summary (the mean over the last 100 of 2,000 periods) against the cost of helping (the built-in ha-cost).
    return {
      ...form,
      x: { path: 'cost', values: '0.005:0.03:0.0025' },
      ticks: 2000,
      metric: { ...form.metric, kind: 'window_mean', series: 'ethnocentric', from: 1901, to: null },
    };
  }
```

- [ ] **Step 3: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS (44 files, 523 tests): the sixteen ethno fingerprints through the engine, the Rules panel's schema (no `allowed`, `tag_mutation` the only nullable field, the kin fields shown only with kin strategies), `allowed` through edits and a share link, the stop at the last period with Inspect on agents and empty sites, the three Compare entries resolving, and the default sweep's 33 points.

```bash
git add web/src/compare-presets.ts web/src/compare-presets.test.ts web/src/experiments/form.ts web/src/experiments/form.test.ts web/src/determinism.test.ts
git commit -m "Pair the ethnocentrism runs the sources disagree on in Compare, and sweep the cost by default" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): the three Compare entries open side by side (next to the parent vs anywhere: the anywhere world turns red — selfish — within a few hundred periods); Experiments from an ethno world offers cost 0.005–0.03 and runs; keyframes and the timeline restore a `jansson-kin` run exactly; Max speed at `ha-lattice`'s 100 × 100.

---

### Task 5: The survey's ethnocentrism claims

**Files:**
- Create: `survey/src/claims/ethno.rs`
- Modify: `survey/src/claim.rs`, `survey/src/claims/ch6.rs`, `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `crate::runner::model_after`, `crate::claim::{range, greater, equivalent, Claim, Outcome, Source, Verdict}`; `sugarscape_core::ethno::{EthnoConfig, EthnoWorld, SERIES, …}` and `ModelWorld::Ethno`.
- Produces: `claim::all_of` (public, moved from `ch6.rs`); 38 claims (`ha-…`, `hks-…`, `jansson-…`).

- [ ] **Step 1: Share `all_of`**

Move `all_of` from `survey/src/claims/ch6.rs` to `survey/src/claim.rs`, before its tests, made `pub` (body unchanged):

```rust
/// Several judged parts of one statement ("at every vision"): the worst
/// verdict wins (Fails, then Weak, then Untestable, then Holds; Error first).
pub fn all_of(parts: Vec<(String, Outcome)>) -> Outcome {
    let rank = |v: Verdict| match v {
        Verdict::Error => 4,
        Verdict::Fails => 3,
        Verdict::Weak => 2,
        Verdict::Untestable => 1,
        Verdict::Holds => 0,
    };
    let verdict = parts
        .iter()
        .map(|(_, o)| o.verdict)
        .max_by_key(|v| rank(*v))
        .unwrap_or(Verdict::Untestable);
    let measured = parts
        .iter()
        .map(|(label, o)| format!("[{label}: {:?}] {}", o.verdict, o.measured))
        .collect::<Vec<_>>()
        .join(" ");
    let detail = parts
        .iter()
        .filter(|(_, o)| !o.detail.is_empty())
        .map(|(label, o)| format!("[{label}] {}", o.detail))
        .collect::<Vec<_>>()
        .join(" ");
    Outcome { verdict, measured, detail }
}
```

and `ch6.rs` imports it: `use crate::claim::{all_of, equivalent, greater, range, untestable, Claim, Outcome, Source};` (`Verdict` is no longer used there). Do not run `cargo fmt` over the survey crate: it is not fmt-clean and would rewrite ch2–ch6, `main.rs` and `stats.rs`; check the new file alone with `rustfmt --edition 2021 --check src/claims/ethno.rs`.

- [ ] **Step 2: The claims**

Create `survey/src/claims/ethno.rs` (Decision 32):

```rust
//! The ethnocentrism model (milestone 14): Hammond & Axelrod 2006 (its
//! text, its appendix and its archived code), Hartshorn, Kaznatcheev &
//! Shultz 2013 and Jansson 2013, each claim in its source's words. A run is
//! summarized by the mean of each series over its last 100 periods (HA06's
//! summary); HKS13's Studies 1 and 3 run their own 50 worlds (seeds 1–50,
//! 1,000 periods). Runs that several claims share are memoized per process.

use std::sync::{Arc, Mutex, OnceLock};

use sugarscape_core::ethno::{
    Discrimination, EthnoConfig, EthnoWorld, KinBasis, Offspring, PairPlay, Start, Strategy, SERIES,
};
use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{all_of, equivalent, greater, range, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const HA06: &str = "Hammond & Axelrod 2006, J. Conflict Resolution 50";
const HA06_APPENDIX: &str = "Hammond & Axelrod 2006, appendix";
const HA_JAVA: &str = "Hammond & Axelrod's Java code (2003)";
const HKS13: &str = "Hartshorn, Kaznatcheev & Shultz 2013, JASSS 16(3)";
const J13: &str = "Jansson 2013, JASSS 16(3)";
const OURS: &str = "spec 2026-09-25-ethnocentrism-design.md; plan Decision 12";

/// A world's tolerance around a source's mean (Decision 12): two per-world
/// standard deviations of the standard case (its s.e. over ten seeds × √10)
/// — 7 points of ethnocentric share, 3 of cooperation.
const E_TOL: f64 = 0.07;
const C_TOL: f64 = 0.03;

fn ethno(w: &ModelWorld) -> &EthnoWorld {
    match w {
        ModelWorld::Ethno(w) => w,
        _ => unreachable!("an ethnocentrism world"),
    }
}

/// The mean of `name` over the run's last 100 periods, skipping undefined ones.
fn last_100(w: &EthnoWorld, name: &str) -> f64 {
    let s = w.stats.series(name).expect("an ethnocentrism series");
    let v: Vec<f64> = s[s.len() - 100..]
        .iter()
        .copied()
        .filter(|x| x.is_finite())
        .collect();
    v.iter().sum::<f64>() / v.len() as f64
}

/// Each series' last-100 mean, in `SERIES` order.
type Summary = [f64; SERIES.len()];

/// HA06's defaults with `edit` applied, run `ticks` periods from every seed (memoized).
fn summaries(seeds: &[u64], edit: impl FnOnce(&mut EthnoConfig), ticks: u32) -> Arc<Vec<Summary>> {
    type Cache = Mutex<Vec<(String, u32, Vec<u64>, Arc<Vec<Summary>>)>>;
    static CACHE: Cache = Mutex::new(Vec::new());
    let mut c = EthnoConfig::default();
    edit(&mut c);
    c.end = ticks;
    let key = serde_json::to_string(&c).expect("configs serialize");
    if let Some((.., v)) = CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .find(|(k, t, s, _)| *k == key && *t == ticks && s == seeds)
    {
        return v.clone();
    }
    let v = Arc::new(model_after(&ModelConfig::Ethno(c), seeds, ticks, |w| {
        let w = ethno(w);
        std::array::from_fn(|k| last_100(w, SERIES[k]))
    }));
    CACHE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((key, ticks, seeds.to_vec(), v.clone()));
    v
}

/// Each world's last-100 mean of `name`.
fn last(seeds: &[u64], edit: impl FnOnce(&mut EthnoConfig), ticks: u32, name: &str) -> Vec<f64> {
    let k = SERIES
        .iter()
        .position(|n| *n == name)
        .expect("an ethnocentrism series");
    summaries(seeds, edit, ticks).iter().map(|s| s[k]).collect()
}

/// A Table 1 row: each world's ethnocentric share and cooperation within
/// `E_TOL` and `C_TOL` of HA06's (`e`, `c`); the worse verdict wins.
fn table_row(s: &[u64], edit: impl Fn(&mut EthnoConfig), ticks: u32, e: f64, c: f64) -> Outcome {
    let es = last(s, &edit, ticks, "ethnocentric");
    let cs = last(s, &edit, ticks, "cooperation");
    all_of(vec![
        (
            "ethnocentric".into(),
            range(&es, e - E_TOL, e + E_TOL, false),
        ),
        (
            "cooperation".into(),
            range(&cs, c - C_TOL, c + C_TOL, false),
        ),
    ])
}

/// Each-colour strategies over periods 1,901–2,000: (helps its own colour
/// only, helps its own colour and refuses at least one other).
fn each_color(seeds: &[u64]) -> Vec<(f64, f64)> {
    let c = EthnoConfig {
        discrimination: Discrimination::EachColor,
        ..EthnoConfig::default()
    };
    model_after(&ModelConfig::Ethno(c), seeds, 1900, |w| {
        let mut w = ethno(w).clone();
        let (mut strict, mut loose) = (0.0, 0.0);
        for _ in 0..100 {
            w.step();
            let n = w.population() as f64;
            let all = (1u64 << w.config.colors) - 1;
            strict += w.agents().filter(|a| a.help == 1 << a.tag).count() as f64 / n;
            loose += w
                .agents()
                .filter(|a| a.help & (1 << a.tag) != 0 && a.help != all)
                .count() as f64
                / n;
        }
        (strict / 100.0, loose / 100.0)
    })
}

/// HKS13's chi-square dominance at one cycle (p < .01, Decision 21): the
/// index (E, H, S, T) of a strategy that beats a uniform split (3 df) and
/// the runner-up (1 df).
fn dominant(counts: [f64; 4]) -> Option<usize> {
    let n: f64 = counts.iter().sum();
    if n < 1.0 {
        return None;
    }
    let e = n / 4.0;
    let chi3: f64 = counts.iter().map(|o| (o - e).powi(2) / e).sum();
    let mut order = [0, 1, 2, 3];
    order.sort_by(|&a, &b| counts[b].total_cmp(&counts[a]));
    let (a, b) = (counts[order[0]], counts[order[1]]);
    let chi1 = if a + b > 0.0 {
        (a - b).powi(2) / (a + b)
    } else {
        0.0
    };
    (chi3 > 11.345 && chi1 > 6.635).then_some(order[0])
}

/// One of HKS13's 50 worlds: when ethnocentrics came to dominate, its early
/// pattern, and its final (last-100) S, T, E, H shares and population.
struct Hks {
    onset: Option<usize>,
    /// 0 humanitarian, 1 ethnocentric, 2 strong competition (Decision 21).
    pattern: usize,
    last: [f64; 5],
}

/// HKS13 Studies 1 and 3: seeds 1–50, 1,000 periods, the defaults (memoized).
fn hks() -> &'static [Hks] {
    static RUNS: OnceLock<Vec<Hks>> = OnceLock::new();
    RUNS.get_or_init(|| {
        let seeds: Vec<u64> = (1..=50).collect();
        let c = EthnoConfig {
            end: 1000,
            ..EthnoConfig::default()
        };
        model_after(&ModelConfig::Ethno(c), &seeds, 1000, |w| {
            let w = ethno(w);
            let names = ["ethnocentric", "humanitarian", "selfish", "traitorous"];
            let pop = w.stats.series("population").unwrap();
            let shares: Vec<Vec<f64>> = names.iter().map(|n| w.stats.series(n).unwrap()).collect();
            let dom: Vec<Option<usize>> = (0..pop.len())
                .map(|t| {
                    if pop[t] == 0.0 {
                        return None;
                    }
                    dominant(std::array::from_fn(|k| (shares[k][t] * pop[t]).round()))
                })
                .collect();
            let onset = (1..=901).find(|&t| (t..t + 100).all(|u| dom[u] == Some(0)));
            let (mut run, mut best) = (0, 0);
            for d in &dom[1..=300] {
                run = if *d == Some(1) { run + 1 } else { 0 };
                best = best.max(run);
            }
            let h = dom[1..=300].iter().filter(|d| **d == Some(1)).count();
            let e = dom[1..=300].iter().filter(|d| **d == Some(0)).count();
            let pattern = if best >= 50 && h > e {
                0
            } else if e >= 150 && e > h {
                1
            } else {
                2
            };
            let last = [
                "selfish",
                "traitorous",
                "ethnocentric",
                "humanitarian",
                "population",
            ]
            .map(|n| last_100(w, n));
            Hks {
                onset,
                pattern,
                last,
            }
        })
    })
}

/// Counts of categories among `n` worlds against a source's: each within
/// two binomial standard deviations of the source's count holds, within
/// three is weak.
fn counts_near(names: [&str; 3], ours: [usize; 3], theirs: [usize; 3], n: usize) -> Outcome {
    let z = (0..3)
        .map(|k| {
            let p = theirs[k] as f64 / n as f64;
            let sd = (n as f64 * p * (1.0 - p)).sqrt();
            (ours[k] as f64 - theirs[k] as f64).abs() / sd
        })
        .fold(0.0, f64::max);
    let verdict = if z <= 2.0 {
        Verdict::Holds
    } else if z <= 3.0 {
        Verdict::Weak
    } else {
        Verdict::Fails
    };
    let list = |c: [usize; 3]| {
        (0..3)
            .map(|k| format!("{} {}", names[k], c[k]))
            .collect::<Vec<_>>()
            .join(" / ")
    };
    Outcome {
        verdict,
        measured: format!(
            "{} of {n} (source {}); largest gap {z:.2} binomial s.d.",
            list(ours),
            list(theirs)
        ),
        detail: String::new(),
    }
}

/// HKS13 Study 2: the mean number of agents of each strategy (E, H, S, T)
/// over the last 100 of 2,000 periods, per world, with only `allowed`.
fn study_2(s: &[u64], allowed: &[Strategy]) -> [Vec<f64>; 4] {
    let edit = |c: &mut EthnoConfig| c.allowed = allowed.to_vec();
    let pop = last(s, edit, 2000, "population");
    ["ethnocentric", "humanitarian", "selfish", "traitorous"].map(|n| {
        last(s, edit, 2000, n)
            .iter()
            .zip(&pop)
            .map(|(x, p)| x * p)
            .collect()
    })
}

/// Study 2's order in the subset `letters` (e.g. "EHS"): each strategy
/// outnumbers the next, one judged part per adjacent pair.
fn order(s: &[u64], letters: &str) -> Vec<(String, Outcome)> {
    let index = |l: char| "EHST".find(l).expect("E, H, S or T");
    let allowed: Vec<Strategy> = letters.chars().map(|l| Strategy::FOUR[index(l)]).collect();
    let counts = study_2(s, &allowed);
    let l: Vec<char> = letters.chars().collect();
    l.windows(2)
        .map(|p| {
            let (a, b) = (index(p[0]), index(p[1]));
            let (x, y) = (p[0].to_string(), p[1].to_string());
            (
                format!("{letters} {x} > {y}"),
                greater(&counts[a], &counts[b], &x, &y),
            )
        })
        .collect()
}

/// J13's kin minus ingroup (tag-ethnocentric) share per world, with kin
/// strategies and `colors` markers.
fn kin_gap(s: &[u64], colors: u32, kin_basis: KinBasis) -> Vec<f64> {
    let edit = |c: &mut EthnoConfig| {
        c.kin_strategies = true;
        c.kin_basis = kin_basis;
        c.colors = colors;
    };
    let kin = last(s, edit, 2000, "kin");
    let e = last(s, edit, 2000, "ethnocentric");
    kin.iter().zip(&e).map(|(k, e)| k - e).collect()
}

fn median(v: &[f64]) -> f64 {
    let mut v = v.to_vec();
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

pub fn claims() -> Vec<Claim> {
    vec![
        Claim {
            id: "ha-table-1.a",
            item: "ha-standard",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 a (the standard case, last 100 of 2,000 periods): 76.3% ethnocentric, 74.2% of interactions cooperative",
            check: |s| {
                table_row(s, |_| {}, 2000, 0.763, 0.742).with(
                    "Cooperation runs about 2 points above HA06 in most rows (mean +2.4, seeds 1–10); HA-Java's pooled ratio differs from our window mean by far less.",
                )
            },
        },
        Claim {
            id: "ha-table-1.b",
            item: "ha-cost",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 b (cost of giving help 0.5%): 76.0% ethnocentric, 77.8% cooperative",
            check: |s| table_row(s, |c| c.cost = 0.005, 2000, 0.76, 0.778),
        },
        Claim {
            id: "ha-table-1.c",
            item: "ha-cost-2",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 c (cost of giving help 2%): 61.8% ethnocentric, 56.1% cooperative",
            check: |s| {
                table_row(s, |c| c.cost = 0.02, 2000, 0.618, 0.561)
                    .with("Measured (seeds 1–10): 63.5% ethnocentric but 64.7% cooperative, 8.6 points above HA06.")
            },
        },
        Claim {
            id: "ha-table-1.d",
            item: "ha-colors",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 d (2 colors): 69.4% ethnocentric, 78.1% cooperative",
            check: |s| table_row(s, |c| c.colors = 2, 2000, 0.694, 0.781),
        },
        Claim {
            id: "ha-table-1.e",
            item: "ha-colors",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 e (8 colors): 79.1% ethnocentric, 71.7% cooperative",
            check: |s| table_row(s, |c| c.colors = 8, 2000, 0.791, 0.717),
        },
        Claim {
            id: "ha-table-1.f",
            item: "ha-figure-1",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 f (mutation rate 0.25%): 82.8% ethnocentric, 79.8% cooperative",
            check: |s| table_row(s, |c| c.mutation = 0.0025, 2000, 0.828, 0.798),
        },
        Claim {
            id: "ha-table-1.g",
            item: "ha-mutation",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 g (mutation rate 1%): 67.1% ethnocentric, 69.0% cooperative",
            check: |s| {
                table_row(s, |c| c.mutation = 0.01, 2000, 0.671, 0.69)
                    .with("Measured (seeds 1–10): 63.0% ethnocentric, 4.1 points short.")
            },
        },
        Claim {
            id: "ha-table-1.h",
            item: "ha-immigration",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 h (immigration rate 0.5): 77.5% ethnocentric, 75.5% cooperative",
            check: |s| table_row(s, |c| c.immigration = 0.5, 2000, 0.775, 0.755),
        },
        Claim {
            id: "ha-table-1.i",
            item: "ha-immigration",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 i (immigration rate 2): 74.4% ethnocentric, 71.4% cooperative",
            check: |s| {
                table_row(s, |c| c.immigration = 2.0, 2000, 0.744, 0.714)
                    .with("Measured (seeds 1–10): 70.5% ethnocentric, 3.9 points short; 73.9% cooperative.")
            },
        },
        Claim {
            id: "ha-table-1.j",
            item: "ha-lattice",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 j (lattice 25 × 25): 70.5% ethnocentric, 69.9% cooperative",
            check: |s| {
                table_row(s, |c| c.width = 25, 2000, 0.705, 0.699)
                    .with("Measured (seeds 1–10): 64.4% ethnocentric, 6.1 points short, with a wide spread between worlds.")
            },
        },
        Claim {
            id: "ha-table-1.k",
            item: "ha-lattice",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 k (lattice 100 × 100): 78.2% ethnocentric, 76.0% cooperative",
            check: |s| {
                table_row(s, |c| c.width = 100, 2000, 0.782, 0.76)
                    .with("Measured (seeds 1–10): 76.5% ethnocentric; 79.1% cooperative, 3.1 points high, and a large lattice's worlds barely differ.")
            },
        },
        Claim {
            id: "ha-table-1.l",
            item: "ha-standard",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 l (run length 500, the last 100 periods): 73.9% ethnocentric, 73.4% cooperative",
            check: |s| {
                table_row(s, |_| {}, 500, 0.739, 0.734).with(
                    "Measured (seeds 1–10): 57.3% ethnocentric, 16.6 points short: ethnocentrics take over later than HA06's row says (72.4% by period 1,500).",
                )
            },
        },
        Claim {
            id: "ha-table-1.m",
            item: "ha-standard",
            source: Source::Book,
            citation: HA06,
            text: "Table 1 m (\"run length 2,000\", read as 4,000 since the standard is 2,000; the last 100 periods): 77.3% ethnocentric, 74.4% cooperative",
            check: |s| table_row(s, |_| {}, 4000, 0.773, 0.744),
        },
        Claim {
            id: "ha-egoist-start.just-as-dominant",
            item: "ha-egoist-start",
            source: Source::Book,
            citation: HA06,
            text: "starting with a full lattice of egoists and no immigration, ethnocentrism becomes just as dominant: its share equals the standard case's within 5 points",
            check: |s| {
                let egoists = last(s, |c| {
                    c.start = Start::Selfish;
                    c.immigration = 0.0;
                }, 2000, "ethnocentric");
                let standard = last(s, |_| {}, 2000, "ethnocentric");
                equivalent(&egoists, &standard, Some(0.05), "egoist start", "standard").with(
                    "Measured (seeds 1–10): 78.6% against 75.9% — if anything more dominant, reached later (7% at period 100, 70% at 500).",
                )
            },
        },
        Claim {
            id: "ha-each-color.eighty",
            item: "ha-each-color",
            source: Source::Book,
            citation: HA06,
            text: "when agents can distinguish all four colors, the result is 80 percent ethnocentric strategies: 75–85% help their own color only",
            check: |s| {
                let v = each_color(s);
                let strict: Vec<f64> = v.iter().map(|x| x.0).collect();
                let loose: Vec<f64> = v.iter().map(|x| x.1).collect();
                range(&strict, 0.75, 0.85, false).with(&format!(
                    "Helping one's own color and refusing at least one other (the loose reading): median {:.3}.",
                    median(&loose)
                ))
            },
        },
        Claim {
            id: "ha-misperception.two-thirds",
            item: "ha-misperception",
            source: Source::Book,
            citation: HA06,
            text: "with a 10 percent chance of misperceiving whether the other agent has the same color, more than two-thirds of agents are ethnocentric",
            check: |s| {
                let v = last(s, |c| c.misperception = 0.1, 2000, "ethnocentric");
                range(&v, 2.0 / 3.0, 1.0, false)
            },
        },
        Claim {
            id: "ha-cost-2.fifty-six",
            item: "ha-cost-2",
            source: Source::Book,
            citation: HA06,
            text: "when the cost of giving help is doubled, cooperation is 56 percent (within 3 points)",
            check: |s| {
                let v = last(s, |c| c.cost = 0.02, 2000, "cooperation");
                range(&v, 0.56 - C_TOL, 0.56 + C_TOL, false)
            },
        },
        Claim {
            id: "ha-cost-2-blind.fourteen",
            item: "ha-cost-2-blind",
            source: Source::Book,
            citation: HA06,
            text: "when agents are unable to distinguish their own color from others, cooperation in the doubled-cost case falls to 14 percent (within 3 points)",
            check: |s| {
                let v = last(s, |c| {
                    c.cost = 0.02;
                    c.discrimination = Discrimination::None;
                }, 2000, "cooperation");
                range(&v, 0.14 - C_TOL, 0.14 + C_TOL, false).with(
                    "No reading of \"unable to distinguish\" reaches 14% (one colour 40.2%, a coin per decision 44.9%, deciding twice 24.5%); only a harsher game does (cost 3%: 12.7%), and then seeing agents cooperate 29.8%, not 56%.",
                )
            },
        },
        Claim {
            id: "ha-cost-2-blind.falls",
            item: "ha-cost-2-blind",
            source: Source::Book,
            citation: HA06,
            text: "at doubled cost, cooperation falls when agents cannot distinguish colors: seeing agents cooperate more than blind ones",
            check: |s| {
                let seeing = last(s, |c| c.cost = 0.02, 2000, "cooperation");
                let blind = last(s, |c| {
                    c.cost = 0.02;
                    c.discrimination = Discrimination::None;
                }, 2000, "cooperation");
                greater(&seeing, &blind, "seeing", "blind")
            },
        },
        Claim {
            id: "ha-appendix-mutation.table-1",
            item: "ha-appendix-mutation",
            source: Source::Book,
            citation: HA06_APPENDIX,
            text: "the appendix's MutationRate = 0.05 is the standard case of Table 1 a (76.3% ethnocentric, 74.2% cooperative)",
            check: |s| {
                table_row(s, |c| c.mutation = 0.05, 2000, 0.763, 0.742).with(
                    "Measured (seeds 1–10): 36.0% ethnocentric, 56.4% cooperative; the text's 0.5% fits, so the appendix's 5% is a slip.",
                )
            },
        },
        Claim {
            id: "ha-appendix-double-play.table-1",
            item: "ha-appendix-double-play",
            source: Source::Book,
            citation: HA06_APPENDIX,
            text: "the appendix's interaction loop (A decides whether to donate to N, then N to A, for each neighbor N of each agent A — every direction decided twice) is the standard case of Table 1 a (76.3% ethnocentric, 74.2% cooperative)",
            check: |s| {
                table_row(s, |c| c.pair_play = PairPlay::Twice, 2000, 0.763, 0.742).with(
                    "Measured (seeds 1–10): 80.9% ethnocentric, 77.3% cooperative — five points above Table 1 a; the code decides once.",
                )
            },
        },
        Claim {
            id: "ha-java-archive.same-outcome",
            item: "ha-java-archive",
            source: Source::App,
            citation: HA_JAVA,
            text: "the archived Java, run as is (five colors, a full random start, no immigration), lands where the paper does: its ethnocentric share equals the standard case's within 5 points",
            check: |s| {
                let archive = last(s, |c| {
                    c.colors = 5;
                    c.start = Start::Random;
                    c.immigration = 0.0;
                }, 2000, "ethnocentric");
                let standard = last(s, |_| {}, 2000, "ethnocentric");
                equivalent(&archive, &standard, Some(0.05), "archive", "standard")
            },
        },
        Claim {
            id: "ha-colors.four-vs-five",
            item: "ha-colors",
            source: Source::App,
            citation: OURS,
            text: "Table 1 cannot tell four colors from the Java's five: at 4 and 5 colors the ethnocentric share is the same within 3 points",
            check: |s| {
                let four = last(s, |_| {}, 2000, "ethnocentric");
                let five = last(s, |c| c.colors = 5, 2000, "ethnocentric");
                equivalent(&four, &five, Some(0.03), "4 colors", "5 colors")
            },
        },
        Claim {
            id: "hks-study-1.shares",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "final (last 100 of 1,000 cycles) shares .08 selfish, .02 traitorous, .73 ethnocentric, .17 humanitarian, over 50 worlds (each world within two of its standard deviations: 4, 1.4, 10 and 10 points)",
            check: |_| {
                let runs = hks();
                let col = |k: usize| runs.iter().map(|r| r.last[k]).collect::<Vec<f64>>();
                all_of(vec![
                    ("selfish".into(), range(&col(0), 0.04, 0.12, false)),
                    ("traitorous".into(), range(&col(1), 0.006, 0.034, false)),
                    ("ethnocentric".into(), range(&col(2), 0.63, 0.83, false)),
                    ("humanitarian".into(), range(&col(3), 0.07, 0.27, false)),
                ])
            },
        },
        Claim {
            id: "hks-study-1.saturates",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "the population saturates just under 1,600 (1,500–1,600 over the last 100 of 1,000 cycles)",
            check: |_| {
                let v: Vec<f64> = hks().iter().map(|r| r.last[4]).collect();
                range(&v, 1500.0, 1600.0, false)
            },
        },
        Claim {
            id: "hks-study-3.around-300",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "ethnocentric dominance is established at around 300 evolutionary cycles: in each world within 200–400 (the first cycle from which ethnocentrics dominate, by HKS13's chi-square tests, for 100 cycles running)",
            check: |_| {
                let v: Vec<f64> = hks().iter().map(|r| r.onset.map_or(f64::NAN, |t| t as f64)).collect();
                range(&v, 200.0, 400.0, false).with(&format!(
                    "The median world settles at cycle {:.0}; worlds range from {:.0} to {:.0}.",
                    median(&v),
                    v.iter().copied().fold(f64::INFINITY, f64::min),
                    v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                ))
            },
        },
        Claim {
            id: "hks-study-3.early-patterns",
            item: "ha-standard",
            source: Source::Book,
            citation: HKS13,
            text: "of 50 worlds, 16 showed early humanitarian dominance, 16 early ethnocentric dominance and 18 strong early competition (our rule, Decision 21)",
            check: |_| {
                let runs = hks();
                let count = |k| runs.iter().filter(|r| r.pattern == k).count();
                counts_near(
                    ["humanitarian", "ethnocentric", "competition"],
                    [count(0), count(1), count(2)],
                    [16, 16, 18],
                    runs.len(),
                )
            },
        },
        Claim {
            id: "hks-study-2.order",
            item: "hks-no-ethnocentrics",
            source: Source::Book,
            citation: HKS13,
            text: "Table 3: in every subset of strategies but HST, ethnocentric > humanitarian > selfish > traitorous (mean agents over the last 100 of 2,000 cycles)",
            check: |s| {
                let subsets = ["EHST", "EHS", "EHT", "EST", "EH", "ES", "ET", "HS", "HT", "ST"];
                all_of(subsets.iter().flat_map(|l| order(s, l)).collect())
            },
        },
        Claim {
            id: "hks-study-2.hst-reversal",
            item: "hks-no-ethnocentrics",
            source: Source::Book,
            citation: HKS13,
            text: "Table 3: without ethnocentrics (HST), traitorous beats selfish (1,368 humanitarian, 150 traitorous, 115 selfish)",
            check: |s| {
                all_of(order(s, "HTS")).with(
                    "Measured (seeds 1–10): 136 traitorous against 114 selfish (Table 3: 150, 115) — the order holds on average, but worlds overlap.",
                )
            },
        },
        Claim {
            id: "jansson-offspring-anywhere.null-model",
            item: "jansson-offspring-anywhere",
            source: Source::Book,
            citation: J13,
            text: "with offspring placed on a random site the results are similar to the null model: at most 12% of interactions cooperative",
            check: |s| {
                let v = last(s, |c| c.offspring = Offspring::Anywhere, 2000, "cooperation");
                range(&v, 0.0, 0.12, false)
            },
        },
        Claim {
            id: "jansson-tag-mutation.thirty",
            item: "jansson-tag-mutation",
            source: Source::Book,
            citation: J13,
            text: "at a marker mutation rate of 30%, altruists (humanitarians) surpass ethnocentrics",
            check: |s| {
                let h = last(s, |c| c.tag_mutation = Some(0.3), 2000, "humanitarian");
                let e = last(s, |c| c.tag_mutation = Some(0.3), 2000, "ethnocentric");
                greater(&h, &e, "humanitarian", "ethnocentric").with(
                    "Measured (seeds 1–10): 44.9% humanitarian, 39.9% ethnocentric; they cross between 25% and 30% (at 25%: 48.8% ethnocentric, 37.0% humanitarian).",
                )
            },
        },
        Claim {
            id: "jansson-tag-mutation.sixty",
            item: "jansson-tag-mutation",
            source: Source::Book,
            citation: J13,
            text: "at a marker mutation rate of 60%, traitors surpass ethnocentrics",
            check: |s| {
                let t = last(s, |c| c.tag_mutation = Some(0.6), 2000, "traitorous");
                let e = last(s, |c| c.tag_mutation = Some(0.6), 2000, "ethnocentric");
                greater(&t, &e, "traitorous", "ethnocentric").with(
                    "Measured (seeds 1–10): 20.8% traitorous, 21.8% ethnocentric — level at 60%; traitors pass by 75% (29.8 vs 14.7).",
                )
            },
        },
        Claim {
            id: "jansson-tag-mutation.ninety",
            item: "jansson-tag-mutation",
            source: Source::Book,
            citation: J13,
            text: "at a marker mutation rate of 90%, traitors outnumber altruists (humanitarians)",
            check: |s| {
                let t = last(s, |c| c.tag_mutation = Some(0.9), 2000, "traitorous");
                let h = last(s, |c| c.tag_mutation = Some(0.9), 2000, "humanitarian");
                greater(&t, &h, "traitorous", "humanitarian").with(
                    "Measured (seeds 1–10): 39.8% traitorous, 43.6% humanitarian.",
                )
            },
        },
        Claim {
            id: "jansson-standard.table-4",
            item: "ha-standard",
            source: Source::Book,
            citation: J13,
            text: "Table 4: relatives are 74.7% of neighboring pairs, P(same marker | relatives) 95.3%, P(relatives | same marker) 89.2% (each world within two of its standard deviations: 3.8, 2 and 4.4 points)",
            check: |s| {
                let m = |n| last(s, |_| {}, 2000, n);
                all_of(vec![
                    ("relatives".into(), range(&m("relatives"), 0.709, 0.785, false)),
                    ("p(i|r)".into(), range(&m("tag_given_relative"), 0.933, 0.973, false)),
                    ("p(r|i)".into(), range(&m("relative_given_tag"), 0.848, 0.936, false)),
                ])
            },
        },
        Claim {
            id: "jansson-standard.kin-help",
            item: "ha-standard",
            source: Source::Book,
            citation: J13,
            text: "89% of an ethnocentric's donations go to relatives (ours: of all helps; within 4.4 points)",
            check: |s| {
                let v = last(s, |_| {}, 2000, "kin_help");
                range(&v, 0.846, 0.934, false)
            },
        },
        Claim {
            id: "jansson-kin.table-5",
            item: "jansson-kin",
            source: Source::Book,
            citation: J13,
            text: "Table 5: with kin strategies, kin discriminators take 76.2% and ingroup (tag) ethnocentrics 16.4% (each world within two of its standard deviations: 14 points)",
            check: |s| {
                let m = |n| last(s, |c| c.kin_strategies = true, 2000, n);
                all_of(vec![
                    ("kin".into(), range(&m("kin"), 0.622, 0.902, false)),
                    ("ingroup".into(), range(&m("ethnocentric"), 0.024, 0.304, false)),
                ])
                .with("With the basis fixed at immigration (jansson-kin-fixed) kin take 65.5%, ingroup 12.6% (seeds 1–10) — nearer Table 5; J13 does not say how the basis is inherited.")
            },
        },
        Claim {
            id: "jansson-kin-fixed.table-5",
            item: "jansson-kin-fixed",
            source: Source::App,
            citation: OURS,
            text: "Table 5 with the kin basis fixed at immigration (Decision 22): kin 76.2%, ingroup 16.4%, each world within 14 points",
            check: |s| {
                let m = |n| {
                    last(s, |c| {
                        c.kin_strategies = true;
                        c.kin_basis = KinBasis::Fixed;
                    }, 2000, n)
                };
                all_of(vec![
                    ("kin".into(), range(&m("kin"), 0.622, 0.902, false)),
                    ("ingroup".into(), range(&m("ethnocentric"), 0.024, 0.304, false)),
                ])
            },
        },
        Claim {
            id: "jansson-markers.thirty-six",
            item: "jansson-markers",
            source: Source::Book,
            citation: J13,
            text: "the gap between kin and ingroup strategies falls below ten points only at 36 markers: at 20 markers it is still ten points or more",
            check: |s| {
                range(&kin_gap(s, 20, KinBasis::Mutates), 0.1, 1.0, false).with(&format!(
                    "Median gap at 36 markers: {:.3}; with the basis fixed, at 20: {:.3}.",
                    median(&kin_gap(s, 36, KinBasis::Mutates)),
                    median(&kin_gap(s, 20, KinBasis::Fixed))
                ))
            },
        },
    ]
}
```

In `survey/src/claims/mod.rs` add `mod ethno;` after `mod civil;` and `ethno::claims(),` after `civil::claims(),`.

- [ ] **Step 3: Run it**

Run: `(cd survey && rustfmt --edition 2021 --check src/claims/ethno.rs && cargo test && cargo run --release -- --only ha- && cargo run --release -- --only hks- && cargo run --release -- --only jansson-)`
Expected: tests PASS (16); verdicts (20 seeds; 50 for HKS13's Studies 1 and 3) — **Holds 15**: `ha-table-1.b`, `.d`, `.f`, `ha-misperception.two-thirds`, `ha-cost-2-blind.falls`, `ha-java-archive.same-outcome`, `ha-colors.four-vs-five`, `hks-study-1.shares`, `hks-study-1.saturates`, `hks-study-3.early-patterns` (17/18/15), `hks-study-2.order`, `jansson-offspring-anywhere.null-model`, `jansson-standard.table-4`, `jansson-standard.kin-help`, `jansson-kin-fixed.table-5`; **Weak 11**: `ha-table-1.a`, `.e`, `.g`, `.h`, `.i`, `.j`, `.k`, `.m` (cooperation ~2 points high; j also ethnocentric), `ha-egoist-start.just-as-dominant`, `hks-study-2.hst-reversal`, `jansson-tag-mutation.thirty`; **Fails 12**: `ha-table-1.c`, `.l`, `ha-each-color.eighty`, `ha-cost-2.fifty-six`, `ha-cost-2-blind.fourteen`, `ha-appendix-mutation.table-1`, `ha-appendix-double-play.table-1`, `hks-study-3.around-300`, `jansson-tag-mutation.sixty`, `jansson-tag-mutation.ninety`, `jansson-kin.table-5`, `jansson-markers.thirty-six`. Each prefix runs in under 25 s (the slowest claim, `hks-study-2.order`, 15 s). Delete the `survey/out/results-*.json` the runs write (not committed).

- [ ] **Step 4: Commit**

```bash
git add survey/src/claim.rs survey/src/claims/ch6.rs survey/src/claims/ethno.rs survey/src/claims/mod.rs
git commit -m "Survey the ethnocentrism claims: fifteen hold, eleven are weak, twelve fail" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 6: README, roadmap, spec notes and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/superpowers/specs/2026-09-25-ethnocentrism-design.md`

- [ ] **Step 1: The spec's notes** (Decision 33)

- Goal: "as a seventh model kind" → "as an eighth model kind (after the tags model and the spatial games)".
- Config table, after `kin_strategies`:

```markdown
| `kin_basis` | `mutates` | reset | how the basis bit is inherited: `mutates` (a trait mutating at `mutation`) or `fixed` (drawn at immigration, never mutating); J13 does not say (plan Decision 22) |
```

- Choices, item 12:

```markdown
12. **Kin strategies** (J13 does not say how the basis is inherited): a basis bit mutating at `mutation` by default; `kin_basis: fixed` draws it at immigration and never mutates it, which fits Table 5 better (kin 65.5 % against 52.1 %; J13 76.2 %) but is not what the sources say either (plan Decision 22).
```

- Views, the charts line:

```markdown
- **Charts:** **Strategies** (the strategy shares, in the Strategy mode's colours), **Cooperation** (`cooperation`, `same_tag`), **Population**, **Kin** (`relatives`, `kin_help`, `tag_given_relative`, `relative_given_tag`: J13 Table 4's three numbers and §4.3's).
```

- Presets, before `hks-no-ethnocentrics`:

```markdown
| `jansson-kin-fixed` | `kin_strategies: true`, `kin_basis: fixed` | J13 §5.2, the basis fixed (plan Decision 22) |
```

- Experiments: "`jansson-markers`: base `jansson-kin`; x = colours 4 … 40; series `kin_basis` mutates vs fixed; metric `kin`."
- Page, the first bullet becomes two:

```markdown
- An **Ethnocentrism** presets group; the schema panel in groups **Game** (cost, benefit, base PTR, pair play), **Population** (width, start, immigration, death, offspring), **Traits** (colours, discrimination, misperception shown only for `same_other`, kin strategies, kin basis and kin mutation shown only with kin strategies), **Mutation** (mutation, tag mutation — empty means the mutation rate), **Run** (end); the four colour modes; the four charts, against the **period**; Inspect; the three Compare entries; `defaultForm('ethno')` (x = cost 0.005–0.03 by 0.0025, 2,000 periods, window-mean `ethnocentric` over 1,901–2,000).
- `allowed` is not on the panel (a list is not a panel kind; presets, files and links set it and it round-trips through live edits, resets, links and sessions), so "allowed shown only for `same_other` without kin" has no field to hide; validation still rejects a restricted `allowed` with another discrimination or with kin strategies. The panel shows a nullable number (`tag_mutation`'s null) as an empty box and sends an empty box as null (the schema's `nullable`), and `show_if` compares a bool field as `"true"`/`"false"`.
```

- [ ] **Step 2: README**

In **Other artificial societies**, the presets menu's groups end "**Tag Cooperation**, **Spatial Games** and **Ethnocentrism**." After the Spatial games section (before `## Experiments`), write:

```markdown
### Ethnocentrism (Hammond & Axelrod 2006, and its critics)

Hammond and Axelrod's model of in-group favoritism: an empty 50 × 50 torus on which every site has
four neighbors. Each period an immigrant with random traits arrives at a random empty site; its
traits are a tag (one of four colors) and two strategy bits, whether to help an agent of its own
color and whether to help one of another. Every agent's potential to reproduce (PTR) is reset to
12 %; then each agent decides, for each occupied neighbor, whether to help it — helping costs the
helper 1 % of PTR and gives the neighbor 3 %. In random order each agent then reproduces with
probability PTR into an empty neighboring site, if there is one, the offspring copying its parent
with a 0.5 % chance of mutation per trait; finally every agent dies with probability 10 %. An agent
that helps only its own color is ethnocentric (E), one that helps everyone humanitarian (H), one that
helps no one selfish (S) and one that helps only other colors traitorous (T). The paper reports the
mean over the last 100 of 2,000 periods, over ten runs: 76.3 % ethnocentric, 74.2 % of decisions
cooperative (its Table 1 a).

**The paper, its appendix and its code disagree**, and each disagreement is a switch or a preset.
The appendix gives `MutationRate = 0.05` where the text, Table 1, the authors' code and NetLogo's
replication use 0.005: at 5 % the lattice never sorts (36.0 % ethnocentric, 56.4 % cooperative;
`ha-appendix-mutation`), so the appendix's figure is a slip. The appendix's interaction loop ("A
decides whether to donate to N … N decides whether to donate to A", for each neighbor N of each agent
A), read literally, decides every direction twice a period; the code decides once, and twice gives
80.9 % ethnocentric and 77.3 % cooperative, five points above Table 1 a (`pair_play: twice`,
`ha-appendix-double-play`). The archived Java draws a new agent's tag with Ascape's inclusive
`randomInRange(0, 4)`, so "four colors" are five (`ha-java-five-colors`); and its archived main loop
has the immigration block commented out and fills the lattice with random agents at the start, so
the code as archived is a full random start with no immigration (`ha-java-archive`) — contrary to its
own documentation, and it lands on the same outcome as the paper, faster: 77.7 % ethnocentric
(43 % by period 100, against 35 % in the standard case).

The paper leaves several things open; the choices made here are stated here and in the module docs.
Immigrants interact, reproduce and can die in the period they arrive (as NetLogo); offspring do not
reproduce in the period they are born but can die in it (the code and NetLogo agree); a tag mutation
always gives a different color (the code); a fractional immigration rate is a chance of one more
immigrant (the code's `halfImmigrant`); misperception, "misperceiving whether the other agent has the
same color", is per decision, the agent then using its other bit (the code's noise). Agents that
"distinguish all four colors" carry one help bit per color, each mutating like any other trait, and
count as ethnocentric when they help their own color only; agents "unable to distinguish their own
color from others" carry a single bit (help everyone or no one). Each period's statistics are
shares of the agents alive at its end, and cooperation is helps ÷ decisions that period; a run's
summary is the mean over its last 100 periods, as the paper's.

What reproduces, measured (release, seeds 1–10, periods 1,901–2,000 unless noted):

- **Table 1 in both columns within 3 points** for rows a (75.9 % ethnocentric, 76.0 % cooperative),
  b (cost 0.5 %), d (two colors), e (eight), f (mutation 0.25 %, Figure 1: 83.4 / 80.2 against
  82.8 / 79.8), h (immigration 0.5) and m ("run length 2,000", read as 4,000 periods since the
  standard is already 2,000: 77.1 / 75.9 against 77.3 / 74.4).
- **A lattice of egoists** with no immigration becomes "just as dominant" (`ha-egoist-start`): 78.6 %
  ethnocentric, though slowly — 7 % at period 100, 70 % at 500, after the population falls to 786.
- **Misperception 10 %** (`ha-misperception`): 71.7 % ethnocentric, "more than two-thirds".
- **Hartshorn, Kaznatcheev and Shultz (2013)** reproduce almost exactly, over their 50 worlds of
  1,000 cycles: final shares 7.7 % selfish, 2.6 % traitorous, 72.4 % ethnocentric and 17.3 %
  humanitarian (theirs .08 / .02 / .73 / .17) with 1,568 agents ("just under 1,600"); by their
  chi-square tests, 17 worlds show early humanitarian dominance, 18 early ethnocentric dominance and
  15 strong competition (theirs 16 / 16 / 18); and in their Study 2 (only some strategies allowed:
  `allowed`, which presets, files and links set) ethnocentric > humanitarian > selfish > traitorous
  in every subset but HST, where traitors beat the selfish (`hks-no-ethnocentrics`: 84.7 %
  humanitarian, 8.4 % traitorous, 7.0 % selfish), all fifteen orders as their Table 3, most counts
  within 5 %.
- **Jansson (2013)**: with offspring placed anywhere instead of next to the parent
  (`jansson-offspring-anywhere`) cooperation collapses to 4.5 % (88.7 % selfish), "similar to the null
  model"; relatives (a common founding immigrant) are 75.4 % of neighboring pairs, P(same tag |
  relatives) 95.1 % and P(relatives | same tag) 90.1 % (his Table 4: 74.7, 95.3, 89.2), and 86.8 % of
  all help goes to relatives (his 89 % of an ethnocentric's); raising the tag's own mutation rate
  (`tag_mutation`), humanitarians pass ethnocentrics between 25 % and 30 % (44.9 against 39.9 at 30 %,
  `jansson-tag-mutation-30`; he says 30 %).

What does not, or only partly:

- **The color-blind 14 %.** At doubled cost HA06 report 56 % cooperation for agents that see color and
  14 % for agents "unable to distinguish their own color from others". Here seeing agents cooperate
  64.7 % and blind ones (`ha-cost-2-blind`) 41.8 %, three times 14 %. No reading of "unable to
  distinguish" gets there: one color 40.2 %, a coin flip per decision 44.9 %, every decision twice
  24.5 %. Only a harsher game does — cost 3 %, 12.7 % (or the benefit halved, 11.6 %) — and then
  seeing agents fall to 29.8 % (17.7 %), far below 56 %. And blind agents cooperate *more* than seeing
  ones whenever helping is cheap (81.2 % against 76.0 % at the standard cost, 89.0 against 78.5 at
  0.5 %): seeing color helps cooperation only above a cost of about 1.25 %.
- **Ethnocentrics take over later than Table 1 l says.** After 500 periods 57.3 % are ethnocentric,
  not 73.9 %; the last-100 mean is 70.3 % by period 1,000 and 72.4 % by 1,500. Hartshorn, Kaznatcheev
  and Shultz's "around 300 cycles" holds for the median world (282) but worlds range from 21 to 596:
  11 of 50 settle before period 100 and 11 after 400.
- **"80 percent ethnocentric"** with each-color strategies (`ha-each-color`) holds only loosely: 27.0 %
  help their own color alone, while 84.3 % help their own color and refuse at least one other.
- Rows c, g, i, j and k are off by more than 3 points: at cost 2 % cooperation is 64.7 %, not 56.1 %;
  mutation 1 %, immigration 2 and a 25 × 25 lattice leave 4–6 points fewer ethnocentrics (63.0, 70.5
  and 64.4 against 67.1, 74.4 and 70.5); a 100 × 100 lattice cooperates 3.1 points more. Cooperation
  runs about 2 points above HA06 in most rows (mean +2.4).
- **Four colors or five?** Table 1 cannot tell: the ethnocentric share is flat from 3 to 6 colors
  (76.2, 75.9, 75.4, 76.1 %), and rows d, a and e fit 2/4/8 colors and the code's 2/5/9 about equally
  (root-mean-square error over the thirteen rows 5.2 against 5.8 points ethnocentric, 3.3 against
  4.0 cooperative).
- **Jansson's kin discriminators** (`kin_strategies`: a basis bit says whether same and other are
  judged by the tag or by a kin marker naming the family's founder) win, but by far less than his
  Table 5: 52.1 % kin and 26.7 % tag-ethnocentric, against 76.2 % and 16.4 % (`jansson-kin`). He does
  not say how the basis is inherited; fixed at immigration instead of mutating (`kin_basis: fixed`,
  `jansson-kin-fixed`), kin take 65.5 % and tag-ethnocentrics 12.6 %, nearer his table. With more
  colors the kin–tag gap closes, as he says, but below ten points from about 12 colors, not 36; with a
  fixed basis it hovers at 10–16 points from 16 to 36 colors (one dip at 24) and closes near 40. At 60 % tag mutation
  traitors only draw level with ethnocentrics (20.8 against 21.8 %; they pass by 75 %), and at 90 %
  they do not outnumber humanitarians (39.8 against 43.6 %), both of which he says they do.

The survey measures 38 of these claims (20 seeds; 50 for Hartshorn, Kaznatcheev and Shultz's own
worlds): 15 hold, 11 are weak and 12 fail. Most Table 1 rows are weak because their cooperation,
about 2 points high, puts too few worlds within 3 points of HA06's.

Seven built-in sweeps (ten seeds, 2,000 periods, the mean over periods 1,901–2,000): `ha-cost`
(cooperation against the cost of helping, seeing and blind: 78.5 / 89.0 % at 0.5 %, 76.0 / 81.2 % at
1 %, 64.7 / 41.8 % at 2 %, 29.8 / 12.7 % at 3 %), `ha-colors` (the ethnocentric share for 2 to 9
colors: 68.9, 76.2, 75.9, 75.4, 76.1, 78.6, 77.9, 81.8 %), `ha-mutation` (0.25 % to 5 %, once and
twice: 83.4 / 86.6 % down to 36.0 / 42.7 %), `ha-immigration` (0.5 to 2 immigrants: 77.9 down to
70.5 %), `ha-lattice` (25 to 100 wide: 64.4, 75.9, 76.6, 76.5 %), `jansson-tag-mutation` (0.5 % to
90 %: humanitarians pass ethnocentrics between 25 % and 30 %) and `jansson-markers` (kin strategies
with 4 to 40 colors, the basis mutating or fixed).

Agents are drawn by **Strategy** (the default: ethnocentric green, humanitarian blue, selfish red,
traitorous yellow, kin purple, non-kin orange, other each-color patterns gray), **Tag** (the code's
blue, red, green and yellow, then up to 40 hues), **Lineage** (a color per founding immigrant) or
**PTR** (this period's, as heat), with empty sites dark. Inspect shows an agent's tag, strategy (and,
with kin strategies, what it judges by), this period's PTR and helps, its lineage, kin marker and
age, and each neighbor's tag and strategy, whether they are related, and who helped whom; an empty
site says so. Charts: **Strategies**, **Cooperation** (helps per decision and the share of decisions
toward the same tag), **Population** and **Kin** (related neighbors, help to relatives, and the two
conditional probabilities of Jansson's Table 4), against the period. A run stops at period 2,000
(`end`; 0 for never); immigration, PTR, cost, benefit, death, the mutation rates, pair play,
misperception and offspring placement apply to the running world. **Compare** entries: "Four colors
vs five (the Java's draw) — Ethnocentrism (Compare)" (`ha-standard` and `ha-java-five-colors`),
"Next to the parent vs anywhere — Ethnocentrism (Compare)" (`ha-standard` and
`jansson-offspring-anywhere`: 75.9 % ethnocentric against 8.3 %) and "Tags vs kin — Ethnocentrism
(Compare)" (`ha-standard` and `jansson-kin`). Credit: Ross A. Hammond and Robert Axelrod, "The
Evolution of Ethnocentrism," *Journal of Conflict Resolution* 50(6) (2006), 926–936, and their
archived Java/Ascape code (2003); Uri Wilensky's NetLogo *Ethnocentrism*, the replication they cite;
Thomas R. Shultz, Max Hartshorn and Ross A. Hammond, "Stages in the evolution of ethnocentrism,"
*CogSci 2008*; Thomas R. Shultz, Max Hartshorn and Artem Kaznatcheev, "Why is ethnocentrism more
common than humanitarianism?", *CogSci 2009*; Max Hartshorn, Artem Kaznatcheev and Thomas R. Shultz,
"The Evolutionary Dominance of Ethnocentric Cooperation," *JASSS* 16(3) 7 (2013); and Fredrik
Jansson, "Pitfalls in Spatial Modelling of Ethnocentrism: A Simulation Analysis of the Model of
Hammond and Axelrod," *JASSS* 16(3) 2 (2013). See
`docs/superpowers/specs/2026-09-25-ethnocentrism-design.md`.
```

- [ ] **Step 3: Roadmap**

After Milestone 13:

```markdown
## Milestone 14: Ethnocentrism (done)

Hammond and Axelrod's evolution of ethnocentrism (2006) as an eighth model kind, with its appendix's and
its archived code's departures as switches and presets, and the variants of Hartshorn, Kaznatcheev and
Shultz (2013) and Jansson (2013). The standard case and most of Table 1 reproduce within 3 points, and
Hartshorn, Kaznatcheev and Shultz's shares, early patterns and Study 2 orders almost exactly; the appendix's
5 % mutation is a slip, the code draws five colors for four (which Table 1 cannot tell apart), and the
color-blind agents' 14 % cooperation does not reproduce under any reading (41.8 %). Ethnocentrics take over
later than Table 1 l says, and Jansson's kin discriminators win by far less than his Table 5 unless the
kin basis never mutates, which he does not say. See
`docs/superpowers/specs/2026-09-25-ethnocentrism-design.md`.
```

and under **Experiments and science**, after the tag-based cooperation line:

```markdown
- **Hammond–Axelrod ethnocentrism** (and Hartshorn, Kaznatcheev & Shultz's and Jansson's critiques): done (Milestone 14).
```

- [ ] **Step 4: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test ethno -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && cargo test)
```

Expected: all PASS. Browser (controller): every ethno preset, the four colour modes, Inspect (agents, neighbors, empty sites, an agent that dies), the Rules panel's hidden fields and empty tag mutation, the three Compare entries, the seven sweeps, keyframes and the timeline on `jansson-kin`, share links and sessions of `hks-no-ethnocentrics` with a live cost change, recording, Max speed at 100 × 100, and every existing scenario.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/roadmap.md docs/superpowers/specs/2026-09-25-ethnocentrism-design.md
git commit -m "Document the ethnocentrism model and mark milestone 14 done" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

## Self-review (planning)

- **Spec coverage:** Architecture, Config, Rules and Statistics — Task 1 (and Decisions 1–11, 14, 22); the choices the sources leave open — Task 1's code and Decisions, Task 6's spec notes; Views — frames and Inspect JSON in Task 1, colour modes, charts and Inspect rows in Task 3; Presets and Compare — Tasks 1 and 4; Experiments and CLI — Task 2 and Task 4's `defaultForm`; Claims — Task 2's book-style tests and Task 5's survey; Page — Tasks 3–4; Testing — every task; Docs — Task 6.
- **Placeholders:** none; every code block is the planning dry run's code, which passed fmt, clippy `-D warnings`, the workspace tests, the ignored ethno tests, wasm-pack, the web build and tests, and the survey's tests (the Review Focus tests were added to the dry run and pass).
- **Type consistency:** see the web draft's notes below; the Rust `EthnoInspection`/`AgentView`/`NeighborView` and `EthnoSnapshot` serialize to Task 3's TS types field for field.
- **Review Focus:** each item names its test and task.
- **Caveat:** only the final state of each part was run; the RED expectations of intermediate steps are stated, not observed.

- **Spec coverage (page, survey, docs):** Views — colour modes (Task 3, `COLOR_MODES.ethno`), Inspect (Task 3), charts (Task 3); Page — the presets group (`MODELS`/`MODEL_LABELS`), the schema panel's groups and `show_if`s (Task 1's schema, Task 3's panel fixes), Compare entries and `defaultForm` (Task 4); Testing — Vitest for schema fields and `show_if`, charts, Inspect rows, Compare entries, `defaultForm`, determinism fingerprints (Tasks 3–4); Claims to test — every measured item has a survey claim (Task 5); Docs (Task 6).
- **Placeholders:** none; every block is the scratch copy's code.
- **Type consistency:** `EthnoInspection` / `EthnoAgentView` / `EthnoNeighborView` mirror the Rust `EthnoInspection` / `AgentView` / `NeighborView` field for field (`strategy` "E" … "mixed", `basis` "tag" | "kin", `kin_marker`, `helped`, `helped_by`); `EthnoStats` mirrors `EthnoSnapshot` (NaN → null); `EthnoConfig` mirrors the Rust config with `tag_mutation: number | null` and `kin_basis`.
