# Minds 3: memory, belief and truffles — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:**
- Flumps that remember the sites they've seen and believe something about what those sites hold now;
- hidden truffle spots only memory can exploit;
- measurements of memory's value as an information asymmetry, of traplining and of the marginal value theorem.

**Architecture:**
- `minds/memory.rs` holds a Flump's remembered sites and the two beliefs.
- `Agent.remembers` is drawn at birth under memory only, and last.
- The move step records what each rememberer sees.
- `movement::candidates` adds remembered out-of-sight sites, at believed values, for rememberers.
- Truffles are a hashed, hidden per-site layer, gathered when a Flump stops on a ripe spot.
- Diagnostics count remembered-target choices, and how wrong the belief was at the moment of choosing.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), `survey/`, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-09-28-minds-3-memory-design.md`

## Global Constraints

- **Existing work is untouched.** Every golden entry, legacy fixture and pinned fingerprint stays green and **unedited**.
- **The defaults change nothing.** `memory.span: 0` and `truffles.share: 0` are the defaults, and they draw nothing new from `World.rng`.
- **`remembers` draws only under memory.** It's drawn from `World.rng` only when `memory.span > 0`, and after every other draw in `Agent::random` and in children.
- **Truffle spots are placed by hash,** `landscape::mix(site index ⊕ seed-derived constant)`, never by `World.rng`. The layout is independent of the world's seed and of memory.
- **Memory requires walking.** `span > 0` requires `movement.mode: walk` (validation error on `memory.span`: "remembered sites out of sight can only be walked to").
- **Memory isn't hashed, but it drives behavior.** Memory and `remembers` are behavioral state that isn't hashed. Checkpoints and keyframes clone them.
- **What a Flump doesn't know about remembered sites:**
  - Remembered out-of-sight candidates are never filtered by true occupancy, since the Flump doesn't know it.
  - Walls are known terrain and are excluded.
  - The utility mind's `crowd` is 0 for them, since it's unknown.
- **Ruling (occupied remembered target):** a walker's A* goal may be occupied, because a remembered target can be. If the site it would stop on is occupied (only possible at the target), it stops at the site before it on the path, or stays.
- **Ruling (diagnostics at choosing):** `belief_error` and `stale_choices` are measured when a Flump chooses a remembered out-of-sight target, as believed against true value then, not on arrival.
- **Deterministic and portable.** No new `ln`/`exp`/`pow`.
- American spelling. Titles follow `titles.rs`.
- Every commit ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- Work in `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-3` (branch `minds-3`).

## Review Focus

1. **A rememberer walking to a remembered site that's now occupied** stops short, with no panic and no step onto the occupant. Owned by Task 4.
2. **A remembered site that has since become a wall can't happen** (walls change only on reset). A remembered site next to a wall is still reachable only through A*. Owned by Task 4.
3. **Old configs without `memory` or `truffles`** load as the book. Switching `belief` live works. `span` and `share` are reset-only. Owned by Task 1.
4. **A child born under memory** gets its own `remembers` draw and an empty memory: children don't inherit memories. Owned by Task 2.
5. **Truffles with a zero-width world region** (every site walled but a few, or share 1.0) place spots only on non-wall sites, and `share: 1.0` gives every non-wall site a spot. Owned by Task 3.

---

## File structure

- `crates/sugarscape-core/src/config.rs`, `legacy.rs`: `Memory`, `Belief`, `Truffles`.
- `crates/sugarscape-core/src/minds/memory.rs` (new): `Memory`, `Seen`, `TruffleSeen`, `observe`, `forget`, `believed_level`, `believed_truffle`.
- `crates/sugarscape-core/src/agent.rs`: `remembers`, `memory` (the three `Agent` literals); `rules/sex.rs` (the child's draw).
- `crates/sugarscape-core/src/rules/truffles.rs` (new), `world.rs` (the `truffles: Vec<Option<u64>>` layer), `landscape.rs` (`mix` becomes `pub(crate)`).
- `crates/sugarscape-core/src/rules/movement.rs`: candidates with memory, gathering truffles in `go_and_gather`, `arrive`'s occupied-target stop. `minds/utility.rs`: `crowd` 0 for remembered sites. `rules/mod.rs`: `memory::observe` after the move.
- `crates/sugarscape-core/src/world.rs`: `TickEvents` gains the memory and truffle counters.
- `crates/sugarscape-core/src/stats.rs`: the optional memory and truffle series.
- `crates/sugarscape-core/src/edit.rs`: `AgentView.memory` (a summary) and `inspect_memory` (sites for the overlay).
- `presets.rs`, `titles.rs`, `tests/golden.rs`, `tests/minds.rs`: presets and reductions.
- `sweeps/mem-span-recall.json`, `sweeps/mem-span-project.json`, `sweeps/mem-share.json`; `sweep.rs`.
- `crates/sugarscape-wasm/src/lib.rs` (the memory overlay call) and `tests/web.rs`.
- `web/src/...`: the Memory and Truffles groups, Inspect, the overlay and the charts.
- `survey/src/claims/minds3.rs`.
- Docs.

---

### Task 1: The `memory` and `truffles` config

**Files:** `crates/sugarscape-core/src/config.rs` (types after `Wall`; `Config` fields after `walls`; `Default`; validation; `RESET_ONLY_PATHS`, `reset_only`, `structural_changes`; tests), `legacy.rs`.

**Interfaces:**
- `Belief { Recall, Project }`, default `Project`, serde `recall`/`project`.
- `Memory { span: u32, share: f64, belief: Belief }`, `Copy`, default `{ 0, 1.0, Project }`, `#[serde(default)]`.
- `Truffles { share: f64, value: f64, regrow: u32, seed: u32 }`, `Copy`, default `{ 0.0, 5.0, 30, 1 }`, `#[serde(default)]`.
- `Config.memory` and `Config.truffles`.

