# Variations on Schelling Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Pancs & Vriend's, Gauvin–Vannimenus–Nadal's, Singh–Vainchtein–Weiss's and Zhang's
(JEBO) variations of Schelling's model as named switches and presets on `schelling` and `line`,
with sweeps, survey claims, page and docs, changing no existing run.

**Architecture:** `schelling.rs` gains `movers`, two movements (`best`, `swap`), `utility` + `beta`,
two starts and three statistics; `line.rs` gains ring edges and the best move. Defaults are
Schelling's, so every existing golden holds.

**Tech Stack:** Rust core, WASM, CLI, TypeScript/Vitest, the survey crate.

**Spec:** `docs/superpowers/specs/2026-09-30-schelling-variations-design.md`; §7 of the reading
notes; the papers in `papers/schelling/`.

## Global Constraints

- Every existing golden (native and WASM) stays unedited and green.
- Copy: preset ids as the spec lists; captions say whose rules each result comes from.
- Commits end with the Claude-Session line; never stage `.claude/`, `papers/`, `survey/out/`;
  rustfmt only the new survey file.
- Checks as milestone 31 (fmt, clippy, workspace tests, wasm-pack, web tsc + vitest, survey).

## Review Focus

1. **Best response** takes the highest-utility empty square with staying among the options, ties at
   random, judged with the mover's square vacated. Pinned by `best_response_takes_the_best_square`.
2. **No inertia**: under `movers: anyone`, content agents move (P&V, Gauvin). Pinned by
   `anyone_moves_even_when_content`.
3. **Swaps**: a pair of non-neighboring occupied squares; with a huge β only improving swaps happen,
   with β = 0 half of them. Pinned by `swaps_follow_the_logit_on_summed_utility`.
4. **Starts**: a checkerboard alternates colors; the deleted checkerboard removes vacancies half
   from each color. Pinned by `checkerboard_starts_alternate_colors`.
5. **Statistics**: clusters (4-adjacent like-colored groups), s, mixed pairs on hand boards. Pinned
   by `cluster_counts_and_mixed_pairs_on_a_hand_board`.

## Decisions

1. One step under `movers: anyone` is one Monte Carlo sweep: every agent once, in a random order
   (P&V and Gauvin draw one agent at a time with replacement; named in the presets).
2. Utilities are over the unlike share among occupied neighbors (P&V; empty neighborhoods lowest),
   or over like neighbors of n (Zhang's tent, his Table 4).
3. A swap step makes `population` attempts (one per agent, on average).

### Task 1: Schelling's switches

**Files:** `crates/sugarscape-core/src/schelling.rs`.

- [ ] Tests first (the five in Review Focus), each on a hand-built board with exact expectations.
- [ ] `Movers { Discontent, Anyone }`, `Movement` adds `Best`, `Swap`; `Utility { Flat, P50, P100,
  Spiked, Tent }` with `fn value(like, occupied, max) -> f64`; `beta: f64` (default 0: unused by
  Schelling's rules); `Start { Random, Checkerboard, DeletedCheckerboard }`; validation (swap needs
  no vacancies but works with them; best needs a utility). Stats `clusters`, `seg_s`,
  `mixed_pairs` appended to `SERIES`.
- [ ] Commit.

### Task 2: The ring and the best move on the line

**Files:** `crates/sugarscape-core/src/line.rs`.

- [ ] Tests: on a ring the first and last are neighbors; the best move takes the insertion point of
  highest utility (ties random, staying included). Implement `edges: Ends | Ring`, `movement:
  Nearest | Best`, `movers`, `utility`. Commit.

### Task 3: Presets, sweeps, goldens

- [ ] Presets and titles per the spec (measured); `MODEL_GOLDEN` and the WASM mirror; sweeps
  `gvn-phase` (T for several ρ), `svw-city` (board size), `zhang-beta`, `zhang-neighborhood`.
  Commit.

### Task 4: The page

- [ ] Types (new config fields), a **Clusters** chart for `schelling`, the new switches show from the
  schema. Vitest; commit.

### Task 5: The survey

- [ ] `survey/src/claims/variations.rs`: the spec's claims with rules from each paper's numbers,
  written before running; record outcomes in presets, sweeps, titles. Commit.

### Task 6: Docs and review

- [ ] README (a Variations section), `docs/papers.md` milestone 32, spec amendments; full checks;
  final whole-branch review; fixes; commit.
