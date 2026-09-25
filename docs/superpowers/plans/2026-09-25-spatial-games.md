# Spatial Games (Nowak–May and its critics) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the spatial Prisoner's Dilemma as a sixth model kind covering the 1992–1994 exchange — Nowak & May's deterministic synchronous lattice game, Huberman & Glance's asynchronous updating, and Nowak, Bonhoeffer & May's probabilistic winning, continuous time, random arrays and cubes — with each paper's figures as presets, the measured findings in descriptions, sweeps and the survey, in the worker engine at every speed, keyframes and the timeline, links, Compare, recording, Experiments and the CLI, without changing any existing run.

**Architecture:** First the anasazi's portable `ln` and civil violence's portable `exp_neg` move into a crate-level `portable` module (bit-identical; old paths keep working). Then a new core module `crates/sugarscape-core/src/spatial/` — `config.rs` (the config, live fields, validation, schema), `geometry.rs` (players, cells and neighbor lists for square, cubic and random lattices), `stats.rs`, `world.rs` (`SpatialWorld`: scores, deterministic and Eq. 1 winning, synchronous and asynchronous generations, frames with a cube's slice, Inspect), `presets.rs`, `mod.rs` — plugged into `ModelConfig`/`ModelWorld` (with keyframes) as `spatial`. The WASM `Sim` needs no new calls. On the page the model joins `models.ts`, `MODEL_CHARTS`, the display (a Slice menu for cubes), Inspect, Compare and Experiments; the schema panel already handles `show_if`.

**Tech Stack:** Rust core (`sugarscape-core`), `wasm-bindgen`, the `sugarscape` CLI, TypeScript + Vite + uPlot + Vitest, the standalone `survey` crate. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-09-25-spatial-games-design.md` (binding). The milestone 1–11 specs stay binding where not changed; this plan follows the civil violence plan's conventions (`docs/superpowers/plans/2026-09-25-civil-violence.md`). Sources: NM92 (Nature 359:826), HG93 (PNAS 90:7716), NBM94 (PNAS 91:4877).

## Global Constraints

- **Existing models unchanged.** Every existing `GOLDEN`/`MODEL_GOLDEN` entry and legacy fixture stays green and **unedited** (`MODEL_GOLDEN` gains twelve entries). `tests/legacy.rs`, `tests/invariants.rs`, `tests/book.rs` and `tests/civil.rs` are not edited. The `portable` move must leave every fingerprint bit-identical.
- **One engine path.** The spatial model plugs into the `Model` trait, `ModelConfig`, `ModelWorld` (including `checkpoint`/`restore`), the WASM `Sim`, host, engine, schema panel, chart table, Compare, sweeps and CLI — no per-model copies of that machinery.
- **Honest descriptions.** Preset and sweep descriptions state the measured results, including what does not reproduce (Decision 12), and name the choices the papers leave open (the spec's list).
- **Deterministic and portable.** A world is a function of (config, seed). Sample `u32` ranges (`gen_range(0..n as u32)`); iterate vectors; never the platform's `powf`/`exp`/`ln` in a rule — `portable::{ln, exp_neg}` (Decision 3).
- **Copy (verbatim):** model label **Spatial Games**; preset ids `nm-1a-static`, `nm-1b-chaos`, `nm-2a-universal`, `nm-3-kaleidoscope`, `nm-no-self`, `nm-four-neighbors`, `hg-async-kaleidoscope`, `nbm-probabilistic`, `nbm-discrete`, `nbm-continuous`, `nbm-random-array`, `nbm-cube`; Compare entries **Synchronous vs asynchronous — Spatial Games (Compare)** (id `nm-sync-vs-async`: `nm-3-kaleidoscope` vs `hg-async-kaleidoscope`) and **Discrete vs continuous time — Spatial Games (Compare)** (id `nbm-discrete-vs-continuous`: `nbm-discrete` vs `nbm-continuous`); color modes **Change**, **Strategy**, **Payoff**; the cube's menu label **Slice** with options `z = 0` … ; schema groups **Game**, **Lattice**, **Update**, **Start**; chart titles **Cooperators**, **Changes**, **Switches**, **Payoffs**; built-in sweep ids `nm-universal`, `hg-async`, `nbm-grid-discrete`, `nbm-grid-continuous`, `nbm-radius` (in that order, after `cv-jail-waits`); series `fraction_c, changed, c_to_d, d_to_c, mean_payoff_c, mean_payoff_d, players`, in that order.
- **Names are binding across tasks:** core `portable::{ln, exp_neg}`; `spatial::{SpatialConfig, Lattice, Neighborhood, Boundary, Update, Winning, Start, LIVE, schema, Geometry, EMPTY, offsets, SpatialSnapshot, SERIES, SpatialWorld, SpatialMode, SpatialInspection, CellXyz, PlayerView, Candidate, C_AFTER_C, D_AFTER_D, D_AFTER_C, C_AFTER_D, presets}`; `SpatialConfig::{dims, validate, changes}`; `Geometry::{new, len, neighbors, xyz, at, central}`; `SpatialWorld::{new, step, run, players, is_cooperator, score, inspect, geometry, stats, config, tick}`; `ModelKind::Spatial`, `ModelConfig::Spatial`, `ModelWorld::Spatial`; survey `claims/spatial.rs`; web `SpatialConfig`, `SpatialStats`, `SpatialInspection`, `PlayerView`, `SpatialCandidate` (types.ts), `isSpatialView` (models.ts), `sliceOptions`, `middleSlice`, `playerRows` (spatial.ts).
- Every commit message ends with a blank line and then `Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn` (a second `-m`). Stage **only** the files named in the task (never `-A`/`.`, never `.claude/`, `.superpowers/` or `web/src/wasm-pkg`).
- Rust tasks finish with `cargo fmt --all && cargo clippy --all-targets -- -D warnings`. CI runs the newest stable clippy: after pushing, check `gh run list`.
- Web tasks run `(cd web && npm run build && npm test)`; single files with `(cd web && npx vitest run src/<file>.test.ts)`.
- **TypeScript:** `strict`, `noUnusedLocals`, `noUnusedParameters`, `noFallthroughCasesInSwitch`; unused imports fail the build; never pass a possibly-null child to `replaceChildren`/`append` (use `h()`); test names use the typographic apostrophe (’) inside single-quoted strings.
- **Browser checks are the controller's.**

## Review Focus

1. **A cube's view and Inspect** — the grid shows one z-slice; Inspect must describe the cell in the slice on screen, and switching slices must not change the run. Pinned in Task 2 by `a_cube_draws_the_chosen_slice_and_inspects_it` and in Task 4 by the `sliceOptions`/`middleSlice` tests.
2. **Random arrays' empty cells** — clicking an empty cell must inspect nothing, not crash; the frame draws empty cells dark. Pinned in Task 2 (`Geometry::at` returns `None`; `inspect` returns `agent: None`) by `random_arrays_hold_a_fraction_and_meet_within_the_radius` and a Task 5 `playerRows` test for a missing player.
3. **Probabilistic winning at the edges** — m = 0, every score 0, huge m: no NaN, no panic, deterministic draws. Pinned in Task 2 by `eq_1_weighs_candidates_by_their_scores` and `portable_powers_match_the_library`.
4. **Live switching of update/winning mid-run and step-back** — both apply to the running world and keyframes restore them exactly. Pinned in Task 2 by `schema_paths_exist_and_match_what_set_config_allows` and the checkpoint test's `nbm-probabilistic` entry.
5. **Tiny lattices** — widths 1–2 with fixed edges and 3 with periodic ones must work (no self-neighbors). Pinned in Task 2 by `sides_radius_and_ranges_are_checked` and `the_change_colours_follow_nm92` (2 × 2).

## Why this task order

- **Core first (Tasks 1–3):** the `portable` move alone (goldens prove it changed nothing), the model with its plumbing, presets and golden entries, then the measurements (book-style tests, the five sweeps, WASM portability).
- **The page (Tasks 4–5):** types, views, charts and the Slice menu; then Inspect, Compare and Experiments.
- **The survey (Task 6)**, **docs and full verification (Task 7)**.

## Decisions (where the spec leaves room)

Binding. Every rule below was implemented in a scratch copy during planning and measured; the Rust code in Tasks 1–3 and 6 is that code and passes. The web code (Tasks 4–5) was written against the files as they stand and was not run during planning.

1. **Module layout.** `crates/sugarscape-core/src/portable.rs` (`ln`, `exp_neg`, their tests); `anasazi::random` re-exports `ln` (`pub use crate::portable::ln;`); `civil/math.rs` is deleted and `civil` re-exports `crate::portable::exp_neg`. `crates/sugarscape-core/src/spatial/` with `config.rs`, `geometry.rs`, `stats.rs`, `world.rs`, `presets.rs`, `mod.rs`; `pub mod portable;` before `presets` and `pub mod spatial;` after `schema` in `lib.rs`.
2. **Geometry.** Players are numbered by cell (`x + y·w + z·w·h`, ascending); each has a neighbor list (CSR), offsets ascending by (dz, dy, dx). Square: Moore (8) or von Neumann (4); cube (n = `width`): Moore (26) or von Neumann (6); random: exactly round(occupancy × cells) cells chosen by one shuffle and sorted (at least one), neighbors = players within Euclidean distance ≤ r. Fixed edges drop off-lattice neighbors; periodic edges wrap. Sides ≥ 3 with periodic edges (≥ 1 with fixed); a cube's side ≤ 64; random arrays need 2·⌊r⌋ < the smaller side.
3. **Scores and portable powers.** A cooperator with k cooperating neighbors scores k + a; a defector scores k·b + (n − k)·ε + a·ε (n its neighbors). (The spec's "sums in a fixed order" is realised as this closed form, exact for integer k.) Eq. 1's A^m is computed as `exp_neg(m · (ln A − ln A_max))` (A^m / A_max^m, so e^x only for x ≤ 0); m = 0 gives every candidate weight 1 (0^0 = 1); with m > 0 a candidate scoring 0 weighs 0; if every candidate scores 0 (m > 0) the site keeps its strategy. ε is 0 or at least 10⁻⁶, so `ln` never sees a subnormal.
4. **Deterministic winning:** the highest-scoring candidates (the site and its neighbors); if both strategies are among them the site keeps its strategy. **Probabilistic:** one `gen::<f64>()` per updated site, drawn before P(C), C iff u < P(C).
5. **Generations.** Schedule entries due at this tick apply first (every score is recomputed only if any applied). Synchronous: every site's next strategy from the current scores, then all switch. Asynchronous: N microsteps, each `gen_range(0..N as u32)` picks a site, which is rescored with its neighbors from the current strategies (`score(j)` on the fly) and updated at once (`microstep`). Then `previous` (the strategies at the start of the generation) and the new strategies give the change counts; every score is recomputed for the new strategies; statistics recorded.
6. **Start.** One shuffle of the players (after a random array's cells): the first round(defectors × N) start D; or the player nearest ((w−1)/2, (h−1)/2[, (d−1)/2]) (integer division; ties to the lowest number) starts D. `previous` = the start, so t = 0 shows only blue and red.
7. **Live and reset fields.** Live: `b`, `epsilon`, `self_weight`, `update`, `winning`, `m` (set_config rescores). Everything else, including `schedule`, is reset-only; schedule entries may only set live fields.
8. **Frames and Inspect.** Frame = the lattice's x × y (a cube's slice); `render(mode, layer)` reads `slice:<z>` (clamped; default the middle, `d / 2`) and remembers it (`view_z`, a `Cell`), and `inspect_json(x, y)` inspects that slice. Colors: blue `BLUE` C→C, red `RED` D→D, yellow `BOTH` C→D, green `LENDER` D→C; Strategy uses blue/red; Payoff `lerp(COOL, HOT, score / max score)`; empty cells `BACKGROUND`. Inspect JSON `{ site: {x, y, z}, agent: { id, strategy, previous, score, candidates: [{x, y, z, strategy, score}] (itself first), next | null, p_c | null } | null }`. `locate(id)` is the player's (x, y). Agents CSV `id,x,y,z,strategy,score`. Fingerprint: FNV-1a over the tick and each 64-player chunk of strategies now and a generation ago.
9. **Keyframes.** `SpatialWorld` derives `Clone`; `ModelWorld::checkpoint`/`restore` gain `Spatial` arms; `tests/checkpoint.rs` adds `nbm-probabilistic` to its list (probabilistic winning restores exactly).
10. **Presets** (after civil violence's in the catalog): as the spec's table, with these measured choices: NBM94's arena starts from **50% defectors** (their m = 0 row, pure drift, keeps its start and shows roughly even colors; 10% would leave ~90% blue) and `nbm-random-array` too (Decision 12); `nbm-cube` uses b = 1.6, 10% defectors; `nbm-discrete` (80², periodic, 50%, deterministic, b = 1.71) is added for the Compare entry. Descriptions carry Decision 12's numbers.
11. **Page.** `isSpatialView(v)` is `'z' in v.site`. `COLOR_MODES.spatial` = Change, Strategy, Payoff; `MODEL_OVERLAYS.spatial = []`. The display shows the **Slice** menu (instead of Landscape) for a cube, with `slice:0` … `slice:n−1`, choosing the middle when the current layer is not one of them; `Layer` gains `` `slice:${number}` ``. Charts: **Cooperators** (`fraction_c`, 0–1), **Changes** (`changed`, 0–1), **Switches** (`c_to_d`, `d_to_c`; split from Changes because they are counts), **Payoffs** (`mean_payoff_c`, `mean_payoff_d`). Inspect summarizes the candidates (C and D counts and best scores) rather than listing up to hundreds. `defaultForm('spatial')`: x `b` `1.05:2.05:0.05`, final `fraction_c`, 200 ticks, 3 seeds.
12. **Measured results** (release, recorded 2026-09-25; seeds 1–20 unless noted):
    - **NM92.** Kaleidoscope (99², one D, b = 1.9): four-fold symmetric at t = 30, 217, 219, 221 and at every generation checked; **reaches the edges at t = 49 exactly**; f_C averages 0.315 over t = 100–221. The cluster thresholds hold exactly (a 6 × 6 D block grows at b = 1.85 and shrinks at 1.75; a 2 × 2 D cluster grows at 1.85; a 2 × 2 C cluster grows at 1.95 and not at 2.05). 400², 40% D, b = 1.9: f_C over t = 201–300 **0.3179 ± 0.0005** (12 ln 2 − 8 = 0.31777). 200², b = 1.9: 0.318–0.324 (10% D); every seed 0.318–0.323 from 5%, 30%, 60% D; from 80%, 19 of 20 and one (seed 12) ends all C; sweep (3 seeds): 0.320–0.321 from 5% to 80%, 0.21 from 90%, 0 from 95%. Fig. 1a (b = 1.77): 0.737–0.748 at t = 200 (paper: 0.7–0.95), 3% of sites blinking. No self-interaction (b = 1.62): 0.301–0.305 (paper ~0.299). Four neighbors (b = 1.8): 0.379–0.382 at every b in (5/3, 2) (paper ~0.374: **not within 0.005**).
    - **HG93.** Asynchronous kaleidoscope, b = 1.9: all D at t = 56–149 (mean 101) — reproduced. But at b = 1.7 the lone defector dies out (f_C 0.974–0.999), and the hg-async sweep (10 seeds): asynchronous 0.99–1.00 for b ≤ 1.42 and at 1.71–1.77, 0.61 at 1.55, 0 at 1.9, 0.04 at 2.01; synchronous 1.00 through 1.77, 0.34 at 1.9, 0.91 at 2.01. **"Always" fails below 1.8; HG93 never state b.**
    - **NBM94.** Grids (5 seeds, 80², periodic, 50% D, t = 200) — discrete: m = ∞ 0.99 at 1.05, 0.88–0.94 from 1.13 to 1.77, 0.30 at 1.9, 0 at 2.01; m = 100: 1.00 to 1.16, 0.33 at 1.9; m = 20: 0 from 1.9; m = 10: 0 from 1.71; m = 1: 0.30 at 1.35, 0.002 at 1.77, 0 from 1.9; m = 0.5: 0.11 at 2.01; m = 0: 0.56 everywhere. Continuous: m = ∞ 0.59–0.99 to 1.77, 0 at 1.9 and 2.01; m = 1: 0.86 at 1.35, 0.02 at 2.01; m = 0.5: 0.23 at 2.01; m = 0: 0.48. The paper's regimes reproduce; its m = 1 discrete thumbnails show scattered C at 1.9–2.01 where these runs are all D. b = 1.9 deterministic: continuous all D in 20 of 20; discrete chaotic in 16 of 20 (two end all C, two nearly all D — small-lattice collapse the paper does not mention). b = 1.71: discrete 0.86–0.92, continuous 0.71–0.73. m = 1 without self-interaction: C gone (≤ 0.007) at b = 1.13 and 1.35, **but 0.16–0.33 at 1.05** (paper: "C cannot persist"); deterministic without self-interaction keeps C (0.85–0.95). Random arrays (b = 1.6, 50% D): all D in 0, 10, 20 of 20 at r = 5, 9, 11 — **r_c ≈ 9 reproduces from 50%**; from 10% no radius up to 11 ends all D (f_C > 0.3; sweep 0.73–0.86). Cube (30³, b = 1.6, 10% D): coexistence for b 1.1–1.8, nearly all D at 1.9; f_C 0.331–0.342 over t = 101–200, 25–27% of the cube changing each generation.
    - **Survey verdicts** (20 seeds): 10 hold; **3 fail**: `nm-four-neighbors.fraction` (0.380 vs 0.374), `nbm-probabilistic.no-self` (C persists at b = 1.05), `hg-async.always`.
    - **Performance** (native release): 400² synchronous 4.8 ms a generation; 200² 1.2 ms; 99² asynchronous 0.7 ms; 30³ cube 3 ms; the five sweeps 1–62 s on 10 threads.
    - **Re-measuring:** `cargo test -p sugarscape-core --release --test spatial -- --ignored` and `cargo run --release -p sugarscape-cli -- sweep --builtin <id> --quiet --summary-csv /dev/stdout --out /dev/null`. If an implementation following this plan gives different numbers, stop and report rather than retune.
13. **Golden entries** (200 ticks, seed 1, `MODEL_GOLDEN`, after `cv-netlogo`): `nm-1a-static` 0x5a72fde83bfb7283, `nm-1b-chaos` 0x8360c871dcead14f, `nm-2a-universal` 0x76ee3f7ba596e8ce, `nm-3-kaleidoscope` 0xedd9930343121ef9, `nm-no-self` 0x8feb7f382db6cbb5, `nm-four-neighbors` 0xfd39a3d611f99c43, `hg-async-kaleidoscope` 0xef13c172a34a980d, `nbm-probabilistic` 0x79a048f606d28d74, `nbm-discrete` 0x2befee9be89b26d9, `nbm-continuous` 0x186717b420e5af31, `nbm-random-array` 0xcf2c74041806d530, `nbm-cube` 0x96baab61504e7923. `wasm-pack test` reproduces `nbm-probabilistic`, `hg-async-kaleidoscope` and `nbm-random-array` (checked during planning).

## File Structure

```
crates/sugarscape-core/src/portable.rs                NEW  ln, exp_neg (1)
crates/sugarscape-core/src/anasazi/random.rs          MOD  re-exports ln (1)
crates/sugarscape-core/src/civil/math.rs              DEL  (1)
crates/sugarscape-core/src/civil/mod.rs, world.rs     MOD  use crate::portable::exp_neg (1)
crates/sugarscape-core/src/lib.rs                     MOD  pub mod portable (1); pub mod spatial (2)
crates/sugarscape-core/src/spatial/*.rs               NEW  (2)
crates/sugarscape-core/src/model.rs, presets.rs       MOD  Spatial kind, config, world, keyframes; catalog (2)
crates/sugarscape-core/tests/golden.rs, checkpoint.rs MOD  twelve entries; nbm-probabilistic (2)
crates/sugarscape-core/tests/spatial.rs               NEW  (3)
sweeps/nm-universal.json, hg-async.json, nbm-grid-discrete.json, nbm-grid-continuous.json, nbm-radius.json   NEW (3)
crates/sugarscape-core/src/sweep.rs                   MOD  five built-ins (3)
crates/sugarscape-cli/tests/cli.rs, crates/sugarscape-wasm/tests/web.rs   MOD  built-in lists; spatial fingerprints (3)
web/src/types.ts, models.ts, spatial.ts, spatial.test.ts, ui/display.ts, ui/series-data.ts   MOD/NEW (4)
web/src/ui/inspect-panel.ts, compare-presets.ts, experiments/form.ts   MOD (5)
survey/src/claims/spatial.rs, claims/mod.rs           NEW/MOD (6)
README.md, docs/roadmap.md, the spec                  MOD (7)
```

---

### Task 1: One home for the portable math

**Files:**
- Create: `crates/sugarscape-core/src/portable.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/anasazi/random.rs`, `crates/sugarscape-core/src/civil/mod.rs`, `crates/sugarscape-core/src/civil/world.rs`
- Delete: `crates/sugarscape-core/src/civil/math.rs`

**Interfaces:**
- Produces: `crate::portable::{ln(x: f64) -> f64, exp_neg(x: f64) -> f64}`; `anasazi::random::ln` and `civil::exp_neg` remain as re-exports.

- [ ] **Step 1: Move the functions**

Create `crates/sugarscape-core/src/portable.rs` — the anasazi's `ln` and civil violence's `exp_neg` with their tests, unchanged:

```rust
//! Elementary functions that are bit-for-bit the same on every platform.
//! `f64::ln` and `f64::exp` call the platform's math library, which may
//! differ in the last bit between a native build and wasm32; rules whose
//! results feed a fingerprint use these instead, built from IEEE-exact
//! `+ − × ÷` (and `sqrt`) only. Milestones 10 and 11 wrote them for the
//! anasazi's harvest noise and civil violence's arrest probability;
//! milestone 12 gathered them here for the spatial games' probabilistic
//! winning.

use std::f64::consts::LN_2;

/// The natural logarithm of a positive, finite, normal `x` from `+ − × ÷`
/// only: `x = m · 2^e` with `m` in `[√½, √2)`, and
/// `ln m = 2 atanh t`, `t = (m − 1)/(m + 1)`, summed as a series (`|t| ≤
/// 0.1716`, so 20 terms are far below one ulp).
pub fn ln(x: f64) -> f64 {
    debug_assert!(x.is_normal() && x > 0.0, "ln of {x}");
    let bits = x.to_bits();
    let mut e = ((bits >> 52) & 0x7ff) as i64 - 1023;
    let mut m = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000);
    if m > std::f64::consts::SQRT_2 {
        m *= 0.5;
        e += 1;
    }
    let t = (m - 1.0) / (m + 1.0);
    let t2 = t * t;
    // Horner's rule over 1 + t²/3 + t⁴/5 + … + t³⁸/39.
    let mut series = 0.0;
    for k in (0..20).rev() {
        series = series * t2 + 1.0 / f64::from(2 * k + 1);
    }
    e as f64 * std::f64::consts::LN_2 + 2.0 * t * series
}

/// ln 2 split in two (fdlibm's): the high part's low 21 bits are zero, so
/// `n · LN2_HI` is exact for every `n` used here.
const LN2_HI: f64 = f64::from_bits(0x3fe6_2e42_fee0_0000);
const LN2_LO: f64 = f64::from_bits(0x3dea_39ef_3579_3c76);

/// `e^x` for `x ≤ 0` from `+ − × ÷` only: `x = n·ln 2 + r` with `n` the
/// nearest whole number to `x / ln 2` (so `|r| ≤ ½ ln 2`, reduced with the
/// split ln 2 so large `n` lose nothing), `e^r` summed by Horner's rule over
/// 20 Taylor terms (far below one ulp for `|r| ≤ 0.35`), then scaled by
/// `2^n` through the exponent bits. Below −708 the result is 0 (the smallest
/// normal double is about e^−708).
pub fn exp_neg(x: f64) -> f64 {
    debug_assert!(x <= 0.0 && !x.is_nan(), "exp_neg of {x}");
    if x < -708.0 {
        return 0.0;
    }
    let n = (x / LN_2).round();
    let r = (x - n * LN2_HI) - n * LN2_LO;
    let mut series = 1.0;
    for k in (1..=20).rev() {
        series = 1.0 + series * r / f64::from(k);
    }
    // 2^n for n in −1022..=0, built from its exponent bits.
    let scale = f64::from_bits(((n as i64 + 1023) as u64) << 52);
    series * scale
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;
    use rand::Rng;

    #[test]
    fn ln_matches_the_library_to_a_few_ulps() {
        let mut r = rng::seeded(7);
        let mut xs: Vec<f64> = (0..20_000)
            .map(|_| r.gen::<f64>())
            .filter(|&x| x > 0.0)
            .collect();
        xs.extend([
            1.0,
            0.5,
            2.0,
            1e-300,
            1e300,
            std::f64::consts::E,
            1.0 + 1e-12,
            0.999_999,
        ]);
        for x in xs {
            let (ours, std) = (ln(x), x.ln());
            assert!(
                (ours - std).abs() <= 4.0 * f64::EPSILON * std.abs().max(1e-300),
                "{x}: {ours} vs {std}"
            );
        }
        assert_eq!(ln(1.0), 0.0);
    }

    #[test]
    fn exp_neg_matches_the_library_to_a_few_ulps() {
        let mut r = crate::rng::seeded(5);
        let mut xs: Vec<f64> = (0..20_000).map(|_| -r.gen::<f64>() * 50.0).collect();
        xs.extend([0.0, -1e-12, -0.5, -1.0, -2.3, -2.3 / 3.0, -100.0, -700.0]);
        for x in xs {
            let (ours, std) = (exp_neg(x), x.exp());
            assert!(
                (ours - std).abs() <= 8.0 * f64::EPSILON * std,
                "{x}: {ours} vs {std}"
            );
        }
        assert_eq!(exp_neg(0.0), 1.0);
        assert_eq!(exp_neg(-800.0), 0.0);
    }
}
```

Add `pub mod portable;` to `crates/sugarscape-core/src/lib.rs` before `pub mod presets;`. Replace `crates/sugarscape-core/src/anasazi/random.rs` with (its `ln` and `ln` test removed, `ln` re-exported):

```rust
//! Normal draws that are bit-for-bit the same on every platform: the
//! harvest noise feeds every fingerprint, so its logarithm is
//! `crate::portable::ln`, not the platform's `f64::ln`.

use rand::Rng;

use crate::rng::SimRng;

/// The portable logarithm (moved to `crate::portable` in milestone 12).
pub use crate::portable::ln;

/// A standard normal draw (Marsaglia's polar method): pairs `(u, v)`
/// uniform in `(−1, 1)²` until `0 < s = u² + v² < 1`, then
/// `u · √(−2 ln s / s)`. Each call draws its own pairs (the second normal
/// is not kept). This is a standard textbook method, chosen (over
/// Box–Muller's trigonometric form) because it needs no `sin`/`cos` — like
/// `ln`, every step is portable, ordinary `+ − × ÷` and `sqrt`
/// arithmetic that agrees bit-for-bit between native and wasm32.
pub fn normal(rng: &mut SimRng) -> f64 {
    loop {
        let u = rng.gen::<f64>() * 2.0 - 1.0;
        let v = rng.gen::<f64>() * 2.0 - 1.0;
        let s = u * u + v * v;
        if s > 0.0 && s < 1.0 {
            return u * (-2.0 * ln(s) / s).sqrt();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    #[test]
    fn normal_draws_have_mean_0_and_sd_1() {
        let mut r = rng::seeded(11);
        let n = 200_000;
        let draws: Vec<f64> = (0..n).map(|_| normal(&mut r)).collect();
        let mean = draws.iter().sum::<f64>() / n as f64;
        let sd = (draws.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / n as f64).sqrt();
        assert!(mean.abs() < 0.01, "mean {mean}");
        assert!((sd - 1.0).abs() < 0.01, "sd {sd}");
        let below = draws.iter().filter(|&&d| d < -1.0).count() as f64 / n as f64;
        assert!((below - 0.1587).abs() < 0.005, "P(z < −1) = {below}");
    }

    #[test]
    fn draws_are_a_function_of_the_seed() {
        let a: Vec<f64> = {
            let mut r = rng::seeded(3);
            (0..5).map(|_| normal(&mut r)).collect()
        };
        let mut r = rng::seeded(3);
        assert_eq!(a, (0..5).map(|_| normal(&mut r)).collect::<Vec<_>>());
    }
}
```

Delete `crates/sugarscape-core/src/civil/math.rs` (`git rm`). In `crates/sugarscape-core/src/civil/mod.rs` remove `mod math;` and change `pub use math::exp_neg;` to `pub use crate::portable::exp_neg;` (rustfmt moves it first among the `pub use`s). In `crates/sugarscape-core/src/civil/world.rs` change `use super::math::exp_neg;` to `use crate::portable::exp_neg;`.

- [ ] **Step 2: Prove nothing moved**

Run: `cargo test -p sugarscape-core --lib portable && cargo test -p sugarscape-core --test golden && cargo test --workspace`
Expected: PASS — 2 portable tests; every golden entry unchanged (the anasazi's and civil's fingerprints depend on these functions bit for bit).

- [ ] **Step 3: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/portable.rs crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/anasazi/random.rs crates/sugarscape-core/src/civil/mod.rs crates/sugarscape-core/src/civil/world.rs
git rm -q crates/sugarscape-core/src/civil/math.rs
git commit -m "Gather the portable logarithm and exponential in one module" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 2: The spatial games in the core

**Files:**
- Create: `crates/sugarscape-core/src/spatial/{mod,config,geometry,stats,world,presets}.rs`
- Modify: `crates/sugarscape-core/src/lib.rs`, `crates/sugarscape-core/src/model.rs`, `crates/sugarscape-core/src/presets.rs`, `crates/sugarscape-core/tests/golden.rs`, `crates/sugarscape-core/tests/checkpoint.rs`

**Interfaces:**
- Consumes: `crate::portable::{ln, exp_neg}`; `crate::config::{FieldError, ScheduledChange}`; `crate::model::{wrong_model, Model, ModelConfig, ModelKind}`; `crate::presets::ModelPreset`; `crate::render::{lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, LENDER, RED}`; `crate::rng::{self, SimRng}`; `crate::schema::{Apply, Param}` (with `shown_if`); `crate::stats::{Series, Stats}`; `crate::export`; `crate::schema::check_schema` (tests).
- Produces: the core names in Global Constraints; `ModelKind::Spatial` (`"spatial"`), `ModelConfig::Spatial`, `ModelWorld::Spatial` with keyframes.

- [ ] **Step 1: Write the failing tests**

Create each file with only its tests module for now (the implementation goes above it in later steps). `config.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fields(c: &SpatialConfig) -> Vec<String> {
        c.validate()
            .err()
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.field)
            .collect()
    }

    #[test]
    fn the_default_is_nm92s_fig_1b_and_validates() {
        let c = SpatialConfig::default();
        assert!(c.validate().is_ok());
        assert_eq!(
            (c.width, c.b, c.defectors, c.self_weight),
            (200, 1.9, 0.1, 1.0)
        );
        assert_eq!(c.dims(), (200, 200, 1));
    }

    #[test]
    fn sides_radius_and_ranges_are_checked() {
        let mut c = SpatialConfig {
            width: 2,
            boundary: Boundary::Periodic,
            ..Default::default()
        };
        assert_eq!(fields(&c), ["width"], "periodic edges need 3 a side");
        c.boundary = Boundary::Fixed;
        assert!(c.validate().is_ok(), "fixed edges allow 1");
        c.lattice = Lattice::Cube;
        c.width = 65;
        assert_eq!(fields(&c), ["width"]);
        c.width = 20;
        assert_eq!(c.dims(), (20, 20, 20));
        c.lattice = Lattice::Random;
        c.radius = 10.0;
        assert_eq!(fields(&c), ["radius"], "2·10 is not below the side 20");
        c.radius = 9.5;
        assert!(c.validate().is_ok());
        for (field, value) in [("b", json!(0)), ("epsilon", json!(1)), ("m", json!(-1))] {
            let bad = ModelConfig::Spatial(SpatialConfig::default())
                .with_path(field, &value)
                .unwrap();
            assert_eq!(bad.validate().unwrap_err()[0].field, field);
        }
    }

    #[test]
    fn schedules_take_only_live_fields() {
        let entry = |path: &str, v: serde_json::Value| ScheduledChange {
            tick: 5,
            set: [(path.to_string(), v)].into_iter().collect(),
        };
        let mut c = SpatialConfig {
            schedule: vec![entry("update", json!("asynchronous"))],
            ..Default::default()
        };
        assert!(c.validate().is_ok());
        c.schedule = vec![entry("width", json!(50))];
        assert_eq!(fields(&c), ["schedule"]);
        c.schedule = vec![entry("b", json!(20))];
        assert_eq!(fields(&c), ["schedule"]);
    }

    #[test]
    fn changes_name_only_reset_fields() {
        let a = SpatialConfig::default();
        let mut b = a.clone();
        b.b = 1.5;
        b.update = Update::Asynchronous;
        b.winning = Winning::Probabilistic;
        assert!(a.changes(&b).is_empty());
        b.lattice = Lattice::Cube;
        b.defectors = 0.5;
        let mut f: Vec<String> = a.changes(&b).into_iter().map(|e| e.field).collect();
        f.sort();
        assert_eq!(f, ["defectors", "lattice"]);
    }

    #[test]
    fn partial_json_takes_defaults_and_unknown_fields_are_errors() {
        let c: SpatialConfig =
            serde_json::from_str(r#"{"lattice": "random", "update": "asynchronous"}"#).unwrap();
        assert_eq!(
            (c.lattice, c.update, c.width),
            (Lattice::Random, Update::Asynchronous, 200)
        );
        assert!(serde_json::from_str::<SpatialConfig>(r#"{"cells": 3}"#).is_err());
    }

    #[test]
    fn schema_paths_exist_and_match_what_set_config_allows() {
        let config = ModelConfig::Spatial(SpatialConfig {
            width: 40,
            height: 40,
            ..Default::default()
        });
        crate::schema::check_schema(&schema(), &config, || {
            crate::model::ModelWorld::new(config.clone(), 1).unwrap()
        });
    }
}
```

`geometry.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng;

    fn geo(edit: impl FnOnce(&mut SpatialConfig)) -> Geometry {
        let mut c = SpatialConfig {
            width: 5,
            height: 4,
            ..Default::default()
        };
        edit(&mut c);
        Geometry::new(&c, &mut rng::seeded(1))
    }

    #[test]
    fn fixed_edges_give_boundary_players_fewer_neighbors() {
        let g = geo(|_| {});
        assert_eq!(g.len(), 20);
        assert_eq!(g.neighbors(g.at(0, 0, 0).unwrap()).len(), 3, "a corner");
        assert_eq!(g.neighbors(g.at(2, 0, 0).unwrap()).len(), 5, "an edge");
        assert_eq!(g.neighbors(g.at(2, 2, 0).unwrap()).len(), 8, "inside");
        let vn = geo(|c| c.neighborhood = Neighborhood::VonNeumann);
        assert_eq!(vn.neighbors(0).len(), 2);
        assert_eq!(vn.neighbors(vn.at(2, 2, 0).unwrap()).len(), 4);
    }

    #[test]
    fn periodic_edges_wrap() {
        let g = geo(|c| c.boundary = Boundary::Periodic);
        let corner = g.neighbors(0);
        assert_eq!(corner.len(), 8);
        assert!(corner.contains(&(g.at(4, 3, 0).unwrap() as u32)));
        assert!(corner.contains(&(g.at(4, 0, 0).unwrap() as u32)));
    }

    #[test]
    fn cubes_have_26_or_6_neighbors() {
        let g = geo(|c| {
            c.lattice = Lattice::Cube;
            c.width = 4;
        });
        assert_eq!(g.len(), 64);
        assert_eq!(g.neighbors(g.at(1, 1, 1).unwrap()).len(), 26);
        assert_eq!(g.neighbors(0).len(), 7, "a corner of the cube");
        let p = geo(|c| {
            c.lattice = Lattice::Cube;
            c.width = 4;
            c.boundary = Boundary::Periodic;
            c.neighborhood = Neighborhood::VonNeumann;
        });
        assert_eq!(p.neighbors(0).len(), 6);
        assert!(p.neighbors(0).contains(&(p.at(0, 0, 3).unwrap() as u32)));
    }

    #[test]
    fn random_arrays_hold_a_fraction_and_meet_within_the_radius() {
        let mut c = SpatialConfig {
            lattice: Lattice::Random,
            width: 40,
            height: 40,
            occupancy: 0.25,
            radius: 3.0,
            ..Default::default()
        };
        let g = Geometry::new(&c, &mut rng::seeded(4));
        assert_eq!(g.len(), 400);
        assert!(g.cell.windows(2).all(|w| w[0] < w[1]), "ascending");
        for i in 0..g.len() {
            let (x, y, _) = g.xyz(i);
            for &j in g.neighbors(i) {
                let (a, b, _) = g.xyz(j as usize);
                let (dx, dy) = (a as f64 - x as f64, b as f64 - y as f64);
                assert!(dx * dx + dy * dy <= 9.0);
            }
        }
        c.occupancy = 1.0;
        let full = Geometry::new(&c, &mut rng::seeded(4));
        assert_eq!(full.neighbors(full.at(20, 20, 0).unwrap()).len(), 28);
    }

    #[test]
    fn the_central_player_is_at_the_middle() {
        let g = geo(|c| {
            c.width = 99;
            c.height = 99;
        });
        assert_eq!(g.xyz(g.central()), (49, 49, 0));
        let even = geo(|c| {
            c.width = 4;
            c.height = 4;
        });
        assert_eq!(even.xyz(even.central()), (1, 1, 0));
    }
}
```

`world.rs` (hand-built lattices; `paint` sets rectangles of strategies; `winner` and `p_c` take a score function so ties and weights can be set exactly):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ScheduledChange;
    use crate::spatial::config::{Boundary, Lattice};
    use serde_json::json;

    /// A `w` × `h` lattice of cooperators with fixed edges, after `edit`.
    fn world(w: u32, h: u32, edit: impl FnOnce(&mut SpatialConfig)) -> SpatialWorld {
        let mut c = SpatialConfig {
            width: w,
            height: h,
            defectors: 0.0,
            ..Default::default()
        };
        edit(&mut c);
        SpatialWorld::new(c, 1).unwrap()
    }

    /// Makes the players in the rectangle [x0, x1) × [y0, y1) defectors (or
    /// cooperators) and rescores.
    fn paint(w: &mut SpatialWorld, (x0, x1): (u32, u32), (y0, y1): (u32, u32), c: bool) {
        for y in y0..y1 {
            for x in x0..x1 {
                let i = w.geometry.at(x, y, 0).unwrap();
                w.coop[i] = c;
            }
        }
        w.previous.clone_from(&w.coop);
        w.rescore_all();
    }

    fn defectors(w: &SpatialWorld) -> usize {
        w.coop.iter().filter(|&&c| !c).count()
    }

    #[test]
    fn scores_sum_the_games_with_neighbors_and_self() {
        let mut w = world(5, 5, |c| c.b = 1.5);
        let mid = w.geometry.at(2, 2, 0).unwrap();
        assert_eq!(w.score(mid), 9.0, "a C among 8 C, with itself");
        paint(&mut w, (2, 3), (2, 3), false);
        assert_eq!(w.score(mid), 12.0, "a D among 8 C scores 8b");
        let next = w.geometry.at(1, 2, 0).unwrap();
        assert_eq!(w.score(next), 8.0, "a C beside the D: 7 C and itself");
        let mut solo = world(5, 5, |c| {
            c.self_weight = 0.0;
            c.epsilon = 0.1;
        });
        assert_eq!(solo.score(mid), 8.0, "no self-interaction");
        paint(&mut solo, (0, 5), (0, 5), false);
        assert!((solo.score(mid) - 0.8).abs() < 1e-12, "8 D neighbors × ε");
        solo.config.self_weight = 1.0;
        assert!((solo.score(mid) - 0.9).abs() < 1e-12, "plus a × ε");
    }

    #[test]
    fn nm92s_cluster_thresholds_hold_exactly() {
        // "If b > 1.8, a 2 × 2 cluster of D will continue to grow … for b <
        // 1.8, big D clusters shrink."
        for (b, grows) in [(1.85, true), (1.75, false)] {
            let mut w = world(30, 30, |c| c.b = b);
            paint(&mut w, (12, 18), (12, 18), false);
            w.step();
            assert_eq!(defectors(&w) > 36, grows, "a 6 × 6 D block at b = {b}");
        }
        let mut w = world(30, 30, |c| c.b = 1.85);
        paint(&mut w, (14, 16), (14, 16), false);
        w.step();
        assert!(defectors(&w) > 4, "a 2 × 2 D cluster grows at b = 1.85");
        // "If b < 2, a 2 × 2 or larger cluster of C will continue to grow;
        // for b > 2, C clusters do not grow."
        for (b, grows) in [(1.95, true), (2.05, false)] {
            let mut w = world(30, 30, |c| c.b = b);
            paint(&mut w, (0, 30), (0, 30), false);
            paint(&mut w, (14, 16), (14, 16), true);
            w.step();
            let cs = w.coop.iter().filter(|&&c| c).count();
            assert_eq!(cs > 4, grows, "a 2 × 2 C cluster at b = {b}");
        }
    }

    #[test]
    fn the_best_candidate_wins_and_a_c_d_tie_keeps_the_owner() {
        let mut w = world(3, 3, |_| {});
        paint(&mut w, (0, 1), (0, 1), false);
        let mid = w.geometry.at(1, 1, 0).unwrap();
        let corner = w.geometry.at(0, 0, 0).unwrap();
        let scores = |best: usize, top: f64| {
            move |_: &SpatialWorld, j: usize| if j == best { top } else { 1.0 }
        };
        assert!(
            !w.winner(mid, scores(corner, 5.0)),
            "the D corner scores highest"
        );
        assert!(
            w.winner(mid, scores(mid, 5.0)),
            "the C owner scores highest"
        );
        // The D corner and the C owner tie for the top score.
        let tie = |_: &SpatialWorld, j: usize| if j == corner || j == mid { 5.0 } else { 1.0 };
        assert!(w.winner(mid, tie), "C owner keeps its site");
        paint(&mut w, (1, 2), (1, 2), false);
        assert!(!w.winner(mid, tie), "D owner keeps its site");
    }

    #[test]
    fn eq_1_weighs_candidates_by_their_scores() {
        let mut w = world(3, 3, |c| c.winning = Winning::Probabilistic);
        paint(&mut w, (0, 3), (0, 1), false); // the top row defects
        let mid = w.geometry.at(1, 1, 0).unwrap();
        w.config.m = 0.0;
        let p = w.p_c(mid, |w, j| w.scores[j]).unwrap();
        assert!((p - 6.0 / 9.0).abs() < 1e-12, "m = 0: the share of C, {p}");
        w.config.m = 1.0;
        let flat = |w: &SpatialWorld, j: usize| if w.coop[j] { 1.0 } else { 3.0 };
        let p = w.p_c(mid, flat).unwrap();
        assert!((p - 6.0 / 15.0).abs() < 1e-12, "m = 1: proportional, {p}");
        w.config.m = 200.0;
        let p = w.p_c(mid, flat).unwrap();
        assert!(p < 1e-12, "large m: the best scorer, {p}");
        let zero = |_: &SpatialWorld, _: usize| 0.0;
        assert_eq!(w.p_c(mid, zero), None, "every score 0: the owner keeps it");
        w.config.m = 0.0;
        assert!(
            (w.p_c(mid, zero).unwrap() - 6.0 / 9.0).abs() < 1e-12,
            "0^0 = 1"
        );
    }

    #[test]
    fn portable_powers_match_the_library() {
        for (a, max, m) in [
            (3.0, 9.0, 1.0),
            (0.5, 8.0, 20.0),
            (7.2, 7.2, 100.0),
            (1.3, 12.0, 0.5),
        ] {
            let ours = exp_neg(m * (ln(a) - ln(max)));
            let std = (a / max).powf(m);
            assert!(
                (ours - std).abs() <= 1e-12 * std.max(1e-300),
                "{a} {max} {m}: {ours} vs {std}"
            );
        }
    }

    #[test]
    fn an_asynchronous_microstep_sees_the_sites_already_changed() {
        let mut w = world(5, 1, |c| {
            c.b = 1.9;
            c.update = Update::Asynchronous;
        });
        // C C D C C in a row: the D's neighbors each score 1 + 1 = 2 (one C
        // neighbor and itself); the D scores 2b = 3.8 and takes both.
        paint(&mut w, (2, 3), (0, 1), false);
        let (left, right) = (
            w.geometry.at(1, 0, 0).unwrap(),
            w.geometry.at(3, 0, 0).unwrap(),
        );
        w.microstep(left);
        assert!(!w.coop[left], "the left C falls to the D");
        // Now the D at (2, 0) has one C neighbor (right) and scores 1.9;
        // (3, 0) scores 2 with its C neighbor (4, 0) and itself: it stays C.
        w.microstep(right);
        assert!(w.coop[right], "rescored after the left site fell");
    }

    #[test]
    fn starts_place_exact_counts_and_the_single_defector_at_the_center() {
        let w = SpatialWorld::new(
            SpatialConfig {
                width: 50,
                height: 40,
                defectors: 0.25,
                ..Default::default()
            },
            3,
        )
        .unwrap();
        assert_eq!(defectors(&w), 500);
        let k = world(99, 99, |c| c.start = Start::SingleDefector);
        assert_eq!(defectors(&k), 1);
        assert!(!k.coop[k.geometry.at(49, 49, 0).unwrap()]);
    }

    #[test]
    fn the_change_colours_follow_nm92() {
        let mut w = world(2, 2, |_| {});
        let (a, b, c, d) = (0, 1, 2, 3);
        w.previous = vec![true, false, true, false];
        w.coop = vec![true, false, false, true];
        let mut buf = Vec::new();
        w.render("change", "", &mut buf).unwrap();
        let px = |buf: &[u8], i: usize| [buf[i * 4], buf[i * 4 + 1], buf[i * 4 + 2]];
        assert_eq!(px(&buf, a), C_AFTER_C);
        assert_eq!(px(&buf, b), D_AFTER_D);
        assert_eq!(px(&buf, c), D_AFTER_C);
        assert_eq!(px(&buf, d), C_AFTER_D);
        w.render("strategy", "", &mut buf).unwrap();
        assert_eq!(px(&buf, c), D_AFTER_D);
        assert!(w.render("nope", "", &mut buf).is_err());
    }

    #[test]
    fn a_cube_draws_the_chosen_slice_and_inspects_it() {
        let mut w = world(4, 4, |c| {
            c.lattice = Lattice::Cube;
            c.boundary = Boundary::Periodic;
        });
        let top = w.geometry.at(1, 2, 3).unwrap();
        w.coop[top] = false;
        w.previous.clone_from(&w.coop);
        w.rescore_all();
        let mut buf = Vec::new();
        w.render("strategy", "slice:3", &mut buf).unwrap();
        assert_eq!(buf.len(), 4 * 4 * 4);
        assert_eq!(&buf[(2 * 4 + 1) * 4..(2 * 4 + 1) * 4 + 3], &D_AFTER_D);
        let seen: serde_json::Value = serde_json::from_str(&w.inspect_json(1, 2).unwrap()).unwrap();
        assert_eq!(
            seen["site"]["z"], 3,
            "Inspect looks in the slice last drawn"
        );
        assert_eq!(seen["agent"]["strategy"], "D");
        assert_eq!(seen["agent"]["candidates"].as_array().unwrap().len(), 27);
        w.render("strategy", "slice:99", &mut buf).unwrap();
        assert_eq!(w.view_z.get(), 3, "clamped to the cube");
    }

    #[test]
    fn schedules_change_live_fields_at_their_tick() {
        let mut w = world(10, 10, |c| {
            c.schedule = vec![ScheduledChange {
                tick: 2,
                set: [("b".to_string(), json!(1.5))].into_iter().collect(),
            }];
        });
        w.run(2);
        assert_eq!(w.config.b, 1.9);
        w.step();
        assert_eq!(w.config.b, 1.5);
    }

    #[test]
    fn worlds_follow_their_seed() {
        let c = SpatialConfig {
            width: 40,
            height: 40,
            update: Update::Asynchronous,
            winning: Winning::Probabilistic,
            ..Default::default()
        };
        let mut a = SpatialWorld::new(c.clone(), 7).unwrap();
        let mut b = SpatialWorld::new(c.clone(), 7).unwrap();
        let mut other = SpatialWorld::new(c, 8).unwrap();
        for w in [&mut a, &mut b, &mut other] {
            w.run(20);
        }
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.fingerprint(), other.fingerprint());
    }

    #[test]
    fn inspect_names_the_next_owner() {
        let mut w = world(5, 5, |_| {});
        paint(&mut w, (2, 3), (2, 3), false);
        let v = w.inspect(1, 2, 0).unwrap().agent.unwrap();
        assert_eq!((v.strategy, v.next, v.p_c), ("C", Some("D"), None));
        assert_eq!(v.candidates.len(), 9);
        assert_eq!(v.candidates[0].strategy, "C", "itself first");
        assert!(w.inspect(5, 0, 0).is_err());
    }
}
```

In `crates/sugarscape-core/src/model.rs`'s tests, before `only_the_anasazi_finishes`:

```rust
    #[test]
    fn spatial_configs_round_trip_with_their_tag() {
        let c = ModelConfig::from_json(
            r#"{"model": "spatial", "lattice": "cube", "width": 10, "update": "asynchronous"}"#,
        )
        .unwrap();
        assert_eq!(c.kind(), ModelKind::Spatial);
        let json = serde_json::to_value(&c).unwrap();
        assert_eq!(json["model"], "spatial");
        assert_eq!(ModelConfig::from_value(json).unwrap(), c);
        assert_eq!(c.series_names()[0], "fraction_c");
        let next = c.with_path("b", &json!(1.6)).unwrap();
        let ModelConfig::Spatial(s) = &next else {
            unreachable!()
        };
        assert_eq!(s.b, 1.6);
        let e = ModelConfig::from_json(r#"{"model": "spatial", "b": 0}"#).unwrap_err();
        assert_eq!(e[0].field, "b");
        let mut w = ModelWorld::new(c, 1).unwrap();
        assert_eq!((w.kind(), w.model().size()), (ModelKind::Spatial, (10, 10)));
        assert_eq!(w.model().population(), 1000);
        let cp = w.checkpoint().expect("spatial worlds have keyframes");
        w.model_mut().run(3);
        w.restore(&cp).unwrap();
        assert_eq!(w.model().tick(), 0);
    }
```

and the expected names in `every_kind_names_itself_and_only_other_models_have_schemas` end with `"civil", "spatial"`. In `crates/sugarscape-core/tests/golden.rs` append to `MODEL_GOLDEN` after `("cv-netlogo", 0x87a92345c017b0ae),` the twelve entries of Decision 13, in that order. In `crates/sugarscape-core/tests/checkpoint.rs` add `"nbm-probabilistic",` to `IDS` after `"cv-run-8-nasty-regime",`.

Run: `cargo test -p sugarscape-core --lib spatial` — Expected: compile errors.

- [ ] **Step 2: Wire the model kind**

`lib.rs`: `pub mod spatial;` after `pub mod schema;`. `model.rs`, each beside civil's: `use crate::spatial::{SpatialConfig, SpatialWorld};`; `spatial` in the `use crate::{…}` list; `ModelKind::Spatial` (after `Civil`, `ALL: [ModelKind; 6]`, `as_str` `"spatial"`, `schema` `spatial::schema()`); `ModelConfig::Spatial(SpatialConfig)`, `Tagged::Spatial(&'a SpatialConfig)`, the serialize and `kind` arms; `from_value`'s `"spatial"` arm (as civil's) and the unknown-model message `"(expected sugarscape, schelling, ring, anasazi, civil or spatial)"`; `validate`, `with_path` (`set_path`), `max_ticks` (`None` arm), `series_names` (`spatial::SERIES`); `ModelWorld::Spatial(Box<SpatialWorld>)` in `with_landscapes`, `kind`, `model`, `model_mut`; and in `checkpoint`/`restore`:

```rust
            ModelWorld::Spatial(w) => copy_without_history!(Spatial, w),
```

```rust
            (ModelWorld::Spatial(live), ModelWorld::Spatial(kept)) => restore_into!(live, kept),
```

`presets.rs`: `out.extend(crate::spatial::presets());` after civil's; the doc comment ends "the anasazi's, civil violence's and the spatial games'."

- [ ] **Step 3: Implement**

`config.rs`, above its tests (Decisions 2, 7; the schema's groups and labels):

```rust
//! The spatial games' parameters: NM92's lattice game with HG93's
//! asynchronous updating and NBM94's probabilistic winning, irregular
//! arrays and cubes, and a schedule of live changes.

use serde::{Deserialize, Serialize};

use crate::config::{FieldError, ScheduledChange};
use crate::model::ModelConfig;
use crate::schema::{Apply, Param};

/// Where the players sit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lattice {
    /// Every cell of a width × height grid holds a player.
    #[default]
    Square,
    /// Every cell of an n × n × n cube (n = width).
    Cube,
    /// NBM94's irregular arrays: a fraction of a width × height grid's
    /// cells, chosen at random, hold players.
    Random,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    /// 8 neighbors in a square, 26 in a cube.
    #[default]
    Moore,
    /// 4 neighbors in a square, 6 in a cube.
    VonNeumann,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    /// Players at the edges have fewer neighbors (NM92's figures).
    #[default]
    Fixed,
    Periodic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    /// Discrete time: every site at once (NM92).
    #[default]
    Synchronous,
    /// Continuous time: one random site at a time (HG93, NBM94 Fig. 2).
    Asynchronous,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Winning {
    /// The highest scorer takes the site (m = ∞).
    #[default]
    Deterministic,
    /// NBM94's Eq. 1 with exponent `m`.
    Probabilistic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Start {
    /// A fraction of the players, chosen at random, start as defectors.
    #[default]
    Random,
    /// One defector at the center, every other player a cooperator.
    SingleDefector,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpatialConfig {
    pub lattice: Lattice,
    pub width: u32,
    pub height: u32,
    pub neighborhood: Neighborhood,
    pub boundary: Boundary,
    /// Random arrays: the fraction of cells holding a player.
    pub occupancy: f64,
    /// Random arrays: the interaction radius, in cells.
    pub radius: f64,
    /// The temptation T = b (R = 1, S = 0).
    pub b: f64,
    /// P = ε.
    pub epsilon: f64,
    /// a: the weight of the payoff a player earns against itself.
    pub self_weight: f64,
    pub update: Update,
    pub winning: Winning,
    /// Eq. 1's exponent when winning is probabilistic.
    pub m: f64,
    pub start: Start,
    /// A random start's fraction of defectors.
    pub defectors: f64,
    pub schedule: Vec<ScheduledChange>,
}

impl Default for SpatialConfig {
    /// NM92's Fig. 1b: 200 × 200, fixed edges, eight neighbors and self,
    /// 10% defectors, b = 1.9 (spatial chaos).
    fn default() -> Self {
        SpatialConfig {
            lattice: Lattice::Square,
            width: 200,
            height: 200,
            neighborhood: Neighborhood::Moore,
            boundary: Boundary::Fixed,
            occupancy: 0.05,
            radius: 5.0,
            b: 1.9,
            epsilon: 0.0,
            self_weight: 1.0,
            update: Update::Synchronous,
            winning: Winning::Deterministic,
            m: 1.0,
            start: Start::Random,
            defectors: 0.1,
            schedule: Vec::new(),
        }
    }
}

/// The fields that apply to a running world; every other field rebuilds it.
pub const LIVE: [&str; 6] = ["b", "epsilon", "self_weight", "update", "winning", "m"];

impl SpatialConfig {
    /// The side lengths of the lattice or base grid: (x, y, z).
    pub fn dims(&self) -> (u32, u32, u32) {
        match self.lattice {
            Lattice::Cube => (self.width, self.width, self.width),
            Lattice::Square | Lattice::Random => (self.width, self.height, 1),
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
        // Periodic edges need 3 cells a side, or a neighbor would be the
        // player itself or counted twice.
        let min = if self.boundary == Boundary::Periodic {
            3
        } else {
            1
        };
        match self.lattice {
            Lattice::Cube => check(
                (min..=64).contains(&self.width),
                "width",
                &format!("a cube's side must be between {min} and 64"),
            ),
            Lattice::Square | Lattice::Random => {
                let message = format!("must be between {min} and 1000");
                check((min..=1000).contains(&self.width), "width", &message);
                check((min..=1000).contains(&self.height), "height", &message);
            }
        }
        if self.lattice == Lattice::Random {
            check(
                self.occupancy > 0.0 && self.occupancy <= 1.0,
                "occupancy",
                "must be above 0 and at most 1",
            );
            let side = f64::from(self.width.min(self.height));
            check(
                self.radius >= 1.0 && 2.0 * self.radius.floor() < side,
                "radius",
                "must be at least 1 and less than half the grid's smaller side",
            );
        }
        check(
            self.b.is_finite() && self.b > 0.0 && self.b <= 10.0,
            "b",
            "must be above 0 and at most 10",
        );
        check(
            self.epsilon == 0.0 || (1e-6..1.0).contains(&self.epsilon),
            "epsilon",
            "must be 0, or at least 0.000001 and below 1",
        );
        check(
            (0.0..=10.0).contains(&self.self_weight),
            "self_weight",
            "must be between 0 and 10",
        );
        check(
            self.m.is_finite() && (0.0..=1000.0).contains(&self.m),
            "m",
            "must be between 0 and 1000",
        );
        check(
            (0.0..=1.0).contains(&self.defectors),
            "defectors",
            "must be 0–1",
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
                match ModelConfig::Spatial(self.clone()).with_path(path, value) {
                    Ok(ModelConfig::Spatial(next)) => {
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
    pub fn changes(&self, next: &SpatialConfig) -> Vec<FieldError> {
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
    vec![
        Param::number("Game", "b", "Temptation (b)", (1.0, 3.0, 0.01), Live)
            .with_help("A defector's payoff against a cooperator (R = 1, S = 0)"),
        Param::number(
            "Game",
            "epsilon",
            "Mutual defection (ε)",
            (0.0, 0.5, 0.01),
            Live,
        )
        .with_help("P = ε: the papers use 0"),
        Param::number(
            "Game",
            "self_weight",
            "Self-interaction (a)",
            (0.0, 2.0, 0.05),
            Live,
        )
        .with_help("The weight of the game a player plays against itself (0: none)"),
        Param::choice(
            "Lattice",
            "lattice",
            "Lattice",
            &[
                ("square", "Square"),
                ("cube", "Cube (n × n × n)"),
                ("random", "Random array"),
            ],
            Reset,
        ),
        Param::integer("Lattice", "width", "Width (a cube's side)", (3, 400), Reset),
        Param::integer("Lattice", "height", "Height", (3, 400), Reset),
        Param::choice(
            "Lattice",
            "neighborhood",
            "Neighbors",
            &[
                ("moore", "Moore (8, or 26 in a cube)"),
                ("von_neumann", "von Neumann (4, or 6)"),
            ],
            Reset,
        ),
        Param::choice(
            "Lattice",
            "boundary",
            "Edges",
            &[("fixed", "Fixed"), ("periodic", "Periodic (wrap)")],
            Reset,
        ),
        Param::number(
            "Lattice",
            "occupancy",
            "Cells occupied",
            (0.01, 1.0, 0.01),
            Reset,
        )
        .shown_if("lattice", "random"),
        Param::number(
            "Lattice",
            "radius",
            "Interaction radius",
            (1.0, 20.0, 0.5),
            Reset,
        )
        .shown_if("lattice", "random"),
        Param::choice(
            "Update",
            "update",
            "Update",
            &[
                ("synchronous", "Synchronous (discrete time)"),
                ("asynchronous", "Asynchronous (continuous time)"),
            ],
            Live,
        )
        .with_help("Asynchronous: one random site at a time, as Huberman and Glance proposed"),
        Param::choice(
            "Update",
            "winning",
            "Winning",
            &[
                ("deterministic", "Highest score wins"),
                ("probabilistic", "Probabilistic (Eq. 1)"),
            ],
            Live,
        ),
        Param::number("Update", "m", "Exponent (m)", (0.0, 100.0, 0.5), Live)
            .with_help("P(C) = Σ A^m (C) / Σ A^m: 0 is random drift, 1 proportional")
            .shown_if("winning", "probabilistic"),
        Param::choice(
            "Start",
            "start",
            "Start",
            &[
                ("random", "Random defectors"),
                ("single_defector", "One defector at the center"),
            ],
            Reset,
        ),
        Param::number("Start", "defectors", "Defectors", (0.0, 1.0, 0.01), Reset)
            .shown_if("start", "random"),
    ]
}
```

`geometry.rs`, above its tests (Decision 2):

```rust
//! Where the spatial games' players sit and whom each plays: square and
//! cubic lattices (Moore or von Neumann, fixed or periodic edges) and NBM94's
//! random arrays (a fraction of a grid's cells, neighbors within a radius).

use rand::seq::SliceRandom;

use super::config::{Boundary, Lattice, Neighborhood, SpatialConfig};
use crate::rng::SimRng;

/// No player on a cell (random arrays).
pub const EMPTY: u32 = u32::MAX;

/// Players, their cells and their neighbor lists.
#[derive(Clone, Debug)]
pub struct Geometry {
    /// The grid's sides (x, y, z); z = 1 except in a cube.
    pub dims: (u32, u32, u32),
    /// Each player's cell, `x + y·w + z·w·h`, in player order (ascending).
    pub cell: Vec<u32>,
    /// Each cell's player, or `EMPTY`.
    pub player: Vec<u32>,
    /// Player i's neighbors are `neighbors[start[i]..start[i + 1]]`, in
    /// ascending order of offset (z, then y, then x).
    start: Vec<u32>,
    neighbors: Vec<u32>,
}

impl Geometry {
    /// Builds the lattice of `c`; a random array draws its cells from `rng`.
    pub fn new(c: &SpatialConfig, rng: &mut SimRng) -> Self {
        let dims = c.dims();
        let cells = (dims.0 * dims.1 * dims.2) as usize;
        let cell: Vec<u32> = match c.lattice {
            Lattice::Square | Lattice::Cube => (0..cells as u32).collect(),
            Lattice::Random => {
                let k = (c.occupancy * cells as f64).round() as usize;
                let mut all: Vec<u32> = (0..cells as u32).collect();
                all.shuffle(rng);
                let mut chosen = all[..k.max(1)].to_vec();
                chosen.sort_unstable();
                chosen
            }
        };
        let mut player = vec![EMPTY; cells];
        for (i, &x) in cell.iter().enumerate() {
            player[x as usize] = i as u32;
        }
        let offsets = offsets(c);
        let periodic = c.boundary == Boundary::Periodic;
        let (w, h, d) = (dims.0 as i64, dims.1 as i64, dims.2 as i64);
        let mut start = Vec::with_capacity(cell.len() + 1);
        let mut neighbors = Vec::new();
        for &x in &cell {
            start.push(neighbors.len() as u32);
            let (px, py, pz) = unpack(x, dims);
            for &(dx, dy, dz) in &offsets {
                let (mut nx, mut ny, mut nz) = (px as i64 + dx, py as i64 + dy, pz as i64 + dz);
                if periodic {
                    nx = nx.rem_euclid(w);
                    ny = ny.rem_euclid(h);
                    nz = nz.rem_euclid(d);
                } else if nx < 0 || ny < 0 || nz < 0 || nx >= w || ny >= h || nz >= d {
                    continue;
                }
                let j = player[(nx + ny * w + nz * w * h) as usize];
                if j != EMPTY {
                    neighbors.push(j);
                }
            }
        }
        start.push(neighbors.len() as u32);
        Geometry {
            dims,
            cell,
            player,
            start,
            neighbors,
        }
    }

    pub fn len(&self) -> usize {
        self.cell.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cell.is_empty()
    }

    /// Player `i`'s neighbors (not itself).
    pub fn neighbors(&self, i: usize) -> &[u32] {
        &self.neighbors[self.start[i] as usize..self.start[i + 1] as usize]
    }

    /// Player `i`'s cell as (x, y, z).
    pub fn xyz(&self, i: usize) -> (u32, u32, u32) {
        unpack(self.cell[i], self.dims)
    }

    /// The player on cell (x, y, z), if any.
    pub fn at(&self, x: u32, y: u32, z: u32) -> Option<usize> {
        let (w, h, d) = self.dims;
        if x >= w || y >= h || z >= d {
            return None;
        }
        let p = self.player[(x + y * w + z * w * h) as usize];
        (p != EMPTY).then_some(p as usize)
    }

    /// The player nearest the grid's center ((n − 1)/2 rounded down on each
    /// axis): the lowest-numbered at the smallest squared distance.
    pub fn central(&self) -> usize {
        let (w, h, d) = self.dims;
        let c = ((w - 1) / 2, (h - 1) / 2, (d - 1) / 2);
        let dist = |i: usize| {
            let (x, y, z) = self.xyz(i);
            let (dx, dy, dz) = (
                x as i64 - c.0 as i64,
                y as i64 - c.1 as i64,
                z as i64 - c.2 as i64,
            );
            dx * dx + dy * dy + dz * dz
        };
        (0..self.len()).min_by_key(|&i| (dist(i), i)).unwrap_or(0)
    }
}

fn unpack(cell: u32, (w, h, _): (u32, u32, u32)) -> (u32, u32, u32) {
    (cell % w, (cell / w) % h, cell / (w * h))
}

/// The neighbor offsets of `c`'s lattice (excluding (0, 0, 0)), ascending by
/// (dz, dy, dx).
pub fn offsets(c: &SpatialConfig) -> Vec<(i64, i64, i64)> {
    let mut out = Vec::new();
    match c.lattice {
        Lattice::Random => {
            let r = c.radius.floor() as i64;
            let r2 = c.radius * c.radius;
            for dy in -r..=r {
                for dx in -r..=r {
                    if (dx, dy) != (0, 0) && ((dx * dx + dy * dy) as f64) <= r2 {
                        out.push((dx, dy, 0));
                    }
                }
            }
        }
        Lattice::Square | Lattice::Cube => {
            let zs: &[i64] = if c.lattice == Lattice::Cube {
                &[-1, 0, 1]
            } else {
                &[0]
            };
            for &dz in zs {
                for dy in -1..=1i64 {
                    for dx in -1..=1i64 {
                        let manhattan = dx.abs() + dy.abs() + dz.abs();
                        let keep = match c.neighborhood {
                            Neighborhood::Moore => manhattan > 0,
                            Neighborhood::VonNeumann => manhattan == 1,
                        };
                        if keep {
                            out.push((dx, dy, dz));
                        }
                    }
                }
            }
        }
    }
    out
}
```

`stats.rs`:

```rust
//! The spatial games' statistics.

use serde::Serialize;

use crate::stats::Series;

/// The statistics series, in the order the CSV and the page list them.
pub const SERIES: [&str; 7] = [
    "fraction_c",
    "changed",
    "c_to_d",
    "d_to_c",
    "mean_payoff_c",
    "mean_payoff_d",
    "players",
];

/// One generation's statistics.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SpatialSnapshot {
    pub tick: u64,
    /// The share of players cooperating.
    pub fraction_c: f64,
    /// The share of players whose strategy changed this generation.
    pub changed: f64,
    /// Players that switched from C to D, and from D to C.
    pub c_to_d: u32,
    pub d_to_c: u32,
    /// Mean score of the cooperators and of the defectors (NaN with none;
    /// JSON null).
    pub mean_payoff_c: f64,
    pub mean_payoff_d: f64,
    pub players: u32,
}

impl Series for SpatialSnapshot {
    fn tick(&self) -> u64 {
        self.tick
    }

    fn value(&self, name: &str) -> Option<f64> {
        Some(match name {
            "tick" => self.tick as f64,
            "fraction_c" => self.fraction_c,
            "changed" => self.changed,
            "c_to_d" => f64::from(self.c_to_d),
            "d_to_c" => f64::from(self.d_to_c),
            "mean_payoff_c" => self.mean_payoff_c,
            "mean_payoff_d" => self.mean_payoff_d,
            "players" => f64::from(self.players),
            _ => return None,
        })
    }
}
```

`world.rs`, above its tests (Decisions 3–6, 8):

```rust
//! The spatial Prisoner's Dilemma world: cooperators and defectors on a
//! lattice or random array, each scoring the sum of its games with its
//! neighbors and itself, each site then taken by the best-scoring candidate
//! (or by NBM94's Eq. 1), all at once or one site at a time.

use std::cell::Cell;
use std::fmt::Write;

use rand::seq::SliceRandom;
use rand::Rng;
use serde::Serialize;

use super::config::{SpatialConfig, Start, Update, Winning};
use super::geometry::Geometry;
use super::stats::{SpatialSnapshot, SERIES};
use crate::config::FieldError;
use crate::export;
use crate::model::{wrong_model, Model, ModelConfig, ModelKind};
use crate::portable::{exp_neg, ln};
use crate::render::{lerp, Rgb, BACKGROUND, BLUE, BOTH, COOL, HOT, LENDER, RED};
use crate::rng::{self, SimRng};
use crate::stats::Stats;

/// NM92's colours: blue C after C, red D after D, yellow D after C, green C
/// after D.
pub const C_AFTER_C: Rgb = BLUE;
pub const D_AFTER_D: Rgb = RED;
pub const D_AFTER_C: Rgb = BOTH;
pub const C_AFTER_D: Rgb = LENDER;

/// The colour modes the frame can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpatialMode {
    /// NM92's four colours.
    Change,
    /// Blue C, red D.
    Strategy,
    /// Cool to hot by score.
    Payoff,
}

impl std::str::FromStr for SpatialMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s {
            "change" => Self::Change,
            "strategy" => Self::Strategy,
            "payoff" => Self::Payoff,
            _ => return Err(format!("unknown color mode {s:?}")),
        })
    }
}

/// What Inspect shows for a cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SpatialInspection {
    pub site: CellXyz,
    /// The player on the cell (none on an empty cell of a random array).
    pub agent: Option<PlayerView>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct CellXyz {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlayerView {
    pub id: u64,
    /// "C" or "D", now and a generation ago.
    pub strategy: &'static str,
    pub previous: &'static str,
    pub score: f64,
    /// The player itself first, then its neighbors: who could take the site.
    pub candidates: Vec<Candidate>,
    /// The strategy the site would take next (deterministic winning), or
    /// null (probabilistic).
    pub next: Option<&'static str>,
    /// Eq. 1's probability that the site becomes C (probabilistic winning).
    pub p_c: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Candidate {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub strategy: &'static str,
    pub score: f64,
}

fn letter(c: bool) -> &'static str {
    if c {
        "C"
    } else {
        "D"
    }
}

#[derive(Clone)]
pub struct SpatialWorld {
    pub config: SpatialConfig,
    pub geometry: Geometry,
    /// Completed generations.
    pub tick: u64,
    /// Each player's strategy (true: C), now and a generation ago.
    coop: Vec<bool>,
    previous: Vec<bool>,
    /// Each player's score against its neighbors and itself, for `coop`.
    scores: Vec<f64>,
    rng: SimRng,
    /// The z-slice last drawn (a cube's view): where Inspect looks.
    view_z: Cell<u32>,
    pub stats: Stats<SpatialSnapshot>,
}

impl SpatialWorld {
    pub fn new(config: SpatialConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let geometry = Geometry::new(&config, &mut rng);
        let n = geometry.len();
        let mut coop = vec![true; n];
        match config.start {
            Start::Random => {
                let k = (config.defectors * n as f64).round() as usize;
                let mut order: Vec<usize> = (0..n).collect();
                order.shuffle(&mut rng);
                for &i in &order[..k] {
                    coop[i] = false;
                }
            }
            Start::SingleDefector => coop[geometry.central()] = false,
        }
        let mut w = SpatialWorld {
            config,
            previous: coop.clone(),
            scores: vec![0.0; n],
            coop,
            view_z: Cell::new(geometry.dims.2 / 2),
            geometry,
            tick: 0,
            rng,
            stats: Stats::default(),
        };
        w.rescore_all();
        w.record();
        Ok(w)
    }

    pub fn players(&self) -> usize {
        self.geometry.len()
    }

    pub fn is_cooperator(&self, i: usize) -> bool {
        self.coop[i]
    }

    /// Player `i`'s score for the current strategies: the sum over its
    /// neighbors of its payoff against each, plus a × its payoff against
    /// itself. A cooperator with k cooperating neighbors scores k + a; a
    /// defector scores k·b + (n − k)·ε + a·ε.
    pub fn score(&self, i: usize) -> f64 {
        let nb = self.geometry.neighbors(i);
        let k = nb.iter().filter(|&&j| self.coop[j as usize]).count() as f64;
        let c = &self.config;
        if self.coop[i] {
            k + c.self_weight
        } else {
            k * c.b + (nb.len() as f64 - k) * c.epsilon + c.self_weight * c.epsilon
        }
    }

    fn rescore_all(&mut self) {
        self.scores = (0..self.players()).map(|i| self.score(i)).collect();
    }

    /// The strategy site `i` takes from its candidates (itself and its
    /// neighbors) with scores `score(j)`; probabilistic winning draws one
    /// uniform number.
    fn next_strategy(&mut self, i: usize, score: impl Fn(&Self, usize) -> f64) -> bool {
        match self.config.winning {
            Winning::Deterministic => self.winner(i, score),
            Winning::Probabilistic => {
                let u = self.rng.gen::<f64>();
                match self.p_c(i, &score) {
                    Some(p) => u < p,
                    None => self.coop[i],
                }
            }
        }
    }

    /// Deterministic winning: the strategy of the highest-scoring candidate;
    /// on a tie between a C and a D, the owner keeps its strategy.
    fn winner(&self, i: usize, score: impl Fn(&Self, usize) -> f64) -> bool {
        let candidates =
            std::iter::once(i as u32).chain(self.geometry.neighbors(i).iter().copied());
        let (mut best, mut c_top, mut d_top) = (f64::NEG_INFINITY, false, false);
        for j in candidates {
            let (s, c) = (score(self, j as usize), self.coop[j as usize]);
            if s > best {
                (best, c_top, d_top) = (s, c, !c);
            } else if s == best {
                c_top |= c;
                d_top |= !c;
            }
        }
        if c_top && d_top {
            self.coop[i]
        } else {
            c_top
        }
    }

    /// Eq. 1: Σ A^m s / Σ A^m over the candidates, or None when every
    /// weight is 0 (every score 0 with m > 0).
    fn p_c(&self, i: usize, score: impl Fn(&Self, usize) -> f64) -> Option<f64> {
        let candidates: Vec<(f64, bool)> = std::iter::once(i as u32)
            .chain(self.geometry.neighbors(i).iter().copied())
            .map(|j| (score(self, j as usize), self.coop[j as usize]))
            .collect();
        let m = self.config.m;
        let max = candidates.iter().map(|c| c.0).fold(0.0, f64::max);
        let weight = |a: f64| {
            if m == 0.0 {
                1.0 // 0^0 = 1: random drift
            } else if a == 0.0 {
                0.0
            } else {
                // A^m / max^m, from portable arithmetic.
                exp_neg(m * (ln(a) - ln(max)))
            }
        };
        if m > 0.0 && max == 0.0 {
            return None;
        }
        let (mut c_sum, mut total) = (0.0, 0.0);
        for &(a, c) in &candidates {
            let w = weight(a);
            total += w;
            if c {
                c_sum += w;
            }
        }
        (total > 0.0).then(|| c_sum / total)
    }

    /// One generation.
    pub fn step(&mut self) {
        self.apply_schedule();
        self.previous.clone_from(&self.coop);
        match self.config.update {
            Update::Synchronous => {
                let next: Vec<bool> = (0..self.players())
                    .map(|i| self.next_strategy(i, |w, j| w.scores[j]))
                    .collect();
                self.coop = next;
            }
            Update::Asynchronous => {
                let n = self.players() as u32;
                for _ in 0..n {
                    let i = self.rng.gen_range(0..n) as usize;
                    self.microstep(i);
                }
            }
        }
        self.rescore_all();
        self.tick += 1;
        self.record();
    }

    /// Asynchronous updating's microstep: site `i` and its neighbors are
    /// rescored from the current strategies, and `i` takes its new strategy
    /// at once.
    fn microstep(&mut self, i: usize) {
        self.coop[i] = self.next_strategy(i, |w, j| w.score(j));
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
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
        if due.is_empty() {
            return;
        }
        for (path, value) in due {
            let next = ModelConfig::Spatial(self.config.clone()).with_path(&path, &value);
            debug_assert!(next.is_ok(), "validated schedule entry {path}");
            if let Ok(ModelConfig::Spatial(next)) = next {
                self.config = next;
            }
        }
        // A new b, ε or a changes every score now.
        self.rescore_all();
    }

    fn record(&mut self) {
        let n = self.players();
        let mut s = SpatialSnapshot {
            tick: self.tick,
            players: n as u32,
            ..Default::default()
        };
        let (mut cs, mut ds, mut c_pay, mut d_pay) = (0u32, 0u32, 0.0, 0.0);
        for i in 0..n {
            let (now, was) = (self.coop[i], self.previous[i]);
            if now {
                cs += 1;
                c_pay += self.scores[i];
            } else {
                ds += 1;
                d_pay += self.scores[i];
            }
            match (was, now) {
                (true, false) => s.c_to_d += 1,
                (false, true) => s.d_to_c += 1,
                _ => {}
            }
        }
        s.fraction_c = if n == 0 {
            0.0
        } else {
            f64::from(cs) / n as f64
        };
        s.changed = if n == 0 {
            0.0
        } else {
            f64::from(s.c_to_d + s.d_to_c) / n as f64
        };
        s.mean_payoff_c = if cs == 0 {
            f64::NAN
        } else {
            c_pay / f64::from(cs)
        };
        s.mean_payoff_d = if ds == 0 {
            f64::NAN
        } else {
            d_pay / f64::from(ds)
        };
        self.stats.push(s);
    }

    fn color(&self, i: usize, mode: SpatialMode, max_score: f64) -> Rgb {
        let (now, was) = (self.coop[i], self.previous[i]);
        match mode {
            SpatialMode::Change => match (was, now) {
                (true, true) => C_AFTER_C,
                (false, false) => D_AFTER_D,
                (true, false) => D_AFTER_C,
                (false, true) => C_AFTER_D,
            },
            SpatialMode::Strategy => {
                if now {
                    C_AFTER_C
                } else {
                    D_AFTER_D
                }
            }
            SpatialMode::Payoff => {
                let t = if max_score > 0.0 {
                    self.scores[i] / max_score
                } else {
                    0.0
                };
                lerp(COOL, HOT, t)
            }
        }
    }

    /// The z-slice a frame shows: `slice:<z>` in a cube (clamped), else 0.
    fn slice(&self, layer: &str) -> u32 {
        let d = self.geometry.dims.2;
        layer
            .strip_prefix("slice:")
            .and_then(|z| z.parse::<u32>().ok())
            .map_or(d / 2, |z| z.min(d - 1))
    }

    pub fn inspect(&self, x: u32, y: u32, z: u32) -> Result<SpatialInspection, String> {
        let (w, h, d) = self.geometry.dims;
        if x >= w || y >= h || z >= d {
            return Err(format!("({x}, {y}, {z}) is outside the lattice"));
        }
        let agent = self.geometry.at(x, y, z).map(|i| {
            let candidates = std::iter::once(i as u32)
                .chain(self.geometry.neighbors(i).iter().copied())
                .map(|j| {
                    let (x, y, z) = self.geometry.xyz(j as usize);
                    Candidate {
                        x,
                        y,
                        z,
                        strategy: letter(self.coop[j as usize]),
                        score: self.scores[j as usize],
                    }
                })
                .collect();
            let (next, p_c) = match self.config.winning {
                Winning::Deterministic => (Some(letter(self.winner(i, |w, j| w.scores[j]))), None),
                Winning::Probabilistic => (None, self.p_c(i, |w, j| w.scores[j])),
            };
            PlayerView {
                id: i as u64,
                strategy: letter(self.coop[i]),
                previous: letter(self.previous[i]),
                score: self.scores[i],
                candidates,
                next,
                p_c,
            }
        });
        Ok(SpatialInspection {
            site: CellXyz { x, y, z },
            agent,
        })
    }
}

impl Model for SpatialWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Spatial(self.config.clone())
    }

    fn run(&mut self, ticks: u32) {
        SpatialWorld::run(self, ticks);
    }

    fn tick(&self) -> u64 {
        self.tick
    }

    fn population(&self) -> usize {
        self.players()
    }

    /// FNV-1a over the tick and every player's strategy now and a
    /// generation ago.
    fn fingerprint(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut eat = |v: u64| {
            for b in v.to_le_bytes() {
                h ^= u64::from(b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        eat(self.tick);
        for chunk in self.coop.chunks(64).zip(self.previous.chunks(64)) {
            let pack = |bits: &[bool]| {
                bits.iter()
                    .enumerate()
                    .fold(0u64, |acc, (k, &b)| acc | (u64::from(b) << k))
            };
            eat(pack(chunk.0));
            eat(pack(chunk.1));
        }
        h
    }

    /// The frame is the lattice (a cube's `slice:<z>`), one pixel a cell.
    fn size(&self) -> (u32, u32) {
        (self.geometry.dims.0, self.geometry.dims.1)
    }

    fn render(&self, mode: &str, layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        let mode: SpatialMode = mode.parse()?;
        let (w, h, _) = self.geometry.dims;
        let z = self.slice(layer);
        self.view_z.set(z);
        let max_score = self.scores.iter().copied().fold(0.0, f64::max);
        buf.resize((w * h * 4) as usize, 0);
        for y in 0..h {
            for x in 0..w {
                let rgb = match self.geometry.at(x, y, z) {
                    Some(i) => self.color(i, mode, max_score),
                    None => BACKGROUND,
                };
                let o = ((y * w + x) * 4) as usize;
                buf[o..o + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
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
        let mut out = String::from("id,x,y,z,strategy,score\n");
        for i in 0..self.players() {
            let (x, y, z) = self.geometry.xyz(i);
            writeln!(
                out,
                "{i},{x},{y},{z},{},{}",
                letter(self.coop[i]),
                self.scores[i]
            )
            .unwrap();
        }
        out
    }

    /// The cell (x, y) of the slice last drawn (a cube's view).
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        let inspection = self.inspect(x, y, self.view_z.get())?;
        Ok(serde_json::to_string(&inspection).expect("inspection serializes"))
    }

    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        (id < self.players() as u64).then(|| {
            let (x, y, _) = self.geometry.xyz(id as usize);
            (x, y)
        })
    }

    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Spatial(next) = next else {
            return Err(wrong_model(ModelKind::Spatial, &next));
        };
        next.validate()?;
        let changes = self.config.changes(&next);
        if !changes.is_empty() {
            return Err(changes);
        }
        self.config = next;
        self.rescore_all();
        Ok(())
    }
}
```

`presets.rs` (Decision 10; the descriptions carry Decision 12):

```rust
//! The papers' figures as presets.

use super::config::{Boundary, Lattice, Neighborhood, SpatialConfig, Start, Update, Winning};
use crate::model::ModelConfig;
use crate::presets::ModelPreset;

fn preset(
    id: &'static str,
    name: &'static str,
    source: &'static str,
    description: &'static str,
    edit: impl FnOnce(&mut SpatialConfig),
) -> ModelPreset {
    let mut c = SpatialConfig::default();
    edit(&mut c);
    ModelPreset {
        id,
        name,
        source,
        description,
        config: ModelConfig::Spatial(c),
    }
}

const NM92: &str = "Nowak & May 1992, Nature 359";
const HG93: &str = "Huberman & Glance 1993, PNAS 90";
const NBM94: &str = "Nowak, Bonhoeffer & May 1994, PNAS 91";

/// The kaleidoscope: one defector at the centre of a 99 × 99 lattice of
/// cooperators, fixed edges, b = 1.9 (NM92 Fig. 3).
fn kaleidoscope(c: &mut SpatialConfig) {
    c.width = 99;
    c.height = 99;
    c.start = Start::SingleDefector;
    c.b = 1.9;
}

/// NBM94's arena: 80 × 80, periodic, 10% defectors.
fn nbm_arena(c: &mut SpatialConfig) {
    c.width = 80;
    c.height = 80;
    c.boundary = Boundary::Periodic;
    c.defectors = 0.5;
}

pub fn presets() -> Vec<ModelPreset> {
    vec![
        preset(
            "nm-1a-static",
            "Fig. 1a — a static network",
            NM92,
            "NM92's Fig. 1a: a 200 × 200 lattice with fixed edges, 10% defectors at random, each player playing its eight neighbors and itself, b = 1.77 (the paper's 1.75 < b < 1.8). The paper: an irregular, mainly static network of D lines in a sea of C, with f_C \"usually between 0.7 and 0.95\". Measured (seeds 1–20): f_C 0.737–0.748 at t = 200, with 3% of sites still blinking each generation.",
            |c| c.b = 1.77,
        ),
        preset(
            "nm-1b-chaos",
            "Fig. 1b — spatial chaos",
            NM92,
            "NM92's Fig. 1b: as Fig. 1a with b = 1.9 (2 > b > 1.8): spatial chaos, C and D both persisting in shifting patterns, lots of yellow and green in the Change colors. Measured (seeds 1–20): f_C settles at 0.318–0.324 by t = 200–300 on this 200 × 200 lattice.",
            |_| {},
        ),
        preset(
            "nm-2a-universal",
            "Fig. 2a — 0.318 from any start",
            NM92,
            "NM92's Fig. 2a: 400 × 400, fixed edges, f_C(0) = 0.6, b = 1.9. The paper: f_C fluctuates around 12 log 2 − 8 ≈ 0.318 \"for almost all starting proportions and configurations\". Measured (seeds 1–20): 0.3179 ± 0.0005 over t = 201–300 — the constant reproduces to three decimals. The sweep nm-universal varies the start.",
            |c| {
                c.width = 400;
                c.height = 400;
                c.defectors = 0.4;
            },
        ),
        preset(
            "nm-3-kaleidoscope",
            "Fig. 3 — the evolutionary kaleidoscope",
            NM92,
            "NM92's Fig. 3, the evolutionary kaleidoscope: one defector at the center of a 99 × 99 lattice of cooperators, fixed edges, b = 1.9. Reproduced exactly: the pattern keeps its four-fold symmetry every generation and reaches the edges at t = 49, as the paper says; f_C then averages 0.315 over t = 100–221. Compare it with its asynchronous twin (Huberman and Glance) from the presets menu.",
            kaleidoscope,
        ),
        preset("nm-no-self", "No self-interaction", NM92, "NM92's variant without self-interaction (a = 0): the interesting region becomes 5/3 > b > 8/5; here b = 1.62, 200 × 200, 10% defectors. The paper: f_C ~0.299. Measured (seeds 1–20): 0.301–0.305 over t = 501–1000.", |c| {
            c.self_weight = 0.0;
            c.b = 1.62;
        }),
        preset("nm-four-neighbors", "Four neighbors", NM92, "NM92's variant with only the four orthogonal neighbors (and self): the interesting region is 2 > b > 5/3; here b = 1.8, 200 × 200, 10% defectors. The paper: f_C around 0.374. Measured (seeds 1–20): 0.379–0.382 over t = 501–1000, the same at every b in the window — close to, but not within 0.005 of, the paper's value.", |c| {
            c.neighborhood = Neighborhood::VonNeumann;
            c.b = 1.8;
        }),
        preset(
            "hg-async-kaleidoscope",
            "The kaleidoscope, asynchronous",
            HG93,
            "HG93's Fig. 1: NM92's kaleidoscope with asynchronous updating — each microstep one random player is rescored and replaced by its best-scoring neighbor, N microsteps a generation. The paper: \"within a hundred generations or so\" all players defect, and \"as long as there is at least one defector … always\". Measured (seeds 1–20): all D at t = 56–149 (mean 101) — reproduced; but HG93 never say which b they used, and below 1.8 the claim fails: at b = 1.7 the lone defector dies out (f_C 0.974–0.999). The sweep hg-async runs every b.",
            |c| {
                kaleidoscope(c);
                c.update = Update::Asynchronous;
            },
        ),
        preset(
            "nbm-probabilistic",
            "Proportional winning (m = 1)",
            NBM94,
            "NBM94's probabilistic winning (Eq. 1) at m = 1, \"proportional winning\": each site goes to C with probability Σ A (C) / Σ A over itself and its neighbors. 80 × 80, periodic edges, b = 1.35, 50% defectors (NBM94 do not state their start; an even start is what their m = 0 row shows). Measured (seeds 1–20): f_C 0.28–0.33 at t = 200. The sweeps nbm-grid-discrete and nbm-grid-continuous run the paper's whole b × m grid.",
            |c| {
                nbm_arena(c);
                c.winning = Winning::Probabilistic;
                c.m = 1.0;
                c.b = 1.35;
            },
        ),
        preset(
            "nbm-discrete",
            "Discrete time, b = 1.71",
            NBM94,
            "NBM94's arena in discrete time: 80 × 80, periodic, deterministic winning, b = 1.71, 50% defectors. Measured (seeds 1–20): f_C 0.86–0.92 at t = 200, mostly static. Its continuous-time twin is nbm-continuous (both open side by side from the presets menu).",
            |c| {
                nbm_arena(c);
                c.b = 1.71;
            },
        ),
        preset(
            "nbm-continuous",
            "Continuous time, b = 1.71",
            NBM94,
            "NBM94's continuous time (as HG93's asynchronous updating) in the same arena, b = 1.71. NBM94: coexistence survives continuous time across a broad band of b; only the 1.8 < b < 2 chaos disappears. Measured (seeds 1–20): f_C 0.71–0.73 at t = 200 — C and D coexist; at b = 1.9 the same setup is all D in 20 of 20 seeds while discrete time stays chaotic in 16 of 20.",
            |c| {
                nbm_arena(c);
                c.update = Update::Asynchronous;
                c.b = 1.71;
            },
        ),
        preset(
            "nbm-random-array",
            "A random array, r = 5",
            NBM94,
            "NBM94's irregular arrays: 5% of a 200 × 200 grid's cells hold players, each playing everyone within radius r = 5, b = 1.6, 50% defectors. NBM94: coexistence for intermediate b \"provided r was not too big\"; \"for b = 1.6, r_c ~ 9\". Measured (seeds 1–20, 50% defectors): all D in 0, 10 and 20 of 20 seeds at r = 5, 9 and 11 — reproduced; but from NM92's 10% start no radius up to 11 ends all D (f_C above 0.3), so r_c depends on the start the paper does not report. The sweep nbm-radius shows both.",
            |c| {
                c.lattice = Lattice::Random;
                c.occupancy = 0.05;
                c.radius = 5.0;
                c.b = 1.6;
                c.defectors = 0.5;
            },
        ),
        preset("nbm-cube", "A cube, 30 × 30 × 30", NBM94, "A 30 × 30 × 30 cube with periodic edges, each player playing its 26 neighbors and itself; the view shows one z-slice (the Slice menu). NBM94 say only that 3D results \"are similar to the two-dimensional ones\"; b = 1.6 and 10% defectors were chosen by measurement: the cube coexists for b from 1.1 to 1.8 and is all but all D at 1.9. Measured (seeds 1–20): f_C 0.331–0.342 over t = 101–200 with a quarter of the cube changing every generation — a 3D spatial chaos.", |c| {
            c.lattice = Lattice::Cube;
            c.width = 30;
            c.boundary = Boundary::Periodic;
            c.b = 1.6;
        }),
    ]
}
```

`mod.rs`:

```rust
//! Spatial games (milestone 12): the spatial Prisoner's Dilemma of Nowak &
//! May, "Evolutionary games and spatial chaos", Nature 359 (1992), with the
//! asynchronous updating of Huberman & Glance (PNAS 90, 1993) and the
//! probabilistic winning, continuous time, irregular arrays and cubes of
//! Nowak, Bonhoeffer & May (PNAS 91, 1994). See
//! docs/superpowers/specs/2026-09-25-spatial-games-design.md.

mod config;
mod geometry;
mod presets;
mod stats;
mod world;

pub use config::{
    schema, Boundary, Lattice, Neighborhood, SpatialConfig, Start, Update, Winning, LIVE,
};
pub use geometry::{offsets, Geometry, EMPTY};
pub use presets::presets;
pub use stats::{SpatialSnapshot, SERIES};
pub use world::{
    Candidate, CellXyz, PlayerView, SpatialInspection, SpatialMode, SpatialWorld, C_AFTER_C,
    C_AFTER_D, D_AFTER_C, D_AFTER_D,
};
```

- [ ] **Step 4: Run the tests**

Run: `cargo test -p sugarscape-core --lib spatial && cargo test -p sugarscape-core --lib model:: && cargo test -p sugarscape-core --test golden && cargo test -p sugarscape-core --test checkpoint`
Expected: PASS — 25 spatial tests, the model tests, every golden entry (the twelve new ones as Decision 13; if one differs, stop and report), the keyframe tests. Then `cargo test --workspace` — PASS.

- [ ] **Step 5: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/src/spatial crates/sugarscape-core/src/lib.rs crates/sugarscape-core/src/model.rs crates/sugarscape-core/src/presets.rs crates/sugarscape-core/tests/golden.rs crates/sugarscape-core/tests/checkpoint.rs
git commit -m "Play Nowak and May's spatial Prisoner's Dilemma as the spatial model kind" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 3: Measured against the papers — book-style tests, five sweeps, WASM

**Files:**
- Create: `crates/sugarscape-core/tests/spatial.rs`, `sweeps/nm-universal.json`, `sweeps/hg-async.json`, `sweeps/nbm-grid-discrete.json`, `sweeps/nbm-grid-continuous.json`, `sweeps/nbm-radius.json`
- Modify: `crates/sugarscape-core/src/sweep.rs`, `crates/sugarscape-cli/tests/cli.rs`, `crates/sugarscape-wasm/tests/web.rs`

**Interfaces:**
- Consumes: Task 2's presets and `SpatialWorld`.
- Produces: built-in sweep ids (Copy order).

- [ ] **Step 1: The book-style tests**

Create `crates/sugarscape-core/tests/spatial.rs` (the kaleidoscope's exact test runs normally; the rest are `#[ignore]`):

```rust
//! The spatial games (milestone 12) against Nowak & May 1992, Huberman &
//! Glance 1993 and Nowak, Bonhoeffer & May 1994. The kaleidoscope's exact
//! claims run with the other tests; the statistical claims run over seeds
//! 1–20 in release: `cargo test -p sugarscape-core --release --test spatial
//! -- --ignored`. Thresholds come from the measurements recorded 2026-09-25
//! (docs/superpowers/plans/2026-09-25-spatial-games.md, Decision 12); claims
//! that do not hold are pinned as measured.

use std::thread;

use sugarscape_core::model::ModelConfig;
use sugarscape_core::presets;
use sugarscape_core::spatial::{SpatialConfig, SpatialWorld, Update, Winning};

fn config(id: &str) -> SpatialConfig {
    match presets::find(id)
        .unwrap_or_else(|| panic!("no preset {id}"))
        .config
    {
        ModelConfig::Spatial(c) => c,
        _ => panic!("{id} is not a spatial preset"),
    }
}

/// Runs `c` for `ticks` from seeds 1–20 in parallel and measures each run.
fn each_seed<T: Send>(
    c: &SpatialConfig,
    ticks: u32,
    f: impl Fn(&SpatialWorld) -> T + Sync,
) -> Vec<T> {
    thread::scope(|s| {
        let handles: Vec<_> = (1..=20u64)
            .map(|seed| {
                let f = &f;
                s.spawn(move || {
                    let mut w = SpatialWorld::new(c.clone(), seed).unwrap();
                    w.run(ticks);
                    f(&w)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    })
}

fn series(w: &SpatialWorld, name: &str) -> Vec<f64> {
    w.stats.series(name).unwrap()
}

fn last(w: &SpatialWorld, name: &str) -> f64 {
    *series(w, name).last().unwrap()
}

/// The mean of `fraction_c` from tick `from` on.
fn tail(w: &SpatialWorld, from: usize) -> f64 {
    let s = series(w, "fraction_c");
    s[from..].iter().sum::<f64>() / (s.len() - from) as f64
}

/// Whether the square lattice's strategies are symmetric under the
/// square's reflections (and so its rotations).
fn four_fold(w: &SpatialWorld) -> bool {
    let (n, _, _) = w.geometry.dims;
    let at = |x: u32, y: u32| w.is_cooperator(w.geometry.at(x, y, 0).unwrap());
    (0..n).all(|y| {
        (0..n).all(|x| {
            let s = at(x, y);
            s == at(y, x) && s == at(n - 1 - x, y) && s == at(x, n - 1 - y)
        })
    })
}

#[test]
fn the_kaleidoscope_keeps_its_symmetry_and_reaches_the_boundary_at_49() {
    // NM92 Fig. 3: "the pattern has reached the boundary (which happens at
    // t = 49)"; "the initial symmetry is always maintained".
    let mut w = SpatialWorld::new(config("nm-3-kaleidoscope"), 1).unwrap();
    let edge = |w: &SpatialWorld| {
        (0..99).any(|k| {
            [(k, 0), (k, 98), (0, k), (98, k)]
                .iter()
                .any(|&(x, y)| !w.is_cooperator(w.geometry.at(x, y, 0).unwrap()))
        })
    };
    let mut reached = None;
    for t in 1..=221u32 {
        w.step();
        if reached.is_none() && edge(&w) {
            reached = Some(t);
        }
        if [30, 217, 219, 221].contains(&t) {
            assert!(four_fold(&w), "t = {t}");
        }
    }
    assert_eq!(reached, Some(49));
}

#[test]
#[ignore]
fn spatial_chaos_settles_at_12_ln_2_minus_8() {
    // Measured: 400², 40% D: 0.3179 ± 0.0005 over t = 201–300.
    let target = 12.0 * std::f64::consts::LN_2 - 8.0;
    let v = each_seed(&config("nm-2a-universal"), 300, |w| tail(w, 201));
    assert!(v.iter().all(|f| (f - target).abs() < 0.003), "{v:?}");
    // From 5% to 60% defectors (200²) every seed settles at 0.318–0.323;
    // from 80%, 19 of 20 do and one (seed 12) ends all C.
    for (d, exceptions) in [(0.05, 0), (0.3, 0), (0.6, 0), (0.8, 1)] {
        let mut c = config("nm-1b-chaos");
        c.defectors = d;
        let v = each_seed(&c, 400, |w| tail(w, 301));
        let off = v.iter().filter(|f| !(0.312..0.33).contains(*f)).count();
        assert_eq!(off, exceptions, "{d}: {v:?}");
    }
}

#[test]
#[ignore]
fn nm92s_other_regimes_and_neighborhoods() {
    // Fig. 1a: f_C "usually between 0.7 and 0.95" (measured 0.737–0.748).
    let v = each_seed(&config("nm-1a-static"), 200, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|f| (0.7..0.95).contains(f)), "{v:?}");
    // Without self-interaction ≈ 0.299 (measured 0.301–0.305); four
    // neighbors ≈ 0.374 (measured 0.379–0.382).
    let v = each_seed(&config("nm-no-self"), 1000, |w| tail(w, 501));
    assert!(v.iter().all(|f| (0.295..0.31).contains(f)), "{v:?}");
    let v = each_seed(&config("nm-four-neighbors"), 1000, |w| tail(w, 501));
    assert!(v.iter().all(|f| (0.37..0.39).contains(f)), "{v:?}");
}

#[test]
#[ignore]
fn asynchronous_updating_ends_in_defection_only_above_1_8() {
    // HG93: all D "within a hundred generations or so" (measured t = 56–149).
    let v = each_seed(&config("hg-async-kaleidoscope"), 300, |w| {
        series(w, "fraction_c").iter().position(|&f| f == 0.0)
    });
    assert!(
        v.iter().all(|t| t.is_some_and(|t| (30..=250).contains(&t))),
        "{v:?}"
    );
    // But not "as long as there is at least one defector": at b = 1.7 the
    // defector dies out (measured f_C 0.974–0.999).
    let mut c = config("hg-async-kaleidoscope");
    c.b = 1.7;
    let v = each_seed(&c, 300, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f > 0.95), "{v:?}");
}

#[test]
#[ignore]
fn nbm94s_continuous_time_and_self_interaction() {
    // b = 1.9, deterministic, 80² periodic, 50% D: all D in continuous time
    // (20 of 20); spatial chaos in discrete time in 16 of 20 (two seeds end
    // all C, two all but all D).
    let mut c = config("nbm-continuous");
    c.b = 1.9;
    let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f == 0.0), "{v:?}");
    c.update = Update::Synchronous;
    let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
    let chaotic = v.iter().filter(|&&f| f > 0.05 && f < 0.95).count();
    assert!(chaotic >= 12, "{chaotic}: {v:?}");
    // "for m = 1 … C cannot persist in the absence of self-interaction"
    // (measured: f_C ≤ 0.007 at b = 1.13 and 1.35; but 0.16–0.33 at 1.05),
    // while deterministic winning keeps C (0.85–0.95).
    let mut c = config("nbm-probabilistic");
    c.self_weight = 0.0;
    for b in [1.13, 1.35] {
        c.b = b;
        c.winning = Winning::Probabilistic;
        let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
        assert!(v.iter().all(|&f| f < 0.02), "{b}: {v:?}");
        c.winning = Winning::Deterministic;
        let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
        assert!(v.iter().all(|&f| f > 0.8), "{b}: {v:?}");
    }
    c.b = 1.05;
    c.winning = Winning::Probabilistic;
    let v = each_seed(&c, 200, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f > 0.1), "C persists at b = 1.05: {v:?}");
}

#[test]
#[ignore]
fn random_arrays_lose_cooperation_near_radius_9() {
    // NBM94: "for b = 1.6, r_c ~ 9" — from a 50% start (measured all D in
    // 0, 10 and 20 of 20 seeds at r = 5, 9 and 11).
    let all_d = |r: f64| {
        let mut c = config("nbm-random-array");
        c.radius = r;
        each_seed(&c, 300, |w| last(w, "fraction_c") == 0.0)
            .iter()
            .filter(|&&d| d)
            .count()
    };
    assert_eq!(all_d(5.0), 0);
    assert!((4..=16).contains(&all_d(9.0)));
    assert_eq!(all_d(11.0), 20);
    // From NM92's 10% start no radius up to 11 ends all D.
    let mut c = config("nbm-random-array");
    c.defectors = 0.1;
    c.radius = 11.0;
    let v = each_seed(&c, 300, |w| last(w, "fraction_c"));
    assert!(v.iter().all(|&f| f > 0.3), "{v:?}");
}

#[test]
#[ignore]
fn the_cube_coexists_like_the_square() {
    // b = 1.6, 30³ periodic: f_C 0.331–0.342, a quarter of the cube changing
    // each generation.
    let v = each_seed(&config("nbm-cube"), 200, |w| {
        (tail(w, 101), last(w, "changed"))
    });
    assert!(
        v.iter()
            .all(|&(f, ch)| (0.3..0.37).contains(&f) && ch > 0.2),
        "{v:?}"
    );
}
```

Run: `cargo test -p sugarscape-core --test spatial && cargo test -p sugarscape-core --release --test spatial -- --ignored`
Expected: PASS (1 test in about 2 s in a debug build; 6 more in about 20 s in release).

- [ ] **Step 2: The sweeps**

`sweeps/nm-universal.json`:

```json
{
  "name": "Spatial games: 0.318 from any start? (NM92 Fig. 2a)",
  "description": "NM92's claim that spatial chaos settles at f_C ≈ 0.318 (12 log 2 − 8) \"for almost all starting proportions\": b = 1.9 on 200 × 200 with fixed edges, the starting fraction of defectors from 5% to 95%, the mean f_C over t = 301–400. Measured (release, seeds 1–3, recorded 2026-09-25): 0.320–0.321 from 5% to 80% defectors; 0.21 from 90% (some runs end all C or all D) and 0 from 95% (defectors win outright). Over 20 seeds from 80%, one ended all C.",
  "base": { "preset": "nm-1b-chaos" },
  "x": { "label": "Defectors at the start", "path": "defectors", "values": [0.05, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 0.95] },
  "seeds": { "from": 1, "count": 3 },
  "ticks": 400,
  "metric": { "kind": "window_mean", "series": "fraction_c", "from": 301 }
}
```

`sweeps/hg-async.json`:

```json
{
  "name": "Spatial games: does one defector take over when updates are asynchronous? (HG93)",
  "description": "HG93's claim that with asynchronous updating one defector always takes over: NM92's kaleidoscope start (99 × 99, one defector) at NBM94's ten b values, synchronous and asynchronous, the final f_C at t = 300. Measured (release, seeds 1–10, recorded 2026-09-25): asynchronous: 0.99–1.00 C for b ≤ 1.42 and at 1.71–1.77, 0.61 at 1.55, 0 at 1.9 and 0.04 at 2.01; synchronous: 1.00 through 1.77, 0.34 at 1.9, 0.91 at 2.01. The takeover HG93 report happens only above b = 1.8: their synchronous picture matches NM92's Fig. 3, so they ran in that range, but they never state b, and below 1.8 the lone defector dies out.",
  "base": { "preset": "hg-async-kaleidoscope" },
  "x": { "label": "Temptation (b)", "path": "b", "values": [1.05, 1.13, 1.16, 1.35, 1.42, 1.55, 1.71, 1.77, 1.9, 2.01] },
  "series": {
    "label": "Update",
    "values": [
      { "at": 0, "name": "Synchronous (NM92)", "set": { "update": "synchronous" } },
      { "at": 1, "name": "Asynchronous (HG93)", "set": { "update": "asynchronous" } }
    ]
  },
  "seeds": { "from": 1, "count": 10 },
  "ticks": 300,
  "metric": { "kind": "final", "series": "fraction_c" }
}
```

`sweeps/nbm-grid-discrete.json`:

```json
{
  "name": "Spatial games: NBM94's Fig. 1 — cooperators by b and m, discrete time",
  "description": "NBM94's Fig. 1 as numbers: the final fraction of cooperators at t = 200 on an 80 × 80 periodic lattice (eight neighbors and self, 50% defectors), for their ten b values — including 1.77, which the figure shows but its caption omits — and seven values of m, in discrete time. The paper: all C near b = 1, all D approaching 2, coexistence between, for every m; D \"fares somewhat better\" as m falls from ∞, and the band of coexistence \"again widens\" below m = 1. Measured (release, seeds 1–5, recorded 2026-09-25): m = ∞: 0.99 at 1.05, 0.88–0.94 from 1.13 to 1.77, 0.30 at 1.9 (the chaos), 0 at 2.01; m = 10: all D from 1.71; m = 1: 0.30 at 1.35, 0.002 at 1.77 and 0 from 1.9; m = 0.5: C persists to 2.01 (0.11); m = 0: 0.56 at every b (drift). The regimes reproduce; where the figure's m = 1 row still shows scattered C at 1.9 and 2.01, these runs are all D.",
  "base": { "preset": "nbm-probabilistic" },
  "x": { "label": "Temptation (b)", "path": "b", "values": [1.05, 1.13, 1.16, 1.35, 1.42, 1.55, 1.71, 1.77, 1.9, 2.01] },
  "series": {
    "label": "Winning",
    "values": [
      { "at": 0, "name": "m = ∞ (highest score wins)", "set": { "winning": "deterministic" } },
      { "at": 100, "name": "m = 100", "set": { "winning": "probabilistic", "m": 100 } },
      { "at": 20, "name": "m = 20", "set": { "winning": "probabilistic", "m": 20 } },
      { "at": 10, "name": "m = 10", "set": { "winning": "probabilistic", "m": 10 } },
      { "at": 1, "name": "m = 1 (proportional)", "set": { "winning": "probabilistic", "m": 1 } },
      { "at": 0.5, "name": "m = 0.5", "set": { "winning": "probabilistic", "m": 0.5 } },
      { "at": -1, "name": "m = 0 (random drift)", "set": { "winning": "probabilistic", "m": 0 } }
    ]
  },
  "seeds": { "from": 1, "count": 5 },
  "ticks": 200,
  "metric": { "kind": "final", "series": "fraction_c" }
}
```

`sweeps/nbm-grid-continuous.json` (the same grid with `"update": "asynchronous"` in every series value):

```json
{
  "name": "Spatial games: NBM94's Fig. 2 — cooperators by b and m, continuous time",
  "description": "NBM94's Fig. 2: the same grid in continuous time (one random site at a time, N a generation). The paper: qualitatively like discrete time, except that the 1.8 < b < 2 chaos becomes all D and, at m = 1, C does better. Measured (release, seeds 1–5, recorded 2026-09-25): m = ∞: 0.59–0.99 from 1.05 to 1.77, 0 at 1.9 and 2.01; m = 1: 0.86 at 1.35 (discrete: 0.30) and C persisting to 2.01 (0.02); m = 0.5: 0.23 at 2.01. Both claims reproduce.",
  "base": { "preset": "nbm-probabilistic" },
  "x": { "label": "Temptation (b)", "path": "b", "values": [1.05, 1.13, 1.16, 1.35, 1.42, 1.55, 1.71, 1.77, 1.9, 2.01] },
  "series": {
    "label": "Winning",
    "values": [
      { "at": 0, "name": "m = ∞ (highest score wins)", "set": { "update": "asynchronous", "winning": "deterministic" } },
      { "at": 100, "name": "m = 100", "set": { "update": "asynchronous", "winning": "probabilistic", "m": 100 } },
      { "at": 20, "name": "m = 20", "set": { "update": "asynchronous", "winning": "probabilistic", "m": 20 } },
      { "at": 10, "name": "m = 10", "set": { "update": "asynchronous", "winning": "probabilistic", "m": 10 } },
      { "at": 1, "name": "m = 1 (proportional)", "set": { "update": "asynchronous", "winning": "probabilistic", "m": 1 } },
      { "at": 0.5, "name": "m = 0.5", "set": { "update": "asynchronous", "winning": "probabilistic", "m": 0.5 } },
      { "at": -1, "name": "m = 0 (random drift)", "set": { "update": "asynchronous", "winning": "probabilistic", "m": 0 } }
    ]
  },
  "seeds": { "from": 1, "count": 5 },
  "ticks": 200,
  "metric": { "kind": "final", "series": "fraction_c" }
}
```

`sweeps/nbm-radius.json`:

```json
{
  "name": "Spatial games: random arrays by interaction radius (NBM94)",
  "description": "NBM94's random arrays at b = 1.6 (5% of 200 × 200 occupied): the final f_C at t = 300 for interaction radii 2–11, from 10% and from 50% defectors. The paper: all D once r passes r_c ~ 9. Measured (release, seeds 1–10, recorded 2026-09-25): from 50%, 0.27–0.58 up to r = 9 and 0 at 10 and 11 — the threshold reproduces; from 10%, 0.73–0.86 at every radius — no threshold. NBM94 do not state the start, so their r_c is a property of an unreported initial condition as much as of b.",
  "base": { "preset": "nbm-random-array" },
  "x": { "label": "Interaction radius (cells)", "path": "radius", "values": [2, 3, 4, 5, 6, 7, 8, 9, 10, 11] },
  "series": {
    "label": "Defectors at the start",
    "values": [
      { "at": 0.1, "name": "10% defectors", "set": { "defectors": 0.1 } },
      { "at": 0.5, "name": "50% defectors", "set": { "defectors": 0.5 } }
    ]
  },
  "seeds": { "from": 1, "count": 10 },
  "ticks": 300,
  "metric": { "kind": "final", "series": "fraction_c" }
}
```

In `crates/sugarscape-core/src/sweep.rs`, `BUILTINS` becomes `[Builtin; 16]` with five entries after `cv-jail-waits`, in the order `nm-universal`, `hg-async`, `nbm-grid-discrete`, `nbm-grid-continuous`, `nbm-radius` (each `Builtin { id, json: include_str!("../../../sweeps/<id>.json") }`), and the three built-in id lists gain the same five ids in that order: `builtin_sweeps_parse_and_validate` (sweep.rs), the `sweeps` listing test in `crates/sugarscape-cli/tests/cli.rs`, and the `builtin_sweeps` test in `crates/sugarscape-wasm/tests/web.rs`.

Run each and compare with Decision 12:

```bash
for id in nm-universal hg-async nbm-grid-discrete nbm-grid-continuous nbm-radius; do cargo run --release -q -p sugarscape-cli -- sweep --builtin $id --quiet --summary-csv /dev/stdout --out /dev/null; done
```

Expected: the means in the descriptions.

- [ ] **Step 3: WASM portability**

In `crates/sugarscape-wasm/tests/web.rs`, before `anasazi_overlays_and_inspection`:

```rust
#[wasm_bindgen_test]
fn spatial_sims_match_the_native_golden_entries() {
    // crates/sugarscape-core/tests/golden.rs, MODEL_GOLDEN: Eq. 1's powers
    // are the same bits here as natively.
    for (id, fp) in [
        ("nbm-probabilistic", "0x79a048f606d28d74"),
        ("hg-async-kaleidoscope", "0xef13c172a34a980d"),
        ("nbm-random-array", "0xcf2c74041806d530"),
    ] {
        let mut sim = Sim::new(&preset_json(id), 1, JsValue::NULL).unwrap();
        assert_eq!(sim.model_kind(), "spatial");
        sim.step(200);
        assert_eq!(sim.fingerprint(), fp, "{id}");
    }
}
```

Run: `cargo test -p sugarscape-core --lib sweep && cargo test -p sugarscape-cli && wasm-pack test --node crates/sugarscape-wasm` — Expected: PASS.

- [ ] **Step 4: Format, lint and commit**

```bash
cargo fmt --all && cargo clippy --all-targets -- -D warnings
git add crates/sugarscape-core/tests/spatial.rs sweeps/nm-universal.json sweeps/hg-async.json sweeps/nbm-grid-discrete.json sweeps/nbm-grid-continuous.json sweeps/nbm-radius.json crates/sugarscape-core/src/sweep.rs crates/sugarscape-cli/tests/cli.rs crates/sugarscape-wasm/tests/web.rs
git commit -m "Measure the spatial games against all three papers and add five sweeps" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 4: The spatial games on the page — types, views, the Slice menu and charts

**Files:**
- Create: `web/src/spatial.ts`, `web/src/spatial.test.ts`
- Modify: `web/src/types.ts`, `web/src/models.ts`, `web/src/ui/display.ts`, `web/src/ui/series-data.ts`, `web/src/ui/inspect-panel.ts` (compile-only branch, see Step 3)
- Test: `web/src/models.test.ts`, `web/src/determinism.test.ts`, `web/src/ui/series-data.test.ts`

**Interfaces:**
- Consumes: the WASM package; `ScheduledChange`, `Layer` (types.ts).
- Produces: `SpatialConfig`, `SpatialStats`, `SpatialCandidate`, `PlayerView`, `SpatialInspection` (types.ts); `ModelKind` gains `'spatial'`; `ColorMode` gains `'change' | 'strategy' | 'payoff'`; `Layer` gains `` `slice:${number}` ``; `isSpatialView` (models.ts); `sliceOptions(c: SpatialConfig): [Layer, string][]`, `middleSlice(c: SpatialConfig): Layer` (spatial.ts).

- [ ] **Step 1: Write the failing tests**

`web/src/models.test.ts` (import `isSpatialView`):

```ts
describe('the spatial model', () => {
  it('is read by its tag, and its inspections by their z', () => {
    const c = { model: 'spatial', lattice: 'square' } as unknown as ModelConfig;
    expect(modelOf(c)).toBe('spatial');
    const cell = { site: { x: 1, y: 2, z: 0 }, agent: null } as unknown as AnyInspection;
    const schelling = { site: { x: 1, y: 2 }, agent: null } as AnyInspection;
    expect([cell, schelling].map(isSpatialView)).toEqual([true, false]);
  });

  it('offers the papers’ four-color change view first', () => {
    expect(COLOR_MODES.spatial).toEqual([
      ['change', 'Change'],
      ['strategy', 'Strategy'],
      ['payoff', 'Payoff'],
    ]);
    expect(MODEL_OVERLAYS.spatial).toEqual([]);
  });
});
```

Create `web/src/spatial.test.ts`:

```ts
import { describe, expect, it } from 'vitest';
import { middleSlice, sliceOptions } from './spatial';
import type { SpatialConfig } from './types';

const spatial = (c: Partial<SpatialConfig>): SpatialConfig => ({ lattice: 'square', width: 30, ...c }) as SpatialConfig;

describe('slices', () => {
  it('offers a cube’s z-slices and none otherwise', () => {
    expect(sliceOptions(spatial({ lattice: 'cube', width: 3 }))).toEqual([
      ['slice:0', 'z = 0'],
      ['slice:1', 'z = 1'],
      ['slice:2', 'z = 2'],
    ]);
    expect(sliceOptions(spatial({}))).toEqual([]);
    expect(sliceOptions(spatial({ lattice: 'random' }))).toEqual([]);
  });

  it('starts at the middle slice, as the core does', () => {
    expect(middleSlice(spatial({ lattice: 'cube', width: 30 }))).toBe('slice:15');
    expect(middleSlice(spatial({ lattice: 'cube', width: 5 }))).toBe('slice:2');
  });
});
```

`web/src/ui/series-data.test.ts`:

```ts
  it('charts the spatial games’ cooperators, changes, switches and payoffs', () => {
    expect(MODEL_CHARTS.spatial.map((c) => c.title)).toEqual(['Cooperators', 'Changes', 'Switches', 'Payoffs']);
  });
```

`web/src/determinism.test.ts`: append the twelve spatial presets to `GOLDEN_MODELS` with Decision 13's fingerprints (as `'0x…'` strings).

Run: `(cd web && npm run build && npm test)` — Expected: FAIL (types, `./spatial`).

- [ ] **Step 2: Types and models**

`web/src/types.ts`: `ModelKind` adds `| 'spatial'`; after `CivilConfig`:

```ts
/** Nowak & May's spatial Prisoner's Dilemma and its variants (milestone 12). */
export interface SpatialConfig {
  model: 'spatial';
  lattice: 'square' | 'cube' | 'random';
  width: number;
  height: number;
  neighborhood: 'moore' | 'von_neumann';
  boundary: 'fixed' | 'periodic';
  occupancy: number;
  radius: number;
  b: number;
  epsilon: number;
  self_weight: number;
  update: 'synchronous' | 'asynchronous';
  winning: 'deterministic' | 'probabilistic';
  m: number;
  start: 'random' | 'single_defector';
  defectors: number;
  schedule: ScheduledChange[];
}
```

`ModelConfig` adds `| SpatialConfig`; after `CivilStats`:

```ts
export interface SpatialStats {
  tick: number;
  fraction_c: number;
  changed: number;
  c_to_d: number;
  d_to_c: number;
  /** Null with no player of that strategy. */
  mean_payoff_c: number | null;
  mean_payoff_d: number | null;
  players: number;
}
```

`ModelStats` adds `| SpatialStats`; after `CivilInspection`:

```ts
export interface SpatialCandidate { x: number; y: number; z: number; strategy: 'C' | 'D'; score: number }
/** A spatial player: its strategy now and a generation ago, its score, and who could take its site (itself first). */
export interface PlayerView {
  id: number;
  strategy: 'C' | 'D';
  previous: 'C' | 'D';
  score: number;
  candidates: SpatialCandidate[];
  /** The next strategy under deterministic winning; null when winning is probabilistic. */
  next: 'C' | 'D' | null;
  /** Eq. 1's P(C) under probabilistic winning (null when deterministic, or when every score is 0). */
  p_c: number | null;
}
/** A spatial cell (in a cube, of the slice on screen); no player on an empty cell of a random array. */
export interface SpatialInspection { site: { x: number; y: number; z: number }; agent: PlayerView | null }
```

`AnyInspection` adds `| SpatialInspection`; `ColorMode` adds `| 'change' | 'strategy' | 'payoff'`; `Layer` becomes `` `resource:${number}` | `capacity:${number}` | `pollution:${number}` | `slice:${number}` ``.

`web/src/models.ts` (import `SpatialInspection`): `MODELS` ends `'civil', 'spatial'`; `MODEL_LABELS.spatial = 'Spatial Games'`; `modelOf` accepts `'spatial'`; after `isCivilView`:

```ts
/** A spatial games cell's inspection (it names its z). */
export function isSpatialView(v: AnyInspection): v is SpatialInspection {
  return 'z' in v.site;
}
```

`COLOR_MODES.spatial`:

```ts
  // NM92's four colors (blue C→C, red D→D, yellow C→D, green D→C) first.
  spatial: [
    ['change', 'Change'],
    ['strategy', 'Strategy'],
    ['payoff', 'Payoff'],
  ],
```

and `MODEL_OVERLAYS.spatial: []`.

Create `web/src/spatial.ts`:

```ts
// The spatial games' pure page helpers (milestone 12): a cube's slices; Inspect's rows (Task 5).
import type { Layer, SpatialConfig } from './types';

/** A cube's z-slices for the Slice menu; none for a square or random lattice. */
export function sliceOptions(c: SpatialConfig): [Layer, string][] {
  if (c.lattice !== 'cube') return [];
  return Array.from({ length: c.width }, (_, z): [Layer, string] => [`slice:${z}`, `z = ${z}`]);
}

/** The slice the core draws when none is chosen: the middle, `width / 2` rounded down. */
export function middleSlice(c: SpatialConfig): Layer {
  return `slice:${Math.floor(c.width / 2)}`;
}
```

- [ ] **Step 3: The Slice menu, charts and a compile stub**

`web/src/ui/display.ts` (imports: `middleSlice`, `sliceOptions` from `../spatial`; `SpatialConfig` type): name the layer label's text so it can change, and fill the menu per model on every refill:

```ts
  const layerName = h('span', {}, 'Landscape ');
  const layerLabel = h('label', {}, layerName, layer);
```

and in `refill`, replace the `layerLabel.hidden = model !== 'sugarscape';` line (inside the model-changed block) and the `layer.replaceChildren(...)` line with, after the model-changed block:

```ts
    // A cube's view is one z-slice (the Slice menu); a sugarscape's landscape layers otherwise.
    const slices = model === 'spatial' ? sliceOptions(engine.config as SpatialConfig) : [];
    layerName.textContent = slices.length > 0 ? 'Slice ' : 'Landscape ';
    layerLabel.hidden = model !== 'sugarscape' && slices.length === 0;
    const options = model === 'sugarscape' ? layerOptions(engine.sugar) : slices;
    layer.replaceChildren(...options.map(([v, l]) => h('option', { value: v }, l)));
    if (slices.length > 0 && !slices.some(([v]) => v === engine.layer)) engine.setDisplay({ layer: middleSlice(engine.config as SpatialConfig) });
```

(The doc comment of `buildDisplay` adds "the spatial games their three modes and, in a cube, the Slice menu".)

`web/src/ui/series-data.ts`, `MODEL_CHARTS.spatial` (doc comment adds the spatial games' charts):

```ts
  spatial: [
    { title: 'Cooperators', lines: [{ key: 'fraction_c', label: 'Share C', color: '--blue' }], range: [0, 1] },
    { title: 'Changes', lines: [{ key: 'changed', label: 'Share that switched', color: '--c4' }], range: [0, 1] },
    {
      title: 'Switches',
      lines: [
        { key: 'c_to_d', label: 'C → D', color: '--both' },
        { key: 'd_to_c', label: 'D → C', color: '--lender' },
      ],
    },
    {
      title: 'Payoffs',
      lines: [
        { key: 'mean_payoff_c', label: 'Mean C', color: '--blue' },
        { key: 'mean_payoff_d', label: 'Mean D', color: '--red' },
      ],
    },
  ],
```

Widening `AnyInspection` breaks `web/src/ui/inspect-panel.ts`'s fallthrough to `schellingRows` (it is typed `SchellingInspection`); add a minimal branch before it so this task builds — `isSpatialView(view) ? this.spatialRows(view, gone)` with

```ts
  /** A spatial cell (Task 5 fills in the player's rows). */
  private spatialRows(view: SpatialInspection, _gone: boolean): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    return [row('Cell', `(${view.site.x}, ${view.site.y}, ${view.site.z})`)];
  }