- [ ] **Step 1: Write the failing tests** (following Minds 2's `movement_and_walls_*` tests):
  - The defaults; configs without `memory` or `truffles` load with the defaults; a partial `memory: { "span": 50 }` gets `share` 1 and `project`.
  - Validation:
    - `span` 0–10 000;
    - `share` 0–1 (NaN rejected);
    - `truffles.share` 0–1, `value` ≥ 0 and finite, `regrow` 1–10 000;
    - `span > 0` with `movement.mode: jump` is an error on `memory.span`, while `span > 0` with walk is valid.
  - Reset-only:
    - `structural_changes` flags `memory.span`, `memory.share`, `truffles.share` and `truffles.seed`;
    - `memory.belief`, `truffles.value` and `truffles.regrow` are live;
    - a schedule may set `memory.belief` and `truffles.regrow`;
    - a schedule setting `memory.span`, `memory` or `truffles.share` is refused "only on reset".

- [ ] **Step 2: Run them and check they fail.**

- [ ] **Step 3: Implement**, mirroring Task 1 of Minds 2.
  - Validation messages:
    - "must be between 0 and 10000";
    - "must be between 0 and 1";
    - "must be a number ≥ 0";
    - "must be between 1 and 10000";
    - "remembered sites out of sight can only be walked to".
  - `RESET_ONLY_PATHS` gains `"memory"`, `"memory.span"`, `"memory.share"`, `"truffles"`, `"truffles.share"` and `"truffles.seed"`.

- [ ] **Step 4: Run** `cargo test -p sugarscape-core --lib config && cargo test -p sugarscape-core --test legacy --test golden`.

- [ ] **Step 5: Commit** ("Minds 3: the memory (span, share, belief) and truffles config").

---

### Task 2: Memory: remembering what a Flump sees

**Files:** `minds/memory.rs` (new; `pub mod memory;` in `minds/mod.rs`), `agent.rs`, `rules/sex.rs`, `testkit.rs`, `rules/mod.rs`.

**Interfaces:**
- Produces:

```rust
/// A remembered site (Minds 3): the levels seen, the most ever seen there, when it was last seen, and
/// what the Flump knows of a truffle spot there.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seen { pub levels: [f64; MAX_GOODS], pub most: [f64; MAX_GOODS], pub tick: u64, pub truffle: Option<TruffleSeen> }

/// A known truffle spot: whether it was ripe when last seen, and when that was (a harvest counts as seen unripe).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TruffleSeen { pub ripe: bool, pub tick: u64 }

/// A Flump's remembered sites, by site index (deterministic order).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Memory { pub sites: std::collections::BTreeMap<u32, Seen> }

/// After the move: a rememberer records every site in its sight and its own site.
pub(crate) fn observe(world: &mut World, id: AgentId);
/// Drops entries last seen more than `span` ticks ago.
pub(crate) fn forget(memory: &mut Memory, now: u64, span: u32);
/// The believed level of good `i` at `site` now, under `belief`.
pub fn believed_level(seen: &Seen, i: usize, now: u64, rate: f64, instant: bool, belief: Belief) -> f64;
/// Whether a known spot is believed ripe now, under `belief`.
pub fn believed_ripe(t: &TruffleSeen, now: u64, regrow: u32, belief: Belief) -> bool;
```

- `Agent.remembers: bool` and `Agent.memory: Memory` are added to the three literals.
- `remembers` is drawn in `Agent::random` and for children as `rng.gen_bool(config.memory.share)`, **only when `config.memory.span > 0`**, and **after every existing draw**. The child's memory starts empty.

- [ ] **Step 1: Write the failing tests** (in `memory.rs`):
  - `observe` records the Flump's own site and every site from `world.sight` (walls stop sight), with levels, `tick = world.tick` and `most` as the running max. It doesn't record for a non-rememberer.
  - A second `observe` updates `levels` and `tick`, and raises `most` only upward.
  - `forget` drops exactly the entries older than `span`: an entry at age `span` stays, and one at `span + 1` goes.
  - `believed_level`:
    - `recall` returns `levels[i]`;
    - `project` returns `min(levels[i] + rate · age, most[i])`;
    - `instant` gives `most[i]`;
    - at age 0 it's `levels[i]`.
  - `believed_ripe`:
    - `recall` returns what was seen;
    - `project` returns seen-ripe, or `age >= regrow`.
  - The draws:
    - With `span` 0, `Agent::random` makes the same draws as before. Compare the rng state after `Agent::random` against a clone advanced by the old draw list, or assert that a world built with `span` 0 has the same fingerprint as today's golden `ii-2-unit`.
    - With `span > 0` and `share` 1, every Flump remembers; with `share` 0, none does.
    - A child under memory gets its own draw and an empty memory. The sex test gives the parents memories and asserts the child's is empty.

- [ ] **Step 2: Run them and check they fail.**

- [ ] **Step 3: Implement.** In `rules/mod.rs` `agent_turn`, after the decision's harvest line and before `metabolize`: `if world.config.memory.span > 0 { crate::minds::memory::observe(world, id); }`.
  - `observe` reads the agent's `pos` and `vision`, builds the list first (from `world.sight` plus its own site), then writes to the agent's memory.
  - It records `levels` from `site.resource` (goods `0..n`), and keeps each existing entry's `truffle` field. Task 3 sets it.
  - It calls `forget` every 16 ticks per agent (`world.tick % 16 == 0`), and always lazily in Task 4's candidates.

- [ ] **Step 4: Run** `cargo test -p sugarscape-core` (golden and legacy unchanged; the reductions in `tests/minds.rs` pass).

- [ ] **Step 5: Commit** ("Minds 3: Flumps that remember what they've seen").

---

### Task 3: Truffles

**Files:** `rules/truffles.rs` (new; `pub mod truffles;` in `rules/mod.rs`), `world.rs` (`truffles: Vec<Option<u64>>`, built in `with_landscapes`; `pub fn truffle(&self, p: Pos) -> Option<bool>`, where `Some(ripe)` marks a spot; `pub(crate) fn truffle_ripe_at` if useful), `landscape.rs` (`mix` becomes `pub(crate)`), `movement.rs` (`go_and_gather`), `minds/memory.rs` (`observe` records the spot the Flump stands on), and `world.rs` `TickEvents` (`truffles_found: u32`, `truffles_by_rememberers: u32`).

**Interfaces:**
- `truffles::has_spot(index: usize, seed: u32, share: f64) -> bool` is `mix(index as u64 ^ (u64::from(seed) << 32 | 0x5eed)) < threshold`, where `threshold = (share * 2⁶⁴)` saturating, and `share >= 1.0` means always. State the exact expression in a doc comment. No spot on walls.
- The layer: `None` means no spot, `Some(t)` means ripe at tick `t`. All spots start `Some(0)`, ripe.
- `go_and_gather`, after gathering the site's goods: if the stop site has a spot ripe now (`ripe_at <= world.tick`), add `truffles.value` to good 0's harvest and the agent's holdings, set `ripe_at = tick + regrow`, and count `truffles_found` (plus `truffles_by_rememberers` if the agent remembers).
- `observe` records `TruffleSeen { ripe, tick }` for the Flump's own site if it has a spot. A Flump that just harvested sees it unripe at `tick`.

- [ ] **Step 1: Write the failing tests:**
  - **Layout:**
    - `has_spot` is deterministic and independent of the world seed: two worlds with seeds 1 and 2 share a layout.
    - The share of spots over a 100 × 100 grid is within 1 % of `share` = 0.05.
    - `share` 0 gives none and `share` 1 gives every non-wall site a spot.
    - There are no spots on walls.
  - **Gathering:**
    - A Flump stopping on a ripe spot gains `value` extra sugar, and the spot becomes ripe at `tick + regrow`.
    - Stopping again before then gains nothing extra; stopping at `tick + regrow` gains it again.
  - **Hidden:** spots don't change any candidate's value for a Flump that doesn't know them.
  - **Counts:** `truffles_found` and `truffles_by_rememberers` are counted.
  - **Unchanged without truffles:** with `share` 0, every golden entry is unchanged (the golden suite).

- [ ] **Step 2: Run them and check they fail.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test -p sugarscape-core`.
- [ ] **Step 5: Commit** ("Minds 3: truffles — hidden spots that ripen again after a harvest").

---

### Task 4: Deciding with memory

**Files:** `rules/movement.rs` (`candidates`, `arrive`), `minds/utility.rs` (`act`'s crowding for remembered sites), `world.rs` `TickEvents` (`moves: u32`, `remembered_moves: u32`, `belief_error_sum: f64`, `stale_choices: u32`), and `tests/minds.rs`.

**Interfaces:**
- `candidates(world, id)` returns the same list as today, extended for a rememberer:
  - **Sites in sight** get their true value, plus `truffles.value` if the Flump knows a spot there that it believes ripe. Knowing a spot means it's in the Flump's memory with `truffle: Some`.
  - **Remembered sites out of sight** are appended, after the sight list, in memory (site index) order:
    - excluding the Flump's own site, sites in sight and walls, and forgetting stale entries first;
    - valued by believed levels (through rule M's welfare) plus the believed truffle value;
    - at distance = torus Manhattan distance.
- `candidates` also returns which entries are remembered: add `pub(crate) fn candidates_with_memory(world, id) -> (Vec<(Pos, u32, f64)>, usize /* index where remembered entries start */)` and keep `candidates` as `.0`, so rule M's and the utility mind's code stay one path.
- The value closure is split so welfare can be computed from explicit levels: `fn welfare_of(world, agent, levels: &[f64]) -> f64`. Refactor the existing closures to call it, with byte-identical arithmetic for the non-memory path. The golden tests prove it.
- `act` and `utility::act`:
  - when the chosen target is a remembered entry, count `remembered_moves`, and add |believed − true| (true = rule M's welfare of the site now, plus the truffle value if it has a ripe spot now) to `belief_error_sum`, and count `stale_choices` if true < believed;
  - count `moves` for every rememberer's choice.
- `utility::act`: `crowd` is 0 for remembered entries (index ≥ start).
- `arrive`: if the chosen `stop` is occupied (possible only at a remembered target), stop at the previous path site instead, or stay if there is none.

- [ ] **Step 1: Write the failing tests:**
  - A rememberer that saw sugar at a site now out of sight chooses it when it's the best believed value, and walks toward it.
  - A non-rememberer beside it doesn't.
  - `recall` against `project`: a harvested site remembered at 0 isn't chosen under `recall` but is under `project` once enough ticks have passed.
  - A remembered site that's occupied at arrival: the walker stops one step short and nobody panics (Review Focus 1).
  - Remembered sites are never filtered by true occupancy. A remembered site that another Flump now stands on out of sight is still a candidate.
  - Walls are excluded from remembered candidates.
  - The counters: a choice of a remembered target with a true value lower than believed counts `stale_choices` and the right `belief_error_sum`.
  - The utility mind with crowding scores remembered sites with `crowd` 0.
  - **The reduction, in `tests/minds.rs`:** every walking preset, and every other non-combat preset, has its golden fingerprint with `memory.span` 0 and `truffles.share` 0 set explicitly. That's the exact reduction: memory is off and draws nothing.
  - **With `span` > 0 and `share` 0**, the extra `remembers` draws change later draws, so fingerprints can't match. Test instead that no Flump remembers, `remembered_moves` stays 0, and no candidate list contains a remembered entry (a unit test in `movement.rs`).

- [ ] **Step 2: Run them and check they fail.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test -p sugarscape-core --release` (golden, legacy and all reductions).
- [ ] **Step 5: Commit** ("Minds 3: rememberers choose remembered sites out of sight, and we count how wrong their beliefs were").

---

### Task 5: Series

**Files:** `crates/sugarscape-core/src/stats.rs`.

**Interfaces:**
- When `memory.span > 0`, `MemoryStats` provides the series:
  - `remembering` (share alive);
  - `remembered_moves` (`remembered_moves / moves`, 0 if none);
  - `belief_error` (`belief_error_sum / remembered_moves`, 0 if none);
  - `stale_choices` (share of remembered choices);
  - `wealth_rememberers` and `wealth_others` (mean sugar, good 0; 0 for an empty group);
  - `wealth_advantage` (their difference, 0 unless both groups exist).
- When `truffles.share > 0`, `TruffleStats` provides `truffles_found` and `truffles_by_rememberers`.
- Both follow the patch series' optional pattern (`series_names`, `Snapshot`, `value`).

- [ ] **Step 1: Write the failing tests.** The series exist only under their switches. Test their values on a hand-built world, including empty groups (0, not NaN).
- [ ] **Step 2: Run them and check they fail.**
- [ ] **Step 3: Implement.**
- [ ] **Step 4: Run** `cargo test -p sugarscape-core && cargo test -p sugarscape-cli`.
- [ ] **Step 5: Commit** ("Minds 3: series for memory and truffles").

---

### Task 6: Presets, titles and golden entries

**Files:** `presets.rs`, `titles.rs`, `tests/golden.rs`.

**Presets:** use the spec's Environments table exactly: `mem-open`, `mem-catchment`, `mem-walled`, `mem-seasons`, `mem-truffles`, `mem-trapline`, `mem-mvt`.

- **`mem-mvt`:**
  - a 60 × 60 torus with 9 peaks (radius 4, height 4) at (10 + 20i, 10 + 20j), i and j in 0–2;
  - 10 Flumps with metabolism 1, endowment 50, vision 1–20, growback 0.25;
  - walk, the utility mind with travel 0.5, and memory (span 400, share 1, `project`).
- **`mem-trapline`:**
  - a 50 × 50 torus on a `flat` map with capacity 1, growback 0.1;
  - 20 Flumps with metabolism 1, endowment 30, vision 1–6;
  - walk; truffles (share 0.02, value 10, regrow 40, seed 1); memory (span 400, share 1).
- **The rest** build on existing presets through helpers, like Minds 2's `walk-*` (the book presets' configs must stay byte-identical). Descriptions state setup only, plus "Measured: see the survey", which Task 10 replaces.

