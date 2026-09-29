# Minds 5: caching for the future — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- a carrying limit, caches that Flumps bury and dig, and a winter everywhere at once;
- three caching rules (`even`, `compensate`, `plan`), each the mechanism one hypothesis names;
- Raby's and Amodio's lab protocols as a scripted harness, with each rule's prediction as an oracle;
- a winter field where the rules compete to survive;
- central-place foraging under the marginal-value rule.

**Architecture:**
- `minds/caching/` holds the mechanics (`mod.rs`), episodic memory and the cycle finder (`episodes.rs`), the rules (`rules.rs`) and the lab harness (`lab.rs`).
- The carrying limit and digging live in `movement::go_and_gather`, the one place every rule harvests. Burying runs after the move in each Flump's step, under any decision rule.
- `seasons.mode: global` is a branch in `growback::rate_at`.
- `minds/central.rs` extends Minds 4's marginal-value rule to round trips from a home.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), `survey/`, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-09-29-minds-5-caching-design.md`

## Global Constraints

- Every existing golden entry, legacy fixture and pinned fingerprint stays green and **unedited**. `caching.rule` defaults to `none`, `caching.capacity` to 0 (no limit), `seasons.mode` to `hemispheres`, `central.enabled` to false; with those defaults nothing new runs and nothing new is drawn.
- **Reduction:** `caching.rule: none` with capacity 0 reproduces Minds 4 bit for bit; `seasons.mode: global` with `winter_divisor: 1` equals `seasons.enabled: false`.
- The caching rules and the cycle finder draw nothing. Only the lab population's per-Flump parameters are drawn, through the world's existing RNG.
- Caching needs exactly one good ("caching needs exactly one good"). `central.enabled` needs `decision.rule` `mvt` or `goap` and `caching.capacity > 0`.
- Sugar is conserved exactly across bury and dig: Σ sites + Σ holdings + Σ caches + eaten.
- Caches are `BTreeMap<u32, f64>` keyed by site index (deterministic order), are not hashed into the fingerprint when empty, die with their Flump, and aren't inherited.
- The reserve is R = good-0 metabolism × `goap.horizon`; surplus = max(0, holdings − R). In the lab the reserve is 0.
- Deterministic and portable. No new `ln`/`exp`/`pow`. Ties in the stated order (site index ascending, compartments K1 < K2 < K3).
- Survey judges and thresholds are committed before any survey run and never tuned.
- **The Minds menu entry (main, 359ed2e).** Minds worlds have their own model-menu entry, apart from the book's Sugarscape. Every Minds 5 preset's `source` names "Minds 5" (e.g. "Raby et al. 2007; Minds 5"), so `presetMenu` puts it under Minds. `web/src/models.ts`: `MINDS_TITLES['5'] = 'Minds 5: caching'`, and `usesMinds` also returns true for `caching.rule !== 'none'`, `caching.capacity > 0`, `central.enabled`, or `seasons.mode === 'global'`. Every new Rules group is `minds: true`; `seasons.mode` goes in the Minds caching group, not the book's Seasons group.
- American spelling. Titles follow `titles.rs`. Every commit ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- Work in `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-5` (branch `minds-5`).

## Review Focus

1. **A Flump at the carrying limit on a rich site** takes only its room and leaves the rest on the site; nothing is lost or created. Owned by Task 3.
2. **A Flump whose cache site is occupied, walled off or far away** when it's hungry must not stall or loop; it falls back to its rule's ordinary choice. Owned by Task 3.
3. **A Flump dying with caches** removes them, and conservation still balances (the sugar leaves the world with it, counted). Owned by Task 3.
4. **The cycle finder on short, constant or aperiodic sequences** returns the stated answer (period 1 for constant; none when no period fits within the span), never panics. Owned by Task 4.
5. **Configs without `caching`, `central` or `seasons.mode`** load as before; live knobs change live; reset-only knobs change only on reset. Owned by Task 1.

---

### Task 1: Config

**Files:** `crates/sugarscape-core/src/config.rs`, `legacy.rs`.

**Interfaces:**
- `CachingRule { None, Even, Compensate, Plan }` (serde lowercase).
- `Caching { rule: CachingRule, capacity: u32, share: f64, lambda: f64, lookahead: u32 }`, default `{ None, 0, 0.5, 0.5, 1 }`, `#[serde(default)]`.
- `SeasonMode { Hemispheres, Global }` as `Seasons.mode`, default `Hemispheres`.
- `Central { enabled: bool }`, default `{ false }`.
- `Config.caching`, `Config.central`.