```

(Task 5 replaces it; import `isSpatialView` and `SpatialInspection`; if `_gone` trips `noUnusedParameters`, drop the parameter and the call's argument.)

- [ ] **Step 4: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS, including the twelve spatial fingerprints through the engine and the model-charts series check.

```bash
git add web/src/types.ts web/src/models.ts web/src/spatial.ts web/src/spatial.test.ts web/src/ui/display.ts web/src/ui/series-data.ts web/src/ui/inspect-panel.ts web/src/models.test.ts web/src/determinism.test.ts web/src/ui/series-data.test.ts
git commit -m "Show the spatial games on the page: their colors, charts and a cube's slices" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): every spatial preset from the **Spatial Games** group renders in Change (blue/red/yellow/green), Strategy and Payoff; `nbm-cube` shows the Slice menu starting at z = 15 and moving it changes only the view; `nbm-random-array` draws empty cells dark; the charts show.

---

### Task 5: Inspect, Compare and Experiments

**Files:**
- Modify: `web/src/spatial.ts`, `web/src/spatial.test.ts`, `web/src/ui/inspect-panel.ts`, `web/src/compare-presets.ts`, `web/src/experiments/form.ts`
- Test: `web/src/compare-presets.test.ts`, `web/src/experiments/form.test.ts`

