# Minds 8: watching — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Agents who see another agent bury remember the cache and raid it on purpose. The survey asks
whether that reaches the field's pilferage rates, breaks Andersson and Krebs's condition, and behaves as a
producer–scrounger game.

**Architecture:**
- `minds/caching/watching.rs` (new) holds sighting, the seen-cache memory, joining seen caches to the
  candidate list, and the raid on arrival.
- Sighting is called from `caching::bury`. The raid is called from `movement::go_and_gather`, between the dig
  and Minds 6's `theft::stumble`. Joining sits beside `join_caches`.
- A raid's take goes through Minds 6's `theft::pilfer`, so fates, counts and conservation are Minds 6's.
- Watchers are dealt by id at founding, by the same rule as Minds 6's cheaters.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), `survey/`, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-10-01-minds-8-watching-design.md`

## Global Constraints

- **Existing fixtures are untouchable.** Every golden entry, legacy fixture and pinned WASM fingerprint stays
  green and **unedited**. Only new entries are added.
- **Defaults reduce to Minds 7.**
  - The defaults are `watching.on` false, `span` 7, `watchers` 1.0 and `raid_when` always.
  - With `on` false every world is its earlier self, bit for bit.
  - Nothing is allocated, observed or swept with `on` false.
- **No draws.** Watching never touches `world.rng`. Observers are visited in id order. A raid takes from the
  first remembered owner at the site, in owner-id order, that still has a cache there.
- **Watchers by id.** Founder id i watches iff ⌊i·s⌋ > ⌊(i − 1)·s⌋, the rule of
  `Theft::founder_cheats`. Factor it into one shared function both call. An agent born later doesn't watch.
- **Validation.**
  - Watching needs caching on.
  - `watchers` must be in [0, 1] and `span` ≥ 1.
  - Watching is refused in `lab` worlds and in `central.enabled` worlds, as theft is.
  - The error fields are `watching.on`, `watching.watchers` and `watching.span`.
- **Conservation.** A raid is a pilfer. Minds 6's ledger holds unchanged with watching on: Σ sites +
  holdings + caches + stomachs + eaten + what left with the dead = start + growback.
- **Minds menu.**
  - Every Minds 8 preset's `source` names "Minds 8".
  - `web/src/models.ts` gets `MINDS_TITLES['8'] = 'Minds 8: watching'`.
  - `usesMinds` returns true for `watching.on`.
  - The new Rules group is `minds: true`.
- **Style.** Never write "Flump"; say "agent". Use American spelling. Titles follow `titles.rs` and are
  written from measured results (Task 8).
- **Survey judges** are committed before any survey run and never tuned. The spec's "Questions and judges"
  section is binding, word for word.
- **Commits.** Every commit ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- **Where to work.** Use `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-8` (branch
  `minds-8`). For web tests, temporarily symlink the main checkout's `web/node_modules`, and remove the link
  afterward. Run `npm run build` before vitest, since a stale `wasm-pkg` fails tests.

## Review Focus

1. **A watcher with no room under the carrying limit** arrives at a seen cache that's still there.
   - It takes nothing, and the cache stays its owner's.
   - Its entries at the site are forgotten, and it harvests the site as usual.
   - The arrival counts as neither a raid nor a wasted raid; it counts in `seen_arrivals`.
   - Owned by Task 3.
2. **An owner and a watcher on the same cache in one tick.** Turn order decides.
   - If the owner digs first and empties the cache, the watcher's arrival is a wasted raid.
   - A partial dig leaves the rest to the raid.
   - Conservation holds either way. Owned by Task 3.
3. **Entries about the dead.** A seen cache whose owner has died (its caches gone as `Lost`) gives a wasted
   raid, with no panic and no lookup of a dead agent. A watcher that dies takes its entries with it. Owned by
   Tasks 2 and 3.
4. **Watching turned off live, then on again.**
   - While off: no sightings, no joins, no raids, no sweep.
   - Entries made earlier stay, and are forgotten by age as soon as they're read again.
   - Shortening `span` live forgets the older entries on their next read.
   - Owned by Task 2.
5. **Configs without `watching` load as before.** Reset-only fields are reset-only. With `cheaters` and
   `watchers` at the same share, the two sets are exactly the same agents. Owned by Task 1.

---

### Task 1: Config

**Files:** `config.rs`, `legacy.rs`.