- [ ] **Step 1: Write the failing tests**, following Minds 4's config tests:
  - older configs without these fields load with the defaults;
  - validation: `share` and `lambda` in (0, 1] (field names `caching.share`, `caching.lambda`); `lookahead` 1–10; caching (rule ≠ none or capacity > 0) with ≠ 1 good is an error on `caching.rule`: "caching needs exactly one good"; `central.enabled` without mvt/goap or with capacity 0 is an error on `central.enabled`: "central-place foraging needs the marginal-value rule or planning, and a carrying limit";
  - live or reset-only: `caching.share`, `caching.lambda`, `caching.lookahead` live; `caching.rule`, `caching.capacity`, `seasons.mode`, `central.enabled` reset-only (`RESET_ONLY_PATHS` and `structural_changes`).
- [ ] **Step 2: Run them and check they fail.**
- [ ] **Step 3: Implement**, including `legacy.rs`'s explicit `Config` literal.
- [ ] **Step 4: Run** `cargo test -p sugarscape-core --lib config && cargo test -p sugarscape-core --test legacy --test golden`.
- [ ] **Step 5: Commit** ("Minds 5: the caching, central and seasons-mode config").

---

### Task 2: Global winter

**Files:** `rules/growback.rs`.

- In `rate_at`, when `seasons.mode == Global`: winter everywhere while `tick mod 2γ ≥ γ` (summer first, like the north), growing at α/β; summer at α.
- Add `pub(crate) fn is_winter(config, tick) -> bool` for the global mode (the Flumps' calendar, used by Task 5).
- **Tests:** global mode flips every γ ticks for every row; `global` with β = 1 equals seasons off over 500 ticks of a preset (fingerprints equal); hemispheres unchanged (golden).
- **Commit** ("Minds 5: a winter everywhere at once").

---

### Task 3: Caching mechanics

**Files:** `minds/caching/mod.rs` (new), `minds/mod.rs`, `agent.rs` (`Agent.caches: BTreeMap<u32, f64>`; the three literals), `rules/movement.rs` (`go_and_gather`, candidates), `rules/lifecycle.rs` or wherever deaths are processed, `world.rs` (`TickEvents.buried: f64`, `dug: f64`, `cache_lost: f64`, `dig_ages_sum: u64`, `digs: u32`; the world's fingerprint).

**Interfaces:**
- `pub(crate) fn bury(world: &mut World, id: AgentId, q: f64)` — at the Flump's current site; clamps q to holdings; adds to `buried`.
- `pub(crate) fn reserve(world: &World, id: AgentId) -> f64` and `surplus(...)`.
- `Agent.cache_since: BTreeMap<u32, u64>` (tick of the first unit buried at a site since it was last empty), for cache age.

**Behavior:**
- **Carrying limit** in `go_and_gather`: with capacity C > 0, good 0 gathered = min(level, C − holdings) (≥ 0); the rest stays on the site. Truffle value is capped the same way (the unpicked excess is lost with the spot's ripeness, as now; state it in a comment).
- **Dig** in `go_and_gather`: if the Flump has a cache at the target and holdings < reserve, it digs instead of harvesting: takes min(cache, room) (room = ∞ with no limit), removes the cache when emptied, counts `dug`, `digs`, `dig_ages_sum`.
- **Caches as candidates:** in the candidate builders every rule uses (`candidates`, `candidates_with_memory`, and forage.rs's `reachable_candidates` through them), when holdings < reserve, each own cache joins as a candidate valued at its amount (merged with that site's own believed value by taking the larger). Unreachable caches (`walled_apart`) are skipped (Review Focus 2).
- **Death:** a dying Flump's caches are summed into `cache_lost` and dropped (Review Focus 3).
- **Fingerprint:** hash caches only when non-empty, so every existing fingerprint is unchanged.
- **Tests:**
  - carrying limit on a rich site (Review Focus 1);
  - bury then dig conserves exactly; conservation over 300 ticks of a small world with a scripted bury each tick (Σ sites + holdings + caches + eaten + cache_lost constant up to growback, which the test accounts for);
  - a hungry Flump walks to its cache and digs it; a walled-off cache is ignored;
  - reduction: rule none + capacity 0 leaves golden and `tests/minds.rs` reductions unchanged.
- **Commit** ("Minds 5: carrying limits, burying and digging").

---

### Task 4: Episodic memory and the cycle finder

**Files:** `minds/caching/episodes.rs` (new).

**Interfaces:**
```rust
/// The smallest p in 1..seq.len() with seq[i] == seq[i + p] for all i; None if none.
pub fn period<T: PartialEq>(seq: &[T]) -> Option<usize>;
/// The value k steps past the end, by the period; None if no period.
pub fn extrapolate<T: PartialEq + Clone>(seq: &[T], k: usize) -> Option<T>;
pub struct Episodes { pub days: Vec<Episode> }   // capacity bounded (MEMORY_CAP)
pub struct Episode { pub place: u32, pub food: bool }
impl Episodes {
    /// Where the Flump will be and whether it'll find food, `k` days ahead (1 = tomorrow).
    pub fn predict(&self, k: usize) -> Option<Episode>;
}
```
- `predict` extrapolates the place sequence and the food sequence **separately** (their periods differ: 3 and 2 in Amodio).
- A period must repeat at least once within the span (p < len) — that's the definition above; with len 1 there's no period (Review Focus 4). A constant sequence of length ≥ 2 has period 1.
- **Tests:** brute force on 1 000 random sequences over alphabets of size 1–3 and lengths 1–12 (the period is minimal and consistent); Amodio's day 1–9 sequences give places K1 (day 10), K2 (11), K3 (12) and, Food-First, food absent (10), present (11), absent (12); Raby's alternating sequence gives period 2.
- **Commit** ("Minds 5: what-where-when memory and the cycle finder").

---

### Task 5: The caching rules, in the field

**Files:** `minds/caching/rules.rs` (new), `minds/mod.rs` or `world.rs` (the burial hook after each Flump's move), `agent.rs` (`Agent.weights: BTreeMap<u32, f64>` for compensate, `Agent.episodes: Option<Episodes>`, `Agent.winter: WinterRecord { intake: f64, ticks: u32, sites: BTreeSet<u32> }` for plan).

**Pure allocation (shared with the lab):**
```rust
/// How much of `amount` each place gets; places in ascending order; whole units in the lab.
pub fn allocate_even(amount: f64, places: &[u32]) -> Vec<(u32, f64)>;
pub fn allocate_compensate(amount: f64, places: &[u32], weights: &BTreeMap<u32, f64>) -> Vec<(u32, f64)>;
pub fn allocate_plan(amount: f64, needy: &[u32]) -> Vec<(u32, f64)>;
```
Remainders in whole-unit mode go to places in ascending order (spec: "remainders go to compartments in K order").

**Field behavior (after the move and harvest, each tick):**
- `compensate` updates the weight of the site it just harvested: w ← w × (1 − λ) if it found food (> 0 of good 0), starting from 1.
- `even`: bury `share × surplus` at the current site.
- `compensate`: bury `min(surplus, share × surplus × w / w̄)`, w̄ the mean weight over its known sites (weights map entries, default 1 each).
- `plan`: in summer, `shortfall = metabolism × γ − forecast − Σ caches`, forecast = last winter's `intake / ticks × γ`, or 0 before its first winter; bury `min(surplus, shortfall)` if > 0, at the current site if it's in last winter's `sites` or if it has no winter record. In winter it records intake, ticks and sites (a new record each winter).
- Seasons must be on for `plan` to cache; with seasons off, `plan` buries nothing (no winter to plan for — state it).
- Counts `buried` per tick.
- **Tests:** each allocation by hand; `plan`'s shortfall and first-winter worst case; `compensate`'s weights; a small global-winter world where `plan` stops burying once its caches cover the shortfall; rule none unchanged (golden).
- **Commit** ("Minds 5: burying by even share, by compensating weights, and by plan").

---

### Task 6: The lab harness and the oracles

**Files:** `minds/caching/lab.rs` (new), `tests/caching_lab.rs` (new).

**The rig:** a small world (e.g. 13 × 7, not a torus in effect: walls enclosing) with a hall and three compartments K1, K2, K3 opening onto it through doorways (Minds 2 walls); one good; walking.
**The harness:**
```rust
pub enum Protocol { Raby { breakfast_first: bool }, Amodio { food_first: bool } }
pub struct LabResult { pub caches: [u32; 3] }   // units cached in K1, K2, K3 on the test evening
pub fn run_lab(protocol: Protocol, rule: CachingRule, params: LabParams, seed: u64) -> LabResult;
```
- A day: the Flump is placed in the day's compartment with the doorway walled (the harness edits walls through the world's existing API) and the compartment's sites set to food or none; it spends the morning there under its decision rule, recording its episode (place, food) at day's end. `compensate` updates weights as in the field.
- Raby: K1 and K3 alternate for 6 days, breakfast in one; K2 is closed. Amodio Experiment 2: places K1,K2,K3,K1,…; food on odd days (Food-First) or even (Empty-First).
- Test evening: F = 30 units in the hall, reserve 0, doorways open; the rule's allocation is buried, walking to each compartment's caching site (a fixed tray site per compartment).
- **Oracle tests (exact):**
  - Raby, both counterbalancings: `even` 15/15; `compensate` (λ 0.5: breakfast compartment w = 0.125, no-breakfast w = 1) → 27/3 by the whole-unit rule — derive and assert; `plan` (lookahead 1) all 30 in the compartment predicted for tomorrow if it lacks food, else nothing — derive per counterbalancing and assert.
  - Amodio Food-First: `plan` lookahead 1 → all in K1; `compensate` → most in K2 (derive the exact split from w: K1 food on days 1, 7; K2 on 5; K3 on 3, 9); `even` → 10/10/10. Empty-First: derive and assert.
  - Amodio `plan` lookahead 3: derive from the cycle finder and assert what it produces; compare with the paper's stated FPH 2 ("across compartments K1 and K2"). If they differ, **stop and report** the derivation in the task report — don't change either; the controller amends the spec.
- **Lab population:** `run_population(protocol, rule, n, seed)`, with per-Flump `share` and `λ` drawn from narrow ranges (share 0.4–0.6, λ 0.3–0.7) through the world's RNG; returns per-Flump `LabResult`s.
- **Commit** ("Minds 5: Raby's and Amodio's protocols, with each hypothesis as an oracle").

---

### Task 7: Central-place foraging

**Files:** `minds/central.rs` (new), `agent.rs` (`Agent.home: Option<Pos>`, `Agent.load_trip: f64`, `Agent.delivery_rate: f64`), `minds/mod.rs` (dispatch when `central.enabled`), `world.rs` (`TickEvents.deliveries: u32`, `delivered: f64`).

- Homes: set to the placement site at birth when `central.enabled`. The larder is the Flump's cache at home.
- **The rule (under `mvt`):** ρ is the delivery rate: ρ ← ρ + α(delivered_this_tick − ρ), travel ticks counting 0. Away from home, in a patch: harvest while the best site within distance 1 yields ≥ ρ and holdings < capacity; otherwise head home. At home: bury all holdings above one tick's need into the larder (a delivery: counts `deliveries`, `delivered`, and the trip's load), then leave for the best known site. It eats from holdings, and at home digs the larder if holdings are short.
- **Under `goap`:** the goal is "deliver G" (G = metabolism × horizon); the plan's last step is home. Report only.
- **Analytic test:** one patch at distance d with a known run-down loading curve (sites of values v₁ ≥ v₂ ≥ …, harvested nearest first); compute by hand the load that maximizes delivered ÷ (2d + time in patch) — the tangent construction — and assert the rule's steady-state load equals it (within one site's value) for d = 2, 5, 10.
- **Commit** ("Minds 5: central-place foragers carry loads home").

---

### Task 8: Series

**`stats.rs`** (optional pattern): when caching is on, `cached` (Σ caches), `buried`, `dug`, `recovery` (Σ dug ÷ Σ buried, cumulative; 0 with none buried), `mean_cache_age` (dig_ages_sum ÷ digs). When central is on, `mean_load` (delivered ÷ deliveries) and `trips` (deliveries ÷ alive). **Tests. Commit.**

---

### Task 9: Presets, with the winter world's balance measured first

**Step 1: measure before building on it.** A throwaway probe (deleted before committing): the field world is `walk-capacity`'s landscape (or the Minds 4 `goap-mvt` torus if walk-capacity can't meet the principle — report why) with `seasons.mode: global`, walking, rule M, memory as in `mem-open`. Record:
- (a) summer surplus per Flump per summer (mean harvest − metabolism × γ);
- (b) the population's winter need (alive × metabolism × γ) against winter regrowth (Σ capacities' regrowth × γ / β).