**Interfaces:**
- Consumes: Task 4's `PlayerView`, `SpatialInspection`, `isSpatialView`.
- Produces: `playerRows(a: PlayerView): [string, string][]` (spatial.ts); the two Compare entries; `defaultForm('spatial')`.

- [ ] **Step 1: Write the failing tests**

Append to `web/src/spatial.test.ts` (import `playerRows`; type `PlayerView`):

```ts
const player = (c: Partial<PlayerView>): PlayerView => ({
  id: 7,
  strategy: 'C',
  previous: 'D',
  score: 6,
  candidates: [
    { x: 1, y: 1, z: 0, strategy: 'C', score: 6 },
    { x: 0, y: 1, z: 0, strategy: 'D', score: 7.6 },
    { x: 2, y: 1, z: 0, strategy: 'C', score: 5 },
  ],
  next: 'D',
  p_c: null,
  ...c,
});

describe('playerRows', () => {
  it('shows the strategy, the score, the neighborhood’s best of each kind and what comes next', () => {
    expect(playerRows(player({}))).toEqual([
      ['Player', '#7 · cooperates (defected last generation)'],
      ['Score', '6'],
      ['Neighborhood', '2 C (best 6), 1 D (best 7.60), itself included'],
      ['Next generation', 'defects'],
    ]);
  });

  it('gives P(C) under probabilistic winning, and says so when no score counts', () => {
    expect(playerRows(player({ next: null, p_c: 0.25 }))[3]).toEqual(['Next generation', 'cooperates with probability 25%']);
    expect(playerRows(player({ next: null, p_c: null }))[3]).toEqual(['Next generation', 'keeps its strategy (every score is 0)']);
  });
});
```

