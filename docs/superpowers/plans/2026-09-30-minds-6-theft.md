# Minds 6: theft — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Agents that stumble on each other's caches and pilfer them. Fixed mixes of hoarders and cheaters.
The measurements are cache fates, a bury cost, and an owner-memory switch. The survey tests Andersson and
Krebs's conditions and Vander Wall and Jenkins's reciprocity and pilferage-rate claims.

**Architecture:**
- `minds/caching/theft.rs` holds the find draw, pilfering and loot handling. It's called from
  `movement::go_and_gather` on arrival, beside the dig.
- Cache fates are recorded as caches end, in a world-level log that the survey reads.
- Cheaters are assigned by agent id at founding, as `caching.mixed` does.
- The arena is a new preset builder, not a world mode.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), `survey/`, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-09-30-minds-6-theft-design.md`

## Global Constraints

- **Existing fixtures are untouchable.** Every golden entry, legacy fixture and pinned WASM fingerprint stays
  green and **unedited**.
- **Defaults reduce to Minds 5.**
  - The defaults are `theft.find` 0, `theft.cheaters` 0, `caching.bury_cost` 0, `theft.owner_memory` on and
    `theft.loot` keep.
  - With these defaults every world is its Minds 5 self, bit for bit, with no new draw.
- **Draws.**
  - The find draw is `world.rng`, one draw per foreign cache on the arrival site, in owner-id order. It
    happens only when `find > 0`.
  - Cheaters are assigned with no draw: agent id i is a cheater iff ⌊i·s⌋ > ⌊(i − 1)·s⌋.
- **Validation.** Theft needs caching on. `find` and `cheaters` must be in [0, 1], and `bury_cost` must be
  ≥ 0. Theft is refused in `lab` and in `central.enabled` worlds. The error field names are `theft.find`,
  `theft.cheaters`, `caching.bury_cost` and `theft.find` (for the lab and central refusals).
- **Conservation.** Sites + holdings + caches + eaten + cache_lost is conserved through pilfering, under
  either loot rule and through bury cost. Loot that is eaten, and bury cost, count as eaten.
- **Fates.** Every cache ends in exactly one fate: `Dug`, `Pilfered { by }`, `Lost` (its owner died) or
  still buried at the query tick. The fate log is not hashed.
- **Minds menu.**
  - Every Minds 6 preset's `source` names "Minds 6".
  - `web/src/models.ts` gets `MINDS_TITLES['6'] = 'Minds 6: theft'`.
  - `usesMinds` also returns true for `theft.find > 0` or `theft.cheaters > 0`.
  - The new Rules group is `minds: true`.
- **Style.** Never write "Flump"; say "agent". Use American spelling. Titles follow `titles.rs`.
- **Survey judges** are committed before any survey run and never tuned.
- **Commits.** Every commit ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- **Where to work.** Use `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-6` (branch `minds-6`).
  For web tests, temporarily symlink the main checkout's `web/node_modules` and remove the link afterward.

## Review Focus

1. **A thief at the carrying limit** takes only its room. The rest stays as the owner's cache, with the
   owner unchanged. Owned by Task 3.
2. **An owner and a thief arriving on the same cache in one tick.** Turn order decides which happens first:
   the owner digs, or the thief pilfers. Each happens at most once, and conservation holds. Owned by Task 3.
3. **A cheater with `caching.mixed` on** is a cheater (rule `none`) whatever the round-robin says. Children
   take the acting parent's rule, cheater or not. Owned by Task 2.
4. **`owner_memory: off`.** Own caches are not candidates, and an owner standing on its own cache finds it
   only at rate `find`. With `find` = 0 the caches are never recovered and end `Lost` or buried. Owned by
   Task 3.
5. **Configs without `theft` or `bury_cost`** load as before. Live knobs are live and reset-only ones are
   reset-only. Owned by Task 1.

---

### Task 1: Config

**Files:** `config.rs`, `legacy.rs`.

**Interfaces:**
- `Theft { find: f64, owner_memory: bool, loot: Loot, cheaters: f64 }`, with default
  `{ 0.0, true, Loot::Keep, 0.0 }` and `#[serde(default)]`.
- `Loot { Eat, Keep }`, serde lowercase.
- `Caching.bury_cost: f64`, default 0.
- `Config.theft`.
- Live: `theft.find`, `theft.loot`, `caching.bury_cost`.
- Reset-only: `theft.owner_memory`, `theft.cheaters`, and the whole `theft` object, as Minds 5 did for
  `caching`.