**Titles** (drafts, after `ifd-wall`'s):
- "Remembering on the open sugarscape: little to gain";
- "Wanderers who remember the patches they've seen";
- "Remembering what lies beyond the wall";
- "Remembering the other hemisphere through the seasons";
- "Hidden truffles only rememberers come back for";
- "Foragers who learn a route between truffle spots";
- "When to leave a patch: a few foragers among nine patches".

Titles count +7; the Sugarscape preset count +7.

- [ ] **Step 1: Write the failing test.** Each preset validates, walks, and has memory; `mem-truffles` and `mem-trapline` have truffles; `mem-mvt` has 9 peaks at the stated centers.
- [ ] **Step 2: Implement.**
- [ ] **Step 3: Record the golden entries** (only the new ones).
- [ ] **Step 4: Run** `cargo test -p sugarscape-core --release`.
- [ ] **Step 5: Commit** ("Minds 3: seven memory presets").

---

### Task 7: Sweeps

**Files:** `sweeps/mem-span-recall.json`, `sweeps/mem-span-project.json`, `sweeps/mem-share.json`; `sweep.rs` (`BUILTINS` +3; the expected list, after `ifd-detour`).

- **`mem-span-{recall,project}`:**
  - base `mem-open`, with `memory.belief` set;
  - x = `memory.span` over 10, 25, 50, 100, 200, 400 (0 is invalid with `share`; the advantage at 0 is undefined);
  - series = `growback.rate` 0.25, 0.5, 1;
  - metric: `window_mean` of `wealth_advantage` from 200; 500 ticks; seeds 1–20.
- **`mem-share`:**
  - base `mem-truffles`;
  - x = `memory.share` over 0.1, 0.25, 0.5, 0.75, 0.9;
  - no series;
  - the same metric.
- Descriptions name what's measured, with no claims.

- [ ] **Step 1:** Append the ids to the test; check it fails.
- [ ] **Step 2:** Write the files and entries.
- [ ] **Step 3:** Run `cargo test -p sugarscape-core --lib sweep`. Then run `mem-span-project` with the CLI and record the table in the report.
- [ ] **Step 4: Commit.**

---

### Task 8: WASM

**Files:** `crates/sugarscape-wasm/src/lib.rs`, `crates/sugarscape-wasm/tests/web.rs`.

- A test pins `mem-truffles`' golden fingerprint (200 ticks, seed 1).
- The sweep list gains the three new sweeps.
- Add `inspect_memory(x, y) -> Vec<u32>`:
  - for the Flump at (x, y), a flat list of `[x, y, age, spot]` for each remembered site;
  - `spot` is 0 for no spot, 1 for a known spot believed unripe and 2 for one believed ripe;
  - empty for none.
- Test it.
- Run `wasm-pack test --node crates/sugarscape-wasm`.
- Commit.

---

### Task 9: The page

**Files:** `web/src/types.ts`, `web/src/schema.ts`, `web/src/ui/inspect-panel.ts`, `web/src/ui/grid-view.ts`, `web/src/ui/charts-panel.ts`, `web/src/engine.ts` (the `inspectMemory` call), plus tests.

- **Types:** `Memory { span; share; belief: 'recall' | 'project' }`, `Truffles { share; value; regrow; seed }`, `Config.memory?`, `Config.truffles?`, and `AgentView.memory?: { remembers: boolean; sites: number; spots: number }`.

  Task 2 or 4 fills `AgentView.memory` in `edit.rs` `inspect()`. Do it in this task if it hasn't been done, with a core test.
- **Rules panel:**
  - **Memory (Minds 3):**
    - `span` (number 0–10 000, reset);
    - `share` (0–1, step 0.05, reset);
    - `belief` (select, live);
    - note: "Memory needs walking (Movement: Walk). A remembered site's belief is what was seen (recall) or that plus growback since (project)."
  - **Truffles:**
    - `share` (0–1, step 0.01, reset);
    - `value` (live);
    - `regrow` (live);
    - `seed` (reset);
    - note: "Hidden spots, found only by stopping on them; they ripen again a fixed time after a harvest."
  - Each seeds a complete object from its defaults when absent (the Minds 1–2 pattern).
- **Inspect:** "Remembers: n sites (m truffle spots)" or "Doesn't remember", shown when memory is on.
- **Overlay:** for the inspected Flump, when memory is on, draw its remembered sites as small squares at alpha 0.4·(1 − age/span), and known spots as small circles (filled when believed ripe).
  - Use `--c2` for sites and `--accent` for spots.
  - Draw it before the planned path.
- **Charts:**
  - "Memory": `remembered_moves`, `stale_choices`, `belief_error`;
  - "Rememberers vs others": the two wealth series;
  - "Truffles": `truffles_found`, `truffles_by_rememberers`.
  - Each is shown when its series exist.
- **Tests:**
  - the schema groups, including defaults when the object is absent;
  - a pure function computing the overlay's alpha and shape from `[x, y, age, spot]` and `span`.
- **Browser check:** `mem-truffles` shows the groups, charts and overlay for an inspected rememberer.
- **Commit.**

---

### Task 10: The survey, and measured descriptions and titles

**Files:** `survey/src/claims/minds3.rs`, `survey/src/claims/mod.rs`, and the preset and title wording.

Claims, following the spec's Survey section. Twenty seeds unless stated. Judges are paired where the same seeds run in both arms. Measures computed in the survey:

- **Advantage:** the per-seed mean of `wealth_advantage` over ticks 200–500.
- **Time to first reach a patch** (`mem-catchment`): per Flump, the first tick it's on a patch; the medians for rememberers and others.
- **Traplining** (`mem-trapline`):
  - Step the world tick by tick and log each Flump's sequence of stops on truffle spots.
  - The return lengths are, for each visit, the number of spot visits since the last visit to the same spot.
  - The index is var(return lengths) ÷ the mean var over 999 shuffles of the sequence. The shuffles use a `rand_pcg` generator seeded from the world seed and the Flump id, never `World.rng`.
  - Take each Flump's index for sequences with at least 10 visits; the claim judges the per-seed median.
  - At 5 Flumps (a variant of `mem-trapline` with population 5) against 20: compare median revisit intervals in ticks.
- **MVT** (`mem-mvt`):
  - residence = consecutive ticks a Flump spends on one patch (`landscape::patch_of`), completed visits only;
  - the per-seed slope of mean residence on spacing, from four spacings (12, 16, 20, 24), for the utility mind with travel and for rule M.
- **Overstaying:** at each departure, the sugar gathered on the Flump's last tick in the patch, against its mean gathered per tick over the run so far. Report the share of departures where the last tick was below the mean.
- **Forgetting:** the best `span` per growback rate, from the `mem-span-*` sweeps' metric computed per seed; report whether it falls as the rate rises.
- **Hornvale:** `project` against `recall`, paired on the advantage and on `belief_error` (`mem-truffles`, `mem-open`).
- **Minds 2 follow-up:** `walk-capacity` with memory (span 100, share 1) against without, on the mean population over ticks 300–500, paired.

Then rewrite the descriptions and titles to the measured values, labeling causes as measured or "likely". Measure the cost table: µs per Flump-tick for `mem-open`, `mem-truffles`, `mem-mvt` and `mem-walled`, against Minds 2's. Golden must be unchanged. Commit.

---

### Task 11: Docs

- **README:** a Minds 3 section, placed after Minds 2's.
- **`docs/studies/2026-09-27-minds.md`:**
  - Status: "Minds 1–3 done; Minds 4 (GOAP and caching) next".
  - A Minds 3 results section and the cost table.
  - **Minds 4's target**, with the verified caching sources:
    - Vander Wall and Balda 1977: a flock of 150 nutcrackers cached 3.3–5.0 × 10⁶ piñon seeds, 2.2–3.3 times their needs;
    - Balda and Kamil 1992: recovery after up to 285 days;
    - Emery and Clayton 2001: jays re-cached when observed, but only if they had pilfered before (0.44 against 0.06 and 0.28 against 0.08);
    - Vander Wall and Jenkins 2003: pilferage of 2–30 % a day, tolerable because pilfering is reciprocal.
  - The design line: a carrying limit gives a reason to bury, winter gives a reason to plan, GOAP does the planning, and pilfering sets up the deception program.
- **Roadmap:** the Minds 3 line.
- **The spec's amendments:**
  - the two rulings (occupied remembered targets; diagnostics measured when choosing, so `stale_choices` replaces `stale_arrivals`);
  - the MVT residence series and sweep replaced by survey measures;
  - the measured values.
- **The full suite:** `cargo test --release`, the web build and tests, `wasm-pack`, clippy and fmt.
- **Commit.**