`web/src/compare-presets.test.ts`:

```ts
  it('pairs the spatial games the debate compared', () => {
    const ids = COMPARE_PRESETS.map((c) => [c.id, c.a, c.b, c.label]);
    expect(ids).toContainEqual(['nm-sync-vs-async', 'nm-3-kaleidoscope', 'hg-async-kaleidoscope', 'Synchronous vs asynchronous — Spatial Games (Compare)']);
    expect(ids).toContainEqual(['nbm-discrete-vs-continuous', 'nbm-discrete', 'nbm-continuous', 'Discrete vs continuous time — Spatial Games (Compare)']);
  });
```

`web/src/experiments/form.test.ts`, beside civil's:

```ts
    expect(defaultForm('spatial')).toMatchObject({
      x: { path: 'b', values: '1.05:2.05:0.05' },
      ticks: 200,
      metric: { kind: 'final', series: 'fraction_c' },
    });
```

Run: `(cd web && npm run build && npm test)` — Expected: FAIL.

- [ ] **Step 2: Implement**

Append to `web/src/spatial.ts` (import `PlayerView`):

```ts
const num = (n: number) => (Number.isInteger(n) ? String(n) : n.toFixed(2));
const verb = (s: 'C' | 'D', past = false) => (s === 'C' ? (past ? 'cooperated' : 'cooperates') : past ? 'defected' : 'defects');

/** A player's Inspect rows: its strategy (and last generation's), score, its candidates by kind, and its next strategy. */
export function playerRows(a: PlayerView): [string, string][] {
  const kind = (s: 'C' | 'D') => a.candidates.filter((c) => c.strategy === s);
  const best = (s: 'C' | 'D') => Math.max(...kind(s).map((c) => c.score));
  const part = (s: 'C' | 'D') => (kind(s).length === 0 ? `no ${s}` : `${kind(s).length} ${s} (best ${num(best(s))})`);
  const next =
    a.next !== null
      ? verb(a.next)
      : a.p_c !== null
        ? `cooperates with probability ${Math.round(a.p_c * 100)}%`
        : 'keeps its strategy (every score is 0)';
  return [
    ['Player', `#${a.id} · ${verb(a.strategy)} (${verb(a.previous, true)} last generation)`],
    ['Score', num(a.score)],
    ['Neighborhood', `${part('C')}, ${part('D')}, itself included`],
    ['Next generation', next],
  ];
}
```

In `web/src/ui/inspect-panel.ts`, replace Task 4's stub with (import `playerRows` from `../spatial`):

```ts
  /** A spatial cell and its player: strategy, score, its neighborhood's best of each kind and what it becomes. */
  private spatialRows(view: SpatialInspection): HTMLElement[] {
    const row = (k: string, v: string) => h('tr', {}, h('th', {}, k), h('td', {}, v));
    const { x, y, z } = view.site;
    const cube = (this.engine.config as SpatialConfig).lattice === 'cube';
    const rows = [row('Cell', cube ? `(${x}, ${y}, z = ${z})` : `(${x}, ${y})`)];
    if (!view.agent) return [...rows, row('Player', 'none (an empty cell)')];
    return [...rows, ...playerRows(view.agent).map(([k, v]) => row(k, v))];
  }
