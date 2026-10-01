# Minds 8, second round — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give the watching model the four named switches, a scrounger world, and Andersson and Krebs's
calibrated world. Then re-test it with judges that can tell the hypotheses apart, and correct the first
round's text.

**Architecture:**
- The switches live in `Watching` (`config.rs`) and act in `minds/caching/watching.rs` (`join_seen`, `raid`),
  in founding (`world.rs`, for `who`) and in candidate building (`rules/movement.rs`, for `scrounge: forgo`).
- The survey gets a new module, `survey/src/claims/minds8b.rs`. The first round's `minds8.rs` stays, labeled
  as the first design.
- The calibration for claim 2 runs under a fixed rule, and its chosen world becomes the `watch-ak` preset.

**Tech Stack:** Rust (sugarscape-core, sugarscape-wasm), `survey/`, TypeScript and Vitest (`web/`).

**Spec:** `docs/superpowers/specs/2026-10-01-minds-8-second-round-design.md`, which amends
`docs/superpowers/specs/2026-10-01-minds-8-watching-design.md`.

## Global Constraints

- **Main's fixtures are untouchable.** Every golden entry, legacy fixture and pinned WASM fingerprint that
  exists on main stays green and **unedited**.
  - `watch-*` entries exist only on this branch and may be re-recorded where these changes move them.
  - New presets get new entries.
- **Defaults:**
  - `raid_if: better`, `value: amount`, `who: share`, `scrounge: harvest`.
  - With `watching.on` false nothing changes anywhere, bit for bit.
- **Liveness:**
  - Live: `raid_if`, `value`, `scrounge`.
  - Reset-only: `who`.
- **No draws.** None of the switches touches `world.rng`.
- **Conservation.** The Minds 6 ledger holds under every switch.
- **Judges.** The survey judges are the second-round spec's "Questions and judges", word for word.
  - They are committed alone, before any survey run, with the spec's pre-mortem table copied into the
    module doc.
  - The claim 2 calibration's rule is committed before it runs, and its result is committed before claims
    2a–2d run.
  - Thresholds are never tuned.
- **Fitness.**
  - Field worlds: agent-ticks alive per founder over ticks 1–200, divided by 200.
  - Arena worlds: wealth per founder at tick 200 (holdings + caches + stomach of the living ÷ founders).
- **Style.** Never write "Flump"; say "agent". Use American spelling. Causes are "likely" unless isolated by a
  switch. Titles follow `titles.rs` and are written from measured results.
- **Commits.** Every commit ends with `Claude-Session: https://claude.ai/code/session_01BxjiP46rwioLoJSaqLMDiJ`.
- **Where to work.** Use `/Users/nathan/Projects/ndouglas/SugarScape/.claude/worktrees/minds-8` (branch
  `minds-8`). For web tests, temporarily symlink the main checkout's `web/node_modules` and remove the link
  afterward. Run `npm run build` before vitest.

## Review Focus

1. **`raid_if: better` with the site value exactly equal to the remembered amount.** It raids (≥). When it
   declines, the agent harvests, keeps its entries, and counts nothing in `seen_arrivals`, `raids` or
   `raids_wasted`. Owned by Task 2.
2. **`value: room`.**
   - The cap is infinite with no carrying limit, and under `loot: eat`.
   - With room 0 the seen cache's value is 0, so it never out-ranks a site.
   - Owned by Task 2.
3. **`who: hoarders` or `who: cheaters` with no cheaters configured.**
   - `hoarders` makes everyone watch.
   - `cheaters` makes nobody watch.
   - `watchers` is ignored, and the page says so.
   - Owned by Tasks 1 and 2.
4. **`scrounge: forgo`.**
   - A scrounger holding fresh entries that are all unreachable or occupied has only staying put. It gathers
     nothing that tick but still pays metabolism. With no fresh entry it forages as usual.
   - Non-scroungers are unaffected.
   - Owned by Task 2.
5. **The calibration can find no qualifying world.**
   - Claims 2a–2d are then reported Untestable, and `watch-ak` is not created. The page and docs say why.
   - Owned by Tasks 4 and 6.