**Interfaces:**
```rust
pub struct Watching { pub on: bool, pub span: u32, pub watchers: f64, pub raid_when: RaidWhen }
pub enum RaidWhen { Always, Hungry }   // serde lowercase
impl Watching { pub fn founder_watches(&self, id: u64) -> bool }
pub(crate) fn dealt_by_id(share: f64, id: u64) -> bool  // the ⌊i·s⌋ rule; Theft::founder_cheats calls it too
```
- `Watching` defaults to `{ on: false, span: 7, watchers: 1.0, raid_when: Always }`, with
  `#[serde(default)]`. It is held at `Config.watching`.
- **Live:** `watching.on`, `watching.span`, `watching.raid_when`.
- **Reset-only:** `watching.watchers`, and the whole `watching` object, as Minds 6 did for `theft`.

- [ ] **Step 1: Write failing tests**, following Minds 6's Task 1 tests:
  - configs without `watching` load with the defaults;
  - every validation rule and error field in the Global Constraints;
  - the live/reset-only split;
  - `founder_watches` gives exact proportions at s = 0.25, 0.5 and 1/3;
  - at equal shares `founder_watches` and `founder_cheats` agree for ids 1..=200 (Review Focus 5).
- [ ] **Step 2: Run them and check they fail.**
- [ ] **Step 3: Implement.** Include `legacy.rs`'s explicit literal.
- [ ] **Step 4:** run `cargo test -p sugarscape-core --lib config && cargo test -p sugarscape-core --test legacy --test golden`.
- [ ] **Step 5: Commit** ("Minds 8: the watching config").

---

### Task 2: Watchers, sighting and the seen-cache memory

**Files:**
- `agent.rs`:
  - `Agent.watches: bool`, set at founding from `founder_watches(id)`, and false at birth.
  - `Agent.seen: BTreeMap<(u32, AgentId), SeenCache>`, keyed (site, owner).
  - Neither field is hashed into fingerprints. Check how `cheater` is handled and match it, so goldens don't
    move.
- `minds/caching/watching.rs` (new) and `minds/caching/mod.rs` (`bury` calls `watching::see` after
  `theft::note`; `pub mod watching`).
- `world.rs`: `TickEvents` gets `burials_seen: u32`, `sightings: u32` and `seen_entries: u32`, plus the
  tick-start sweep in the step where Minds 6 counts `pilfer_candidates`.

**Interfaces:**
```rust
pub struct SeenCache { pub amount: f64, pub tick: u64 }
/// `owner` has just buried `q` at site `site` (its own position).
pub(crate) fn see(world: &mut World, owner: AgentId, site: u32, q: f64);
/// Drops every entry older than `span` (now − tick > span), from every agent.
pub(crate) fn sweep(world: &mut World);
/// Whether an entry seen at `tick` is still remembered now.
pub(crate) fn fresh(world: &World, tick: u64) -> bool;
```

**Behavior (spec "Seeing a burial"):**
- `see` returns at once unless `watching.on`.
- Otherwise every living agent w ≠ owner with `w.watches`, whose sight covers the site, adds q to its
  entry for (site, owner) and sets the entry's tick to now.
- "Sight covers" means the site is in `world.sight(w.pos, w.vision)`. Implement it by walking the four
  lattice lines out from the burial site, up to the largest vision any agent has, stopping at an opaque wall
  as `sight_until` does. Each occupant met at distance d ≤ its own vision sees the burial. Visit the
  watchers in id order.
- Write a test that this walk gives the same set as checking `world.sight(w.pos, w.vision)` for every agent.
  Run it on random small walled tori and seeds, including a torus small enough that the lines wrap.
- `burials_seen` counts the calls with at least one watcher; `sightings` counts (watcher, burial) pairs.
- `sweep` runs only while `watching.on`. After it, `seen_entries` is the total entries held.

**Tests:**
- sighting on each lattice line;
- sighting at exactly the watcher's vision, and none one beyond;
- an opaque wall between them stops the sighting, and a fence doesn't;
- no self-sighting;
- non-watchers never see;
- a second seen burial adds its amount and refreshes the tick;
- the span boundary: kept at exactly `span`, gone at `span` + 1;
- Review Focus 4: off then on, and `span` shortened live;
- a dead watcher's entries are gone with it;
- the default golden is unchanged, and with `on` false nothing is allocated in `seen`.

**Commit** ("Minds 8: watchers see burials and remember them").

---

### Task 3: Raids

**Files:** `minds/caching/watching.rs`, `minds/caching/mod.rs`, `rules/movement.rs`, `world.rs`.