```

(its call site becomes `this.spatialRows(view)` — spatial players never leave, so there is no "gone" case; import `SpatialConfig` from `../types`). In `web/src/compare-presets.ts` append the two entries of the Copy list; in `web/src/experiments/form.ts`'s `defaultForm`, before `return form;`:

```ts
  if (model === 'spatial') {
    // NBM94's axis: cooperators against the temptation b.
    return { ...form, x: { path: 'b', values: '1.05:2.05:0.05' }, ticks: 200, metric: { ...form.metric, kind: 'final', series: 'fraction_c' } };
  }
```

- [ ] **Step 3: Run the tests and commit**

Run: `(cd web && npm run build && npm test)` — Expected: PASS.

```bash
git add web/src/spatial.ts web/src/spatial.test.ts web/src/ui/inspect-panel.ts web/src/compare-presets.ts web/src/compare-presets.test.ts web/src/experiments/form.ts web/src/experiments/form.test.ts
git commit -m "Inspect spatial players, pair the debate's runs in Compare, and sweep b by default" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

Browser (controller): Inspect a kaleidoscope cell (Next generation matches the next frame); an empty cell of `nbm-random-array` reads "none (an empty cell)"; in `nbm-cube`, Inspect after changing the slice shows that slice's z; both Compare entries open side by side (the asynchronous kaleidoscope turns all red by t ≈ 100 while the synchronous one keeps its symmetry); Experiments runs `nbm-grid-discrete` and `hg-async`.