The principle: (a) ≥ 1.5 × one Flump's winter need, and winter regrowth < 0.5 × the winter need. Adjust mechanically: β first (up to 32), then the population; γ = 100 unless the principle can't be met. Then 5 seeds × 1 000 ticks under `none` and `even`: report survival after the first winter. If `none` survives the first winter anyway, or `even` dies out, report BLOCKED with the numbers.

**Presets:** `cache-winter-none`, `-even`, `-compensate`, `-plan`, `cache-winter-mixed` (a quarter on each rule: a reset-only `caching.mixed: bool`, default false, assigns each founder `none`, `even`, `compensate`, `plan` round-robin by agent id, with no draw, stored as `Agent.caching_rule`; children take their parent's rule; add it in this task with its config test), `central-near`, `central-far`, `central-linear` (sites with instant growback — linear loading), `cache-raby` and `cache-amodio` (the rig at the test evening, 8 and 6 Flumps). Titles (drafts):
- "Winter with nothing put away"
- "Flumps who bury a share of every surplus"
- "Flumps who bury more where food has been scarce"
- "Flumps who plan for winter bury what they'll need"
- "Four ways to face winter, side by side"
- "Carrying loads home from a near patch"
- "Carrying loads home from a far patch"
- "Carrying loads home when the patch never runs down"
- "Where scrub-jays' breakfast lessons send each kind of cacher"
- "Where Amodio's rotating compartments send each kind of cacher"

Every preset's `source` names "Minds 5". Update the counts; record only new golden entries; `tests/minds.rs` reductions pass. **Commit.**

---

### Task 10: Sweeps

`cache-capacity` (survival after the first winter against capacity, each rule's preset as base), `cache-winter` (against γ), `central-distance` (mean load against the patch's mean distance). `BUILTINS` + 3; ids in `sweep.rs`'s test and the WASM list. Run `central-distance` with the CLI and record the table. **Commit.**

---

### Task 11: WASM

Pin `cache-winter-plan`'s golden. `AgentView.caching: Option<CachingView { holdings_cap: f64, caches: Vec<[u32; 3]>, total: f64, forecast: Option<f64> }>` and `AgentView.central: Option<CentralView { home: [u32; 2], last_load: f64 }>` through `inspect`. A WASM test inspects a caching Flump. **Commit.**

---

### Task 12: The page

- **Caching group (`minds: true`, titled "Caching (Minds 5)"):** rule (none, even, compensate, plan; reset), capacity (0–200, reset), share (0.05–1, live), λ (0.05–1, live), lookahead (1–10, live).
- **In the same Minds group:** the seasons mode select (hemispheres, global; reset; the book's Seasons group is unchanged) and central-place on/off (reset).
- **The menu:** `usesMinds` and `MINDS_TITLES` as in the Global Constraints, with tests in `models.test.ts` (a caching config and a global-winter config are Minds worlds; the Minds 5 presets group under "Minds 5: caching").
- Seed defaults when the objects are absent (the Minds 1–4 pattern).
- **Inspect:** "Carrying x of C", "Caches: n, holding y" (singular at 1), the forecast under plan, home and last load under central.
- **Map:** the inspected Flump's caches drawn as small markers.
- **Charts:** "Caching" (cached, buried, dug) and "Recovery" (0–1) when caching is on; "Loads" (mean load) and "Trips" when central is on.
- **Tests:** schema defaults and the Inspect text. **Browser check. Commit.**

---

### Task 13: The survey, measured descriptions and the cost table

`survey/src/claims/minds5.rs`, per the spec's Survey section; reuse Minds 3–4 helpers as `pub(crate)`. **Commit the judges and thresholds first, in their own commit, before running anything.**
- **Raby:** per rule, paired per Flump over a population of 8 × 20 seeds, no-breakfast caches > breakfast caches; reported beside 16.3 against 5.4.
- **Amodio:** per rule and group, the modal pattern matches the rule's oracle; then a multinomial likelihood comparison of the four patterns (FPH 1, FPH 2 as derived, CCH, compartment-independent) on the pooled population data, with the paper's Bayesian comparison used only if it can be written exactly from its text (state which). Reported beside 0.997 and 0.72.
- **Winter:** per rule against `none`, paired: survival after the first winter and after five; `plan` against `even` and `compensate`; recovery and cache age; the mixed world's survivor shares. Every survival and wealth figure also per founding Flump, dead as 0.
- **Central place:** per-seed slope of mean load on distance > 0; near-far within a habitat equal (paired difference within a tolerance stated in the judge before running); linear loading reported; GOAP's loads reported.
- **Usage:** bury and dig counts per world; share of buried sugar never dug.
- Describe the presets by measurements (causes "likely" where not isolated). Golden unchanged. **Cost table:** µs per Flump-tick for the new presets beside Minds 4's, same run. **Commit.**

---

### Task 14: Docs

- README: a Minds 5 section.
- The program document: status "Minds 1–5 done"; results; the cost table; the next target (the campaign on pilfering, watching and deception, or Minds 6 behavior trees — state both, pick neither).
- The roadmap line.
- The spec's amendments: the lab rig as built, the balance as measured, the rulings, the FPH 2 derivation outcome, and the values.
- The full suite (cargo, web, wasm, clippy, fmt). **Commit.**
