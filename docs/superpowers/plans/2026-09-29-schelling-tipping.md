# Schelling's Bounded Neighborhood Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add Schelling's bounded-neighborhood model (1971, pp. 167–186) as a model kind `tipping`
("Schelling's tipping"), with his tolerance schedules, moves, speeds and limits as switches, carried
through presets, sweeps, the survey, the page, goldens and docs.

**Architecture:** One core module `crates/sugarscape-core/src/tipping.rs` (config, schedules, the
world, stats, the plane render, presets), registered in `model.rs` like `line`. Each color's people
are ranked by tolerance once; the state is who is inside.

**Tech Stack:** Rust core, `wasm-bindgen`, the CLI, TypeScript + Vite + Vitest, the `survey` crate.

**Spec:** `docs/superpowers/specs/2026-09-29-schelling-tipping-design.md`; rules and numbers in
`docs/superpowers/specs/2026-09-29-schelling-reading-notes.md` §4.

## Global Constraints

- Existing runs unchanged: every golden entry stays green and unedited, native and WASM (grep each
  new fingerprint's hex into both places the WASM test mirrors).
- Literal defaults: Fig. 18's setup (100 Red, 50 Blue, straight lines from 2.0 to 0, the exact
  schedule); unstated choices named: entrants count themselves (his parabolas: n people tolerate
  n·R(n)); a step moves up to each color's speed, Red then Blue.
- Copy: model label **Schelling's tipping**; kind `tipping`; groups **Population**, **Tolerance**,
  **Moves**, **Limits**; Red and Blue on the page, his "whites" and "blacks" only in quotes.
- Commits end with `Claude-Session: https://claude.ai/code/session_01Ab2xuSb7ebpL1LqY9f1c1L`; never
  stage `.claude/`, `papers/` or `survey/out/`; in `survey/` run `rustfmt` on the new file only.
- Checks: `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`, `wasm-pack test --node crates/sugarscape-wasm`,
  `(cd web && npm run wasm && npx tsc --noEmit && npx vitest run)`.

## Review Focus

1. **Ranks and schedules:** the i-th most tolerant (i = 1…N) of a straight line with intercept a has
   tolerance a·(1 − i/N) (so 50 of 100 abide 1.0 or more, and the least tolerant none); a hyperbola
   k has k/i; tiers give each share its value; `intolerant_share` zeroes the least tolerant.
   Pinned by `schedules_rank_as_schelling_counts_them`.
2. **Moves:** the least tolerant insider leaves when the other color's count per own exceeds its
   tolerance; otherwise the most tolerant outsider enters when, counting itself, the ratio is within
   its tolerance and the limits allow. Pinned by `the_least_tolerant_leave_and_the_most_tolerant_enter`.
3. **Fig. 18's two stable states and Fig. 19's stable mix** at the texts' numbers. Pinned by
   `fig_18_ends_all_one_color_and_fig_19_holds_80_80`.
4. **Limits** cap entry without forcing anyone out. Pinned by `a_limit_stops_entry_at_the_cap`.
5. **Order and speeds**: `simultaneous` decides both colors' moves from the step's start state.
   Pinned by `simultaneous_moves_read_the_same_state`.

## Decisions

1. Entrants count themselves (his curves are n·R(n) for n including the entrant); a switch
   `entry: counting_self | as_is` names the other reading.
2. Starting insiders are the most tolerant of each color (his analysis), or random with `start`.
3. One step: each color in `order`, up to its speed of moves, each move re-reading the state.

---

### Task 1: The `tipping` model in the core

**Files:** create `crates/sugarscape-core/src/tipping.rs`; modify `lib.rs`, `model.rs`,
`presets.rs`, `titles.rs`, `tests/golden.rs`.

**Produces:** `tipping::{TippingConfig, Schedule, Draws, Start, Order, Entry, TippingWorld,
TippingSnapshot, SERIES, schema, presets}`; `TippingWorld::{new, step, run, inside, tolerances,
curve}`; `ModelKind::Tipping` (`"tipping"`), `ModelConfig::Tipping`, `ModelWorld::Tipping`.

- [ ] **Step 1: Tests first** (in `tipping.rs`):