---

### Task 6: The survey's spatial claims

**Files:**
- Create: `survey/src/claims/spatial.rs`
- Modify: `survey/src/claims/mod.rs`

**Interfaces:**
- Consumes: `crate::runner::{model_after, model_preset}` (from milestone 11), `crate::claim::{range, Claim, Source}`.

- [ ] **Step 1: The claims**

Create `survey/src/claims/spatial.rs` (Decision 12):

```rust
//! The spatial games (milestone 12): Nowak & May 1992, Huberman & Glance
//! 1993 and Nowak, Bonhoeffer & May 1994, each claim in its paper's words.

use sugarscape_core::model::{ModelConfig, ModelWorld};

use crate::claim::{range, Claim, Source};
use crate::runner::{model_after, model_preset};

const NM92: &str = "Nowak & May 1992, Nature 359";
const HG93: &str = "Huberman & Glance 1993, PNAS 90";
const NBM94: &str = "Nowak, Bonhoeffer & May 1994, PNAS 91";

fn series(w: &ModelWorld, name: &str) -> Vec<f64> {
    w.model()
        .series(name)
        .unwrap_or_else(|| panic!("no series {name}"))
}

fn last(w: &ModelWorld, name: &str) -> f64 {
    *series(w, name).last().expect("a recorded tick")
}

/// The mean of `fraction_c` from tick `from` on.
fn tail(w: &ModelWorld, from: usize) -> f64 {
    let s = series(w, "fraction_c");
    s[from..].iter().sum::<f64>() / (s.len() - from) as f64
}

/// Preset `id` with `edit` applied.
fn with(id: &str, edit: impl FnOnce(&mut sugarscape_core::spatial::SpatialConfig)) -> ModelConfig {
    let mut c = model_preset(id);
    if let ModelConfig::Spatial(s) = &mut c {
        edit(s);
    }
    c
}

pub fn claims() -> Vec<Claim> {
    use sugarscape_core::spatial::{Update, Winning};
    vec![
        Claim {
            id: "nm-2a.universal",
            item: "nm-2a-universal",
            source: Source::Book,
            citation: NM92,
            text: "in 2 > b > 1.8 f_C fluctuates around 0.318 (Fig. 2a: 12 log 2 − 8): the mean over t = 201–300 within 0.003 of 12 ln 2 − 8",
            check: |s| {
                let v = model_after(&model_preset("nm-2a-universal"), s, 300, |w| tail(w, 201));
                let t = 12.0 * std::f64::consts::LN_2 - 8.0;
                range(&v, t - 0.003, t + 0.003, false)
            },
        },
        Claim {
            id: "nm-1b.any-start",
            item: "nm-1b-chaos",
            source: Source::Book,
            citation: NM92,
            text: "f_C ≈ 0.318 'for almost all starting proportions': from 80% defectors the mean over t = 301–400 is 0.312–0.33",
            check: |s| {
                let c = with("nm-1b-chaos", |c| c.defectors = 0.8);
                range(&model_after(&c, s, 400, |w| tail(w, 301)), 0.312, 0.33, false)
            },
        },
        Claim {
            id: "nm-1a.static",
            item: "nm-1a-static",
            source: Source::Book,
            citation: NM92,
            text: "for 1.75 < b < 1.8 the equilibrium frequency of C is usually between 0.7 and 0.95",
            check: |s| {
                let v = model_after(&model_preset("nm-1a-static"), s, 200, |w| last(w, "fraction_c"));
                range(&v, 0.7, 0.95, false)
            },
        },
        Claim {
            id: "nm-no-self.fraction",
            item: "nm-no-self",
            source: Source::Book,
            citation: NM92,
            text: "without self-interaction the asymptotic C fraction is now ~0.299 (within 0.005)",
            check: |s| {
                let v = model_after(&model_preset("nm-no-self"), s, 1000, |w| tail(w, 501));
                range(&v, 0.294, 0.304, false)
            },
        },
        Claim {
            id: "nm-four-neighbors.fraction",
            item: "nm-four-neighbors",
            source: Source::Book,
            citation: NM92,
            text: "with only the four orthogonal neighbours f_C is around 0.374 (within 0.005)",
            check: |s| {
                let v = model_after(&model_preset("nm-four-neighbors"), s, 1000, |w| tail(w, 501));
                range(&v, 0.369, 0.379, false)
            },
        },
        Claim {
            id: "hg-async.defection",
            item: "hg-async-kaleidoscope",
            source: Source::Book,
            citation: HG93,
            text: "within a hundred generations or so the array evolves into a fixed state in which all players defect: all D by t = 200",
            check: |s| {
                let v = model_after(&model_preset("hg-async-kaleidoscope"), s, 200, |w| {
                    last(w, "fraction_c")
                });
                range(&v, 0.0, 0.0, false)
            },
        },
        Claim {
            id: "hg-async.always",
            item: "hg-async-kaleidoscope",
            source: Source::Book,
            citation: HG93,
            text: "as long as there is at least one defector in the initial state, the matrix always evolved rapidly into overall defection: all D at b = 1.7 by t = 300",
            check: |s| {
                let c = with("hg-async-kaleidoscope", |c| c.b = 1.7);
                range(&model_after(&c, s, 300, |w| last(w, "fraction_c")), 0.0, 0.0, false)
            },
        },
        Claim {
            id: "nbm-continuous.chaos-gone",
            item: "nbm-continuous",
            source: Source::Book,
            citation: NBM94,
            text: "for deterministic winning and b between 1.8 and 2 … all D for continuous time: all D at b = 1.9, t = 200",
            check: |s| {
                let c = with("nbm-continuous", |c| c.b = 1.9);
                range(&model_after(&c, s, 200, |w| last(w, "fraction_c")), 0.0, 0.0, false)
            },
        },
        Claim {
            id: "nbm-discrete.polymorphism",
            item: "nbm-discrete",
            source: Source::Book,
            citation: NBM94,
            text: "… we find polymorphism for discrete time: at b = 1.9 both C and D above 5% at t = 200",
            check: |s| {
                let c = with("nbm-discrete", |c| c.b = 1.9);
                range(&model_after(&c, s, 200, |w| last(w, "fraction_c")), 0.05, 0.95, false)
            },
        },
        Claim {
            id: "nbm-probabilistic.no-self",
            item: "nbm-probabilistic",
            source: Source::Book,
            citation: NBM94,
            text: "for m = 1 (proportional winning), C cannot persist in the absence of self-interaction: under 1% C at t = 200 for b = 1.05",
            check: |s| {
                let c = with("nbm-probabilistic", |c| {
                    c.self_weight = 0.0;
                    c.b = 1.05;
                    c.winning = Winning::Probabilistic;
                    c.m = 1.0;
                    c.update = Update::Synchronous;
                });
                range(&model_after(&c, s, 200, |w| last(w, "fraction_c")), 0.0, 0.01, false)
            },
        },
        Claim {
            id: "nbm-random-array.radius",
            item: "nbm-random-array",
            source: Source::Book,
            citation: NBM94,
            text: "if players interact with too many neighbors, the system became all D; for b = 1.6, r_c ~ 9: all D at r = 11 (from 50% defectors)",
            check: |s| {
                let c = with("nbm-random-array", |c| c.radius = 11.0);
                range(&model_after(&c, s, 300, |w| last(w, "fraction_c")), 0.0, 0.0, false)
            },
        },
        Claim {
            id: "nbm-random-array.ten-percent",
            item: "nbm-random-array",
            source: Source::App,
            citation: "spec 2026-09-25-spatial-games-design.md; plan Decision 12",
            text: "the radius threshold depends on the start: from NM92's 10% defectors, r = 11 keeps more than 30% C",
            check: |s| {
                let c = with("nbm-random-array", |c| {
                    c.radius = 11.0;
                    c.defectors = 0.1;
                });
                range(&model_after(&c, s, 300, |w| last(w, "fraction_c")), 0.3, 1.0, false)
            },
        },
        Claim {
            id: "nbm-cube.coexistence",
            item: "nbm-cube",
            source: Source::Book,
            citation: NBM94,
            text: "three-dimensional arrays … the results are similar to the two-dimensional ones: at b = 1.6 both C and D persist (f_C 0.1–0.9 over t = 101–200)",
            check: |s| {
                let v = model_after(&model_preset("nbm-cube"), s, 200, |w| tail(w, 101));
                range(&v, 0.1, 0.9, false)
            },
        },
    ]
}
```