**Interfaces:**
```rust
/// Adds `id`'s remembered caches to rule M's candidates (after its own caches, before Minds 3's remembered sites).
pub(crate) fn join_seen(world: &World, id: AgentId, out: &mut Vec<(Pos, u32, f64)>, start: &mut usize);
/// What `id` believes is buried at `p` by others it saw (summed over owners), when seen caches are candidates now.
pub(crate) fn seen_value(world: &World, id: AgentId, p: Pos) -> Option<f64>;
/// On arrival: raid a remembered cache at `site`. `None` = nothing taken, so go on to stumbling.
pub(crate) fn raid(world: &mut World, id: AgentId, site: u32, room: f64) -> Option<Harvest>;
```

**Behavior (spec "Going to a seen cache" and "Arriving"):**
- **`join_seen`** mirrors `join_caches`.
  - It skips the same sites: a wall, a site walled apart from the agent, a site another agent stands on,
    and the target the agent's last walk found no path to.
  - It merges with an existing entry by the larger value, in the same way.
  - It runs only while `watching.on` and the agent has fresh entries. Under `raid_when: hungry`, it also
    requires holdings < R / 2. That is `reserve(world, id) / 2`, without `hungry()`'s requirement that the
    agent has caches of its own.
  - Call it in both places `join_caches` is called in `movement.rs`, right after.
- **`true_value`** (the Minds 3 diagnostic) also takes `seen_value` into account, by the larger value, the
  way it does `cache_value`.
- **In `go_and_gather`:** after the dig branch and before the `find > 0` stumble, when `watching.on`, call
  `raid`.
  - It forgets the agent's entries at the site, counting `seen_arrivals` once if there were any.
  - It takes from the first remembered owner, in owner-id order, whose cache is still at the site. The take
    goes through `theft::pilfer` with the same `loot` and `room` handling as `stumble`: `keep` goes to
    holdings, and `eat` goes to `fed` and `loot_eaten`.
  - It counts `raids` and adds the take to `raided`.
  - If none of the remembered caches is there, that is a wasted raid (`raids_wasted`), and it returns
    `None`.
  - A take of 0 (no room under `keep`) returns `None` and counts neither raid nor waste (Review Focus 1).
  - Make sure the stumble's `foreign` count and `pilfer_draws` aren't changed by a raid that took
    something: the arrival ends there, as Minds 6's take does.
- **New events:** `raids: u32`, `raided: f64`, `raids_wasted: u32`, `seen_arrivals: u32`.

**Tests:**
- a seen cache still there is always taken;
- an unseen cache is taken only through `find`; with `find` 0 it is never taken;
- the dig wins over a raid;
- one take per arrival, even with a raid and a stumble possible on one site;
- a wasted raid falls through to stumbling;
- Review Focus 1, 2 and 3;
- `raid_when: hungry` gating: a fed watcher's list has no seen cache, a hungry watcher's does, and the
  watcher needn't have caches;
- conservation over 300 ticks with watching on, under both loot rules, with `find` 0 and 0.25 and with
  deaths;
- watching on with every agent a non-watcher (`watchers` 0) gives exactly the run without watching (the same
  fingerprint);
- the default golden is unchanged.

**Commit** ("Minds 8: raids on seen caches").

---

### Task 4: Series

**`stats.rs`** (optional, present only while `watching.on`; follow `TheftStats` and `CheaterStats`):
- `WatchStats { raids: u32, raided: f64, raids_wasted: u32, seen_arrivals: u32, burials_seen: u32, sightings: u32, seen_entries: u32 }`, per tick.
- `WatcherStats { watcher_alive, other_alive, watcher_holdings, other_holdings }`, the means and counts as in
  `CheaterStats`. Present only when both groups have founders, which means 0 < `watchers` < 1.

**Tests. Commit.**

---

### Task 5: Presets, titles drafts, goldens and sweeps

**Step 1: Run the numbers.** This is a sanity check, not a tuning step, since there is nothing to tune.
- Build the six presets below and run each for 200 ticks over seeds 1–5.
- Record in the report: the pilferage rate, raids, wasted raids, sightings per burial, and survival.
- If any preset has no raid in any seed, report BLOCKED with the numbers. Don't change parameters to fix it.