- [ ] **Step 1: Failing tests**, following the tests of Minds 5's Task 1:
  - older configs load with the defaults;
  - every validation rule and message in the Global Constraints;
  - the live and reset-only split.
- [ ] **Step 2: Run them and check they fail.**
- [ ] **Step 3: Implement.** Include `legacy.rs`'s explicit literal. Add a `Theft::is_on()` that returns
  `find > 0 || cheaters > 0`.
- [ ] **Step 4:** run `cargo test -p sugarscape-core --lib config && cargo test -p sugarscape-core --test legacy --test golden`.
- [ ] **Step 5: Commit** ("Minds 6: the theft config and a bury cost").

---

### Task 2: Cheaters, bury cost and cache fates

**Files:**
- `minds/caching/mod.rs`: `bury` charges the bury cost and opens a fate record.
- `minds/caching/rules.rs`: `rule_of` returns `None` for cheaters.
- `agent.rs`: `Agent.cheater: bool`, set at founding by id; children inherit from the acting parent, as
  `caching_rule` does.
- `world.rs`: `World.cache_log: Vec<CacheRecord>`, and fates closed in dig, remove and pilfer.

**Interfaces:**
```rust
pub struct CacheRecord { pub owner: AgentId, pub site: u32, pub amount: f64, pub buried: u64, pub fate: Option<(u64, Fate)> }
pub enum Fate { Dug, Pilfered { by: AgentId }, Lost }
```
- One record per burial event, holding the amount buried at that event.
- A dig or a pilfer consumes records at that site in FIFO order, splitting a record when only part of it is
  taken. Keep this simple and exact, and document it.
- **Performance:** the log grows. Cap it at 1 000 000 records. Past the cap, stop logging, set a
  `cache_log_full` flag the survey checks, and document this.
- Bury cost: burying q takes `q × bury_cost` more from holdings, counted as eaten. `bury` clamps so that
  holdings never go negative: it buries the most that holdings can pay for.

**Tests:**
- cheaters by id in exact proportion, for s = 0.25, 0.5 and 1/3;
- cheaters under `mixed` (Review Focus 3);
- bury cost conserved;
- every record ends in one fate after dig, death and pilfer (once pilfer exists; use a stub test hook here);
- defaults unchanged (golden).

**Commit.**

---

### Task 3: Theft

**Files:** `minds/caching/theft.rs` (new); `rules/movement.rs` (`go_and_gather`: pilfer on arrival);
`minds/caching/mod.rs` (`join_caches`: own caches are not candidates when `owner_memory` is off).

**Behavior (spec "Mechanics"):**
- **On arrival.** When `find > 0`, draw once per foreign cache at the site, in owner-id order.
  - The first success is pilfered, up to the room left under the carrying limit. Any remainder stays with
    its owner.
  - Pilfering replaces the tick's harvest, the same way a dig does.
  - If the agent also has its own cache at that site and would dig it, the dig wins and no pilfer draw is
    made. State this rule.
- **Loot.** With `eat`, the loot is counted as eaten. With `keep`, it goes into the thief's holdings, and
  the burial hook may bury it again that tick.
- **`owner_memory: off`.**
  - Own caches are not candidates.
  - On arrival at its own cache the owner draws with probability `find`, like any thief. A success is a
    `Dug` fate, and it doesn't count toward "pilfered".