In `survey/src/claims/mod.rs` add `mod spatial;` after `mod civil;` and `spatial::claims(),` after `civil::claims(),`.

- [ ] **Step 2: Run it**

Run: `(cd survey && cargo fmt && cargo test && cargo run --release -- --only nm- && cargo run --release -- --only hg- && cargo run --release -- --only nbm-)`
Expected: tests PASS; spatial verdicts 10 Holds and 3 Fails (`nm-four-neighbors.fraction`, `nbm-probabilistic.no-self`, `hg-async.always`), as Decision 12. (`--only nm-` also lists nothing else; `n-…` claims belong to the N-goods presets.)

- [ ] **Step 3: Commit**

```bash
git add survey/src/claims/spatial.rs survey/src/claims/mod.rs
git commit -m "Survey the spatial games' claims: ten hold, three do not" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

---

### Task 7: README, roadmap, spec notes and full verification

**Files:**
- Modify: `README.md`, `docs/roadmap.md`, `docs/superpowers/specs/2026-09-25-spatial-games-design.md`

- [ ] **Step 1: The spec's notes**

In the spec: the NBM94 start is 50% defectors (the evidence of the m = 0 row, Decision 10); `nbm-discrete` is added for the second Compare entry; the charts are Cooperators, Changes, Switches and Payoffs; scores use the closed form of Decision 3.

- [ ] **Step 2: README**

A section **Spatial games (Nowak & May 1992, and its critics)** after civil violence's, in the same voice: the rules (NM92's game; HG93's asynchronous updating; NBM94's Eq. 1, continuous time, random arrays and cubes); the choices the papers leave open; what reproduces — the kaleidoscope exactly (symmetry, the edges at t = 49), 12 ln 2 − 8 to three decimals from starts of 5–80% defectors, the cluster thresholds, NBM94's regimes, r_c ≈ 9 — and what does not or only partly: four neighbors (0.380 vs ~0.374), HG93's "always" (only above b = 1.8, a b they never state), NBM94's "C cannot persist" at m = 1 without self-interaction (not at b = 1.05), r_c's dependence on an unreported start, the caption missing 1.77, the m = 1 thumbnails' scattered C; the five sweeps; the two Compare entries; the Slice menu. Credit all three papers. Add **Spatial Games** to the models the presets menu groups.

- [ ] **Step 3: Roadmap**

After Milestone 11:

```markdown
## Milestone 12: Spatial games (done)