**Presets** (`presets.rs`, ids and worlds from the spec's table). Each `source` is "Bugnyar & Kotrschal
2002; Heinrich & Pepper 1998; Minds 8", adding "Andersson & Krebs 1978" to `watch-half` and "Barnard & Sibly
1981" to the two scrounger presets.
- `watch-winter`: `theft_winter(c, 0.0)`, then `find` 0 and watching on.
- `watch-winter-stumble`: the same with `find` 0.25 (`THEFT_FIND`).
- `watch-half`: `theft_winter(c, 0.5)` with watching on.
- `watch-scroungers`: `theft_winter(c, 0.0)`, `find` 0, watching on, `watchers` 0.5.
- `watch-scroungers-only`: the same with `cheaters` 0.5.
- `watch-arena`: `theft_arena(c, 4)`, `cheaters` 0, watching on, `watchers` 0.5.

**Descriptions and titles:**
- Descriptions describe the setup only; Task 8 measures them.
- Title drafts: "Agents who watch others bury", "Watching, and stumbling on caches too", "Half the agents
  never cache, and everyone watches", "Half the agents watch others bury", "Half the agents only watch and
  steal", "Four agents in a room, half of them watching".

**Goldens:** update the counts. Record new golden entries for `watch-winter` and `watch-arena` only.

**Sweeps** (`BUILTINS` + 2, ids in `sweep.rs`'s test and the WASM list):
- `watch-span`: the pilferage rate against `span` (1, 3, 7, 13) in `watch-winter`.
- `watch-scroungers`: watcher and non-watcher survival against the watcher share 0.1–0.9.

**Commit.**

---

### Task 6: WASM

- Pin `watch-winter`'s and `watch-arena`'s fingerprints in `crates/sugarscape-wasm/tests/web.rs`.
- `AgentView.watching: Option<WatchView { watches: bool, scrounger: bool, seen: Vec<SeenView { site: u32, owner: u64, amount: f64, age: u64 }> }>`.
  - It is `None` while watching is off.
  - `scrounger` is `watches && cheater`.
  - Pass it through `inspect`.
- `edit.rs`/`render.rs`: a `Watching` color mode (watchers who bury, scroungers, others).
- Add a WASM test. Run `wasm-pack test --node crates/sugarscape-wasm`.

**Commit.**

---

### Task 7: The page

- **Rules:** a "Watching (Minds 8)" group (`minds: true`):
  - watching, a checkbox, live;
  - span, 1–30, step 1, live;
  - watchers, 0–1, step 0.05, reset;
  - raid when, a select, live.
- **A note** where both `cheaters` and `watchers` are between 0 and 1: "Watchers and cheaters are dealt by
  the same id rule: at equal shares they are the same agents."
- **Menu:** `MINDS_TITLES['8']`, plus `usesMinds` extended for watching, with tests.
- **Colors:** `layers.ts`'s `defaultColorMode` picks `watching` when `watching.on`, before the caching-rule
  check. Add a legend entry.
- **Inspect:** "Watches: yes/no", plus the remembered caches (site, owner, amount, age).
- **Charts:**
  - "Pilfers by source": seen, and stumbled = pilfers − raids.
  - "Wasted raids".
  - "Watchers vs others": alive and holdings, shown when `WatcherStats` is present.
- **CSV export:** missing groups give empty cells, not NaN (Minds 6's fix).
- **Tests.** Browser check. **Commit.**

---

### Task 8: The survey, measured descriptions and the cost table

`survey/src/claims/minds8.rs`. Follow the spec's "Questions and judges" exactly, and reuse Minds 6's helpers
as `pub(crate)`.

**The judges go in their own commit first, before any survey run.** They are:
1. `watch-winter.pilferage`;
2. `watch-half.threshold`;
3. `watch-scroungers.frequency`, for both variants;
4. `watch-winter.usage`.

Each is judged at span 7 and reported at span 1, 3, 7 and 13, with the decomposition and the reported-only
measures.

- **Then run** and write the results into `survey/out` as the other claims do.
- **Measured descriptions and titles** for all six presets: per founder too, with causes labeled "likely".
- **The cost table:** µs per agent-tick beside Minds 6's.
- **Golden unchanged.** **Commit.**

---

### Task 9: Docs

- **README:** a Minds 8 section.
- **The program document:**
  - status: Minds 1–8 done;
  - the results and the cost table;
  - the next target, Minds 9 (spatial P1b).
- **The roadmap line.**
- **The spec's amendments:** rulings made in execution, and
  anything the Task 5 numbers changed in how claims are read (not their thresholds).
- **Full suite:** cargo, clippy (`-D warnings`), fmt, web build and vitest, and wasm-pack. **Commit.**