---

### Task 1: The switches' config

**Files:** `config.rs`, `legacy.rs`.

**Interfaces:**
```rust
pub enum RaidIf { Better, Always }        // serde lowercase; default Better
pub enum SeenValue { Amount, Room }       // serde lowercase; default Amount
pub enum Who { Share, Hoarders, Cheaters } // serde lowercase; default Share
pub enum Scrounge { Harvest, Forgo }      // serde lowercase; default Harvest
// Watching gains: raid_if: RaidIf, value: SeenValue, who: Who, scrounge: Scrounge
impl Watching { pub fn founder_watches(&self, id: u64, cheats: bool) -> bool } // share → dealt_by_id; hoarders → !cheats; cheaters → cheats
```
- Every caller of `founder_watches` passes the founder's cheater flag.

- [ ] **Step 1: Write failing tests:**
  - configs without the new fields load with the defaults;
  - the liveness split;
  - `founder_watches` under each `who`, including Review Focus 3.
- [ ] **Step 2: Run them and check they fail.**
- [ ] **Step 3: Implement.** Include `legacy.rs`'s literal.
- [ ] **Step 4:** run `cargo test -p sugarscape-core --lib config && cargo test -p sugarscape-core --test legacy --test golden`.
- [ ] **Step 5: Commit** ("Minds 8b: the watching switches' config").

---

### Task 2: The switches' mechanics

**Files:** `minds/caching/watching.rs`, `rules/movement.rs`, `world.rs` (founding), tests.
`crates/sugarscape-core/tests/golden.rs` and `crates/sugarscape-wasm/tests/web.rs`: re-record the `watch-*`
entries only.

**Behavior** (the second-round spec, "Named switches"):
- **`raid_if: better`.** In `raid`, before forgetting anything:
  - sum the remembered amounts at the site over the agent's fresh entries;
  - get the site's welfare value as rule M counts it, the same quantity `go_and_gather`'s dig test uses
    (`site_value`);
  - if the remembered sum is below the site value, return `None` without forgetting or counting. The agent
    harvests as usual.
  - Equal raids.
- **`value: room`.** In `join_seen`, cap each site's summed value at the agent's room under the carrying
  limit. The cap is infinite with no limit, or under `loot: eat`.
- **`who`.** At founding, `watches = founder_watches(id, cheater)`. Make sure the cheater flag is set before
  this.
- **`scrounge: forgo`.** For a scrounger (`watches && cheater`) holding at least one fresh entry:
  - its candidate list is its seen caches (as `join_seen` adds them, with the same skips) plus its own site;
  - if it stays put, it gathers nothing that tick.

  Implement this in `candidates_with_memory`, or where its list is consumed, with the fewest touch points.
  Document the choice.

**Tests:**
- each switch's behavior;
- Review Focus 1–4;
- conservation over 300 ticks under each switch (`loot` keep and eat);
- watching off is identical to main's worlds (the existing golden suite);
- `raid_if: always` reproduces the first round's `watch-winter` fingerprint, as recorded at the branch's
  current head before this task (record it in the test).

**Commit** ("Minds 8b: raid only when better, value at room, who watches, scroungers who forgo").

---

### Task 3: The scrounger preset, WASM and page

- **Preset.** `watch-scroungers-forgo` (`presets.rs`):
  - `theft_winter(c, 0.5)`, then `find` 0;
  - watching on, `watchers` 0.5, `scrounge: forgo`, `theft.loot: eat`.
  - Its source names "Bugnyar & Kotrschal 2002; Barnard & Sibly 1981; Vickery et al. 1991; Minds 8".
  - Write a draft title and a setup-only description. Update the counts, add a golden entry, and pin it in
    `web.rs`.
- **WASM.** Expose the new fields through the config. Extend `WatchView`'s inspector only if needed. Nothing
  else changes.