Nowak and May's spatial Prisoner's Dilemma (1992) as a sixth model kind, with Huberman and Glance's
asynchronous updating (1993) and Nowak, Bonhoeffer and May's probabilistic winning, continuous time, random
arrays and cubes (1994). The kaleidoscope reproduces exactly and the chaotic regime settles at 12 ln 2 − 8 to
three decimals; Huberman and Glance's "always all D" holds only above b = 1.8, a value they never state; and
the random arrays' r_c ≈ 9 depends on an unreported starting mix. See
`docs/superpowers/specs/2026-09-25-spatial-games-design.md`.
```

and change **Nowak–May spatial games** under Experiments and science to "done (Milestone 12)".

- [ ] **Step 4: Full verification**

```bash
cargo fmt --all --check && cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo test -p sugarscape-core --release --test spatial -- --ignored
wasm-pack test --node crates/sugarscape-wasm
(cd web && npm run build && npm test)
(cd survey && cargo test)
```

Expected: all PASS. Browser (controller): every spatial preset, the cube's slices, both Compare entries, the five sweeps, keyframes (step back and the timeline on an asynchronous probabilistic run restore exactly), share links and sessions of a spatial run with a live b change, recording, Max speed on `nm-2a-universal` (400²), and every existing scenario.

- [ ] **Step 5: Commit**

```bash
git add README.md docs/roadmap.md docs/superpowers/specs/2026-09-25-spatial-games-design.md
git commit -m "Document the spatial games and mark milestone 12 done" -m "Claude-Session: https://claude.ai/code/session_0162kiYafiiHTpZRBMetxUPn"
```

## Self-review (planning)

- **Spec coverage:** geometry, game, update, winning and start (Task 2); the stated choices (Task 2 code, Task 7 notes); statistics, views, Inspect, charts (Tasks 2, 4, 5); presets and Compare entries (Tasks 2, 5); sweeps, CLI and WASM (Task 3); claims (Tasks 3, 6); page (Tasks 4–5); docs (Task 7). Keyframes (Task 2) and the portable math (Task 1) support the constraints.
- **Placeholders:** none; the Rust blocks are the dry run's code.
- **Type consistency:** Rust `PlayerView`/`Candidate`/`SpatialInspection` serialize to the TS types of Task 4 (`strategy` "C"/"D", `next` and `p_c` nullable, `site.z`).
- **Review Focus:** each item names its test and task.