```rust
#[test]
fn schedules_rank_as_schelling_counts_them() {
    let line = Schedule::Line { intercept: 2.0 };
    let t = line.tolerances(100, 0.0);
    assert_eq!((t[0], t[49], t[99]), (1.98, 1.0, 0.0), "50 of 100 abide 1.0 or more; the least, none");
    let n: Vec<f64> = (1..=100).map(|n| n as f64 * t[n - 1]).collect();
    assert_eq!((n[49], n[74], n[89]), (50.0, 37.5, 18.0), "p. 169: 50 tolerate 50, 75 tolerate 37.5, 90 tolerate 18");
    assert_eq!(n[19], 32.0, "20 tolerate 32 (his 36 is a slip)");
    assert_eq!(Schedule::Hyperbola { k: 90.0 }.tolerances(100, 0.0)[9], 9.0);
    let cut = line.tolerances(100, 0.6);
    assert!(cut[40..].iter().all(|&r| r == 0.0) && cut[39] > 0.0, "the least tolerant 60 % intolerant");
}

#[test]
fn the_least_tolerant_leave_and_the_most_tolerant_enter() { /* hand-built: 3 Red inside, 4 Blue inside, ... */ }

#[test]
fn fig_18_ends_all_one_color_and_fig_19_holds_80_80() {
    // Fig. 18 (100 and 50, intercept 2): from 25 and 25, it empties of one color.
    let end = run_to_rest(fig18(25, 25));
    assert!(end.0 == 0 || end.1 == 0, "{end:?}");
    // Fig. 19 (100 and 100, intercept 5): from half of each, the 80–80 mixture.
    assert_eq!(run_to_rest(fig19(50, 50)), (80, 80));
    // From all Red, fewer than 25 Blue entering together is not enough.
    assert_eq!(run_to_rest(fig19(100, 20)).1, 0);
}

#[test]
fn a_limit_stops_entry_at_the_cap() { /* limit_red 40 on Fig. 20's numbers: Red never above 40 */ }

#[test]
fn simultaneous_moves_read_the_same_state() { /* both colors decided from the step's start */ }
```

(`run_to_rest(config) -> (u32, u32)` steps until still, up to 10 000 steps; `fig18(r, b)` and
`fig19(r, b)` build the configs with `start: Start::Given { red: r, blue: b }`. The two hand-built
tests are written with exact expected counts before the code.)

- [ ] **Step 2: Implement.** `Schedule` (`#[serde(tag = "shape")]`): `Line { intercept }`,
`Hyperbola { k }`, `Tiers { tiers: Vec<(f64, f64)> }` (share, tolerance, most tolerant first);
`fn tolerances(&self, n, intolerant_share) -> Vec<f64>` in rank order. `Draws::{Schedule, Random}`
(random: each person's tolerance the schedule at a uniform rank, then sorted). `Start::{Given {red,
blue}, Random {chance}}`. `Order::{Alternate, Simultaneous, RedFirst, BlueFirst}` (alternate: Red
then Blue each step, which for speeds of 1 equals `RedFirst`; kept as the named default).
`Entry::{CountingSelf, AsIs}`. Config fields: `red, blue, red_schedule, blue_schedule,
intolerant_red, intolerant_blue, draws, start, speed_red, speed_blue, order, entry, limit_red,
limit_blue, limit_total` (limits 0 = none). World: tolerances per color (rank order), `inside` as a
count per color when starts are ranked (insiders are always the top ranks: least tolerant leaves,
most tolerant enters) — with `Start::Random`, a set per color, the least tolerant insider and most
tolerant outsider found by scan. Stats: `red_in, blue_in, red_unhappy, blue_unhappy, still,
blue_share`. Render: the plane (`red + 1` × `blue + 1` pixels): each color's tolerated region
tinted (Red content where blue ≤ n·R(n)), both where both, the path so far, the current point.
Inspect a pixel: the state (r, b) and who would move there.

- [ ] **Step 3: Presets and titles** (sources "Schelling 1971" / "1969"): `s71-fig18`, `s71-fig19`,
`s71-fig20` (Fig. 19's schedules with 200 Red and 100 Blue), `s71-fig21` (equal numbers, intercept
3.0: the border), `s71-fig22` (Fig. 20 with Red limited to 40), `s69-intolerant` (equal 100s,
intercept 2, the least tolerant 60 % intolerant: "a stable equilibrium will occur at forty apiece"),
`s71-minority` (a 5:1 minority as tolerant as the majority), `s71-less-tolerant` (Fig. 22's case
with the least tolerant two-thirds of Red made intolerant instead of limited). Titles measured.

- [ ] **Step 4: Register** (every `ModelKind`/`ModelConfig`/`ModelWorld` match, the ALL count,
the kind-names test), goldens (200 steps from seed 1), `cargo test -p sugarscape-core`, commit.

### Task 2: Sweeps, CLI and WASM

- [ ] WASM test `tipping_sims_match_the_native_golden_entries` (three presets); the WASM builtin
sweep list. Sweeps: `tipping-plane` (starting insiders on a grid, where each ends: the share ending
mixed), `tipping-intercept` (equal numbers, intercept 1–6: stable mix or not), `tipping-speeds`
(Blue's speed 1–5 against Red's 1, from Fig. 19's 50–50), `tipping-limit` (Fig. 20 with Red
limited 20–100). Measured descriptions. Commit.

### Task 3: The page

- [ ] `ModelKind` `'tipping'`, label, color modes (`plane`), charts (**Inside**: `red_in`,
`blue_in`; **Unhappy**: `red_unhappy`, `blue_unhappy`), Inspect rows, time axis **Steps**; the
config type. Vitest for the kind. Build and tests; commit.

### Task 4: The survey

- [ ] `survey/src/claims/tipping.rs`: the design's claims, rules fixed before running (the numbers
Schelling gives: 80–80; over 40 %; more than 25 %; the 3.0 threshold; 40 whites; 40 apiece; the
minority; the less tolerant two-thirds; no discontinuity at the median). Run, record in preset
descriptions and titles. Commit.

### Task 5: Docs and verification

- [ ] README section, `docs/papers.md` milestone 31, spec amendments; full checks; commit; final
whole-branch review.