- **Page.** The "Watching (Minds 8)" rules group gets:
  - "raid if" (select: better / always, live);
  - "seen cache value" (select: amount / room, live);
  - "who watches" (select: share / hoarders / cheaters, reset);
  - "scroungers" (select: harvest / forgo, live).
- **A note** when `who` ≠ share: "Who watches is set by kind; the watcher share is ignored."
- **Tests.** vitest for the schema and the note.
- **Commit.**

---

### Task 4: Fitness, and the claim 2 calibration

**Files:** `survey/src/claims/minds8b.rs` (new; register it), and `presets.rs` and `titles.rs` (only if a
world qualifies).

**Steps:**
- [ ] **Step 1:** Implement the fitness measures from the Global Constraints, with unit tests on hand-built
  runs.
- [ ] **Step 2:** Implement the calibration exactly as the spec's "precondition" states.
  - The fixed list, in order.
  - p_s and p_o as in Minds 6.
  - The probe `probe_dig_at_reserve` for items 4 and 5.
  - Its rule: qualifies when p_s > p_o in ≥ 16 of 20 seeds.

  **Commit the code alone** ("Minds 8b: claim 2's calibration rule, before it runs").
- [ ] **Step 3:** Run it.
  - Commit the output, every list item's per-seed p_s, p_o and qualifying count, as a tracked file at
    `survey/out/minds8b-calibration.md`.
  - If a world qualifies, add `watch-ak`: that world with watching on, everyone watching (`who: share`,
    `watchers` 1). Give it a draft title, a golden entry and a pin in `web.rs`.
  - If none qualifies, record that. Claims 2a–2d will be Untestable (Review Focus 5).

  **Commit** ("Minds 8b: calibration result").

---

### Task 5: The second-round judges (committed before any run)

**File:** `survey/src/claims/minds8b.rs`.
- Implement claims 1a, 2a, 2b, 2c, 2d, 3a (two ids), 3b and 3c, the reported-only measures and the usage
  check, word for word from the spec.
- Copy the pre-mortem table into the module doc.
- **Measures:**
  - The cohort hazard comes from the fate log, from caches first created in ticks 1–90.
  - The bottleneck rule.
  - Claim 3 uses seeds 1–60.
  - The Inconclusive / Flat verdicts follow the spec's 3a, using the existing outcome types. If none fits
    "Inconclusive", add it in the claim-outcome code the way Weak is, and document it.
- Reuse the Minds 6 and Minds 8 helpers as `pub(crate)`.
- Unit tests for every judge's decision logic, including edge cases.
- **Commit alone** ("Minds 8b: second-round judges, committed before any run"), before running anything.

---

### Task 6: Run, measured descriptions and titles

- Run the second-round claims.
  - Record the outputs (the tracked files, plus `survey/out/minds8b-results.md` with every claim's table).
  - Report every claim under `raid_if: always`, `value: room` and spans 1, 2, 3, 7 and 13, and the
    first-round probe.
- **Re-measure every `watch-*` preset under the new defaults.** Rewrite its measured description and title
  from these results: per founder too, and causes "likely" unless a switch isolated them.
- Golden unchanged by this task. Run the workspace tests, clippy, fmt, the web build and vitest.
- **Commit.**

---

### Task 7: Docs and corrections

- **The first round's text,** per the spec's "Corrections to the first round's text": replace the listed
  overstatements in:
  - the README;
  - the program doc (`docs/studies/2026-09-27-minds.md`);
  - the first-round spec's amendments;
  - the Minds 9 rationale.
- **The first round's verdict table** stays in the program doc. Label it as the first design's, with the
  audits' explanation and a pointer to the second round.
- **The program doc's second-round results:**
  - the claims and their verdicts;
  - the calibration;
  - the switch table;
  - the bottleneck classification;
  - an upper-bound framing (no hiding, no defense, generous memory);
  - a lesson paragraph (the pre-mortem).
- **The second-round spec gets an "Amendments (implementation)" section** with every execution ruling.
- **README:** update the Minds 8 section.
- **Full suite:** cargo, clippy, fmt, wasm-pack, web build and vitest, and the book job.
- **Commit.**