- **Events.** `TickEvents.pilfered: f64`, `pilfers: u32`, `pilfer_candidates: u32` (foreign caches
  existing at the tick's start, for the rate), and `owner_finds: u32`.

**Tests:**
- Review Focus 1, 2 and 4;
- conservation over 300 ticks under both loot rules, with theft and deaths;
- a binomial test that the pilfer frequency matches `find` over 10 000 scripted trials;
- the reduction: `find` 0 leaves golden unchanged.

**Commit.**

---

### Task 4: Series

**`stats.rs`** (optional pattern: present only when theft is on):
- `pilfered`;
- `pilferage_rate`, pilfers ÷ `pilfer_candidates` this tick, 0 if there are none;
- the cumulative fate shares `fate_dug`, `fate_pilfered`, `fate_lost` and `fate_buried`, carried forward
  like `recovery`, without being lossy;
- `hoarder_holdings`, `cheater_holdings`, `hoarder_alive` and `cheater_alive`, only when cheaters > 0.

**Tests. Commit.**

---

### Task 5: The arena, presets, and the balance measured first

**Step 1: measure before building on it** (throwaway probe):
- Take Minds 5's winter world (`presets.rs` `winter_world`, with `even`) and turn theft on.
- Find the smallest `find` on {0.01, 0.02, 0.05, 0.1, 0.2} at which the measured daily pilferage rate lands
  inside 2–30 % over ticks 100–200. Record the rate at each value.
- **Arena:** a walled square patch where n agents (n = 2, 4, 8) spend one winter. The site count and
  capacity are scaled so that sugar per agent is the same at every n. Measure its balance the same way:
  hoarders without theft survive the winter, and non-hoarders mostly don't.
- If no `find` in the list reaches 2–30 %, or the arena can't meet its balance, report BLOCKED with the
  numbers.

**Presets.** Each `source` names "Minds 6".
- `theft-winter`: hoarders only, theft on.
- `theft-winter-quarter`: a quarter cheaters.
- `theft-winter-half`: half cheaters.
- `theft-arena-2`, `theft-arena-4`, `theft-arena-8`: half cheaters each.

Descriptions describe the setup only (Task 9 measures them). Title drafts:
- "Hoarders who can be robbed"
- "A quarter of the agents never cache and steal"
- "Half the agents never cache and steal"
- "Two agents share a winter's caches"
- "Four agents share a winter's caches"
- "Eight agents share a winter's caches"

Update the counts and record only new golden entries. **Commit.**

---

### Task 6: Sweeps

- `theft-find`: pilferage rate and hoarder survival against `find`.
- `theft-cheaters`: the hoarder–cheater survival gap against the cheater share, 0.1–0.9.
- `theft-winter`: the cheater advantage against β (winter severity), for claim 8.

`BUILTINS` + 3, with the ids in `sweep.rs`'s test and the WASM list. Run `theft-cheaters` with the CLI and
record the table. **Commit.**

---

### Task 7: WASM

- Pin `theft-winter-half`'s golden in `web.rs`.
- `AgentView.theft: Option<TheftView { cheater: bool, stolen_by_me: f64, stolen_from_me: f64 }>` goes
  through `inspect`. The totals are per agent; keep running totals on `Agent`, not hashed.
- Add a WASM test.

**Commit.**

---

### Task 8: The page

- **Rules:** a "Theft (Minds 6)" group (`minds: true`):
  - find, 0–1, step 0.01, live;
  - owner memory, a checkbox, reset;
  - loot, a select, live;
  - cheaters, 0–1, step 0.05, reset;
  - bury cost, 0–2, step 0.05, live.
- **Menu:** `MINDS_TITLES['6']`, plus `usesMinds` extended for theft, with tests.
- **Inspect:** "Cheater: yes/no", "Stole: x", "Lost to thieves: y".
- **Charts:** "Pilferage" (the rate, 0–1), "Cache fates" (four shares, 0–1), and "Hoarders vs cheaters"
  (alive and holdings, shown when cheaters > 0).
- **Tests.** Browser check. **Commit.**

---

### Task 9: The survey, measured descriptions and the cost table

`survey/src/claims/minds6.rs`, following the spec's Survey section. Reuse the Minds 5 helpers as
`pub(crate)`. **The judges go in their own commit, first.**
- **Constant pilferage:**
  - Fit cohort survival per seed. The judge is the residual tolerance, stated in the judge commit.
  - Report r against 2–30 % with a median of 9 %.
- **Andersson and Krebs's threshold:**
  - Arena runs sweep `find`, `bury_cost` and n.
  - Measure p_s (own caches dug ÷ own caches ended, excluding Lost) and p_o (pilfered ÷ ended) from the fates.
  - C/G = bury_cost.
  - The judge is the share of runs whose advantage sign agrees with the condition.
- **Frequency independence:** the slope's confidence interval must include 0 across cheater shares 0.1–0.9.
- **Equal recovery:** with `owner_memory` off, cheaters win at every share, per seed.
- **The mixed equilibrium:** locate the crossing, where one exists, and report it.
- **Reciprocity:** `keep` against `eat`, paired on seeds.
- **Loss that hoarding withstands:** against 18 %.
- **The mild-winter cheater:** a per-seed slope against β.
- **Usage:** fate shares per world, and the `cache_log_full` flag.
- **Descriptions and titles:** measured, per founder too, with causes labeled "likely".
- **Cost:** the table beside Minds 5's.

Golden unchanged. **Commit.**

---

### Task 10: Docs

- **README:** a Minds 6 section.
- **The program document:** the status "Minds 1–6 done"; the pilfering campaign's outline (P1–P4, with
  P1b as Minds 7); the results; the cost table; and the next target, P1b.
- **The roadmap line.**
- **The spec's amendments:** the balance as measured, the rulings and the values.
- Run the full suite: cargo, web, wasm, clippy, fmt. **Commit.**
